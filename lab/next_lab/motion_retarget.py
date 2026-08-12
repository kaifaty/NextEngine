from __future__ import annotations

from dataclasses import dataclass, replace
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.cmu_motion import AmcFrame, AsfSkeleton, SourcePose, source_forward_kinematics
from next_lab.motion_math import (
    FloatArray,
    axis_angle_matrix,
    collider_minimum_y,
    matrix_to_quaternion,
    quaternion_q1_30,
    quaternion_to_matrix,
    rotation_axis,
    target_center_of_mass,
    target_effectors,
    target_forward_kinematics,
)


CONTACT_IDS = (
    "contact.left-sole",
    "contact.right-sole",
    "contact.left-palm",
    "contact.right-palm",
    "contact.left-knee",
    "contact.right-knee",
    "contact.body",
)

SOURCE_OVERLAY_BONES = (
    "lfemur",
    "ltibia",
    "lfoot",
    "rfemur",
    "rtibia",
    "rfoot",
    "lowerback",
    "thorax",
    "lhumerus",
    "lradius",
    "rhumerus",
    "rradius",
    "head",
)


@dataclass(frozen=True)
class RetargetedClip:
    clip_id: str
    source_clip_id: str
    skill: str
    partition: str
    split: str
    split_group_id: str
    source_frames: NDArray[np.int64]
    root_position_um: NDArray[np.int64]
    root_quaternion_q1_30: NDArray[np.int64]
    root_linear_velocity_um_s: NDArray[np.int64]
    root_yaw_urad: NDArray[np.int64]
    root_yaw_velocity_urad_s: NDArray[np.int64]
    joint_position_urad: NDArray[np.int64]
    joint_velocity_urad_s: NDArray[np.int64]
    center_of_mass_um: NDArray[np.int64]
    effector_ids: tuple[str, ...]
    effector_position_um: NDArray[np.int64]
    contacts: NDArray[np.uint8]
    phase_u16: NDArray[np.uint16]
    ground_correction_um: NDArray[np.int64]
    minimum_collider_height_um: NDArray[np.int64]
    minimum_nonfoot_height_um: NDArray[np.int64]
    raw_soft_rom_excess_urad: NDArray[np.int64]
    locomotion_collision_projection_urad: NDArray[np.int64]
    velocity_projection_urad: NDArray[np.int64]
    source_overlay_bones: tuple[str, ...]
    source_overlay_position_um: NDArray[np.int64]
    loop: bool
    coverage_class: str = ""
    mirrored_from: str | None = None
    derived_from: str | None = None
    derivation: str | None = None


