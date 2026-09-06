#!/usr/bin/env python3
"""Count V35 P1 witnesses and hybrid coverage without targets or models."""

from __future__ import annotations

import argparse
import ast
import copy
import hashlib
import json
import math
import resource
import shutil
import struct
import sys
import tempfile
import time
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
from typing import Any

import physical_sound_v31_p1_modal_owner_v1 as p1
import physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1 as f0

F0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v35-f0-geometry-hybrid-profile-freeze-result-2026-09-02.md"
)
F0_RESULT_SHA256 = "159b09d8d65e504f0f1e9e0ac59444afb614bccc0074e93af23c4540e17678c3"
F0_OWNER_SHA256 = "5173638f200ded42f50a72258617b69df0225f9a0b8fd3fe092475533e1548b2"
P1_OWNER_SHA256 = "04b44d77ce073d07f30e94c3c361ca4c199cb554ecbe017947aa423842781650"
OWNER_PATH = "lab/scripts/physical_sound_v35_c0_witness_and_coverage_census_v1.py"
CLAIM = (
    "SIGNAL_BLIND_P1_WITNESS_AND_GEOMETRY_HYBRID_COVERAGE_CENSUS_ONLY / "
    "NO_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
ROLES = ("train", "development", "method_holdout")
EVALUATION_ROLES = ("development", "method_holdout")
FORBIDDEN_IMPORT_STEMS = {
    "physical_sound_v33_d0_development_tournament_v1",
    "physical_sound_v33_i0_mode_local_spectral_owner_v1",
    "physical_sound_v34_d0_development_tournament_v1",
    "physical_sound_v34_h0_method_holdout_v1",
    "torch",
}
ZERO_FORBIDDEN_ACCESS = dict(f0.ZERO_ACCESS)


class C0CensusError(RuntimeError):
    """The signal-blind V35 C0 census contract failed."""


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


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise C0CensusError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise C0CensusError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_owner_import_boundary() -> dict[str, Any]:
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    tree = ast.parse(owner_data)
    imports: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            imports.add(node.module.split(".")[0])
    forbidden = sorted(imports & FORBIDDEN_IMPORT_STEMS)
    if forbidden:
        raise C0CensusError(f"forbidden owner import: {','.join(forbidden)}")
    required = {
        "physical_sound_v31_p1_modal_owner_v1",
        "physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1",
    }
    if not required.issubset(imports):
        raise C0CensusError("required P1/F0 owner imports missing")
    return {"forbidden_imports": forbidden, "imports": sorted(imports)}


def load_context(profile_path: Path) -> tuple[dict[str, Any], dict[str, Any], bytes]:
    overlay, profile_data = f0.load_profile(profile_path)
    if sha256_bytes(profile_data) != f0.PROFILE_SHA256:
        raise C0CensusError("F0 profile identity drift")
    f0.validate_dependencies(overlay)
    v33 = f0.load_declared_json(overlay, "v33_profile")
    effective = f0.build_effective_profile(overlay, v33)
    f0.validate_preserved_sections(overlay, v33)
    f0.validate_method_freeze(overlay, effective, v33)
    p0_declaration = v33["parent"]["p0_profile"]
    p0_path = repository_root() / p0_declaration["path"]
    p0_data = p0_path.read_bytes()
    if sha256_bytes(p0_data) != p0_declaration["sha256"]:
        raise C0CensusError("P0 profile dependency drift")
    p1_declaration = v33["parent"]["p1_owner"]
    if p1_declaration["sha256"] != P1_OWNER_SHA256:
        raise C0CensusError("P1 owner declaration drift")
    validate_bound_file(p1_declaration["path"], P1_OWNER_SHA256)
    validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256)
    validate_bound_file(F0_RESULT_PATH, F0_RESULT_SHA256)
    return overlay, effective, p0_data


def decimal_product(value: str, multiplier: str) -> str:
    return format(Decimal(value) * Decimal(multiplier), "f")


