from __future__ import annotations

from dataclasses import dataclass
import json
import re
from typing import Any


SCHEMA_VERSION = 1
MAX_JSON_BYTES = 64 * 1024
MAX_SESSION_ID_BYTES = 128
MAX_TOKEN_BYTES = 256
SESSION_ID_PATTERN = re.compile(r"^[A-Za-z0-9._:-]+$")


class ProtocolError(RuntimeError):
    def __init__(self, code: str, detail: str, *, terminal: bool = False) -> None:
        super().__init__(detail)
        self.code = code
        self.detail = detail
        self.terminal = terminal

    def event(self) -> dict[str, object]:
        return {
            "schema_version": SCHEMA_VERSION,
            "type": "error",
            "code": self.code,
            "terminal": self.terminal,
            "detail": self.detail[:512],
        }


@dataclass(frozen=True)
class ClientHello:
    token: str


@dataclass(frozen=True)
class SessionStart:
    session_id: str
    locale: str | None
    sample_rate_hz: int
    encoding: str
    channels: int


@dataclass(frozen=True)
class SessionFinish:
    session_id: str


@dataclass(frozen=True)
class SessionCancel:
    session_id: str


ClientMessage = ClientHello | SessionStart | SessionFinish | SessionCancel


def parse_client_message(payload: str | bytes) -> ClientMessage:
    raw = payload.encode("utf-8") if isinstance(payload, str) else payload
    if len(raw) > MAX_JSON_BYTES:
        raise ProtocolError("JSON_TOO_LARGE", "control message exceeds 65536 bytes", terminal=True)
    try:
        decoded = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ProtocolError("INVALID_UTF8", "control message is not valid UTF-8", terminal=True) from error
    try:
        value = json.loads(decoded)
    except json.JSONDecodeError as error:
        raise ProtocolError("INVALID_JSON", "control message is not valid JSON", terminal=True) from error
    if not isinstance(value, dict):
        raise ProtocolError("INVALID_MESSAGE", "control message must be a JSON object", terminal=True)
    if value.get("schema_version") != SCHEMA_VERSION:
        raise ProtocolError("UNSUPPORTED_VERSION", "schema_version must be 1", terminal=True)
    message_type = value.get("type")
    if message_type == "client.hello":
        _require_keys(value, {"schema_version", "type", "token"})
        token = _bounded_string(value.get("token"), "token", MAX_TOKEN_BYTES)
        return ClientHello(token=token)
    if message_type == "session.start":
        _require_keys(
            value,
            {
                "schema_version",
                "type",
                "session_id",
                "locale",
                "sample_rate_hz",
                "encoding",
                "channels",
            },
        )
        session_id = _session_id(value.get("session_id"))
        locale_value = value.get("locale")
        if locale_value is not None:
            locale_value = _bounded_string(locale_value, "locale", 32)
        sample_rate = _exact_integer(value.get("sample_rate_hz"), "sample_rate_hz")
        channels = _exact_integer(value.get("channels"), "channels")
        encoding = _bounded_string(value.get("encoding"), "encoding", 32)
        if (sample_rate, encoding, channels) != (16_000, "pcm_s16le", 1):
            raise ProtocolError(
                "UNSUPPORTED_AUDIO_FORMAT",
                "audio must be mono pcm_s16le at 16000 Hz",
                terminal=True,
            )
        return SessionStart(session_id, locale_value, sample_rate, encoding, channels)
    if message_type in {"session.finish", "session.cancel"}:
        _require_keys(value, {"schema_version", "type", "session_id"})
        session_id = _session_id(value.get("session_id"))
        if message_type == "session.finish":
            return SessionFinish(session_id)
        return SessionCancel(session_id)
    raise ProtocolError("UNKNOWN_MESSAGE_TYPE", "unsupported control message type", terminal=True)


def event(message_type: str, **fields: object) -> dict[str, object]:
    return {"schema_version": SCHEMA_VERSION, "type": message_type, **fields}


def encode_event(value: dict[str, object]) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if len(encoded.encode("utf-8")) > MAX_JSON_BYTES:
        raise ProtocolError("EVENT_TOO_LARGE", "service event exceeds 65536 bytes", terminal=True)
    return encoded


def _require_keys(value: dict[str, Any], expected: set[str]) -> None:
    actual = set(value)
    if actual != expected:
        raise ProtocolError(
            "INVALID_FIELDS",
            f"message fields do not match schema: missing={sorted(expected - actual)}, "
            f"extra={sorted(actual - expected)}",
            terminal=True,
        )


def _bounded_string(value: object, name: str, max_bytes: int) -> str:
    if not isinstance(value, str) or not value:
        raise ProtocolError("INVALID_FIELD", f"{name} must be a non-empty string", terminal=True)
    if len(value.encode("utf-8")) > max_bytes:
        raise ProtocolError("INVALID_FIELD", f"{name} is too large", terminal=True)
    return value


def _session_id(value: object) -> str:
    result = _bounded_string(value, "session_id", MAX_SESSION_ID_BYTES)
    if SESSION_ID_PATTERN.fullmatch(result) is None:
        raise ProtocolError("INVALID_SESSION_ID", "session_id contains unsupported characters", terminal=True)
    return result


def _exact_integer(value: object, name: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ProtocolError("INVALID_FIELD", f"{name} must be an integer", terminal=True)
    return value