def retarget_clip(
    *,
    clip: dict[str, Any],
    skeleton: AsfSkeleton,
    frames: tuple[AmcFrame, ...],
    descriptor: dict[str, Any],
    profile: dict[str, Any],
) -> RetargetedClip:
    first = int(clip["source_first_frame"])
    last = int(clip["source_last_frame"])
    selected = tuple(frame for frame in frames if first <= frame.source_frame <= last)
    if not selected or selected[0].source_frame != first or selected[-1].source_frame != last:
        raise ValueError(f"clip {clip['clip_id']} crop is outside the source frame range")
    selected = selected[::2]
    if len(selected) < 2:
        raise ValueError(f"clip {clip['clip_id']} has fewer than two 60 Hz frames")

    poses = tuple(source_forward_kinematics(skeleton, frame) for frame in selected)
    source_root = np.stack([pose.root_position for pose in poses])
    source_root[:, 0] -= source_root[0, 0]
    source_root[:, 2] -= source_root[0, 2]
    root_rotations = _root_rotations(clip["skill"], source_root, poses)
    solved_frames: list[FloatArray] = []
    collision_projections: list[FloatArray] = []
    previous: FloatArray | None = None
    for frame, pose, root_rotation in zip(selected, poses, root_rotations, strict=True):
        previous, collision_projection = _solve_target_joints(
            descriptor,
            frame,
            pose,
            root_rotation,
            previous,
            profile["retarget"].get("locomotion_collision_projection")
            if clip["partition"] == "locomotion"
            else None,
        )
        solved_frames.append(previous)
        collision_projections.append(collision_projection)
    raw_joint_positions = np.stack(solved_frames)
    soft_minimum, soft_maximum = _soft_limits(descriptor)
    raw_urad = np.rint(raw_joint_positions * 1_000_000.0).astype(np.int64)
    soft_clamped_urad = np.maximum(soft_minimum, np.minimum(soft_maximum, raw_urad))
    velocity_limit_basis_points = int(
        profile["retarget"].get("joint_velocity_limit_basis_points", 10_000)
    )
    clamped_urad = _project_joint_velocity(
        soft_clamped_urad,
        descriptor,
        60,
        velocity_limit_basis_points=velocity_limit_basis_points,
    )
    velocity_projection = np.abs(clamped_urad - soft_clamped_urad)
    raw_excess = np.maximum(soft_minimum - raw_urad, np.maximum(raw_urad - soft_maximum, 0))
    collision_projection_urad = np.rint(
        np.stack(collision_projections) * 1_000_000.0
    ).astype(np.int64)

    frame_count = len(selected)
    root_positions = source_root.copy()
    root_quaternions = np.stack([matrix_to_quaternion(rotation) for rotation in root_rotations])
    body_positions = np.empty((frame_count, len(descriptor["bodies"]), 3), dtype=np.float64)
    body_rotations = np.empty((frame_count, len(descriptor["bodies"]), 3, 3), dtype=np.float64)
    required_correction = np.zeros(frame_count, dtype=np.float64)
    minimum_collider_height = np.zeros(frame_count, dtype=np.float64)
    minimum_nonfoot_height = np.zeros(frame_count, dtype=np.float64)
    correction_profile = profile["retarget"]["ground_correction"]
    correction_field = (
        "locomotion_maximum_absolute_micrometres"
        if clip["partition"] == "locomotion"
        else "recovery_maximum_absolute_micrometres"
    )
    maximum_correction = int(correction_profile[correction_field]) / 1_000_000.0
    for index in range(frame_count):
        positions, rotations = target_forward_kinematics(
            descriptor,
            root_positions[index],
            root_quaternions[index],
            clamped_urad[index].astype(np.float64) / 1_000_000.0,
        )
        collider_heights = [
            (
                int(collider["contact_role"]),
                collider_minimum_y(
                    positions[int(body["body_slot"])],
                    rotations[int(body["body_slot"])],
                    collider,
                ),
            )
            for body in descriptor["bodies"]
            for collider in body["colliders"]
        ]
        minimum_y = min(height for _, height in collider_heights)
        alignment_y = (
            min(height for role, height in collider_heights if role == 8)
            if clip["partition"] == "locomotion"
            else minimum_y
        )
        required_correction[index] = -alignment_y
        applied = float(np.clip(required_correction[index], -maximum_correction, maximum_correction))
        root_positions[index, 1] += applied
        positions[:, 1] += applied
        minimum_collider_height[index] = minimum_y + applied
        minimum_nonfoot_height[index] = min(
            height + applied for role, height in collider_heights if role != 8
        )
        body_positions[index] = positions
        body_rotations[index] = rotations

    effector_ids = tuple(sorted(effector["effector_id"] for effector in descriptor["effectors"]))
    effector_positions = np.empty((frame_count, len(effector_ids), 3), dtype=np.float64)
    centers = np.empty((frame_count, 3), dtype=np.float64)
    for index in range(frame_count):
        effectors = target_effectors(descriptor, body_positions[index], body_rotations[index])
        effector_positions[index] = np.stack([effectors[name] for name in effector_ids])
        centers[index] = target_center_of_mass(descriptor, body_positions[index], body_rotations[index])

    source_overlay = _source_overlay(poses, root_positions, root_rotations)
    joint_velocity = _velocity(clamped_urad, 60)
    root_position_um = np.rint(root_positions * 1_000_000.0).astype(np.int64)
    root_velocity = _velocity(root_position_um, 60)
    root_yaw = np.rint(np.unwrap(np.asarray([_yaw(rotation) for rotation in root_rotations])) * 1_000_000.0).astype(np.int64)
    root_yaw_velocity = _velocity(root_yaw[:, None], 60)[:, 0]
    effector_um = np.rint(effector_positions * 1_000_000.0).astype(np.int64)
    contacts = _detect_contacts(
        descriptor=descriptor,
        body_positions=body_positions,
        body_rotations=body_rotations,
        effector_ids=effector_ids,
        effector_positions_um=effector_um,
        thresholds=profile["retarget"]["contact_thresholds"],
    )
    phase = np.rint(np.linspace(0.0, 65535.0, frame_count)).astype(np.uint16)
    return RetargetedClip(
        clip_id=clip["clip_id"],
        source_clip_id=f"cmu-{clip['subject']}-{clip['trial']}",
        skill=clip["skill"],
        partition=clip["partition"],
        split=clip["split"],
        split_group_id=clip["split_group_id"],
        source_frames=np.asarray([frame.source_frame for frame in selected], dtype=np.int64),
        root_position_um=root_position_um,
        root_quaternion_q1_30=np.stack([quaternion_q1_30(rotation) for rotation in root_rotations]),
        root_linear_velocity_um_s=root_velocity,
        root_yaw_urad=root_yaw,
        root_yaw_velocity_urad_s=root_yaw_velocity,
        joint_position_urad=clamped_urad,
        joint_velocity_urad_s=joint_velocity,
        center_of_mass_um=np.rint(centers * 1_000_000.0).astype(np.int64),
        effector_ids=effector_ids,
        effector_position_um=effector_um,
        contacts=contacts,
        phase_u16=phase,
        ground_correction_um=np.rint(np.clip(required_correction, -maximum_correction, maximum_correction) * 1_000_000.0).astype(np.int64),
        minimum_collider_height_um=np.rint(minimum_collider_height * 1_000_000.0).astype(np.int64),
        minimum_nonfoot_height_um=np.rint(minimum_nonfoot_height * 1_000_000.0).astype(np.int64),
        raw_soft_rom_excess_urad=raw_excess,
        locomotion_collision_projection_urad=collision_projection_urad,
        velocity_projection_urad=velocity_projection,
        source_overlay_bones=SOURCE_OVERLAY_BONES,
        source_overlay_position_um=source_overlay,
        loop=bool(clip["loop"]),
        coverage_class=clip["coverage_class"],
    )


