#!/usr/bin/env python3
"""Deterministic V9 modal/noise-band renderer and neural contact field."""

from __future__ import annotations

import hashlib
import math
import struct
from typing import Any

import numpy as np
import torch
from torch import nn

import physical_sound_contact_field_r3a_v9_common as common


def configure_torch() -> None:
    torch.use_deterministic_algorithms(True)
    torch.set_num_threads(1)
    try:
        torch.set_num_interop_threads(1)
    except RuntimeError:
        pass


def array_hash(value: np.ndarray, dtype: str = "<f8") -> str:
    payload = np.ascontiguousarray(value, dtype=dtype).tobytes()
    return hashlib.sha256(payload).hexdigest()


def tensor_hash(values: dict[str, torch.Tensor]) -> str:
    digest = hashlib.sha256()
    for name in sorted(values):
        value = values[name].detach().cpu().contiguous()
        digest.update(name.encode())
        digest.update(str(value.dtype).encode())
        digest.update(np.asarray(value.shape, dtype=np.int64).tobytes())
        digest.update(value.numpy().tobytes())
    return digest.hexdigest()


def grid_coordinates() -> np.ndarray:
    axis = np.linspace(0.0, 1.0, common.GRID_SIZE, dtype=np.float64)
    rows = []
    for x in axis:
        for y in axis:
            z = 0.12 + 0.08 * math.sin(math.pi * x) * math.sin(math.pi * y)
            rows.append((x, y, z))
    return np.asarray(rows, dtype=np.float64)


def farthest_point_indices(coordinates: np.ndarray, count: int) -> np.ndarray:
    value = np.asarray(coordinates, dtype=np.float64)
    if value.ndim != 2 or value.shape[1] != 3:
        raise common.V9Error("farthest-point coordinates must have shape [N, 3]")
    if count <= 0 or count >= value.shape[0]:
        raise common.V9Error("farthest-point count is invalid")
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


def band_centers_hz() -> np.ndarray:
    return np.geomspace(
        common.EVALUATION_MIN_HZ,
        common.EVALUATION_MAX_HZ,
        common.NOISE_BAND_COUNT,
        dtype=np.float64,
    )


def generate_noise_bands() -> np.ndarray:
    centers = band_centers_hz()
    log_centers = np.log(centers)
    boundaries = np.empty(centers.size + 1, dtype=np.float64)
    boundaries[1:-1] = 0.5 * (log_centers[:-1] + log_centers[1:])
    boundaries[0] = math.log(common.EVALUATION_MIN_HZ)
    boundaries[-1] = math.log(common.EVALUATION_MAX_HZ)
    frequencies = np.fft.rfftfreq(
        common.NOISE_LOOP_SAMPLES, 1.0 / common.SAMPLE_RATE_HZ
    )
    log_frequency = np.log(np.maximum(frequencies, common.EVALUATION_MIN_HZ))
    generator = np.random.Generator(np.random.PCG64(common.NOISE_SEED))
    rows = []
    for index, center in enumerate(log_centers):
        lower = boundaries[index]
        upper = boundaries[index + 1]
        magnitude = np.zeros_like(frequencies)
        left = (log_frequency >= lower) & (log_frequency <= center)
        right = (log_frequency > center) & (log_frequency <= upper)
        magnitude[left] = (log_frequency[left] - lower) / max(center - lower, 1e-12)
        magnitude[right] = (upper - log_frequency[right]) / max(upper - center, 1e-12)
        magnitude[(frequencies < common.EVALUATION_MIN_HZ)] = 0.0
        magnitude[(frequencies > common.EVALUATION_MAX_HZ)] = 0.0
        if not np.any(magnitude > 0.0):
            nearest = int(np.argmin(np.abs(frequencies - math.exp(center))))
            magnitude[nearest] = 1.0
        phase = generator.uniform(-math.pi, math.pi, magnitude.size)
        spectrum = magnitude * np.exp(1j * phase)
        spectrum[0] = 0.0
        spectrum[-1] = 0.0
        row = np.fft.irfft(spectrum, n=common.NOISE_LOOP_SAMPLES)
        rms = math.sqrt(float(np.mean(np.square(row))))
        if not math.isfinite(rms) or rms <= 0.0:
            raise common.V9Error(f"noise band {index} is invalid")
        rows.append(row / rms)
    result = np.ascontiguousarray(np.stack(rows), dtype=np.float64)
    if result.shape != (common.NOISE_BAND_COUNT, common.NOISE_LOOP_SAMPLES):
        raise common.V9Error("noise-band shape changed")
    return result


