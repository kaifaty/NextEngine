from __future__ import annotations

from dataclasses import dataclass
import math
from typing import Literal, Protocol

import numpy as np


SAMPLE_RATE_HZ = 16_000
ActivityState = Literal["speech", "no_speech"]


@dataclass(frozen=True)
class VoiceActivityConfig:
    """Deterministic baseline VAD settings for the resident service.

    This is deliberately a replaceable energy gate, not a claim that RMS is a
    complete noise-robust VAD. It gives the service a safe no-speech abstention
    path while a Silero/WebRTC adapter can be evaluated behind the same API.
    """

    frame_ms: int = 20
    speech_threshold_dbfs: float = -45.0
    silence_threshold_dbfs: float = -55.0
    start_frames: int = 2
    end_frames: int = 20
    pre_roll_ms: int = 200
    min_voiced_ms: int = 600
    min_voiced_ratio: float = 0.35

    def __post_init__(self) -> None:
        if self.frame_ms not in {10, 20, 30}:
            raise ValueError("frame_ms must be 10, 20 or 30")
        if not math.isfinite(self.speech_threshold_dbfs) or not math.isfinite(
            self.silence_threshold_dbfs
        ):
            raise ValueError("VAD thresholds must be finite")
        if self.silence_threshold_dbfs >= self.speech_threshold_dbfs:
            raise ValueError("silence threshold must be below speech threshold")
        if self.start_frames <= 0 or self.end_frames <= 0:
            raise ValueError("VAD hysteresis frame counts must be positive")
        if self.pre_roll_ms < 0 or self.min_voiced_ms <= 0:
            raise ValueError("VAD durations must be non-negative/positive")
        if not 0.0 < self.min_voiced_ratio <= 1.0:
            raise ValueError("min_voiced_ratio must be in (0, 1]")

    @property
    def frame_samples(self) -> int:
        return SAMPLE_RATE_HZ * self.frame_ms // 1_000

    @property
    def pre_roll_samples(self) -> int:
        return SAMPLE_RATE_HZ * self.pre_roll_ms // 1_000

    @property
    def min_voiced_samples(self) -> int:
        return SAMPLE_RATE_HZ * self.min_voiced_ms // 1_000


@dataclass(frozen=True)
class SpeechActivityWindow:
    start_sample: int
    end_sample: int
    voiced_samples: int
    sample_count: int
    state: ActivityState
    eligible: bool

    @property
    def voiced_ratio(self) -> float:
        if self.sample_count <= 0:
            return 0.0
        return self.voiced_samples / self.sample_count

    @property
    def eligible_for_affect(self) -> bool:
        return self.eligible


@dataclass(frozen=True)
class SpeechActivitySegment:
    start_sample: int
    end_sample: int
    state: ActivityState
    voiced_samples: int
    voiced_ratio: float


@dataclass(frozen=True)
class _ActivityFrame:
    start_sample: int
    end_sample: int
    voiced: bool
    dbfs: float


class VoiceActivityDetector(Protocol):
    state: ActivityState

    def feed_pcm16(self, start_sample: int, payload: bytes) -> bool: ...

    def flush(self) -> bool: ...

    def window(self, start_sample: int, end_sample: int) -> SpeechActivityWindow: ...

    def timeline(self, total_samples: int) -> tuple[SpeechActivitySegment, ...]: ...

    def latest_speech_span(
        self, total_samples: int, *, max_samples: int = 2 * SAMPLE_RATE_HZ
    ) -> tuple[int, int] | None: ...

    def capabilities(self) -> dict[str, object]: ...


