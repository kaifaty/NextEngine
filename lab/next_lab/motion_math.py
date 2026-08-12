from __future__ import annotations

import math
import struct
from typing import Any, Iterable

import numpy as np
from numpy.typing import NDArray


FloatArray = NDArray[np.float64]


def rotation_axis(axis: str, angle_radians: float) -> FloatArray:
    sine = math.sin(angle_radians)
    cosine = math.cos(angle_radians)
    if axis == "X":
        return np.asarray(
            ((1.0, 0.0, 0.0), (0.0, cosine, -sine), (0.0, sine, cosine)),
            dtype=np.float64,
        )
    if axis == "Y":
        return np.asarray(
            ((cosine, 0.0, sine), (0.0, 1.0, 0.0), (-sine, 0.0, cosine)),
            dtype=np.float64,
        )
    if axis == "Z":
        return np.asarray(
            ((cosine, -sine, 0.0), (sine, cosine, 0.0), (0.0, 0.0, 1.0)),
            dtype=np.float64,
        )
    raise ValueError(f"unsupported rotation axis {axis!r}")


def euler_matrix(order: str, values_degrees: Iterable[float]) -> FloatArray:
    values = tuple(values_degrees)
    if len(values) != len(order):
        raise ValueError("Euler channel/value length mismatch")
    result = np.eye(3, dtype=np.float64)
    # ASF declares intrinsic channel order. With column vectors, applying the
    # listed X, then Y, then Z rotations is represented by Rz @ Ry @ Rx.
    for axis, value in zip(order.upper(), values, strict=True):
        result = rotation_axis(axis, math.radians(value)) @ result
    return result


def axis_angle_matrix(axis: FloatArray, angle_radians: float) -> FloatArray:
    norm = float(np.linalg.norm(axis))
    if norm <= 0.0:
        raise ValueError("axis must be non-zero")
    x, y, z = axis / norm
    sine = math.sin(angle_radians)
    cosine = math.cos(angle_radians)
    one_minus = 1.0 - cosine
    return np.asarray(
        (
            (
                cosine + x * x * one_minus,
                x * y * one_minus - z * sine,
                x * z * one_minus + y * sine,
            ),
            (
                y * x * one_minus + z * sine,
                cosine + y * y * one_minus,
                y * z * one_minus - x * sine,
            ),
            (
                z * x * one_minus - y * sine,
                z * y * one_minus + x * sine,
                cosine + z * z * one_minus,
            ),
        ),
        dtype=np.float64,
    )


def decompose_xzy(rotation: FloatArray) -> tuple[float, float, float]:
    """Return angles for ``Rx(pitch) @ Rz(roll) @ Ry(yaw)``."""

    sine_roll = max(-1.0, min(1.0, -float(rotation[0, 1])))
    roll = math.asin(sine_roll)
    cosine_roll = math.cos(roll)
    if abs(cosine_roll) > 1.0e-8:
        pitch = math.atan2(float(rotation[2, 1]), float(rotation[1, 1]))
        yaw = math.atan2(float(rotation[0, 2]), float(rotation[0, 0]))
    else:
        pitch = math.atan2(-float(rotation[1, 2]), float(rotation[2, 2]))
        yaw = 0.0
    return pitch, roll, yaw


def matrix_to_quaternion(rotation: FloatArray) -> FloatArray:
    trace = float(np.trace(rotation))
    if trace > 0.0:
        scale = math.sqrt(trace + 1.0) * 2.0
        quaternion = np.asarray(
            (
                (rotation[2, 1] - rotation[1, 2]) / scale,
                (rotation[0, 2] - rotation[2, 0]) / scale,
                (rotation[1, 0] - rotation[0, 1]) / scale,
                0.25 * scale,
            ),
            dtype=np.float64,
        )
    else:
        diagonal = np.diag(rotation)
        index = int(np.argmax(diagonal))
        if index == 0:
            scale = math.sqrt(1.0 + rotation[0, 0] - rotation[1, 1] - rotation[2, 2]) * 2.0
            quaternion = np.asarray(
                (
                    0.25 * scale,
                    (rotation[0, 1] + rotation[1, 0]) / scale,
                    (rotation[0, 2] + rotation[2, 0]) / scale,
                    (rotation[2, 1] - rotation[1, 2]) / scale,
                ),
                dtype=np.float64,
            )
        elif index == 1:
            scale = math.sqrt(1.0 + rotation[1, 1] - rotation[0, 0] - rotation[2, 2]) * 2.0
            quaternion = np.asarray(
                (
                    (rotation[0, 1] + rotation[1, 0]) / scale,
                    0.25 * scale,
                    (rotation[1, 2] + rotation[2, 1]) / scale,
                    (rotation[0, 2] - rotation[2, 0]) / scale,
                ),
                dtype=np.float64,
            )
        else:
            scale = math.sqrt(1.0 + rotation[2, 2] - rotation[0, 0] - rotation[1, 1]) * 2.0
            quaternion = np.asarray(
                (
                    (rotation[0, 2] + rotation[2, 0]) / scale,
                    (rotation[1, 2] + rotation[2, 1]) / scale,
                    0.25 * scale,
                    (rotation[1, 0] - rotation[0, 1]) / scale,
                ),
                dtype=np.float64,
            )
    quaternion /= np.linalg.norm(quaternion)
    for value in quaternion:
        if abs(float(value)) > 1.0e-14:
            if value < 0.0:
                quaternion = -quaternion
            break
    return quaternion


