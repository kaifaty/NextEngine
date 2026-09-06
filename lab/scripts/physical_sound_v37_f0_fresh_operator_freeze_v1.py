#!/usr/bin/env python3
"""Freeze fresh QSO identities and science without evaluating target values."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import cast

import physical_sound_v37_query_surface_contract_v1 as contract

PROFILE_PATH = "lab/profiles/physical-sound-v37-f0-fresh-query-surface-operator.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v37_f0_fresh_operator_freeze_v1.py"
CONTRACT_PATH = "lab/scripts/physical_sound_v37_query_surface_contract_v1.py"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-f0-fresh-query-surface-"
    "operator-profile.v1"
)
PROFILE_ID = "physical-sound-v37-f0-fresh-query-surface-operator-v1"
PROFILE_SHA256 = "409d84b0dff6b28c0df8976afda198fab5c854b1a1e64788096babe4b486d809"
BASELINE_COMMIT = "a4a2f5622ed492ffb32d5bfc663f52797f643663"
CLAIM = (
    "TARGET_FREE_FRESH_QUERY_SURFACE_OPERATOR_IDENTITY_SCIENCE_AND_ROLE_FREEZE_"
    "ONLY / NO_TARGET_EVALUATION_MODEL_EXECUTION_OFFICIAL_CAPABILITY_QUALITY_"
    "REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
PRIOR_GENERATION_PATHS = (
    "lab/profiles/physical-sound-v32-m0-physics-locked-residual.v1.json",
    "lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json",
    "lab/profiles/physical-sound-v34-f0-target-safe-spectral-recovery.v1.json",
    "lab/profiles/physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json",
    "lab/profiles/physical-sound-v36-f0-fresh-role-unchanged-science.v1.json",
)
V35_PATH = PRIOR_GENERATION_PATHS[3]
V36_PATH = PRIOR_GENERATION_PATHS[4]
ROLE_NAMES = ("train", "development", "method_holdout")
EXPECTED_CASE_COUNTS = {"development": 432, "method_holdout": 432, "train": 648}
EXPECTED_STRATA = {
    "development": {"contact-only": 108, "geometry-only": 216, "joint": 108},
    "method_holdout": {
        "contact-only": 108,
        "geometry-only": 216,
        "joint": 108,
    },
    "train": {"train": 648},
}
EXPECTED_BASELINES = (
    "continuous-local-interpolation-v1",
    "fixed-rbf-integral-ridge-v1",
    "nearest-causal-surface-query-v1",
    "qso-field-only-ablation-v1",
    "qso-no-interaction-ablation-v1",
    "qso-no-topology-ablation-v1",
    "qso-query-only-ablation-v1",
    "v36-shaped-pointwise-mlp-v1",
)
EXPECTED_DIFF_PATHS = (
    "baselines",
    "candidate.family",
    "candidate.field_query_interaction",
    "candidate.representation",
    "candidate.topology_layers",
    "truth.contact_components",
)
ZERO_ACCESS = {
    "development_target_rows": 0,
    "feature_rows_materialized": 0,
    "fresh_v37_target_rows": 0,
    "method_holdout_target_rows": 0,
    "model_parameters_initialized": 0,
    "network_requests": 0,
    "official_capabilities_constructed": 0,
    "prior_generation_metric_values_read": 0,
    "prior_generation_prediction_values_read": 0,
    "prior_generation_target_values_read": 0,
    "prior_generation_weight_values_read": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "target_arrays_constructed": 0,
    "train_target_rows": 0,
    "truth_formulas_evaluated": 0,
}
EXPECTED_AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "c0_allowed_after_f0": True,
    "external_research_only": True,
    "model_execution_allowed": False,
    "network_allowed": False,
    "official_capability_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "runtime_authority": False,
    "signal_decode_allowed": False,
    "synthetic_only": True,
    "target_evaluation_allowed": False,
    "validator_release_authority": False,
}
EXPECTED_ACCESS_ORDER = [
    "a0-query-surface-contract",
    "f0-fresh-operator-truth-and-role-freeze",
    "c0-zero-target-structural-and-cost-census",
    "x0-complete-owner-and-terminal-conformance",
    "e0-full-surrogate-rehearsal-and-execution-seal",
    "d0-one-shot-fresh-development",
    "h0-one-shot-method-holdout",
    "terminal-report",
]


class F0FreezeError(RuntimeError):
    """The value-free V37 F0 freeze contract failed."""


@dataclass(frozen=True, slots=True)
class LoadedDependency:
    path: str
    data: bytes
    document: dict[str, object] | None


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise F0FreezeError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def as_map(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise F0FreezeError(f"{label} must be an object")
    return cast(dict[str, object], value)


def as_list(value: object, label: str) -> list[object]:
    if not isinstance(value, list):
        raise F0FreezeError(f"{label} must be an array")
    return cast(list[object], value)


def map_at(parent: dict[str, object], key: str) -> dict[str, object]:
    if key not in parent:
        raise F0FreezeError(f"required object is absent: {key}")
    return as_map(parent[key], key)


def list_at(parent: dict[str, object], key: str) -> list[object]:
    if key not in parent:
        raise F0FreezeError(f"required array is absent: {key}")
    return as_list(parent[key], key)


def str_at(parent: dict[str, object], key: str) -> str:
    value = parent.get(key)
    if not isinstance(value, str) or not value:
        raise F0FreezeError(f"required string is invalid: {key}")
    return value


def int_at(parent: dict[str, object], key: str) -> int:
    value = parent.get(key)
    if isinstance(value, bool) or not isinstance(value, int):
        raise F0FreezeError(f"required integer is invalid: {key}")
    return value


def string_list(value: object, label: str) -> tuple[str, ...]:
    values = as_list(value, label)
    if not all(isinstance(item, str) and item for item in values):
        raise F0FreezeError(f"{label} must contain nonempty strings")
    return tuple(cast(str, item) for item in values)


def integer_list(value: object, label: str) -> tuple[int, ...]:
    values = as_list(value, label)
    if not all(isinstance(item, int) and not isinstance(item, bool) for item in values):
        raise F0FreezeError(f"{label} must contain integers")
    return tuple(cast(int, item) for item in values)


def load_json_bytes(data: bytes, label: str) -> dict[str, object]:
    try:
        parsed = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise F0FreezeError(f"invalid JSON: {label}") from error
    document = as_map(parsed, label)
    if canonical_json(document) != data:
        raise F0FreezeError(f"JSON is not canonical: {label}")
    return document


def load_profile(path: Path) -> tuple[dict[str, object], bytes]:
    if not path.is_file() or path.is_symlink():
        raise F0FreezeError("profile must be a regular file")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise F0FreezeError("profile size outside bound")
    profile = load_json_bytes(data, "F0 profile")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise F0FreezeError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or map_at(profile, "authority") != EXPECTED_AUTHORITY
        or profile.get("access_order") != EXPECTED_ACCESS_ORDER
    ):
        raise F0FreezeError("profile identity, authority or access order mismatch")
    return profile, data


def load_dependencies(
    profile: dict[str, object],
) -> tuple[dict[str, dict[str, object]], dict[str, dict[str, object]]]:
    declarations = list_at(profile, "dependencies")
    paths: list[str] = []
    records: dict[str, dict[str, object]] = {}
    documents: dict[str, dict[str, object]] = {}
    root = repository_root()
    for raw in declarations:
        declaration = as_map(raw, "dependency")
        path_text = str_at(declaration, "path")
        expected_hash = str_at(declaration, "sha256")
        paths.append(path_text)
        path = root / path_text
        if not path.is_file() or path.is_symlink():
            raise F0FreezeError(f"dependency is absent or linked: {path_text}")
        data = path.read_bytes()
        if sha256_bytes(data) != expected_hash:
            raise F0FreezeError(f"dependency drift: {path_text}")
        records[path_text] = {
            "bytes": len(data),
            "path": path_text,
            "sha256": expected_hash,
        }
        if path.suffix == ".json":
            documents[path_text] = load_json_bytes(data, path_text)
    if tuple(paths) != tuple(sorted(set(paths))):
        raise F0FreezeError("dependency paths must be canonical and unique")
    if not set(PRIOR_GENERATION_PATHS).issubset(documents):
        raise F0FreezeError("prior generation identity closure is incomplete")
    return records, documents


def values_for_key(value: object, wanted: str) -> list[object]:
    results: list[object] = []
    if isinstance(value, dict):
        mapping = cast(dict[str, object], value)
        for key, child in mapping.items():
            if key == wanted:
                results.append(child)
            results.extend(values_for_key(child, wanted))
    elif isinstance(value, list):
        for child in cast(list[object], value):
            results.extend(values_for_key(child, wanted))
    return results


def all_object_keys(value: object) -> set[str]:
    result: set[str] = set()
    if isinstance(value, dict):
        for key, child in cast(dict[str, object], value).items():
            result.add(key)
            result.update(all_object_keys(child))
    elif isinstance(value, list):
        for child in cast(list[object], value):
            result.update(all_object_keys(child))
    return result


def collect_integer_values(value: object) -> list[int]:
    if isinstance(value, bool):
        return []
    if isinstance(value, int):
        return [value]
    if isinstance(value, dict):
        result: list[int] = []
        for child in cast(dict[str, object], value).values():
            result.extend(collect_integer_values(child))
        return result
    if isinstance(value, list):
        result = []
        for child in cast(list[object], value):
            result.extend(collect_integer_values(child))
        return result
    return []


def collect_seed_values(value: object) -> list[int]:
    result: list[int] = []
    if isinstance(value, dict):
        for key, child in cast(dict[str, object], value).items():
            if "seed" in key:
                result.extend(collect_integer_values(child))
            else:
                result.extend(collect_seed_values(child))
    elif isinstance(value, list):
        for child in cast(list[object], value):
            result.extend(collect_seed_values(child))
    return result


def coordinate_pairs(value: object) -> set[tuple[str, str]]:
    result: set[tuple[str, str]] = set()
    if isinstance(value, list):
        values = cast(list[object], value)
        if len(values) == 2 and all(isinstance(item, str) for item in values):
            left, right = (cast(str, item) for item in values)
            try:
                numbers = (float(left), float(right))
            except ValueError:
                numbers = (math.nan, math.nan)
            if all(
                math.isfinite(number) and 0.0 <= number <= 1.0 for number in numbers
            ):
                result.add((left, right))
        for child in values:
            result.update(coordinate_pairs(child))
    elif isinstance(value, dict):
        for child in cast(dict[str, object], value).values():
            result.update(coordinate_pairs(child))
    return result


def geometry_triples(value: object) -> set[tuple[str, str, str]]:
    result: set[tuple[str, str, str]] = set()
    for raw in as_list(value, "geometry cells"):
        row = as_list(raw, "geometry cell")
        if len(row) != 3 or not all(isinstance(item, str) for item in row):
            raise F0FreezeError("geometry cell must contain three decimal strings")
        result.add((cast(str, row[0]), cast(str, row[1]), cast(str, row[2])))
    return result


def material_identities(value: object) -> set[bytes]:
    result: set[bytes] = set()
    for raw in as_list(value, "materials"):
        result.add(canonical_json(as_map(raw, "material")))
    return result


def material_physics_signatures(value: object) -> set[tuple[str, str, str, str]]:
    result: set[tuple[str, str, str, str]] = set()
    for raw in as_list(value, "materials"):
        material = as_map(raw, "material")
        result.add(
            (
                str_at(material, "density_kg_m3"),
                str_at(material, "loss_rate_per_second"),
                str_at(material, "poisson_ratio"),
                str_at(material, "youngs_modulus_pa"),
            )
        )
    return result


def string_values_for_keys(value: object, keys: set[str]) -> set[str]:
    result: set[str] = set()
    if isinstance(value, dict):
        for key, child in cast(dict[str, object], value).items():
            if key in keys and isinstance(child, str):
                result.add(child)
            result.update(string_values_for_keys(child, keys))
    elif isinstance(value, list):
        for child in cast(list[object], value):
            result.update(string_values_for_keys(child, keys))
    return result


def prior_identity_sets(
    documents: dict[str, dict[str, object]],
) -> dict[str, object]:
    contacts: set[tuple[str, str]] = set()
    geometries: set[tuple[str, str, str]] = set()
    materials: set[bytes] = set()
    material_physics: set[tuple[str, str, str, str]] = set()
    formula_ids: set[str] = set()
    prefixes: set[str] = set()
    seeds: set[int] = set()
    for path in PRIOR_GENERATION_PATHS:
        document = documents[path]
        for value in values_for_key(document, "contacts"):
            contacts.update(coordinate_pairs(value))
        for value in values_for_key(document, "geometry_multiplier_cells"):
            geometries.update(geometry_triples(value))
        for value in values_for_key(document, "materials"):
            materials.update(material_identities(value))
            material_physics.update(material_physics_signatures(value))
        formula_ids.update(
            string_values_for_keys(
                document,
                {
                    "expression",
                    "formula_id",
                    "function_id",
                    "mixture_id",
                    "surface_function_family",
                },
            )
        )
        prefixes.update(string_values_for_keys(document, {"role_identity_prefix"}))
        seeds.update(collect_seed_values(document))
    return {
        "contacts": contacts,
        "formula_ids": formula_ids,
        "geometries": geometries,
        "materials": materials,
        "material_physics": material_physics,
        "prefixes": prefixes,
        "seeds": seeds,
    }


def corpus_section(profile: dict[str, object]) -> dict[str, object]:
    return map_at(profile, "corpus")


def role_string_records(
    section: dict[str, object], key: str, identity_key: str
) -> dict[str, tuple[str, ...]]:
    grouped = map_at(section, key)
    result: dict[str, tuple[str, ...]] = {}
    for role in ROLE_NAMES:
        identities = tuple(
            str_at(as_map(raw, f"{key} row"), identity_key)
            for raw in list_at(grouped, role)
        )
        if not identities or len(identities) != len(set(identities)):
            raise F0FreezeError(f"{key} identities are empty or duplicated: {role}")
        result[role] = identities
    return result


def current_contact_sets(
    corpus: dict[str, object],
) -> dict[str, tuple[tuple[str, str], ...]]:
    grouped = map_at(corpus, "contacts")
    result: dict[str, tuple[tuple[str, str], ...]] = {}
    for role in ROLE_NAMES:
        rows: list[tuple[str, str]] = []
        for raw in list_at(grouped, role):
            row = as_list(raw, "contact")
            if len(row) != 2 or not all(isinstance(item, str) for item in row):
                raise F0FreezeError("contact must contain two decimal strings")
            pair = (cast(str, row[0]), cast(str, row[1]))
            numbers = tuple(float(value) for value in pair)
            if not all(
                math.isfinite(number) and 0.0 <= number <= 1.0 for number in numbers
            ):
                raise F0FreezeError("contact lies outside the finite unit square")
            rows.append(pair)
        if (
            len(rows) != len(set(rows))
            or len([row for row in rows if row[0] == "0"]) != 1
        ):
            raise F0FreezeError(f"contact role is duplicated or lacks one node: {role}")
        result[role] = tuple(rows)
    if {role: len(rows) for role, rows in result.items()} != {
        "development": 6,
        "method_holdout": 6,
        "train": 12,
    }:
        raise F0FreezeError("contact role counts drift")
    flattened = [row for rows in result.values() for row in rows]
    if len(flattened) != len(set(flattened)):
        raise F0FreezeError("fresh contact roles intersect")
    return result


def truth_formula_ids(profile: dict[str, object]) -> tuple[str, ...]:
    truth = map_at(map_at(profile, "science"), "truth")
    contact = map_at(truth, "contact")
    component_ids = tuple(
        str_at(as_map(raw, "contact component"), "formula_id")
        for raw in list_at(contact, "components")
    )
    return (
        *component_ids,
        str_at(map_at(truth, "decay"), "formula_id"),
        str_at(map_at(truth, "global_gain"), "formula_id"),
        str_at(truth, "surface_function_family"),
    )


def validate_fresh_identities(
    profile: dict[str, object], documents: dict[str, dict[str, object]]
) -> dict[str, object]:
    corpus = corpus_section(profile)
    contacts_by_role = current_contact_sets(corpus)
    current_contacts = {row for rows in contacts_by_role.values() for row in rows}
    geometries = geometry_triples(corpus["geometry_cells"])
    if len(geometries) != 18 or any(
        not all(math.isfinite(float(value)) and float(value) > 0.0 for value in row)
        for row in geometries
    ):
        raise F0FreezeError("fresh geometry identities are invalid")
    materials = material_identities(corpus["materials"])
    material_physics = material_physics_signatures(corpus["materials"])
    if len(materials) != 3 or len(material_physics) != 3:
        raise F0FreezeError("fresh material identities are invalid")
    for raw in list_at(corpus, "materials"):
        material = as_map(raw, "material")
        if not str_at(material, "material_id").startswith("synthetic-qso-"):
            raise F0FreezeError(
                "material identity is outside the synthetic QSO namespace"
            )
        for key in (
            "density_kg_m3",
            "loss_rate_per_second",
            "poisson_ratio",
            "youngs_modulus_pa",
        ):
            value = float(str_at(material, key))
            if not math.isfinite(value) or value <= 0.0:
                raise F0FreezeError("material scalar is non-positive or non-finite")

    surface_ids = role_string_records(corpus, "surface_functions", "function_id")
    mixture_ids = role_string_records(corpus, "operator_mixtures", "mixture_id")
    if {role: len(values) for role, values in surface_ids.items()} != {
        "development": 4,
        "method_holdout": 4,
        "train": 6,
    }:
        raise F0FreezeError("surface function counts drift")
    if {role: len(values) for role, values in mixture_ids.items()} != {
        "development": 2,
        "method_holdout": 2,
        "train": 4,
    }:
        raise F0FreezeError("operator mixture counts drift")
    all_surface_ids = {item for values in surface_ids.values() for item in values}
    all_mixture_ids = {item for values in mixture_ids.values() for item in values}
    if sum(len(values) for values in surface_ids.values()) != len(all_surface_ids):
        raise F0FreezeError("surface function roles intersect")
    if sum(len(values) for values in mixture_ids.values()) != len(all_mixture_ids):
        raise F0FreezeError("operator mixture roles intersect")
    mixtures = map_at(corpus, "operator_mixtures")
    for role in ROLE_NAMES:
        for raw in list_at(mixtures, role):
            mixture = as_map(raw, "operator mixture")
            weights = tuple(
                float(value)
                for value in string_list(
                    mixture["component_weights"], "operator component weights"
                )
            )
            if (
                len(weights) != 3
                or any(weight <= 0.0 for weight in weights)
                or not math.isclose(sum(weights), 1.0, rel_tol=0.0, abs_tol=1.0e-12)
            ):
                raise F0FreezeError(
                    "operator mixture must have three positive unit-sum weights"
                )

    formula_ids = set(truth_formula_ids(profile))
    if len(formula_ids) != 6:
        raise F0FreezeError("truth formula identities are duplicated")
    seed_values = collect_seed_values(profile)
    if len(seed_values) != 32 or len(seed_values) != len(set(seed_values)):
        raise F0FreezeError("V37 seeds must be 32 unique integers")
    prefix = str_at(corpus, "role_identity_prefix")
    if not prefix.startswith("physical-sound-v37-"):
        raise F0FreezeError("role identity prefix is outside V37")

    prior = prior_identity_sets(documents)
    intersections = {
        "contact": len(
            current_contacts & cast(set[tuple[str, str]], prior["contacts"])
        ),
        "formula": len(formula_ids & cast(set[str], prior["formula_ids"])),
        "geometry": len(
            geometries & cast(set[tuple[str, str, str]], prior["geometries"])
        ),
        "material_identity": len(materials & cast(set[bytes], prior["materials"])),
        "material_physics": len(
            material_physics
            & cast(set[tuple[str, str, str, str]], prior["material_physics"])
        ),
        "prefix": int(prefix in cast(set[str], prior["prefixes"])),
        "seed": len(set(seed_values) & cast(set[int], prior["seeds"])),
        "surface_function": len(all_surface_ids & cast(set[str], prior["formula_ids"])),
        "operator_mixture": len(all_mixture_ids & cast(set[str], prior["formula_ids"])),
    }
    if any(intersections.values()):
        raise F0FreezeError(f"V37 identity intersects V32-V36: {intersections}")
    return {
        "contact_count_by_role": {
            role: len(values) for role, values in sorted(contacts_by_role.items())
        },
        "fresh_geometry_cells": len(geometries),
        "fresh_materials": len(materials),
        "fresh_operator_mixtures": len(all_mixture_ids),
        "fresh_seeds": len(seed_values),
        "fresh_surface_functions": len(all_surface_ids),
        "fresh_truth_formula_ids": len(formula_ids),
        "intersections_with_v32_through_v36": intersections,
        "role_identity_prefix": prefix,
    }


def linear_parameter_count(input_width: int, output_width: int) -> int:
    return input_width * output_width + output_width


def validate_science(profile: dict[str, object]) -> dict[str, object]:
    forbidden_payload_keys = {
        "candidate_weights",
        "oracle_values",
        "predictions",
        "target_values",
        "targets",
    }
    if all_object_keys(profile) & forbidden_payload_keys:
        raise F0FreezeError("F0 profile contains a value-bearing payload key")
    science = map_at(profile, "science")
    baselines = string_list(science["baselines"], "baselines")
    if baselines != EXPECTED_BASELINES:
        raise F0FreezeError("frozen baseline set drift")
    controls = map_at(science, "control_contracts")
    if tuple(sorted(controls)) != EXPECTED_BASELINES:
        raise F0FreezeError("control contract closure drift")
    context_names = string_list(science["context_feature_names"], "context features")
    if tuple(sorted(set(context_names))) != context_names or any(
        name in contract.FORBIDDEN_CONTEXT_FEATURES for name in context_names
    ):
        raise F0FreezeError("context feature names are noncanonical or forbidden")
    candidate = map_at(science, "candidate")
    latent = int_at(candidate, "latent_width")
    graph_layers = int_at(candidate, "graph_layers")
    node_input = int_at(candidate, "node_input_width")
    query_input = int_at(candidate, "query_input_width")
    head_input = int_at(candidate, "head_input_width")
    if (
        len(context_names) != 14
        or node_input != 8 + len(context_names)
        or query_input != 10 + len(context_names)
        or head_input != 3 * latent + 1
        or graph_layers != 2
        or latent != 24
    ):
        raise F0FreezeError("QSO width or graph-layer algebra drift")
    node_encoder = linear_parameter_count(node_input, latent) + linear_parameter_count(
        latent, latent
    )
    message = linear_parameter_count(2 * latent + 4, latent) + linear_parameter_count(
        latent, latent
    )
    update = linear_parameter_count(2 * latent, latent) + linear_parameter_count(
        latent, latent
    )
    query_encoder = linear_parameter_count(
        query_input, latent
    ) + linear_parameter_count(latent, latent)
    head = linear_parameter_count(head_input, latent) + linear_parameter_count(
        latent, 1
    )
    contact_parameter_count = (
        node_encoder + graph_layers * (message + update) + query_encoder + head
    )
    unchanged_head_parameters = 481 + 513
    parameter_count = contact_parameter_count + unchanged_head_parameters
    if (
        contact_parameter_count != 11449
        or int_at(candidate, "contact_parameter_count") != contact_parameter_count
        or int_at(candidate, "unchanged_decay_global_head_parameter_count")
        != unchanged_head_parameters
        or parameter_count != 12443
        or int_at(candidate, "parameter_count") != parameter_count
        or parameter_count > int_at(candidate, "parameter_count_max")
    ):
        raise F0FreezeError("QSO parameter-count oracle drift")
    training = map_at(candidate, "training")
    if int_at(training, "steps") != 96 or int_at(training, "batch_rows") != 1024:
        raise F0FreezeError("QSO training shape drift")
    for name in EXPECTED_BASELINES:
        control_spec = as_map(controls[name], f"control {name}")
        if (
            name.startswith("qso-") or name == "v36-shaped-pointwise-mlp-v1"
        ) and int_at(control_spec, "steps") != 96:
            raise F0FreezeError("neural control training steps drift")
    metrics = map_at(science, "metrics")
    if string_list(metrics["primary_strata"], "primary strata") != (
        "contact-only",
        "geometry-only",
        "joint",
    ):
        raise F0FreezeError("primary transfer strata drift")
    truth_ids = truth_formula_ids(profile)
    if truth_ids[:3] != (
        "qso-v37-local-anisotropic-integral-v1",
        "qso-v37-two-hop-canonical-graph-diffusion-v1",
        "qso-v37-global-low-rank-field-query-interaction-v1",
    ):
        raise F0FreezeError("three-component contact truth drift")
    resources = map_at(profile, "resources")
    if resources != {
        "field_cache_max_bytes": 33_554_432,
        "max_combined_trained_parameters": 70_000,
        "max_output_bytes": 67_108_864,
        "max_peak_rss_bytes": 1_073_741_824,
        "max_wall_seconds": 300,
        "network_requests": 0,
        "real_signal_values_decoded": 0,
    }:
        raise F0FreezeError("resource envelope drift")

    hashes = tuple(character * 64 for character in "1234")
    passed = contract.candidate_disposition(
        contract.TerminalDecision.PASS,
        owner_sha256=hashes[0],
        profile_sha256=hashes[1],
        terminal_sha256=hashes[2],
        candidate_weights_sha256=hashes[3],
    )
    rejected = contract.candidate_disposition(
        contract.TerminalDecision.METRIC_REJECT,
        owner_sha256=hashes[0],
        profile_sha256=hashes[1],
        terminal_sha256=hashes[2],
        candidate_weights_sha256=hashes[3],
    )
    if (
        not passed.candidate_bundle_authority
        or passed.freeze_document is None
        or rejected.candidate_bundle_authority
        or rejected.freeze_document is not None
        or rejected.rejected_evidence_document is None
    ):
        raise F0FreezeError("A0 candidate disposition binding drift")
    if map_at(profile, "terminal_publication") != {
        "fault": "neither-candidate-document",
        "pass": "candidate-freeze-and-bundle-authority",
        "scientific_reject": "rejected-candidate-evidence-only",
    }:
        raise F0FreezeError("terminal publication policy drift")
    return {
        "candidate_parameter_count": parameter_count,
        "candidate_training_steps": int_at(training, "steps"),
        "contact_truth_components": list(truth_ids[:3]),
        "control_count": len(baselines),
        "graph_layers": graph_layers,
        "latent_width": latent,
        "resource_envelope": resources,
        "target_axis_count": int_at(map_at(science, "truth"), "target_axis_count"),
    }


def scientific_projection(profile: dict[str, object]) -> dict[str, object]:
    science = map_at(profile, "science")
    candidate = map_at(science, "candidate")
    truth = map_at(science, "truth")
    return {
        "baselines": list(string_list(science["baselines"], "baselines")),
        "candidate": {
            "family": "query-conditioned-surface-operator",
            "field_query_interaction": str_at(candidate, "field_query_interaction"),
            "learned_axis": "bounded-contact-correction",
            "representation": "canonical-p1-probe-graph-plus-independent-query",
            "topology_layers": int_at(candidate, "graph_layers"),
        },
        "inherited_branches": map_at(science, "inherited_branches"),
        "truth": {
            "contact_components": list(truth_formula_ids(profile)[:3]),
            "target_axis_count": int_at(truth, "target_axis_count"),
        },
    }


def v36_scientific_projection(
    documents: dict[str, dict[str, object]],
) -> dict[str, object]:
    v35 = documents[V35_PATH]
    v36 = documents[V36_PATH]
    composition = str_at(
        map_at(map_at(v35, "method_overlay"), "candidate"), "composition"
    )
    if "local_weight*local_contact" not in composition:
        raise F0FreezeError("bound V35/V36 scalar hybrid identity drift")
    inheritance = map_at(v36, "scientific_inheritance")
    if inheritance.get("model_seed_policy") != "inherit-v35-exactly":
        raise F0FreezeError("bound V36 scientific inheritance drift")
    return {
        "baselines": [
            "continuous-local-interpolation-v1",
            "nearest-causal-surface-query-v1",
            "v36-shaped-pointwise-mlp-v1",
        ],
        "candidate": {
            "family": "fixed-scalar-local-neural-hybrid",
            "field_query_interaction": "fixed-convex-local-neural-blend",
            "learned_axis": "bounded-contact-correction",
            "representation": "flattened-pointwise-local-stencil",
            "topology_layers": 0,
        },
        "inherited_branches": {
            "decay": "v36-bounded-decay-head-architecture-and-gates-unchanged",
            "global_gain": "v36-bounded-global-gain-head-architecture-and-gates-unchanged",
        },
        "truth": {
            "contact_components": ["single-pointwise-contact-expression-v36"],
            "target_axis_count": 3,
        },
    }


def diff_paths(left: object, right: object, prefix: str = "") -> tuple[str, ...]:
    if isinstance(left, dict) and isinstance(right, dict):
        left_map = cast(dict[str, object], left)
        right_map = cast(dict[str, object], right)
        if set(left_map) != set(right_map):
            return (prefix or "<root>",)
        paths: list[str] = []
        for key in sorted(left_map):
            child_prefix = f"{prefix}.{key}" if prefix else key
            paths.extend(diff_paths(left_map[key], right_map[key], child_prefix))
        return tuple(paths)
    if left != right:
        return (prefix or "<root>",)
    return ()


def build_scientific_diff(
    profile: dict[str, object], documents: dict[str, dict[str, object]]
) -> dict[str, object]:
    previous = v36_scientific_projection(documents)
    current = scientific_projection(profile)
    changed = diff_paths(previous, current)
    declared = string_list(
        map_at(profile, "science")["scientific_diff_from_v36"],
        "scientific diff paths",
    )
    if changed != declared or changed != EXPECTED_DIFF_PATHS:
        raise F0FreezeError(
            f"scientific projection diff is undeclared: {changed} != {declared}"
        )
    return {
        "changed_paths": list(changed),
        "current_projection": current,
        "current_projection_sha256": sha256_bytes(canonical_json(current)),
        "previous_projection": previous,
        "previous_projection_sha256": sha256_bytes(canonical_json(previous)),
        "status": "MATERIAL_QSO_REPRESENTATION_CHANGE_ONLY_AT_DECLARED_PATHS",
        "unchanged_paths": [
            "candidate.learned_axis",
            "inherited_branches",
            "truth.target_axis_count",
        ],
    }


def merkle_root(identities: set[str]) -> str:
    if not identities:
        raise F0FreezeError("identity set is empty")
    return sha256_bytes(("\n".join(sorted(identities)) + "\n").encode())


def role_plan(profile: dict[str, object], role: str) -> list[dict[str, object]]:
    plans = map_at(corpus_section(profile), "role_plan")
    return [as_map(raw, "role plan entry") for raw in list_at(plans, role)]


def build_role_commitments(profile: dict[str, object]) -> dict[str, object]:
    if profile.get("selection_policy") != (
        "surface-and-mixture-index-equals-canonical-case-ordinal-modulo-"
        "declared-set-width"
    ):
        raise F0FreezeError("surface/operator selection policy drift")
    corpus = corpus_section(profile)
    contacts = current_contact_sets(corpus)
    families = [as_map(raw, "family") for raw in list_at(corpus, "families")]
    materials = [as_map(raw, "material") for raw in list_at(corpus, "materials")]
    geometries = [
        tuple(cast(str, item) for item in as_list(raw, "geometry cell"))
        for raw in list_at(corpus, "geometry_cells")
    ]
    surface_ids = role_string_records(corpus, "surface_functions", "function_id")
    mixture_ids = role_string_records(corpus, "operator_mixtures", "mixture_id")
    seeds = map_at(profile, "seeds")
    role_seeds = map_at(seeds, "role_seeds")
    case_seed = int_at(seeds, "case_enumeration_seed")
    truth_ids = truth_formula_ids(profile)
    prefix = str_at(corpus, "role_identity_prefix")
    mode_count = int_at(corpus, "mode_count")
    if len(families) != 3 or len(materials) != 3 or mode_count != 10:
        raise F0FreezeError("role algebra base dimensions drift")

    physical_sets: dict[str, set[str]] = {}
    role_records: dict[str, object] = {}
    total_cases = 0
    total_rows = 0
    for role in ROLE_NAMES:
        physical_roots: set[str] = set()
        case_ids: set[str] = set()
        field_ids: set[str] = set()
        row_ids: set[str] = set()
        strata: dict[str, int] = {}
        used_surfaces: set[str] = set()
        used_mixtures: set[str] = set()
        ordinal = 0
        for entry in role_plan(profile, role):
            stratum = str_at(entry, "stratum")
            geometry_indices = integer_list(entry["geometry_cells"], "geometry indices")
            contact_set = str_at(entry, "contact_set")
            contact_indices = integer_list(entry["contact_indices"], "contact indices")
            surface_set = str_at(entry, "surface_set")
            mixture_set = str_at(entry, "mixture_set")
            if (
                contact_set not in contacts
                or surface_set not in surface_ids
                or mixture_set not in mixture_ids
                or any(
                    index < 0 or index >= len(geometries) for index in geometry_indices
                )
                or any(
                    index < 0 or index >= len(contacts[contact_set])
                    for index in contact_indices
                )
            ):
                raise F0FreezeError("role plan references an unknown identity")
            for family in families:
                for material in materials:
                    for geometry_index in geometry_indices:
                        for contact_index in contact_indices:
                            surface_id = surface_ids[surface_set][
                                ordinal % len(surface_ids[surface_set])
                            ]
                            mixture_id = mixture_ids[mixture_set][
                                ordinal % len(mixture_ids[mixture_set])
                            ]
                            physical = {
                                "contact": contacts[contact_set][contact_index],
                                "family": family,
                                "geometry": geometries[geometry_index],
                                "material": material,
                                "operator_mixture_id": mixture_id,
                                "surface_function_id": surface_id,
                            }
                            physical_root = sha256_bytes(canonical_json(physical))
                            case_payload = {
                                "case_enumeration_seed": case_seed,
                                "physical_root_sha256": physical_root,
                                "prefix": prefix,
                                "profile_sha256": PROFILE_SHA256,
                                "role": role,
                                "role_seed": int_at(role_seeds, role),
                                "stratum": stratum,
                                "truth_formula_ids": truth_ids,
                            }
                            case_id = "case-" + sha256_bytes(
                                canonical_json(case_payload)
                            )
                            physical_roots.add(physical_root)
                            case_ids.add(case_id)
                            used_surfaces.add(surface_id)
                            used_mixtures.add(mixture_id)
                            strata[stratum] = strata.get(stratum, 0) + 1
                            for mode_ordinal in range(mode_count):
                                field_payload = {
                                    "family": family,
                                    "geometry": geometries[geometry_index],
                                    "material": material,
                                    "mode_ordinal": mode_ordinal,
                                    "profile_sha256": PROFILE_SHA256,
                                    "surface_function_id": surface_id,
                                }
                                field_id = "field-" + sha256_bytes(
                                    canonical_json(field_payload)
                                )
                                field_ids.add(field_id)
                                row_ids.add(f"{case_id}:mode-{mode_ordinal:02d}")
                            ordinal += 1
        expected_cases = EXPECTED_CASE_COUNTS[role]
        if (
            len(case_ids) != expected_cases
            or len(physical_roots) != expected_cases
            or len(row_ids) != expected_cases * mode_count
            or strata != EXPECTED_STRATA[role]
        ):
            raise F0FreezeError(f"role commitment algebra drift: {role}")
        if role == "train":
            expected_surfaces = set(surface_ids["train"])
            expected_mixtures = set(mixture_ids["train"])
        else:
            expected_surfaces = set(surface_ids["train"]) | set(surface_ids[role])
            expected_mixtures = set(mixture_ids["train"]) | set(mixture_ids[role])
        if used_surfaces != expected_surfaces or used_mixtures != expected_mixtures:
            raise F0FreezeError(f"surface/operator identity is unreachable: {role}")
        physical_sets[role] = physical_roots
        role_records[role] = {
            "case_count": len(case_ids),
            "case_root_sha256": merkle_root(case_ids),
            "field_count": len(field_ids),
            "field_root_sha256": merkle_root(field_ids),
            "modal_row_count": len(row_ids),
            "modal_row_root_sha256": merkle_root(row_ids),
            "operator_mixture_count": len(used_mixtures),
            "physical_case_root_sha256": merkle_root(physical_roots),
            "role_seed": int_at(role_seeds, role),
            "strata": strata,
            "surface_function_count": len(used_surfaces),
        }
        total_cases += len(case_ids)
        total_rows += len(row_ids)
    intersections = {
        "development_method_holdout": len(
            physical_sets["development"] & physical_sets["method_holdout"]
        ),
        "train_development": len(physical_sets["train"] & physical_sets["development"]),
        "train_method_holdout": len(
            physical_sets["train"] & physical_sets["method_holdout"]
        ),
    }
    if any(intersections.values()) or total_cases != 1512 or total_rows != 15120:
        raise F0FreezeError("complete role identity closure drift")
    return {
        "case_enumeration_seed": case_seed,
        "pairwise_physical_case_intersections": intersections,
        "role_identity_prefix": prefix,
        "roles": role_records,
        "total_cases": total_cases,
        "total_modal_rows": total_rows,
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


def artifact_reference(path: Path) -> dict[str, object]:
    data = path.read_bytes()
    return {"bytes": len(data), "path": path.name, "sha256": sha256_bytes(data)}


def build_conformance(
    profile: dict[str, object], profile_data: bytes
) -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    dependency_records, documents = load_dependencies(profile)
    freshness = validate_fresh_identities(profile, documents)
    science = validate_science(profile)
    scientific_diff = build_scientific_diff(profile, documents)
    commitments = build_role_commitments(profile)
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    conformance: dict[str, object] = {
        "access": ZERO_ACCESS,
        "claim": CLAIM,
        "dependencies": dependency_records,
        "freshness": freshness,
        "gates": {
            "a0_contract_bound": "Pass",
            "authority_narrow": "Pass",
            "candidate_disposition_bound": "Pass",
            "fresh_against_v32_through_v36": "Pass",
            "model_or_target_values": "0 Exact",
            "role_commitments_complete": "Pass",
            "scientific_diff_declared": "Pass",
            "three_operator_components_frozen": "Pass",
        },
        "official_values_opened": False,
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
            "total_cases": commitments["total_cases"],
            "total_modal_rows": commitments["total_modal_rows"],
        },
        "schema": "nextengine.experimental-physical-sound-v37-f0-conformance.v1",
        "science": science,
        "scientific_diff_sha256": sha256_bytes(canonical_json(scientific_diff)),
        "status": "Pass",
    }
    return conformance, commitments, scientific_diff


def run(profile_path: Path, output_path: Path) -> dict[str, object]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v37-f0-", dir=output.parent))
    try:
        conformance, commitments, scientific_diff = build_conformance(
            profile, profile_data
        )
        payloads = {
            "conformance.json": canonical_json(conformance),
            "identity-commitments.json": canonical_json(commitments),
            "scientific-diff.json": canonical_json(scientific_diff),
        }
        for name, data in payloads.items():
            (staging / name).write_bytes(data)
        evidence: dict[str, object] = {
            "access": ZERO_ACCESS,
            "artifacts": {
                name: artifact_reference(staging / name) for name in sorted(payloads)
            },
            "claim": CLAIM,
            "owner_identity": conformance["owner_identity"],
            "profile_identity": conformance["profile_identity"],
            "schema": "nextengine.experimental-physical-sound-v37-f0-evidence.v1",
        }
        (staging / "evidence.json").write_bytes(canonical_json(evidence))
        report: dict[str, object] = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "decision": "F0_FRESH_QUERY_SURFACE_OPERATOR_FREEZE_PASS",
            "evidence": artifact_reference(staging / "evidence.json"),
            "next_authorized_stage": "V37-C0-zero-target-structural-and-cost-census",
            "official_values_opened": False,
            "profile_sha256": PROFILE_SHA256,
            "schema": "nextengine.experimental-physical-sound-v37-f0-report.v1",
            "status": "Pass",
        }
        (staging / "report.json").write_bytes(canonical_json(report))
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
