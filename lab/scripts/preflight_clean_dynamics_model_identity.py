"""Build the report-only TRAIN-4 R113 clean model-identity preflight."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.clean_dynamics_model_identity_preflight import (
    build_clean_dynamics_model_identity_preflight,
    isaac_source_paths,
    tracked_source_paths,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r112-report", type=Path, required=True)
    parser.add_argument("--r112-profile", type=Path, required=True)
    parser.add_argument("--r110-report", type=Path, required=True)
    parser.add_argument("--r110-profile", type=Path, required=True)
    parser.add_argument("--r112-bundle", type=Path, required=True)
    parser.add_argument("--isaaclab-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R113 model-identity preflight requires a clean repository")
    isaac_root = args.isaaclab_root.resolve()
    isaac_repository = _isaac_repository_state(isaac_root)
    if isaac_repository["dirty"]:
        raise ValueError("R113 requires a clean pinned Isaac Lab repository")
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
    bundle = args.r112_bundle.resolve()
    report = build_clean_dynamics_model_identity_preflight(
        profile_path=args.profile,
        r112_report_path=args.r112_report,
        r112_profile_path=args.r112_profile,
        r110_report_path=args.r110_report,
        r110_profile_path=args.r110_profile,
        descriptor_bytes=descriptor_bytes,
        translation_manifest_path=bundle / "translation-manifest.json",
        humanoid_usd_path=bundle / "humanoid.usda",
        ground_usd_path=bundle / "ground.usda",
        tracked_sources=tracked_source_paths(repository_root),
        isaac_sources=isaac_source_paths(isaac_root),
        isaac_repository=isaac_repository,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-r113-model-identity-", dir=output.parent)
    )
    try:
        (staging / "clean-dynamics-model-identity-preflight.json").write_bytes(
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
                "model_identity_preflights": report["model_identity_preflights"],
                "solver_runs": report["solver_runs"],
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
            raise RuntimeError(f"R113 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R113 model-identity report must stay outside repository")
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
