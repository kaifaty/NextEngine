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
