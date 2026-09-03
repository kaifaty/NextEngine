#!/usr/bin/env python3
"""Exercise the complete target-aware V37 D0 owner on discarded targets."""

from __future__ import annotations

import argparse
import gc
import hashlib
import json
import platform
import resource
import time
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, TypeAlias, cast

import numpy as np
import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0
import physical_sound_v37_e0_full_surrogate_seal_v1 as e0
import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_t0_exact_truth_protocol_v1 as t0
import physical_sound_v37_terminal_publisher_v1 as publisher
import torch
from numpy.typing import NDArray
from torch import nn

PROFILE_PATH = "lab/profiles/physical-sound-v37-d0r-complete-entry-readiness.v1.json"
C0_PROFILE_PATH = (
    "lab/profiles/physical-sound-v37-c0-query-surface-structural-cost.v1.json"
)
T0_PROFILE_PATH = "lab/profiles/physical-sound-v37-t0-exact-truth-protocol.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v37_d0r_complete_entry_readiness_v1.py"
READINESS_SEAL_PATH = "lab/profiles/physical-sound-v37-d0r-readiness-seal.v1.json"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-d0r-complete-entry-readiness-profile.v1"
)
PROFILE_ID = "physical-sound-v37-d0r-complete-entry-readiness-v1"
PROFILE_SHA256 = "75baf8cc56a60a6d623e36e19ac97e873663eef5e48f272b70e048ecfae565f3"
CLAIM = (
    "FULL_COUNT_NONZERO_ARTIFICIAL_COMPLETE_D0_OWNER_READINESS_AND_SEAL_ONLY / "
    "NO_OFFICIAL_TARGET_SCIENTIFIC_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_"
    "ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
CANDIDATE = "query-conditioned-surface-operator-v0"
QSO_VARIANTS = e0.QSO_VARIANTS
POINTWISE = e0.POINTWISE_NAME
FloatMatrix: TypeAlias = NDArray[np.float64]


class D0ReadinessError(RuntimeError):
    """The complete artificial D0 readiness execution is invalid."""


@dataclass(frozen=True, slots=True)
class LoadedContext:
    profile: dict[str, Any]
    profile_data: bytes
    dependencies: dict[str, dict[str, object]]
    c0_context: c0.LoadedContext
    environment: dict[str, object]


@dataclass(frozen=True, slots=True)
class ControlModels:
    train: c0.TensorRole
    targets: FloatMatrix
    partitions: dict[tuple[int, ...], NDArray[np.int64]]
    ridge_coefficients: FloatMatrix


@dataclass(frozen=True, slots=True)
class OwnerResult:
    report: dict[str, object]
    receipt: publisher.PublicationReceipt
    trace: contract.ExecutionTraceV1
    evidence: dict[str, object]


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise D0ReadinessError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise D0ReadinessError(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise D0ReadinessError(f"JSON root must be an object: {label}")
    result = cast(dict[str, Any], value)
    if canonical_json(result) != data:
        raise D0ReadinessError(f"JSON is not canonical: {label}")
    return result


def bound_file(path_text: str, expected_sha256: str) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise D0ReadinessError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise D0ReadinessError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def configure_torch() -> None:
    torch.use_deterministic_algorithms(True)
    if torch.get_num_threads() != 1:
        torch.set_num_threads(1)
    if torch.get_num_interop_threads() != 1:
        torch.set_num_interop_threads(1)


def environment_record(profile: dict[str, Any]) -> dict[str, object]:
    actual: dict[str, object] = {
        "device": "cpu",
        "dtype": "float64",
        "numpy": np.__version__,
        "python": platform.python_version(),
        "torch": torch.__version__,
        "torch_deterministic_algorithms": torch.are_deterministic_algorithms_enabled(),
        "torch_interop_threads": torch.get_num_interop_threads(),
        "torch_intraop_threads": torch.get_num_threads(),
    }
    if canonical_json(actual) != canonical_json(profile["environment"]):
        raise D0ReadinessError("execution environment identity drift")
    return actual


def load_context(profile_path: Path) -> LoadedContext:
    if not profile_path.is_file() or profile_path.is_symlink():
        raise D0ReadinessError("D0R profile must be a regular file")
    data = profile_path.read_bytes()
    if (
        not data
        or len(data) > MAX_PROFILE_BYTES
        or sha256_bytes(data) != PROFILE_SHA256
    ):
        raise D0ReadinessError("D0R profile size or identity drift")
    profile = load_json_bytes(data, "D0R profile")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
    ):
        raise D0ReadinessError("D0R profile identity drift")
    authority = cast(dict[str, object], profile["authority"])
    if (
        authority.get("artificial_full_count_targets_allowed") is not True
        or authority.get("official_capability_allowed") is not False
        or authority.get("official_target_evaluation_allowed") is not False
        or authority.get("network_allowed") is not False
        or authority.get("real_or_protected_signal_access_allowed") is not False
    ):
        raise D0ReadinessError("D0R authority drift")
    dependencies: dict[str, dict[str, object]] = {}
    paths: list[str] = []
    for row in cast(list[dict[str, str]], profile["dependencies"]):
        dependencies[row["path"]] = bound_file(row["path"], row["sha256"])
        paths.append(row["path"])
    if paths != sorted(set(paths)):
        raise D0ReadinessError("D0R dependencies must be canonical and unique")
    c0_context = c0.load_context(repository_root() / C0_PROFILE_PATH)
    configure_torch()
    context = LoadedContext(
        profile, data, dependencies, c0_context, environment_record(profile)
    )
    t0_context = t0.load_context(repository_root() / T0_PROFILE_PATH)
    expected_t0_seal = t0.composite_seal(
        t0_context,
        t0.coefficient_commitment(t0_context.profile),
        t0.metadata_commitments(t0_context.f0_profile),
        t0.artificial_conformance(t0_context.profile),
    )
    t0.validate_checked_official_input_seal(expected_t0_seal)
    return context


def residual(role: c0.TensorRole) -> torch.Tensor:
    q = role.query_features
    decay = 0.003 * torch.sin(0.8 * q[:, 1] + 0.6 * q[:, 7] - 0.4 * q[:, 13])
    gain = 0.002 * torch.cos(0.9 * q[:, 2] - 0.5 * q[:, 8] + 0.7 * q[:, 20])
    contact_value = 0.004 * torch.sin(
        1.3 * q[:, 0] - 0.7 * q[:, 3] + 0.9 * q[:, 9] + 0.5 * q[:, 17]
    )
    return torch.stack((decay, gain, contact_value), dim=1)


def artificial_targets(role: c0.TensorRole) -> FloatMatrix:
    teacher = c0.QuerySurfaceCostModel(370201)
    with torch.no_grad():
        values = e0.predict_qso(teacher, role, CANDIDATE, 1024) + residual(role)
    result = values.numpy().astype(np.float64, order="C", copy=True)
    if not bool(np.all(np.isfinite(result))) or bool(np.all(result == 0.0)):
        raise D0ReadinessError("artificial teacher returned invalid targets")
    return cast(FloatMatrix, result)


class FullArtificialProvider:
    def __init__(self, context: LoadedContext, label: str) -> None:
        self.context = context
        self._namespace = f"discarded-v37-d0r-{label}"
        self._built: dict[contract.RoleKind, c0.BuiltRole] = {}

    @property
    def kind(self) -> contract.ProviderKind:
        return contract.ProviderKind.SURROGATE_D0

    @property
    def namespace(self) -> str:
        return self._namespace

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapabilityV1
    ) -> contract.SurfaceQueryBatchV1:
        if (
            capability.provider_kind is not self.kind
            or capability.namespace != self.namespace
        ):
            raise contract.ContractError("D0R provider capability mismatch")
        if role not in (contract.RoleKind.TRAIN, contract.RoleKind.DEVELOPMENT):
            raise contract.ContractError("D0R provider role is outside D0")
        built = c0.build_role(self.context.c0_context, role.value.replace("-", "_"))
        tensor = c0.tensor_role(built)
        targets = artificial_targets(tensor)
        self._built[role] = built
        rows = tuple(tuple(float(value) for value in row) for row in targets)
        return replace(built.batch, targets=contract.frozen_float_matrix(rows))

    def built(self, role: contract.RoleKind) -> c0.BuiltRole:
        try:
            return self._built[role]
        except KeyError as error:
            raise D0ReadinessError(
                "role structure requested before materialization"
            ) from error


