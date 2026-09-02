#!/usr/bin/env python3
"""Census fresh V36 P1 witnesses without loading truth, targets, or models."""

from __future__ import annotations

import argparse
import ast
import copy
import hashlib
import json
import math
import resource
import shutil
import sys
import tempfile
import time
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
from typing import Any, cast

import physical_sound_v31_p1_modal_owner_v1 as p1
import physical_sound_v35_c0_witness_and_coverage_census_v1 as baseline

PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v36-c0-fresh-structural-census-profile.v1"
)
PROFILE_ID = "physical-sound-v36-c0-fresh-structural-census-v1"
PROFILE_SHA256 = "55d29722836976ad0fce22d362452564a49ffbbc905722f725001ee6845be48b"
BASELINE_COMMIT = "eb4f9cf70ad2626202616494afe58f86160b426e"
CLAIM = (
    "FRESH_SIGNAL_BLIND_P1_WITNESS_AND_HYBRID_SUPPORT_CENSUS_ONLY / "
    "NO_TRUTH_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_"
    "ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
OWNER_PATH = "lab/scripts/physical_sound_v36_c0_fresh_structural_census_v1.py"
PROFILE_PATH = "lab/profiles/physical-sound-v36-c0-fresh-structural-census.v1.json"
MAX_PROFILE_BYTES = 1_048_576
ROLES = ("train", "development", "method_holdout")
EVALUATION_ROLES = ("development", "method_holdout")
EXPECTED_AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "external_research_only": True,
    "model_allowed": False,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "target_allowed": False,
    "truth_allowed": False,
    "validator_release_authority": False,
    "x0_allowed_after_c0": True,
}
ZERO_FORBIDDEN_ACCESS = {
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
    "truth_coefficient_values_read": 0,
}


