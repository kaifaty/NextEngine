#!/usr/bin/env python3
"""Apply the frozen G0B1c0 target to the 15 selected IETeasy records."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any

import numpy as np
import scipy

import physical_sound_v41_c0_disclosed_corpus_v1 as c0
import physical_sound_v44_g0b1c0_noise_robust_target_v1 as g0b1c0

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c1-profile.v1"
PROTOCOL_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c1-ieteasy-corpus-protocol.v1"
)
AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c1-audit.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c1-access-ledger.v1"
MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c1-corpus-increment.v1"
)
PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c1-generator-train-projection.v1"
)
CARD_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c1-corpus-card.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c1-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v44-g0b1c1-ieteasy-corpus.v1.json"
PROTOCOL_PATH = "lab/profiles/physical-sound-v44-g0b1c1-ieteasy-corpus-protocol.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v44_g0b1c1_ieteasy_corpus_v1.py"
C0_OWNER_PATH = "lab/scripts/physical_sound_v41_c0_disclosed_corpus_v1.py"
TARGET_OWNER_PATH = "lab/scripts/physical_sound_v44_g0b1c0_noise_robust_target_v1.py"
TARGET_PROTOCOL_PATH = (
    "lab/profiles/physical-sound-v44-g0b1c0-noise-robust-target-protocol.v1.json"
)

CLAIM = (
    "FROZEN_G0B1C0_TARGET_APPLICATION_TO_15_SIGNAL_BLIND_IETEASY_RECORDS_AND_"
    "ONE_DISCLOSED_TRAIN_INCREMENT / NO_PSEL_MODEL_VALIDATOR_ADMISSION_COOKER_"
    "DEMO_OR_RUNTIME_AUTHORITY"
)
G0B1A_DECISION = "G0B1A_IETEASY_PAYLOAD_REPEATABLE_TARGET_EXTRACTOR_AUDIT_REQUIRED"
G0B1C0_DECISION = "NoiseRobustTargetSyntheticAdmitted"
HASH_LENGTH = 64
MAX_JSON_BYTES = 4 * 1024 * 1024
MAX_FILE_BYTES = 4 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "corpus_increment_materialization_authority": True,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}
ACCESS_POLICY = {
    "candidate_access_allowed": False,
    "disclosed_audio_access_allowed": True,
    "model_access_allowed": False,
    "network_allowed": False,
    "outputs_external": True,
    "protected_access_allowed": False,
    "validator_access_allowed": False,
}
FORBIDDEN_COUNTERS = (
    "candidate_values_read",
    "model_values_read",
    "network_requests",
    "protected_values_read",
    "validator_values_read",
)


class CorpusIncrementError(RuntimeError):
    """G0B1c1 cannot publish a trustworthy disclosed corpus increment."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--protocol", required=True, type=Path)
    parser.add_argument("--g0b1a-root", required=True, type=Path)
    parser.add_argument("--g0b1c0-root", required=True, type=Path)
    parser.add_argument("--payload-root", required=True, type=Path)
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
        raise CorpusIncrementError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise CorpusIncrementError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise CorpusIncrementError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise CorpusIncrementError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise CorpusIncrementError(f"{context} must be an integer >= {minimum}")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise CorpusIncrementError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_FILE_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise CorpusIncrementError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise CorpusIncrementError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context, MAX_JSON_BYTES)
    try:
        value = require_dict(json.loads(data), context)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise CorpusIncrementError(f"{context} is not valid UTF-8 JSON") from error
    if data != canonical_json(value):
        raise CorpusIncrementError(f"{context} must be canonical JSON")
    return data, value


def validate_binding_bytes(data: bytes, binding: dict[str, Any], context: str) -> None:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise CorpusIncrementError(f"{context} binding fields changed")
    require_string(binding["path"], f"{context} path")
    if len(data) != require_int(binding["bytes"], f"{context} bytes", 1):
        raise CorpusIncrementError(f"{context} byte count changed")
    if sha256_bytes(data) != require_hash(binding["sha256"], f"{context} hash"):
        raise CorpusIncrementError(f"{context} hash changed")


