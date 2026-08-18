"""Pinned emotion2vec+ loader and inference helpers."""

from __future__ import annotations

import contextlib
import hashlib
import os
import sys
import time
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Any

DEFAULT_MODEL_ID = "emotion2vec/emotion2vec_plus_base"
DEFAULT_MODEL_REVISION = "b318240bfe67db81a8c572ecb37ce9c3759b81c9"
MAX_AUDIO_BYTES = 64 * 1024 * 1024


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
    os.environ.setdefault("HF_HOME", str(resolved / "huggingface"))
    os.environ.setdefault("MODELSCOPE_CACHE", str(resolved / "modelscope"))


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
    ) -> None:
        configure_cache(cache_dir)
        self.model_id = model_id
        self.model_revision = model_revision
        self.cache_dir = cache_dir.expanduser().resolve()
        self.device = select_device(device)
        self._model: Any = None
        self._resolved_model_path: Path | None = None

    def load(self) -> None:
        if self._model is not None:
            return
        from funasr import AutoModel
        from huggingface_hub import snapshot_download

        with contextlib.redirect_stdout(sys.stderr):
            model_path = snapshot_download(
                repo_id=self.model_id,
                revision=self.model_revision,
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

    def analyze(self, audio_path: Path) -> dict[str, Any]:
        path = validate_audio(audio_path)
        metadata = audio_metadata(path)
        self.load()
        started = time.perf_counter()
        try:
            with contextlib.redirect_stdout(sys.stderr):
                raw_result = self._model.generate(
                    input=str(path),
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
