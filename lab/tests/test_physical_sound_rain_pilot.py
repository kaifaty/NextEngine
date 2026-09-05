from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import torch
from safetensors.torch import save_file

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_rain_pilot as rain


class RainPilotTests(unittest.TestCase):
    def test_source_scale(self):
        power = np.full((2, 513), 32768**2 * 1e-8)
        np.testing.assert_allclose(rain.digital_psd(power), -80)
        for bad in [np.zeros((1, 513)), np.full((1, 513), np.nan)]:
            with self.assertRaises(ValueError):
                rain.digital_psd(bad)

    def test_daily_split_and_guard(self):
        days = np.arange(
            np.datetime64("2023-05-01"), np.datetime64("2023-06-01")
        ).astype(int)
        roles = rain.split_days(days)
        self.assertEqual(
            roles[days == np.datetime64("2023-05-10").astype(int)].item(), "development"
        )
        training = set(days[roles == "train"].tolist())
        development = set(days[roles == "development"].tolist())
        self.assertTrue(training)
        self.assertFalse(training & development)
        self.assertFalse(
            training & {day + step for day in development for step in (-1, 1)}
        )
        duplicated = rain.split_days(np.repeat(days, 2)).reshape(-1, 2)
        np.testing.assert_array_equal(duplicated[:, 0], duplicated[:, 1])

    def test_native_rain_rate_psd(self):
        wave = rain.synthesize(np.full(513, -80.0), 3, 314, rain.RATE)
        self.assertEqual(len(wave), 144000)
        self.assertLess(abs(float(np.mean(wave**2)) / (24000 * 1e-8) - 1), 0.05)
        with self.assertRaises(ValueError):
            rain.synthesize(np.full(513, -80.0), 3, 314, 16000)

    def test_reference_free_inference_and_invalid_requests(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            model = rain.RainNet()
            model.mean.copy_(torch.full((513,), -100.0))
            checkpoint = root / "model.safetensors"
            save_file(model.state_dict(), checkpoint)
            meta = {
                "format": "rain-spectrum-v1",
                "maximum_mm_5min": 10,
                "checkpoint_sha256": hashlib.sha256(
                    checkpoint.read_bytes()
                ).hexdigest(),
            }
            (root / "model.json").write_text(json.dumps(meta))
            with patch.object(
                rain.sf, "read", side_effect=AssertionError("no reference input")
            ):
                rain.render(root, 2, 314, root / "generated")
            self.assertTrue((root / "generated/generated.wav").exists())
            for amount in [-1, 11, float("nan")]:
                with self.assertRaises(ValueError):
                    rain.render(root, amount, 314, root / "bad")
            meta["checkpoint_sha256"] = "0" * 64
            (root / "model.json").write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                rain.render(root, 2, 314, root / "bad-hash")


if __name__ == "__main__":
    unittest.main()
