#!/usr/bin/env python3
"""Run the V32 V0 label-blind synthetic validator mechanics externally."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
import platform
import shutil
import struct
import tempfile
import time
from pathlib import Path
from typing import Any

import numpy as np


PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0-validator-mechanics-profile.v1"
)
FEATURE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0-validator-features.v1"
)
DECISION_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0-validator-decision.v1"
)
RELEASE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0-validator-mechanics-release.v1"
)
EVIDENCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0-validator-mechanics-evidence.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0-validator-mechanics-report.v1"
)
PROFILE_ID = "physical-sound-v32-v0-validator-mechanics-v1"
PROFILE_SHA256 = "8865cf8ca52f76816ad538c1ebedd44ba8ee68a6a96e1f1980a50cd35ebe4709"
BASELINE_COMMIT = "d67bfabfb9d3d682b5c51be091a273131c7a8048"
T0_PROFILE_SHA256 = "44d83804398b2e08a54dbe28e4e3269098b1112785d8377dac34a11d4e5eebe8"
T0_OWNER_SHA256 = "4a0c5a5ea7cece8dfca2c8d346695bc826b9b375d94c2ef5aa1d723760391d6f"
T0_RESULT_SHA256 = "b17631b4032d632bb0ae9d30e05a421895756d65ce961109da94fed962c15ce9"
PROTOCOL_SHA256 = "301822d41da601d40705e9ea477a5b6e9eaa85085bc4c60f5590e5417709b525"
T0_RELEASE_SHA256 = "6a5ff92000e27b8db325f29a670055147bcc891267087100876866b341a5b1d9"
T0_EVIDENCE_SHA256 = "b55bb66e798e199d40e166686f94bfe35f7f09457585d3edd80250e2e5bcd9b3"
T0_REPORT_SHA256 = "60ab3dd66ed6fd1ae0eba0e25da9cc2a703528e8bf1b775ace56c3fd2be927ec"
CLAIM = (
    "SYNTHETIC_VALIDATOR_MECHANICS_ONLY / NO_REAL_THRESHOLDS_MATERIAL_QUALITY_"
    "VALIDATOR_RELEASE_ML_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 256 * 1024
MAX_JSON_BYTES = 4 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "external_research_only": True,
    "model_allowed": False,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "real_thresholds_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "validator_mechanics_authority": True,
    "validator_release_authority": False,
}
ZERO_ACCESS = {
    "model_parameters_opened": 0,
    "network_requests": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
}
MODAL_STRUCTURE_FIELDS = (
    "ordinal",
    "family_index_a",
    "family_index_b",
    "frequency_hz",
    "contact_participation",
    "pickup_participation",
    "signed_gain",
)


class ValidatorMechanicsError(RuntimeError):
    """The frozen V0 validator cannot produce a complete exact result."""


class CandidateIntegrityError(RuntimeError):
    """One candidate cannot be decoded as canonical PCM/metadata."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise ValidatorMechanicsError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValidatorMechanicsError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise ValidatorMechanicsError(f"non-finite JSON number: {value}")


