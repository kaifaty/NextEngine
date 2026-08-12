from __future__ import annotations

import hashlib
import json
import math
import os
import re
import struct
from pathlib import Path
from typing import Any

from next_lab.motor_mirror import (
    BIOMECHANICS_TRANSLATOR_ID,
    CURRENT_TRANSLATOR_VERSION,
    validate_biomechanics_descriptor,
    validate_descriptor,
)

TRANSLATOR_VERSION = CURRENT_TRANSLATOR_VERSION
BIOMECHANICS_TRANSLATOR_VERSION = "nextengine.isaac-biomechanics-usda-translator.v1"


def render_usda(descriptor: dict[str, Any]) -> str:
    if descriptor.get("translator_id") == BIOMECHANICS_TRANSLATOR_ID:
        return _render_biomechanics_usda(descriptor)
    validate_descriptor(descriptor)
    if descriptor.get("translator_version") != TRANSLATOR_VERSION:
        raise ValueError("translator version mismatch")
    body_by_id = {record["body_id"]: record for record in descriptor["bodies"]}
    world_bind_translations = _world_bind_translations(body_by_id)
    lines = [
        "#usda 1.0",
        "(",
        '    defaultPrim = "Humanoid"',
        "    metersPerUnit = 1",
        "    upAxis = \"Z\"",
        ")",
        "",
        'def Xform "Humanoid" (',
        '        prepend apiSchemas = ["PhysicsArticulationRootAPI"]',
        ")",
        "{",
        f'    custom string nextengine:bodySchemaHash = "{descriptor["body_schema_hash"]}"',
        f'    custom string nextengine:translatorVersion = "{TRANSLATOR_VERSION}"',
        '    def Scope "Bodies"',
        "    {",
    ]
    for body_id in descriptor["ordered_body_ids"]:
        body = body_by_id[body_id]
        prim = _prim(body_id)
        translation = _metres(world_bind_translations[body_id])
        mass = body["mass_microkilograms"] / 1_000_000.0
        center_of_mass = _metres(body["center_of_mass_micrometres"])
        engine_inertia = body["inertia_microkilogram_metre_squared"]
        diagonal_inertia = (
            engine_inertia[0] / 1_000_000.0,
            engine_inertia[2] / 1_000_000.0,
            engine_inertia[1] / 1_000_000.0,
        )
        lines.extend(
            [
                f'        def Xform "{prim}" (',
                '            prepend apiSchemas = ["PhysicsRigidBodyAPI", "PhysicsMassAPI"]',
                "        )",
                "        {",
                f"            double3 xformOp:translate = ({_triplet(translation)})",
                '            uniform token[] xformOpOrder = ["xformOp:translate"]',
                f"            float physics:mass = {mass:.9g}",
                f"            point3f physics:centerOfMass = ({_triplet(center_of_mass)})",
                f"            float3 physics:diagonalInertia = ({_triplet(diagonal_inertia)})",
                f'            custom string nextengine:semanticId = "{body_id}"',
            ]
        )
        for collider in body["colliders"]:
            lines.extend(_collider_lines(collider, indent="            "))
        lines.append("        }")
    lines.extend(["    }", '    def Scope "Joints"', "    {"])
    for joint in descriptor["joints"]:
        prim = _prim(joint["joint_id"])
        parent = _prim(joint["parent_body_id"])
        child = _prim(joint["child_body_id"])
        lower = joint["limit_min_microradians"] * 180.0 / 1_000_000.0 / 3.141592653589793
        upper = joint["limit_max_microradians"] * 180.0 / 1_000_000.0 / 3.141592653589793
        parent_position = _metres(joint["parent_translation_micrometres"])
        child_position = _metres(joint["child_translation_micrometres"])
        lines.extend(
            [
                f'        def PhysicsRevoluteJoint "{prim}"',
                "        {",
                f"            rel physics:body0 = </Humanoid/Bodies/{parent}>",
                f"            rel physics:body1 = </Humanoid/Bodies/{child}>",
                f"            point3f physics:localPos0 = ({_triplet(parent_position)})",
                f"            point3f physics:localPos1 = ({_triplet(child_position)})",
                '            uniform token physics:axis = "X"',
                f"            float physics:lowerLimit = {lower:.9g}",
                f"            float physics:upperLimit = {upper:.9g}",
                f'            custom string nextengine:semanticId = "{joint["joint_id"]}"',
                "        }",
            ]
        )
    lines.extend(["    }", "}", ""])
    return "\n".join(lines)


