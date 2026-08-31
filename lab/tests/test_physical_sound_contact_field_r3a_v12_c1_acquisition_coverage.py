#!/usr/bin/env python3
"""Focused guards for the preregistered V12-C1 coverage oracle."""

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
import physical_sound_contact_field_r3a_v11_b1r3_local_modal_support as parent  # noqa: E402
import physical_sound_contact_field_r3a_v12_c1_acquisition_coverage as oracle  # noqa: E402


class V12C1AcquisitionCoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.force_role = oracle.generate_force_role(
            "fit",
            oracle.FIT_PROFILES,
            oracle.FIT_REPEATS,
            oracle.ROLE_SEEDS["fit"]["force"],
        )
        cls.certificate = oracle.force_certificate(cls.force_role)

    def test_fresh_fixture_changes_truth_and_is_globally_scaled(self) -> None:
        transfers, components = oracle.exact_transfers()
        old = parent.exact_transfers()
        self.assertEqual(transfers.shape, old.shape)
        self.assertEqual(
            components.shape,
            (
                base.CONTACT_COUNT,
                len(oracle.TRUTH_FREQUENCIES_HZ),
                base.TRANSFER_SAMPLES,
            ),
        )
        self.assertAlmostEqual(
            float(np.max(np.abs(transfers))), base.TRANSFER_PEAK, places=14
        )
        self.assertFalse(
            np.array_equal(oracle.TRUTH_FREQUENCIES_HZ, parent.TRUTH_FREQUENCIES_HZ)
        )

    def test_force_role_contains_no_response_or_truth(self) -> None:
        item = self.force_role["trials"][0]
        self.assertEqual(
            set(item),
            {
                "contact",
                "profile_index",
                "repeat",
                "exact_force",
                "force_sensor_noise",
                "observed_force",
            },
        )
        self.assertTrue(
            np.array_equal(
                item["observed_force"],
                item["exact_force"] + item["force_sensor_noise"],
            )
        )

    def test_ordinary_force_certificate_covers_band_and_truth(self) -> None:
        certificate = self.certificate
        self.assertEqual(certificate["supported_band_fraction"], 1.0)
        self.assertGreaterEqual(certificate["minimum_band_profile_support_count"], 3)
        self.assertEqual(certificate["supported_truth_indices"], list(range(7)))
        self.assertEqual(certificate["unsupported_truth_indices"], [])
        self.assertTrue(all(item["certified"] for item in certificate["truth_neighborhoods"]))

    def test_every_leave_one_profile_out_force_certificate_remains_eligible(self) -> None:
        checks = oracle.ordinary_certificate_checks(self.certificate)
        self.assertEqual(len(checks), 38)
        self.assertTrue(all(item["passed"] for item in checks))

    def test_notch_is_nonnegative_and_exactly_zero_at_six_kilohertz(self) -> None:
        force = oracle.notched_profile(oracle.NOTCH_BASE_PROFILES[0])
        self.assertTrue(np.all(force >= 0.0))
        self.assertAlmostEqual(float(np.sum(force)), 1.0, places=15)
        spectrum = np.abs(np.fft.rfft(force, n=base.ANALYSIS_FFT_SAMPLES))
        target_bin = int(
            round(
                oracle.TRUTH_FREQUENCIES_HZ[oracle.NOTCH_TRUTH_INDEX]
                * base.ANALYSIS_FFT_SAMPLES
                / base.SAMPLE_RATE_HZ
            )
        )
        self.assertEqual(target_bin, 32_768)
        self.assertLess(spectrum[target_bin], 1.0e-14)
        self.assertGreater(spectrum[target_bin - 2_000], 0.01)

    def test_notch_certificate_isolates_only_declared_truth_mode(self) -> None:
        role = oracle.generate_force_role(
            "notch",
            oracle.NOTCH_BASE_PROFILES,
            oracle.FIT_REPEATS,
            oracle.ROLE_SEEDS["development_controls"]["force"],
            notch=True,
        )
        certificate = oracle.force_certificate(role)
        expected = [0, 1, 2, 3, 5, 6]
        self.assertEqual(certificate["supported_truth_indices"], expected)
        self.assertEqual(certificate["unsupported_truth_indices"], [4])

    def test_weak_acquisition_has_no_complete_certificate(self) -> None:
        control = oracle.weak_acquisition_control(
            oracle.ROLE_SEEDS["development_controls"]["force"] + 40
        )
        self.assertEqual(control["decision"], "OOD_WEAK_EXCITATION")
        self.assertLess(
            len(control["certificate"]["supported_truth_indices"]),
            len(oracle.TRUTH_FREQUENCIES_HZ),
        )
        self.assertEqual(control["complete_domain_admitted_mode_count"], 0)

    def test_manifest_hash_closes_protocol_and_forbids_real_data(self) -> None:
        root = base.repository_root()
        value = oracle.expected_manifest(root)
        payload = base.canonical_json(value)
        self.assertEqual(json.loads(payload), value)
        self.assertFalse(value["data_policy"]["real_payload_allowed"])
        self.assertFalse(value["force_certificate"]["response_or_truth_access_allowed"])
        self.assertEqual(
            value["implementation_sha256"]["protocol"],
            base.sha256_file(root / oracle.PROTOCOL_PATH),
        )

    def test_remove_profile_reindexes_without_changing_samples(self) -> None:
        reduced = oracle.remove_profile_role(self.force_role, 2)
        self.assertEqual(len(reduced["profiles"]), 4)
        self.assertEqual(len(reduced["trials"]), 96)
        self.assertEqual(
            sorted({item["profile_index"] for item in reduced["trials"]}),
            [0, 1, 2, 3],
        )

    def test_freeze_and_preflight_generate_no_samples(self) -> None:
        root = base.repository_root()
        with tempfile.TemporaryDirectory() as temporary:
            freeze_root = Path(temporary) / "freeze"
            preflight_root = Path(temporary) / "preflight"
            freeze_root.mkdir()
            preflight_root.mkdir()
            oracle.freeze(root, freeze_root)
            payload = (freeze_root / "manifest.json").read_bytes()
            manifest = json.loads(payload)
            oracle.preflight(payload, manifest, preflight_root)
            report = json.loads((preflight_root / "report.json").read_bytes())
            self.assertEqual(report["force_samples_generated"], 0)
            self.assertEqual(report["truth_samples_generated"], 0)
            self.assertEqual(report["response_samples_generated"], 0)
            self.assertEqual(report["decision"], "C1_ACQUISITION_COVERAGE_ORACLE_FROZEN")


if __name__ == "__main__":
    unittest.main()
