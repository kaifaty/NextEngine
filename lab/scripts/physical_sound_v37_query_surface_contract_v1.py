#!/usr/bin/env python3
"""Typed target-agnostic contract for the V37 query-surface operator."""

from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping
from dataclasses import dataclass
from enum import Enum
from typing import Protocol, TypeAlias, assert_never, cast

import numpy as np
from numpy.typing import NDArray

CONTRACT_SCHEMA = "nextengine.experimental-physical-sound-v37-query-surface-contract.v1"
TARGET_AXIS_COUNT = 3
FORBIDDEN_CONTEXT_FEATURES = {
    "development-stratum",
    "material-label",
    "object-id",
    "role-id",
    "target-distance",
}

FloatMatrix: TypeAlias = NDArray[np.float64]
FloatVector: TypeAlias = NDArray[np.float64]
IntVector: TypeAlias = NDArray[np.int64]


class ContractError(RuntimeError):
    """A V37 surface/query/provider/disposition contract is invalid."""


class RoleKind(str, Enum):
    TRAIN = "train"
    DEVELOPMENT = "development"
    METHOD_HOLDOUT = "method-holdout"


class PipelineKind(str, Enum):
    D0 = "d0"
    H0 = "h0"


class ProviderKind(str, Enum):
    STRUCTURAL_D0 = "structural-d0"
    STRUCTURAL_H0 = "structural-h0"
    SURROGATE_D0 = "surrogate-d0"
    SURROGATE_H0 = "surrogate-h0"
    OFFICIAL_D0 = "official-d0"
    OFFICIAL_H0 = "official-h0"


class LifecycleStage(str, Enum):
    PRE_ACCESS_CONTEXT = "pre-access-context"
    TRAIN_ROLE = "train-role"
    TRAIN_RECONSTRUCTION = "train-reconstruction"
    CANDIDATE_LOAD = "candidate-load"
    FIELD_ENCODING = "field-encoding"
    CANDIDATE_TRAINING = "candidate-training"
    CONTROL_TRAINING = "control-training"
    DEVELOPMENT_ROLE = "development-role"
    METHOD_HOLDOUT_ROLE = "method-holdout-role"
    QUERY_EVALUATION = "query-evaluation"
    METRIC_EVALUATION = "metric-evaluation"
    HARD_GATES = "hard-gates"
    RESOURCE_GATES = "resource-gates"
    TERMINAL_PUBLICATION = "terminal-publication"


class TerminalDecision(str, Enum):
    PREFLIGHT_PASS = "PreflightPass"
    PASS = "Pass"
    METRIC_REJECT = "MetricReject"
    HARD_GATE_REJECT = "HardGateReject"
    RESOURCE_REJECT = "ResourceReject"
    OWNER_FAULT = "OwnerFault"
    CONTRACT_REJECT = "ContractReject"


class CandidateStatus(str, Enum):
    FROZEN_AFTER_PASS = "CandidateFrozenAfterPass"
    REJECTED_EVIDENCE = "RejectedCandidateEvidence"


D0_STAGES = (
    LifecycleStage.PRE_ACCESS_CONTEXT,
    LifecycleStage.TRAIN_ROLE,
    LifecycleStage.FIELD_ENCODING,
    LifecycleStage.CANDIDATE_TRAINING,
    LifecycleStage.CONTROL_TRAINING,
    LifecycleStage.DEVELOPMENT_ROLE,
    LifecycleStage.QUERY_EVALUATION,
    LifecycleStage.METRIC_EVALUATION,
    LifecycleStage.HARD_GATES,
    LifecycleStage.RESOURCE_GATES,
)

H0_STAGES = (
    LifecycleStage.PRE_ACCESS_CONTEXT,
    LifecycleStage.TRAIN_RECONSTRUCTION,
    LifecycleStage.CANDIDATE_LOAD,
    LifecycleStage.METHOD_HOLDOUT_ROLE,
    LifecycleStage.FIELD_ENCODING,
    LifecycleStage.QUERY_EVALUATION,
    LifecycleStage.METRIC_EVALUATION,
    LifecycleStage.HARD_GATES,
    LifecycleStage.RESOURCE_GATES,
)

D0_STRUCTURAL_STAGES = (
    LifecycleStage.PRE_ACCESS_CONTEXT,
    LifecycleStage.TRAIN_ROLE,
    LifecycleStage.FIELD_ENCODING,
    LifecycleStage.DEVELOPMENT_ROLE,
    LifecycleStage.QUERY_EVALUATION,
    LifecycleStage.HARD_GATES,
    LifecycleStage.RESOURCE_GATES,
)

H0_STRUCTURAL_STAGES = (
    LifecycleStage.PRE_ACCESS_CONTEXT,
    LifecycleStage.TRAIN_RECONSTRUCTION,
    LifecycleStage.FIELD_ENCODING,
    LifecycleStage.METHOD_HOLDOUT_ROLE,
    LifecycleStage.QUERY_EVALUATION,
    LifecycleStage.HARD_GATES,
    LifecycleStage.RESOURCE_GATES,
)

