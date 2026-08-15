#!/usr/bin/env python3
"""Freeze the solver-free TRAIN-4 R121 fixed-PD ID execution contract."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    build_fixed_pd_inverse_dynamics_execution_formulation,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r108-report", type=Path, required=True)
    parser.add_argument("--r108-profile", type=Path, required=True)
    parser.add_argument("--r113-report", type=Path, required=True)
    parser.add_argument("--r113-profile", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
    parser.add_argument("--r120-profile", type=Path, required=True)
    parser.add_argument("--r120-cache", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R121 formulation requires a clean repository")
    solver_import_audit = {
        "osqp_module_loaded": "osqp" in sys.modules,
        "pinocchio_module_loaded": "pinocchio" in sys.modules,
        "inverse_dynamics_solver_module_imported": (
            "next_lab.fixed_pd_inverse_dynamics_solver" in sys.modules
        ),
        "inverse_dynamics_execution_module_imported": (
            "next_lab.fixed_pd_inverse_dynamics_execution" in sys.modules
        ),
        "solver_execution_requested": False,
    }
    if any(solver_import_audit.values()):
        raise ValueError("R121 process imported or requested solver work")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if "osqp" in sys.modules or "pinocchio" in sys.modules:
        raise ValueError("R121 validations contaminated the formulation process")
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
    report = build_fixed_pd_inverse_dynamics_execution_formulation(
        profile_path=args.profile,
        r108_report_path=args.r108_report,
        r108_profile_path=args.r108_profile,
        r113_report_path=args.r113_report,
        r113_profile_path=args.r113_profile,
        r120_report_path=args.r120_report,
        r120_profile_path=args.r120_profile,
        r120_cache_path=args.r120_cache,
        v9_complete_clip_path=args.v9_complete_clip,
        descriptor_bytes=descriptor_bytes,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
        solver_import_audit=solver_import_audit,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r121-", dir=output.parent))
    try:
        (staging / "fixed-pd-inverse-dynamics-execution-formulation.json").write_bytes(
            (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        )
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    print(
        json.dumps(
            {
                "status": report["status"],
                "gate_decision": report["gate_decision"],
                "report_sha256": report["report_sha256"],
                "decision_scalars": report["system_inventory_audit"][
                    "total_decision_scalar_count"
                ],
                "equality_rows": report["system_inventory_audit"][
                    "total_equality_row_count"
                ],
                "friction_cones": report["system_inventory_audit"][
                    "friction_second_order_cone_count"
                ],
                "inverse_dynamics_solves": report["inverse_dynamics_solves"],
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
            raise RuntimeError(f"R121 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R121 formulation report must stay outside repository")
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
