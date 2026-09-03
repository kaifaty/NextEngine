#!/usr/bin/env python3
"""Publish a corrected disclosed corpus from the frozen V43 C1A repair plan."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import re
import shutil
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v43-d2-c0r-profile.v1"
D1_SCHEMA = "nextengine.experimental-physical-sound-v41-d1-disclosed-roster.v1"
D2_SCHEMA = "nextengine.experimental-physical-sound-v43-d2-disclosed-roster.v1"
C0_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-corpus.v1"
C0_PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-v41-c0-role-projection.v1"
)
C0R_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-corpus.v1"
C0R_PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-v43-c0r-role-projection.v1"
)
C0R_CARD_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-corpus-card.v1"
REPAIR_SCHEMA = "nextengine.experimental-physical-sound-v43-d2-repair-record.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-report.v1"
C1A_REPORT_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-report.v1"
C1A_FINDINGS_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-findings.v1"
C1A_PLAN_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-repair-plan.v1"

PROFILE_ID = "physical-sound-v43-d2-c0r-identity-repair-v1"
DECISION = "C0R_CORRECTED_CORPUS_REPEATABLE_B0R_R0R_C1_AUTHORIZED"
SOURCE_COMPONENT = "source-component.realimpact-objectfolder-av-msf.v1"
TRAIN_ROLE = "generator_train"
DEVELOPMENT_ROLE = "generator_development"
VALIDATOR_ROLE = "validator_calibration"
ROLES = (DEVELOPMENT_ROLE, TRAIN_ROLE, VALIDATOR_ROLE)
AV_FAMILY = "zisen-shao--av-msf"
OF_FAMILY = "stanford-objectfolder--objectfolder-real"
RI_FAMILY = "samuel-clarke--realimpact"
SOURCE_FAMILIES = (RI_FAMILY, OF_FAMILY, AV_FAMILY)
EXPECTED_SOURCE_ROLES = {
    DEVELOPMENT_ROLE: 70,
    TRAIN_ROLE: 44,
    VALIDATOR_ROLE: 25,
}
CORRECTED_ROLES = {
    DEVELOPMENT_ROLE: 70,
    TRAIN_ROLE: 64,
    VALIDATOR_ROLE: 5,
}
CORRECTED_PARENTS = {
    DEVELOPMENT_ROLE: 30,
    TRAIN_ROLE: 34,
    VALIDATOR_ROLE: 1,
}
MATERIAL_QUARANTINE_PARENT = "objectfolder-real-object-80--av-msf-object-80"
UNKNOWN_MATERIAL = "Unknown"
C0_FILES = {
    "access_ledger": "access-ledger.json",
    "corpus_card": "corpus-card.json",
    "generator_development": "projections/generator_development.json",
    "generator_train": "projections/generator_train.json",
    "manifest": "manifest.json",
    "profile": "profile.json",
    "report": "report.json",
    "validator_calibration": "projections/validator_calibration.json",
}
C1A_FILES = {
    "access_ledger": "access-ledger.json",
    "alias_findings": "alias-findings.json",
    "evidence_inventory": "evidence-inventory.json",
    "profile": "profile.json",
    "repair_plan": "repair-plan.json",
    "report": "report.json",
}
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": [
        "v43_b0r_r0r_corrected_baseline",
        "v43_c1_descriptor_source_growth",
    ],
    "candidate_training_authority": False,
    "corpus_repair_authority": True,
    "descriptor_baseline_authority": False,
    "model_training_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "source_payload_decode_authority": False,
    "validator_calibration_authority": False,
}


class IdentityRepairError(RuntimeError):
    """The frozen D2/C0R correction cannot be published safely."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"
    ).encode("utf-8")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def binding(data: bytes) -> dict[str, Any]:
    return {"bytes": len(data), "sha256": sha256_bytes(data)}


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise IdentityRepairError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise IdentityRepairError(f"{context} must be an array")
    return value


def validate_binding(value: Any, context: str) -> dict[str, Any]:
    value = require_dict(value, context)
    if set(value) != {"bytes", "sha256"}:
        raise IdentityRepairError(f"{context} binding fields changed")
    size = value["bytes"]
    digest = value["sha256"]
    if not isinstance(size, int) or size <= 0:
        raise IdentityRepairError(f"{context} byte count is invalid")
    if not isinstance(digest, str) or re.fullmatch(r"[0-9a-f]{64}", digest) is None:
        raise IdentityRepairError(f"{context} SHA-256 is invalid")
    return value


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def read_regular(path: Path, context: str, maximum: int = 64 * 1024 * 1024) -> bytes:
    if has_symlink_component(path) or not path.is_file():
        raise IdentityRepairError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise IdentityRepairError(f"{context} size is outside the frozen bound")
    data = path.read_bytes()
    if len(data) != size:
        raise IdentityRepairError(f"{context} changed while reading")
    return data


