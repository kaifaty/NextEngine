from __future__ import annotations

from contextlib import ExitStack
import importlib
import os
from pathlib import Path
import sys
import time
from types import ModuleType
from typing import Any, Sequence

from ..capabilities import ModelLoadEvidence, TranscriberCapabilities, WarmupEvidence
from .base import AdapterError, TranscriberConfig, TranscriptRevision


SAMPLE_RATE_HZ = 16_000
VALID_DELAYS_MS = tuple(range(80, 1_201, 80)) + (2_400,)
MIN_PARTIAL_DECODE_INTERVAL_MS = 80
MAX_PARTIAL_DECODE_INTERVAL_MS = 2_400


def transcribe_root(explicit: Path | None) -> Path:
    value = explicit or os.environ.get("TRANSCRIBE_CPP_ROOT")
    if not value:
        raise AdapterError("set transcribe.cpp root in the service profile")
    root = Path(value).expanduser().resolve()
    binding = root / "bindings" / "python" / "src" / "transcribe_cpp"
    if not binding.is_dir():
        raise AdapterError(f"transcribe.cpp Python binding not found: {binding}")
    return root


def find_library(root: Path, explicit: Path | None) -> Path:
    configured = explicit or os.environ.get("TRANSCRIBE_LIBRARY")
    if configured:
        library = Path(configured).expanduser().resolve()
        if not library.is_file():
            raise AdapterError(f"libtranscribe not found: {library}")
        return library
    candidates = (
        root / "build" / "src" / "libtranscribe.so",
        root / "build-shared" / "src" / "libtranscribe.so",
        root / "build" / "src" / "libtranscribe.dylib",
        root / "build-shared" / "src" / "libtranscribe.dylib",
        root / "build" / "bin" / "transcribe.dll",
        root / "build-shared" / "bin" / "transcribe.dll",
    )
    for candidate in candidates:
        if candidate.is_file():
            return candidate.resolve()
    raise AdapterError(
        "shared libtranscribe was not found; build with "
        "-DTRANSCRIBE_BUILD_SHARED=ON or configure an explicit library"
    )


def load_transcribe(root: Path, library: Path) -> ModuleType:
    os.environ["TRANSCRIBE_LIBRARY"] = os.fspath(library)
    binding_src = root / "bindings" / "python" / "src"
    binding_path = os.fspath(binding_src)
    if binding_path not in sys.path:
        sys.path.insert(0, binding_path)
    try:
        return importlib.import_module("transcribe_cpp")
    except Exception as error:
        raise AdapterError(f"failed to load transcribe.cpp Python binding: {error}") from error


class VoxtralTranscriberAdapter:
    """Resident Voxtral Realtime adapter over the transcribe.cpp binding."""

    def __init__(
        self,
        model_path: Path,
        root: Path | None,
        library: Path | None,
        *,
        backend: str = "cuda",
        delay_ms: int = 480,
        partial_decode_interval_ms: int = 240,
        module: ModuleType | Any | None = None,
    ) -> None:
        if delay_ms not in VALID_DELAYS_MS:
            raise AdapterError(f"unsupported Voxtral delay: {delay_ms} ms")
        if (
            partial_decode_interval_ms < MIN_PARTIAL_DECODE_INTERVAL_MS
            or partial_decode_interval_ms > MAX_PARTIAL_DECODE_INTERVAL_MS
            or partial_decode_interval_ms % 80 != 0
        ):
            raise AdapterError(
                "Voxtral partial decode interval must be an 80 ms multiple "
                "between 80 and 2400 ms"
            )
        self.model_path = model_path.expanduser().resolve()
        self.root = root
        self.library = library
        self.backend = backend
        self.delay_ms = delay_ms
        self.partial_decode_interval_ms = partial_decode_interval_ms
        self._module = module
        self._model: Any = None
        self._model_stack: ExitStack | None = None
        self._active = False
        self._load_count = 0
        self._warmup_count = 0

    @property
    def load_count(self) -> int:
        return self._load_count

    def load(self) -> ModelLoadEvidence:
        if self._model is not None:
            return ModelLoadEvidence(elapsed_ms=0, load_count=self._load_count)
        if not self.model_path.is_file():
            raise AdapterError(f"Voxtral model not found: {self.model_path}")
        started = time.perf_counter()
        if self._module is None:
            root = transcribe_root(self.root)
            library = find_library(root, self.library)
            self._module = load_transcribe(root, library)
        stack = ExitStack()
        try:
            model = stack.enter_context(self._module.Model(self.model_path, backend=self.backend))
        except Exception as error:
            stack.close()
            raise AdapterError(f"failed to load Voxtral model: {type(error).__name__}: {error}") from error
        if not model.capabilities.supports_streaming:
            stack.close()
            raise AdapterError(f"{model.arch}/{model.variant} does not support streaming")
        self._model_stack = stack
        self._model = model
        self._load_count += 1
        return ModelLoadEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1000),
            load_count=self._load_count,
        )

    def warmup(self) -> WarmupEvidence:
        started = time.perf_counter()
        self.load()
        self._warmup_count += 1
        return WarmupEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1000),
            warmup_count=self._warmup_count,
        )

    def capabilities(self) -> TranscriberCapabilities:
        if self._model is None:
            raise AdapterError("Voxtral model is not loaded")
        return TranscriberCapabilities(
            adapter_id="voxtral-transcribe-cpp/1",
            model_id=f"{self._model.arch}/{self._model.variant}",
            runtime_id="transcribe.cpp",
            backend=str(self._model.backend),
            sample_rate_hz=SAMPLE_RATE_HZ,
            encoding="pcm_f32",
            channels=1,
            supports_streaming=True,
            timing_precision="utterance",
            resolution_samples=None,
            supported_delay_ms=VALID_DELAYS_MS,
            configured_delay_ms=self.delay_ms,
            partial_decode_interval_ms=self.partial_decode_interval_ms,
            streaming_mode="stateful_native",
        )

    def start(self, config: TranscriberConfig | None = None) -> VoxtralTranscriberSession:
        self.load()
        if self._active:
            raise AdapterError("Voxtral adapter already owns an active session")
        self._active = True
        try:
            return VoxtralTranscriberSession(self, config or TranscriberConfig())
        except Exception:
            self._active = False
            raise

    def _session_closed(self) -> None:
        self._active = False

    def close(self) -> None:
        if self._active:
            raise AdapterError("cannot close Voxtral model with an active session")
        if self._model_stack is not None:
            self._model_stack.close()
        self._model_stack = None
        self._model = None

    def __enter__(self) -> VoxtralTranscriberAdapter:
        self.load()
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


