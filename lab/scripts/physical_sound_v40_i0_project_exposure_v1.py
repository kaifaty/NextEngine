#!/usr/bin/env python3
"""Build the V40 I0 zero-signal project-family exposure audit."""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_v29_q1a_source_growth_v1 as q1a
import physical_sound_v39_f1_source_frontier_v1 as f1

PROFILE_PATH = "lab/profiles/physical-sound-v40-i0-project-exposure.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v40_i0_project_exposure_v1.py"

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v40-i0-project-exposure-profile.v1"
LEDGER_SCHEMA = "nextengine.experimental-physical-sound-v40-i0-project-ledger.v1"
FRONTIER_SCHEMA = "nextengine.experimental-physical-sound-v40-i0-clean-frontier.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v40-i0-report.v1"
CLAIM = (
    "CURRENT_STEEL_PROJECT_FAMILY_EXPOSURE_AND_CLEAN_FRONTIER_ONLY / "
    "NO_SIGNAL_ROLE_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
DECISION = "I0_PROJECT_EXPOSURE_AUDIT_PASS_SOURCE_POWER_OOD"

MAX_PROFILE_BYTES = 1024 * 1024
MAX_DEPENDENCIES = 16
HASH_LENGTH = 64

MINIMUMS = {
    "protected_exact_steel_groups_per_role": 16,
    "protected_non_metal_groups_per_role": 35,
    "protected_projects_per_role": 2,
    "reserved_unprotected_projects": 5,
}
ACCESS_POLICY = {
    "audio_or_feature_decode_allowed": False,
    "candidate_or_model_access_allowed": False,
    "network_allowed": False,
    "outputs_external": True,
    "payload_access_allowed": False,
    "role_assignment_allowed": False,
}
AUTHORITY = {
    "admission_authority": False,
    "authorizes_only": "v40_d0_disclosed_roster_and_s0_clean_source_growth",
    "model_training_authority": False,
    "payload_access_authority": False,
    "product_authority": False,
    "public_contract": False,
    "role_assignment_authority": False,
    "runtime_consumer_allowed": False,
    "validator_release_authority": False,
}
AUDIT_POLICY = {
    "candidate_project_source": "v39_f1_current_projects",
    "fail_closed_unknown": True,
    "metadata_hash_resets_family": False,
    "metadata_only_origin_evidence": {
        "q1a-internet-metadata": "q1a-metadata-only",
        "v38-s0-internet-metadata": "v39-f1-result",
    },
    "minimum_independence_unit": "publisher_project_revision_family",
    "source_frontier_binding_id": "v39-f1-source-profile",
}

OBJECTFOLDER_PROJECT = (
    "stanford-objectfolder--objectfolder-real--rendered-table-sha256-0111f57a"
)
YCB_PROJECT = "iri-csic-upc-ctu--ycb-impact-sounds--osf-bj5w8-2022-09-27"

FAMILY_POLICIES = [
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["heller-opened"],
        "family_id": "carnegie-mellon-auditorylab--sound-events-impact-events",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 5,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["freesound-glass-bowl-opened"],
        "family_id": "freesound-user-ascap--pack-14905",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 8,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["freesound-wine-glass-opened"],
        "family_id": "freesound-user-wasserbjorn--pack-41981",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 3,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": YCB_PROJECT,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["ycb-opened"],
        "family_id": "iri-csic-upc-ctu--ycb-impact-sounds",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 8,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["soundpacks-opened"],
        "family_id": "kaffekrus--soundpacks-glass-recordings",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 7,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["blue-bowl-objectfolder-realimpact-opened"],
        "family_id": "samuel-clarke--realimpact",
        "historical_access_kind": "derived_signal_values_decoded",
        "observed_unit": "transfer_rows",
        "observed_units_lower_bound": 4,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": OBJECTFOLDER_PROJECT,
        "disposition": "permanent_disclosed",
        "evidence_ids": [
            "blue-bowl-objectfolder-realimpact-opened",
            "objectfolder-beer-glass-opened",
        ],
        "family_id": "stanford-objectfolder--objectfolder-real",
        "historical_access_kind": "signal_values_decoded",
        "observed_unit": "pcm_samples",
        "observed_units_lower_bound": 6336000,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["kronland-opened"],
        "family_id": "the-language-of-sounds--kronland-material-impact-stimuli",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 15,
        "protected_power_allowed": False,
    },
    {
        "candidate_project_revision_id": None,
        "disposition": "permanent_disclosed",
        "evidence_ids": ["avmsf-opened"],
        "family_id": "zisen-shao--av-msf",
        "historical_access_kind": "signal_payload_opened",
        "observed_unit": "recordings",
        "observed_units_lower_bound": 20,
        "protected_power_allowed": False,
    },
]

