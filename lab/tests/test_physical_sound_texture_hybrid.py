from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_hybrid as hybrid


class HybridTests(unittest.TestCase):
    def fixture(self):
        physical, _ = hybrid.event.profile()
        physical = hybrid.surface.features(
            "Glass", 0.4, 0.38, physical[3] * 20 + 40, physical[4] * 0.25 + 0.75
        )
        native = np.random.default_rng(1).normal(
            0, 0.001, (physical.shape[1] * hybrid.event.HOP, 2)
        )
        first, last = hybrid.calibration_window(physical, 40, len(native))
        own = hybrid.spectrum.spectrum(
            hybrid.resample_poly(native[first:last].mean(1), 1, 2)
        )
        return physical, native, own

    def test_identity_filter_and_full_tail(self):
        physical, native, own = self.fixture()
        corrected, details = hybrid.anchor(native, physical, 40, own)
        np.testing.assert_allclose(corrected[: len(native)], native, atol=1e-14)
        self.assertEqual(len(corrected), len(native) + 256)
        self.assertAlmostEqual(details["moving_level_change_db"], 0, places=10)
        self.assertEqual(details["applied_source_fitted_shift_samples"], 0)

    def test_filter_is_source_free_and_ignores_target_level(self):
        physical, native, own = self.fixture()
        target = own + np.linspace(-4, 4, 513)
        with (
            patch.object(
                hybrid.sf, "read", side_effect=AssertionError("no real audio")
            ),
            patch.object(
                hybrid.np, "genfromtxt", side_effect=AssertionError("no sensors")
            ),
            patch.object(
                hybrid.surface, "load_data", side_effect=AssertionError("no corpus")
            ),
        ):
            a, details = hybrid.anchor(native, physical, 40, target)
            b, _ = hybrid.anchor(native, physical, 40, target + 10)
        np.testing.assert_allclose(a, b, atol=1e-10)
        self.assertLess(abs(details["moving_level_change_db"]), 0.2)
        self.assertFalse(np.array_equal(a[: len(native)], native))

    def test_reject_invalid_and_silent_inputs(self):
        physical, native, own = self.fixture()
        for wave in (
            native[:, :1],
            native * float("nan"),
            np.zeros_like(native),
            np.full_like(native, 0.005),
            native[:100],
        ):
            with self.assertRaises(ValueError):
                hybrid.anchor(wave, physical, 40, own)
        for target in (own[:-1], own * float("nan"), np.ones(513), np.full(513, -181)):
            with self.assertRaises(ValueError):
                hybrid.anchor(native, physical, 40, target)

    def test_mean_removed_psd_does_not_authorize_dc_amplification(self):
        physical, native, own = self.fixture()
        native = native + 0.005
        corrected, details = hybrid.anchor(
            native, physical, 40, own + np.linspace(-4, 4, 513)
        )
        first, last = hybrid.calibration_window(physical, 40, len(native))
        self.assertAlmostEqual(details["dc_gain"], 1, places=12)
        self.assertLess(
            abs(corrected[first:last].mean() - native[first:last].mean()), 1e-5
        )
        self.assertLess(abs(details["moving_level_change_db"]), 0.1)

    def test_requested_window_does_not_depend_on_generated_samples(self):
        physical, native, _ = self.fixture()
        first, last = hybrid.calibration_window(physical, 40, len(native))
        self.assertEqual(last - first, round(0.75 * 44100))
        self.assertGreater(first, 0)
        idle = physical.copy()
        idle[3] = -2
        with self.assertRaises(ValueError):
            hybrid.calibration_window(idle, 40, len(native))

    def test_motion_gate_keeps_idle_neural_samples_exact(self):
        physical, native, _ = self.fixture()
        base = np.pad(native, ((0, 256), (0, 0)))
        corrected = base * 2
        gated, info = hybrid.motion_gate(native, corrected, physical, 40)
        np.testing.assert_array_equal(gated[:4410], native[:4410])
        np.testing.assert_array_equal(gated[132300 : len(native)], native[132300:])
        np.testing.assert_allclose(
            gated[44100:88200], corrected[44100:88200], atol=1e-8
        )
        self.assertGreater(info["unchanged_idle_samples"], 4410)
        with self.assertRaises(ValueError):
            hybrid.motion_gate(native, corrected[:-1], physical, 40)

    def test_source_free_spectrum_prediction_interpolates_both_motion_axes(self):
        surface = hybrid.surface
        rows = surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        training = [r for r in rows if surface.role(r) == "train"]
        meta = {
            "training_rows": training,
            "surfaces": [
                {
                    "texture_id": t,
                    "category": c,
                    "static_friction": mu,
                    "dynamic_friction": mu - 0.05,
                }
                for c, tt in zip(
                    surface.CATEGORIES, ((0, 2), (65, 67), (74, 77)), strict=True
                )
                for t, mu in zip(tt, (0.4, 0.6), strict=True)
            ],
        }
        bank = np.stack(
            [
                np.full(
                    513,
                    -100
                    + r["commanded_speed_mm_s"] / 10
                    + r["commanded_normal_force_N"] * 4,
                )
                for r in training
            ]
        )
        predicted, _ = hybrid.predict(bank, meta, "Glass", 0.5, 0.45, 40, 0.75)
        np.testing.assert_allclose(predicted, -93)
        for speed, force in ((61, 0.75), (40, 1.1), (np.nan, 0.5)):
            with self.assertRaises(ValueError):
                hybrid.predict(bank, meta, "Glass", 0.5, 0.45, speed, force)

    def test_spectrum_bank_rejects_held_surface_and_nonfinite_values(self):
        s = hybrid.surface
        rows = s.source.conditions(True, {t: str(t) for t in s.SURFACES})
        training = [{**r, "role": "train"} for r in rows if s.role(r) == "train"]
        surfaces = [
            {
                "texture_id": t,
                "category": c,
                "static_friction": 0.5,
                "dynamic_friction": 0.4,
            }
            for c, tt in zip(s.CATEGORIES, ((0, 2), (65, 67), (74, 77)), strict=True)
            for t in tt
        ]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            data = np.full((48, 513), -100, np.float32)
            weights = root / "spectra.safetensors"
            hybrid.save_file({"spectra": data}, weights)
            meta = {
                "format": hybrid.FORMAT,
                "table_sha256": s.TABLE_SHA,
                "spectra_sha256": hybrid.flow.codec.sha(weights),
                "training_rows": training,
                "surfaces": surfaces,
            }
            path = root / "spectra.json"
            path.write_text(json.dumps(meta))
            loaded, _ = hybrid.load_bank(root)
            np.testing.assert_array_equal(loaded, data)
            meta["training_rows"][0] = next(r for r in rows if r["texture_id"] == 76)
            path.write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                hybrid.load_bank(root)
            meta["training_rows"] = [
                {**r, "role": "train"} for r in rows if s.role(r) == "train"
            ]
            data[0, 0] = np.nan
            hybrid.save_file({"spectra": data}, weights)
            meta["spectra_sha256"] = hybrid.flow.codec.sha(weights)
            path.write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                hybrid.load_bank(root)


if __name__ == "__main__":
    unittest.main()
