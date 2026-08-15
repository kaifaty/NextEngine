from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.fixed_mode_kinodynamic_graph_conformance import (
    _validate_profile,
    assemble_step_layout,
    audit_index_map,
    audit_synthetic_controller_cases,
    audit_synthetic_layout_cases,
    classify_point_transition,
    physics_address,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-fixed-mode-kinodynamic-graph-conformance-r138.v1.json"
)
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class FixedModeKinodynamicGraphConformanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())
        cls.descriptor = json.loads(DESCRIPTOR.read_bytes())

    def test_profile_authorizes_only_r139_report_only_formulation(self) -> None:
        _validate_profile(self.profile)
        bounded = self.profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r139_kinodynamic_solve_formulation"],
            "AUTHORIZED_REPORT_ONLY_ON_R138_PASS",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r139_kinodynamic_solve_formulation"
            )
        )

    def test_profile_rejects_real_solve_authority(self) -> None:
        invalid = deepcopy(self.profile)
        invalid["bounded_acceptance"]["kinodynamic_solve"] = "AUTHORIZED"
        with self.assertRaisesRegex(ValueError, "conformance profile differs"):
            _validate_profile(invalid)

    def test_motor_to_physics_address_is_total_and_target_held_four_steps(
        self,
    ) -> None:
        self.assertEqual(
            physics_address(0, 0),
            {
                "motor_interval": 0,
                "substep": 0,
                "physics_interval": 0,
                "left_state_node": 0,
                "right_state_node": 1,
                "command_target_row": 0,
            },
        )
        self.assertEqual(physics_address(799, 3)["right_state_node"], 3200)
        audit = audit_index_map()
        self.assertEqual(audit["unique_physics_intervals"], 3200)
        self.assertEqual(audit["physics_steps_per_target"], 4)
        with self.assertRaisesRegex(ValueError, "address differs"):
            physics_address(800, 0)

    def test_layout_cases_close_force_activation_and_release_ownership(self) -> None:
        audit = audit_synthetic_layout_cases(self.profile)
        self.assertEqual(audit["passing_case_count"], 6)
        self.assertEqual(audit["rejected_malformed_case_count"], 1)
        add_heel = assemble_step_layout([2, 0], [3, 0])
        self.assertEqual(add_heel["activated_point_ordinals"], [0])
        self.assertEqual(add_heel["continuous_force_scalars"], 3)
        self.assertEqual(add_heel["activation_impulse_scalars"], 3)
        remove_heel = assemble_step_layout([3, 0], [2, 0])
        self.assertEqual(remove_heel["deactivated_point_ordinals"], [0])
        self.assertEqual(remove_heel["release_impulse"], "EXACT_ZERO")
        self.assertEqual(remove_heel["activation_impulse_scalars"], 0)

    def test_transition_rejects_release_and_activation_on_same_boundary(self) -> None:
        with self.assertRaisesRegex(ValueError, "activates and releases together"):
            classify_point_transition([2, 0], [0, 2])

    def test_free_effort_inactive_force_and_unscheduled_impulse_are_absent(
        self,
    ) -> None:
        for from_modes, to_modes in (([0, 0], [0, 0]), ([0, 0], [0, 3])):
            layout = assemble_step_layout(from_modes, to_modes)
            self.assertEqual(layout["free_effort_scalars"], 0)
            self.assertEqual(layout["inactive_force_scalars"], 0)
            self.assertEqual(layout["unscheduled_impulse_scalars"], 0)

    def test_exact_controller_pair_matches_all_three_synthetic_cases(self) -> None:
        audit = audit_synthetic_controller_cases(
            self.profile, descriptor=self.descriptor
        )
        self.assertEqual(audit["case_count"], 3)
        self.assertEqual(audit["schedule_derivations"], 6)
        self.assertEqual(audit["controller_rows_evaluated"], 19200)
        self.assertEqual(audit["differing_target_scalars"], 0)
        self.assertEqual(audit["differing_effort_scalars"], 0)
        by_name = {row["case"]: row for row in audit["cases"]}
        self.assertEqual(by_name["ties_even"]["first_target_microradians"], [0, 2, -2])
        self.assertEqual(
            by_name["limiter_state"]["activation_counts"]["target_slew"], 1
        )
        self.assertEqual(
            by_name["limiter_state"]["activation_counts"]["effort_rate_clamp"],
            8,
        )
        self.assertEqual(
            by_name["limiter_state"]["activation_counts"]["positive_work_clamp"],
            6,
        )


if __name__ == "__main__":
    unittest.main()