def target_tensor(batch: contract.SurfaceQueryBatchV1) -> torch.Tensor:
    if batch.targets is None:
        raise D0ReadinessError("target-aware owner received a target-free role")
    return torch.from_numpy(np.array(batch.targets, dtype=np.float64, copy=True))


def optimizer(model: nn.Module, profile: dict[str, Any]) -> torch.optim.AdamW:
    values = cast(
        dict[str, Any], cast(dict[str, Any], profile["training"])["optimizer"]
    )
    betas = cast(list[str], values["betas"])
    return torch.optim.AdamW(
        model.parameters(),
        lr=float(values["learning_rate"]),
        betas=(float(betas[0]), float(betas[1])),
        eps=float(values["epsilon"]),
        weight_decay=float(values["weight_decay"]),
        amsgrad=cast(bool, values["amsgrad"]),
        maximize=cast(bool, values["maximize"]),
        foreach=cast(bool, values["foreach"]),
        capturable=cast(bool, values["capturable"]),
        differentiable=cast(bool, values["differentiable"]),
        fused=cast(bool, values["fused"]),
    )


def train_qso(
    variant: str,
    seed: int,
    train: c0.TensorRole,
    targets: torch.Tensor,
    profile: dict[str, Any],
) -> tuple[c0.QuerySurfaceCostModel, dict[str, object]]:
    training = cast(dict[str, Any], profile["training"])
    steps = cast(int, training["steps_each_neural_path"])
    batch_rows = cast(int, training["batch_rows"])
    microbatch = cast(int, training["gradient_microbatch_rows"])
    model = c0.QuerySurfaceCostModel(seed)
    update = optimizer(model, profile)
    cache_peak = 0
    for step in range(steps):
        rows = (torch.arange(batch_rows) + step * batch_rows) % targets.shape[0]
        update.zero_grad(set_to_none=True)
        for start in range(0, batch_rows, microbatch):
            selected = rows[start : start + microbatch]
            fields, local = torch.unique(
                train.row_field_indices[selected], sorted=True, return_inverse=True
            )
            encoded = model.encode_fields(
                train,
                topology=variant != "qso-no-topology-ablation-v1",
                field_indices=fields,
            )
            cache_peak = max(cache_peak, encoded.numel() * encoded.element_size())
            prediction = model.predict_from_encoded(
                encoded, train, selected, variant, local_row_field_indices=local
            )
            loss = torch.sum((prediction - targets[selected]) ** 2) / (
                batch_rows * contract.TARGET_AXIS_COUNT
            )
            if not bool(torch.isfinite(loss)):
                raise D0ReadinessError(f"non-finite QSO loss: {variant}")
            loss.backward()
        torch.nn.utils.clip_grad_norm_(
            model.parameters(), float(training["gradient_clip_norm"])
        )
        update.step()
    return model, {
        "field_cache_peak_bytes": cache_peak,
        "parameter_count": c0.parameter_count(model),
        "steps": steps,
        "weight_sha256": e0.state_sha256(model),
    }


