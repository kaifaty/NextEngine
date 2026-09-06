import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_mmaudio_timing as timing


class TimingTests(unittest.TestCase):
    def test_one_to_one_and_empty_prediction(self):
        result = timing.match([1, 1.1], [1.05])
        self.assertEqual(result["matched"], 1)
        self.assertEqual(result["missed"], 1)
        self.assertEqual(timing.match([1], [2])["matched"], 0)
        self.assertEqual(timing.match([1], [])["recall"], 0)
        self.assertIsNone(timing.match([], [])["recall"])

    def test_known_impulses_delay_and_gain(self):
        rate = 16000
        wave = np.zeros(8 * rate)
        times = np.array([1, 2.3, 4.1, 5.7])
        for t in times:
            wave[int(t * rate) : int(t * rate) + 50] = 0.1
        detected = timing.onsets(wave, rate)
        np.testing.assert_allclose(detected, times, atol=0.01)
        np.testing.assert_array_equal(detected, timing.onsets(wave * 0.5, rate))
        delayed = np.r_[np.zeros(rate), wave[:-rate]]
        np.testing.assert_allclose(
            timing.onsets(delayed, rate), detected + 1, atol=0.01
        )
        self.assertEqual(
            timing.match(times + 1, timing.onsets(delayed, rate))["matched"], 4
        )
        self.assertEqual(
            timing.match(times, timing.onsets(delayed, rate))["matched"], 0
        )
        self.assertEqual(len(timing.onsets(np.zeros(rate), rate)), 0)


if __name__ == "__main__":
    unittest.main()
