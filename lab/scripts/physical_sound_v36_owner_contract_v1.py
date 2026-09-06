"""Typed, target-agnostic execution contract for V36 physical-sound owners."""

from __future__ import annotations

import hashlib
import math
from dataclasses import dataclass
from enum import Enum
from typing import Protocol, TypeAlias, assert_never

import numpy as np
from numpy.typing import NDArray

CONTRACT_SCHEMA = "nextengine.experimental-physical-sound-v36-owner-contract.v1"
TARGET_AXIS_COUNT = 3
LOCAL_KEY_WIDTH = 15
GEOMETRY_KEY_WIDTH = 3
CONTACT_KEY_WIDTH = 2

FloatMatrix: TypeAlias = NDArray[np.float64]


class ContractError(RuntimeError):
    """A V36 owner/provider/lifecycle contract is invalid."""


class RoleKind(str, Enum):
    TRAIN = "train"
    DEVELOPMENT = "development"
    METHOD_HOLDOUT = "method-holdout"


class PipelineKind(str, Enum):
    D0 = "d0"
    H0 = "h0"


class ProviderKind(str, Enum):
    SURROGATE_D0 = "surrogate-d0"
    SURROGATE_H0 = "surrogate-h0"
    OFFICIAL_D0 = "official-d0"
    OFFICIAL_H0 = "official-h0"


class LifecycleStage(str, Enum):
    PRE_ACCESS_CONTEXT = "pre-access-context"
    TRAIN_ROLE = "train-role"
    LOCAL_CROSS_FIT = "local-cross-fit"
    TRAINING_CANDIDATE = "training-candidate"
    TRAINING_RAW_MLP = "training-raw-mlp"
    TRAINING_V34_SHAPED_SPECTRAL_MLP = "training-v34-shaped-spectral-mlp"
    TRAINING_WITHOUT_EXPLICIT_GEOMETRY = "training-without-explicit-geometry"
    RIDGE_CONTROLS = "ridge-controls"
    DEVELOPMENT_ROLE = "development-role"
    DEVELOPMENT_HYBRID_SUPPORT = "development-hybrid-support"
    DEVELOPMENT_PREDICTIONS = "development-predictions"
    DEVELOPMENT_EVALUATION = "development-evaluation"
    TRAIN_RECONSTRUCTION = "train-reconstruction"
    CANDIDATE_LOAD = "candidate-load"
    METHOD_HOLDOUT_ROLE = "method-holdout-role"
    METHOD_HOLDOUT_HYBRID_SUPPORT = "method-holdout-hybrid-support"
    METHOD_HOLDOUT_PREDICTIONS = "method-holdout-predictions"
    METHOD_HOLDOUT_EVALUATION = "method-holdout-evaluation"
    HARD_GATES = "hard-gates"
    RESOURCE_GATES = "resource-gates"
    TERMINAL_PUBLICATION = "terminal-publication"


class TerminalDecision(str, Enum):
    PASS = "Pass"
    METRIC_REJECT = "MetricReject"
    HARD_GATE_REJECT = "HardGateReject"
    RESOURCE_REJECT = "ResourceReject"
    OWNER_FAULT = "OwnerFault"
    CONTRACT_REJECT = "ContractReject"


D0_STAGES = (
    LifecycleStage.PRE_ACCESS_CONTEXT,
    LifecycleStage.TRAIN_ROLE,
    LifecycleStage.LOCAL_CROSS_FIT,
    LifecycleStage.TRAINING_CANDIDATE,
    LifecycleStage.TRAINING_RAW_MLP,
    LifecycleStage.TRAINING_V34_SHAPED_SPECTRAL_MLP,
    LifecycleStage.TRAINING_WITHOUT_EXPLICIT_GEOMETRY,
    LifecycleStage.RIDGE_CONTROLS,
    LifecycleStage.DEVELOPMENT_ROLE,
    LifecycleStage.DEVELOPMENT_HYBRID_SUPPORT,
    LifecycleStage.DEVELOPMENT_PREDICTIONS,
    LifecycleStage.DEVELOPMENT_EVALUATION,
    LifecycleStage.HARD_GATES,
    LifecycleStage.RESOURCE_GATES,
)

