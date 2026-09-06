#!/usr/bin/env python3
"""Batched, algebraically equivalent V22 F1r model training."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import physical_sound_v21_f1_model as reference
import physical_sound_v22_f1r_common as common
import torch

PriorNetwork = reference.PriorNetwork
CandidateSpec = reference.CandidateSpec
ContinuousSystem = reference.ContinuousSystem
TrainingResult = reference.TrainingResult
CANDIDATES = reference.CANDIDATES
WIDTH = reference.WIDTH
UPDATES = reference.UPDATES
LEARNING_RATE_START = reference.LEARNING_RATE_START
LEARNING_RATE_END = reference.LEARNING_RATE_END
WEIGHT_DECAY = reference.WEIGHT_DECAY
GRADIENT_LOSS_WEIGHT = reference.GRADIENT_LOSS_WEIGHT
PAIR_LOSS_WEIGHT = reference.PAIR_LOSS_WEIGHT
CANDIDATE_SEED = reference.CANDIDATE_SEED
PAIRED_HARMONIC_SEED = reference.PAIRED_HARMONIC_SEED
PARAMETER_COUNT = reference.PARAMETER_COUNT
PARAMETER_BYTES = reference.PARAMETER_BYTES
MODEL_BYTE_LIMIT = reference.MODEL_BYTE_LIMIT
BASIS_REVISION = reference.BASIS_REVISION

analytic_surface = reference.analytic_surface
positional_features = reference.positional_features
prior_features_at = reference.prior_features_at
prior_features = reference.prior_features
basis_matrix = reference.basis_matrix
prepare_system = reference.prepare_system
fit_gain_scale = reference.fit_gain_scale
predict_continuous = reference.predict_continuous
direct_probe_prediction = reference.direct_probe_prediction
predict_paired_harmonic = reference.predict_paired_harmonic
compatible_predictions = reference.compatible_predictions
parameter_count = reference.parameter_count
parameter_bytes = reference.parameter_bytes
encode_models = reference.encode_models
decode_models = reference.decode_models

EQUIVALENCE_SEED = 220_001
EQUIVALENCE_TOLERANCE = 5.0e-12


@dataclass(frozen=True)
class PreparedView:
    item: common.FieldObject
    system: ContinuousSystem
    truth: torch.Tensor
    edge: torch.Tensor
    vertex_slice: slice
    probe_slice: slice
    probe_basis: torch.Tensor


def _tensor(value: np.ndarray) -> torch.Tensor:
    return torch.tensor(np.asarray(value), dtype=torch.float64, device="cpu")


def _index_tensor(value: np.ndarray) -> torch.Tensor:
    return torch.tensor(np.asarray(value), dtype=torch.int64, device="cpu")


def _slices(lengths: list[int]) -> tuple[slice, ...]:
    result = []
    offset = 0
    for length in lengths:
        result.append(slice(offset, offset + length))
        offset += length
    return tuple(result)


def _prepare_candidate(
    train: tuple[common.FieldObject, ...], spec: CandidateSpec, gain_scale: np.ndarray
) -> tuple[torch.Tensor, torch.Tensor, tuple[PreparedView, ...]]:
    features = [prior_features(item) for item in train]
    probes = reference._probe_grid()
    probe_features = [prior_features_at(item.row, probes) for item in train]
    vertex_slices = _slices([value.shape[0] for value in features])
    probe_slices = _slices([value.shape[0] for value in probe_features])
    prepared = tuple(
        PreparedView(
            item=item,
            system=prepare_system(item, spec, "train"),
            truth=_tensor(item.gains / gain_scale),
            edge=_index_tensor(reference._active_edges(item)),
            vertex_slice=vertex_slices[index],
            probe_slice=probe_slices[index],
            probe_basis=_tensor(basis_matrix(item.row.topology, probes, spec.order)[0]),
        )
        for index, item in enumerate(train)
    )
    return (
        _tensor(np.concatenate(features)),
        _tensor(np.concatenate(probe_features)),
        prepared,
    )


def _candidate_loss(
    trained: PriorNetwork,
    all_features: torch.Tensor,
    all_probe_features: torch.Tensor,
    prepared: tuple[PreparedView, ...],
) -> torch.Tensor:
    all_prior = trained(all_features)
    all_probe_prior = trained(all_probe_features)
    view_losses = []
    coefficients = []
    by_group: dict[str, list[int]] = {}
    for index, value in enumerate(prepared):
        prior = all_prior[value.vertex_slice]
        residual = value.truth[value.item.context] - prior[value.item.context]
        context_basis = value.system.basis[value.item.context]
        right = context_basis.T @ residual / value.item.context.size
        coefficient = torch.cholesky_solve(right, value.system.cholesky)
        if not torch.isfinite(coefficient).all():
            raise common.F1Error("F1r residual coefficient changed")
        prediction = prior + value.system.basis @ coefficient
        prediction = prediction.clone()
        prediction[value.item.context] = value.truth[value.item.context]
        query_loss = torch.mean(
            (
                prediction[value.item.accepted_query]
                - value.truth[value.item.accepted_query]
            )
            ** 2
        )
        edge = value.edge
        gradient_loss = torch.mean(
            (
                (prediction[edge[:, 0]] - prediction[edge[:, 1]])
                - (value.truth[edge[:, 0]] - value.truth[edge[:, 1]])
            )
            ** 2
        )
        view_losses.append(query_loss + GRADIENT_LOSS_WEIGHT * gradient_loss)
        coefficients.append(coefficient)
        by_group.setdefault(value.item.row.physical_group_id, []).append(index)
    if len(by_group) != 24 or any(len(indices) != 2 for indices in by_group.values()):
        raise common.F1Error("F1r train pair ledger changed")
    pair_losses = []
    for indices in by_group.values():
        values = []
        for index in indices:
            value = prepared[index]
            values.append(
                all_probe_prior[value.probe_slice]
                + value.probe_basis @ coefficients[index]
            )
        pair_losses.append(torch.mean((values[0] - values[1]) ** 2))
    return torch.mean(torch.stack(view_losses)) + PAIR_LOSS_WEIGHT * torch.mean(
        torch.stack(pair_losses)
    )


def _train_candidate(
    train: tuple[common.FieldObject, ...], spec: CandidateSpec
) -> TrainingResult:
    gain_scale = fit_gain_scale(train)
    all_features, all_probe_features, prepared = _prepare_candidate(
        train, spec, gain_scale
    )
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
        loss = _candidate_loss(trained, all_features, all_probe_features, prepared)
        if not torch.isfinite(loss):
            raise common.F1Error("F1r candidate loss is non-finite")
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        scalar = float(loss.detach())
        if update == 0:
            initial_loss = scalar
        if update == UPDATES - 1:
            final_loss = scalar
    trained.eval()
    reference._verify_model(trained)
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
    features = [prior_features(item) for item in train]
    slices = _slices([value.shape[0] for value in features])
    all_features = _tensor(np.concatenate(features))
    truth = tuple(_tensor(item.gains / gain_scale) for item in train)
    edges = tuple(_index_tensor(reference._active_edges(item)) for item in train)
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
        all_prediction = trained(all_features)
        losses = []
        for index, item in enumerate(train):
            prediction = all_prediction[slices[index]]
            exact = prediction.clone()
            exact[item.context] = truth[index][item.context]
            edge = edges[index]
            query_loss = torch.mean(
                (prediction[item.accepted_query] - truth[index][item.accepted_query])
                ** 2
            )
            gradient_loss = torch.mean(
                (
                    (exact[edge[:, 0]] - exact[edge[:, 1]])
                    - (truth[index][edge[:, 0]] - truth[index][edge[:, 1]])
                )
                ** 2
            )
            losses.append(query_loss + GRADIENT_LOSS_WEIGHT * gradient_loss)
        loss = torch.mean(torch.stack(losses))
        if not torch.isfinite(loss):
            raise common.F1Error("F1r paired-harmonic loss is non-finite")
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        scalar = float(loss.detach())
        if update == 0:
            initial_loss = scalar
        if update == UPDATES - 1:
            final_loss = scalar
    trained.eval()
    reference._verify_model(trained)
    return TrainingResult(
        trained,
        np.asarray(gain_scale, dtype=np.float64),
        initial_loss,
        final_loss,
        None,
        PAIRED_HARMONIC_SEED,
    )


@dataclass(frozen=True)
class FixtureView:
    features: torch.Tensor
    probes: torch.Tensor
    truth: torch.Tensor
    context: torch.Tensor
    query: torch.Tensor
    edge: torch.Tensor
    basis: torch.Tensor
    probe_basis: torch.Tensor
    cholesky: torch.Tensor


def _fixture_views() -> tuple[FixtureView, ...]:
    counts = (7, 9, 8, 10)
    raw_truth = []
    raw: list[tuple[np.ndarray, ...]] = []
    for view, count in enumerate(counts):
        j = np.arange(count, dtype=np.float64)[:, None]
        k = np.arange(61, dtype=np.float64)[None, :]
        m = np.arange(8, dtype=np.float64)[None, :]
        features = np.sin((view + 1) * (j + 1) * (k + 1) / 113.0) + 0.25 * np.cos(
            (view + 3) * (j + 1) * (k + 2) / 79.0
        )
        truth = 0.5 * np.sin((view + 2) * (j + 1) * (m + 1) / 37.0) + 0.2 * np.cos(
            (view + 1) * (j + 3) * (m + 2) / 43.0
        )
        context = np.asarray([0, count // 2, count - 1], dtype=np.int64)
        query = np.asarray(
            [value for value in range(count) if value not in set(context)],
            dtype=np.int64,
        )
        edge = np.asarray(
            [
                (value, value + 1)
                for value in range(count - 1)
                if not (value in set(context) and value + 1 in set(context))
            ],
            dtype=np.int64,
        )
        b = np.arange(25, dtype=np.float64)[None, :]
        basis = np.cos((view + 1) * (j + 1) * (b + 1) / 17.0) + 0.1 * np.sin(
            (view + 2) * (j + 1) * (b + 3) / 29.0
        )
        probe_j = np.arange(49, dtype=np.float64)[:, None] + 101.0
        probes = np.sin((view + 1) * (probe_j + 1) * (k + 1) / 113.0) + 0.25 * np.cos(
            (view + 3) * (probe_j + 1) * (k + 2) / 79.0
        )
        probe_basis = np.cos(
            (view + 1) * (probe_j + 1) * (b + 1) / 17.0
        ) + 0.1 * np.sin((view + 2) * (probe_j + 1) * (b + 3) / 29.0)
        frequencies = np.arange(25, dtype=np.float64)
        penalty = (
            1.0 + (frequencies % 5.0) ** 2 + np.floor(frequencies / 5.0) ** 2
        ) ** 2
        matrix = basis[context].T @ basis[context] / context.size
        matrix += 1.0e-4 * np.diag(penalty)
        raw_truth.append(truth)
        raw.append(
            (features, probes, truth, context, query, edge, basis, probe_basis, matrix)
        )
    scale = np.sqrt(
        np.mean(
            [
                np.mean(raw_truth[index][raw[index][4]] ** 2, axis=0)
                for index in range(4)
            ],
            axis=0,
        )
    )
    return tuple(
        FixtureView(
            *(_tensor(value) for value in values[:3]),
            *(_index_tensor(value) for value in values[3:6]),
            _tensor(values[6]),
            _tensor(values[7]),
            _tensor(np.linalg.cholesky(values[8])),
        )
        for values in (
            (
                features,
                probes,
                truth / scale,
                context,
                query,
                edge,
                basis,
                probe_basis,
                matrix,
            )
            for (
                features,
                probes,
                truth,
                context,
                query,
                edge,
                basis,
                probe_basis,
                matrix,
            ) in raw
        )
    )


def _fixture_loss(
    trained: PriorNetwork, views: tuple[FixtureView, ...], batched: bool
) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    if batched:
        feature_slices = _slices([value.features.shape[0] for value in views])
        probe_slices = _slices([value.probes.shape[0] for value in views])
        joined = trained(torch.cat([value.features for value in views]))
        joined_probe = trained(torch.cat([value.probes for value in views]))
        priors = [joined[value] for value in feature_slices]
        probe_priors = [joined_probe[value] for value in probe_slices]
    else:
        priors = [trained(value.features) for value in views]
        probe_priors = [trained(value.probes) for value in views]
    captures: dict[str, torch.Tensor] = {}
    coefficients = []
    view_losses = []
    for index, value in enumerate(views):
        residual = value.truth[value.context] - priors[index][value.context]
        right = value.basis[value.context].T @ residual / value.context.numel()
        coefficient = torch.cholesky_solve(right, value.cholesky)
        prediction = priors[index] + value.basis @ coefficient
        prediction = prediction.clone()
        prediction[value.context] = value.truth[value.context]
        query_loss = torch.mean(
            (prediction[value.query] - value.truth[value.query]) ** 2
        )
        edge = value.edge
        gradient_loss = torch.mean(
            (
                (prediction[edge[:, 0]] - prediction[edge[:, 1]])
                - (value.truth[edge[:, 0]] - value.truth[edge[:, 1]])
            )
            ** 2
        )
        coefficients.append(coefficient)
        view_losses.append(query_loss + GRADIENT_LOSS_WEIGHT * gradient_loss)
        captures[f"prior-{index}"] = priors[index]
        captures[f"coefficient-{index}"] = coefficient
        captures[f"view-loss-{index}"] = view_losses[-1]
    pair_losses = []
    for pair, (left, right) in enumerate(((0, 1), (2, 3))):
        left_value = probe_priors[left] + views[left].probe_basis @ coefficients[left]
        right_value = (
            probe_priors[right] + views[right].probe_basis @ coefficients[right]
        )
        pair_losses.append(torch.mean((left_value - right_value) ** 2))
        captures[f"probe-{left}"] = probe_priors[left]
        captures[f"probe-{right}"] = probe_priors[right]
        captures[f"pair-loss-{pair}"] = pair_losses[-1]
    loss = torch.mean(torch.stack(view_losses)) + PAIR_LOSS_WEIGHT * torch.mean(
        torch.stack(pair_losses)
    )
    captures["loss"] = loss
    return loss, captures


def _difference(left: torch.Tensor, right: torch.Tensor) -> tuple[float, float]:
    left_value = left.detach()
    right_value = right.detach()
    if left_value.shape != right_value.shape or not (
        torch.isfinite(left_value).all() and torch.isfinite(right_value).all()
    ):
        raise common.F1Error("F1r equivalence shape/value changed")
    difference = torch.abs(left_value - right_value)
    maximum = 0.0 if difference.numel() == 0 else float(torch.max(difference))
    denominator = torch.maximum(
        torch.ones_like(left_value),
        torch.maximum(torch.abs(left_value), torch.abs(right_value)),
    )
    scaled = (
        0.0 if difference.numel() == 0 else float(torch.max(difference / denominator))
    )
    return maximum, scaled


def equivalence_report() -> dict[str, Any]:
    views = _fixture_views()
    torch.manual_seed(EQUIVALENCE_SEED)
    baseline = PriorNetwork().to(dtype=torch.float64, device="cpu")
    reference_model = PriorNetwork().to(dtype=torch.float64, device="cpu")
    batched_model = PriorNetwork().to(dtype=torch.float64, device="cpu")
    reference_model.load_state_dict(baseline.state_dict())
    batched_model.load_state_dict(baseline.state_dict())
    reference_optimizer = torch.optim.AdamW(
        reference_model.parameters(),
        lr=LEARNING_RATE_START,
        weight_decay=WEIGHT_DECAY,
    )
    batched_optimizer = torch.optim.AdamW(
        batched_model.parameters(),
        lr=LEARNING_RATE_START,
        weight_decay=WEIGHT_DECAY,
    )
    reference_loss, reference_capture = _fixture_loss(reference_model, views, False)
    batched_loss, batched_capture = _fixture_loss(batched_model, views, True)
    reference_optimizer.zero_grad()
    batched_optimizer.zero_grad()
    reference_loss.backward()
    batched_loss.backward()
    comparisons = {
        name: _difference(reference_capture[name], batched_capture[name])
        for name in sorted(reference_capture)
    }
    for name, reference_parameter in reference_model.named_parameters():
        batched_parameter = dict(batched_model.named_parameters())[name]
        if reference_parameter.grad is None or batched_parameter.grad is None:
            raise common.F1Error("F1r equivalence gradient missing")
        comparisons[f"gradient-{name}"] = _difference(
            reference_parameter.grad, batched_parameter.grad
        )
    reference_optimizer.step()
    batched_optimizer.step()
    for name, reference_parameter in reference_model.named_parameters():
        comparisons[f"parameter-{name}"] = _difference(
            reference_parameter, dict(batched_model.named_parameters())[name]
        )
    max_absolute = max(value[0] for value in comparisons.values())
    max_scaled = max(value[1] for value in comparisons.values())
    passed = (
        max_absolute <= EQUIVALENCE_TOLERANCE and max_scaled <= EQUIVALENCE_TOLERANCE
    )
    if not passed:
        raise common.F1Error(
            f"F1r equivalence failed: absolute={max_absolute}, scaled={max_scaled}"
        )
    return {
        "comparison_count": len(comparisons),
        "max_absolute_difference": max_absolute,
        "max_scaled_difference": max_scaled,
        "passed": passed,
        "seed": EQUIVALENCE_SEED,
        "tolerance": EQUIVALENCE_TOLERANCE,
    }
