#!/usr/bin/env python3
"""Audit V29 Q1-M exact-Steel role power without reading signal values."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v29-q1m-role-power-audit.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v29-q1m.report.v1"
Q0_INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-v29-q0m-inventory.v1"
Q0_REPORT_SCHEMA = "nextengine.experimental-physical-sound-v29-q0m.report.v1"
EXPOSURE_SCHEMA = "nextengine.experimental-physical-sound-revision-aware-exposure-census.v1"
IDENTITY_MAP_SCHEMA = "nextengine.experimental-physical-sound-revision-identity-map.v1"
YCB_SCHEMA = "nextengine.experimental-physical-sound-ycb-capability.v1"

Q0_INVENTORY_IDENTITY = (
    172_172,
    "9a04c8a2fb9f0c6a3d351dac2796936d933d223e9400afc79a95f06e01665f43",
)
Q0_REPORT_IDENTITY = (
    2_018,
    "a33c0e2eae12663d8c8dc90cca926b1d614bec332a07e4dddea4d75bf403b9d5",
)
EXPOSURE_IDENTITY = (
    2_318_665,
    "ce3c5ab861a91bd85fc9a5a90f24629f15dbdc8d9faa936cd04e791efa667365",
)
IDENTITY_MAP_IDENTITY = (
    266_858,
    "66e9a29e69b72b95836beef9dbd8d7d52cd33b1213c709fc2f57422139d1ea40",
)
YCB_IDENTITY = (
    227_019,
    "7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf",
)

MAX_INPUT_BYTES = 4 * 1024 * 1024
MIN_PROTECTED_POSITIVES = 16
MIN_PROTECTED_REJECTS = 35
PROTECTED_EVALUATIONS = ("validator-holdout", "joint-admission-shadow")
ROLE_REQUIREMENTS = (
    ("validator-development", 1, False),
    ("validator-calibration", 1, False),
    ("validator-holdout", 2, True),
    ("generator-training", 1, False),
    ("generator-development", 1, False),
    ("generator-method-holdout", 1, False),
    ("joint-admission-shadow", 2, True),
)
FULL_ROLE_PROJECT_FLOOR = sum(requirement[1] for requirement in ROLE_REQUIREMENTS)
PROTECTED_PROJECT_FLOOR = sum(
    requirement[1] for requirement in ROLE_REQUIREMENTS if requirement[2]
)

SIGNAL_COUNTERS = {
    "archive_member_bodies_read": 0,
    "audio_headers_parsed": 0,
    "force_sample_values_decoded": 0,
    "mesh_values_decoded": 0,
    "network_requests": 0,
    "pcm_sample_values_decoded": 0,
    "protected_signal_values_decoded": 0,
    "role_signal_values_opened": 0,
    "source_payload_bytes_read": 0,
}
FORBIDDEN_INPUT_ACCESS_COUNTERS = set(SIGNAL_COUNTERS) | {
    "network_archive_body_bytes",
    "npy_headers_parsed",
    "source_payload_members_extracted",
    "wav_headers_parsed",
}

Q0_INVENTORY_KEYS = {
    "access",
    "authority",
    "groups",
    "historical_glass_split",
    "schema",
    "sources",
}
Q0_REPORT_KEYS = {
    "access",
    "available_counts",
    "decision",
    "excluded_group_count",
    "group_count",
    "input_identities",
    "inventory_sha256",
    "minimums",
    "project_revision_counts",
    "q1_blockers",
    "schema",
}
Q0_GROUP_BASE_KEYS = {
    "acquisition_axes",
    "candidate_available_after_historical_exclusion",
    "group_id",
    "historical_exclusion",
    "material_relation",
    "object_id",
    "object_name",
    "primary_material",
    "project_id",
    "publisher_id",
    "recording_parent_id",
    "revision",
    "role_state",
    "secondary_material",
    "source_id",
}
Q0_GROUP_YCB_EXTRA_KEYS = {"missing_evaluation_axes", "missing_training_axes"}
Q0_SOURCE_KEYS = {
    "landing_page_url",
    "license_expression",
    "project_id",
    "publisher_id",
    "redistribution_policy",
    "revision",
}
Q0_HISTORY_KEYS = {
    "manifest_sha256",
    "matched_objectfolder_groups",
    "matched_ycb_groups",
    "policy",
    "target_material_label",
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
IDENTITY_MAP_KEYS = {
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
YCB_KEYS = {"acquisition_facts", "objects", "project", "schema"}
YCB_OBJECT_REQUIRED_KEYS = {
    "exposure_state",
    "object_id",
    "primary_material",
    "recording_parents",
}
YCB_PARENT_REQUIRED_KEYS = {"group_id", "parent_id"}


class Q1MRolePowerError(RuntimeError):
    """A Q1-M input or result violates the frozen metadata-only protocol."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--q0-inventory", required=True, type=Path)
    parser.add_argument("--q0-report", required=True, type=Path)
    parser.add_argument("--exposure-census", required=True, type=Path)
    parser.add_argument("--identity-map", required=True, type=Path)
    parser.add_argument("--ycb-inventory", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise Q1MRolePowerError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise Q1MRolePowerError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Q1MRolePowerError(f"{context} is not strict JSON: {error}") from error
    if not isinstance(value, dict):
        raise Q1MRolePowerError(f"{context} root must be an object")
    return value


def require_keys(value: Any, keys: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != keys:
        raise Q1MRolePowerError(f"{context} has unknown or missing fields")
    return value


def require_string(value: Any, context: str, maximum: int = 512) -> str:
    if not isinstance(value, str) or not 1 <= len(value) <= maximum:
        raise Q1MRolePowerError(f"{context} must be a bounded string")
    return value


def project_revision_id(group: dict[str, Any]) -> str:
    return "--".join(
        (group["publisher_id"], group["project_id"], group["revision"])
    )


def material_relation(material: str) -> str:
    if material == "Steel":
        return "exact_steel_candidate"
    if material in {"Iron", "Aluminium"}:
        return "other_metal_candidate"
    return "non_metal_candidate"


def validate_zero_signal_access(access: Any, context: str) -> dict[str, int]:
    if not isinstance(access, dict):
        raise Q1MRolePowerError(f"{context} access must be an object")
    for key, value in access.items():
        if not isinstance(key, str) or not isinstance(value, int) or value < 0:
            raise Q1MRolePowerError(f"{context} access counters are invalid")
        if key in FORBIDDEN_INPUT_ACCESS_COUNTERS and value != 0:
            raise Q1MRolePowerError(f"{context} opened signal-bearing evidence")
    return access


def validate_q0(
    inventory_data: bytes, report_data: bytes
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    inventory = require_keys(
        parse_json(inventory_data, "Q0 inventory"),
        Q0_INVENTORY_KEYS,
        "Q0 inventory",
    )
    report = require_keys(
        parse_json(report_data, "Q0 report"), Q0_REPORT_KEYS, "Q0 report"
    )
    if (
        inventory["schema"] != Q0_INVENTORY_SCHEMA
        or inventory["authority"]
        != "SOURCE_IDENTITY_AND_DECLARED_AXIS_INVENTORY_ONLY / NO_ROLE_OR_ADMISSION_AUTHORITY"
        or report["schema"] != Q0_REPORT_SCHEMA
        or report["decision"]
        != "Q0M_METAL_SOURCE_INVENTORY_FEASIBLE_Q1M_NEXT"
    ):
        raise Q1MRolePowerError("Q0 decision or authority is not the frozen feasible input")
    if report["inventory_sha256"] != sha256_bytes(inventory_data):
        raise Q1MRolePowerError("Q0 report does not bind the supplied inventory")
    validate_zero_signal_access(inventory["access"], "Q0 inventory")
    validate_zero_signal_access(report["access"], "Q0 report")
    if inventory["access"] != report["access"]:
        raise Q1MRolePowerError("Q0 inventory/report access ledgers differ")

    history = require_keys(
        inventory["historical_glass_split"], Q0_HISTORY_KEYS, "Q0 history"
    )
    if (
        history["target_material_label"] != "Glass"
        or history["policy"] != "exclude_without_relabelling_or_threshold_use"
    ):
        raise Q1MRolePowerError("historical Glass exclusion policy changed")
    sources = inventory["sources"]
    if not isinstance(sources, list) or len(sources) != 2:
        raise Q1MRolePowerError("Q0 must contain exactly two source projects")
    source_projects: set[str] = set()
    for source in sources:
        source = require_keys(source, Q0_SOURCE_KEYS, "Q0 source")
        source_projects.add(
            "--".join((source["publisher_id"], source["project_id"], source["revision"]))
        )
    if len(source_projects) != 2:
        raise Q1MRolePowerError("Q0 source project identities are not unique")

    groups = inventory["groups"]
    if not isinstance(groups, list) or len(groups) != report["group_count"]:
        raise Q1MRolePowerError("Q0 group count does not match its report")
    seen: set[str] = set()
    available: list[dict[str, Any]] = []
    excluded: list[dict[str, Any]] = []
    for raw in groups:
        if not isinstance(raw, dict):
            raise Q1MRolePowerError("Q0 group must be an object")
        allowed = Q0_GROUP_BASE_KEYS
        if raw.get("project_id") == "ycb-impact-sounds":
            allowed = allowed | Q0_GROUP_YCB_EXTRA_KEYS
        if set(raw) != allowed:
            raise Q1MRolePowerError("Q0 group has unknown or missing fields")
        group_id = require_string(raw["group_id"], "Q0 group ID")
        if group_id in seen:
            raise Q1MRolePowerError("duplicate Q0 physical group")
        seen.add(group_id)
        for field in (
            "publisher_id",
            "project_id",
            "revision",
            "object_id",
            "object_name",
            "primary_material",
            "source_id",
        ):
            require_string(raw[field], f"Q0 {field}")
        if raw["role_state"] != "unassigned":
            raise Q1MRolePowerError("Q0 assigned a role before Q1-M")
        expected_relation = material_relation(raw["primary_material"])
        if raw["material_relation"] != expected_relation:
            raise Q1MRolePowerError("Q0 material relation changed or pooled Metal")
        is_available = raw["candidate_available_after_historical_exclusion"]
        if not isinstance(is_available, bool):
            raise Q1MRolePowerError("Q0 availability must be boolean")
        if is_available != (raw["historical_exclusion"] is None):
            raise Q1MRolePowerError("Q0 historical availability is inconsistent")
        if project_revision_id(raw) not in source_projects:
            raise Q1MRolePowerError("Q0 group references an unknown project revision")
        (available if is_available else excluded).append(raw)

    if len(excluded) != report["excluded_group_count"]:
        raise Q1MRolePowerError("Q0 excluded count does not match")
    counts = Counter(group["material_relation"] for group in available)
    expected_counts = report["available_counts"]
    if (
        counts["exact_steel_candidate"] != expected_counts["exact_steel_candidates"]
        or counts["other_metal_candidate"] != expected_counts["other_metal_candidates"]
        or counts["non_metal_candidate"] != expected_counts["non_metal_candidates"]
        or counts["exact_steel_candidate"] + counts["other_metal_candidate"]
        != expected_counts["broad_metal_candidates"]
    ):
        raise Q1MRolePowerError("Q0 available counts do not match its groups")
    return available, excluded, report


def validate_exposure_census(data: bytes) -> dict[str, dict[str, Any]]:
    document = require_keys(
        parse_json(data, "exposure census"), EXPOSURE_KEYS, "exposure census"
    )
    if (
        document["schema"] != EXPOSURE_SCHEMA
        or document["decision"]
        != "S0A_IDENTITY_EXPOSURE_PASS_METAL_SCOPE_REMAINS_POTENTIAL"
    ):
        raise Q1MRolePowerError("unsupported exposure census identity")
    validate_zero_signal_access(document["access"], "exposure census")
    result: dict[str, dict[str, Any]] = {}
    groups = document["groups"]
    if not isinstance(groups, list):
        raise Q1MRolePowerError("exposure census groups must be a list")
    for raw in groups:
        row = require_keys(raw, EXPOSURE_GROUP_KEYS, "exposure group")
        group_id = require_string(row["physical_group_id"], "physical group ID")
        if group_id in result:
            raise Q1MRolePowerError("duplicate exposure physical group")
        if row["exposure_state"] not in {"exposed", "unexposed"}:
            raise Q1MRolePowerError("unknown historical exposure state")
        if not isinstance(row["candidate_source_ids"], list):
            raise Q1MRolePowerError("exposure candidate sources must be a list")
        result[group_id] = row
    return result


def validate_identity_map(data: bytes) -> dict[tuple[str, str, str, str], str]:
    document = require_keys(
        parse_json(data, "identity map"), IDENTITY_MAP_KEYS, "identity map"
    )
    if document["schema"] != IDENTITY_MAP_SCHEMA:
        raise Q1MRolePowerError("unsupported identity-map schema")
    groups = document["groups"]
    if not isinstance(groups, list):
        raise Q1MRolePowerError("identity-map groups must be a list")
    result: dict[tuple[str, str, str, str], str] = {}
    for raw in groups:
        group = require_keys(raw, IDENTITY_GROUP_KEYS, "identity group")
        physical_group_id = require_string(
            group["physical_group_id"], "identity physical group ID"
        )
        members = group["members"]
        if not isinstance(members, list):
            raise Q1MRolePowerError("identity members must be a list")
        for raw_member in members:
            member = require_keys(raw_member, IDENTITY_MEMBER_KEYS, "identity member")
            if member["source_id"] != "objectfolder_real_current":
                continue
            key = (
                require_string(member["publisher_object_id"], "ObjectFolder object ID"),
                require_string(member["publisher_name"], "ObjectFolder name"),
                require_string(member["publisher_material"], "ObjectFolder material"),
                require_string(member["revision_id"], "ObjectFolder revision"),
            )
            previous = result.setdefault(key, physical_group_id)
            if previous != physical_group_id:
                raise Q1MRolePowerError("ambiguous ObjectFolder physical-group join")
    return result


def validate_ycb(data: bytes) -> dict[str, dict[str, Any]]:
    document = require_keys(parse_json(data, "YCB inventory"), YCB_KEYS, "YCB inventory")
    if document["schema"] != YCB_SCHEMA:
        raise Q1MRolePowerError("unsupported YCB capability schema")
    project = document["project"]
    if not isinstance(project, dict) or project != {
        "component_revision": "2022-09-27T11:36:47.167853",
        "id": "ycb-impact-sounds",
        "publisher": "iri-csic-upc-ctu",
    }:
        raise Q1MRolePowerError("YCB project identity changed")
    objects = document["objects"]
    if not isinstance(objects, list) or len(objects) != 77:
        raise Q1MRolePowerError("YCB inventory must contain 77 objects")
    result: dict[str, dict[str, Any]] = {}
    parent_count = 0
    for raw in objects:
        if not isinstance(raw, dict) or not YCB_OBJECT_REQUIRED_KEYS.issubset(raw):
            raise Q1MRolePowerError("YCB object is missing required identity fields")
        object_id = require_string(raw["object_id"], "YCB object ID", 8)
        if object_id in result:
            raise Q1MRolePowerError("duplicate YCB object ID")
        state = raw["exposure_state"]
        if state not in {
            "repository_adapter_referenced",
            "not_in_adapter_freshness_unassessed",
        }:
            raise Q1MRolePowerError("unknown YCB exposure state")
        parents = raw["recording_parents"]
        if not isinstance(parents, list) or len(parents) > 1:
            raise Q1MRolePowerError("YCB row has ambiguous recording parents")
        for parent in parents:
            if not isinstance(parent, dict) or not YCB_PARENT_REQUIRED_KEYS.issubset(parent):
                raise Q1MRolePowerError("YCB parent lacks exact identity")
            require_string(parent["group_id"], "YCB group ID")
            require_string(parent["parent_id"], "YCB parent ID")
            parent_count += 1
        result[object_id] = raw
    if parent_count != 39:
        raise Q1MRolePowerError("YCB inventory must contain 39 exact recording parents")
    return result


def join_exposure(
    groups: list[dict[str, Any]],
    exposure: dict[str, dict[str, Any]],
    identity_map: dict[tuple[str, str, str, str], str],
    ycb: dict[str, dict[str, Any]],
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for group in groups:
        if group["project_id"] == "objectfolder-real":
            key = (
                group["object_id"],
                group["object_name"],
                group["primary_material"],
                group["revision"],
            )
            physical_group_id = identity_map.get(key)
            if physical_group_id is None:
                joined_state = "not_in_s0a_identity_map_unassessed"
                evidence = "v15_s0a_identity_map_absence"
            else:
                exposure_group = exposure.get(physical_group_id)
                if exposure_group is None:
                    raise Q1MRolePowerError(
                        "ObjectFolder identity map points outside exposure census"
                    )
                if "objectfolder_real_current" not in exposure_group["candidate_source_ids"]:
                    joined_state = "identity_mapped_but_not_exposure_candidate_unassessed"
                else:
                    joined_state = (
                        "historical_exposed"
                        if exposure_group["exposure_state"] == "exposed"
                        else "historical_unexposed_not_currently_certified"
                    )
                evidence = physical_group_id
        elif group["project_id"] == "ycb-impact-sounds":
            row = ycb.get(group["object_id"])
            if row is None or len(row["recording_parents"]) != 1:
                raise Q1MRolePowerError("Q0 YCB object lacks its exact capability parent")
            parent = row["recording_parents"][0]
            if (
                parent["group_id"] != group["group_id"]
                or parent["parent_id"] != group["recording_parent_id"]
                or row["primary_material"] != group["primary_material"]
            ):
                raise Q1MRolePowerError("Q0/YCB exact parent identity mismatch")
            joined_state = (
                "historical_exposed"
                if row["exposure_state"] == "repository_adapter_referenced"
                else "historical_freshness_unassessed"
            )
            evidence = f"ycb-capability:{row['exposure_state']}"
        else:
            raise Q1MRolePowerError("unsupported Q0 source project")
        rows.append(
            {
                "exposure_evidence": evidence,
                "exposure_state": joined_state,
                "freshness_eligible": False,
                "group_id": group["group_id"],
                "material_relation": group["material_relation"],
                "object_id": group["object_id"],
                "primary_material": group["primary_material"],
                "project_revision_id": project_revision_id(group),
                "role_state": "unassigned_source_power_ood",
                "source_id": group["source_id"],
            }
        )
    rows.sort(key=lambda row: row["group_id"])
    if len(rows) != len(groups) or len({row["group_id"] for row in rows}) != len(rows):
        raise Q1MRolePowerError("exposure join is not a complete one-to-one accounting")
    return rows


def build_documents(
    q0_inventory_data: bytes,
    q0_report_data: bytes,
    exposure_data: bytes,
    identity_map_data: bytes,
    ycb_data: bytes,
) -> dict[str, bytes]:
    available, excluded, q0_report = validate_q0(q0_inventory_data, q0_report_data)
    exposure = validate_exposure_census(exposure_data)
    identity_map = validate_identity_map(identity_map_data)
    ycb = validate_ycb(ycb_data)
    joined = join_exposure(available, exposure, identity_map, ycb)

    raw_counts = Counter(row["material_relation"] for row in joined)
    broad_metal = (
        raw_counts["exact_steel_candidate"] + raw_counts["other_metal_candidate"]
    )
    projects = sorted({row["project_revision_id"] for row in joined})
    projects_by_relation: dict[str, set[str]] = defaultdict(set)
    for row in joined:
        projects_by_relation[row["material_relation"]].add(row["project_revision_id"])
    exposure_counts = Counter(row["exposure_state"] for row in joined)
    freshness_counts = Counter(
        row["material_relation"] for row in joined if row["freshness_eligible"]
    )

    protected_positive_floor = MIN_PROTECTED_POSITIVES * len(PROTECTED_EVALUATIONS)
    protected_reject_floor = MIN_PROTECTED_REJECTS * len(PROTECTED_EVALUATIONS)
    remaining_exact = raw_counts["exact_steel_candidate"] - protected_positive_floor
    remaining_reject = raw_counts["non_metal_candidate"] - protected_reject_floor

    blockers: list[str] = []
    if len(projects) < FULL_ROLE_PROJECT_FLOOR:
        blockers.append(
            f"full_role_project_topology_requires_{FULL_ROLE_PROJECT_FLOOR}_has_{len(projects)}"
        )
    if len(projects) < PROTECTED_PROJECT_FLOOR:
        blockers.append(
            "protected_role_project_topology_requires_"
            f"{PROTECTED_PROJECT_FLOOR}_has_{len(projects)}"
        )
    if raw_counts["exact_steel_candidate"] < protected_positive_floor:
        blockers.append(
            "exact_steel_protected_minimum_requires_"
            f"{protected_positive_floor}_has_{raw_counts['exact_steel_candidate']}"
        )
    if raw_counts["non_metal_candidate"] < protected_reject_floor:
        blockers.append(
            "non_metal_protected_minimum_requires_"
            f"{protected_reject_floor}_has_{raw_counts['non_metal_candidate']}"
        )
    if remaining_reject <= 0:
        blockers.append(
            "non_metal_power_exhausted_by_protected_minima_leaves_"
            f"{max(0, remaining_reject)}_for_development_and_calibration"
        )
    if not any(row["freshness_eligible"] for row in joined):
        blockers.append("current_freshness_not_certified_for_any_available_group")
    if any(count == 0 for count in (
        freshness_counts["exact_steel_candidate"],
        freshness_counts["non_metal_candidate"],
    )):
        blockers.append("freshness_qualified_class_power_is_zero")

    decision = (
        "Q1M_ROLE_POWER_FREEZE_VERIFIED_Q2_AND_P2_ELIGIBLE"
        if not blockers
        else "Q1M_SOURCE_POWER_INSUFFICIENT_SOURCE_GROWTH_REQUIRED"
    )
    if decision == "Q1M_ROLE_POWER_FREEZE_VERIFIED_Q2_AND_P2_ELIGIBLE":
        raise Q1MRolePowerError(
            "positive role assignment is unsupported by the frozen historical-freshness inputs"
        )

    access = {
        **SIGNAL_COUNTERS,
        "metadata_bytes_read": sum(
            map(
                len,
                (
                    q0_inventory_data,
                    q0_report_data,
                    exposure_data,
                    identity_map_data,
                    ycb_data,
                ),
            )
        ),
    }
    required_roles = [
        {
            "minimum_project_revisions": project_count,
            "protected_evaluation": protected,
            "role": role,
            **(
                {
                    "minimum_exact_steel_groups": MIN_PROTECTED_POSITIVES,
                    "minimum_non_metal_reject_groups": MIN_PROTECTED_REJECTS,
                }
                if protected
                else {}
            ),
        }
        for role, project_count, protected in ROLE_REQUIREMENTS
    ]
    audit = {
        "access": access,
        "authority": "METADATA_ONLY_SOURCE_POWER_OOD / NO_ROLE_OR_PAYLOAD_ACCESS_AUTHORITY",
        "blockers": blockers,
        "decision": decision,
        "excluded_historical_group_count": len(excluded),
        "exposure_accounting": {
            "accounted_available_groups": len(joined),
            "available_group_count": len(available),
            "states": dict(sorted(exposure_counts.items())),
        },
        "groups": joined,
        "input_identities": {
            "exposure_census": {
                "bytes": len(exposure_data),
                "sha256": sha256_bytes(exposure_data),
            },
            "identity_map": {
                "bytes": len(identity_map_data),
                "sha256": sha256_bytes(identity_map_data),
            },
            "q0_inventory": {
                "bytes": len(q0_inventory_data),
                "sha256": sha256_bytes(q0_inventory_data),
            },
            "q0_report": {
                "bytes": len(q0_report_data),
                "sha256": sha256_bytes(q0_report_data),
            },
            "ycb_inventory": {
                "bytes": len(ycb_data),
                "sha256": sha256_bytes(ycb_data),
            },
        },
        "power": {
            "freshness_qualified_counts": {
                "exact_steel_candidates": freshness_counts["exact_steel_candidate"],
                "non_metal_candidates": freshness_counts["non_metal_candidate"],
                "other_metal_candidates": freshness_counts["other_metal_candidate"],
            },
            "project_revision_count": len(projects),
            "project_revision_counts_by_relation": {
                relation: len(projects_by_relation[relation])
                for relation in (
                    "exact_steel_candidate",
                    "other_metal_candidate",
                    "non_metal_candidate",
                )
            },
            "project_revisions": projects,
            "protected_only_floor": {
                "exact_steel_groups": protected_positive_floor,
                "non_metal_reject_groups": protected_reject_floor,
                "project_revisions": PROTECTED_PROJECT_FLOOR,
            },
            "raw_available_counts": {
                "broad_metal_candidates_diagnostic_only": broad_metal,
                "exact_steel_candidates": raw_counts["exact_steel_candidate"],
                "non_metal_candidates": raw_counts["non_metal_candidate"],
                "other_metal_candidates": raw_counts["other_metal_candidate"],
            },
            "remaining_after_protected_only_floor": {
                "exact_steel_candidates": remaining_exact,
                "non_metal_candidates": remaining_reject,
            },
        },
        "q0_report_sha256": sha256_bytes(q0_report_data),
        "required_roles": required_roles,
        "role_assignment": [],
        "role_policy": "whole_publisher_project_revision_atomic_or_no_assignment",
        "schema": AUDIT_SCHEMA,
        "target_policy": {
            "other_metal_policy": "not_positive_not_reject_unassigned_ood",
            "positive_material": "Steel",
            "positive_relation": "exact_steel_candidate",
            "policy_id": "exact_primary_material_steel_v1",
            "reject_relation": "non_metal_candidate",
        },
    }
    audit_bytes = canonical_json(audit)
    report = {
        "access": access,
        "audit_sha256": sha256_bytes(audit_bytes),
        "blockers": blockers,
        "decision": decision,
        "exposure_accounted_groups": len(joined),
        "input_identities": audit["input_identities"],
        "project_revision_count": len(projects),
        "protected_only_floor": audit["power"]["protected_only_floor"],
        "q0_decision": q0_report["decision"],
        "raw_available_counts": audit["power"]["raw_available_counts"],
        "role_assignment_count": 0,
        "schema": REPORT_SCHEMA,
        "status": (
            "COMPLETE / REPEATABLE_METADATA_ONLY_SOURCE_POWER_OOD / "
            "PROTECTED_PAYLOADS_SEALED"
        ),
        "target_policy_id": "exact_primary_material_steel_v1",
    }
    return {
        "metal-role-power-audit.json": audit_bytes,
        "report.json": canonical_json(report),
    }


def ensure_external_path(path: Path, context: str, require_absent: bool) -> Path:
    if path.is_symlink():
        raise Q1MRolePowerError(f"{context} cannot be a symlink")
    resolved_parent = path.parent.resolve(strict=True)
    resolved = resolved_parent / path.name
    repository = repository_root().resolve()
    if resolved == repository or repository in resolved.parents:
        raise Q1MRolePowerError(f"{context} must be outside the repository")
    if require_absent and (path.exists() or path.is_symlink()):
        raise Q1MRolePowerError(f"refusing to replace existing {context}: {path}")
    return resolved


def read_input(path: Path, context: str, expected: tuple[int, str]) -> bytes:
    ensure_external_path(path, context, False)
    if path.is_symlink() or not path.is_file():
        raise Q1MRolePowerError(f"{context} must be a regular non-symlink file")
    if path.stat().st_size > MAX_INPUT_BYTES:
        raise Q1MRolePowerError(f"{context} exceeds its byte bound")
    data = path.read_bytes()
    identity = (len(data), sha256_bytes(data))
    if identity != expected:
        raise Q1MRolePowerError(
            f"{context} identity changed: got bytes={identity[0]}, sha256={identity[1]}"
        )
    return data


def publish(path: Path, documents: dict[str, bytes]) -> Path:
    destination = ensure_external_path(path, "Q1-M output", True)
    temporary = Path(tempfile.mkdtemp(prefix=f".{path.name}.", dir=destination.parent))
    try:
        for name, data in documents.items():
            (temporary / name).write_bytes(data)
        os.replace(temporary, destination)
    except BaseException:
        shutil.rmtree(temporary, ignore_errors=True)
        raise
    return destination


def run_build(arguments: argparse.Namespace) -> Path:
    inputs = (
        read_input(arguments.q0_inventory, "Q0 inventory", Q0_INVENTORY_IDENTITY),
        read_input(arguments.q0_report, "Q0 report", Q0_REPORT_IDENTITY),
        read_input(arguments.exposure_census, "exposure census", EXPOSURE_IDENTITY),
        read_input(arguments.identity_map, "identity map", IDENTITY_MAP_IDENTITY),
        read_input(arguments.ycb_inventory, "YCB inventory", YCB_IDENTITY),
    )
    return publish(arguments.output, build_documents(*inputs))


def main() -> int:
    try:
        output = run_build(parse_arguments())
    except (OSError, Q1MRolePowerError) as error:
        print(f"physical-sound-v29-q1m-role-power: {error}", file=os.sys.stderr)
        return 1
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
