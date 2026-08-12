from __future__ import annotations

import math
import unittest
from pathlib import Path

import torch
from torch.distributions import Normal

from next_lab.reference_ppo import (
    TanhActorCritic,
    TinyReferencePpoProfile,
    _transformed_log_probability,
)


PROFILE = Path(__file__).parents[1] / "profiles/humanoid-reference-ppo-tiny.v1.json"


class ReferencePpoTests(unittest.TestCase):
    def test_frozen_tiny_profile_closes_transformed_policy_and_batch(self) -> None:
        profile = TinyReferencePpoProfile.load(PROFILE)
        self.assertEqual(len(profile.sha256), 64)
        self.assertEqual(
            profile.document["policy_distribution"]["kind"],
            "diagonal-normal-followed-by-tanh",
        )
        execution = profile.document["execution"]
        ppo = profile.document["ppo"]
        self.assertEqual(
            execution["num_envs"] * execution["rollout_steps_per_env"]
            % ppo["minibatches"],
            0,
        )

    def test_tanh_log_probability_includes_jacobian_and_actions_are_bounded(self) -> None:
        distribution = Normal(torch.zeros(2), torch.ones(2))
        pre_tanh = torch.tensor([[0.0, 1.0]], dtype=torch.float32)
        actual = _transformed_log_probability(distribution, pre_tanh)
        expected = torch.sum(
            distribution.log_prob(pre_tanh)
            - torch.log(torch.clamp(1.0 - torch.tanh(pre_tanh).square(), min=1.0e-12)),
            dim=-1,
        )
        torch.testing.assert_close(actual, expected)

        torch.manual_seed(7)
        model = TanhActorCritic(TinyReferencePpoProfile.load(PROFILE))
        action, _, log_probability, value = model.sample(torch.zeros((3, 435)))
        self.assertTrue(torch.all(action > -1.0))
        self.assertTrue(torch.all(action < 1.0))
        self.assertTrue(torch.isfinite(log_probability).all())
        self.assertTrue(torch.isfinite(value).all())
        self.assertTrue(math.isfinite(float(model.log_std.mean().item())))


if __name__ == "__main__":
    unittest.main()
