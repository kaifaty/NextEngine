from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_manifold import project_reference_contact_manifold
from next_lab.motion_math import target_effectors, target_forward_kinematics


FIXTURES = Path(__file__).parent / "fixtures"
SCRIPT = Path(__file__).parents[1] / "scripts" / "build_contact_manifold_prototype.py"
SPEC = importlib.util.spec_from_file_location(
    "build_contact_manifold_prototype", SCRIPT
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("contact prototype builder module is unavailable")
builder = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(builder)


class ContactManifoldBuilderTests(unittest.TestCase):
    def test_overlap_identity_requires_exact_shared_values(self) -> None:
        complete = {
            "reference_frame": np.arange(8, dtype=np.int64),
            "root_position_um": np.arange(24, dtype=np.int64).reshape(8, 3),
        }
        cases = [
            {
                "clip_id": "clip",
                "frame_first": 1,
                "frame_last": 5,
                "arrays": {name: values[1:6] for name, values in complete.items()},
            },
            {
                "clip_id": "clip",
                "frame_first": 3,
                "frame_last": 7,
                "arrays": {name: values[3:8] for name, values in complete.items()},
            },
        ]

        self.assertEqual(builder._overlap_identity(cases)["status"], "PASS")
        changed = {name: values.copy() for name, values in cases[1]["arrays"].items()}
        changed["root_position_um"][0, 0] += 1
        cases[1] = {**cases[1], "arrays": changed}
        report = builder._overlap_identity(cases)
        self.assertEqual(report["status"], "FAIL")
        self.assertEqual(report["disagreement_count"], 1)

    def test_clip_projection_case_is_an_exact_full_trajectory_slice(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        frame_count = 6
        joint_position = np.zeros(
            (frame_count, len(descriptor["joints"])), dtype=np.int64
        )
        root_position = np.zeros((frame_count, 3), dtype=np.int64)
        root_position[:, 0] = np.arange(frame_count) * 5_000
        root_position[:, 1] = 943_500
        root_quaternion = np.zeros((frame_count, 4), dtype=np.int64)
        root_quaternion[:, 3] = 1 << 30
        effector_ids = tuple(
            sorted(
                effector["effector_id"]
                for effector in descriptor["effectors"]
            )
        )
        effector_position = np.empty(
            (frame_count, len(effector_ids), 3), dtype=np.int64
        )
        for frame in range(frame_count):
            positions, rotations = target_forward_kinematics(
                descriptor,
                root_position[frame].astype(np.float64) / 1_000_000.0,
                np.asarray((0.0, 0.0, 0.0, 1.0)),
                joint_position[frame].astype(np.float64),
            )
            effectors = target_effectors(descriptor, positions, rotations)
            effector_position[frame] = np.rint(
                np.stack([effectors[name] for name in effector_ids])
                * 1_000_000.0
            ).astype(np.int64)
        root_yaw = np.zeros(frame_count, dtype=np.int64)
        projection = project_reference_contact_manifold(
            descriptor=descriptor,
            effector_ids=effector_ids,
            root_position_um=root_position,
            root_quaternion_q1_30=root_quaternion,
            root_yaw_urad=root_yaw,
            joint_position_urad=joint_position,
            effector_position_um=effector_position,
            contacts=np.zeros((frame_count, 7), dtype=np.uint8),
            support_state=np.zeros(frame_count, dtype=np.int64),
            frame_first=0,
            frame_last=frame_count - 1,
        )

        sliced = builder._slice_clip_projection(
            projection=projection,
            descriptor=descriptor,
            effector_ids=effector_ids,
            root_quaternion_q1_30=root_quaternion,
            source_root_position_um=root_position,
            source_joint_position_urad=joint_position,
            frame_first=1,
            frame_last=4,
            tolerances=builder.ContactManifoldTolerances(),
            collider_closure=None,
        )

        self.assertEqual(sliced.diagnostics["status"], "PASS")
        np.testing.assert_array_equal(
            sliced.root_position_um, projection.root_position_um[1:5]
        )
        np.testing.assert_array_equal(
            sliced.root_linear_velocity_um_s,
            projection.root_linear_velocity_um_s[1:5],
        )
        np.testing.assert_array_equal(
            sliced.joint_velocity_urad_s,
            projection.joint_velocity_urad_s[1:5],
        )
        source_arrays = {
            "phase_u16": np.arange(frame_count, dtype=np.uint16),
            "root_quaternion_q1_30": root_quaternion,
            "root_yaw_urad": root_yaw,
        }
        actual = builder._projection_artifact_arrays(
            projection=sliced,
            source_arrays=source_arrays,
            frame_first=1,
            frame_last=4,
        )
        complete = builder._projection_artifact_arrays(
            projection=projection,
            source_arrays=source_arrays,
            frame_first=0,
            frame_last=5,
        )
        expected = {name: values[1:5] for name, values in complete.items()}
        self.assertTrue(builder._arrays_are_exact(actual, expected))


if __name__ == "__main__":
    unittest.main()
