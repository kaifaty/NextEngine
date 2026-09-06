#!/usr/bin/env python3
"""Audit source-domain predictability and cross-project material signal for V42 R0."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Callable

import numpy as np

import physical_sound_v41_b0_grouped_baselines_v1 as b0

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v42-r0-profile.v1"
AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v42-r0-audit.v1"
INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-v42-r0-inventory.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v42-r0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v42-r0-report.v1"
POWER_SCHEMA = "nextengine.experimental-physical-sound-v42-r0-power-plan.v1"
HASH_LENGTH = 64
MAX_JSON_BYTES = 16 * 1024 * 1024

DECISIONS = (
    "CorpusSignalInsufficient",
    "DescriptorSignalPlausible",
    "DomainNormalizationRequired",
)

AUTHORITY = {
    "admission_authority": False,
    "candidate_model_training_authority": False,
    "descriptor_or_recipe_freeze_authority": False,
    "diagnostic_audit_authority": True,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_access_authority": False,
}

AUDIT_POLICY = {
    "bootstrap": {
        "confidence_interval_percent": [2.5, 97.5],
        "iterations": 8192,
        "seed": 4201,
        "statistic": "median_paired_relative_improvement",
    },
    "cross_source_parent_policy": "exclude_from_project_statistics_report_separately",
    "leave_one_parent_out_controls": [
        "global",
        "coarse_material",
        "project",
        "project_and_coarse_material",
    ],
    "leave_project_out_controls": ["global", "coarse_material"],
    "parent_weighting": "equal_weight_per_physical_parent",
    "permutation": {
        "iterations": 4096,
        "label_shuffle": "within_coarse_material",
        "seed": 4200,
        "statistic": "nearest_project_centroid_balanced_accuracy",
    },
    "project_identity": "c0_family_id",
    "source_centering": "descriptive_oracle_project_mean_not_deployable",
    "target_scaler": "b0_generator_train_mean_std",
}

DECISION_POLICY = {
    "domain_signal": {
        "maximum_permutation_p_value": 0.01,
        "minimum_accuracy_lift_over_null_median": 0.20,
        "minimum_balanced_accuracy": 0.70,
        "minimum_eligible_parents": 30,
    },
    "material_signal": {
        "minimum_bootstrap_lower_relative_improvement": 0.0,
        "minimum_improved_parent_fraction": 0.60,
        "minimum_median_relative_improvement": 0.05,
    },
    "normalization_recovery": {
        "minimum_improved_parent_fraction": 0.60,
        "minimum_median_relative_improvement": 0.05,
    },
    "precedence": [
        "DescriptorSignalPlausible_if_raw_material_signal_passes",
        "DomainNormalizationRequired_if_domain_and_normalization_recovery_pass",
        "CorpusSignalInsufficient_otherwise",
    ],
}

POWER_POLICY = {
    "alpha_two_sided": 0.05,
    "minimum_independent_projects_per_priority_material_train": 2,
    "practical_relative_improvement": 0.05,
    "target_power": 0.80,
    "z_alpha_two_sided": 1.959963985,
    "z_power": 0.841621234,
}


class AuditError(RuntimeError):
    """Raised when the frozen R0 contract cannot be satisfied."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--baseline-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise AuditError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def rounded(value: float, digits: int = 9) -> float:
    if not math.isfinite(value):
        raise AuditError("derived audit value is non-finite")
    result = round(float(value), digits)
    return 0.0 if result == 0 else result


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise AuditError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise AuditError(f"{context} must be an array")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise AuditError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_JSON_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise AuditError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise AuditError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise AuditError(f"{context} is not valid UTF-8 JSON") from error
    return require_dict(value, context)


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise AuditError(f"{context} binding fields changed")
    expected_bytes = binding["bytes"]
    require_hash(binding["sha256"], f"{context} SHA-256")
    if type(expected_bytes) is not int or expected_bytes <= 0:
        raise AuditError(f"{context} binding byte count is invalid")
    data = read_regular(path, context)
    if len(data) != expected_bytes or sha256_bytes(data) != binding["sha256"]:
        raise AuditError(f"{context} bytes or SHA-256 changed")
    return data


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, "R0 profile")
    profile = parse_json(data, "R0 profile")
    if canonical_json(profile) != data:
        raise AuditError("R0 profile must use canonical JSON")
    return data, profile


