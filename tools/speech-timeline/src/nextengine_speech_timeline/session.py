from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import threading


SAMPLE_RATE_HZ = 16_000
SAMPLE_WIDTH_BYTES = 2
DEFAULT_MAX_FRAME_BYTES = 32_000
DEFAULT_MAX_TURN_BYTES = 960_000


class SessionError(RuntimeError):
    def __init__(self, code: str, detail: str) -> None:
        super().__init__(detail)
        self.code = code


class SessionState(str, Enum):
    CONNECTED = "connected"
    AUTHENTICATED = "authenticated"
    ACTIVE = "active"
    FINALIZING = "finalizing"
    FINAL = "final"
    CANCELLED = "cancelled"
    FAILED = "failed"


@dataclass(frozen=True)
class SessionBounds:
    max_frame_bytes: int = DEFAULT_MAX_FRAME_BYTES
    max_turn_bytes: int = DEFAULT_MAX_TURN_BYTES

    def __post_init__(self) -> None:
        if self.max_frame_bytes <= 0 or self.max_frame_bytes % SAMPLE_WIDTH_BYTES:
            raise ValueError("max_frame_bytes must be a positive aligned PCM size")
        if self.max_turn_bytes < self.max_frame_bytes or self.max_turn_bytes % SAMPLE_WIDTH_BYTES:
            raise ValueError("max_turn_bytes must be aligned and at least one frame")


@dataclass(frozen=True)
class PcmFrame:
    sequence: int
    start_sample: int
    end_sample: int
    payload: bytes


