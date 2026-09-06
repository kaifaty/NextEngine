#!/usr/bin/env python3
"""Repair the V40 disclosed roster at physical-parent alias-component scope."""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_v40_d0_disclosed_roster_v1 as d0

PROFILE_PATH = "lab/profiles/physical-sound-v41-d1-alias-component-roster.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v41_d1_alias_component_roster_v1.py"

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v41-d1-profile.v1"
ROSTER_SCHEMA = "nextengine.experimental-physical-sound-v41-d1-disclosed-roster.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v41-d1-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v41-d1-report.v1"
CLAIM = (
    "DISCLOSED_PHYSICAL_PARENT_ALIAS_COMPONENT_ROLE_REPAIR_ONLY / "
    "NO_PAYLOAD_SIGNAL_FEATURE_MODEL_PROTECTED_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
DECISION = "D1_ALIAS_COMPONENT_ROSTER_REPAIRED_C0_AUTHORIZED"

MAX_PROFILE_BYTES = 1024 * 1024
MAX_ROSTER_BYTES = 1024 * 1024
HASH_LENGTH = 64
DISCLOSED_ROLES = d0.DISCLOSED_ROLES
PROTECTED_ROLES = d0.PROTECTED_ROLES

DEPENDENCY_PATHS = {
    "d0-owner": "lab/scripts/physical_sound_v40_d0_disclosed_roster_v1.py",
    "d0-profile": "lab/profiles/physical-sound-v40-d0-disclosed-roster.v1.json",
    "d0-result": (
        "docs/development/physical-sound-v40-d0-disclosed-roster-result-2026-09-03.md"
    ),
    "i0-result": (
        "docs/development/physical-sound-v40-i0-project-exposure-result-2026-09-03.md"
    ),
    "v24-x0-blue-bowl-result": (
        "docs/development/physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md"
    ),
    "v41-roadmap": "docs/plans/physical-sound-synthesis-roadmap-v41.md",
}

D0_BINDING = {
    "bytes": 10142,
    "roster_root_sha256": (
        "5038c54d60b610fedb52bfbf2bd6bc231327a99c67689dcc481651f1b1aaf7bd"
    ),
    "schema": d0.ROSTER_SCHEMA,
    "sha256": "6d6f29fa2e2292bb5f69acde401284e7fc5a012061f01db15ee1f1d722480de9",
}

ACCESS_POLICY = {
    "candidate_output_access_allowed": False,
    "feature_access_allowed": False,
    "model_access_allowed": False,
    "network_allowed": False,
    "outputs_external": True,
    "payload_access_allowed": False,
    "protected_access_allowed": False,
    "signal_decode_allowed": False,
}
ARTIFACT_POLICY = d0.ARTIFACT_POLICY
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": "v41_c0_disclosed_corpus",
    "disclosed_role_assignment_authority": True,
    "model_training_authority": False,
    "payload_access_authority": False,
    "product_authority": False,
    "protected_role_assignment_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_release_authority": False,
}
ROLE_POLICY = {
    "cross_family_parent_alias_behavior": "merge_component_before_role_projection",
    "descendants_inherit_component_role": True,
    "disclosed_roles": list(DISCLOSED_ROLES),
    "family_role_reassignment_after_d1": "forbidden",
    "partition_unit": "physical_parent_alias_component",
    "protected_roles": list(PROTECTED_ROLES),
    "unknown_alias_behavior": "reject",
    "whole_component_disjoint": True,
}
REPAIR_POLICY = {
    "expected_alias_component_count": 1,
    "expected_family_count": 9,
    "expected_original_cross_role_component_count": 1,
    "expected_reassigned_family_count": 2,
    "expected_role_counts": {
        "generator_development": 2,
        "generator_train": 5,
        "validator_calibration": 2,
    },
    "repair_rule": (
        "co_locate_realimpact_and_objectfolder_blue_bowl_in_generator_train_"
        "then_move_independent_av_msf_to_validator_calibration_v1"
    ),
}
ALIAS_COMPONENTS = [
    {
        "component_id": "physical-parent.realimpact-objectfolder-blue-bowl.v1",
        "evidence_binding_id": "v24-x0-blue-bowl-result",
        "family_ids": [
            "samuel-clarke--realimpact",
            "stanford-objectfolder--objectfolder-real",
        ],
        "physical_parent_id": "realimpact-6-bowl--objectfolder-real-object-6",
    }
]
EXPECTED_ASSIGNMENTS = [
    {
        "family_id": "carnegie-mellon-auditorylab--sound-events-impact-events",
        "from_role": "validator_calibration",
        "role": "validator_calibration",
    },
    {
        "family_id": "freesound-user-ascap--pack-14905",
        "from_role": "generator_development",
        "role": "generator_development",
    },
    {
        "family_id": "freesound-user-wasserbjorn--pack-41981",
        "from_role": "generator_train",
        "role": "generator_train",
    },
    {
        "family_id": "iri-csic-upc-ctu--ycb-impact-sounds",
        "from_role": "generator_development",
        "role": "generator_development",
    },
    {
        "family_id": "kaffekrus--soundpacks-glass-recordings",
        "from_role": "generator_train",
        "role": "generator_train",
    },
    {
        "family_id": "samuel-clarke--realimpact",
        "from_role": "generator_train",
        "role": "generator_train",
    },
    {
        "family_id": "stanford-objectfolder--objectfolder-real",
        "from_role": "validator_calibration",
        "role": "generator_train",
    },
    {
        "family_id": "the-language-of-sounds--kronland-material-impact-stimuli",
        "from_role": "generator_train",
        "role": "generator_train",
    },
    {
        "family_id": "zisen-shao--av-msf",
        "from_role": "generator_train",
        "role": "validator_calibration",
    },
]
EXPECTED_RESULT = {
    "alias_component_count": 1,
    "clean_project_families_spent": 0,
    "decision": DECISION,
    "disclosed_family_count": 9,
    "generator_development_families": 2,
    "generator_train_families": 5,
    "original_cross_role_component_count": 1,
    "protected_family_count": 0,
    "reassigned_family_count": 2,
    "repaired_cross_role_component_count": 0,
    "validator_calibration_families": 2,
}