def base_fixture_for_family(
    family: dict[str, Any], p0_profile: dict[str, Any]
) -> dict[str, Any]:
    for fixture in p0_profile["fixtures"]:
        if fixture["family"] == family["family"]:
            result = copy.deepcopy(fixture)
            result["formula_id"] = family["formula_id"]
            result["support"] = family["support"]
            return result
    raise C0CensusError(f"missing P0 family fixture: {family['family']}")


def build_fixture(
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
    fixture = base_fixture_for_family(family, p0_profile)
    case_id = (
        f"v35/{role}/{stratum}/f{family_index}/m{material_index}/"
        f"g{geometry_cell_index:02d}/{contact_set}/c{contact_index:02d}"
    )
    fixture["fixture_id"] = case_id
    material = corpus["materials"][material_index]
    fixture["material"] = {
        "density_kg_m3": material["density_kg_m3"],
        "loss_rate_per_second": material["loss_rate_per_second"],
        "material_id": "synthetic-elastic-reference",
        "poisson_ratio": material["poisson_ratio"],
        "youngs_modulus_pa": material["youngs_modulus_pa"],
    }
    multiplier_text = corpus["geometry_multiplier_cells"][geometry_cell_index]
    for field, multiplier in zip(
        family["geometry_component_order"], multiplier_text, strict=True
    ):
        fixture["geometry"][field] = decimal_product(
            fixture["geometry"][field], multiplier
        )
    u, v = corpus["contacts"][contact_set][contact_index]
    fixture["contact"] = {
        "normal_impulse_ns": corpus["impulse_ns"],
        "u": u,
        "v": v,
    }
    fixture["mode_count"] = corpus["mode_count"]
    multipliers = tuple(float(value) for value in multiplier_text)
    return case_id, fixture, multipliers


def id_commitment(values: list[str]) -> dict[str, Any]:
    ordered = sorted(values)
    if len(ordered) != len(set(ordered)):
        raise C0CensusError("witness identity is duplicated")
    return {
        "count": len(ordered),
        "first": ordered[0] if ordered else None,
        "last": ordered[-1] if ordered else None,
        "sha256": sha256_bytes(canonical_json(ordered)),
    }


def binary64_pair(value_a: float, value_b: float) -> bytes:
    return struct.pack("<dd", value_a, value_b)


def binary64(value: float) -> bytes:
    return struct.pack("<d", value)


def stencil_coordinates(u: float, v: float, h: float) -> dict[str, tuple[float, float]]:
    return {
        "u_minus": (max(0.0, u - h), v),
        "u_plus": (min(1.0, u + h), v),
        "v_minus": (u, max(0.0, v - h)),
        "v_plus": (u, min(1.0, v + h)),
    }


def structural_stencil(
    solution: dict[str, Any], p0_profile: dict[str, Any], h: float
) -> dict[str, tuple[float, ...]]:
    validated = solution["validated"]
    coordinates = stencil_coordinates(
        float(validated["contact"]["u"]),
        float(validated["contact"]["v"]),
        h,
    )
    return {
        name: tuple(
            float(value)
            for value in p1.participation(
                validated,
                p0_profile,
                solution["indices"],
                point[0],
                point[1],
            )[0]
        )
        for name, point in coordinates.items()
    }


def local_key(
    solution: dict[str, Any],
    fixture: dict[str, Any],
    multipliers: tuple[float, float, float],
    ordinal: int,
    stencil: dict[str, tuple[float, ...]],
) -> tuple[float, ...]:
    mode = solution["modal_document"]["modes"][ordinal]
    material = fixture["material"]
    material_values = (
        math.log2(float(material["youngs_modulus_pa"]) / 83_000_000_000.0) / 2.0,
        math.log2(float(material["density_kg_m3"]) / 3100.0) / 2.0,
        (float(material["poisson_ratio"]) - 0.275) / 0.05,
        (float(material["loss_rate_per_second"]) - 15.0) / 6.0,
    )
    geometry_values = tuple(math.log(value) / math.log(1.25) for value in multipliers)
    u = float(fixture["contact"]["u"])
    v = float(fixture["contact"]["v"])
    log_frequency = max(
        -1.5,
        min(1.5, math.log2(float(mode["frequency_hz"]) / 1000.0) / 4.0),
    )
    values = (
        *material_values,
        *geometry_values,
        2.0 * u - 1.0,
        2.0 * v - 1.0,
        log_frequency,
        float(mode["contact_participation"]),
        stencil["u_minus"][ordinal],
        stencil["u_plus"][ordinal],
        stencil["v_minus"][ordinal],
        stencil["v_plus"][ordinal],
    )
    if len(values) != 15 or not all(math.isfinite(value) for value in values):
        raise C0CensusError("local structural key is not finite width-15")
    return values


def collect_role(
    role: str,
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
    case_ids: list[str] = []
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
                        peak = float(solution["metrics"]["sample_peak"])
                        energy = float(solution["metrics"]["sample_energy"])
                        if peak > 0.0 and energy > 0.0:
                            non_silent[stratum][family_key].append(case_id)
                        modes = solution["modal_document"]["modes"]
                        if len(modes) != corpus["mode_count"]:
                            raise C0CensusError(f"mode count drift: {case_id}")
                        stencil = structural_stencil(solution, p0_profile, h)
                        contact_key = tuple(
                            2.0 * float(value) - 1.0 for value in contact_text
                        )
                        for ordinal, mode in enumerate(modes):
                            row_id = f"{case_id}/o{ordinal:02d}"
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
                                (row_id, binary64_pair(frequency, decay))
                            )
                            contact_group_key = (
                                family_index,
                                material_index,
                                geometry_cell_index,
                                ordinal,
                            )
                            contact_groups[stratum][contact_group_key].append(
                                (row_id, binary64(contact))
                            )
                            row_local_key = local_key(
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
    if len(case_ids) != expected["cases"] or len(all_rows) != expected["modal_rows"]:
        raise C0CensusError(f"role count closure mismatch: {role}")
    result: dict[str, Any] = {
        "all_case_ids": id_commitment(case_ids),
        "all_modal_row_ids": id_commitment(all_rows),
        "role": role,
    }
    if role == "train":
        return records, result

    witness = overlay["witness_contract"]
    strata: dict[str, Any] = {}
    for stratum in witness["strata"]:
        nodal_commitment = id_commitment(nodal[stratum])
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
        material_commitment = id_commitment(material_sensitive)
        contact_commitment = id_commitment(contact_sensitive)
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
            commitment = id_commitment(non_silent[stratum][family_key])
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

    positive = id_commitment(pickup_positive)
    negative = id_commitment(pickup_negative)
    if (
        positive["count"] < witness["minimum_nonzero_pickup_positive_rows_per_role"]
        or negative["count"] < witness["minimum_nonzero_pickup_negative_rows_per_role"]
    ):
        raise C0CensusError(f"pickup-sign witness shortfall: {role}")
    remesh_result: dict[str, Any] = {}
    for family_index, family in enumerate(corpus["families"]):
        family_key = f"f{family_index}:{family['formula_id']}"
        commitment = id_commitment(remesh[family_key])
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


def squared_distance(left: tuple[float, ...], right: tuple[float, ...]) -> float:
    if len(left) != len(right):
        raise C0CensusError("distance key width mismatch")
    value = math.fsum((a - b) ** 2 for a, b in zip(left, right, strict=True))
    if not math.isfinite(value) or value < 0.0:
        raise C0CensusError("distance is not finite nonnegative")
    return value


def hex_range(values: list[float]) -> dict[str, str]:
    if not values or not all(math.isfinite(value) for value in values):
        raise C0CensusError("finite range is empty or non-finite")
    return {"maximum": max(values).hex(), "minimum": min(values).hex()}


def coverage_gate_values(
    geometry_key: tuple[float, ...],
    contact_key: tuple[float, ...],
    train_geometry: list[tuple[float, ...]],
    train_contacts: list[tuple[float, ...]],
    geometry_bandwidth: float,
    contact_bandwidth: float,
) -> tuple[float, float, float]:
    geometry_distance = math.sqrt(
        min(squared_distance(geometry_key, candidate) for candidate in train_geometry)
    )
    contact_distance = math.sqrt(
        min(squared_distance(contact_key, candidate) for candidate in train_contacts)
    )
    normalized_squared = (geometry_distance / geometry_bandwidth) ** 2 + (
        contact_distance / contact_bandwidth
    ) ** 2
    local_weight = 0.80 * math.exp(-0.5 * normalized_squared)
    neural_weight = 1.0 - local_weight
    if not all(
        math.isfinite(value)
        for value in (normalized_squared, local_weight, neural_weight)
    ):
        raise C0CensusError("coverage gate is non-finite")
    return normalized_squared, local_weight, neural_weight


def analyze_hybrid_coverage(
    overlay: dict[str, Any],
    effective: dict[str, Any],
    train_records: list[dict[str, Any]],
    evaluation_records: list[dict[str, Any]],
) -> dict[str, Any]:
    method = overlay["method_overlay"]
    local_spec = method["local_expert"]
    gate_spec = method["coverage_gate"]
    if (
        local_spec["maximum_compatible_rows"] != 216
        or local_spec["neighbor_count"] != "all-compatible-train-rows"
        or gate_spec["local_weight_cap"] != "0.80"
        or gate_spec["exact_match_shortcut"] is not False
    ):
        raise C0CensusError("hybrid coverage profile drift")

    train_by_partition: dict[tuple[Any, ...], list[dict[str, Any]]] = defaultdict(list)
    for record in train_records:
        train_by_partition[record["partition"]].append(record)
    for rows in train_by_partition.values():
        rows.sort(key=lambda row: (row["partition"], row["local_key"]))
        if len(rows) != local_spec["maximum_compatible_rows"]:
            raise C0CensusError("compatible train partition is not exactly 216 rows")

    cross_fit_rows: list[str] = []
    cross_fit_counts: list[int] = []
    for query in train_records:
        compatible = train_by_partition[query["partition"]]
        remaining = sum(
            row["case_group_key"] != query["case_group_key"] for row in compatible
        )
        if remaining != 215:
            raise C0CensusError("train case-group leave-out support is not 215")
        cross_fit_rows.append(query["row_id"])
        cross_fit_counts.append(remaining)

    corpus = effective["corpus"]
    train_geometry = [
        tuple(math.log(float(value)) / math.log(1.25) for value in cell)
        for cell in corpus["geometry_multiplier_cells"][:6]
    ]
    train_contacts = [
        tuple(2.0 * float(value) - 1.0 for value in pair)
        for pair in corpus["contacts"]["train"]
    ]
    geometry_bandwidth = f0.median_positive_nearest_distance(train_geometry)
    contact_bandwidth = f0.median_positive_nearest_distance(train_contacts)
    strict_ood_max = float(gate_spec["ood_normalized_squared_distance_strict_max"])
    forbidden = set(method["features"]["forbidden_fields"])
    declared_inputs = set(
        local_spec["distance_key"]
        + gate_spec["contact_key"]
        + gate_spec["geometry_key"]
    )
    forbidden_intersection = sorted(forbidden & declared_inputs)
    if forbidden_intersection:
        raise C0CensusError("forbidden identifier entered hybrid input")

    accumulators: dict[tuple[str, str], dict[str, Any]] = defaultdict(
        lambda: {
            "both": [],
            "compatible": [],
            "denominator": [],
            "gate_d2": [],
            "local_distance": [],
            "local_weight": [],
            "neural_weight": [],
        }
    )
    for query in evaluation_records:
        compatible = train_by_partition.get(query["partition"], [])
        if len(compatible) != 216:
            raise C0CensusError("evaluation local support is not exactly 216 rows")
        distances = [
            squared_distance(query["local_key"], row["local_key"]) for row in compatible
        ]
        nearest = min(distances)
        denominator = math.fsum(
            math.exp(-0.5 * (distance - nearest)) for distance in distances
        )
        if not math.isfinite(denominator) or denominator <= 0.0:
            raise C0CensusError("local kernel denominator is not finite-positive")
        gate_d2, local_weight, neural_weight = coverage_gate_values(
            query["geometry_key"],
            query["contact_key"],
            train_geometry,
            train_contacts,
            geometry_bandwidth,
            contact_bandwidth,
        )
        if gate_d2 >= strict_ood_max:
            raise C0CensusError(
                f"official row outside frozen support: {query['row_id']}"
            )
        if not 0.0 < local_weight < 1.0 or not 0.0 < neural_weight < 1.0:
            raise C0CensusError(f"expert unreachable: {query['row_id']}")
        key = (query["role"], query["stratum"])
        target = accumulators[key]
        target["both"].append(query["row_id"])
        target["compatible"].append(len(compatible))
        target["denominator"].append(denominator)
        target["gate_d2"].append(gate_d2)
        target["local_distance"].append(nearest)
        target["local_weight"].append(local_weight)
        target["neural_weight"].append(neural_weight)

    witness = overlay["witness_contract"]
    roles: dict[str, Any] = {}
    for role in EVALUATION_ROLES:
        roles[role] = {}
        for stratum in witness["strata"]:
            values = accumulators[(role, stratum)]
            both = id_commitment(values["both"])
            if (
                both["count"]
                < witness["minimum_both_expert_reachable_rows_per_stratum"]
                or both["count"]
                < witness["minimum_finite_local_support_rows_per_stratum"]
            ):
                raise C0CensusError(
                    f"hybrid coverage witness shortfall: {role}/{stratum}"
                )
            if min(values["compatible"]) != 216 or max(values["compatible"]) != 216:
                raise C0CensusError("local support count drift")
            roles[role][stratum] = {
                "both_experts_reachable_rows": both,
                "compatible_train_rows": {"maximum": 216, "minimum": 216},
                "gate_normalized_squared_distance": hex_range(values["gate_d2"]),
                "kernel_denominator": hex_range(values["denominator"]),
                "local_nearest_squared_distance": hex_range(values["local_distance"]),
                "local_weight": hex_range(values["local_weight"]),
                "neural_weight": hex_range(values["neural_weight"]),
                "ood_rows": 0,
            }
    return {
        "bandwidths": {
            "contact_hex": contact_bandwidth.hex(),
            "geometry_hex": geometry_bandwidth.hex(),
        },
        "evaluation": roles,
        "forbidden_feature_intersection": forbidden_intersection,
        "train_cross_fit": {
            "available_rows": id_commitment(cross_fit_rows),
            "compatible_after_case_group_leave_out": {
                "maximum": max(cross_fit_counts),
                "minimum": min(cross_fit_counts),
            },
        },
    }


def external_output(path: Path) -> Path:
    try:
        return f0.external_output(path)
    except f0.F0FreezeError as error:
        raise C0CensusError(str(error)) from error


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    overlay, effective, p0_data = load_context(profile_path)
    output = external_output(output_path)
    import_boundary = validate_owner_import_boundary()
    p0_profile = json.loads(p0_data)
    records: dict[str, list[dict[str, Any]]] = {}
    role_census: dict[str, Any] = {}
    for role in ROLES:
        records[role], role_census[role] = collect_role(
            role, overlay, effective, p0_profile
        )
    coverage = analyze_hybrid_coverage(
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
        "schema": "nextengine.experimental-physical-sound-v35-c0-census.v1",
        "status": "Pass",
        "structural_access": structural_access,
    }
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    dependencies = {
        "f0_owner": validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256),
        "f0_result": validate_bound_file(F0_RESULT_PATH, F0_RESULT_SHA256),
        "p1_owner": validate_bound_file(
            "lab/scripts/physical_sound_v31_p1_modal_owner_v1.py", P1_OWNER_SHA256
        ),
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
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v35-c0-", dir=output.parent))
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
            "profile_identity": {
                "bytes": len((repository_root() / f0.PROFILE_PATH).read_bytes()),
                "path": f0.PROFILE_PATH,
                "sha256": f0.PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v35-c0-evidence.v1",
            "structural_access": structural_access,
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "census": census_ref,
            "claim": CLAIM,
            "decision": "C0_WITNESS_AND_GEOMETRY_HYBRID_COVERAGE_PASS",
            "evidence": evidence_ref,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "next_authorized_stage": "V35-B0-discarded-local-and-gate-conformance",
            "official_target_or_model_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v35-c0-report.v1",
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
        f0.F0FreezeError,
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
                "official_target_or_model_values_opened": report[
                    "official_target_or_model_values_opened"
                ],
                "profile_sha256": f0.PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
