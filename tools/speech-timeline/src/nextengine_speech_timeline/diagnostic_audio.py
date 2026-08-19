"""Explicit, bounded local WAV retention for microphone diagnostics."""

from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
import secrets
import threading
import time
import wave


SAMPLE_RATE_HZ = 16_000
MAX_RECORDINGS = 5
_PREFIX = "record-"


class DiagnosticAudioError(RuntimeError):
    pass


@dataclass(frozen=True)
class DiagnosticAudioRecord:
    record_id: str
    byte_length: int
    duration_ms: int
    created_at_unix_ms: int

    def as_dict(self) -> dict[str, object]:
        return {
            "id": self.record_id,
            "byte_length": self.byte_length,
            "duration_ms": self.duration_ms,
            "created_at_unix_ms": self.created_at_unix_ms,
        }


class DiagnosticAudioStore:
    """Serves only WAV files this process wrote into one external directory."""

    def __init__(self, root: Path, max_records: int = MAX_RECORDINGS) -> None:
        if not 1 <= max_records <= MAX_RECORDINGS:
            raise ValueError(f"max_records must be between 1 and {MAX_RECORDINGS}")
        self.root = root.expanduser().resolve()
        self.max_records = max_records
        self._lock = threading.RLock()

    def start(self) -> None:
        with self._lock:
            self.root.mkdir(mode=0o700, parents=True, exist_ok=True)
            if not self.root.is_dir() or self.root.is_symlink():
                raise DiagnosticAudioError("diagnostic audio root is not a directory")
            os.chmod(self.root, 0o700)
            self._prune_locked()

    def capabilities(self) -> dict[str, object]:
        return {
            "enabled": True,
            "max_records": self.max_records,
            "list_path": "/api/diagnostic-audio",
        }

    def record(self, pcm: bytes) -> DiagnosticAudioRecord:
        if not pcm or len(pcm) % 2:
            raise DiagnosticAudioError("diagnostic WAV requires non-empty aligned PCM")
        if len(pcm) > 960_000:
            raise DiagnosticAudioError("diagnostic WAV exceeds the service turn bound")
        with self._lock:
            self.start()
            record_id = secrets.token_hex(16)
            target = self._path(record_id)
            temporary = self.root / f".{_PREFIX}{record_id}.tmp"
            try:
                with wave.open(str(temporary), "wb") as destination:
                    destination.setnchannels(1)
                    destination.setsampwidth(2)
                    destination.setframerate(SAMPLE_RATE_HZ)
                    destination.writeframes(pcm)
                os.chmod(temporary, 0o600)
                os.replace(temporary, target)
            except OSError as error:
                temporary.unlink(missing_ok=True)
                raise DiagnosticAudioError(f"cannot persist diagnostic WAV: {error}") from error
            record = self._record_from_path(target)
            self._prune_locked()
            return record

    def list_records(self) -> list[DiagnosticAudioRecord]:
        with self._lock:
            return [self._record_from_path(path) for path in self._paths_locked()]

    def read(self, record_id: str) -> bytes | None:
        with self._lock:
            if not _valid_record_id(record_id):
                return None
            path = self._path(record_id)
            if not path.is_file() or path.is_symlink():
                return None
            try:
                return path.read_bytes()
            except OSError:
                return None

    def _paths_locked(self) -> list[Path]:
        if not self.root.is_dir():
            return []
        paths = [path for path in self.root.glob(f"{_PREFIX}*.wav") if _valid_filename(path.name)]
        return sorted(paths, key=lambda path: path.stat().st_mtime_ns, reverse=True)[: self.max_records]

    def _prune_locked(self) -> None:
        if not self.root.is_dir():
            return
        paths = sorted(
            (path for path in self.root.glob(f"{_PREFIX}*.wav") if _valid_filename(path.name)),
            key=lambda path: path.stat().st_mtime_ns,
            reverse=True,
        )
        for path in paths[self.max_records :]:
            path.unlink(missing_ok=True)

    def _path(self, record_id: str) -> Path:
        return self.root / f"{_PREFIX}{record_id}.wav"

    @staticmethod
    def _record_from_path(path: Path) -> DiagnosticAudioRecord:
        stat = path.stat()
        record_id = path.stem.removeprefix(_PREFIX)
        return DiagnosticAudioRecord(
            record_id=record_id,
            byte_length=stat.st_size,
            duration_ms=max(0, (stat.st_size - 44) * 1_000 // (SAMPLE_RATE_HZ * 2)),
            created_at_unix_ms=stat.st_mtime_ns // 1_000_000,
        )


def _valid_record_id(value: str) -> bool:
    return len(value) == 32 and all(character in "0123456789abcdef" for character in value)


def _valid_filename(value: str) -> bool:
    return value.startswith(_PREFIX) and value.endswith(".wav") and _valid_record_id(
        value[len(_PREFIX) : -4]
    )
