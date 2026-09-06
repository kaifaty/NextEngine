"""Geometry failures and physical port controls, not perceptual validation."""

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_elastic as meshing
import physical_sound_objectfolder2_elastic_assess as assess
import physical_sound_objectfolder2_elastic_solve as elastic


class ElasticTests(unittest.TestCase):
    def setUp(self):
        self.v = np.array(
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        )
        self.f, _, _ = elastic.boundary(self.v, np.array([[0, 1, 2, 3]]))

    def test_oriented_shell_and_cavity(self):
        r = meshing.validate_surface(self.v, self.f)
        self.assertAlmostEqual(r["signed_volume"], 1 / 6)
        self.assertEqual(r["hole_seeds"], [])
        inner = 0.2 * (self.v - self.v.mean(0)) + self.v.mean(0)
        f = np.vstack([self.f, self.f[:, [0, 2, 1]] + 4])
        r = meshing.validate_surface(np.vstack([self.v, inner]), f)
        self.assertEqual(len(r["hole_seeds"]), 1)
        self.assertAlmostEqual(r["signed_volume"], (1 - 0.2**3) / 6)
        self.assertAlmostEqual(meshing.winding(self.v[self.f], self.v.mean(0)), 1)
        self.assertAlmostEqual(meshing.winding(self.v[self.f], [2, 2, 2]), 0)

    def test_bad_surface_is_not_repaired(self):
        for faces in (self.f[:-1], self.f[:, [0, 0, 2]], self.f + 5):
            with self.assertRaises(ValueError):
                meshing.validate_surface(self.v, faces)
        with self.assertRaisesRegex(ValueError, "disconnected"):
            meshing.validate_surface(
                np.vstack([self.v, self.v + 2]), np.vstack([self.f, self.f + 4])
            )
        with self.assertRaises(ValueError):
            meshing.validate_surface(np.vstack([self.v, self.v[0]]), self.f)

    def test_physical_ports_sign_impulse_and_self_residues(self):
        omega = 2 * np.pi * np.array([400.0, 700.0])
        ports = np.array([[1.0, -0.5], [-2.0, 3.0], [0.2, -0.1], [-0.3, 1.0]])
        raw, _, audible, residue = elastic.port_response(omega, ports)
        self.assertTrue(audible.all())
        np.testing.assert_array_equal(residue, ports[:-1] * ports[-1])
        np.testing.assert_array_equal(raw, elastic.port_response(omega, -ports)[0])
        np.testing.assert_array_equal(
            raw, elastic.port_response(omega, ports * [-1, 1])[0]
        )
        np.testing.assert_array_equal(
            raw * 2, elastic.port_response(omega, ports, 0.002)[0]
        )
        self.assertFalse(elastic.port_response(omega, ports, 0)[0].any())

    def test_distance_checks_triangle_interior_edges_and_vertices(self):
        tri = np.array([[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]])
        p = np.array(
            [[0.2, 0.2, 1.0], [0.5, -0.2, 0.0], [-1.0, -1.0, 0.0], [0.5, 0.5, 0.0]]
        )
        np.testing.assert_allclose(
            assess.surface_distances(p, tri), [1.0, 0.2, np.sqrt(2), 0.0]
        )
        with self.assertRaises(ValueError):
            assess.surface_distances(p, np.zeros((1, 3, 3)))


if __name__ == "__main__":
    unittest.main()
