import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_wave_codec as c


class CodecTests(unittest.TestCase):
    def test_padding_channels_and_roundtrip(self):
        wave = (0.01 * np.sin(np.arange(c.v.p.SAMPLES) * 0.1)).astype(np.float32)
        native = c.input_wave(wave)
        self.assertEqual(native.shape, (2, 88 * 2048))
        np.testing.assert_array_equal(native[0], native[1])
        self.assertEqual(c.mono_wave(native).shape, wave.shape)
        self.assertLess(
            float(np.sqrt(np.mean((c.mono_wave(native) - wave) ** 2))), 0.0001
        )

    def test_invalid_audio_rejected(self):
        for wave in (
            np.zeros(3),
            np.ones(c.v.p.SAMPLES),
            np.full(c.v.p.SAMPLES, np.nan),
        ):
            with self.assertRaises(ValueError):
                c.input_wave(wave)
        with self.assertRaises(ValueError):
            c.mono_wave(np.ones((2, 88 * 2048)))


if __name__ == "__main__":
    unittest.main()
