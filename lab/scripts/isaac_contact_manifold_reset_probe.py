#!/usr/bin/env python3
"""Run the bounded TRAIN-4 contact-manifold and reset-semantics probe."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import traceback
from pathlib import Path
from typing import Any, Mapping, Sequence

import numpy as np
from isaaclab.app import AppLauncher

from next_lab.contact_manifold_physx import (
    ContactPrototypeCase,
    authored_root_state,
    build_fresh_scene_usda,
    compare_reset_paths,
    evaluate_bounded_acceptance,
    load_case_arrays,
    load_contact_prototype_cases,
    minimum_normalized_quaternion_dot_q1_30,
    overlay_bounded_reference_window,
)
from next_lab.motion_math import collider_minimum_y, target_forward_kinematics


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--probe-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--reference-profile", type=Path, required=True)
    parser.add_argument("--prototype-manifest", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=120_812)
    parser.add_argument("--worker-case-ordinal", type=int, default=-1)
    parser.add_argument("--partial-reset-worker", action="store_true")
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    inputs = _validated_inputs(args)
    if args.partial_reset_worker and args.worker_case_ordinal >= 0:
        raise ValueError("reset-probe worker modes are mutually exclusive")
    if args.partial_reset_worker and not inputs["profile"]["execution"][
        "indexed_partial_reset"
    ].get("enabled", True):
        raise ValueError("indexed partial reset is disabled by the frozen profile")
    if args.partial_reset_worker:
        output = args.output.resolve()
        report_path = _partial_result_path(output)
        if not output.is_dir() or report_path.exists():
            raise ValueError("partial-reset worker paths are invalid")
        _run_partial_reset(args=args, report_path=report_path, **inputs)
    elif args.worker_case_ordinal >= 0:
        _run_fresh_worker(args=args, **inputs)
    else:
        _run_driver(args=args, **inputs)


def _validated_inputs(args: argparse.Namespace) -> dict[str, Any]:
    paths = {
        "source_audit_path": args.source_audit.resolve(),
        "probe_profile_path": args.probe_profile.resolve(),
        "descriptor_path": args.descriptor.resolve(),
        "reference_profile_path": args.reference_profile.resolve(),
        "prototype_manifest_path": args.prototype_manifest.resolve(),
        "gate_report_path": args.gate_report.resolve(),
        "usd_path": args.usd.resolve(),
    }
    corpus_root = args.corpus_root.resolve()
    corpus_manifest_path = corpus_root / "corpus-manifest.json"
    for path in (*paths.values(), corpus_manifest_path):
        if not path.is_file():
            raise FileNotFoundError(path)
    if not corpus_root.is_dir() or args.seed < 0 or not args.device.startswith("cuda:"):
        raise ValueError("invalid reset-probe execution configuration")
    profile = json.loads(paths["probe_profile_path"].read_bytes())
    descriptor = json.loads(paths["descriptor_path"].read_bytes())
    corpus_manifest = json.loads(corpus_manifest_path.read_bytes())
    manifest, source_audit, cases = load_contact_prototype_cases(
        manifest_path=paths["prototype_manifest_path"],
        source_audit_path=paths["source_audit_path"],
    )
    source = profile.get("source", {})
    execution = profile.get("execution", {})
    probe_id = profile.get("probe_id")
    version_shape_is_valid = _probe_version_shape_is_valid(
        probe_id=probe_id,
        profile=profile,
        prototype_manifest=manifest,
        cases=cases,
    )
    if (
        not version_shape_is_valid
        or profile.get("status") != "FrozenResearchOnly"
        or source.get("audit_sha256") != _sha256(paths["source_audit_path"])
        or source.get("contact_prototype_manifest_sha256")
        != manifest.get("manifest_sha256")
        or source.get("contact_prototype_manifest_file_sha256")
        != _sha256(paths["prototype_manifest_path"])
        or source.get("contact_prototype_profile_sha256")
        != manifest.get("identities", {}).get("prototype_profile_sha256")
        or source.get("corpus_manifest_sha256")
        != corpus_manifest.get("manifest_sha256")
        or source.get("corpus_manifest_sha256")
        != manifest.get("identities", {}).get("corpus_manifest_sha256")
        or source.get("descriptor_sha256") != _sha256(paths["descriptor_path"])
        or source.get("gate_report_sha256") != _sha256(paths["gate_report_path"])
        or source.get("reference_tracker_profile_sha256")
        != _sha256(paths["reference_profile_path"])
        or source.get("corpus_manifest_file_sha256")
        != _sha256(corpus_manifest_path)
        or source.get("usd_sha256") != _sha256(paths["usd_path"])
        or int(execution.get("horizon_motor_ticks", -1))
        != cases[0].horizon_motor_ticks
        or int(execution.get("vector_environment_count", -1)) != len(cases)
        or execution.get("reference_overlay")
        != {
            "projected_values": (
                "exact bounded prototype artifact on relative frames 0 through 11"
            ),
            "outside_projected_window": (
                "unchanged reference values from the hash-closed V18 source corpus"
            ),
            "observation_horizon_offsets_motor_ticks": [0, 4, 8, 16],
        }
        or execution.get("optimizer_steps") != 0
        or execution.get("training_runs") != 0
    ):
        raise ValueError("frozen reset-probe profile does not close its inputs")
    return {
        **paths,
        "corpus_root": corpus_root,
        "corpus_manifest_path": corpus_manifest_path,
        "profile": profile,
        "descriptor": descriptor,
        "prototype_manifest": manifest,
        "source_audit": source_audit,
        "cases": cases,
    }


def _probe_version_shape_is_valid(
    *,
    probe_id: Any,
    profile: Mapping[str, Any],
    prototype_manifest: Mapping[str, Any],
    cases: Sequence[ContactPrototypeCase],
) -> bool:
    execution = profile.get("execution", {})
    partial = execution.get("indexed_partial_reset", {})
    acceptance = profile.get("bounded_acceptance", {})
    if probe_id == "nextengine.humanoid-contact-manifold-physx-probe.v1":
        return (
            prototype_manifest.get("prototype_id")
            == "nextengine.humanoid-contact-manifold-prototype.v1"
            and len(cases) == 17
            and partial.get("process_isolation")
            == (
                "one dedicated worker process; the non-Isaac driver requires "
                "its report before aggregation"
            )
            and partial.get("enabled", True) is True
            and acceptance.get(
                "acceptance_authority", "fresh-and-indexed-partial"
            )
            == "fresh-and-indexed-partial"
        )
    if probe_id == "nextengine.humanoid-contact-manifold-physx-probe.v6":
        return (
            prototype_manifest.get("prototype_id")
            == "nextengine.humanoid-contact-manifold-prototype.v5"
            and prototype_manifest.get("scope", {}).get("case_scope") == "all"
            and len(cases) == 17
            and prototype_manifest.get("scope", {}).get("failure_case_count")
            == 7
            and prototype_manifest.get("scope", {}).get("control_case_count")
            == 10
            and partial.get("enabled") is False
            and partial.get("evidence_role") == "report-only"
            and acceptance.get("acceptance_authority") == "fresh-scene"
            and acceptance.get("pass_gate_decision")
            == "PERMIT_CLIP_GLOBAL_PROTOTYPE_ONLY"
            and acceptance.get("fail_gate_decision") == "STOP_AND_RESEARCH"
        )
    if probe_id == "nextengine.humanoid-contact-manifold-physx-probe.v7":
        return (
            prototype_manifest.get("prototype_id")
            == "nextengine.humanoid-contact-manifold-prototype.v6"
            and prototype_manifest.get("scope", {}).get("case_scope")
            == "discriminator"
            and tuple(case.source_case_ordinal for case in cases)
            == (3749, 3750, 3753, 7978, 8144)
            and partial.get("enabled") is False
            and partial.get("evidence_role") == "report-only"
            and acceptance.get("acceptance_authority") == "fresh-scene"
            and acceptance.get("pass_gate_decision")
            == "PERMIT_ALL_17_BOUNDED_PROTOTYPE_ONLY"
            and acceptance.get("fail_gate_decision") == "STOP_AND_RESEARCH"
        )
    if probe_id == "nextengine.humanoid-contact-manifold-physx-probe.v8":
        return (
            prototype_manifest.get("prototype_id")
            == "nextengine.humanoid-contact-manifold-prototype.v6"
            and prototype_manifest.get("scope", {}).get("case_scope") == "all"
            and len(cases) == 17
            and prototype_manifest.get("scope", {}).get("failure_case_count")
            == 7
            and prototype_manifest.get("scope", {}).get("control_case_count")
            == 10
            and partial.get("enabled") is False
            and partial.get("evidence_role") == "report-only"
            and acceptance.get("acceptance_authority") == "fresh-scene"
            and acceptance.get("pass_gate_decision")
            == "PERMIT_CLIP_GLOBAL_PROTOTYPE_ONLY"
            and acceptance.get("fail_gate_decision") == "STOP_AND_RESEARCH"
        )
    bounded_versions = {
        "nextengine.humanoid-contact-manifold-physx-probe.v2": (
            "nextengine.humanoid-contact-manifold-prototype.v2"
        ),
        "nextengine.humanoid-contact-manifold-physx-probe.v3": (
            "nextengine.humanoid-contact-manifold-prototype.v3"
        ),
        "nextengine.humanoid-contact-manifold-physx-probe.v4": (
            "nextengine.humanoid-contact-manifold-prototype.v4"
        ),
        "nextengine.humanoid-contact-manifold-physx-probe.v5": (
            "nextengine.humanoid-contact-manifold-prototype.v5"
        ),
    }
    if isinstance(probe_id, str) and probe_id in bounded_versions:
        return (
            prototype_manifest.get("prototype_id")
            == bounded_versions[probe_id]
            and prototype_manifest.get("scope", {}).get("case_scope")
            == "discriminator"
            and tuple(case.source_case_ordinal for case in cases)
            == (3749, 3750, 3753, 8144)
            and partial.get("enabled") is False
            and partial.get("evidence_role") == "report-only"
            and acceptance.get("acceptance_authority") == "fresh-scene"
            and acceptance.get("pass_gate_decision")
            == "PERMIT_ALL_17_BOUNDED_PROTOTYPE_ONLY"
            and acceptance.get("fail_gate_decision") == "STOP_AND_RESEARCH"
        )
    return False


def _run_driver(
    *,
    args: argparse.Namespace,
    profile: dict[str, Any],
    descriptor: dict[str, Any],
    prototype_manifest: dict[str, Any],
    source_audit: dict[str, Any],
    cases: tuple[ContactPrototypeCase, ...],
    **paths: Any,
) -> None:
    repository = _repository_state()
    if repository["dirty"]:
        raise RuntimeError("reset-probe evidence requires a clean repository commit")
    output = _external_output_directory(args.output)
    fresh_root = output / "fresh-scene"
    usd_root = fresh_root / "usd"
    result_root = fresh_root / "results"
    log_root = fresh_root / "logs"
    for directory in (usd_root, result_root, log_root):
        directory.mkdir(parents=True)
    overlay_hashes: dict[int, str] = {}
    for case in cases:
        arrays = load_case_arrays(case)
        payload = build_fresh_scene_usda(
            base_usd_path=paths["usd_path"],
            descriptor=descriptor,
            arrays=arrays,
            source_case_ordinal=case.source_case_ordinal,
            artifact_sha256=case.artifact_sha256,
        ).encode("utf-8")
        overlay_path = _fresh_usd_path(output, case.ordinal)
        overlay_path.write_bytes(payload)
        overlay_hashes[case.ordinal] = hashlib.sha256(payload).hexdigest()

    for case in cases:
        print(
            f"fresh-scene {case.ordinal + 1}/{len(cases)}: "
            f"{case.clip_id}@{case.frame_first}",
            file=sys.stderr,
            flush=True,
        )
        command = _worker_command(args, case.ordinal)
        log_path = log_root / f"case-{case.ordinal:02d}.log"
        with log_path.open("wb") as log:
            completed = subprocess.run(
                command,
                cwd=Path(__file__).resolve().parents[2],
                stdout=log,
                stderr=subprocess.STDOUT,
                check=False,
            )
        result_path = _fresh_result_path(output, case.ordinal)
        if completed.returncode != 0 or not result_path.is_file():
            raise RuntimeError(
                f"fresh-scene worker {case.ordinal} failed; inspect {log_path}"
            )
        worker_report = json.loads(result_path.read_bytes())
        if worker_report.get("status") != "PASS":
            raise RuntimeError(
                f"fresh-scene worker {case.ordinal} is invalid; inspect {log_path}"
            )

    partial_enabled = bool(
        profile["execution"]["indexed_partial_reset"].get("enabled", True)
    )
    partial_path = _partial_result_path(output)
    partial_log_path = output / "indexed-partial-reset.log"
    partial_report: dict[str, Any] | None = None
    if partial_enabled:
        with partial_log_path.open("wb") as log:
            partial_completed = subprocess.run(
                _partial_worker_command(args),
                cwd=Path(__file__).resolve().parents[2],
                stdout=log,
                stderr=subprocess.STDOUT,
                check=False,
            )
        if partial_completed.returncode != 0 or not partial_path.is_file():
            raise RuntimeError(
                f"indexed partial-reset worker failed; inspect {partial_log_path}"
            )
        partial_report = json.loads(partial_path.read_bytes())
        if (
            partial_report.get("check")
            != "TRAIN-4-ISAAC-CONTACT-MANIFOLD-INDEXED-PARTIAL-RESET"
            or partial_report.get("status") != "PASS"
            or partial_report.get("repository") != repository
        ):
            raise RuntimeError(
                "indexed partial-reset worker is invalid; inspect "
                f"{partial_log_path}"
            )

    fresh_reports = []
    fresh_rows = []
    for case in cases:
        result_path = _fresh_result_path(output, case.ordinal)
        report = json.loads(result_path.read_bytes())
        if (
            report.get("status") != "PASS"
            or report.get("case_ordinal") != case.ordinal
            or report.get("overlay_usd_sha256") != overlay_hashes[case.ordinal]
            or report.get("repository") != repository
        ):
            raise RuntimeError(f"fresh-scene result {case.ordinal} is invalid")
        fresh_reports.append(
            {
                "case_ordinal": case.ordinal,
                "report_relative_path": str(result_path.relative_to(output)),
                "report_sha256": _sha256(result_path),
                "log_relative_path": str(
                    (log_root / f"case-{case.ordinal:02d}.log").relative_to(output)
                ),
                "log_sha256": _sha256(log_root / f"case-{case.ordinal:02d}.log"),
                "overlay_usd_relative_path": str(
                    _fresh_usd_path(output, case.ordinal).relative_to(output)
                ),
                "overlay_usd_sha256": overlay_hashes[case.ordinal],
                "initial_state_verification": report["initial_state_verification"],
                "trajectory_diagnostics": report["trajectory_diagnostics"],
                "deferred_terminal_reset_count": report[
                    "deferred_terminal_reset_count"
                ],
            }
        )
        fresh_rows.append(report["phase_result"])
    fresh_acceptance = evaluate_bounded_acceptance(cases=cases, rows=fresh_rows)
    if partial_report is not None:
        partial_rows = partial_report["results"]["phase_results"]
        maximum_impulse_delta = int(
            profile["reset_comparison"][
                "maximum_first_tick_contact_pair_impulse_delta_micronewton_seconds"
            ]
        )
        reset_comparison = compare_reset_paths(
            fresh_rows=fresh_rows,
            partial_rows=partial_rows,
            maximum_first_tick_impulse_delta=maximum_impulse_delta,
        )
        partial_acceptance = evaluate_bounded_acceptance(
            cases=cases, rows=partial_rows
        )
        partial_section = {
            "status": partial_report["status"],
            "evidence_role": "acceptance-comparison-input",
            "report_sha256": _sha256(partial_path),
            "log_relative_path": str(partial_log_path.relative_to(output)),
            "log_sha256": _sha256(partial_log_path),
            "results": partial_report["results"],
        }
        disposition_key = (
            "equivalence_disposition"
            if reset_comparison["status"] == "PASS"
            else "divergence_disposition"
        )
        architecture_decision = profile["reset_comparison"][disposition_key]
    else:
        reset_comparison = {
            "status": "NOT_RUN",
            "required_for_acceptance": False,
            "reason": (
                "ADR-070 fresh-scene authority is already retained; indexed "
                "partial reset is report-only"
            ),
        }
        partial_acceptance = {
            "status": "NOT_RUN",
            "required_for_acceptance": False,
        }
        partial_section = {
            "status": "NOT_RUN",
            "evidence_role": profile["execution"]["indexed_partial_reset"][
                "evidence_role"
            ],
            "results": None,
        }
        architecture_decision = profile["reset_comparison"][
            "report_only_disposition"
        ]
    acceptance_profile = profile["bounded_acceptance"]
    acceptance_authority = acceptance_profile.get(
        "acceptance_authority", "fresh-and-indexed-partial"
    )
    if acceptance_authority == "fresh-scene":
        accepted = fresh_acceptance["status"] == "PASS"
    elif acceptance_authority == "fresh-and-indexed-partial":
        accepted = (
            reset_comparison["status"] == "PASS"
            and fresh_acceptance["status"] == "PASS"
            and partial_acceptance["status"] == "PASS"
        )
    else:
        raise ValueError("reset-probe acceptance authority is invalid")
    gate_decision = acceptance_profile.get(
        "pass_gate_decision" if accepted else "fail_gate_decision",
        "PERMIT_FULL_V19_DATA_BUILD_ONLY" if accepted else "STOP_AND_RESEARCH",
    )
    report = {
        "schema_version": 1,
        "check": (
            "TRAIN-4-ISAAC-CONTACT-MANIFOLD-FRESH-SCENE-DISCRIMINATOR"
            if acceptance_authority == "fresh-scene"
            else "TRAIN-4-ISAAC-CONTACT-MANIFOLD-RESET-PROBE"
        ),
        "status": "PASS" if accepted else "FAIL",
        "claim": "OptimizerFreeBoundedResearchOnly",
        "gate_decision": gate_decision,
        "architecture_disposition": {
            "adr": "ADR-070",
            "decision": architecture_decision,
            "accepted_semantics_changed": False,
            "indexed_partial_reset_is_acceptance_evidence": False,
        },
        "scope": {
            "case_count": len(cases),
            "fresh_scene_process_count": len(cases),
            "vector_environment_count": len(cases),
            "horizon_motor_ticks": cases[0].horizon_motor_ticks,
            "fresh_scene_post_create_state_writes": 0,
            "indexed_partial_reset_warmup_episodes_per_case": (
                1 if partial_enabled else 0
            ),
            "indexed_partial_reset_process_count": 1 if partial_enabled else 0,
        },
        "method": profile["execution"],
        "fresh_scene": {
            "status": "PASS",
            "workers": fresh_reports,
            "results": _summarize_rows(fresh_rows),
        },
        "indexed_partial_reset": partial_section,
        "reset_comparison": reset_comparison,
        "bounded_acceptance": {
            "status": "PASS" if accepted else "FAIL",
            "acceptance_authority": acceptance_authority,
            "full_v19_corpus_authorized": (
                accepted
                and gate_decision == "PERMIT_FULL_V19_DATA_BUILD_ONLY"
            ),
            "fresh_scene": fresh_acceptance,
            "indexed_partial_reset": partial_acceptance,
        },
        "identities": {
            "source_audit_sha256": _sha256(paths["source_audit_path"]),
            "probe_profile_sha256": _sha256(paths["probe_profile_path"]),
            "prototype_manifest_file_sha256": _sha256(
                paths["prototype_manifest_path"]
            ),
            "prototype_manifest_sha256": prototype_manifest["manifest_sha256"],
            "descriptor_sha256": _sha256(paths["descriptor_path"]),
            "reference_tracker_profile_sha256": _sha256(
                paths["reference_profile_path"]
            ),
            "corpus_manifest_file_sha256": _sha256(
                paths["corpus_manifest_path"]
            ),
            "gate_report_sha256": _sha256(paths["gate_report_path"]),
            "usd_sha256": _sha256(paths["usd_path"]),
            "tool_sha256": _sha256(Path(__file__).resolve()),
        },
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": repository,
    }
    report["manifest_sha256"] = hashlib.sha256(_canonical_json(report)).hexdigest()
    report_path = output / "reset-probe-report.json"
    _write_json(report_path, report)
    print(
        json.dumps(
            {
                "output": str(output),
                "status": report["status"],
                "gate_decision": report["gate_decision"],
                "fresh_failed_cases": report["fresh_scene"]["results"][
                    "required_safety_failed_case_count"
                ],
                "partial_failed_cases": (
                    report["indexed_partial_reset"]["results"]["overall"][
                        "required_safety_failed_case_count"
                    ]
                    if partial_enabled
                    else None
                ),
                "maximum_first_tick_impulse_delta_micronewton_seconds": (
                    reset_comparison.get(
                        "maximum_first_tick_impulse_delta_micronewton_seconds"
                    )
                ),
                "manifest_sha256": report["manifest_sha256"],
                "optimizer_steps": 0,
                "training_runs": 0,
            },
            indent=2,
            sort_keys=True,
        )
    )
    if not accepted:
        raise SystemExit(1)


def _run_fresh_worker(
    *,
    args: argparse.Namespace,
    profile: dict[str, Any],
    descriptor: dict[str, Any],
    cases: tuple[ContactPrototypeCase, ...],
    **paths: Any,
) -> None:
    ordinal = args.worker_case_ordinal
    if not 0 <= ordinal < len(cases):
        raise ValueError("fresh-scene worker case ordinal is outside the inventory")
    output = args.output.resolve()
    if not output.is_dir():
        raise FileNotFoundError(output)
    case = cases[ordinal]
    overlay_path = _fresh_usd_path(output, ordinal)
    result_path = _fresh_result_path(output, ordinal)
    if not overlay_path.is_file() or result_path.exists():
        raise ValueError("fresh-scene worker paths are invalid")
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(overlay_path)
    simulation_app = None
    environment = None
    pending_error: BaseException | None = None
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app
        import torch

        from next_lab.isaac_reference_env import (
            ACTION_CHANNELS,
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
            REFERENCE_OFFSETS,
        )
        from next_lab import isaac_reference_env as reference_env_module
        from next_lab.reference_dynamic_feasibility import (
            DynamicFeasibilityAccumulator,
        )
        from isaaclab.sim import SimulationContext
        from isaacsim.core.simulation_manager import SimulationManager

        vector_count = int(profile["execution"]["vector_environment_count"])
        reference_overlay = profile["execution"]["reference_overlay"]
        if tuple(reference_overlay["observation_horizon_offsets_motor_ticks"]) != tuple(
            REFERENCE_OFFSETS
        ):
            raise RuntimeError("reference overlay offsets differ from the tracker")
        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = vector_count
        cfg.sim.device = args.device
        cfg.seed = args.seed
        cfg.eligible_clip_ids = (case.clip_id,)
        cfg.phase_randomization = False
        cfg.fixed_horizon_motor_ticks = case.horizon_motor_ticks
        cfg.diagnostic_exhaustive_phase_sweep_repeats = 1
        original_reset = SimulationContext.reset
        original_spawn_ground = reference_env_module.sim_utils.spawn_ground_plane
        original_setup_contact_view = (
            NextEngineReferenceDirectEnv._setup_contact_pair_view
        )
        initialization_metrics: dict[str, Any] = {}
        ground_state: dict[str, Any] = {}

        def bounded_time_initialization_reset(
            simulation_context: Any, soft: bool = False
        ) -> None:
            episode_dt = float(simulation_context.get_physics_dt())
            initialization_dt = float(
                profile["execution"]["fresh_scene"][
                    "initialization_physics_dt_seconds"
                ]
            )
            steps_before = int(SimulationManager.get_num_physics_steps())
            simulation_context.set_simulation_dt(physics_dt=initialization_dt)
            try:
                original_reset(simulation_context, soft=soft)
            finally:
                simulation_context.set_simulation_dt(physics_dt=episode_dt)
            initialization_metrics.update(
                {
                    "physics_steps": int(
                        SimulationManager.get_num_physics_steps()
                    )
                    - steps_before,
                    "initialization_physics_dt_seconds": initialization_dt,
                    "restored_episode_physics_dt_seconds": float(
                        simulation_context.get_physics_dt()
                    ),
                }
            )

        def spawn_disabled_ground(*spawn_args: Any, **spawn_kwargs: Any) -> Any:
            prim = original_spawn_ground(*spawn_args, **spawn_kwargs)
            ground_path = str(spawn_args[0]) if spawn_args else str(
                spawn_kwargs["prim_path"]
            )
            collision = prim.GetStage().GetPrimAtPath(
                f"{ground_path}/GroundPlane/CollisionPlane"
            )
            attribute = collision.GetAttribute("physics:collisionEnabled")
            if not collision.IsValid() or not attribute.IsValid():
                raise RuntimeError("fresh-scene ground collision attribute is absent")
            attribute.Set(False)
            ground_state["attribute"] = attribute
            ground_state["disabled_before_initialization"] = not bool(
                attribute.Get()
            )
            return prim

        def setup_contact_view_after_ground_enable(environment_self: Any) -> None:
            attribute = ground_state.get("attribute")
            if attribute is None or bool(attribute.Get()):
                raise RuntimeError("fresh-scene ground was not disabled for warmup")
            attribute.Set(True)
            ground_state["enabled_before_contact_view"] = bool(attribute.Get())
            original_setup_contact_view(environment_self)

        SimulationContext.reset = bounded_time_initialization_reset
        reference_env_module.sim_utils.spawn_ground_plane = spawn_disabled_ground
        NextEngineReferenceDirectEnv._setup_contact_pair_view = (
            setup_contact_view_after_ground_enable
        )
        try:
            environment = NextEngineReferenceDirectEnv(
                cfg,
                descriptor_path=str(paths["descriptor_path"]),
                profile_path=str(paths["reference_profile_path"]),
                corpus_root=str(paths["corpus_root"]),
                gate_report_path=str(paths["gate_report_path"]),
            )
        finally:
            SimulationContext.reset = original_reset
            reference_env_module.sim_utils.spawn_ground_plane = original_spawn_ground
            NextEngineReferenceDirectEnv._setup_contact_pair_view = (
                original_setup_contact_view
            )
        fresh_method = profile["execution"]["fresh_scene"]
        if (
            initialization_metrics.get("physics_steps")
            != int(fresh_method["mandatory_initialization_physics_step_count"])
            or initialization_metrics.get("initialization_physics_dt_seconds")
            != float(fresh_method["initialization_physics_dt_seconds"])
            or abs(
                initialization_metrics.get(
                    "restored_episode_physics_dt_seconds", -1.0
                )
                - float(fresh_method["episode_physics_dt_seconds"])
            )
            > 1.0e-15
            or ground_state.get("disabled_before_initialization") is not True
            or ground_state.get("enabled_before_contact_view") is not True
        ):
            raise RuntimeError(
                "fresh-scene worker initialization cadence differs from profile"
            )
        environment.diagnostic_episode_schedule = tuple(
            (0, case.frame_first, case.frame_last, 0)
            for _ in range(vector_count)
        )
        arrays = load_case_arrays(case)
        _install_reference_override(
            environment=environment,
            arrays_by_env=tuple(arrays for _ in range(vector_count)),
            torch=torch,
        )
        before = _verify_authored_state(
            environment=environment,
            arrays=arrays,
            profile=profile,
            torch=torch,
        )
        original_root_write = environment.robot.write_root_state_to_sim
        original_joint_write = environment.robot.write_joint_state_to_sim
        write_attempts = {"root": 0, "joint": 0}

        def suppress_root_write(*unused_args: Any, **unused_kwargs: Any) -> None:
            write_attempts["root"] += 1

        def suppress_joint_write(*unused_args: Any, **unused_kwargs: Any) -> None:
            write_attempts["joint"] += 1

        environment.robot.write_root_state_to_sim = suppress_root_write
        environment.robot.write_joint_state_to_sim = suppress_joint_write
        try:
            environment.reset_diagnostic_episode_sequence()
            observations, _ = environment.reset()
        finally:
            environment.robot.write_root_state_to_sim = original_root_write
            environment.robot.write_joint_state_to_sim = original_joint_write
        if (
            write_attempts != {"root": 1, "joint": 1}
            or not torch.isfinite(observations["policy"]).all()
        ):
            raise RuntimeError("fresh-scene reset suppression contract failed")
        after = _verify_authored_state(
            environment=environment,
            arrays=arrays,
            profile=profile,
            torch=torch,
        )
        target_slot = ordinal
        local_case = _dynamic_case(case, ordinal=0, clip_index=0)
        action_channel_ids = tuple(
            environment.reference_profile.document["action"]["ordered_actuator_ids"]
        )
        contact_channel_ids = tuple(
            environment.reference_profile.document["observation"]["contact_order"]
        )
        accumulator = DynamicFeasibilityAccumulator(
            cases=(local_case,),
            action_channel_ids=action_channel_ids,
            contact_channel_ids=contact_channel_ids,
            contact_pair_ids=environment.contact_pair_ids,
            reset_safety_window_motor_ticks=10,
        )
        zero = torch.zeros(
            (vector_count, ACTION_CHANNELS),
            dtype=torch.float32,
            device=environment.device,
        )
        first_tick_impulses: tuple[int, ...] | None = None
        trajectory_diagnostics: list[dict[str, Any]] = []
        deferred_terminal_reset_count = 0
        original_episode_reset = environment._reset_idx

        def defer_terminal_reset(env_ids: Any | None) -> None:
            nonlocal deferred_terminal_reset_count
            deferred_terminal_reset_count += int(
                environment.num_envs
                if env_ids is None
                else env_ids.numel()
            )

        environment._reset_idx = defer_terminal_reset
        steps = 0
        try:
            while accumulator.completed_case_count == 0:
                observations, _, terminated, truncated, _ = environment.step(zero)
                steps += 1
                if not torch.isfinite(observations["policy"]).all():
                    raise RuntimeError("fresh-scene observation became non-finite")
                done = terminated | truncated
                tensors = _step_tensors_to_cpu(
                    environment=environment,
                    done=done,
                    truncated=truncated,
                )
                elapsed = int(tensors["elapsed_ticks"][target_slot].item())
                if elapsed == 1:
                    first_tick_impulses = _integer_row(
                        tensors["contact_pair_impulses"], target_slot
                    )
                trajectory_diagnostics.append(
                    _fresh_trajectory_sample(
                        environment=environment,
                        descriptor=descriptor,
                        arrays=arrays,
                        tensors=tensors,
                        tensor_index=target_slot,
                    )
                )
                _record_step(
                    accumulator=accumulator,
                    tensors=tensors,
                    tensor_index=target_slot,
                    assignment_ordinal=0,
                    clip_index=0,
                )
                if steps > case.horizon_motor_ticks:
                    raise RuntimeError(
                        "fresh-scene episode exceeded its progress bound"
                    )
        finally:
            environment._reset_idx = original_episode_reset
        if first_tick_impulses is None:
            raise RuntimeError("fresh-scene episode did not expose first-tick impulses")
        sections = accumulator.report_sections()
        row = dict(sections["phase_results"][0])
        row["case_ordinal"] = case.ordinal
        row["source_case_ordinal"] = case.source_case_ordinal
        row["first_tick_impulses"] = list(first_tick_impulses)
        report = {
            "schema_version": 1,
            "check": "TRAIN-4-ISAAC-CONTACT-MANIFOLD-FRESH-SCENE-WORKER",
            "status": "PASS",
            "case_ordinal": case.ordinal,
            "source_case_ordinal": case.source_case_ordinal,
            "clip_id": case.clip_id,
            "start_frame": case.frame_first,
            "target_vector_slot": target_slot,
            "vector_environment_count": vector_count,
            "overlay_usd_sha256": _sha256(overlay_path),
            "initialization": initialization_metrics,
            "ground_collision_lifecycle": {
                "disabled_before_initialization": ground_state[
                    "disabled_before_initialization"
                ],
                "enabled_before_contact_view": ground_state[
                    "enabled_before_contact_view"
                ],
            },
            "reference_overlay": reference_overlay,
            "trajectory_diagnostics": trajectory_diagnostics,
            "deferred_terminal_reset_count": deferred_terminal_reset_count,
            "post_create_state_write_attempts_suppressed": write_attempts,
            "post_create_root_or_joint_state_writes_executed": 0,
            "initial_state_verification": {"before_reset": before, "after_reset": after},
            "contact_pair_ids": list(environment.contact_pair_ids),
            "contact_pair_hard_limits_micronewton_seconds": list(
                environment.contact_pair_hard_limits_micronewton_seconds
            ),
            "phase_result": row,
            "optimizer_steps": 0,
            "training_runs": 0,
            "repository": _repository_state(),
        }
        _write_json(result_path, report)
    except BaseException as error:
        pending_error = error
        traceback.print_exc()
    finally:
        cleanup_failed = False
        if environment is not None:
            try:
                environment.close()
            except BaseException:
                cleanup_failed = True
                traceback.print_exc()
        if simulation_app is not None:
            try:
                simulation_app.close()
            except BaseException:
                cleanup_failed = True
                traceback.print_exc()
        if cleanup_failed and pending_error is None:
            raise RuntimeError("fresh-scene worker cleanup failed")
    if pending_error is not None:
        raise pending_error


def _run_partial_reset(
    *,
    args: argparse.Namespace,
    profile: dict[str, Any],
    descriptor: dict[str, Any],
    cases: tuple[ContactPrototypeCase, ...],
    report_path: Path,
    **paths: Any,
) -> dict[str, Any]:
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(paths["usd_path"])
    simulation_app = None
    environment = None
    pending_error: BaseException | None = None
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app
        import torch

        from next_lab.isaac_reference_env import (
            ACTION_CHANNELS,
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
            REFERENCE_OFFSETS,
        )
        from next_lab.reference_dynamic_feasibility import (
            DynamicFeasibilityAccumulator,
        )

        reference_overlay = profile["execution"]["reference_overlay"]
        if tuple(reference_overlay["observation_horizon_offsets_motor_ticks"]) != tuple(
            REFERENCE_OFFSETS
        ):
            raise RuntimeError("reference overlay offsets differ from the tracker")
        clip_ids = tuple(profile_case for profile_case in _ordered_clip_ids(cases))
        clip_index = {clip_id: index for index, clip_id in enumerate(clip_ids)}
        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = len(cases)
        cfg.sim.device = args.device
        cfg.seed = args.seed
        cfg.eligible_clip_ids = clip_ids
        cfg.phase_randomization = False
        cfg.fixed_horizon_motor_ticks = cases[0].horizon_motor_ticks
        cfg.diagnostic_exhaustive_phase_sweep_repeats = 1
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(paths["descriptor_path"]),
            profile_path=str(paths["reference_profile_path"]),
            corpus_root=str(paths["corpus_root"]),
            gate_report_path=str(paths["gate_report_path"]),
        )
        environment.diagnostic_episode_schedule = tuple(
            (
                clip_index[case.clip_id],
                case.frame_first,
                case.frame_last,
                0,
            )
            for case in cases
        )
        arrays_by_env = tuple(load_case_arrays(case) for case in cases)
        _install_reference_override(
            environment=environment,
            arrays_by_env=arrays_by_env,
            torch=torch,
        )
        original_reset = environment._reset_idx
        episode_generation = [-1] * len(cases)

        def reset_same_case_by_slot(env_ids: Any | None) -> None:
            selected = (
                environment.robot._ALL_INDICES
                if env_ids is None
                else env_ids
            )
            for env_id in selected.detach().cpu().tolist():
                environment._diagnostic_assignment_ordinal = int(env_id)
                single = torch.tensor(
                    [env_id], dtype=torch.int64, device=environment.device
                )
                original_reset(single)
                episode_generation[env_id] += 1

        environment._reset_idx = reset_same_case_by_slot
        environment.reset_diagnostic_episode_sequence()
        observations, _ = environment.reset()
        _rewrite_reference_root_link_velocity(
            environment, environment.robot._ALL_INDICES, torch
        )
        if not torch.isfinite(observations["policy"]).all():
            raise RuntimeError("partial-reset warmup observation is non-finite")
        action_channel_ids = tuple(
            environment.reference_profile.document["action"]["ordered_actuator_ids"]
        )
        contact_channel_ids = tuple(
            environment.reference_profile.document["observation"]["contact_order"]
        )
        dynamic_cases = tuple(
            _dynamic_case(
                case,
                ordinal=case.ordinal,
                clip_index=clip_index[case.clip_id],
            )
            for case in cases
        )
        accumulator = DynamicFeasibilityAccumulator(
            cases=dynamic_cases,
            action_channel_ids=action_channel_ids,
            contact_channel_ids=contact_channel_ids,
            contact_pair_ids=environment.contact_pair_ids,
            reset_safety_window_motor_ticks=10,
        )
        zero = torch.zeros(
            (len(cases), ACTION_CHANNELS),
            dtype=torch.float32,
            device=environment.device,
        )
        first_tick_impulses: list[tuple[int, ...] | None] = [None] * len(cases)
        executed_steps = 0
        while accumulator.completed_case_count < accumulator.case_count:
            generation_before = tuple(episode_generation)
            observations, _, terminated, truncated, _ = environment.step(zero)
            executed_steps += 1
            if not torch.isfinite(observations["policy"]).all():
                raise RuntimeError("partial-reset observation became non-finite")
            done = terminated | truncated
            tensors = _step_tensors_to_cpu(
                environment=environment,
                done=done,
                truncated=truncated,
            )
            for env_index, generation in enumerate(generation_before):
                if generation != 1:
                    continue
                elapsed = int(tensors["elapsed_ticks"][env_index].item())
                if elapsed == 1:
                    first_tick_impulses[env_index] = _integer_row(
                        tensors["contact_pair_impulses"], env_index
                    )
                _record_step(
                    accumulator=accumulator,
                    tensors=tensors,
                    tensor_index=env_index,
                    assignment_ordinal=env_index,
                    clip_index=clip_index[cases[env_index].clip_id],
                )
            reset_ids = torch.nonzero(done, as_tuple=False).flatten()
            if reset_ids.numel():
                _rewrite_reference_root_link_velocity(environment, reset_ids, torch)
            if executed_steps > 2 * (cases[0].horizon_motor_ticks + 1):
                raise RuntimeError("partial-reset probe exceeded its progress bound")
        if any(value is None for value in first_tick_impulses):
            raise RuntimeError("partial-reset probe missed first-tick impulses")
        sections = accumulator.report_sections()
        rows = []
        for row, impulses, case in zip(
            sections["phase_results"], first_tick_impulses, cases, strict=True
        ):
            item = dict(row)
            item["source_case_ordinal"] = case.source_case_ordinal
            item["first_tick_impulses"] = list(impulses or ())
            rows.append(item)
        sections["phase_results"] = rows
        report = {
            "schema_version": 1,
            "check": "TRAIN-4-ISAAC-CONTACT-MANIFOLD-INDEXED-PARTIAL-RESET",
            "status": "PASS",
            "method": {
                "warmup_episode_count_per_case": 1,
                "measured_episode_count_per_case": 1,
                "same_case_before_and_after_reset": True,
                "root_velocity_semantics": "root-link writer before next physics step",
                "executed_vector_motor_steps": executed_steps,
                "reference_overlay": reference_overlay,
            },
            "contact_pair_ids": list(environment.contact_pair_ids),
            "contact_pair_hard_limits_micronewton_seconds": list(
                environment.contact_pair_hard_limits_micronewton_seconds
            ),
            "results": sections,
            "optimizer_steps": 0,
            "training_runs": 0,
            "repository": _repository_state(),
        }
        _write_json(report_path, report)
        return report
    except BaseException as error:
        pending_error = error
        traceback.print_exc()
    finally:
        cleanup_failed = False
        if environment is not None:
            try:
                environment.close()
            except BaseException:
                cleanup_failed = True
                traceback.print_exc()
        if simulation_app is not None:
            try:
                simulation_app.close()
            except BaseException:
                cleanup_failed = True
                traceback.print_exc()
        if cleanup_failed and pending_error is None:
            raise RuntimeError("partial-reset probe cleanup failed")
    if pending_error is not None:
        raise pending_error
    raise RuntimeError("partial-reset probe completed without a report")


def _install_reference_override(
    *,
    environment: Any,
    arrays_by_env: Sequence[Mapping[str, np.ndarray]],
    torch: Any,
) -> None:
    if len(arrays_by_env) != environment.num_envs:
        raise ValueError("reference override vector width differs")
    common_names = set(arrays_by_env[0])
    if any(set(arrays) != common_names for arrays in arrays_by_env):
        raise ValueError("reference override fields differ by environment")
    tensors = {}
    for name in common_names:
        stacked = np.stack(
            [np.asarray(arrays[name]) for arrays in arrays_by_env]
        )
        if stacked.dtype in {np.dtype(np.uint8), np.dtype(np.uint16)}:
            stacked = stacked.astype(np.int64)
        tensors[name] = torch.from_numpy(stacked).to(environment.device)
    original = environment._reference_at
    all_ids = torch.arange(
        environment.num_envs, dtype=torch.int64, device=environment.device
    )

    def reference_at(
        name: str,
        frame: Any | None = None,
        env_ids: Any | None = None,
    ) -> Any:
        if name not in tensors:
            return original(name, frame, env_ids)
        selected_ids = all_ids if env_ids is None else env_ids
        selected_frame = (
            environment._cursor[selected_ids]
            if frame is None
            else frame
        )
        return overlay_bounded_reference_window(
            source_values=original(name, frame, env_ids),
            projected_by_environment=tensors[name],
            selected_environment_ids=selected_ids,
            selected_frames=selected_frame,
            episode_start_frames=environment._episode_start_frame[selected_ids],
            where=torch.where,
        )

    environment._reference_at = reference_at


def _fresh_trajectory_sample(
    *,
    environment: Any,
    descriptor: Mapping[str, Any],
    arrays: Mapping[str, np.ndarray],
    tensors: Mapping[str, Any],
    tensor_index: int,
) -> dict[str, Any]:
    reference_frame = int(tensors["reference_frame"][tensor_index].item())
    start_frame = int(tensors["start_frame"][tensor_index].item())
    relative_frame = reference_frame - start_frame
    if not 0 <= relative_frame < len(arrays["root_position_um"]):
        raise RuntimeError("fresh trajectory sample is outside the projection")
    current = environment._canonical_current()
    actual_root = (
        current["root_position_um"][tensor_index].detach().cpu().numpy()
    ).astype(np.int64)
    actual_quaternion = (
        current["root_quaternion_q1_30"][tensor_index]
        .detach()
        .cpu()
        .numpy()
    ).astype(np.int64)
    actual_joint = (
        current["joint_position_urad"][tensor_index].detach().cpu().numpy()
    ).astype(np.int64)
    actual_root_velocity = (
        current["root_linear_velocity_um_s"][tensor_index]
        .detach()
        .cpu()
        .numpy()
    ).astype(np.int64)
    actual_joint_velocity = (
        current["joint_velocity_urad_s"][tensor_index]
        .detach()
        .cpu()
        .numpy()
    ).astype(np.int64)
    reference_root = np.asarray(
        arrays["root_position_um"][relative_frame], dtype=np.int64
    )
    reference_quaternion = np.asarray(
        arrays["root_quaternion_q1_30"][relative_frame], dtype=np.int64
    )
    reference_joint = np.asarray(
        arrays["joint_position_urad"][relative_frame], dtype=np.int64
    )
    joint_error = actual_joint - reference_joint
    joint_lookup = {
        joint["joint_id"]: int(joint["dof_ordinal"])
        for joint in descriptor["joints"]
    }
    leg_suffixes = (
        "hip-pitch",
        "hip-roll",
        "hip-yaw",
        "knee",
        "ankle-pitch",
        "ankle-roll",
    )
    leg_ordinals = {
        side: tuple(
            joint_lookup[f"joint.{side}-{suffix}"] for suffix in leg_suffixes
        )
        for side in ("left", "right")
    }
    pair_indices = {
        side: environment.contact_pair_ids.index(
            f"ground:body.{side}-ankle-roll"
        )
        for side in ("left", "right")
    }
    return {
        "elapsed_motor_ticks": int(
            tensors["elapsed_ticks"][tensor_index].item()
        ),
        "reference_frame": reference_frame,
        "actual_root_position_micrometres": actual_root.tolist(),
        "reference_root_position_micrometres": reference_root.tolist(),
        "root_position_error_vector_micrometres": (
            actual_root - reference_root
        ).tolist(),
        "actual_root_linear_velocity_micrometres_per_second": (
            actual_root_velocity.tolist()
        ),
        "reference_root_linear_velocity_micrometres_per_second": np.asarray(
            arrays["root_linear_velocity_um_s"][relative_frame], dtype=np.int64
        ).tolist(),
        "reference_contacts": np.asarray(
            arrays["contacts"][relative_frame], dtype=np.int64
        ).tolist(),
        "reference_contact_modes": np.asarray(
            arrays["contact_modes"][relative_frame], dtype=np.int64
        ).tolist(),
        "actual_left_foot_collider_minimum_micrometres": (
            _foot_collider_minimum_micrometres(
                descriptor=descriptor,
                side="left",
                root_position_um=actual_root,
                root_quaternion_q1_30=actual_quaternion,
                joint_position_urad=actual_joint,
            )
        ),
        "reference_left_foot_collider_minimum_micrometres": (
            _foot_collider_minimum_micrometres(
                descriptor=descriptor,
                side="left",
                root_position_um=reference_root,
                root_quaternion_q1_30=reference_quaternion,
                joint_position_urad=reference_joint,
            )
        ),
        "actual_right_foot_collider_minimum_micrometres": (
            _foot_collider_minimum_micrometres(
                descriptor=descriptor,
                side="right",
                root_position_um=actual_root,
                root_quaternion_q1_30=actual_quaternion,
                joint_position_urad=actual_joint,
            )
        ),
        "reference_right_foot_collider_minimum_micrometres": (
            _foot_collider_minimum_micrometres(
                descriptor=descriptor,
                side="right",
                root_position_um=reference_root,
                root_quaternion_q1_30=reference_quaternion,
                joint_position_urad=reference_joint,
            )
        ),
        "maximum_absolute_joint_position_error_microradians": int(
            np.max(np.abs(joint_error))
        ),
        "maximum_absolute_left_leg_joint_position_error_microradians": int(
            np.max(np.abs(joint_error[list(leg_ordinals["left"])]))
        ),
        "maximum_absolute_right_leg_joint_position_error_microradians": int(
            np.max(np.abs(joint_error[list(leg_ordinals["right"])]))
        ),
        "actual_left_leg_joint_position_microradians": actual_joint[
            list(leg_ordinals["left"])
        ].tolist(),
        "reference_left_leg_joint_position_microradians": reference_joint[
            list(leg_ordinals["left"])
        ].tolist(),
        "left_leg_joint_position_error_microradians": joint_error[
            list(leg_ordinals["left"])
        ].tolist(),
        "actual_right_leg_joint_position_microradians": actual_joint[
            list(leg_ordinals["right"])
        ].tolist(),
        "reference_right_leg_joint_position_microradians": reference_joint[
            list(leg_ordinals["right"])
        ].tolist(),
        "right_leg_joint_position_error_microradians": joint_error[
            list(leg_ordinals["right"])
        ].tolist(),
        "maximum_absolute_joint_velocity_microradians_per_second": int(
            np.max(np.abs(actual_joint_velocity))
        ),
        "observed_contacts": _integer_row(
            tensors["observed_contacts"], tensor_index
        ),
        "left_foot_episode_maximum_impulse_micronewton_seconds": int(
            tensors["contact_pair_impulses"][
                tensor_index, pair_indices["left"]
            ].item()
        ),
        "right_foot_episode_maximum_impulse_micronewton_seconds": int(
            tensors["contact_pair_impulses"][
                tensor_index, pair_indices["right"]
            ].item()
        ),
    }


def _foot_collider_minimum_micrometres(
    *,
    descriptor: Mapping[str, Any],
    side: str,
    root_position_um: np.ndarray,
    root_quaternion_q1_30: np.ndarray,
    joint_position_urad: np.ndarray,
) -> int:
    positions, rotations = target_forward_kinematics(
        descriptor,
        root_position_um.astype(np.float64) / 1_000_000.0,
        root_quaternion_q1_30.astype(np.float64) / float(1 << 30),
        joint_position_urad.astype(np.float64) / 1_000_000.0,
    )
    body = next(
        item
        for item in descriptor["bodies"]
        if item["body_id"] == f"body.{side}-ankle-roll"
    )
    slot = int(body["body_slot"])
    minimum = min(
        collider_minimum_y(positions[slot], rotations[slot], collider)
        for collider in body["colliders"]
        if int(collider["contact_role"]) == 8
    )
    return int(np.floor(minimum * 1_000_000.0 + 1.0e-9))


def _verify_authored_state(
    *,
    environment: Any,
    arrays: Mapping[str, np.ndarray],
    profile: Mapping[str, Any],
    torch: Any,
) -> dict[str, Any]:
    expected = authored_root_state(arrays)
    root = environment.robot.data.root_link_state_w.detach().cpu().numpy()
    joint_position = (
        environment.robot.data.joint_pos[:, environment._dof_joint_ids]
        .detach()
        .cpu()
        .numpy()
    )
    joint_velocity = (
        environment.robot.data.joint_vel[:, environment._dof_joint_ids]
        .detach()
        .cpu()
        .numpy()
    )
    origins = environment.scene.env_origins.detach().cpu().numpy()
    expected_position = expected.position_m[None] + origins
    expected_quaternion = np.repeat(
        expected.quaternion_wxyz[None], environment.num_envs, axis=0
    )
    position_error = int(
        np.rint(
            np.max(np.linalg.norm(root[:, :3] - expected_position, axis=1))
            * 1_000_000.0
        )
    )
    root_quaternion_norm = np.linalg.norm(root[:, 3:7], axis=1)
    minimum_orientation = minimum_normalized_quaternion_dot_q1_30(
        root[:, 3:7], expected_quaternion
    )
    linear_error = int(
        np.rint(
            np.max(
                np.linalg.norm(
                    root[:, 7:10] - expected.linear_velocity_world_m_s[None],
                    axis=1,
                )
            )
            * 1_000_000.0
        )
    )
    angular_error = int(
        np.rint(
            np.max(
                np.linalg.norm(
                    root[:, 10:13] - expected.angular_velocity_world_rad_s[None],
                    axis=1,
                )
            )
            * 1_000_000.0
        )
    )
    expected_joint_position = arrays["joint_position_urad"][0] / 1_000_000.0
    expected_joint_velocity = arrays["joint_velocity_urad_s"][0] / 1_000_000.0
    joint_position_error = int(
        np.rint(np.max(np.abs(joint_position - expected_joint_position[None])) * 1_000_000.0)
    )
    joint_velocity_error = int(
        np.rint(np.max(np.abs(joint_velocity - expected_joint_velocity[None])) * 1_000_000.0)
    )
    result = {
        "maximum_root_position_error_micrometres": position_error,
        "minimum_raw_root_quaternion_norm_q1_30": int(
            np.floor(np.min(root_quaternion_norm) * (1 << 30))
        ),
        "minimum_root_orientation_absolute_dot_q1_30": minimum_orientation,
        "maximum_root_linear_velocity_error_micrometres_per_second": linear_error,
        "maximum_root_angular_velocity_error_microradians_per_second": angular_error,
        "maximum_joint_position_error_microradians": joint_position_error,
        "maximum_joint_velocity_error_microradians_per_second": joint_velocity_error,
    }
    bounds = profile["fresh_state_verification"]
    if (
        position_error > int(bounds["maximum_root_position_error_micrometres"])
        or minimum_orientation
        < int(bounds["minimum_root_orientation_absolute_dot_q1_30"])
        or linear_error
        > int(bounds["maximum_root_linear_velocity_error_micrometres_per_second"])
        or angular_error
        > int(bounds["maximum_root_angular_velocity_error_microradians_per_second"])
        or joint_position_error
        > int(bounds["maximum_joint_position_error_microradians"])
        or joint_velocity_error
        > int(bounds["maximum_joint_velocity_error_microradians_per_second"])
        or not all(
            np.all(np.isfinite(value))
            for value in (root, joint_position, joint_velocity)
        )
    ):
        raise RuntimeError(f"authored fresh-scene state differs: {result}")
    return result


def _rewrite_reference_root_link_velocity(
    environment: Any, env_ids: Any, torch: Any
) -> None:
    frame = environment._episode_start_frame[env_ids]
    root_linear_engine = (
        environment._reference_at("root_linear_velocity_um_s", frame, env_ids).to(
            torch.float32
        )
        / 1_000_000.0
    )
    x, y, z = root_linear_engine.unbind(dim=-1)
    root_linear = torch.stack((x, -z, y), dim=-1)
    yaw = (
        environment._reference_at("root_yaw_velocity_urad_s", frame, env_ids).to(
            torch.float32
        )
        / 1_000_000.0
    )
    root_angular = torch.stack(
        (torch.zeros_like(yaw), torch.zeros_like(yaw), yaw), dim=-1
    )
    environment.robot.write_root_link_velocity_to_sim(
        torch.cat((root_linear, root_angular), dim=-1), env_ids
    )
    environment._canonical_cache = None


def _step_tensors_to_cpu(
    *, environment: Any, done: Any, truncated: Any
) -> dict[str, Any]:
    from next_lab.reference_dynamic_feasibility import (
        CONTACT_SAFETY_REASONS,
        REQUIRED_SAFETY_REASONS,
    )

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
            for reason in REQUIRED_SAFETY_REASONS
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
            for reason in CONTACT_SAFETY_REASONS
        },
        "contact_pair_impulses": _cpu(
            environment.last_step_episode_contact_max_impulse_micronewton_seconds
        ),
        "forbidden_contact_mask": _cpu(environment.last_step_forbidden_contact_mask),
    }


def _record_step(
    *,
    accumulator: Any,
    tensors: Mapping[str, Any],
    tensor_index: int,
    assignment_ordinal: int,
    clip_index: int,
) -> None:
    from next_lab.reference_dynamic_feasibility import (
        CONTACT_SAFETY_REASONS,
        REQUIRED_SAFETY_REASONS,
    )

    accumulator.record_step(
        assignment_ordinal=assignment_ordinal,
        clip_index=clip_index,
        start_frame=int(tensors["start_frame"][tensor_index].item()),
        reference_frame=int(tensors["reference_frame"][tensor_index].item()),
        elapsed_motor_ticks=int(tensors["elapsed_ticks"][tensor_index].item()),
        done=bool(tensors["done"][tensor_index].item()),
        success=bool(tensors["success"][tensor_index].item()),
        failure=bool(tensors["failure"][tensor_index].item()),
        truncated=bool(tensors["truncated"][tensor_index].item()),
        safety={
            reason: bool(tensors["safety"][reason][tensor_index].item())
            for reason in REQUIRED_SAFETY_REASONS
        },
        tracking_lost=bool(tensors["tracking_lost"][tensor_index].item()),
        root_position_error_micrometres=int(
            tensors["root_position_error"][tensor_index].item()
        ),
        root_orientation_absolute_dot_q1_30=int(
            tensors["root_orientation_dot"][tensor_index].item()
        ),
        observed_joint_position_microradians=_integer_row(
            tensors["observed_joint_position"], tensor_index
        ),
        reference_joint_target_microradians=_integer_row(
            tensors["reference_joint_position"], tensor_index
        ),
        hard_rom_excess_microradians=_integer_row(
            tensors["hard_rom_excess"], tensor_index
        ),
        velocity_excess_microradians_per_second=_integer_row(
            tensors["velocity_excess"], tensor_index
        ),
        effort_envelope_violation=_boolean_row(
            tensors["effort_envelope_channels"], tensor_index
        ),
        observed_contacts=_integer_row(tensors["observed_contacts"], tensor_index),
        reference_contacts=_integer_row(
            tensors["reference_contacts"], tensor_index
        ),
        contact_pair_masks={
            reason: _boolean_row(
                tensors["contact_pair_masks"][reason], tensor_index
            )
            for reason in CONTACT_SAFETY_REASONS
        },
        contact_pair_maximum_impulses_micronewton_seconds=_integer_row(
            tensors["contact_pair_impulses"], tensor_index
        ),
        forbidden_contact_mask=int(
            tensors["forbidden_contact_mask"][tensor_index].item()
        ),
    )


def _dynamic_case(case: ContactPrototypeCase, *, ordinal: int, clip_index: int) -> Any:
    from next_lab.reference_dynamic_feasibility import DynamicAuditCase

    return DynamicAuditCase(
        ordinal=ordinal,
        clip_index=clip_index,
        clip_id=case.clip_id,
        split=case.split,
        start_frame=case.frame_first,
        terminal_frame=case.frame_last,
        repeat_index=0,
    )


def _summarize_rows(rows: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    from next_lab.reference_dynamic_feasibility import REQUIRED_SAFETY_REASONS

    counts = {reason: 0 for reason in REQUIRED_SAFETY_REASONS}
    failed = 0
    for row in rows:
        violation = row.get("first_required_safety_violation")
        reasons = set(violation.get("reasons", ())) if violation else set()
        failed += bool(reasons)
        for reason in reasons:
            counts[reason] += 1
    return {
        "case_count": len(rows),
        "coverage_complete": all(row["status"] in {"PASS", "FAIL"} for row in rows),
        "required_safety_failed_case_count": failed,
        "required_safety_event_counts": counts,
        "phase_results": list(rows),
    }


def _ordered_clip_ids(cases: Sequence[ContactPrototypeCase]) -> tuple[str, ...]:
    return tuple(dict.fromkeys(case.clip_id for case in cases))


def _worker_command(args: argparse.Namespace, ordinal: int) -> list[str]:
    return [
        sys.executable,
        str(Path(__file__).resolve()),
        "--source-audit",
        str(args.source_audit.resolve()),
        "--probe-profile",
        str(args.probe_profile.resolve()),
        "--descriptor",
        str(args.descriptor.resolve()),
        "--reference-profile",
        str(args.reference_profile.resolve()),
        "--prototype-manifest",
        str(args.prototype_manifest.resolve()),
        "--corpus-root",
        str(args.corpus_root.resolve()),
        "--gate-report",
        str(args.gate_report.resolve()),
        "--usd",
        str(args.usd.resolve()),
        "--output",
        str(args.output.resolve()),
        "--seed",
        str(args.seed),
        "--worker-case-ordinal",
        str(ordinal),
        "--device",
        args.device,
        "--headless",
    ]


def _partial_worker_command(args: argparse.Namespace) -> list[str]:
    command = _worker_command(args, -1)
    command.extend(("--partial-reset-worker",))
    return command


def _fresh_usd_path(output: Path, ordinal: int) -> Path:
    return output / "fresh-scene" / "usd" / f"case-{ordinal:02d}.usda"


def _fresh_result_path(output: Path, ordinal: int) -> Path:
    return output / "fresh-scene" / "results" / f"case-{ordinal:02d}.json"


def _partial_result_path(output: Path) -> Path:
    return output / "indexed-partial-reset.json"


def _external_output_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("reset-probe evidence must stay outside the repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    output.mkdir()
    return output


def _write_json(path: Path, value: Mapping[str, Any]) -> None:
    payload = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_bytes(payload)
    temporary.replace(path)


def _canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


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


def _cpu(value: Any) -> Any:
    return value.detach().cpu()


def _integer_row(value: Any, index: int) -> tuple[int, ...]:
    return tuple(int(item) for item in value[index].tolist())


def _boolean_row(value: Any, index: int) -> tuple[bool, ...]:
    return tuple(bool(item) for item in value[index].tolist())


if __name__ == "__main__":
    main()
