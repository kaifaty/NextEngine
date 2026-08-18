from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Mapping

if TYPE_CHECKING:
    import numpy as np


class AdapterError(RuntimeError):
    """Typed failure at a replaceable model boundary."""


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
