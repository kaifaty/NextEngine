from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.motion_math import (
    FloatArray,
    matrix_to_quaternion,
    quaternion_to_matrix,
    rotation_axis,
    target_center_of_mass,
    target_effectors,
    target_forward_kinematics,
)


class FootContactMode(IntEnum):
    FLIGHT = 0
    HEEL_STICKING = 1
    FOREFOOT_STICKING = 2
    FLAT_STICKING = 3


CONTACT_MODE_NAMES = tuple(mode.name for mode in FootContactMode)
_SIDES = ("left", "right")
_POINTS = ("heel", "forefoot")
_LEG_SUFFIXES = (
    "hip-pitch",
    "hip-roll",
    "hip-yaw",
    "knee",
    "ankle-pitch",
    "ankle-roll",
)


@dataclass(frozen=True)
class ContactManifoldTolerances:
    maximum_tangential_step_micrometres: int = 2_000
    maximum_normal_step_micrometres: int = 1_000
    maximum_normal_residual_micrometres: int = 5_000
    maximum_mode_inference_height_micrometres: int = 45_000
    maximum_mode_inference_speed_micrometres_per_second: int = 600_000
    maximum_mode_retention_height_micrometres: int = 65_000
    maximum_mode_retention_speed_micrometres_per_second: int = 900_000
    minimum_mode_on_frames: int = 3
    minimum_mode_off_frames: int = 3

    def validate(self) -> None:
        values = (
            self.maximum_tangential_step_micrometres,
            self.maximum_normal_step_micrometres,
            self.maximum_normal_residual_micrometres,
            self.maximum_mode_inference_height_micrometres,
            self.maximum_mode_inference_speed_micrometres_per_second,
            self.maximum_mode_retention_height_micrometres,
            self.maximum_mode_retention_speed_micrometres_per_second,
            self.minimum_mode_on_frames,
            self.minimum_mode_off_frames,
        )
        if any(
            isinstance(value, bool) or not isinstance(value, int) or value <= 0
            for value in values
        ) or (
            self.maximum_mode_inference_height_micrometres
            < self.maximum_normal_residual_micrometres
            or self.maximum_mode_retention_height_micrometres
            < self.maximum_mode_inference_height_micrometres
            or self.maximum_mode_retention_speed_micrometres_per_second
            < self.maximum_mode_inference_speed_micrometres_per_second
        ):
            raise ValueError("contact-manifold tolerances are invalid")


@dataclass(frozen=True)
class ContactManifoldProjection:
    frame_first: int
    frame_last: int
    root_position_um: NDArray[np.int64]
    root_linear_velocity_um_s: NDArray[np.int64]
    root_yaw_velocity_urad_s: NDArray[np.int64]
    joint_position_urad: NDArray[np.int64]
    joint_velocity_urad_s: NDArray[np.int64]
    center_of_mass_um: NDArray[np.int64]
    effector_position_um: NDArray[np.int64]
    contacts: NDArray[np.uint8]
    contact_modes: NDArray[np.uint8]
    diagnostics: dict[str, Any]


