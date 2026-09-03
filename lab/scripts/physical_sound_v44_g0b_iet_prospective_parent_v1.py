#!/usr/bin/env python3
"""Freeze the V44 G0B IETeasy prospective-parent/source increment."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b-iet-profile.v1"
CONTRACT_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b-prospective-parent-contract.v1"
)
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-role-projection.v1"
PARENT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b-prospective-parents.v1"
SOURCE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b-source-card.v1"
POWER_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b-power.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v44-g0b-iet-prospective-parent.v1.json"
CONTRACT_PATH = (
    "lab/profiles/physical-sound-v44-g0b-prospective-parent-contract.v1.json"
)
OWNER_PATH = "lab/scripts/physical_sound_v44_g0b_iet_prospective_parent_v1.py"
F1_PROFILE_PATH = "lab/profiles/physical-sound-v39-f1-source-frontier.v1.json"

DECISION = "G0B_IETEASY_PROSPECTIVE_INCREMENT_REPEATABLE_PSEL_BLOCKED"
CLAIM = (
    "PROSPECTIVE_PARENT_AND_PUBLISHER_WAVEFORM_BINDING_ONLY / "
    "NO_PAYLOAD_ROLE_TARGET_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
PROJECT_ID = "mendeley-data--10.17632-srfp7x6wxm"
REVISION_ID = "mendeley-data--10.17632-srfp7x6wxm--v1"
SOURCE_COMPONENT_ID = "source-component.ieteasy-mendeley-srfp7x6wxm-v1"
AUDIO_ROOT_ID = "7013e754-791b-435d-9837-b7c8ed32b3f8"
PARENT_PREFIX = f"{REVISION_ID}--sample-"
HASH_LENGTH = 64
MAX_METADATA_BYTES = 16 * 1024 * 1024
EXPECTED_AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}
EXPECTED_ACCESS_POLICY = {
    "audio_decode_allowed": False,
    "audio_payload_access_allowed": False,
    "feature_access_allowed": False,
    "model_access_allowed": False,
    "network_allowed_at_build": False,
    "outputs_external": True,
    "protected_access_allowed": False,
}
EXPECTED_RESULT = {
    "audio_bytes_declared": 17981616,
    "descriptor_complete_parents": 0,
    "existing_c0r_parents": 64,
    "glass_pack_parents": 0,
    "independent_project_revisions": 1,
    "prospective_parents": 15,
    "publisher_hashed_waveforms": 150,
    "steel_pack_parents": 5,
    "wood_pack_parents": 0,
}
FORBIDDEN_COUNTERS = (
    "audio_files_decoded",
    "audio_payload_bytes_read",
    "candidate_values_read",
    "feature_values_read",
    "model_values_read",
    "network_requests",
    "pcm_sample_values_decoded",
    "protected_values_read",
)
UUID_RE = re.compile(r"^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$")
FILE_RE = re.compile(r"^(.+) \(([1-9]|10)\)\.mp3$")


class ProspectiveParentError(RuntimeError):
    """G0B cannot publish a trustworthy prospective-parent increment."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--raw-root", required=True, type=Path)
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
        raise ProspectiveParentError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ProspectiveParentError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise ProspectiveParentError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise ProspectiveParentError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise ProspectiveParentError(f"{context} must be an integer >= {minimum}")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise ProspectiveParentError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_METADATA_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ProspectiveParentError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise ProspectiveParentError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json(data: bytes, context: str) -> Any:
    try:
        return json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ProspectiveParentError(f"{context} is not valid UTF-8 JSON") from error


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    value = require_dict(parse_json(data, context), context)
    if data != canonical_json(value):
        raise ProspectiveParentError(f"{context} must be canonical JSON")
    return data, value


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise ProspectiveParentError(f"{context} binding fields changed")
    expected_path = require_string(binding["path"], f"{context} path")
    relative = Path(expected_path)
    if relative.is_absolute() or ".." in relative.parts:
        raise ProspectiveParentError(f"{context} path escapes its root")
    data = read_regular(path / relative, context)
    if len(data) != require_int(
        binding["bytes"], f"{context} bytes", 1
    ) or sha256_bytes(data) != require_hash(binding["sha256"], f"{context} hash"):
        raise ProspectiveParentError(f"{context} binding mismatch")
    return data