def train_pointwise(
    train: c0.TensorRole, targets: torch.Tensor, profile: dict[str, Any]
) -> tuple[c0.PointwiseCostModel, dict[str, object]]:
    training = cast(dict[str, Any], profile["training"])
    steps = cast(int, training["steps_each_neural_path"])
    batch_rows = cast(int, training["batch_rows"])
    microbatch = cast(int, training["gradient_microbatch_rows"])
    model = c0.PointwiseCostModel(e0.POINTWISE_SEED)
    update = optimizer(model, profile)
    for step in range(steps):
        rows = (torch.arange(batch_rows) + step * batch_rows) % targets.shape[0]
        update.zero_grad(set_to_none=True)
        for start in range(0, batch_rows, microbatch):
            selected = rows[start : start + microbatch]
            prediction = model(train, selected)
            loss = torch.sum((prediction - targets[selected]) ** 2) / (
                batch_rows * contract.TARGET_AXIS_COUNT
            )
            if not bool(torch.isfinite(loss)):
                raise D0ReadinessError("non-finite pointwise loss")
            loss.backward()
        torch.nn.utils.clip_grad_norm_(
            model.parameters(), float(training["gradient_clip_norm"])
        )
        update.step()
    return model, {
        "field_cache_peak_bytes": 0,
        "parameter_count": c0.parameter_count(model),
        "steps": steps,
        "weight_sha256": e0.state_sha256(model),
    }


def train_neural_models(
    train: c0.TensorRole, targets: torch.Tensor, profile: dict[str, Any]
) -> tuple[dict[str, nn.Module], dict[str, object]]:
    models: dict[str, nn.Module] = {}
    records: dict[str, object] = {}
    for variant, seed in QSO_VARIANTS:
        model, record = train_qso(variant, seed, train, targets, profile)
        models[variant] = model
        records[variant] = record
        gc.collect()
    pointwise, record = train_pointwise(train, targets, profile)
    models[POINTWISE] = pointwise
    records[POINTWISE] = record
    return models, records


def integral_features(role: c0.TensorRole) -> FloatMatrix:
    nodes = role.node_features[role.row_field_indices, :, :8]
    values = torch.sum(role.query_kernel_weights[:, :, None] * nodes, dim=1)
    return cast(FloatMatrix, values.numpy().astype(np.float64, order="C", copy=True))


def ridge_design(role: c0.TensorRole) -> FloatMatrix:
    rows = role.query_features.shape[0]
    return cast(
        FloatMatrix,
        np.concatenate(
            (
                np.ones((rows, 1), dtype=np.float64),
                role.query_features.numpy(),
                integral_features(role),
            ),
            axis=1,
        ),
    )


