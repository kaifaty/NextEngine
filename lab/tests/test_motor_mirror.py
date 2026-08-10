from pathlib import Path
import unittest

import torch

from next_lab.isaac_env import fixed_pd_tensor, round_div_ties_even_tensor
from next_lab.motor_mirror import load_json, validate_golden


FIXTURE = Path(__file__).parent / "fixtures/stage0_motor_mirror_v1.json"


class MotorMirrorTests(unittest.TestCase):
    def test_rust_golden_matches_python_seed_and_pd(self) -> None:
        validate_golden(load_json(FIXTURE))

    def test_torch_integer_pd_matches_rust_golden(self) -> None:
        target = torch.full((2, 23), 1_000_000, dtype=torch.int64)
        zero = torch.zeros_like(target)
        first, flags = fixed_pd_tensor(target, zero, zero, zero)
        second, second_flags = fixed_pd_tensor(target, zero, zero, first)
        self.assertTrue(torch.all(first == 25_000_000))
        self.assertTrue(torch.all(second == 50_000_000))
        self.assertTrue(torch.all(flags == 4))
        self.assertTrue(torch.all(second_flags == 4))

    def test_tensor_ties_to_even_is_sign_symmetric(self) -> None:
        values = torch.tensor([5, 7, -5, -7], dtype=torch.int64)
        actual = round_div_ties_even_tensor(values, 2)
        torch.testing.assert_close(actual, torch.tensor([2, 4, -2, -4], dtype=torch.int64))


if __name__ == "__main__":
    unittest.main()
