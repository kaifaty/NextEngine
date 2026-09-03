#!/usr/bin/env python3
"""Compile the V46 D1 path-neutral multi-fidelity corpus index."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-profile.v1"
INDEX_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-corpus-index.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-report.v1"

R0_SCHEMA = "nextengine.experimental-physical-sound-v45-r0-source-claim-ledger.v1"
T0_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-recipe-v3.v1"
C0_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-prior.v1"
D0_SCHEMA = "nextengine.experimental-physical-sound-v46-d0-source-preflight.v1"
C0R_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-corpus.v1"
C0R_PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-v43-c0r-role-projection.v1"
)
C0R_REPORT_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-report.v1"
IET_MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c1-corpus-increment.v1"
)
IET_PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c1-generator-train-projection.v1"
)
IET_REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c1-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v46-d1-corpus-compiler.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v46_d1_corpus_compiler_v1.py"
PROTOCOL_PATH = (
    "docs/development/physical-sound-v46-d1-corpus-compiler-protocol-2026-09-03.md"
)
D0_PROFILE_PATH = "lab/profiles/physical-sound-v46-d0-source-preflight.v1.json"
D0_RESULT_PATH = (
    "docs/development/physical-sound-v46-d0-synthetic-source-preflight-result-"
    "2026-09-03.md"
)

STUDY_ID = "physical-sound-v46-d1-corpus-compiler"
CLAIM = (
    "PATH_NEUTRAL_CLAIM_MASKED_CORPUS_INDEX / B0_CONTROL_INPUT_ONLY / NO_NEW_"
    "PARENT_PROJECT_MODEL_VALIDATOR_PROTECTED_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
DECISION = "CorpusIndexCompiled"
HASH_LENGTH = 64
MAX_PROFILE_BYTES = 2 * 1024 * 1024
MAX_INPUT_BYTES = 2 * 1024 * 1024
MAX_RECORDS = 2_000
MAX_CONTROL_ROWS = 1_000

ROLES = ("generator_development", "generator_train", "validator_calibration")
ZERO_CONTENT_COUNTERS = (
    "content_objects_copied",
    "content_objects_opened",
    "model_values_read",
    "network_requests",
    "new_external_modal_values_decoded",
    "pcm_sample_values_decoded",
    "protected_values_read",
    "real_target_values_decoded",
    "waveform_bytes_read",
)
AUTHORITY = {
    "admission_authority": False,
    "authorizes_only": "v46_b0_control_tournament",
    "corpus_index_authority": True,
    "model_training_authority": False,
    "new_parent_or_project_credit": 0,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_qualification_authority": False,
}


class CorpusCompilerError(RuntimeError):
    """D1 cannot publish a trustworthy corpus index."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--inputs", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(
                value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False
            )
            + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise CorpusCompilerError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise CorpusCompilerError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise CorpusCompilerError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise CorpusCompilerError(f"{context} must be a non-empty string")
    return value


def require_bool(value: Any, context: str) -> bool:
    if not isinstance(value, bool):
        raise CorpusCompilerError(f"{context} must be a boolean")
    return value


