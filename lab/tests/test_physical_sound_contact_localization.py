import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_contact_localization as contact


class ContactTests(unittest.TestCase):
    def setUp(self):
        self.frame = (
            np.random.default_rng(42).integers(30, 150, (240, 320, 3)).astype(float)
        )

    def test_integer_camera_translation_is_removed(self):
        frames = [np.roll(self.frame, (i, -i), axis=(0, 1)) for i in range(-3, 4)]
        result, energy = contact.motion_region(frames)
        self.assertEqual(result["reason"], "no residual motion")
        self.assertEqual(float(energy.max()), 0)

    def test_moving_patch_localizes_with_translating_camera(self):
        frames = []
        for i in range(-3, 4):
            value = self.frame.copy()
            value[95:110, 130 + i * 4 : 145 + i * 4] = 250
            frames.append(np.roll(value, (i, -i), axis=(0, 1)))
        result, _ = contact.motion_region(frames)
        x, y, w, h = result["box_xywh"]
        self.assertLessEqual(x, 118)
        self.assertGreaterEqual(x + w, 157)
        self.assertLessEqual(y, 95)
        self.assertGreaterEqual(y + h, 110)
        self.assertGreater(result["motion_mass_fraction"], 0.99)

    def test_invalid_layout_and_nonfinite_are_rejected(self):
        for frames in (
            [self.frame] * 6,
            [np.zeros((20, 20, 3))] * 7,
            [np.full((240, 320, 3), np.nan)] * 7,
        ):
            with self.assertRaises(ValueError):
                contact.motion_region(frames)

    def test_large_translation_abstains(self):
        frames = [self.frame] * 7
        frames[0] = np.roll(self.frame, 40, axis=0)
        result, energy = contact.motion_region(frames)
        self.assertEqual(result["reason"], "large global translation")
        self.assertIsNone(energy)


if __name__ == "__main__":
    unittest.main()