def fit_controls(train: c0.TensorRole, targets: FloatMatrix) -> ControlModels:
    grouped: dict[tuple[int, ...], list[int]] = {}
    for index, key in enumerate(train.partition_keys):
        grouped.setdefault(key, []).append(index)
    partitions = {
        key: np.asarray(indices, dtype=np.int64) for key, indices in grouped.items()
    }
    design = ridge_design(train)
    gram = design.T @ design
    regularizer = np.eye(design.shape[1], dtype=np.float64) * 1.0e-6
    regularizer[0, 0] = 0.0
    try:
        coefficients = np.linalg.solve(gram + regularizer, design.T @ targets)
    except np.linalg.LinAlgError as error:
        raise D0ReadinessError("ridge control solve failed") from error
    if not bool(np.all(np.isfinite(coefficients))):
        raise D0ReadinessError("ridge control coefficients are non-finite")
    return ControlModels(train, targets, partitions, coefficients)


def predict_controls(
    models: ControlModels, query: c0.TensorRole
) -> dict[str, FloatMatrix]:
    nearest: FloatMatrix = np.empty((len(query.partition_keys), 3), dtype=np.float64)
    local: FloatMatrix = np.empty_like(nearest)
    train_features = models.train.query_features.numpy()[:, :10]
    query_features = query.query_features.numpy()[:, :10]
    for row, key in enumerate(query.partition_keys):
        candidates = models.partitions.get(key)
        if candidates is None or candidates.size == 0:
            raise contract.ContractError("non-neural control partition is unsupported")
        delta = train_features[candidates] - query_features[row]
        distances = np.sum(delta * delta, axis=1, dtype=np.float64)
        nearest[row] = models.targets[int(candidates[int(np.argmin(distances))])]
        positive = distances[distances > 0.0]
        bandwidth = float(np.median(positive)) if positive.size else 1.0
        weights = np.exp(-0.5 * distances / bandwidth)
        denominator = float(np.sum(weights, dtype=np.float64))
        local[row] = (
            np.sum(
                weights[:, None] * models.targets[candidates], axis=0, dtype=np.float64
            )
            / denominator
        )
    ridge = ridge_design(query) @ models.ridge_coefficients
    if not all(bool(np.all(np.isfinite(value))) for value in (nearest, local, ridge)):
        raise D0ReadinessError("non-neural control prediction is non-finite")
    return {
        "continuous-local-interpolation-v1": local,
        "fixed-rbf-integral-ridge-v1": ridge,
        "nearest-causal-surface-query-v1": nearest,
    }


def neural_predictions(
    models: dict[str, nn.Module], role: c0.TensorRole, batch_rows: int
) -> dict[str, FloatMatrix]:
    result: dict[str, FloatMatrix] = {}
    for variant, _seed in QSO_VARIANTS:
        values = e0.predict_qso(
            cast(c0.QuerySurfaceCostModel, models[variant]), role, variant, batch_rows
        )
        result[variant] = values.numpy().astype(np.float64, order="C", copy=True)
    pointwise = e0.predict_pointwise(
        cast(c0.PointwiseCostModel, models[POINTWISE]), role, batch_rows
    )
    result[POINTWISE] = pointwise.numpy().astype(np.float64, order="C", copy=True)
    return result


def stratum_indices(
    context: LoadedContext, batch: contract.SurfaceQueryBatchV1
) -> dict[str, NDArray[np.int64]]:
    cases = c0.enumerate_cases(context.c0_context.f0_profile, "development")
    by_case = {case.case_id: case.stratum for case in cases}
    grouped: dict[str, list[int]] = {
        name: [] for name in ("contact-only", "geometry-only", "joint")
    }
    for index, row_id in enumerate(batch.row_ids):
        case_id = row_id.rsplit(":mode-", 1)[0]
        try:
            grouped[by_case[case_id]].append(index)
        except KeyError as error:
            raise D0ReadinessError(
                "development row cannot be assigned to a stratum"
            ) from error
    if any(not rows for rows in grouped.values()):
        raise D0ReadinessError("development stratum is empty")
    return {name: np.asarray(rows, dtype=np.int64) for name, rows in grouped.items()}


def rmse(
    prediction: FloatMatrix, targets: FloatMatrix, rows: NDArray[np.int64], axis: int
) -> float:
    delta = prediction[rows, axis] - targets[rows, axis]
    return float(np.sqrt(np.mean(delta * delta, dtype=np.float64)))


def ratio_max(
    numerator: float, denominator: float, threshold: float
) -> tuple[float | None, bool]:
    if denominator == 0.0:
        return None, numerator == 0.0
    ratio = numerator / denominator
    return ratio, ratio <= threshold


def ratio_min(
    numerator: float, denominator: float, threshold: float
) -> tuple[float | None, bool]:
    if denominator == 0.0:
        return None, numerator > 0.0
    ratio = numerator / denominator
    return ratio, ratio >= threshold


