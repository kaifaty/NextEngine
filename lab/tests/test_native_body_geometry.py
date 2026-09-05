import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

from lab.scripts import audit_body_proportions
from lab.scripts.native_body_geometry import (
    collider_geometry,
    neutral_proportions,
    physical_geometry,
)
from lab.tests.test_cpu_walking_contact_audit import fixture


def body_fixture():
    trace, descriptor, _ = fixture()
    descriptor["body_schema_id"] = "test.body"
    root = descriptor["bodies"][0]
    root["body_id"] = "body.pelvis"
    root["colliders"] = []
    box = copy.deepcopy(descriptor["bodies"][1]["colliders"][0])
    box["collider_id"] = "torso"
    box["local_translation_micrometres"] = [0, 300_000, 0]
    box["geometry"]["half_extents_micrometres"] = [160_000, 220_000, 120_000]
    head = copy.deepcopy(box)
    head["collider_id"] = "head"
    head["local_translation_micrometres"] = [0, 570_000, 0]
    head["geometry"] = {"kind": "sphere", "radius_micrometres": 105_000}
    root["colliders"] = [box, head]
    for body in descriptor["bodies"][1:]:
        body["colliders"][0]["collider_id"] = body["body_id"]
    for body, link in zip(
        descriptor["bodies"], trace["frames"][0][0]["links"], strict=True
    ):
        body["initial_translation_f32_bits"] = (
            np.asarray(np.asarray(link["position_um"]) / 1e6, dtype="<f4")
            .view("<u4")
            .tolist()
        )
        body["initial_rotation_f32_bits"] = [0, 0, 0, 1065353216]
    return descriptor, trace["frames"][0][0]


class PhysicalBodyGeometryTests(unittest.TestCase):
    def test_audit_cli_retains_source_path_after_constructing_plot_arrays(self):
        descriptor, frame = body_fixture()
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / "source"
            source.mkdir()
            (source / "run-manifest.json").write_text("{}")
            output = Path(temp) / "output"
            manifest = {"inputs": {"target_descriptor_sha256": "ab" * 32}}
            with (
                patch(
                    "sys.argv",
                    ["audit", "--evaluation", str(source), "--output", str(output)],
                ),
                patch.object(
                    audit_body_proportions,
                    "load_closed_evaluation",
                    return_value=(manifest, descriptor, [frame], [], {}),
                ),
                patch.object(
                    audit_body_proportions,
                    "neutral_proportions",
                    return_value={"status": "report_only"},
                ),
                patch.object(audit_body_proportions, "plot_comparison"),
                patch.object(audit_body_proportions, "plot") as draw,
            ):
                audit_body_proportions.main()
            report = json.loads((output / "report.json").read_text())
            self.assertEqual(report["native_ticks"], 1)
            self.assertFalse(report["physics_or_weights_changed"])
            self.assertEqual((source / "run-manifest.json").read_text(), "{}")
            self.assertEqual(draw.call_args.args[-1], output)

    def test_torso_and_head_are_present_even_without_child_joint_origins(self):
        descriptor, frame = body_fixture()
        before = copy.deepcopy((descriptor, frame))
        shapes, _ = physical_geometry(descriptor, frame)
        self.assertEqual(len(shapes), 4)
        self.assertEqual(shapes[0]["segments"].shape, (12, 2, 3))
        self.assertAlmostEqual(shapes[1]["maximum"][1], 1.675)
        self.assertAlmostEqual(shapes[0]["minimum"][1], 1.08)
        self.assertEqual((descriptor, frame), before)

    def test_composes_body_and_collider_rotation_and_translation(self):
        descriptor, _ = body_fixture()
        box = descriptor["bodies"][0]["colliders"][0]
        box["local_rotation_q1_30"] = [0, 0, 759250125, 759250125]
        body_rotation = np.diag([-1.0, -1.0, 1.0])
        _, low, high = collider_geometry(box, [2, 3, 4], body_rotation)
        np.testing.assert_allclose((low + high) / 2, [2, 2.7, 4])
        np.testing.assert_allclose((high - low) / 2, [0.22, 0.16, 0.12])

    def test_initial_world_poses_are_not_composed_as_parent_local_poses(self):
        descriptor, frame = body_fixture()
        for body in descriptor["bodies"][1:]:
            body["parent_body_slot"] = 0
        initial, _ = physical_geometry(descriptor)
        native, _ = physical_geometry(descriptor, frame)
        for a, b in zip(initial, native, strict=True):
            np.testing.assert_allclose(a["segments"], b["segments"], atol=1e-8)

    def test_native_token_matching_is_order_independent(self):
        descriptor, frame = body_fixture()
        expected, _ = physical_geometry(descriptor, frame)
        frame["links"].reverse()
        actual, _ = physical_geometry(descriptor, frame)
        for a, b in zip(expected, actual, strict=True):
            np.testing.assert_array_equal(a["segments"], b["segments"])

    def test_invalid_shapes_and_poses_reject_without_inventing_geometry(self):
        for case in (
            "unsupported",
            "size",
            "rotation",
            "duplicate",
            "missing",
            "carrier",
        ):
            descriptor, frame = body_fixture()
            collider = descriptor["bodies"][0]["colliders"][0]
            if case == "unsupported":
                collider["geometry"]["kind"] = "convex"
            elif case == "size":
                collider["geometry"]["half_extents_micrometres"][0] = -1
            elif case == "rotation":
                frame["links"][0]["rotation_q1_30"] = [0, 0, 0, 0]
            elif case == "duplicate":
                frame["links"].append(copy.deepcopy(frame["links"][0]))
            elif case == "missing":
                frame["links"].pop()
            elif case == "carrier":
                descriptor["bodies"][0]["non_colliding_carrier"] = True
            with self.subTest(case=case), self.assertRaises(ValueError):
                physical_geometry(descriptor, frame)

    def test_proportions_use_actual_stature_and_anatomical_joint_locations(self):
        descriptor, _ = body_fixture()
        for side in ("left", "right"):
            for name, height in (
                ("hip-pitch", 0.865),
                ("knee", 0.457),
                ("ankle-pitch", 0.06),
                ("shoulder-pitch", 1.397),
            ):
                descriptor["bodies"].append(
                    {
                        "body_id": f"body.{side}-{name}",
                        "body_token": 2000 + len(descriptor["bodies"]),
                        "initial_translation_f32_bits": np.asarray(
                            [0, height, 0], dtype="<f4"
                        )
                        .view("<u4")
                        .tolist(),
                        "initial_rotation_f32_bits": [0, 0, 0, 1065353216],
                        "colliders": [],
                        "non_colliding_carrier": True,
                    }
                )
        report = neutral_proportions(descriptor)
        self.assertAlmostEqual(report["stature_m"], 1.675)
        self.assertEqual(
            report["sides"]["left"]["hip_to_shoulder_vertical_m"],
            float(np.float32(1.397)) - float(np.float32(0.865)),
        )
        self.assertEqual(report["status"], "report_only")


if __name__ == "__main__":
    unittest.main()
