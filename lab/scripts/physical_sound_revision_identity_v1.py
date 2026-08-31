#!/usr/bin/env python3
"""Build the zero-signal Physical Sound V15-S0a identity/exposure census."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import tempfile
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterator

import physical_sound_exposure_ledger_v0 as ledger_v0
import physical_sound_source_inventory_v1 as inventory_v1

IDENTITY_SCHEMA = "nextengine.experimental-physical-sound-revision-identity-map.v1"
CENSUS_SCHEMA = "nextengine.experimental-physical-sound-revision-aware-exposure-census.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v15-s0a.report.v1"

N1B_INVENTORY_SHA256 = (
    "2861bb0e42e40fe78e67dab7ed5e0897b40c42caf6ab639235948adfa38fcfc0"
)
M1C_EXCLUSIONS_SHA256 = (
    "87e71fb217a217044ed1c049ef90eb663818d84e5fdd6e66514be87c89498520"
)
HISTORICAL_TABLE_SHA256 = inventory_v1.OBJECTFOLDER_HISTORICAL_SHA256
REALIMPACT_NAMES_SHA256 = inventory_v1.REALIMPACT_NAMES_SHA256
REALIMPACT_PAPER_TEXT_SHA256 = (
    "3d5c5ea23b9c65c9af7fb7b1d2e082f7507233c06681d9274285e6c4f6b0ad14"
)

REAL_INPUTS = {
    "source_inventory": (1_118_080, N1B_INVENTORY_SHA256),
    "historical_exclusions": (730_462, M1C_EXCLUSIONS_SHA256),
    "historical_table": (8_668, HISTORICAL_TABLE_SHA256),
    "realimpact_names": (738, REALIMPACT_NAMES_SHA256),
    "realimpact_paper_text": (106_070, REALIMPACT_PAPER_TEXT_SHA256),
}

CURRENT_SOURCE = "objectfolder_real_current"
HISTORICAL_SOURCE = "objectfolder_real_historical_d058"
REALIMPACT_SOURCE = "realimpact_fca2"
REAL_TIERS = {"T2_sparse_real_contact", "T3_dense_real_listener"}
CANDIDATE_STATES = {"metadata_candidate", "member_preflight_required"}
MATERIALS = inventory_v1.MATERIALS
MAX_JSON_FILES = 4096
MAX_JSON_BYTES = 16 * 1024 * 1024
EXCLUDED_COMPONENTS = {
    ".venv",
    "__pycache__",
    "dependencies",
    "site-packages",
    "venv",
}
EXCLUDED_TOP_LEVEL_PREFIXES = (
    "physical-sound-v13-m1",
    "physical-sound-v14-n1b",
    "physical-sound-v14-n1c-source-research",
    "physical-sound-v15-s0a",
    "ps2-source-feasibility-v1",
)
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


class RevisionIdentityError(RuntimeError):
    """The S0a identity/exposure boundary failed closed."""


@dataclass(frozen=True)
class Inputs:
    source_inventory: Path
    historical_exclusions: Path
    historical_table: Path
    realimpact_names: Path
    realimpact_paper_text: Path


def canonical_json(value: Any) -> bytes:
    return ledger_v0.canonical_json(value)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise RevisionIdentityError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise RevisionIdentityError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def read_bound_file(
    path: Path, context: str, expected: tuple[int, str] | None
) -> bytes:
    try:
        size, digest = inventory_v1.hash_regular_file(path, context)
    except inventory_v1.SourceInventoryError as error:
        raise RevisionIdentityError(str(error)) from error
    if expected is not None and (size, digest) != expected:
        raise RevisionIdentityError(
            f"{context} identity changed: got bytes={size}, sha256={digest}"
        )
    data = path.read_bytes()
    if len(data) != size:
        raise RevisionIdentityError(f"{context} changed while reading")
    return data


def parse_json(data: bytes, context: str, *, canonical: bool = False) -> Any:
    try:
        value = ledger_v0.parse_json_bytes(data, context)
    except ledger_v0.ExposureLedgerError as error:
        raise RevisionIdentityError(str(error)) from error
    if canonical and canonical_json(value) != data:
        raise RevisionIdentityError(f"{context} is not canonical JSON")
    return value


def validate_paper_claims(data: bytes) -> None:
    try:
        text = data.decode("utf-8").replace("\x00", " ").replace("\f", " ")
    except UnicodeDecodeError as error:
        raise RevisionIdentityError("RealImpact paper text is not UTF-8") from error
    claims = (
        "dataset of 150,000 real",
        "object impact sounds. Along with these sounds",
        "We purchase 50 objects from the ObjectFolder",
        "dataset [19], which is comprised",
        "Each object in R EAL I MPACT has a high-",
        "resolution 3D mesh model generated from a scan of the real",
        "rigid and consist of a single ho-",
        "mogeneous material belonging to one of the following cat-",
    )
    for claim in claims:
        if claim.lower() not in text.lower():
            raise RevisionIdentityError("RealImpact paper origin/material claim changed")


def current_rows(inventory: dict[str, Any]) -> dict[int, dict[str, Any]]:
    require_exact_keys(inventory, INVENTORY_KEYS, "N1b source inventory")
    if inventory.get("schema") != inventory_v1.INVENTORY_SCHEMA:
        raise RevisionIdentityError("unknown N1b source-inventory schema")
    raw_objects = inventory.get("objects")
    if not isinstance(raw_objects, list):
        raise RevisionIdentityError("source inventory objects must be an array")
    current: dict[int, dict[str, Any]] = {}
    realimpact: dict[str, dict[str, Any]] = {}
    for index, raw in enumerate(raw_objects):
        row = require_exact_keys(raw, INVENTORY_OBJECT_KEYS, f"inventory.objects[{index}]")
        source_id = row["source_id"]
        if source_id == CURRENT_SOURCE:
            object_id = row["publisher_object_id"]
            if not isinstance(object_id, str) or not object_id.isdigit():
                raise RevisionIdentityError("current ObjectFolder ID is malformed")
            numeric = int(object_id)
            if numeric in current:
                raise RevisionIdentityError("duplicate current ObjectFolder row")
            current[numeric] = row
        elif source_id == REALIMPACT_SOURCE:
            publisher_id = row["publisher_object_id"]
            if not isinstance(publisher_id, str) or publisher_id in realimpact:
                raise RevisionIdentityError("duplicate or malformed RealImpact row")
            realimpact[publisher_id] = row
    if set(current) != set(range(1, 101)):
        raise RevisionIdentityError("source inventory must contain current IDs 1..100")
    if len(realimpact) != 50:
        raise RevisionIdentityError("source inventory must contain 50 RealImpact rows")
    inventory["_s0a_current"] = current
    inventory["_s0a_realimpact"] = realimpact
    return current


def physical_group_id(basis: dict[str, Any]) -> str:
    return sha256_bytes(canonical_json(basis))


def inventory_candidate(row: dict[str, Any]) -> bool:
    return (
        row["source_tier"] in REAL_TIERS
        and row["candidate_state"] in CANDIDATE_STATES
        and row["material_family"] in MATERIALS
    )


def identity_map(
    source_inventory: dict[str, Any],
    historical: dict[int, tuple[str, str]],
    realimpact_names: dict[int, str],
    input_identities: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    current = current_rows(source_inventory)
    realimpact_rows: dict[str, dict[str, Any]] = source_inventory["_s0a_realimpact"]
    if set(realimpact_rows) != set(realimpact_names.values()):
        raise RevisionIdentityError("N1b and RealImpact object-name rosters disagree")

    groups: dict[str, dict[str, Any]] = {}
    locators: dict[tuple[str, str], str] = {}
    alias_edges: list[dict[str, Any]] = []
    discrepancies: list[dict[str, Any]] = []

    for object_id, (name, publisher_material) in sorted(historical.items()):
        material = inventory_v1.MATERIAL_MAP.get(publisher_material)
        basis = {
            "identity_revision": "objectfolder-real-physical-group-v1",
            "origin_revision": inventory_v1.SOURCE_REVISIONS[HISTORICAL_SOURCE],
            "publisher_material": publisher_material,
            "publisher_name": name,
            "publisher_object_id": str(object_id),
        }
        group_id = physical_group_id(basis)
        groups[group_id] = {
            "candidate_routes": [],
            "identity_basis": basis,
            "material_family": material,
            "members": [
                {
                    "alias_basis": "identity_origin",
                    "evidence_sha256s": [input_identities["historical_table"]["sha256"]],
                    "publisher_material": publisher_material,
                    "publisher_name": name,
                    "publisher_object_id": str(object_id),
                    "revision_id": inventory_v1.SOURCE_REVISIONS[HISTORICAL_SOURCE],
                    "source_id": HISTORICAL_SOURCE,
                    "source_tier": "T2_sparse_real_contact",
                }
            ],
            "numeric_object_ids": [str(object_id)],
            "physical_group_id": group_id,
        }
        locators[(HISTORICAL_SOURCE, str(object_id))] = group_id

    for object_id, row in sorted(current.items()):
        historical_name, historical_material = historical[object_id]
        same = (
            inventory_v1.normalize_name(row["publisher_name"])
            == inventory_v1.normalize_name(historical_name)
            and row["publisher_material"] == historical_material
        )
        if same:
            group_id = locators[(HISTORICAL_SOURCE, str(object_id))]
            alias_basis = "same_id_normalized_name_and_material"
            alias_edges.append(
                {
                    "basis": alias_basis,
                    "from_inventory_object_id": row["inventory_object_id"],
                    "physical_group_id": group_id,
                    "source_id": CURRENT_SOURCE,
                }
            )
        else:
            basis = {
                "identity_revision": "objectfolder-real-physical-group-v1",
                "origin_revision": inventory_v1.SOURCE_REVISIONS[CURRENT_SOURCE],
                "publisher_material": row["publisher_material"],
                "publisher_name": row["publisher_name"],
                "publisher_object_id": str(object_id),
            }
            group_id = physical_group_id(basis)
            if group_id in groups:
                raise RevisionIdentityError("current drift group collided with historical group")
            groups[group_id] = {
                "candidate_routes": [],
                "identity_basis": basis,
                "material_family": row["material_family"],
                "members": [],
                "numeric_object_ids": [str(object_id)],
                "physical_group_id": group_id,
            }
            alias_basis = "current_revision_distinct_identity"
            discrepancies.append(
                {
                    "code": "objectfolder_revision_identity_conflict",
                    "current_name": row["publisher_name"],
                    "historical_name": historical_name,
                    "numeric_object_id": str(object_id),
                }
            )
        member = {
            "alias_basis": alias_basis,
            "evidence_sha256s": [input_identities["source_inventory"]["sha256"]],
            "publisher_material": row["publisher_material"],
            "publisher_name": row["publisher_name"],
            "publisher_object_id": str(object_id),
            "revision_id": inventory_v1.SOURCE_REVISIONS[CURRENT_SOURCE],
            "source_id": CURRENT_SOURCE,
            "source_tier": row["source_tier"],
        }
        groups[group_id]["members"].append(member)
        if inventory_candidate(row):
            groups[group_id]["candidate_routes"].append(
                {
                    "candidate_state": row["candidate_state"],
                    "inventory_object_id": row["inventory_object_id"],
                    "source_id": CURRENT_SOURCE,
                    "source_tier": row["source_tier"],
                }
            )
        locators[(CURRENT_SOURCE, str(object_id))] = group_id

    for object_id, publisher_id in sorted(realimpact_names.items()):
        row = realimpact_rows[publisher_id]
        group_id = locators[(HISTORICAL_SOURCE, str(object_id))]
        historical_name, historical_material = historical[object_id]
        material = inventory_v1.MATERIAL_MAP.get(historical_material)
        member = {
            "alias_basis": "paper_origin_plus_frozen_numeric_filename_prefix",
            "evidence_sha256s": sorted(
                {
                    input_identities["historical_table"]["sha256"],
                    input_identities["realimpact_names"]["sha256"],
                    input_identities["realimpact_paper_text"]["sha256"],
                }
            ),
            "publisher_material": historical_material,
            "publisher_name": publisher_id,
            "publisher_object_id": publisher_id,
            "revision_id": inventory_v1.SOURCE_REVISIONS[REALIMPACT_SOURCE],
            "source_id": REALIMPACT_SOURCE,
            "source_tier": row["source_tier"],
        }
        groups[group_id]["members"].append(member)
        structural = row["structural_presence"]
        if not isinstance(structural, dict):
            raise RevisionIdentityError("RealImpact structural presence is malformed")
        if material in MATERIALS and structural.get("archive_identity_available") is True:
            groups[group_id]["candidate_routes"].append(
                {
                    "candidate_state": "member_preflight_required",
                    "inventory_object_id": row["inventory_object_id"],
                    "source_id": REALIMPACT_SOURCE,
                    "source_tier": row["source_tier"],
                }
            )
        alias_edges.append(
            {
                "basis": "paper_origin_plus_frozen_numeric_filename_prefix",
                "from_inventory_object_id": row["inventory_object_id"],
                "physical_group_id": group_id,
                "source_id": REALIMPACT_SOURCE,
            }
        )
        locators[(REALIMPACT_SOURCE, publisher_id)] = group_id

    for group in groups.values():
        group["members"].sort(
            key=lambda item: (item["source_id"], item["publisher_object_id"])
        )
        group["candidate_routes"].sort(
            key=lambda item: (item["source_id"], item["inventory_object_id"])
        )
        if len(group["members"]) != len(
            {(item["source_id"], item["publisher_object_id"]) for item in group["members"]}
        ):
            raise RevisionIdentityError("duplicate source member in physical group")

    document = {
        "alias_edges": sorted(
            alias_edges,
            key=lambda item: (
                item["physical_group_id"],
                item["source_id"],
                item["from_inventory_object_id"],
            ),
        ),
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "discrepancies": sorted(
            discrepancies, key=lambda item: int(item["numeric_object_id"])
        ),
        "groups": sorted(groups.values(), key=lambda item: item["physical_group_id"]),
        "input_identities": input_identities,
        "schema": IDENTITY_SCHEMA,
    }
    return document


def encode_pointer_token(value: str) -> str:
    return value.replace("~", "~0").replace("/", "~1")


def string_scalars(value: Any, pointer: str = "") -> Iterator[tuple[str, str]]:
    if isinstance(value, dict):
        for key in sorted(value):
            yield from string_scalars(
                value[key], f"{pointer}/{encode_pointer_token(key)}"
            )
    elif isinstance(value, list):
        for index, item in enumerate(value):
            yield from string_scalars(item, f"{pointer}/{index}")
    elif isinstance(value, str):
        yield pointer, value


def excluded_path(relative: Path) -> bool:
    if any(part in EXCLUDED_COMPONENTS for part in relative.parts):
        return True
    return bool(
        relative.parts
        and relative.parts[0].startswith(EXCLUDED_TOP_LEVEL_PREFIXES)
    )


def exact_name_pattern(names: list[str]) -> re.Pattern[str]:
    alternatives = "|".join(re.escape(name) for name in sorted(names, key=len, reverse=True))
    return re.compile(rf"(?<![A-Za-z0-9_])({alternatives})(?![A-Za-z0-9_])")


def scan_store(store: Path, names: list[str]) -> dict[str, Any]:
    if store.is_symlink():
        raise RevisionIdentityError("store root cannot be a symlink")
    root = store.resolve(strict=True)
    if not root.is_dir():
        raise RevisionIdentityError("store root must be a directory")
    repo = repository_root().resolve(strict=True)
    if root == repo or root.is_relative_to(repo):
        raise RevisionIdentityError("store root must remain outside the repository")

    pattern = exact_name_pattern(names)
    included: list[dict[str, Any]] = []
    evidence: list[dict[str, Any]] = []
    counters = {
        "json_bytes_read": 0,
        "json_files_parsed": 0,
        "json_scalar_values_parsed": 0,
    }
    paths = sorted(root.rglob("*.json"), key=lambda item: item.relative_to(root).as_posix())
    for path in paths:
        relative = path.relative_to(root)
        if excluded_path(relative):
            continue
        relative_name = relative.as_posix()
        if ledger_v0.has_symlink_component(root, relative_name):
            raise RevisionIdentityError(f"JSON path contains symlink: {relative_name}")
        if not path.is_file():
            raise RevisionIdentityError(f"JSON path is not a regular file: {relative_name}")
        before = path.stat()
        if before.st_size > MAX_JSON_BYTES:
            raise RevisionIdentityError(f"JSON artifact exceeds limit: {relative_name}")
        data = path.read_bytes()
        after = path.stat()
        if (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
        ) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise RevisionIdentityError(f"JSON artifact changed while read: {relative_name}")
        value = parse_json(data, f"historical JSON {relative_name}")
        artifact_id = f"artifact-{sha256_bytes(relative_name.encode())[:24]}"
        file_evidence = 0
        for pointer, string_value in string_scalars(value):
            matches = sorted(set(pattern.findall(string_value)))
            for publisher_id in matches:
                evidence.append(
                    {
                        "artifact_id": artifact_id,
                        "json_pointer": pointer,
                        "pointed_value_sha256": sha256_bytes(
                            canonical_json(string_value)
                        ),
                        "publisher_object_id": publisher_id,
                        "relative_path": relative_name,
                    }
                )
                file_evidence += 1
        included.append(
            {
                "artifact_id": artifact_id,
                "bytes": len(data),
                "exact_name_evidence_count": file_evidence,
                "relative_path": relative_name,
                "sha256": sha256_bytes(data),
            }
        )
        counters["json_bytes_read"] += len(data)
        counters["json_files_parsed"] += 1
        counters["json_scalar_values_parsed"] += ledger_v0.scalar_count(value)
        if counters["json_files_parsed"] > MAX_JSON_FILES:
            raise RevisionIdentityError("historical JSON census exceeds file limit")
    evidence.sort(
        key=lambda item: (
            item["publisher_object_id"],
            item["relative_path"],
            item["json_pointer"],
        )
    )
    return {
        "counters": counters,
        "evidence": evidence,
        "included": included,
        "root": root,
    }


def validate_exclusions(value: Any) -> dict[str, Any]:
    document = require_exact_keys(
        value,
        {"direct_exposures", "path_token_exposures", "revision", "schema"},
        "M1c exclusions",
    )
    if document["schema"] != "nextengine.experimental-physical-sound-object-exclusions.v0":
        raise RevisionIdentityError("unknown M1c exclusions schema")
    expected = {
        "direct_exposures": {"artifact_id", "json_pointer", "object_id", "relative_path"},
        "path_token_exposures": {
            "artifact_id",
            "json_pointer",
            "object_id",
            "relative_path",
            "token",
        },
    }
    for channel, keys in expected.items():
        rows = document[channel]
        if not isinstance(rows, list):
            raise RevisionIdentityError(f"{channel} must be an array")
        for index, row in enumerate(rows):
            record = require_exact_keys(row, keys, f"{channel}[{index}]")
            object_id = record["object_id"]
            if not isinstance(object_id, str) or not object_id.isdigit() or not 1 <= int(object_id) <= 100:
                raise RevisionIdentityError("M1c exposure object ID is malformed")
    return document


def exposure_census(
    identity: dict[str, Any],
    exclusions: dict[str, Any],
    store_scan: dict[str, Any],
    input_identities: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    direct_counts: dict[int, int] = defaultdict(int)
    token_counts: dict[int, int] = defaultdict(int)
    for row in exclusions["direct_exposures"]:
        direct_counts[int(row["object_id"])] += 1
    for row in exclusions["path_token_exposures"]:
        token_counts[int(row["object_id"])] += 1

    name_counts: dict[str, int] = defaultdict(int)
    for row in store_scan["evidence"]:
        name_counts[row["publisher_object_id"]] += 1

    group_rows = []
    candidate_counts = {
        material: {
            "all_metadata_candidate_groups": 0,
            "exposed_metadata_candidate_groups": 0,
            "unexposed_metadata_candidate_groups": 0,
        }
        for material in MATERIALS
    }
    for group in identity["groups"]:
        group_id = group["physical_group_id"]
        numeric_ids = [int(value) for value in group["numeric_object_ids"]]
        direct = sum(direct_counts[value] for value in numeric_ids)
        tokens = sum(token_counts[value] for value in numeric_ids)
        realimpact_members = [
            item["publisher_object_id"]
            for item in group["members"]
            if item["source_id"] == REALIMPACT_SOURCE
        ]
        names = sum(name_counts[value] for value in realimpact_members)
        exposed = direct > 0 or tokens > 0 or names > 0
        material = group["material_family"]
        candidate = bool(group["candidate_routes"]) and material in MATERIALS
        if candidate:
            counts = candidate_counts[material]
            counts["all_metadata_candidate_groups"] += 1
            counts[
                "exposed_metadata_candidate_groups"
                if exposed
                else "unexposed_metadata_candidate_groups"
            ] += 1
        group_rows.append(
            {
                "candidate_route_count": len(group["candidate_routes"]),
                "candidate_source_ids": sorted(
                    {item["source_id"] for item in group["candidate_routes"]}
                ),
                "exposure_evidence": {
                    "direct_numeric_record_count": direct,
                    "path_token_record_count": tokens,
                    "realimpact_name_record_count": names,
                },
                "exposure_state": "exposed" if exposed else "unexposed",
                "material_family": material,
                "physical_group_id": group_id,
            }
        )

    metal_potential = (
        candidate_counts["Metal"]["unexposed_metadata_candidate_groups"] >= 8
    )
    decision = (
        "S0A_IDENTITY_EXPOSURE_PASS_METAL_SCOPE_REMAINS_POTENTIAL"
        if metal_potential
        else "S0A_IDENTITY_EXPOSURE_PASS_METAL_SOURCE_INSUFFICIENT"
    )
    access = {
        **store_scan["counters"],
        "force_sample_values_decoded": 0,
        "network_archive_body_bytes": 0,
        "network_requests": 0,
        "npy_headers_parsed": 0,
        "numeric_exposure_records_read": len(exclusions["direct_exposures"])
        + len(exclusions["path_token_exposures"]),
        "pcm_sample_values_decoded": 0,
        "protected_signal_values_decoded": 0,
        "realimpact_name_evidence_records": len(store_scan["evidence"]),
        "source_payload_bytes_read": 0,
        "source_payload_members_extracted": 0,
        "wav_headers_parsed": 0,
    }
    return {
        "access": access,
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "candidate_counts": candidate_counts,
        "decision": decision,
        "excluded_path_policy": {
            "components": sorted(EXCLUDED_COMPONENTS),
            "top_level_prefixes": list(EXCLUDED_TOP_LEVEL_PREFIXES),
        },
        "groups": sorted(group_rows, key=lambda item: item["physical_group_id"]),
        "historical_json_manifest": store_scan["included"],
        "input_identities": input_identities,
        "metal_scope_remains_potential": metal_potential,
        "realimpact_name_evidence": store_scan["evidence"],
        "schema": CENSUS_SCHEMA,
    }


def build_documents(
    inputs: Inputs,
    store: Path,
    *,
    expected_identities: dict[str, tuple[int, str]] | None = REAL_INPUTS,
) -> dict[str, bytes]:
    raw: dict[str, bytes] = {}
    input_identities: dict[str, dict[str, Any]] = {}
    for key in Inputs.__dataclass_fields__:
        path = getattr(inputs, key)
        expected = None if expected_identities is None else expected_identities[key]
        data = read_bound_file(path, key, expected)
        raw[key] = data
        input_identities[key] = {"bytes": len(data), "sha256": sha256_bytes(data)}

    source_inventory = parse_json(raw["source_inventory"], "N1b inventory", canonical=True)
    exclusions = validate_exclusions(
        parse_json(raw["historical_exclusions"], "M1c exclusions", canonical=True)
    )
    try:
        historical = inventory_v1.parse_historical_table(raw["historical_table"])
        realimpact_names = inventory_v1.parse_realimpact_names(raw["realimpact_names"])
    except inventory_v1.SourceInventoryError as error:
        raise RevisionIdentityError(str(error)) from error
    validate_paper_claims(raw["realimpact_paper_text"])

    identity = identity_map(
        source_inventory,
        historical,
        realimpact_names,
        input_identities,
    )
    scan = scan_store(store, sorted(realimpact_names.values()))
    census = exposure_census(
        identity,
        exclusions,
        scan,
        input_identities,
    )
    identity_bytes = canonical_json(identity)
    census_bytes = canonical_json(census)
    report = {
        "access": census["access"],
        "candidate_counts": census["candidate_counts"],
        "counts": {
            "alias_edge_count": len(identity["alias_edges"]),
            "historical_json_file_count": len(census["historical_json_manifest"]),
            "physical_group_count": len(identity["groups"]),
            "realimpact_name_evidence_count": len(census["realimpact_name_evidence"]),
            "revision_discrepancy_count": len(identity["discrepancies"]),
        },
        "decision": census["decision"],
        "exposure_census_sha256": sha256_bytes(census_bytes),
        "identity_map_sha256": sha256_bytes(identity_bytes),
        "opens_only": "V15_S0b_YCB_metadata_capability_adapter",
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "schema": REPORT_SCHEMA,
    }
    return {
        "exposure-census.json": census_bytes,
        "identity-map.json": identity_bytes,
        "report.json": canonical_json(report),
    }


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def prepare_output(output: Path) -> Path:
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise RevisionIdentityError("output must remain outside the repository")
    if resolved.exists():
        raise RevisionIdentityError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish(output: Path, documents: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for name, data in sorted(documents.items()):
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def execute(
    inputs: Inputs,
    store: Path,
    output: Path,
    *,
    expected_identities: dict[str, tuple[int, str]] | None = REAL_INPUTS,
) -> Path:
    return publish(
        output,
        build_documents(inputs, store, expected_identities=expected_identities),
    )


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    for name in Inputs.__dataclass_fields__:
        parser.add_argument(f"--{name.replace('_', '-')}", type=Path, required=True)
    parser.add_argument("--store", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    inputs = Inputs(
        **{
            name: getattr(arguments, name)
            for name in Inputs.__dataclass_fields__
        }
    )
    print(execute(inputs, arguments.store, arguments.output))


if __name__ == "__main__":
    main()
