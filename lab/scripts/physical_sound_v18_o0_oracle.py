#!/usr/bin/env python3
"""Official intrinsic-coverage runner for Physical Sound V18 O0."""

from __future__ import annotations

import argparse
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
from scipy.sparse import csr_matrix
from scipy.sparse.csgraph import dijkstra
from scipy.spatial.distance import cdist

import physical_sound_v18_o0_common as common


MUTATION_ORDER = (
    "intrinsic-cap",
    "component-isolation",
    "thinning",
    "ambient-shortcut",
)


@dataclass(frozen=True)
class MeshAnalysis:
    mesh: common.Mesh
    graph: csr_matrix
    all_pairs: np.ndarray
    graph_diameter: float
    euclidean_distances: np.ndarray
    euclidean_diameter: float
    context: np.ndarray
    query: np.ndarray
    lexicographic_rank: np.ndarray


@dataclass(frozen=True)
class EvaluationCase:
    role: str
    object_id: str
    topology: str
    case: str
    context: np.ndarray
    query: np.ndarray
    graph: csr_matrix


@dataclass(frozen=True)
class CaseResult:
    role: str
    object_id: str
    topology: str
    case: str
    context_count: int
    query: np.ndarray
    intrinsic_scores: np.ndarray
    euclidean_scores: np.ndarray


def graph_matrix(mesh: common.Mesh, edge_mask: np.ndarray | None = None) -> csr_matrix:
    if edge_mask is None:
        edge_mask = np.ones(mesh.edges.shape[0], dtype=bool)
    edge_mask = np.asarray(edge_mask, dtype=bool)
    if edge_mask.shape != (mesh.edges.shape[0],):
        raise common.O0Error("O0 edge mask shape changed")
    edges = mesh.edges[edge_mask]
    weights = mesh.edge_lengths[edge_mask]
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


def _lexicographic_rank(vertices: np.ndarray) -> np.ndarray:
    indices = np.arange(vertices.shape[0], dtype=np.int64)
    order = np.lexsort(
        (indices, vertices[:, 2], vertices[:, 1], vertices[:, 0])
    )
    rank = np.empty_like(order)
    rank[order] = np.arange(order.size, dtype=np.int64)
    return rank


def _ordered_by_distance(
    distances: np.ndarray, lexicographic_rank: np.ndarray, descending: bool
) -> np.ndarray:
    distances = np.asarray(distances, dtype=np.float64)
    if distances.shape != lexicographic_rank.shape or np.isnan(distances).any():
        raise common.O0Error("O0 distance ordering input changed")
    primary = -distances if descending else distances
    return np.lexsort((lexicographic_rank, primary))


