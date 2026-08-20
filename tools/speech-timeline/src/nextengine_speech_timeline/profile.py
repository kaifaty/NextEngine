from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
from typing import Any

from nextengine_emotion_probe.probe import EmotionProbe

from .adapters.dpdfnet import ADAPTER_ID as DPDFNET_ADAPTER_ID
from .adapters.dpdfnet import (
    DEFAULT_WHISPER_ATTENUATION_LIMIT_DB,
    DpdfNetAudioPreprocessor,
    GAIN_PLACEMENT_POST_DENOISE,
    SpeechAwareGainConfig,
    SUPPORTED_GAIN_PLACEMENTS,
    SUPPORTED_MODELS as DPDFNET_MODELS,
)
from .adapters.emotion2vec import Emotion2VecAffectAdapter
from .adapters.base import AudioPreprocessor
from .adapters.gigaam import (
    MODEL_ROUTE_ID as GIGAAM_MODEL_ROUTE_ID,
    GigaAmTranscriberAdapter,
)
from .adapters.gigastt import (
    MODEL_ROUTE_ID as GIGASTT_MODEL_ROUTE_ID,
    GigasttTranscriberAdapter,
)
from .adapters.nemotron_nemo_speech_cpp import (
    MODEL_ROUTE_ID as NEMOTRON_MODEL_ROUTE_ID,
    SUPPORTED_RIGHT_CONTEXT as NEMOTRON_SUPPORTED_RIGHT_CONTEXT,
    NemotronTranscriberAdapter,
)
from .adapters.voxtral_transcribe_cpp import VoxtralTranscriberAdapter
from .adapters.spectral_onnx import (
    CompositeAudioPreprocessor,
    GTCRN_ADAPTER_ID,
    SPECS_BY_ADAPTER_ID,
    UL_UNAS_ADAPTER_ID,
    StreamingSpectralOnnxAudioPreprocessor,
)
from .adapters.wavlm_russian_resd import (
    ADAPTER_ID as WAVLM_RUSSIAN_RESD_ADAPTER_ID,
    DEFAULT_LABEL_MAP,
    GENERIC_AUDIO_ADAPTER_ID as TRANSFORMERS_AUDIO_CLASSIFICATION_ADAPTER_ID,
    GENERIC_AUDIO_MODEL_TYPES,
    GENERIC_ADAPTER_ID as WAVLM_AUDIO_CLASSIFICATION_ADAPTER_ID,
    WAVLM_MODEL_TYPES,
    WEIGHTS_FILENAME as WAVLM_WEIGHTS_FILENAME,
    WavlmAffectAdapter,
    WavlmRussianResdAffectAdapter,
)
from .session import SessionBounds


REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
EMOTION2VEC_ADAPTER_ID = "emotion2vec-plus/1"
WAVLM_ADAPTER_IDS = {WAVLM_RUSSIAN_RESD_ADAPTER_ID, WAVLM_AUDIO_CLASSIFICATION_ADAPTER_ID}
TRANSFORMERS_AUDIO_ADAPTER_IDS = {*WAVLM_ADAPTER_IDS, TRANSFORMERS_AUDIO_CLASSIFICATION_ADAPTER_ID}
SUPPORTED_EMOTION_ADAPTERS = {EMOTION2VEC_ADAPTER_ID, *TRANSFORMERS_AUDIO_ADAPTER_IDS}
SUPPORTED_AUDIO_PREPROCESSORS = {DPDFNET_ADAPTER_ID}
SUPPORTED_AUDIO_ENHANCERS = {GTCRN_ADAPTER_ID, UL_UNAS_ADAPTER_ID}
VOXTRAL_MODEL_ROUTE_ID = "voxtral-realtime"
SUPPORTED_ASR_MODEL_ROUTES = {
    VOXTRAL_MODEL_ROUTE_ID,
    GIGAAM_MODEL_ROUTE_ID,
    GIGASTT_MODEL_ROUTE_ID,
    NEMOTRON_MODEL_ROUTE_ID,
}


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
class GigaAmProfile:
    model_id: str
    model_revision: str
    snapshot_path: Path
    device: str
    classification: str
    weights_sha256: str
    modeling_sha256: str
    config_sha256: str
    tokenizer_sha256: str


