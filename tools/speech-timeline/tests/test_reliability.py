from __future__ import annotations

import hashlib
import json
from pathlib import Path
import stat
import tempfile
import unittest
import wave

from nextengine_speech_timeline.reliability import (
    ReliabilityCorpusError,
    dry_run_reliability_corpus,
    load_dataset_recipe,
    normalize_ru_asr,
    prepare_reliability_index,
    score_transcript,
    write_reliability_report,
)


class ReliabilityScoringTests(unittest.TestCase):
    def test_ru_normalizer_is_versioned_behavior(self) -> None:
        self.assertEqual(
            normalize_ru_asr("  Ёж, ПРИВЕТ!!!\tмир — снова.  "),
            "еж привет мир снова",
        )
        self.assertEqual(normalize_ru_asr("Привет\u0301"), "привет")

    def test_score_reports_deterministic_word_and_character_deletions(self) -> None:
        score = score_transcript("Привет, мир!", "привет")

        self.assertFalse(score.exact_match)
        self.assertEqual(score.word_substitutions, 0)
        self.assertEqual(score.word_deletions, 1)
        self.assertEqual(score.word_insertions, 0)
        self.assertEqual(score.wer, 0.5)
        self.assertEqual(score.character_deletions, 3)
        self.assertAlmostEqual(score.cer, 1 / 3)

    def test_reference_with_digits_is_excluded(self) -> None:
        with self.assertRaisesRegex(ReliabilityCorpusError, "decimal digits"):
            score_transcript(
                "У меня 2 предмета", "у меня два предмета"
            )


