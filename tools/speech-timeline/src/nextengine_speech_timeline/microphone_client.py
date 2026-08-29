from __future__ import annotations

import asyncio
from collections.abc import AsyncIterator, Callable
from dataclasses import dataclass
import importlib.util
import json
import os
from pathlib import Path
import secrets
import stat
import subprocess
import sys
import time
from types import ModuleType
from urllib.parse import urlsplit

from websockets.asyncio.client import connect

from . import SERVICE_PROTOCOL
from .protocol import ASR_AUDIO_ROUTES, ASR_AUDIO_ROUTE_RAW, MAX_JSON_BYTES


REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
CAPTURE_SCRIPT = REPOSITORY_ROOT / "lab" / "scripts" / "voxtral_microphone.py"


class ClientError(RuntimeError):
    pass


@dataclass(frozen=True)
class ReadyInfo:
    uri: str
    token: str
    protocol: str
    bounds: dict[str, object]
    models: dict[str, object] | None = None
    model_identity: dict[str, object] | None = None


def load_ready_file(path: Path) -> ReadyInfo:
    resolved = path.expanduser().absolute()
    try:
        metadata = resolved.lstat()
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            raise ClientError("ready file must be a regular file, not a symlink")
        if metadata.st_uid != os.getuid():
            raise ClientError("ready file must be owned by the current user")
        mode = stat.S_IMODE(metadata.st_mode)
        raw = resolved.read_bytes()
    except OSError as error:
        raise ClientError(f"cannot read ready file: {error}") from error
    if mode & 0o077:
        raise ClientError("ready file must not be readable by group or others")
    if len(raw) > MAX_JSON_BYTES:
        raise ClientError("ready file exceeds 65536 bytes")
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ClientError(f"invalid ready file JSON: {error}") from error
    if not isinstance(value, dict) or value.get("schema_version") != 1:
        raise ClientError("ready file schema_version must be 1")
    uri = value.get("uri")
    token = value.get("token")
    protocol = value.get("protocol")
    if not isinstance(uri, str):
        raise ClientError("ready file URI must use ws://127.0.0.1")
    try:
        parsed_uri = urlsplit(uri)
        port = parsed_uri.port
    except ValueError as error:
        raise ClientError("ready file URI has an invalid port") from error
    if (
        parsed_uri.scheme != "ws"
        or parsed_uri.hostname != "127.0.0.1"
        or parsed_uri.username is not None
        or parsed_uri.password is not None
        or port is None
        or parsed_uri.path not in {"", "/"}
        or parsed_uri.query
        or parsed_uri.fragment
    ):
        raise ClientError("ready file URI must use ws://127.0.0.1:<port>")
    if not isinstance(token, str) or len(token) != 64:
        raise ClientError("ready file token must contain 256 bits as lowercase hex")
    if any(character not in "0123456789abcdef" for character in token):
        raise ClientError("ready file token must use lowercase hex")
    if protocol != SERVICE_PROTOCOL:
        raise ClientError(f"unsupported service protocol: {protocol!r}")
    models = value.get("models")
    model_identity = value.get("model_identity")
    if not isinstance(models, dict):
        models = {}
    if not isinstance(model_identity, dict):
        model_identity = {}
    return ReadyInfo(
        uri=uri,
        token=token,
        protocol=protocol,
        bounds={},
        models=models,
        model_identity=model_identity,
    )