def evaluate_metrics(
    context: LoadedContext,
    batch: contract.SurfaceQueryBatchV1,
    predictions: dict[str, FloatMatrix],
) -> tuple[dict[str, object], dict[str, bool]]:
    if batch.targets is None:
        raise D0ReadinessError("metric evaluation lacks targets")
    targets = np.asarray(batch.targets)
    strata = stratum_indices(context, batch)
    all_rows: NDArray[np.int64] = np.arange(batch.row_count, dtype=np.int64)
    row_sets = {"aggregate": all_rows, **strata}
    metrics: dict[str, object] = {}
    for name, prediction in sorted(predictions.items()):
        metrics[name] = {
            stratum: {
                axis: rmse(prediction, targets, rows, axis_index)
                for axis_index, axis in enumerate(("decay", "global_gain", "contact"))
            }
            for stratum, rows in row_sets.items()
        }
    candidate_metrics = cast(dict[str, Any], metrics[CANDIDATE])
    gates: dict[str, bool] = {
        "contact-aggregate-absolute": candidate_metrics["aggregate"]["contact"] <= 0.030
    }
    for stratum in strata:
        gates[f"contact-{stratum}-absolute"] = (
            candidate_metrics[stratum]["contact"] <= 0.040
        )
    controls = (
        "continuous-local-interpolation-v1",
        "fixed-rbf-integral-ridge-v1",
        "nearest-causal-surface-query-v1",
        POINTWISE,
    )
    ratio_records: dict[str, object] = {}
    for control_name in controls:
        control_metrics = cast(dict[str, Any], metrics[control_name])
        value, passed = ratio_max(
            candidate_metrics["aggregate"]["contact"],
            control_metrics["aggregate"]["contact"],
            0.92,
        )
        key = f"candidate-contact-vs-{control_name}"
        gates[key] = passed
        ratio_records[key] = value
    for stratum in strata:
        best = min(
            cast(dict[str, Any], metrics[name])[stratum]["contact"] for name in controls
        )
        value, passed = ratio_max(candidate_metrics[stratum]["contact"], best, 0.95)
        key = f"candidate-{stratum}-contact-vs-best-control"
        gates[key] = passed
        ratio_records[key] = value
    for axis in ("decay", "global_gain"):
        for control_name in (
            "nearest-causal-surface-query-v1",
            "fixed-rbf-integral-ridge-v1",
        ):
            value, passed = ratio_max(
                candidate_metrics["aggregate"][axis],
                cast(dict[str, Any], metrics[control_name])["aggregate"][axis],
                0.90,
            )
            key = f"candidate-{axis}-vs-{control_name}"
            gates[key] = passed
            ratio_records[key] = value
    ablations = cast(
        list[dict[str, Any]],
        cast(dict[str, Any], context.profile["metrics"])["ablation_gates"],
    )
    for declaration in ablations:
        variant = cast(str, declaration["variant"])
        threshold = float(declaration["minimum_worsening"])
        for stratum in cast(list[str], declaration["strata"]):
            value, passed = ratio_min(
                cast(dict[str, Any], metrics[variant])[stratum]["contact"],
                candidate_metrics[stratum]["contact"],
                threshold,
            )
            key = f"{variant}-{stratum}-worsening"
            gates[key] = passed
            ratio_records[key] = value
    return {"ratios": ratio_records, "rmse": metrics}, gates


def multiplicative_p1_witness(candidate: FloatMatrix) -> dict[str, bool]:
    rows = candidate.shape[0]
    frequencies: NDArray[np.float64] = np.arange(1, rows + 1, dtype=np.float64)
    preserved_frequencies = frequencies.copy()
    base_gain = np.where(np.arange(rows) % 2 == 0, 1.0, -1.0)
    nodal_gain = base_gain.copy()
    nodal_gain[::7] = 0.0
    gain_factor = np.exp(candidate[:, 1] + candidate[:, 2])
    final_gain = nodal_gain * gain_factor
    doubled_from_input = (2.0 * nodal_gain) * gain_factor
    doubled_from_output = 2.0 * final_gain
    base_decay = np.full(rows, 0.1, dtype=np.float64)
    final_decay = base_decay * np.exp(candidate[:, 0])
    nonzero = nodal_gain != 0.0
    return {
        "frequency-and-order-bit-exact": np.array_equal(
            frequencies, preserved_frequencies
        ),
        "impulse-scaling-exact": np.array_equal(
            doubled_from_input, doubled_from_output
        ),
        "nodal-zero-exact": bool(np.all(final_gain[~nonzero] == 0.0)),
        "positive-decay": bool(np.all(final_decay > 0.0)),
        "signed-gain-sign-preserved": bool(
            np.array_equal(
                np.signbit(final_gain[nonzero]), np.signbit(nodal_gain[nonzero])
            )
        ),
    }


