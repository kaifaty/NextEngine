from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
import tempfile
import unittest
import wave

import numpy as np

from nextengine_speech_timeline.corpus_augment import (
    ReliabilityAugmentError,
    augment_corpus,
)
from nextengine_speech_timeline.profile import REPOSITORY_ROOT


SEED = "aug-seed-1"
DATASET_ID = "public-safe-v0"


def _bucket(source_id: str, group: str) -> int:
    payload = (
        f"speaker-source-hash-v0\0{SEED}\0{DATASET_ID}\0{source_id}\0{group}"
    ).encode("utf-8")
    return int.from_bytes(hashlib.sha256(payload).digest()[:8], "big") % 10_000


def _group_in(source_id: str, target: str) -> str:
    low, high = {"train": (0, 6_999), "calibration": (7_000, 8_499), "held_out": (8_500, 10_000)}[target]
    for index in range(10_000):
        group = f"g{index}"
        if low <= _bucket(source_id, group) <= high:
            return group
    raise AssertionError("unreachable")


class CorpusAugmentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = self.root / "store"
        self.audio = self.store / "audio"
        self.audio.mkdir(parents=True)
        self.out_index = self.root / "augmented.jsonl"

        # Speech clips: one voiced train clip, one silent train clip, one
        # voiced held-out clip. Noise/RIR pools exist only for train.
        self.train_wav = self._write_tone(self.audio / "src" / "train.wav", amplitude=8_000)
        self.silent_wav = self._write_silence(self.audio / "src" / "silent.wav")
        self.held_wav = self._write_tone(self.audio / "src" / "held.wav", amplitude=6_000)
        self.noise_wav = self._write_noise_asset(self.audio / "assets" / "noise.wav")
        self.rir_wav = self._write_rir(self.audio / "assets" / "rir.wav")

        speech_source = "main-speech"
        train_speaker = _group_in(speech_source, "train")
        held_speaker = _group_in(speech_source, "held_out")
        noise_source = "fixture-noise"
        noise_group = _group_in(noise_source, "train")
        rir_source = "fixture-rir"
        rir_group = _group_in(rir_source, "train")

        rows = [
            self._speech_row(
                speech_source,
                "clip-train",
                train_speaker,
                "audio/src/train.wav",
                self.train_wav,
                "Привет мир",
                "train",
            ),
            self._speech_row(
                speech_source,
                "clip-silent",
                train_speaker,
                "audio/src/silent.wav",
                self.silent_wav,
                "Тишина",
                "train",
            ),
            self._speech_row(
                speech_source,
                "clip-held",
                held_speaker,
                "audio/src/held.wav",
                self.held_wav,
                "Проверка",
                "held_out",
            ),
            {
                **self._asset_row(
                    noise_source,
                    "noise-1",
                    noise_group,
                    "audio/assets/noise.wav",
                    self.noise_wav,
                ),
                "entry_kind": "noise",
            },
            {
                **self._asset_row(
                    rir_source,
                    "rir-1",
                    rir_group,
                    "audio/assets/rir.wav",
                    self.rir_wav,
                ),
                "entry_kind": "rir",
            },
        ]
        self.prepared_path = self.store / "prepared.jsonl"
        self.prepared_path.write_text(
            "".join(
                json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n"
                for row in rows
            ),
            encoding="utf-8",
        )
        self.recipe_path = self.root / "recipe.json"
        self.recipe_path.write_text(json.dumps(self._recipe()), encoding="utf-8")

    def tearDown(self) -> None:
        self.temp.cleanup()

    # -- fixture helpers ---------------------------------------------------

    def _recipe(self) -> dict:
        def source(source_id: str, role: str, index_name: str) -> dict:
            return {
                "source_id": source_id,
                "role": role,
                "release": "fixture release",
                "source_url": "https://example.invalid/dataset",
                "status": "planned",
                "relative_index_path": None,
                "index_sha256": None,
                "rights": {
                    "license_expression": "CC0-1.0",
                    "admission": "allowed",
                    "raw_redistribution": "test-only",
                    "notice": "Synthetic augment fixture.",
                },
            }

        return {
            "schema_version": 0,
            "kind": "nextengine.speech-reliability.dataset-recipe",
            "dataset_id": DATASET_ID,
            "dataset_revision": 1,
            "calibration_domain": "generic_public_ru_v0",
            "text_normalizer_profile": "ru-asr-normalize-v0",
            "split_policy": {
                "profile": "speaker-source-hash-v0",
                "seed": SEED,
                "train_permyriad": 7_000,
                "calibration_permyriad": 1_500,
                "held_out_permyriad": 1_500,
                "forced_held_out_sources": [],
            },
            "augmentation_profile": {
                "profile": "speech-reliability-degradation-v0",
                "seed": SEED,
                "max_variants_per_clip": 5,
                "conditions": [
                    {"kind": "attenuation", "parameters": {"gain_db": [-6, -12]}},
                    {"kind": "musan-noise", "parameters": {"snr_db": [0, 20]}},
                    {
                        "kind": "rirs-room-response",
                        "parameters": {"wet_mix_permyriad": [3_000]},
                    },
                    {
                        "kind": "microphone-band-eq",
                        "parameters": {"low_cut_hz": [100], "high_cut_hz": [4_000]},
                    },
                    {
                        "kind": "bounded-codec-damage",
                        "parameters": {"clip_peak_dbfs": [-3], "frame_loss_permyriad": [5_000]},
                    },
                ],
            },
            "replay_profile": {
                "profile": "speech-timeline-replay-v0",
                "identity_status": "planned",
                "asr_adapter_id": "fake-asr/1",
                "model_id": "test/model",
                "model_revision": "rev",
                "model_artifact_sha256": None,
                "runtime_revision": "runtime",
                "model_delay_ms": 0,
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
                source("main-speech", "speech", "speech.jsonl"),
                source("fixture-noise", "noise", "noise.jsonl"),
                source("fixture-rir", "rir", "rir.jsonl"),
            ],
        }

    @staticmethod
    def _sha(path: Path) -> str:
        return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"

    def _samples_of(self, path: Path) -> int:
        with wave.open(str(path)) as handle:
            return handle.getnframes()

    def _speech_row(
        self,
        source_id: str,
        clip_id: str,
        speaker_group: str,
        relative: str,
        path: Path,
        reference: str,
        split: str,
    ) -> dict:
        return {
            "schema_version": 0,
            "entry_kind": "speech",
            "source_id": source_id,
            "clip_id": clip_id,
            "speaker_group_hash": f"sha256:{hashlib.sha256(speaker_group.encode()).hexdigest()}",
            "split": split,
            "relative_audio_path": relative,
            "audio_sha256": self._sha(path),
            "samples": self._samples_of(path),
            "sample_rate_hz": 16_000,
            "channels": 1,
            "encoding": "pcm_s16le_wav",
            "duration_ms": self._samples_of(path) * 1000 // 16_000,
            "normalized_reference": reference,
            "normalizer_profile": "ru-asr-normalize-v0",
        }

    def _asset_row(
        self,
        source_id: str,
        asset_id: str,
        partition_group_hash_source: str,
        relative: str,
        path: Path,
    ) -> dict:
        return {
            "schema_version": 0,
            "source_id": source_id,
            "asset_id": asset_id,
            "partition_group_hash": f"sha256:{hashlib.sha256(partition_group_hash_source.encode()).hexdigest()}",
            "split": "train",
            "relative_audio_path": relative,
            "audio_sha256": self._sha(path),
            "samples": self._samples_of(path),
            "sample_rate_hz": 16_000,
            "channels": 1,
            "encoding": "pcm_s16le_wav",
            "duration_ms": self._samples_of(path) * 1000 // 16_000,
        }

    @staticmethod
    def _write_tone(path: Path, *, amplitude: int, frames: int = 16_000) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        ints = np.array(
            [
                int(amplitude * math.sin(2 * math.pi * 220 * i / 16_000))
                for i in range(frames)
            ],
            dtype="<i2",
        )
        with wave.open(str(path), "wb") as handle:
            handle.setnchannels(1)
            handle.setsampwidth(2)
            handle.setframerate(16_000)
            handle.writeframes(ints.tobytes())
        return path

    @staticmethod
    def _write_silence(path: Path, frames: int = 8_000) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        with wave.open(str(path), "wb") as handle:
            handle.setnchannels(1)
            handle.setsampwidth(2)
            handle.setframerate(16_000)
            handle.writeframes(np.zeros(frames, dtype="<i2").tobytes())
        return path

    @staticmethod
    def _write_noise_asset(path: Path, frames: int = 40_000) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        rng = np.random.default_rng(7)
        ints = (rng.integers(-2_000, 2_000, frames)).astype("<i2")
        with wave.open(str(path), "wb") as handle:
            handle.setnchannels(1)
            handle.setsampwidth(2)
            handle.setframerate(16_000)
            handle.writeframes(ints.tobytes())
        return path

    @staticmethod
    def _write_rir(path: Path) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        taps = np.zeros(3_200, dtype="<i2")
        taps[10] = 12_000
        taps[400] = -6_000
        taps[1_500] = 2_000
        with wave.open(str(path), "wb") as handle:
            handle.setnchannels(1)
            handle.setsampwidth(2)
            handle.setframerate(16_000)
            handle.writeframes(taps.tobytes())
        return path

    def read_index(self, path: Path | None = None) -> list[dict]:
        target = path or self.out_index
        return [
            json.loads(line)
            for line in target.read_text(encoding="utf-8").splitlines()
        ]

    def run_augment(self, *, splits=None, recipe=None, out_index=None):
        return augment_corpus(
            self.store,
            manifest_path=recipe or self.recipe_path,
            prepared_index_path=self.prepared_path,
            out_index=out_index or self.out_index,
            splits=splits,
        )

    # -- tests -------------------------------------------------------------

    def test_control_rows_reference_source_and_rerun_is_byte_identical(self) -> None:
        first = self.run_augment(splits=("train",))
        digest = first["out_index_sha256"]
        wav_hashes = {
            row["derivation_id"]: row["audio_sha256"] for row in self.read_index()
        }
        second = self.run_augment(splits=("train",))

        self.assertEqual(digest, second["out_index_sha256"])
        self.assertEqual(wav_hashes, {
            row["derivation_id"]: row["audio_sha256"] for row in self.read_index()
        })
        control = next(
            row for row in self.read_index() if row["derivation_id"] == "clip-train-a0"
        )
        self.assertEqual(control["condition_kind"], "control")
        self.assertEqual(control["transform_order"], ["control"])
        self.assertEqual(control["relative_audio_path"], "audio/src/train.wav")
        self.assertEqual(control["audio_sha256"], self._sha(self.train_wav))
        # The reference is inherited verbatim so replay can score it.
        self.assertEqual(control["normalized_reference"], "Привет мир")  # inherited verbatim

    def test_attenuation_matches_documented_gain_ratio(self) -> None:
        self.run_augment(splits=("train",))
        rows = {row["derivation_id"]: row for row in self.read_index()}
        attenuated_row = next(
            row
            for row in rows.values()
            if row["condition_kind"] == "attenuation"
            and row["source_clip_id"] == "clip-train"
        )
        attenuated = self.store / attenuated_row["relative_audio_path"]

        def rms(path: Path) -> float:
            with wave.open(str(path)) as handle:
                raw = np.frombuffer(handle.readframes(handle.getnframes()), dtype="<i2")
            return float(np.sqrt(np.mean((raw.astype(np.float64) / 32_767) ** 2)))

        ratio = rms(attenuated) / rms(self.train_wav)
        self.assertAlmostEqual(ratio, 10 ** (-6 / 20), delta=0.02)

    def test_noise_conditions_carry_aux_identity_and_respect_snr(self) -> None:
        self.run_augment(splits=("train",))
        rows = {row["derivation_id"]: row for row in self.read_index()}
        noisy = [
            row
            for row in rows.values()
            if row["condition_kind"] == "musan-noise"
        ]
        self.assertTrue(noisy)
        for row in noisy:
            self.assertEqual(row.get("noise_asset_id"), "noise-1")
            self.assertRegex(row["noise_sha256"], r"^sha256:[0-9a-f]{64}$")
            self.assertIn(row["parameters"]["snr_db"], [0, 20])
            self.assertEqual(row["split"], "train")
        quiet = next(row for row in noisy if row["parameters"]["snr_db"] == 20)

        def rms(path: Path) -> float:
            with wave.open(str(path)) as handle:
                raw = np.frombuffer(handle.readframes(handle.getnframes()), dtype="<i2")
            return float(np.sqrt(np.mean((raw.astype(np.float64)) ** 2)))

        base = rms(self.train_wav)
        self.assertLess(abs(rms(self.store / quiet["relative_audio_path"]) - base), base * 0.35)

    def test_frame_loss_zeroes_about_half_frames(self) -> None:
        self.run_augment(splits=("train",))
        damaged = [
            row
            for row in self.read_index()
            if row["condition_kind"] == "bounded-codec-damage"
        ]
        self.assertTrue(damaged)
        row = damaged[0]
        with wave.open(str(self.store / row["relative_audio_path"])) as handle:
            raw = np.frombuffer(handle.readframes(handle.getnframes()), dtype="<i2")
        frames = raw.reshape(-1, 320)
        zero_frames = sum(1 for frame in frames if not frame.any())
        self.assertGreater(zero_frames, len(frames) * 0.35)
        self.assertLess(zero_frames, len(frames) * 0.65)

    def test_eq_and_combined_room_rows_record_transform_order(self) -> None:
        recipe = self._recipe()
        recipe["augmentation_profile"]["conditions"] = [
            {
                "kind": "noise-plus-room-response",
                "parameters": {"snr_db": [6], "wet_mix_permyriad": [2_000]},
            }
        ]
        recipe["augmentation_profile"]["max_variants_per_clip"] = 1
        combined_recipe = self.root / "combined.json"
        combined_recipe.write_text(json.dumps(recipe), encoding="utf-8")

        self.run_augment(splits=("train",), recipe=combined_recipe)

        combined = [
            row
            for row in self.read_index()
            if row["condition_kind"] == "noise-plus-room-response"
        ]
        self.assertTrue(combined)
        row = combined[0]
        self.assertEqual(row["transform_order"], ["rir", "wet-mix", "noise"])
        self.assertIn("rir_asset_id", row)
        self.assertIn("noise_asset_id", row)

    def test_silent_clip_skips_noise_condition_but_keeps_control(self) -> None:
        report = self.run_augment(splits=("train",))
        rows = {row["derivation_id"]: row for row in self.read_index()}
        self.assertIn("clip-silent-a0", rows)
        silent_variants = [
            row
            for derivation_id, row in rows.items()
            if row["source_clip_id"] == "clip-silent"
            and row["condition_kind"] == "musan-noise"
        ]
        self.assertEqual(silent_variants, [])
        self.assertGreaterEqual(report["skips"]["counts"].get("silent_source", 0), 1)

    def test_missing_pool_for_split_fails_typed(self) -> None:
        with self.assertRaisesRegex(ReliabilityAugmentError, "pool is empty"):
            self.run_augment()

    def test_splits_filter_excludes_other_partitions(self) -> None:
        self.run_augment(splits=("train",))
        splits_used = {row["split"] for row in self.read_index()}
        self.assertEqual(splits_used, {"train"})
        self.assertTrue(all(row["derivation_id"].startswith(("clip-train", "clip-silent")) for row in self.read_index()))

    def test_repository_output_refused(self) -> None:
        inside = REPOSITORY_ROOT / "tools" / "speech-timeline"
        with self.assertRaisesRegex(ReliabilityAugmentError, "outside the repository"):
            self.run_augment(out_index=inside / "aug.jsonl")


if __name__ == "__main__":
    unittest.main()
