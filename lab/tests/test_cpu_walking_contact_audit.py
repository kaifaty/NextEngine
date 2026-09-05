from __future__ import annotations

import copy
import unittest

import numpy as np

from lab.scripts.cpu_walking_contact_audit import (
    analyze,
    classified_support,
    foot_box,
    sole_support,
)


def fixture():
    bodies = [{"body_id": "root", "body_token": 1000}]
    links = [
        {
            "body_token": 1000,
            "position_um": [0, 1_000_000, 0],
            "rotation_q1_30": [0, 0, 0, 1 << 30],
            "linear_velocity_um_s": [0, 0, 0],
        }
    ]
    for side, token in (("left", 1006), ("right", 1012)):
        bodies.append(
            {
                "body_id": f"body.{side}-ankle-roll",
                "body_token": token,
                "colliders": [
                    {
                        "local_rotation_q1_30": [0, 0, 0, 1 << 30],
                        "local_translation_micrometres": [0, 0, 0],
                        "geometry": {
                            "kind": "box",
                            "half_extents_micrometres": [50_000, 30_000, 130_000],
                        },
                    }
                ],
            }
        )
        links.append(
            {
                "body_token": token,
                "position_um": [0, 30_000, 0],
                "rotation_q1_30": [0, 0, 0, 1 << 30],
                "linear_velocity_um_s": [0, 0, 0],
            }
        )
    contacts = [
        {
            "actor_tokens": [1, 1006],
            "position_um": [0, 0, z],
            "impulse_uns": [0, impulse, 0],
        }
        for z, impulse in ((-130_000, 1_000_000), (130_000, 3_000_000))
    ]
    frame = {
        "tick": 1,
        "links": links,
        "contacts": contacts,
        "joint_position_urad": [100_000],
        "contact_flags": [1, 1],
        "command_raw": [0, 500_000, 0],
    }
    descriptor = {"bodies": bodies, "actuators": [{"dof_ordinal": 0}]}
    evaluation = {
        "root_position_m": np.array([[0, 1, 0]]),
        "root_quaternion_xyzw": np.array([[0, 0, 0, 1]]),
        "root_velocity_mps": np.zeros((1, 3)),
        "joint_position_rad": np.array([[0.1]]),
        "command": np.array([[0, 0.5, 0]]),
        "contact_occupancy": np.array([[True, True]]),
    }
    return {"frames": [[frame]]}, descriptor, evaluation


