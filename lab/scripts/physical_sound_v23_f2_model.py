#!/usr/bin/env python3
"""Closed-form fixed-feature ridge prior and continuous residual for V23 F2."""

from __future__ import annotations

import io
import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import physical_sound_v19_f0_model as f0_model
import physical_sound_v21_f1_model as reference
import physical_sound_v23_f2_common as common

BANK_SEED = 230_001
MAX_DIMENSION = 128
PRIOR_RIDGE = 1.0e-4
RESIDUAL_RIDGE = 1.0e-4
CONDITION_LIMIT = 1.0e12
MODEL_BYTE_LIMIT = 128 * 1024
PARAMETER_BYTES = 75_648
BASIS_REVISION = "topology-native-continuous-tensor-v1"

REQUIRED_MODEL_API = (
    "candidate_specs",
    "fit_candidates",
    "validate_request",
    "predict_candidate",
    "predict_direct_probes",
    "compatible_predictions",
    "encode_models",
    "decode_models",
    "parameter_bytes",
)


@dataclass(frozen=True)
class CandidateSpec:
    candidate_id: str
    dimension: int
    bandwidth: float
    order: int
    regularization: float = RESIDUAL_RIDGE


CANDIDATES = (
    CandidateSpec("ffr-d64-s0p5-q2", 64, 0.5, 2),
    CandidateSpec("ffr-d64-s0p5-q3", 64, 0.5, 3),
    CandidateSpec("ffr-d64-s1p0-q2", 64, 1.0, 2),
    CandidateSpec("ffr-d64-s1p0-q3", 64, 1.0, 3),
    CandidateSpec("ffr-d128-s0p5-q2", 128, 0.5, 2),
    CandidateSpec("ffr-d128-s0p5-q3", 128, 0.5, 3),
    CandidateSpec("ffr-d128-s1p0-q2", 128, 1.0, 2),
    CandidateSpec("ffr-d128-s1p0-q3", 128, 1.0, 3),
)


@dataclass(frozen=True)
class FixedFeatureModel:
    coefficients: np.ndarray
    dimension: int
    bandwidth: float


@dataclass(frozen=True)
class TrainingResult:
    model: FixedFeatureModel
    gain_scale: np.ndarray
    initial_loss: float
    final_loss: float
    spec: CandidateSpec | None
    seed: int
    condition_number: float

    def summary(self) -> dict[str, Any]:
        return {
            "bandwidth": self.model.bandwidth,
            "candidate_id": None if self.spec is None else self.spec.candidate_id,
            "condition_number": self.condition_number,
            "dimension": self.model.dimension,
            "gain_scale": self.gain_scale,
            "parameter_bytes": parameter_bytes(self.model),
            "prior_ridge": PRIOR_RIDGE,
            "seed": self.seed,
            "training_kind": "closed-form-weighted-ridge",
            "updates": 0,
        }


@dataclass(frozen=True)
class ContinuousSystem:
    basis: np.ndarray
    cholesky: np.ndarray
    condition_number: float


def candidate_specs() -> tuple[CandidateSpec, ...]:
    return CANDIDATES


def _bank() -> tuple[np.ndarray, np.ndarray]:
    generator = np.random.Generator(np.random.PCG64(BANK_SEED))
    weights = generator.standard_normal((MAX_DIMENSION, 61), dtype=np.float64)
    phase = generator.uniform(0.0, 2.0 * np.pi, MAX_DIMENSION)
    if weights.shape != (128, 61) or phase.shape != (128,):
        raise common.F1Error("F2 fixed feature bank changed")
    return weights, phase