class VoxtralTranscriberSession:
    def __init__(self, adapter: VoxtralTranscriberAdapter, config: TranscriberConfig) -> None:
        delay_ms = adapter.delay_ms if config.delay_ms is None else config.delay_ms
        if delay_ms not in VALID_DELAYS_MS:
            raise AdapterError(f"unsupported Voxtral delay: {delay_ms} ms")
        self._adapter = adapter
        self._stack = ExitStack()
        try:
            model_session = self._stack.enter_context(adapter._model.session())
            family = adapter._module.VoxtralRealtimeStreamOptions(
                num_delay_tokens=delay_ms // 80,
                min_decode_interval_ms=adapter.partial_decode_interval_ms,
            )
            self._stream = self._stack.enter_context(
                model_session.stream(language=config.language, family=family)
            )
        except Exception as error:
            self._stack.close()
            raise AdapterError(
                f"failed to start Voxtral stream: {type(error).__name__}: {error}"
            ) from error
        self._revision = 0
        self._finished = False
        self._closed = False
        self._final_revision: TranscriptRevision | None = None

    def push_pcm(self, samples: Sequence[float]) -> TranscriptRevision | None:
        if self._closed or self._finished:
            raise AdapterError("Voxtral stream is not active")
        try:
            update = self._stream.feed(samples)
        except Exception as error:
            raise AdapterError(f"Voxtral feed failed: {type(error).__name__}: {error}") from error
        if not (update.committed_changed or update.tentative_changed):
            return None
        return self._read_revision(final=False)

    def finish(self) -> TranscriptRevision:
        if self._closed and self._final_revision is None:
            raise AdapterError("Voxtral stream was cancelled")
        if self._final_revision is not None:
            return self._final_revision
        try:
            self._stream.finalize()
            self._finished = True
            self._final_revision = self._read_revision(final=True)
            return self._final_revision
        except Exception as error:
            raise AdapterError(f"Voxtral finalize failed: {type(error).__name__}: {error}") from error

    def cancel(self) -> None:
        self.close()

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._stack.close()
        self._adapter._session_closed()

    def _read_revision(self, *, final: bool) -> TranscriptRevision:
        text = self._stream.text()
        committed = str(getattr(text, "committed"))
        tentative = str(getattr(text, "tentative"))
        full_value = getattr(text, "full", None)
        if callable(full_value):
            full_value = full_value()
        full = committed + tentative if full_value is None else str(full_value)
        self._revision += 1
        return TranscriptRevision(
            revision=self._revision,
            full_text=full,
            committed_text=committed,
            tentative_text=tentative,
            final=final,
        )

    def __enter__(self) -> VoxtralTranscriberSession:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()
