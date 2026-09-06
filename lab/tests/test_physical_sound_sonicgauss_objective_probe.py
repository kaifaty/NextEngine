import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_objective_probe as probe


class ObjectiveProbeTests(unittest.TestCase):
    def test_exact_convex_optimum_and_missing_frequency_shortcut(self):
        a, b, _ = probe.toy_signals()
        self.assertAlmostEqual(probe.optimum_gain(a, a)["optimal_gain"], 1, places=6)
        self.assertAlmostEqual(
            probe.optimum_gain(a, 2 * a)["optimal_gain"], 0.5, places=6
        )
        wrong = probe.optimum_gain(a, b)
        self.assertGreater(wrong["gradient_at_gain_one"], 0)
        self.assertLess(wrong["optimal_gain"], 0.05)
        self.assertLess(wrong["loss_at_optimum"], wrong["loss_at_gain_one"])
        with self.assertRaisesRegex(ValueError, "silent"):
            probe.optimum_gain(np.zeros_like(a), b)

    def test_timing_control_and_lossless_translation(self):
        a = probe.toy_signals()[0]
        translated = probe.translate(a, 88)
        np.testing.assert_array_equal(translated[16384 + 88 : 16384 + 88 + len(a)], a)
        self.assertEqual(probe.lag_frames(probe.translate(a, 0), translated), 1)
        with self.assertRaisesRegex(ValueError, "guard"):
            probe.translate(a, 20000)

    def test_distribution_term_rejects_silence_shortcut(self):
        scores = probe.energy_choices(probe.toy_signals())
        correct = scores["correct_empirical_distribution"]
        silence = scores["silence"]
        self.assertLess(silence["paired_distance"], correct["paired_distance"])
        self.assertLess(correct["energy_score"], silence["energy_score"])
        self.assertLess(
            correct["energy_score"], scores["collapsed_first_tone"]["energy_score"]
        )


if __name__ == "__main__":
    unittest.main()
