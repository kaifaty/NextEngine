import sys
import unittest
from pathlib import Path

import numpy as np
import torch
from PIL import Image
from transformers import BitImageProcessor

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_syncfusion_visual as visual


class VisualTests(unittest.TestCase):
    def test_full_frame_preserves_boundary_marker_stock_crop_discards(self):
        processor = BitImageProcessor(
            size={"shortest_edge": 256},
            crop_size={"height": 224, "width": 224},
            do_normalize=False,
            do_rescale=False,
        )
        values = np.zeros((240, 320, 3), dtype=np.uint8)
        values[150:210, :35, 0] = 255
        image = Image.fromarray(values)
        stock = visual.image_inputs(processor, image)["pixel_values"]
        full = visual.image_inputs(processor, image, "full-frame-letterbox-224")[
            "pixel_values"
        ]
        self.assertEqual(tuple(full.shape), (1, 3, 224, 224))
        self.assertEqual(int(stock.max()), 0)
        red = (full[0, 0] > 240) & (full[0, 1] < 10)
        self.assertGreater(int(red.sum()), 500)
        self.assertTrue(
            torch.equal(
                stock, processor(images=image, return_tensors="pt")["pixel_values"]
            )
        )
        with self.assertRaises(ValueError):
            visual.image_inputs(processor, image, "unknown")

    def test_frame_time_is_preimpact_and_one_based(self):
        index = visual.frame_index(23.662781, 15)
        self.assertEqual(index, 354)
        self.assertLessEqual((index - 1) / 15, 23.662781 - 0.1)
        for time, fps in [(-1, 15), (float("nan"), 15), (1, 30)]:
            with self.assertRaises(ValueError):
                visual.frame_index(time, fps)

    def test_mismatch_preserves_descriptor_but_changes_recording(self):
        rows = [
            {"recording": name, "material": "glass", "motion": "static"}
            for name in ["a", "a", "b"]
        ]
        self.assertEqual(visual.mismatch_indices(rows), [2, 2, 0])
        with self.assertRaises(ValueError):
            visual.mismatch_indices(rows[:2])

    def test_development_images_and_targets_cannot_change_ridge_fit(self):
        torch.manual_seed(42)
        rows = [
            {
                "recording": str(i),
                "role": "train" if i < 5 else "recording-dev",
                "material": "wood",
                "motion": "static",
            }
            for i in range(6)
        ]
        images = torch.randn(6, 8)
        baseline, targets = torch.randn(6, 16), torch.randn(6, 16)
        mean, mapping = visual.ridge_residual(images, baseline, targets, rows)
        images[-1] *= -100
        targets[-1] *= 100
        other_mean, other_mapping = visual.ridge_residual(
            images, baseline, targets, rows
        )
        self.assertTrue(torch.equal(mean, other_mean))
        self.assertTrue(torch.equal(mapping, other_mapping))

    def test_recording_leakage_rejected(self):
        rows = [
            {"recording": "same", "role": role, "material": "wood", "motion": "static"}
            for role in ["train", "recording-dev"]
        ]
        with self.assertRaises(ValueError):
            visual.ridge_residual(
                torch.ones(2, 3), torch.ones(2, 4), torch.ones(2, 4), rows
            )


if __name__ == "__main__":
    unittest.main()