def mirror_clip(clip: RetargetedClip, descriptor: dict[str, Any], clip_id: str) -> RetargetedClip:
    joint_ids = [None] * len(descriptor["joints"])
    for joint in descriptor["joints"]:
        joint_ids[int(joint["dof_ordinal"])] = joint["joint_id"]
    joint_lookup = {name: index for index, name in enumerate(joint_ids)}
    source_indices: list[int] = []
    signs: list[int] = []
    for name in joint_ids:
        if ".left-" in name:
            source_indices.append(joint_lookup[name.replace(".left-", ".right-")])
            signs.append(1)
        elif ".right-" in name:
            source_indices.append(joint_lookup[name.replace(".right-", ".left-")])
            signs.append(1)
        else:
            source_indices.append(joint_lookup[name])
            signs.append(-1 if name in {"joint.torso-roll", "joint.torso-yaw"} else 1)
    indices = np.asarray(source_indices, dtype=np.int64)
    sign_array = np.asarray(signs, dtype=np.int64)

    root_position = clip.root_position_um.copy()
    root_position[:, 0] *= -1
    root_velocity = clip.root_linear_velocity_um_s.copy()
    root_velocity[:, 0] *= -1
    root_quaternion = clip.root_quaternion_q1_30.copy()
    root_quaternion[:, 1:3] *= -1
    center = clip.center_of_mass_um.copy()
    center[:, 0] *= -1
    effectors = clip.effector_position_um.copy()
    effectors[:, :, 0] *= -1
    effector_indices = _mirror_name_indices(clip.effector_ids)
    effectors = effectors[:, effector_indices]
    contacts = clip.contacts.copy()
    contacts[:, [0, 1]] = contacts[:, [1, 0]]
    contacts[:, [2, 3]] = contacts[:, [3, 2]]
    contacts[:, [4, 5]] = contacts[:, [5, 4]]
    source_overlay = clip.source_overlay_position_um.copy()
    source_overlay[:, :, 0] *= -1
    return replace(
        clip,
        clip_id=clip_id,
        skill=_mirrored_skill(clip.skill),
        root_position_um=root_position,
        root_quaternion_q1_30=root_quaternion,
        root_linear_velocity_um_s=root_velocity,
        root_yaw_urad=-clip.root_yaw_urad,
        root_yaw_velocity_urad_s=-clip.root_yaw_velocity_urad_s,
        joint_position_urad=clip.joint_position_urad[:, indices] * sign_array,
        joint_velocity_urad_s=clip.joint_velocity_urad_s[:, indices] * sign_array,
        raw_soft_rom_excess_urad=clip.raw_soft_rom_excess_urad[:, indices],
        locomotion_collision_projection_urad=(
            clip.locomotion_collision_projection_urad[:, indices]
        ),
        velocity_projection_urad=clip.velocity_projection_urad[:, indices],
        center_of_mass_um=center,
        effector_position_um=effectors,
        contacts=contacts,
        source_overlay_position_um=source_overlay,
        mirrored_from=clip.clip_id,
        derived_from=clip.clip_id,
        derivation="sagittal-mirror",
    )


def rotate_clip_quarter_yaw(
    clip: RetargetedClip,
    *,
    clip_id: str,
    skill: str,
    quarter_turns: int,
) -> RetargetedClip:
    turns = quarter_turns % 4
    if turns == 0:
        raise ValueError("derived yaw rotation must be non-zero")

    def rotate_vectors(values: NDArray[np.int64]) -> NDArray[np.int64]:
        result = values.copy()
        x = values[..., 0]
        z = values[..., 2]
        if turns == 1:
            result[..., 0], result[..., 2] = z, -x
        elif turns == 2:
            result[..., 0], result[..., 2] = -x, -z
        else:
            result[..., 0], result[..., 2] = -z, x
        return result

    yaw_rotation = rotation_axis("Y", turns * np.pi / 2.0)
    root_quaternion = np.stack(
        [
            quaternion_q1_30(
                yaw_rotation
                @ quaternion_to_matrix(quaternion.astype(np.float64) / float(1 << 30))
            )
            for quaternion in clip.root_quaternion_q1_30
        ]
    )
    yaw_offset = int(round(turns * np.pi / 2.0 * 1_000_000.0))
    return replace(
        clip,
        clip_id=clip_id,
        skill=skill,
        root_position_um=rotate_vectors(clip.root_position_um),
        root_quaternion_q1_30=root_quaternion,
        root_linear_velocity_um_s=rotate_vectors(clip.root_linear_velocity_um_s),
        root_yaw_urad=clip.root_yaw_urad + yaw_offset,
        center_of_mass_um=rotate_vectors(clip.center_of_mass_um),
        effector_position_um=rotate_vectors(clip.effector_position_um),
        source_overlay_position_um=rotate_vectors(clip.source_overlay_position_um),
        mirrored_from=None,
        derived_from=clip.clip_id,
        derivation=f"world-yaw-quarter-turns:{turns}",
    )


