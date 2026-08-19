from __future__ import annotations

from pathlib import Path
import tempfile
import unittest
import wave

from nextengine_speech_timeline.diagnostic_audio import DiagnosticAudioStore


class DiagnosticAudioStoreTests(unittest.TestCase):
    def test_retains_only_the_newest_five_valid_wavs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            store = DiagnosticAudioStore(Path(temporary), max_records=5)
            for index in range(6):
                record = store.record((index.to_bytes(2, "little", signed=True)) * 160)
            records = store.list_records()
            self.assertEqual(len(records), 5)
            self.assertEqual(records[0].record_id, record.record_id)
            payload = store.read(records[0].record_id)
            self.assertIsNotNone(payload)
            assert payload is not None
            with wave.open(str(Path(temporary) / f"record-{records[0].record_id}.wav"), "rb") as source:
                self.assertEqual((source.getframerate(), source.getnchannels(), source.getsampwidth()), (16_000, 1, 2))
            self.assertGreater(len(payload), 44)
            self.assertIsNone(store.read("../not-a-record"))

    def test_retains_a_clock_matched_asr_enhanced_variant_with_the_raw_wav(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            store = DiagnosticAudioStore(Path(temporary), max_records=5)
            raw = b"\x10\x00" * 800
            enhanced = b"\x20\x00" * 800
            record = store.record(raw, asr_enhanced_pcm=enhanced)
            self.assertTrue(record.enhanced_available)
            self.assertEqual(store.read(record.record_id, variant="asr_enhanced")[44:], enhanced)
            with self.assertRaisesRegex(Exception, "preserve the raw sample clock"):
                store.record(raw, asr_enhanced_pcm=enhanced[:-2])


if __name__ == "__main__":
    unittest.main()