H0_STAGES = (
    LifecycleStage.PRE_ACCESS_CONTEXT,
    LifecycleStage.TRAIN_RECONSTRUCTION,
    LifecycleStage.CANDIDATE_LOAD,
    LifecycleStage.METHOD_HOLDOUT_ROLE,
    LifecycleStage.METHOD_HOLDOUT_HYBRID_SUPPORT,
    LifecycleStage.METHOD_HOLDOUT_PREDICTIONS,
    LifecycleStage.METHOD_HOLDOUT_EVALUATION,
    LifecycleStage.HARD_GATES,
    LifecycleStage.RESOURCE_GATES,
)

STAGE_CALLABLES = {
    LifecycleStage.PRE_ACCESS_CONTEXT: "owner.load-and-validate-context.v1",
    LifecycleStage.TRAIN_ROLE: "provider.materialize-role.v1",
    LifecycleStage.LOCAL_CROSS_FIT: "owner.compute-local-cross-fit.v1",
    LifecycleStage.TRAINING_CANDIDATE: "owner.fit-candidate.v1",
    LifecycleStage.TRAINING_RAW_MLP: "owner.fit-raw-mlp.v1",
    LifecycleStage.TRAINING_V34_SHAPED_SPECTRAL_MLP: (
        "owner.fit-v34-shaped-spectral-mlp.v1"
    ),
    LifecycleStage.TRAINING_WITHOUT_EXPLICIT_GEOMETRY: (
        "owner.fit-without-explicit-geometry.v1"
    ),
    LifecycleStage.RIDGE_CONTROLS: "owner.fit-ridge-controls.v1",
    LifecycleStage.DEVELOPMENT_ROLE: "provider.materialize-role.v1",
    LifecycleStage.DEVELOPMENT_HYBRID_SUPPORT: (
        "owner.compute-development-hybrid-support.v1"
    ),
    LifecycleStage.DEVELOPMENT_PREDICTIONS: "owner.predict-development.v1",
    LifecycleStage.DEVELOPMENT_EVALUATION: "owner.evaluate-development.v1",
    LifecycleStage.TRAIN_RECONSTRUCTION: "provider.materialize-role.v1",
    LifecycleStage.CANDIDATE_LOAD: "owner.load-frozen-candidate.v1",
    LifecycleStage.METHOD_HOLDOUT_ROLE: "provider.materialize-role.v1",
    LifecycleStage.METHOD_HOLDOUT_HYBRID_SUPPORT: (
        "owner.compute-method-holdout-hybrid-support.v1"
    ),
    LifecycleStage.METHOD_HOLDOUT_PREDICTIONS: "owner.predict-method-holdout.v1",
    LifecycleStage.METHOD_HOLDOUT_EVALUATION: "owner.evaluate-method-holdout.v1",
    LifecycleStage.HARD_GATES: "owner.evaluate-hard-gates.v1",
    LifecycleStage.RESOURCE_GATES: "owner.evaluate-resource-gates.v1",
    LifecycleStage.TERMINAL_PUBLICATION: "owner.publish-terminal-atomically.v1",
}

ROLE_STAGE = {
    LifecycleStage.TRAIN_ROLE: RoleKind.TRAIN,
    LifecycleStage.DEVELOPMENT_ROLE: RoleKind.DEVELOPMENT,
    LifecycleStage.TRAIN_RECONSTRUCTION: RoleKind.TRAIN,
    LifecycleStage.METHOD_HOLDOUT_ROLE: RoleKind.METHOD_HOLDOUT,
}


def _is_sha256(value: str) -> bool:
    return len(value) == 64 and all(
        character in "0123456789abcdef" for character in value
    )


def pipeline_for_provider(kind: ProviderKind) -> PipelineKind:
    match kind:
        case ProviderKind.SURROGATE_D0 | ProviderKind.OFFICIAL_D0:
            return PipelineKind.D0
        case ProviderKind.SURROGATE_H0 | ProviderKind.OFFICIAL_H0:
            return PipelineKind.H0
    assert_never(kind)


def stages_for_pipeline(pipeline: PipelineKind) -> tuple[LifecycleStage, ...]:
    match pipeline:
        case PipelineKind.D0:
            return D0_STAGES
        case PipelineKind.H0:
            return H0_STAGES
    assert_never(pipeline)


