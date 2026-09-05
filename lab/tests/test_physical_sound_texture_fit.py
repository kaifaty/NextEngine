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
import physical_sound_texture_fit as fit


class TextureFitTests(unittest.TestCase):
    def test_split_excludes_middle_speed_and_second_recording(self):
        rows = fit.source.conditions(True)
        roles = [fit.role(row) for row in rows]
        self.assertEqual(roles.count("train"), 24)
        self.assertEqual(roles.count("unseen_speed"), 12)
        self.assertEqual(roles.count("repeat_development"), 24)
        self.assertTrue(
            all(
                row["repeat"] == 0 and row["commanded_speed_mm_s"] != 40
                for row in rows
                if fit.role(row) == "train"
            )
        )

    def test_feature_domain(self):
        self.assertEqual(fit.features(74, 40, 0.75).tolist(), [0, 0, 1, 0, 0])
        for args in [
            (12, 40, 0.5),
            (0, 19, 0.5),
            (65, 61, 1),
            (74, 40, 1.1),
            (0, float("nan"), 0.5),
        ]:
            with self.assertRaises(ValueError):
                fit.features(*args)

    def test_psd_scaling_seed_and_limits(self):
        db = np.full(513, -80.0)
        a = fit.synthesize(db, 3, 314)
        np.testing.assert_array_equal(a, fit.synthesize(db, 3, 314))
        self.assertFalse(np.array_equal(a, fit.synthesize(db, 3, 2718)))
        # One-sided density -80dB/Hz integrates across the Nyquist bandwidth.
        expected = (fit.RATE / 2) * 1e-8
        self.assertLess(abs(float(np.mean(a**2)) / expected - 1), 0.05)
        with self.assertRaises(ValueError):
            fit.synthesize(np.full(513, np.nan), 1, 0)
        with self.assertRaises(ValueError):
            fit.synthesize(db, 11, 0)

    def test_interpolation_uses_training_conditions(self):
        rows = [row for row in fit.source.conditions(True) if fit.role(row) == "train"]
        spectra = np.stack([np.full(513, row["commanded_speed_mm_s"]) for row in rows])
        query = dict(rows[0], commanded_speed_mm_s=40)
        np.testing.assert_array_equal(
            fit.interpolate(query, rows, spectra), np.full(513, 40)
        )

    def test_standalone_inference_needs_only_model_and_conditions(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            model = fit.SpectrumNet()
            model.mean.copy_(torch.full((513,), -100.0))
            checkpoint = root / "model.safetensors"
            save_file(model.state_dict(), checkpoint)
            meta = {
                "format": "texture-spectrum-v1",
                "checkpoint_sha256": hashlib.sha256(
                    checkpoint.read_bytes()
                ).hexdigest(),
            }
            (root / "model.json").write_text(json.dumps(meta))
            with patch.object(
                fit.sf, "read", side_effect=AssertionError("no reference audio allowed")
            ):
                fit.render(root, 74, 40, 0.5, 1, 314, root / "generated")
            self.assertTrue((root / "generated/generated.wav").exists())
            meta["checkpoint_sha256"] = "0" * 64
            (root / "model.json").write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                fit.render(root, 74, 40, 0.5, 1, 314, root / "bad")

    def test_rank4_projection_and_waveform_diagnostics(self):
        model = fit.SpectrumNet(4)
        model.mean.copy_(torch.full((513,), -100.0))
        model.basis.copy_(torch.eye(513)[:4])
        result = model(torch.from_numpy(fit.features(0, 40, 0.5)))
        self.assertEqual(tuple(result.shape), (513,))
        torch.testing.assert_close(result, torch.full((513,), -100.0))
        with self.assertRaises(ValueError):
            fit.SpectrumNet(8)
        wave = fit.synthesize(np.full(513, -100.0), 1, 314)
        metrics = fit.waveform_metrics(wave, wave)
        self.assertEqual(metrics["waveform_spectrum_rmse_db"], 0)
        self.assertEqual(metrics["level_error_db"], 0)
        self.assertAlmostEqual(
            fit.waveform_metrics(wave * 2, wave)["level_error_db"], 6.0206, places=4
        )


if __name__ == "__main__":
    unittest.main()
