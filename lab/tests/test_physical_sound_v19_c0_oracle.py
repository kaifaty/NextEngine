#!/usr/bin/env python3
"""Focused development-only guards for Physical Sound V19 C0."""

from __future__ import annotations

import io
import math
import os
import sys
import tempfile
import unittest
from pathlib import Path

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

import physical_sound_v19_c0_common as common  # noqa: E402
import physical_sound_v19_c0_oracle as oracle  # noqa: E402


class PhysicalSoundV19C0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.development_meshes = common.generate_meshes("development")
        cls.development = tuple(
            oracle.analyze_mesh(mesh) for mesh in cls.development_meshes
        )

    def test_protocol_environment_dependency_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertEqual(
            environment["geometry_dependency_sha256"], common.LEGACY_COMMON_SHA256
        )
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_development_enumeration_is_exact_and_test_role_stays_unbuilt(self) -> None:
        rows = tuple(mesh.row for mesh in self.development_meshes)
        self.assertEqual([row.halton_index for row in rows], list(range(1101, 1113)))
        self.assertEqual(
            [row.material for row in rows],
            [
                "Steel",
                "Steel",
                "Steel",
                "Steel",
                "Wood",
                "Wood",
                "Wood",
                "Wood",
                "Glass",
                "Glass",
                "Glass",
                "Glass",
            ],
        )
        self.assertEqual(
            [row.topology for row in rows], list(common.TOPOLOGY_ORDER) * 3
        )
        self.assertEqual([row.support for row in rows], ["Free", "BaseClamped"] * 6)
        self.assertEqual(
            [(row.grid_u, row.grid_v) for row in rows],
            [(22 + cell % 2, 18 + cell % 3) for cell in range(12)],
        )
        self.assertEqual(
            common.sha256_bytes(
                common.canonical_json(
                    [mesh.record() for mesh in self.development_meshes]
                )
            ),
            common.DEVELOPMENT_RECORD_ROOT,
        )
        self.assertEqual(
            tuple(mesh.mesh_hash for mesh in self.development_meshes),
            common.DEVELOPMENT_MESH_HASHES,
        )
        self.assertEqual(common.ROLE_SPECS["test"]["base"], 1201)
        self.assertEqual(
            [common._grid("test", cell) for cell in range(12)],
            [(24 + cell % 2, 19 + cell % 3) for cell in range(12)],
        )

    def test_mesh_graph_and_rolledsheet_intrinsic_seam_are_canonical(self) -> None:
        graph_hashes = set()
        for analysis in self.development:
            mesh = analysis.mesh
            self.assertEqual(mesh.vertex_count, mesh.row.grid_u * mesh.row.grid_v)
            self.assertEqual(analysis.graph_hash, oracle.graph_identity(analysis.graph))
            self.assertTrue(np.isfinite(analysis.all_pairs).all())
            self.assertGreater(analysis.graph_diameter, 0.0)
            graph_hashes.add(analysis.graph_hash)
        self.assertEqual(len(graph_hashes), 12)
        rolled = next(
            analysis
            for analysis in self.development
            if analysis.mesh.row.topology == "RolledSheet"
        )
        for v_index in range(rolled.mesh.row.grid_v):
            left = v_index * rolled.mesh.row.grid_u
            right = left + rolled.mesh.row.grid_u - 1
            np.testing.assert_allclose(
                rolled.mesh.vertices[left], rolled.mesh.vertices[right], atol=1.0e-15
            )
            self.assertLessEqual(rolled.euclidean_distances[left, right], 1.0e-15)
            self.assertGreater(rolled.all_pairs[left, right], 0.0)

    def test_context_budget_fps_and_query_identity_are_exact(self) -> None:
        for analysis in self.development:
            expected = max(16, math.ceil(analysis.mesh.vertex_count / 8))
            self.assertEqual(analysis.context.size, expected)
            self.assertEqual(analysis.query.size, analysis.mesh.vertex_count - expected)
            repeated_context, repeated_query = oracle.farthest_point_context(
                analysis.all_pairs, analysis.mesh.vertices
            )
            np.testing.assert_array_equal(repeated_context, analysis.context)
            np.testing.assert_array_equal(repeated_query, analysis.query)
            value = oracle.valid_input(analysis)
            self.assertIsNone(oracle.structural_failure(analysis, value))

    def test_development_calibration_reproduces_frozen_thresholds(self) -> None:
        calibration = oracle.calibrate_development(self.development)
        self.assertEqual(
            calibration["development_mesh_record_root"],
            common.DEVELOPMENT_RECORD_ROOT,
        )
        self.assertEqual(
            calibration["intrinsic"]["local_threshold"],
            common.LOCAL_INTRINSIC_THRESHOLD,
        )
        self.assertEqual(
            calibration["intrinsic"]["global_threshold"],
            common.GLOBAL_INTRINSIC_THRESHOLD,
        )
        self.assertEqual(
            calibration["euclidean"]["local_threshold"],
            common.LOCAL_EUCLIDEAN_THRESHOLD,
        )
        self.assertEqual(
            calibration["euclidean"]["global_threshold"],
            common.GLOBAL_EUCLIDEAN_THRESHOLD,
        )

    def test_structural_mutations_reject_before_distance_with_stable_detail(
        self,
    ) -> None:
        expected_detail = {
            "duplicate-context": "DUPLICATE_CONTEXT",
            "out-of-range": "OUT_OF_RANGE",
            "minimum-tamper": "DECLARED_MINIMUM",
            "identity-mismatch": "MESH_IDENTITY",
        }
        for analysis in self.development:
            for mutation, detail in expected_detail.items():
                value = oracle.structural_input(analysis, mutation)
                self.assertEqual(oracle.structural_failure(analysis, value), detail)
                decisions = oracle.evaluate(analysis, value, "composite")
                self.assertTrue(decisions)
                self.assertTrue(
                    all(item["reason"] == "OOD_CONTEXT_BUDGET" for item in decisions)
                )
                self.assertTrue(all(item["detail"] == detail for item in decisions))
                self.assertTrue(
                    all(not item["distance_evaluated"] for item in decisions)
                )

    def test_numerical_mutations_reach_frozen_reason_layers(self) -> None:
        expected_reason = {
            "intrinsic-cap": "OOD_INTRINSIC_FILL",
            "component-isolation": "OOD_DISCONNECTED",
            "thinning": "OOD_CONTEXT_BUDGET",
            "ambient-shortcut": "OOD_INTRINSIC_FILL",
        }
        for analysis in self.development:
            for mutation, reason in expected_reason.items():
                value = oracle.numerical_input(analysis, mutation)
                decisions = oracle.evaluate(analysis, value, "composite")
                self.assertTrue(decisions)
                self.assertTrue(all(item["reason"] == reason for item in decisions))
                if mutation == "thinning":
                    self.assertTrue(
                        all(not item["distance_evaluated"] for item in decisions)
                    )
                else:
                    self.assertTrue(
                        all(item["distance_evaluated"] for item in decisions)
                    )

    def test_layer_ablation_and_development_preview_pass_without_test(self) -> None:
        first = self.development[0]
        cap = oracle.numerical_input(first, "intrinsic-cap")
        structural_only = oracle.evaluate(first, cap, "structural-only")
        self.assertTrue(all(item["reason"] == "ACCEPT" for item in structural_only))
        mismatch = oracle.structural_input(first, "identity-mismatch")
        graph_only = oracle.evaluate(first, mismatch, "graph-only")
        self.assertTrue(all(item["reason"] == "ACCEPT" for item in graph_only))
        preview = oracle.development_preview()
        self.assertEqual(preview["test_rows_generated"], 0)
        self.assertTrue(all(preview["gates"].values()))
        self.assertEqual(preview["aggregate"]["utility"]["composite"]["utility"], 1.0)

    def test_serialization_is_canonical_finite_and_timestamp_free(self) -> None:
        arrays = {
            "alpha": np.asarray([1.0, 2.0], dtype=np.float64),
            "beta": np.asarray([3, 4], dtype=np.int64),
        }
        first = common.deterministic_npz(arrays)
        second = common.deterministic_npz(arrays)
        self.assertEqual(first, second)
        with np.load(io.BytesIO(first), allow_pickle=False) as loaded:
            np.testing.assert_array_equal(loaded["alpha"], arrays["alpha"])
            np.testing.assert_array_equal(loaded["beta"], arrays["beta"])
        geometry_a = common.geometry_npz(self.development_meshes)
        geometry_b = common.geometry_npz(common.generate_meshes("development"))
        self.assertEqual(geometry_a, geometry_b)
        with self.assertRaises(ValueError):
            common.canonical_json({"invalid": float("inf")})
        with self.assertRaises(common.C0Error):
            common.deterministic_npz(
                {"invalid": np.asarray([float("nan")], dtype=np.float64)}
            )

    def test_output_boundary_rejects_escape_existing_and_symlink(self) -> None:
        with self.assertRaises(common.C0Error):
            common.prepare_output(common.repository_root() / "forbidden-c0-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            temporary_root = Path(temporary)
            with self.assertRaises(common.C0Error):
                common.prepare_output(temporary_root)
            staging, _output = common.prepare_output(temporary_root / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)
            symlink = temporary_root / "linked-output"
            symlink.symlink_to(common.EXPERIMENT_ROOT, target_is_directory=True)
            with self.assertRaises(common.C0Error):
                common.prepare_output(symlink / "child")


if __name__ == "__main__":
    unittest.main()
