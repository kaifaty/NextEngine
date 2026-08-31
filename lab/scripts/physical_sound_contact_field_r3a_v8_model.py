#!/usr/bin/env python3
"""Explicit modal renderer and neural mode-shape field for R3A V8 controls."""

from __future__ import annotations

import hashlib
import math
from typing import Any

import numpy as np
import torch
from torch import nn

import physical_sound_contact_field_r3a_v8_common as common


def configure_torch() -> None:
    torch.use_deterministic_algorithms(True)
    torch.set_num_threads(1)
    try:
        torch.set_num_interop_threads(1)
    except RuntimeError:
        pass


def render_modal(
    frequencies_hz: torch.Tensor,
    damping_per_second: torch.Tensor,
    gains: torch.Tensor,
    time_seconds: torch.Tensor,
) -> torch.Tensor:
    if not (
        frequencies_hz.ndim
        == damping_per_second.ndim
        == gains.ndim
        == 1
    ):
        raise common.V8Error("modal parameters must be one-dimensional")
    if not (
        frequencies_hz.shape == damping_per_second.shape == gains.shape
    ):
        raise common.V8Error("modal parameter shapes differ")
    phase = 2.0 * math.pi * frequencies_hz[:, None] * time_seconds[None, :]
    envelope = torch.exp(-damping_per_second[:, None] * time_seconds[None, :])
    return (gains[:, None] * envelope * torch.sin(phase)).sum(dim=0)


def tensor_hash(values: dict[str, torch.Tensor]) -> str:
    digest = hashlib.sha256()
    for name in sorted(values):
        value = values[name].detach().cpu().contiguous()
        digest.update(name.encode())
        digest.update(str(value.dtype).encode())
        digest.update(np.asarray(value.shape, dtype=np.int64).tobytes())
        digest.update(value.numpy().tobytes())
    return digest.hexdigest()


def modal_recovery_control() -> dict[str, Any]:
    configure_torch()
    config = common.MODAL_CONTROL_CONFIG
    dtype = torch.float64
    time = torch.arange(config["sample_count"], dtype=dtype) / config[
        "sample_rate_hz"
    ]
    target_frequency = torch.tensor(config["frequencies_hz"], dtype=dtype)
    target_damping = torch.tensor(config["damping_per_second"], dtype=dtype)
    target_gain = torch.tensor(config["gains"], dtype=dtype)
    target = render_modal(target_frequency, target_damping, target_gain, time)

    log_frequency = nn.Parameter(
        torch.log(
            target_frequency
            * torch.tensor(config["frequency_initial_factors"], dtype=dtype)
        )
    )
    log_damping = nn.Parameter(
        torch.log(
            target_damping
            * torch.tensor(config["damping_initial_factors"], dtype=dtype)
        )
    )
    gain = nn.Parameter(
        target_gain * torch.tensor(config["gain_initial_factors"], dtype=dtype)
    )
    optimizer = torch.optim.Adam(
        [
            {
                "params": [log_frequency],
                "lr": config["frequency_learning_rate"],
            },
            {"params": [log_damping], "lr": config["damping_learning_rate"]},
            {"params": [gain], "lr": config["gain_learning_rate"]},
        ]
    )
    for _ in range(config["updates"]):
        optimizer.zero_grad(set_to_none=True)
        prediction = render_modal(
            log_frequency.exp(), log_damping.exp(), gain, time
        )
        loss = torch.mean((prediction - target) ** 2)
        loss.backward()
        optimizer.step()

    with torch.inference_mode():
        frequency = log_frequency.exp()
        damping = log_damping.exp()
        prediction = render_modal(frequency, damping, gain, time)
        waveform_mse = torch.mean((prediction - target) ** 2)
        frequency_error_cents = torch.abs(
            1_200.0 * torch.log2(frequency / target_frequency)
        )
        relative_damping_error = torch.abs(damping / target_damping - 1.0)
        relative_gain_error = torch.abs((gain - target_gain) / target_gain)
        finite = bool(
            torch.isfinite(
                torch.cat(
                    [
                        frequency,
                        damping,
                        gain,
                        waveform_mse.reshape(1),
                    ]
                )
            ).all()
        )
        maximum_frequency_error = float(torch.max(frequency_error_cents))
        maximum_damping_error = float(torch.max(relative_damping_error))
        maximum_gain_error = float(torch.max(relative_gain_error))
        mse = float(waveform_mse)
        passed = (
            finite
            and maximum_frequency_error
            <= config["maximum_frequency_error_cents"]
            and maximum_damping_error
            <= config["maximum_relative_damping_error"]
            and maximum_gain_error <= config["maximum_relative_gain_error"]
            and mse <= config["maximum_waveform_mse"]
        )
        return {
            "passed": passed,
            "finite": finite,
            "updates": config["updates"],
            "waveform_mse": mse,
            "maximum_frequency_error_cents": maximum_frequency_error,
            "maximum_relative_damping_error": maximum_damping_error,
            "maximum_relative_gain_error": maximum_gain_error,
            "recovered_frequencies_hz": frequency.tolist(),
            "recovered_damping_per_second": damping.tolist(),
            "recovered_gains": gain.tolist(),
            "parameter_sha256": tensor_hash(
                {"frequency": frequency, "damping": damping, "gain": gain}
            ),
            "prediction_sha256": hashlib.sha256(
                prediction.cpu().numpy().tobytes()
            ).hexdigest(),
        }


