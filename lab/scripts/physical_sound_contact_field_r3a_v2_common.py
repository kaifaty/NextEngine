#!/usr/bin/env python3
"""Frozen source, dependency and lineage boundary for the R3A V2 DAC oracle."""

from __future__ import annotations

import hashlib
import json
import os
import pickletools
import zipfile
from pathlib import Path
from typing import Any

import numpy as np

PROFILE_ID = "realimpact-large-swan-canonical-contact-r3a-v2-dac-oracle"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v2-corpus.manifest.v1"
PREFLIGHT_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v2-source-preflight.report.v1"
)
EXTRACTION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v2-contact-extraction.report.v1"
)
ORACLE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v2-learned-codec-oracle.report.v1"
)
IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v2_common.py",
    "preflight": "physical_sound_contact_field_r3a_v2_preflight.py",
    "extraction": "physical_sound_contact_field_r3a_v2_extract.py",
    "oracle": "physical_sound_contact_field_r3a_v2_codec_oracle.py",
}

SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 209_213
ROW_COUNT = 3_000
LISTENERS_PER_IMPACT = 600
IMPACT_COUNT = 5
CANONICAL_ANGLE_DEGREES = 0
CANONICAL_DISTANCE_MILLIMETRES = 0
CANONICAL_MICROPHONE_ID = 7
PEAK_ALIGNMENT_SAMPLE = 512
ANALYSIS_SAMPLES = 3 * SAMPLE_RATE_HZ

ARCHIVE_URL = (
    "https://downloads.cs.stanford.edu/viscam/RealImpact/79_LargeSwanCeramic.zip"
)
ARCHIVE_BYTES = 2_324_144_869
ARCHIVE_ETAG = "6433df70-8a87a2e5"
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 10:05:36 GMT"
ARCHIVE_CENTRAL_SHA256 = (
    "cb842d77ad318d2ad0af66c36b75e60314e5b5e60487562b195ae583fa335721"
)
SOURCE_REPOSITORY_REVISION = "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987"
SOURCE_ROSTER_SHA256 = (
    "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
)
DATASET_OBJECT_ID = "79_LargeSwanCeramic"
OBJECT_ID = "realimpact-79-large-swan-ceramic"
GEOMETRY_REVISION = "mesh-f68409bb1337-v1"

AUDIO_ENTRY = {
    "name": "79_LargeSwanCeramic/preprocessed/deconvolved_0db.npy",
    "local_offset": 436,
    "data_offset": 546,
    "compressed_bytes": 2_317_727_372,
    "uncompressed_bytes": 2_510_556_128,
    "crc32": "7f4ec9f0",
    "dtype": "<f4",
    "shape": [ROW_COUNT, SAMPLE_COUNT],
}

METADATA_ENTRIES = {
    "vertex_xyz": {
        "name": "79_LargeSwanCeramic/preprocessed/vertexXYZ.npy",
        "local_offset": 2_324_142_947,
        "data_offset": 2_324_143_051,
        "compressed_bytes": 441,
        "uncompressed_bytes": 72_128,
        "crc32": "93441f24",
        "sha256": "cd9c496d71ebe35952bb52dfc5c55c9b1e24700f3a59c2c104e249abff0d3df7",
    },
    "microphone_id": {
        "name": "79_LargeSwanCeramic/preprocessed/micID.npy",
        "local_offset": 2_317_727_918,
        "data_offset": 2_317_728_018,
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "crc32": "082ec0c6",
        "sha256": "d603b6155b4bad60d9ed6633734b8f52fcad208d5320f925911fc7c463224d6b",
    },
    "mesh": {
        "name": "79_LargeSwanCeramic/preprocessed/transformed.obj",
        "local_offset": 2_323_479_994,
        "data_offset": 2_323_480_100,
        "compressed_bytes": 662_847,
        "uncompressed_bytes": 3_539_723,
        "crc32": "ee396a31",
        "sha256": "f68409bb1337c1a276856b0be7c78033381ae95d8a4d0eaa9056c637c44cab2a",
    },
    "vertex_id": {
        "name": "79_LargeSwanCeramic/preprocessed/vertexID.npy",
        "local_offset": 169,
        "data_offset": 272,
        "compressed_bytes": 164,
        "uncompressed_bytes": 24_128,
        "crc32": "bd33f62c",
        "sha256": "39caf423868b41c326e556b03baa6abb257841783568d95cfffed0f52ef300e1",
    },
    "listener_xyz": {
        "name": "79_LargeSwanCeramic/preprocessed/listenerXYZ.npy",
        "local_offset": 2_317_729_032,
        "data_offset": 2_317_729_138,
        "compressed_bytes": 2_923,
        "uncompressed_bytes": 72_128,
        "crc32": "ee426e91",
        "sha256": "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
    },
    "distance": {
        "name": "79_LargeSwanCeramic/preprocessed/distance.npy",
        "local_offset": 2_317_728_243,
        "data_offset": 2_317_728_346,
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "crc32": "570b48dd",
        "sha256": "95dbc48e33263762a9d0c6233902e53ba754dc5e53943b17043491827dc7188a",
    },
    "angle": {
        "name": "79_LargeSwanCeramic/preprocessed/angle.npy",
        "local_offset": 2_317_728_640,
        "data_offset": 2_317_728_740,
        "compressed_bytes": 292,
        "uncompressed_bytes": 24_128,
        "crc32": "fabb9a2e",
        "sha256": "ed65ac28e45cc119b5d42c49293546f2749aa5f9629f6ffa9e0f35f2420874a3",
    },
}