def require_nonnegative_int(value: Any, context: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise CorpusCompilerError(f"{context} must be a non-negative integer")
    return value


def require_hash(value: Any, context: str) -> str:
    text = require_string(value, context)
    if len(text) != HASH_LENGTH or any(
        character not in "0123456789abcdef" for character in text
    ):
        raise CorpusCompilerError(f"{context} must be a lowercase SHA-256")
    return text


def read_regular(path: Path, context: str, maximum: int = MAX_INPUT_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise CorpusCompilerError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise CorpusCompilerError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_json(path: Path, context: str) -> tuple[bytes, Any]:
    data = read_regular(path, context)
    try:
        return data, json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise CorpusCompilerError(f"{context} is not valid UTF-8 JSON") from error


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data, value = read_json(path, context)
    document = require_dict(value, context)
    if data != canonical_json(document):
        raise CorpusCompilerError(f"{context} is not canonical JSON")
    return data, document


def validate_dependency(value: Any, context: str) -> dict[str, Any]:
    binding = require_dict(value, context)
    if set(binding) != {"bytes", "path", "sha256"}:
        raise CorpusCompilerError(f"{context} fields changed")
    relative = Path(require_string(binding["path"], f"{context}.path"))
    count = require_nonnegative_int(binding["bytes"], f"{context}.bytes")
    digest = require_hash(binding["sha256"], f"{context}.sha256")
    if relative.is_absolute() or ".." in relative.parts or count == 0:
        raise CorpusCompilerError(f"{context} path or byte count is invalid")
    data = read_regular(repository_root() / relative, context, MAX_PROFILE_BYTES)
    if len(data) != count or sha256_bytes(data) != digest:
        raise CorpusCompilerError(f"{context} binding mismatch")
    return binding


def validate_input_binding(value: Any, context: str) -> dict[str, Any]:
    binding = require_dict(value, context)
    if set(binding) != {"bytes", "filename", "schema", "sha256", "source_id"}:
        raise CorpusCompilerError(f"{context} fields changed")
    filename = Path(require_string(binding["filename"], f"{context}.filename"))
    if filename.is_absolute() or len(filename.parts) != 1 or ".." in filename.parts:
        raise CorpusCompilerError(f"{context} filename is unsafe")
    if require_nonnegative_int(binding["bytes"], f"{context}.bytes") == 0:
        raise CorpusCompilerError(f"{context} byte count must be positive")
    require_hash(binding["sha256"], f"{context}.sha256")
    require_string(binding["schema"], f"{context}.schema")
    require_string(binding["source_id"], f"{context}.source_id")
    return binding


def validate_profile(profile: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, int]]:
    if set(profile) != {
        "access_policy",
        "authority",
        "claim",
        "dependency_bindings",
        "expected",
        "external_inputs",
        "schema",
        "study_id",
    }:
        raise CorpusCompilerError("profile fields changed")
    if (
        profile["schema"] != PROFILE_SCHEMA
        or profile["study_id"] != STUDY_ID
        or profile["claim"] != CLAIM
        or profile["authority"] != AUTHORITY
    ):
        raise CorpusCompilerError("profile identity or authority changed")
    if profile["access_policy"] != {
        "content_object_access_allowed": False,
        "external_metadata_only": True,
        "network_allowed": False,
        "outputs_external": True,
    }:
        raise CorpusCompilerError("access policy changed")
    dependencies = [
        validate_dependency(item, f"dependency {index}")
        for index, item in enumerate(
            require_list(profile["dependency_bindings"], "dependency bindings")
        )
    ]
    paths = [item["path"] for item in dependencies]
    if paths != sorted(set(paths)) or not {
        OWNER_PATH,
        PROTOCOL_PATH,
        D0_PROFILE_PATH,
        D0_RESULT_PATH,
    }.issubset(paths):
        raise CorpusCompilerError("dependencies must be sorted, unique and complete")
    inputs = [
        validate_input_binding(item, f"external input {index}")
        for index, item in enumerate(
            require_list(profile["external_inputs"], "external inputs")
        )
    ]
    filenames = [item["filename"] for item in inputs]
    if filenames != sorted(set(filenames)) or len(inputs) != 12:
        raise CorpusCompilerError("external input inventory changed")
    expected = require_dict(profile["expected"], "expected")
    required_expected = {
        "clatter_control_groups": 36,
        "clatter_control_rows": 84,
        "content_references": 308,
        "external_modal_teacher_rows": 0,
        "planning_parent_deficit": 34,
        "planning_parent_floor": 105,
        "planning_supported_parents": 71,
        "real_physical_parents": 80,
        "real_projects": 10,
        "real_records": 154,
        "structural_transfer_rows": 0,
        "validator_projects": 1,
    }
    if set(expected) != set(required_expected) | {"role_counts"}:
        raise CorpusCompilerError("expected fields changed")
    for key, value in required_expected.items():
        if require_nonnegative_int(expected[key], f"expected.{key}") != value:
            raise CorpusCompilerError(f"expected.{key} changed")
    role_counts = require_dict(expected["role_counts"], "expected.role_counts")
    if role_counts != {
        "generator_development": 70,
        "generator_train": 79,
        "validator_calibration": 5,
    }:
        raise CorpusCompilerError("expected role counts changed")
    return inputs, expected


def validate_input_directory(
    directory: Path, bindings: list[dict[str, Any]]
) -> tuple[dict[str, dict[str, Any]], list[dict[str, Any]], int]:
    root = repository_root().resolve()
    resolved = directory.resolve()
    if directory.is_symlink() or not directory.is_dir():
        raise CorpusCompilerError("inputs must be a regular external directory")
    if resolved == root or root in resolved.parents:
        raise CorpusCompilerError("inputs must stay outside the repository")
    expected_names = [item["filename"] for item in bindings]
    actual = sorted(item.name for item in directory.iterdir())
    if actual != expected_names:
        raise CorpusCompilerError("external input filenames changed")
    documents: dict[str, dict[str, Any]] = {}
    summaries = []
    total_bytes = 0
    for binding in bindings:
        filename = binding["filename"]
        data, value = read_json(directory / filename, f"external input {filename}")
        document = require_dict(value, f"external input {filename}")
        if len(data) != binding["bytes"] or sha256_bytes(data) != binding["sha256"]:
            raise CorpusCompilerError(f"external input {filename} binding mismatch")
        if document.get("schema") != binding["schema"]:
            raise CorpusCompilerError(f"external input {filename} schema changed")
        documents[filename] = document
        total_bytes += len(data)
        summaries.append(
            {
                "bytes": len(data),
                "filename": filename,
                "schema": binding["schema"],
                "sha256": binding["sha256"],
                "source_id": binding["source_id"],
            }
        )
    return documents, summaries, total_bytes


def validate_t0_contract(contract: dict[str, Any]) -> tuple[int, dict[str, list[str]]]:
    if contract.get("schema") != T0_SCHEMA:
        raise CorpusCompilerError("T0 schema changed")
    layout = require_list(contract.get("layout"), "T0 layout")
    cursor = 0
    for index, value in enumerate(layout):
        item = require_dict(value, f"T0 layout {index}")
        offset = require_nonnegative_int(item.get("offset"), f"T0 layout {index} offset")
        count = require_nonnegative_int(item.get("count"), f"T0 layout {index} count")
        if offset != cursor or count == 0:
            raise CorpusCompilerError("T0 layout is not contiguous")
        cursor += count
    if cursor != 189:
        raise CorpusCompilerError("T0 vector dimension changed")
    lane_fields = require_dict(contract.get("lane_fields"), "T0 lane fields")
    expected_lanes = {
        "empirical_prior",
        "modal_teacher",
        "protected_admission",
        "real_acoustic",
        "structural_transfer",
        "validator_calibration",
    }
    if set(lane_fields) != expected_lanes:
        raise CorpusCompilerError("T0 lanes changed")
    normalized = {}
    for lane, fields in lane_fields.items():
        strings = [require_string(item, f"T0 {lane} field") for item in require_list(fields, f"T0 {lane} fields")]
        if strings != list(dict.fromkeys(strings)):
            raise CorpusCompilerError(f"T0 {lane} fields are duplicated")
        normalized[lane] = strings
    return cursor, normalized


def validate_r0_ledger(ledger: dict[str, Any]) -> None:
    if ledger.get("schema") != R0_SCHEMA:
        raise CorpusCompilerError("R0 schema changed")
    sources = require_list(ledger.get("sources"), "R0 sources")
    if len(sources) != 10:
        raise CorpusCompilerError("R0 source count changed")
    source_ids = [require_string(require_dict(item, "R0 source").get("source_id"), "R0 source ID") for item in sources]
    if source_ids != sorted(set(source_ids)) or not {"nisr-v5", "vibraverse"}.issubset(source_ids):
        raise CorpusCompilerError("R0 source identities changed")


def validate_d0_sources(document: dict[str, Any]) -> None:
    if document.get("schema") != D0_SCHEMA:
        raise CorpusCompilerError("D0 schema changed")
    sources = require_list(document.get("sources"), "D0 sources")
    decisions = {
        require_string(item.get("source_id"), "D0 source ID"): (
            item.get("decision"),
            item.get("reason"),
            require_dict(item.get("sample"), "D0 sample").get("accessed"),
        )
        for item in (require_dict(value, "D0 source") for value in sources)
    }
    if decisions != {
        "nisr-v5": (
            "SyntheticTeacherUntrusted",
            "DeclaredGenerationDocumentUnavailable",
            False,
        ),
        "vibraverse": (
            "SyntheticTeacherUntrusted",
            "GenerationLineageIncomplete",
            False,
        ),
    }:
        raise CorpusCompilerError("D0 source decisions changed")


def numeric_leaf_count(value: Any) -> int:
    if isinstance(value, bool):
        return 0
    if isinstance(value, (int, float)):
        return 1
    if isinstance(value, list):
        return sum(numeric_leaf_count(item) for item in value)
    if isinstance(value, dict):
        return sum(numeric_leaf_count(item) for item in value.values())
    return 0


def compile_clatter_groups(document: dict[str, Any]) -> tuple[list[dict[str, Any]], int, int]:
    if document.get("schema") != C0_SCHEMA or document.get("source_lane") != "empirical_prior":
        raise CorpusCompilerError("C0 prior identity changed")
    observed_fields = require_list(document.get("observed_fields"), "C0 observed fields")
    rows = require_list(document.get("rows"), "C0 rows")
    if len(rows) > MAX_CONTROL_ROWS:
        raise CorpusCompilerError("C0 control row bound exceeded")
    groups: dict[str, list[dict[str, str]]] = defaultdict(list)
    scalar_count = 0
    filenames = []
    for index, value in enumerate(rows):
        row = require_dict(value, f"C0 row {index}")
        filename = require_string(row.get("filename"), f"C0 row {index} filename")
        recipe = require_dict(row.get("recipe"), f"C0 row {index} recipe")
        target = require_dict(row.get("target"), f"C0 row {index} target")
        if recipe.get("source_lane") != "empirical_prior" or target.get("source_lane") != "empirical_prior":
            raise CorpusCompilerError("C0 row lane changed")
        if target.get("observed_fields") != observed_fields:
            raise CorpusCompilerError("C0 row observed fields changed")
        heads = require_dict(recipe.get("heads"), f"C0 row {index} heads")
        modal = require_dict(heads.get("modal"), f"C0 row {index} modal head")
        modal_hash = sha256_bytes(canonical_json(modal))
        recipe_hash = require_hash(row.get("recipe_sha256"), f"C0 row {index} recipe hash")
        target_hash = require_hash(row.get("target_sha256"), f"C0 row {index} target hash")
        if recipe_hash != sha256_bytes(canonical_json(recipe)) or target_hash != sha256_bytes(canonical_json(target)):
            raise CorpusCompilerError("C0 row inline hash mismatch")
        groups[modal_hash].append(
            {
                "filename": filename,
                "recipe_sha256": recipe_hash,
                "target_sha256": target_hash,
            }
        )
        scalar_count += numeric_leaf_count(modal)
        filenames.append(filename)
    if filenames != sorted(set(filenames)):
        raise CorpusCompilerError("C0 filenames must be sorted and unique")
    compiled = [
        {
            "group_id": f"clatter-modal-{modal_hash}",
            "members": sorted(members, key=lambda item: item["filename"]),
            "modal_head_sha256": modal_hash,
        }
        for modal_hash, members in sorted(groups.items())
    ]
    return compiled, len(rows), scalar_count


def validate_content_reference(value: Any, context: str) -> dict[str, Any]:
    reference = require_dict(value, context)
    path_key = "path" if "path" in reference else "object_path"
    if not {"bytes", "sha256", path_key}.issubset(reference):
        raise CorpusCompilerError(f"{context} fields are incomplete")
    count = require_nonnegative_int(reference["bytes"], f"{context}.bytes")
    digest = require_hash(reference["sha256"], f"{context}.sha256")
    path = Path(require_string(reference[path_key], f"{context}.{path_key}"))
    if count == 0 or path.is_absolute() or ".." in path.parts:
        raise CorpusCompilerError(f"{context} path or byte count is invalid")
    if digest not in path.name:
        raise CorpusCompilerError(f"{context} is not content addressed")
    return {"bytes": count, "path": path.as_posix(), "sha256": digest}


def projection_records(
    document: dict[str, Any], schema: str, role: str, context: str
) -> dict[str, dict[str, Any]]:
    if document.get("schema") != schema or document.get("role") != role:
        raise CorpusCompilerError(f"{context} identity changed")
    values = document.get("rows") if "rows" in document else document.get("records")
    records = require_list(values, f"{context} records")
    indexed = {}
    for value in records:
        item = require_dict(value, f"{context} record")
        record_id = require_string(item.get("record_id"), f"{context} record ID")
        if record_id in indexed:
            raise CorpusCompilerError(f"{context} duplicates record ID")
        indexed[record_id] = item
    declared_count = document.get("record_count")
    if declared_count is not None and declared_count != len(records):
        raise CorpusCompilerError(f"{context} record count changed")
    return indexed


def compile_c0r_rows(
    manifest: dict[str, Any], projections: dict[str, dict[str, Any]], report: dict[str, Any]
) -> list[dict[str, Any]]:
    if manifest.get("schema") != C0R_MANIFEST_SCHEMA:
        raise CorpusCompilerError("C0R manifest schema changed")
    if report.get("schema") != C0R_REPORT_SCHEMA or report.get("decision") != "C0R_CORRECTED_CORPUS_REPEATABLE_B0R_R0R_C1_AUTHORIZED":
        raise CorpusCompilerError("C0R terminal decision changed")
    projected = {
        role: projection_records(
            projections[role], C0R_PROJECTION_SCHEMA, role, f"C0R {role}"
        )
        for role in ROLES
    }
    records = require_list(manifest.get("records"), "C0R records")
    if len(records) > MAX_RECORDS:
        raise CorpusCompilerError("C0R record bound exceeded")
    compiled = []
    seen = set()
    for value in records:
        item = require_dict(value, "C0R record")
        record_id = require_string(item.get("record_id"), "C0R record ID")
        role = require_string(item.get("role"), "C0R role")
        if role not in ROLES or record_id in seen:
            raise CorpusCompilerError("C0R role or record identity is invalid")
        seen.add(record_id)
        projection = projected[role].get(record_id)
        if projection is None:
            raise CorpusCompilerError("C0R record is missing from its role projection")
        parent = require_string(item.get("physical_parent_id"), "C0R parent")
        if projection.get("physical_parent_id") != parent:
            raise CorpusCompilerError("C0R parent differs from role projection")
        provenance = require_dict(item.get("provenance"), "C0R provenance")
        compiled.append(
            {
                "claim_lane": "real_acoustic",
                "material_label": require_string(item.get("material_label"), "C0R material"),
                "observation_mask": require_dict(item.get("axis_mask"), "C0R axis mask"),
                "pcm": validate_content_reference(item.get("canonical_pcm"), "C0R PCM"),
                "physical_parent_id": parent,
                "project_id": require_string(provenance.get("project_id"), "C0R project"),
                "record_id": record_id,
                "role": role,
                "source_artifact": "c0r",
                "source_component_id": require_string(item.get("family_component_id"), "C0R component"),
                "target": validate_content_reference(item.get("acoustic_target"), "C0R target"),
                "target_contract": "c0r_acoustic_pseudo_target",
            }
        )
    projected_ids = set().union(*(set(records) for records in projected.values()))
    if projected_ids != seen or sum(len(records) for records in projected.values()) != len(seen):
        raise CorpusCompilerError("C0R role projections are not an exact partition")
    return compiled


def compile_iet_rows(
    manifest: dict[str, Any], projection: dict[str, Any], report: dict[str, Any]
) -> tuple[list[dict[str, Any]], dict[str, int]]:
    if manifest.get("schema") != IET_MANIFEST_SCHEMA:
        raise CorpusCompilerError("IET manifest schema changed")
    if report.get("schema") != IET_REPORT_SCHEMA or report.get("decision") != "CorpusSuccessorMaterialized":
        raise CorpusCompilerError("IET terminal decision changed")
    projected = projection_records(
        projection, IET_PROJECTION_SCHEMA, "generator_train", "IET projection"
    )
    records = require_list(manifest.get("records"), "IET records")
    compiled = []
    seen = set()
    for value in records:
        item = require_dict(value, "IET record")
        record_id = require_string(item.get("record_id"), "IET record ID")
        if record_id in seen or item.get("role") != "generator_train":
            raise CorpusCompilerError("IET record identity or role is invalid")
        seen.add(record_id)
        target = validate_content_reference(item.get("target"), "IET target")
        projected_item = projected.get(record_id)
        if projected_item is None or projected_item.get("target_sha256") != target["sha256"]:
            raise CorpusCompilerError("IET projection target differs")
        lineage = require_dict(item.get("lineage"), "IET lineage")
        descriptor = require_dict(item.get("descriptor"), "IET descriptor")
        mask = require_list(descriptor.get("observed_mask"), "IET descriptor mask")
        if not mask or not all(isinstance(value, bool) for value in mask):
            raise CorpusCompilerError("IET descriptor mask is invalid")
        compiled.append(
            {
                "claim_lane": "real_acoustic",
                "material_label": require_string(item.get("material_label"), "IET material"),
                "observation_mask": {"runtime_descriptor_observed": mask},
                "pcm": validate_content_reference(item.get("canonical_pcm"), "IET PCM"),
                "physical_parent_id": require_string(lineage.get("physical_parent_id"), "IET parent"),
                "project_id": require_string(lineage.get("project_id"), "IET project"),
                "record_id": record_id,
                "role": "generator_train",
                "source_artifact": "ieteasy_increment",
                "source_component_id": require_string(lineage.get("source_component_id"), "IET component"),
                "target": target,
                "target_contract": require_string(require_dict(item.get("target"), "IET target source").get("schema"), "IET target schema"),
            }
        )
    if set(projected) != seen:
        raise CorpusCompilerError("IET projection is not exact")
    measured = require_dict(report.get("measured"), "IET measured")
    planning = {
        "parent_deficit": require_nonnegative_int(
            measured.get("supported_parent_deficit_after"), "IET parent deficit"
        ),
        "parent_floor": require_nonnegative_int(
            require_dict(manifest.get("base_planning"), "IET base planning").get("parent_floor"),
            "IET parent floor",
        ),
        "supported_parents": require_nonnegative_int(
            measured.get("supported_parents_after"), "IET supported parents"
        ),
    }
    return compiled, planning


def validate_role_isolation(rows: list[dict[str, Any]]) -> None:
    record_ids = [item["record_id"] for item in rows]
    if len(record_ids) != len(set(record_ids)):
        raise CorpusCompilerError("real record IDs collide")
    parent_roles: dict[str, set[str]] = defaultdict(set)
    project_roles: dict[str, set[str]] = defaultdict(set)
    component_roles: dict[str, set[str]] = defaultdict(set)
    for item in rows:
        parent_roles[item["physical_parent_id"]].add(item["role"])
        project_roles[item["project_id"]].add(item["role"])
        component_roles[item["source_component_id"]].add(item["role"])
    if any(len(roles) != 1 for roles in parent_roles.values()):
        raise CorpusCompilerError("physical parent crosses roles")
    if any(len(roles) != 1 for roles in project_roles.values()):
        raise CorpusCompilerError("project crosses roles")
    if any(len(roles) != 1 for roles in component_roles.values()):
        raise CorpusCompilerError("source component crosses roles")


def validate_expected(
    expected: dict[str, Any], groups: list[dict[str, Any]], control_rows: int,
    real_rows: list[dict[str, Any]], planning: dict[str, int]
) -> dict[str, Any]:
    role_counts = Counter(item["role"] for item in real_rows)
    parents = {item["physical_parent_id"] for item in real_rows}
    projects = {item["project_id"] for item in real_rows}
    validator_projects = {
        item["project_id"]
        for item in real_rows
        if item["role"] == "validator_calibration"
    }
    measured = {
        "clatter_control_groups": len(groups),
        "clatter_control_rows": control_rows,
        "content_references": 2 * len(real_rows),
        "external_modal_teacher_rows": 0,
        "planning_parent_deficit": planning["parent_deficit"],
        "planning_parent_floor": planning["parent_floor"],
        "planning_supported_parents": planning["supported_parents"],
        "real_physical_parents": len(parents),
        "real_projects": len(projects),
        "real_records": len(real_rows),
        "role_counts": {role: role_counts[role] for role in ROLES},
        "structural_transfer_rows": 0,
        "validator_projects": len(validator_projects),
    }
    if measured != expected:
        raise CorpusCompilerError(f"compiled corpus counts changed: {measured}")
    return measured


def validate_output(path: Path) -> None:
    root = repository_root().resolve()
    resolved = path.resolve()
    if resolved == root or root in resolved.parents:
        raise CorpusCompilerError("output must stay outside the repository")
    if path.exists():
        raise CorpusCompilerError("output must not already exist")
    if not path.parent.is_dir():
        raise CorpusCompilerError("output parent must exist")


def publish(output: Path, documents: dict[str, Any]) -> None:
    validate_output(output)
    temporary = Path(tempfile.mkdtemp(prefix=f".{output.name}-", dir=output.parent))
    try:
        for name, document in sorted(documents.items()):
            (temporary / name).write_bytes(canonical_json(document))
        os.replace(temporary, output)
    except Exception:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def run(profile_path: Path, inputs_directory: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "profile")
    input_bindings, expected = validate_profile(profile)
    documents, source_artifacts, metadata_bytes = validate_input_directory(
        inputs_directory, input_bindings
    )
    validate_output(output)

    validate_r0_ledger(documents["r0-ledger.json"])
    vector_dimension, lane_fields = validate_t0_contract(documents["t0-contract.json"])
    validate_d0_sources(documents["d0-sources.json"])
    groups, control_rows, control_scalars = compile_clatter_groups(
        documents["c0-clatter-prior.json"]
    )
    c0r_rows = compile_c0r_rows(
        documents["c0r-manifest.json"],
        {
            "generator_development": documents["c0r-generator-development.json"],
            "generator_train": documents["c0r-generator-train.json"],
            "validator_calibration": documents["c0r-validator-calibration.json"],
        },
        documents["c0r-report.json"],
    )
    iet_rows, planning = compile_iet_rows(
        documents["iet-manifest.json"],
        documents["iet-generator-train.json"],
        documents["iet-report.json"],
    )
    real_rows = sorted(
        c0r_rows + iet_rows,
        key=lambda item: (item["role"], item["source_artifact"], item["record_id"]),
    )
    validate_role_isolation(real_rows)
    measured = validate_expected(
        expected, groups, control_rows, real_rows, planning
    )

    index = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "control_lanes": {
            "empirical_prior": {
                "authority": "control_only",
                "groups": groups,
                "row_count": control_rows,
                "source_artifact": "c0-clatter-prior.json",
            },
            "modal_teacher": {
                "reason": "D0_NoTrustedSyntheticTeacher",
                "rows": [],
            },
            "structural_transfer": {
                "reason": "NoMaterializedStructuralRowsYet",
                "rows": [],
            },
        },
        "planning_power": planning,
        "real_rows": real_rows,
        "recipe_contract": {
            "lane_fields": lane_fields,
            "source_artifact": "t0-contract.json",
            "vector_dimension": vector_dimension,
        },
        "schema": INDEX_SCHEMA,
        "source_artifacts": source_artifacts,
        "study_id": STUDY_ID,
    }
    index_bytes = canonical_json(index)
    access = {
        "counters": {
            **{name: 0 for name in ZERO_CONTENT_COUNTERS},
            "clatter_modal_control_scalars_read": control_scalars,
            "external_metadata_bytes_read": metadata_bytes,
            "external_metadata_files_read": len(input_bindings),
            "referenced_content_objects": measured["content_references"],
        },
        "schema": ACCESS_SCHEMA,
    }
    gates = {
        "claim_lanes_not_interchanged": True,
        "content_objects_remained_closed": all(
            access["counters"][name] == 0 for name in ZERO_CONTENT_COUNTERS
        ),
        "d0_untrusted_sources_add_zero_rows": measured[
            "external_modal_teacher_rows"
        ]
        == 0,
        "expected_counts": True,
        "input_hashes_and_schemas": True,
        "role_project_parent_isolation": True,
    }
    report = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": DECISION,
        "gates": gates,
        "index": {"bytes": len(index_bytes), "sha256": sha256_bytes(index_bytes)},
        "measured": measured,
        "profile": {
            "bytes": len(profile_bytes),
            "sha256": sha256_bytes(profile_bytes),
        },
        "schema": REPORT_SCHEMA,
        "study_id": STUDY_ID,
    }
    if not all(gates.values()):
        raise CorpusCompilerError("D1 publication gates failed")
    publish(
        output,
        {
            "access.json": access,
            "corpus-index.json": index,
            "report.json": report,
        },
    )


def main() -> int:
    arguments = parse_arguments()
    try:
        run(arguments.profile, arguments.inputs, arguments.output)
    except CorpusCompilerError as error:
        print(f"physical-sound V46 D1 corpus compiler failed: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
