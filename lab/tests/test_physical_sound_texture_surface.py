from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np
import torch
from safetensors.torch import save_file

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_surface as surface


class SurfaceTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)

    def test_split_excludes_every_new_surface_and_middle_speed(self):
        rows = surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        training = [r for r in rows if surface.role(r) == "train"]
        self.assertEqual(len(training), 48)
        self.assertEqual({r["texture_id"] for r in training}, set(surface.TRAIN))
        self.assertTrue(
            all(r["repeat"] == 0 and r["commanded_speed_mm_s"] != 40 for r in training)
        )
        self.assertEqual(sum(surface.role(r) == "unseen_surface" for r in rows), 60)
        self.assertTrue(set(surface.HELD).isdisjoint(surface.source.TEXTURES))

    def test_descriptor_roundtrip_and_invalid_requests(self):
        physical = surface.features(
            "Glass", 0.4, 0.38, np.full(32, 40), np.full(32, 0.5)
        )
        surface.validate_features(physical)
        for bad in (physical * np.nan, physical[:5], np.ones((7, 32))):
            with self.assertRaises(ValueError):
                surface.validate_features(bad)
        varied = physical.copy()
        varied[5, -1] += 0.1
        with self.assertRaises(ValueError):
            surface.validate_features(varied)
        for args in (("Glass", 0.3, 0.4), ("Glass", 2.1, 0.3), ("Stone", 0.4, 0.3)):
            with self.assertRaises(ValueError):
                surface.features(*args, np.full(32, 40), np.full(32, 0.5))

    def test_warm_start_preserves_parent_before_descriptor_fit(self):
        parent = surface.flow.TextureFlow()
        torch.nn.init.normal_(parent.output.weight, std=0.01)
        model = surface.expand(parent)
        physical = surface.features(
            "Wood", 0.7, 0.65, np.full(32, 40), np.full(32, 0.5)
        )
        controls = torch.tensor(physical[None])
        x, t = torch.randn(1, 64, 32), torch.tensor([0.3])
        torch.testing.assert_close(
            parent(x, t, controls[:, :5]), model(x, t, controls), rtol=0, atol=0
        )
        self.assertEqual(int(torch.count_nonzero(model.context[0].weight[:, 5:7])), 0)
        for dim in (0, 6, 8):
            with self.assertRaises(ValueError):
                surface.flow.TextureFlow(dim)

    def test_source_free_generation_and_category_ablation(self):
        model = surface.expand(surface.flow.TextureFlow())
        torch.nn.init.normal_(model.output.weight, std=0.01)
        torch.nn.init.normal_(model.context[0].weight[:, 5:7], std=0.1)
        vae = SimpleNamespace(
            decode=lambda z: SimpleNamespace(
                sample=z[:, :2].repeat_interleave(surface.event.HOP, dim=2) * 0.001
            )
        )
        a = surface.features("Glass", 0.4, 0.38, np.full(32, 40), np.full(32, 0.5))
        b = surface.features("Glass", 0.5, 0.48, np.full(32, 40), np.full(32, 0.5))
        with (
            patch.object(
                surface.event.sf, "read", side_effect=AssertionError("no audio")
            ),
            patch.object(
                surface.np, "genfromtxt", side_effect=AssertionError("no sensors")
            ),
            patch.object(surface, "load_data", side_effect=AssertionError("no corpus")),
        ):
            first = surface.generate(model, vae, a, 314)
            repeated = surface.generate(model, vae, a, 314)
            different = surface.generate(model, vae, b, 314)
            off_a = surface.generate(model, vae, a, 314, category_only=True)
            off_b = surface.generate(model, vae, b, 314, category_only=True)
        np.testing.assert_array_equal(first, repeated)
        np.testing.assert_array_equal(off_a, off_b)
        self.assertFalse(np.array_equal(first, different))
        self.assertEqual(first.shape, (32 * surface.event.HOP, 2))

    def test_shared_band_level_uses_db_difference(self):
        physical, _ = surface.event.profile()
        real = np.random.default_rng(1).normal(0, 0.001, (140000, 2))
        record = {
            "row": {"crop_start_seconds": 1, "commanded_speed_mm_s": 40},
            "physical": physical,
        }
        result = surface.metrics(real * 2, real, record)
        self.assertAlmostEqual(
            result["moving_level_absolute_error_db"], 20 * np.log10(2), places=6
        )
        self.assertLess(result["moving_shape_rmse_db"], 1e-4)

    def test_saved_model_identity_and_training_isolation(self):
        model = surface.expand(surface.flow.TextureFlow())
        rows = surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        meta = {
            "format": surface.FORMAT,
            "codec_sha256": surface.flow.CODEC_SHA,
            "input_gain": surface.flow.GAIN,
            "table_sha256": surface.TABLE_SHA,
            "training_surfaces": list(surface.TRAIN),
            "held_surfaces": list(surface.HELD),
            "training_ids": [r["id"] for r in rows if surface.role(r) == "train"],
        }
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for arm in ("descriptor", "category_only"):
                weights = root / f"{arm}.safetensors"
                save_file(model.state_dict(), weights)
                meta[arm] = {"sha256": surface.flow.codec.sha(weights)}
            path = root / "model.json"
            path.write_text(json.dumps(meta))
            loaded, _ = surface.load_models(root, "cpu")
            self.assertEqual(len(loaded), 2)
            meta["training_ids"][0] = "76_0_20_500_0"
            path.write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                surface.load_models(root, "cpu")
            meta["training_ids"] = [r["id"] for r in rows if surface.role(r) == "train"]
            meta["descriptor"]["sha256"] = "0" * 64
            path.write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                surface.load_models(root, "cpu")


if __name__ == "__main__":
    unittest.main()