STAGE_CALLABLES = {
    LifecycleStage.PRE_ACCESS_CONTEXT: "owner.load-and-validate-context.v1",
    LifecycleStage.TRAIN_ROLE: "provider.materialize-role.v1",
    LifecycleStage.TRAIN_RECONSTRUCTION: "provider.materialize-role.v1",
    LifecycleStage.CANDIDATE_LOAD: "owner.load-frozen-candidate.v1",
    LifecycleStage.FIELD_ENCODING: "owner.encode-canonical-surface-fields.v1",
    LifecycleStage.CANDIDATE_TRAINING: "owner.fit-query-surface-operator.v1",
    LifecycleStage.CONTROL_TRAINING: "owner.fit-frozen-controls.v1",
    LifecycleStage.DEVELOPMENT_ROLE: "provider.materialize-role.v1",
    LifecycleStage.METHOD_HOLDOUT_ROLE: "provider.materialize-role.v1",
    LifecycleStage.QUERY_EVALUATION: "owner.evaluate-surface-queries.v1",
    LifecycleStage.METRIC_EVALUATION: "owner.evaluate-frozen-metrics.v1",
    LifecycleStage.HARD_GATES: "owner.evaluate-hard-gates.v1",
    LifecycleStage.RESOURCE_GATES: "owner.evaluate-resource-gates.v1",
    LifecycleStage.TERMINAL_PUBLICATION: "owner.publish-terminal-atomically.v1",
}

ROLE_STAGE = {
    LifecycleStage.TRAIN_ROLE: RoleKind.TRAIN,
    LifecycleStage.TRAIN_RECONSTRUCTION: RoleKind.TRAIN,
    LifecycleStage.DEVELOPMENT_ROLE: RoleKind.DEVELOPMENT,
    LifecycleStage.METHOD_HOLDOUT_ROLE: RoleKind.METHOD_HOLDOUT,
}


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise ContractError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_sha256(value: str) -> bool:
    return len(value) == 64 and all(
        character in "0123456789abcdef" for character in value
    )


def frozen_float_matrix(rows: tuple[tuple[float, ...], ...]) -> FloatMatrix:
    try:
        matrix = np.array(rows, dtype=np.float64, order="C", copy=True)
    except (TypeError, ValueError, OverflowError) as error:
        raise ContractError("float matrix cannot be materialized") from error
    if matrix.ndim != 2 or matrix.shape[0] == 0 or matrix.shape[1] == 0:
        raise ContractError("float matrix must be nonempty and two-dimensional")
    if not bool(np.all(np.isfinite(matrix))):
        raise ContractError("float matrix contains a non-finite value")
    matrix.flags.writeable = False
    return matrix


def frozen_float_vector(values: tuple[float, ...]) -> FloatVector:
    try:
        vector = np.array(values, dtype=np.float64, order="C", copy=True)
    except (TypeError, ValueError, OverflowError) as error:
        raise ContractError("float vector cannot be materialized") from error
    if vector.ndim != 1 or vector.shape[0] == 0:
        raise ContractError("float vector must be nonempty and one-dimensional")
    if not bool(np.all(np.isfinite(vector))):
        raise ContractError("float vector contains a non-finite value")
    vector.flags.writeable = False
    return vector


def frozen_int_vector(values: tuple[int, ...]) -> IntVector:
    if any(type(value) is not int for value in values):
        raise ContractError("integer vector contains a non-integer value")
    vector = np.array(values, dtype=np.int64, order="C", copy=True)
    if vector.ndim != 1 or vector.shape[0] == 0:
        raise ContractError("integer vector must be nonempty and one-dimensional")
    vector.flags.writeable = False
    return vector


def _array_record(value: FloatMatrix | FloatVector | IntVector) -> dict[str, object]:
    return {
        "bytes": value.nbytes,
        "dtype": value.dtype.str,
        "sha256": sha256_bytes(value.tobytes(order="C")),
        "shape": list(value.shape),
    }


def _validate_float_matrix(
    value: FloatMatrix, rows: int, width: int, label: str
) -> None:
    if (
        value.ndim != 2
        or value.shape != (rows, width)
        or value.dtype != np.dtype(np.float64)
        or value.flags.writeable
        or not value.flags.c_contiguous
        or not bool(np.all(np.isfinite(value)))
    ):
        raise ContractError(f"{label} must be immutable finite float64[{rows},{width}]")


def _validate_float_vector(value: FloatVector, rows: int, label: str) -> None:
    if (
        value.ndim != 1
        or value.shape != (rows,)
        or value.dtype != np.dtype(np.float64)
        or value.flags.writeable
        or not value.flags.c_contiguous
        or not bool(np.all(np.isfinite(value)))
    ):
        raise ContractError(f"{label} must be immutable finite float64[{rows}]")


def _validate_int_vector(value: IntVector, rows: int, label: str) -> None:
    if (
        value.ndim != 1
        or value.shape != (rows,)
        or value.dtype != np.dtype(np.int64)
        or value.flags.writeable
        or not value.flags.c_contiguous
    ):
        raise ContractError(f"{label} must be immutable int64[{rows}]")


def _validate_unit_normals(normals: FloatMatrix, label: str) -> None:
    lengths = np.sqrt(np.sum(normals * normals, axis=1, dtype=np.float64))
    if not bool(np.all(np.abs(lengths - 1.0) <= 1.0e-12)):
        raise ContractError(f"{label} must have unit length")


@dataclass(frozen=True, slots=True)
class ProbeInputV1:
    probe_id: str
    position: tuple[float, float, float]
    normal: tuple[float, float, float]
    area_weight: float
    mode_value: float
    neighbor_ids: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class FieldInputV1:
    field_id: str
    probes: tuple[ProbeInputV1, ...]


