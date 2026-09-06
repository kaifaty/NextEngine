import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_modal3d_pilot as pilot


class Modal3DTests(unittest.TestCase):
    def test_actual_3d_solver_residual_and_physical_scaling(self):
        shape = pilot.DEV_SHAPES[0]
        base = pilot.solve(shape, pilot.DEV_CONTACTS)
        scaled = pilot.solve(shape, pilot.DEV_CONTACTS, length=2, young=4, density=2)
        self.assertLess(base["max_residual"], 1e-7)
        expected = pilot.physical_modes(base["omega"], base["gains"], 2, 4, 2)
        np.testing.assert_allclose(scaled["omega"], expected[0], rtol=1e-7)
        np.testing.assert_allclose(scaled["gains"], expected[1], rtol=1e-5, atol=1e-7)

    def test_contact_changes_participation_not_object_frequencies(self):
        torch.manual_seed(42)
        model = pilot.SharedModes().eval()
        inputs = torch.tensor(
            np.stack([pilot.object_input(pilot.DEV_SHAPES[0])] * 2), dtype=torch.float32
        )
        with torch.no_grad():
            omega, gains = model(
                inputs, torch.tensor(pilot.DEV_CONTACTS[:2], dtype=torch.float32)
            )
        torch.testing.assert_close(omega[0], omega[1], rtol=0, atol=0)
        self.assertGreater((gains[0] - gains[1]).abs().max().item(), 0)

    def test_excitation_linearity_and_silence(self):
        omega = np.array([700, 1700]) * 2 * np.pi
        gains = np.array([1.0, 0.4])
        base = pilot.render_wave(omega, gains)
        np.testing.assert_allclose(
            pilot.render_wave(omega, gains, impulse=0.004),
            base * 2,
            rtol=1e-6,
            atol=1e-9,
        )
        self.assertEqual(np.max(np.abs(pilot.render_wave(omega, gains, impulse=0))), 0)
        self.assertTrue(np.isfinite(base).all())
        self.assertEqual(base.shape, (2 * pilot.RATE,))

    def test_declared_train_dev_combinations_and_contacts_do_not_overlap(self):
        self.assertFalse(set(pilot.TRAIN_SHAPES) & set(pilot.DEV_SHAPES))
        self.assertFalse(
            set(map(tuple, pilot.TRAIN_CONTACTS)) & set(map(tuple, pilot.DEV_CONTACTS))
        )
        self.assertEqual(len(pilot.TRAIN_SHAPES), 36)
        self.assertEqual(len(pilot.DEV_SHAPES), 12)

    def test_invalid_excitation_and_physical_parameters_rejected(self):
        with self.assertRaises(ValueError):
            pilot.physical_modes(np.ones(8), np.ones(8), -1, 1, 1)
        with self.assertRaises(ValueError):
            pilot.render_wave(np.ones(8), np.ones(8), duration=-1)

    def test_stronger_baseline_uses_full_grid_and_reproduces_linear_field(self):
        rows = []
        for shape in pilot.TRAIN_SHAPES:
            rows.append(
                {
                    "shape": np.array(shape),
                    "omega": np.ones(8) * sum(shape),
                    "gains": np.array(
                        [
                            np.ones(8) * (sum(shape) + sum(c))
                            for c in pilot.TRAIN_CONTACTS
                        ]
                    ),
                }
            )
        f, g = pilot.interpolation_baseline(rows)
        shape, contact = pilot.DEV_SHAPES[0], pilot.DEV_CONTACTS[0]
        np.testing.assert_allclose(f([shape])[0], sum(shape), rtol=1e-12)
        np.testing.assert_allclose(
            g([list(shape) + list(contact)])[0], sum(shape) + sum(contact), rtol=1e-12
        )
        with self.assertRaises(ValueError):
            pilot.interpolation_baseline(rows[:-1])
        with self.assertRaises(ValueError):
            pilot.interpolation_baseline([rows[0]] * len(rows))

    def test_frequency_gain_alone_cannot_pass_quality_baseline(self):
        summary = {
            m: {"neural": 0.5, "interpolation": 1.0}
            for m in (
                "frequency_relative_mean",
                "participation_relative_l1",
                "spectrum",
                "envelope",
                "level",
            )
        }
        self.assertTrue(pilot.beats_baseline(summary, "interpolation"))
        summary["spectrum"]["neural"] = 1.5
        self.assertFalse(pilot.beats_baseline(summary, "interpolation"))


if __name__ == "__main__":
    unittest.main()
