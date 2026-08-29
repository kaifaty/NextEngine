"""Pinned emotion2vec+ loader and inference helpers."""

from __future__ import annotations

import contextlib
import hashlib
import math
import os
import sys
import time
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np

DEFAULT_MODEL_ID = "emotion2vec/emotion2vec_plus_base"
DEFAULT_MODEL_REVISION = "b318240bfe67db81a8c572ecb37ce9c3759b81c9"
MAX_AUDIO_BYTES = 64 * 1024 * 1024
IN_MEMORY_SAMPLE_RATE_HZ = 16_000
MAX_IN_MEMORY_SECONDS = 30
MAX_IN_MEMORY_SAMPLES = IN_MEMORY_SAMPLE_RATE_HZ * MAX_IN_MEMORY_SECONDS


class ProbeError(RuntimeError):
    """Stable user-facing failure raised by the isolated probe."""


@dataclass(frozen=True)
class AudioMetadata:
    content_hash: str
    byte_length: int
    sample_rate_hz: int | None
    channels: int | None
    duration_ms: int | None


def configure_cache(cache_dir: Path) -> None:
    resolved = cache_dir.expanduser().resolve()
    resolved.mkdir(parents=True, exist_ok=True)
    os.environ["HF_HOME"] = str(resolved / "huggingface")
    os.environ["MODELSCOPE_CACHE"] = str(resolved / "modelscope")


def select_device(requested: str) -> str:
    if requested not in {"auto", "cpu", "cuda"}:
        raise ProbeError(f"unsupported device: {requested}")
    if requested == "cpu":
        return "cpu"

    import torch

    if torch.cuda.is_available():
        return "cuda:0"
    if requested == "cuda":
        raise ProbeError("CUDA was requested but torch.cuda.is_available() is false")
    return "cpu"


def validate_audio(path: Path) -> Path:
    resolved = path.expanduser().resolve()
    if not resolved.is_file():
        raise ProbeError(f"audio file does not exist: {resolved}")
    size = resolved.stat().st_size
    if size == 0:
        raise ProbeError(f"audio file is empty: {resolved}")
    if size > MAX_AUDIO_BYTES:
        raise ProbeError(
            f"audio file exceeds the {MAX_AUDIO_BYTES}-byte probe limit: {size}"
        )
    return resolved


def audio_metadata(path: Path) -> AudioMetadata:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)

    sample_rate = None
    channels = None
    duration_ms = None
    try:
        with wave.open(str(path), "rb") as wav:
            sample_rate = wav.getframerate()
            channels = wav.getnchannels()
            frames = wav.getnframes()
            if sample_rate > 0:
                duration_ms = round(frames * 1000 / sample_rate)
    except (wave.Error, EOFError):
        pass

    return AudioMetadata(
        content_hash=f"sha256:{digest.hexdigest()}",
        byte_length=path.stat().st_size,
        sample_rate_hz=sample_rate,
        channels=channels,
        duration_ms=duration_ms,
    )


def normalize_predictions(raw_result: Any) -> list[dict[str, Any]]:
    result = raw_result
    if isinstance(result, list):
        if not result:
            raise ProbeError("model returned an empty result list")
        result = result[0]
    if not isinstance(result, dict):
        raise ProbeError(f"model returned unsupported result type: {type(result).__name__}")

    labels = result.get("labels")
    scores = result.get("scores")
    if not isinstance(labels, (list, tuple)) or not isinstance(scores, (list, tuple)):
        raise ProbeError("model result does not contain labels and scores lists")
    if len(labels) != len(scores) or not labels:
        raise ProbeError("model returned mismatched or empty labels and scores")

    predictions: list[dict[str, Any]] = []
    for label, score in zip(labels, scores, strict=True):
        try:
            numeric_score = float(score)
        except (TypeError, ValueError) as error:
            raise ProbeError(f"model returned a non-numeric score for {label!r}") from error
        if not math.isfinite(numeric_score):
            raise ProbeError(f"model returned a non-finite score for {label!r}")
        predictions.append({"label": str(label), "score": numeric_score})

    predictions.sort(key=lambda item: (-item["score"], item["label"]))
    return predictions


