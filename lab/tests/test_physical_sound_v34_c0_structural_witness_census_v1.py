from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = (
    LAB / "profiles" / "physical-sound-v34-f0-target-safe-spectral-recovery.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v34_c0_structural_witness_census_v1 as c0


class StructuralWitnessCensusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.overlay, cls.effective, p0_data = c0.load_context(PROFILE)
        cls.p0 = json.loads(p0_data)

    def test_dependencies_and_owner_import_boundary_are_target_safe(self) -> None:
        boundary = c0.validate_owner_import_boundary()
        self.assertEqual(boundary["forbidden_imports"], [])
        self.assertIn("physical_sound_v31_p1_modal_owner_v1", boundary["imports"])
        self.assertIn(
            "physical_sound_v34_f0_target_safe_profile_freeze_v1",
            boundary["imports"],
        )
        self.assertEqual(self.overlay["witness_contract"]["target_rows_allowed"], 0)
        self.assertEqual(c0.ZERO_FORBIDDEN_ACCESS["oracle_values_evaluated"], 0)
        self.assertEqual(c0.ZERO_FORBIDDEN_ACCESS["model_parameters_initialized"], 0)

    def test_discarded_p1_boundary_fixtures_have_exact_nodes(self) -> None:
        for family_index, family in enumerate(self.effective["corpus"]["families"]):
            fixture = c0.base_fixture_for_family(family, self.p0)
            fixture["fixture_id"] = f"discarded-c0-f{family_index}"
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

    def test_fixture_builder_and_identity_commitment_are_deterministic(self) -> None:
        args = (
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
        case_a, fixture_a = c0.build_fixture(*args)
        case_b, fixture_b = c0.build_fixture(*args)
        self.assertEqual(case_a, case_b)
        self.assertEqual(fixture_a, fixture_b)
        self.assertEqual(fixture_a["contact"]["u"], "0")
        commitment = c0.id_commitment(["row-b", "row-a"])
        self.assertEqual(commitment["count"], 2)
        self.assertEqual(commitment["first"], "row-a")
        self.assertEqual(commitment["last"], "row-b")
        self.assertEqual(commitment, c0.id_commitment(["row-a", "row-b"]))
        with self.assertRaisesRegex(c0.C0CensusError, "duplicated"):
            c0.id_commitment(["row-a", "row-a"])

    def test_forbidden_role_and_occupied_output_fail_before_census(self) -> None:
        with self.assertRaisesRegex(c0.C0CensusError, "role access forbidden"):
            c0.census_role("train", self.overlay, self.effective, self.p0)
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v34-c0-output-"
        ) as temporary:
            occupied = Path(temporary) / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(c0.C0CensusError, "fresh external path"):
                c0.external_output(occupied)

    def test_invalid_structural_fixture_rejects_without_target_path(self) -> None:
        family = self.effective["corpus"]["families"][0]
        fixture = c0.base_fixture_for_family(family, self.p0)
        fixture = copy.deepcopy(fixture)
        fixture["fixture_id"] = "discarded-invalid-c0"
        fixture["contact"]["u"] = "2"
        with self.assertRaises(c0.p1.OutOfDomain):
            c0.p1.solve_case(fixture["fixture_id"], fixture, self.p0)


if __name__ == "__main__":
    unittest.main()
