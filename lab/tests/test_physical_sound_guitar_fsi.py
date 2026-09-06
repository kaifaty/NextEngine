import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from scipy import linalg, sparse

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_guitar_fsi as fsi


def oscillator():
    # Two bodies connected through a skew energy-exchange term.
    j = np.array([[0, 0, 1, 0], [0, 0, 0, 1], [-1, 0, 0, 0.2], [0, -1, -0.2, 0]])
    d = np.diag([0, 0, 1, 2])
    q = np.diag([100, 400, 1, 1])
    return {
        k: sparse.csc_matrix(v)
        for k, v in {
            "E": np.eye(4),
            "J": j,
            "D": d,
            "Q": q,
            "A": (j - d) @ q,
            "B": np.array([[0], [0], [1], [0]]),
            "C": np.eye(4),
        }.items()
    }


class GuitarTests(unittest.TestCase):
    def test_projected_system_retains_transfer_and_power_identity(self):
        d = oscillator()
        r = fsi.reduce_system(d, [1, 2, 3])
        self.assertEqual(len(r["a"]), 4)
        for frequency in (1.3, 2.7, 4.2):
            _, reference = fsi.transfer(d, frequency)
            actual = r["c"] @ linalg.solve(
                2j * np.pi * frequency * np.eye(4) - r["a"], r["b"]
            )
            np.testing.assert_allclose(actual[:, 0], reference, rtol=1e-9, atol=1e-12)
        np.testing.assert_allclose(r["port"], r["b"].T, rtol=1e-12, atol=1e-12)
        x = np.array([0.2, -0.1, 0.6, 0.3])
        force = 0.7
        energy_rate = x @ (r["a"] @ x + r["b"][:, 0] * force)
        supplied = (r["port"] @ x).item() * force
        self.assertLess(energy_rate, supplied)

    def test_midpoint_matches_independent_constant_force_exponential(self):
        d = oscillator()
        block = np.zeros((5, 5))
        block[:4, :4], block[:4, 4] = d["A"].toarray(), d["B"].toarray()[:, 0]
        exact = (linalg.expm(block * 0.1) @ [0, 0, 0, 0, 1])[:4]
        coarse = fsi.midpoint(d, np.ones(101), 0.001)[-1]
        fine = fsi.midpoint(d, np.ones(201), 0.0005)[-1]
        self.assertLess(
            np.linalg.norm(fine - exact), np.linalg.norm(coarse - exact) / 3.9
        )

    def test_zero_input_and_no_coupling_control(self):
        d = oscillator()
        self.assertTrue(
            np.array_equal(fsi.midpoint(d, np.zeros(101), 0.001), np.zeros((101, 4)))
        )
        j = d["J"].toarray()
        j[2, 3] = j[3, 2] = 0
        d["A"] = (sparse.csc_matrix(j) - d["D"]) @ d["Q"]
        y = fsi.midpoint(d, np.ones(101), 0.001)
        self.assertGreater(np.max(abs(y[:, 0])), 0)
        self.assertTrue(np.array_equal(y[:, [1, 3]], np.zeros((101, 2))))

    def test_pulse_integral_and_bad_width(self):
        t = np.arange(2001) / 100000
        for width in (0.002, 0.005, 0.010):
            self.assertAlmostEqual(np.trapezoid(fsi.impulse(t, width), t), width / 2)
        for width in (0, -1, np.nan, np.inf):
            with self.assertRaises(ValueError):
                fsi.impulse(t, width)

    def test_bad_source_rejected_before_parsing(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "invalid.mat"
            path.write_bytes(b"not the published data")
            with self.assertRaisesRegex(ValueError, "exact published"):
                fsi.load_source(path)


if __name__ == "__main__":
    unittest.main()
