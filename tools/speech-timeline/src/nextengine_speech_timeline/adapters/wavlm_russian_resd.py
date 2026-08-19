"""Local-only adapter for the pinned Russian RESD WavLM classifier.

The Hugging Face repository is a model artifact, not executable application
code: loading is constrained to its exact revision, to files already present
in the operator-provided cache, and to the stock Transformers WavLM classes.
"""

from __future__ import annotations

import hashlib
import math
from pathlib import Path
import time
from types import MappingProxyType
from typing import Any, Mapping

import numpy as np

from nextengine_emotion_probe.probe import ProbeError, select_device

from ..capabilities import AffectCapabilities, ModelLoadEvidence, WarmupEvidence
from .base import AdapterError, AffectObservation, AudioWindow


ADAPTER_ID = "transformers-wavlm-russian-ser/1"
GENERIC_ADAPTER_ID = "transformers-wavlm-audio-classification/1"
GENERIC_AUDIO_ADAPTER_ID = "transformers-audio-classification/1"
MODEL_TYPE = "wavlm"
WAVLM_MODEL_TYPES = frozenset({MODEL_TYPE})
GENERIC_AUDIO_MODEL_TYPES = frozenset({"wavlm", "wav2vec2"})
WEIGHTS_FILENAME = "model.safetensors"
SAMPLE_RATE_HZ = 16_000
MAX_INPUT_SECONDS = 12
MAX_INPUT_SAMPLES = SAMPLE_RATE_HZ * MAX_INPUT_SECONDS
UPSTREAM_LABELS = MappingProxyType(
    {
        "anger": "angry",
        "disgust": "disgusted",
        "enthusiasm": "enthusiasm",
        "fear": "fearful",
        "happiness": "happy",
        "neutral": "neutral",
        "sadness": "sad",
    }
)
NORMALIZED_LABELS = frozenset(UPSTREAM_LABELS.values())
DEFAULT_LABEL_MAP = UPSTREAM_LABELS


