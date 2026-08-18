from __future__ import annotations

import unittest

import numpy as np

from nextengine_speech_timeline.adapters.base import AdapterError, AudioWindow
from nextengine_speech_timeline.adapters.emotion2vec import (
    Emotion2VecAffectAdapter,
    NORMALIZED_LABELS,
    UPSTREAM_LABELS,
)


class FakeProbe:
    model_id = "emotion/model"
    model_revision = "exact-revision"

    def __init__(self, labels: list[str] | None = None) -> None:
        self.labels = labels or list(UPSTREAM_LABELS)
        self.calls = 0

    def load(self) -> None:
        return None

    def analyze_waveform(self, samples: np.ndarray, sample_rate_hz: int) -> dict[str, object]:
        self.calls += 1
        return {
            "predictions": [
                {"label": label, "score": 0.9 if index == 0 else 0.01}
                for index, label in enumerate(self.labels)
            ],
            "inference_elapsed_ms": 12,
        }


def window() -> AudioWindow:
    return AudioWindow(
        samples=np.zeros(16_000, dtype=np.float32),
        sample_rate_hz=16_000,
        start_sample=8_000,
        end_sample=24_000,
        source_revision=7,
    )


class EmotionAdapterTests(unittest.TestCase):
    def test_upstream_labels_become_one_complete_neutral_vector(self) -> None:
        probe = FakeProbe()
        observation = Emotion2VecAffectAdapter(probe).observe(window())
        self.assertEqual(set(observation.scores), NORMALIZED_LABELS)
        self.assertEqual(observation.top_label, "angry")
        self.assertEqual((observation.start_sample, observation.end_sample), (8_000, 24_000))
        self.assertEqual(observation.source_revision, 7)
        self.assertEqual(probe.calls, 1)

    def test_unknown_label_fails_closed(self) -> None:
        labels = list(UPSTREAM_LABELS)
        labels[-1] = "invented"
        with self.assertRaises(AdapterError):
            Emotion2VecAffectAdapter(FakeProbe(labels)).observe(window())

    def test_duplicate_normalized_label_fails_closed(self) -> None:
        labels = list(UPSTREAM_LABELS)
        labels[-1] = labels[0]
        with self.assertRaises(AdapterError):
            Emotion2VecAffectAdapter(FakeProbe(labels)).observe(window())

    def test_interval_must_match_waveform(self) -> None:
        invalid = AudioWindow(
            samples=np.zeros(10, dtype=np.float32),
            sample_rate_hz=16_000,
            start_sample=0,
            end_sample=11,
            source_revision=1,
        )
        with self.assertRaises(AdapterError):
            Emotion2VecAffectAdapter(FakeProbe()).observe(invalid)


if __name__ == "__main__":
    unittest.main()
