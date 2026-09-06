import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_pilot as p
import physical_sound_pouring_relative_level as r


class RelativeLevelTests(unittest.TestCase):
    def test_relative_fit_is_invariant_to_record_gain(self):
        controls = np.zeros((3, 2, 11))
        controls[:, 1, 4] = [0.2, 0.3, 0.4]
        controls[:, :, 0] = np.array([0.3, 0.6, 0.9])[:, None]
        levels = np.array([[-20.0, -25.0], [-30.0, -37.0], [-40.0, -49.0]])
        a = r.coefficients(controls, levels)
        b = r.coefficients(controls, levels + np.array([13.0, -7.0, 30.0])[:, None])
        np.testing.assert_allclose(a["relative"], b["relative"], atol=1e-12)
        self.assertLess(a["relative"][5], 0)
        self.assertTrue(np.all(a["zero-phase"] == 0))

    def test_calibration_matches_relative_level_and_preserves_shape(self):
        wave = np.random.default_rng(53).normal(0, 0.01, p.SAMPLES).astype(np.float32)
        anchor = wave * 2
        result, gain = r.calibrate(wave, anchor, 0.25, -20)
        self.assertAlmostEqual(r.level(result) - r.level(anchor), -5, places=5)
        np.testing.assert_allclose(result, wave * gain, atol=1e-8)
        self.assertLess(p.metrics(result, wave)["spectrum_shape_rmse_db"], 0.01)
        with self.assertRaises(ValueError):
            r.calibrate(wave, np.ones_like(wave) * 2, 0, -20)
        with self.assertRaises(ValueError):
            r.calibrate(wave, anchor, 1.1, -20)

    def test_source_free_render_and_parent_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cal = root / "calibration"
            cal.mkdir()
            meta = {
                "format": "pour-relative-level-v1",
                "parent_checkpoint_sha256": "parent",
                "phase_slopes_db": {"relative": -20},
            }
            (cal / "model.json").write_text(json.dumps(meta))
            controls = p.condition(
                {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7},
                "glass",
                "cylindrical",
                15,
                0.2,
            )
            with (
                patch.object(
                    r.compare,
                    "load_model",
                    return_value=(object(), {"checkpoint_sha256": "parent"}),
                ),
                patch.object(
                    p, "load_source", side_effect=AssertionError("dataset forbidden")
                ),
                patch.object(
                    p,
                    "sample",
                    return_value=np.full((256, 256), -1.5, dtype=np.float32),
                ) as sample,
            ):
                r.render(root, cal, root / "out", controls)
                self.assertEqual(
                    [float(x.args[1][4]) for x in sample.call_args_list],
                    [0, float(controls[4])],
                )
                result = json.loads((root / "out/result.json").read_text())
                self.assertFalse(result["reference_audio_input"])
                meta["parent_checkpoint_sha256"] = "wrong"
                (cal / "model.json").write_text(json.dumps(meta))
                with self.assertRaises(ValueError):
                    r.render(root, cal, root / "bad", controls)


if __name__ == "__main__":
    unittest.main()
