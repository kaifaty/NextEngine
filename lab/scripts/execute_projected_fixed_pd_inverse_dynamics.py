#!/usr/bin/env python3
"""Consume the single bounded TRAIN-4 R136 projected ID authority."""

from __future__ import annotations

import argparse
import hashlib
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
    parser.add_argument("--r135-report", type=Path, required=True)
    parser.add_argument("--r135-profile", type=Path, required=True)
    parser.add_argument("--r135-conformance-module", type=Path, required=True)
    parser.add_argument("--r135-tool", type=Path, required=True)
    parser.add_argument("--r134-report", type=Path, required=True)
    parser.add_argument("--r134-profile", type=Path, required=True)
    parser.add_argument("--r134-module", type=Path, required=True)
    parser.add_argument("--r134-tool", type=Path, required=True)
    parser.add_argument("--r133-report", type=Path, required=True)
    parser.add_argument("--r133-profile", type=Path, required=True)
    parser.add_argument("--r133-module", type=Path, required=True)
    parser.add_argument("--r133-tool", type=Path, required=True)
    parser.add_argument("--r126-report", type=Path, required=True)
    parser.add_argument("--r126-profile", type=Path, required=True)
    parser.add_argument("--gauge-aware-kernel", type=Path, required=True)
    parser.add_argument("--gauge-execution-module", type=Path, required=True)
    parser.add_argument("--r126-conformance-module", type=Path, required=True)
    parser.add_argument("--r126-tool", type=Path, required=True)
    parser.add_argument("--r121-report", type=Path, required=True)
    parser.add_argument("--r121-profile", type=Path, required=True)
    parser.add_argument("--r113-report", type=Path, required=True)
    parser.add_argument("--r113-profile", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
    parser.add_argument("--r120-profile", type=Path, required=True)
    parser.add_argument("--r120-cache", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--contact-conformance-module", type=Path, required=True)
    parser.add_argument("--collocation-lift-module", type=Path, required=True)
    parser.add_argument("--exit-mode-lift-module", type=Path, required=True)
    parser.add_argument("--projection-module", type=Path, required=True)
    parser.add_argument("--dynamics-kernel", type=Path, required=True)
    parser.add_argument("--composition-module", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.projected_fixed_pd_inverse_dynamics_execution import (
        execute_and_build_projected_fixed_pd_inverse_dynamics_report,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R136 execution requires a clean repository")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R136 repository identity changed during validation")
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
        raise ValueError("R136 repository changed during descriptor export")

    staging = Path(tempfile.mkdtemp(prefix="nextengine-r136-", dir=output.parent))
    try:
        report, cache_bytes = (
            execute_and_build_projected_fixed_pd_inverse_dynamics_report(
                profile_path=args.profile,
                r135_report_path=args.r135_report,
                r135_profile_path=args.r135_profile,
                r135_conformance_module_path=args.r135_conformance_module,
                r135_tool_path=args.r135_tool,
                r134_report_path=args.r134_report,
                r134_profile_path=args.r134_profile,
                r134_module_path=args.r134_module,
                r134_tool_path=args.r134_tool,
                r133_report_path=args.r133_report,
                r133_profile_path=args.r133_profile,
                r133_module_path=args.r133_module,
                r133_tool_path=args.r133_tool,
                r126_report_path=args.r126_report,
                r126_profile_path=args.r126_profile,
                gauge_aware_kernel_path=args.gauge_aware_kernel,
                gauge_execution_module_path=args.gauge_execution_module,
                r126_conformance_module_path=args.r126_conformance_module,
                r126_tool_path=args.r126_tool,
                r121_report_path=args.r121_report,
                r121_profile_path=args.r121_profile,
                r113_report_path=args.r113_report,
                r113_profile_path=args.r113_profile,
                r120_report_path=args.r120_report,
                r120_profile_path=args.r120_profile,
                r120_cache_path=args.r120_cache,
                v9_complete_clip_path=args.v9_complete_clip,
                contact_conformance_module_path=args.contact_conformance_module,
                collocation_lift_module_path=args.collocation_lift_module,
                exit_mode_lift_module_path=args.exit_mode_lift_module,
                projection_module_path=args.projection_module,
                dynamics_kernel_path=args.dynamics_kernel,
                composition_module_path=args.composition_module,
                descriptor_bytes=descriptor_bytes,
                validation_results=validations,
                tool_path=Path(__file__),
                repository=repository,
                execution_environment=THREAD_ENVIRONMENT,
            )
        )
        if cache_bytes is not None:
            cache_path = staging / "solver-private-r136-projected-fixed-pd.npz"
            cache_path.write_bytes(cache_bytes)
            if (
                hashlib.sha256(cache_bytes).hexdigest()
                != report["solver_private_cache"]["file_sha256"]
            ):
                raise ValueError("R136 solver-private cache identity differs")
        report_path = staging / "projected-fixed-pd-inverse-dynamics.json"
        report_path.write_bytes(
            (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        )
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    solver = report["solver_result"]
    print(
        json.dumps(
            {
                "status": report["status"],
                "feasibility": solver["feasibility"],
                "invalid_reason": solver["invalid_reason"],
                "gate_decision": report["gate_decision"],
                "result_transition": report["result_transition"],
                "report_sha256": report["report_sha256"],
                "collocation_rows": solver["aggregate"]["collocation_rows_recorded"],
                "feasible_collocations": solver["aggregate"]["feasible_collocations"],
                "infeasible_collocations": solver["aggregate"][
                    "infeasible_collocations"
                ],
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
            raise RuntimeError(f"R136 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R136 output must stay outside repository")
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
