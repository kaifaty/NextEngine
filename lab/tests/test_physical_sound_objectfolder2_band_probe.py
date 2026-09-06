"""Energy packing math and counterexamples, not perceptual validation."""

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_band_probe as probe
import physical_sound_objectfolder2_compact as compact


class BandProbeTests(unittest.TestCase):
    def test_compact_common_poles_and_contact_fields(self):
        f, d = np.linspace(1000, 1005, 8), np.linspace(5, 7, 8)
        g = np.random.default_rng(42).normal(size=(32, 3, 8))
        packed, rows = compact.pack(f, d, g)
        self.assertEqual(packed["frequency"].shape, (1,))
        self.assertEqual(packed["gains"].shape, (32, 3, 1))
        np.testing.assert_allclose(packed["gains"][:, :, 0], g.sum(-1), rtol=1e-6)
        self.assertEqual(sum(r["source_modes"] for r in rows), len(f))
        permuted, _ = compact.pack(f, d, g[::-1])
        np.testing.assert_allclose(
            packed["frequency"], permuted["frequency"], rtol=1e-14
        )
        with self.assertRaisesRegex(ValueError, "finite"):
            compact.pack(f * np.nan, d, g)

    def test_high_frequency_qr_keeps_energy_without_gram_inverse(self):
        f, d = np.linspace(21500, 21800, 8), np.full(8, 19000.0)
        g = np.random.default_rng(42).normal(size=(3, 8))
        _packed, rows = probe.compress(f, d, g)
        self.assertLess(max(r["gram_relative_error"] for r in rows), 1e-9)
        q = probe.orthogonal_carriers([21507.6, 21626.4, 21745.3], [19000.0] * 3)
        np.testing.assert_allclose(
            q @ q.T / probe.teacher.RATE, np.eye(3), rtol=1e-12, atol=1e-14
        )

    def test_discrete_gram_matches_direct_sum(self):
        f, d = np.array([111.0, 114.0, 17300.0]), np.array([3.0, 13.0, 200.0])
        t = np.arange(probe.SAMPLES) / probe.teacher.RATE
        b = np.exp(-d[:, None] * t) * np.sin(2 * np.pi * f[:, None] * t)
        np.testing.assert_allclose(
            probe.basis_gram(f, d), b @ b.T / probe.teacher.RATE, rtol=1e-11, atol=1e-14
        )

    def test_energy_packing_preserves_force_covariance(self):
        f = np.linspace(1000, 1005, 8)
        d = np.linspace(5, 7, 8)
        g = np.random.default_rng(42).normal(size=(3, 8))
        packed, bands = probe.compress(f, d, g)
        self.assertEqual(len(bands), 1)
        self.assertEqual(len(packed["energy_gram"]["frequency"]), 3)
        v = packed["energy_gram"]
        original = g @ probe.basis_gram(f, d) @ g.T
        response = probe.axis_responses(v)
        reproduced = response @ response.T / probe.teacher.RATE
        np.testing.assert_allclose(reproduced, original, rtol=1e-9, atol=1e-12)
        for force in ([1, 0, 0], [0, 1, -1], [1, 1, 1], [-2, 3, 1]):
            force = np.array(force)
            self.assertAlmostEqual(
                float(force @ original @ force / (force @ reproduced @ force)),
                1,
                places=9,
            )

    def test_sparse_poles_retained_and_sign_linearity(self):
        f, d, g = (
            np.array([1000.0, 1001.0]),
            np.array([5.0, 7.0]),
            np.array([[1.0, -1.0], [0.0, 2.0], [-3.0, 1.0]]),
        )
        packed, _ = probe.compress(f, d, g)
        p = packed["energy_gram"]
        for k, original in (("frequency", f), ("damping", d), ("gains", g)):
            np.testing.assert_array_equal(p[k], original)
        wave = probe.teacher.waveform(g, f, d, [1, 1, 1])
        np.testing.assert_array_equal(
            -wave, probe.teacher.waveform(g, f, d, [-1, -1, -1])
        )
        self.assertFalse(np.any(probe.teacher.waveform(g, f, d, [0, 0, 0])))

    def test_negative_grams_and_singular_carriers_rejected(self):
        with self.assertRaisesRegex(ValueError, "indefinite"):
            probe.psd_root(np.diag([1.0, 1.0, -0.1]))
        with self.assertRaisesRegex(ValueError, "ill-conditioned"):
            probe.psd_root(np.diag([1.0, 1.0, 0.0]), inverse=True)
        with self.assertRaisesRegex(ValueError, "poles"):
            probe.basis_gram([1000.0], [-1.0])


if __name__ == "__main__":
    unittest.main()