def canonical_integer_arrays(clip: RetargetedClip) -> dict[str, NDArray[Any]]:
    return {
        "center_of_mass_um": clip.center_of_mass_um,
        "contacts": clip.contacts,
        "effector_position_um": clip.effector_position_um,
        "ground_correction_um": clip.ground_correction_um,
        "joint_position_urad": clip.joint_position_urad,
        "joint_velocity_urad_s": clip.joint_velocity_urad_s,
        "minimum_collider_height_um": clip.minimum_collider_height_um,
        "minimum_nonfoot_height_um": clip.minimum_nonfoot_height_um,
        "locomotion_collision_projection_urad": clip.locomotion_collision_projection_urad,
        "phase_u16": clip.phase_u16,
        "raw_soft_rom_excess_urad": clip.raw_soft_rom_excess_urad,
        "root_linear_velocity_um_s": clip.root_linear_velocity_um_s,
        "root_position_um": clip.root_position_um,
        "root_quaternion_q1_30": clip.root_quaternion_q1_30,
        "root_yaw_urad": clip.root_yaw_urad,
        "root_yaw_velocity_urad_s": clip.root_yaw_velocity_urad_s,
        "source_frames": clip.source_frames,
        "source_overlay_position_um": clip.source_overlay_position_um,
        "velocity_projection_urad": clip.velocity_projection_urad,
    }


def _solve_target_joints(
    descriptor: dict[str, Any],
    frame: AmcFrame,
    pose: SourcePose,
    root_rotation: FloatArray,
    previous: FloatArray | None,
    locomotion_collision_projection: dict[str, Any] | None,
) -> tuple[FloatArray, FloatArray]:
    target = np.zeros(len(descriptor["joints"]), dtype=np.float64)
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}

    def set_value(joint_id: str, value_degrees: float) -> None:
        target[int(by_id[joint_id]["dof_ordinal"])] = np.deg2rad(value_degrees)

    lowerback = frame.channels["lowerback"]
    upperback = frame.channels["upperback"]
    thorax = frame.channels["thorax"]
    set_value("joint.torso-pitch", lowerback[0] + upperback[0] + thorax[0])
    set_value("joint.torso-roll", -(lowerback[2] + upperback[2] + thorax[2]))
    set_value("joint.torso-yaw", -(lowerback[1] + upperback[1] + thorax[1]))

    for side, source_prefix in (("left", "l"), ("right", "r")):
        femur = frame.channels[f"{source_prefix}femur"]
        humerus = frame.channels[f"{source_prefix}humerus"]
        clavicle = frame.channels[f"{source_prefix}clavicle"]
        knee = frame.channels[f"{source_prefix}tibia"][0]
        elbow = frame.channels[f"{source_prefix}radius"][0]
        foot = frame.channels[f"{source_prefix}foot"]
        side_sign = 1.0 if side == "left" else -1.0
        set_value(f"joint.{side}-hip-pitch", femur[0])
        set_value(f"joint.{side}-hip-roll", side_sign * femur[2])
        set_value(f"joint.{side}-hip-yaw", side_sign * femur[1])
        set_value(f"joint.{side}-knee", knee)
        set_value(f"joint.{side}-ankle-pitch", foot[0])
        set_value(f"joint.{side}-ankle-roll", side_sign * foot[1])
        set_value(f"joint.{side}-shoulder-pitch", humerus[0])
        set_value(
            f"joint.{side}-shoulder-roll",
            side_sign * humerus[2] - 90.0 + side_sign * clavicle[1],
        )
        set_value(f"joint.{side}-shoulder-yaw", side_sign * humerus[1])
        set_value(f"joint.{side}-elbow", elbow)
        _fit_leg(
            descriptor=descriptor,
            target=target,
            pose=pose,
            root_rotation=root_rotation,
            side=side,
            source_prefix=source_prefix,
            previous=previous,
        )
        _fit_arm(
            descriptor=descriptor,
            target=target,
            pose=pose,
            root_rotation=root_rotation,
            side=side,
            source_prefix=source_prefix,
            previous=previous,
        )
    collision_projection = np.zeros_like(target)
    if locomotion_collision_projection is not None:
        for side in ("left", "right"):
            hip_yaw_ordinal = int(by_id[f"joint.{side}-hip-yaw"]["dof_ordinal"])
            hip_roll_ordinal = int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ankle_pitch_ordinal = int(
                by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"]
            )
            ankle_roll_ordinal = int(
                by_id[f"joint.{side}-ankle-roll"]["dof_ordinal"]
            )
            knee_ordinal = int(by_id[f"joint.{side}-knee"]["dof_ordinal"])
            elbow_ordinal = int(by_id[f"joint.{side}-elbow"]["dof_ordinal"])
            shoulder_roll_ordinal = int(
                by_id[f"joint.{side}-shoulder-roll"]["dof_ordinal"]
            )
            hip_yaw = int(locomotion_collision_projection["hip_yaw_microradians"])
            hip_roll_minimum = int(
                locomotion_collision_projection["hip_roll_minimum_microradians"]
            )
            ankle_pitch_minimum = int(
                locomotion_collision_projection[
                    "ankle_pitch_minimum_microradians"
                ]
            )
            ankle_roll_minimum = int(
                locomotion_collision_projection[
                    "ankle_roll_minimum_microradians"
                ]
            )
            ankle_roll_maximum = int(
                locomotion_collision_projection[
                    "ankle_roll_maximum_microradians"
                ]
            )
            knee_minimum = int(
                locomotion_collision_projection["knee_minimum_microradians"]
            )
            elbow_minimum = int(
                locomotion_collision_projection["elbow_minimum_microradians"]
            )
            shoulder_roll_minimum = int(
                locomotion_collision_projection[
                    "shoulder_roll_minimum_microradians"
                ]
            )
            projected_hip_yaw = hip_yaw / 1_000_000.0
            projected_hip_roll = max(
                target[hip_roll_ordinal], hip_roll_minimum / 1_000_000.0
            )
            projected_ankle_pitch = max(
                target[ankle_pitch_ordinal], ankle_pitch_minimum / 1_000_000.0
            )
            projected_ankle_roll = float(
                np.clip(
                    target[ankle_roll_ordinal],
                    ankle_roll_minimum / 1_000_000.0,
                    ankle_roll_maximum / 1_000_000.0,
                )
            )
            projected_knee = max(
                target[knee_ordinal], knee_minimum / 1_000_000.0
            )
            projected_elbow = max(
                target[elbow_ordinal], elbow_minimum / 1_000_000.0
            )
            projected_shoulder_roll = max(
                target[shoulder_roll_ordinal], shoulder_roll_minimum / 1_000_000.0
            )
            collision_projection[hip_yaw_ordinal] = abs(
                target[hip_yaw_ordinal] - projected_hip_yaw
            )
            collision_projection[hip_roll_ordinal] = abs(
                target[hip_roll_ordinal] - projected_hip_roll
            )
            collision_projection[ankle_pitch_ordinal] = abs(
                target[ankle_pitch_ordinal] - projected_ankle_pitch
            )
            collision_projection[ankle_roll_ordinal] = abs(
                target[ankle_roll_ordinal] - projected_ankle_roll
            )
            collision_projection[knee_ordinal] = abs(
                target[knee_ordinal] - projected_knee
            )
            collision_projection[elbow_ordinal] = abs(
                target[elbow_ordinal] - projected_elbow
            )
            collision_projection[shoulder_roll_ordinal] = abs(
                target[shoulder_roll_ordinal] - projected_shoulder_roll
            )
            target[hip_yaw_ordinal] = projected_hip_yaw
            target[hip_roll_ordinal] = projected_hip_roll
            target[ankle_pitch_ordinal] = projected_ankle_pitch
            target[ankle_roll_ordinal] = projected_ankle_roll
            target[knee_ordinal] = projected_knee
            target[elbow_ordinal] = projected_elbow
            target[shoulder_roll_ordinal] = projected_shoulder_roll
    return target, collision_projection