def residual_atoms() -> np.ndarray:
    frame_times = (
        np.arange(0, common.ACTIVE_SAMPLES, common.RESIDUAL_FRAME_HOP)
        / common.SAMPLE_RATE_HZ
    )
    band_axis = np.linspace(0.0, 1.0, common.NOISE_BAND_COUNT)
    rows = []
    for index in range(common.RESIDUAL_ATOM_COUNT):
        center = (index + 0.55) / common.RESIDUAL_ATOM_COUNT
        alternate = ((index * 3 + 2) % common.RESIDUAL_ATOM_COUNT + 0.5) / common.RESIDUAL_ATOM_COUNT
        width = 0.055 + 0.008 * (index % 3)
        spectral = np.exp(-0.5 * np.square((band_axis - center) / width))
        spectral += 0.28 * np.exp(
            -0.5 * np.square((band_axis - alternate) / (width * 1.35))
        )
        decay = 2.0 + 0.55 * index
        modulation = np.square(
            0.82 + 0.18 * np.cos(2.0 * math.pi * (0.7 + 0.13 * index) * frame_times)
        )
        attack = 1.0 - np.exp(-(45.0 + 4.0 * index) * frame_times)
        temporal = attack * np.exp(-decay * frame_times) * modulation
        atom = spectral[:, None] * temporal[None, :]
        atom /= max(float(np.max(atom)), 1.0e-30)
        rows.append(atom)
    return np.ascontiguousarray(np.stack(rows), dtype=np.float64)


def truth_latents(coordinates: np.ndarray) -> np.ndarray:
    value = np.asarray(coordinates, dtype=np.float64)
    x = value[:, 0]
    y = value[:, 1]
    u = 2.0 * x - 1.0
    v = 2.0 * y - 1.0
    latents = np.column_stack(
        [
            0.72 + 0.16 * u + 0.08 * v,
            0.66 - 0.12 * u + 0.14 * v,
            0.58 + 0.10 * u * v + 0.08 * np.square(u),
            0.62 + 0.13 * np.sin(math.pi * x) * np.cos(math.pi * y),
            0.64 + 0.09 * np.square(v) - 0.07 * u,
            0.57 + 0.08 * np.cos(math.pi * x) + 0.11 * v,
            0.52 + 0.15 * x * y,
            0.68 + 0.09 * np.sin(math.pi * (x + y)),
        ]
    )
    if np.min(latents) <= 0.0:
        raise common.V9Error("synthetic residual latent is not positive")
    return np.ascontiguousarray(latents, dtype=np.float64)


def truth_modal_gains(coordinates: np.ndarray) -> np.ndarray:
    value = np.asarray(coordinates, dtype=np.float64)
    x = value[:, 0, None]
    y = value[:, 1, None]
    mode = np.arange(len(common.MODAL_FREQUENCIES_HZ), dtype=np.float64)[None, :]
    amplitude = 0.032 / np.sqrt(1.0 + 0.12 * mode)
    amplitude = amplitude * (
        0.82
        + 0.12 * np.sin(math.pi * (1.0 + mode % 3.0) * x)
        + 0.08 * np.cos(math.pi * (1.0 + mode % 4.0) * y)
    )
    phase = 0.21 * mode + 0.8 * x - 0.55 * y
    return np.ascontiguousarray(
        np.stack([amplitude * np.cos(phase), amplitude * np.sin(phase)], axis=2),
        dtype=np.float64,
    )


