#!/usr/bin/env python3
"""Frozen source, dependency and lineage boundary for the R3A V3B NDAC oracle."""

from __future__ import annotations

import hashlib
import json
import os
import pickletools
import zipfile
from pathlib import Path
from typing import Any

import numpy as np

PROFILE_ID = "realimpact-purple-scoop-canonical-contact-r3a-v3b-ndac-oracle"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v3b-corpus.manifest.v1"
PREFLIGHT_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v3b-source-preflight.report.v1"
)
EXTRACTION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v3b-contact-extraction.report.v1"
)
ORACLE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v3b-learned-codec-oracle.report.v1"
)
IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v3b_common.py",
    "preflight": "physical_sound_contact_field_r3a_v3b_preflight.py",
    "extraction": "physical_sound_contact_field_r3a_v3b_extract.py",
    "oracle": "physical_sound_contact_field_r3a_v3b_codec_oracle.py",
}

SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 230_980
ROW_COUNT = 3_000
LISTENERS_PER_IMPACT = 600
IMPACT_COUNT = 5
CANONICAL_ANGLE_DEGREES = 0
CANONICAL_DISTANCE_MILLIMETRES = 0
CANONICAL_MICROPHONE_ID = 7
PEAK_ALIGNMENT_SAMPLE = 512
ANALYSIS_SAMPLES = 3 * SAMPLE_RATE_HZ