def _fit_leg(
    *,
    descriptor: dict[str, Any],
    target: FloatArray,
    pose: SourcePose,
    root_rotation: FloatArray,
    side: str,
    source_prefix: str,
    previous: FloatArray | None,
) -> None:
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    joint_names = (
        f"joint.{side}-hip-pitch",
        f"joint.{side}-hip-roll",
        f"joint.{side}-hip-yaw",
        f"joint.{side}-knee",
        f"joint.{side}-ankle-pitch",
        f"joint.{side}-ankle-roll",
    )
    ordinals = np.asarray([int(by_id[name]["dof_ordinal"]) for name in joint_names], dtype=np.int64)
    minimum = np.asarray([by_id[name]["soft_limit_microradians"][0] for name in joint_names], dtype=np.float64) / 1_000_000.0
    maximum = np.asarray([by_id[name]["soft_limit_microradians"][1] for name in joint_names], dtype=np.float64) / 1_000_000.0
    reference = np.clip(target[ordinals], minimum, maximum)
    values = np.clip(
        reference if previous is None else previous[ordinals], minimum, maximum
    )
    axes = [np.asarray(by_id[name]["axis_q1_30"], dtype=np.float64) / float(1 << 30) for name in joint_names]
    hip_offset = np.asarray(by_id[joint_names[0]]["parent_frame"]["translation_micrometres"], dtype=np.float64) / 1_000_000.0
    knee_offset = np.asarray(by_id[joint_names[3]]["parent_frame"]["translation_micrometres"], dtype=np.float64) / 1_000_000.0
    ankle_offset = np.asarray(by_id[joint_names[4]]["parent_frame"]["translation_micrometres"], dtype=np.float64) / 1_000_000.0
    ankle_roll_offset = np.asarray(by_id[joint_names[5]]["parent_frame"]["translation_micrometres"], dtype=np.float64) / 1_000_000.0
    forefoot_record = next(
        effector for effector in descriptor["effectors"] if effector["effector_id"] == f"effector.{side}-forefoot"
    )
    forefoot_offset = np.asarray(forefoot_record["local_translation_micrometres"], dtype=np.float64) / 1_000_000.0
    thigh_length = float(np.linalg.norm(knee_offset))
    shank_length = float(np.linalg.norm(ankle_offset))
    source_hip = pose.bone_starts[f"{source_prefix}femur"]
    source_knee = pose.bone_ends[f"{source_prefix}femur"]
    source_ankle = pose.bone_ends[f"{source_prefix}tibia"]
    source_toe = pose.bone_ends[f"{source_prefix}foot"]
    source_to_target = root_rotation @ pose.root_rotation.T
    thigh_direction = source_to_target @ _unit(source_knee - source_hip)
    shank_direction = source_to_target @ _unit(source_ankle - source_knee)
    foot_direction = source_to_target @ _unit(source_toe - source_ankle)

    def leg_points(candidate: FloatArray) -> tuple[FloatArray, FloatArray, FloatArray, FloatArray]:
        hip = root_rotation @ hip_offset
        rotation = root_rotation
        for index in range(3):
            rotation = rotation @ axis_angle_matrix(axes[index], float(candidate[index]))
        knee = hip + rotation @ knee_offset
        rotation = rotation @ axis_angle_matrix(axes[3], float(candidate[3]))
        ankle = knee + rotation @ ankle_offset
        rotation = rotation @ axis_angle_matrix(axes[4], float(candidate[4]))
        ankle_roll = ankle + rotation @ ankle_roll_offset
        rotation = rotation @ axis_angle_matrix(axes[5], float(candidate[5]))
        forefoot = ankle_roll + rotation @ forefoot_offset
        return hip, knee, ankle, forefoot

    def residual(candidate: FloatArray) -> FloatArray:
        hip, knee, ankle, forefoot = leg_points(candidate)
        desired_knee = hip + thigh_direction * thigh_length
        desired_ankle = desired_knee + shank_direction * shank_length
        foot_length = float(np.linalg.norm(ankle_roll_offset + forefoot_offset))
        desired_forefoot = desired_ankle + foot_direction * foot_length
        return np.concatenate(
            (
                knee - desired_knee,
                ankle - desired_ankle,
                forefoot - desired_forefoot,
            )
        )

    for _ in range(7):
        base = residual(values)
        jacobian = np.empty((len(base), len(values)), dtype=np.float64)
        step = 1.0e-4
        for column in range(len(values)):
            perturbed = values.copy()
            perturbed[column] += step
            jacobian[:, column] = (residual(perturbed) - base) / step
        regularization = 2.0e-3 * np.eye(len(values), dtype=np.float64)
        delta = np.linalg.solve(
            jacobian.T @ jacobian + regularization,
            -(jacobian.T @ base) - regularization @ (values - reference),
        )
        values = np.clip(values + np.clip(delta, -0.25, 0.25), minimum, maximum)
    if previous is not None:
        maximum_step = np.asarray(
            [by_id[name]["maximum_velocity_microradians_per_second"] for name in joint_names],
            dtype=np.float64,
        ) / 60_000_000.0
        values = np.clip(values, previous[ordinals] - maximum_step, previous[ordinals] + maximum_step)
    target[ordinals] = values


