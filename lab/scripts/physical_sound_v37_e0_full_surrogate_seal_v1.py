#!/usr/bin/env python3
"""Run the full discarded V37 QSO owner twice and build its execution seal."""

from __future__ import annotations

import argparse
import base64
import gc
import hashlib
import json
import platform
import resource
import struct
import tempfile
import time
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, cast

import numpy as np
import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0
import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_terminal_publisher_v1 as publisher
import torch
from numpy.typing import NDArray
from torch import nn

PROFILE_PATH = "lab/profiles/physical-sound-v37-e0-full-surrogate-seal.v1.json"
C0_PROFILE_PATH = (
    "lab/profiles/physical-sound-v37-c0-query-surface-structural-cost.v1.json"
)
OWNER_PATH = "lab/scripts/physical_sound_v37_e0_full_surrogate_seal_v1.py"
PUBLISHER_PATH = "lab/scripts/physical_sound_v37_terminal_publisher_v1.py"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-e0-full-surrogate-seal-profile.v1"
)
PROFILE_ID = "physical-sound-v37-e0-full-surrogate-seal-v1"
PROFILE_SHA256 = "f95ccc622430a6dd27661d34526c5970a722dbd40a717230d24200c62dd62f2a"
CLAIM = (
    "FULL_COUNT_DISCARDED_ZERO_TARGET_QSO_OWNER_REHEARSAL_AND_EXECUTION_SEAL_ONLY / "
    "NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_"
    "COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
ROLE_NAMES = ("train", "development", "method-holdout")
QSO_VARIANTS = (
    ("query-conditioned-surface-operator-v0", 370201),
    ("qso-field-only-ablation-v1", 370204),
    ("qso-no-interaction-ablation-v1", 370206),
    ("qso-no-topology-ablation-v1", 370205),
    ("qso-query-only-ablation-v1", 370203),
)
POINTWISE_NAME = "v36-shaped-pointwise-mlp-v1"
POINTWISE_SEED = 370202
FloatMatrix = NDArray[np.float64]


class E0Error(RuntimeError):
    """The full-count rehearsal or execution seal is invalid."""


@dataclass(frozen=True, slots=True)
class LoadedContext:
    profile: dict[str, Any]
    profile_data: bytes
    dependencies: dict[str, dict[str, object]]
    c0_context: c0.LoadedContext
    environment: dict[str, object]


@dataclass(frozen=True, slots=True)
class CandidateBundle:
    data: bytes
    weights_sha256: str
    models: dict[str, nn.Module]
    freeze_document: bytes | None = None
    decision_root_sha256: str | None = None


@dataclass(frozen=True, slots=True)
class FullOwnerRun:
    bundle: CandidateBundle
    report: dict[str, object]
    trace: contract.ExecutionTraceV1
    receipt: publisher.PublicationReceipt
    evidence: dict[str, object]


class FullSurrogateProvider:
    def __init__(
        self, context: LoadedContext, pipeline: contract.PipelineKind, label: str
    ) -> None:
        self.context = context
        self._kind = (
            contract.ProviderKind.SURROGATE_D0
            if pipeline is contract.PipelineKind.D0
            else contract.ProviderKind.SURROGATE_H0
        )
        self._namespace = f"discarded-v37-e0-{label}-{pipeline.value}"

    @property
    def kind(self) -> contract.ProviderKind:
        return self._kind

    @property
    def namespace(self) -> str:
        return self._namespace

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapabilityV1
    ) -> contract.SurfaceQueryBatchV1:
        if capability.provider_kind is not self.kind:
            raise contract.ContractError("E0 provider kind mismatch")
        if capability.namespace != self.namespace:
            raise contract.ContractError("E0 provider namespace mismatch")
        built = c0.build_role(self.context.c0_context, role.value.replace("-", "_"))
        expected = cast(dict[str, int], self.context.profile["workload"]["role_rows"])
        if built.batch.row_count != expected[role.value]:
            raise E0Error(f"full role row-count drift: {role.value}")
        zeros = tuple((0.0, 0.0, 0.0) for _ in range(built.batch.row_count))
        return replace(built.batch, targets=contract.frozen_float_matrix(zeros))


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise E0Error(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise E0Error(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise E0Error(f"JSON root must be an object: {label}")
    result = cast(dict[str, Any], value)
    if canonical_json(result) != data:
        raise E0Error(f"JSON is not canonical: {label}")
    return result


def bound_file(path_text: str, expected_sha256: str) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise E0Error(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise E0Error(f"bound file drift: {path_text}")
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
        raise E0Error("execution environment identity drift")
    return actual


def load_context(profile_path: Path) -> LoadedContext:
    if not profile_path.is_file() or profile_path.is_symlink():
        raise E0Error("E0 profile must be a regular file")
    data = profile_path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise E0Error("E0 profile size outside bound")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise E0Error("E0 profile hash mismatch")
    profile = load_json_bytes(data, "E0 profile")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
    ):
        raise E0Error("E0 profile identity mismatch")
    authority = cast(dict[str, object], profile["authority"])
    if (
        authority.get("official_capability_allowed_during_rehearsal") is not False
        or authority.get("surrogate_zero_targets_only") is not True
        or authority.get("execution_seal_allowed_after_two_exact_runs") is not True
    ):
        raise E0Error("E0 authority drift")
    dependencies: dict[str, dict[str, object]] = {}
    paths: list[str] = []
    for declaration in cast(list[dict[str, str]], profile["dependencies"]):
        path_text = declaration["path"]
        dependencies[path_text] = bound_file(path_text, declaration["sha256"])
        paths.append(path_text)
    if paths != sorted(set(paths)):
        raise E0Error("E0 dependencies must be canonical and unique")
    c0_context = c0.load_context(repository_root() / C0_PROFILE_PATH)
    workload = cast(dict[str, Any], profile["workload"])
    c0_cost = cast(dict[str, Any], c0_context.profile["cost_oracle"])
    if (
        workload["batch_rows"] != c0_cost["batch_rows"]
        or workload["gradient_microbatch_rows"] != c0_cost["gradient_microbatch_rows"]
        or workload["neural_steps_each"] != c0_cost["neural_cost_steps"]
        or workload["candidate_and_control_order"]
        != c0_cost["candidate_and_control_order"]
        or workload["combined_neural_parameter_count"]
        != c0_cost["expected_combined_neural_parameters"]
    ):
        raise E0Error("E0 workload differs from frozen C0 cost path")
    configure_torch()
    environment = environment_record(profile)
    return LoadedContext(profile, data, dependencies, c0_context, environment)


def role_name(role: contract.RoleKind) -> str:
    return cast(str, role.value).replace("-", "_")


def tensorize(batch: contract.SurfaceQueryBatchV1) -> c0.TensorRole:
    built = c0.BuiltRole(
        role=role_name(batch.role),
        batch=replace(batch, targets=None),
        case_ids=("discarded-e0-receipt",),
        remesh_root_sha256=batch.fields.root_sha256,
        permutation_root_sha256=batch.structural_root_sha256,
        query_support_count=batch.row_count,
        exact_two_hop_pairs=1,
    )
    return c0.tensor_role(built)


def require_zero_targets(batch: contract.SurfaceQueryBatchV1) -> None:
    if batch.targets is None or not bool(np.all(batch.targets == 0.0)):
        raise E0Error("E0 provider returned nonzero or absent surrogate targets")


def train_qso(
    variant: str,
    seed: int,
    train: c0.TensorRole,
    steps: int,
    batch_rows: int,
    microbatch_rows: int,
) -> tuple[c0.QuerySurfaceCostModel, dict[str, object]]:
    model = c0.QuerySurfaceCostModel(seed)
    if c0.parameter_count(model) != 12_443:
        raise E0Error(f"QSO parameter count drift: {variant}")
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.002, weight_decay=0.0)
    cache_peak = 0
    topology = variant != "qso-no-topology-ablation-v1"
    row_count = train.query_features.shape[0]
    for step in range(steps):
        rows = (
            torch.arange(batch_rows, dtype=torch.int64) + step * batch_rows
        ) % row_count
        optimizer.zero_grad(set_to_none=True)
        for start in range(0, batch_rows, microbatch_rows):
            micro_rows = rows[start : start + microbatch_rows]
            fields, local_rows = torch.unique(
                train.row_field_indices[micro_rows], sorted=True, return_inverse=True
            )
            encoded = model.encode_fields(
                train, topology=topology, field_indices=fields
            )
            cache_peak = max(cache_peak, encoded.numel() * encoded.element_size())
            predictions = model.predict_from_encoded(
                encoded,
                train,
                micro_rows,
                variant,
                local_row_field_indices=local_rows,
            )
            loss = torch.sum(predictions * predictions) / (
                batch_rows * contract.TARGET_AXIS_COUNT
            )
            if not bool(torch.isfinite(loss)):
                raise E0Error(f"non-finite QSO loss: {variant}")
            loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
    return model, {
        "field_cache_peak_bytes": cache_peak,
        "parameter_count": c0.parameter_count(model),
        "steps": steps,
        "weight_sha256": state_sha256(model),
    }


def train_pointwise(
    train: c0.TensorRole, steps: int, batch_rows: int, microbatch_rows: int
) -> tuple[c0.PointwiseCostModel, dict[str, object]]:
    model = c0.PointwiseCostModel(POINTWISE_SEED)
    if c0.parameter_count(model) != 1_859:
        raise E0Error("pointwise parameter count drift")
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.002, weight_decay=0.0)
    row_count = train.query_features.shape[0]
    for step in range(steps):
        rows = (
            torch.arange(batch_rows, dtype=torch.int64) + step * batch_rows
        ) % row_count
        optimizer.zero_grad(set_to_none=True)
        for start in range(0, batch_rows, microbatch_rows):
            micro_rows = rows[start : start + microbatch_rows]
            predictions = model(train, micro_rows)
            loss = torch.sum(predictions * predictions) / (
                batch_rows * contract.TARGET_AXIS_COUNT
            )
            if not bool(torch.isfinite(loss)):
                raise E0Error("non-finite pointwise loss")
            loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
    return model, {
        "field_cache_peak_bytes": 0,
        "parameter_count": c0.parameter_count(model),
        "steps": steps,
        "weight_sha256": state_sha256(model),
    }


