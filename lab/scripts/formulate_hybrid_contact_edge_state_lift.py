#!/usr/bin/env python3
"""Run report-only R131 hybrid contact-edge state-lift formulation."""

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
    parser.add_argument("--r130-rc1-report", type=Path, required=True)
    parser.add_argument("--r130-rc1-profile", type=Path, required=True)
    parser.add_argument("--r130-rc1-module", type=Path, required=True)
    parser.add_argument("--r130-rc1-tool", type=Path, required=True)
    parser.add_argument("--r130-report", type=Path, required=True)
    parser.add_argument("--r130-profile", type=Path, required=True)
    parser.add_argument("--r130-execution-module", type=Path, required=True)
    parser.add_argument("--r130-tool", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--collocation-lift-module", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.hybrid_contact_edge_state_lift_formulation import (
        build_hybrid_contact_edge_state_lift_formulation,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R131 formulation requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R131 repository identity changed during validation")

    report = build_hybrid_contact_edge_state_lift_formulation(
        profile_path=args.profile,
        r130_rc1_report_path=args.r130_rc1_report,
        r130_rc1_profile_path=args.r130_rc1_profile,
        r130_rc1_module_path=args.r130_rc1_module,
        r130_rc1_tool_path=args.r130_rc1_tool,
        r130_report_path=args.r130_report,
        r130_profile_path=args.r130_profile,
        r130_execution_module_path=args.r130_execution_module,
        r130_tool_path=args.r130_tool,
        r121_report_path=args.r121_report,
        r121_profile_path=args.r121_profile,
        collocation_lift_module_path=args.collocation_lift_module,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r131-", dir=output.parent))
    try:
        report_path = staging / "hybrid-contact-edge-state-lift-formulation.json"
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
                "selected_alternative": report["alternative_decision_audit"][
                    "selected_alternative"
                ],
                "contact_exit_boundaries": report["contact_edge_inventory_audit"][
                    "contact_exit_boundary_count"
                ],
                "projection_solves": report["projection_solves"],
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
            raise RuntimeError(f"R131 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R131 output must stay outside repository")
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
