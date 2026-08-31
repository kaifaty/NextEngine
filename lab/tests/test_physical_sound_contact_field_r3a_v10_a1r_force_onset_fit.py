#!/usr/bin/env python3
"""Focused guards for the frozen V10 A1R force-onset fit runner."""

from __future__ import annotations

import io
import struct
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v10_a1r_force_onset_fit as fit  # noqa: E402


def pcm_wave(values: np.ndarray) -> bytes:
    payload = np.asarray(values, dtype="<i2").tobytes()
    return (
        b"RIFF"
        + struct.pack("<I", 36 + len(payload))
        + b"WAVEfmt "
        + struct.pack("<IHHIIHH", 16, 1, 1, 48_000, 96_000, 2, 16)
        + b"data"
        + struct.pack("<I", len(payload))
        + payload
    )


class A1RForceOnsetFitTests(unittest.TestCase):
    def test_force_onset_uses_frozen_peak_and_noise_thresholds(self) -> None:
        value = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        value[20_000] = 0.5
        onset, threshold, peak_threshold, noise_threshold, peak = (
            fit.force_onset_sample(value)
        )
        self.assertEqual(onset, 20_000)
        self.assertEqual(peak, 0.5)
        self.assertEqual(threshold, peak_threshold)
        self.assertEqual(peak_threshold, 0.025)
        self.assertEqual(noise_threshold, 0.0)

    def test_force_onset_includes_threshold_equality(self) -> None:
        value = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        value[12_000] = 0.05
        value[20_000] = 1.0
        onset, *_ = fit.force_onset_sample(value)
        self.assertEqual(onset, 12_000)

    def test_pair_alignment_uses_one_force_derived_shift_without_wrap(self) -> None:
        microphone = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        force = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        microphone[20_005] = 0.25
        force[20_000] = 0.5
        aligned_microphone, aligned_force, descriptor = fit.preprocess_pair(
            microphone, force
        )
        self.assertEqual(descriptor["force_onset_sample"], 20_000)
        self.assertEqual(descriptor["alignment_shift_samples"], -19_488)
        self.assertEqual(aligned_force[fit.ALIGNMENT_SAMPLE], 0.5)
        self.assertEqual(aligned_microphone[fit.ALIGNMENT_SAMPLE + 5], 0.25)
        self.assertEqual(aligned_microphone[0], 0.0)

    def test_zero_force_rejects_before_representation(self) -> None:
        with self.assertRaisesRegex(fit.FitError, "threshold is invalid"):
            fit.force_onset_sample(np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64))

    def test_extractor_reads_only_committed_fit_pair(self) -> None:
        microphone = pcm_wave(np.zeros(fit.SOURCE_SAMPLES, dtype=np.int16))
        force = pcm_wave(np.ones(fit.SOURCE_SAMPLES, dtype=np.int16))
        protected = pcm_wave(np.full(fit.SOURCE_SAMPLES, 2, dtype=np.int16))
        members = [
            ("51/audio/27/mic.wav", microphone),
            ("51/audio/27/Force.wav", force),
            ("51/audio/9/mic.wav", protected),
        ]
        commitments = {
            name: (len(payload), fit.representation.sha256_bytes(payload))
            for name, payload in members[:2]
        }
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "prefix.tar.gz"
            with tarfile.open(path, "w:gz") as archive:
                for name, payload in members:
                    info = tarfile.TarInfo(name)
                    info.size = len(payload)
                    info.mtime = 0
                    archive.addfile(info, io.BytesIO(payload))
            original = fit.FIT_CONTACT_IDS
            try:
                fit.FIT_CONTACT_IDS = (27,)
                found = fit.extract_fit_pairs(path, commitments)
            finally:
                fit.FIT_CONTACT_IDS = original
        self.assertEqual(tuple(found), (27,))
        self.assertEqual(set(found[27]), {"microphone", "force"})

    def test_frozen_representation_hashes_match_repository(self) -> None:
        observed = fit.validate_implementation(fit.repository_root())
        self.assertEqual(
            observed["representation"], fit.REPRESENTATION_IMPLEMENTATION_SHA256
        )
        self.assertEqual(observed["protocol"], fit.PROTOCOL_SHA256)

    def test_output_inside_repository_is_rejected(self) -> None:
        root = fit.repository_root()
        with self.assertRaisesRegex(fit.FitError, "outside the repository"):
            fit.representation.prepare_output(root, root / "forbidden-a1r-output")


if __name__ == "__main__":
    unittest.main()