def _unit(value: FloatArray) -> FloatArray:
    norm = float(np.linalg.norm(value))
    if norm <= 1.0e-9:
        raise ValueError("source motion contains a zero-length semantic limb")
    return value / norm


def _fit_arm(
    *,
    descriptor: dict[str, Any],
    target: FloatArray,
    pose: SourcePose,
    root_rotation: FloatArray,
    side: str,
    source_prefix: str,
    previous: FloatArray | None,
) -> None:
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    joint_names = (
        f"joint.{side}-shoulder-pitch",
        f"joint.{side}-shoulder-roll",
        f"joint.{side}-shoulder-yaw",
        f"joint.{side}-elbow",
    )
    ordinals = np.asarray(
        [int(by_id[name]["dof_ordinal"]) for name in joint_names], dtype=np.int64
    )
    minimum = (
        np.asarray(
            [by_id[name]["soft_limit_microradians"][0] for name in joint_names],
            dtype=np.float64,
        )
        / 1_000_000.0
    )
    maximum = (
        np.asarray(
            [by_id[name]["soft_limit_microradians"][1] for name in joint_names],
            dtype=np.float64,
        )
        / 1_000_000.0
    )
    reference = np.clip(target[ordinals], minimum, maximum)
    values = np.clip(
        reference if previous is None else previous[ordinals], minimum, maximum
    )
    axes = [
        np.asarray(by_id[name]["axis_q1_30"], dtype=np.float64) / float(1 << 30)
        for name in joint_names
    ]
    torso_rotation = root_rotation.copy()
    for torso_name in ("joint.torso-pitch", "joint.torso-roll", "joint.torso-yaw"):
        record = by_id[torso_name]
        axis = np.asarray(record["axis_q1_30"], dtype=np.float64) / float(1 << 30)
        torso_rotation = torso_rotation @ axis_angle_matrix(
            axis, float(target[int(record["dof_ordinal"])])
        )
    elbow_offset = (
        np.asarray(by_id[joint_names[3]]["parent_frame"]["translation_micrometres"], dtype=np.float64)
        / 1_000_000.0
    )
    palm_record = next(
        effector
        for effector in descriptor["effectors"]
        if effector["effector_id"] == f"effector.{side}-palm"
    )
    palm_offset = (
        np.asarray(palm_record["local_translation_micrometres"], dtype=np.float64)
        / 1_000_000.0
    )
    source_shoulder = pose.bone_starts[f"{source_prefix}humerus"]
    source_elbow = pose.bone_ends[f"{source_prefix}humerus"]
    source_wrist = pose.bone_ends[f"{source_prefix}radius"]
    source_to_target = root_rotation @ pose.root_rotation.T
    upper_direction = source_to_target @ _unit(source_elbow - source_shoulder)
    forearm_direction = source_to_target @ _unit(source_wrist - source_elbow)

    def arm_points(candidate: FloatArray) -> tuple[FloatArray, FloatArray]:
        rotation = torso_rotation
        for index in range(3):
            rotation = rotation @ axis_angle_matrix(axes[index], float(candidate[index]))
        elbow = rotation @ elbow_offset
        rotation = rotation @ axis_angle_matrix(axes[3], float(candidate[3]))
        palm = elbow + rotation @ palm_offset
        return elbow, palm

    def residual(candidate: FloatArray) -> FloatArray:
        elbow, palm = arm_points(candidate)
        desired_elbow = upper_direction * float(np.linalg.norm(elbow_offset))
        desired_palm = desired_elbow + forearm_direction * float(np.linalg.norm(palm_offset))
        return np.concatenate((elbow - desired_elbow, palm - desired_palm))

    for _ in range(7):
        base = residual(values)
        jacobian = np.empty((len(base), len(values)), dtype=np.float64)
        step = 1.0e-4
        for column in range(len(values)):
            perturbed = values.copy()
            perturbed[column] += step
            jacobian[:, column] = (residual(perturbed) - base) / step
        regularization = 2.0e-3 * np.eye(len(values), dtype=np.float64)
        delta = np.linalg.solve(
            jacobian.T @ jacobian + regularization,
            -(jacobian.T @ base) - regularization @ (values - reference),
        )
        values = np.clip(values + np.clip(delta, -0.25, 0.25), minimum, maximum)
    if previous is not None:
        maximum_step = np.asarray(
            [by_id[name]["maximum_velocity_microradians_per_second"] for name in joint_names],
            dtype=np.float64,
        ) / 60_000_000.0
        values = np.clip(values, previous[ordinals] - maximum_step, previous[ordinals] + maximum_step)
    target[ordinals] = values


