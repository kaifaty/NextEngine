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
            self.assertTrue(
                all(
                    torch.equal(x, loaded.state_dict()[k])
                    for k, x in model.state_dict().items()
                )
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


if __name__ == "__main__":
    unittest.main()
