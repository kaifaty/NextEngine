#!/usr/bin/env python3
"""Run the preregistered V44 G0B1b target-cap/normalization audit."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

import numpy as np
import scipy

import physical_sound_v41_b0_grouped_baselines_v1 as b0
import physical_sound_v41_c0_disclosed_corpus_v1 as c0

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1b-audit-profile.v1"
PROTOCOL_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1b-target-audit-protocol.v1"
)
AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1b-target-audit.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1b-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1b-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v44-g0b1b-target-audit.v1.json"
PROTOCOL_PATH = "lab/profiles/physical-sound-v44-g0b1b-target-audit-protocol.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v44_g0b1b_target_audit_v1.py"
C0_OWNER_PATH = "lab/scripts/physical_sound_v41_c0_disclosed_corpus_v1.py"
B0_OWNER_PATH = "lab/scripts/physical_sound_v41_b0_grouped_baselines_v1.py"

CLAIM = (
    "PREREGISTERED_IETEASY_TARGET_CAP_AND_NORMALIZATION_AUDIT_ONLY / "
    "NO_CORPUS_ROLE_PSEL_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_"
    "AUTHORITY"
)
G0B1A_DECISION = "G0B1A_IETEASY_PAYLOAD_REPEATABLE_TARGET_EXTRACTOR_AUDIT_REQUIRED"
HASH_LENGTH = 64
MAX_JSON_BYTES = 4 * 1024 * 1024
MAX_FILE_BYTES = 4 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "corpus_materialization_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "target_policy_audit_authority": True,
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


class TargetAuditError(RuntimeError):
    """G0B1b cannot publish a trustworthy target-policy audit."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--protocol", required=True, type=Path)
    parser.add_argument("--g0b1a-root", required=True, type=Path)
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
        raise TargetAuditError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TargetAuditError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise TargetAuditError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise TargetAuditError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise TargetAuditError(f"{context} must be an integer >= {minimum}")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise TargetAuditError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_FILE_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise TargetAuditError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise TargetAuditError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context, MAX_JSON_BYTES)
    try:
        value = require_dict(json.loads(data), context)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise TargetAuditError(f"{context} is not valid UTF-8 JSON") from error
    if data != canonical_json(value):
        raise TargetAuditError(f"{context} must be canonical JSON")
    return data, value


def validate_binding_bytes(data: bytes, binding: dict[str, Any], context: str) -> None:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise TargetAuditError(f"{context} binding fields changed")
    require_string(binding["path"], f"{context} path")
    if len(data) != require_int(binding["bytes"], f"{context} bytes", 1):
        raise TargetAuditError(f"{context} byte count changed")
    if sha256_bytes(data) != require_hash(binding["sha256"], f"{context} hash"):
        raise TargetAuditError(f"{context} hash changed")


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
            raise TargetAuditError("dependency path is invalid or duplicated")
        seen.add(path_text)
        data = read_regular(root / relative, f"dependency {path_text}")
        validate_binding_bytes(data, binding, f"dependency {path_text}")
    if not {OWNER_PATH, PROTOCOL_PATH, C0_OWNER_PATH, B0_OWNER_PATH}.issubset(seen):
        raise TargetAuditError("required dependency binding is missing")