def parse_canonical_json(data: bytes, label: str) -> dict[str, Any]:
    if not data or len(data) > MAX_JSON_BYTES:
        raise ValidatorMechanicsError(f"{label} JSON size outside bound")
    try:
        value = json.loads(
            data,
            object_pairs_hook=_no_duplicates,
            parse_constant=_reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValidatorMechanicsError(f"invalid {label} JSON: {error}") from error
    if not isinstance(value, dict) or data != canonical_json(value):
        raise ValidatorMechanicsError(f"{label} is not canonical JSON")
    return value


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise ValidatorMechanicsError("profile must not be a symlink")
    try:
        data = path.read_bytes()
    except OSError as error:
        raise ValidatorMechanicsError(f"cannot read V0 profile: {error}") from error
    if len(data) > MAX_PROFILE_BYTES:
        raise ValidatorMechanicsError("V0 profile exceeds size bound")
    profile = parse_canonical_json(data, "V0 profile")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise ValidatorMechanicsError("V0 profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or profile.get("authority") != AUTHORITY
    ):
        raise ValidatorMechanicsError("V0 frozen identity or authority mismatch")
    publication = profile.get("publication")
    resources = profile.get("resources")
    numeric = profile.get("numeric_profile")
    decision = profile.get("decision_contract")
    if not all(
        isinstance(item, dict)
        for item in (publication, resources, numeric, decision)
    ):
        raise ValidatorMechanicsError("V0 publication/resource contract missing")
    if (
        publication.get("candidate_count") != 16
        or publication.get("expected_file_count") != 35
        or publication.get("candidate_count")
        > resources.get("max_candidate_count", -1)
        or publication.get("expected_file_count") > resources.get("max_file_count", -1)
        or numeric.get("sample_rate_hz") != 48_000
        or numeric.get("frame_count") != 144_000
        or numeric.get("randomness") != "forbidden"
        or decision.get("expected_pass_count") != 9
        or decision.get("expected_reject_count") != 7
    ):
        raise ValidatorMechanicsError("V0 frozen publication/resource values mismatch")
    truth = profile.get("truth")
    if not isinstance(truth, dict):
        raise ValidatorMechanicsError("V0 truth contract missing")
    if (
        truth.get("truth_release_sha256") != T0_RELEASE_SHA256
        or truth.get("evidence_sha256") != T0_EVIDENCE_SHA256
        or truth.get("report_sha256") != T0_REPORT_SHA256
        or len(truth.get("clean_case_ids", [])) != 9
        or len(truth.get("mutation_expectations", [])) != 7
    ):
        raise ValidatorMechanicsError("V0 frozen truth identities mismatch")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    root = repository_root()
    expected = {
        "t0_owner": (
            "lab/scripts/physical_sound_v32_t0_truth_mutations_v1.py",
            T0_OWNER_SHA256,
        ),
        "t0_profile": (
            "lab/profiles/physical-sound-v32-t0-truth-mutations.v1.json",
            T0_PROFILE_SHA256,
        ),
        "t0_result": (
            "docs/development/physical-sound-v32-t0-truth-mutation-result-2026-09-02.md",
            T0_RESULT_SHA256,
        ),
    }
    parent = profile.get("parent")
    if not isinstance(parent, dict) or set(parent) != set(expected):
        raise ValidatorMechanicsError("V0 parent dependency set mismatch")
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, (relative, expected_sha256) in expected.items():
        declared = parent[dependency_id]
        if declared != {"path": relative, "sha256": expected_sha256}:
            raise ValidatorMechanicsError(
                f"V0 declared dependency mismatch: {dependency_id}"
            )
        data = (root / relative).read_bytes()
        actual = sha256_bytes(data)
        if actual != expected_sha256:
            raise ValidatorMechanicsError(f"V0 dependency drift: {dependency_id}")
        result[dependency_id] = {
            "bytes": len(data),
            "path": relative,
            "sha256": actual,
        }
    protocol = profile.get("protocol")
    expected_protocol = {
        "path": "docs/development/physical-sound-v32-v0-validator-mechanics-protocol-2026-09-02.md",
        "sha256": PROTOCOL_SHA256,
    }
    if protocol != expected_protocol:
        raise ValidatorMechanicsError("V0 protocol identity mismatch")
    data = (root / expected_protocol["path"]).read_bytes()
    if sha256_bytes(data) != PROTOCOL_SHA256:
        raise ValidatorMechanicsError("V0 protocol drift")
    result["v0_protocol"] = {"bytes": len(data), **expected_protocol}
    return result


def external_truth(path: Path, max_input_bytes: int) -> Path:
    root = repository_root().resolve(strict=True)
    if path.is_symlink():
        raise ValidatorMechanicsError("truth input must not be a symlink")
    try:
        truth = path.resolve(strict=True)
    except OSError as error:
        raise ValidatorMechanicsError(f"truth input missing: {error}") from error
    if not truth.is_dir() or truth.is_relative_to(root):
        raise ValidatorMechanicsError("truth input must be an external directory")
    total = sum(item.stat().st_size for item in truth.rglob("*") if item.is_file())
    if total > max_input_bytes:
        raise ValidatorMechanicsError("truth input exceeds resource profile")
    return truth


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise ValidatorMechanicsError("output must not be a symlink")
    try:
        parent = unresolved.parent.resolve(strict=True)
    except OSError as error:
        raise ValidatorMechanicsError(f"output parent must exist: {error}") from error
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise ValidatorMechanicsError("output must be a fresh external path")
    return output


def load_truth(
    path: Path, profile: dict[str, Any]
) -> tuple[dict[str, dict[str, Any]], dict[str, bytes], dict[str, Any]]:
    truth = external_truth(path, profile["resources"]["max_input_bytes"])
    release_data = (truth / "truth-release.json").read_bytes()
    evidence_data = (truth / "evidence.json").read_bytes()
    report_data = (truth / "report.json").read_bytes()
    if sha256_bytes(release_data) != T0_RELEASE_SHA256:
        raise ValidatorMechanicsError("T0 truth release drift")
    if sha256_bytes(evidence_data) != T0_EVIDENCE_SHA256:
        raise ValidatorMechanicsError("T0 evidence drift")
    if sha256_bytes(report_data) != T0_REPORT_SHA256:
        raise ValidatorMechanicsError("T0 report drift")
    release = parse_canonical_json(release_data, "T0 truth release")
    evidence = parse_canonical_json(evidence_data, "T0 evidence")
    report = parse_canonical_json(report_data, "T0 report")
    if (
        release.get("status") != "Pass"
        or report.get("decision") != "T0_TRUTH_MUTATION_LIBRARY_PASS"
        or report.get("truth_release_sha256") != T0_RELEASE_SHA256
        or report.get("evidence_sha256") != T0_EVIDENCE_SHA256
        or evidence.get("truth_release_sha256") != T0_RELEASE_SHA256
        or release.get("profile_identity", {}).get("sha256") != T0_PROFILE_SHA256
        or release.get("owner_identity", {}).get("sha256") != T0_OWNER_SHA256
    ):
        raise ValidatorMechanicsError("T0 truth closure mismatch")
    summaries = release.get("records")
    if not isinstance(summaries, list) or len(summaries) != 16:
        raise ValidatorMechanicsError("T0 truth record index mismatch")
    record_ids = [item.get("record_id") for item in summaries]
    if record_ids != sorted(record_ids) or len(record_ids) != len(set(record_ids)):
        raise ValidatorMechanicsError("T0 truth record order/uniqueness mismatch")

    records: dict[str, dict[str, Any]] = {}
    audio: dict[str, bytes] = {}
    expected_paths = {"truth-release.json", "evidence.json", "report.json"}
    record_sha256: dict[str, str] = {}
    for summary in summaries:
        record_id = summary["record_id"]
        base = Path("records") / record_id
        record_relative = str(base / "record.json")
        audio_relative = str(base / "audio.wav")
        expected_paths.update({record_relative, audio_relative})
        record_data = (truth / record_relative).read_bytes()
        audio_data = (truth / audio_relative).read_bytes()
        if (
            sha256_bytes(record_data) != summary.get("record_sha256")
            or sha256_bytes(audio_data) != summary.get("audio_sha256")
        ):
            raise ValidatorMechanicsError(f"T0 record artifact drift: {record_id}")
        record = parse_canonical_json(record_data, f"T0 record {record_id}")
        if record.get("record_id") != record_id:
            raise ValidatorMechanicsError(f"T0 record identity mismatch: {record_id}")
        records[record_id] = record
        audio[record_id] = audio_data
        record_sha256[record_id] = summary["record_sha256"]
    actual_paths = {
        str(item.relative_to(truth))
        for item in truth.rglob("*")
        if item.is_file()
    }
    if actual_paths != expected_paths or len(actual_paths) != 35:
        raise ValidatorMechanicsError("T0 truth directory closure mismatch")
    clean_ids = profile["truth"]["clean_case_ids"]
    if any(record_id not in records for record_id in clean_ids):
        raise ValidatorMechanicsError("T0 clean truth record missing")
    return records, audio, {
        "evidence_sha256": T0_EVIDENCE_SHA256,
        "record_sha256": record_sha256,
        "report_sha256": T0_REPORT_SHA256,
        "root": truth,
        "truth_release_sha256": T0_RELEASE_SHA256,
    }


def parse_float32_wav(data: bytes, sample_rate_hz: int, frame_count: int) -> np.ndarray:
    expected_bytes = 44 + frame_count * 4
    if len(data) != expected_bytes:
        raise CandidateIntegrityError("WAV byte/frame length mismatch")
    try:
        riff_size = struct.unpack_from("<I", data, 4)[0]
        fmt_size = struct.unpack_from("<I", data, 16)[0]
        audio_format, channels, rate, byte_rate, block_align, bits = struct.unpack_from(
            "<HHIIHH", data, 20
        )
        payload_size = struct.unpack_from("<I", data, 40)[0]
    except struct.error as error:
        raise CandidateIntegrityError("truncated WAV header") from error
    if (
        data[:4] != b"RIFF"
        or riff_size != len(data) - 8
        or data[8:16] != b"WAVEfmt "
        or fmt_size != 16
        or audio_format != 3
        or channels != 1
        or rate != sample_rate_hz
        or byte_rate != sample_rate_hz * 4
        or block_align != 4
        or bits != 32
        or data[36:40] != b"data"
        or payload_size != frame_count * 4
    ):
        raise CandidateIntegrityError("non-canonical float32 WAV")
    samples = np.frombuffer(data, dtype="<f4", offset=44).astype(np.float64)
    if len(samples) != frame_count or np.any(~np.isfinite(samples)):
        raise CandidateIntegrityError("invalid or non-finite PCM")
    return samples


def candidate_view(record: dict[str, Any], profile: dict[str, Any]) -> dict[str, Any]:
    allowed = profile["candidate_view"]["allowed_fields"]
    if any(field not in record for field in allowed):
        raise CandidateIntegrityError("candidate view field missing")
    return {field: copy.deepcopy(record[field]) for field in allowed}


def decoded_hash(value: str, label: str) -> bytes:
    try:
        result = bytes.fromhex(value)
    except (TypeError, ValueError) as error:
        raise ValidatorMechanicsError(f"invalid {label} SHA-256") from error
    if len(result) != 32:
        raise ValidatorMechanicsError(f"invalid {label} SHA-256 length")
    return result


def feature_cache_key(
    profile: dict[str, Any],
    candidate_view_sha256: str,
    audio_sha256: str,
    target_record_sha256: str,
) -> str:
    separator = profile["cache"]["domain_separator"]
    if not separator.endswith("\\0"):
        raise ValidatorMechanicsError("cache domain separator mismatch")
    domain = separator[:-2].encode("utf-8") + b"\0"
    return sha256_bytes(
        domain
        + decoded_hash(PROFILE_SHA256, "profile")
        + decoded_hash(candidate_view_sha256, "candidate view")
        + decoded_hash(audio_sha256, "candidate audio")
        + decoded_hash(target_record_sha256, "target record")
    )


def rms(values: np.ndarray) -> float:
    if values.size == 0:
        return 0.0
    return float(math.sqrt(float(np.mean(values * values))))


def modal_values(record: dict[str, Any]) -> list[dict[str, Any]]:
    modal = record.get("modal")
    if not isinstance(modal, dict) or not isinstance(modal.get("modes"), list):
        return []
    return [item for item in modal["modes"] if isinstance(item, dict)]


def make_decision(
    record_id: str,
    cache_key: str | None,
    decision: str,
    reason_codes: list[str],
) -> dict[str, Any]:
    return {
        "cache_key": cache_key,
        "claim": CLAIM,
        "decision": decision,
        "reason_codes": reason_codes,
        "record_id": record_id,
        "schema": DECISION_SCHEMA,
    }


def validate_candidate(
    profile: dict[str, Any],
    record: dict[str, Any],
    audio_data: bytes,
    clean_records: dict[str, dict[str, Any]],
    clean_audio: dict[str, bytes],
    clean_record_sha256: dict[str, str],
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    record_id = str(record.get("record_id", "invalid-record"))
    target_case_id = record.get("target_case_id")
    if target_case_id not in clean_records:
        return None, make_decision(
            record_id,
            None,
            profile["decision_contract"]["out_of_domain"]["decision"],
            [profile["decision_contract"]["out_of_domain"]["reason_code"]],
        )
    target = clean_records[target_case_id]
    view = candidate_view(record, profile)
    view_data = canonical_json(view)
    view_sha256 = sha256_bytes(view_data)
    audio_sha256 = sha256_bytes(audio_data)
    cache_key = feature_cache_key(
        profile,
        view_sha256,
        audio_sha256,
        clean_record_sha256[target_case_id],
    )
    numeric = profile["numeric_profile"]
    try:
        samples = parse_float32_wav(
            audio_data, numeric["sample_rate_hz"], numeric["frame_count"]
        )
    except CandidateIntegrityError as error:
        features = {
            "audio_sha256": audio_sha256,
            "cache_key": cache_key,
            "candidate_view_sha256": view_sha256,
            "claim": CLAIM,
            "integrity_error": str(error),
            "record_id": record_id,
            "schema": FEATURE_SCHEMA,
            "target_clean_record_sha256": clean_record_sha256[target_case_id],
        }
        return features, make_decision(
            record_id, cache_key, "Reject", ["IntegrityMismatch"]
        )

    audio_declaration = view.get("audio")
    audio_identity_exact = isinstance(audio_declaration, dict) and audio_declaration == {
        "bytes": len(audio_data),
        "channels": 1,
        "encoding": "ieee-float32-little-endian",
        "frame_count": numeric["frame_count"],
        "sample_rate_hz": numeric["sample_rate_hz"],
        "sha256": audio_sha256,
    }
    sample_rms = rms(samples)
    sample_mean = float(np.mean(samples))
    integrity = profile["specialists"]["integrity"]
    minimum_rms = float(integrity["minimum_rms"])
    dc_over_rms = abs(sample_mean) / max(sample_rms, minimum_rms)
    clipping_count = int(
        np.count_nonzero(np.abs(samples) >= float(integrity["full_scale"]))
    )

    lineage = view.get("lineage")
    if not isinstance(lineage, dict):
        lineage = {}
    actual_present = "actual_parent_audio_sha256" in lineage
    declared_present = "declared_parent_audio_sha256" in lineage
    actual_parent = lineage.get("actual_parent_audio_sha256")
    declared_parent = lineage.get("declared_parent_audio_sha256")
    clean_audio_hashes = {
        case_id: sha256_bytes(data) for case_id, data in clean_audio.items()
    }
    provenance_mismatch = actual_present != declared_present
    if actual_present and declared_present:
        provenance_mismatch = provenance_mismatch or (
            actual_parent != declared_parent
            or actual_parent not in set(clean_audio_hashes.values())
        )

    copied_target_ids = sorted(
        case_id
        for case_id, clean_sha256 in clean_audio_hashes.items()
        if clean_sha256 == audio_sha256 and case_id != target_case_id
    )

    candidate_modes = modal_values(view)
    target_modes = modal_values(target)
    candidate_mode_count = len(candidate_modes)
    target_mode_count = len(target_modes)
    modal_structure_exact = candidate_mode_count == target_mode_count and all(
        all(candidate.get(field) == expected.get(field) for field in MODAL_STRUCTURE_FIELDS)
        for candidate, expected in zip(candidate_modes, target_modes, strict=True)
    )
    decay_exact = candidate_mode_count == target_mode_count and all(
        candidate.get("decay_per_second") == expected.get("decay_per_second")
        for candidate, expected in zip(candidate_modes, target_modes, strict=True)
    )

    target_decay = [float(mode["decay_per_second"]) for mode in target_modes]
    if not target_decay or any(value != target_decay[0] for value in target_decay):
        raise ValidatorMechanicsError("V0 target lacks one exact scalar decay")
    sample_ordinals = np.arange(numeric["frame_count"], dtype=np.float64)
    seconds = sample_ordinals / float(numeric["sample_rate_hz"])
    de_enveloped = samples * np.exp(target_decay[0] * seconds)
    lag = profile["specialists"]["temporal_evolution"]["lag_samples"]
    temporal_minimum = float(
        profile["specialists"]["temporal_evolution"]["minimum_rms"]
    )
    periodicity = rms(de_enveloped[lag:] - de_enveloped[:-lag]) / max(
        rms(de_enveloped), temporal_minimum
    )

    carrier = np.zeros(numeric["frame_count"], dtype=np.float64)
    for mode in target_modes:
        carrier += float(mode["signed_gain"]) * np.sin(
            2.0
            * math.pi
            * float(mode["frequency_hz"])
            * seconds
        )
    envelope_profile = profile["specialists"]["envelope_order"]
    carrier_floor = float(envelope_profile["carrier_mask_relative_floor"]) * float(
        np.max(np.abs(carrier))
    )
    block_size = envelope_profile["block_size_samples"]
    block_amplitudes: list[float] = []
    for block in range(envelope_profile["block_count"]):
        begin = block * block_size
        end = begin + block_size
        block_carrier = carrier[begin:end]
        mask = np.abs(block_carrier) >= carrier_floor
        if not np.any(mask):
            raise ValidatorMechanicsError("V0 envelope block has no carrier support")
        ratios = np.abs(samples[begin:end][mask] / block_carrier[mask])
        amplitude = float(np.median(ratios))
        if not math.isfinite(amplitude) or amplitude <= 0.0:
            raise ValidatorMechanicsError("V0 envelope amplitude is invalid")
        block_amplitudes.append(amplitude)
    monotonic_tolerance = float(envelope_profile["monotonic_relative_tolerance"])
    monotonic_violations = sum(
        next_value > previous * (1.0 + monotonic_tolerance)
        for previous, next_value in zip(block_amplitudes, block_amplitudes[1:])
    )

    features = {
        "audio_sha256": audio_sha256,
        "cache_key": cache_key,
        "candidate_view_sha256": view_sha256,
        "claim": CLAIM,
        "envelope_order": {
            "block_amplitudes": block_amplitudes,
            "monotonic_violation_count": monotonic_violations,
        },
        "integrity": {
            "audio_identity_exact": audio_identity_exact,
            "clipping_sample_count": clipping_count,
            "dc_over_rms": dc_over_rms,
            "peak": float(np.max(np.abs(samples))),
            "rms": sample_rms,
        },
        "modal": {
            "candidate_mode_count": candidate_mode_count,
            "decay_exact": decay_exact,
            "structure_exact": modal_structure_exact,
            "target_mode_count": target_mode_count,
        },
        "provenance": {
            "actual_parent_present": actual_present,
            "declared_parent_present": declared_present,
            "mismatch": provenance_mismatch,
        },
        "record_id": record_id,
        "retrieval": {"copied_clean_target_ids": copied_target_ids},
        "schema": FEATURE_SCHEMA,
        "target_clean_record_sha256": clean_record_sha256[target_case_id],
        "temporal_evolution": {
            "lag_samples": lag,
            "normalized_periodicity_rms": periodicity,
        },
    }

    reason: str | None = None
    if provenance_mismatch:
        reason = "ProvenanceMismatch"
    elif clipping_count > 0:
        reason = "PcmClipping"
    elif copied_target_ids:
        reason = "RetrievalCopyDetected"
    elif candidate_mode_count < target_mode_count:
        reason = "ModalCoverageCollapsed"
    elif candidate_mode_count != target_mode_count or not modal_structure_exact:
        reason = "ModalParameterMismatch"
    elif not decay_exact:
        reason = "PhysicalDecayMismatch"
    elif periodicity <= float(
        profile["specialists"]["temporal_evolution"]["normalized_rms_max"]
    ):
        reason = "TemporalEvolutionFrozen"
    elif monotonic_violations > 0:
        reason = "EnvelopeOrderMismatch"
    elif (
        not audio_identity_exact
        or sample_rms < minimum_rms
        or dc_over_rms > float(integrity["dc_over_rms_max"])
    ):
        reason = "IntegrityMismatch"
    if reason is None:
        return features, make_decision(record_id, cache_key, "Pass", [])
    return features, make_decision(record_id, cache_key, "Reject", [reason])


def write_bytes(root: Path, relative: str, data: bytes) -> dict[str, Any]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {
        "bytes": len(data),
        "path": relative,
        "sha256": sha256_bytes(data),
    }


def verify_artifacts(root: Path, artifacts: list[dict[str, Any]]) -> None:
    paths = [item["path"] for item in artifacts]
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise ValidatorMechanicsError("V0 artifact index order/uniqueness mismatch")
    for artifact in artifacts:
        data = (root / artifact["path"]).read_bytes()
        if len(data) != artifact["bytes"] or sha256_bytes(data) != artifact["sha256"]:
            raise ValidatorMechanicsError(f"V0 artifact drift: {artifact['path']}")


def expected_decisions(profile: dict[str, Any]) -> dict[str, tuple[str, list[str]]]:
    result = {
        record_id: ("Pass", []) for record_id in profile["truth"]["clean_case_ids"]
    }
    for item in profile["truth"]["mutation_expectations"]:
        result[item["record_id"]] = ("Reject", [item["reason_code"]])
    return result


def deterministic_memory_bound(profile: dict[str, Any]) -> int:
    frames = profile["numeric_profile"]["frame_count"]
    return 8 * frames * 6 + 96 * 1024 * 1024


def build_into(
    staging: Path,
    profile: dict[str, Any],
    profile_data: bytes,
    truth_path: Path,
    started: float,
) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    records, audio, truth_identity = load_truth(truth_path, profile)
    clean_ids = profile["truth"]["clean_case_ids"]
    clean_records = {record_id: records[record_id] for record_id in clean_ids}
    clean_audio = {record_id: audio[record_id] for record_id in clean_ids}
    clean_record_sha256 = {
        record_id: truth_identity["record_sha256"][record_id]
        for record_id in clean_ids
    }
    expected = expected_decisions(profile)
    if set(expected) != set(records):
        raise ValidatorMechanicsError("V0 expected candidate closure mismatch")

    cold: dict[str, tuple[bytes, bytes]] = {}
    summaries: list[dict[str, Any]] = []
    label_invariance: list[dict[str, Any]] = []
    for record_id in sorted(records):
        features, decision = validate_candidate(
            profile,
            records[record_id],
            audio[record_id],
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
        if features is None:
            raise ValidatorMechanicsError(f"official T0 candidate became OOD: {record_id}")
        feature_data = canonical_json(features)
        decision_data = canonical_json(decision)
        if (decision["decision"], decision["reason_codes"]) != expected[record_id]:
            raise ValidatorMechanicsError(
                f"V0 decision matrix mismatch: {record_id}: "
                f"{decision['decision']} {decision['reason_codes']}"
            )
        cold[decision["cache_key"]] = (feature_data, decision_data)

        forged = copy.deepcopy(records[record_id])
        forged.update(
            {
                "expected": {"decision": "Forged", "reason_codes": ["Forged"]},
                "invariant_checks": {"forged": False},
                "kind": "ForgedLabel",
                "mutation": {"operation": {"operation_id": "forged"}},
                "source_case_id": "forged-source",
            }
        )
        original_view = canonical_json(candidate_view(records[record_id], profile))
        forged_view = canonical_json(candidate_view(forged, profile))
        if original_view != forged_view:
            raise ValidatorMechanicsError(f"ignored label leaked: {record_id}")
        forged_features, forged_decision = validate_candidate(
            profile,
            forged,
            audio[record_id],
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
        if (
            forged_features is None
            or canonical_json(forged_features) != feature_data
            or canonical_json(forged_decision) != decision_data
        ):
            raise ValidatorMechanicsError(f"ignored label changed V0 result: {record_id}")
        label_invariance.append({"record_id": record_id, "status": "Pass"})
        summaries.append(
            {
                "cache_key": decision["cache_key"],
                "decision": decision["decision"],
                "reason_codes": decision["reason_codes"],
                "record_id": record_id,
            }
        )

    warm_hits = 0
    for record_id in sorted(records, reverse=True):
        view_sha256 = sha256_bytes(canonical_json(candidate_view(records[record_id], profile)))
        target_id = records[record_id]["target_case_id"]
        key = feature_cache_key(
            profile,
            view_sha256,
            sha256_bytes(audio[record_id]),
            clean_record_sha256[target_id],
        )
        if key not in cold:
            raise ValidatorMechanicsError(f"V0 cold/warm cache miss: {record_id}")
        features, decision = validate_candidate(
            profile,
            records[record_id],
            audio[record_id],
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
        if features is None or cold[key] != (
            canonical_json(features),
            canonical_json(decision),
        ):
            raise ValidatorMechanicsError(f"V0 cold/warm bytes drift: {record_id}")
        warm_hits += 1

    artifacts: list[dict[str, Any]] = []
    publication_summaries: list[dict[str, Any]] = []
    by_record = {item["record_id"]: item for item in summaries}
    for record_id in sorted(records):
        key = by_record[record_id]["cache_key"]
        feature_data, decision_data = cold[key]
        base = f"records/{record_id}"
        feature_ref = write_bytes(staging, f"{base}/features.json", feature_data)
        decision_ref = write_bytes(staging, f"{base}/decision.json", decision_data)
        artifacts.extend([decision_ref, feature_ref])
        publication_summaries.append(
            {
                **by_record[record_id],
                "decision_sha256": decision_ref["sha256"],
                "features_sha256": feature_ref["sha256"],
            }
        )
    artifacts.sort(key=lambda item: item["path"])
    verify_artifacts(staging, artifacts)

    pass_count = sum(item["decision"] == "Pass" for item in summaries)
    reject_count = sum(item["decision"] == "Reject" for item in summaries)
    reason_counts: dict[str, int] = {}
    for item in summaries:
        for reason in item["reason_codes"]:
            reason_counts[reason] = reason_counts.get(reason, 0) + 1
    if (
        pass_count != profile["decision_contract"]["expected_pass_count"]
        or reject_count != profile["decision_contract"]["expected_reject_count"]
        or reason_counts
        != {item["reason_code"]: 1 for item in profile["truth"]["mutation_expectations"]}
    ):
        raise ValidatorMechanicsError("V0 aggregate decision matrix mismatch")

    owner_data = Path(__file__).read_bytes()
    owner_identity = {
        "bytes": len(owner_data),
        "path": "lab/scripts/physical_sound_v32_v0_validator_mechanics_v1.py",
        "sha256": sha256_bytes(owner_data),
    }
    release = {
        "access": ZERO_ACCESS,
        "artifact_index_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "authority": profile["authority"],
        "claim": CLAIM,
        "decision_counts": {"OutOfDomain": 0, "Pass": pass_count, "Reject": reject_count},
        "owner_identity": owner_identity,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "reason_counts": reason_counts,
        "records": publication_summaries,
        "schema": RELEASE_SCHEMA,
        "status": "Pass",
        "truth_identity": {
            "evidence_sha256": truth_identity["evidence_sha256"],
            "report_sha256": truth_identity["report_sha256"],
            "truth_release_sha256": truth_identity["truth_release_sha256"],
        },
    }
    release_ref = write_bytes(
        staging, "validator-mechanics-release.json", canonical_json(release)
    )
    artifacts.append(release_ref)
    artifacts.sort(key=lambda item: item["path"])
    verify_artifacts(staging, artifacts)

    memory_bound = deterministic_memory_bound(profile)
    if memory_bound > profile["resources"]["max_peak_rss_bytes"]:
        raise ValidatorMechanicsError("V0 deterministic memory bound exceeds profile")
    artifact_bytes = sum(item["bytes"] for item in artifacts)
    if artifact_bytes > profile["resources"]["max_output_bytes"]:
        raise ValidatorMechanicsError("V0 artifacts exceed output resource profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise ValidatorMechanicsError("V0 execution exceeded wall resource profile")

    evidence = {
        "access": ZERO_ACCESS,
        "artifact_bytes_before_evidence": artifact_bytes,
        "artifact_count_before_evidence": len(artifacts),
        "artifact_index_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "authority": profile["authority"],
        "cache": {
            "cold_entries": len(cold),
            "reverse_order_warm_hits": warm_hits,
            "warm_cold_bytes_exact": True,
        },
        "claim": CLAIM,
        "dependencies": dependencies,
        "deterministic_memory_bound_bytes": memory_bound,
        "environment": {"numpy": np.__version__, "python": platform.python_version()},
        "label_invariance": label_invariance,
        "owner_identity": owner_identity,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "schema": EVIDENCE_SCHEMA,
        "status": "Pass",
        "truth_identity": release["truth_identity"],
        "validator_release_sha256": release_ref["sha256"],
    }
    evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
    output_bytes_before_report = sum(
        item.stat().st_size for item in staging.rglob("*") if item.is_file()
    )
    report = {
        "access": ZERO_ACCESS,
        "authority": profile["authority"],
        "claim": CLAIM,
        "decision": "V0_VALIDATOR_MECHANICS_PASS",
        "evidence_sha256": evidence_ref["sha256"],
        "gates": {
            "byte_exact_repeat_required": True,
            "cache_cold_warm_and_order": "16/16 Pass",
            "decision_matrix": "9 Pass / 7 Reject",
            "ignored_label_invariance": "16/16 Pass",
            "mutation_reason_coverage": "7/7 Pass",
            "resource_envelope": "Pass",
            "zero_real_signal_model_network": "Pass",
        },
        "next_authorized_stage": "V32-M0-bounded-correction-protocol",
        "output_bytes_before_report": output_bytes_before_report,
        "owner_sha256": owner_identity["sha256"],
        "profile_sha256": sha256_bytes(profile_data),
        "record_count": len(records),
        "schema": REPORT_SCHEMA,
        "status": "Pass",
        "validator_release_sha256": release_ref["sha256"],
    }
    write_bytes(staging, "report.json", canonical_json(report))
    files = sorted(item for item in staging.rglob("*") if item.is_file())
    if len(files) != profile["publication"]["expected_file_count"]:
        raise ValidatorMechanicsError("V0 final file count drift")
    final_bytes = sum(item.stat().st_size for item in files)
    if final_bytes > profile["resources"]["max_output_bytes"]:
        raise ValidatorMechanicsError("V0 final publication exceeds output profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise ValidatorMechanicsError("V0 final execution exceeded wall profile")
    return report


def run(
    profile_path: Path, truth_path: Path, output_path: Path
) -> dict[str, Any]:
    started = time.monotonic()
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(
        tempfile.mkdtemp(prefix=".nextengine-v32-v0-", dir=str(output.parent))
    )
    try:
        report = build_into(staging, profile, profile_data, truth_path, started)
        os.replace(staging, output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--truth", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.profile, arguments.truth, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable external CLI boundary
        print(f"physical-sound-v32-v0: ContractReject: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
