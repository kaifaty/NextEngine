#!/usr/bin/env python3
"""Run report-only R129 tangent-velocity projection conformance."""

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
    parser.add_argument("--r128-report", type=Path, required=True)
    parser.add_argument("--r128-profile", type=Path, required=True)
    parser.add_argument("--r128-module", type=Path, required=True)
    parser.add_argument("--r128-tool", type=Path, required=True)
    parser.add_argument("--r122-report", type=Path, required=True)
    parser.add_argument("--r122-profile", type=Path, required=True)
    parser.add_argument("--dynamics-kernel", type=Path, required=True)
    parser.add_argument("--dynamics-conformance-module", type=Path, required=True)
    parser.add_argument("--r122-tool", type=Path, required=True)
    parser.add_argument("--r113-report", type=Path, required=True)
    parser.add_argument("--r113-profile", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
    parser.add_argument("--r120-profile", type=Path, required=True)
    parser.add_argument("--r120-cache", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--collocation-lift-module", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.tangent_velocity_projection_conformance import (
        build_tangent_velocity_projection_conformance,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R129 conformance requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R129 repository identity changed during validation")
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
        raise ValueError("R129 repository changed during descriptor export")
    report = build_tangent_velocity_projection_conformance(
        profile_path=args.profile,
        r128_report_path=args.r128_report,
        r128_profile_path=args.r128_profile,
        r128_module_path=args.r128_module,
        r128_tool_path=args.r128_tool,
        r122_report_path=args.r122_report,
        r122_profile_path=args.r122_profile,
        dynamics_kernel_path=args.dynamics_kernel,
        dynamics_conformance_module_path=args.dynamics_conformance_module,
        r122_tool_path=args.r122_tool,
        r113_report_path=args.r113_report,
        r113_profile_path=args.r113_profile,
        r120_report_path=args.r120_report,
        r120_profile_path=args.r120_profile,
        r120_cache_path=args.r120_cache,
        v9_complete_clip_path=args.v9_complete_clip,
        collocation_lift_module_path=args.collocation_lift_module,
        descriptor_bytes=descriptor_bytes,
        validation_results=validations,
        tool_path=Path(__file__),
        repository=repository,
        execution_environment=THREAD_ENVIRONMENT,
    )
    staging = Path(tempfile.mkdtemp(prefix="nextengine-r129-", dir=output.parent))
    try:
        report_path = staging / "tangent-velocity-projection-conformance.json"
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
                "report_sha256": report["report_sha256"],
                "real_state_projections": report["real_state_projections"],
                "full_schedule_projections": report["full_schedule_projections"],
                "inverse_dynamics_evaluations": report["inverse_dynamics_evaluations"],
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
            raise RuntimeError(f"R129 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R129 output must stay outside repository")
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
