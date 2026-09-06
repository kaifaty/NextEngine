import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_modal3d_passive as passive
import physical_sound_modal3d_passive_assess as assess


class PassiveTests(unittest.TestCase):
    def test_field_enforces_reciprocity_psd_and_collocation_before_training(self):
        torch.manual_seed(42)
        model = passive.PassiveModes().double()
        x = torch.zeros(2, 3, dtype=torch.float64)
        points = torch.tensor(
            [[[0.55, 0.35], [1, 0.5], [1, 0.5]]] * 2, dtype=torch.float64
        )
        _, a = model(x, points)
        matrix = passive.residues(a)
        self.assertTrue(torch.equal(matrix, matrix.transpose(-1, -2)))
        self.assertTrue(torch.equal(a[:, 1], a[:, 2]))
        self.assertTrue(torch.all(torch.diagonal(matrix, dim1=-2, dim2=-1) >= 0))
        self.assertGreaterEqual(torch.linalg.eigvalsh(matrix).min().item(), -1e-14)
        self.assertTrue(torch.equal(matrix[:, :, 1, 1], matrix[:, :, 1, 2]))
        self.assertFalse(any(p.requires_grad for p in model.body.parameters()))

    def test_mode_sign_gauge_does_not_change_matrix_targets(self):
        a = torch.arange(1, 25, dtype=torch.float64).reshape(1, 3, 8)
        signs = torch.tensor([1, -1] * 4, dtype=torch.float64)
        self.assertTrue(torch.equal(passive.residues(a), passive.residues(a * signs)))

    def test_multiport_power_balance_for_arbitrary_forces(self):
        rng = np.random.default_rng(42)
        a = rng.normal(size=(4, 8))
        q = rng.normal(size=8)
        v = rng.normal(size=8)
        omega = np.arange(1, 9, dtype=float)
        d = np.arange(1, 9) / 10
        force = rng.normal(size=4)
        acceleration = a.T @ force - 2 * d * v - omega**2 * q
        rate = v @ (acceleration + omega**2 * q)
        port_power = force @ (a @ v)
        np.testing.assert_allclose(rate, port_power - np.sum(2 * d * v * v), rtol=1e-12)

    def test_matrix_interpolation_is_psd_and_exact_on_grid(self):
        rows = []
        for shape in passive.pilot.TRAIN_SHAPES:
            modes = np.array(
                [
                    np.arange(1, 9) * (sum(shape) + sum(p))
                    for p in passive.pilot.TRAIN_CONTACTS
                ]
            )
            modes = np.vstack([modes, modes[7]])
            rows.append(
                {
                    "shape": np.array(shape),
                    "omega": np.arange(1, 9, dtype=float),
                    "port_modes": modes,
                }
            )
        f, m = assess.interpolate(rows)
        np.testing.assert_allclose(f([passive.pilot.DEV_SHAPES[0]])[0], np.arange(1, 9))
        shape = passive.pilot.TRAIN_SHAPES[0]
        point = passive.pilot.TRAIN_CONTACTS[0]
        np.testing.assert_allclose(
            m([list(shape) + list(point)])[0],
            assess.port_matrix(rows[0]["port_modes"][[0, -1]]),
        )
        result = m(
            [list(passive.pilot.DEV_SHAPES[0]) + list(passive.pilot.DEV_CONTACTS[0])]
        )[0]
        self.assertGreaterEqual(np.linalg.eigvalsh(result).min(), -1e-10)
        with self.assertRaises(ValueError):
            assess.interpolate(rows[:-1])

    def test_target_poisson_is_explicit_without_changing_legacy_default(self):
        p = passive.impact
        a = p.parameters(**p.BASE)
        b = p.parameters(**p.BASE, target_poisson=p.SHAPE[2])
        self.assertEqual(a, b)
        self.assertNotEqual(
            a["stiffness"], p.parameters(**p.BASE, target_poisson=0.32)["stiffness"]
        )
        with self.assertRaises(ValueError):
            p.parameters(**p.BASE, target_poisson=0.5)


if __name__ == "__main__":
    unittest.main()
