import copy
import json
import random
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np
import torch
from next_lab.isaac_training import atomic_write_json, sha256_file
from tensordict import TensorDict

from lab.scripts.canonical_walking_ppo import ROOT, evaluation_claim, train
from lab.scripts.validate_walking_checkpoint import validate_checkpoint


class ValidationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.policy = torch.nn.Linear(2, 1)
        self.checkpoint = self.root / "model.pt"
        torch.save({"model_state_dict": self.policy.state_dict()}, self.checkpoint)
        self.profile = {
            "environment_profile_id": "test",
            "evaluation": {"seeds": list(range(1001, 1006))},
        }
        self.corrupt_lineage = False

    def evaluate(self, policy, headless, descriptor, profile, output):
        policy.eval()
        random.random()
        np.random.random()
        torch.rand(2)
        if torch.cuda.is_available():
            torch.rand(2, device="cuda")
        for seed in profile["evaluation"]["seeds"]:
            np.savez(output / f"evaluation-{seed}.npz", action=np.zeros((1, 23)))
        return {"episodes": [{"seed": s} for s in profile["evaluation"]["seeds"]]}

    def replay(self, command, **kwargs):
        tape = json.loads(Path(command[1]).read_text())
        atomic_write_json(
            Path(command[2]),
            {
                "action_tape_sha256": "bad"
                if self.corrupt_lineage
                else sha256_file(Path(command[1])),
                "profile_id": tape["profile_id"],
                "run_root": bytes(tape["run_root_bytes"]).hex(),
                "executable_sha256": sha256_file(self.checkpoint),
            },
        )

    def run_validation(self):
        with (
            patch(
                "lab.scripts.canonical_walking_ppo.evaluate", side_effect=self.evaluate
            ),
            patch(
                "lab.scripts.validate_walking_checkpoint.subprocess.run",
                side_effect=self.replay,
            ),
            patch(
                "lab.scripts.evaluate_corrected_walking.corrected_episode",
                side_effect=lambda episode, *a: (
                    {**episode, "passed": False},
                    {},
                    None,
                    None,
                ),
            ),
        ):
            return validate_checkpoint(
                self.policy,
                self.checkpoint,
                self.checkpoint,
                {},
                self.profile,
                self.root / "evaluation",
                self.checkpoint,
            )

    def test_failed_quality_still_closes_complete_matrix_and_restores_state_rng(self):
        before = copy.deepcopy(self.policy.state_dict())
        rng = (random.getstate(), np.random.get_state(), torch.get_rng_state())
        cuda = torch.cuda.get_rng_state_all() if torch.cuda.is_available() else None
        result = self.run_validation()
        self.assertEqual(result["status"], "failed")
        self.assertEqual(len(result["episodes"]), 5)
        self.assertTrue(self.policy.training)
        self.assertEqual(random.getstate(), rng[0])
        np.testing.assert_array_equal(np.random.get_state()[1], rng[1][1])
        self.assertTrue(torch.equal(torch.get_rng_state(), rng[2]))
        if cuda is not None:
            self.assertTrue(
                all(
                    torch.equal(a, b)
                    for a, b in zip(cuda, torch.cuda.get_rng_state_all())
                )
            )
        self.assertTrue(
            all(torch.equal(v, self.policy.state_dict()[k]) for k, v in before.items())
        )
        manifest = json.loads((self.root / "evaluation/run-manifest.json").read_text())
        self.assertEqual(manifest["status"], "completed")
        self.assertTrue(
            all(
                sha256_file(self.root / "evaluation" / p) == h
                for p, h in manifest["artifacts"].items()
            )
        )

    def test_bad_native_lineage_aborts_and_restores_mode(self):
        self.corrupt_lineage = True
        with self.assertRaisesRegex(ValueError, "lineage"):
            self.run_validation()
        self.assertTrue(self.policy.training)
        manifest = json.loads((self.root / "evaluation/run-manifest.json").read_text())
        self.assertEqual(manifest["status"], "failed")

    def test_checkpoint_mismatch_rejects_before_evaluation(self):
        with torch.no_grad():
            self.policy.weight.add_(1)
        with self.assertRaisesRegex(ValueError, "checkpoint does not match"):
            self.run_validation()
        self.assertFalse((self.root / "evaluation").exists())

    def test_incomplete_matrix_rejected(self):
        self.profile["evaluation"]["seeds"].pop()
        with self.assertRaisesRegex(ValueError, "five distinct"):
            self.run_validation()


