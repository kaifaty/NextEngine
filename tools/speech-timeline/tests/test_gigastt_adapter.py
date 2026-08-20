from __future__ import annotations

import json
from pathlib import Path
import queue
import threading
import time
import unittest

import numpy as np

from nextengine_speech_timeline.adapters.base import AdapterError, TranscriberConfig
from nextengine_speech_timeline.adapters.gigastt import GigasttTranscriberAdapter


class _Closed(Exception):
    pass


class _FakeConnection:
    def __init__(self) -> None:
        self.messages: queue.Queue[str | _Closed] = queue.Queue()
        self.messages.put(
            json.dumps(
                {
                    "type": "ready",
                    "model": "gigaam-v3-rnnt",
                    "sample_rate": 48_000,
                    "supported_rates": [8_000, 16_000, 48_000],
                    "version": "1.0",
                }
            )
        )
        self.sent: list[str | bytes] = []
        self.closed = False
        self.binary_count = 0

    def recv(self, timeout: float | None = None) -> str:
        try:
            message = self.messages.get(timeout=timeout)
        except queue.Empty as error:
            raise TimeoutError from error
        if isinstance(message, _Closed):
            raise message
        return message

    def send(self, message: str | bytes) -> None:
        self.sent.append(message)
        if isinstance(message, bytes):
            self.binary_count += 1
            self.messages.put(
                json.dumps(
                    {
                        "type": "partial",
                        "text": "тихая речь" if self.binary_count == 1 else "тихая речь слышна",
                    }
                )
            )
        elif json.loads(message).get("type") == "stop":
            self.messages.put(json.dumps({"type": "final", "text": "тихая речь слышна"}))

    def close(self) -> None:
        if self.closed:
            return
        self.closed = True
        self.messages.put(_Closed())


class _FakeSidecar:
    backend = "cpu"

    def __init__(self) -> None:
        self.connection = _FakeConnection()
        self.started = False
        self.closed = False

    def start(self) -> None:
        self.started = True

    def connect(self) -> _FakeConnection:
        return self.connection

    def close(self) -> None:
        self.closed = True


class GigasttAdapterTests(unittest.TestCase):
    def test_buffered_partial_and_final_are_exposed_without_false_stability(self) -> None:
        sidecar = _FakeSidecar()
        adapter = GigasttTranscriberAdapter(
            Path("/external/gigastt"),
            Path("/external/models"),
            model_id="GigaAM-v3-rnnt-int8",
            model_revision="release-2.18.0",
            runtime_version="2.18.0",
            launcher=lambda _runtime, _models: sidecar,
        )
        evidence = adapter.load()
        self.assertTrue(sidecar.started)
        self.assertEqual(evidence.load_count, 1)
        capabilities = adapter.capabilities()
        self.assertTrue(capabilities.supports_streaming)
        self.assertEqual(capabilities.streaming_mode, "buffered_emulation")
        self.assertEqual(capabilities.partial_decode_interval_ms, 800)
        self.assertEqual(capabilities.streaming_window_ms, 2_500)
        self.assertEqual(capabilities.streaming_left_context_ms, 1_500)
        self.assertEqual(capabilities.runtime_id, "gigastt/2.18.0")

        session = adapter.start(TranscriberConfig(language="ru"))
        configure = json.loads(str(sidecar.connection.sent[0]))
        self.assertEqual(configure["endpoint_mode"], "manual")
        self.assertFalse(configure["punctuation"])

        first = session.push_pcm(np.array([-1.0, 0.0, 1.0], dtype=np.float32))
        self.assertIsNone(first)
        self.assertEqual(sidecar.connection.sent[1], b"\x00\x80\x00\x00\xff\x7f")
        self._wait_for_reader(session)
        partial = session.push_pcm(np.zeros(160, dtype=np.float32))
        self.assertIsNotNone(partial)
        assert partial is not None
        self.assertEqual(partial.full_text, "тихая речь")
        self.assertEqual(partial.committed_text, "")
        self.assertEqual(partial.tentative_text, "тихая речь")
        self.assertFalse(partial.final)

        final = session.finish()
        self.assertEqual(final.full_text, "тихая речь слышна")
        self.assertEqual(final.committed_text, "тихая речь слышна")
        self.assertTrue(final.final)
        session.close()
        adapter.close()
        self.assertTrue(sidecar.closed)

    def test_non_russian_locale_is_rejected_without_leaking_active_session(self) -> None:
        sidecar = _FakeSidecar()
        adapter = GigasttTranscriberAdapter(
            Path("/external/gigastt"),
            Path("/external/models"),
            model_id="GigaAM-v3-rnnt-int8",
            model_revision="release-2.18.0",
            runtime_version="2.18.0",
            launcher=lambda _runtime, _models: sidecar,
        )
        adapter.load()
        with self.assertRaises(AdapterError):
            adapter.start(TranscriberConfig(language="en"))
        session = adapter.start(TranscriberConfig(language="ru-RU"))
        session.cancel()
        adapter.close()

    def _wait_for_reader(self, session: object) -> None:
        responses = getattr(session, "_responses")
        deadline = time.monotonic() + 1
        while responses.qsize() == 0 and time.monotonic() < deadline:
            threading.Event().wait(0.001)
        self.assertGreater(responses.qsize(), 0)


if __name__ == "__main__":
    unittest.main()
