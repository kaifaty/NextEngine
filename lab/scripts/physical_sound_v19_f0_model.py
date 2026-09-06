#!/usr/bin/env python3
"""Frozen learned prior and intrinsic residual operators for V19 F0."""

from __future__ import annotations

import io
import math
import warnings
from dataclasses import dataclass
from typing import Any

import numpy as np
import physical_sound_v19_f0_common as common
import torch
from scipy.sparse import csr_matrix, diags
from scipy.sparse.csgraph import connected_components
from scipy.sparse.linalg import MatrixRankWarning, spsolve
from scipy.spatial.distance import cdist
from torch import nn

SEED = 190_001
WIDTH = 128
UPDATES = 1_500
LEARNING_RATE_START = 2.0e-3
LEARNING_RATE_END = 1.0e-5
WEIGHT_DECAY = 1.0e-6
GRADIENT_LOSS_WEIGHT = 0.15
RBF_BANDWIDTH = 0.20
PARAMETER_COUNT = 41_992
PARAMETER_BYTES = 335_936
MODEL_BYTE_LIMIT = 400_000


@dataclass(frozen=True)
class TrainingResult:
    model: PriorNetwork
    gain_scale: np.ndarray
    initial_loss: float
    final_loss: float

    def summary(self) -> dict[str, Any]:
        return {
            "final_loss": self.final_loss,
            "gain_scale": self.gain_scale,
            "initial_loss": self.initial_loss,
            "learning_rate_end": LEARNING_RATE_END,
            "learning_rate_start": LEARNING_RATE_START,
            "parameter_bytes": parameter_bytes(self.model),
            "parameter_count": parameter_count(self.model),
            "seed": SEED,
            "updates": UPDATES,
            "weight_decay": WEIGHT_DECAY,
        }


class PriorNetwork(nn.Module):
    """Topology-conditioned pointwise modal-gain prior."""

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


def _tensor(value: np.ndarray) -> torch.Tensor:
    return torch.tensor(np.asarray(value), dtype=torch.float64, device="cpu")


def positional_features(uv: np.ndarray) -> np.ndarray:
    value = np.asarray(uv, dtype=np.float64)
    columns: list[np.ndarray] = []
    u = value[:, 0]
    v = value[:, 1]
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
        raise common.F0Error("F0 positional feature changed")
    return result


def prior_features(item: common.FieldObject) -> np.ndarray:
    vertices = item.mesh.vertices
    minimum = np.min(vertices, axis=0)
    maximum = np.max(vertices, axis=0)
    center = 0.5 * (minimum + maximum)
    half_extent = 0.5 * (maximum - minimum)
    safe_extent = np.where(half_extent > 1.0e-12, half_extent, 1.0)
    local = np.column_stack(
        (
            item.mesh.uv,
            (vertices - center) / safe_extent,
            item.normals,
            item.curvatures * item.row.length_m,
            positional_features(item.mesh.uv),
        )
    )
    static = np.asarray(
        [item.row.material == value for value in common.MATERIAL_ORDER]
        + [item.row.topology == value for value in common.TOPOLOGY_ORDER]
        + [item.row.support == value for value in common.SUPPORT_ORDER]
        + [math.log(item.row.aspect), math.log(item.row.slenderness)],
        dtype=np.float64,
    )
    result = np.column_stack((local, np.tile(static, (vertices.shape[0], 1))))
    if result.shape != (item.mesh.vertex_count, 61) or not np.isfinite(result).all():
        raise common.F0Error("F0 prior feature changed")
    return result


def fit_gain_scale(train: tuple[common.FieldObject, ...]) -> np.ndarray:
    if len(train) != 24 or any(item.row.role != "train" for item in train):
        raise common.F0Error("F0 gain scale requires 24 train objects")
    values = np.concatenate([item.gains[item.accepted_query] for item in train], axis=0)
    scale = np.sqrt(np.mean(values**2, axis=0))
    if scale.shape != (common.MODE_COUNT,) or np.any(scale <= 1.0e-12):
        raise common.F0Error("F0 gain scale is degenerate")
    return scale


def _active_edges(item: common.FieldObject) -> np.ndarray:
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    edge = item.mesh.edges
    return edge[active[edge[:, 0]] & active[edge[:, 1]]]


