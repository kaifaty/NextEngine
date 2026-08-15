from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _validate_profile,
    audit_one_local_system_resource,
    finite_difference_frozen_jacobian,
    finite_difference_frozen_jdot_v,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    Configuration,
    build_spatial_model,
    forward_kinematics,
    generalized_contact_force,
    inverse_dynamics_mass_matrix,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    point_position,
    propagate_motion,
    rotation_exp,
)
from next_lab.motion_math import matrix_to_quaternion, target_forward_kinematics

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-fixed-pd-inverse-dynamics-implementation-conformance-r122.v1.json"
)
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class FixedPdInverseDynamicsConformanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())
        cls.descriptor = json.loads(DESCRIPTOR.read_bytes())
        cls.model = build_spatial_model(cls.descriptor)
        cls.configuration = Configuration(
            root_position=np.asarray((0.2, 1.1, -0.3), dtype=np.float64),
            root_rotation=rotation_exp(
                np.asarray((0.07, -0.11, 0.04), dtype=np.float64)
            ),
            joint_positions=np.linspace(-0.12, 0.14, 23, dtype=np.float64),
        )

    def test_profile_authorizes_only_one_r123_execution_on_pass(self) -> None:
        _validate_profile(self.profile)
        self.assertFalse(self.profile["scope"]["inverse_dynamics_execution"])
        self.assertEqual(self.profile["scope"]["r123_local_system_solves"], 0)
        self.assertEqual(
            self.profile["decision"]["pass"],
            "PERMIT_R123_SINGLE_BOUNDED_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_ONLY",
        )
        self.assertEqual(self.profile["bounded_acceptance"]["physx"], "NOT_AUTHORIZED")

    def test_descriptor_forward_kinematics_matches_frozen_kernel(self) -> None:
        actual = forward_kinematics(self.model, self.configuration)
        positions, rotations = target_forward_kinematics(
            self.descriptor,
            self.configuration.root_position,
            matrix_to_quaternion(self.configuration.root_rotation),
            self.configuration.joint_positions,
        )
        np.testing.assert_allclose(actual.body_positions, positions, atol=1.0e-12)
        np.testing.assert_allclose(actual.body_rotations, rotations, atol=1.0e-12)

    def test_mass_matrix_matches_independent_inverse_dynamics_columns(self) -> None:
        velocity = np.linspace(-0.7, 0.8, 29, dtype=np.float64)
        kinetic, _ = mass_matrix(self.model, self.configuration)
        inverse = inverse_dynamics_mass_matrix(self.model, self.configuration, velocity)
        np.testing.assert_allclose(kinetic, kinetic.T, atol=1.0e-12)
        np.linalg.cholesky(kinetic)
        np.testing.assert_allclose(kinetic, inverse, atol=1.0e-11, rtol=1.0e-11)

    def test_contact_jacobian_and_jdot_v_match_frozen_finite_difference(self) -> None:
        effector = next(
            row
            for row in self.descriptor["effectors"]
            if row["effector_id"] == "effector.left-forefoot"
        )
        body_slot = next(
            int(row["body_slot"])
            for row in self.descriptor["bodies"]
            if row["body_id"] == effector["body_id"]
        )
        local = (
            np.asarray(effector["local_translation_micrometres"], dtype=np.float64)
            / 1_000_000.0
        )
        kinematics = forward_kinematics(self.model, self.configuration)
        actual = point_jacobian(self.model, kinematics, body_slot, local)
        reference = finite_difference_frozen_jacobian(
            descriptor=self.descriptor,
            configuration=self.configuration,
            body_slot=body_slot,
            local_point=local,
            probe=1.0e-6,
        )
        np.testing.assert_allclose(actual, reference, atol=1.0e-7, rtol=1.0e-6)

        velocity = np.linspace(-0.35, 0.4, 29, dtype=np.float64)
        motion = propagate_motion(
            self.model,
            kinematics,
            velocity,
            np.zeros(29, dtype=np.float64),
        )
        analytic = point_acceleration(kinematics, motion, body_slot, local)
        finite_difference = finite_difference_frozen_jdot_v(
            descriptor=self.descriptor,
            configuration=self.configuration,
            velocity=velocity,
            body_slot=body_slot,
            local_point=local,
            probe_seconds=1.0e-4,
        )
        np.testing.assert_allclose(analytic, finite_difference, atol=1.0e-5)

    def test_r113_force_order_projects_world_wrench(self) -> None:
        effector = next(
            row
            for row in self.descriptor["effectors"]
            if row["effector_id"] == "effector.right-heel"
        )
        body_slot = next(
            int(row["body_slot"])
            for row in self.descriptor["bodies"]
            if row["body_id"] == effector["body_id"]
        )
        local = (
            np.asarray(effector["local_translation_micrometres"], dtype=np.float64)
            / 1_000_000.0
        )
        kinematics = forward_kinematics(self.model, self.configuration)
        jacobian = point_jacobian(self.model, kinematics, body_slot, local)
        force_nrf = np.asarray((100.0, -6.0, 8.0), dtype=np.float64)
        projected = generalized_contact_force(jacobian, force_nrf)
        point = point_position(kinematics, body_slot, local)
        world_force = force_nrf[[1, 0, 2]]
        np.testing.assert_array_equal(projected[:3], world_force)
        np.testing.assert_allclose(
            projected[3:6],
            np.cross(point - kinematics.body_positions[0], world_force),
            atol=1.0e-12,
        )

    def test_resource_probe_allocates_but_never_solves_r123_system(self) -> None:
        audit = audit_one_local_system_resource(self.profile)
        self.assertEqual(audit["status"], "PASS")
        self.assertEqual(audit["matrix_shape"], [64, 64])
        self.assertFalse(audit["factorization_performed"])
        self.assertFalse(audit["solve_performed"])
        self.assertEqual(audit["r123_local_system_solves"], 0)


if __name__ == "__main__":
    unittest.main()
