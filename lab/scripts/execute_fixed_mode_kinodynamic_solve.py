#!/usr/bin/env python3
"""Consume the sole bounded TRAIN-4 R141 kinodynamic execution authority."""

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
    parser.add_argument("--authority-document", type=Path, required=True)
    parser.add_argument("--r140-report", type=Path, required=True)
    parser.add_argument("--r140-profile", type=Path, required=True)
    parser.add_argument("--r140-module", type=Path, required=True)
    parser.add_argument("--r140-tool", type=Path, required=True)
    parser.add_argument("--r139-report", type=Path, required=True)
    parser.add_argument("--r139-profile", type=Path, required=True)
    parser.add_argument("--r139-module", type=Path, required=True)
    parser.add_argument("--r139-tool", type=Path, required=True)
    parser.add_argument("--r120-report", type=Path, required=True)
    parser.add_argument("--r120-profile", type=Path, required=True)
    parser.add_argument("--r120-cache", type=Path, required=True)
    parser.add_argument("--r133-report", type=Path, required=True)
    parser.add_argument("--r133-profile", type=Path, required=True)
    parser.add_argument("--r133-module", type=Path, required=True)
    parser.add_argument("--r133-tool", type=Path, required=True)
    parser.add_argument("--r138-report", type=Path, required=True)
    parser.add_argument("--r138-profile", type=Path, required=True)
    parser.add_argument("--r138-module", type=Path, required=True)
    parser.add_argument("--r138-tool", type=Path, required=True)
    parser.add_argument("--v9-complete-clip", type=Path, required=True)
    parser.add_argument("--dynamics-kernel", type=Path, required=True)
    parser.add_argument("--projection-module", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    for name, value in THREAD_ENVIRONMENT.items():
        os.environ[name] = value
    from next_lab.fixed_mode_kinodynamic_execution import (
        _validate_profile,
        execute_and_build_fixed_mode_kinodynamic_report,
    )

    args = parse_args()
    output = _external_directory(args.output)
    repository_root = Path(__file__).resolve().parents[2]
    profile = json.loads(args.profile.read_bytes())
    _validate_profile(profile)
    repository = _repository_state(repository_root)
    if repository["dirty"]:
        raise ValueError("R141 execution requires a clean repository")
    authority_commit = profile["source"]["authority_repository_commit"]
    ancestry = subprocess.run(
        ["git", "merge-base", "--is-ancestor", authority_commit, repository["commit"]],
        cwd=repository_root,
        check=False,
        capture_output=True,
    )
    if ancestry.returncode != 0:
        raise ValueError("R141 implementation is outside the authority lineage")
    validations = _run_validations(repository_root, profile["validation_commands"])
    if _repository_state(repository_root) != repository:
        raise ValueError("R141 repository identity changed during validation")
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
        raise ValueError("R141 repository changed during descriptor export")

    staging = Path(tempfile.mkdtemp(prefix="nextengine-r141-", dir=output.parent))
    try:
        report, cache_bytes = execute_and_build_fixed_mode_kinodynamic_report(
            profile_path=args.profile,
            authority_document_path=args.authority_document,
            r140_report_path=args.r140_report,
            r140_profile_path=args.r140_profile,
            r140_module_path=args.r140_module,
            r140_tool_path=args.r140_tool,
            r139_report_path=args.r139_report,
            r139_profile_path=args.r139_profile,
            r139_module_path=args.r139_module,
            r139_tool_path=args.r139_tool,
            r120_report_path=args.r120_report,
            r120_profile_path=args.r120_profile,
            r120_cache_path=args.r120_cache,
            r133_report_path=args.r133_report,
            r133_profile_path=args.r133_profile,
            r133_module_path=args.r133_module,
            r133_tool_path=args.r133_tool,
            r138_report_path=args.r138_report,
            r138_profile_path=args.r138_profile,
            r138_module_path=args.r138_module,
            r138_tool_path=args.r138_tool,
            v9_complete_clip_path=args.v9_complete_clip,
            dynamics_kernel_path=args.dynamics_kernel,
            projection_module_path=args.projection_module,
            descriptor_bytes=descriptor_bytes,
            validation_results=validations,
            tool_path=Path(__file__),
            repository=repository,
            execution_environment=THREAD_ENVIRONMENT,
        )
        if cache_bytes is not None:
            cache_path = staging / "solver-private-r141-fixed-mode-kinodynamic.npz"
            cache_path.write_bytes(cache_bytes)
            expected = report["solver_private_cache"]["file_sha256"]
            if hashlib.sha256(cache_bytes).hexdigest() != expected:
                raise ValueError("R141 solver-private cache identity differs")
        report_path = staging / "fixed-mode-kinodynamic-execution.json"
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
                "feasibility": report["feasibility"],
                "termination": report["termination"],
                "invalid_reason": report["invalid_reason"],
                "gate_decision": report["gate_decision"],
                "result_transition": report["result_transition"],
                "report_sha256": report["report_sha256"],
                "qp_solves": report["real_qp_solves"],
                "exact_trial_audits": report["real_exact_trial_audits"],
                "optimizer_steps": report["optimizer_steps"],
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
            raise RuntimeError(f"R141 validation {row['id']} failed:\n{detail}")
        results.append({"id": row["id"], "status": "PASS"})
    return results


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("R141 output must stay outside repository")
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
