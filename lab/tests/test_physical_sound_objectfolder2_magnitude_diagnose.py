"""Oracle query equivalence and intervention isolation."""

import math
import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_magnitude_diagnose as probe


class MagnitudeDiagnosticTests(unittest.TestCase):
    def test_same_count_query_equals_standalone(self):
        model = probe.magnitude.MagnitudeStudent().eval().requires_grad_(False)
        model.core.count.weight.zero_()
        model.core.count.bias.zero_()
        model.core.count_mean.fill_(math.log(4))
        geometry = (
            np.zeros((512, 3), np.float32),
            np.array([0, 0, 0, 1, 0, 0, 0, 0], np.float32),
            np.zeros((2, 3), np.float32),
        )
        raw = probe.magnitude.predict(model, *geometry)
        oracle = probe.oracle_count_query(model, *geometry, 4)
        for key in oracle:
            np.testing.assert_array_equal(raw[key], oracle[key])
        with self.assertRaisesRegex(ValueError, "count"):
            probe.oracle_count_query(model, *geometry, 0)

    def test_pole_and_field_changes_are_separate(self):
        target = {
            "frequency": np.array([100.0, 200.0]),
            "damping": np.array([1.0, 2.0]),
            "gains": np.ones((1, 3, 2)),
        }
        predicted = {k: v * 2 for k, v in target.items()}
        controls = probe.compose(target, predicted)
        for key in ("frequency", "damping"):
            np.testing.assert_array_equal(controls["oracle_poles"][key], target[key])
            np.testing.assert_array_equal(controls["oracle_field"][key], predicted[key])
        np.testing.assert_array_equal(
            controls["oracle_poles"]["gains"], predicted["gains"]
        )
        np.testing.assert_array_equal(
            controls["oracle_field"]["gains"], target["gains"]
        )
        with self.assertRaisesRegex(ValueError, "aligned"):
            probe.compose(target, predicted | {"gains": np.ones((1, 3, 3))})


if __name__ == "__main__":
    unittest.main()