class ContactAuditTests(unittest.TestCase):
    def substep_fixture(self, loads):
        trace, descriptor, _ = fixture()
        frame = trace["frames"][0][0]
        frame["completed_physics_substeps"] = len(loads)
        frame["classified_contact_substeps"] = [
            {
                "ordinal": ordinal,
                "contacts": [
                    {
                        "actor_tokens": [1, token],
                        "class": "SoleSupport",
                        "impulse_uns": [0, impulse, 0],
                    }
                    for token, impulse in zip((1006, 1012), pair, strict=True)
                ],
            }
            for ordinal, pair in enumerate(loads)
        ]
        return [frame], descriptor

    def test_full_tick_load_does_not_use_raw_presence_bits(self):
        frames, descriptor = self.substep_fixture([[100, 0]] * 4)
        result = classified_support(frames, descriptor)[0]
        self.assertEqual(frames[0]["contact_flags"], [1, 1])
        self.assertEqual(
            result["exclusive_positive_load_side_all_four_substeps_report_only"], 0
        )
        self.assertEqual(result["vertical_impulse_uns_by_substep"], [[100, 0]] * 4)

    def test_last_substep_cannot_hide_earlier_bilateral_load(self):
        frames, descriptor = self.substep_fixture([[100, 50]] + [[100, 0]] * 3)
        result = classified_support(frames, descriptor)[0]
        self.assertIsNone(
            result["exclusive_positive_load_side_all_four_substeps_report_only"]
        )

    def test_partial_and_zero_substeps_cannot_qualify(self):
        for count in range(4):
            frames, descriptor = self.substep_fixture([[0, -100]] * count)
            result = classified_support(frames, descriptor)[0]
            self.assertEqual(result["actual_substeps"], count)
            self.assertIsNone(
                result["exclusive_positive_load_side_all_four_substeps_report_only"]
            )

    def test_padding_and_reordered_substeps_reject(self):
        frames, descriptor = self.substep_fixture([[100, 0]] * 4)
        frames[0]["completed_physics_substeps"] = 3
        with self.assertRaisesRegex(ValueError, "count mismatch"):
            classified_support(frames, descriptor)
        frames[0]["completed_physics_substeps"] = 4
        frames[0]["classified_contact_substeps"][1]["ordinal"] = 0
        with self.assertRaisesRegex(ValueError, "order mismatch"):
            classified_support(frames, descriptor)

    def test_classified_contact_without_vertical_load_is_not_loaded_support(self):
        frames, descriptor = self.substep_fixture([[0, 0]] * 4)
        result = classified_support(frames, descriptor)[0]
        self.assertEqual(result["active_sole_contacts_by_substep"], [[True, True]] * 4)
        self.assertIsNone(
            result["exclusive_positive_load_side_all_four_substeps_report_only"]
        )

    def test_only_sole_class_and_integer_load_are_accepted(self):
        frames, descriptor = self.substep_fixture([[100, 0]] * 4)
        for substep in frames[0]["classified_contact_substeps"]:
            substep["contacts"][0]["class"] = "ForbiddenLocomotion"
        self.assertIsNone(
            classified_support(frames, descriptor)[0][
                "exclusive_positive_load_side_all_four_substeps_report_only"
            ]
        )
        contact = frames[0]["classified_contact_substeps"][0]["contacts"][1]
        contact["impulse_uns"][1] = 1.5
        with self.assertRaisesRegex(ValueError, "integer"):
            classified_support(frames, descriptor)

    def test_flat_sole_does_not_mean_body_origin_at_ground(self):
        trace, descriptor, evaluation = fixture()
        report, heights, _ = analyze(trace, descriptor, evaluation)
        self.assertTrue(report["exact_native_replay"])
        np.testing.assert_array_equal(heights, [[0, 0]])
        feet = sole_support(trace["frames"][0], descriptor)
        left = feet[0]["selected_frames_including_maximum_heel_raise"][0]
        self.assertEqual(left["heel_above_toe_m"], 0)
        self.assertEqual(left["pressure_heel_to_toe_fraction"], 0.75)
        self.assertIsNone(
            feet[1]["selected_frames_including_maximum_heel_raise"][0][
                "pressure_heel_to_toe_fraction"
            ]
        )

    def test_contact_presence_cannot_override_measured_clearance(self):
        trace, descriptor, evaluation = fixture()
        trace["frames"][0][0]["links"][2]["position_um"][1] += 10_000
        report, heights, _ = analyze(trace, descriptor, evaluation)
        self.assertAlmostEqual(heights[0, 1], 0.01)
        right = report["feet"][1]
        self.assertEqual(right["moving_contact_bit_with_clearance_above_5mm"], 1)
        self.assertEqual(
            right["moving_contact_bit_with_zero_last_substep_vertical_impulse"], 1
        )

    def test_exact_replay_rejects_pose_velocity_joint_command_and_contact_corruption(
        self,
    ):
        for key in (
            "root_position_m",
            "root_quaternion_xyzw",
            "root_velocity_mps",
            "joint_position_rad",
            "command",
            "contact_occupancy",
        ):
            trace, descriptor, evaluation = fixture()
            evaluation[key] = evaluation[key].astype(np.float64)
            evaluation[key][0, 0] = 1 if key == "contact_occupancy" else 0.125
            if key == "contact_occupancy":
                evaluation[key][0, 0] = False
            with self.subTest(key=key), self.assertRaises(AssertionError):
                analyze(trace, descriptor, evaluation)

    def test_positive_pitch_raises_heel_not_toe_and_preserves_box_length(self):
        trace, descriptor, _ = fixture()
        body, link = (
            descriptor["bodies"][1],
            copy.deepcopy(trace["frames"][0][0]["links"][1]),
        )
        angle = np.deg2rad(10)
        link["rotation_q1_30"] = np.rint(
            np.array([np.sin(angle / 2), 0, 0, np.cos(angle / 2)]) * (1 << 30)
        ).astype(np.int64)
        corners, _, _, _ = foot_box(body, link)
        heel, toe = corners[:2].mean(axis=0), corners[2:4].mean(axis=0)
        self.assertAlmostEqual(heel[1] - toe[1], 0.26 * np.sin(angle), places=8)
        self.assertAlmostEqual(np.linalg.norm(heel - toe), 0.26, places=8)

    def test_non_box_sole_fails_closed(self):
        trace, descriptor, _ = fixture()
        body, link = descriptor["bodies"][1], trace["frames"][0][0]["links"][1]
        body["colliders"][0]["geometry"]["kind"] = "sphere"
        with self.assertRaisesRegex(ValueError, "box collider"):
            foot_box(body, link)


if __name__ == "__main__":
    unittest.main()
