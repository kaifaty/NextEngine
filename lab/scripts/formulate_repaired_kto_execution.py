#!/usr/bin/env python3
"""Freeze the solver-free TRAIN-4 R119 repaired-KTO execution contract."""

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

from next_lab.repaired_kto_execution_formulation import (
    build_repaired_kto_execution_formulation,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r118-report", type=Path, required=True)
    parser.add_argument("--r118-profile", type=Path, required=True)
    parser.add_argument("--r117-report", type=Path, required=True)
    parser.add_argument("--r117-profile", type=Path, required=True)
    parser.add_argument("--r114-report", type=Path, required=True)
    parser.add_argument("--r114-profile", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R119 formulation requires a clean repository")
    solver_import_audit = {
        "osqp_module_loaded": "osqp" in sys.modules,
        "solver_module_imported": (
            "next_lab.quantization_aware_kto_solver" in sys.modules
        ),
        "execution_module_imported": (
            "next_lab.quantization_aware_kto_execution" in sys.modules
        ),
        "solver_execution_requested": False,
    }
    if any(solver_import_audit.values()):
        raise ValueError("R119 process imported or requested solver work")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if "osqp" in sys.modules:
        raise ValueError("R119 validations contaminated the formulation process")
    report = build_repaired_kto_execution_formulation(
        profile_path=args.profile,
        r118_report_path=args.r118_report,
        r118_profile_path=args.r118_profile,
        r117_report_path=args.r117_report,
        r117_profile_path=args.r117_profile,
        r114_report_path=args.r114_report,
        r114_profile_path=args.r114_profile,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
        solver_import_audit=solver_import_audit,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r119-", dir=output.parent))
    try:
        (staging / "repaired-kto-execution-formulation.json").write_bytes(
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
                "replacement_contact_rows": report[
                    "repaired_contact_row_inventory_audit"
                ]["replacement_row_count"],
                "resulting_total_constraint_rows": report[
                    "repaired_contact_row_inventory_audit"
                ]["resulting_total_constraint_row_count"],
                "qp_solves": report["qp_solves"],
                "kto_solves": report["kto_solves"],
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
            raise RuntimeError(f"R119 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R119 formulation report must stay outside repository")
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
