import sys
import unittest
from pathlib import Path
from unittest import mock

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_condition_probe as probe


class ConditionProbeTests(unittest.TestCase):
    def test_compare_exact_and_zero_candidate(self):
        a = np.arange(6, dtype=float).reshape(2, 3)
        exact = probe.compare(a, a.copy())
        self.assertTrue(exact["exact"])
        self.assertEqual(exact["relative_rms"], 0)
        zero = probe.compare(a, np.zeros_like(a))
        self.assertEqual(zero["relative_rms"], 1)
        self.assertIsNone(zero["cosine"])
        for bad in [np.zeros(3), np.full_like(a, np.nan)]:
            with self.assertRaises(ValueError):
                probe.compare(a, bad)

    def test_interventions_leave_unrelated_inputs_and_source_unchanged(self):
        inputs = {
            "means": torch.tensor([[0.2, 0.1, 0.3], [0.7, 0.8, 0.9]]),
            "scales": torch.ones(2, 3),
            "features_dc": torch.ones(2, 3),
            "features_rest": torch.ones(2, 3, 3),
        }
        position = torch.tensor([0.7, 0.8, 0.9])
        saved = {key: value.clone() for key, value in inputs.items()}
        narrowed = probe.counterfactual(inputs, position, "centers-contract-x")
        torch.testing.assert_close(narrowed["means"][:, 0], torch.tensor([0.4, 0.7]))
        torch.testing.assert_close(narrowed["means"][1], position)
        torch.testing.assert_close(narrowed["means"][:, 1:], inputs["means"][:, 1:])
        neutral = probe.counterfactual(inputs, position, "neutral-appearance")
        self.assertEqual(float(neutral["features_dc"].abs().sum()), 0)
        self.assertEqual(float(neutral["features_rest"].abs().sum()), 0)
        for key, value in inputs.items():
            torch.testing.assert_close(value, saved[key], rtol=0, atol=0)
            if key != "means":
                torch.testing.assert_close(narrowed[key], value, rtol=0, atol=0)
            if key not in {"features_dc", "features_rest"}:
                torch.testing.assert_close(neutral[key], value, rtol=0, atol=0)

    def test_cached_encoder_owns_features_and_restores_both_rng_states(self):
        features = torch.ones(1, 2, 3)
        cpu = torch.get_rng_state()
        fake_cuda = torch.tensor([1, 2], dtype=torch.uint8)
        cached = probe.CachedGaussian(features, (cpu, fake_cuda))
        features.zero_()
        with mock.patch.object(torch.cuda, "set_rng_state") as setter:
            output = cached({}, "cuda")
            setter.assert_called_once_with(fake_cuda)
        torch.testing.assert_close(output, torch.ones(1, 2, 3))
        self.assertTrue(torch.equal(torch.get_rng_state(), cpu))

    def test_fusion_ablation_is_after_actual_fusion_and_preserves_inputs(self):
        class Fusion(torch.nn.Module):
            def forward(self, gaussian, position):
                return gaussian + position.unsqueeze(1)

        gaussian = torch.ones(1, 2, 3)
        position = torch.ones(1, 3) * 2
        zero_position = probe.FusionProbe(Fusion(), "zero-position")
        torch.testing.assert_close(zero_position(gaussian, position), gaussian)
        zero_all = probe.FusionProbe(Fusion(), "zero-conditioning")
        self.assertEqual(float(zero_all(gaussian, position).sum()), 0)
        torch.testing.assert_close(position, torch.ones(1, 3) * 2)
        self.assertTrue(np.array_equal(zero_all.captured["position"], position.numpy()))

    def test_single_key_structure_probe_does_not_mutate_original(self):
        class Fusion(torch.nn.Module):
            def __init__(self):
                super().__init__()
                self.norm1 = torch.nn.LayerNorm(8)
                self.cross_attn = torch.nn.MultiheadAttention(8, 2, batch_first=True)

            def forward(self, gaussian, position):
                return (
                    gaussian
                    + self.cross_attn(
                        self.norm1(gaussian), position[:, None], position[:, None]
                    )[0]
                )

        torch.manual_seed(0)
        model = Fusion().eval().requires_grad_(False)
        saved = {k: v.clone() for k, v in model.state_dict().items()}
        result = probe.attention_structure(
            model, torch.randn(1, 4, 8).numpy(), torch.randn(1, 8).numpy()
        )
        self.assertEqual(result["query_key_value_gradient_norms"][:2], [0, 0])
        self.assertGreater(result["query_key_value_gradient_norms"][2], 0)
        self.assertTrue(result["attention_output_unchanged_when_queries_zeroed"])
        for key, value in model.state_dict().items():
            torch.testing.assert_close(value, saved[key], rtol=0, atol=0)
        self.assertTrue(
            all(not x.requires_grad and x.grad is None for x in model.parameters())
        )


if __name__ == "__main__":
    unittest.main()
