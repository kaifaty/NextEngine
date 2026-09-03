#!/usr/bin/env python3
"""Verify the frozen V44 G0B1a IETeasy payload/target probe."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from collections import Counter
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP
from pathlib import Path
from typing import Any

import numpy as np
import scipy

import physical_sound_v41_c0_disclosed_corpus_v1 as c0

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1a-probe-profile.v1"
SELECTION_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1a-selection.v1"
PROSPECTIVE_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b-prospective-parents.v1"
)
PROSPECTIVE_CLAIM = (
    "PROSPECTIVE_PARENT_AND_PUBLISHER_WAVEFORM_BINDING_ONLY / "
    "NO_PAYLOAD_ROLE_TARGET_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_"
    "AUTHORITY"
)
SELECTION_OUTPUT_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1a-selected-records.v1"
)
MEDIA_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1a-media-probe.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1a-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1a-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v44-g0b1a-iet-payload-probe.v1.json"
SELECTION_PATH = "lab/profiles/physical-sound-v44-g0b1a-iet-payload-selection.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v44_g0b1a_iet_payload_probe_v1.py"
C0_OWNER_PATH = "lab/scripts/physical_sound_v41_c0_disclosed_corpus_v1.py"

CLAIM = (
    "SIGNAL_BLIND_IETEASY_REPETITION_1_PAYLOAD_AND_TARGET_FEASIBILITY_ONLY / "
    "NO_CORPUS_ROLE_CREDIT_PSEL_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_"
    "RUNTIME_AUTHORITY"
)
DECISION = "G0B1A_IETEASY_PAYLOAD_REPEATABLE_TARGET_EXTRACTOR_AUDIT_REQUIRED"
HASH_LENGTH = 64
MAX_JSON_BYTES = 4 * 1024 * 1024
MAX_DEPENDENCY_BYTES = 4 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "corpus_materialization_probe_authority": True,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}
ACCESS_POLICY = {
    "audio_decode_allowed_after_hash_verification": True,
    "feature_probe_allowed_after_selection_freeze": True,
    "model_access_allowed": False,
    "network_allowed_by_owner": False,
    "outputs_external": True,
    "protected_access_allowed": False,
    "selection_signal_blind": True,
}
FORBIDDEN_COUNTERS = (
    "candidate_values_read",
    "model_values_read",
    "network_requests_by_owner",
    "protected_values_read",
    "validator_values_read",
)


class PayloadProbeError(RuntimeError):
    """G0B1a cannot publish a trustworthy payload/target probe."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--selection-profile", required=True, type=Path)
    parser.add_argument("--prospective-manifest", required=True, type=Path)
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
        raise PayloadProbeError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise PayloadProbeError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise PayloadProbeError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise PayloadProbeError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise PayloadProbeError(f"{context} must be an integer >= {minimum}")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise PayloadProbeError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise PayloadProbeError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise PayloadProbeError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context, MAX_JSON_BYTES)
    try:
        value = require_dict(json.loads(data), context)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise PayloadProbeError(f"{context} is not valid UTF-8 JSON") from error
    if data != canonical_json(value):
        raise PayloadProbeError(f"{context} must be canonical JSON")
    return data, value


def validate_binding_bytes(data: bytes, binding: dict[str, Any], context: str) -> None:
    expected_keys = {"bytes", "path", "sha256"}
    if set(binding) != expected_keys:
        raise PayloadProbeError(f"{context} binding fields changed")
    require_string(binding["path"], f"{context} path")
    if len(data) != require_int(binding["bytes"], f"{context} bytes", 1):
        raise PayloadProbeError(f"{context} byte count changed")
    if sha256_bytes(data) != require_hash(binding["sha256"], f"{context} hash"):
        raise PayloadProbeError(f"{context} hash changed")


