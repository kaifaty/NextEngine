from __future__ import annotations

import asyncio
import hashlib
import json
from pathlib import Path
import stat
import tempfile
import time
import unittest
import wave
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from websockets.asyncio.client import connect
from websockets.exceptions import InvalidStatus

from nextengine_speech_timeline.adapters.base import AffectObservation
from nextengine_speech_timeline.adapters.voxtral_transcribe_cpp import TranscriptRevision
from nextengine_speech_timeline.benchmark import (
    benchmark_service,
    evaluate_affect_calibration,
    load_affect_calibration_manifest,
)
from nextengine_speech_timeline.diagnostic_audio import DiagnosticAudioStore
from nextengine_speech_timeline.microphone_client import (
    ClientError,
    ReadyInfo,
    run_websocket_session,
)
from nextengine_speech_timeline.metrics import ModelJobMetric
from nextengine_speech_timeline.protocol import MAX_JSON_BYTES, encode_event, event
from nextengine_speech_timeline.service import (
    SpeechConnection,
    SpeechTimelineRuntime,
    _timeline_event,
)
from nextengine_speech_timeline.session import SessionBounds
from nextengine_speech_timeline.timeline import SpeechTimeline
from nextengine_speech_timeline.transport_websocket import SpeechTimelineWebSocketService


class FakeTranscriberSession:
    def __init__(self, owner: FakeTranscriber) -> None:
        self.owner = owner
        self.revision = 0
        self.finalize_count = 0
        self.closed = False
        self.pushed_samples: list[object] = []

    def push_pcm(self, samples: object) -> TranscriptRevision:
        self.pushed_samples.append(samples)
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
            full_text=self.owner.final_text,
            committed_text=self.owner.final_text,
            tentative_text="",
            final=True,
        )

    def cancel(self) -> None:
        self.closed = True

    def close(self) -> None:
        self.closed = True


class FakeTranscriber:
    def __init__(
        self,
        adapter_id: str = "fake-asr/1",
        final_text: str = "готово",
        *,
        supported_delay_ms: tuple[int, ...] = (),
        configured_delay_ms: int = 0,
    ) -> None:
        self.adapter_id = adapter_id
        self.final_text = final_text
        self.load_count = 0
        self.start_count = 0
        self.push_count = 0
        self.push_delay = 0.0
        self.sessions: list[FakeTranscriberSession] = []
        self.configs: list[object] = []
        self.supported_delay_ms = supported_delay_ms
        self.configured_delay_ms = configured_delay_ms

    def load(self) -> dict[str, int]:
        if self.load_count == 0:
            self.load_count = 1
        return {"load_count": self.load_count, "elapsed_ms": 0}

    def warmup(self) -> dict[str, int]:
        return {"warmup_count": 1, "elapsed_ms": 0}

    def capabilities(self) -> dict[str, object]:
        return {
            "adapter_id": self.adapter_id,
            "timing_precision": "utterance",
            "supported_delay_ms": self.supported_delay_ms,
            "configured_delay_ms": self.configured_delay_ms,
        }

    def start(self, config: object) -> FakeTranscriberSession:
        self.start_count += 1
        self.configs.append(config)
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