@dataclass(frozen=True, slots=True)
class SurfaceFieldSetV1:
    contract_schema: str
    field_ids: tuple[str, ...]
    field_offsets: IntVector
    probe_ids: tuple[str, ...]
    probe_positions: FloatMatrix
    probe_normals: FloatMatrix
    probe_area_weights: FloatVector
    probe_mode_values: FloatVector
    edge_offsets: IntVector
    edge_indices: IntVector

    @property
    def field_count(self) -> int:
        return len(self.field_ids)

    @property
    def probe_count(self) -> int:
        return len(self.probe_ids)

    @property
    def directed_edge_count(self) -> int:
        return int(self.edge_indices.shape[0])

    @property
    def root_sha256(self) -> str:
        return sha256_bytes(
            canonical_json(
                {
                    "contract_schema": self.contract_schema,
                    "edge_indices": _array_record(self.edge_indices),
                    "edge_offsets": _array_record(self.edge_offsets),
                    "field_ids": self.field_ids,
                    "field_offsets": _array_record(self.field_offsets),
                    "probe_area_weights": _array_record(self.probe_area_weights),
                    "probe_ids": self.probe_ids,
                    "probe_mode_values": _array_record(self.probe_mode_values),
                    "probe_normals": _array_record(self.probe_normals),
                    "probe_positions": _array_record(self.probe_positions),
                }
            )
        )

    def __post_init__(self) -> None:
        if self.contract_schema != CONTRACT_SCHEMA:
            raise ContractError("surface field contract schema mismatch")
        if (
            not self.field_ids
            or tuple(sorted(set(self.field_ids))) != self.field_ids
            or not self.probe_ids
            or len(set(self.probe_ids)) != self.probe_count
            or any(not value for value in (*self.field_ids, *self.probe_ids))
        ):
            raise ContractError("field/probe identities must be nonempty canonical IDs")
        _validate_int_vector(self.field_offsets, self.field_count + 1, "field offsets")
        _validate_float_matrix(
            self.probe_positions, self.probe_count, 3, "probe positions"
        )
        _validate_float_matrix(self.probe_normals, self.probe_count, 3, "probe normals")
        _validate_unit_normals(self.probe_normals, "probe normals")
        _validate_float_vector(
            self.probe_area_weights, self.probe_count, "probe area weights"
        )
        _validate_float_vector(
            self.probe_mode_values, self.probe_count, "probe mode values"
        )
        _validate_int_vector(self.edge_offsets, self.probe_count + 1, "edge offsets")
        _validate_int_vector(
            self.edge_indices, self.directed_edge_count, "edge indices"
        )
        if (
            int(self.field_offsets[0]) != 0
            or int(self.field_offsets[-1]) != self.probe_count
            or np.any(np.diff(self.field_offsets) <= 0)
        ):
            raise ContractError("field offsets must partition nonempty fields")
        if (
            int(self.edge_offsets[0]) != 0
            or int(self.edge_offsets[-1]) != self.directed_edge_count
            or np.any(np.diff(self.edge_offsets) <= 0)
        ):
            raise ContractError("edge offsets must partition nonempty adjacency")
        if np.any(self.edge_indices < 0) or np.any(
            self.edge_indices >= self.probe_count
        ):
            raise ContractError("edge index is outside the probe set")
        if np.any(self.probe_area_weights <= 0.0):
            raise ContractError("probe area weights must be positive")

        probe_fields: IntVector = np.empty(self.probe_count, dtype=np.int64)
        for field_index in range(self.field_count):
            start = int(self.field_offsets[field_index])
            end = int(self.field_offsets[field_index + 1])
            if tuple(sorted(self.probe_ids[start:end])) != self.probe_ids[start:end]:
                raise ContractError("probe IDs must be canonical inside each field")
            probe_fields[start:end] = field_index

        adjacency: list[tuple[int, ...]] = []
        for source in range(self.probe_count):
            start = int(self.edge_offsets[source])
            end = int(self.edge_offsets[source + 1])
            neighbors = tuple(int(value) for value in self.edge_indices[start:end])
            if tuple(sorted(set(neighbors))) != neighbors or source in neighbors:
                raise ContractError(
                    "adjacency must be sorted, unique and non-reflexive"
                )
            if any(
                probe_fields[target] != probe_fields[source] for target in neighbors
            ):
                raise ContractError("adjacency must not cross a field boundary")
            adjacency.append(neighbors)
        for source, neighbors in enumerate(adjacency):
            if any(source not in adjacency[target] for target in neighbors):
                raise ContractError("adjacency must be symmetric")
        for field_index in range(self.field_count):
            start = int(self.field_offsets[field_index])
            end = int(self.field_offsets[field_index + 1])
            seen = {start}
            pending = [start]
            while pending:
                source = pending.pop()
                for target in adjacency[source]:
                    if target not in seen:
                        seen.add(target)
                        pending.append(target)
            if seen != set(range(start, end)):
                raise ContractError("every surface field graph must be connected")