def validate_dependency_bindings(profile: dict[str, Any]) -> None:
    root = repository_root()
    seen: set[str] = set()
    for index, raw in enumerate(
        require_list(profile.get("dependency_bindings"), "dependency bindings")
    ):
        binding = require_dict(raw, f"dependency {index}")
        path_text = require_string(binding.get("path"), f"dependency {index} path")
        relative = Path(path_text)
        if relative.is_absolute() or ".." in relative.parts or path_text in seen:
            raise PayloadProbeError("dependency path is invalid or duplicated")
        seen.add(path_text)
        data = read_regular(
            root / relative, f"dependency {path_text}", MAX_DEPENDENCY_BYTES
        )
        validate_binding_bytes(data, binding, f"dependency {path_text}")
    required = {OWNER_PATH, C0_OWNER_PATH, SELECTION_PATH}
    if not required.issubset(seen):
        raise PayloadProbeError("required dependency binding is missing")


def validate_environment(profile: dict[str, Any]) -> tuple[Path, Path]:
    environment = require_dict(profile.get("environment"), "environment")
    if set(environment) != {"ffmpeg", "ffprobe", "numpy_version", "scipy_version"}:
        raise PayloadProbeError("environment fields changed")
    binaries = []
    for name in ("ffmpeg", "ffprobe"):
        binding = require_dict(environment[name], f"{name} binding")
        path = Path(require_string(binding.get("path"), f"{name} path"))
        data = read_regular(path, name, MAX_DEPENDENCY_BYTES)
        validate_binding_bytes(data, binding, name)
        binaries.append(path)
    if environment["numpy_version"] != np.__version__:
        raise PayloadProbeError("NumPy version changed")
    if environment["scipy_version"] != scipy.__version__:
        raise PayloadProbeError("SciPy version changed")
    return binaries[0], binaries[1]