def render_modal(gains: np.ndarray) -> np.ndarray:
    value = np.asarray(gains, dtype=np.float64)
    if value.shape != (len(common.MODAL_FREQUENCIES_HZ), 2):
        raise common.V9Error("modal gain shape changed")
    time = np.arange(common.ACTIVE_SAMPLES, dtype=np.float64) / common.SAMPLE_RATE_HZ
    result = np.zeros(common.SAMPLE_COUNT, dtype=np.float64)
    active = np.zeros(common.ACTIVE_SAMPLES, dtype=np.float64)
    for index, (frequency, damping) in enumerate(
        zip(common.MODAL_FREQUENCIES_HZ, common.MODAL_DAMPING_PER_SECOND, strict=True)
    ):
        phase = 2.0 * math.pi * frequency * time
        envelope = np.exp(-damping * time)
        active += envelope * (
            value[index, 0] * np.cos(phase) + value[index, 1] * np.sin(phase)
        )
    result[common.ALIGNMENT_SAMPLE :] = active
    return result


def render_residual(
    latent: np.ndarray, noise_bands: np.ndarray, atoms: np.ndarray
) -> np.ndarray:
    value = np.asarray(latent, dtype=np.float64)
    if value.shape != (common.RESIDUAL_ATOM_COUNT,):
        raise common.V9Error("residual latent shape changed")
    if np.any(value < 0.0):
        raise common.V9Error("residual latent must be nonnegative")
    amplitudes = np.tensordot(value, atoms, axes=(0, 0))
    frame_positions = np.arange(amplitudes.shape[1]) * common.RESIDUAL_FRAME_HOP
    sample_positions = np.arange(common.ACTIVE_SAMPLES)
    repeats = math.ceil(common.ACTIVE_SAMPLES / common.NOISE_LOOP_SAMPLES)
    looped = np.tile(noise_bands, (1, repeats))[:, : common.ACTIVE_SAMPLES]
    active = np.zeros(common.ACTIVE_SAMPLES, dtype=np.float64)
    for band_index in range(common.NOISE_BAND_COUNT):
        envelope = np.interp(
            sample_positions,
            frame_positions,
            amplitudes[band_index],
            left=amplitudes[band_index, 0],
            right=amplitudes[band_index, -1],
        )
        active += looped[band_index] * envelope
    active *= common.RESIDUAL_SCALE / math.sqrt(common.NOISE_BAND_COUNT)
    result = np.zeros(common.SAMPLE_COUNT, dtype=np.float64)
    result[common.ALIGNMENT_SAMPLE :] = active
    return result


def render_contact(
    gains: np.ndarray,
    latent: np.ndarray,
    noise_bands: np.ndarray,
    atoms: np.ndarray,
) -> np.ndarray:
    return render_modal(gains) + render_residual(latent, noise_bands, atoms)


def quantize_scaled(value: np.ndarray) -> tuple[np.float32, np.ndarray, np.ndarray]:
    maximum = float(np.max(np.abs(value)))
    scale = np.float32(max(maximum, 1.0e-30))
    quantized = np.asarray(value / float(scale), dtype="<f2")
    decoded = quantized.astype(np.float64) * float(scale)
    return scale, quantized, decoded


def encode_contact_record(
    coordinate: np.ndarray, gains: np.ndarray, latent: np.ndarray
) -> tuple[bytes, dict[str, np.ndarray]]:
    coordinate_value = np.asarray(coordinate, dtype="<f4")
    gain_scale, gain_q, gain_decoded = quantize_scaled(gains)
    latent_scale, latent_q, latent_decoded = quantize_scaled(latent)
    record = b"".join(
        [
            b"PSV9",
            coordinate_value.tobytes(),
            struct.pack("<f", float(gain_scale)),
            gain_q.tobytes(),
            struct.pack("<f", float(latent_scale)),
            latent_q.tobytes(),
        ]
    )
    return record, {
        "coordinate": coordinate_value.astype(np.float64),
        "gains": gain_decoded,
        "latent": latent_decoded,
    }


def encode_position(coordinates: np.ndarray) -> np.ndarray:
    value = np.asarray(coordinates, dtype=np.float32)
    normalized = value * np.float32(2.0) - np.float32(1.0)
    parts = [normalized]
    for band in common.POSITION_FOURIER_BANDS:
        phase = np.float32(math.pi * band) * normalized
        parts.extend((np.sin(phase), np.cos(phase)))
    return np.ascontiguousarray(np.concatenate(parts, axis=1), dtype=np.float32)


