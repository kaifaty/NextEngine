from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v5_capacity_train as capacity_train
import physical_sound_contact_field_r3a_v5_common as common


class PhysicalSoundContactFieldR3AV5CapacityTrainTests(unittest.TestCase):
    def test_learning_rate_matches_frozen_warmup_and_cosine_endpoints(self) -> None:
        base = common.TRAINING_CONFIG["learning_rate"]
        self.assertEqual(
            capacity_train.learning_rate_for_step(1),
            base / common.TRAINING_CONFIG["warmup_steps"],
        )
        self.assertEqual(
            capacity_train.learning_rate_for_step(
                common.TRAINING_CONFIG["warmup_steps"]
            ),
            base,
        )
        self.assertTrue(
            math.isclose(
                capacity_train.learning_rate_for_step(
                    common.TRAINING_CONFIG["maximum_steps"]
                ),
                base * 0.05,
            )
        )

    def test_stateless_sampler_repeats_item_crop_and_mask(self) -> None:
        items = [
            capacity_train.WaveformItem(
                id=f"item-{index}",
                role="train",
                source_kind="fixture",
                source_group=f"group-{index}",
                samples=np.linspace(-0.9, 0.9, 60_000 + index, dtype="<f4"),
                samples_sha256="0" * 64,
                output_gain=1.0,
            )
            for index in range(3)
        ]
        first = capacity_train.segment_for_step(items, "rvq-6kbps", 77)
        second = capacity_train.segment_for_step(items, "rvq-6kbps", 77)
        self.assertEqual(first[0].id, second[0].id)
        self.assertEqual(first[3], second[3])
        np.testing.assert_array_equal(first[1], second[1])
        np.testing.assert_array_equal(first[2], second[2])

    def test_fit_projection_decodes_only_first_three_rows(self) -> None:
        values = np.zeros((4, 64), dtype="<f4")
        values[0, 3] = 0.2
        values[1, 4] = 0.4
        values[2, 5] = 0.6
        values[3] = np.nan
        metadata = {
            "contacts": [
                {"role": "fit"},
                {"role": "fit"},
                {"role": "fit"},
                {"role": "representation_development"},
            ]
        }
        items = capacity_train.extract_fit_rows(values, metadata, "fixture")
        self.assertEqual(len(items), 3)
        self.assertTrue(all(np.isfinite(item.samples).all() for item in items))
        self.assertTrue(all(item.role == "fit" for item in items))

    def test_short_segment_mask_excludes_right_padding(self) -> None:
        item = capacity_train.WaveformItem(
            id="short",
            role="train",
            source_kind="fixture",
            source_group="fixture",
            samples=np.ones(1_000, dtype="<f4") * 0.2,
            samples_sha256="0" * 64,
            output_gain=1.0,
        )
        _, segment, mask, _ = capacity_train.segment_for_step([item], "rvq-12kbps", 4)
        self.assertEqual(segment.size, common.TRAINING_SEGMENT_SAMPLES)
        self.assertEqual(mask.size, common.TRAINING_SEGMENT_SAMPLES)
        self.assertEqual(float(mask.sum()), 1_000.0)
        self.assertEqual(float((segment * (1.0 - mask)).sum()), 0.0)


if __name__ == "__main__":
    unittest.main()
