#!/usr/bin/env python3
"""Consume the single report-only TRAIN-4 R133 schedule authority."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

THREAD_ENVIRONMENT = {
    "BLIS_NUM_THREADS": "1",
    "MKL_NUM_THREADS": "1",
    "NUMEXPR_NUM_THREADS": "1",
    "OMP_NUM_THREADS": "1",
    "OPENBLAS_NUM_THREADS": "1",
    "VECLIB_MAXIMUM_THREADS": "1",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r132-report", type=Path, required=True)
    parser.add_argument("--r132-profile", type=Path, required=True)
    parser.add_argument("--r132-module", type=Path, required=True)
    parser.add_argument("--r132-tool", type=Path, required=True)
    parser.add_argument("--r129-report", type=Path, required=True)
    parser.add_argument("--r129-profile", type=Path, required=True)
    parser.add_argument("--projection-module", type=Path, required=True)
    parser.add_argument("--r129-tool", type=Path, required=True)
    parser.add_argument("--r130-report", type=Path, required=True)
    parser.add_argument("--r130-profile", type=Path, required=True)
    parser.add_argument("--r130-execution-module", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--r113-report", type=Path, required=True)
    parser.add_argument("--r113-profile", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
    parser.add_argument("--r120-profile", type=Path, required=True)
    parser.add_argument("--r120-cache", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--collocation-lift-module", type=Path, required=True)
    parser.add_argument("--dynamics-kernel", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.exit_mode_owned_projected_schedule_execution import (
        execute_and_build_exit_mode_owned_projected_schedule_report,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R133 execution requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R133 repository identity changed during validation")
    descriptor_bytes = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "next_motor",
            "--example",
            "export_biomechanics_isaac_mirror_v2",
        ],
        cwd=repository_root,
        check=True,
        capture_output=True,
    ).stdout
    if _repository_state(repository_root) != repository:
        raise ValueError("R133 repository changed during descriptor export")
    report = execute_and_build_exit_mode_owned_projected_schedule_report(
        profile_path=args.profile,
        r132_report_path=args.r132_report,
        r132_profile_path=args.r132_profile,
        r132_module_path=args.r132_module,
        r132_tool_path=args.r132_tool,
        r129_report_path=args.r129_report,
        r129_profile_path=args.r129_profile,
        projection_module_path=args.projection_module,
        r129_tool_path=args.r129_tool,
        r130_report_path=args.r130_report,
        r130_profile_path=args.r130_profile,
        r130_execution_module_path=args.r130_execution_module,
        r121_report_path=args.r121_report,
        r121_profile_path=args.r121_profile,
        r113_report_path=args.r113_report,
        r113_profile_path=args.r113_profile,
        r120_report_path=args.r120_report,
        r120_profile_path=args.r120_profile,
        r120_cache_path=args.r120_cache,
        v9_complete_clip_path=args.v9_complete_clip,
        collocation_lift_module_path=args.collocation_lift_module,
        dynamics_kernel_path=args.dynamics_kernel,
        descriptor_bytes=descriptor_bytes,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
        execution_environment=THREAD_ENVIRONMENT,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r133-", dir=output.parent))
    try:
        report_path = staging / "exit-mode-owned-projected-schedule.json"
        report_path.write_bytes(
            (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        )
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    schedule = report["projected_fixed_pd_schedule_audit"]
    events = report["controller_activation_event_audit"]
    print(
        json.dumps(
            {
                "status": report["status"],
                "invalid_reason": report["invalid_reason"],
                "gate_decision": report["gate_decision"],
                "result_transition": report["result_transition"],
                "report_sha256": report["report_sha256"],
                "projection_rows": report["projection_result"]["aggregate"][
                    "collocation_rows_recorded"
                ],
                "changed_base_velocity_rows": report["edge_lift_audit"][
                    "changed_base_velocity_rows"
                ],
                "schedule_status": None if schedule is None else schedule["status"],
                "velocity_violations": (
                    None
                    if schedule is None
                    else schedule["activation_counts"]["velocity_violation"]
                ),
                "infeasible_effort_envelopes": (
                    None
                    if schedule is None
                    else schedule["activation_counts"]["infeasible_effort_envelope"]
                ),
                "controller_activation_events": (
                    None if events is None else events["event_count"]
                ),
                "inverse_dynamics_system_assemblies": report[
                    "inverse_dynamics_system_assemblies"
                ],
                "physx_scene_runs": report["physx_scene_runs"],
                "training_runs": report["training_runs"],
                "output": str(output),
            },
            sort_keys=True,
        )
    )


def _run_validations(
    repository: Path, commands: list[dict[str, Any]]
) -> list[dict[str, str]]:
    results = []
    for row in commands:
        environment = os.environ.copy()
        environment.update(row.get("environment", {}))
        result = subprocess.run(
            row["arguments"],
            cwd=repository,
            check=False,
            capture_output=True,
            text=True,
            env=environment,
        )
        if result.returncode != 0:
            detail = (result.stdout + result.stderr)[-4000:]
            raise RuntimeError(f"R133 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R133 output must stay outside repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _repository_state(repository: Path) -> dict[str, Any]:
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
    return {
        "commit": commit,
        "dirty": bool(dirty_paths),
        "dirty_paths": dirty_paths,
    }


if __name__ == "__main__":
    main()