def allowed_roles(kind: ProviderKind) -> tuple[RoleKind, ...]:
    match kind:
        case ProviderKind.SURROGATE_D0 | ProviderKind.OFFICIAL_D0:
            return (RoleKind.TRAIN, RoleKind.DEVELOPMENT)
        case ProviderKind.SURROGATE_H0 | ProviderKind.OFFICIAL_H0:
            return (RoleKind.TRAIN, RoleKind.METHOD_HOLDOUT)
    assert_never(kind)


def frozen_matrix(rows: tuple[tuple[float, ...], ...]) -> FloatMatrix:
    """Copy finite two-dimensional rows into immutable canonical float64 storage."""

    matrix = np.array(rows, dtype=np.float64, order="C", copy=True)
    if matrix.ndim != 2 or matrix.shape[0] == 0 or matrix.shape[1] == 0:
        raise ContractError("matrix must be nonempty and two-dimensional")
    if not bool(np.all(np.isfinite(matrix))):
        raise ContractError("matrix contains a non-finite value")
    matrix.flags.writeable = False
    return matrix


@dataclass(frozen=True, slots=True)
class FeatureMatrices:
    decay: FloatMatrix
    global_gain: FloatMatrix
    contact: FloatMatrix

    def values(self) -> tuple[FloatMatrix, FloatMatrix, FloatMatrix]:
        return (self.decay, self.global_gain, self.contact)


@dataclass(frozen=True, slots=True)
class CaseSpan:
    case_id: str
    start: int
    end: int
    stratum: str

    def __post_init__(self) -> None:
        if (
            not self.case_id
            or not self.stratum
            or self.start < 0
            or self.end <= self.start
        ):
            raise ContractError("case span is invalid")


@dataclass(frozen=True, slots=True)
class StratumRows:
    name: str
    indices: tuple[int, ...]

    def __post_init__(self) -> None:
        if not self.name or not self.indices:
            raise ContractError("stratum rows are empty")
        if tuple(sorted(set(self.indices))) != self.indices or self.indices[0] < 0:
            raise ContractError(
                "stratum indices must be sorted, unique and nonnegative"
            )


@dataclass(frozen=True, slots=True)
class RoleBatch:
    """One concrete immutable role container. It intentionally has no __len__."""

    contract_schema: str
    role: RoleKind
    row_ids: tuple[str, ...]
    features: FeatureMatrices
    no_geometry_features: FeatureMatrices
    raw_features: FeatureMatrices
    targets: FloatMatrix
    local_keys: FloatMatrix
    geometry_keys: FloatMatrix
    contact_keys: FloatMatrix
    local_partition_ids: tuple[str, ...]
    local_group_ids: tuple[str, ...]
    case_spans: tuple[CaseSpan, ...]
    strata: tuple[StratumRows, ...]
    role_local_geometry_groups: int

    @property
    def row_count(self) -> int:
        return len(self.row_ids)

    @property
    def case_count(self) -> int:
        return len(self.case_spans)

    def __post_init__(self) -> None:
        if self.contract_schema != CONTRACT_SCHEMA:
            raise ContractError("role batch contract schema mismatch")
        row_count = self.row_count
        if row_count == 0 or len(set(self.row_ids)) != row_count:
            raise ContractError("row IDs must be nonempty and unique")
        if any(not row_id for row_id in self.row_ids):
            raise ContractError("row ID must not be empty")
        if self.role_local_geometry_groups <= 0:
            raise ContractError("role-local geometry group count must be positive")
        self._validate_matrix(self.targets, TARGET_AXIS_COUNT, "targets")
        self._validate_matrix(self.local_keys, LOCAL_KEY_WIDTH, "local keys")
        self._validate_matrix(self.geometry_keys, GEOMETRY_KEY_WIDTH, "geometry keys")
        self._validate_matrix(self.contact_keys, CONTACT_KEY_WIDTH, "contact keys")
        for name, matrices in (
            ("features", self.features),
            ("no-geometry features", self.no_geometry_features),
            ("raw features", self.raw_features),
        ):
            for matrix in matrices.values():
                self._validate_matrix(matrix, None, name)
        if (
            len(self.local_partition_ids) != row_count
            or len(self.local_group_ids) != row_count
        ):
            raise ContractError("local partition/group identity count mismatch")
        if any(
            not value for value in (*self.local_partition_ids, *self.local_group_ids)
        ):
            raise ContractError("local partition/group identity must not be empty")
        cursor = 0
        for span in self.case_spans:
            if span.start != cursor or span.end > row_count:
                raise ContractError("case spans must cover rows contiguously")
            cursor = span.end
        if cursor != row_count:
            raise ContractError("case spans do not cover every row")
        covered: list[int] = []
        for stratum in self.strata:
            if stratum.indices[-1] >= row_count:
                raise ContractError("stratum index is outside the role batch")
            covered.extend(stratum.indices)
        if tuple(sorted(covered)) != tuple(range(row_count)):
            raise ContractError("strata must partition every role row exactly once")

    def _validate_matrix(
        self, matrix: FloatMatrix, width: int | None, name: str
    ) -> None:
        if matrix.ndim != 2 or matrix.shape[0] != self.row_count:
            raise ContractError(f"{name} row count mismatch")
        if width is not None and matrix.shape[1] != width:
            raise ContractError(f"{name} width mismatch")
        if matrix.shape[1] == 0 or matrix.dtype != np.dtype(np.float64):
            raise ContractError(f"{name} must be a nonempty float64 matrix")
        if matrix.flags.writeable or not matrix.flags.c_contiguous:
            raise ContractError(f"{name} must be immutable and C-contiguous")
        if not bool(np.all(np.isfinite(matrix))):
            raise ContractError(f"{name} contains a non-finite value")


