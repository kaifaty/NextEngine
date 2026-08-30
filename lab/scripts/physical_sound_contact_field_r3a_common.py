#!/usr/bin/env python3
"""Shared frozen boundary for the R3A contact-position representation gate."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
from typing import Any

import numpy as np

PROFILE_ID = "realimpact-blue-bowl-canonical-contact-r3a-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-corpus.manifest.v1"
PREFLIGHT_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-source-preflight.report.v1"
)
EXTRACTION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-contact-extraction.report.v1"
)
ORACLE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-representation-oracle.report.v1"
)
IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_common.py",
    "preflight": "physical_sound_contact_field_r3a_preflight.py",
    "extraction": "physical_sound_contact_field_r3a_extract.py",
    "oracle": "physical_sound_contact_field_r3a_oracle.py",
}

SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 230_215
ROW_COUNT = 3_000
LISTENERS_PER_IMPACT = 600
IMPACT_COUNT = 5
CANONICAL_ANGLE_DEGREES = 0
CANONICAL_DISTANCE_MILLIMETRES = 0
CANONICAL_MICROPHONE_ID = 7
PEAK_ALIGNMENT_SAMPLE = 512
ANALYSIS_SAMPLES = 3 * SAMPLE_RATE_HZ

ARCHIVE_URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/6_Bowl.zip"
ARCHIVE_BYTES = 2_397_750_726
ARCHIVE_ETAG = "6433dc5b-8eeac5c6"
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 09:52:27 GMT"
ARCHIVE_CENTRAL_SHA256 = (
    "05fecf1eec74c68968d90870417f0651b18825003e58dedd317315ff1dcd36b1"
)
SOURCE_REPOSITORY_REVISION = "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987"
DATASET_OBJECT_ID = "6_Bowl"
OBJECT_ID = "realimpact-6-bowl"
GEOMETRY_REVISION = "mesh-f23127b45b0b-v1"

AUDIO_ENTRY = {
    "name": "6_Bowl/preprocessed/deconvolved_0db.npy",
    "local_offset": 4_442,
    "data_offset": 4_539,
    "compressed_bytes": 2_393_833_339,
    "uncompressed_bytes": 2_762_580_128,
    "crc32": "e13a8db1",
    "dtype": "<f4",
    "shape": [ROW_COUNT, SAMPLE_COUNT],
}

METADATA_ENTRIES = {
    "vertex_xyz": {
        "name": "6_Bowl/preprocessed/vertexXYZ.npy",
        "local_offset": 583,
        "data_offset": 674,
        "compressed_bytes": 440,
        "uncompressed_bytes": 72_128,
        "crc32": "6680f5af",
        "sha256": "57b41204077d622d727a6f2727e96aa116b40ab31ef8fd2356cd8633246f1a94",
    },
    "microphone_id": {
        "name": "6_Bowl/preprocessed/micID.npy",
        "local_offset": 1_114,
        "data_offset": 1_201,
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "crc32": "082ec0c6",
        "sha256": "d603b6155b4bad60d9ed6633734b8f52fcad208d5320f925911fc7c463224d6b",
    },
    "mesh": {
        "name": "6_Bowl/preprocessed/transformed.obj",
        "local_offset": 2_397_069_970,
        "data_offset": 2_397_070_063,
        "compressed_bytes": 679_442,
        "uncompressed_bytes": 3_495_111,
        "crc32": "d9c0e316",
        "sha256": "f23127b45b0b163c796b4ef44d707b59fac58c4ef6e94e7676a4d88470e7f973",
    },
    "vertex_id": {
        "name": "6_Bowl/preprocessed/vertexID.npy",
        "local_offset": 330,
        "data_offset": 420,
        "compressed_bytes": 163,
        "uncompressed_bytes": 24_128,
        "crc32": "96be713a",
        "sha256": "bd9def2f8ce5896a9100b2ffd12092c9688ad1c465a2612d668d712744b05049",
    },
    "listener_xyz": {
        "name": "6_Bowl/preprocessed/listenerXYZ.npy",
        "local_offset": 1_426,
        "data_offset": 1_519,
        "compressed_bytes": 2_923,
        "uncompressed_bytes": 72_128,
        "crc32": "ee426e91",
        "sha256": "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
    },
    "distance": {
        "name": "6_Bowl/preprocessed/distance.npy",
        "local_offset": 2_393_838_257,
        "data_offset": 2_393_838_347,
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "crc32": "570b48dd",
        "sha256": "95dbc48e33263762a9d0c6233902e53ba754dc5e53943b17043491827dc7188a",
    },
    "angle": {
        "name": "6_Bowl/preprocessed/angle.npy",
        "local_offset": 2_393_837_878,
        "data_offset": 2_393_837_965,
        "compressed_bytes": 292,
        "uncompressed_bytes": 24_128,
        "crc32": "fabb9a2e",
        "sha256": "ed65ac28e45cc119b5d42c49293546f2749aa5f9629f6ffa9e0f35f2420874a3",
    },
}

OBJECTFOLDER_AUDIO_BATCH = {
    "url": (
        "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
        "audio_data_1_10.tar.gz"
    ),
    "content_length": 36_367_088_523,
    "etag": "63e36547-877a5bb8b",
    "last_modified": "Wed, 08 Feb 2023 09:03:03 GMT",
    "declared_objects": [1, 10],
    "declared_impacts_per_object": [30, 50],
    "declared_recording_seconds": 6,
}


class R3AError(RuntimeError):
    """The frozen R3A data, lineage or numeric boundary failed closed."""


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise R3AError(f"{label} must be an external file: {resolved}")
    return resolved


def prepare_output(root: Path, argument: Path) -> tuple[Path, Path]:
    unresolved = argument if argument.is_absolute() else root / argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise R3AError(f"output must be a new external directory: {output}")
    staging = parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        raise R3AError(f"staging directory already exists: {staging}")
    staging.mkdir()
    return output, staging


def publish_output(output: Path, staging: Path) -> None:
    os.replace(staging, output)


def load_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise R3AError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise R3AError(f"{label} must contain one JSON object")
    return data, value


def validate_manifest(manifest: dict[str, Any]) -> None:
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("profile") != PROFILE_ID
        or manifest.get("dataset_object_id") != DATASET_OBJECT_ID
        or manifest.get("sample_rate_hz") != SAMPLE_RATE_HZ
        or manifest.get("sample_count") != SAMPLE_COUNT
        or manifest.get("audio_entry") != AUDIO_ENTRY
        or manifest.get("canonical_listener", {}).get("microphone_id")
        != CANONICAL_MICROPHONE_ID
    ):
        raise R3AError("R3A corpus manifest identity changed")
    contacts = manifest.get("contacts")
    if not isinstance(contacts, list) or len(contacts) != IMPACT_COUNT:
        raise R3AError("R3A manifest must bind exactly five impacts")
    expected_roles = [
        "fit",
        "fit",
        "fit",
        "representation_development",
        "field_holdout",
    ]
    if [item.get("role") for item in contacts] != expected_roles:
        raise R3AError("R3A contact roles changed")
    rows = [item.get("row_index") for item in contacts]
    if rows != sorted(rows) or len(set(rows)) != IMPACT_COUNT:
        raise R3AError("R3A contact rows must be unique and source ordered")
    implementation = manifest.get("implementation_sha256")
    if not isinstance(implementation, dict) or set(implementation) != set(
        IMPLEMENTATION_FILES
    ):
        raise R3AError("R3A implementation lineage is incomplete")
    if any(
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
        for value in implementation.values()
    ):
        raise R3AError("R3A implementation hash is invalid")


def implementation_hashes(script_directory: Path) -> dict[str, str]:
    return {
        role: sha256_file(script_directory / filename)
        for role, filename in IMPLEMENTATION_FILES.items()
    }


def validate_implementation(manifest: dict[str, Any], script_directory: Path) -> None:
    if manifest.get("implementation_sha256") != implementation_hashes(script_directory):
        raise R3AError("R3A implementation changed after manifest freeze")


def align_peak(signal: np.ndarray) -> np.ndarray:
    if signal.ndim != 1 or signal.size < ANALYSIS_SAMPLES:
        raise R3AError("contact waveform is too short")
    if not np.all(np.isfinite(signal)):
        raise R3AError("contact waveform contains non-finite samples")
    peak = int(np.argmax(np.abs(signal)))
    shift = PEAK_ALIGNMENT_SAMPLE - peak
    aligned = np.zeros_like(signal, dtype=np.float64)
    if shift >= 0:
        aligned[shift:] = signal[: signal.size - shift]
    else:
        aligned[:shift] = signal[-shift:]
    return aligned
