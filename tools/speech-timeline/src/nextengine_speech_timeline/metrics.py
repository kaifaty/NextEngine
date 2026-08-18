from __future__ import annotations

from dataclasses import asdict, dataclass


@dataclass(frozen=True)
class ModelJobMetric:
    session_generation: int
    job_kind: str
    audio_start_sample: int
    audio_end_sample: int
    queue_wait_ms: int
    inference_ms: int

    def as_dict(self) -> dict[str, int | str]:
        return asdict(self)
