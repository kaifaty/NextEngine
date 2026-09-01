#!/usr/bin/env python3
"""Assemble passed V24 T0 and X0 lane fragments into one external D0 V3 manifest."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path
from typing import Any


T0_SCHEMA = "nextengine.experimental-physical-sound-v24-t0-teacher.lane-records.v1"
T0_CLAIM = "SYNTHETIC_ANALYTIC_MODAL_TEACHER_ONLY / NO_REAL_MATERIAL_QUALITY_OR_RUNTIME_AUTHORITY"
X0_SCHEMA = "nextengine.experimental-physical-sound-v24-x0.lane-records.v1"
X0_CLAIM = (
    "DISCLOSED_BLUE_BOWL_REAL_EVIDENCE_ONLY / "
    "NO_MISSING_AXIS_MODEL_ADMISSION_OR_RUNTIME_AUTHORITY"
)
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-neural-data-plane.manifest.v3"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v24-v3-assembly.report.v1"
MAX_LANE_BYTES = 8 * 1024 * 1024
MAX_ARTIFACT_BYTES = 128 * 1024 * 1024
ROLES = ("train", "development", "calibration", "method_holdout", "admission_shadow")
LANES = ("synthetic_teacher", "exact_real_transfer", "identified_real_recording")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def external_file(root: Path, path: Path, role: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise ValueError(f"V3 assembly {role} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise ValueError(f"V3 assembly output must be a new external path: {resolved}")
    return resolved


def parse_lane(path: Path, schema: str, claim: str, lane_name: str) -> tuple[dict[str, Any], bytes]:
    data = path.read_bytes()
    if not 0 < len(data) <= MAX_LANE_BYTES:
        raise ValueError(f"V3 {lane_name} lane must be 1..={MAX_LANE_BYTES} bytes")
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"invalid V3 {lane_name} lane JSON: {error}") from error
    if (
        not isinstance(value, dict)
        or value.get("schema") != schema
        or value.get("status") != "Validated"
        or value.get("claim") != claim
        or not isinstance(value.get("lineage_report"), dict)
        or not isinstance(value.get("rows"), list)
    ):
        raise ValueError(f"V3 {lane_name} lane identity changed")
    return value, data


def resolve_ref(root: Path, lane_directory: Path, reference: dict[str, Any], role: str) -> dict[str, str]:
    if set(reference) != {"path", "sha256"}:
        raise ValueError(f"V3 {role} file reference fields changed")
    relative = reference["path"]
    expected = reference["sha256"]
    if (
        not isinstance(relative, str)
        or not relative
        or not isinstance(expected, str)
        or len(expected) != 64
        or any(character not in "0123456789abcdef" for character in expected)
    ):
        raise ValueError(f"V3 {role} file reference is invalid")
    path = Path(relative)
    path = path if path.is_absolute() else lane_directory / path
    path = external_file(root, path, role)
    if path.stat().st_size > MAX_ARTIFACT_BYTES:
        raise ValueError(f"V3 {role} exceeds {MAX_ARTIFACT_BYTES} bytes")
    actual = sha256_bytes(path.read_bytes())
    if actual != expected:
        raise ValueError(f"V3 {role} hash mismatch: expected {expected}, got {actual}")
    return {"path": str(path), "sha256": expected}


def rewrite_refs(root: Path, lane_directory: Path, value: Any, role: str) -> Any:
    if isinstance(value, dict):
        if set(value) == {"path", "sha256"}:
            return resolve_ref(root, lane_directory, value, role)
        return {
            key: rewrite_refs(root, lane_directory, child, f"{role}.{key}")
            for key, child in value.items()
        }
    if isinstance(value, list):
        return [rewrite_refs(root, lane_directory, child, f"{role}[{index}]") for index, child in enumerate(value)]
    return value


def validate_combination(lineages: list[dict[str, Any]], rows: list[dict[str, Any]]) -> None:
    if [lineage.get("id") for lineage in lineages] != sorted(
        lineage.get("id") for lineage in lineages
    ):
        raise ValueError("V3 lineage reports are not strictly sorted")
    if len({lineage.get("id") for lineage in lineages}) != len(lineages):
        raise ValueError("V3 lineage report IDs are not unique")
    if [row.get("row_id") for row in rows] != sorted(row.get("row_id") for row in rows):
        raise ValueError("V3 rows are not strictly sorted")
    if len({row.get("row_id") for row in rows}) != len(rows):
        raise ValueError("V3 row IDs are not unique")
    role_counts = {role: 0 for role in ROLES}
    lane_counts = {lane: 0 for lane in LANES}
    context_query = {role: {"context": 0, "query": 0} for role in ROLES}
    for row in rows:
        role = row.get("split_role")
        lane = row.get("evidence_lane")
        sample_role = row.get("sample_role")
        if role not in role_counts or lane not in lane_counts or sample_role not in {"context", "query"}:
            raise ValueError("V3 row role, lane or sample role is invalid")
        role_counts[role] += 1
        lane_counts[lane] += 1
        if row.get("corpus_role") == "target":
            context_query[role][sample_role] += 1
    if any(count == 0 for count in role_counts.values()) or any(count == 0 for count in lane_counts.values()):
        raise ValueError("V3 combination does not cover every role and lane")
    if any(not counts["context"] or not counts["query"] for counts in context_query.values()):
        raise ValueError("V3 combination lacks target context/query coverage")


def build_into(staging: Path, root: Path, t0_path: Path, x0_path: Path) -> dict[str, Any]:
    t0, t0_bytes = parse_lane(t0_path, T0_SCHEMA, T0_CLAIM, "T0")
    x0, x0_bytes = parse_lane(x0_path, X0_SCHEMA, X0_CLAIM, "X0")
    t0_lineage = rewrite_refs(root, t0_path.parent, t0["lineage_report"], "T0 lineage")
    x0_lineage = rewrite_refs(root, x0_path.parent, x0["lineage_report"], "X0 lineage")
    lineages = sorted((t0_lineage, x0_lineage), key=lambda value: value["id"])
    rows = [
        *rewrite_refs(root, t0_path.parent, t0["rows"], "T0 rows"),
        *rewrite_refs(root, x0_path.parent, x0["rows"], "X0 rows"),
    ]
    rows.sort(key=lambda value: value["row_id"])
    validate_combination(lineages, rows)
    manifest = {
        "schema": MANIFEST_SCHEMA,
        "projection_id": "v24-blue-bowl-real-pilot",
        "revision": "v1",
        "task_scope": "exact_object_few_shot_impact_listener_field",
        "split_policy": {
            "object_groups_disjoint_across_roles": True,
            "source_groups_disjoint_across_roles": True,
            "recording_parents_disjoint_across_roles": True,
            "condition_groups_disjoint_across_roles": True,
            "mutation_parents_disjoint_across_roles": True,
            "identical_audio_disjoint_across_roles": True,
            "method_holdout_sealed_before_candidate_freeze": True,
            "admission_shadow_sealed_until_validator_release": True,
        },
        "lineage_reports": lineages,
        "rows": rows,
    }
    manifest_bytes = canonical_json(manifest)
    manifest_path = staging / "combined-manifest.json"
    manifest_path.write_bytes(manifest_bytes)
    role_counts = {role: sum(row["split_role"] == role for row in rows) for role in ROLES}
    lane_counts = {lane: sum(row["evidence_lane"] == lane for row in rows) for lane in LANES}
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "T0_X0_V3_MANIFEST_ASSEMBLED",
        "t0_lane_sha256": sha256_bytes(t0_bytes),
        "x0_lane_sha256": sha256_bytes(x0_bytes),
        "combined_manifest_sha256": sha256_bytes(manifest_bytes),
        "lineage_report_count": len(lineages),
        "row_count": len(rows),
        "role_counts": role_counts,
        "lane_counts": lane_counts,
        "d0_execution_authorized": True,
        "model_training_authorized": False,
        "real_material_admission_authorized": False,
        "runtime_authorized": False,
    }
    (staging / "report.json").write_bytes(canonical_json(report))
    return report


def run(t0_path: Path, x0_path: Path, output_path: Path) -> dict[str, Any]:
    root = repository_root().resolve(strict=True)
    t0_path = external_file(root, t0_path, "T0 lane")
    x0_path = external_file(root, x0_path, "X0 lane")
    output_path = external_output(root, output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v24-v3-", dir=output_path.parent))
    try:
        report = build_into(staging, root, t0_path, x0_path)
        os.replace(staging, output_path)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--t0-lane", required=True, type=Path)
    parser.add_argument("--x0-lane", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.t0_lane, arguments.x0_lane, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable CLI diagnostic boundary
        print(f"physical-sound-v24-v3-assemble: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