def infer_contact_modes(
    *,
    effector_ids: tuple[str, ...],
    effector_position_um: NDArray[np.int64],
    support_state: NDArray[np.int64],
    tolerances: ContactManifoldTolerances,
) -> NDArray[np.uint8]:
    """Freeze one explicit, point-consistent mode per foot and frame."""

    tolerances.validate()
    frame_count = len(effector_position_um)
    if (
        effector_position_um.ndim != 3
        or effector_position_um.shape[2] != 3
        or support_state.shape != (frame_count,)
        or frame_count < 3
        or np.any((support_state < 0) | (support_state > 2))
        or len(effector_ids) != effector_position_um.shape[1]
        or len(effector_ids) != len(set(effector_ids))
    ):
        raise ValueError("contact-mode trajectory shape mismatch")
    indices = _sole_effector_indices(effector_ids)
    sole = effector_position_um[:, indices]
    heights = sole[..., 1]
    speeds = np.linalg.norm(
        _integer_velocity(sole.reshape(frame_count, 4, 3), 60).reshape(
            frame_count, 2, 2, 3
        ),
        axis=3,
    )
    point_contacts = np.zeros((frame_count, 2, 2), dtype=np.bool_)
    for side in range(2):
        for point in range(2):
            point_contacts[:, side, point] = _stabilize_contact_intervals(
                enter=(
                    heights[:, side, point]
                    <= tolerances.maximum_mode_inference_height_micrometres
                )
                & (
                    speeds[:, side, point]
                    <= tolerances.maximum_mode_inference_speed_micrometres_per_second
                ),
                retain=(
                    heights[:, side, point]
                    <= tolerances.maximum_mode_retention_height_micrometres
                )
                & (
                    speeds[:, side, point]
                    <= tolerances.maximum_mode_retention_speed_micrometres_per_second
                ),
                minimum_on_frames=tolerances.minimum_mode_on_frames,
                minimum_off_frames=tolerances.minimum_mode_off_frames,
            )

    modes = np.zeros((frame_count, 2), dtype=np.uint8)
    for frame in range(frame_count):
        state = int(support_state[frame])
        active_sides = (0, 1) if state == 2 else (state,)
        for side in active_sides:
            heel = bool(point_contacts[frame, side, 0])
            forefoot = bool(point_contacts[frame, side, 1])
            if heel and forefoot and abs(
                int(heights[frame, side, 0])
                - int(heights[frame, side, 1])
            ) > 2 * tolerances.maximum_normal_step_micrometres:
                if heights[frame, side, 0] <= heights[frame, side, 1]:
                    forefoot = False
                else:
                    heel = False
            modes[frame, side] = np.uint8(int(heel) + 2 * int(forefoot))
    return modes


def contact_point_mask(contact_modes: NDArray[np.uint8]) -> NDArray[np.bool_]:
    if (
        contact_modes.ndim != 2
        or contact_modes.shape[1] != 2
        or np.any(contact_modes > FootContactMode.FLAT_STICKING)
    ):
        raise ValueError("contact modes are invalid")
    return np.stack(
        (
            (contact_modes & FootContactMode.HEEL_STICKING) != 0,
            (contact_modes & FootContactMode.FOREFOOT_STICKING) != 0,
        ),
        axis=2,
    )


