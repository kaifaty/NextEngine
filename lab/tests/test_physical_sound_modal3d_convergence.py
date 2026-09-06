import copy
import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_modal3d_convergence as convergence
import physical_sound_modal3d_pilot as pilot


class ModalConvergenceTests(unittest.TestCase):
    def test_assurance_ignores_sign_and_scale_but_matches_permutation(self):
        a = np.eye(12)
        order = np.array([2, 0, 1] + list(range(3, 12)))
        b = a[:, order] * np.linspace(-3, -1, 12)
        coarse = {
            "mode_samples": a,
            "omega": np.arange(1, 13, dtype=float),
            "gains": np.arange(1, 49, dtype=float).reshape(4, 12),
        }
        fine = {
            "mode_samples": b,
            "omega": coarse["omega"][order],
            "gains": coarse["gains"][:, order],
        }
        pair = convergence.compare(coarse, fine)
        np.testing.assert_array_equal(pair["matched_columns"], np.argsort(order))
        np.testing.assert_allclose(pair["matched_assurance"], 1)
        np.testing.assert_allclose(pair["matched_frequency_relative_change"], 0)
        np.testing.assert_allclose(pair["first8_participation_relative_l1"], 0)
        self.assertGreater(
            max(abs(np.array(pair["sorted_frequency_relative_change"]))), 0
        )

    def test_invalid_modal_vectors_rejected(self):
        for invalid in (np.zeros((12, 12)), np.full((12, 12), np.nan), np.eye(11)):
            with self.subTest(shape=invalid.shape), self.assertRaises(ValueError):
                convergence.assurance(np.eye(12), invalid)

    def test_local_stability_requires_all_four_diagnostics(self):
        pair = {
            "matched_frequency_relative_change": [0.001] * 12,
            "matched_assurance": [0.999] * 12,
            "first8_participation_relative_l1": [0.01] * 4,
        }
        self.assertTrue(convergence.locally_stable(pair, [0.01] * 4))
        for key, bad in (
            ("matched_frequency_relative_change", 0.02),
            ("matched_assurance", 0.90),
            ("first8_participation_relative_l1", 0.06),
        ):
            changed = copy.deepcopy(pair)
            changed[key][0] = bad
            self.assertFalse(convergence.locally_stable(changed, [0.01] * 4))
        self.assertFalse(convergence.locally_stable(pair, [0.01, 0.01, 0.01, 0.06]))
        self.assertFalse(convergence.locally_stable(pair, [0.01, np.nan, 0.01, 0.01]))
        changed = copy.deepcopy(pair)
        changed["matched_assurance"][3] = np.nan
        self.assertFalse(convergence.locally_stable(changed, [0.01] * 4))

    def test_twelve_mode_solver_preserves_old_eight_mode_response(self):
        points = np.array([[0.3, 0.4, 0.5], [1, 0.5, 1]])
        old = pilot.solve(pilot.DEV_SHAPES[0], pilot.DEV_CONTACTS)
        new = pilot.solve(
            pilot.DEV_SHAPES[0],
            pilot.DEV_CONTACTS,
            mode_count=12,
            sample_points=points,
        )
        self.assertEqual(new["mode_samples"].shape, (6, 12))
        self.assertTrue(np.isfinite(new["mode_samples"]).all())
        np.testing.assert_allclose(old["omega"], new["omega"][:8], rtol=1e-8)
        np.testing.assert_allclose(
            old["gains"], new["gains"][:, :8], rtol=1e-5, atol=1e-7
        )


if __name__ == "__main__":
    unittest.main()
