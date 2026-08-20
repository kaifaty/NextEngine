from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Mapping, Protocol, Sequence

if TYPE_CHECKING:
    import numpy as np


class AdapterError(RuntimeError):
    """Typed failure at a replaceable model boundary."""


@dataclass(frozen=True)
class TranscriberConfig:
    language: str | None = None


@dataclass(frozen=True)
class TranscriptRevision:
    revision: int
    full_text: str
    committed_text: str
    tentative_text: str
    final: bool
    timing_precision: str = "utterance"


class TranscriberSession(Protocol):
    def push_pcm(self, samples: Sequence[float]) -> TranscriptRevision | None: ...

    def finish(self) -> TranscriptRevision: ...

    def cancel(self) -> None: ...

    def close(self) -> None: ...


class Transcriber(Protocol):
    def load(self) -> Mapping[str, object]: ...

    def warmup(self) -> Mapping[str, object]: ...

    def capabilities(self) -> Mapping[str, object]: ...

    def start(self, config: TranscriberConfig | None = None) -> TranscriberSession: ...

    def close(self) -> None: ...


class AudioPreprocessor(Protocol):
    """Resident, stateful PCM preprocessor on the service's 16 kHz clock.

    It is intentionally separate from ASR and vocal-affect adapters.  A
    concrete implementation may feed a cleaned stream to ASR, but raw PCM
    remains available to VAD, affect and diagnostics until an explicit A/B
    result proves that enhancement preserves their acoustic evidence.
    """

    def load(self) -> Mapping[str, object]: ...

    def warmup(self) -> Mapping[str, object]: ...

    def capabilities(self) -> Mapping[str, object]: ...

    def reset(
        self,
        route: str,
        *,
        noise_floor_dbfs: float | None = None,
    ) -> None: ...

    def process_pcm(self, pcm: bytes) -> bytes: ...

    def flush(self) -> bytes: ...

    def close(self) -> None: ...


@dataclass(frozen=True)
class AudioWindow:
    samples: np.ndarray
    sample_rate_hz: int
    start_sample: int
    end_sample: int
    source_revision: int


@dataclass(frozen=True)
class AffectObservation:
    model_id: str
    model_revision: str
    start_sample: int
    end_sample: int
    source_revision: int
    scores: Mapping[str, float]
    top_label: str
    inference_elapsed_ms: int
    semantics: str = "uncalibrated_observed_expression"
    activity: str = "speech"
    voiced_ratio: float = 1.0
    evidence_samples: int = 0
