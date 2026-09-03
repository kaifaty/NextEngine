#!/usr/bin/env python3
"""Freeze the V40 D0 permanent disclosed-family role roster without signal access."""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_v39_f0_foundry_preflight_v1 as f0
import physical_sound_v40_i0_project_exposure_v1 as i0

PROFILE_PATH = "lab/profiles/physical-sound-v40-d0-disclosed-roster.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v40_d0_disclosed_roster_v1.py"

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v40-d0-profile.v1"
ROSTER_SCHEMA = "nextengine.experimental-physical-sound-v40-d0-disclosed-roster.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v40-d0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v40-d0-report.v1"
CLAIM = (
    "PERMANENT_DISCLOSED_PROJECT_FAMILY_ROLE_FREEZE_ONLY / "
    "NO_PAYLOAD_SIGNAL_FEATURE_MODEL_PROTECTED_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
DECISION = "D0_DISCLOSED_ROSTER_FROZEN_C0_AUTHORIZED"

MAX_PROFILE_BYTES = 1024 * 1024
MAX_DEPENDENCIES = 16
HASH_LENGTH = 64

DISCLOSED_ROLES = (
    "generator_development",
    "generator_train",
    "validator_calibration",
)
PROTECTED_ROLES = (
    "generator_method_holdout",
    "joint_admission_shadow",
    "validator_qualification",
)

DEPENDENCY_PATHS = {
    "i0-owner": "lab/scripts/physical_sound_v40_i0_project_exposure_v1.py",
    "i0-profile": "lab/profiles/physical-sound-v40-i0-project-exposure.v1.json",
    "i0-result": (
        "docs/development/physical-sound-v40-i0-project-exposure-result-2026-09-03.md"
    ),
    "legacy-split-result": (
        "docs/development/physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md"
    ),
    "v39-f0-owner": "lab/scripts/physical_sound_v39_f0_foundry_preflight_v1.py",
    "v39-f0-profile": "lab/profiles/physical-sound-v39-f0-foundry.v1.json",
    "v39-f0-result": (
        "docs/development/physical-sound-v39-f0-foundry-preflight-result-2026-09-03.md"
    ),
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
ARTIFACT_POLICY = {
    "candidate_outputs_visible_to_validator_calibration": False,
    "generator_namespace": "external/physical-sound/generator",
    "shared_learned_artifacts_allowed": False,
    "shared_thresholds_allowed": False,
    "validator_namespace": "external/physical-sound/validator",
}
ROLE_POLICY = {
    "clean_project_consumption_allowed": False,
    "cross_family_parent_alias_behavior": "reject",
    "descendants_inherit_family_role": True,
    "disclosed_roles": list(DISCLOSED_ROLES),
    "family_role_reassignment_after_freeze": "forbidden",
    "partition_unit": "publisher_project_revision_family",
    "protected_roles": list(PROTECTED_ROLES),
    "unknown_family_behavior": "reject",
    "whole_family_disjoint": True,
}
ASSIGNMENT_POLICY = {
    "expected_family_count": 9,
    "expected_role_counts": {
        "generator_development": 2,
        "generator_train": 5,
        "validator_calibration": 2,
    },
    "historical_partition_role_map": {
        "calibration": "validator_calibration",
        "dev": "generator_development",
        "holdout": "generator_train",
        "shadow": "generator_train",
        "v24_development": "generator_train",
    },
    "legacy_partitioned_manifest_sha256": (
        "9ddec04de271c034f1389f5fcec98ce410028b307ae565b3f890eb7f7a726c33"
    ),
    "selection_rule": (
        "preserve_legacy_dev_and_calibration_roles_then_collapse_spent_"
        "holdout_shadow_and_prior_development_into_generator_train_v1"
    ),
}
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": "v40_c0_disclosed_corpus_and_s0_clean_source_growth",
    "disclosed_role_assignment_authority": True,
    "model_training_authority": False,
    "payload_access_authority": False,
    "product_authority": False,
    "protected_role_assignment_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_release_authority": False,
}

EXPECTED_ASSIGNMENTS = [
    {
        "candidate_project_revision_id": None,
        "family_id": "carnegie-mellon-auditorylab--sound-events-impact-events",
        "historical_partition": "calibration",
        "role": "validator_calibration",
    },
    {
        "candidate_project_revision_id": None,
        "family_id": "freesound-user-ascap--pack-14905",
        "historical_partition": "dev",
        "role": "generator_development",
    },
    {
        "candidate_project_revision_id": None,
        "family_id": "freesound-user-wasserbjorn--pack-41981",
        "historical_partition": "shadow",
        "role": "generator_train",
    },
    {
        "candidate_project_revision_id": (
            "iri-csic-upc-ctu--ycb-impact-sounds--osf-bj5w8-2022-09-27"
        ),
        "family_id": "iri-csic-upc-ctu--ycb-impact-sounds",
        "historical_partition": "dev",
        "role": "generator_development",
    },
    {
        "candidate_project_revision_id": None,
        "family_id": "kaffekrus--soundpacks-glass-recordings",
        "historical_partition": "holdout",
        "role": "generator_train",
    },
    {
        "candidate_project_revision_id": None,
        "family_id": "samuel-clarke--realimpact",
        "historical_partition": "v24_development",
        "role": "generator_train",
    },
    {
        "candidate_project_revision_id": (
            "stanford-objectfolder--objectfolder-real--rendered-table-"
            "sha256-0111f57a"
        ),
        "family_id": "stanford-objectfolder--objectfolder-real",
        "historical_partition": "calibration",
        "role": "validator_calibration",
    },
    {
        "candidate_project_revision_id": None,
        "family_id": (
            "the-language-of-sounds--kronland-material-impact-stimuli"
        ),
        "historical_partition": "shadow",
        "role": "generator_train",
    },
    {
        "candidate_project_revision_id": None,
        "family_id": "zisen-shao--av-msf",
        "historical_partition": "holdout",
        "role": "generator_train",
    },
]

EXPECTED_RESULT = {
    "clean_project_families_spent": 0,
    "decision": DECISION,
    "disclosed_family_count": 9,
    "generator_development_families": 2,
    "generator_train_families": 5,
    "protected_family_count": 0,
    "unassigned_disclosed_families": 0,
    "unknown_family_count": 0,
    "validator_calibration_families": 2,
}

FORBIDDEN_COUNTERS = (
    "audio_header_values_read",
    "audio_preview_bytes_read",
    "candidate_output_values_read",
    "feature_values_read",
    "force_sample_values_decoded",
    "mesh_values_decoded",
    "model_target_values_read",
    "network_requests",
    "pcm_sample_values_decoded",
    "protected_signal_values_decoded",
    "role_signal_values_opened",
    "source_payload_bytes_read",
)

PROFILE_KEYS = {
    "access_policy",
    "artifact_policy",
    "assignment_policy",
    "assignments",
    "authority",
    "claim",
    "dependency_bindings",
    "expected_result",
    "role_policy",
    "schema",
}
DEPENDENCY_KEYS = {"binding_id", "bytes", "path", "sha256"}
ASSIGNMENT_KEYS = {
    "candidate_project_revision_id",
    "family_id",
    "historical_partition",
    "role",
}


class DisclosedRosterError(RuntimeError):
    """The V40 D0 permanent disclosed roster cannot be frozen."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return i0.canonical_json(value)
    except i0.ProjectExposureError as error:
        raise DisclosedRosterError(str(error)) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise DisclosedRosterError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise DisclosedRosterError(
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
        raise DisclosedRosterError(f"{context} must be a lowercase SHA-256")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise DisclosedRosterError(f"{context} must be a non-negative integer")
    return value


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    if path.is_symlink() or not path.is_file():
        raise DisclosedRosterError("profile must be a regular non-symlink file")
    if not 0 < path.stat().st_size <= MAX_PROFILE_BYTES:
        raise DisclosedRosterError("profile size is outside the frozen limit")
    data = path.read_bytes()
    try:
        value = i0.q1a.parse_json(data, "V40 D0 profile")
    except i0.q1a.Q1ASourceGrowthError as error:
        raise DisclosedRosterError(str(error)) from error
    if canonical_json(value) != data:
        raise DisclosedRosterError("V40 D0 profile must use canonical JSON")
    return data, value


def bound_repository_file(path_text: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink() or not path.is_file():
        raise DisclosedRosterError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    return {"bytes": len(data), "path": path_text, "sha256": sha256_bytes(data)}


def validate_dependencies(value: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(value, list) or not 1 <= len(value) <= MAX_DEPENDENCIES:
        raise DisclosedRosterError("dependency bindings must be a bounded array")
    checked: dict[str, dict[str, Any]] = {}
    order = []
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, DEPENDENCY_KEYS, f"dependencies[{index}]")
        binding_id = item["binding_id"]
        if not isinstance(binding_id, str) or binding_id not in DEPENDENCY_PATHS:
            raise DisclosedRosterError("unknown dependency binding")
        if item["path"] != DEPENDENCY_PATHS[binding_id]:
            raise DisclosedRosterError("dependency path substitution")
        require_nonnegative_integer(item["bytes"], "dependency bytes")
        require_hash(item["sha256"], "dependency sha256")
        expected = bound_repository_file(item["path"])
        if {key: item[key] for key in ("bytes", "path", "sha256")} != expected:
            raise DisclosedRosterError(f"dependency drift: {item['path']}")
        if binding_id in checked:
            raise DisclosedRosterError("duplicate dependency binding")
        checked[binding_id] = item
        order.append(binding_id)
    if order != sorted(DEPENDENCY_PATHS) or len(checked) != len(DEPENDENCY_PATHS):
        raise DisclosedRosterError("dependency roster or order changed")
    return checked


def validate_assignments(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        raise DisclosedRosterError("assignments must be an array")
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, ASSIGNMENT_KEYS, f"assignments[{index}]")
        if not isinstance(item["family_id"], str) or not item["family_id"]:
            raise DisclosedRosterError("assignment family must be a non-empty string")
        revision = item["candidate_project_revision_id"]
        if revision is not None and (not isinstance(revision, str) or not revision):
            raise DisclosedRosterError("candidate revision must be null or non-empty")
        if item["role"] not in DISCLOSED_ROLES:
            raise DisclosedRosterError("assignment uses a non-disclosed role")
        expected_role = ASSIGNMENT_POLICY["historical_partition_role_map"].get(
            item["historical_partition"]
        )
        if item["role"] != expected_role:
            raise DisclosedRosterError("historical partition role map changed")
    if value != EXPECTED_ASSIGNMENTS:
        raise DisclosedRosterError("frozen family assignments changed")
    return value


def validate_profile(value: Any) -> tuple[dict[str, Any], dict[str, dict[str, Any]]]:
    profile = require_exact_keys(value, PROFILE_KEYS, "profile")
    if profile["schema"] != PROFILE_SCHEMA:
        raise DisclosedRosterError("unknown V40 D0 profile schema")
    if profile["claim"] != CLAIM:
        raise DisclosedRosterError("V40 D0 claim changed")
    for key, expected in (
        ("access_policy", ACCESS_POLICY),
        ("artifact_policy", ARTIFACT_POLICY),
        ("assignment_policy", ASSIGNMENT_POLICY),
        ("authority", AUTHORITY),
        ("expected_result", EXPECTED_RESULT),
        ("role_policy", ROLE_POLICY),
    ):
        if profile[key] != expected:
            raise DisclosedRosterError(f"profile {key} changed")
    dependencies = validate_dependencies(profile["dependency_bindings"])
    validate_assignments(profile["assignments"])
    return profile, dependencies


def load_prerequisites(
    dependencies: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    f0_path = repository_root() / dependencies["v39-f0-profile"]["path"]
    f0_profile_bytes, foundry_profile = f0.load_and_validate_profile(f0_path)
    if foundry_profile["artifact_policy"] != ARTIFACT_POLICY:
        raise DisclosedRosterError("F0 artifact separation policy drift")
    if tuple(sorted(foundry_profile["role_policy"]["disclosed_roles"])) != DISCLOSED_ROLES:
        raise DisclosedRosterError("F0 disclosed roles drift")
    if tuple(sorted(foundry_profile["role_policy"]["protected_roles"])) != PROTECTED_ROLES:
        raise DisclosedRosterError("F0 protected roles drift")

    i0_path = repository_root() / dependencies["i0-profile"]["path"]
    i0_profile_bytes, raw_i0_profile = i0.read_canonical_profile(i0_path)
    i0_profile, i0_dependencies = i0.validate_profile(raw_i0_profile)
    source_profile_bytes, _, source_solved = i0.load_source_frontier(i0_dependencies)
    i0_solved = i0.solve_audit(source_solved)
    i0_documents = i0.build_documents(
        i0_profile_bytes,
        i0_profile,
        i0_dependencies,
        source_profile_bytes,
        source_solved,
        i0_solved,
    )
    return {
        "f0_profile_bytes": f0_profile_bytes,
        "i0_documents": i0_documents,
        "i0_profile_bytes": i0_profile_bytes,
        "i0_solved": i0_solved,
    }


def family_commitment_projection(row: dict[str, Any]) -> dict[str, Any]:
    return {
        "candidate_project_revision_id": row["candidate_project_revision_id"],
        "evidence_ids": row["evidence_ids"],
        "family_id": row["family_id"],
        "historical_access_kind": row["historical_access_kind"],
        "historical_partition": row["historical_partition"],
        "role": row["role"],
    }


def freeze_roster(prerequisites: dict[str, Any]) -> dict[str, Any]:
    policies = {item["family_id"]: item for item in i0.FAMILY_POLICIES}
    if len(policies) != len(i0.FAMILY_POLICIES):
        raise DisclosedRosterError("I0 contains duplicate disclosed family")
    assignments = {item["family_id"]: item for item in EXPECTED_ASSIGNMENTS}
    if len(assignments) != len(EXPECTED_ASSIGNMENTS):
        raise DisclosedRosterError("D0 contains duplicate family assignment")
    if set(assignments) != set(policies):
        raise DisclosedRosterError("D0 does not cover the exact I0 disclosed roster")

    clean_families = {
        item["family_id"]
        for item in prerequisites["i0_solved"]["classifications"]
        if item["protected_eligible"]
    }
    if clean_families & set(assignments):
        raise DisclosedRosterError("D0 assignment consumes a clean project family")

    rows = []
    for family_id in sorted(assignments):
        assignment = assignments[family_id]
        policy = policies[family_id]
        if assignment["candidate_project_revision_id"] != policy[
            "candidate_project_revision_id"
        ]:
            raise DisclosedRosterError("candidate revision alias changed")
        if policy["disposition"] != "permanent_disclosed":
            raise DisclosedRosterError("I0 family is not permanently disclosed")
        if policy["protected_power_allowed"] is not False:
            raise DisclosedRosterError("disclosed family regained protected power")
        row = {
            "candidate_project_revision_id": policy["candidate_project_revision_id"],
            "current_run_signal_access": "none",
            "evidence_ids": policy["evidence_ids"],
            "family_id": family_id,
            "historical_access_kind": policy["historical_access_kind"],
            "historical_partition": assignment["historical_partition"],
            "observed_unit": policy["observed_unit"],
            "observed_units_lower_bound": policy["observed_units_lower_bound"],
            "permanent_disclosed": True,
            "protected": False,
            "role": assignment["role"],
        }
        row["family_commitment_sha256"] = sha256_bytes(
            canonical_json(family_commitment_projection(row))
        )
        rows.append(row)

    role_counts = {
        role: sum(row["role"] == role for row in rows) for role in DISCLOSED_ROLES
    }
    if role_counts != ASSIGNMENT_POLICY["expected_role_counts"]:
        raise DisclosedRosterError("disclosed role counts changed")
    role_commitments = []
    for role in DISCLOSED_ROLES:
        family_rows = [
            {
                "family_commitment_sha256": row["family_commitment_sha256"],
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

    result = {
        "clean_project_families_spent": 0,
        "decision": DECISION,
        "disclosed_family_count": len(rows),
        "generator_development_families": role_counts["generator_development"],
        "generator_train_families": role_counts["generator_train"],
        "protected_family_count": 0,
        "unassigned_disclosed_families": len(set(policies) - set(assignments)),
        "unknown_family_count": len(set(assignments) - set(policies)),
        "validator_calibration_families": role_counts["validator_calibration"],
    }
    if result != EXPECTED_RESULT:
        raise DisclosedRosterError(f"D0 result drift: {result}")
    return {
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
    prerequisites: dict[str, Any],
    frozen: dict[str, Any],
) -> dict[str, bytes]:
    owner = bound_repository_file(OWNER_PATH)
    roster_core = {
        "artifact_policy": ARTIFACT_POLICY,
        "assignment_policy": ASSIGNMENT_POLICY,
        "authority": AUTHORITY,
        "families": frozen["rows"],
        "role_commitments": frozen["role_commitments"],
        "role_policy": ROLE_POLICY,
        "schema": ROSTER_SCHEMA,
    }
    roster_root_sha256 = sha256_bytes(canonical_json(roster_core))
    roster = {**roster_core, "roster_root_sha256": roster_root_sha256}
    roster_bytes = canonical_json(roster)

    entries = [
        {
            "counters": zero_forbidden_access(),
            "family_commitment_sha256": row["family_commitment_sha256"],
            "family_id": row["family_id"],
            "protected": False,
            "role": row["role"],
        }
        for row in frozen["rows"]
    ]
    access = {
        "authority": AUTHORITY,
        "counters": zero_forbidden_access(),
        "entries": entries,
        "profile_sha256": sha256_bytes(profile_bytes),
        "roster_root_sha256": roster_root_sha256,
        "schema": ACCESS_SCHEMA,
    }
    access_bytes = canonical_json(access)
    if any(value != 0 for value in access["counters"].values()):
        raise DisclosedRosterError("D0 forbidden access is non-zero")

    i0_report_sha256 = sha256_bytes(prerequisites["i0_documents"]["report.json"])
    gates = {
        "all_i0_disclosed_families_assigned_once": len(frozen["rows"]) == 9,
        "clean_project_families_unspent": (
            frozen["result"]["clean_project_families_spent"] == 0
        ),
        "disclosed_roles_match_f0": tuple(DISCLOSED_ROLES)
        == tuple(sorted(f0.DISCLOSED_ROLES)),
        "generator_validator_artifacts_disjoint": (
            ARTIFACT_POLICY["shared_learned_artifacts_allowed"] is False
            and ARTIFACT_POLICY["shared_thresholds_allowed"] is False
        ),
        "historical_role_rule_frozen": all(
            ASSIGNMENT_POLICY["historical_partition_role_map"][
                row["historical_partition"]
            ]
            == row["role"]
            for row in frozen["rows"]
        ),
        "protected_roles_empty": frozen["result"]["protected_family_count"] == 0,
        "whole_families_disjoint": len(
            {row["family_commitment_sha256"] for row in frozen["rows"]}
        )
        == len(frozen["rows"]),
        "zero_forbidden_access": all(
            access["counters"][counter] == 0 for counter in FORBIDDEN_COUNTERS
        ),
    }
    if not all(gates.values()):
        raise DisclosedRosterError("D0 conjunctive gate failed")
    report = {
        "access": {
            **access["counters"],
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
            "f0_profile_sha256": sha256_bytes(prerequisites["f0_profile_bytes"]),
            "i0_profile_sha256": sha256_bytes(prerequisites["i0_profile_bytes"]),
            "i0_report_sha256": i0_report_sha256,
            "owner_sha256": owner["sha256"],
            "profile_sha256": sha256_bytes(profile_bytes),
            "roster_sha256": sha256_bytes(roster_bytes),
        },
        "next_authorized_stage": "V40-C0-disclosed-corpus",
        "owner": owner,
        "result": frozen["result"],
        "role_counts": frozen["role_counts"],
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
        raise DisclosedRosterError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise DisclosedRosterError("output must remain outside the repository")
    if resolved.exists():
        raise DisclosedRosterError(f"refusing to replace existing output: {resolved}")
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
                raise DisclosedRosterError("output file name must be a flat basename")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile, dependencies = validate_profile(raw_profile)
    prerequisites = load_prerequisites(dependencies)
    frozen = freeze_roster(prerequisites)
    return publish_directory(
        output,
        build_documents(
            profile_bytes,
            profile,
            dependencies,
            prerequisites,
            frozen,
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    destination = run(arguments.profile, arguments.output)
    print(destination)


if __name__ == "__main__":
    main()
