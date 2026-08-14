#!/usr/bin/env python3
"""Build the report-only TRAIN-4 R114 KTO execution formulation."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.quantization_aware_kto_formulation import (
    build_quantization_aware_kto_formulation,
    tracked_source_paths,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r113-report", type=Path, required=True)
    parser.add_argument("--r113-profile", type=Path, required=True)
    parser.add_argument("--r108-report", type=Path, required=True)
    parser.add_argument("--r108-profile", type=Path, required=True)
    parser.add_argument("--legacy-descriptor", type=Path, required=True)
    parser.add_argument("--v9-profile", type=Path, required=True)
    parser.add_argument("--v9-manifest", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--v7-profile", type=Path, required=True)
    parser.add_argument("--v7-manifest", type=Path, required=True)
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
        raise ValueError("R114 KTO formulation requires a clean repository")
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
    report = build_quantization_aware_kto_formulation(
        profile_path=args.profile,
        r113_report_path=args.r113_report,
        r113_profile_path=args.r113_profile,
        r108_report_path=args.r108_report,
        r108_profile_path=args.r108_profile,
        descriptor_bytes=descriptor_bytes,
        legacy_descriptor_path=args.legacy_descriptor,
        v9_profile_path=args.v9_profile,
        v9_manifest_path=args.v9_manifest,
        v9_complete_clip_path=args.v9_complete_clip,
        v7_profile_path=args.v7_profile,
        v7_manifest_path=args.v7_manifest,
        v7_case_path=args.v7_case,
        tracked_sources=tracked_source_paths(repository_root),
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-r114-kto-formulation-", dir=output.parent)
    )
    try:
        (staging / "quantization-aware-kto-execution-formulation.json").write_bytes(
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
                "total_scalar_count": report["variable_inventory"][
                    "total_scalar_count"
                ],
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
            raise RuntimeError(f"R114 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R114 KTO formulation report must stay outside repository")
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
