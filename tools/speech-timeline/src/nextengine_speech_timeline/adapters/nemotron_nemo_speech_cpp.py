from __future__ import annotations

from collections.abc import Callable, Sequence
import ctypes
from dataclasses import dataclass
import multiprocessing
from multiprocessing.connection import Connection
from pathlib import Path
import signal
import time
from typing import Any, Protocol

import numpy as np

from ..capabilities import ModelLoadEvidence, TranscriberCapabilities, WarmupEvidence
from .base import AdapterError, TranscriberConfig, TranscriptRevision


ADAPTER_ID = "nemotron-nemo-speech-cpp/1"
MODEL_ROUTE_ID = "nemotron-3.5-streaming"
SAMPLE_RATE_HZ = 16_000
SUPPORTED_RIGHT_CONTEXT = (0, 1, 6, 13)
SUPPORTED_DELAY_MS = (80, 160, 560, 1_120)


@dataclass(frozen=True)
class _NativeUpdate:
    text: str
    final: bool


class _NativeStream(Protocol):
    def push(self, samples: np.ndarray) -> _NativeUpdate | None: ...

    def finish(self) -> _NativeUpdate | None: ...

    def close(self) -> None: ...


class _Backend(Protocol):
    device: str
    runtime_version: str

    def start(self, language: str | None) -> _NativeStream: ...

    def close(self) -> None: ...


class _BackendConfig(ctypes.Structure):
    _fields_ = [("size", ctypes.c_size_t), ("gpu", ctypes.c_int32)]


class _ModelConfig(ctypes.Structure):
    _fields_ = [
        ("size", ctypes.c_size_t),
        ("path", ctypes.c_char_p),
        ("name", ctypes.c_char_p),
    ]


class _StreamingConfig(ctypes.Structure):
    _fields_ = [
        ("size", ctypes.c_size_t),
        ("chunk_size", ctypes.c_float),
        ("ctc_left_padding", ctypes.c_float),
        ("ctc_right_padding", ctypes.c_float),
        ("rnnt_right_context", ctypes.c_int32),
    ]


class _RecognizerConfig(ctypes.Structure):
    _fields_ = [
        ("size", ctypes.c_size_t),
        ("backend", ctypes.POINTER(_BackendConfig)),
        ("model", ctypes.POINTER(_ModelConfig)),
        ("streaming", ctypes.POINTER(_StreamingConfig)),
        ("decoder", ctypes.c_void_p),
        ("vad", ctypes.c_void_p),
        ("endpointing", ctypes.c_void_p),
        ("postproc", ctypes.c_void_p),
        ("diar", ctypes.c_void_p),
        ("batching", ctypes.c_void_p),
    ]


class _RecognitionOptions(ctypes.Structure):
    _fields_ = [
        ("size", ctypes.c_size_t),
        ("request_id", ctypes.c_char_p),
        ("language_code", ctypes.c_char_p),
        ("interim_results", ctypes.c_bool),
        ("enable_word_time_offsets", ctypes.c_bool),
        ("enable_automatic_punctuation", ctypes.c_bool),
        ("verbatim_transcripts", ctypes.c_bool),
        ("profanity_filter", ctypes.c_bool),
        ("stop_history_eou_ms", ctypes.c_int32),
        ("speech_contexts", ctypes.c_void_p),
        ("speech_context_count", ctypes.c_size_t),
        ("max_alternatives", ctypes.c_int32),
        ("enable_speaker_diarization", ctypes.c_bool),
        ("max_speaker_count", ctypes.c_int32),
    ]


