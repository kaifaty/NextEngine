"""Development selection must reject attack regressions and preserve splits."""

import sys
import time
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_audible_glass as glass
import physical_sound_audible_glass_cycle as cycle
from test_physical_sound_audible_glass import tone_parameters


class GlassCycleTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        torch.set_num_threads(2)

    def test_recording_split_and_missing_partition(self):
        rows = [
            {"sound_id": sound_id}
            for sound_id in ("761162", "761160", "761161", "761162")
        ]
        train, development = cycle.split_rows(rows)
        self.assertEqual(train, [1, 2])
        self.assertEqual(development, [0, 3])
        for incomplete in ([], [{"sound_id": "761160"}], [{"sound_id": "761162"}]):
            with self.assertRaises(ValueError):
                cycle.split_rows(incomplete)

    def test_validator_rejects_attack_regression_and_nonfinite(self):
        baseline = {"spectral": 1.0, "envelope": 1.0, "attack": 1.0}
        improved = {"spectral": 0.8, "envelope": 0.9, "attack": 0.9}
        self.assertTrue(cycle.eligible(improved, baseline))
        for broken in (
            {**improved, "attack": 1.2},
            {**improved, "envelope": 1.2},
            {**improved, "spectral": 0.95},
            {**improved, "attack": np.nan},
        ):
            self.assertFalse(cycle.eligible(broken, baseline))

    def test_reference_metric_prefers_correct_signal(self):
        target = glass.Synthesizer()(tone_parameters(), [0]).detach()
        correct, _ = cycle.metrics(target, target, [0])
        wrong, _ = cycle.metrics(target, target * 0.5, [0])
        self.assertTrue(cycle.eligible(correct, wrong))
        self.assertAlmostEqual(correct["spectral"], 0)
        self.assertGreater(wrong["attack"], 0.4)

    def test_teacher_timeout_preserves_initial_parameters(self):
        initial = tone_parameters()
        synth = glass.Synthesizer()
        target = synth(initial, [0])[0].detach()
        result, report = cycle.fit_teacher(
            initial, target, 0, synth, time.monotonic() - 1
        )
        self.assertTrue(torch.equal(initial, result))
        self.assertEqual(report["status"], "time_limit")


if __name__ == "__main__":
    unittest.main()
