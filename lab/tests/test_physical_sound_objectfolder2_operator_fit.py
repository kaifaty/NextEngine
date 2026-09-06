"""Physical training objective and differentiability controls."""

import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

import numpy as np
import torch
from scipy.sparse import diags

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_operator_fit as fit


class OperatorFitTests(unittest.TestCase):
    def test_operator_weights_cannot_use_independent_frequency_renderer(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)
            fit.shared.write(path / "fit.json", {"requires_operator_ritz": True})
            with self.assertRaisesRegex(ValueError, "require Ritz"):
                fit.shared.render(SimpleNamespace(fit=path))

    def test_trace_exact_and_invertible_basis_invariant(self):
        k = fit.sparse(diags([0.0, 2.0, 6.0, 12.0, 20.0]), "cpu")
        m = fit.sparse(diags([1.0, 2.0, 3.0, 4.0, 5.0]), "cpu")
        rigid = torch.eye(5, dtype=torch.float64)[:, :1]
        v = torch.eye(5, dtype=torch.float64)[:, 1:4]
        mix = torch.tensor(
            [[1.0, 2.0, 0.0], [0.0, -1.0, 1.0], [1.0, 0.0, 1.0]], dtype=torch.float64
        )
        a, b = [fit.rayleigh_trace(x, k, m, rigid) for x in (v, v @ mix)]
        self.assertAlmostEqual(float(a), 6.0, places=10)
        torch.testing.assert_close(a, b)

    def test_gradient_matches_finite_difference_and_rigid_pollution_is_removed(self):
        k = fit.sparse(diags([0.0, 2.0, 6.0, 12.0, 20.0]), "cpu")
        m = fit.sparse(diags([1.0, 2.0, 3.0, 4.0, 5.0]), "cpu")
        rigid = torch.eye(5, dtype=torch.float64)[:, :1]
        v = torch.tensor(
            np.random.default_rng(4).normal(size=(5, 2)), requires_grad=True
        )
        self.assertTrue(
            torch.autograd.gradcheck(lambda x: fit.rayleigh_trace(x, k, m, rigid), (v,))
        )
        torch.testing.assert_close(
            fit.rayleigh_trace(v, k, m, rigid),
            fit.rayleigh_trace(v + rigid * 7.0, k, m, rigid),
        )


if __name__ == "__main__":
    unittest.main()
