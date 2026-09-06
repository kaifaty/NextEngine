#!/usr/bin/env python3
"""Scale-separated and raw-dimensional neural heads for V17 G0."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import torch
from torch import nn
from torch.nn import functional as functional

import physical_sound_v17_g0_common as common


def candidate_features(row: common.GRow) -> np.ndarray:
    return np.asarray(
        [float(row.material == value) for value in common.MATERIAL_ORDER]
        + [float(row.topology == value) for value in common.TOPOLOGY_ORDER]
        + [float(row.support == value) for value in common.SUPPORT_ORDER]
        + [math.log(row.aspect), math.log(row.slenderness)],
        dtype=np.float64,
    )


def raw_features(row: common.GRow) -> np.ndarray:
    material = common.MATERIALS[row.material]
    return np.concatenate(
        (
            candidate_features(row),
            np.asarray(
                [
                    math.log(row.length_m),
                    math.log(row.wall_m),
                    math.log(material["density"]),
                    math.log(material["wave_speed"]),
                    math.log(material["decay"]),
                ],
                dtype=np.float64,
            ),
        )
    )


def continuous_ood_features(row: common.GRow) -> np.ndarray:
    return np.asarray(
        [math.log(row.aspect), math.log(row.slenderness)], dtype=np.float64
    )


@dataclass(frozen=True)
class FamilyStatistics:
    family: str
    feature_normalizer: common.Normalizer
    frequency_inverse_mean: np.ndarray
    frequency_inverse_scale: np.ndarray
    damping_inverse_mean: np.ndarray
    damping_inverse_scale: np.ndarray
    log_frequency_scale: np.ndarray
    log_damping_scale: np.ndarray

    def record(self) -> dict[str, Any]:
        return {
            "damping_inverse_mean": self.damping_inverse_mean,
            "damping_inverse_scale": self.damping_inverse_scale,
            "family": self.family,
            "feature_normalizer": self.feature_normalizer.record(),
            "frequency_inverse_mean": self.frequency_inverse_mean,
            "frequency_inverse_scale": self.frequency_inverse_scale,
            "log_damping_scale": self.log_damping_scale,
            "log_frequency_scale": self.log_frequency_scale,
        }


def target_values(
    row: common.GRow, family: str
) -> tuple[np.ndarray, np.ndarray]:
    if family == "scale_separated":
        return (
            row.frequencies / row.frequency_scale,
            row.damping / row.damping_scale,
        )
    if family == "raw_dimensional":
        return row.frequencies, row.damping
    raise common.G0Error(f"unknown G0 family: {family}")


def features(row: common.GRow, family: str) -> np.ndarray:
    if family == "scale_separated":
        return candidate_features(row)
    if family == "raw_dimensional":
        return raw_features(row)
    raise common.G0Error(f"unknown G0 family: {family}")


def _mean_scale(value: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    mean = np.mean(value, axis=0)
    scale = np.std(value, axis=0)
    return mean, np.where(scale > 1.0e-12, scale, 1.0)


def fit_statistics(
    train: tuple[common.GRow, ...], family: str
) -> FamilyStatistics:
    if len(train) != 72 or any(row.role != "train" for row in train):
        raise common.G0Error("G0 statistics require exactly 72 train rows")
    feature_normalizer = common.fit_normalizer(features(row, family) for row in train)
    frequency = np.stack([target_values(row, family)[0] for row in train])
    damping = np.stack([target_values(row, family)[1] for row in train])
    increments = np.column_stack((frequency[:, 0], np.diff(frequency, axis=1)))
    frequency_inverse = common.inverse_softplus(increments)
    damping_inverse = common.inverse_softplus(damping)
    frequency_inverse_mean, frequency_inverse_scale = _mean_scale(frequency_inverse)
    damping_inverse_mean, damping_inverse_scale = _mean_scale(damping_inverse)
    _, log_frequency_scale = _mean_scale(np.log(frequency))
    _, log_damping_scale = _mean_scale(np.log(damping))
    return FamilyStatistics(
        family=family,
        feature_normalizer=feature_normalizer,
        frequency_inverse_mean=frequency_inverse_mean,
        frequency_inverse_scale=frequency_inverse_scale,
        damping_inverse_mean=damping_inverse_mean,
        damping_inverse_scale=damping_inverse_scale,
        log_frequency_scale=log_frequency_scale,
        log_damping_scale=log_damping_scale,
    )


class GlobalHead(nn.Module):
    def __init__(self, input_dimension: int) -> None:
        super().__init__()
        self.encoder = nn.Sequential(
            nn.Linear(input_dimension, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
            nn.Linear(64, 32),
        )
        self.frequency_head = nn.Linear(32, common.MODE_COUNT)
        self.damping_head = nn.Linear(32, common.MODE_COUNT)
        nn.init.zeros_(self.frequency_head.bias)
        nn.init.zeros_(self.damping_head.bias)

    def forward(self, value: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        latent = self.encoder(value)
        return self.frequency_head(latent), self.damping_head(latent)


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


def _decode_target(
    frequency_raw: torch.Tensor,
    damping_raw: torch.Tensor,
    statistics: FamilyStatistics,
) -> tuple[torch.Tensor, torch.Tensor]:
    frequency_inverse = (
        frequency_raw * _tensor(statistics.frequency_inverse_scale)
        + _tensor(statistics.frequency_inverse_mean)
    )
    frequency = torch.cumsum(functional.softplus(frequency_inverse), dim=-1)
    damping_inverse = (
        damping_raw * _tensor(statistics.damping_inverse_scale)
        + _tensor(statistics.damping_inverse_mean)
    )
    damping = functional.softplus(damping_inverse)
    return frequency, damping


def train_member(
    train: tuple[common.GRow, ...],
    statistics: FamilyStatistics,
    seed: int,
    updates: int = common.UPDATES,
) -> tuple[GlobalHead, dict[str, Any]]:
    if seed not in common.SEEDS:
        raise common.G0Error(f"unlisted G0 seed: {seed}")
    if updates <= 0:
        raise common.G0Error("G0 update count must be positive")
    configure_determinism(seed)
    feature_matrix = np.stack(
        [statistics.feature_normalizer.apply(features(row, statistics.family)) for row in train]
    )
    target_frequency = np.stack(
        [target_values(row, statistics.family)[0] for row in train]
    )
    target_damping = np.stack(
        [target_values(row, statistics.family)[1] for row in train]
    )
    feature_tensor = _tensor(feature_matrix)
    frequency_tensor = _tensor(target_frequency)
    damping_tensor = _tensor(target_damping)
    model = GlobalHead(feature_matrix.shape[1]).to(dtype=torch.float64, device="cpu")
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=common.LEARNING_RATE_START,
        weight_decay=common.WEIGHT_DECAY,
    )
    final: dict[str, float] = {}
    for step in range(updates):
        phase = 0.0 if updates == 1 else step / (updates - 1)
        learning_rate = common.LEARNING_RATE_END + (
            common.LEARNING_RATE_START - common.LEARNING_RATE_END
        ) * 0.5 * (1.0 + math.cos(math.pi * phase))
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        optimizer.zero_grad(set_to_none=True)
        frequency_raw, damping_raw = model(feature_tensor)
        predicted_frequency, predicted_damping = _decode_target(
            frequency_raw, damping_raw, statistics
        )
        frequency_loss = torch.mean(
            (
                (torch.log(predicted_frequency) - torch.log(frequency_tensor))
                / _tensor(statistics.log_frequency_scale)
            )
            ** 2
        )
        damping_loss = torch.mean(
            (
                (torch.log(predicted_damping) - torch.log(damping_tensor))
                / _tensor(statistics.log_damping_scale)
            )
            ** 2
        )
        loss = frequency_loss + damping_loss
        if not torch.isfinite(loss):
            raise common.G0Error(f"non-finite G0 loss at step {step}")
        loss.backward()
        optimizer.step()
        final = {
            "damping_loss": float(damping_loss.detach()),
            "frequency_loss": float(frequency_loss.detach()),
            "learning_rate": learning_rate,
            "total_loss": float(loss.detach()),
        }
    if not all(torch.isfinite(parameter).all() for parameter in model.parameters()):
        raise common.G0Error("G0 trained model is non-finite")
    model.eval()
    return model, {
        "family": statistics.family,
        "final": final,
        "seed": seed,
        "updates": updates,
    }


@torch.no_grad()
def predict_member(
    model: GlobalHead,
    row: common.GRow,
    statistics: FamilyStatistics,
) -> tuple[np.ndarray, np.ndarray]:
    feature = statistics.feature_normalizer.apply(features(row, statistics.family))
    frequency_raw, damping_raw = model(_tensor(feature)[None, :])
    target_frequency, target_damping = _decode_target(
        frequency_raw, damping_raw, statistics
    )
    frequency = target_frequency[0].cpu().numpy()
    damping = target_damping[0].cpu().numpy()
    if statistics.family == "scale_separated":
        frequency = frequency * row.frequency_scale
        damping = damping * row.damping_scale
    if not np.isfinite(frequency).all() or not np.isfinite(damping).all():
        raise common.G0Error("G0 prediction is non-finite")
    return frequency, damping


def predict_ensemble(
    models: tuple[GlobalHead, ...],
    row: common.GRow,
    statistics: FamilyStatistics,
) -> dict[str, np.ndarray]:
    values = [predict_member(model, row, statistics) for model in models]
    frequency = np.stack([value[0] for value in values])
    damping = np.stack([value[1] for value in values])
    return {
        "damping": np.mean(damping, axis=0),
        "damping_members": damping,
        "frequencies": np.mean(frequency, axis=0),
        "frequency_members": frequency,
    }
