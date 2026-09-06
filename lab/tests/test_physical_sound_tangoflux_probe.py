import sys
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_tangoflux_probe as probe


class ProbeTest(unittest.TestCase):
    def test_guidance_one_is_conditional_not_unconditional(self):
        conditional = torch.tensor([1.0, 2.0])
        unconditional = torch.tensor([1e20, -1e20])
        self.assertIs(
            probe.guided_prediction(conditional, unconditional, 1), conditional
        )

    def test_guidance_extrapolation_and_invalid_scale(self):
        actual = probe.guided_prediction(torch.tensor([3.0]), torch.tensor([1.0]), 2)
        self.assertEqual(float(actual), 5)
        for scale in (0, 0.99, 5.1, float("nan"), float("inf")):
            with self.assertRaises(ValueError):
                probe.guided_prediction(torch.zeros(1), torch.zeros(1), scale)

    def test_pairing_preserves_both_seeds_and_rejects_duplicates(self):
        rows = []
        for duration in (1.5, 5.0):
            for guidance in (1.0, 2.0, 4.5):
                modes = (
                    ("base", "adapted", "base-unconditional")
                    if guidance == 4.5
                    else ("base", "adapted")
                )
                for mode in modes:
                    for seed in (42, 123):
                        score = 0.3 if mode == "base" else (0.4 if seed == 42 else 0.25)
                        rows.append(
                            {
                                "mode": mode,
                                "duration": duration,
                                "guidance": guidance,
                                "seed": seed,
                                "clap": [score],
                            }
                        )
        summary = probe.paired_summary(rows)
        self.assertEqual(len(summary), 8)
        self.assertTrue(all(row["mean_delta"] > 0 for row in summary))
        self.assertFalse(any(row["both_seeds_improve"] for row in summary))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            probe.paired_summary(rows + rows[:1])


if __name__ == "__main__":
    unittest.main()