def normalized_analytic_features_at(row: common.FieldRow, uv: np.ndarray) -> np.ndarray:
    value = reference.prior_features_at(row, uv).copy()
    value[:, 8:10] /= 2.0 * np.pi
    aspect_center = 0.5 * math.log(0.72 * 1.48)
    aspect_half_range = 0.5 * math.log(1.48 / 0.72)
    slender_center = 0.5 * math.log(0.004 * 0.008)
    slender_half_range = 0.5 * math.log(0.008 / 0.004)
    value[:, 59] = (value[:, 59] - aspect_center) / aspect_half_range
    value[:, 60] = (value[:, 60] - slender_center) / slender_half_range
    if value.shape != (uv.shape[0], 61) or not np.isfinite(value).all():
        raise common.F1Error("F2 normalized analytic feature changed")
    return value


def _feature_map(value: np.ndarray, dimension: int, bandwidth: float) -> np.ndarray:
    source = np.asarray(value, dtype=np.float64)
    if source.ndim != 2 or source.shape[1] != 61 or dimension not in (0, 64, 128):
        raise common.F1Error("F2 fixed feature input changed")
    if dimension == 0:
        result = np.column_stack((np.ones(source.shape[0]), source))
    else:
        weights, phase = _bank()
        nonlinear = math.sqrt(2.0 / dimension) * np.cos(
            source @ weights[:dimension].T / bandwidth + phase[:dimension]
        )
        result = np.column_stack((np.ones(source.shape[0]), source, nonlinear))
    expected = 62 + dimension
    if result.shape != (source.shape[0], expected) or not np.isfinite(result).all():
        raise common.F1Error("F2 fixed feature map changed")
    return result


def _features_for_item(
    item: common.FieldObject, dimension: int, bandwidth: float
) -> np.ndarray:
    return _feature_map(
        normalized_analytic_features_at(item.row, item.mesh.uv),
        dimension,
        bandwidth,
    )


def _fit_model(
    train: tuple[common.FieldObject, ...],
    gain_scale: np.ndarray,
    dimension: int,
    bandwidth: float,
) -> tuple[FixedFeatureModel, float]:
    width = 62 + dimension
    gram = np.zeros((width, width), dtype=np.float64)
    right = np.zeros((width, common.MODE_COUNT), dtype=np.float64)
    for item in train:
        features = _features_for_item(item, dimension, bandwidth)[item.accepted_query]
        target = item.gains[item.accepted_query] / gain_scale
        gram += features.T @ features / item.accepted_query.size
        right += features.T @ target / item.accepted_query.size
    gram /= len(train)
    right /= len(train)
    penalty = np.ones(width, dtype=np.float64)
    penalty[0] = 0.0
    gram += PRIOR_RIDGE * np.diag(penalty)
    condition = float(np.linalg.cond(gram, p=2))
    if not math.isfinite(condition) or condition > CONDITION_LIMIT:
        raise common.F1Error("F2 prior ridge condition rejected")
    try:
        cholesky = np.linalg.cholesky(gram)
        coefficients = np.linalg.solve(cholesky, right)
        coefficients = np.linalg.solve(cholesky.T, coefficients)
    except np.linalg.LinAlgError as error:
        raise common.F1Error("F2 prior ridge solve rejected") from error
    result = FixedFeatureModel(coefficients, dimension, bandwidth)
    if parameter_bytes(result) > MODEL_BYTE_LIMIT:
        raise common.F1Error("F2 model byte ceiling exceeded")
    return result, condition


def fit_candidates(
    train: tuple[common.FieldObject, ...],
) -> tuple[TrainingResult, ...]:
    gain_scale = reference.fit_gain_scale(train)
    results = []
    for spec in CANDIDATES:
        trained, condition = _fit_model(
            train, gain_scale, spec.dimension, spec.bandwidth
        )
        results.append(
            TrainingResult(
                trained,
                gain_scale,
                math.nan,
                math.nan,
                spec,
                BANK_SEED,
                condition,
            )
        )
    return tuple(results)


def train_candidates(
    train: tuple[common.FieldObject, ...],
) -> tuple[TrainingResult, ...]:
    return fit_candidates(train)


def _fit_analytic_control(
    train: tuple[common.FieldObject, ...], gain_scale: np.ndarray
) -> TrainingResult:
    trained, condition = _fit_model(train, gain_scale, 0, 1.0)
    return TrainingResult(
        trained,
        gain_scale,
        math.nan,
        math.nan,
        None,
        BANK_SEED,
        condition,
    )


