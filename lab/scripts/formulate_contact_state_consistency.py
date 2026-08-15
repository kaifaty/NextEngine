#!/usr/bin/env python3
"""Freeze the report-only R128 contact-state consistency formulation."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r127-rc1-report", type=Path, required=True)
    parser.add_argument("--r127-rc1-profile", type=Path, required=True)
    parser.add_argument("--r127-rc1-module", type=Path, required=True)
    parser.add_argument("--r127-rc1-tool", type=Path, required=True)
    parser.add_argument("--r127-report", type=Path, required=True)
    parser.add_argument("--r127-profile", type=Path, required=True)
    parser.add_argument("--r127-module", type=Path, required=True)
    parser.add_argument("--r127-tool", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--r121-module", type=Path, required=True)
    parser.add_argument("--r121-tool", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
    parser.add_argument("--r120-profile", type=Path, required=True)
    parser.add_argument("--r120-cache", type=Path, required=True)
    parser.add_argument("--gauge-kernel", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    from next_lab.contact_state_consistency_formulation import (
        build_contact_state_consistency_formulation,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R128 formulation requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R128 repository identity changed during validation")
    report = build_contact_state_consistency_formulation(
        profile_path=args.profile,
        r127_rc1_report_path=args.r127_rc1_report,
        r127_rc1_profile_path=args.r127_rc1_profile,
        r127_rc1_module_path=args.r127_rc1_module,
        r127_rc1_tool_path=args.r127_rc1_tool,
        r127_report_path=args.r127_report,
        r127_profile_path=args.r127_profile,
        r127_module_path=args.r127_module,
        r127_tool_path=args.r127_tool,
        r121_report_path=args.r121_report,
        r121_profile_path=args.r121_profile,
        r121_module_path=args.r121_module,
        r121_tool_path=args.r121_tool,
        r120_report_path=args.r120_report,
        r120_profile_path=args.r120_profile,
        r120_cache_path=args.r120_cache,
        gauge_kernel_path=args.gauge_kernel,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r128-", dir=output.parent))
    try:
        report_path = staging / "contact-state-consistency-formulation.json"
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
                "selected_alternative": report["alternative_decision_audit"][
                    "selected_alternative"
                ],
                "report_sha256": report["report_sha256"],
                "state_projections": report["state_projections"],
                "projection_solves": report["projection_solves"],
                "inverse_dynamics_solves": report["inverse_dynamics_solves"],
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
            raise RuntimeError(f"R128 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R128 output must stay outside repository")
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
