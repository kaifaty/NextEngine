from __future__ import annotations

import copy
import json
import math
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v35_c0_witness_and_coverage_census_v1 as c0


class WitnessAndCoverageCensusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.overlay, cls.effective, p0_data = c0.load_context(PROFILE)
        cls.p0 = json.loads(p0_data)

    def test_dependencies_and_owner_import_boundary_are_target_safe(self) -> None:
        boundary = c0.validate_owner_import_boundary()
        self.assertEqual(boundary["forbidden_imports"], [])
        self.assertIn("physical_sound_v31_p1_modal_owner_v1", boundary["imports"])
        self.assertIn(
            "physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1",
            boundary["imports"],
        )
        self.assertEqual(self.overlay["witness_contract"]["target_rows_allowed"], 0)
        self.assertEqual(c0.ZERO_FORBIDDEN_ACCESS["oracle_values_evaluated"], 0)
        self.assertEqual(c0.ZERO_FORBIDDEN_ACCESS["model_parameters_initialized"], 0)
        self.assertEqual(
            c0.ZERO_FORBIDDEN_ACCESS["prior_generation_metric_values_read"], 0
        )

    def test_discarded_p1_boundary_fixtures_have_exact_nodes(self) -> None:
        for family_index, family in enumerate(self.effective["corpus"]["families"]):
            fixture = c0.base_fixture_for_family(family, self.p0)
            fixture["fixture_id"] = f"discarded-v35-c0-f{family_index}"
            fixture["contact"] = {
                "normal_impulse_ns": "1",
                "u": "0",
                "v": "0.333",
            }
            solution = c0.p1.solve_case(fixture["fixture_id"], fixture, self.p0)
            modes = solution["modal_document"]["modes"]
            self.assertEqual(len(modes), 10)
            self.assertTrue(all(mode["contact_participation"] == 0.0 for mode in modes))
            self.assertTrue(all(mode["signed_gain"] == 0.0 for mode in modes))
            self.assertTrue(solution["metrics"]["remesh_common_vertices_exact"])

    def test_fixture_local_key_and_identity_are_deterministic(self) -> None:
        arguments = (
            self.effective,
            self.p0,
            "development",
            "joint",
            0,
            0,
            6,
            "development",
            0,
        )
        case_a, fixture_a, multipliers_a = c0.build_fixture(*arguments)
        case_b, fixture_b, multipliers_b = c0.build_fixture(*arguments)
        self.assertEqual(
            (case_a, fixture_a, multipliers_a), (case_b, fixture_b, multipliers_b)
        )
        solution = c0.p1.solve_case(case_a, fixture_a, self.p0)
        stencil = c0.structural_stencil(solution, self.p0, 0.0625)
        key_a = c0.local_key(solution, fixture_a, multipliers_a, 0, stencil)
        key_b = c0.local_key(solution, fixture_a, multipliers_a, 0, stencil)
        self.assertEqual(key_a, key_b)
        self.assertEqual(len(key_a), 15)
        self.assertTrue(all(math.isfinite(value) for value in key_a))
        commitment = c0.id_commitment(["row-b", "row-a"])
        self.assertEqual(commitment, c0.id_commitment(["row-a", "row-b"]))
        with self.assertRaisesRegex(c0.C0CensusError, "duplicated"):
            c0.id_commitment(["row-a", "row-a"])

    def test_target_free_bandwidth_and_gate_keep_both_experts_reachable(self) -> None:
        corpus = self.effective["corpus"]
        train_geometry = [
            tuple(math.log(float(value)) / math.log(1.25) for value in cell)
            for cell in corpus["geometry_multiplier_cells"][:6]
        ]
        train_contacts = [
            tuple(2.0 * float(value) - 1.0 for value in pair)
            for pair in corpus["contacts"]["train"]
        ]
        geometry_bandwidth = c0.f0.median_positive_nearest_distance(train_geometry)
        contact_bandwidth = c0.f0.median_positive_nearest_distance(train_contacts)
        self.assertEqual(geometry_bandwidth.hex(), "0x1.ac6db4c237254p-1")
        self.assertEqual(contact_bandwidth.hex(), "0x1.e6d4df96cc6b2p-2")
        values = c0.coverage_gate_values(
            train_geometry[0],
            train_contacts[0],
            train_geometry,
            train_contacts,
            geometry_bandwidth,
            contact_bandwidth,
        )
        self.assertEqual(values, (0.0, 0.8, 0.19999999999999996))
        self.assertEqual(c0.squared_distance((1.0, 2.0), (1.0, 2.0)), 0.0)

    def test_forbidden_role_fields_and_occupied_output_fail_closed(self) -> None:
        forbidden = set(self.overlay["method_overlay"]["features"]["forbidden_fields"])
        inputs = set(
            self.overlay["method_overlay"]["local_expert"]["distance_key"]
            + self.overlay["method_overlay"]["coverage_gate"]["contact_key"]
            + self.overlay["method_overlay"]["coverage_gate"]["geometry_key"]
        )
        self.assertEqual(forbidden & inputs, set())
        with self.assertRaisesRegex(c0.C0CensusError, "role access forbidden"):
            c0.collect_role("forbidden", self.overlay, self.effective, self.p0)
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v35-c0-output-"
        ) as temporary:
            occupied = Path(temporary) / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(c0.C0CensusError, "fresh external path"):
                c0.external_output(occupied)

    def test_invalid_structural_fixture_rejects_without_target_path(self) -> None:
        family = self.effective["corpus"]["families"][0]
        fixture = copy.deepcopy(c0.base_fixture_for_family(family, self.p0))
        fixture["fixture_id"] = "discarded-invalid-v35-c0"
        fixture["contact"]["u"] = "2"
        with self.assertRaises(c0.p1.OutOfDomain):
            c0.p1.solve_case(fixture["fixture_id"], fixture, self.p0)


if __name__ == "__main__":
    unittest.main()
