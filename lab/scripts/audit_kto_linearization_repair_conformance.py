#!/usr/bin/env python3
"""Run the solver-free TRAIN-4 R118 exact-kernel conformance audit."""

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

from next_lab.kto_linearization_repair_conformance import (
    build_kto_linearization_repair_conformance,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r117-report", type=Path, required=True)
    parser.add_argument("--r117-profile", type=Path, required=True)
    parser.add_argument("--v9-profile", type=Path, required=True)
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
        raise ValueError("R118 conformance requires a clean repository")
    solver_import_audit = {
        "osqp_module_loaded": "osqp" in sys.modules,
        "solver_module_imported": (
            "next_lab.quantization_aware_kto_solver" in sys.modules
        ),
        "solver_execution_requested": False,
    }
    if any(solver_import_audit.values()):
        raise ValueError("R118 process imported or requested solver work")
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
    if "osqp" in sys.modules:
        raise ValueError("R118 descriptor export contaminated the audit process")
    report = build_kto_linearization_repair_conformance(
        profile_path=args.profile,
        r117_report_path=args.r117_report,
        r117_profile_path=args.r117_profile,
        descriptor_bytes=descriptor_bytes,
        v9_profile_path=args.v9_profile,
        v9_complete_clip_path=args.v9_complete_clip,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
        solver_import_audit=solver_import_audit,
    )
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-r118-conformance-", dir=output.parent)
    )
    try:
        (staging / "kto-linearization-repair-conformance.json").write_bytes(
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
                "active_point_frame_count": report["all_active_function_identity"][
                    "active_point_frame_count"
                ],
                "anchor_count": len(report["deterministic_anchor_audits"]),
                "qp_solves": report["qp_solves"],
                "kto_solves": report["kto_solves"],
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
            raise RuntimeError(f"R118 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R118 conformance report must stay outside repository")
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
