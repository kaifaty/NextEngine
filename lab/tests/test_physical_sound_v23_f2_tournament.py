#!/usr/bin/env python3
"""Whole-boundary no-F2-value smoke for Physical Sound V23 F2a."""

from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

for _thread_variable in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
):
    os.environ[_thread_variable] = "1"

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v23_f2_common as common
import physical_sound_v23_f2_model as model
import physical_sound_v23_f2_tournament as tournament


def _handcrafted_row(
    role: str,
    pair: int,
    twin: bool,
    grid_u: int,
    grid_v: int,
) -> common.FieldRow:
    topology = ("Plate", "Cylinder")[pair]
    support = ("Free", "BaseClamped")[pair]
    group = f"api-smoke-pair-{pair}"
    return common.FieldRow(
        physical_group_id=group,
        object_id=f"{group}-{'twin' if twin else 'primary'}",
        role=role,
        cell=pair,
        halton_index=pair + 1,
        material="Steel",
        topology=topology,
        support=support,
        length_m=0.31 + 0.04 * pair,
        aspect=0.91 + 0.08 * pair,
        slenderness=0.005 + 0.001 * pair,
        wall_m=(0.31 + 0.04 * pair) * (0.005 + 0.001 * pair),
        grid_u=grid_u,
        grid_v=grid_v,
    )


