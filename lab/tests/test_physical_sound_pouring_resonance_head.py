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
import physical_sound_pouring_resonance_head as r


class ResonanceHeadTests(unittest.TestCase):
    def test_filter_identity_and_level_preservation(self):
        wave = np.random.default_rng(53).normal(0, 0.01, p.SAMPLES).astype(np.float32)
        hz = np.linspace(600, 2500, 256)
        identity = r.filter_wave(wave, hz, boost=0)
        np.testing.assert_allclose(identity, wave, atol=1e-7)
        changed = r.filter_wave(wave, hz)
        self.assertAlmostEqual(
            float(np.mean(changed**2)), float(np.mean(wave**2)), places=9
        )
        self.assertFalse(np.allclose(changed, wave))
        for invalid in [np.full(256, np.nan), np.zeros(256), np.ones(5)]:
            with self.assertRaises(ValueError):
                r.filter_wave(wave, invalid)

    def test_trajectory_grid_and_head_transform(self):
        c = p.condition(
            {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7},
            "glass",
            "cylindrical",
            15,
            0.1,
        )
        controls = r.grid(c, np.array([0, 3, 30]))
        np.testing.assert_allclose(controls[:, 4], [0.1, 0.3, 1], atol=1e-6)
        model = r.ResonanceHead()
        with torch.no_grad():
            for parameter in model.parameters():
                parameter.zero_()
        np.testing.assert_allclose(r.predict(model, controls), 34000 / 32)
        self.assertGreater(r.simple_hz(controls)[1], r.simple_hz(controls)[0])
        with self.assertRaises(ValueError):
            r.grid(np.zeros(11), np.array([0]))
        for index, value in [(0, -1), (5, 0.5), (9, 0)]:
            bad = c.copy()
            bad[index] = value
            with self.assertRaises(ValueError):
                r.grid(bad, np.array([0]))

    def test_source_free_render_does_not_load_references(self):
        c = p.condition(
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
                patch.object(
                    r.compare,
                    "load_model",
                    return_value=(object(), {"checkpoint_sha256": "parent"}),
                ),
                patch.object(
                    r,
                    "load_head",
                    return_value=(r.ResonanceHead(), {"checkpoint_sha256": "head"}),
                ),
                patch.object(
                    p, "load_source", side_effect=AssertionError("reference forbidden")
                ),
                patch.object(p, "sample", return_value=None),
                patch.object(p, "decode", return_value=wave),
            ):
                r.render(root, root, root / "out", c, audition_gain=1, device="cpu")
                with self.assertRaises(ValueError):
                    r.render(
                        root, root, root / "unsafe", c, audition_gain=100, device="cpu"
                    )
                self.assertFalse((root / "unsafe").exists())
            result = json.loads((root / "out/result.json").read_text())
            self.assertFalse(result["reference_audio_input"])
            self.assertFalse(result["teacher_required_at_inference"])
            self.assertEqual(
                [x["kind"] for x in result["rows"]], ["base", "neural", "simple"]
            )


if __name__ == "__main__":
    unittest.main()
