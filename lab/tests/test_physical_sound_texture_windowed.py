from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_windowed as windowed


class WindowedTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)

    def test_all_bounded_lengths_are_covered_with_halos(self):
        for n in range(32, 257):
            counts = np.zeros(n, int)
            for first, low, high in windowed.windows(n):
                self.assertLessEqual(first + 32, n)
                counts[first + low : first + high] += 1
                self.assertTrue(first == 0 or low == 10)
                self.assertTrue(first + 32 == n or high == 22)
            self.assertTrue((counts > 0).all())
        for n in (31, 257, 32.0):
            with self.assertRaises(ValueError):
                windowed.windows(n)

    def test_training_length_sampler_is_exactly_unchanged(self):
        model = windowed.flow.TextureFlow(7).eval()
        torch.nn.init.normal_(model.output.weight, std=0.01)
        physical = windowed.surface.features(
            "Glass", 0.4, 0.38, np.full(32, 40), np.full(32, 0.5)
        )
        a = windowed.flow.sample_features(model, physical[None], 314, 32)
        b = windowed.flow.sample_features(
            windowed.WindowedField(model), physical[None], 314, 32
        )
        torch.testing.assert_close(a, b, rtol=0, atol=0)

    def test_remote_dependency_is_not_only_the_convolution(self):
        model = windowed.flow.TextureFlow(7).eval()
        torch.nn.init.normal_(model.output.weight, std=0.01)
        result = windowed.field_diagnostic(model)
        self.assertGreater(result["global_remote_change_rms"], 1e-5)
        self.assertEqual(result["windowed_remote_change_rms"], 0)


if __name__ == "__main__":
    unittest.main()