def _grid_arrays(
    grid_u: int, grid_v: int, pair: int
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    u_axis = np.linspace(-1.0, 1.0, grid_u, dtype=np.float64)
    v_axis = np.linspace(-1.0, 1.0, grid_v, dtype=np.float64)
    uv = np.asarray([(u, v) for v in v_axis for u in u_axis], dtype=np.float64)
    u = uv[:, 0]
    v = uv[:, 1]
    vertices = np.column_stack(
        (
            0.16 * u,
            (0.13 + 0.01 * pair) * v,
            0.004 * (pair + 1) * np.sin(np.pi * u) * np.cos(np.pi * v),
        )
    )
    faces: list[tuple[int, int, int]] = []
    for row in range(grid_v - 1):
        for column in range(grid_u - 1):
            lower = row * grid_u + column
            faces.extend(
                (
                    (lower, lower + 1, lower + grid_u + 1),
                    (lower, lower + grid_u + 1, lower + grid_u),
                )
            )
    face_array = np.asarray(faces, dtype=np.int64)
    edge_set: set[tuple[int, int]] = set()
    for face in face_array:
        for left, right in (
            (int(face[0]), int(face[1])),
            (int(face[1]), int(face[2])),
            (int(face[2]), int(face[0])),
        ):
            edge_set.add((min(left, right), max(left, right)))
    edges = np.asarray(sorted(edge_set), dtype=np.int64)
    edge_lengths = np.linalg.norm(vertices[edges[:, 0]] - vertices[edges[:, 1]], axis=1)
    return uv, vertices, face_array, edges, edge_lengths


def _handcrafted_gains(uv: np.ndarray, pair: int) -> np.ndarray:
    u = uv[:, 0:1]
    v = uv[:, 1:2]
    mode = np.arange(1, common.MODE_COUNT + 1, dtype=np.float64)[None, :]
    gains = (
        0.55
        + 0.17 * np.sin((0.3 + 0.11 * mode) * np.pi * u + 0.2 * pair)
        + 0.13 * np.cos((0.4 + 0.07 * mode) * np.pi * v - 0.1 * pair)
        + 0.04 * mode * u * v
    )
    if gains.shape != (uv.shape[0], common.MODE_COUNT):
        raise AssertionError("handcrafted gain shape changed")
    return np.asarray(gains, dtype=np.float64)


def _handcrafted_object(
    role: str,
    pair: int,
    twin: bool,
) -> common.FieldObject:
    grid_u, grid_v = ((5, 5), (6, 4))[int(twin)]
    row = _handcrafted_row(role, pair, twin, grid_u, grid_v)
    uv, vertices, faces, edges, edge_lengths = _grid_arrays(grid_u, grid_v, pair)
    mesh_hash = common.sha256_bytes(
        common.canonical_json(row.record())
        + common.f0_common.coverage_common.array_bytes(uv)
        + common.f0_common.coverage_common.array_bytes(vertices)
        + common.f0_common.coverage_common.array_bytes(faces)
    )
    mesh = common.f0_common.Mesh(
        row=row,
        uv=uv,
        vertices=vertices,
        faces=faces,
        edges=edges,
        edge_lengths=edge_lengths,
        mesh_hash=mesh_hash,
    )
    oracle = common.f0_common.coverage_oracle
    analysis = oracle.analyze_mesh(mesh)
    context_count = 18 if twin else 16
    context = oracle._restricted_fps(
        analysis.all_pairs,
        analysis.lexicographic_rank,
        np.arange(mesh.vertex_count, dtype=np.int64),
        context_count,
    )
    selected = np.zeros(mesh.vertex_count, dtype=bool)
    selected[context] = True
    query = np.flatnonzero(~selected).astype(np.int64)
    coverage_input = oracle._coverage_input(
        analysis, "api-smoke", "api-smoke", context, query, analysis.graph
    )
    decisions = tuple(oracle.evaluate(analysis, coverage_input, "composite"))
    accepted = np.asarray(
        [value["query_vertex"] for value in decisions if value["reason"] == "ACCEPT"],
        dtype=np.int64,
    )
    rejected = np.asarray(
        [value["query_vertex"] for value in decisions if value["reason"] != "ACCEPT"],
        dtype=np.int64,
    )
    if accepted.size == 0:
        raise AssertionError("handcrafted smoke has no accepted query")
    normals = np.column_stack(
        (
            np.zeros(mesh.vertex_count),
            np.zeros(mesh.vertex_count),
            np.ones(mesh.vertex_count),
        )
    )
    curvatures = np.column_stack(
        (
            0.02 * (pair + 1) * np.ones(mesh.vertex_count),
            0.01 * np.sin(np.pi * uv[:, 0]),
        )
    )
    return common.FieldObject(
        row=row,
        mesh=mesh,
        analysis=analysis,
        context=context,
        query=query,
        accepted_query=accepted,
        rejected_query=rejected,
        coverage_input=coverage_input,
        coverage_decisions=decisions,
        normals=np.asarray(normals, dtype=np.float64),
        curvatures=np.asarray(curvatures, dtype=np.float64),
        gains=_handcrafted_gains(uv, pair),
    )


def _smoke_objects() -> tuple[
    tuple[common.FieldObject, ...], tuple[common.FieldObject, ...]
]:
    train_base = tuple(
        _handcrafted_object("train", pair, twin)
        for pair in range(2)
        for twin in (False, True)
    )
    development = tuple(
        _handcrafted_object("development", pair, twin)
        for pair in range(2)
        for twin in (False, True)
    )
    train = tuple(item for _ in range(12) for item in train_base)
    return train, development


def _gate_fixture() -> tuple[
    dict[str, list[dict[str, Any]]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
]:
    rows = []
    for group in range(12):
        topology = common.f0_common.TOPOLOGY_ORDER[group % 4]
        for suffix in ("primary", "twin"):
            rows.append(
                {
                    "edge_gradient_p99": 0.10,
                    "gain_nrmse": 0.10,
                    "object_id": f"api-gate-{group}-{suffix}",
                    "topology": topology,
                }
            )
    metrics = {}
    for method in tournament.METHOD_ORDER:
        value = 0.10 if method == "candidate" else 0.20
        metrics[method] = [
            {**row, "edge_gradient_p99": value, "gain_nrmse": value} for row in rows
        ]
    remesh = [
        {"gain_metric_drift": 0.01, "probe_disagreement_nrmse": 0.01} for _ in range(12)
    ]
    mutations = [
        {"mutation": mutation, "quality_reject": True}
        for mutation in tournament.MUTATION_ORDER
        for _ in range(12)
    ]
    structural = [{"pre_inference_reject": True} for _ in range(120)]
    f0_metrics = [
        {**row, "edge_gradient_p99": 0.11, "gain_nrmse": 0.11} for row in rows
    ]
    f0_remesh = [{"gain_metric_drift": 0.02} for _ in range(12)]
    coverage = [
        {
            "fallback_complete": True,
            "global_pass": True,
            "local_false_ood_fraction": 0.0,
        }
        for _ in range(24)
    ]
    return (
        metrics,
        remesh,
        mutations,
        structural,
        f0_metrics,
        f0_remesh,
        coverage,
    )


def _selection_through_adapter(
    trained: model.TrainingResult,
    metrics: dict[str, list[dict[str, Any]]],
    remesh: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "selection_key": tournament.evaluator._selection_key(trained, metrics, remesh)
    }


def _full_api_smoke() -> bytes:
    api = tournament.api_surface_report()
    train, development = _smoke_objects()
    candidates = model.fit_candidates(train)
    control = model.train_paired_harmonic(train, candidates[0].gain_scale)
    payload = model.encode_models(candidates, control)
    if payload != model.encode_models(candidates, control):
        raise AssertionError("model serialization is not exact")
    decoded, decoded_control, scale = model.decode_models(payload)

    prediction_hashes: dict[str, dict[str, str]] = {}
    first_predictions: tuple[np.ndarray, ...] | None = None
    for index, trained in enumerate(candidates):
        if trained.spec is None:
            raise AssertionError("candidate spec is absent")
        model.validate_request(
            development[0], trained.spec, expected_role="development"
        )
        prediction = model.predict_candidate(
            trained.model, trained.gain_scale, development[0], trained.spec
        )
        decoded_prediction = model.predict_candidate(
            decoded[index], scale, development[0], trained.spec
        )
        if not np.array_equal(prediction, decoded_prediction):
            raise AssertionError("candidate prediction roundtrip changed")
        probes = model.predict_direct_probes(
            trained.model, trained.gain_scale, development[0], trained.spec
        )
        compatible = model.compatible_predictions(
            trained.model, trained.gain_scale, development[0], trained.spec
        )
        if set(compatible) != set(tournament.METHOD_ORDER):
            raise AssertionError("compatibility prediction set changed")
        if any(not np.isfinite(value).all() for value in compatible.values()):
            raise AssertionError("compatibility prediction is non-finite")
        prediction_hashes[trained.spec.candidate_id] = {
            "candidate": common.identity_hash(prediction),
            "direct_probes": common.identity_hash(probes),
            **{
                f"method_{name}": common.identity_hash(value)
                for name, value in sorted(compatible.items())
            },
        }
        if index == 0:
            first_predictions = tuple(
                model.predict_candidate(
                    trained.model, trained.gain_scale, item, trained.spec
                )
                for item in development
            )

    if first_predictions is None:
        raise AssertionError("candidate prediction smoke is empty")
    decoded_control_prediction = model.predict_paired_harmonic(
        decoded_control, scale, development[0]
    )
    control_prediction = model.predict_paired_harmonic(
        control.model, control.gain_scale, development[0]
    )
    if not np.array_equal(decoded_control_prediction, control_prediction):
        raise AssertionError("analytic control roundtrip changed")

    numerical = tournament._invoke(
        tournament.evaluator._mutation_rows,
        candidates[0],
        development,
        first_predictions,
    )
    structural = tournament._invoke(
        tournament.evaluator._structural_rows,
        development,
        candidates[0].spec,
    )
    if {row["mutation"] for row in numerical} != set(tournament.MUTATION_ORDER):
        raise AssertionError("numerical mutation coverage changed")
    if {row["mutation"] for row in structural} != set(tournament.STRUCTURAL_ORDER):
        raise AssertionError("structural mutation coverage changed")
    if not all(row["pre_inference_reject"] for row in structural):
        raise AssertionError("structural mutation escaped validation")

    fixture = _gate_fixture()
    gates = tournament._invoke(
        tournament.evaluator._candidate_gates,
        *fixture,
        True,
    )
    if not gates or not all(gates.values()):
        raise AssertionError("inherited gate adapter did not pass smoke fixture")
    selection = tournament._invoke(
        _selection_through_adapter,
        candidates[0],
        fixture[0],
        fixture[1],
    )

    report = {
        "api": api,
        "candidate_count": len(candidates),
        "candidate_summaries": [value.summary() for value in candidates],
        "control_summary": control.summary(),
        "gates": gates,
        "numerical_mutation_count": len(numerical),
        "parameter_bytes": [model.parameter_bytes(value.model) for value in candidates],
        "prediction_hashes": prediction_hashes,
        "selection": selection,
        "structural_mutation_count": len(structural),
    }
    report_payload = common.canonical_json(report)
    metrics_payload = common.canonical_json_lines([*numerical, *structural])
    files = {
        "access-ledger.json": common.canonical_json(common.ZERO_ACCESS),
        "corpus.json": common.canonical_json(
            {"base_pair_count": 2, "development_views": len(development)}
        ),
        "geometry.npz": common.geometry_npz(development),
        "manifest.json": common.canonical_json(
            {"api_members": sorted(api["members"]), "smoke": True}
        ),
        "metrics.jsonl": metrics_payload,
        "models.npz": payload,
        "predictions.npz": common.deterministic_npz(
            {
                "candidate": first_predictions[0],
                "control": control_prediction,
            }
        ),
        "report.json": report_payload,
        "selection.json": common.canonical_json(selection),
    }
    if set(files) != tournament.OUTPUT_FILES:
        raise AssertionError("smoke output member set changed")
    common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
        staging, _target = common.prepare_output(Path(temporary) / "api-smoke")
        common.write_files(staging, files)
        if set(common.directory_file_map(staging)) != tournament.OUTPUT_FILES:
            raise AssertionError("written smoke output member set changed")
        common.abandon_output(staging)
        if staging.exists():
            raise AssertionError("smoke staging abandon failed")
    return common.canonical_json(
        {
            "file_hashes": {
                name: common.sha256_bytes(value)
                for name, value in sorted(files.items())
            },
            "report": report,
        }
    )


class PhysicalSoundV23F2Tests(unittest.TestCase):
    def test_protocol_environment_lineage_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_fresh_role_roots_are_metadata_only_and_sealed(self) -> None:
        expected = {"train": 48, "development": 24, "test": 24, "integration": 24}
        for role, count in expected.items():
            rows = common.generate_rows(role)
            self.assertEqual(len(rows), count)
            self.assertEqual(
                common.sha256_bytes(
                    common.canonical_json([row.record() for row in rows])
                ),
                common.ROW_ROOTS[role],
            )
        self.assertEqual(
            common.sha256_bytes(common.canonical_json(common.row_ledger())),
            common.ROW_LEDGER_ROOT,
        )
        with self.assertRaises(common.F1Error):
            common.generate_objects("test", "0" * 40)
        with self.assertRaises(common.F1Error):
            common.generate_objects("integration", "0" * 40)

    def test_candidate_identity_budget_and_bank_are_frozen(self) -> None:
        self.assertEqual(
            [
                (value.candidate_id, value.dimension, value.bandwidth, value.order)
                for value in model.candidate_specs()
            ],
            [
                ("ffr-d64-s0p5-q2", 64, 0.5, 2),
                ("ffr-d64-s0p5-q3", 64, 0.5, 3),
                ("ffr-d64-s1p0-q2", 64, 1.0, 2),
                ("ffr-d64-s1p0-q3", 64, 1.0, 3),
                ("ffr-d128-s0p5-q2", 128, 0.5, 2),
                ("ffr-d128-s0p5-q3", 128, 0.5, 3),
                ("ffr-d128-s1p0-q2", 128, 1.0, 2),
                ("ffr-d128-s1p0-q3", 128, 1.0, 3),
            ],
        )
        self.assertEqual(model.PARAMETER_BYTES, 75_648)
        self.assertEqual(model.MODEL_BYTE_LIMIT, 128 * 1024)
        self.assertEqual(model.BANK_SEED, 230_001)
        self.assertTrue(tournament.api_surface_report()["passed"])

    def test_whole_evaluator_api_smoke_is_twice_byte_exact(self) -> None:
        first = _full_api_smoke()
        second = _full_api_smoke()
        self.assertEqual(first, second)
        self.assertGreater(len(first), 1_000)

    def test_external_output_boundary_remains_atomic(self) -> None:
        self.assertEqual(len(tournament.OUTPUT_FILES), 9)
        with self.assertRaises(common.F1Error):
            common.prepare_output(common.repository_root() / "forbidden-f2-output")


if __name__ == "__main__":
    unittest.main()