FORBIDDEN_COUNTERS = d0.FORBIDDEN_COUNTERS
PROFILE_KEYS = {
    "access_policy",
    "alias_components",
    "artifact_policy",
    "assignments",
    "authority",
    "claim",
    "d0_binding",
    "dependency_bindings",
    "expected_result",
    "repair_policy",
    "role_policy",
    "schema",
}
DEPENDENCY_KEYS = {"binding_id", "bytes", "path", "sha256"}
ALIAS_KEYS = {
    "component_id",
    "evidence_binding_id",
    "family_ids",
    "physical_parent_id",
}
ASSIGNMENT_KEYS = {"family_id", "from_role", "role"}


class AliasRosterError(RuntimeError):
    """The disclosed roster cannot be repaired without signal access."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--d0-roster", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return d0.canonical_json(value)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise AliasRosterError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise AliasRosterError(
            f"{context} fields changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise AliasRosterError(f"{context} must be a lowercase SHA-256")
    return value


def read_canonical_json(
    path: Path, role: str, maximum: int
) -> tuple[bytes, dict[str, Any]]:
    if path.is_symlink() or not path.is_file():
        raise AliasRosterError(f"{role} must be a regular non-symlink file")
    size = path.stat().st_size
    if not 0 < size <= maximum:
        raise AliasRosterError(f"{role} size is outside the frozen limit")
    data = path.read_bytes()
    try:
        value = d0.i0.q1a.parse_json(data, role)
    except d0.i0.q1a.Q1ASourceGrowthError as error:
        raise AliasRosterError(str(error)) from error
    if canonical_json(value) != data:
        raise AliasRosterError(f"{role} must use canonical JSON")
    return data, value


def bound_repository_file(path_text: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink() or not path.is_file():
        raise AliasRosterError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    return {"bytes": len(data), "path": path_text, "sha256": sha256_bytes(data)}


def validate_dependencies(value: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(value, list) or len(value) != len(DEPENDENCY_PATHS):
        raise AliasRosterError("dependency bindings must cover the frozen roster")
    checked: dict[str, dict[str, Any]] = {}
    order = []
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, DEPENDENCY_KEYS, f"dependencies[{index}]")
        binding_id = item["binding_id"]
        if not isinstance(binding_id, str) or binding_id not in DEPENDENCY_PATHS:
            raise AliasRosterError("unknown dependency binding")
        if item["path"] != DEPENDENCY_PATHS[binding_id]:
            raise AliasRosterError("dependency path substitution")
        if type(item["bytes"]) is not int or item["bytes"] < 0:
            raise AliasRosterError("dependency bytes must be non-negative")
        require_hash(item["sha256"], "dependency sha256")
        if {
            key: item[key] for key in ("bytes", "path", "sha256")
        } != bound_repository_file(item["path"]):
            raise AliasRosterError(f"dependency drift: {item['path']}")
        if binding_id in checked:
            raise AliasRosterError("duplicate dependency binding")
        checked[binding_id] = item
        order.append(binding_id)
    if order != sorted(DEPENDENCY_PATHS):
        raise AliasRosterError("dependency order changed")
    return checked


def validate_aliases(value: Any, dependencies: dict[str, dict[str, Any]]) -> None:
    if not isinstance(value, list):
        raise AliasRosterError("alias components must be an array")
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, ALIAS_KEYS, f"alias_components[{index}]")
        if item["evidence_binding_id"] not in dependencies:
            raise AliasRosterError("alias evidence binding is absent")
        if (
            not isinstance(item["family_ids"], list)
            or item["family_ids"] != sorted(set(item["family_ids"]))
            or len(item["family_ids"]) < 2
        ):
            raise AliasRosterError("alias family IDs must be sorted unique components")
    if value != ALIAS_COMPONENTS:
        raise AliasRosterError("frozen alias components changed")


def validate_assignments(value: Any) -> None:
    if not isinstance(value, list):
        raise AliasRosterError("assignments must be an array")
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, ASSIGNMENT_KEYS, f"assignments[{index}]")
        if (
            item["from_role"] not in DISCLOSED_ROLES
            or item["role"] not in DISCLOSED_ROLES
        ):
            raise AliasRosterError("assignment uses an unknown disclosed role")
    if value != EXPECTED_ASSIGNMENTS:
        raise AliasRosterError("frozen D1 assignments changed")


def validate_profile(value: Any) -> tuple[dict[str, Any], dict[str, dict[str, Any]]]:
    profile = require_exact_keys(value, PROFILE_KEYS, "profile")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM:
        raise AliasRosterError("unknown V41 D1 profile identity")
    for key, expected in (
        ("access_policy", ACCESS_POLICY),
        ("artifact_policy", ARTIFACT_POLICY),
        ("authority", AUTHORITY),
        ("d0_binding", D0_BINDING),
        ("expected_result", EXPECTED_RESULT),
        ("repair_policy", REPAIR_POLICY),
        ("role_policy", ROLE_POLICY),
    ):
        if profile[key] != expected:
            raise AliasRosterError(f"profile {key} changed")
    dependencies = validate_dependencies(profile["dependency_bindings"])
    validate_aliases(profile["alias_components"], dependencies)
    validate_assignments(profile["assignments"])
    return profile, dependencies


def load_d0_roster(path: Path) -> tuple[bytes, dict[str, Any]]:
    data, roster = read_canonical_json(path, "V40 D0 roster", MAX_ROSTER_BYTES)
    if len(data) != D0_BINDING["bytes"] or sha256_bytes(data) != D0_BINDING["sha256"]:
        raise AliasRosterError("V40 D0 roster bytes or SHA-256 changed")
    if roster.get("schema") != D0_BINDING["schema"]:
        raise AliasRosterError("V40 D0 roster schema changed")
    if roster.get("roster_root_sha256") != D0_BINDING["roster_root_sha256"]:
        raise AliasRosterError("V40 D0 roster root changed")
    if roster.get("authority") != d0.AUTHORITY:
        raise AliasRosterError("V40 D0 authority changed")
    families = roster.get("families")
    if not isinstance(families, list) or len(families) != 9:
        raise AliasRosterError("V40 D0 family roster changed")
    if [row.get("family_id") for row in families] != sorted(
        row.get("family_id") for row in families
    ):
        raise AliasRosterError("V40 D0 families are not canonical")
    return data, roster


def family_component_map(family_ids: set[str]) -> dict[str, str]:
    result = {family_id: f"family:{family_id}" for family_id in family_ids}
    seen: set[str] = set()
    for component in ALIAS_COMPONENTS:
        for family_id in component["family_ids"]:
            if family_id not in family_ids:
                raise AliasRosterError("alias references an unknown D0 family")
            if family_id in seen:
                raise AliasRosterError("family belongs to multiple alias components")
            seen.add(family_id)
            result[family_id] = component["component_id"]
    return result


def role_by_component(
    roles: dict[str, str], components: dict[str, str]
) -> dict[str, set[str]]:
    result: dict[str, set[str]] = {}
    for family_id, role in roles.items():
        result.setdefault(components[family_id], set()).add(role)
    return result


def commitment_projection(row: dict[str, Any]) -> dict[str, Any]:
    return {
        "candidate_project_revision_id": row["candidate_project_revision_id"],
        "evidence_ids": row["evidence_ids"],
        "family_component_id": row["family_component_id"],
        "family_id": row["family_id"],
        "historical_access_kind": row["historical_access_kind"],
        "role": row["role"],
    }


def repair_roster(d0_roster: dict[str, Any]) -> dict[str, Any]:
    original_rows = d0_roster["families"]
    original = {row["family_id"]: row for row in original_rows}
    assignments = {row["family_id"]: row for row in EXPECTED_ASSIGNMENTS}
    if len(original) != 9 or set(original) != set(assignments):
        raise AliasRosterError("D1 assignments do not cover the exact D0 families")
    for family_id, assignment in assignments.items():
        if original[family_id]["role"] != assignment["from_role"]:
            raise AliasRosterError("D0 source role does not match D1 precondition")

    components = family_component_map(set(original))
    original_roles = {family_id: row["role"] for family_id, row in original.items()}
    original_cross = {
        component_id: roles
        for component_id, roles in role_by_component(original_roles, components).items()
        if len(roles) > 1
    }
    if (
        len(original_cross)
        != REPAIR_POLICY["expected_original_cross_role_component_count"]
    ):
        raise AliasRosterError("unexpected original cross-role alias component count")
    if set(original_cross) != {ALIAS_COMPONENTS[0]["component_id"]}:
        raise AliasRosterError("unexpected original cross-role alias component")

    rows = []
    for family_id in sorted(original):
        source = original[family_id]
        assignment = assignments[family_id]
        row = {
            "candidate_project_revision_id": source["candidate_project_revision_id"],
            "current_run_signal_access": "none",
            "evidence_ids": source["evidence_ids"],
            "family_component_id": components[family_id],
            "family_id": family_id,
            "historical_access_kind": source["historical_access_kind"],
            "observed_unit": source["observed_unit"],
            "observed_units_lower_bound": source["observed_units_lower_bound"],
            "permanent_disclosed": True,
            "protected": False,
            "role": assignment["role"],
            "source_d0_family_commitment_sha256": source["family_commitment_sha256"],
        }
        row["family_commitment_sha256"] = sha256_bytes(
            canonical_json(commitment_projection(row))
        )
        rows.append(row)

    repaired_roles = {row["family_id"]: row["role"] for row in rows}
    repaired_cross = {
        component_id: roles
        for component_id, roles in role_by_component(repaired_roles, components).items()
        if len(roles) > 1
    }
    role_counts = {
        role: sum(row["role"] == role for row in rows) for role in DISCLOSED_ROLES
    }
    reassigned = sum(
        assignment["from_role"] != assignment["role"]
        for assignment in EXPECTED_ASSIGNMENTS
    )
    result = {
        "alias_component_count": len(ALIAS_COMPONENTS),
        "clean_project_families_spent": 0,
        "decision": DECISION,
        "disclosed_family_count": len(rows),
        "generator_development_families": role_counts["generator_development"],
        "generator_train_families": role_counts["generator_train"],
        "original_cross_role_component_count": len(original_cross),
        "protected_family_count": 0,
        "reassigned_family_count": reassigned,
        "repaired_cross_role_component_count": len(repaired_cross),
        "validator_calibration_families": role_counts["validator_calibration"],
    }
    if (
        role_counts != REPAIR_POLICY["expected_role_counts"]
        or result != EXPECTED_RESULT
    ):
        raise AliasRosterError(f"D1 result drift: {result}")

    component_roles = []
    for component_id in sorted(set(components.values())):
        family_rows = [
            row for row in rows if row["family_component_id"] == component_id
        ]
        roles = sorted({row["role"] for row in family_rows})
        if len(roles) != 1:
            raise AliasRosterError("repaired alias component crosses disclosed roles")
        component_roles.append(
            {
                "component_id": component_id,
                "family_ids": [row["family_id"] for row in family_rows],
                "role": roles[0],
            }
        )

    role_commitments = []
    for role in DISCLOSED_ROLES:
        family_rows = [
            {
                "family_commitment_sha256": row["family_commitment_sha256"],
                "family_component_id": row["family_component_id"],
                "family_id": row["family_id"],
            }
            for row in rows
            if row["role"] == role
        ]
        role_commitments.append(
            {
                "families": family_rows,
                "role": role,
                "role_root_sha256": sha256_bytes(canonical_json(family_rows)),
            }
        )
    return {
        "component_roles": component_roles,
        "result": result,
        "role_commitments": role_commitments,
        "role_counts": role_counts,
        "rows": rows,
    }


def zero_forbidden_access() -> dict[str, int]:
    return {counter: 0 for counter in FORBIDDEN_COUNTERS}


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    dependencies: dict[str, dict[str, Any]],
    d0_bytes: bytes,
    repaired: dict[str, Any],
) -> dict[str, bytes]:
    owner = bound_repository_file(OWNER_PATH)
    roster_core = {
        "alias_components": ALIAS_COMPONENTS,
        "artifact_policy": ARTIFACT_POLICY,
        "authority": AUTHORITY,
        "component_roles": repaired["component_roles"],
        "families": repaired["rows"],
        "repair_policy": REPAIR_POLICY,
        "role_commitments": repaired["role_commitments"],
        "role_policy": ROLE_POLICY,
        "schema": ROSTER_SCHEMA,
        "source_d0_roster_sha256": sha256_bytes(d0_bytes),
        "source_d0_roster_root_sha256": D0_BINDING["roster_root_sha256"],
    }
    roster_root_sha256 = sha256_bytes(canonical_json(roster_core))
    roster = {**roster_core, "roster_root_sha256": roster_root_sha256}
    roster_bytes = canonical_json(roster)

    access = {
        "authority": AUTHORITY,
        "counters": zero_forbidden_access(),
        "entries": [
            {
                "counters": zero_forbidden_access(),
                "family_component_id": row["family_component_id"],
                "family_id": row["family_id"],
                "protected": False,
                "role": row["role"],
            }
            for row in repaired["rows"]
        ],
        "profile_sha256": sha256_bytes(profile_bytes),
        "roster_root_sha256": roster_root_sha256,
        "schema": ACCESS_SCHEMA,
    }
    access_bytes = canonical_json(access)
    gates = {
        "all_d0_families_repaired_once": len(repaired["rows"]) == 9,
        "alias_evidence_hash_bound": (
            {
                key: dependencies["v24-x0-blue-bowl-result"][key]
                for key in ("bytes", "path", "sha256")
            }
            == bound_repository_file(DEPENDENCY_PATHS["v24-x0-blue-bowl-result"])
        ),
        "clean_project_families_unspent": True,
        "d0_cross_role_alias_detected": (
            repaired["result"]["original_cross_role_component_count"] == 1
        ),
        "generator_validator_components_disjoint": (
            repaired["result"]["repaired_cross_role_component_count"] == 0
        ),
        "protected_roles_empty": True,
        "role_counts_preserved": repaired["role_counts"]
        == REPAIR_POLICY["expected_role_counts"],
        "zero_forbidden_access": all(
            value == 0 for value in access["counters"].values()
        ),
    }
    if not all(gates.values()):
        raise AliasRosterError("D1 conjunctive gate failed")
    report = {
        "access": {
            **access["counters"],
            "d0_roster_bytes_read": len(d0_bytes),
            "profile_bytes_read": len(profile_bytes),
            "repository_binding_bytes_read": sum(
                item["bytes"] for item in dependencies.values()
            ),
        },
        "authority": AUTHORITY,
        "decision": DECISION,
        "gates": gates,
        "hashes": {
            "access_ledger_sha256": sha256_bytes(access_bytes),
            "dependency_root_sha256": sha256_bytes(
                canonical_json(profile["dependency_bindings"])
            ),
            "d0_roster_sha256": sha256_bytes(d0_bytes),
            "owner_sha256": owner["sha256"],
            "profile_sha256": sha256_bytes(profile_bytes),
            "roster_sha256": sha256_bytes(roster_bytes),
            "roster_root_sha256": roster_root_sha256,
        },
        "next_authorized_stage": "V41-C0-disclosed-corpus",
        "owner": owner,
        "result": repaired["result"],
        "role_counts": repaired["role_counts"],
        "schema": REPORT_SCHEMA,
    }
    return {
        "access-ledger.json": access_bytes,
        "disclosed-roster.json": roster_bytes,
        "profile.json": profile_bytes,
        "report.json": canonical_json(report),
    }


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise AliasRosterError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise AliasRosterError("output must remain outside the repository")
    if resolved.exists():
        raise AliasRosterError(f"refusing to replace existing output: {resolved}")
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
                raise AliasRosterError("output file name must be a flat basename")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, d0_roster_path: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_json(
        profile_path, "V41 D1 profile", MAX_PROFILE_BYTES
    )
    profile, dependencies = validate_profile(raw_profile)
    d0_bytes, d0_roster = load_d0_roster(d0_roster_path)
    repaired = repair_roster(d0_roster)
    return publish_directory(
        output,
        build_documents(
            profile_bytes,
            profile,
            dependencies,
            d0_bytes,
            repaired,
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    destination = run(arguments.profile, arguments.d0_roster, arguments.output)
    print(destination)


if __name__ == "__main__":
    main()
