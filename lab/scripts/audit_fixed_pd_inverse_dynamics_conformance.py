#!/usr/bin/env python3
"""Run the report-only TRAIN-4 R122 dynamics-kernel conformance audit."""

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

from next_lab.fixed_pd_inverse_dynamics_conformance import (
    build_fixed_pd_inverse_dynamics_conformance,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--r113-report", type=Path, required=True)
    parser.add_argument("--r113-profile", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
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
        raise ValueError("R122 conformance requires a clean repository")
    solver_import_audit = {
        "osqp_module_loaded": "osqp" in sys.modules,
        "scipy_module_loaded": "scipy" in sys.modules,
        "pinocchio_module_loaded": "pinocchio" in sys.modules,
        "r123_execution_module_imported": (
            "next_lab.fixed_pd_inverse_dynamics_execution" in sys.modules
        ),
        "r123_system_solve_requested": False,
    }
    if any(solver_import_audit.values()):
        raise ValueError("R122 process imported or requested R123 solver work")
    validations = _run_validations(repository_root, profile["validation_commands"])
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
    if any(name in sys.modules for name in ("osqp", "scipy", "pinocchio")):
        raise ValueError("R122 descriptor export contaminated the audit process")
    report = build_fixed_pd_inverse_dynamics_conformance(
        profile_path=args.profile,
        r121_report_path=args.r121_report,
        r121_profile_path=args.r121_profile,
        r113_report_path=args.r113_report,
        r113_profile_path=args.r113_profile,
        r120_report_path=args.r120_report,
        r120_cache_path=args.r120_cache,
        v9_complete_clip_path=args.v9_complete_clip,
        descriptor_bytes=descriptor_bytes,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
        solver_import_audit=solver_import_audit,
    )
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-r122-conformance-", dir=output.parent)
    )
    try:
        report_path = (
            staging / "fixed-pd-inverse-dynamics-implementation-conformance.json"
        )
        report_path.write_bytes(
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
                "anchor_count": len(report["anchor_audits"]),
                "maximum_mass_condition_number": max(
                    row["mass_matrix"]["condition_number"]
                    for row in report["anchor_audits"]
                ),
                "r123_local_system_solves": report["r123_local_system_solves"],
                "inverse_dynamics_execution_runs": report[
                    "inverse_dynamics_execution_runs"
                ],
                "physx_scene_runs": report["physx_scene_runs"],
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
            raise RuntimeError(f"R122 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R122 conformance report must stay outside repository")
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