DAC_REPOSITORY_URL = "https://github.com/descriptinc/descript-audio-codec.git"
DAC_REPOSITORY_REVISION = "c7cfc5d2647e26471dc394f95846a0830e7bec34"
DAC_MODEL_TYPE = "44khz"
DAC_MODEL_BITRATE = "8kbps"
DAC_MODEL_TAG = "0.0.1"
DAC_MODEL_SAMPLE_RATE_HZ = 44_100
DAC_MODEL_HOP_LENGTH = 512
DAC_MODEL_CODEBOOKS = 9
DAC_MODEL_CODEBOOK_SIZE = 1_024
DAC_WEIGHTS_URL = (
    "https://github.com/descriptinc/descript-audio-codec/releases/download/"
    "0.0.1/weights.pth"
)
DAC_WEIGHTS_BYTES = 306_717_287
DAC_WEIGHTS_SHA256 = "a88eed82a7024ccc1facdb1e605c4c2f99281c8118c22c9895ffa846d8fb61aa"
DAC_REQUIRED_PACKAGES = {
    "descript-audio-codec": "1.0.0",
    "descript-audiotools": "0.7.2",
    "einops": "0.6.1",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "torch": "2.0.1+cpu",
    "torchaudio": "2.0.2+cpu",
}
DAC_ALLOWED_PICKLE_GLOBALS = {
    "collections OrderedDict",
    "torch FloatStorage",
    "torch._utils _rebuild_tensor_v2",
}


class R3AV2Error(RuntimeError):
    """The frozen R3A V2 source, dependency or numeric boundary failed."""


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
        raise R3AV2Error(f"{label} must be an external file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise R3AV2Error(f"{label} must be an external directory: {resolved}")
    return resolved


def prepare_output(root: Path, argument: Path) -> tuple[Path, Path]:
    unresolved = argument if argument.is_absolute() else root / argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise R3AV2Error(f"output must be a new external directory: {output}")
    staging = parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        raise R3AV2Error(f"staging directory already exists: {staging}")
    staging.mkdir()
    return output, staging


def publish_output(output: Path, staging: Path) -> None:
    os.replace(staging, output)


def load_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise R3AV2Error(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise R3AV2Error(f"{label} must contain one JSON object")
    return data, value


def implementation_hashes(script_directory: Path) -> dict[str, str]:
    return {
        role: sha256_file(script_directory / filename)
        for role, filename in IMPLEMENTATION_FILES.items()
    }


def validate_implementation(manifest: dict[str, Any], script_directory: Path) -> None:
    if manifest.get("implementation_sha256") != implementation_hashes(script_directory):
        raise R3AV2Error("R3A V2 implementation changed after manifest freeze")


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
        or manifest.get("dac_dependency", {}).get("weights_sha256")
        != DAC_WEIGHTS_SHA256
        or manifest.get("dac_dependency", {}).get("repository_revision")
        != DAC_REPOSITORY_REVISION
    ):
        raise R3AV2Error("R3A V2 corpus manifest identity changed")
    contacts = manifest.get("contacts")
    if not isinstance(contacts, list) or len(contacts) != IMPACT_COUNT:
        raise R3AV2Error("R3A V2 manifest must bind exactly five impacts")
    expected_roles = [
        "fit",
        "fit",
        "fit",
        "representation_development",
        "field_holdout",
    ]
    if [item.get("role") for item in contacts] != expected_roles:
        raise R3AV2Error("R3A V2 contact roles changed")
    rows = [item.get("row_index") for item in contacts]
    if rows != sorted(rows) or len(set(rows)) != IMPACT_COUNT:
        raise R3AV2Error("R3A V2 contact rows must be unique and source ordered")
    implementation = manifest.get("implementation_sha256")
    if not isinstance(implementation, dict) or set(implementation) != set(
        IMPLEMENTATION_FILES
    ):
        raise R3AV2Error("R3A V2 implementation lineage is incomplete")
    if any(
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
        for value in implementation.values()
    ):
        raise R3AV2Error("R3A V2 implementation hash is invalid")


def validate_dac_weights(path: Path) -> dict[str, Any]:
    if (
        path.stat().st_size != DAC_WEIGHTS_BYTES
        or sha256_file(path) != DAC_WEIGHTS_SHA256
    ):
        raise R3AV2Error("DAC weights identity changed")
    try:
        with zipfile.ZipFile(path) as archive:
            pickle_payload = archive.read("weights/data.pkl")
    except (KeyError, OSError, zipfile.BadZipFile) as error:
        raise R3AV2Error(f"inspect DAC weights archive: {error}") from error
    globals_seen = {
        argument
        for opcode, argument, _ in pickletools.genops(pickle_payload)
        if opcode.name == "GLOBAL"
    }
    if globals_seen != DAC_ALLOWED_PICKLE_GLOBALS:
        raise R3AV2Error(f"DAC checkpoint pickle globals changed: {globals_seen}")
    return {
        "bytes": path.stat().st_size,
        "sha256": DAC_WEIGHTS_SHA256,
        "pickle_globals": sorted(globals_seen),
        "pickle_global_policy": "exact_allowlist_before_torch_load",
    }


def align_peak(signal: np.ndarray) -> np.ndarray:
    if signal.ndim != 1 or signal.size < ANALYSIS_SAMPLES:
        raise R3AV2Error("contact waveform is too short")
    if not np.all(np.isfinite(signal)):
        raise R3AV2Error("contact waveform contains non-finite samples")
    peak = int(np.argmax(np.abs(signal)))
    shift = PEAK_ALIGNMENT_SAMPLE - peak
    aligned = np.zeros_like(signal, dtype=np.float64)
    if shift >= 0:
        aligned[shift:] = signal[: signal.size - shift]
    else:
        aligned[:shift] = signal[-shift:]
    return aligned
