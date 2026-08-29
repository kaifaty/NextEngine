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
from nextengine_speech_timeline.protocol import (
    ASR_AUDIO_ROUTE_GAIN_ONLY,
    ASR_AUDIO_ROUTE_WHISPER,
)


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

    def test_calibrated_noise_floor_prevents_background_gain_activation(self) -> None:
        gain = SpeechAwareGain(SpeechAwareGainConfig(enabled=True))
        background = np.full(640, 10.0 ** (-72.0 / 20.0), dtype=np.float32)
        gain.reset(activation_threshold_dbfs=-70.0)
        output = np.concatenate((gain.process(background), gain.flush()))
        self.assertAlmostEqual(dbfs(output), dbfs(background), places=3)

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

    def test_whisper_route_keeps_a_dry_safety_floor_when_denoiser_erases_voice(self) -> None:
        class ErasingEnhancer:
            def __init__(self, **_: object) -> None:
                pass

            def process(self, samples: np.ndarray, *, sample_rate: int) -> np.ndarray:
                return np.zeros(len(samples), dtype=np.float32)

            def flush(self) -> np.ndarray:
                return np.zeros(0, dtype=np.float32)

            def reset(self) -> None:
                pass

        source = np.full(777, 0.05, dtype=np.float32)
        with patch.dict("sys.modules", {"dpdfnet": SimpleNamespace(StreamEnhancer=ErasingEnhancer)}):
            preprocessor = DpdfNetAudioPreprocessor(
                model_id="test/dpdfnet",
                model_revision="test",
                model_name="dpdfnet2",
                onnx_path=Path("/tmp/test-dpdfnet.onnx"),
                gain_config=SpeechAwareGainConfig(enabled=False),
                whisper_attenuation_limit_db=12.0,
            )
            preprocessor.load()
            preprocessor.warmup()
            preprocessor.reset(ASR_AUDIO_ROUTE_WHISPER)
            whisper = pcm16le_to_float32(
                preprocessor.process_pcm(float32_to_pcm16le(source)) + preprocessor.flush()
            )
            preprocessor.reset(ASR_AUDIO_ROUTE_GAIN_ONLY)
            gain_only = pcm16le_to_float32(
                preprocessor.process_pcm(float32_to_pcm16le(source)) + preprocessor.flush()
            )

        self.assertEqual(len(whisper), len(source))
        self.assertGreater(dbfs(whisper), -40.0)
        self.assertTrue(np.allclose(gain_only, source, atol=1.0 / 32768.0))

    def test_whisper_route_uses_pre_gain_even_when_enhanced_profile_is_post_only(self) -> None:
        class IdentityEnhancer:
            def __init__(self, **_: object) -> None:
                pass

            def process(self, samples: np.ndarray, *, sample_rate: int) -> np.ndarray:
                return np.asarray(samples, dtype=np.float32)

            def flush(self) -> np.ndarray:
                return np.zeros(0, dtype=np.float32)

            def reset(self) -> None:
                pass

        source = np.full(640, 10.0 ** (-58.0 / 20.0), dtype=np.float32)
        with patch.dict("sys.modules", {"dpdfnet": SimpleNamespace(StreamEnhancer=IdentityEnhancer)}):
            preprocessor = DpdfNetAudioPreprocessor(
                model_id="test/dpdfnet",
                model_revision="test",
                model_name="dpdfnet2",
                onnx_path=Path("/tmp/test-dpdfnet.onnx"),
                gain_config=SpeechAwareGainConfig(enabled=True),
                gain_placement=GAIN_PLACEMENT_POST_DENOISE,
            )
            preprocessor.load()
            preprocessor.warmup()
            preprocessor.reset(ASR_AUDIO_ROUTE_WHISPER)
            output = pcm16le_to_float32(
                preprocessor.process_pcm(float32_to_pcm16le(source)) + preprocessor.flush()
            )

        self.assertGreater(dbfs(output), dbfs(source) + 10.0)


if __name__ == "__main__":
    unittest.main()