def _soft_limits(descriptor: dict[str, Any]) -> tuple[NDArray[np.int64], NDArray[np.int64]]:
    minimum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    maximum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    for joint in descriptor["joints"]:
        ordinal = int(joint["dof_ordinal"])
        minimum[ordinal], maximum[ordinal] = joint["soft_limit_microradians"]
    return minimum, maximum


def _project_joint_velocity(
    values: NDArray[np.int64],
    descriptor: dict[str, Any],
    rate_hz: int,
    *,
    velocity_limit_basis_points: int = 10_000,
) -> NDArray[np.int64]:
    if not 0 < velocity_limit_basis_points <= 10_000:
        raise ValueError("joint velocity reserve basis points are invalid")
    maximum_step = np.empty(len(descriptor["joints"]), dtype=np.int64)
    for joint in descriptor["joints"]:
        ordinal = int(joint["dof_ordinal"])
        maximum_velocity = (
            int(joint["maximum_velocity_microradians_per_second"])
            * velocity_limit_basis_points
            // 10_000
        )
        maximum_step[ordinal] = maximum_velocity // rate_hz
    result = values.copy()
    for frame_index in range(1, len(result)):
        delta = result[frame_index] - result[frame_index - 1]
        result[frame_index] = result[frame_index - 1] + np.clip(
            delta, -maximum_step, maximum_step
        )
    return result


def _root_rotations(skill: str, root_positions: FloatArray, poses: tuple[SourcePose, ...]) -> FloatArray:
    source = np.stack([pose.root_rotation for pose in poses])
    if not _is_locomotion(skill):
        return source
    if skill in {"neutral_idle", "weight_shift"}:
        source_yaws = np.unwrap(np.asarray([_yaw(rotation) for rotation in source]))
        return np.stack([rotation_axis("Y", yaw) for yaw in source_yaws])
    planar = root_positions[:, (0, 2)]
    differences = np.gradient(planar, axis=0)
    for index in range(len(differences)):
        low = max(0, index - 6)
        high = min(len(differences), index + 7)
        differences[index] = np.mean(differences[low:high], axis=0)
    magnitudes = np.linalg.norm(differences, axis=1)
    valid = magnitudes > 1.0e-6
    if not np.any(valid):
        source_yaws = np.unwrap(np.asarray([_yaw(rotation) for rotation in source]))
        return np.stack([rotation_axis("Y", yaw) for yaw in source_yaws])
    first_valid = int(np.flatnonzero(valid)[0])
    last_yaw = float(np.arctan2(differences[first_valid, 0], differences[first_valid, 1]))
    yaws = np.empty(len(source), dtype=np.float64)
    for index in range(len(source)):
        if valid[index]:
            last_yaw = float(np.arctan2(differences[index, 0], differences[index, 1]))
        yaws[index] = last_yaw
    yaws = np.unwrap(yaws)
    return np.stack([rotation_axis("Y", yaw) for yaw in yaws])


def _is_locomotion(skill: str) -> bool:
    return skill in {"neutral_idle", "weight_shift"} or skill.startswith(
        ("start_", "stop_", "walk_", "turn_")
    )


def _source_overlay(
    poses: tuple[SourcePose, ...], root_positions: FloatArray, root_rotations: FloatArray
) -> NDArray[np.int64]:
    result = np.empty((len(poses), len(SOURCE_OVERLAY_BONES), 3), dtype=np.float64)
    for frame_index, pose in enumerate(poses):
        for bone_index, bone in enumerate(SOURCE_OVERLAY_BONES):
            root_local = pose.root_rotation.T @ (pose.bone_ends[bone] - pose.root_position)
            result[frame_index, bone_index] = (
                root_positions[frame_index] + root_rotations[frame_index] @ root_local
            )
    return np.rint(result * 1_000_000.0).astype(np.int64)


