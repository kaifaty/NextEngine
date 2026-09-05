import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_resonance_probe as r


class ResonanceProbeTests(unittest.TestCase):
    def test_noise_ridge_is_not_automatically_admitted(self):
        wave, _, _ = r.synthetic("noise")
        report, _, _ = r.inspect(wave)
        self.assertFalse(report["automatic_label_admission"])
        # Smooth paths and a positive shuffle margin also occur in pure noise.
        self.assertGreater(report["score_above_max_shuffle"], 0)

    def test_known_rising_and_falling_tracks(self):
        for kind in ("rising", "falling"):
            wave, times, targets = r.synthetic(kind)
            frames, spectrum, emissions = r.analyze(wave)
            path, score = r.track(emissions)
            truth = np.interp(frames, times, targets[0])
            error = abs(1200 * np.log2(r.GRID[path] / truth))[4:-4]
            self.assertLess(np.median(error), 25)
            self.assertGreater(np.mean(error < 50), 0.95)
            backwards, reverse_score = r.track(emissions[:, ::-1])
            self.assertAlmostEqual(score, reverse_score, places=8)
            np.testing.assert_array_equal(path, backwards[::-1])
            ridge, residual = r.reconstruct(wave, frames, spectrum, path)
            np.testing.assert_allclose(ridge + residual, wave, atol=1e-7)

    def test_gain_invariance_and_invalid_inputs(self):
        wave, _, _ = r.synthetic("rising")
        a = r.analyze(wave)[2]
        b = r.analyze(wave * 2)[2]
        np.testing.assert_allclose(a, b, atol=1e-5)
        for invalid in (np.zeros(3000), np.full(3000, np.nan), np.ones(4)):
            with self.assertRaises(ValueError):
                r.analyze(invalid)
        with self.assertRaises(ValueError):
            r.track(np.zeros((2, 10)))


if __name__ == "__main__":
    unittest.main()
