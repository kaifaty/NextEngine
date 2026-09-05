from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_interpolation as interpolation


class InterpolationTests(unittest.TestCase):
    def test_projection_and_bounds(self):
        alpha, distance = interpolation.projection([0.4, 0.4], [0, 0], [1, 1])
        self.assertAlmostEqual(alpha, 0.4)
        self.assertAlmostEqual(distance, 0)
        self.assertEqual(interpolation.projection([2, 2], [0, 0], [1, 1])[0], 1)
        for value, low, high in (
            ([1], [0, 0], [1, 1]),
            ([np.nan], [0], [1]),
            ([1], [0], [0]),
        ):
            with self.assertRaises(ValueError):
                interpolation.projection(value, low, high)

    def test_oracle_recovers_shape_mixture_without_level(self):
        rng = np.random.default_rng(1)
        low, high = rng.normal(-90, 3, (2, 513))
        target = interpolation.blend(low, high, 0.3) + 12
        self.assertAlmostEqual(interpolation.oracle_alpha(target, low, high), 0.3)
        self.assertAlmostEqual(
            interpolation.shape_error(target, interpolation.blend(low, high, 0.3)), 0
        )
        for bad in (-0.1, 1.1, np.nan):
            with self.assertRaises(ValueError):
                interpolation.blend(low, high, bad)

    def test_only_train_endpoints_and_speed_interpolation(self):
        surface = interpolation.surface
        rows = surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        training = [r for r in rows if surface.role(r) == "train"]
        table = {
            t: {
                "texture_id": t,
                "category": "Wood" if t < 65 else "Metals" if t < 74 else "Glass",
                "static_friction": 0.6 if t in (2, 67, 77) else 0.4,
                "dynamic_friction": 0.5 if t in (2, 67, 77) else 0.3,
            }
            for t in surface.SURFACES
        }
        table[76].update(static_friction=0.5, dynamic_friction=0.4)
        spectra = np.stack(
            [
                np.full(
                    513,
                    -100
                    + r["commanded_speed_mm_s"] * 0.1
                    + (r["texture_id"] == 77) * 2,
                )
                for r in training
            ]
        )
        row = next(r for r in rows if r["id"] == "76_0_40_500_0")
        low, high, info = interpolation.endpoints(row, table, training, spectra)
        np.testing.assert_allclose(low, -96)
        np.testing.assert_allclose(high, -94)
        self.assertAlmostEqual(info["coefficient_alpha"], 0.5)
        contaminated = [row] + training[1:]
        with self.assertRaises(ValueError):
            interpolation.endpoints(row, table, contaminated, spectra)
        duplicate = [training[1]] + training[1:]
        with self.assertRaises(ValueError):
            interpolation.endpoints(row, table, duplicate, spectra)


if __name__ == "__main__":
    unittest.main()
