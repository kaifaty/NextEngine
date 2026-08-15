from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.motion_math import (
    axis_angle_matrix,
    decode_q1_30,
    quaternion_to_matrix,
)

FloatArray = NDArray[np.float64]
GENERALIZED_WIDTH = 29
ROOT_WIDTH = 6
JOINT_COUNT = 23
ENGINE_GRAVITY = np.asarray((0.0, -9.81, 0.0), dtype=np.float64)


@dataclass(frozen=True)
class JointModel:
    joint_id: str
    dof: int
    parent: int
    child: int
    parent_translation: FloatArray
    parent_rotation: FloatArray
    child_translation: FloatArray
    child_rotation: FloatArray
    axis: FloatArray


@dataclass(frozen=True)
class SpatialModel:
    body_ids: tuple[str, ...]
    parent_slots: tuple[int | None, ...]
    masses: FloatArray
    centres_of_mass: FloatArray
    inertias_body: FloatArray
    joints_by_child: tuple[JointModel | None, ...]
    joints_by_dof: tuple[JointModel, ...]
    ancestors_by_body: tuple[tuple[int, ...], ...]
    gravity: FloatArray
    inertia_identity: tuple[dict[str, Any], ...]


@dataclass(frozen=True)
class Configuration:
    root_position: FloatArray
    root_rotation: FloatArray
    joint_positions: FloatArray


@dataclass(frozen=True)
class Kinematics:
    body_positions: FloatArray
    body_rotations: FloatArray
    joint_origins: FloatArray
    joint_axes: FloatArray


@dataclass(frozen=True)
class Motion:
    linear_velocity: FloatArray
    angular_velocity: FloatArray
    linear_acceleration: FloatArray
    angular_acceleration: FloatArray


def build_spatial_model(descriptor: dict[str, Any]) -> SpatialModel:
    """Build the solver-private world-coordinate model from exact descriptor fields."""

    bodies = sorted(descriptor["bodies"], key=lambda row: int(row["body_slot"]))
    if len(bodies) != 24 or [int(row["body_slot"]) for row in bodies] != list(
        range(24)
    ):
        raise ValueError("descriptor body order differs")
    parent_slots = tuple(
        None if row["parent_body_slot"] is None else int(row["parent_body_slot"])
        for row in bodies
    )
    if parent_slots[0] is not None or any(
        parent is None or not 0 <= parent < slot
        for slot, parent in enumerate(parent_slots[1:], start=1)
    ):
        raise ValueError("descriptor body hierarchy differs")

    masses = (
        np.asarray([row["mass_microkilograms"] for row in bodies], dtype=np.float64)
        / 1_000_000.0
    )
    centres = (
        np.asarray(
            [row["center_of_mass_micrometres"] for row in bodies], dtype=np.float64
        )
        / 1_000_000.0
    )
    inertias = np.empty((len(bodies), 3, 3), dtype=np.float64)
    inertia_identity: list[dict[str, Any]] = []
    for slot, body in enumerate(bodies):
        frame = body["solver_principal_frame"]
        if frame["translation_micrometres"] != [0, 0, 0]:
            raise ValueError("descriptor principal-frame translation differs")
        rotation = quaternion_to_matrix(decode_q1_30(frame["rotation_q1_30"]))
        principal = np.asarray(
            body["solver_principal_inertia_microkilogram_metre_squared"],
            dtype=np.float64,
        )
        solver_tensor_micro = rotation @ np.diag(principal) @ rotation.T
        authoritative = _symmetric_tensor(
            body["authoritative_inertia_tensor_microkilogram_metre_squared"]
        )
        error = float(np.max(np.abs(solver_tensor_micro - authoritative)))
        declared = float(body["solver_tensor_error_max_microkilogram_metre_squared"])
        if error > declared + 1.0e-6:
            raise ValueError(
                "descriptor solver inertia exceeds declared projection error"
            )
        if np.min(np.linalg.eigvalsh(solver_tensor_micro)) <= 0.0:
            raise ValueError("descriptor solver inertia is not positive definite")
        inertias[slot] = solver_tensor_micro / 1_000_000.0
        inertia_identity.append(
            {
                "body_id": body["body_id"],
                "maximum_projection_error_microkilogram_metre_squared": error,
                "declared_maximum_error_microkilogram_metre_squared": declared,
                "status": "PASS",
            }
        )

    joints_by_child: list[JointModel | None] = [None] * len(bodies)
    joints_by_dof: list[JointModel | None] = [None] * JOINT_COUNT
    for row in descriptor["joints"]:
        dof = int(row["dof_ordinal"])
        parent = int(row["parent_body_slot"])
        child = int(row["child_body_slot"])
        axis = decode_q1_30(row["axis_q1_30"])
        axis_norm = float(np.linalg.norm(axis))
        if (
            not 0 <= dof < JOINT_COUNT
            or not 0 < child < len(bodies)
            or parent_slots[child] != parent
            or joints_by_child[child] is not None
            or joints_by_dof[dof] is not None
            or abs(axis_norm - 1.0) > 2.0e-9
        ):
            raise ValueError("descriptor joint identity differs")
        joint = JointModel(
            joint_id=row["joint_id"],
            dof=dof,
            parent=parent,
            child=child,
            parent_translation=_translation(row["parent_frame"]),
            parent_rotation=_rotation(row["parent_frame"]),
            child_translation=_translation(row["child_frame"]),
            child_rotation=_rotation(row["child_frame"]),
            axis=axis / axis_norm,
        )
        joints_by_child[child] = joint
        joints_by_dof[dof] = joint
    if any(joint is None for joint in joints_by_child[1:]) or any(
        joint is None for joint in joints_by_dof
    ):
        raise ValueError("descriptor joint closure differs")

    ancestors: list[tuple[int, ...]] = [()]
    for slot in range(1, len(bodies)):
        joint = joints_by_child[slot]
        assert joint is not None
        ancestors.append((*ancestors[joint.parent], joint.dof))
    return SpatialModel(
        body_ids=tuple(row["body_id"] for row in bodies),
        parent_slots=parent_slots,
        masses=masses,
        centres_of_mass=centres,
        inertias_body=inertias,
        joints_by_child=tuple(joints_by_child),
        joints_by_dof=tuple(joint for joint in joints_by_dof if joint is not None),
        ancestors_by_body=tuple(ancestors),
        gravity=ENGINE_GRAVITY.copy(),
        inertia_identity=tuple(inertia_identity),
    )


