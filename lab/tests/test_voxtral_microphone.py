from __future__ import annotations

import array
from contextlib import redirect_stderr
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import wave


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "voxtral_microphone.py"
SPEC = importlib.util.spec_from_file_location("voxtral_microphone", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
voxtral_microphone = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = voxtral_microphone
SPEC.loader.exec_module(voxtral_microphone)


class VoxtralMicrophoneTests(unittest.TestCase):
    def test_pcm16le_conversion_preserves_extremes(self) -> None:
        source = array.array("h", (-32768, -1, 0, 1, 32767))
        if sys.byteorder == "big":
            source.byteswap()
        converted = voxtral_microphone.pcm16le_to_float32(source.tobytes())
        self.assertEqual(len(converted), 5)
        self.assertEqual(converted[0], -1.0)
        self.assertEqual(converted[2], 0.0)
        self.assertAlmostEqual(converted[-1], 32767 / 32768.0)

    def test_pcm_chunks_honors_exact_duration(self) -> None:
        source = array.array("h", range(10))
        if sys.byteorder == "big":
            source.byteswap()
        captured = io.BytesIO()
        chunks = list(
            voxtral_microphone.pcm_chunks(
                io.BytesIO(source.tobytes()),
                chunk_samples=4,
                max_samples=6,
                raw_sink=captured.write,
            )
        )
        self.assertEqual([len(chunk) for chunk in chunks], [4, 2])
        self.assertEqual(sum(map(len, chunks)), 6)
        self.assertEqual(captured.getvalue(), source.tobytes()[:12])

    def test_pcm_chunks_flushes_aligned_tail(self) -> None:
        source = array.array("h", (1, 2, 3))
        if sys.byteorder == "big":
            source.byteswap()
        chunks = list(
            voxtral_microphone.pcm_chunks(
                io.BytesIO(source.tobytes()), chunk_samples=4, max_samples=None
            )
        )
        self.assertEqual([len(chunk) for chunk in chunks], [3])

    def test_raw_pcm_chunks_preserve_exact_bytes_and_duration(self) -> None:
        source = array.array("h", range(10))
        if sys.byteorder == "big":
            source.byteswap()
        chunks = list(
            voxtral_microphone.raw_pcm_chunks(
                io.BytesIO(source.tobytes()), chunk_samples=4, max_samples=6
            )
        )
        self.assertEqual([len(chunk) for chunk in chunks], [8, 4])
        self.assertEqual(b"".join(chunks), source.tobytes()[:12])

    def test_alsa_capture_command_has_explicit_audio_contract(self) -> None:
        original = voxtral_microphone.shutil.which
        voxtral_microphone.shutil.which = lambda _: "/usr/bin/arecord"
        try:
            command = voxtral_microphone.capture_command("pipewire", sample_count=32000)
        finally:
            voxtral_microphone.shutil.which = original
        self.assertEqual(command[0], "/usr/bin/arecord")
        self.assertIn("S16_LE", command)
        self.assertIn("16000", command)
        self.assertIn("pipewire", command)
        self.assertIn("32000", command)

    def test_pipewire_capture_command_targets_exact_source(self) -> None:
        original = voxtral_microphone.shutil.which
        voxtral_microphone.shutil.which = lambda _: "/usr/bin/pw-record"
        try:
            command = voxtral_microphone.capture_command(
                "pw:bluez_input.example", sample_count=16000
            )
        finally:
            voxtral_microphone.shutil.which = original
        self.assertEqual(command[0], "/usr/bin/pw-record")
        self.assertIn("bluez_input.example", command)
        self.assertIn("16000", command)
        self.assertEqual(command[-1], "-")

    def test_signal_levels_detect_digital_silence(self) -> None:
        rms, peak = voxtral_microphone.signal_levels_dbfs(bytes(32000))
        self.assertEqual(rms, -float("inf"))
        self.assertEqual(peak, -float("inf"))

    def test_signal_levels_measure_known_peak(self) -> None:
        source = array.array("h", (0, 16384, -16384, 0))
        if sys.byteorder == "big":
            source.byteswap()
        rms, peak = voxtral_microphone.signal_levels_dbfs(source.tobytes())
        self.assertAlmostEqual(peak, -6.0206, places=3)
        self.assertAlmostEqual(rms, -9.0309, places=3)

    def test_debug_wav_has_explicit_capture_format(self) -> None:
        source = array.array("h", (1, -2, 3))
        if sys.byteorder == "big":
            source.byteswap()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "probe.wav"
            with voxtral_microphone.debug_wav_writer(path) as writer:
                assert writer is not None
                writer.writeframes(source.tobytes())
            with wave.open(str(path), "rb") as saved:
                self.assertEqual(saved.getnchannels(), 1)
                self.assertEqual(saved.getsampwidth(), 2)
                self.assertEqual(saved.getframerate(), 16000)
                self.assertEqual(saved.readframes(3), source.tobytes())

    def test_debug_wav_refuses_to_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "existing.wav"
            path.write_bytes(b"keep")
            with self.assertRaises(SystemExit):
                with voxtral_microphone.debug_wav_writer(path):
                    pass
            self.assertEqual(path.read_bytes(), b"keep")

    def test_hardware_input_parser_emits_selectable_device(self) -> None:
        output = (
            "card 1: Generic [HD-Audio Generic], device 0: "
            "ALC1220 Analog [ALC1220 Analog]\n"
        )
        self.assertEqual(
            voxtral_microphone.alsa_hardware_inputs(output),
            [("plughw:CARD=Generic,DEV=0", "HD-Audio Generic / ALC1220 Analog")],
        )

    def test_pipewire_input_reports_inactive_bluetooth_profile(self) -> None:
        objects = [
            {
                "id": 7,
                "info": {
                    "props": {
                        "media.class": "Audio/Device",
                        "device.api": "bluez5",
                    },
                    "params": {
                        "Profile": [
                            {
                                "name": "a2dp-sink",
                                "classes": [["Audio/Sink", 1]],
                            }
                        ]
                    },
                },
            },
            {
                "id": 8,
                "info": {
                    "props": {
                        "media.class": "Audio/Source",
                        "device.id": 7,
                        "node.name": "bluez_input.example",
                        "node.description": "Example Headset",
                    }
                },
            },
        ]
        original_which = voxtral_microphone.shutil.which
        original_run = voxtral_microphone.subprocess.run
        voxtral_microphone.shutil.which = lambda _: "/usr/bin/pw-dump"
        voxtral_microphone.subprocess.run = lambda *args, **kwargs: subprocess.CompletedProcess(
            args[0], 0, stdout=voxtral_microphone.json.dumps(objects), stderr=""
        )
        try:
            inputs = voxtral_microphone.pipewire_inputs()
        finally:
            voxtral_microphone.shutil.which = original_which
            voxtral_microphone.subprocess.run = original_run
        self.assertEqual(
            inputs,
            [("pw:bluez_input.example", "Example Headset", "Bluetooth capture profile is inactive")],
        )

    def test_pipewire_input_accepts_active_bluetooth_capture_profile(self) -> None:
        objects = [
            {
                "id": 7,
                "info": {
                    "props": {
                        "media.class": "Audio/Device",
                        "device.api": "bluez5",
                    },
                    "params": {
                        "Profile": [
                            {
                                "name": "headset-head-unit",
                                "classes": [["Audio/Source", 1], ["Audio/Sink", 1]],
                            }
                        ]
                    },
                },
            },
            {
                "id": 8,
                "info": {
                    "props": {
                        "media.class": "Audio/Source",
                        "device.id": 7,
                        "node.name": "bluez_input.example",
                        "node.description": "Example Headset",
                    }
                },
            },
        ]
        original_which = voxtral_microphone.shutil.which
        original_run = voxtral_microphone.subprocess.run
        voxtral_microphone.shutil.which = lambda _: "/usr/bin/pw-dump"
        voxtral_microphone.subprocess.run = lambda *args, **kwargs: subprocess.CompletedProcess(
            args[0], 0, stdout=voxtral_microphone.json.dumps(objects), stderr=""
        )
        try:
            inputs = voxtral_microphone.pipewire_inputs()
        finally:
            voxtral_microphone.shutil.which = original_which
            voxtral_microphone.subprocess.run = original_run
        self.assertEqual(inputs, [("pw:bluez_input.example", "Example Headset", None)])

    def test_repeated_device_is_rejected(self) -> None:
        parser = voxtral_microphone.build_parser()
        with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            parser.parse_args(("--device", "default", "--device", "pipewire"))


if __name__ == "__main__":
    unittest.main()
