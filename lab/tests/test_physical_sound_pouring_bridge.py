import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np
import torch
from scipy.io import wavfile
from torch import nn

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_bridge as b


class BridgeTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)

    def test_zero_exact_and_only_positive_text_changes(self):
        bridge = b.Bridge()
        hidden, pooled = torch.randn(2, 7, 1024), torch.randn(2, 1024)
        controls = torch.randn(1, 11)
        first, second = bridge.condition(controls, hidden, pooled, cfg=True)
        self.assertTrue(torch.equal(first, hidden))
        self.assertTrue(torch.equal(second, pooled))
        with torch.no_grad():
            bridge.network[-1].bias.fill_(0.25)
        first, second = bridge.condition(controls, hidden, pooled, cfg=True)
        self.assertTrue(torch.equal(first[:1], hidden[:1]))
        self.assertTrue(torch.equal(second[:1], pooled[:1]))
        self.assertTrue(torch.equal(first[:, -1:], hidden[:, -1:]))
        torch.testing.assert_close(first[1, :-1], hidden[1, :-1] + 0.25)
        torch.testing.assert_close(second[1], pooled[1] + 0.25)

    def test_gradients_with_frozen_backbone(self):
        bridge = b.Bridge()
        backbone = nn.Linear(1024, 1).requires_grad_(False)
        hidden, pooled = bridge.condition(
            torch.ones(1, 11), torch.randn(1, 7, 1024), torch.randn(1, 1024)
        )
        (backbone(hidden).square().mean() + backbone(pooled).square().mean()).backward()
        self.assertTrue(all(p.grad is None for p in backbone.parameters()))
        self.assertTrue(
            all(
                p.grad is not None and torch.isfinite(p.grad).all()
                for p in bridge.parameters()
            )
        )
        self.assertGreater(float(bridge.network[-1].weight.grad.abs().sum()), 0)

    def test_hook_removed_after_failure(self):
        bridge = b.Bridge()
        backbone = nn.Identity()
        with (
            self.assertRaisesRegex(RuntimeError, "expected"),
            bridge.hook(backbone, torch.ones(1, 11)),
        ):
            self.assertEqual(len(backbone._forward_pre_hooks), 1)
            raise RuntimeError("expected")
        self.assertEqual(len(backbone._forward_pre_hooks), 0)

    def test_input_and_checkpoint_failures(self):
        bridge = b.Bridge()
        for controls in [torch.zeros(2, 11), torch.full((1, 11), float("nan"))]:
            with self.assertRaises(ValueError):
                bridge.condition(
                    controls, torch.zeros(1, 7, 1024), torch.zeros(1, 1024)
                )
        with self.assertRaises(ValueError):
            bridge.condition(
                torch.zeros(1, 11), torch.zeros(2, 7, 1024), torch.zeros(2, 1024)
            )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "bridge.safetensors"
            b.save_file(bridge.state_dict(), path)
            meta = {
                "format": b.FORMAT,
                "status": "failed",
                "revision": b.train.tango.REVISION,
                "bridge_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            }
            (root / "result.json").write_text(json.dumps(meta))
            with self.assertRaisesRegex(ValueError, "identity"):
                b.load_bridge(root)
            meta["status"] = "complete"
            meta["bridge_sha256"] = "wrong"
            (root / "result.json").write_text(json.dumps(meta))
            with self.assertRaisesRegex(ValueError, "identity"):
                b.load_bridge(root)
        with self.assertRaises(ValueError):
            b.run(Path("absent"), Path("absent"), Path("unused"), steps=0)
        with self.assertRaises(ValueError):
            b.generate(None, None, Path("unused"), "bad", seed=-1)

    def test_comparison_uses_published_pcm_at_headroom_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            value = np.array([0.98, -0.98] * 8000, dtype=np.float32)
            row = b.train.pilot.write_audio(root / "input.wav", value)
            row["seed"] = 2718
            result = b.write_comparison(root, [row])
            rate, pcm = wavfile.read(result["wav"])
            _, original = wavfile.read(row["wav"])
            self.assertEqual(rate, 16000)
            np.testing.assert_array_equal(pcm[: len(original)], original)
            self.assertTrue(np.all(pcm[len(original) :] == 0))
            self.assertEqual(result["pcm_gain"], 1)
            row["sha256"] = "wrong"
            with self.assertRaisesRegex(ValueError, "hash"):
                b.write_comparison(root, [row])

    def test_generation_freezes_reload_parameter_flags(self):
        class Stub(nn.Module):
            def __init__(self):
                super().__init__()
                self.weight = nn.Parameter(torch.ones(1))

            def to(self, *args, **kwargs):
                return self

            def inference_flow(self, *args, **kwargs):
                return torch.ones(1, 3, 64)

            def decode(self, latent):
                return SimpleNamespace(sample=torch.ones(1, 2, 300))

        model, vae = Stub(), Stub()
        with patch.object(b.train.tango, "publish", return_value=({}, np.ones(300))):
            b.generate(model, vae, Path("unused"), "flag-test")
        self.assertFalse(model.weight.requires_grad)
        self.assertFalse(vae.weight.requires_grad)
        self.assertTrue(torch.equal(model.weight, torch.ones(1)))

    def test_centering_preserves_pairwise_deltas_and_original_checkpoint(self):
        bridge = b.Bridge()
        with torch.no_grad():
            bridge.network[-1].weight.normal_(0, 0.01)
        controls = torch.randn(2, 11)
        raw = bridge.network(controls)
        bridge.offset = raw.mean(0, keepdim=True).detach()
        adjusted = []
        for c in controls:
            hidden, pooled = bridge.condition(
                c[None], torch.zeros(2, 3, 1024), torch.zeros(2, 1024), cfg=True
            )
            adjusted.append(torch.cat([hidden[1, 0], pooled[1]]))
            self.assertTrue(torch.equal(hidden[0], torch.zeros_like(hidden[0])))
            self.assertTrue(torch.equal(hidden[:, -1], torch.zeros(2, 1024)))
        torch.testing.assert_close(adjusted[1] - adjusted[0], raw[1] - raw[0])
        self.assertNotIn("offset", bridge.state_dict())

    def test_centering_loader_identity_and_tensor_checks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = {
                "bridge_sha256": "bridge",
                "frozen_model_sha256_after": "frozen",
                "source": {"posterior_sha256": "train"},
            }
            meta = {
                "status": "complete",
                "bridge_sha256": "bridge",
                "frozen_model_sha256": "frozen",
                "training_posterior_sha256": "train",
            }
            for value, valid in [
                (torch.zeros(1, 2048), True),
                (torch.zeros(2, 2048), False),
                (torch.zeros(1, 2048, dtype=torch.float64), False),
                (torch.full((1, 2048), float("nan")), False),
            ]:
                path = root / "offset.safetensors"
                b.save_file({"offset": value}, path)
                meta["offset_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
                (root / "result.json").write_text(json.dumps(meta))
                if valid:
                    actual, _ = b.load_offset(root, model)
                    self.assertTrue(torch.equal(actual, value))
                else:
                    with self.assertRaisesRegex(ValueError, "tensor"):
                        b.load_offset(root, model)
            meta["training_posterior_sha256"] = "different"
            (root / "result.json").write_text(json.dumps(meta))
            with self.assertRaisesRegex(ValueError, "identity"):
                b.load_offset(root, model)

    def test_training_centering_cancels_common_bias_and_freezes_exactly(self):
        bridge = b.Bridge()
        bank = torch.randn(13, 11)
        bridge.center_controls = bank
        with torch.no_grad():
            bridge.network[-1].weight.normal_(0, 0.01)
        hidden, pooled = torch.zeros(1, 3, 1024), torch.zeros(1, 1024)
        predictions = [bridge.condition(c[None], hidden, pooled) for c in bank]
        torch.testing.assert_close(
            torch.stack([v[1] for v in predictions]).mean(0),
            torch.zeros_like(pooled),
            rtol=0,
            atol=1e-7,
        )
        predictions[0][1].square().mean().backward()
        self.assertGreater(float(bridge.network[-1].weight.grad.abs().sum()), 0)
        torch.testing.assert_close(
            bridge.network[-1].bias.grad,
            torch.zeros_like(bridge.network[-1].bias),
            rtol=0,
            atol=1e-9,
        )
        offset = bridge.freeze_centering()
        self.assertFalse(offset.requires_grad)
        self.assertIsNone(bridge.center_controls)
        frozen = bridge.condition(bank[:1], hidden, pooled)
        for a, original in zip(frozen, predictions[0], strict=True):
            self.assertTrue(torch.equal(a, original))
        self.assertNotIn("center_controls", bridge.state_dict())
        with self.assertRaises(ValueError):
            bridge.freeze_centering()

    def test_center_trained_checkpoint_automatically_loads_frozen_offset(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            b.save_file(b.Bridge().state_dict(), root / "bridge.safetensors")
            b.save_file({"offset": torch.ones(1, 2048)}, root / "offset.safetensors")
            meta = {
                "format": b.FORMAT,
                "status": "complete",
                "revision": b.train.tango.REVISION,
                "center_training": True,
                "bridge_sha256": hashlib.sha256(
                    (root / "bridge.safetensors").read_bytes()
                ).hexdigest(),
                "offset_sha256": hashlib.sha256(
                    (root / "offset.safetensors").read_bytes()
                ).hexdigest(),
                "source": {"posterior_sha256": "train"},
                "training_posterior_sha256": "train",
                "frozen_model_sha256_after": "model",
                "frozen_model_sha256": "model",
            }
            (root / "result.json").write_text(json.dumps(meta))
            with patch.object(b.Bridge, "to", lambda self, *a, **kw: self):
                model, _ = b.load_bridge(root)
            self.assertTrue(torch.equal(model.offset, torch.ones(1, 2048)))
            meta["offset_sha256"] = "wrong"
            (root / "result.json").write_text(json.dumps(meta))
            with self.assertRaisesRegex(ValueError, "identity"):
                b.load_bridge(root)


if __name__ == "__main__":
    unittest.main()
