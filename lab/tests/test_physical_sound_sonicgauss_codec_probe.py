import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_codec_probe as codec


class CodecProbeTests(unittest.TestCase):
    def test_audible_diagnostic_detects_ring_erasure_despite_low_frequency_energy(self):
        t = np.arange(131418) / 44100
        low = np.stack([0.1 * np.sin(2 * np.pi * 3 * t)] * 2).astype(np.float32)
        reference = low + codec.controls()["quiet-ring"]
        raw = codec.metric(reference, low)
        audible = codec.metric(
            codec.audible_component(reference), codec.audible_component(low)
        )
        self.assertLess(raw, 0.02)
        self.assertGreater(audible, 0.8)
        self.assertGreater(audible, raw * 50)
        self.assertEqual(
            codec.metric(
                codec.audible_component(reference), codec.audible_component(reference)
            ),
            0,
        )

    def test_controls_only_level_differs_for_ring(self):
        controls = codec.controls()
        self.assertEqual(len(controls), 4)
        for value in controls.values():
            self.assertEqual(value.shape, (2, 131418))
            self.assertTrue(np.isfinite(value).all())
        np.testing.assert_array_equal(controls["quiet-ring"], controls["ring"] * 0.1)
        self.assertAlmostEqual(
            codec.signal_stats(controls["ring"])["audible_peak_hz"], 700, delta=1
        )

    def test_full_tail_and_silence_controls(self):
        reference = codec.controls()["ring"]
        self.assertEqual(codec.metric(reference, reference.copy()), 0)
        altered = reference.copy()
        altered[:, 44100:] += 0.01
        self.assertGreater(codec.metric(reference, altered), 0)
        self.assertGreater(
            codec.signal_stats(altered)["late_energy_fraction_after_100ms"],
            codec.signal_stats(reference)["late_energy_fraction_after_100ms"],
        )
        self.assertIsNone(codec.metric(np.zeros_like(reference), reference))
        self.assertIsNone(
            codec.signal_stats(np.zeros_like(reference))["audible_peak_hz"]
        )

    def test_invalid_signal_rejects(self):
        for wave in [
            np.ones(10),
            np.ones((1, 10)),
            np.zeros((2, 0)),
            np.full((2, 10), np.nan),
        ]:
            with self.assertRaisesRegex(ValueError, "stereo"):
                codec.signal_stats(wave)


if __name__ == "__main__":
    unittest.main()
