"""Frozen-core magnitude factorization guards, not perceptual quality tests."""

import math
import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_magnitude as magnitude


class MagnitudeTests(unittest.TestCase):
    def test_core_is_frozen_and_magnitude_head_can_learn(self):
        model = magnitude.MagnitudeStudent()
        self.assertFalse(any(p.requires_grad for p in model.core.parameters()))
        self.assertTrue(all(p.requires_grad for p in model.magnitude.parameters()))
        result = model.positive_field(
            torch.zeros(4, 64), torch.ones(4, 1) * 0.5, torch.zeros(4, 3)
        )
        self.assertTrue(torch.all(result > 0))
        result.square().mean().backward()
        self.assertTrue(
            any(
                p.grad is not None and bool(p.grad.any())
                for p in model.magnitude.parameters()
            )
        )
        self.assertTrue(all(p.grad is None for p in model.core.parameters()))

    def test_poles_count_original_gains_and_signs_unchanged(self):
        model = magnitude.MagnitudeStudent().eval().requires_grad_(False)
        model.core.count.weight.zero_()
        model.core.count.bias.zero_()
        model.core.count_mean.fill_(math.log(4))
        cloud = np.zeros((512, 3), np.float32)
        features = np.array([0, 0, 0, 1, 0, 0, 0, 0], np.float32)
        contacts = np.zeros((2, 3), np.float32)
        original = magnitude.shared.predict(model.core, cloud, features, contacts)
        result = magnitude.predict(model, cloud, features, contacts)
        for key in ("frequency", "damping"):
            np.testing.assert_array_equal(result[key], original[key])
        np.testing.assert_array_equal(result["original_gains"], original["gains"])
        np.testing.assert_array_equal(
            np.sign(result["gains"]), np.sign(original["gains"])
        )


if __name__ == "__main__":
    unittest.main()
