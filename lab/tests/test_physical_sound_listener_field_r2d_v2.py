from __future__ import annotations

import copy
import math
import sys
import unittest
from pathlib import Path

import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_listener_field_r2d_v2_common as common  # noqa: E402
import physical_sound_listener_field_r2d_v2_trainability as runner  # noqa: E402


class PhysicalSoundListenerFieldR2DV2Tests(unittest.TestCase):
    def test_profile_changes_only_identity_and_learning_rate_schedule(self) -> None:
        v1_profile = copy.deepcopy(common.v1.profile())
        v2_profile = copy.deepcopy(common.profile())
        self.assertEqual(v2_profile["representation"], v1_profile["representation"])
        self.assertEqual(v2_profile["model"], v1_profile["model"])
        self.assertEqual(v2_profile["objective"], v1_profile["objective"])
        self.assertEqual(v2_profile["controls"], v1_profile["controls"])
        self.assertEqual(v2_profile["gates"], v1_profile["gates"])
        self.assertFalse(v2_profile["query_audio_available"])
        self.assertFalse(v2_profile["method_holdout_or_shadow_available"])
        v1_optimizer = v1_profile["optimizer"]
        v2_optimizer = v2_profile["optimizer"]
        self.assertEqual(v1_optimizer["tasks"], v2_optimizer["tasks"])
        self.assertEqual(v1_optimizer["betas"], v2_optimizer["betas"])
        self.assertEqual(v1_optimizer["epsilon"], v2_optimizer["epsilon"])
        self.assertEqual(
            v1_optimizer["weight_decay"], v2_optimizer["weight_decay"]
        )
        self.assertEqual(
            v1_optimizer["gradient_clip_norm"],
            v2_optimizer["gradient_clip_norm"],
        )
        self.assertEqual(
            v1_optimizer["learning_rate"],
            v2_optimizer["learning_rate_schedule"]["initial"],
        )

    def test_half_cosine_schedule_is_bounded_monotonic_and_frozen(self) -> None:
        steps = 1_600
        values = [
            common.learning_rate_for_step(index, steps) for index in range(steps)
        ]
        self.assertEqual(values[0], common.INITIAL_LEARNING_RATE)
        self.assertEqual(values[-1], common.FINAL_LEARNING_RATE)
        self.assertTrue(all(left >= right for left, right in zip(values, values[1:])))
        expected_midpoint = common.FINAL_LEARNING_RATE + 0.5 * (
            common.INITIAL_LEARNING_RATE - common.FINAL_LEARNING_RATE
        )
        measured_midpoint = common.learning_rate_for_step((steps - 1) // 2, steps)
        self.assertLess(abs(measured_midpoint - expected_midpoint), 5.0e-5)
        with self.assertRaises(common.R2DError):
            common.learning_rate_for_step(-1, steps)
        with self.assertRaises(common.R2DError):
            common.learning_rate_for_step(steps, steps)

    def test_schedule_samples_cover_unchanged_task_endpoints(self) -> None:
        samples = runner.schedule_samples()
        self.assertEqual(set(samples), {task["task_id"] for task in common.TASKS})
        for task in common.TASKS:
            task_samples = samples[task["task_id"]]
            self.assertEqual(task_samples[0]["optimizer_step"], 1)
            self.assertEqual(task_samples[-1]["optimizer_step"], task["steps"])
            self.assertEqual(
                task_samples[0]["learning_rate"], common.INITIAL_LEARNING_RATE
            )
            self.assertEqual(
                task_samples[-1]["learning_rate"], common.FINAL_LEARNING_RATE
            )

    def test_synthetic_table_converges_under_frozen_schedule(self) -> None:
        torch.manual_seed(common.SEED)
        target = torch.randn((4, 6, 2), dtype=torch.float32) * 0.4
        component_rms = torch.linspace(0.5, 1.5, 6)
        projection = torch.zeros((6, 2))
        mean_energy = torch.tensor(5.0)
        model = common.CoefficientTable(4, rank=6)
        optimizer = torch.optim.AdamW(
            model.parameters(),
            lr=common.INITIAL_LEARNING_RATE,
            betas=common.ADAM_BETAS,
            eps=common.ADAM_EPSILON,
            weight_decay=common.WEIGHT_DECAY,
        )
        steps = 400
        clipped = 0
        for step in range(steps):
            for group in optimizer.param_groups:
                group["lr"] = common.learning_rate_for_step(step, steps)
            objective, _ = common.v1.coefficient_objective(
                model(), target, component_rms, mean_energy, projection
            )
            optimizer.zero_grad(set_to_none=True)
            objective.backward()
            norm = torch.nn.utils.clip_grad_norm_(
                model.parameters(), common.GRADIENT_CLIP_NORM
            )
            clipped += int(float(norm) > common.GRADIENT_CLIP_NORM)
            optimizer.step()
        final, parts = common.v1.coefficient_objective(
            model(), target, component_rms, mean_energy, projection
        )
        self.assertEqual(clipped, 0)
        self.assertLess(float(final.detach()), 1.0e-10)
        self.assertLess(
            math.sqrt(float(parts["whitened_coefficient_mse"].detach())), 1.0e-5
        )


if __name__ == "__main__":
    unittest.main()
