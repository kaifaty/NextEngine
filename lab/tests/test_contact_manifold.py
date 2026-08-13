from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_manifold import (
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


if __name__ == "__main__":
    unittest.main()
