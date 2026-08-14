from __future__ import annotations

from collections import Counter
from typing import Any

BIOMECHANICS_TRANSLATOR_ID_V2 = "nextengine.isaac.biomechanics-mirror.v2"
BIOMECHANICS_USDA_TRANSLATOR_VERSION_V2 = (
    "nextengine.isaac-biomechanics-usda-translator.v2"
)
MATERIAL_DESCRIPTOR_CONTRACT = "PhysicsMaterialDescriptorV2"
MATERIAL_COMBINE_CONTRACT = "PhysicsMaterialCombineProfileV1"

BODY_MATERIAL_ID = "physics-material.humanoid-body.v1"
GROUND_MATERIAL_ID = "physics-material.humanoid-ground.v1"
SOLE_MATERIAL_ID = "physics-material.humanoid-sole.v1"

EXPECTED_MATERIALS = [
    {
        "canonical_material_tags": [],
        "descriptor_revision": 1,
        "dynamic_friction_q16": 45_875,
        "material_id": BODY_MATERIAL_ID,
        "restitution_q16": 0,
        "rolling_friction_q16": 0,
        "schema_version": 2,
        "spinning_friction_q16": 0,
        "static_friction_q16": 52_429,
        "surface_velocity_micrometres_per_second": [0, 0, 0],
    },
    {
        "canonical_material_tags": [],
        "descriptor_revision": 1,
        "dynamic_friction_q16": 45_875,
        "material_id": GROUND_MATERIAL_ID,
        "restitution_q16": 0,
        "rolling_friction_q16": 0,
        "schema_version": 2,
        "spinning_friction_q16": 0,
        "static_friction_q16": 52_429,
        "surface_velocity_micrometres_per_second": [0, 0, 0],
    },
    {
        "canonical_material_tags": [],
        "descriptor_revision": 1,
        "dynamic_friction_q16": 45_875,
        "material_id": SOLE_MATERIAL_ID,
        "restitution_q16": 0,
        "rolling_friction_q16": 0,
        "schema_version": 2,
        "spinning_friction_q16": 0,
        "static_friction_q16": 52_429,
        "surface_velocity_micrometres_per_second": [0, 0, 0],
    },
]

EXPECTED_COMBINE_PROFILE = {
    "dynamic_friction": "ArithmeticMeanTiesToEven",
    "profile_id": "nextengine.physics-material-combine.humanoid-motor.v1",
    "profile_revision": 1,
    "restitution": "ArithmeticMeanTiesToEven",
    "rolling_friction": "ArithmeticMeanTiesToEven",
    "schema_version": 1,
    "spinning_friction": "ArithmeticMeanTiesToEven",
    "static_friction": "ArithmeticMeanTiesToEven",
    "surface_velocity": "CanonicalParticipantOrder",
}

EXPECTED_ASSIGNMENT_COUNTS = [
    {"collider_count": 17, "material_id": BODY_MATERIAL_ID},
    {"collider_count": 2, "material_id": SOLE_MATERIAL_ID},
]

EXPECTED_DESCRIPTOR_FIELDS = {
    "action_width",
    "actuators",
    "bodies",
    "body_count",
    "body_schema_hash",
    "body_schema_id",
    "body_schema_revision",
    "collider_material_assignment_counts",
    "collision_exclusions",
    "compiled_descriptor_hash",
    "compiled_descriptor_schema_version",
    "coordinate_mapping",
    "effectors",
    "ground_material_id",
    "joints",
    "material_combine_profile",
    "material_lineage_hash",
    "materials",
    "motor_hz",
    "ordered_actuator_ids",
    "ordered_body_ids",
    "physics_hz",
    "schema_version",
    "translator_id",
}


def validate_current_material_lineage(descriptor: dict[str, Any]) -> None:
    """Validate the exact R111 mirror V2 material closure before translation."""
    if (
        set(descriptor) != EXPECTED_DESCRIPTOR_FIELDS
        or descriptor.get("schema_version") != 2
        or descriptor.get("translator_id") != BIOMECHANICS_TRANSLATOR_ID_V2
        or descriptor.get("compiled_descriptor_schema_version") != 3
    ):
        raise ValueError("current biomechanics material descriptor identity mismatch")
    _require_hash(descriptor.get("material_lineage_hash"), "material_lineage_hash")
    if descriptor.get("ground_material_id") != GROUND_MATERIAL_ID:
        raise ValueError("current biomechanics ground material mismatch")
    _validate_numeric_types(descriptor)
    if descriptor.get("materials") != EXPECTED_MATERIALS:
        raise ValueError("current biomechanics material catalog mismatch")
    if descriptor.get("material_combine_profile") != EXPECTED_COMBINE_PROFILE:
        raise ValueError("current biomechanics material combine profile mismatch")
    if (
        descriptor.get("collider_material_assignment_counts")
        != EXPECTED_ASSIGNMENT_COUNTS
    ):
        raise ValueError("current biomechanics declared material counts mismatch")

    actual = Counter(
        collider.get("material_id")
        for body in descriptor.get("bodies", ())
        for collider in body.get("colliders", ())
    )
    expected = Counter(
        {
            BODY_MATERIAL_ID: 17,
            SOLE_MATERIAL_ID: 2,
        }
    )
    if actual != expected:
        raise ValueError("current biomechanics collider material closure mismatch")


def material_catalog(descriptor: dict[str, Any]) -> dict[str, dict[str, Any]]:
    validate_current_material_lineage(descriptor)
    return {row["material_id"]: row for row in descriptor["materials"]}


def _validate_numeric_types(descriptor: dict[str, Any]) -> None:
    material_fields = (
        "schema_version",
        "descriptor_revision",
        "static_friction_q16",
        "dynamic_friction_q16",
        "restitution_q16",
        "rolling_friction_q16",
        "spinning_friction_q16",
    )
    materials = descriptor.get("materials")
    combine = descriptor.get("material_combine_profile")
    counts = descriptor.get("collider_material_assignment_counts")
    if (
        not isinstance(materials, list)
        or not isinstance(combine, dict)
        or not isinstance(counts, list)
        or any(
            type(row.get(field)) is not int
            for row in materials
            if isinstance(row, dict)
            for field in material_fields
        )
        or any(not isinstance(row, dict) for row in materials)
        or any(
            type(value) is not int
            for row in materials
            for value in row.get("surface_velocity_micrometres_per_second", ())
        )
        or type(combine.get("schema_version")) is not int
        or type(combine.get("profile_revision")) is not int
        or any(
            not isinstance(row, dict) or type(row.get("collider_count")) is not int
            for row in counts
        )
    ):
        raise ValueError("current biomechanics material numeric type mismatch")


def _require_hash(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise ValueError(f"{label} must be a lowercase SHA-256 hash")
    return value
