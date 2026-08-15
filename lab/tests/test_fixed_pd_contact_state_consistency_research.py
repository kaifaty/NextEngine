from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

import numpy as np
from next_lab.fixed_pd_contact_state_consistency_research import (
    _validate_profile,
    audit_rigid_line_compatibility,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-post-r127-contact-state-consistency-research.v1.json"
)


class FixedPdContactStateConsistencyResearchTests(unittest.TestCase):
    def test_profile_is_static_and_blocks_every_execution(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertTrue(
            all(
                value == "NOT_AUTHORIZED"
                for value in profile["bounded_acceptance"].values()
            )
        )
        scope = profile["scope"]
        self.assertEqual(scope["kinematic_state_audits"], 1)
        self.assertEqual(scope["local_system_reconstructions"], 0)
        self.assertEqual(scope["singular_value_decompositions"], 0)
        self.assertEqual(scope["particular_solutions"], 0)
        self.assertEqual(scope["gauge_interval_classifications"], 0)

    def test_profile_rejects_a_missing_execution_prohibition(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        incomplete = deepcopy(profile)
        del incomplete["bounded_acceptance"]["training"]
        with self.assertRaisesRegex(ValueError, "research profile differs"):
            _validate_profile(incomplete)

    def test_rigid_line_projection_equals_centripetal_identity(self) -> None:
        length = 0.215
        angular_speed = 0.13
        heel = np.zeros(3, dtype=np.float64)
        forefoot = np.asarray((0.0, 0.0, length), dtype=np.float64)
        omega = np.asarray((0.0, angular_speed, 0.0), dtype=np.float64)
        forefoot_velocity = np.cross(omega, forefoot)
        forefoot_acceleration = np.cross(omega, np.cross(omega, forefoot))
        audit = audit_rigid_line_compatibility(
            heel_local=heel,
            forefoot_local=forefoot,
            body_rotation=np.eye(3, dtype=np.float64),
            body_angular_velocity=omega,
            heel_jdot_v=np.zeros(3, dtype=np.float64),
            forefoot_jdot_v=forefoot_acceleration,
            heel_velocity=np.zeros(3, dtype=np.float64),
            forefoot_velocity=forefoot_velocity,
        )
        expected = -length * angular_speed * angular_speed
        self.assertAlmostEqual(
            audit["observed_line_compatibility_metres_per_second_squared"],
            expected,
        )
        self.assertAlmostEqual(
            audit["centripetal_identity_prediction_metres_per_second_squared"],
            expected,
        )
        self.assertLessEqual(audit["identity_absolute_error"], 1.0e-18)
        self.assertFalse(audit["two_zero_point_accelerations_are_compatible"])

    def test_parallel_angular_velocity_is_compatible_along_line(self) -> None:
        audit = audit_rigid_line_compatibility(
            heel_local=np.zeros(3, dtype=np.float64),
            forefoot_local=np.asarray((0.0, 0.0, 0.215), dtype=np.float64),
            body_rotation=np.eye(3, dtype=np.float64),
            body_angular_velocity=np.asarray((0.0, 0.0, 0.2), dtype=np.float64),
            heel_jdot_v=np.zeros(3, dtype=np.float64),
            forefoot_jdot_v=np.zeros(3, dtype=np.float64),
            heel_velocity=np.zeros(3, dtype=np.float64),
            forefoot_velocity=np.zeros(3, dtype=np.float64),
        )
        self.assertEqual(audit["perpendicular_angular_speed_radians_per_second"], 0.0)
        self.assertEqual(audit["identity_absolute_error"], 0.0)
        self.assertTrue(audit["two_zero_point_accelerations_are_compatible"])

    def test_mismatched_jdot_v_is_detected(self) -> None:
        audit = audit_rigid_line_compatibility(
            heel_local=np.zeros(3, dtype=np.float64),
            forefoot_local=np.asarray((0.0, 0.0, 0.215), dtype=np.float64),
            body_rotation=np.eye(3, dtype=np.float64),
            body_angular_velocity=np.asarray((0.0, 0.1, 0.0), dtype=np.float64),
            heel_jdot_v=np.zeros(3, dtype=np.float64),
            forefoot_jdot_v=np.zeros(3, dtype=np.float64),
            heel_velocity=np.zeros(3, dtype=np.float64),
            forefoot_velocity=np.zeros(3, dtype=np.float64),
        )
        self.assertGreater(audit["identity_absolute_error"], 1.0e-4)


if __name__ == "__main__":
    unittest.main()
