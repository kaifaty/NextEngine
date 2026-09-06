"""Small exact controls for physical subspace correction, no large FEM run."""

import sys
import unittest
from pathlib import Path

import numpy as np
from scipy.sparse import diags, eye

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_ritz_probe as probe


class RitzTests(unittest.TestCase):
    def test_basis_rotation_and_sign_do_not_change_spectrum_or_projector(self):
        m = diags([1.0, 2.0, 3.0, 4.0, 5.0])
        k = diags([0.0, 2.0, 6.0, 12.0, 20.0])
        rigid = np.eye(5)[:, :1]
        v = np.eye(5)[:, 1:4]
        a, u, residual = probe.ritz(v, k, m, rigid)
        mix = np.array([[1.0, 2.0, 0.0], [0.0, -1.0, 1.0], [1.0, 0.0, 1.0]])
        b, w, _ = probe.ritz(v @ mix, k, m, rigid)
        np.testing.assert_allclose(a, [1, 2, 3], atol=1e-12)
        np.testing.assert_allclose(a, b, atol=1e-12)
        np.testing.assert_allclose(u @ u.T, w @ w.T, atol=1e-12)
        self.assertLess(residual.max(), 1e-12)

    def test_rank_loss_is_not_silently_repaired(self):
        with self.assertRaisesRegex(ValueError, "rank deficient"):
            probe.mass_basis(np.ones((5, 2)), eye(5))

    def test_rigid_projection_removes_translation_rotation(self):
        points = np.array(
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        )
        r = probe.mass_basis(probe.rigid_fields(points), eye(12))
        np.testing.assert_allclose(r.T @ r, np.eye(6), atol=1e-12)
        np.testing.assert_allclose(probe.remove_rigid(r, r, eye(12)), 0, atol=1e-12)

    def test_inaudible_candidate_is_rejected_but_reference_failure_is_fatal(self):
        geometry = {"normals": np.array([[1.0, 0.0, 0.0]]), "length": 0.2, "probe": 0}
        good = {"omega": np.array([0.1]), "field": np.ones((1, 3, 1))}
        bad = {"omega": np.array([20.0]), "field": good["field"]}
        waves, rejected = probe.render_variants(
            {"reference": good, "candidate": bad}, geometry
        )
        self.assertEqual(set(waves), {"reference"})
        self.assertEqual(rejected, {"candidate": "no audible oscillatory modes"})
        with self.assertRaisesRegex(ValueError, "no audible"):
            probe.render_variants({"reference": bad}, geometry)


if __name__ == "__main__":
    unittest.main()
