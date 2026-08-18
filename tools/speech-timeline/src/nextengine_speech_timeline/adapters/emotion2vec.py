from __future__ import annotations

import math
import time

from nextengine_emotion_probe.probe import EmotionProbe, ProbeError

from ..capabilities import AffectCapabilities, ModelLoadEvidence, WarmupEvidence
from .base import AdapterError, AffectObservation, AudioWindow


UPSTREAM_LABELS = {
    "生气/angry": "angry",
    "厌恶/disgusted": "disgusted",
    "恐惧/fearful": "fearful",
    "开心/happy": "happy",
    "中立/neutral": "neutral",
    "其他/other": "other",
    "难过/sad": "sad",
    "吃惊/surprised": "surprised",
    "<unk>": "unknown",
}
NORMALIZED_LABELS = frozenset(UPSTREAM_LABELS.values())


class Emotion2VecAffectAdapter:
    """Normalizes the pinned emotion2vec+ checkpoint behind a neutral API."""

    def __init__(self, probe: EmotionProbe) -> None:
        self._probe = probe
        self._warmup_count = 0

    @property
    def load_count(self) -> int:
        return self._probe.load_count

    def load(self) -> ModelLoadEvidence:
        before = self._probe.load_count
        started = time.perf_counter()
        self._probe.load()
        elapsed = round((time.perf_counter() - started) * 1000)
        return ModelLoadEvidence(
            elapsed_ms=elapsed if self._probe.load_count != before else 0,
            load_count=self._probe.load_count,
        )

    def warmup(self) -> WarmupEvidence:
        # The first real bounded observation is measured separately. Running a
        # fabricated waveform here could make readiness depend on model quirks.
        started = time.perf_counter()
        self.load()
        self._warmup_count += 1
        return WarmupEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1000),
            warmup_count=self._warmup_count,
        )

    def capabilities(self) -> AffectCapabilities:
        return AffectCapabilities(
            adapter_id="emotion2vec-plus/1",
            model_id=self._probe.model_id,
            model_revision=self._probe.model_revision,
            device=self._probe.device,
            sample_rate_hz=16_000,
            labels=tuple(sorted(NORMALIZED_LABELS)),
            semantics="uncalibrated_observed_expression",
        )

    def observe(self, window: AudioWindow) -> AffectObservation:
        if window.start_sample < 0 or window.end_sample <= window.start_sample:
            raise AdapterError("audio window must have a positive half-open interval")
        if window.end_sample - window.start_sample != window.samples.size:
            raise AdapterError("audio window interval does not match waveform length")
        try:
            result = self._probe.analyze_waveform(window.samples, window.sample_rate_hz)
        except ProbeError as error:
            raise AdapterError(str(error)) from error
        scores: dict[str, float] = {}
        for prediction in result["predictions"]:
            upstream = prediction["label"]
            normalized = UPSTREAM_LABELS.get(upstream)
            if normalized is None:
                raise AdapterError(f"unknown emotion2vec label: {upstream!r}")
            if normalized in scores:
                raise AdapterError(f"duplicate normalized emotion label: {normalized}")
            score = float(prediction["score"])
            if not math.isfinite(score):
                raise AdapterError(f"non-finite score for emotion label: {normalized}")
            scores[normalized] = score
        if scores.keys() != NORMALIZED_LABELS:
            missing = sorted(NORMALIZED_LABELS.difference(scores))
            extra = sorted(set(scores).difference(NORMALIZED_LABELS))
            raise AdapterError(f"emotion score vector mismatch: missing={missing}, extra={extra}")
        top_label = max(scores, key=lambda label: (scores[label], label))
        return AffectObservation(
            model_id=self._probe.model_id,
            model_revision=self._probe.model_revision,
            start_sample=window.start_sample,
            end_sample=window.end_sample,
            source_revision=window.source_revision,
            scores=scores,
            top_label=top_label,
            inference_elapsed_ms=int(result["inference_elapsed_ms"]),
        )
