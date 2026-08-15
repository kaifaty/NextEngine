from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.hybrid_contact_edge_state_lift_formulation import (
    MOTOR_INTERVAL_COUNT,
    SUBSTEPS_PER_INTERVAL,
    _active_points_for_modes,
    _validate_profile,
    audit_alternative_decisions,
    audit_contact_edges,
    velocity_lift_endpoint_weights,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-hybrid-contact-edge-state-lift-formulation-r131.v1.json"
)


def _collocations(modes_by_interval: list[list[int]]) -> list[dict[str, object]]:
    rows = []
    for interval, modes in enumerate(modes_by_interval):
        for substep in range(SUBSTEPS_PER_INTERVAL):
            rows.append(
                {
                    "collocation": interval * SUBSTEPS_PER_INTERVAL + substep,
                    "interval": interval,
                    "substep": substep,
                    "contact_modes": modes,
                    "active_point_ordinals": _active_points_for_modes(modes),
                }
            )
    return rows


class HybridContactEdgeStateLiftFormulationTests(unittest.TestCase):
    def test_profile_authorizes_only_r132_report_only_conformance(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r132_edge_lift_conformance"],
            "AUTHORIZED_REPORT_ONLY_ON_R131_COMPLETE",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r132_edge_lift_conformance"
            )
        )

    def test_profile_rejects_missing_execution_prohibition(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        incomplete = deepcopy(profile)
        del incomplete["bounded_acceptance"]["training"]
        with self.assertRaisesRegex(ValueError, "formulation profile differs"):
            _validate_profile(incomplete)

    def test_exit_lift_holds_left_trace_and_ordinary_lift_stays_affine(self) -> None:
        self.assertEqual(
            velocity_lift_endpoint_weights(substep=0, contact_exit=True),
            {"left_numerator": 4, "right_numerator": 0, "denominator": 4},
        )
        self.assertEqual(
            velocity_lift_endpoint_weights(substep=3, contact_exit=True),
            {"left_numerator": 4, "right_numerator": 0, "denominator": 4},
        )
        self.assertEqual(
            velocity_lift_endpoint_weights(substep=3, contact_exit=False),
            {"left_numerator": 1, "right_numerator": 3, "denominator": 4},
        )
        with self.assertRaisesRegex(ValueError, "substep differs"):
            velocity_lift_endpoint_weights(substep=4, contact_exit=True)

    def test_contact_edge_audit_classifies_entry_change_and_exit(self) -> None:
        modes = [[0, 0] for _ in range(MOTOR_INTERVAL_COUNT)]
        modes[10] = [0, 3]
        modes[11] = [0, 2]
        audit = audit_contact_edges(_collocations(modes), hotspot_intervals=[11])
        self.assertEqual(audit["changed_boundary_count"], 3)
        self.assertEqual(audit["entry_intervals"], [9])
        self.assertEqual(audit["active_mode_change_intervals"], [10])
        self.assertEqual(audit["exit_intervals"], [11])
        self.assertEqual(audit["right"]["entry_count"], 1)
        self.assertEqual(audit["right"]["active_mode_change_count"], 1)
        self.assertEqual(audit["right"]["exit_count"], 1)
        self.assertEqual(audit["selected_exit_collocation_rows"], 4)
        self.assertEqual(audit["selected_changed_base_velocity_rows"], 3)

    def test_contact_edge_audit_rejects_substep_mode_drift(self) -> None:
        modes = [[0, 0] for _ in range(MOTOR_INTERVAL_COUNT)]
        rows = _collocations(modes)
        rows[2]["contact_modes"] = [0, 2]
        rows[2]["active_point_ordinals"] = [3]
        with self.assertRaisesRegex(ValueError, "interval contact ownership"):
            audit_contact_edges(rows, hotspot_intervals=[])

    def test_alternative_matrix_selects_one_bounded_discriminator(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_alternative_decisions(profile)
        self.assertEqual(audit["alternatives_compared"], 5)
        self.assertEqual(
            audit["selected_alternative"],
            "exit_mode_owned_left_velocity_trace_hold",
        )
        changed = deepcopy(profile)
        changed["alternative_decisions"][2]["decision"] = (
            "SELECT_R132_BOUNDED_CONFORMANCE"
        )
        with self.assertRaisesRegex(ValueError, "alternative decision matrix"):
            audit_alternative_decisions(changed)


if __name__ == "__main__":
    unittest.main()
