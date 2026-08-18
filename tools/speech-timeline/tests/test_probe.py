from __future__ import annotations

import tempfile
import unittest
import wave
from pathlib import Path

import numpy as np

from nextengine_emotion_probe.probe import (
    EmotionProbe,
    ProbeError,
    audio_metadata,
    normalize_predictions,
    validate_waveform,
)
from nextengine_emotion_probe.recording import recording_command, validate_recorded_wav


class PredictionTests(unittest.TestCase):
    """Preserve the original emotion probe behavior after the project move."""
    def test_predictions_are_sorted_without_renaming_scores(self) -> None:
        predictions = normalize_predictions(
            [{"labels": ["sad", "angry", "neutral"], "scores": [0.2, 0.7, 0.1]}]
        )
        self.assertEqual([item["label"] for item in predictions], ["angry", "sad", "neutral"])
        self.assertEqual(predictions[0]["score"], 0.7)

    def test_malformed_result_fails_closed(self) -> None:
        with self.assertRaises(ProbeError):
            normalize_predictions([{"labels": ["sad"], "scores": []}])

    def test_non_finite_score_fails_closed(self) -> None:
        with self.assertRaises(ProbeError):
            normalize_predictions([{"labels": ["sad"], "scores": [float("nan")]}])


class InMemoryInferenceTests(unittest.TestCase):
    class FakeModel:
        def __init__(self) -> None:
            self.inputs: list[object] = []

        def generate(self, *, input: object, **_: object) -> list[dict[str, object]]:
            self.inputs.append(input)
            return [{"labels": ["neutral", "angry"], "scores": [0.8, 0.2]}]

    def test_waveform_requires_exact_bounded_audio_contract(self) -> None:
        valid = np.zeros(16_000, dtype=np.float32)
        self.assertIs(validate_waveform(valid, 16_000), valid)
        invalid = (
            (valid.astype(np.float64), 16_000),
            (valid.reshape(1, -1), 16_000),
            (valid, 8_000),
            (np.array([float("nan")], dtype=np.float32), 16_000),
            (np.array([1.1], dtype=np.float32), 16_000),
        )
        for samples, sample_rate in invalid:
            with self.subTest(dtype=samples.dtype, shape=samples.shape, rate=sample_rate):
                with self.assertRaises(ProbeError):
                    validate_waveform(samples, sample_rate)

    def test_file_and_waveform_use_one_result_builder_without_writing_audio(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            audio = root / "sample.wav"
            with wave.open(str(audio), "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(16_000)
                wav.writeframes(b"\0\0" * 16_000)
            cache = root / "cache"
            probe = EmotionProbe("model", "revision", cache, "cpu")
            model = self.FakeModel()
            probe._model = model
            before = sorted(path.relative_to(root) for path in root.rglob("*"))
            file_result = probe.analyze(audio)
            waveform_result = probe.analyze_waveform(np.zeros(16_000, dtype=np.float32))
            after = sorted(path.relative_to(root) for path in root.rglob("*"))
        self.assertEqual(file_result["predictions"], waveform_result["predictions"])
        self.assertEqual(len(model.inputs), 2)
        self.assertIsInstance(model.inputs[0], str)
        self.assertIsInstance(model.inputs[1], np.ndarray)
        self.assertEqual(before, after)


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
