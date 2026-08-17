from __future__ import annotations

import importlib.util
import struct
import sys
import tempfile
import unittest
import wave
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "fish_s2_pro_demo.py"
SPEC = importlib.util.spec_from_file_location("fish_s2_pro_demo", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class FishS2ProDemoTests(unittest.TestCase):
    def test_profile_codec_routes_are_explicit(self) -> None:
        self.assertEqual(
            MODULE.codec_arguments("profile", MODULE.MODEL_PROFILES["q6"]),
            ["--codec-follow-backend"],
        )
        self.assertEqual(
            MODULE.codec_arguments("profile", MODULE.MODEL_PROFILES["q8"]),
            ["--codec-cpu"],
        )
        self.assertEqual(
            MODULE.codec_arguments("auto", MODULE.MODEL_PROFILES["q6"]),
            ["--codec-auto"],
        )

    def test_generated_output_is_rejected_inside_repository(self) -> None:
        with self.assertRaises(MODULE.DemoError):
            MODULE._output_path(
                Path("/tmp/external-model"),
                "q6",
                MODULE.REPOSITORY_ROOT / "demo.wav",
            )

    def test_wav_inspection_reports_exact_shape(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            wav_path = Path(temporary_directory) / "sample.wav"
            with wave.open(str(wav_path), "wb") as wav_file:
                wav_file.setnchannels(1)
                wav_file.setsampwidth(2)
                wav_file.setframerate(24_000)
                wav_file.writeframes(b"\x00\x00" * 2_400)
            result = MODULE._inspect_wav(wav_path)
        self.assertEqual(result["channels"], 1)
        self.assertEqual(result["sample_rate_hz"], 24_000)
        self.assertEqual(result["sample_width_bytes"], 2)
        self.assertEqual(result["frames"], 2_400)
        self.assertEqual(result["duration_seconds"], 0.1)

    def test_wav_inspection_accepts_ieee_float_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            wav_path = Path(temporary_directory) / "float.wav"
            samples = struct.pack("<240f", *([0.0] * 240))
            wav_path.write_bytes(
                struct.pack("<4sI4s", b"RIFF", 36 + len(samples), b"WAVE")
                + struct.pack(
                    "<4sIHHIIHH",
                    b"fmt ",
                    16,
                    3,
                    1,
                    24_000,
                    96_000,
                    4,
                    32,
                )
                + struct.pack("<4sI", b"data", len(samples))
                + samples
            )
            result = MODULE._inspect_wav(wav_path)
        self.assertEqual(result["format"], "ieee-float")
        self.assertEqual(result["sample_width_bytes"], 4)
        self.assertEqual(result["frames"], 240)
        self.assertEqual(result["duration_seconds"], 0.01)


if __name__ == "__main__":
    unittest.main()
