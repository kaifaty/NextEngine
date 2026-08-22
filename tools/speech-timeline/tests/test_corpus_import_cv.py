from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import wave

from nextengine_speech_timeline.corpus_import import (
    ReliabilityImportError,
    import_common_voice,
)

FFMPEG = shutil.which("ffmpeg")
MP3_ARGS = ["-f", "lavfi", "-i", "sine=frequency=440:duration=0.2", "-q:a", "9"]


def make_mp3(path: Path) -> None:
    assert FFMPEG is not None
    result = subprocess.run(
        [FFMPEG, "-y", "-loglevel", "error", *MP3_ARGS, str(path)],
        capture_output=True,
        timeout=60,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode("utf-8", "replace")[:200])


@unittest.skipUnless(FFMPEG, "ffmpeg is required for Common Voice fixtures")
class CommonVoiceImportTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = self.root / "store"
        self.store.mkdir()
        self.out_index = self.root / "indexes" / "cv.jsonl"
        self.out_index.parent.mkdir()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def read_index(self) -> list[dict[str, object]]:
        return [
            json.loads(line)
            for line in self.out_index.read_text(encoding="utf-8").splitlines()
        ]

    def _scripted_fixture(self, root: Path) -> None:
        clips = root / "clips"
        clips.mkdir(parents=True)
        for name in ("aaa.mp3", "bbb.mp3"):
            make_mp3(clips / name)
        (root / "validated.tsv").write_text(
            "client_id\tpath\tsentence_id\tsentence\tdomain\tup\tdown\n"
            "speaker-a\taaa.mp3\ts1\tПривет мир.\td\t2\t0\n"
            "speaker-b\tbbb.mp3\ts2\tЁжик.\td\t3\t0\n"
            "speaker-c\tmissing.mp3\ts3\tНет файла.\td\t1\t0\n",
            encoding="utf-8",
        )

    def test_scripted_import_skips_header_and_keeps_real_speakers(self) -> None:
        cv_root = self.root / "cv-scripted"
        self._scripted_fixture(cv_root)

        report = import_common_voice(
            self.store,
            cv_root=cv_root,
            out_index=self.out_index,
            kind="scripted",
            max_rows=100,
        )

        self.assertEqual(report["rows_written"], 2)
        self.assertEqual(report["selected_of_available"], "2/3")
        rows = {row["clip_id"]: row for row in self.read_index()}
        row = rows["cv-scripted-aaa"]
        expected_speaker = "cv-scripted-spk-" + hashlib.sha256(
            b"cv-speaker-v0\0speaker-a"
        ).hexdigest()[:32]
        self.assertEqual(row["speaker_id"], expected_speaker)
        self.assertEqual(row["transcript"], "Привет мир.")
        self.assertIn("missing.mp3", report["skips"]["missing_file"][0])
        converted = self.store / row["relative_audio_path"]
        with wave.open(str(converted)) as handle:
            self.assertEqual(
                (handle.getframerate(), handle.getnchannels(), handle.getsampwidth()),
                (16_000, 1, 2),
            )

    def test_scripted_respects_max_rows_cap_in_file_order(self) -> None:
        cv_root = self.root / "cv-scripted"
        self._scripted_fixture(cv_root)

        report = import_common_voice(
            self.store,
            cv_root=cv_root,
            out_index=self.out_index,
            kind="scripted",
            max_rows=1,
        )

        self.assertEqual(report["rows_written"], 1)
        clip_ids = [row["clip_id"] for row in self.read_index()]
        self.assertEqual(clip_ids, ["cv-scripted-aaa"])

    def test_spontaneous_header_and_columns_are_parsed(self) -> None:
        root = self.root / "cv-sps"
        audios = root / "audios"
        audios.mkdir(parents=True)
        make_mp3(audios / "spontaneous-speech-ru-1.mp3")
        make_mp3(audios / "spontaneous-speech-ru-2.mp3")
        (root / "ss-corpus-ru.tsv").write_text(
            "client_id\taudio_id\taudio_file\tduration_ms\tprompt_id\tprompt\ttranscription\n"
            "spk-1\t1\tspontaneous-speech-ru-1.mp3\t900\tp1\tВопрос?\tОтвет раз.\n"
            "spk-2\t2\tspontaneous-speech-ru-2.mp3\t800\tp2\tВопрос?\tОтвет два.\n",
            encoding="utf-8",
        )

        report = import_common_voice(
            self.store,
            cv_root=root,
            out_index=self.out_index,
            kind="spontaneous",
            max_rows=10,
        )

        self.assertEqual(report["rows_written"], 2)
        rows = {row["clip_id"]: row for row in self.read_index()}
        self.assertEqual(rows["cv-spontaneous-spontaneous-speech-ru-1"]["transcript"], "Ответ раз.")
        speaker_two = rows["cv-spontaneous-spontaneous-speech-ru-2"]["speaker_id"]
        self.assertTrue(speaker_two.startswith("cv-spontaneous-spk-"))
        self.assertNotEqual(
            rows["cv-spontaneous-spontaneous-speech-ru-1"]["speaker_id"],
            speaker_two,
        )

    def test_unknown_kind_and_bad_max_rows_fail_closed(self) -> None:
        with self.assertRaisesRegex(ReliabilityImportError, "kind must be"):
            import_common_voice(
                self.store,
                cv_root=self.root,
                out_index=self.out_index,
                kind="other",
                max_rows=1,
            )
        cv_root = self.root / "cv-scripted"
        self._scripted_fixture(cv_root)
        with self.assertRaisesRegex(ReliabilityImportError, "max_rows"):
            import_common_voice(
                self.store,
                cv_root=cv_root,
                out_index=self.out_index,
                kind="scripted",
                max_rows=0,
            )


if __name__ == "__main__":
    unittest.main()
