from __future__ import annotations

from pathlib import Path
import unittest

import numpy as np

from nextengine_speech_timeline.adapters.base import AdapterError, TranscriberConfig
from nextengine_speech_timeline.adapters.gigaam import (
    MAX_AUDIO_SAMPLES,
    GigaAmTranscriberAdapter,
)


class FakeBackend:
    device = "cuda"

    def __init__(self) -> None:
        self.inputs: list[np.ndarray] = []
        self.closed = False

    def transcribe(self, samples: np.ndarray) -> str:
        self.inputs.append(samples.copy())
        return "шёпот распознан"

    def close(self) -> None:
        self.closed = True


class GigaAmAdapterTests(unittest.TestCase):
    def test_resident_final_only_session_accumulates_one_bounded_utterance(self) -> None:
        backend = FakeBackend()
        loads: list[tuple[Path, str]] = []

        def loader(snapshot: Path, device: str) -> FakeBackend:
            loads.append((snapshot, device))
            return backend

        adapter = GigaAmTranscriberAdapter(
            Path("/tmp/gigaam-snapshot"),
            model_id="ai-sage/GigaAM-v3",
            model_revision="a" * 40,
            device="cuda",
            loader=loader,
        )
        adapter.load()
        adapter.warmup()
        capabilities = adapter.capabilities()
        self.assertFalse(capabilities.supports_streaming)
        self.assertEqual(capabilities.max_audio_duration_ms, 25_000)
        self.assertEqual(adapter.load_count, 1)
        self.assertEqual(len(loads), 1)

        session = adapter.start(TranscriberConfig(language="ru"))
        self.assertIsNone(session.push_pcm(np.array([0.1, 0.2], dtype=np.float32)))
        self.assertIsNone(session.push_pcm(np.array([0.3], dtype=np.float32)))
        final = session.finish()
        self.assertTrue(final.final)
        self.assertEqual(final.full_text, "шёпот распознан")
        np.testing.assert_allclose(backend.inputs[0], [0.1, 0.2, 0.3])
        session.close()
        adapter.close()
        self.assertTrue(backend.closed)

    def test_language_and_25_second_bound_fail_closed(self) -> None:
        backend = FakeBackend()
        adapter = GigaAmTranscriberAdapter(
            Path("/tmp/gigaam-snapshot"),
            model_id="ai-sage/GigaAM-v3",
            model_revision="a" * 40,
            loader=lambda _snapshot, _device: backend,
        )
        adapter.load()
        with self.assertRaisesRegex(AdapterError, "Russian"):
            adapter.start(TranscriberConfig(language="en"))
        session = adapter.start(TranscriberConfig(language="ru-RU"))
        with self.assertRaisesRegex(AdapterError, "25 second"):
            session.push_pcm(np.zeros(MAX_AUDIO_SAMPLES + 1, dtype=np.float32))
        session.cancel()
        adapter.close()


if __name__ == "__main__":
    unittest.main()
