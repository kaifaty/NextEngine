#!/usr/bin/env python3
"""Prove V37 full-shape QSO structure and cost without scientific targets."""

from __future__ import annotations

import argparse
import ast
import copy
import gc
import hashlib
import json
import math
import resource
import shutil
import tempfile
import time
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path
from typing import Any, cast

import numpy as np
import physical_sound_v31_p1_modal_owner_v1 as p1
import physical_sound_v37_f0_fresh_operator_freeze_v1 as f0
import physical_sound_v37_query_surface_contract_v1 as contract
import torch
from numpy.typing import NDArray
from torch import nn

PROFILE_PATH = (
    "lab/profiles/physical-sound-v37-c0-query-surface-structural-cost.v1.json"
)
OWNER_PATH = "lab/scripts/physical_sound_v37_c0_query_surface_structural_cost_v1.py"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-c0-query-surface-structural-"
    "cost-profile.v1"
)
PROFILE_ID = "physical-sound-v37-c0-query-surface-structural-cost-v1"
PROFILE_SHA256 = "acf424694564f09cfab0aac36c47c00c9b72cb1d98711f7f57216d4012934def"
BASELINE_COMMIT = "b06524e543b5e25d0052796ccdcfa64770f3c79d"
CLAIM = (
    "TARGET_FREE_FULL_SHAPE_QUERY_SURFACE_STRUCTURE_REACHABILITY_AND_COST_ONLY / "
    "NO_TRUTH_EVALUATION_SCIENTIFIC_TARGET_QUALITY_REAL_MATERIAL_VALIDATOR_"
    "RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
ROLE_NAMES = ("train", "development", "method_holdout")
F0_PROFILE_PATH = (
    "lab/profiles/physical-sound-v37-f0-fresh-query-surface-operator.v1.json"
)
P0_PROFILE_PATH = "lab/profiles/physical-sound-v31-p0-causal-baseline.v1.json"
EXPECTED_AUTHORITY = {
    "admission_authority": False,
    "artificial_zero_cost_workload_allowed": True,
    "authored_fallback_required": True,
    "external_research_only": True,
    "official_capability_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "runtime_authority": False,
    "scientific_model_parameter_authority": False,
    "scientific_target_allowed": False,
    "signal_decode_allowed": False,
    "synthetic_only": True,
    "truth_evaluation_allowed": False,
    "validator_release_authority": False,
    "x0_allowed_after_c0": True,
}
ZERO_FORBIDDEN_ACCESS = {
    "development_scientific_rows": 0,
    "fresh_v37_target_rows": 0,
    "method_holdout_scientific_rows": 0,
    "network_requests": 0,
    "official_capabilities_constructed": 0,
    "official_d0_rows": 0,
    "official_h0_rows": 0,
    "prior_generation_metric_values_read": 0,
    "prior_generation_prediction_values_read": 0,
    "prior_generation_target_values_read": 0,
    "prior_generation_weight_values_read": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "scientific_model_parameters_initialized": 0,
    "target_arrays_constructed": 0,
    "train_scientific_rows": 0,
    "truth_coefficient_values_read": 0,
    "truth_formulas_evaluated": 0,
}

FloatArray = NDArray[np.float64]


class C0StructuralCostError(RuntimeError):
    """The target-free V37 C0 structure or cost contract failed."""


@dataclass(frozen=True, slots=True)
class LoadedContext:
    profile: dict[str, Any]
    profile_data: bytes
    f0_profile: dict[str, Any]
    p0_profile: dict[str, Any]
    dependencies: dict[str, dict[str, object]]


@dataclass(frozen=True, slots=True)
class CaseSpec:
    role: str
    stratum: str
    case_id: str
    family: dict[str, Any]
    material: dict[str, Any]
    geometry: tuple[str, str, str]
    contact: tuple[str, str]
    surface_id: str
    surface_seed: int
    mixture_id: str


@dataclass(frozen=True, slots=True)
class P1Context:
    fixture: dict[str, Any]
    validated: dict[str, Any]
    frequencies: FloatArray
    indices: NDArray[np.uint32]


@dataclass(frozen=True, slots=True)
class BuiltRole:
    role: str
    batch: contract.SurfaceQueryBatchV1
    case_ids: tuple[str, ...]
    remesh_root_sha256: str
    permutation_root_sha256: str
    query_support_count: int
    exact_two_hop_pairs: int


@dataclass(frozen=True, slots=True)
class TensorRole:
    role: str
    node_features: torch.Tensor
    edge_summaries: torch.Tensor
    normalized_adjacency: torch.Tensor
    query_features: torch.Tensor
    context_features: torch.Tensor
    pointwise_features: torch.Tensor
    query_kernel_weights: torch.Tensor
    row_field_indices: torch.Tensor
    partition_keys: tuple[tuple[int, ...], ...]


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise C0StructuralCostError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise C0StructuralCostError(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise C0StructuralCostError(f"JSON root must be an object: {label}")
    document = cast(dict[str, Any], value)
    if canonical_json(document) != data:
        raise C0StructuralCostError(f"JSON is not canonical: {label}")
    return document


def validate_profile_boundary(profile: dict[str, Any]) -> dict[str, object]:
    prohibitions = cast(dict[str, Any], profile["structural_prohibitions"])
    forbidden_sections = set(
        cast(list[str], prohibitions["forbidden_profile_sections"])
    )
    present = sorted(forbidden_sections & set(profile))
    if present:
        raise C0StructuralCostError(
            f"forbidden value-bearing profile section: {','.join(present)}"
        )
    cost = cast(dict[str, Any], profile["cost_oracle"])
    if (
        cost.get("scientific_targets") != "forbidden"
        or cost.get("workload_values") != "constant-artificial-zero-only"
        or cost.get("artificial_axis_count") != contract.TARGET_AXIS_COUNT
    ):
        raise C0StructuralCostError("C0 artificial-zero boundary drift")
    return {
        "forbidden_profile_section_intersection": present,
        "scientific_targets": "forbidden",
        "workload_values": "constant-artificial-zero-only",
    }


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if not path.is_file() or path.is_symlink():
        raise C0StructuralCostError("profile must be a regular file")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise C0StructuralCostError("profile size outside bound")
    profile = load_json_bytes(data, "C0 profile")
    validate_profile_boundary(profile)
    if sha256_bytes(data) != PROFILE_SHA256:
        raise C0StructuralCostError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or profile.get("authority") != EXPECTED_AUTHORITY
    ):
        raise C0StructuralCostError("profile identity or authority mismatch")
    return profile, data


def validate_dependencies(
    profile: dict[str, Any],
) -> tuple[dict[str, dict[str, object]], dict[str, dict[str, Any]]]:
    declarations = cast(list[dict[str, str]], profile["dependencies"])
    paths = [declaration["path"] for declaration in declarations]
    if tuple(paths) != tuple(sorted(set(paths))):
        raise C0StructuralCostError("dependency paths are not canonical and unique")
    records: dict[str, dict[str, object]] = {}
    documents: dict[str, dict[str, Any]] = {}
    for declaration in declarations:
        path_text = declaration["path"]
        path = repository_root() / path_text
        if not path.is_file() or path.is_symlink():
            raise C0StructuralCostError(f"dependency is absent or linked: {path_text}")
        data = path.read_bytes()
        if sha256_bytes(data) != declaration["sha256"]:
            raise C0StructuralCostError(f"dependency drift: {path_text}")
        records[path_text] = {
            "bytes": len(data),
            "path": path_text,
            "sha256": declaration["sha256"],
        }
        if path.suffix == ".json":
            documents[path_text] = load_json_bytes(data, path_text)
    required = {F0_PROFILE_PATH, P0_PROFILE_PATH}
    if not required.issubset(documents):
        raise C0StructuralCostError("C0 JSON dependency closure is incomplete")
    return records, documents


def validate_import_boundary(profile: dict[str, Any]) -> dict[str, object]:
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    tree = ast.parse(owner_data)
    imports: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            imports.add(node.module.split(".")[0])
    prohibitions = cast(dict[str, Any], profile["structural_prohibitions"])
    forbidden = sorted(
        imports & set(cast(list[str], prohibitions["forbidden_import_stems"]))
    )
    if forbidden:
        raise C0StructuralCostError(f"forbidden owner import: {','.join(forbidden)}")
    required = {
        "physical_sound_v31_p1_modal_owner_v1",
        "physical_sound_v37_f0_fresh_operator_freeze_v1",
        "physical_sound_v37_query_surface_contract_v1",
    }
    if not required.issubset(imports):
        raise C0StructuralCostError("required F0/P1/contract import is absent")
    return {"forbidden_imports": forbidden, "imports": sorted(imports)}


def load_context(profile_path: Path) -> LoadedContext:
    profile, profile_data = load_profile(profile_path)
    dependencies, documents = validate_dependencies(profile)
    f0_profile = documents[F0_PROFILE_PATH]
    p0_profile = documents[P0_PROFILE_PATH]
    if sha256_bytes(canonical_json(f0_profile)) != f0.PROFILE_SHA256:
        raise C0StructuralCostError("bound F0 profile identity drift")
    f0.validate_science(f0_profile)
    commitments = f0.build_role_commitments(f0_profile)
    expected = cast(dict[str, Any], profile["expected_role_commitments"])
    actual_roles = cast(dict[str, Any], commitments["roles"])
    for role in ROLE_NAMES:
        for key, value in cast(dict[str, Any], expected[role]).items():
            if actual_roles[role][key] != value:
                raise C0StructuralCostError(f"F0 commitment drift: {role}/{key}")
    return LoadedContext(profile, profile_data, f0_profile, p0_profile, dependencies)


def line_root(identities: tuple[str, ...]) -> str:
    if not identities or len(identities) != len(set(identities)):
        raise C0StructuralCostError("identity root requires unique nonempty values")
    return sha256_bytes(("\n".join(sorted(identities)) + "\n").encode())


def role_kind(role: str) -> contract.RoleKind:
    if role == "train":
        return contract.RoleKind.TRAIN
    if role == "development":
        return contract.RoleKind.DEVELOPMENT
    if role == "method_holdout":
        return contract.RoleKind.METHOD_HOLDOUT
    raise C0StructuralCostError(f"unknown role: {role}")


def surface_seed_map(f0_profile: dict[str, Any]) -> dict[str, int]:
    corpus = cast(dict[str, Any], f0_profile["corpus"])
    grouped = cast(dict[str, list[dict[str, Any]]], corpus["surface_functions"])
    result: dict[str, int] = {}
    for role in ROLE_NAMES:
        for row in grouped[role]:
            result[cast(str, row["function_id"])] = cast(int, row["seed"])
    if len(result) != 14:
        raise C0StructuralCostError("surface seed map is incomplete")
    return result


def enumerate_cases(f0_profile: dict[str, Any], role: str) -> tuple[CaseSpec, ...]:
    if role not in ROLE_NAMES:
        raise C0StructuralCostError(f"unknown role: {role}")
    corpus = cast(dict[str, Any], f0_profile["corpus"])
    families = cast(list[dict[str, Any]], corpus["families"])
    materials = cast(list[dict[str, Any]], corpus["materials"])
    geometries = [tuple(row) for row in cast(list[list[str]], corpus["geometry_cells"])]
    contacts = cast(dict[str, list[list[str]]], corpus["contacts"])
    surfaces = cast(dict[str, list[dict[str, Any]]], corpus["surface_functions"])
    mixtures = cast(dict[str, list[dict[str, Any]]], corpus["operator_mixtures"])
    seeds = cast(dict[str, Any], f0_profile["seeds"])
    role_seeds = cast(dict[str, int], seeds["role_seeds"])
    prefix = cast(str, corpus["role_identity_prefix"])
    truth_ids = f0.truth_formula_ids(f0_profile)
    seed_by_surface = surface_seed_map(f0_profile)
    plans = cast(dict[str, list[dict[str, Any]]], corpus["role_plan"])
    result: list[CaseSpec] = []
    ordinal = 0
    for entry in plans[role]:
        surface_rows = surfaces[cast(str, entry["surface_set"])]
        mixture_rows = mixtures[cast(str, entry["mixture_set"])]
        contact_rows = contacts[cast(str, entry["contact_set"])]
        for family in families:
            for material in materials:
                for geometry_index in cast(list[int], entry["geometry_cells"]):
                    for contact_index in cast(list[int], entry["contact_indices"]):
                        surface_id = cast(
                            str,
                            surface_rows[ordinal % len(surface_rows)]["function_id"],
                        )
                        mixture_id = cast(
                            str,
                            mixture_rows[ordinal % len(mixture_rows)]["mixture_id"],
                        )
                        geometry = cast(
                            tuple[str, str, str], geometries[geometry_index]
                        )
                        contact_values = contact_rows[contact_index]
                        contact_pair = (contact_values[0], contact_values[1])
                        physical = {
                            "contact": contact_pair,
                            "family": family,
                            "geometry": geometry,
                            "material": material,
                            "operator_mixture_id": mixture_id,
                            "surface_function_id": surface_id,
                        }
                        physical_root = sha256_bytes(canonical_json(physical))
                        case_payload = {
                            "case_enumeration_seed": seeds["case_enumeration_seed"],
                            "physical_root_sha256": physical_root,
                            "prefix": prefix,
                            "profile_sha256": f0.PROFILE_SHA256,
                            "role": role,
                            "role_seed": role_seeds[role],
                            "stratum": entry["stratum"],
                            "truth_formula_ids": truth_ids,
                        }
                        case_id = "case-" + sha256_bytes(canonical_json(case_payload))
                        result.append(
                            CaseSpec(
                                role=role,
                                stratum=cast(str, entry["stratum"]),
                                case_id=case_id,
                                family=family,
                                material=material,
                                geometry=geometry,
                                contact=contact_pair,
                                surface_id=surface_id,
                                surface_seed=seed_by_surface[surface_id],
                                mixture_id=mixture_id,
                            )
                        )
                        ordinal += 1
    expected = cast(dict[str, Any], f0.EXPECTED_CASE_COUNTS)[role]
    if len(result) != expected or len({case.case_id for case in result}) != expected:
        raise C0StructuralCostError(f"case enumeration drift: {role}")
    return tuple(result)


def base_fixture(
    family: dict[str, Any],
    material: dict[str, Any],
    geometry: tuple[str, str, str],
    p0_profile: dict[str, Any],
) -> dict[str, Any]:
    family_name = cast(str, family["family_id"]).replace("-", "_")
    fixtures = cast(list[dict[str, Any]], p0_profile["fixtures"])
    source = next(
        (fixture for fixture in fixtures if fixture["family"] == family_name), None
    )
    if source is None:
        raise C0StructuralCostError(f"P1 base fixture missing: {family_name}")
    fixture = copy.deepcopy(source)
    fixture["fixture_id"] = "v37-c0-structural-context"
    fixture["formula_id"] = family["formula_id"]
    fixture["support"] = family["support_id"]
    fixture["material"] = {
        "density_kg_m3": material["density_kg_m3"],
        "loss_rate_per_second": material["loss_rate_per_second"],
        "material_id": "synthetic-elastic-reference",
        "poisson_ratio": material["poisson_ratio"],
        "youngs_modulus_pa": material["youngs_modulus_pa"],
    }
    component_order = (
        ("length_x_m", "length_y_m", "thickness_m")
        if family_name == "rectangular_plate"
        else ("length_m", "width_m", "height_m")
    )
    for component, multiplier in zip(component_order, geometry, strict=True):
        fixture["geometry"][component] = format(
            Decimal(fixture["geometry"][component]) * Decimal(multiplier), "f"
        )
    fixture["contact"] = {"normal_impulse_ns": "1", "u": "0.5", "v": "0.5"}
    fixture["mode_count"] = 10
    return fixture


def p1_context(
    case: CaseSpec,
    p0_profile: dict[str, Any],
    cache: dict[str, P1Context],
) -> P1Context:
    key = sha256_bytes(
        canonical_json(
            {
                "family": case.family,
                "geometry": case.geometry,
                "material": case.material,
            }
        )
    )
    if key not in cache:
        fixture = base_fixture(case.family, case.material, case.geometry, p0_profile)
        validated = p1.validate_fixture(fixture, p0_profile)
        frequencies, _decay, indices = p1.solve_modes(fixture, p0_profile)
        cache[key] = P1Context(fixture, validated, frequencies, indices)
    return cache[key]


def field_id(case: CaseSpec, mode_ordinal: int) -> str:
    return "field-" + sha256_bytes(
        canonical_json(
            {
                "family": case.family,
                "geometry": case.geometry,
                "material": case.material,
                "mode_ordinal": mode_ordinal,
                "profile_sha256": f0.PROFILE_SHA256,
                "surface_function_id": case.surface_id,
            }
        )
    )


def surface_envelope(
    seed: int, u: FloatArray | float, v: FloatArray | float
) -> FloatArray:
    u_values = np.asarray(u, dtype=np.float64)
    v_values = np.asarray(v, dtype=np.float64)
    a = 1 + seed % 5
    b = 1 + (seed // 5) % 7
    phase = ((seed // 35) % 17) / 17.0
    values = (
        1.0
        + 0.06 * np.sin(2.0 * math.pi * (a * u_values + b * v_values + phase))
        + 0.03 * (2.0 * u_values - 1.0) * (2.0 * v_values - 1.0)
    )
    if np.any(values < 0.91 - 1.0e-12) or np.any(~np.isfinite(values)):
        raise C0StructuralCostError("surface envelope positivity drift")
    return np.asarray(values, dtype=np.float64)


def context_values(
    names: tuple[str, ...], case: CaseSpec, context: P1Context, mode_ordinal: int
) -> tuple[float, ...]:
    family = cast(str, case.family["family_id"])
    support = cast(str, case.family["support_id"])
    density = float(case.material["density_kg_m3"])
    youngs = float(case.material["youngs_modulus_pa"])
    values = {
        "family.beam": float(family == "rectangular-beam"),
        "family.plate": float(family == "rectangular-plate"),
        "geometry.log_multiplier_0": math.log(float(case.geometry[0])) / math.log(1.25),
        "geometry.log_multiplier_1": math.log(float(case.geometry[1])) / math.log(1.25),
        "geometry.log_multiplier_2": math.log(float(case.geometry[2])) / math.log(1.25),
        "material.log_density": math.log2(density / 5000.0),
        "material.log_youngs": math.log2(youngs / 100_000_000_000.0),
        "material.loss": float(case.material["loss_rate_per_second"]) / 3.0,
        "material.poisson": (float(case.material["poisson_ratio"]) - 0.275) / 0.1,
        "mode.log_frequency": max(
            -1.5,
            min(
                1.5, math.log2(float(context.frequencies[mode_ordinal]) / 1000.0) / 4.0
            ),
        ),
        "mode.ordinal_signed": 2.0 * mode_ordinal / 9.0 - 1.0,
        "support.beam_cantilever": float(support == "cantilever-clamped-u0"),
        "support.beam_simple": float(support == "simply-supported-both-ends"),
        "support.plate_simple": float(support == "simply-supported-all-edges"),
    }
    if tuple(sorted(values)) != names:
        raise C0StructuralCostError("context feature contract drift")
    result = tuple(values[name] for name in names)
    if not all(math.isfinite(value) for value in result):
        raise C0StructuralCostError("context feature is non-finite")
    return result


def normalized_probe_grid() -> tuple[FloatArray, FloatArray]:
    coordinates = np.linspace(0.0, 1.0, 3, dtype=np.float64)
    u = np.tile(coordinates, 3)
    v = np.repeat(coordinates, 3)
    return u, v


def physical_positions(context: P1Context, u: FloatArray, v: FloatArray) -> FloatArray:
    dimensions = cast(dict[str, float], context.validated["dimensions"])
    if context.validated["family"] == "rectangular_plate":
        rows = np.column_stack(
            (
                dimensions["length_x_m"] * u,
                dimensions["length_y_m"] * v,
                np.full_like(u, dimensions["thickness_m"] / 2.0),
            )
        )
    else:
        rows = np.column_stack(
            (
                dimensions["length_m"] * u,
                dimensions["width_m"] * (v - 0.5),
                np.full_like(u, dimensions["height_m"] / 2.0),
            )
        )
    return np.asarray(rows, dtype=np.float64)


def probe_neighbors(field_identity: str, v_index: int, u_index: int) -> tuple[str, ...]:
    rows: list[str] = []
    for delta_v, delta_u in ((-1, 0), (0, -1), (0, 1), (1, 0)):
        neighbor_v = v_index + delta_v
        neighbor_u = u_index + delta_u
        if 0 <= neighbor_v < 3 and 0 <= neighbor_u < 3:
            rows.append(f"{field_identity}:probe-v{neighbor_v:02d}-u{neighbor_u:02d}")
    return tuple(sorted(rows))


def field_input(
    case: CaseSpec,
    mode_ordinal: int,
    context: P1Context,
    p0_profile: dict[str, Any],
    *,
    reverse_source_enumeration: bool,
) -> contract.FieldInputV1:
    identity = field_id(case, mode_ordinal)
    u, v = normalized_probe_grid()
    positions = physical_positions(context, u, v)
    normals = np.tile(np.asarray([[0.0, 0.0, 1.0]], dtype=np.float64), (9, 1))
    one_dimensional_weights = np.asarray([0.25, 0.5, 0.25], dtype=np.float64)
    normalized_areas = np.outer(
        one_dimensional_weights, one_dimensional_weights
    ).ravel()
    dimensions = cast(dict[str, float], context.validated["dimensions"])
    surface_area = (
        dimensions["length_x_m"] * dimensions["length_y_m"]
        if context.validated["family"] == "rectangular_plate"
        else dimensions["length_m"] * dimensions["width_m"]
    )
    mode_values = p1.participation(
        context.validated, p0_profile, context.indices, u, v
    )[:, mode_ordinal] * surface_envelope(case.surface_seed, u, v)
    probes = [
        contract.ProbeInputV1(
            probe_id=f"{identity}:probe-v{index // 3:02d}-u{index % 3:02d}",
            position=cast(tuple[float, float, float], tuple(positions[index])),
            normal=cast(tuple[float, float, float], tuple(normals[index])),
            area_weight=float(surface_area * normalized_areas[index]),
            mode_value=float(mode_values[index]),
            neighbor_ids=probe_neighbors(identity, index // 3, index % 3),
        )
        for index in range(9)
    ]
    if reverse_source_enumeration:
        probes.reverse()
        probes = [
            contract.ProbeInputV1(
                probe.probe_id,
                probe.position,
                probe.normal,
                probe.area_weight,
                probe.mode_value,
                tuple(reversed(probe.neighbor_ids)),
            )
            for probe in probes
        ]
    return contract.FieldInputV1(identity, tuple(probes))


def query_support(u: float, v: float) -> tuple[str, tuple[float, float, float]]:
    scaled_u = 2.0 * u
    scaled_v = 2.0 * v
    cell_u = min(math.floor(scaled_u), 1)
    cell_v = min(math.floor(scaled_v), 1)
    local_u = scaled_u - cell_u
    local_v = scaled_v - cell_v
    if local_u + local_v <= 1.0:
        triangle = f"tri-v{cell_v:02d}-u{cell_u:02d}-lower"
        barycentrics = (1.0 - local_u - local_v, local_u, local_v)
    else:
        triangle = f"tri-v{cell_v:02d}-u{cell_u:02d}-upper"
        barycentrics = (local_u + local_v - 1.0, 1.0 - local_u, 1.0 - local_v)
    if any(value < -1.0e-15 or value > 1.0 + 1.0e-15 for value in barycentrics):
        raise C0StructuralCostError("query barycentric support is invalid")
    return triangle, barycentrics


def query_input(
    case: CaseSpec,
    mode_ordinal: int,
    context: P1Context,
    p0_profile: dict[str, Any],
    context_names: tuple[str, ...],
) -> contract.QueryInputV1:
    u = float(case.contact[0])
    v = float(case.contact[1])
    identity = field_id(case, mode_ordinal)
    triangle, barycentrics = query_support(u, v)
    position = physical_positions(
        context,
        np.asarray([u], dtype=np.float64),
        np.asarray([v], dtype=np.float64),
    )[0]
    raw_value = p1.participation(context.validated, p0_profile, context.indices, u, v)[
        0, mode_ordinal
    ]
    p1_value = float(raw_value * surface_envelope(case.surface_seed, u, v))
    return contract.QueryInputV1(
        row_id=f"{case.case_id}:mode-{mode_ordinal:02d}",
        field_id=identity,
        triangle_id=f"{identity}:{triangle}",
        position=cast(tuple[float, float, float], tuple(position)),
        normal=(0.0, 0.0, 1.0),
        barycentrics=barycentrics,
        context_values=context_values(context_names, case, context, mode_ordinal),
        p1_contact_value=p1_value,
    )


def validate_query_support(batch: contract.SurfaceQueryBatchV1) -> int:
    fields = batch.fields
    probe_index = {probe_id: index for index, probe_id in enumerate(fields.probe_ids)}
    supported = 0
    for row_index, triangle_id in enumerate(batch.query_triangle_ids):
        try:
            field_identity, suffix = triangle_id.rsplit(":tri-", maxsplit=1)
            tokens = suffix.split("-")
            cell_v = int(tokens[0][1:])
            cell_u = int(tokens[1][1:])
            side = tokens[2]
        except (ValueError, IndexError) as error:
            raise C0StructuralCostError(
                "query triangle identity is malformed"
            ) from error
        field_index = int(batch.row_field_indices[row_index])
        if fields.field_ids[field_index] != field_identity:
            raise C0StructuralCostError("query triangle references the wrong field")
        if side == "lower":
            vertices = ((cell_v, cell_u), (cell_v, cell_u + 1), (cell_v + 1, cell_u))
        elif side == "upper":
            vertices = (
                (cell_v + 1, cell_u + 1),
                (cell_v + 1, cell_u),
                (cell_v, cell_u + 1),
            )
        else:
            raise C0StructuralCostError("query triangle side is invalid")
        indices = [
            probe_index[f"{field_identity}:probe-v{v:02d}-u{u:02d}"]
            for v, u in vertices
        ]
        barycentrics = batch.query_barycentrics[row_index]
        reconstructed = sum(
            float(barycentrics[index]) * fields.probe_positions[probe]
            for index, probe in enumerate(indices)
        )
        if not np.array_equal(
            reconstructed, batch.query_positions[row_index]
        ) and not np.allclose(
            reconstructed,
            batch.query_positions[row_index],
            rtol=0.0,
            atol=1.0e-15,
        ):
            raise C0StructuralCostError("query position is outside canonical support")
        supported += 1
    return supported


def exact_two_hop_pairs(fields: contract.SurfaceFieldSetV1) -> int:
    total = 0
    for field_index in range(fields.field_count):
        start = int(fields.field_offsets[field_index])
        end = int(fields.field_offsets[field_index + 1])
        if end - start != 9:
            raise C0StructuralCostError("canonical probe count drift")
        adjacency = np.zeros((9, 9), dtype=np.int8)
        for source in range(start, end):
            edge_start = int(fields.edge_offsets[source])
            edge_end = int(fields.edge_offsets[source + 1])
            for target in fields.edge_indices[edge_start:edge_end]:
                adjacency[source - start, int(target) - start] = 1
        two_hop = (adjacency @ adjacency) > 0
        total += int(
            np.count_nonzero(two_hop & (adjacency == 0) & ~np.eye(9, dtype=bool))
        )
    if total < fields.field_count * 4:
        raise C0StructuralCostError("two-hop topology reachability shortfall")
    return total


def build_role(context: LoadedContext, role: str) -> BuiltRole:
    cases = enumerate_cases(context.f0_profile, role)
    context_names = tuple(
        cast(
            list[str],
            cast(dict[str, Any], context.f0_profile["science"])[
                "context_feature_names"
            ],
        )
    )
    p1_cache: dict[str, P1Context] = {}
    field_specs: dict[str, tuple[CaseSpec, int, P1Context]] = {}
    queries: list[contract.QueryInputV1] = []
    for case in cases:
        modal_context = p1_context(case, context.p0_profile, p1_cache)
        for mode_ordinal in range(10):
            identity = field_id(case, mode_ordinal)
            field_specs.setdefault(identity, (case, mode_ordinal, modal_context))
            queries.append(
                query_input(
                    case,
                    mode_ordinal,
                    modal_context,
                    context.p0_profile,
                    context_names,
                )
            )
    fields_input = tuple(
        field_input(
            case,
            mode,
            modal_context,
            context.p0_profile,
            reverse_source_enumeration=False,
        )
        for case, mode, modal_context in field_specs.values()
    )
    fields = contract.canonical_surface_fields(fields_input)
    batch = contract.canonical_query_batch(
        role_kind(role), fields, context_names, tuple(queries), targets=None
    )
    reverse_fields = contract.canonical_surface_fields(
        tuple(
            reversed(
                tuple(
                    field_input(
                        case,
                        mode,
                        modal_context,
                        context.p0_profile,
                        reverse_source_enumeration=True,
                    )
                    for case, mode, modal_context in field_specs.values()
                )
            )
        )
    )
    if reverse_fields.root_sha256 != fields.root_sha256:
        raise C0StructuralCostError("equivalent remesh field root diverged")
    permuted_queries = tuple(
        contract.QueryInputV1(
            row_id=query.row_id,
            field_id=query.field_id,
            triangle_id=query.triangle_id,
            position=query.position,
            normal=query.normal,
            barycentrics=query.barycentrics,
            context_values=tuple(reversed(query.context_values)),
            p1_contact_value=query.p1_contact_value,
        )
        for query in reversed(queries)
    )
    permuted = contract.canonical_query_batch(
        role_kind(role),
        reverse_fields,
        tuple(reversed(context_names)),
        permuted_queries,
        targets=None,
    )
    if permuted.structural_root_sha256 != batch.structural_root_sha256:
        raise C0StructuralCostError("input permutation changed structural root")
    expected = cast(
        dict[str, Any],
        cast(dict[str, Any], context.profile["expected_role_commitments"])[role],
    )
    case_ids = tuple(case.case_id for case in cases)
    if (
        len(case_ids) != expected["case_count"]
        or line_root(case_ids) != expected["case_root_sha256"]
        or fields.field_count != expected["field_count"]
        or f0.merkle_root(set(fields.field_ids)) != expected["field_root_sha256"]
        or batch.row_count != expected["modal_row_count"]
        or f0.merkle_root(set(batch.row_ids)) != expected["modal_row_root_sha256"]
        or batch.has_targets
    ):
        raise C0StructuralCostError(f"full-shape F0 commitment mismatch: {role}")
    supported = validate_query_support(batch)
    hops = exact_two_hop_pairs(fields)
    return BuiltRole(
        role=role,
        batch=batch,
        case_ids=case_ids,
        remesh_root_sha256=reverse_fields.root_sha256,
        permutation_root_sha256=permuted.structural_root_sha256,
        query_support_count=supported,
        exact_two_hop_pairs=hops,
    )


class MemoryStructuralProvider:
    def __init__(
        self,
        kind: contract.ProviderKind,
        namespace: str,
        batches: dict[contract.RoleKind, contract.SurfaceQueryBatchV1],
    ) -> None:
        self._kind = kind
        self._namespace = namespace
        self._batches = batches

    @property
    def kind(self) -> contract.ProviderKind:
        return self._kind

    @property
    def namespace(self) -> str:
        return self._namespace

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapabilityV1
    ) -> contract.SurfaceQueryBatchV1:
        if (
            capability.provider_kind is not self.kind
            or capability.namespace != self.namespace
        ):
            raise contract.ContractError("structural provider capability mismatch")
        try:
            return self._batches[role]
        except KeyError as error:
            raise contract.ContractError("structural provider role absent") from error


def structural_trace(
    roles: dict[str, BuiltRole], kind: contract.ProviderKind
) -> contract.ExecutionTraceV1:
    pipeline = contract.pipeline_for_provider(kind)
    namespace = f"structural-v37-c0-{pipeline.value}"
    capability = contract.AccessCapabilityV1.structural(kind, namespace)
    batches = {role_kind(role): built.batch for role, built in roles.items()}
    provider = MemoryStructuralProvider(kind, namespace, batches)
    lifecycle = contract.OwnerLifecycleV1(pipeline, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    if pipeline is contract.PipelineKind.D0:
        lifecycle.materialize_role(
            contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
        )
        lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
        lifecycle.materialize_role(
            contract.LifecycleStage.DEVELOPMENT_ROLE,
            contract.RoleKind.DEVELOPMENT,
            provider,
        )
    else:
        lifecycle.materialize_role(
            contract.LifecycleStage.TRAIN_RECONSTRUCTION,
            contract.RoleKind.TRAIN,
            provider,
        )
        lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
        lifecycle.materialize_role(
            contract.LifecycleStage.METHOD_HOLDOUT_ROLE,
            contract.RoleKind.METHOD_HOLDOUT,
            provider,
        )
    lifecycle.step(contract.LifecycleStage.QUERY_EVALUATION)
    lifecycle.step(contract.LifecycleStage.HARD_GATES)
    lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
    trace = lifecycle.finish(contract.TerminalDecision.PREFLIGHT_PASS)
    contract.assert_complete_trace(trace)
    if (
        trace.access.target_rows_accessed != 0
        or trace.access.forbidden_access_count != 0
    ):
        raise C0StructuralCostError("structural lifecycle opened target access")
    return trace


def tensor_role(built: BuiltRole) -> TensorRole:
    batch = built.batch
    fields = batch.fields
    offsets = np.diff(fields.field_offsets)
    if not np.all(offsets == 9):
        raise C0StructuralCostError("cost path requires the frozen nine-probe grid")
    field_count = fields.field_count
    context_width = len(batch.context_feature_names)
    field_context = np.empty((field_count, context_width), dtype=np.float64)
    assigned = np.zeros(field_count, dtype=bool)
    for row, field_index_raw in enumerate(batch.row_field_indices):
        field_index = int(field_index_raw)
        if assigned[field_index]:
            if not np.array_equal(
                field_context[field_index], batch.context_features[row]
            ):
                raise C0StructuralCostError("field context is not query-invariant")
        else:
            field_context[field_index] = batch.context_features[row]
            assigned[field_index] = True
    if not np.all(assigned):
        raise C0StructuralCostError("a canonical field is unreachable from queries")
    positions = fields.probe_positions.reshape(field_count, 9, 3)
    normals = fields.probe_normals.reshape(field_count, 9, 3)
    areas = fields.probe_area_weights.reshape(field_count, 9, 1)
    modes = fields.probe_mode_values.reshape(field_count, 9, 1)
    expanded_context = np.repeat(field_context[:, None, :], 9, axis=1)
    node_features = np.concatenate(
        (positions, normals, areas, modes, expanded_context), axis=2
    )
    if node_features.shape[2] != 22:
        raise C0StructuralCostError("node feature width drift")
    adjacency = np.zeros((9, 9), dtype=np.float64)
    for field_index in range(field_count):
        start = int(fields.field_offsets[field_index])
        local = np.zeros((9, 9), dtype=np.float64)
        for source in range(start, start + 9):
            edge_start = int(fields.edge_offsets[source])
            edge_end = int(fields.edge_offsets[source + 1])
            for target in fields.edge_indices[edge_start:edge_end]:
                local[source - start, int(target) - start] = 1.0
        if field_index == 0:
            adjacency = local
        elif not np.array_equal(local, adjacency):
            raise C0StructuralCostError("canonical adjacency varies by field")
    degrees = np.sum(adjacency, axis=1)
    normalized = adjacency / np.sqrt(degrees[:, None] * degrees[None, :])
    neighbor_positions = (
        np.einsum("ij,fjk->fik", adjacency, positions) / degrees[None, :, None]
    )
    scale = np.maximum(np.ptp(positions, axis=1).max(axis=1), 1.0e-12)
    deltas = (neighbor_positions - positions) / scale[:, None, None]
    degree_feature = np.broadcast_to(degrees[None, :, None] / 4.0, (field_count, 9, 1))
    edge_summaries = np.concatenate((deltas, degree_feature), axis=2)
    row_fields = np.asarray(batch.row_field_indices, dtype=np.int64)
    query_positions = batch.query_positions
    gathered_positions = positions[row_fields]
    gathered_areas = areas[row_fields, :, 0]
    difference = gathered_positions - query_positions[:, None, :]
    distance_squared = np.sum(difference * difference, axis=2)
    row_scale = np.maximum(np.ptp(gathered_positions, axis=1).max(axis=1), 1.0e-12)
    kernel = gathered_areas * np.exp(
        -distance_squared / (2.0 * (0.55 * row_scale[:, None]) ** 2)
    )
    kernel /= np.sum(kernel, axis=1, keepdims=True)
    query_features = np.concatenate(
        (
            batch.query_positions,
            batch.query_normals,
            batch.query_barycentrics,
            batch.p1_contact_values[:, None],
            batch.context_features,
        ),
        axis=1,
    )
    if query_features.shape[1] != 24:
        raise C0StructuralCostError("query feature width drift")
    gathered_modes = modes[row_fields, :, 0]
    weighted_mean = np.sum(kernel * gathered_modes, axis=1, keepdims=True)
    weighted_variance = np.sum(
        kernel * (gathered_modes - weighted_mean) ** 2, axis=1, keepdims=True
    )
    local_summary = np.concatenate(
        (gathered_modes, weighted_mean, weighted_variance), axis=1
    )
    pointwise = np.concatenate((query_features, local_summary), axis=1)
    if pointwise.shape[1] != 35:
        raise C0StructuralCostError("V36-shaped pointwise width drift")
    index = {name: offset for offset, name in enumerate(batch.context_feature_names)}
    partition_names = (
        "family.beam",
        "family.plate",
        "mode.ordinal_signed",
        "support.beam_cantilever",
        "support.beam_simple",
        "support.plate_simple",
    )
    partition_keys = tuple(
        tuple(round(float(row[index[name]]) * 1_000_000.0) for name in partition_names)
        for row in batch.context_features
    )
    arrays = (
        node_features,
        edge_summaries,
        normalized,
        query_features,
        pointwise,
        kernel,
    )
    if any(np.any(~np.isfinite(value)) for value in arrays):
        raise C0StructuralCostError("cost tensor contains non-finite values")
    return TensorRole(
        role=built.role,
        node_features=torch.from_numpy(np.ascontiguousarray(node_features)),
        edge_summaries=torch.from_numpy(np.ascontiguousarray(edge_summaries)),
        normalized_adjacency=torch.from_numpy(np.ascontiguousarray(normalized)),
        query_features=torch.from_numpy(np.ascontiguousarray(query_features)),
        context_features=torch.from_numpy(
            np.array(batch.context_features, dtype=np.float64, order="C", copy=True)
        ),
        pointwise_features=torch.from_numpy(np.ascontiguousarray(pointwise)),
        query_kernel_weights=torch.from_numpy(np.ascontiguousarray(kernel)),
        row_field_indices=torch.from_numpy(
            np.array(row_fields, dtype=np.int64, order="C", copy=True)
        ),
        partition_keys=partition_keys,
    )


class TwoLinear(nn.Module):
    def __init__(self, input_width: int, hidden_width: int, output_width: int) -> None:
        super().__init__()
        self.first = nn.Linear(input_width, hidden_width, dtype=torch.float64)
        self.second = nn.Linear(hidden_width, output_width, dtype=torch.float64)

    def forward(self, values: torch.Tensor) -> torch.Tensor:
        return torch.nn.functional.silu(
            self.second(torch.nn.functional.silu(self.first(values)))
        )


class BoundedHead(nn.Module):
    def __init__(self, input_width: int, bound: float) -> None:
        super().__init__()
        self.first = nn.Linear(input_width, 16, dtype=torch.float64)
        self.second = nn.Linear(16, 16, dtype=torch.float64)
        self.output = nn.Linear(16, 1, dtype=torch.float64)
        self.bound = bound

    def forward(self, values: torch.Tensor) -> torch.Tensor:
        hidden = torch.nn.functional.silu(self.first(values))
        hidden = torch.nn.functional.silu(self.second(hidden))
        return self.bound * torch.tanh(self.output(hidden))


class QuerySurfaceCostModel(nn.Module):
    def __init__(self, seed: int) -> None:
        super().__init__()
        self.node_encoder = TwoLinear(22, 24, 24)
        self.messages = nn.ModuleList(TwoLinear(52, 24, 24) for _ in range(2))
        self.updates = nn.ModuleList(TwoLinear(48, 24, 24) for _ in range(2))
        self.query_encoder = TwoLinear(24, 24, 24)
        self.contact_head = TwoLinear(73, 24, 1)
        self.decay_head = BoundedHead(11, 0.25)
        self.global_gain_head = BoundedHead(13, 0.20)
        initialize_cost_parameters(self, seed)

    def encode_fields(
        self,
        role: TensorRole,
        *,
        topology: bool,
        field_indices: torch.Tensor | None = None,
    ) -> torch.Tensor:
        node_features = (
            role.node_features
            if field_indices is None
            else role.node_features[field_indices]
        )
        edge_summaries = (
            role.edge_summaries
            if field_indices is None
            else role.edge_summaries[field_indices]
        )
        hidden: torch.Tensor = self.node_encoder(node_features)
        for message, update in zip(self.messages, self.updates, strict=True):
            if topology:
                neighbors = torch.einsum(
                    "ij,fjk->fik", role.normalized_adjacency, hidden
                )
                encoded_message = message(
                    torch.cat((hidden, neighbors, edge_summaries), dim=2)
                )
                hidden = hidden + cast(
                    torch.Tensor,
                    update(torch.cat((hidden, encoded_message), dim=2)),
                )
        return hidden

    def predict_from_encoded(
        self,
        encoded: torch.Tensor,
        role: TensorRole,
        rows: torch.Tensor,
        variant: str,
        local_row_field_indices: torch.Tensor | None = None,
    ) -> torch.Tensor:
        field_indices = (
            role.row_field_indices[rows]
            if local_row_field_indices is None
            else local_row_field_indices
        )
        field_rows = encoded[field_indices]
        kernel = role.query_kernel_weights[rows]
        branch = torch.sum(kernel[:, :, None] * field_rows, dim=1)
        trunk = self.query_encoder(role.query_features[rows])
        if variant == "qso-field-only-ablation-v1":
            trunk = torch.zeros_like(trunk)
            product = torch.zeros_like(branch)
        elif variant == "qso-query-only-ablation-v1":
            branch = torch.zeros_like(branch)
            product = torch.zeros_like(trunk)
        elif variant == "qso-no-interaction-ablation-v1":
            product = torch.zeros_like(branch)
        else:
            product = branch * trunk
        p1_value = role.query_features[rows, 9:10]
        contact_value = 0.25 * torch.tanh(
            self.contact_head(torch.cat((branch, trunk, product, p1_value), dim=1))
        )
        context = role.context_features[rows]
        decay = self.decay_head(context[:, :11])
        gain = self.global_gain_head(context[:, :13])
        return torch.cat((decay, gain, contact_value), dim=1)


class PointwiseCostModel(nn.Module):
    def __init__(self, seed: int) -> None:
        super().__init__()
        self.contact_head = BoundedHead(35, 0.25)
        self.decay_head = BoundedHead(11, 0.25)
        self.global_gain_head = BoundedHead(13, 0.20)
        initialize_cost_parameters(self, seed)

    def forward(self, role: TensorRole, rows: torch.Tensor) -> torch.Tensor:
        context = role.context_features[rows]
        return torch.cat(
            (
                self.decay_head(context[:, :11]),
                self.global_gain_head(context[:, :13]),
                self.contact_head(role.pointwise_features[rows]),
            ),
            dim=1,
        )


def initialize_cost_parameters(model: nn.Module, seed: int) -> None:
    torch.manual_seed(seed)
    for module in model.modules():
        if isinstance(module, nn.Linear):
            nn.init.xavier_uniform_(module.weight)
            nn.init.zeros_(module.bias)


def parameter_count(model: nn.Module) -> int:
    return sum(parameter.numel() for parameter in model.parameters())


def digest_tensors(values: list[torch.Tensor]) -> str:
    digest = hashlib.sha256()
    for value in values:
        array = value.detach().cpu().numpy().astype("<f8", copy=False)
        digest.update(array.tobytes(order="C"))
    return digest.hexdigest()


def train_qso_cost(
    variant: str,
    seed: int,
    roles: dict[str, TensorRole],
    steps: int,
    batch_rows: int,
    microbatch_rows: int,
) -> dict[str, object]:
    model = QuerySurfaceCostModel(seed)
    if parameter_count(model) != 12_443:
        raise C0StructuralCostError(f"QSO parameter count drift: {variant}")
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.002, weight_decay=0.0)
    train = roles["train"]
    row_count = train.query_features.shape[0]
    cache_peak = 0
    topology = variant != "qso-no-topology-ablation-v1"
    for step in range(steps):
        rows = (
            torch.arange(batch_rows, dtype=torch.int64) + step * batch_rows
        ) % row_count
        optimizer.zero_grad(set_to_none=True)
        for start in range(0, batch_rows, microbatch_rows):
            micro_rows = rows[start : start + microbatch_rows]
            batch_field_indices, local_row_fields = torch.unique(
                train.row_field_indices[micro_rows], sorted=True, return_inverse=True
            )
            encoded = model.encode_fields(
                train, topology=topology, field_indices=batch_field_indices
            )
            cache_peak = max(cache_peak, encoded.numel() * encoded.element_size())
            predictions = model.predict_from_encoded(
                encoded,
                train,
                micro_rows,
                variant,
                local_row_field_indices=local_row_fields,
            )
            loss = torch.sum(predictions * predictions) / (
                batch_rows * contract.TARGET_AXIS_COUNT
            )
            if not bool(torch.isfinite(loss)):
                raise C0StructuralCostError(
                    f"non-finite artificial QSO cost: {variant}"
                )
            loss.backward()  # type: ignore[no-untyped-call]
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
    outputs: list[torch.Tensor] = []
    with torch.no_grad():
        for role in ROLE_NAMES:
            tensor_role_value = roles[role]
            encoded = model.encode_fields(tensor_role_value, topology=topology)
            cache_peak = max(cache_peak, encoded.numel() * encoded.element_size())
            chunks = []
            for start in range(
                0, tensor_role_value.query_features.shape[0], batch_rows
            ):
                rows = torch.arange(
                    start,
                    min(start + batch_rows, tensor_role_value.query_features.shape[0]),
                    dtype=torch.int64,
                )
                chunks.append(
                    model.predict_from_encoded(
                        encoded, tensor_role_value, rows, variant
                    )
                )
            outputs.append(torch.cat(chunks, dim=0))
    if cache_peak > 33_554_432:
        raise C0StructuralCostError(f"field cache ceiling exceeded: {variant}")
    return {
        "artificial_output_sha256": digest_tensors(outputs),
        "field_cache_peak_bytes": cache_peak,
        "parameter_count": parameter_count(model),
        "logical_batch_rows": batch_rows,
        "gradient_microbatch_rows": microbatch_rows,
        "steps": steps,
    }


def train_pointwise_cost(
    seed: int,
    roles: dict[str, TensorRole],
    steps: int,
    batch_rows: int,
    microbatch_rows: int,
) -> dict[str, object]:
    model = PointwiseCostModel(seed)
    if parameter_count(model) != 1_859:
        raise C0StructuralCostError("pointwise parameter count drift")
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.002, weight_decay=0.0)
    train = roles["train"]
    row_count = train.query_features.shape[0]
    for step in range(steps):
        rows = (
            torch.arange(batch_rows, dtype=torch.int64) + step * batch_rows
        ) % row_count
        optimizer.zero_grad(set_to_none=True)
        for start in range(0, batch_rows, microbatch_rows):
            micro_rows = rows[start : start + microbatch_rows]
            predictions = model(train, micro_rows)
            loss = torch.sum(predictions * predictions) / (
                batch_rows * contract.TARGET_AXIS_COUNT
            )
            if not bool(torch.isfinite(loss)):
                raise C0StructuralCostError("non-finite artificial pointwise cost")
            loss.backward()  # type: ignore[no-untyped-call]
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
    outputs: list[torch.Tensor] = []
    with torch.no_grad():
        for role in ROLE_NAMES:
            tensor_role_value = roles[role]
            chunks = []
            for start in range(
                0, tensor_role_value.query_features.shape[0], batch_rows
            ):
                rows = torch.arange(
                    start,
                    min(start + batch_rows, tensor_role_value.query_features.shape[0]),
                    dtype=torch.int64,
                )
                chunks.append(model(tensor_role_value, rows))
            outputs.append(torch.cat(chunks, dim=0))
    return {
        "artificial_output_sha256": digest_tensors(outputs),
        "field_cache_peak_bytes": 0,
        "parameter_count": parameter_count(model),
        "logical_batch_rows": batch_rows,
        "gradient_microbatch_rows": microbatch_rows,
        "steps": steps,
    }


def non_neural_cost(roles: dict[str, TensorRole]) -> dict[str, dict[str, object]]:
    train = roles["train"]
    train_features = train.query_features.numpy()
    grouped_train: dict[tuple[int, ...], NDArray[np.int64]] = {}
    for key in sorted(set(train.partition_keys)):
        grouped_train[key] = np.asarray(
            [
                index
                for index, candidate in enumerate(train.partition_keys)
                if candidate == key
            ],
            dtype=np.int64,
        )
    nearest_digest = hashlib.sha256()
    local_digest = hashlib.sha256()
    ridge_digest = hashlib.sha256()
    for role in ROLE_NAMES:
        query = roles[role]
        query_features = query.query_features.numpy()
        nearest_indices = np.empty(query_features.shape[0], dtype=np.int64)
        local_normalizers = np.empty(query_features.shape[0], dtype=np.float64)
        for row, key in enumerate(query.partition_keys):
            candidates = grouped_train.get(key)
            if candidates is None or candidates.size == 0:
                raise C0StructuralCostError(
                    "non-neural control partition is unsupported"
                )
            difference = train_features[candidates, :10] - query_features[row, :10]
            distance = np.sum(difference * difference, axis=1)
            minimum = int(np.argmin(distance))
            nearest_indices[row] = int(candidates[minimum])
            bandwidth = max(
                float(np.median(distance[distance > 0.0]))
                if np.any(distance > 0.0)
                else 1.0,
                1.0e-12,
            )
            local_normalizers[row] = math.fsum(
                float(value) for value in np.exp(-0.5 * distance / bandwidth)
            )
        nearest_digest.update(nearest_indices.astype("<i8", copy=False).tobytes())
        local_digest.update(local_normalizers.astype("<f8", copy=False).tobytes())
        integral_features = torch.cat(
            (
                query.query_features,
                torch.sum(
                    query.query_kernel_weights[:, :, None]
                    * query.node_features[query.row_field_indices, :, :8],
                    dim=1,
                ),
            ),
            dim=1,
        ).numpy()
        gram = integral_features.T @ integral_features
        gram.flat[:: gram.shape[0] + 1] += 1.0e-6
        artificial_rhs = integral_features.T @ np.zeros(
            (integral_features.shape[0], 3), dtype=np.float64
        )
        coefficients = np.linalg.solve(gram, artificial_rhs)
        ridge_digest.update(coefficients.astype("<f8", copy=False).tobytes())
    return {
        "continuous-local-interpolation-v1": {
            "artificial_path_sha256": local_digest.hexdigest(),
            "roles_executed": 3,
        },
        "fixed-rbf-integral-ridge-v1": {
            "artificial_path_sha256": ridge_digest.hexdigest(),
            "feature_width": 32,
            "roles_executed": 3,
        },
        "nearest-causal-surface-query-v1": {
            "artificial_path_sha256": nearest_digest.hexdigest(),
            "roles_executed": 3,
        },
    }


def ablation_reachability(train: TensorRole) -> dict[str, str]:
    model = QuerySurfaceCostModel(370201)
    rows = torch.arange(0, min(64, train.query_features.shape[0]), dtype=torch.int64)
    with torch.no_grad():
        encoded = model.encode_fields(train, topology=True)
        baseline = model.predict_from_encoded(
            encoded, train, rows, "query-conditioned-surface-operator-v0"
        )[:, 2]
        variants = {
            "field_branch": "qso-query-only-ablation-v1",
            "field_query_interaction": "qso-no-interaction-ablation-v1",
            "query_trunk": "qso-field-only-ablation-v1",
        }
        result: dict[str, str] = {}
        for label, variant in variants.items():
            output = model.predict_from_encoded(encoded, train, rows, variant)[:, 2]
            if torch.equal(output, baseline):
                raise C0StructuralCostError(f"QSO path is unreachable: {label}")
            result[label] = "Reachable"
        no_topology = model.encode_fields(train, topology=False)
        topology_output = model.predict_from_encoded(
            no_topology, train, rows, "qso-no-topology-ablation-v1"
        )[:, 2]
        if torch.equal(topology_output, baseline):
            raise C0StructuralCostError("QSO topology path is unreachable")
        result["topology_propagation"] = "Reachable"
    return result


def run_cost_oracle(
    context: LoadedContext, roles: dict[str, BuiltRole]
) -> dict[str, object]:
    torch.use_deterministic_algorithms(True)
    if torch.get_num_threads() != 1:
        torch.set_num_threads(1)
    if torch.get_num_interop_threads() != 1:
        torch.set_num_interop_threads(1)
    tensor_roles = {role: tensor_role(built) for role, built in roles.items()}
    # C0 transfers ownership to compact tensors after structural evidence exists.
    # Keeping the immutable construction graph would double-retain IDs and arrays
    # that are deliberately outside the cost path.
    roles.clear()
    gc.collect()
    cost = cast(dict[str, Any], context.profile["cost_oracle"])
    steps = cast(int, cost["neural_cost_steps"])
    batch_rows = cast(int, cost["batch_rows"])
    microbatch_rows = cast(int, cost["gradient_microbatch_rows"])
    if batch_rows % microbatch_rows != 0:
        raise C0StructuralCostError("gradient microbatch does not divide logical batch")
    qso_variants = (
        ("query-conditioned-surface-operator-v0", 370201),
        ("qso-field-only-ablation-v1", 370204),
        ("qso-no-interaction-ablation-v1", 370206),
        ("qso-no-topology-ablation-v1", 370205),
        ("qso-query-only-ablation-v1", 370203),
    )
    workloads: dict[str, object] = {}
    for variant, seed in qso_variants:
        workloads[variant] = train_qso_cost(
            variant, seed, tensor_roles, steps, batch_rows, microbatch_rows
        )
        gc.collect()
    workloads["v36-shaped-pointwise-mlp-v1"] = train_pointwise_cost(
        370202, tensor_roles, steps, batch_rows, microbatch_rows
    )
    gc.collect()
    workloads.update(non_neural_cost(tensor_roles))
    order = cast(list[str], cost["candidate_and_control_order"])
    if tuple(workloads) != tuple(order):
        workloads = {name: workloads[name] for name in order}
    combined = 5 * 12_443 + 1_859
    if (
        combined != cost["expected_combined_neural_parameters"]
        or combined > cost["max_combined_trained_parameters"]
    ):
        raise C0StructuralCostError("combined neural parameter ceiling drift")
    cache_peak = max(
        cast(int, cast(dict[str, object], value).get("field_cache_peak_bytes", 0))
        for value in workloads.values()
    )
    if cache_peak > cost["field_cache_max_bytes"]:
        raise C0StructuralCostError("combined field cache ceiling exceeded")
    return {
        "ablation_reachability": ablation_reachability(tensor_roles["train"]),
        "artificial_zero_rows_per_step": batch_rows,
        "combined_neural_parameters": combined,
        "field_cache_peak_bytes": cache_peak,
        "gradient_microbatch_rows": microbatch_rows,
        "neural_cost_steps_each": steps,
        "scientific_target_rows": 0,
        "workloads": workloads,
    }


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def structural_census(
    context: LoadedContext, roles: dict[str, BuiltRole]
) -> dict[str, object]:
    role_rows: dict[str, object] = {}
    total_cases = total_fields = total_rows = total_probes = total_edges = 0
    row_sets: dict[str, set[str]] = {}
    for role in ROLE_NAMES:
        built = roles[role]
        batch = built.batch
        fields = batch.fields
        row_sets[role] = set(batch.row_ids)
        role_rows[role] = {
            "case_count": len(built.case_ids),
            "case_root_sha256": line_root(built.case_ids),
            "directed_edge_count": fields.directed_edge_count,
            "exact_two_hop_pairs": built.exact_two_hop_pairs,
            "field_count": fields.field_count,
            "field_identity_root_sha256": f0.merkle_root(set(fields.field_ids)),
            "field_structural_root_sha256": fields.root_sha256,
            "modal_row_count": batch.row_count,
            "modal_row_identity_root_sha256": f0.merkle_root(set(batch.row_ids)),
            "permutation_root_sha256": built.permutation_root_sha256,
            "probe_count": fields.probe_count,
            "query_support_count": built.query_support_count,
            "query_structural_root_sha256": batch.structural_root_sha256,
            "remesh_root_sha256": built.remesh_root_sha256,
            "targets_present": batch.has_targets,
        }
        total_cases += len(built.case_ids)
        total_fields += fields.field_count
        total_rows += batch.row_count
        total_probes += fields.probe_count
        total_edges += fields.directed_edge_count
    intersections = {
        "development_method_holdout": len(
            row_sets["development"] & row_sets["method_holdout"]
        ),
        "train_development": len(row_sets["train"] & row_sets["development"]),
        "train_method_holdout": len(row_sets["train"] & row_sets["method_holdout"]),
    }
    totals = {
        "cases": total_cases,
        "directed_edges": total_edges,
        "fields": total_fields,
        "modal_rows": total_rows,
        "probes": total_probes,
    }
    structural = cast(dict[str, Any], context.profile["structural"])
    if totals != structural["expected_totals"] or any(intersections.values()):
        raise C0StructuralCostError("full structural total or role separation drift")
    d0_trace = structural_trace(roles, contract.ProviderKind.STRUCTURAL_D0)
    h0_trace = structural_trace(roles, contract.ProviderKind.STRUCTURAL_H0)
    return {
        "claim": CLAIM,
        "forbidden_access": ZERO_FORBIDDEN_ACCESS,
        "role_row_intersections": intersections,
        "roles": role_rows,
        "schema": "nextengine.experimental-physical-sound-v37-c0-structural-census.v1",
        "structural_traces": {
            "d0": {
                "provider_calls": d0_trace.access.provider_calls,
                "structural_rows": d0_trace.access.structural_rows,
                "terminal": d0_trace.terminal.value,
                "topology_sha256": d0_trace.topology_sha256,
            },
            "h0": {
                "provider_calls": h0_trace.access.provider_calls,
                "structural_rows": h0_trace.access.structural_rows,
                "terminal": h0_trace.terminal.value,
                "topology_sha256": h0_trace.topology_sha256,
            },
        },
        "totals": totals,
    }


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise C0StructuralCostError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise C0StructuralCostError("output must be a fresh external path")
    return output


def artifact_reference(name: str, data: bytes) -> dict[str, object]:
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, object]:
    started = time.monotonic()
    context = load_context(profile_path)
    validate_import_boundary(context.profile)
    output = external_output(output_path)
    roles = {role: build_role(context, role) for role in ROLE_NAMES}
    census = structural_census(context, roles)
    cost_oracle = run_cost_oracle(context, roles)
    elapsed = time.monotonic() - started
    rss = peak_rss_bytes()
    cost = cast(dict[str, Any], context.profile["cost_oracle"])
    resource_gates = {
        "field_cache_within_frozen_limit": cost_oracle["field_cache_peak_bytes"]
        <= cost["field_cache_max_bytes"],
        "peak_rss_within_frozen_limit": rss <= cost["max_peak_rss_bytes"],
        "wall_within_frozen_limit": elapsed <= cost["max_wall_seconds"],
    }
    if not all(resource_gates.values()):
        raise C0StructuralCostError("C0 observed resource envelope exceeded")
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v37-c0-", dir=output.parent))
    try:
        census_data = canonical_json(census)
        cost_data = canonical_json(
            {
                "claim": CLAIM,
                "cost_oracle": cost_oracle,
                "forbidden_access": ZERO_FORBIDDEN_ACCESS,
                "schema": "nextengine.experimental-physical-sound-v37-c0-cost-oracle.v1",
                "status": "Pass",
            }
        )
        owner_data = (repository_root() / OWNER_PATH).read_bytes()
        evidence = {
            "artifacts": {
                "cost-oracle.json": artifact_reference("cost-oracle.json", cost_data),
                "structural-census.json": artifact_reference(
                    "structural-census.json", census_data
                ),
            },
            "claim": CLAIM,
            "dependencies": context.dependencies,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "import_boundary": validate_import_boundary(context.profile),
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "profile_boundary": validate_profile_boundary(context.profile),
            "profile_identity": {
                "bytes": len(context.profile_data),
                "path": PROFILE_PATH,
                "sha256": PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v37-c0-evidence.v1",
        }
        evidence_data = canonical_json(evidence)
        report = {
            "artifacts": {
                "cost_oracle": artifact_reference("cost-oracle.json", cost_data),
                "evidence": artifact_reference("evidence.json", evidence_data),
                "structural_census": artifact_reference(
                    "structural-census.json", census_data
                ),
            },
            "claim": CLAIM,
            "decision": "C0_QUERY_SURFACE_STRUCTURE_AND_COST_PASS",
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "next_authorized_stage": "V37-X0-complete-owner-and-terminal-conformance",
            "official_values_opened": False,
            "profile_sha256": PROFILE_SHA256,
            "schema": "nextengine.experimental-physical-sound-v37-c0-report.v1",
            "status": "Pass",
        }
        report_data = canonical_json(report)
        payloads = {
            "cost-oracle.json": cost_data,
            "evidence.json": evidence_data,
            "report.json": report_data,
            "structural-census.json": census_data,
        }
        total_bytes = sum(len(data) for data in payloads.values())
        if total_bytes > cost["max_output_bytes"]:
            raise C0StructuralCostError("C0 output resource envelope exceeded")
        for name, data in payloads.items():
            (staging / name).write_bytes(data)
        staging.replace(output)
        return report
    except BaseException:
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
        C0StructuralCostError,
        contract.ContractError,
        f0.F0FreezeError,
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
                "official_values_opened": report["official_values_opened"],
                "profile_sha256": PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
