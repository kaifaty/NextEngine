from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from next_lab.repaired_kto_execution_formulation import (
    _validate_profile,
    audit_execution_budget,
    project_anchor_contact_rows,
    repaired_contact_row_inventory,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-repaired-kto-execution-formulation-r119.v1.json"


class RepairedKtoExecutionFormulationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_one_named_r120_execution(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["qp_solves"], 0)
        self.assertEqual(self.profile["scope"]["kto_solves"], 0)
        self.assertEqual(
            self.profile["decision"]["complete"],
            "PERMIT_R120_SINGLE_BOUNDED_REPAIRED_KTO_EXECUTION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )
        self.assertEqual(self.profile["bounded_acceptance"]["physx"], "NOT_AUTHORIZED")

    def test_repaired_rows_replace_three_components_with_normal_and_norm(self) -> None:
        inventory = repaired_contact_row_inventory(1241)
        self.assertEqual(inventory["retired_component_row_count"], 3723)
        self.assertEqual(inventory["normal_row_count"], 1241)
        self.assertEqual(inventory["tangential_norm_squared_row_count"], 1241)
        self.assertEqual(inventory["replacement_row_count"], 2482)
        self.assertEqual(inventory["resulting_total_constraint_row_count"], 129939)

    def test_anchor_projection_applies_the_norm_squared_chain_rule(self) -> None:
        anchor = _synthetic_anchor()
        projection = project_anchor_contact_rows(anchor)
        self.assertEqual(projection["status"], "PASS")
        self.assertAlmostEqual(
            projection["tangential_baseline_square_metres_per_square_second"],
            0.0104,
        )
        self.assertAlmostEqual(
            projection["tangential_upper_rhs_square_metres_per_square_second"],
            0.004,
        )
        self.assertEqual(projection["tangential_configuration_nonzero_count"], 1)
        self.assertEqual(projection["tangential_velocity_nonzero_count"], 1)

    def test_budget_is_one_execution_with_twelve_bounded_subproblems(self) -> None:
        audit = audit_execution_budget(self.profile["execution_budget"])
        self.assertEqual(audit["status"], "PASS")
        self.assertEqual(audit["maximum_qp_solves"], 12)
        self.assertEqual(audit["derived_maximum_exact_emission_audits"], 72)
        self.assertTrue(audit["one_outer_execution_not_twelve_kto_solves"])

    def test_profile_rejects_component_box_or_v9_only_rebase(self) -> None:
        invalid = copy.deepcopy(self.profile)
        invalid["repaired_contact_row_contract"]["tangential_function"] = (
            "abs(vx) <= bound and abs(vz) <= bound"
        )
        with self.assertRaisesRegex(ValueError, "profile differs"):
            _validate_profile(invalid)
        invalid = copy.deepcopy(self.profile)
        invalid["anchor_rebase_contract"]["zero_perturbation_identity"] = (
            "always reconstruct the V9 stencil"
        )
        with self.assertRaisesRegex(ValueError, "profile differs"):
            _validate_profile(invalid)


def _synthetic_anchor() -> dict[str, object]:
    rows: list[dict[str, object]] = [
        {
            "block": "configuration",
            "production_xyz_micrometres_per_second_per_unit": [
                1_000_000.0,
                0.0,
                0.0,
            ],
        },
        {
            "block": "velocity",
            "production_xyz_micrometres_per_second_per_unit": [
                0.0,
                1_000_000.0,
                1_000_000.0,
            ],
        },
    ]
    rows.extend(
        {
            "block": "velocity",
            "production_xyz_micrometres_per_second_per_unit": [0.0, 0.0, 0.0],
        }
        for _ in range(59)
    )
    return {
        "status": "PASS",
        "point_id": "frame-328:left-forefoot",
        "production_baseline_velocity_micrometres_per_second": [
            100_000.0,
            10_000.0,
            20_000.0,
        ],
        "variable_rows": rows,
    }


if __name__ == "__main__":
    unittest.main()