def predict_qso(
    model: c0.QuerySurfaceCostModel,
    role: c0.TensorRole,
    variant: str,
    batch_rows: int,
) -> torch.Tensor:
    topology = variant != "qso-no-topology-ablation-v1"
    chunks: list[torch.Tensor] = []
    with torch.no_grad():
        encoded = model.encode_fields(role, topology=topology)
        for start in range(0, role.query_features.shape[0], batch_rows):
            rows = torch.arange(
                start,
                min(start + batch_rows, role.query_features.shape[0]),
                dtype=torch.int64,
            )
            chunks.append(model.predict_from_encoded(encoded, role, rows, variant))
    return torch.cat(chunks, dim=0)


def predict_pointwise(
    model: c0.PointwiseCostModel, role: c0.TensorRole, batch_rows: int
) -> torch.Tensor:
    chunks: list[torch.Tensor] = []
    with torch.no_grad():
        for start in range(0, role.query_features.shape[0], batch_rows):
            rows = torch.arange(
                start,
                min(start + batch_rows, role.query_features.shape[0]),
                dtype=torch.int64,
            )
            chunks.append(model(role, rows))
    return torch.cat(chunks, dim=0)


def train_neural_models(
    train: c0.TensorRole, workload: dict[str, Any]
) -> tuple[dict[str, nn.Module], dict[str, object]]:
    steps = cast(int, workload["neural_steps_each"])
    batch_rows = cast(int, workload["batch_rows"])
    microbatch_rows = cast(int, workload["gradient_microbatch_rows"])
    models: dict[str, nn.Module] = {}
    records: dict[str, object] = {}
    for variant, seed in QSO_VARIANTS:
        model, record = train_qso(
            variant, seed, train, steps, batch_rows, microbatch_rows
        )
        models[variant] = model
        records[variant] = record
        gc.collect()
    pointwise, record = train_pointwise(train, steps, batch_rows, microbatch_rows)
    models[POINTWISE_NAME] = pointwise
    records[POINTWISE_NAME] = record
    if sum(c0.parameter_count(model) for model in models.values()) != cast(
        int, workload["combined_neural_parameter_count"]
    ):
        raise E0Error("combined neural parameter count drift")
    return models, records