def canonical_surface_fields(fields: tuple[FieldInputV1, ...]) -> SurfaceFieldSetV1:
    if not fields:
        raise ContractError("surface field input is empty")
    ordered_fields = tuple(sorted(fields, key=lambda field: field.field_id))
    if any(not field.field_id or not field.probes for field in ordered_fields):
        raise ContractError("surface fields and IDs must be nonempty")
    if len({field.field_id for field in ordered_fields}) != len(ordered_fields):
        raise ContractError("surface field ID is duplicated")

    field_ids: list[str] = []
    field_offsets = [0]
    probe_ids: list[str] = []
    positions: list[tuple[float, float, float]] = []
    normals: list[tuple[float, float, float]] = []
    areas: list[float] = []
    modes: list[float] = []
    neighbor_rows: list[tuple[str, ...]] = []
    field_by_probe: dict[str, str] = {}

    for field in ordered_fields:
        field_ids.append(field.field_id)
        probes = tuple(sorted(field.probes, key=lambda probe: probe.probe_id))
        if len({probe.probe_id for probe in probes}) != len(probes):
            raise ContractError("probe ID is duplicated inside a field")
        for probe in probes:
            if not probe.probe_id or probe.probe_id in field_by_probe:
                raise ContractError("probe IDs must be globally unique and nonempty")
            field_by_probe[probe.probe_id] = field.field_id
            probe_ids.append(probe.probe_id)
            positions.append(probe.position)
            normals.append(probe.normal)
            areas.append(probe.area_weight)
            modes.append(probe.mode_value)
            neighbor_rows.append(tuple(sorted(probe.neighbor_ids)))
        field_offsets.append(len(probe_ids))

    index_by_probe = {probe_id: index for index, probe_id in enumerate(probe_ids)}
    edge_offsets = [0]
    edge_indices: list[int] = []
    for source, neighbors in zip(probe_ids, neighbor_rows, strict=True):
        if not neighbors or len(set(neighbors)) != len(neighbors):
            raise ContractError("probe adjacency is empty or duplicated")
        for neighbor in neighbors:
            if neighbor not in index_by_probe:
                raise ContractError("probe adjacency references an unknown ID")
            if field_by_probe[neighbor] != field_by_probe[source]:
                raise ContractError("probe adjacency crosses a field boundary")
            edge_indices.append(index_by_probe[neighbor])
        edge_offsets.append(len(edge_indices))

    return SurfaceFieldSetV1(
        contract_schema=CONTRACT_SCHEMA,
        field_ids=tuple(field_ids),
        field_offsets=frozen_int_vector(tuple(field_offsets)),
        probe_ids=tuple(probe_ids),
        probe_positions=frozen_float_matrix(tuple(positions)),
        probe_normals=frozen_float_matrix(tuple(normals)),
        probe_area_weights=frozen_float_vector(tuple(areas)),
        probe_mode_values=frozen_float_vector(tuple(modes)),
        edge_offsets=frozen_int_vector(tuple(edge_offsets)),
        edge_indices=frozen_int_vector(tuple(edge_indices)),
    )


@dataclass(frozen=True, slots=True)
class QueryInputV1:
    row_id: str
    field_id: str
    triangle_id: str
    position: tuple[float, float, float]
    normal: tuple[float, float, float]
    barycentrics: tuple[float, float, float]
    context_values: tuple[float, ...]
    p1_contact_value: float


@dataclass(frozen=True, slots=True)
class TargetInputV1:
    row_id: str
    values: tuple[float, float, float]


@dataclass(frozen=True, slots=True)
class SurfaceQueryBatchV1:
    contract_schema: str
    role: RoleKind
    fields: SurfaceFieldSetV1
    row_ids: tuple[str, ...]
    row_field_indices: IntVector
    query_triangle_ids: tuple[str, ...]
    query_positions: FloatMatrix
    query_normals: FloatMatrix
    query_barycentrics: FloatMatrix
    context_feature_names: tuple[str, ...]
    context_features: FloatMatrix
    p1_contact_values: FloatVector
    targets: FloatMatrix | None

    @property
    def row_count(self) -> int:
        return len(self.row_ids)

    @property
    def has_targets(self) -> bool:
        return self.targets is not None

    @property
    def structural_root_sha256(self) -> str:
        return sha256_bytes(
            canonical_json(
                {
                    "context_feature_names": self.context_feature_names,
                    "context_features": _array_record(self.context_features),
                    "contract_schema": self.contract_schema,
                    "field_root_sha256": self.fields.root_sha256,
                    "p1_contact_values": _array_record(self.p1_contact_values),
                    "query_barycentrics": _array_record(self.query_barycentrics),
                    "query_normals": _array_record(self.query_normals),
                    "query_positions": _array_record(self.query_positions),
                    "query_triangle_ids": self.query_triangle_ids,
                    "role": self.role.value,
                    "row_field_indices": _array_record(self.row_field_indices),
                    "row_ids": self.row_ids,
                }
            )
        )

    @property
    def target_root_sha256(self) -> str | None:
        if self.targets is None:
            return None
        return sha256_bytes(canonical_json(_array_record(self.targets)))

    def __post_init__(self) -> None:
        if self.contract_schema != CONTRACT_SCHEMA:
            raise ContractError("surface query contract schema mismatch")
        if (
            not self.row_ids
            or tuple(sorted(set(self.row_ids))) != self.row_ids
            or any(not value for value in self.row_ids)
        ):
            raise ContractError("query row IDs must be nonempty canonical IDs")
        if (
            not self.context_feature_names
            or tuple(sorted(set(self.context_feature_names)))
            != self.context_feature_names
            or any(
                not value or value in FORBIDDEN_CONTEXT_FEATURES
                for value in self.context_feature_names
            )
        ):
            raise ContractError("context feature names are invalid or noncanonical")
        _validate_int_vector(
            self.row_field_indices, self.row_count, "row field indices"
        )
        if np.any(self.row_field_indices < 0) or np.any(
            self.row_field_indices >= self.fields.field_count
        ):
            raise ContractError("query references an unknown surface field")
        if len(self.query_triangle_ids) != self.row_count or any(
            not value for value in self.query_triangle_ids
        ):
            raise ContractError("query triangle identity count mismatch")
        _validate_float_matrix(
            self.query_positions, self.row_count, 3, "query positions"
        )
        _validate_float_matrix(self.query_normals, self.row_count, 3, "query normals")
        _validate_unit_normals(self.query_normals, "query normals")
        _validate_float_matrix(
            self.query_barycentrics, self.row_count, 3, "query barycentrics"
        )
        if np.any(self.query_barycentrics < 0.0) or np.any(
            self.query_barycentrics > 1.0
        ):
            raise ContractError("query barycentrics must be inside the triangle")
        sums = np.sum(self.query_barycentrics, axis=1, dtype=np.float64)
        if not bool(np.all(np.abs(sums - 1.0) <= 1.0e-12)):
            raise ContractError("query barycentrics must sum to one")
        _validate_float_matrix(
            self.context_features,
            self.row_count,
            len(self.context_feature_names),
            "context features",
        )
        _validate_float_vector(
            self.p1_contact_values, self.row_count, "P1 contact values"
        )
        if self.targets is not None:
            _validate_float_matrix(
                self.targets, self.row_count, TARGET_AXIS_COUNT, "targets"
            )


