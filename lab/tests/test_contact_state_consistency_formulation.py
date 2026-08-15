from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.contact_state_consistency_formulation import (
    _validate_profile,
    audit_alternative_decisions,
    audit_projection_inventory,
    canonical_json,
    projection_system_layout,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-contact-state-consistency-r128.v1.json"


class ContactStateConsistencyFormulationTests(unittest.TestCase):
    def test_profile_authorizes_only_r129_report_only_conformance(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r129_projection_conformance"],
            "AUTHORIZED_REPORT_ONLY_ON_R128_COMPLETE",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r129_projection_conformance"
            )
        )

    def test_projection_layout_retains_flat_multiplier_gauge(self) -> None:
        self.assertEqual(
            projection_system_layout(active_point_count=0, same_body_flat=False),
            {
                "active_point_count": 0,
                "constraint_row_count": 0,
                "expected_independent_rank": 0,
                "expected_multiplier_nullity": 0,
                "projection_kkt_width": 29,
                "projected_velocity_width": 29,
            },
        )
        single = projection_system_layout(active_point_count=1, same_body_flat=False)
        flat = projection_system_layout(active_point_count=2, same_body_flat=True)
        self.assertEqual(
            (single["expected_independent_rank"], single["projection_kkt_width"]),
            (3, 32),
        )
        self.assertEqual(
            (
                flat["expected_independent_rank"],
                flat["expected_multiplier_nullity"],
                flat["projection_kkt_width"],
            ),
            (5, 1, 35),
        )

    def test_projection_inventory_closes_r121(self) -> None:
        inventory = audit_projection_inventory(_r121_stub())
        aggregate = inventory["aggregate"]
        self.assertEqual(aggregate["collocations"], 3200)
        self.assertEqual(aggregate["flight_collocations"], 560)
        self.assertEqual(aggregate["single_point_collocations"], 324)
        self.assertEqual(aggregate["flat_foot_collocations"], 2316)
        self.assertEqual(aggregate["projection_systems"], 2640)
        self.assertEqual(aggregate["constraint_rows"], 14868)
        self.assertEqual(aggregate["expected_independent_rank"], 12552)
        self.assertEqual(aggregate["redundant_multiplier_gauge_scalars"], 2316)
        self.assertEqual(aggregate["maximum_projection_kkt_width"], 35)

    def test_alternative_matrix_has_one_selected_repair(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_alternative_decisions(profile)
        self.assertEqual(
            audit["selected_alternative"],
            "mass_metric_tangent_velocity_projection",
        )
        changed = deepcopy(profile)
        changed["alternative_decisions"][1]["decision"] = "SELECTED_STAGE2_DIAGNOSTIC"
        with self.assertRaisesRegex(ValueError, "alternative decision matrix"):
            audit_alternative_decisions(changed)

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
        "system_inventory_audit": {
            "active_contact_acceleration_closure_row_count": 14868
        },
    }


if __name__ == "__main__":
    unittest.main()