def tensor_bytes(value: torch.Tensor) -> bytes:
    array = value.detach().cpu().numpy().astype("<f8", copy=False)
    return cast(bytes, array.tobytes(order="C"))


def encode_models(models: dict[str, nn.Module]) -> bytes:
    output = bytearray(b"NXV37E0QSO\x00\x01")
    for model_name, model in sorted(models.items()):
        name_data = model_name.encode()
        output.extend(struct.pack("<I", len(name_data)))
        output.extend(name_data)
        state = model.state_dict()
        output.extend(struct.pack("<I", len(state)))
        for tensor_name, tensor in sorted(state.items()):
            tensor_name_data = tensor_name.encode()
            output.extend(struct.pack("<I", len(tensor_name_data)))
            output.extend(tensor_name_data)
            output.extend(struct.pack("<I", tensor.ndim))
            output.extend(struct.pack(f"<{tensor.ndim}Q", *tensor.shape))
            data = tensor_bytes(tensor)
            output.extend(struct.pack("<Q", len(data)))
            output.extend(data)
    return bytes(output)


def model_factories() -> dict[str, nn.Module]:
    models: dict[str, nn.Module] = {
        variant: c0.QuerySurfaceCostModel(seed) for variant, seed in QSO_VARIANTS
    }
    models[POINTWISE_NAME] = c0.PointwiseCostModel(POINTWISE_SEED)
    return models


