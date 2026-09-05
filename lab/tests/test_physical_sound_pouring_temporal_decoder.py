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
import physical_sound_pouring_temporal_decoder as d


class TemporalDecoderTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)

    def test_constant_filter_identity(self):
        noise = torch.randn(2, d.p.SAMPLES)
        gain = torch.ones(2, d.FFT // 2 + 1, 256) * 0.01
        torch.testing.assert_close(
            d.filtered_noise(noise, gain), noise * 0.01, atol=1e-8, rtol=1e-5
        )

    def test_temporal_response_and_frozen_head_gradients(self):
        model = d.TemporalDecoder()
        controls = torch.tensor(d.h.grid(d.controls_for(), d.TIMES)[None])
        hz = torch.linspace(500, 2500, 256)[None]
        moving = model.response(controls, hz)
        static = model.response(controls, hz, "static")
        self.assertFalse(torch.allclose(moving, static))
        torch.testing.assert_close(moving.square().mean(-1), static.square().mean(-1))
        noise = torch.randn(1, d.p.SAMPLES)
        wave = model(controls, noise)
        loss = d.objective(wave, noise * 0.02)
        loss.backward()
        self.assertTrue(all(x.grad is None for x in model.head.parameters()))
        self.assertTrue(
            all(
                x.grad is not None and torch.isfinite(x.grad).all()
                for n, x in model.named_parameters()
                if not n.startswith("head.")
            )
        )

    def test_objective_identity_and_time_mismatch(self):
        target = torch.randn(1, d.p.SAMPLES) * torch.linspace(0.001, 0.04, d.p.SAMPLES)
        wave = target.clone().requires_grad_()
        loss = d.objective(wave, target)
        self.assertLess(float(loss.detach()), 1e-7)
        loss.backward()
        self.assertTrue(torch.isfinite(wave.grad).all())
        self.assertGreater(float(d.objective(target.flip(-1), target)), 0.1)

    def test_reference_free_sampling_and_validation(self):
        model = d.TemporalDecoder()
        controls = d.controls_for()
        with patch.object(
            d.p, "load_source", side_effect=AssertionError("no references")
        ):
            first = d.sample(model, controls, 2718)
            np.testing.assert_array_equal(first, d.sample(model, controls, 2718))
            self.assertFalse(np.array_equal(first, d.sample(model, controls, 314)))
        self.assertEqual(first.shape, (d.p.SAMPLES,))
        for bad in [-1, 2**32]:
            with self.assertRaises(ValueError):
                d.sample(model, controls, bad)
        with self.assertRaises(ValueError):
            d.sample(model, np.zeros(11))
        with self.assertRaises(ValueError):
            d.sample(model, controls, mode="wrong")

    def test_checkpoint_and_source_free_render(self):
        model = d.TemporalDecoder()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            weights = root / "model.safetensors"
            d.save_file(model.state_dict(), weights)
            metadata = {
                "format": "pour-temporal-decoder-v1",
                "checkpoint_sha256": hashlib.sha256(weights.read_bytes()).hexdigest(),
            }
            (root / "model.json").write_text(json.dumps(metadata))
            loaded, _ = d.load(root, "cpu")
            np.testing.assert_array_equal(
                d.sample(model, d.controls_for()), d.sample(loaded, d.controls_for())
            )
            with patch.object(
                d.p, "load_source", side_effect=AssertionError("no references")
            ):
                d.render(root, root / "out", d.controls_for(), device="cpu")
            result = json.loads((root / "out/result.json").read_text())
            self.assertFalse(result["reference_audio_input"])
            self.assertFalse(result["teacher_required_at_inference"])
            weights.write_bytes(b"bad")
            with self.assertRaises(ValueError):
                d.load(root, "cpu")


if __name__ == "__main__":
    unittest.main()