def validate_dependencies(profile: dict[str, Any]) -> None:
    root = repository_root()
    seen: set[str] = set()
    for index, raw in enumerate(
        require_list(profile.get("dependency_bindings"), "dependency bindings")
    ):
        binding = require_dict(raw, f"dependency {index}")
        path_text = require_string(binding.get("path"), f"dependency {index} path")
        relative = Path(path_text)
        if relative.is_absolute() or ".." in relative.parts or path_text in seen:
            raise CorpusIncrementError("dependency path is invalid or duplicated")
        seen.add(path_text)
        data = read_regular(root / relative, f"dependency {path_text}")
        validate_binding_bytes(data, binding, f"dependency {path_text}")
    required = {OWNER_PATH, PROTOCOL_PATH, C0_OWNER_PATH, TARGET_OWNER_PATH}
    if not required.issubset(seen):
        raise CorpusIncrementError("required dependency binding is missing")


def validate_environment(profile: dict[str, Any]) -> Path:
    environment = require_dict(profile.get("environment"), "environment")
    if set(environment) != {"ffmpeg", "numpy_version", "scipy_version"}:
        raise CorpusIncrementError("environment fields changed")
    binding = require_dict(environment["ffmpeg"], "ffmpeg binding")
    path = Path(require_string(binding.get("path"), "ffmpeg path"))
    validate_binding_bytes(read_regular(path, "ffmpeg"), binding, "ffmpeg")
    if environment["numpy_version"] != np.__version__:
        raise CorpusIncrementError("NumPy version changed")
    if environment["scipy_version"] != scipy.__version__:
        raise CorpusIncrementError("SciPy version changed")
    return path


