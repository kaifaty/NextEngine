#!/usr/bin/env python3
"""Frozen constants and lineage helpers for R3A V5 neural preflight B."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import struct
import sys
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

import numpy as np
import scipy
import torch

STUDY_ID = "physical-sound-contact-field-r3a-v5-neural-representation"
REVISION = "internet-impact-rvq-preflight-b-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-preflight.report.v1"

IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v5_common.py",
    "model": "physical_sound_contact_field_r3a_v5_model.py",
    "preflight": "physical_sound_contact_field_r3a_v5_preflight.py",
}

SOURCE_MANIFEST_SHA256 = (
    "6377ea4019ca37ada7a9ca1c1a6e8bff71647a83d2b8d9ea39ee074caf31567e"
)
SOURCE_ENTRY_ID = "cmu-auditorylab-impact-glass-vase"
SOURCE_ARCHIVE_URL = "https://ndownloader.figshare.com/files/36113411"
ARCHIVE_NAME = "Impacts_audio1.zip"
ARCHIVE_BYTES = 38_059_995
ARCHIVE_SHA256 = "1d57964b4f48d277b6cf567a0bce938b3b8901f6b43ca6787db1036e8d737783"
README_MEMBER = "Impacts_audio1/readmeSoundEvents.txt"
README_SHA256 = "eb8325072b3fa53a13968349492835b4f68c2d4e169487eb614e35a7e76e49c8"
LICENSE_EXPRESSION = "LicenseRef-Heller-Sound-Events-Research-Only"
REDISTRIBUTION_POLICY = "external_research_only"
CORPUS_SPLIT_SEED = "nextengine-v5-corpus-split-v1"
INTERNAL_VALIDATION_GROUP_COUNT = 4

V4_MANIFEST_SHA256 = "6f9fe00ea14c99da2b2fc8b71b22af6772725ac5f28430a2417b8717819386b1"
V4_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v4-dev.manifest.v1"
V4_OBJECT_IDS = ["blue_bowl", "large_swan", "plastic_bin", "purple_scoop"]

SOURCE_SAMPLE_RATE_HZ = 44_100
SOURCE_CHANNELS = 2
SOURCE_SAMPLE_WIDTH_BYTES = 2
TARGET_SAMPLE_RATE_HZ = 48_000
TRAINING_SEGMENT_SAMPLES = 48_000
ANALYSIS_SAMPLES = 144_000

CAPACITIES = (
    {"id": "rvq-6kbps", "quantizers": 4, "nominal_bits_per_second": 6_000},
    {"id": "rvq-12kbps", "quantizers": 8, "nominal_bits_per_second": 12_000},
    {"id": "rvq-24kbps", "quantizers": 16, "nominal_bits_per_second": 24_000},
)

MODEL_CONFIG = {
    "sample_rate_hz": TARGET_SAMPLE_RATE_HZ,
    "encoder_strides": [2, 4, 5, 8],
    "hop_samples": 320,
    "latent_frames_per_second": 150,
    "base_channels": 32,
    "maximum_channels": 256,
    "latent_dimension": 128,
    "codebook_size": 1_024,
    "codebook_dimension": 8,
    "maximum_quantizers": 16,
    "activation": "snake1d",
    "normalization": "weight_norm_truncated_normal_std_0.02_zero_bias",
    "rvq": "factorized_l2_selection_sequential_residual",
    "decoder_output": "tanh",
}

LOSS_CONFIG = {
    "waveform_l1": {"weight": 1.0},
    "si_sdr": {"weight": 1.0, "epsilon": 1.0e-8},
    "complex_stft": {
        "weight": 2.0,
        "window_samples": [512, 2_048, 8_192],
        "hop_divisor": 4,
    },
    "log_spectrum": {
        "weight": 10.0,
        "window_samples": [2_048, 8_192, 32_768],
        "hop_divisor": 2,
        "minimum_hz": 120.0,
        "maximum_hz": 18_000.0,
        "floor_db": -100.0,
    },
    "log_mel": {
        "weight": 5.0,
        "window_samples": [128, 256, 512, 1_024, 2_048],
        "mel_bins": [20, 40, 80, 160, 320],
        "hop_divisor": 4,
    },
    "envelope": {"weight": 2.0, "smoothing_samples": 240},
    "decay_energy_curve": {"weight": 2.0, "floor_db": -100.0},
    "target_peak_emphasis": {"weight": 4.0, "top_bins_per_scale": 32},
    "rvq_codebook": {"weight": 1.0},
    "rvq_commitment": {"weight": 0.25},
    "adversarial": {"enabled": False, "reason": "bounded_first_reconstruction_test"},
}

TRAINING_CONFIG = {
    "separate_model_per_capacity": True,
    "optimizer": "adamw",
    "learning_rate": 2.0e-4,
    "betas": [0.8, 0.9],
    "weight_decay": 1.0e-4,
    "gradient_clip_norm": 1.0,
    "warmup_steps": 2_000,
    "maximum_steps": 50_000,
    "schedule": "linear_warmup_then_half_cosine_to_0.05x",
    "checkpoint_interval_steps": 1_000,
    "checkpoint_selection": (
        "minimum_internal_validation_frozen_loss_then_earliest_step"
    ),
    "random_seed": 20_260_831,
    "segment_policy": {
        "samples": TRAINING_SEGMENT_SAMPLES,
        "onset_aligned_probability": 0.5,
        "otherwise": "seeded_uniform_valid_crop",
        "short_clip": "right_zero_pad_with_loss_mask",
    },
    "normalization": "per_clip_peak_to_0.92_plus_one_float32_output_gain",
}

TRACKING_CONTRACT = {
    "canonical_run_schema": "nextengine.experimental-ml-run.v1",
    "required_namespaces": ["params", "metrics", "artifacts", "lineage"],
    "primary_store": "canonical_json_external_directory",
    "optional_mirror": "mlflow_local_file_store_external_only",
    "remote_tracking_server_authorized": False,
    "model_registry_promotion_authorized": False,
}

REQUIRED_CONTROL_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "torch": "2.0.1+cpu",
    "openblas_num_threads": "1",
    "omp_num_threads": "1",
    "mkl_num_threads": "1",
}

MAXIMUM_ARCHIVE_UNCOMPRESSED_BYTES = 128 * 1024 * 1024
MAXIMUM_MEMBER_BYTES = 32 * 1024 * 1024


class V5Error(RuntimeError):
    """The frozen V5 preflight boundary was violated."""


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def load_json(
    path: Path, label: str, canonical: bool = False
) -> tuple[bytes, dict[str, Any]]:
    payload = path.read_bytes()
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise V5Error(f"cannot parse {label}") from error
    if not isinstance(value, dict) or (canonical and canonical_json(value) != payload):
        raise V5Error(f"{label} has an invalid JSON boundary")
    return payload, value


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise V5Error(f"{label} must be an external file")
    return resolved


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise V5Error("V5 output must stay outside the repository")
    if resolved.exists():
        raise V5Error("V5 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def publish_output(output: Path, staging: Path) -> None:
    staging.rename(output)


def implementation_hashes(directory: Path) -> dict[str, str]:
    result = {}
    for key, filename in IMPLEMENTATION_FILES.items():
        path = directory / filename
        if not path.is_file():
            raise V5Error(f"missing V5 implementation file: {filename}")
        result[key] = sha256_file(path)
    return result


def control_environment() -> dict[str, Any]:
    observed = {
        "python": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "torch": torch.__version__,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "byteorder": sys.byteorder,
        "openblas_num_threads": os.environ.get("OPENBLAS_NUM_THREADS", ""),
        "omp_num_threads": os.environ.get("OMP_NUM_THREADS", ""),
        "mkl_num_threads": os.environ.get("MKL_NUM_THREADS", ""),
    }
    for key, expected in REQUIRED_CONTROL_ENVIRONMENT.items():
        if observed[key] != expected:
            raise V5Error(f"V5 control environment changed for {key}: {observed[key]}")
    if observed["byteorder"] != "little":
        raise V5Error("V5 requires a little-endian control host")
    return observed


def safe_member_name(name: str) -> bool:
    path = PurePosixPath(name)
    return (
        bool(name)
        and "\\" not in name
        and not path.is_absolute()
        and all(part not in {"", ".", ".."} for part in path.parts)
    )


def parse_pcm_wave_header(payload: bytes, label: str) -> dict[str, int]:
    if len(payload) < 12 or payload[:4] != b"RIFF" or payload[8:12] != b"WAVE":
        raise V5Error(f"{label} is not RIFF/WAVE")
    offset = 12
    fmt = None
    data_bytes = None
    while offset + 8 <= len(payload):
        chunk_id = payload[offset : offset + 4]
        chunk_bytes = struct.unpack_from("<I", payload, offset + 4)[0]
        offset += 8
        if offset + chunk_bytes > len(payload):
            raise V5Error(f"{label} has a truncated RIFF chunk")
        if chunk_id == b"fmt ":
            if chunk_bytes < 16:
                raise V5Error(f"{label} has a short fmt chunk")
            fmt = struct.unpack_from("<HHIIHH", payload, offset)
        elif chunk_id == b"data":
            data_bytes = chunk_bytes
            break
        offset += chunk_bytes + (chunk_bytes & 1)
    if fmt is None or data_bytes is None:
        raise V5Error(f"{label} lacks fmt or data")
    audio_format, channels, sample_rate, byte_rate, block_align, bits = fmt
    expected_align = SOURCE_CHANNELS * SOURCE_SAMPLE_WIDTH_BYTES
    if (
        audio_format != 1
        or channels != SOURCE_CHANNELS
        or sample_rate != SOURCE_SAMPLE_RATE_HZ
        or bits != SOURCE_SAMPLE_WIDTH_BYTES * 8
        or block_align != expected_align
        or byte_rate != sample_rate * expected_align
        or data_bytes % expected_align
    ):
        raise V5Error(f"{label} PCM identity changed")
    return {
        "channels": channels,
        "sample_rate_hz": sample_rate,
        "sample_width_bytes": SOURCE_SAMPLE_WIDTH_BYTES,
        "frames": data_bytes // expected_align,
        "data_bytes": data_bytes,
    }


def group_role(group: str, all_groups: list[str]) -> str:
    ranked = sorted(
        all_groups,
        key=lambda value: hashlib.sha256(
            f"{CORPUS_SPLIT_SEED}\0{value}".encode()
        ).digest(),
    )
    return (
        "internal_validation"
        if group in ranked[:INTERNAL_VALIDATION_GROUP_COUNT]
        else "train"
    )


def inspect_archive(path: Path) -> dict[str, Any]:
    if path.name != ARCHIVE_NAME or path.stat().st_size != ARCHIVE_BYTES:
        raise V5Error("Heller archive size or name changed")
    if sha256_file(path) != ARCHIVE_SHA256:
        raise V5Error("Heller archive hash changed")
    with zipfile.ZipFile(path) as archive:
        members = archive.infolist()
        names = [item.filename for item in members]
        if len(names) != len(set(names)) or not all(
            safe_member_name(name) for name in names
        ):
            raise V5Error("Heller archive has duplicate or unsafe member names")
        if any(item.flag_bits & 1 for item in members):
            raise V5Error("Heller archive contains encrypted members")
        if sum(item.file_size for item in members) > MAXIMUM_ARCHIVE_UNCOMPRESSED_BYTES:
            raise V5Error("Heller archive exceeds the uncompressed budget")
        if any(item.file_size > MAXIMUM_MEMBER_BYTES for item in members):
            raise V5Error("Heller archive member exceeds the size budget")
        readme = archive.read(README_MEMBER)
        if sha256_bytes(readme) != README_SHA256:
            raise V5Error("Heller readme changed")
        wav_members = [
            item
            for item in members
            if item.filename.startswith("Impacts_audio1/")
            and item.filename.lower().endswith(".wav")
        ]
        groups = sorted({item.filename.split("/")[1] for item in wav_members})
        clips = []
        for item in sorted(wav_members, key=lambda value: value.filename):
            payload = archive.read(item)
            header = parse_pcm_wave_header(payload, item.filename)
            group = item.filename.split("/")[1]
            clips.append(
                {
                    "member": item.filename,
                    "event_group": group,
                    "role": group_role(group, groups),
                    "member_bytes": item.file_size,
                    "member_sha256": sha256_bytes(payload),
                    **header,
                }
            )
    if len(clips) != 73 or len(groups) != 17:
        raise V5Error("Heller V5 corpus cardinality changed")
    return {
        "archive_name": ARCHIVE_NAME,
        "archive_url": SOURCE_ARCHIVE_URL,
        "archive_bytes": ARCHIVE_BYTES,
        "archive_sha256": ARCHIVE_SHA256,
        "readme_member": README_MEMBER,
        "readme_sha256": README_SHA256,
        "license_expression": LICENSE_EXPRESSION,
        "redistribution_policy": REDISTRIBUTION_POLICY,
        "semantic_scope": "unlabeled_impact_reconstruction_training_only",
        "groups": [
            {"id": group, "role": group_role(group, groups)} for group in groups
        ],
        "clips": clips,
    }


def validate_source_manifest(path: Path) -> None:
    payload, value = load_json(path, "Heller source manifest")
    if sha256_bytes(payload) != SOURCE_MANIFEST_SHA256:
        raise V5Error("Heller source manifest hash changed")
    matches = [
        item for item in value.get("sources", []) if item.get("id") == SOURCE_ENTRY_ID
    ]
    if len(matches) != 1:
        raise V5Error("Heller source entry is missing or duplicated")
    source = matches[0]
    artifacts = {item.get("id"): item for item in source.get("artifacts", [])}
    archive = artifacts.get("impact-audio-archive", {})
    if (
        source.get("license_expression") != LICENSE_EXPRESSION
        or source.get("redistribution_policy") != REDISTRIBUTION_POLICY
        or archive.get("url") != SOURCE_ARCHIVE_URL
        or archive.get("expected_byte_count") != ARCHIVE_BYTES
        or archive.get("expected_sha256") != ARCHIVE_SHA256
    ):
        raise V5Error("Heller source/provenance identity changed")


def validate_v4_manifest(path: Path) -> dict[str, Any]:
    payload, value = load_json(path, "V4 manifest", canonical=True)
    if (
        sha256_bytes(payload) != V4_MANIFEST_SHA256
        or value.get("schema") != V4_MANIFEST_SCHEMA
        or [item.get("id") for item in value.get("objects", [])] != V4_OBJECT_IDS
        or value.get("new_object_or_sealed_waveform_access_authorized") is not False
    ):
        raise V5Error("V4 development commitment changed")
    return {
        "manifest_sha256": V4_MANIFEST_SHA256,
        "object_ids": V4_OBJECT_IDS,
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
    }
