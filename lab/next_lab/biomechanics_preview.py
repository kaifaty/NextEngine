from __future__ import annotations

import argparse
import html
import json
import math
import struct
from pathlib import Path
from typing import Any

from next_lab.motor_mirror import validate_biomechanics_descriptor


def render_biomechanics_preview(descriptor: dict[str, Any]) -> str:
    validate_biomechanics_descriptor(descriptor)
    width, height = 1800, 720
    panel_width, panel_height = 560, 600
    margin_x, panel_top = 20, 70
    projections = (
        ("front  (X / Y)", 0, 1),
        ("side  (Z / Y)", 2, 1),
        ("top  (X / Z)", 0, 2),
    )
    bodies = descriptor["bodies"]
    positions = [_decode_f32_vector(body["initial_translation_f32_bits"]) for body in bodies]
    collider_centers = [
        tuple(
            positions[body["body_slot"]][axis]
            + collider["local_translation_micrometres"][axis] / 1_000_000.0
            for axis in range(3)
        )
        for body in bodies
        for collider in body["colliders"]
    ]
    all_points = positions + collider_centers
    elements = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#0b1020"/>',
        '<style>text{font-family:ui-monospace,monospace;fill:#e5e7eb}.title{font-size:20px;font-weight:700}.small{font-size:11px}.tiny{font-size:8px}</style>',
        '<text x="24" y="32" class="title">TRAIN-2 generated biomechanics articulation preview</text>',
        f'<text x="24" y="52" class="small">BodySchema {html.escape(descriptor["body_schema_hash"])} · compiled {html.escape(descriptor["compiled_descriptor_hash"])}</text>',
    ]
    for panel, (title, horizontal, vertical) in enumerate(projections):
        origin_x = margin_x + panel * (panel_width + 30)
        minimum_h = min(point[horizontal] for point in all_points) - 0.18
        maximum_h = max(point[horizontal] for point in all_points) + 0.18
        minimum_v = min(point[vertical] for point in all_points) - 0.18
        maximum_v = max(point[vertical] for point in all_points) + 0.18
        scale = min(
            (panel_width - 40) / (maximum_h - minimum_h),
            (panel_height - 55) / (maximum_v - minimum_v),
        )

        def project(point: tuple[float, float, float]) -> tuple[float, float]:
            x = origin_x + 20 + (point[horizontal] - minimum_h) * scale
            y = panel_top + panel_height - 20 - (point[vertical] - minimum_v) * scale
            return x, y

        elements.extend(
            (
                f'<rect x="{origin_x}" y="{panel_top}" width="{panel_width}" height="{panel_height}" rx="8" fill="#111827" stroke="#334155"/>',
                f'<text x="{origin_x + 18}" y="{panel_top + 26}" class="title">{title}</text>',
            )
        )
        if vertical == 1:
            ground_y = project((0.0, 0.0, 0.0))[1]
            elements.append(
                f'<line x1="{origin_x + 10}" y1="{ground_y:.2f}" x2="{origin_x + panel_width - 10}" y2="{ground_y:.2f}" stroke="#64748b" stroke-dasharray="6 4"/>'
            )
        for slot, body in enumerate(bodies):
            parent = body["parent_body_slot"]
            if parent is not None:
                x1, y1 = project(positions[parent])
                x2, y2 = project(positions[slot])
                elements.append(
                    f'<line x1="{x1:.2f}" y1="{y1:.2f}" x2="{x2:.2f}" y2="{y2:.2f}" stroke="#64748b" stroke-width="2"/>'
                )
        collider_index = 0
        for body in bodies:
            body_position = positions[body["body_slot"]]
            for collider in body["colliders"]:
                center = collider_centers[collider_index]
                collider_index += 1
                x, y = project(center)
                extent_h, extent_v = _projected_extents(collider["geometry"], horizontal, vertical)
                role = int(collider["contact_role"])
                color = "#22d3ee" if role == 8 else "#f59e0b"
                elements.append(
                    f'<rect x="{x - extent_h * scale:.2f}" y="{y - extent_v * scale:.2f}" width="{2 * extent_h * scale:.2f}" height="{2 * extent_v * scale:.2f}" rx="4" fill="none" stroke="{color}" stroke-width="2" opacity="0.9"/>'
                )
                bx, by = project(body_position)
                elements.append(
                    f'<line x1="{bx:.2f}" y1="{by:.2f}" x2="{x:.2f}" y2="{y:.2f}" stroke="{color}" stroke-width="1" opacity="0.45"/>'
                )
        for body in bodies:
            x, y = project(positions[body["body_slot"]])
            carrier = bool(body["non_colliding_carrier"])
            color = "#94a3b8" if carrier else "#a78bfa"
            elements.append(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="4" fill="{color}"/>')
            elements.append(
                f'<text x="{x + 5:.2f}" y="{y - 5:.2f}" class="tiny">{body["body_slot"]}</text>'
            )
        for joint in descriptor["joints"]:
            parent_position = positions[joint["parent_body_slot"]]
            offset = joint["parent_frame"]["translation_micrometres"]
            point = tuple(
                parent_position[axis] + offset[axis] / 1_000_000.0 for axis in range(3)
            )
            axis = tuple(value / float(1 << 30) for value in joint["axis_q1_30"])
            start_x, start_y = project(point)
            end_point = tuple(point[index] + axis[index] * 0.07 for index in range(3))
            end_x, end_y = project(end_point)
            axis_color = "#ef4444" if axis[0] else "#22c55e" if axis[1] else "#3b82f6"
            if abs(end_x - start_x) + abs(end_y - start_y) < 1.0:
                elements.append(
                    f'<circle cx="{start_x:.2f}" cy="{start_y:.2f}" r="6" fill="none" stroke="{axis_color}" stroke-width="2"/>'
                )
            else:
                elements.append(
                    f'<line x1="{start_x:.2f}" y1="{start_y:.2f}" x2="{end_x:.2f}" y2="{end_y:.2f}" stroke="{axis_color}" stroke-width="3"/>'
                )
    elements.extend(
        (
            '<text x="24" y="698" class="small"><tspan fill="#a78bfa">●</tspan> physical body  <tspan fill="#94a3b8">●</tspan> carrier  <tspan fill="#22d3ee">□</tspan> sole collider  <tspan fill="#f59e0b">□</tspan> other collider  axes: <tspan fill="#ef4444">X</tspan> <tspan fill="#22c55e">Y</tspan> <tspan fill="#3b82f6">Z</tspan></text>',
            "</svg>",
        )
    )
    return "\n".join(elements) + "\n"


def render_biomechanics_limit_preview(descriptor: dict[str, Any]) -> str:
    validate_biomechanics_descriptor(descriptor)
    width, height = 1800, 720
    groups = (
        ("spine", lambda name: "torso-" in name),
        ("hips", lambda name: "-hip-" in name),
        ("knees", lambda name: name.endswith("-knee")),
        ("ankles", lambda name: "-ankle-" in name),
        ("shoulders", lambda name: "-shoulder-" in name),
        ("elbows", lambda name: name.endswith("-elbow")),
    )
    elements = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#0b1020"/>',
        '<style>text{font-family:ui-monospace,monospace;fill:#e5e7eb}.title{font-size:20px;font-weight:700}.small{font-size:11px}.panel{font-size:15px;font-weight:700}</style>',
        '<text x="24" y="32" class="title">TRAIN-2 joint-group hard-limit preview · isometric X/Z/Y</text>',
        f'<text x="24" y="52" class="small">compiled {html.escape(descriptor["compiled_descriptor_hash"])} · red=hard minimum · green=hard maximum · grey=neutral</text>',
    ]
    neutral = _forward_kinematics(descriptor, {})
    for group_index, (group_name, includes) in enumerate(groups):
        selected = [joint for joint in descriptor["joints"] if includes(joint["joint_id"])]
        for bound_index, bound in enumerate((0, 1)):
            panel_index = group_index * 2 + bound_index
            column, row = panel_index % 6, panel_index // 6
            x0, y0 = 20 + column * 295, 72 + row * 305
            panel_width, panel_height = 275, 285
            targets = {
                joint["dof_ordinal"]: joint["hard_limit_microradians"][bound] / 1_000_000.0
                for joint in selected
            }
            posed = _forward_kinematics(descriptor, targets)
            all_projected = [_isometric(point) for point, _ in neutral + posed]
            minimum_h = min(point[0] for point in all_projected) - 0.12
            maximum_h = max(point[0] for point in all_projected) + 0.12
            minimum_v = min(point[1] for point in all_projected) - 0.08
            maximum_v = max(point[1] for point in all_projected) + 0.08
            scale = min(
                (panel_width - 24) / (maximum_h - minimum_h),
                (panel_height - 52) / (maximum_v - minimum_v),
            )

            def project(point: tuple[float, float, float]) -> tuple[float, float]:
                horizontal, vertical = _isometric(point)
                return (
                    x0 + 12 + (horizontal - minimum_h) * scale,
                    y0 + panel_height - 12 - (vertical - minimum_v) * scale,
                )

            label = "minimum" if bound == 0 else "maximum"
            color = "#ef4444" if bound == 0 else "#22c55e"
            elements.extend(
                (
                    f'<rect x="{x0}" y="{y0}" width="{panel_width}" height="{panel_height}" rx="8" fill="#111827" stroke="#334155"/>',
                    f'<text x="{x0 + 12}" y="{y0 + 22}" class="panel">{group_name} · {label}</text>',
                )
            )
            _append_skeleton(elements, descriptor, neutral, project, "#64748b", 1, 0.55)
            _append_skeleton(elements, descriptor, posed, project, color, 3, 1.0)
    elements.extend(("</svg>",))
    return "\n".join(elements) + "\n"