def farthest_point_indices(coordinates: np.ndarray, count: int) -> np.ndarray:
    value = np.asarray(coordinates, dtype=np.float64)
    if value.ndim != 2 or value.shape[1] != 3:
        raise common.V8Error("farthest-point coordinates must have shape [N, 3]")
    if count <= 0 or count > value.shape[0]:
        raise common.V8Error("farthest-point count is invalid")
    selected = np.empty(count, dtype=np.int64)
    selected[0] = int(np.lexsort((value[:, 2], value[:, 1], value[:, 0]))[0])
    distance = np.sum((value - value[selected[0]]) ** 2, axis=1)
    distance[selected[0]] = -1.0
    for index in range(1, count):
        selected[index] = int(np.argmax(distance))
        candidate = np.sum((value - value[selected[index]]) ** 2, axis=1)
        distance = np.minimum(distance, candidate)
        distance[selected[: index + 1]] = -1.0
    return selected


def encode_position(coordinates: np.ndarray, surface: np.ndarray) -> np.ndarray:
    normalized = (
        np.asarray(coordinates, dtype=np.float32)
        / np.float32(common.FIELD_CONFIG["coordinate_extent"])
        * np.float32(2.0)
        - np.float32(1.0)
    )
    parts = [normalized]
    for band in common.FIELD_CONFIG["fourier_bands"]:
        phase = np.float32(math.pi * band) * normalized
        parts.extend((np.sin(phase), np.cos(phase)))
    parts.append(np.asarray(surface, dtype=np.float32))
    return np.concatenate(parts, axis=1).astype(np.float32, copy=False)


class ModeShapeField(nn.Module):
    def __init__(self, input_dimension: int, output_dimension: int) -> None:
        super().__init__()
        width = common.FIELD_CONFIG["hidden_width"]
        self.layers = nn.Sequential(
            nn.Linear(input_dimension, width),
            nn.SiLU(),
            nn.Linear(width, width),
            nn.SiLU(),
            nn.Linear(width, output_dimension),
        )

    def forward(self, value: torch.Tensor) -> torch.Tensor:
        return self.layers(value)


def model_hash(model: nn.Module) -> str:
    return tensor_hash(dict(model.named_parameters()))