def _configure_abi(library: ctypes.CDLL) -> None:
    pointer = ctypes.c_void_p
    library.nemo_speech_asr_create.argtypes = [ctypes.POINTER(_RecognizerConfig), ctypes.POINTER(pointer)]
    library.nemo_speech_asr_create.restype = ctypes.c_int
    library.nemo_speech_asr_destroy.argtypes = [pointer]
    library.nemo_speech_asr_destroy.restype = None
    library.nemo_speech_asr_recognition_options_default.argtypes = []
    library.nemo_speech_asr_recognition_options_default.restype = _RecognitionOptions
    library.nemo_speech_asr_streaming_recognize.argtypes = [
        pointer,
        ctypes.POINTER(_RecognitionOptions),
        ctypes.POINTER(pointer),
    ]
    library.nemo_speech_asr_streaming_recognize.restype = ctypes.c_int
    library.nemo_speech_asr_stream_push_f32.argtypes = [
        pointer,
        ctypes.POINTER(ctypes.c_float),
        ctypes.c_size_t,
        ctypes.c_int32,
    ]
    library.nemo_speech_asr_stream_push_f32.restype = ctypes.c_int
    library.nemo_speech_asr_stream_finish.argtypes = [pointer]
    library.nemo_speech_asr_stream_finish.restype = ctypes.c_int
    library.nemo_speech_asr_stream_next.argtypes = [pointer, ctypes.POINTER(pointer)]
    library.nemo_speech_asr_stream_next.restype = ctypes.c_int
    library.nemo_speech_asr_stream_close.argtypes = [pointer]
    library.nemo_speech_asr_stream_close.restype = None
    library.nemo_speech_asr_result_is_final.argtypes = [pointer]
    library.nemo_speech_asr_result_is_final.restype = ctypes.c_bool
    library.nemo_speech_asr_result_alternative_count.argtypes = [pointer]
    library.nemo_speech_asr_result_alternative_count.restype = ctypes.c_size_t
    library.nemo_speech_asr_result_transcript.argtypes = [pointer, ctypes.c_size_t]
    library.nemo_speech_asr_result_transcript.restype = ctypes.c_char_p
    library.nemo_speech_asr_result_destroy.argtypes = [pointer]
    library.nemo_speech_asr_result_destroy.restype = None
    library.nemo_speech_asr_last_error.argtypes = []
    library.nemo_speech_asr_last_error.restype = ctypes.c_char_p
    library.nemo_speech_asr_version.argtypes = []
    library.nemo_speech_asr_version.restype = ctypes.c_char_p