class C0CensusError(RuntimeError):
    """The signal-blind V36 C0 census contract failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise C0CensusError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise C0CensusError("profile must not be a symlink")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise C0CensusError("profile size outside bound")
    try:
        profile = json.loads(data)
    except json.JSONDecodeError as error:
        raise C0CensusError(f"invalid profile JSON: {error}") from error
    if data != canonical_json(profile):
        raise C0CensusError("profile is not canonical JSON")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise C0CensusError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or profile.get("authority") != EXPECTED_AUTHORITY
    ):
        raise C0CensusError("profile identity or authority mismatch")
    validate_structural_profile_boundary(profile)
    return profile, data


def validate_structural_profile_boundary(profile: dict[str, Any]) -> dict[str, Any]:
    prohibitions = profile["structural_prohibitions"]
    forbidden_sections = set(prohibitions["forbidden_top_level_sections"])
    present = sorted(forbidden_sections & set(profile))
    if present:
        raise C0CensusError(
            f"forbidden structural profile section: {','.join(present)}"
        )
    if profile["witness_contract"]["target_rows_allowed"] != 0:
        raise C0CensusError("structural profile allows target rows")
    if set(profile["identity"]) != {
        "case_enumeration_seed",
        "role_identity_prefix",
        "role_seed_by_role",
        "truth_namespace_seed_by_branch",
    }:
        raise C0CensusError("structural identity namespace drift")
    encoded = canonical_json(profile)
    forbidden_literals = (
        b'"oracle":',
        b'"targets":',
        b'"training":',
        b'"weights":',
    )
    if any(literal in encoded for literal in forbidden_literals):
        raise C0CensusError("value-bearing literal entered structural profile")
    return {
        "forbidden_top_level_intersection": present,
        "profile_contains_oracle_section": False,
        "profile_contains_target_section": False,
        "profile_contains_training_section": False,
        "profile_contains_weight_section": False,
    }


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise C0CensusError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise C0CensusError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    declarations = {**profile["parent"], "protocol": profile["protocol"]}
    return {
        dependency_id: validate_bound_file(declared["path"], declared["sha256"])
        for dependency_id, declared in sorted(declarations.items())
    }


def validate_owner_import_boundary(profile: dict[str, Any]) -> dict[str, Any]:
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    tree = ast.parse(owner_data)
    imports: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            imports.add(node.module.split(".")[0])
    forbidden_stems = set(profile["structural_prohibitions"]["forbidden_import_stems"])
    forbidden = sorted(imports & forbidden_stems)
    if forbidden:
        raise C0CensusError(f"forbidden owner import: {','.join(forbidden)}")
    required = {
        "physical_sound_v31_p1_modal_owner_v1",
        "physical_sound_v35_c0_witness_and_coverage_census_v1",
    }
    if not required.issubset(imports):
        raise C0CensusError("required P1/baseline imports missing")
    parent_boundary = baseline.validate_owner_import_boundary()
    if parent_boundary["forbidden_imports"]:
        raise C0CensusError("frozen target-free baseline import boundary drift")
    return {
        "forbidden_imports": forbidden,
        "imports": sorted(imports),
        "reused_baseline_forbidden_imports": parent_boundary["forbidden_imports"],
    }


def load_context(
    profile_path: Path,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any], bytes]:
    profile, profile_data = load_profile(profile_path)
    validate_dependencies(profile)
    p0_declaration = profile["parent"]["p0_profile"]
    p0_data = (repository_root() / p0_declaration["path"]).read_bytes()
    if sha256_bytes(p0_data) != p0_declaration["sha256"]:
        raise C0CensusError("P0 profile dependency drift")
    try:
        p0_value = json.loads(p0_data)
    except json.JSONDecodeError as error:
        raise C0CensusError("P0 profile is not valid JSON") from error
    if not isinstance(p0_value, dict):
        raise C0CensusError("P0 profile must be an object")
    p0_profile = cast(dict[str, Any], p0_value)
    effective = {
        "corpus": copy.deepcopy(profile["corpus"]),
        "features": copy.deepcopy(profile["features"]),
        "resources": copy.deepcopy(profile["resources"]),
    }
    overlay = {
        "method_overlay": copy.deepcopy(profile["hybrid_support"]),
        "witness_contract": copy.deepcopy(profile["witness_contract"]),
    }
    return profile, overlay, effective, p0_profile, profile_data


def case_identity(
    profile: dict[str, Any],
    role: str,
    stratum: str,
    family: dict[str, Any],
    material: dict[str, Any],
    geometry: list[str],
    contact: tuple[str, str],
) -> str:
    identity = profile["identity"]
    payload = {
        "contact": contact,
        "enumeration_seed": identity["case_enumeration_seed"],
        "family": family,
        "geometry": geometry,
        "material": material,
        "prefix": identity["role_identity_prefix"],
        "role": role,
        "role_seed": identity["role_seed_by_role"][role],
        "stratum": stratum,
        "truth_seed_by_branch": identity["truth_namespace_seed_by_branch"],
    }
    return "case-" + sha256_bytes(canonical_json(payload))


def build_fixture(
    profile: dict[str, Any],
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
    role: str,
    stratum: str,
    family_index: int,
    material_index: int,
    geometry_cell_index: int,
    contact_set: str,
    contact_index: int,
) -> tuple[str, dict[str, Any], tuple[float, float, float]]:
    if role not in ROLES:
        raise C0CensusError(f"C0 role access forbidden: {role}")
    corpus = effective["corpus"]
    family = corpus["families"][family_index]
    fixture = baseline.base_fixture_for_family(family, p0_profile)
    material = corpus["materials"][material_index]
    geometry_text = corpus["geometry_multiplier_cells"][geometry_cell_index]
    contact_text = tuple(corpus["contacts"][contact_set][contact_index])
    case_id = case_identity(
        profile,
        role,
        stratum,
        family,
        material,
        geometry_text,
        contact_text,
    )
    fixture["fixture_id"] = case_id
    fixture["material"] = {
        "density_kg_m3": material["density_kg_m3"],
        "loss_rate_per_second": material["loss_rate_per_second"],
        "material_id": "synthetic-elastic-reference",
        "poisson_ratio": material["poisson_ratio"],
        "youngs_modulus_pa": material["youngs_modulus_pa"],
    }
    for field, multiplier in zip(
        family["geometry_component_order"], geometry_text, strict=True
    ):
        fixture["geometry"][field] = format(
            Decimal(fixture["geometry"][field]) * Decimal(multiplier), "f"
        )
    fixture["contact"] = {
        "normal_impulse_ns": corpus["impulse_ns"],
        "u": contact_text[0],
        "v": contact_text[1],
    }
    fixture["mode_count"] = corpus["mode_count"]
    multipliers = tuple(float(value) for value in geometry_text)
    if len(multipliers) != 3:
        raise C0CensusError("geometry multiplier width must be three")
    return case_id, fixture, (multipliers[0], multipliers[1], multipliers[2])


def line_root(identities: list[str]) -> str:
    if not identities or len(identities) != len(set(identities)):
        raise C0CensusError("identity root requires unique nonempty values")
    return sha256_bytes(("\n".join(sorted(identities)) + "\n").encode())


def collect_role(
    role: str,
    profile: dict[str, Any],
    overlay: dict[str, Any],
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    if role not in ROLES:
        raise C0CensusError(f"C0 role access forbidden: {role}")
    corpus = effective["corpus"]
    h = float(effective["features"]["lift"]["local_stencil_offset"])
    records: list[dict[str, Any]] = []
    all_rows: list[str] = []
    case_ids: list[str] = []
    nodal: dict[str, list[str]] = defaultdict(list)
    pickup_positive: list[str] = []
    pickup_negative: list[str] = []
    remesh: dict[str, list[str]] = defaultdict(list)
    non_silent: dict[str, dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))
    material_groups: dict[str, dict[tuple[Any, ...], list[tuple[str, bytes]]]] = (
        defaultdict(lambda: defaultdict(list))
    )
    contact_groups: dict[str, dict[tuple[Any, ...], list[tuple[str, bytes]]]] = (
        defaultdict(lambda: defaultdict(list))
    )
    for entry in corpus["role_plan"][role]:
        stratum = entry["stratum"]
        contact_set = entry["contact_set"]
        for family_index, family in enumerate(corpus["families"]):
            family_key = f"f{family_index}:{family['formula_id']}"
            partition_base = (family["formula_id"], family["support"])
            for material_index in range(len(corpus["materials"])):
                for geometry_cell_index in entry["geometry_cells"]:
                    multipliers = tuple(
                        float(value)
                        for value in corpus["geometry_multiplier_cells"][
                            geometry_cell_index
                        ]
                    )
                    geometry_key = tuple(
                        math.log(value) / math.log(1.25) for value in multipliers
                    )
                    for contact_index, contact_text in enumerate(
                        corpus["contacts"][contact_set]
                    ):
                        case_id, fixture, fixture_multipliers = build_fixture(
                            profile,
                            effective,
                            p0_profile,
                            role,
                            stratum,
                            family_index,
                            material_index,
                            geometry_cell_index,
                            contact_set,
                            contact_index,
                        )
                        solution = p1.solve_case(case_id, fixture, p0_profile)
                        case_ids.append(case_id)
                        if (
                            solution["metrics"]["remesh_common_vertices_exact"]
                            is not True
                        ):
                            raise C0CensusError(f"remesh witness failed: {case_id}")
                        remesh[family_key].append(case_id)
                        if (
                            float(solution["metrics"]["sample_peak"]) > 0.0
                            and float(solution["metrics"]["sample_energy"]) > 0.0
                        ):
                            non_silent[stratum][family_key].append(case_id)
                        modes = solution["modal_document"]["modes"]
                        if len(modes) != corpus["mode_count"]:
                            raise C0CensusError(f"mode count drift: {case_id}")
                        stencil = baseline.structural_stencil(solution, p0_profile, h)
                        contact_key = tuple(
                            2.0 * float(value) - 1.0 for value in contact_text
                        )
                        for ordinal, mode in enumerate(modes):
                            row_id = f"{case_id}:mode-{ordinal:02d}"
                            all_rows.append(row_id)
                            contact = float(mode["contact_participation"])
                            pickup = float(mode["pickup_participation"])
                            signed_gain = float(mode["signed_gain"])
                            frequency = float(mode["frequency_hz"])
                            decay = float(mode["decay_per_second"])
                            if (
                                int(mode["ordinal"]) != ordinal
                                or not all(
                                    math.isfinite(value)
                                    for value in (
                                        contact,
                                        pickup,
                                        signed_gain,
                                        frequency,
                                        decay,
                                    )
                                )
                                or frequency <= 0.0
                                or decay <= 0.0
                                or signed_gain != contact * pickup
                            ):
                                raise C0CensusError(f"invalid structural row: {row_id}")
                            if contact == 0.0:
                                if signed_gain != 0.0:
                                    raise C0CensusError(f"nodal gain drift: {row_id}")
                                nodal[stratum].append(row_id)
                            if pickup > 0.0:
                                pickup_positive.append(row_id)
                            elif pickup < 0.0:
                                pickup_negative.append(row_id)
                            material_group_key = (
                                family_index,
                                geometry_cell_index,
                                contact_set,
                                contact_index,
                                ordinal,
                            )
                            material_groups[stratum][material_group_key].append(
                                (row_id, baseline.binary64_pair(frequency, decay))
                            )
                            contact_group_key = (
                                family_index,
                                material_index,
                                geometry_cell_index,
                                ordinal,
                            )
                            contact_groups[stratum][contact_group_key].append(
                                (row_id, baseline.binary64(contact))
                            )
                            row_local_key = baseline.local_key(
                                solution,
                                fixture,
                                fixture_multipliers,
                                ordinal,
                                stencil,
                            )
                            records.append(
                                {
                                    "case_group_key": (
                                        *partition_base,
                                        row_local_key[:4],
                                        geometry_key,
                                        contact_key,
                                    ),
                                    "contact_key": contact_key,
                                    "geometry_key": geometry_key,
                                    "local_key": row_local_key,
                                    "partition": (*partition_base, ordinal),
                                    "role": role,
                                    "row_id": row_id,
                                    "stratum": stratum,
                                }
                            )

    expected = corpus["counts"][role]
    f0_expected = profile["f0_commitments"][role]
    case_root = line_root(case_ids)
    row_root = line_root(all_rows)
    if (
        len(case_ids) != expected["cases"]
        or len(all_rows) != expected["modal_rows"]
        or len(case_ids) != f0_expected["case_count"]
        or len(all_rows) != f0_expected["modal_row_count"]
        or case_root != f0_expected["case_root_sha256"]
        or row_root != f0_expected["modal_row_root_sha256"]
    ):
        raise C0CensusError(f"F0 role commitment mismatch: {role}")
    result: dict[str, Any] = {
        "all_case_ids": baseline.id_commitment(case_ids),
        "all_modal_row_ids": baseline.id_commitment(all_rows),
        "f0_case_root_sha256": case_root,
        "f0_modal_row_root_sha256": row_root,
        "role": role,
    }
    if role == "train":
        return records, result

    witness = overlay["witness_contract"]
    strata: dict[str, Any] = {}
    for stratum in witness["strata"]:
        nodal_commitment = baseline.id_commitment(nodal[stratum])
        if (
            nodal_commitment["count"]
            < witness["exact_nodal_min_modal_rows_per_stratum"]
        ):
            raise C0CensusError(f"nodal witness shortfall: {role}/{stratum}")
        material_sensitive = [
            row_id
            for rows in material_groups[stratum].values()
            if len({value for _, value in rows}) > 1
            for row_id, _ in rows
        ]
        contact_sensitive = [
            row_id
            for rows in contact_groups[stratum].values()
            if len({value for _, value in rows}) > 1
            for row_id, _ in rows
        ]
        material_commitment = baseline.id_commitment(material_sensitive)
        contact_commitment = baseline.id_commitment(contact_sensitive)
        if (
            material_commitment["count"]
            < witness["minimum_material_sensitive_rows_per_stratum"]
            or contact_commitment["count"]
            < witness["minimum_contact_sensitive_rows_per_stratum"]
        ):
            raise C0CensusError(f"sensitivity witness shortfall: {role}/{stratum}")
        family_non_silent: dict[str, Any] = {}
        for family_index, family in enumerate(corpus["families"]):
            family_key = f"f{family_index}:{family['formula_id']}"
            commitment = baseline.id_commitment(non_silent[stratum][family_key])
            if (
                commitment["count"]
                < witness["minimum_non_silent_cases_per_family_and_stratum"]
            ):
                raise C0CensusError(
                    f"non-silent witness shortfall: {role}/{stratum}/{family_key}"
                )
            family_non_silent[family_key] = commitment
        strata[stratum] = {
            "contact_sensitive_rows": contact_commitment,
            "material_sensitive_rows": material_commitment,
            "nodal_zero_rows": nodal_commitment,
            "non_silent_cases_by_family": family_non_silent,
        }

    positive = baseline.id_commitment(pickup_positive)
    negative = baseline.id_commitment(pickup_negative)
    if (
        positive["count"] < witness["minimum_nonzero_pickup_positive_rows_per_role"]
        or negative["count"] < witness["minimum_nonzero_pickup_negative_rows_per_role"]
    ):
        raise C0CensusError(f"pickup-sign witness shortfall: {role}")
    remesh_result: dict[str, Any] = {}
    for family_index, family in enumerate(corpus["families"]):
        family_key = f"f{family_index}:{family['formula_id']}"
        commitment = baseline.id_commitment(remesh[family_key])
        if commitment["count"] < witness["minimum_remesh_pairs_per_family_and_role"]:
            raise C0CensusError(f"remesh witness shortfall: {role}/{family_key}")
        remesh_result[family_key] = commitment
    result.update(
        {
            "pickup_negative_rows": negative,
            "pickup_positive_rows": positive,
            "remesh_cases_by_family": remesh_result,
            "strata": strata,
        }
    )
    return records, result


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise C0CensusError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise C0CensusError("output must be a fresh external path")
    return output


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    profile, overlay, effective, p0_profile, profile_data = load_context(profile_path)
    output = external_output(output_path)
    import_boundary = validate_owner_import_boundary(profile)
    profile_boundary = validate_structural_profile_boundary(profile)
    records: dict[str, list[dict[str, Any]]] = {}
    role_census: dict[str, Any] = {}
    for role in ROLES:
        records[role], role_census[role] = collect_role(
            role, profile, overlay, effective, p0_profile
        )
    coverage = baseline.analyze_hybrid_coverage(
        overlay,
        effective,
        records["train"],
        records["development"] + records["method_holdout"],
    )
    structural_access = {
        "p1_cases_solved": sum(
            role["all_case_ids"]["count"] for role in role_census.values()
        ),
        "p1_modal_rows_observed": sum(
            role["all_modal_row_ids"]["count"] for role in role_census.values()
        ),
        "roles_observed": list(ROLES),
    }
    if structural_access != {
        "p1_cases_solved": 1512,
        "p1_modal_rows_observed": 15120,
        "roles_observed": ["train", "development", "method_holdout"],
    }:
        raise C0CensusError("structural access closure mismatch")
    census = {
        "claim": CLAIM,
        "forbidden_access": ZERO_FORBIDDEN_ACCESS,
        "hybrid_coverage": coverage,
        "roles": role_census,
        "schema": "nextengine.experimental-physical-sound-v36-c0-census.v1",
        "status": "Pass",
        "structural_access": structural_access,
    }
    elapsed = time.monotonic() - started
    rss = peak_rss_bytes()
    resources = effective["resources"]
    resource_gates = {
        "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
        "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
    }
    if not all(resource_gates.values()):
        raise C0CensusError("C0 resource envelope exceeded")
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    dependencies = validate_dependencies(profile)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v36-c0-", dir=output.parent))
    try:
        census_ref = write_bytes(staging, "census.json", canonical_json(census))
        evidence = {
            "claim": CLAIM,
            "dependencies": dependencies,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "import_boundary": import_boundary,
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "profile_boundary": profile_boundary,
            "profile_identity": {
                "bytes": len(profile_data),
                "path": PROFILE_PATH,
                "sha256": sha256_bytes(profile_data),
            },
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v36-c0-evidence.v1",
            "structural_access": structural_access,
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "census": census_ref,
            "claim": CLAIM,
            "decision": "C0_FRESH_P1_WITNESS_AND_HYBRID_SUPPORT_PASS",
            "evidence": evidence_ref,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "next_authorized_stage": "V36-X0-mutation-and-lifecycle-conformance",
            "official_target_truth_or_model_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v36-c0-report.v1",
            "status": "Pass",
            "structural_access": structural_access,
        }
        write_bytes(staging, "report.json", canonical_json(report))
        total_bytes = sum(path.stat().st_size for path in staging.iterdir())
        if total_bytes > resources["max_output_bytes"]:
            raise C0CensusError("C0 output resource envelope exceeded")
        staging.replace(output)
        print(
            f"c0-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
            f"output_bytes={total_bytes}",
            file=sys.stderr,
        )
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
    except (
        C0CensusError,
        baseline.C0CensusError,
        p1.ModalOwnerError,
        p1.OutOfDomain,
        OSError,
        KeyError,
        TypeError,
        ValueError,
    ) as error:
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
                "official_target_truth_or_model_values_opened": report[
                    "official_target_truth_or_model_values_opened"
                ],
                "profile_sha256": PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
