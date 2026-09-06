from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path

import numpy as np
import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_listener_field_r2c as trainer  # noqa: E402
import physical_sound_listener_field_r2c_common as common  # noqa: E402
import physical_sound_listener_field_r2c_evaluate as evaluator  # noqa: E402


class PhysicalSoundListenerFieldR2CTests(unittest.TestCase):
    def test_profile_freezes_only_the_two_authorized_candidates(self) -> None:
        profile = common.candidate_profile()
        self.assertEqual(profile["training"]["steps"], 8_000)
        self.assertEqual(profile["model"]["latent_width"], 96)
        self.assertEqual(
            profile["physics_ablation"]["candidate_weights"],
            {
                "dense_complex_field_data_only_v1": 0.0,
                "dense_complex_field_helmholtz_v1": 0.0001,
            },
        )
        self.assertFalse(profile["physics_ablation"]["query_audio_used"])
        self.assertFalse(profile["query_audio_available_to_training"])
        self.assertEqual(len(profile["candidate_ladder"]), 2)

    def test_separable_field_and_losses_are_finite_on_cpu(self) -> None:
        torch.manual_seed(common.SEED)
        model = common.SeparableComplexField(
            np.zeros(3, dtype=np.float32),
            np.ones(3, dtype=np.float32),
            4.0,
        )
        listener = torch.tensor(
            [[1.0, 0.0, -0.2], [0.0, 1.0, 0.2]], dtype=torch.float32
        )
        impact = torch.zeros((1, 3), dtype=torch.float32)
        times = torch.tensor([0.0, 1.0, 2.0], dtype=torch.float32)
        frequencies = torch.tensor([93.75, 1_000.0, 12_000.0], dtype=torch.float32)
        prediction = model(listener, impact, times, frequencies)
        self.assertEqual(tuple(prediction.shape), (2, 3, 2))
        self.assertTrue(torch.isfinite(prediction).all())
        total, complex_l1, log_l1 = common.data_losses(
            prediction, prediction.detach().clone(), 0.1
        )
        self.assertTrue(torch.isfinite(total))
        self.assertLess(float(complex_l1.detach()), 1.0e-6)
        self.assertLess(float(log_l1.detach()), 1.0e-6)

    def test_collocation_midpoints_are_complete_and_query_audio_free(self) -> None:
        rows = []
        for angle in common.CONTEXT_ANGLES:
            radians = math.radians(angle)
            for distance in (0, 333, 666, 1_000):
                radius = 1.0 + distance / 1_000.0
                for microphone in range(common.r2b.MICROPHONES_PER_COLUMN):
                    rows.append(
                        {
                            "split_role": "context",
                            "azimuth_degrees": angle,
                            "gantry_distance_offset_millimetres": distance,
                            "microphone_id": microphone,
                            "listener_position_metres": [
                                radius * math.cos(radians),
                                radius * math.sin(radians),
                                (microphone - 7) * 0.13,
                            ],
                        }
                    )
        first = common.collocation_positions(rows)
        second = common.collocation_positions(rows)
        self.assertEqual(first.shape, (360, 3))
        np.testing.assert_array_equal(first, second)
        self.assertEqual(
            common.candidate_profile()["physics_ablation"]["collocation"],
            "cartesian_midpoints_between_adjacent_context_azimuth_planes",
        )

    def test_finite_difference_helmholtz_loss_backpropagates(self) -> None:
        torch.manual_seed(common.SEED)
        model = common.SeparableComplexField(
            np.zeros(3, dtype=np.float32),
            np.ones(3, dtype=np.float32),
            4.0,
        )
        positions = torch.tensor(
            [[1.0, 0.0, -0.1], [0.8, 0.3, 0.1]], dtype=torch.float32
        )
        impact = torch.zeros((1, 3), dtype=torch.float32)
        times = torch.tensor([0.0, 1.0], dtype=torch.float32)
        frequencies = torch.tensor([93.75, 1_000.0], dtype=torch.float32)
        loss = trainer.physics_loss(model, positions, impact, times, frequencies)
        self.assertTrue(torch.isfinite(loss))
        loss.backward()
        self.assertTrue(
            all(
                parameter.grad is None or torch.isfinite(parameter.grad).all()
                for parameter in model.parameters()
            )
        )

    def test_selection_prefers_simpler_candidate_without_physics_dominance(self) -> None:
        data = {
            "candidate_id": "dense_complex_field_data_only_v1",
            "passes_frozen_r2c_rule": True,
            "aggregate": {endpoint: 1.0 for endpoint in common.PRIMARY_ENDPOINTS},
        }
        physics = {
            "candidate_id": "dense_complex_field_helmholtz_v1",
            "passes_frozen_r2c_rule": True,
            "aggregate": {endpoint: 0.9 for endpoint in common.PRIMARY_ENDPOINTS},
        }
        selected, decision = evaluator.select_candidate([data, physics])
        self.assertEqual(selected, physics["candidate_id"])
        self.assertEqual(decision, "GoComplexListenerField")
        physics["aggregate"][common.PRIMARY_ENDPOINTS[-1]] = 1.1
        selected, _ = evaluator.select_candidate([data, physics])
        self.assertEqual(selected, data["candidate_id"])


if __name__ == "__main__":
    unittest.main()
