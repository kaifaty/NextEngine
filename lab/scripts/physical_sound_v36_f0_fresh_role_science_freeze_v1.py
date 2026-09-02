#!/usr/bin/env python3
"""Freeze fresh V36 role identities while preserving V35 science exactly."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1 as v35
import physical_sound_v36_owner_contract_v1 as owner_contract

PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v36-f0-fresh-role-"
    "unchanged-science-profile.v1"
)
PROFILE_ID = "physical-sound-v36-f0-fresh-role-unchanged-science-v1"
PROFILE_SHA256 = "6c71249f4cf999bbed14a5e311fdd7d04776722af1ce0943a2b00012690eb3a8"
BASELINE_COMMIT = "ae75a9905f9181344b84c5d6e14c98c821ab3375"
CLAIM = (
    "TARGET_SAFE_FRESH_ROLE_AND_UNCHANGED_SCIENCE_PROFILE_ONLY / "
    "NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_"
    "DEMO_OR_RUNTIME_AUTHORITY"
)
OWNER_PATH = "lab/scripts/physical_sound_v36_f0_fresh_role_science_freeze_v1.py"
PROFILE_PATH = "lab/profiles/physical-sound-v36-f0-fresh-role-unchanged-science.v1.json"
MAX_PROFILE_BYTES = 1_048_576
ZERO_ACCESS = {
    "development_target_rows": 0,
    "feature_rows_materialized": 0,
    "fresh_v36_target_rows": 0,
    "method_holdout_target_rows": 0,
    "model_parameters_initialized": 0,
    "network_requests": 0,
    "oracle_values_evaluated": 0,
    "prior_generation_metric_values_read": 0,
    "prior_generation_prediction_values_read": 0,
    "prior_generation_target_values_read": 0,
    "prior_generation_weight_values_read": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "train_target_rows": 0,
}
EXPECTED_AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "c0_allowed_after_f0": True,
    "d0_allowed_after_f0": False,
    "e0_allowed_after_f0": False,
    "external_research_only": True,
    "h0_allowed_after_f0": False,
    "model_allowed_after_f0": False,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "validator_release_authority": False,
    "x0_allowed_after_f0": False,
}
IDENTITY_DIFFERENCE_PATHS = [
    "corpus.contacts",
    "corpus.geometry_multiplier_cells",
    "corpus.materials",
    "corpus.role_identity_prefix",
    "oracle.contact.expression",
    "oracle.decay.expression",
    "oracle.global_gain.expression",
    "witness_contract.b0_train_permutation_seeds",
]


class F0FreezeError(RuntimeError):
    """The target-safe V36 F0 freeze contract failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise F0FreezeError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise F0FreezeError("profile must not be a symlink")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise F0FreezeError("profile size outside bound")
    try:
        profile = json.loads(data)
    except json.JSONDecodeError as error:
        raise F0FreezeError(f"invalid profile JSON: {error}") from error
    if data != canonical_json(profile):
        raise F0FreezeError("profile is not canonical JSON")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise F0FreezeError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or profile.get("authority") != EXPECTED_AUTHORITY
    ):
        raise F0FreezeError("profile identity or authority mismatch")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    declarations = {**profile["parent"], "protocol": profile["protocol"]}
    root = repository_root()
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, declared in sorted(declarations.items()):
        path = root / declared["path"]
        if path.is_symlink():
            raise F0FreezeError(f"dependency must not be a symlink: {dependency_id}")
        data = path.read_bytes()
        if sha256_bytes(data) != declared["sha256"]:
            raise F0FreezeError(f"dependency drift: {dependency_id}")
        result[dependency_id] = {
            "bytes": len(data),
            "path": declared["path"],
            "sha256": declared["sha256"],
        }
    return result


