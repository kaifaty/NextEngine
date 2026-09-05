import sys
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import pandas as pd

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_epic_information as info


class InformationTests(unittest.TestCase):
    def test_selection_caps_exclusions_and_all_annotation_overlap(self):
        records = []
        for part in range(1, 17):
            for label_id, label in enumerate(info.source.CLASSES):
                for i in range(4):
                    start = (label_id * 10 + i * 2) * 24000
                    records.append(
                        {
                            "annotation_id": f"P{part:02}_01_{label_id * 4 + i}",
                            "participant_id": f"P{part:02}",
                            "video_id": f"P{part:02}_01",
                            "class": label,
                            "start_sample": start,
                            "stop_sample": start + 12000,
                        }
                    )
        frame = pd.DataFrame(records)
        rows, counts = info.select(frame)
        self.assertEqual(len(rows), 216)
        self.assertEqual(set(counts.values()), {56})
        self.assertFalse(
            {"P04", "P07", "P15", "P16"} & {r["participant_id"] for r in rows}
        )
        # Nested non-material event must invalidate an otherwise eligible impact.
        background = {
            **records[0],
            "annotation_id": "P01_01_999",
            "class": "speech",
            "start_sample": 100,
            "stop_sample": 200,
        }
        rows, _ = info.select(pd.concat([frame, pd.DataFrame([background])]))
        self.assertNotIn(
            records[0]["annotation_id"], {r["annotation_id"] for r in rows}
        )
        self.assertIn(records[3]["annotation_id"], {r["annotation_id"] for r in rows})
        long_event = {**background, "stop_sample": 110000}
        nested = {
            **background,
            "annotation_id": "P01_01_998",
            "start_sample": 200,
            "stop_sample": 300,
        }
        rows, _ = info.select(pd.concat([frame, pd.DataFrame([long_event, nested])]))
        selected = {r["annotation_id"] for r in rows}
        self.assertNotIn(records[1]["annotation_id"], selected)
        self.assertNotIn(records[2]["annotation_id"], selected)
        self.assertIn(records[3]["annotation_id"], selected)
        with self.assertRaisesRegex(ValueError, "identity"):
            info.select(pd.concat([frame, frame.iloc[:1]]))

    def test_grouped_probe_positive_control_and_training_only_scaling(self):
        labels = np.tile(np.repeat(np.arange(6), 3), 4)
        groups = np.repeat(np.arange(4), 18)
        features = np.column_stack([np.eye(6)[labels] * 10, groups])
        observed = []
        original = info.StandardScaler.fit

        def fit(scale, values, *args, **kwargs):
            observed.append(set(values[:, -1].tolist()))
            return original(scale, values, *args, **kwargs)

        with patch.object(info.StandardScaler, "fit", fit):
            predicted, folds = info.cross_predict(features, labels, groups)
        self.assertEqual(info.summary(labels, predicted)["macro_recall"], 1)
        self.assertEqual(len(folds), 4)
        for group, training in enumerate(observed):
            self.assertEqual(training, set(range(4)) - {group})
        with self.assertRaisesRegex(ValueError, "finite"):
            info.cross_predict(features * np.nan, labels, groups)
        with self.assertRaisesRegex(ValueError, "six classes"):
            info.cross_predict(features, labels, labels)

    def test_aggregate_recall_exposes_rare_class_failures(self):
        labels = np.r_[np.zeros(100, int), np.arange(1, 6)]
        result = info.summary(labels, np.zeros(len(labels), int))
        self.assertGreater(result["accuracy"], 0.95)
        self.assertAlmostEqual(result["macro_recall"], 1 / 6)
        self.assertEqual(result["recalls"], [1, 0, 0, 0, 0, 0])
        with self.assertRaisesRegex(ValueError, "six classes"):
            info.summary(np.array([0]), np.array([0]))

    def test_fixed_permutation_discriminator_rejects_wrong_labels(self):
        labels = np.tile(np.repeat(np.arange(6), 3), 4)
        groups = np.repeat(np.arange(4), 18)
        rows = [
            {"class": info.source.CLASSES[label], "participant_id": str(group)}
            for label, group in zip(labels, groups, strict=True)
        ]
        result = info.evaluate({"ast_raw": np.eye(6)[labels]}, rows)
        self.assertEqual(result["methods"]["ast_raw"]["macro_recall"], 1)
        null = result["within_participant_permutation"]
        self.assertEqual(len(null["macro_recalls"]), 32)
        self.assertLess(null["max"], 1)
        self.assertEqual(null["monte_carlo_p"], 1 / 33)


if __name__ == "__main__":
    unittest.main()