def decode_models(data: bytes) -> dict[str, nn.Module]:
    magic = b"NXV37E0QSO\x00\x01"
    if not data.startswith(magic):
        raise E0Error("candidate bundle magic mismatch")
    cursor = len(magic)
    models = model_factories()
    for model_name, model in sorted(models.items()):
        name_size = struct.unpack_from("<I", data, cursor)[0]
        cursor += 4
        encoded_name = data[cursor : cursor + name_size].decode()
        cursor += name_size
        if encoded_name != model_name:
            raise E0Error("candidate model ordering mismatch")
        tensor_count = struct.unpack_from("<I", data, cursor)[0]
        cursor += 4
        source_state: dict[str, torch.Tensor] = {}
        expected_state = model.state_dict()
        for _ in range(tensor_count):
            tensor_name_size = struct.unpack_from("<I", data, cursor)[0]
            cursor += 4
            tensor_name = data[cursor : cursor + tensor_name_size].decode()
            cursor += tensor_name_size
            ndim = struct.unpack_from("<I", data, cursor)[0]
            cursor += 4
            shape = struct.unpack_from(f"<{ndim}Q", data, cursor)
            cursor += 8 * ndim
            byte_count = struct.unpack_from("<Q", data, cursor)[0]
            cursor += 8
            tensor_data = data[cursor : cursor + byte_count]
            cursor += byte_count
            expected = expected_state.get(tensor_name)
            if expected is None or tuple(expected.shape) != tuple(shape):
                raise E0Error("candidate tensor shape or name mismatch")
            array = np.frombuffer(tensor_data, dtype="<f8").reshape(shape).copy()
            source_state[tensor_name] = torch.from_numpy(array)
        model.load_state_dict(source_state, strict=True)
    if cursor != len(data):
        raise E0Error("candidate bundle has trailing bytes")
    return models


def state_sha256(model: nn.Module) -> str:
    digest = hashlib.sha256()
    for name, tensor in sorted(model.state_dict().items()):
        digest.update(name.encode() + b"\x00")
        digest.update(tensor_bytes(tensor))
    return digest.hexdigest()


def bundle_document(weights: bytes) -> bytes:
    return canonical_json(
        {
            "model_count": 6,
            "schema": "nextengine.experimental-physical-sound-v37-e0-model-bundle.v1",
            "weights_base64": base64.b64encode(weights).decode("ascii"),
            "weights_sha256": sha256_bytes(weights),
        }
    )


def load_bundle_document(data: bytes) -> tuple[bytes, dict[str, nn.Module], str]:
    document = load_json_bytes(data, "E0 candidate bundle")
    encoded = document.get("weights_base64")
    expected = document.get("weights_sha256")
    if not isinstance(encoded, str) or not isinstance(expected, str):
        raise E0Error("candidate bundle identity is absent")
    try:
        weights = base64.b64decode(encoded, validate=True)
    except ValueError as error:
        raise E0Error("candidate bundle base64 is invalid") from error
    if sha256_bytes(weights) != expected:
        raise E0Error("candidate bundle weight hash mismatch")
    return weights, decode_models(weights), expected


def prediction_metrics(
    models: dict[str, nn.Module], role: c0.TensorRole, batch_rows: int
) -> tuple[dict[str, object], dict[str, torch.Tensor]]:
    predictions: dict[str, torch.Tensor] = {}
    for variant, _seed in QSO_VARIANTS:
        predictions[variant] = predict_qso(
            cast(c0.QuerySurfaceCostModel, models[variant]),
            role,
            variant,
            batch_rows,
        )
    predictions[POINTWISE_NAME] = predict_pointwise(
        cast(c0.PointwiseCostModel, models[POINTWISE_NAME]), role, batch_rows
    )
    zeros = torch.zeros(
        (role.query_features.shape[0], contract.TARGET_AXIS_COUNT), dtype=torch.float64
    )
    for name in (
        "continuous-local-interpolation-v1",
        "fixed-rbf-integral-ridge-v1",
        "nearest-causal-surface-query-v1",
    ):
        predictions[name] = zeros
    records: dict[str, object] = {}
    for name, values in predictions.items():
        finite = bool(torch.all(torch.isfinite(values)))
        if not finite:
            raise E0Error(f"non-finite rehearsal prediction: {name}")
        records[name] = {
            "maximum_absolute": float(torch.max(torch.abs(values))),
            "output_sha256": c0.digest_tensors([values]),
            "rmse_to_discarded_zero": float(torch.sqrt(torch.mean(values * values))),
            "row_count": values.shape[0],
        }
    return records, predictions


