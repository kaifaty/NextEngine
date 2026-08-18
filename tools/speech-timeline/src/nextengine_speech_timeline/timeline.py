from __future__ import annotations

from dataclasses import dataclass
import math
from collections.abc import Callable
from typing import Mapping

from .activity import SpeechActivitySegment
from .adapters.base import AffectObservation


SAMPLE_RATE_HZ = 16_000
FAST_WINDOW_SAMPLES = SAMPLE_RATE_HZ
STABLE_WINDOW_SAMPLES = 2 * SAMPLE_RATE_HZ
HOP_SAMPLES = SAMPLE_RATE_HZ // 4


@dataclass(frozen=True)
class AffectWindowRequest:
    mode: str
    start_sample: int
    end_sample: int
    final: bool = False


class AffectCadence:
    def __init__(self) -> None:
        self._next_end_sample = FAST_WINDOW_SAMPLES

    def advance(
        self,
        audio_end_sample: int,
        *,
        eligible: Callable[[int, int], bool] | None = None,
    ) -> list[AffectWindowRequest]:
        requests = []
        while self._next_end_sample <= audio_end_sample:
            end = self._next_end_sample
            if end < STABLE_WINDOW_SAMPLES:
                mode = "fast"
                length = FAST_WINDOW_SAMPLES
            else:
                mode = "stable"
                length = STABLE_WINDOW_SAMPLES
            start = end - length
            if eligible is None or eligible(start, end):
                requests.append(AffectWindowRequest(mode, start, end))
            self._next_end_sample += HOP_SAMPLES
        return requests

    @staticmethod
    def final(audio_end_sample: int, *, start_sample: int = 0) -> AffectWindowRequest:
        if audio_end_sample <= start_sample or start_sample < 0:
            raise ValueError("final affect window requires audio")
        return AffectWindowRequest("final", start_sample, audio_end_sample, final=True)


@dataclass(frozen=True)
class RawAffectObservation:
    observation_id: int
    start_sample: int
    end_sample: int
    scores: Mapping[str, float]
    top_label: str
    activity: str = "speech"
    voiced_ratio: float = 1.0
    evidence_samples: int = 0


@dataclass(frozen=True)
class AffectSegment:
    start_sample: int
    end_sample: int
    label: str
    score: float


@dataclass(frozen=True)
class TranscriptTrack:
    revision: int = 0
    text: str = ""
    stable_prefix: str = ""
    final: bool = False
    timing_precision: str = "utterance"


@dataclass(frozen=True)
class AffectTrack:
    revision: int = 0
    replace_from_sample: int = 0
    raw_observations: tuple[RawAffectObservation, ...] = ()
    segments: tuple[AffectSegment, ...] = ()
    speech_activity: tuple[SpeechActivitySegment, ...] = ()


@dataclass(frozen=True)
class FusionTrack:
    revision: int = 0
    alignment_grade: str = "utterance"
    spans: tuple[object, ...] = ()
    observed_vocal_expression: str = "unknown"


@dataclass(frozen=True)
class TimelineSnapshot:
    revision: int
    transcript: TranscriptTrack
    vocal_affect: AffectTrack
    fusion: FusionTrack


@dataclass(frozen=True)
class SmoothingPolicy:
    score_threshold: float = 0.35
    switch_margin: float = 0.05
    minimum_dwell_samples: int = HOP_SAMPLES * 2


