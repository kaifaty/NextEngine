from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

import numpy as np
from next_lab.exit_mode_owned_lift_conformance import (
    BOUNDED_ACCEPTANCE,
    _validate_profile,
    audit_synthetic_edge_cases,
    classify_contact_edge,
    select_edge_owned_velocity,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-exit-mode-owned-lift-conformance-r132.v1.json"


class ExitModeOwnedLiftConformanceTests(unittest.TestCase):
    def test_profile_authorizes_only_one_r133_schedule_on_pass(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertEqual(profile["bounded_acceptance"], BOUNDED_ACCEPTANCE)
        self.assertEqual(
            profile["bounded_acceptance"]["r133_projected_schedule_execution"],
            "AUTHORIZED_ON_R132_PASS_ONLY",
        )
        self.assertEqual(profile["scope"]["maximum_real_anchor_rows"], 36)
        self.assertEqual(profile["scope"]["full_schedule_projections"], 0)

    def test_profile_rejects_local_diagnostic_as_acceptance_gate(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        changed = deepcopy(profile)
        changed["acceptance_contract"]["joint_velocity_diagnostic_is_gate"] = True
        with self.assertRaisesRegex(ValueError, "conformance profile differs"):
            _validate_profile(changed)

    def test_exit_holds_left_velocity_for_all_substeps(self) -> None:
        left = np.arange(29, dtype=np.float64)
        right = left + 8.0
        for substep in range(4):
            selected = select_edge_owned_velocity(
                left=left,
                right=right,
                substep=substep,
                edge_class="exit",
            )
            np.testing.assert_array_equal(selected, left)
            self.assertIsNot(selected, left)

    def test_non_exit_edges_keep_affine_velocity(self) -> None:
        left = np.zeros(29, dtype=np.float64)
        right = np.full(29, 4.0, dtype=np.float64)
        for edge_class in ("ordinary", "entry", "active_mode_change"):
            selected = select_edge_owned_velocity(
                left=left,
                right=right,
                substep=3,
                edge_class=edge_class,
            )
            np.testing.assert_array_equal(selected, np.full(29, 3.0))

    def test_edge_classifier_covers_hybrid_transitions(self) -> None:
        self.assertEqual(classify_contact_edge((0, 0), (0, 0)), "ordinary")
        self.assertEqual(classify_contact_edge((0, 0), (2, 0)), "entry")
        self.assertEqual(classify_contact_edge((0, 3), (0, 2)), "active_mode_change")
        self.assertEqual(classify_contact_edge((2, 0), (0, 0)), "exit")

    def test_malformed_or_mixed_edge_fails_closed(self) -> None:
        with self.assertRaisesRegex(ValueError, "mode value differs"):
            classify_contact_edge((1, 0), (0, 0))
        with self.assertRaisesRegex(ValueError, "mixed contact edge differs"):
            classify_contact_edge((2, 0), (0, 2))
        left = np.zeros(29, dtype=np.float64)
        with self.assertRaisesRegex(ValueError, "edge class differs"):
            select_edge_owned_velocity(
                left=left,
                right=left,
                substep=0,
                edge_class="unknown",
            )

    def test_synthetic_edge_audit_uses_no_projection_systems(self) -> None:
        audit = audit_synthetic_edge_cases()
        self.assertEqual(audit["status"], "PASS")
        self.assertEqual(len(audit["cases"]), 5)
        self.assertEqual(audit["state_projection_systems"], 0)


if __name__ == "__main__":
    unittest.main()
