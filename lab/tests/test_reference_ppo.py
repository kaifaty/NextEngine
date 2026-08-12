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
    build_sweep_report,
    build_throughput_report,
    parse_gpu_telemetry_csv,
)


PROFILE = Path(__file__).parents[1] / "profiles/humanoid-reference-ppo-tiny.v1.json"
CURRICULUM_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-ppo-curriculum-start-phase.v1.json"
)
SOFT_ROM_TINY_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-ppo-tiny-soft-rom-cost.v1.json"
)
SOFT_ROM_CURRICULUM_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-ppo-curriculum-start-phase-soft-rom-cost.v1.json"
)
PREDICTIVE_ROM_TINY_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-ppo-tiny-predictive-rom-cost.v1.json"
)
PREDICTIVE_ROM_CURRICULUM_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-ppo-curriculum-start-phase-predictive-rom-cost.v1.json"
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

    def test_soft_rom_tiny_profile_preserves_fixed_overfit_scope(self) -> None:
        profile = TinyReferencePpoProfile.load(SOFT_ROM_TINY_PROFILE)
        self.assertEqual(
            profile.document["environment_profile_id"],
            "nextengine.motor.env.humanoid-reference-tracker-soft-rom-cost.v1",
        )
        self.assertFalse(profile.document["scope"]["phase_randomization"])

    def test_soft_rom_curriculum_uses_reproducible_tiny_checkpoint(self) -> None:
        profile = TinyReferencePpoProfile.load(SOFT_ROM_CURRICULUM_PROFILE)
        self.assertEqual(
            profile.document["initialization"]["checkpoint_sha256"],
            "30778aa6b8b4d3c1b0b5ff87118a025969dc035df461dedff9c03eccbbebe8e2",
        )
        self.assertEqual(
            profile.document["evaluation"]["episode_matrix"],
            "fixed-vector-waves-v1",
        )

    def test_predictive_rom_tiny_variant_replaces_only_environment(self) -> None:
        profile = TinyReferencePpoProfile.load(PREDICTIVE_ROM_TINY_PROFILE)
        baseline = TinyReferencePpoProfile.load(PROFILE)
        self.assertEqual(
            profile.document["environment_profile_id"],
            "nextengine.motor.env.humanoid-reference-tracker-predictive-rom-cost.v1",
        )
        self.assertEqual(profile.document["network"], baseline.document["network"])
        self.assertEqual(profile.document["ppo"], baseline.document["ppo"])
        self.assertEqual(profile.document["scope"], baseline.document["scope"])

    def test_predictive_rom_curriculum_variant_preserves_fixed_matrix(self) -> None:
        profile = TinyReferencePpoProfile.load(PREDICTIVE_ROM_CURRICULUM_PROFILE)
        baseline = TinyReferencePpoProfile.load(CURRICULUM_PROFILE)
        self.assertEqual(profile.document["ppo"], baseline.document["ppo"])
        self.assertEqual(profile.document["evaluation"], baseline.document["evaluation"])
        self.assertEqual(
            profile.document["initialization"]["checkpoint_sha256"],
            "29e1e703c36f368c6bc1ae50ba4416f497f8a1b5673739e2299827891d4369ed",
        )

    def test_frozen_curriculum_stage_closes_phase_and_initial_checkpoint(self) -> None:
        profile = TinyReferencePpoProfile.load(CURRICULUM_PROFILE)
        scope = profile.document["scope"]
        self.assertTrue(scope["phase_randomization"])
        self.assertEqual(scope["eligible_clip_ids"], ["cmu104-start-right"])
        self.assertEqual(scope["horizon_motor_ticks"], 11)
        self.assertTrue(
            profile.document["execution"]["reset_episode_sequence_before_training"]
        )
        self.assertEqual(
            profile.document["evaluation"]["episode_matrix"],
            "fixed-vector-waves-v1",
        )
        self.assertEqual(
            profile.document["initialization"]["checkpoint_sha256"],
            "bda3df1d4770649e3f1cbf976997cb0d76a0a60ce88657f47c3e2a650e0a157b",
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

    def test_sweep_report_selects_fastest_hash_consistent_run(self) -> None:
        def run(run_id: str, num_envs: int, throughput: float) -> dict[str, object]:
            return {
                "run_id": run_id,
                "status": "completed",
                "claim": "PerformanceEvidenceOnly",
                "learned_policy_claim": False,
                "training_profile_sha256": "profile",
                "training_generation_manifest_hash": "generation",
                "repository": {"commit": "commit"},
                "resolved_execution": {"num_envs": num_envs},
                "resolved_ppo": {"minibatches": 4, "learning_rate": 0.0003},
                "performance_overrides": {},
                "throughput": {
                    "measured_iterations": 8,
                    "measured_samples": num_envs * 32 * 8,
                    "samples_per_second": throughput,
                    "gpu_telemetry": {
                        "sample_count": 4,
                        "gpu_utilization_percent": {"mean": 70.0},
                        "memory_used_mib": {"maximum": 6000.0},
                    },
                },
            }

        report = build_sweep_report(
            [run("n256", 256, 3000.0), run("n512", 512, 4500.0)]
        )
        self.assertEqual(report["winner_run_id"], "n512")
        self.assertEqual(report["winner_samples_per_second"], 4500.0)


if __name__ == "__main__":
    unittest.main()
