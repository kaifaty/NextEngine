from __future__ import annotations

from collections import deque
from collections.abc import Callable, Sequence
import json
from pathlib import Path
import queue
import socket
import subprocess
import threading
import time
from typing import Any, Protocol
from urllib.error import HTTPError, URLError
from urllib.request import urlopen

import numpy as np
from websockets.exceptions import ConnectionClosed
from websockets.sync.client import ClientConnection, connect

from ..capabilities import ModelLoadEvidence, TranscriberCapabilities, WarmupEvidence
from .base import AdapterError, TranscriberConfig, TranscriptRevision


ADAPTER_ID = "gigastt-sidecar/1"
MODEL_ROUTE_ID = "gigastt-v3-rnnt-buffered"
SAMPLE_RATE_HZ = 16_000
DECODE_STRIDE_MS = 800
MAX_WINDOW_MS = 2_500
LEFT_CONTEXT_MS = 1_500
PROTOCOL_VERSION = "1.0"
_READY_TIMEOUT_SECONDS = 60.0
_FINAL_TIMEOUT_SECONDS = 30.0


class _Connection(Protocol):
    def recv(self, timeout: float | None = None) -> str | bytes: ...

    def send(self, message: str | bytes) -> None: ...

    def close(self) -> None: ...


class _Sidecar(Protocol):
    backend: str

    def start(self) -> None: ...

    def connect(self) -> _Connection: ...

    def close(self) -> None: ...


class _ManagedGigasttSidecar:
    """One loopback-only, resident gigastt process owned by the adapter."""

    backend = "cpu"

    def __init__(self, runtime_path: Path, model_dir: Path) -> None:
        self._runtime_path = runtime_path
        self._model_dir = model_dir
        self._port = _available_loopback_port()
        self._process: subprocess.Popen[str] | None = None
        self._log_tail: deque[str] = deque(maxlen=64)
        self._log_thread: threading.Thread | None = None

    def start(self) -> None:
        if self._process is not None:
            return
        command = [
            str(self._runtime_path),
            "--log-level",
            "warn",
            "--offline",
            "serve",
            "--host",
            "127.0.0.1",
            "--port",
            str(self._port),
            "--model-dir",
            str(self._model_dir),
            "--model-variant",
            "rnnt",
            "--punctuation",
            "off",
            "--itn",
            "off",
            "--endpoint-mode",
            "manual",
            "--pool-size",
            "1",
            "--pool-min-size",
            "1",
        ]
        try:
            self._process = subprocess.Popen(
                command,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
                start_new_session=True,
            )
        except OSError as error:
            raise AdapterError(f"cannot start pinned gigastt sidecar: {error}") from error
        assert self._process.stderr is not None
        self._log_thread = threading.Thread(
            target=self._drain_logs,
            args=(self._process.stderr,),
            name="nextengine-gigastt-log",
            daemon=True,
        )
        self._log_thread.start()
        try:
            self._wait_until_ready()
        except BaseException:
            self.close()
            raise

    def connect(self) -> ClientConnection:
        if self._process is None or self._process.poll() is not None:
            raise AdapterError("gigastt sidecar is not running")
        try:
            return connect(
                f"ws://127.0.0.1:{self._port}/v1/ws",
                open_timeout=10,
                close_timeout=2,
                max_size=1024 * 1024,
                compression=None,
            )
        except (OSError, TimeoutError, ConnectionClosed) as error:
            raise AdapterError(f"cannot connect to gigastt sidecar: {error}") from error

    def close(self) -> None:
        process = self._process
        self._process = None
        if process is None:
            return
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        if process.stderr is not None:
            process.stderr.close()
        if self._log_thread is not None:
            self._log_thread.join(timeout=1)
        self._log_thread = None

    def _wait_until_ready(self) -> None:
        deadline = time.monotonic() + _READY_TIMEOUT_SECONDS
        while time.monotonic() < deadline:
            process = self._process
            if process is None or process.poll() is not None:
                detail = " | ".join(self._log_tail) or "no sidecar diagnostics"
                raise AdapterError(f"gigastt sidecar exited during startup: {detail}")
            try:
                with urlopen(
                    f"http://127.0.0.1:{self._port}/ready",
                    timeout=0.5,
                ) as response:
                    payload = json.loads(response.read(4096))
                    if response.status == 200 and payload.get("status") == "ready":
                        return
            except (HTTPError, URLError, TimeoutError, json.JSONDecodeError):
                pass
            time.sleep(0.05)
        detail = " | ".join(self._log_tail) or "no sidecar diagnostics"
        raise AdapterError(f"gigastt sidecar readiness timed out: {detail}")

    def _drain_logs(self, stream: Any) -> None:
        try:
            for line in stream:
                bounded = line.strip()
                if bounded:
                    self._log_tail.append(bounded[:512])
        except (OSError, ValueError):
            pass


