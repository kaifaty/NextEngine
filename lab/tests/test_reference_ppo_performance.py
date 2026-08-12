from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

import torch

from next_lab.reference_ppo import TinyReferencePpoProfile, TinyReferencePpoTrainer


PROFILE = Path(__file__).parents[1] / "profiles/humanoid-reference-ppo-tiny.v1.json"


class ReferencePpoPerformanceTests(unittest.TestCase):
    def test_rollout_storage_is_preallocated_and_reused(self) -> None:
        document = copy.deepcopy(TinyReferencePpoProfile.load(PROFILE).document)
        document["execution"].update(
            {"device": "cpu", "num_envs": 2, "rollout_steps_per_env": 3}
        )
        document["ppo"]["minibatches"] = 1
        profile = TinyReferencePpoProfile(document=document, sha256="test-profile")

        class FakeEnvironment:
            num_envs = 2

            def __init__(self) -> None:
                self.steps = 0
                self.last_step_success = torch.zeros(2, dtype=torch.bool)
                self.last_step_failure = torch.zeros(2, dtype=torch.bool)

            def step(self, _: torch.Tensor):
                self.steps += 1
                observation = torch.full((2, 435), float(self.steps))
                reward = torch.full((2,), float(self.steps))
                done = torch.zeros(2, dtype=torch.bool)
                return {"policy": observation}, reward, done, done, {}

        environment = FakeEnvironment()
        with tempfile.TemporaryDirectory() as directory:
            trainer = TinyReferencePpoTrainer(
                environment, profile, Path(directory) / "metrics.jsonl"
            )
            first = trainer._collect_rollout(torch.zeros((2, 435)), 3)
            storage_pointer = first["observation"].data_ptr()
            torch.testing.assert_close(
                first["reward"],
                torch.tensor([[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]),
            )
            second = trainer._collect_rollout(first["next_observation"], 3)

        self.assertEqual(second["observation"].data_ptr(), storage_pointer)
        self.assertEqual(trainer.samples, 12)
        with self.assertRaisesRegex(ValueError, "resolved training profile"):
            trainer._collect_rollout(second["next_observation"], 2)


if __name__ == "__main__":
    unittest.main()
