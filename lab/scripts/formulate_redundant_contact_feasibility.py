#!/usr/bin/env python3
"""Freeze the report-only R125 redundant-contact feasibility formulation."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.redundant_contact_feasibility_formulation import (
    build_redundant_contact_feasibility_formulation,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--research-report", type=Path, required=True)
    parser.add_argument("--research-profile", type=Path, required=True)
    parser.add_argument("--research-module", type=Path, required=True)
    parser.add_argument("--research-tool", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--r121-module", type=Path, required=True)
    parser.add_argument("--r121-tool", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R125 formulation requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    report = build_redundant_contact_feasibility_formulation(
        profile_path=args.profile,
        research_report_path=args.research_report,
        research_profile_path=args.research_profile,
        research_module_path=args.research_module,
        research_tool_path=args.research_tool,
        r121_report_path=args.r121_report,
        r121_profile_path=args.r121_profile,
        r121_module_path=args.r121_module,
        r121_tool_path=args.r121_tool,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r125-", dir=output.parent))
    try:
        (staging / "redundant-contact-feasibility-formulation.json").write_bytes(
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
                "local_system_reconstructions": report["local_system_reconstructions"],
                "local_system_solves": report["local_system_solves"],
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
            raise RuntimeError(f"R125 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R125 output must stay outside repository")
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
