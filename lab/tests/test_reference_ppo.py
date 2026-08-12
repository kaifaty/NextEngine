from __future__ import annotations

import math
import unittest
from pathlib import Path
from unittest import mock

import torch
from torch.distributions import Normal

from next_lab.reference_ppo import (
    TanhActorCritic,
    TinyReferencePpoProfile,
    _transformed_log_probability,
)
from next_lab.reference_performance import (
    PhaseTiming,
    assert_no_competing_training_process,
    build_throughput_report,
    parse_gpu_telemetry_csv,
)


PROFILE = Path(__file__).parents[1] / "profiles/humanoid-reference-ppo-tiny.v1.json"
CURRICULUM_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-ppo-curriculum-start-phase.v1.json"
)


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
        self.assertEqual(execution["cublas_workspace_config"], ":4096:8")
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

    def test_frozen_curriculum_stage_closes_phase_and_initial_checkpoint(self) -> None:
        profile = TinyReferencePpoProfile.load(CURRICULUM_PROFILE)
        scope = profile.document["scope"]
        self.assertTrue(scope["phase_randomization"])
        self.assertEqual(scope["eligible_clip_ids"], ["cmu104-start-right"])
        self.assertEqual(scope["horizon_motor_ticks"], 11)
        self.assertEqual(
            profile.document["initialization"]["checkpoint_sha256"],
            "3431d1a83ed429eb8978f531cab59f34ad59c06f19dea7f1330f2accfe588acb",
        )

    def test_report_only_throughput_summary_excludes_warmup(self) -> None:
        report = build_throughput_report(
            timings=[
                PhaseTiming(4, "rollout", 12_000_000, 10.0),
                PhaseTiming(4, "update", 3_000_000, 2.0),
                PhaseTiming(5, "rollout", 10_000_000, 8.0),
                PhaseTiming(5, "update", 2_000_000, 1.0),
            ],
            elapsed_nanoseconds=20_000_000,
            num_envs=64,
            rollout_steps_per_env=32,
            warmup_iterations=3,
            telemetry=[
                {
                    "gpu_utilization_percent": 40,
                    "memory_utilization_percent": 2,
                    "memory_used_mib": 3771,
                    "power_watts": 130.0,
                    "temperature_celsius": 67,
                    "sm_clock_mhz": 1920,
                }
            ],
        )
        self.assertEqual(report["claim"], "PerformanceEvidenceOnly")
        self.assertEqual(report["measured_iterations"], 2)
        self.assertEqual(report["measured_samples"], 4096)
        self.assertEqual(report["samples_per_second"], 204800.0)
        self.assertEqual(report["phase_cuda_milliseconds"]["rollout"]["mean"], 9.0)

    def test_gpu_telemetry_parser_and_exclusive_training_preflight(self) -> None:
        self.assertEqual(
            parse_gpu_telemetry_csv("39, 1, 3771, 130.25, 67, 1920, P2\n"),
            {
                "gpu_utilization_percent": 39,
                "memory_utilization_percent": 1,
                "memory_used_mib": 3771,
                "power_watts": 130.25,
                "temperature_celsius": 67,
                "sm_clock_mhz": 1920,
                "pstate": "P2",
            },
        )
        result = mock.Mock(stdout="123, /usr/bin/python3\n456, /opt/google/chrome\n")
        with self.assertRaisesRegex(RuntimeError, "123:/usr/bin/python3"):
            assert_no_competing_training_process(
                device_index=0, current_pid=999, runner=mock.Mock(return_value=result)
            )


if __name__ == "__main__":
    unittest.main()