@dataclass(frozen=True)
class GigasttProfile:
    model_id: str
    model_revision: str
    model_dir: Path
    runtime_path: Path
    runtime_version: str
    runtime_revision: str
    runtime_sha256: str
    encoder_sha256: str
    decoder_sha256: str
    joint_sha256: str
    vocab_sha256: str
    classification: str


@dataclass(frozen=True)
class NemotronProfile:
    model_id: str
    model_revision: str
    model_path: Path
    model_size_bytes: int
    model_sha256: str
    runtime_root: Path
    runtime_revision: str
    implementation_library: Path
    implementation_library_sha256: str
    abi_library: Path
    abi_library_sha256: str
    gpu: int
    right_context: int
    classification: str


@dataclass(frozen=True)
class AudioPreprocessorProfile:
    adapter_id: str
    model_id: str
    model_revision: str
    model_name: str
    model_path: Path
    model_size_bytes: int
    model_sha256: str
    routing: str
    gain_config: SpeechAwareGainConfig
    gain_placement: str
    whisper_attenuation_limit_db: float


@dataclass(frozen=True)
class AudioEnhancerProfile:
    adapter_id: str
    model_id: str
    model_revision: str
    model_path: Path
    model_size_bytes: int
    model_sha256: str
    routing: str


@dataclass(frozen=True)
class ServiceProfile:
    port: int
    ready_file: Path
    bounds: SessionBounds
    diagnostic_audio_root: Path | None
    diagnostic_audio_max_records: int | None