class _PinnedBackend:
    def __init__(
        self,
        implementation_library: Path,
        abi_library: Path,
        model_path: Path,
        gpu: int,
        right_context: int,
    ) -> None:
        try:
            self._implementation = ctypes.CDLL(
                str(implementation_library),
                mode=getattr(ctypes, "RTLD_GLOBAL", 0),
            )
            self._library = ctypes.CDLL(str(abi_library))
            _configure_abi(self._library)
        except (OSError, AttributeError) as error:
            raise AdapterError(f"failed to load pinned NeMo-Speech.cpp runtime: {error}") from error

        self._recognizer = ctypes.c_void_p()
        backend = _BackendConfig(ctypes.sizeof(_BackendConfig), gpu)
        encoded_model = str(model_path).encode("utf-8")
        model = _ModelConfig(ctypes.sizeof(_ModelConfig), encoded_model, None)
        streaming = _StreamingConfig(
            ctypes.sizeof(_StreamingConfig),
            0.16,
            1.92,
            1.92,
            right_context,
        )
        config = _RecognizerConfig(
            ctypes.sizeof(_RecognizerConfig),
            ctypes.pointer(backend),
            ctypes.pointer(model),
            ctypes.pointer(streaming),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        status = self._library.nemo_speech_asr_create(
            ctypes.byref(config), ctypes.byref(self._recognizer)
        )
        if status != 0 or not self._recognizer:
            raise AdapterError(f"failed to load Nemotron model: {self._last_error()}")
        version = self._library.nemo_speech_asr_version()
        self.runtime_version = version.decode("utf-8", errors="replace") if version else "unknown"
        self.device = "cpu" if gpu < 0 else f"cuda:{gpu}"

    def start(self, language: str | None) -> _NativeStream:
        language_bytes = language.encode("utf-8") if language else None
        options = self._library.nemo_speech_asr_recognition_options_default()
        options.language_code = language_bytes
        options.interim_results = True
        stream = ctypes.c_void_p()
        status = self._library.nemo_speech_asr_streaming_recognize(
            self._recognizer, ctypes.byref(options), ctypes.byref(stream)
        )
        if status != 0 or not stream:
            raise AdapterError(f"failed to start Nemotron stream: {self._last_error()}")
        return _PinnedStream(self._library, stream, language_bytes)

    def close(self) -> None:
        if self._recognizer:
            self._library.nemo_speech_asr_destroy(self._recognizer)
            self._recognizer = ctypes.c_void_p()

    def _last_error(self) -> str:
        value = self._library.nemo_speech_asr_last_error()
        return value.decode("utf-8", errors="replace") if value else "unknown runtime error"


class _PinnedStream:
    def __init__(
        self,
        library: ctypes.CDLL,
        stream: ctypes.c_void_p,
        language_bytes: bytes | None,
    ) -> None:
        self._library = library
        self._stream = stream
        self._language_bytes = language_bytes
        self._closed = False

    def push(self, samples: np.ndarray) -> _NativeUpdate | None:
        contiguous = np.ascontiguousarray(samples, dtype=np.float32)
        status = self._library.nemo_speech_asr_stream_push_f32(
            self._stream,
            contiguous.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
            contiguous.size,
            SAMPLE_RATE_HZ,
        )
        self._check(status, "push audio")
        return self._drain()

    def finish(self) -> _NativeUpdate | None:
        status = self._library.nemo_speech_asr_stream_finish(self._stream)
        self._check(status, "finish stream")
        return self._drain()

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._library.nemo_speech_asr_stream_close(self._stream)
        self._stream = ctypes.c_void_p()

    def _drain(self) -> _NativeUpdate | None:
        latest: _NativeUpdate | None = None
        while True:
            result = ctypes.c_void_p()
            status = self._library.nemo_speech_asr_stream_next(
                self._stream, ctypes.byref(result)
            )
            self._check(status, "decode stream")
            if not result:
                return latest
            try:
                alternatives = self._library.nemo_speech_asr_result_alternative_count(result)
                transcript = (
                    self._library.nemo_speech_asr_result_transcript(result, 0)
                    if alternatives > 0
                    else None
                )
                latest = _NativeUpdate(
                    text=(transcript.decode("utf-8", errors="replace") if transcript else "").strip(),
                    final=bool(self._library.nemo_speech_asr_result_is_final(result)),
                )
            finally:
                self._library.nemo_speech_asr_result_destroy(result)

    def _check(self, status: int, operation: str) -> None:
        if status == 0:
            return
        value = self._library.nemo_speech_asr_last_error()
        detail = value.decode("utf-8", errors="replace") if value else "unknown runtime error"
        raise AdapterError(f"Nemotron {operation} failed: {detail}")


def _load_pinned_backend(
    implementation_library: Path,
    abi_library: Path,
    model_path: Path,
    gpu: int,
    right_context: int,
) -> _Backend:
    return _PinnedBackend(
        implementation_library,
        abi_library,
        model_path,
        gpu,
        right_context,
    )


def _worker_main(
    connection: Connection,
    implementation_library: str,
    abi_library: str,
    model_path: str,
    gpu: int,
    right_context: int,
) -> None:
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    backend: _PinnedBackend | None = None
    stream: _NativeStream | None = None
    try:
        backend = _PinnedBackend(
            Path(implementation_library),
            Path(abi_library),
            Path(model_path),
            gpu,
            right_context,
        )
        connection.send(
            {
                "status": "ready",
                "device": backend.device,
                "runtime_version": backend.runtime_version,
            }
        )
        while True:
            request = connection.recv()
            if not isinstance(request, tuple) or not request:
                raise AdapterError("invalid parent request")
            operation = request[0]
            try:
                if operation == "start":
                    if stream is not None:
                        raise AdapterError("worker already owns an active stream")
                    language = request[1]
                    if language is not None and not isinstance(language, str):
                        raise AdapterError("invalid worker language")
                    stream = backend.start(language)
                    connection.send({"status": "ok"})
                elif operation == "push":
                    if stream is None:
                        raise AdapterError("worker stream is not active")
                    payload = request[1]
                    if not isinstance(payload, bytes) or len(payload) == 0 or len(payload) % 4:
                        raise AdapterError("invalid worker PCM")
                    update = stream.push(np.frombuffer(payload, dtype=np.float32))
                    connection.send(_worker_update(update))
                elif operation == "finish":
                    if stream is None:
                        raise AdapterError("worker stream is not active")
                    update = stream.finish()
                    connection.send(_worker_update(update))
                elif operation == "stream_close":
                    if stream is not None:
                        stream.close()
                        stream = None
                    connection.send({"status": "ok"})
                elif operation == "close":
                    if stream is not None:
                        stream.close()
                        stream = None
                    backend.close()
                    backend = None
                    connection.send({"status": "ok"})
                    return
                else:
                    raise AdapterError("unsupported worker operation")
            except Exception as error:
                connection.send(
                    {
                        "status": "error",
                        "error": f"{type(error).__name__}: {str(error)[:512]}",
                    }
                )
    except EOFError:
        pass
    except Exception as error:
        try:
            connection.send(
                {
                    "status": "error",
                    "error": f"{type(error).__name__}: {str(error)[:512]}",
                }
            )
        except (BrokenPipeError, EOFError, OSError):
            pass
    finally:
        if stream is not None:
            stream.close()
        if backend is not None:
            backend.close()
        connection.close()


def _worker_update(update: _NativeUpdate | None) -> dict[str, object]:
    return {
        "status": "ok",
        "update": None if update is None else {"text": update.text, "final": update.final},
    }


class _ProcessBackend:
    """Isolates NVIDIA's patched GGML from Voxtral's incompatible GGML SONAMEs."""

    def __init__(
        self,
        implementation_library: Path,
        abi_library: Path,
        model_path: Path,
        gpu: int,
        right_context: int,
    ) -> None:
        context = multiprocessing.get_context("spawn")
        parent, child = context.Pipe()
        self._connection = parent
        self._process = context.Process(
            target=_worker_main,
            args=(
                child,
                str(implementation_library),
                str(abi_library),
                str(model_path),
                gpu,
                right_context,
            ),
            name="nextengine-nemotron",
            daemon=True,
        )
        self._closed = False
        self._process.start()
        child.close()
        try:
            response = self._receive(120.0, "load worker")
        except BaseException:
            self.close()
            raise
        if response.get("status") != "ready":
            self.close()
            raise AdapterError(
                f"failed to load isolated Nemotron worker: {response.get('error', 'invalid response')}"
            )
        device = response.get("device")
        version = response.get("runtime_version")
        if not isinstance(device, str) or not isinstance(version, str):
            self.close()
            raise AdapterError("isolated Nemotron worker returned invalid capabilities")
        self.device = device
        self.runtime_version = version

    def start(self, language: str | None) -> _NativeStream:
        self._request(("start", language), "start stream")
        return _ProcessStream(self)

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        if self._process.is_alive():
            try:
                self._connection.send(("close",))
                self._receive(5.0, "close worker")
            except (AdapterError, BrokenPipeError, EOFError, OSError):
                pass
        self._connection.close()
        self._process.join(timeout=5.0)
        if self._process.is_alive():
            self._process.terminate()
            self._process.join(timeout=5.0)

    def request(self, request: tuple[object, ...], operation: str) -> dict[str, Any]:
        return self._request(request, operation)

    def _request(self, request: tuple[object, ...], operation: str) -> dict[str, Any]:
        if self._closed:
            raise AdapterError("isolated Nemotron worker is closed")
        try:
            self._connection.send(request)
        except (BrokenPipeError, EOFError, OSError) as error:
            raise AdapterError(f"Nemotron worker {operation} failed: {error}") from error
        response = self._receive(30.0, operation)
        if response.get("status") != "ok":
            raise AdapterError(
                f"Nemotron worker {operation} failed: {response.get('error', 'invalid response')}"
            )
        return response

    def _receive(self, timeout: float, operation: str) -> dict[str, Any]:
        if not self._connection.poll(timeout):
            state = "exited" if not self._process.is_alive() else "timed out"
            raise AdapterError(f"Nemotron worker {operation} {state}")
        try:
            response = self._connection.recv()
        except (EOFError, OSError) as error:
            raise AdapterError(f"Nemotron worker {operation} disconnected: {error}") from error
        if not isinstance(response, dict) or not isinstance(response.get("status"), str):
            raise AdapterError(f"Nemotron worker {operation} returned an invalid response")
        return response


class _ProcessStream:
    def __init__(self, backend: _ProcessBackend) -> None:
        self._backend = backend
        self._closed = False

    def push(self, samples: np.ndarray) -> _NativeUpdate | None:
        contiguous = np.ascontiguousarray(samples, dtype=np.float32)
        response = self._backend.request(("push", contiguous.tobytes()), "push audio")
        return _decode_worker_update(response)

    def finish(self) -> _NativeUpdate | None:
        response = self._backend.request(("finish",), "finish stream")
        return _decode_worker_update(response)

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._backend.request(("stream_close",), "close stream")


def _decode_worker_update(response: dict[str, Any]) -> _NativeUpdate | None:
    update = response.get("update")
    if update is None:
        return None
    if (
        not isinstance(update, dict)
        or not isinstance(update.get("text"), str)
        or not isinstance(update.get("final"), bool)
    ):
        raise AdapterError("Nemotron worker returned an invalid transcript update")
    return _NativeUpdate(text=update["text"], final=update["final"])


def _load_process_backend(
    implementation_library: Path,
    abi_library: Path,
    model_path: Path,
    gpu: int,
    right_context: int,
) -> _Backend:
    return _ProcessBackend(
        implementation_library,
        abi_library,
        model_path,
        gpu,
        right_context,
    )


class NemotronTranscriberAdapter:
    """Resident cache-aware Nemotron 3.5 streaming adapter over the stable C ABI."""

    def __init__(
        self,
        model_path: Path,
        implementation_library: Path,
        abi_library: Path,
        *,
        model_id: str,
        model_revision: str,
        gpu: int = 0,
        right_context: int = 1,
        loader: Callable[[Path, Path, Path, int, int], _Backend] = _load_process_backend,
    ) -> None:
        if right_context not in SUPPORTED_RIGHT_CONTEXT:
            raise AdapterError(
                f"Nemotron right context must be one of {SUPPORTED_RIGHT_CONTEXT}"
            )
        if gpu < -1:
            raise AdapterError("Nemotron GPU must be -1 for CPU or a non-negative index")
        self.model_path = model_path.expanduser().resolve()
        self.implementation_library = implementation_library.expanduser().resolve()
        self.abi_library = abi_library.expanduser().resolve()
        self.model_id = model_id
        self.model_revision = model_revision
        self.gpu = gpu
        self.right_context = right_context
        self._loader = loader
        self._backend: _Backend | None = None
        self._active = False
        self._load_count = 0
        self._warmup_count = 0

    @property
    def load_count(self) -> int:
        return self._load_count

    def load(self) -> ModelLoadEvidence:
        if self._backend is not None:
            return ModelLoadEvidence(elapsed_ms=0, load_count=self._load_count)
        for path, name in (
            (self.model_path, "Nemotron model"),
            (self.implementation_library, "NeMo-Speech.cpp implementation library"),
            (self.abi_library, "NeMo-Speech.cpp C ABI library"),
        ):
            if not path.is_file():
                raise AdapterError(f"{name} not found: {path}")
        started = time.perf_counter()
        self._backend = self._loader(
            self.implementation_library,
            self.abi_library,
            self.model_path,
            self.gpu,
            self.right_context,
        )
        self._load_count += 1
        return ModelLoadEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1_000),
            load_count=self._load_count,
        )

    def warmup(self) -> WarmupEvidence:
        started = time.perf_counter()
        self.load()
        assert self._backend is not None
        stream = self._backend.start("ru")
        try:
            sample_clock = np.arange(SAMPLE_RATE_HZ, dtype=np.float32)
            probe = (0.05 * np.sin(2.0 * np.pi * 220.0 * sample_clock / SAMPLE_RATE_HZ)).astype(
                np.float32
            )
            for offset in range(0, probe.size, SAMPLE_RATE_HZ * 80 // 1_000):
                stream.push(probe[offset : offset + SAMPLE_RATE_HZ * 80 // 1_000])
            stream.finish()
        finally:
            stream.close()
        self._warmup_count += 1
        return WarmupEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1_000),
            warmup_count=self._warmup_count,
        )

    def capabilities(self) -> TranscriberCapabilities:
        if self._backend is None:
            raise AdapterError("Nemotron model is not loaded")
        return TranscriberCapabilities(
            adapter_id=ADAPTER_ID,
            model_id=self.model_id,
            runtime_id=f"NeMo-Speech.cpp/{self._backend.runtime_version}",
            backend=self._backend.device,
            sample_rate_hz=SAMPLE_RATE_HZ,
            encoding="pcm_f32",
            channels=1,
            supports_streaming=True,
            timing_precision="utterance",
            resolution_samples=None,
            supported_delay_ms=SUPPORTED_DELAY_MS,
            configured_delay_ms=(self.right_context + 1) * 80,
            partial_decode_interval_ms=80,
        )

    def start(self, config: TranscriberConfig | None = None) -> NemotronTranscriberSession:
        self.load()
        language = (config or TranscriberConfig()).language
        if language is not None and language.lower().replace("_", "-") not in {"ru", "ru-ru"}:
            raise AdapterError("the selected Nemotron profile accepts Russian audio only")
        if self._active:
            raise AdapterError("Nemotron adapter already owns an active session")
        assert self._backend is not None
        self._active = True
        try:
            return NemotronTranscriberSession(self, self._backend.start("ru"))
        except Exception:
            self._active = False
            raise

    def _session_closed(self) -> None:
        self._active = False

    def close(self) -> None:
        if self._active:
            raise AdapterError("cannot close Nemotron with an active session")
        if self._backend is not None:
            self._backend.close()
        self._backend = None


class NemotronTranscriberSession:
    def __init__(self, adapter: NemotronTranscriberAdapter, stream: _NativeStream) -> None:
        self._adapter = adapter
        self._stream = stream
        self._revision = 0
        self._text = ""
        self._closed = False
        self._finished = False
        self._final_revision: TranscriptRevision | None = None

    def push_pcm(self, samples: Sequence[float]) -> TranscriptRevision | None:
        if self._closed or self._finished:
            raise AdapterError("Nemotron stream is not active")
        chunk = np.asarray(samples, dtype=np.float32)
        if chunk.ndim != 1 or chunk.size == 0 or not np.isfinite(chunk).all():
            raise AdapterError("Nemotron received invalid mono PCM")
        update = self._stream.push(chunk)
        if update is None or update.text == self._text:
            return None
        self._text = update.text
        return self._revision_value(final=False)

    def finish(self) -> TranscriptRevision:
        if self._closed and self._final_revision is None:
            raise AdapterError("Nemotron stream was cancelled")
        if self._final_revision is not None:
            return self._final_revision
        update = self._stream.finish()
        if update is not None:
            self._text = update.text
        self._finished = True
        self._final_revision = self._revision_value(final=True)
        return self._final_revision

    def cancel(self) -> None:
        self.close()

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._stream.close()
        self._adapter._session_closed()

    def _revision_value(self, *, final: bool) -> TranscriptRevision:
        self._revision += 1
        return TranscriptRevision(
            revision=self._revision,
            full_text=self._text,
            committed_text=self._text if final else "",
            tentative_text="" if final else self._text,
            final=final,
        )
