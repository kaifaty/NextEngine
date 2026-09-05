import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_latent_flow as f


class LatentFlowTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)

    def test_network_shape_and_gradients(self):
        model = f.LatentFlow()
        wave = torch.randn(2, 64, 88)
        controls = torch.tensor(np.stack([f.c.v.phase.d.controls_for()] * 2))
        output = model(wave, torch.rand(2), controls)
        self.assertEqual(output.shape, wave.shape)
        output.square().mean().backward()
        self.assertTrue(
            all(
                p.grad is not None and torch.isfinite(p.grad).all()
                for p in model.parameters()
            )
        )

    def test_normalization_includes_posterior_variance(self):
        center, scale = f.normalization(
            torch.full((3, 64, 88), 100.0), torch.full((3, 64, 88), 2.0)
        )
        self.assertTrue(torch.all(center == 100))
        self.assertTrue(torch.all(scale == 2))
        with self.assertRaises(ValueError):
            f.normalization(torch.zeros(3, 64, 88), -torch.ones(3, 64, 88))

    def test_gaussian_skip_has_finite_exact_endpoints(self):
        x = torch.randn(3, 64, 88)
        t = torch.tensor([0.0, 0.5, 1.0])
        expected = torch.stack([-x[0], torch.zeros_like(x[1]), x[2]])
        self.assertTrue(torch.equal(f.gaussian_velocity(x, t), expected))
        model = f.LatentFlow(gaussian_skip=True)
        torch.nn.init.zeros_(model.output.weight)
        torch.nn.init.zeros_(model.output.bias)
        controls = torch.tensor(np.stack([f.c.v.phase.d.controls_for()] * 3))
        self.assertTrue(torch.equal(model(x, t, controls), expected))
        self.assertTrue(
            torch.isfinite(
                f.gaussian_velocity(x, torch.tensor([1e-6, 0.37, 0.99999]))
            ).all()
        )

    def test_source_free_seeded_sampling(self):
        model = f.LatentFlow().eval()
        torch.nn.init.zeros_(model.output.weight)
        torch.nn.init.zeros_(model.output.bias)
        controls = f.c.v.phase.d.controls_for()
        with (
            patch.object(
                f.c, "input_wave", side_effect=AssertionError("audio input forbidden")
            ),
            patch.object(
                f.c.v.p,
                "load_source",
                side_effect=AssertionError("source access forbidden"),
            ),
        ):
            first = f.sample(model, controls, 314)
            self.assertTrue(torch.equal(first, f.sample(model, controls, 314)))
            self.assertFalse(torch.equal(first, f.sample(model, controls, 2718)))
        with self.assertRaises(ValueError):
            f.sample(model, controls, -1)
        with self.assertRaises(ValueError):
            f.sample(model, np.zeros(11), 314)

    def test_checkpoint_integrity(self):
        model = f.LatentFlow()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "model.safetensors"
            f.save_file(model.state_dict(), path)
            (root / "model.json").write_text(
                json.dumps(
                    {
                        "format": "pour-latent-flow-v1",
                        "codec_sha256": f.c.CODEC_SHA,
                        "checkpoint_sha256": hashlib.sha256(
                            path.read_bytes()
                        ).hexdigest(),
                    }
                )
            )
            loaded, _ = f.load(root, "cpu")
            self.assertFalse(loaded.gaussian_skip)
            self.assertTrue(
                all(
                    torch.equal(x, loaded.state_dict()[k])
                    for k, x in model.state_dict().items()
                )
            )
            meta_path = root / "model.json"
            meta = json.loads(meta_path.read_text())
            meta["format"] = "pour-latent-flow-gaussian-v1"
            meta_path.write_text(json.dumps(meta))
            gaussian, _ = f.load(root, "cpu")
            self.assertTrue(gaussian.gaussian_skip)
            x = torch.randn(1, 64, 88)
            t = torch.tensor([0.25])
            controls = torch.tensor(f.c.v.phase.d.controls_for()[None])
            with torch.no_grad():
                torch.testing.assert_close(
                    gaussian(x, t, controls),
                    loaded(x, t, controls) + f.gaussian_velocity(x, t),
                )
            path.write_bytes(b"broken")
            with self.assertRaises(ValueError):
                f.load(root, "cpu")

    def test_partial_path_fixed_time_grid(self):
        class Constant:
            def __call__(self, x, time, controls):
                return torch.ones_like(x)

        x = torch.zeros(1, 64, 88)
        for start in (0, 32, 56, 64):
            self.assertTrue(
                torch.all(f.integrate(Constant(), x, None, start) == 1 - start / 64)
            )
        with self.assertRaises(ValueError):
            f.integrate(Constant(), x, None, 65)

    def test_single_crop_scope_preserves_original_cache(self):
        data = {
            "mean": torch.randn(3, 64, 88),
            "std": torch.ones(3, 64, 88),
            "controls": torch.zeros(3, 11),
        }
        rows = [
            {"phase": "first", "item_id": "one"},
            {"phase": "middle", "item_id": "one"},
            {"phase": "first", "item_id": "two"},
        ]
        selected, fitted = f.training_view(data, rows, True)
        self.assertEqual(len(selected["mean"]), 1)
        self.assertEqual(fitted, rows[:1])
        self.assertEqual(len(data["mean"]), 3)
        unchanged, full = f.training_view(data, rows, False)
        self.assertIs(unchanged, data)
        self.assertIs(full, rows)
        with self.assertRaises(ValueError):
            f.training_view(data, rows[1:], True)

    def test_training_phase_bounds_and_parent_output_guard(self):
        with self.assertRaisesRegex(ValueError, "unknown posterior"):
            f.fit(Path("absent"), Path("unused"), training_path="wrong")
        for steps in (0, -1, 20001, 1.5):
            with self.assertRaisesRegex(ValueError, "20000 steps"):
                f.fit(Path("absent"), Path("unused"), steps=steps)
        with self.assertRaisesRegex(ValueError, "preserve parent"):
            f.fit(
                Path("absent"), Path("unused"), parent=Path("parent"), zero_output=True
            )

    def test_warm_start_rejects_scope_and_normalization_before_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "posterior.safetensors"
            f.save_file(
                {
                    "mean": torch.zeros(1, 64, 88),
                    "std": torch.ones(1, 64, 88),
                    "controls": torch.tensor(f.c.v.phase.d.controls_for()[None]),
                },
                path,
            )
            cache_path = root / "cache.json"
            cache_path.write_text(
                json.dumps(
                    {
                        "format": "pour-oobleck-cache-v1",
                        "codec_sha256": f.c.CODEC_SHA,
                        "posterior_sha256": hashlib.sha256(
                            path.read_bytes()
                        ).hexdigest(),
                        "train_ids": ["one"],
                        "rows": [
                            {
                                "item_id": "one",
                                "container_id": "train",
                                "phase": "first",
                            }
                        ],
                    }
                )
            )
            for defect in ("cache", "ids", "scope", "center", "scale", "mode", "path"):
                with self.subTest(defect=defect):
                    model = f.LatentFlow(gaussian_skip=defect == "mode")
                    meta = {
                        "cache_sha256": hashlib.sha256(
                            cache_path.read_bytes()
                        ).hexdigest(),
                        "train_ids": ["one"],
                        "single_crop_control": False,
                    }
                    if defect == "cache":
                        meta["cache_sha256"] = "wrong"
                    elif defect == "ids":
                        meta["train_ids"] = ["other"]
                    elif defect == "scope":
                        meta["single_crop_control"] = True
                    elif defect == "center":
                        model.center.fill_(1)
                    elif defect == "scale":
                        model.scale.fill_(2)
                    elif defect == "path":
                        meta["training_path"] = "affine"
                    output = root / "must-not-exist"
                    with (
                        patch.object(f, "load", return_value=(model, meta)),
                        self.assertRaisesRegex(ValueError, "parent/cache/scope"),
                    ):
                        f.fit(root, output, device="cpu", parent=root / "parent")
                    self.assertFalse(output.exists())

    def test_posterior_paths_endpoints_derivative_and_legacy_exactness(self):
        mean = torch.randn(3, 64, 88, dtype=torch.float64)
        std = torch.rand_like(mean)
        center, scale = f.normalization(mean, std)
        posterior_noise, noise = torch.randn_like(mean), torch.randn_like(mean)
        t = torch.tensor([0.0, 0.37, 1.0], dtype=torch.float64)
        before = torch.get_rng_state()
        x, velocity = f.posterior_path(
            mean, std, center, scale, posterior_noise, noise, t, "independent"
        )
        target = (mean + std * posterior_noise - center) / scale
        self.assertTrue(
            torch.equal(x, noise * (1 - t[:, None, None]) + target * t[:, None, None])
        )
        self.assertTrue(torch.equal(velocity, target - noise))
        affine, derivative = f.posterior_path(
            mean, std, center, scale, posterior_noise, noise, t, "affine"
        )
        torch.testing.assert_close(affine[0], noise[0])
        torch.testing.assert_close(
            affine[2], ((mean - center + std * noise) / scale)[2]
        )
        later, _ = f.posterior_path(
            mean, std, center, scale, posterior_noise, noise, t + 1e-5, "affine"
        )
        torch.testing.assert_close((later - affine) / 1e-5, derivative)
        self.assertTrue(torch.equal(before, torch.get_rng_state()))
        with self.assertRaisesRegex(ValueError, "unknown posterior"):
            f.posterior_path(
                mean, std, center, scale, posterior_noise, noise, t, "wrong"
            )


if __name__ == "__main__":
    unittest.main()
