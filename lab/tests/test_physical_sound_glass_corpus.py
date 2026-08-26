from __future__ import annotations

import copy
import json
import unittest
import numpy as np

from next_lab.physical_sound_glass_corpus import (
    ModalSolution,
    assemble_linear_elasticity,
    build_solver_profile,
    build_structured_vessel_mesh,
    default_recipe_path,
    load_recipe,
    solve_modes,
)


class PhysicalSoundGlassCorpusTests(unittest.TestCase):
    def setUp(self) -> None:
        self.recipe, self.recipe_bytes = load_recipe(default_recipe_path())

    def test_default_geometry_is_exact_and_bounded(self) -> None:
        mesh = build_structured_vessel_mesh(self.recipe)

        self.assertEqual(mesh.nodes.shape, (9775, 3))
        self.assertEqual(mesh.tetrahedra.shape, (30864, 4))
        self.assertEqual(mesh.fixed_nodes.tolist(), [0, 1, 4, 5])
        self.assertTrue(np.allclose(mesh.nodes.min(axis=0), [-0.042, 0.0, -0.036]))
        self.assertTrue(np.allclose(mesh.nodes.max(axis=0), [0.042, 0.12, 0.036]))

    def test_small_elastic_problem_produces_finite_modes(self) -> None:
        recipe = copy.deepcopy(self.recipe)
        recipe["geometry"].update(
            {
                "outer_size_m": [0.018, 0.018, 0.018],
                "wall_thickness_m": 0.003,
                "bottom_thickness_m": 0.003,
            }
        )
        recipe["solver"].update(
            {
                "eigensolve_mode_count": 8,
                "retained_mode_count": 2,
                "minimum_frequency_hz": 1.0,
                "maximum_frequency_hz": 1000000.0,
            }
        )
        mesh = build_structured_vessel_mesh(recipe)
        stiffness, mass = assemble_linear_elasticity(mesh, recipe["material"])

        solution = solve_modes(mesh, stiffness, mass, recipe["solver"])

        self.assertEqual(solution.undamped_frequency_hz.shape, (2,))
        self.assertTrue(np.all(np.isfinite(solution.undamped_frequency_hz)))
        self.assertTrue(np.all(solution.undamped_frequency_hz > 0.0))
        self.assertLessEqual(float(np.max(solution.relative_residuals)), 1.0e-5)

    def test_profile_predeclares_train_and_two_axis_holdouts(self) -> None:
        mesh = build_structured_vessel_mesh(self.recipe)
        mode_count = 2
        vectors = np.zeros((len(mesh.nodes) * 3, mode_count), dtype=np.float64)
        for node_index, point in enumerate(mesh.nodes):
            vectors[3 * node_index, 0] = 1.0 + point[1]
            vectors[3 * node_index, 1] = 0.5 + point[2] + 2.0 * point[1]
        solution = ModalSolution(
            undamped_frequency_hz=np.asarray([1600.0, 4200.0]),
            eigenvectors=vectors,
            relative_residuals=np.asarray([1.0e-10, 2.0e-10]),
        )
        artifact_hashes = {
            name: "a" * 64
            for name in [
                "nodes.npy",
                "tetrahedra.npy",
                "fixed-nodes.npy",
                "eigenvectors.npy",
            ]
        }

        profile = build_solver_profile(
            self.recipe,
            self.recipe_bytes,
            mesh,
            solution,
            artifact_hashes,
        )

        split_counts: dict[str, int] = {}
        for condition in profile["conditions"]:
            split_counts[condition["split"]] = split_counts.get(condition["split"], 0) + 1
        self.assertEqual(
            split_counts,
            {
                "train": 8,
                "force-holdout": 4,
                "position-holdout": 2,
                "joint-holdout": 1,
            },
        )
        encoded = json.dumps(profile, allow_nan=False, sort_keys=True)
        self.assertNotIn("NaN", encoded)
        self.assertNotIn("Infinity", encoded)


if __name__ == "__main__":
    unittest.main()
