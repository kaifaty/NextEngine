import copy
import tempfile
import unittest
from pathlib import Path

import numpy as np
from next_lab.isaac_training import atomic_write_json, sha256_file

from lab.scripts.render_native_walking_video import (
    frame_geometry,
    load_closed_evaluation,
)
from lab.tests.test_cpu_walking_contact_audit import fixture


class NativeVideoTests(unittest.TestCase):
    def setUp(self):
        trace, self.descriptor, _ = fixture()
        self.frame = trace["frames"][0][0]
        self.descriptor["motor_hz"] = 60
        for index, body in enumerate(self.descriptor["bodies"]):
            body["parent_body_slot"] = None if index == 0 else 0

    def test_real_sole_boxes_are_drawn_not_slanted_origin_connections(self):
        segments, colors, feet, root = frame_geometry(self.frame, self.descriptor)
        self.assertEqual(segments.shape, (26, 2, 3))
        self.assertEqual(len(colors), 26)
        self.assertEqual(len(feet), 2)
        self.assertEqual(float(feet[0][1][:, :, 1].min()), 0)
        np.testing.assert_array_equal(root, [0, 1, 0])
        self.frame["links"][2]["position_um"][1] += 60_000
        _, _, feet, _ = frame_geometry(self.frame, self.descriptor)
        self.assertAlmostEqual(float(feet[1][1][:, :, 1].min()), 0.06)

    def test_duplicate_body_tokens_reject(self):
        self.frame["links"].append(copy.deepcopy(self.frame["links"][0]))
        with self.assertRaisesRegex(ValueError, "token set"):
            frame_geometry(self.frame, self.descriptor)

    def test_closed_loader_preserves_failed_full_prefix_and_rejects_corruption(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)
            dp = path / "descriptor.json"
            atomic_write_json(dp, self.descriptor)
            atomic_write_json(
                path / "native-1001.json",
                {"profile_id": "v8", "frames": [[self.frame]]},
            )
            atomic_write_json(
                path / "support-1001/report.json",
                {"lifted_support": {"frames": [{"tick": 1}]}},
            )
            atomic_write_json(
                path / "evaluation.json",
                {"episodes": [{"seed": 1001, "ticks": 1, "passed": False}]},
            )
            manifest = {
                "schema": "nextengine.corrected-walking-evaluation-run.v1",
                "status": "completed",
                "matrix": {
                    "evaluation": {"seeds": [1001]},
                    "source_checkpoint_name": "model_9999.pt",
                    "target_environment_profile_id": "v8",
                },
                "target_descriptor_path": str(dp),
                "inputs": {"target_descriptor_sha256": sha256_file(dp)},
                "artifacts": {
                    str(p.relative_to(path)): sha256_file(p)
                    for p in path.rglob("*")
                    if p.is_file()
                },
            }
            atomic_write_json(path / "run-manifest.json", manifest)
            self.assertFalse(load_closed_evaluation(path, 1001)[-1]["passed"])
            with self.assertRaisesRegex(ValueError, "seed outside"):
                load_closed_evaluation(path, 1002)
            atomic_write_json(path / "native-1001.json", {})
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                load_closed_evaluation(path, 1001)


if __name__ == "__main__":
    unittest.main()
