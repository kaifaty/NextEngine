from __future__ import annotations

import math
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v9_common as common
import physical_sound_contact_field_r3a_v9_model as model
import physical_sound_contact_field_r3a_v9_preflight as preflight


class PhysicalSoundContactFieldR3AV9Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.coordinates = model.grid_coordinates()
        cls.latents = model.truth_latents(cls.coordinates)
        cls.gains = model.truth_modal_gains(cls.coordinates)
        cls.bands = model.generate_noise_bands()
        cls.atoms = model.residual_atoms()

    def test_noise_bands_repeat_and_have_frozen_shape(self) -> None:
        repeated = model.generate_noise_bands()
        self.assertTrue(np.array_equal(self.bands, repeated))
        self.assertEqual(
            self.bands.shape,
            (common.NOISE_BAND_COUNT, common.NOISE_LOOP_SAMPLES),
        )
        self.assertTrue(np.isfinite(self.bands).all())
        self.assertTrue(np.allclose(np.sqrt(np.mean(self.bands**2, axis=1)), 1.0))

    def test_grid_split_and_truth_are_finite(self) -> None:
        context = model.farthest_point_indices(
            self.coordinates, common.CONTEXT_COUNT
        )
        self.assertEqual(self.coordinates.shape, (49, 3))
        self.assertEqual(len(np.unique(context)), common.CONTEXT_COUNT)
        self.assertEqual(self.latents.shape, (49, common.RESIDUAL_ATOM_COUNT))
        self.assertGreater(float(np.min(self.latents)), 0.0)

    def test_contact_record_is_small_and_reconstructs_truth(self) -> None:
        record, decoded = model.encode_contact_record(
            self.coordinates[0], self.gains[0], self.latents[0]
        )
        target = model.render_contact(
            self.gains[0], self.latents[0], self.bands, self.atoms
        )
        candidate = model.render_contact(
            decoded["gains"], decoded["latent"], self.bands, self.atoms
        )
        self.assertLess(len(record), common.MAXIMUM_CONTACT_BYTES)
        self.assertLessEqual(
            preflight.normalized_rmse(target, candidate),
            common.GATES["maximum_record_waveform_nrmse"],
        )

    def test_modal_and_residual_renderers_are_distinct_and_finite(self) -> None:
        modal = model.render_modal(self.gains[3])
        residual = model.render_residual(self.latents[3], self.bands, self.atoms)
        self.assertTrue(np.isfinite(modal).all())
        self.assertTrue(np.isfinite(residual).all())
        self.assertEqual(modal.shape, (common.SAMPLE_COUNT,))
        self.assertTrue(np.all(modal[: common.ALIGNMENT_SAMPLE] == 0.0))
        self.assertGreater(float(np.sqrt(np.mean(residual**2))), 0.0)
        self.assertFalse(np.array_equal(modal, residual))

    def test_latent_field_repeats_and_beats_constant(self) -> None:
        first, prediction_a, nearest_a, query_a = model.train_latent_field(
            self.coordinates, self.latents
        )
        second, prediction_b, nearest_b, query_b = model.train_latent_field(
            self.coordinates, self.latents
        )
        self.assertEqual(first, second)
        self.assertTrue(np.array_equal(prediction_a, prediction_b))
        self.assertTrue(np.array_equal(nearest_a, nearest_b))
        self.assertTrue(np.array_equal(query_a, query_b))
        self.assertTrue(first["finite"])
        self.assertTrue(first["neural_below_constant"])

    def test_manifest_forbids_real_and_runtime_authority(self) -> None:
        manifest = preflight.build_manifest(
            {"environment": "fixture"},
            {key: "0" * 64 for key in common.IMPLEMENTATION_FILES},
        )
        self.assertFalse(manifest["source"]["real_audio"])
        self.assertEqual(manifest["real_waveform_sample_values_decoded"], 0)
        self.assertEqual(
            manifest["prior_v8_development_waveform_sample_values_decoded"], 0
        )
        self.assertFalse(manifest["real_quality_credit_authorized"])
        self.assertFalse(manifest["r3b_authorized"])
        self.assertFalse(manifest["runtime_neural_inference_authorized"])
        self.assertTrue(manifest["authored_clip_fallback_required"])

    def test_output_must_stay_external_and_unique(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "result"
            resolved, staging = common.prepare_output(
                common.repository_root(), output
            )
            self.assertFalse(resolved.exists())
            self.assertTrue(staging.is_dir())
            staging.rmdir()
        with self.assertRaises(common.V9Error):
            common.prepare_output(
                common.repository_root(), common.repository_root() / "forbidden"
            )


if __name__ == "__main__":
    unittest.main()

