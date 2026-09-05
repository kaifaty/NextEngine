import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_phase_oracle as o


class PhaseOracleTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)

    def test_roundtrip_and_coarse_constant(self):
        wave = torch.randn(o.d.p.SAMPLES) * 0.01
        torch.testing.assert_close(
            o.inverse(o.transform(wave)), wave, atol=1e-8, rtol=1e-5
        )
        magnitude = torch.full((513, 256), 0.1)
        torch.testing.assert_close(o.coarse(magnitude), magnitude)

    def test_phase_iterations_reduce_magnitude_error(self):
        t = torch.arange(o.d.p.SAMPLES) / o.d.p.RATE
        wave = 0.01 * torch.sin(2 * torch.pi * (500 * t + 100 * t.square()))
        magnitude = o.transform(wave).abs()
        first = o.resynthesize(wave, 2718, "noise_phase")
        refined = o.resynthesize(wave, 2718, "iterative_phase")
        self.assertLess(
            float((o.transform(refined).abs() - magnitude).square().mean()),
            float((o.transform(first).abs() - magnitude).square().mean()),
        )
        torch.testing.assert_close(
            o.resynthesize(wave, 2718, "noise_phase"), first, rtol=0, atol=0
        )
        for mode in o.MODES:
            self.assertTrue(torch.isfinite(o.resynthesize(wave, 314, mode)).all())

    def test_invalid_inputs(self):
        for wave, seed, mode in [
            (torch.zeros(4), 1, "noise_phase"),
            (torch.full((o.d.p.SAMPLES,), float("nan")), 1, "noise_phase"),
            (torch.zeros(o.d.p.SAMPLES), -1, "noise_phase"),
            (torch.zeros(o.d.p.SAMPLES), 1, "unknown"),
        ]:
            with self.assertRaises(ValueError):
                o.resynthesize(wave, seed, mode)

    def test_source_free_zero_step_matches_decoder(self):
        model = o.d.TemporalDecoder()
        controls = o.d.controls_for()
        np.testing.assert_allclose(
            o.refine_generated(model, controls, 2718, 0),
            o.d.sample(model, controls, 2718),
            atol=2e-8,
            rtol=1e-5,
        )
        self.assertTrue(np.isfinite(o.refine_generated(model, controls, 2718)).all())
        with self.assertRaises(ValueError):
            o.refine_generated(model, controls, -1)


if __name__ == "__main__":
    unittest.main()
