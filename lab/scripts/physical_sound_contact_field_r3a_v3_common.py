#!/usr/bin/env python3
"""Frozen source, dependency and lineage boundary for the R3A V3 NDAC oracle."""

from __future__ import annotations

import hashlib
import json
import os
import pickletools
import zipfile
from pathlib import Path
from typing import Any

import numpy as np

PROFILE_ID = "realimpact-plastic-bin-canonical-contact-r3a-v3-ndac-oracle"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v3-corpus.manifest.v1"
PREFLIGHT_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v3-source-preflight.report.v1"
)
EXTRACTION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v3-contact-extraction.report.v1"
)
ORACLE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v3-learned-codec-oracle.report.v1"
)
IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v3_common.py",
    "preflight": "physical_sound_contact_field_r3a_v3_preflight.py",
    "extraction": "physical_sound_contact_field_r3a_v3_extract.py",
    "oracle": "physical_sound_contact_field_r3a_v3_codec_oracle.py",
}

SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 209_057
ROW_COUNT = 3_000
LISTENERS_PER_IMPACT = 600
IMPACT_COUNT = 5
CANONICAL_ANGLE_DEGREES = 0
CANONICAL_DISTANCE_MILLIMETRES = 0
CANONICAL_MICROPHONE_ID = 7
PEAK_ALIGNMENT_SAMPLE = 512
ANALYSIS_SAMPLES = 3 * SAMPLE_RATE_HZ

ARCHIVE_URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/73_PlasticBin.zip"
ARCHIVE_BYTES = 2_331_118_925
ARCHIVE_ETAG = "6433dcfb-8af20d4d"
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 09:55:07 GMT"
ARCHIVE_CENTRAL_SHA256 = (
    "bf47ccc2e3895c60f78b8dfd308102e34f01d34a2e8cc6345434ffa5c29af770"
)
SOURCE_REPOSITORY_REVISION = "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987"
SOURCE_ROSTER_SHA256 = (
    "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
)
PRIOR_ACCESSED_DATASET_OBJECT_IDS = (
    "100_Frisbee",
    "10_bowl",
    "17_IronSkillet",
    "19_Pan",
    "22_Cup",
    "31_WoodSlab",
    "32_WoodChalice",
    "33_WoodWineGlass",
    "34_WoodMug",
    "36_SmallMeasuringCup",
    "37_PiePan",
    "43_IronMortar",
    "48_PlasticBowl",
    "49_PlasticBowl",
    "51_ShellPlate",
    "60_SkullCup",
    "63_SmallPlanterCeramic",
    "65_PitcherCeramic",
    "67_IronPlate",
    "68_WoodBoard",
    "6_Bowl",
    "78_CeramicCup",
    "79_LargeSwanCeramic",
    "81_WoodPad",
    "83_WoodVase",
    "86_MetalHoledSpoon",
    "89_MetalSpatula",
    "90_MetalLadle",
    "91_MetalSpoon",
    "92_MetalSpatula",
    "93_GreenGoblet",
    "94_GlassGoblet",
    "9_BowlCeramic",
)
DATASET_OBJECT_ID = "73_PlasticBin"
OBJECT_ID = "realimpact-73-plastic-bin"
GEOMETRY_REVISION = "mesh-197e6b2dd235-v1"

AUDIO_ENTRY = {
    "name": "73_PlasticBin/preprocessed/deconvolved_0db.npy",
    "local_offset": 4_425_453,
    "data_offset": 4_425_557,
    "compressed_bytes": 2_326_131_431,
    "uncompressed_bytes": 2_508_684_128,
    "crc32": "6bbccb74",
    "dtype": "<f4",
    "shape": [ROW_COUNT, SAMPLE_COUNT],
}

