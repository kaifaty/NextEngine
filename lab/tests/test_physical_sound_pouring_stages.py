import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_pilot as p
from physical_sound_pouring_stages import StagedField, endpoint_oracle


class Constant(torch.nn.Module):
    def __init__(self, value):
        super().__init__()
        self.value = torch.nn.Parameter(torch.tensor(float(value)), requires_grad=False)

    def forward(self, x, t, c):
        return torch.ones_like(x) * self.value


class StageTests(unittest.TestCase):
    def test_reference_endpoint_recovers_velocity_at_each_time(self):
        target = torch.tensor([[-1.0, 0.5], [0.7, -0.2]], dtype=torch.float64)
        noise = torch.tensor([[0.3, -0.1], [-0.6, 0.8]], dtype=torch.float64)
        for t in (0, 0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 0.95):
            xt = (1 - t) * noise + t * target
            torch.testing.assert_close(
                endpoint_oracle(target, xt, t), target - noise, atol=1e-12, rtol=0
            )

    def test_pure_endpoints_reproduce_original_sampler(self):
        a, b = Constant(0.1), Constant(0.2)
        controls = np.zeros(11, dtype=np.float32)
        for switch, expected in [(0, b), (1, a)]:
            np.testing.assert_array_equal(
                p.sample(StagedField(a, b, switch), controls),
                p.sample(expected, controls),
            )

    def test_boundary_selects_late_and_rejects_mixed_times(self):
        model = StagedField(Constant(1), Constant(2), 0.25)
        x = torch.zeros(1, 1, 2, 2)
        c = torch.zeros(1, 11)
        self.assertEqual(float(model(x, torch.tensor([0.249]), c).mean()), 1)
        self.assertEqual(float(model(x, torch.tensor([0.25]), c).mean()), 2)
        with self.assertRaises(ValueError):
            model(x, torch.tensor([0.1, 0.5]), c)
        for bad in [-1, 2, float("nan")]:
            with self.assertRaises(ValueError):
                StagedField(Constant(1), Constant(2), bad)


if __name__ == "__main__":
    unittest.main()
