from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_gradient_probe as probe


class GradientProbeTests(unittest.TestCase):
    def test_cosine_includes_opposition_and_zero_is_unavailable(self):
        self.assertEqual(probe.cosine([1, 0], [2, 0]), 1)
        self.assertEqual(probe.cosine([1, 0], [-2, 0]), -1)
        self.assertEqual(probe.cosine([1, 0], [0, 2]), 0)
        self.assertIsNone(probe.cosine([0, 0], [2, 0]))
        with self.assertRaises(ValueError):
            probe.cosine([1], [np.nan])

    def test_gain_projection_distinguishes_gain_from_new_signal(self):
        a = np.array([1.0, -1.0, 1.0, -1.0])
        result = probe.gain_projection(2 * a, a)
        self.assertEqual(result["least_squares_gain_to_fm"], 2)
        self.assertEqual(result["gain_fit_residual_power_fraction"], 0)
        result = probe.gain_projection(np.array([1.0, 1.0, -1.0, -1.0]), a)
        self.assertEqual(result["gain_fit_residual_power_fraction"], 1)
        with self.assertRaises(ValueError):
            probe.gain_projection(a, np.zeros(4))


if __name__ == "__main__":
    unittest.main()
