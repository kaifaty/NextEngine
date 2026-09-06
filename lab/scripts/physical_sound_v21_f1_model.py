#!/usr/bin/env python3
"""Continuous residual models and deterministic training for V21 F1a."""

from __future__ import annotations

import io
import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import physical_sound_v19_f0_model as f0_model
import physical_sound_v21_f1_common as common
import torch
from torch import nn

WIDTH = 128
UPDATES = 1_500
LEARNING_RATE_START = 2.0e-3
LEARNING_RATE_END = 1.0e-5
WEIGHT_DECAY = 1.0e-6
GRADIENT_LOSS_WEIGHT = 0.15
PAIR_LOSS_WEIGHT = 0.10
CANDIDATE_SEED = 210_001
PAIRED_HARMONIC_SEED = 210_002
PARAMETER_COUNT = 41_992
PARAMETER_BYTES = 335_936
MODEL_BYTE_LIMIT = 400_000
CONDITION_LIMIT = 1.0e12
RBF_BANDWIDTH = 0.20
BASIS_REVISION = "topology-native-continuous-tensor-v1"


@dataclass(frozen=True)
class CandidateSpec:
    candidate_id: str
    order: int
    regularization: float


CANDIDATES = (
    CandidateSpec("continuous-q2-l1e-6", 2, 1.0e-6),
    CandidateSpec("continuous-q2-l1e-4", 2, 1.0e-4),
    CandidateSpec("continuous-q2-l1e-2", 2, 1.0e-2),
    CandidateSpec("continuous-q3-l1e-6", 3, 1.0e-6),
    CandidateSpec("continuous-q3-l1e-4", 3, 1.0e-4),
    CandidateSpec("continuous-q3-l1e-2", 3, 1.0e-2),
)


class PriorNetwork(nn.Module):
    """F1 mesh-independent topology-conditioned modal-gain prior."""

    def __init__(self) -> None:
        super().__init__()
        self.network = nn.Sequential(
            nn.Linear(61, WIDTH),
            nn.SiLU(),
            nn.Linear(WIDTH, WIDTH),
            nn.SiLU(),
            nn.Linear(WIDTH, WIDTH),
            nn.SiLU(),
            nn.Linear(WIDTH, common.MODE_COUNT),
        )

    def forward(self, value: torch.Tensor) -> torch.Tensor:
        return self.network(value)


@dataclass(frozen=True)
class ContinuousSystem:
    basis: torch.Tensor
    cholesky: torch.Tensor
    condition_number: float


@dataclass(frozen=True)
class TrainingResult:
    model: PriorNetwork
    gain_scale: np.ndarray
    initial_loss: float
    final_loss: float
    spec: CandidateSpec | None
    seed: int

    def summary(self) -> dict[str, Any]:
        return {
            "candidate_id": None if self.spec is None else self.spec.candidate_id,
            "final_loss": self.final_loss,
            "gain_scale": self.gain_scale,
            "initial_loss": self.initial_loss,
            "learning_rate_end": LEARNING_RATE_END,
            "learning_rate_start": LEARNING_RATE_START,
            "parameter_bytes": parameter_bytes(self.model),
            "parameter_count": parameter_count(self.model),
            "seed": self.seed,
            "updates": UPDATES,
            "weight_decay": WEIGHT_DECAY,
        }


def _tensor(value: np.ndarray) -> torch.Tensor:
    return torch.tensor(np.asarray(value), dtype=torch.float64, device="cpu")


def _index_tensor(value: np.ndarray) -> torch.Tensor:
    return torch.tensor(np.asarray(value), dtype=torch.int64, device="cpu")


