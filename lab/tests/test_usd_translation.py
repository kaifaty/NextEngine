from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from next_lab.usd_translation import render_usda, translate_to_store


def descriptor() -> dict:
    actuator_ids = [f"actuator.{index:02}" for index in range(23)]
    return {
        "schema_version": 2,
        "translator_version": "nextengine.isaac-usda-translator.v2",
        "observation_width": 84,
        "action_width": 23,
        "body_schema_hash": "12" * 32,
        "physics_hz": 240,
        "motor_hz": 60,
        "ordered_body_ids": ["body.root", "body.child"],
        "ordered_actuator_ids": actuator_ids,
        "bodies": [
            {
                "body_id": "body.child",
                "parent_body_id": "body.root",
                "local_bind_translation_micrometres": [100_000, 200_000, 300_000],
                "mass_microkilograms": 1_000_000,
                "colliders": [
                    {
                        "collider_id": "collider.child",
                        "geometry": {"kind": "sphere", "radius_micrometres": 100_000},
                    }
                ],
            },
            {
                "body_id": "body.root",
                "parent_body_id": None,
                "local_bind_translation_micrometres": [0, 1_050_000, 0],
                "mass_microkilograms": 10_000_000,
                "colliders": [
                    {
                        "collider_id": "collider.root",
                        "geometry": {"kind": "sphere", "radius_micrometres": 200_000},
                    }
                ],
            },
        ],
        "joints": [
            {
                "joint_id": "joint.child",
                "parent_body_id": "body.root",
                "child_body_id": "body.child",
                "parent_translation_micrometres": [100_000, 200_000, 300_000],
                "child_translation_micrometres": [0, 0, 0],
                "limit_min_microradians": -1_000_000,
                "limit_max_microradians": 1_000_000,
            }
        ],
        "actuators": [{"actuator_id": value} for value in actuator_ids],
        "environment_profiles": [
            _profile("nextengine.motor.env.humanoid-standing.v1", "world", 3_600, 8),
            _profile("nextengine.motor.env.humanoid-flat-command.v1", "root-local", 1_200, 10),
        ],
    }


def _profile(profile_id: str, velocity_frame: str, maximum_steps: int, reward_count: int) -> dict:
    standing = [
        "reward.upright",
        "reward.root-height-tracking",
        "reward.standing-pose-tracking",
        "reward.velocity-penalty",
        "reward.effort-penalty",
        "reward.action-rate-penalty",
        "reward.foot-slip-penalty",
        "reward.fall-terminal",
    ]
    locomotion = [
        "reward.planar-command-tracking",
        "reward.yaw-rate-tracking",
        "reward.upright-yaw-invariant",
        "reward.root-height-tracking",
        "reward.vertical-velocity-cost",
        "reward.roll-pitch-rate-cost",
        "reward.normalized-applied-effort-cost",
        "reward.applied-action-rate-cost",
        "reward.contacting-foot-tangential-slip-cost",
        "reward.fall-component",
    ]
    profile = {
        "profile_id": profile_id,
        "velocity_frame": velocity_frame,
        "maximum_episode_steps": maximum_steps,
        "observation_source_ids": [f"observation.{index}" for index in range(84)],
        "reward_components": [
            {"component_id": value} for value in (standing if reward_count == 8 else locomotion)
        ],
    }
    for field in (
        "manifest_hash",
        "observation_layout_hash",
        "action_layout_hash",
        "command_schedule_profile_hash",
        "reward_profile_hash",
        "termination_profile_hash",
        "rng_derivation_profile_hash",
        "correspondence_profile_hash",
    ):
        profile[field] = "12" * 32
    if reward_count == 10:
        profile["command_profile"] = {
            "warmup_ticks": 60,
            "segment_ticks": 120,
            "episode_ticks": 1_200,
            "mode_weights_basis_points": [2_500, 3_500, 2_000, 2_000],
        }
    return profile


class UsdTranslationTests(unittest.TestCase):
    def test_translation_is_stable_and_maps_engine_y_up_to_usd_z_up(self) -> None:
        first = render_usda(descriptor())
        second = render_usda(descriptor())
        self.assertEqual(first, second)
        self.assertIn('upAxis = "Z"', first)
        self.assertIn("double3 xformOp:translate = (0, 0, 1.05)", first)
        self.assertIn("double3 xformOp:translate = (0.1, -0.3, 1.25)", first)
        self.assertIn("rel physics:body0 = </Humanoid/Bodies/body_root>", first)
        self.assertIn("point3f physics:localPos0 = (0.1, -0.3, 0.2)", first)
        self.assertIn("point3f physics:localPos1 = (0, 0, 0)", first)
        self.assertEqual(first.count("{"), first.count("}"))
        self.assertNotIn("\n{\n    {\n", first)

    def test_generated_usd_and_manifest_live_in_external_store(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            manifest = translate_to_store(descriptor(), store, Path(__file__).parents[2])
            output = store / "derived" / ("12" * 32)
            self.assertTrue((output / "humanoid.usda").is_file())
            self.assertTrue((output / "translation-manifest.json").is_file())
            self.assertEqual(len(manifest["usd_sha256"]), 64)


if __name__ == "__main__":
    unittest.main()
