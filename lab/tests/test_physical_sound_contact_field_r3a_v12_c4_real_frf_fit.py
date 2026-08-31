#!/usr/bin/env python3
"""Focused guards for the frozen V12-C4a real force/FRF fit."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v12_c4_real_frf_fit as fit  # noqa: E402


def synthetic_source_pair(*, second_impact: bool = False) -> tuple[np.ndarray, np.ndarray]:
    force = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
    microphone = np.zeros_like(force)
    onset = 24_000
    force[onset] = 0.75
    if second_impact:
        force[onset + 600] = 0.5
    time = np.arange(fit.SOURCE_SAMPLES - onset, dtype=np.float64) / fit.SAMPLE_RATE_HZ
    microphone[onset:] = 0.2 * np.exp(-12.0 * time) * np.sin(
        2.0 * np.pi * 1_200.0 * time
    )
    return microphone, force


def certificate_records(*, impulse: bool) -> list[dict[str, np.ndarray]]:
    records = []
    for _ in fit.FIT_CONTACTS:
        force = np.zeros(fit.RETAINED_SAMPLES, dtype=np.float64)
        if impulse:
            force[fit.ONSET_SAMPLE] = 1.0
        records.append(
            {
                "force": force,
                "baseline_force": np.zeros(fit.BASELINE_SAMPLES, dtype=np.float64),
            }
        )
    return records


class RealFrfFitTests(unittest.TestCase):
    def test_manifest_freezes_roles_budgets_and_single_record_coherence(self) -> None:
        manifest = fit.expected_manifest()
        self.assertEqual(manifest["roles"]["fit"], list(fit.FIT_CONTACTS))
        self.assertEqual(len(manifest["roles"]["protected"]), 19)
        self.assertEqual(manifest["sample_budget"]["microphone_decoded"], 4_608_000)
        self.assertEqual(manifest["sample_budget"]["force_decoded"], 4_608_000)
        self.assertEqual(manifest["sample_budget"]["total_decoded"], 9_216_000)
        self.assertEqual(
            manifest["candidate"]["coherence"], "NOT_APPLICABLE_SINGLE_RECORD"
        )

    def test_freeze_and_repeated_preflight_are_byte_identical_and_zero_read(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            frozen = fit.freeze(root / "freeze")
            first = fit.preflight(frozen / "manifest.json", root / "preflight-a")
            second = fit.preflight(frozen / "manifest.json", root / "preflight-b")
            first_bytes = (first / "report.json").read_bytes()
            second_bytes = (second / "report.json").read_bytes()
            report = json.loads(first_bytes)
        self.assertEqual(first_bytes, second_bytes)
        self.assertEqual(report["decision"], "C4A_REAL_FRF_FIT_FROZEN")
        self.assertEqual(report["counters"]["source_bytes_read"], 0)
        self.assertEqual(report["counters"]["microphone_samples_decoded"], 0)
        self.assertEqual(report["counters"]["force_samples_decoded"], 0)
        self.assertEqual(report["counters"]["protected_samples_decoded"], 0)
        self.assertFalse(report["fit_decode_authorized"])

    def test_manifest_mutation_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            frozen = fit.freeze(root / "freeze")
            path = frozen / "manifest.json"
            manifest = json.loads(path.read_text())
            manifest["roles"]["fit"][0] = 0
            path.write_bytes(fit.canonical_json(manifest))
            with self.assertRaisesRegex(fit.FitError, "manifest or implementation changed"):
                fit.validate_manifest(path)

    def test_output_inside_repository_is_rejected(self) -> None:
        output = fit.repository_root() / "forbidden-c4-output"
        with self.assertRaisesRegex(fit.FitError, "outside the repository"):
            fit.prepare_output(output)

    def test_paired_preprocessing_uses_force_onset_and_preserves_alignment(self) -> None:
        microphone, force = synthetic_source_pair()
        processed = fit.preprocess_pair(microphone, force)
        self.assertEqual(processed["onset"], 24_000)
        self.assertEqual(int(np.flatnonzero(processed["force"])[0]), fit.ONSET_SAMPLE)
        self.assertGreater(abs(processed["microphone"][fit.ONSET_SAMPLE + 10]), 0.0)
        self.assertLess(processed["tail_rms_ratio"], 0.10)

    def test_double_impact_is_ood(self) -> None:
        microphone, force = synthetic_source_pair(second_impact=True)
        with self.assertRaisesRegex(fit.FitError, "double impact"):
            fit.preprocess_pair(microphone, force)

    def test_force_certificate_accepts_broadband_impulses(self) -> None:
        certificate = fit.force_certificate(certificate_records(impulse=True))
        self.assertTrue(certificate["passed"])
        self.assertEqual(certificate["valid_contact_count"], 16)
        self.assertTrue(all(item["passed"] for item in certificate["leave_quarter_out"]))

    def test_force_certificate_rejects_zero_and_admits_no_poles(self) -> None:
        certificate = fit.force_certificate(certificate_records(impulse=False))
        self.assertFalse(certificate["passed"])
        self.assertEqual(certificate["valid_contact_count"], 0)
        self.assertEqual(certificate["supported_fraction"], 0.0)

    def test_local_certificate_fraction_obeys_frozen_neighborhood(self) -> None:
        frequencies = np.fft.rfftfreq(fit.FFT_SAMPLES, 1.0 / fit.SAMPLE_RATE_HZ)
        mask = np.abs(frequencies - 1_000.0) <= 30.0
        fraction, radius = fit.local_certificate_fraction(1_000.0, 10.0, mask)
        self.assertEqual(radius, 24.0)
        self.assertEqual(fraction, 1.0)


if __name__ == "__main__":
    unittest.main()
