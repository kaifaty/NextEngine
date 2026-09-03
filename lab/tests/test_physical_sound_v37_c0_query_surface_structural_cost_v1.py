from __future__ import annotations

import copy
import math
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = (
    LAB / "profiles" / "physical-sound-v37-c0-query-surface-structural-cost.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0


class QuerySurfaceStructuralCostTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.context = c0.load_context(PROFILE)
        cls.roles = {role: c0.build_role(cls.context, role) for role in c0.ROLE_NAMES}

    def test_profile_dependency_and_import_boundaries_are_target_free(self) -> None:
        self.assertEqual(c0.sha256_bytes(self.context.profile_data), c0.PROFILE_SHA256)
        self.assertEqual(self.context.profile["authority"], c0.EXPECTED_AUTHORITY)
        self.assertEqual(
            set(self.context.dependencies),
            {row["path"] for row in self.context.profile["dependencies"]},
        )
        boundary = c0.validate_profile_boundary(self.context.profile)
        self.assertEqual(boundary["forbidden_profile_section_intersection"], [])
        imports = c0.validate_import_boundary(self.context.profile)
        self.assertEqual(imports["forbidden_imports"], [])
        self.assertTrue(all(value == 0 for value in c0.ZERO_FORBIDDEN_ACCESS.values()))

    def test_f0_case_field_and_row_commitments_close_for_every_role(self) -> None:
        expected = self.context.profile["expected_role_commitments"]
        for role, built in self.roles.items():
            batch = built.batch
            commitment = expected[role]
            self.assertEqual(len(built.case_ids), commitment["case_count"])
            self.assertEqual(
                c0.line_root(built.case_ids), commitment["case_root_sha256"]
            )
            self.assertEqual(batch.fields.field_count, commitment["field_count"])
            self.assertEqual(
                c0.f0.merkle_root(set(batch.fields.field_ids)),
                commitment["field_root_sha256"],
            )
            self.assertEqual(batch.row_count, commitment["modal_row_count"])
            self.assertEqual(
                c0.f0.merkle_root(set(batch.row_ids)),
                commitment["modal_row_root_sha256"],
            )
            self.assertFalse(batch.has_targets)

    def test_full_shape_query_remesh_permutation_and_topology_close(self) -> None:
        expected_totals = self.context.profile["structural"]["expected_totals"]
        census = c0.structural_census(self.context, dict(self.roles))
        self.assertEqual(census["totals"], expected_totals)
        self.assertTrue(
            all(value == 0 for value in census["role_row_intersections"].values())
        )
        for role in c0.ROLE_NAMES:
            record = census["roles"][role]
            self.assertEqual(record["query_support_count"], record["modal_row_count"])
            self.assertEqual(
                record["field_structural_root_sha256"], record["remesh_root_sha256"]
            )
            self.assertEqual(
                record["query_structural_root_sha256"],
                record["permutation_root_sha256"],
            )
            self.assertFalse(record["targets_present"])
        self.assertEqual(census["structural_traces"]["d0"]["terminal"], "PreflightPass")
        self.assertEqual(census["structural_traces"]["h0"]["terminal"], "PreflightPass")

    def test_surface_envelope_and_query_support_preserve_physical_bounds(self) -> None:
        coordinates = np.linspace(0.0, 1.0, 101, dtype=np.float64)
        u, v = np.meshgrid(coordinates, coordinates, indexing="xy")
        for seed in c0.surface_seed_map(self.context.f0_profile).values():
            envelope = c0.surface_envelope(seed, u.ravel(), v.ravel())
            self.assertGreaterEqual(float(np.min(envelope)), 0.91 - 1.0e-12)
            self.assertLessEqual(float(np.max(envelope)), 1.09 + 1.0e-12)
        for u_value, v_value in ((0.0, 0.0), (1.0, 1.0), (0.5, 0.5), (0.37, 0.91)):
            _triangle, barycentrics = c0.query_support(u_value, v_value)
            self.assertTrue(all(0.0 <= value <= 1.0 for value in barycentrics))
            self.assertTrue(math.isclose(sum(barycentrics), 1.0, abs_tol=1.0e-15))

    def test_tensor_shapes_parameter_counts_and_ablation_paths_are_reachable(
        self,
    ) -> None:
        train = c0.tensor_role(self.roles["train"])
        self.assertEqual(tuple(train.node_features.shape), (3240, 9, 22))
        self.assertEqual(tuple(train.query_features.shape), (6480, 24))
        self.assertEqual(tuple(train.pointwise_features.shape), (6480, 35))
        self.assertEqual(tuple(train.query_kernel_weights.shape), (6480, 9))
        self.assertTrue(
            torch.allclose(
                torch.sum(train.query_kernel_weights, dim=1),
                torch.ones(6480, dtype=torch.float64),
                rtol=0.0,
                atol=1.0e-15,
            )
        )
        self.assertEqual(c0.parameter_count(c0.QuerySurfaceCostModel(370201)), 12443)
        self.assertEqual(c0.parameter_count(c0.PointwiseCostModel(370202)), 1859)
        self.assertEqual(
            c0.ablation_reachability(train),
            {
                "field_branch": "Reachable",
                "field_query_interaction": "Reachable",
                "query_trunk": "Reachable",
                "topology_propagation": "Reachable",
            },
        )

    def test_bounded_artificial_workload_executes_without_scientific_values(
        self,
    ) -> None:
        tensor_roles = {
            role: c0.tensor_role(built) for role, built in self.roles.items()
        }
        result = c0.train_qso_cost(
            "query-conditioned-surface-operator-v0",
            370201,
            tensor_roles,
            steps=1,
            batch_rows=64,
            microbatch_rows=32,
        )
        self.assertEqual(result["steps"], 1)
        self.assertEqual(result["parameter_count"], 12443)
        self.assertLessEqual(result["field_cache_peak_bytes"], 33554432)
        self.assertEqual(len(result["artificial_output_sha256"]), 64)
        controls = c0.non_neural_cost(tensor_roles)
        self.assertEqual(
            set(controls),
            {
                "continuous-local-interpolation-v1",
                "fixed-rbf-integral-ridge-v1",
                "nearest-causal-surface-query-v1",
            },
        )

    def test_value_profile_and_identity_mutations_fail_closed(self) -> None:
        mutated = copy.deepcopy(self.context.profile)
        mutated["targets"] = [[0.0, 0.0, 0.0]]
        with self.assertRaisesRegex(c0.C0StructuralCostError, "value-bearing"):
            c0.validate_profile_boundary(mutated)
        with self.assertRaisesRegex(c0.C0StructuralCostError, "unique nonempty"):
            c0.line_root(("duplicate", "duplicate"))
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-c0-mutation-") as temp:
            path = Path(temp) / "mutated.json"
            changed = copy.deepcopy(self.context.profile)
            changed["cost_oracle"]["batch_rows"] = 1023
            path.write_bytes(c0.canonical_json(changed))
            with self.assertRaisesRegex(c0.C0StructuralCostError, "profile hash"):
                c0.load_profile(path)

    def test_external_output_is_fresh_and_outside_repository(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-c0-output-") as temp:
            root = Path(temp)
            fresh = root / "fresh"
            self.assertEqual(c0.external_output(fresh), fresh)
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(c0.C0StructuralCostError, "fresh external"):
                c0.external_output(occupied)


if __name__ == "__main__":
    unittest.main()