def _render_biomechanics_usda(descriptor: dict[str, Any]) -> str:
    validate_biomechanics_descriptor(descriptor)
    bodies = sorted(descriptor["bodies"], key=lambda body: body["body_slot"])
    body_paths = [f"/Humanoid/Bodies/{_prim(body['body_id'])}" for body in bodies]
    exclusions: dict[int, list[int]] = {slot: [] for slot in range(len(bodies))}
    for first, second in descriptor["collision_exclusions"]:
        exclusions[first].append(second)
        exclusions[second].append(first)
    lines = [
        "#usda 1.0",
        "(",
        '    defaultPrim = "Humanoid"',
        "    metersPerUnit = 1",
        '    upAxis = "Z"',
        ")",
        "",
        'def Xform "Humanoid" (',
        '        prepend apiSchemas = ["PhysicsArticulationRootAPI"]',
        ")",
        "{",
        f'    custom string nextengine:bodySchemaHash = "{descriptor["body_schema_hash"]}"',
        f'    custom string nextengine:compiledDescriptorHash = "{descriptor["compiled_descriptor_hash"]}"',
        f'    custom string nextengine:translatorVersion = "{BIOMECHANICS_TRANSLATOR_VERSION}"',
        '    def Scope "Bodies"',
        "    {",
    ]
    for body in bodies:
        slot = body["body_slot"]
        prim = _prim(body["body_id"])
        position = _isaac_vector_from_f32_bits(body["initial_translation_f32_bits"])
        rotation = _isaac_quaternion_from_f32_bits(body["initial_rotation_f32_bits"])
        center_of_mass = _metres(body["center_of_mass_micrometres"])
        inertia = body["solver_principal_inertia_microkilogram_metre_squared"]
        diagonal_inertia = (
            inertia[0] / 1_000_000.0,
            inertia[2] / 1_000_000.0,
            inertia[1] / 1_000_000.0,
        )
        principal_axes = _isaac_quaternion_from_q1_30(
            body["solver_principal_frame"]["rotation_q1_30"]
        )
        lines.extend(
            [
                f'        def Xform "{prim}" (',
                '            prepend apiSchemas = ["PhysicsRigidBodyAPI", "PhysicsMassAPI", "PhysicsFilteredPairsAPI"]',
                "        )",
                "        {",
                f"            double3 xformOp:translate = ({_triplet(position)})",
                f"            quatf xformOp:orient = {_quaternion(principal=rotation)}",
                '            uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:orient"]',
                f"            float physics:mass = {body['mass_microkilograms'] / 1_000_000.0:.9g}",
                f"            point3f physics:centerOfMass = ({_triplet(center_of_mass)})",
                f"            float3 physics:diagonalInertia = ({_triplet(diagonal_inertia)})",
                f"            quatf physics:principalAxes = {_quaternion(principal=principal_axes)}",
                f'            custom string nextengine:semanticId = "{body["body_id"]}"',
                f"            custom uint64 nextengine:bodyToken = {body['body_token']}",
            ]
        )
        if exclusions[slot]:
            targets = ", ".join(f"<{body_paths[other]}>" for other in exclusions[slot])
            lines.append(f"            rel physics:filteredPairs = [{targets}]")
        for collider in body["colliders"]:
            lines.extend(_biomechanics_collider_lines(collider, indent="            "))
        lines.append("        }")
    lines.extend(["    }", '    def Scope "Joints"', "    {"])
    for joint in sorted(descriptor["joints"], key=lambda item: item["dof_ordinal"]):
        parent = _prim(bodies[joint["parent_body_slot"]]["body_id"])
        child = _prim(bodies[joint["child_body_slot"]]["body_id"])
        lower, upper = joint["hard_limit_microradians"]
        parent_rotation = _isaac_quaternion_from_f32_bits(
            joint["solver_parent_rotation_f32_bits"]
        )
        child_rotation = _isaac_quaternion_from_f32_bits(
            joint["solver_child_rotation_f32_bits"]
        )
        lines.extend(
            [
                f'        def PhysicsRevoluteJoint "{_prim(joint["joint_id"])}"',
                "        {",
                f"            rel physics:body0 = </Humanoid/Bodies/{parent}>",
                f"            rel physics:body1 = </Humanoid/Bodies/{child}>",
                f"            point3f physics:localPos0 = ({_triplet(_metres(joint['parent_frame']['translation_micrometres']))})",
                f"            point3f physics:localPos1 = ({_triplet(_metres(joint['child_frame']['translation_micrometres']))})",
                f"            quatf physics:localRot0 = {_quaternion(principal=parent_rotation)}",
                f"            quatf physics:localRot1 = {_quaternion(principal=child_rotation)}",
                '            uniform token physics:axis = "X"',
                f"            float physics:lowerLimit = {lower * 180.0 / 1_000_000.0 / math.pi:.9g}",
                f"            float physics:upperLimit = {upper * 180.0 / 1_000_000.0 / math.pi:.9g}",
                f'            custom string nextengine:semanticId = "{joint["joint_id"]}"',
                f"            custom int nextengine:dofOrdinal = {joint['dof_ordinal']}",
                "        }",
            ]
        )
    lines.extend(["    }", "}", ""])
    return "\n".join(lines)


