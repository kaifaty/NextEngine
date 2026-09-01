#!/usr/bin/env python3
"""Official composite-coverage runner for Physical Sound V19 C0."""

from __future__ import annotations

import argparse
import math
import sys
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v19_c0_common as common
from scipy.sparse import csr_matrix
from scipy.sparse.csgraph import dijkstra
from scipy.spatial.distance import cdist

NUMERICAL_CASE_ORDER = (
    "intrinsic-cap",
    "component-isolation",
    "thinning",
    "ambient-shortcut",
)
STRUCTURAL_CASE_ORDER = (
    "duplicate-context",
    "out-of-range",
    "minimum-tamper",
    "identity-mismatch",
)
METHOD_ORDER = (
    "composite",
    "intrinsic-local-only",
    "euclidean-local-only",
    "graph-only",
    "structural-only",
)
REJECT_REASONS = {
    "OOD_CONTEXT_BUDGET",
    "OOD_DISCONNECTED",
    "OOD_INTRINSIC_FILL",
    "OOD_INTRINSIC_GAP",
    "CONTROL_INPUT_INVALID",
}


@dataclass(frozen=True)
class MeshAnalysis:
    mesh: common.Mesh
    graph: csr_matrix
    graph_hash: str
    all_pairs: np.ndarray
    graph_diameter: float
    euclidean_distances: np.ndarray
    euclidean_diameter: float
    context: np.ndarray
    query: np.ndarray
    lexicographic_rank: np.ndarray


@dataclass(frozen=True)
class CoverageInput:
    schema: str
    revision: str
    role: str
    object_id: str
    topology: str
    case: str
    case_group: str
    mesh_hash: str
    row_hash: str
    graph_hash: str
    vertex_count: int
    edge_count: int
    declared_k: int
    declared_context_count: int
    declared_query_count: int
    context_hash: str
    query_hash: str
    context: np.ndarray
    query: np.ndarray
    graph: csr_matrix

    def record(self) -> dict[str, Any]:
        return {
            "case": self.case,
            "case_group": self.case_group,
            "context_count": int(self.context.size),
            "context_hash": self.context_hash,
            "declared_context_count": self.declared_context_count,
            "declared_k": self.declared_k,
            "declared_query_count": self.declared_query_count,
            "edge_count": self.edge_count,
            "graph_hash": self.graph_hash,
            "mesh_hash": self.mesh_hash,
            "object_id": self.object_id,
            "query_count": int(self.query.size),
            "query_hash": self.query_hash,
            "revision": self.revision,
            "role": self.role,
            "row_hash": self.row_hash,
            "schema": self.schema,
            "topology": self.topology,
            "vertex_count": self.vertex_count,
        }


def graph_matrix(mesh: common.Mesh, edge_mask: np.ndarray | None = None) -> csr_matrix:
    if edge_mask is None:
        edge_mask = np.ones(mesh.edges.shape[0], dtype=bool)
    mask = np.asarray(edge_mask, dtype=bool)
    if mask.shape != (mesh.edges.shape[0],):
        raise common.C0Error("C0 edge mask shape changed")
    edges = mesh.edges[mask]
    weights = mesh.edge_lengths[mask]
    if edges.size == 0:
        raise common.C0Error("C0 graph has no edges")
    row = np.concatenate((edges[:, 0], edges[:, 1]))
    column = np.concatenate((edges[:, 1], edges[:, 0]))
    values = np.concatenate((weights, weights))
    graph = csr_matrix(
        (values, (row, column)),
        shape=(mesh.vertex_count, mesh.vertex_count),
        dtype=np.float64,
    )
    graph.sort_indices()
    return graph


def graph_identity(graph: csr_matrix) -> str:
    canonical = graph.copy()
    canonical.sort_indices()
    digest = common.sha256_bytes(
        common.array_bytes(np.asarray(canonical.indptr, dtype=np.int64))
        + common.array_bytes(np.asarray(canonical.indices, dtype=np.int64))
        + common.array_bytes(np.asarray(canonical.data, dtype=np.float64))
    )
    return digest


def _lexicographic_rank(vertices: np.ndarray) -> np.ndarray:
    indices = np.arange(vertices.shape[0], dtype=np.int64)
    order = np.lexsort((indices, vertices[:, 2], vertices[:, 1], vertices[:, 0]))
    rank = np.empty_like(order)
    rank[order] = np.arange(order.size, dtype=np.int64)
    return rank


def _ordered_by_distance(
    distances: np.ndarray, rank: np.ndarray, descending: bool
) -> np.ndarray:
    values = np.asarray(distances, dtype=np.float64)
    if values.shape != rank.shape or np.isnan(values).any():
        raise common.C0Error("C0 distance ordering input changed")
    primary = -values if descending else values
    return np.lexsort((rank, primary))


