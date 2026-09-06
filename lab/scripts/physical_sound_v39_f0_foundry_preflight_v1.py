#!/usr/bin/env python3
"""Validate the target-free V39 F0 two-lane sound-foundry preflight."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_exposure_ledger_v0 as ledger_v0

PROFILE_PATH = "lab/profiles/physical-sound-v39-f0-foundry.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v39_f0_foundry_preflight_v1.py"

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v39-f0-foundry-profile.v1"
ROLE_PLAN_SCHEMA = "nextengine.experimental-physical-sound-v39-f0-role-plan.v1"
ACCESS_LEDGER_SCHEMA = (
    "nextengine.experimental-physical-sound-v39-f0-access-ledger.v1"
)
DESCRIPTOR_SCHEMA = (
    "nextengine.experimental-physical-sound-v39-f0-foundry-descriptor.v1"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v39-f0-report.v1"
CLAIM = (
    "TARGET_FREE_TWO_LANE_FOUNDRY_MECHANICS_ONLY / "
    "NO_SIGNAL_MODEL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)

MAX_INPUT_BYTES = 1024 * 1024
MAX_PROJECTS = 128
HASH_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,191}\Z")

ROLES = (
    "generator_train",
    "generator_development",
    "validator_calibration",
    "generator_method_holdout",
    "validator_qualification",
    "joint_admission_shadow",
)
DISCLOSED_ROLES = (
    "generator_train",
    "generator_development",
    "validator_calibration",
)
PROTECTED_ROLES = (
    "generator_method_holdout",
    "validator_qualification",
    "joint_admission_shadow",
)
ACCESS_COUNTERS = (
    "candidate_output_values_read",
    "feature_values_read",
    "force_sample_values_decoded",
    "model_target_values_read",
    "network_requests",
    "pcm_sample_values_decoded",
    "protected_signal_values_decoded",
    "source_artifact_bytes_read",
)
DEPENDENCY_PATHS = (
    "lab/scripts/physical_sound_dataset_contract_v1.py",
    "lab/scripts/physical_sound_exposure_ledger_v0.py",
    "lab/scripts/physical_sound_research_record_v0.py",
    "tools/xtask/src/physical_sound_registry_command/neural_data_plane.rs",
    "tools/xtask/src/physical_sound_registry_command/neural_data_plane/evidence_record.rs",
    "tools/xtask/src/physical_sound_registry_command/neural_data_plane/row_projection.rs",
)

ROLE_POLICY = {
    "disclosed_never_protected": True,
    "disclosed_roles": list(DISCLOSED_ROLES),
    "object_parent_disjoint": True,
    "partition_unit": "publisher_project_revision",
    "protected_one_shot": True,
    "protected_roles": list(PROTECTED_ROLES),
    "recording_parent_disjoint": True,
    "roles": list(ROLES),
    "whole_project_disjoint": True,
}
EXPERIMENT_BUDGET = {
    "candidate_family_limit": 2,
    "candidate_iterations_per_family_limit": 8,
    "peak_rss_bytes_per_run_limit": 2_147_483_648,
    "protected_experiment_wip_limit": 1,
    "seed_count_per_candidate": 1,
    "train_step_limit": 20_000,
    "wall_seconds_per_run_limit": 1_800,
}
ACCESS_POLICY = {
    "candidate_outputs_allowed": False,
    "model_values_allowed": False,
    "network_allowed": False,
    "outputs_external": True,
    "signal_decode_allowed": False,
    "source_artifact_access_allowed": False,
}
ARTIFACT_POLICY = {
    "candidate_outputs_visible_to_validator_calibration": False,
    "generator_namespace": "external/physical-sound/generator",
    "shared_learned_artifacts_allowed": False,
    "shared_thresholds_allowed": False,
    "validator_namespace": "external/physical-sound/validator",
}
TERMINAL_POLICY = {
    "allowed_outcomes": [
        "FallbackOutOfDomain",
        "MechanicsReady",
        "NoCandidate",
        "Reject",
    ],
    "f0_outcome": "MechanicsReady",
    "failure_publication": "none",
    "next_authorized_stage": "V39-F1-source-frontier-automation",
}
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": "v39_f0_mechanics",
    "model_training_authority": False,
    "product_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_release_authority": False,
}

PROFILE_KEYS = {
    "access_policy",
    "artifact_policy",
    "authority",
    "claim",
    "dependency_bindings",
    "experiment_budget",
    "role_policy",
    "schema",
    "terminal_policy",
}
DEPENDENCY_KEYS = {"bytes", "path", "sha256"}
ROLE_PLAN_KEYS = {
    "authority",
    "fixture_kind",
    "plan_id",
    "profile_sha256",
    "projects",
    "revision",
    "schema",
}
PROJECT_KEYS = {
    "object_parent_sha256s",
    "permanent_disclosed",
    "project_id",
    "project_parent_sha256",
    "protected",
    "provenance_sha256",
    "publisher_id",
    "recording_parent_sha256s",
    "revision_id",
    "role",
    "signal_state",
    "source_kind",
}
ACCESS_LEDGER_KEYS = {
    "authority",
    "counters",
    "entries",
    "plan_sha256",
    "revision",
    "schema",
    "terminal",
}
ACCESS_ENTRY_KEYS = {"counters", "project_parent_sha256", "protected", "role"}
TERMINAL_KEYS = {
    "candidate_frozen",
    "outcome",
    "product_authority",
    "protected_roles_opened",
    "validator_release_frozen",
}


class FoundryPreflightError(RuntimeError):
    """The V39 F0 preflight cannot publish MechanicsReady."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=("fixture", "validate"))
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--role-plan", type=Path)
    parser.add_argument("--access-ledger", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return ledger_v0.canonical_json(value)
    except ledger_v0.ExposureLedgerError as error:
        raise FoundryPreflightError(str(error)) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def synthetic_hash(label: str) -> str:
    return sha256_bytes(f"physical-sound-v39-f0:{label}".encode())


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise FoundryPreflightError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise FoundryPreflightError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_identifier(value: Any, context: str) -> str:
    if not isinstance(value, str) or not IDENTIFIER_PATTERN.fullmatch(value):
        raise FoundryPreflightError(f"{context} must be a bounded identifier")
    return value


def require_hash(value: Any, context: str) -> str:
    if not isinstance(value, str) or not HASH_PATTERN.fullmatch(value):
        raise FoundryPreflightError(f"{context} must be a lowercase SHA-256")
    return value


def require_bool(value: Any, context: str) -> bool:
    if type(value) is not bool:
        raise FoundryPreflightError(f"{context} must be boolean")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise FoundryPreflightError(f"{context} must be a non-negative integer")
    return value


def read_canonical_document(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    if path.is_symlink() or not path.is_file():
        raise FoundryPreflightError(f"{context} must be a regular non-symlink file")
    if path.stat().st_size > MAX_INPUT_BYTES:
        raise FoundryPreflightError(f"{context} exceeds the frozen byte limit")
    data = path.read_bytes()
    try:
        value = ledger_v0.parse_json_bytes(data, context)
    except ledger_v0.ExposureLedgerError as error:
        raise FoundryPreflightError(str(error)) from error
    if not isinstance(value, dict):
        raise FoundryPreflightError(f"{context} must be an object")
    if canonical_json(value) != data:
        raise FoundryPreflightError(f"{context} must use canonical JSON")
    return data, value


def bound_repository_file(path_text: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink() or not path.is_file():
        raise FoundryPreflightError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    return {"bytes": len(data), "path": path_text, "sha256": sha256_bytes(data)}


def validate_profile(profile_bytes: bytes, value: Any) -> dict[str, Any]:
    profile = require_exact_keys(value, PROFILE_KEYS, "profile")
    if profile["schema"] != PROFILE_SCHEMA:
        raise FoundryPreflightError("unknown foundry profile schema")
    if profile["claim"] != CLAIM:
        raise FoundryPreflightError("foundry claim changed")
    for key, expected in (
        ("role_policy", ROLE_POLICY),
        ("experiment_budget", EXPERIMENT_BUDGET),
        ("access_policy", ACCESS_POLICY),
        ("artifact_policy", ARTIFACT_POLICY),
        ("terminal_policy", TERMINAL_POLICY),
        ("authority", AUTHORITY),
    ):
        if profile[key] != expected:
            raise FoundryPreflightError(f"profile {key} changed")

    bindings = profile["dependency_bindings"]
    if not isinstance(bindings, list):
        raise FoundryPreflightError("dependency bindings must be an array")
    checked = []
    for index, raw in enumerate(bindings):
        binding = require_exact_keys(raw, DEPENDENCY_KEYS, f"dependencies[{index}]")
        path_text = require_identifier(binding["path"], f"dependencies[{index}].path")
        require_hash(binding["sha256"], f"dependencies[{index}].sha256")
        require_nonnegative_integer(binding["bytes"], f"dependencies[{index}].bytes")
        checked.append(binding)
    paths = [binding["path"] for binding in checked]
    if paths != list(DEPENDENCY_PATHS):
        raise FoundryPreflightError("dependency binding paths changed or are not sorted")
    for binding in checked:
        actual = bound_repository_file(binding["path"])
        if binding != actual:
            raise FoundryPreflightError(f"bound file drift: {binding['path']}")
    del profile_bytes
    return profile


def project_identity(project: dict[str, Any]) -> dict[str, str]:
    return {
        "project_id": project["project_id"],
        "publisher_id": project["publisher_id"],
        "revision_id": project["revision_id"],
    }


def validate_hash_list(value: Any, context: str) -> list[str]:
    if not isinstance(value, list) or not value:
        raise FoundryPreflightError(f"{context} must be a non-empty array")
    for index, item in enumerate(value):
        require_hash(item, f"{context}[{index}]")
    if value != sorted(set(value)):
        raise FoundryPreflightError(f"{context} must be unique and sorted")
    return value


def validate_role_plan(
    plan_bytes: bytes,
    value: Any,
    profile_sha256: str,
) -> dict[str, Any]:
    plan = require_exact_keys(value, ROLE_PLAN_KEYS, "role plan")
    if plan["schema"] != ROLE_PLAN_SCHEMA:
        raise FoundryPreflightError("unknown role-plan schema")
    require_identifier(plan["plan_id"], "role plan id")
    require_identifier(plan["revision"], "role plan revision")
    if plan["profile_sha256"] != profile_sha256:
        raise FoundryPreflightError("role plan profile hash changed")
    if plan["fixture_kind"] != "synthetic_target_free":
        raise FoundryPreflightError("F0 accepts only a synthetic target-free plan")
    if plan["authority"] != AUTHORITY:
        raise FoundryPreflightError("role-plan authority changed")
    projects = plan["projects"]
    if not isinstance(projects, list) or not 1 <= len(projects) <= MAX_PROJECTS:
        raise FoundryPreflightError(f"project count must be in 1..={MAX_PROJECTS}")

    project_hashes: list[str] = []
    object_hashes: set[str] = set()
    recording_hashes: set[str] = set()
    role_counts = {role: 0 for role in ROLES}
    for index, raw in enumerate(projects):
        item = require_exact_keys(raw, PROJECT_KEYS, f"projects[{index}]")
        for field in ("publisher_id", "project_id", "revision_id"):
            require_identifier(item[field], f"projects[{index}].{field}")
        require_hash(item["provenance_sha256"], f"projects[{index}].provenance")
        project_hash = require_hash(
            item["project_parent_sha256"], f"projects[{index}].project_parent"
        )
        expected = sha256_bytes(canonical_json(project_identity(item)))
        if project_hash != expected:
            raise FoundryPreflightError("project parent hash changed")
        role = item["role"]
        if role not in ROLES:
            raise FoundryPreflightError("unknown foundry role")
        protected = require_bool(item["protected"], f"projects[{index}].protected")
        permanent = require_bool(
            item["permanent_disclosed"],
            f"projects[{index}].permanent_disclosed",
        )
        if protected != (role in PROTECTED_ROLES):
            raise FoundryPreflightError("role protected flag changed")
        if permanent != (role in DISCLOSED_ROLES):
            raise FoundryPreflightError("disclosed/protected permanence changed")
        if item["signal_state"] != "sealed":
            raise FoundryPreflightError("F0 project signal must remain sealed")
        if item["source_kind"] != "synthetic_fixture":
            raise FoundryPreflightError("F0 project must remain synthetic fixture")
        for digest in validate_hash_list(
            item["object_parent_sha256s"], f"projects[{index}].object_parents"
        ):
            if digest in object_hashes:
                raise FoundryPreflightError("object parent crosses foundry roles")
            object_hashes.add(digest)
        for digest in validate_hash_list(
            item["recording_parent_sha256s"], f"projects[{index}].recording_parents"
        ):
            if digest in recording_hashes:
                raise FoundryPreflightError("recording parent crosses foundry roles")
            recording_hashes.add(digest)
        project_hashes.append(project_hash)
        role_counts[role] += 1
    if project_hashes != sorted(project_hashes):
        raise FoundryPreflightError("projects must be sorted by project parent hash")
    if len(project_hashes) != len(set(project_hashes)):
        raise FoundryPreflightError("project revision crosses foundry roles")
    missing_roles = [role for role, count in role_counts.items() if count == 0]
    if missing_roles:
        raise FoundryPreflightError(f"role plan omits roles: {missing_roles}")
    del plan_bytes
    return plan


def zero_counters() -> dict[str, int]:
    return {name: 0 for name in ACCESS_COUNTERS}


def validate_counters(value: Any, context: str) -> dict[str, int]:
    counters = require_exact_keys(value, set(ACCESS_COUNTERS), context)
    result = {}
    for name in ACCESS_COUNTERS:
        result[name] = require_nonnegative_integer(counters[name], f"{context}.{name}")
        if result[name] != 0:
            raise FoundryPreflightError(f"F0 access must remain zero: {name}")
    return result


def validate_access_ledger(
    ledger_bytes: bytes,
    value: Any,
    plan_bytes: bytes,
    plan: dict[str, Any],
) -> dict[str, Any]:
    ledger = require_exact_keys(value, ACCESS_LEDGER_KEYS, "access ledger")
    if ledger["schema"] != ACCESS_LEDGER_SCHEMA:
        raise FoundryPreflightError("unknown F0 access-ledger schema")
    require_identifier(ledger["revision"], "access ledger revision")
    if ledger["plan_sha256"] != sha256_bytes(plan_bytes):
        raise FoundryPreflightError("access ledger plan hash changed")
    if ledger["authority"] != AUTHORITY:
        raise FoundryPreflightError("access-ledger authority changed")
    projects = {item["project_parent_sha256"]: item for item in plan["projects"]}
    entries = ledger["entries"]
    if not isinstance(entries, list) or len(entries) != len(projects):
        raise FoundryPreflightError("access ledger must cover every project exactly once")
    observed: list[str] = []
    aggregate = zero_counters()
    for index, raw in enumerate(entries):
        entry = require_exact_keys(raw, ACCESS_ENTRY_KEYS, f"entries[{index}]")
        digest = require_hash(
            entry["project_parent_sha256"], f"entries[{index}].project_parent"
        )
        project = projects.get(digest)
        if project is None:
            raise FoundryPreflightError("access ledger references unknown project")
        if entry["role"] != project["role"]:
            raise FoundryPreflightError("access ledger role changed")
        if require_bool(entry["protected"], f"entries[{index}].protected") != project[
            "protected"
        ]:
            raise FoundryPreflightError("access ledger protected flag changed")
        counters = validate_counters(entry["counters"], f"entries[{index}].counters")
        for name in ACCESS_COUNTERS:
            aggregate[name] += counters[name]
        observed.append(digest)
    if observed != sorted(observed) or len(observed) != len(set(observed)):
        raise FoundryPreflightError("access entries must be unique and sorted")
    global_counters = validate_counters(ledger["counters"], "access ledger counters")
    if global_counters != aggregate:
        raise FoundryPreflightError("access ledger aggregate counters changed")
    terminal = require_exact_keys(ledger["terminal"], TERMINAL_KEYS, "terminal")
    if terminal != {
        "candidate_frozen": False,
        "outcome": "MechanicsReady",
        "product_authority": False,
        "protected_roles_opened": [],
        "validator_release_frozen": False,
    }:
        raise FoundryPreflightError("F0 terminal state changed")
    del ledger_bytes
    return ledger


def synthetic_project(role: str) -> dict[str, Any]:
    token = role.replace("_", "-")
    item = {
        "object_parent_sha256s": [synthetic_hash(f"object:{token}")],
        "permanent_disclosed": role in DISCLOSED_ROLES,
        "project_id": f"fixture-{token}",
        "project_parent_sha256": "",
        "protected": role in PROTECTED_ROLES,
        "provenance_sha256": synthetic_hash(f"provenance:{token}"),
        "publisher_id": "nextengine-synthetic",
        "recording_parent_sha256s": [synthetic_hash(f"recording:{token}")],
        "revision_id": "v1",
        "role": role,
        "signal_state": "sealed",
        "source_kind": "synthetic_fixture",
    }
    item["project_parent_sha256"] = sha256_bytes(canonical_json(project_identity(item)))
    return item


def synthetic_role_plan(profile_sha256: str) -> dict[str, Any]:
    return {
        "authority": AUTHORITY,
        "fixture_kind": "synthetic_target_free",
        "plan_id": "physical-sound-v39-f0-target-free-fixture",
        "profile_sha256": profile_sha256,
        "projects": sorted(
            (synthetic_project(role) for role in ROLES),
            key=lambda item: item["project_parent_sha256"],
        ),
        "revision": "v1",
        "schema": ROLE_PLAN_SCHEMA,
    }


def synthetic_access_ledger(plan_bytes: bytes, plan: dict[str, Any]) -> dict[str, Any]:
    return {
        "authority": AUTHORITY,
        "counters": zero_counters(),
        "entries": [
            {
                "counters": zero_counters(),
                "project_parent_sha256": item["project_parent_sha256"],
                "protected": item["protected"],
                "role": item["role"],
            }
            for item in plan["projects"]
        ],
        "plan_sha256": sha256_bytes(plan_bytes),
        "revision": "v1",
        "schema": ACCESS_LEDGER_SCHEMA,
        "terminal": {
            "candidate_frozen": False,
            "outcome": "MechanicsReady",
            "product_authority": False,
            "protected_roles_opened": [],
            "validator_release_frozen": False,
        },
    }


def report_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    plan_bytes: bytes,
    plan: dict[str, Any],
    ledger_bytes: bytes,
    ledger: dict[str, Any],
) -> dict[str, bytes]:
    owner = bound_repository_file(OWNER_PATH)
    dependencies = profile["dependency_bindings"]
    descriptor = {
        "access_policy": profile["access_policy"],
        "artifact_policy": profile["artifact_policy"],
        "authority": AUTHORITY,
        "claim": CLAIM,
        "dependency_root_sha256": sha256_bytes(canonical_json(dependencies)),
        "experiment_budget": profile["experiment_budget"],
        "role_policy": profile["role_policy"],
        "schema": DESCRIPTOR_SCHEMA,
        "terminal_policy": profile["terminal_policy"],
    }
    descriptor_bytes = canonical_json(descriptor)
    role_counts = {
        role: sum(item["role"] == role for item in plan["projects"]) for role in ROLES
    }
    report = {
        "access_counters": ledger["counters"],
        "authority": AUTHORITY,
        "counts": {
            "object_parent_count": sum(
                len(item["object_parent_sha256s"]) for item in plan["projects"]
            ),
            "project_count": len(plan["projects"]),
            "recording_parent_count": sum(
                len(item["recording_parent_sha256s"]) for item in plan["projects"]
            ),
            "role_project_counts": role_counts,
        },
        "decision": "MechanicsReady",
        "hashes": {
            "access_ledger_sha256": sha256_bytes(ledger_bytes),
            "descriptor_sha256": sha256_bytes(descriptor_bytes),
            "owner_sha256": owner["sha256"],
            "profile_sha256": sha256_bytes(profile_bytes),
            "role_plan_sha256": sha256_bytes(plan_bytes),
        },
        "invariants": {
            "access_zero": all(value == 0 for value in ledger["counters"].values()),
            "disclosed_permanent": all(
                item["permanent_disclosed"]
                for item in plan["projects"]
                if item["role"] in DISCLOSED_ROLES
            ),
            "learned_artifacts_disjoint": True,
            "parent_groups_disjoint": True,
            "protected_sealed": all(
                item["signal_state"] == "sealed"
                for item in plan["projects"]
                if item["role"] in PROTECTED_ROLES
            ),
            "whole_projects_disjoint": True,
        },
        "next_authorized_stage": "V39-F1-source-frontier-automation",
        "owner": owner,
        "schema": REPORT_SCHEMA,
    }
    if not all(report["invariants"].values()):
        raise FoundryPreflightError("F0 invariant report is not conjunctively true")
    return {
        "access-ledger.json": ledger_bytes,
        "descriptor.json": descriptor_bytes,
        "profile.json": profile_bytes,
        "report.json": canonical_json(report),
        "role-plan.json": plan_bytes,
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
        raise FoundryPreflightError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise FoundryPreflightError("output must remain outside the repository")
    if resolved.exists():
        raise FoundryPreflightError(f"refusing to replace existing output: {resolved}")
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
                raise FoundryPreflightError("output file name must be a flat basename")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def load_and_validate_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    profile_bytes, raw_profile = read_canonical_document(path, "foundry profile")
    return profile_bytes, validate_profile(profile_bytes, raw_profile)


def run_fixture(profile_path: Path, output: Path) -> Path:
    profile_bytes, profile = load_and_validate_profile(profile_path)
    profile_sha256 = sha256_bytes(profile_bytes)
    plan = synthetic_role_plan(profile_sha256)
    plan_bytes = canonical_json(plan)
    validate_role_plan(plan_bytes, plan, profile_sha256)
    ledger = synthetic_access_ledger(plan_bytes, plan)
    ledger_bytes = canonical_json(ledger)
    validate_access_ledger(ledger_bytes, ledger, plan_bytes, plan)
    return publish_directory(
        output,
        report_documents(
            profile_bytes,
            profile,
            plan_bytes,
            plan,
            ledger_bytes,
            ledger,
        ),
    )


def run_validate(
    profile_path: Path,
    role_plan_path: Path,
    access_ledger_path: Path,
    output: Path,
) -> Path:
    profile_bytes, profile = load_and_validate_profile(profile_path)
    plan_bytes, raw_plan = read_canonical_document(role_plan_path, "role plan")
    plan = validate_role_plan(plan_bytes, raw_plan, sha256_bytes(profile_bytes))
    ledger_bytes, raw_ledger = read_canonical_document(
        access_ledger_path, "F0 access ledger"
    )
    ledger = validate_access_ledger(ledger_bytes, raw_ledger, plan_bytes, plan)
    return publish_directory(
        output,
        report_documents(
            profile_bytes,
            profile,
            plan_bytes,
            plan,
            ledger_bytes,
            ledger,
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    if arguments.stage == "fixture":
        if arguments.role_plan is not None or arguments.access_ledger is not None:
            raise FoundryPreflightError("fixture stage does not accept plan or ledger")
        destination = run_fixture(arguments.profile, arguments.output)
    else:
        if arguments.role_plan is None or arguments.access_ledger is None:
            raise FoundryPreflightError("validate stage requires role plan and access ledger")
        destination = run_validate(
            arguments.profile,
            arguments.role_plan,
            arguments.access_ledger,
            arguments.output,
        )
    print(destination)


if __name__ == "__main__":
    main()
