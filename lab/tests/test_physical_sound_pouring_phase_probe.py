from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_phase_probe as probe


class PhaseProbeTests(unittest.TestCase):
    def test_shuffle_changes_only_condition_pairing(self):
        targets = torch.stack([torch.zeros(1, 4, 4), torch.ones(1, 4, 4)])
        conditions = torch.zeros(2, 11)
        conditions[1, 4] = 1
        torch.manual_seed(53)
        correct = probe.batch(targets, conditions, False)
        after_correct = torch.rand(10)
        torch.manual_seed(53)
        shuffled = probe.batch(targets, conditions, True)
        after_shuffled = torch.rand(10)
        for index in (0, 1, 3):
            torch.testing.assert_close(correct[index], shuffled[index], rtol=0, atol=0)
        torch.testing.assert_close(after_correct, after_shuffled, rtol=0, atol=0)
        self.assertEqual(correct[2][:, 4].tolist(), [0, 1, 0, 1, 0, 1])
        self.assertEqual(sorted(shuffled[2][:, 4].tolist()), [0, 0, 0, 1, 1, 1])
        self.assertFalse(torch.equal(correct[2], shuffled[2]))

    def test_multi_record_shuffle_keeps_geometry_and_target_noise(self):
        targets = torch.arange(8, dtype=torch.float32)[:, None, None, None].expand(
            8, 1, 4, 4
        )
        conditions = torch.zeros(8, 11)
        conditions[:, 0] = torch.arange(8) // 2
        conditions[:, 4] = torch.arange(8) % 2
        torch.manual_seed(53)
        a = probe.batch(targets, conditions, False)
        torch.manual_seed(53)
        b = probe.batch(targets, conditions, True)
        for index in (0, 1, 3):
            torch.testing.assert_close(a[index], b[index], rtol=0, atol=0)
        torch.testing.assert_close(a[2][:, 0], b[2][:, 0], rtol=0, atol=0)
        self.assertFalse(torch.equal(a[2][:, 4], b[2][:, 4]))

    def test_invalid_budget_and_existing_output_precede_io(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for steps in (0, 601):
                with self.assertRaises(ValueError):
                    probe.run(root, root, root / "missing", steps, "cpu")
            with self.assertRaises(ValueError):
                probe.run(root, root, root, 1, "cpu")


if __name__ == "__main__":
    unittest.main()
