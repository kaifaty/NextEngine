#!/usr/bin/env python3
"""Exercise the complete V37 owner and atomic terminal on discarded targets."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import tempfile
from collections.abc import Callable
from dataclasses import dataclass, replace
from enum import Enum
from pathlib import Path
from typing import cast

import numpy as np
import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_terminal_publisher_v1 as publisher
from numpy.typing import NDArray

PROFILE_PATH = "lab/profiles/physical-sound-v37-x0-complete-owner-terminal.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v37_x0_complete_owner_terminal_v1.py"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-x0-complete-owner-terminal-profile.v1"
)
PROFILE_ID = "physical-sound-v37-x0-complete-owner-terminal-v1"
PROFILE_SHA256 = "29eebe43719594298d130ebe1d959500d955db202e89aba4966ccacf342b5f66"
CLAIM = (
    "DISCARDED_SURROGATE_COMPLETE_OWNER_AND_ATOMIC_TERMINAL_CONFORMANCE_ONLY / "
    "NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_"
    "COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
FloatMatrix = NDArray[np.float64]


class X0Error(RuntimeError):
    """The complete-owner conformance proof is invalid."""


class InjectedOwnerError(RuntimeError):
    """A deterministic owner fault used only by discarded X0 fixtures."""


class RunFixture(str, Enum):
    NATURAL = "natural"
    METRIC_REJECT = "metric-reject"
    HARD_GATE_REJECT = "hard-gate-reject"
    RESOURCE_REJECT = "resource-reject"
    PRE_ACCESS_FAULT = "pre-access-fault"
    POST_ACCESS_FAULT = "post-access-fault"


@dataclass(frozen=True, slots=True)
class LinearModel:
    weights: FloatMatrix
    feature_names: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class OwnerResult:
    decision: contract.TerminalDecision
    trace: contract.ExecutionTraceV1
    receipt: publisher.PublicationReceipt | None
    metrics: dict[str, float]


class SurrogateProvider:
    def __init__(self, pipeline: contract.PipelineKind, label: str) -> None:
        self._kind = (
            contract.ProviderKind.SURROGATE_D0
            if pipeline is contract.PipelineKind.D0
            else contract.ProviderKind.SURROGATE_H0
        )
        self._namespace = f"discarded-v37-x0-{label}-{pipeline.value}"

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
            raise contract.ContractError("surrogate provider kind mismatch")
        if capability.namespace != self.namespace:
            raise contract.ContractError("surrogate provider namespace mismatch")
        return build_role(role)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise X0Error(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_object(data: bytes, label: str) -> dict[str, object]:
    try:
        value = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise X0Error(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise X0Error(f"JSON root must be an object: {label}")
    result = cast(dict[str, object], value)
    if canonical_json(result) != data:
        raise X0Error(f"JSON must be canonical: {label}")
    return result


def object_map(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise X0Error(f"expected object: {label}")
    return cast(dict[str, object], value)


def float_value(value: object, label: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise X0Error(f"expected numeric value: {label}")
    result = float(value)
    if not math.isfinite(result):
        raise X0Error(f"expected finite value: {label}")
    return result


def int_value(value: object, label: str) -> int:
    if type(value) is not int:
        raise X0Error(f"expected integer value: {label}")
    return value


def load_profile(path: Path) -> tuple[dict[str, object], bytes]:
    if not path.is_file() or path.is_symlink():
        raise X0Error("profile must be a regular file")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise X0Error("profile size outside bound")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise X0Error("profile hash mismatch")
    profile = load_object(data, "X0 profile")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
    ):
        raise X0Error("profile identity mismatch")
    authority = object_map(profile.get("authority"), "authority")
    if (
        authority.get("official_capability_allowed") is not False
        or authority.get("scientific_target_allowed") is not False
        or authority.get("surrogate_targets_allowed") is not True
        or authority.get("candidate_authority") != "discarded-surrogate-pass-only"
    ):
        raise X0Error("profile authority drift")
    dependencies = profile.get("dependencies")
    if not isinstance(dependencies, list):
        raise X0Error("dependency declarations missing")
    paths: list[str] = []
    for index, raw in enumerate(dependencies):
        declaration = object_map(raw, f"dependency {index}")
        path_text = declaration.get("path")
        expected = declaration.get("sha256")
        if not isinstance(path_text, str) or not isinstance(expected, str):
            raise X0Error("dependency declaration is invalid")
        dependency = repository_root() / path_text
        if (
            not dependency.is_file()
            or dependency.is_symlink()
            or sha256_bytes(dependency.read_bytes()) != expected
        ):
            raise X0Error(f"dependency drift: {path_text}")
        paths.append(path_text)
    if paths != sorted(set(paths)):
        raise X0Error("dependency declarations must be canonical and unique")
    return profile, data


def build_role(role: contract.RoleKind) -> contract.SurfaceQueryBatchV1:
    role_shift = {
        contract.RoleKind.TRAIN: 0.0,
        contract.RoleKind.DEVELOPMENT: 0.04,
        contract.RoleKind.METHOD_HOLDOUT: -0.03,
    }[role]
    fields: list[contract.FieldInputV1] = []
    queries: list[contract.QueryInputV1] = []
    targets: list[contract.TargetInputV1] = []
    positions = ((0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0))
    for field_index in range(2):
        field_id = f"{role.value}/field-{field_index}"
        probe_ids = tuple(f"{field_id}/probe-{index}" for index in range(4))
        probes = tuple(
            contract.ProbeInputV1(
                probe_id=probe_ids[index],
                position=(u, v, 0.0),
                normal=(0.0, 0.0, 1.0),
                area_weight=0.25,
                mode_value=0.8 + 0.1 * field_index + 0.05 * u - 0.03 * v,
                neighbor_ids=tuple(
                    probe_ids[other] for other in range(4) if other != index
                ),
            )
            for index, (u, v) in enumerate(positions)
        )
        fields.append(contract.FieldInputV1(field_id, probes))
        for query_index, (u, v) in enumerate(positions):
            row_id = f"{role.value}/row-{field_index}-{query_index}"
            p1_value = 0.75 + 0.08 * field_index + 0.04 * u - 0.02 * v
            context_values = (
                -0.2 + 0.1 * query_index + role_shift,
                0.3 + 0.15 * field_index - role_shift,
            )
            queries.append(
                contract.QueryInputV1(
                    row_id=row_id,
                    field_id=field_id,
                    triangle_id=f"{field_id}/triangle-{query_index % 2}",
                    position=(u, v, 0.0),
                    normal=(0.0, 0.0, 1.0),
                    barycentrics=(0.5, 0.25, 0.25),
                    context_values=context_values,
                    p1_contact_value=p1_value,
                )
            )
            targets.append(
                contract.TargetInputV1(
                    row_id=row_id,
                    values=(
                        0.1 + 0.7 * p1_value + 0.12 * u - 0.08 * v,
                        -0.2 + 0.4 * p1_value + 0.2 * context_values[0],
                        0.05 + 0.3 * p1_value + 0.15 * context_values[1],
                    ),
                )
            )
    field_set = contract.canonical_surface_fields(tuple(fields))
    return contract.canonical_query_batch(
        role,
        field_set,
        ("contact.log_impulse", "mode.log_frequency"),
        tuple(queries),
        tuple(targets),
    )


def design_matrix(batch: contract.SurfaceQueryBatchV1) -> FloatMatrix:
    field_means = np.array(
        [
            float(
                np.mean(
                    batch.fields.probe_mode_values[
                        int(batch.fields.field_offsets[index]) : int(
                            batch.fields.field_offsets[index + 1]
                        )
                    ]
                )
            )
            for index in range(batch.fields.field_count)
        ],
        dtype=np.float64,
    )
    return np.column_stack(
        (
            np.ones(batch.row_count, dtype=np.float64),
            batch.p1_contact_values,
            batch.query_positions[:, 0],
            batch.query_positions[:, 1],
            batch.context_features,
            field_means[batch.row_field_indices],
        )
    )


def require_targets(batch: contract.SurfaceQueryBatchV1) -> FloatMatrix:
    if batch.targets is None:
        raise X0Error("surrogate owner requires target-bearing roles")
    return batch.targets


def fit_candidate(
    batch: contract.SurfaceQueryBatchV1, ridge_lambda: float
) -> LinearModel:
    design = design_matrix(batch)
    targets = require_targets(batch)
    gram = design.T @ design + ridge_lambda * np.eye(design.shape[1])
    weights = np.asarray(np.linalg.solve(gram, design.T @ targets), dtype=np.float64)
    if not bool(np.all(np.isfinite(weights))):
        raise X0Error("candidate fit produced non-finite weights")
    return LinearModel(
        weights,
        (
            "bias",
            "p1_contact",
            "query_u",
            "query_v",
            "contact.log_impulse",
            "mode.log_frequency",
            "field.mean_mode",
        ),
    )


def predict(model: LinearModel, batch: contract.SurfaceQueryBatchV1) -> FloatMatrix:
    values = design_matrix(batch) @ model.weights
    if not bool(np.all(np.isfinite(values))):
        raise X0Error("candidate prediction is non-finite")
    return np.asarray(values, dtype=np.float64)


def fit_controls(
    batch: contract.SurfaceQueryBatchV1, ridge_lambda: float
) -> tuple[FloatMatrix, FloatMatrix]:
    targets = require_targets(batch)
    mean = np.mean(targets, axis=0, keepdims=True)
    p1_design = np.column_stack((np.ones(batch.row_count), batch.p1_contact_values))
    gram = p1_design.T @ p1_design + ridge_lambda * np.eye(2)
    p1_weights = np.linalg.solve(gram, p1_design.T @ targets)
    return np.asarray(mean, dtype=np.float64), np.asarray(p1_weights, dtype=np.float64)


def matrix_values(matrix: FloatMatrix) -> list[list[float]]:
    return [[float(value) for value in row] for row in matrix]


def model_payload(model: LinearModel) -> bytes:
    weights = canonical_json(matrix_values(model.weights))
    return canonical_json(
        {
            "feature_names": model.feature_names,
            "schema": "nextengine.experimental-physical-sound-v37-qso-bundle.v1",
            "weights": matrix_values(model.weights),
            "weights_sha256": sha256_bytes(weights),
        }
    )


def load_model_payload(data: bytes) -> tuple[LinearModel, str]:
    document = load_object(data, "candidate bundle")
    raw_names = document.get("feature_names")
    raw_weights = document.get("weights")
    expected_sha = document.get("weights_sha256")
    if not isinstance(raw_names, list) or not all(
        isinstance(value, str) for value in raw_names
    ):
        raise X0Error("candidate feature names are invalid")
    if not isinstance(raw_weights, list):
        raise X0Error("candidate weight matrix is invalid")
    try:
        weights = np.asarray(raw_weights, dtype=np.float64)
    except (TypeError, ValueError) as error:
        raise X0Error("candidate weight matrix cannot be decoded") from error
    if weights.ndim != 2 or weights.shape[1] != contract.TARGET_AXIS_COUNT:
        raise X0Error("candidate weight matrix shape mismatch")
    weights_data = canonical_json(matrix_values(weights))
    actual_sha = sha256_bytes(weights_data)
    if expected_sha != actual_sha:
        raise X0Error("candidate weight identity mismatch")
    return LinearModel(weights, tuple(cast(list[str], raw_names))), actual_sha


def rmse(predictions: FloatMatrix, targets: FloatMatrix) -> float:
    return float(np.sqrt(np.mean(np.square(predictions - targets))))


def receipt_record(receipt: publisher.PublicationReceipt) -> dict[str, object]:
    return {
        "decision_root_sha256": receipt.decision_root_sha256,
        "file_count": receipt.file_count,
        "terminal_sha256": receipt.terminal_sha256,
        "total_bytes": receipt.total_bytes,
        "tree_sha256": receipt.tree_sha256,
    }


class CompleteOwner:
    def __init__(
        self,
        pipeline: contract.PipelineKind,
        provider: SurrogateProvider,
        profile: dict[str, object],
        profile_sha256: str,
        output: Path,
        *,
        d0_pass_output: Path | None = None,
        fixture: RunFixture = RunFixture.NATURAL,
        failpoint: publisher.PublicationFailpoint | None = None,
        extra_evidence: dict[str, object] | None = None,
    ) -> None:
        self.pipeline = pipeline
        self.provider = provider
        self.profile = profile
        self.profile_sha256 = profile_sha256
        self.output = output
        self.d0_pass_output = d0_pass_output
        self.fixture = fixture
        self.failpoint = failpoint
        self.extra_evidence = extra_evidence
        self.owner_sha256 = sha256_bytes((repository_root() / OWNER_PATH).read_bytes())

    def _capability(self) -> contract.AccessCapabilityV1:
        return contract.AccessCapabilityV1.surrogate(
            self.provider.kind, self.provider.namespace
        )

    def _gates(self) -> dict[str, float | int]:
        raw = object_map(self.profile.get("gates"), "gates")
        gates: dict[str, float | int] = {
            "maximum_absolute_prediction": float_value(
                raw.get("maximum_absolute_prediction"), "maximum absolute prediction"
            ),
            "maximum_output_bytes": int_value(
                raw.get("maximum_output_bytes"), "maximum output bytes"
            ),
            "maximum_parameter_count": int_value(
                raw.get("maximum_parameter_count"), "maximum parameter count"
            ),
            "maximum_rmse": float_value(raw.get("maximum_rmse"), "maximum RMSE"),
        }
        if self.fixture in {
            RunFixture.METRIC_REJECT,
            RunFixture.HARD_GATE_REJECT,
            RunFixture.RESOURCE_REJECT,
        }:
            fixtures = object_map(self.profile.get("fixtures"), "fixtures")
            override = object_map(
                fixtures.get(self.fixture.value.replace("-", "_")), "fixture"
            )
            for key, value in override.items():
                if key == "maximum_parameter_count":
                    gates[key] = int_value(value, key)
                else:
                    gates[key] = float_value(value, key)
        return gates

    def _load_d0_candidate(self) -> tuple[LinearModel, str]:
        if self.d0_pass_output is None:
            raise X0Error("H0 candidate load requires a D0 Pass output")
        root = self.d0_pass_output
        terminal_data = (root / "terminal.json").read_bytes()
        terminal = load_object(terminal_data, "D0 terminal")
        freeze_data = (root / "candidate-freeze.json").read_bytes()
        bundle_data = (root / "candidate-bundle.json").read_bytes()
        payloads = object_map(terminal.get("payloads"), "D0 terminal payloads")
        for name, data in (
            ("candidate-bundle.json", bundle_data),
            ("candidate-freeze.json", freeze_data),
        ):
            record = object_map(payloads.get(name), f"D0 payload {name}")
            if record.get("sha256") != sha256_bytes(data):
                raise X0Error(f"D0 terminal does not bind {name}")
        model, weights_sha256 = load_model_payload(bundle_data)
        decision_root = terminal.get("decision_root_sha256")
        if not isinstance(decision_root, str):
            raise X0Error("D0 terminal decision root is absent")
        disposition = contract.CandidateDispositionV1(
            contract.TerminalDecision.PASS, True, freeze_data, None
        )
        contract.validate_h0_candidate_freeze(
            disposition,
            expected_owner_sha256=self.owner_sha256,
            expected_profile_sha256=self.profile_sha256,
            expected_terminal_sha256=decision_root,
            expected_weights_sha256=weights_sha256,
        )
        return model, weights_sha256

    def _publish(
        self,
        trace: contract.ExecutionTraceV1,
        model: LinearModel | None,
        weights_sha256: str | None,
        evidence: dict[str, object],
    ) -> publisher.PublicationReceipt:
        root = publisher.decision_root_sha256(
            trace, CLAIM, self.owner_sha256, self.profile_sha256, weights_sha256
        )
        disposition = contract.candidate_disposition(
            trace.terminal,
            owner_sha256=self.owner_sha256,
            profile_sha256=self.profile_sha256,
            terminal_sha256=root,
            candidate_weights_sha256=weights_sha256,
        )
        if trace.terminal is contract.TerminalDecision.PASS:
            if model is None or disposition.freeze_document is None:
                raise X0Error("Pass publication lacks a candidate")
            payloads = {
                "candidate-bundle.json": model_payload(model),
                "candidate-freeze.json": disposition.freeze_document,
                "evidence.json": canonical_json(evidence),
            }
        elif trace.terminal in {
            contract.TerminalDecision.METRIC_REJECT,
            contract.TerminalDecision.HARD_GATE_REJECT,
            contract.TerminalDecision.RESOURCE_REJECT,
        }:
            if disposition.rejected_evidence_document is None:
                raise X0Error("reject publication lacks rejected evidence")
            payloads = {
                "evidence.json": canonical_json(evidence),
                "rejected-candidate.json": disposition.rejected_evidence_document,
            }
        else:
            payloads = {"owner-fault.json": canonical_json(evidence)}
        gates = self._gates()
        return publisher.publish_terminal(
            self.output,
            repository_root(),
            trace,
            CLAIM,
            self.owner_sha256,
            self.profile_sha256,
            weights_sha256,
            disposition,
            payloads,
            cast(int, gates["maximum_output_bytes"]),
            failpoint=self.failpoint,
        )

    def run(self) -> OwnerResult:
        lifecycle = contract.OwnerLifecycleV1(self.pipeline, self._capability())
        metrics: dict[str, float] = {}
        try:
            if self.fixture is RunFixture.PRE_ACCESS_FAULT:
                raise InjectedOwnerError("injected before target access")
            lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
            train_stage = (
                contract.LifecycleStage.TRAIN_ROLE
                if self.pipeline is contract.PipelineKind.D0
                else contract.LifecycleStage.TRAIN_RECONSTRUCTION
            )
            train = lifecycle.materialize_role(
                train_stage, contract.RoleKind.TRAIN, self.provider
            )
            if self.fixture is RunFixture.POST_ACCESS_FAULT:
                raise InjectedOwnerError("injected after target access")
            surrogate = object_map(self.profile.get("surrogate"), "surrogate")
            ridge_lambda = float_value(surrogate.get("ridge_lambda"), "ridge lambda")
            if self.pipeline is contract.PipelineKind.D0:
                lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
                model = fit_candidate(train, ridge_lambda)
                weights_sha256 = sha256_bytes(
                    canonical_json(matrix_values(model.weights))
                )
                lifecycle.step(contract.LifecycleStage.CANDIDATE_TRAINING)
                mean_control, p1_control = fit_controls(train, ridge_lambda)
                lifecycle.step(contract.LifecycleStage.CONTROL_TRAINING)
                evaluation = lifecycle.materialize_role(
                    contract.LifecycleStage.DEVELOPMENT_ROLE,
                    contract.RoleKind.DEVELOPMENT,
                    self.provider,
                )
            else:
                mean_control, p1_control = fit_controls(train, ridge_lambda)
                model, weights_sha256 = self._load_d0_candidate()
                lifecycle.step(contract.LifecycleStage.CANDIDATE_LOAD)
                evaluation = lifecycle.materialize_role(
                    contract.LifecycleStage.METHOD_HOLDOUT_ROLE,
                    contract.RoleKind.METHOD_HOLDOUT,
                    self.provider,
                )
                lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
            predictions = predict(model, evaluation)
            lifecycle.step(contract.LifecycleStage.QUERY_EVALUATION)
            targets = require_targets(evaluation)
            mean_predictions = np.repeat(mean_control, evaluation.row_count, axis=0)
            p1_design = np.column_stack(
                (np.ones(evaluation.row_count), evaluation.p1_contact_values)
            )
            metrics = {
                "candidate_rmse": rmse(predictions, targets),
                "mean_control_rmse": rmse(mean_predictions, targets),
                "p1_control_rmse": rmse(p1_design @ p1_control, targets),
            }
            lifecycle.step(contract.LifecycleStage.METRIC_EVALUATION)
            maximum_prediction = float(np.max(np.abs(predictions)))
            lifecycle.step(contract.LifecycleStage.HARD_GATES)
            parameter_count = int(model.weights.size)
            lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
            gates = self._gates()
            if metrics["candidate_rmse"] > cast(float, gates["maximum_rmse"]):
                decision = contract.TerminalDecision.METRIC_REJECT
            elif maximum_prediction > cast(float, gates["maximum_absolute_prediction"]):
                decision = contract.TerminalDecision.HARD_GATE_REJECT
            elif parameter_count > cast(int, gates["maximum_parameter_count"]):
                decision = contract.TerminalDecision.RESOURCE_REJECT
            else:
                decision = contract.TerminalDecision.PASS
            trace = lifecycle.finish(decision)
            evidence: dict[str, object] = {
                "access": publisher.access_record(trace.access),
                "candidate_weights_sha256": weights_sha256,
                "claim": CLAIM,
                "field_root_sha256": evaluation.fields.root_sha256,
                "fixture": self.fixture.value,
                "gates": gates,
                "metrics": metrics,
                "official_access": {
                    "network_requests": 0,
                    "official_capabilities_issued": 0,
                    "official_target_rows": 0,
                    "protected_signal_values_decoded": 0,
                    "real_signal_values_decoded": 0,
                },
                "parameter_count": parameter_count,
                "pipeline": self.pipeline.value,
                "profile_sha256": self.profile_sha256,
                "schema": "nextengine.experimental-physical-sound-v37-x0-owner-evidence.v1",
                "structural_root_sha256": evaluation.structural_root_sha256,
                "target_root_sha256": evaluation.target_root_sha256,
            }
            if self.extra_evidence is not None:
                evidence["conformance"] = self.extra_evidence
            receipt = self._publish(trace, model, weights_sha256, evidence)
            return OwnerResult(decision, trace, receipt, metrics)
        except InjectedOwnerError as error:
            decision = publisher.unexpected_exception_decision(lifecycle.access)
            trace = lifecycle.finish(decision)
            if decision is contract.TerminalDecision.CONTRACT_REJECT:
                return OwnerResult(decision, trace, None, metrics)
            evidence = {
                "error_class": type(error).__name__,
                "message": str(error),
                "schema": "nextengine.experimental-physical-sound-v37-x0-owner-fault.v1",
            }
            disposition = contract.candidate_disposition(
                decision,
                owner_sha256=self.owner_sha256,
                profile_sha256=self.profile_sha256,
                terminal_sha256=publisher.decision_root_sha256(
                    trace, CLAIM, self.owner_sha256, self.profile_sha256, None
                ),
                candidate_weights_sha256=None,
            )
            gates = self._gates()
            receipt = publisher.publish_terminal(
                self.output,
                repository_root(),
                trace,
                CLAIM,
                self.owner_sha256,
                self.profile_sha256,
                None,
                disposition,
                {"owner-fault.json": canonical_json(evidence)},
                cast(int, gates["maximum_output_bytes"]),
                failpoint=self.failpoint,
            )
            return OwnerResult(decision, trace, receipt, metrics)


def run_owner(
    profile: dict[str, object],
    profile_sha256: str,
    root: Path,
    pipeline: contract.PipelineKind,
    fixture: RunFixture,
    *,
    d0_pass_output: Path | None = None,
    failpoint: publisher.PublicationFailpoint | None = None,
    label: str | None = None,
) -> OwnerResult:
    suffix = label or f"{pipeline.value}-{fixture.value}"
    return CompleteOwner(
        pipeline,
        SurrogateProvider(pipeline, suffix),
        profile,
        profile_sha256,
        root / suffix,
        d0_pass_output=d0_pass_output,
        fixture=fixture,
        failpoint=failpoint,
    ).run()


def expect_error(
    action: Callable[[], object], expected: type[Exception], fragment: str
) -> bool:
    try:
        action()
    except expected as error:
        return fragment in str(error)
    return False


def mutation_suite(
    profile: dict[str, object], profile_sha256: str, root: Path, d0_pass: Path
) -> dict[str, bool]:
    provider = SurrogateProvider(contract.PipelineKind.D0, "mutation")
    capability = contract.AccessCapabilityV1.surrogate(
        provider.kind, provider.namespace
    )
    lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )
    trace = lifecycle.finish(contract.TerminalDecision.OWNER_FAULT)
    callable_trace = replace(
        trace,
        events=(
            replace(trace.events[0], callable_id="mutated.callable"),
            *trace.events[1:],
        ),
    )
    h0_output = root / "mutation-h0"
    owner = CompleteOwner(
        contract.PipelineKind.H0,
        SurrogateProvider(contract.PipelineKind.H0, "mutation-h0"),
        profile,
        profile_sha256,
        h0_output,
        d0_pass_output=d0_pass,
    )
    freeze = (d0_pass / "candidate-freeze.json").read_bytes()
    bad_freeze = freeze.replace(
        b"CandidateFrozenAfterPass", b"CandidateFrozenAfterFail"
    )
    freeze_rejected = expect_error(
        lambda: contract.validate_h0_candidate_freeze(
            contract.CandidateDispositionV1(
                contract.TerminalDecision.PASS, True, bad_freeze, None
            ),
            expected_owner_sha256=owner.owner_sha256,
            expected_profile_sha256=profile_sha256,
            expected_terminal_sha256="0" * 64,
            expected_weights_sha256="1" * 64,
        ),
        contract.ContractError,
        "mismatch",
    )
    return {
        "candidate_disposition_contradiction_rejected": expect_error(
            lambda: contract.CandidateDispositionV1(
                contract.TerminalDecision.METRIC_REJECT, True, freeze, None
            ),
            contract.ContractError,
            "contradicts",
        ),
        "H0_mutated_freeze_rejected": freeze_rejected,
        "lifecycle_reorder_rejected": expect_error(
            lambda: contract.OwnerLifecycleV1(
                contract.PipelineKind.D0, capability
            ).step(contract.LifecycleStage.FIELD_ENCODING),
            contract.ContractError,
            "stage mismatch",
        ),
        "trace_callable_mutation_rejected": expect_error(
            lambda: publisher.validate_trace(callable_trace),
            publisher.PublicationError,
            "callable mutation",
        ),
    }


def conformance_suite(
    profile: dict[str, object], profile_sha256: str
) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="nextengine-v37-x0-") as temporary:
        root = Path(temporary)
        d0_pass_result = run_owner(
            profile,
            profile_sha256,
            root,
            contract.PipelineKind.D0,
            RunFixture.NATURAL,
            label="d0-natural-pass",
        )
        d0_pass = root / "d0-natural-pass"
        h0_pass_result = run_owner(
            profile,
            profile_sha256,
            root,
            contract.PipelineKind.H0,
            RunFixture.NATURAL,
            d0_pass_output=d0_pass,
            label="h0-natural-pass",
        )
        terminals: dict[str, object] = {
            "d0/Pass": receipt_record(
                cast(publisher.PublicationReceipt, d0_pass_result.receipt)
            ),
            "h0/Pass": receipt_record(
                cast(publisher.PublicationReceipt, h0_pass_result.receipt)
            ),
        }
        expected = {
            RunFixture.METRIC_REJECT: contract.TerminalDecision.METRIC_REJECT,
            RunFixture.HARD_GATE_REJECT: contract.TerminalDecision.HARD_GATE_REJECT,
            RunFixture.RESOURCE_REJECT: contract.TerminalDecision.RESOURCE_REJECT,
        }
        shape_checks: dict[str, bool] = {}
        for pipeline in contract.PipelineKind:
            for fixture, decision in expected.items():
                result = run_owner(
                    profile,
                    profile_sha256,
                    root,
                    pipeline,
                    fixture,
                    d0_pass_output=d0_pass
                    if pipeline is contract.PipelineKind.H0
                    else None,
                )
                if result.decision is not decision or result.receipt is None:
                    raise X0Error(
                        f"terminal fixture failed: {pipeline.value}/{decision.value}"
                    )
                terminals[f"{pipeline.value}/{decision.value}"] = receipt_record(
                    result.receipt
                )
                output = Path(result.receipt.output)
                names = {entry.name for entry in output.iterdir()}
                shape_checks[f"{pipeline.value}/{decision.value}"] = names == {
                    "evidence.json",
                    "rejected-candidate.json",
                    "terminal.json",
                }
        exception_checks: dict[str, bool] = {}
        for pipeline in contract.PipelineKind:
            pre = run_owner(
                profile,
                profile_sha256,
                root,
                pipeline,
                RunFixture.PRE_ACCESS_FAULT,
                d0_pass_output=d0_pass
                if pipeline is contract.PipelineKind.H0
                else None,
            )
            post = run_owner(
                profile,
                profile_sha256,
                root,
                pipeline,
                RunFixture.POST_ACCESS_FAULT,
                d0_pass_output=d0_pass
                if pipeline is contract.PipelineKind.H0
                else None,
            )
            exception_checks[f"{pipeline.value}/pre-access"] = (
                pre.decision is contract.TerminalDecision.CONTRACT_REJECT
                and pre.receipt is None
                and not (root / f"{pipeline.value}-pre-access-fault").exists()
            )
            exception_checks[f"{pipeline.value}/post-access"] = (
                post.decision is contract.TerminalDecision.OWNER_FAULT
                and post.receipt is not None
                and {entry.name for entry in Path(post.receipt.output).iterdir()}
                == {"owner-fault.json", "terminal.json"}
            )
        failpoints: dict[str, bool] = {}
        for point in publisher.PublicationFailpoint:
            label = f"failpoint-{point.value}"
            output = root / label
            staging = root / f".{label}.nextengine-v37-terminal-staging"
            try:
                run_owner(
                    profile,
                    profile_sha256,
                    root,
                    contract.PipelineKind.D0,
                    RunFixture.METRIC_REJECT,
                    failpoint=point,
                    label=label,
                )
            except publisher.PublicationError as error:
                injected = "injected publication failure" in str(error)
            else:
                injected = False
            failpoints[point.value] = (
                injected and not output.exists() and not staging.exists()
            )
        mutations = mutation_suite(profile, profile_sha256, root, d0_pass)
        all_pass = (
            d0_pass_result.decision is contract.TerminalDecision.PASS
            and h0_pass_result.decision is contract.TerminalDecision.PASS
            and len(terminals) == 8
            and all(shape_checks.values())
            and all(exception_checks.values())
            and all(failpoints.values())
            and all(mutations.values())
        )
        if not all_pass:
            raise X0Error("complete-owner conformance suite failed")
        return {
            "all_pass": all_pass,
            "exception_checks": exception_checks,
            "failpoints": failpoints,
            "mutation_checks": mutations,
            "official_access": {
                "network_requests": 0,
                "official_capabilities_issued": 0,
                "official_target_rows": 0,
                "protected_signal_values_decoded": 0,
                "real_signal_values_decoded": 0,
            },
            "scientific_terminal_shapes": shape_checks,
            "terminals": terminals,
        }


def run(profile_path: Path, output: Path) -> dict[str, object]:
    profile, profile_data = load_profile(profile_path)
    profile_sha256 = sha256_bytes(profile_data)
    conformance = conformance_suite(profile, profile_sha256)
    result = CompleteOwner(
        contract.PipelineKind.D0,
        SurrogateProvider(contract.PipelineKind.D0, "top-level"),
        profile,
        profile_sha256,
        output,
        extra_evidence=conformance,
    ).run()
    if result.decision is not contract.TerminalDecision.PASS or result.receipt is None:
        raise X0Error("top-level natural owner did not Pass")
    return {
        "claim": CLAIM,
        "decision": result.decision.value,
        "metrics": result.metrics,
        "next_authorized_stage": "V37-E0-full-surrogate-rehearsal-and-execution-seal",
        "official_access": conformance["official_access"],
        "receipt": receipt_record(result.receipt),
        "schema": "nextengine.experimental-physical-sound-v37-x0-report.v1",
        "status": "X0_COMPLETE_OWNER_AND_ATOMIC_TERMINAL_PASS",
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default=PROFILE_PATH, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    report = run(arguments.profile, arguments.output.absolute())
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