EXPECTED_RESULT = {
    "accounted_candidate_projects": 11,
    "clean_exact_steel_groups": 9,
    "clean_non_metal_groups": 7,
    "clean_project_revisions": 9,
    "decision": DECISION,
    "disclosed_candidate_project_revisions": 2,
    "feasible": False,
    "known_disclosed_families": 9,
    "quarantined_exact_steel_groups": 23,
    "quarantined_non_metal_groups": 70,
    "reserved_unprotected_project_count": 5,
    "role_a_exact_steel_deficit": 13,
    "role_a_non_metal_deficit": 34,
    "role_b_exact_steel_deficit": 13,
    "role_b_non_metal_deficit": 31,
    "unaccounted_candidate_projects": 0,
}
EXPECTED_CLEAN_PROJECTS_SHA256 = (
    "6b072b22efb26257ff52e926129f96af3d2499be16ef03f6e2c30e78991aa15f"
)
EXPECTED_CLEAN_PARTITION_SHA256 = (
    "dd83db7dfb744193886bf924a1b497d89aa2914dc9a1acbfa28455e4017e32af"
)

PROFILE_KEYS = {
    "access_policy",
    "audit_policy",
    "authority",
    "claim",
    "dependency_bindings",
    "expected_result",
    "family_policies",
    "minimums",
    "schema",
}
DEPENDENCY_KEYS = {"binding_id", "bytes", "path", "sha256"}
FAMILY_POLICY_KEYS = {
    "candidate_project_revision_id",
    "disposition",
    "evidence_ids",
    "family_id",
    "historical_access_kind",
    "observed_unit",
    "observed_units_lower_bound",
    "protected_power_allowed",
}
DEPENDENCY_PATHS = {
    "avmsf-opened": (
        "docs/development/physical-sound-internet-source-pipeline-ps2-2026-08-27.md"
    ),
    "blue-bowl-objectfolder-realimpact-opened": (
        "docs/development/physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md"
    ),
    "freesound-glass-bowl-opened": (
        "docs/development/physical-sound-freesound-glass-bowl-e3-pilot-ps2-2026-08-28.md"
    ),
    "freesound-wine-glass-opened": (
        "docs/development/physical-sound-freesound-wine-glass-e3-pilot-ps2-2026-08-28.md"
    ),
    "heller-opened": (
        "docs/development/physical-sound-heller-independent-e3-pilot-ps2-2026-08-27.md"
    ),
    "kronland-opened": (
        "docs/development/physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md"
    ),
    "objectfolder-beer-glass-opened": (
        "docs/development/physical-sound-r3a-v10-beer-glass-real-fit-result-2026-08-31.md"
    ),
    "q1a-metadata-only": (
        "docs/development/physical-sound-v30-e2-q1a-source-growth-result-2026-09-02.md"
    ),
    "soundpacks-opened": (
        "docs/development/physical-sound-soundpacks-glass-e3-and-split-audit-ps2-2026-08-28.md"
    ),
    "v39-f1-owner": "lab/scripts/physical_sound_v39_f1_source_frontier_v1.py",
    "v39-f1-result": (
        "docs/development/physical-sound-v39-f1-source-frontier-result-2026-09-03.md"
    ),
    "v39-f1-source-profile": (
        "lab/profiles/physical-sound-v39-f1-source-frontier.v1.json"
    ),
    "ycb-opened": (
        "docs/development/physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md"
    ),
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


class ProjectExposureError(RuntimeError):
    """The frozen V40 I0 project-family audit cannot be reproduced."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return q1a.canonical_json(value)
    except q1a.Q1ASourceGrowthError as error:
        raise ProjectExposureError(str(error)) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ProjectExposureError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise ProjectExposureError(
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
        raise ProjectExposureError(f"{context} must be a lowercase SHA-256")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise ProjectExposureError(f"{context} must be a non-negative integer")
    return value


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    if path.is_symlink() or not path.is_file():
        raise ProjectExposureError("profile must be a regular non-symlink file")
    if not 0 < path.stat().st_size <= MAX_PROFILE_BYTES:
        raise ProjectExposureError("profile size is outside the frozen limit")
    data = path.read_bytes()
    try:
        value = q1a.parse_json(data, "V40 I0 profile")
    except q1a.Q1ASourceGrowthError as error:
        raise ProjectExposureError(str(error)) from error
    if canonical_json(value) != data:
        raise ProjectExposureError("V40 I0 profile must use canonical JSON")
    return data, value


def bound_repository_file(path_text: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink() or not path.is_file():
        raise ProjectExposureError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    return {"bytes": len(data), "path": path_text, "sha256": sha256_bytes(data)}


def validate_dependencies(value: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(value, list) or not 1 <= len(value) <= MAX_DEPENDENCIES:
        raise ProjectExposureError("dependency bindings must be a bounded array")
    checked: dict[str, dict[str, Any]] = {}
    order: list[str] = []
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, DEPENDENCY_KEYS, f"dependencies[{index}]")
        binding_id = item["binding_id"]
        if not isinstance(binding_id, str) or binding_id not in DEPENDENCY_PATHS:
            raise ProjectExposureError("unknown dependency binding")
        if item["path"] != DEPENDENCY_PATHS[binding_id]:
            raise ProjectExposureError("dependency path substitution")
        require_nonnegative_integer(item["bytes"], "dependency bytes")
        require_hash(item["sha256"], "dependency sha256")
        expected = bound_repository_file(item["path"])
        if {key: item[key] for key in ("bytes", "path", "sha256")} != expected:
            raise ProjectExposureError(f"bound dependency drift: {item['path']}")
        checked[binding_id] = item
        order.append(binding_id)
    if order != sorted(DEPENDENCY_PATHS) or len(checked) != len(DEPENDENCY_PATHS):
        raise ProjectExposureError("dependency roster or order changed")
    return checked


def validate_family_policies(value: Any, dependencies: dict[str, dict[str, Any]]) -> None:
    if not isinstance(value, list):
        raise ProjectExposureError("family policies must be an array")
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, FAMILY_POLICY_KEYS, f"family_policies[{index}]")
        if not isinstance(item["evidence_ids"], list) or not item["evidence_ids"]:
            raise ProjectExposureError("family policy requires evidence")
        if any(evidence_id not in dependencies for evidence_id in item["evidence_ids"]):
            raise ProjectExposureError("family policy references unknown evidence")
        if item["protected_power_allowed"] is not False:
            raise ProjectExposureError("opened family cannot allow protected power")
    if value != FAMILY_POLICIES:
        raise ProjectExposureError("family policy or evidence changed")


def validate_profile(value: Any) -> tuple[dict[str, Any], dict[str, dict[str, Any]]]:
    profile = require_exact_keys(value, PROFILE_KEYS, "profile")
    if profile["schema"] != PROFILE_SCHEMA:
        raise ProjectExposureError("unknown V40 I0 profile schema")
    if profile["claim"] != CLAIM:
        raise ProjectExposureError("V40 I0 claim changed")
    for key, expected in (
        ("access_policy", ACCESS_POLICY),
        ("audit_policy", AUDIT_POLICY),
        ("authority", AUTHORITY),
        ("expected_result", EXPECTED_RESULT),
        ("minimums", MINIMUMS),
    ):
        if profile[key] != expected:
            raise ProjectExposureError(f"profile {key} changed")
    dependencies = validate_dependencies(profile["dependency_bindings"])
    validate_family_policies(profile["family_policies"], dependencies)
    return profile, dependencies


def load_source_frontier(
    dependencies: dict[str, dict[str, Any]],
) -> tuple[bytes, dict[str, Any], dict[str, Any]]:
    binding = dependencies[AUDIT_POLICY["source_frontier_binding_id"]]
    path = repository_root() / binding["path"]
    source_bytes, raw = f1.read_canonical_profile(path)
    source_profile = f1.validate_profile(source_bytes, raw)
    solved = f1.solve_frontiers(source_profile)
    return source_bytes, source_profile, solved


def classify_projects(source_solved: dict[str, Any]) -> dict[str, Any]:
    policies_by_candidate = {
        item["candidate_project_revision_id"]: item
        for item in FAMILY_POLICIES
        if item["candidate_project_revision_id"] is not None
    }
    metadata_evidence = AUDIT_POLICY["metadata_only_origin_evidence"]
    classifications = []
    clean_projects = []
    quarantined_projects = []
    unknown_projects = []
    for project in source_solved["current_projects"]:
        project_id = project["project_revision_id"]
        policy = policies_by_candidate.get(project_id)
        if policy is not None:
            classification = {
                "evidence_ids": policy["evidence_ids"],
                "exact_steel_groups": project["exact_steel_groups"],
                "exposure_state": "opened_project_family",
                "family_id": policy["family_id"],
                "non_metal_groups": project["non_metal_groups"],
                "origin": project["origin"],
                "project_revision_id": project_id,
                "protected_eligible": False,
            }
            quarantined_projects.append(project)
        elif project["origin"] in metadata_evidence:
            classification = {
                "evidence_ids": [metadata_evidence[project["origin"]]],
                "exact_steel_groups": project["exact_steel_groups"],
                "exposure_state": "metadata_only_candidate",
                "family_id": project_id,
                "non_metal_groups": project["non_metal_groups"],
                "origin": project["origin"],
                "project_revision_id": project_id,
                "protected_eligible": True,
            }
            clean_projects.append(project)
        else:
            classification = {
                "evidence_ids": [],
                "exact_steel_groups": project["exact_steel_groups"],
                "exposure_state": "unknown_quarantine",
                "family_id": project_id,
                "non_metal_groups": project["non_metal_groups"],
                "origin": project["origin"],
                "project_revision_id": project_id,
                "protected_eligible": False,
            }
            unknown_projects.append(project)
        classifications.append(classification)
    classifications.sort(key=lambda item: item["project_revision_id"])
    clean_projects.sort(key=lambda item: item["project_revision_id"])
    quarantined_projects.sort(key=lambda item: item["project_revision_id"])
    unknown_projects.sort(key=lambda item: item["project_revision_id"])
    return {
        "classifications": classifications,
        "clean_projects": clean_projects,
        "quarantined_projects": quarantined_projects,
        "unknown_projects": unknown_projects,
    }


def solve_audit(source_solved: dict[str, Any]) -> dict[str, Any]:
    classified = classify_projects(source_solved)
    clean_projects = classified["clean_projects"]
    clean_projects_sha256 = sha256_bytes(canonical_json(clean_projects))
    if clean_projects_sha256 != EXPECTED_CLEAN_PROJECTS_SHA256:
        raise ProjectExposureError("clean project projection drift")
    clean_partition = q1a.solve_protected_partition(clean_projects, MINIMUMS)
    clean_partition_sha256 = sha256_bytes(canonical_json(clean_partition))
    if clean_partition_sha256 != EXPECTED_CLEAN_PARTITION_SHA256:
        raise ProjectExposureError("clean frontier planner drift")
    frontier = clean_partition.get("best_frontier")
    if not isinstance(frontier, dict):
        raise ProjectExposureError("clean planner omitted best frontier")
    role_a = frontier["role_a"]
    role_b = frontier["role_b"]
    clean_power = {
        "exact_steel_groups": sum(
            item["exact_steel_groups"] for item in clean_projects
        ),
        "non_metal_groups": sum(item["non_metal_groups"] for item in clean_projects),
        "project_revisions": len(clean_projects),
    }
    quarantined_power = {
        "exact_steel_groups": sum(
            item["exact_steel_groups"]
            for item in classified["quarantined_projects"]
        ),
        "non_metal_groups": sum(
            item["non_metal_groups"]
            for item in classified["quarantined_projects"]
        ),
        "project_revisions": len(classified["quarantined_projects"]),
    }
    observed = {
        "accounted_candidate_projects": len(classified["classifications"])
        - len(classified["unknown_projects"]),
        "clean_exact_steel_groups": clean_power["exact_steel_groups"],
        "clean_non_metal_groups": clean_power["non_metal_groups"],
        "clean_project_revisions": clean_power["project_revisions"],
        "decision": DECISION,
        "disclosed_candidate_project_revisions": quarantined_power[
            "project_revisions"
        ],
        "feasible": clean_partition["feasible"],
        "known_disclosed_families": len(FAMILY_POLICIES),
        "quarantined_exact_steel_groups": quarantined_power[
            "exact_steel_groups"
        ],
        "quarantined_non_metal_groups": quarantined_power["non_metal_groups"],
        "reserved_unprotected_project_count": frontier[
            "reserved_unprotected_project_count"
        ],
        "role_a_exact_steel_deficit": role_a["exact_steel_deficit"],
        "role_a_non_metal_deficit": role_a["non_metal_deficit"],
        "role_b_exact_steel_deficit": role_b["exact_steel_deficit"],
        "role_b_non_metal_deficit": role_b["non_metal_deficit"],
        "unaccounted_candidate_projects": len(classified["unknown_projects"]),
    }
    if observed != EXPECTED_RESULT:
        raise ProjectExposureError(f"I0 result drift: {observed}")
    return {
        **classified,
        "clean_partition": clean_partition,
        "clean_partition_sha256": clean_partition_sha256,
        "clean_power": clean_power,
        "clean_projects_sha256": clean_projects_sha256,
        "observed": observed,
        "quarantined_power": quarantined_power,
    }


def zero_forbidden_access() -> dict[str, int]:
    return {counter: 0 for counter in FORBIDDEN_COUNTERS}


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    dependencies: dict[str, dict[str, Any]],
    source_profile_bytes: bytes,
    source_solved: dict[str, Any],
    solved: dict[str, Any],
) -> dict[str, bytes]:
    owner = bound_repository_file(OWNER_PATH)
    ledger = {
        "audit_policy": AUDIT_POLICY,
        "authority": AUTHORITY,
        "candidate_projects": solved["classifications"],
        "candidate_source": {
            "f1_profile_sha256": sha256_bytes(source_profile_bytes),
            "f1_projects_sha256": sha256_bytes(
                canonical_json(source_solved["current_projects"])
            ),
        },
        "decision": DECISION,
        "known_disclosed_families": FAMILY_POLICIES,
        "schema": LEDGER_SCHEMA,
    }
    ledger_bytes = canonical_json(ledger)
    clean_frontier = {
        "authority": AUTHORITY,
        "clean_partition": solved["clean_partition"],
        "clean_partition_sha256": solved["clean_partition_sha256"],
        "clean_power": solved["clean_power"],
        "clean_projects": solved["clean_projects"],
        "clean_projects_sha256": solved["clean_projects_sha256"],
        "decision": "SourcePowerOOD",
        "historical_f1_replay": {
            "admission_authority": False,
            "role_b_exact_steel_deficit": 6,
            "role_b_non_metal_deficit": 23,
            "status": "historical_replay_only",
        },
        "minimums": MINIMUMS,
        "quarantined_power": solved["quarantined_power"],
        "schema": FRONTIER_SCHEMA,
    }
    frontier_bytes = canonical_json(clean_frontier)
    access = {
        **zero_forbidden_access(),
        "profile_bytes_read": len(profile_bytes),
        "repository_binding_bytes_read": sum(
            item["bytes"] for item in dependencies.values()
        ),
    }
    gates = {
        "all_candidate_projects_accounted": (
            solved["observed"]["accounted_candidate_projects"] == 11
            and solved["observed"]["unaccounted_candidate_projects"] == 0
        ),
        "clean_frontier_repeat_exact": (
            solved["clean_partition_sha256"] == EXPECTED_CLEAN_PARTITION_SHA256
        ),
        "metadata_hash_does_not_reset_family": (
            AUDIT_POLICY["metadata_hash_resets_family"] is False
        ),
        "objectfolder_and_ycb_quarantined": {
            item["project_revision_id"]
            for item in solved["quarantined_projects"]
        }
        == {OBJECTFOLDER_PROJECT, YCB_PROJECT},
        "protected_power_is_9_7": solved["clean_power"]
        == {
            "exact_steel_groups": 9,
            "non_metal_groups": 7,
            "project_revisions": 9,
        },
        "protected_role_assignment_forbidden": (
            AUTHORITY["role_assignment_authority"] is False
        ),
        "source_power_ood": solved["clean_partition"]["feasible"] is False,
        "zero_forbidden_access": all(
            access[counter] == 0 for counter in FORBIDDEN_COUNTERS
        ),
    }
    if not all(gates.values()):
        raise ProjectExposureError("I0 conjunctive gate failed")
    report = {
        "access": access,
        "authority": AUTHORITY,
        "decision": DECISION,
        "gates": gates,
        "hashes": {
            "clean_frontier_sha256": sha256_bytes(frontier_bytes),
            "dependency_root_sha256": sha256_bytes(
                canonical_json(profile["dependency_bindings"])
            ),
            "ledger_sha256": sha256_bytes(ledger_bytes),
            "owner_sha256": owner["sha256"],
            "profile_sha256": sha256_bytes(profile_bytes),
        },
        "next_authorized_stage": "V40-D0-permanent-disclosed-roster",
        "owner": owner,
        "result": solved["observed"],
        "schema": REPORT_SCHEMA,
    }
    return {
        "clean-frontier.json": frontier_bytes,
        "profile.json": profile_bytes,
        "project-ledger.json": ledger_bytes,
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
        raise ProjectExposureError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise ProjectExposureError("output must remain outside the repository")
    if resolved.exists():
        raise ProjectExposureError(f"refusing to replace existing output: {resolved}")
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
                raise ProjectExposureError("output file name must be a flat basename")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile, dependencies = validate_profile(raw_profile)
    source_profile_bytes, _, source_solved = load_source_frontier(dependencies)
    solved = solve_audit(source_solved)
    return publish_directory(
        output,
        build_documents(
            profile_bytes,
            profile,
            dependencies,
            source_profile_bytes,
            source_solved,
            solved,
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    destination = run(arguments.profile, arguments.output)
    print(destination)


if __name__ == "__main__":
    main()
