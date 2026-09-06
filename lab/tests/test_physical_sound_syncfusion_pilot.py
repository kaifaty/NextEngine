import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile as sf
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_syncfusion_pilot as pilot


class SyncFusionTests(unittest.TestCase):
    def test_event_times_are_sample_impulses_not_audio(self):
        track = pilot.event_track([0.6, 1.5, 2.7, 4.0])
        self.assertEqual(tuple(track.shape), (1, 1, 262144))
        self.assertEqual(
            torch.nonzero(track[0, 0]).ravel().tolist(), [28800, 72000, 129600, 192000]
        )
        self.assertEqual(float(track.sum()), 4)
        self.assertEqual(float(pilot.event_track([]).sum()), 0)

    def test_event_validation(self):
        for times in (
            [np.nan],
            [-1],
            [pilot.LENGTH / pilot.RATE],
            [[1]],
            [1, 1],
            list(range(33)),
        ):
            with self.assertRaises(ValueError):
                pilot.event_track(times)

    def test_finalization_keeps_failed_control_and_prevents_overwrite(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            pilot.save(root / "result.json", {"status": "failed", "rows": []})
            for name, _, _ in pilot.CASES:
                wave = np.full(
                    pilot.LENGTH, 2.2 if name == "empty" else 0.01, dtype=np.float32
                )
                sf.write(root / f"{name}-raw.wav", wave, pilot.RATE, subtype="FLOAT")
            pilot.finalize(root)
            result = json.loads((root / "result.json").read_text())
            self.assertEqual(result["status"], "complete_with_rejections")
            self.assertEqual(result["rows"][-1]["status"], "rejected")
            self.assertFalse((root / "empty.wav").exists())
            self.assertEqual(result["comparison_order"], ["glass", "delayed", "wood"])
            with self.assertRaises(ValueError):
                pilot.finalize(root)

    def test_tensor_subset_rejects_missing_or_nonfinite(self):
        state = {
            "a.weight": torch.tensor([1.0]),
            "b.weight": torch.tensor([float("nan")]),
        }
        self.assertEqual(list(pilot.subset(state, "a.")), ["weight"])
        for prefix in ("b.", "unknown."):
            with self.assertRaises(ValueError):
                pilot.subset(state, prefix)


if __name__ == "__main__":
    unittest.main()
