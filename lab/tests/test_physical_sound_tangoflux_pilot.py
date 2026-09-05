import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_tangoflux_pilot as tango


class TangoTest(unittest.TestCase):
    def test_extraction_treats_empty_controls_like_candidates(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            pcm = np.zeros((6 * tango.RATE, 2), dtype=np.int16)
            pcm[int(5.5 * tango.RATE) : int(5.6 * tango.RATE)] = 16000
            wav = root / "full.wav"
            wavfile.write(wav, tango.RATE, pcm)
            full = {
                "native_wav": str(wav),
                "native_sha256": hashlib.sha256(wav.read_bytes()).hexdigest(),
            }
            source = root / "result.json"
            source.write_text(
                json.dumps(
                    {
                        "status": "complete",
                        "keep_full_horizon": True,
                        "model": tango.MODEL,
                        "revision": tango.REVISION,
                        "seconds": 5,
                        "seeds": [314],
                        "cases": [{"id": "glass", "prompt": "Glass."}],
                        "rows": [{"case": 0, "seed": 314, "full_horizon": full}],
                        "controls": [
                            {"id": "empty-prompt", "seed": 314, "full_horizon": full}
                        ],
                    }
                )
            )
            result = tango.extract_events(source, root / "out")
            self.assertEqual(result["status"], "complete")
            self.assertFalse(result["keep_full_horizon"])
            self.assertEqual(
                result["rows"][0]["sha256"], result["controls"][0]["sha256"]
            )
            self.assertGreater(
                result["controls"][0]["event_window"]["crop_start_seconds"], 5
            )
            wav.write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "hash"):
                tango.extract_events(source, root / "bad")

    def test_event_window_does_not_boost_noise_and_preserves_samples(self):
        wave = np.full((2, 10000), 1e-5, dtype=np.float32)
        clip, info = tango.event_window(wave, 5, rate=1000)
        self.assertIsNone(clip)
        self.assertEqual(info["status"], "no_detected_event")
        wave[:, 7000:7100] = 0.5
        clip, info = tango.event_window(wave, 5, rate=1000)
        self.assertEqual(info["crop_start_seconds"], 6.95)
        self.assertEqual(clip.shape, (2, 5000))
        np.testing.assert_array_equal(clip[:, :3050], wave[:, 6950:])
        self.assertEqual(info["padding_samples"], 1950)
        self.assertFalse(info["active_after_window"])

    def test_event_window_reports_activity_beyond_requested_clip(self):
        wave = np.zeros((2, 10000), dtype=np.float32)
        wave[:, 100:200] = 0.5
        wave[:, 9000:9100] = 0.5
        _, info = tango.event_window(wave, 5, rate=1000)
        self.assertTrue(info["active_after_window"])

    def test_horizon_distinguishes_omission_from_late_event(self):
        wave = np.zeros((2, 100), dtype=np.float32)
        silent = tango.horizon_metrics(wave, 5, rate=10)
        self.assertIsNone(silent["tail_energy_fraction"])
        wave[:, 70] = 1
        measured = tango.horizon_metrics(wave, 5, rate=10)
        self.assertEqual(measured["head_rms"], 0)
        self.assertGreater(measured["tail_rms"], 0)
        self.assertEqual(measured["tail_energy_fraction"], 1)
        self.assertEqual(measured["peak_time_seconds"], 7)
        self.assertEqual(len(measured["one_second_rms"]), 10)
        for invalid, seconds in (
            (wave, 10),
            (wave, 0),
            (wave[0], 5),
            (wave + np.nan, 5),
        ):
            with self.assertRaises(ValueError):
                tango.horizon_metrics(invalid, seconds, rate=10)

    def test_custom_prompt_cases_and_default_compatibility(self):
        self.assertEqual(
            tango.load_cases(None),
            [{"id": k, "prompt": p} for k, p in tango.pilot.CASES],
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "prompts.json"
            rows = [
                {
                    "id": "jar",
                    "prompt": "A wooden rod taps a glass jar.",
                    "diagnostic_id": "glass-wood",
                }
            ]
            path.write_text(json.dumps(rows))
            self.assertEqual(tango.load_cases(path), rows)

    def test_bad_prompt_file_rejected_before_generation(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "prompts.json"
            valid = {"id": "jar", "prompt": "Glass."}
            for rows in (
                {},
                [],
                [valid] * 33,
                [valid, valid],
                [{**valid, "id": "../escape"}],
                [{**valid, "id": "empty-prompt"}],
                [{**valid, "prompt": " "}],
                [{**valid, "prompt": "x" * 513}],
                [{**valid, "prompt": "x" + " " * 512}],
                [{**valid, "prompt": "x\ny"}],
                [{**valid, "diagnostic_id": "invented"}],
                [{**valid, "audio": "forbidden.wav"}],
            ):
                path.write_text(json.dumps(rows))
                with self.assertRaises(ValueError):
                    tango.load_cases(path)

    def test_adapter_is_complete_finite_and_cannot_replace_base_weights(self):
        expected = {"transformer.to_q.lora_A.default.weight": torch.zeros(8, 16)}
        tango.validate_adapter_weights(expected, expected)
        for invalid in (
            {},
            {**expected, "transformer.to_q.weight": torch.zeros(16, 16)},
            {name: torch.zeros(4, 16) for name in expected},
            {name: torch.full((8, 16), float("nan")) for name in expected},
            {name: value.half() for name, value in expected.items()},
        ):
            with self.assertRaises(ValueError):
                tango.validate_adapter_weights(invalid, expected)

    def test_complete_weights_except_tied_alias(self):
        tango.verify_weight_keys(["text_encoder.encoder.embed_tokens.weight"], [])
        for missing, extra in (
            ([], []),
            (["fc.0.weight"], []),
            (["text_encoder.encoder.embed_tokens.weight"], ["bogus"]),
        ):
            with self.assertRaises(ValueError):
                tango.verify_weight_keys(missing, extra)

    def test_stereo_export_and_resample(self):
        with tempfile.TemporaryDirectory() as directory:
            wave = (
                np.stack([np.sin(np.arange(44100) * 0.1)] * 2).astype(np.float32) * 1.5
            )
            record, mono = tango.publish(Path(directory), "test", wave)
            sr, pcm = wavfile.read(record["native_wav"])
            self.assertEqual((sr, pcm.shape), (44100, (44100, 2)))
            self.assertEqual(len(mono), 16000)
            self.assertLessEqual(np.abs(pcm.astype(np.int32)).max(), 32113)
            self.assertLess(record["native_pcm_gain"], 1)

    def test_invalid_wave_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            for wave in (np.zeros((2, 100)), np.ones(100), np.full((2, 100), np.nan)):
                with self.assertRaises(ValueError):
                    tango.publish(Path(directory), "test", wave)


if __name__ == "__main__":
    unittest.main()
