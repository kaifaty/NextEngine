from __future__ import annotations

import asyncio
import hashlib
import json
from pathlib import Path
import stat
import tempfile
import unittest
import wave

from nextengine_speech_timeline.benchmark import load_benchmark_wav
from nextengine_speech_timeline.corpus_replay import (
    OUTCOME_NO_SPEECH,
    OUTCOME_SPEECH_BUT_EMPTY,
    OUTCOME_TECHNICAL_FAILURE,
    OUTCOME_TIMEOUT,
    ReliabilityReplayError,
    load_closed_recipe,
    read_prepared_rows,
    replay_corpus,
)
from nextengine_speech_timeline.microphone_client import ReadyInfo
from nextengine_speech_timeline.profile import REPOSITORY_ROOT
from nextengine_speech_timeline.reliability import (
    load_dataset_recipe,
    prepare_reliability_index,
)
from nextengine_speech_timeline.transport_websocket import SpeechTimelineWebSocketService

from test_websocket import FakeAffect, FakeTranscriber


def _sha256_file(path: Path) -> str:
    return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"


class CorpusReplayFixtureMixin:
    """Build a genuinely prepared external corpus fixture."""

    def make_store(self, root: Path, *, adapter_id: str = "fake-transcribe/1") -> tuple[Path, Path]:
        indexes = root / "indexes"
        audio_dir = root / "audio"
        indexes.mkdir()
        audio_dir.mkdir()

        speech_rows = []
        clips = (
            ("ready-1", "Готово!", 600),
            ("phrase-1", "Привет мир", 900),
        )
        for clip_id, transcript, amplitude in clips:
            audio_path = audio_dir / f"{clip_id}.wav"
            self._write_wav(audio_path, amplitude, 3_200)
            speech_rows.append(
                {
                    "schema_version": 0,
                    "clip_id": clip_id,
                    "speaker_id": f"speaker-{clip_id}",
                    "relative_audio_path": f"audio/{clip_id}.wav",
                    "audio_sha256": _sha256_file(audio_path),
                    "samples": 3_200,
                    "sample_rate_hz": 16_000,
                    "channels": 1,
                    "encoding": "pcm_s16le_wav",
                    "transcript": transcript,
                }
            )
        speech_index = indexes / "speech.jsonl"
        self._write_jsonl(speech_index, speech_rows)

        noise_audio = audio_dir / "noise-1.wav"
        self._write_wav(noise_audio, 100, 3_200)
        noise_index = indexes / "noise.jsonl"
        self._write_jsonl(
            noise_index,
            [
                {
                    "schema_version": 0,
                    "asset_id": "noise-1",
                    "partition_group_id": "noise-group",
                    "relative_audio_path": "audio/noise-1.wav",
                    "audio_sha256": _sha256_file(noise_audio),
                    "samples": 3_200,
                    "sample_rate_hz": 16_000,
                    "channels": 1,
                    "encoding": "pcm_s16le_wav",
                }
            ],
        )

        recipe = {
            "schema_version": 0,
            "kind": "nextengine.speech-reliability.dataset-recipe",
            "dataset_id": "public-safe-v0",
            "dataset_revision": 1,
            "calibration_domain": "generic_public_ru_v0",
            "text_normalizer_profile": "ru-asr-normalize-v0",
            "split_policy": {
                "profile": "speaker-source-hash-v0",
                "seed": "replay-split-1",
                "train_permyriad": 7_000,
                "calibration_permyriad": 1_500,
                "held_out_permyriad": 1_500,
                "forced_held_out_sources": [],
            },
            "augmentation_profile": {
                "profile": "speech-reliability-degradation-v0",
                "seed": "replay-augmentation-1",
                "max_variants_per_clip": 1,
                "conditions": [
                    {"kind": "attenuation", "parameters": {"gain_db": [-6]}}
                ],
            },
            "replay_profile": {
                "profile": "speech-timeline-replay-v0",
                "identity_status": "closed",
                "asr_adapter_id": adapter_id,
                "model_id": "test/model",
                "model_revision": "test-revision",
                "model_artifact_sha256": "sha256:" + "a" * 64,
                "runtime_revision": "test-runtime",
                "model_delay_ms": 480,
                "partial_decode_interval_ms": 240,
                "asr_audio_route": "raw",
                "sample_rate_hz": 16000,
                "channels": 1,
                "encoding": "pcm_s16le",
                "chunk_ms": 80,
                "explicit_finish": True,
                "retain_diagnostic_audio": False,
                "feature_schema": "speech-reliability-features-v0",
            },
            "sources": [
                {
                    "source_id": "main-speech",
                    "role": "speech",
                    "release": "fixture release",
                    "source_url": "https://example.invalid/dataset",
                    "status": "closed",
                    "relative_index_path": "indexes/speech.jsonl",
                    "index_sha256": _sha256_file(speech_index),
                    "rights": {
                        "license_expression": "CC0-1.0",
                        "admission": "allowed",
                        "raw_redistribution": "test-only",
                        "notice": "Synthetic replay fixture.",
                    },
                },
                {
                    "source_id": "fixture-noise",
                    "role": "noise",
                    "release": "fixture release",
                    "source_url": "https://example.invalid/noise",
                    "status": "closed",
                    "relative_index_path": "indexes/noise.jsonl",
                    "index_sha256": _sha256_file(noise_index),
                    "rights": {
                        "license_expression": "CC-BY-4.0",
                        "admission": "allowed",
                        "raw_redistribution": "test-only",
                        "notice": "Synthetic replay fixture.",
                    },
                },
            ],
        }
        recipe_path = root / "recipe.json"
        recipe_path.write_text(json.dumps(recipe), encoding="utf-8")
        prepared_path = root / "prepared.jsonl"
        prepare_reliability_index(
            load_dataset_recipe(recipe_path), root, prepared_path
        )
        return recipe_path, prepared_path

    def _write_wav(self, path: Path, amplitude: int, samples: int) -> None:
        sample = int(amplitude).to_bytes(2, "little", signed=True)
        with wave.open(str(path), "wb") as destination:
            destination.setnchannels(1)
            destination.setsampwidth(2)
            destination.setframerate(16_000)
            destination.writeframes(sample * samples)

    def _write_jsonl(self, path: Path, rows: list[dict]) -> None:
        path.write_text(
            "".join(
                json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n"
                for row in rows
            ),
            encoding="utf-8",
        )


