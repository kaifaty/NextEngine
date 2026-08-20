from __future__ import annotations

import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np

from nextengine_speech_timeline.adapters.dpdfnet import (
    DpdfNetAudioPreprocessor,
    GAIN_PLACEMENT_POST_DENOISE,
    GAIN_PLACEMENT_PRE_AND_POST_DENOISE,
    SpeechAwareGain,
    SpeechAwareGainConfig,
)
from nextengine_speech_timeline.audio import float32_to_pcm16le, pcm16le_to_float32


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

    def test_pre_and_post_mode_preserves_quiet_voice_before_an_aggressive_denoiser(self) -> None:
        class AttenuatingEnhancer:
            def __init__(self, **_: object) -> None:
                pass

            def process(self, samples: np.ndarray, *, sample_rate: int) -> np.ndarray:
                if sample_rate != 16_000:
                    raise AssertionError(f"unexpected sample rate: {sample_rate}")
                return np.asarray(samples, dtype=np.float32) * 0.1

            def flush(self) -> np.ndarray:
                return np.zeros(0, dtype=np.float32)

            def reset(self) -> None:
                pass

        config = SpeechAwareGainConfig(enabled=True)
        source = (
            10.0 ** (-58.0 / 20.0)
            * np.sin(2.0 * np.pi * 240.0 * np.arange(16_123) / 16_000)
        ).astype(np.float32)

        def render(placement: str) -> np.ndarray:
            with patch.dict("sys.modules", {"dpdfnet": SimpleNamespace(StreamEnhancer=AttenuatingEnhancer)}):
                preprocessor = DpdfNetAudioPreprocessor(
                    model_id="test/dpdfnet",
                    model_revision="test",
                    model_name="dpdfnet2",
                    onnx_path=Path("/tmp/test-dpdfnet.onnx"),
                    gain_config=config,
                    gain_placement=placement,
                )
                preprocessor.load()
                preprocessor.warmup()
                preprocessor.reset()
                output = preprocessor.process_pcm(float32_to_pcm16le(source)) + preprocessor.flush()
            return pcm16le_to_float32(output)

        post_only = render(GAIN_PLACEMENT_POST_DENOISE)
        pre_and_post = render(GAIN_PLACEMENT_PRE_AND_POST_DENOISE)

        self.assertEqual(len(post_only), len(source))
        self.assertEqual(len(pre_and_post), len(source))
        self.assertGreater(dbfs(pre_and_post), dbfs(post_only) + 18.0)


if __name__ == "__main__":
    unittest.main()
