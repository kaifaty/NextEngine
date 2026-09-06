import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_material_controls as controls


class MaterialControlsTests(unittest.TestCase):
    def setUp(self):
        self.rows = [
            {"material": m, "recording": f"{m}-{i}", "role": "train"}
            for m in controls.MATERIALS
            for i in range(2)
        ] + [{"material": "glass", "recording": "held", "role": "recording-dev"}]
        self.features = np.vstack([np.repeat(np.eye(3), 2, axis=0), [[1.0, 0.0, 0.0]]])

    def test_separated_positive_controls_and_record_exclusion(self):
        for pooling in ("material", "recording"):
            p, names = controls.bank(
                self.features, self.rows, pooling, excluded="glass-0"
            )
            for representation in ("shape", "embedding"):
                result = controls.classify(np.eye(3), p, names, representation)
                self.assertEqual(result, list(controls.MATERIALS))
            self.assertEqual(len(p), 3 if pooling == "material" else 5)

    def test_held_targets_features_cannot_change_bank(self):
        for pooling in ("material", "recording"):
            before, names = controls.bank(self.features, self.rows, pooling)
            changed = self.features.copy()
            changed[-1] *= -100
            rows = [dict(r) for r in self.rows]
            rows[-1]["material"] = "metal"
            after, labels = controls.bank(changed, rows, pooling)
            np.testing.assert_array_equal(before, after)
            self.assertEqual(names, labels)

    def test_recording_balancing_not_event_count_weighting(self):
        rows = [{"material": "glass", "recording": "a", "role": "recording-dev"}] * 10
        rows += [{"material": "glass", "recording": "b", "role": "recording-dev"}]
        result = controls.metrics(rows, ["glass"] * 10 + ["metal"])
        self.assertEqual(result["recording_balanced_recall"]["glass"], 0.5)
        self.assertIsNone(result["macro_recall"])
        self.assertEqual(result["material_recording_support"]["glass"], 2)

    def test_training_bank_weights_each_record_once(self):
        self.features[0, 0] = 3
        rows = self.rows + [self.rows[0]] * 10
        features = np.vstack([self.features, [self.features[0]] * 10])
        before, _ = controls.bank(self.features, self.rows, "material")
        after, _ = controls.bank(features, rows, "material")
        np.testing.assert_array_equal(before, after)

    def test_leakage_missing_class_and_invalid_data_rejected(self):
        rows = [dict(r) for r in self.rows]
        rows[-1]["recording"] = rows[0]["recording"]
        with self.assertRaisesRegex(ValueError, "leakage"):
            controls.bank(self.features, rows, "material")
        with self.assertRaisesRegex(ValueError, "Missing"):
            controls.bank(self.features[2:], self.rows[2:], "material")
        features = self.features.copy()
        features[0, 0] = np.nan
        with self.assertRaisesRegex(ValueError, "Invalid"):
            controls.bank(features, self.rows, "material")


if __name__ == "__main__":
    unittest.main()
