from __future__ import annotations

import asyncio
import json
from pathlib import Path
import stat
import tempfile
import time
import unittest
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from websockets.asyncio.client import connect
from websockets.exceptions import InvalidStatus

from nextengine_speech_timeline.adapters.base import AffectObservation
from nextengine_speech_timeline.adapters.voxtral_transcribe_cpp import TranscriptRevision
from nextengine_speech_timeline.benchmark import benchmark_service
from nextengine_speech_timeline.microphone_client import ReadyInfo, run_websocket_session
from nextengine_speech_timeline.metrics import ModelJobMetric
from nextengine_speech_timeline.protocol import MAX_JSON_BYTES, encode_event, event
from nextengine_speech_timeline.service import SpeechConnection, SpeechTimelineRuntime
from nextengine_speech_timeline.session import SessionBounds
from nextengine_speech_timeline.transport_websocket import SpeechTimelineWebSocketService


class FakeTranscriberSession:
    def __init__(self, owner: FakeTranscriber) -> None:
        self.owner = owner
        self.revision = 0
        self.finalize_count = 0
        self.closed = False

    def push_pcm(self, samples: object) -> TranscriptRevision:
        if self.owner.push_delay:
            time.sleep(self.owner.push_delay)
        self.revision += 1
        self.owner.push_count += 1
        return TranscriptRevision(
            revision=self.revision,
            full_text="промежуточный текст",
            committed_text="промежуточный ",
            tentative_text="текст",
            final=False,
        )

    def finish(self) -> TranscriptRevision:
        self.finalize_count += 1
        self.revision += 1
        return TranscriptRevision(
            revision=self.revision,
            full_text="готово",
            committed_text="готово",
            tentative_text="",
            final=True,
        )

    def cancel(self) -> None:
        self.closed = True

    def close(self) -> None:
        self.closed = True


class FakeTranscriber:
    def __init__(self) -> None:
        self.load_count = 0
        self.start_count = 0
        self.push_count = 0
        self.push_delay = 0.0
        self.sessions: list[FakeTranscriberSession] = []

    def load(self) -> dict[str, int]:
        if self.load_count == 0:
            self.load_count = 1
        return {"load_count": self.load_count, "elapsed_ms": 0}

    def warmup(self) -> dict[str, int]:
        return {"warmup_count": 1, "elapsed_ms": 0}

    def capabilities(self) -> dict[str, object]:
        return {"adapter_id": "fake-asr/1", "timing_precision": "utterance"}

    def start(self, config: object) -> FakeTranscriberSession:
        self.start_count += 1
        session = FakeTranscriberSession(self)
        self.sessions.append(session)
        return session

    def close(self) -> None:
        return None


class FakeAffect:
    def __init__(self) -> None:
        self.load_count = 0
        self.observe_count = 0

    def load(self) -> dict[str, int]:
        if self.load_count == 0:
            self.load_count = 1
        return {"load_count": self.load_count, "elapsed_ms": 0}

    def warmup(self) -> dict[str, int]:
        return {"warmup_count": 1, "elapsed_ms": 0}

    def capabilities(self) -> dict[str, object]:
        return {"adapter_id": "fake-affect/1", "semantics": "observed_expression"}

    def observe(self, window: object) -> AffectObservation:
        self.observe_count += 1
        return AffectObservation(
            model_id="fake-emotion",
            model_revision="revision",
            start_sample=window.start_sample,
            end_sample=window.end_sample,
            source_revision=window.source_revision,
            scores={"neutral": 0.8, "angry": 0.2},
            top_label="neutral",
            inference_elapsed_ms=1,
        )


class WebSocketServiceTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.ready_file = Path(self.temp.name) / "ready.json"
        self.transcriber = FakeTranscriber()
        self.affect = FakeAffect()
        self.service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime(self.transcriber, self.affect),
            ready_file=self.ready_file,
            port=0,
        )
        await self.service.start()
        self.ready = json.loads(self.ready_file.read_text(encoding="utf-8"))

    async def asyncTearDown(self) -> None:
        await self.service.close()
        self.temp.cleanup()

    async def authenticate(self, websocket: object) -> dict[str, object]:
        await websocket.send(
            json.dumps(
                {
                    "schema_version": 1,
                    "type": "client.hello",
                    "token": self.ready["token"],
                }
            )
        )
        return json.loads(await websocket.recv())

    async def run_turn(self, session_id: str) -> list[dict[str, object]]:
        async with connect(self.service.uri, compression=None) as websocket:
            ready = await self.authenticate(websocket)
            self.assertEqual(ready["type"], "service.ready")
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "session.start",
                        "session_id": session_id,
                        "locale": "ru",
                        "sample_rate_hz": 16_000,
                        "encoding": "pcm_s16le",
                        "channels": 1,
                    }
                )
            )
            started = json.loads(await websocket.recv())
            self.assertEqual(started["type"], "session.started")
            await websocket.send(b"\0\0" * 4_000)
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "session.finish",
                        "session_id": session_id,
                    }
                )
            )
            events = []
            while True:
                message = json.loads(await asyncio.wait_for(websocket.recv(), 2))
                events.append(message)
                if message["type"] == "utterance.final":
                    return events

    async def test_ready_file_is_private_and_two_sessions_do_not_reload_models(self) -> None:
        self.assertEqual(stat.S_IMODE(self.ready_file.stat().st_mode), 0o600)
        self.assertEqual(self.ready["dashboard_uri"], self.service.dashboard_uri)
        first = await self.run_turn("turn-1")
        second = await self.run_turn("turn-2")
        self.assertTrue(any(item["type"] == "speech_timeline.update" for item in first))
        final = next(item for item in second if item["type"] == "utterance.final")
        self.assertEqual(final["text"], "готово")
        self.assertIn("metrics", final)
        self.assertEqual(final["spans"], [])
        self.assertEqual(self.transcriber.load_count, 1)
        self.assertEqual(self.affect.load_count, 1)
        self.assertEqual(self.transcriber.start_count, 2)
        self.assertEqual([item.finalize_count for item in self.transcriber.sessions], [1, 1])

    async def test_dashboard_and_private_bootstrap_are_served_same_origin(self) -> None:
        def get(path: str) -> tuple[int, dict[str, str], bytes]:
            with urlopen(self.service.dashboard_uri + path, timeout=2) as response:
                return response.status, dict(response.headers.items()), response.read()

        status, headers, index = await asyncio.to_thread(get, "")
        self.assertEqual(status, 200)
        self.assertIn(b'<div id="app"></div>', index)
        self.assertIn("default-src 'self'", headers["Content-Security-Policy"])
        self.assertEqual(headers["X-Frame-Options"], "DENY")

        status, headers, raw = await asyncio.to_thread(get, "api/bootstrap")
        bootstrap = json.loads(raw)
        self.assertEqual(status, 200)
        self.assertEqual(headers["Cache-Control"], "no-store")
        self.assertEqual(bootstrap["token"], self.ready["token"])
        self.assertEqual(bootstrap["service"]["type"], "service.ready")
        self.assertEqual(
            bootstrap["service"]["bounds"]["max_turn_duration_ms"], 30_000
        )

    async def test_dashboard_rejects_unsafe_http_and_foreign_websocket_origins(
        self,
    ) -> None:
        def fetch(request: Request) -> int:
            try:
                with urlopen(request, timeout=2) as response:
                    return response.status
            except HTTPError as error:
                return error.code

        traversal = Request(self.service.dashboard_uri + "%2e%2e/pyproject.toml")
        self.assertEqual(await asyncio.to_thread(fetch, traversal), 404)
        post = Request(self.service.dashboard_uri + "api/bootstrap", method="POST")
        self.assertEqual(await asyncio.to_thread(fetch, post), 405)
        rebound = Request(
            self.service.dashboard_uri + "api/bootstrap",
            headers={"Host": "malicious.example"},
        )
        self.assertEqual(await asyncio.to_thread(fetch, rebound), 421)

        async with connect(
            self.service.uri,
            compression=None,
            origin=self.service.dashboard_uri.rstrip("/"),
        ) as websocket:
            ready = await self.authenticate(websocket)
            self.assertEqual(ready["type"], "service.ready")
        with self.assertRaises(InvalidStatus):
            async with connect(
                self.service.uri,
                compression=None,
                origin="http://malicious.example",
            ):
                pass

    async def test_bad_auth_and_protocol_version_fail_closed(self) -> None:
        async with connect(self.service.uri, compression=None) as websocket:
            await websocket.send(
                json.dumps(
                    {"schema_version": 1, "type": "client.hello", "token": "wrong"}
                )
            )
            error = json.loads(await websocket.recv())
            self.assertEqual(error["code"], "AUTH_FAILED")
            self.assertTrue(error["terminal"])
        async with connect(self.service.uri, compression=None) as websocket:
            await self.authenticate(websocket)
            await websocket.send(json.dumps({"schema_version": 2, "type": "session.finish", "session_id": "x"}))
            error = json.loads(await websocket.recv())
            self.assertEqual(error["code"], "UNSUPPORTED_VERSION")

    async def test_post_final_binary_frame_is_rejected(self) -> None:
        async with connect(self.service.uri, compression=None) as websocket:
            await self.authenticate(websocket)
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "session.start",
                        "session_id": "turn",
                        "locale": "ru",
                        "sample_rate_hz": 16_000,
                        "encoding": "pcm_s16le",
                        "channels": 1,
                    }
                )
            )
            await websocket.recv()
            await websocket.send(b"\0\0" * 100)
            await websocket.send(json.dumps({"schema_version": 1, "type": "session.finish", "session_id": "turn"}))
            while json.loads(await websocket.recv())["type"] != "utterance.final":
                pass
            await websocket.send(b"\0\0")
            error = json.loads(await websocket.recv())
            self.assertEqual(error["code"], "PCM_NOT_ACCEPTED")
            self.assertFalse(error["terminal"])

    async def test_disconnect_releases_the_single_session_slot(self) -> None:
        websocket = await connect(self.service.uri, compression=None)
        await self.authenticate(websocket)
        await websocket.send(
            json.dumps(
                {
                    "schema_version": 1,
                    "type": "session.start",
                    "session_id": "abandoned",
                    "locale": "ru",
                    "sample_rate_hz": 16_000,
                    "encoding": "pcm_s16le",
                    "channels": 1,
                }
            )
        )
        await websocket.recv()
        await websocket.close()
        await asyncio.sleep(0.05)
        events = await self.run_turn("replacement")
        self.assertEqual(events[-1]["type"], "utterance.final")

    async def test_duplicate_start_oversized_frame_and_second_active_session_fail_closed(self) -> None:
        first = await connect(self.service.uri, compression=None)
        await self.authenticate(first)
        start = {
            "schema_version": 1,
            "type": "session.start",
            "session_id": "owner",
            "locale": "ru",
            "sample_rate_hz": 16_000,
            "encoding": "pcm_s16le",
            "channels": 1,
        }
        await first.send(json.dumps(start))
        await first.recv()
        await first.send(b"\0" * 32_002)
        oversized = json.loads(await first.recv())
        self.assertEqual(oversized["code"], "FRAME_TOO_LARGE")

        second = await connect(self.service.uri, compression=None)
        await self.authenticate(second)
        busy_start = dict(start, session_id="other")
        await second.send(json.dumps(busy_start))
        busy = json.loads(await second.recv())
        self.assertEqual(busy["code"], "SERVICE_BUSY")
        await second.close()

        await first.send(json.dumps(start))
        duplicate = json.loads(await first.recv())
        self.assertEqual(duplicate["code"], "DUPLICATE_START")
        await first.close()

    async def test_model_neutral_client_streams_chunks_and_receives_final_timeline(self) -> None:
        async def chunks():
            yield b"\0\0" * 2_000
            yield b"\0\0" * 2_000

        received: list[dict[str, object]] = []
        final = await run_websocket_session(
            ReadyInfo(
                uri=self.service.uri,
                token=self.ready["token"],
                protocol="nextengine.speech-timeline/1",
                bounds={},
            ),
            chunks(),
            locale="ru",
            on_event=received.append,
            session_id="client-turn",
        )
        self.assertEqual(final["text"], "готово")
        self.assertTrue(any(item["type"] == "speech_timeline.update" for item in received))
        self.assertEqual(self.transcriber.push_count, 2)

    async def test_unpaced_benchmark_reports_two_resident_runs_without_content(self) -> None:
        report = await benchmark_service(
            ReadyInfo(
                uri=self.service.uri,
                token=self.ready["token"],
                protocol="nextengine.speech-timeline/1",
                bounds={},
                models={"fake": "resident"},
                model_identity={"revision": "exact"},
            ),
            b"\0\0" * 4_000,
            4_000,
            mode="unpaced",
            runs=2,
            chunk_ms=125,
        )
        self.assertEqual(len(report["runs"]), 2)
        self.assertEqual(report["privacy"], "audio_and_transcript_omitted")
        serialized = json.dumps(report, ensure_ascii=False)
        self.assertNotIn("готово", serialized)
        self.assertEqual(self.transcriber.load_count, 1)
        self.assertEqual(self.transcriber.start_count, 2)

    async def test_final_metrics_remain_within_event_bound_after_long_turn(self) -> None:
        connection = SpeechConnection(
            self.service.runtime,
            SessionBounds(),
            self.service._claim,
            self.service._release,
        )
        connection._job_metrics = [
            ModelJobMetric(1, "voxtral_push", index * 1_000, (index + 1) * 1_000, 3, 40)
            for index in range(1_000)
        ]

        metrics = connection.metrics_payload()
        self.assertTrue(metrics["jobs_truncated"])
        self.assertEqual(metrics["jobs_total"], 1_000)
        self.assertEqual(len(metrics["jobs"]), 64)
        encoded = encode_event(
            event(
                "utterance.final",
                session_id="long-turn",
                text="готово",
                timing_precision="utterance",
                observed_vocal_expression="neutral",
                alignment_grade="utterance",
                spans=[],
                metrics=metrics,
            )
        )
        self.assertLessEqual(len(encoded.encode("utf-8")), MAX_JSON_BYTES)

    async def test_transport_backpressure_absorbs_short_unpaced_burst(self) -> None:
        self.transcriber.push_delay = 0.05

        async def chunks():
            for _ in range(10):
                yield b"\0\0" * 100

        final = await run_websocket_session(
            ReadyInfo(
                uri=self.service.uri,
                token=self.ready["token"],
                protocol="nextengine.speech-timeline/1",
                bounds={},
            ),
            chunks(),
            locale="ru",
            on_event=lambda _: None,
            session_id="backpressured",
        )
        self.assertEqual(final["text"], "готово")
        self.assertEqual(self.transcriber.push_count, 10)

        self.transcriber.push_delay = 0.0
        events = await self.run_turn("after-overload")
        self.assertEqual(events[-1]["type"], "utterance.final")

    async def test_turn_too_large_is_one_terminal_error(self) -> None:
        ready_file = Path(self.temp.name) / "small-ready.json"
        service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime(FakeTranscriber(), FakeAffect()),
            ready_file=ready_file,
            port=0,
            bounds=SessionBounds(max_frame_bytes=320, max_turn_bytes=640),
        )
        await service.start()
        ready = json.loads(ready_file.read_text(encoding="utf-8"))
        try:
            async with connect(service.uri, compression=None) as websocket:
                await websocket.send(
                    json.dumps(
                        {
                            "schema_version": 1,
                            "type": "client.hello",
                            "token": ready["token"],
                        }
                    )
                )
                await websocket.recv()
                await websocket.send(
                    json.dumps(
                        {
                            "schema_version": 1,
                            "type": "session.start",
                            "session_id": "too-long",
                            "locale": "ru",
                            "sample_rate_hz": 16_000,
                            "encoding": "pcm_s16le",
                            "channels": 1,
                        }
                    )
                )
                await websocket.recv()
                await websocket.send(b"\0" * 320)
                await websocket.send(b"\0" * 320)
                await websocket.send(b"\0\0")
                errors = []
                async for message in websocket:
                    payload = json.loads(message)
                    if payload["type"] == "error":
                        errors.append(payload)
        finally:
            await service.close()
        self.assertEqual(len(errors), 1)
        self.assertEqual(errors[0]["code"], "TURN_TOO_LARGE")
        self.assertTrue(errors[0]["terminal"])

    async def test_client_auto_finalizes_at_server_turn_limit(self) -> None:
        ready_file = Path(self.temp.name) / "bounded-ready.json"
        transcriber = FakeTranscriber()
        service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime(transcriber, FakeAffect()),
            ready_file=ready_file,
            port=0,
            bounds=SessionBounds(max_frame_bytes=320, max_turn_bytes=640),
        )
        await service.start()
        ready = json.loads(ready_file.read_text(encoding="utf-8"))

        async def chunks():
            yield b"\0" * 320
            yield b"\0" * 320
            yield b"\0" * 320

        received: list[dict[str, object]] = []
        try:
            final = await run_websocket_session(
                ReadyInfo(
                    uri=service.uri,
                    token=ready["token"],
                    protocol="nextengine.speech-timeline/1",
                    bounds={},
                ),
                chunks(),
                locale="ru",
                on_event=received.append,
                session_id="bounded-client",
            )
        finally:
            await service.close()
        limit = next(
            item for item in received if item["type"] == "client.capture_limit_reached"
        )
        self.assertEqual(limit["captured_bytes"], 640)
        self.assertEqual(final["text"], "готово")
        self.assertEqual(transcriber.push_count, 2)


if __name__ == "__main__":
    unittest.main()
