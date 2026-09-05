import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_neuralresonator_finetune as experiment


class FineTuneTests(unittest.TestCase):
    def test_shapes_are_distinct_convex_and_reproducible(self):
        first, second = experiment.polygons(), experiment.polygons()
        self.assertEqual(len(first), 12)
        masks = set()
        for a, b in zip(first, second):
            np.testing.assert_array_equal(a, b)
            self.assertTrue(np.all((a > 0) & (a < 1)))
            edges = np.roll(a, -1, axis=0) - a
            next_edges = np.roll(edges, -1, axis=0)
            self.assertTrue(
                np.all(
                    edges[:, 0] * next_edges[:, 1] - edges[:, 1] * next_edges[:, 0] > 0
                )
            )
            masks.add(experiment.reference.mask_for(a).numpy().tobytes())
        self.assertEqual(len(masks), 12)

    def test_losses_zero_for_identity_and_measure_gain(self):
        time = torch.arange(32000) / 32000
        wave = (torch.sin(2 * torch.pi * 800 * time) * torch.exp(-20 * time))[None]
        spectral, temporal = experiment.losses(wave, wave)
        self.assertEqual(float(spectral), 0)
        self.assertEqual(float(temporal), 0)
        spectral, temporal = experiment.losses(wave * 2, wave)
        self.assertAlmostEqual(float(spectral), 1, places=5)
        self.assertAlmostEqual(float(temporal), 1, places=3)

    def test_frequency_renderer_has_finite_gradients_and_causal_identity(self):
        coefficients = torch.zeros(1, 32, 2, 6)
        coefficients[..., 0] = coefficients[..., 3] = 1
        coefficients.requires_grad_(True)
        wave = experiment.differentiable_wave(coefficients)
        expected = torch.zeros(1, 32000)
        expected[0, 0] = 32
        torch.testing.assert_close(wave, expected, atol=1e-6, rtol=1e-6)
        wave.square().sum().backward()
        self.assertTrue(torch.isfinite(coefficients.grad).all())


if __name__ == "__main__":
    unittest.main()
