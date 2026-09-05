import copy
import unittest

import torch

from lab.scripts.diagnose_canonical_ppo_update import check_returns, run_arm
from lab.tests.test_ppo_diagnostics import algorithm_fixture, fill


class GradientProbeTests(unittest.TestCase):
    def test_zero_lr_is_a_true_control_and_active_arm_changes_only_copy(self):
        torch.manual_seed(44)
        original = algorithm_fixture("cpu")
        fill(original)
        before = {k: v.clone() for k, v in original.policy.state_dict().items()}
        zero = run_arm(copy.deepcopy(original), "zero-effective-lr", 55, 4)
        active = run_arm(copy.deepcopy(original), "source-adaptive-lr", 55, 4)
        self.assertEqual(zero["policy_max_parameter_delta"], 0)
        self.assertEqual(zero["pre"], zero["post"])
        self.assertGreater(active["policy_max_parameter_delta"], 0)
        self.assertGreater(active["post"]["analytic_kl_mean"], 0)
        self.assertTrue(all(r["learning_rate"] == 0 for r in zero["minibatches"]))
        self.assertTrue(all(r["learning_rate"] > 0 for r in active["minibatches"]))
        self.assertTrue(
            all(
                torch.equal(v, original.policy.state_dict()[k])
                for k, v in before.items()
            )
        )

    def test_return_oracle_rejects_corrupted_targets(self):
        algorithm = algorithm_fixture("cpu")
        fill(algorithm)
        final = torch.zeros(8, 1)
        # Last transition is a true done, so its following value is masked.
        check_returns(algorithm.storage, final, algorithm.gamma, algorithm.lam)
        algorithm.storage.returns[0, 0] += 1
        with self.assertRaises(AssertionError):
            check_returns(algorithm.storage, final, algorithm.gamma, algorithm.lam)

    def test_unknown_arm_rejected(self):
        with self.assertRaisesRegex(ValueError, "unknown"):
            run_arm(algorithm_fixture("cpu"), "another-lr", 0, 4)


if __name__ == "__main__":
    unittest.main()