def train_prior(train: tuple[common.FieldObject, ...]) -> TrainingResult:
    gain_scale = fit_gain_scale(train)
    prepared = [
        (
            item,
            _tensor(prior_features(item)),
            _tensor(item.gains / gain_scale),
        )
        for item in train
    ]
    torch.manual_seed(SEED)
    model = PriorNetwork().to(dtype=torch.float64, device="cpu")
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=LEARNING_RATE_START, weight_decay=WEIGHT_DECAY
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
            prediction = model(features)
            exact = prediction.clone()
            exact[item.context] = truth[item.context]
            query_loss = torch.mean(
                (prediction[item.accepted_query] - truth[item.accepted_query]) ** 2
            )
            edge = _active_edges(item)
            context_mask = np.zeros(item.mesh.vertex_count, dtype=bool)
            context_mask[item.context] = True
            edge = edge[~(context_mask[edge[:, 0]] & context_mask[edge[:, 1]])]
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
            raise common.F0Error("F0 training loss is non-finite")
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        value = float(loss.detach())
        if update == 0:
            initial_loss = value
        if update == UPDATES - 1:
            final_loss = value
    if not all(torch.isfinite(parameter).all() for parameter in model.parameters()):
        raise common.F0Error("F0 trained model is non-finite")
    model.eval()
    if (
        parameter_count(model) != PARAMETER_COUNT
        or parameter_bytes(model) != PARAMETER_BYTES
    ):
        raise common.F0Error("F0 model size changed")
    return TrainingResult(model, gain_scale, initial_loss, final_loss)


def parameter_count(model: PriorNetwork) -> int:
    return sum(parameter.numel() for parameter in model.parameters())


def parameter_bytes(model: PriorNetwork) -> int:
    return sum(
        parameter.numel() * parameter.element_size() for parameter in model.parameters()
    )


def encode_model(model: PriorNetwork, gain_scale: np.ndarray) -> bytes:
    names = sorted(model.state_dict())
    arrays = {
        f"p{index:03d}": model.state_dict()[name].detach().cpu().numpy()
        for index, name in enumerate(names)
    }
    arrays["gain_scale"] = np.asarray(gain_scale, dtype=np.float64)
    payload = common.deterministic_npz(arrays)
    if parameter_bytes(model) > MODEL_BYTE_LIMIT:
        raise common.F0Error("F0 model parameter byte limit exceeded")
    return payload


def decode_model(payload: bytes) -> tuple[PriorNetwork, np.ndarray]:
    model = PriorNetwork().to(dtype=torch.float64, device="cpu")
    names = sorted(model.state_dict())
    expected = {f"p{index:03d}" for index in range(len(names))} | {"gain_scale"}
    try:
        with np.load(io.BytesIO(payload), allow_pickle=False) as archive:
            if set(archive.files) != expected:
                raise common.F0Error("F0 model NPZ member set changed")
            state = {
                name: _tensor(np.asarray(archive[f"p{index:03d}"]))
                for index, name in enumerate(names)
            }
            gain_scale = np.asarray(archive["gain_scale"], dtype=np.float64)
    except (OSError, ValueError) as error:
        raise common.F0Error("F0 model artifact is invalid") from error
    expected_state = model.state_dict()
    if any(state[name].shape != expected_state[name].shape for name in names):
        raise common.F0Error("F0 model parameter shape changed")
    if gain_scale.shape != (common.MODE_COUNT,) or not np.isfinite(gain_scale).all():
        raise common.F0Error("F0 model gain scale changed")
    model.load_state_dict(state)
    model.eval()
    return model, gain_scale


def _active_mask(item: common.FieldObject) -> np.ndarray:
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    return active


def _intrinsic_harmonic(
    item: common.FieldObject, context_residual: np.ndarray
) -> np.ndarray:
    edge = item.mesh.edges
    conductance = 1.0 / np.maximum(item.mesh.edge_lengths, 1.0e-12)
    adjacency = csr_matrix(
        (
            np.concatenate((conductance, conductance)),
            (
                np.concatenate((edge[:, 0], edge[:, 1])),
                np.concatenate((edge[:, 1], edge[:, 0])),
            ),
        ),
        shape=(item.mesh.vertex_count, item.mesh.vertex_count),
    )
    component_count, labels = connected_components(adjacency, directed=False)
    if component_count != 1 or labels.size != item.mesh.vertex_count:
        raise common.F0Error("F0 harmonic domain is disconnected")
    laplacian = diags(np.asarray(adjacency.sum(axis=1)).ravel()) - adjacency
    query = item.query
    context = item.context
    result = np.zeros((item.mesh.vertex_count, common.MODE_COUNT), dtype=np.float64)
    result[context] = context_residual
    with warnings.catch_warnings():
        warnings.simplefilter("error", MatrixRankWarning)
        try:
            solved = spsolve(
                laplacian[np.ix_(query, query)],
                -(laplacian[np.ix_(query, context)] @ context_residual),
            )
        except (MatrixRankWarning, RuntimeError, ValueError) as error:
            raise common.F0Error("F0 harmonic solve failed") from error
    result[query] = np.asarray(solved, dtype=np.float64)
    if not np.isfinite(result).all():
        raise common.F0Error("F0 harmonic result is non-finite")
    return result


