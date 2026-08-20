from __future__ import annotations

import asyncio
from http import HTTPStatus
import json
import time
import mimetypes
import os
from pathlib import Path
import secrets
from urllib.parse import unquote, urlsplit

from websockets.asyncio.server import Server, ServerConnection, serve
from websockets.datastructures import Headers
from websockets.exceptions import ConnectionClosed
from websockets.http11 import Request, Response

from . import SERVICE_PROTOCOL
from .diagnostic_audio import DiagnosticAudioStore
from .protocol import (
    ASR_AUDIO_ROUTE_RAW,
    MAX_JSON_BYTES,
    ClientHello,
    ProtocolError,
    SessionCancel,
    SessionFinish,
    SessionStart,
    encode_event,
    event,
    parse_client_message,
)
from .service import SpeechConnection, SpeechTimelineRuntime
from .session import SessionBounds, SessionError, SessionState


class SpeechTimelineWebSocketService:
    """Authenticated loopback-only transport for one resident speech runtime."""

    def __init__(
        self,
        runtime: SpeechTimelineRuntime,
        *,
        ready_file: Path,
        port: int = 0,
        bounds: SessionBounds | None = None,
        model_identity: dict[str, object] | None = None,
        diagnostic_audio: DiagnosticAudioStore | None = None,
    ) -> None:
        self.runtime = runtime
        self.ready_file = ready_file.expanduser().resolve()
        self.port = port
        self.bounds = bounds or SessionBounds()
        self.model_identity = model_identity or {}
        self.diagnostic_audio = diagnostic_audio
        self.token = secrets.token_hex(32)
        self._server: Server | None = None
        self._active_lock = asyncio.Lock()
        self._active: SpeechConnection | None = None
        self._ready_public: dict[str, object] | None = None

    @property
    def uri(self) -> str:
        if self._server is None or not self._server.sockets:
            raise RuntimeError("service is not listening")
        port = self._server.sockets[0].getsockname()[1]
        return f"ws://127.0.0.1:{port}"

    @property
    def dashboard_uri(self) -> str:
        if self._server is None or not self._server.sockets:
            raise RuntimeError("service is not listening")
        port = self._server.sockets[0].getsockname()[1]
        return f"http://127.0.0.1:{port}/"

    async def start(self) -> dict[str, object]:
        if self._server is not None:
            raise RuntimeError("service is already started")
        startup = self.runtime.start()
        if self.diagnostic_audio is not None:
            self.diagnostic_audio.start()
        self._server = await serve(
            self._handle,
            "127.0.0.1",
            self.port,
            max_size=MAX_JSON_BYTES,
            max_queue=16,
            compression=None,
            process_request=self._process_request,
        )
        self._ready_public = event(
            "service.ready",
            protocol=SERVICE_PROTOCOL,
            uri=self.uri,
            models=startup,
            model_identity=self.model_identity,
            asr_audio_routing={
                "default_route": ASR_AUDIO_ROUTE_RAW,
                "available_routes": list(self.runtime.available_asr_audio_routes),
                "vocal_activity_route": ASR_AUDIO_ROUTE_RAW,
                "vocal_affect_route": ASR_AUDIO_ROUTE_RAW,
            },
            bounds={
                "max_json_bytes": MAX_JSON_BYTES,
                "max_frame_bytes": self.bounds.max_frame_bytes,
                "max_turn_bytes": self.bounds.max_turn_bytes,
                "max_turn_duration_ms": (
                    self.bounds.max_turn_bytes * 1_000 // (16_000 * 2)
                ),
                "sample_rate_hz": 16_000,
                "encoding": "pcm_s16le",
                "channels": 1,
                "max_active_sessions": 1,
            },
            diagnostic_audio=(
                self.diagnostic_audio.capabilities() if self.diagnostic_audio is not None else {"enabled": False}
            ),
        )
        try:
            self._write_ready_file(
                {
                    "schema_version": 1,
                    "status": "ready",
                    "uri": self.uri,
                    "dashboard_uri": self.dashboard_uri,
                    "token": self.token,
                    "protocol": SERVICE_PROTOCOL,
                    "models": startup,
                    "model_identity": self.model_identity,
                    "asr_audio_routing": self._ready_public["asr_audio_routing"],
                }
            )
        except BaseException:
            self._server.close()
            await self._server.wait_closed()
            self._server = None
            self.runtime.close()
            raise
        return self._ready_public

    def _process_request(
        self, connection: ServerConnection, request: Request
    ) -> Response | None:
        del connection
        parsed = urlsplit(request.path)
        path = unquote(parsed.path)
        hosts = request.headers.get_all("Host")
        expected_host = urlsplit(self.dashboard_uri).netloc
        if hosts != [expected_host]:
            return self._http_error(HTTPStatus.MISDIRECTED_REQUEST)
        upgrades = request.headers.get_all("Upgrade")
        websocket_upgrade = len(upgrades) == 1 and upgrades[0].lower() == "websocket"
        if websocket_upgrade:
            if path not in {"", "/"} or parsed.query or parsed.fragment:
                return self._http_error(HTTPStatus.NOT_FOUND)
            origins = request.headers.get_all("Origin")
            allowed_origin = self.dashboard_uri.rstrip("/")
            if len(origins) > 1 or (origins and origins[0] != allowed_origin):
                return self._http_error(HTTPStatus.FORBIDDEN)
            return None

        if request.method != "GET":
            return self._http_error(HTTPStatus.METHOD_NOT_ALLOWED, allow="GET")
        if parsed.query or parsed.fragment:
            return self._http_error(HTTPStatus.BAD_REQUEST)
        if path == "/api/bootstrap":
            if self._ready_public is None:
                return self._http_error(HTTPStatus.SERVICE_UNAVAILABLE)
            body = json.dumps(
                {
                    "schema_version": 1,
                    "status": "ready",
                    "uri": self.uri,
                    "token": self.token,
                    "protocol": SERVICE_PROTOCOL,
                    "service": self._ready_public,
                },
                ensure_ascii=False,
                separators=(",", ":"),
            ).encode("utf-8")
            return self._http_response(
                HTTPStatus.OK,
                body,
                content_type="application/json; charset=utf-8",
                cache_control="no-store",
            )

        if path == "/api/diagnostic-audio":
            if self.diagnostic_audio is None:
                return self._http_error(HTTPStatus.NOT_FOUND)
            body = json.dumps(
                {
                    "schema_version": 1,
                    "records": [item.as_dict() for item in self.diagnostic_audio.list_records()],
                },
                ensure_ascii=False,
                separators=(",", ":"),
            ).encode("utf-8")
            return self._http_response(
                HTTPStatus.OK,
                body,
                content_type="application/json; charset=utf-8",
                cache_control="no-store",
            )

        audio_prefix = "/api/diagnostic-audio/"
        if path.startswith(audio_prefix):
            if self.diagnostic_audio is None:
                return self._http_error(HTTPStatus.NOT_FOUND)
            filename = path.removeprefix(audio_prefix)
            if not filename.endswith(".wav") or "/" in filename:
                return self._http_error(HTTPStatus.NOT_FOUND)
            if filename.endswith(".asr.wav"):
                record_id = filename.removesuffix(".asr.wav")
                variant = "asr_enhanced"
            else:
                record_id = filename[:-4]
                variant = "raw"
            payload = self.diagnostic_audio.read(record_id, variant=variant)
            if payload is None:
                return self._http_error(HTTPStatus.NOT_FOUND)
            return self._http_response(
                HTTPStatus.OK,
                payload,
                content_type="audio/wav",
                cache_control="no-store",
            )

        relative = "index.html" if path in {"", "/", "/index.html"} else path.lstrip("/")
        if (
            not relative
            or "\\" in relative
            or "\x00" in relative
            or any(part in {"", ".", ".."} for part in Path(relative).parts)
        ):
            return self._http_error(HTTPStatus.NOT_FOUND)
        static_root = Path(__file__).with_name("dashboard_static").resolve()
        target = (static_root / relative).resolve()
        try:
            target.relative_to(static_root)
        except ValueError:
            return self._http_error(HTTPStatus.NOT_FOUND)
        try:
            if target.is_symlink() or not target.is_file():
                return self._http_error(HTTPStatus.NOT_FOUND)
            body = target.read_bytes()
        except OSError:
            return self._http_error(HTTPStatus.NOT_FOUND)
        if len(body) > 1_048_576:
            return self._http_error(HTTPStatus.NOT_FOUND)
        content_type = mimetypes.guess_type(target.name)[0] or "application/octet-stream"
        if content_type.startswith("text/") or content_type in {
            "application/javascript",
            "application/json",
        }:
            content_type += "; charset=utf-8"
        cache_control = (
            "no-store"
            if target.name == "index.html"
            else "public, max-age=31536000, immutable"
        )
        return self._http_response(
            HTTPStatus.OK,
            body,
            content_type=content_type,
            cache_control=cache_control,
        )

    @classmethod
    def _http_error(cls, status: HTTPStatus, *, allow: str | None = None) -> Response:
        body = f"{status.value} {status.phrase}\n".encode("ascii")
        response = cls._http_response(
            status,
            body,
            content_type="text/plain; charset=utf-8",
            cache_control="no-store",
        )
        if allow is not None:
            response.headers["Allow"] = allow
        return response

    @staticmethod
    def _http_response(
        status: HTTPStatus,
        body: bytes,
        *,
        content_type: str,
        cache_control: str,
    ) -> Response:
        headers = Headers(
            {
                "Cache-Control": cache_control,
                "Content-Length": str(len(body)),
                "Content-Security-Policy": (
                    "default-src 'self'; script-src 'self'; style-src 'self'; "
                    "connect-src 'self' ws://127.0.0.1:*; worker-src 'self' blob:; "
                    "img-src 'self' data:; object-src 'none'; base-uri 'none'; "
                    "frame-ancestors 'none'"
                ),
                "Content-Type": content_type,
                "Referrer-Policy": "no-referrer",
                "X-Content-Type-Options": "nosniff",
                "X-Frame-Options": "DENY",
            }
        )
        return Response(status.value, status.phrase, headers, body)

    async def serve_forever(self) -> None:
        if self._server is None:
            await self.start()
        assert self._server is not None
        await self._server.serve_forever()

    async def close(self) -> None:
        if self._server is not None:
            self._server.close()
            await self._server.wait_closed()
            self._server = None
        self.runtime.close()
        try:
            self.ready_file.unlink(missing_ok=True)
        except OSError:
            pass

    async def _claim(self, connection: SpeechConnection) -> bool:
        async with self._active_lock:
            if self._active is connection:
                return True
            if self._active is not None:
                return False
            self._active = connection
            return True

    async def _release(self, connection: SpeechConnection) -> None:
        async with self._active_lock:
            if self._active is connection:
                self._active = None

    async def _handle(self, websocket: ServerConnection) -> None:
        connection = SpeechConnection(
            self.runtime,
            self.bounds,
            self._claim,
            self._release,
            diagnostic_audio=self.diagnostic_audio,
        )
        sender: asyncio.Task[None] | None = None
        try:
            try:
                first = await asyncio.wait_for(websocket.recv(), timeout=5)
                if isinstance(first, bytes):
                    raise ProtocolError("AUTH_REQUIRED", "first message must be client.hello", terminal=True)
                hello = parse_client_message(first)
                if not isinstance(hello, ClientHello) or not secrets.compare_digest(
                    hello.token, self.token
                ):
                    raise ProtocolError("AUTH_FAILED", "authentication failed", terminal=True)
                connection.session.authenticate()
                assert self._ready_public is not None
                await websocket.send(encode_event(self._ready_public))
                sender = asyncio.create_task(self._send_events(websocket, connection))
                async for message in websocket:
                    if connection.session.state is SessionState.FAILED:
                        await connection.terminal_ready.wait()
                        await connection.events.join()
                        break
                    if isinstance(message, bytes):
                        try:
                            await connection.append_pcm(message)
                        except SessionError as error:
                            if error.code == "TURN_TOO_LARGE":
                                await connection.fail_input(error.code, str(error))
                            else:
                                await connection.events.put(
                                    event(
                                        "error",
                                        code=error.code,
                                        terminal=False,
                                        detail=str(error)[:512],
                                    )
                                )
                        await asyncio.sleep(0)
                        if connection.session.state is SessionState.FAILED:
                            await connection.terminal_ready.wait()
                            await connection.events.join()
                            break
                        continue
                    client_message = parse_client_message(message)
                    if isinstance(client_message, SessionStart):
                        await connection.start(
                            client_message.session_id,
                            client_message.locale,
                            vad_calibration=client_message.vad_calibration,
                            asr_audio_route=client_message.asr_audio_route,
                        )
                    elif isinstance(client_message, SessionFinish):
                        await connection.finish(client_message.session_id)
                    elif isinstance(client_message, SessionCancel):
                        await connection.cancel(client_message.session_id)
                    else:
                        raise ProtocolError(
                            "INVALID_STATE", "client.hello is accepted only once", terminal=True
                        )
            except (ProtocolError, SessionError) as error:
                if isinstance(error, ProtocolError):
                    payload = error.event()
                else:
                    payload = event("error", code=error.code, terminal=True, detail=str(error)[:512])
                if sender is None:
                    await websocket.send(encode_event(payload))
                else:
                    await connection.events.put(payload)
                    await connection.events.join()
            except asyncio.TimeoutError:
                await websocket.send(
                    encode_event(event("error", code="AUTH_TIMEOUT", terminal=True, detail="client.hello timeout"))
                )
            except Exception as error:
                payload = event(
                    "error",
                    code="INTERNAL_ERROR",
                    terminal=True,
                    detail=f"service operation failed: {type(error).__name__}",
                )
                if sender is None:
                    await websocket.send(encode_event(payload))
                else:
                    await connection.events.put(payload)
                    await connection.events.join()
        except ConnectionClosed:
            pass
        finally:
            await connection.disconnect()
            if sender is not None:
                sender.cancel()
                await asyncio.gather(sender, return_exceptions=True)

    @staticmethod
    async def _send_events(
        websocket: ServerConnection, connection: SpeechConnection
    ) -> None:
        while True:
            payload = await connection.events.get()
            try:
                encode_started = time.perf_counter_ns()
                encoded = encode_event(payload)
                encode_ms = round((time.perf_counter_ns() - encode_started) / 1_000_000)
                send_started = time.perf_counter_ns()
                await websocket.send(encoded)
                send_ms = round((time.perf_counter_ns() - send_started) / 1_000_000)
                connection.record_event_sent(
                    payload_bytes=len(encoded.encode("utf-8")),
                    encode_ms=encode_ms,
                    send_ms=send_ms,
                )
            except ProtocolError as error:
                # Keep an oversized update from killing the sender task and
                # leaving the client waiting forever for utterance.final.
                fallback = event(
                    "error",
                    code=error.code,
                    terminal=True,
                    detail=str(error)[:512],
                )
                try:
                    await websocket.send(encode_event(fallback))
                except ConnectionClosed:
                    return
                return
            finally:
                connection.events.task_done()

    def _write_ready_file(self, payload: dict[str, object]) -> None:
        encoded = json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True).encode("utf-8")
        descriptor = os.open(
            self.ready_file,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL,
            0o600,
        )
        try:
            with os.fdopen(descriptor, "wb") as stream:
                stream.write(encoded)
                stream.flush()
                os.fsync(stream.fileno())
        except BaseException:
            try:
                self.ready_file.unlink(missing_ok=True)
            finally:
                raise
