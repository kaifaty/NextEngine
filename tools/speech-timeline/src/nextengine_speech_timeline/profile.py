from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import re
import subprocess
from typing import Any

from nextengine_emotion_probe.probe import EmotionProbe

from .adapters.emotion2vec import Emotion2VecAffectAdapter
from .adapters.voxtral_transcribe_cpp import VoxtralTranscriberAdapter
from .adapters.wavlm_russian_resd import (
    ADAPTER_ID as WAVLM_RUSSIAN_RESD_ADAPTER_ID,
    DEFAULT_LABEL_MAP,
    GENERIC_ADAPTER_ID as WAVLM_AUDIO_CLASSIFICATION_ADAPTER_ID,
    WEIGHTS_FILENAME as WAVLM_WEIGHTS_FILENAME,
    WavlmAffectAdapter,
    WavlmRussianResdAffectAdapter,
)
from .session import SessionBounds


REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
EMOTION2VEC_ADAPTER_ID = "emotion2vec-plus/1"
WAVLM_ADAPTER_IDS = {WAVLM_RUSSIAN_RESD_ADAPTER_ID, WAVLM_AUDIO_CLASSIFICATION_ADAPTER_ID}
SUPPORTED_EMOTION_ADAPTERS = {EMOTION2VEC_ADAPTER_ID, *WAVLM_ADAPTER_IDS}


class ProfileError(RuntimeError):
    pass


@dataclass(frozen=True)
class VoxtralProfile:
    model_path: Path
    model_size_bytes: int
    model_sha256: str
    transcribe_root: Path
    library: Path
    runtime_revision: str
    backend: str
    delay_ms: int
    partial_decode_interval_ms: int


@dataclass(frozen=True)
class EmotionProfile:
    adapter_id: str
    model_id: str
    model_revision: str
    cache_dir: Path
    device: str
    classification: str
    weights_sha256: str | None
    label_map: tuple[tuple[str, str], ...] | None


@dataclass(frozen=True)
class ServiceProfile:
    port: int
    ready_file: Path
    bounds: SessionBounds


@dataclass(frozen=True)
class SpeechTimelineProfile:
    voxtral: VoxtralProfile
    emotion: EmotionProfile
    service: ServiceProfile


def load_profile(path: Path) -> SpeechTimelineProfile:
    resolved = path.expanduser().resolve()
    _require_external(resolved, "profile")
    try:
        raw = resolved.read_bytes()
    except OSError as error:
        raise ProfileError(f"cannot read profile: {error}") from error
    if len(raw) > 64 * 1024:
        raise ProfileError("profile exceeds 65536 bytes")
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ProfileError(f"invalid profile JSON: {error}") from error
    root = _object(value, {"schema_version", "voxtral", "emotion", "service"}, "profile")
    if root["schema_version"] != 1:
        raise ProfileError("profile schema_version must be 1")
    voxtral = _object_with_optional(
        root["voxtral"],
        {
            "model_path",
            "model_size_bytes",
            "model_sha256",
            "transcribe_root",
            "library",
            "runtime_revision",
            "backend",
            "delay_ms",
        },
        {"partial_decode_interval_ms"},
        "voxtral",
    )
    emotion = _object_with_optional(
        root["emotion"],
        {"model_id", "model_revision", "cache_dir", "device", "classification"},
        {"adapter_id", "weights_sha256", "label_map"},
        "emotion",
    )
    service = _object(
        root["service"],
        {"port", "ready_file", "max_frame_bytes", "max_turn_bytes"},
        "service",
    )
    adapter_id = _choice(
        emotion.get("adapter_id", EMOTION2VEC_ADAPTER_ID),
        "emotion.adapter_id",
        SUPPORTED_EMOTION_ADAPTERS,
    )
    profile = SpeechTimelineProfile(
        voxtral=VoxtralProfile(
            model_path=_path(voxtral["model_path"], "voxtral.model_path"),
            model_size_bytes=_positive_int(voxtral["model_size_bytes"], "model_size_bytes"),
            model_sha256=_sha256(voxtral["model_sha256"], "model_sha256"),
            transcribe_root=_path(voxtral["transcribe_root"], "transcribe_root"),
            library=_path(voxtral["library"], "library"),
            runtime_revision=_string(voxtral["runtime_revision"], "runtime_revision", 128),
            backend=_choice(voxtral["backend"], "backend", {"auto", "cpu", "cuda", "vulkan"}),
            delay_ms=_positive_int(voxtral["delay_ms"], "delay_ms"),
            partial_decode_interval_ms=_positive_int(
                voxtral.get("partial_decode_interval_ms", 240),
                "partial_decode_interval_ms",
            ),
        ),
        emotion=EmotionProfile(
            adapter_id=adapter_id,
            model_id=_string(emotion["model_id"], "model_id", 256),
            model_revision=_string(emotion["model_revision"], "model_revision", 128),
            cache_dir=_path(emotion["cache_dir"], "cache_dir"),
            device=_choice(emotion["device"], "device", {"auto", "cpu", "cuda"}),
            classification=_choice(
                emotion["classification"],
                "classification",
                {"unclassified_local_only", "classified_local_only"},
            ),
            weights_sha256=(
                _sha256(emotion["weights_sha256"], "emotion.weights_sha256")
                if "weights_sha256" in emotion
                else None
            ),
            label_map=(
                _label_map(emotion["label_map"], "emotion.label_map")
                if "label_map" in emotion
                else tuple(DEFAULT_LABEL_MAP.items())
                if adapter_id == WAVLM_RUSSIAN_RESD_ADAPTER_ID
                else None
            ),
        ),
        service=ServiceProfile(
            port=_port(service["port"]),
            ready_file=_path(service["ready_file"], "ready_file"),
            bounds=SessionBounds(
                max_frame_bytes=_positive_int(service["max_frame_bytes"], "max_frame_bytes"),
                max_turn_bytes=_positive_int(service["max_turn_bytes"], "max_turn_bytes"),
            ),
        ),
    )
    validate_profile_artifacts(profile)
    return profile