def exercise_non_neural_paths(train: c0.TensorRole) -> dict[str, object]:
    paths = c0.non_neural_cost({name: train for name in c0.ROLE_NAMES})
    if tuple(sorted(paths)) != (
        "continuous-local-interpolation-v1",
        "fixed-rbf-integral-ridge-v1",
        "nearest-causal-surface-query-v1",
    ):
        raise E0Error("non-neural control closure drift")
    return cast(dict[str, object], paths)


def resource_record(context: LoadedContext, started: float) -> dict[str, object]:
    resources = cast(dict[str, Any], context.profile["resources"])
    wall = time.monotonic() - started
    rss = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024
    gates = {
        "output_budget_declared": int(resources["maximum_output_bytes"]) > 0,
        "peak_rss": rss <= int(resources["maximum_peak_rss_bytes"]),
        "wall": wall <= float(resources["maximum_wall_seconds"]),
    }
    return {
        "gates": gates,
        "maximum_peak_rss_bytes": int(resources["maximum_peak_rss_bytes"]),
        "maximum_wall_seconds": float(resources["maximum_wall_seconds"]),
    }


def owner_identity() -> dict[str, object]:
    data = (repository_root() / OWNER_PATH).read_bytes()
    return {"bytes": len(data), "path": OWNER_PATH, "sha256": sha256_bytes(data)}


def zero_official_access() -> dict[str, int]:
    return {
        "fresh_v37_truth_values_evaluated": 0,
        "network_requests": 0,
        "official_capabilities_issued": 0,
        "official_d0_target_rows": 0,
        "official_h0_target_rows": 0,
        "prior_generation_numeric_values_read": 0,
        "protected_signal_values_decoded": 0,
        "real_signal_values_decoded": 0,
    }


def publish_pass(
    context: LoadedContext,
    output: Path,
    trace: contract.ExecutionTraceV1,
    bundle: CandidateBundle,
    evidence: dict[str, object],
) -> tuple[publisher.PublicationReceipt, bytes]:
    owner_sha256 = cast(str, owner_identity()["sha256"])
    root = publisher.decision_root_sha256(
        trace, CLAIM, owner_sha256, PROFILE_SHA256, bundle.weights_sha256
    )
    disposition = contract.candidate_disposition(
        contract.TerminalDecision.PASS,
        owner_sha256=owner_sha256,
        profile_sha256=PROFILE_SHA256,
        terminal_sha256=root,
        candidate_weights_sha256=bundle.weights_sha256,
    )
    if disposition.freeze_document is None:
        raise E0Error("Pass disposition lacks freeze")
    resources = cast(dict[str, Any], context.profile["resources"])
    receipt = publisher.publish_terminal(
        output,
        repository_root(),
        trace,
        CLAIM,
        owner_sha256,
        PROFILE_SHA256,
        bundle.weights_sha256,
        disposition,
        {
            "candidate-bundle.json": bundle.data,
            "candidate-freeze.json": disposition.freeze_document,
            "evidence.json": canonical_json(evidence),
        },
        cast(int, resources["maximum_output_bytes"]),
    )
    return receipt, disposition.freeze_document


