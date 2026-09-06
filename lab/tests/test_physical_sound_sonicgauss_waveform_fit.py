import sys
import unittest
from pathlib import Path
from types import SimpleNamespace

import numpy as np
import torch
from torch import nn

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_codec_probe as codec
import physical_sound_sonicgauss_waveform_fit as fit


class ToyTransformer(nn.Module):
    def forward(
        self,
        hidden_states,
        timestep,
        encoder_hidden_states,
        pooled_projections,
        **kwargs,
    ):
        return (
            hidden_states * 0.1
            + encoder_hidden_states.mean() * timestep
            + pooled_projections.mean(),
        )


class WaveformFitTests(unittest.TestCase):
    def test_independent_evaluation_detects_level_and_attack_changes(self):
        wave = codec.controls()["ring"].mean(0)
        unchanged = fit.audio_metrics(wave, wave.copy())
        self.assertEqual(unchanged, {"spectrum": 0.0, "envelope": 0.0, "level": 0.0})
        changed = fit.audio_metrics(wave, wave * 0.5)
        self.assertAlmostEqual(changed["level"], np.log(2), places=4)
        self.assertGreater(changed["envelope"], 0.49)
        shifted = fit.audio_metrics(wave, np.roll(wave, 4410))
        self.assertGreater(shifted["envelope"], 0.1)

    def test_checkpoint_preserves_all_step_outputs_and_gradients(self):
        from diffusers import FlowMatchEulerDiscreteScheduler

        model = SimpleNamespace(
            audio_seq_len=2,
            transformer=ToyTransformer(),
            fc=nn.Identity(),
            noise_scheduler=FlowMatchEulerDiscreteScheduler(),
            duration_emebdder=lambda x: torch.zeros(1, 1, 4),
        )
        torch.manual_seed(9)
        fused = torch.randn(1, 3, 4, requires_grad=True)
        noise = torch.randn(1, 2, 64)
        first = fit.generate(model, fused, noise)
        grad = torch.autograd.grad(first.square().mean(), fused)[0]
        second = fit.generate(model, fused, noise, use_checkpoint=True)
        other = torch.autograd.grad(second.square().mean(), fused)[0]
        torch.testing.assert_close(first, second, rtol=0, atol=0)
        torch.testing.assert_close(grad, other, rtol=0, atol=0)
        self.assertGreater(grad.norm().item(), 0)

    def test_objective_identity_level_shift_and_finite_gradient(self):
        wave = torch.from_numpy(codec.controls()["ring"])[None]
        value, components = fit.waveform_loss(wave, wave.clone())
        self.assertEqual(value.item(), 0)
        prediction = (wave * 0.5).requires_grad_(True)
        changed, components = fit.waveform_loss(wave, prediction)
        self.assertGreater(changed.item(), 0.5)
        self.assertAlmostEqual(components["level"].item(), np.log(2), places=4)
        changed.backward()
        self.assertTrue(torch.isfinite(prediction.grad).all())
        self.assertGreater(prediction.grad.norm().item(), 0)
        shifted = torch.roll(wave, 4410, -1)
        _, metrics = fit.waveform_loss(wave, shifted)
        self.assertGreater(metrics["envelope"].item(), 0.1)

    def test_ring_erasure_not_hidden_by_infrasound(self):
        t = np.arange(131418) / 44100
        low = np.stack([0.1 * np.sin(2 * np.pi * 3 * t)] * 2).astype(np.float32)
        reference = torch.from_numpy(low + codec.controls()["quiet-ring"])[None]
        erased = torch.from_numpy(low)[None]
        loss, components = fit.waveform_loss(reference, erased)
        self.assertGreater(components["spectrum"].item(), 0.8)
        self.assertGreater(loss.item(), 1)


if __name__ == "__main__":
    unittest.main()
