#!/usr/bin/env python3
"""Build the metadata-only V15-S0c source-sufficiency certificate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any


SUFFICIENCY_SCHEMA = (
    "nextengine.experimental-physical-sound-source-sufficiency.v1"
)
ROLE_DECISION_SCHEMA = (
    "nextengine.experimental-physical-sound-role-freeze-decision.v1"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v15-s0c.report.v1"

IDENTITY_SCHEMA = "nextengine.experimental-physical-sound-revision-identity-map.v1"
EXPOSURE_SCHEMA = (
    "nextengine.experimental-physical-sound-revision-aware-exposure-census.v1"
)
INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-source-inventory.v1"
YCB_SCHEMA = "nextengine.experimental-physical-sound-ycb-capability.v1"

MATERIALS = ("Glass", "Metal", "Wood")
TARGET_MATERIAL = "Metal"
REAL_TIERS = {"T2_sparse_real_contact", "T3_dense_real_listener"}
CANDIDATE_STATES = {"metadata_candidate", "member_preflight_required"}
EXPOSURE_STATES = {"exposed", "unexposed"}
ROLE_EXPOSURE_STATES = {
    "generator_exposed",
    "protected_or_unknown_exposed",
    "unexposed",
}
YCB_EXPOSURE_STATES = {
    "not_in_adapter_freshness_unassessed",
    "repository_adapter_referenced",
}
YCB_AXIS_STATES = {"known", "partial", "unknown"}
AXES = (
    "canonical_excitation",
    "contact_geometry_binding",
    "contact_position",
    "geometry",
    "geometry_scale",
    "listener_condition",
    "material_identity",
    "object_identity",
    "recorded_response",
    "support_condition",
)
YCB_AXES = tuple(axis for axis in AXES if axis != "contact_position")
TRAINING_AXES = tuple(axis for axis in AXES if axis != "support_condition")
EVALUATION_AXES = AXES
ROLE_MINIMUM = {
    "admission_shadow": 1,
    "generator_development": 1,
    "generator_train": 4,
    "validator_calibration": 1,
    "validator_method_holdout": 1,
}
PROTECTED_ROLE_ORDER = (
    "validator_calibration",
    "validator_method_holdout",
    "admission_shadow",
)
GENERATOR_ROLE_ORDER = (
    "generator_train",
    "generator_train",
    "generator_train",
    "generator_train",
    "generator_development",
)
HASH_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
MAX_INPUT_BYTES = 8 * 1024 * 1024
MAX_ROWS = 100_000

IDENTITY_KEYS = {
    "alias_edges",
    "authority",
    "discrepancies",
    "groups",
    "input_identities",
    "schema",
}
IDENTITY_GROUP_KEYS = {
    "candidate_routes",
    "identity_basis",
    "material_family",
    "members",
    "numeric_object_ids",
    "physical_group_id",
}
IDENTITY_BASIS_KEYS = {
    "identity_revision",
    "origin_revision",
    "publisher_material",
    "publisher_name",
    "publisher_object_id",
}
IDENTITY_MEMBER_KEYS = {
    "alias_basis",
    "evidence_sha256s",
    "publisher_material",
    "publisher_name",
    "publisher_object_id",
    "revision_id",
    "source_id",
    "source_tier",
}
IDENTITY_ROUTE_KEYS = {
    "candidate_state",
    "inventory_object_id",
    "source_id",
    "source_tier",
}
EXPOSURE_KEYS = {
    "access",
    "authority",
    "candidate_counts",
    "decision",
    "excluded_path_policy",
    "groups",
    "historical_json_manifest",
    "input_identities",
    "metal_scope_remains_potential",
    "realimpact_name_evidence",
    "schema",
}
EXPOSURE_GROUP_KEYS = {
    "candidate_route_count",
    "candidate_source_ids",
    "exposure_evidence",
    "exposure_state",
    "material_family",
    "physical_group_id",
}
EXPOSURE_EVIDENCE_KEYS = {
    "direct_numeric_record_count",
    "path_token_record_count",
    "realimpact_name_record_count",
}
INVENTORY_KEYS = {
    "access",
    "alias_edges",
    "archives",
    "authority",
    "discrepancies",
    "input_identities",
    "objects",
    "publisher_claims",
    "schema",
    "sources",
}
INVENTORY_OBJECT_KEYS = {
    "alias_status",
    "candidate_state",
    "exposure",
    "inventory_object_id",
    "material_family",
    "member_root_sha256",
    "publisher_material",
    "publisher_name",
    "publisher_object_id",
    "reason_codes",
    "source_id",
    "source_tier",
    "structural_presence",
}
YCB_KEYS = {"acquisition_facts", "objects", "project", "schema"}
YCB_OBJECT_KEYS = {
    "capability_axes",
    "evaluation_complete",
    "exposure_state",
    "geometry",
    "material_family",
    "missing_evaluation_axes",
    "missing_training_axes",
    "object_id",
    "object_name",
    "primary_material",
    "recording_parents",
    "secondary_material",
    "training_usable",
}
YCB_GEOMETRY_KEYS = {
    "exact_payload_identity",
    "metric_scale_known",
    "route_count",
    "state",
    "variants",
}
YCB_PARENT_KEYS = {
    "acquisition_mode",
    "conditions",
    "cost",
    "group_id",
    "materialized_path",
    "parent_id",
}

ZERO_ACCESS = {
    "archive_member_bodies_read": 0,
    "force_sample_values_decoded": 0,
    "mesh_values_decoded": 0,
    "network_requests": 0,
    "npy_headers_parsed": 0,
    "pcm_sample_values_decoded": 0,
    "protected_signal_values_decoded": 0,
    "source_payload_bytes_read": 0,
    "video_frames_decoded": 0,
    "wav_headers_parsed": 0,
}


class SourceSufficiencyError(RuntimeError):
    """The S0c input or role decision failed closed."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise SourceSufficiencyError(f"cannot encode canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def reject_constant(value: str) -> None:
    raise SourceSufficiencyError(f"non-finite JSON number is forbidden: {value}")


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise SourceSufficiencyError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SourceSufficiencyError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise SourceSufficiencyError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 1024:
        raise SourceSufficiencyError(f"{context} must be a bounded string")
    return value