class SpeechSession:
    """One bounded explicit-finish utterance and its authoritative sample clock."""

    def __init__(self, bounds: SessionBounds | None = None) -> None:
        self.bounds = bounds or SessionBounds()
        self.state = SessionState.CONNECTED
        self.session_id: str | None = None
        self.locale: str | None = None
        self.generation = 0
        self._buffer = bytearray()
        self._received_bytes = 0
        self._sequence = 0
        self._terminal_emitted = False
        self._lock = threading.RLock()
        self._full_copy_bytes = 0
        self._window_copy_bytes = 0
        self._window_copy_calls = 0
        self._peak_buffer_bytes = 0

    @property
    def total_samples(self) -> int:
        with self._lock:
            return self._received_bytes // SAMPLE_WIDTH_BYTES

    @property
    def pcm_bytes(self) -> bytes:
        with self._lock:
            payload = bytes(self._buffer)
            self._full_copy_bytes += len(payload)
            return payload

    def pcm_window(self, start_sample: int, end_sample: int) -> bytes:
        """Copy only one bounded audio window from the authoritative buffer."""
        with self._lock:
            if start_sample < 0 or end_sample < start_sample:
                raise SessionError("INVALID_AUDIO_WINDOW", "audio window bounds are invalid")
            total_samples = self._received_bytes // SAMPLE_WIDTH_BYTES
            if end_sample > total_samples:
                raise SessionError("INVALID_AUDIO_WINDOW", "audio window exceeds received PCM")
            payload = bytes(
                self._buffer[
                    start_sample * SAMPLE_WIDTH_BYTES : end_sample * SAMPLE_WIDTH_BYTES
                ]
            )
            self._window_copy_bytes += len(payload)
            self._window_copy_calls += 1
            return payload

    def copy_metrics(self) -> dict[str, int]:
        with self._lock:
            return {
                "full_copy_bytes": self._full_copy_bytes,
                "window_copy_bytes": self._window_copy_bytes,
                "window_copy_calls": self._window_copy_calls,
                "buffer_bytes": len(self._buffer),
                "peak_buffer_bytes": self._peak_buffer_bytes,
            }

    def authenticate(self) -> None:
        with self._lock:
            if self.state is SessionState.AUTHENTICATED:
                return
            if self.state is not SessionState.CONNECTED:
                raise SessionError("INVALID_STATE", "connection cannot be authenticated now")
            self.state = SessionState.AUTHENTICATED

    def start(self, session_id: str, locale: str | None) -> int:
        with self._lock:
            if self.state is not SessionState.AUTHENTICATED:
                raise SessionError("DUPLICATE_START", "connection already owns or completed a session")
            self.session_id = session_id
            self.locale = locale
            self.generation += 1
            self.state = SessionState.ACTIVE
            return self.generation

    def append_pcm(self, payload: bytes) -> PcmFrame:
        with self._lock:
            if self.state is not SessionState.ACTIVE:
                raise SessionError("PCM_NOT_ACCEPTED", "PCM is accepted only in active state")
            if not isinstance(payload, bytes):
                raise SessionError("INVALID_PCM", "PCM frame must be bytes")
            if not payload:
                raise SessionError("EMPTY_PCM", "PCM frame must not be empty")
            if len(payload) % SAMPLE_WIDTH_BYTES:
                raise SessionError("MISALIGNED_PCM", "PCM frame must contain complete int16 samples")
            if len(payload) > self.bounds.max_frame_bytes:
                raise SessionError("FRAME_TOO_LARGE", "PCM frame exceeds the configured ceiling")
            if self._received_bytes + len(payload) > self.bounds.max_turn_bytes:
                raise SessionError("TURN_TOO_LARGE", "utterance exceeds the configured ceiling")
            start = self._received_bytes // SAMPLE_WIDTH_BYTES
            self._buffer.extend(payload)
            self._received_bytes += len(payload)
            self._peak_buffer_bytes = max(self._peak_buffer_bytes, len(self._buffer))
            end = self._received_bytes // SAMPLE_WIDTH_BYTES
            frame = PcmFrame(self._sequence, start, end, payload)
            self._sequence += 1
            return frame

    def begin_finish(self, session_id: str) -> bool:
        with self._lock:
            self._require_identity(session_id)
            if self.state in {SessionState.FINALIZING, SessionState.FINAL}:
                return False
            if self.state is not SessionState.ACTIVE:
                raise SessionError("INVALID_STATE", "session cannot be finalized now")
            self.state = SessionState.FINALIZING
            return True

    def complete(self, session_id: str) -> bool:
        with self._lock:
            self._require_identity(session_id)
            if self.state is SessionState.FINAL:
                return False
            if self.state is not SessionState.FINALIZING:
                raise SessionError("INVALID_STATE", "session is not finalizing")
            self.state = SessionState.FINAL
            self._buffer.clear()
            return True

    def cancel(self, session_id: str | None = None) -> bool:
        with self._lock:
            if session_id is not None:
                self._require_identity(session_id)
            if self.state in {SessionState.CANCELLED, SessionState.FINAL, SessionState.FAILED}:
                return False
            if self.state not in {SessionState.ACTIVE, SessionState.FINALIZING}:
                raise SessionError("INVALID_STATE", "session cannot be cancelled now")
            self.state = SessionState.CANCELLED
            self._buffer.clear()
            return True

    def fail(self) -> bool:
        with self._lock:
            if self.state in {SessionState.FINAL, SessionState.CANCELLED, SessionState.FAILED}:
                return False
            self.state = SessionState.FAILED
            self._buffer.clear()
            return True

    def claim_terminal_event(self) -> bool:
        with self._lock:
            if self.state not in {SessionState.FINAL, SessionState.CANCELLED, SessionState.FAILED}:
                raise SessionError("INVALID_STATE", "session is not terminal")
            if self._terminal_emitted:
                return False
            self._terminal_emitted = True
            return True

    def is_generation_current(self, generation: int) -> bool:
        with self._lock:
            return self.generation == generation and self.state in {
                SessionState.ACTIVE,
                SessionState.FINALIZING,
            }

    def _require_identity(self, session_id: str) -> None:
        if self.session_id != session_id:
            raise SessionError("SESSION_MISMATCH", "session identity does not match")
