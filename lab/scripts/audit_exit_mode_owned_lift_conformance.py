#!/usr/bin/env python3
"""Consume the single report-only TRAIN-4 R132 conformance authority."""

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
    parser.add_argument("--r131-report", type=Path, required=True)
    parser.add_argument("--r131-profile", type=Path, required=True)
    parser.add_argument("--r131-module", type=Path, required=True)
    parser.add_argument("--r131-tool", type=Path, required=True)
    parser.add_argument("--r129-report", type=Path, required=True)
    parser.add_argument("--r129-profile", type=Path, required=True)
    parser.add_argument("--projection-module", type=Path, required=True)
    parser.add_argument("--r129-tool", type=Path, required=True)
    parser.add_argument("--r130-report", type=Path, required=True)
    parser.add_argument("--r130-profile", type=Path, required=True)
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
    from next_lab.exit_mode_owned_lift_conformance import (
        build_exit_mode_owned_lift_conformance_report,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R132 conformance requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R132 repository identity changed during validation")
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
        raise ValueError("R132 repository changed during descriptor export")

    report = build_exit_mode_owned_lift_conformance_report(
        profile_path=args.profile,
        r131_report_path=args.r131_report,
        r131_profile_path=args.r131_profile,
        r131_module_path=args.r131_module,
        r131_tool_path=args.r131_tool,
        r129_report_path=args.r129_report,
        r129_profile_path=args.r129_profile,
        projection_module_path=args.projection_module,
        r129_tool_path=args.r129_tool,
        r130_report_path=args.r130_report,
        r130_profile_path=args.r130_profile,
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
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r132-", dir=output.parent))
    try:
        report_path = staging / "exit-mode-owned-lift-conformance.json"
        report_path.write_bytes(
            (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        )
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    aggregate = report["conformance_result"]["aggregate"]
    print(
        json.dumps(
            {
                "status": report["status"],
                "invalid_reason": report["invalid_reason"],
                "gate_decision": report["gate_decision"],
                "result_transition": report["result_transition"],
                "report_sha256": report["report_sha256"],
                "real_anchor_rows": report["real_anchor_rows"],
                "changed_base_velocity_rows": aggregate[
                    "changed_base_velocity_row_count"
                ],
                "projected_joint_velocity_violation_rows": aggregate[
                    "rows_with_projected_joint_velocity_violation"
                ],
                "controller_schedule_derivations": report[
                    "controller_schedule_derivations"
                ],
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
            raise RuntimeError(f"R132 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R132 output must stay outside repository")
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
