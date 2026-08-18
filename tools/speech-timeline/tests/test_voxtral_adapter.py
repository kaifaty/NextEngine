from __future__ import annotations

from contextlib import contextmanager
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest

from nextengine_speech_timeline.adapters.base import AdapterError
from nextengine_speech_timeline.adapters.voxtral_transcribe_cpp import (
    TranscriberConfig,
    VoxtralTranscriberAdapter,
)


class FakeStream:
    def __init__(self, owner: FakeModule) -> None:
        self.owner = owner
        self.finalize_count = 0
        self.finalized = False

    def feed(self, samples: object) -> SimpleNamespace:
        self.owner.feed_count += 1
        return SimpleNamespace(committed_changed=True, tentative_changed=True)

    def text(self) -> SimpleNamespace:
        if self.finalized:
            return SimpleNamespace(
                committed="final display",
                tentative="",
                full="final authoritative",
            )
        return SimpleNamespace(
            committed="stable ",
            tentative="display",
            full="authoritative hypothesis",
        )

    def finalize(self) -> None:
        self.finalize_count += 1
        self.finalized = True


class FakeSession:
    def __init__(self, owner: FakeModule) -> None:
        self.owner = owner

    @contextmanager
    def stream(self, *, language: str | None, family: object):
        self.owner.languages.append(language)
        stream = FakeStream(self.owner)
        self.owner.streams.append(stream)
        yield stream


class FakeModel:
    arch = "voxtral"
    variant = "realtime"
    backend = "fake-cuda"
    capabilities = SimpleNamespace(supports_streaming=True)

    def __init__(self, owner: FakeModule) -> None:
        self.owner = owner

    def __enter__(self) -> FakeModel:
        self.owner.model_enter_count += 1
        return self

    def __exit__(self, *_: object) -> None:
        self.owner.model_exit_count += 1

    @contextmanager
    def session(self):
        self.owner.session_count += 1
        yield FakeSession(self.owner)


class FakeModule:
    def __init__(self) -> None:
        self.model_construct_count = 0
        self.model_enter_count = 0
        self.model_exit_count = 0
        self.session_count = 0
        self.feed_count = 0
        self.languages: list[str | None] = []
        self.streams: list[FakeStream] = []

    def Model(self, path: Path, *, backend: str) -> FakeModel:
        self.model_construct_count += 1
        return FakeModel(self)

    @staticmethod
    def VoxtralRealtimeStreamOptions(*, num_delay_tokens: int) -> SimpleNamespace:
        return SimpleNamespace(num_delay_tokens=num_delay_tokens)


class VoxtralAdapterTests(unittest.TestCase):
    def test_two_sessions_share_one_model_and_finalize_once_each(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            model_path = Path(temp_dir) / "model.gguf"
            model_path.write_bytes(b"fake")
            module = FakeModule()
            adapter = VoxtralTranscriberAdapter(
                model_path, None, None, backend="cuda", delay_ms=480, module=module
            )
            load = adapter.load()
            self.assertEqual(load.load_count, 1)
            for _ in range(2):
                with adapter.start(TranscriberConfig(language="ru")) as session:
                    revision = session.push_pcm([0.0, 0.1])
                    assert revision is not None
                    self.assertEqual(revision.full_text, "authoritative hypothesis")
                    self.assertEqual(
                        revision.committed_text + revision.tentative_text,
                        "stable display",
                    )
                    final = session.finish()
                    self.assertIs(session.finish(), final)
                    self.assertTrue(final.final)
                    self.assertEqual(final.full_text, "final authoritative")
            self.assertEqual(adapter.load_count, 1)
            self.assertEqual(module.model_construct_count, 1)
            self.assertEqual(module.model_enter_count, 1)
            self.assertEqual(module.session_count, 2)
            self.assertEqual([stream.finalize_count for stream in module.streams], [1, 1])
            adapter.close()
            self.assertEqual(module.model_exit_count, 1)

    def test_cancel_does_not_finalize(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            model_path = Path(temp_dir) / "model.gguf"
            model_path.write_bytes(b"fake")
            module = FakeModule()
            adapter = VoxtralTranscriberAdapter(model_path, None, None, module=module)
            session = adapter.start()
            session.cancel()
            self.assertEqual(module.streams[0].finalize_count, 0)
            with self.assertRaises(AdapterError):
                session.finish()
            adapter.close()


if __name__ == "__main__":
    unittest.main()
