import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_separation as separation


class SeparationTests(unittest.TestCase):
    @staticmethod
    def tone(frequency):
        t = np.arange(131072) / 44100
        mono = (0.03 * np.sin(2 * np.pi * frequency * t) * np.exp(-6 * t)).astype(
            np.float32
        )
        return np.stack([mono, mono])

    def test_signature_is_level_invariant_not_frequency_invariant(self):
        wave = self.tone(700)
        first = separation.signature(wave)
        np.testing.assert_allclose(
            first, separation.signature(wave * 0.25), atol=1e-8, rtol=1e-6
        )
        self.assertGreater(
            separation.shape_distance(first, separation.signature(self.tone(1400))), 1.5
        )
        self.assertAlmostEqual(first.sum(), 1)

    def test_silence_and_nonfinite_are_not_normalized_into_identity(self):
        for bad in (np.zeros((2, 131072)), np.full((2, 131072), np.nan)):
            with self.assertRaises(ValueError):
                separation.signature(bad)

    def test_retrieval_control_and_wrong_assignment(self):
        values = {
            i: separation.signature(self.tone(f))
            for i, f in enumerate((700, 1400, 2800))
        }
        self.assertEqual(separation.retrieval(values, values, list(values))["top1"], 3)
        wrong = {i: values[(i + 1) % 3] for i in values}
        self.assertEqual(separation.retrieval(values, wrong, list(values))["top1"], 0)

    def test_ties_do_not_produce_false_wins_and_roster_must_match(self):
        values = {i: np.array([1.0, 0.0]) for i in range(3)}
        self.assertEqual(separation.retrieval(values, values, list(values))["top1"], 0)
        with self.assertRaises(ValueError):
            separation.retrieval(values, {0: values[0]}, list(values))


if __name__ == "__main__":
    unittest.main()
