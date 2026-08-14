from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_manifold import (
    ColliderClosure,
    ContactManifoldTolerances,
    FootContactMode,
    contact_point_mask,
    infer_contact_modes,
    project_reference_contact_manifold,
)
from next_lab.motion_math import target_effectors, target_forward_kinematics


FIXTURES = Path(__file__).parent / "fixtures"


class ContactManifoldTests(unittest.TestCase):
    def test_contact_mode_never_combines_different_point_evidence(self) -> None:
        ids = (
            "effector.left-heel",
            "effector.left-forefoot",
            "effector.right-heel",
            "effector.right-forefoot",
        )
        positions = np.zeros((5, 4, 3), dtype=np.int64)
        positions[:, 0, 0] = np.arange(5) * 20_000
        positions[:, 1, 1] = 50_000
        positions[:, 2:, 1] = 100_000
        modes = infer_contact_modes(
            effector_ids=ids,
            effector_position_um=positions,
            support_state=np.zeros(5, dtype=np.int64),
            tolerances=ContactManifoldTolerances(
                minimum_mode_on_frames=1,
                minimum_mode_off_frames=1,
            ),
        )
        np.testing.assert_array_equal(modes, np.zeros((5, 2), dtype=np.uint8))

    def test_contact_modes_select_exact_sticking_points(self) -> None:
        ids = (
            "effector.left-heel",
            "effector.left-forefoot",
            "effector.right-heel",
            "effector.right-forefoot",
        )
        tolerances = ContactManifoldTolerances(
            minimum_mode_on_frames=1,
            minimum_mode_off_frames=1,
        )
        expected = (
            FootContactMode.HEEL_STICKING,
            FootContactMode.FOREFOOT_STICKING,
            FootContactMode.FLAT_STICKING,
        )
        heights = ((0, 50_000), (50_000, 0), (0, 0))
        for pair, mode in zip(heights, expected, strict=True):
            positions = np.zeros((5, 4, 3), dtype=np.int64)
            positions[:, 0, 1] = pair[0]
            positions[:, 1, 1] = pair[1]
            positions[:, 2:, 1] = 100_000
            modes = infer_contact_modes(
                effector_ids=ids,
                effector_position_um=positions,
                support_state=np.zeros(5, dtype=np.int64),
                tolerances=tolerances,
            )
            self.assertTrue(np.all(modes[:, 0] == mode))
            self.assertTrue(np.all(modes[:, 1] == FootContactMode.FLIGHT))

        mask = contact_point_mask(
            np.asarray(
                [[mode, FootContactMode.FLIGHT] for mode in expected],
                dtype=np.uint8,
            )
        )
        np.testing.assert_array_equal(
            mask[:, 0],
            np.asarray(((True, False), (False, True), (True, True))),
        )

    def test_window_projection_closes_pose_and_derived_velocity(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        frame_count = 6
        joint_position = np.zeros(
            (frame_count, len(descriptor["joints"])), dtype=np.int64
        )
        root_position = np.zeros((frame_count, 3), dtype=np.int64)
        root_position[:, 0] = np.arange(frame_count) * 5_000
        root_position[:, 1] = 943_500
        root_quaternion = np.zeros((frame_count, 4), dtype=np.int64)
        root_quaternion[:, 3] = 1 << 30
        effector_ids = tuple(
            sorted(effector["effector_id"] for effector in descriptor["effectors"])
        )
        effector_position = np.empty(
            (frame_count, len(effector_ids), 3), dtype=np.int64
        )
        for frame in range(frame_count):
            positions, rotations = target_forward_kinematics(
                descriptor,
                root_position[frame].astype(np.float64) / 1_000_000.0,
                np.asarray((0.0, 0.0, 0.0, 1.0)),
                joint_position[frame].astype(np.float64),
            )
            effectors = target_effectors(descriptor, positions, rotations)
            effector_position[frame] = np.rint(
                np.stack([effectors[name] for name in effector_ids]) * 1_000_000.0
            ).astype(np.int64)

        projection = project_reference_contact_manifold(
            descriptor=descriptor,
            effector_ids=effector_ids,
            root_position_um=root_position,
            root_quaternion_q1_30=root_quaternion,
            root_yaw_urad=np.zeros(frame_count, dtype=np.int64),
            joint_position_urad=joint_position,
            effector_position_um=effector_position,
            contacts=np.zeros((frame_count, 7), dtype=np.uint8),
            support_state=np.zeros(frame_count, dtype=np.int64),
            frame_first=0,
            frame_last=frame_count - 1,
        )

        self.assertEqual(projection.diagnostics["status"], "PASS")
        self.assertLessEqual(
            projection.diagnostics["maximum_tangential_step_micrometres"],
            2_000,
        )
        self.assertLessEqual(
            projection.diagnostics[
                "maximum_analytic_tangential_step_micrometres"
            ],
            2_000,
        )
        np.testing.assert_array_equal(projection.joint_position_urad, joint_position)
        self.assertTrue(np.all(projection.contacts[:, 0] == 1))
        self.assertTrue(np.all(projection.contacts[:, 1] == 0))
        self.assertGreater(
            projection.diagnostics["maximum_root_correction_micrometres"], 0
        )

    def test_collider_closure_uses_leg_chain_for_flight_foot(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        inputs = _flight_collider_inputs(descriptor)

        projection = project_reference_contact_manifold(
            descriptor=descriptor,
            **inputs,
            frame_first=0,
            frame_last=5,
            collider_closure=_collider_closure(descriptor),
        )

        self.assertEqual(projection.diagnostics["status"], "PASS")
        self.assertEqual(
            projection.diagnostics["collider_closure_status"], "PASS"
        )
        self.assertGreater(
            projection.diagnostics["initial_flight_collider_deficit_frame_count"],
            0,
        )
        self.assertEqual(
            projection.diagnostics[
                "flight_collider_deficit_without_leg_correction_count"
            ],
            0,
        )
        self.assertGreater(
            projection.diagnostics["maximum_joint_correction_microradians"],
            0,
        )
        self.assertGreaterEqual(
            projection.diagnostics["minimum_collider_height_micrometres"],
            -2,
        )

    def test_collider_closure_reports_unsatisfied_contact_after_bounded_solve(
        self,
    ) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        inputs = _flight_collider_inputs(descriptor)

        projection = project_reference_contact_manifold(
            descriptor=descriptor,
            **inputs,
            frame_first=0,
            frame_last=5,
            collider_closure=_collider_closure(
                descriptor,
                outer_iterations=1,
                maximum_joint_update_microradians=1,
            ),
        )

        self.assertEqual(projection.diagnostics["status"], "FAIL")
        self.assertEqual(projection.diagnostics["active_contact_status"], "FAIL")
        self.assertGreater(
            projection.diagnostics["maximum_normal_residual_micrometres"],
            5_000,
        )

    def test_collider_closure_anchors_active_support_before_swing(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        inputs = _flight_collider_inputs(descriptor)
        inputs["root_position_um"][:, 1] += 5_000
        inputs["effector_position_um"][:, :, 1] += 5_000

        projection = project_reference_contact_manifold(
            descriptor=descriptor,
            **inputs,
            frame_first=0,
            frame_last=5,
            collider_closure=_collider_closure(
                descriptor,
                active_contact_anchor_target_micrometres=0,
            ),
        )

        self.assertEqual(projection.diagnostics["status"], "PASS")
        self.assertGreaterEqual(
            projection.diagnostics[
                "maximum_active_contact_anchor_micrometres"
            ],
            4_999,
        )
        self.assertLessEqual(
            projection.diagnostics["maximum_normal_residual_micrometres"],
            1_500,
        )


def _flight_collider_inputs(descriptor: dict[str, object]) -> dict[str, object]:
    frame_count = 6
    joints = descriptor["joints"]
    joint_lookup = {
        joint["joint_id"]: int(joint["dof_ordinal"])
        for joint in joints
    }
    joint_position = np.zeros((frame_count, len(joints)), dtype=np.int64)
    joint_position[:, joint_lookup["joint.right-ankle-roll"]] = 261_799
    root_position = np.zeros((frame_count, 3), dtype=np.int64)
    root_position[:, 1] = 943_500
    root_quaternion = np.zeros((frame_count, 4), dtype=np.int64)
    root_quaternion[:, 3] = 1 << 30
    effector_ids = tuple(
        sorted(
            effector["effector_id"] for effector in descriptor["effectors"]
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
            np.stack([effectors[name] for name in effector_ids]) * 1_000_000.0
        ).astype(np.int64)
    return {
        "effector_ids": effector_ids,
        "root_position_um": root_position,
        "root_quaternion_q1_30": root_quaternion,
        "root_yaw_urad": np.zeros(frame_count, dtype=np.int64),
        "joint_position_urad": joint_position,
        "effector_position_um": effector_position,
        "contacts": np.zeros((frame_count, 7), dtype=np.uint8),
        "support_state": np.zeros(frame_count, dtype=np.int64),
    }


def _collider_closure(
    descriptor: dict[str, object],
    *,
    outer_iterations: int = 40,
    maximum_joint_update_microradians: int = 80_000,
    active_contact_anchor_target_micrometres: int | None = None,
) -> ColliderClosure:
    suffixes = (
        "hip-pitch",
        "hip-roll",
        "knee",
        "ankle-pitch",
        "ankle-roll",
    )
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    return ColliderClosure(
        minimum_collider_height_micrometres=-2,
        swing_clearance_target_micrometres=5_000,
        maximum_root_vertical_velocity_micrometres_per_second=1_000_000,
        joint_velocity_limit_basis_points=10_000,
        ordered_joint_suffixes=suffixes,
        joint_bounds_microradians=tuple(
            tuple(
                tuple(by_id[f"joint.{side}-{suffix}"]["soft_limit_microradians"])
                for suffix in suffixes
            )
            for side in ("left", "right")
        ),
        outer_iterations=outer_iterations,
        jacobian_probe_microradians=100,
        maximum_joint_update_microradians=(
            maximum_joint_update_microradians
        ),
        correction_smoothing_kernel_weights=(1, 4, 6, 4, 1),
        correction_smoothing_passes=2,
        active_contact_anchor_target_micrometres=(
            active_contact_anchor_target_micrometres
        ),
    )


if __name__ == "__main__":
    unittest.main()
