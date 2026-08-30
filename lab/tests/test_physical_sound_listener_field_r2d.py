from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path

import numpy as np
import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_listener_field_r2d_common as common  # noqa: E402


class PhysicalSoundListenerFieldR2DTests(unittest.TestCase):
    def test_profile_is_query_free_and_freezes_three_trainability_tasks(self) -> None:
        profile = common.profile()
        self.assertEqual(profile["representation"]["rank"], 96)
        self.assertFalse(profile["representation"]["query_audio_used"])
        self.assertFalse(profile["query_audio_available"])
        self.assertFalse(profile["method_holdout_or_shadow_available"])
        self.assertEqual(
            [task["task_id"] for task in profile["optimizer"]["tasks"]],
            [task["task_id"] for task in common.TASKS],
        )
        self.assertEqual(len(common.ONE_ROW_CACHE_INDICES), 1)
        self.assertEqual(len(common.SMALL_BLOCK_CACHE_INDICES), 8)
        self.assertEqual(len(common.FULL_PROBE_CACHE_INDICES), 28)

    def test_reconstructed_energy_matches_explicit_complex_field(self) -> None:
        basis = torch.tensor(
            [[1.0 + 0.0j, 0.0 + 0.0j], [0.0 + 0.0j, 1.0 + 0.0j]],
            dtype=torch.complex64,
        )
        mean = torch.tensor([0.2 + 0.1j, -0.1 + 0.3j], dtype=torch.complex64)
        normalized = torch.tensor(
            [[[0.5, -0.2], [0.1, 0.3]], [[-0.2, 0.4], [0.7, -0.1]]],
            dtype=torch.float32,
        )
        component_rms = torch.tensor([2.0, 0.5], dtype=torch.float32)
        projection_complex = basis @ torch.conj(mean)
        projection = torch.stack(
            (projection_complex.real, projection_complex.imag), dim=-1
        )
        measured = common.reconstructed_energy(
            normalized,
            component_rms,
            torch.sum(torch.abs(mean) ** 2),
            projection,
        )
        coefficients = common.as_complex(normalized) * component_rms[None, :]
        explicit = mean[None, :] + coefficients @ basis
        expected = torch.sum(torch.abs(explicit) ** 2, dim=1)
        torch.testing.assert_close(measured, expected)

    def test_exact_coefficients_zero_every_objective_component(self) -> None:
        target = torch.tensor(
            [[[0.3, -0.1], [0.7, 0.2]], [[-0.2, 0.5], [0.1, -0.4]]],
            dtype=torch.float32,
        )
        component_rms = torch.tensor([2.0, 0.5])
        projection = torch.zeros((2, 2))
        objective, parts = common.coefficient_objective(
            target.clone(), target, component_rms, torch.tensor(1.0), projection
        )
        self.assertEqual(float(objective), 0.0)
        self.assertTrue(all(float(value) == 0.0 for value in parts.values()))

    def test_synthetic_direct_table_converges_without_clipping(self) -> None:
        torch.manual_seed(common.SEED)
        target = torch.randn((4, 6, 2), dtype=torch.float32) * 0.4
        component_rms = torch.linspace(0.5, 1.5, 6)
        projection = torch.zeros((6, 2))
        mean_energy = torch.tensor(5.0)
        model = common.CoefficientTable(4, rank=6)
        optimizer = torch.optim.AdamW(
            model.parameters(),
            lr=common.LEARNING_RATE,
            betas=common.ADAM_BETAS,
            eps=common.ADAM_EPSILON,
            weight_decay=common.WEIGHT_DECAY,
        )
        clipped = 0
        for _ in range(400):
            objective, _ = common.coefficient_objective(
                model(), target, component_rms, mean_energy, projection
            )
            optimizer.zero_grad(set_to_none=True)
            objective.backward()
            norm = torch.nn.utils.clip_grad_norm_(
                model.parameters(), common.GRADIENT_CLIP_NORM
            )
            clipped += int(float(norm) > common.GRADIENT_CLIP_NORM)
            optimizer.step()
        final, parts = common.coefficient_objective(
            model(), target, component_rms, mean_energy, projection
        )
        self.assertEqual(clipped, 0)
        self.assertLess(float(final.detach()), 1.0e-5)
        self.assertLess(
            math.sqrt(float(parts["whitened_coefficient_mse"].detach())), 0.005
        )

    def test_eigenvector_phase_is_canonical(self) -> None:
        vectors = torch.tensor(
            [[1.0j, -2.0j], [2.0 + 2.0j, 1.0 + 0.0j]], dtype=torch.complex64
        )
        canonical = common.canonicalize_eigenvectors(vectors)
        for column in range(canonical.shape[1]):
            values = canonical[:, column]
            pivot = values[torch.argmax(torch.abs(values))]
            self.assertGreater(float(pivot.real), 0.0)
            self.assertAlmostEqual(float(pivot.imag), 0.0, places=6)

    def test_gate_requires_oracle_proximity_and_trivial_control_improvement(self) -> None:
        endpoints = common.r2c.PRIMARY_ENDPOINTS
        aggregate = lambda value: {endpoint: value for endpoint in endpoints}
        training = {
            "optimizer_steps": 100,
            "gradient_clip_count": 0,
            "zero_predictor_metrics": {"objective": 1.0},
            "final_metrics": {
                "objective": 1.0e-4,
                "raw_coefficient_nmse": 1.0e-5,
                "whitened_coefficient_mse": 1.0e-6,
                "mean_absolute_log_energy_error": 1.0e-4,
            },
        }
        trained = {"aggregate": aggregate(1.01)}
        oracle = {"aggregate": aggregate(1.0)}
        zero = {"aggregate": aggregate(10.0)}
        mean = {"aggregate": aggregate(5.0)}
        passed, evidence = common.task_metrics_pass(
            training, trained, oracle, zero, mean
        )
        self.assertTrue(passed)
        self.assertTrue(all(evidence["checks"].values()))
        trained["aggregate"][endpoints[0]] = 5.1
        passed, _ = common.task_metrics_pass(training, trained, oracle, zero, mean)
        self.assertFalse(passed)


if __name__ == "__main__":
    unittest.main()