class EnergyVoiceActivityDetector:
    """Bounded stateful energy VAD with a global sample-clock timeline."""

    def __init__(self, config: VoiceActivityConfig | None = None) -> None:
        self.config = config or VoiceActivityConfig()
        self._pending = np.empty(0, dtype=np.int16)
        self._pending_start_sample = 0
        self._next_sample = 0
        self._frames: list[_ActivityFrame] = []
        self._state: ActivityState = "no_speech"
        self._positive_run = 0
        self._negative_run = 0
        self._active_start: int | None = None
        self._speech_runs: list[tuple[int, int]] = []

    @property
    def state(self) -> ActivityState:
        return self._state

    @property
    def next_sample(self) -> int:
        return self._next_sample

    def feed_pcm16(self, start_sample: int, payload: bytes) -> bool:
        """Consume contiguous PCM and return whether the state changed."""
        if start_sample != self._next_sample:
            raise ValueError(
                f"VAD input is not contiguous: expected {self._next_sample}, got {start_sample}"
            )
        if not payload or len(payload) % 2:
            raise ValueError("VAD PCM must be non-empty aligned int16")
        samples = np.frombuffer(payload, dtype="<i2")
        return self.feed_samples(start_sample, samples)

    def feed_samples(self, start_sample: int, samples: np.ndarray) -> bool:
        if start_sample != self._next_sample:
            raise ValueError(
                f"VAD input is not contiguous: expected {self._next_sample}, got {start_sample}"
            )
        if samples.ndim != 1:
            raise ValueError("VAD samples must be a one-dimensional array")
        if samples.dtype != np.int16:
            samples = samples.astype(np.int16, copy=False)
        if samples.size == 0:
            raise ValueError("VAD samples must not be empty")
        incoming_count = int(samples.size)
        if self._pending.size:
            samples = np.concatenate((self._pending, samples))
            start_sample = self._pending_start_sample
        self._pending = np.empty(0, dtype=np.int16)
        self._next_sample += incoming_count
        changed = False
        frame_samples = self.config.frame_samples
        offset = 0
        while offset + frame_samples <= samples.size:
            changed = (
                self._consume_frame(
                    start_sample + offset,
                    samples[offset : offset + frame_samples],
                )
                or changed
            )
            offset += frame_samples
        if offset < samples.size:
            self._pending = samples[offset:].copy()
            self._pending_start_sample = start_sample + offset
        return changed

    def flush(self) -> bool:
        """Classify the final partial frame, if any, without padding it."""
        if self._pending.size == 0:
            return False
        pending = self._pending
        start = self._pending_start_sample
        self._pending = np.empty(0, dtype=np.int16)
        return self._consume_frame(start, pending)

    def window(self, start_sample: int, end_sample: int) -> SpeechActivityWindow:
        if start_sample < 0 or end_sample <= start_sample:
            raise ValueError("activity window must be a positive half-open interval")
        voiced_samples = 0
        observed_samples = 0
        for frame in self._frames:
            overlap_start = max(start_sample, frame.start_sample)
            overlap_end = min(end_sample, frame.end_sample)
            if overlap_end <= overlap_start:
                continue
            overlap = overlap_end - overlap_start
            observed_samples += overlap
            if frame.voiced:
                voiced_samples += overlap
        sample_count = end_sample - start_sample
        # Unprocessed tail is silence until the next complete/flush frame. This
        # prevents a just-arrived partial buffer from manufacturing evidence.
        state: ActivityState = "speech" if voiced_samples > 0 else "no_speech"
        return SpeechActivityWindow(
            start_sample=start_sample,
            end_sample=end_sample,
            voiced_samples=voiced_samples,
            sample_count=sample_count if sample_count > observed_samples else observed_samples,
            state=state,
            eligible=(
                state == "speech"
                and voiced_samples >= self.config.min_voiced_samples
                and (
                    voiced_samples / sample_count
                    if sample_count > 0
                    else 0.0
                )
                >= self.config.min_voiced_ratio
            ),
        )

    def timeline(self, total_samples: int) -> tuple[SpeechActivitySegment, ...]:
        if total_samples < 0:
            raise ValueError("total_samples must be non-negative")
        runs = list(self._speech_runs)
        if self._active_start is not None:
            runs.append((self._active_start, total_samples))
        clipped: list[tuple[int, int]] = []
        for start, end in runs:
            start = max(0, min(start, total_samples))
            end = max(start, min(end, total_samples))
            if end > start:
                if clipped and start <= clipped[-1][1]:
                    clipped[-1] = (clipped[-1][0], max(clipped[-1][1], end))
                else:
                    clipped.append((start, end))
        result: list[SpeechActivitySegment] = []
        cursor = 0
        for start, end in clipped:
            if start > cursor:
                result.append(SpeechActivitySegment(cursor, start, "no_speech", 0, 0.0))
            voiced = self._voiced_between(start, end)
            result.append(
                SpeechActivitySegment(
                    start,
                    end,
                    "speech",
                    voiced,
                    voiced / (end - start) if end > start else 0.0,
                )
            )
            cursor = end
        if cursor < total_samples:
            result.append(SpeechActivitySegment(cursor, total_samples, "no_speech", 0, 0.0))
        if not result and total_samples > 0:
            result.append(SpeechActivitySegment(0, total_samples, "no_speech", 0, 0.0))
        return tuple(result)

    def latest_speech_span(
        self, total_samples: int, *, max_samples: int = 2 * SAMPLE_RATE_HZ
    ) -> tuple[int, int] | None:
        segments = [item for item in self.timeline(total_samples) if item.state == "speech"]
        for segment in reversed(segments):
            if segment.voiced_samples < self.config.min_voiced_samples:
                continue
            start = max(segment.start_sample, segment.end_sample - max_samples)
            if self.window(start, segment.end_sample).eligible_for_affect:
                return start, segment.end_sample
        return None

    def capabilities(self) -> dict[str, object]:
        return {
            "adapter_id": "energy-vad/1",
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "frame_ms": self.config.frame_ms,
            "speech_threshold_dbfs": self.config.speech_threshold_dbfs,
            "silence_threshold_dbfs": self.config.silence_threshold_dbfs,
            "start_frames": self.config.start_frames,
            "end_frames": self.config.end_frames,
            "pre_roll_ms": self.config.pre_roll_ms,
            "min_voiced_ms": self.config.min_voiced_ms,
            "min_voiced_ratio": self.config.min_voiced_ratio,
        }

    def _consume_frame(self, start_sample: int, samples: np.ndarray) -> bool:
        dbfs = _rms_dbfs(samples)
        voiced = dbfs >= self.config.speech_threshold_dbfs
        negative = dbfs <= self.config.silence_threshold_dbfs
        self._frames.append(
            _ActivityFrame(start_sample, start_sample + samples.size, voiced, dbfs)
        )
        previous = self._state
        if self._state == "no_speech":
            if voiced:
                self._positive_run += 1
            else:
                self._positive_run = 0
            if self._positive_run >= self.config.start_frames:
                self._state = "speech"
                self._active_start = max(
                    0, start_sample - self.config.pre_roll_samples
                )
                self._negative_run = 0
        else:
            if negative:
                self._negative_run += 1
            elif voiced:
                self._negative_run = 0
            # Intermediate-energy frames hold the current speech state. This
            # avoids chopping consonants and short pauses into separate turns.
            if self._negative_run >= self.config.end_frames:
                self._close_speech(start_sample + samples.size)
                self._state = "no_speech"
                self._positive_run = 0
                self._negative_run = 0
        return previous != self._state

    def _close_speech(self, end_sample: int) -> None:
        if self._active_start is not None and end_sample > self._active_start:
            self._speech_runs.append((self._active_start, end_sample))
        self._active_start = None

    def _voiced_between(self, start_sample: int, end_sample: int) -> int:
        return sum(
            min(end_sample, frame.end_sample) - max(start_sample, frame.start_sample)
            for frame in self._frames
            if frame.voiced
            and min(end_sample, frame.end_sample) > max(start_sample, frame.start_sample)
        )


def _rms_dbfs(samples: np.ndarray) -> float:
    if samples.size == 0:
        return -120.0
    values = samples.astype(np.float32, copy=False) / 32768.0
    rms = float(np.sqrt(np.mean(values * values)))
    if rms <= 1e-8:
        return -120.0
    return 20.0 * math.log10(rms)