def canonical_query_batch(
    role: RoleKind,
    fields: SurfaceFieldSetV1,
    context_feature_names: tuple[str, ...],
    queries: tuple[QueryInputV1, ...],
    targets: tuple[TargetInputV1, ...] | None = None,
) -> SurfaceQueryBatchV1:
    if not queries:
        raise ContractError("surface query input is empty")
    if not context_feature_names or len(set(context_feature_names)) != len(
        context_feature_names
    ):
        raise ContractError("context feature names must be nonempty and unique")
    names = tuple(sorted(context_feature_names))
    source_column = {name: index for index, name in enumerate(context_feature_names)}
    canonical_columns = tuple(source_column[name] for name in names)
    ordered = tuple(sorted(queries, key=lambda query: query.row_id))
    if len({query.row_id for query in ordered}) != len(ordered):
        raise ContractError("surface query row ID is duplicated")
    target_by_row: Mapping[str, tuple[float, float, float]] | None = None
    if targets is not None:
        materialized = {target.row_id: target.values for target in targets}
        if len(materialized) != len(targets) or set(materialized) != {
            query.row_id for query in ordered
        }:
            raise ContractError("target identities must match query rows exactly")
        target_by_row = materialized
    field_indices = {value: index for index, value in enumerate(fields.field_ids)}
    if any(len(query.context_values) != len(names) for query in ordered):
        raise ContractError("query context width mismatch")
    try:
        row_field_indices = tuple(field_indices[query.field_id] for query in ordered)
    except KeyError as error:
        raise ContractError("query references an unknown surface field") from error
    target_matrix = (
        None
        if target_by_row is None
        else frozen_float_matrix(
            tuple(target_by_row[query.row_id] for query in ordered)
        )
    )
    return SurfaceQueryBatchV1(
        contract_schema=CONTRACT_SCHEMA,
        role=role,
        fields=fields,
        row_ids=tuple(query.row_id for query in ordered),
        row_field_indices=frozen_int_vector(row_field_indices),
        query_triangle_ids=tuple(query.triangle_id for query in ordered),
        query_positions=frozen_float_matrix(tuple(query.position for query in ordered)),
        query_normals=frozen_float_matrix(tuple(query.normal for query in ordered)),
        query_barycentrics=frozen_float_matrix(
            tuple(query.barycentrics for query in ordered)
        ),
        context_feature_names=names,
        context_features=frozen_float_matrix(
            tuple(
                tuple(query.context_values[index] for index in canonical_columns)
                for query in ordered
            )
        ),
        p1_contact_values=frozen_float_vector(
            tuple(query.p1_contact_value for query in ordered)
        ),
        targets=target_matrix,
    )


def pipeline_for_provider(kind: ProviderKind) -> PipelineKind:
    match kind:
        case (
            ProviderKind.STRUCTURAL_D0
            | ProviderKind.SURROGATE_D0
            | ProviderKind.OFFICIAL_D0
        ):
            return PipelineKind.D0
        case (
            ProviderKind.STRUCTURAL_H0
            | ProviderKind.SURROGATE_H0
            | ProviderKind.OFFICIAL_H0
        ):
            return PipelineKind.H0
    assert_never(kind)


def provider_is_structural(kind: ProviderKind) -> bool:
    return kind in (ProviderKind.STRUCTURAL_D0, ProviderKind.STRUCTURAL_H0)


def provider_is_official(kind: ProviderKind) -> bool:
    return kind in (ProviderKind.OFFICIAL_D0, ProviderKind.OFFICIAL_H0)


def allowed_roles(kind: ProviderKind) -> tuple[RoleKind, ...]:
    if pipeline_for_provider(kind) is PipelineKind.D0:
        return (RoleKind.TRAIN, RoleKind.DEVELOPMENT)
    return (RoleKind.TRAIN, RoleKind.METHOD_HOLDOUT)


def stages_for_pipeline(pipeline: PipelineKind) -> tuple[LifecycleStage, ...]:
    match pipeline:
        case PipelineKind.D0:
            return D0_STAGES
        case PipelineKind.H0:
            return H0_STAGES
    assert_never(pipeline)


