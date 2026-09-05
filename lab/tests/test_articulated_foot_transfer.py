import math
import unittest

from lab.scripts.audit_articulated_foot_transfer import (
    analyze,
    longest,
    rotate,
    sole_heights,
    velocity_violations,
    vertical_impulse,
)


class LoadedFootOracleTests(unittest.TestCase):
    def test_velocity_observation_tolerance_matches_native_safety_boundary(self):
        limits = {
            0: {
                "joint_id": "mtp",
                "maximum_velocity_microradians_per_second": 8_000_000,
            }
        }
        for sign in [-1, 1]:
            frame = {"joints": [{"ordinal": 0, "velocity_urad_s": sign * 8_001_000}]}
            self.assertEqual(velocity_violations(frame, limits), [])
            frame["joints"][0]["velocity_urad_s"] = sign * 8_001_001
            self.assertEqual(len(velocity_violations(frame, limits)), 1)

    def test_complete_loaded_interval_and_return_are_both_required(self):
        bodies = []
        for token, name in enumerate(
            ["left-ankle-roll", "left-mtp", "right-ankle-roll", "right-mtp"], 10
        ):
            bodies.append(
                {
                    "body_id": f"body.{name}",
                    "body_token": token,
                    "colliders": [
                        {
                            "geometry": {
                                "kind": "box",
                                "half_extents_micrometres": [50_000, 20_000, 100_000],
                            },
                            "local_translation_micrometres": [0, 20_000, 100_000],
                            "local_rotation_q1_30": [0, 0, 0, 1],
                        }
                    ],
                }
            )
        descriptor = {"body_schema_hash": "fixture", "bodies": bodies}
        frames = []
        for step in range(1, 3601):
            tick = (step + 3) // 4
            links = [
                {
                    "body_token": b["body_token"],
                    "position_um": [
                        0,
                        10_000 if b["body_token"] == 10 and 300 < tick <= 540 else 0,
                        0,
                    ],
                    "rotation_q1_30": [0, 0, 0, 1],
                }
                for b in bodies
            ]
            frames.append(
                {
                    "physics_substep": step,
                    "motor_tick": tick,
                    "links": links,
                    "joints": [],
                    "contacts": [
                        {"actors": [1, b["body_token"]], "impulse_uns": [0, 100_000, 0]}
                        for b in bodies
                    ],
                }
            )
        trace = {
            "probe": "nextengine.articulated-foot-transfer.v1",
            "body_schema_hash": "fixture",
            "side": "left",
            "reason": "terminal.timeout",
            "frames": frames,
        }
        result = analyze(descriptor, trace)["sides"]["left"]
        self.assertTrue(result["loaded_transfer_pass"])
        self.assertEqual(result["longest_loaded_heel_raise_substeps"], 960)
        self.assertEqual(result["longest_recontact_substeps"], 1440)
        for f in frames[1200:2160]:
            f["links"][1]["position_um"][1] = 10_000
        self.assertFalse(
            analyze(descriptor, trace)["sides"]["left"]["loaded_transfer_pass"]
        )
        for f in frames[1200:2160]:
            f["links"][1]["position_um"][1] = 0
            f["contacts"][1]["impulse_uns"][1] = 0
        self.assertFalse(
            analyze(descriptor, trace)["sides"]["left"]["loaded_transfer_pass"]
        )

    def test_scalar_rotation_and_collider_offset_are_not_body_origins(self):
        body = {
            "colliders": [
                {
                    "geometry": {
                        "kind": "box",
                        "half_extents_micrometres": [50_000, 20_000, 100_000],
                    },
                    "local_rotation_q1_30": [0, 0, 0, 1],
                    "local_translation_micrometres": [0, 20_000, 100_000],
                }
            ]
        }
        link = {"position_um": [0, 0, 0], "rotation_q1_30": [0, 0, 0, 1]}
        self.assertEqual(sole_heights(body, link), [0] * 4)
        link["rotation_q1_30"] = [math.sin(math.pi / 12), 0, 0, math.cos(math.pi / 12)]
        link["position_um"][1] = 100_000
        height = sole_heights(body, link)
        for value in height[:2]:
            self.assertAlmostEqual(value, 0.1)
        for value in height[2:]:
            self.assertAlmostEqual(value, 0)
        self.assertAlmostEqual(rotate([0, 0, 1, 1], [1, 0, 0])[1], 1)
        with self.assertRaises(ValueError):
            rotate([0, 0, 0, 0], [1, 0, 0])

    def test_reversal_and_net_load_not_unsigned_point_sum(self):
        frame = {
            "contacts": [
                {"actors": [1, 10], "impulse_uns": [0, 100_000, 0]},
                {"actors": [10, 1], "impulse_uns": [0, -50_000, 0]},
                {"actors": [1, 11], "impulse_uns": [0, 999_999, 0]},
            ]
        }
        self.assertEqual(vertical_impulse(frame, 10), 0.15)
        frame["contacts"][1]["impulse_uns"][1] = 50_000
        self.assertEqual(vertical_impulse(frame, 10), 0.05)

    def test_duration_requires_consecutive_substeps(self):
        self.assertEqual(longest([True] * 40 + [False] + [True] * 40), 40)
        self.assertEqual(longest([False] + [True] * 60), 60)


if __name__ == "__main__":
    unittest.main()