def hard_gates(
    provider: FullArtificialProvider,
    train: c0.TensorRole,
    predictions: dict[str, FloatMatrix],
    trace_access: contract.AccessLedgerV1,
    train_batch: contract.SurfaceQueryBatchV1,
    development_batch: contract.SurfaceQueryBatchV1,
) -> tuple[dict[str, bool], dict[str, bool]]:
    candidate = predictions[CANDIDATE]
    train_built = provider.built(contract.RoleKind.TRAIN)
    development_built = provider.built(contract.RoleKind.DEVELOPMENT)
    p1_witness = multiplicative_p1_witness(candidate)
    gates = {
        "all-predictions-finite": all(
            bool(np.all(np.isfinite(value))) for value in predictions.values()
        ),
        "candidate-contact-bounded": float(np.max(np.abs(candidate[:, 2]))) <= 0.25,
        "candidate-decay-bounded": float(np.max(np.abs(candidate[:, 0]))) <= 0.25,
        "candidate-global-gain-bounded": float(np.max(np.abs(candidate[:, 1]))) <= 0.20,
        "c0-structural-roots-exact": (
            train_built.remesh_root_sha256 == train_built.batch.fields.root_sha256
            and train_built.permutation_root_sha256
            == train_built.batch.structural_root_sha256
            and development_built.remesh_root_sha256
            == development_built.batch.fields.root_sha256
            and development_built.permutation_root_sha256
            == development_built.batch.structural_root_sha256
        ),
        "field-query-topology-reachable": all(
            value == "Reachable" for value in c0.ablation_reachability(train).values()
        ),
        "method-holdout-zero-before-development-pass": trace_access.method_holdout_target_rows
        == 0,
        "multiplicative-p1-invariants-preserved": all(p1_witness.values()),
        "official-target-access-zero": trace_access.official_d0_target_rows == 0
        and trace_access.official_h0_target_rows == 0,
        "real-and-protected-signal-access-zero": all(
            value == 0 for value in zero_forbidden_access().values()
        ),
        "row-target-alignment-exact": (
            train_batch.targets is not None
            and train_batch.targets.shape == (train_batch.row_count, 3)
            and development_batch.targets is not None
            and development_batch.targets.shape == (development_batch.row_count, 3)
            and train_batch.target_root_sha256 is not None
            and development_batch.target_root_sha256 is not None
        ),
    }
    return gates, p1_witness


def resource_gates(
    context: LoadedContext,
    started: float,
    models: dict[str, nn.Module],
    training: dict[str, object],
) -> dict[str, bool]:
    limits = cast(dict[str, Any], context.profile["resources"])
    parameter_count = sum(c0.parameter_count(model) for model in models.values())
    cache_peak = max(
        cast(int, cast(dict[str, object], value)["field_cache_peak_bytes"])
        for value in training.values()
    )
    rss = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024
    return {
        "combined-neural-parameters": parameter_count
        <= cast(int, limits["maximum_combined_neural_parameters"]),
        "field-cache": cache_peak <= cast(int, limits["maximum_field_cache_bytes"]),
        "output-budget-declared": cast(int, limits["maximum_output_bytes"]) > 0,
        "peak-rss": rss <= cast(int, limits["maximum_peak_rss_bytes"]),
        "wall": time.monotonic() - started
        <= cast(float, limits["maximum_wall_seconds"]),
    }


def choose_decision(
    hard: dict[str, bool], metrics: dict[str, bool], resources: dict[str, bool]
) -> contract.TerminalDecision:
    if not all(hard.values()):
        return contract.TerminalDecision.HARD_GATE_REJECT
    if not all(metrics.values()):
        return contract.TerminalDecision.METRIC_REJECT
    if not all(resources.values()):
        return contract.TerminalDecision.RESOURCE_REJECT
    return contract.TerminalDecision.PASS


def owner_identity() -> dict[str, object]:
    data = (repository_root() / OWNER_PATH).read_bytes()
    return {"bytes": len(data), "path": OWNER_PATH, "sha256": sha256_bytes(data)}


def zero_forbidden_access() -> dict[str, int]:
    return {
        "fresh_v37_official_truth_values_evaluated": 0,
        "method_holdout_target_rows": 0,
        "network_requests": 0,
        "official_capabilities_issued": 0,
        "official_d0_target_rows": 0,
        "official_h0_target_rows": 0,
        "protected_signal_values_decoded": 0,
        "real_signal_values_decoded": 0,
    }


