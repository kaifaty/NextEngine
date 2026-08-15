from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.projected_inverse_dynamics_composition_conformance import (
    _validate_profile,
    audit_composition_api,
    audit_synthetic_block_composition,
    audit_synthetic_hash_guards,
    audit_synthetic_rank_and_cones,
    audit_warm_scale_independence,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-projected-inverse-dynamics-composition-conformance-r135.v1.json"
)


class ProjectedInverseDynamicsCompositionConformanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_conditional_r136_execution(self) -> None:
        _validate_profile(self.profile)
        bounded = self.profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r136_projected_inverse_dynamics_execution"],
            "AUTHORIZED_ONE_EXECUTION_ONLY_ON_R135_PASS",
        )
        self.assertTrue(
            all(
                value == "NOT_AUTHORIZED"
                for key, value in bounded.items()
                if key != "r136_projected_inverse_dynamics_execution"
            )
        )

    def test_profile_rejects_real_conformance_work(self) -> None:
        invalid = deepcopy(self.profile)
        invalid["scope"]["real_inverse_dynamics_system_assemblies"] = 1
        with self.assertRaisesRegex(ValueError, "conformance profile differs"):
            _validate_profile(invalid)

    def test_composition_api_and_block_oracles_pass(self) -> None:
        self.assertEqual(audit_composition_api()["status"], "PASS")
        block = audit_synthetic_block_composition()
        self.assertEqual(block["status"], "PASS")
        self.assertEqual(block["case_count"], 3)

    def test_warm_acceleration_is_scale_only(self) -> None:
        audit = audit_warm_scale_independence()
        self.assertEqual(audit["status"], "PASS")
        self.assertTrue(audit["matrix_byte_identical_across_warm_scale_inputs"])
        self.assertTrue(
            audit["right_hand_side_byte_identical_across_warm_scale_inputs"]
        )
        self.assertTrue(audit["acceleration_scale_changes"])

    def test_rank_cone_and_failure_paths_pass(self) -> None:
        audit = audit_synthetic_rank_and_cones(self.profile["numeric_contract"])
        self.assertEqual(audit["status"], "PASS")
        self.assertEqual(audit["singular_value_decompositions"], 7)
        cases = {row["case"]: row for row in audit["cases"]}
        self.assertEqual(cases["flat_gauge_feasible"]["feasibility"], "FEASIBLE")
        self.assertEqual(cases["flat_gauge_infeasible"]["feasibility"], "INFEASIBLE")
        self.assertEqual(cases["rank_gap_invalid"]["analysis_status"], "INVALID")
        self.assertEqual(
            cases["inconsistent_rhs_invalid"]["analysis_status"], "INVALID"
        )

    def test_hash_guards_use_synthetic_arrays_only(self) -> None:
        audit = audit_synthetic_hash_guards()
        self.assertEqual(audit["status"], "PASS")
        self.assertEqual(audit["case_count"], 2)
        self.assertEqual(audit["real_r133_array_values_read"], 0)


if __name__ == "__main__":
    unittest.main()