def validate_dependencies(profile: dict[str, Any]) -> None:
    bindings = require_list(profile.get("dependency_bindings"), "dependencies")
    paths = []
    for raw in bindings:
        binding = require_dict(raw, "dependency")
        path_text = binding.get("path")
        if not isinstance(path_text, str) or not path_text:
            raise AuditError("dependency path is invalid")
        paths.append(path_text)
        check_binding(
            repository_root() / path_text,
            binding,
            f"dependency {path_text}",
        )
    if paths != sorted(set(paths)):
        raise AuditError("dependency bindings must be path-sorted and unique")


def validate_profile(value: dict[str, Any]) -> dict[str, Any]:
    required = {
        "audit_policy",
        "authority",
        "decision_policy",
        "dependency_bindings",
        "environment",
        "expected_counts",
        "input_bindings",
        "material_taxonomy",
        "power_policy",
        "profile_id",
        "schema",
    }
    if set(value) != required:
        raise AuditError("R0 profile fields changed")
    if value["schema"] != PROFILE_SCHEMA or value["profile_id"] != (
        "physical-sound-v42-r0-domain-information-audit-v1"
    ):
        raise AuditError("unknown R0 profile identity")
    for key, expected in (
        ("audit_policy", AUDIT_POLICY),
        ("authority", AUTHORITY),
        ("decision_policy", DECISION_POLICY),
        ("power_policy", POWER_POLICY),
    ):
        if value[key] != expected:
            raise AuditError(f"R0 {key} changed")
    if value["environment"] != {"numpy_version": np.__version__}:
        raise AuditError("R0 NumPy environment changed")
    if (
        value["material_taxonomy"]
        != b0.validate_profile(
            parse_json(
                read_regular(
                    repository_root()
                    / "lab/profiles/physical-sound-v41-b0-grouped-baselines.v1.json",
                    "tracked B0 profile",
                ),
                "tracked B0 profile",
            )
        )["material_taxonomy"]
    ):
        raise AuditError("R0 material taxonomy differs from B0")
    counts = require_dict(value["expected_counts"], "expected counts")
    if set(counts) != {
        "combined_parents",
        "cross_source_parents",
        "development_parents",
        "development_records",
        "leave_project_out_supported_parents",
        "project_classifier_eligible_parents",
        "project_count",
        "single_source_parents",
        "source_centered_supported_parents",
        "train_parents",
        "train_records",
    } or any(type(item) is not int or item < 0 for item in counts.values()):
        raise AuditError("R0 expected counts changed")
    inputs = require_dict(value["input_bindings"], "input bindings")
    if set(inputs) != {"baseline", "corpus"}:
        raise AuditError("R0 input root bindings changed")
    expected = {
        "baseline": {"metrics", "profile", "report", "representation"},
        "corpus": {
            "corpus_card",
            "development_projection",
            "profile",
            "report",
            "train_projection",
        },
    }
    for root_name, names in expected.items():
        root = require_dict(inputs[root_name], f"{root_name} inputs")
        if set(root) != names:
            raise AuditError(f"R0 {root_name} input bindings changed")
        for name, raw in root.items():
            binding = require_dict(raw, f"{root_name} input {name}")
            if set(binding) != {"bytes", "path", "sha256"}:
                raise AuditError("input binding fields changed")
            path = Path(binding["path"])
            if path.is_absolute() or ".." in path.parts:
                raise AuditError("input binding escapes its external root")
            require_hash(binding["sha256"], f"{root_name} input {name}")
    validate_dependencies(value)
    return value


def load_bound_json(
    root: Path, binding: dict[str, Any], context: str
) -> tuple[bytes, dict[str, Any]]:
    data = check_binding(root / binding["path"], binding, context)
    value = parse_json(data, context)
    if canonical_json(value) != data:
        raise AuditError(f"{context} must remain canonical")
    return data, value


def validate_baseline_inputs(
    root: Path, bindings: dict[str, Any]
) -> tuple[int, dict[str, Any]]:
    total = 0
    values = {}
    for name in sorted(bindings):
        data, value = load_bound_json(root, bindings[name], f"B0 {name}")
        total += len(data)
        values[name] = value
    report = values["report"]
    metrics = values["metrics"]
    representation = values["representation"]
    if report.get("decision") != "B0_GROUPED_BASELINE_SURFACE_FROZEN_M0_AUTHORIZED":
        raise AuditError("B0 decision changed")
    summary = require_dict(metrics.get("summary"), "B0 metric summary")
    if summary.get("best_baseline") != "global_prototype":
        raise AuditError("B0 global prototype is no longer the reference floor")
    floor = require_dict(summary.get("reference_floor"), "B0 reference floor")
    if floor.get("median_parent_rmse") != 1.33759365:
        raise AuditError("B0 reference floor changed")
    policy = require_dict(representation.get("policy"), "B0 representation")
    if policy != b0.REPRESENTATION_POLICY:
        raise AuditError("B0 representation policy changed")
    return total, values