def _decode_f32_vector(bits: list[int]) -> tuple[float, float, float]:
    return tuple(struct.unpack("<f", struct.pack("<I", value))[0] for value in bits)  # type: ignore[return-value]


def _projected_extents(geometry: dict[str, Any], horizontal: int, vertical: int) -> tuple[float, float]:
    kind = geometry["kind"]
    if kind == "box":
        values = [value / 1_000_000.0 for value in geometry["half_extents_micrometres"]]
    elif kind == "sphere":
        radius = geometry["radius_micrometres"] / 1_000_000.0
        values = [radius, radius, radius]
    else:
        radius = geometry["radius_micrometres"] / 1_000_000.0
        half_segment = geometry["half_segment_micrometres"] / 1_000_000.0
        values = [radius + half_segment, radius, radius]
    return values[horizontal], values[vertical]


def _forward_kinematics(
    descriptor: dict[str, Any], targets: dict[int, float]
) -> list[tuple[tuple[float, float, float], tuple[float, float, float, float]]]:
    bodies = descriptor["bodies"]
    joint_by_child = {joint["child_body_slot"]: joint for joint in descriptor["joints"]}
    poses: list[tuple[tuple[float, float, float], tuple[float, float, float, float]]] = []
    for slot, body in enumerate(bodies):
        if slot == 0:
            poses.append(
                (
                    _decode_f32_vector(body["initial_translation_f32_bits"]),
                    _decode_q1_30(body["initial_rotation_f32_bits"], bits=True),
                )
            )
            continue
        joint = joint_by_child[slot]
        parent_position, parent_rotation = poses[joint["parent_body_slot"]]
        parent_frame = joint["parent_frame"]
        child_frame = joint["child_frame"]
        parent_frame_rotation = _decode_q1_30(parent_frame["rotation_q1_30"])
        child_frame_rotation = _decode_q1_30(child_frame["rotation_q1_30"])
        angle = targets.get(joint["dof_ordinal"], 0.0)
        axis = tuple(value / float(1 << 30) for value in joint["axis_q1_30"])
        joint_rotation = _axis_angle(axis, angle)
        child_rotation = _quaternion_multiply(
            _quaternion_multiply(
                _quaternion_multiply(parent_rotation, parent_frame_rotation),
                joint_rotation,
            ),
            _quaternion_conjugate(child_frame_rotation),
        )
        parent_offset = tuple(
            value / 1_000_000.0 for value in parent_frame["translation_micrometres"]
        )
        child_offset = tuple(
            value / 1_000_000.0 for value in child_frame["translation_micrometres"]
        )
        anchor = _vector_add(parent_position, _rotate(parent_rotation, parent_offset))
        child_position = _vector_subtract(anchor, _rotate(child_rotation, child_offset))
        poses.append((child_position, child_rotation))
    return poses


