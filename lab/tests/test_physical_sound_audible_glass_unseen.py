"""Unseen-window selection and unchanged preprocessing checks."""

import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_audible_glass as glass
import physical_sound_audible_glass_unseen as unseen


class UnseenGlassTest(unittest.TestCase):
    def test_no_training_or_evaluation_overlap(self):
        for sound_id, first_peak, _ in glass.SOURCES:
            starts = [
                round((peak - 0.05) * glass.RATE) for peak in unseen.PEAKS[sound_id]
            ]
            unseen.check_disjoint(starts, round((first_peak - 0.05) * glass.RATE))
        with self.assertRaises(ValueError):
            unseen.check_disjoint([glass.SAMPLES - 1], 0)
        with self.assertRaises(ValueError):
            unseen.check_disjoint([glass.SAMPLES, glass.SAMPLES + 1], 0)
        unseen.check_disjoint([glass.SAMPLES], 0)

    def test_crop_preserves_training_preprocessing(self):
        rng = np.random.default_rng(42)
        decoded = rng.normal(size=glass.RATE * 3).astype(np.float32)
        peak = 0.58
        actual, onset, metadata = unseen.crop(decoded, peak)
        start = round((peak - 0.05) * glass.RATE)
        expected = decoded[start : start + glass.SAMPLES].copy()
        expected -= expected.mean()
        expected *= 0.8 / float(np.max(np.abs(expected)))
        rms = np.sqrt(np.convolve(expected[:3200] ** 2, np.ones(32) / 32, mode="same"))
        self.assertTrue(torch.equal(actual, torch.tensor(expected)))
        self.assertEqual(onset, int(np.flatnonzero(rms > rms.max() * 0.1)[0]))
        self.assertEqual(metadata["crop_start_sample"], start)

    def test_invalid_windows_fail(self):
        for audio, peak in (
            (np.zeros(100), 1),
            (np.ones(100000), 0.5),
            (np.ones(100000), -1),
            (np.full(100000, np.nan), 0.5),
        ):
            with self.assertRaises(ValueError):
                unseen.crop(audio, peak)


if __name__ == "__main__":
    unittest.main()