class FakeAudioPreprocessor:
    def __init__(self) -> None:
        self.load_count = 0
        self.reset_count = 0
        self.process_count = 0
        self.flush_count = 0
        self.reset_routes: list[tuple[str, float | None]] = []

    def load(self) -> dict[str, int]:
        self.load_count += 1
        return {"load_count": self.load_count, "elapsed_ms": 0}

    def warmup(self) -> dict[str, int]:
        return {"elapsed_ms": 0}

    def capabilities(self) -> dict[str, object]:
        return {
            "adapter_id": "fake-audio-preprocessor/1",
            "routes": ["asr"],
            "asr_audio_routes": ["enhanced", "gain_only", "whisper"],
        }

    def reset(
        self,
        route: str = "enhanced",
        *,
        noise_floor_dbfs: float | None = None,
    ) -> None:
        self.reset_count += 1
        self.reset_routes.append((route, noise_floor_dbfs))

    def process_pcm(self, pcm: bytes) -> bytes:
        self.process_count += 1
        return b"\x00\x10" * (len(pcm) // 2)

    def flush(self) -> bytes:
        self.flush_count += 1
        return b""

    def close(self) -> None:
        return None


class WebSocketServiceTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.ready_file = Path(self.temp.name) / "ready.json"
        self.diagnostic_audio_root = Path(self.temp.name) / "diagnostic-audio"
        self.transcriber = FakeTranscriber()
        self.affect = FakeAffect()
        self.service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime(self.transcriber, self.affect),
            ready_file=self.ready_file,
            port=0,
            diagnostic_audio=DiagnosticAudioStore(self.diagnostic_audio_root),
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

    async def test_session_selects_one_resident_asr_model_without_reloading(self) -> None:
        voxtral = FakeTranscriber("fake-voxtral/1", "потоковый результат")
        gigaam = FakeTranscriber("fake-gigaam/1", "финальный результат гигаам")
        service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime(
                {"voxtral-realtime": voxtral, "gigaam-v3-e2e-rnnt": gigaam},
                FakeAffect(),
                default_transcriber="voxtral-realtime",
            ),
            ready_file=Path(self.temp.name) / "selectable-asr-ready.json",
            port=0,
        )
        ready = await service.start()
        try:
            self.assertEqual(
                ready["asr_model_routing"],
                {
                    "default_model": "voxtral-realtime",
                    "available_models": ["voxtral-realtime", "gigaam-v3-e2e-rnnt"],
                },
            )
            events = await self._run_turn_against(
                service,
                "gigaam-turn",
                asr_model="gigaam-v3-e2e-rnnt",
            )
            started = next(item for item in events if item["type"] == "session.started")
            final = next(item for item in events if item["type"] == "utterance.final")
            self.assertEqual(started["asr_model"], "gigaam-v3-e2e-rnnt")
            self.assertEqual(final["asr_model"], "gigaam-v3-e2e-rnnt")
            self.assertEqual(final["text"], "финальный результат гигаам")
            self.assertEqual(voxtral.load_count, 1)
            self.assertEqual(gigaam.load_count, 1)
            self.assertEqual(voxtral.start_count, 0)
            self.assertEqual(gigaam.start_count, 1)
        finally:
            await service.close()

    async def test_session_selects_supported_asr_delay_without_reloading(self) -> None:
        voxtral = FakeTranscriber(
            "fake-voxtral/1",
            supported_delay_ms=(480, 960, 2_400),
            configured_delay_ms=480,
        )
        service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime(voxtral, FakeAffect()),
            ready_file=Path(self.temp.name) / "selectable-delay-ready.json",
            port=0,
        )
        await service.start()
        try:
            for index, delay_ms in enumerate((960, 2_400), start=1):
                events = await self._run_turn_against(
                    service,
                    f"delay-turn-{index}",
                    asr_delay_ms=delay_ms,
                )
                started = next(item for item in events if item["type"] == "session.started")
                final = next(item for item in events if item["type"] == "utterance.final")
                self.assertEqual(started["asr_delay_ms"], delay_ms)
                self.assertEqual(final["metrics"]["asr"]["selected_delay_ms"], delay_ms)
                self.assertEqual(getattr(voxtral.configs[-1], "delay_ms"), delay_ms)
            self.assertEqual(voxtral.load_count, 1)
            self.assertEqual(voxtral.start_count, 2)

            async def chunks():
                yield b"\0\0" * 4_000

            received: list[dict[str, object]] = []
            await run_websocket_session(
                ReadyInfo(
                    uri=service.uri,
                    token=service.token,
                    protocol="nextengine.speech-timeline/1",
                    bounds={},
                ),
                chunks(),
                locale="ru",
                on_event=received.append,
                session_id="delay-client-turn",
                asr_delay_ms=960,
            )
            started = next(item for item in received if item["type"] == "session.started")
            self.assertEqual(started["asr_delay_ms"], 960)
            self.assertEqual(getattr(voxtral.configs[-1], "delay_ms"), 960)
            self.assertEqual(voxtral.load_count, 1)
            self.assertEqual(voxtral.start_count, 3)
            async with connect(service.uri, compression=None) as websocket:
                await websocket.send(
                    json.dumps(
                        {
                            "schema_version": 1,
                            "type": "client.hello",
                            "token": service.token,
                        }
                    )
                )
                await websocket.recv()
                await websocket.send(
                    json.dumps(
                        {
                            "schema_version": 1,
                            "type": "session.start",
                            "session_id": "unsupported-delay",
                            "locale": "ru",
                            "sample_rate_hz": 16_000,
                            "encoding": "pcm_s16le",
                            "channels": 1,
                            "asr_delay_ms": 640,
                        }
                    )
                )
                error = json.loads(await websocket.recv())
                self.assertEqual(error["code"], "ASR_DELAY_UNAVAILABLE")
                self.assertTrue(error["terminal"])
            self.assertEqual(voxtral.start_count, 3)
        finally:
            await service.close()

    async def test_silence_is_vad_gated_and_final_timeline_is_no_speech(self) -> None:
        events = await self.run_turn("silence-turn")
        self.assertEqual(self.affect.observe_count, 0)
        updates = [item for item in events if item["type"] == "speech_timeline.update"]
        self.assertTrue(updates)
        activity = updates[-1]["vocal_affect"]["speech_activity"]
        self.assertEqual(activity[0]["state"], "no_speech")
        self.assertEqual(updates[-1]["vocal_affect"]["raw_observations"], [])

    async def test_enhanced_route_fails_before_capture_when_not_configured(self) -> None:
        async def chunks():
            yield b"\0\0" * 160

        ready = ReadyInfo(
            uri=self.service.uri,
            token=self.service.token,
            protocol="nextengine.speech-timeline/1",
            bounds={},
        )
        with self.assertRaises(ClientError) as caught:
            await run_websocket_session(
                ready,
                chunks(),
                locale="ru",
                on_event=lambda _: None,
                asr_audio_route="enhanced",
            )
        self.assertIn("unavailable", str(caught.exception))

    async def test_audio_preprocessor_is_resident_and_routes_only_asr(self) -> None:
        preprocessor = FakeAudioPreprocessor()
        transcriber = FakeTranscriber()
        runtime = SpeechTimelineRuntime(transcriber, FakeAffect(), preprocessor)
        service = SpeechTimelineWebSocketService(
            runtime,
            ready_file=Path(self.temp.name) / "preprocessed-ready.json",
            port=0,
            diagnostic_audio=DiagnosticAudioStore(self.diagnostic_audio_root),
        )
        await service.start()
        try:
            events = await self._run_turn_against(
                service,
                "preprocessed-turn",
                asr_audio_route="enhanced",
                vad_noise_floor_dbfs=-62.0,
            )
            final = next(item for item in events if item["type"] == "utterance.final")
            self.assertEqual(preprocessor.load_count, 1)
            self.assertEqual(preprocessor.reset_count, 1)
            self.assertEqual(preprocessor.reset_routes, [("enhanced", -62.0)])
            self.assertEqual(preprocessor.process_count, 1)
            self.assertEqual(preprocessor.flush_count, 1)
            self.assertEqual(transcriber.sessions[0].pushed_samples[0][0], 0.125)
            metrics = final["metrics"]["audio_preprocessor"]
            self.assertTrue(metrics["enabled"])
            self.assertTrue(metrics["active"])
            self.assertEqual(metrics["selected_route"], "enhanced")
            self.assertEqual(metrics["input_samples"], 4_000)
            self.assertEqual(metrics["output_samples"], 4_000)
            records = service.diagnostic_audio.list_records() if service.diagnostic_audio else []
            self.assertEqual(len(records), 1)
            self.assertTrue(records[0].enhanced_available)
            self.assertEqual(records[0].asr_audio_route, "enhanced")

            def load_enhanced() -> bytes:
                with urlopen(
                    service.dashboard_uri + f"api/diagnostic-audio/{records[0].record_id}.asr.wav",
                    timeout=2,
                ) as response:
                    return response.read()

            self.assertTrue((await asyncio.to_thread(load_enhanced)).startswith(b"RIFF"))
        finally:
            await service.close()

    async def test_raw_is_default_and_bypasses_resident_audio_preprocessor(self) -> None:
        preprocessor = FakeAudioPreprocessor()
        transcriber = FakeTranscriber()
        runtime = SpeechTimelineRuntime(transcriber, FakeAffect(), preprocessor)
        service = SpeechTimelineWebSocketService(
            runtime,
            ready_file=Path(self.temp.name) / "raw-default-ready.json",
            port=0,
            diagnostic_audio=DiagnosticAudioStore(
                Path(self.temp.name) / "raw-default-diagnostic-audio"
            ),
        )
        ready = await service.start()
        try:
            routing = ready["asr_audio_routing"]
            self.assertEqual(routing["default_route"], "raw")
            self.assertEqual(
                routing["available_routes"],
                ["raw", "enhanced", "gain_only", "whisper"],
            )
            events = await self._run_turn_against(service, "raw-default-turn")
            started = next(item for item in events if item["type"] == "session.started")
            final = next(item for item in events if item["type"] == "utterance.final")
            self.assertEqual(started["asr_audio_route"], "raw")
            self.assertEqual(preprocessor.load_count, 1)
            self.assertEqual(preprocessor.reset_count, 0)
            self.assertEqual(preprocessor.process_count, 0)
            self.assertEqual(preprocessor.flush_count, 0)
            self.assertEqual(transcriber.sessions[0].pushed_samples[0][0], 0.0)
            metrics = final["metrics"]["audio_preprocessor"]
            self.assertTrue(metrics["enabled"])
            self.assertFalse(metrics["active"])
            self.assertEqual(metrics["selected_route"], "raw")
            self.assertEqual(metrics["input_samples"], 0)
            records = service.diagnostic_audio.list_records() if service.diagnostic_audio else []
            self.assertEqual(len(records), 1)
            self.assertFalse(records[0].enhanced_available)
        finally:
            await service.close()

    async def _run_turn_against(
        self,
        service: SpeechTimelineWebSocketService,
        session_id: str,
        *,
        asr_audio_route: str | None = None,
        asr_model: str | None = None,
        asr_delay_ms: int | None = None,
        vad_noise_floor_dbfs: float | None = None,
    ) -> list[dict[str, object]]:
        async with connect(service.uri, compression=None) as websocket:
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "client.hello",
                        "token": service.token,
                    }
                )
            )
            await websocket.recv()
            start = {
                "schema_version": 1,
                "type": "session.start",
                "session_id": session_id,
                "locale": "ru",
                "sample_rate_hz": 16_000,
                "encoding": "pcm_s16le",
                "channels": 1,
            }
            if asr_audio_route is not None:
                start["asr_audio_route"] = asr_audio_route
            if asr_model is not None:
                start["asr_model"] = asr_model
            if asr_delay_ms is not None:
                start["asr_delay_ms"] = asr_delay_ms
            if vad_noise_floor_dbfs is not None:
                start["vad_calibration"] = {
                    "noise_floor_dbfs": vad_noise_floor_dbfs,
                    "duration_ms": 2_000,
                }
            await websocket.send(json.dumps(start))
            started = json.loads(await websocket.recv())
            await websocket.send(b"\0\0" * 4_000)
            await websocket.send(
                json.dumps(
                    {"schema_version": 1, "type": "session.finish", "session_id": session_id}
                )
            )
            events = [started]
            while True:
                event_value = json.loads(await asyncio.wait_for(websocket.recv(), 2))
                events.append(event_value)
                if event_value["type"] == "utterance.final":
                    return events

    async def test_last_five_diagnostic_wavs_are_listed_and_playable(self) -> None:
        for index in range(6):
            await self.run_turn(f"diagnostic-{index}")

        def listing() -> list[dict[str, object]]:
            with urlopen(self.service.dashboard_uri + "api/diagnostic-audio", timeout=2) as response:
                return json.loads(response.read())["records"]

        records = await asyncio.to_thread(listing)
        self.assertEqual(len(records), 5)
        audio_id = records[0]["id"]
        def load_audio() -> tuple[bytes, str]:
            with urlopen(
                self.service.dashboard_uri + f"api/diagnostic-audio/{audio_id}.wav", timeout=2
            ) as response:
                return response.read(), response.headers.get_content_type()

        payload, content_type = await asyncio.to_thread(load_audio)
        self.assertEqual(content_type, "audio/wav")
        self.assertTrue(payload.startswith(b"RIFF"))

    async def test_held_out_calibration_uses_websocket_vad_timeline_and_omits_text(self) -> None:
        root = Path(self.temp.name) / "calibration"
        audio = root / "audio" / "neutral.wav"
        audio.parent.mkdir(parents=True)
        with wave.open(str(audio), "wb") as destination:
            destination.setnchannels(1)
            destination.setsampwidth(2)
            destination.setframerate(16_000)
            destination.writeframes(b"\x40\x1f" * 40_000)
        audio_hash = hashlib.sha256(audio.read_bytes()).hexdigest()
        manifest_path = root / "manifest.json"
        manifest_path.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "kind": "nextengine.speech-timeline.manual-affect-calibration-set",
                    "source": {"dataset_id": "test"},
                    "clips": [
                        {
                            "clip_id": "neutral-1",
                            "source_emotion": "neutral",
                            "expected_emotion2vec_label": "neutral",
                            "relative_audio_path": "audio/neutral.wav",
                            "normalized_audio_sha256": f"sha256:{audio_hash}",
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        report = await evaluate_affect_calibration(
            ReadyInfo(
                uri=self.service.uri,
                token=self.ready["token"],
                protocol=self.ready["protocol"],
                bounds={},
                models=self.ready["models"],
                model_identity=self.ready["model_identity"],
            ),
            load_affect_calibration_manifest(manifest_path),
            mode="unpaced",
            chunk_ms=80,
        )
        self.assertEqual(report["summary"]["clips_complete"], 1)
        self.assertEqual(report["summary"]["speech_admitted_clips"], 1)
        self.assertEqual(report["summary"]["exact_top1_matches"], 1)
        self.assertEqual(report["rows"][0]["result"]["final_label"], "neutral")
        self.assertNotIn("готово", json.dumps(report, ensure_ascii=False))

    async def test_vad_calibration_is_reflected_in_session_and_final_metrics(self) -> None:
        async with connect(self.service.uri, compression=None) as websocket:
            await self.authenticate(websocket)
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "session.start",
                        "session_id": "calibrated",
                        "locale": "ru",
                        "sample_rate_hz": 16_000,
                        "encoding": "pcm_s16le",
                        "channels": 1,
                        "vad_calibration": {
                            "noise_floor_dbfs": -50.0,
                            "duration_ms": 2_000,
                        },
                    }
                )
            )
            started = json.loads(await websocket.recv())
            activity = started["vocal_activity"]
            self.assertEqual(activity["speech_threshold_dbfs"], -41.0)
            self.assertEqual(activity["calibration"]["noise_floor_dbfs"], -50.0)
            await websocket.send(b"\0\0" * 4_000)
            await websocket.send(
                json.dumps(
                    {
                        "schema_version": 1,
                        "type": "session.finish",
                        "session_id": "calibrated",
                    }
                )
            )
            while True:
                final = json.loads(await asyncio.wait_for(websocket.recv(), 2))
                if final["type"] == "utterance.final":
                    break
        metrics = final["metrics"]
        self.assertEqual(
            metrics["vocal_activity"]["calibration"]["mode"],
            "browser_quiet_noise_floor",
        )
        self.assertIn("vocal_affect", metrics)

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
        self.assertEqual(report["run_configuration"]["asr_audio_route"], "raw")
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
            ModelJobMetric(1, "asr_push", index * 1_000, (index + 1) * 1_000, 3, 40)
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

    async def test_timeline_event_emits_affect_observations_as_append_only_delta(self) -> None:
        timeline = SpeechTimeline()
        connection = SpeechConnection(
            self.service.runtime,
            SessionBounds(),
            self.service._claim,
            self.service._release,
        )

        first = timeline.apply_affect(
            AffectObservation(
                model_id="fake-emotion",
                model_revision="revision",
                start_sample=0,
                end_sample=16_000,
                source_revision=16_000,
                scores={"neutral": 0.8},
                top_label="neutral",
                inference_elapsed_ms=1,
            )
        )
        first_event = connection._timeline_event(first)
        second = timeline.apply_affect(
            AffectObservation(
                model_id="fake-emotion",
                model_revision="revision",
                start_sample=4_000,
                end_sample=20_000,
                source_revision=20_000,
                scores={"angry": 0.9},
                top_label="angry",
                inference_elapsed_ms=1,
            )
        )
        second_event = connection._timeline_event(second)

        first_affect = first_event["vocal_affect"]
        second_affect = second_event["vocal_affect"]
        self.assertEqual(first_affect["raw_observations_mode"], "append")
        self.assertEqual(second_affect["raw_observations_mode"], "append")
        self.assertEqual(len(first_affect["raw_observations"]), 1)
        self.assertEqual(len(second_affect["raw_observations"]), 1)
        self.assertEqual(second_affect["raw_observations_total"], 2)
        self.assertEqual(second_affect["raw_observations"][0]["observation_id"], 2)

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
