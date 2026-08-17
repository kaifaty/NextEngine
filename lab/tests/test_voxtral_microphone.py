from __future__ import annotations

import array
import importlib.util
import io
from pathlib import Path
import sys
import unittest


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
        chunks = list(
            voxtral_microphone.pcm_chunks(
                io.BytesIO(source.tobytes()), chunk_samples=4, max_samples=6
            )
        )
        self.assertEqual([len(chunk) for chunk in chunks], [4, 2])
        self.assertEqual(sum(map(len, chunks)), 6)

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

    def test_arecord_command_has_explicit_audio_contract(self) -> None:
        original = voxtral_microphone.shutil.which
        voxtral_microphone.shutil.which = lambda _: "/usr/bin/arecord"
        try:
            command = voxtral_microphone.arecord_command("pipewire")
        finally:
            voxtral_microphone.shutil.which = original
        self.assertEqual(command[0], "/usr/bin/arecord")
        self.assertIn("S16_LE", command)
        self.assertIn("16000", command)
        self.assertIn("pipewire", command)


if __name__ == "__main__":
    unittest.main()