def farthest_point_context(
    all_pairs: np.ndarray, vertices: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    vertex_count = vertices.shape[0]
    expected = common.required_context_count(vertex_count)
    if all_pairs.shape != (vertex_count, vertex_count):
        raise common.C0Error("C0 all-pairs shape changed")
    if not np.isfinite(all_pairs).all():
        raise common.C0Error("valid C0 mesh is disconnected")
    rank = _lexicographic_rank(vertices)
    allowed = np.arange(vertex_count, dtype=np.int64)
    context = _restricted_fps(all_pairs, rank, allowed, expected)
    selected = np.zeros(vertex_count, dtype=bool)
    selected[context] = True
    return context, np.flatnonzero(~selected).astype(np.int64)


def _restricted_fps(
    all_pairs: np.ndarray,
    rank: np.ndarray,
    allowed: np.ndarray,
    count: int,
) -> np.ndarray:
    candidates_allowed = np.asarray(allowed, dtype=np.int64)
    if (
        candidates_allowed.ndim != 1
        or candidates_allowed.size < count
        or count <= 0
        or len(set(candidates_allowed.tolist())) != candidates_allowed.size
    ):
        raise common.C0Error("C0 restricted FPS domain changed")
    seed = int(candidates_allowed[np.argmin(rank[candidates_allowed])])
    selected = [seed]
    selected_mask = np.zeros(all_pairs.shape[0], dtype=bool)
    selected_mask[seed] = True
    allowed_mask = np.zeros_like(selected_mask)
    allowed_mask[candidates_allowed] = True
    nearest = all_pairs[seed].copy()
    while len(selected) < count:
        score = nearest.copy()
        score[~allowed_mask | selected_mask] = -np.inf
        maximum = np.max(score)
        tied = np.flatnonzero(score == maximum)
        if tied.size == 0 or not np.isfinite(maximum):
            raise common.C0Error("C0 restricted FPS failed")
        candidate = int(tied[np.argmin(rank[tied])])
        selected.append(candidate)
        selected_mask[candidate] = True
        nearest = np.minimum(nearest, all_pairs[candidate])
    return np.asarray(selected, dtype=np.int64)


def analyze_mesh(mesh: common.Mesh) -> MeshAnalysis:
    graph = graph_matrix(mesh)
    all_pairs = np.asarray(dijkstra(graph, directed=False), dtype=np.float64)
    if all_pairs.shape != (mesh.vertex_count, mesh.vertex_count):
        raise common.C0Error("C0 Dijkstra shape changed")
    if not np.isfinite(all_pairs).all():
        raise common.C0Error("valid C0 graph is disconnected")
    graph_diameter = float(np.max(all_pairs))
    if graph_diameter <= 0.0:
        raise common.C0Error("C0 graph diameter is non-positive")
    euclidean = cdist(mesh.vertices, mesh.vertices, metric="euclidean")
    euclidean_diameter = float(np.max(euclidean))
    if not np.isfinite(euclidean).all() or euclidean_diameter <= 0.0:
        raise common.C0Error("C0 Euclidean geometry is invalid")
    context, query = farthest_point_context(all_pairs, mesh.vertices)
    return MeshAnalysis(
        mesh=mesh,
        graph=graph,
        graph_hash=graph_identity(graph),
        all_pairs=all_pairs,
        graph_diameter=graph_diameter,
        euclidean_distances=euclidean,
        euclidean_diameter=euclidean_diameter,
        context=context,
        query=query,
        lexicographic_rank=_lexicographic_rank(mesh.vertices),
    )


def _multi_source_distance(graph: csr_matrix, context: np.ndarray) -> np.ndarray:
    selected = np.asarray(context, dtype=np.int64)
    result = np.asarray(
        dijkstra(graph, directed=False, indices=selected, min_only=True),
        dtype=np.float64,
    )
    if result.shape != (graph.shape[0],) or np.isnan(result).any():
        raise common.C0Error("C0 multi-source Dijkstra result changed")
    return result


def _coverage_input(
    analysis: MeshAnalysis,
    case: str,
    case_group: str,
    context: np.ndarray,
    query: np.ndarray,
    graph: csr_matrix,
) -> CoverageInput:
    selected = np.asarray(context, dtype=np.int64)
    requested = np.asarray(query, dtype=np.int64)
    return CoverageInput(
        schema="CoverageInputV0",
        revision=common.REVISION,
        role=analysis.mesh.row.role,
        object_id=analysis.mesh.row.object_id,
        topology=analysis.mesh.row.topology,
        case=case,
        case_group=case_group,
        mesh_hash=analysis.mesh.mesh_hash,
        row_hash=common.identity_hash(analysis.mesh.row.record()),
        graph_hash=graph_identity(graph),
        vertex_count=analysis.mesh.vertex_count,
        edge_count=int(graph.nnz // 2),
        declared_k=common.required_context_count(analysis.mesh.vertex_count),
        declared_context_count=int(selected.size),
        declared_query_count=int(requested.size),
        context_hash=common.identity_hash(selected),
        query_hash=common.identity_hash(requested),
        context=selected,
        query=requested,
        graph=graph,
    )


def valid_input(analysis: MeshAnalysis) -> CoverageInput:
    return _coverage_input(
        analysis,
        "valid",
        "valid",
        analysis.context,
        analysis.query,
        analysis.graph,
    )


def numerical_input(analysis: MeshAnalysis, mutation: str) -> CoverageInput:
    mesh = analysis.mesh
    vertex_count = mesh.vertex_count
    if mutation == "intrinsic-cap":
        seed = int(np.argmin(analysis.lexicographic_rank))
        distance = analysis.all_pairs[seed]
        nearest = _ordered_by_distance(
            distance, analysis.lexicographic_rank, descending=False
        )
        farthest = _ordered_by_distance(
            distance, analysis.lexicographic_rank, descending=True
        )
        context = nearest[: math.ceil(0.40 * vertex_count)].astype(np.int64)
        query = farthest[: math.ceil(0.30 * vertex_count)].astype(np.int64)
        graph = analysis.graph
    elif mutation == "component-isolation":
        row_index = np.arange(vertex_count, dtype=np.int64) // mesh.row.grid_u
        lower = row_index < mesh.row.grid_v // 2
        seed = int(np.argmin(analysis.lexicographic_rank))
        seed_side = lower[seed]
        allowed = np.flatnonzero(lower == seed_side).astype(np.int64)
        context = _restricted_fps(
            analysis.all_pairs,
            analysis.lexicographic_rank,
            allowed,
            common.required_context_count(vertex_count),
        )
        query = np.flatnonzero(lower != seed_side).astype(np.int64)
        edge_mask = lower[mesh.edges[:, 0]] == lower[mesh.edges[:, 1]]
        graph = graph_matrix(mesh, edge_mask)
    elif mutation == "thinning":
        context = analysis.context[::4]
        distance = _multi_source_distance(analysis.graph, context)
        order = _ordered_by_distance(
            distance, analysis.lexicographic_rank, descending=True
        )
        query = order[: math.ceil(0.25 * vertex_count)].astype(np.int64)
        graph = analysis.graph
    elif mutation == "ambient-shortcut":
        if mesh.row.topology != "RolledSheet":
            return replace(
                numerical_input(analysis, "intrinsic-cap"),
                case=mutation,
            )
        context = np.flatnonzero(mesh.uv[:, 0] <= -0.75).astype(np.int64)
        query = np.flatnonzero(mesh.uv[:, 0] >= 0.75).astype(np.int64)
        graph = analysis.graph
    else:
        raise common.C0Error(f"unknown C0 numerical mutation: {mutation}")
    if set(context.tolist()) & set(query.tolist()):
        raise common.C0Error(f"C0 mutation overlaps context/query: {mutation}")
    return _coverage_input(
        analysis, mutation, "numerical-mutation", context, query, graph
    )


def structural_input(analysis: MeshAnalysis, mutation: str) -> CoverageInput:
    original = valid_input(analysis)
    if mutation == "duplicate-context":
        context = original.context.copy()
        context[-1] = context[0]
        return replace(
            original,
            case=mutation,
            case_group="structural-mutation",
            context=context,
            context_hash=common.identity_hash(context),
        )
    if mutation == "out-of-range":
        context = original.context.copy()
        context[-1] = analysis.mesh.vertex_count
        return replace(
            original,
            case=mutation,
            case_group="structural-mutation",
            context=context,
            context_hash=common.identity_hash(context),
        )
    if mutation == "minimum-tamper":
        return replace(
            original,
            case=mutation,
            case_group="structural-mutation",
            declared_k=original.declared_k - 1,
        )
    if mutation == "identity-mismatch":
        return replace(
            original,
            case=mutation,
            case_group="structural-mutation",
            mesh_hash="0" * 64,
        )
    raise common.C0Error(f"unknown C0 structural mutation: {mutation}")


def all_inputs(analysis: MeshAnalysis) -> tuple[CoverageInput, ...]:
    return (
        valid_input(analysis),
        *(numerical_input(analysis, case) for case in NUMERICAL_CASE_ORDER),
        *(structural_input(analysis, case) for case in STRUCTURAL_CASE_ORDER),
    )


def structural_failure(analysis: MeshAnalysis, value: CoverageInput) -> str | None:
    if value.schema != "CoverageInputV0" or value.revision != common.REVISION:
        return "SCHEMA_OR_REVISION"
    if (
        value.role != analysis.mesh.row.role
        or value.object_id != analysis.mesh.row.object_id
    ):
        return "OBJECT_IDENTITY"
    if value.topology != analysis.mesh.row.topology:
        return "TOPOLOGY_IDENTITY"
    if value.mesh_hash != analysis.mesh.mesh_hash:
        return "MESH_IDENTITY"
    if value.row_hash != common.identity_hash(analysis.mesh.row.record()):
        return "ROW_IDENTITY"
    if value.vertex_count != analysis.mesh.vertex_count:
        return "VERTEX_COUNT"
    if value.edge_count != value.graph.nnz // 2:
        return "EDGE_COUNT"
    if value.graph_hash != graph_identity(value.graph):
        return "GRAPH_IDENTITY"
    required = common.required_context_count(analysis.mesh.vertex_count)
    if value.declared_k != required:
        return "DECLARED_MINIMUM"
    if value.declared_context_count != value.context.size:
        return "DECLARED_CONTEXT_COUNT"
    if value.declared_query_count != value.query.size:
        return "DECLARED_QUERY_COUNT"
    if value.context.ndim != 1 or value.query.ndim != 1:
        return "INDEX_RANK"
    if value.context.size == 0 or value.query.size == 0:
        return "EMPTY_INDEX_SET"
    if (
        np.any(value.context < 0)
        or np.any(value.context >= analysis.mesh.vertex_count)
        or np.any(value.query < 0)
        or np.any(value.query >= analysis.mesh.vertex_count)
    ):
        return "OUT_OF_RANGE"
    if len(set(value.context.tolist())) != value.context.size:
        return "DUPLICATE_CONTEXT"
    if len(set(value.query.tolist())) != value.query.size:
        return "DUPLICATE_QUERY"
    if value.context_hash != common.identity_hash(value.context):
        return "CONTEXT_HASH"
    if value.query_hash != common.identity_hash(value.query):
        return "QUERY_HASH"
    if set(value.context.tolist()) & set(value.query.tolist()):
        return "CONTEXT_QUERY_OVERLAP"
    if value.context.size < required:
        return "CONTEXT_BUDGET"
    return None


def _safe_numeric_indices(analysis: MeshAnalysis, value: CoverageInput) -> bool:
    return bool(
        value.context.ndim == 1
        and value.query.ndim == 1
        and value.context.size > 0
        and value.query.size > 0
        and np.all(value.context >= 0)
        and np.all(value.context < analysis.mesh.vertex_count)
        and np.all(value.query >= 0)
        and np.all(value.query < analysis.mesh.vertex_count)
    )


def _decision(
    value: CoverageInput,
    method: str,
    vertex: int,
    reason: str,
    detail: str | None,
    distance_evaluated: bool,
    local_score: float | None,
    global_fill: float | None,
    mesh_ratio: float | None,
) -> dict[str, Any]:
    for number in (local_score, global_fill, mesh_ratio):
        if number is not None and not math.isfinite(number):
            raise common.C0Error("C0 attempted to serialize a non-finite score")
    return {
        "case": value.case,
        "case_group": value.case_group,
        "context_count": int(value.context.size),
        "detail": detail,
        "distance_evaluated": distance_evaluated,
        "global_fill": global_fill,
        "local_score": local_score,
        "mesh_ratio": mesh_ratio,
        "method": method,
        "object_id": value.object_id,
        "query_vertex": int(vertex),
        "reason": reason,
        "role": value.role,
        "schema": common.DECISION_SCHEMA,
        "topology": value.topology,
    }


def _graph_statistics(
    analysis: MeshAnalysis, value: CoverageInput
) -> tuple[np.ndarray, float | None, float | None]:
    distances = _multi_source_distance(value.graph, value.context)
    if np.isfinite(distances).all():
        global_fill = float(np.max(distances) / analysis.graph_diameter)
    else:
        global_fill = None
    pair = analysis.all_pairs[np.ix_(value.context, value.context)].copy()
    pair[np.diag_indices_from(pair)] = np.inf
    separation = 0.5 * float(np.min(pair) / analysis.graph_diameter)
    mesh_ratio = (
        None
        if global_fill is None or separation <= 0.0
        else float(global_fill / separation)
    )
    return distances, global_fill, mesh_ratio


def evaluate(
    analysis: MeshAnalysis,
    value: CoverageInput,
    method: str,
) -> list[dict[str, Any]]:
    if method not in METHOD_ORDER:
        raise common.C0Error(f"unknown C0 method: {method}")
    failure = structural_failure(analysis, value)
    if method in ("composite", "structural-only") and failure is not None:
        return [
            _decision(
                value,
                method,
                int(vertex),
                "OOD_CONTEXT_BUDGET",
                failure,
                False,
                None,
                None,
                None,
            )
            for vertex in value.query
        ]
    if method == "structural-only":
        return [
            _decision(
                value, method, int(vertex), "ACCEPT", None, False, None, None, None
            )
            for vertex in value.query
        ]
    if not _safe_numeric_indices(analysis, value):
        return [
            _decision(
                value,
                method,
                int(vertex),
                "CONTROL_INPUT_INVALID",
                "MEMORY_SAFE_INDEX_BOUND",
                False,
                None,
                None,
                None,
            )
            for vertex in value.query
        ]
    if method == "euclidean-local-only":
        scores = (
            np.min(analysis.euclidean_distances[value.context][:, value.query], axis=0)
            / analysis.euclidean_diameter
        )
        return [
            _decision(
                value,
                method,
                int(vertex),
                "OOD_INTRINSIC_GAP"
                if float(scores[offset]) > common.LOCAL_EUCLIDEAN_THRESHOLD
                else "ACCEPT",
                None,
                True,
                float(scores[offset]),
                None,
                None,
            )
            for offset, vertex in enumerate(value.query)
        ]
    distances, global_fill, mesh_ratio = _graph_statistics(analysis, value)
    decisions: list[dict[str, Any]] = []
    for vertex in value.query:
        raw = float(distances[int(vertex)])
        local = None if not math.isfinite(raw) else raw / analysis.graph_diameter
        if method == "intrinsic-local-only":
            reason = (
                "OOD_DISCONNECTED"
                if local is None
                else (
                    "OOD_INTRINSIC_GAP"
                    if local > common.LOCAL_INTRINSIC_THRESHOLD
                    else "ACCEPT"
                )
            )
            emitted_global = None
            emitted_ratio = None
        else:
            if local is None or global_fill is None:
                reason = "OOD_DISCONNECTED"
            elif global_fill > common.GLOBAL_INTRINSIC_THRESHOLD:
                reason = "OOD_INTRINSIC_FILL"
            elif local > common.LOCAL_INTRINSIC_THRESHOLD:
                reason = "OOD_INTRINSIC_GAP"
            else:
                reason = "ACCEPT"
            emitted_global = global_fill
            emitted_ratio = mesh_ratio
        decisions.append(
            _decision(
                value,
                method,
                int(vertex),
                reason,
                None,
                True,
                local,
                emitted_global,
                emitted_ratio,
            )
        )
    return decisions


def calibrate_development(analyses: tuple[MeshAnalysis, ...]) -> dict[str, Any]:
    intrinsic_scores: list[np.ndarray] = []
    euclidean_scores: list[np.ndarray] = []
    intrinsic_fill: list[float] = []
    euclidean_fill: list[float] = []
    mesh_ratio: list[float] = []
    for analysis in analyses:
        value = valid_input(analysis)
        distances, global_fill, ratio = _graph_statistics(analysis, value)
        if global_fill is None or ratio is None:
            raise common.C0Error("valid C0 development graph is unreachable")
        intrinsic_scores.append(distances[value.query] / analysis.graph_diameter)
        intrinsic_fill.append(global_fill)
        mesh_ratio.append(ratio)
        euclidean = (
            np.min(analysis.euclidean_distances[value.context][:, value.query], axis=0)
            / analysis.euclidean_diameter
        )
        euclidean_scores.append(euclidean)
        euclidean_fill.append(
            float(
                np.max(np.min(analysis.euclidean_distances[value.context], axis=0))
                / analysis.euclidean_diameter
            )
        )
    local = max(
        0.05,
        1.25 * float(np.percentile(np.concatenate(intrinsic_scores), 99.0)),
    )
    global_value = max(0.05, 1.25 * max(intrinsic_fill))
    euclidean_local = max(
        0.05,
        1.25 * float(np.percentile(np.concatenate(euclidean_scores), 99.0)),
    )
    euclidean_global = max(0.05, 1.25 * max(euclidean_fill))
    observed = (
        local,
        global_value,
        euclidean_local,
        euclidean_global,
    )
    expected = (
        common.LOCAL_INTRINSIC_THRESHOLD,
        common.GLOBAL_INTRINSIC_THRESHOLD,
        common.LOCAL_EUCLIDEAN_THRESHOLD,
        common.GLOBAL_EUCLIDEAN_THRESHOLD,
    )
    if observed != expected:
        raise common.C0Error(f"C0 frozen calibration changed: {observed}")
    return {
        "development_mesh_record_root": common.DEVELOPMENT_RECORD_ROOT,
        "euclidean": {
            "global_threshold": euclidean_global,
            "local_threshold": euclidean_local,
        },
        "intrinsic": {
            "global_fill_max": max(intrinsic_fill),
            "global_fill_min": min(intrinsic_fill),
            "global_threshold": global_value,
            "local_p99": float(np.percentile(np.concatenate(intrinsic_scores), 99.0)),
            "local_threshold": local,
            "mesh_ratio_max": max(mesh_ratio),
            "mesh_ratio_min": min(mesh_ratio),
        },
        "schema": common.CALIBRATION_SCHEMA,
    }


def _is_reject(decision: dict[str, Any]) -> bool:
    reason = decision["reason"]
    if reason != "ACCEPT" and reason not in REJECT_REASONS:
        raise common.C0Error(f"unknown C0 decision reason: {reason}")
    return reason != "ACCEPT"


def _fraction(decisions: list[dict[str, Any]]) -> float:
    if not decisions:
        raise common.C0Error("C0 aggregate received no decisions")
    return float(np.mean([_is_reject(item) for item in decisions]))


def _select(
    decisions: list[dict[str, Any]],
    *,
    case_group: str | None = None,
    case: str | None = None,
    topology: str | None = None,
    object_id: str | None = None,
) -> list[dict[str, Any]]:
    return [
        item
        for item in decisions
        if (case_group is None or item["case_group"] == case_group)
        and (case is None or item["case"] == case)
        and (topology is None or item["topology"] == topology)
        and (object_id is None or item["object_id"] == object_id)
    ]


def _utility(decisions: list[dict[str, Any]]) -> dict[str, float]:
    numerical_cells = [
        _fraction(
            _select(
                decisions,
                case_group="numerical-mutation",
                case=case,
                topology=topology,
            )
        )
        for case in NUMERICAL_CASE_ORDER
        for topology in common.TOPOLOGY_ORDER
    ]
    valid_cells = [
        _fraction(_select(decisions, case_group="valid", topology=topology))
        for topology in common.TOPOLOGY_ORDER
    ]
    mutation = float(np.mean(numerical_cells))
    valid = float(np.mean(valid_cells))
    return {
        "mutation_rejected_fraction": mutation,
        "utility": mutation - valid,
        "valid_rejected_fraction": valid,
    }


def aggregate_report(
    decisions_by_method: dict[str, list[dict[str, Any]]],
) -> tuple[dict[str, Any], dict[str, bool]]:
    composite = decisions_by_method["composite"]
    valid_object = {
        object_id: _fraction(
            _select(composite, case_group="valid", object_id=object_id)
        )
        for object_id in sorted(
            {item["object_id"] for item in composite if item["case_group"] == "valid"}
        )
    }
    valid_topology = {
        topology: _fraction(_select(composite, case_group="valid", topology=topology))
        for topology in common.TOPOLOGY_ORDER
    }
    mutation_class_topology = {
        f"{case}:{topology}": _fraction(
            _select(
                composite,
                case_group="numerical-mutation",
                case=case,
                topology=topology,
            )
        )
        for case in NUMERICAL_CASE_ORDER
        for topology in common.TOPOLOGY_ORDER
    }
    mutation_object = {
        object_id: _fraction(
            _select(
                composite,
                case_group="numerical-mutation",
                object_id=object_id,
            )
        )
        for object_id in sorted(
            {
                item["object_id"]
                for item in composite
                if item["case_group"] == "numerical-mutation"
            }
        )
    }
    utility = {
        method: _utility(decisions_by_method[method])
        for method in ("composite", "intrinsic-local-only", "euclidean-local-only")
    }
    thinning = _select(composite, case_group="numerical-mutation", case="thinning")
    component = _select(
        composite, case_group="numerical-mutation", case="component-isolation"
    )
    rolled = _select(
        composite,
        case_group="numerical-mutation",
        case="ambient-shortcut",
        topology="RolledSheet",
    )
    rolled_euclidean = _select(
        decisions_by_method["euclidean-local-only"],
        case_group="numerical-mutation",
        case="ambient-shortcut",
        topology="RolledSheet",
    )
    structural = _select(composite, case_group="structural-mutation")
    graph_identity_mismatch = _select(
        decisions_by_method["graph-only"],
        case_group="structural-mutation",
        case="identity-mismatch",
    )
    structural_cap = _select(
        decisions_by_method["structural-only"],
        case_group="numerical-mutation",
        case="intrinsic-cap",
    )
    keys = [
        (
            item["role"],
            item["object_id"],
            item["case"],
            item["query_vertex"],
        )
        for item in composite
    ]
    finite_records = all(
        value is None or math.isfinite(float(value))
        for item in composite
        for value in (item["local_score"], item["global_fill"], item["mesh_ratio"])
    )
    composite_utility = utility["composite"]["utility"]
    gates = {
        "composite_noninferior_utility": all(
            composite_utility >= utility[method]["utility"]
            for method in ("intrinsic-local-only", "euclidean-local-only")
        ),
        "component_reason_100_percent": all(
            item["reason"] == "OOD_DISCONNECTED" for item in component
        ),
        "complete_unique_decisions": len(keys) == len(set(keys)),
        "finite_or_null_scores": finite_records,
        "graph_layer_ablation_reachable": (
            _fraction(structural_cap) == 0.0
            and _fraction(
                _select(
                    composite,
                    case_group="numerical-mutation",
                    case="intrinsic-cap",
                )
            )
            == 1.0
        ),
        "mutation_class_topology_95_percent": all(
            value >= 0.95 for value in mutation_class_topology.values()
        ),
        "mutation_objects_11_of_12": sum(
            value >= 0.95 for value in mutation_object.values()
        )
        >= 11,
        "rolledsheet_ambient_intrinsic_beats_euclidean": (
            _fraction(rolled) >= 0.95
            and _fraction(rolled) > _fraction(rolled_euclidean)
        ),
        "structural_layer_ablation_reachable": (
            _fraction(graph_identity_mismatch) == 0.0
            and _fraction(
                _select(
                    composite,
                    case_group="structural-mutation",
                    case="identity-mismatch",
                )
            )
            == 1.0
        ),
        "structural_mutations_100_percent_before_distance": all(
            item["reason"] == "OOD_CONTEXT_BUDGET" and not item["distance_evaluated"]
            for item in structural
        ),
        "thinning_reason_100_percent": all(
            item["reason"] == "OOD_CONTEXT_BUDGET" for item in thinning
        ),
        "valid_objects_11_of_12": sum(value <= 0.10 for value in valid_object.values())
        >= 11,
        "valid_topologies_10_percent": all(
            value <= 0.10 for value in valid_topology.values()
        ),
    }
    report = {
        "mutation_class_topology": mutation_class_topology,
        "mutation_object": mutation_object,
        "reason_rates": {
            "component_disconnected": float(
                np.mean([item["reason"] == "OOD_DISCONNECTED" for item in component])
            ),
            "rolledsheet_ambient_rejected": _fraction(rolled),
            "structural_budget": float(
                np.mean([item["reason"] == "OOD_CONTEXT_BUDGET" for item in structural])
            ),
            "thinning_budget": float(
                np.mean([item["reason"] == "OOD_CONTEXT_BUDGET" for item in thinning])
            ),
        },
        "utility": utility,
        "valid_object": valid_object,
        "valid_topology": valid_topology,
    }
    return report, gates


def evaluate_role(
    analyses: tuple[MeshAnalysis, ...],
) -> dict[str, list[dict[str, Any]]]:
    values = [value for analysis in analyses for value in all_inputs(analysis)]
    by_object = {analysis.mesh.row.object_id: analysis for analysis in analyses}
    return {
        method: [
            decision
            for value in values
            for decision in evaluate(by_object[value.object_id], value, method)
        ]
        for method in METHOD_ORDER
    }


def development_preview() -> dict[str, Any]:
    meshes = common.generate_meshes("development")
    analyses = tuple(analyze_mesh(mesh) for mesh in meshes)
    calibration = calibrate_development(analyses)
    decisions = evaluate_role(analyses)
    aggregate, gates = aggregate_report(decisions)
    return {
        "aggregate": aggregate,
        "calibration": calibration,
        "gates": gates,
        "mesh_count": len(meshes),
        "test_rows_generated": 0,
    }


def _score_arrays(
    decisions_by_method: dict[str, list[dict[str, Any]]],
) -> dict[str, np.ndarray]:
    arrays: dict[str, np.ndarray] = {}
    for method in METHOD_ORDER:
        records = decisions_by_method[method]
        token = method.replace("-", "_")
        arrays[f"{token}_reject"] = np.asarray(
            [_is_reject(item) for item in records], dtype=np.int64
        )
        for field in ("local_score", "global_fill", "mesh_ratio"):
            arrays[f"{token}_{field}"] = np.asarray(
                [float(item[field]) for item in records if item[field] is not None],
                dtype=np.float64,
            )
    return arrays


def build_run() -> tuple[dict[str, bytes], dict[str, Any]]:
    environment = common.verify_protocol_environment()
    implementation = common.implementation_hashes()
    development_meshes = common.generate_meshes("development")
    development = tuple(analyze_mesh(mesh) for mesh in development_meshes)
    calibration = calibrate_development(development)

    test_meshes = common.generate_meshes("test")
    test = tuple(analyze_mesh(mesh) for mesh in test_meshes)
    development_decisions = evaluate_role(development)
    test_decisions = evaluate_role(test)
    aggregate, gates = aggregate_report(test_decisions)
    gates.update(
        {
            "all_case_records_complete": all(
                len(all_inputs(analysis)) == 9 for analysis in development + test
            ),
            "zero_access": all(value == 0 for value in common.ZERO_ACCESS.values()),
        }
    )
    single_run_pass = all(gates.values())
    report = {
        "access": common.ZERO_ACCESS,
        "aggregate": aggregate,
        "decision": (
            "C0_SINGLE_RUN_PASS_REPEAT_PENDING"
            if single_run_pass
            else "C0_CAPABILITY_REJECT"
        ),
        "environment": environment,
        "gates": gates,
        "repeat_gate": "REQUIRES_SECOND_INDEPENDENT_FRESH_ROOT",
        "schema": common.REPORT_SCHEMA,
        "single_run_pass": single_run_pass,
        "study_id": common.STUDY_ID,
    }
    all_meshes = development_meshes + test_meshes
    all_analyses = development + test
    corpus = {
        "cases": [
            value.record()
            for analysis in all_analyses
            for value in all_inputs(analysis)
        ],
        "development": [mesh.record() for mesh in development_meshes],
        "protocol_sha256": common.PROTOCOL_SHA256,
        "schema": common.CORPUS_SCHEMA,
        "test": [mesh.record() for mesh in test_meshes],
    }
    composite_decisions = (
        development_decisions["composite"] + test_decisions["composite"]
    )
    access = {
        "counters": common.ZERO_ACCESS,
        "schema": common.ACCESS_SCHEMA,
    }
    combined_scores = {
        method: development_decisions[method] + test_decisions[method]
        for method in METHOD_ORDER
    }
    payload_files = {
        "access-ledger.json": common.canonical_json(access),
        "calibration.json": common.canonical_json(calibration),
        "corpus.json": common.canonical_json(corpus),
        "decisions.jsonl": common.canonical_json_lines(composite_decisions),
        "geometry.npz": common.geometry_npz(all_meshes),
        "report.json": common.canonical_json(report),
        "scores.npz": common.deterministic_npz(_score_arrays(combined_scores)),
    }
    manifest = {
        "access": common.ZERO_ACCESS,
        "artifact_files": {
            name: common.sha256_bytes(value)
            for name, value in sorted(payload_files.items())
        },
        "counts": {
            "development_meshes": len(development_meshes),
            "test_meshes": len(test_meshes),
        },
        "implementation_hashes": implementation,
        "protocol_sha256": common.PROTOCOL_SHA256,
        "revision": common.REVISION,
        "schema": common.MANIFEST_SCHEMA,
        "study_id": common.STUDY_ID,
    }
    files = {**payload_files, "manifest.json": common.canonical_json(manifest)}
    if tuple(sorted(files)) != (
        "access-ledger.json",
        "calibration.json",
        "corpus.json",
        "decisions.jsonl",
        "geometry.npz",
        "manifest.json",
        "report.json",
        "scores.npz",
    ):
        raise common.C0Error("C0 artifact set changed")
    return files, report


def run(output: Path) -> dict[str, Any]:
    staging, destination = common.prepare_output(output)
    try:
        files, report = build_run()
        common.write_files(staging, files)
        if len(common.directory_file_map(staging)) != 8:
            raise common.C0Error("C0 output file count changed")
        common.publish_output(staging, destination)
    except BaseException:
        common.abandon_output(staging)
        raise
    return {
        "decision": report["decision"],
        "file_count": 8,
        "output": destination.as_posix(),
        "single_run_pass": report["single_run_pass"],
        "tree_digest": common.tree_digest(common.directory_file_map(destination)),
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    run_parser = commands.add_parser("run", help="execute one frozen C0 run")
    run_parser.add_argument("--output", required=True, type=Path)
    compare = commands.add_parser("compare", help="compare two completed C0 roots")
    compare.add_argument("--left", required=True, type=Path)
    compare.add_argument("--right", required=True, type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    if arguments.command == "run":
        result = run(arguments.output)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0 if result["single_run_pass"] else 2
    if arguments.command == "compare":
        result = common.compare_directories(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0 if result["byte_identical"] else 1
    raise common.C0Error(f"unknown C0 command: {arguments.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except common.C0Error as error:
        sys.stderr.write(f"C0_ERROR: {error}\n")
        raise SystemExit(3) from error
