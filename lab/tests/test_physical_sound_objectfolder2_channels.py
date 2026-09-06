"""Fixed frequency-channel packing and source-free decoder invariants."""

import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_channels as channels
import physical_sound_objectfolder2_rayleigh as rayleigh


class ChannelTests(unittest.TestCase):
    def test_rayleigh_oscillator_equations_and_unknown_material(self):
        f = np.array([20.0, 1000.0, 21000.0])
        for material, (alpha, beta) in rayleigh.COEFFICIENTS.items():
            d = rayleigh.damping(f, material)
            eigenvalue = (2 * np.pi * f) ** 2 + d**2
            np.testing.assert_allclose(d, (alpha + beta * eigenvalue) / 2, rtol=1e-14)
        with self.assertRaisesRegex(ValueError, "known"):
            rayleigh.damping(f, "Unknown")
        with self.assertRaisesRegex(ValueError, "frequencies"):
            rayleigh.damping([np.nan], "Wood")

    def test_rayleigh_changes_only_decay(self):
        model = channels.ChannelStudent().eval().requires_grad_(False)
        cloud = np.zeros((512, 3), np.float32)
        features = np.zeros(8, np.float32)
        features[3] = 1  # Wood in shared material vocabulary.
        contacts = np.zeros((2, 3), np.float32)
        raw = channels.predict(model, cloud, features, contacts)
        corrected = rayleigh.predict(model, cloud, features, contacts)
        for key in ("frequency", "gains", "occupancy_probability"):
            np.testing.assert_array_equal(raw[key], corrected[key])
        np.testing.assert_array_equal(raw["damping"], corrected["learned_damping"])
        self.assertTrue(np.all(corrected["damping"] >= 30))
        features[4] = 1
        with self.assertRaisesRegex(ValueError, "one known"):
            rayleigh.predict(model, cloud, features, contacts)

    def test_fixed_channel_identity_and_reject_wrong_band(self):
        edges = channels.band_edges()
        row = {
            "bands": [{"band": 20, "packed_modes": 2}, {"band": 70, "packed_modes": 1}]
        }
        f = np.array([(edges[20] + edges[21]) / 2] * 2 + [(edges[70] + edges[71]) / 2])
        np.testing.assert_array_equal(
            channels.channel_indices(row, {"frequency": f}), [60, 61, 210]
        )
        with self.assertRaisesRegex(ValueError, "outside"):
            channels.channel_indices(row, {"frequency": f[::-1]})
        row["bands"][1]["band"] = 20
        with self.assertRaisesRegex(ValueError, "slots"):
            channels.channel_indices(row, {"frequency": f})

    def test_parameter_budget_and_contact_independent_poles(self):
        model = channels.ChannelStudent().eval().requires_grad_(False)
        count = sum(p.numel() for p in model.parameters())
        self.assertEqual(count, 69712)
        self.assertLess(count, 68486 * 1.02)
        cloud = np.zeros((512, 3), np.float32)
        features = np.zeros(8, np.float32)
        points = np.array([[0, 0, 0], [0.1, 0.2, 0.3]], np.float32)
        a = channels.predict(model, cloud, features, points)
        b = channels.predict(model, cloud, features, points[1:])
        np.testing.assert_array_equal(a["frequency"], b["frequency"])
        np.testing.assert_array_equal(a["damping"], b["damping"])
        self.assertEqual(a["gains"].shape, (2, 3, len(a["frequency"])))
        self.assertTrue(np.all((a["frequency"] > 0) & (a["frequency"] < 22050)))
        self.assertTrue(np.all(a["damping"] > 0))

    def test_empty_mask_is_silent_not_repaired(self):
        model = channels.ChannelStudent().eval().requires_grad_(False)
        model.poles[-1].weight.zero_()
        model.poles[-1].bias.reshape(-1, 3)[:, 2] = -100
        result = channels.predict(
            model,
            np.zeros((512, 3), np.float32),
            np.zeros(8, np.float32),
            np.zeros((1, 3), np.float32),
        )
        self.assertEqual(len(result["frequency"]), 0)
        wave = channels.teacher.waveform(
            result["gains"][0], result["frequency"], result["damping"], [1, 1, 1]
        )
        self.assertFalse(np.any(wave))

    def test_output_channels_independent(self):
        model = channels.ChannelStudent().eval().requires_grad_(False)
        model.field[-1].weight.zero_()
        model.field[-1].bias.zero_()
        model.field[-1].bias[5] = -2
        values = model.signed_field(torch.zeros(1, 64), torch.zeros(1, 3)).numpy()
        self.assertEqual(np.count_nonzero(values), 1)
        self.assertEqual(values[0, 1, 2], -2)

    def test_development_manifest_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rows = [
                {"object_id": i, "role": "train"} for i in sorted(channels.shared.TRAIN)
            ]
            rows[0]["role"] = "development"
            (root / "train.json").write_text(json.dumps({"rows": rows}))
            with self.assertRaisesRegex(ValueError, "TRAIN-only"):
                channels.fit(SimpleNamespace(data=root))


if __name__ == "__main__":
    unittest.main()