def farthest_point_context(
    all_pairs: np.ndarray, vertices: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    vertex_count = vertices.shape[0]
    expected = max(16, math.ceil(vertex_count / 8))
    if all_pairs.shape != (vertex_count, vertex_count):
        raise common.O0Error("O0 all-pairs shape changed")
    if not np.isfinite(all_pairs).all():
        raise common.O0Error("valid O0 mesh is disconnected")
    rank = _lexicographic_rank(vertices)
    seed = int(np.argmin(rank))
    selected = [seed]
    selected_mask = np.zeros(vertex_count, dtype=bool)
    selected_mask[seed] = True
    nearest = all_pairs[seed].copy()
    while len(selected) < expected:
        candidate_distance = nearest.copy()
        candidate_distance[selected_mask] = -np.inf
        maximum = np.max(candidate_distance)
        candidates = np.flatnonzero(candidate_distance == maximum)
        if candidates.size == 0 or not np.isfinite(maximum):
            raise common.O0Error("O0 farthest-point selection failed")
        candidate = int(candidates[np.argmin(rank[candidates])])
        selected.append(candidate)
        selected_mask[candidate] = True
        nearest = np.minimum(nearest, all_pairs[candidate])
    context = np.asarray(selected, dtype=np.int64)
    query = np.flatnonzero(~selected_mask).astype(np.int64)
    return context, query


def analyze_mesh(mesh: common.Mesh) -> MeshAnalysis:
    graph = graph_matrix(mesh)
    all_pairs = np.asarray(dijkstra(graph, directed=False), dtype=np.float64)
    if all_pairs.shape != (mesh.vertex_count, mesh.vertex_count):
        raise common.O0Error("O0 Dijkstra shape changed")
    finite = all_pairs[np.isfinite(all_pairs)]
    graph_diameter = float(np.max(finite))
    if not np.isfinite(all_pairs).all() or graph_diameter <= 0.0:
        raise common.O0Error("valid O0 graph is disconnected or degenerate")
    euclidean = cdist(mesh.vertices, mesh.vertices, metric="euclidean")
    euclidean_diameter = float(np.max(euclidean))
    if not np.isfinite(euclidean).all() or euclidean_diameter <= 0.0:
        raise common.O0Error("O0 Euclidean geometry is degenerate")
    context, query = farthest_point_context(all_pairs, mesh.vertices)
    return MeshAnalysis(
        mesh=mesh,
        graph=graph,
        all_pairs=all_pairs,
        graph_diameter=graph_diameter,
        euclidean_distances=euclidean,
        euclidean_diameter=euclidean_diameter,
        context=context,
        query=query,
        lexicographic_rank=_lexicographic_rank(mesh.vertices),
    )


def _multi_source_distance(graph: csr_matrix, context: np.ndarray) -> np.ndarray:
    context = np.asarray(context, dtype=np.int64)
    if context.ndim != 1 or context.size == 0 or len(set(context.tolist())) != context.size:
        raise common.O0Error("O0 context is empty or non-unique")
    result = np.asarray(
        dijkstra(graph, directed=False, indices=context, min_only=True),
        dtype=np.float64,
    )
    if result.ndim != 1 or result.shape[0] != graph.shape[0] or np.isnan(result).any():
        raise common.O0Error("O0 multi-source Dijkstra result changed")
    return result


def _intrinsic_cap(analysis: MeshAnalysis, case: str) -> EvaluationCase:
    vertex_count = analysis.mesh.vertex_count
    seed = int(np.argmin(analysis.lexicographic_rank))
    seed_distance = analysis.all_pairs[seed]
    nearest = _ordered_by_distance(
        seed_distance, analysis.lexicographic_rank, descending=False
    )
    farthest = _ordered_by_distance(
        seed_distance, analysis.lexicographic_rank, descending=True
    )
    context = nearest[: math.ceil(0.40 * vertex_count)].astype(np.int64)
    query = farthest[: math.ceil(0.30 * vertex_count)].astype(np.int64)
    if set(context.tolist()) & set(query.tolist()):
        raise common.O0Error("O0 intrinsic-cap context/query overlap")
    return EvaluationCase(
        role=analysis.mesh.row.role,
        object_id=analysis.mesh.row.object_id,
        topology=analysis.mesh.row.topology,
        case=case,
        context=context,
        query=query,
        graph=analysis.graph,
    )


def mutation_case(analysis: MeshAnalysis, mutation: str) -> EvaluationCase:
    mesh = analysis.mesh
    if mutation == "intrinsic-cap":
        return _intrinsic_cap(analysis, mutation)
    if mutation == "component-isolation":
        row_index = np.arange(mesh.vertex_count, dtype=np.int64) // mesh.row.grid_u
        split = mesh.row.grid_v // 2
        lower = row_index < split
        seed = int(np.argmin(analysis.lexicographic_rank))
        seed_side = lower[seed]
        context = analysis.context[lower[analysis.context] == seed_side]
        query = np.flatnonzero(lower != seed_side).astype(np.int64)
        edge_mask = lower[mesh.edges[:, 0]] == lower[mesh.edges[:, 1]]
        return EvaluationCase(
            role=mesh.row.role,
            object_id=mesh.row.object_id,
            topology=mesh.row.topology,
            case=mutation,
            context=context,
            query=query,
            graph=graph_matrix(mesh, edge_mask),
        )
    if mutation == "thinning":
        context = analysis.context[::4]
        distance = _multi_source_distance(analysis.graph, context)
        order = _ordered_by_distance(
            distance, analysis.lexicographic_rank, descending=True
        )
        query = order[: math.ceil(0.25 * mesh.vertex_count)].astype(np.int64)
        return EvaluationCase(
            role=mesh.row.role,
            object_id=mesh.row.object_id,
            topology=mesh.row.topology,
            case=mutation,
            context=context,
            query=query,
            graph=analysis.graph,
        )
    if mutation == "ambient-shortcut":
        if mesh.row.topology != "RolledSheet":
            return _intrinsic_cap(analysis, mutation)
        context = np.flatnonzero(mesh.uv[:, 0] <= -0.75).astype(np.int64)
        query = np.flatnonzero(mesh.uv[:, 0] >= 0.75).astype(np.int64)
        return EvaluationCase(
            role=mesh.row.role,
            object_id=mesh.row.object_id,
            topology=mesh.row.topology,
            case=mutation,
            context=context,
            query=query,
            graph=analysis.graph,
        )
    raise common.O0Error(f"unknown O0 mutation: {mutation}")


def evaluate_case(analysis: MeshAnalysis, case: EvaluationCase) -> CaseResult:
    if case.object_id != analysis.mesh.row.object_id or case.role != analysis.mesh.row.role:
        raise common.O0Error("O0 case/analysis identity mismatch")
    if case.query.size == 0 or case.context.size == 0:
        raise common.O0Error("O0 case has an empty context or query")
    intrinsic_distance = _multi_source_distance(case.graph, case.context)
    intrinsic_scores = intrinsic_distance[case.query] / analysis.graph_diameter
    euclidean_scores = (
        np.min(analysis.euclidean_distances[case.context][:, case.query], axis=0)
        / analysis.euclidean_diameter
    )
    if np.isnan(intrinsic_scores).any() or not np.isfinite(euclidean_scores).all():
        raise common.O0Error("O0 case score is invalid")
    return CaseResult(
        role=case.role,
        object_id=case.object_id,
        topology=case.topology,
        case=case.case,
        context_count=int(case.context.size),
        query=case.query,
        intrinsic_scores=intrinsic_scores,
        euclidean_scores=euclidean_scores,
    )


def valid_case(analysis: MeshAnalysis) -> EvaluationCase:
    return EvaluationCase(
        role=analysis.mesh.row.role,
        object_id=analysis.mesh.row.object_id,
        topology=analysis.mesh.row.topology,
        case="valid",
        context=analysis.context,
        query=analysis.query,
        graph=analysis.graph,
    )


def calibrate_thresholds(
    development: tuple[MeshAnalysis, ...]
) -> tuple[float, float, tuple[CaseResult, ...]]:
    valid = tuple(evaluate_case(item, valid_case(item)) for item in development)
    intrinsic = np.concatenate([item.intrinsic_scores for item in valid])
    euclidean = np.concatenate([item.euclidean_scores for item in valid])
    intrinsic_threshold = max(0.05, 1.25 * float(np.percentile(intrinsic, 99.0)))
    euclidean_threshold = max(0.05, 1.25 * float(np.percentile(euclidean, 99.0)))
    return intrinsic_threshold, euclidean_threshold, valid


def _fraction(values: np.ndarray, threshold: float) -> float:
    return float(np.mean(np.asarray(values) > threshold))


def _score_records(
    cases: tuple[CaseResult, ...], intrinsic_threshold: float, euclidean_threshold: float
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for case in cases:
        for offset, vertex in enumerate(case.query):
            intrinsic = float(case.intrinsic_scores[offset])
            unreachable = not np.isfinite(intrinsic)
            euclidean = float(case.euclidean_scores[offset])
            records.append(
                {
                    "case": case.case,
                    "context_count": case.context_count,
                    "euclidean_reject": euclidean > euclidean_threshold,
                    "euclidean_score": euclidean,
                    "intrinsic_reject": unreachable or intrinsic > intrinsic_threshold,
                    "intrinsic_score": None if unreachable else intrinsic,
                    "intrinsic_unreachable": unreachable,
                    "object_id": case.object_id,
                    "role": case.role,
                    "topology": case.topology,
                    "vertex": int(vertex),
                }
            )
    return records


def _aggregate_report(
    test_valid: tuple[CaseResult, ...],
    test_mutations: tuple[CaseResult, ...],
    intrinsic_threshold: float,
    euclidean_threshold: float,
) -> tuple[dict[str, Any], dict[str, bool]]:
    valid_object: dict[str, float] = {}
    valid_topology: dict[str, float] = {}
    for case in test_valid:
        valid_object[case.object_id] = _fraction(case.intrinsic_scores, intrinsic_threshold)
    for topology in common.TOPOLOGY_ORDER:
        values = np.concatenate(
            [case.intrinsic_scores for case in test_valid if case.topology == topology]
        )
        valid_topology[topology] = _fraction(values, intrinsic_threshold)

    mutation_class_topology: dict[str, float] = {}
    for mutation in MUTATION_ORDER:
        for topology in common.TOPOLOGY_ORDER:
            values = np.concatenate(
                [
                    case.intrinsic_scores
                    for case in test_mutations
                    if case.case == mutation and case.topology == topology
                ]
            )
            mutation_class_topology[f"{mutation}:{topology}"] = _fraction(
                values, intrinsic_threshold
            )
    mutation_object: dict[str, float] = {}
    for object_id in sorted({case.object_id for case in test_mutations}):
        values = np.concatenate(
            [case.intrinsic_scores for case in test_mutations if case.object_id == object_id]
        )
        mutation_object[object_id] = _fraction(values, intrinsic_threshold)

    valid_intrinsic = np.concatenate([case.intrinsic_scores for case in test_valid])
    valid_euclidean = np.concatenate([case.euclidean_scores for case in test_valid])
    mutation_intrinsic = np.concatenate(
        [case.intrinsic_scores for case in test_mutations]
    )
    mutation_euclidean = np.concatenate(
        [case.euclidean_scores for case in test_mutations]
    )
    intrinsic_valid_reject = _fraction(valid_intrinsic, intrinsic_threshold)
    euclidean_valid_reject = _fraction(valid_euclidean, euclidean_threshold)
    intrinsic_mutation_reject = _fraction(mutation_intrinsic, intrinsic_threshold)
    euclidean_mutation_reject = _fraction(mutation_euclidean, euclidean_threshold)
    intrinsic_utility = intrinsic_mutation_reject - intrinsic_valid_reject
    euclidean_utility = euclidean_mutation_reject - euclidean_valid_reject

    rolled = np.concatenate(
        [
            case.intrinsic_scores
            for case in test_mutations
            if case.case == "ambient-shortcut" and case.topology == "RolledSheet"
        ]
    )
    rolled_rejection = _fraction(rolled, intrinsic_threshold)
    gates = {
        "intrinsic_beats_euclidean_utility": intrinsic_utility > euclidean_utility,
        "mutation_class_topology_95_percent": all(
            value >= 0.95 for value in mutation_class_topology.values()
        ),
        "mutation_objects_11_of_12": sum(
            value >= 0.95 for value in mutation_object.values()
        )
        >= 11,
        "rolledsheet_ambient_95_percent": rolled_rejection >= 0.95,
        "valid_objects_11_of_12": sum(value <= 0.10 for value in valid_object.values())
        >= 11,
        "valid_topologies_10_percent": all(
            value <= 0.10 for value in valid_topology.values()
        ),
    }
    report = {
        "euclidean": {
            "mutation_rejected_fraction": euclidean_mutation_reject,
            "utility": euclidean_utility,
            "valid_rejected_fraction": euclidean_valid_reject,
        },
        "intrinsic": {
            "mutation_rejected_fraction": intrinsic_mutation_reject,
            "rolledsheet_ambient_rejected_fraction": rolled_rejection,
            "utility": intrinsic_utility,
            "valid_rejected_fraction": intrinsic_valid_reject,
        },
        "mutation_class_topology": mutation_class_topology,
        "mutation_object": mutation_object,
        "valid_object": valid_object,
        "valid_topology": valid_topology,
    }
    return report, gates


def build_run() -> tuple[dict[str, bytes], dict[str, Any]]:
    environment = common.verify_protocol_environment_and_b0()
    implementation = common.implementation_hashes()
    development_meshes = common.generate_meshes("development")
    test_meshes = common.generate_meshes("test")
    development = tuple(analyze_mesh(mesh) for mesh in development_meshes)
    test = tuple(analyze_mesh(mesh) for mesh in test_meshes)

    intrinsic_threshold, euclidean_threshold, development_valid = calibrate_thresholds(
        development
    )
    test_valid = tuple(evaluate_case(item, valid_case(item)) for item in test)
    development_mutations = tuple(
        evaluate_case(item, mutation_case(item, mutation))
        for item in development
        for mutation in MUTATION_ORDER
    )
    test_mutations = tuple(
        evaluate_case(item, mutation_case(item, mutation))
        for item in test
        for mutation in MUTATION_ORDER
    )
    aggregate, gates = _aggregate_report(
        test_valid,
        test_mutations,
        intrinsic_threshold,
        euclidean_threshold,
    )
    gates.update(
        {
            "development_and_test_mutations_complete": (
                len(development_mutations) == 48 and len(test_mutations) == 48
            ),
            "finite_valid_scores": all(
                np.isfinite(case.intrinsic_scores).all()
                and np.isfinite(case.euclidean_scores).all()
                for case in development_valid + test_valid
            ),
            "integration_sealed": common.ZERO_ACCESS["integration_rows_generated"] == 0,
            "zero_access": all(value == 0 for value in common.ZERO_ACCESS.values()),
        }
    )
    single_run_pass = all(gates.values())
    report = {
        "access": common.ZERO_ACCESS,
        "aggregate": aggregate,
        "decision": (
            "O0_SINGLE_RUN_PASS_REPEAT_PENDING"
            if single_run_pass
            else "O0_CAPABILITY_REJECT"
        ),
        "environment": environment,
        "gates": gates,
        "repeat_gate": "REQUIRES_SECOND_INDEPENDENT_FRESH_ROOT",
        "schema": common.REPORT_SCHEMA,
        "single_run_pass": single_run_pass,
        "study_id": common.STUDY_ID,
        "thresholds": {
            "euclidean": euclidean_threshold,
            "intrinsic": intrinsic_threshold,
        },
    }
    all_cases = (
        development_valid + test_valid + development_mutations + test_mutations
    )
    scores = {
        "records": _score_records(all_cases, intrinsic_threshold, euclidean_threshold),
        "schema": "nextengine.experimental-physical-sound-v18-o0.scores.v1",
    }
    all_meshes = development_meshes + test_meshes
    corpus = {
        "b0_result_sha256": common.B0_RESULT_SHA256,
        "development": [mesh.record() for mesh in development_meshes],
        "integration_rows_generated": 0,
        "parent_protocol_sha256": common.PARENT_PROTOCOL_SHA256,
        "schema": "nextengine.experimental-physical-sound-v18-o0.corpus.v1",
        "test": [mesh.record() for mesh in test_meshes],
    }
    payload_files = {
        **common.geometry_files(all_meshes),
        "corpus.json": common.canonical_json(corpus),
        "report.json": common.canonical_json(report),
        "scores.json": common.canonical_json(scores),
    }
    manifest = {
        "access": common.ZERO_ACCESS,
        "artifact_files": {
            name: common.sha256_bytes(value)
            for name, value in sorted(payload_files.items())
        },
        "b0_result_sha256": common.B0_RESULT_SHA256,
        "counts": {
            "development_meshes": len(development_meshes),
            "integration_rows": 0,
            "test_meshes": len(test_meshes),
        },
        "implementation_hashes": implementation,
        "parent_protocol_sha256": common.PARENT_PROTOCOL_SHA256,
        "protocol_sha256": common.PROTOCOL_SHA256,
        "revision": common.REVISION,
        "schema": common.MANIFEST_SCHEMA,
        "study_id": common.STUDY_ID,
    }
    files = {**payload_files, "manifest.json": common.canonical_json(manifest)}
    return files, report


def run(output: Path) -> dict[str, Any]:
    staging, destination = common.prepare_output(output)
    try:
        files, report = build_run()
        common.write_files(staging, files)
        if len(common.directory_file_map(staging)) != len(files):
            raise common.O0Error("O0 output file count changed")
        common.publish_output(staging, destination)
    except BaseException:
        common.abandon_output(staging)
        raise
    return {
        "decision": report["decision"],
        "file_count": len(files),
        "output": destination.as_posix(),
        "single_run_pass": report["single_run_pass"],
        "tree_digest": common.tree_digest(common.directory_file_map(destination)),
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    run_parser = commands.add_parser("run", help="execute one frozen O0 run")
    run_parser.add_argument("--output", required=True, type=Path)
    compare = commands.add_parser("compare", help="compare two completed O0 roots")
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
    raise common.O0Error(f"unknown O0 command: {arguments.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except common.O0Error as error:
        sys.stderr.write(f"O0_ERROR: {error}\n")
        raise SystemExit(3) from error
