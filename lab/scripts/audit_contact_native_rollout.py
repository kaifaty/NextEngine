#!/usr/bin/env python3
"""Build the report-only TRAIN-4 R102 native-rollout decision audit."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.contact_native_rollout_audit import build_native_rollout_audit


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--v7-report", type=Path, required=True)
    parser.add_argument("--v9-report", type=Path, required=True)
    parser.add_argument("--r100-report", type=Path, required=True)
    parser.add_argument("--r101-report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    report = build_native_rollout_audit(
        profile_path=args.profile,
        source_report_paths={
            "v7": args.v7_report,
            "v9": args.v9_report,
            "r100": args.r100_report,
            "r101": args.r101_report,
        },
        tool_path=Path(__file__),
        repository=_repository_state(),
    )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-native-rollout-audit-", dir=output.parent
        )
    )
    try:
        (staging / "native-rollout-audit.json").write_bytes(
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
                "candidate_search": report["bounded_acceptance"][
                    "candidate_search"
                ],
                "output": str(output),
            },
            sort_keys=True,
        )
    )


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("native-rollout audit must stay outside the repository")
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