class EmptyFinalTranscriber(FakeTranscriber):
    """Admits ASR turns but always finalizes an empty transcript."""

    def __init__(self) -> None:
        super().__init__(adapter_id="fake-transcribe/1", final_text="")


class FailingPushTranscriber(FakeTranscriber):
    """Fails every streaming push with a typed model error."""

    def __init__(self) -> None:
        super().__init__(adapter_id="fake-transcribe/1")

    def start(self, config: object):
        super().start(config)
        session = self.sessions[-1]

        class ExplodingSession:
            def push_pcm(self, samples: object):
                raise RuntimeError("model exploded")

            def finish(self):
                raise RuntimeError("model exploded")

            def cancel(self):
                return None

            def close(self):
                return None

        self.sessions[-1] = ExplodingSession()
        del session
        return self.sessions[-1]


class SlowPushTranscriber(FakeTranscriber):
    """Delays pushes long enough to trip a short replay timeout."""

    def __init__(self) -> None:
        super().__init__(adapter_id="fake-transcribe/1")
        self.push_delay = 0.15


class CorpusReplayTests(
    CorpusReplayFixtureMixin, unittest.IsolatedAsyncioTestCase
):
    async def asyncSetUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = self.root / "store"
        self.store.mkdir()
        self.recipe_path, self.prepared_path = self.make_store(self.store)
        self.out_dir = self.root / "out"
        self.out_dir.mkdir()

    async def asyncTearDown(self) -> None:
        if getattr(self, "service", None) is not None:
            await self.service.close()
        self.temp.cleanup()

    async def start_service(self, transcriber) -> None:
        from nextengine_speech_timeline.service import SpeechTimelineRuntime

        self.service = SpeechTimelineWebSocketService(
            SpeechTimelineRuntime({"default": transcriber}, FakeAffect()),
            ready_file=self.root / "ready.json",
            port=0,
        )
        await self.service.start()

    def ready_info(self) -> ReadyInfo:
        payload = json.loads((self.root / "ready.json").read_text(encoding="utf-8"))
        return ReadyInfo(
            uri=payload["uri"],
            token=payload["token"],
            protocol=payload["protocol"],
            bounds={},
            models=payload.get("models"),
            model_identity=payload.get("model_identity"),
        )

    async def replay(self, **overrides: object) -> dict[str, object]:
        arguments: dict[str, object] = {
            "manifest_path": self.recipe_path,
            "store": self.store,
            "prepared_index_path": self.prepared_path,
            "out_dir": self.out_dir,
            "asr_model": "default",
            "asr_audio_route": "raw",
            "mode": "unpaced",
            "timeout_seconds": 30.0,
        }
        arguments.update(overrides)
        return await replay_corpus(self.ready_info(), **arguments)

    async def test_happy_path_replays_prepared_clips_once_and_writes_private_outputs(
        self,
    ) -> None:
        await self.start_service(FakeTranscriber(adapter_id="fake-transcribe/1"))
        report = await self.replay()

        self.assertEqual(report["status"], "complete")
        self.assertEqual(report["counts"]["rows_selected"], 2)
        self.assertEqual(report["counts"]["completed"], 2)
        self.assertEqual(
            report["requested_identity"]["asr_adapter_id"], "fake-transcribe/1"
        )
        self.assertEqual(
            report["prepared_index_sha256"],
            f"sha256:{hashlib.sha256(self.prepared_path.read_bytes()).hexdigest()}",
        )

        results = [
            json.loads(line)
            for line in (self.out_dir / "results.jsonl").read_text(
                encoding="utf-8"
            ).splitlines()
        ]
        self.assertEqual(len(results), 2)
        ready_row = next(row for row in results if row["clip_id"] == "ready-1")
        self.assertTrue(ready_row["scores"]["exact_match"])
        self.assertEqual(ready_row["outcome"], "completed")
        phrase_row = next(row for row in results if row["clip_id"] == "phrase-1")
        self.assertFalse(phrase_row["scores"]["exact_match"])
        self.assertGreater(phrase_row["scores"]["wer"], 0.0)

        features = [
            json.loads(line)
            for line in (self.out_dir / "features.jsonl").read_text(
                encoding="utf-8"
            ).splitlines()
        ]
        self.assertEqual(len(features), 2)
        self.assertEqual(features[0]["completeness"], "complete")
        self.assertEqual(
            features[0]["identity"]["asr_adapter_id"], "fake-transcribe/1"
        )
        self.assertEqual(features[0]["identity"]["audio_route"], "raw")

        traces = [
            json.loads(line)
            for line in (self.out_dir / "trace.jsonl").read_text(
                encoding="utf-8"
            ).splitlines()
        ]
        self.assertGreaterEqual(len(traces), 2)
        self.assertTrue(all("clip_id" in row for row in traces))

        for name in ("results.jsonl", "features.jsonl", "trace.jsonl", "report.json"):
            mode = stat.S_IMODE((self.out_dir / name).stat().st_mode)
            self.assertEqual(mode, 0o600, name)
        serialized_report = (self.out_dir / "report.json").read_text(encoding="utf-8")
        self.assertNotIn("готово", serialized_report)
        self.assertNotIn("привет", serialized_report)

    async def test_planned_recipe_is_refused_before_any_audio_read(self) -> None:
        raw = json.loads(self.recipe_path.read_text(encoding="utf-8"))
        raw["replay_profile"]["identity_status"] = "planned"
        raw["replay_profile"]["model_artifact_sha256"] = None
        planned_path = self.root / "planned.json"
        planned_path.write_text(json.dumps(raw), encoding="utf-8")

        with self.assertRaisesRegex(
            ReliabilityReplayError, "REPLAY_IDENTITY_NOT_HASH_CLOSED"
        ):
            load_closed_recipe(planned_path)

        self.assertFalse(any(self.out_dir.iterdir()))

    async def test_unclosed_source_blocks_even_with_closed_replay_identity(
        self,
    ) -> None:
        raw = json.loads(self.recipe_path.read_text(encoding="utf-8"))
        raw["sources"][0]["status"] = "planned"
        raw["sources"][0]["relative_index_path"] = None
        raw["sources"][0]["index_sha256"] = None
        unclosed_path = self.root / "unclosed-source.json"
        unclosed_path.write_text(json.dumps(raw), encoding="utf-8")

        with self.assertRaisesRegex(ReliabilityReplayError, "SOURCE_NOT_HASH_CLOSED"):
            load_closed_recipe(unclosed_path)

    async def test_identity_mismatch_aborts_before_first_clip(self) -> None:
        await self.start_service(FakeTranscriber(adapter_id="other-adapter/1"))
        with self.assertRaisesRegex(ReliabilityReplayError, "REPLAY_IDENTITY_MISMATCH"):
            await self.replay()
        self.assertFalse(list(self.out_dir.iterdir()))

    async def test_tampered_prepared_audio_is_detected_before_send(self) -> None:
        await self.start_service(FakeTranscriber(adapter_id="fake-transcribe/1"))
        audio = self.store / "audio" / "ready-1.wav"
        audio.write_bytes(audio.read_bytes()[:-2])
        with self.assertRaisesRegex(ReliabilityReplayError, "SHA-256 mismatch"):
            await self.replay()

    async def test_silence_is_typed_no_speech_without_scores(self) -> None:
        await self.start_service(EmptyFinalTranscriber())
        raw = json.loads(self.recipe_path.read_text(encoding="utf-8"))
        # Replace the loud fixture with pure digital silence under the VAD
        # gate; two distinct files keep the duplicate-audio guard satisfied.
        silent_a = self.store / "audio" / "silent-a.wav"
        silent_b = self.store / "audio" / "silent-b.wav"
        self._write_wav(silent_a, 0, 3_200)
        self._write_wav(silent_b, 0, 3_201)
        replacements = {
            "ready-1": ("audio/silent-a.wav", _sha256_file(silent_a), 3_200),
            "phrase-1": ("audio/silent-b.wav", _sha256_file(silent_b), 3_201),
        }
        rows = []
        index_path = self.store / raw["sources"][0]["relative_index_path"]
        for line in index_path.read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            path, digest, samples = replacements[row["clip_id"]]
            row["relative_audio_path"] = path
            row["audio_sha256"] = digest
            row["samples"] = samples
            rows.append(row)
        self._write_jsonl(index_path, rows)
        raw["sources"][0]["index_sha256"] = _sha256_file(index_path)
        self.recipe_path.write_text(json.dumps(raw), encoding="utf-8")
        prepare_reliability_index(
            load_dataset_recipe(self.recipe_path),
            self.store,
            self.store / "prepared.jsonl",
        )

        report = await self.replay()
        self.assertEqual(report["counts"]["by_outcome"], {OUTCOME_NO_SPEECH: 2})
        results = [
            json.loads(line)
            for line in (self.out_dir / "results.jsonl").read_text(
                encoding="utf-8"
            ).splitlines()
        ]
        self.assertTrue(all(row["scores"] is None for row in results))

    async def test_admitted_speech_with_empty_final_is_speech_but_empty(self) -> None:
        await self.start_service(EmptyFinalTranscriber())
        report = await self.replay()
        self.assertEqual(
            report["counts"]["by_outcome"], {OUTCOME_SPEECH_BUT_EMPTY: 2}
        )

    async def test_model_push_failure_is_one_typed_technical_row(self) -> None:
        await self.start_service(FailingPushTranscriber())
        report = await self.replay(timeout_seconds=10.0)

        self.assertEqual(
            report["counts"]["by_outcome"], {OUTCOME_TECHNICAL_FAILURE: 2}
        )
        results = [
            json.loads(line)
            for line in (self.out_dir / "results.jsonl").read_text(
                encoding="utf-8"
            ).splitlines()
        ]
        self.assertTrue(all(row["error_code"] == "MODEL_FAILURE" for row in results))
        self.assertTrue(all(row["scores"] is None for row in results))
        self.assertEqual(len(results), len({row["clip_id"] for row in results}))

    async def test_short_timeout_produces_typed_timeout_without_retry(self) -> None:
        await self.start_service(SlowPushTranscriber())
        report = await self.replay(limit=1, timeout_seconds=0.2)
        self.assertEqual(report["counts"]["by_outcome"], {OUTCOME_TIMEOUT: 1})

    async def test_feature_payload_rides_terminal_metrics_with_identity(self) -> None:
        from nextengine_speech_timeline.corpus_replay import (
            OUTCOME_OVERLOAD,
            OUTCOME_TURN_TOO_LARGE,
            outcome_from_error_code,
        )

        self.assertEqual(
            outcome_from_error_code("SERVICE_OVERLOADED"), OUTCOME_OVERLOAD
        )
        self.assertEqual(
            outcome_from_error_code("TURN_TOO_LARGE"), OUTCOME_TURN_TOO_LARGE
        )
        self.assertEqual(
            outcome_from_error_code("SOMETHING_ELSE"), OUTCOME_TECHNICAL_FAILURE
        )

        await self.start_service(FakeTranscriber(adapter_id="fake-transcribe/1"))
        await self.replay()
        features = [
            json.loads(line)
            for line in (self.out_dir / "features.jsonl").read_text(
                encoding="utf-8"
            ).splitlines()
        ]
        payload = features[0]
        self.assertEqual(payload["identity"]["asr_model"], "default")
        self.assertEqual(payload["identity"]["sample_rate_hz"], 16_000)
        self.assertEqual(payload["runtime"]["preprocessor_active"], False)
        self.assertGreaterEqual(payload["runtime"]["ingress_frames"], 2)
        self.assertEqual(payload["runtime"]["discontinuous_frames"], 0)
        self.assertGreaterEqual(payload["transcript"]["revisions_received"], 2)
        self.assertFalse(payload["transcript"]["all_revisions_empty"])
        self.assertIn("clipping_ratio", payload["acoustic"]["raw"])

    def test_repository_output_directory_is_refused(self) -> None:
        from nextengine_speech_timeline.corpus_replay import write_replay_outputs

        inside_repo = REPOSITORY_ROOT / "tools" / "speech-timeline"
        with self.assertRaisesRegex(Exception, "outside the repository"):
            write_replay_outputs(
                inside_repo, report={}, results=[], traces=[], features=[]
            )


if __name__ == "__main__":
    unittest.main()