def parse_canonical_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise IdentityRepairError(f"{context} is not valid JSON") from error
    value = require_dict(value, context)
    if canonical_json(value) != data:
        raise IdentityRepairError(f"{context} must use canonical JSON")
    return value


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, "D2/C0R profile", 512 * 1024)
    return data, parse_canonical_json(data, "D2/C0R profile")


def load_bound_json(
    path: Path, expected: Any, context: str
) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context, 2 * 1024 * 1024)
    expected = validate_binding(expected, f"{context} expected")
    if binding(data) != expected:
        raise IdentityRepairError(f"{context} binding changed")
    return data, parse_canonical_json(data, context)


def validate_profile(profile: dict[str, Any]) -> dict[str, Any]:
    expected_fields = {
        "authority",
        "dependency_bindings",
        "expected_counts",
        "input_bindings",
        "profile_id",
        "repair_policy",
        "revision",
        "schema",
    }
    if set(profile) != expected_fields:
        raise IdentityRepairError("D2/C0R profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["profile_id"] != PROFILE_ID:
        raise IdentityRepairError("unknown D2/C0R profile identity")
    if profile["revision"] != 1 or profile["authority"] != AUTHORITY:
        raise IdentityRepairError("D2/C0R revision or authority changed")

    required_paths = {
        "docs/architecture/45-physical-sound-synthesis-and-acoustic-presentation.md",
        "docs/development/physical-sound-v41-c0-disclosed-corpus-result-2026-09-03.md",
        "docs/development/physical-sound-v43-c1a-lineage-audit-result-2026-09-03.md",
        "lab/profiles/physical-sound-v43-c1a-lineage-audit.v1.json",
        "lab/scripts/physical_sound_v43_c1a_lineage_audit_v1.py",
        "lab/scripts/physical_sound_v43_d2_c0r_identity_repair_v1.py",
    }
    dependencies = require_list(profile["dependency_bindings"], "dependencies")
    seen: set[str] = set()
    root = repository_root()
    for raw in dependencies:
        row = require_dict(raw, "dependency binding")
        if set(row) != {"bytes", "path", "sha256"}:
            raise IdentityRepairError("dependency binding fields changed")
        path = row["path"]
        if not isinstance(path, str) or path in seen or path not in required_paths:
            raise IdentityRepairError("dependency binding path changed")
        seen.add(path)
        data = read_regular(root / path, f"dependency {path}", 2 * 1024 * 1024)
        if binding(data) != {"bytes": row["bytes"], "sha256": row["sha256"]}:
            raise IdentityRepairError(f"dependency binding changed: {path}")
    if seen != required_paths:
        raise IdentityRepairError("dependency binding set changed")

    inputs = require_dict(profile["input_bindings"], "input bindings")
    if set(inputs) != {"c0", "c1a", "d1_roster"}:
        raise IdentityRepairError("input binding groups changed")
    for group, names in (("c0", C0_FILES), ("c1a", C1A_FILES)):
        bindings = require_dict(inputs[group], f"{group} bindings")
        if set(bindings) != set(names):
            raise IdentityRepairError(f"{group} binding set changed")
        for name, value in bindings.items():
            validate_binding(value, f"{group} {name}")
    d1 = require_dict(inputs["d1_roster"], "D1 roster binding")
    if set(d1) != {"bytes", "roster_root_sha256", "sha256"}:
        raise IdentityRepairError("D1 roster binding fields changed")
    validate_binding({"bytes": d1["bytes"], "sha256": d1["sha256"]}, "D1 roster")
    if (
        not isinstance(d1["roster_root_sha256"], str)
        or re.fullmatch(r"[0-9a-f]{64}", d1["roster_root_sha256"]) is None
    ):
        raise IdentityRepairError("D1 roster root is invalid")

    counts = require_dict(profile["expected_counts"], "expected counts")
    if counts != {
        "corrected_material_identity_observed": 134,
        "corrected_physical_parents": 65,
        "corrected_role_parents": CORRECTED_PARENTS,
        "corrected_role_records": CORRECTED_ROLES,
        "identified_recordings": 135,
        "material_quarantine_records": 5,
        "moved_family_records": 20,
        "source_content_objects": 278,
        "source_families": 9,
        "source_physical_parents": 67,
        "source_role_records": EXPECTED_SOURCE_ROLES,
        "total_records": 139,
        "transfer_records": 4,
        "validator_projects_after_repair": 1,
    }:
        raise IdentityRepairError("expected counts changed")

    policy = require_dict(profile["repair_policy"], "repair policy")
    if policy != {
        "alias_overrides": [
            {
                "canonical_parent_id": "realimpact-6-bowl--objectfolder-real-object-6",
                "object_id": "6",
            },
            {
                "canonical_parent_id": MATERIAL_QUARANTINE_PARENT,
                "object_id": "80",
            },
        ],
        "family_reassignment": {
            "family_id": AV_FAMILY,
            "from_role": VALIDATOR_ROLE,
            "to_role": TRAIN_ROLE,
        },
        "material_quarantine": {
            "axis": "material_identity",
            "canonical_parent_id": MATERIAL_QUARANTINE_PARENT,
            "replacement_label": UNKNOWN_MATERIAL,
        },
        "no_audio_or_feature_decode": True,
        "source_component": {
            "component_id": SOURCE_COMPONENT,
            "family_ids": list(SOURCE_FAMILIES),
            "role": TRAIN_ROLE,
        },
    }:
        raise IdentityRepairError("repair policy changed")
    return profile


def prepare_root(path: Path, context: str) -> Path:
    if has_symlink_component(path) or not path.is_dir():
        raise IdentityRepairError(f"{context} must be a non-symlink directory")
    return path.resolve()


def load_d1_roster(path: Path, profile: dict[str, Any]) -> tuple[bytes, dict[str, Any]]:
    expected = profile["input_bindings"]["d1_roster"]
    data = read_regular(path, "D1 roster", 2 * 1024 * 1024)
    if binding(data) != {"bytes": expected["bytes"], "sha256": expected["sha256"]}:
        raise IdentityRepairError("D1 roster binding changed")
    roster = parse_canonical_json(data, "D1 roster")
    if roster.get("schema") != D1_SCHEMA:
        raise IdentityRepairError("D1 roster schema changed")
    if roster.get("roster_root_sha256") != expected["roster_root_sha256"]:
        raise IdentityRepairError("D1 roster root changed")
    families = require_list(roster.get("families"), "D1 families")
    if len(families) != profile["expected_counts"]["source_families"]:
        raise IdentityRepairError("D1 family count changed")
    by_id = {require_dict(row, "D1 family").get("family_id"): row for row in families}
    if len(by_id) != len(families) or any(
        family not in by_id for family in SOURCE_FAMILIES
    ):
        raise IdentityRepairError("D1 family identity changed")
    if by_id[AV_FAMILY].get("role") != VALIDATOR_ROLE:
        raise IdentityRepairError("D1 AV-MSF source role changed")
    if any(
        by_id[family].get("role") != TRAIN_ROLE for family in (RI_FAMILY, OF_FAMILY)
    ):
        raise IdentityRepairError("D1 ObjectFolder/RealImpact source role changed")
    return data, roster


def load_c1a(
    root: Path, profile: dict[str, Any]
) -> tuple[dict[str, bytes], dict[str, dict[str, Any]]]:
    bindings = profile["input_bindings"]["c1a"]
    raw: dict[str, bytes] = {}
    documents: dict[str, dict[str, Any]] = {}
    for name, relative in C1A_FILES.items():
        raw[name], documents[name] = load_bound_json(
            root / relative, bindings[name], f"C1A {name}"
        )
    report = documents["report"]
    plan = documents["repair_plan"]
    findings = documents["alias_findings"]
    if (
        report.get("schema") != C1A_REPORT_SCHEMA
        or report.get("decision") != "C0_IDENTITY_REPAIR_REQUIRED"
    ):
        raise IdentityRepairError("C1A report decision changed")
    gates = require_dict(report.get("gates"), "C1A gates")
    if not gates or not all(value is True for value in gates.values()):
        raise IdentityRepairError("C1A gates are not all true")
    if (
        plan.get("schema") != C1A_PLAN_SCHEMA
        or plan.get("next_stage") != "v43_d2_c0r_identity_repair"
    ):
        raise IdentityRepairError("C1A repair-plan identity changed")
    if (
        findings.get("schema") != C1A_FINDINGS_SCHEMA
        or findings.get("cross_role_finding_count") != 2
    ):
        raise IdentityRepairError("C1A findings identity changed")
    expected_policy = profile["repair_policy"]
    reassignment = require_dict(plan.get("family_reassignment"), "C1A reassignment")
    if {
        key: reassignment.get(key) for key in ("family_id", "from_role", "to_role")
    } != expected_policy["family_reassignment"]:
        raise IdentityRepairError("C1A family reassignment changed")
    aliases = require_list(plan.get("alias_overrides"), "C1A aliases")
    by_object: dict[str, dict[str, Any]] = {}
    for finding in require_list(findings.get("findings"), "C1A findings"):
        finding = require_dict(finding, "C1A finding")
        by_object[str(finding.get("object_id"))] = finding
    for expected_alias in expected_policy["alias_overrides"]:
        finding = by_object.get(expected_alias["object_id"])
        if (
            finding is None
            or finding.get("canonical_parent_id")
            != expected_alias["canonical_parent_id"]
        ):
            raise IdentityRepairError("C1A alias finding changed")
        if not any(
            row.get("canonical_parent_id") == expected_alias["canonical_parent_id"]
            for row in aliases
            if isinstance(row, dict)
        ):
            raise IdentityRepairError("C1A alias plan changed")
    if plan.get("material_quarantine_parent_ids") != [MATERIAL_QUARANTINE_PARENT]:
        raise IdentityRepairError("C1A material quarantine changed")
    return raw, documents


def load_c0(
    root: Path, profile: dict[str, Any]
) -> tuple[dict[str, bytes], dict[str, dict[str, Any]], dict[str, dict[str, Any]]]:
    bindings = profile["input_bindings"]["c0"]
    raw: dict[str, bytes] = {}
    documents: dict[str, dict[str, Any]] = {}
    for name, relative in C0_FILES.items():
        raw[name], documents[name] = load_bound_json(
            root / relative, bindings[name], f"C0 {name}"
        )
    manifest = documents["manifest"]
    source_profile = documents["profile"]
    if manifest.get("schema") != C0_MANIFEST_SCHEMA:
        raise IdentityRepairError("C0 manifest schema changed")
    if manifest.get("profile_sha256") != sha256_bytes(raw["profile"]):
        raise IdentityRepairError("C0 manifest/profile binding changed")
    if (
        source_profile.get("authority", {}).get("protected_access_authority")
        is not False
    ):
        raise IdentityRepairError("C0 protected authority changed")
    records = require_list(manifest.get("records"), "C0 records")
    if len(records) != profile["expected_counts"]["total_records"]:
        raise IdentityRepairError("C0 record count changed")
    by_id: dict[str, dict[str, Any]] = {}
    roles: Counter[str] = Counter()
    for raw_record in records:
        record = require_dict(raw_record, "C0 record")
        record_id = record.get("record_id")
        if not isinstance(record_id, str) or record_id in by_id:
            raise IdentityRepairError("C0 record identity changed")
        by_id[record_id] = record
        roles[str(record.get("role"))] += 1
    if dict(sorted(roles.items())) != EXPECTED_SOURCE_ROLES:
        raise IdentityRepairError("C0 source role counts changed")
    source_kinds = Counter(row.get("source_kind") for row in by_id.values())
    if dict(sorted(source_kinds.items())) != {
        "force_deconvolved_transfer": profile["expected_counts"]["transfer_records"],
        "identified_recording": profile["expected_counts"]["identified_recordings"],
    }:
        raise IdentityRepairError("C0 source-kind counts changed")
    if (
        len({row.get("physical_parent_id") for row in by_id.values()})
        != profile["expected_counts"]["source_physical_parents"]
    ):
        raise IdentityRepairError("C0 source parent count changed")

    projected: set[str] = set()
    for role in ROLES:
        projection = documents[role]
        rows = require_list(projection.get("rows"), f"C0 {role} rows")
        if (
            projection.get("schema") != C0_PROJECTION_SCHEMA
            or projection.get("role") != role
            or projection.get("record_count") != EXPECTED_SOURCE_ROLES[role]
            or projection.get("rows_root_sha256") != sha256_bytes(canonical_json(rows))
        ):
            raise IdentityRepairError(f"C0 {role} projection changed")
        for raw_row in rows:
            row = require_dict(raw_row, f"C0 {role} row")
            record_id = row.get("record_id")
            record = by_id.get(record_id)
            if record is None or record.get("role") != role or record_id in projected:
                raise IdentityRepairError("C0 projection/manifest identity changed")
            projected.add(record_id)
    if projected != set(by_id):
        raise IdentityRepairError("C0 projections do not cover manifest")
    report = documents["report"]
    gates = require_dict(report.get("gates"), "C0 gates")
    if (
        report.get("decision") != "C0_DISCLOSED_CORPUS_REPEATABLE_B0_V0_AUTHORIZED"
        or not gates
        or not all(value is True for value in gates.values())
    ):
        raise IdentityRepairError("C0 source report changed")
    return raw, documents, by_id


def build_d2_roster(
    profile: dict[str, Any],
    d1_bytes: bytes,
    d1: dict[str, Any],
    c1a_raw: dict[str, bytes],
) -> dict[str, Any]:
    repaired_families: list[dict[str, Any]] = []
    for raw_family in require_list(d1.get("families"), "D1 families"):
        family = copy.deepcopy(require_dict(raw_family, "D1 family"))
        source_commitment = family.pop("family_commitment_sha256", None)
        if (
            not isinstance(source_commitment, str)
            or re.fullmatch(r"[0-9a-f]{64}", source_commitment) is None
        ):
            raise IdentityRepairError("D1 family commitment changed")
        family["source_d1_family_commitment_sha256"] = source_commitment
        family["source_d1_family_component_id"] = family.get("family_component_id")
        family["source_d1_role"] = family.get("role")
        if family["family_id"] in SOURCE_FAMILIES:
            family["family_component_id"] = SOURCE_COMPONENT
        if family["family_id"] == AV_FAMILY:
            family["role"] = TRAIN_ROLE
        commitment_core = copy.deepcopy(family)
        family["family_commitment_sha256"] = sha256_bytes(
            canonical_json(commitment_core)
        )
        repaired_families.append(family)
    repaired_families.sort(key=lambda row: row["family_id"])

    component_families: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for family in repaired_families:
        component_families[family["family_component_id"]].append(family)
    component_roles = []
    for component_id, families in sorted(component_families.items()):
        roles = {row["role"] for row in families}
        if len(roles) != 1:
            raise IdentityRepairError("D2 source component crosses roles")
        component_roles.append(
            {
                "component_id": component_id,
                "family_ids": sorted(row["family_id"] for row in families),
                "role": next(iter(roles)),
            }
        )
    role_counts = Counter(row["role"] for row in repaired_families)
    if dict(sorted(role_counts.items())) != {
        DEVELOPMENT_ROLE: 2,
        TRAIN_ROLE: 6,
        VALIDATOR_ROLE: 1,
    }:
        raise IdentityRepairError("D2 family role counts changed")

    core = {
        "authority": AUTHORITY,
        "component_roles": component_roles,
        "families": repaired_families,
        "family_role_counts": dict(sorted(role_counts.items())),
        "repair_policy": profile["repair_policy"],
        "schema": D2_SCHEMA,
        "source_c1a_report_sha256": sha256_bytes(c1a_raw["report"]),
        "source_d1_roster_root_sha256": d1["roster_root_sha256"],
        "source_d1_roster_sha256": sha256_bytes(d1_bytes),
    }
    return {**core, "roster_root_sha256": sha256_bytes(canonical_json(core))}


def repair_records(
    profile: dict[str, Any],
    source_records: dict[str, dict[str, Any]],
    d2_roster: dict[str, Any],
    c1a: dict[str, dict[str, Any]],
) -> tuple[list[dict[str, Any]], dict[str, Any], dict[str, Any]]:
    family_state = {
        row["family_id"]: row
        for row in require_list(d2_roster.get("families"), "D2 families")
    }
    aliases: dict[str, str] = {}
    alias_rows = require_list(c1a["repair_plan"].get("alias_overrides"), "aliases")
    for raw_alias in alias_rows:
        alias = require_dict(raw_alias, "alias")
        canonical = alias.get("canonical_parent_id")
        for current in require_list(alias.get("current_parent_ids"), "current parents"):
            if not isinstance(current, str) or not isinstance(canonical, str):
                raise IdentityRepairError("C1A alias identity changed")
            aliases[current] = canonical

    repaired: list[dict[str, Any]] = []
    moved_ids: list[str] = []
    quarantined: list[dict[str, str]] = []
    alias_members: dict[str, list[str]] = defaultdict(list)
    for source in source_records.values():
        record = copy.deepcopy(source)
        family = family_state.get(record.get("family_id"))
        if family is None:
            raise IdentityRepairError("C0 record family is absent from D2")
        source_role = record["role"]
        record["role"] = family["role"]
        record["family_component_id"] = family["family_component_id"]
        if source_role != record["role"]:
            moved_ids.append(record["record_id"])
        source_parent = record["physical_parent_id"]
        record["physical_parent_id"] = aliases.get(source_parent, source_parent)
        if source_parent in aliases:
            alias_members[record["physical_parent_id"]].append(record["record_id"])
        if record["physical_parent_id"] == MATERIAL_QUARANTINE_PARENT:
            source_material = record["material_label"]
            record["material_label"] = UNKNOWN_MATERIAL
            axis_mask = copy.deepcopy(
                require_dict(record.get("axis_mask"), "axis mask")
            )
            axis_mask["material_identity"] = False
            record["axis_mask"] = axis_mask
            quarantined.append(
                {
                    "record_id": record["record_id"],
                    "source_material_label": source_material,
                }
            )
        repaired.append(record)
    repaired.sort(key=lambda row: row["record_id"])

    roles = Counter(row["role"] for row in repaired)
    if dict(sorted(roles.items())) != CORRECTED_ROLES:
        raise IdentityRepairError("corrected record role counts changed")
    if len(moved_ids) != profile["expected_counts"]["moved_family_records"]:
        raise IdentityRepairError("moved AV-MSF record count changed")
    if len(quarantined) != profile["expected_counts"]["material_quarantine_records"]:
        raise IdentityRepairError("material quarantine record count changed")
    material_observed = sum(
        require_dict(row["axis_mask"], "axis mask").get("material_identity") is True
        for row in repaired
    )
    if (
        material_observed
        != profile["expected_counts"]["corrected_material_identity_observed"]
    ):
        raise IdentityRepairError("corrected material axis count changed")

    parent_roles: dict[str, set[str]] = defaultdict(set)
    component_roles: dict[str, set[str]] = defaultdict(set)
    for row in repaired:
        parent_roles[row["physical_parent_id"]].add(row["role"])
        component_roles[row["family_component_id"]].add(row["role"])
    if any(len(value) != 1 for value in parent_roles.values()) or any(
        len(value) != 1 for value in component_roles.values()
    ):
        raise IdentityRepairError("corrected parent or component crosses roles")
    parent_counts: Counter[str] = Counter()
    for values in parent_roles.values():
        parent_counts[next(iter(values))] += 1
    if dict(sorted(parent_counts.items())) != CORRECTED_PARENTS:
        raise IdentityRepairError("corrected role parent counts changed")

    validator_projects: set[tuple[str, str]] = set()
    for row in repaired:
        if row["role"] != VALIDATOR_ROLE:
            continue
        provenance = require_dict(row.get("provenance"), "provenance")
        publisher = provenance.get("publisher_id")
        project = provenance.get("project_id")
        if not isinstance(publisher, str) or not isinstance(project, str):
            raise IdentityRepairError("validator project identity changed")
        validator_projects.add((publisher, project))
    if (
        len(validator_projects)
        != profile["expected_counts"]["validator_projects_after_repair"]
    ):
        raise IdentityRepairError("corrected validator project count changed")

    repair = {
        "alias_components": [
            {
                "canonical_parent_id": parent,
                "record_ids": sorted(record_ids),
            }
            for parent, record_ids in sorted(alias_members.items())
        ],
        "family_reassignment": {
            **profile["repair_policy"]["family_reassignment"],
            "record_ids": sorted(moved_ids),
        },
        "material_quarantine": {
            **profile["repair_policy"]["material_quarantine"],
            "records": sorted(quarantined, key=lambda row: row["record_id"]),
        },
        "schema": REPAIR_SCHEMA,
    }
    isolation = {
        "physical_parent_count": len(parent_roles),
        "role_parent_counts": dict(sorted(parent_counts.items())),
        "role_record_counts": dict(sorted(roles.items())),
        "validator_project_count": len(validator_projects),
    }
    return repaired, repair, isolation


def load_content_objects(
    source_root: Path,
    records: list[dict[str, Any]],
    expected_count: int,
) -> tuple[dict[str, bytes], dict[str, Any]]:
    expected: dict[str, dict[str, Any]] = {}
    kinds: Counter[str] = Counter()
    for record in records:
        for field, prefix in (
            ("canonical_pcm", "objects/pcm/"),
            ("acoustic_target", "objects/features/"),
        ):
            artifact = require_dict(record.get(field), f"record {field}")
            if set(artifact) != {"bytes", "path", "sha256"}:
                raise IdentityRepairError(f"record {field} binding changed")
            relative = artifact["path"]
            if not isinstance(relative, str) or not relative.startswith(prefix):
                raise IdentityRepairError(f"record {field} path changed")
            path = Path(relative)
            if path.is_absolute() or ".." in path.parts:
                raise IdentityRepairError("content-object path escapes source root")
            previous = expected.setdefault(relative, artifact)
            if previous != artifact:
                raise IdentityRepairError("content-object binding conflicts")
    if len(expected) != expected_count:
        raise IdentityRepairError("source content-object count changed")

    object_root = source_root / "objects"
    if has_symlink_component(object_root) or not object_root.is_dir():
        raise IdentityRepairError("C0 objects root changed")
    actual = {
        path.relative_to(source_root).as_posix()
        for path in object_root.rglob("*")
        if path.is_file()
    }
    if actual != set(expected):
        raise IdentityRepairError("C0 object tree differs from manifest bindings")

    files: dict[str, bytes] = {}
    total = 0
    for relative, artifact in sorted(expected.items()):
        data = read_regular(
            source_root / relative, f"content object {relative}", 4 * 1024 * 1024
        )
        if binding(data) != {"bytes": artifact["bytes"], "sha256": artifact["sha256"]}:
            raise IdentityRepairError(f"content object binding changed: {relative}")
        files[relative] = data
        total += len(data)
        kinds["pcm" if relative.startswith("objects/pcm/") else "features"] += 1
    if dict(sorted(kinds.items())) != {"features": 139, "pcm": 139}:
        raise IdentityRepairError("content-object kind counts changed")
    return files, {
        "bytes": total,
        "count": len(files),
        "kinds": dict(sorted(kinds.items())),
    }


def projection_rows(records: list[dict[str, Any]], role: str) -> list[dict[str, Any]]:
    return [
        {
            "acoustic_target": row["acoustic_target"],
            "axis_mask": row["axis_mask"],
            "canonical_pcm": row["canonical_pcm"],
            "family_component_id": row["family_component_id"],
            "family_id": row["family_id"],
            "material_label": row["material_label"],
            "physical_parent_id": row["physical_parent_id"],
            "record_id": row["record_id"],
        }
        for row in records
        if row["role"] == role
    ]


def build_outputs(
    profile_bytes: bytes,
    profile: dict[str, Any],
    source_root: Path,
    d1_bytes: bytes,
    d1: dict[str, Any],
    c0_raw: dict[str, bytes],
    c0: dict[str, dict[str, Any]],
    source_records: dict[str, dict[str, Any]],
    c1a_raw: dict[str, bytes],
    c1a: dict[str, dict[str, Any]],
) -> dict[str, bytes]:
    d2_roster = build_d2_roster(profile, d1_bytes, d1, c1a_raw)
    records, repair, isolation = repair_records(profile, source_records, d2_roster, c1a)
    content_files, content = load_content_objects(
        source_root, records, profile["expected_counts"]["source_content_objects"]
    )
    files = dict(content_files)

    d2_bytes = canonical_json(d2_roster)
    repair_bytes = canonical_json(repair)
    files["disclosed-roster.json"] = d2_bytes
    files["lineage-repair.json"] = repair_bytes

    projection_bindings: dict[str, dict[str, Any]] = {}
    for role in ROLES:
        rows = projection_rows(records, role)
        projection = {
            "record_count": len(rows),
            "role": role,
            "rows": rows,
            "rows_root_sha256": sha256_bytes(canonical_json(rows)),
            "schema": C0R_PROJECTION_SCHEMA,
        }
        data = canonical_json(projection)
        relative = f"projections/{role}.json"
        files[relative] = data
        projection_bindings[role] = {
            **binding(data),
            "path": relative,
            "rows_root_sha256": projection["rows_root_sha256"],
        }

    source_manifest = c0["manifest"]
    manifest_core = {
        "authority": AUTHORITY,
        "corpus_policy": source_manifest["corpus_policy"],
        "d2_roster_root_sha256": d2_roster["roster_root_sha256"],
        "d2_roster_sha256": sha256_bytes(d2_bytes),
        "feature_policy": source_manifest["feature_policy"],
        "profile_sha256": sha256_bytes(profile_bytes),
        "projections": projection_bindings,
        "records": records,
        "schema": C0R_MANIFEST_SCHEMA,
        "source_c0_manifest_root_sha256": source_manifest["manifest_root_sha256"],
        "source_c0_manifest_sha256": sha256_bytes(c0_raw["manifest"]),
        "source_c1a_report_sha256": sha256_bytes(c1a_raw["report"]),
    }
    manifest_root = sha256_bytes(canonical_json(manifest_core))
    manifest = {**manifest_core, "manifest_root_sha256": manifest_root}
    manifest_bytes = canonical_json(manifest)
    files["manifest.json"] = manifest_bytes

    material_counts = Counter(row["material_label"] for row in records)
    axes = sorted(require_dict(records[0]["axis_mask"], "axis mask"))
    axis_counts = {
        axis: sum(
            require_dict(row["axis_mask"], "axis mask").get(axis) is True
            for row in records
        )
        for axis in axes
    }
    card = {
        "axis_observed_record_counts": axis_counts,
        "claim": (
            "lineage-corrected disclosed internet evidence for repeatable lab "
            "baselines and descriptor growth only"
        ),
        "limitations": [
            "AV-MSF is co-located with its disclosed ObjectFolder/RealImpact source component",
            "object 80 material identity is quarantined after a Wood/Ceramic conflict",
            "one validator-calibration project and parent have insufficient independent power",
            "no record grants model training, validator calibration, admission, product, cooker, demo, or runtime authority",
        ],
        "material_record_counts": dict(sorted(material_counts.items())),
        "physical_parent_count": isolation["physical_parent_count"],
        "record_count": len(records),
        "role_parent_counts": isolation["role_parent_counts"],
        "role_record_counts": isolation["role_record_counts"],
        "schema": C0R_CARD_SCHEMA,
        "validator_project_count": isolation["validator_project_count"],
    }
    card_bytes = canonical_json(card)
    files["corpus-card.json"] = card_bytes

    access = {
        "acoustic_feature_objects_decoded": 0,
        "audio_decoded_samples": 0,
        "candidate_model_bytes_read": 0,
        "content_object_bytes_copied": content["bytes"],
        "content_objects_copied": content["count"],
        "network_requests": 0,
        "protected_payload_bytes_read": 0,
        "source_c0_metadata_bytes_read": sum(len(data) for data in c0_raw.values()),
        "source_c1a_bytes_read": sum(len(data) for data in c1a_raw.values()),
        "source_d1_bytes_read": len(d1_bytes),
        "validator_candidate_outputs_read": 0,
    }
    access_ledger = {
        "access": access,
        "authority": AUTHORITY,
        "profile_sha256": sha256_bytes(profile_bytes),
        "schema": ACCESS_SCHEMA,
        "source_c0_manifest_sha256": sha256_bytes(c0_raw["manifest"]),
        "source_c1a_report_sha256": sha256_bytes(c1a_raw["report"]),
    }
    access_bytes = canonical_json(access_ledger)
    files["access-ledger.json"] = access_bytes

    gates = {
        "all_source_content_objects_hash_verified": content["count"]
        == profile["expected_counts"]["source_content_objects"],
        "all_source_metadata_hash_bound": True,
        "c1a_repair_plan_hash_bound": True,
        "corrected_components_role_disjoint": True,
        "corrected_parents_role_disjoint": True,
        "material_conflict_quarantined": len(repair["material_quarantine"]["records"])
        == profile["expected_counts"]["material_quarantine_records"],
        "no_audio_or_acoustic_feature_decode": True,
        "no_candidate_protected_validator_or_network_access": True,
        "whole_av_msf_family_reassigned": len(
            repair["family_reassignment"]["record_ids"]
        )
        == profile["expected_counts"]["moved_family_records"],
    }
    if not all(gates.values()):
        raise IdentityRepairError("D2/C0R conjunctive gate failed")
    report = {
        "access": access,
        "artifacts": {
            "access_ledger_sha256": sha256_bytes(access_bytes),
            "corpus_card_sha256": sha256_bytes(card_bytes),
            "d2_roster_root_sha256": d2_roster["roster_root_sha256"],
            "d2_roster_sha256": sha256_bytes(d2_bytes),
            "lineage_repair_sha256": sha256_bytes(repair_bytes),
            "manifest_root_sha256": manifest_root,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "profile_sha256": sha256_bytes(profile_bytes),
        },
        "authority": AUTHORITY,
        "decision": DECISION,
        "gates": gates,
        "result": {
            "content_object_count": content["count"],
            "identified_recordings": profile["expected_counts"][
                "identified_recordings"
            ],
            "material_quarantine_records": len(
                repair["material_quarantine"]["records"]
            ),
            "physical_parents": isolation["physical_parent_count"],
            "role_parents": isolation["role_parent_counts"],
            "role_records": isolation["role_record_counts"],
            "total_records": len(records),
            "transfer_records": profile["expected_counts"]["transfer_records"],
            "validator_projects": isolation["validator_project_count"],
        },
        "schema": REPORT_SCHEMA,
    }
    files["profile.json"] = profile_bytes
    files["report.json"] = canonical_json(report)
    return files


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise IdentityRepairError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise IdentityRepairError("output must remain outside the repository")
    if resolved.exists():
        raise IdentityRepairError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish_directory(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for relative, data in sorted(files.items()):
            path = Path(relative)
            if path.is_absolute() or ".." in path.parts:
                raise IdentityRepairError("output path escapes the corpus root")
            target = staging / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(
    profile_path: Path,
    d1_roster_path: Path,
    c0_root: Path,
    c1a_root: Path,
    output: Path,
) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(raw_profile)
    d1_bytes, d1 = load_d1_roster(d1_roster_path, profile)
    c0_raw, c0, source_records = load_c0(prepare_root(c0_root, "C0 root"), profile)
    c1a_raw, c1a = load_c1a(prepare_root(c1a_root, "C1A root"), profile)
    return publish_directory(
        output,
        build_outputs(
            profile_bytes,
            profile,
            c0_root.resolve(),
            d1_bytes,
            d1,
            c0_raw,
            c0,
            source_records,
            c1a_raw,
            c1a,
        ),
    )


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--d1-roster", required=True, type=Path)
    parser.add_argument("--c0-root", required=True, type=Path)
    parser.add_argument("--c1a-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    print(
        run(
            arguments.profile,
            arguments.d1_roster,
            arguments.c0_root,
            arguments.c1a_root,
            arguments.output,
        )
    )


if __name__ == "__main__":
    main()
