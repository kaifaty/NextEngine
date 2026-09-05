from __future__ import annotations

import copy
import unittest

import torch
from next_lab.canonical_ppo import TerminalObservationPPO
from next_lab.ppo_diagnostics import (
    distribution_metrics,
    observe_ppo_update,
    summarize_update,
)
from rsl_rl.modules import ActorCritic
from tensordict import TensorDict


def algorithm_fixture(device, canonical=False):
    count, width, actions = (128, 88, 23) if canonical else (8, 4, 3)
    hidden = [256, 128, 64] if canonical else [16, 8]
    obs = TensorDict({"policy": torch.zeros(count, width, device=device)}, [count])
    policy = ActorCritic(
        obs,
        {"policy": ["policy"], "critic": ["policy"]},
        actions,
        actor_hidden_dims=hidden,
        critic_hidden_dims=hidden,
        activation="elu",
        actor_obs_normalization=True,
        critic_obs_normalization=True,
        init_noise_std=0.25,
    ).to(device)
    algorithm = TerminalObservationPPO(
        policy,
        device=device,
        num_learning_epochs=5 if canonical else 2,
        num_mini_batches=4 if canonical else 2,
        desired_kl=0.008,
        schedule="adaptive",
        learning_rate=1e-4,
        max_grad_norm=1,
        gamma=0.99,
        lam=0.95,
        entropy_coef=0.001,
    )
    algorithm.init_storage("rl", count, 32 if canonical else 4, obs, [actions])
    return algorithm


def fill(algorithm):
    device = algorithm.device
    steps, count, width = algorithm.storage.observations["policy"].shape
    obs = TensorDict({"policy": torch.randn(count, width, device=device)}, [count])
    with torch.inference_mode():
        for tick in range(steps):
            action = algorithm.act(obs)
            obs = TensorDict(
                {"policy": torch.randn(count, width, device=device)}, [count]
            )
            done = torch.tensor([tick == steps - 1] * count, device=device)
            timeout = done.clone()
            timeout[0] = False  # Includes both real termination and truncation.
            algorithm.process_env_step(
                obs,
                10 - action.square().sum(-1),
                done,
                {"time_outs": timeout, "terminal_observation": obs.clone()},
            )
        algorithm.compute_returns(obs)