@dataclass(frozen=True, slots=True)
class ExecutionSeal:
    contract_schema: str
    owner_sha256: str
    profile_sha256: str
    environment_sha256: str
    d0_rehearsal_root_sha256: str
    h0_rehearsal_root_sha256: str
    d0_topology_sha256: str
    h0_topology_sha256: str
    rehearsal_run_count: int
    repeat_exact: bool
    forbidden_access_count: int

    def __post_init__(self) -> None:
        hashes = (
            self.owner_sha256,
            self.profile_sha256,
            self.environment_sha256,
            self.d0_rehearsal_root_sha256,
            self.h0_rehearsal_root_sha256,
            self.d0_topology_sha256,
            self.h0_topology_sha256,
        )
        if self.contract_schema != CONTRACT_SCHEMA or not all(map(_is_sha256, hashes)):
            raise ContractError("execution seal identity is invalid")
        if self.rehearsal_run_count != 2 or not self.repeat_exact:
            raise ContractError("execution seal requires two repeat-exact rehearsals")
        if self.forbidden_access_count != 0:
            raise ContractError("execution seal has forbidden access")
        if self.d0_topology_sha256 != expected_topology_sha256(PipelineKind.D0):
            raise ContractError("D0 topology identity mismatch")
        if self.h0_topology_sha256 != expected_topology_sha256(PipelineKind.H0):
            raise ContractError("H0 topology identity mismatch")


@dataclass(frozen=True, slots=True)
class AccessCapability:
    provider_kind: ProviderKind
    namespace: str
    allowed_roles: tuple[RoleKind, ...]
    execution_seal: ExecutionSeal | None

    @classmethod
    def surrogate(cls, provider_kind: ProviderKind, namespace: str) -> AccessCapability:
        if provider_kind not in (
            ProviderKind.SURROGATE_D0,
            ProviderKind.SURROGATE_H0,
        ):
            raise ContractError("surrogate capability requires a surrogate provider")
        return cls(provider_kind, namespace, allowed_roles(provider_kind), None)

    @classmethod
    def official(
        cls,
        provider_kind: ProviderKind,
        namespace: str,
        execution_seal: ExecutionSeal,
    ) -> AccessCapability:
        if provider_kind not in (ProviderKind.OFFICIAL_D0, ProviderKind.OFFICIAL_H0):
            raise ContractError("official capability requires an official provider")
        return cls(
            provider_kind,
            namespace,
            allowed_roles(provider_kind),
            execution_seal,
        )

    def __post_init__(self) -> None:
        if not self.namespace or tuple(
            sorted(set(self.allowed_roles), key=lambda role: role.value)
        ) != tuple(
            sorted(allowed_roles(self.provider_kind), key=lambda role: role.value)
        ):
            raise ContractError("capability namespace or role set is invalid")
        if self.provider_kind in (
            ProviderKind.SURROGATE_D0,
            ProviderKind.SURROGATE_H0,
        ):
            if self.execution_seal is not None or not self.namespace.startswith(
                "discarded-"
            ):
                raise ContractError(
                    "surrogate capability must use a discarded namespace"
                )
        elif self.execution_seal is None or not self.namespace.startswith("v36-"):
            raise ContractError("official capability requires a V36 namespace and seal")