def validate_environment(profile: dict[str, Any]) -> Path:
    environment = require_dict(profile.get("environment"), "environment")
    if set(environment) != {"ffmpeg", "numpy_version", "scipy_version"}:
        raise TargetAuditError("environment fields changed")
    binding = require_dict(environment["ffmpeg"], "ffmpeg binding")
    path = Path(require_string(binding.get("path"), "ffmpeg path"))
    validate_binding_bytes(read_regular(path, "ffmpeg"), binding, "ffmpeg")
    if environment["numpy_version"] != np.__version__:
        raise TargetAuditError("NumPy version changed")
    if environment["scipy_version"] != scipy.__version__:
        raise TargetAuditError("SciPy version changed")
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
        raise TargetAuditError("execution profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM:
        raise TargetAuditError("execution profile schema or claim changed")
    if profile["authority"] != AUTHORITY or profile["access_policy"] != ACCESS_POLICY:
        raise TargetAuditError("execution authority or access policy changed")
    expected = require_dict(profile["expected_input"], "expected input")
    if set(expected) != {"physical_parents", "selected_payload_bytes", "waveforms"}:
        raise TargetAuditError("expected input fields changed")
    for key, value in expected.items():
        require_int(value, f"expected input {key}", 1)
    validate_dependencies(profile)
    return validate_environment(profile)


def validate_protocol(protocol: dict[str, Any]) -> None:
    required = {
        "access_policy",
        "algorithm_bindings",
        "authority",
        "claim",
        "decision_policy",
        "input_bindings",
        "normalization_audit",
        "profile_id",
        "reference_policy",
        "schema",
    }
    if set(protocol) != required:
        raise TargetAuditError("protocol fields changed")
    if protocol["schema"] != PROTOCOL_SCHEMA or protocol["claim"] != CLAIM:
        raise TargetAuditError("protocol schema or claim changed")
    if protocol["authority"] != AUTHORITY or protocol["access_policy"] != ACCESS_POLICY:
        raise TargetAuditError("protocol authority or access policy changed")
    bindings = require_list(protocol["algorithm_bindings"], "algorithm bindings")
    if {row.get("path") for row in bindings} != {C0_OWNER_PATH, B0_OWNER_PATH}:
        raise TargetAuditError("protocol algorithm bindings changed")
    root = repository_root()
    for raw in bindings:
        binding = require_dict(raw, "algorithm binding")
        validate_binding_bytes(
            read_regular(root / binding["path"], "bound algorithm"),
            binding,
            "bound algorithm",
        )
    policy = require_dict(protocol["decision_policy"], "decision policy")
    if policy.get("candidate_mode_caps_ascending") != [12, 24, 48, 96]:
        raise TargetAuditError("candidate mode caps changed")
    if policy.get("mode_cap_choice") != "smallest_candidate_passing_every_record":
        raise TargetAuditError("mode-cap choice rule changed")
    reference = require_dict(protocol["reference_policy"], "reference policy")
    maximum = require_int(reference.get("maximum_modes"), "reference cap", 1)
    possible = require_int(
        reference.get("maximum_possible_peaks_from_frequency_distance"),
        "maximum possible peaks",
        1,
    )
    if maximum <= possible or reference.get("reference_must_not_saturate") is not True:
        raise TargetAuditError("reference cap is not demonstrably non-saturating")
    normalization = require_dict(protocol["normalization_audit"], "normalization audit")
    if normalization.get("positive_scale_factors_ppm") != [500000, 2000000]:
        raise TargetAuditError("normalization scale mutations changed")


def load_protocol(
    profile: dict[str, Any], protocol_path: Path
) -> tuple[bytes, dict[str, Any]]:
    data, protocol = read_canonical_json(protocol_path, "target audit protocol")
    binding = require_dict(profile["protocol"], "protocol binding")
    validate_binding_bytes(data, binding, "target audit protocol")
    if binding["path"] != PROTOCOL_PATH:
        raise TargetAuditError("protocol path changed")
    validate_protocol(protocol)
    return data, protocol


def load_external_input(
    root: Path, binding: dict[str, Any], context: str
) -> dict[str, Any]:
    if set(binding) != {
        "bytes",
        "external_relative_path",
        "schema",
        "sha256",
    }:
        raise TargetAuditError(f"{context} binding fields changed")
    relative = Path(
        require_string(binding["external_relative_path"], f"{context} path")
    )
    if relative.is_absolute() or ".." in relative.parts:
        raise TargetAuditError(f"{context} path escapes its root")
    data, document = read_canonical_json(root / relative, context)
    if len(data) != binding["bytes"] or sha256_bytes(data) != binding["sha256"]:
        raise TargetAuditError(f"{context} binding changed")
    if document.get("schema") != binding["schema"]:
        raise TargetAuditError(f"{context} schema changed")
    return document


def load_g0b1a(
    root: Path, protocol: dict[str, Any], expected: dict[str, Any]
) -> tuple[list[dict[str, Any]], dict[str, dict[str, Any]]]:
    if root.is_symlink() or not root.is_dir():
        raise TargetAuditError("G0B1a root must be a non-symlink directory")
    bindings = require_dict(protocol["input_bindings"], "input bindings")
    selection = load_external_input(
        root,
        require_dict(bindings["g0b1a_selection"], "selection binding"),
        "selection",
    )
    media = load_external_input(
        root, require_dict(bindings["g0b1a_media_probe"], "media binding"), "media"
    )
    report = load_external_input(
        root, require_dict(bindings["g0b1a_report"], "report binding"), "report"
    )
    if report.get("decision") != G0B1A_DECISION:
        raise TargetAuditError("G0B1a decision changed")
    if report.get("gates", {}).get("target_extraction_completed") is not True:
        raise TargetAuditError("G0B1a target extraction did not complete")
    records = [require_dict(row, "selected record") for row in selection["records"]]
    media_rows = [require_dict(row, "media row") for row in media["records"]]
    by_record = {row.get("record_id"): row for row in media_rows}
    if len(records) != expected["waveforms"] or len(by_record) != len(media_rows):
        raise TargetAuditError("G0B1a record count or identity changed")
    parents = {row["lineage"]["physical_parent_id"] for row in records}
    payload_bytes = sum(row["record"]["bytes"] for row in records)
    if (
        len(parents) != expected["physical_parents"]
        or payload_bytes != expected["selected_payload_bytes"]
    ):
        raise TargetAuditError("G0B1a selected parent/payload count changed")
    if {row["record"]["record_id"] for row in records} != set(by_record):
        raise TargetAuditError("selection/media record IDs disagree")
    return records, by_record


def extract_with_cap(samples: np.ndarray, cap: int) -> dict[str, Any]:
    previous = c0.FEATURE_POLICY["maximum_modes"]
    try:
        c0.FEATURE_POLICY["maximum_modes"] = cap
        return c0.extract_acoustic_target(samples)
    except c0.CorpusError as error:
        raise TargetAuditError(f"target extraction failed: {error}") from error
    finally:
        c0.FEATURE_POLICY["maximum_modes"] = previous


def target_hash(target: dict[str, Any]) -> str:
    return sha256_bytes(canonical_json(target))


def modal_histogram(target: dict[str, Any]) -> np.ndarray:
    try:
        vector, _ = b0.feature_vector(target)
    except b0.BaselineError as error:
        raise TargetAuditError(f"target vectorization failed: {error}") from error
    return vector[72:96]


def ppm(value: float) -> int:
    if not math.isfinite(value):
        raise TargetAuditError("audit metric is non-finite")
    return max(0, min(1_000_000, round(value * 1_000_000)))


def compare_targets(
    candidate: dict[str, Any], reference: dict[str, Any]
) -> dict[str, int]:
    candidate_modes = require_list(candidate.get("modes"), "candidate modes")
    reference_modes = require_list(reference.get("modes"), "reference modes")
    reference_amplitude = sum(
        10.0 ** (float(row["relative_level_db"]) / 20.0) for row in reference_modes
    )
    candidate_amplitude = sum(
        10.0 ** (float(row["relative_level_db"]) / 20.0) for row in candidate_modes
    )
    retention = (
        candidate_amplitude / reference_amplitude if reference_amplitude > 0 else 1.0
    )
    left = modal_histogram(candidate)
    right = modal_histogram(reference)
    denominator = float(np.linalg.norm(left) * np.linalg.norm(right))
    cosine = float(np.dot(left, right) / denominator) if denominator > 0 else 1.0
    normalized_l1 = float(
        np.sum(np.abs(left - right)) / max(float(np.sum(np.abs(right))), 1e-12)
    )
    return {
        "modal_histogram_cosine_similarity_ppm": ppm(cosine),
        "modal_histogram_normalized_l1_ppm": ppm(normalized_l1),
        "retained_linear_modal_amplitude_ppm": ppm(retention),
    }


def candidate_passes(metrics: dict[str, int], policy: dict[str, Any]) -> bool:
    return (
        metrics["retained_linear_modal_amplitude_ppm"]
        >= policy["minimum_retained_linear_modal_amplitude_ppm"]
        and metrics["modal_histogram_cosine_similarity_ppm"]
        >= policy["minimum_modal_histogram_cosine_similarity_ppm"]
        and metrics["modal_histogram_normalized_l1_ppm"]
        <= policy["maximum_modal_histogram_normalized_l1_ppm"]
    )


def audit_record(
    selected: dict[str, Any],
    prior: dict[str, Any],
    payload_root: Path,
    ffmpeg_path: Path,
    protocol: dict[str, Any],
) -> tuple[dict[str, Any], int, int]:
    record = require_dict(selected["record"], "selected waveform")
    digest = require_hash(record.get("sha256"), "payload hash")
    payload = read_regular(payload_root / digest, f"payload {digest}")
    if len(payload) != record["bytes"] or sha256_bytes(payload) != digest:
        raise TargetAuditError("publisher payload byte/hash binding changed")
    try:
        decoded = c0.decode_audio(payload, 48000, ffmpeg_path)
        canonical, normalization = c0.canonical_segment(decoded)
    except c0.CorpusError as error:
        raise TargetAuditError(f"canonical decode failed: {error}") from error
    baseline = extract_with_cap(canonical, 12)
    pcm_hash = sha256_bytes(c0.wav_pcm16_bytes(canonical))
    if (
        pcm_hash != prior["canonical_pcm"]["sha256"]
        or target_hash(baseline) != prior["target_probe"]["sha256"]
        or normalization != prior["normalization"]
    ):
        raise TargetAuditError("G0B1a canonical PCM/target identity changed")

    normalization_policy = protocol["normalization_audit"]
    scale_rows = []
    for scale_ppm in normalization_policy["positive_scale_factors_ppm"]:
        scaled, _ = c0.canonical_segment(decoded * (scale_ppm / 1_000_000.0))
        scaled_target = extract_with_cap(scaled, 12)
        scale_rows.append(
            {
                "canonical_pcm_identical": sha256_bytes(c0.wav_pcm16_bytes(scaled))
                == pcm_hash,
                "scale_ppm": scale_ppm,
                "target_identical": target_hash(scaled_target) == target_hash(baseline),
            }
        )
    clipped = int(np.count_nonzero(np.abs(canonical) >= 1.0))

    reference_cap = protocol["reference_policy"]["maximum_modes"]
    reference = extract_with_cap(canonical, reference_cap)
    reference_count = len(reference["modes"])
    policy = protocol["decision_policy"]
    candidates = []
    for cap in policy["candidate_mode_caps_ascending"]:
        target = baseline if cap == 12 else extract_with_cap(canonical, cap)
        metrics = compare_targets(target, reference)
        candidates.append(
            {
                "cap": cap,
                "metrics": metrics,
                "mode_count": len(target["modes"]),
                "passes": candidate_passes(metrics, policy),
                "target_sha256": target_hash(target),
            }
        )
    row = {
        "candidate_caps": candidates,
        "lineage": selected["lineage"],
        "normalization": {
            "post_normalization_clipped_samples": clipped,
            "scale_mutations": scale_rows,
        },
        "record_id": record["record_id"],
        "reference": {
            "cap": reference_cap,
            "mode_count": reference_count,
            "saturated": reference_count == reference_cap,
            "target_sha256": target_hash(reference),
        },
        "source_payload_sha256": digest,
    }
    return row, len(decoded), len(scale_rows) + 1 + len(candidates)


def terminal_decision(
    records: list[dict[str, Any]], protocol: dict[str, Any]
) -> tuple[str, int | None, dict[str, Any]]:
    normalization = protocol["normalization_audit"]
    normalization_pass = all(
        row["normalization"]["post_normalization_clipped_samples"] == 0
        and all(
            mutation["canonical_pcm_identical"]
            == normalization["canonical_pcm_identity_required"]
            and mutation["target_identical"]
            == normalization["target_identity_required"]
            for mutation in row["normalization"]["scale_mutations"]
        )
        for row in records
    )
    reference_pass = all(not row["reference"]["saturated"] for row in records)
    policy = protocol["decision_policy"]
    passing_caps = [
        cap
        for cap in policy["candidate_mode_caps_ascending"]
        if all(
            next(item for item in row["candidate_caps"] if item["cap"] == cap)["passes"]
            for row in records
        )
    ]
    chosen = passing_caps[0] if passing_caps else None
    if not normalization_pass:
        decision = policy["normalization_failure_decision"]
        chosen = None
    elif not reference_pass:
        decision = policy["reference_cap_saturation_decision"]
        chosen = None
    elif chosen is None:
        decision = policy["fallback_decision"]
    elif chosen == 12:
        decision = "Legacy12ModeTargetAdmitted"
    else:
        decision = "VersionedModeCapTargetRequired"
    gates = {
        "candidate_caps_passing_all_records": passing_caps,
        "normalization_passed": normalization_pass,
        "reference_non_saturating": reference_pass,
        "target_policy_resolved": chosen is not None,
    }
    return decision, chosen, gates


def build_outputs(
    profile: dict[str, Any],
    protocol: dict[str, Any],
    records: list[dict[str, Any]],
    decoded_samples: int,
    target_extractions: int,
) -> dict[str, Any]:
    decision, chosen, gates = terminal_decision(records, protocol)
    caps = protocol["decision_policy"]["candidate_mode_caps_ascending"]
    aggregates = {}
    for cap in caps:
        items = [
            next(item for item in row["candidate_caps"] if item["cap"] == cap)
            for row in records
        ]
        aggregates[str(cap)] = {
            "maximum_modal_histogram_normalized_l1_ppm": max(
                item["metrics"]["modal_histogram_normalized_l1_ppm"] for item in items
            ),
            "minimum_modal_histogram_cosine_similarity_ppm": min(
                item["metrics"]["modal_histogram_cosine_similarity_ppm"]
                for item in items
            ),
            "minimum_retained_linear_modal_amplitude_ppm": min(
                item["metrics"]["retained_linear_modal_amplitude_ppm"] for item in items
            ),
            "records_passing": sum(item["passes"] for item in items),
        }
    measured = {
        "chosen_mode_cap": chosen,
        "decoded_sample_values": decoded_samples,
        "normalization_scale_mutations": sum(
            len(row["normalization"]["scale_mutations"]) for row in records
        ),
        "physical_parents": len(
            {row["lineage"]["physical_parent_id"] for row in records}
        ),
        "reference_mode_count_maximum": max(
            row["reference"]["mode_count"] for row in records
        ),
        "reference_mode_count_minimum": min(
            row["reference"]["mode_count"] for row in records
        ),
        "selected_payload_bytes": profile["expected_input"]["selected_payload_bytes"],
        "target_extractions": target_extractions,
        "waveforms": len(records),
    }
    audit = {
        "aggregates_by_cap": aggregates,
        "claim": CLAIM,
        "decision": decision,
        "decision_gates": gates,
        "records": records,
        "schema": AUDIT_SCHEMA,
    }
    report = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": decision,
        "gates": {
            **gates,
            "corpus_materialization_authorized": False,
            "psel_pack_eligible": False,
            "runtime_consumer_allowed": False,
            "training_allowed": False,
        },
        "measured": measured,
        "next_action": (
            "freeze_the_resolved_target_policy_in_a_15_record_disclosed_corpus_"
            "successor_then_continue_second_independent_steel_project_search"
            if chosen is not None
            else "preregister_a_noise_robust_modal_distribution_successor_before_"
            "corpus_materialization_while_continuing_second_independent_steel_"
            "project_search"
        ),
        "schema": REPORT_SCHEMA,
    }
    access = {
        "claim": CLAIM,
        "counters": {
            "audio_payload_bytes_read": profile["expected_input"][
                "selected_payload_bytes"
            ],
            "audio_payload_files_read": len(records),
            "candidate_values_read": 0,
            "decoded_sample_values": decoded_samples,
            "model_values_read": 0,
            "network_requests": 0,
            "protected_values_read": 0,
            "target_extractions": target_extractions,
            "validator_values_read": 0,
        },
        "schema": ACCESS_SCHEMA,
    }
    return {"access-ledger.json": access, "audit.json": audit, "report.json": report}


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
        raise TargetAuditError("output must be outside the repository")
    if output.is_symlink():
        raise TargetAuditError("output must not be a symlink")
    if output.exists():
        raise TargetAuditError("output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)


def publish(
    output: Path,
    outputs: dict[str, Any],
    profile_bytes: bytes,
    protocol_bytes: bytes,
) -> None:
    prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, value in sorted(outputs.items()):
            (staging / name).write_bytes(canonical_json(value))
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
    payload_root: Path,
    output: Path,
) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "execution profile")
    ffmpeg_path = validate_profile(profile)
    protocol_bytes, protocol = load_protocol(profile, protocol_path)
    selected, prior_by_record = load_g0b1a(
        g0b1a_root, protocol, profile["expected_input"]
    )
    if (
        payload_root.is_symlink()
        or not payload_root.is_dir()
        or is_within(payload_root.resolve(), repository_root().resolve())
    ):
        raise TargetAuditError("payload root must be an external non-symlink directory")
    records = []
    decoded_samples = 0
    target_extractions = 0
    for item in selected:
        record_id = item["record"]["record_id"]
        row, samples, extractions = audit_record(
            item, prior_by_record[record_id], payload_root, ffmpeg_path, protocol
        )
        records.append(row)
        decoded_samples += samples
        target_extractions += extractions
    outputs = build_outputs(
        profile, protocol, records, decoded_samples, target_extractions
    )
    for counter in FORBIDDEN_COUNTERS:
        if outputs["access-ledger.json"]["counters"][counter] != 0:
            raise TargetAuditError("forbidden access counter is nonzero")
    publish(output, outputs, profile_bytes, protocol_bytes)


def main() -> int:
    arguments = parse_arguments()
    try:
        run(
            arguments.profile,
            arguments.protocol,
            arguments.g0b1a_root,
            arguments.payload_root,
            arguments.output,
        )
    except TargetAuditError as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
