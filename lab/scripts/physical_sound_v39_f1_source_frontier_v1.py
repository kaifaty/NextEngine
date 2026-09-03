#!/usr/bin/env python3
"""Replay the V30 plus IETeasy metadata-only source frontier for V39 F1."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import tempfile
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any

import physical_sound_v29_q1a_source_growth_v1 as q1a

PROFILE_PATH = "lab/profiles/physical-sound-v39-f1-source-frontier.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v39_f1_source_frontier_v1.py"

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v39-f1-source-frontier-profile.v1"
CLASSIFICATION_SCHEMA = (
    "nextengine.experimental-physical-sound-v39-f1-source-classification.v1"
)
FRONTIER_SCHEMA = "nextengine.experimental-physical-sound-v39-f1-frontier.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v39-f1-report.v1"
CLAIM = (
    "V30_PLUS_IETEASY_SOURCE_FRONTIER_REPLAY_ONLY / "
    "NO_SIGNAL_ROLE_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)

MAX_PROFILE_BYTES = 1024 * 1024
MAX_PROJECTS = 18
HASH_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,255}\Z")

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
    "authorizes_only": "v39_f1_source_frontier_replay",
    "model_training_authority": False,
    "payload_access_authority": False,
    "product_authority": False,
    "public_contract": False,
    "role_assignment_authority": False,
    "runtime_consumer_allowed": False,
    "validator_release_authority": False,
}
BATCH_POLICY = {
    "allowed_dispositions": [
        "ADMITTED_METADATA_INCREMENT",
        "METADATA_UNAVAILABLE",
        "MISSING_SOURCE_IDENTITY",
    ],
    "max_leads": 3,
    "metadata_before_payload": True,
    "repeated_recordings_count_as_one_group": True,
    "stable_project_revision_required": True,
}
EXPECTED_RESULT = {
    "baseline_role_b_exact_steel_deficit": 6,
    "baseline_role_b_non_metal_deficit": 27,
    "current_exact_steel_groups": 32,
    "current_non_metal_groups": 77,
    "current_project_revisions": 11,
    "current_role_b_exact_steel_deficit": 6,
    "current_role_b_non_metal_deficit": 23,
    "decision": "ImprovedFrontier",
    "feasible": False,
    "reserved_unprotected_project_count": 5,
}

BASELINE_AUDIT_RECEIPT = {
    "bytes": 31_439,
    "sha256": "c40716fee9c43ebd0132bc14a0d8c77acd01feca9dc7518282a7cca5336bdccd",
}
BASELINE_REPORT_RECEIPT = {
    "bytes": 3_407,
    "sha256": "93d95fe7ef1e4233f3421850d488a8a48d56cd9d6394236af2fa0ca3613458f4",
}
BASELINE_PROJECTS_SHA256 = (
    "b0644e969eb3798d4bce54c81fad71ff317b050ad528747cce35dfbb4dc06768"
)
BASELINE_PARTITION_SHA256 = (
    "3dd0325a6af90e0af2cb0b4fffd7f59e2be9263bdd4db1c634a9264601c3d2df"
)
BASELINE_POWER = {
    "exact_steel_groups": 32,
    "non_metal_groups": 73,
    "project_revisions": 10,
}
BASELINE_DECISION = "Q1A_RAW_GROWTH_VERIFIED_BALANCED_ROLE_POWER_REQUIRED"
TARGET_POLICY = "exact_unqualified_publisher_steel_v1"

DEPENDENCY_PATHS = (
    "docs/development/physical-sound-v30-e2-q1a-source-growth-result-2026-09-02.md",
    "docs/development/physical-sound-v38-s0-gap-directed-source-result-2026-09-03.md",
    "docs/development/physical-sound-v39-f0-foundry-preflight-result-2026-09-03.md",
    "lab/profiles/physical-sound-v29-q1a-source-growth.v1.json",
    "lab/profiles/physical-sound-v39-f0-foundry.v1.json",
    "lab/scripts/physical_sound_v29_q1a_source_growth_v1.py",
    "lab/scripts/physical_sound_v39_f0_foundry_preflight_v1.py",
)

EXPECTED_PRIMARY_SOURCES = [
    "https://data.mendeley.com/datasets/srfp7x6wxm/1",
    "https://pmc.ncbi.nlm.nih.gov/articles/PMC8567360/",
    "https://pmc.ncbi.nlm.nih.gov/articles/PMC9123443/",
]
EXPECTED_LEADS_SHA256 = (
    "d1e48be4bdd5b38cac0007910e680157bc0516ce0948adc13f358cc009a5b52e"
)
EXPECTED_SAMPLE_ROSTER_SHA256 = (
    "ba3535bcdc41eec5fe663b9adafd41a022595847ff52974fd0651ed6ed31ed60"
)
EXPECTED_LEADS = {
    "diffimpact-asmr": {
        "counts": (0, 0),
        "disposition": "MISSING_SOURCE_IDENTITY",
        "urls": [
            "https://github.com/samuel-clarke/diffimpact",
            "https://samuelpclarke.com/files/clarkeCorl2021.pdf",
            "https://sites.google.com/view/diffimpact",
        ],
    },
    "ieteasy-v1": {
        "counts": (0, 4),
        "disposition": "ADMITTED_METADATA_INCREMENT",
        "urls": [
            "https://data.mendeley.com/datasets/srfp7x6wxm/1",
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC8567360/",
        ],
    },
    "rsaudio-isnn": {
        "counts": (0, 0),
        "disposition": "METADATA_UNAVAILABLE",
        "urls": [
            "https://gamma.cs.unc.edu/ISNN/",
            "https://gamma.cs.unc.edu/ISNN/isnn-supplemental_eccv.pdf",
        ],
    },
}

EXPECTED_SAMPLE_RELATIONS = {
    "6082 aluminium": ("6082-aluminium", "ineligible_other_metal"),
    "AISI 304 steel": ("aisi-304-steel", "ineligible_qualified_steel"),
    "AISI 316 steel": ("aisi-316-steel", "ineligible_qualified_steel"),
    "B10 bronze": ("b10-bronze", "ineligible_other_metal"),
    "B12 bronze": ("b12-bronze", "ineligible_other_metal"),
    "BrAl": ("bral", "ineligible_other_metal"),
    "C45E steel": ("c45e-steel", "ineligible_qualified_steel"),
    "Copper": ("copper", "ineligible_other_metal"),
    "CW614 brass": ("cw614-brass", "ineligible_other_metal"),
    "Fe37 steel": ("fe37-steel", "ineligible_qualified_steel"),
    "Nylon 6": ("nylon-6", "non_metal"),
    "Polizene": ("polizene", "non_metal"),
    "Pom-C": ("pom-c", "non_metal"),
    "Teflon": ("teflon", "non_metal"),
    "X150 steel": ("x150-steel", "ineligible_qualified_steel"),
}

PROFILE_KEYS = {
    "access_policy",
    "authority",
    "baseline",
    "batch_policy",
    "claim",
    "dependency_bindings",
    "expected_result",
    "increment",
    "leads",
    "minimums",
    "schema",
}
BASELINE_KEYS = {
    "audit_receipt",
    "decision",
    "partition_sha256",
    "power",
    "projects",
    "projects_sha256",
    "report_receipt",
    "target_policy",
}
RECEIPT_KEYS = {"bytes", "sha256"}
POWER_KEYS = {"exact_steel_groups", "non_metal_groups", "project_revisions"}
PROJECT_KEYS = {
    "exact_steel_groups",
    "non_metal_groups",
    "origin",
    "project_revision_id",
}
DEPENDENCY_KEYS = {"bytes", "path", "sha256"}
INCREMENT_KEYS = {
    "license_expression",
    "origin",
    "primary_sources",
    "project_revision_id",
    "recording_repetitions_per_sample",
    "samples",
    "source_revision",
}
SAMPLE_KEYS = {
    "depth_cm",
    "length_cm",
    "mass_g",
    "power_relation",
    "sample_id",
    "source_label",
    "thickness_cm",
}
LEAD_KEYS = {
    "declared_exact_steel_groups",
    "declared_non_metal_groups",
    "disposition",
    "lead_id",
    "primary_urls",
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


class SourceFrontierError(RuntimeError):
    """The frozen V39 F1 metadata frontier cannot be reproduced."""


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
        raise SourceFrontierError(str(error)) from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SourceFrontierError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise SourceFrontierError(
            f"{context} fields changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_identifier(value: Any, context: str) -> str:
    if not isinstance(value, str) or not IDENTIFIER_PATTERN.fullmatch(value):
        raise SourceFrontierError(f"{context} must be a bounded identifier")
    return value


def require_bounded_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not 1 <= len(value) <= 255:
        raise SourceFrontierError(f"{context} must be a bounded string")
    return value


def require_hash(value: Any, context: str) -> str:
    if not isinstance(value, str) or not HASH_PATTERN.fullmatch(value):
        raise SourceFrontierError(f"{context} must be a lowercase SHA-256")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise SourceFrontierError(f"{context} must be a non-negative integer")
    return value


def require_positive_decimal_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9]+\.[0-9]+", value):
        raise SourceFrontierError(f"{context} must be a positive decimal string")
    try:
        parsed = Decimal(value)
    except InvalidOperation as error:
        raise SourceFrontierError(f"{context} is not decimal") from error
    if not parsed.is_finite() or parsed <= 0:
        raise SourceFrontierError(f"{context} must be finite and positive")
    return value


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    if path.is_symlink() or not path.is_file():
        raise SourceFrontierError("profile must be a regular non-symlink file")
    if not 0 < path.stat().st_size <= MAX_PROFILE_BYTES:
        raise SourceFrontierError("profile size is outside the frozen limit")
    data = path.read_bytes()
    try:
        value = q1a.parse_json(data, "F1 profile")
    except q1a.Q1ASourceGrowthError as error:
        raise SourceFrontierError(str(error)) from error
    if canonical_json(value) != data:
        raise SourceFrontierError("F1 profile must use canonical JSON")
    return data, value


def bound_repository_file(path_text: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink() or not path.is_file():
        raise SourceFrontierError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    return {"bytes": len(data), "path": path_text, "sha256": sha256_bytes(data)}


def validate_receipt(value: Any, expected: dict[str, Any], context: str) -> None:
    receipt = require_exact_keys(value, RECEIPT_KEYS, context)
    require_nonnegative_integer(receipt["bytes"], f"{context}.bytes")
    require_hash(receipt["sha256"], f"{context}.sha256")
    if receipt != expected:
        raise SourceFrontierError(f"{context} changed")


def validate_dependencies(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        raise SourceFrontierError("dependency bindings must be an array")
    checked = []
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, DEPENDENCY_KEYS, f"dependencies[{index}]")
        require_identifier(item["path"], f"dependencies[{index}].path")
        require_hash(item["sha256"], f"dependencies[{index}].sha256")
        require_nonnegative_integer(item["bytes"], f"dependencies[{index}].bytes")
        checked.append(item)
    if [item["path"] for item in checked] != list(DEPENDENCY_PATHS):
        raise SourceFrontierError("dependency path roster or order changed")
    for item in checked:
        if item != bound_repository_file(item["path"]):
            raise SourceFrontierError(f"bound dependency drift: {item['path']}")
    return checked


def validate_projects(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list) or not 1 <= len(value) <= MAX_PROJECTS:
        raise SourceFrontierError("baseline projects must be a bounded array")
    checked = []
    ids = []
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, PROJECT_KEYS, f"projects[{index}]")
        project_id = require_identifier(
            item["project_revision_id"], f"projects[{index}].project_revision_id"
        )
        require_nonnegative_integer(
            item["exact_steel_groups"], f"projects[{index}].exact_steel_groups"
        )
        require_nonnegative_integer(
            item["non_metal_groups"], f"projects[{index}].non_metal_groups"
        )
        if item["origin"] not in {"q0-m", "q1a-internet-metadata"}:
            raise SourceFrontierError("baseline project origin changed")
        checked.append(item)
        ids.append(project_id)
    if ids != sorted(set(ids)):
        raise SourceFrontierError("baseline project parents must be unique and sorted")
    if sha256_bytes(canonical_json(checked)) != BASELINE_PROJECTS_SHA256:
        raise SourceFrontierError("V30 baseline project projection drift")
    return checked


def validate_baseline(value: Any) -> list[dict[str, Any]]:
    baseline = require_exact_keys(value, BASELINE_KEYS, "baseline")
    validate_receipt(baseline["audit_receipt"], BASELINE_AUDIT_RECEIPT, "audit receipt")
    validate_receipt(
        baseline["report_receipt"], BASELINE_REPORT_RECEIPT, "report receipt"
    )
    if baseline["decision"] != BASELINE_DECISION:
        raise SourceFrontierError("V30 baseline decision changed")
    if baseline["partition_sha256"] != BASELINE_PARTITION_SHA256:
        raise SourceFrontierError("V30 partition receipt changed")
    if baseline["projects_sha256"] != BASELINE_PROJECTS_SHA256:
        raise SourceFrontierError("V30 project receipt changed")
    if baseline["power"] != BASELINE_POWER:
        raise SourceFrontierError("V30 baseline power changed")
    if baseline["target_policy"] != TARGET_POLICY:
        raise SourceFrontierError("exact-Steel target policy changed")
    projects = validate_projects(baseline["projects"])
    actual_power = {
        "exact_steel_groups": sum(row["exact_steel_groups"] for row in projects),
        "non_metal_groups": sum(row["non_metal_groups"] for row in projects),
        "project_revisions": len(projects),
    }
    if actual_power != BASELINE_POWER:
        raise SourceFrontierError("baseline project power does not close its receipt")
    return projects


def validate_url_list(value: Any, context: str) -> list[str]:
    if not isinstance(value, list) or not value:
        raise SourceFrontierError(f"{context} must be a non-empty URL array")
    for index, url in enumerate(value):
        if not isinstance(url, str) or not url.startswith("https://"):
            raise SourceFrontierError(f"{context}[{index}] must be an HTTPS URL")
    if value != sorted(set(value)):
        raise SourceFrontierError(f"{context} must be unique and sorted")
    return value


def validate_leads(value: Any) -> None:
    if not isinstance(value, list) or len(value) != BATCH_POLICY["max_leads"]:
        raise SourceFrontierError("F1 must preserve the exact three-lead batch")
    ids = []
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, LEAD_KEYS, f"leads[{index}]")
        lead_id = require_identifier(item["lead_id"], f"leads[{index}].lead_id")
        expected = EXPECTED_LEADS.get(lead_id)
        if expected is None:
            raise SourceFrontierError("unknown source lead")
        steel = require_nonnegative_integer(
            item["declared_exact_steel_groups"],
            f"leads[{index}].declared_exact_steel_groups",
        )
        non_metal = require_nonnegative_integer(
            item["declared_non_metal_groups"],
            f"leads[{index}].declared_non_metal_groups",
        )
        if item["disposition"] != expected["disposition"]:
            raise SourceFrontierError("source lead disposition changed")
        if (steel, non_metal) != expected["counts"]:
            raise SourceFrontierError("source lead power changed")
        if validate_url_list(item["primary_urls"], "lead URLs") != expected["urls"]:
            raise SourceFrontierError("source lead URL identity changed")
        if item["disposition"] != "ADMITTED_METADATA_INCREMENT" and (steel or non_metal):
            raise SourceFrontierError("ineligible source lead claims power")
        ids.append(lead_id)
    if ids != sorted(EXPECTED_LEADS):
        raise SourceFrontierError("source lead roster or order changed")
    if sha256_bytes(canonical_json(value)) != EXPECTED_LEADS_SHA256:
        raise SourceFrontierError("source lead evidence hash drift")


def validate_samples(value: Any) -> tuple[list[dict[str, Any]], dict[str, int]]:
    if not isinstance(value, list) or len(value) != len(EXPECTED_SAMPLE_RELATIONS):
        raise SourceFrontierError("IETeasy must preserve exactly 15 physical samples")
    labels = []
    sample_ids = []
    normalized = []
    exact_steel_groups = 0
    non_metal_groups = 0
    for index, raw in enumerate(value):
        item = require_exact_keys(raw, SAMPLE_KEYS, f"samples[{index}]")
        label = require_bounded_string(
            item["source_label"], f"samples[{index}].source_label"
        )
        sample_id = require_identifier(item["sample_id"], f"samples[{index}].sample_id")
        expected = EXPECTED_SAMPLE_RELATIONS.get(label)
        if expected is None:
            raise SourceFrontierError("unknown IETeasy source material label")
        expected_id, expected_relation = expected
        if sample_id != expected_id:
            raise SourceFrontierError("IETeasy sample identity changed")
        if item["power_relation"] != expected_relation:
            raise SourceFrontierError(
                f"material inflation or relation drift for {label}"
            )
        for key in ("depth_cm", "length_cm", "thickness_cm"):
            require_positive_decimal_string(item[key], f"samples[{index}].{key}")
        if type(item["mass_g"]) is not int or item["mass_g"] <= 0:
            raise SourceFrontierError("sample mass must be a positive integer")
        if expected_relation == "non_metal":
            non_metal_groups += 1
            eligible_relation = "non_metal_candidate"
        elif expected_relation == "exact_steel":
            exact_steel_groups += 1
            eligible_relation = "exact_steel_candidate"
        else:
            eligible_relation = "ineligible_other_metal"
        normalized.append(
            {
                "dimensions_cm": {
                    "depth": item["depth_cm"],
                    "length": item["length_cm"],
                    "thickness": item["thickness_cm"],
                },
                "eligible_relation": eligible_relation,
                "mass_g": item["mass_g"],
                "sample_id": sample_id,
                "source_label": label,
            }
        )
        labels.append(label)
        sample_ids.append(sample_id)
    if len(labels) != len(set(labels)) or sample_ids != sorted(set(sample_ids)):
        raise SourceFrontierError("IETeasy samples must be unique and sorted")
    if sha256_bytes(canonical_json(value)) != EXPECTED_SAMPLE_ROSTER_SHA256:
        raise SourceFrontierError("IETeasy physical sample evidence hash drift")
    return normalized, {
        "exact_steel_groups": exact_steel_groups,
        "non_metal_groups": non_metal_groups,
        "physical_sample_groups": len(normalized),
    }


def validate_increment(value: Any) -> tuple[dict[str, Any], list[dict[str, Any]], dict[str, int]]:
    increment = require_exact_keys(value, INCREMENT_KEYS, "increment")
    if increment["license_expression"] != "CC-BY-4.0":
        raise SourceFrontierError("IETeasy license expression changed")
    if increment["origin"] != "v38-s0-internet-metadata":
        raise SourceFrontierError("IETeasy origin changed")
    if increment["project_revision_id"] != (
        "mendeley-data--10.17632-srfp7x6wxm--v1"
    ):
        raise SourceFrontierError("IETeasy project parent changed")
    if increment["source_revision"] != "10.17632/srfp7x6wxm.1":
        raise SourceFrontierError("IETeasy source revision changed")
    if increment["recording_repetitions_per_sample"] != 10:
        raise SourceFrontierError("IETeasy repetition count changed")
    if validate_url_list(increment["primary_sources"], "primary sources") != (
        EXPECTED_PRIMARY_SOURCES
    ):
        raise SourceFrontierError("IETeasy primary-source identity changed")
    normalized, counts = validate_samples(increment["samples"])
    if counts != {
        "exact_steel_groups": 0,
        "non_metal_groups": 4,
        "physical_sample_groups": 15,
    }:
        raise SourceFrontierError("IETeasy derived source power changed")
    return increment, normalized, counts


def validate_profile(profile_bytes: bytes, value: Any) -> dict[str, Any]:
    profile = require_exact_keys(value, PROFILE_KEYS, "profile")
    if profile["schema"] != PROFILE_SCHEMA:
        raise SourceFrontierError("unknown F1 profile schema")
    if profile["claim"] != CLAIM:
        raise SourceFrontierError("F1 claim changed")
    for key, expected in (
        ("access_policy", ACCESS_POLICY),
        ("authority", AUTHORITY),
        ("batch_policy", BATCH_POLICY),
        ("expected_result", EXPECTED_RESULT),
        ("minimums", MINIMUMS),
    ):
        if profile[key] != expected:
            raise SourceFrontierError(f"profile {key} changed")
    validate_dependencies(profile["dependency_bindings"])
    validate_baseline(profile["baseline"])
    validate_leads(profile["leads"])
    validate_increment(profile["increment"])
    del profile_bytes
    return profile


def zero_forbidden_access() -> dict[str, int]:
    return {key: 0 for key in FORBIDDEN_COUNTERS}


def role_b_deficits(partition: dict[str, Any]) -> tuple[int, int, int]:
    if partition.get("feasible") is not False:
        raise SourceFrontierError("F1 unexpectedly found a feasible partition")
    frontier = partition.get("best_frontier")
    if not isinstance(frontier, dict):
        raise SourceFrontierError("planner omitted best frontier")
    role_b = frontier.get("role_b")
    if not isinstance(role_b, dict):
        raise SourceFrontierError("planner omitted role B")
    return (
        role_b.get("exact_steel_deficit"),
        role_b.get("non_metal_deficit"),
        frontier.get("reserved_unprotected_project_count"),
    )


def solve_frontiers(profile: dict[str, Any]) -> dict[str, Any]:
    baseline_projects = profile["baseline"]["projects"]
    baseline_partition = q1a.solve_protected_partition(baseline_projects, MINIMUMS)
    if sha256_bytes(canonical_json(baseline_partition)) != BASELINE_PARTITION_SHA256:
        raise SourceFrontierError("V30 baseline planner result drift")
    if role_b_deficits(baseline_partition) != (6, 27, 5):
        raise SourceFrontierError("V30 baseline deficit no longer reproduces 6/27")

    _, normalized_samples, increment_counts = validate_increment(profile["increment"])
    increment_row = {
        "exact_steel_groups": increment_counts["exact_steel_groups"],
        "non_metal_groups": increment_counts["non_metal_groups"],
        "origin": profile["increment"]["origin"],
        "project_revision_id": profile["increment"]["project_revision_id"],
    }
    if increment_row["project_revision_id"] in {
        row["project_revision_id"] for row in baseline_projects
    }:
        raise SourceFrontierError("IETeasy project parent duplicates V30 baseline")
    current_projects = sorted(
        [*baseline_projects, increment_row],
        key=lambda row: row["project_revision_id"],
    )
    if len(current_projects) != len(
        {row["project_revision_id"] for row in current_projects}
    ):
        raise SourceFrontierError("current project parents are not unique")
    current_partition = q1a.solve_protected_partition(current_projects, MINIMUMS)
    current_power = {
        "exact_steel_groups": sum(
            row["exact_steel_groups"] for row in current_projects
        ),
        "non_metal_groups": sum(row["non_metal_groups"] for row in current_projects),
        "project_revisions": len(current_projects),
    }
    current_deficits = role_b_deficits(current_partition)
    observed = {
        "baseline_role_b_exact_steel_deficit": role_b_deficits(baseline_partition)[0],
        "baseline_role_b_non_metal_deficit": role_b_deficits(baseline_partition)[1],
        "current_exact_steel_groups": current_power["exact_steel_groups"],
        "current_non_metal_groups": current_power["non_metal_groups"],
        "current_project_revisions": current_power["project_revisions"],
        "current_role_b_exact_steel_deficit": current_deficits[0],
        "current_role_b_non_metal_deficit": current_deficits[1],
        "decision": "ImprovedFrontier",
        "feasible": current_partition["feasible"],
        "reserved_unprotected_project_count": current_deficits[2],
    }
    if observed != EXPECTED_RESULT:
        raise SourceFrontierError(f"F1 result drift: {observed}")
    return {
        "baseline_partition": baseline_partition,
        "current_partition": current_partition,
        "current_power": current_power,
        "current_projects": current_projects,
        "increment_counts": increment_counts,
        "increment_row": increment_row,
        "normalized_samples": normalized_samples,
        "observed": observed,
    }


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    solved: dict[str, Any],
) -> dict[str, bytes]:
    owner = bound_repository_file(OWNER_PATH)
    dependencies = profile["dependency_bindings"]
    classification = {
        "authority": AUTHORITY,
        "counts": {
            **solved["increment_counts"],
            "recording_repetitions_per_sample": profile["increment"][
                "recording_repetitions_per_sample"
            ],
            "recording_rows_not_independent_groups": (
                solved["increment_counts"]["physical_sample_groups"]
                * profile["increment"]["recording_repetitions_per_sample"]
            ),
        },
        "license_expression": profile["increment"]["license_expression"],
        "primary_sources": profile["increment"]["primary_sources"],
        "project_revision_id": profile["increment"]["project_revision_id"],
        "samples": solved["normalized_samples"],
        "schema": CLASSIFICATION_SCHEMA,
        "source_revision": profile["increment"]["source_revision"],
        "target_policy": TARGET_POLICY,
    }
    classification_bytes = canonical_json(classification)
    frontier = {
        "authority": AUTHORITY,
        "baseline": {
            "audit_receipt": profile["baseline"]["audit_receipt"],
            "partition": solved["baseline_partition"],
            "power": BASELINE_POWER,
            "projects_sha256": BASELINE_PROJECTS_SHA256,
            "report_receipt": profile["baseline"]["report_receipt"],
        },
        "current": {
            "partition": solved["current_partition"],
            "power": solved["current_power"],
            "projects_sha256": sha256_bytes(canonical_json(solved["current_projects"])),
        },
        "decision": "ImprovedFrontier",
        "increment": solved["increment_row"],
        "minimums": MINIMUMS,
        "planner_owner": bound_repository_file(
            "lab/scripts/physical_sound_v29_q1a_source_growth_v1.py"
        ),
        "schema": FRONTIER_SCHEMA,
    }
    frontier_bytes = canonical_json(frontier)
    access = {
        **zero_forbidden_access(),
        "profile_bytes_read": len(profile_bytes),
        "repository_binding_bytes_read": sum(
            item["bytes"] for item in dependencies
        ),
    }
    gates = {
        "baseline_6_27_reproduced": role_b_deficits(
            solved["baseline_partition"]
        )[:2]
        == (6, 27),
        "current_6_23_reproduced": role_b_deficits(solved["current_partition"])[
            :2
        ]
        == (6, 23),
        "exact_three_lead_batch": len(profile["leads"]) == 3,
        "five_projects_reserved": role_b_deficits(solved["current_partition"])[2]
        == 5,
        "ieteasy_one_whole_project": solved["increment_counts"][
            "physical_sample_groups"
        ]
        == 15,
        "material_inflation_absent": solved["increment_counts"][
            "exact_steel_groups"
        ]
        == 0,
        "project_parents_unique": len(solved["current_projects"])
        == len({row["project_revision_id"] for row in solved["current_projects"]}),
        "repetitions_collapsed": classification["counts"][
            "recording_rows_not_independent_groups"
        ]
        == 150,
        "zero_forbidden_access": all(
            access[key] == 0 for key in FORBIDDEN_COUNTERS
        ),
    }
    if not all(gates.values()):
        raise SourceFrontierError("F1 conjunctive gate failed")
    report = {
        "access": access,
        "authority": AUTHORITY,
        "decision": "ImprovedFrontier",
        "gates": gates,
        "hashes": {
            "classification_sha256": sha256_bytes(classification_bytes),
            "dependency_root_sha256": sha256_bytes(canonical_json(dependencies)),
            "frontier_sha256": sha256_bytes(frontier_bytes),
            "owner_sha256": owner["sha256"],
            "profile_sha256": sha256_bytes(profile_bytes),
        },
        "next_authorized_stage": "V39-D0-permanent-disclosed-roster",
        "owner": owner,
        "result": solved["observed"],
        "schema": REPORT_SCHEMA,
    }
    return {
        "classification.json": classification_bytes,
        "frontier.json": frontier_bytes,
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
        raise SourceFrontierError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise SourceFrontierError("output must remain outside the repository")
    if resolved.exists():
        raise SourceFrontierError(f"refusing to replace existing output: {resolved}")
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
                raise SourceFrontierError("output file name must be a flat basename")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(profile_bytes, raw_profile)
    solved = solve_frontiers(profile)
    return publish_directory(
        output,
        build_documents(profile_bytes, profile, solved),
    )


def main() -> None:
    arguments = parse_arguments()
    destination = run(arguments.profile, arguments.output)
    print(destination)


if __name__ == "__main__":
    main()