def forward_kinematics(model: SpatialModel, configuration: Configuration) -> Kinematics:
    _validate_configuration(configuration)
    body_count = len(model.body_ids)
    positions = np.empty((body_count, 3), dtype=np.float64)
    rotations = np.empty((body_count, 3, 3), dtype=np.float64)
    joint_origins = np.empty((JOINT_COUNT, 3), dtype=np.float64)
    joint_axes = np.empty((JOINT_COUNT, 3), dtype=np.float64)
    positions[0] = configuration.root_position
    rotations[0] = configuration.root_rotation
    for slot in range(1, body_count):
        joint = model.joints_by_child[slot]
        assert joint is not None
        parent_rotation = rotations[joint.parent]
        pre_rotation = parent_rotation @ joint.parent_rotation
        joint_origins[joint.dof] = (
            positions[joint.parent] + parent_rotation @ joint.parent_translation
        )
        joint_axes[joint.dof] = pre_rotation @ joint.axis
        child_rotation = (
            pre_rotation
            @ axis_angle_matrix(
                joint.axis, float(configuration.joint_positions[joint.dof])
            )
            @ joint.child_rotation.T
        )
        positions[slot] = (
            joint_origins[joint.dof] - child_rotation @ joint.child_translation
        )
        rotations[slot] = child_rotation
    return Kinematics(positions, rotations, joint_origins, joint_axes)


def propagate_motion(
    model: SpatialModel,
    kinematics: Kinematics,
    velocity: FloatArray,
    acceleration: FloatArray,
) -> Motion:
    _validate_generalized(velocity, "velocity")
    _validate_generalized(acceleration, "acceleration")
    body_count = len(model.body_ids)
    linear_velocity = np.empty((body_count, 3), dtype=np.float64)
    angular_velocity = np.empty((body_count, 3), dtype=np.float64)
    linear_acceleration = np.empty((body_count, 3), dtype=np.float64)
    angular_acceleration = np.empty((body_count, 3), dtype=np.float64)
    linear_velocity[0] = velocity[:3]
    angular_velocity[0] = velocity[3:6]
    linear_acceleration[0] = acceleration[:3]
    angular_acceleration[0] = acceleration[3:6]

    for slot in range(1, body_count):
        joint = model.joints_by_child[slot]
        assert joint is not None
        parent = joint.parent
        origin = kinematics.joint_origins[joint.dof]
        axis = kinematics.joint_axes[joint.dof]
        parent_to_joint = origin - kinematics.body_positions[parent]
        joint_to_child = kinematics.body_positions[slot] - origin
        parent_omega = angular_velocity[parent]
        child_omega = parent_omega + axis * velocity[ROOT_WIDTH + joint.dof]
        axis_rate = np.cross(parent_omega, axis)
        child_alpha = (
            angular_acceleration[parent]
            + axis * acceleration[ROOT_WIDTH + joint.dof]
            + axis_rate * velocity[ROOT_WIDTH + joint.dof]
        )
        joint_velocity = linear_velocity[parent] + np.cross(
            parent_omega, parent_to_joint
        )
        joint_acceleration = (
            linear_acceleration[parent]
            + np.cross(angular_acceleration[parent], parent_to_joint)
            + np.cross(parent_omega, np.cross(parent_omega, parent_to_joint))
        )
        linear_velocity[slot] = joint_velocity + np.cross(child_omega, joint_to_child)
        linear_acceleration[slot] = (
            joint_acceleration
            + np.cross(child_alpha, joint_to_child)
            + np.cross(child_omega, np.cross(child_omega, joint_to_child))
        )
        angular_velocity[slot] = child_omega
        angular_acceleration[slot] = child_alpha
    return Motion(
        linear_velocity,
        angular_velocity,
        linear_acceleration,
        angular_acceleration,
    )