class ReliabilityCorpusTests(unittest.TestCase):
    def test_planned_recipe_dry_run_is_content_free_and_fail_closed(self) -> None:
        recipe_path = (
            Path(__file__).resolve().parents[1]
            / "examples"
            / "speech-reliability-public-safe-v0.recipe.json"
        )
        recipe = load_dataset_recipe(recipe_path)

        with tempfile.TemporaryDirectory() as temp_dir:
            report = dry_run_reliability_corpus(recipe, Path(temp_dir))

        self.assertEqual(report["status"], "planned_with_blockers")
        self.assertFalse(report["ready_for_prepare"])
        self.assertEqual(report["downloads_performed"], 0)
        self.assertEqual(report["privacy"], "no_audio_or_transcript_read")
        blocker_codes = {row["code"] for row in report["blockers"]}
        self.assertIn("SOURCE_NOT_HASH_CLOSED", blocker_codes)
        self.assertIn("REPLAY_IDENTITY_NOT_HASH_CLOSED", blocker_codes)

    def test_prepare_verifies_audio_and_groups_speaker_splits(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            recipe_path = self._write_closed_fixture(root)
            recipe = load_dataset_recipe(recipe_path)

            dry_run = dry_run_reliability_corpus(recipe, root)
            self.assertTrue(dry_run["ready_for_prepare"])

            prepared_path = root / "prepared.jsonl"
            report_path = root / "prepare-report.json"
            report = prepare_reliability_index(recipe, root, prepared_path)
            write_reliability_report(report_path, report)

            rows = [
                json.loads(line)
                for line in prepared_path.read_text(encoding="utf-8").splitlines()
            ]
            main_rows = [row for row in rows if row["source_id"] == "main-speech"]
            held_out_rows = [
                row for row in rows if row["source_id"] == "held-out-speech"
            ]
            noise_rows = [row for row in rows if row["entry_kind"] == "noise"]

            self.assertEqual(len(rows), 6)
            self.assertEqual(main_rows[0]["split"], main_rows[1]["split"])
            self.assertEqual(
                main_rows[0]["speaker_group_hash"],
                main_rows[1]["speaker_group_hash"],
            )
            self.assertEqual(held_out_rows[0]["split"], "held_out")
            self.assertEqual(noise_rows[0]["split"], noise_rows[1]["split"])
            self.assertEqual(
                noise_rows[0]["partition_group_hash"],
                noise_rows[1]["partition_group_hash"],
            )
            self.assertEqual(report["samples_prepared"], 3)
            self.assertEqual(report["augmentation_assets_prepared"], 3)
            self.assertEqual(
                report["role_counts"], {"speech": 3, "noise": 2, "rir": 1}
            )
            self.assertEqual(
                report["sample_index_root"], report["prepared_index_sha256"]
            )
            for root_name in (
                "speaker_partition_root",
                "augmentation_partition_root",
                "split_assignment_root",
                "feature_schema_hash",
                "label_schema_hash",
            ):
                self.assertRegex(report[root_name], r"^sha256:[0-9a-f]{64}$")
            self.assertEqual(stat.S_IMODE(prepared_path.stat().st_mode), 0o600)
            self.assertEqual(stat.S_IMODE(report_path.stat().st_mode), 0o600)
            encoded_report = report_path.read_text(encoding="utf-8")
            self.assertNotIn("speaker-main", encoded_report)
            self.assertNotIn("noise-group", encoded_report)
            self.assertNotIn("привет", encoded_report)

    def test_prepare_rejects_audio_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            recipe_path = self._write_closed_fixture(root)
            recipe_raw = json.loads(recipe_path.read_text(encoding="utf-8"))
            index_path = root / recipe_raw["sources"][0]["relative_index_path"]
            rows = self._read_jsonl(index_path)
            rows[0]["audio_sha256"] = "sha256:" + "0" * 64
            self._write_jsonl(index_path, rows)
            recipe_raw["sources"][0]["index_sha256"] = self._file_hash(index_path)
            recipe_path.write_text(json.dumps(recipe_raw), encoding="utf-8")
            recipe = load_dataset_recipe(recipe_path)

            with self.assertRaisesRegex(ReliabilityCorpusError, "audio SHA-256"):
                prepare_reliability_index(recipe, root, root / "prepared.jsonl")

    def test_prepare_rejects_duplicate_audio_content(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            recipe_path = self._write_closed_fixture(root)
            recipe_raw = json.loads(recipe_path.read_text(encoding="utf-8"))
            index_path = root / recipe_raw["sources"][0]["relative_index_path"]
            rows = self._read_jsonl(index_path)
            first_audio = root / rows[0]["relative_audio_path"]
            second_audio = root / rows[1]["relative_audio_path"]
            second_audio.write_bytes(first_audio.read_bytes())
            rows[1]["audio_sha256"] = rows[0]["audio_sha256"]
            self._write_jsonl(index_path, rows)
            recipe_raw["sources"][0]["index_sha256"] = self._file_hash(index_path)
            recipe_path.write_text(json.dumps(recipe_raw), encoding="utf-8")
            recipe = load_dataset_recipe(recipe_path)

            with self.assertRaisesRegex(ReliabilityCorpusError, "duplicate audio"):
                prepare_reliability_index(recipe, root, root / "prepared.jsonl")

    def test_prepare_rejects_audio_path_escape(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            recipe_path = self._write_closed_fixture(root)
            recipe_raw = json.loads(recipe_path.read_text(encoding="utf-8"))
            index_path = root / recipe_raw["sources"][0]["relative_index_path"]
            rows = self._read_jsonl(index_path)
            rows[0]["relative_audio_path"] = "../outside.wav"
            self._write_jsonl(index_path, rows)
            recipe_raw["sources"][0]["index_sha256"] = self._file_hash(index_path)
            recipe_path.write_text(json.dumps(recipe_raw), encoding="utf-8")
            recipe = load_dataset_recipe(recipe_path)

            with self.assertRaisesRegex(ReliabilityCorpusError, "relative POSIX path"):
                prepare_reliability_index(recipe, root, root / "prepared.jsonl")

    def test_boolean_cannot_impersonate_integer_schema_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            recipe_path = self._write_closed_fixture(root)
            recipe_raw = json.loads(recipe_path.read_text(encoding="utf-8"))
            recipe_raw["schema_version"] = False
            recipe_path.write_text(json.dumps(recipe_raw), encoding="utf-8")

            with self.assertRaisesRegex(ReliabilityCorpusError, "integer"):
                load_dataset_recipe(recipe_path)

    def _write_closed_fixture(self, root: Path) -> Path:
        indexes = root / "indexes"
        audio = root / "audio"
        indexes.mkdir()
        audio.mkdir()

        main_rows = []
        for clip_id, amplitude, transcript in (
            ("main-1", 100, "Привет, мир!"),
            ("main-2", 200, "Как твои дела?"),
        ):
            audio_path = audio / f"{clip_id}.wav"
            self._write_wav(audio_path, amplitude)
            main_rows.append(
                self._index_row(
                    clip_id,
                    "speaker-main",
                    f"audio/{clip_id}.wav",
                    audio_path,
                    transcript,
                )
            )

        held_out_audio = audio / "held-1.wav"
        self._write_wav(held_out_audio, 300)
        held_out_rows = [
            self._index_row(
                "held-1",
                "speaker-held",
                "audio/held-1.wav",
                held_out_audio,
                "Независимая проверка",
            )
        ]
        main_index = indexes / "main.jsonl"
        held_out_index = indexes / "held-out.jsonl"
        self._write_jsonl(main_index, main_rows)
        self._write_jsonl(held_out_index, held_out_rows)

        augmentation_sources = []
        for role, source_id, amplitude in (
            ("noise", "test-noise", 400),
            ("rir", "test-rir", 500),
        ):
            asset_rows = []
            asset_count = 2 if role == "noise" else 1
            for asset_number in range(1, asset_count + 1):
                asset_id = f"{role}-{asset_number}"
                asset_audio = audio / f"{asset_id}.wav"
                self._write_wav(asset_audio, amplitude + asset_number)
                asset_rows.append(
                    {
                        "schema_version": 0,
                        "asset_id": asset_id,
                        "partition_group_id": f"{role}-group",
                        "relative_audio_path": f"audio/{asset_id}.wav",
                        "audio_sha256": self._file_hash(asset_audio),
                        "samples": 320,
                        "sample_rate_hz": 16000,
                        "channels": 1,
                        "encoding": "pcm_s16le_wav",
                    }
                )
            asset_index = indexes / f"{role}.jsonl"
            self._write_jsonl(asset_index, asset_rows)
            augmentation_sources.append(
                self._source(
                    source_id,
                    f"indexes/{role}.jsonl",
                    asset_index,
                    role=role,
                )
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
                "seed": "test-split-1",
                "train_permyriad": 7000,
                "calibration_permyriad": 1500,
                "held_out_permyriad": 1500,
                "forced_held_out_sources": ["held-out-speech"],
            },
            "augmentation_profile": {
                "profile": "speech-reliability-degradation-v0",
                "seed": "test-augmentation-1",
                "max_variants_per_clip": 1,
                "conditions": [
                    {"kind": "attenuation", "parameters": {"gain_db": [-6]}}
                ],
            },
            "replay_profile": {
                "profile": "speech-timeline-replay-v0",
                "identity_status": "closed",
                "asr_adapter_id": "voxtral-realtime",
                "model_id": "test/voxtral",
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
                self._source("main-speech", "indexes/main.jsonl", main_index),
                self._source(
                    "held-out-speech", "indexes/held-out.jsonl", held_out_index
                ),
                *augmentation_sources,
            ],
        }
        recipe_path = root / "recipe.json"
        recipe_path.write_text(json.dumps(recipe), encoding="utf-8")
        return recipe_path

    def _source(
        self,
        source_id: str,
        relative_index: str,
        index: Path,
        *,
        role: str = "speech",
    ) -> dict:
        return {
            "source_id": source_id,
            "role": role,
            "release": "test release",
            "source_url": "https://example.invalid/dataset",
            "status": "closed",
            "relative_index_path": relative_index,
            "index_sha256": self._file_hash(index),
            "rights": {
                "license_expression": "CC0-1.0",
                "admission": "allowed",
                "raw_redistribution": "test-only",
                "notice": "Synthetic unit-test fixture.",
            },
        }

    def _index_row(
        self,
        clip_id: str,
        speaker_id: str,
        relative_audio_path: str,
        audio_path: Path,
        transcript: str,
    ) -> dict:
        return {
            "schema_version": 0,
            "clip_id": clip_id,
            "speaker_id": speaker_id,
            "relative_audio_path": relative_audio_path,
            "audio_sha256": self._file_hash(audio_path),
            "samples": 320,
            "sample_rate_hz": 16000,
            "channels": 1,
            "encoding": "pcm_s16le_wav",
            "transcript": transcript,
        }

    @staticmethod
    def _write_wav(path: Path, amplitude: int) -> None:
        sample = int(amplitude).to_bytes(2, "little", signed=True)
        with wave.open(str(path), "wb") as destination:
            destination.setnchannels(1)
            destination.setsampwidth(2)
            destination.setframerate(16_000)
            destination.writeframes(sample * 320)

    @staticmethod
    def _write_jsonl(path: Path, rows: list[dict]) -> None:
        path.write_text(
            "".join(
                json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n"
                for row in rows
            ),
            encoding="utf-8",
        )

    @staticmethod
    def _read_jsonl(path: Path) -> list[dict]:
        return [
            json.loads(line)
            for line in path.read_text(encoding="utf-8").splitlines()
        ]

    @staticmethod
    def _file_hash(path: Path) -> str:
        return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"


if __name__ == "__main__":
    unittest.main()