def train_paired_harmonic(
    train: tuple[common.FieldObject, ...], gain_scale: np.ndarray
) -> TrainingResult:
    expected = reference.fit_gain_scale(train)
    if not np.array_equal(expected, gain_scale):
        raise common.F1Error("F2 analytic control gain scale changed")
    return _fit_analytic_control(train, gain_scale)


def _prior_at(
    trained: FixedFeatureModel, row: common.FieldRow, uv: np.ndarray
) -> np.ndarray:
    features = _feature_map(
        normalized_analytic_features_at(row, uv),
        trained.dimension,
        trained.bandwidth,
    )
    result = features @ trained.coefficients
    if (
        result.shape != (uv.shape[0], common.MODE_COUNT)
        or not np.isfinite(result).all()
    ):
        raise common.F1Error("F2 prior prediction changed")
    return result


def _prior_prediction(
    trained: FixedFeatureModel, item: common.FieldObject
) -> np.ndarray:
    active = np.zeros(item.mesh.vertex_count, dtype=bool)
    active[item.context] = True
    active[item.accepted_query] = True
    result = np.zeros((item.mesh.vertex_count, common.MODE_COUNT), dtype=np.float64)
    all_prior = _prior_at(trained, item.row, item.mesh.uv)
    result[active] = all_prior[active]
    return result


def _prepare_system(item: common.FieldObject, spec: CandidateSpec) -> ContinuousSystem:
    validate_request(item, spec, expected_role=item.row.role)
    basis, penalty = reference.basis_matrix(item.row.topology, item.mesh.uv, spec.order)
    context_basis = basis[item.context]
    matrix = context_basis.T @ context_basis / item.context.size
    matrix += spec.regularization * np.diag(penalty)
    condition = float(np.linalg.cond(matrix, p=2))
    if not math.isfinite(condition) or condition > CONDITION_LIMIT:
        raise common.F1Error("F2 residual system condition rejected")
    try:
        cholesky = np.linalg.cholesky(matrix)
    except np.linalg.LinAlgError as error:
        raise common.F1Error("F2 residual system rejected") from error
    return ContinuousSystem(basis, cholesky, condition)


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
    if spec not in CANDIDATES:
        raise common.F1Error("F2 candidate identity changed")
    role = item.row.role if declared_role is None else declared_role
    if (
        expected_role not in ("train", "development")
        or item.row.role != expected_role
        or role != expected_role
    ):
        raise common.F1Error("F2 role identity changed")
    if declared_basis_revision != BASIS_REVISION:
        raise common.F1Error("F2 basis identity changed")
    regularization = (
        spec.regularization
        if declared_regularization is None
        else declared_regularization
    )
    if regularization != spec.regularization:
        raise common.F1Error("F2 regularization identity changed")
    if available_model_bytes < PARAMETER_BYTES:
        raise common.F1Error("F2 model budget is insufficient")
    if coefficient is not None:
        expected = ((1 + 2 * spec.order) ** 2, common.MODE_COUNT)
        value = np.asarray(coefficient, dtype=np.float64)
        if value.shape != expected or not np.isfinite(value).all():
            raise common.F1Error("F2 residual coefficient changed")


def _residual_coefficients(
    prior: np.ndarray,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
    observed: np.ndarray,
) -> tuple[ContinuousSystem, np.ndarray]:
    system = _prepare_system(item, spec)
    residual = observed / gain_scale - prior[item.context]
    context_basis = system.basis[item.context]
    right = context_basis.T @ residual / item.context.size
    coefficient = np.linalg.solve(system.cholesky, right)
    coefficient = np.linalg.solve(system.cholesky.T, coefficient)
    validate_request(
        item,
        spec,
        expected_role="development",
        coefficient=coefficient,
    )
    return system, coefficient