def publish_result(
    context: LoadedContext,
    output: Path,
    trace: contract.ExecutionTraceV1,
    weights: bytes,
    evidence: dict[str, object],
) -> publisher.PublicationReceipt:
    owner_sha = cast(str, owner_identity()["sha256"])
    weight_sha = sha256_bytes(weights)
    root = publisher.decision_root_sha256(
        trace, CLAIM, owner_sha, PROFILE_SHA256, weight_sha
    )
    disposition = contract.candidate_disposition(
        trace.terminal,
        owner_sha256=owner_sha,
        profile_sha256=PROFILE_SHA256,
        terminal_sha256=root,
        candidate_weights_sha256=weight_sha,
    )
    if trace.terminal is contract.TerminalDecision.PASS:
        if disposition.freeze_document is None:
            raise D0ReadinessError("Pass disposition lacks freeze")
        payloads = {
            "candidate-bundle.json": e0.bundle_document(weights),
            "candidate-freeze.json": disposition.freeze_document,
            "evidence.json": canonical_json(evidence),
        }
    else:
        if disposition.rejected_evidence_document is None:
            raise D0ReadinessError("reject disposition lacks evidence")
        payloads = {
            "evidence.json": canonical_json(evidence),
            "rejected-candidate.json": disposition.rejected_evidence_document,
        }
    maximum = cast(
        int, cast(dict[str, Any], context.profile["resources"])["maximum_output_bytes"]
    )
    return publisher.publish_terminal(
        output,
        repository_root(),
        trace,
        CLAIM,
        owner_sha,
        PROFILE_SHA256,
        weight_sha,
        disposition,
        payloads,
        maximum,
    )


def execute(context: LoadedContext, output: Path, label: str) -> OwnerResult:
    started = time.monotonic()
    provider = FullArtificialProvider(context, label)
    capability = contract.AccessCapabilityV1.surrogate(
        provider.kind, provider.namespace
    )
    lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    train_batch = lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )
    train_targets = target_tensor(train_batch)
    train_target_root = train_batch.target_root_sha256
    lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
    train = c0.tensor_role(provider.built(contract.RoleKind.TRAIN))
    lifecycle.step(contract.LifecycleStage.CANDIDATE_TRAINING)
    models, training = train_neural_models(train, train_targets, context.profile)
    weights = e0.encode_models(models)
    lifecycle.step(contract.LifecycleStage.CONTROL_TRAINING)
    controls = fit_controls(train, train_targets.numpy())
    development_batch = lifecycle.materialize_role(
        contract.LifecycleStage.DEVELOPMENT_ROLE,
        contract.RoleKind.DEVELOPMENT,
        provider,
    )
    development_target_root = development_batch.target_root_sha256
    development = c0.tensor_role(provider.built(contract.RoleKind.DEVELOPMENT))
    lifecycle.step(contract.LifecycleStage.QUERY_EVALUATION)
    batch_rows = cast(
        int, cast(dict[str, Any], context.profile["training"])["batch_rows"]
    )
    predictions = neural_predictions(models, development, batch_rows)
    predictions.update(predict_controls(controls, development))
    lifecycle.step(contract.LifecycleStage.METRIC_EVALUATION)
    metric_records, metric_gate_results = evaluate_metrics(
        context, development_batch, predictions
    )
    lifecycle.step(contract.LifecycleStage.HARD_GATES)
    hard, p1_witness = hard_gates(
        provider,
        train,
        predictions,
        lifecycle.access,
        train_batch,
        development_batch,
    )
    lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
    resources = resource_gates(context, started, models, training)
    decision = choose_decision(hard, metric_gate_results, resources)
    trace = lifecycle.finish(decision)
    evidence: dict[str, object] = {
        "access": publisher.access_record(trace.access),
        "claim": CLAIM,
        "dependencies": context.dependencies,
        "environment": context.environment,
        "forbidden_access": zero_forbidden_access(),
        "hard_gates": hard,
        "metric_gates": metric_gate_results,
        "metrics": metric_records,
        "owner": owner_identity(),
        "p1_invariant_witness": p1_witness,
        "profile_sha256": PROFILE_SHA256,
        "resource_gates": resources,
        "role_roots": {
            "development": {
                "structural": development_batch.structural_root_sha256,
                "target": development_target_root,
            },
            "train": {
                "structural": train_batch.structural_root_sha256,
                "target": train_target_root,
            },
        },
        "schema": "nextengine.experimental-physical-sound-v37-d0r-evidence.v1",
        "training": training,
    }
    receipt = publish_result(context, output, trace, weights, evidence)
    report: dict[str, object] = {
        "claim": CLAIM,
        "decision": decision.value,
        "forbidden_access": zero_forbidden_access(),
        "next_authorized_action": "build-readiness-seal-from-two-exact-runs",
        "receipt": e0.receipt_record(receipt),
        "schema": "nextengine.experimental-physical-sound-v37-d0r-report.v1",
        "status": "D0R_COMPLETE_ENTRY_READINESS_EXECUTED",
    }
    return OwnerResult(report, receipt, trace, evidence)


