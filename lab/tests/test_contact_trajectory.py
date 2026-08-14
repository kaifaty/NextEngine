from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_trajectory import (
    ALGORITHM_ID,
    SCALAR_COLLIDER_LINEARIZATION,
    STABLE_FOOT_BOX_ALGORITHM_ID,
    STABLE_FOOT_BOX_COLLIDER_LINEARIZATION,
    _collider_linearization_rows,
    _collider_linearization_values,
    hybrid_velocity_stencil,
    project_reference_coupled_trajectory,
)
from next_lab.motion_math import (
    axis_angle_matrix,
    target_effectors,
    target_forward_kinematics,
)


FIXTURES = Path(__file__).parent / "fixtures"
PROFILE = (
    Path(__file__).parents[1]
    / "profiles"
    / "humanoid-contact-manifold-prototype.v8.json"
)
PROFILE_V9 = (
    Path(__file__).parents[1]
    / "profiles"
    / "humanoid-contact-manifold-prototype.v9.json"
)
SCRIPT = Path(__file__).parents[1] / "scripts" / "build_contact_manifold_prototype.py"
SPEC = importlib.util.spec_from_file_location(
    "build_contact_manifold_prototype_for_trajectory_test", SCRIPT
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("contact prototype builder module is unavailable")
builder = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(builder)


class ContactTrajectoryTests(unittest.TestCase):
    def test_v8_profile_rejects_changed_frozen_safety_identity(self) -> None:
        profile = json.loads(PROFILE.read_text(encoding="utf-8"))
        changed_contact = copy.deepcopy(profile["projection"])
        changed_contact["maximum_normal_residual_micrometres"] = 5_001
        with self.assertRaisesRegex(ValueError, "profile identity"):
            builder._trajectory_closure(changed_contact)

        changed_collider = copy.deepcopy(profile["projection"])
        changed_collider["trajectory_closure"][
            "minimum_collider_height_micrometres"
        ] = -3
        with self.assertRaisesRegex(ValueError, "profile identity"):
            builder._trajectory_closure(changed_collider)

    def test_v9_profile_binds_stable_foot_box_linearization(self) -> None:
        profile = json.loads(PROFILE_V9.read_text(encoding="utf-8"))
        closure = builder._trajectory_closure(profile["projection"])
        self.assertIsNotNone(closure)
        assert closure is not None
        self.assertEqual(closure.algorithm_id, STABLE_FOOT_BOX_ALGORITHM_ID)
        self.assertEqual(
            closure.collider_linearization_policy,
            STABLE_FOOT_BOX_COLLIDER_LINEARIZATION,
        )

        changed = copy.deepcopy(profile["projection"])
        changed["trajectory_closure"]["collider_linearization_policy"] = (
            SCALAR_COLLIDER_LINEARIZATION
        )
        with self.assertRaisesRegex(ValueError, "profile identity"):
            builder._trajectory_closure(changed)

    def test_stable_box_vertices_cross_scalar_minimum_cusp(self) -> None:
        collider = {
            "collider_id": "collider.test-foot",
            "contact_role": 8,
            "local_translation_micrometres": [0, 0, 0],
            "local_rotation_q1_30": [0, 0, 0, 1 << 30],
            "geometry": {
                "kind": "box",
                "half_extents_micrometres": [55_000, 30_000, 130_000],
            },
        }
        inventory = ((0, collider),)
        scalar_rows = _collider_linearization_rows(
            inventory, SCALAR_COLLIDER_LINEARIZATION
        )
        vertex_rows = _collider_linearization_rows(
            inventory, STABLE_FOOT_BOX_COLLIDER_LINEARIZATION
        )
        self.assertEqual(len(scalar_rows), 1)
        self.assertEqual(len(vertex_rows), 8)

        positions = np.zeros((1, 3), dtype=np.float64)
        baseline_angle = 50.0e-6
        probe = 100.0e-6
        delta = -100.0e-6
        baseline_rotation = axis_angle_matrix(
            np.asarray((1.0, 0.0, 0.0)), baseline_angle
        )[None, ...]
        probe_rotation = axis_angle_matrix(
            np.asarray((1.0, 0.0, 0.0)), baseline_angle + probe
        )[None, ...]
        exact_rotation = axis_angle_matrix(
            np.asarray((1.0, 0.0, 0.0)), baseline_angle + delta
        )[None, ...]

        scalar_base = _collider_linearization_values(
            positions, baseline_rotation, scalar_rows
        )
        scalar_probe = _collider_linearization_values(
            positions, probe_rotation, scalar_rows
        )
        scalar_prediction = scalar_base + (
            (scalar_probe - scalar_base) / probe * delta
        )
        vertex_base = _collider_linearization_values(
            positions, baseline_rotation, vertex_rows
        )
        vertex_probe = _collider_linearization_values(
            positions, probe_rotation, vertex_rows
        )
        vertex_prediction = np.min(
            vertex_base + (vertex_probe - vertex_base) / probe * delta
        )
        exact = float(
            _collider_linearization_values(
                positions, exact_rotation, scalar_rows
            )[0]
        )

        scalar_error = abs(exact - float(scalar_prediction[0]))
        vertex_error = abs(exact - vertex_prediction)
        self.assertGreater(scalar_error, 10.0e-6)
        self.assertLess(vertex_error, 0.001e-6)
        self.assertLess(vertex_error, scalar_error / 100.0)

    def test_hybrid_velocity_stencil_closes_entry_and_exit_sides(self) -> None:
        active = np.zeros((7, 2, 2), dtype=np.bool_)
        active[2:5, 0, 0] = True

        indices, coefficients = hybrid_velocity_stencil(active)

        np.testing.assert_array_equal(indices[2], (2, 3))
        np.testing.assert_array_equal(coefficients[2], (-60.0, 60.0))
        np.testing.assert_array_equal(indices[3], (2, 4))
        np.testing.assert_array_equal(coefficients[3], (-30.0, 30.0))
        np.testing.assert_array_equal(indices[4], (3, 4))
        np.testing.assert_array_equal(coefficients[4], (-60.0, 60.0))

    def test_v8_profile_executes_one_complete_clip_solve(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile = json.loads(PROFILE.read_text(encoding="utf-8"))
        closure = builder._trajectory_closure(profile["projection"])
        self.assertIsNotNone(closure)
        assert closure is not None
        self.assertEqual(closure.algorithm_id, ALGORITHM_ID)
        self.assertEqual(
            closure.collider_linearization_policy,
            SCALAR_COLLIDER_LINEARIZATION,
        )
        frame_count = 6
        joint_position = np.zeros(
            (frame_count, len(descriptor["joints"])), dtype=np.int64
        )
        root_position = np.zeros((frame_count, 3), dtype=np.int64)
        root_position[:, 1] = 943_500
        root_quaternion = np.zeros((frame_count, 4), dtype=np.int64)
        root_quaternion[:, 3] = 1 << 30
        effector_ids = tuple(
            sorted(
                effector["effector_id"]
                for effector in descriptor["effectors"]
            )
        )
        effector_position = np.empty(
            (frame_count, len(effector_ids), 3), dtype=np.int64
        )
        for frame in range(frame_count):
            positions, rotations = target_forward_kinematics(
                descriptor,
                root_position[frame].astype(np.float64) / 1_000_000.0,
                np.asarray((0.0, 0.0, 0.0, 1.0)),
                joint_position[frame].astype(np.float64) / 1_000_000.0,
            )
            effectors = target_effectors(descriptor, positions, rotations)
            effector_position[frame] = np.rint(
                np.stack([effectors[name] for name in effector_ids])
                * 1_000_000.0
            ).astype(np.int64)

        projection = project_reference_coupled_trajectory(
            descriptor=descriptor,
            effector_ids=effector_ids,
            root_position_um=root_position,
            root_quaternion_q1_30=root_quaternion,
            root_yaw_velocity_urad_s=np.zeros(
                frame_count, dtype=np.int64
            ),
            joint_position_urad=joint_position,
            effector_position_um=effector_position,
            contacts=np.zeros((frame_count, 7), dtype=np.uint8),
            support_state=np.zeros(frame_count, dtype=np.int64),
            frame_first=0,
            frame_last=frame_count - 1,
            tolerances=builder._tolerances(profile["projection"]),
            closure=closure,
        )

        self.assertEqual(projection.diagnostics["status"], "PASS")
        self.assertEqual(
            projection.diagnostics["trajectory_solver"]["termination"],
            "exact_quantized_pass",
        )
        self.assertGreaterEqual(
            projection.diagnostics["minimum_collider_height_micrometres"],
            -2,
        )
        self.assertLessEqual(
            projection.diagnostics["maximum_joint_velocity_basis_points"],
            2_500,
        )


if __name__ == "__main__":
    unittest.main()