class TargetRoleProvider(Protocol):
    @property
    def kind(self) -> ProviderKind: ...

    @property
    def namespace(self) -> str: ...

    def materialize(
        self, role: RoleKind, capability: AccessCapability
    ) -> RoleBatch: ...


@dataclass(frozen=True, slots=True)
class AccessLedger:
    provider_calls: int = 0
    train_target_rows: int = 0
    development_target_rows: int = 0
    method_holdout_target_rows: int = 0
    official_d0_target_rows: int = 0
    official_h0_target_rows: int = 0
    real_signal_values_decoded: int = 0
    protected_signal_values_decoded: int = 0
    network_requests: int = 0
    prior_generation_values_read: int = 0

    @property
    def target_rows_accessed(self) -> int:
        return (
            self.train_target_rows
            + self.development_target_rows
            + self.method_holdout_target_rows
        )

    @property
    def forbidden_access_count(self) -> int:
        return (
            self.real_signal_values_decoded
            + self.protected_signal_values_decoded
            + self.network_requests
            + self.prior_generation_values_read
        )

    def record(
        self, provider_kind: ProviderKind, role: RoleKind, row_count: int
    ) -> AccessLedger:
        if row_count <= 0:
            raise ContractError("provider returned an empty role")
        train = self.train_target_rows
        development = self.development_target_rows
        holdout = self.method_holdout_target_rows
        match role:
            case RoleKind.TRAIN:
                train += row_count
            case RoleKind.DEVELOPMENT:
                development += row_count
            case RoleKind.METHOD_HOLDOUT:
                holdout += row_count
        official_d0 = self.official_d0_target_rows
        official_h0 = self.official_h0_target_rows
        if provider_kind is ProviderKind.OFFICIAL_D0:
            official_d0 += row_count
        elif provider_kind is ProviderKind.OFFICIAL_H0:
            official_h0 += row_count
        return AccessLedger(
            provider_calls=self.provider_calls + 1,
            train_target_rows=train,
            development_target_rows=development,
            method_holdout_target_rows=holdout,
            official_d0_target_rows=official_d0,
            official_h0_target_rows=official_h0,
            real_signal_values_decoded=self.real_signal_values_decoded,
            protected_signal_values_decoded=self.protected_signal_values_decoded,
            network_requests=self.network_requests,
            prior_generation_values_read=self.prior_generation_values_read,
        )


@dataclass(frozen=True, slots=True)
class TraceEvent:
    ordinal: int
    stage: LifecycleStage
    callable_id: str

    def topology_line(self) -> str:
        return f"{self.ordinal:02d}|{self.stage.value}|{self.callable_id}\n"


@dataclass(frozen=True, slots=True)
class ExecutionTrace:
    pipeline: PipelineKind
    provider_kind: ProviderKind
    events: tuple[TraceEvent, ...]
    terminal: TerminalDecision
    access: AccessLedger

    @property
    def topology_sha256(self) -> str:
        payload = "".join(event.topology_line() for event in self.events).encode()
        return hashlib.sha256(payload).hexdigest()


def expected_topology_sha256(pipeline: PipelineKind) -> str:
    stages = (*stages_for_pipeline(pipeline), LifecycleStage.TERMINAL_PUBLICATION)
    payload = "".join(
        f"{ordinal:02d}|{stage.value}|{STAGE_CALLABLES[stage]}\n"
        for ordinal, stage in enumerate(stages)
    ).encode()
    return hashlib.sha256(payload).hexdigest()


def terminal_exit_code(decision: TerminalDecision) -> int:
    match decision:
        case TerminalDecision.PASS:
            return 0
        case (
            TerminalDecision.METRIC_REJECT
            | TerminalDecision.HARD_GATE_REJECT
            | TerminalDecision.RESOURCE_REJECT
            | TerminalDecision.OWNER_FAULT
        ):
            return 1
        case TerminalDecision.CONTRACT_REJECT:
            return 2
    assert_never(decision)