def _available_loopback_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        return int(probe.getsockname()[1])


def _launch_sidecar(runtime_path: Path, model_dir: Path) -> _Sidecar:
    return _ManagedGigasttSidecar(runtime_path, model_dir)


class GigasttTranscriberAdapter:
    """Bounded re-decode streaming baseline over pinned gigastt/GigaAM ONNX."""

    def __init__(
        self,
        runtime_path: Path,
        model_dir: Path,
        *,
        model_id: str,
        model_revision: str,
        runtime_version: str,
        launcher: Callable[[Path, Path], _Sidecar] = _launch_sidecar,
    ) -> None:
        self.runtime_path = runtime_path.expanduser().resolve()
        self.model_dir = model_dir.expanduser().resolve()
        self.model_id = model_id
        self.model_revision = model_revision
        self.runtime_version = runtime_version
        self._launcher = launcher
        self._sidecar: _Sidecar | None = None
        self._active = False
        self._load_count = 0
        self._warmup_count = 0

    def load(self) -> ModelLoadEvidence:
        if self._sidecar is not None:
            return ModelLoadEvidence(elapsed_ms=0, load_count=self._load_count)
        started = time.perf_counter()
        sidecar = self._launcher(self.runtime_path, self.model_dir)
        try:
            sidecar.start()
        except BaseException:
            sidecar.close()
            raise
        self._sidecar = sidecar
        self._load_count += 1
        return ModelLoadEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1_000),
            load_count=self._load_count,
        )

    def warmup(self) -> WarmupEvidence:
        started = time.perf_counter()
        self.load()
        self._warmup_count += 1
        return WarmupEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1_000),
            warmup_count=self._warmup_count,
        )

    def capabilities(self) -> TranscriberCapabilities:
        if self._sidecar is None:
            raise AdapterError("gigastt sidecar is not loaded")
        return TranscriberCapabilities(
            adapter_id=ADAPTER_ID,
            model_id=self.model_id,
            runtime_id=f"gigastt/{self.runtime_version}",
            backend=self._sidecar.backend,
            sample_rate_hz=SAMPLE_RATE_HZ,
            encoding="pcm_f32",
            channels=1,
            supports_streaming=True,
            timing_precision="utterance",
            resolution_samples=DECODE_STRIDE_MS * SAMPLE_RATE_HZ // 1_000,
            supported_delay_ms=(DECODE_STRIDE_MS,),
            configured_delay_ms=DECODE_STRIDE_MS,
            partial_decode_interval_ms=DECODE_STRIDE_MS,
            streaming_mode="buffered_emulation",
            streaming_window_ms=MAX_WINDOW_MS,
            streaming_left_context_ms=LEFT_CONTEXT_MS,
        )

    def start(self, config: TranscriberConfig | None = None) -> GigasttTranscriberSession:
        self.load()
        language = (config or TranscriberConfig()).language
        if language is not None and language.lower().replace("_", "-") not in {"ru", "ru-ru"}:
            raise AdapterError("the selected gigastt GigaAM profile accepts Russian audio only")
        if self._active:
            raise AdapterError("gigastt adapter already owns an active session")
        assert self._sidecar is not None
        self._active = True
        try:
            return GigasttTranscriberSession(self, self._sidecar.connect())
        except BaseException:
            self._active = False
            raise

    def _session_closed(self) -> None:
        self._active = False

    def close(self) -> None:
        if self._active:
            raise AdapterError("cannot close gigastt with an active session")
        if self._sidecar is not None:
            self._sidecar.close()
        self._sidecar = None


