#!/usr/bin/env python3
"""Freeze the V35 geometry/contact hybrid without opening target/model values."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import shutil
import statistics
import tempfile
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v35-f0-geometry-conditioned-hybrid-profile.v1"
PROFILE_ID = "physical-sound-v35-f0-geometry-conditioned-hybrid-v1"
PROFILE_SHA256 = "019cadd51254c27da32e35ab39b35551ba965198f33282feb975af83cb9304cc"
BASELINE_COMMIT = "31e9cea3e79e095387ee70d312194962bddc6d7c"
CLAIM = (
    "TARGET_SAFE_SYNTHETIC_GEOMETRY_CONTACT_HYBRID_PROFILE_ONLY / "
    "NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
OWNER_PATH = "lab/scripts/physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1.py"
PROFILE_PATH = "lab/profiles/physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json"
MAX_PROFILE_BYTES = 1_048_576
ZERO_ACCESS = {
    "development_target_rows": 0,
    "feature_rows_materialized": 0,
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


class F0FreezeError(RuntimeError):
    """The target-safe V35 F0 freeze contract failed."""


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
    ):
        raise F0FreezeError("profile identity mismatch")
    required_authority = {
        "admission_authority": False,
        "authored_fallback_required": True,
        "b0_allowed_after_f0": False,
        "c0_allowed_after_f0": True,
        "external_research_only": True,
        "i0_allowed_after_f0": False,
        "model_allowed_after_f0": False,
        "network_allowed": False,
        "quality_authority": False,
        "real_material_authority": False,
        "real_signal_allowed": False,
        "runtime_authority": False,
        "synthetic_only": True,
        "validator_release_authority": False,
    }
    if profile.get("authority") != required_authority:
        raise F0FreezeError("profile authority drift")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    declarations = {
        **profile["parent"],
        "protocol": profile["protocol"],
        "research": profile["research"],
    }
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


def load_declared_json(profile: dict[str, Any], dependency_id: str) -> dict[str, Any]:
    declared = profile["parent"][dependency_id]
    data = (repository_root() / declared["path"]).read_bytes()
    if sha256_bytes(data) != declared["sha256"]:
        raise F0FreezeError(f"dependency drift: {dependency_id}")
    try:
        return json.loads(data)
    except json.JSONDecodeError as error:
        raise F0FreezeError(f"dependency JSON invalid: {dependency_id}") from error


def section_sha256(value: Any) -> str:
    return sha256_bytes(canonical_json(value))


def validate_preserved_sections(
    profile: dict[str, Any], v33_profile: dict[str, Any]
) -> dict[str, str]:
    sections = {
        "corpus_counts": v33_profile["corpus"]["counts"],
        "corpus_families": v33_profile["corpus"]["families"],
        "corpus_role_plan": v33_profile["corpus"]["role_plan"],
        "decay_features": v33_profile["features"]["branch_fields"]["decay"],
        "decay_head": v33_profile["model"]["heads"]["decay"],
        "global_gain_features": v33_profile["features"]["branch_fields"]["global_gain"],
        "global_gain_head": v33_profile["model"]["heads"]["global_gain"],
        "numeric_profile": v33_profile["numeric_profile"],
        "resources": v33_profile["resources"],
        "training": v33_profile["training"],
    }
    actual = {name: section_sha256(value) for name, value in sorted(sections.items())}
    if actual != profile["preserved_sha256"]:
        raise F0FreezeError("preserved V34 hypothesis section drift")
    return actual


def build_effective_profile(
    profile: dict[str, Any], v33_profile: dict[str, Any]
) -> dict[str, Any]:
    effective = copy.deepcopy(v33_profile)
    corpus_overlay = profile["corpus_overlay"]
    effective["corpus"]["contacts"] = copy.deepcopy(corpus_overlay["contacts"])
    effective["corpus"]["geometry_multiplier_cells"] = copy.deepcopy(
        corpus_overlay["geometry_multiplier_cells"]
    )
    effective["corpus"]["materials"] = copy.deepcopy(corpus_overlay["materials"])
    effective["oracle"] = copy.deepcopy(corpus_overlay["oracle"])
    effective["method_overlay"] = copy.deepcopy(profile["method_overlay"])
    effective["profile_id"] = PROFILE_ID
    effective["schema"] = PROFILE_SCHEMA
    return effective


def contact_pairs(corpus: dict[str, Any]) -> dict[str, tuple[tuple[str, str], ...]]:
    return {
        role: tuple(tuple(pair) for pair in pairs)
        for role, pairs in corpus["contacts"].items()
    }


def all_parent_contacts(parent: dict[str, Any]) -> set[tuple[str, str]]:
    corpus = parent.get("corpus")
    if corpus is not None:
        contacts = corpus["contacts"]
    else:
        contacts = parent["corpus_overlay"]["contacts"]
    groups = contacts.values() if isinstance(contacts, dict) else [contacts]
    return {tuple(pair) for pairs in groups for pair in pairs}


def parent_cells(parent: dict[str, Any]) -> set[tuple[str, ...]]:
    corpus = parent.get("corpus")
    cells = (
        corpus["geometry_multiplier_cells"]
        if corpus is not None
        else parent["corpus_overlay"]["geometry_multiplier_cells"]
    )
    return {tuple(cell) for cell in cells}


def parent_materials(parent: dict[str, Any]) -> set[bytes]:
    corpus = parent.get("corpus")
    materials = (
        corpus["materials"]
        if corpus is not None
        else parent["corpus_overlay"]["materials"]
    )
    return {canonical_json(material) for material in materials}


def parent_oracle(parent: dict[str, Any]) -> dict[str, Any]:
    if "oracle" in parent:
        return parent["oracle"]
    return parent["corpus_overlay"]["oracle"]


def enumerate_role_cases(
    corpus: dict[str, Any], role: str
) -> tuple[set[tuple[int, int, int, tuple[str, str]]], dict[str, dict[str, int]]]:
    family_count = len(corpus["families"])
    material_count = len(corpus["materials"])
    contacts = contact_pairs(corpus)
    cases: set[tuple[int, int, int, tuple[str, str]]] = set()
    strata: dict[str, dict[str, int]] = {}
    for entry in corpus["role_plan"][role]:
        stratum_cases = {
            (family, material, cell, contact)
            for family in range(family_count)
            for material in range(material_count)
            for cell in entry["geometry_cells"]
            for contact in contacts[entry["contact_set"]]
        }
        if entry["stratum"] in strata or cases & stratum_cases:
            raise F0FreezeError(f"role stratum overlap: {role}/{entry['stratum']}")
        cases |= stratum_cases
        strata[entry["stratum"]] = {
            "cases": len(stratum_cases),
            "modal_rows": len(stratum_cases) * corpus["mode_count"],
            "role_local_geometry_groups": (
                family_count * material_count * len(set(entry["geometry_cells"]))
            ),
        }
    return cases, strata


def validate_freshness_counts_and_witness_plan(
    profile: dict[str, Any],
    effective: dict[str, Any],
    parents: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    corpus = effective["corpus"]
    contacts = contact_pairs(corpus)
    all_contacts = {pair for pairs in contacts.values() for pair in pairs}
    if sum(map(len, contacts.values())) != len(all_contacts):
        raise F0FreezeError("V35 contact sets overlap")
    current_cells = {tuple(cell) for cell in corpus["geometry_multiplier_cells"]}
    current_materials = {canonical_json(row) for row in corpus["materials"]}
    for parent_name, parent in sorted(parents.items()):
        if all_contacts & all_parent_contacts(parent):
            raise F0FreezeError(f"{parent_name} contact reused")
        if current_cells & parent_cells(parent):
            raise F0FreezeError(f"{parent_name} geometry cell reused")
        if current_materials & parent_materials(parent):
            raise F0FreezeError(f"{parent_name} material reused")

    expected_contact_counts = {"development": 6, "method_holdout": 6, "train": 12}
    if {
        name: len(pairs) for name, pairs in contacts.items()
    } != expected_contact_counts:
        raise F0FreezeError("contact-set count drift")
    nodal_contacts = {
        name: [pair for pair in pairs if pair[0] == "0"]
        for name, pairs in contacts.items()
    }
    if any(len(pairs) != 1 for pairs in nodal_contacts.values()):
        raise F0FreezeError("each contact set requires exactly one declared node")

    expected_strata = {
        "development": {"contact-only", "geometry-only", "joint"},
        "method_holdout": {"contact-only", "geometry-only", "joint"},
        "train": {"train"},
    }
    role_cases: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    summaries: dict[str, Any] = {}
    for role in ("train", "development", "method_holdout"):
        cases, strata = enumerate_role_cases(corpus, role)
        if set(strata) != expected_strata[role]:
            raise F0FreezeError(f"stratum closure mismatch: {role}")
        expected = corpus["counts"][role]
        geometry_cells = {
            cell
            for entry in corpus["role_plan"][role]
            for cell in entry["geometry_cells"]
        }
        groups = (
            len(corpus["families"]) * len(corpus["materials"]) * len(geometry_cells)
        )
        if (
            len(cases) != expected["cases"]
            or len(cases) * corpus["mode_count"] != expected["modal_rows"]
            or groups != expected["role_local_geometry_groups"]
        ):
            raise F0FreezeError(f"role count algebra mismatch: {role}")
        if role != "train" and strata != expected["strata"]:
            raise F0FreezeError(f"stratum count algebra mismatch: {role}")
        role_cases[role] = cases
        summaries[role] = {
            "cases": len(cases),
            "modal_rows": len(cases) * corpus["mode_count"],
        }
    for left, right in (
        ("train", "development"),
        ("train", "method_holdout"),
        ("development", "method_holdout"),
    ):
        if role_cases[left] & role_cases[right]:
            raise F0FreezeError(f"case identity crosses roles: {left}/{right}")

    witness = profile["witness_contract"]
    required_forbidden = {
        "development_metrics",
        "model_owner",
        "optimizer",
        "oracle_targets",
        "prior_generation_values",
        "protected_signal",
        "real_signal",
    }
    if (
        witness["target_rows_allowed"] != 0
        or witness["evaluation_roles"] != ["development", "method_holdout"]
        or witness["strata"] != ["contact-only", "geometry-only", "joint"]
        or witness["exact_nodal_min_modal_rows_per_stratum"] != 180
        or set(witness["forbidden_dependencies"]) != required_forbidden
        or witness["minimum_both_expert_reachable_rows_per_stratum"] < 1
        or witness["minimum_finite_local_support_rows_per_stratum"] < 1
    ):
        raise F0FreezeError("witness contract drift")
    predicted_nodal_rows: dict[str, dict[str, int]] = {}
    for role in witness["evaluation_roles"]:
        predicted_nodal_rows[role] = {}
        for entry in corpus["role_plan"][role]:
            predicted = (
                len(corpus["families"])
                * len(corpus["materials"])
                * len(entry["geometry_cells"])
                * len(nodal_contacts[entry["contact_set"]])
                * corpus["mode_count"]
            )
            if predicted < witness["exact_nodal_min_modal_rows_per_stratum"]:
                raise F0FreezeError(f"nodal witness algebra shortfall: {role}")
            predicted_nodal_rows[role][entry["stratum"]] = predicted
    return {
        "contact_count_by_set": expected_contact_counts,
        "declared_nodal_contact_by_set": nodal_contacts,
        "predicted_nodal_rows": predicted_nodal_rows,
        "role_identity_prefix": profile["corpus_overlay"]["role_identity_prefix"],
        "roles": summaries,
        "total_cases": sum(len(cases) for cases in role_cases.values()),
        "total_modal_rows": sum(len(cases) for cases in role_cases.values())
        * corpus["mode_count"],
    }


def median_positive_nearest_distance(points: list[tuple[float, ...]]) -> float:
    if len(set(points)) != len(points) or len(points) < 2:
        raise F0FreezeError("support points must be unique and nontrivial")
    nearest: list[float] = []
    for index, point in enumerate(points):
        distances = [
            math.sqrt(
                math.fsum((left - right) ** 2 for left, right in zip(point, other))
            )
            for other_index, other in enumerate(points)
            if other_index != index
        ]
        positive = [value for value in distances if math.isfinite(value) and value > 0]
        if not positive:
            raise F0FreezeError("support bandwidth has no positive distance")
        nearest.append(min(positive))
    value = statistics.median(nearest)
    if not math.isfinite(value) or value <= 0:
        raise F0FreezeError("support bandwidth is not finite-positive")
    return value


def validate_method_freeze(
    profile: dict[str, Any], effective: dict[str, Any], v33: dict[str, Any]
) -> dict[str, Any]:
    method = profile["method_overlay"]
    features = method["features"]
    parent_contact = v33["features"]["branch_fields"]["contact"]
    geometry_fields = [
        "geometry.log_multiplier_0",
        "geometry.log_multiplier_1",
        "geometry.log_multiplier_2",
    ]
    if (
        features["contact_input_count"] != 35
        or features["contact_fields"] != parent_contact + geometry_fields
        or features["geometry_append_order"] != geometry_fields
    ):
        raise F0FreezeError("geometry-conditioned contact feature drift")
    model = method["candidate"]["model"]
    heads = model["heads"]
    if (
        model["parameter_count"] != 1859
        or model["parameter_count_max"] != 2000
        or heads["contact"]["parameter_count"] != 865
        or heads["contact"]["input_count"] != 35
        or heads["decay"] != v33["model"]["heads"]["decay"]
        or heads["global_gain"] != v33["model"]["heads"]["global_gain"]
        or model["initializer"] != v33["model"]["initializer"]
    ):
        raise F0FreezeError("candidate topology or preserved-head drift")

    required_controls = {
        "continuous_local",
        "geometry_neural_only",
        "geometry_spectral_ridge",
        "identity",
        "nearest",
        "raw_mlp",
        "raw_ridge",
        "spectral_ridge",
        "v34_shaped_spectral_mlp",
    }
    if set(method["controls"]) != required_controls:
        raise F0FreezeError("binding control set drift")
    if set(method["ablations"]) != {
        "without_explicit_geometry",
        "without_local_expert",
        "without_neural_expert",
    }:
        raise F0FreezeError("ablation set drift")

    local = method["local_expert"]
    gate = method["coverage_gate"]
    if (
        local["neighbor_count"] != "all-compatible-train-rows"
        or local["maximum_compatible_rows"] != 216
        or local["zero_distance_behavior"] != "same-kernel-equation-no-shortcut"
        or "case-group" not in local["train_cross_fit_exclusion"]
        or gate["exact_match_shortcut"] is not False
        or gate["local_weight_cap"] != "0.80"
        or gate["zero_distance_behavior"]
        != "same-blend-equation-local-weight-exactly-0.80"
        or gate["ood_normalized_squared_distance_strict_max"] != "16"
    ):
        raise F0FreezeError("local expert or coverage gate drift")
    forbidden = set(features["forbidden_fields"])
    allowed_inputs = set(
        local["distance_key"] + gate["contact_key"] + gate["geometry_key"]
    )
    if forbidden & allowed_inputs:
        raise F0FreezeError("forbidden identifier entered local/gate input")
    if set(gate["contact_key"]) != {"contact.u_signed", "contact.v_signed"}:
        raise F0FreezeError("coverage contact key drift")
    if gate["geometry_key"] != geometry_fields:
        raise F0FreezeError("coverage geometry key drift")

    corpus = effective["corpus"]
    normalization = v33["features"]["normalization"]
    if normalization["contact_signed"] != "2*x-1":
        raise F0FreezeError("contact normalizer drift")
    train_contacts = [
        tuple(2.0 * float(value) - 1.0 for value in pair)
        for pair in corpus["contacts"]["train"]
    ]
    geometry_denominator = math.log(1.25)
    train_cells = [
        tuple(math.log(float(value)) / geometry_denominator for value in cell)
        for cell in corpus["geometry_multiplier_cells"][:6]
    ]
    contact_bandwidth = median_positive_nearest_distance(train_contacts)
    geometry_bandwidth = median_positive_nearest_distance(train_cells)
    if 3 * 6 * 12 != local["maximum_compatible_rows"]:
        raise F0FreezeError("local compatible-row algebra drift")
    return {
        "candidate_parameter_count": model["parameter_count"],
        "contact_bandwidth_hex": contact_bandwidth.hex(),
        "contact_input_count": features["contact_input_count"],
        "control_count": len(required_controls),
        "geometry_bandwidth_hex": geometry_bandwidth.hex(),
        "local_maximum_compatible_rows": local["maximum_compatible_rows"],
        "local_weight_cap": gate["local_weight_cap"],
        "ood_normalized_squared_distance_strict_max": gate[
            "ood_normalized_squared_distance_strict_max"
        ],
    }


def validate_oracle_and_access_order(
    profile: dict[str, Any],
    effective: dict[str, Any],
    parents: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    for branch in ("contact", "decay", "global_gain"):
        expression = effective["oracle"][branch]["expression"]
        if any(
            expression == parent_oracle(parent)[branch]["expression"]
            for parent in parents.values()
        ):
            raise F0FreezeError(f"parent oracle expression reused: {branch}")
    if "geometry0" not in effective["oracle"]["contact"]["expression"]:
        raise F0FreezeError("fresh contact truth lacks explicit geometry")
    expected = [
        "f0-identity-freshness-and-static-freeze",
        "c0-signal-blind-witness-and-coverage-census",
        "b0-discarded-local-and-gate-conformance",
        "i0-discarded-whole-owner-terminal-path-proof",
        "d0-train-materialize-and-fit",
        "d0-development-materialize-and-gate",
        "candidate-freeze-if-development-pass",
        "h0-method-holdout-materialize-once-if-frozen",
        "terminal-report",
    ]
    if profile["access_order"] != expected:
        raise F0FreezeError("access order drift")
    terminal = profile["terminal_publication"]
    if (
        terminal["post_access_raise_allowed"] is not False
        or terminal["atomic_staging_required"] is not True
        or terminal["repeat_exact_required"] is not True
        or terminal["pre_access_contract_reject"]
        != {"partial_output_allowed": False, "target_rows": 0}
        or terminal["post_access_outcomes"]
        != ["Pass", "MetricReject", "HardGateReject", "ResourceReject"]
    ):
        raise F0FreezeError("terminal publication contract drift")
    return {
        "fresh_truth_expressions": 3,
        "first_target_stage": "d0-train-materialize-and-fit",
        "pretraining_stages": expected[:4],
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


def build_conformance(profile: dict[str, Any], profile_data: bytes) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    v32 = load_declared_json(profile, "v32_profile")
    v33 = load_declared_json(profile, "v33_profile")
    v34 = load_declared_json(profile, "v34_profile")
    parents = {"V32": v32, "V33": v33, "V34": v34}
    preserved = validate_preserved_sections(profile, v33)
    effective = build_effective_profile(profile, v33)
    freshness = validate_freshness_counts_and_witness_plan(profile, effective, parents)
    method = validate_method_freeze(profile, effective, v33)
    oracle_access = validate_oracle_and_access_order(profile, effective, parents)
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    return {
        "access": ZERO_ACCESS,
        "claim": CLAIM,
        "dependencies": dependencies,
        "freshness_counts_and_witness_plan": freshness,
        "gates": {
            "authority_narrow": "Pass",
            "canonical_profile": "Pass",
            "fresh_against_v32_v33_v34": "Pass",
            "geometry_hybrid_method_closed": "Pass",
            "model_or_target_values": "0 Exact",
            "preserved_decay_gain_and_training": "Pass",
            "prior_generation_values": "0 Exact",
            "target_safe_access_order": "Pass",
            "witness_plan_nonvacuous": "Pass",
        },
        "method_freeze": method,
        "official_values_opened": False,
        "oracle_and_access_order": oracle_access,
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "preserved_sha256": preserved,
        "profile_identity": {
            "bytes": len(profile_data),
            "path": PROFILE_PATH,
            "sha256": sha256_bytes(profile_data),
        },
        "schema": "nextengine.experimental-physical-sound-v35-f0-conformance.v1",
        "status": "Pass",
    }


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v35-f0-", dir=output.parent))
    try:
        conformance = build_conformance(profile, profile_data)
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        evidence = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "conformance": conformance_ref,
            "owner_identity": conformance["owner_identity"],
            "profile_identity": conformance["profile_identity"],
            "schema": "nextengine.experimental-physical-sound-v35-f0-evidence.v1",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "decision": "F0_TARGET_SAFE_GEOMETRY_HYBRID_PROFILE_FREEZE_PASS",
            "evidence": evidence_ref,
            "next_authorized_stage": "V35-C0-signal-blind-witness-and-coverage-census",
            "official_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v35-f0-report.v1",
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
