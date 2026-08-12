from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np
from numpy.typing import NDArray

from next_lab.motion_math import FloatArray, euler_matrix


ENGINE_FROM_CMU = np.diag((-1.0, 1.0, 1.0)).astype(np.float64)


@dataclass(frozen=True)
class AsfBone:
    name: str
    direction: FloatArray
    length: float
    axis_degrees: tuple[float, float, float]
    axis_order: str
    dof: tuple[str, ...]
    parent: str


@dataclass(frozen=True)
class AsfSkeleton:
    length_scale_metres: float
    root_order: tuple[str, ...]
    root_axis_order: str
    bones: dict[str, AsfBone]
    traversal: tuple[str, ...]


@dataclass(frozen=True)
class AmcFrame:
    source_frame: int
    channels: dict[str, tuple[float, ...]]


@dataclass(frozen=True)
class SourcePose:
    root_position: FloatArray
    root_rotation: FloatArray
    bone_starts: dict[str, FloatArray]
    bone_ends: dict[str, FloatArray]
    global_rotations: dict[str, FloatArray]
    local_rotations: dict[str, FloatArray]


def parse_asf(path: Path, *, expected_length_scale_metres: float) -> AsfSkeleton:
    lines = [_clean_line(line) for line in path.read_text(encoding="utf-8").splitlines()]
    lines = [line for line in lines if line]
    root_order: tuple[str, ...] | None = None
    root_axis_order: str | None = None
    declared_length: float | None = None
    raw_bones: dict[str, dict[str, object]] = {}
    hierarchy: dict[str, str] = {}
    section = ""
    index = 0
    while index < len(lines):
        line = lines[index]
        if line.startswith(":"):
            section = line[1:].lower()
            index += 1
            continue
        if section == "units":
            values = line.split()
            if values[0].lower() == "length":
                declared_length = float(values[1])
        elif section == "root":
            values = line.split()
            if values[0].lower() == "order":
                root_order = tuple(value.lower() for value in values[1:])
            elif values[0].lower() == "axis":
                root_axis_order = values[1].upper()
        elif section == "bonedata" and line.lower() == "begin":
            record: dict[str, object] = {}
            index += 1
            while index < len(lines) and lines[index].lower() != "end":
                values = lines[index].split()
                key = values[0].lower()
                if key == "name":
                    record["name"] = values[1]
                elif key == "direction":
                    record["direction"] = tuple(float(value) for value in values[1:4])
                elif key == "length":
                    record["length"] = float(values[1])
                elif key == "axis":
                    record["axis_degrees"] = tuple(float(value) for value in values[1:4])
                    record["axis_order"] = values[4].upper()
                elif key == "dof":
                    record["dof"] = tuple(value.lower() for value in values[1:])
                index += 1
            name = str(record.get("name", ""))
            if not name or name in raw_bones:
                raise ValueError(f"invalid or duplicate ASF bone {name!r}")
            raw_bones[name] = record
        elif section == "hierarchy" and line.lower() not in {"begin", "end"}:
            values = line.split()
            if len(values) >= 2:
                parent = values[0]
                for child in values[1:]:
                    if child in hierarchy:
                        raise ValueError(f"duplicate ASF hierarchy parent for {child!r}")
                    hierarchy[child] = parent
        index += 1

    if root_order is None or root_axis_order is None or declared_length is None:
        raise ValueError("ASF root/units closure is incomplete")
    official_scale = (1.0 / declared_length) * 2.54 / 100.0
    if abs(official_scale - expected_length_scale_metres) > 1.0e-12:
        raise ValueError("ASF length scale does not match the frozen profile")

    bones: dict[str, AsfBone] = {}
    for name, record in raw_bones.items():
        parent = hierarchy.get(name)
        if parent is None:
            raise ValueError(f"ASF bone {name!r} has no hierarchy parent")
        direction = np.asarray(record.get("direction"), dtype=np.float64)
        norm = float(np.linalg.norm(direction))
        if direction.shape != (3,) or abs(norm - 1.0) > 1.0e-4:
            raise ValueError(f"ASF bone {name!r} direction is not normalized")
        bones[name] = AsfBone(
            name=name,
            direction=direction,
            length=float(record.get("length", 0.0)),
            axis_degrees=tuple(record.get("axis_degrees", (0.0, 0.0, 0.0))),  # type: ignore[arg-type]
            axis_order=str(record.get("axis_order", "XYZ")),
            dof=tuple(record.get("dof", ())),  # type: ignore[arg-type]
            parent=parent,
        )
    traversal = _topological_traversal(bones)
    return AsfSkeleton(
        length_scale_metres=official_scale,
        root_order=root_order,
        root_axis_order=root_axis_order,
        bones=bones,
        traversal=traversal,
    )


