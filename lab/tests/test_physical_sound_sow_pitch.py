import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sow_pitch as s


class SowPitchTests(unittest.TestCase):
    def test_bad_checkpoint_and_audio_fail_before_inference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / s.FILENAME).write_bytes(b"not a checkpoint")
            with self.assertRaises(ValueError):
                s.load(root, "cpu")
        for wave in (np.ones(3), np.full(500, np.nan), np.ones((1, 500))):
            with self.assertRaises(ValueError):
                s.infer(None, None, wave)
        with self.assertRaises(ValueError):
            s.infer(None, None, np.ones(500), start_seconds=-1)

    def test_time_encoding_matches_upstream_formula(self):
        timestamps = torch.linspace(0, 4.08, 203)[None]
        ticks = (timestamps * 49).to(int)
        phase = ticks[:, :, None].float() * torch.exp(
            torch.arange(0, 512, 2) * -(np.log(10000) / 512)
        )
        expected = torch.zeros(1, 203, 512)
        expected[:, :, 0::2] = torch.sin(phase)
        expected[:, :, 1::2] = torch.cos(phase)
        torch.testing.assert_close(
            s.time_encoding(203, 4.08, "cpu"), expected[0] * 0.01
        )
        # Nonzero clip starts are part of the upstream interface too.
        shifted = s.time_encoding(203, 4.08, "cpu", start_seconds=2)
        self.assertFalse(torch.equal(shifted, expected[0] * 0.01))
        self.assertAlmostEqual(float(shifted[0, 0]), np.sin(98) * 0.01, places=8)

    def test_block_shuffle_preserves_samples_and_length(self):
        wave = np.arange(4 * 16000 + 17, dtype=np.float32)
        shuffled = s.shuffle_blocks(wave)
        np.testing.assert_array_equal(np.sort(shuffled), wave)
        np.testing.assert_array_equal(shuffled[-17:], wave[-17:])
        self.assertFalse(np.array_equal(wave, shuffled))


if __name__ == "__main__":
    unittest.main()
