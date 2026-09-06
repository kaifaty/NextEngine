"""Size transport equations, no-op and negative controls."""

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_size_probe as probe


class SizeTests(unittest.TestCase):
    def parameters(self):
        f = np.array([1000.0, 2000.0, 5000.0])
        return {
            "frequency": f,
            "damping": probe.rayleigh.damping(f, "Wood"),
            "gains": np.ones((1, 3, 3)),
        }

    def test_inverse_square_and_rayleigh_law(self):
        source = self.parameters()
        for scale in (0.8, 1.25):
            out = probe.transport(source, scale, "Wood")
            np.testing.assert_allclose(
                probe.eigenvalues(out), probe.eigenvalues(source) / scale**2, rtol=1e-14
            )
            np.testing.assert_allclose(
                out["damping"],
                probe.rayleigh.damping(out["frequency"], "Wood"),
                rtol=1e-14,
            )
            np.testing.assert_array_equal(out["gains"], source["gains"])

    def test_identity_and_composition(self):
        source = self.parameters()
        identity = probe.transport(source, 1, "Wood")
        for key in source:
            np.testing.assert_array_equal(identity[key], source[key])
        a = probe.transport(probe.transport(source, 0.8, "Wood"), 1.25, "Wood")
        np.testing.assert_allclose(a["frequency"], source["frequency"], rtol=1e-14)

    def test_ignore_size_is_negative_control(self):
        source = self.parameters()
        expected = probe.transport(source, 1.25, "Wood")
        correct = probe.comparison(expected, expected)
        wrong = probe.comparison(expected, source)
        self.assertEqual(correct["log_frequency_w1"], 0)
        self.assertAlmostEqual(wrong["log_frequency_w1"], np.log(1.25), places=12)
        self.assertAlmostEqual(
            wrong["lowest_natural_frequency_relative_error"], 0.25, places=12
        )

    def test_invalid_scale_and_alias_rejected(self):
        with self.assertRaisesRegex(ValueError, "scale"):
            probe.transport(self.parameters(), 0, "Wood")
        with self.assertRaisesRegex(ValueError, "alias"):
            probe.transport(self.parameters(), 0.2, "Wood")


if __name__ == "__main__":
    unittest.main()