def _rbf_residual(
    item: common.FieldObject,
    context_residual: np.ndarray,
    intrinsic: bool,
) -> np.ndarray:
    active = _active_mask(item)
    output = np.zeros((item.mesh.vertex_count, common.MODE_COUNT), dtype=np.float64)
    if intrinsic:
        distance = (
            item.analysis.all_pairs[:, item.context] / item.analysis.graph_diameter
        )
    else:
        vertices = item.mesh.vertices
        diameter = float(np.max(cdist(vertices, vertices)))
        distance = cdist(vertices, vertices[item.context]) / diameter
    weight = np.exp(-0.5 * (distance / RBF_BANDWIDTH) ** 2)
    output[active] = (weight[active] @ context_residual) / np.sum(
        weight[active], axis=1, keepdims=True
    )
    output[item.context] = context_residual
    return output


def _prior_prediction(model: PriorNetwork, item: common.FieldObject) -> np.ndarray:
    active = _active_mask(item)
    result = np.zeros((item.mesh.vertex_count, common.MODE_COUNT), dtype=np.float64)
    with torch.no_grad():
        result[active] = model(_tensor(prior_features(item)[active])).cpu().numpy()
    if not np.isfinite(result[active]).all():
        raise common.F0Error("F0 prior prediction is non-finite")
    return result


def predict_methods(
    model: PriorNetwork,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    observed_context: np.ndarray | None = None,
) -> dict[str, np.ndarray]:
    scale = np.asarray(gain_scale, dtype=np.float64)
    observed = (
        item.gains[item.context]
        if observed_context is None
        else np.asarray(observed_context, dtype=np.float64)
    )
    if observed.shape != (item.context.size, common.MODE_COUNT):
        raise common.F0Error("F0 observed context shape changed")
    normalized_observed = observed / scale
    prior = _prior_prediction(model, item)
    residual = normalized_observed - prior[item.context]
    active = _active_mask(item)

    def finalized(value: np.ndarray) -> np.ndarray:
        result = np.zeros_like(value)
        result[active] = value[active] * scale
        result[item.context] = observed
        if not np.isfinite(result[active]).all():
            raise common.F0Error("F0 method prediction is non-finite")
        return result

    candidate = finalized(prior + _intrinsic_harmonic(item, residual))
    prior_only = finalized(prior)
    prior_geodesic = finalized(prior + _rbf_residual(item, residual, True))
    prior_euclidean = finalized(prior + _rbf_residual(item, residual, False))
    zero = np.zeros_like(prior)
    raw_harmonic = finalized(_intrinsic_harmonic(item, normalized_observed))
    raw_geodesic = finalized(_rbf_residual(item, normalized_observed, True))
    raw_euclidean = finalized(_rbf_residual(item, normalized_observed, False))
    context_mean = zero.copy()
    context_mean[active] = np.mean(normalized_observed, axis=0)
    context_mean = finalized(context_mean)
    distances = item.analysis.all_pairs[:, item.context]
    nearest_index = np.argmin(distances, axis=1)
    nearest = zero.copy()
    nearest[active] = normalized_observed[nearest_index[active]]
    nearest = finalized(nearest)
    return {
        "candidate": candidate,
        "context-mean": context_mean,
        "nearest-intrinsic": nearest,
        "prior-euclidean-rbf": prior_euclidean,
        "prior-geodesic-rbf": prior_geodesic,
        "prior-only": prior_only,
        "raw-euclidean-rbf": raw_euclidean,
        "raw-geodesic-rbf": raw_geodesic,
        "raw-harmonic": raw_harmonic,
    }


def gain_nrmse(item: common.FieldObject, prediction: np.ndarray) -> float:
    query = item.accepted_query
    numerator = np.sqrt(np.mean((prediction[query] - item.gains[query]) ** 2))
    denominator = np.sqrt(np.mean(item.gains[query] ** 2))
    if denominator <= 1.0e-12:
        raise common.F0Error("F0 gain NRMSE denominator changed")
    return float(numerator / denominator)


def edge_gradient_p99(item: common.FieldObject, prediction: np.ndarray) -> float:
    edge = _active_edges(item)
    delta = (
        prediction[edge[:, 0]]
        - prediction[edge[:, 1]]
        - item.gains[edge[:, 0]]
        + item.gains[edge[:, 1]]
    )
    denominator = float(np.sqrt(np.mean(item.gains**2)))
    if denominator <= 1.0e-12:
        raise common.F0Error("F0 gradient denominator changed")
    per_edge = np.sqrt(np.mean(delta**2, axis=1)) / denominator
    return float(np.quantile(per_edge, 0.99))


def metrics(item: common.FieldObject, prediction: np.ndarray) -> dict[str, float]:
    return {
        "edge_gradient_p99": edge_gradient_p99(item, prediction),
        "gain_nrmse": gain_nrmse(item, prediction),
    }