def execute_d0(
    context: LoadedContext, output: Path, label: str, started: float
) -> FullOwnerRun:
    provider = FullSurrogateProvider(context, contract.PipelineKind.D0, label)
    capability = contract.AccessCapabilityV1.surrogate(
        provider.kind, provider.namespace
    )
    lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    train_batch = lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )
    require_zero_targets(train_batch)
    train_roots = {
        "structural": train_batch.structural_root_sha256,
        "target": train_batch.target_root_sha256,
    }
    lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
    train = tensorize(train_batch)
    del train_batch
    gc.collect()
    workload = cast(dict[str, Any], context.profile["workload"])
    lifecycle.step(contract.LifecycleStage.CANDIDATE_TRAINING)
    models, training = train_neural_models(train, workload)
    weights = encode_models(models)
    bundle = CandidateBundle(bundle_document(weights), sha256_bytes(weights), models)
    lifecycle.step(contract.LifecycleStage.CONTROL_TRAINING)
    non_neural = exercise_non_neural_paths(train)
    development_batch = lifecycle.materialize_role(
        contract.LifecycleStage.DEVELOPMENT_ROLE,
        contract.RoleKind.DEVELOPMENT,
        provider,
    )
    require_zero_targets(development_batch)
    development_roots = {
        "structural": development_batch.structural_root_sha256,
        "target": development_batch.target_root_sha256,
    }
    development = tensorize(development_batch)
    del development_batch
    gc.collect()
    lifecycle.step(contract.LifecycleStage.QUERY_EVALUATION)
    metrics, predictions = prediction_metrics(
        models, development, cast(int, workload["batch_rows"])
    )
    lifecycle.step(contract.LifecycleStage.METRIC_EVALUATION)
    candidate = predictions["query-conditioned-surface-operator-v0"]
    lifecycle.step(contract.LifecycleStage.HARD_GATES)
    hard = {
        "all_models_finite": all(
            bool(torch.all(torch.isfinite(values))) for values in predictions.values()
        ),
        "candidate_contact_bounded": float(torch.max(torch.abs(candidate[:, 2])))
        <= 0.25,
        "candidate_decay_bounded": float(torch.max(torch.abs(candidate[:, 0]))) <= 0.25,
        "candidate_global_gain_bounded": float(torch.max(torch.abs(candidate[:, 1])))
        <= 0.20,
        "discarded_targets_exact_zero": True,
        "field_query_topology_reachable": all(
            value == "Reachable" for value in c0.ablation_reachability(train).values()
        ),
    }
    lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
    resources = resource_record(context, started)
    if not all(hard.values()):
        raise E0Error("D0 rehearsal hard gate failed")
    if not all(cast(dict[str, bool], resources["gates"]).values()):
        raise E0Error("D0 rehearsal resource gate failed")
    trace = lifecycle.finish(contract.TerminalDecision.PASS)
    evidence: dict[str, object] = {
        "access": publisher.access_record(trace.access),
        "claim": CLAIM,
        "dependencies": context.dependencies,
        "hard": hard,
        "metrics": metrics,
        "non_neural_training": non_neural,
        "official_access": zero_official_access(),
        "owner": owner_identity(),
        "pipeline": "d0",
        "resource": resources,
        "role_roots": {"development": development_roots, "train": train_roots},
        "schema": "nextengine.experimental-physical-sound-v37-e0-owner-evidence.v1",
        "training": training,
    }
    receipt, freeze = publish_pass(context, output, trace, bundle, evidence)
    frozen = replace(
        bundle,
        freeze_document=freeze,
        decision_root_sha256=receipt.decision_root_sha256,
    )
    report: dict[str, object] = {
        "decision": "Pass",
        "pipeline": "d0",
        "terminal_sha256": receipt.terminal_sha256,
        "tree_sha256": receipt.tree_sha256,
    }
    return FullOwnerRun(frozen, report, trace, receipt, evidence)


def validate_d0_bundle(bundle: CandidateBundle) -> dict[str, object]:
    if bundle.freeze_document is None or bundle.decision_root_sha256 is None:
        raise E0Error("H0 requires a complete D0 Pass bundle")
    disposition = contract.CandidateDispositionV1(
        contract.TerminalDecision.PASS, True, bundle.freeze_document, None
    )
    return cast(
        dict[str, object],
        contract.validate_h0_candidate_freeze(
            disposition,
            expected_owner_sha256=cast(str, owner_identity()["sha256"]),
            expected_profile_sha256=PROFILE_SHA256,
            expected_terminal_sha256=bundle.decision_root_sha256,
            expected_weights_sha256=bundle.weights_sha256,
        ),
    )