def stages_for_provider(kind: ProviderKind) -> tuple[LifecycleStage, ...]:
    if provider_is_structural(kind):
        return (
            D0_STRUCTURAL_STAGES
            if pipeline_for_provider(kind) is PipelineKind.D0
            else H0_STRUCTURAL_STAGES
        )
    return stages_for_pipeline(pipeline_for_provider(kind))


@dataclass(frozen=True, slots=True)
class AccessCapabilityV1:
    provider_kind: ProviderKind
    namespace: str
    allowed_roles: tuple[RoleKind, ...]
    execution_seal_sha256: str | None

    @classmethod
    def structural(
        cls, provider_kind: ProviderKind, namespace: str
    ) -> AccessCapabilityV1:
        if not provider_is_structural(provider_kind):
            raise ContractError("structural capability requires a structural provider")
        return cls(provider_kind, namespace, allowed_roles(provider_kind), None)

    @classmethod
    def surrogate(
        cls, provider_kind: ProviderKind, namespace: str
    ) -> AccessCapabilityV1:
        if provider_kind not in (
            ProviderKind.SURROGATE_D0,
            ProviderKind.SURROGATE_H0,
        ):
            raise ContractError("surrogate capability requires a surrogate provider")
        return cls(provider_kind, namespace, allowed_roles(provider_kind), None)

    @classmethod
    def official(
        cls, provider_kind: ProviderKind, namespace: str, execution_seal_sha256: str
    ) -> AccessCapabilityV1:
        if not provider_is_official(provider_kind):
            raise ContractError("official capability requires an official provider")
        return cls(
            provider_kind,
            namespace,
            allowed_roles(provider_kind),
            execution_seal_sha256,
        )

    def __post_init__(self) -> None:
        unique_roles = tuple(
            sorted(set(self.allowed_roles), key=lambda role: role.value)
        )
        expected_roles = tuple(
            sorted(allowed_roles(self.provider_kind), key=lambda role: role.value)
        )
        if (
            not self.namespace
            or len(unique_roles) != len(self.allowed_roles)
            or unique_roles != expected_roles
        ):
            raise ContractError("capability namespace or role set is invalid")
        if provider_is_structural(self.provider_kind):
            valid = self.namespace.startswith("structural-")
        elif provider_is_official(self.provider_kind):
            valid = self.namespace.startswith("v37-") and bool(
                self.execution_seal_sha256 and is_sha256(self.execution_seal_sha256)
            )
        else:
            valid = self.namespace.startswith("discarded-")
        if not valid:
            raise ContractError("capability namespace or seal is invalid")
        if not provider_is_official(self.provider_kind) and (
            self.execution_seal_sha256 is not None
        ):
            raise ContractError("only an official capability may carry a seal")


class SurfaceRoleProviderV1(Protocol):
    @property
    def kind(self) -> ProviderKind: ...

    @property
    def namespace(self) -> str: ...

    def materialize(
        self, role: RoleKind, capability: AccessCapabilityV1
    ) -> SurfaceQueryBatchV1: ...


@dataclass(frozen=True, slots=True)
class AccessLedgerV1:
    provider_calls: int = 0
    structural_rows: int = 0
    train_target_rows: int = 0
    development_target_rows: int = 0
    method_holdout_target_rows: int = 0
    official_d0_target_rows: int = 0
    official_h0_target_rows: int = 0
    forbidden_access_count: int = 0

    @property
    def target_rows_accessed(self) -> int:
        return (
            self.train_target_rows
            + self.development_target_rows
            + self.method_holdout_target_rows
        )

    def record(
        self, provider_kind: ProviderKind, role: RoleKind, row_count: int
    ) -> AccessLedgerV1:
        if row_count <= 0:
            raise ContractError("provider returned an empty role")
        if provider_is_structural(provider_kind):
            return AccessLedgerV1(
                provider_calls=self.provider_calls + 1,
                structural_rows=self.structural_rows + row_count,
                forbidden_access_count=self.forbidden_access_count,
            )
        train = self.train_target_rows + (row_count if role is RoleKind.TRAIN else 0)
        development = self.development_target_rows + (
            row_count if role is RoleKind.DEVELOPMENT else 0
        )
        holdout = self.method_holdout_target_rows + (
            row_count if role is RoleKind.METHOD_HOLDOUT else 0
        )
        official_d0 = self.official_d0_target_rows + (
            row_count if provider_kind is ProviderKind.OFFICIAL_D0 else 0
        )
        official_h0 = self.official_h0_target_rows + (
            row_count if provider_kind is ProviderKind.OFFICIAL_H0 else 0
        )
        return AccessLedgerV1(
            provider_calls=self.provider_calls + 1,
            structural_rows=self.structural_rows,
            train_target_rows=train,
            development_target_rows=development,
            method_holdout_target_rows=holdout,
            official_d0_target_rows=official_d0,
            official_h0_target_rows=official_h0,
            forbidden_access_count=self.forbidden_access_count,
        )


@dataclass(frozen=True, slots=True)
class TraceEventV1:
    ordinal: int
    stage: LifecycleStage
    callable_id: str

    def topology_line(self) -> str:
        return f"{self.ordinal:02d}|{self.stage.value}|{self.callable_id}\n"


