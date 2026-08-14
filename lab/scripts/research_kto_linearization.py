#!/usr/bin/env python3
"""Run the report-only post-R115 analytic-contact linearization research."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.kto_linearization_research import build_kto_linearization_research


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r115-report", type=Path, required=True)
    parser.add_argument("--r114-profile", type=Path, required=True)
    parser.add_argument("--execution-profile", type=Path, required=True)
    parser.add_argument("--v9-profile", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--v7-case", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("post-R115 research requires a clean repository")
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
    report = build_kto_linearization_research(
        profile_path=args.profile,
        r115_report_path=args.r115_report,
        r114_profile_path=args.r114_profile,
        execution_profile_path=args.execution_profile,
        descriptor_bytes=descriptor_bytes,
        v9_profile_path=args.v9_profile,
        v9_complete_clip_path=args.v9_complete_clip,
        v7_case_path=args.v7_case,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-post-r115-research-", dir=output.parent)
    )
    try:
        (staging / "post-r115-kto-linearization-research.json").write_bytes(
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
            raise RuntimeError(
                f"post-R115 research validation {row['id']} failed:\n{detail}"
            )
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("post-R115 research output must stay outside repository")
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