def quaternion_to_matrix(quaternion: FloatArray) -> FloatArray:
    x, y, z, w = quaternion / np.linalg.norm(quaternion)
    return np.asarray(
        (
            (1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)),
            (2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)),
            (2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)),
        ),
        dtype=np.float64,
    )


def q1_30(values: FloatArray) -> NDArray[np.int64]:
    return np.rint(np.clip(values, -1.0, 1.0) * float(1 << 30)).astype(np.int64)


def quaternion_q1_30(rotation: FloatArray) -> NDArray[np.int64]:
    return q1_30(matrix_to_quaternion(rotation))


def decode_q1_30(values: Iterable[int]) -> FloatArray:
    return np.asarray(tuple(values), dtype=np.float64) / float(1 << 30)


def decode_f32_bits(values: Iterable[int]) -> FloatArray:
    return np.asarray(
        [struct.unpack("<f", struct.pack("<I", value))[0] for value in values],
        dtype=np.float64,
    )


def target_forward_kinematics(
    descriptor: dict[str, Any],
    root_position: FloatArray,
    root_quaternion: FloatArray,
    joint_positions: FloatArray,
) -> tuple[FloatArray, FloatArray]:
    bodies = descriptor["bodies"]
    joint_by_child = {joint["child_body_slot"]: joint for joint in descriptor["joints"]}
    positions = np.empty((len(bodies), 3), dtype=np.float64)
    rotations = np.empty((len(bodies), 3, 3), dtype=np.float64)
    positions[0] = root_position
    rotations[0] = quaternion_to_matrix(root_quaternion)
    for slot in range(1, len(bodies)):
        joint = joint_by_child[slot]
        parent_slot = int(joint["parent_body_slot"])
        parent_rotation = rotations[parent_slot]
        parent_frame = joint["parent_frame"]
        child_frame = joint["child_frame"]
        parent_frame_rotation = quaternion_to_matrix(decode_q1_30(parent_frame["rotation_q1_30"]))
        child_frame_rotation = quaternion_to_matrix(decode_q1_30(child_frame["rotation_q1_30"]))
        axis = decode_q1_30(joint["axis_q1_30"])
        angle = float(joint_positions[int(joint["dof_ordinal"])])
        child_rotation = (
            parent_rotation
            @ parent_frame_rotation
            @ axis_angle_matrix(axis, angle)
            @ child_frame_rotation.T
        )
        parent_offset = np.asarray(parent_frame["translation_micrometres"], dtype=np.float64) / 1_000_000.0
        child_offset = np.asarray(child_frame["translation_micrometres"], dtype=np.float64) / 1_000_000.0
        anchor = positions[parent_slot] + parent_rotation @ parent_offset
        positions[slot] = anchor - child_rotation @ child_offset
        rotations[slot] = child_rotation
    return positions, rotations


def target_effectors(
    descriptor: dict[str, Any], positions: FloatArray, rotations: FloatArray
) -> dict[str, FloatArray]:
    body_slots = {body["body_id"]: int(body["body_slot"]) for body in descriptor["bodies"]}
    result: dict[str, FloatArray] = {}
    for effector in descriptor["effectors"]:
        slot = body_slots[effector["body_id"]]
        offset = np.asarray(effector["local_translation_micrometres"], dtype=np.float64) / 1_000_000.0
        result[effector["effector_id"]] = positions[slot] + rotations[slot] @ offset
    return result


def collider_minimum_y(
    body_position: FloatArray,
    body_rotation: FloatArray,
    collider: dict[str, Any],
) -> float:
    local_rotation = quaternion_to_matrix(decode_q1_30(collider["local_rotation_q1_30"]))
    rotation = body_rotation @ local_rotation
    offset = np.asarray(collider["local_translation_micrometres"], dtype=np.float64) / 1_000_000.0
    center = body_position + body_rotation @ offset
    geometry = collider["geometry"]
    if geometry["kind"] == "box":
        half_extents = np.asarray(geometry["half_extents_micrometres"], dtype=np.float64) / 1_000_000.0
        vertical_extent = float(np.abs(rotation[1]) @ half_extents)
    elif geometry["kind"] == "sphere":
        vertical_extent = float(geometry["radius_micrometres"]) / 1_000_000.0
    elif geometry["kind"] == "capsule":
        radius = float(geometry["radius_micrometres"]) / 1_000_000.0
        half_segment = float(geometry["half_segment_micrometres"]) / 1_000_000.0
        vertical_extent = radius + abs(float(rotation[1, 0])) * half_segment
    else:
        raise ValueError(f"unsupported collider geometry {geometry['kind']!r}")
    return float(center[1]) - vertical_extent


def target_center_of_mass(
    descriptor: dict[str, Any], positions: FloatArray, rotations: FloatArray
) -> FloatArray:
    weighted = np.zeros(3, dtype=np.float64)
    total_mass = 0.0
    for body in descriptor["bodies"]:
        slot = int(body["body_slot"])
        mass = float(body["mass_microkilograms"])
        offset = np.asarray(body["center_of_mass_micrometres"], dtype=np.float64) / 1_000_000.0
        weighted += mass * (positions[slot] + rotations[slot] @ offset)
        total_mass += mass
    if total_mass <= 0.0:
        raise ValueError("target descriptor has no positive mass")
    return weighted / total_mass