class ResidualLatentField(nn.Module):
    def __init__(self, input_dimension: int) -> None:
        super().__init__()
        width = common.FIELD_CONFIG["hidden_width"]
        self.layers = nn.Sequential(
            nn.Linear(input_dimension, width),
            nn.SiLU(),
            nn.Linear(width, width),
            nn.SiLU(),
            nn.Linear(width, common.RESIDUAL_ATOM_COUNT),
        )

    def forward(self, value: torch.Tensor) -> torch.Tensor:
        return self.layers(value)


def train_latent_field(
    coordinates: np.ndarray, latents: np.ndarray
) -> tuple[dict[str, Any], np.ndarray, np.ndarray, np.ndarray]:
    configure_torch()
    context_indices = farthest_point_indices(coordinates, common.CONTEXT_COUNT)
    query_mask = np.ones(coordinates.shape[0], dtype=bool)
    query_mask[context_indices] = False
    query_indices = np.flatnonzero(query_mask)
    encoded = encode_position(coordinates)
    mean = np.mean(latents[context_indices], axis=0, keepdims=True)
    scale = np.std(latents[context_indices], axis=0, keepdims=True) + 1.0e-6
    normalized = ((latents - mean) / scale).astype(np.float32)

    context_x = torch.from_numpy(encoded[context_indices])
    context_y = torch.from_numpy(normalized[context_indices])
    query_x = torch.from_numpy(encoded[query_indices])

    torch.manual_seed(common.FIELD_CONFIG["seed"])
    model = ResidualLatentField(encoded.shape[1])
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=common.FIELD_CONFIG["learning_rate"],
        weight_decay=common.FIELD_CONFIG["weight_decay"],
    )
    for _ in range(common.FIELD_CONFIG["updates"]):
        optimizer.zero_grad(set_to_none=True)
        prediction = model(context_x)
        loss = torch.mean((prediction - context_y) ** 2)
        loss.backward()
        optimizer.step()

    with torch.inference_mode():
        query_normalized = model(query_x).numpy().astype(np.float64)
        query_prediction = query_normalized * scale + mean
        query_prediction = np.maximum(query_prediction, 0.0)
        context_prediction = model(context_x)
        train_rmse = float(torch.sqrt(torch.mean((context_prediction - context_y) ** 2)))

    context_coordinates = coordinates[context_indices]
    query_coordinates = coordinates[query_indices]
    squared_distance = np.sum(
        np.square(query_coordinates[:, None, :] - context_coordinates[None, :, :]),
        axis=2,
    )
    nearest_rows = np.argmin(squared_distance, axis=1)
    nearest_latents = latents[context_indices[nearest_rows]]
    query_truth = latents[query_indices]
    neural_rmse = float(np.sqrt(np.mean(np.square(query_prediction - query_truth))))
    nearest_rmse = float(np.sqrt(np.mean(np.square(nearest_latents - query_truth))))
    mean_latent = np.mean(latents[context_indices], axis=0, keepdims=True)
    mean_rmse = float(np.sqrt(np.mean(np.square(mean_latent - query_truth))))
    parameter_bytes = sum(parameter.numel() * 4 for parameter in model.parameters())
    finite = bool(
        np.isfinite(query_prediction).all()
        and all(math.isfinite(value) for value in (neural_rmse, nearest_rmse, mean_rmse))
        and nearest_rmse > 0.0
    )
    report = {
        "finite": finite,
        "context_count": int(context_indices.size),
        "query_count": int(query_indices.size),
        "updates": common.FIELD_CONFIG["updates"],
        "final_context_normalized_rmse": train_rmse,
        "neural_query_latent_rmse": neural_rmse,
        "nearest_query_latent_rmse": nearest_rmse,
        "constant_query_latent_rmse": mean_rmse,
        "neural_to_nearest_latent_rmse_ratio": neural_rmse / nearest_rmse,
        "neural_below_constant": neural_rmse < mean_rmse,
        "parameter_bytes": parameter_bytes,
        "model_parameter_sha256": tensor_hash(dict(model.named_parameters())),
        "query_prediction_sha256": array_hash(query_prediction),
        "context_indices_sha256": hashlib.sha256(context_indices.tobytes()).hexdigest(),
        "query_indices_sha256": hashlib.sha256(query_indices.tobytes()).hexdigest(),
    }
    return report, query_prediction, nearest_latents, query_indices

