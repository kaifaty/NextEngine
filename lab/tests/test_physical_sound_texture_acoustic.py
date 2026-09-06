from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch
from scipy.signal import resample_poly

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_acoustic as acoustic


class AcousticTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)

    def test_shared_band_matches_scipy_and_has_gradient(self):
        wave = torch.tensor(
            np.random.default_rng(1).normal(size=(2, 2, 4097)), requires_grad=True
        )
        actual = acoustic.shared_band(wave)
        expected = resample_poly(wave.detach().numpy().mean(1), 1, 2, axis=1)
        np.testing.assert_allclose(actual.detach().numpy(), expected, atol=1e-12)
        actual.square().mean().backward()
        self.assertGreater(float(wave.grad.norm()), 0)

    def test_endpoint_recovers_clean_target_for_true_velocity(self):
        noise, target = torch.randn(2, 64, 32), torch.randn(2, 64, 32)
        t = torch.tensor([0.0, 0.75])
        mixed = noise * (1 - t[:, None, None]) + target * t[:, None, None]
        torch.testing.assert_close(acoustic.endpoint(mixed, target - noise, t), target)

    def test_differentiable_sampler_matches_inference_and_backpropagates(self):
        model = acoustic.flow.TextureFlow(7).eval()
        torch.nn.init.normal_(model.output.weight, std=0.01)
        for frames in (32, 68):
            physical = acoustic.surface.features(
                "Glass", 0.4, 0.38, np.full(frames, 40), np.full(frames, 0.5)
            )
            expected = acoustic.flow.sample_features(model, physical[None], 607, frames)
            actual = acoustic.sampled_latent(model, physical, 607)
            torch.testing.assert_close(actual, expected, rtol=0, atol=0)
            model.zero_grad()
            actual.square().mean().backward()
            self.assertTrue(torch.isfinite(model.output.weight.grad).all())
            self.assertGreater(float(model.output.weight.grad.norm()), 0)
        for seed in (-1, 2**32, 3.0):
            with self.assertRaises(ValueError):
                acoustic.sampled_latent(model, physical, seed)

    def test_objective_identity_is_explicit(self):
        self.assertEqual(acoustic.candidate_arm(acoustic.FORMAT), "acoustic")
        self.assertEqual(acoustic.candidate_arm(acoustic.SAMPLED_FORMAT), "sampled")
        self.assertEqual(
            acoustic.candidate_arm(acoustic.SEPARATED_FORMAT), "level_shape"
        )
        with self.assertRaises(ValueError):
            acoustic.candidate_arm("unknown")

    def test_loss_detects_gain_without_normalizing_it_away(self):
        wave = torch.randn(1, 2, 65536) * 0.01
        equal = acoustic.acoustic_parts(wave, wave)
        self.assertEqual(float(sum(equal.values())), 0)
        amplified = (wave * 2).requires_grad_()
        loss = acoustic.acoustic_parts(amplified, wave)
        for value in loss.values():
            self.assertAlmostEqual(float(value.detach()), np.log(4), places=5)
        sum(loss.values()).backward()
        self.assertTrue(torch.isfinite(amplified.grad).all())
        self.assertGreater(float(amplified.grad.norm()), 0)

    def test_rejects_wrong_audio_and_incomplete_train(self):
        with self.assertRaises(ValueError):
            acoustic.training_records([])
        with self.assertRaises(ValueError):
            acoustic.acoustic_parts(torch.zeros(1, 2, 1000), torch.zeros(1, 2, 1000))
        with self.assertRaises(ValueError):
            acoustic.acoustic_parts(
                torch.full((1, 2, 65536), float("nan")), torch.zeros(1, 2, 65536)
            )

    def test_separated_shape_ignores_gain_but_envelope_keeps_it(self):
        wave = torch.tensor(np.random.default_rng(17).normal(0, 0.01, (1, 2, 65536)))
        loss = acoustic.acoustic_parts(wave * 2, wave, separate=True)
        self.assertLess(float(loss["spectrum"]), 1e-12)
        self.assertAlmostEqual(float(loss["envelope"]), np.log(4), places=10)
        filtered = torch.nn.functional.avg_pool1d(wave, 9, stride=1, padding=4)
        gain = torch.tensor(1.5, dtype=wave.dtype, requires_grad=True)
        changed = acoustic.acoustic_parts(filtered * gain, wave, separate=True)
        self.assertGreater(float(changed["spectrum"].detach()), 0.1)
        derivative = torch.autograd.grad(changed["spectrum"], gain)[0]
        self.assertLess(abs(float(derivative)), 1e-10)

    def test_loader_rejects_training_or_weight_identity_before_device_use(self):
        parent = {
            "codec_sha256": acoustic.flow.CODEC_SHA,
            "input_gain": acoustic.flow.GAIN,
            "table_sha256": acoustic.surface.TABLE_SHA,
            "descriptor": {
                "sha256": "6d36e47c055512eef37a9b182c19b31f2db246c0c03d33a00c21822a4672a8e0"
            },
        }
        ids = [
            r["id"]
            for r in acoustic.surface.source.conditions(
                True, {t: str(t) for t in acoustic.surface.SURFACES}
            )
            if acoustic.surface.role(r) == "train"
        ]
        meta = {
            "format": acoustic.FORMAT,
            "parent": parent,
            "steps_per_arm": acoustic.STEPS,
            "acoustic_weight": acoustic.WEIGHT,
            "training_ids": ids,
            "fm_only": {"sha256": "wrong"},
        }
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "fm_only.safetensors").write_bytes(b"invalid weights")
            for training in (ids[:-1], ids):
                (directory / "model.json").write_text(
                    json.dumps({**meta, "training_ids": training})
                )
                with self.assertRaises(ValueError):
                    acoustic.load_candidates(directory, parent)


if __name__ == "__main__":
    unittest.main()
