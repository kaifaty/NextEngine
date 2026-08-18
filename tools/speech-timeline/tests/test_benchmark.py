from __future__ import annotations

import json
import hashlib
from pathlib import Path
import stat
import tempfile
import unittest
import wave

from nextengine_speech_timeline.benchmark import (
    BenchmarkError,
    load_affect_calibration_manifest,
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

    def test_external_calibration_manifest_validates_hash_and_audio_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            audio = root / "audio" / "neutral.wav"
            audio.parent.mkdir()
            with wave.open(str(audio), "wb") as destination:
                destination.setnchannels(1)
                destination.setsampwidth(2)
                destination.setframerate(16_000)
                destination.writeframes(b"\x10\0" * 16_000)
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
            manifest = load_affect_calibration_manifest(manifest_path)
            self.assertEqual(manifest.clips[0].samples, 16_000)
            self.assertEqual(manifest.clips[0].expected_label, "neutral")

            raw = json.loads(manifest_path.read_text(encoding="utf-8"))
            raw["clips"][0]["normalized_audio_sha256"] = "sha256:" + "0" * 64
            manifest_path.write_text(json.dumps(raw), encoding="utf-8")
            with self.assertRaises(BenchmarkError):
                load_affect_calibration_manifest(manifest_path)


if __name__ == "__main__":
    unittest.main()
