#!/usr/bin/env python3
"""Audit disclosed physical-sound parent lineage before V43 corpus repair."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-profile.v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-corpus.v1"
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-role-projection.v1"
EVIDENCE_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-evidence.v1"
FINDINGS_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-findings.v1"
PLAN_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-repair-plan.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-report.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v43-c1a-access.v1"

PROFILE_ID = "physical-sound-v43-c1a-lineage-audit-v1"
DECISION = "C0_IDENTITY_REPAIR_REQUIRED"
NEXT_STAGE = "v43_d2_c0r_identity_repair"
SOURCE_ROLE = "validator_calibration"
DESTINATION_ROLE = "generator_train"
AV_FAMILY = "zisen-shao--av-msf"
OF_FAMILY = "stanford-objectfolder--objectfolder-real"
EXPECTED_ROLES = {
    "generator_development": 70,
    "generator_train": 44,
    "validator_calibration": 25,
}
CORRECTED_ROLES = {
    "generator_development": 70,
    "generator_train": 64,
    "validator_calibration": 5,
}
CORRECTED_PARENTS = {
    "generator_development": 30,
    "generator_train": 34,
    "validator_calibration": 1,
}
EVIDENCE_FILES = (
    "av-msf-object-6.png",
    "av-msf-object-80.png",
    "av-msf-page.html",
    "av-msf-paper-v2.pdf",
    "av-msf-paper-v2.txt",
    "objectfolder-object-6-clip-0.mp4",
    "objectfolder-object-80-clip-0.mp4",
    "objectfolder-real-download.html",
)
PROJECTION_FILES = {
    "generator_development": "projections/generator_development.json",
    "generator_train": "projections/generator_train.json",
    "validator_calibration": "projections/validator_calibration.json",
}
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": [NEXT_STAGE],
    "candidate_training_authority": False,
    "descriptor_baseline_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}


class LineageAuditError(RuntimeError):
    """The C1A lineage evidence cannot support an atomic result."""


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
        raise LineageAuditError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise LineageAuditError(f"{context} must be an array")
    return value


def read_regular(path: Path, context: str, maximum: int = 64 * 1024 * 1024) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise LineageAuditError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise LineageAuditError(f"{context} size is outside the frozen bound")
    data = path.read_bytes()
    if len(data) != size:
        raise LineageAuditError(f"{context} changed while reading")
    return data


def parse_canonical_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise LineageAuditError(f"{context} is not valid JSON") from error
    value = require_dict(value, context)
    if canonical_json(value) != data:
        raise LineageAuditError(f"{context} must use canonical JSON")
    return value


def parse_source_canonical_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise LineageAuditError(f"{context} is not valid JSON") from error
    value = require_dict(value, context)
    if canonical_json(value) != data:
        raise LineageAuditError(f"{context} must use the frozen C0 canonical JSON")
    return value


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, "C1A profile", 512 * 1024)
    return data, parse_canonical_json(data, "C1A profile")


def validate_binding(value: Any, context: str) -> dict[str, Any]:
    value = require_dict(value, context)
    if set(value) != {"bytes", "sha256"}:
        raise LineageAuditError(f"{context} binding fields changed")
    size = value["bytes"]
    digest = value["sha256"]
    if not isinstance(size, int) or size <= 0:
        raise LineageAuditError(f"{context} byte count is invalid")
    if not isinstance(digest, str) or re.fullmatch(r"[0-9a-f]{64}", digest) is None:
        raise LineageAuditError(f"{context} SHA-256 is invalid")
    return value


def validate_profile(profile: dict[str, Any]) -> dict[str, Any]:
    expected_fields = {
        "alias_claims",
        "authority",
        "decision",
        "dependency_bindings",
        "evidence_bindings",
        "evidence_policy",
        "expected_counts",
        "input_bindings",
        "profile_id",
        "revision",
        "schema",
    }
    if set(profile) != expected_fields:
        raise LineageAuditError("C1A profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["profile_id"] != PROFILE_ID:
        raise LineageAuditError("unknown C1A profile identity")
    if profile["revision"] != 1 or profile["decision"] != DECISION:
        raise LineageAuditError("C1A revision or decision changed")
    if profile["authority"] != AUTHORITY:
        raise LineageAuditError("C1A authority changed")

    dependencies = require_list(profile["dependency_bindings"], "dependencies")
    required_paths = {
        "docs/architecture/45-physical-sound-synthesis-and-acoustic-presentation.md",
        "docs/development/physical-sound-v41-c0-disclosed-corpus-result-2026-09-03.md",
        "docs/development/physical-sound-v42-r0-domain-information-audit-result-2026-09-03.md",
        "lab/profiles/physical-sound-v41-c0-disclosed-corpus.v1.json",
        "lab/scripts/physical_sound_v43_c1a_lineage_audit_v1.py",
    }
    seen: set[str] = set()
    root = repository_root()
    for raw in dependencies:
        row = require_dict(raw, "dependency binding")
        if set(row) != {"bytes", "path", "sha256"}:
            raise LineageAuditError("dependency binding fields changed")
        path = row["path"]
        if not isinstance(path, str) or path in seen or path not in required_paths:
            raise LineageAuditError("dependency binding path changed")
        seen.add(path)
        data = read_regular(root / path, f"dependency {path}", 2 * 1024 * 1024)
        if binding(data) != {"bytes": row["bytes"], "sha256": row["sha256"]}:
            raise LineageAuditError(f"dependency binding changed: {path}")
    if seen != required_paths:
        raise LineageAuditError("dependency binding set changed")

    inputs = require_dict(profile["input_bindings"], "input bindings")
    if set(inputs) != {"manifest", "profile", *PROJECTION_FILES}:
        raise LineageAuditError("C0 input binding set changed")
    for name, value in inputs.items():
        validate_binding(value, f"C0 input {name}")

    evidence = require_dict(profile["evidence_bindings"], "evidence bindings")
    if set(evidence) != set(EVIDENCE_FILES):
        raise LineageAuditError("source evidence binding set changed")
    for name, value in evidence.items():
        validate_binding(value, f"source evidence {name}")

    policy = require_dict(profile["evidence_policy"], "evidence policy")
    if policy != {
        "allow_network": False,
        "av_msf_dataset_marker": (
            "we evaluate our method on objectfolder real [12] and realimpact [2], "
            "two real-world multisensory datasets that include impact sound recordings."
        ),
        "metadata_only": True,
        "source_urls": {
            "av_msf_page": "https://zisenshao.github.io/AV-MSF/",
            "av_msf_paper": "https://arxiv.org/abs/2608.05145",
            "objectfolder_real": (
                "https://objectfolder.stanford.edu/objectfolder-real-download"
            ),
        },
    }:
        raise LineageAuditError("source evidence policy changed")

    counts = require_dict(profile["expected_counts"], "expected counts")
    if counts != {
        "alias_findings": 2,
        "av_msf_family_records": 20,
        "corrected_physical_parents": CORRECTED_PARENTS,
        "corrected_role_records": CORRECTED_ROLES,
        "current_physical_parents": 67,
        "current_role_records": EXPECTED_ROLES,
        "objectfolder_family_records": 15,
    }:
        raise LineageAuditError("expected counts changed")

    claims = require_list(profile["alias_claims"], "alias claims")
    expected_claims = [
        {
            "av_msf_material": "Glass",
            "canonical_parent_id": "realimpact-6-bowl--objectfolder-real-object-6",
            "finding": "ConfirmedPhysicalAlias",
            "object_id": "6",
            "objectfolder_material": "Glass",
            "objectfolder_name": "Blue_Bowl",
        },
        {
            "av_msf_material": "Ceramic",
            "canonical_parent_id": "objectfolder-real-object-80--av-msf-object-80",
            "finding": "QuarantineIdentityConflict",
            "object_id": "80",
            "objectfolder_material": "Wood",
            "objectfolder_name": "Spoon_Holder",
        },
    ]
    if claims != expected_claims:
        raise LineageAuditError("alias claims changed")
    return profile


def load_bound_json(
    path: Path, expected: Any, context: str
) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context, 2 * 1024 * 1024)
    expected = validate_binding(expected, f"{context} expected")
    if binding(data) != expected:
        raise LineageAuditError(f"{context} binding changed")
    return data, parse_source_canonical_json(data, context)


def load_bound_bytes(path: Path, expected: Any, context: str) -> bytes:
    data = read_regular(path, context)
    expected = validate_binding(expected, f"{context} expected")
    if binding(data) != expected:
        raise LineageAuditError(f"{context} binding changed")
    return data


def normalized_text(data: bytes, context: str) -> str:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise LineageAuditError(f"{context} is not UTF-8") from error
    text = re.sub(r"-\s+", "", text)
    return " ".join(text.lower().split())


def parse_av_msf_cards(html: bytes) -> dict[str, str]:
    text = html.decode("utf-8", errors="strict")
    cards: dict[str, str] = {}
    pattern = re.compile(
        r'<button\s+class="object-card(?: active)?".*?'
        r'data-name="Object (?P<object>[0-9]+)".*?'
        r'data-original-material="(?P<material>[A-Za-z]+)".*?'
        r'data-demo-path="data/demo/(?P=object)"',
        re.DOTALL,
    )
    for match in pattern.finditer(text):
        object_id = match.group("object")
        if object_id in cards:
            raise LineageAuditError("duplicate AV-MSF object card")
        cards[object_id] = match.group("material")
    return cards


def require_objectfolder_row(
    html: bytes, object_id: str, name: str, material: str
) -> None:
    text = html.decode("utf-8", errors="strict")
    cell = r'<td style="text-align: center">{}</td>'
    pattern = re.compile(
        cell.format(re.escape(object_id))
        + r"\s*"
        + cell.format(re.escape(name))
        + r"\s*"
        + cell.format(re.escape(material))
    )
    if pattern.search(text) is None:
        raise LineageAuditError(f"ObjectFolder row changed for object {object_id}")


def validate_corpus(
    corpus_root: Path, profile: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, bytes], dict[str, dict[str, Any]]]:
    inputs = require_dict(profile["input_bindings"], "input bindings")
    raw: dict[str, bytes] = {}
    raw["manifest"], manifest = load_bound_json(
        corpus_root / "manifest.json", inputs["manifest"], "C0 manifest"
    )
    raw["profile"], corpus_profile = load_bound_json(
        corpus_root / "profile.json", inputs["profile"], "C0 profile"
    )
    if manifest.get("schema") != MANIFEST_SCHEMA:
        raise LineageAuditError("C0 manifest schema changed")
    if sha256_bytes(raw["profile"]) != manifest.get("profile_sha256"):
        raise LineageAuditError("C0 manifest/profile binding changed")
    if (
        corpus_profile.get("authority", {}).get("protected_access_authority")
        is not False
    ):
        raise LineageAuditError("C0 protected authority changed")

    projections: dict[str, dict[str, Any]] = {}
    for role, relative in PROJECTION_FILES.items():
        raw[role], projection = load_bound_json(
            corpus_root / relative, inputs[role], f"C0 {role} projection"
        )
        if (
            projection.get("schema") != PROJECTION_SCHEMA
            or projection.get("role") != role
        ):
            raise LineageAuditError(f"C0 {role} projection identity changed")
        rows = require_list(projection.get("rows"), f"C0 {role} rows")
        if len(rows) != EXPECTED_ROLES[role]:
            raise LineageAuditError(f"C0 {role} row count changed")
        projections[role] = projection

    records = require_list(manifest.get("records"), "C0 records")
    if len(records) != sum(EXPECTED_ROLES.values()):
        raise LineageAuditError("C0 manifest record count changed")
    by_id: dict[str, dict[str, Any]] = {}
    role_counts: Counter[str] = Counter()
    for raw_record in records:
        record = require_dict(raw_record, "C0 record")
        record_id = record.get("record_id")
        role = record.get("role")
        if not isinstance(record_id, str) or record_id in by_id:
            raise LineageAuditError("C0 record identity changed")
        if role not in EXPECTED_ROLES:
            raise LineageAuditError("C0 record role changed")
        by_id[record_id] = record
        role_counts[role] += 1
    if dict(sorted(role_counts.items())) != EXPECTED_ROLES:
        raise LineageAuditError("C0 role counts changed")

    projected_ids: set[str] = set()
    for role, projection in projections.items():
        for raw_row in require_list(projection["rows"], f"C0 {role} rows"):
            row = require_dict(raw_row, f"C0 {role} row")
            record_id = row.get("record_id")
            record = by_id.get(record_id)
            if record is None or record["role"] != role or record_id in projected_ids:
                raise LineageAuditError("C0 projection/manifest identity changed")
            projected_ids.add(record_id)
    if projected_ids != set(by_id):
        raise LineageAuditError("C0 projections do not cover the manifest exactly")
    return manifest, raw, by_id


def validate_evidence(
    evidence_root: Path, profile: dict[str, Any]
) -> tuple[dict[str, bytes], list[dict[str, Any]], dict[str, str]]:
    expected = require_dict(profile["evidence_bindings"], "evidence bindings")
    raw: dict[str, bytes] = {}
    inventory: list[dict[str, Any]] = []
    for name in EVIDENCE_FILES:
        data = load_bound_bytes(
            evidence_root / name, expected[name], f"evidence {name}"
        )
        raw[name] = data
        inventory.append({"file": name, **binding(data)})

    marker = profile["evidence_policy"]["av_msf_dataset_marker"]
    if marker not in normalized_text(raw["av-msf-paper-v2.txt"], "AV-MSF paper text"):
        raise LineageAuditError("AV-MSF ObjectFolder Real dataset marker changed")
    cards = parse_av_msf_cards(raw["av-msf-page.html"])
    if cards.get("6") != "Glass" or cards.get("80") != "Ceramic":
        raise LineageAuditError("AV-MSF alias-card evidence changed")
    require_objectfolder_row(
        raw["objectfolder-real-download.html"], "6", "Blue_Bowl", "Glass"
    )
    require_objectfolder_row(
        raw["objectfolder-real-download.html"], "80", "Spoon_Holder", "Wood"
    )
    return raw, inventory, cards


def records_for(
    records: dict[str, dict[str, Any]], family: str, object_id: str | None = None
) -> list[dict[str, Any]]:
    result = [
        record for record in records.values() if record.get("family_id") == family
    ]
    if object_id is not None:
        result = [
            record for record in result if str(record.get("object_id")) == object_id
        ]
    return sorted(result, key=lambda row: row["record_id"])


def build_findings(
    profile: dict[str, Any], records: dict[str, dict[str, Any]]
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    av_records = records_for(records, AV_FAMILY)
    of_records = records_for(records, OF_FAMILY)
    if len(av_records) != 20 or len(of_records) != 15:
        raise LineageAuditError("source-family record counts changed")
    if {row["role"] for row in av_records} != {SOURCE_ROLE}:
        raise LineageAuditError("AV-MSF no longer occupies the expected source role")
    if {row["role"] for row in of_records} != {DESTINATION_ROLE}:
        raise LineageAuditError(
            "ObjectFolder no longer occupies the expected destination role"
        )
    current_parents = {row["physical_parent_id"] for row in records.values()}
    if len(current_parents) != profile["expected_counts"]["current_physical_parents"]:
        raise LineageAuditError("current physical-parent count changed")

    findings: list[dict[str, Any]] = []
    aliases: dict[str, str] = {}
    material_quarantine: list[str] = []
    for claim in profile["alias_claims"]:
        object_id = claim["object_id"]
        av = records_for(records, AV_FAMILY, object_id)
        of = records_for(records, OF_FAMILY, object_id)
        if not av or not of:
            raise LineageAuditError(
                f"missing alias-side records for object {object_id}"
            )
        av_materials = sorted({row["material_label"] for row in av})
        of_materials = sorted({row["material_label"] for row in of})
        if av_materials != [claim["av_msf_material"]]:
            raise LineageAuditError(f"AV-MSF material changed for object {object_id}")
        if of_materials != [claim["objectfolder_material"]]:
            raise LineageAuditError(
                f"ObjectFolder material changed for object {object_id}"
            )
        current_parents = sorted({row["physical_parent_id"] for row in [*av, *of]})
        if len(current_parents) != 2:
            raise LineageAuditError(
                f"object {object_id} is no longer split across parents"
            )
        for parent in current_parents:
            aliases[parent] = claim["canonical_parent_id"]
        conflict = av_materials != of_materials
        if conflict:
            material_quarantine.append(claim["canonical_parent_id"])
        findings.append(
            {
                "canonical_parent_id": claim["canonical_parent_id"],
                "current_parent_ids": current_parents,
                "finding": claim["finding"],
                "material_conflict": conflict,
                "material_labels": sorted({*av_materials, *of_materials}),
                "object_id": object_id,
                "objectfolder_name": claim["objectfolder_name"],
                "record_count": len(av) + len(of),
                "roles": sorted({row["role"] for row in [*av, *of]}),
            }
        )

    corrected_role_counts: Counter[str] = Counter()
    corrected_parent_roles: dict[str, set[str]] = defaultdict(set)
    corrected_validator_projects: set[tuple[str, str]] = set()
    for record in records.values():
        role = DESTINATION_ROLE if record["family_id"] == AV_FAMILY else record["role"]
        parent = aliases.get(record["physical_parent_id"], record["physical_parent_id"])
        corrected_role_counts[role] += 1
        corrected_parent_roles[parent].add(role)
        if role == SOURCE_ROLE:
            provenance = require_dict(record.get("provenance"), "record provenance")
            publisher = provenance.get("publisher_id")
            project = provenance.get("project_id")
            if not isinstance(publisher, str) or not isinstance(project, str):
                raise LineageAuditError("validator project identity changed")
            corrected_validator_projects.add((publisher, project))
    if dict(sorted(corrected_role_counts.items())) != CORRECTED_ROLES:
        raise LineageAuditError("corrected role counts changed")
    if any(len(roles) != 1 for roles in corrected_parent_roles.values()):
        raise LineageAuditError("corrected plan still crosses roles")
    parent_counts: Counter[str] = Counter()
    for roles in corrected_parent_roles.values():
        parent_counts[next(iter(roles))] += 1
    if dict(sorted(parent_counts.items())) != CORRECTED_PARENTS:
        raise LineageAuditError("corrected parent counts changed")
    if len(corrected_validator_projects) != 1:
        raise LineageAuditError("corrected validator project count changed")

    plan = {
        "alias_overrides": [
            {
                "canonical_parent_id": finding["canonical_parent_id"],
                "current_parent_ids": finding["current_parent_ids"],
            }
            for finding in findings
        ],
        "corrected_physical_parent_count": len(corrected_parent_roles),
        "corrected_role_parent_counts": dict(sorted(parent_counts.items())),
        "corrected_role_record_counts": dict(sorted(corrected_role_counts.items())),
        "family_reassignment": {
            "family_id": AV_FAMILY,
            "from_role": SOURCE_ROLE,
            "record_count": len(av_records),
            "to_role": DESTINATION_ROLE,
        },
        "material_quarantine_parent_ids": sorted(material_quarantine),
        "next_stage": NEXT_STAGE,
        "schema": PLAN_SCHEMA,
        "validator_calibration_project_count_after_repair": len(
            corrected_validator_projects
        ),
        "validator_calibration_status_after_repair": "InsufficientIndependentPower",
    }
    return findings, plan


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    corpus_root: Path,
    evidence_root: Path,
) -> dict[str, bytes]:
    manifest, corpus_raw, records = validate_corpus(corpus_root, profile)
    evidence_raw, evidence_inventory, _cards = validate_evidence(evidence_root, profile)
    findings, repair_plan = build_findings(profile, records)

    evidence = {
        "artifacts": evidence_inventory,
        "claims": {
            "av_msf_uses_objectfolder_real": True,
            "object_6_card_and_table_identity_present": True,
            "object_80_card_and_table_identity_present": True,
            "visual_identity_assets_hash_bound_not_model_scored": True,
        },
        "schema": EVIDENCE_SCHEMA,
        "source_urls": profile["evidence_policy"]["source_urls"],
    }
    findings_document = {
        "cross_role_finding_count": len(findings),
        "findings": findings,
        "schema": FINDINGS_SCHEMA,
    }
    gates = {
        "all_c0_metadata_hash_bound": True,
        "all_public_evidence_hash_bound": True,
        "av_msf_objectfolder_dataset_relation_present": True,
        "corrected_parent_roles_disjoint": True,
        "current_cross_role_alias_found": len(findings) == 2,
        "no_audio_decode_or_acoustic_feature_access": True,
        "no_candidate_validator_protected_or_network_access": True,
        "whole_family_repair_plan_complete": True,
    }
    if not all(gates.values()):
        raise LineageAuditError("C1A conjunctive gate failed")

    access = {
        "acoustic_feature_objects_read": 0,
        "audio_decoded_samples": 0,
        "candidate_model_bytes_read": 0,
        "corpus_metadata_bytes_read": sum(len(data) for data in corpus_raw.values()),
        "network_requests": 0,
        "protected_payload_bytes_read": 0,
        "source_evidence_bytes_read": sum(len(data) for data in evidence_raw.values()),
        "validator_candidate_outputs_read": 0,
    }
    access_ledger = {
        "access": access,
        "authority": AUTHORITY,
        "corpus_manifest_sha256": sha256_bytes(corpus_raw["manifest"]),
        "profile_sha256": sha256_bytes(profile_bytes),
        "schema": ACCESS_SCHEMA,
    }
    report = {
        "authority": AUTHORITY,
        "decision": DECISION,
        "gates": gates,
        "input": {
            "c0_manifest_root_sha256": manifest["manifest_root_sha256"],
            "c0_manifest_sha256": sha256_bytes(corpus_raw["manifest"]),
            "c0_profile_sha256": sha256_bytes(corpus_raw["profile"]),
            "public_evidence_root_sha256": sha256_bytes(
                canonical_json(evidence_inventory)
            ),
        },
        "next_actions": [NEXT_STAGE, "v43_c1_descriptor_source_growth"],
        "result": {
            "alias_findings": len(findings),
            "corrected_physical_parents": repair_plan[
                "corrected_physical_parent_count"
            ],
            "corrected_role_records": repair_plan["corrected_role_record_counts"],
            "moved_family_records": repair_plan["family_reassignment"]["record_count"],
            "validator_calibration_projects_after_repair": repair_plan[
                "validator_calibration_project_count_after_repair"
            ],
            "validator_calibration_status_after_repair": repair_plan[
                "validator_calibration_status_after_repair"
            ],
        },
        "schema": REPORT_SCHEMA,
    }
    documents = {
        "access-ledger.json": canonical_json(access_ledger),
        "alias-findings.json": canonical_json(findings_document),
        "evidence-inventory.json": canonical_json(evidence),
        "profile.json": profile_bytes,
        "repair-plan.json": canonical_json(repair_plan),
        "report.json": canonical_json(report),
    }
    return documents


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def prepare_root(path: Path, context: str) -> Path:
    if has_symlink_component(path) or not path.is_dir():
        raise LineageAuditError(f"{context} must be a non-symlink directory")
    return path.resolve()


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise LineageAuditError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise LineageAuditError("output must remain outside the repository")
    if resolved.exists():
        raise LineageAuditError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish_directory(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for name, data in sorted(files.items()):
            if Path(name).name != name:
                raise LineageAuditError("C1A outputs must be flat files")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(
    profile_path: Path,
    corpus_root: Path,
    evidence_root: Path,
    output: Path,
) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(raw_profile)
    return publish_directory(
        output,
        build_documents(
            profile_bytes,
            profile,
            prepare_root(corpus_root, "C0 corpus root"),
            prepare_root(evidence_root, "public evidence root"),
        ),
    )


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--evidence-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    print(
        run(
            arguments.profile,
            arguments.corpus_root,
            arguments.evidence_root,
            arguments.output,
        )
    )


if __name__ == "__main__":
    main()
