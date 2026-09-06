import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch
from torch import nn

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2 as of


class DummyAudio(nn.Module):
    def __init__(self, **_):
        super().__init__()
        self.bias = nn.Parameter(torch.zeros(1))

    def forward(self, x, y, z):
        return tuple(v[:, :2] + self.bias for v in (x, y, z))


class ObjectFolderTests(unittest.TestCase):
    def test_modal_equation_matches_independent_single_mode(self):
        t = np.arange(of.RATE * 3) / of.RATE
        wave = of.waveform(np.array([[2], [3], [4]]), [1000], [12], [0, 1, 0])
        exact = 3 * np.exp(-12 * t) * np.sin(2 * np.pi * 1000 * t)
        exact[of.RATE * 2 :] = 0
        np.testing.assert_allclose(wave, exact, atol=1e-13)

    def test_linear_force_superposition_sign_and_zero(self):
        g = np.array([[2, 3], [-1, 0.2], [0.5, -1]])

        def render(force):
            return of.waveform(g, [1000, 2134], [10, 25], force)

        base = render([1, 1, 1])
        self.assertTrue(np.array_equal(render([0.5] * 3), base * 0.5))
        self.assertTrue(np.array_equal(render([-1] * 3), -base))
        self.assertTrue(np.array_equal(render([0] * 3), np.zeros_like(base)))
        np.testing.assert_allclose(
            render([1, 0, 0]) + render([0, 1, 0]) + render([0, 0, 1]), base, atol=1e-14
        )
        with self.assertRaises(ValueError):
            render([1, np.nan, 0])

    def test_peak_normalization_would_erase_force_and_break_zero(self):
        y = of.waveform(np.ones((3, 1)), [1000], [10], [1, 0, 0])
        self.assertTrue(
            np.array_equal(y / abs(y).max(), (0.5 * y) / abs(0.5 * y).max())
        )
        with np.errstate(invalid="ignore"):
            bad = np.zeros(3) / np.max(np.zeros(3))
        self.assertFalse(np.isfinite(bad).any())

    def test_pcm_quantization_bound_does_not_hide_force_error(self):
        # Each output is independently rounded from its higher precision value.
        tiny = float(np.nextafter(np.float32(0), np.float32(1)))
        full = np.array([tiny * 1.49, 0.3], dtype=np.float64)
        base = full.astype(np.float32)
        half = (full * 0.5).astype(np.float32)
        check = of.pcm_scaling_check(base, half, 0.5)
        self.assertFalse(check["bit_exact"])
        self.assertTrue(check["within_pcm_rounding_bound"])
        self.assertFalse(
            of.pcm_scaling_check(base, base * 0.51, 0.5)["within_pcm_rounding_bound"]
        )

    def test_coordinate_mapping_gain_restore_and_support_rejection(self):
        audio = {
            "normalizer": {"xyz_min": -2.0, "xyz_max": 2.0},
            "frequencies": [100, 200],
            "model_state_dict": {"module.bias": torch.tensor([0.25])},
        }
        for i in range(1, 4):
            audio["normalizer"].update({f"f{i}_min": -i, f"f{i}_max": i})
        definitions = {
            "get_embedder": lambda *_: (lambda x: x, 3),
            "AudioNeRF": DummyAudio,
        }
        actual = of.point_gains(audio, definitions, [[0, -2, 2]])
        np.testing.assert_allclose(actual[0], [[0.5, -0.5], [1, -1], [1.5, -1.5]])
        with self.assertRaisesRegex(ValueError, "outside"):
            of.point_gains(audio, definitions, [[0, 0, 2.01]])

    def test_changed_sources_rejected_before_execution_or_deserialization(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "changed"
            path.write_bytes(b"unreviewed content")
            with self.assertRaisesRegex(ValueError, "unreviewed"):
                of.source_definitions(path)
            with self.assertRaisesRegex(ValueError, "changed"):
                of.load_audio(path, of.DEMO_WEIGHTS)


if __name__ == "__main__":
    unittest.main()
