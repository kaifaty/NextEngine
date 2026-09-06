#!/usr/bin/env python3
"""Focused guards for the preregistered V11-B1R2 GTLS oracle."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v11_b1_force_response_oracle as parent  # noqa: E402
import physical_sound_contact_field_r3a_v11_b1r2_noise_aware_gtls as oracle  # noqa: E402


class V11B1R2NoiseAwareGtlsTests(unittest.TestCase):
    def test_fresh_fixture_changes_poles_and_residues(self) -> None:
        fresh = oracle.exact_transfers()
        previous = parent.exact_transfers()
        self.assertEqual(fresh.shape, previous.shape)
        self.assertAlmostEqual(
            float(np.max(np.abs(fresh))), parent.TRANSFER_PEAK, places=14
        )
        self.assertFalse(
            np.array_equal(oracle.TRUTH_FREQUENCIES_HZ, parent.TRUTH_FREQUENCIES_HZ)
        )
        self.assertGreater(parent.normalized_rms_error(previous, fresh), 0.1)

    def test_noise_calibration_components_sum_to_observation(self) -> None:
        transfers = oracle.exact_transfers()
        role = oracle.generate_role(
            "development", parent.DEVELOPMENT_PROFILES, transfers, 1
        )
        item = role["trials"][0]
        self.assertTrue(
            np.array_equal(
                item["observed_force"], item["exact_force"] + item["force_noise"]
            )
        )
        self.assertTrue(
            np.array_equal(
                item["observed_response"],
                item["clean_response"] + item["response_noise"],
            )
        )

    def test_gtls_recovers_noiseless_complex_direction(self) -> None:
        transfer = oracle.exact_transfers()[0]
        spectrum = np.fft.rfft(transfer, n=parent.ANALYSIS_FFT_SAMPLES)
        trials = []
        for repeat in range(oracle.FIT_REPEATS):
            for profile_index, profile in enumerate(parent.FIT_PROFILES):
                force = parent.force_profile(profile)
                response = parent.fft_convolve(force, spectrum)
                zero = np.zeros(parent.TRANSFER_SAMPLES, dtype=np.float64)
                trials.append(
                    {
                        "observed_force": force,
                        "observed_response": response,
                        "force_noise": zero,
                        "response_noise": zero,
                        "contact": 0,
                        "profile_index": profile_index,
                        "repeat": repeat,
                    }
                )
        estimate = oracle.estimate_noise_aware(trials)
        self.assertGreaterEqual(
            parent.coverage(estimate["force_valid"], parent.COVERAGE_MAXIMUM_HZ),
            oracle.GATES["minimum_force_coverage"],
        )
        held = parent.force_profile(parent.IDENTITY_PROFILE)
        expected = parent.fft_convolve(held, spectrum)
        actual = np.fft.irfft(
            np.fft.rfft(held, n=parent.ANALYSIS_FFT_SAMPLES) * estimate["gtls"],
            n=parent.ANALYSIS_FFT_SAMPLES,
        )[: parent.TRANSFER_SAMPLES]
        self.assertLess(parent.normalized_rms_error(expected, actual), 0.02)

    def test_weak_force_is_not_promoted_from_sensor_noise(self) -> None:
        result = oracle.evaluate_corruption(
            "weak_excitation", 900, oracle.exact_transfers()[0]
        )
        self.assertEqual(result["decision"], "OOD_WEAK_EXCITATION")
        self.assertLessEqual(
            result["high_band_force_coverage"],
            oracle.GATES["maximum_weak_high_band_coverage"],
        )
        self.assertEqual(result["admitted_mode_count"], 0)

    def test_modal_support_requires_noise_aware_snr_and_coherence(self) -> None:
        bins = parent.ANALYSIS_FFT_SAMPLES // 2 + 1
        snr = np.zeros((parent.CONTACT_COUNT, bins), dtype=np.float64)
        coherence = np.zeros_like(snr)
        snr[:3] = 1_000.0
        coherence[:3] = 0.95
        modes = [{"frequency_hz": 1_000.0, "decay_per_second": 10.0}]
        result = oracle.modal_support(modes, snr, coherence)
        self.assertEqual(result[0]["supporting_contact_count"], 3)
        self.assertEqual(result[0]["confident_residue_count"], 3)

    def test_manifest_hash_closes_noise_roles_and_dependencies(self) -> None:
        root = parent.repository_root()
        value = oracle.expected_manifest(root)
        payload = parent.canonical_json(value)
        self.assertEqual(json.loads(payload), value)
        self.assertEqual(
            value["roles"]["fit"]["trial_count"],
            parent.CONTACT_COUNT * len(parent.FIT_PROFILES) * oracle.FIT_REPEATS,
        )
        self.assertEqual(
            value["implementation_sha256"]["parent_runner"],
            parent.sha256_file(Path(parent.__file__).resolve()),
        )

    def test_output_inside_repository_is_rejected(self) -> None:
        root = parent.repository_root()
        with self.assertRaisesRegex(parent.OracleError, "outside repository"):
            parent.prepare_output(root, root / "forbidden-v11-b1r2-output")

    def test_freeze_writes_only_canonical_manifest(self) -> None:
        root = parent.repository_root()
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "freeze"
            output.mkdir()
            oracle.freeze(root, output)
            self.assertEqual([item.name for item in output.iterdir()], ["manifest.json"])
            payload = (output / "manifest.json").read_bytes()
            self.assertEqual(payload, parent.canonical_json(json.loads(payload)))


if __name__ == "__main__":
    unittest.main()