class TrainingSelectionTests(unittest.TestCase):
    def test_pipeline_smoke_cannot_certify_even_passing_proxy_thresholds(self):
        for passed in (False, True):
            records = [{"seed": 1001, "passed": passed, "gates": {"example": passed}}]
            result = evaluation_claim({"run_class": "pipeline-smoke"}, records)
            self.assertEqual(result["status"], "report-only")
            self.assertNotIn("passed", result["episodes"][0])
            self.assertEqual(
                result["episodes"][0]["legacy_proxy_thresholds_met"], passed
            )
            self.assertEqual(
                evaluation_claim({}, records)["status"],
                "passed" if passed else "failed",
            )

    def test_pipeline_smoke_rejects_selection_before_environment_creation(self):
        with patch("lab.scripts.canonical_walking_ppo.make_env") as factory:
            with self.assertRaisesRegex(ValueError, "cannot select"):
                train(
                    None,
                    {},
                    {"run_class": "pipeline-smoke", "validation": {"interval": 1}},
                    Path("unused"),
                )
            factory.assert_not_called()

    def test_first_pass_stops_and_budget_failure_does_not_select(self):
        class Env:
            num_envs, num_actions = 4, 23
            sole_height_offset = 86

            def __init__(self):
                self.raw = np.zeros((4, 88))
                self.closed = False
                self.last_steps = [
                    SimpleNamespace(command_raw=np.zeros(3)) for _ in range(4)
                ]

            def get_observations(self):
                return TensorDict({"policy": torch.zeros(4, 88)}, [4])

            def step(self, actions):
                return (
                    self.get_observations(),
                    torch.ones(4),
                    torch.zeros(4, dtype=torch.bool),
                    {
                        "time_outs": torch.zeros(4),
                        "terminal_observation": self.get_observations(),
                    },
                )

            def close(self):
                self.closed = True

        for statuses in (["failed", "passed"], ["failed"] * 3):
            with self.subTest(statuses=statuses), tempfile.TemporaryDirectory() as d:
                profile = json.loads(
                    (ROOT / "lab/profiles/canonical-rsl-rl-walking.v4.json").read_text()
                )
                profile.update(
                    device="cpu", iterations=3, steps_per_env=2, save_interval=1
                )
                profile["policy"].update(
                    actor_hidden_dims=[16], critic_hidden_dims=[16]
                )
                profile["algorithm"].update(num_learning_epochs=1, num_mini_batches=1)
                profile["validation"]["interval"] = 1
                env = Env()
                with (
                    patch(
                        "lab.scripts.canonical_walking_ppo.make_env", return_value=env
                    ),
                    patch(
                        "lab.scripts.validate_walking_checkpoint.validate_checkpoint",
                        side_effect=[{"status": s} for s in statuses],
                    ) as validation,
                ):
                    result = train(None, {}, profile, Path(d), auditor=Path("unused"))
                self.assertEqual(result["status"], statuses[-1])
                self.assertEqual(validation.call_count, len(statuses))
                self.assertTrue(env.closed)
                rows = [
                    json.loads(s)
                    for s in (Path(d) / "metrics.jsonl").read_text().splitlines()
                ]
                self.assertEqual(len(rows), len(statuses))
                self.assertTrue(
                    all(r["learning_rate"] == 1e-5 and "ppo_update" in r for r in rows)
                )
                self.assertEqual(
                    (Path(d) / "selection.json").exists(), statuses[-1] == "passed"
                )
                if statuses[-1] == "passed":
                    self.assertFalse((Path(d) / "model_2.pt").exists())
                    self.assertEqual(
                        json.loads((Path(d) / "selection.json").read_text())[
                            "iteration"
                        ],
                        1,
                    )


if __name__ == "__main__":
    unittest.main()
