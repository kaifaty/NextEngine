#!/usr/bin/env python3
"""Build the report-only TRAIN-4 R109 dynamics-model identity audit."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.dynamics_model_identity_preflight import (
    build_dynamics_model_identity_preflight,
    isaac_source_paths,
    tracked_source_paths,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r108-report", type=Path, required=True)
    parser.add_argument("--r108-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--translation-manifest", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--isaaclab-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R109 model-identity preflight requires a clean repository")
    isaac_root = args.isaaclab_root.resolve()
    isaac_repository = _isaac_repository_state(isaac_root)
    if isaac_repository["dirty"]:
        raise ValueError("R109 requires a clean pinned Isaac Lab repository")
    report = build_dynamics_model_identity_preflight(
        profile_path=args.profile,
        r108_report_path=args.r108_report,
        r108_profile_path=args.r108_profile,
        descriptor_path=args.descriptor,
        translation_manifest_path=args.translation_manifest,
        usd_path=args.usd,
        tracked_sources=tracked_source_paths(repository_root),
        isaac_sources=isaac_source_paths(isaac_root),
        isaac_repository=isaac_repository,
        tool_path=Path(__file__),
        repository={**repository, "root": str(repository_root)},
    )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-dynamics-model-identity-preflight-",
            dir=output.parent,
        )
    )
    try:
        (staging / "dynamics-model-identity-preflight.json").write_bytes(
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
                "blocking_reasons": report["model_identity_result"]["blocking_reasons"],
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
        raise ValueError("model-identity report must stay outside repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _repository_state(repository: Path) -> dict[str, Any]:
    commit = _git(repository, "rev-parse", "HEAD")
    dirty_paths = _git(repository, "status", "--short").splitlines()
    return {
        "commit": commit,
        "dirty": bool(dirty_paths),
        "dirty_paths": dirty_paths,
    }


def _isaac_repository_state(repository: Path) -> dict[str, Any]:
    dirty_paths = _git(repository, "status", "--short").splitlines()
    return {
        "commit": _git(repository, "rev-parse", "HEAD"),
        "tag": _git(repository, "describe", "--tags", "--exact-match"),
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
