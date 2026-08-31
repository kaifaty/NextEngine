from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v8_real_fit as fit


class PhysicalSoundContactFieldR3AV8RealFitTests(unittest.TestCase):
    def test_fit_member_paths_exclude_development_and_sealed(self) -> None:
        paths = fit.fit_member_paths()
        self.assertEqual(len(paths), 12)
        self.assertTrue(all("/20/" not in path for path in paths))
        self.assertTrue(all("/27/" not in path for path in paths))

    def test_force_alignment_uses_shared_shift_and_native_length(self) -> None:
        audio = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        force = np.zeros(fit.SOURCE_SAMPLES, dtype=np.float64)
        onset = 1_500
        force[onset : onset + 4] = [0.3, 1.0, 0.5, 0.1]
        audio[onset + 10] = 0.8
        aligned_audio, aligned_force, descriptor = fit.preprocess_contact(audio, force)
        self.assertEqual(descriptor["source_onset_sample"], onset)
        self.assertEqual(descriptor["alignment_shift_samples"], fit.ALIGNMENT_SAMPLE - onset)
        self.assertEqual(aligned_audio.shape, (fit.ANALYSIS_SAMPLES,))
        self.assertEqual(aligned_force.shape, (fit.ANALYSIS_SAMPLES,))
        self.assertEqual(int(np.argmax(np.abs(aligned_force))), fit.ALIGNMENT_SAMPLE + 1)
        self.assertAlmostEqual(float(np.max(np.abs(aligned_force))), 1.0)
        self.assertEqual(int(np.argmax(np.abs(aligned_audio))), fit.ALIGNMENT_SAMPLE + 10)

    def test_quantizers_are_finite_and_bounded(self) -> None:
        value = np.linspace(-3.0, 2.0, 1_000, dtype=np.float64)
        q16, scale16, decoded16 = fit.quantize_int16(value)
        qf, scalef, decodedf = fit.quantize_float16_scaled(value)
        self.assertEqual(q16.dtype, np.dtype("<i2"))
        self.assertEqual(qf.dtype, np.dtype("<f2"))
        self.assertTrue(np.isfinite(decoded16).all())
        self.assertTrue(np.isfinite(decodedf).all())
        self.assertGreater(float(scale16), 0.0)
        self.assertGreater(float(scalef), 0.0)
        self.assertLess(float(np.max(np.abs(value - decoded16))), 0.001)

    def test_residual_bin_selection_is_stable_and_in_band(self) -> None:
        time = np.arange(fit.ANALYSIS_SAMPLES) / fit.SAMPLE_RATE_HZ
        rows = np.stack(
            [
                np.sin(2.0 * math.pi * 440.0 * time),
                np.sin(2.0 * math.pi * 880.0 * time),
                np.sin(2.0 * math.pi * 1_760.0 * time),
            ]
        )
        first = fit.select_residual_bins(rows, count=128)
        second = fit.select_residual_bins(rows, count=128)
        frequencies = np.fft.rfftfreq(fit.ANALYSIS_SAMPLES, 1.0 / fit.SAMPLE_RATE_HZ)
        self.assertTrue(np.array_equal(first, second))
        self.assertTrue(np.all(np.diff(first) > 0))
        self.assertGreaterEqual(float(np.min(frequencies[first])), 120.0)
        self.assertLessEqual(float(np.max(frequencies[first])), 18_000.0)

    def test_contact_record_matches_frozen_byte_budget(self) -> None:
        target = np.zeros(fit.ANALYSIS_SAMPLES, dtype=np.float64)
        target[fit.ALIGNMENT_SAMPLE] = 0.5
        modal = np.zeros_like(target)
        spectrum = np.fft.rfft(target)
        selected = np.arange(1, fit.RESIDUAL_BIN_COUNT + 1, dtype="<u4")
        gains = np.zeros((fit.MODE_COUNT, 2), dtype="<f2")
        global_candidate, global_record, _ = fit.encode_contact(
            target,
            modal,
            spectrum,
            selected,
            gains,
            np.float32(1.0),
            None,
        )
        contact_candidate, contact_record, _ = fit.encode_contact(
            target,
            modal,
            spectrum,
            selected,
            gains,
            np.float32(1.0),
            np.ones(fit.MODE_COUNT, dtype="<f2"),
        )
        self.assertEqual(len(global_record), 63_392)
        self.assertEqual(len(contact_record), 63_520)
        self.assertLess(len(contact_record), fit.MAXIMUM_CONTACT_BYTES)
        self.assertTrue(np.isfinite(global_candidate).all())
        self.assertTrue(np.isfinite(contact_candidate).all())

    def test_measured_force_modal_fit_reduces_synthetic_error(self) -> None:
        force = np.zeros((1, fit.ANALYSIS_SAMPLES), dtype=np.float64)
        force[0, fit.ALIGNMENT_SAMPLE] = 1.0
        frequencies = np.asarray([440.0, 1_200.0])
        damping = np.asarray([[4.0, 12.0]])
        expected_gains = np.asarray([[[0.8, -0.2], [0.3, 0.1]]])
        target = fit.synthesize_modal(force, frequencies, damping, expected_gains)
        gains, residual = fit.fit_modal_gains(
            target, force, frequencies, damping, np.asarray([0, 1])
        )
        candidate = fit.synthesize_modal(force, frequencies, damping, gains)
        self.assertLess(float(np.mean(np.square(residual))), 1.0e-8)
        self.assertLess(float(np.mean(np.square(candidate - target))), 1.0e-8)

    def test_manifest_forbids_development_and_runtime(self) -> None:
        manifest = fit.build_manifest(
            {"environment": "fixture"},
            {"fit": "0" * 64, "inventory": "1" * 64, "endpoints": "2" * 64},
        )
        self.assertEqual(manifest["fit_waveform_sample_values_decoded"], 0)
        self.assertEqual(manifest["development_waveform_sample_values_decoded"], 0)
        self.assertEqual(manifest["sealed_waveform_sample_values_decoded"], 0)
        self.assertFalse(manifest["real_development_authorized"])
        self.assertFalse(manifest["r3b_authorized"])
        self.assertFalse(manifest["runtime_neural_inference_authorized"])


if __name__ == "__main__":
    unittest.main()