def analytic_surface(
    row: common.FieldRow, uv: np.ndarray
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    value = np.asarray(uv, dtype=np.float64)
    if value.ndim != 2 or value.shape[1] != 2 or not np.isfinite(value).all():
        raise common.F1Error("F1 analytic UV changed")
    u = value[:, 0]
    v = value[:, 1]
    x = 0.5 * (u + 1.0)
    y = 0.5 * (v + 1.0)
    theta = 2.0 * np.pi * x
    if row.topology == "Plate":
        xyz = np.column_stack((u, v, np.zeros_like(u)))
        normal = np.tile(np.asarray([0.0, 0.0, 1.0]), (u.size, 1))
        curvature = np.zeros((u.size, 2), dtype=np.float64)
    elif row.topology in ("Cylinder", "RolledSheet"):
        radial = np.column_stack((np.cos(theta), np.sin(theta)))
        xyz = np.column_stack((radial, v))
        normal = np.column_stack((radial, np.zeros_like(u)))
        curvature = np.column_stack((np.full(u.size, 2.0 * np.pi), np.zeros(u.size)))
    elif row.topology == "Bowl":
        alpha = 0.5 * np.pi * y
        sin_alpha = np.sin(alpha)
        cos_alpha = np.cos(alpha)
        xyz = np.column_stack(
            (sin_alpha * np.cos(theta), sin_alpha * np.sin(theta), -cos_alpha)
        )
        radial = 0.5 * row.length_m
        axial = 0.5 * row.length_m * row.aspect
        raw = np.column_stack(
            (
                sin_alpha * np.cos(theta) / radial,
                sin_alpha * np.sin(theta) / radial,
                -cos_alpha / axial,
            )
        )
        normal = raw / np.linalg.norm(raw, axis=1, keepdims=True)
        denominator = np.sqrt(
            radial * radial * cos_alpha**2 + axial * axial * sin_alpha**2
        )
        curvature = row.length_m * np.column_stack(
            (
                radial * axial / denominator**3,
                axial / (radial * denominator),
            )
        )
    else:
        raise common.F1Error(f"unknown F1 topology: {row.topology}")
    if (
        xyz.shape != (u.size, 3)
        or normal.shape != (u.size, 3)
        or curvature.shape != (u.size, 2)
        or not np.isfinite(xyz).all()
        or not np.isfinite(normal).all()
        or not np.isfinite(curvature).all()
    ):
        raise common.F1Error("F1 analytic surface feature changed")
    return xyz, normal, curvature


def positional_features(uv: np.ndarray) -> np.ndarray:
    value = np.asarray(uv, dtype=np.float64)
    u = value[:, 0]
    v = value[:, 1]
    columns: list[np.ndarray] = []
    for frequency in range(1, 6):
        for coordinate in (u, v, u + v, u - v):
            columns.extend(
                (
                    np.sin(frequency * np.pi * coordinate),
                    np.cos(frequency * np.pi * coordinate),
                )
            )
    result = np.column_stack(columns)
    if result.shape != (value.shape[0], 40) or not np.isfinite(result).all():
        raise common.F1Error("F1 positional features changed")
    return result


def prior_features_at(row: common.FieldRow, uv: np.ndarray) -> np.ndarray:
    xyz, normal, curvature_times_length = analytic_surface(row, uv)
    local = np.column_stack(
        (uv, xyz, normal, curvature_times_length, positional_features(uv))
    )
    static = np.asarray(
        [row.material == value for value in common.f0_common.MATERIAL_ORDER]
        + [row.topology == value for value in common.f0_common.TOPOLOGY_ORDER]
        + [row.support == value for value in common.f0_common.SUPPORT_ORDER]
        + [math.log(row.aspect), math.log(row.slenderness)],
        dtype=np.float64,
    )
    result = np.column_stack((local, np.tile(static, (uv.shape[0], 1))))
    if result.shape != (uv.shape[0], 61) or not np.isfinite(result).all():
        raise common.F1Error("F1 prior feature changed")
    return result


def prior_features(item: common.FieldObject) -> np.ndarray:
    return prior_features_at(item.row, item.mesh.uv)


def _axis_features(
    coordinate: np.ndarray, order: int, periodic: bool
) -> tuple[np.ndarray, np.ndarray]:
    columns = [np.ones_like(coordinate)]
    frequencies = [0]
    scale = 2.0 if periodic else 1.0
    for frequency in range(1, order + 1):
        angle = scale * np.pi * frequency * coordinate
        columns.extend((np.cos(angle), np.sin(angle)))
        frequencies.extend((frequency, frequency))
    return np.column_stack(columns), np.asarray(frequencies, dtype=np.float64)


def basis_matrix(
    topology: str, uv: np.ndarray, order: int
) -> tuple[np.ndarray, np.ndarray]:
    if order not in (2, 3):
        raise common.F1Error(f"unknown F1 basis order: {order}")
    value = np.asarray(uv, dtype=np.float64)
    x = 0.5 * (value[:, 0] + 1.0)
    y = 0.5 * (value[:, 1] + 1.0)
    u_axis, u_frequency = _axis_features(x, order, topology in ("Cylinder", "Bowl"))
    v_axis, v_frequency = _axis_features(y, order, False)
    columns = []
    penalties = []
    for u_index in range(u_axis.shape[1]):
        for v_index in range(v_axis.shape[1]):
            columns.append(u_axis[:, u_index] * v_axis[:, v_index])
            penalties.append(
                (1.0 + u_frequency[u_index] ** 2 + v_frequency[v_index] ** 2) ** 2
            )
    result = np.column_stack(columns)
    penalty = np.asarray(penalties, dtype=np.float64)
    expected = (1 + 2 * order) ** 2
    if (
        result.shape != (value.shape[0], expected)
        or penalty.shape != (expected,)
        or not np.isfinite(result).all()
        or np.any(penalty <= 0.0)
    ):
        raise common.F1Error("F1 continuous basis changed")
    return result, penalty


def prepare_system(
    item: common.FieldObject, spec: CandidateSpec, expected_role: str
) -> ContinuousSystem:
    validate_request(item, spec, expected_role=expected_role)
    basis, penalty = basis_matrix(item.row.topology, item.mesh.uv, spec.order)
    context_basis = basis[item.context]
    matrix = context_basis.T @ context_basis / item.context.size
    matrix += spec.regularization * np.diag(penalty)
    condition = float(np.linalg.cond(matrix, p=2))
    if not math.isfinite(condition) or condition > CONDITION_LIMIT:
        raise common.F1Error("F1 residual system condition rejected")
    try:
        cholesky = np.linalg.cholesky(matrix)
    except np.linalg.LinAlgError as error:
        raise common.F1Error("F1 residual system is not positive definite") from error
    return ContinuousSystem(_tensor(basis), _tensor(cholesky), condition)


def validate_request(
    item: common.FieldObject,
    spec: CandidateSpec,
    *,
    expected_role: str,
    declared_role: str | None = None,
    declared_basis_revision: str = BASIS_REVISION,
    declared_regularization: float | None = None,
    coefficient: np.ndarray | None = None,
    available_model_bytes: int = MODEL_BYTE_LIMIT,
) -> None:
    """Reject structural and basis identity changes before publishing output."""

    if spec not in CANDIDATES:
        raise common.F1Error("F1 candidate identity changed")
    if expected_role not in ("train", "development"):
        raise common.F1Error("F1 expected role changed")
    role = item.row.role if declared_role is None else declared_role
    if item.row.role != expected_role or role != expected_role:
        raise common.F1Error("F1 role identity changed")
    if declared_basis_revision != BASIS_REVISION:
        raise common.F1Error("F1 basis identity changed")
    regularization = (
        spec.regularization
        if declared_regularization is None
        else declared_regularization
    )
    if regularization != spec.regularization:
        raise common.F1Error("F1 regularization identity changed")
    if available_model_bytes < PARAMETER_BYTES:
        raise common.F1Error("F1 model budget is insufficient")
    if coefficient is not None:
        expected = ((1 + 2 * spec.order) ** 2, common.MODE_COUNT)
        value = np.asarray(coefficient, dtype=np.float64)
        if value.shape != expected or not np.isfinite(value).all():
            raise common.F1Error("F1 residual coefficient changed")


def fit_gain_scale(train: tuple[common.FieldObject, ...]) -> np.ndarray:
    if len(train) != 48 or any(item.row.role != "train" for item in train):
        raise common.F1Error("F1 gain scale requires 48 train views")
    view_means = np.asarray(
        [np.mean(item.gains[item.accepted_query] ** 2, axis=0) for item in train],
        dtype=np.float64,
    )
    scale = np.sqrt(np.mean(view_means, axis=0))
    if scale.shape != (common.MODE_COUNT,) or np.any(scale <= 1.0e-12):
        raise common.F1Error("F1 gain scale is degenerate")
    return scale


def _active_edges(item: common.FieldObject) -> np.ndarray:
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    edge = item.mesh.edges
    edge = edge[active[edge[:, 0]] & active[edge[:, 1]]]
    context = np.zeros(item.mesh.vertex_count, dtype=bool)
    context[item.context] = True
    return edge[~(context[edge[:, 0]] & context[edge[:, 1]])]


def _continuous_prediction_torch(
    trained: PriorNetwork,
    scale: torch.Tensor,
    item: common.FieldObject,
    system: ContinuousSystem,
    features: torch.Tensor,
    truth: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor]:
    prior = trained(features)
    residual = truth[item.context] - prior[item.context]
    context_basis = system.basis[item.context]
    right = context_basis.T @ residual / item.context.size
    coefficient = torch.cholesky_solve(right, system.cholesky)
    if not torch.isfinite(coefficient).all():
        raise common.F1Error("F1 residual coefficient changed")
    prediction = prior + system.basis @ coefficient
    prediction = prediction.clone()
    prediction[item.context] = truth[item.context]
    return prediction, coefficient


def _probe_grid() -> np.ndarray:
    grid = np.linspace(-0.75, 0.75, 7, dtype=np.float64)
    return np.asarray([(u, v) for v in grid for u in grid], dtype=np.float64)


def _train_candidate(
    train: tuple[common.FieldObject, ...], spec: CandidateSpec
) -> TrainingResult:
    gain_scale = fit_gain_scale(train)
    scale = _tensor(gain_scale)
    prepared = [
        (
            item,
            prepare_system(item, spec, "train"),
            _tensor(prior_features(item)),
            _tensor(item.gains / gain_scale),
            _index_tensor(_active_edges(item)),
        )
        for item in train
    ]
    by_group: dict[str, list[int]] = {}
    for index, item in enumerate(train):
        by_group.setdefault(item.row.physical_group_id, []).append(index)
    if len(by_group) != 24 or any(len(indices) != 2 for indices in by_group.values()):
        raise common.F1Error("F1 train pair ledger changed")
    probes = _probe_grid()
    torch.manual_seed(CANDIDATE_SEED)
    trained = PriorNetwork().to(dtype=torch.float64, device="cpu")
    optimizer = torch.optim.AdamW(
        trained.parameters(), lr=LEARNING_RATE_START, weight_decay=WEIGHT_DECAY
    )
    initial_loss = math.nan
    final_loss = math.nan
    for update in range(UPDATES):
        phase = update / (UPDATES - 1)
        learning_rate = LEARNING_RATE_END + 0.5 * (1.0 + math.cos(math.pi * phase)) * (
            LEARNING_RATE_START - LEARNING_RATE_END
        )
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        view_losses = []
        coefficients: list[torch.Tensor] = []
        for item, system, features, truth, edge in prepared:
            prediction, coefficient = _continuous_prediction_torch(
                trained, scale, item, system, features, truth
            )
            coefficients.append(coefficient)
            query_loss = torch.mean(
                (prediction[item.accepted_query] - truth[item.accepted_query]) ** 2
            )
            gradient_loss = torch.mean(
                (
                    (prediction[edge[:, 0]] - prediction[edge[:, 1]])
                    - (truth[edge[:, 0]] - truth[edge[:, 1]])
                )
                ** 2
            )
            view_losses.append(query_loss + GRADIENT_LOSS_WEIGHT * gradient_loss)
        pair_losses = []
        for indices in by_group.values():
            values = []
            for index in indices:
                item, _system, _features, _truth, _edge = prepared[index]
                probe_features = _tensor(prior_features_at(item.row, probes))
                probe_basis = _tensor(
                    basis_matrix(item.row.topology, probes, spec.order)[0]
                )
                values.append(
                    trained(probe_features) + probe_basis @ coefficients[index]
                )
            pair_losses.append(torch.mean((values[0] - values[1]) ** 2))
        loss = torch.mean(torch.stack(view_losses)) + PAIR_LOSS_WEIGHT * torch.mean(
            torch.stack(pair_losses)
        )
        if not torch.isfinite(loss):
            raise common.F1Error("F1 candidate loss is non-finite")
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        value = float(loss.detach())
        if update == 0:
            initial_loss = value
        if update == UPDATES - 1:
            final_loss = value
    trained.eval()
    _verify_model(trained)
    return TrainingResult(
        trained, gain_scale, initial_loss, final_loss, spec, CANDIDATE_SEED
    )


def train_candidates(
    train: tuple[common.FieldObject, ...],
) -> tuple[TrainingResult, ...]:
    return tuple(_train_candidate(train, spec) for spec in CANDIDATES)


def train_paired_harmonic(
    train: tuple[common.FieldObject, ...], gain_scale: np.ndarray
) -> TrainingResult:
    prepared = [
        (item, _tensor(prior_features(item)), _tensor(item.gains / gain_scale))
        for item in train
    ]
    torch.manual_seed(PAIRED_HARMONIC_SEED)
    trained = PriorNetwork().to(dtype=torch.float64, device="cpu")
    optimizer = torch.optim.AdamW(
        trained.parameters(), lr=LEARNING_RATE_START, weight_decay=WEIGHT_DECAY
    )
    initial_loss = math.nan
    final_loss = math.nan
    for update in range(UPDATES):
        phase = update / (UPDATES - 1)
        learning_rate = LEARNING_RATE_END + 0.5 * (1.0 + math.cos(math.pi * phase)) * (
            LEARNING_RATE_START - LEARNING_RATE_END
        )
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        losses = []
        for item, features, truth in prepared:
            prediction = trained(features)
            exact = prediction.clone()
            exact[item.context] = truth[item.context]
            edge = _index_tensor(_active_edges(item))
            query_loss = torch.mean(
                (prediction[item.accepted_query] - truth[item.accepted_query]) ** 2
            )
            gradient_loss = torch.mean(
                (
                    (exact[edge[:, 0]] - exact[edge[:, 1]])
                    - (truth[edge[:, 0]] - truth[edge[:, 1]])
                )
                ** 2
            )
            losses.append(query_loss + GRADIENT_LOSS_WEIGHT * gradient_loss)
        loss = torch.mean(torch.stack(losses))
        if not torch.isfinite(loss):
            raise common.F1Error("F1 paired-harmonic loss is non-finite")
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        value = float(loss.detach())
        if update == 0:
            initial_loss = value
        if update == UPDATES - 1:
            final_loss = value
    trained.eval()
    _verify_model(trained)
    return TrainingResult(
        trained,
        np.asarray(gain_scale, dtype=np.float64),
        initial_loss,
        final_loss,
        None,
        PAIRED_HARMONIC_SEED,
    )


def _prior_prediction(trained: PriorNetwork, item: common.FieldObject) -> np.ndarray:
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    result = np.zeros((item.mesh.vertex_count, common.MODE_COUNT), dtype=np.float64)
    with torch.no_grad():
        result[active] = trained(_tensor(prior_features(item)[active])).cpu().numpy()
    return result


def _finalize(
    normalized: np.ndarray,
    scale: np.ndarray,
    item: common.FieldObject,
    observed: np.ndarray,
) -> np.ndarray:
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    result = np.zeros_like(normalized)
    result[active] = normalized[active] * scale
    result[item.context] = observed
    if not np.isfinite(result[active]).all():
        raise common.F1Error("F1 prediction is non-finite")
    return result


def predict_continuous(
    trained: PriorNetwork,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
    observed_context: np.ndarray | None = None,
) -> np.ndarray:
    observed = (
        item.gains[item.context]
        if observed_context is None
        else np.asarray(observed_context, dtype=np.float64)
    )
    if observed.shape != (item.context.size, common.MODE_COUNT):
        raise common.F1Error("F1 observed context shape changed")
    system = prepare_system(item, spec, "development")
    with torch.no_grad():
        prior = _prior_prediction(trained, item)
        normalized_observed = observed / gain_scale
        residual = normalized_observed - prior[item.context]
        context_basis = system.basis[item.context].numpy()
        right = context_basis.T @ residual / item.context.size
        coefficient = np.linalg.solve(system.cholesky.numpy(), right)
        coefficient = np.linalg.solve(system.cholesky.numpy().T, coefficient)
        validate_request(
            item, spec, expected_role="development", coefficient=coefficient
        )
        normalized = prior + system.basis.numpy() @ coefficient
    return _finalize(normalized, gain_scale, item, observed)


def direct_probe_prediction(
    trained: PriorNetwork,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
) -> np.ndarray:
    probes = _probe_grid()
    system = prepare_system(item, spec, "development")
    prior = _prior_prediction(trained, item)
    residual = item.gains[item.context] / gain_scale - prior[item.context]
    context_basis = system.basis[item.context].numpy()
    right = context_basis.T @ residual / item.context.size
    coefficient = np.linalg.solve(system.cholesky.numpy(), right)
    coefficient = np.linalg.solve(system.cholesky.numpy().T, coefficient)
    validate_request(item, spec, expected_role="development", coefficient=coefficient)
    with torch.no_grad():
        probe_prior = (
            trained(_tensor(prior_features_at(item.row, probes))).cpu().numpy()
        )
    probe_basis = basis_matrix(item.row.topology, probes, spec.order)[0]
    result = gain_scale * (probe_prior + probe_basis @ coefficient)
    if result.shape != (49, common.MODE_COUNT) or not np.isfinite(result).all():
        raise common.F1Error("F1 direct probe prediction changed")
    return result


def predict_paired_harmonic(
    trained: PriorNetwork, gain_scale: np.ndarray, item: common.FieldObject
) -> np.ndarray:
    prior = _prior_prediction(trained, item)
    observed = item.gains[item.context]
    residual = observed / gain_scale - prior[item.context]
    normalized = prior + f0_model._intrinsic_harmonic(item, residual)
    return _finalize(normalized, gain_scale, item, observed)


def compatible_predictions(
    trained: PriorNetwork,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
) -> dict[str, np.ndarray]:
    observed = item.gains[item.context]
    normalized_observed = observed / gain_scale
    prior = _prior_prediction(trained, item)
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    zero = np.zeros_like(prior)
    context_mean = zero.copy()
    context_mean[active] = np.mean(normalized_observed, axis=0)
    distances = item.analysis.all_pairs[:, item.context]
    nearest_index = np.argmin(distances, axis=1)
    nearest = zero.copy()
    nearest[active] = normalized_observed[nearest_index[active]]
    raw_harmonic = f0_model._intrinsic_harmonic(item, normalized_observed)
    raw_geo = f0_model._rbf_residual(item, normalized_observed, True)
    raw_euclidean = f0_model._rbf_residual(item, normalized_observed, False)
    residual = normalized_observed - prior[item.context]
    prior_euclidean = prior + f0_model._rbf_residual(item, residual, False)
    system = prepare_system(item, spec, "development")
    context_basis = system.basis[item.context].numpy()
    right = context_basis.T @ normalized_observed / item.context.size
    coefficient = np.linalg.solve(system.cholesky.numpy(), right)
    coefficient = np.linalg.solve(system.cholesky.numpy().T, coefficient)
    validate_request(item, spec, expected_role="development", coefficient=coefficient)
    raw_continuous = system.basis.numpy() @ coefficient
    return {
        "candidate": predict_continuous(trained, gain_scale, item, spec),
        "context-mean": _finalize(context_mean, gain_scale, item, observed),
        "nearest-intrinsic": _finalize(nearest, gain_scale, item, observed),
        "prior-euclidean-rbf": _finalize(prior_euclidean, gain_scale, item, observed),
        "prior-only": _finalize(prior, gain_scale, item, observed),
        "raw-continuous": _finalize(raw_continuous, gain_scale, item, observed),
        "raw-euclidean-rbf": _finalize(raw_euclidean, gain_scale, item, observed),
        "raw-geodesic-rbf": _finalize(raw_geo, gain_scale, item, observed),
        "raw-harmonic": _finalize(raw_harmonic, gain_scale, item, observed),
    }


def parameter_count(trained: PriorNetwork) -> int:
    return sum(parameter.numel() for parameter in trained.parameters())


def parameter_bytes(trained: PriorNetwork) -> int:
    return sum(
        parameter.numel() * parameter.element_size()
        for parameter in trained.parameters()
    )


def _verify_model(trained: PriorNetwork) -> None:
    if (
        parameter_count(trained) != PARAMETER_COUNT
        or parameter_bytes(trained) != PARAMETER_BYTES
        or parameter_bytes(trained) > MODEL_BYTE_LIMIT
        or not all(
            torch.isfinite(parameter).all() for parameter in trained.parameters()
        )
    ):
        raise common.F1Error("F1 model size/value changed")


def encode_models(
    candidates: tuple[TrainingResult, ...], paired: TrainingResult
) -> bytes:
    if tuple(result.spec for result in candidates) != CANDIDATES:
        raise common.F1Error("F1 candidate model order changed")
    scale = candidates[0].gain_scale
    if any(
        not np.array_equal(result.gain_scale, scale) for result in candidates
    ) or not np.array_equal(paired.gain_scale, scale):
        raise common.F1Error("F1 model gain scale changed")
    arrays: dict[str, np.ndarray] = {
        "candidate_lambda": np.asarray(
            [spec.regularization for spec in CANDIDATES], dtype=np.float64
        ),
        "candidate_order": np.asarray(
            [spec.order for spec in CANDIDATES], dtype=np.int64
        ),
        "gain_scale": np.asarray(scale, dtype=np.float64),
    }
    for candidate_index, result in enumerate(candidates):
        for parameter_index, name in enumerate(sorted(result.model.state_dict())):
            arrays[f"c{candidate_index:02d}_p{parameter_index:03d}"] = (
                result.model.state_dict()[name].detach().cpu().numpy()
            )
    for parameter_index, name in enumerate(sorted(paired.model.state_dict())):
        arrays[f"h_p{parameter_index:03d}"] = (
            paired.model.state_dict()[name].detach().cpu().numpy()
        )
    return common.deterministic_npz(arrays)


def decode_models(
    payload: bytes,
) -> tuple[tuple[PriorNetwork, ...], PriorNetwork, np.ndarray]:
    template = PriorNetwork().to(dtype=torch.float64, device="cpu")
    names = sorted(template.state_dict())
    expected = {"candidate_lambda", "candidate_order", "gain_scale"}
    expected |= {
        f"c{candidate_index:02d}_p{parameter_index:03d}"
        for candidate_index in range(len(CANDIDATES))
        for parameter_index in range(len(names))
    }
    expected |= {f"h_p{parameter_index:03d}" for parameter_index in range(len(names))}
    try:
        with np.load(io.BytesIO(payload), allow_pickle=False) as archive:
            if set(archive.files) != expected:
                raise common.F1Error("F1 models NPZ member set changed")
            if not np.array_equal(
                archive["candidate_order"],
                np.asarray([spec.order for spec in CANDIDATES], dtype=np.int64),
            ) or not np.array_equal(
                archive["candidate_lambda"],
                np.asarray([spec.regularization for spec in CANDIDATES]),
            ):
                raise common.F1Error("F1 candidate identity changed")
            scale = np.asarray(archive["gain_scale"], dtype=np.float64)
            candidates = []
            for candidate_index in range(len(CANDIDATES)):
                trained = PriorNetwork().to(dtype=torch.float64, device="cpu")
                state = {
                    name: _tensor(
                        np.asarray(
                            archive[f"c{candidate_index:02d}_p{parameter_index:03d}"]
                        )
                    )
                    for parameter_index, name in enumerate(names)
                }
                trained.load_state_dict(state)
                trained.eval()
                _verify_model(trained)
                candidates.append(trained)
            paired = PriorNetwork().to(dtype=torch.float64, device="cpu")
            paired.load_state_dict(
                {
                    name: _tensor(np.asarray(archive[f"h_p{parameter_index:03d}"]))
                    for parameter_index, name in enumerate(names)
                }
            )
            paired.eval()
            _verify_model(paired)
    except (OSError, ValueError) as error:
        raise common.F1Error("F1 models artifact is invalid") from error
    if scale.shape != (common.MODE_COUNT,) or not np.isfinite(scale).all():
        raise common.F1Error("F1 decoded gain scale changed")
    return tuple(candidates), paired, scale
