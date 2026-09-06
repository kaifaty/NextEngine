"""Oracle distinctions and basis-invariant positive control, not quality tests."""

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_field_cause as probe


class FieldCauseTests(unittest.TestCase):
    def test_scalar_error_is_repaired_only_by_magnitude_or_scalar_oracle(self):
        target = np.array([[[1.0, -2.0], [3.0, -4.0], [5.0, -6.0]]])
        predicted = target / 10
        values, scalar = probe.variants(target, predicted)
        self.assertAlmostEqual(scalar, 10)
        np.testing.assert_array_equal(values["oracle_sign"], predicted)
        np.testing.assert_array_equal(values["oracle_magnitude"], target)
        np.testing.assert_allclose(values["oracle_scalar_rms"], target, rtol=1e-15)

    def test_mode_sign_error_does_not_become_magnitude_error(self):
        target = np.array([[[1.0, -2.0], [3.0, -4.0], [5.0, -6.0]]])
        predicted = target * np.array([1.0, -1.0])
        values, scalar = probe.variants(target, predicted)
        self.assertEqual(scalar, 1)
        np.testing.assert_array_equal(values["oracle_sign"], target)
        np.testing.assert_array_equal(values["oracle_magnitude"], predicted)
        np.testing.assert_array_equal(values["reference_mode_flip"], predicted)

    def test_zero_prediction_stays_zero(self):
        values, scalar = probe.variants(np.ones((2, 3, 4)), np.zeros((2, 3, 4)))
        self.assertIsNone(scalar)
        self.assertNotIn("oracle_scalar_rms", values)
        self.assertFalse(values["oracle_magnitude"].any())

    def test_consistent_basis_flip_preserves_residues(self):
        a = np.array([[1.0, 2.0, 3.0], [-2.0, 3.0, 4.0], [5.0, -1.0, 2.0]])
        signs = np.array([1.0, -1.0, 1.0])
        np.testing.assert_array_equal(
            probe.port_residues(a), probe.port_residues(a * signs)
        )
        self.assertFalse(np.array_equal(a[0] * a[-1], (a[0] * signs) * a[-1]))

    def test_invalid_shape_or_values_reject(self):
        with self.assertRaisesRegex(ValueError, "matching"):
            probe.variants(np.ones((2, 3, 4)), np.ones((2, 3, 5)))
        with self.assertRaisesRegex(ValueError, "finite"):
            probe.variants(np.full((2, 3, 4), np.nan), np.ones((2, 3, 4)))


if __name__ == "__main__":
    unittest.main()