def parent_metadata(rows: list[dict[str, Any]], role: str) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for row in rows:
        parent_id = row.get("physical_parent_id")
        family_id = row.get("family_id")
        if not isinstance(parent_id, str) or not isinstance(family_id, str):
            raise AuditError("C0 parent or family identity is invalid")
        item = result.setdefault(
            parent_id,
            {"family_ids": set(), "record_ids": [], "role": role},
        )
        if item["role"] != role:
            raise AuditError("physical parent crosses generator roles")
        item["family_ids"].add(family_id)
        item["record_ids"].append(row["record_id"])
    return result


def prepare_parents(
    train_rows: list[dict[str, Any]],
    development_rows: list[dict[str, Any]],
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    mean: np.ndarray,
    scale: np.ndarray,
) -> list[dict[str, Any]]:
    metadata = parent_metadata(train_rows, b0.TRAIN_ROLE)
    for parent_id, item in parent_metadata(
        development_rows, b0.DEVELOPMENT_ROLE
    ).items():
        if parent_id in metadata:
            raise AuditError("physical parent crosses train and development")
        metadata[parent_id] = item
    parents = []
    for raw in train + development:
        item = metadata[raw["parent_id"]]
        family_ids = sorted(item["family_ids"])
        parent = dict(raw)
        parent["family_ids"] = family_ids
        parent["project_id"] = family_ids[0] if len(family_ids) == 1 else None
        parent["role"] = item["role"]
        parent["standardized_target"] = b0.standardized(raw, mean, scale)
        parents.append(parent)
    if len({parent["parent_id"] for parent in parents}) != len(parents):
        raise AuditError("combined parent inventory is not unique")
    return sorted(parents, key=lambda item: item["parent_id"])


def prototype(parents: list[dict[str, Any]], key: str) -> np.ndarray:
    if not parents:
        raise AuditError("cannot build a prototype from no parents")
    return b0.masked_mean(
        [parent[key] for parent in parents],
        [parent["target_mask"] for parent in parents],
    )


def error(parent: dict[str, Any], prediction: np.ndarray, key: str) -> float:
    return b0.parent_distance(parent[key], prediction, parent["target_mask"])


def summarize(values: list[float]) -> dict[str, Any]:
    if not values:
        raise AuditError("cannot summarize an empty audit stratum")
    return {
        "maximum": rounded(max(values)),
        "mean": rounded(float(np.mean(values))),
        "median": rounded(float(np.percentile(values, 50, method="linear"))),
        "p90": rounded(float(np.percentile(values, 90, method="linear"))),
        "parent_count": len(values),
    }


def leave_one_parent_out(parents: list[dict[str, Any]]) -> dict[str, Any]:
    names = tuple(AUDIT_POLICY["leave_one_parent_out_controls"])
    errors: dict[str, dict[str, float]] = {name: {} for name in names}
    for query in parents:
        references = [
            item for item in parents if item["parent_id"] != query["parent_id"]
        ]
        selections = {
            "global": references,
            "coarse_material": [
                item
                for item in references
                if item["coarse_material"] == query["coarse_material"]
            ],
            "project": [
                item for item in references if item["project_id"] == query["project_id"]
            ],
            "project_and_coarse_material": [
                item
                for item in references
                if item["project_id"] == query["project_id"]
                and item["coarse_material"] == query["coarse_material"]
            ],
        }
        for name, selected in selections.items():
            if selected:
                errors[name][query["parent_id"]] = error(
                    query,
                    prototype(selected, "standardized_target"),
                    "standardized_target",
                )
    common = sorted(set.intersection(*(set(value) for value in errors.values())))
    return {
        "common_parent_count": len(common),
        "common_support": {
            name: summarize([errors[name][parent_id] for parent_id in common])
            for name in names
        },
        "per_control": {name: summarize(list(errors[name].values())) for name in names},
    }


def relative_improvement(global_error: float, material_error: float) -> float:
    if global_error <= 0:
        raise AuditError("global control error must be positive")
    return (global_error - material_error) / global_error


