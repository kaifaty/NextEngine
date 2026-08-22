from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import stat
import struct
import tempfile
import unittest
import wave

from nextengine_speech_timeline.corpus_import import (
    ReliabilityImportError,
    import_fleurs_ru,
    import_musan_noise,
    import_rirs,
)
from nextengine_speech_timeline.profile import REPOSITORY_ROOT


def write_float_wav(path: Path, samples: list[float], rate: int = 16_000) -> None:
    """Write a tiny IEEE-float (format tag 3) WAV like the FLEURS archives."""

    data = b"".join(struct.pack("<f", value) for value in samples)
    block_align = 4
    byte_rate = rate * block_align
    fmt = struct.pack(
        "<HHIIHH",
        3,  # IEEE float
        1,
        rate,
        byte_rate,
        block_align,
        32,
    )
    payload = b"fmt " + struct.pack("<I", len(fmt)) + fmt
    payload += b"data" + struct.pack("<I", len(data)) + data
    path.write_bytes(b"RIFF" + struct.pack("<I", len(payload) + 4) + b"WAVE" + payload)


def write_pcm_wav(
    path: Path,
    *,
    samples: int = 320,
    amplitude: int = 500,
    rate: int = 16_000,
    channels: int = 1,
    width: int = 2,
) -> None:
    frame = b"".join(
        int(amplitude).to_bytes(2, "little", signed=True) for _ in range(channels)
    )
    with wave.open(str(path), "wb") as destination:
        destination.setnchannels(channels)
        destination.setsampwidth(width)
        destination.setframerate(rate)
        for _ in range(samples):
            destination.writeframes(frame)


class CorpusImportTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = self.root / "store"
        self.store.mkdir()
        self.out_index = self.root / "indexes" / "out.jsonl"
        self.out_index.parent.mkdir()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def read_index(self) -> list[dict[str, object]]:
        return [
            json.loads(line)
            for line in self.out_index.read_text(encoding="utf-8").splitlines()
        ]

    # -- FLEURS ------------------------------------------------------------

    def test_fleurs_converts_float_wavs_into_contract_rows(self) -> None:
        fleurs = self.root / "fleurs"
        (fleurs / "dev").mkdir(parents=True)
        write_float_wav(fleurs / "dev" / "111.wav", [0.5, -0.5] * 200)
        write_float_wav(fleurs / "dev" / "222.wav", [0.25] * 320)
        (fleurs / "dev.tsv").write_text(
            "7\t111.wav\tПривет, мир!\tпривет мир\tп р и в е т\n"
            "8\t222.wav\tЁж.\tеж\tе ж\n",
            encoding="utf-8",
        )

        report = import_fleurs_ru(
            self.store,
            fleurs_root=fleurs,
            out_index=self.out_index,
            splits=("dev",),
        )

        self.assertEqual(report["rows_written"], 2)
        self.assertEqual(report["converted_files"], 2)
        rows = {row["clip_id"]: row for row in self.read_index()}
        row = rows["fleurs-dev-111"]
        self.assertEqual(row["speaker_id"], "fleurs-unlabeled-speakers")
        self.assertEqual(row["relative_audio_path"], "audio/fleurs/dev/111.wav")
        self.assertEqual(row["transcript"], "Привет, мир!")
        converted = self.store / "audio" / "fleurs" / "dev" / "111.wav"
        with wave.open(str(converted)) as handle:
            self.assertEqual(
                (handle.getframerate(), handle.getnchannels(), handle.getsampwidth()),
                (16_000, 1, 2),
            )
        self.assertEqual(row["samples"], 400)
        expected_hash = f"sha256:{hashlib.sha256(converted.read_bytes()).hexdigest()}"
        self.assertEqual(row["audio_sha256"], expected_hash)
        self.assertEqual(report["out_index_sha256"].split(":", 1)[1], hashlib.sha256(
            self.out_index.read_bytes()
        ).hexdigest())
        self.assertEqual(stat.S_IMODE(self.out_index.stat().st_mode), 0o600)

    def test_fleurs_skips_are_counted_with_typed_reasons(self) -> None:
        fleurs = self.root / "fleurs"
        (fleurs / "train").mkdir(parents=True)
        write_float_wav(fleurs / "train" / "ok.wav", [0.1] * 16)
        write_float_wav(fleurs / "train" / "ok2.wav", [0.1] * 16)
        write_float_wav(fleurs / "train" / "long.wav", [0.1] * 64)
        (fleurs / "train.tsv").write_text(
            "1\tmissing.wav\tРаз\tраз\tр\n"
            "2\tlong.wav\tДва\tдва\tд\n"
            "3\tok2.wav\t\t\t\n"
            "4\tok.wav\tТри\tтри\tт\n"
            "bad-line-without-columns\n",
            encoding="utf-8",
        )

        report = import_fleurs_ru(
            self.store,
            fleurs_root=fleurs,
            out_index=self.out_index,
            splits=("train",),
            max_samples=32,
        )

        self.assertEqual(report["rows_written"], 1)
        skips = report["skips"]
        self.assertIn("fleurs/train:missing.wav", skips["missing_file"])
        self.assertIn("fleurs/train:long.wav", skips["too_long"])
        self.assertTrue(any("empty-transcript" in item for item in skips["bad_name"]))
        self.assertTrue(any(":columns" in item for item in skips["bad_name"]))

    def test_fleurs_rejects_repository_index_paths(self) -> None:
        fleurs = self.root / "fleurs"
        (fleurs / "dev").mkdir(parents=True)
        write_float_wav(fleurs / "dev" / "1.wav", [0.0])
        (fleurs / "dev.tsv").write_text("1\t1.wav\tА\tа\tа\n", encoding="utf-8")

        inside_repo = REPOSITORY_ROOT / "tools" / "speech-timeline"
        with self.assertRaisesRegex(ReliabilityImportError, "outside the repository"):
            import_fleurs_ru(
                self.store,
                fleurs_root=fleurs,
                out_index=inside_repo / "evil.jsonl",
                splits=("dev",),
            )

    def test_fleurs_conversion_failure_is_fatal_not_silent(self) -> None:
        fleurs = self.root / "fleurs"
        (fleurs / "dev").mkdir(parents=True)
        (fleurs / "dev" / "broken.wav").write_bytes(b"not a wav at all")
        (fleurs / "dev.tsv").write_text("1\tbroken.wav\tА\tа\tа\n", encoding="utf-8")

        with self.assertRaises(ReliabilityImportError):
            import_fleurs_ru(
                self.store,
                fleurs_root=fleurs,
                out_index=self.out_index,
                splits=("dev",),
            )
        self.assertFalse(self.out_index.exists())

    # -- MUSAN noise -------------------------------------------------------

    def test_musan_references_conforming_files_and_converts_the_rest(self) -> None:
        noise_root = self.store / "raw" / "musan-noise"
        (noise_root / "sound-bible").mkdir(parents=True)
        conforming = noise_root / "sound-bible" / "a.wav"
        write_pcm_wav(conforming, samples=400, amplitude=300)
        stereo = noise_root / "sound-bible" / "b.wav"
        write_pcm_wav(stereo, samples=100, channels=2)

        report = import_musan_noise(
            self.store,
            noise_root=noise_root,
            out_index=self.out_index,
        )

        self.assertEqual(report["rows_written"], 2)
        self.assertEqual(report["converted_files"], 1)
        rows = {row["asset_id"]: row for row in self.read_index()}
        direct = rows["musan-noise-sound-bible_a"]
        self.assertEqual(
            direct["relative_audio_path"],
            "raw/musan-noise/sound-bible/a.wav",
        )
        self.assertEqual(direct["partition_group_id"], "sound-bible")
        converted = rows["musan-noise-sound-bible_b"]
        self.assertEqual(
            converted["relative_audio_path"],
            "audio/musan-noise/sound-bible_b.wav",
        )
        with wave.open(str(self.store / converted["relative_audio_path"])) as handle:
            self.assertEqual(handle.getnchannels(), 1)

    def test_musan_requires_raw_tree_under_store(self) -> None:
        outside = self.root / "outside-musan"
        outside.mkdir()
        write_pcm_wav(outside / "x.wav")

        with self.assertRaisesRegex(ReliabilityImportError, "under the store root"):
            import_musan_noise(
                self.store,
                noise_root=outside,
                out_index=self.out_index,
            )

    def test_musan_skips_oversized_assets(self) -> None:
        noise_root = self.store / "raw" / "musan-noise" / "free-sound"
        noise_root.mkdir(parents=True)
        write_pcm_wav(noise_root / "big.wav", samples=1_000)
        write_pcm_wav(noise_root / "small.wav", samples=10)

        report = import_musan_noise(
            self.store,
            noise_root=noise_root.parent,
            out_index=self.out_index,
            max_samples=100,
        )

        self.assertEqual(report["rows_written"], 1)
        self.assertIn("musan-noise/free-sound_big", report["skips"]["too_long"])

    # -- RIRS --------------------------------------------------------------

    def test_rirs_groups_by_room_and_rejects_contract_drift(self) -> None:
        rirs = self.store / "raw" / "rirs_noises" / "RIRS_NOISES"
        sim = rirs / "simulated_rirs"
        real = rirs / "real_rirs_isotropic_noises"
        sim.mkdir(parents=True, exist_ok=True)
        real.mkdir(parents=True, exist_ok=True)
        write_pcm_wav(sim / "Room099-00024.wav", samples=3_200)
        write_pcm_wav(sim / "Room100-00001.wav", samples=3_200)
        write_pcm_wav(real / "RVB2014_2ch_dining_room_3-00007.wav", samples=640)
        write_pcm_wav(real / "drift.wav", samples=640, channels=2)

        report = import_rirs(self.store, rirs_root=rirs, out_index=self.out_index)

        self.assertEqual(report["rows_written"], 3)
        rows = {row["asset_id"]: row for row in self.read_index()}
        self.assertEqual(rows["simulated_rirs-room099-00024"]["partition_group_id"], "room099")
        self.assertEqual(rows["simulated_rirs-room100-00001"]["partition_group_id"], "room100")
        self.assertEqual(
            rows["real_rirs_isotropic_noises-rvb2014_2ch_dining_room_3-00007"][
                "partition_group_id"
            ],
            "rvb2014_2ch_dining_room_3",
        )
        self.assertIn(
            "rirs/real_rirs_isotropic_noises-drift:contract",
            report["skips"]["invalid_wav"],
        )
        direct_row = rows["simulated_rirs-room099-00024"]
        self.assertEqual(
            (self.store / direct_row["relative_audio_path"]).read_bytes(),
            (sim / "Room099-00024.wav").read_bytes(),
        )

    def test_rirs_missing_subset_is_a_typed_failure(self) -> None:
        rirs = self.store / "raw" / "rirs_noises" / "RIRS_NOISES"
        rirs.mkdir(parents=True)
        with self.assertRaisesRegex(ReliabilityImportError, "missing RIRS subset"):
            import_rirs(self.store, rirs_root=rirs, out_index=self.out_index)


if __name__ == "__main__":
    unittest.main()