def validate_profile_artifacts(profile: SpeechTimelineProfile) -> None:
    model = profile.voxtral.model_path
    _require_external(model, "Voxtral model")
    _require_external(profile.voxtral.transcribe_root, "transcribe.cpp checkout")
    _require_external(profile.voxtral.library, "transcribe.cpp library")
    if not model.is_file():
        raise ProfileError(f"Voxtral model does not exist: {model}")
    actual_size = model.stat().st_size
    if actual_size != profile.voxtral.model_size_bytes:
        raise ProfileError(
            f"Voxtral model size mismatch: expected {profile.voxtral.model_size_bytes}, got {actual_size}"
        )
    digest = hashlib.sha256()
    with model.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    actual_hash = f"sha256:{digest.hexdigest()}"
    if actual_hash != profile.voxtral.model_sha256:
        raise ProfileError("Voxtral model SHA-256 does not match the profile")
    if not profile.voxtral.library.is_file():
        raise ProfileError(f"transcribe.cpp library does not exist: {profile.voxtral.library}")
    if not profile.emotion.cache_dir.is_dir():
        raise ProfileError(f"emotion cache directory does not exist: {profile.emotion.cache_dir}")
    if profile.emotion.adapter_id in WAVLM_ADAPTER_IDS:
        if profile.emotion.weights_sha256 is None:
            raise ProfileError("WavLM emotion profile requires emotion.weights_sha256")
        if profile.emotion.label_map is None:
            raise ProfileError("generic WavLM emotion profile requires emotion.label_map")
        _validate_wavlm_weights(profile.emotion)
    elif profile.emotion.weights_sha256 is not None or profile.emotion.label_map is not None:
        raise ProfileError("emotion weights and label map are only valid for a WavLM adapter")
    try:
        result = subprocess.run(
            ["git", "-C", str(profile.voxtral.transcribe_root), "rev-parse", "HEAD"],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ProfileError(f"cannot inspect transcribe.cpp revision: {error}") from error
    if result.returncode != 0 or result.stdout.strip() != profile.voxtral.runtime_revision:
        raise ProfileError("transcribe.cpp revision does not match the profile")
    _require_external(profile.emotion.cache_dir, "emotion cache")
    _require_external(profile.service.ready_file, "ready file")
    if not profile.service.ready_file.parent.is_dir():
        raise ProfileError("ready-file parent directory does not exist")
    if profile.service.ready_file.exists():
        raise ProfileError("ready file already exists")


def build_adapters(
    profile: SpeechTimelineProfile,
) -> tuple[VoxtralTranscriberAdapter, Emotion2VecAffectAdapter | WavlmRussianResdAffectAdapter]:
    transcriber = VoxtralTranscriberAdapter(
        profile.voxtral.model_path,
        profile.voxtral.transcribe_root,
        profile.voxtral.library,
        backend=profile.voxtral.backend,
        delay_ms=profile.voxtral.delay_ms,
        partial_decode_interval_ms=profile.voxtral.partial_decode_interval_ms,
    )
    if profile.emotion.adapter_id == EMOTION2VEC_ADAPTER_ID:
        probe = EmotionProbe(
            profile.emotion.model_id,
            profile.emotion.model_revision,
            profile.emotion.cache_dir,
            profile.emotion.device,
            local_files_only=True,
        )
        return transcriber, Emotion2VecAffectAdapter(probe)
    if profile.emotion.adapter_id in WAVLM_ADAPTER_IDS:
        assert profile.emotion.weights_sha256 is not None
        assert profile.emotion.label_map is not None
        return transcriber, WavlmAffectAdapter(
            profile.emotion.model_id,
            profile.emotion.model_revision,
            profile.emotion.cache_dir,
            profile.emotion.device,
            profile.emotion.weights_sha256,
            dict(profile.emotion.label_map),
            profile.emotion.adapter_id,
        )
    raise AssertionError(f"unsupported validated emotion adapter: {profile.emotion.adapter_id}")


def _object(value: object, keys: set[str], name: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != keys:
        raise ProfileError(f"{name} fields do not match schema")
    return value


def _object_with_optional(
    value: object, required: set[str], optional: set[str], name: str
) -> dict[str, Any]:
    if (
        not isinstance(value, dict)
        or not required.issubset(value)
        or not set(value).issubset(required | optional)
    ):
        raise ProfileError(f"{name} fields do not match schema")
    return value


def _string(value: object, name: str, maximum: int) -> str:
    if not isinstance(value, str) or not value or len(value.encode("utf-8")) > maximum:
        raise ProfileError(f"{name} must be a bounded non-empty string")
    return value


def _path(value: object, name: str) -> Path:
    return Path(_string(value, name, 4096)).expanduser().resolve()


def _positive_int(value: object, name: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        raise ProfileError(f"{name} must be a positive integer")
    return value


def _port(value: object) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not 0 <= value <= 65535:
        raise ProfileError("port must be between 0 and 65535")
    return value


def _choice(value: object, name: str, choices: set[str]) -> str:
    result = _string(value, name, 128)
    if result not in choices:
        raise ProfileError(f"unsupported {name}: {result}")
    return result


def _sha256(value: object, name: str) -> str:
    result = _string(value, name, 71)
    if len(result) != 71 or not result.startswith("sha256:"):
        raise ProfileError(f"{name} must be sha256:<64 lowercase hex>")
    try:
        int(result[7:], 16)
    except ValueError as error:
        raise ProfileError(f"{name} must contain hexadecimal digits") from error
    if result.lower() != result:
        raise ProfileError(f"{name} must use lowercase hexadecimal digits")
    return result


def _label_map(value: object, name: str) -> tuple[tuple[str, str], ...]:
    if not isinstance(value, dict) or not 1 <= len(value) <= 32:
        raise ProfileError(f"{name} must be an object with between 1 and 32 labels")
    result: list[tuple[str, str]] = []
    normalized: set[str] = set()
    for source, target in value.items():
        source_name = _string(source, f"{name} source label", 128)
        target_name = _string(target, f"{name}.{source_name}", 64)
        if re.fullmatch(r"[a-z][a-z0-9_-]*", target_name) is None:
            raise ProfileError(f"{name} values must be lowercase timeline labels")
        if target_name in normalized:
            raise ProfileError(f"{name} values must be unique")
        normalized.add(target_name)
        result.append((source_name, target_name))
    return tuple(result)


def _validate_wavlm_weights(emotion: EmotionProfile) -> None:
    assert emotion.weights_sha256 is not None
    model_directory = f"models--{emotion.model_id.replace('/', '--')}"
    weights = (
        emotion.cache_dir
        / model_directory
        / "snapshots"
        / emotion.model_revision
        / WAVLM_WEIGHTS_FILENAME
    )
    if not weights.is_file():
        raise ProfileError(f"pinned WavLM weights do not exist: {weights}")
    resolved_weights = weights.resolve()
    try:
        resolved_weights.relative_to(emotion.cache_dir)
    except ValueError as error:
        raise ProfileError("pinned WavLM weights resolve outside the emotion cache") from error
    digest = hashlib.sha256()
    with resolved_weights.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    if f"sha256:{digest.hexdigest()}" != emotion.weights_sha256:
        raise ProfileError("WavLM model.safetensors SHA-256 does not match the profile")


def _require_external(path: Path, name: str) -> None:
    if path == REPOSITORY_ROOT or path.is_relative_to(REPOSITORY_ROOT):
        raise ProfileError(f"{name} must be outside the repository")