def bootstrap_interval(values: np.ndarray) -> dict[str, Any]:
    policy = AUDIT_POLICY["bootstrap"]
    rng = np.random.Generator(np.random.PCG64(policy["seed"]))
    statistics = np.empty(policy["iterations"], dtype=np.float64)
    for index in range(policy["iterations"]):
        sample = values[rng.integers(0, len(values), len(values))]
        statistics[index] = float(np.median(sample))
    low, high = np.percentile(
        statistics,
        policy["confidence_interval_percent"],
        method="linear",
    )
    return {
        "high": rounded(high),
        "iterations": policy["iterations"],
        "low": rounded(low),
        "median": rounded(float(np.median(statistics))),
        "seed": policy["seed"],
    }


def evaluate_lpo(
    parents: list[dict[str, Any]], key: str, require_multi_parent_projects: bool
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    project_counts = Counter(parent["project_id"] for parent in parents)
    eligible_projects = {
        project
        for project, count in project_counts.items()
        if project is not None and (count >= 2 or not require_multi_parent_projects)
    }
    eligible = [
        parent for parent in parents if parent["project_id"] in eligible_projects
    ]
    rows = []
    for query in eligible:
        references = [
            parent for parent in eligible if parent["project_id"] != query["project_id"]
        ]
        material = [
            parent
            for parent in references
            if parent["coarse_material"] == query["coarse_material"]
        ]
        if not material:
            continue
        global_error = error(query, prototype(references, key), key)
        material_error = error(query, prototype(material, key), key)
        rows.append(
            {
                "coarse_material": query["coarse_material"],
                "global_error": rounded(global_error),
                "material_error": rounded(material_error),
                "material_improved": material_error < global_error,
                "parent_id": query["parent_id"],
                "project_id": query["project_id"],
                "relative_improvement": rounded(
                    relative_improvement(global_error, material_error)
                ),
            }
        )
    improvements = np.asarray(
        [row["relative_improvement"] for row in rows], dtype=np.float64
    )
    summary = {
        "global": summarize([row["global_error"] for row in rows]),
        "improved_parent_fraction": rounded(
            float(np.mean([row["material_improved"] for row in rows]))
        ),
        "material": summarize([row["material_error"] for row in rows]),
        "median_relative_improvement": rounded(float(np.median(improvements))),
        "per_coarse_material": {},
    }
    for material in sorted({row["coarse_material"] for row in rows}):
        selected = [row for row in rows if row["coarse_material"] == material]
        summary["per_coarse_material"][material] = {
            "global": summarize([row["global_error"] for row in selected]),
            "improved_parent_fraction": rounded(
                float(np.mean([row["material_improved"] for row in selected]))
            ),
            "material": summarize([row["material_error"] for row in selected]),
            "median_relative_improvement": rounded(
                float(np.median([row["relative_improvement"] for row in selected]))
            ),
        }
    return summary, rows


def add_source_centered_targets(parents: list[dict[str, Any]]) -> None:
    project_groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for parent in parents:
        if parent["project_id"] is not None:
            project_groups[parent["project_id"]].append(parent)
    means = {
        project: prototype(group, "standardized_target")
        for project, group in project_groups.items()
    }
    for parent in parents:
        if parent["project_id"] is not None:
            parent["source_centered_target"] = (
                parent["standardized_target"] - means[parent["project_id"]]
            )


def project_classifier_score(
    parents: list[dict[str, Any]],
    labels: list[str],
    eligible: list[int],
) -> tuple[float, float, list[dict[str, Any]]]:
    decisions = []
    for index in eligible:
        query = parents[index]
        references = [
            reference
            for reference in range(len(parents))
            if reference != index
            and parents[reference]["coarse_material"] == query["coarse_material"]
        ]
        projects = sorted({labels[reference] for reference in references})
        if labels[index] not in projects or len(projects) < 2:
            raise AuditError("permuted classifier lost frozen eligible support")
        distances = []
        for project in projects:
            selected = [parents[ref] for ref in references if labels[ref] == project]
            distances.append(
                (
                    error(
                        query,
                        prototype(selected, "standardized_target"),
                        "standardized_target",
                    ),
                    project,
                )
            )
        prediction = min(distances)[1]
        decisions.append(
            {
                "coarse_material": query["coarse_material"],
                "correct": prediction == labels[index],
                "parent_id": query["parent_id"],
                "predicted_project_id": prediction,
                "project_id": labels[index],
            }
        )
    accuracy = float(np.mean([item["correct"] for item in decisions]))
    recalls = []
    for project in sorted({item["project_id"] for item in decisions}):
        selected = [item for item in decisions if item["project_id"] == project]
        recalls.append(float(np.mean([item["correct"] for item in selected])))
    return accuracy, float(np.mean(recalls)), decisions


def project_identity_audit(parents: list[dict[str, Any]]) -> dict[str, Any]:
    labels = [parent["project_id"] for parent in parents]
    eligible = []
    for index, query in enumerate(parents):
        references = [
            ref
            for ref, parent in enumerate(parents)
            if ref != index and parent["coarse_material"] == query["coarse_material"]
        ]
        projects = {labels[ref] for ref in references}
        if labels[index] in projects and len(projects) >= 2:
            eligible.append(index)
    accuracy, balanced, decisions = project_classifier_score(parents, labels, eligible)
    policy = AUDIT_POLICY["permutation"]
    rng = np.random.Generator(np.random.PCG64(policy["seed"]))
    null_accuracy = np.empty(policy["iterations"], dtype=np.float64)
    null_balanced = np.empty(policy["iterations"], dtype=np.float64)
    materials = sorted({parent["coarse_material"] for parent in parents})
    for iteration in range(policy["iterations"]):
        permuted = list(labels)
        for material in materials:
            indices = [
                index
                for index in eligible
                if parents[index]["coarse_material"] == material
            ]
            old = [labels[index] for index in indices]
            order = rng.permutation(len(indices))
            for offset, index in enumerate(indices):
                permuted[index] = old[int(order[offset])]
        null_accuracy[iteration], null_balanced[iteration], _ = (
            project_classifier_score(parents, permuted, eligible)
        )
    p_value = (1.0 + float(np.sum(null_balanced >= balanced))) / (
        policy["iterations"] + 1.0
    )
    return {
        "accuracy": rounded(accuracy),
        "accuracy_lift_over_null_median": rounded(
            accuracy - float(np.median(null_accuracy))
        ),
        "balanced_accuracy": rounded(balanced),
        "decisions": decisions,
        "eligible_parent_count": len(eligible),
        "null": {
            "accuracy_mean": rounded(float(np.mean(null_accuracy))),
            "accuracy_p05": rounded(
                float(np.percentile(null_accuracy, 5, method="linear"))
            ),
            "accuracy_p50": rounded(float(np.median(null_accuracy))),
            "accuracy_p95": rounded(
                float(np.percentile(null_accuracy, 95, method="linear"))
            ),
            "balanced_accuracy_mean": rounded(float(np.mean(null_balanced))),
            "balanced_accuracy_p05": rounded(
                float(np.percentile(null_balanced, 5, method="linear"))
            ),
            "balanced_accuracy_p50": rounded(float(np.median(null_balanced))),
            "balanced_accuracy_p95": rounded(
                float(np.percentile(null_balanced, 95, method="linear"))
            ),
            "iterations": policy["iterations"],
            "p_value": rounded(p_value),
            "seed": policy["seed"],
        },
    }


def explained_fraction(
    parents: list[dict[str, Any]], group: Callable[[dict[str, Any]], str | None]
) -> float:
    between = 0.0
    total = 0.0
    for dimension in range(b0.REPRESENTATION_POLICY["target_dimension"]):
        selected = [
            parent
            for parent in parents
            if group(parent) is not None and parent["target_mask"][dimension]
        ]
        if not selected:
            continue
        values = np.asarray(
            [parent["standardized_target"][dimension] for parent in selected]
        )
        global_mean = float(np.mean(values))
        total += float(np.sum((values - global_mean) ** 2))
        grouped: dict[str, list[float]] = defaultdict(list)
        for parent in selected:
            key = group(parent)
            if key is None:
                raise AuditError("group key disappeared")
            grouped[key].append(float(parent["standardized_target"][dimension]))
        between += sum(
            len(group_values) * (float(np.mean(group_values)) - global_mean) ** 2
            for group_values in grouped.values()
        )
    if total <= 0:
        raise AuditError("variance attribution has zero total variation")
    return between / total


def anchor_diagnostics(
    rows: list[dict[str, Any]],
    corpus_root: Path,
    mean: np.ndarray,
    scale: np.ndarray,
) -> tuple[int, list[dict[str, Any]]]:
    by_parent: dict[str, dict[str, list[tuple[np.ndarray, np.ndarray]]]] = defaultdict(
        lambda: defaultdict(list)
    )
    bytes_read = 0
    parent_families: dict[str, set[str]] = defaultdict(set)
    for row in rows:
        parent_families[row["physical_parent_id"]].add(row["family_id"])
    cross_source = {
        parent_id
        for parent_id, families in parent_families.items()
        if len(families) > 1
    }
    for row in rows:
        if row["physical_parent_id"] not in cross_source:
            continue
        count, feature = b0.load_feature(corpus_root, row["acoustic_target"])
        bytes_read += count
        vector, mask = b0.feature_vector(feature)
        by_parent[row["physical_parent_id"]][row["family_id"]].append((vector, mask))
    results = []
    for parent_id in sorted(by_parent):
        family_vectors = []
        for family_id, values in sorted(by_parent[parent_id].items()):
            aggregate = b0.masked_mean(
                [value for value, _ in values], [mask for _, mask in values]
            )
            mask = np.any(np.stack([item for _, item in values]), axis=0)
            family_vectors.append(
                (family_id, (aggregate - mean) / scale, mask, len(values))
            )
        if len(family_vectors) != 2:
            raise AuditError("R0 supports exactly two-family cross-source anchors")
        left, right = family_vectors
        common = left[2] & right[2]
        category = {}
        for name, (start, stop) in b0.metric_groups().items():
            category[name] = rounded(
                b0.parent_distance(
                    left[1][start:stop],
                    right[1][start:stop],
                    common[start:stop],
                )
            )
        results.append(
            {
                "aggregate_rmse": rounded(
                    b0.parent_distance(left[1], right[1], common)
                ),
                "category_rmse": category,
                "families": [
                    {"family_id": left[0], "record_count": left[3]},
                    {"family_id": right[0], "record_count": right[3]},
                ],
                "parent_id": parent_id,
            }
        )
    return bytes_read, results


def decision(
    classifier: dict[str, Any],
    raw_lpo: dict[str, Any],
    centered_lpo: dict[str, Any],
    bootstrap: dict[str, Any],
) -> tuple[str, dict[str, bool]]:
    domain = DECISION_POLICY["domain_signal"]
    material = DECISION_POLICY["material_signal"]
    normalization = DECISION_POLICY["normalization_recovery"]
    gates = {
        "domain_signal": (
            classifier["eligible_parent_count"] >= domain["minimum_eligible_parents"]
            and classifier["balanced_accuracy"] >= domain["minimum_balanced_accuracy"]
            and classifier["accuracy_lift_over_null_median"]
            >= domain["minimum_accuracy_lift_over_null_median"]
            and classifier["null"]["p_value"] <= domain["maximum_permutation_p_value"]
        ),
        "normalization_recovery": (
            centered_lpo["median_relative_improvement"]
            >= normalization["minimum_median_relative_improvement"]
            and centered_lpo["improved_parent_fraction"]
            >= normalization["minimum_improved_parent_fraction"]
        ),
        "raw_material_signal": (
            raw_lpo["median_relative_improvement"]
            >= material["minimum_median_relative_improvement"]
            and raw_lpo["improved_parent_fraction"]
            >= material["minimum_improved_parent_fraction"]
            and bootstrap["low"]
            > material["minimum_bootstrap_lower_relative_improvement"]
        ),
    }
    if gates["raw_material_signal"]:
        result = "DescriptorSignalPlausible"
    elif gates["domain_signal"] and gates["normalization_recovery"]:
        result = "DomainNormalizationRequired"
    else:
        result = "CorpusSignalInsufficient"
    if result not in DECISIONS:
        raise AuditError("R0 decision is outside the frozen set")
    return result, gates


def power_plan(raw_rows: list[dict[str, Any]]) -> dict[str, Any]:
    values = np.asarray(
        [row["relative_improvement"] for row in raw_rows], dtype=np.float64
    )
    if len(values) < 2:
        raise AuditError("power plan requires multiple grouped parents")
    deviation = float(np.std(values, ddof=1))
    target = POWER_POLICY["practical_relative_improvement"]
    z_total = POWER_POLICY["z_alpha_two_sided"] + POWER_POLICY["z_power"]
    minimum = int(math.ceil((z_total * deviation / target) ** 2))
    return {
        "current_supported_parent_count": len(values),
        "estimated_relative_delta_stddev": rounded(deviation),
        "minimum_supported_evaluation_parents": minimum,
        "policy": POWER_POLICY,
        "schema": POWER_SCHEMA,
        "warning": "normal_approximation_planning_floor_not_admission_evidence",
    }


def verify_counts(
    profile: dict[str, Any],
    train_rows: list[dict[str, Any]],
    development_rows: list[dict[str, Any]],
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    parents: list[dict[str, Any]],
    raw_rows: list[dict[str, Any]],
    centered_rows: list[dict[str, Any]],
    classifier: dict[str, Any],
) -> dict[str, int]:
    single = [parent for parent in parents if parent["project_id"] is not None]
    actual = {
        "combined_parents": len(parents),
        "cross_source_parents": len(parents) - len(single),
        "development_parents": len(development),
        "development_records": len(development_rows),
        "leave_project_out_supported_parents": len(raw_rows),
        "project_classifier_eligible_parents": classifier["eligible_parent_count"],
        "project_count": len({parent["project_id"] for parent in single}),
        "single_source_parents": len(single),
        "source_centered_supported_parents": len(centered_rows),
        "train_parents": len(train),
        "train_records": len(train_rows),
    }
    if actual != profile["expected_counts"]:
        raise AuditError(f"R0 count drift: {actual}")
    return actual


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    corpus_root: Path,
    baseline_root: Path,
) -> dict[str, bytes]:
    baseline_bytes, _ = validate_baseline_inputs(
        baseline_root, profile["input_bindings"]["baseline"]
    )
    corpus_bindings = profile["input_bindings"]["corpus"]
    corpus_metadata_bytes = 0
    for name in ("corpus_card", "profile", "report"):
        data, _ = load_bound_json(corpus_root, corpus_bindings[name], f"C0 {name}")
        corpus_metadata_bytes += len(data)
    train_bytes, train_projection = b0.load_projection(
        corpus_root, corpus_bindings["train_projection"], b0.TRAIN_ROLE
    )
    development_bytes, development_projection = b0.load_projection(
        corpus_root,
        corpus_bindings["development_projection"],
        b0.DEVELOPMENT_ROLE,
    )
    train_rows = train_projection["rows"]
    development_rows = development_projection["rows"]
    b0.validate_split(train_rows, development_rows)
    train, train_feature_bytes = b0.aggregate_parents(
        train_rows, corpus_root, profile["material_taxonomy"]
    )
    development, development_feature_bytes = b0.aggregate_parents(
        development_rows, corpus_root, profile["material_taxonomy"]
    )
    mean, scale = b0.fit_scaler(train)
    parents = prepare_parents(
        train_rows, development_rows, train, development, mean, scale
    )
    single = [parent for parent in parents if parent["project_id"] is not None]

    loo = leave_one_parent_out(single)
    raw_lpo, raw_rows = evaluate_lpo(
        single, "standardized_target", require_multi_parent_projects=False
    )
    raw_bootstrap = bootstrap_interval(
        np.asarray([row["relative_improvement"] for row in raw_rows], dtype=np.float64)
    )
    add_source_centered_targets(single)
    centered_lpo, centered_rows = evaluate_lpo(
        single, "source_centered_target", require_multi_parent_projects=True
    )
    classifier = project_identity_audit(single)
    outcome, decision_gates = decision(classifier, raw_lpo, centered_lpo, raw_bootstrap)
    counts = verify_counts(
        profile,
        train_rows,
        development_rows,
        train,
        development,
        parents,
        raw_rows,
        centered_rows,
        classifier,
    )
    anchor_feature_bytes, anchors = anchor_diagnostics(
        train_rows + development_rows, corpus_root, mean, scale
    )
    variance = {
        "coarse_material_explained_fraction": rounded(
            explained_fraction(single, lambda parent: parent["coarse_material"])
        ),
        "exact_material_explained_fraction": rounded(
            explained_fraction(single, lambda parent: parent["material_label"])
        ),
        "project_explained_fraction": rounded(
            explained_fraction(single, lambda parent: parent["project_id"])
        ),
        "role_explained_fraction": rounded(
            explained_fraction(single, lambda parent: parent["role"])
        ),
        "warning": "marginal_nonorthogonal_descriptive_fractions_do_not_sum_to_one",
    }
    project_counts = Counter(parent["project_id"] for parent in single)
    inventory = {
        "cross_source_anchors": anchors,
        "parents": [
            {
                "coarse_material": parent["coarse_material"],
                "family_component_id": parent["family_component_id"],
                "family_ids": parent["family_ids"],
                "material_label": parent["material_label"],
                "parent_id": parent["parent_id"],
                "project_id": parent["project_id"],
                "record_count": parent["record_count"],
                "role": parent["role"],
            }
            for parent in parents
        ],
        "project_parent_counts": dict(sorted(project_counts.items())),
        "schema": INVENTORY_SCHEMA,
    }
    audit = {
        "bootstrap": raw_bootstrap,
        "decision": outcome,
        "decision_gates": decision_gates,
        "leave_one_parent_out": loo,
        "leave_project_out": {
            "raw": raw_lpo,
            "source_centered_oracle": centered_lpo,
        },
        "project_identity": classifier,
        "schema": AUDIT_SCHEMA,
        "variance_attribution": variance,
    }
    power = power_plan(raw_rows)
    access = {
        "anchor_feature_bytes_read": anchor_feature_bytes,
        "baseline_metadata_bytes_read": baseline_bytes,
        "candidate_model_bytes_read": 0,
        "corpus_metadata_bytes_read": corpus_metadata_bytes,
        "development_feature_bytes_read": development_feature_bytes,
        "development_projection_bytes_read": len(development_bytes),
        "network_requests": 0,
        "pcm_bytes_read": 0,
        "protected_bytes_read": 0,
        "train_feature_bytes_read": train_feature_bytes,
        "train_projection_bytes_read": len(train_bytes),
        "validator_feature_bytes_read": 0,
        "validator_projection_bytes_read": 0,
    }
    access_document = {
        "access": access,
        "authority": AUTHORITY,
        "profile_sha256": sha256_bytes(profile_bytes),
        "schema": ACCESS_SCHEMA,
    }
    artifacts = {
        "access_ledger.json": canonical_json(access_document),
        "audit.json": canonical_json(audit),
        "parent_inventory.json": canonical_json(inventory),
        "power_plan.json": canonical_json(power),
        "profile.json": profile_bytes,
    }
    artifact_bindings = {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in sorted(artifacts.items())
    }
    gates = {
        "b0_global_floor_verified": True,
        "combined_parent_inventory_complete": len(parents)
        == counts["combined_parents"],
        "cross_source_parent_excluded_from_project_statistics": counts[
            "cross_source_parents"
        ]
        == len(anchors),
        "decision_is_exclusive": outcome in DECISIONS,
        "forbidden_access_zero": all(
            access[name] == 0
            for name in (
                "candidate_model_bytes_read",
                "network_requests",
                "pcm_bytes_read",
                "protected_bytes_read",
                "validator_feature_bytes_read",
                "validator_projection_bytes_read",
            )
        ),
        "material_and_project_diagnostics_complete": (
            len(raw_rows) == counts["leave_project_out_supported_parents"]
            and classifier["eligible_parent_count"]
            == counts["project_classifier_eligible_parents"]
        ),
        "no_candidate_training_or_selection": True,
    }
    if not all(gates.values()):
        raise AuditError("R0 conjunctive gate failed")
    next_actions = (
        ["v42_m0_freeze_after_b1", "v42_t0_recipe_representation", "v42_v0"]
        if outcome == "DescriptorSignalPlausible"
        else (
            ["v42_c1_descriptor_source_growth", "v42_t0_domain_normalization", "v42_v0"]
            if outcome == "DomainNormalizationRequired"
            else ["v42_c1_descriptor_source_growth", "v42_v0"]
        )
    )
    report = {
        "access": access,
        "artifacts": artifact_bindings,
        "authority": AUTHORITY,
        "counts": counts,
        "decision": outcome,
        "gates": gates,
        "next_actions": next_actions,
        "result": {
            "cross_project_material_median_relative_improvement": raw_lpo[
                "median_relative_improvement"
            ],
            "cross_project_material_paired_bootstrap_95_low": raw_bootstrap["low"],
            "minimum_supported_evaluation_parents": power[
                "minimum_supported_evaluation_parents"
            ],
            "project_balanced_accuracy": classifier["balanced_accuracy"],
            "project_permutation_p_value": classifier["null"]["p_value"],
            "source_centered_material_median_relative_improvement": centered_lpo[
                "median_relative_improvement"
            ],
        },
        "schema": REPORT_SCHEMA,
    }
    artifacts["report.json"] = canonical_json(report)
    return artifacts


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def prepare_root(path: Path, context: str) -> Path:
    if path.is_symlink() or not path.is_dir():
        raise AuditError(f"{context} must be a non-symlink directory")
    return path.resolve()


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise AuditError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise AuditError("output must remain outside the repository")
    if resolved.exists():
        raise AuditError(f"refusing to replace existing output: {resolved}")
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
                raise AuditError("R0 outputs must be flat files")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(
    profile_path: Path,
    corpus_root: Path,
    baseline_root: Path,
    output: Path,
) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(raw_profile)
    return publish_directory(
        output,
        build_documents(
            profile_bytes,
            profile,
            prepare_root(corpus_root, "C0 corpus root"),
            prepare_root(baseline_root, "B0 baseline root"),
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    print(
        run(
            arguments.profile,
            arguments.corpus_root,
            arguments.baseline_root,
            arguments.output,
        )
    )


if __name__ == "__main__":
    main()
