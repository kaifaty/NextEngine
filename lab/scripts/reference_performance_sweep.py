#!/usr/bin/env python3
"""Aggregate hash-consistent report-only reference-training sweep runs."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from next_lab.reference_performance import build_sweep_report


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-dir", action="append", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = args.output.resolve()
    if output == REPOSITORY_ROOT or REPOSITORY_ROOT in output.parents:
        raise ValueError("performance sweep output must remain outside the repository")
    runs = [_load_run(directory.resolve()) for directory in args.run_dir]
    report = build_sweep_report(runs)
    report["report_hash"] = hashlib.sha256(_canonical_json(report)).hexdigest()
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(output.suffix + ".tmp")
    temporary.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    temporary.replace(output)
    print(json.dumps(report, indent=2, sort_keys=True), flush=True)


def _load_run(run_dir: Path) -> dict[str, Any]:
    manifest_path = run_dir / "run-manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    throughput_record = manifest.get("throughput_report")
    if isinstance(throughput_record, dict):
        throughput_path = run_dir / str(throughput_record["file"])
        payload = throughput_path.read_bytes()
        if hashlib.sha256(payload).hexdigest() != throughput_record["sha256"]:
            raise ValueError(f"throughput report hash mismatch: {run_dir}")
        manifest["throughput"] = json.loads(payload)
    return manifest


def _canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")


if __name__ == "__main__":
    main()
