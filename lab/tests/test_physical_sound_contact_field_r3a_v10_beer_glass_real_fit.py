#!/usr/bin/env python3
"""Focused guards for the frozen V10 Beer Glass real-fit runner."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v10_beer_glass_real_fit as fit  # noqa: E402


class BeerGlassRealFitTests(unittest.TestCase):
    def test_canonical_json_rejects_non_finite_values(self) -> None:
        with self.assertRaises(ValueError):
            fit.canonical_json({"bad": float("nan")})

    def test_onset_uses_both_peak_and_baseline_noise_thresholds(self) -> None:
        value = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        value[20_000] = 0.5
        onset, threshold, peak_threshold, noise_threshold = fit.onset_sample(value)
        self.assertEqual(onset, 20_000)
        self.assertEqual(threshold, peak_threshold)
        self.assertEqual(peak_threshold, 0.03)
        self.assertEqual(noise_threshold, 0.0)

    def test_shift_and_crop_has_no_wrap(self) -> None:
        value = np.arange(fit.SOURCE_SAMPLES, dtype=np.float64)
        delayed = fit.shift_and_crop(value, 3)
        advanced = fit.shift_and_crop(value, -3)
        np.testing.assert_array_equal(delayed[:5], [0.0, 0.0, 0.0, 0.0, 1.0])
        np.testing.assert_array_equal(advanced[:3], [3.0, 4.0, 5.0])
        self.assertEqual(delayed.shape, (fit.ANALYSIS_SAMPLES,))
        self.assertEqual(advanced.shape, (fit.ANALYSIS_SAMPLES,))

    def test_carrier_bank_is_exact_and_bounded(self) -> None:
        centers = fit.band_centers_hz()
        first = fit.generate_carriers(centers)
        second = fit.generate_carriers(centers)
        self.assertTrue(np.array_equal(first, second))
        self.assertEqual(first.shape, (fit.NOISE_BAND_COUNT, fit.NOISE_LOOP_SAMPLES))
        self.assertTrue(np.isfinite(first).all())
        np.testing.assert_allclose(
            np.sqrt(np.mean(np.square(first), axis=1)), 1.0, rtol=1e-12, atol=1e-12
        )
        self.assertGreaterEqual(float(centers.min()), fit.endpoint.EVALUATION_MIN_HZ)
        self.assertLessEqual(float(centers.max()), fit.endpoint.EVALUATION_MAX_HZ)

    def test_triangles_cross_at_half_amplitude(self) -> None:
        centers = fit.band_centers_hz()
        profiles = fit.triangular_profiles(centers, fit.NOISE_LOOP_SAMPLES)
        self.assertEqual(
            profiles.shape,
            (fit.NOISE_BAND_COUNT, fit.NOISE_LOOP_SAMPLES // 2 + 1),
        )
        frequencies = np.fft.rfftfreq(
            fit.NOISE_LOOP_SAMPLES, 1.0 / fit.SAMPLE_RATE_HZ
        )
        midpoint = np.sqrt(centers[40] * centers[41])
        index = int(np.argmin(np.abs(frequencies - midpoint)))
        self.assertAlmostEqual(profiles[40, index], 0.5, delta=0.04)
        self.assertAlmostEqual(profiles[41, index], 0.5, delta=0.04)

    def test_inventory_hash_guard_fails_before_decode(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            path.write_text(json.dumps({"schema": "wrong"}))
            with self.assertRaisesRegex(fit.FitError, "manifest hash changed"):
                fit.validate_inventory_manifest(Path(__file__).resolve().parents[2], path)

    def test_output_inside_repository_is_rejected(self) -> None:
        root = Path(__file__).resolve().parents[2]
        with self.assertRaisesRegex(fit.FitError, "outside the repository"):
            fit.prepare_output(root, root / "forbidden-v10-fit-output")


if __name__ == "__main__":
    unittest.main()
