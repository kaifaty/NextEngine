from __future__ import annotations

import unittest

import numpy as np

from nextengine_speech_timeline.adapters.dpdfnet import SpeechAwareGain, SpeechAwareGainConfig


def dbfs(samples: np.ndarray) -> float:
    rms = float(np.sqrt(np.mean(samples * samples)))
    return 20.0 * np.log10(max(rms, 1e-8))


class SpeechAwareGainTests(unittest.TestCase):
    def test_quiet_active_signal_is_raised_and_limited(self) -> None:
        config = SpeechAwareGainConfig(enabled=True)
        gain = SpeechAwareGain(config)
        amplitude = 10.0 ** (-58.0 / 20.0)
        input_samples = amplitude * np.sin(2.0 * np.pi * 240.0 * np.arange(16_000) / 16_000)
        output = np.concatenate((gain.process(input_samples.astype(np.float32)), gain.flush()))

        self.assertEqual(len(output), len(input_samples))
        self.assertGreater(dbfs(output), dbfs(input_samples) + 16.0)
        self.assertLessEqual(float(np.max(np.abs(output))), 10.0 ** (-1.0 / 20.0) + 1e-6)

    def test_silence_is_not_raised_when_the_gate_is_inactive(self) -> None:
        gain = SpeechAwareGain(SpeechAwareGainConfig(enabled=True))
        silence = np.zeros(1_280, dtype=np.float32)
        output = np.concatenate((gain.process(silence), gain.flush()))
        self.assertTrue(np.array_equal(output, silence))

    def test_disabled_gain_is_clock_preserving_passthrough(self) -> None:
        gain = SpeechAwareGain(SpeechAwareGainConfig(enabled=False))
        source = np.linspace(-0.2, 0.2, 777, dtype=np.float32)
        output = np.concatenate((gain.process(source), gain.flush()))
        self.assertTrue(np.array_equal(output, source))


if __name__ == "__main__":
    unittest.main()