async def run_websocket_session(
    ready: ReadyInfo,
    pcm_chunks: AsyncIterator[bytes],
    *,
    locale: str | None,
    on_event: Callable[[dict[str, object]], None],
    session_id: str | None = None,
    measurement: dict[str, float] | None = None,
    asr_audio_route: str = ASR_AUDIO_ROUTE_RAW,
    asr_model: str | None = None,
    asr_delay_ms: int | None = None,
    retain_diagnostic_audio: bool = True,
) -> dict[str, object]:
    if asr_audio_route not in ASR_AUDIO_ROUTES:
        raise ClientError(f"ASR audio route must be one of {sorted(ASR_AUDIO_ROUTES)}")
    identity = session_id or secrets.token_hex(16)
    async with connect(
        ready.uri,
        max_size=MAX_JSON_BYTES,
        max_queue=16,
        compression=None,
    ) as websocket:
        await websocket.send(
            json.dumps(
                {
                    "schema_version": 1,
                    "type": "client.hello",
                    "token": ready.token,
                },
                separators=(",", ":"),
            )
        )
        service_ready = _decode_event(await websocket.recv())
        _require_event(service_ready, "service.ready")
        bounds = service_ready.get("bounds")
        if not isinstance(bounds, dict):
            raise ClientError("service.ready does not contain bounds")
        max_frame_bytes = bounds.get("max_frame_bytes")
        if not isinstance(max_frame_bytes, int):
            raise ClientError("service.ready has invalid max_frame_bytes")
        max_turn_bytes = bounds.get("max_turn_bytes")
        if (
            not isinstance(max_turn_bytes, int)
            or max_turn_bytes < max_frame_bytes
            or max_turn_bytes % 2
        ):
            raise ClientError("service.ready has invalid max_turn_bytes")
        routing = service_ready.get("asr_audio_routing")
        if not isinstance(routing, dict):
            raise ClientError("service.ready does not contain ASR audio routing")
        available_routes = routing.get("available_routes")
        if (
            not isinstance(available_routes, list)
            or any(not isinstance(item, str) for item in available_routes)
        ):
            raise ClientError("service.ready has invalid ASR audio routing")
        if asr_audio_route not in available_routes:
            raise ClientError(f"ASR audio route is unavailable: {asr_audio_route}")
        model_routing = service_ready.get("asr_model_routing")
        if not isinstance(model_routing, dict):
            raise ClientError("service.ready does not contain ASR model routing")
        available_models = model_routing.get("available_models")
        default_model = model_routing.get("default_model")
        if (
            not isinstance(available_models, list)
            or any(not isinstance(item, str) for item in available_models)
            or not isinstance(default_model, str)
        ):
            raise ClientError("service.ready has invalid ASR model routing")
        selected_model = asr_model or default_model
        if selected_model not in available_models:
            raise ClientError(f"ASR model is unavailable: {selected_model}")
        models = service_ready.get("models")
        transcribers = models.get("transcribers") if isinstance(models, dict) else None
        selected_capabilities = (
            transcribers.get(selected_model) if isinstance(transcribers, dict) else None
        )
        if asr_delay_ms is not None:
            supported_delay_ms = (
                selected_capabilities.get("supported_delay_ms")
                if isinstance(selected_capabilities, dict)
                else None
            )
            if (
                not isinstance(supported_delay_ms, list)
                or asr_delay_ms not in supported_delay_ms
            ):
                raise ClientError(
                    f"ASR delay is unavailable for {selected_model}: {asr_delay_ms} ms"
                )
        model_limit_ms = (
            selected_capabilities.get("max_audio_duration_ms")
            if isinstance(selected_capabilities, dict)
            else None
        )
        if isinstance(model_limit_ms, int) and model_limit_ms > 0:
            max_turn_bytes = min(max_turn_bytes, model_limit_ms * 16_000 * 2 // 1_000)
        start_message: dict[str, object] = {
            "schema_version": 1,
            "type": "session.start",
            "session_id": identity,
            "locale": locale,
            "sample_rate_hz": 16_000,
            "encoding": "pcm_s16le",
            "channels": 1,
            "asr_model": selected_model,
            "asr_audio_route": asr_audio_route,
            "retain_diagnostic_audio": retain_diagnostic_audio,
        }
        if asr_delay_ms is not None:
            start_message["asr_delay_ms"] = asr_delay_ms
        await websocket.send(json.dumps(start_message, separators=(",", ":")))
        started = _decode_event(await websocket.recv())
        _require_event(started, "session.started")
        on_event(started)
        final_event: dict[str, object] | None = None

        async def send_audio() -> None:
            sent = 0
            capture_limit_reached = False
            async for chunk in pcm_chunks:
                if not isinstance(chunk, bytes) or not chunk or len(chunk) % 2:
                    raise ClientError("microphone source returned invalid PCM")
                if len(chunk) > max_frame_bytes:
                    raise ClientError("microphone chunk exceeds service max_frame_bytes")
                remaining = max_turn_bytes - sent
                if remaining <= 0:
                    capture_limit_reached = True
                    break
                if len(chunk) > remaining:
                    chunk = chunk[:remaining]
                    capture_limit_reached = True
                if sent == 0 and measurement is not None:
                    measurement["first_chunk_send_monotonic"] = time.monotonic()
                sent += len(chunk)
                await websocket.send(chunk)
                if sent == max_turn_bytes:
                    capture_limit_reached = True
                    break
            if sent == 0:
                raise ClientError("microphone returned no PCM")
            if capture_limit_reached:
                on_event(
                    {
                        "schema_version": 1,
                        "type": "client.capture_limit_reached",
                        "captured_bytes": sent,
                        "max_turn_bytes": max_turn_bytes,
                        "duration_ms": sent * 1_000 // (16_000 * 2),
                    }
                )
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "session.finish",
                        "session_id": identity,
                    },
                    separators=(",", ":"),
                )
            )
            if measurement is not None:
                measurement["finish_send_monotonic"] = time.monotonic()

        async def receive_events() -> None:
            nonlocal final_event
            async for message in websocket:
                payload = _decode_event(message)
                if (
                    measurement is not None
                    and payload.get("type") == "speech_timeline.update"
                ):
                    observed_at = time.monotonic()
                    measurement.setdefault("first_update_monotonic", observed_at)
                    transcript = payload.get("transcript")
                    if (
                        isinstance(transcript, dict)
                        and isinstance(transcript.get("revision"), int)
                        and transcript["revision"] > 0
                    ):
                        measurement.setdefault(
                            "first_transcript_update_monotonic", observed_at
                        )
                    vocal_affect = payload.get("vocal_affect")
                    if (
                        isinstance(vocal_affect, dict)
                        and isinstance(vocal_affect.get("raw_observations"), list)
                        and vocal_affect["raw_observations"]
                    ):
                        measurement.setdefault("first_affect_update_monotonic", observed_at)
                on_event(payload)
                if payload.get("type") == "error" and payload.get("terminal") is True:
                    raise ClientError(f"service failed: {payload.get('code')}")
                if payload.get("type") == "utterance.final":
                    if measurement is not None:
                        measurement["final_monotonic"] = time.monotonic()
                    final_event = payload
                    return

        try:
            async with asyncio.TaskGroup() as group:
                group.create_task(send_audio())
                group.create_task(receive_events())
        except* ClientError as group:
            raise group.exceptions[0]
        except* Exception as group:
            raise ClientError(f"WebSocket session failed: {type(group.exceptions[0]).__name__}")
        if final_event is None:
            raise ClientError("service closed before utterance.final")
        return final_event


