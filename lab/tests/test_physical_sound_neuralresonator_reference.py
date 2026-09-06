import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_neuralresonator_reference as reference


class ReferenceTests(unittest.TestCase):
    def test_mask_centroid_and_background(self):
        for vertices in reference.POLYGONS.values():
            polygon = np.asarray(vertices)
            mask = reference.mask_for(polygon)
            self.assertEqual(mask.shape, (64, 64))
            self.assertEqual(float(mask[0, 0]), 0)
            x, y = np.round(polygon.mean(0) * 64).astype(int)
            self.assertEqual(float(mask[y, x]), 1)

    def test_elastic_scale_and_density_positive_controls(self):
        polygon = np.asarray(reference.POLYGONS["rectangle"])
        material = reference.pilot.BASE
        a = reference.solve_modes(polygon, material, 2, 1.0)
        b = reference.solve_modes(polygon, material, 2, 2.0)
        dense = list(material)
        dense[0] *= 4
        c = reference.solve_modes(polygon, dense, 2, 1.0)
        np.testing.assert_allclose(a["eigenvalues"] / b["eigenvalues"], 4, rtol=1e-8)
        np.testing.assert_allclose(a["eigenvalues"] / c["eigenvalues"], 4, rtol=1e-8)
        self.assertLess(a["max_relative_residual"], 1e-8)
        self.assertIsInstance(a["dofs"], int)

    def test_reference_requires_same_contact(self):
        modes = {
            "points": np.array([[0.5, 0.5]]),
            "gains": np.array([[0.1]]),
            "frequencies": np.array([800]),
            "damping": np.array([20]),
        }
        wave = reference.render_reference(modes, np.array([0.5, 0.5]))
        self.assertEqual(len(wave), reference.pilot.RATE)
        self.assertTrue(np.isfinite(wave).all())
        with self.assertRaisesRegex(ValueError, "exact"):
            reference.render_reference(modes, np.array([0.51, 0.5]))

    def test_shape_metric_ignores_global_gain_not_pitch(self):
        time = np.arange(reference.pilot.RATE) / reference.pilot.RATE
        a = np.cos(2 * np.pi * 800 * time) * np.exp(-20 * time)
        b = np.cos(2 * np.pi * 1600 * time) * np.exp(-20 * time)
        self.assertLess(reference.spectrum_distance(a, a * 3), 1e-10)
        self.assertGreater(reference.spectrum_distance(a, b), 1)


if __name__ == "__main__":
    unittest.main()