def point_position(
    kinematics: Kinematics, body_slot: int, local_point: FloatArray
) -> FloatArray:
    return (
        kinematics.body_positions[body_slot]
        + kinematics.body_rotations[body_slot] @ local_point
    )


def point_jacobian(
    model: SpatialModel,
    kinematics: Kinematics,
    body_slot: int,
    local_point: FloatArray,
) -> FloatArray:
    point = point_position(kinematics, body_slot, local_point)
    jacobian = np.zeros((3, GENERALIZED_WIDTH), dtype=np.float64)
    jacobian[:, :3] = np.eye(3, dtype=np.float64)
    root_lever = point - kinematics.body_positions[0]
    for axis in range(3):
        jacobian[:, 3 + axis] = np.cross(np.eye(3)[axis], root_lever)
    for dof in model.ancestors_by_body[body_slot]:
        jacobian[:, ROOT_WIDTH + dof] = np.cross(
            kinematics.joint_axes[dof], point - kinematics.joint_origins[dof]
        )
    return jacobian


def point_acceleration(
    kinematics: Kinematics,
    motion: Motion,
    body_slot: int,
    local_point: FloatArray,
) -> FloatArray:
    lever = kinematics.body_rotations[body_slot] @ local_point
    omega = motion.angular_velocity[body_slot]
    return (
        motion.linear_acceleration[body_slot]
        + np.cross(motion.angular_acceleration[body_slot], lever)
        + np.cross(omega, np.cross(omega, lever))
    )


def body_jacobians(
    model: SpatialModel,
    kinematics: Kinematics,
    body_slot: int,
    local_point: FloatArray,
) -> tuple[FloatArray, FloatArray]:
    linear = point_jacobian(model, kinematics, body_slot, local_point)
    angular = np.zeros((3, GENERALIZED_WIDTH), dtype=np.float64)
    angular[:, 3:6] = np.eye(3, dtype=np.float64)
    for dof in model.ancestors_by_body[body_slot]:
        angular[:, ROOT_WIDTH + dof] = kinematics.joint_axes[dof]
    return linear, angular


def mass_matrix(
    model: SpatialModel, configuration: Configuration
) -> tuple[FloatArray, Kinematics]:
    """Assemble M independently from the kinetic-energy body Jacobians."""

    kinematics = forward_kinematics(model, configuration)
    matrix = np.zeros((GENERALIZED_WIDTH, GENERALIZED_WIDTH), dtype=np.float64)
    for slot in range(len(model.body_ids)):
        linear, angular = body_jacobians(
            model, kinematics, slot, model.centres_of_mass[slot]
        )
        inertia_world = (
            kinematics.body_rotations[slot]
            @ model.inertias_body[slot]
            @ kinematics.body_rotations[slot].T
        )
        matrix += model.masses[slot] * (linear.T @ linear)
        matrix += angular.T @ inertia_world @ angular
    return matrix, kinematics


