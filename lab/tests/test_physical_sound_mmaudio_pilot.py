import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import soundfile as sf

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_mmaudio_pilot as pilot


class PilotTests(unittest.TestCase):
    def test_input_rejects_audio_even_if_silent(self):
        for streams in (
            [],
            [{"codec_type": "audio"}],
            [{"codec_type": "video"}, {"codec_type": "audio"}],
        ):
            with (
                patch.object(pilot, "probe", return_value={"streams": streams}),
                self.assertRaises(ValueError),
            ):
                pilot.require_silent_video(Path("source.mp4"))
        with patch.object(
            pilot, "probe", return_value={"streams": [{"codec_type": "video"}]}
        ):
            pilot.require_silent_video(Path("silent.mp4"))

    def test_publish_preserves_float_and_pcm_without_gain(self):
        wave = (0.1 * np.sin(np.arange(8 * 44100) * 0.1)).astype(np.float32)
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "example.wav"
            item = pilot.publish(path, wave)
            decoded, rate = sf.read(path, dtype="float32")
            full, _ = sf.read(path.with_stem("example-float"), dtype="float32")
            np.testing.assert_array_equal(full, wave)
            self.assertLessEqual(float(np.max(np.abs(decoded - wave))), 1 / 32768)
            self.assertEqual(rate, 44100)
            self.assertEqual(item["sha256"], pilot.digest(path))

    def test_publish_rejects_bad_length_layout_nonfinite_and_headroom(self):
        valid = np.zeros(8 * 44100, dtype=np.float32)
        bad_nan = valid.copy()
        bad_nan[0] = np.nan
        for wave in (valid[:100], valid[None], bad_nan, valid + 0.99):
            with tempfile.TemporaryDirectory() as temp:
                with self.assertRaises(ValueError):
                    pilot.publish(Path(temp) / "bad.wav", wave)
                self.assertEqual(list(Path(temp).iterdir()), [])

    def test_asset_scope_excludes_training_checkpoints(self):
        for _, revision, names in pilot.ASSETS.values():
            self.assertEqual(len(revision), 40)
            self.assertFalse(any(n.startswith("checkpoints/") for n in names))
            self.assertFalse(any("optimizer" in n for n in names))

    def test_explicit_shared_attenuation_preserves_raw(self):
        raw = np.full(8 * 44100, 0.999, dtype=np.float32)
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "hot.wav"
            with self.assertRaises(ValueError):
                pilot.publish(path, raw)
            item = pilot.publish(path, raw, playback_gain=0.5)
            full, _ = sf.read(path.with_stem("hot-float"), dtype="float32")
            pcm, _ = sf.read(path, dtype="float32")
            np.testing.assert_array_equal(full, raw)
            self.assertLess(float(np.abs(pcm).max()), 0.5)
            self.assertEqual(item["playback_gain"], 0.5)


if __name__ == "__main__":
    unittest.main()
