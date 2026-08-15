from __future__ import annotations

import hashlib
import unittest

import numpy as np
from next_lab.projected_inverse_dynamics_composition import (
    ARRAY_SHAPES,
    applied_effort_newton_metres,
    assemble_projected_reduced_local_system,
    projected_reduced_column_scale,
    verify_r133_array_hashes,
)


class ProjectedInverseDynamicsCompositionTests(unittest.TestCase):
    def test_effort_conversion_requires_integer_valued_float64(self) -> None:
        effort = np.arange(23, dtype=np.float64) * 1_000_000.0
        np.testing.assert_array_equal(
            applied_effort_newton_metres(effort), np.arange(23, dtype=np.float64)
        )
        non_integer = effort.copy()
        non_integer[0] = 0.5
        with self.assertRaisesRegex(ValueError, "effort differs"):
            applied_effort_newton_metres(non_integer)
        with self.assertRaisesRegex(ValueError, "effort differs"):
            applied_effort_newton_metres(effort.astype(np.float32))

    def test_effort_substitution_changes_only_actuated_dynamics_rhs(self) -> None:
        mass = np.eye(29, dtype=np.float64)
        bias = np.arange(29, dtype=np.float64)
        jacobians = np.zeros((4, 3, 29), dtype=np.float64)
        jdot_v = np.zeros((4, 3), dtype=np.float64)
        zero = np.zeros(23, dtype=np.float64)
        applied = np.arange(23, dtype=np.float64) * 1_000_000.0
        matrix_zero, right_zero = assemble_projected_reduced_local_system(
            mass=mass,
            bias_from_projected_velocity=bias,
            applied_effort_micronewton_metres=zero,
            contact_jacobians=jacobians,
            contact_jdot_v_from_projected_velocity=jdot_v,
            active_point_ordinals=(3,),
            modes=np.asarray((0, 2), dtype=np.uint8),
        )
        matrix_applied, right_applied = assemble_projected_reduced_local_system(
            mass=mass,
            bias_from_projected_velocity=bias,
            applied_effort_micronewton_metres=applied,
            contact_jacobians=jacobians,
            contact_jdot_v_from_projected_velocity=jdot_v,
            active_point_ordinals=(3,),
            modes=np.asarray((0, 2), dtype=np.uint8),
        )
        np.testing.assert_array_equal(matrix_zero, matrix_applied)
        np.testing.assert_array_equal(right_applied[:6], right_zero[:6])
        np.testing.assert_array_equal(
            right_applied[6:29] - right_zero[6:29], applied / 1_000_000.0
        )
        np.testing.assert_array_equal(right_applied[29:], right_zero[29:])

    def test_r120_acceleration_changes_scale_only(self) -> None:
        low = np.zeros((2, 29), dtype=np.float64)
        high = np.full((2, 29), 3.0, dtype=np.float64)
        low_scale = projected_reduced_column_scale(
            r120_acceleration_knots=low,
            total_body_mass_kilograms=75.0,
            gravity_y_metres_per_second_squared=-10.0,
            active_point_count=2,
        )
        high_scale = projected_reduced_column_scale(
            r120_acceleration_knots=high,
            total_body_mass_kilograms=75.0,
            gravity_y_metres_per_second_squared=-10.0,
            active_point_count=2,
        )
        np.testing.assert_array_equal(low_scale[:29], 1.0)
        np.testing.assert_array_equal(high_scale[:29], 3.0)
        np.testing.assert_array_equal(low_scale[29:], high_scale[29:])

    def test_r133_hash_guard_rejects_one_scalar_mutation(self) -> None:
        arrays = {
            name: np.zeros(shape, dtype=np.float64)
            for name, shape in ARRAY_SHAPES.items()
        }
        hashes = {
            name: hashlib.sha256(value.tobytes()).hexdigest()
            for name, value in arrays.items()
        }
        self.assertEqual(verify_r133_array_hashes(arrays, hashes), hashes)
        arrays["applied_effort_micronewton_metres"][0, 0] = 1.0
        with self.assertRaisesRegex(ValueError, "hash differs"):
            verify_r133_array_hashes(arrays, hashes)


if __name__ == "__main__":
    unittest.main()