def execute_h0(
    context: LoadedContext,
    output: Path,
    label: str,
    started: float,
    d0_bundle: CandidateBundle,
) -> FullOwnerRun:
    provider = FullSurrogateProvider(context, contract.PipelineKind.H0, label)
    capability = contract.AccessCapabilityV1.surrogate(
        provider.kind, provider.namespace
    )
    lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.H0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    train_batch = lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_RECONSTRUCTION,
        contract.RoleKind.TRAIN,
        provider,
    )
    require_zero_targets(train_batch)
    train_roots = {
        "structural": train_batch.structural_root_sha256,
        "target": train_batch.target_root_sha256,
    }
    train = tensorize(train_batch)
    del train_batch
    gc.collect()
    lifecycle.step(contract.LifecycleStage.CANDIDATE_LOAD)
    validate_d0_bundle(d0_bundle)
    weights, models, weights_sha256 = load_bundle_document(d0_bundle.data)
    if weights_sha256 != d0_bundle.weights_sha256 or encode_models(models) != weights:
        raise E0Error("loaded D0 candidate is not byte-exact")
    holdout_batch = lifecycle.materialize_role(
        contract.LifecycleStage.METHOD_HOLDOUT_ROLE,
        contract.RoleKind.METHOD_HOLDOUT,
        provider,
    )
    require_zero_targets(holdout_batch)
    holdout_roots = {
        "structural": holdout_batch.structural_root_sha256,
        "target": holdout_batch.target_root_sha256,
    }
    lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
    holdout = tensorize(holdout_batch)
    del holdout_batch
    gc.collect()
    workload = cast(dict[str, Any], context.profile["workload"])
    lifecycle.step(contract.LifecycleStage.QUERY_EVALUATION)
    metrics, predictions = prediction_metrics(
        models, holdout, cast(int, workload["batch_rows"])
    )
    non_neural = exercise_non_neural_paths(train)
    lifecycle.step(contract.LifecycleStage.METRIC_EVALUATION)
    candidate = predictions["query-conditioned-surface-operator-v0"]
    lifecycle.step(contract.LifecycleStage.HARD_GATES)
    hard = {
        "all_models_finite": all(
            bool(torch.all(torch.isfinite(values))) for values in predictions.values()
        ),
        "candidate_contact_bounded": float(torch.max(torch.abs(candidate[:, 2])))
        <= 0.25,
        "candidate_decay_bounded": float(torch.max(torch.abs(candidate[:, 0]))) <= 0.25,
        "candidate_global_gain_bounded": float(torch.max(torch.abs(candidate[:, 1])))
        <= 0.20,
        "discarded_targets_exact_zero": True,
        "train_reconstruction_recorded": True,
    }
    lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
    resources = resource_record(context, started)
    if not all(hard.values()):
        raise E0Error("H0 rehearsal hard gate failed")
    if not all(cast(dict[str, bool], resources["gates"]).values()):
        raise E0Error("H0 rehearsal resource gate failed")
    trace = lifecycle.finish(contract.TerminalDecision.PASS)
    evidence: dict[str, object] = {
        "access": publisher.access_record(trace.access),
        "candidate_load": validate_d0_bundle(d0_bundle),
        "claim": CLAIM,
        "hard": hard,
        "metrics": metrics,
        "non_neural_reconstruction": non_neural,
        "official_access": zero_official_access(),
        "owner": owner_identity(),
        "pipeline": "h0",
        "resource": resources,
        "role_roots": {"method_holdout": holdout_roots, "train": train_roots},
        "schema": "nextengine.experimental-physical-sound-v37-e0-owner-evidence.v1",
        "training_steps": 0,
    }
    h0_bundle = CandidateBundle(d0_bundle.data, d0_bundle.weights_sha256, models)
    receipt, freeze = publish_pass(context, output, trace, h0_bundle, evidence)
    frozen = replace(
        h0_bundle,
        freeze_document=freeze,
        decision_root_sha256=receipt.decision_root_sha256,
    )
    report: dict[str, object] = {
        "decision": "Pass",
        "pipeline": "h0",
        "terminal_sha256": receipt.terminal_sha256,
        "tree_sha256": receipt.tree_sha256,
    }
    return FullOwnerRun(frozen, report, trace, receipt, evidence)


def receipt_record(receipt: publisher.PublicationReceipt) -> dict[str, object]:
    return {
        "decision_root_sha256": receipt.decision_root_sha256,
        "file_count": receipt.file_count,
        "terminal_sha256": receipt.terminal_sha256,
        "total_bytes": receipt.total_bytes,
        "tree_sha256": receipt.tree_sha256,
    }


def run(profile_path: Path, output: Path) -> dict[str, object]:
    started = time.monotonic()
    context = load_context(profile_path)
    with tempfile.TemporaryDirectory(prefix="nextengine-v37-e0-owner-") as temporary:
        root = Path(temporary)
        d0 = execute_d0(context, root / "d0", "full", started)
        h0 = execute_h0(context, root / "h0", "full", started, d0.bundle)
        d0_train = cast(dict[str, object], d0.evidence["role_roots"])["train"]
        h0_train = cast(dict[str, object], h0.evidence["role_roots"])["train"]
        if canonical_json(d0_train) != canonical_json(h0_train):
            raise E0Error("D0/H0 train reconstruction mismatch")
        rehearsal: dict[str, object] = {
            "claim": CLAIM,
            "d0": {
                "access": publisher.access_record(d0.trace.access),
                "receipt": receipt_record(d0.receipt),
                "topology_sha256": d0.trace.topology_sha256,
            },
            "environment": context.environment,
            "environment_sha256": sha256_bytes(canonical_json(context.environment)),
            "h0": {
                "access": publisher.access_record(h0.trace.access),
                "receipt": receipt_record(h0.receipt),
                "topology_sha256": h0.trace.topology_sha256,
            },
            "official_access": zero_official_access(),
            "owner": owner_identity(),
            "profile_sha256": PROFILE_SHA256,
            "publisher": bound_file(
                PUBLISHER_PATH,
                next(
                    row["sha256"]
                    for row in cast(
                        list[dict[str, str]], context.profile["dependencies"]
                    )
                    if row["path"] == PUBLISHER_PATH
                ),
            ),
            "schema": "nextengine.experimental-physical-sound-v37-e0-rehearsal.v1",
            "train_reconstruction_exact": True,
            "workload": context.profile["workload"],
        }
        top_evidence: dict[str, object] = {
            "dependencies": context.dependencies,
            "official_access": zero_official_access(),
            "rehearsal": rehearsal,
            "schema": "nextengine.experimental-physical-sound-v37-e0-evidence.v1",
        }
        receipt, freeze = publish_pass(
            context, output, d0.trace, d0.bundle, top_evidence
        )
        if freeze != d0.bundle.freeze_document:
            raise E0Error("top-level D0 freeze differs from inner D0 freeze")
        report: dict[str, object] = {
            "claim": CLAIM,
            "d0_artifact_root_sha256": d0.receipt.tree_sha256,
            "decision": "Pass",
            "h0_artifact_root_sha256": h0.receipt.tree_sha256,
            "next_authorized_action": "build-seal-from-two-exact-rehearsals",
            "official_access": zero_official_access(),
            "receipt": receipt_record(receipt),
            "schema": "nextengine.experimental-physical-sound-v37-e0-report.v1",
            "status": "E0_FULL_SURROGATE_REHEARSAL_PASS",
        }
        return report