def project_reference_contact_manifold(
    *,
    descriptor: dict[str, Any],
    effector_ids: tuple[str, ...],
    root_position_um: NDArray[np.int64],
    root_quaternion_q1_30: NDArray[np.int64],
    root_yaw_urad: NDArray[np.int64],
    joint_position_urad: NDArray[np.int64],
    effector_position_um: NDArray[np.int64],
    contacts: NDArray[np.uint8],
    support_state: NDArray[np.int64],
    frame_first: int,
    frame_last: int,
    tolerances: ContactManifoldTolerances = ContactManifoldTolerances(),
) -> ContactManifoldProjection:
    """Close one bounded episode window on contact pose and velocity.

    Shared sticking points define a continuous root-link correction.  Its
    normal component is constrained by both the ground-residual interval and
    the per-frame velocity interval.  Joint poses remain unchanged.  Emitted
    root and joint velocities are finite differences of the final pose; their
    analytic point velocity is checked against the same frozen limits.
    """

    tolerances.validate()
    frame_count, joint_count = joint_position_urad.shape
    if (
        frame_count < 3
        or root_position_um.shape != (frame_count, 3)
        or root_quaternion_q1_30.shape != (frame_count, 4)
        or root_yaw_urad.shape != (frame_count,)
        or effector_position_um.shape != (frame_count, len(effector_ids), 3)
        or contacts.ndim != 2
        or contacts.shape[0] != frame_count
        or contacts.shape[1] < 2
        or support_state.shape != (frame_count,)
        or joint_count != len(descriptor.get("joints", ()))
        or isinstance(frame_first, bool)
        or isinstance(frame_last, bool)
        or not isinstance(frame_first, int)
        or not isinstance(frame_last, int)
        or not 0 <= frame_first < frame_last < frame_count
    ):
        raise ValueError("contact-manifold reference shape mismatch")

    full_modes = infer_contact_modes(
        effector_ids=effector_ids,
        effector_position_um=effector_position_um,
        support_state=support_state,
        tolerances=tolerances,
    )
    interval = slice(frame_first, frame_last + 1)
    modes = full_modes[interval].copy()
    active = contact_point_mask(modes)
    sole_indices = _sole_effector_indices(effector_ids)
    sole = effector_position_um[interval][:, sole_indices]
    correction_um, active = _continuous_root_contact_projection(
        sole_position_um=sole,
        active=active,
        maximum_normal_residual_micrometres=(
            tolerances.maximum_normal_residual_micrometres - 1
        ),
        maximum_normal_step_micrometres=(
            tolerances.maximum_normal_step_micrometres
        ),
    )
    modes = (
        active[:, :, 0].astype(np.uint8)
        + 2 * active[:, :, 1].astype(np.uint8)
    )
    solved_root_um = root_position_um[interval] + correction_um
    solved_joint_urad = joint_position_urad[interval].copy()
    solved_roots = solved_root_um.astype(np.float64) / 1_000_000.0
    solved_joints = solved_joint_urad.astype(np.float64) / 1_000_000.0
    quaternions = (
        root_quaternion_q1_30[interval].astype(np.float64) / float(1 << 30)
    )
    solved_effectors, centers = _trajectory_metrics(
        descriptor=descriptor,
        effector_ids=effector_ids,
        root_positions=solved_roots,
        root_quaternions=quaternions,
        joint_positions=solved_joints,
    )
    solved_effector_um = np.rint(solved_effectors * 1_000_000.0).astype(
        np.int64
    )
    root_velocity_um_s = _integer_velocity(solved_root_um, 60)
    joint_velocity_urad_s = _integer_velocity(solved_joint_urad, 60)
    yaw_velocity_urad_s = _integer_velocity(
        root_yaw_urad[interval, None], 60
    )[:, 0]
    solved_contacts = contacts[interval].copy()
    solved_contacts[:, :2] = np.any(active, axis=2).astype(np.uint8)
    diagnostics = contact_manifold_diagnostics(
        descriptor=descriptor,
        effector_ids=effector_ids,
        root_position_um=solved_root_um,
        root_quaternion_q1_30=root_quaternion_q1_30[interval],
        root_linear_velocity_um_s=root_velocity_um_s,
        root_yaw_velocity_urad_s=yaw_velocity_urad_s,
        joint_position_urad=solved_joint_urad,
        joint_velocity_urad_s=joint_velocity_urad_s,
        effector_position_um=solved_effector_um,
        contact_modes=modes,
        tolerances=tolerances,
    )
    correction_step = np.diff(correction_um, axis=0)
    diagnostics.update(
        {
            "frame_first": frame_first,
            "frame_last": frame_last,
            "maximum_root_correction_micrometres": int(
                np.max(np.linalg.norm(correction_um, axis=1))
            ),
            "maximum_root_correction_step_micrometres": int(
                np.max(np.linalg.norm(correction_step, axis=1))
            ),
            "maximum_joint_correction_microradians": 0,
            "mode_counts": {
                CONTACT_MODE_NAMES[value]: int(np.sum(modes == value))
                for value in range(len(CONTACT_MODE_NAMES))
            },
        }
    )
    return ContactManifoldProjection(
        frame_first=frame_first,
        frame_last=frame_last,
        root_position_um=solved_root_um,
        root_linear_velocity_um_s=root_velocity_um_s,
        root_yaw_velocity_urad_s=yaw_velocity_urad_s,
        joint_position_urad=solved_joint_urad,
        joint_velocity_urad_s=joint_velocity_urad_s,
        center_of_mass_um=np.rint(centers * 1_000_000.0).astype(np.int64),
        effector_position_um=solved_effector_um,
        contacts=solved_contacts,
        contact_modes=modes,
        diagnostics=diagnostics,
    )


