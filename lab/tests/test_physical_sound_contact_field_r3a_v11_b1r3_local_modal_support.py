#!/usr/bin/env python3
"""Focused guards for the preregistered V11-B1R3 local-support oracle."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v11_b1_force_response_oracle as base  # noqa: E402
import physical_sound_contact_field_r3a_v11_b1r2_noise_aware_gtls as parent  # noqa: E402
import physical_sound_contact_field_r3a_v11_b1r3_local_modal_support as oracle  # noqa: E402


class V11B1R3LocalModalSupportTests(unittest.TestCase):
    def test_fresh_fixture_changes_truth_and_residues(self) -> None:
        fresh = oracle.exact_transfers()
        previous = parent.exact_transfers()
        self.assertEqual(fresh.shape, previous.shape)
        self.assertAlmostEqual(float(np.max(np.abs(fresh))), base.TRANSFER_PEAK, places=14)
        self.assertFalse(
            np.array_equal(oracle.TRUTH_FREQUENCIES_HZ, parent.TRUTH_FREQUENCIES_HZ)
        )
        self.assertGreater(base.normalized_rms_error(previous, fresh), 0.1)

    def test_comb_notch_force_is_nonnegative_and_hits_frozen_bin(self) -> None:
        force = oracle.force_profile(oracle.COMB_NOTCH_PROFILE)
        self.assertTrue(np.all(force >= 0.0))
        self.assertAlmostEqual(float(np.sum(force)), 1.0, places=15)
        spectrum = np.abs(np.fft.rfft(force, n=base.ANALYSIS_FFT_SAMPLES))
        target_bin = int(
            round(
                oracle.TRUTH_FREQUENCIES_HZ[-1]
                * base.ANALYSIS_FFT_SAMPLES
                / base.SAMPLE_RATE_HZ
            )
        )
        self.assertEqual(target_bin, 43_691)
        self.assertLess(spectrum[target_bin], 1.0e-12)
        self.assertGreater(spectrum[target_bin - 512], spectrum[target_bin] * 1.0e6)

    def test_sensor_calibration_excludes_room_disturbance(self) -> None:
        role = oracle.generate_role(
            "development",
            (oracle.DEVELOPMENT_PROFILES[0],),
            oracle.exact_transfers(),
            1,
        )
        item = role["trials"][0]
        self.assertTrue(
            np.array_equal(
                item["observed_force"],
                item["exact_force"] + item["force_sensor_noise"],
            )
        )
        self.assertTrue(
            np.array_equal(
                item["observed_response"],
                item["clean_response"]
                + item["response_sensor_noise"]
                + item["room_disturbance"],
            )
        )
        self.assertGreater(float(np.linalg.norm(item["room_disturbance"])), 0.0)
        self.assertFalse(
            np.array_equal(item["room_disturbance"], item["response_sensor_noise"])
        )

    def test_local_support_requires_snr_mask_and_coherence(self) -> None:
        bins = base.ANALYSIS_FFT_SAMPLES // 2 + 1
        snr = np.zeros((base.CONTACT_COUNT, bins), dtype=np.float64)
        coherence = np.zeros_like(snr)
        discovery = np.zeros_like(snr, dtype=np.bool_)
        local = oracle.neighborhood(1_777.0, 11.75)
        snr[:3, local] = 20.0
        coherence[:3, local] = 0.95
        discovery[:3, local] = True
        report = oracle.support_for_frequency(
            1_777.0, 11.75, snr, coherence, discovery
        )
        self.assertEqual(report["source_supporting_contact_count"], 3)
        self.assertEqual(report["supporting_contact_count"], 3)
        self.assertEqual(report["confident_residue_count"], 3)
        discovery[2, local] = False
        report = oracle.support_for_frequency(
            1_777.0, 11.75, snr, coherence, discovery
        )
        self.assertEqual(report["supporting_contact_count"], 2)

    def test_truth_match_uses_fresh_fixture(self) -> None:
        modes = [
            {
                "frequency_hz": float(frequency),
                "decay_per_second": float(oracle.TRUTH_DECAYS_PER_SECOND[index]),
            }
            for index, frequency in enumerate(oracle.TRUTH_FREQUENCIES_HZ)
        ]
        result = oracle.match_truth(modes)
        self.assertEqual(result["truth_match_count"], 7)
        self.assertEqual(result["matched_truth_indices"], list(range(7)))
        self.assertEqual(result["false_positive_count"], 0)

    def test_manifest_hash_closes_local_policy_and_parent_lineage(self) -> None:
        root = base.repository_root()
        value = oracle.expected_manifest(root)
        payload = base.canonical_json(value)
        self.assertEqual(json.loads(payload), value)
        self.assertFalse(value["candidate"]["broad_band_coverage_is_gate"])
        self.assertEqual(value["candidate"]["local_input_snr_floor"], 10.0)
        self.assertEqual(
            value["implementation_sha256"]["parent_runner"],
            base.sha256_file(root / oracle.PARENT_RUNNER_PATH),
        )

    def test_output_inside_repository_is_rejected(self) -> None:
        root = base.repository_root()
        with self.assertRaisesRegex(base.OracleError, "outside repository"):
            base.prepare_output(root, root / "forbidden-v11-b1r3-output")

    def test_freeze_writes_only_canonical_manifest(self) -> None:
        root = base.repository_root()
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "freeze"
            output.mkdir()
            oracle.freeze(root, output)
            self.assertEqual([item.name for item in output.iterdir()], ["manifest.json"])
            payload = (output / "manifest.json").read_bytes()
            self.assertEqual(payload, base.canonical_json(json.loads(payload)))


if __name__ == "__main__":
    unittest.main()
