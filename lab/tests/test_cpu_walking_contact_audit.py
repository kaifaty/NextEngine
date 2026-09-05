from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

import numpy as np

from lab.scripts.cpu_walking_contact_audit import (
    analyze,
    canonical_box_height_um,
    classified_support,
    foot_box,
    lifted_support_report,
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
    def test_corrected_matrix_preserves_original_task_and_final_selection(self):
        profiles = Path(__file__).resolve().parents[1] / "profiles"
        training = json.loads(
            (profiles / "canonical-rsl-rl-walking.v3.json").read_text()
        )
        corrected = json.loads(
            (profiles / "canonical-walking-corrected-evaluation.v1.json").read_text()
        )
        self.assertEqual(corrected["evaluation"], training["evaluation"])
        self.assertEqual(
            corrected["source_checkpoint_iteration"], training["iterations"] - 1
        )
        self.assertEqual(corrected["source_checkpoint_name"], "model_9999.pt")
        self.assertEqual(corrected["mode"], "final-weights-only-inference-transfer")

    def test_canonical_height_rounds_q30_ties_even_then_floors_negative_height(self):
        trace, descriptor, _ = fixture()
        body = descriptor["bodies"][1]
        link = trace["frames"][0][0]["links"][1]
        # Synthetic bounded coefficients isolate rounding; no physical-pose claim.
        for x, expected in ((1, 0), (3, -1), (-1, 0), (-3, -1)):
            link["rotation_q1_30"] = [x, 0, 0, 1 << 28]
            with self.subTest(x=x):
                self.assertEqual(canonical_box_height_um(body, link), expected)

    def test_canonical_height_rejects_rotated_or_invalid_boxes_and_overflow(self):
        for invalid in ("rotation", "extent", "quaternion", "overflow"):
            trace, descriptor, _ = fixture()
            body = descriptor["bodies"][1]
            link = trace["frames"][0][0]["links"][1]
            if invalid == "rotation":
                body["colliders"][0]["local_rotation_q1_30"] = [1, 0, 0, 1 << 30]
            elif invalid == "extent":
                body["colliders"][0]["geometry"]["half_extents_micrometres"][0] = 0
            elif invalid == "quaternion":
                link["rotation_q1_30"][3] += 1
            else:
                link["position_um"][1] = -(1 << 63)
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                canonical_box_height_um(body, link)

    def test_penetrating_free_foot_cannot_qualify(self):
        frames, descriptor = self.lifted_fixture([(0, -1)] * 8)
        self.assertEqual(
            lifted_support_report(frames, descriptor)["load_and_lift"][
                "ticks_by_stance_side"
            ],
            [0, 0],
        )

    def lifted_fixture(self, sequence):
        frames = []
        for index, (side, height) in enumerate(sequence):
            loads = [100, 0] if side == 0 else [0, 100]
            row, descriptor = self.substep_fixture([loads] * 4)
            frame = row[0]
            frame["tick"] = index + 1
            frame["observation_raw"] = [0] * 88
            frame["observation_raw"][86 + (1 - side)] = height
            frame["links"][1 + (1 - side)]["position_um"][1] += height
            frames.append(frame)
        descriptor.update(
            observation_width=88,
            environment_profiles=[
                {
                    "profile_id": "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v7"
                }
            ],
        )
        return frames, descriptor

    def test_grounded_unloading_cannot_qualify_as_lift(self):
        frames, descriptor = self.lifted_fixture([(0, 0)] * 8)
        report = lifted_support_report(frames, descriptor)
        self.assertEqual(report["load_only"]["ticks_by_stance_side"], [8, 0])
        self.assertEqual(report["load_and_lift"]["ticks_by_stance_side"], [0, 0])
        self.assertEqual(report["status"], "report_only")

    def test_exact_positive_height_and_existing_duration_count_switches(self):
        frames, descriptor = self.lifted_fixture(
            [(0, 1)] * 8 + [(1, 20_000)] * 8 + [(0, 60_000)] * 8
        )
        report = lifted_support_report(frames, descriptor)["load_and_lift"]
        self.assertEqual(report["ticks_by_stance_side"], [16, 8])
        self.assertEqual(report["longest_continuous_ticks_by_stance_side"], [8, 8])
        self.assertEqual(report["switches_between_runs_of_at_least_eight_ticks"], 2)

    def test_grounded_gap_breaks_run_and_short_opposite_run_does_not_switch(self):
        frames, descriptor = self.lifted_fixture(
            [(0, 100)] * 7 + [(0, 0)] + [(0, 100)] * 7 + [(1, 100)] * 7
        )
        report = lifted_support_report(frames, descriptor)["load_and_lift"]
        self.assertEqual(report["longest_continuous_ticks_by_stance_side"], [7, 7])
        self.assertEqual(report["switches_between_runs_of_at_least_eight_ticks"], 0)
        self.assertEqual([s["ticks"] for s in report["segments"]], [7, 7, 7])

    def test_lift_does_not_override_partial_or_bilateral_loaded_substeps(self):
        frames, descriptor = self.lifted_fixture([(0, 10_000)] * 2)
        frames[0]["completed_physics_substeps"] = 3
        frames[0]["classified_contact_substeps"].pop()
        frames[1]["classified_contact_substeps"][0]["contacts"][1]["impulse_uns"][1] = 1
        self.assertEqual(
            lifted_support_report(frames, descriptor)["load_and_lift"][
                "ticks_by_stance_side"
            ],
            [0, 0],
        )

    def test_lift_report_rejects_layout_height_geometry_and_tick_corruption(self):
        for corruption in ("layout", "height", "geometry", "tick", "width"):
            frames, descriptor = self.lifted_fixture([(0, 10_000)])
            if corruption == "layout":
                descriptor["environment_profiles"][0]["profile_id"] = "unknown.v7"
            elif corruption == "height":
                frames[0]["observation_raw"][87] = True
            elif corruption == "geometry":
                frames[0]["observation_raw"][87] += 2
            elif corruption == "tick":
                frames[0]["tick"] = 2
            else:
                frames[0]["observation_raw"].pop()
            with self.subTest(corruption=corruption), self.assertRaises(ValueError):
                lifted_support_report(frames, descriptor)

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