class WavlmAffectAdapter:
    """Normalizes a pinned stock-Transformers WavLM classification head.

    The profile owns the mapping from its immutable upstream head labels to
    engine timeline labels. This lets an operator trial another standard WavLM
    checkpoint without accepting repository Python or weakening the existing
    RESD-specific adapter contract.
    """

    def __init__(
        self,
        model_id: str,
        model_revision: str,
        cache_dir: Path,
        device: str,
        weights_sha256: str,
        label_map: Mapping[str, str] = DEFAULT_LABEL_MAP,
        adapter_id: str = ADAPTER_ID,
        model_types: frozenset[str] = WAVLM_MODEL_TYPES,
    ) -> None:
        self._label_map = _validate_label_map(label_map)
        self._adapter_id = adapter_id
        self._model_types = _validate_model_types(model_types)
        self._model_id = model_id
        self._model_revision = model_revision
        self._cache_dir = cache_dir.expanduser().resolve()
        self._requested_device = device
        self._weights_sha256 = weights_sha256
        self._device: str | None = None
        self._feature_extractor: Any | None = None
        self._model: Any | None = None
        self._upstream_labels: tuple[str, ...] | None = None
        self._warmup_count = 0
        self._load_count = 0

    @property
    def load_count(self) -> int:
        return self._load_count

    def load(self) -> ModelLoadEvidence:
        if self._model is not None:
            return ModelLoadEvidence(elapsed_ms=0, load_count=self._load_count)
        started = time.perf_counter()
        try:
            snapshot = self._snapshot_directory()
            self._verify_snapshot(snapshot)
            from transformers import AutoConfig, AutoFeatureExtractor, AutoModelForAudioClassification

            config = AutoConfig.from_pretrained(
                snapshot,
                local_files_only=True,
                trust_remote_code=False,
            )
            upstream_labels = _validate_config(config, self._label_map, self._model_types)
            feature_extractor = AutoFeatureExtractor.from_pretrained(
                snapshot,
                local_files_only=True,
                trust_remote_code=False,
            )
            model = AutoModelForAudioClassification.from_pretrained(
                snapshot,
                local_files_only=True,
                trust_remote_code=False,
            )
            runtime_device = select_device(self._requested_device)
            model.to(runtime_device)
            model.eval()
        except (AdapterError, OSError, ValueError, ProbeError, RuntimeError) as error:
            raise AdapterError(f"cannot load pinned WavLM affect model: {error}") from error
        self._device = runtime_device
        self._feature_extractor = feature_extractor
        self._model = model
        self._upstream_labels = upstream_labels
        self._load_count += 1
        return ModelLoadEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1000),
            load_count=self._load_count,
        )

    def warmup(self) -> WarmupEvidence:
        # As with emotion2vec, readiness establishes a resident model but does
        # not fabricate an observation that could hide model-specific behavior.
        started = time.perf_counter()
        self.load()
        self._warmup_count += 1
        return WarmupEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1000),
            warmup_count=self._warmup_count,
        )

    def capabilities(self) -> AffectCapabilities:
        return AffectCapabilities(
            adapter_id=self._adapter_id,
            model_id=self._model_id,
            model_revision=self._model_revision,
            device=self._device or self._requested_device,
            sample_rate_hz=SAMPLE_RATE_HZ,
            labels=tuple(sorted(set(self._label_map.values()))),
            semantics="uncalibrated_observed_expression",
        )

    def observe(self, window: AudioWindow) -> AffectObservation:
        _validate_window(window)
        self.load()
        assert self._model is not None
        assert self._feature_extractor is not None
        assert self._device is not None
        upstream_labels = self._upstream_labels or tuple(self._label_map)
        try:
            import torch

            encoded = self._feature_extractor(
                np.ascontiguousarray(window.samples, dtype=np.float32),
                sampling_rate=SAMPLE_RATE_HZ,
                return_tensors="pt",
                padding=True,
            )
            model_inputs = {key: value.to(self._device) for key, value in encoded.items()}
            if self._device.startswith("cuda"):
                torch.cuda.synchronize()
            started = time.perf_counter()
            with torch.inference_mode():
                logits = self._model(**model_inputs).logits
                probabilities = torch.softmax(logits, dim=-1)[0].detach().cpu().numpy()
            if self._device.startswith("cuda"):
                torch.cuda.synchronize()
            elapsed_ms = round((time.perf_counter() - started) * 1000)
        except (RuntimeError, ValueError, TypeError) as error:
            raise AdapterError(f"WavLM affect inference failed: {error}") from error
        scores = normalize_probabilities(probabilities, upstream_labels, self._label_map)
        top_label = max(scores, key=lambda label: (scores[label], label))
        return AffectObservation(
            model_id=self._model_id,
            model_revision=self._model_revision,
            start_sample=window.start_sample,
            end_sample=window.end_sample,
            source_revision=window.source_revision,
            scores=scores,
            top_label=top_label,
            inference_elapsed_ms=elapsed_ms,
            activity="speech",
            voiced_ratio=1.0,
            evidence_samples=window.end_sample - window.start_sample,
        )

    def _snapshot_directory(self) -> Path:
        model_directory = f"models--{self._model_id.replace('/', '--')}"
        snapshot = self._cache_dir / model_directory / "snapshots" / self._model_revision
        if not snapshot.is_dir():
            raise AdapterError("the pinned WavLM snapshot is not available in the configured local cache")
        return snapshot.resolve()

    def _verify_snapshot(self, snapshot: Path) -> None:
        try:
            snapshot.relative_to(self._cache_dir)
        except ValueError as error:
            raise AdapterError("WavLM snapshot resolves outside the configured cache") from error
        for filename in ("config.json", "preprocessor_config.json", WEIGHTS_FILENAME):
            artifact = snapshot / filename
            if not artifact.is_file():
                raise AdapterError(f"pinned WavLM snapshot lacks {filename}")
            try:
                artifact.resolve().relative_to(self._cache_dir)
            except ValueError as error:
                raise AdapterError(f"WavLM {filename} resolves outside the configured cache") from error
        resolved_weights = (snapshot / WEIGHTS_FILENAME).resolve()
        actual_hash = _sha256_file(resolved_weights)
        if actual_hash != self._weights_sha256:
            raise AdapterError("WavLM model.safetensors SHA-256 does not match the profile")


