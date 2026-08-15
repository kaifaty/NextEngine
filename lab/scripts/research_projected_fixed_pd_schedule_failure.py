#!/usr/bin/env python3
"""Run report-only R130-RC1 projected-schedule failure research."""

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
    parser.add_argument("--r130-report", type=Path, required=True)
    parser.add_argument("--r130-profile", type=Path, required=True)
    parser.add_argument("--r130-execution-module", type=Path, required=True)
    parser.add_argument("--r130-tool", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.projected_fixed_pd_schedule_failure_research import (
        build_projected_schedule_failure_research,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("post-R130 research requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("post-R130 repository identity changed during validation")
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
        raise ValueError("post-R130 repository changed during descriptor export")

    report = build_projected_schedule_failure_research(
        profile_path=args.profile,
        r130_report_path=args.r130_report,
        r130_profile_path=args.r130_profile,
        r130_execution_module_path=args.r130_execution_module,
        r130_tool_path=args.r130_tool,
        r121_report_path=args.r121_report,
        r121_profile_path=args.r121_profile,
        descriptor_bytes=descriptor_bytes,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r130-rc1-", dir=output.parent))
    try:
        report_path = staging / "post-r130-projected-schedule-research.json"
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
                "finding": report["finding"],
                "gate_decision": report["gate_decision"],
                "report_sha256": report["report_sha256"],
                "research_audits": report["research_audits"],
                "state_projection_systems": report["state_projection_systems"],
                "inverse_dynamics_system_assemblies": report[
                    "inverse_dynamics_system_assemblies"
                ],
                "kinodynamic_solves": report["kinodynamic_solves"],
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
            raise RuntimeError(f"post-R130 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("post-R130 output must stay outside repository")
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
