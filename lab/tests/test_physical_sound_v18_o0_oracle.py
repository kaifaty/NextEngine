#!/usr/bin/env python3
"""Focused guards for the frozen Physical Sound V18 O0 oracle."""

from __future__ import annotations

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

import physical_sound_v18_o0_common as common  # noqa: E402
import physical_sound_v18_o0_oracle as oracle  # noqa: E402


class PhysicalSoundV18O0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.development_meshes = common.generate_meshes("development")
        cls.development = tuple(
            oracle.analyze_mesh(mesh) for mesh in cls.development_meshes
        )

    def test_frozen_role_enumeration_and_sealed_integration(self) -> None:
        development = common.generate_rows("development")
        test = common.generate_rows("test")
        self.assertEqual(len(development), 12)
        self.assertEqual(len(test), 12)
        self.assertEqual([row.halton_index for row in development], list(range(501, 513)))
        self.assertEqual([row.halton_index for row in test], list(range(601, 613)))
        self.assertEqual(
            [(row.grid_u, row.grid_v) for row in development],
            [(17 + cell % 2, 15 + cell % 3) for cell in range(12)],
        )
        self.assertEqual(
            [(row.grid_u, row.grid_v) for row in test],
            [(19 + cell % 2, 16 + cell % 3) for cell in range(12)],
        )
        self.assertTrue(all(row.support == "BaseClamped" for row in development + test))
        with self.assertRaises(common.O0Error):
            common.generate_rows("integration")
        self.assertEqual(common.ZERO_ACCESS["integration_rows_generated"], 0)

    def test_mesh_counts_indices_edges_and_hashes_are_canonical(self) -> None:
        hashes = set()
        for mesh in self.development_meshes:
            row = mesh.row
            self.assertEqual(mesh.vertex_count, row.grid_u * row.grid_v)
            u_cells = row.grid_u if row.topology in ("Cylinder", "Bowl") else row.grid_u - 1
            expected_faces = 2 * u_cells * (row.grid_v - 1)
            self.assertEqual(mesh.faces.shape, (expected_faces, 3))
            self.assertTrue(np.all(mesh.faces >= 0))
            self.assertLess(int(np.max(mesh.faces)), mesh.vertex_count)
            self.assertEqual(len({tuple(edge) for edge in mesh.edges.tolist()}), mesh.edges.shape[0])
            self.assertTrue(np.all(mesh.edges[:, 0] < mesh.edges[:, 1]))
            self.assertTrue(np.all(mesh.edge_lengths > 0.0))
            self.assertEqual(mesh.mesh_hash, common.build_mesh(row).mesh_hash)
            hashes.add(mesh.mesh_hash)
        self.assertEqual(len(hashes), 12)

    def test_rolled_sheet_seam_is_ambiently_closed_but_intrinsically_open(self) -> None:
        analysis = next(
            item for item in self.development if item.mesh.row.topology == "RolledSheet"
        )
        mesh = analysis.mesh
        for v_index in range(mesh.row.grid_v):
            left = v_index * mesh.row.grid_u
            right = left + mesh.row.grid_u - 1
            np.testing.assert_allclose(mesh.vertices[left], mesh.vertices[right], atol=1.0e-15)
            self.assertLessEqual(analysis.euclidean_distances[left, right], 1.0e-15)
            self.assertGreater(analysis.all_pairs[left, right], 0.0)
            self.assertNotIn(
                (min(left, right), max(left, right)),
                {tuple(edge) for edge in mesh.edges.tolist()},
            )

    def test_all_pairs_and_farthest_point_context_are_exact_and_stable(self) -> None:
        for analysis in self.development:
            self.assertTrue(np.isfinite(analysis.all_pairs).all())
            np.testing.assert_allclose(
                analysis.all_pairs,
                analysis.all_pairs.T,
                rtol=0.0,
                atol=1.0e-15,
            )
            np.testing.assert_array_equal(np.diag(analysis.all_pairs), 0.0)
            expected = max(16, math.ceil(analysis.mesh.vertex_count / 8))
            self.assertEqual(analysis.context.size, expected)
            self.assertEqual(analysis.query.size, analysis.mesh.vertex_count - expected)
            repeated_context, repeated_query = oracle.farthest_point_context(
                analysis.all_pairs, analysis.mesh.vertices
            )
            np.testing.assert_array_equal(repeated_context, analysis.context)
            np.testing.assert_array_equal(repeated_query, analysis.query)

    def test_development_calibration_is_exact_and_valid_control_reachable(self) -> None:
        intrinsic, euclidean, valid = oracle.calibrate_thresholds(self.development)
        expected_intrinsic = max(
            0.05,
            1.25
            * float(
                np.percentile(
                    np.concatenate([case.intrinsic_scores for case in valid]), 99.0
                )
            ),
        )
        expected_euclidean = max(
            0.05,
            1.25
            * float(
                np.percentile(
                    np.concatenate([case.euclidean_scores for case in valid]), 99.0
                )
            ),
        )
        self.assertEqual(intrinsic, expected_intrinsic)
        self.assertEqual(euclidean, expected_euclidean)
        self.assertTrue(
            all(np.isfinite(case.intrinsic_scores).all() for case in valid)
        )
        self.assertLessEqual(
            float(
                np.mean(
                    np.concatenate([case.intrinsic_scores for case in valid])
                    > intrinsic
                )
            ),
            0.01,
        )

    def test_component_isolation_makes_every_other_component_query_unreachable(self) -> None:
        for analysis in self.development:
            case = oracle.mutation_case(analysis, "component-isolation")
            result = oracle.evaluate_case(analysis, case)
            self.assertTrue(np.isinf(result.intrinsic_scores).all())
            self.assertTrue(np.isfinite(result.euclidean_scores).all())

    def test_development_rolledsheet_shortcut_rejects_intrinsically(self) -> None:
        intrinsic_threshold, euclidean_threshold, _ = oracle.calibrate_thresholds(
            self.development
        )
        rolled = [
            analysis
            for analysis in self.development
            if analysis.mesh.row.topology == "RolledSheet"
        ]
        intrinsic_scores = []
        euclidean_scores = []
        for analysis in rolled:
            result = oracle.evaluate_case(
                analysis, oracle.mutation_case(analysis, "ambient-shortcut")
            )
            intrinsic_scores.append(result.intrinsic_scores)
            euclidean_scores.append(result.euclidean_scores)
        intrinsic_fraction = float(
            np.mean(np.concatenate(intrinsic_scores) > intrinsic_threshold)
        )
        euclidean_fraction = float(
            np.mean(np.concatenate(euclidean_scores) > euclidean_threshold)
        )
        self.assertGreaterEqual(intrinsic_fraction, 0.95)
        self.assertGreater(intrinsic_fraction, euclidean_fraction)

    def test_mutation_context_query_identity_is_stable_on_development(self) -> None:
        identities = set()
        for analysis in self.development:
            for mutation in oracle.MUTATION_ORDER:
                case = oracle.mutation_case(analysis, mutation)
                self.assertGreater(case.context.size, 0)
                self.assertGreater(case.query.size, 0)
                self.assertEqual(len(set(case.context.tolist())), case.context.size)
                self.assertEqual(len(set(case.query.tolist())), case.query.size)
                identity = (
                    case.object_id,
                    case.case,
                    tuple(case.context.tolist()),
                    tuple(case.query.tolist()),
                )
                identities.add(identity)
        self.assertEqual(len(identities), 48)

    def test_geometry_serialization_repeats_exactly(self) -> None:
        first = common.geometry_files(self.development_meshes)
        second = common.geometry_files(common.generate_meshes("development"))
        self.assertEqual(first, second)
        self.assertEqual(
            common.canonical_json([mesh.record() for mesh in self.development_meshes]),
            common.canonical_json([mesh.record() for mesh in common.generate_meshes("development")]),
        )

    def test_output_boundary_access_and_dependency_guards(self) -> None:
        environment = common.verify_protocol_environment_and_b0()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES))
        with self.assertRaises(common.O0Error):
            common.prepare_output(common.repository_root() / "forbidden-o0-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            with self.assertRaises(common.O0Error):
                common.prepare_output(Path(temporary))
            symlink = Path(temporary) / "linked-output"
            symlink.symlink_to(common.EXPERIMENT_ROOT, target_is_directory=True)
            with self.assertRaises(common.O0Error):
                common.prepare_output(symlink)


if __name__ == "__main__":
    unittest.main()