def validate_profile(profile: dict[str, Any]) -> Path:
    required = {
        "access_policy",
        "authority",
        "claim",
        "dependency_bindings",
        "environment",
        "expected_input",
        "profile_id",
        "protocol",
        "schema",
    }
    if set(profile) != required:
        raise CorpusIncrementError("execution profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM:
        raise CorpusIncrementError("execution profile schema or claim changed")
    if profile["authority"] != AUTHORITY or profile["access_policy"] != ACCESS_POLICY:
        raise CorpusIncrementError("execution authority or access policy changed")
    expected = require_dict(profile["expected_input"], "expected input")
    if set(expected) != {"physical_parents", "selected_payload_bytes", "waveforms"}:
        raise CorpusIncrementError("expected input fields changed")
    for key, value in expected.items():
        require_int(value, f"expected input {key}", 1)
    validate_dependencies(profile)
    return validate_environment(profile)


def validate_protocol(protocol: dict[str, Any]) -> None:
    required = {
        "access_policy",
        "authority",
        "base_planning",
        "claim",
        "decision_policy",
        "input_bindings",
        "profile_id",
        "role_policy",
        "schema",
        "target_binding",
    }
    if set(protocol) != required:
        raise CorpusIncrementError("protocol fields changed")
    if protocol["schema"] != PROTOCOL_SCHEMA or protocol["claim"] != CLAIM:
        raise CorpusIncrementError("protocol schema or claim changed")
    if protocol["authority"] != AUTHORITY or protocol["access_policy"] != ACCESS_POLICY:
        raise CorpusIncrementError("protocol authority or access policy changed")
    planning = require_dict(protocol["base_planning"], "base planning")
    if planning.get("supported_parents_before") + planning.get(
        "supported_parent_deficit_before"
    ) != planning.get("parent_floor"):
        raise CorpusIncrementError("base planning counts do not close")
    decision = require_dict(protocol["decision_policy"], "decision policy")
    if decision.get("require_target_schema") != g0b1c0.TARGET_SCHEMA:
        raise CorpusIncrementError("required target schema changed")
    if decision.get("require_target_vector_dimension") != 75:
        raise CorpusIncrementError("required target dimension changed")
    if decision.get("minimum_unique_target_hashes") != 15:
        raise CorpusIncrementError("target uniqueness gate changed")
    role = require_dict(protocol["role_policy"], "role policy")
    if role.get("required_role_intent") != "generator_train":
        raise CorpusIncrementError("frozen disclosed role changed")
    if role.get("project_must_not_cross_roles") is not True:
        raise CorpusIncrementError("project role co-location changed")
    bindings = require_dict(protocol["input_bindings"], "input bindings")
    if set(bindings) != {
        "g0b1a_media_probe",
        "g0b1a_report",
        "g0b1a_selection",
        "g0b1c0_report",
    }:
        raise CorpusIncrementError("input binding set changed")
    target = require_dict(protocol["target_binding"], "target binding")
    if set(target) != {"bytes", "path", "schema", "sha256"}:
        raise CorpusIncrementError("target binding fields changed")
    if (
        target["path"] != TARGET_PROTOCOL_PATH
        or target["schema"] != g0b1c0.PROTOCOL_SCHEMA
    ):
        raise CorpusIncrementError("target protocol binding changed")


def load_protocol(
    profile: dict[str, Any], protocol_path: Path
) -> tuple[bytes, dict[str, Any]]:
    data, protocol = read_canonical_json(protocol_path, "corpus protocol")
    binding = require_dict(profile["protocol"], "protocol binding")
    validate_binding_bytes(data, binding, "corpus protocol")
    if binding["path"] != PROTOCOL_PATH:
        raise CorpusIncrementError("corpus protocol path changed")
    validate_protocol(protocol)
    return data, protocol


def load_target_protocol(protocol: dict[str, Any]) -> dict[str, Any]:
    binding = protocol["target_binding"]
    data, target = read_canonical_json(
        repository_root() / binding["path"], "frozen target protocol"
    )
    if (
        len(data) != binding["bytes"]
        or sha256_bytes(data) != binding["sha256"]
        or target.get("schema") != binding["schema"]
    ):
        raise CorpusIncrementError("frozen target protocol binding changed")
    try:
        g0b1c0.validate_protocol(target)
    except g0b1c0.SyntheticTargetError as error:
        raise CorpusIncrementError(
            f"frozen target protocol is invalid: {error}"
        ) from error
    return target


def load_external_input(
    root: Path, binding: dict[str, Any], context: str
) -> dict[str, Any]:
    if set(binding) != {"bytes", "external_relative_path", "schema", "sha256"}:
        raise CorpusIncrementError(f"{context} binding fields changed")
    relative = Path(
        require_string(binding["external_relative_path"], f"{context} path")
    )
    if relative.is_absolute() or ".." in relative.parts:
        raise CorpusIncrementError(f"{context} path escapes its root")
    data, document = read_canonical_json(root / relative, context)
    if len(data) != binding["bytes"] or sha256_bytes(data) != binding["sha256"]:
        raise CorpusIncrementError(f"{context} binding changed")
    if document.get("schema") != binding["schema"]:
        raise CorpusIncrementError(f"{context} schema changed")
    return document


def validate_external_root(root: Path, context: str) -> None:
    if root.is_symlink() or not root.is_dir():
        raise CorpusIncrementError(f"{context} must be a non-symlink directory")


def load_inputs(
    g0b1a_root: Path,
    g0b1c0_root: Path,
    protocol: dict[str, Any],
    expected: dict[str, Any],
) -> tuple[list[dict[str, Any]], dict[str, dict[str, Any]]]:
    validate_external_root(g0b1a_root, "G0B1a root")
    validate_external_root(g0b1c0_root, "G0B1c0 root")
    bindings = protocol["input_bindings"]
    selection = load_external_input(
        g0b1a_root, bindings["g0b1a_selection"], "G0B1a selection"
    )
    media = load_external_input(
        g0b1a_root, bindings["g0b1a_media_probe"], "G0B1a media"
    )
    prior_report = load_external_input(
        g0b1a_root, bindings["g0b1a_report"], "G0B1a report"
    )
    target_report = load_external_input(
        g0b1c0_root, bindings["g0b1c0_report"], "G0B1c0 report"
    )
    if prior_report.get("decision") != G0B1A_DECISION:
        raise CorpusIncrementError("G0B1a decision changed")
    if target_report.get("decision") != G0B1C0_DECISION:
        raise CorpusIncrementError("G0B1c0 target is not admitted")
    if (
        target_report.get("gates", {}).get("ieteasy_target_access_authorized")
        is not True
    ):
        raise CorpusIncrementError("G0B1c0 did not authorize IETeasy target access")
    records = [require_dict(row, "selected record") for row in selection["records"]]
    media_rows = [require_dict(row, "media row") for row in media["records"]]
    by_record = {row.get("record_id"): row for row in media_rows}
    if len(records) != expected["waveforms"] or len(by_record) != len(media_rows):
        raise CorpusIncrementError("G0B1a record count or identity changed")
    parents = {row["lineage"]["physical_parent_id"] for row in records}
    payload_bytes = sum(row["record"]["bytes"] for row in records)
    if (
        len(parents) != expected["physical_parents"]
        or payload_bytes != expected["selected_payload_bytes"]
    ):
        raise CorpusIncrementError("G0B1a parent or payload count changed")
    if {row["record"]["record_id"] for row in records} != set(by_record):
        raise CorpusIncrementError("G0B1a selection/media record IDs disagree")
    validate_role_roster(records, protocol["role_policy"])
    return records, by_record


def validate_role_roster(records: list[dict[str, Any]], policy: dict[str, Any]) -> None:
    if len(records) != policy["expected_physical_parents"]:
        raise CorpusIncrementError("role roster parent count changed")
    parents = [row["lineage"]["physical_parent_id"] for row in records]
    if len(set(parents)) != len(parents):
        raise CorpusIncrementError("role roster repeats a physical parent")
    for row in records:
        lineage = row["lineage"]
        if (
            row.get("role_intent") != policy["required_role_intent"]
            or lineage.get("project_id") != policy["expected_project_id"]
            or lineage.get("revision_id") != policy["expected_revision_id"]
            or lineage.get("source_component_id")
            != policy["expected_source_component_id"]
        ):
            raise CorpusIncrementError("frozen project/role roster changed")
    counts = dict(sorted(Counter(row["material_label"] for row in records).items()))
    if counts != policy["expected_material_parent_counts"]:
        raise CorpusIncrementError("frozen material-parent roster changed")


def validate_target(target: dict[str, Any], policy: dict[str, Any]) -> None:
    if target.get("schema") != policy["require_target_schema"]:
        raise CorpusIncrementError("target schema changed")
    if target.get("vector_dimension") != policy["require_target_vector_dimension"]:
        raise CorpusIncrementError("target vector dimension changed")
    if len(g0b1c0.target_vector(target)) != policy["require_target_vector_dimension"]:
        raise CorpusIncrementError("target vector layout changed")
    tonal = require_list(target.get("tonal_excess_energy_ppm_by_band"), "tonal bands")
    residual = require_list(target.get("residual_energy_ppm_by_band"), "residual bands")
    counts = require_list(target.get("qualified_peak_count_by_band"), "peak counts")
    if len(tonal) != 24 or len(residual) != 24 or len(counts) != 24:
        raise CorpusIncrementError("target fixed-size partitions changed")
    if sum(tonal) + sum(residual) != policy["require_energy_partition_ppm"]:
        raise CorpusIncrementError("target energy partition does not close")
    if sum(counts) != target.get("qualified_peak_count"):
        raise CorpusIncrementError("target peak-count partition does not close")


def content_path(kind: str, digest: str) -> str:
    require_hash(digest, "content hash")
    if kind not in {"pcm", "target"}:
        raise CorpusIncrementError("content kind changed")
    return f"objects/{kind}/{digest[:2]}/{digest}"


def add_content(objects: dict[str, bytes], path: str, data: bytes) -> None:
    existing = objects.setdefault(path, data)
    if existing != data:
        raise CorpusIncrementError("content-address collision")


def materialize_record(
    selected: dict[str, Any],
    prior: dict[str, Any],
    payload_root: Path,
    ffmpeg_path: Path,
    target_protocol: dict[str, Any],
    decision_policy: dict[str, Any],
) -> tuple[dict[str, Any], dict[str, bytes], int]:
    source = require_dict(selected["record"], "selected source record")
    digest = require_hash(source.get("sha256"), "source payload hash")
    payload = read_regular(payload_root / digest, f"payload {digest}")
    if len(payload) != source["bytes"] or sha256_bytes(payload) != digest:
        raise CorpusIncrementError("publisher payload byte/hash binding changed")
    try:
        decoded = c0.decode_audio(payload, 48000, ffmpeg_path)
        canonical, normalization = c0.canonical_segment(decoded)
    except c0.CorpusError as error:
        raise CorpusIncrementError(f"canonical decode failed: {error}") from error
    pcm = c0.wav_pcm16_bytes(canonical)
    pcm_hash = sha256_bytes(pcm)
    if decision_policy["require_canonical_pcm_identity"] and (
        pcm_hash != prior["canonical_pcm"]["sha256"]
        or len(pcm) != prior["canonical_pcm"]["bytes"]
        or normalization != prior["normalization"]
    ):
        raise CorpusIncrementError("G0B1a canonical PCM identity changed")
    try:
        target = g0b1c0.extract_target(canonical, target_protocol)
    except g0b1c0.SyntheticTargetError as error:
        raise CorpusIncrementError(
            f"frozen target extraction failed: {error}"
        ) from error
    validate_target(target, decision_policy)
    target_bytes = canonical_json(target)
    if len(target_bytes) > decision_policy["maximum_target_canonical_json_bytes"]:
        raise CorpusIncrementError("target byte resource bound exceeded")
    target_hash = sha256_bytes(target_bytes)
    pcm_path = content_path("pcm", pcm_hash)
    target_path = content_path("target", target_hash)
    objects: dict[str, bytes] = {}
    add_content(objects, pcm_path, pcm)
    add_content(objects, target_path, target_bytes)
    record = {
        "canonical_pcm": {
            "bytes": len(pcm),
            "object_path": pcm_path,
            "sha256": pcm_hash,
        },
        "descriptor": selected["descriptor"],
        "lineage": selected["lineage"],
        "material_label": selected["material_label"],
        "record_id": source["record_id"],
        "role": selected["role_intent"],
        "sample_id": selected["sample_id"],
        "source_payload": {"bytes": len(payload), "sha256": digest},
        "target": {
            "bytes": len(target_bytes),
            "object_path": target_path,
            "qualified_peak_count": target["qualified_peak_count"],
            "schema": target["schema"],
            "sha256": target_hash,
            "spectral_flatness_ppm": target["spectral_flatness_ppm"],
            "tonal_excess_mass_ppm": target["tonal_excess_mass_ppm"],
            "vector_dimension": target["vector_dimension"],
        },
    }
    return record, objects, len(decoded)


def target_values(record: dict[str, Any], objects: dict[str, bytes]) -> list[int]:
    path = record["target"]["object_path"]
    try:
        target = json.loads(objects[path])
    except (KeyError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise CorpusIncrementError(
            "materialized target object is unreadable"
        ) from error
    return g0b1c0.target_vector(target)


def dataset_gates(
    records: list[dict[str, Any]],
    objects: dict[str, bytes],
    protocol: dict[str, Any],
) -> tuple[dict[str, bool], dict[str, Any]]:
    policy = protocol["decision_policy"]
    vectors = np.asarray(
        [target_values(row, objects) for row in records], dtype=np.int64
    )
    if vectors.shape != (len(records), policy["require_target_vector_dimension"]):
        raise CorpusIncrementError("materialized target matrix shape changed")
    unique_hashes = len({row["target"]["sha256"] for row in records})
    varying_dimensions = int(np.count_nonzero(np.ptp(vectors, axis=0) > 0))
    target_sizes = [row["target"]["bytes"] for row in records]
    peak_counts = [row["target"]["qualified_peak_count"] for row in records]
    tonal_mass = [row["target"]["tonal_excess_mass_ppm"] for row in records]
    flatness = [row["target"]["spectral_flatness_ppm"] for row in records]
    gates = {
        "one_project_one_role": len({row["role"] for row in records}) == 1
        and len({row["lineage"]["project_id"] for row in records}) == 1,
        "target_resource_bounds": max(target_sizes)
        <= policy["maximum_target_canonical_json_bytes"],
        "target_uniqueness": unique_hashes >= policy["minimum_unique_target_hashes"],
        "target_variation": varying_dimensions
        >= policy["minimum_varying_target_dimensions"],
    }
    diagnostics = {
        "qualified_peak_count_maximum": max(peak_counts),
        "qualified_peak_count_median": round(float(np.median(peak_counts)), 6),
        "qualified_peak_count_minimum": min(peak_counts),
        "spectral_flatness_ppm_maximum": max(flatness),
        "spectral_flatness_ppm_minimum": min(flatness),
        "target_bytes_maximum": max(target_sizes),
        "target_bytes_minimum": min(target_sizes),
        "tonal_excess_mass_ppm_maximum": max(tonal_mass),
        "tonal_excess_mass_ppm_minimum": min(tonal_mass),
        "unique_target_hashes": unique_hashes,
        "varying_target_dimensions": varying_dimensions,
    }
    return gates, diagnostics


def build_outputs(
    profile: dict[str, Any],
    protocol: dict[str, Any],
    records: list[dict[str, Any]],
    objects: dict[str, bytes],
    decoded_samples: int,
) -> tuple[dict[str, Any], dict[str, bytes]]:
    gates, diagnostics = dataset_gates(records, objects, protocol)
    passed = all(gates.values())
    policy = protocol["decision_policy"]
    decision = policy["pass_decision"] if passed else policy["failure_decision"]
    record_root = sha256_bytes(canonical_json(records))
    projection_rows = [
        {
            "physical_parent_id": row["lineage"]["physical_parent_id"],
            "record_id": row["record_id"],
            "target_sha256": row["target"]["sha256"],
        }
        for row in records
    ]
    projection_root = sha256_bytes(canonical_json(projection_rows))
    planning = protocol["base_planning"]
    after = (
        planning["supported_parents_before"] + len(records)
        if passed
        else planning["supported_parents_before"]
    )
    deficit = planning["parent_floor"] - after
    audit = {
        "claim": CLAIM,
        "decision": decision,
        "diagnostics": diagnostics,
        "gates": gates,
        "record_summaries": [
            {
                "physical_parent_id": row["lineage"]["physical_parent_id"],
                "record_id": row["record_id"],
                "target": row["target"],
            }
            for row in records
        ],
        "schema": AUDIT_SCHEMA,
    }
    outputs: dict[str, Any] = {
        "access-ledger.json": {
            "claim": CLAIM,
            "counters": {
                "candidate_values_read": 0,
                "decoded_sample_values": decoded_samples,
                "disclosed_audio_payload_bytes_read": profile["expected_input"][
                    "selected_payload_bytes"
                ],
                "disclosed_audio_payload_files_read": len(records),
                "model_values_read": 0,
                "network_requests": 0,
                "protected_values_read": 0,
                "target_extractions": len(records),
                "validator_values_read": 0,
            },
            "schema": ACCESS_SCHEMA,
        },
        "audit.json": audit,
        "report.json": {
            "authority": AUTHORITY,
            "claim": CLAIM,
            "decision": decision,
            "gates": {
                **gates,
                "corpus_increment_materialized": passed,
                "psel_pack_eligible": False,
                "runtime_consumer_allowed": False,
                "training_allowed": False,
            },
            "measured": {
                **diagnostics,
                "decoded_sample_values": decoded_samples,
                "disclosed_role": protocol["role_policy"]["required_role_intent"],
                "physical_parents": len(records),
                "project_revisions": 1,
                "selected_payload_bytes": profile["expected_input"][
                    "selected_payload_bytes"
                ],
                "steel_parents": sum(
                    row["material_label"] == "Steel" for row in records
                ),
                "supported_parent_deficit_after": deficit,
                "supported_parents_after": after,
                "waveforms": len(records),
            },
            "next_action": (
                "continue_the_second_independent_descriptor_to_signal_project_"
                "search_before_psel"
                if passed
                else "return_to_a_new_synthetic_only_target_revision_without_"
                "retuning_from_these_ieteasy_values"
            ),
            "schema": REPORT_SCHEMA,
        },
    }
    if passed:
        manifest = {
            "base_planning": planning,
            "claim": CLAIM,
            "increment_record_root_sha256": record_root,
            "records": records,
            "schema": MANIFEST_SCHEMA,
        }
        projection = {
            "claim": CLAIM,
            "record_root_sha256": projection_root,
            "records": projection_rows,
            "role": protocol["role_policy"]["required_role_intent"],
            "schema": PROJECTION_SCHEMA,
        }
        card = {
            "claim": CLAIM,
            "content_objects": len(objects),
            "increment_record_root_sha256": record_root,
            "material_parent_counts": protocol["role_policy"][
                "expected_material_parent_counts"
            ],
            "physical_parents": len(records),
            "project_revisions": 1,
            "projection_root_sha256": projection_root,
            "role": protocol["role_policy"]["required_role_intent"],
            "schema": CARD_SCHEMA,
            "supported_parent_deficit_after": deficit,
            "supported_parents_after": after,
        }
        outputs.update(
            {
                "corpus-card.json": card,
                "manifest.json": manifest,
                "projections/generator_train.json": projection,
            }
        )
    else:
        objects = {}
    return outputs, objects


def is_within(child: Path, parent: Path) -> bool:
    try:
        child.relative_to(parent)
        return True
    except ValueError:
        return False


def prepare_output(output: Path) -> None:
    repository = repository_root().resolve()
    resolved = output.resolve(strict=False)
    if is_within(resolved, repository):
        raise CorpusIncrementError("output must be outside the repository")
    if output.is_symlink():
        raise CorpusIncrementError("output must not be a symlink")
    if output.exists():
        raise CorpusIncrementError("output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)


def publish(
    output: Path,
    outputs: dict[str, Any],
    objects: dict[str, bytes],
    profile_bytes: bytes,
    protocol_bytes: bytes,
) -> None:
    prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, value in sorted(outputs.items()):
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(canonical_json(value))
        for name, value in sorted(objects.items()):
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(value)
        (staging / "profile.json").write_bytes(profile_bytes)
        (staging / "protocol.json").write_bytes(protocol_bytes)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def run(
    profile_path: Path,
    protocol_path: Path,
    g0b1a_root: Path,
    g0b1c0_root: Path,
    payload_root: Path,
    output: Path,
) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "execution profile")
    ffmpeg_path = validate_profile(profile)
    protocol_bytes, protocol = load_protocol(profile, protocol_path)
    target_protocol = load_target_protocol(protocol)
    selected, prior_by_record = load_inputs(
        g0b1a_root, g0b1c0_root, protocol, profile["expected_input"]
    )
    if (
        payload_root.is_symlink()
        or not payload_root.is_dir()
        or is_within(payload_root.resolve(), repository_root().resolve())
    ):
        raise CorpusIncrementError(
            "payload root must be an external non-symlink directory"
        )
    records = []
    objects: dict[str, bytes] = {}
    decoded_samples = 0
    for selected_record in selected:
        record_id = selected_record["record"]["record_id"]
        record, content, sample_values = materialize_record(
            selected_record,
            prior_by_record[record_id],
            payload_root,
            ffmpeg_path,
            target_protocol,
            protocol["decision_policy"],
        )
        records.append(record)
        decoded_samples += sample_values
        for path, data in content.items():
            add_content(objects, path, data)
    outputs, objects = build_outputs(
        profile, protocol, records, objects, decoded_samples
    )
    for counter in FORBIDDEN_COUNTERS:
        if outputs["access-ledger.json"]["counters"][counter] != 0:
            raise CorpusIncrementError("forbidden access counter is nonzero")
    publish(output, outputs, objects, profile_bytes, protocol_bytes)


def main() -> int:
    arguments = parse_arguments()
    try:
        run(
            arguments.profile,
            arguments.protocol,
            arguments.g0b1a_root,
            arguments.g0b1c0_root,
            arguments.payload_root,
            arguments.output,
        )
    except CorpusIncrementError as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
