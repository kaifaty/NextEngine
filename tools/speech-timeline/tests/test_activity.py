from __future__ import annotations

import unittest

import numpy as np

from nextengine_speech_timeline.activity import (
    EnergyVoiceActivityDetector,
    VoiceActivityConfig,
)


def pcm(value: int, samples: int) -> bytes:
    return np.full(samples, value, dtype="<i2").tobytes()


class ActivityTests(unittest.TestCase):
    def test_silence_is_not_eligible_and_preserves_full_timeline(self) -> None:
        detector = EnergyVoiceActivityDetector()
        detector.feed_pcm16(0, pcm(0, 16_000))
        detector.flush()

        window = detector.window(0, 16_000)
        self.assertEqual(window.state, "no_speech")
        self.assertEqual(window.voiced_samples, 0)
        self.assertFalse(window.eligible_for_affect)
        self.assertEqual(
            [(item.start_sample, item.end_sample, item.state) for item in detector.timeline(16_000)],
            [(0, 16_000, "no_speech")],
        )

    def test_speech_requires_start_hysteresis_and_has_preroll(self) -> None:
        detector = EnergyVoiceActivityDetector(
            VoiceActivityConfig(end_frames=3, pre_roll_ms=100)
        )
        # 40 ms silence, then 800 ms speech, then enough silence to close.
        detector.feed_pcm16(0, pcm(0, 640))
        detector.feed_pcm16(640, pcm(12_000, 12_800))
        detector.feed_pcm16(13_440, pcm(0, 2_000))
        detector.flush()

        segments = detector.timeline(15_440)
        speech = [item for item in segments if item.state == "speech"]
        self.assertEqual(len(speech), 1)
        self.assertEqual(speech[0].start_sample, 0)
        self.assertGreaterEqual(speech[0].voiced_samples, 12_000)
        self.assertTrue(detector.latest_speech_span(15_440))

    def test_window_coverage_ignores_pause_and_uses_global_clock(self) -> None:
        detector = EnergyVoiceActivityDetector(
            VoiceActivityConfig(end_frames=10, min_voiced_ms=100, min_voiced_ratio=0.3)
        )
        detector.feed_pcm16(0, pcm(8_000, 8_000))
        detector.feed_pcm16(8_000, pcm(0, 8_000))
        detector.feed_pcm16(16_000, pcm(8_000, 8_000))

        window = detector.window(0, 24_000)
        self.assertEqual(window.voiced_samples, 16_000)
        self.assertAlmostEqual(window.voiced_ratio, 2 / 3, places=3)
        self.assertTrue(window.eligible_for_affect)

    def test_quiet_room_calibration_adapts_energy_gate_without_touching_model_audio(self) -> None:
        config = VoiceActivityConfig.calibrated(
            noise_floor_dbfs=-50.0,
            duration_ms=2_000,
        )
        detector = EnergyVoiceActivityDetector(config)

        self.assertEqual(config.speech_threshold_dbfs, -41.0)
        self.assertEqual(config.silence_threshold_dbfs, -46.0)
        capabilities = detector.capabilities()
        self.assertEqual(capabilities["calibration"]["mode"], "browser_quiet_noise_floor")
        self.assertEqual(capabilities["calibration"]["noise_floor_dbfs"], -50.0)


if __name__ == "__main__":
    unittest.main()
