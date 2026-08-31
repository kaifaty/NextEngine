#!/usr/bin/env python3
"""Focused guards for the preregistered V11-B1 force/response oracle."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v11_b1_force_response_oracle as oracle  # noqa: E402


class V11B1ForceResponseOracleTests(unittest.TestCase):
    def test_force_families_are_causal_nonnegative_and_unit_sum(self) -> None:
        for profile in (
            *oracle.FIT_PROFILES,
            *oracle.DEVELOPMENT_PROFILES,
            *oracle.HOLDOUT_PROFILES,
            oracle.IDENTITY_PROFILE,
        ):
            value = oracle.force_profile(profile)
            self.assertEqual(value.shape, (oracle.TRANSFER_SAMPLES,))
            self.assertTrue(np.all(value >= 0.0))
            self.assertAlmostEqual(float(np.sum(value)), 1.0, places=14)
            count = int(profile[1][0])
            self.assertTrue(np.all(value[count:] == 0.0))

    def test_exact_transfer_uses_global_peak_without_contact_normalization(self) -> None:
        value = oracle.exact_transfers()
        self.assertEqual(value.shape, (oracle.CONTACT_COUNT, oracle.TRANSFER_SAMPLES))
        self.assertAlmostEqual(float(np.max(np.abs(value))), oracle.TRANSFER_PEAK, places=14)
        contact_peaks = np.max(np.abs(value), axis=1)
        self.assertGreater(float(np.max(contact_peaks) - np.min(contact_peaks)), 1.0e-3)

    def test_noiseless_identity_is_numerically_exact(self) -> None:
        result = oracle.evaluate_identity(oracle.exact_transfers())
        self.assertTrue(result["finite"])
        self.assertLessEqual(result["nrmse"], oracle.GATES["maximum_identity_nrmse"])

    def test_h1_estimator_recovers_noiseless_transfer(self) -> None:
        transfer = oracle.exact_transfers()[0]
        transfer_spectrum = np.fft.rfft(transfer, n=oracle.ANALYSIS_FFT_SAMPLES)
        trials = []
        for index, profile in enumerate(oracle.FIT_PROFILES):
            force = oracle.force_profile(profile)
            response = oracle.fft_convolve(force, transfer_spectrum)
            trials.append(
                {
                    "observed_force": force,
                    "observed_response": response,
                    "contact": 0,
                    "profile_index": index,
                }
            )
        estimate = oracle.estimate_transfer(trials)
        self.assertGreaterEqual(
            oracle.coverage(estimate["valid"], oracle.COVERAGE_MAXIMUM_HZ),
            oracle.GATES["minimum_valid_coverage"],
        )
        held = oracle.force_profile(oracle.IDENTITY_PROFILE)
        expected = oracle.fft_convolve(held, transfer_spectrum)
        observed = np.fft.irfft(
            np.fft.rfft(held, n=oracle.ANALYSIS_FFT_SAMPLES) * estimate["transfer"],
            n=oracle.ANALYSIS_FFT_SAMPLES,
        )[: oracle.TRANSFER_SAMPLES]
        self.assertLess(oracle.normalized_rms_error(expected, observed), 0.02)

    def test_weak_force_is_rejected_by_conditioning(self) -> None:
        result = oracle.evaluate_corruption(
            "weak_excitation", 900, oracle.exact_transfers()[0]
        )
        self.assertEqual(result["decision"], "OOD_WEAK_EXCITATION")
        self.assertEqual(result["admitted_mode_count"], 0)

    def test_manifest_is_canonical_and_hash_closes_implementations(self) -> None:
        root = oracle.repository_root()
        value = oracle.expected_manifest(root)
        payload = oracle.canonical_json(value)
        self.assertEqual(json.loads(payload), value)
        self.assertEqual(
            value["implementation_sha256"]["runner"],
            oracle.sha256_file(Path(oracle.__file__).resolve()),
        )
        self.assertEqual(value["roles"]["order"], ["fit", "development", "holdout"])

    def test_output_inside_repository_is_rejected(self) -> None:
        root = oracle.repository_root()
        with self.assertRaisesRegex(oracle.OracleError, "outside repository"):
            oracle.prepare_output(root, root / "forbidden-v11-b1-output")

    def test_freeze_writes_only_manifest(self) -> None:
        root = oracle.repository_root()
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "freeze"
            output.mkdir()
            oracle.freeze(root, output)
            self.assertEqual([item.name for item in output.iterdir()], ["manifest.json"])
            payload = (output / "manifest.json").read_bytes()
            self.assertEqual(payload, oracle.canonical_json(json.loads(payload)))


if __name__ == "__main__":
    unittest.main()
