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
