#!/usr/bin/env python3
"""Run optimizer-free causal counterfactuals over the exact TRAIN-4 audit sweep."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import traceback
from collections import Counter
from pathlib import Path
from typing import Any, Sequence

from isaaclab.app import AppLauncher


SCRIPTED_MODES = (
    "zero-residual",
    "root-link-reset-velocity",
    "contact-projected-root-link-reset-velocity",
    "zero-joint-reset-velocity",
    "zero-root-reset-velocity",
    "zero-all-reset-velocity",
    "lead-1",
    "lead-4",
    "lead-6",
    "velocity-feedforward",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=120_812)
    parser.add_argument(
        "--modes", nargs="+", choices=SCRIPTED_MODES, default=SCRIPTED_MODES
    )
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if (
        args.seed < 0
        or not args.device.startswith("cuda:")
        or "zero-residual" not in args.modes
        or len(args.modes) != len(set(args.modes))
    ):
        raise ValueError("invalid causal-probe configuration")
    paths = (
        args.source_audit,
        args.descriptor,
        args.profile,
        args.gate_report,
        args.usd,
    )
    for path in paths:
        if not path.resolve().is_file():
            raise FileNotFoundError(path)
    corpus_root = args.corpus_root.resolve()
    if not corpus_root.is_dir():
        raise FileNotFoundError(corpus_root)
    manifest_path = corpus_root / "corpus-manifest.json"
    if not manifest_path.is_file():
        raise FileNotFoundError(manifest_path)
    output = _external_output(args.output)
    source_audit = json.loads(args.source_audit.resolve().read_bytes())
    source_scope = source_audit.get("scope", {})
    source_rows = source_audit.get("results", {}).get("phase_results", [])
    horizon = int(source_scope.get("horizon_motor_ticks", 0))
    repeats = int(source_scope.get("repeat_count_per_start_phase", 0))
    requested_num_envs = int(source_scope.get("requested_num_envs", 0))
    reset_window = int(
        source_audit.get("results", {})
        .get("overall", {})
        .get("reset_safety_window_motor_ticks", 1)
    )
    clip_inventory = tuple(
        (
            item["clip_id"],
            item["split"],
            int(item["reference_frame_count"]),
        )
        for item in source_scope.get("clip_inventory", ())
    )
    if (
        source_audit.get("check")
        != "TRAIN-4-ISAAC-EXHAUSTIVE-DYNAMIC-REFERENCE-FEASIBILITY"
        or horizon <= 0
        or repeats <= 0
        or requested_num_envs <= 0
        or not clip_inventory
        or not source_rows
    ):
        raise ValueError("source audit is not a complete TRAIN-4 dynamic sweep")

    from next_lab.reference_dynamic_feasibility import build_dynamic_audit_cases

    cases = build_dynamic_audit_cases(
        clips=clip_inventory,
        horizon_motor_ticks=horizon,
        repeats=repeats,
    )
    _validate_source_scope(cases, source_rows, source_scope)
    effective_num_envs = min(requested_num_envs, len(cases))
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(args.usd.resolve())
    simulation_app = None
    environment = None
    failure_pending = False
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch

        from next_lab.isaac_reference_env import (
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
        )

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = effective_num_envs
        cfg.sim.device = args.device
        cfg.seed = args.seed
        cfg.eligible_clip_ids = tuple(item[0] for item in clip_inventory)
        cfg.phase_randomization = False
        cfg.fixed_horizon_motor_ticks = horizon
        cfg.diagnostic_exhaustive_phase_sweep_repeats = repeats
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.profile.resolve()),
            corpus_root=str(corpus_root),
            gate_report_path=str(args.gate_report.resolve()),
        )
        environment_inventory = tuple(
            (clip.clip_id, clip.split, clip.frame_count)
            for clip in environment.reference_clips
        )
        if environment_inventory != clip_inventory:
            raise RuntimeError(
                "environment clip inventory differs from the source audit"
            )
        expected_schedule = tuple(
            (
                case.clip_index,
                case.start_frame,
                case.terminal_frame,
                case.repeat_index,
            )
            for case in cases
        )
        if environment.diagnostic_episode_schedule != expected_schedule:
            raise RuntimeError(
                "environment schedule differs from the source audit schedule"
            )

        action_channel_ids = tuple(
            environment.reference_profile.document["action"][
                "ordered_actuator_ids"
            ]
        )
        contact_channel_ids = tuple(
            environment.reference_profile.document["observation"]["contact_order"]
        )
        mode_results: dict[str, Any] = {}
        baseline_validation: dict[str, Any] | None = None
        for mode in args.modes:
            sections, executed_steps = _run_mode(
                environment=environment,
                cases=cases,
                source_rows=source_rows,
                mode=mode,
                horizon=horizon,
                effective_num_envs=effective_num_envs,
                reset_window=reset_window,
                action_channel_ids=action_channel_ids,
                contact_channel_ids=contact_channel_ids,
                torch=torch,
            )
            if mode == "zero-residual":
                baseline_validation = _validate_baseline(
                    sections["phase_results"], source_rows
                )
            mode_results[mode] = _compact_mode_result(
                sections=sections,
                source_rows=source_rows,
                executed_vector_motor_steps=executed_steps,
            )
        if baseline_validation is None:
            raise RuntimeError("zero-residual validation was not executed")

        report = {
            "schema_version": 1,
            "check": "TRAIN-4-ISAAC-REFERENCE-CAUSAL-PROBE",
            "status": "PASS",
            "claim": "OptimizerFreeCounterfactualResearchOnly",
            "gate_decision": "NO_CHANGE",
            "research_questions": [
                "whether current-frame target indexing creates avoidable dynamic lag",
                "whether fixed-PD damping requires deterministic reference-velocity feedforward",
                "whether exact-reference joint or root reset velocity causes the failures",
                "whether retargeted root-link velocity was written with center-of-mass semantics",
            ],
            "method": {
                "source_schedule_replayed_in_full": True,
                "source_requested_num_envs_preserved": True,
                "source_asynchronous_reassignment_preserved": True,
                "source_accumulator_reused": True,
                "baseline_acceptance_rule": (
                    "every case must match source status, safety reasons, and terminal tick"
                ),
                "counterfactuals_change_one scripted-controller or reset factor at a time": True,
            },
            "baseline_validation": baseline_validation,
            "scope": {
                "case_count": len(cases),
                "clip_count": len(clip_inventory),
                "horizon_motor_ticks": horizon,
                "repeat_count_per_start_phase": repeats,
                "requested_num_envs": requested_num_envs,
                "effective_num_envs": effective_num_envs,
                "modes": list(args.modes),
            },
            "identities": {
                "source_audit_sha256": _sha256(args.source_audit.resolve()),
                "descriptor_sha256": _sha256(args.descriptor.resolve()),
                "profile_sha256": _sha256(args.profile.resolve()),
                "corpus_manifest_file_sha256": _sha256(manifest_path),
                "gate_report_sha256": _sha256(args.gate_report.resolve()),
                "usd_sha256": _sha256(args.usd.resolve()),
                "tool_sha256": _sha256(Path(__file__).resolve()),
            },
            "modes": mode_results,
            "optimizer_steps": 0,
            "training_runs": 0,
            "learned_policy_claim": False,
            "repository": _repository_state(),
        }
        _write_report(output, report)
        print(json.dumps(_console_summary(report, output), indent=2, sort_keys=True))
    except BaseException:
        failure_pending = True
        traceback.print_exc()
        raise
    finally:
        cleanup_failed = False
        if environment is not None:
            try:
                environment.close()
            except BaseException:
                cleanup_failed = True
                traceback.print_exc()
        if simulation_app is not None:
            if failure_pending or cleanup_failed:
                sys.stdout.flush()
                sys.stderr.flush()
                os._exit(1)
            simulation_app.close()
        if cleanup_failed:
            raise RuntimeError("causal-probe environment cleanup failed")


def _run_mode(
    *,
    environment: Any,
    cases: Sequence[Any],
    source_rows: Sequence[dict[str, Any]],
    mode: str,
    horizon: int,
    effective_num_envs: int,
    reset_window: int,
    action_channel_ids: Sequence[str],
    contact_channel_ids: Sequence[str],
    torch: Any,
) -> tuple[dict[str, Any], int]:
    from next_lab.isaac_reference_env import ACTION_CHANNELS
    from next_lab.reference_dynamic_feasibility import (
        CONTACT_SAFETY_REASONS,
        REQUIRED_SAFETY_REASONS,
        DynamicFeasibilityAccumulator,
    )

    accumulator = DynamicFeasibilityAccumulator(
        cases=cases,
        action_channel_ids=action_channel_ids,
        contact_channel_ids=contact_channel_ids,
        contact_pair_ids=environment.contact_pair_ids,
        reset_safety_window_motor_ticks=reset_window,
    )
    environment.reset_diagnostic_episode_sequence()
    observations, _ = environment.reset()
    if not torch.isfinite(observations["policy"]).all():
        raise RuntimeError(f"{mode} reset observation is non-finite")
    if mode in {
        "root-link-reset-velocity",
        "contact-projected-root-link-reset-velocity",
        "zero-joint-reset-velocity",
        "zero-root-reset-velocity",
        "zero-all-reset-velocity",
    }:
        _apply_reset_counterfactual(
            environment, environment.robot._ALL_INDICES, mode=mode, torch=torch
        )
    zero = torch.zeros(
        (effective_num_envs, ACTION_CHANNELS),
        dtype=torch.float32,
        device=environment.device,
    )
    executed_steps = 0
    waves = (len(cases) + effective_num_envs - 1) // effective_num_envs
    maximum_steps = (waves + 1) * (horizon + 1)
    while accumulator.completed_case_count < accumulator.case_count:
        actions = (
            zero
            if mode
            in {
                "zero-residual",
                "root-link-reset-velocity",
                "contact-projected-root-link-reset-velocity",
                "zero-joint-reset-velocity",
                "zero-root-reset-velocity",
                "zero-all-reset-velocity",
            }
            else _scripted_action(environment, mode, torch)
        )
        observations, _, terminated, truncated, _ = environment.step(actions)
        executed_steps += 1
        if not torch.isfinite(observations["policy"]).all():
            raise RuntimeError(f"{mode} step observation is non-finite")
        done = terminated | truncated
        tensors = _step_tensors_to_cpu(
            environment=environment,
            done=done,
            truncated=truncated,
            safety_reasons=REQUIRED_SAFETY_REASONS,
            contact_safety_reasons=CONTACT_SAFETY_REASONS,
        )
        for env_index in range(effective_num_envs):
            assignment_ordinal = int(tensors["assignment"][env_index].item())
            if assignment_ordinal >= len(cases):
                continue
            accumulator.record_step(
                assignment_ordinal=assignment_ordinal,
                clip_index=int(tensors["clip_index"][env_index].item()),
                start_frame=int(tensors["start_frame"][env_index].item()),
                reference_frame=int(tensors["reference_frame"][env_index].item()),
                elapsed_motor_ticks=int(tensors["elapsed_ticks"][env_index].item()),
                done=bool(tensors["done"][env_index].item()),
                success=bool(tensors["success"][env_index].item()),
                failure=bool(tensors["failure"][env_index].item()),
                truncated=bool(tensors["truncated"][env_index].item()),
                safety={
                    reason: bool(tensors["safety"][reason][env_index].item())
                    for reason in REQUIRED_SAFETY_REASONS
                },
                tracking_lost=bool(tensors["tracking_lost"][env_index].item()),
                root_position_error_micrometres=int(
                    tensors["root_position_error"][env_index].item()
                ),
                root_orientation_absolute_dot_q1_30=int(
                    tensors["root_orientation_dot"][env_index].item()
                ),
                observed_joint_position_microradians=_integer_row(
                    tensors["observed_joint_position"], env_index
                ),
                reference_joint_target_microradians=_integer_row(
                    tensors["reference_joint_position"], env_index
                ),
                hard_rom_excess_microradians=_integer_row(
                    tensors["hard_rom_excess"], env_index
                ),
                velocity_excess_microradians_per_second=_integer_row(
                    tensors["velocity_excess"], env_index
                ),
                effort_envelope_violation=_boolean_row(
                    tensors["effort_envelope_channels"], env_index
                ),
                observed_contacts=_integer_row(tensors["observed_contacts"], env_index),
                reference_contacts=_integer_row(
                    tensors["reference_contacts"], env_index
                ),
                contact_pair_masks={
                    reason: _boolean_row(
                        tensors["contact_pair_masks"][reason], env_index
                    )
                    for reason in CONTACT_SAFETY_REASONS
                },
                contact_pair_maximum_impulses_micronewton_seconds=_integer_row(
                    tensors["contact_pair_impulses"], env_index
                ),
                forbidden_contact_mask=int(
                    tensors["forbidden_contact_mask"][env_index].item()
                ),
            )
        if mode in {
            "root-link-reset-velocity",
            "contact-projected-root-link-reset-velocity",
            "zero-joint-reset-velocity",
            "zero-root-reset-velocity",
            "zero-all-reset-velocity",
        }:
            reset_ids = torch.nonzero(done, as_tuple=False).flatten()
            if reset_ids.numel():
                _apply_reset_counterfactual(
                    environment, reset_ids, mode=mode, torch=torch
                )
        if executed_steps > maximum_steps:
            raise RuntimeError(f"{mode} exceeded its progress bound")
        if (
            executed_steps % 100 == 0
            or accumulator.completed_case_count == accumulator.case_count
        ):
            print(
                f"causal-probe {mode}: {accumulator.completed_case_count}/{len(cases)}",
                file=sys.stderr,
                flush=True,
            )
    sections = accumulator.report_sections()
    if len(sections["phase_results"]) != len(source_rows):
        raise RuntimeError(f"{mode} produced an incomplete phase result set")
    return sections, executed_steps


def _scripted_action(environment: Any, mode: str, torch: Any) -> Any:
    current_frame = environment._cursor
    current = environment._reference_at("joint_position_urad", current_frame)[
        :, environment._action_to_dof
    ].to(torch.float64)
    if mode.startswith("lead-"):
        lead = int(mode.removeprefix("lead-"))
        future_frame = torch.minimum(
            current_frame + lead,
            environment._reference_lengths[environment._clip_index] - 1,
        )
        desired = environment._reference_at("joint_position_urad", future_frame)[
            :, environment._action_to_dof
        ].to(torch.float64)
    elif mode == "velocity-feedforward":
        velocity = environment._reference_at(
            "joint_velocity_urad_s", current_frame
        )[:, environment._action_to_dof].to(torch.float64)
        desired = current + (
            environment._damping_q16 / environment._stiffness_q16
        ) * velocity
    else:
        raise ValueError(f"unknown scripted mode: {mode}")
    return torch.clamp(
        (desired - current) / environment._residual_scale, -1.0, 1.0
    ).to(torch.float32)


def _zero_reset_velocity(
    environment: Any,
    env_ids: Any,
    *,
    zero_joint: bool,
    zero_root: bool,
    torch: Any,
) -> None:
    if not zero_joint and not zero_root:
        raise ValueError("reset-velocity counterfactual changes no velocity")
    if zero_root:
        root_state = environment.robot.data.root_state_w[env_ids].clone()
        root_state[:, 7:13] = 0.0
        environment.robot.write_root_state_to_sim(root_state, env_ids)
    if zero_joint:
        joint_position = environment.robot.data.joint_pos[env_ids][
            :, environment._dof_joint_ids
        ].clone()
        joint_velocity = torch.zeros_like(joint_position)
        environment.robot.write_joint_state_to_sim(
            joint_position,
            joint_velocity,
            joint_ids=environment._dof_joint_ids,
            env_ids=env_ids,
        )
    environment._canonical_cache = None


def _apply_reset_counterfactual(
    environment: Any, env_ids: Any, *, mode: str, torch: Any
) -> None:
    if mode == "root-link-reset-velocity":
        _rewrite_reference_root_link_velocity(
            environment, env_ids, project_declared_contact=False, torch=torch
        )
        return
    if mode == "contact-projected-root-link-reset-velocity":
        _rewrite_reference_root_link_velocity(
            environment, env_ids, project_declared_contact=True, torch=torch
        )
        return
    _zero_reset_velocity(
        environment,
        env_ids,
        zero_joint=mode != "zero-root-reset-velocity",
        zero_root=mode != "zero-joint-reset-velocity",
        torch=torch,
    )


def _rewrite_reference_root_link_velocity(
    environment: Any,
    env_ids: Any,
    *,
    project_declared_contact: bool,
    torch: Any,
) -> None:
    frame = environment._episode_start_frame[env_ids]
    root_linear_engine = (
        environment._reference_at(
            "root_linear_velocity_um_s", frame, env_ids
        ).to(torch.float32)
        / 1_000_000.0
    )
    if project_declared_contact:
        root_linear_engine -= _declared_contact_point_velocity(
            environment, env_ids, frame, torch
        )
    root_linear = _engine_to_isaac_vector(root_linear_engine, torch)
    yaw = (
        environment._reference_at(
            "root_yaw_velocity_urad_s", frame, env_ids
        ).to(torch.float32)
        / 1_000_000.0
    )
    root_angular_engine = torch.stack(
        (torch.zeros_like(yaw), yaw, torch.zeros_like(yaw)), dim=-1
    )
    root_angular = _engine_to_isaac_vector(root_angular_engine, torch)
    root_link_velocity = torch.cat((root_linear, root_angular), dim=-1)
    environment.robot.write_root_link_velocity_to_sim(root_link_velocity, env_ids)
    environment._canonical_cache = None


def _declared_contact_point_velocity(
    environment: Any, env_ids: Any, frame: Any, torch: Any
) -> Any:
    clip_index = environment._clip_index[env_ids]
    previous_frame = torch.clamp(frame - 1, min=0)
    next_frame = torch.minimum(
        frame + 1, environment._reference_lengths[clip_index] - 1
    )
    interval = (next_frame - previous_frame).to(torch.float64)
    previous = environment._reference_at(
        "effector_position_um", previous_frame, env_ids
    ).to(torch.float64)
    following = environment._reference_at(
        "effector_position_um", next_frame, env_ids
    ).to(torch.float64)
    velocity = (following - previous) * (60.0 / interval[:, None, None])
    contacts = environment._reference_at("contacts", frame, env_ids)[:, :2].bool()
    selected: list[Any] = []
    row = torch.arange(len(env_ids), device=environment.device)
    for pair in ((0, 1), (3, 4)):
        candidates = velocity[:, pair]
        choice = torch.argmin(torch.linalg.vector_norm(candidates, dim=-1), dim=-1)
        selected.append(candidates[row, choice])
    selected_velocity = torch.stack(selected, dim=1)
    count = torch.sum(contacts, dim=1, keepdim=True)
    return torch.where(
        count > 0,
        torch.sum(selected_velocity * contacts[:, :, None], dim=1)
        / torch.clamp(count, min=1),
        torch.zeros_like(selected_velocity[:, 0]),
    ).to(torch.float32) / 1_000_000.0


def _engine_to_isaac_vector(value: Any, torch: Any) -> Any:
    x, y, z = value.unbind(dim=-1)
    return torch.stack((x, -z, y), dim=-1)


def _compact_mode_result(
    *,
    sections: dict[str, Any],
    source_rows: Sequence[dict[str, Any]],
    executed_vector_motor_steps: int,
) -> dict[str, Any]:
    phase_rows = sections["phase_results"]
    transitions: Counter[str] = Counter()
    source_failure_transitions: Counter[str] = Counter()
    mode_failure_signatures: Counter[str] = Counter()
    changed_cases: list[dict[str, Any]] = []
    cases_of_interest: list[dict[str, Any]] = []
    outcome_payload: list[tuple[str, tuple[str, ...], int]] = []
    for source, mode in zip(source_rows, phase_rows, strict=True):
        source_outcome = _outcome(source)
        mode_outcome = _outcome(mode)
        transitions[f"{source_outcome['status']}->{mode_outcome['status']}"] += 1
        if source_outcome["status"] == "FAIL":
            source_signature = "+".join(source_outcome["reasons"])
            source_failure_transitions[
                f"{source_signature}->{mode_outcome['status']}"
            ] += 1
        if mode_outcome["status"] == "FAIL":
            mode_failure_signatures["+".join(mode_outcome["reasons"])] += 1
        if source_outcome != mode_outcome:
            changed_cases.append(
                {
                    "case_ordinal": int(source["case_ordinal"]),
                    "clip_id": source["clip_id"],
                    "start_frame": int(source["start_frame"]),
                    "source": source_outcome,
                    "mode": mode_outcome,
                }
            )
        if source_outcome["status"] == "FAIL" or mode_outcome["status"] == "FAIL":
            cases_of_interest.append(
                {
                    "case_ordinal": int(source["case_ordinal"]),
                    "clip_id": source["clip_id"],
                    "start_frame": int(source["start_frame"]),
                    "source": {
                        **source_outcome,
                        "violation": source["first_required_safety_violation"],
                    },
                    "mode": {
                        **mode_outcome,
                        "violation": mode["first_required_safety_violation"],
                    },
                }
            )
        outcome_payload.append(
            (
                mode_outcome["status"],
                tuple(mode_outcome["reasons"]),
                int(mode_outcome["terminal_motor_tick"]),
            )
        )
    canonical_outcomes = json.dumps(
        outcome_payload, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")
    return {
        "executed_vector_motor_steps": executed_vector_motor_steps,
        "overall": sections["overall"],
        "comparison_to_source": {
            "transition_counts": dict(sorted(transitions.items())),
            "source_failure_transition_counts": dict(
                sorted(source_failure_transitions.items())
            ),
            "mode_failure_signature_counts": dict(
                sorted(mode_failure_signatures.items())
            ),
            "changed_case_count": len(changed_cases),
        },
        "phase_outcome_sha256": hashlib.sha256(canonical_outcomes).hexdigest(),
        "changed_cases": changed_cases,
        "cases_of_interest": cases_of_interest,
    }


def _validate_baseline(
    baseline_rows: Sequence[dict[str, Any]],
    source_rows: Sequence[dict[str, Any]],
) -> dict[str, Any]:
    mismatches: list[dict[str, Any]] = []
    for source, baseline in zip(source_rows, baseline_rows, strict=True):
        source_outcome = _outcome(source)
        baseline_outcome = _outcome(baseline)
        if source_outcome != baseline_outcome:
            mismatches.append(
                {
                    "case_ordinal": int(source["case_ordinal"]),
                    "clip_id": source["clip_id"],
                    "start_frame": int(source["start_frame"]),
                    "expected": source_outcome,
                    "actual": baseline_outcome,
                }
            )
    if mismatches:
        raise RuntimeError(
            "zero-residual baseline differs from R14: "
            + json.dumps(mismatches[:8], sort_keys=True)
        )
    return {
        "status": "PASS",
        "matched_case_count": len(source_rows),
        "mismatched_case_count": 0,
        "matched_fields": ["status", "required_safety_reasons", "terminal_motor_tick"],
    }


def _outcome(row: dict[str, Any]) -> dict[str, Any]:
    violation = row["first_required_safety_violation"]
    return {
        "status": row["status"],
        "reasons": sorted(violation["reasons"]) if violation else [],
        "terminal_motor_tick": int(row["observed_motor_ticks"]),
    }


def _validate_source_scope(
    cases: Sequence[Any],
    source_rows: Sequence[dict[str, Any]],
    source_scope: dict[str, Any],
) -> None:
    if (
        len(cases) != len(source_rows)
        or len(cases) != int(source_scope.get("required_case_count", -1))
    ):
        raise ValueError("source audit case count differs from its canonical scope")
    for case, row in zip(cases, source_rows, strict=True):
        if (
            case.ordinal != int(row["case_ordinal"])
            or case.clip_id != row["clip_id"]
            or case.split != row["split"]
            or case.start_frame != int(row["start_frame"])
            or case.terminal_frame != int(row["terminal_frame"])
            or case.repeat_index != int(row["repeat_index"])
        ):
            raise ValueError("source phase results differ from their canonical schedule")


def _step_tensors_to_cpu(
    *,
    environment: Any,
    done: Any,
    truncated: Any,
    safety_reasons: Sequence[str],
    contact_safety_reasons: Sequence[str],
) -> dict[str, Any]:
    branch_attributes = {
        "hard_rom": "last_step_failure_hard_rom",
        "joint_safety": "last_step_failure_joint_safety",
        "joint_velocity": "last_step_failure_joint_velocity",
        "effort_envelope": "last_step_failure_effort_envelope",
        "hard_impact": "last_step_failure_hard_impact",
        "self_collision": "last_step_failure_self_collision",
        "forbidden_contact": "last_step_failure_forbidden_contact",
        "fall": "last_step_failure_fall",
        "world_bounds": "last_step_failure_world_bounds",
        "non_finite": "last_step_failure_non_finite",
    }
    pair_attributes = {
        "hard_impact": "last_step_hard_impact_pair_mask",
        "self_collision": "last_step_self_collision_pair_mask",
        "forbidden_contact": "last_step_forbidden_contact_pair_mask",
    }
    return {
        "assignment": _cpu(environment.last_step_diagnostic_assignment_ordinal),
        "clip_index": _cpu(environment.last_step_episode_clip_index),
        "start_frame": _cpu(environment.last_step_episode_start_frame),
        "reference_frame": _cpu(environment.last_step_reference_frame),
        "elapsed_ticks": _cpu(environment.last_step_episode_elapsed_motor_ticks),
        "done": _cpu(done),
        "success": _cpu(environment.last_step_success),
        "failure": _cpu(environment.last_step_failure),
        "truncated": _cpu(truncated),
        "safety": {
            reason: _cpu(getattr(environment, branch_attributes[reason]))
            for reason in safety_reasons
        },
        "tracking_lost": _cpu(environment.last_step_failure_tracking_lost),
        "root_position_error": _cpu(
            environment.last_step_root_position_error_micrometres
        ),
        "root_orientation_dot": _cpu(
            environment.last_step_root_orientation_absolute_dot_q1_30
        ),
        "observed_joint_position": _cpu(
            environment.last_step_action_joint_position_microradians
        ),
        "reference_joint_position": _cpu(
            environment.last_step_reference_joint_position_microradians
        ),
        "hard_rom_excess": _cpu(
            environment.last_step_hard_rom_excess_by_action_channel
        ),
        "velocity_excess": _cpu(
            environment.last_step_velocity_excess_by_action_channel
        ),
        "effort_envelope_channels": _cpu(
            environment.last_step_effort_envelope_violation_by_action_channel
        ),
        "observed_contacts": _cpu(environment.last_step_observed_contacts),
        "reference_contacts": _cpu(environment.last_step_reference_contacts),
        "contact_pair_masks": {
            reason: _cpu(getattr(environment, pair_attributes[reason]))
            for reason in contact_safety_reasons
        },
        "contact_pair_impulses": _cpu(
            environment.last_step_episode_contact_max_impulse_micronewton_seconds
        ),
        "forbidden_contact_mask": _cpu(environment.last_step_forbidden_contact_mask),
    }


def _cpu(value: Any) -> Any:
    return value.detach().cpu()


def _integer_row(value: Any, index: int) -> tuple[int, ...]:
    return tuple(int(item) for item in value[index].tolist())


def _boolean_row(value: Any, index: int) -> tuple[bool, ...]:
    return tuple(bool(item) for item in value[index].tolist())


def _console_summary(report: dict[str, Any], output: Path) -> dict[str, Any]:
    return {
        "output": str(output),
        "status": report["status"],
        "baseline_validation": report["baseline_validation"],
        "modes": {
            mode: {
                "failed_case_count": result["overall"][
                    "required_safety_failed_case_count"
                ],
                "required_safety_event_counts": result["overall"][
                    "required_safety_event_counts"
                ],
                "transition_counts": result["comparison_to_source"][
                    "transition_counts"
                ],
            }
            for mode, result in report["modes"].items()
        },
        "optimizer_steps": report["optimizer_steps"],
        "training_runs": report["training_runs"],
    }


def _external_output(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("causal-probe reports must stay outside the repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _write_report(path: Path, report: dict[str, Any]) -> None:
    payload = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_bytes(payload)
    temporary.replace(path)


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _repository_state() -> dict[str, Any]:
    repository = Path(__file__).resolve().parents[2]
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    dirty_paths = subprocess.run(
        ["git", "status", "--short"],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    return {"commit": commit, "dirty": bool(dirty_paths), "dirty_paths": dirty_paths}


if __name__ == "__main__":
    main()
