from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from lab.scripts.canonical_walking_ppo import artifact_hashes, diagnostic_evaluation
from lab.scripts.verify_walking_lift_return import verify
from lab.tests.test_cpu_walking_contact_audit import fixture


class WalkingV7Checks(unittest.TestCase):
    def test_native_verifier_rejects_geometry_cost_and_physics_corruption(self):
        source, descriptor, _ = fixture()
        frame = source["frames"][0][0]
        frame["physics_root"] = "a" * 64
        frame["reward_components_q16"] = [0] * 11
        source["run_root"] = "b" * 64
        source["cases"] = [{"ticks": 1, "terminal": None}]
        new = copy.deepcopy(source)
        new["profile_id"] = (
            "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v7"
        )
        new["frames"][0][0]["observation_raw"] = [0] * 88
        new["frames"][0][0]["reward_components_q16"] += [0, 0]
        self.assertEqual(verify(source, new, descriptor)["status"], "passed")
        for key, index in (
            ("observation_raw", 86),
            ("reward_components_q16", 11),
            ("command_raw", 0),
        ):
            bad = copy.deepcopy(new)
            bad["frames"][0][0][key][index] += 100
            with self.subTest(key=key), self.assertRaises(ValueError):
                verify(source, bad, descriptor)

    def test_diagnostic_restores_mode_on_success_and_failure_and_closes_nested_files(
        self,
    ):
        policy = torch.nn.Linear(2, 1)
        initial = copy.deepcopy(policy.state_dict())
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)

            def evaluate(*args):
                policy.eval()
                return {"status": "failed"}

            with patch(
                "lab.scripts.canonical_walking_ppo.evaluate", side_effect=evaluate
            ):
                diagnostic_evaluation(policy, None, {}, {}, output, 999)
            self.assertTrue(policy.training)
            hashes = artifact_hashes(output)
            self.assertIn("diagnostic-999/evaluation.json", hashes)
            self.assertEqual(len(hashes), 1)
            for key, value in policy.state_dict().items():
                torch.testing.assert_close(value, initial[key], rtol=0, atol=0)

            def fail(*args):
                policy.eval()
                raise RuntimeError("evaluation failure")

            with (
                patch("lab.scripts.canonical_walking_ppo.evaluate", side_effect=fail),
                self.assertRaisesRegex(RuntimeError, "evaluation failure"),
            ):
                diagnostic_evaluation(policy, None, {}, {}, output, 3999)
            self.assertTrue(policy.training)
            with self.assertRaises(FileExistsError):
                diagnostic_evaluation(policy, None, {}, {}, output, 999)


if __name__ == "__main__":
    unittest.main()
