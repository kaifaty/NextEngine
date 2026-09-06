"""Shared physical representation and existing-cohort guards."""

import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_elastic as meshing
import physical_sound_objectfolder2_physical_shared as shared


class SharedPhysicalTests(unittest.TestCase):
    def test_rank_one_physical_matrices_ignore_mode_signs(self):
        f = torch.arange(2 * 4 * 3 * 32, dtype=torch.float32).reshape(2, 4, 3, 32) / 100
        sign = torch.where(torch.arange(32) % 2 == 0, -1.0, 1.0)
        matrices = shared.gram(f)
        torch.testing.assert_close(matrices, shared.gram(f * sign), rtol=0, atol=0)
        torch.testing.assert_close(matrices, matrices.transpose(-1, -2), rtol=0, atol=0)
        self.assertTrue((matrices.diagonal(dim1=-1, dim2=-2) >= 0).all())

    def test_farthest_points_are_unique_and_translation_scale_stable(self):
        p = np.array(
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 3.0],
                [0.1, 0.2, 0.3],
            ]
        )
        a = shared.farthest(p, 4)
        np.testing.assert_array_equal(a, shared.farthest(p * 2 + 3, 4))
        self.assertEqual(len(set(a)), 4)
        with self.assertRaises(ValueError):
            shared.farthest(p, 6)

    def test_frequency_is_not_contact_conditioned(self):
        torch.manual_seed(4)
        model = shared.PhysicalStudent().eval().requires_grad_(False)
        cloud, features, points = (
            torch.zeros(1, 512, 3),
            torch.zeros(1, 4),
            torch.rand(1, 4, 3),
        )
        w, f = model(cloud, features, points)
        w2, f2 = model(cloud, features, points.flip(1))
        torch.testing.assert_close(w, w2, rtol=0, atol=0)
        torch.testing.assert_close(f, f2.flip(1))
        self.assertEqual(f.shape, (1, 4, 3, 32))

    def test_cohort_role_cannot_be_relabelled(self):
        manifest = {
            "rows": [{"object_id": 59, "role": "development", "material": "Ceramic"}]
        }
        with self.assertRaisesRegex(ValueError, "role"):
            meshing.source_mesh(manifest, 59)
        with self.assertRaisesRegex(ValueError, "cohort"):
            meshing.source_mesh({}, 11)

    def test_meter_response_is_sign_invariant_and_impulse_linear(self):
        g = {
            "normals": np.array([[1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]),
            "probe": 0,
            "length": 0.2,
        }
        w = np.array([0.1, 0.2])
        f = np.arange(12.0).reshape(2, 3, 2) / 10
        a = shared.physical_waves(w, f, g)
        np.testing.assert_array_equal(a, shared.physical_waves(w, -f, g))
        np.testing.assert_array_equal(2 * a, shared.physical_waves(w, f, g, 0.002))
        self.assertFalse(shared.physical_waves(w, f, g, 0).any())


if __name__ == "__main__":
    unittest.main()
