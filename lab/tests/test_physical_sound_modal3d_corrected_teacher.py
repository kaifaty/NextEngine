import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_modal3d_corrected_teacher as corrected
import physical_sound_modal3d_teacher_effect as effect


class CorrectedTeacherTests(unittest.TestCase):
    def test_fit_projection_excludes_dev_and_numerical_diagnostics(self):
        rows = [
            {
                "role": "train",
                "shape": shape,
                "file": f"train-{i}.npz",
                "sha256": "test-hash",
                "pair": {"secret_diagnostic": 3},
            }
            for i, shape in enumerate(corrected.pilot.TRAIN_SHAPES)
        ]
        data = {
            "rows": rows
            + [
                {
                    "role": "development",
                    "file": "unopened.npz",
                    "pair": {"response": "not for fit"},
                }
            ]
        }
        projection = effect.train_projection(data, Path("/tmp/synthetic-test"))
        self.assertEqual(len(projection["rows"]), 36)
        for row in projection["rows"]:
            self.assertEqual(set(row), {"role", "file", "sha256"})
            self.assertEqual(row["role"], "train")
            self.assertTrue(Path(row["file"]).is_absolute())
        with self.assertRaises(ValueError):
            effect.train_projection({"rows": rows[:-1]}, Path("/tmp/synthetic-test"))
        with self.assertRaises(ValueError):
            effect.train_projection(
                {"rows": [rows[0]] * 36}, Path("/tmp/synthetic-test")
            )

    def test_fixed_train_dev_meshes_and_failed_checks_do_not_select_examples(self):
        for role, expected_level in (("train", 3), ("development", 4)):
            with self.subTest(role=role), tempfile.TemporaryDirectory() as temp:
                output = Path(temp)

                def solver(shape, contacts, refinement, sample_points):
                    self.assertEqual(sample_points.shape, (90, 3))
                    return {
                        "omega": np.arange(1, 9, dtype=float) * refinement,
                        "gains": np.ones((len(contacts), 8)),
                        "mode_samples": np.eye(8),
                        "max_residual": 0.0,
                        "dofs": np.int32(975),
                    }

                with patch.object(
                    corrected.pilot, "solve", side_effect=solver
                ) as solve:
                    row = corrected.one_body(
                        (role, 0, (0.14, 0.055, 0.24), np.array([[0.55, 0.35]]), output)
                    )
                self.assertEqual(solve.call_count, 2)
                self.assertFalse(row["pair"]["locally_stable"])
                self.assertEqual(row["refinement"], expected_level)
                self.assertEqual(row["role"], role)
                with np.load(output / row["file"], allow_pickle=False) as saved:
                    np.testing.assert_array_equal(
                        saved["omega"], np.arange(1, 9) * expected_level
                    )
                self.assertEqual(len(list(output.glob("*.wav"))), 1)
                self.assertTrue((output / f"{role}-00-mesh3.npz").exists())
                self.assertTrue((output / f"{role}-00-mesh4.npz").exists())


if __name__ == "__main__":
    unittest.main()
