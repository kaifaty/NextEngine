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
import physical_sound_pouring_cvae as v


class PourCVAETests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)

    def test_kl_identity_and_nonidentity(self):
        mu, lv = torch.randn(2, 8, 32, 16), torch.randn(2, 8, 32, 16)
        self.assertAlmostEqual(float(v.gaussian_kl((mu, lv), (mu, lv))), 0, places=6)
        self.assertGreater(float(v.gaussian_kl((mu, lv), (mu + 1, lv))), 0)

    def test_shapes_and_reconstruction_gradients(self):
        model = v.PourCVAE()
        c = torch.tensor(v.phase.d.controls_for()[None])
        target = v.encode(torch.randn(1, v.p.SAMPLES) * 0.01)[:, None]
        self.assertEqual(tuple(target.shape), (1, 1, 512, 256))
        q, prior = model.posterior(target, c), model.prior(c)
        self.assertEqual(tuple(q[0].shape), (1, 8, 32, 16))
        z = q[0] + (q[1] * 0.5).exp() * torch.randn_like(q[0])
        decoded = model.decode(z, c)
        self.assertEqual(decoded.shape, target.shape)
        loss = v.reconstruction_loss(
            decoded, target, torch.randn(1, v.p.SAMPLES)
        ) + 0.01 * v.gaussian_kl(q, prior)
        loss.backward()
        self.assertTrue(
            all(
                p.grad is not None and torch.isfinite(p.grad).all()
                for p in model.parameters()
            )
        )

    def test_source_free_seeded_prior_avoids_posterior(self):
        model = v.PourCVAE().eval()
        controls = v.phase.d.controls_for()
        with (
            patch.object(
                model,
                "posterior",
                side_effect=AssertionError("reference path forbidden"),
            ),
            patch.object(
                v, "encode", side_effect=AssertionError("reference audio forbidden")
            ),
        ):
            a = v.sample(model, controls, 2718)
            np.testing.assert_array_equal(a, v.sample(model, controls, 2718))
            self.assertFalse(np.array_equal(a, v.sample(model, controls, 314)))
        self.assertEqual(a.shape, (v.p.SAMPLES,))
        with self.assertRaises(ValueError):
            v.sample(model, controls, -1)
        with self.assertRaises(ValueError):
            v.sample(model, np.zeros(11))
        with self.assertRaises(ValueError):
            v.sample(model, controls, reference=np.zeros(3))

    def test_checkpoint_roundtrip_and_integrity(self):
        model = v.PourCVAE()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkpoint = root / "model.safetensors"
            v.save_file(model.state_dict(), checkpoint)
            meta = {
                "format": "pour-cvae-v1",
                "checkpoint_sha256": hashlib.sha256(
                    checkpoint.read_bytes()
                ).hexdigest(),
            }
            (root / "model.json").write_text(json.dumps(meta))
            loaded, _ = v.load(root, "cpu")
            self.assertTrue(
                all(
                    torch.equal(x, loaded.state_dict()[k])
                    for k, x in model.state_dict().items()
                )
            )
            checkpoint.write_bytes(b"bad")
            with self.assertRaises(ValueError):
                v.load(root, "cpu")


if __name__ == "__main__":
    unittest.main()