def inverse_dynamics(
    model: SpatialModel,
    configuration: Configuration,
    velocity: FloatArray,
    acceleration: FloatArray,
) -> FloatArray:
    """Project body inertial wrenches; no articulated-system solve is performed."""

    kinematics = forward_kinematics(model, configuration)
    motion = propagate_motion(model, kinematics, velocity, acceleration)
    generalized = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    for slot in range(len(model.body_ids)):
        com_local = model.centres_of_mass[slot]
        com_acceleration = point_acceleration(kinematics, motion, slot, com_local)
        linear, angular = body_jacobians(model, kinematics, slot, com_local)
        inertia_world = (
            kinematics.body_rotations[slot]
            @ model.inertias_body[slot]
            @ kinematics.body_rotations[slot].T
        )
        omega = motion.angular_velocity[slot]
        force = model.masses[slot] * (com_acceleration - model.gravity)
        moment = inertia_world @ motion.angular_acceleration[slot] + np.cross(
            omega, inertia_world @ omega
        )
        generalized += linear.T @ force + angular.T @ moment
    return generalized


def inverse_dynamics_mass_matrix(
    model: SpatialModel,
    configuration: Configuration,
    velocity: FloatArray,
) -> FloatArray:
    """Recover M from independent inverse-dynamics unit-acceleration probes."""

    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    bias = inverse_dynamics(model, configuration, velocity, zero)
    columns = []
    for column in range(GENERALIZED_WIDTH):
        acceleration = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
        acceleration[column] = 1.0
        columns.append(
            inverse_dynamics(model, configuration, velocity, acceleration) - bias
        )
    return np.column_stack(columns)


def integrate_configuration(
    configuration: Configuration, velocity: FloatArray, delta_seconds: float
) -> Configuration:
    _validate_generalized(velocity, "velocity")
    return Configuration(
        root_position=configuration.root_position + delta_seconds * velocity[:3],
        root_rotation=rotation_exp(delta_seconds * velocity[3:6])
        @ configuration.root_rotation,
        joint_positions=(
            configuration.joint_positions + delta_seconds * velocity[ROOT_WIDTH:]
        ),
    )


def rotation_exp(vector: FloatArray) -> FloatArray:
    angle = float(np.linalg.norm(vector))
    skew = np.asarray(
        (
            (0.0, -float(vector[2]), float(vector[1])),
            (float(vector[2]), 0.0, -float(vector[0])),
            (-float(vector[1]), float(vector[0]), 0.0),
        ),
        dtype=np.float64,
    )
    if angle < 1.0e-12:
        return np.eye(3, dtype=np.float64) + skew + 0.5 * (skew @ skew)
    return (
        np.eye(3, dtype=np.float64)
        + np.sin(angle) / angle * skew
        + (1.0 - np.cos(angle)) / (angle * angle) * (skew @ skew)
    )


def generalized_contact_force(
    jacobian_xyz: FloatArray, force_normal_right_forward: FloatArray
) -> FloatArray:
    if jacobian_xyz.shape != (3, GENERALIZED_WIDTH):
        raise ValueError("contact Jacobian shape differs")
    force = np.asarray(force_normal_right_forward, dtype=np.float64)
    if force.shape != (3,) or not np.all(np.isfinite(force)):
        raise ValueError("contact force differs")
    world_force = force[[1, 0, 2]]
    return jacobian_xyz.T @ world_force


def _translation(frame: dict[str, Any]) -> FloatArray:
    return np.asarray(frame["translation_micrometres"], dtype=np.float64) / 1_000_000.0


def _rotation(frame: dict[str, Any]) -> FloatArray:
    return quaternion_to_matrix(decode_q1_30(frame["rotation_q1_30"]))


def _symmetric_tensor(values: list[int]) -> FloatArray:
    xx, xy, xz, yy, yz, zz = (float(value) for value in values)
    return np.asarray(((xx, xy, xz), (xy, yy, yz), (xz, yz, zz)), dtype=np.float64)


def _validate_configuration(configuration: Configuration) -> None:
    if (
        configuration.root_position.shape != (3,)
        or configuration.root_rotation.shape != (3, 3)
        or configuration.joint_positions.shape != (JOINT_COUNT,)
        or not np.all(np.isfinite(configuration.root_position))
        or not np.all(np.isfinite(configuration.root_rotation))
        or not np.all(np.isfinite(configuration.joint_positions))
    ):
        raise ValueError("configuration differs")
    if (
        np.max(
            np.abs(
                configuration.root_rotation @ configuration.root_rotation.T - np.eye(3)
            )
        )
        > 1.0e-10
        or abs(float(np.linalg.det(configuration.root_rotation)) - 1.0) > 1.0e-10
    ):
        raise ValueError("root rotation differs")


def _validate_generalized(value: FloatArray, name: str) -> None:
    if value.shape != (GENERALIZED_WIDTH,) or not np.all(np.isfinite(value)):
        raise ValueError(f"generalized {name} differs")