class GigasttTranscriberSession:
    def __init__(self, adapter: GigasttTranscriberAdapter, connection: _Connection) -> None:
        self._adapter = adapter
        self._connection = connection
        self._responses: queue.Queue[dict[str, Any] | BaseException] = queue.Queue()
        self._closed = False
        self._finished = False
        self._revision = 0
        self._last_text = ""
        self._final_revision: TranscriptRevision | None = None
        try:
            ready = _decode_message(connection.recv(timeout=10))
            supported_rates = ready.get("supported_rates", [])
            if (
                ready.get("type") != "ready"
                or ready.get("version") != PROTOCOL_VERSION
                or (
                    ready.get("sample_rate") != SAMPLE_RATE_HZ
                    and (
                        not isinstance(supported_rates, list)
                        or SAMPLE_RATE_HZ not in supported_rates
                    )
                )
            ):
                raise AdapterError("gigastt returned an incompatible ready frame")
            connection.send(
                json.dumps(
                    {
                        "type": "configure",
                        "sample_rate": SAMPLE_RATE_HZ,
                        "protocol_version": PROTOCOL_VERSION,
                        "punctuation": False,
                        "itn": False,
                        "endpoint_mode": "manual",
                    },
                    separators=(",", ":"),
                )
            )
        except BaseException:
            connection.close()
            raise
        self._reader = threading.Thread(
            target=self._read_responses,
            name="nextengine-gigastt-ws",
            daemon=True,
        )
        self._reader.start()

    def push_pcm(self, samples: Sequence[float]) -> TranscriptRevision | None:
        if self._closed or self._finished:
            raise AdapterError("gigastt stream is not active")
        chunk = np.asarray(samples, dtype=np.float32)
        if chunk.ndim != 1 or chunk.size == 0 or not np.isfinite(chunk).all():
            raise AdapterError("gigastt received invalid mono PCM")
        scaled = np.rint(np.clip(chunk, -1.0, 1.0) * 32768.0)
        pcm16 = np.clip(scaled, -32768, 32767).astype("<i2", copy=False).tobytes()
        try:
            self._connection.send(pcm16)
        except (OSError, ConnectionClosed) as error:
            raise AdapterError(f"gigastt audio send failed: {error}") from error
        return self._drain_partial_responses()

    def finish(self) -> TranscriptRevision:
        if self._closed and self._final_revision is None:
            raise AdapterError("gigastt stream was cancelled")
        if self._final_revision is not None:
            return self._final_revision
        try:
            self._connection.send('{"type":"stop"}')
        except (OSError, ConnectionClosed) as error:
            raise AdapterError(f"gigastt stop failed: {error}") from error
        deadline = time.monotonic() + _FINAL_TIMEOUT_SECONDS
        while time.monotonic() < deadline:
            try:
                response = self._responses.get(timeout=max(0.01, deadline - time.monotonic()))
            except queue.Empty as error:
                raise AdapterError("gigastt final response timed out") from error
            revision = self._revision_from_response(response, allow_final=True)
            if revision is not None and revision.final:
                self._finished = True
                self._final_revision = revision
                return revision
        raise AdapterError("gigastt final response timed out")

    def cancel(self) -> None:
        self.close()

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._connection.close()
        self._reader.join(timeout=2)
        self._adapter._session_closed()

    def _read_responses(self) -> None:
        try:
            while True:
                self._responses.put(_decode_message(self._connection.recv()))
        except ConnectionClosed:
            return
        except BaseException as error:
            if not self._closed:
                self._responses.put(error)

    def _drain_partial_responses(self) -> TranscriptRevision | None:
        latest: TranscriptRevision | None = None
        while True:
            try:
                response = self._responses.get_nowait()
            except queue.Empty:
                return latest
            revision = self._revision_from_response(response, allow_final=False)
            if revision is not None:
                latest = revision

    def _revision_from_response(
        self,
        response: dict[str, Any] | BaseException,
        *,
        allow_final: bool,
    ) -> TranscriptRevision | None:
        if isinstance(response, BaseException):
            raise AdapterError(f"gigastt response reader failed: {response}") from response
        message_type = response.get("type")
        if message_type == "error":
            code = str(response.get("code", "unknown_error"))[:128]
            message = str(response.get("message", "model operation failed"))[:512]
            raise AdapterError(f"gigastt {code}: {message}")
        if message_type not in {"partial", "final"}:
            return None
        if message_type == "final" and not allow_final:
            raise AdapterError("gigastt finalized before the outer turn ended")
        text = response.get("text")
        if not isinstance(text, str):
            raise AdapterError("gigastt transcript frame has no text")
        text = text.strip()
        final = message_type == "final"
        if not final and text == self._last_text:
            return None
        self._last_text = text
        self._revision += 1
        return TranscriptRevision(
            revision=self._revision,
            full_text=text,
            committed_text=text if final else "",
            tentative_text="" if final else text,
            final=final,
        )


def _decode_message(value: str | bytes) -> dict[str, Any]:
    if isinstance(value, bytes):
        try:
            value = value.decode("utf-8")
        except UnicodeDecodeError as error:
            raise AdapterError("gigastt returned non-UTF-8 control data") from error
    try:
        decoded = json.loads(value)
    except json.JSONDecodeError as error:
        raise AdapterError("gigastt returned invalid JSON") from error
    if not isinstance(decoded, dict):
        raise AdapterError("gigastt returned a non-object control frame")
    return decoded
