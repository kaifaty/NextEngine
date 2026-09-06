from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v4_common as common
import physical_sound_contact_field_r3a_v4_preflight as preflight


class PhysicalSoundContactFieldR3AV4Tests(unittest.TestCase):
    def test_frontier_is_bounded_nested_and_uses_only_opened_objects(self) -> None:
        self.assertEqual(len(common.CAPACITIES), 3)
        for field in (
            "mode_count",
            "long_residual_rank",
            "spectral_residual_bins",
            "transient_residual_rank",
        ):
            values = [item[field] for item in common.CAPACITIES]
            self.assertEqual(values, sorted(values))
            self.assertEqual(len(values), len(set(values)))
        self.assertEqual(len(common.PARENT_OBJECTS), 4)
        self.assertTrue(
            all(item["development_lineage"] for item in common.PARENT_OBJECTS)
        )
        self.assertEqual(common.SEALED_CONTACT_INDEX, 4)

    def test_overlap_add_is_an_exact_deterministic_inverse(self) -> None:
        generator = np.random.Generator(np.random.PCG64(7))
        value = generator.standard_normal(11_111)
        frames = common.analysis_frames(value, 512, 256)
        first = common.synthesis_frames(frames, value.size, 512, 256)
        second = common.synthesis_frames(frames, value.size, 512, 256)
        np.testing.assert_array_equal(first, second)
        np.testing.assert_allclose(first, value, rtol=0.0, atol=2.0e-15)

    def test_randomized_basis_is_exact_for_frozen_seed(self) -> None:
        generator = np.random.Generator(np.random.PCG64(11))
        frames = generator.standard_normal((48, 32))
        first = common.randomized_basis(frames, 8, 99)
        second = common.randomized_basis(frames, 8, 99)
        np.testing.assert_array_equal(first, second)
        np.testing.assert_allclose(first @ first.T, np.eye(8), atol=1.0e-12)

    def test_fit_only_normalization_cannot_see_development_peak(self) -> None:
        contacts = np.zeros((4, common.ANALYSIS_SAMPLES), dtype=np.float64)
        contacts[0, 0] = 0.25
        contacts[1, 1] = -0.50
        contacts[2, 2] = 0.125
        contacts[3, 3] = 100.0
        normalized, scale = common.normalize_fit_group(contacts)
        self.assertEqual(scale, 0.92 / 0.50)
        self.assertEqual(float(np.max(np.abs(normalized[:3]))), 0.92)
        self.assertGreater(float(np.max(np.abs(normalized[3]))), 100.0)

    def test_modal_contact_record_uses_float32_cooked_budget(self) -> None:
        value = np.zeros(common.ANALYSIS_SAMPLES, dtype=np.float64)
        poles = np.asarray([[880.0, 8.0, 0.0]], dtype=np.float64)
        long_basis = np.zeros((1, common.LONG_FRAME_SAMPLES), dtype=np.float64)
        long_basis[0, 1] = 1.0
        transient_basis = np.zeros(
            (1, common.TRANSIENT_FRAME_SAMPLES), dtype=np.float64
        )
        transient_basis[0, 1] = 1.0
        spectral_bins = np.asarray([20], dtype=np.int64)
        _, record = common.cook_contact(
            value, poles, long_basis, spectral_bins, transient_basis
        )
        expected_count = (
            2
            + record["long_coefficients"].size
            + record["spectral_coefficients"].size * 2
            + record["transient_coefficients"].size
            + 1
        )
        self.assertEqual(record["encoded_float32_count"], expected_count)
        self.assertEqual(record["encoded_bytes"], expected_count * 4)

    def test_candidate_gate_requires_absolute_quality_and_baseline_gain(self) -> None:
        candidate = {
            endpoint: 0.1 * threshold
            for endpoint, threshold in common.ABSOLUTE_THRESHOLDS.items()
        }
        baseline = {
            endpoint: 2.0 * threshold
            for endpoint, threshold in common.ABSOLUTE_THRESHOLDS.items()
        }
        self.assertTrue(common.candidate_gate(candidate, baseline)["passed"])
        candidate["modal_frequency_median_error_cents"] = 101.0
        self.assertFalse(common.candidate_gate(candidate, baseline)["passed"])

    def test_manifest_freezes_no_new_or_sealed_object_access(self) -> None:
        objects = [
            preflight.public_object_descriptor(
                {
                    **item,
                    "directory": "/external/omitted",
                    "roles": ["fit", "fit", "fit", "representation_development"],
                    "sealed_row": 2407,
                    "sealed_waveform_samples_decoded": 0,
                }
            )
            for item in common.PARENT_OBJECTS
        ]
        manifest = preflight.build_manifest(
            {key: "0" * 64 for key in common.IMPLEMENTATION_FILES},
            {"test": True},
            objects,
        )
        common.validate_manifest(manifest)
        self.assertFalse(manifest["new_object_or_sealed_waveform_access_authorized"])
        self.assertFalse(manifest["neural_training_authorized"])
        self.assertNotIn("directory", manifest["objects"][0])


if __name__ == "__main__":
    unittest.main()
