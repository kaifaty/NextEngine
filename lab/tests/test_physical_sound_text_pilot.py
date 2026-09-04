"""Pure signal/measurement checks; no model download needed."""

import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_text_pilot as pilot
import physical_sound_text_tags as tags


class TextPilotTest(unittest.TestCase):
    def test_unknown_ontology_axis_is_unscored(self):
        self.assertEqual(tags.EXPECTED["steel-metal"], ())
        result = tags.summarize(np.array([0.7, 0.2]), ["Glass", "Rain"], ())
        self.assertIsNone(result["expected_in_top5"])
        with self.assertRaises(ValueError):
            tags.summarize(np.array([0.7, 0.2]), ["Glass", "Rain"], ("Steel",))

    def test_waveform_hash_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "test.wav"
            pilot.write_audio(path, np.ones(1000, dtype=np.float32) * 0.1)
            with self.assertRaises(ValueError):
                tags.load_audio(path, "wrong")

    def test_bad_signal_rejected(self):
        for value in ([], [float("nan")], [float("inf")], [[1, 2]]):
            with self.assertRaises(ValueError):
                pilot.signal_stats(np.asarray(value))

    def test_pcm_no_amplification_and_no_clipping(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "test.wav"
            for amplitude in (0.1, 1.5):
                report = pilot.write_audio(
                    path, np.array([amplitude, -amplitude], dtype=np.float32)
                )
                rate, pcm = wavfile.read(path)
                self.assertEqual(rate, pilot.RATE)
                self.assertLessEqual(report["pcm_gain"], 1)
                self.assertLessEqual(np.abs(pcm).max(), 32113)
            with self.assertRaises(ValueError):
                pilot.write_audio(path, np.zeros(100))

    def test_alignment_detects_swapped_label(self):
        self.assertEqual(pilot.alignment(np.array([0.8, 0.2]), 0)["target_rank"], 1)
        self.assertLess(
            pilot.alignment(np.array([0.8, 0.2]), 1)["target_minus_best_other"], 0
        )
        with self.assertRaises(ValueError):
            pilot.alignment(np.array([0.8, np.nan]), 0)
        with self.assertRaises(ValueError):
            pilot.alignment(np.array([0.8, 0.2]), -1)

    def test_pair_contrast_and_no_effect_control(self):
        matrix = np.array([[0.8, 0.2], [0.3, 0.7]])
        self.assertAlmostEqual(pilot.pair_margin(matrix, 0, 1), 0.5)
        self.assertLess(pilot.pair_margin(matrix[::-1], 0, 1), 0)
        self.assertAlmostEqual(
            pilot.pair_margin(np.array([[0.8, 0.2], [0.8, 0.2]]), 0, 1), 0
        )


if __name__ == "__main__":
    unittest.main()