def _append_skeleton(
    elements: list[str],
    descriptor: dict[str, Any],
    poses: list[tuple[tuple[float, float, float], tuple[float, float, float, float]]],
    project: Any,
    color: str,
    width: int,
    opacity: float,
) -> None:
    for slot, body in enumerate(descriptor["bodies"]):
        parent = body["parent_body_slot"]
        if parent is not None:
            x1, y1 = project(poses[parent][0])
            x2, y2 = project(poses[slot][0])
            elements.append(
                f'<line x1="{x1:.2f}" y1="{y1:.2f}" x2="{x2:.2f}" y2="{y2:.2f}" stroke="{color}" stroke-width="{width}" opacity="{opacity}"/>'
            )
        x, y = project(poses[slot][0])
        elements.append(
            f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{width + 1}" fill="{color}" opacity="{opacity}"/>'
        )


def _isometric(point: tuple[float, float, float]) -> tuple[float, float]:
    return point[0] + 0.45 * point[2], point[1] + 0.18 * point[2]


def _decode_q1_30(
    values: list[int], *, bits: bool = False
) -> tuple[float, float, float, float]:
    if bits:
        return tuple(struct.unpack("<f", struct.pack("<I", value))[0] for value in values)  # type: ignore[return-value]
    return tuple(value / float(1 << 30) for value in values)  # type: ignore[return-value]