def predict_candidate(
    trained: FixedFeatureModel,
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
        raise common.F1Error("F2 observed context shape changed")
    prior = _prior_prediction(trained, item)
    system, coefficient = _residual_coefficients(
        prior, gain_scale, item, spec, observed
    )
    normalized = prior + system.basis @ coefficient
    return reference._finalize(normalized, gain_scale, item, observed)


def predict_continuous(
    trained: FixedFeatureModel,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
    observed_context: np.ndarray | None = None,
) -> np.ndarray:
    return predict_candidate(trained, gain_scale, item, spec, observed_context)


def predict_direct_probes(
    trained: FixedFeatureModel,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
) -> np.ndarray:
    prior = _prior_prediction(trained, item)
    _system, coefficient = _residual_coefficients(
        prior, gain_scale, item, spec, item.gains[item.context]
    )
    probes = reference._probe_grid()
    probe_prior = _prior_at(trained, item.row, probes)
    probe_basis = reference.basis_matrix(item.row.topology, probes, spec.order)[0]
    result = gain_scale * (probe_prior + probe_basis @ coefficient)
    if result.shape != (49, common.MODE_COUNT) or not np.isfinite(result).all():
        raise common.F1Error("F2 direct probe prediction changed")
    return result


def direct_probe_prediction(
    trained: FixedFeatureModel,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    spec: CandidateSpec,
) -> np.ndarray:
    return predict_direct_probes(trained, gain_scale, item, spec)


def predict_paired_harmonic(
    trained: FixedFeatureModel, gain_scale: np.ndarray, item: common.FieldObject
) -> np.ndarray:
    prior = _prior_prediction(trained, item)
    observed = item.gains[item.context]
    residual = observed / gain_scale - prior[item.context]
    normalized = prior + f0_model._intrinsic_harmonic(item, residual)
    return reference._finalize(normalized, gain_scale, item, observed)


def compatible_predictions(
    trained: FixedFeatureModel,
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
    system = _prepare_system(item, spec)
    context_basis = system.basis[item.context]
    right = context_basis.T @ normalized_observed / item.context.size
    coefficient = np.linalg.solve(system.cholesky, right)
    coefficient = np.linalg.solve(system.cholesky.T, coefficient)
    raw_continuous = system.basis @ coefficient
    return {
        "candidate": predict_candidate(trained, gain_scale, item, spec),
        "context-mean": reference._finalize(context_mean, gain_scale, item, observed),
        "nearest-intrinsic": reference._finalize(nearest, gain_scale, item, observed),
        "prior-euclidean-rbf": reference._finalize(
            prior_euclidean, gain_scale, item, observed
        ),
        "prior-only": reference._finalize(prior, gain_scale, item, observed),
        "raw-continuous": reference._finalize(
            raw_continuous, gain_scale, item, observed
        ),
        "raw-euclidean-rbf": reference._finalize(
            raw_euclidean, gain_scale, item, observed
        ),
        "raw-geodesic-rbf": reference._finalize(raw_geo, gain_scale, item, observed),
        "raw-harmonic": reference._finalize(raw_harmonic, gain_scale, item, observed),
    }


def parameter_bytes(trained: FixedFeatureModel) -> int:
    bank_bytes = 0
    if trained.dimension:
        bank_bytes = (trained.dimension * 61 + trained.dimension) * 8
    return bank_bytes + trained.coefficients.nbytes


def encode_models(
    candidates: tuple[TrainingResult, ...], control: TrainingResult
) -> bytes:
    if tuple(value.spec for value in candidates) != CANDIDATES:
        raise common.F1Error("F2 candidate model order changed")
    for value, spec in zip(candidates, CANDIDATES, strict=True):
        expected = (62 + spec.dimension, common.MODE_COUNT)
        if (
            value.model.dimension != spec.dimension
            or value.model.bandwidth != spec.bandwidth
            or value.model.coefficients.shape != expected
            or not np.isfinite(value.model.coefficients).all()
        ):
            raise common.F1Error("F2 candidate model identity changed")
    if (
        control.spec is not None
        or control.model.dimension != 0
        or control.model.bandwidth != 1.0
        or control.model.coefficients.shape != (62, common.MODE_COUNT)
        or not np.isfinite(control.model.coefficients).all()
    ):
        raise common.F1Error("F2 analytic control identity changed")
    scale = candidates[0].gain_scale
    if any(
        not np.array_equal(value.gain_scale, scale) for value in candidates
    ) or not np.array_equal(control.gain_scale, scale):
        raise common.F1Error("F2 model gain scale changed")
    weights, phase = _bank()
    arrays: dict[str, np.ndarray] = {
        "bank_phase": phase,
        "bank_weights": weights,
        "candidate_bandwidth": np.asarray(
            [value.bandwidth for value in CANDIDATES], dtype=np.float64
        ),
        "candidate_dimension": np.asarray(
            [value.dimension for value in CANDIDATES], dtype=np.int64
        ),
        "candidate_order": np.asarray(
            [value.order for value in CANDIDATES], dtype=np.int64
        ),
        "control_coefficients": control.model.coefficients,
        "gain_scale": scale,
    }
    for index, value in enumerate(candidates):
        arrays[f"candidate_{index:02d}_coefficients"] = value.model.coefficients
    return common.deterministic_npz(arrays)


def decode_models(
    payload: bytes,
) -> tuple[tuple[FixedFeatureModel, ...], FixedFeatureModel, np.ndarray]:
    expected = {
        "bank_phase",
        "bank_weights",
        "candidate_bandwidth",
        "candidate_dimension",
        "candidate_order",
        "control_coefficients",
        "gain_scale",
        *{f"candidate_{index:02d}_coefficients" for index in range(len(CANDIDATES))},
    }
    try:
        with np.load(io.BytesIO(payload), allow_pickle=False) as archive:
            if set(archive.files) != expected:
                raise common.F1Error("F2 models NPZ member set changed")
            weights, phase = _bank()
            if not np.array_equal(
                archive["bank_weights"], weights
            ) or not np.array_equal(archive["bank_phase"], phase):
                raise common.F1Error("F2 fixed bank identity changed")
            if (
                not np.array_equal(
                    archive["candidate_dimension"],
                    np.asarray([value.dimension for value in CANDIDATES]),
                )
                or not np.array_equal(
                    archive["candidate_bandwidth"],
                    np.asarray([value.bandwidth for value in CANDIDATES]),
                )
                or not np.array_equal(
                    archive["candidate_order"],
                    np.asarray([value.order for value in CANDIDATES]),
                )
            ):
                raise common.F1Error("F2 candidate identity changed")
            scale = np.asarray(archive["gain_scale"], dtype=np.float64)
            candidates = tuple(
                FixedFeatureModel(
                    np.asarray(
                        archive[f"candidate_{index:02d}_coefficients"],
                        dtype=np.float64,
                    ),
                    spec.dimension,
                    spec.bandwidth,
                )
                for index, spec in enumerate(CANDIDATES)
            )
            control = FixedFeatureModel(
                np.asarray(archive["control_coefficients"], dtype=np.float64), 0, 1.0
            )
    except (OSError, ValueError) as error:
        raise common.F1Error("F2 models artifact is invalid") from error
    if scale.shape != (common.MODE_COUNT,) or not np.isfinite(scale).all():
        raise common.F1Error("F2 decoded scale changed")
    for trained, expected in (
        *(
            (trained, (62 + spec.dimension, common.MODE_COUNT))
            for trained, spec in zip(candidates, CANDIDATES, strict=True)
        ),
        (control, (62, common.MODE_COUNT)),
    ):
        if (
            trained.coefficients.shape != expected
            or not np.isfinite(trained.coefficients).all()
            or parameter_bytes(trained) > MODEL_BYTE_LIMIT
        ):
            raise common.F1Error("F2 decoded model changed")
    return candidates, control, scale