def contact_manifold_diagnostics(
    *,
    descriptor: dict[str, Any],
    effector_ids: tuple[str, ...],
    root_position_um: NDArray[np.int64],
    root_quaternion_q1_30: NDArray[np.int64],
    root_linear_velocity_um_s: NDArray[np.int64],
    root_yaw_velocity_urad_s: NDArray[np.int64],
    joint_position_urad: NDArray[np.int64],
    joint_velocity_urad_s: NDArray[np.int64],
    effector_position_um: NDArray[np.int64],
    contact_modes: NDArray[np.uint8],
    tolerances: ContactManifoldTolerances,
) -> dict[str, Any]:
    tolerances.validate()
    active = contact_point_mask(contact_modes)
    frame_count = len(contact_modes)
    if (
        root_position_um.shape != (frame_count, 3)
        or root_quaternion_q1_30.shape != (frame_count, 4)
        or root_linear_velocity_um_s.shape != (frame_count, 3)
        or root_yaw_velocity_urad_s.shape != (frame_count,)
        or joint_position_urad.shape != joint_velocity_urad_s.shape
        or joint_position_urad.shape[0] != frame_count
        or effector_position_um.shape != (frame_count, len(effector_ids), 3)
    ):
        raise ValueError("contact-manifold diagnostic shape mismatch")
    sole = effector_position_um[:, _sole_effector_indices(effector_ids)]
    normal_residual = np.abs(sole[..., 1])
    shared = active[1:] & active[:-1]
    step = sole[1:] - sole[:-1]
    tangential_step = np.linalg.norm(step[..., (0, 2)], axis=-1)
    normal_step = np.abs(step[..., 1])
    analytic_velocity = _analytic_active_point_velocities(
        descriptor=descriptor,
        root_positions=root_position_um.astype(np.float64) / 1_000_000.0,
        root_quaternions=(
            root_quaternion_q1_30.astype(np.float64) / float(1 << 30)
        ),
        joint_positions=joint_position_urad.astype(np.float64) / 1_000_000.0,
        root_linear_velocity_um_s=root_linear_velocity_um_s,
        root_yaw_velocity_urad_s=root_yaw_velocity_urad_s,
        joint_velocity_urad_s=joint_velocity_urad_s,
        active=active,
        probe=1.0e-4,
    )
    analytic_tangential_step = (
        np.linalg.norm(analytic_velocity[..., (0, 2)], axis=-1) / 60.0
    )
    analytic_normal_step = np.abs(analytic_velocity[..., 1]) / 60.0

    maximum_normal = int(_masked_max(normal_residual, active))
    maximum_tangent_step = int(np.ceil(_masked_max(tangential_step, shared)))
    maximum_normal_step = int(_masked_max(normal_step, shared))
    maximum_analytic_tangent_step = int(
        np.ceil(_masked_max(analytic_tangential_step, active))
    )
    maximum_analytic_normal_step = int(
        np.ceil(_masked_max(analytic_normal_step, active))
    )
    passed = (
        maximum_normal <= tolerances.maximum_normal_residual_micrometres
        and maximum_tangent_step
        <= tolerances.maximum_tangential_step_micrometres
        and maximum_normal_step <= tolerances.maximum_normal_step_micrometres
        and maximum_analytic_tangent_step
        <= tolerances.maximum_tangential_step_micrometres
        and maximum_analytic_normal_step
        <= tolerances.maximum_normal_step_micrometres
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "active_point_frame_count": int(np.sum(active)),
        "shared_active_point_step_count": int(np.sum(shared)),
        "maximum_normal_residual_micrometres": maximum_normal,
        "maximum_tangential_step_micrometres": maximum_tangent_step,
        "maximum_normal_step_micrometres": maximum_normal_step,
        "maximum_analytic_tangential_step_micrometres": (
            maximum_analytic_tangent_step
        ),
        "maximum_analytic_normal_step_micrometres": (
            maximum_analytic_normal_step
        ),
        "tolerances": {
            "maximum_tangential_step_micrometres": (
                tolerances.maximum_tangential_step_micrometres
            ),
            "maximum_normal_step_micrometres": (
                tolerances.maximum_normal_step_micrometres
            ),
            "maximum_normal_residual_micrometres": (
                tolerances.maximum_normal_residual_micrometres
            ),
        },
    }


