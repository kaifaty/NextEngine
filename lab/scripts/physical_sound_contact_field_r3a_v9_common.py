#!/usr/bin/env python3
"""Frozen boundary for the R3A V9 synthetic residual preflight."""

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

STUDY_ID = "physical-sound-contact-field-r3a-v9-time-varying-residual"
REVISION = "explicit-modal-deterministic-noise-band-residual-synthetic-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v9.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v9-preflight.report.v1"

IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v9_common.py",
    "model": "physical_sound_contact_field_r3a_v9_model.py",
    "preflight": "physical_sound_contact_field_r3a_v9_preflight.py",
    "endpoints": "physical_sound_contact_field_r3a_v4_common.py",
}

REQUIRED_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "torch": "2.12.1+cu130",
}

SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 48_000
ALIGNMENT_SAMPLE = 512
ACTIVE_SAMPLES = SAMPLE_COUNT - ALIGNMENT_SAMPLE
EVALUATION_MIN_HZ = 120.0
EVALUATION_MAX_HZ = 18_000.0

NOISE_BAND_COUNT = 96
NOISE_LOOP_SAMPLES = 16_384
NOISE_SEED = 20_260_831
RESIDUAL_ATOM_COUNT = 8
RESIDUAL_FRAME_HOP = 128
RESIDUAL_SCALE = 0.0125

MODAL_FREQUENCIES_HZ = np.geomspace(360.0, 12_500.0, 16).tolist()
MODAL_DAMPING_PER_SECOND = np.linspace(4.0, 42.0, 16).tolist()

GRID_SIZE = 7
CONTEXT_COUNT = 24
POSITION_FOURIER_BANDS = (1.0, 2.0, 4.0)
FIELD_CONFIG = {
    "grid_size": GRID_SIZE,
    "point_count": GRID_SIZE * GRID_SIZE,
    "context_count": CONTEXT_COUNT,
    "query_count": GRID_SIZE * GRID_SIZE - CONTEXT_COUNT,
    "fourier_bands": list(POSITION_FOURIER_BANDS),
    "hidden_width": 64,
    "hidden_layers": 2,
    "activation": "silu",
    "optimizer": "adamw",
    "learning_rate": 0.003,
    "weight_decay": 1.0e-6,
    "updates": 2_500,
    "seed": 20_260_831,
    "target": "eight_nonnegative_time_varying_residual_atom_weights",
    "normalization": "context_only_per_component_mean_and_std_plus_1e-6",
}

MAXIMUM_SHARED_BYTES = 4 * 1024 * 1024
MAXIMUM_CONTACT_BYTES = 64 * 1024
GATES = {
    "maximum_record_waveform_nrmse": 0.01,
    "maximum_neural_to_nearest_latent_rmse_ratio": 0.60,
    "maximum_neural_to_nearest_waveform_nrmse_ratio": 0.65,
    "maximum_query_envelope_rmse": 0.20,
    "maximum_median_query_log_spectrum_rmse_db": 4.0,
}


class V9Error(RuntimeError):
    """The frozen V9 synthetic boundary was violated."""


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


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise V9Error("V9 output must stay outside the repository")
    if resolved.exists():
        raise V9Error("V9 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def implementation_hashes(directory: Path) -> dict[str, str]:
    hashes = {}
    for key, filename in IMPLEMENTATION_FILES.items():
        path = directory / filename
        if not path.is_file():
            raise V9Error(f"missing V9 implementation file: {filename}")
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
                raise V9Error(
                    f"V9 environment changed for {key}: {observed[key]} != {expected}"
                )
    return observed

