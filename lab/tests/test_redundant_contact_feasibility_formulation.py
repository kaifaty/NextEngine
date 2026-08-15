from __future__ import annotations

import json
import unittest
from pathlib import Path

from next_lab.redundant_contact_feasibility_formulation import (
    _validate_profile,
    audit_mode_inventory,
    canonical_json,
    scaled_line_cone_coefficients,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-redundant-contact-feasibility-formulation-r125.v1.json"
)


class RedundantContactFeasibilityFormulationTests(unittest.TestCase):
    def test_profile_authorizes_only_r126_report_only_conformance(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r126_implementation_conformance"],
            "AUTHORIZED_REPORT_ONLY_ON_R125_COMPLETE",
        )
        self.assertTrue(
            all(
                value == "NOT_AUTHORIZED"
                for key, value in bounded.items()
                if key != "r126_implementation_conformance"
            )
        )

    def test_mode_inventory_eliminates_inactive_force_and_effort_variables(
        self,
    ) -> None:
        inventory = audit_mode_inventory(_r121_stub())
        self.assertEqual(inventory["status"], "PASS")
        aggregate = inventory["aggregate"]
        self.assertEqual(aggregate["collocations"], 3200)
        self.assertEqual(aggregate["flight_collocations"], 560)
        self.assertEqual(aggregate["single_point_collocations"], 324)
        self.assertEqual(aggregate["flat_foot_collocations"], 2316)
        self.assertEqual(aggregate["friction_cones"], 4956)
        self.assertEqual(aggregate["reduced_decision_scalars"], 107668)
        self.assertEqual(aggregate["algebraic_equality_rows"], 107668)
        self.assertEqual(aggregate["expected_independent_equality_rank"], 105352)
        self.assertEqual(aggregate["exact_force_gauge_scalars"], 2316)
        self.assertEqual(aggregate["maximum_reduced_local_unknown_count"], 35)
        self.assertEqual(aggregate["maximum_flat_foot_gauge_dimension"], 1)

    def test_mode_inventory_rejects_unregistered_mode(self) -> None:
        source = _r121_stub()
        source["contact_schedule_audit"]["mode_counts_over_motor_intervals"][
            "heel_flight"
        ] = 1
        with self.assertRaisesRegex(ValueError, "source mode inventory"):
            audit_mode_inventory(source)

    def test_line_cone_coefficients_preserve_exact_rational_friction(self) -> None:
        coefficients = scaled_line_cone_coefficients(
            force_nrf=(10, 1, 2), direction_nrf=(0, 0, 1)
        )
        den2 = 65536**2
        num2 = 52429**2
        self.assertEqual(coefficients["quadratic"], den2)
        self.assertEqual(coefficients["linear"], 4 * den2)
        self.assertEqual(coefficients["constant"], 5 * den2 - 100 * num2)
        self.assertEqual(coefficients["normal_intercept"], 10)
        self.assertEqual(coefficients["normal_slope"], 0)

    def test_canonical_json_is_order_independent(self) -> None:
        self.assertEqual(canonical_json({"b": 2, "a": 1}), b'{"a":1,"b":2}')


def _r121_stub() -> dict[str, object]:
    return {
        "contact_schedule_audit": {
            "active_point_collocation_count": 4956,
            "mode_counts_over_motor_intervals": {
                "flat_flight": 302,
                "flight_flat": 277,
                "flight_flight": 140,
                "flight_forefoot": 43,
                "forefoot_flight": 38,
            },
        },
        "equation_contract": {"friction_second_order_cone_count": 4956},
    }


if __name__ == "__main__":
    unittest.main()