def _continuous_root_contact_projection(
    *,
    sole_position_um: NDArray[np.int64],
    active: NDArray[np.bool_],
    maximum_normal_residual_micrometres: int,
    maximum_normal_step_micrometres: int,
) -> tuple[NDArray[np.int64], NDArray[np.bool_]]:
    if (
        sole_position_um.shape != (*active.shape, 3)
        or len(active) < 2
        or maximum_normal_residual_micrometres <= 0
        or maximum_normal_step_micrometres <= 0
    ):
        raise ValueError("root contact-projection trajectory shape mismatch")
    resolved = active.copy()
    correction = np.zeros((len(active), 3), dtype=np.float64)
    for frame in range(len(active)):
        if frame > 0:
            correction[frame] = correction[frame - 1]
        if not np.any(resolved[frame]):
            continue
        previous_active = (
            resolved[frame - 1]
            if frame > 0
            else np.zeros_like(resolved[frame])
        )
        shared = resolved[frame] & previous_active
        if np.any(shared):
            point_step = (
                sole_position_um[frame][shared]
                - sole_position_um[frame - 1][shared]
            ).astype(np.float64)
            correction[frame, (0, 2)] -= np.mean(
                point_step[:, (0, 2)], axis=0
            )
            preferred_normal = correction[frame - 1, 1] - float(
                np.mean(point_step[:, 1])
            )
        else:
            preferred_normal = correction[frame, 1]

        while np.any(resolved[frame]):
            active_heights = sole_position_um[frame][
                resolved[frame], 1
            ].astype(np.float64)
            lower = float(
                np.max(
                    -active_heights - maximum_normal_residual_micrometres
                )
            )
            upper = float(
                np.min(
                    -active_heights + maximum_normal_residual_micrometres
                )
            )
            shared = resolved[frame] & previous_active
            if np.any(shared):
                previous_height = (
                    sole_position_um[frame - 1][shared, 1].astype(np.float64)
                    + correction[frame - 1, 1]
                )
                current_height = sole_position_um[frame][shared, 1].astype(
                    np.float64
                )
                lower = max(
                    lower,
                    float(
                        np.max(
                            previous_height
                            - current_height
                            - maximum_normal_step_micrometres
                        )
                    ),
                )
                upper = min(
                    upper,
                    float(
                        np.min(
                            previous_height
                            - current_height
                            + maximum_normal_step_micrometres
                        )
                    ),
                )
            if lower <= upper:
                correction[frame, 1] = np.clip(
                    preferred_normal, lower, upper
                )
                break
            newly_active = resolved[frame] & ~previous_active
            if np.any(newly_active):
                resolved[frame][newly_active] = False
            elif np.sum(resolved[frame]) > 1:
                selected = np.argwhere(resolved[frame])
                heights = np.asarray(
                    [
                        sole_position_um[frame, int(side), int(point), 1]
                        for side, point in selected
                    ]
                )
                drop_side, drop_point = selected[int(np.argmax(heights))]
                resolved[frame, int(drop_side), int(drop_point)] = False
            else:
                raise ValueError(
                    "active contact point has no pose/velocity interval "
                    f"at frame {frame}"
                )
    return np.rint(correction).astype(np.int64), resolved


def _sole_effector_indices(effector_ids: tuple[str, ...]) -> NDArray[np.int64]:
    lookup = {name: index for index, name in enumerate(effector_ids)}
    required = tuple(
        f"effector.{side}-{point}" for side in _SIDES for point in _POINTS
    )
    if len(lookup) != len(effector_ids) or any(name not in lookup for name in required):
        raise ValueError("sole effector identity closure mismatch")
    return np.asarray(
        [
            [lookup[f"effector.{side}-{point}"] for point in _POINTS]
            for side in _SIDES
        ],
        dtype=np.int64,
    )


def _leg_joint_ordinals(descriptor: dict[str, Any]) -> NDArray[np.int64]:
    lookup = {
        joint["joint_id"]: int(joint["dof_ordinal"])
        for joint in descriptor["joints"]
    }
    required = tuple(
        f"joint.{side}-{suffix}" for side in _SIDES for suffix in _LEG_SUFFIXES
    )
    if len(lookup) != len(descriptor["joints"]) or any(
        name not in lookup for name in required
    ):
        raise ValueError("leg joint identity closure mismatch")
    return np.asarray(
        [
            [lookup[f"joint.{side}-{suffix}"] for suffix in _LEG_SUFFIXES]
            for side in _SIDES
        ],
        dtype=np.int64,
    )


def _trajectory_metrics(
    *,
    descriptor: dict[str, Any],
    effector_ids: tuple[str, ...],
    root_positions: FloatArray,
    root_quaternions: FloatArray,
    joint_positions: FloatArray,
) -> tuple[FloatArray, FloatArray]:
    frame_count = len(root_positions)
    effectors = np.empty((frame_count, len(effector_ids), 3), dtype=np.float64)
    centers = np.empty((frame_count, 3), dtype=np.float64)
    for frame in range(frame_count):
        positions, rotations = target_forward_kinematics(
            descriptor,
            root_positions[frame],
            root_quaternions[frame],
            joint_positions[frame],
        )
        values = target_effectors(descriptor, positions, rotations)
        effectors[frame] = np.stack([values[name] for name in effector_ids])
        centers[frame] = target_center_of_mass(descriptor, positions, rotations)
    return effectors, centers