@dataclass(frozen=True)
class SpeechTimelineProfile:
    voxtral: VoxtralProfile
    gigaam: GigaAmProfile | None
    gigastt: GigasttProfile | None
    nemotron: NemotronProfile | None
    default_asr_model: str
    emotion: EmotionProfile
    audio_preprocessor: AudioPreprocessorProfile | None
    audio_enhancers: tuple[AudioEnhancerProfile, ...]
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
    root = _object_with_optional(
        value,
        {"schema_version", "voxtral", "emotion", "service"},
        {
            "audio_preprocessor",
            "audio_enhancers",
            "gigaam",
            "gigastt",
            "nemotron",
            "default_asr_model",
        },
        "profile",
    )
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
    gigaam = (
        _object(
            root["gigaam"],
            {
                "model_id",
                "model_revision",
                "snapshot_path",
                "device",
                "classification",
                "weights_sha256",
                "modeling_sha256",
                "config_sha256",
                "tokenizer_sha256",
            },
            "gigaam",
        )
        if "gigaam" in root
        else None
    )
    gigastt = (
        _object(
            root["gigastt"],
            {
                "model_id",
                "model_revision",
                "model_dir",
                "runtime_path",
                "runtime_version",
                "runtime_revision",
                "runtime_sha256",
                "encoder_sha256",
                "decoder_sha256",
                "joint_sha256",
                "vocab_sha256",
                "classification",
            },
            "gigastt",
        )
        if "gigastt" in root
        else None
    )
    nemotron = (
        _object(
            root["nemotron"],
            {
                "model_id",
                "model_revision",
                "model_path",
                "model_size_bytes",
                "model_sha256",
                "runtime_root",
                "runtime_revision",
                "implementation_library",
                "implementation_library_sha256",
                "abi_library",
                "abi_library_sha256",
                "gpu",
                "right_context",
                "classification",
            },
            "nemotron",
        )
        if "nemotron" in root
        else None
    )
    audio_preprocessor = (
        _object_with_optional(
            root["audio_preprocessor"],
            {
                "adapter_id",
                "model_id",
                "model_revision",
                "model_name",
                "model_path",
                "model_size_bytes",
                "model_sha256",
                "routing",
            },
            {"gain", "gain_placement", "whisper_attenuation_limit_db"},
            "audio_preprocessor",
        )
        if "audio_preprocessor" in root
        else None
    )
    gain = (
        _object(
            audio_preprocessor["gain"],
            {
                "enabled",
                "activation_threshold_dbfs",
                "target_dbfs",
                "max_gain_db",
                "attack_ms",
                "release_ms",
                "limiter_peak_dbfs",
            },
            "audio_preprocessor.gain",
        )
        if audio_preprocessor is not None and "gain" in audio_preprocessor
        else None
    )
    audio_enhancers = _audio_enhancers(root.get("audio_enhancers", []))
    service = _object_with_optional(
        root["service"],
        {"port", "ready_file", "max_frame_bytes", "max_turn_bytes"},
        {"diagnostic_audio"},
        "service",
    )
    diagnostic_audio = (
        _object(service["diagnostic_audio"], {"root", "max_records"}, "service.diagnostic_audio")
        if "diagnostic_audio" in service
        else None
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
        gigaam=(
            GigaAmProfile(
                model_id=_string(gigaam["model_id"], "gigaam.model_id", 256),
                model_revision=_string(
                    gigaam["model_revision"], "gigaam.model_revision", 128
                ),
                snapshot_path=_path(gigaam["snapshot_path"], "gigaam.snapshot_path"),
                device=_choice(gigaam["device"], "gigaam.device", {"auto", "cpu", "cuda"}),
                classification=_choice(
                    gigaam["classification"],
                    "gigaam.classification",
                    {"unclassified_local_only", "classified_local_only"},
                ),
                weights_sha256=_sha256(
                    gigaam["weights_sha256"], "gigaam.weights_sha256"
                ),
                modeling_sha256=_sha256(
                    gigaam["modeling_sha256"], "gigaam.modeling_sha256"
                ),
                config_sha256=_sha256(
                    gigaam["config_sha256"], "gigaam.config_sha256"
                ),
                tokenizer_sha256=_sha256(
                    gigaam["tokenizer_sha256"], "gigaam.tokenizer_sha256"
                ),
            )
            if gigaam is not None
            else None
        ),
        gigastt=(
            GigasttProfile(
                model_id=_string(gigastt["model_id"], "gigastt.model_id", 256),
                model_revision=_string(
                    gigastt["model_revision"], "gigastt.model_revision", 128
                ),
                model_dir=_path(gigastt["model_dir"], "gigastt.model_dir"),
                runtime_path=_path(gigastt["runtime_path"], "gigastt.runtime_path"),
                runtime_version=_string(
                    gigastt["runtime_version"], "gigastt.runtime_version", 64
                ),
                runtime_revision=_string(
                    gigastt["runtime_revision"], "gigastt.runtime_revision", 128
                ),
                runtime_sha256=_sha256(
                    gigastt["runtime_sha256"], "gigastt.runtime_sha256"
                ),
                encoder_sha256=_sha256(
                    gigastt["encoder_sha256"], "gigastt.encoder_sha256"
                ),
                decoder_sha256=_sha256(
                    gigastt["decoder_sha256"], "gigastt.decoder_sha256"
                ),
                joint_sha256=_sha256(
                    gigastt["joint_sha256"], "gigastt.joint_sha256"
                ),
                vocab_sha256=_sha256(
                    gigastt["vocab_sha256"], "gigastt.vocab_sha256"
                ),
                classification=_choice(
                    gigastt["classification"],
                    "gigastt.classification",
                    {"unclassified_local_only", "classified_local_only"},
                ),
            )
            if gigastt is not None
            else None
        ),
        nemotron=(
            NemotronProfile(
                model_id=_string(nemotron["model_id"], "nemotron.model_id", 256),
                model_revision=_string(
                    nemotron["model_revision"], "nemotron.model_revision", 128
                ),
                model_path=_path(nemotron["model_path"], "nemotron.model_path"),
                model_size_bytes=_positive_int(
                    nemotron["model_size_bytes"], "nemotron.model_size_bytes"
                ),
                model_sha256=_sha256(
                    nemotron["model_sha256"], "nemotron.model_sha256"
                ),
                runtime_root=_path(nemotron["runtime_root"], "nemotron.runtime_root"),
                runtime_revision=_string(
                    nemotron["runtime_revision"], "nemotron.runtime_revision", 128
                ),
                implementation_library=_path(
                    nemotron["implementation_library"],
                    "nemotron.implementation_library",
                ),
                implementation_library_sha256=_sha256(
                    nemotron["implementation_library_sha256"],
                    "nemotron.implementation_library_sha256",
                ),
                abi_library=_path(
                    nemotron["abi_library"], "nemotron.abi_library"
                ),
                abi_library_sha256=_sha256(
                    nemotron["abi_library_sha256"],
                    "nemotron.abi_library_sha256",
                ),
                gpu=_bounded_int(nemotron["gpu"], "nemotron.gpu", -1, 15),
                right_context=_choice_int(
                    nemotron["right_context"],
                    "nemotron.right_context",
                    set(NEMOTRON_SUPPORTED_RIGHT_CONTEXT),
                ),
                classification=_choice(
                    nemotron["classification"],
                    "nemotron.classification",
                    {"unclassified_local_only", "classified_local_only"},
                ),
            )
            if nemotron is not None
            else None
        ),
        default_asr_model=_choice(
            root.get("default_asr_model", VOXTRAL_MODEL_ROUTE_ID),
            "default_asr_model",
            SUPPORTED_ASR_MODEL_ROUTES,
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
        audio_preprocessor=(
            AudioPreprocessorProfile(
                adapter_id=_choice(
                    audio_preprocessor["adapter_id"],
                    "audio_preprocessor.adapter_id",
                    SUPPORTED_AUDIO_PREPROCESSORS,
                ),
                model_id=_string(audio_preprocessor["model_id"], "audio_preprocessor.model_id", 256),
                model_revision=_string(
                    audio_preprocessor["model_revision"],
                    "audio_preprocessor.model_revision",
                    128,
                ),
                model_name=_choice(
                    audio_preprocessor["model_name"],
                    "audio_preprocessor.model_name",
                    DPDFNET_MODELS,
                ),
                model_path=_path(audio_preprocessor["model_path"], "audio_preprocessor.model_path"),
                model_size_bytes=_positive_int(
                    audio_preprocessor["model_size_bytes"],
                    "audio_preprocessor.model_size_bytes",
                ),
                model_sha256=_sha256(
                    audio_preprocessor["model_sha256"],
                    "audio_preprocessor.model_sha256",
                ),
                routing=_choice(
                    audio_preprocessor["routing"],
                    "audio_preprocessor.routing",
                    {"asr_only"},
                ),
                gain_config=_gain_config(gain),
                gain_placement=_choice(
                    audio_preprocessor.get("gain_placement", GAIN_PLACEMENT_POST_DENOISE),
                    "audio_preprocessor.gain_placement",
                    SUPPORTED_GAIN_PLACEMENTS,
                ),
                whisper_attenuation_limit_db=_bounded_float(
                    audio_preprocessor.get(
                        "whisper_attenuation_limit_db",
                        DEFAULT_WHISPER_ATTENUATION_LIMIT_DB,
                    ),
                    "audio_preprocessor.whisper_attenuation_limit_db",
                    0.0,
                    40.0,
                ),
            )
            if audio_preprocessor is not None
            else None
        ),
        audio_enhancers=audio_enhancers,
        service=ServiceProfile(
            port=_port(service["port"]),
            ready_file=_path(service["ready_file"], "ready_file"),
            bounds=SessionBounds(
                max_frame_bytes=_positive_int(service["max_frame_bytes"], "max_frame_bytes"),
                max_turn_bytes=_positive_int(service["max_turn_bytes"], "max_turn_bytes"),
            ),
            diagnostic_audio_root=(
                _path(diagnostic_audio["root"], "service.diagnostic_audio.root")
                if diagnostic_audio is not None
                else None
            ),
            diagnostic_audio_max_records=(
                _bounded_int(
                    diagnostic_audio["max_records"],
                    "service.diagnostic_audio.max_records",
                    1,
                    5,
                )
                if diagnostic_audio is not None
                else None
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
    if profile.emotion.adapter_id in TRANSFORMERS_AUDIO_ADAPTER_IDS:
        if profile.emotion.weights_sha256 is None:
            raise ProfileError("WavLM emotion profile requires emotion.weights_sha256")
        if profile.emotion.label_map is None:
            raise ProfileError("generic WavLM emotion profile requires emotion.label_map")
        _validate_wavlm_weights(profile.emotion)
    elif profile.emotion.weights_sha256 is not None or profile.emotion.label_map is not None:
        raise ProfileError("emotion weights and label map are only valid for a WavLM adapter")
    if profile.audio_preprocessor is not None:
        _validate_audio_preprocessor(profile.audio_preprocessor)
    for enhancer in profile.audio_enhancers:
        _validate_audio_enhancer(enhancer)
    if profile.gigaam is not None:
        _validate_gigaam(profile.gigaam)
    elif profile.default_asr_model == GIGAAM_MODEL_ROUTE_ID:
        raise ProfileError("default_asr_model requires a configured gigaam profile")
    if profile.gigastt is not None:
        _validate_gigastt(profile.gigastt)
    elif profile.default_asr_model == GIGASTT_MODEL_ROUTE_ID:
        raise ProfileError("default_asr_model requires a configured gigastt profile")
    if profile.nemotron is not None:
        _validate_nemotron(profile.nemotron)
    elif profile.default_asr_model == NEMOTRON_MODEL_ROUTE_ID:
        raise ProfileError("default_asr_model requires a configured nemotron profile")
    _validate_git_revision(
        profile.voxtral.transcribe_root,
        profile.voxtral.runtime_revision,
        "transcribe.cpp",
    )
    _require_external(profile.emotion.cache_dir, "emotion cache")
    _require_external(profile.service.ready_file, "ready file")
    if not profile.service.ready_file.parent.is_dir():
        raise ProfileError("ready-file parent directory does not exist")
    if profile.service.ready_file.exists():
        raise ProfileError("ready file already exists")
    if profile.service.diagnostic_audio_root is not None:
        _require_external(profile.service.diagnostic_audio_root, "diagnostic audio root")
        if not profile.service.diagnostic_audio_root.parent.is_dir():
            raise ProfileError("diagnostic audio root parent directory does not exist")


def build_adapters(
    profile: SpeechTimelineProfile,
) -> tuple[
    dict[
        str,
        VoxtralTranscriberAdapter
        | GigaAmTranscriberAdapter
        | GigasttTranscriberAdapter
        | NemotronTranscriberAdapter,
    ],
    str,
    Emotion2VecAffectAdapter | WavlmRussianResdAffectAdapter,
    AudioPreprocessor | None,
]:
    transcribers: dict[
        str,
        VoxtralTranscriberAdapter
        | GigaAmTranscriberAdapter
        | GigasttTranscriberAdapter
        | NemotronTranscriberAdapter,
    ] = {
        VOXTRAL_MODEL_ROUTE_ID: VoxtralTranscriberAdapter(
            profile.voxtral.model_path,
            profile.voxtral.transcribe_root,
            profile.voxtral.library,
            backend=profile.voxtral.backend,
            delay_ms=profile.voxtral.delay_ms,
            partial_decode_interval_ms=profile.voxtral.partial_decode_interval_ms,
        )
    }
    if profile.gigaam is not None:
        transcribers[GIGAAM_MODEL_ROUTE_ID] = GigaAmTranscriberAdapter(
            profile.gigaam.snapshot_path,
            model_id=profile.gigaam.model_id,
            model_revision=profile.gigaam.model_revision,
            device=profile.gigaam.device,
        )
    if profile.gigastt is not None:
        transcribers[GIGASTT_MODEL_ROUTE_ID] = GigasttTranscriberAdapter(
            profile.gigastt.runtime_path,
            profile.gigastt.model_dir,
            model_id=profile.gigastt.model_id,
            model_revision=profile.gigastt.model_revision,
            runtime_version=profile.gigastt.runtime_version,
        )
    if profile.nemotron is not None:
        transcribers[NEMOTRON_MODEL_ROUTE_ID] = NemotronTranscriberAdapter(
            profile.nemotron.model_path,
            profile.nemotron.implementation_library,
            profile.nemotron.abi_library,
            model_id=profile.nemotron.model_id,
            model_revision=profile.nemotron.model_revision,
            gpu=profile.nemotron.gpu,
            right_context=profile.nemotron.right_context,
        )
    if profile.emotion.adapter_id == EMOTION2VEC_ADAPTER_ID:
        probe = EmotionProbe(
            profile.emotion.model_id,
            profile.emotion.model_revision,
            profile.emotion.cache_dir,
            profile.emotion.device,
            local_files_only=True,
        )
        affect = Emotion2VecAffectAdapter(probe)
    elif profile.emotion.adapter_id in TRANSFORMERS_AUDIO_ADAPTER_IDS:
        assert profile.emotion.weights_sha256 is not None
        assert profile.emotion.label_map is not None
        affect = WavlmAffectAdapter(
            profile.emotion.model_id,
            profile.emotion.model_revision,
            profile.emotion.cache_dir,
            profile.emotion.device,
            profile.emotion.weights_sha256,
            dict(profile.emotion.label_map),
            profile.emotion.adapter_id,
            (
                GENERIC_AUDIO_MODEL_TYPES
                if profile.emotion.adapter_id == TRANSFORMERS_AUDIO_CLASSIFICATION_ADAPTER_ID
                else WAVLM_MODEL_TYPES
            ),
        )
    else:
        raise AssertionError(f"unsupported validated emotion adapter: {profile.emotion.adapter_id}")
    preprocessors: list[AudioPreprocessor] = []
    if profile.audio_preprocessor is not None:
        preprocessors.append(
            DpdfNetAudioPreprocessor(
                model_id=profile.audio_preprocessor.model_id,
                model_revision=profile.audio_preprocessor.model_revision,
                model_name=profile.audio_preprocessor.model_name,
                onnx_path=profile.audio_preprocessor.model_path,
                gain_config=profile.audio_preprocessor.gain_config,
                gain_placement=profile.audio_preprocessor.gain_placement,
                whisper_attenuation_limit_db=(
                    profile.audio_preprocessor.whisper_attenuation_limit_db
                ),
            )
        )
    for enhancer in profile.audio_enhancers:
        preprocessors.append(
            StreamingSpectralOnnxAudioPreprocessor(
                spec=SPECS_BY_ADAPTER_ID[enhancer.adapter_id],
                model_id=enhancer.model_id,
                model_revision=enhancer.model_revision,
                onnx_path=enhancer.model_path,
            )
        )
    preprocessor: AudioPreprocessor | None
    if len(preprocessors) == 1:
        preprocessor = preprocessors[0]
    elif preprocessors:
        preprocessor = CompositeAudioPreprocessor(preprocessors)
    else:
        preprocessor = None
    return transcribers, profile.default_asr_model, affect, preprocessor


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


def _bool(value: object, name: str) -> bool:
    if not isinstance(value, bool):
        raise ProfileError(f"{name} must be a boolean")
    return value


def _finite_float(value: object, name: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ProfileError(f"{name} must be a finite number")
    result = float(value)
    if not math.isfinite(result):
        raise ProfileError(f"{name} must be a finite number")
    return result


def _bounded_float(value: object, name: str, minimum: float, maximum: float) -> float:
    result = _finite_float(value, name)
    if not minimum <= result <= maximum:
        raise ProfileError(f"{name} must be between {minimum} and {maximum}")
    return result


def _gain_config(value: dict[str, Any] | None) -> SpeechAwareGainConfig:
    if value is None:
        return SpeechAwareGainConfig()
    try:
        return SpeechAwareGainConfig(
            enabled=_bool(value["enabled"], "audio_preprocessor.gain.enabled"),
            activation_threshold_dbfs=_finite_float(
                value["activation_threshold_dbfs"],
                "audio_preprocessor.gain.activation_threshold_dbfs",
            ),
            target_dbfs=_finite_float(value["target_dbfs"], "audio_preprocessor.gain.target_dbfs"),
            max_gain_db=_finite_float(value["max_gain_db"], "audio_preprocessor.gain.max_gain_db"),
            attack_ms=_positive_int(value["attack_ms"], "audio_preprocessor.gain.attack_ms"),
            release_ms=_positive_int(value["release_ms"], "audio_preprocessor.gain.release_ms"),
            limiter_peak_dbfs=_finite_float(
                value["limiter_peak_dbfs"], "audio_preprocessor.gain.limiter_peak_dbfs"
            ),
        )
    except ValueError as error:
        raise ProfileError(f"invalid audio_preprocessor.gain: {error}") from error


def _bounded_int(value: object, name: str, minimum: int, maximum: int) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not minimum <= value <= maximum:
        raise ProfileError(f"{name} must be between {minimum} and {maximum}")
    return value


def _choice_int(value: object, name: str, choices: set[int]) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value not in choices:
        raise ProfileError(f"unsupported {name}: {value}")
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


def _validate_audio_preprocessor(preprocessor: AudioPreprocessorProfile) -> None:
    model = preprocessor.model_path
    _require_external(model, "audio preprocessor model")
    if not model.is_file():
        raise ProfileError(f"audio preprocessor model does not exist: {model}")
    actual_size = model.stat().st_size
    if actual_size != preprocessor.model_size_bytes:
        raise ProfileError(
            "audio preprocessor model size mismatch: "
            f"expected {preprocessor.model_size_bytes}, got {actual_size}"
        )
    digest = hashlib.sha256()
    with model.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    if f"sha256:{digest.hexdigest()}" != preprocessor.model_sha256:
        raise ProfileError("audio preprocessor model SHA-256 does not match the profile")


def _audio_enhancers(value: object) -> tuple[AudioEnhancerProfile, ...]:
    if not isinstance(value, list) or len(value) > len(SUPPORTED_AUDIO_ENHANCERS):
        raise ProfileError("audio_enhancers must be a bounded list")
    result: list[AudioEnhancerProfile] = []
    adapter_ids: set[str] = set()
    for index, item in enumerate(value):
        name = f"audio_enhancers[{index}]"
        enhancer = _object(
            item,
            {
                "adapter_id",
                "model_id",
                "model_revision",
                "model_path",
                "model_size_bytes",
                "model_sha256",
                "routing",
            },
            name,
        )
        adapter_id = _choice(
            enhancer["adapter_id"], f"{name}.adapter_id", SUPPORTED_AUDIO_ENHANCERS
        )
        if adapter_id in adapter_ids:
            raise ProfileError("audio_enhancers adapter IDs must be unique")
        adapter_ids.add(adapter_id)
        result.append(
            AudioEnhancerProfile(
                adapter_id=adapter_id,
                model_id=_string(enhancer["model_id"], f"{name}.model_id", 256),
                model_revision=_string(
                    enhancer["model_revision"], f"{name}.model_revision", 128
                ),
                model_path=_path(enhancer["model_path"], f"{name}.model_path"),
                model_size_bytes=_positive_int(
                    enhancer["model_size_bytes"], f"{name}.model_size_bytes"
                ),
                model_sha256=_sha256(
                    enhancer["model_sha256"], f"{name}.model_sha256"
                ),
                routing=_choice(
                    enhancer["routing"], f"{name}.routing", {"asr_only"}
                ),
            )
        )
    return tuple(result)


def _validate_audio_enhancer(enhancer: AudioEnhancerProfile) -> None:
    model = enhancer.model_path
    _require_external(model, f"{enhancer.adapter_id} model")
    if not model.is_file() or model.is_symlink():
        raise ProfileError(f"audio enhancer model does not exist: {model}")
    if model.stat().st_size != enhancer.model_size_bytes:
        raise ProfileError(f"{enhancer.adapter_id} model size does not match the profile")
    if _file_sha256(model) != enhancer.model_sha256:
        raise ProfileError(f"{enhancer.adapter_id} model SHA-256 does not match the profile")


def _validate_gigaam(gigaam: GigaAmProfile) -> None:
    snapshot = gigaam.snapshot_path
    _require_external(snapshot, "GigaAM snapshot")
    if not snapshot.is_dir() or snapshot.is_symlink():
        raise ProfileError(f"GigaAM snapshot does not exist: {snapshot}")
    expected = {
        "pytorch_model.bin": gigaam.weights_sha256,
        "modeling_gigaam.py": gigaam.modeling_sha256,
        "config.json": gigaam.config_sha256,
        "tokenizer.model": gigaam.tokenizer_sha256,
    }
    for filename, expected_hash in expected.items():
        path = snapshot / filename
        if not path.is_file():
            raise ProfileError(f"pinned GigaAM artifact does not exist: {path}")
        actual_hash = _file_sha256(path)
        if actual_hash != expected_hash:
            raise ProfileError(f"GigaAM {filename} SHA-256 does not match the profile")


def _validate_gigastt(gigastt: GigasttProfile) -> None:
    _require_external(gigastt.runtime_path, "gigastt runtime")
    _require_external(gigastt.model_dir, "gigastt model directory")
    if not gigastt.runtime_path.is_file() or gigastt.runtime_path.is_symlink():
        raise ProfileError(f"pinned gigastt runtime does not exist: {gigastt.runtime_path}")
    if not gigastt.model_dir.is_dir() or gigastt.model_dir.is_symlink():
        raise ProfileError(f"pinned gigastt model directory does not exist: {gigastt.model_dir}")
    expected = (
        (gigastt.runtime_path, gigastt.runtime_sha256, "runtime"),
        (
            gigastt.model_dir / "v3_rnnt_encoder_int8.onnx",
            gigastt.encoder_sha256,
            "encoder",
        ),
        (
            gigastt.model_dir / "v3_rnnt_decoder.onnx",
            gigastt.decoder_sha256,
            "decoder",
        ),
        (
            gigastt.model_dir / "v3_rnnt_joint.onnx",
            gigastt.joint_sha256,
            "joint",
        ),
        (gigastt.model_dir / "v3_vocab.txt", gigastt.vocab_sha256, "vocabulary"),
    )
    for path, expected_hash, name in expected:
        if not path.is_file() or path.is_symlink():
            raise ProfileError(f"pinned gigastt {name} does not exist: {path}")
        if _file_sha256(path) != expected_hash:
            raise ProfileError(f"gigastt {name} SHA-256 does not match the profile")
    try:
        result = subprocess.run(
            [str(gigastt.runtime_path), "--version"],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ProfileError(f"cannot inspect gigastt runtime version: {error}") from error
    if result.returncode != 0 or result.stdout.strip() != f"gigastt {gigastt.runtime_version}":
        raise ProfileError("gigastt runtime version does not match the profile")


def _validate_nemotron(nemotron: NemotronProfile) -> None:
    _require_external(nemotron.model_path, "Nemotron model")
    _require_external(nemotron.runtime_root, "NeMo-Speech.cpp checkout")
    _require_external(
        nemotron.implementation_library,
        "NeMo-Speech.cpp implementation library",
    )
    _require_external(nemotron.abi_library, "NeMo-Speech.cpp C ABI library")
    if not nemotron.model_path.is_file():
        raise ProfileError(f"Nemotron model does not exist: {nemotron.model_path}")
    if nemotron.model_path.stat().st_size != nemotron.model_size_bytes:
        raise ProfileError("Nemotron model size does not match the profile")
    expected = (
        (nemotron.model_path, nemotron.model_sha256, "model"),
        (
            nemotron.implementation_library,
            nemotron.implementation_library_sha256,
            "implementation library",
        ),
        (nemotron.abi_library, nemotron.abi_library_sha256, "C ABI library"),
    )
    for path, expected_hash, name in expected:
        if not path.is_file():
            raise ProfileError(f"pinned Nemotron {name} does not exist: {path}")
        if _file_sha256(path) != expected_hash:
            raise ProfileError(f"Nemotron {name} SHA-256 does not match the profile")
    _validate_git_revision(
        nemotron.runtime_root,
        nemotron.runtime_revision,
        "NeMo-Speech.cpp",
    )


def _validate_git_revision(root: Path, revision: str, name: str) -> None:
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ProfileError(f"cannot inspect {name} revision: {error}") from error
    if result.returncode != 0 or result.stdout.strip() != revision:
        raise ProfileError(f"{name} revision does not match the profile")


def _file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def _require_external(path: Path, name: str) -> None:
    if path == REPOSITORY_ROOT or path.is_relative_to(REPOSITORY_ROOT):
        raise ProfileError(f"{name} must be outside the repository")
