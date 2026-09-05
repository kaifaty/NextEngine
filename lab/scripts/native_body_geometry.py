"""Read-only physical geometry for diagnostics; never a skeleton or pose owner."""

from __future__ import annotations

import numpy as np
from next_lab.motion_math import decode_f32_bits, quaternion_to_matrix

BOX_SIGNS = np.array(
    [
        (-1, -1, -1),
        (1, -1, -1),
        (1, -1, 1),
        (-1, -1, 1),
        (-1, 1, -1),
        (1, 1, -1),
        (1, 1, 1),
        (-1, 1, 1),
    ]
)
BOX_EDGES = (
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
)


def rotation(values):
    q = np.asarray(values, dtype=float)
    if q.shape != (4,) or not np.isfinite(q).all() or np.linalg.norm(q) == 0:
        raise ValueError("invalid diagnostic rotation")
    return quaternion_to_matrix(q)


def vector(values):
    value = np.asarray(values, dtype=float)
    if value.shape != (3,) or not np.isfinite(value).all():
        raise ValueError("invalid diagnostic vector")
    return value


def collider_geometry(collider, position, body_rotation):
    """World wireframe and exact analytic AABB for a declared box or sphere.

    Sphere circles are sampled for display only; extrema use the exact radius.
    No radius is invented for the non-colliding serial joint carriers.
    """
    center = vector(position) + body_rotation @ (
        vector(collider["local_translation_micrometres"]) / 1e6
    )
    orientation = body_rotation @ rotation(collider["local_rotation_q1_30"])
    geometry = collider["geometry"]
    if geometry["kind"] == "box":
        half = vector(geometry["half_extents_micrometres"]) / 1e6
        if np.any(half <= 0):
            raise ValueError("non-positive collider size")
        corners = center + (BOX_SIGNS * half) @ orientation.T
        edges = corners[np.asarray(BOX_EDGES)]
        extent = np.abs(orientation) @ half
    elif geometry["kind"] == "sphere":
        radius = float(geometry["radius_micrometres"]) / 1e6
        if not np.isfinite(radius) or radius <= 0:
            raise ValueError("non-positive collider size")
        angles = np.linspace(0, 2 * np.pi, 33)
        circles = []
        for a, b in ((0, 1), (1, 2), (0, 2)):
            points = np.zeros((len(angles), 3))
            points[:, a] = radius * np.cos(angles)
            points[:, b] = radius * np.sin(angles)
            points = center + points @ orientation.T
            circles.extend(np.stack((points[:-1], points[1:]), axis=1))
        edges = np.asarray(circles)
        extent = np.full(3, radius)
    else:
        raise ValueError(f"unsupported diagnostic collider: {geometry['kind']}")
    return edges, center - extent, center + extent


def physical_geometry(descriptor, frame=None):
    """Use native poses, or the descriptor's actual initial world transforms."""
    bodies = descriptor["bodies"]
    tokens = [b["body_token"] for b in bodies]
    if len(set(tokens)) != len(tokens):
        raise ValueError("duplicate descriptor body token")
    links = None
    if frame is not None:
        links = {link["body_token"]: link for link in frame["links"]}
        if len(links) != len(frame["links"]) or set(links) != set(tokens):
            raise ValueError("native body token set mismatch")
    shapes, origins = [], {}
    for body in bodies:
        if links is None:
            position = decode_f32_bits(body["initial_translation_f32_bits"])
            orient = rotation(decode_f32_bits(body["initial_rotation_f32_bits"]))
        else:
            link = links[body["body_token"]]
            position = vector(link["position_um"]) / 1e6
            orient = rotation(link["rotation_q1_30"])
        origins[body["body_id"]] = vector(position)
        colliders = body["colliders"]
        if body.get("non_colliding_carrier", False) and colliders:
            raise ValueError("non-colliding carrier has shapes")
        for collider in colliders:
            edges, low, high = collider_geometry(collider, position, orient)
            shapes.append(
                {
                    "body_id": body["body_id"],
                    "collider_id": collider["collider_id"],
                    "segments": edges,
                    "minimum": low,
                    "maximum": high,
                    "color": "#2684d9"
                    if ".left-" in body["body_id"]
                    else "#e6699b"
                    if ".right-" in body["body_id"]
                    else "#a67c00",
                }
            )
    if not shapes or len({s["collider_id"] for s in shapes}) != len(shapes):
        raise ValueError("empty or duplicate physical colliders")
    return shapes, origins


def neutral_proportions(descriptor):
    """Measurements, not anthropometric acceptance or learned quality gates."""
    shapes, origins = physical_geometry(descriptor)
    low = np.min([s["minimum"] for s in shapes], axis=0)
    high = np.max([s["maximum"] for s in shapes], axis=0)
    sides = {}
    for side in ("left", "right"):
        hip = origins[f"body.{side}-hip-pitch"]
        knee = origins[f"body.{side}-knee"]
        ankle = origins[f"body.{side}-ankle-pitch"]
        shoulder = origins[f"body.{side}-shoulder-pitch"]
        sides[side] = {
            "hip_height_m": float(hip[1] - low[1]),
            "shoulder_height_m": float(shoulder[1] - low[1]),
            "hip_to_shoulder_vertical_m": float(shoulder[1] - hip[1]),
            "thigh_joint_distance_m": float(np.linalg.norm(knee - hip)),
            "shank_joint_distance_m": float(np.linalg.norm(ankle - knee)),
            "hip_height_fraction_of_stature": float(
                (hip[1] - low[1]) / (high[1] - low[1])
            ),
        }
    return {
        "status": "report_only",
        "pose": "descriptor_initial_world_transforms_not_learned_pose",
        "body_schema_id": descriptor["body_schema_id"],
        "collider_count": len(shapes),
        "stature_m": float(high[1] - low[1]),
        "ground_minimum_y_m": float(low[1]),
        "sides": sides,
    }