def parse_amc(path: Path) -> tuple[AmcFrame, ...]:
    frames: list[AmcFrame] = []
    current_number: int | None = None
    current_channels: dict[str, tuple[float, ...]] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = _clean_line(raw_line)
        if not line or line.startswith(":"):
            continue
        if line.isdigit():
            if current_number is not None:
                frames.append(AmcFrame(current_number, current_channels))
            current_number = int(line)
            current_channels = {}
            continue
        if current_number is None:
            raise ValueError("AMC channel appears before the first frame")
        values = line.split()
        name = values[0]
        if name in current_channels:
            raise ValueError(f"duplicate AMC channel {name!r} in frame {current_number}")
        current_channels[name] = tuple(float(value) for value in values[1:])
    if current_number is not None:
        frames.append(AmcFrame(current_number, current_channels))
    if not frames:
        raise ValueError("AMC contains no frames")
    expected = list(range(frames[0].source_frame, frames[0].source_frame + len(frames)))
    if [frame.source_frame for frame in frames] != expected:
        raise ValueError("AMC frame numbers are not contiguous")
    return tuple(frames)


def source_forward_kinematics(skeleton: AsfSkeleton, frame: AmcFrame) -> SourcePose:
    root_values = frame.channels.get("root")
    if root_values is None or len(root_values) != len(skeleton.root_order):
        raise ValueError(f"AMC frame {frame.source_frame} has invalid root channels")
    root_by_channel = dict(zip(skeleton.root_order, root_values, strict=True))
    translation_cmu = np.asarray(
        (root_by_channel.get("tx", 0.0), root_by_channel.get("ty", 0.0), root_by_channel.get("tz", 0.0)),
        dtype=np.float64,
    )
    root_angles = tuple(root_by_channel.get(f"r{axis.lower()}", 0.0) for axis in skeleton.root_axis_order)
    root_rotation_cmu = euler_matrix(skeleton.root_axis_order, root_angles)
    root_position = ENGINE_FROM_CMU @ (translation_cmu * skeleton.length_scale_metres)
    root_rotation = ENGINE_FROM_CMU @ root_rotation_cmu @ ENGINE_FROM_CMU

    starts: dict[str, FloatArray] = {}
    ends: dict[str, FloatArray] = {"root": root_position}
    globals_: dict[str, FloatArray] = {"root": root_rotation}
    locals_: dict[str, FloatArray] = {}
    for name in skeleton.traversal:
        bone = skeleton.bones[name]
        values = frame.channels.get(name, ())
        if len(values) != len(bone.dof):
            raise ValueError(
                f"AMC frame {frame.source_frame} channel {name!r} has {len(values)} values; expected {len(bone.dof)}"
            )
        motion = np.eye(3, dtype=np.float64)
        for channel, value in zip(bone.dof, values, strict=True):
            motion = motion @ euler_matrix(channel[-1].upper(), (value,))
        basis = euler_matrix(bone.axis_order, bone.axis_degrees)
        local_cmu = basis @ motion @ basis.T
        local = ENGINE_FROM_CMU @ local_cmu @ ENGINE_FROM_CMU
        parent_rotation = globals_[bone.parent]
        start = ends[bone.parent]
        global_rotation = parent_rotation @ local
        direction = ENGINE_FROM_CMU @ bone.direction
        end = start + global_rotation @ (direction * bone.length * skeleton.length_scale_metres)
        starts[name] = start
        ends[name] = end
        globals_[name] = global_rotation
        locals_[name] = local
    return SourcePose(
        root_position=root_position,
        root_rotation=root_rotation,
        bone_starts=starts,
        bone_ends={name: value for name, value in ends.items() if name != "root"},
        global_rotations={name: value for name, value in globals_.items() if name != "root"},
        local_rotations=locals_,
    )


def _topological_traversal(bones: dict[str, AsfBone]) -> tuple[str, ...]:
    remaining = set(bones)
    ordered: list[str] = []
    resolved = {"root"}
    while remaining:
        eligible = sorted(name for name in remaining if bones[name].parent in resolved)
        if not eligible:
            raise ValueError("ASF hierarchy is cyclic or references an unknown parent")
        for name in eligible:
            ordered.append(name)
            resolved.add(name)
            remaining.remove(name)
    return tuple(ordered)


def _clean_line(line: str) -> str:
    return line.split("#", 1)[0].strip()