class EmotionProbe:
    def __init__(
        self,
        model_id: str,
        model_revision: str,
        cache_dir: Path,
        device: str,
        *,
        local_files_only: bool = False,
    ) -> None:
        configure_cache(cache_dir)
        self.model_id = model_id
        self.model_revision = model_revision
        self.cache_dir = cache_dir.expanduser().resolve()
        self.device = select_device(device)
        self.local_files_only = local_files_only
        self._model: Any = None
        self._resolved_model_path: Path | None = None
        self._load_count = 0

    @property
    def load_count(self) -> int:
        return self._load_count

    def load(self) -> None:
        if self._model is not None:
            return
        from funasr import AutoModel
        from huggingface_hub import snapshot_download

        with contextlib.redirect_stdout(sys.stderr):
            model_path = snapshot_download(
                repo_id=self.model_id,
                revision=self.model_revision,
                cache_dir=self.cache_dir / "huggingface" / "hub",
                local_files_only=self.local_files_only,
            )
            self._resolved_model_path = Path(model_path).resolve()
            self._model = AutoModel(
                model=str(self._resolved_model_path),
                hub="hf",
                device=self.device,
                disable_update=True,
                disable_pbar=True,
                log_level="WARNING",
            )
            self._load_count += 1

    def analyze(self, audio_path: Path) -> dict[str, Any]:
        path = validate_audio(audio_path)
        metadata = audio_metadata(path)
        return self._analyze_input(str(path), metadata)

    def analyze_waveform(
        self,
        samples: np.ndarray,
        sample_rate_hz: int = IN_MEMORY_SAMPLE_RATE_HZ,
    ) -> dict[str, Any]:
        waveform = validate_waveform(samples, sample_rate_hz)
        pcm_bytes = waveform.tobytes(order="C")
        metadata = AudioMetadata(
            content_hash=f"sha256:{hashlib.sha256(pcm_bytes).hexdigest()}",
            byte_length=len(pcm_bytes),
            sample_rate_hz=sample_rate_hz,
            channels=1,
            duration_ms=round(waveform.size * 1000 / sample_rate_hz),
        )
        return self._analyze_input(waveform, metadata)

    def _analyze_input(
        self,
        model_input: str | np.ndarray,
        metadata: AudioMetadata,
    ) -> dict[str, Any]:
        self.load()
        started = time.perf_counter()
        try:
            with contextlib.redirect_stdout(sys.stderr):
                raw_result = self._model.generate(
                    input=model_input,
                    granularity="utterance",
                    extract_embedding=False,
                )
        except Exception as error:
            raise ProbeError(f"model inference failed: {type(error).__name__}: {error}") from error
        elapsed_ms = round((time.perf_counter() - started) * 1000)
        predictions = normalize_predictions(raw_result)
        return {
            "schema_version": 1,
            "model": {
                "id": self.model_id,
                "revision": self.model_revision,
                "hub": "huggingface",
                "device": self.device,
            },
            "audio": {
                "content_hash": metadata.content_hash,
                "byte_length": metadata.byte_length,
                "sample_rate_hz": metadata.sample_rate_hz,
                "channels": metadata.channels,
                "duration_ms": metadata.duration_ms,
            },
            "top_label": predictions[0]["label"],
            "predictions": predictions,
            "inference_elapsed_ms": elapsed_ms,
            "interpretation": "scores_are_uncalibrated_observed_expression",
        }


def validate_waveform(samples: np.ndarray, sample_rate_hz: int) -> np.ndarray:
    if sample_rate_hz != IN_MEMORY_SAMPLE_RATE_HZ:
        raise ProbeError(
            f"in-memory waveform must be {IN_MEMORY_SAMPLE_RATE_HZ} Hz, got {sample_rate_hz}"
        )
    if not isinstance(samples, np.ndarray):
        raise ProbeError("in-memory waveform must be a numpy.ndarray")
    if samples.dtype != np.float32:
        raise ProbeError(f"in-memory waveform must have dtype float32, got {samples.dtype}")
    if samples.ndim != 1:
        raise ProbeError(f"in-memory waveform must be mono rank-1, got rank {samples.ndim}")
    if samples.size == 0:
        raise ProbeError("in-memory waveform must not be empty")
    if samples.size > MAX_IN_MEMORY_SAMPLES:
        raise ProbeError(
            f"in-memory waveform exceeds {MAX_IN_MEMORY_SECONDS}s limit: {samples.size} samples"
        )
    if not samples.flags.c_contiguous:
        raise ProbeError("in-memory waveform must be C-contiguous")
    if not np.isfinite(samples).all():
        raise ProbeError("in-memory waveform contains non-finite samples")
    if np.max(np.abs(samples)) > 1.0:
        raise ProbeError("in-memory waveform samples must be within [-1.0, 1.0]")
    return samples