class SpeechTimeline:
    """Independently revisioned transcript, affect, and utterance fusion tracks."""

    def __init__(self, smoothing: SmoothingPolicy | None = None) -> None:
        self.smoothing = smoothing or SmoothingPolicy()
        self.transcript = TranscriptTrack()
        self.affect = AffectTrack()
        self.fusion = FusionTrack()
        self._revision = 0
        self._observations: list[RawAffectObservation] = []
        self._segments: list[AffectSegment] = []
        self._speech_activity: tuple[SpeechActivitySegment, ...] = ()
        self._candidate: str | None = None
        self._candidate_hops = 0

    def apply_transcript(
        self,
        *,
        text: str,
        stable_prefix: str,
        final: bool,
        timing_precision: str,
    ) -> TimelineSnapshot:
        if timing_precision != "utterance":
            raise ValueError("Phase 1 accepts only utterance-grade timing")
        self.transcript = TranscriptTrack(
            revision=self.transcript.revision + 1,
            text=text,
            stable_prefix=stable_prefix,
            final=final,
            timing_precision=timing_precision,
        )
        self._rebuild_fusion()
        return self.snapshot()

    def apply_affect(self, observation: AffectObservation) -> TimelineSnapshot:
        scores = {label: float(score) for label, score in observation.scores.items()}
        if not scores or any(not math.isfinite(score) for score in scores.values()):
            raise ValueError("affect observation must contain finite scores")
        raw = RawAffectObservation(
            observation_id=len(self._observations) + 1,
            start_sample=observation.start_sample,
            end_sample=observation.end_sample,
            scores=scores,
            top_label=observation.top_label,
            activity=observation.activity,
            voiced_ratio=observation.voiced_ratio,
            evidence_samples=observation.evidence_samples,
        )
        self._observations.append(raw)
        self._smooth(raw)
        self.affect = AffectTrack(
            revision=self.affect.revision + 1,
            replace_from_sample=raw.start_sample,
            raw_observations=tuple(self._observations),
            segments=tuple(self._segments),
            speech_activity=self._speech_activity,
        )
        self._rebuild_fusion()
        return self.snapshot()

    def apply_activity(
        self, segments: tuple[SpeechActivitySegment, ...]
    ) -> TimelineSnapshot:
        if segments == self._speech_activity:
            return self.snapshot()
        self._speech_activity = segments
        self.affect = AffectTrack(
            revision=self.affect.revision + 1,
            replace_from_sample=segments[0].start_sample if segments else 0,
            raw_observations=tuple(self._observations),
            segments=tuple(self._segments),
            speech_activity=segments,
        )
        self._rebuild_fusion()
        return self.snapshot()

    def snapshot(self) -> TimelineSnapshot:
        self._revision += 1
        return TimelineSnapshot(self._revision, self.transcript, self.affect, self.fusion)

    def utterance_final(self) -> dict[str, object]:
        if not self.transcript.final:
            raise ValueError("transcript is not final")
        return {
            "text": self.transcript.text,
            "timing_precision": "utterance",
            "observed_vocal_expression": self.fusion.observed_vocal_expression,
            "alignment_grade": "utterance",
            "spans": [],
        }

    def _smooth(self, observation: RawAffectObservation) -> None:
        recent = sorted(self._observations, key=lambda item: item.end_sample)[-4:]
        labels = set().union(*(item.scores.keys() for item in recent))
        weights = {
            id(item): max(1, item.evidence_samples or item.end_sample - item.start_sample)
            for item in recent
        }
        total_weight = sum(weights.values())
        totals = {
            label: sum(item.scores.get(label, 0.0) * weights[id(item)] for item in recent)
            / total_weight
            for label in labels
        }
        candidate = max(totals, key=lambda label: (totals[label], label))
        sorted_scores = sorted(totals.values(), reverse=True)
        margin = sorted_scores[0] - (sorted_scores[1] if len(sorted_scores) > 1 else 0.0)
        if candidate == self._candidate:
            self._candidate_hops += 1
        else:
            self._candidate = candidate
            self._candidate_hops = 1
        admitted = (
            self._candidate_hops >= 2
            and totals[candidate] >= self.smoothing.score_threshold
            and margin >= self.smoothing.switch_margin
        )
        label = candidate if admitted else "unknown"
        score = totals[candidate] if admitted else 0.0
        replace_from = observation.start_sample
        while self._segments and self._segments[-1].start_sample >= replace_from:
            self._segments.pop()
        if self._segments and self._segments[-1].end_sample > replace_from:
            previous = self._segments[-1]
            self._segments[-1] = AffectSegment(
                previous.start_sample, replace_from, previous.label, previous.score
            )
        current = self._segments[-1] if self._segments else None
        if (
            current is not None
            and current.label != label
            and current.end_sample - current.start_sample < self.smoothing.minimum_dwell_samples
        ):
            label = current.label
            score = current.score
        if current is not None and current.label == label and current.end_sample == replace_from:
            self._segments[-1] = AffectSegment(
                current.start_sample, observation.end_sample, label, score
            )
        else:
            self._segments.append(
                AffectSegment(replace_from, observation.end_sample, label, score)
            )

    def _rebuild_fusion(self) -> None:
        expression = self._segments[-1].label if self._segments else "unknown"
        self.fusion = FusionTrack(
            revision=self.fusion.revision + 1,
            alignment_grade="utterance",
            spans=(),
            observed_vocal_expression=expression,
        )