def require_hash(value: Any, context: str) -> str:
    if not isinstance(value, str) or not HASH_PATTERN.fullmatch(value):
        raise SourceSufficiencyError(f"{context} must be a lowercase SHA-256")
    return value


def require_bool(value: Any, context: str) -> bool:
    if type(value) is not bool:
        raise SourceSufficiencyError(f"{context} must be boolean")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise SourceSufficiencyError(f"{context} must be a non-negative integer")
    return value


def hash_regular_file(path: Path, context: str) -> tuple[int, str]:
    if path.is_symlink() or not path.is_file():
        raise SourceSufficiencyError(f"{context} must be a regular file")
    before = path.stat()
    if before.st_size > MAX_INPUT_BYTES:
        raise SourceSufficiencyError(f"{context} exceeds frozen byte limit")
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as source:
        while block := source.read(1024 * 1024):
            digest.update(block)
            size += len(block)
    after = path.stat()
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
        after.st_dev,
        after.st_ino,
        after.st_size,
        after.st_mtime_ns,
    ):
        raise SourceSufficiencyError(f"{context} changed while hashing")
    return size, digest.hexdigest()


def read_bound_json(path: Path, context: str, expected_sha256: str) -> tuple[bytes, Any]:
    require_hash(expected_sha256, f"{context} expected SHA-256")
    size, digest = hash_regular_file(path, context)
    if digest != expected_sha256:
        raise SourceSufficiencyError(
            f"{context} identity changed: got bytes={size}, sha256={digest}"
        )
    data = path.read_bytes()
    if len(data) != size:
        raise SourceSufficiencyError(f"{context} changed while reading")
    try:
        value = json.loads(
            data,
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as error:
        raise SourceSufficiencyError(f"cannot parse {context}: {error}") from error
    if canonical_json(value) != data:
        raise SourceSufficiencyError(f"{context} is not canonical JSON")
    return data, value


def validate_identity_map(value: Any) -> dict[str, dict[str, Any]]:
    document = require_exact_keys(value, IDENTITY_KEYS, "identity map")
    if document["schema"] != IDENTITY_SCHEMA:
        raise SourceSufficiencyError("unknown identity-map schema")
    groups = document["groups"]
    if not isinstance(groups, list) or not groups or len(groups) > MAX_ROWS:
        raise SourceSufficiencyError("identity-map groups must be a bounded array")
    result: dict[str, dict[str, Any]] = {}
    for index, raw_group in enumerate(groups):
        group = require_exact_keys(
            raw_group, IDENTITY_GROUP_KEYS, f"identity-map groups[{index}]"
        )
        group_id = require_hash(
            group["physical_group_id"], f"identity-map groups[{index}].physical_group_id"
        )
        if group_id in result:
            raise SourceSufficiencyError("duplicate physical group")
        material = group["material_family"]
        if material is not None and material not in MATERIALS:
            raise SourceSufficiencyError("unknown material family")
        basis = require_exact_keys(
            group["identity_basis"],
            IDENTITY_BASIS_KEYS,
            f"identity-map groups[{index}].identity_basis",
        )
        for field in IDENTITY_BASIS_KEYS:
            require_string(basis[field], f"identity-map groups[{index}].identity_basis.{field}")
        if group_id != sha256_bytes(canonical_json(basis)):
            raise SourceSufficiencyError("physical group identity changed")
        members = group["members"]
        routes = group["candidate_routes"]
        if not isinstance(members, list) or not members:
            raise SourceSufficiencyError("identity group must contain members")
        if not isinstance(routes, list):
            raise SourceSufficiencyError("identity candidate routes must be an array")
        for member_index, member in enumerate(members):
            member = require_exact_keys(
                member,
                IDENTITY_MEMBER_KEYS,
                f"identity-map groups[{index}].members[{member_index}]",
            )
            evidence = member["evidence_sha256s"]
            if not isinstance(evidence, list) or not evidence or evidence != sorted(set(evidence)):
                raise SourceSufficiencyError("identity member evidence is invalid")
            for evidence_hash in evidence:
                require_hash(evidence_hash, "identity member evidence")
        for route_index, route in enumerate(routes):
            route = require_exact_keys(
                route,
                IDENTITY_ROUTE_KEYS,
                f"identity-map groups[{index}].candidate_routes[{route_index}]",
            )
            if route["candidate_state"] not in CANDIDATE_STATES:
                raise SourceSufficiencyError("unknown candidate state")
            require_hash(route["inventory_object_id"], "candidate inventory object ID")
        result[group_id] = group
    if groups != sorted(groups, key=lambda row: row["physical_group_id"]):
        raise SourceSufficiencyError("identity groups are not sorted")
    return result


def validate_exposure_census(
    value: Any, identities: dict[str, dict[str, Any]]
) -> dict[str, dict[str, Any]]:
    document = require_exact_keys(value, EXPOSURE_KEYS, "exposure census")
    if document["schema"] != EXPOSURE_SCHEMA:
        raise SourceSufficiencyError("unknown exposure-census schema")
    groups = document["groups"]
    if not isinstance(groups, list) or len(groups) != len(identities):
        raise SourceSufficiencyError("exposure census group closure changed")
    result: dict[str, dict[str, Any]] = {}
    for index, raw_group in enumerate(groups):
        group = require_exact_keys(
            raw_group, EXPOSURE_GROUP_KEYS, f"exposure-census groups[{index}]"
        )
        group_id = require_hash(group["physical_group_id"], "exposure physical group ID")
        if group_id not in identities or group_id in result:
            raise SourceSufficiencyError("exposure census has missing or duplicate identity")
        if group["material_family"] != identities[group_id]["material_family"]:
            raise SourceSufficiencyError("identity/exposure material mismatch")
        if group["exposure_state"] not in EXPOSURE_STATES:
            raise SourceSufficiencyError("unknown exposure state")
        evidence = require_exact_keys(
            group["exposure_evidence"],
            EXPOSURE_EVIDENCE_KEYS,
            f"exposure-census groups[{index}].exposure_evidence",
        )
        for name, count in evidence.items():
            require_nonnegative_integer(count, f"exposure evidence {name}")
        route_count = require_nonnegative_integer(
            group["candidate_route_count"], "exposure candidate route count"
        )
        routes = identities[group_id]["candidate_routes"]
        if route_count != len(routes):
            raise SourceSufficiencyError("identity/exposure candidate route mismatch")
        expected_sources = sorted({route["source_id"] for route in routes})
        if group["candidate_source_ids"] != expected_sources:
            raise SourceSufficiencyError("identity/exposure candidate sources mismatch")
        result[group_id] = group
    if set(result) != set(identities):
        raise SourceSufficiencyError("exposure census omits identity groups")
    return result


def validate_inventory(value: Any) -> dict[str, dict[str, Any]]:
    document = require_exact_keys(value, INVENTORY_KEYS, "source inventory")
    if document["schema"] != INVENTORY_SCHEMA:
        raise SourceSufficiencyError("unknown source-inventory schema")
    objects = document["objects"]
    if not isinstance(objects, list) or not objects or len(objects) > MAX_ROWS:
        raise SourceSufficiencyError("source inventory objects must be a bounded array")
    result: dict[str, dict[str, Any]] = {}
    for index, raw_item in enumerate(objects):
        item = require_exact_keys(
            raw_item, INVENTORY_OBJECT_KEYS, f"source-inventory objects[{index}]"
        )
        object_id = require_hash(item["inventory_object_id"], "inventory object ID")
        if object_id in result:
            raise SourceSufficiencyError("duplicate inventory object ID")
        state = item["candidate_state"]
        if state not in CANDIDATE_STATES | {"source_ood"}:
            raise SourceSufficiencyError("unknown inventory candidate state")
        material = item["material_family"]
        if material is not None and material not in MATERIALS:
            raise SourceSufficiencyError("unknown inventory material family")
        if state == "metadata_candidate":
            require_hash(item["member_root_sha256"], "metadata candidate member root")
        elif item["member_root_sha256"] is not None:
            require_hash(item["member_root_sha256"], "inventory member root")
        result[object_id] = item
    return result


def route_inventory_rows(
    group: dict[str, Any], inventory: dict[str, dict[str, Any]]
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for route in group["candidate_routes"]:
        item = inventory.get(route["inventory_object_id"])
        if item is None:
            raise SourceSufficiencyError("candidate route is dangling")
        if item["source_id"] != route["source_id"]:
            raise SourceSufficiencyError("candidate route source mismatch")
        if item["source_tier"] != route["source_tier"]:
            raise SourceSufficiencyError("candidate route tier mismatch")
        realimpact_upgrade = (
            route["source_id"] == "realimpact_fca2"
            and route["candidate_state"] == "member_preflight_required"
            and item["candidate_state"] == "source_ood"
            and item["material_family"] is None
            and item["alias_status"] == "ambiguous_revision_identity"
        )
        if item["candidate_state"] != route["candidate_state"] and not realimpact_upgrade:
            raise SourceSufficiencyError("candidate route state mismatch")
        if item["material_family"] != group["material_family"] and not realimpact_upgrade:
            raise SourceSufficiencyError("candidate route material mismatch")
        matching_members = [
            member
            for member in group["members"]
            if member["source_id"] == route["source_id"]
            and member["publisher_object_id"] == item["publisher_object_id"]
            and member["publisher_name"] == item["publisher_name"]
        ]
        if len(matching_members) != 1:
            raise SourceSufficiencyError("candidate route object identity mismatch")
        resolved_item = dict(item)
        resolved_item["candidate_state"] = route["candidate_state"]
        resolved_item["material_family"] = group["material_family"]
        rows.append(resolved_item)
    return rows


def validate_ycb(value: Any) -> list[dict[str, Any]]:
    document = require_exact_keys(value, YCB_KEYS, "YCB capability")
    if document["schema"] != YCB_SCHEMA:
        raise SourceSufficiencyError("unknown YCB capability schema")
    objects = document["objects"]
    if not isinstance(objects, list) or not objects or len(objects) > MAX_ROWS:
        raise SourceSufficiencyError("YCB objects must be a bounded array")
    result: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    seen_parents: set[str] = set()
    for index, raw_item in enumerate(objects):
        item = require_exact_keys(raw_item, YCB_OBJECT_KEYS, f"YCB objects[{index}]")
        object_id = require_string(item["object_id"], "YCB object ID")
        if object_id in seen_ids:
            raise SourceSufficiencyError("duplicate YCB object ID")
        seen_ids.add(object_id)
        material = item["material_family"]
        if material is not None and material not in MATERIALS:
            raise SourceSufficiencyError("unknown YCB material family")
        if item["exposure_state"] not in YCB_EXPOSURE_STATES:
            raise SourceSufficiencyError("unknown YCB exposure state")
        axes = require_exact_keys(
            item["capability_axes"], set(YCB_AXES), f"YCB objects[{index}].axes"
        )
        for axis, state in axes.items():
            if state not in YCB_AXIS_STATES:
                raise SourceSufficiencyError(f"unknown YCB axis state: {axis}")
        geometry = require_exact_keys(
            item["geometry"], YCB_GEOMETRY_KEYS, f"YCB objects[{index}].geometry"
        )
        exact_geometry = require_bool(
            geometry["exact_payload_identity"], "YCB exact geometry identity"
        )
        metric_scale = require_bool(geometry["metric_scale_known"], "YCB metric scale")
        require_nonnegative_integer(geometry["route_count"], "YCB route count")
        parents = item["recording_parents"]
        if not isinstance(parents, list):
            raise SourceSufficiencyError("YCB recording parents must be an array")
        for parent_index, raw_parent in enumerate(parents):
            parent = require_exact_keys(
                raw_parent,
                YCB_PARENT_KEYS,
                f"YCB objects[{index}].recording_parents[{parent_index}]",
            )
            group_id = require_string(parent["group_id"], "YCB recording parent group")
            if group_id in seen_parents:
                raise SourceSufficiencyError("duplicate YCB recording parent")
            seen_parents.add(group_id)
        expected_training_missing = sorted(
            axis for axis in YCB_AXES if axis != "support_condition" and axes[axis] != "known"
        )
        expected_evaluation_missing = sorted(
            axis for axis in YCB_AXES if axes[axis] != "known"
        )
        if item["missing_training_axes"] != expected_training_missing:
            raise SourceSufficiencyError("YCB missing-training axes changed")
        if item["missing_evaluation_axes"] != expected_evaluation_missing:
            raise SourceSufficiencyError("YCB missing-evaluation axes changed")
        training = require_bool(item["training_usable"], "YCB training usable")
        evaluation = require_bool(item["evaluation_complete"], "YCB evaluation complete")
        structurally_training = not expected_training_missing and exact_geometry and metric_scale
        structurally_evaluation = (
            not expected_evaluation_missing and exact_geometry and metric_scale
        )
        if training and not structurally_training:
            raise SourceSufficiencyError("YCB training usable contradicts required axes")
        if evaluation and not structurally_evaluation:
            raise SourceSufficiencyError("YCB evaluation complete contradicts required axes")
        if training or evaluation:
            raise SourceSufficiencyError(
                "YCB role credit cannot omit contact_position or proven freshness"
            )
        result.append(item)
    return result


def candidate_observations(
    identities: dict[str, dict[str, Any]],
    exposures: dict[str, dict[str, Any]],
    inventory: dict[str, dict[str, Any]],
    ycb_objects: list[dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, dict[str, int]]]:
    objectfolder_rows: list[dict[str, Any]] = []
    realimpact_counts = {
        material: {"all_groups": 0, "unexposed_groups": 0} for material in MATERIALS
    }
    for group_id, group in sorted(identities.items()):
        material = group["material_family"]
        if material not in MATERIALS:
            continue
        exposure = exposures[group_id]
        has_realimpact = any(
            member["source_id"] == "realimpact_fca2" for member in group["members"]
        )
        if has_realimpact:
            realimpact_counts[material]["all_groups"] += 1
            if exposure["exposure_state"] == "unexposed":
                realimpact_counts[material]["unexposed_groups"] += 1
        if exposure["exposure_state"] != "unexposed" or not group["candidate_routes"]:
            continue
        rows = route_inventory_rows(group, inventory)
        states = sorted({row["candidate_state"] for row in rows})
        blockers = ["n1b_axis_certificates_absent", "recording_parent_certificate_absent"]
        if "member_preflight_required" in states:
            blockers.append("member_preflight_required")
        objectfolder_rows.append(
            {
                "blockers": sorted(blockers),
                "evaluation_complete": False,
                "exposure_state": "unexposed",
                "material_family": material,
                "physical_group_id": group_id,
                "publisher_name": group["identity_basis"]["publisher_name"],
                "route_states": states,
                "source_ids": sorted({row["source_id"] for row in rows}),
                "training_usable": False,
            }
        )

    ycb_rows: list[dict[str, Any]] = []
    for item in ycb_objects:
        material = item["material_family"]
        if material not in MATERIALS:
            continue
        parents = item["recording_parents"]
        nonambiguous_geometry = (
            item["geometry"]["state"] == "route_available"
            and len(item["geometry"]["variants"]) == 1
        )
        blockers = set(item["missing_training_axes"])
        blockers.add("contact_position_unrepresented")
        if item["exposure_state"] != "unexposed":
            blockers.add("freshness_not_proven")
        if not item["geometry"]["exact_payload_identity"]:
            blockers.add("geometry_payload_identity_absent")
        if not item["geometry"]["metric_scale_known"]:
            blockers.add("geometry_scale_unproven")
        if not parents:
            blockers.add("exact_recording_parent_absent")
        ycb_rows.append(
            {
                "blockers": sorted(blockers),
                "evaluation_complete": False,
                "exposure_state": item["exposure_state"],
                "material_family": material,
                "nonambiguous_geometry_route": nonambiguous_geometry,
                "object_id": item["object_id"],
                "object_name": item["object_name"],
                "recording_parent_ids": sorted(parent["group_id"] for parent in parents),
                "training_usable": False,
            }
        )
    objectfolder_rows.sort(key=lambda row: row["physical_group_id"])
    ycb_rows.sort(key=lambda row: (row["material_family"], row["object_id"]))
    return objectfolder_rows, ycb_rows, realimpact_counts


def validate_candidate(candidate: dict[str, Any]) -> None:
    required = {
        "candidate_id",
        "evaluation_complete",
        "exposure_state",
        "physical_group_id",
        "recording_parent_sha256s",
        "source_id",
        "source_tier",
        "training_usable",
    }
    require_exact_keys(candidate, required, "role candidate")
    require_hash(candidate["physical_group_id"], "candidate physical group")
    require_string(candidate["candidate_id"], "candidate ID")
    require_string(candidate["source_id"], "candidate source ID")
    if candidate["source_tier"] not in REAL_TIERS | {"T0_known_truth", "T1_synthetic_teacher"}:
        raise SourceSufficiencyError("unknown role candidate tier")
    if candidate["exposure_state"] not in ROLE_EXPOSURE_STATES:
        raise SourceSufficiencyError("unknown role candidate exposure")
    require_bool(candidate["training_usable"], "candidate training usability")
    require_bool(candidate["evaluation_complete"], "candidate evaluation completeness")
    if candidate["evaluation_complete"] and not candidate["training_usable"]:
        raise SourceSufficiencyError("evaluation-complete candidate must be training usable")
    parents = candidate["recording_parent_sha256s"]
    if not isinstance(parents, list) or not parents or parents != sorted(set(parents)):
        raise SourceSufficiencyError("candidate recording parents are invalid")
    for parent in parents:
        require_hash(parent, "candidate recording parent")


def select_roles(material: str, candidates: list[dict[str, Any]]) -> dict[str, Any]:
    if material not in MATERIALS:
        raise SourceSufficiencyError("unknown role material")
    for candidate in candidates:
        validate_candidate(candidate)
    if candidates != sorted(candidates, key=lambda row: row["physical_group_id"]):
        raise SourceSufficiencyError("role candidates are not sorted")
    physical_groups = [row["physical_group_id"] for row in candidates]
    if len(physical_groups) != len(set(physical_groups)):
        raise SourceSufficiencyError("duplicate role-candidate physical group")

    protected = [
        row
        for row in candidates
        if row["evaluation_complete"]
        and row["exposure_state"] == "unexposed"
        and row["source_tier"] in REAL_TIERS
    ]
    reserved: list[dict[str, Any]] = []
    used_parents: set[str] = set()
    for row in protected:
        row_parents = set(row["recording_parent_sha256s"])
        if row_parents.isdisjoint(used_parents):
            reserved.append(row)
            used_parents.update(row_parents)
        if len(reserved) == len(PROTECTED_ROLE_ORDER):
            break
    reserved_ids = {row["physical_group_id"] for row in reserved}
    generators: list[dict[str, Any]] = []
    for row in candidates:
        row_parents = set(row["recording_parent_sha256s"])
        if (
            row["training_usable"]
            and row["physical_group_id"] not in reserved_ids
            and row["exposure_state"] in {"generator_exposed", "unexposed"}
            and row_parents.isdisjoint(used_parents)
        ):
            generators.append(row)
            used_parents.update(row_parents)
        if len(generators) == len(GENERATOR_ROLE_ORDER):
            break
    enough = (
        len(reserved) == len(PROTECTED_ROLE_ORDER)
        and len(generators) == len(GENERATOR_ROLE_ORDER)
    )
    assignments: list[dict[str, Any]] = []
    if enough:
        for role, row in zip(PROTECTED_ROLE_ORDER, reserved, strict=True):
            assignments.append(
                {
                    "candidate_id": row["candidate_id"],
                    "physical_group_id": row["physical_group_id"],
                    "recording_parent_sha256s": row["recording_parent_sha256s"],
                    "role": role,
                    "source_id": row["source_id"],
                }
            )
        for role, row in zip(GENERATOR_ROLE_ORDER, generators, strict=True):
            assignments.append(
                {
                    "candidate_id": row["candidate_id"],
                    "physical_group_id": row["physical_group_id"],
                    "recording_parent_sha256s": row["recording_parent_sha256s"],
                    "role": role,
                    "source_id": row["source_id"],
                }
            )
        assignments.sort(key=lambda row: (row["role"], row["physical_group_id"]))
    decision = {
        "assignments": assignments,
        "generator_available": sum(row["training_usable"] for row in candidates),
        "material_family": material,
        "protected_available": len(protected),
        "state": "ROLE_FREEZE_COMPLETE" if enough else "SOURCE_INSUFFICIENT",
    }
    validate_material_decision(decision)
    return decision


def validate_material_decision(decision: dict[str, Any]) -> None:
    require_exact_keys(
        decision,
        {
            "assignments",
            "generator_available",
            "material_family",
            "protected_available",
            "state",
        },
        "material role decision",
    )
    if decision["material_family"] not in MATERIALS:
        raise SourceSufficiencyError("unknown decision material")
    assignments = decision["assignments"]
    if not isinstance(assignments, list):
        raise SourceSufficiencyError("role assignments must be an array")
    if decision["state"] == "SOURCE_INSUFFICIENT":
        if assignments:
            raise SourceSufficiencyError("partial role assignment is forbidden")
        return
    if decision["state"] != "ROLE_FREEZE_COMPLETE":
        raise SourceSufficiencyError("unknown material role decision")
    counts = Counter(row.get("role") for row in assignments)
    if dict(sorted(counts.items())) != ROLE_MINIMUM:
        raise SourceSufficiencyError("role minimum changed or assignment is incomplete")
    physical_groups: set[str] = set()
    parents: set[str] = set()
    for row in assignments:
        require_exact_keys(
            row,
            {
                "candidate_id",
                "physical_group_id",
                "recording_parent_sha256s",
                "role",
                "source_id",
            },
            "role assignment",
        )
        group_id = require_hash(row["physical_group_id"], "assigned physical group")
        if group_id in physical_groups:
            raise SourceSufficiencyError("physical group crosses roles")
        physical_groups.add(group_id)
        for parent in row["recording_parent_sha256s"]:
            require_hash(parent, "assigned recording parent")
            if parent in parents:
                raise SourceSufficiencyError("recording parent crosses roles")
            parents.add(parent)


def build_documents(
    input_identities: dict[str, dict[str, Any]],
    identities: dict[str, dict[str, Any]],
    exposures: dict[str, dict[str, Any]],
    inventory: dict[str, dict[str, Any]],
    ycb_objects: list[dict[str, Any]],
) -> tuple[dict[str, Any], dict[str, Any]]:
    objectfolder_rows, ycb_rows, realimpact_counts = candidate_observations(
        identities, exposures, inventory, ycb_objects
    )
    summaries: list[dict[str, Any]] = []
    material_decisions: list[dict[str, Any]] = []
    for material in MATERIALS:
        objectfolder = [row for row in objectfolder_rows if row["material_family"] == material]
        ycb = [row for row in ycb_rows if row["material_family"] == material]
        metadata_count = sum(
            row["route_states"] == ["metadata_candidate"] for row in objectfolder
        )
        preflight_count = sum(
            "member_preflight_required" in row["route_states"] for row in objectfolder
        )
        exact_ycb = sum(bool(row["recording_parent_ids"]) for row in ycb)
        route_ycb = sum(
            bool(row["recording_parent_ids"]) and row["nonambiguous_geometry_route"]
            for row in ycb
        )
        decision = select_roles(material, [])
        material_decisions.append(decision)
        blockers = ["no_training_usable_groups", "no_evaluation_complete_groups"]
        if material == "Wood":
            blockers.append("no_fresh_objectfolder_candidates")
        if material == "Glass":
            blockers.append("no_exact_ycb_vertical_parents")
        summaries.append(
            {
                "blockers": sorted(blockers),
                "decision": decision["state"],
                "evaluation_complete_groups": 0,
                "generator_required_groups": 5,
                "material_family": material,
                "objectfolder_member_preflight_groups": preflight_count,
                "objectfolder_metadata_candidate_groups": metadata_count,
                "objectfolder_unexposed_candidate_groups": len(objectfolder),
                "protected_required_groups": 3,
                "realimpact_all_groups": realimpact_counts[material]["all_groups"],
                "realimpact_unexposed_groups": realimpact_counts[material]["unexposed_groups"],
                "training_usable_groups": 0,
                "ycb_exact_parent_groups": exact_ycb,
                "ycb_nonambiguous_route_parent_groups": route_ycb,
            }
        )

    sufficiency = {
        "authority": {
            "authorizes_signal_access": False,
            "authorizes_s1": False,
            "fallback_required": True,
            "public_contract": False,
            "runtime_consumer_allowed": False,
        },
        "input_identities": input_identities,
        "material_summaries": summaries,
        "objectfolder_candidates": objectfolder_rows,
        "policy": {
            "evaluation_required_axes": list(EVALUATION_AXES),
            "partition_unit": "physical_object_group_id",
            "role_minimum": ROLE_MINIMUM,
            "training_required_axes": list(TRAINING_AXES),
        },
        "realimpact_exposure": realimpact_counts,
        "schema": SUFFICIENCY_SCHEMA,
        "ycb_candidates": ycb_rows,
    }
    any_complete = any(row["state"] == "ROLE_FREEZE_COMPLETE" for row in material_decisions)
    role_decision = {
        "authority": {
            "authorizes_only": (
                "s1_preregistration" if any_complete else "published_source_growth"
            ),
            "fallback_required": True,
            "public_contract": False,
            "runtime_consumer_allowed": False,
        },
        "decision": (
            "S0C_ROLE_FREEZE_PASS_S1_NEXT"
            if any_complete
            else "S0C_SOURCE_INSUFFICIENT_SOURCE_GROWTH_REQUIRED"
        ),
        "material_decisions": material_decisions,
        "policy_provenance": {
            "n1a_protocol_sha256": (
                "a8a25ec63fb402867c10a96efd48cfc360a6af2004a9987bee8205351253a754"
            ),
            "role_minimum": ROLE_MINIMUM,
        },
        "schema": ROLE_DECISION_SCHEMA,
    }
    return sufficiency, role_decision


def atomic_write_documents(output: Path, documents: dict[str, dict[str, Any]]) -> None:
    if output.exists() or output.is_symlink():
        raise SourceSufficiencyError("output directory already exists")
    candidate = output.parent.resolve() / output.name
    try:
        candidate.relative_to(repository_root().resolve())
    except ValueError:
        pass
    else:
        raise SourceSufficiencyError("output directory must remain outside the repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, value in documents.items():
            path = temporary / name
            path.write_bytes(canonical_json(value))
        os.replace(temporary, output)
    except BaseException:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def build(
    identity_path: Path,
    exposure_path: Path,
    inventory_path: Path,
    ycb_path: Path,
    expected_hashes: dict[str, str],
    output: Path,
) -> Path:
    raw_inputs: dict[str, bytes] = {}
    values: dict[str, Any] = {}
    paths = {
        "exposure_census": exposure_path,
        "identity_map": identity_path,
        "source_inventory": inventory_path,
        "ycb_capability": ycb_path,
    }
    if set(expected_hashes) != set(paths):
        raise SourceSufficiencyError("expected input hash set changed")
    for name, path in paths.items():
        data, value = read_bound_json(path, name, expected_hashes[name])
        raw_inputs[name] = data
        values[name] = value
    identities = validate_identity_map(values["identity_map"])
    exposures = validate_exposure_census(values["exposure_census"], identities)
    inventory = validate_inventory(values["source_inventory"])
    ycb_objects = validate_ycb(values["ycb_capability"])
    input_identities = {
        name: {"bytes": len(raw_inputs[name]), "sha256": sha256_bytes(raw_inputs[name])}
        for name in sorted(raw_inputs)
    }
    sufficiency, role_decision = build_documents(
        input_identities, identities, exposures, inventory, ycb_objects
    )
    sufficiency_bytes = canonical_json(sufficiency)
    role_bytes = canonical_json(role_decision)
    report = {
        "access": ZERO_ACCESS,
        "counts": {
            "identity_group_count": len(identities),
            "material_count": len(MATERIALS),
            "objectfolder_candidate_count": len(sufficiency["objectfolder_candidates"]),
            "role_assignment_count": sum(
                len(row["assignments"])
                for row in role_decision["material_decisions"]
            ),
            "ycb_candidate_count": len(sufficiency["ycb_candidates"]),
        },
        "decision": role_decision["decision"],
        "input_identities": input_identities,
        "output_identities": {
            "role-freeze-decision.json": {
                "bytes": len(role_bytes),
                "sha256": sha256_bytes(role_bytes),
            },
            "source-sufficiency.json": {
                "bytes": len(sufficiency_bytes),
                "sha256": sha256_bytes(sufficiency_bytes),
            },
        },
        "schema": REPORT_SCHEMA,
    }
    atomic_write_documents(
        output,
        {
            "report.json": report,
            "role-freeze-decision.json": role_decision,
            "source-sufficiency.json": sufficiency,
        },
    )
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--identity-map", required=True, type=Path)
    parser.add_argument("--identity-map-sha256", required=True)
    parser.add_argument("--exposure-census", required=True, type=Path)
    parser.add_argument("--exposure-census-sha256", required=True)
    parser.add_argument("--source-inventory", required=True, type=Path)
    parser.add_argument("--source-inventory-sha256", required=True)
    parser.add_argument("--ycb-capability", required=True, type=Path)
    parser.add_argument("--ycb-capability-sha256", required=True)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    build(
        arguments.identity_map,
        arguments.exposure_census,
        arguments.source_inventory,
        arguments.ycb_capability,
        {
            "exposure_census": arguments.exposure_census_sha256,
            "identity_map": arguments.identity_map_sha256,
            "source_inventory": arguments.source_inventory_sha256,
            "ycb_capability": arguments.ycb_capability_sha256,
        },
        arguments.output,
    )


if __name__ == "__main__":
    main()
