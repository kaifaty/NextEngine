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
    planar_root_correction_um: NDArray[np.int64] | None = None
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
    planar_root_scale_basis_points = int(
        clip.get("planar_root_scale_basis_points", 10_000)
    )
    if not 0 < planar_root_scale_basis_points <= 10_000:
        raise ValueError(f"clip {clip['clip_id']} planar root scale is invalid")
    source_root[:, (0, 2)] *= planar_root_scale_basis_points / 10_000.0
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
    temporal_contact_profile = (
        profile["retarget"].get("temporal_contact_solve")
        if clip["partition"] == "locomotion"
        else None
    )
    planar_root_correction_um: NDArray[np.int64] | None = None
    support_state: NDArray[np.int64] | None = None
    if temporal_contact_profile is not None:
        soft_clamped_urad = _project_temporal_joint_trajectory(
            values=soft_clamped_urad,
            descriptor=descriptor,
            profile=temporal_contact_profile,
        )
        soft_clamped_urad, support_state = _project_contact_aware_locomotion_joints(
            values=soft_clamped_urad,
            descriptor=descriptor,
            root_positions=source_root,
            root_rotations=root_rotations,
            profile=temporal_contact_profile,
        )
    velocity_limit_basis_points = int(
        profile["retarget"].get("joint_velocity_limit_basis_points", 10_000)
    )
    clamped_urad = _project_joint_velocity(
        soft_clamped_urad,
        descriptor,
        60,
        velocity_limit_basis_points=velocity_limit_basis_points,
    )
    if temporal_contact_profile is not None:
        if support_state is None:
            raise ValueError("temporal support solve did not produce a state")
        planar_root_correction_um = _lock_stance_root_planar(
            descriptor=descriptor,
            joint_position_urad=clamped_urad,
            root_positions=source_root,
            root_rotations=root_rotations,
            support_state=support_state,
            profile=temporal_contact_profile["root_planar"],
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
    applied_correction = np.zeros(frame_count, dtype=np.float64)
    minimum_collider_height = np.zeros(frame_count, dtype=np.float64)
    minimum_nonfoot_height = np.zeros(frame_count, dtype=np.float64)
    correction_profile = profile["retarget"]["ground_correction"]
    correction_field = (
        "locomotion_maximum_absolute_micrometres"
        if clip["partition"] == "locomotion"
        else "recovery_maximum_absolute_micrometres"
    )
    maximum_correction = int(correction_profile[correction_field]) / 1_000_000.0
    uncorrected_body_positions = np.empty_like(body_positions)
    uncorrected_body_rotations = np.empty_like(body_rotations)
    collider_heights_by_frame: list[list[tuple[int, float]]] = []
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
        uncorrected_body_positions[index] = positions
        uncorrected_body_rotations[index] = rotations
        collider_heights_by_frame.append(collider_heights)

    if temporal_contact_profile is not None:
        required_root_height = root_positions[:, 1] + required_correction
        root_height_profile = temporal_contact_profile["root_height"]
        solved_root_height = _lipschitz_majorant(
            required_root_height
            + int(root_height_profile["minimum_clearance_micrometres"])
            / 1_000_000.0,
            maximum_step=(
                int(root_height_profile["maximum_vertical_speed_micrometres_per_second"])
                / 60_000_000.0
            ),
        )
        applied_correction = np.clip(
            solved_root_height - root_positions[:, 1],
            -maximum_correction,
            maximum_correction,
        )
    else:
        applied_correction = np.clip(
            required_correction, -maximum_correction, maximum_correction
        )

    for index in range(frame_count):
        positions = uncorrected_body_positions[index]
        rotations = uncorrected_body_rotations[index]
        collider_heights = collider_heights_by_frame[index]
        minimum_y = min(height for _, height in collider_heights)
        applied = float(applied_correction[index])
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
        ground_correction_um=np.rint(applied_correction * 1_000_000.0).astype(
            np.int64
        ),
        minimum_collider_height_um=np.rint(minimum_collider_height * 1_000_000.0).astype(np.int64),
        minimum_nonfoot_height_um=np.rint(minimum_nonfoot_height * 1_000_000.0).astype(np.int64),
        raw_soft_rom_excess_urad=raw_excess,
        locomotion_collision_projection_urad=collision_projection_urad,
        velocity_projection_urad=velocity_projection,
        source_overlay_bones=SOURCE_OVERLAY_BONES,
        source_overlay_position_um=source_overlay,
        loop=bool(clip["loop"]),
        coverage_class=clip["coverage_class"],
        planar_root_correction_um=planar_root_correction_um,
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
    planar_root_correction = (
        None
        if clip.planar_root_correction_um is None
        else clip.planar_root_correction_um.copy()
    )
    if planar_root_correction is not None:
        planar_root_correction[:, 0] *= -1
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
        planar_root_correction_um=planar_root_correction,
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
    planar_root_correction = (
        None
        if clip.planar_root_correction_um is None
        else rotate_vectors(clip.planar_root_correction_um)
    )
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
        planar_root_correction_um=planar_root_correction,
        mirrored_from=None,
        derived_from=clip.clip_id,
        derivation=f"world-yaw-quarter-turns:{turns}",
    )


