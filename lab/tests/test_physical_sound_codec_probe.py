import sys
import unittest
from pathlib import Path

import numpy as np
import torch
from diffusers.models.autoencoders.autoencoder_oobleck import (
    OobleckDiagonalGaussianDistribution,
)

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_codec_probe as codec


class CodecTest(unittest.TestCase):
    def test_sample_matches_installed_distribution_and_repeats(self):
        distribution = OobleckDiagonalGaussianDistribution(
            torch.arange(24, dtype=torch.float32).reshape(1, 8, 3) / 20
        )
        expected = distribution.sample(torch.Generator().manual_seed(42))
        actual = codec.posterior_sample(distribution.mean, distribution.std, 42)
        self.assertTrue(torch.equal(expected, actual))
        self.assertFalse(
            torch.equal(
                actual, codec.posterior_sample(distribution.mean, distribution.std, 123)
            )
        )

    def test_invalid_posterior_rejected(self):
        mean = torch.zeros(1, 3, 4)
        for std in (
            torch.ones(1, 4, 3),
            torch.full_like(mean, -1),
            torch.full_like(mean, float("nan")),
        ):
            with self.assertRaises(ValueError):
                codec.posterior_sample(mean, std, 42)

    def test_padding_measurement_is_separate_from_active_signal(self):
        wave = np.zeros((2, 500), dtype=np.float32)
        wave[:, :150] = 0.1
        wave[:, 150:] = 0.01
        parts = codec.energy_regions(wave, 100)
        self.assertAlmostEqual(parts["padding_to_active_db"], -20, places=5)
        self.assertAlmostEqual(parts["padding_dbfs"], -40, places=5)
        with self.assertRaises(ValueError):
            codec.energy_regions(wave[:, :450], 100)


if __name__ == "__main__":
    unittest.main()
