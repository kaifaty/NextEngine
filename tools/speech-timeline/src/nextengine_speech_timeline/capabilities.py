from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ModelLoadEvidence:
    elapsed_ms: int
    load_count: int


@dataclass(frozen=True)
class WarmupEvidence:
    elapsed_ms: int
    warmup_count: int


@dataclass(frozen=True)
class TranscriberCapabilities:
    adapter_id: str
    model_id: str
    runtime_id: str
    backend: str
    sample_rate_hz: int
    encoding: str
    channels: int
    supports_streaming: bool
    timing_precision: str
    resolution_samples: int | None
    supported_delay_ms: tuple[int, ...]
    configured_delay_ms: int
    partial_decode_interval_ms: int
    max_audio_duration_ms: int | None = None
    streaming_mode: str = "unspecified"
    streaming_window_ms: int | None = None
    streaming_left_context_ms: int | None = None


@dataclass(frozen=True)
class AffectCapabilities:
    adapter_id: str
    model_id: str
    model_revision: str
    device: str
    sample_rate_hz: int
    labels: tuple[str, ...]
    semantics: str
