#!/usr/bin/env python3
"""Run the one exact optimizer-free TRAIN-4 R107 projected audit."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.contact_projected_direction_audit import (
    build_projected_direction_exact_audit,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--r103-report", type=Path, required=True)
    parser.add_argument("--r105-report", type=Path, required=True)
    parser.add_argument("--r105-profile", type=Path, required=True)
    parser.add_argument("--r106-report", type=Path, required=True)
    parser.add_argument("--r106-profile", type=Path, required=True)
    parser.add_argument("--v7-manifest", type=Path, required=True)
    parser.add_argument("--v9-manifest", type=Path, required=True)
    parser.add_argument("--v9-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    repository = _repository_state()
    if repository["dirty"]:
        raise ValueError(
            "R107 projected-direction exact audit requires a clean repository"
        )
    report = build_projected_direction_exact_audit(
        profile_path=args.profile,
        source_audit_path=args.source_audit,
        r103_report_path=args.r103_report,
        r105_report_path=args.r105_report,
        r105_profile_path=args.r105_profile,
        r106_report_path=args.r106_report,
        r106_profile_path=args.r106_profile,
        v7_manifest_path=args.v7_manifest,
        v9_manifest_path=args.v9_manifest,
        v9_profile_path=args.v9_profile,
        descriptor_path=args.descriptor,
        tool_path=Path(__file__),
        repository=repository,
    )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-projected-direction-exact-audit-",
            dir=output.parent,
        )
    )
    try:
        (staging / "projected-direction-exact-audit.json").write_bytes(
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
                "exact_status": report["exact_offline_result"]["status"],
                "failure_reasons": report["exact_offline_result"][
                    "failure_reasons"
                ],
                "gate_decision": report["gate_decision"],
                "report_sha256": report["report_sha256"],
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
        raise ValueError("projected-direction audit must stay outside repository")
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
