from __future__ import annotations

import asyncio
import json
import os
from pathlib import Path
import secrets

from websockets.asyncio.server import Server, ServerConnection, serve
from websockets.exceptions import ConnectionClosed

from . import SERVICE_PROTOCOL
from .protocol import (
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
from .session import SessionBounds, SessionError


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
    ) -> None:
        self.runtime = runtime
        self.ready_file = ready_file.expanduser().resolve()
        self.port = port
        self.bounds = bounds or SessionBounds()
        self.model_identity = model_identity or {}
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

    async def start(self) -> dict[str, object]:
        if self._server is not None:
            raise RuntimeError("service is already started")
        startup = self.runtime.start()
        self._server = await serve(
            self._handle,
            "127.0.0.1",
            self.port,
            max_size=MAX_JSON_BYTES,
            max_queue=16,
            compression=None,
        )
        self._ready_public = event(
            "service.ready",
            protocol=SERVICE_PROTOCOL,
            uri=self.uri,
            models=startup,
            model_identity=self.model_identity,
            bounds={
                "max_json_bytes": MAX_JSON_BYTES,
                "max_frame_bytes": self.bounds.max_frame_bytes,
                "max_turn_bytes": self.bounds.max_turn_bytes,
                "sample_rate_hz": 16_000,
                "encoding": "pcm_s16le",
                "channels": 1,
                "max_active_sessions": 1,
            },
        )
        try:
            self._write_ready_file(
                {
                    "schema_version": 1,
                    "status": "ready",
                    "uri": self.uri,
                    "token": self.token,
                    "protocol": SERVICE_PROTOCOL,
                    "models": startup,
                    "model_identity": self.model_identity,
                }
            )
        except BaseException:
            self._server.close()
            await self._server.wait_closed()
            self._server = None
            self.runtime.close()
            raise
        return self._ready_public

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
        connection = SpeechConnection(self.runtime, self.bounds, self._claim, self._release)
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
                    if isinstance(message, bytes):
                        try:
                            await connection.append_pcm(message)
                        except SessionError as error:
                            await connection.events.put(
                                event("error", code=error.code, terminal=False, detail=str(error)[:512])
                            )
                        continue
                    client_message = parse_client_message(message)
                    if isinstance(client_message, SessionStart):
                        await connection.start(client_message.session_id, client_message.locale)
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
                await websocket.send(encode_event(payload))
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
