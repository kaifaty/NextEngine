#!/usr/bin/env python3
"""Build the report-only TRAIN-4 R110 material/point-force formulation."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.canonical_material_point_force_formulation import (
    build_canonical_material_point_force_formulation,
    tracked_source_paths,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r109-report", type=Path, required=True)
    parser.add_argument("--r109-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--v9-profile", type=Path, required=True)
    parser.add_argument("--v9-manifest", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--physx-sdk-manifest", type=Path, required=True)
    parser.add_argument("--px-material-header", type=Path, required=True)
    parser.add_argument("--px-shape-header", type=Path, required=True)
    parser.add_argument("--px-contact-modify-header", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R110 repair formulation requires a clean repository")
    report = build_canonical_material_point_force_formulation(
        profile_path=args.profile,
        r109_report_path=args.r109_report,
        r109_profile_path=args.r109_profile,
        descriptor_path=args.descriptor,
        v9_profile_path=args.v9_profile,
        v9_manifest_path=args.v9_manifest,
        v9_complete_clip_path=args.v9_complete_clip,
        physx_sdk_manifest_path=args.physx_sdk_manifest,
        px_material_header_path=args.px_material_header,
        px_shape_header_path=args.px_shape_header,
        px_contact_modify_header_path=args.px_contact_modify_header,
        tracked_sources=tracked_source_paths(repository_root),
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-canonical-material-point-force-formulation-",
            dir=output.parent,
        )
    )
    try:
        (staging / "canonical-material-point-force-formulation.json").write_bytes(
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
                "repair_status": report["repair_assessment"]["status"],
                "runtime_changes": report["runtime_changes"],
                "solver_runs": report["solver_runs"],
                "physx_runs": report["physx_runs"],
                "output": str(output),
            },
            sort_keys=True,
        )
    )


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R110 formulation report must stay outside repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _repository_state(repository: Path) -> dict[str, Any]:
    dirty_paths = _git(repository, "status", "--short").splitlines()
    return {
        "commit": _git(repository, "rev-parse", "HEAD"),
        "dirty": bool(dirty_paths),
        "dirty_paths": dirty_paths,
    }


def _git(repository: Path, *arguments: str) -> str:
    return subprocess.run(
        ["git", *arguments],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


if __name__ == "__main__":
    main()
