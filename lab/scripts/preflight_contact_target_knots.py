#!/usr/bin/env python3
"""Run the exact optimizer-free TRAIN-4 R104 target-knot preflight."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from next_lab.contact_target_knot_preflight import build_target_knot_preflight


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--r103-report", type=Path, required=True)
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
        raise ValueError("R104 target-knot preflight requires a clean repository")
    report = build_target_knot_preflight(
        profile_path=args.profile,
        source_audit_path=args.source_audit,
        r103_report_path=args.r103_report,
        v7_manifest_path=args.v7_manifest,
        v9_manifest_path=args.v9_manifest,
        v9_profile_path=args.v9_profile,
        descriptor_path=args.descriptor,
        tool_path=Path(__file__),
        repository=repository,
        progress=lambda row: print(json.dumps(row, sort_keys=True), flush=True),
    )
    staging = Path(
        tempfile.mkdtemp(
            prefix="nextengine-target-knot-preflight-", dir=output.parent
        )
    )
    try:
        (staging / "target-knot-preflight.json").write_bytes(
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
                "candidate_count": report["summary"]["candidate_count"],
                "nonzero_pass_count": report["summary"]["nonzero_pass_count"],
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
        raise ValueError("target-knot preflight must stay outside the repository")
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