class OwnerLifecycle:
    """The sole stage/access state machine used by future V36 D0/H0 owners."""

    def __init__(self, pipeline: PipelineKind, capability: AccessCapability) -> None:
        if pipeline_for_provider(capability.provider_kind) is not pipeline:
            raise ContractError("provider capability does not match the pipeline")
        self._pipeline = pipeline
        self._capability = capability
        self._stages = stages_for_pipeline(pipeline)
        self._cursor = 0
        self._events: list[TraceEvent] = []
        self._opened_roles: set[RoleKind] = set()
        self._terminal: TerminalDecision | None = None
        self._ledger = AccessLedger()

    @property
    def access(self) -> AccessLedger:
        return self._ledger

    @property
    def current_stage(self) -> LifecycleStage | None:
        if not self._events:
            return None
        return self._events[-1].stage

    def step(self, stage: LifecycleStage) -> None:
        if stage in ROLE_STAGE:
            raise ContractError("role stage must execute through materialize_role")
        self._enter(stage)

    def materialize_role(
        self,
        stage: LifecycleStage,
        role: RoleKind,
        provider: TargetRoleProvider,
    ) -> RoleBatch:
        expected_role = ROLE_STAGE.get(stage)
        if expected_role is None or expected_role is not role:
            raise ContractError("stage and role materialization do not match")
        if role in self._opened_roles:
            raise ContractError("role may be materialized only once")
        if role not in self._capability.allowed_roles:
            raise ContractError("role is outside the capability")
        if (
            provider.kind is not self._capability.provider_kind
            or provider.namespace != self._capability.namespace
        ):
            raise ContractError("provider identity does not match the capability")
        self._enter(stage)
        batch = provider.materialize(role, self._capability)
        if batch.contract_schema != CONTRACT_SCHEMA or batch.role is not role:
            raise ContractError("provider returned the wrong concrete role batch")
        self._opened_roles.add(role)
        self._ledger = self._ledger.record(provider.kind, role, batch.row_count)
        return batch

    def finish(self, decision: TerminalDecision) -> ExecutionTrace:
        if self._terminal is not None:
            raise ContractError("terminal decision already published")
        completed = self._cursor == len(self._stages)
        if (
            decision
            in (
                TerminalDecision.PASS,
                TerminalDecision.METRIC_REJECT,
                TerminalDecision.HARD_GATE_REJECT,
                TerminalDecision.RESOURCE_REJECT,
            )
            and not completed
        ):
            raise ContractError("scientific terminal requires the complete owner path")
        if (
            decision is TerminalDecision.CONTRACT_REJECT
            and self._ledger.target_rows_accessed
        ):
            raise ContractError("contract reject is allowed only before target access")
        if (
            decision is TerminalDecision.OWNER_FAULT
            and not self._ledger.target_rows_accessed
        ):
            raise ContractError("owner fault is post-access only")
        self._events.append(
            TraceEvent(
                ordinal=len(self._events),
                stage=LifecycleStage.TERMINAL_PUBLICATION,
                callable_id=STAGE_CALLABLES[LifecycleStage.TERMINAL_PUBLICATION],
            )
        )
        self._terminal = decision
        return ExecutionTrace(
            pipeline=self._pipeline,
            provider_kind=self._capability.provider_kind,
            events=tuple(self._events),
            terminal=decision,
            access=self._ledger,
        )

    def _enter(self, stage: LifecycleStage) -> None:
        if self._terminal is not None:
            raise ContractError("owner cannot advance after terminal publication")
        if self._cursor >= len(self._stages) or self._stages[self._cursor] is not stage:
            expected = (
                self._stages[self._cursor].value
                if self._cursor < len(self._stages)
                else "terminal-publication"
            )
            raise ContractError(
                f"lifecycle stage mismatch: expected {expected}, got {stage.value}"
            )
        callable_id = STAGE_CALLABLES.get(stage)
        if callable_id is None:
            raise ContractError("stage has no frozen callable identity")
        self._events.append(
            TraceEvent(
                ordinal=len(self._events),
                stage=stage,
                callable_id=callable_id,
            )
        )
        self._cursor += 1


def assert_complete_trace(trace: ExecutionTrace) -> None:
    expected_pipeline = pipeline_for_provider(trace.provider_kind)
    if trace.pipeline is not expected_pipeline:
        raise ContractError("trace pipeline/provider mismatch")
    if trace.topology_sha256 != expected_topology_sha256(trace.pipeline):
        raise ContractError("trace topology is not the complete frozen owner path")
    if trace.access.forbidden_access_count != 0:
        raise ContractError("trace contains forbidden access")
    if not math.isfinite(float(trace.access.target_rows_accessed)):
        raise ContractError("trace access count is invalid")
