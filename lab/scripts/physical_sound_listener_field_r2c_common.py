#!/usr/bin/env python3
"""Shared frozen R2C complex acoustic-field protocol and utilities."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import shutil
import struct
import subprocess
import sys
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import torch
from torch import nn

import physical_sound_listener_field_r2b_dense_preflight as r2b

MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c.manifest.v1"
)
PREFLIGHT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c-preflight.report.v1"
)
TRAINING_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c-training.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c-mlflow-lineage.v1"
)
EVALUATION_MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c-evaluation.manifest.v1"
)
EVALUATION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c-evaluation.report.v1"
)
REVISION = "green-goblet-fixed-impact-dense-complex-field-r2c-v1"
ACQUISITION_MANIFEST_SHA256 = (
    "487423a2973348968d87072e015bb895f4f3703ae4e1e45bcaf9a9d3f5e6b7b3"
)
R2B_REPORT_SHA256 = (
    "e3db03ac75efc2d7153b8c3f02172a47eb5ad384cb85ff489810f2f239d31696"
)
BLOCK_SHA256 = "381e958c53bf73b2aa62bf8dae874cf0167945a916c62514a9d2ab81e3c5188e"
R2B_REPORT_SCHEMA = r2b.REPORT_SCHEMA
ACQUISITION_SCHEMA = r2b.MANIFEST_SCHEMA
SEED = 2_608_300_203
STEPS = 8_000
LEARNING_RATE = 1.0e-4
WEIGHT_DECAY = 1.0e-6
ADAM_BETAS = (0.9, 0.999)
ADAM_EPSILON = 1.0e-8
GRADIENT_CLIP_NORM = 1.0
CONTEXT_BATCH_ROWS = 64
TF_BATCH_SIZE = 1_024
UNIFORM_TF_SAMPLES = 768
SALIENT_TF_SAMPLES = TF_BATCH_SIZE - UNIFORM_TF_SAMPLES
SALIENT_TF_COUNT = 8_192
SPATIAL_HIDDEN_WIDTH = 192
SPATIAL_HIDDEN_LAYERS = 3
TF_HIDDEN_WIDTH = 256
TF_HIDDEN_LAYERS = 4
LATENT_WIDTH = 96
FIRST_OMEGA = 30.0
HIDDEN_OMEGA = 1.0
DATA_COMPLEX_WEIGHT = 1.0
DATA_LOG_MAGNITUDE_WEIGHT = 1.0
HELMHOLTZ_WEIGHT = 1.0e-4
SPEED_OF_SOUND_METRES_PER_SECOND = 343.0
PHYSICS_MIN_HZ = 93.75
PHYSICS_MAX_HZ = 12_000.0
PHYSICS_BATCH_POSITIONS = 24
PHYSICS_BATCH_TF = 128
PHYSICS_FINITE_DIFFERENCE_METRES = 0.005
PHYSICS_CHARACTERISTIC_LENGTH_METRES = 0.13
PREDICTION_TF_CHUNK = 4_096
LOSS_EPSILON = 1.0e-8
LOG_INTERVAL = 100
QUERY_ANGLES = (40, 100, 160)
CONTEXT_ANGLES = (0, 20, 60, 80, 120, 140, 180)
COLLOCATION_ANGLE_PAIRS = ((0, 20), (20, 60), (60, 80), (80, 120), (120, 140), (140, 180))
CANDIDATES = (
    {"candidate_id": "dense_complex_field_data_only_v1", "helmholtz_weight": 0.0},
    {
        "candidate_id": "dense_complex_field_helmholtz_v1",
        "helmholtz_weight": HELMHOLTZ_WEIGHT,
    },
)
PRIMARY_ENDPOINTS = r2b.PRIMARY_ENDPOINTS
METRIC_SOURCE_PATHS = r2b.METRIC_SOURCE_PATHS


class R2CError(RuntimeError):
    """The frozen R2C boundary failed closed."""


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def read_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise R2CError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise R2CError(f"{label} must contain one JSON object")
    return data, value


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise R2CError(f"{label} must be an external file: {resolved}")
    return resolved


def require_ref(value: Any, label: str) -> dict[str, Any]:
    if (
        not isinstance(value, dict)
        or set(value) not in ({"path", "sha256"}, {"path", "sha256", "byte_count"})
        or not isinstance(value.get("path"), str)
        or not isinstance(value.get("sha256"), str)
        or len(value["sha256"]) != 64
        or any(character not in "0123456789abcdef" for character in value["sha256"])
    ):
        raise R2CError(f"invalid {label} reference")
    if "byte_count" in value and (
        not isinstance(value["byte_count"], int) or value["byte_count"] < 0
    ):
        raise R2CError(f"invalid {label} byte count")
    return value


def file_ref(path: Path, *, relative_to: Path | None = None) -> dict[str, Any]:
    target = path if relative_to is None else path.relative_to(relative_to)
    return {
        "path": str(target),
        "sha256": sha256_file(path),
        "byte_count": path.stat().st_size,
    }


def resolve_ref(
    root: Path, directory: Path, value: Any, label: str
) -> tuple[Path, bytes]:
    reference = require_ref(value, label)
    unresolved = Path(reference["path"])
    path = unresolved if unresolved.is_absolute() else directory / unresolved
    path = external_file(root, path, label)
    if "byte_count" in reference and path.stat().st_size != reference["byte_count"]:
        raise R2CError(f"{label} byte count changed")
    data = path.read_bytes()
    if sha256_bytes(data) != reference["sha256"]:
        raise R2CError(f"{label} hash changed")
    return path, data


def prepare_output(root: Path, argument: Path, label: str) -> tuple[Path, Path]:
    unresolved = argument if argument.is_absolute() else root / argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise R2CError(f"{label} must be a new external path: {output}")
    staging = parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        raise R2CError(f"{label} staging path exists: {staging}")
    staging.mkdir()
    return output, staging


def publish_staging(staging: Path, output: Path) -> None:
    staging.rename(output)


def discard_staging(staging: Path) -> None:
    shutil.rmtree(staging, ignore_errors=True)


def source_records(root: Path, relative_paths: list[str]) -> list[dict[str, Any]]:
    records = []
    for relative in sorted(relative_paths):
        path = (root / relative).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file():
            raise R2CError(f"implementation source escaped repository: {relative}")
        records.append(file_ref(path, relative_to=root))
    return records


def nvidia_driver_version() -> str:
    completed = subprocess.run(
        [
            "nvidia-smi",
            "--query-gpu=driver_version",
            "--format=csv,noheader",
            "--id=0",
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0 or not completed.stdout.strip():
        raise R2CError("R2C requires the frozen CUDA GPU environment")
    return completed.stdout.strip().splitlines()[0]


def environment_profile() -> dict[str, Any]:
    if not torch.cuda.is_available() or torch.cuda.device_count() != 1:
        raise R2CError("R2C requires exactly one visible CUDA device")
    properties = torch.cuda.get_device_properties(0)
    return {
        "python": platform.python_version(),
        "numpy": np.__version__,
        "torch": torch.__version__,
        "mlflow": mlflow.__version__,
        "device": "cuda:0",
        "device_name": properties.name,
        "compute_capability": list(torch.cuda.get_device_capability(0)),
        "gpu_total_memory_bytes": properties.total_memory,
        "cuda_runtime": torch.version.cuda,
        "nvidia_driver": nvidia_driver_version(),
        "dtype": "float32",
        "torch_threads": 1,
        "deterministic_algorithms": True,
        "cublas_workspace_config": ":4096:8",
        "allow_tf32": False,
        "float32_matmul_precision": "highest",
    }


def configure_determinism() -> torch.device:
    os.environ["CUBLAS_WORKSPACE_CONFIG"] = ":4096:8"
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    torch.use_deterministic_algorithms(True)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    torch.backends.cudnn.benchmark = False
    torch.backends.cudnn.deterministic = True
    torch.set_float32_matmul_precision("highest")
    torch.manual_seed(SEED)
    torch.cuda.manual_seed_all(SEED)
    np.random.seed(SEED & 0xFFFF_FFFF)
    device = torch.device("cuda:0")
    torch.cuda.set_device(device)
    return device


def candidate_profile() -> dict[str, Any]:
    return {
        "profile_id": "dense-complex-separable-siren-rank96-v1",
        "revision": REVISION,
        "seed": SEED,
        "model": {
            "family": "separable_coordinate_time_frequency_siren",
            "spatial_input": "listener_minus_published_impact_metres",
            "time_frequency_input": ["frame_time_seconds", "frequency_hz"],
            "output": ["pressure_real", "pressure_imaginary"],
            "spatial_hidden_width": SPATIAL_HIDDEN_WIDTH,
            "spatial_hidden_layers": SPATIAL_HIDDEN_LAYERS,
            "time_frequency_hidden_width": TF_HIDDEN_WIDTH,
            "time_frequency_hidden_layers": TF_HIDDEN_LAYERS,
            "latent_width": LATENT_WIDTH,
            "activation": "sine",
            "first_omega": FIRST_OMEGA,
            "hidden_omega": HIDDEN_OMEGA,
            "spatial_normalization": "context_only_axis_midpoint_and_half_range",
            "time_normalization": "zero_to_last_stft_frame_mapped_to_minus1_plus1",
            "frequency_normalization": "zero_to_nyquist_mapped_to_minus1_plus1",
            "pressure_scale": "context_complex_rms_from_preflight",
        },
        "training": {
            "optimizer": "torch_adamw_fixed_steps",
            "steps": STEPS,
            "learning_rate": LEARNING_RATE,
            "weight_decay": WEIGHT_DECAY,
            "betas": list(ADAM_BETAS),
            "epsilon": ADAM_EPSILON,
            "gradient_clip_norm": GRADIENT_CLIP_NORM,
            "context_batch_rows": CONTEXT_BATCH_ROWS,
            "time_frequency_batch_size": TF_BATCH_SIZE,
            "uniform_time_frequency_samples": UNIFORM_TF_SAMPLES,
            "salient_time_frequency_samples": SALIENT_TF_SAMPLES,
            "salient_time_frequency_count": SALIENT_TF_COUNT,
            "checkpoint": "final_step_only_no_query_or_early_stopping",
        },
        "data_loss": {
            "complex_l1_weight": DATA_COMPLEX_WEIGHT,
            "log_magnitude_l1_weight": DATA_LOG_MAGNITUDE_WEIGHT,
            "normalization": "context_mean_complex_magnitude",
            "epsilon": LOSS_EPSILON,
        },
        "physics_ablation": {
            "candidate_weights": {
                candidate["candidate_id"]: candidate["helmholtz_weight"]
                for candidate in CANDIDATES
            },
            "equation": "laplacian_p_plus_(2pi_frequency_over_343)^2_p_equals_zero",
            "speed_of_sound_metres_per_second": SPEED_OF_SOUND_METRES_PER_SECOND,
            "frequency_band_hz": [PHYSICS_MIN_HZ, PHYSICS_MAX_HZ],
            "collocation": "cartesian_midpoints_between_adjacent_context_azimuth_planes",
            "position_batch": PHYSICS_BATCH_POSITIONS,
            "time_frequency_batch": PHYSICS_BATCH_TF,
            "laplacian": "differentiable_second_order_central_difference",
            "finite_difference_metres": PHYSICS_FINITE_DIFFERENCE_METRES,
            "residual_normalization_length_metres": PHYSICS_CHARACTERISTIC_LENGTH_METRES,
            "query_audio_used": False,
        },
        "candidate_ladder": list(CANDIDATES),
        "selection": (
            "strictly_below_all_three_frozen_controls_on_all_five_primary_"
            "endpoints_else_reject_complex_field"
        ),
        "repetition": "two_independent_runs_require_byte_identical_training_report",
        "query_audio_available_to_training": False,
        "method_holdout_or_shadow_available": False,
    }


class SineLayer(nn.Module):
    def __init__(
        self, input_count: int, output_count: int, *, first: bool, omega: float
    ) -> None:
        super().__init__()
        self.linear = nn.Linear(input_count, output_count)
        self.omega = omega
        with torch.no_grad():
            bound = (
                1.0 / input_count
                if first
                else math.sqrt(6.0 / input_count) / omega
            )
            self.linear.weight.uniform_(-bound, bound)
            self.linear.bias.uniform_(-bound, bound)

    def forward(self, values: torch.Tensor) -> torch.Tensor:
        return torch.sin(self.omega * self.linear(values))


def siren_stack(
    input_count: int, hidden_count: int, hidden_layers: int
) -> nn.Sequential:
    layers: list[nn.Module] = [
        SineLayer(input_count, hidden_count, first=True, omega=FIRST_OMEGA)
    ]
    for _ in range(hidden_layers - 1):
        layers.append(
            SineLayer(hidden_count, hidden_count, first=False, omega=HIDDEN_OMEGA)
        )
    return nn.Sequential(*layers)


class SeparableComplexField(nn.Module):
    """Factorized coordinate field with reusable spatial and TF embeddings."""

    def __init__(
        self,
        spatial_center: np.ndarray,
        spatial_scale: np.ndarray,
        last_frame_time_seconds: float,
    ) -> None:
        super().__init__()
        self.register_buffer(
            "spatial_center",
            torch.as_tensor(spatial_center, dtype=torch.float32),
        )
        self.register_buffer(
            "spatial_scale",
            torch.as_tensor(spatial_scale, dtype=torch.float32),
        )
        self.register_buffer(
            "last_frame_time_seconds",
            torch.tensor(last_frame_time_seconds, dtype=torch.float32),
        )
        self.spatial_hidden = siren_stack(
            3, SPATIAL_HIDDEN_WIDTH, SPATIAL_HIDDEN_LAYERS
        )
        self.spatial_output = nn.Linear(SPATIAL_HIDDEN_WIDTH, LATENT_WIDTH)
        self.tf_hidden = siren_stack(2, TF_HIDDEN_WIDTH, TF_HIDDEN_LAYERS)
        self.tf_output = nn.Linear(TF_HIDDEN_WIDTH, 2 * (LATENT_WIDTH + 1))
        with torch.no_grad():
            spatial_bound = math.sqrt(6.0 / SPATIAL_HIDDEN_WIDTH)
            self.spatial_output.weight.uniform_(-spatial_bound, spatial_bound)
            self.spatial_output.bias.uniform_(-spatial_bound, spatial_bound)
            tf_bound = math.sqrt(6.0 / TF_HIDDEN_WIDTH)
            self.tf_output.weight.uniform_(-tf_bound, tf_bound)
            self.tf_output.bias.uniform_(-tf_bound, tf_bound)

    def spatial_embedding(
        self, listener_metres: torch.Tensor, impact_metres: torch.Tensor
    ) -> torch.Tensor:
        relative = listener_metres - impact_metres
        normalized = (relative - self.spatial_center) / self.spatial_scale
        return self.spatial_output(self.spatial_hidden(normalized))

    def time_frequency_embedding(
        self, time_seconds: torch.Tensor, frequency_hz: torch.Tensor
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
        time = 2.0 * time_seconds / self.last_frame_time_seconds - 1.0
        frequency = 2.0 * frequency_hz / (r2b.SAMPLE_RATE_HZ / 2.0) - 1.0
        values = torch.stack((time, frequency), dim=-1)
        output = self.tf_output(self.tf_hidden(values))
        real = output[:, :LATENT_WIDTH]
        imag = output[:, LATENT_WIDTH : 2 * LATENT_WIDTH]
        real_bias = output[:, 2 * LATENT_WIDTH]
        imag_bias = output[:, 2 * LATENT_WIDTH + 1]
        return real, imag, real_bias, imag_bias

    def forward(
        self,
        listener_metres: torch.Tensor,
        impact_metres: torch.Tensor,
        time_seconds: torch.Tensor,
        frequency_hz: torch.Tensor,
    ) -> torch.Tensor:
        spatial = self.spatial_embedding(listener_metres, impact_metres)
        real, imag, real_bias, imag_bias = self.time_frequency_embedding(
            time_seconds, frequency_hz
        )
        scale = math.sqrt(LATENT_WIDTH)
        pressure_real = spatial @ real.T / scale + real_bias[None, :]
        pressure_imag = spatial @ imag.T / scale + imag_bias[None, :]
        return torch.stack((pressure_real, pressure_imag), dim=-1)


def model_parameter_count(model: nn.Module) -> int:
    return sum(parameter.numel() for parameter in model.parameters())


def data_losses(
    prediction: torch.Tensor, target: torch.Tensor, mean_abs_scaled: float
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    difference = prediction - target
    complex_error = torch.sqrt(torch.sum(difference * difference, dim=-1) + LOSS_EPSILON**2)
    complex_l1 = torch.mean(complex_error) / mean_abs_scaled
    prediction_magnitude = torch.sqrt(
        torch.sum(prediction * prediction, dim=-1) + LOSS_EPSILON**2
    )
    target_magnitude = torch.sqrt(
        torch.sum(target * target, dim=-1) + LOSS_EPSILON**2
    )
    log_l1 = torch.mean(
        torch.abs(
            torch.log1p(prediction_magnitude / mean_abs_scaled)
            - torch.log1p(target_magnitude / mean_abs_scaled)
        )
    )
    total = DATA_COMPLEX_WEIGHT * complex_l1 + DATA_LOG_MAGNITUDE_WEIGHT * log_l1
    return total, complex_l1, log_l1


def tf_coordinates(indices: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    frames = indices // (r2b.FFT_LENGTH // 2 + 1)
    bins = indices % (r2b.FFT_LENGTH // 2 + 1)
    times = frames.astype(np.float32) * (r2b.HOP_LENGTH / r2b.SAMPLE_RATE_HZ)
    frequencies = bins.astype(np.float32) * (
        r2b.SAMPLE_RATE_HZ / r2b.FFT_LENGTH
    )
    return times, frequencies


def spatial_normalization(
    context_rows: list[dict[str, Any]], impact: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    listener = np.asarray(
        [row["listener_position_metres"] for row in context_rows], dtype=np.float64
    )
    relative = listener - impact[None, :]
    minimum = np.min(relative, axis=0)
    maximum = np.max(relative, axis=0)
    center = 0.5 * (minimum + maximum)
    scale = np.maximum(0.5 * (maximum - minimum), 1.0e-6)
    return center, scale


def collocation_positions(rows: list[dict[str, Any]]) -> np.ndarray:
    by_key = {
        (
            row["azimuth_degrees"],
            row["gantry_distance_offset_millimetres"],
            row["microphone_id"],
        ): np.asarray(row["listener_position_metres"], dtype=np.float64)
        for row in rows
        if row["split_role"] == "context"
    }
    points = []
    for lower, upper in COLLOCATION_ANGLE_PAIRS:
        for distance in (0, 333, 666, 1_000):
            for microphone in range(r2b.MICROPHONES_PER_COLUMN):
                left = by_key[(lower, distance, microphone)]
                right = by_key[(upper, distance, microphone)]
                points.append(0.5 * (left + right))
    result = np.asarray(points, dtype=np.float64)
    if result.shape != (360, 3) or len(np.unique(result, axis=0)) != 360:
        raise R2CError("R2C collocation midpoint coverage changed")
    return result


def validate_acquisition_manifest(
    root: Path, path: Path, data: bytes, manifest: dict[str, Any]
) -> tuple[Path, list[dict[str, Any]]]:
    if sha256_bytes(data) != ACQUISITION_MANIFEST_SHA256:
        raise R2CError("R2C acquisition manifest identity changed")
    if (
        manifest.get("schema") != ACQUISITION_SCHEMA
        or manifest.get("profile") != r2b.PROFILE
        or manifest.get("row_count") != r2b.ROW_COUNT
        or manifest.get("column_count") != r2b.COLUMN_COUNT
        or manifest.get("context_row_count") != r2b.CONTEXT_ROWS
        or manifest.get("query_row_count") != r2b.QUERY_ROWS
        or manifest.get("representation_preflight") != r2b.expected_representation()
    ):
        raise R2CError("R2C acquisition manifest semantics changed")
    block = require_ref(manifest.get("block_payload"), "dense block")
    if block["sha256"] != BLOCK_SHA256:
        raise R2CError("R2C dense block identity changed")
    unresolved = Path(block["path"])
    block_path = unresolved if unresolved.is_absolute() else path.parent / unresolved
    block_path = external_file(root, block_path, "R2C dense block")
    if block_path.stat().st_size != block.get("byte_count"):
        raise R2CError("R2C dense block byte count changed")
    rows = manifest.get("rows")
    if not isinstance(rows, list) or len(rows) != r2b.ROW_COUNT:
        raise R2CError("R2C acquisition rows changed")
    for index, row in enumerate(rows):
        role = "query" if row.get("azimuth_degrees") in QUERY_ANGLES else "context"
        if row.get("row_index") != index or row.get("split_role") != role:
            raise R2CError(f"R2C split identity changed at row {index}")
    return block_path, rows


def validate_r2b_report(
    acquisition_bytes: bytes, data: bytes, report: dict[str, Any]
) -> None:
    if sha256_bytes(data) != R2B_REPORT_SHA256:
        raise R2CError("R2B preflight report identity changed")
    if (
        report.get("schema") != R2B_REPORT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "ReadyForComplexFieldTraining"
        or report.get("manifest_sha256") != sha256_bytes(acquisition_bytes)
        or report.get("block_payload_sha256") != BLOCK_SHA256
        or report.get("optimizer_steps") != 0
        or report.get("training_authorized") is not True
        or report.get("quality_or_admission_authorized") is not False
        or report.get("future_candidate_protocol") != r2b.future_candidate_protocol()
        or report.get("split", {}).get("query_rows_used_for_normalization") != 0
        or report.get("split", {}).get("query_rows_cached_for_candidate_fit") != 0
        or report.get("split", {}).get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise R2CError("R2B preflight semantics changed")


def encode_wav(samples: np.ndarray) -> bytes:
    if samples.shape != (r2b.SAMPLE_COUNT,) or not np.isfinite(samples).all():
        raise R2CError("R2C prediction must be finite source-length mono audio")
    peak = float(np.max(np.abs(samples)))
    if peak >= 1.0:
        raise R2CError(f"R2C prediction peak {peak} is not cookable")
    pcm = np.clip(np.rint(samples * 32768.0), -32768, 32767).astype("<i2")
    payload = pcm.tobytes()
    return b"".join(
        (
            b"RIFF",
            struct.pack("<I", 36 + len(payload)),
            b"WAVEfmt ",
            struct.pack(
                "<IHHIIHH",
                16,
                1,
                1,
                r2b.SAMPLE_RATE_HZ,
                r2b.SAMPLE_RATE_HZ * 2,
                2,
                16,
            ),
            b"data",
            struct.pack("<I", len(payload)),
            payload,
        )
    )


def write_checkpoint(output: Path, candidate_id: str, model: nn.Module) -> dict[str, Any]:
    directory = output / "checkpoint"
    directory.mkdir()
    payload = bytearray()
    parameters = []
    offset = 0
    for name, tensor in sorted(model.state_dict().items()):
        array = tensor.detach().cpu().numpy().astype("<f4", copy=False)
        data = array.tobytes(order="C")
        parameters.append(
            {
                "name": name,
                "shape": list(array.shape),
                "dtype": "float32le",
                "offset_bytes": offset,
                "byte_count": len(data),
            }
        )
        payload.extend(data)
        offset += len(data)
    weights = directory / "weights.f32le"
    weights.write_bytes(payload)
    descriptor = {
        "schema": "nextengine.experimental-dense-complex-field-checkpoint.v1",
        "revision": REVISION,
        "candidate_id": candidate_id,
        "weights_sha256": sha256_bytes(payload),
        "weights_byte_count": len(payload),
        "parameters": parameters,
    }
    descriptor_bytes = canonical_json(descriptor)
    descriptor_path = directory / "checkpoint.json"
    descriptor_path.write_bytes(descriptor_bytes)
    return {
        "descriptor_file": str(descriptor_path.relative_to(output)),
        "descriptor_sha256": sha256_bytes(descriptor_bytes),
        "weights_file": str(weights.relative_to(output)),
        "weights_sha256": descriptor["weights_sha256"],
        "weights_byte_count": len(payload),
    }


def normalized_metric_sources(root: Path) -> list[dict[str, Any]]:
    return source_records(root, list(METRIC_SOURCE_PATHS))


def percentile_nearest(values: list[float], percentile: int) -> float:
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered) / 100.0))
    return ordered[rank - 1]


def aggregate_metric_rows(rows: list[dict[str, Any]]) -> dict[str, float]:
    level = [row["absolute_rms_level_error_db"] for row in rows]
    spectrum = [
        row["gain_matched_multiresolution_log_spectrum_rmse_db"] for row in rows
    ]
    waveform = [row["normalized_waveform_rmse_db"] for row in rows]
    return {
        "mean_absolute_rms_level_error_db": sum(level) / len(level),
        "p95_absolute_rms_level_error_db": percentile_nearest(level, 95),
        "mean_gain_matched_multiresolution_log_spectrum_rmse_db": sum(spectrum)
        / len(spectrum),
        "p95_gain_matched_multiresolution_log_spectrum_rmse_db": percentile_nearest(
            spectrum, 95
        ),
        "mean_normalized_waveform_rmse_db": sum(waveform) / len(waveform),
    }


def root_from_script(script: Path) -> Path:
    root = script.resolve(strict=True).parents[2]
    if not (root / "Cargo.toml").is_file():
        raise R2CError("R2C runner could not locate the workspace root")
    return root


def exact_candidate(candidate_id: str) -> dict[str, Any]:
    for candidate in CANDIDATES:
        if candidate["candidate_id"] == candidate_id:
            return candidate
    raise R2CError(f"unknown R2C candidate: {candidate_id}")


def emit_summary(value: dict[str, Any], keys: tuple[str, ...]) -> None:
    print(canonical_json({key: value[key] for key in keys}).decode(), end="")


if __name__ == "__main__":
    sys.exit("physical_sound_listener_field_r2c_common.py is a library module")