def train_mode_shape_field(source: dict[str, Any]) -> dict[str, Any]:
    configure_torch()
    config = common.FIELD_CONFIG
    coordinates = source["coords"]
    surface = source["surface"]
    target_raw = source["features"].reshape(coordinates.shape[0], -1)
    context_count = max(
        config["minimum_context_points"],
        int(math.ceil(config["context_fraction"] * coordinates.shape[0])),
    )
    context_indices = farthest_point_indices(coordinates, context_count)
    query_mask = np.ones(coordinates.shape[0], dtype=bool)
    query_mask[context_indices] = False
    query_indices = np.flatnonzero(query_mask)
    if query_indices.size == 0:
        raise common.V8Error("V8 field split has no query points")

    scale = np.sqrt(
        np.mean(np.square(target_raw[context_indices]), axis=0, keepdims=True)
    ) + np.float32(1.0e-6)
    target = (target_raw / scale).astype(np.float32, copy=False)
    encoded = encode_position(coordinates, surface)
    distance_features = np.concatenate(
        [
            coordinates.astype(np.float32)
            / np.float32(config["coordinate_extent"]),
            surface.astype(np.float32),
        ],
        axis=1,
    )

    context_x = torch.from_numpy(encoded[context_indices])
    context_y = torch.from_numpy(target[context_indices])
    query_x = torch.from_numpy(encoded[query_indices])
    query_y = torch.from_numpy(target[query_indices])

    torch.manual_seed(config["seed"])
    model = ModeShapeField(encoded.shape[1], target.shape[1])
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config["learning_rate"],
        weight_decay=config["weight_decay"],
    )
    for _ in range(config["updates"]):
        optimizer.zero_grad(set_to_none=True)
        prediction = model(context_x)
        loss = torch.mean((prediction - context_y) ** 2)
        loss.backward()
        optimizer.step()

    with torch.inference_mode():
        query_prediction = model(query_x)
        neural_rmse = float(torch.sqrt(torch.mean((query_prediction - query_y) ** 2)))
        constant = torch.mean(context_y, dim=0, keepdim=True)
        constant_rmse = float(torch.sqrt(torch.mean((query_y - constant) ** 2)))
        context_distance = distance_features[context_indices]
        query_distance = distance_features[query_indices]
        squared_distance = np.sum(
            np.square(query_distance[:, None, :] - context_distance[None, :, :]),
            axis=2,
        )
        nearest_indices = np.argmin(squared_distance, axis=1)
        nearest = context_y[torch.from_numpy(nearest_indices)]
        nearest_rmse = float(torch.sqrt(torch.mean((query_y - nearest) ** 2)))
        prediction_std = float(torch.mean(torch.std(query_prediction, dim=0)))
        final_train_rmse = float(
            torch.sqrt(torch.mean((model(context_x) - context_y) ** 2))
        )
        finite = bool(
            torch.isfinite(query_prediction).all()
            and math.isfinite(neural_rmse)
            and math.isfinite(nearest_rmse)
            and nearest_rmse > 0.0
        )
        neural_to_nearest = neural_rmse / nearest_rmse
        neural_to_constant = neural_rmse / constant_rmse
        passed = (
            finite
            and prediction_std >= config["minimum_query_prediction_std"]
            and neural_to_nearest
            <= config["maximum_neural_to_nearest_rmse_ratio"]
            and neural_rmse < constant_rmse
        )
        return {
            "object_id": source["object_id"],
            "passed": passed,
            "finite": finite,
            "point_count": int(coordinates.shape[0]),
            "context_count": int(context_indices.size),
            "query_count": int(query_indices.size),
            "updates": config["updates"],
            "final_train_rmse": final_train_rmse,
            "neural_query_rmse": neural_rmse,
            "nearest_query_rmse": nearest_rmse,
            "constant_query_rmse": constant_rmse,
            "neural_to_nearest_rmse_ratio": neural_to_nearest,
            "neural_to_constant_rmse_ratio": neural_to_constant,
            "query_prediction_mean_component_std": prediction_std,
            "context_indices_sha256": hashlib.sha256(
                context_indices.tobytes()
            ).hexdigest(),
            "model_parameter_sha256": model_hash(model),
            "query_prediction_sha256": hashlib.sha256(
                query_prediction.numpy().tobytes()
            ).hexdigest(),
        }

