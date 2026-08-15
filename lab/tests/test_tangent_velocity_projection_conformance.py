from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.tangent_velocity_projection_conformance import (
    _projection_passes,
    _validate_profile,
    project_tangent_velocity,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-tangent-velocity-projection-r129.v1.json"


class TangentVelocityProjectionConformanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())
        self.numeric = self.profile["numeric_contract"]
        self.mass = np.diag(np.linspace(0.5, 3.0, 29)).astype(np.float64)
        self.velocity = np.linspace(-0.7, 0.9, 29, dtype=np.float64)

    def test_profile_bounds_r129_to_seven_anchors(self) -> None:
        _validate_profile(self.profile)
        scope = self.profile["scope"]
        self.assertEqual(scope["frozen_anchor_count"], 7)
        self.assertEqual(scope["maximum_real_state_projections"], 6)
        self.assertEqual(scope["full_schedule_projections"], 0)
        self.assertEqual(scope["inverse_dynamics_evaluations"], 0)

    def test_single_point_projection_is_minimum_mass_metric_solution(self) -> None:
        jacobian = np.zeros((3, 29), dtype=np.float64)
        jacobian[:, :3] = np.eye(3)
        result = project_tangent_velocity(
            mass=self.mass,
            jacobian=jacobian,
            velocity=self.velocity,
            expected_rank=3,
            numeric=self.numeric,
        )
        self.assertTrue(
            _projection_passes(analysis=result, numeric=self.numeric, flat_audit=None)
        )
        assert result.projected_velocity is not None
        np.testing.assert_allclose(result.projected_velocity[:3], 0.0, atol=1.0e-14)
        np.testing.assert_allclose(
            result.projected_velocity[3:], self.velocity[3:], atol=1.0e-14
        )
        self.assertLessEqual(result.kinetic_energy_increase_joules or 0.0, 0.0)

    def test_same_body_two_point_projection_has_one_multiplier_gauge(self) -> None:
        separation = np.asarray((0.0, 0.0, 0.215), dtype=np.float64)
        jacobian = np.zeros((6, 29), dtype=np.float64)
        jacobian[:3, :3] = np.eye(3)
        jacobian[3:, :3] = np.eye(3)
        jacobian[3:, 3:6] = -_skew(separation)
        result = project_tangent_velocity(
            mass=self.mass,
            jacobian=jacobian,
            velocity=self.velocity,
            expected_rank=5,
            numeric=self.numeric,
        )
        self.assertEqual((result.rank, result.nullity), (5, 1))
        self.assertTrue(
            _projection_passes(analysis=result, numeric=self.numeric, flat_audit=None)
        )
        self.assertLessEqual(
            result.gauge_velocity_effect_maximum_absolute or 0.0, 1.0e-12
        )

    def test_projection_is_idempotent(self) -> None:
        jacobian = np.zeros((3, 29), dtype=np.float64)
        jacobian[:, :3] = np.eye(3)
        first = project_tangent_velocity(
            mass=self.mass,
            jacobian=jacobian,
            velocity=self.velocity,
            expected_rank=3,
            numeric=self.numeric,
        )
        assert first.projected_velocity is not None
        second = project_tangent_velocity(
            mass=self.mass,
            jacobian=jacobian,
            velocity=first.projected_velocity,
            expected_rank=3,
            numeric=self.numeric,
        )
        assert second.projected_velocity is not None
        np.testing.assert_allclose(
            second.projected_velocity, first.projected_velocity, atol=1.0e-14
        )

    def test_rank_gap_is_invalid_without_threshold_tuning(self) -> None:
        jacobian = np.zeros((3, 29), dtype=np.float64)
        jacobian[0, 0] = 1.0
        jacobian[1, 1] = 1.0
        jacobian[2, 2] = 5.0e-11
        result = project_tangent_velocity(
            mass=np.eye(29, dtype=np.float64),
            jacobian=jacobian,
            velocity=self.velocity,
            expected_rank=3,
            numeric=self.numeric,
        )
        self.assertEqual(result.status, "INVALID")
        self.assertEqual(result.invalid_reason, "AMBIGUOUS_PROJECTION_RANK_GAP")

    def test_nonfinite_input_is_rejected(self) -> None:
        jacobian = np.zeros((3, 29), dtype=np.float64)
        jacobian[:, :3] = np.eye(3)
        with self.assertRaisesRegex(ValueError, "projection input differs"):
            project_tangent_velocity(
                mass=self.mass,
                jacobian=jacobian,
                velocity=np.full(29, np.nan, dtype=np.float64),
                expected_rank=3,
                numeric=self.numeric,
            )

    def test_asymmetric_mass_matrix_is_invalid(self) -> None:
        jacobian = np.zeros((3, 29), dtype=np.float64)
        jacobian[:, :3] = np.eye(3)
        mass = self.mass.copy()
        mass[0, 1] = 1.0e-4
        result = project_tangent_velocity(
            mass=mass,
            jacobian=jacobian,
            velocity=self.velocity,
            expected_rank=3,
            numeric=self.numeric,
        )
        self.assertEqual(result.status, "INVALID")
        self.assertEqual(result.invalid_reason, "MASS_MATRIX_NOT_SYMMETRIC")

    def test_non_positive_definite_mass_matrix_is_invalid(self) -> None:
        jacobian = np.zeros((3, 29), dtype=np.float64)
        jacobian[:, :3] = np.eye(3)
        mass = self.mass.copy()
        mass[0, 0] = -1.0
        result = project_tangent_velocity(
            mass=mass,
            jacobian=jacobian,
            velocity=self.velocity,
            expected_rank=3,
            numeric=self.numeric,
        )
        self.assertEqual(result.status, "INVALID")
        self.assertEqual(result.invalid_reason, "MASS_MATRIX_NOT_POSITIVE_DEFINITE")


def _skew(vector: np.ndarray) -> np.ndarray:
    x, y, z = vector
    return np.asarray(((0.0, -z, y), (z, 0.0, -x), (-y, x, 0.0)))


if __name__ == "__main__":
    unittest.main()