def _biomechanics_collider_lines(
    collider: dict[str, Any], indent: str
) -> list[str]:
    geometry = collider["geometry"]
    prim = _prim(collider["collider_id"])
    kind = geometry["kind"]
    if kind == "sphere":
        header = f'def Sphere "{prim}"'
        geometry_properties = [
            f"double radius = {geometry['radius_micrometres'] / 1_000_000.0:.9g}"
        ]
    elif kind == "capsule":
        header = f'def Capsule "{prim}"'
        geometry_properties = [
            f"double radius = {geometry['radius_micrometres'] / 1_000_000.0:.9g}",
            f"double height = {geometry['half_segment_micrometres'] * 2 / 1_000_000.0:.9g}",
            'uniform token axis = "X"',
        ]
    elif kind == "box":
        header = f'def Cube "{prim}"'
        half = _metres_without_axis_swap(geometry["half_extents_micrometres"])
        geometry_properties = ["double size = 2", f"double3 xformOp:scale = ({_triplet(half)})"]
    else:
        raise ValueError(f"unsupported collider geometry: {kind}")
    translation = _metres(collider["local_translation_micrometres"])
    rotation = _isaac_quaternion_from_q1_30(collider["local_rotation_q1_30"])
    scale_order = ', "xformOp:scale"' if kind == "box" else ""
    output = [
        f"{indent}{header} (",
        f'{indent}    prepend apiSchemas = ["PhysicsCollisionAPI"]',
        f"{indent})",
        f"{indent}{{",
    ]
    output.extend(f"{indent}    {line}" for line in geometry_properties)
    output.extend(
        [
            f"{indent}    double3 xformOp:translate = ({_triplet(translation)})",
            f"{indent}    quatf xformOp:orient = {_quaternion(principal=rotation)}",
            f'{indent}    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:orient"{scale_order}]',
            f'{indent}    custom string nextengine:semanticId = "{collider["collider_id"]}"',
            f"{indent}    custom uint64 nextengine:shapeToken = {collider['shape_token']}",
            f"{indent}    custom int nextengine:contactRole = {collider['contact_role']}",
            f"{indent}}}",
        ]
    )
    return output


