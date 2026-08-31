#!/usr/bin/env python3
"""Validate and freeze the zero-signal Physical Sound Dataset Contract V1."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_exposure_ledger_v0 as ledger_v0

CONTRACT_SCHEMA = "nextengine.experimental-physical-sound-dataset-contract.v1"
DESCRIPTOR_SCHEMA = (
    "nextengine.experimental-physical-sound-dataset-contract-descriptor.v1"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v14-n1a.report.v1"
CLAIM_KIND = "learned_canonical_impact_prior"
MAX_INPUT_BYTES = 16 * 1024 * 1024
MAX_SOURCES = 1024
MAX_OBJECTS = 100_000

MATERIALS = ("Glass", "Metal", "Wood")
ACTIVE_ROLES = (
    "admission_shadow",
    "generator_development",
    "generator_train",
    "validator_calibration",
    "validator_method_holdout",
)
ALL_ROLES = tuple(sorted((*ACTIVE_ROLES, "excluded")))
PROTECTED_ROLES = (
    "admission_shadow",
    "validator_calibration",
    "validator_method_holdout",
)
GENERATOR_ROLES = ("generator_development", "generator_train")
MINIMUM_PER_MATERIAL = {
    "admission_shadow": 1,
    "generator_development": 1,
    "generator_train": 4,
    "validator_calibration": 1,
    "validator_method_holdout": 1,
}

TIERS = (
    "T0_known_truth",
    "T1_synthetic_teacher",
    "T2_sparse_real_contact",
    "T3_dense_real_listener",
    "T4_audio_only_real",
)
REAL_CONTACT_TIERS = {"T2_sparse_real_contact", "T3_dense_real_listener"}
TRAINING_TIERS = REAL_CONTACT_TIERS | {"T0_known_truth", "T1_synthetic_teacher"}
QUALITY_CLASSES = ("evaluation_complete", "source_ood", "training_usable")
QUALITY_EVIDENCE_KINDS = (
    "artifact_presence",
    "axis_binding",
    "byte_identity",
    "exposure_ledger",
    "format_header",
    "publisher_metadata",
)
QUALITY_DECISIONS = ("Fail", "Pass")
ACTIVE_REASON = "all_required_structure_present"
OOD_REASONS = {
    "artifact_incomplete",
    "missing_required_axis",
    "prior_exposure_conflict",
    "unsupported_tier",
}

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
TRAINING_REQUIRED_AXES = tuple(axis for axis in AXES if axis != "support_condition")
EVALUATION_REQUIRED_AXES = AXES

GENERATOR_EXPOSURE_ROLES = {
    "source_inventory",
    "estimator_fit",
    "generator_development",
}

HASH_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,191}\Z")

CONTRACT_KEYS = {
    "schema",
    "contract_id",
    "revision",
    "claim_kind",
    "parent_exposure_ledger",
    "role_policy",
    "axis_policy",
    "sources",
    "objects",
    "access_policy",
    "authority",
}
LEDGER_REF_KEYS = {"schema", "sha256", "role_root_sha256"}
ROLE_POLICY_KEYS = {
    "roles",
    "protected_roles",
    "minimum_per_material",
    "partition_unit",
    "object_group_disjoint",
    "recording_parent_disjoint",
    "source_revision_policy",
}
AXIS_POLICY_KEYS = {"vocabulary", "training_required", "evaluation_required"}
SOURCE_KEYS = {
    "source_id",
    "tier",
    "source_namespace",
    "publisher_id",
    "project_id",
    "revision_id",
    "source_revision_group_sha256",
    "provenance_sha256",
}
OBJECT_KEYS = {
    "object_group_sha256",
    "known_alias_object_group_sha256s",
    "physical_object_group_id",
    "source_id",
    "publisher_object_id",
    "material_family",
    "role",
    "quality_mask",
    "axes",
    "recording_parent_sha256s",
    "source_member_root_sha256",
    "prior_exposure",
}
QUALITY_MASK_KEYS = {"classification", "reason_code", "evidence"}
QUALITY_EVIDENCE_KEYS = {
    "kind",
    "decision",
    "sha256",
    "signal_values_decoded",
}
AXIS_KEYS = {"axis", "state", "evidence_sha256"}
PRIOR_EXPOSURE_KEYS = {
    "object_group_sha256",
    "queried_object_group_sha256s",
    "ledger_roles",
    "state",
}
ACCESS_POLICY_KEYS = {
    "network_allowed",
    "source_artifact_access_allowed",
    "signal_decode_allowed",
    "outputs_external",
}
AUTHORITY_KEYS = {
    "public_contract",
    "runtime_consumer_allowed",
    "fallback_required",
    "authorizes_only",
}

LEDGER_KEYS = {
    "schema",
    "revision",
    "catalog_sha256",
    "store_root_id",
    "partition_policy",
    "artifact_root_sha256",
    "role_root_sha256",
    "entries",
    "group_index",
    "partition_observations",
    "counters",
    "decision",
    "public_contract",
    "runtime_consumer_allowed",
}
LEDGER_ENTRY_KEYS = {
    "identity_sha256",
    "identity",
    "groups",
    "roles",
    "exposures",
}
LEDGER_GROUP_KEYS = {
    "object_parent",
    "contact_parent",
    "listener_parent",
    "mutation_parent",
}
LEDGER_EXPOSURE_KEYS = {
    "role",
    "access_kind",
    "values_decoded",
    "protected",
    "evidence_sha256",
}
LEDGER_COUNTER_KEYS = {
    "artifact_count",
    "exposure_count",
    "sample_identity_count",
    "historical_signal_values_decoded",
    "historical_protected_signal_values_decoded",
}
LEDGER_PARTITION_KEYS = {"cross_side_group_count", "cross_side_groups"}


class DatasetContractError(RuntimeError):
    """The dataset contract violates the frozen V14-N1a protocol."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=("fixture", "build"))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--contract", type=Path)
    parser.add_argument("--ledger", type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return ledger_v0.canonical_json(value)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def synthetic_hash(label: str) -> str:
    return sha256_bytes(f"physical-sound-v14-n1a:{label}".encode())


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise DatasetContractError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise DatasetContractError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_identifier(value: Any, context: str) -> str:
    if not isinstance(value, str) or not IDENTIFIER_PATTERN.fullmatch(value):
        raise DatasetContractError(f"{context} must be a bounded identifier")
    return value


def require_hash(value: Any, context: str) -> str:
    if not isinstance(value, str) or not HASH_PATTERN.fullmatch(value):
        raise DatasetContractError(f"{context} must be a lowercase SHA-256")
    return value


def require_bool(value: Any, context: str) -> bool:
    if type(value) is not bool:
        raise DatasetContractError(f"{context} must be boolean")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise DatasetContractError(f"{context} must be a non-negative integer")
    return value


def parse_canonical_document(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    if path.is_symlink():
        raise DatasetContractError(f"{context} cannot be a symlink")
    if not path.is_file():
        raise DatasetContractError(f"{context} is not a regular file")
    if path.stat().st_size > MAX_INPUT_BYTES:
        raise DatasetContractError(f"{context} exceeds frozen byte limit")
    data = path.read_bytes()
    try:
        value = ledger_v0.parse_json_bytes(data, context)
    except ledger_v0.ExposureLedgerError as error:
        raise DatasetContractError(str(error)) from error
    if canonical_json(value) != data:
        raise DatasetContractError(f"{context} is not canonical JSON")
    if not isinstance(value, dict):
        raise DatasetContractError(f"{context} must be an object")
    return data, value


def source_revision_identity(source: dict[str, Any]) -> dict[str, str]:
    return {
        "publisher_id": source["publisher_id"],
        "project_id": source["project_id"],
        "revision_id": source["revision_id"],
    }


def object_parent_identity(source: dict[str, Any], publisher_object_id: str) -> dict[str, str]:
    return {
        "source_namespace": source["source_namespace"],
        "project_id": source["project_id"],
        "object_id": publisher_object_id,
    }


def object_parent_hash(source: dict[str, Any], publisher_object_id: str) -> str:
    return sha256_bytes(canonical_json(object_parent_identity(source, publisher_object_id)))


def validate_ledger(ledger_bytes: bytes, value: Any) -> dict[str, Any]:
    ledger = require_exact_keys(value, LEDGER_KEYS, "ledger")
    if ledger["schema"] != ledger_v0.LEDGER_SCHEMA:
        raise DatasetContractError("unknown exposure-ledger schema")
    for field in ("catalog_sha256", "artifact_root_sha256", "role_root_sha256"):
        require_hash(ledger[field], f"ledger.{field}")
    require_identifier(ledger["revision"], "ledger.revision")
    require_identifier(ledger["store_root_id"], "ledger.store_root_id")
    if ledger["partition_policy"] not in ledger_v0.PARTITION_POLICIES:
        raise DatasetContractError("unknown ledger partition policy")
    if ledger["public_contract"] is not False or ledger["runtime_consumer_allowed"] is not False:
        raise DatasetContractError("ledger cannot carry public/runtime authority")
    if not isinstance(ledger["entries"], list):
        raise DatasetContractError("ledger.entries must be an array")

    entries: list[dict[str, Any]] = []
    identity_hashes: set[str] = set()
    for index, raw_entry in enumerate(ledger["entries"]):
        entry = require_exact_keys(raw_entry, LEDGER_ENTRY_KEYS, f"ledger.entries[{index}]")
        identity = require_exact_keys(
            entry["identity"], ledger_v0.IDENTITY_KEYS, f"ledger.entries[{index}].identity"
        )
        try:
            ledger_v0.validate_identity(identity, index)
        except ledger_v0.ExposureLedgerError as error:
            raise DatasetContractError(str(error)) from error
        identity_hash = require_hash(
            entry["identity_sha256"], f"ledger.entries[{index}].identity_sha256"
        )
        if identity_hash != sha256_bytes(canonical_json(identity)):
            raise DatasetContractError("ledger identity hash changed")
        if identity_hash in identity_hashes:
            raise DatasetContractError("duplicate ledger identity")
        identity_hashes.add(identity_hash)
        groups = require_exact_keys(
            entry["groups"], LEDGER_GROUP_KEYS, f"ledger.entries[{index}].groups"
        )
        for kind in sorted(LEDGER_GROUP_KEYS):
            expected = ledger_v0.group_hash(identity, kind)
            if groups[kind] != expected:
                raise DatasetContractError(f"ledger {kind} group hash changed")
        roles = entry["roles"]
        if (
            not isinstance(roles, list)
            or not roles
            or roles != sorted(set(roles))
            or any(role not in ledger_v0.ROLES for role in roles)
        ):
            raise DatasetContractError("ledger roles are invalid or not sorted")
        if not isinstance(entry["exposures"], list) or not entry["exposures"]:
            raise DatasetContractError("ledger entry exposures must be non-empty")
        exposure_roles: set[str] = set()
        for exposure_index, raw_exposure in enumerate(entry["exposures"]):
            exposure = require_exact_keys(
                raw_exposure,
                LEDGER_EXPOSURE_KEYS,
                f"ledger.entries[{index}].exposures[{exposure_index}]",
            )
            role = exposure["role"]
            if role not in ledger_v0.ROLES:
                raise DatasetContractError("unknown ledger exposure role")
            access_kind = exposure["access_kind"]
            if access_kind not in ledger_v0.ACCESS_KINDS:
                raise DatasetContractError("unknown ledger access kind")
            values = require_nonnegative_integer(
                exposure["values_decoded"],
                f"ledger.entries[{index}].exposures[{exposure_index}].values_decoded",
            )
            protected = require_bool(
                exposure["protected"],
                f"ledger.entries[{index}].exposures[{exposure_index}].protected",
            )
            if access_kind not in ledger_v0.DECODED_ACCESS_KINDS and values != 0:
                raise DatasetContractError("ledger metadata/hash exposure decoded values")
            if protected and role in ledger_v0.FIT_OR_DEVELOPMENT_ROLES:
                raise DatasetContractError("ledger protected fit/development exposure")
            require_hash(
                exposure["evidence_sha256"],
                f"ledger.entries[{index}].exposures[{exposure_index}].evidence_sha256",
            )
            exposure_roles.add(role)
        if entry["exposures"] != sorted(
            entry["exposures"], key=lambda item: canonical_json(item)
        ):
            raise DatasetContractError("ledger entry exposures are not sorted")
        if set(roles) != exposure_roles:
            raise DatasetContractError("ledger entry roles do not match exposures")
        entries.append(entry)
    if entries != sorted(entries, key=lambda item: item["identity_sha256"]):
        raise DatasetContractError("ledger entries are not sorted")
    if ledger["role_root_sha256"] != ledger_v0.role_root(entries):
        raise DatasetContractError("ledger role root changed")
    if ledger["group_index"] != ledger_v0.group_index(entries):
        raise DatasetContractError("ledger group index changed")
    partition = require_exact_keys(
        ledger["partition_observations"],
        LEDGER_PARTITION_KEYS,
        "ledger.partition_observations",
    )
    reconstructed = [
        {"identity": entry["identity"], "role": exposure["role"]}
        for entry in entries
        for exposure in entry["exposures"]
    ]
    overlaps = ledger_v0.partition_overlaps(reconstructed)
    if partition["cross_side_group_count"] != len(overlaps):
        raise DatasetContractError("ledger partition overlap count changed")
    if partition["cross_side_groups"] != overlaps:
        raise DatasetContractError("ledger partition overlap rows changed")
    counters = require_exact_keys(
        ledger["counters"], LEDGER_COUNTER_KEYS, "ledger.counters"
    )
    for key in LEDGER_COUNTER_KEYS:
        require_nonnegative_integer(counters[key], f"ledger.counters.{key}")
    exposures = [exposure for entry in entries for exposure in entry["exposures"]]
    historical_signal = sum(
        exposure["values_decoded"]
        for exposure in exposures
        if exposure["access_kind"] in ledger_v0.DECODED_ACCESS_KINDS
    )
    historical_protected = sum(
        exposure["values_decoded"]
        for exposure in exposures
        if exposure["access_kind"] in ledger_v0.DECODED_ACCESS_KINDS
        and exposure["protected"]
    )
    expected_counters = {
        "exposure_count": len(exposures),
        "sample_identity_count": len(entries),
        "historical_signal_values_decoded": historical_signal,
        "historical_protected_signal_values_decoded": historical_protected,
    }
    for key, expected in expected_counters.items():
        if counters[key] != expected:
            raise DatasetContractError(f"ledger counter changed: {key}")
    require_identifier(ledger["decision"], "ledger.decision")
    del ledger_bytes
    return ledger


def ledger_roles_by_object(ledger: dict[str, Any]) -> dict[str, list[str]]:
    roles: dict[str, set[str]] = {}
    for entry in ledger["entries"]:
        digest = entry["groups"]["object_parent"]
        roles.setdefault(digest, set()).update(entry["roles"])
    return {digest: sorted(values) for digest, values in roles.items()}


def exposure_state(roles: list[str]) -> str:
    if not roles:
        return "unexposed"
    if set(roles).issubset(GENERATOR_EXPOSURE_ROLES):
        return "generator_exposed"
    return "protected_or_unknown_exposed"


def validate_role_policy(value: Any) -> dict[str, Any]:
    policy = require_exact_keys(value, ROLE_POLICY_KEYS, "role_policy")
    if policy["roles"] != list(ALL_ROLES):
        raise DatasetContractError("role policy vocabulary changed")
    if policy["protected_roles"] != list(PROTECTED_ROLES):
        raise DatasetContractError("protected role policy changed")
    expected_minimum = {material: MINIMUM_PER_MATERIAL for material in MATERIALS}
    if policy["minimum_per_material"] != expected_minimum:
        raise DatasetContractError("minimum per-material role shape changed")
    if policy["partition_unit"] != "physical_object_group_id":
        raise DatasetContractError("partition unit changed")
    for key in ("object_group_disjoint", "recording_parent_disjoint"):
        if policy[key] is not True:
            raise DatasetContractError(f"{key} must remain true")
    if policy["source_revision_policy"] != "record_cluster_for_statistics":
        raise DatasetContractError("source revision policy changed")
    return policy


def validate_axis_policy(value: Any) -> dict[str, Any]:
    policy = require_exact_keys(value, AXIS_POLICY_KEYS, "axis_policy")
    if policy["vocabulary"] != list(AXES):
        raise DatasetContractError("axis vocabulary changed")
    if policy["training_required"] != list(TRAINING_REQUIRED_AXES):
        raise DatasetContractError("training-required axes changed")
    if policy["evaluation_required"] != list(EVALUATION_REQUIRED_AXES):
        raise DatasetContractError("evaluation-required axes changed")
    return policy


def validate_source(value: Any, index: int) -> dict[str, Any]:
    source = require_exact_keys(value, SOURCE_KEYS, f"sources[{index}]")
    for field in (
        "source_id",
        "source_namespace",
        "publisher_id",
        "project_id",
        "revision_id",
    ):
        require_identifier(source[field], f"sources[{index}].{field}")
    if source["tier"] not in TIERS:
        raise DatasetContractError("unknown source tier")
    require_hash(source["provenance_sha256"], f"sources[{index}].provenance_sha256")
    expected_group = sha256_bytes(canonical_json(source_revision_identity(source)))
    if source["source_revision_group_sha256"] != expected_group:
        raise DatasetContractError("source revision group hash changed")
    return source


def validate_quality_mask(value: Any, context: str) -> dict[str, Any]:
    mask = require_exact_keys(value, QUALITY_MASK_KEYS, context)
    classification = mask["classification"]
    if classification not in QUALITY_CLASSES:
        raise DatasetContractError("unknown quality class")
    reason = mask["reason_code"]
    require_identifier(reason, f"{context}.reason_code")
    if classification == "source_ood":
        if reason not in OOD_REASONS:
            raise DatasetContractError("source OOD reason is not structural")
    elif reason != ACTIVE_REASON:
        raise DatasetContractError("active quality mask reason changed")
    evidence = mask["evidence"]
    if not isinstance(evidence, list) or not evidence:
        raise DatasetContractError("quality evidence must be a non-empty array")
    previous_key: tuple[str, str] | None = None
    decisions: list[str] = []
    for index, raw_row in enumerate(evidence):
        row = require_exact_keys(raw_row, QUALITY_EVIDENCE_KEYS, f"{context}.evidence[{index}]")
        if row["kind"] not in QUALITY_EVIDENCE_KINDS:
            raise DatasetContractError("unknown quality evidence kind")
        if row["decision"] not in QUALITY_DECISIONS:
            raise DatasetContractError("unknown quality evidence decision")
        require_hash(row["sha256"], f"{context}.evidence[{index}].sha256")
        if require_nonnegative_integer(
            row["signal_values_decoded"],
            f"{context}.evidence[{index}].signal_values_decoded",
        ) != 0:
            raise DatasetContractError("quality evidence decoded signal values")
        key = (row["kind"], row["sha256"])
        if previous_key is not None and key <= previous_key:
            raise DatasetContractError("quality evidence is duplicate or not sorted")
        previous_key = key
        decisions.append(row["decision"])
    if classification == "source_ood" and "Fail" not in decisions:
        raise DatasetContractError("source OOD requires failed structural evidence")
    if classification != "source_ood" and any(item != "Pass" for item in decisions):
        raise DatasetContractError("active quality mask contains failed evidence")
    return mask


def validate_axes(value: Any, context: str) -> dict[str, str]:
    if not isinstance(value, list) or len(value) != len(AXES):
        raise DatasetContractError("object axes must contain the complete vocabulary")
    states: dict[str, str] = {}
    for index, raw_axis in enumerate(value):
        axis = require_exact_keys(raw_axis, AXIS_KEYS, f"{context}[{index}]")
        if axis["axis"] != AXES[index]:
            raise DatasetContractError("object axes are missing, unknown or not sorted")
        if axis["state"] not in {"absent", "known"}:
            raise DatasetContractError("unknown axis state")
        evidence = axis["evidence_sha256"]
        if axis["state"] == "known":
            require_hash(evidence, f"{context}[{index}].evidence_sha256")
        elif evidence is not None:
            raise DatasetContractError("absent axis must have null evidence")
        states[axis["axis"]] = axis["state"]
    return states


def validate_prior_exposure(
    value: Any,
    object_group_sha256: str,
    queried_object_group_sha256s: list[str],
    observed_roles: list[str],
    context: str,
) -> str:
    prior = require_exact_keys(value, PRIOR_EXPOSURE_KEYS, context)
    if prior["object_group_sha256"] != object_group_sha256:
        raise DatasetContractError("prior exposure object group hash changed")
    if prior["queried_object_group_sha256s"] != queried_object_group_sha256s:
        raise DatasetContractError("prior exposure alias query set changed")
    if prior["ledger_roles"] != observed_roles:
        raise DatasetContractError("prior exposure ledger roles changed")
    expected_state = exposure_state(observed_roles)
    if prior["state"] != expected_state:
        raise DatasetContractError("prior exposure state changed")
    return expected_state


def validate_object(
    value: Any,
    index: int,
    sources: dict[str, dict[str, Any]],
    ledger_roles: dict[str, list[str]],
) -> dict[str, Any]:
    context = f"objects[{index}]"
    item = require_exact_keys(value, OBJECT_KEYS, context)
    source_id = require_identifier(item["source_id"], f"{context}.source_id")
    if source_id not in sources:
        raise DatasetContractError("object references unknown source")
    source = sources[source_id]
    publisher_object_id = require_identifier(
        item["publisher_object_id"], f"{context}.publisher_object_id"
    )
    expected_group = object_parent_hash(source, publisher_object_id)
    if item["object_group_sha256"] != expected_group:
        raise DatasetContractError("object group hash changed")
    aliases = item["known_alias_object_group_sha256s"]
    if not isinstance(aliases, list) or aliases != sorted(set(aliases)):
        raise DatasetContractError("object alias group hashes are duplicate or not sorted")
    for alias_index, alias in enumerate(aliases):
        require_hash(alias, f"{context}.known_alias_object_group_sha256s[{alias_index}]")
    if expected_group not in aliases:
        raise DatasetContractError("object alias group hashes omit the current object group")
    require_identifier(
        item["physical_object_group_id"], f"{context}.physical_object_group_id"
    )
    if item["material_family"] not in MATERIALS:
        raise DatasetContractError("unknown material family")
    role = item["role"]
    if role not in ALL_ROLES:
        raise DatasetContractError("unknown dataset role")
    quality = validate_quality_mask(item["quality_mask"], f"{context}.quality_mask")
    states = validate_axes(item["axes"], f"{context}.axes")
    parents = item["recording_parent_sha256s"]
    if not isinstance(parents, list) or parents != sorted(set(parents)):
        raise DatasetContractError("recording parents are duplicate or not sorted")
    for parent_index, parent in enumerate(parents):
        require_hash(parent, f"{context}.recording_parent_sha256s[{parent_index}]")
    require_hash(item["source_member_root_sha256"], f"{context}.source_member_root_sha256")
    observed_roles = sorted(
        {
            role
            for alias in aliases
            for role in ledger_roles.get(alias, [])
        }
    )
    prior_state = validate_prior_exposure(
        item["prior_exposure"],
        expected_group,
        aliases,
        observed_roles,
        f"{context}.prior_exposure",
    )

    classification = quality["classification"]
    if role == "excluded":
        if classification != "source_ood":
            raise DatasetContractError("excluded object must be source OOD")
    elif classification == "source_ood":
        raise DatasetContractError("source OOD cannot be active data")
    if classification == "training_usable" and role not in GENERATOR_ROLES:
        raise DatasetContractError("training-usable object cannot receive evaluation role")
    if role in PROTECTED_ROLES and classification != "evaluation_complete":
        raise DatasetContractError("protected role requires evaluation-complete quality")
    required_axes = (
        EVALUATION_REQUIRED_AXES
        if classification == "evaluation_complete"
        else TRAINING_REQUIRED_AXES
    )
    if classification != "source_ood":
        missing = [axis for axis in required_axes if states[axis] != "known"]
        if missing:
            raise DatasetContractError(f"quality class has absent required axes: {missing}")
    if role != "excluded" and not parents:
        raise DatasetContractError("active role requires a recording parent")
    if role in PROTECTED_ROLES:
        if source["tier"] not in REAL_CONTACT_TIERS:
            raise DatasetContractError("protected role requires a real-contact tier")
        if prior_state != "unexposed":
            raise DatasetContractError("prior exposure cannot enter a protected role")
    elif role in GENERATOR_ROLES:
        if source["tier"] not in TRAINING_TIERS:
            raise DatasetContractError("generator role uses unsupported tier")
        if prior_state == "protected_or_unknown_exposed":
            raise DatasetContractError("protected or unknown exposure cannot enter generator data")
    elif role == "excluded":
        pass
    return item


def validate_access_policy(value: Any) -> None:
    policy = require_exact_keys(value, ACCESS_POLICY_KEYS, "access_policy")
    for key in (
        "network_allowed",
        "source_artifact_access_allowed",
        "signal_decode_allowed",
    ):
        if require_bool(policy[key], f"access_policy.{key}") is not False:
            raise DatasetContractError(f"access_policy.{key} must remain false")
    if require_bool(policy["outputs_external"], "access_policy.outputs_external") is not True:
        raise DatasetContractError("outputs must remain external")


def validate_authority(value: Any) -> None:
    authority = require_exact_keys(value, AUTHORITY_KEYS, "authority")
    if authority["public_contract"] is not False:
        raise DatasetContractError("dataset contract cannot be public authority")
    if authority["runtime_consumer_allowed"] is not False:
        raise DatasetContractError("dataset contract cannot authorize runtime")
    if authority["fallback_required"] is not True:
        raise DatasetContractError("authored fallback must remain required")
    if authority["authorizes_only"] != "N1b_metadata_only_source_inventory":
        raise DatasetContractError("N1a authority widened")


def validate_contract(
    contract_bytes: bytes,
    value: Any,
    ledger_bytes: bytes,
    ledger: dict[str, Any],
) -> dict[str, Any]:
    contract = require_exact_keys(value, CONTRACT_KEYS, "contract")
    if contract["schema"] != CONTRACT_SCHEMA:
        raise DatasetContractError("unknown dataset-contract schema")
    require_identifier(contract["contract_id"], "contract.contract_id")
    require_identifier(contract["revision"], "contract.revision")
    if contract["claim_kind"] != CLAIM_KIND:
        raise DatasetContractError("dataset claim kind widened or changed")
    parent = require_exact_keys(
        contract["parent_exposure_ledger"], LEDGER_REF_KEYS, "parent_exposure_ledger"
    )
    if parent["schema"] != ledger_v0.LEDGER_SCHEMA:
        raise DatasetContractError("parent ledger schema changed")
    if parent["sha256"] != sha256_bytes(ledger_bytes):
        raise DatasetContractError("parent ledger hash changed")
    if parent["role_root_sha256"] != ledger["role_root_sha256"]:
        raise DatasetContractError("parent ledger role root changed")
    validate_role_policy(contract["role_policy"])
    validate_axis_policy(contract["axis_policy"])
    validate_access_policy(contract["access_policy"])
    validate_authority(contract["authority"])

    raw_sources = contract["sources"]
    if not isinstance(raw_sources, list) or not raw_sources:
        raise DatasetContractError("sources must be a non-empty array")
    if len(raw_sources) > MAX_SOURCES:
        raise DatasetContractError("too many dataset sources")
    sources_list = [validate_source(source, index) for index, source in enumerate(raw_sources)]
    source_ids = [source["source_id"] for source in sources_list]
    if source_ids != sorted(set(source_ids)):
        raise DatasetContractError("sources are duplicate or not sorted")
    sources = {source["source_id"]: source for source in sources_list}

    raw_objects = contract["objects"]
    if not isinstance(raw_objects, list) or not raw_objects:
        raise DatasetContractError("objects must be a non-empty array")
    if len(raw_objects) > MAX_OBJECTS:
        raise DatasetContractError("too many dataset objects")
    observed = ledger_roles_by_object(ledger)
    objects = [
        validate_object(item, index, sources, observed)
        for index, item in enumerate(raw_objects)
    ]
    object_hashes = [item["object_group_sha256"] for item in objects]
    if object_hashes != sorted(set(object_hashes)):
        raise DatasetContractError("objects are duplicate or not sorted")

    physical_roles: dict[str, set[str]] = {}
    physical_materials: dict[str, set[str]] = {}
    recording_roles: dict[str, set[str]] = {}
    for item in objects:
        physical = item["physical_object_group_id"]
        physical_roles.setdefault(physical, set()).add(item["role"])
        physical_materials.setdefault(physical, set()).add(item["material_family"])
        for parent in item["recording_parent_sha256s"]:
            recording_roles.setdefault(parent, set()).add(item["role"])
    if any(len(roles) > 1 for roles in physical_roles.values()):
        raise DatasetContractError("physical object group crosses dataset roles")
    if any(len(materials) > 1 for materials in physical_materials.values()):
        raise DatasetContractError("physical object group changes material family")
    if any(len(roles) > 1 for roles in recording_roles.values()):
        raise DatasetContractError("recording parent crosses dataset roles")

    counts = {
        material: {role: set() for role in ACTIVE_ROLES} for material in MATERIALS
    }
    for item in objects:
        role = item["role"]
        if role in ACTIVE_ROLES:
            counts[item["material_family"]][role].add(item["physical_object_group_id"])
    for material in MATERIALS:
        for role, minimum in MINIMUM_PER_MATERIAL.items():
            actual = len(counts[material][role])
            if actual < minimum:
                raise DatasetContractError(
                    f"minimum role coverage not met: {material}/{role} {actual} < {minimum}"
                )
    del contract_bytes
    return contract


def policy_descriptor() -> dict[str, Any]:
    policy = {
        "claim_kind": CLAIM_KIND,
        "materials": list(MATERIALS),
        "roles": list(ALL_ROLES),
        "protected_roles": list(PROTECTED_ROLES),
        "tiers": list(TIERS),
        "quality_classes": list(QUALITY_CLASSES),
        "quality_evidence_kinds": list(QUALITY_EVIDENCE_KINDS),
        "axes": list(AXES),
        "training_required_axes": list(TRAINING_REQUIRED_AXES),
        "evaluation_required_axes": list(EVALUATION_REQUIRED_AXES),
        "minimum_per_material": {
            material: MINIMUM_PER_MATERIAL for material in MATERIALS
        },
        "partition_unit": "physical_object_group_id",
        "source_revision_policy": "record_cluster_for_statistics",
    }
    return {
        "schema": DESCRIPTOR_SCHEMA,
        "contract_schema": CONTRACT_SCHEMA,
        "policy": policy,
        "policy_sha256": sha256_bytes(canonical_json(policy)),
        "public_contract": False,
        "runtime_consumer_allowed": False,
    }


def report_documents(
    contract_bytes: bytes,
    contract: dict[str, Any],
    ledger_bytes: bytes,
    ledger: dict[str, Any],
    access: dict[str, int],
    decision: str,
) -> dict[str, bytes]:
    descriptor = policy_descriptor()
    descriptor_bytes = canonical_json(descriptor)
    counts = {
        material: {role: 0 for role in ACTIVE_ROLES} for material in MATERIALS
    }
    physical_seen: dict[tuple[str, str], set[str]] = {}
    recording_parents: set[str] = set()
    clusters: dict[str, dict[str, Any]] = {}
    sources = {item["source_id"]: item for item in contract["sources"]}
    for item in contract["objects"]:
        recording_parents.update(item["recording_parent_sha256s"])
        role = item["role"]
        if role in ACTIVE_ROLES:
            key = (item["material_family"], role)
            physical_seen.setdefault(key, set()).add(item["physical_object_group_id"])
        source = sources[item["source_id"]]
        group = source["source_revision_group_sha256"]
        cluster = clusters.setdefault(
            group,
            {
                "source_revision_group_sha256": group,
                "source_ids": set(),
                "materials": set(),
                "roles": set(),
                "physical_object_group_ids": set(),
            },
        )
        cluster["source_ids"].add(item["source_id"])
        cluster["materials"].add(item["material_family"])
        cluster["roles"].add(role)
        cluster["physical_object_group_ids"].add(item["physical_object_group_id"])
    for material in MATERIALS:
        for role in ACTIVE_ROLES:
            counts[material][role] = len(physical_seen.get((material, role), set()))
    cluster_rows = []
    for group in sorted(clusters):
        item = clusters[group]
        cluster_rows.append(
            {
                "source_revision_group_sha256": group,
                "source_ids": sorted(item["source_ids"]),
                "materials": sorted(item["materials"]),
                "roles": sorted(item["roles"]),
                "physical_object_group_count": len(item["physical_object_group_ids"]),
            }
        )
    report = {
        "schema": REPORT_SCHEMA,
        "decision": decision,
        "hashes": {
            "contract_sha256": sha256_bytes(contract_bytes),
            "descriptor_sha256": sha256_bytes(descriptor_bytes),
            "ledger_sha256": sha256_bytes(ledger_bytes),
            "ledger_role_root_sha256": ledger["role_root_sha256"],
        },
        "counts": {
            "source_count": len(contract["sources"]),
            "object_row_count": len(contract["objects"]),
            "physical_object_group_count": len(
                {item["physical_object_group_id"] for item in contract["objects"]}
            ),
            "recording_parent_count": len(recording_parents),
            "source_revision_cluster_count": len(cluster_rows),
            "eligible_role_counts": counts,
        },
        "source_revision_clusters": cluster_rows,
        "build_access": access,
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "fallback_required": True,
        "opens_only": "N1b_metadata_only_source_inventory",
    }
    return {
        "contract.json": contract_bytes,
        "descriptor.json": descriptor_bytes,
        "report.json": canonical_json(report),
    }


def prepare_output(output: Path) -> Path:
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise DatasetContractError("output must remain outside the repository")
    if resolved.exists():
        raise DatasetContractError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish_directory(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent))
    try:
        for name, data in sorted(files.items()):
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def build_contract(
    contract_path: Path,
    ledger_path: Path,
    output: Path,
    *,
    decision: str = "N1A_DATASET_CONTRACT_V1_BUILD_PASS",
) -> Path:
    contract_bytes, raw_contract = parse_canonical_document(contract_path, "contract")
    ledger_bytes, raw_ledger = parse_canonical_document(ledger_path, "ledger")
    ledger = validate_ledger(ledger_bytes, raw_ledger)
    contract = validate_contract(
        contract_bytes, raw_contract, ledger_bytes, ledger
    )
    access = {
        "contract_bytes_read": len(contract_bytes),
        "ledger_bytes_read": len(ledger_bytes),
        "metadata_scalar_values_parsed": (
            ledger_v0.scalar_count(raw_contract) + ledger_v0.scalar_count(raw_ledger)
        ),
        "network_requests": 0,
        "source_artifact_bytes_read": 0,
        "pcm_sample_values_decoded": 0,
        "force_sample_values_decoded": 0,
        "protected_signal_values_decoded": 0,
    }
    files = report_documents(
        contract_bytes,
        contract,
        ledger_bytes,
        ledger,
        access,
        decision,
    )
    return publish_directory(output, files)


def synthetic_ledger() -> dict[str, Any]:
    identity = {
        "source_namespace": "synthetic-glass-source",
        "project_id": "synthetic-glass-project",
        "object_id": "glass-00",
        "contact_id": None,
        "listener_id": None,
        "impact_id": None,
        "mutation_parent_id": None,
        "payload_kind": "metadata",
    }
    entry = {
        "identity_sha256": sha256_bytes(canonical_json(identity)),
        "identity": identity,
        "groups": {
            kind: ledger_v0.group_hash(identity, kind)
            for kind in sorted(LEDGER_GROUP_KEYS)
        },
        "roles": ["estimator_fit"],
        "exposures": [
            {
                "role": "estimator_fit",
                "access_kind": "metadata_only",
                "values_decoded": 0,
                "protected": False,
                "evidence_sha256": synthetic_hash("ledger-evidence"),
            }
        ],
    }
    entries = [entry]
    return {
        "schema": ledger_v0.LEDGER_SCHEMA,
        "revision": "synthetic-v14-n1a-ledger-v0",
        "catalog_sha256": synthetic_hash("ledger-catalog"),
        "store_root_id": "synthetic-external-store",
        "partition_policy": "historical_union",
        "artifact_root_sha256": synthetic_hash("ledger-artifacts"),
        "role_root_sha256": ledger_v0.role_root(entries),
        "entries": entries,
        "group_index": ledger_v0.group_index(entries),
        "partition_observations": {
            "cross_side_group_count": 0,
            "cross_side_groups": [],
        },
        "counters": {
            "artifact_count": 1,
            "exposure_count": 1,
            "sample_identity_count": 1,
            "historical_signal_values_decoded": 0,
            "historical_protected_signal_values_decoded": 0,
        },
        "decision": "EXPOSURE_LEDGER_V0_BUILT",
        "public_contract": False,
        "runtime_consumer_allowed": False,
    }


def quality_evidence(label: str, *, failed: bool = False) -> list[dict[str, Any]]:
    rows = []
    for kind in QUALITY_EVIDENCE_KINDS:
        rows.append(
            {
                "kind": kind,
                "decision": "Fail" if failed and kind == "artifact_presence" else "Pass",
                "sha256": synthetic_hash(f"quality:{label}:{kind}"),
                "signal_values_decoded": 0,
            }
        )
    return rows


def known_axes(label: str) -> list[dict[str, Any]]:
    return [
        {
            "axis": axis,
            "state": "known",
            "evidence_sha256": synthetic_hash(f"axis:{label}:{axis}"),
        }
        for axis in AXES
    ]


def source_for_material(material: str) -> dict[str, Any]:
    lower = material.lower()
    source = {
        "source_id": f"synthetic-{lower}-source",
        "tier": "T2_sparse_real_contact",
        "source_namespace": f"synthetic-{lower}-source",
        "publisher_id": "nextengine-synthetic",
        "project_id": f"synthetic-{lower}-project",
        "revision_id": "v1",
        "source_revision_group_sha256": "",
        "provenance_sha256": synthetic_hash(f"source:{lower}:provenance"),
    }
    source["source_revision_group_sha256"] = sha256_bytes(
        canonical_json(source_revision_identity(source))
    )
    return source


def synthetic_contract(ledger_bytes: bytes, ledger: dict[str, Any]) -> dict[str, Any]:
    sources = [source_for_material(material) for material in MATERIALS]
    source_map = {source["source_id"]: source for source in sources}
    role_sequence = (
        "generator_train",
        "generator_train",
        "generator_train",
        "generator_train",
        "generator_development",
        "validator_calibration",
        "validator_method_holdout",
        "admission_shadow",
    )
    observed = ledger_roles_by_object(ledger)
    objects = []
    for material in MATERIALS:
        lower = material.lower()
        source = source_map[f"synthetic-{lower}-source"]
        for index, role in enumerate(role_sequence):
            publisher_object_id = f"{lower}-{index:02d}"
            object_group = object_parent_hash(source, publisher_object_id)
            roles = observed.get(object_group, [])
            classification = (
                "training_usable"
                if role == "generator_train" and index == 1
                else "evaluation_complete"
            )
            axes = known_axes(f"{lower}:{index:02d}")
            if classification == "training_usable":
                axes[-1] = {
                    "axis": "support_condition",
                    "state": "absent",
                    "evidence_sha256": None,
                }
            objects.append(
                {
                    "object_group_sha256": object_group,
                    "known_alias_object_group_sha256s": [object_group],
                    "physical_object_group_id": f"synthetic:{lower}:{index:02d}",
                    "source_id": source["source_id"],
                    "publisher_object_id": publisher_object_id,
                    "material_family": material,
                    "role": role,
                    "quality_mask": {
                        "classification": classification,
                        "reason_code": ACTIVE_REASON,
                        "evidence": quality_evidence(f"{lower}:{index:02d}"),
                    },
                    "axes": axes,
                    "recording_parent_sha256s": [
                        synthetic_hash(f"recording:{lower}:{index:02d}")
                    ],
                    "source_member_root_sha256": synthetic_hash(
                        f"members:{lower}:{index:02d}"
                    ),
                    "prior_exposure": {
                        "object_group_sha256": object_group,
                        "queried_object_group_sha256s": [object_group],
                        "ledger_roles": roles,
                        "state": exposure_state(roles),
                    },
                }
            )
    return {
        "schema": CONTRACT_SCHEMA,
        "contract_id": "physical-sound-v14-n1a-synthetic",
        "revision": "dataset-contract-v1-fixture",
        "claim_kind": CLAIM_KIND,
        "parent_exposure_ledger": {
            "schema": ledger_v0.LEDGER_SCHEMA,
            "sha256": sha256_bytes(ledger_bytes),
            "role_root_sha256": ledger["role_root_sha256"],
        },
        "role_policy": {
            "roles": list(ALL_ROLES),
            "protected_roles": list(PROTECTED_ROLES),
            "minimum_per_material": {
                material: MINIMUM_PER_MATERIAL for material in MATERIALS
            },
            "partition_unit": "physical_object_group_id",
            "object_group_disjoint": True,
            "recording_parent_disjoint": True,
            "source_revision_policy": "record_cluster_for_statistics",
        },
        "axis_policy": {
            "vocabulary": list(AXES),
            "training_required": list(TRAINING_REQUIRED_AXES),
            "evaluation_required": list(EVALUATION_REQUIRED_AXES),
        },
        "sources": sorted(sources, key=lambda item: item["source_id"]),
        "objects": sorted(objects, key=lambda item: item["object_group_sha256"]),
        "access_policy": {
            "network_allowed": False,
            "source_artifact_access_allowed": False,
            "signal_decode_allowed": False,
            "outputs_external": True,
        },
        "authority": {
            "public_contract": False,
            "runtime_consumer_allowed": False,
            "fallback_required": True,
            "authorizes_only": "N1b_metadata_only_source_inventory",
        },
    }


def write_synthetic_inputs(root: Path) -> tuple[Path, Path]:
    root.mkdir(parents=True)
    ledger = synthetic_ledger()
    ledger_bytes = canonical_json(ledger)
    contract = synthetic_contract(ledger_bytes, ledger)
    ledger_path = root / "ledger.json"
    contract_path = root / "contract.json"
    ledger_path.write_bytes(ledger_bytes)
    contract_path.write_bytes(canonical_json(contract))
    return contract_path, ledger_path


def build_fixture(output: Path) -> Path:
    destination = output.resolve()
    prepare_output(destination)
    with tempfile.TemporaryDirectory(
        prefix="physical-sound-v14-n1a-input-", dir=destination.parent
    ) as temporary:
        contract, ledger = write_synthetic_inputs(Path(temporary) / "inputs")
        return build_contract(
            contract,
            ledger,
            destination,
            decision="N1A_DATASET_CONTRACT_V1_FIXTURE_PASS",
        )


def main() -> None:
    arguments = parse_arguments()
    if arguments.stage == "fixture":
        if arguments.contract is not None or arguments.ledger is not None:
            raise DatasetContractError("fixture stage does not accept input paths")
        destination = build_fixture(arguments.output)
    else:
        if arguments.contract is None or arguments.ledger is None:
            raise DatasetContractError("build stage requires --contract and --ledger")
        destination = build_contract(arguments.contract, arguments.ledger, arguments.output)
    print(destination)


if __name__ == "__main__":
    main()