ARCHIVE_URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/23_PurpleScoop.zip"
ARCHIVE_BYTES = 2_551_224_404
ARCHIVE_ETAG = "6433dc4c-98109854"
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 09:52:12 GMT"
ARCHIVE_CENTRAL_SHA256 = (
    "fc9fc13ca884a4ff1ba8c53d4da6be690312f3e72493557a7f847e2d95e4a5ef"
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
    "73_PlasticBin",
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
DATASET_OBJECT_ID = "23_PurpleScoop"
OBJECT_ID = "realimpact-23-purple-scoop"
GEOMETRY_REVISION = "mesh-fd64baab41fe-v1"

AUDIO_ENTRY = {
    "name": "23_PurpleScoop/preprocessed/deconvolved_0db.npy",
    "local_offset": 4_114_780,
    "data_offset": 4_114_885,
    "compressed_bytes": 2_546_414_608,
    "uncompressed_bytes": 2_771_760_128,
    "crc32": "6f84a031",
    "dtype": "<f4",
    "shape": [ROW_COUNT, SAMPLE_COUNT],
}

METADATA_ENTRIES = {
    "vertex_xyz": {
        "name": "23_PurpleScoop/preprocessed/vertexXYZ.npy",
        "local_offset": 354,
        "data_offset": 453,
        "compressed_bytes": 441,
        "uncompressed_bytes": 72_128,
        "crc32": "ed8e4ae7",
        "sha256": "a3597878ece77a0a07d5f2372117e6e1fd26c5345f4b58e11c0535d81d30924b",
    },
    "microphone_id": {
        "name": "23_PurpleScoop/preprocessed/micID.npy",
        "local_offset": 1_542,
        "data_offset": 1_637,
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "crc32": "082ec0c6",
        "sha256": "d603b6155b4bad60d9ed6633734b8f52fcad208d5320f925911fc7c463224d6b",
    },
    "mesh": {
        "name": "23_PurpleScoop/preprocessed/transformed.obj",
        "local_offset": 2_550_529_493,
        "data_offset": 2_550_529_594,
        "compressed_bytes": 693_493,
        "uncompressed_bytes": 3_551_815,
        "crc32": "38068706",
        "sha256": "fd64baab41fe7febc58f0cb460d9cfcde5c8659e8a21c3cbcccfda6e9668e4ed",
    },
    "vertex_id": {
        "name": "23_PurpleScoop/preprocessed/vertexID.npy",
        "local_offset": 894,
        "data_offset": 992,
        "compressed_bytes": 163,
        "uncompressed_bytes": 24_128,
        "crc32": "772ac34e",
        "sha256": "7be477535192a3d71d72628d2b9289ef5ccf98d2b32e1ebdc8f0808519ff49be",
    },
    "listener_xyz": {
        "name": "23_PurpleScoop/preprocessed/listenerXYZ.npy",
        "local_offset": 4_111_756,
        "data_offset": 4_111_857,
        "compressed_bytes": 2_923,
        "uncompressed_bytes": 72_128,
        "crc32": "ee426e91",
        "sha256": "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
    },
    "distance": {
        "name": "23_PurpleScoop/preprocessed/distance.npy",
        "local_offset": 4_111_364,
        "data_offset": 4_111_462,
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "crc32": "570b48dd",
        "sha256": "95dbc48e33263762a9d0c6233902e53ba754dc5e53943b17043491827dc7188a",
    },
    "angle": {
        "name": "23_PurpleScoop/preprocessed/angle.npy",
        "local_offset": 1_155,
        "data_offset": 1_250,
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
DAC_DECODE_GUARD_SAMPLES = 8
DAC_ANALYSIS_CODE_FRAMES = 226
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


class R3AV3BError(RuntimeError):
    """The frozen R3A V3B source, dependency or numeric boundary failed."""


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
        raise R3AV3BError(f"{label} must be an external file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise R3AV3BError(f"{label} must be an external directory: {resolved}")
    return resolved


def prepare_output(root: Path, argument: Path) -> tuple[Path, Path]:
    unresolved = argument if argument.is_absolute() else root / argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise R3AV3BError(f"output must be a new external directory: {output}")
    staging = parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        raise R3AV3BError(f"staging directory already exists: {staging}")
    staging.mkdir()
    return output, staging


def publish_output(output: Path, staging: Path) -> None:
    os.replace(staging, output)


def load_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise R3AV3BError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise R3AV3BError(f"{label} must contain one JSON object")
    return data, value


def implementation_hashes(script_directory: Path) -> dict[str, str]:
    return {
        role: sha256_file(script_directory / filename)
        for role, filename in IMPLEMENTATION_FILES.items()
    }


def validate_implementation(manifest: dict[str, Any], script_directory: Path) -> None:
    if manifest.get("implementation_sha256") != implementation_hashes(script_directory):
        raise R3AV3BError("R3A V3B implementation changed after manifest freeze")


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
        or manifest.get("predecessor_invalid_evidence", {}).get("failure")
        != "ndac_decoded_143992_samples_for_144000_sample_target"
        or manifest.get("predecessor_invalid_evidence", {}).get(
            "quality_metrics_published"
        )
        is not False
        or manifest.get("predecessor_invalid_evidence", {}).get(
            "sealed_waveform_samples_decoded"
        )
        != 0
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
        or manifest.get("oracle_policy", {}).get("decode_guard_samples")
        != DAC_DECODE_GUARD_SAMPLES
    ):
        raise R3AV3BError("R3A V3B corpus manifest identity changed")
    contacts = manifest.get("contacts")
    if not isinstance(contacts, list) or len(contacts) != IMPACT_COUNT:
        raise R3AV3BError("R3A V3B manifest must bind exactly five impacts")
    expected_roles = [
        "fit",
        "fit",
        "fit",
        "representation_development",
        "field_holdout",
    ]
    if [item.get("role") for item in contacts] != expected_roles:
        raise R3AV3BError("R3A V3B contact roles changed")
    rows = [item.get("row_index") for item in contacts]
    if rows != sorted(rows) or len(set(rows)) != IMPACT_COUNT:
        raise R3AV3BError("R3A V3B contact rows must be unique and source ordered")
    implementation = manifest.get("implementation_sha256")
    if not isinstance(implementation, dict) or set(implementation) != set(
        IMPLEMENTATION_FILES
    ):
        raise R3AV3BError("R3A V3B implementation lineage is incomplete")
    if any(
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
        for value in implementation.values()
    ):
        raise R3AV3BError("R3A V3B implementation hash is invalid")


def validate_dac_weights(path: Path) -> dict[str, Any]:
    if (
        path.stat().st_size != DAC_WEIGHTS_BYTES
        or sha256_file(path) != DAC_WEIGHTS_SHA256
    ):
        raise R3AV3BError("NDAC-75 weights identity changed")
    try:
        with zipfile.ZipFile(path) as archive:
            pickle_payload = archive.read("weights/data.pkl")
    except (KeyError, OSError, zipfile.BadZipFile) as error:
        raise R3AV3BError(f"inspect NDAC-75 weights archive: {error}") from error
    globals_seen = {
        argument
        for opcode, argument, _ in pickletools.genops(pickle_payload)
        if opcode.name == "GLOBAL"
    }
    if globals_seen != DAC_ALLOWED_PICKLE_GLOBALS:
        raise R3AV3BError(f"NDAC-75 checkpoint pickle globals changed: {globals_seen}")
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
        raise R3AV3BError("FlowDec release archive identity changed")
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
        raise R3AV3BError(f"inspect FlowDec release archive: {error}") from error
    if (
        info.file_size != DAC_WEIGHTS_BYTES
        or info.compress_size != DAC_RELEASE_MEMBER_COMPRESSED_BYTES
        or f"{info.CRC:08x}" != DAC_RELEASE_MEMBER_CRC32
        or decoded_bytes != DAC_WEIGHTS_BYTES
        or digest.hexdigest() != DAC_WEIGHTS_SHA256
        or sha256_file(weights) != DAC_WEIGHTS_SHA256
    ):
        raise R3AV3BError("NDAC-75 member lineage changed")
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
        raise R3AV3BError("contact waveform is too short")
    if not np.all(np.isfinite(signal)):
        raise R3AV3BError("contact waveform contains non-finite samples")
    peak = int(np.argmax(np.abs(signal)))
    shift = PEAK_ALIGNMENT_SAMPLE - peak
    aligned = np.zeros_like(signal, dtype=np.float64)
    if shift >= 0:
        aligned[shift:] = signal[: signal.size - shift]
    else:
        aligned[:shift] = signal[-shift:]
    return aligned
