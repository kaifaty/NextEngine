"""Bounded extraction, source-free input and pre-audio family split guards."""

import io
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_objectfolder2_expand as expansion
import physical_sound_objectfolder2_expanded_eval as evaluation
import physical_sound_objectfolder2_expanded_train as training


class ExpansionTests(unittest.TestCase):
    def test_joined_stream_preserves_boundaries(self):
        stream = io.BufferedReader(
            expansion.Joined(
                [io.BytesIO(b"abc"), io.BytesIO(b""), io.BytesIO(b"defghi")]
            )
        )
        self.assertEqual(stream.read(4), b"abcd")
        self.assertEqual(stream.read(), b"efghi")
        self.assertEqual(stream.read(), b"")

    def test_extract_only_complete_regular_known_names(self):
        member = tarfile.TarInfo("ObjectFolder1-100/91/model.obj")
        member.size = 100
        self.assertEqual(expansion.member_target(member), (91, "model.obj"))
        for name in (
            "ObjectFolder1-100/7/model.obj",
            "ObjectFolder1-100/91/../../model.obj",
            "/ObjectFolder1-100/91/model.obj",
            "ObjectFolder1-100/101/model.obj",
            "ObjectFolder1-100/91/texture.png",
        ):
            member.name = name
            self.assertIsNone(expansion.member_target(member))
        member.name = "ObjectFolder1-100/91/ObjectFile.pth"
        member.type = tarfile.SYMTYPE
        self.assertIsNone(expansion.member_target(member))
        member.type = tarfile.REGTYPE
        member.size = 60_000_001
        self.assertIsNone(expansion.member_target(member))

    def test_related_cups_and_possible_bowl_family_stay_development(self):
        for identity in range(37, 47):
            self.assertEqual(evaluation.cohort_role(identity), "development")
        self.assertEqual(evaluation.cohort_role(53), "development")
        self.assertEqual(evaluation.cohort_role(59), "train")
        self.assertEqual(expansion.role(37), "train")  # Acquisition record retained.

    def test_geometry_rejects_targets_unknown_material_and_nonfinite(self):
        values = {
            "cloud": np.zeros((512, 3), np.float32),
            "features": np.array([0, 0, 0, 1, 0, 0, 0, 0], np.float32),
            "contacts": np.zeros((32, 3), np.float32),
        }
        evaluation.check_geometry(values)
        with self.assertRaisesRegex(ValueError, "geometry-only"):
            evaluation.check_geometry(values | {"frequency": np.ones(3)})
        values["features"][3] = 0
        with self.assertRaisesRegex(ValueError, "one-hot"):
            evaluation.check_geometry(values)
        values["features"][3] = 1
        values["cloud"][0, 0] = np.nan
        with self.assertRaisesRegex(ValueError, "finite"):
            evaluation.check_geometry(values)

    def test_artifact_hash_required(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input.npz"
            np.savez(path, frequency=np.ones(3))
            with self.assertRaisesRegex(ValueError, "changed"):
                evaluation.read_npz(path, "wrong")
            value = evaluation.read_npz(path, expansion.digest(path))
            np.testing.assert_array_equal(value["frequency"], np.ones(3))

    def test_expanded_fit_rejects_development_and_duplicate_rows(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rows = [
                {"object_id": i, "role": "train"} for i in sorted(training.ALL_TRAIN)
            ]
            rows[0]["role"] = "development"
            (root / "train.json").write_text(json.dumps({"rows": rows}))
            with self.assertRaisesRegex(ValueError, "TRAIN-only"):
                training.shared.fit(
                    SimpleNamespace(data=root), train_ids=training.ALL_TRAIN
                )
            rows[0]["role"] = "train"
            rows.append(rows[0])
            (root / "train.json").write_text(json.dumps({"rows": rows}))
            with self.assertRaisesRegex(ValueError, "TRAIN-only"):
                training.shared.fit(
                    SimpleNamespace(data=root), train_ids=training.ALL_TRAIN
                )


if __name__ == "__main__":
    unittest.main()