class PpoDiagnosticsTests(unittest.TestCase):
    def test_analytic_gaussian_and_ratio_oracle(self):
        old_mean = torch.zeros(2, 3)
        mean = torch.ones(2, 3)
        std = torch.ones(2, 3)
        row = distribution_metrics(
            mean, std, old_mean, std, torch.tensor([0.0, 2.0]), torch.zeros(2), 0.2
        )
        self.assertEqual(row["analytic_kl_mean"], 1.5)
        self.assertEqual(row["analytic_kl_max"], 1.5)
        self.assertEqual(row["ratio_clip_fraction"], 0.5)
        same = distribution_metrics(
            mean, std, mean, std, torch.zeros(2), torch.zeros(2), 0.2
        )
        self.assertEqual(same["analytic_kl_mean"], 0)
        self.assertEqual(same["ratio_clip_fraction"], 0)

    def test_reject_invalid_inputs_and_empty_summary(self):
        mean, std = torch.zeros(2, 3), torch.ones(2, 3)
        for bad in (torch.zeros(2, 3), torch.full((2, 3), torch.nan)):
            with self.assertRaises(ValueError):
                distribution_metrics(
                    mean, bad, mean, std, torch.zeros(2), torch.zeros(2), 0.2
                )
        with self.assertRaises(ValueError):
            summarize_update([])

    def assert_nested_exact(self, left, right):
        if isinstance(left, torch.Tensor):
            self.assertTrue(torch.equal(left, right))
        elif isinstance(left, dict):
            self.assertEqual(left.keys(), right.keys())
            for key in left:
                self.assert_nested_exact(left[key], right[key])
        elif isinstance(left, (list, tuple)):
            self.assertEqual(len(left), len(right))
            for a, b in zip(left, right, strict=True):
                self.assert_nested_exact(a, b)
        else:
            self.assertEqual(left, right)

    def check_noninterference(self, device, canonical=False):
        torch.set_num_threads(1)
        torch.backends.cuda.matmul.allow_tf32 = False
        torch.backends.cudnn.allow_tf32 = False
        torch.manual_seed(44)
        baseline = algorithm_fixture(device, canonical)
        observed = copy.deepcopy(baseline)
        for iteration in range(2):
            torch.manual_seed(100 + iteration)
            fill(baseline)
            torch.manual_seed(100 + iteration)
            fill(observed)
            norm_before = {
                k: v.clone()
                for k, v in observed.policy.state_dict().items()
                if "normalizer" in k
            }
            torch.manual_seed(200 + iteration)
            losses_a = baseline.update()
            rng_a = torch.get_rng_state().clone()
            cuda_a = torch.cuda.get_rng_state().clone() if device == "cuda:0" else None
            torch.manual_seed(200 + iteration)
            with observe_ppo_update(observed) as records:
                losses_b = observed.update()
            self.assertEqual(len(records), 20 if canonical else 4)
            self.assertEqual(
                summarize_update(records)["sample_visits"], 20480 if canonical else 64
            )
            self.assertEqual(losses_a, losses_b)
            self.assertTrue(torch.equal(rng_a, torch.get_rng_state()))
            if cuda_a is not None:
                self.assertTrue(torch.equal(cuda_a, torch.cuda.get_rng_state()))
            self.assert_nested_exact(
                baseline.policy.state_dict(), observed.policy.state_dict()
            )
            self.assert_nested_exact(
                baseline.optimizer.state_dict(), observed.optimizer.state_dict()
            )
            self.assertEqual(baseline.learning_rate, observed.learning_rate)
            for key, value in norm_before.items():
                self.assertTrue(torch.equal(value, observed.policy.state_dict()[key]))
            self.assertNotIn("mini_batch_generator", vars(observed.storage))
            self.assertFalse(observed.optimizer._optimizer_step_pre_hooks)
            self.assertTrue(
                all(not p._backward_hooks for p in observed.policy.parameters())
            )
            self.assertTrue(
                all(r["gradient_norm"] >= r["post_clip_gradient_norm"] for r in records)
            )
            self.assertTrue(
                all(r["post_clip_gradient_norm"] <= 1.000001 for r in records)
            )

    def test_cpu_noninterference_two_complete_updates(self):
        self.check_noninterference("cpu")

    @unittest.skipUnless(torch.cuda.is_available(), "CUDA unavailable")
    def test_cuda_noninterference_two_complete_updates(self):
        self.check_noninterference("cuda:0")

    @unittest.skipUnless(torch.cuda.is_available(), "CUDA unavailable")
    def test_canonical_network_and_batch_cuda_noninterference(self):
        self.check_noninterference("cuda:0", canonical=True)

    def test_hooks_restore_after_error_and_reject_nested_observation(self):
        algorithm = algorithm_fixture("cpu")
        with (
            self.assertRaisesRegex(RuntimeError, "control failure"),
            observe_ppo_update(algorithm),
        ):
            with (
                self.assertRaisesRegex(ValueError, "already-observed"),
                observe_ppo_update(algorithm),
            ):
                pass
            raise RuntimeError("control failure")
        self.assertNotIn("mini_batch_generator", vars(algorithm.storage))
        self.assertFalse(algorithm.optimizer._optimizer_step_pre_hooks)
        self.assertTrue(
            all(not p._backward_hooks for p in algorithm.policy.parameters())
        )

    def test_cleanup_after_backward_failure_preserves_external_hook(self):
        algorithm = algorithm_fixture("cpu")
        fill(algorithm)

        def fail(optimizer, args, kwargs):
            raise RuntimeError("pre-step failure")

        external = algorithm.optimizer.register_step_pre_hook(fail)
        try:
            with (
                self.assertRaisesRegex(RuntimeError, "pre-step failure"),
                observe_ppo_update(algorithm),
            ):
                algorithm.update()
            self.assertNotIn("mini_batch_generator", vars(algorithm.storage))
            self.assertEqual(len(algorithm.optimizer._optimizer_step_pre_hooks), 1)
            self.assertTrue(
                all(not p._backward_hooks for p in algorithm.policy.parameters())
            )
        finally:
            external.remove()


if __name__ == "__main__":
    unittest.main()
