from __future__ import annotations

from pathlib import Path
import tempfile
import unittest

import numpy as np

from nextengine_speech_timeline.adapters.base import AdapterError, TranscriberConfig
from nextengine_speech_timeline.adapters.nemotron_nemo_speech_cpp import (
    NemotronTranscriberAdapter,
    _NativeUpdate,
)


class FakeStream:
    def __init__(self) -> None:
        self.pushes: list[np.ndarray] = []
        self.closed = False

    def push(self, samples: np.ndarray) -> _NativeUpdate | None:
        self.pushes.append(samples.copy())
        if len(self.pushes) == 1:
            return None
        return _NativeUpdate("шёпот распоз", False)

    def finish(self) -> _NativeUpdate:
        return _NativeUpdate("шёпот распознан", True)

    def close(self) -> None:
        self.closed = True


class FakeBackend:
    device = "cuda:0"
    runtime_version = "0.6.0"

    def __init__(self) -> None:
        self.streams: list[FakeStream] = []
        self.languages: list[str | None] = []
        self.closed = False

    def start(self, language: str | None) -> FakeStream:
        self.languages.append(language)
        stream = FakeStream()
        self.streams.append(stream)
        return stream

    def close(self) -> None:
        self.closed = True


class NemotronAdapterTests(unittest.TestCase):
    def test_resident_cache_aware_stream_emits_partial_and_final_revisions(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            paths = [root / "model.gguf", root / "libasr.so", root / "libasr_c.so"]
            for path in paths:
                path.write_bytes(b"pinned")
            backend = FakeBackend()
            loads: list[tuple[Path, Path, Path, int, int]] = []

            def loader(
                implementation: Path,
                abi: Path,
                model: Path,
                gpu: int,
                right_context: int,
            ) -> FakeBackend:
                loads.append((implementation, abi, model, gpu, right_context))
                return backend

            adapter = NemotronTranscriberAdapter(
                paths[0],
                paths[1],
                paths[2],
                model_id="nvidia/nemotron-3.5-asr-streaming-0.6b",
                model_revision="a" * 40,
                gpu=0,
                right_context=1,
                loader=loader,
            )
            adapter.load()
            adapter.warmup()
            capabilities = adapter.capabilities()
            self.assertTrue(capabilities.supports_streaming)
            self.assertEqual(capabilities.configured_delay_ms, 160)
            self.assertEqual(capabilities.partial_decode_interval_ms, 80)
            self.assertEqual(adapter.load_count, 1)
            self.assertEqual(len(loads), 1)

            session = adapter.start(TranscriberConfig(language="ru-RU"))
            self.assertIsNone(session.push_pcm(np.array([0.1, 0.2], dtype=np.float32)))
            partial = session.push_pcm(np.array([0.3], dtype=np.float32))
            assert partial is not None
            self.assertFalse(partial.final)
            self.assertEqual(partial.tentative_text, "шёпот распоз")
            self.assertEqual(partial.committed_text, "")
            final = session.finish()
            self.assertIs(session.finish(), final)
            self.assertTrue(final.final)
            self.assertEqual(final.committed_text, "шёпот распознан")
            session.close()
            adapter.close()
            self.assertEqual(backend.languages, ["ru", "ru"])
            self.assertTrue(all(stream.closed for stream in backend.streams))
            self.assertTrue(backend.closed)

    def test_invalid_language_and_right_context_fail_closed(self) -> None:
        with self.assertRaisesRegex(AdapterError, "right context"):
            NemotronTranscriberAdapter(
                Path("/tmp/model"),
                Path("/tmp/implementation"),
                Path("/tmp/abi"),
                model_id="model",
                model_revision="revision",
                right_context=3,
            )

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            paths = [root / "model", root / "implementation", root / "abi"]
            for path in paths:
                path.write_bytes(b"pinned")
            backend = FakeBackend()
            adapter = NemotronTranscriberAdapter(
                paths[0],
                paths[1],
                paths[2],
                model_id="model",
                model_revision="revision",
                loader=lambda *_args: backend,
            )
            adapter.load()
            with self.assertRaisesRegex(AdapterError, "Russian"):
                adapter.start(TranscriberConfig(language="en"))
            adapter.close()


if __name__ == "__main__":
    unittest.main()
