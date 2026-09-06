import sys
import unittest
from pathlib import Path

import numpy as np
from scipy.special import beta

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_modal3d_impact as impact


class ImpactTests(unittest.TestCase):
    def test_hertz_energy_momentum_and_independent_closed_form(self):
        a = impact.collision(impact.BASE)
        self.assertLess(a["energy_balance_error"], 1e-8)
        self.assertLess(a["momentum_relative_error"], 1e-7)
        np.testing.assert_allclose(
            a["duration"], 0.8 * beta(0.4, 0.5) * a["time_scale"], rtol=1e-8
        )
        np.testing.assert_allclose(a["peak_force"], a["force_scale"], rtol=1e-6)
        np.testing.assert_allclose(
            a["impulse"], 2 * a["mass"] * impact.BASE["velocity"], rtol=1e-7
        )
        self.assertEqual(a["force"](np.array([-1.0, 1.0])).tolist(), [0.0, 0.0])

    def test_velocity_and_both_material_moduli_change_contact_by_scaling_laws(self):
        a = impact.collision(impact.BASE)
        faster = impact.collision({**impact.BASE, "velocity": 1.0})
        np.testing.assert_allclose(
            faster["peak_force"] / a["peak_force"], 2**1.2, rtol=1e-7
        )
        np.testing.assert_allclose(
            faster["duration"] / a["duration"], 2**-0.2, rtol=1e-7
        )
        np.testing.assert_allclose(faster["impulse"] / a["impulse"], 2, rtol=1e-7)
        for key in ("young", "target_young"):
            soft = impact.collision({**impact.BASE, key: 2e9})
            self.assertGreater(soft["duration"], a["duration"])
            self.assertLess(soft["peak_force"], a["peak_force"])
            np.testing.assert_allclose(soft["impulse"], a["impulse"], rtol=1e-7)

    def test_negative_residues_rejected_and_zero_feedback_reproduces_hertz(self):
        with self.assertRaises(ValueError):
            impact.collision(impact.BASE, np.array([1000.0]), np.array([-1.0]))
        a = impact.collision(impact.BASE)
        b = impact.collision(impact.BASE, np.array([1000.0]), np.array([0.0]))
        np.testing.assert_allclose(
            [a["duration"], a["impulse"]], [b["duration"], b["impulse"]], rtol=1e-7
        )

    def test_coupled_energy_balance_and_force_convolution_match_modal_state(self):
        omega = np.array([1000.0, 7000.0, 20000.0])
        self_g = np.array([100.0, 200.0, 50.0])
        gains = np.array([80.0, -50.0, 10.0])
        a = impact.collision(impact.BASE, omega, self_g)
        self.assertLess(a["energy_balance_error"], 1e-8)
        self.assertLess(a["momentum_relative_error"], 1e-7)
        self.assertLessEqual(abs(a["velocity_out"]), impact.BASE["velocity"])
        wave = impact.render_force(omega, gains, a)
        state = a["solution"].y[:, -1]
        r = state[2:5] * a["force_scale"] * a["time_scale"] ** 2
        rd = state[5:8] * a["force_scale"] * a["time_scale"]
        t = np.arange(len(wave)) / impact.pilot.RATE - a["duration"]
        keep = t >= 0
        t = t[keep]
        d = 2 + 1e-8 * omega**2
        wd = np.sqrt(omega**2 - d**2)
        expected = np.sum(
            gains[:, None]
            * np.exp(-d[:, None] * t)
            * (
                rd[:, None] * np.cos(wd[:, None] * t)
                - ((omega**2 * r + d * rd) / wd)[:, None] * np.sin(wd[:, None] * t)
            ),
            axis=0,
        )
        self.assertLess(
            np.linalg.norm(wave[keep] - expected) / np.linalg.norm(expected), 1e-5
        )

    def test_quadrature_and_zero_force(self):
        omega = np.array([700.0, 1700.0, 18000.0]) * 2 * np.pi
        gains = np.array([2.0, 1.0, -0.5])
        a = impact.collision(impact.BASE)
        x = impact.render_force(omega, gains, a, quadrature=64)
        y = impact.render_force(omega, gains, a, quadrature=128)
        self.assertLess(np.linalg.norm(x - y) / np.linalg.norm(y), 1e-5)
        zero = impact.render_force(
            omega, gains, {"duration": 0.001, "force": lambda t: np.zeros_like(t)}
        )
        self.assertEqual(np.max(abs(zero)), 0)

    def test_fem_collocated_residue_positivity_and_reciprocal_rank_one(self):
        p = impact.pilot.solve(
            impact.SHAPE, impact.pilot.DEV_CONTACTS, contact_response=True
        )
        self.assertTrue(np.all(p["self_gains"] >= 0))
        np.testing.assert_allclose(
            p["gains"] ** 2,
            p["self_gains"] * p["probe_self_gains"],
            rtol=1e-12,
            atol=1e-20,
        )

    def test_invalid_inputs_and_weak_contact_rule(self):
        for key, bad in (
            ("velocity", -1),
            ("radius", 0),
            ("young", float("nan")),
            ("poisson", 0.5),
        ):
            with self.assertRaises(ValueError):
                impact.parameters(**{**impact.BASE, key: bad})
        a = {"duration": 1.0, "peak_force": 1.0, "impulse": 1.0}
        self.assertTrue(impact.weak_contact_passes(a, a, 0.01))
        self.assertFalse(impact.weak_contact_passes(a, a, 0.06))
        self.assertFalse(impact.weak_contact_passes(a, {**a, "impulse": 0.8}, 0.01))


if __name__ == "__main__":
    unittest.main()