def _velocity(values: NDArray[np.int64], rate_hz: int) -> NDArray[np.int64]:
    result = np.empty_like(values)
    result[0] = (values[1] - values[0]) * rate_hz
    result[-1] = (values[-1] - values[-2]) * rate_hz
    if len(values) > 2:
        differences = values[2:] - values[:-2]
        result[1:-1] = np.rint(differences.astype(np.float64) * (rate_hz / 2.0)).astype(np.int64)
    return result


def _yaw(rotation: FloatArray) -> float:
    forward = rotation @ np.asarray((0.0, 0.0, 1.0), dtype=np.float64)
    return float(np.arctan2(forward[0], forward[2]))


def _detect_contacts(
    *,
    descriptor: dict[str, Any],
    body_positions: FloatArray,
    body_rotations: FloatArray,
    effector_ids: tuple[str, ...],
    effector_positions_um: NDArray[np.int64],
    thresholds: dict[str, Any],
) -> NDArray[np.uint8]:
    effector_lookup = {name: index for index, name in enumerate(effector_ids)}
    effector_speed = np.linalg.norm(_velocity(effector_positions_um, 60).astype(np.float64), axis=2)
    sole_height = int(thresholds["sole_height_micrometres"])
    sole_speed = int(thresholds["sole_speed_micrometres_per_second"])
    recovery_height = int(thresholds["recovery_height_micrometres"])
    recovery_speed = int(thresholds["recovery_speed_micrometres_per_second"])
    contacts = np.zeros((len(body_positions), len(CONTACT_IDS)), dtype=np.uint8)
    for side_index, side in enumerate(("left", "right")):
        heel = effector_lookup[f"effector.{side}-heel"]
        forefoot = effector_lookup[f"effector.{side}-forefoot"]
        height = np.minimum(effector_positions_um[:, heel, 1], effector_positions_um[:, forefoot, 1])
        speed = np.minimum(effector_speed[:, heel], effector_speed[:, forefoot])
        contacts[:, side_index] = ((height <= sole_height) & (speed <= sole_speed)).astype(np.uint8)
        palm = effector_lookup[f"effector.{side}-palm"]
        contacts[:, 2 + side_index] = (
            (effector_positions_um[:, palm, 1] <= recovery_height)
            & (effector_speed[:, palm] <= recovery_speed)
        ).astype(np.uint8)

    body_by_id = {body["body_id"]: body for body in descriptor["bodies"]}
    knee_heights: list[NDArray[np.int64]] = []
    body_heights = np.empty(len(body_positions), dtype=np.int64)
    for frame_index in range(len(body_positions)):
        non_limb_minimum = float("inf")
        frame_knees: list[int] = []
        for body in descriptor["bodies"]:
            slot = int(body["body_slot"])
            for collider in body["colliders"]:
                height = int(round(collider_minimum_y(body_positions[frame_index, slot], body_rotations[frame_index, slot], collider) * 1_000_000.0))
                role = int(collider["contact_role"])
                if role == 6:
                    frame_knees.append(height)
                elif role in {1, 2, 3}:
                    non_limb_minimum = min(non_limb_minimum, height)
        if len(frame_knees) != 2:
            raise ValueError("target descriptor does not expose exactly two knee colliders")
        if frame_index == 0:
            knee_heights = [np.empty(len(body_positions), dtype=np.int64) for _ in frame_knees]
        for index, height in enumerate(frame_knees):
            knee_heights[index][frame_index] = height
        if not np.isfinite(non_limb_minimum):
            raise ValueError("target descriptor does not expose a trunk contact collider")
        body_heights[frame_index] = int(non_limb_minimum)
    knee_speed = [np.abs(_velocity(height[:, None], 60)[:, 0]) for height in knee_heights]
    for side_index in range(2):
        contacts[:, 4 + side_index] = (
            (knee_heights[side_index] <= recovery_height) & (knee_speed[side_index] <= recovery_speed)
        ).astype(np.uint8)
    body_speed = np.abs(_velocity(body_heights[:, None], 60)[:, 0])
    contacts[:, 6] = ((body_heights <= recovery_height) & (body_speed <= recovery_speed)).astype(np.uint8)
    for column in range(contacts.shape[1]):
        contacts[:, column] = _close_single_frame_gaps(contacts[:, column])
    return contacts


def _close_single_frame_gaps(values: NDArray[np.uint8]) -> NDArray[np.uint8]:
    result = values.copy()
    if len(result) > 2:
        result[1:-1] = np.maximum(result[1:-1], result[:-2] & result[2:])
    return result


def _mirror_name_indices(names: tuple[str, ...]) -> NDArray[np.int64]:
    lookup = {name: index for index, name in enumerate(names)}
    indices = []
    for name in names:
        if ".left-" in name:
            indices.append(lookup[name.replace(".left-", ".right-")])
        elif ".right-" in name:
            indices.append(lookup[name.replace(".right-", ".left-")])
        else:
            indices.append(lookup[name])
    return np.asarray(indices, dtype=np.int64)


def _mirrored_skill(skill: str) -> str:
    if "left" in skill:
        return skill.replace("left", "right")
    if "right" in skill:
        return skill.replace("right", "left")
    return skill
