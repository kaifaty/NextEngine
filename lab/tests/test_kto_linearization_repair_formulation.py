from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from next_lab.kto_linearization_repair_formulation import (
    _validate_profile,
    funnel_strictly_decreases,
    select_historical_bridge,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-kto-linearization-repair-formulation-r117.v1.json"
)


class KtoLinearizationRepairFormulationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_report_only_r118(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["qp_solves"], 0)
        self.assertEqual(self.profile["scope"]["kto_solves"], 0)
        self.assertEqual(
            self.profile["decision"]["complete"],
            "PERMIT_R118_KTO_LINEARIZATION_REPAIR_IMPLEMENTATION_CONFORMANCE_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["r116_inverse_dynamics_formulation"],
            "NOT_AUTHORIZED",
        )

    def test_contact_contract_uses_exact_kernel_and_full_derivative(self) -> None:
        contact = self.profile["exact_contact_function"]
        derivative = self.profile["complete_derivative_contract"]
        self.assertIn("velocity_x_m_s^2", contact["tangential_constraint"])
        self.assertNotIn("sqrt(2", contact["tangential_constraint"])
        self.assertTrue(
            any(
                "neighboring orientation" in row
                for row in derivative["configuration_terms"]
            )
        )
        self.assertTrue(
            any("root-linear" in row for row in derivative["velocity_terms"])
        )

        invalid = copy.deepcopy(self.profile)
        invalid["exact_contact_function"]["tangential_constraint"] = "component box"
        with self.assertRaisesRegex(ValueError, "profile differs"):
            _validate_profile(invalid)

    def test_bridge_rule_selects_smallest_isolated_exact_excess(self) -> None:
        result = select_historical_bridge(_fraction_rows())
        self.assertEqual(result["eligible_fraction_count"], 4)
        self.assertEqual(result["selected_fraction"], "1/32")
        self.assertEqual(result["selected_analytic_tangent_excess_micrometres"], 368)
        self.assertEqual(result["selected_tracking_improvement_basis_points"], 615)
        self.assertFalse(result["r115_direction_reused"])

    def test_restoration_funnel_requires_strict_lexicographic_decrease(self) -> None:
        self.assertTrue(funnel_strictly_decreases((0.25, 1.5), (0.20, 4.0)))
        self.assertTrue(funnel_strictly_decreases((0.25, 1.5), (0.25, 1.4)))
        self.assertFalse(funnel_strictly_decreases((0.25, 1.5), (0.25, 1.5)))
        self.assertFalse(funnel_strictly_decreases((0.25, 1.5), (0.26, 1.0)))
        with self.assertRaisesRegex(ValueError, "sum cannot be smaller"):
            funnel_strictly_decreases((0.25, 0.20), (0.10, 0.20))


def _fraction_rows() -> list[dict[str, object]]:
    fractions = ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
    analytic = (16753, 9225, 5498, 3673, 2791, 2368)
    emitted = (
        1_847_946,
        4_287_192_374,
        9_643_656_850,
        13_125_104_123,
        15_066_975_034,
        16_087_896_233,
    )
    rows = []
    for ordinal, fraction in enumerate(fractions):
        isolated = ordinal >= 2
        rows.append(
            {
                "fraction": fraction,
                "status": "FAIL",
                "failure_reasons": ["contact"] if isolated else ["contact", "collider"],
                "contact": {
                    "status": "FAIL",
                    "maximum_normal_residual_micrometres": 4913,
                    "maximum_normal_step_micrometres": 982,
                    "maximum_tangential_step_micrometres": 1982,
                    "maximum_analytic_normal_step_micrometres": 994,
                    "maximum_analytic_tangential_step_micrometres": analytic[ordinal],
                },
                "minimum_collider_height_micrometres": 47 if isolated else -166,
                "maximum_root_vertical_velocity_micrometres_per_second": 199800,
                "maximum_joint_velocity_basis_points": 2500,
                "maximum_descriptor_rom_violation_microradians": 0,
                "maximum_effective_rom_violation_microradians": 0,
                "endpoint_identity": "PASS",
                "tracking_progress": {
                    "status": "PASS",
                    "changed_joint_position_cell_count": 136,
                    "strict_distance_decrease": True,
                    "strict_positive_directional_dot": True,
                    "baseline_squared_distance_microradians_squared": 17_142_428_727,
                    "emitted_squared_distance_microradians_squared": emitted[ordinal],
                },
            }
        )
    return rows


if __name__ == "__main__":
    unittest.main()
