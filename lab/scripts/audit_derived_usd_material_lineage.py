"""Build the static-only TRAIN-4 R112 derived material-lineage audit."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.derived_usd_material_lineage import (
    build_derived_usd_material_lineage_report,
    tracked_source_paths,
)
from next_lab.usd_translation import translate_to_store


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--r111-report", type=Path, required=True)
    parser.add_argument("--r111-profile", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root, profile)
    if repository["dirty"]:
        raise ValueError("R112 implementation audit requires a clean repository")
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
    descriptor = json.loads(descriptor_bytes)
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-r112-derived-material-", dir=output.parent)
    )
    try:
        manifest = translate_to_store(descriptor, staging, repository_root)
        bundle = (
            staging
            / "derived"
            / descriptor["body_schema_hash"]
            / descriptor["compiled_descriptor_hash"]
        )
        report = build_derived_usd_material_lineage_report(
            profile_path=args.profile,
            r111_report_path=args.r111_report,
            r111_profile_path=args.r111_profile,
            descriptor_bytes=descriptor_bytes,
            translation_manifest_path=bundle / "translation-manifest.json",
            humanoid_usd_path=bundle / manifest["usd_path"],
            ground_usd_path=bundle / manifest["ground_usd_path"],
            tracked_sources=tracked_source_paths(repository_root),
            validation_results=validations,
            tool_path=Path(__file__),
            repository=repository,
        )
        (staging / "derived-usd-material-lineage.json").write_bytes(
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
                "humanoid_usd_sha256": report["translation_lineage"][
                    "humanoid_usd_sha256"
                ],
                "ground_usd_sha256": report["translation_lineage"]["ground_usd_sha256"],
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
            raise RuntimeError(f"R112 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R112 report must stay outside repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _repository_state(repository: Path, profile: dict[str, Any]) -> dict[str, Any]:
    implementation = profile["source"]["implementation_commit"]
    dirty_paths = _git(repository, "status", "--short").splitlines()
    return {
        "commit": _git(repository, "rev-parse", "HEAD"),
        "dirty": bool(dirty_paths),
        "dirty_paths": dirty_paths,
        "implementation_commit": implementation,
        "implementation_commit_is_ancestor": _is_ancestor(repository, implementation),
    }


def _is_ancestor(repository: Path, commit: str) -> bool:
    return (
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", commit, "HEAD"],
            cwd=repository,
            check=False,
        ).returncode
        == 0
    )


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
