import sys
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_syncfusion_visual as visual


class VisualTests(unittest.TestCase):
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
