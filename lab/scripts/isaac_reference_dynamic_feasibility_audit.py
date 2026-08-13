#!/usr/bin/env python3
"""Exhaust every admitted reference clip/start phase without an optimizer."""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
import os
import platform
import sys
import traceback
from pathlib import Path

from isaaclab.app import AppLauncher


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--admission-gate-report", type=Path, required=True)
    parser.add_argument("--remediation-gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--substrate-profile", type=Path, required=True)
    parser.add_argument("--horizon", type=int, default=11)
    parser.add_argument("--repeats", type=int, default=1)
    parser.add_argument("--num-envs", type=int, default=256)
    parser.add_argument("--seed", type=int, default=120_812)
    parser.add_argument("--reset-safety-window-motor-ticks", type=int, default=1)
    parser.add_argument("--output", type=Path, required=True)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if (
        args.horizon <= 0
        or isinstance(args.repeats, bool)
        or args.repeats <= 0
        or args.num_envs <= 0
        or args.seed < 0
        or args.reset_safety_window_motor_ticks <= 0
        or not args.device.startswith("cuda:")
    ):
        raise ValueError("invalid dynamic-feasibility audit bounds")
    paths = (
        args.descriptor,
        args.profile,
        args.admission_gate_report,
        args.remediation_gate_report,
        args.usd,
        args.substrate_profile,
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
    substrate_profile = json.loads(args.substrate_profile.resolve().read_bytes())
    if (
        substrate_profile.get("schema_version") != 1
        or substrate_profile.get("profile_id")
        != "nextengine.isaac-lab-physx-stage0.v1"
        or substrate_profile.get("backend") != "physx"
    ):
        raise ValueError("invalid Isaac/PhysX substrate profile")
    runtime_package_versions = {
        name: _package_version(name) for name in ("torch", "isaaclab", "isaacsim")
    }
    if (
        f"{sys.version_info.major}.{sys.version_info.minor}"
        != substrate_profile.get("python_version")
        or runtime_package_versions["isaaclab"]
        != substrate_profile.get("isaac_lab_distribution_version")
        or runtime_package_versions["isaacsim"]
        != substrate_profile.get("isaac_sim_distribution_version")
    ):
        raise ValueError("installed Isaac substrate differs from the frozen profile")
    raw_profile = json.loads(args.profile.resolve().read_bytes())
    corpus_contract = raw_profile.get("corpus") or raw_profile.get("variant", {}).get(
        "corpus"
    )
    if not isinstance(corpus_contract, dict):
        raise ValueError("reference tracker profile has no corpus contract")
    partition = corpus_contract.get("eligible_partition")
    manifest = json.loads(manifest_path.read_bytes())
    descriptor = json.loads(args.descriptor.resolve().read_bytes())
    remediation_gate = json.loads(
        args.remediation_gate_report.resolve().read_bytes()
    )
    from next_lab.reference_dynamic_feasibility import (
        build_dynamic_audit_cases,
        validate_remediation_authorization,
    )

    validate_remediation_authorization(
        remediation_gate,
        admission_gate_report_sha256=_sha256(
            args.admission_gate_report.resolve()
        ),
        reference_tracker_profile_sha256=_sha256(args.profile.resolve()),
        corpus_manifest_sha256=manifest["manifest_sha256"],
        body_schema_hash=descriptor["body_schema_hash"],
        compiled_descriptor_hash=descriptor["compiled_descriptor_hash"],
        usd_sha256=_sha256(args.usd.resolve()),
    )
    entries = sorted(
        (
            entry
            for entry in manifest.get("clips", ())
            if entry.get("partition") == partition
        ),
        key=lambda entry: entry["clip_id"].encode("utf-8"),
    )
    if not entries:
        raise ValueError("dynamic-feasibility audit has no admitted clips")
    clip_inventory = tuple(
        (entry["clip_id"], entry["split"], entry["reference_frame_count"])
        for entry in entries
    )

    cases = build_dynamic_audit_cases(
        clips=clip_inventory,
        horizon_motor_ticks=args.horizon,
        repeats=args.repeats,
    )
    effective_num_envs = min(args.num_envs, len(cases))
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(args.usd.resolve())
    simulation_app = None
    environment = None
    failure_pending = False
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch

        import next_lab.isaac_reference_env as reference_environment_module
        from next_lab.isaac_reference_env import (
            ACTION_CHANNELS,
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
        )
        from next_lab.reference_dynamic_feasibility import (
            CONTACT_SAFETY_REASONS,
            REQUIRED_SAFETY_REASONS,
            DynamicFeasibilityAccumulator,
        )

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = effective_num_envs
        cfg.sim.device = args.device
        cfg.seed = args.seed
        cfg.eligible_clip_ids = tuple(clip_id for clip_id, _, _ in clip_inventory)
        cfg.phase_randomization = False
        cfg.fixed_horizon_motor_ticks = args.horizon
        cfg.diagnostic_exhaustive_phase_sweep_repeats = args.repeats
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.profile.resolve()),
            corpus_root=str(corpus_root),
            gate_report_path=str(args.admission_gate_report.resolve()),
        )
        environment_inventory = tuple(
            (clip.clip_id, clip.split, clip.frame_count)
            for clip in environment.reference_clips
        )
        if environment_inventory != clip_inventory:
            raise RuntimeError(
                "environment clip inventory differs from the manifest audit scope"
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
                "environment exhaustive schedule differs from the audit cases"
            )

        action_channel_ids = tuple(
            environment.reference_profile.document["action"]["ordered_actuator_ids"]
        )
        contact_channel_ids = tuple(
            environment.reference_profile.document["observation"]["contact_order"]
        )
        accumulator = DynamicFeasibilityAccumulator(
            cases=cases,
            action_channel_ids=action_channel_ids,
            contact_channel_ids=contact_channel_ids,
            contact_pair_ids=environment.contact_pair_ids,
            reset_safety_window_motor_ticks=(
                args.reset_safety_window_motor_ticks
            ),
        )
        environment.reset_diagnostic_episode_sequence()
        observations, _ = environment.reset()
        if not torch.isfinite(observations["policy"]).all():
            raise RuntimeError("dynamic-feasibility reset observation is non-finite")
        zero = torch.zeros(
            (effective_num_envs, ACTION_CHANNELS),
            dtype=torch.float32,
            device=environment.device,
        )
        executed_vector_motor_steps = 0
        waves = (len(cases) + effective_num_envs - 1) // effective_num_envs
        maximum_vector_motor_steps = (waves + 1) * (args.horizon + 1)
        while accumulator.completed_case_count < accumulator.case_count:
            observations, _, terminated, truncated, _ = environment.step(zero)
            executed_vector_motor_steps += 1
            if not torch.isfinite(observations["policy"]).all():
                raise RuntimeError("dynamic-feasibility step observation is non-finite")
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
                    reference_frame=int(
                        tensors["reference_frame"][env_index].item()
                    ),
                    elapsed_motor_ticks=int(
                        tensors["elapsed_ticks"][env_index].item()
                    ),
                    done=bool(tensors["done"][env_index].item()),
                    success=bool(tensors["success"][env_index].item()),
                    failure=bool(tensors["failure"][env_index].item()),
                    truncated=bool(tensors["truncated"][env_index].item()),
                    safety={
                        reason: bool(tensors["safety"][reason][env_index].item())
                        for reason in REQUIRED_SAFETY_REASONS
                    },
                    tracking_lost=bool(
                        tensors["tracking_lost"][env_index].item()
                    ),
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
                    observed_contacts=_integer_row(
                        tensors["observed_contacts"], env_index
                    ),
                    reference_contacts=_integer_row(
                        tensors["reference_contacts"], env_index
                    ),
                    contact_pair_masks={
                        reason: _boolean_row(
                            tensors["contact_pair_masks"][reason], env_index
                        )
                        for reason in CONTACT_SAFETY_REASONS
                    },
                    contact_pair_maximum_impulses_micronewton_seconds=(
                        _integer_row(tensors["contact_pair_impulses"], env_index)
                    ),
                    forbidden_contact_mask=int(
                        tensors["forbidden_contact_mask"][env_index].item()
                    ),
                )
            if executed_vector_motor_steps > maximum_vector_motor_steps:
                raise RuntimeError("dynamic-feasibility audit exceeded its progress bound")
            if (
                executed_vector_motor_steps % 50 == 0
                or accumulator.completed_case_count == accumulator.case_count
            ):
                print(
                    (
                        "dynamic-feasibility progress: "
                        f"{accumulator.completed_case_count}/{accumulator.case_count}"
                    ),
                    file=sys.stderr,
                    flush=True,
                )

        sections = accumulator.report_sections()
        overall = sections["overall"]
        report = {
            "schema_version": 1,
            "check": "TRAIN-4-ISAAC-EXHAUSTIVE-DYNAMIC-REFERENCE-FEASIBILITY",
            "status": overall["status"],
            "claim": "OptimizerFreeScriptedReferenceDynamicFeasibilityOnly",
            "requirement_id": "REQ-HUM-DATA-007",
            "required_safety_reasons": list(REQUIRED_SAFETY_REASONS),
            "report_only_metrics": [
                "reference_completion",
                "tracking_loss",
                "root_tracking_error",
                "joint_tracking_error",
                "contact_precision_recall",
            ],
            "audit_semantics": {
                "zero_residual_action": True,
                "optimizer_free": True,
                "isaac_gpu_physx_is_non_authoritative_training_mirror": True,
                "every_admitted_partition_clip": True,
                "every_valid_start_phase": True,
                "tracking_loss_is_observed_but_does_not_end_the_diagnostic_sweep": True,
                "tracking_continuation_changes_no_physics_control_or_safety_rule": True,
                "pass_rule": (
                    "complete coverage and exactly zero required safety events"
                ),
            },
            "identities": {
                "reference_tracker_profile_id": (
                    environment.reference_profile.document["profile_id"]
                ),
                "reference_tracker_profile_sha256": (
                    environment.reference_profile.document_sha256
                ),
                "motion_corpus_profile_id": corpus_contract["profile_id"],
                "motion_corpus_profile_sha256": corpus_contract["profile_sha256"],
                "descriptor_file_sha256": _sha256(args.descriptor.resolve()),
                "body_schema_hash": descriptor["body_schema_hash"],
                "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
                "usd_sha256": _sha256(args.usd.resolve()),
                "admission_gate_report_sha256": _sha256(
                    args.admission_gate_report.resolve()
                ),
                "remediation_gate_report_sha256": _sha256(
                    args.remediation_gate_report.resolve()
                ),
                "corpus_manifest_file_sha256": _sha256(manifest_path),
                "corpus_manifest_sha256": manifest["manifest_sha256"],
                "substrate_profile_sha256": _sha256(
                    args.substrate_profile.resolve()
                ),
                "audit_tool_sha256": _sha256(Path(__file__).resolve()),
                "audit_aggregator_sha256": _sha256(
                    Path(sys.modules[DynamicFeasibilityAccumulator.__module__].__file__).resolve()
                ),
                "isaac_reference_env_sha256": _sha256(
                    Path(reference_environment_module.__file__).resolve()
                ),
            },
            "substrate": {
                "profile": substrate_profile,
                "python_version": platform.python_version(),
                "platform": platform.platform(),
                "runtime_package_versions": {
                    name: runtime_package_versions[name]
                    for name in ("torch", "isaaclab", "isaacsim")
                },
                "device": str(environment.device),
                "cuda_device_name": torch.cuda.get_device_name(environment.device),
                "cuda_compute_capability": list(
                    torch.cuda.get_device_capability(environment.device)
                ),
                "torch_cuda_version": torch.version.cuda,
                "seed": args.seed,
                "physics_substeps_per_motor_tick": cfg.decimation,
                "motor_rate_hz": 60,
                "isaac_physics_velocity_limit_basis_points": (
                    environment.physics_velocity_limit_basis_points
                ),
            },
            "scope": {
                "eligible_partition": partition,
                "clip_order": "canonical UTF-8 clip_id byte order",
                "clip_count": len(clip_inventory),
                "clip_inventory": [
                    {
                        "clip_id": clip_id,
                        "split": split,
                        "reference_frame_count": frame_count,
                        "valid_start_frame_count": frame_count - args.horizon,
                    }
                    for clip_id, split, frame_count in clip_inventory
                ],
                "horizon_motor_ticks": args.horizon,
                "repeat_count_per_start_phase": args.repeats,
                "required_case_count": len(cases),
                "requested_num_envs": args.num_envs,
                "effective_num_envs": effective_num_envs,
                "executed_vector_motor_steps": executed_vector_motor_steps,
            },
            "channel_layout": {
                "action_channel_ids": list(action_channel_ids),
                "contact_channel_ids": list(contact_channel_ids),
                "contact_pair_ids": list(environment.contact_pair_ids),
            },
            "results": sections,
            "optimizer_steps": 0,
            "training_runs": 0,
            "learned_policy_claim": False,
        }
        _write_report(output, report)
        print(
            json.dumps(
                {
                    "output": str(output),
                    "status": report["status"],
                    "case_count": overall["case_count"],
                    "required_safety_failed_case_count": overall[
                        "required_safety_failed_case_count"
                    ],
                    "required_safety_event_counts": overall[
                        "required_safety_event_counts"
                    ],
                },
                indent=2,
                sort_keys=True,
            ),
            flush=True,
        )
        if report["status"] != "PASS":
            raise RuntimeError("exhaustive dynamic-reference feasibility audit failed")
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
        # SimulationApp.close() can make a pending Python exception exit with code 0.
        # On failure the process owns no reusable state, so OS teardown preserves the
        # fail-closed shell status. Successful audits still use graceful Kit cleanup.
        if simulation_app is not None:
            if failure_pending or cleanup_failed:
                sys.stdout.flush()
                sys.stderr.flush()
                os._exit(1)
            simulation_app.close()
        if cleanup_failed:
            raise RuntimeError("dynamic-feasibility environment cleanup failed")


def _step_tensors_to_cpu(
    *,
    environment: object,
    done: object,
    truncated: object,
    safety_reasons: tuple[str, ...],
    contact_safety_reasons: tuple[str, ...],
) -> dict[str, object]:
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
        "forbidden_contact_mask": _cpu(
            environment.last_step_forbidden_contact_mask
        ),
    }


def _cpu(value: object) -> object:
    return value.detach().cpu()


def _integer_row(value: object, index: int) -> tuple[int, ...]:
    return tuple(int(item) for item in value[index].tolist())


def _boolean_row(value: object, index: int) -> tuple[bool, ...]:
    return tuple(bool(item) for item in value[index].tolist())


def _external_output(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("dynamic-feasibility reports must stay outside the repository")
    return output


def _write_report(path: Path, report: dict[str, object]) -> None:
    payload = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and path.read_bytes() != payload:
        raise ValueError(f"refusing to overwrite a different audit: {path}")
    if not path.exists():
        temporary = path.with_suffix(path.suffix + ".tmp")
        temporary.write_bytes(payload)
        temporary.replace(path)


def _package_version(distribution: str) -> str | None:
    try:
        return importlib.metadata.version(distribution)
    except importlib.metadata.PackageNotFoundError:
        return None


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    main()
