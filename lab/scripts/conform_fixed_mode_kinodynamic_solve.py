#!/usr/bin/env python3
"""Run report-only R140 fixed-mode kinodynamic solve conformance."""

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
    parser.add_argument("--authority-document", type=Path, required=True)
    parser.add_argument("--r139-report", type=Path, required=True)
    parser.add_argument("--r139-profile", type=Path, required=True)
    parser.add_argument("--r139-module", type=Path, required=True)
    parser.add_argument("--r139-tool", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.fixed_mode_kinodynamic_solve_conformance import (
        build_fixed_mode_kinodynamic_solve_conformance,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R140 conformance requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R140 repository identity changed during validation")

    report = build_fixed_mode_kinodynamic_solve_conformance(
        profile_path=args.profile,
        authority_document_path=args.authority_document,
        r139_report_path=args.r139_report,
        r139_profile_path=args.r139_profile,
        r139_module_path=args.r139_module,
        r139_tool_path=args.r139_tool,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r140-", dir=output.parent))
    try:
        report_path = staging / "fixed-mode-kinodynamic-solve-conformance.json"
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
                "result_transition": report["result_transition"],
                "report_sha256": report["report_sha256"],
                "synthetic_case_count": (
                    report["synthetic_reconstruction_gate_cases"]
                    + report["synthetic_qp_schema_cases"]
                    + report["synthetic_trust_cases"]
                    + report["synthetic_rounding_cases"]
                    + report["synthetic_branch_cases"]
                    + report["synthetic_cone_classification_cases"]
                    + report["synthetic_cone_separation_cases"]
                    + report["synthetic_numeric_valid_cases"]
                    + report["synthetic_numeric_invalid_cases"]
                ),
                "source_array_payloads_read": report["source_array_payloads_read"],
                "real_qp_solves": report["real_qp_solves"],
                "real_kinodynamic_solves": report["real_kinodynamic_solves"],
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
            raise RuntimeError(f"R140 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R140 output must stay outside repository")
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
