import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_envelope as e
import physical_sound_pouring_pilot as p


class EnvelopeTests(unittest.TestCase):
    def test_evaluation_does_not_overwrite_existing_result(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / "result.json").write_text("{}")
            with self.assertRaises(ValueError):
                e.evaluate([], None, {}, {}, {}, {}, path)

    def test_identity_and_constant_gain(self):
        wave = np.full(p.SAMPLES, 0.01, dtype=np.float32)
        wave[::2] *= -1
        code = e.encode(wave)
        np.testing.assert_allclose(e.apply_envelope(wave, code), wave, atol=1e-7)
        np.testing.assert_allclose(
            e.apply_envelope(wave, code + np.log(2) / 3), wave * 2, atol=1e-7
        )

    def test_invalid_and_unsafe_outputs_rejected(self):
        with self.assertRaises(ValueError):
            e.envelope(np.zeros(10))
        wave = np.zeros(p.SAMPLES, dtype=np.float32)
        wave[0] = 0.1
        for code in [np.ones(3), np.full(32, float("nan")), np.ones(32) * 1.5]:
            with self.assertRaises(ValueError):
                e.apply_envelope(wave, code)

    def test_condition_gradient_and_reference_free_sampling(self):
        torch.manual_seed(53)
        model = e.EnvelopeFlow()
        c = torch.zeros(2, 11, requires_grad=True)
        model(
            torch.randn(2, 32), torch.tensor([0.2, 0.6]), c
        ).square().mean().backward()
        self.assertGreater(float(c.grad[:, 4].abs().sum()), 0)
        controls = np.zeros(11, dtype=np.float32)
        a = e.sample(model, controls, 314)
        b = e.sample(model, controls, 314)
        np.testing.assert_array_equal(a, b)
        self.assertEqual(a.shape, (32,))
        self.assertTrue(np.isfinite(a).all())


if __name__ == "__main__":
    unittest.main()
