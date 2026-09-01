#!/usr/bin/env python3
"""Structured neural candidate for the frozen Physical Sound V16 L0 oracle."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import torch
from torch import nn
from torch.nn import functional as functional

import physical_sound_v16_l0_common as common


@dataclass(frozen=True)
class TrainingStatistics:
    geometry_aware: bool
    static_normalizer: common.Normalizer
    local_normalizer: common.Normalizer
    frequency_increment_inverse_mean: np.ndarray
    frequency_increment_inverse_scale: np.ndarray
    damping_inverse_mean: np.ndarray
    damping_inverse_scale: np.ndarray
    log_frequency_mean: np.ndarray
    log_frequency_scale: np.ndarray
    log_damping_mean: np.ndarray
    log_damping_scale: np.ndarray
    gain_scale: np.ndarray

    def record(self) -> dict[str, Any]:
        return {
            "damping_inverse_mean": self.damping_inverse_mean,
            "damping_inverse_scale": self.damping_inverse_scale,
            "frequency_increment_inverse_mean": self.frequency_increment_inverse_mean,
            "frequency_increment_inverse_scale": self.frequency_increment_inverse_scale,
            "gain_scale": self.gain_scale,
            "geometry_aware": self.geometry_aware,
            "local_normalizer": self.local_normalizer.record(),
            "log_damping_mean": self.log_damping_mean,
            "log_damping_scale": self.log_damping_scale,
            "log_frequency_mean": self.log_frequency_mean,
            "log_frequency_scale": self.log_frequency_scale,
            "static_normalizer": self.static_normalizer.record(),
        }


def _per_mode_normalizer(value: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    mean = np.mean(value, axis=0)
    scale = np.std(value, axis=0)
    return mean, np.where(scale > 1.0e-12, scale, 1.0)


def fit_training_statistics(
    train: tuple[common.ObjectData, ...], geometry_aware: bool
) -> TrainingStatistics:
    if len(train) != 9 or any(item.spec.role != "train" for item in train):
        raise common.L0Error("training statistics require the nine train objects")
    static_normalizer = common.fit_normalizer(
        common.static_features(item.spec, geometry_aware) for item in train
    )
    local_normalizer = common.fit_normalizer(
        common.local_features(item, geometry_aware) for item in train
    )
    frequencies = np.stack([item.frequencies for item in train])
    increments = np.column_stack(
        (frequencies[:, 0], np.diff(frequencies, axis=1))
    )
    frequency_inverse = common.inverse_softplus(increments)
    damping_inverse = common.inverse_softplus(
        np.stack([item.damping for item in train])
    )
    frequency_inverse_mean, frequency_inverse_scale = _per_mode_normalizer(
        frequency_inverse
    )
    damping_inverse_mean, damping_inverse_scale = _per_mode_normalizer(
        damping_inverse
    )
    log_frequency_mean, log_frequency_scale = _per_mode_normalizer(
        np.log(frequencies)
    )
    damping = np.stack([item.damping for item in train])
    log_damping_mean, log_damping_scale = _per_mode_normalizer(np.log(damping))
    query_gains = np.concatenate(
        [item.gains[item.query_indices] for item in train], axis=0
    )
    gain_scale = np.sqrt(np.mean(query_gains**2, axis=0))
    if np.any(gain_scale <= 1.0e-12):
        raise common.L0Error("train gain normalization is degenerate")
    return TrainingStatistics(
        geometry_aware=geometry_aware,
        static_normalizer=static_normalizer,
        local_normalizer=local_normalizer,
        frequency_increment_inverse_mean=frequency_inverse_mean,
        frequency_increment_inverse_scale=frequency_inverse_scale,
        damping_inverse_mean=damping_inverse_mean,
        damping_inverse_scale=damping_inverse_scale,
        log_frequency_mean=log_frequency_mean,
        log_frequency_scale=log_frequency_scale,
        log_damping_mean=log_damping_mean,
        log_damping_scale=log_damping_scale,
        gain_scale=gain_scale,
    )


class StructuredModalField(nn.Module):
    def __init__(self, static_dimension: int, local_dimension: int) -> None:
        super().__init__()
        self.object_encoder = nn.Sequential(
            nn.Linear(static_dimension, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
            nn.Linear(64, 32),
        )
        self.frequency_head = nn.Linear(32, common.MODE_COUNT)
        self.damping_head = nn.Linear(32, common.MODE_COUNT)
        self.context_encoder = nn.Sequential(
            nn.Linear(local_dimension + common.MODE_COUNT, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
        )
        self.surface_decoder = nn.Sequential(
            nn.Linear(32 + 128 + local_dimension, 128),
            nn.SiLU(),
            nn.Linear(128, 128),
            nn.SiLU(),
            nn.Linear(128, common.MODE_COUNT),
        )
        nn.init.zeros_(self.frequency_head.bias)
        nn.init.zeros_(self.damping_head.bias)

    def latent(
        self,
        static: torch.Tensor,
        local: torch.Tensor,
        context_indices: torch.Tensor,
        normalized_context_gains: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        object_latent = self.object_encoder(static)
        context_rows = torch.cat(
            (local[context_indices], normalized_context_gains), dim=1
        )
        encoded = self.context_encoder(context_rows)
        context_latent = torch.cat(
            (torch.mean(encoded, dim=0), torch.max(encoded, dim=0).values), dim=0
        )
        return object_latent, context_latent

    def forward(
        self,
        static: torch.Tensor,
        local: torch.Tensor,
        context_indices: torch.Tensor,
        normalized_context_gains: torch.Tensor,
        output_indices: torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        object_latent, context_latent = self.latent(
            static, local, context_indices, normalized_context_gains
        )
        frequency_raw = self.frequency_head(object_latent)
        damping_raw = self.damping_head(object_latent)
        count = output_indices.shape[0]
        decoder_input = torch.cat(
            (
                object_latent.expand(count, -1),
                context_latent.expand(count, -1),
                local[output_indices],
            ),
            dim=1,
        )
        gain = self.surface_decoder(decoder_input)
        return frequency_raw, damping_raw, gain


def configure_determinism(seed: int) -> None:
    torch.use_deterministic_algorithms(True)
    torch.set_num_threads(1)
    try:
        torch.set_num_interop_threads(1)
    except RuntimeError:
        if torch.get_num_interop_threads() != 1:
            raise
    torch.manual_seed(seed)
    np.random.seed(seed)


def _tensor(value: np.ndarray) -> torch.Tensor:
    return torch.as_tensor(value, dtype=torch.float64, device="cpu")


def _decode_global(
    frequency_raw: torch.Tensor,
    damping_raw: torch.Tensor,
    statistics: TrainingStatistics,
) -> tuple[torch.Tensor, torch.Tensor]:
    frequency_inverse = (
        frequency_raw * _tensor(statistics.frequency_increment_inverse_scale)
        + _tensor(statistics.frequency_increment_inverse_mean)
    )
    increments = functional.softplus(frequency_inverse)
    frequencies = torch.cumsum(increments, dim=0)
    damping_inverse = (
        damping_raw * _tensor(statistics.damping_inverse_scale)
        + _tensor(statistics.damping_inverse_mean)
    )
    damping = functional.softplus(damping_inverse)
    return frequencies, damping


def _prepared_object(
    data: common.ObjectData,
    statistics: TrainingStatistics,
    spec_override: common.ObjectSpec | None = None,
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    spec = data.spec if spec_override is None else spec_override
    raw_static = common.static_features(spec, statistics.geometry_aware)
    raw_local = common.local_features(data, statistics.geometry_aware)
    static = _tensor(statistics.static_normalizer.apply(raw_static))
    local = _tensor(statistics.local_normalizer.apply(raw_local))
    context_indices = torch.as_tensor(data.context_indices, dtype=torch.int64)
    context_gains = _tensor(
        data.gains[data.context_indices] / statistics.gain_scale[None, :]
    )
    return static, local, context_indices, context_gains


def train_member(
    train: tuple[common.ObjectData, ...],
    statistics: TrainingStatistics,
    seed: int,
    updates: int = common.UPDATES,
) -> tuple[StructuredModalField, dict[str, Any]]:
    if seed not in common.SEEDS:
        raise common.L0Error(f"unlisted L0 seed: {seed}")
    if updates <= 0:
        raise common.L0Error("training update count must be positive")
    configure_determinism(seed)
    model = StructuredModalField(
        statistics.static_normalizer.mean.size,
        statistics.local_normalizer.mean.size,
    ).to(dtype=torch.float64, device="cpu")
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=common.LEARNING_RATE_START,
        weight_decay=common.WEIGHT_DECAY,
    )
    prepared = []
    for data in train:
        static, local, context, context_gains = _prepared_object(data, statistics)
        query = torch.as_tensor(data.query_indices, dtype=torch.int64)
        prepared.append((data, static, local, context, context_gains, query))

    final_components: dict[str, float] = {}
    for step in range(updates):
        if updates == 1:
            phase = 0.0
        else:
            phase = step / (updates - 1)
        learning_rate = common.LEARNING_RATE_END + (
            common.LEARNING_RATE_START - common.LEARNING_RATE_END
        ) * 0.5 * (1.0 + math.cos(math.pi * phase))
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        optimizer.zero_grad(set_to_none=True)
        frequency_losses = []
        damping_losses = []
        gain_losses = []
        for data, static, local, context, context_gains, query in prepared:
            frequency_raw, damping_raw, gain = model(
                static, local, context, context_gains, query
            )
            frequencies, damping = _decode_global(
                frequency_raw, damping_raw, statistics
            )
            target_frequency = _tensor(data.frequencies)
            target_damping = _tensor(data.damping)
            frequency_loss = torch.mean(
                (
                    (torch.log(frequencies) - torch.log(target_frequency))
                    / _tensor(statistics.log_frequency_scale)
                )
                ** 2
            )
            damping_loss = torch.mean(
                (
                    (torch.log(damping) - torch.log(target_damping))
                    / _tensor(statistics.log_damping_scale)
                )
                ** 2
            )
            target_gain = _tensor(
                data.gains[data.query_indices] / statistics.gain_scale[None, :]
            )
            gain_loss = torch.mean((gain - target_gain) ** 2)
            frequency_losses.append(frequency_loss)
            damping_losses.append(damping_loss)
            gain_losses.append(gain_loss)
        frequency_component = torch.mean(torch.stack(frequency_losses))
        damping_component = torch.mean(torch.stack(damping_losses))
        gain_component = torch.mean(torch.stack(gain_losses))
        loss = frequency_component + damping_component + 2.0 * gain_component
        if not torch.isfinite(loss):
            raise common.L0Error(f"non-finite training loss at step {step}")
        loss.backward()
        optimizer.step()
        final_components = {
            "damping_loss": float(damping_component.detach()),
            "frequency_loss": float(frequency_component.detach()),
            "gain_loss": float(gain_component.detach()),
            "learning_rate": learning_rate,
            "total_loss": float(loss.detach()),
        }
    if not all(torch.isfinite(parameter).all() for parameter in model.parameters()):
        raise common.L0Error("trained model contains non-finite parameters")
    model.eval()
    return model, {
        "final": final_components,
        "geometry_aware": statistics.geometry_aware,
        "seed": seed,
        "updates": updates,
    }


@torch.no_grad()
def predict_member(
    model: StructuredModalField,
    data: common.ObjectData,
    statistics: TrainingStatistics,
    output_indices: np.ndarray | None = None,
    context_indices: np.ndarray | None = None,
    spec_override: common.ObjectSpec | None = None,
) -> dict[str, np.ndarray]:
    static, local, frozen_context, context_gains = _prepared_object(
        data, statistics, spec_override
    )
    if context_indices is not None:
        frozen_context = torch.as_tensor(context_indices, dtype=torch.int64)
        context_gains = _tensor(
            data.gains[context_indices] / statistics.gain_scale[None, :]
        )
    indices = data.query_indices if output_indices is None else output_indices
    output = torch.as_tensor(indices, dtype=torch.int64)
    frequency_raw, damping_raw, normalized_gain = model(
        static, local, frozen_context, context_gains, output
    )
    frequencies, damping = _decode_global(frequency_raw, damping_raw, statistics)
    gains = normalized_gain * _tensor(statistics.gain_scale)
    result = {
        "damping": damping.cpu().numpy(),
        "frequencies": frequencies.cpu().numpy(),
        "gains": gains.cpu().numpy(),
    }
    if not all(np.isfinite(value).all() for value in result.values()):
        raise common.L0Error("model prediction is non-finite")
    return result


def predict_ensemble(
    models: tuple[StructuredModalField, ...],
    data: common.ObjectData,
    statistics: TrainingStatistics,
    output_indices: np.ndarray | None = None,
    context_indices: np.ndarray | None = None,
    spec_override: common.ObjectSpec | None = None,
) -> dict[str, np.ndarray]:
    members = [
        predict_member(
            model,
            data,
            statistics,
            output_indices=output_indices,
            context_indices=context_indices,
            spec_override=spec_override,
        )
        for model in models
    ]
    frequencies = np.stack([item["frequencies"] for item in members])
    damping = np.stack([item["damping"] for item in members])
    gains = np.stack([item["gains"] for item in members])
    return {
        "damping": np.mean(damping, axis=0),
        "damping_members": damping,
        "frequencies": np.mean(frequencies, axis=0),
        "frequency_members": frequencies,
        "gains": np.mean(gains, axis=0),
        "gain_members": gains,
    }