def validate_dependency_bindings(profile: dict[str, Any]) -> None:
    root = repository_root()
    seen: set[str] = set()
    for index, raw_binding in enumerate(
        require_list(profile.get("dependency_bindings"), "dependency bindings")
    ):
        binding = require_dict(raw_binding, f"dependency {index}")
        path = require_string(binding.get("path"), f"dependency {index} path")
        if path in seen:
            raise ProspectiveParentError("duplicate dependency binding")
        seen.add(path)
        check_binding(root, binding, f"dependency {path}")
    required = {CONTRACT_PATH, F1_PROFILE_PATH, OWNER_PATH}
    if not required.issubset(seen):
        raise ProspectiveParentError("required dependency binding is missing")


def validate_profile(profile: dict[str, Any]) -> None:
    required_keys = {
        "access_policy",
        "authority",
        "claim",
        "contract",
        "dependency_bindings",
        "expected_result",
        "input_bindings",
        "profile_id",
        "project",
        "raw_inputs",
        "sample_folder_bridge",
        "schema",
    }
    if set(profile) != required_keys:
        raise ProspectiveParentError("profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM:
        raise ProspectiveParentError("profile schema or claim changed")
    if profile["authority"] != EXPECTED_AUTHORITY:
        raise ProspectiveParentError("profile authority changed")
    if profile["access_policy"] != EXPECTED_ACCESS_POLICY:
        raise ProspectiveParentError("profile access policy changed")
    if profile["expected_result"] != EXPECTED_RESULT:
        raise ProspectiveParentError("profile expected result changed")
    contract = require_dict(profile["contract"], "contract identity")
    if contract.get("path") != CONTRACT_PATH:
        raise ProspectiveParentError("contract path changed")
    project = require_dict(profile["project"], "project")
    expected_project = {
        "audio_root_folder_id": AUDIO_ROOT_ID,
        "doi": "10.17632/srfp7x6wxm.1",
        "license_expression": "CC-BY-4.0",
        "project_id": PROJECT_ID,
        "publisher_id": "mendeley-data",
        "revision_id": REVISION_ID,
        "source_component_id": SOURCE_COMPONENT_ID,
    }
    if project != expected_project:
        raise ProspectiveParentError("project identity changed")
    bridges = require_list(profile["sample_folder_bridge"], "sample folder bridge")
    if len(bridges) != EXPECTED_RESULT["prospective_parents"]:
        raise ProspectiveParentError("sample folder bridge count changed")
    sample_ids: set[str] = set()
    folder_ids: set[str] = set()
    for index, raw_bridge in enumerate(bridges):
        bridge = require_dict(raw_bridge, f"sample bridge {index}")
        if set(bridge) != {
            "filename_stem",
            "folder_id",
            "folder_label",
            "normalized_material_label",
            "sample_id",
        }:
            raise ProspectiveParentError("sample folder bridge fields changed")
        sample_id = require_string(bridge["sample_id"], "sample id")
        folder_id = require_string(bridge["folder_id"], "folder id")
        if not UUID_RE.fullmatch(folder_id):
            raise ProspectiveParentError("sample folder id is not a UUID")
        if sample_id in sample_ids or folder_id in folder_ids:
            raise ProspectiveParentError("duplicate sample or folder bridge")
        sample_ids.add(sample_id)
        folder_ids.add(folder_id)
        if bridge["normalized_material_label"] not in {
            "Aluminium",
            "Metal",
            "Plastic",
            "Steel",
        }:
            raise ProspectiveParentError("unsupported normalized material")
    raw_inputs = require_dict(profile["raw_inputs"], "raw inputs")
    if set(raw_inputs) != {
        "audio_folders",
        "data_article",
        "dataset_page",
        "folder_index",
        "hardware_article",
    }:
        raise ProspectiveParentError("raw input fields changed")
    folder_inputs = require_list(raw_inputs["audio_folders"], "folder inputs")
    if len(folder_inputs) != len(bridges):
        raise ProspectiveParentError("folder input count changed")
    input_ids = set()
    for raw_folder in folder_inputs:
        folder = require_dict(raw_folder, "folder input")
        if set(folder) != {"binding", "folder_id"}:
            raise ProspectiveParentError("folder input fields changed")
        folder_id = require_string(folder["folder_id"], "folder input id")
        if folder_id in input_ids:
            raise ProspectiveParentError("duplicate folder input")
        input_ids.add(folder_id)
        require_dict(folder["binding"], "folder input binding")
    if input_ids != folder_ids:
        raise ProspectiveParentError("folder inputs do not match sample bridge")
    validate_dependency_bindings(profile)


def validate_contract(profile: dict[str, Any]) -> dict[str, Any]:
    root = repository_root()
    binding = require_dict(profile["contract"], "contract identity")
    data = check_binding(root, binding, "contract")
    contract = require_dict(parse_json(data, "contract"), "contract")
    if data != canonical_json(contract):
        raise ProspectiveParentError("contract must be canonical JSON")
    if contract.get("schema") != CONTRACT_SCHEMA or contract.get("claim") != CLAIM:
        raise ProspectiveParentError("contract schema or claim changed")
    minimum = require_dict(
        contract.get("independence_policy"), "independence policy"
    ).get("minimum_independent_projects_per_psel_pack")
    if minimum != 2:
        raise ProspectiveParentError("PSEL project minimum changed")
    return contract


def read_projection(
    corpus_root: Path, binding: dict[str, Any], context: str
) -> dict[str, Any]:
    data = check_binding(corpus_root, binding, context)
    projection = require_dict(parse_json(data, context), context)
    if projection.get("schema") != PROJECTION_SCHEMA:
        raise ProspectiveParentError(f"{context} schema changed")
    require_list(projection.get("rows"), f"{context} rows")
    return projection


def current_corpus_power(
    profile: dict[str, Any], corpus_root: Path
) -> tuple[set[str], set[str]]:
    bindings = require_dict(profile["input_bindings"], "input bindings")
    if set(bindings) != {"development_projection", "train_projection"}:
        raise ProspectiveParentError("input binding fields changed")
    parents: set[str] = set()
    components: set[str] = set()
    for name in ("train_projection", "development_projection"):
        projection = read_projection(
            corpus_root, require_dict(bindings[name], name), name
        )
        for index, raw_row in enumerate(projection["rows"]):
            row = require_dict(raw_row, f"{name} row {index}")
            parents.add(require_string(row.get("physical_parent_id"), "parent id"))
            components.add(
                require_string(row.get("family_component_id"), "component id")
            )
    if len(parents) != EXPECTED_RESULT["existing_c0r_parents"]:
        raise ProspectiveParentError("current C0R parent count changed")
    if SOURCE_COMPONENT_ID in components:
        raise ProspectiveParentError("IETeasy source component already exists in C0R")
    return parents, components


def bioc_text(data: bytes, context: str) -> str:
    collections = require_list(parse_json(data, context), context)
    passages: list[str] = []
    for collection in collections:
        for document in require_list(
            require_dict(collection, context).get("documents"), f"{context} documents"
        ):
            for passage in require_list(
                require_dict(document, context).get("passages"),
                f"{context} passages",
            ):
                text = require_dict(passage, context).get("text")
                if isinstance(text, str) and text:
                    passages.append(text)
    if not passages:
        raise ProspectiveParentError(f"{context} has no text passages")
    return "\n".join(passages)


def validate_primary_metadata(raw: dict[str, bytes]) -> None:
    try:
        page = raw["dataset_page"].decode("utf-8")
    except UnicodeDecodeError as error:
        raise ProspectiveParentError("dataset page is not UTF-8") from error
    for marker in (
        "10.17632/srfp7x6wxm.1",
        "CC BY 4.0",
        "original audio file",
        "dimensions, weight",
    ):
        if marker not in page:
            raise ProspectiveParentError(f"dataset page marker missing: {marker}")
    article = bioc_text(raw["data_article"], "data article")
    for marker in (
        "fifteen different materials",
        "ten measurements for each sample",
        "suspending the sample on PA6 strings",
        "Length (cm)",
        "Mass (g)",
        "sampling rate of 48,000",
    ):
        if marker not in article:
            raise ProspectiveParentError(f"data article marker missing: {marker}")
    hardware = bioc_text(raw["hardware_article"], "hardware article")
    for marker in (
        "rectangular samples",
        "Free-Free boundary conditions",
        "two taut nylon strings",
        "mallet is released",
        "microphone positioned over the sample",
    ):
        if marker not in hardware:
            raise ProspectiveParentError(f"hardware article marker missing: {marker}")


def read_raw_inputs(
    profile: dict[str, Any], raw_root: Path
) -> tuple[dict[str, bytes], dict[str, bytes]]:
    raw_inputs = require_dict(profile["raw_inputs"], "raw inputs")
    metadata: dict[str, bytes] = {}
    for name in ("data_article", "dataset_page", "folder_index", "hardware_article"):
        metadata[name] = check_binding(
            raw_root, require_dict(raw_inputs[name], name), f"raw input {name}"
        )
    folder_files: dict[str, bytes] = {}
    for raw_folder in require_list(raw_inputs["audio_folders"], "folder inputs"):
        folder = require_dict(raw_folder, "folder input")
        folder_id = require_string(folder["folder_id"], "folder input id")
        folder_files[folder_id] = check_binding(
            raw_root,
            require_dict(folder["binding"], "folder input binding"),
            f"audio folder {folder_id}",
        )
    return metadata, folder_files


def load_f1_samples() -> list[dict[str, Any]]:
    _, profile = read_canonical_json(
        repository_root() / F1_PROFILE_PATH, "F1 source-frontier profile"
    )
    increment = require_dict(profile.get("increment"), "F1 increment")
    if increment.get("project_revision_id") != REVISION_ID:
        raise ProspectiveParentError("F1 IETeasy revision changed")
    samples = require_list(increment.get("samples"), "F1 IETeasy samples")
    if len(samples) != EXPECTED_RESULT["prospective_parents"]:
        raise ProspectiveParentError("F1 IETeasy sample count changed")
    return [require_dict(sample, "F1 IETeasy sample") for sample in samples]


def decimal_micrometres(value: Any, context: str) -> int:
    try:
        converted = Decimal(require_string(value, context)) * Decimal(10000)
    except InvalidOperation as error:
        raise ProspectiveParentError(f"{context} is not decimal centimetres") from error
    integral = converted.to_integral_value()
    if converted != integral or integral <= 0:
        raise ProspectiveParentError(f"{context} does not convert to micrometres")
    return int(integral)


def validate_folder_index(data: bytes, bridges: list[dict[str, Any]]) -> None:
    rows = require_list(parse_json(data, "folder index"), "folder index")
    audio_roots = [
        row
        for row in rows
        if require_dict(row, "folder row").get("name") == "Audio data"
        and "parent_id" not in row
    ]
    if len(audio_roots) != 1 or audio_roots[0].get("id") != AUDIO_ROOT_ID:
        raise ProspectiveParentError("audio root identity changed")
    actual = {
        require_string(row.get("id"), "audio child id"): require_string(
            row.get("name"), "audio child name"
        )
        for row in rows
        if require_dict(row, "folder row").get("parent_id") == AUDIO_ROOT_ID
    }
    expected = {bridge["folder_id"]: bridge["folder_label"] for bridge in bridges}
    if actual != expected:
        raise ProspectiveParentError("audio folder identity set changed")


def waveform_rows(data: bytes, bridge: dict[str, Any]) -> list[dict[str, Any]]:
    raw_rows = require_list(parse_json(data, "audio file list"), "audio file list")
    if len(raw_rows) != 10:
        raise ProspectiveParentError("each IETeasy sample must expose ten waveforms")
    expected_repetitions = set(range(1, 11))
    repetitions: set[int] = set()
    result: list[dict[str, Any]] = []
    folder_id = bridge["folder_id"]
    filename_stem = bridge["filename_stem"]
    for raw_row in raw_rows:
        row = require_dict(raw_row, "audio file row")
        filename = require_string(row.get("filename"), "audio filename")
        match = FILE_RE.fullmatch(filename)
        if not match or match.group(1) != filename_stem:
            raise ProspectiveParentError("audio filename does not match folder label")
        repetition = int(match.group(2))
        if repetition in repetitions:
            raise ProspectiveParentError("duplicate audio repetition")
        repetitions.add(repetition)
        if row.get("folder_id") != folder_id or row.get("status") != "COMPLETED":
            raise ProspectiveParentError("audio row folder or status changed")
        file_id = require_string(row.get("id"), "publisher file id")
        if not UUID_RE.fullmatch(file_id):
            raise ProspectiveParentError("publisher file id is not a UUID")
        details = require_dict(row.get("content_details"), "audio content details")
        media_type = require_string(details.get("content_type"), "audio media type")
        if media_type != "audio/mpeg":
            raise ProspectiveParentError("audio media type changed")
        size = require_int(details.get("size"), "audio bytes", 1)
        if row.get("size") != size:
            raise ProspectiveParentError("outer and inner audio sizes disagree")
        digest = require_hash(details.get("sha256_hash"), "publisher audio hash")
        source_url = require_string(details.get("download_url"), "audio source URL")
        parsed = urlparse(source_url)
        expected_path = (
            f"/public-files/datasets/srfp7x6wxm/files/{file_id}/file_downloaded"
        )
        if (
            parsed.scheme != "https"
            or parsed.netloc != "data.mendeley.com"
            or parsed.path != expected_path
        ):
            raise ProspectiveParentError("audio source URL changed")
        result.append(
            {
                "bytes": size,
                "media_type": media_type,
                "publisher_file_id": file_id,
                "record_id": f"identified:ieteasy-{bridge['sample_id']}--impact-{repetition:03d}",
                "repetition_index": repetition,
                "sha256": digest,
                "source_url": source_url,
            }
        )
    if repetitions != expected_repetitions:
        raise ProspectiveParentError("audio repetition set changed")
    return sorted(result, key=lambda row: row["repetition_index"])


def descriptor(sample: dict[str, Any], material: str) -> dict[str, Any]:
    extents = [
        decimal_micrometres(sample.get("length_cm"), "sample length"),
        decimal_micrometres(sample.get("depth_cm"), "sample depth"),
        decimal_micrometres(sample.get("thickness_cm"), "sample thickness"),
    ]
    mass_mg = require_int(sample.get("mass_g"), "sample mass", 1) * 1000
    return {
        "confidence_ppm": [
            1000000,
            950000,
            1000000,
            0,
            900000,
            900000,
            1000000,
            1000000,
            0,
        ],
        "missing_reasons": [
            None,
            None,
            None,
            "not_applicable",
            None,
            None,
            None,
            None,
            "not_published",
        ],
        "observed_mask": [True, True, True, False, True, True, True, True, False],
        "values": [
            material,
            "thin_plate",
            extents,
            None,
            "none",
            "none",
            mass_mg,
            "suspended",
            None,
        ],
    }


def normalized_material(source_label: str) -> str:
    label = source_label.casefold()
    if "steel" in label:
        return "Steel"
    if "aluminium" in label:
        return "Aluminium"
    if any(token in label for token in ("nylon", "polizene", "pom-c", "teflon")):
        return "Plastic"
    return "Metal"


def build_parents(
    profile: dict[str, Any], folder_files: dict[str, bytes], current_parents: set[str]
) -> list[dict[str, Any]]:
    bridges = {
        bridge["sample_id"]: bridge
        for bridge in require_list(profile["sample_folder_bridge"], "sample bridge")
    }
    parents: list[dict[str, Any]] = []
    file_ids: set[str] = set()
    hashes: set[str] = set()
    for sample in load_f1_samples():
        sample_id = require_string(sample.get("sample_id"), "sample id")
        bridge = require_dict(bridges.get(sample_id), f"bridge for {sample_id}")
        source_label = require_string(sample.get("source_label"), "sample source label")
        if bridge["normalized_material_label"] != normalized_material(source_label):
            raise ProspectiveParentError("sample material normalization changed")
        parent_id = f"{PARENT_PREFIX}{sample_id}"
        if parent_id in current_parents:
            raise ProspectiveParentError("prospective parent collides with C0R")
        waveforms = waveform_rows(folder_files[bridge["folder_id"]], bridge)
        for waveform in waveforms:
            if waveform["publisher_file_id"] in file_ids:
                raise ProspectiveParentError("publisher file appears under two parents")
            if waveform["sha256"] in hashes:
                raise ProspectiveParentError("waveform hash appears under two records")
            file_ids.add(waveform["publisher_file_id"])
            hashes.add(waveform["sha256"])
        parents.append(
            {
                "descriptor": descriptor(sample, bridge["normalized_material_label"]),
                "lineage": {
                    "physical_parent_id": parent_id,
                    "project_id": PROJECT_ID,
                    "publisher_id": "mendeley-data",
                    "revision_id": REVISION_ID,
                    "source_component_id": SOURCE_COMPONENT_ID,
                    "source_parent_label": source_label,
                },
                "prospective_catalogue_eligible": True,
                "sample_id": sample_id,
                "source_folder_id": bridge["folder_id"],
                "source_folder_label": bridge["folder_label"],
                "waveforms": waveforms,
            }
        )
    if set(bridges) != {parent["sample_id"] for parent in parents}:
        raise ProspectiveParentError("sample bridge contains an unused row")
    return sorted(parents, key=lambda parent: parent["sample_id"])


def build_outputs(
    profile: dict[str, Any],
    contract: dict[str, Any],
    parents: list[dict[str, Any]],
    component_count: int,
) -> dict[str, Any]:
    by_material: dict[str, int] = {}
    waveform_count = 0
    audio_bytes = 0
    for parent in parents:
        material = parent["descriptor"]["values"][0]
        by_material[material] = by_material.get(material, 0) + 1
        waveform_count += len(parent["waveforms"])
        audio_bytes += sum(waveform["bytes"] for waveform in parent["waveforms"])
    target_counts = {
        material: by_material.get(material, 0)
        for material in ("Glass", "Steel", "Wood")
    }
    target_projects = {
        material: int(count > 0) for material, count in target_counts.items()
    }
    psel_minimum = contract["independence_policy"][
        "minimum_independent_projects_per_psel_pack"
    ]
    eligible_packs = sorted(
        material
        for material, projects in target_projects.items()
        if projects >= psel_minimum
    )
    measured = {
        "audio_bytes_declared": audio_bytes,
        "descriptor_complete_parents": 0,
        "existing_c0r_parents": EXPECTED_RESULT["existing_c0r_parents"],
        "glass_pack_parents": target_counts["Glass"],
        "independent_project_revisions": 1,
        "prospective_parents": len(parents),
        "publisher_hashed_waveforms": waveform_count,
        "steel_pack_parents": target_counts["Steel"],
        "wood_pack_parents": target_counts["Wood"],
    }
    if measured != EXPECTED_RESULT:
        raise ProspectiveParentError(f"measured result changed: {measured}")
    source_card = {
        "capability": "identified_real_recording_metadata_with_publisher_hashes",
        "claim": CLAIM,
        "corpus_materialization_eligible": False,
        "license_expression": "CC-BY-4.0",
        "prospective_catalogue_eligible": True,
        "psel_pack_eligible": False,
        "reason_materialization_blocked": "audio_payloads_not_fetched_or_verified_and_roles_not_frozen",
        "reason_psel_blocked": "no_target_pack_has_two_independent_project_revisions",
        "revision_id": REVISION_ID,
        "schema": SOURCE_SCHEMA,
        "source_component_id": SOURCE_COMPONENT_ID,
        "source_urls": [
            "https://data.mendeley.com/datasets/srfp7x6wxm/1",
            "https://doi.org/10.1016/j.dib.2021.107503",
            "https://doi.org/10.1016/j.ohx.2021.e00228",
        ],
    }
    parent_manifest = {
        "claim": CLAIM,
        "field_order": contract["descriptor_policy"]["field_order"],
        "parents": parents,
        "schema": PARENT_SCHEMA,
    }
    power = {
        "best_case_supported_parent_deficit_after_materialization": 34,
        "by_normalized_material": dict(sorted(by_material.items())),
        "current_supported_parent_deficit": 49,
        "current_supported_parent_floor": 105,
        "current_supported_parents": 56,
        "eligible_psel_packs": eligible_packs,
        "prospective_increment_is_not_current_support": True,
        "psel_minimum_independent_projects_per_pack": psel_minimum,
        "schema": POWER_SCHEMA,
        "source_component_count_after_increment": component_count + 1,
        "target_pack_parent_counts": target_counts,
        "target_pack_project_counts": target_projects,
    }
    access = {
        "claim": CLAIM,
        "counters": {
            **{counter: 0 for counter in FORBIDDEN_COUNTERS},
            "corpus_projection_files_read": 2,
            "publisher_file_hashes_read": waveform_count,
            "publisher_file_metadata_bytes_read": sum(len(data) for data in []),
            "repository_dependency_files_verified": len(profile["dependency_bindings"]),
            "source_metadata_files_read": 19,
        },
        "schema": ACCESS_SCHEMA,
    }
    report = {
        "authority": profile["authority"],
        "claim": CLAIM,
        "decision": DECISION,
        "gates": {
            "candidate_training_allowed": False,
            "corpus_materialization_eligible": False,
            "prospective_catalogue_eligible": True,
            "psel_power_sufficient": False,
            "runtime_consumer_allowed": False,
        },
        "measured": measured,
        "next_action": "fetch_and_hash_verify_a_bounded_ieteasy_payload_subset_then_freeze_a_disclosed_role_and_target_extraction_successor_while_searching_a_second_independent_steel_project",
        "schema": REPORT_SCHEMA,
    }
    return {
        "access-ledger.json": access,
        "contract.json": contract,
        "power.json": power,
        "prospective-parents.json": parent_manifest,
        "report.json": report,
        "source-card.json": source_card,
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
        raise ProspectiveParentError("output must be outside the repository")
    if output.is_symlink():
        raise ProspectiveParentError("output must not be a symlink")
    if output.exists():
        raise ProspectiveParentError("output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)


def publish(output: Path, outputs: dict[str, Any], profile_bytes: bytes) -> None:
    prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, value in sorted(outputs.items()):
            (staging / name).write_bytes(canonical_json(value))
        (staging / "profile.json").write_bytes(profile_bytes)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def run(profile_path: Path, corpus_root: Path, raw_root: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "G0B profile")
    validate_profile(profile)
    contract = validate_contract(profile)
    current_parents, components = current_corpus_power(profile, corpus_root)
    metadata, folder_files = read_raw_inputs(profile, raw_root)
    validate_primary_metadata(metadata)
    bridges = [
        require_dict(bridge, "sample bridge")
        for bridge in profile["sample_folder_bridge"]
    ]
    validate_folder_index(metadata["folder_index"], bridges)
    parents = build_parents(profile, folder_files, current_parents)
    outputs = build_outputs(profile, contract, parents, len(components))
    outputs["access-ledger.json"]["counters"]["publisher_file_metadata_bytes_read"] = (
        sum(len(data) for data in folder_files.values())
    )
    publish(output, outputs, profile_bytes)


def main() -> int:
    arguments = parse_arguments()
    try:
        run(
            arguments.profile,
            arguments.corpus_root,
            arguments.raw_root,
            arguments.output,
        )
    except ProspectiveParentError as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
