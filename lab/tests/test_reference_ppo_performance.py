from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

import torch

from next_lab.reference_performance import resolve_performance_overrides
from next_lab.reference_ppo import (
    TinyReferencePpoProfile,
    TinyReferencePpoTrainer,
    _selection_episode_matrix_hash,
)


PROFILE = Path(__file__).parents[1] / "profiles/humanoid-reference-ppo-tiny.v1.json"


class ReferencePpoPerformanceTests(unittest.TestCase):
    def test_report_only_overrides_are_bounded_and_hash_visible(self) -> None:
        profile = TinyReferencePpoProfile.load(PROFILE)
        resolved, overrides = resolve_performance_overrides(
            profile.document,
            iterations=10,
            num_envs=1_024,
            minibatches=8,
            learning_rate=0.0006,
        )
        self.assertEqual(resolved["execution"]["num_envs"], 1_024)
        self.assertEqual(resolved["ppo"]["minibatches"], 8)
        self.assertEqual(overrides["num_envs"], {"source": 64, "resolved": 1_024})
        with self.assertRaisesRegex(ValueError, "between 1 and 4096"):
            resolve_performance_overrides(profile.document, num_envs=4_097)
        with self.assertRaisesRegex(ValueError, "divide evenly"):
            resolve_performance_overrides(
                profile.document, num_envs=65, minibatches=3
            )

    def test_selection_episode_matrix_hash_ignores_results_but_not_sampling(self) -> None:
        first = {
            "clip:1": {"episodes": 2, "failure_count": 2},
            "clip:0": {"episodes": 1, "failure_count": 0},
        }
        same_matrix = {
            "clip:0": {"episodes": 1, "failure_count": 1},
            "clip:1": {"episodes": 2, "failure_count": 0},
        }
        changed_matrix = {
            "clip:0": {"episodes": 2, "failure_count": 1},
            "clip:1": {"episodes": 1, "failure_count": 0},
        }
        self.assertEqual(
            _selection_episode_matrix_hash(first),
            _selection_episode_matrix_hash(same_matrix),
        )
        self.assertNotEqual(
            _selection_episode_matrix_hash(first),
            _selection_episode_matrix_hash(changed_matrix),
        )

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