@dataclass(frozen=True, slots=True)
class ExecutionTraceV1:
    pipeline: PipelineKind
    provider_kind: ProviderKind
    events: tuple[TraceEventV1, ...]
    terminal: TerminalDecision
    access: AccessLedgerV1

    @property
    def topology_sha256(self) -> str:
        return sha256_bytes(
            "".join(event.topology_line() for event in self.events).encode()
        )


class OwnerLifecycleV1:
    def __init__(self, pipeline: PipelineKind, capability: AccessCapabilityV1) -> None:
        if pipeline_for_provider(capability.provider_kind) is not pipeline:
            raise ContractError("capability does not match owner pipeline")
        self._pipeline = pipeline
        self._capability = capability
        self._stages = stages_for_provider(capability.provider_kind)
        self._cursor = 0
        self._events: list[TraceEventV1] = []
        self._opened_roles: set[RoleKind] = set()
        self._terminal: TerminalDecision | None = None
        self._ledger = AccessLedgerV1()

    @property
    def access(self) -> AccessLedgerV1:
        return self._ledger

    def _enter(self, stage: LifecycleStage) -> None:
        if self._terminal is not None or self._cursor >= len(self._stages):
            raise ContractError("owner lifecycle is already terminal or complete")
        expected = self._stages[self._cursor]
        if stage is not expected:
            raise ContractError(
                f"lifecycle stage mismatch: expected {expected.value}, got {stage.value}"
            )
        self._events.append(
            TraceEventV1(len(self._events), stage, STAGE_CALLABLES[stage])
        )
        self._cursor += 1

    def step(self, stage: LifecycleStage) -> None:
        if stage in ROLE_STAGE:
            raise ContractError("role stage must use materialize_role")
        self._enter(stage)

    def materialize_role(
        self,
        stage: LifecycleStage,
        role: RoleKind,
        provider: SurfaceRoleProviderV1,
    ) -> SurfaceQueryBatchV1:
        if ROLE_STAGE.get(stage) is not role or role in self._opened_roles:
            raise ContractError("role stage is invalid or already opened")
        if role not in self._capability.allowed_roles:
            raise ContractError("role is outside the capability")
        if (
            provider.kind is not self._capability.provider_kind
            or provider.namespace != self._capability.namespace
        ):
            raise ContractError("provider identity does not match capability")
        self._enter(stage)
        batch = provider.materialize(role, self._capability)
        self._ledger = self._ledger.record(provider.kind, role, batch.row_count)
        if batch.contract_schema != CONTRACT_SCHEMA or batch.role is not role:
            raise ContractError("provider returned the wrong surface query role")
        if provider_is_structural(provider.kind) == batch.has_targets:
            raise ContractError("provider target state does not match its authority")
        self._opened_roles.add(role)
        return batch

    def finish(self, decision: TerminalDecision) -> ExecutionTraceV1:
        if self._terminal is not None:
            raise ContractError("terminal decision already published")
        complete = self._cursor == len(self._stages)
        structural = provider_is_structural(self._capability.provider_kind)
        scientific = decision in (
            TerminalDecision.PASS,
            TerminalDecision.METRIC_REJECT,
            TerminalDecision.HARD_GATE_REJECT,
            TerminalDecision.RESOURCE_REJECT,
        )
        if decision is TerminalDecision.PREFLIGHT_PASS and (
            not structural or not complete or self._ledger.target_rows_accessed != 0
        ):
            raise ContractError("preflight pass requires a complete structural path")
        if scientific and (
            structural or not complete or self._ledger.target_rows_accessed == 0
        ):
            raise ContractError("scientific terminal requires a complete target path")
        if decision is TerminalDecision.CONTRACT_REJECT and (
            self._ledger.target_rows_accessed != 0
        ):
            raise ContractError("contract reject is pre-target only")
        if decision is TerminalDecision.OWNER_FAULT and (
            self._ledger.target_rows_accessed == 0
        ):
            raise ContractError("owner fault is post-target only")
        self._events.append(
            TraceEventV1(
                len(self._events),
                LifecycleStage.TERMINAL_PUBLICATION,
                STAGE_CALLABLES[LifecycleStage.TERMINAL_PUBLICATION],
            )
        )
        self._terminal = decision
        return ExecutionTraceV1(
            self._pipeline,
            self._capability.provider_kind,
            tuple(self._events),
            decision,
            self._ledger,
        )


def expected_topology_sha256(provider_kind: ProviderKind) -> str:
    stages = (*stages_for_provider(provider_kind), LifecycleStage.TERMINAL_PUBLICATION)
    return sha256_bytes(
        "".join(
            f"{index:02d}|{stage.value}|{STAGE_CALLABLES[stage]}\n"
            for index, stage in enumerate(stages)
        ).encode()
    )


def assert_complete_trace(trace: ExecutionTraceV1) -> None:
    expected_stages = (
        *stages_for_provider(trace.provider_kind),
        LifecycleStage.TERMINAL_PUBLICATION,
    )
    if tuple(event.stage for event in trace.events) != expected_stages:
        raise ContractError("trace stage sequence is incomplete or mutated")
    if any(
        event.ordinal != index or event.callable_id != STAGE_CALLABLES[event.stage]
        for index, event in enumerate(trace.events)
    ):
        raise ContractError("trace ordinal or callable identity is mutated")
    if trace.topology_sha256 != expected_topology_sha256(trace.provider_kind):
        raise ContractError("trace topology hash mismatch")


