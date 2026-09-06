import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_pilot as p
import physical_sound_pouring_timbre_probe as t


class TimbreProbeTests(unittest.TestCase):
    def test_profile_ignores_recording_gain(self):
        wave = np.random.default_rng(53).normal(0, 0.01, p.SAMPLES).astype(np.float32)
        np.testing.assert_allclose(t.profile(wave), t.profile(wave * 13), atol=1e-5)
        with self.assertRaises(ValueError):
            t.profile(np.zeros(p.SAMPLES))

    def test_phase_zero_and_recoverable_conditioned_signal(self):
        rng = np.random.default_rng(53)
        c = rng.uniform(0.1, 0.8, (100, 11))
        targets = t.features(c) @ rng.normal(size=(11, 32))
        model = t.fit(c, targets)
        scores = t.predict(model, c)
        self.assertLess(np.mean((scores["conditioned"] - targets) ** 2), 0.01)
        self.assertGreater(np.mean((scores["shuffled"] - targets) ** 2), 0.1)
        c[:, 4] = 0
        for k in ["conditioned", "global", "shuffled"]:
            np.testing.assert_array_equal(t.predict(model, c)[k], np.zeros((100, 32)))

    def test_recolor_identity_and_level_preservation(self):
        wave = np.random.default_rng(53).normal(0, 0.01, p.SAMPLES).astype(np.float32)
        identity = t.recolor(wave, wave, np.zeros(32))
        np.testing.assert_allclose(identity, wave, atol=1e-7)
        delta = np.linspace(-5, 5, 32)
        changed = t.recolor(wave, wave, delta)
        self.assertLess(
            np.sqrt(np.mean((t.profile(changed) - t.profile(wave) - delta) ** 2)), 0.3
        )
        self.assertAlmostEqual(
            float(np.mean(changed**2)), float(np.mean(wave**2)), places=9
        )
        with self.assertRaises(ValueError):
            t.recolor(wave, wave, np.full(32, 100))


if __name__ == "__main__":
    unittest.main()