def load_dependency_json(profile: dict[str, Any], dependency_id: str) -> dict[str, Any]:
    declared = profile["parent"][dependency_id]
    data = (repository_root() / declared["path"]).read_bytes()
    if sha256_bytes(data) != declared["sha256"]:
        raise F0FreezeError(f"dependency drift: {dependency_id}")
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise F0FreezeError(f"dependency JSON invalid: {dependency_id}") from error
    if not isinstance(value, dict):
        raise F0FreezeError(f"dependency JSON must be an object: {dependency_id}")
    return value


def section_sha256(value: Any) -> str:
    return sha256_bytes(canonical_json(value))


def build_experiments(
    profile: dict[str, Any], v35_profile: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any]]:
    v33 = v35.load_declared_json(v35_profile, "v33_profile")
    parent = v35.build_effective_profile(v35_profile, v33)
    parent["witness_contract"] = copy.deepcopy(v35_profile["witness_contract"])

    current = copy.deepcopy(parent)
    fresh = profile["fresh_identity"]
    current["corpus"]["contacts"] = copy.deepcopy(fresh["contacts"])
    current["corpus"]["geometry_multiplier_cells"] = copy.deepcopy(
        fresh["geometry_multiplier_cells"]
    )
    current["corpus"]["materials"] = copy.deepcopy(fresh["materials"])
    current["corpus"]["role_identity_prefix"] = fresh["role_identity_prefix"]
    current["oracle"] = copy.deepcopy(fresh["oracle"])
    current["witness_contract"].update(profile["witness_contract_overlay"])
    current["profile_id"] = PROFILE_ID
    current["schema"] = PROFILE_SCHEMA
    return parent, current


def scientific_projection(experiment: dict[str, Any]) -> dict[str, Any]:
    projection = copy.deepcopy(experiment)
    projection.pop("profile_id", None)
    projection.pop("schema", None)
    corpus = projection["corpus"]
    for field in (
        "contacts",
        "geometry_multiplier_cells",
        "materials",
        "role_identity_prefix",
    ):
        corpus.pop(field, None)
    for branch in ("contact", "decay", "global_gain"):
        projection["oracle"][branch].pop("expression")
    projection["witness_contract"].pop("b0_train_permutation_seeds")
    return projection


def validate_unchanged_science(
    profile: dict[str, Any],
    parent: dict[str, Any],
    current: dict[str, Any],
) -> dict[str, Any]:
    inheritance = profile["scientific_inheritance"]
    expected_inheritance = {
        "identity_difference_paths": IDENTITY_DIFFERENCE_PATHS,
        "model_seed_policy": "inherit-v35-exactly",
        "parent_profile": "parent.v35_f0_profile",
        "projection_rule": (
            "remove-only-identity-difference-paths-then-canonical-compare"
        ),
        "truth_structure_policy": (
            "inherit-v35-bounds-components-and-transcendentals-exactly"
        ),
    }
    if inheritance != expected_inheritance:
        raise F0FreezeError("scientific inheritance contract drift")
    parent_projection = scientific_projection(parent)
    current_projection = scientific_projection(current)
    if current_projection != parent_projection:
        raise F0FreezeError("V36 scientific projection differs from V35")

    parent_truth = copy.deepcopy(parent["oracle"])
    current_truth = copy.deepcopy(current["oracle"])
    changed_expressions = 0
    for branch in ("contact", "decay", "global_gain"):
        if current_truth[branch]["expression"] == parent_truth[branch]["expression"]:
            raise F0FreezeError(f"truth expression was reused: {branch}")
        changed_expressions += 1
        current_truth[branch].pop("expression")
        parent_truth[branch].pop("expression")
    if current_truth != parent_truth:
        raise F0FreezeError("truth structure changed outside coefficient expressions")

    model_seeds = {
        "candidate_initializer": current["method_overlay"]["candidate"]["model"][
            "initializer"
        ]["seed"],
        "raw_mlp_control": current["method_overlay"]["controls"]["raw_mlp"]["seed"],
        "training_candidate": current["training"]["candidate_seed"],
        "training_raw_mlp_control": current["training"]["raw_mlp_control_seed"],
        "v34_shaped_control": current["method_overlay"]["controls"][
            "v34_shaped_spectral_mlp"
        ]["seed"],
    }
    if model_seeds != {
        "candidate_initializer": 3301,
        "raw_mlp_control": 3302,
        "training_candidate": 3301,
        "training_raw_mlp_control": 3302,
        "v34_shaped_control": 3301,
    }:
        raise F0FreezeError("unchanged model/training seed policy drift")
    return {
        "changed_truth_expressions": changed_expressions,
        "identity_difference_paths": IDENTITY_DIFFERENCE_PATHS,
        "model_seeds": model_seeds,
        "projection_bytes": len(canonical_json(current_projection)),
        "projection_sha256": section_sha256(current_projection),
        "status": "SEMANTICALLY_EXACT_TO_V35_OUTSIDE_IDENTITY_PATHS",
        "truth_structure_sha256": section_sha256(current_truth),
    }