async def microphone_chunks(
    device: str,
    *,
    chunk_ms: int,
    duration: float | None,
    save_wav: Path | None,
) -> AsyncIterator[bytes]:
    capture = capture_module()
    command = capture.capture_command(device)
    chunk_samples = max(1, capture.SAMPLE_RATE * chunk_ms // 1_000)
    max_samples = None if duration is None else int(capture.SAMPLE_RATE * duration)
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if process.stdout is None:
        capture.stop_capture(process)
        raise ClientError("microphone process did not provide PCM")
    captured = 0
    try:
        with capture.debug_wav_writer(save_wav) as writer:
            raw_sink = None if writer is None else writer.writeframesraw
            iterator = capture.raw_pcm_chunks(
                process.stdout,
                chunk_samples,
                max_samples,
                raw_sink=raw_sink,
            )
            while True:
                chunk = await asyncio.to_thread(_next_chunk, iterator)
                if chunk is None:
                    break
                captured += len(chunk)
                yield chunk
    finally:
        capture.stop_capture(process)
    if captured == 0:
        detail = ""
        if process.stderr is not None:
            detail = process.stderr.read().decode("utf-8", "replace").strip()
        suffix = f": {detail[:256]}" if detail else ""
        raise ClientError(f"microphone stopped before returning audio{suffix}")


def capture_module() -> ModuleType:
    name = "nextengine_voxtral_microphone_capture"
    existing = sys.modules.get(name)
    if existing is not None:
        return existing
    spec = importlib.util.spec_from_file_location(name, CAPTURE_SCRIPT)
    if spec is None or spec.loader is None:
        raise ClientError(f"cannot load microphone capture helpers: {CAPTURE_SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def validate_debug_wav(path: Path | None) -> Path | None:
    if path is None:
        return None
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
        raise ClientError("debug WAV must be outside the repository")
    if not resolved.parent.is_dir():
        raise ClientError("debug WAV parent directory does not exist")
    if resolved.exists():
        raise ClientError("refusing to overwrite an existing debug WAV")
    return resolved


def render_event(payload: dict[str, object], *, json_output: bool) -> None:
    if json_output:
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True), flush=True)
        return
    message_type = payload.get("type")
    if message_type == "session.started":
        print("listening ...", flush=True)
    elif message_type == "speech_timeline.update":
        transcript = payload.get("transcript", {})
        affect = payload.get("vocal_affect", {})
        text = transcript.get("text", "") if isinstance(transcript, dict) else ""
        segments = affect.get("segments", []) if isinstance(affect, dict) else []
        labels = []
        if isinstance(segments, list):
            labels = [str(item.get("label")) for item in segments if isinstance(item, dict)]
        suffix = f"  emotion={' → '.join(labels)}" if labels else ""
        print(f"transcript={text}{suffix}", flush=True)
    elif message_type == "utterance.final":
        print(
            f"final={payload.get('text', '')}  "
            f"expression={payload.get('observed_vocal_expression', 'unknown')}",
            flush=True,
        )
    elif message_type == "client.capture_limit_reached":
        print(
            f"capture limit reached at {payload.get('duration_ms', 0)} ms; finalizing ...",
            flush=True,
        )
    elif message_type == "error":
        print(f"error={payload.get('code')}: {payload.get('detail', '')}", file=sys.stderr)


def _next_chunk(iterator: object) -> bytes | None:
    try:
        return next(iterator)  # type: ignore[arg-type]
    except StopIteration:
        return None


def _decode_event(message: str | bytes) -> dict[str, object]:
    if isinstance(message, bytes):
        raise ClientError("service sent an unexpected binary frame")
    try:
        payload = json.loads(message)
    except json.JSONDecodeError as error:
        raise ClientError("service sent invalid JSON") from error
    if not isinstance(payload, dict) or payload.get("schema_version") != 1:
        raise ClientError("service event has an unsupported schema")
    return payload


def _require_event(payload: dict[str, object], expected: str) -> None:
    if payload.get("type") == "error":
        raise ClientError(f"service rejected the session: {payload.get('code')}")
    if payload.get("type") != expected:
        raise ClientError(f"expected {expected}, got {payload.get('type')}")
