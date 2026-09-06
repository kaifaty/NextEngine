"""Feature-oracle span preservation and head reconstruction controls."""

import sys
import unittest
from pathlib import Path

import numpy as np
import torch
from scipy.sparse import diags, eye

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_feature_probe as f


class FeatureProbeTests(unittest.TestCase):
    def test_dictionary_contains_componentwise_linear_head(self):
        h = np.random.default_rng(1).normal(size=(8, 4))
        weights = np.random.default_rng(2).normal(size=(3, 4, 2))
        v = np.einsum("nh,chm->ncm", h, weights).reshape(24, 2)
        np.testing.assert_allclose(f.vector_dictionary(h) @ weights.reshape(12, 2), v)

    def test_enlargement_preserves_head_and_improves_variational_bound(self):
        k, m = diags(np.arange(1.0, 13.0)), eye(12)
        rigid = np.empty((12, 0))
        head = np.eye(12)[:, 7:9]
        dictionary = np.c_[np.eye(12)[:, :5], head, head]
        q, stats = f.expanded_basis(dictionary, head, m, rigid)
        np.testing.assert_allclose(q @ (q.T @ head), head, atol=1e-12)
        w, _, _ = f.lowest(q, k, m, rigid, count=2)
        np.testing.assert_allclose(w, [1.0, 2.0], atol=1e-12)
        self.assertEqual(stats["dimension"], 7)

    def test_hidden_hook_reconstructs_the_actual_network(self):
        torch.manual_seed(42)
        model = f.shared.PhysicalStudent().eval().requires_grad_(False)
        g = {
            "cloud": np.zeros((512, 3), np.float32),
            "features": np.zeros(4, np.float32),
            "points": np.random.default_rng(1).normal(size=(8, 3)).astype(np.float32),
        }
        h, v, error = f.extract(model, g)
        self.assertEqual(h.shape, (8, 129))
        self.assertEqual(v.shape, (24, 32))
        self.assertLess(error, 1e-6)


if __name__ == "__main__":
    unittest.main()