METADATA_ENTRIES = {
    "vertex_xyz": {
        "name": "73_PlasticBin/preprocessed/vertexXYZ.npy",
        "local_offset": 801,
        "data_offset": 899,
        "compressed_bytes": 439,
        "uncompressed_bytes": 72_128,
        "crc32": "48dbd1bf",
        "sha256": "164919ad3bbfaa03fd196e0d88cac71a24e83da83f201d356c0b92949208d050",
    },
    "microphone_id": {
        "name": "73_PlasticBin/preprocessed/micID.npy",
        "local_offset": 2_331_117_301,
        "data_offset": 2_331_117_395,
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "crc32": "082ec0c6",
        "sha256": "d603b6155b4bad60d9ed6633734b8f52fcad208d5320f925911fc7c463224d6b",
    },
    "mesh": {
        "name": "73_PlasticBin/preprocessed/transformed.obj",
        "local_offset": 2_330_560_205,
        "data_offset": 2_330_560_305,
        "compressed_bytes": 556_996,
        "uncompressed_bytes": 3_523_505,
        "crc32": "f7cf4d66",
        "sha256": "197e6b2dd235349613c6e0121619f062915f84bd9028b456513a94496d3499f5",
    },
    "vertex_id": {
        "name": "73_PlasticBin/preprocessed/vertexID.npy",
        "local_offset": 548,
        "data_offset": 645,
        "compressed_bytes": 156,
        "uncompressed_bytes": 24_128,
        "crc32": "a062bbd6",
        "sha256": "1645310e9ff1be546ef8ffe7508973eecaa4c5be15c4ce6e5efd42874cf7813a",
    },
    "listener_xyz": {
        "name": "73_PlasticBin/preprocessed/listenerXYZ.npy",
        "local_offset": 2_330_556_988,
        "data_offset": 2_330_557_088,
        "compressed_bytes": 2_923,
        "uncompressed_bytes": 72_128,
        "crc32": "ee426e91",
        "sha256": "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
    },
    "distance": {
        "name": "73_PlasticBin/preprocessed/distance.npy",
        "local_offset": 157,
        "data_offset": 254,
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "crc32": "570b48dd",
        "sha256": "95dbc48e33263762a9d0c6233902e53ba754dc5e53943b17043491827dc7188a",
    },
    "angle": {
        "name": "73_PlasticBin/preprocessed/angle.npy",
        "local_offset": 1_338,
        "data_offset": 1_432,
        "compressed_bytes": 292,
        "uncompressed_bytes": 24_128,
        "crc32": "fabb9a2e",
        "sha256": "ed65ac28e45cc119b5d42c49293546f2749aa5f9629f6ffa9e0f35f2420874a3",
    },
}

DAC_REPOSITORY_URL = "https://github.com/facebookresearch/FlowDec.git"
DAC_REPOSITORY_REVISION = "26ec106281f061a720e0399b0edfd3c3c020d7ab"
DAC_REPOSITORY_TREE = "671e689063e0c90d3c9c3a5766192097f4bbcf5b"
DAC_MODEL_TYPE = "ndac-75"
DAC_MODEL_BITRATE = "7.5kbps"
DAC_MODEL_TAG = "800k"
DAC_MODEL_SAMPLE_RATE_HZ = 48_000
DAC_MODEL_HOP_LENGTH = 640
DAC_MODEL_CODEBOOKS = 10
DAC_MODEL_CODEBOOK_SIZE = 1_024
DAC_WEIGHTS_URL = (
    "https://github.com/facebookresearch/FlowDec/releases/download/checkpoints/"
    "checkpoints.zip#checkpoints/ndac/ndac-75/800k/dac/weights.pth"
)
DAC_RELEASE_ARCHIVE_BYTES = 1_205_002_207
DAC_RELEASE_ARCHIVE_SHA256 = (
    "4f8c72ae264f32be583ed7d4fcc60704af8c9ab200dc0e7d8414dec57f4d163c"
)
DAC_RELEASE_MEMBER = "checkpoints/ndac/ndac-75/800k/dac/weights.pth"
DAC_RELEASE_MEMBER_COMPRESSED_BYTES = 239_821_592
DAC_RELEASE_MEMBER_CRC32 = "2f5c1782"
DAC_WEIGHTS_BYTES = 258_100_682
DAC_WEIGHTS_SHA256 = "4eb9d0daf1d5f0efa0c890b8902ac1d40d8d18b331b6b05940d03f172fab08f0"
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


