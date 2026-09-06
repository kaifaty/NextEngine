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
import physical_sound_pouring_pilot as p
import physical_sound_pouring_pitch_adapter as a


class PitchAdapterTests(unittest.TestCase):
    def test_zero_adapter_exactly_preserves_parent(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)
        base = p.PourFlow().eval()
        model = a.from_base(base).eval()
        c = p.condition(
            {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7},
            "glass",
            "cylindrical",
            15,
            0.1,
        )
        hz = np.linspace(700, 2000, 256).astype(np.float32)
        np.testing.assert_array_equal(
            p.sample(base, c, 2718, steps=2), a.sample(model, c, hz, 2718, steps=2)
        )
        self.assertEqual(
            sum(x.numel() for x in model.parameters() if x.requires_grad), 144
        )

    def test_plane_and_exact_endpoint_objective(self):
        field = a.plane(torch.full((2, 256), 1000.0))
        self.assertEqual(tuple(field.shape), (2, 1, 256, 256))
        self.assertTrue(torch.isfinite(field).all())
        self.assertTrue((field >= 0).all() and (field <= 1).all())
        self.assertEqual(int(field[0, 0, :, 0].argmax()), 31)
        with self.assertRaises(ValueError):
            a.plane(torch.zeros(2, 256))
        target = torch.randn(2, 1, 256, 256)
        noise = torch.randn_like(target)
        self.assertLess(
            float(a.objective(target - noise, target, noise, torch.tensor([0.2, 0.8]))),
            1e-6,
        )

    def test_gradient_only_updates_adapter(self):
        torch.set_num_threads(2)
        model = a.from_base(p.PourFlow())
        output = model(
            torch.randn(1, 1, 256, 256),
            torch.tensor([0.3]),
            torch.zeros(1, 11),
            a.plane(torch.full((1, 256), 1000.0)),
        )
        output.square().mean().backward()
        self.assertGreater(float(model.pitch_adapter.weight.grad.abs().sum()), 0)
        self.assertTrue(
            all(
                v.grad is None
                for k, v in model.named_parameters()
                if not k.startswith("pitch_adapter.")
            )
        )

    def test_checkpoint_validation_and_sampling_bounds(self):
        base = p.PourFlow()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkpoint = root / "adapter.safetensors"
            a.save_file({"weight": torch.ones(16, 1, 3, 3)}, checkpoint)
            meta = {
                "format": "pour-pitch-adapter-v1",
                "parent_checkpoint_sha256": "parent",
                "checkpoint_sha256": hashlib.sha256(
                    checkpoint.read_bytes()
                ).hexdigest(),
            }
            (root / "model.json").write_text(json.dumps(meta))
            model, _ = a.load(root, {"checkpoint_sha256": "parent"}, base)
            self.assertTrue((model.pitch_adapter.weight == 1).all())
            with self.assertRaises(ValueError):
                a.load(root, {"checkpoint_sha256": "wrong"}, base)
            checkpoint.write_bytes(b"corrupt")
            with self.assertRaises(ValueError):
                a.load(root, {"checkpoint_sha256": "parent"}, base)
            (root / "model.json").write_text(" " * 100001)
            with self.assertRaises(ValueError):
                a.load(root, {"checkpoint_sha256": "parent"}, base)
        for seed, steps in [(-1, 64), (2**32, 64), (1, 0), (1, 1025)]:
            with self.assertRaises(ValueError):
                a.sample(base, np.zeros(11), np.ones(256), seed, steps)

    def test_source_free_render_and_head_identity(self):
        controls = p.condition(
            {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7},
            "glass",
            "cylindrical",
            15,
            0.1,
        )
        wave = np.random.default_rng(53).normal(0, 0.005, p.SAMPLES).astype(np.float32)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(a.compare, "load_model", return_value=(object(), {})),
                patch.object(
                    a.h,
                    "load_head",
                    return_value=(object(), {"checkpoint_sha256": "head"}),
                ),
                patch.object(
                    a,
                    "load",
                    return_value=(object(), {"head_checkpoint_sha256": "head"}),
                ) as loader,
                patch.object(
                    p, "load_source", side_effect=AssertionError("reference forbidden")
                ),
                patch.object(a.h, "predict", return_value=np.ones(256) * 1000),
                patch.object(p, "sample", return_value=None),
                patch.object(a, "sample", return_value=None),
                patch.object(p, "decode", return_value=wave),
            ):
                a.render(root, root, root, root / "out", controls, device="cpu")
                result = json.loads((root / "out/result.json").read_text())
                self.assertFalse(result["reference_audio_input"])
                self.assertFalse(result["teacher_required_at_inference"])
                self.assertEqual(
                    [r["kind"] for r in result["rows"]], ["base", "adapter"]
                )
                loader.return_value = (object(), {"head_checkpoint_sha256": "wrong"})
                with self.assertRaises(ValueError):
                    a.render(root, root, root, root / "bad", controls, device="cpu")
                self.assertFalse((root / "bad").exists())


if __name__ == "__main__":
    unittest.main()
