from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace
import unittest

import numpy as np
import torch

from nextengine_speech_timeline.adapters.base import AdapterError, AudioWindow
from nextengine_speech_timeline.adapters.wavlm_russian_resd import (
    ADAPTER_ID,
    MAX_INPUT_SAMPLES,
    NORMALIZED_LABELS,
    UPSTREAM_LABELS,
    WavlmRussianResdAffectAdapter,
    _validate_config,
    normalize_probabilities,
)


class FakeFeatureExtractor:
    def __call__(self, samples: np.ndarray, **kwargs: object) -> dict[str, torch.Tensor]:
        self.samples = samples
        self.kwargs = kwargs
        return {"input_values": torch.from_numpy(samples[None, :])}


class FakeModel:
    def __call__(self, **kwargs: object) -> SimpleNamespace:
        self.kwargs = kwargs
        return SimpleNamespace(
            logits=torch.tensor([[-5.0, -4.0, 6.0, -3.0, -2.0, -1.0, -6.0]])
        )


def window(samples: int = 16_000) -> AudioWindow:
    return AudioWindow(
        samples=np.zeros(samples, dtype=np.float32),
        sample_rate_hz=16_000,
        start_sample=8_000,
        end_sample=8_000 + samples,
        source_revision=9,
    )


class WavlmRussianResdAdapterTests(unittest.TestCase):
    def test_probabilities_become_one_complete_neutral_vector(self) -> None:
        adapter = WavlmRussianResdAffectAdapter(
            "Aniemore/wavlm-emotion-russian-resd",
            "a" * 40,
            Path("/tmp"),
            "cpu",
            "sha256:" + "b" * 64,
        )
        adapter._device = "cpu"
        adapter._feature_extractor = FakeFeatureExtractor()
        adapter._model = FakeModel()

        observation = adapter.observe(window())

        self.assertEqual(set(observation.scores), NORMALIZED_LABELS)
        self.assertEqual(observation.top_label, "enthusiasm")
        self.assertEqual((observation.start_sample, observation.end_sample), (8_000, 24_000))
        self.assertEqual(observation.source_revision, 9)
        self.assertEqual(adapter.capabilities().adapter_id, ADAPTER_ID)

    def test_invalid_config_labels_fail_closed(self) -> None:
        config = SimpleNamespace(model_type="wavlm", id2label={0: "anger"})
        with self.assertRaises(AdapterError):
            _validate_config(config)

    def test_invalid_score_vector_fails_closed(self) -> None:
        with self.assertRaises(AdapterError):
            normalize_probabilities(np.ones(6, dtype=np.float64))
        with self.assertRaises(AdapterError):
            normalize_probabilities(np.array([np.nan] * len(UPSTREAM_LABELS)))

    def test_input_over_twelve_seconds_fails_closed(self) -> None:
        invalid = window(MAX_INPUT_SAMPLES + 1)
        with self.assertRaises(AdapterError):
            WavlmRussianResdAffectAdapter(
                "Aniemore/wavlm-emotion-russian-resd",
                "a" * 40,
                Path("/tmp"),
                "cpu",
                "sha256:" + "b" * 64,
            ).observe(invalid)


if __name__ == "__main__":
    unittest.main()
