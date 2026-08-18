from __future__ import annotations

import json
from pathlib import Path
import stat
import tempfile
import unittest
import wave

from nextengine_speech_timeline.benchmark import (
    BenchmarkError,
    load_benchmark_wav,
    write_report,
)


class BenchmarkArtifactTests(unittest.TestCase):
    def test_external_pcm_wav_and_private_atomic_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            audio = root / "sample.wav"
            with wave.open(str(audio), "wb") as destination:
                destination.setnchannels(1)
                destination.setsampwidth(2)
                destination.setframerate(16_000)
                destination.writeframes(b"\0\0" * 8_000)
            pcm, samples = load_benchmark_wav(audio)
            self.assertEqual((len(pcm), samples), (16_000, 8_000))
            report_path = root / "report.json"
            write_report(
                report_path,
                {
                    "schema_version": 1,
                    "privacy": "audio_and_transcript_omitted",
                },
            )
            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(report["privacy"], "audio_and_transcript_omitted")
            self.assertEqual(stat.S_IMODE(report_path.stat().st_mode), 0o600)

    def test_wrong_wav_contract_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            audio = Path(temp_dir) / "sample.wav"
            with wave.open(str(audio), "wb") as destination:
                destination.setnchannels(2)
                destination.setsampwidth(2)
                destination.setframerate(44_100)
                destination.writeframes(b"\0\0" * 100)
            with self.assertRaises(BenchmarkError):
                load_benchmark_wav(audio)


if __name__ == "__main__":
    unittest.main()
