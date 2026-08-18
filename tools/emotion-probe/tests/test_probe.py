from __future__ import annotations

import tempfile
import unittest
import wave
from pathlib import Path

from nextengine_emotion_probe.probe import (
    ProbeError,
    audio_metadata,
    normalize_predictions,
)
from nextengine_emotion_probe.recording import recording_command, validate_recorded_wav


class PredictionTests(unittest.TestCase):
    def test_predictions_are_sorted_without_renaming_scores(self) -> None:
        predictions = normalize_predictions(
            [{"labels": ["sad", "angry", "neutral"], "scores": [0.2, 0.7, 0.1]}]
        )
        self.assertEqual([item["label"] for item in predictions], ["angry", "sad", "neutral"])
        self.assertEqual(predictions[0]["score"], 0.7)

    def test_malformed_result_fails_closed(self) -> None:
        with self.assertRaises(ProbeError):
            normalize_predictions([{"labels": ["sad"], "scores": []}])


class RecordingTests(unittest.TestCase):
    def test_recording_command_is_bounded_mono_16khz(self) -> None:
        command = recording_command(Path("sample.wav"), 2.5, "source-name")
        self.assertIn("16000", command)
        self.assertIn("MONO", command)
        self.assertIn("40000", command)
        self.assertEqual(command[-3:], ["--target", "source-name", "sample.wav"])

    def test_recording_duration_is_bounded(self) -> None:
        with self.assertRaises(ProbeError):
            recording_command(Path("sample.wav"), 31.0)

    def test_completed_wav_is_accepted_independently_of_recorder_exit_code(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "sample.wav"
            with wave.open(str(path), "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(16_000)
                wav.writeframes(b"\0\0" * 16_000)
            validate_recorded_wav(path, 16_000)

    def test_incomplete_wav_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "sample.wav"
            with wave.open(str(path), "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(16_000)
                wav.writeframes(b"\0\0" * 8_000)
            with self.assertRaises(ProbeError):
                validate_recorded_wav(path, 16_000)


class AudioMetadataTests(unittest.TestCase):
    def test_wav_metadata_and_hash_are_reported(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "sample.wav"
            with wave.open(str(path), "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(16_000)
                wav.writeframes(b"\0\0" * 8_000)
            metadata = audio_metadata(path)
        self.assertEqual(metadata.sample_rate_hz, 16_000)
        self.assertEqual(metadata.channels, 1)
        self.assertEqual(metadata.duration_ms, 500)
        self.assertTrue(metadata.content_hash.startswith("sha256:"))


if __name__ == "__main__":
    unittest.main()
