from __future__ import annotations

import hashlib
import math
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v8_common as common
import physical_sound_contact_field_r3a_v8_model as model
import physical_sound_contact_field_r3a_v8_preflight as preflight


def synthetic_feature_arrays(point_count: int = 96) -> dict[str, np.ndarray]:
    index = np.arange(point_count, dtype=np.int64)
    coords = np.column_stack(
        [index % 32, (index * 7) % 32, (index * 13) % 32]
    )
    surface = np.zeros((point_count, 6), dtype=np.float64)
    surface[np.arange(point_count), index % 6] = 1.0
    normalized = coords.astype(np.float64) / 31.0
    features = np.empty((point_count, 3, 20), dtype=np.float64)
    for axis in range(3):
        for mode_index in range(20):
            frequency = 1.0 + mode_index % 4
            features[:, axis, mode_index] = (
                np.sin(math.pi * frequency * normalized[:, axis])
                + 0.1 * surface[:, (axis + mode_index) % 6]
            )
    return {
        "coords": coords,
        "feats_in": features,
        "surface": surface,
        "freqs": np.linspace(250.0, 12_000.0, 20, dtype=np.float32),
    }


class PhysicalSoundContactFieldR3AV8Tests(unittest.TestCase):
    def test_modal_renderer_is_differentiable_and_finite(self) -> None:
        time = torch.arange(256, dtype=torch.float64) / 16_000.0
        frequency = torch.tensor([440.0, 1_220.0], dtype=torch.float64)
        damping = torch.tensor([4.0, 12.0], dtype=torch.float64)
        gains = torch.tensor([0.8, -0.3], dtype=torch.float64, requires_grad=True)
        value = model.render_modal(frequency, damping, gains, time)
        value.square().mean().backward()
        self.assertEqual(value.shape, (256,))
        self.assertTrue(torch.isfinite(value).all())
        self.assertIsNotNone(gains.grad)
        self.assertTrue(torch.isfinite(gains.grad).all())

    def test_farthest_point_split_is_deterministic_and_unique(self) -> None:
        coords = synthetic_feature_arrays()["coords"]
        first = model.farthest_point_indices(coords, 32)
        second = model.farthest_point_indices(coords[::-1], 32)
        self.assertEqual(len(np.unique(first)), 32)
        self.assertEqual(len(np.unique(second)), 32)
        self.assertTrue(np.array_equal(first, model.farthest_point_indices(coords, 32)))

    def test_position_encoding_has_frozen_dimension(self) -> None:
        arrays = synthetic_feature_arrays(40)
        value = model.encode_position(arrays["coords"], arrays["surface"])
        expected = 3 + 2 * 3 * len(common.FIELD_CONFIG["fourier_bands"]) + 6
        self.assertEqual(value.shape, (40, expected))
        self.assertEqual(value.dtype, np.float32)
        self.assertTrue(np.isfinite(value).all())

    def test_feature_loader_checks_exact_external_hash_and_schema(self) -> None:
        arrays = synthetic_feature_arrays()
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "feature.npz"
            np.savez(path, **arrays)
            descriptor = {
                "object_id": "fixture",
                "role": "test",
                "path": "fixture.npz",
                "bytes": path.stat().st_size,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            }
            value = common.validate_nisr_feature(
                common.repository_root(), path, descriptor
            )
            self.assertEqual(value["point_count"], 96)
            broken = {**descriptor, "sha256": "0" * 64}
            with self.assertRaises(common.V8Error):
                common.validate_nisr_feature(
                    common.repository_root(), path, broken
                )

    def test_modal_recovery_passes_frozen_truth_gate(self) -> None:
        result = model.modal_recovery_control()
        self.assertTrue(result["passed"])
        self.assertLessEqual(result["maximum_frequency_error_cents"], 1.0)
        self.assertLessEqual(result["maximum_relative_damping_error"], 0.02)
        self.assertLessEqual(result["maximum_relative_gain_error"], 0.02)

    def test_smooth_synthetic_mode_shape_field_beats_controls(self) -> None:
        arrays = synthetic_feature_arrays(160)
        source = {
            "object_id": "fixture",
            "coords": arrays["coords"].astype(np.float32),
            "surface": arrays["surface"].astype(np.float32),
            "features": arrays["feats_in"].astype(np.float32),
        }
        result = model.train_mode_shape_field(source)
        self.assertTrue(result["finite"])
        self.assertLess(result["neural_query_rmse"], result["constant_query_rmse"])
        self.assertGreater(result["query_prediction_mean_component_std"], 0.01)

    def test_manifest_forbids_real_quality_and_runtime_authority(self) -> None:
        manifest = preflight.build_manifest(
            {"environment": "fixture"},
            {key: "0" * 64 for key in common.IMPLEMENTATION_FILES},
            [
                {
                    **common.NISR_FILES[0],
                    "point_count": 501,
                    "frequency_min_hz": 1.0,
                    "frequency_max_hz": 2.0,
                    "sha256_observed": common.NISR_FILES[0]["sha256"],
                }
            ],
        )
        self.assertFalse(manifest["real_quality_credit_authorized"])
        self.assertFalse(manifest["r3b_authorized"])
        self.assertFalse(manifest["runtime_neural_inference_authorized"])
        self.assertEqual(manifest["real_v8_waveform_samples_decoded"], 0)
        self.assertTrue(manifest["authored_clip_fallback_required"])


if __name__ == "__main__":
    unittest.main()
