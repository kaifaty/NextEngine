"""Focused shared geometry-to-modal student checks; not perceptual validation."""

import itertools
import json
import math
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_shared as shared
import physical_sound_objectfolder2_shared_assess as assessment


class SharedTests(unittest.TestCase):
    def test_size_baseline_identity_and_alias_accounting(self):
        source = {
            "frequency": np.array([1000.0, 15000.0]),
            "damping": np.array([10.0, 20.0]),
            "gains": np.ones((1, 3, 2)),
        }
        original = shared.teacher.waveform(
            source["gains"][0], source["frequency"], source["damping"], [1, 1, 1]
        )
        identity, omitted = assessment.scaled_response(source, 0, 1)
        np.testing.assert_array_equal(identity, original)
        self.assertEqual(omitted, 0)
        changed, omitted = assessment.scaled_response(source, 0, 2)
        self.assertEqual(omitted, 1)
        self.assertTrue(np.isfinite(changed).all())
        with self.assertRaisesRegex(ValueError, "ratio"):
            assessment.scaled_response(source, 0, -1)

    def test_metrics_detect_level_error(self):
        t = np.arange(44100 * 3) / 44100
        wave = np.sin(2 * np.pi * 1000 * t) * np.exp(-10 * t)
        equal = assessment.audio_metrics(wave, wave)
        quieter = assessment.audio_metrics(wave, wave * 0.1)
        for metric in equal:
            self.assertEqual(equal[metric], 0)
            self.assertGreater(quieter[metric], 0.5)

    def test_geometry_preserves_size_and_material(self):
        vertices = np.array(list(itertools.product((-1.0, 1.0), repeat=3)))
        cloud, features, _, _ = shared.geometry(vertices, "Wood")
        moved, moved_features, _, _ = shared.geometry(vertices + 7, "Wood")
        scaled, scaled_features, _, _ = shared.geometry(vertices * 2, "Wood")
        np.testing.assert_array_equal(cloud, moved)
        np.testing.assert_array_equal(features, moved_features)
        np.testing.assert_array_equal(cloud, scaled)
        np.testing.assert_allclose(scaled_features[:3] - features[:3], math.log(2))
        self.assertFalse(
            np.array_equal(features, shared.geometry(vertices, "Steel")[1])
        )
        with self.assertRaisesRegex(ValueError, "vocabulary"):
            shared.geometry(vertices, "Unknown")

    def test_pooling_point_permutation(self):
        model = shared.SharedStudent().double().eval()
        cloud = torch.randn(1, 512, 3, dtype=torch.float64)
        features = torch.randn(1, 8, dtype=torch.float64)
        # FP tolerance only for permutation of summation, not a bit-equality claim.
        torch.testing.assert_close(
            model.encode(cloud, features),
            model.encode(cloud.flip(1), features),
            rtol=1e-12,
            atol=1e-12,
        )

    def test_variable_modes_and_contact_independent_poles(self):
        model = shared.SharedStudent().eval().requires_grad_(False)
        model.count.weight.zero_()
        model.count.bias.zero_()
        model.count_mean.fill_(math.log(1965))
        cloud = np.zeros((512, 3), dtype=np.float32)
        features = np.zeros(8, dtype=np.float32)
        contacts = np.array([[0, 0, 0], [0.1, 0.2, 0.3]], dtype=np.float32)
        result = shared.predict(model, cloud, features, contacts)
        self.assertEqual(result["gains"].shape, (2, 3, 1965))
        other = shared.predict(model, cloud, features, contacts[1:])
        np.testing.assert_array_equal(result["frequency"], other["frequency"])
        self.assertTrue(
            np.all((result["frequency"] >= 1) & (result["frequency"] <= 22049))
        )
        self.assertTrue(np.all(result["damping"] > 0))
        model.count_mean.fill_(math.log(5000))
        with self.assertRaisesRegex(ValueError, "count"):
            shared.predict(model, cloud, features, contacts)

    def test_development_or_duplicate_train_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rows = [{"object_id": i, "role": "train"} for i in sorted(shared.TRAIN)]
            rows[0]["role"] = "development"
            (root / "train.json").write_text(json.dumps({"rows": rows}))
            with self.assertRaisesRegex(ValueError, "TRAIN-only"):
                shared.fit(SimpleNamespace(data=root))
            rows[0]["role"] = "train"
            rows.append(rows[0])
            (root / "train.json").write_text(json.dumps({"rows": rows}))
            with self.assertRaisesRegex(ValueError, "TRAIN-only"):
                shared.fit(SimpleNamespace(data=root))


if __name__ == "__main__":
    unittest.main()