def parent_identity_sets(
    parents: dict[str, dict[str, Any]],
) -> tuple[set[tuple[str, str]], set[tuple[str, ...]], set[bytes], set[str], set[int]]:
    contacts: set[tuple[str, str]] = set()
    cells: set[tuple[str, ...]] = set()
    materials: set[bytes] = set()
    truth_expressions: set[str] = set()
    seeds: set[int] = set()

    def collect_seeds(value: Any, key: str = "") -> None:
        if isinstance(value, dict):
            for child_key, child in value.items():
                collect_seeds(child, child_key)
        elif isinstance(value, list):
            for child in value:
                collect_seeds(child, key)
        elif "seed" in key and isinstance(value, int) and not isinstance(value, bool):
            seeds.add(value)

    for parent in parents.values():
        contacts.update(v35.all_parent_contacts(parent))
        cells.update(v35.parent_cells(parent))
        materials.update(v35.parent_materials(parent))
        oracle = v35.parent_oracle(parent)
        if isinstance(oracle, dict):
            for branch in ("contact", "decay", "global_gain"):
                branch_value = oracle.get(branch)
                if isinstance(branch_value, dict):
                    expression = branch_value.get("expression")
                    if isinstance(expression, str):
                        truth_expressions.add(expression)
        collect_seeds(parent)
    return contacts, cells, materials, truth_expressions, seeds


def flatten_identity_seeds(profile: dict[str, Any]) -> tuple[int, ...]:
    seed_identity = profile["fresh_identity"]["seed_identity"]
    values = [seed_identity["case_enumeration"]]
    values.extend(seed_identity["role_by_role"].values())
    values.extend(seed_identity["truth_by_branch"].values())
    values.extend(profile["witness_contract_overlay"]["b0_train_permutation_seeds"])
    if any(isinstance(value, bool) or not isinstance(value, int) for value in values):
        raise F0FreezeError("identity seed must be an integer")
    return tuple(values)