def _validate_label_map(label_map: Mapping[str, str]) -> dict[str, str]:
    if not isinstance(label_map, Mapping) or not 1 <= len(label_map) <= 32:
        raise AdapterError("WavLM label map must contain between 1 and 32 labels")
    normalized: dict[str, str] = {}
    for source, target in label_map.items():
        if not isinstance(source, str) or not source or not isinstance(target, str) or not target:
            raise AdapterError("WavLM label map must use non-empty string labels")
        if source in normalized or target in normalized.values():
            raise AdapterError("WavLM label map must be one-to-one")
        normalized[source] = target
    return normalized


def _validate_model_types(model_types: frozenset[str]) -> frozenset[str]:
    if (
        not isinstance(model_types, frozenset)
        or not model_types
        or not model_types.issubset(GENERIC_AUDIO_MODEL_TYPES)
    ):
        raise AdapterError("unsupported Transformers audio-classification model family")
    return model_types


def _validate_config(
    config: Any,
    label_map: Mapping[str, str] = DEFAULT_LABEL_MAP,
    model_types: frozenset[str] = WAVLM_MODEL_TYPES,
) -> tuple[str, ...]:
    supported_model_types = _validate_model_types(model_types)
    if getattr(config, "model_type", None) not in supported_model_types:
        raise AdapterError(f"expected model_type in {sorted(supported_model_types)!r}")
    raw_labels = getattr(config, "id2label", None)
    if not isinstance(raw_labels, Mapping):
        raise AdapterError("WavLM config has no id2label mapping")
    labels: dict[int, str] = {}
    for index, label in raw_labels.items():
        try:
            numeric_index = int(index)
        except (TypeError, ValueError) as error:
            raise AdapterError("WavLM config label index is invalid") from error
        if not isinstance(label, str):
            raise AdapterError("WavLM config label is invalid")
        labels[numeric_index] = label
    expected = set(_validate_label_map(label_map))
    expected_indices = set(range(len(expected)))
    if set(labels) != expected_indices or set(labels.values()) != expected:
        raise AdapterError("WavLM config labels do not match the profile label map")
    return tuple(labels[index] for index in range(len(labels)))


def _validate_window(window: AudioWindow) -> None:
    if window.sample_rate_hz != SAMPLE_RATE_HZ:
        raise AdapterError(f"WavLM requires {SAMPLE_RATE_HZ} Hz audio")
    if window.start_sample < 0 or window.end_sample <= window.start_sample:
        raise AdapterError("audio window must have a positive half-open interval")
    if window.end_sample - window.start_sample != window.samples.size:
        raise AdapterError("audio window interval does not match waveform length")
    if window.samples.size > MAX_INPUT_SAMPLES:
        raise AdapterError(f"WavLM input exceeds {MAX_INPUT_SECONDS} seconds")
    if not np.isfinite(window.samples).all():
        raise AdapterError("audio window contains non-finite samples")


def normalize_probabilities(
    probabilities: np.ndarray,
    upstream_labels: tuple[str, ...] = tuple(UPSTREAM_LABELS),
    label_map: Mapping[str, str] = DEFAULT_LABEL_MAP,
) -> dict[str, float]:
    validated_map = _validate_label_map(label_map)
    values = np.asarray(probabilities, dtype=np.float64)
    if values.shape != (len(upstream_labels),):
        raise AdapterError("WavLM score vector has an unexpected shape")
    if set(upstream_labels) != set(validated_map) or len(set(upstream_labels)) != len(upstream_labels):
        raise AdapterError("WavLM score vector labels do not match the profile label map")
    if not np.isfinite(values).all():
        raise AdapterError("WavLM score vector contains non-finite values")
    scores = {
        validated_map[label]: float(values[index])
        for index, label in enumerate(upstream_labels)
    }
    if set(scores) != set(validated_map.values()):
        raise AdapterError("WavLM score vector does not cover every supported label")
    total = sum(scores.values())
    if not math.isfinite(total) or not 0.999 <= total <= 1.001:
        raise AdapterError("WavLM score vector is not a probability distribution")
    return scores


# The old public class name and adapter id retain the frozen RESD contract.
WavlmRussianResdAffectAdapter = WavlmAffectAdapter


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"
