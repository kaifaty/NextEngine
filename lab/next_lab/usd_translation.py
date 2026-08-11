from __future__ import annotations

import hashlib
import json
import os
import re
from pathlib import Path
from typing import Any

from next_lab.motor_mirror import validate_descriptor

TRANSLATOR_VERSION = "nextengine.isaac-usda-translator.v2"


def render_usda(descriptor: dict[str, Any]) -> str:
    validate_descriptor(descriptor)
    if descriptor.get("translator_version") != TRANSLATOR_VERSION:
        raise ValueError("translator version mismatch")
    body_by_id = {record["body_id"]: record for record in descriptor["bodies"]}
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
        translation = _metres(body["local_bind_translation_micrometres"])
        mass = body["mass_microkilograms"] / 1_000_000.0
        lines.extend(
            [
                f'        def Xform "{prim}" (',
                '            prepend apiSchemas = ["PhysicsRigidBodyAPI", "PhysicsMassAPI"]',
                "        )",
                "        {",
                f"            double3 xformOp:translate = ({_triplet(translation)})",
                '            uniform token[] xformOpOrder = ["xformOp:translate"]',
                f"            float physics:mass = {mass:.9g}",
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
        lines.extend(
            [
                f'        def PhysicsRevoluteJoint "{prim}"',
                "        {",
                f"            rel physics:body0 = </Humanoid/Bodies/{parent}>",
                f"            rel physics:body1 = </Humanoid/Bodies/{child}>",
                '            uniform token physics:axis = "X"',
                f"            float physics:lowerLimit = {lower:.9g}",
                f"            float physics:upperLimit = {upper:.9g}",
                f'            custom string nextengine:semanticId = "{joint["joint_id"]}"',
                "        }",
            ]
        )
    lines.extend(["    }", "}", ""])
    return "\n".join(lines)


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
    manifest = {
        "schema_version": 2,
        "translator_version": TRANSLATOR_VERSION,
        "body_schema_hash": descriptor["body_schema_hash"],
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


def _metres(values: list[int]) -> tuple[float, float, float]:
    x, y, z = values
    return (x / 1_000_000.0, -z / 1_000_000.0, y / 1_000_000.0)


def _triplet(values: tuple[float, float, float]) -> str:
    return ", ".join(f"{value:.9g}" for value in values)