@dataclass(frozen=True, slots=True)
class CandidateDispositionV1:
    decision: TerminalDecision
    candidate_bundle_authority: bool
    freeze_document: bytes | None
    rejected_evidence_document: bytes | None

    def __post_init__(self) -> None:
        if self.decision is TerminalDecision.PASS:
            valid = (
                self.candidate_bundle_authority
                and self.freeze_document is not None
                and self.rejected_evidence_document is None
            )
        elif self.decision in (
            TerminalDecision.METRIC_REJECT,
            TerminalDecision.HARD_GATE_REJECT,
            TerminalDecision.RESOURCE_REJECT,
        ):
            valid = (
                not self.candidate_bundle_authority
                and self.freeze_document is None
                and self.rejected_evidence_document is not None
            )
        else:
            valid = (
                not self.candidate_bundle_authority
                and self.freeze_document is None
                and self.rejected_evidence_document is None
            )
        if not valid:
            raise ContractError("candidate disposition contradicts terminal decision")


def candidate_disposition(
    decision: TerminalDecision,
    *,
    owner_sha256: str,
    profile_sha256: str,
    terminal_sha256: str,
    candidate_weights_sha256: str | None,
) -> CandidateDispositionV1:
    identities = (owner_sha256, profile_sha256, terminal_sha256)
    if not all(is_sha256(value) for value in identities):
        raise ContractError("candidate disposition identity is invalid")
    common: dict[str, object] = {
        "candidate_weights_sha256": candidate_weights_sha256,
        "contract_schema": CONTRACT_SCHEMA,
        "decision": decision.value,
        "owner_sha256": owner_sha256,
        "profile_sha256": profile_sha256,
        "terminal_sha256": terminal_sha256,
    }
    if decision is TerminalDecision.PASS:
        if candidate_weights_sha256 is None or not is_sha256(candidate_weights_sha256):
            raise ContractError("Pass candidate requires exact weight identity")
        common["status"] = CandidateStatus.FROZEN_AFTER_PASS.value
        return CandidateDispositionV1(decision, True, canonical_json(common), None)
    if decision in (
        TerminalDecision.METRIC_REJECT,
        TerminalDecision.HARD_GATE_REJECT,
        TerminalDecision.RESOURCE_REJECT,
    ):
        if candidate_weights_sha256 is None or not is_sha256(candidate_weights_sha256):
            raise ContractError("scientific reject evidence requires weight identity")
        common["status"] = CandidateStatus.REJECTED_EVIDENCE.value
        return CandidateDispositionV1(decision, False, None, canonical_json(common))
    if candidate_weights_sha256 is not None:
        raise ContractError(
            "fault/preflight disposition cannot carry candidate weights"
        )
    return CandidateDispositionV1(decision, False, None, None)


def _load_canonical_document(data: bytes, label: str) -> dict[str, object]:
    try:
        parsed = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise ContractError(f"{label} is invalid JSON") from error
    if not isinstance(parsed, dict) or canonical_json(parsed) != data:
        raise ContractError(f"{label} is not a canonical object")
    return cast(dict[str, object], parsed)


def validate_h0_candidate_freeze(
    disposition: CandidateDispositionV1,
    *,
    expected_owner_sha256: str,
    expected_profile_sha256: str,
    expected_terminal_sha256: str,
    expected_weights_sha256: str,
) -> dict[str, object]:
    expected_identities = (
        expected_owner_sha256,
        expected_profile_sha256,
        expected_terminal_sha256,
        expected_weights_sha256,
    )
    if not all(is_sha256(value) for value in expected_identities):
        raise ContractError("H0 candidate expectation identity is invalid")
    if (
        disposition.decision is not TerminalDecision.PASS
        or not disposition.candidate_bundle_authority
        or disposition.freeze_document is None
        or disposition.rejected_evidence_document is not None
    ):
        raise ContractError("H0 requires an authoritative D0 Pass freeze")
    document = _load_canonical_document(disposition.freeze_document, "candidate freeze")
    if document != {
        "candidate_weights_sha256": expected_weights_sha256,
        "contract_schema": CONTRACT_SCHEMA,
        "decision": TerminalDecision.PASS.value,
        "owner_sha256": expected_owner_sha256,
        "profile_sha256": expected_profile_sha256,
        "status": CandidateStatus.FROZEN_AFTER_PASS.value,
        "terminal_sha256": expected_terminal_sha256,
    }:
        raise ContractError("candidate freeze identity or Pass authority mismatch")
    return document


def trace_record(trace: ExecutionTraceV1) -> dict[str, object]:
    return {
        "access": {
            "development_target_rows": trace.access.development_target_rows,
            "forbidden_access_count": trace.access.forbidden_access_count,
            "method_holdout_target_rows": trace.access.method_holdout_target_rows,
            "official_d0_target_rows": trace.access.official_d0_target_rows,
            "official_h0_target_rows": trace.access.official_h0_target_rows,
            "provider_calls": trace.access.provider_calls,
            "structural_rows": trace.access.structural_rows,
            "target_rows_accessed": trace.access.target_rows_accessed,
            "train_target_rows": trace.access.train_target_rows,
        },
        "events": [
            {
                "callable_id": event.callable_id,
                "ordinal": event.ordinal,
                "stage": event.stage.value,
            }
            for event in trace.events
        ],
        "pipeline": trace.pipeline.value,
        "provider_kind": trace.provider_kind.value,
        "terminal": trace.terminal.value,
        "topology_sha256": trace.topology_sha256,
    }
