from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import pandas as pd
import torch
from safetensors.torch import save_file

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_pilot as pouring


class PouringTests(unittest.TestCase):
    def test_measurements_are_bounded_literals(self):
        valid = "{'net_height':10,'diameter_top':7,'diameter_bottom':7}"
        self.assertEqual(pouring.parse_measurements(valid)["net_height"], 10)
        for value in ["{}", "x" * 2049, "__import__('os').system('false')"]:
            with self.assertRaises(ValueError):
                pouring.parse_measurements(value)
        with self.assertRaises(TypeError):
            pouring.parse_measurements("[]")

    def test_condition_shape_and_limits(self):
        dims = {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7}
        c = pouring.condition(dims, "glass", "cylindrical", 15, 0.2)
        self.assertEqual(c.shape, (11,))
        np.testing.assert_allclose(c[:5], [0.5, 0.35, 0.35, 0.5, 0.2])
        for material, duration, progress in [
            ("steel", 15, 0),
            ("glass", float("nan"), 0),
            ("glass", 15, 1.1),
        ]:
            with self.assertRaises(ValueError):
                pouring.condition(dims, material, "cylindrical", duration, progress)

    def test_whole_container_exclusion_and_annotation_filter(self):
        common = {
            "flow_rate_appx": "constant",
            "liquid": "water_normal",
            "clean": "yes",
            "material": "glass",
            "shape": "cylindrical",
            "measurements": "{'net_height':10,'diameter_top':7,'diameter_bottom':7}",
        }
        rows = []
        for i, container in enumerate(["container_6", "container_18", "container_30"]):
            rows.append(
                {
                    **common,
                    "container_id": container,
                    "video_id": f"VID_20240101_00000{i}",
                    "item_id": f"VID_20240101_00000{i}_1.0_10.0",
                }
            )
        frame = pd.DataFrame(rows)
        self.assertEqual(
            pouring.eligible(frame).role.tolist(),
            ["train", "unseen_container", "unseen_container"],
        )
        frame.loc[0, "flow_rate_appx"] = "TODO"
        self.assertEqual(len(pouring.eligible(frame)), 2)
        with self.assertRaises(ValueError):
            pouring.eligible(pd.concat([frame, frame]))

    def test_transform_identity_and_decoder(self):
        from scipy.signal import istft

        rng = np.random.default_rng(314)
        wave = rng.normal(0, 0.02, pouring.SAMPLES).astype(np.float32)
        spec = pouring.transform(wave)
        restored = istft(
            spec,
            fs=pouring.RATE,
            nperseg=pouring.FFT,
            noverlap=pouring.FFT - pouring.HOP,
        )[1]
        np.testing.assert_allclose(restored, wave, atol=3e-8)
        encoded = pouring.encode(wave)
        self.assertEqual(encoded.shape, (256, 256))
        a = pouring.decode(encoded, 314, 2)
        np.testing.assert_array_equal(a, pouring.decode(encoded, 314, 2))
        self.assertEqual(len(a), pouring.SAMPLES)
        self.assertTrue(np.isfinite(a).all())
        with self.assertRaises(ValueError):
            pouring.decode(np.full((256, 256), np.nan))

    def test_network_shapes_and_condition_gradient(self):
        torch.set_num_threads(2)
        model = pouring.PourFlow()
        x = torch.randn(1, 1, 256, 256)
        c = torch.ones(1, 11, requires_grad=True)
        result = model(x, torch.tensor([0.5]), c)
        self.assertEqual(result.shape, x.shape)
        result.square().mean().backward()
        self.assertTrue(torch.isfinite(c.grad).all())
        self.assertGreater(float(c.grad.abs().sum()), 0)

    def test_standalone_has_no_audio_input_and_checks_identity(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            checkpoint = root / "model.safetensors"
            save_file(pouring.PourFlow().state_dict(), checkpoint)
            meta = {
                "format": "pour-flow-v1",
                "checkpoint_sha256": hashlib.sha256(
                    checkpoint.read_bytes()
                ).hexdigest(),
            }
            (root / "model.json").write_text(json.dumps(meta))
            with (
                patch.object(
                    pouring.sf,
                    "read",
                    side_effect=AssertionError("reference forbidden"),
                ),
                patch.object(
                    pouring,
                    "sample",
                    return_value=np.full((256, 256), -1.5, dtype=np.float32),
                ),
            ):
                pouring.render(
                    root,
                    root / "generated",
                    device="cpu",
                    height=12,
                    progress=0.3,
                    playback_gain=2,
                )
            result = json.loads((root / "generated/result.json").read_text())
            self.assertFalse(result["reference_audio_input"])
            self.assertEqual(result["request"]["dimensions_cm"]["net_height"], 12)
            self.assertEqual(result["requested_audition_gain"], 2)
            meta["checkpoint_sha256"] = "0" * 64
            (root / "model.json").write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                pouring.render(root, root / "bad", device="cpu")
            with self.assertRaises(ValueError):
                pouring.render(
                    root, root / "bad-gain", device="cpu", playback_gain=float("nan")
                )

    def test_power_envelope_loss_identity_gain_variation_and_gradients(self):
        target = torch.full((2, 1, 256, 256), -1.0)
        self.assertEqual(pouring.power_envelope_loss(target, target).tolist(), [0, 0])
        changed = (target + 0.2).requires_grad_()
        loss = pouring.power_envelope_loss(changed, target)
        self.assertTrue(torch.all(loss > 0))
        loss.mean().backward()
        self.assertTrue(torch.isfinite(changed.grad).all())
        self.assertGreater(float(changed.grad.abs().sum()), 0)
        variable = target.clone()
        variable[:, :, :, ::2] += 0.3
        variable[:, :, :, 1::2] -= 0.3
        self.assertTrue(torch.all(pouring.power_envelope_loss(variable, target) > 0))
        self.assertTrue(
            torch.isfinite(pouring.power_envelope_loss(target * 100, target)).all()
        )

    def test_metrics_separate_level_and_shape(self):
        wave = (
            np.random.default_rng(1).normal(0, 0.1, pouring.SAMPLES).astype(np.float32)
        )
        result = pouring.metrics(wave * 2, wave)
        self.assertAlmostEqual(result["level_error_db"], 6.0206, places=3)
        self.assertLess(result["spectrum_shape_rmse_db"], 0.01)

    def test_onset_sampling_preserves_rng_and_interior_support(self):
        a = np.random.default_rng(53)
        b = np.random.default_rng(53)
        for i in range(100):
            uniform = pouring.patch_start(a, 800, "uniform", i)
            onset = pouring.patch_start(b, 800, "onset-balanced", i)
            self.assertEqual(onset, 0 if i % 2 == 0 else uniform)
            self.assertTrue(0 <= uniform <= 800 - pouring.FRAMES)
            self.assertEqual(a.integers(100), b.integers(100))
        self.assertEqual(pouring.patch_start(a, 100, "onset-balanced", 1), 0)
        with self.assertRaises(ValueError):
            pouring.patch_start(a, 0, "uniform", 0)
        with self.assertRaises(ValueError):
            pouring.patch_start(a, 800, "unknown", 0)

    def test_auxiliary_statistic_does_not_preserve_flow_optimum(self):
        a = torch.tensor([[-0.5, -1.5], [-0.5, -1.5]])
        target = torch.stack([a, a.flip(-1)])[:, None]
        mean = target.mean(0, keepdim=True).requires_grad_()
        velocity_loss = 4 * (mean - target).square().mean()
        gradient = torch.autograd.grad(velocity_loss, mean, retain_graph=True)[0]
        self.assertEqual(float(gradient.abs().max()), 0)
        loss = (
            velocity_loss
            + 0.25
            * 0.5**2
            * pouring.power_envelope_loss(mean.expand_as(target), target).mean()
        )
        self.assertGreater(float(torch.autograd.grad(loss, mean)[0].abs().max()), 0)


if __name__ == "__main__":
    unittest.main()