def canonical_integer_arrays(clip: RetargetedClip) -> dict[str, NDArray[Any]]:
    arrays = {
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
    if clip.planar_root_correction_um is not None:
        arrays["planar_root_correction_um"] = clip.planar_root_correction_um
    return arrays


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
            unidirectional_reserve = int(
                locomotion_collision_projection[
                    "unidirectional_joint_minimum_hard_reserve_microradians"
                ]
            )
            knee_maximum = (
                int(by_id[f"joint.{side}-knee"]["hard_limit_microradians"][1])
                - unidirectional_reserve
            )
            elbow_maximum = (
                int(by_id[f"joint.{side}-elbow"]["hard_limit_microradians"][1])
                - unidirectional_reserve
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
            projected_knee = float(
                np.clip(
                    target[knee_ordinal],
                    knee_minimum / 1_000_000.0,
                    knee_maximum / 1_000_000.0,
                )
            )
            projected_elbow = float(
                np.clip(
                    target[elbow_ordinal],
                    elbow_minimum / 1_000_000.0,
                    elbow_maximum / 1_000_000.0,
                )
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


def _project_temporal_joint_trajectory(
    *,
    values: NDArray[np.int64],
    descriptor: dict[str, Any],
    profile: dict[str, Any],
) -> NDArray[np.int64]:
    smoothing = profile["smoothing"]
    kernel = tuple(int(value) for value in smoothing["kernel_weights"])
    whole_body_passes = int(smoothing["whole_body_passes"])
    protected_passes = int(smoothing["protected_additional_passes"])
    protected_ordinals = _joint_ordinals(
        descriptor, tuple(smoothing["protected_joint_ids"])
    )
    result = _weighted_temporal_smooth(
        values,
        kernel=kernel,
        passes=whole_body_passes,
    )
    if protected_passes:
        result[:, protected_ordinals] = _weighted_temporal_smooth(
            result[:, protected_ordinals],
            kernel=kernel,
            passes=protected_passes,
        )
    return _apply_declared_joint_bounds(
        result,
        descriptor=descriptor,
        bounds=profile["joint_bounds_microradians"],
    )


def _project_contact_aware_locomotion_joints(
    *,
    values: NDArray[np.int64],
    descriptor: dict[str, Any],
    root_positions: FloatArray,
    root_rotations: FloatArray,
    profile: dict[str, Any],
) -> tuple[NDArray[np.int64], NDArray[np.int64]]:
    support_profile = profile["support_phase"]
    required_root_height = np.empty((len(values), 2), dtype=np.float64)
    sole_bodies = tuple(
        next(
            body
            for body in descriptor["bodies"]
            if body["body_id"] == f"body.{side}-ankle-roll"
        )
        for side in ("left", "right")
    )
    for frame_index in range(len(values)):
        positions, rotations = target_forward_kinematics(
            descriptor,
            root_positions[frame_index],
            matrix_to_quaternion(root_rotations[frame_index]),
            values[frame_index].astype(np.float64) / 1_000_000.0,
        )
        for side_index, body in enumerate(sole_bodies):
            slot = int(body["body_slot"])
            sole_height = min(
                collider_minimum_y(positions[slot], rotations[slot], collider)
                for collider in body["colliders"]
                if int(collider["contact_role"]) == 8
            )
            required_root_height[frame_index, side_index] = (
                root_positions[frame_index, 1] - sole_height
            )

    difference = required_root_height[:, 0] - required_root_height[:, 1]
    double_support_height = (
        int(support_profile["double_support_height_micrometres"])
        / 1_000_000.0
    )
    raw_state = np.where(
        np.abs(difference) <= double_support_height,
        2,
        np.where(difference > 0.0, 0, 1),
    ).astype(np.int64)
    support_state = _minimum_dwell_states(
        raw_state,
        minimum_frames=int(support_profile["minimum_state_frames"]),
    )
    support_weight = np.stack(
        (
            np.logical_or(support_state == 0, support_state == 2),
            np.logical_or(support_state == 1, support_state == 2),
        ),
        axis=1,
    ).astype(np.float64)
    smoothing = profile["smoothing"]
    support_weight = _weighted_temporal_smooth_float(
        support_weight,
        kernel=tuple(int(value) for value in smoothing["kernel_weights"]),
        passes=int(support_profile["transition_smoothing_passes"]),
    )
    support_weight = np.clip(support_weight, 0.0, 1.0)

    result = values.astype(np.float64)
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    for side_index, side in enumerate(("left", "right")):
        stance_weight = support_weight[:, side_index]
        swing_weight = 1.0 - stance_weight
        knee = int(by_id[f"joint.{side}-knee"]["dof_ordinal"])
        hip_roll = int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
        stance_knee = int(support_profile["stance_knee_microradians"])
        stance_hip_roll = int(
            support_profile["stance_hip_roll_microradians"]
        )
        swing_knee_lift = int(
            support_profile["swing_knee_lift_microradians"]
        )
        result[:, knee] = (
            result[:, knee] * swing_weight
            + stance_knee * stance_weight
            + swing_knee_lift * swing_weight
        )
        result[:, hip_roll] = (
            result[:, hip_roll] * swing_weight
            + stance_hip_roll * stance_weight
        )

    result = _level_stance_soles(
        values=result,
        descriptor=descriptor,
        root_positions=root_positions,
        root_rotations=root_rotations,
        support_weight=support_weight,
        iterations=int(support_profile["stance_sole_leveling_iterations"]),
        probe_microradians=int(
            support_profile["stance_sole_leveling_probe_microradians"]
        ),
    )

    result = np.rint(result).astype(np.int64)
    result = _apply_declared_joint_bounds(
        result,
        descriptor=descriptor,
        bounds=profile["joint_bounds_microradians"],
    )
    protected_ordinals = _joint_ordinals(
        descriptor, tuple(smoothing["protected_joint_ids"])
    )
    result[:, protected_ordinals] = _weighted_temporal_smooth(
        result[:, protected_ordinals],
        kernel=tuple(int(value) for value in smoothing["kernel_weights"]),
        passes=int(support_profile["transition_smoothing_passes"]),
    )
    result = _apply_declared_joint_bounds(
        result,
        descriptor=descriptor,
        bounds=profile["joint_bounds_microradians"],
    )
    return result, support_state


def _level_stance_soles(
    *,
    values: FloatArray,
    descriptor: dict[str, Any],
    root_positions: FloatArray,
    root_rotations: FloatArray,
    support_weight: FloatArray,
    iterations: int,
    probe_microradians: int,
) -> FloatArray:
    frame_count = len(values)
    if (
        values.ndim != 2
        or root_positions.shape != (frame_count, 3)
        or root_rotations.shape != (frame_count, 3, 3)
        or support_weight.shape != (frame_count, 2)
        or iterations <= 0
        or probe_microradians <= 0
    ):
        raise ValueError("stance sole-leveling trajectory is invalid")
    result = values.astype(np.float64, copy=True)
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    body_by_id = {body["body_id"]: body for body in descriptor["bodies"]}
    probe = probe_microradians / 1_000_000.0
    for frame_index in range(frame_count):
        for side_index, side in enumerate(("left", "right")):
            weight = float(support_weight[frame_index, side_index])
            if weight <= 1.0e-9:
                continue
            ordinals = np.asarray(
                (
                    int(
                        by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"]
                    ),
                    int(by_id[f"joint.{side}-ankle-roll"]["dof_ordinal"]),
                ),
                dtype=np.int64,
            )
            body_slot = int(
                body_by_id[f"body.{side}-ankle-roll"]["body_slot"]
            )
            radians = result[frame_index] / 1_000_000.0
            original = radians[ordinals].copy()
            solved = radians.copy()
            for _ in range(iterations):
                _, rotations = target_forward_kinematics(
                    descriptor,
                    root_positions[frame_index],
                    matrix_to_quaternion(root_rotations[frame_index]),
                    solved,
                )
                error = rotations[body_slot][(0, 2), 1]
                jacobian = np.empty((2, 2), dtype=np.float64)
                for column, ordinal in enumerate(ordinals):
                    probed = solved.copy()
                    probed[ordinal] += probe
                    _, probed_rotations = target_forward_kinematics(
                        descriptor,
                        root_positions[frame_index],
                        matrix_to_quaternion(root_rotations[frame_index]),
                        probed,
                    )
                    jacobian[:, column] = (
                        probed_rotations[body_slot][(0, 2), 1] - error
                    ) / probe
                delta, *_ = np.linalg.lstsq(jacobian, -error, rcond=None)
                solved[ordinals] += np.clip(delta, -0.5, 0.5)
            result[frame_index, ordinals] = (
                original * (1.0 - weight) + solved[ordinals] * weight
            ) * 1_000_000.0
    return result


def _lock_stance_root_planar(
    *,
    descriptor: dict[str, Any],
    joint_position_urad: NDArray[np.int64],
    root_positions: FloatArray,
    root_rotations: FloatArray,
    support_state: NDArray[np.int64],
    profile: dict[str, Any],
) -> NDArray[np.int64]:
    frame_count = len(joint_position_urad)
    if (
        root_positions.shape != (frame_count, 3)
        or root_rotations.shape != (frame_count, 3, 3)
        or support_state.shape != (frame_count,)
    ):
        raise ValueError("stance root-lock trajectory shape mismatch")
    sole_planar = np.empty((frame_count, 2, 2), dtype=np.float64)
    for frame_index in range(frame_count):
        positions, rotations = target_forward_kinematics(
            descriptor,
            root_positions[frame_index],
            matrix_to_quaternion(root_rotations[frame_index]),
            joint_position_urad[frame_index].astype(np.float64) / 1_000_000.0,
        )
        effectors = target_effectors(descriptor, positions, rotations)
        for side_index, side in enumerate(("left", "right")):
            heel = effectors[f"effector.{side}-heel"]
            forefoot = effectors[f"effector.{side}-forefoot"]
            sole_planar[frame_index, side_index] = (
                (heel + forefoot) / 2.0
            )[(0, 2),]

    correction = _continuous_stance_planar_correction(
        sole_planar=sole_planar,
        support_state=support_state,
        profile=profile,
    )
    root_positions[:, (0, 2)] += correction
    result = np.zeros((frame_count, 3), dtype=np.int64)
    result[:, (0, 2)] = np.rint(correction * 1_000_000.0).astype(np.int64)
    return result


def _continuous_stance_planar_correction(
    *,
    sole_planar: FloatArray,
    support_state: NDArray[np.int64],
    profile: dict[str, Any],
) -> FloatArray:
    frame_count = len(support_state)
    if sole_planar.shape != (frame_count, 2, 2):
        raise ValueError("stance sole trajectory shape mismatch")
    if frame_count == 0 or np.any((support_state < 0) | (support_state > 2)):
        raise ValueError("stance support state is invalid")

    active = np.stack(
        (
            np.logical_or(support_state == 0, support_state == 2),
            np.logical_or(support_state == 1, support_state == 2),
        ),
        axis=1,
    )
    correction_step = np.zeros((frame_count, 2), dtype=np.float64)
    for frame_index in range(1, frame_count):
        shared_support = active[frame_index - 1] & active[frame_index]
        if np.any(shared_support):
            sole_step = (
                sole_planar[frame_index, shared_support]
                - sole_planar[frame_index - 1, shared_support]
            )
            correction_step[frame_index] = -np.mean(sole_step, axis=0)

    correction = np.cumsum(correction_step, axis=0)
    phase = np.linspace(0.0, 1.0, frame_count, dtype=np.float64)[:, None]
    correction -= (
        (1.0 - phase) * correction[0] + phase * correction[-1]
    )
    correction = _weighted_temporal_smooth_float(
        correction,
        kernel=tuple(int(value) for value in profile["kernel_weights"]),
        passes=int(profile["smoothing_passes"]),
    )
    taper_frames = int(profile["endpoint_taper_frames"])
    if taper_frames > 0:
        taper = np.ones(frame_count, dtype=np.float64)
        ramp_length = min(taper_frames, frame_count)
        ramp = 0.5 - 0.5 * np.cos(
            np.linspace(0.0, np.pi, ramp_length, dtype=np.float64)
        )
        taper[:ramp_length] = np.minimum(taper[:ramp_length], ramp)
        taper[-ramp_length:] = np.minimum(taper[-ramp_length:], ramp[::-1])
        correction *= taper[:, None]
    else:
        correction -= (
            (1.0 - phase) * correction[0] + phase * correction[-1]
        )
    maximum_um = int(profile["maximum_absolute_correction_micrometres"])
    maximum = max(0, maximum_um - 1) / 1_000_000.0
    magnitude = np.linalg.norm(correction, axis=1)
    correction *= np.minimum(1.0, maximum / np.maximum(magnitude, 1.0e-12))[:, None]
    if frame_count > 1:
        maximum_speed_um_s = int(
            profile["maximum_speed_micrometres_per_second"]
        )
        maximum_step = max(0, maximum_speed_um_s - 100) / 60_000_000.0
        observed_step = float(
            np.max(np.linalg.norm(np.diff(correction, axis=0), axis=1))
        )
        if observed_step > maximum_step:
            correction *= maximum_step / observed_step
    return correction


def _joint_ordinals(
    descriptor: dict[str, Any], joint_ids: tuple[str, ...]
) -> NDArray[np.int64]:
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    if len(joint_ids) != len(set(joint_ids)) or any(
        joint_id not in by_id for joint_id in joint_ids
    ):
        raise ValueError("temporal retarget joint identity closure mismatch")
    return np.asarray(
        [int(by_id[joint_id]["dof_ordinal"]) for joint_id in joint_ids],
        dtype=np.int64,
    )


def _apply_declared_joint_bounds(
    values: NDArray[np.int64],
    *,
    descriptor: dict[str, Any],
    bounds: dict[str, Any],
) -> NDArray[np.int64]:
    result = values.copy()
    by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    for joint_id, declared in bounds.items():
        if joint_id not in by_id or not isinstance(declared, list) or len(declared) != 2:
            raise ValueError("temporal retarget joint bound identity mismatch")
        minimum, maximum = (int(value) for value in declared)
        if minimum > maximum:
            raise ValueError("temporal retarget joint bound is empty")
        ordinal = int(by_id[joint_id]["dof_ordinal"])
        result[:, ordinal] = np.clip(result[:, ordinal], minimum, maximum)
    return result


def _weighted_temporal_smooth(
    values: NDArray[np.int64], *, kernel: tuple[int, ...], passes: int
) -> NDArray[np.int64]:
    return np.rint(
        _weighted_temporal_smooth_float(
            values.astype(np.float64), kernel=kernel, passes=passes
        )
    ).astype(np.int64)


def _weighted_temporal_smooth_float(
    values: FloatArray, *, kernel: tuple[int, ...], passes: int
) -> FloatArray:
    if (
        not kernel
        or len(kernel) % 2 == 0
        or any(value <= 0 for value in kernel)
        or tuple(reversed(kernel)) != kernel
        or isinstance(passes, bool)
        or passes < 0
    ):
        raise ValueError("invalid temporal smoothing profile")
    result = values.astype(np.float64, copy=True)
    if passes == 0:
        return result
    radius = len(kernel) // 2
    weights = np.asarray(kernel, dtype=np.float64)
    weights /= np.sum(weights)
    for _ in range(passes):
        padding = [(radius, radius), *((0, 0),) * (result.ndim - 1)]
        padded = np.pad(result, padding, mode="edge")
        filtered = np.zeros_like(result)
        for offset, weight in enumerate(weights):
            filtered += weight * padded[offset : offset + len(result)]
        result = filtered
    return result


def _minimum_dwell_states(
    values: NDArray[np.int64], *, minimum_frames: int
) -> NDArray[np.int64]:
    if (
        values.ndim != 1
        or len(values) == 0
        or minimum_frames <= 0
        or np.any((values < 0) | (values > 2))
    ):
        raise ValueError("invalid support-state sequence")
    result = values.copy()
    current = int(result[0])
    candidate = current
    candidate_start = 0
    for index in range(1, len(result)):
        value = int(values[index])
        if value == current:
            candidate = current
            candidate_start = index
        elif value != candidate:
            candidate = value
            candidate_start = index
        elif index - candidate_start + 1 >= minimum_frames:
            result[candidate_start : index + 1] = candidate
            current = candidate
        result[index] = current
    return result


def _lipschitz_majorant(
    values: FloatArray, *, maximum_step: float
) -> FloatArray:
    if values.ndim != 1 or len(values) == 0 or maximum_step <= 0.0:
        raise ValueError("invalid root-height temporal projection")
    result = values.astype(np.float64, copy=True)
    for index in range(1, len(result)):
        result[index] = max(result[index], result[index - 1] - maximum_step)
    for index in range(len(result) - 2, -1, -1):
        result[index] = max(result[index], result[index + 1] - maximum_step)
    return result


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
    interval_profile = thresholds.get("interval_stabilization")
    contacts = np.zeros((len(body_positions), len(CONTACT_IDS)), dtype=np.uint8)
    for side_index, side in enumerate(("left", "right")):
        heel = effector_lookup[f"effector.{side}-heel"]
        forefoot = effector_lookup[f"effector.{side}-forefoot"]
        height = np.minimum(effector_positions_um[:, heel, 1], effector_positions_um[:, forefoot, 1])
        speed = np.minimum(effector_speed[:, heel], effector_speed[:, forefoot])
        if interval_profile is None:
            contacts[:, side_index] = (
                (height <= sole_height) & (speed <= sole_speed)
            ).astype(np.uint8)
        else:
            contacts[:, side_index] = _stabilize_binary_intervals(
                enter=(
                    (height <= int(interval_profile["sole_enter_height_micrometres"]))
                    & (
                        speed
                        <= int(
                            interval_profile[
                                "sole_enter_speed_micrometres_per_second"
                            ]
                        )
                    )
                ),
                retain=(
                    (height <= int(interval_profile["sole_exit_height_micrometres"]))
                    & (
                        speed
                        <= int(
                            interval_profile[
                                "sole_exit_speed_micrometres_per_second"
                            ]
                        )
                    )
                ),
                minimum_on_frames=int(interval_profile["minimum_on_frames"]),
                minimum_off_frames=int(interval_profile["minimum_off_frames"]),
            )
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
        if interval_profile is None:
            contacts[:, column] = _close_single_frame_gaps(contacts[:, column])
        elif column >= 2:
            raw = contacts[:, column] != 0
            contacts[:, column] = _stabilize_binary_intervals(
                enter=raw,
                retain=raw,
                minimum_on_frames=int(interval_profile["minimum_on_frames"]),
                minimum_off_frames=int(interval_profile["minimum_off_frames"]),
            )
    return contacts


def _close_single_frame_gaps(values: NDArray[np.uint8]) -> NDArray[np.uint8]:
    result = values.copy()
    if len(result) > 2:
        result[1:-1] = np.maximum(result[1:-1], result[:-2] & result[2:])
    return result


def _stabilize_binary_intervals(
    *,
    enter: NDArray[np.bool_],
    retain: NDArray[np.bool_],
    minimum_on_frames: int,
    minimum_off_frames: int,
) -> NDArray[np.uint8]:
    if (
        enter.ndim != 1
        or retain.shape != enter.shape
        or len(enter) == 0
        or minimum_on_frames <= 0
        or minimum_off_frames <= 0
        or np.any(enter & ~retain)
    ):
        raise ValueError("invalid contact interval stabilization input")
    result = np.zeros(len(enter), dtype=np.uint8)
    active = False
    candidate_start = 0
    pending = 0
    for index in range(len(result)):
        if active:
            if retain[index]:
                pending = 0
                result[index] = 1
            else:
                if pending == 0:
                    candidate_start = index
                pending += 1
                if pending < minimum_off_frames:
                    result[index] = 1
                else:
                    result[candidate_start : index + 1] = 0
                    active = False
                    pending = 0
        elif enter[index]:
            if pending == 0:
                candidate_start = index
            pending += 1
            if pending >= minimum_on_frames:
                result[candidate_start : index + 1] = 1
                active = True
                pending = 0
        else:
            pending = 0
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
