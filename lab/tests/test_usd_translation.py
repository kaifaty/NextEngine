from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from next_lab.usd_translation import render_usda, translate_to_store


def descriptor() -> dict:
    actuator_ids = [f"actuator.{index:02}" for index in range(23)]
    return {
        "schema_version": 1,
        "translator_version": "nextengine.isaac-usda-translator.v1",
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
                "limit_min_microradians": -1_000_000,
                "limit_max_microradians": 1_000_000,
            }
        ],
        "actuators": [{"actuator_id": value} for value in actuator_ids],
    }


class UsdTranslationTests(unittest.TestCase):
    def test_translation_is_stable_and_maps_engine_y_up_to_usd_z_up(self) -> None:
        first = render_usda(descriptor())
        second = render_usda(descriptor())
        self.assertEqual(first, second)
        self.assertIn('upAxis = "Z"', first)
        self.assertIn("double3 xformOp:translate = (0, 0, 1.05)", first)
        self.assertIn("rel physics:body0 = </Humanoid/Bodies/body_root>", first)

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
