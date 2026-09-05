from __future__ import annotations

import io
import sys
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_probe as probe


class TextureProbeTests(unittest.TestCase):
    def wav(self, data, rate=44100):
        buffer = io.BytesIO()
        sf.write(buffer, data, rate, format="WAV", subtype="FLOAT")
        return buffer.getvalue()

    def test_fixed_physical_grid(self):
        rows = probe.conditions()
        self.assertEqual(len(rows), 12)
        self.assertEqual(len({r["id"] for r in rows}), 12)
        self.assertEqual({r["texture_id"] for r in rows}, {0, 65, 74})
        self.assertEqual({r["commanded_normal_force_N"] for r in rows}, {0.5, 1})
        self.assertEqual({r["commanded_speed_mm_s"] for r in rows}, {20, 60})
        self.assertTrue(all(r["probe_material"] == "urethane rubber" for r in rows))

    def test_raw_channels_kept_separate(self):
        data = np.tile([0.25, 0.125], (4410, 1))
        result = probe.validate_audio(self.wav(data), 2)
        self.assertEqual(result["channels"], 2)
        self.assertEqual(result["rms"], [0.25, 0.125])
        self.assertEqual(result["seconds"], 0.1)

    def test_training_grid_has_repeats_and_middle_speed(self):
        rows = probe.conditions(True)
        self.assertEqual(len(rows), 60)
        self.assertEqual(len({row["id"] for row in rows}), 60)
        self.assertEqual({row["repeat"] for row in rows}, {0, 1})
        self.assertEqual(
            {row["commanded_speed_mm_s"] for row in rows}, {20, 30, 40, 50, 60}
        )

    def test_invalid_audio_rejected(self):
        for data, rate, channels in [
            (np.zeros(441), 16000, 1),
            (np.zeros((441, 2)), 44100, 1),
            (np.zeros(0), 44100, 1),
            (np.zeros(44100 * 15 + 1), 44100, 1),
            (np.full(441, np.nan), 44100, 1),
        ]:
            with (
                self.subTest(rate=rate, shape=data.shape),
                self.assertRaises(ValueError),
            ):
                probe.validate_audio(self.wav(data, rate), channels)

    def test_no_source_amplitude_invention(self):
        result = probe.validate_audio(self.wav(np.zeros(441)), 1)
        self.assertEqual(result["rms"], [0.0])
        # Silence is evidence, not a reason to amplify or omit a source.


if __name__ == "__main__":
    unittest.main()