def _axis_angle(
    axis: tuple[float, float, float], angle: float
) -> tuple[float, float, float, float]:
    sine = math.sin(angle * 0.5)
    return axis[0] * sine, axis[1] * sine, axis[2] * sine, math.cos(angle * 0.5)


def _quaternion_multiply(
    left: tuple[float, float, float, float],
    right: tuple[float, float, float, float],
) -> tuple[float, float, float, float]:
    lx, ly, lz, lw = left
    rx, ry, rz, rw = right
    return (
        lw * rx + lx * rw + ly * rz - lz * ry,
        lw * ry - lx * rz + ly * rw + lz * rx,
        lw * rz + lx * ry - ly * rx + lz * rw,
        lw * rw - lx * rx - ly * ry - lz * rz,
    )


def _quaternion_conjugate(
    value: tuple[float, float, float, float]
) -> tuple[float, float, float, float]:
    return -value[0], -value[1], -value[2], value[3]


def _rotate(
    rotation: tuple[float, float, float, float],
    value: tuple[float, float, float],
) -> tuple[float, float, float]:
    pure = value[0], value[1], value[2], 0.0
    rotated = _quaternion_multiply(
        _quaternion_multiply(rotation, pure), _quaternion_conjugate(rotation)
    )
    return rotated[0], rotated[1], rotated[2]


def _vector_add(
    left: tuple[float, float, float], right: tuple[float, float, float]
) -> tuple[float, float, float]:
    return tuple(left[index] + right[index] for index in range(3))  # type: ignore[return-value]


def _vector_subtract(
    left: tuple[float, float, float], right: tuple[float, float, float]
) -> tuple[float, float, float]:
    return tuple(left[index] - right[index] for index in range(3))  # type: ignore[return-value]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("descriptor", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    descriptor = json.loads(arguments.descriptor.read_text(encoding="utf-8"))
    arguments.output.write_text(render_biomechanics_preview(descriptor), encoding="utf-8")
    limits_output = arguments.output.with_name(f"{arguments.output.stem}-limits.svg")
    limits_output.write_text(render_biomechanics_limit_preview(descriptor), encoding="utf-8")


if __name__ == "__main__":
    main()