class R3AV3Error(RuntimeError):
    """The frozen R3A V3 source, dependency or numeric boundary failed."""


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
        raise R3AV3Error(f"{label} must be an external file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise R3AV3Error(f"{label} must be an external directory: {resolved}")
    return resolved


def prepare_output(root: Path, argument: Path) -> tuple[Path, Path]:
    unresolved = argument if argument.is_absolute() else root / argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise R3AV3Error(f"output must be a new external directory: {output}")
    staging = parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        raise R3AV3Error(f"staging directory already exists: {staging}")
    staging.mkdir()
    return output, staging


def publish_output(output: Path, staging: Path) -> None:
    os.replace(staging, output)


def load_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise R3AV3Error(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise R3AV3Error(f"{label} must contain one JSON object")
    return data, value


def implementation_hashes(script_directory: Path) -> dict[str, str]:
    return {
        role: sha256_file(script_directory / filename)
        for role, filename in IMPLEMENTATION_FILES.items()
    }


def validate_implementation(manifest: dict[str, Any], script_directory: Path) -> None:
    if manifest.get("implementation_sha256") != implementation_hashes(script_directory):
        raise R3AV3Error("R3A V3 implementation changed after manifest freeze")


def validate_manifest(manifest: dict[str, Any]) -> None:
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("profile") != PROFILE_ID
        or manifest.get("dataset_object_id") != DATASET_OBJECT_ID
        or manifest.get("sample_rate_hz") != SAMPLE_RATE_HZ
        or manifest.get("sample_count") != SAMPLE_COUNT
        or manifest.get("audio_entry") != AUDIO_ENTRY
        or manifest.get("source_disjoint_audit", {}).get(
            "prior_accessed_dataset_object_ids"
        )
        != list(PRIOR_ACCESSED_DATASET_OBJECT_IDS)
        or manifest.get("source_disjoint_audit", {}).get("candidate_absent") is not True
        or DATASET_OBJECT_ID in PRIOR_ACCESSED_DATASET_OBJECT_IDS
        or manifest.get("prior_object_payload_bytes_opened") != 0
        or manifest.get("canonical_listener", {}).get("microphone_id")
        != CANONICAL_MICROPHONE_ID
        or manifest.get("dac_dependency", {}).get("weights_sha256")
        != DAC_WEIGHTS_SHA256
        or manifest.get("dac_dependency", {}).get("repository_revision")
        != DAC_REPOSITORY_REVISION
        or manifest.get("dac_dependency", {}).get("release_archive_sha256")
        != DAC_RELEASE_ARCHIVE_SHA256
        or manifest.get("dac_dependency", {}).get("model_sample_rate_hz")
        != DAC_MODEL_SAMPLE_RATE_HZ
        or manifest.get("dac_dependency", {}).get("model_codebooks")
        != DAC_MODEL_CODEBOOKS
        or manifest.get("oracle_policy", {}).get("flowdec_postfilter")
        != "excluded_stochastic_not_loaded"
    ):
        raise R3AV3Error("R3A V3 corpus manifest identity changed")
    contacts = manifest.get("contacts")
    if not isinstance(contacts, list) or len(contacts) != IMPACT_COUNT:
        raise R3AV3Error("R3A V3 manifest must bind exactly five impacts")
    expected_roles = [
        "fit",
        "fit",
        "fit",
        "representation_development",
        "field_holdout",
    ]
    if [item.get("role") for item in contacts] != expected_roles:
        raise R3AV3Error("R3A V3 contact roles changed")
    rows = [item.get("row_index") for item in contacts]
    if rows != sorted(rows) or len(set(rows)) != IMPACT_COUNT:
        raise R3AV3Error("R3A V3 contact rows must be unique and source ordered")
    implementation = manifest.get("implementation_sha256")
    if not isinstance(implementation, dict) or set(implementation) != set(
        IMPLEMENTATION_FILES
    ):
        raise R3AV3Error("R3A V3 implementation lineage is incomplete")
    if any(
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
        for value in implementation.values()
    ):
        raise R3AV3Error("R3A V3 implementation hash is invalid")


def validate_dac_weights(path: Path) -> dict[str, Any]:
    if (
        path.stat().st_size != DAC_WEIGHTS_BYTES
        or sha256_file(path) != DAC_WEIGHTS_SHA256
    ):
        raise R3AV3Error("NDAC-75 weights identity changed")
    try:
        with zipfile.ZipFile(path) as archive:
            pickle_payload = archive.read("weights/data.pkl")
    except (KeyError, OSError, zipfile.BadZipFile) as error:
        raise R3AV3Error(f"inspect NDAC-75 weights archive: {error}") from error
    globals_seen = {
        argument
        for opcode, argument, _ in pickletools.genops(pickle_payload)
        if opcode.name == "GLOBAL"
    }
    if globals_seen != DAC_ALLOWED_PICKLE_GLOBALS:
        raise R3AV3Error(f"NDAC-75 checkpoint pickle globals changed: {globals_seen}")
    return {
        "bytes": path.stat().st_size,
        "sha256": DAC_WEIGHTS_SHA256,
        "pickle_globals": sorted(globals_seen),
        "pickle_global_policy": "exact_allowlist_before_torch_load",
    }


def validate_dac_release_archive(path: Path, weights: Path) -> dict[str, Any]:
    if (
        path.stat().st_size != DAC_RELEASE_ARCHIVE_BYTES
        or sha256_file(path) != DAC_RELEASE_ARCHIVE_SHA256
    ):
        raise R3AV3Error("FlowDec release archive identity changed")
    try:
        with zipfile.ZipFile(path) as archive:
            info = archive.getinfo(DAC_RELEASE_MEMBER)
            digest = hashlib.sha256()
            decoded_bytes = 0
            with archive.open(info) as handle:
                while block := handle.read(8 * 1024 * 1024):
                    digest.update(block)
                    decoded_bytes += len(block)
    except (KeyError, OSError, zipfile.BadZipFile) as error:
        raise R3AV3Error(f"inspect FlowDec release archive: {error}") from error
    if (
        info.file_size != DAC_WEIGHTS_BYTES
        or info.compress_size != DAC_RELEASE_MEMBER_COMPRESSED_BYTES
        or f"{info.CRC:08x}" != DAC_RELEASE_MEMBER_CRC32
        or decoded_bytes != DAC_WEIGHTS_BYTES
        or digest.hexdigest() != DAC_WEIGHTS_SHA256
        or sha256_file(weights) != DAC_WEIGHTS_SHA256
    ):
        raise R3AV3Error("NDAC-75 member lineage changed")
    return {
        "bytes": path.stat().st_size,
        "sha256": DAC_RELEASE_ARCHIVE_SHA256,
        "member": DAC_RELEASE_MEMBER,
        "member_bytes": info.file_size,
        "member_compressed_bytes": info.compress_size,
        "member_crc32": f"{info.CRC:08x}",
        "member_sha256": digest.hexdigest(),
        "extracted_weights_match_member": True,
    }


def align_peak(signal: np.ndarray) -> np.ndarray:
    if signal.ndim != 1 or signal.size < ANALYSIS_SAMPLES:
        raise R3AV3Error("contact waveform is too short")
    if not np.all(np.isfinite(signal)):
        raise R3AV3Error("contact waveform contains non-finite samples")
    peak = int(np.argmax(np.abs(signal)))
    shift = PEAK_ALIGNMENT_SAMPLE - peak
    aligned = np.zeros_like(signal, dtype=np.float64)
    if shift >= 0:
        aligned[shift:] = signal[: signal.size - shift]
    else:
        aligned[:shift] = signal[-shift:]
    return aligned
