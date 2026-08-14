#!/usr/bin/env python3
"""Build the report-only TRAIN-4 V7/V9 native-dynamics differential audit."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.contact_dynamics_audit import build_native_dynamics_audit


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--v7-manifest", type=Path, required=True)
    parser.add_argument("--v9-manifest", type=Path, required=True)
    parser.add_argument("--r94-report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = _external_directory(args.output)
    report = build_native_dynamics_audit(
        profile_path=args.profile,
        source_audit_path=args.source_audit,
        descriptor_path=args.descriptor,
        v7_manifest_path=args.v7_manifest,
        v9_manifest_path=args.v9_manifest,
        r94_report_path=args.r94_report,
        tool_path=Path(__file__),
        repository=_repository_state(),
    )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-native-dynamics-audit-", dir=output.parent
        )
    )
    try:
        report_path = staging / "native-dynamics-audit.json"
        report_path.write_bytes(
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
                "report_sha256": report["report_sha256"],
                "selected_counterfactual_case_ordinals": report["findings"][
                    "selected_counterfactual_case_ordinals"
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
        raise ValueError("native-dynamics audit must stay outside the repository")
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