def translate_to_store(
    descriptor: dict[str, Any], store_root: Path, repository_root: Path
) -> dict[str, Any]:
    root = store_root.resolve()
    repository = repository_root.resolve()
    if root == repository or repository in root.parents:
        raise ValueError("generated USD must use an external configured store")
    run_root = root / "derived" / descriptor["body_schema_hash"]
    run_root.mkdir(parents=True, exist_ok=True)
    payload = render_usda(descriptor).encode("utf-8")
    usd_hash = hashlib.sha256(payload).hexdigest()
    usd_path = run_root / "humanoid.usda"
    _atomic_write(usd_path, payload)
    translator_version = (
        BIOMECHANICS_TRANSLATOR_VERSION
        if descriptor.get("translator_id") == BIOMECHANICS_TRANSLATOR_ID
        else TRANSLATOR_VERSION
    )
    manifest = {
        "schema_version": 2,
        "translator_version": translator_version,
        "body_schema_hash": descriptor["body_schema_hash"],
        "compiled_descriptor_hash": descriptor.get("compiled_descriptor_hash"),
        "usd_sha256": usd_hash,
        "usd_path": "humanoid.usda",
    }
    _atomic_write(
        run_root / "translation-manifest.json",
        (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8"),
    )
    return manifest


def _collider_lines(collider: dict[str, Any], indent: str) -> list[str]:
    geometry = collider["geometry"]
    prim = _prim(collider["collider_id"])
    kind = geometry["kind"]
    if kind == "sphere":
        header = f'def Sphere "{prim}"'
        properties = [f"double radius = {geometry['radius_micrometres'] / 1_000_000.0:.9g}"]
    elif kind == "capsule":
        header = f'def Capsule "{prim}"'
        properties = [
            f"double radius = {geometry['radius_micrometres'] / 1_000_000.0:.9g}",
            f"double height = {geometry['half_segment_micrometres'] * 2 / 1_000_000.0:.9g}",
            'uniform token axis = "X"',
        ]
    elif kind == "box":
        header = f'def Cube "{prim}"'
        half = _metres(geometry["half_extents_micrometres"])
        properties = ["double size = 2", f"double3 xformOp:scale = ({_triplet(half)})"]
    else:
        raise ValueError(f"unsupported collider geometry: {kind}")
    output = [f"{indent}{header} (", f'{indent}    prepend apiSchemas = ["PhysicsCollisionAPI"]', f"{indent})", f"{indent}{{"]
    output.extend(f"{indent}    {property_line}" for property_line in properties)
    output.append(f"{indent}}}")
    return output


def _atomic_write(path: Path, payload: bytes) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_bytes(payload)
    os.replace(temporary, path)


def _prim(identifier: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", identifier)


def _world_bind_translations(
    body_by_id: dict[str, dict[str, Any]],
) -> dict[str, tuple[int, int, int]]:
    resolved: dict[str, tuple[int, int, int]] = {}
    visiting: set[str] = set()

    def resolve(body_id: str) -> tuple[int, int, int]:
        if body_id in resolved:
            return resolved[body_id]
        if body_id in visiting:
            raise ValueError("body hierarchy contains a cycle")
        body = body_by_id.get(body_id)
        if body is None:
            raise ValueError(f"body hierarchy references unknown body: {body_id}")
        visiting.add(body_id)
        local = tuple(body["local_bind_translation_micrometres"])
        parent_id = body["parent_body_id"]
        if parent_id is None:
            world = local
        else:
            parent = resolve(parent_id)
            world = tuple(parent[index] + local[index] for index in range(3))
        visiting.remove(body_id)
        resolved[body_id] = world
        return world

    for body_id in body_by_id:
        resolve(body_id)
    return resolved


def _metres(values: tuple[int, int, int] | list[int]) -> tuple[float, float, float]:
    x, y, z = values
    return (x / 1_000_000.0, -z / 1_000_000.0, y / 1_000_000.0)


def _metres_without_axis_swap(
    values: tuple[int, int, int] | list[int],
) -> tuple[float, float, float]:
    x, y, z = values
    return (x / 1_000_000.0, z / 1_000_000.0, y / 1_000_000.0)


def _f32_from_bits(value: int) -> float:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value <= 0xFFFF_FFFF:
        raise ValueError("invalid f32 bit pattern")
    result = struct.unpack("<f", struct.pack("<I", value))[0]
    if not math.isfinite(result):
        raise ValueError("non-finite f32 bit pattern")
    return result


def _isaac_vector_from_f32_bits(values: list[int]) -> tuple[float, float, float]:
    if len(values) != 3:
        raise ValueError("engine vector must have three f32 values")
    x, y, z = (_f32_from_bits(value) for value in values)
    return (x, -z, y)


def _isaac_quaternion_from_f32_bits(
    values: list[int],
) -> tuple[float, float, float, float]:
    if len(values) != 4:
        raise ValueError("engine quaternion must have four f32 values")
    x, y, z, w = (_f32_from_bits(value) for value in values)
    return _normalized_quaternion((w, x, -z, y))


def _isaac_quaternion_from_q1_30(
    values: list[int],
) -> tuple[float, float, float, float]:
    if len(values) != 4:
        raise ValueError("engine quaternion must have four Q1.30 values")
    x, y, z, w = (value / float(1 << 30) for value in values)
    return _normalized_quaternion((w, x, -z, y))


def _normalized_quaternion(
    value: tuple[float, float, float, float],
) -> tuple[float, float, float, float]:
    norm = math.sqrt(sum(component * component for component in value))
    if not math.isfinite(norm) or norm <= 1.0e-12:
        raise ValueError("invalid quaternion")
    result = tuple(component / norm for component in value)
    if result[0] < 0.0:
        result = tuple(-component for component in result)
    return result  # type: ignore[return-value]


def _quaternion(
    *, principal: tuple[float, float, float, float]
) -> str:
    w, x, y, z = principal
    return f"({w:.9g}, {x:.9g}, {y:.9g}, {z:.9g})"


def _triplet(values: tuple[float, float, float]) -> str:
    return ", ".join(f"{value:.9g}" for value in values)
