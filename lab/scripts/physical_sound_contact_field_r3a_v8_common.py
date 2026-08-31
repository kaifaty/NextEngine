#!/usr/bin/env python3
"""Frozen lineage and data boundary for the R3A V8 synthetic preflight."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import sys
from pathlib import Path
from typing import Any

import numpy as np
import scipy
import torch

STUDY_ID = "physical-sound-contact-field-r3a-v8-explicit-modal-neural"
REVISION = "nisr-glass-explicit-modal-synthetic-preflight-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8-preflight.report.v1"

NISR_REPOSITORY = "BumsooKim00/nisr-dataset"
NISR_REVISION = "20368791bcd7829e04ae3eb07c10aa0bb370e38a"
NISR_LICENSE = "CC-BY-4.0"
NISR_SOURCE_URL = f"https://huggingface.co/datasets/{NISR_REPOSITORY}"
NISR_FILES = (
    {
        "object_id": "1",
        "role": "opened_synthetic_format_control",
        "path": "training_dataset/1/feat/feat_Glass.npz",
        "bytes": 156_935,
        "sha256": "6548be1d819cef4c010aed787d5277e0c4de8021fa901b5345ad1dac115f36aa",
    },
    {
        "object_id": "2",
        "role": "opened_synthetic_scale_control",
        "path": "training_dataset/2/feat/feat_Glass.npz",
        "bytes": 866_616,
        "sha256": "d527053ef8c06f96ba4435a2e46ac03cbc690138f37dd2b9273f5496828322b9",
    },
)

IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v8_common.py",
    "model": "physical_sound_contact_field_r3a_v8_model.py",
    "preflight": "physical_sound_contact_field_r3a_v8_preflight.py",
}

REQUIRED_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "torch": "2.12.1+cu130",
}

MODAL_CONTROL_CONFIG = {
    "sample_rate_hz": 16_000,
    "sample_count": 8_000,
    "frequencies_hz": [333.0, 817.0, 1_511.0, 2_879.0],
    "damping_per_second": [3.0, 7.0, 15.0, 30.0],
    "gains": [0.8, -0.5, 0.32, 0.18],
    "frequency_initial_factors": [0.99, 1.01, 0.985, 1.015],
    "damping_initial_factors": [1.2, 0.8, 1.2, 0.8],
    "gain_initial_factors": [0.85, 1.1, 0.9, 1.2],
    "frequency_learning_rate": 2.0e-4,
    "damping_learning_rate": 2.0e-3,
    "gain_learning_rate": 3.0e-3,
    "updates": 2_000,
    "maximum_frequency_error_cents": 1.0,
    "maximum_relative_damping_error": 0.02,
    "maximum_relative_gain_error": 0.02,
    "maximum_waveform_mse": 1.0e-6,
}

FIELD_CONFIG = {
    "material": "Glass",
    "mode_count": 20,
    "coordinate_extent": 31.0,
    "fourier_bands": [1.0, 2.0, 4.0, 8.0],
    "context_fraction": 0.20,
    "minimum_context_points": 32,
    "hidden_width": 128,
    "hidden_layers": 2,
    "activation": "silu",
    "optimizer": "adamw",
    "learning_rate": 0.002,
    "weight_decay": 1.0e-5,
    "updates": 1_500,
    "seed": 20_260_831,
    "maximum_neural_to_nearest_rmse_ratio": 0.50,
    "minimum_query_prediction_std": 0.01,
    "target": "all_3x20_signed_boundary_mode_shape_components",
    "normalization": "context_only_per_component_rms_plus_1e-6",
}


class V8Error(RuntimeError):
    """The frozen V8 preflight boundary was violated."""


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


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise V8Error(f"{label} must be an external file")
    return resolved


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise V8Error("V8 output must stay outside the repository")
    if resolved.exists():
        raise V8Error("V8 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def publish_output(output: Path, staging: Path) -> None:
    staging.rename(output)


def implementation_hashes(directory: Path) -> dict[str, str]:
    hashes = {}
    for key, filename in IMPLEMENTATION_FILES.items():
        path = directory / filename
        if not path.is_file():
            raise V8Error(f"missing V8 implementation file: {filename}")
        hashes[key] = sha256_file(path)
    return hashes


def environment_identity(require_exact: bool = True) -> dict[str, Any]:
    observed = {
        "python": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "torch": torch.__version__,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "byteorder": sys.byteorder,
        "compute_device": "cpu",
        "torch_deterministic_algorithms": True,
        "torch_threads": 1,
    }
    if require_exact:
        for key, expected in REQUIRED_ENVIRONMENT.items():
            if observed[key] != expected:
                raise V8Error(
                    f"V8 environment changed for {key}: {observed[key]} != {expected}"
                )
    return observed


def validate_nisr_feature(
    root: Path, path: Path, descriptor: dict[str, Any]
) -> dict[str, Any]:
    resolved = require_external_file(
        root, path, f"NISR object {descriptor['object_id']} feature"
    )
    if resolved.stat().st_size != descriptor["bytes"]:
        raise V8Error(f"NISR object {descriptor['object_id']} byte size changed")
    digest = sha256_file(resolved)
    if digest != descriptor["sha256"]:
        raise V8Error(f"NISR object {descriptor['object_id']} hash changed")
    try:
        with np.load(resolved, allow_pickle=False) as archive:
            if set(archive.files) != {"coords", "feats_in", "surface", "freqs"}:
                raise V8Error("NISR feature keys changed")
            coords = np.asarray(archive["coords"])
            features = np.asarray(archive["feats_in"])
            surface = np.asarray(archive["surface"])
            frequencies = np.asarray(archive["freqs"])
    except (OSError, ValueError) as error:
        raise V8Error("cannot load NISR feature archive") from error

    point_count = coords.shape[0] if coords.ndim == 2 else 0
    if coords.shape != (point_count, 3) or point_count < 33:
        raise V8Error("NISR coordinate shape is invalid")
    if features.shape != (point_count, 3, FIELD_CONFIG["mode_count"]):
        raise V8Error("NISR mode-shape tensor is invalid")
    if surface.shape != (point_count, 6):
        raise V8Error("NISR surface encoding is invalid")
    if frequencies.shape != (FIELD_CONFIG["mode_count"],):
        raise V8Error("NISR frequency vector is invalid")
    numeric = (coords, features, surface, frequencies)
    if not all(np.isfinite(value).all() for value in numeric):
        raise V8Error("NISR feature archive contains non-finite values")
    if np.min(coords) < 0 or np.max(coords) > FIELD_CONFIG["coordinate_extent"]:
        raise V8Error("NISR coordinates exceed the frozen voxel extent")
    if np.any(frequencies <= 0.0) or np.any(np.diff(frequencies) < 0.0):
        raise V8Error("NISR frequencies are not positive and ordered")

    return {
        **descriptor,
        "resolved_path": str(resolved),
        "point_count": int(point_count),
        "frequency_min_hz": float(frequencies[0]),
        "frequency_max_hz": float(frequencies[-1]),
        "coords": coords.astype(np.float32, copy=False),
        "features": features.astype(np.float32, copy=False),
        "surface": surface.astype(np.float32, copy=False),
        "frequencies": frequencies.astype(np.float32, copy=False),
        "sha256_observed": digest,
    }


def public_source_descriptor(source: dict[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in source.items()
        if key
        not in {
            "resolved_path",
            "coords",
            "features",
            "surface",
            "frequencies",
        }
    }