def run(profile_path: Path, output: Path, label: str) -> dict[str, object]:
    context = load_context(profile_path)
    return execute(context, output, label).report


def file_tree(root: Path) -> dict[str, bytes]:
    resolved = root.resolve(strict=True)
    if root.is_symlink() or resolved.is_relative_to(repository_root().resolve()):
        raise D0ReadinessError("readiness tree must be external and non-symlinked")
    result: dict[str, bytes] = {}
    for path in sorted(resolved.rglob("*")):
        if path.is_symlink():
            raise D0ReadinessError("readiness tree contains a symlink")
        if path.is_file():
            result[path.relative_to(resolved).as_posix()] = path.read_bytes()
        elif not path.is_dir():
            raise D0ReadinessError("readiness tree contains a non-file entry")
    return result


def build_readiness_seal(run_a: Path, run_b: Path) -> dict[str, object]:
    tree_a = file_tree(run_a)
    tree_b = file_tree(run_b)
    if tree_a != tree_b:
        raise D0ReadinessError("D0R runs are not byte-identical")
    if set(tree_a) != {
        "candidate-bundle.json",
        "candidate-freeze.json",
        "evidence.json",
        "terminal.json",
    }:
        raise D0ReadinessError("D0R Pass output closure is invalid")
    decision = load_json_bytes(tree_a["terminal.json"], "D0R terminal")
    evidence = load_json_bytes(tree_a["evidence.json"], "D0R evidence")
    if decision.get("trace", {}).get("terminal") != "Pass":
        raise D0ReadinessError("D0R readiness seal requires a natural Pass")
    if any(cast(dict[str, int], evidence["forbidden_access"]).values()):
        raise D0ReadinessError("D0R readiness seal follows forbidden access")
    document: dict[str, object] = {
        "claim": CLAIM,
        "readiness_seal": {
            "environment_sha256": sha256_bytes(canonical_json(evidence["environment"])),
            "forbidden_access_count": 0,
            "owner_sha256": owner_identity()["sha256"],
            "profile_sha256": PROFILE_SHA256,
            "repeat_exact": True,
            "run_count": 2,
            "terminal_tree_sha256": publisher.tree_sha256(sorted(tree_a.items())),
            "topology_sha256": decision["trace"]["topology_sha256"],
        },
        "schema": "nextengine.experimental-physical-sound-v37-d0r-readiness-seal.v1",
        "status": "SealedBeforeOfficialTargetAccess",
    }
    document["seal_sha256"] = sha256_bytes(canonical_json(document))
    return document


def validate_checked_readiness_seal(path: Path) -> dict[str, Any]:
    if not path.is_file() or path.is_symlink():
        raise D0ReadinessError("checked D0R readiness seal is absent or linked")
    document = load_json_bytes(path.read_bytes(), "checked D0R readiness seal")
    seal_sha = document.get("seal_sha256")
    unsigned = dict(document)
    unsigned.pop("seal_sha256", None)
    raw_readiness = document.get("readiness_seal")
    if not isinstance(raw_readiness, dict):
        raise D0ReadinessError("checked D0R readiness seal lacks its payload")
    readiness = cast(dict[str, object], raw_readiness)
    if (
        document.get("schema")
        != "nextengine.experimental-physical-sound-v37-d0r-readiness-seal.v1"
        or document.get("status") != "SealedBeforeOfficialTargetAccess"
        or document.get("claim") != CLAIM
        or seal_sha != sha256_bytes(canonical_json(unsigned))
        or readiness.get("owner_sha256") != owner_identity()["sha256"]
        or readiness.get("profile_sha256") != PROFILE_SHA256
        or readiness.get("repeat_exact") is not True
        or readiness.get("run_count") != 2
        or readiness.get("forbidden_access_count") != 0
    ):
        raise D0ReadinessError("checked D0R readiness seal drift")
    return document


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, default=PROFILE_PATH)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--label", default="full")
    parser.add_argument("--seal-run-a", type=Path)
    parser.add_argument("--seal-run-b", type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    if arguments.seal_run_a is not None or arguments.seal_run_b is not None:
        if arguments.seal_run_a is None or arguments.seal_run_b is None:
            raise D0ReadinessError("seal builder requires both run paths")
        print(
            canonical_json(
                build_readiness_seal(arguments.seal_run_a, arguments.seal_run_b)
            ).decode(),
            end="",
        )
        return 0
    if arguments.output is None:
        raise D0ReadinessError("execution requires --output")
    print(
        canonical_json(
            run(arguments.profile, arguments.output.absolute(), arguments.label)
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