def file_tree(root: Path) -> dict[str, bytes]:
    resolved = root.resolve(strict=True)
    if root.is_symlink() or resolved.is_relative_to(repository_root().resolve()):
        raise E0Error("rehearsal tree must be external and non-symlinked")
    files: dict[str, bytes] = {}
    for path in sorted(resolved.rglob("*")):
        if path.is_symlink():
            raise E0Error("rehearsal tree contains a symlink")
        if path.is_file():
            files[path.relative_to(resolved).as_posix()] = path.read_bytes()
        elif not path.is_dir():
            raise E0Error("rehearsal tree contains a non-file entry")
    return files


def tree_root(files: dict[str, bytes]) -> str:
    return cast(str, publisher.tree_sha256(sorted(files.items())))


def build_execution_seal(run_a: Path, run_b: Path) -> dict[str, object]:
    tree_a = file_tree(run_a)
    tree_b = file_tree(run_b)
    if tree_a != tree_b:
        raise E0Error("rehearsal runs are not byte-identical")
    required = {
        "candidate-bundle.json",
        "candidate-freeze.json",
        "evidence.json",
        "terminal.json",
    }
    if set(tree_a) != required:
        raise E0Error("rehearsal output closure is invalid")
    evidence = load_json_bytes(tree_a["evidence.json"], "E0 evidence")
    rehearsal = cast(dict[str, Any], evidence["rehearsal"])
    if any(cast(dict[str, int], evidence["official_access"]).values()):
        raise E0Error("execution seal cannot follow forbidden access")
    owner = cast(dict[str, object], rehearsal["owner"])
    publisher_record = cast(dict[str, object], rehearsal["publisher"])
    d0 = cast(dict[str, Any], rehearsal["d0"])
    h0 = cast(dict[str, Any], rehearsal["h0"])
    document: dict[str, object] = {
        "claim": CLAIM,
        "execution_seal": {
            "contract_schema": contract.CONTRACT_SCHEMA,
            "d0_rehearsal_root_sha256": d0["receipt"]["tree_sha256"],
            "d0_topology_sha256": d0["topology_sha256"],
            "environment_sha256": rehearsal["environment_sha256"],
            "forbidden_access_count": 0,
            "h0_rehearsal_root_sha256": h0["receipt"]["tree_sha256"],
            "h0_topology_sha256": h0["topology_sha256"],
            "owner_sha256": owner["sha256"],
            "profile_sha256": rehearsal["profile_sha256"],
            "publisher_sha256": publisher_record["sha256"],
            "rehearsal_run_count": 2,
            "repeat_exact": True,
        },
        "rehearsal_tree_sha256": tree_root(tree_a),
        "schema": "nextengine.experimental-physical-sound-v37-execution-seal.v1",
        "status": "SealedBeforeOfficialAccess",
    }
    document["seal_sha256"] = sha256_bytes(canonical_json(document))
    return document


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default=PROFILE_PATH, type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--seal-run-a", type=Path)
    parser.add_argument("--seal-run-b", type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    if arguments.seal_run_a is not None or arguments.seal_run_b is not None:
        if arguments.seal_run_a is None or arguments.seal_run_b is None:
            raise E0Error("seal builder requires both rehearsal runs")
        print(
            canonical_json(
                build_execution_seal(arguments.seal_run_a, arguments.seal_run_b)
            ).decode(),
            end="",
        )
        return 0
    if arguments.output is None:
        raise E0Error("rehearsal mode requires --output")
    report = run(arguments.profile, arguments.output.absolute())
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
