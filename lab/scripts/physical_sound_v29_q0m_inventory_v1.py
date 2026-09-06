#!/usr/bin/env python3
"""Build the V29 Q0-M signal-blind Steel/Metal source inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from collections import Counter, defaultdict
from html.parser import HTMLParser
from pathlib import Path
from typing import Any


INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-v29-q0m-inventory.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v29-q0m.report.v1"
YCB_SCHEMA = "nextengine.experimental-physical-sound-ycb-capability.v1"
LEGACY_SCHEMA = "nextengine.experimental-physical-sound-identified-corpus.manifest.v1"

OBJECTFOLDER_IDENTITY = (
    34_749,
    "0111f57a336fdeb805eb03543fb10b2cb174f8d0bcb7ef5f0e30c17324e06db0",
)
YCB_IDENTITY = (
    227_019,
    "7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf",
)
LEGACY_IDENTITY = (
    9_112,
    "9ddec04de271c034f1389f5fcec98ce410028b307ae565b3f890eb7f7a726c33",
)

OBJECTFOLDER_MATERIALS = {
    "Ceramic",
    "Glass",
    "Wood",
    "Plastic",
    "Iron",
    "Polycarbonate",
    "Steel",
}
YCB_MATERIALS = {
    "Aluminium",
    "Ceramic",
    "Felt",
    "Fiber",
    "Foam",
    "Glass",
    "Hard Plastic",
    "Leather",
    "Other Plastic",
    "Paper",
    "Rubber",
    "Soft Plastic",
    "Steel",
    "Wood",
}
AXIS_STATES = {"known", "partial", "unknown"}
PARTITIONS = {"dev", "calibration", "holdout", "shadow"}
ROLES = {"target", "reject_parent"}
MIN_EXACT_STEEL = 16
MIN_BROAD_METAL = 16
MIN_NON_METAL = 35
MIN_PROJECTS = 2
MAX_INPUT_BYTES = 1024 * 1024

YCB_KEYS = {"acquisition_facts", "objects", "project", "schema"}
YCB_PROJECT_KEYS = {"component_revision", "id", "publisher"}
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
YCB_AXES = {
    "canonical_excitation",
    "contact_geometry_binding",
    "geometry",
    "geometry_scale",
    "listener_condition",
    "material_identity",
    "object_identity",
    "recorded_response",
    "support_condition",
}
YCB_GEOMETRY_KEYS = {
    "exact_payload_identity",
    "metric_scale_known",
    "route_count",
    "state",
    "variants",
}
YCB_VARIANT_KEYS = {"publisher_name", "routes", "variant"}
YCB_ROUTE_KEYS = {"kind", "state", "url"}
YCB_PARENT_KEYS = {
    "acquisition_mode",
    "conditions",
    "cost",
    "group_id",
    "materialized_path",
    "parent_id",
}
YCB_COST_KEYS = {"declared_bytes", "file_count", "unknown_size_file_count"}
LEGACY_KEYS = {
    "assignments",
    "corpus_id",
    "corpus_plan_report",
    "internet_source_manifest",
    "revision",
    "schema",
    "target_material_label",
}
LEGACY_REF_KEYS = {"path", "sha256"}
LEGACY_ASSIGNMENT_KEYS = {"corpus_role", "partition", "source_id"}

ACCESS = {
    "archive_member_bodies_read": 0,
    "audio_headers_parsed": 0,
    "force_sample_values_decoded": 0,
    "mesh_values_decoded": 0,
    "network_requests": 0,
    "pcm_sample_values_decoded": 0,
    "protected_signal_values_decoded": 0,
    "source_payload_bytes_read": 0,
}


class Q0MInventoryError(RuntimeError):
    """A Q0-M input or result violates the frozen metadata-only protocol."""


class TableCells(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.cells: list[str] = []
        self._parts: list[str] | None = None

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        del attrs
        if tag == "td":
            self._parts = []

    def handle_data(self, data: str) -> None:
        if self._parts is not None:
            self._parts.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "td" and self._parts is not None:
            self.cells.append("".join(self._parts).strip())
            self._parts = None


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--objectfolder-table", required=True, type=Path)
    parser.add_argument("--ycb-inventory", required=True, type=Path)
    parser.add_argument("--legacy-glass-manifest", required=True, type=Path)
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
        raise Q0MInventoryError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require_keys(value: Any, keys: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != keys:
        raise Q0MInventoryError(f"{context} has unknown or missing fields")
    return value


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise Q0MInventoryError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Q0MInventoryError(f"{context} is not strict JSON: {error}") from error
    if not isinstance(value, dict):
        raise Q0MInventoryError(f"{context} root must be an object")
    return value


def validate_label(value: Any, context: str, maximum: int = 128) -> str:
    if (
        not isinstance(value, str)
        or not 1 <= len(value) <= maximum
        or not value.isascii()
        or any(character.isspace() and character != " " for character in value)
    ):
        raise Q0MInventoryError(f"{context} must be bounded ASCII")
    return value


def parse_objectfolder(data: bytes) -> list[tuple[int, str, str]]:
    parser = TableCells()
    try:
        parser.feed(data.decode("utf-8"))
    except UnicodeDecodeError as error:
        raise Q0MInventoryError("ObjectFolder table is not UTF-8") from error
    rows: dict[int, tuple[str, str]] = {}
    for index in range(0, len(parser.cells), 3):
        cells = parser.cells[index : index + 3]
        if len(cells) != 3 or not cells[0].isdigit():
            continue
        object_id = int(cells[0])
        if object_id in rows:
            raise Q0MInventoryError("duplicate ObjectFolder object ID")
        name = validate_label(cells[1], f"ObjectFolder name {object_id}")
        material = validate_label(cells[2], f"ObjectFolder material {object_id}")
        if material not in OBJECTFOLDER_MATERIALS:
            raise Q0MInventoryError(f"unknown ObjectFolder material {material}")
        rows[object_id] = (name, material)
    if set(rows) != set(range(1, 101)):
        raise Q0MInventoryError("ObjectFolder table must contain exact IDs 1..100")
    return [(object_id, *rows[object_id]) for object_id in sorted(rows)]


def validate_ycb(data: bytes) -> list[dict[str, Any]]:
    document = require_keys(parse_json(data, "YCB inventory"), YCB_KEYS, "YCB inventory")
    if document["schema"] != YCB_SCHEMA:
        raise Q0MInventoryError("wrong YCB inventory schema")
    project = require_keys(document["project"], YCB_PROJECT_KEYS, "YCB project")
    if project != {
        "component_revision": "2022-09-27T11:36:47.167853",
        "id": "ycb-impact-sounds",
        "publisher": "iri-csic-upc-ctu",
    }:
        raise Q0MInventoryError("YCB project identity changed")
    if not isinstance(document["acquisition_facts"], dict):
        raise Q0MInventoryError("YCB acquisition facts must be an object")
    objects = document["objects"]
    if not isinstance(objects, list) or len(objects) != 77:
        raise Q0MInventoryError("YCB inventory must contain 77 objects")
    result: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    parent_ids: set[str] = set()
    for raw in objects:
        row = require_keys(raw, YCB_OBJECT_KEYS, "YCB object")
        object_id_text = validate_label(row["object_id"], "YCB object ID", 4)
        if not re.fullmatch(r"(?:[1-9][0-9]?|x_0[12])", object_id_text):
            raise Q0MInventoryError("YCB object ID is outside the frozen workbook identity")
        if object_id_text in seen_ids:
            raise Q0MInventoryError("duplicate YCB object ID")
        seen_ids.add(object_id_text)
        validate_label(row["object_name"], f"YCB object name {object_id_text}")
        material = validate_label(row["primary_material"], "YCB primary material")
        if material not in YCB_MATERIALS:
            raise Q0MInventoryError(f"unknown YCB primary material {material}")
        secondary = row["secondary_material"]
        if secondary is not None:
            validate_label(secondary, "YCB secondary material")
        axes = require_keys(row["capability_axes"], YCB_AXES, "YCB capability axes")
        if any(state not in AXIS_STATES for state in axes.values()):
            raise Q0MInventoryError("YCB capability axis has an unknown state")
        geometry = require_keys(row["geometry"], YCB_GEOMETRY_KEYS, "YCB geometry")
        if geometry["state"] not in {
            "route_absent",
            "route_available",
            "variant_ambiguous",
        }:
            raise Q0MInventoryError("YCB geometry has an unknown state")
        if not isinstance(geometry["variants"], list):
            raise Q0MInventoryError("YCB geometry variants must be a list")
        for variant in geometry["variants"]:
            variant = require_keys(variant, YCB_VARIANT_KEYS, "YCB geometry variant")
            if not isinstance(variant["routes"], list):
                raise Q0MInventoryError("YCB geometry routes must be a list")
            for route in variant["routes"]:
                require_keys(route, YCB_ROUTE_KEYS, "YCB geometry route")
        if row["training_usable"] is not False or row["evaluation_complete"] is not False:
            raise Q0MInventoryError("metadata-only YCB rows cannot be role usable")
        for field in ("missing_training_axes", "missing_evaluation_axes"):
            if not isinstance(row[field], list) or any(axis not in YCB_AXES for axis in row[field]):
                raise Q0MInventoryError(f"YCB {field} is invalid")
        parents = row["recording_parents"]
        if not isinstance(parents, list) or len(parents) > 1:
            raise Q0MInventoryError("YCB rows may have at most one exact parent")
        for parent in parents:
            parent = require_keys(parent, YCB_PARENT_KEYS, "YCB parent")
            if parent["acquisition_mode"] not in {"vertical-known", "vertical-unknown"}:
                raise Q0MInventoryError("aggregate YCB parent is ineligible")
            require_keys(parent["cost"], YCB_COST_KEYS, "YCB parent cost")
            parent_id = validate_label(parent["parent_id"], "YCB parent ID", 64)
            group_id = validate_label(parent["group_id"], "YCB group ID", 256)
            if parent_id in parent_ids or group_id in parent_ids:
                raise Q0MInventoryError("duplicate YCB recording parent")
            parent_ids.update((parent_id, group_id))
        result.append(row)
    expected_ids = {str(object_id) for object_id in range(1, 76)} | {"x_01", "x_02"}
    if seen_ids != expected_ids or sum(len(row["recording_parents"]) for row in result) != 39:
        raise Q0MInventoryError("YCB inventory must contain its 77 frozen workbook IDs and 39 exact parents")
    return result


def validate_legacy(data: bytes) -> tuple[dict[int, dict[str, str]], dict[int, dict[str, str]]]:
    document = require_keys(parse_json(data, "legacy Glass manifest"), LEGACY_KEYS, "legacy Glass manifest")
    if (
        document["schema"] != LEGACY_SCHEMA
        or document["corpus_id"]
        != "av-msf-ycb-heller-freesound-objectfolder-ycb-vertical-kronland-soundpacks-explicit-e3"
        or document["revision"] != "v3-project-disjoint-split-frozen"
        or document["target_material_label"] != "Glass"
    ):
        raise Q0MInventoryError("legacy Glass manifest identity changed")
    for field in ("internet_source_manifest", "corpus_plan_report"):
        require_keys(document[field], LEGACY_REF_KEYS, f"legacy {field}")
    assignments = document["assignments"]
    if not isinstance(assignments, list) or len(assignments) != 64:
        raise Q0MInventoryError("legacy Glass manifest must contain 64 assignments")
    objectfolder: dict[int, dict[str, str]] = {}
    ycb: dict[int, dict[str, str]] = {}
    seen: set[str] = set()
    for raw in assignments:
        assignment = require_keys(raw, LEGACY_ASSIGNMENT_KEYS, "legacy assignment")
        source_id = validate_label(assignment["source_id"], "legacy source ID", 256)
        if source_id in seen:
            raise Q0MInventoryError("duplicate legacy source assignment")
        seen.add(source_id)
        if assignment["partition"] not in PARTITIONS or assignment["corpus_role"] not in ROLES:
            raise Q0MInventoryError("legacy assignment has unknown partition or role")
        evidence = {
            "historical_partition": assignment["partition"],
            "historical_role": assignment["corpus_role"],
            "historical_source_id": source_id,
        }
        match = re.fullmatch(r"objectfolder-real-demo-([1-9][0-9]{0,2})", source_id)
        if match:
            object_id = int(match.group(1))
            if object_id in objectfolder:
                raise Q0MInventoryError("duplicate ObjectFolder historical exclusion")
            objectfolder[object_id] = evidence
        match = re.fullmatch(r"ycb-impact-vertical-object-([0-9]{3})-[a-z0-9-]+", source_id)
        if match:
            object_id = int(match.group(1))
            if object_id in ycb:
                raise Q0MInventoryError("duplicate YCB historical exclusion")
            ycb[object_id] = evidence
    return objectfolder, ycb


def relation(material: str) -> str:
    if material == "Steel":
        return "exact_steel_candidate"
    if material in {"Iron", "Aluminium"}:
        return "other_metal_candidate"
    return "non_metal_candidate"


def exclusion(value: dict[str, str] | None) -> dict[str, str] | None:
    return None if value is None else {**value, "reason": "historical_glass_split_identity"}


def project_id(group: dict[str, Any]) -> str:
    return "--".join((group["publisher_id"], group["project_id"], group["revision"]))


def build_documents(
    objectfolder_data: bytes, ycb_data: bytes, legacy_data: bytes
) -> dict[str, bytes]:
    objectfolder_rows = parse_objectfolder(objectfolder_data)
    ycb_rows = validate_ycb(ycb_data)
    objectfolder_exclusions, ycb_exclusions = validate_legacy(legacy_data)

    groups: list[dict[str, Any]] = []
    for object_id, name, material in objectfolder_rows:
        excluded = exclusion(objectfolder_exclusions.get(object_id))
        groups.append(
            {
                "acquisition_axes": {
                    "contact_position": "declared_opaque_payload",
                    "force_profile": "declared_opaque_payload",
                    "geometry": "declared_opaque_payload",
                    "listener_condition": "missing",
                    "material_identity": "known",
                    "object_identity": "known",
                    "recorded_response": "declared_opaque_payload",
                    "support_condition": "missing",
                },
                "candidate_available_after_historical_exclusion": excluded is None,
                "group_id": f"stanford-objectfolder--objectfolder-real--rendered-table-sha256-0111f57a--object-{object_id}",
                "historical_exclusion": excluded,
                "material_relation": relation(material),
                "object_id": str(object_id),
                "object_name": name,
                "primary_material": material,
                "project_id": "objectfolder-real",
                "publisher_id": "stanford-objectfolder",
                "recording_parent_id": None,
                "revision": "rendered-table-sha256-0111f57a",
                "role_state": "unassigned",
                "secondary_material": None,
                "source_id": f"objectfolder-real-{object_id}",
            }
        )
    for row in ycb_rows:
        if not row["recording_parents"]:
            continue
        parent = row["recording_parents"][0]
        object_id = int(row["object_id"])
        material = row["primary_material"]
        excluded = exclusion(ycb_exclusions.get(object_id))
        groups.append(
            {
                "acquisition_axes": row["capability_axes"],
                "candidate_available_after_historical_exclusion": excluded is None,
                "group_id": parent["group_id"],
                "historical_exclusion": excluded,
                "material_relation": relation(material),
                "missing_evaluation_axes": row["missing_evaluation_axes"],
                "missing_training_axes": row["missing_training_axes"],
                "object_id": row["object_id"],
                "object_name": row["object_name"],
                "primary_material": material,
                "project_id": "ycb-impact-sounds",
                "publisher_id": "iri-csic-upc-ctu",
                "recording_parent_id": parent["parent_id"],
                "revision": "osf-bj5w8-2022-09-27",
                "role_state": "unassigned",
                "secondary_material": row["secondary_material"],
                "source_id": f"ycb-impact-object-{object_id:03}-parent-{parent['parent_id']}",
            }
        )
    groups.sort(key=lambda group: group["group_id"])
    group_ids = [group["group_id"] for group in groups]
    if len(group_ids) != len(set(group_ids)):
        raise Q0MInventoryError("duplicate physical candidate group")

    sources = [
        {
            "landing_page_url": "https://objectfolder.stanford.edu/objectfolder-real-download",
            "license_expression": "NOASSERTION",
            "project_id": "objectfolder-real",
            "publisher_id": "stanford-objectfolder",
            "redistribution_policy": "external_research_only",
            "revision": "rendered-table-sha256-0111f57a",
        },
        {
            "landing_page_url": "https://osf.io/4tcp6/",
            "license_expression": "NOASSERTION",
            "project_id": "ycb-impact-sounds",
            "publisher_id": "iri-csic-upc-ctu",
            "redistribution_policy": "external_research_only",
            "revision": "osf-bj5w8-2022-09-27",
        },
    ]
    inventory = {
        "access": {
            **ACCESS,
            "metadata_bytes_read": len(objectfolder_data) + len(ycb_data) + len(legacy_data),
        },
        "authority": "SOURCE_IDENTITY_AND_DECLARED_AXIS_INVENTORY_ONLY / NO_ROLE_OR_ADMISSION_AUTHORITY",
        "groups": groups,
        "historical_glass_split": {
            "manifest_sha256": sha256_bytes(legacy_data),
            "matched_objectfolder_groups": len(objectfolder_exclusions),
            "matched_ycb_groups": len(ycb_exclusions),
            "policy": "exclude_without_relabelling_or_threshold_use",
            "target_material_label": "Glass",
        },
        "schema": INVENTORY_SCHEMA,
        "sources": sources,
    }
    inventory_bytes = canonical_json(inventory)

    available = [group for group in groups if group["candidate_available_after_historical_exclusion"]]
    relation_counts = Counter(group["material_relation"] for group in available)
    broad_metal = relation_counts["exact_steel_candidate"] + relation_counts["other_metal_candidate"]
    projects: dict[str, set[str]] = defaultdict(set)
    materials = Counter()
    for group in available:
        projects[group["material_relation"]].add(project_id(group))
        materials[group["primary_material"]] += 1
    broad_projects = projects["exact_steel_candidate"] | projects["other_metal_candidate"]
    feasible = (
        relation_counts["exact_steel_candidate"] >= MIN_EXACT_STEEL
        and broad_metal >= MIN_BROAD_METAL
        and relation_counts["non_metal_candidate"] >= MIN_NON_METAL
        and len(projects["exact_steel_candidate"]) >= MIN_PROJECTS
        and len(broad_projects) >= MIN_PROJECTS
        and len(projects["non_metal_candidate"]) >= MIN_PROJECTS
    )
    decision = (
        "Q0M_METAL_SOURCE_INVENTORY_FEASIBLE_Q1M_NEXT"
        if feasible
        else "Q0M_SOURCE_IDENTITY_INSUFFICIENT_SOURCE_GROWTH_REQUIRED"
    )
    report = {
        "access": inventory["access"],
        "available_counts": {
            "broad_metal_candidates": broad_metal,
            "exact_steel_candidates": relation_counts["exact_steel_candidate"],
            "non_metal_candidates": relation_counts["non_metal_candidate"],
            "other_metal_candidates": relation_counts["other_metal_candidate"],
            "primary_materials": dict(sorted(materials.items())),
        },
        "decision": decision,
        "excluded_group_count": len(groups) - len(available),
        "group_count": len(groups),
        "input_identities": {
            "legacy_glass_manifest": {
                "bytes": len(legacy_data),
                "sha256": sha256_bytes(legacy_data),
            },
            "objectfolder_table": {
                "bytes": len(objectfolder_data),
                "sha256": sha256_bytes(objectfolder_data),
            },
            "ycb_inventory": {
                "bytes": len(ycb_data),
                "sha256": sha256_bytes(ycb_data),
            },
        },
        "inventory_sha256": sha256_bytes(inventory_bytes),
        "minimums": {
            "broad_metal_candidates": MIN_BROAD_METAL,
            "exact_steel_candidates": MIN_EXACT_STEEL,
            "non_metal_candidates": MIN_NON_METAL,
            "project_revisions_per_required_class": MIN_PROJECTS,
        },
        "project_revision_counts": {
            "broad_metal_candidates": len(broad_projects),
            "exact_steel_candidates": len(projects["exact_steel_candidate"]),
            "non_metal_candidates": len(projects["non_metal_candidate"]),
        },
        "q1_blockers": [
            "effective_cluster_power_not_frozen",
            "exposure_freshness_not_reaudited",
            "one_use_roles_unassigned",
            "protected_payloads_sealed",
            "target_submaterial_policy_unfrozen",
        ],
        "schema": REPORT_SCHEMA,
    }
    return {
        "metal-source-inventory.json": inventory_bytes,
        "report.json": canonical_json(report),
    }


def ensure_external_path(path: Path, context: str, require_absent: bool) -> Path:
    if path.is_symlink():
        raise Q0MInventoryError(f"{context} cannot be a symlink")
    resolved_parent = path.parent.resolve(strict=True)
    resolved = resolved_parent / path.name
    repository = repository_root().resolve()
    if resolved == repository or repository in resolved.parents:
        raise Q0MInventoryError(f"{context} must be outside the repository")
    if require_absent and (path.exists() or path.is_symlink()):
        raise Q0MInventoryError(f"refusing to replace existing {context}: {path}")
    return resolved


def read_input(path: Path, context: str, expected: tuple[int, str]) -> bytes:
    ensure_external_path(path, context, False)
    if path.is_symlink() or not path.is_file():
        raise Q0MInventoryError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size > MAX_INPUT_BYTES:
        raise Q0MInventoryError(f"{context} exceeds its byte bound")
    data = path.read_bytes()
    identity = (len(data), sha256_bytes(data))
    if identity != expected:
        raise Q0MInventoryError(
            f"{context} identity changed: got bytes={identity[0]}, sha256={identity[1]}"
        )
    return data


def publish(path: Path, documents: dict[str, bytes]) -> Path:
    destination = ensure_external_path(path, "Q0-M output", True)
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
    objectfolder = read_input(
        arguments.objectfolder_table, "ObjectFolder table", OBJECTFOLDER_IDENTITY
    )
    ycb = read_input(arguments.ycb_inventory, "YCB inventory", YCB_IDENTITY)
    legacy = read_input(
        arguments.legacy_glass_manifest, "legacy Glass manifest", LEGACY_IDENTITY
    )
    return publish(arguments.output, build_documents(objectfolder, ycb, legacy))


def main() -> int:
    try:
        result = run_build(parse_arguments())
    except Q0MInventoryError as error:
        raise SystemExit(f"physical-sound Q0-M inventory error: {error}") from error
    print(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
