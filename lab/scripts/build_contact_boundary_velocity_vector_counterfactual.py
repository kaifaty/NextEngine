#!/usr/bin/env python3
"""Build the hash-closed frame-0 velocity-vector TRAIN-4 R101 input."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.contact_boundary_velocity_vector_counterfactual import (
    build_boundary_velocity_vector_counterfactual,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--v7-manifest", type=Path, required=True)
    parser.add_argument("--v9-manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository = _repository_state()
    if repository["dirty"]:
        raise RuntimeError(
            "boundary-velocity-vector input requires a clean repository"
        )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-boundary-velocity-vector-",
            dir=output.parent,
        )
    )
    try:
        report = build_boundary_velocity_vector_counterfactual(
            profile_path=args.profile,
            source_audit_path=args.source_audit,
            v7_manifest_path=args.v7_manifest,
            v9_manifest_path=args.v9_manifest,
            staging_directory=staging,
            tool_path=Path(__file__),
            repository=repository,
        )
        (staging / "prototype-manifest.json").write_bytes(
            (json.dumps(report, indent=2, sort_keys=True) + "\n").encode(
                "utf-8"
            )
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
                "manifest_sha256": report["manifest_sha256"],
                "output": str(output),
                "changed_array_element_count": report[
                    "offline_verification"
                ]["changed_array_element_count"],
                "optimizer_steps": 0,
                "training_runs": 0,
                "physx_runs": 0,
            },
            indent=2,
            sort_keys=True,
        )
    )


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("counterfactual output must stay outside the repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _repository_state() -> dict[str, Any]:
    repository = Path(__file__).resolve().parents[2]
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