def validate_fresh_identities(
    profile: dict[str, Any],
    current: dict[str, Any],
    parents: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    corpus = current["corpus"]
    contact_groups = v35.contact_pairs(corpus)
    current_contacts = {pair for group in contact_groups.values() for pair in group}
    if sum(map(len, contact_groups.values())) != len(current_contacts):
        raise F0FreezeError("V36 contact roles overlap")
    if {name: len(group) for name, group in contact_groups.items()} != {
        "development": 6,
        "method_holdout": 6,
        "train": 12,
    }:
        raise F0FreezeError("V36 contact counts drift")
    for role, group in contact_groups.items():
        if len([pair for pair in group if pair[0] == "0"]) != 1:
            raise F0FreezeError(f"V36 role lacks one exact nodal contact: {role}")
        for pair in group:
            numbers = tuple(float(value) for value in pair)
            if not all(math.isfinite(value) and 0 <= value <= 1 for value in numbers):
                raise F0FreezeError("contact coordinate outside finite unit square")

    current_cells = {tuple(cell) for cell in corpus["geometry_multiplier_cells"]}
    if len(current_cells) != 10 or any(
        not all(math.isfinite(float(value)) and float(value) > 0 for value in cell)
        for cell in current_cells
    ):
        raise F0FreezeError("V36 geometry cells are invalid")
    current_materials = {canonical_json(row) for row in corpus["materials"]}
    if len(current_materials) != 3:
        raise F0FreezeError("V36 material identities are not three unique tuples")
    for material in corpus["materials"]:
        numeric_fields = (
            "density_kg_m3",
            "loss_rate_per_second",
            "poisson_ratio",
            "youngs_modulus_pa",
        )
        if not material["material_id"].startswith("synthetic-") or any(
            not math.isfinite(float(material[field])) or float(material[field]) <= 0
            for field in numeric_fields
        ):
            raise F0FreezeError("V36 material tuple is invalid")

    prior_contacts, prior_cells, prior_materials, prior_truth, prior_seeds = (
        parent_identity_sets(parents)
    )
    if current_contacts & prior_contacts:
        raise F0FreezeError("V36 contact reused a V32-V35 identity")
    if current_cells & prior_cells:
        raise F0FreezeError("V36 geometry reused a V32-V35 identity")
    if current_materials & prior_materials:
        raise F0FreezeError("V36 material reused a V32-V35 identity")
    current_truth = {
        current["oracle"][branch]["expression"]
        for branch in ("contact", "decay", "global_gain")
    }
    if len(current_truth) != 3 or current_truth & prior_truth:
        raise F0FreezeError("V36 truth expression reused a V32-V35 identity")

    seeds = flatten_identity_seeds(profile)
    if len(seeds) != 10 or len(set(seeds)) != len(seeds):
        raise F0FreezeError("V36 identity seeds must be ten unique values")
    if set(seeds) & prior_seeds:
        raise F0FreezeError("V36 identity seed reused a V32-V35 seed")

    prefix = corpus["role_identity_prefix"]
    prior_prefixes = {
        parent.get("corpus_overlay", {}).get("role_identity_prefix")
        or parent.get("profile_id")
        for parent in parents.values()
    }
    if not isinstance(prefix, str) or not prefix.startswith("physical-sound-v36-"):
        raise F0FreezeError("V36 role identity prefix is invalid")
    if prefix in prior_prefixes:
        raise F0FreezeError("V36 role identity prefix was reused")
    return {
        "contact_count_by_role": {
            role: len(group) for role, group in sorted(contact_groups.items())
        },
        "fresh_geometry_cells": len(current_cells),
        "fresh_identity_seeds": len(seeds),
        "fresh_materials": len(current_materials),
        "fresh_truth_expressions": len(current_truth),
        "prior_contact_intersection": 0,
        "prior_geometry_intersection": 0,
        "prior_material_intersection": 0,
        "prior_seed_intersection": 0,
        "prior_truth_intersection": 0,
        "role_identity_prefix": prefix,
    }


def merkle_root(identities: list[str]) -> str:
    if not identities or len(identities) != len(set(identities)):
        raise F0FreezeError("identity set must be nonempty and unique")
    return sha256_bytes(("\n".join(sorted(identities)) + "\n").encode())


def build_role_commitments(
    profile: dict[str, Any], current: dict[str, Any]
) -> dict[str, Any]:
    corpus = current["corpus"]
    contacts = v35.contact_pairs(corpus)
    seed_identity = profile["fresh_identity"]["seed_identity"]
    role_seeds = seed_identity["role_by_role"]
    prefix = corpus["role_identity_prefix"]
    role_cases: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    commitments: dict[str, Any] = {}
    total_cases = 0
    total_rows = 0
    for role in ("train", "development", "method_holdout"):
        structural_cases, strata = v35.enumerate_role_cases(corpus, role)
        case_ids: list[str] = []
        row_ids: list[str] = []
        for entry in corpus["role_plan"][role]:
            for family in corpus["families"]:
                for material in corpus["materials"]:
                    for cell_index in entry["geometry_cells"]:
                        geometry = corpus["geometry_multiplier_cells"][cell_index]
                        for contact in contacts[entry["contact_set"]]:
                            payload = {
                                "contact": contact,
                                "enumeration_seed": seed_identity["case_enumeration"],
                                "family": family,
                                "geometry": geometry,
                                "material": material,
                                "prefix": prefix,
                                "role": role,
                                "role_seed": role_seeds[role],
                                "stratum": entry["stratum"],
                                "truth_seed_by_branch": seed_identity[
                                    "truth_by_branch"
                                ],
                            }
                            case_id = "case-" + section_sha256(payload)
                            case_ids.append(case_id)
                            row_ids.extend(
                                f"{case_id}:mode-{ordinal:02d}"
                                for ordinal in range(corpus["mode_count"])
                            )
        if len(case_ids) != len(structural_cases):
            raise F0FreezeError(f"role case commitment count mismatch: {role}")
        expected = corpus["counts"][role]
        if len(case_ids) != expected["cases"] or len(row_ids) != expected["modal_rows"]:
            raise F0FreezeError(f"role row commitment count mismatch: {role}")
        role_cases[role] = structural_cases
        commitments[role] = {
            "case_count": len(case_ids),
            "case_root_sha256": merkle_root(case_ids),
            "modal_row_count": len(row_ids),
            "modal_row_root_sha256": merkle_root(row_ids),
            "role_seed": role_seeds[role],
            "strata": strata,
        }
        total_cases += len(case_ids)
        total_rows += len(row_ids)
    for left, right in (
        ("train", "development"),
        ("train", "method_holdout"),
        ("development", "method_holdout"),
    ):
        if role_cases[left] & role_cases[right]:
            raise F0FreezeError(f"case identity crosses roles: {left}/{right}")
    if total_cases != 1512 or total_rows != 15120:
        raise F0FreezeError("complete V36 role algebra drift")
    return {
        "case_enumeration_seed": profile["fresh_identity"]["seed_identity"][
            "case_enumeration"
        ],
        "pairwise_role_case_intersections": {
            "development_method_holdout": 0,
            "train_development": 0,
            "train_method_holdout": 0,
        },
        "role_identity_prefix": prefix,
        "roles": commitments,
        "total_cases": total_cases,
        "total_modal_rows": total_rows,
    }


def validate_access_and_owner_contract(profile: dict[str, Any]) -> dict[str, Any]:
    expected_order = [
        "f0-fresh-role-and-unchanged-science-freeze",
        "c0-fresh-zero-target-structural-census",
        "x0-mutation-and-lifecycle-conformance",
        "e0-full-surrogate-rehearsal-and-execution-seal",
        "d0-one-shot-fresh-development",
        "h0-one-shot-method-holdout",
        "terminal-report",
    ]
    if profile["access_order"] != expected_order:
        raise F0FreezeError("V36 access order drift")
    terminal = profile["terminal_publication"]
    expected_outcomes = [
        owner_contract.TerminalDecision.PASS.value,
        owner_contract.TerminalDecision.METRIC_REJECT.value,
        owner_contract.TerminalDecision.HARD_GATE_REJECT.value,
        owner_contract.TerminalDecision.RESOURCE_REJECT.value,
        owner_contract.TerminalDecision.OWNER_FAULT.value,
    ]
    if (
        terminal["post_access_outcomes"] != expected_outcomes
        or terminal["post_access_raise_allowed"] is not False
        or terminal["atomic_staging_required"] is not True
        or terminal["repeat_exact_required"] is not True
        or terminal["pre_access_contract_reject"]
        != {"partial_output_allowed": False, "target_rows": 0}
        or owner_contract.CONTRACT_SCHEMA
        != "nextengine.experimental-physical-sound-v36-owner-contract.v1"
    ):
        raise F0FreezeError("V36 terminal or typed owner contract drift")
    return {
        "access_order": expected_order,
        "contract_schema": owner_contract.CONTRACT_SCHEMA,
        "d0_topology_sha256": owner_contract.expected_topology_sha256(
            owner_contract.PipelineKind.D0
        ),
        "h0_topology_sha256": owner_contract.expected_topology_sha256(
            owner_contract.PipelineKind.H0
        ),
        "post_access_outcomes": expected_outcomes,
    }


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise F0FreezeError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise F0FreezeError("output must be a fresh external path")
    return output


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def build_conformance(
    profile: dict[str, Any], profile_data: bytes
) -> tuple[dict[str, Any], dict[str, Any]]:
    dependencies = validate_dependencies(profile)
    v35_profile = load_dependency_json(profile, "v35_f0_profile")
    v35_profile_path = repository_root() / profile["parent"]["v35_f0_profile"]["path"]
    loaded_v35, _ = v35.load_profile(v35_profile_path)
    if loaded_v35 != v35_profile:
        raise F0FreezeError("V35 scientific parent did not load exactly")
    v32 = load_dependency_json(profile, "v32_profile")
    v33 = load_dependency_json(profile, "v33_profile")
    v34 = load_dependency_json(profile, "v34_profile")
    parent, current = build_experiments(profile, v35_profile)
    science = validate_unchanged_science(profile, parent, current)
    freshness = validate_fresh_identities(
        profile,
        current,
        {"V32": v32, "V33": v33, "V34": v34, "V35": v35_profile},
    )
    commitments = build_role_commitments(profile, current)
    contract = validate_access_and_owner_contract(profile)
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    conformance = {
        "access": ZERO_ACCESS,
        "claim": CLAIM,
        "dependencies": dependencies,
        "freshness": freshness,
        "gates": {
            "authority_narrow": "Pass",
            "canonical_profile": "Pass",
            "fresh_against_v32_v33_v34_v35": "Pass",
            "model_or_target_values": "0 Exact",
            "owner_contract_bound": "Pass",
            "prior_generation_values": "0 Exact",
            "role_commitments_complete": "Pass",
            "science_semantically_exact_to_v35": "Pass",
            "target_safe_access_order": "Pass",
        },
        "official_values_opened": False,
        "owner_contract": contract,
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "profile_identity": {
            "bytes": len(profile_data),
            "path": PROFILE_PATH,
            "sha256": sha256_bytes(profile_data),
        },
        "role_commitment_summary": {
            "role_identity_prefix": commitments["role_identity_prefix"],
            "total_cases": commitments["total_cases"],
            "total_modal_rows": commitments["total_modal_rows"],
        },
        "schema": "nextengine.experimental-physical-sound-v36-f0-conformance.v1",
        "science": science,
        "status": "Pass",
    }
    return conformance, commitments


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v36-f0-", dir=output.parent))
    try:
        conformance, commitments = build_conformance(profile, profile_data)
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        commitments_ref = write_bytes(
            staging, "identity-commitments.json", canonical_json(commitments)
        )
        evidence = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "conformance": conformance_ref,
            "identity_commitments": commitments_ref,
            "owner_identity": conformance["owner_identity"],
            "profile_identity": conformance["profile_identity"],
            "schema": "nextengine.experimental-physical-sound-v36-f0-evidence.v1",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "decision": "F0_FRESH_ROLE_UNCHANGED_SCIENCE_FREEZE_PASS",
            "evidence": evidence_ref,
            "next_authorized_stage": "V36-C0-fresh-zero-target-structural-census",
            "official_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v36-f0-report.v1",
            "status": "Pass",
        }
        write_bytes(staging, "report.json", canonical_json(report))
        staging.replace(output)
        return report
    except Exception:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    try:
        report = run(arguments.profile, arguments.output)
    except (F0FreezeError, OSError, KeyError, TypeError, ValueError) as error:
        print(
            canonical_json(
                {"decision": "CONTRACT_REJECT", "error": str(error)}
            ).decode(),
            end="",
        )
        return 2
    print(
        canonical_json(
            {
                "decision": report["decision"],
                "official_values_opened": report["official_values_opened"],
                "profile_sha256": PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
