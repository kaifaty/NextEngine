from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.fixed_mode_controller_reachable_kinodynamic_formulation import (
    MOTOR_INTERVAL_COUNT,
    SUBSTEPS_PER_INTERVAL,
    _validate_profile,
    active_points_for_modes,
    audit_branch_decision,
    audit_fixed_mode_transitions,
    audit_variable_inventory,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-fixed-mode-controller-reachable-kinodynamic-formulation-r137.v1.json"
)


def _collocations(modes_by_interval: list[list[int]]) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for interval, modes in enumerate(modes_by_interval):
        for substep in range(SUBSTEPS_PER_INTERVAL):
            rows.append(
                {
                    "collocation": interval * SUBSTEPS_PER_INTERVAL + substep,
                    "interval": interval,
                    "substep": substep,
                    "contact_modes": modes,
                    "active_point_ordinals": active_points_for_modes(modes),
                }
            )
    return rows


class FixedModeControllerReachableKinodynamicFormulationTests(unittest.TestCase):
    def test_profile_authorizes_only_r138_report_only_conformance(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r138_kinodynamic_formulation_conformance"],
            "AUTHORIZED_REPORT_ONLY_ON_R137_COMPLETE",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r138_kinodynamic_formulation_conformance"
            )
        )

    def test_profile_rejects_premature_solve_authority(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        invalid = deepcopy(profile)
        invalid["bounded_acceptance"]["kinodynamic_solve"] = "AUTHORIZED"
        with self.assertRaisesRegex(ValueError, "formulation profile differs"):
            _validate_profile(invalid)

    def test_effort_is_derived_and_exact_replay_is_required(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        controller = profile["controller_reachability_contract"]
        inventory = profile["variable_inventory_contract"]
        self.assertEqual(controller["effort_status"], "EXACT_DERIVED_NOT_FREE")
        self.assertEqual(
            controller["acceptance_replay"],
            "BYTE_EXACT_QUANTIZED_FIXED_PD_REQUIRED",
        )
        self.assertEqual(inventory["free_effort_decision_scalars"], 0)
        self.assertEqual(inventory["derived_applied_effort_scalars"], 73600)

    def test_active_point_order_is_source_owned(self) -> None:
        self.assertEqual(active_points_for_modes([0, 0]), [])
        self.assertEqual(active_points_for_modes([2, 0]), [1])
        self.assertEqual(active_points_for_modes([3, 0]), [0, 1])
        self.assertEqual(active_points_for_modes([0, 2]), [3])
        self.assertEqual(active_points_for_modes([0, 3]), [2, 3])
        with self.assertRaisesRegex(ValueError, "contact mode differs"):
            active_points_for_modes([1, 0])

    def test_transition_audit_separates_activation_and_zero_impulse_release(
        self,
    ) -> None:
        modes = [[0, 0] for _ in range(MOTOR_INTERVAL_COUNT)]
        modes[10] = [0, 2]
        modes[11] = [0, 3]
        modes[12] = [0, 2]
        audit = audit_fixed_mode_transitions(_collocations(modes))
        self.assertEqual(audit["changed_motor_boundary_count"], 4)
        self.assertEqual(audit["activation_boundary_count"], 2)
        self.assertEqual(audit["activation_point_count"], 2)
        self.assertEqual(audit["deactivation_boundary_count"], 2)
        self.assertEqual(audit["deactivation_point_count"], 2)
        self.assertEqual(
            [row["physics_node"] for row in audit["transition_rows"]],
            [40, 44, 48, 52],
        )
        self.assertEqual(
            [row["event"] for row in audit["transition_rows"]],
            [
                "RIGID_POINT_ACTIVATION_IMPULSE",
                "RIGID_POINT_ACTIVATION_IMPULSE",
                "ZERO_IMPULSE_POINT_RELEASE",
                "ZERO_IMPULSE_POINT_RELEASE",
            ],
        )

    def test_transition_audit_rejects_simultaneous_reseat(self) -> None:
        modes = [[2, 0] for _ in range(MOTOR_INTERVAL_COUNT)]
        modes[10] = [0, 2]
        with self.assertRaisesRegex(
            ValueError, "changes activation and release together"
        ):
            audit_fixed_mode_transitions(_collocations(modes))

    def test_variable_inventory_has_no_hidden_effort_or_anchor_variables(self) -> None:
        transitions = {
            "active_point_force_rows": 4956,
            "activation_point_count": 18,
        }
        r136 = {"solver_result": {"active_point_cones": 4956}}
        audit = audit_variable_inventory(r136=r136, transitions=transitions)
        self.assertEqual(audit["primary_decision_scalars"], 311780)
        self.assertEqual(audit["activation_impulse_scalars"], 54)
        self.assertEqual(audit["free_effort_decision_scalars"], 0)
        self.assertEqual(audit["contact_anchor_decision_scalars"], 0)

    def test_branch_matrix_selects_fixed_mode_codesign_only(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_branch_decision(profile)
        self.assertEqual(audit["branches_compared"], 4)
        self.assertEqual(
            audit["selected_branch"],
            "fixed_mode_controller_reachable_trajectory_codesign",
        )
        changed = deepcopy(profile)
        changed["branch_decisions"][1]["decision"] = (
            "SELECT_R137_REPORT_ONLY_FORMULATION"
        )
        with self.assertRaisesRegex(ValueError, "branch decision matrix"):
            audit_branch_decision(changed)

    def test_impulse_order_is_post_integration_and_release_is_zero(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        hybrid = profile["hybrid_transcription_contract"]
        self.assertIn(
            "before a scheduled boundary impulse", hybrid["configuration_integration"]
        )
        self.assertIn("11 activation boundaries", hybrid["activation_momentum_jump"])
        self.assertEqual(hybrid["release_impulse"], "EXACT_ZERO")
        self.assertEqual(hybrid["contact_mode_choice"], "FORBIDDEN")


if __name__ == "__main__":
    unittest.main()
