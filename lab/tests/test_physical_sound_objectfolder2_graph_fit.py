"""Shared graph correction controls independent of any training result."""

import sys
import unittest
from pathlib import Path

import numpy as np
import torch
from scipy.sparse import diags

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_graph_fit as graph


class GraphTests(unittest.TestCase):
    def test_aggregation_preserves_constant_physical_components(self):
        points = np.array([[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.4, 0.4, 0.4]])
        k = diags(np.arange(1.0, 10.0))
        levels = graph.hierarchy(k, points)
        x = torch.tensor(np.tile([1.0, 2.0, 3.0], 3))[:, None]
        for level in levels[:-1]:
            coarse = torch.sparse.mm(level["restriction"], x)
            torch.testing.assert_close(coarse[level["parent"]], x)
            x = coarse

    def test_graph_is_zero_safe_linear_and_initially_identity(self):
        torch.manual_seed(42)
        model = graph.GraphCorrection().eval().requires_grad_(False)
        levels = graph.hierarchy(
            diags(np.arange(1.0, 13.0)), np.arange(12.0).reshape(4, 3) / 20
        )
        x = torch.arange(24.0, dtype=torch.float64).reshape(12, 2)
        torch.testing.assert_close(model(x, levels), x, rtol=0, atol=0)
        model.output.weight.fill_(0.1)
        y = x.flip(0)
        torch.testing.assert_close(
            model(x + y, levels), model(x, levels) + model(y, levels)
        )
        torch.testing.assert_close(model(-x, levels), -model(x, levels), rtol=0, atol=0)
        self.assertFalse(model(torch.zeros_like(x), levels).any())

    def test_classical_budget_never_exceeds_declared_sparse_work(self):
        k, m = diags(np.arange(1.0, 13.0)), diags(np.ones(12))
        levels = graph.hierarchy(k, np.arange(12.0).reshape(4, 3) / 20)
        steps, work = graph.classical_steps(levels, k, m)
        self.assertLessEqual(steps * (k.nnz + m.nnz), work)


if __name__ == "__main__":
    unittest.main()