def _analytic_active_point_velocities(
    *,
    descriptor: dict[str, Any],
    root_positions: FloatArray,
    root_quaternions: FloatArray,
    joint_positions: FloatArray,
    root_linear_velocity_um_s: NDArray[np.int64],
    root_yaw_velocity_urad_s: NDArray[np.int64],
    joint_velocity_urad_s: NDArray[np.int64],
    active: NDArray[np.bool_],
    probe: float,
) -> FloatArray:
    result = np.zeros((*active.shape, 3), dtype=np.float64)
    joint_ordinals = _leg_joint_ordinals(descriptor)
    for frame in range(len(active)):
        positions, rotations = target_forward_kinematics(
            descriptor,
            root_positions[frame],
            root_quaternions[frame],
            joint_positions[frame],
        )
        baseline_effectors = target_effectors(descriptor, positions, rotations)
        for side, point in np.argwhere(active[frame]):
            side_index = int(side)
            point_index = int(point)
            point_name = (
                f"effector.{_SIDES[side_index]}-{_POINTS[point_index]}"
            )
            velocity = root_linear_velocity_um_s[frame].astype(np.float64)
            root_rotation = quaternion_to_matrix(root_quaternions[frame])
            candidate_quaternion = matrix_to_quaternion(
                rotation_axis("Y", probe) @ root_rotation
            )
            candidate_positions, candidate_rotations = target_forward_kinematics(
                descriptor,
                root_positions[frame],
                candidate_quaternion,
                joint_positions[frame],
            )
            candidate_effectors = target_effectors(
                descriptor, candidate_positions, candidate_rotations
            )
            yaw_jacobian_um = (
                candidate_effectors[point_name]
                - baseline_effectors[point_name]
            ) * (1_000_000.0 / probe)
            velocity += yaw_jacobian_um * (
                root_yaw_velocity_urad_s[frame] / 1_000_000.0
            )
            for ordinal in joint_ordinals[side_index]:
                candidate_joint = joint_positions[frame].copy()
                candidate_joint[ordinal] += probe
                candidate_positions, candidate_rotations = target_forward_kinematics(
                    descriptor,
                    root_positions[frame],
                    root_quaternions[frame],
                    candidate_joint,
                )
                candidate_effectors = target_effectors(
                    descriptor, candidate_positions, candidate_rotations
                )
                joint_jacobian_um = (
                    candidate_effectors[point_name]
                    - baseline_effectors[point_name]
                ) * (1_000_000.0 / probe)
                velocity += joint_jacobian_um * (
                    joint_velocity_urad_s[frame, ordinal] / 1_000_000.0
                )
            result[frame, side_index, point_index] = velocity
    return result


def _integer_velocity(
    values: NDArray[np.int64], rate_hz: int
) -> NDArray[np.int64]:
    result = np.empty_like(values)
    result[0] = (values[1] - values[0]) * rate_hz
    result[-1] = (values[-1] - values[-2]) * rate_hz
    result[1:-1] = np.rint(
        (values[2:] - values[:-2]).astype(np.float64) * (rate_hz / 2.0)
    ).astype(np.int64)
    return result


def _stabilize_contact_intervals(
    *,
    enter: NDArray[np.bool_],
    retain: NDArray[np.bool_],
    minimum_on_frames: int,
    minimum_off_frames: int,
) -> NDArray[np.bool_]:
    if (
        enter.ndim != 1
        or retain.shape != enter.shape
        or len(enter) == 0
        or minimum_on_frames <= 0
        or minimum_off_frames <= 0
        or np.any(enter & ~retain)
    ):
        raise ValueError("invalid contact interval stabilization input")
    result = np.zeros(len(enter), dtype=np.bool_)
    active = False
    candidate_start = 0
    pending = 0
    for index in range(len(result)):
        if active:
            if retain[index]:
                pending = 0
                result[index] = True
            else:
                if pending == 0:
                    candidate_start = index
                pending += 1
                if pending < minimum_off_frames:
                    result[index] = True
                else:
                    result[candidate_start : index + 1] = False
                    active = False
                    pending = 0
        elif enter[index]:
            if pending == 0:
                candidate_start = index
            pending += 1
            if pending >= minimum_on_frames:
                result[candidate_start : index + 1] = True
                active = True
                pending = 0
        else:
            pending = 0
    return result


def _masked_max(values: NDArray[Any], mask: NDArray[np.bool_]) -> float:
    return float(np.max(values[mask])) if np.any(mask) else 0.0
