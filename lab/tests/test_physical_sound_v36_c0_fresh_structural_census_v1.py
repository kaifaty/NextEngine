from __future__ import annotations

import copy
import math
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v36-c0-fresh-structural-census.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v36_c0_fresh_structural_census_v1 as c0


class FreshStructuralCensusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        (
            cls.profile,
            cls.overlay,
            cls.effective,
            cls.p0,
            cls.profile_data,
        ) = c0.load_context(PROFILE)

    def test_profile_dependencies_and_import_boundary_are_truth_target_safe(
        self,
    ) -> None:
        self.assertEqual(c0.sha256_bytes(self.profile_data), c0.PROFILE_SHA256)
        self.assertEqual(self.profile["authority"], c0.EXPECTED_AUTHORITY)
        self.assertEqual(
            set(c0.validate_dependencies(self.profile)),
            set(self.profile["parent"]) | {"protocol"},
        )
        boundary = c0.validate_structural_profile_boundary(self.profile)
        self.assertEqual(boundary["forbidden_top_level_intersection"], [])
        self.assertFalse(boundary["profile_contains_oracle_section"])
        imports = c0.validate_owner_import_boundary(self.profile)
        self.assertEqual(imports["forbidden_imports"], [])
        self.assertEqual(imports["reused_baseline_forbidden_imports"], [])
        self.assertIn("physical_sound_v31_p1_modal_owner_v1", imports["imports"])
        self.assertEqual(self.overlay["witness_contract"]["target_rows_allowed"], 0)
        self.assertTrue(all(value == 0 for value in c0.ZERO_FORBIDDEN_ACCESS.values()))

    def test_forbidden_value_section_is_rejected_even_before_profile_hash(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["oracle"] = {"contact": {"expression": "forbidden"}}
        with self.assertRaisesRegex(c0.C0CensusError, "forbidden structural"):
            c0.validate_structural_profile_boundary(mutated)

    def test_f0_case_commitment_roots_reproduce_without_solving_targets(self) -> None:
        corpus = self.effective["corpus"]
        for role in c0.ROLES:
            identities: list[str] = []
            for entry in corpus["role_plan"][role]:
                for family in corpus["families"]:
                    for material in corpus["materials"]:
                        for geometry_index in entry["geometry_cells"]:
                            geometry = corpus["geometry_multiplier_cells"][
                                geometry_index
                            ]
                            for contact in corpus["contacts"][entry["contact_set"]]:
                                identities.append(
                                    c0.case_identity(
                                        self.profile,
                                        role,
                                        entry["stratum"],
                                        family,
                                        material,
                                        geometry,
                                        tuple(contact),
                                    )
                                )
            expected = self.profile["f0_commitments"][role]
            self.assertEqual(len(identities), expected["case_count"])
            self.assertEqual(c0.line_root(identities), expected["case_root_sha256"])

    def test_fresh_fixture_local_key_and_p1_node_are_deterministic(self) -> None:
        arguments = (
            self.profile,
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
        modes = solution["modal_document"]["modes"]
        self.assertEqual(len(modes), 10)
        self.assertTrue(all(mode["contact_participation"] == 0.0 for mode in modes))
        self.assertTrue(all(mode["signed_gain"] == 0.0 for mode in modes))
        stencil = c0.baseline.structural_stencil(solution, self.p0, 0.0625)
        key = c0.baseline.local_key(solution, fixture_a, multipliers_a, 0, stencil)
        self.assertEqual(len(key), 15)
        self.assertTrue(all(math.isfinite(value) for value in key))
        self.assertTrue(solution["metrics"]["remesh_common_vertices_exact"])

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
        geometry_bandwidth = c0.baseline.f0.median_positive_nearest_distance(
            train_geometry
        )
        contact_bandwidth = c0.baseline.f0.median_positive_nearest_distance(
            train_contacts
        )
        self.assertEqual(geometry_bandwidth.hex(), "0x1.bdf15691df66ap-1")
        self.assertEqual(contact_bandwidth.hex(), "0x1.0000000000000p-1")
        self.assertEqual(
            c0.baseline.coverage_gate_values(
                train_geometry[0],
                train_contacts[0],
                train_geometry,
                train_contacts,
                geometry_bandwidth,
                contact_bandwidth,
            ),
            (0.0, 0.8, 0.19999999999999996),
        )

    def test_wrong_role_and_duplicate_identity_fail_closed(self) -> None:
        with self.assertRaisesRegex(c0.C0CensusError, "role access forbidden"):
            c0.build_fixture(
                self.profile,
                self.effective,
                self.p0,
                "forbidden",
                "joint",
                0,
                0,
                0,
                "train",
                0,
            )
        with self.assertRaisesRegex(c0.C0CensusError, "unique nonempty"):
            c0.line_root(["same", "same"])

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-c0-failure-"
        ) as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["corpus"]["contacts"]["train"][0][1] = "0.38"
            bad.write_bytes(c0.canonical_json(profile))
            with self.assertRaisesRegex(c0.C0CensusError, "profile hash mismatch"):
                c0.load_profile(bad)
            output = root / "occupied"
            output.mkdir()
            with self.assertRaisesRegex(c0.C0CensusError, "fresh external path"):
                c0.external_output(output)


if __name__ == "__main__":
    unittest.main()
