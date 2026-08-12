from __future__ import annotations

import io
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

import numpy as np

from next_lab.cmu_motion import parse_amc, parse_asf, source_forward_kinematics
from next_lab.motion_corpus import (
    deterministic_npz_bytes,
    load_motion_corpus_profile,
    validate_clip,
)
from next_lab.motion_retarget import (
    CONTACT_IDS,
    SOURCE_OVERLAY_BONES,
    RetargetedClip,
    canonical_integer_arrays,
    mirror_clip,
    rotate_clip_quarter_yaw,
    _project_joint_velocity,
)


FIXTURES = Path(__file__).parent / "fixtures"
PROFILES = Path(__file__).parents[1] / "profiles"
CMU_SCALE_METRES = 127 / 2250


class MotionCorpusTests(unittest.TestCase):
    def test_safety_reserve_overlay_is_base_hash_bound(self) -> None:
        profile, profile_bytes = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-safety-reserve.v2.json"
        )
        self.assertTrue(profile_bytes.startswith(b"{"))
        self.assertEqual(
            profile["profile_id"],
            "nextengine.motion-corpus.humanoid-biomechanics-cmu-locomotion-safety-reserve.v2",
        )
        self.assertEqual(
            profile["retarget"]["joint_velocity_limit_basis_points"], 9500
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["knee_minimum_microradians"], 34906)
        self.assertEqual(projection["elbow_minimum_microradians"], 34906)

    def test_joint_velocity_projection_applies_declared_reserve(self) -> None:
        descriptor = {
            "joints": [
                {
                    "dof_ordinal": 0,
                    "maximum_velocity_microradians_per_second": 8_000_000,
                }
            ]
        }
        values = np.asarray([[0], [500_000], [-500_000]], dtype=np.int64)
        projected = _project_joint_velocity(
            values,
            descriptor,
            60,
            velocity_limit_basis_points=9500,
        )
        maximum_step = 8_000_000 * 9500 // 10_000 // 60
        np.testing.assert_array_equal(
            projected,
            np.asarray([[0], [maximum_step], [0]], dtype=np.int64),
        )

    def test_cmu_parser_applies_frozen_units_handedness_and_rate_boundary(self) -> None:
        asf = """
:version 1.10
:units
length 0.45
angle deg
:root
order TX TY TZ RX RY RZ
axis XYZ
position 0 0 0
orientation 0 0 0
:bonedata
begin
id 1
name femur
direction 1 0 0
length 1
axis 0 0 0 XYZ
dof rx
end
:hierarchy
begin
root femur
end
""".strip()
        amc = """
:FULLY-SPECIFIED
:DEGREES
1
root 1 2 3 0 0 0
femur 0
2
root 2 2 3 0 0 0
femur 0
""".strip()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            asf_path = root / "fixture.asf"
            amc_path = root / "fixture.amc"
            asf_path.write_text(asf, encoding="utf-8")
            amc_path.write_text(amc, encoding="utf-8")
            skeleton = parse_asf(
                asf_path,
                expected_length_scale_metres=CMU_SCALE_METRES,
            )
            frames = parse_amc(amc_path)

        self.assertEqual([frame.source_frame for frame in frames], [1, 2])
        pose = source_forward_kinematics(skeleton, frames[0])
        np.testing.assert_allclose(
            pose.root_position,
            np.asarray((-CMU_SCALE_METRES, 2 * CMU_SCALE_METRES, 3 * CMU_SCALE_METRES)),
        )
        np.testing.assert_allclose(
            pose.bone_ends["femur"],
            pose.root_position + np.asarray((-CMU_SCALE_METRES, 0.0, 0.0)),
        )

    def test_cmu_parser_rejects_noncontiguous_frames_and_wrong_scale(self) -> None:
        asf = """
:units
length 0.45
:root
order TX TY TZ RX RY RZ
axis XYZ
:bonedata
begin
name bone
direction 0 1 0
length 1
axis 0 0 0 XYZ
end
:hierarchy
begin
root bone
end
""".strip()
        amc = "1\nroot 0 0 0 0 0 0\n3\nroot 0 0 0 0 0 0\n"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            asf_path = root / "fixture.asf"
            amc_path = root / "fixture.amc"
            asf_path.write_text(asf, encoding="utf-8")
            amc_path.write_text(amc, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "length scale"):
                parse_asf(asf_path, expected_length_scale_metres=1.0)
            with self.assertRaisesRegex(ValueError, "not contiguous"):
                parse_amc(amc_path)

    def test_mirror_twice_restores_every_canonical_integer_array(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        original = _clip(descriptor)
        mirrored = mirror_clip(original, descriptor, "clip.start-left")
        restored = mirror_clip(mirrored, descriptor, original.clip_id)
        for name, expected in canonical_integer_arrays(original).items():
            np.testing.assert_array_equal(canonical_integer_arrays(restored)[name], expected)

    def test_yaw_variant_rotates_world_vectors_and_preserves_group(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        original = _clip(descriptor)
        rotated = rotate_clip_quarter_yaw(
            original,
            clip_id="clip.fall-left",
            skill="brace_safe_fall_left",
            quarter_turns=1,
        )
        expected_root = original.root_position_um.copy()
        expected_root[:, 0] = original.root_position_um[:, 2]
        expected_root[:, 2] = -original.root_position_um[:, 0]
        np.testing.assert_array_equal(rotated.root_position_um, expected_root)
        np.testing.assert_array_equal(rotated.contacts, original.contacts)
        np.testing.assert_array_equal(
            rotated.joint_position_urad, original.joint_position_urad
        )
        self.assertEqual(rotated.split_group_id, original.split_group_id)
        self.assertEqual(rotated.derived_from, original.clip_id)
        self.assertEqual(rotated.derivation, "world-yaw-quarter-turns:1")

    def test_npz_serialization_is_byte_exact_and_has_fixed_metadata(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        clip = _clip(descriptor)
        metadata = {"schema_version": 1, "clip_id": clip.clip_id}
        first = deterministic_npz_bytes(clip, metadata)
        self.assertEqual(first, deterministic_npz_bytes(clip, metadata))
        with zipfile.ZipFile(io.BytesIO(first)) as archive:
            self.assertTrue(all(item.date_time == (1980, 1, 1, 0, 0, 0) for item in archive.infolist()))
            self.assertEqual(
                sorted(item.filename for item in archive.infolist()),
                sorted(f"{name}.npy" for name in (*canonical_integer_arrays(clip), "metadata_json_utf8")),
            )

    def test_locomotion_ankle_roll_audit_blocks_soft_boundary_saturation(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        profile = json.loads(
            (
                Path(__file__).parents[1]
                / "profiles/humanoid-motion-corpus-cmu.v1.json"
            ).read_text(encoding="utf-8")
        )
        clip = _clip(descriptor)
        clip.joint_position_urad.fill(0)
        clip.joint_velocity_urad_s.fill(0)
        ankle_roll_ordinals = [
            int(joint["dof_ordinal"])
            for joint in descriptor["joints"]
            if joint["joint_id"]
            in {"joint.left-ankle-roll", "joint.right-ankle-roll"}
        ]
        clip.joint_position_urad[:, ankle_roll_ordinals] = 261799

        saturated = validate_clip(clip, descriptor, profile)

        self.assertIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_SOFT_BOUNDARY_SATURATION",
            saturated["errors"],
        )
        self.assertIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_HARD_RESERVE_SHORTFALL",
            saturated["errors"],
        )
        self.assertEqual(saturated["metrics"]["ankle_roll_soft_boundary_fraction"], 1.0)
        self.assertEqual(
            saturated["metrics"]["minimum_ankle_roll_hard_reserve_microradians"],
            87267,
        )

        clip.joint_position_urad[:, ankle_roll_ordinals] = 87266
        unidirectional_ordinals = [
            int(joint["dof_ordinal"])
            for joint in descriptor["joints"]
            if joint["joint_id"].endswith(("-knee", "-elbow"))
        ]
        clip.joint_position_urad[:, unidirectional_ordinals] = 17453
        reserved = validate_clip(clip, descriptor, profile)

        self.assertNotIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_SOFT_BOUNDARY_SATURATION",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_HARD_RESERVE_SHORTFALL",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_PROJECTION_MISMATCH",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_UNIDIRECTIONAL_LOWER_SOFT_BOUNDARY_SATURATION",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_UNIDIRECTIONAL_HARD_RESERVE_SHORTFALL",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_UNIDIRECTIONAL_PROJECTION_MISMATCH",
            reserved["errors"],
        )
        self.assertEqual(reserved["metrics"]["ankle_roll_soft_boundary_fraction"], 0.0)
        self.assertEqual(
            reserved["metrics"]["minimum_ankle_roll_hard_reserve_microradians"],
            261800,
        )
        self.assertEqual(
            reserved["metrics"][
                "unidirectional_joint_lower_soft_boundary_fraction"
            ],
            0.0,
        )
        self.assertEqual(
            reserved["metrics"][
                "minimum_unidirectional_joint_hard_reserve_microradians"
            ],
            17453,
        )


def _clip(descriptor: dict) -> RetargetedClip:
    frame_count = 3
    joint_count = len(descriptor["joints"])
    effector_ids = tuple(sorted(item["effector_id"] for item in descriptor["effectors"]))
    effector_count = len(effector_ids)
    joint_position = np.arange(frame_count * joint_count, dtype=np.int64).reshape(
        frame_count, joint_count
    )
    root_quaternion = np.zeros((frame_count, 4), dtype=np.int64)
    root_quaternion[:, 3] = 1 << 30
    return RetargetedClip(
        clip_id="clip.start-right",
        source_clip_id="cmu-104-14",
        skill="start_right_lead",
        partition="locomotion",
        split="train",
        split_group_id="cmu-subject-104-trial-14",
        source_frames=np.asarray((1, 3, 5), dtype=np.int64),
        root_position_um=np.arange(frame_count * 3, dtype=np.int64).reshape(frame_count, 3),
        root_quaternion_q1_30=root_quaternion,
        root_linear_velocity_um_s=np.arange(frame_count * 3, dtype=np.int64).reshape(
            frame_count, 3
        ),
        root_yaw_urad=np.asarray((1, 2, 3), dtype=np.int64),
        root_yaw_velocity_urad_s=np.asarray((4, 5, 6), dtype=np.int64),
        joint_position_urad=joint_position,
        joint_velocity_urad_s=joint_position * 60,
        center_of_mass_um=np.arange(frame_count * 3, dtype=np.int64).reshape(frame_count, 3),
        effector_ids=effector_ids,
        effector_position_um=np.arange(
            frame_count * effector_count * 3, dtype=np.int64
        ).reshape(frame_count, effector_count, 3),
        contacts=np.arange(frame_count * len(CONTACT_IDS), dtype=np.uint8).reshape(
            frame_count, len(CONTACT_IDS)
        ),
        phase_u16=np.asarray((0, 32768, 65535), dtype=np.uint16),
        ground_correction_um=np.asarray((1, -2, 3), dtype=np.int64),
        minimum_collider_height_um=np.asarray((0, 1, 2), dtype=np.int64),
        minimum_nonfoot_height_um=np.asarray((3, 4, 5), dtype=np.int64),
        raw_soft_rom_excess_urad=joint_position + 100,
        locomotion_collision_projection_urad=joint_position + 200,
        velocity_projection_urad=joint_position,
        source_overlay_bones=SOURCE_OVERLAY_BONES,
        source_overlay_position_um=np.arange(
            frame_count * len(SOURCE_OVERLAY_BONES) * 3, dtype=np.int64
        ).reshape(frame_count, len(SOURCE_OVERLAY_BONES), 3),
        loop=False,
    )


if __name__ == "__main__":
    unittest.main()