def validate_execution_profile(profile: dict[str, Any]) -> tuple[Path, Path]:
    required = {
        "access_policy",
        "authority",
        "c0_policies",
        "claim",
        "dependency_bindings",
        "environment",
        "expected_result",
        "profile_id",
        "schema",
        "selection_profile",
    }
    if set(profile) != required:
        raise PayloadProbeError("execution profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM:
        raise PayloadProbeError("execution profile schema or claim changed")
    if profile["authority"] != AUTHORITY or profile["access_policy"] != ACCESS_POLICY:
        raise PayloadProbeError("execution authority or access policy changed")
    policies = require_dict(profile["c0_policies"], "C0 policies")
    if policies != {"corpus": c0.CORPUS_POLICY, "features": c0.FEATURE_POLICY}:
        raise PayloadProbeError("C0 target extraction policy changed")
    expected = require_dict(profile["expected_result"], "expected result")
    if set(expected) != {
        "decoded_waveforms",
        "physical_parents",
        "project_revisions",
        "selected_payload_bytes",
        "steel_parents",
        "target_probes",
    }:
        raise PayloadProbeError("expected result fields changed")
    for name in expected:
        require_int(expected[name], f"expected result {name}", 1)
    validate_dependency_bindings(profile)
    return validate_environment(profile)


def validate_selection_profile(selection: dict[str, Any]) -> None:
    required = {
        "access_policy",
        "authority",
        "claim",
        "expected_selection",
        "input_binding",
        "payload_policy",
        "profile_id",
        "project",
        "role_intent",
        "schema",
        "selection_policy",
        "target_probe_policy",
    }
    if set(selection) != required:
        raise PayloadProbeError("selection profile fields changed")
    if selection["schema"] != SELECTION_SCHEMA or selection["claim"] != CLAIM:
        raise PayloadProbeError("selection profile schema or claim changed")
    if (
        selection["authority"] != AUTHORITY
        or selection["access_policy"] != ACCESS_POLICY
    ):
        raise PayloadProbeError("selection authority or access policy changed")
    policy = require_dict(selection["selection_policy"], "selection policy")
    expected_policy = {
        "parent_order": "lineage.physical_parent_id_ascending",
        "parent_scope": "all_prospective_catalogue_eligible_parents",
        "records_per_parent": 1,
        "repetition_index": 1,
        "waveform_tie_break": "record_id_ascending",
    }
    if policy != expected_policy:
        raise PayloadProbeError("signal-blind selection policy changed")
    role = require_dict(selection["role_intent"], "role intent")
    if role.get("role") != "generator_train" or role.get("credit_allowed_after_probe"):
        raise PayloadProbeError("role intent changed")
    target = require_dict(selection["target_probe_policy"], "target probe policy")
    if target != {
        "canonical_segment_owner": "physical_sound_v41_c0_disclosed_corpus_v1.py",
        "emit_audio_or_feature_values": False,
        "extract_acoustic_target": True,
        "hash_canonical_pcm_and_target": True,
        "target_use": "feasibility_only",
    }:
        raise PayloadProbeError("target probe policy changed")


def load_bound_selection(
    profile: dict[str, Any], selection_path: Path
) -> tuple[bytes, dict[str, Any]]:
    data, selection = read_canonical_json(selection_path, "selection profile")
    binding = require_dict(profile["selection_profile"], "selection profile binding")
    validate_binding_bytes(data, binding, "selection profile")
    if binding["path"] != SELECTION_PATH:
        raise PayloadProbeError("selection profile path changed")
    validate_selection_profile(selection)
    return data, selection


def load_bound_manifest(
    selection: dict[str, Any], manifest_path: Path
) -> dict[str, Any]:
    data, manifest = read_canonical_json(manifest_path, "prospective manifest")
    binding = require_dict(selection["input_binding"], "prospective input binding")
    if set(binding) != {"bytes", "external_relative_path", "schema", "sha256"}:
        raise PayloadProbeError("prospective input binding fields changed")
    if len(data) != require_int(binding["bytes"], "prospective bytes", 1):
        raise PayloadProbeError("prospective manifest byte count changed")
    if sha256_bytes(data) != require_hash(binding["sha256"], "prospective hash"):
        raise PayloadProbeError("prospective manifest hash changed")
    if (
        manifest.get("schema") != binding["schema"]
        or binding["schema"] != PROSPECTIVE_SCHEMA
    ):
        raise PayloadProbeError("prospective manifest schema changed")
    if manifest.get("claim") != PROSPECTIVE_CLAIM:
        raise PayloadProbeError("prospective manifest claim changed")
    return manifest


def select_records(
    selection: dict[str, Any], manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    parents = require_list(manifest.get("parents"), "prospective parents")
    project = require_dict(selection["project"], "project")
    selected: list[dict[str, Any]] = []
    parent_ids: set[str] = set()
    for raw_parent in parents:
        parent = require_dict(raw_parent, "prospective parent")
        if parent.get("prospective_catalogue_eligible") is not True:
            raise PayloadProbeError("selection scope contains an ineligible parent")
        lineage = require_dict(parent.get("lineage"), "parent lineage")
        for key in ("project_id", "revision_id", "source_component_id"):
            if lineage.get(key) != project.get(key):
                raise PayloadProbeError("parent lineage changed")
        parent_id = require_string(lineage.get("physical_parent_id"), "parent ID")
        if parent_id in parent_ids:
            raise PayloadProbeError("physical parent is duplicated")
        parent_ids.add(parent_id)
        candidates = sorted(
            (
                require_dict(item, "waveform")
                for item in require_list(parent.get("waveforms"), "waveforms")
                if item.get("repetition_index") == 1
            ),
            key=lambda item: require_string(item.get("record_id"), "record ID"),
        )
        if len(candidates) != 1:
            raise PayloadProbeError("each parent must have exactly one repetition 1")
        waveform = candidates[0]
        selected.append(
            {
                "descriptor": require_dict(parent.get("descriptor"), "descriptor"),
                "lineage": lineage,
                "material_label": require_string(
                    require_list(
                        parent["descriptor"].get("values"), "descriptor values"
                    )[0],
                    "material label",
                ),
                "record": waveform,
                "role_intent": selection["role_intent"]["role"],
                "sample_id": require_string(parent.get("sample_id"), "sample ID"),
            }
        )
    selected.sort(key=lambda item: item["lineage"]["physical_parent_id"])
    expected = require_dict(selection["expected_selection"], "expected selection")
    payload_policy = require_dict(selection["payload_policy"], "payload policy")
    if len(selected) != expected.get("physical_parents") or len(
        selected
    ) != expected.get("selected_waveforms"):
        raise PayloadProbeError("selected record count changed")
    total_bytes = 0
    materials: Counter[str] = Counter()
    for item in selected:
        waveform = item["record"]
        if waveform.get("media_type") != payload_policy.get("allowed_media_type"):
            raise PayloadProbeError("selected media type changed")
        size = require_int(waveform.get("bytes"), "payload bytes", 1)
        if size > payload_policy.get("maximum_payload_bytes_each"):
            raise PayloadProbeError("selected payload exceeds per-file limit")
        require_hash(waveform.get("sha256"), "payload hash")
        require_string(waveform.get("source_url"), "source URL")
        total_bytes += size
        materials[item["material_label"]] += 1
    if total_bytes != expected.get("selected_payload_bytes"):
        raise PayloadProbeError("selected payload byte total changed")
    if total_bytes > payload_policy.get("maximum_selected_payload_bytes"):
        raise PayloadProbeError("selected payload exceeds aggregate limit")
    if dict(sorted(materials.items())) != expected.get("material_counts"):
        raise PayloadProbeError("selected material counts changed")
    return selected


def probe_media(
    path: Path, ffprobe_path: Path, policy: dict[str, Any]
) -> dict[str, Any]:
    command = [
        str(ffprobe_path),
        "-v",
        "error",
        "-select_streams",
        "a",
        "-show_entries",
        "stream=index,codec_name,codec_type,sample_rate,channels,duration",
        "-show_entries",
        "format=format_name,duration,size",
        "-of",
        "json",
        str(path),
    ]
    try:
        completed = subprocess.run(
            command, check=False, capture_output=True, timeout=30
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise PayloadProbeError(f"ffprobe execution failed: {error}") from error
    if completed.returncode != 0 or completed.stderr:
        detail = completed.stderr.decode(errors="replace")[:400]
        raise PayloadProbeError(f"ffprobe rejected payload: {detail}")
    try:
        document = require_dict(json.loads(completed.stdout), "ffprobe output")
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise PayloadProbeError("ffprobe output is not valid JSON") from error
    streams = require_list(document.get("streams"), "ffprobe streams")
    if len(streams) != 1:
        raise PayloadProbeError("payload must contain exactly one audio stream")
    stream = require_dict(streams[0], "ffprobe audio stream")
    format_value = require_dict(document.get("format"), "ffprobe format")
    if stream.get("codec_type") != "audio" or stream.get("codec_name") != policy.get(
        "codec_name"
    ):
        raise PayloadProbeError("payload codec changed")
    if format_value.get("format_name") != policy.get("codec_name"):
        raise PayloadProbeError("payload container changed")
    try:
        sample_rate = int(stream.get("sample_rate"))
        channels = int(stream.get("channels"))
        duration = Decimal(require_string(stream.get("duration"), "stream duration"))
        format_duration = Decimal(
            require_string(format_value.get("duration"), "format duration")
        )
        format_bytes = int(format_value.get("size"))
    except (InvalidOperation, TypeError, ValueError) as error:
        raise PayloadProbeError("ffprobe numeric field is invalid") from error
    if sample_rate != policy.get("sample_rate_hz") or channels != policy.get(
        "channel_count"
    ):
        raise PayloadProbeError("payload channel/rate contract changed")
    if duration != format_duration:
        raise PayloadProbeError("stream and format durations disagree")
    duration_ms = int((duration * 1000).to_integral_value(rounding=ROUND_HALF_UP))
    if not (
        policy.get("minimum_decoded_duration_milliseconds")
        <= duration_ms
        <= policy.get("maximum_decoded_duration_milliseconds")
    ):
        raise PayloadProbeError("payload duration is outside the frozen envelope")
    return {
        "channel_count": channels,
        "codec_name": stream["codec_name"],
        "duration_milliseconds": duration_ms,
        "format_name": format_value["format_name"],
        "reported_bytes": format_bytes,
        "sample_rate_hz": sample_rate,
    }


def probe_record(
    item: dict[str, Any],
    payload_root: Path,
    ffmpeg_path: Path,
    ffprobe_path: Path,
    selection: dict[str, Any],
) -> tuple[dict[str, Any], int]:
    waveform = item["record"]
    digest = require_hash(waveform["sha256"], "payload hash")
    path = payload_root / digest
    maximum = require_int(
        selection["payload_policy"]["maximum_payload_bytes_each"],
        "maximum payload bytes",
        1,
    )
    payload = read_regular(path, f"payload {digest}", maximum)
    if len(payload) != waveform["bytes"] or sha256_bytes(payload) != digest:
        raise PayloadProbeError("publisher payload byte/hash binding changed")
    media = probe_media(path, ffprobe_path, selection["payload_policy"])
    if media["reported_bytes"] != len(payload):
        raise PayloadProbeError("ffprobe payload byte count changed")
    try:
        decoded = c0.decode_audio(payload, media["sample_rate_hz"], ffmpeg_path)
        canonical, normalization = c0.canonical_segment(decoded)
        target = c0.extract_acoustic_target(canonical)
        pcm_bytes = c0.wav_pcm16_bytes(canonical)
        target_bytes = canonical_json(target)
    except c0.CorpusError as error:
        raise PayloadProbeError(f"C0 target probe failed: {error}") from error
    decoded_duration_ms = int(
        (Decimal(len(decoded)) * 1000 / media["sample_rate_hz"]).to_integral_value(
            rounding=ROUND_HALF_UP
        )
    )
    if abs(decoded_duration_ms - media["duration_milliseconds"]) > 100:
        raise PayloadProbeError("decoded duration disagrees with ffprobe")
    return (
        {
            "canonical_pcm": {
                "bytes": len(pcm_bytes),
                "sha256": sha256_bytes(pcm_bytes),
            },
            "decoded_duration_milliseconds": decoded_duration_ms,
            "decoded_sample_count": len(decoded),
            "lineage": item["lineage"],
            "material_label": item["material_label"],
            "media": media,
            "normalization": normalization,
            "record_id": waveform["record_id"],
            "role_intent": item["role_intent"],
            "sample_id": item["sample_id"],
            "source_payload": {
                "bytes": len(payload),
                "sha256": digest,
            },
            "target_probe": {
                "bytes": len(target_bytes),
                "mode_count": target["uncertainty"]["mode_count"],
                "schema": target["schema"],
                "sha256": sha256_bytes(target_bytes),
            },
        },
        len(decoded),
    )


def build_outputs(
    profile: dict[str, Any],
    selection: dict[str, Any],
    selected: list[dict[str, Any]],
    probes: list[dict[str, Any]],
    decoded_samples: int,
) -> dict[str, Any]:
    expected = profile["expected_result"]
    measured = {
        "complete_source_coverage_probes": sum(
            row["normalization"]["source_onset_sample_48k"]
            >= c0.CORPUS_POLICY["pretrigger_samples"]
            and row["normalization"]["source_onset_sample_48k"]
            - c0.CORPUS_POLICY["pretrigger_samples"]
            + c0.CORPUS_POLICY["segment_samples"]
            <= row["decoded_sample_count"]
            for row in probes
        ),
        "decoded_peak_over_unity_records": sum(
            row["normalization"]["raw_peak"] > 1.0 for row in probes
        ),
        "decoded_sample_values": decoded_samples,
        "decoded_waveforms": len(probes),
        "modal_cap_saturated_target_probes": sum(
            row["target_probe"]["mode_count"] == c0.FEATURE_POLICY["maximum_modes"]
            for row in probes
        ),
        "physical_parents": len(
            {row["lineage"]["physical_parent_id"] for row in probes}
        ),
        "project_revisions": len({row["lineage"]["revision_id"] for row in probes}),
        "selected_payload_bytes": sum(row["source_payload"]["bytes"] for row in probes),
        "steel_parents": len(
            {
                row["lineage"]["physical_parent_id"]
                for row in probes
                if row["material_label"] == "Steel"
            }
        ),
        "target_probes": len(probes),
    }
    for key, value in expected.items():
        if measured[key] != value:
            raise PayloadProbeError(f"measured {key} changed")
    access = {
        "claim": CLAIM,
        "counters": {
            "acoustic_targets_extracted": len(probes),
            "audio_payload_bytes_read": measured["selected_payload_bytes"],
            "audio_payload_files_read": len(probes),
            "candidate_values_read": 0,
            "decoded_sample_values": decoded_samples,
            "decoded_waveforms": len(probes),
            "model_values_read": 0,
            "network_requests_by_owner": 0,
            "protected_values_read": 0,
            "validator_values_read": 0,
        },
        "schema": ACCESS_SCHEMA,
    }
    report = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": DECISION,
        "gates": {
            "corpus_materialization_eligible": False,
            "media_and_onset_probe_passed": True,
            "payload_hash_verification_passed": True,
            "psel_pack_eligible": False,
            "runtime_consumer_allowed": False,
            "target_extraction_completed": True,
            "target_extractor_reuse_without_audit_allowed": False,
            "training_allowed": False,
        },
        "measured": measured,
        "next_action": "audit_the_saturated_12_mode_cap_and_over_unity_decode_normalization_before_freezing_the_15_record_disclosed_corpus_successor_while_continuing_metadata_first_search_for_a_second_independent_steel_project",
        "schema": REPORT_SCHEMA,
    }
    selected_output = {
        "claim": CLAIM,
        "records": selected,
        "schema": SELECTION_OUTPUT_SCHEMA,
        "selection_policy": selection["selection_policy"],
    }
    media_output = {
        "claim": CLAIM,
        "c0_policies": profile["c0_policies"],
        "records": probes,
        "schema": MEDIA_SCHEMA,
    }
    return {
        "access-ledger.json": access,
        "media-probe.json": media_output,
        "report.json": report,
        "selection.json": selected_output,
    }


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
        raise PayloadProbeError("output must be outside the repository")
    if output.is_symlink():
        raise PayloadProbeError("output must not be a symlink")
    if output.exists():
        raise PayloadProbeError("output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)


def publish(
    output: Path,
    outputs: dict[str, Any],
    profile_bytes: bytes,
    selection_bytes: bytes,
) -> None:
    prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, value in sorted(outputs.items()):
            (staging / name).write_bytes(canonical_json(value))
        (staging / "profile.json").write_bytes(profile_bytes)
        (staging / "selection-profile.json").write_bytes(selection_bytes)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def run(
    profile_path: Path,
    selection_path: Path,
    manifest_path: Path,
    payload_root: Path,
    output: Path,
) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "execution profile")
    ffmpeg_path, ffprobe_path = validate_execution_profile(profile)
    selection_bytes, selection = load_bound_selection(profile, selection_path)
    manifest = load_bound_manifest(selection, manifest_path)
    selected = select_records(selection, manifest)
    if (
        payload_root.is_symlink()
        or not payload_root.is_dir()
        or is_within(payload_root.resolve(), repository_root().resolve())
    ):
        raise PayloadProbeError(
            "payload root must be an external non-symlink directory"
        )
    probes: list[dict[str, Any]] = []
    decoded_samples = 0
    for item in selected:
        probe, count = probe_record(
            item, payload_root, ffmpeg_path, ffprobe_path, selection
        )
        probes.append(probe)
        decoded_samples += count
    outputs = build_outputs(profile, selection, selected, probes, decoded_samples)
    for counter in FORBIDDEN_COUNTERS:
        if outputs["access-ledger.json"]["counters"][counter] != 0:
            raise PayloadProbeError("forbidden access counter is nonzero")
    publish(output, outputs, profile_bytes, selection_bytes)


def main() -> int:
    arguments = parse_arguments()
    try:
        run(
            arguments.profile,
            arguments.selection_profile,
            arguments.prospective_manifest,
            arguments.payload_root,
            arguments.output,
        )
    except PayloadProbeError as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
