#!/usr/bin/env python3
"""Freeze corrected grouped baselines and domain diagnostics on V43 C0R."""

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
from typing import Any

import numpy as np

import physical_sound_v41_b0_grouped_baselines_v1 as b0
import physical_sound_v42_r0_domain_information_audit_v1 as r0

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-r0r-profile.v1"
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-role-projection.v1"
REPRESENTATION_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-b0r-representation.v1"
)
MODEL_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-models.v1"
PREDICTION_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-predictions.v1"
METRICS_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-metrics.v1"
AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v44-r0r-audit.v1"
INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-v44-r0r-inventory.v1"
POWER_SCHEMA = "nextengine.experimental-physical-sound-v44-r0r-power-plan.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-r0r-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-r0r-report.v1"

TRAIN_ROLE = b0.TRAIN_ROLE
DEVELOPMENT_ROLE = b0.DEVELOPMENT_ROLE
UNKNOWN_MATERIAL = "Unknown"
UNOBSERVED_CLASS = "__material_unobserved__"
MAX_JSON_BYTES = 32 * 1024 * 1024
HASH_LENGTH = 64

DECISIONS = (
    "CorrectedCorpusSignalInsufficient",
    "CorrectedDescriptorSignalPlausible",
    "CorrectedDomainNormalizationRequired",
)

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": ["v44_c1_descriptor_source_growth"],
    "baseline_fit_authority": True,
    "candidate_model_training_authority": False,
    "descriptor_or_recipe_freeze_authority": False,
    "diagnostic_audit_authority": True,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_access_authority": False,
}

MATERIAL_MASK_POLICY = {
    "axis_key": "material_identity",
    "known_label_requires_axis_true": True,
    "material_conditioned_models_exclude_unobserved": True,
    "material_unobserved_class": UNOBSERVED_CLASS,
    "material_unobserved_label": UNKNOWN_MATERIAL,
    "material_unobserved_requires_axis_false": True,
    "material_unobserved_target_use": "scaler_and_global_prototype_only",
    "parent_members_must_agree": True,
}


class CorrectedBaselineError(RuntimeError):
    """The corrected B0R/R0R surface cannot be published safely."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(
                value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False
            )
            + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise CorrectedBaselineError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def rounded(value: float, digits: int = 9) -> float:
    if not math.isfinite(value):
        raise CorrectedBaselineError("derived value is non-finite")
    result = round(float(value), digits)
    return 0.0 if result == 0 else result


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise CorrectedBaselineError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise CorrectedBaselineError(f"{context} must be an array")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise CorrectedBaselineError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_JSON_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise CorrectedBaselineError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise CorrectedBaselineError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise CorrectedBaselineError(f"{context} is not valid UTF-8 JSON") from error
    return require_dict(value, context)


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise CorrectedBaselineError(f"{context} binding fields changed")
    expected_bytes = binding["bytes"]
    require_hash(binding["sha256"], f"{context} SHA-256")
    if type(expected_bytes) is not int or expected_bytes <= 0:
        raise CorrectedBaselineError(f"{context} byte count is invalid")
    data = read_regular(path, context)
    if len(data) != expected_bytes or sha256_bytes(data) != binding["sha256"]:
        raise CorrectedBaselineError(f"{context} bytes or SHA-256 changed")
    return data


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, "B0R/R0R profile")
    profile = parse_json(data, "B0R/R0R profile")
    if canonical_json(profile) != data:
        raise CorrectedBaselineError("B0R/R0R profile must use canonical JSON")
    return data, profile


def validate_dependencies(profile: dict[str, Any]) -> None:
    bindings = require_list(profile.get("dependency_bindings"), "dependencies")
    paths = []
    for raw in bindings:
        binding = require_dict(raw, "dependency")
        if set(binding) != {"bytes", "path", "sha256"}:
            raise CorrectedBaselineError("dependency fields changed")
        path_text = binding["path"]
        if not isinstance(path_text, str) or not path_text:
            raise CorrectedBaselineError("dependency path is invalid")
        path = Path(path_text)
        if path.is_absolute() or ".." in path.parts:
            raise CorrectedBaselineError("dependency path escapes the repository")
        paths.append(path_text)
        check_binding(repository_root() / path, binding, f"dependency {path_text}")
    if paths != sorted(set(paths)):
        raise CorrectedBaselineError("dependencies must be path-sorted and unique")


def validate_profile(value: dict[str, Any]) -> dict[str, Any]:
    required = {
        "audit_policy",
        "authority",
        "baseline_policy",
        "corpus_identity",
        "decision_policy",
        "dependency_bindings",
        "environment",
        "evaluation_policy",
        "expected_counts",
        "input_bindings",
        "material_mask_policy",
        "material_taxonomy",
        "power_policy",
        "priority_materials",
        "profile_id",
        "representation_policy",
        "schema",
    }
    if set(value) != required:
        raise CorrectedBaselineError("B0R/R0R profile fields changed")
    if value["schema"] != PROFILE_SCHEMA or not isinstance(value["profile_id"], str):
        raise CorrectedBaselineError("unknown B0R/R0R profile identity")
    for key, expected in (
        ("audit_policy", r0.AUDIT_POLICY),
        ("authority", AUTHORITY),
        ("baseline_policy", b0.BASELINE_POLICY),
        ("decision_policy", r0.DECISION_POLICY),
        ("evaluation_policy", b0.EVALUATION_POLICY),
        ("material_mask_policy", MATERIAL_MASK_POLICY),
        ("power_policy", r0.POWER_POLICY),
        ("representation_policy", b0.REPRESENTATION_POLICY),
    ):
        if value[key] != expected:
            raise CorrectedBaselineError(f"B0R/R0R {key} changed")
    if value["environment"] != {"numpy_version": np.__version__}:
        raise CorrectedBaselineError("B0R/R0R NumPy environment changed")
    taxonomy = require_dict(value["material_taxonomy"], "material taxonomy")
    if not taxonomy or list(taxonomy) != sorted(taxonomy):
        raise CorrectedBaselineError("material taxonomy must be sorted and non-empty")
    if UNKNOWN_MATERIAL in taxonomy or UNOBSERVED_CLASS in taxonomy.values():
        raise CorrectedBaselineError(
            "unobserved material cannot become a taxonomy class"
        )
    if any(
        not isinstance(key, str) or not isinstance(item, str)
        for key, item in taxonomy.items()
    ):
        raise CorrectedBaselineError("material taxonomy values must be strings")
    priorities = require_list(value["priority_materials"], "priority materials")
    if priorities != sorted(set(priorities)) or any(
        item not in taxonomy for item in priorities
    ):
        raise CorrectedBaselineError("priority materials must be sorted known labels")
    identity = require_dict(value["corpus_identity"], "corpus identity")
    if set(identity) != {
        "corpus_schema",
        "manifest_root_sha256",
        "projection_schema",
        "report_decision",
        "report_schema",
    }:
        raise CorrectedBaselineError("corpus identity fields changed")
    require_hash(identity["manifest_root_sha256"], "manifest root")
    if identity["projection_schema"] != PROJECTION_SCHEMA:
        raise CorrectedBaselineError("C0R projection schema changed")
    counts = require_dict(value["expected_counts"], "expected counts")
    if set(counts) != {"audit", "baseline"}:
        raise CorrectedBaselineError("expected count groups changed")
    expected_keys = {
        "baseline": {
            "coarse_supported_development_parents",
            "development_parents",
            "development_records",
            "exact_supported_development_parents",
            "material_unobserved_train_parents",
            "train_parents",
            "train_records",
        },
        "audit": {
            "combined_parents",
            "cross_source_parents",
            "leave_project_out_supported_parents",
            "material_unobserved_parents",
            "project_classifier_eligible_parents",
            "project_count",
            "project_statistics_parents",
            "single_source_parents",
            "source_centered_supported_parents",
        },
    }
    for group, keys in expected_keys.items():
        values = require_dict(counts[group], f"{group} expected counts")
        if set(values) != keys or any(
            type(item) is not int or item < 0 for item in values.values()
        ):
            raise CorrectedBaselineError(f"{group} expected counts changed")
    inputs = require_dict(value["input_bindings"], "input bindings")
    if set(inputs) != {
        "corpus_card",
        "corpus_profile",
        "corpus_report",
        "development_projection",
        "lineage_repair",
        "manifest",
        "train_projection",
    }:
        raise CorrectedBaselineError("input bindings changed")
    for name, raw in inputs.items():
        binding = require_dict(raw, f"input {name}")
        if set(binding) != {"bytes", "path", "sha256"}:
            raise CorrectedBaselineError("input binding fields changed")
        path = Path(binding["path"])
        if path.is_absolute() or ".." in path.parts:
            raise CorrectedBaselineError("input path escapes the corpus root")
        require_hash(binding["sha256"], f"input {name}")
    validate_dependencies(value)
    return value


def load_bound_json(
    root: Path, binding: dict[str, Any], context: str
) -> tuple[bytes, dict[str, Any]]:
    data = check_binding(root / binding["path"], binding, context)
    value = parse_json(data, context)
    if canonical_json(value) != data:
        raise CorrectedBaselineError(f"{context} must remain canonical")
    return data, value


def validate_corpus_metadata(
    root: Path, profile: dict[str, Any]
) -> tuple[int, dict[str, Any]]:
    inputs = profile["input_bindings"]
    values = {}
    total = 0
    for name in (
        "corpus_card",
        "corpus_profile",
        "corpus_report",
        "lineage_repair",
        "manifest",
    ):
        data, value = load_bound_json(root, inputs[name], f"C0R {name}")
        values[name] = value
        total += len(data)
    identity = profile["corpus_identity"]
    report = values["corpus_report"]
    manifest = values["manifest"]
    card = values["corpus_card"]
    repair = values["lineage_repair"]
    if (
        report.get("schema") != identity["report_schema"]
        or report.get("decision") != identity["report_decision"]
    ):
        raise CorrectedBaselineError("C0R report identity changed")
    artifacts = require_dict(report.get("artifacts"), "C0R report artifacts")
    if artifacts.get("manifest_root_sha256") != identity["manifest_root_sha256"]:
        raise CorrectedBaselineError("C0R report manifest root changed")
    if (
        manifest.get("schema") != identity["corpus_schema"]
        or manifest.get("manifest_root_sha256") != identity["manifest_root_sha256"]
    ):
        raise CorrectedBaselineError("C0R manifest identity changed")
    projections = require_dict(manifest.get("projections"), "C0R projections")
    for role, input_name in (
        (TRAIN_ROLE, "train_projection"),
        (DEVELOPMENT_ROLE, "development_projection"),
    ):
        manifest_binding = require_dict(
            projections.get(role), f"C0R {role} manifest binding"
        )
        input_binding = inputs[input_name]
        if any(
            manifest_binding.get(key) != input_binding[key]
            for key in ("bytes", "path", "sha256")
        ):
            raise CorrectedBaselineError(f"C0R {role} binding differs from manifest")
    if (
        card.get("schema")
        != "nextengine.experimental-physical-sound-v43-c0r-corpus-card.v1"
    ):
        raise CorrectedBaselineError("C0R corpus card schema changed")
    quarantine = require_dict(
        repair.get("material_quarantine"), "C0R material quarantine"
    )
    if (
        repair.get("schema")
        != "nextengine.experimental-physical-sound-v43-d2-repair-record.v1"
        or quarantine.get("axis") != MATERIAL_MASK_POLICY["axis_key"]
        or quarantine.get("replacement_label") != UNKNOWN_MATERIAL
    ):
        raise CorrectedBaselineError("C0R material quarantine changed")
    if (
        values["corpus_profile"].get("schema")
        != "nextengine.experimental-physical-sound-v43-d2-c0r-profile.v1"
    ):
        raise CorrectedBaselineError("C0R profile schema changed")
    return total, values


def load_projection(
    root: Path, binding: dict[str, Any], role: str
) -> tuple[bytes, dict[str, Any]]:
    data, projection = load_bound_json(root, binding, f"C0R {role} projection")
    if projection.get("schema") != PROJECTION_SCHEMA or projection.get("role") != role:
        raise CorrectedBaselineError("C0R projection identity changed")
    rows = require_list(projection.get("rows"), "projection rows")
    if projection.get("record_count") != len(rows):
        raise CorrectedBaselineError("C0R projection record count changed")
    if projection.get("rows_root_sha256") != sha256_bytes(canonical_json(rows)):
        raise CorrectedBaselineError("C0R projection row root changed")
    if len({row.get("record_id") for row in rows}) != len(rows):
        raise CorrectedBaselineError("projection record IDs are not unique")
    return data, projection


def validate_material_masks(
    rows: list[dict[str, Any]], taxonomy: dict[str, str]
) -> dict[str, bool]:
    parent_state: dict[str, tuple[str, bool]] = {}
    for raw in rows:
        row = require_dict(raw, "projection row")
        if set(row) != {
            "acoustic_target",
            "axis_mask",
            "canonical_pcm",
            "family_component_id",
            "family_id",
            "material_label",
            "physical_parent_id",
            "record_id",
        }:
            raise CorrectedBaselineError("C0R row fields changed")
        axes = require_dict(row["axis_mask"], "axis mask")
        observed = axes.get(MATERIAL_MASK_POLICY["axis_key"])
        label = row["material_label"]
        if type(observed) is not bool:
            raise CorrectedBaselineError("material identity mask is not boolean")
        if label == UNKNOWN_MATERIAL:
            if observed:
                raise CorrectedBaselineError("unobserved material is marked observed")
        elif label not in taxonomy or not observed:
            raise CorrectedBaselineError("known material label/mask is invalid")
        parent_id = row.get("physical_parent_id")
        if not isinstance(parent_id, str) or not parent_id:
            raise CorrectedBaselineError("physical parent identity is invalid")
        state = (label, observed)
        previous = parent_state.setdefault(parent_id, state)
        if previous != state:
            raise CorrectedBaselineError(
                "one physical parent crosses material mask state"
            )
    return {parent_id: state[1] for parent_id, state in parent_state.items()}


def aggregate_parents(
    rows: list[dict[str, Any]], root: Path, taxonomy: dict[str, str]
) -> tuple[list[dict[str, Any]], int]:
    observed = validate_material_masks(rows, taxonomy)
    extended = dict(taxonomy)
    extended[UNKNOWN_MATERIAL] = UNOBSERVED_CLASS
    parents, feature_bytes = b0.aggregate_parents(rows, root, extended)
    for parent in parents:
        parent["material_observed"] = observed[parent["parent_id"]]
    return parents, feature_bytes


def material_inputs(parents: list[dict[str, Any]], classes: list[str]) -> np.ndarray:
    lookup = {name: index for index, name in enumerate(classes)}
    matrix = np.zeros((len(parents), len(classes)), dtype=np.float64)
    for row, parent in enumerate(parents):
        if parent["material_observed"]:
            material = parent["coarse_material"]
            if material not in lookup:
                raise CorrectedBaselineError(
                    "observed material has no conditioning class"
                )
            matrix[row, lookup[material]] = 1.0
    return matrix


def fit_corrected_baselines(
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    taxonomy: dict[str, str],
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]:
    mean, scale = b0.fit_scaler(train)
    train_targets = np.stack([b0.standardized(parent, mean, scale) for parent in train])
    train_masks = np.stack([parent["target_mask"] for parent in train])
    global_prototype = b0.masked_mean(list(train_targets), list(train_masks))
    conditioned = [
        index for index, parent in enumerate(train) if parent["material_observed"]
    ]
    if not conditioned:
        raise CorrectedBaselineError("no material-observed train parent remains")
    classes = b0.conditioning_classes(taxonomy)
    train_inputs = material_inputs([train[index] for index in conditioned], classes)
    development_inputs = material_inputs(development, classes)

    by_class: dict[str, list[int]] = defaultdict(list)
    for index in conditioned:
        by_class[train[index]["coarse_material"]].append(index)
    prototypes: dict[str, np.ndarray] = {}
    retrieval: dict[str, int] = {}
    medoids: dict[str, int] = {}
    for material in sorted(by_class):
        indices = by_class[material]
        prototypes[material] = b0.masked_mean(
            [train_targets[index] for index in indices],
            [train_masks[index] for index in indices],
        )
        retrieval[material] = min(indices, key=lambda index: train[index]["parent_id"])
        medoids[material] = min(
            indices,
            key=lambda index: (
                b0.parent_distance(
                    train_targets[index], prototypes[material], train_masks[index]
                ),
                train[index]["parent_id"],
            ),
        )

    conditioned_targets = train_targets[conditioned]
    conditioned_masks = train_masks[conditioned]
    ridge_bias, ridge_weights = b0.fit_ridge(
        train_inputs, conditioned_targets, conditioned_masks
    )
    mlp, final_loss = b0.fit_mlp(train_inputs, conditioned_targets, conditioned_masks)
    predictions = {
        "global_prototype": np.repeat(
            global_prototype[np.newaxis, :], len(development), axis=0
        ),
        "ridge": b0.ridge_predict(development_inputs, ridge_bias, ridge_weights),
        "pointwise_mlp": b0.mlp_predict(development_inputs, mlp),
    }
    predictions["material_modal_prototype"] = np.stack(
        [
            prototypes.get(parent["coarse_material"], global_prototype)
            for parent in development
        ]
    )
    predictions["retrieval_copy"] = np.stack(
        [
            train_targets[retrieval[parent["coarse_material"]]]
            if parent["coarse_material"] in retrieval
            else global_prototype
            for parent in development
        ]
    )
    predictions["nearest_local_medoid"] = np.stack(
        [
            train_targets[medoids[parent["coarse_material"]]]
            if parent["coarse_material"] in medoids
            else global_prototype
            for parent in development
        ]
    )
    train_materials = {
        parent["material_label"] for parent in train if parent["material_observed"]
    }
    train_classes = set(by_class)
    metrics, prediction_detail = b0.evaluate_predictions(
        development,
        predictions,
        mean,
        scale,
        train_materials,
        train_classes,
    )
    ranking = sorted(
        b0.BASELINE_NAMES,
        key=lambda name: (
            metrics[name]["coarse_supported"]["median_parent_rmse"],
            name,
        ),
    )
    model = {
        "conditioning_classes": classes,
        "conditioning_training_parent_count": len(conditioned),
        "global_prototype": b0.rounded_array(global_prototype),
        "global_training_parent_count": len(train),
        "material_prototypes": {
            key: b0.rounded_array(value) for key, value in sorted(prototypes.items())
        },
        "material_unobserved_train_parent_count": len(train) - len(conditioned),
        "mlp": {
            "final_train_masked_mse": b0.rounded(final_loss),
            "parameters": {
                name: {"shape": list(value.shape), "values": b0.rounded_array(value)}
                for name, value in sorted(mlp.items())
            },
        },
        "nearest_local_medoid_parent": {
            key: train[index]["parent_id"] for key, index in sorted(medoids.items())
        },
        "retrieval_copy_parent": {
            key: train[index]["parent_id"] for key, index in sorted(retrieval.items())
        },
        "ridge": {
            "bias": b0.rounded_array(ridge_bias),
            "weights": {
                "shape": list(ridge_weights.shape),
                "values": b0.rounded_array(ridge_weights),
            },
        },
        "scaler": {"mean": b0.rounded_array(mean), "scale": b0.rounded_array(scale)},
        "schema": MODEL_SCHEMA,
    }
    summary = {
        "best_baseline": ranking[0],
        "ranking": ranking,
        "reference_floor": metrics[ranking[0]]["coarse_supported"],
        "train_coarse_materials": sorted(train_classes),
        "train_exact_materials": sorted(train_materials),
    }
    return model, metrics, prediction_detail, summary


def corrected_decision(
    classifier: dict[str, Any],
    raw_lpo: dict[str, Any],
    centered_lpo: dict[str, Any],
    bootstrap: dict[str, Any],
) -> tuple[str, dict[str, bool]]:
    old, gates = r0.decision(classifier, raw_lpo, centered_lpo, bootstrap)
    mapping = {
        "CorpusSignalInsufficient": "CorrectedCorpusSignalInsufficient",
        "DescriptorSignalPlausible": "CorrectedDescriptorSignalPlausible",
        "DomainNormalizationRequired": "CorrectedDomainNormalizationRequired",
    }
    result = mapping[old]
    if result not in DECISIONS:
        raise CorrectedBaselineError("corrected decision is outside the frozen set")
    return result, gates


def anchor_diagnostics(
    rows: list[dict[str, Any]],
    corpus_root: Path,
    mean: np.ndarray,
    scale: np.ndarray,
) -> tuple[int, list[dict[str, Any]]]:
    parent_families: dict[str, set[str]] = defaultdict(set)
    for row in rows:
        parent_families[row["physical_parent_id"]].add(row["family_id"])
    cross_source = {
        parent_id
        for parent_id, family_ids in parent_families.items()
        if len(family_ids) > 1
    }
    by_parent: dict[str, dict[str, list[tuple[np.ndarray, np.ndarray]]]] = defaultdict(
        lambda: defaultdict(list)
    )
    bytes_read = 0
    for row in rows:
        if row["physical_parent_id"] not in cross_source:
            continue
        count, feature = b0.load_feature(corpus_root, row["acoustic_target"])
        bytes_read += count
        vector, mask = b0.feature_vector(feature)
        by_parent[row["physical_parent_id"]][row["family_id"]].append((vector, mask))

    results = []
    for parent_id in sorted(by_parent):
        families = []
        for family_id, values in sorted(by_parent[parent_id].items()):
            aggregate = b0.masked_mean(
                [value for value, _ in values], [mask for _, mask in values]
            )
            mask = np.any(np.stack([item for _, item in values]), axis=0)
            families.append(
                {
                    "family_id": family_id,
                    "mask": mask,
                    "record_count": len(values),
                    "standardized_target": (aggregate - mean) / scale,
                }
            )
        if len(families) < 2:
            raise CorrectedBaselineError("cross-source anchor lost family support")
        comparisons = []
        for left_index, left in enumerate(families):
            for right in families[left_index + 1 :]:
                common = left["mask"] & right["mask"]
                category = {}
                for name, (start, stop) in b0.metric_groups().items():
                    category[name] = rounded(
                        b0.parent_distance(
                            left["standardized_target"][start:stop],
                            right["standardized_target"][start:stop],
                            common[start:stop],
                        )
                    )
                comparisons.append(
                    {
                        "aggregate_rmse": rounded(
                            b0.parent_distance(
                                left["standardized_target"],
                                right["standardized_target"],
                                common,
                            )
                        ),
                        "category_rmse": category,
                        "left_family_id": left["family_id"],
                        "right_family_id": right["family_id"],
                    }
                )
        results.append(
            {
                "families": [
                    {
                        "family_id": family["family_id"],
                        "record_count": family["record_count"],
                    }
                    for family in families
                ],
                "pair_count": len(comparisons),
                "pairwise_comparisons": comparisons,
                "parent_id": parent_id,
            }
        )
    return bytes_read, results


def priority_material_support(
    parents: list[dict[str, Any]], priorities: list[str]
) -> dict[str, Any]:
    minimum_projects = r0.POWER_POLICY[
        "minimum_independent_projects_per_priority_material_train"
    ]
    result = {}
    for material in priorities:
        selected = [
            parent
            for parent in parents
            if parent["role"] == TRAIN_ROLE
            and parent["project_id"] is not None
            and parent["material_observed"]
            and parent["material_label"] == material
        ]
        projects = sorted({parent["project_id"] for parent in selected})
        result[material] = {
            "independent_project_count": len(projects),
            "independent_project_deficit": max(0, minimum_projects - len(projects)),
            "parent_count": len(selected),
            "project_ids": projects,
        }
    return result


def verify_counts(
    profile: dict[str, Any],
    train_rows: list[dict[str, Any]],
    development_rows: list[dict[str, Any]],
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    parents: list[dict[str, Any]],
    project_statistics: list[dict[str, Any]],
    raw_rows: list[dict[str, Any]],
    centered_rows: list[dict[str, Any]],
    classifier: dict[str, Any],
) -> dict[str, dict[str, int]]:
    train_materials = {
        parent["material_label"] for parent in train if parent["material_observed"]
    }
    train_classes = {
        parent["coarse_material"] for parent in train if parent["material_observed"]
    }
    single = [parent for parent in parents if parent["project_id"] is not None]
    actual = {
        "baseline": {
            "coarse_supported_development_parents": sum(
                parent["coarse_material"] in train_classes for parent in development
            ),
            "development_parents": len(development),
            "development_records": len(development_rows),
            "exact_supported_development_parents": sum(
                parent["material_label"] in train_materials for parent in development
            ),
            "material_unobserved_train_parents": sum(
                not parent["material_observed"] for parent in train
            ),
            "train_parents": len(train),
            "train_records": len(train_rows),
        },
        "audit": {
            "combined_parents": len(parents),
            "cross_source_parents": len(parents) - len(single),
            "leave_project_out_supported_parents": len(raw_rows),
            "material_unobserved_parents": sum(
                not parent["material_observed"] for parent in parents
            ),
            "project_classifier_eligible_parents": classifier["eligible_parent_count"],
            "project_count": len({parent["project_id"] for parent in single}),
            "project_statistics_parents": len(project_statistics),
            "single_source_parents": len(single),
            "source_centered_supported_parents": len(centered_rows),
        },
    }
    if actual != profile["expected_counts"]:
        raise CorrectedBaselineError(f"B0R/R0R input count drift: {actual}")
    return actual


def build_documents(
    profile_bytes: bytes, profile: dict[str, Any], corpus_root: Path
) -> dict[str, bytes]:
    metadata_bytes, _ = validate_corpus_metadata(corpus_root, profile)
    inputs = profile["input_bindings"]
    train_bytes, train_projection = load_projection(
        corpus_root, inputs["train_projection"], TRAIN_ROLE
    )
    development_bytes, development_projection = load_projection(
        corpus_root, inputs["development_projection"], DEVELOPMENT_ROLE
    )
    train_rows = train_projection["rows"]
    development_rows = development_projection["rows"]
    b0.validate_split(train_rows, development_rows)
    taxonomy = profile["material_taxonomy"]
    train, train_feature_bytes = aggregate_parents(train_rows, corpus_root, taxonomy)
    development, development_feature_bytes = aggregate_parents(
        development_rows, corpus_root, taxonomy
    )
    model, metrics, predictions, baseline_summary = fit_corrected_baselines(
        train, development, taxonomy
    )
    mean = np.asarray(model["scaler"]["mean"], dtype=np.float64)
    scale = np.asarray(model["scaler"]["scale"], dtype=np.float64)
    parents = r0.prepare_parents(
        train_rows, development_rows, train, development, mean, scale
    )
    project_statistics = [
        parent
        for parent in parents
        if parent["project_id"] is not None and parent["material_observed"]
    ]
    loo = r0.leave_one_parent_out(project_statistics)
    raw_lpo, raw_rows = r0.evaluate_lpo(
        project_statistics, "standardized_target", require_multi_parent_projects=False
    )
    raw_bootstrap = r0.bootstrap_interval(
        np.asarray([row["relative_improvement"] for row in raw_rows], dtype=np.float64)
    )
    r0.add_source_centered_targets(project_statistics)
    centered_lpo, centered_rows = r0.evaluate_lpo(
        project_statistics, "source_centered_target", require_multi_parent_projects=True
    )
    classifier = r0.project_identity_audit(project_statistics)
    outcome, decision_gates = corrected_decision(
        classifier, raw_lpo, centered_lpo, raw_bootstrap
    )
    counts = verify_counts(
        profile,
        train_rows,
        development_rows,
        train,
        development,
        parents,
        project_statistics,
        raw_rows,
        centered_rows,
        classifier,
    )
    anchor_feature_bytes, anchors = anchor_diagnostics(
        train_rows + development_rows, corpus_root, mean, scale
    )
    variance = {
        "coarse_material_explained_fraction": r0.rounded(
            r0.explained_fraction(
                project_statistics, lambda parent: parent["coarse_material"]
            )
        ),
        "exact_material_explained_fraction": r0.rounded(
            r0.explained_fraction(
                project_statistics, lambda parent: parent["material_label"]
            )
        ),
        "project_explained_fraction": r0.rounded(
            r0.explained_fraction(
                project_statistics, lambda parent: parent["project_id"]
            )
        ),
        "role_explained_fraction": r0.rounded(
            r0.explained_fraction(project_statistics, lambda parent: parent["role"])
        ),
        "warning": "marginal_nonorthogonal_descriptive_fractions_do_not_sum_to_one",
    }
    representation = {
        "feature_groups": {
            name: {"start": start, "stop": stop}
            for name, (start, stop) in b0.metric_groups().items()
        },
        "names": b0.representation_names(),
        "policy": b0.REPRESENTATION_POLICY,
        "schema": REPRESENTATION_SCHEMA,
    }
    metrics_document = {
        "baseline_metrics": metrics,
        "evaluation_policy": b0.EVALUATION_POLICY,
        "schema": METRICS_SCHEMA,
        "summary": baseline_summary,
    }
    predictions_document = {
        "development_parent_count": len(development),
        "evaluation_policy": b0.EVALUATION_POLICY,
        "predictions": predictions,
        "schema": PREDICTION_SCHEMA,
    }
    project_counts = Counter(parent["project_id"] for parent in project_statistics)
    inventory = {
        "cross_source_anchors": anchors,
        "parents": [
            {
                "coarse_material": parent["coarse_material"],
                "family_component_id": parent["family_component_id"],
                "family_ids": parent["family_ids"],
                "material_label": parent["material_label"],
                "material_observed": parent["material_observed"],
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
    old_power = r0.power_plan(raw_rows)
    minimum = old_power["minimum_supported_evaluation_parents"]
    current = old_power["current_supported_parent_count"]
    power = {
        "current_supported_parent_count": current,
        "estimated_relative_delta_stddev": old_power["estimated_relative_delta_stddev"],
        "minimum_supported_evaluation_parents": minimum,
        "new_independent_parent_deficit": max(0, minimum - current),
        "policy": r0.POWER_POLICY,
        "priority_material_train_support": priority_material_support(
            parents, profile["priority_materials"]
        ),
        "schema": POWER_SCHEMA,
        "warning": "normal_approximation_planning_floor_not_admission_evidence",
    }
    access = {
        "anchor_feature_bytes_read": anchor_feature_bytes,
        "candidate_model_bytes_read": 0,
        "corpus_metadata_bytes_read": metadata_bytes,
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
        "access-ledger.json": canonical_json(access_document),
        "baseline-metrics.json": canonical_json(metrics_document),
        "baseline-models.json": canonical_json(model),
        "baseline-predictions.json": canonical_json(predictions_document),
        "domain-audit.json": canonical_json(audit),
        "parent-inventory.json": canonical_json(inventory),
        "power-plan.json": canonical_json(power),
        "profile.json": profile_bytes,
        "representation.json": canonical_json(representation),
    }
    artifact_bindings = {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in sorted(artifacts.items())
    }
    gates = {
        "all_c0r_inputs_hash_verified": True,
        "all_six_baselines_executed": set(metrics) == set(b0.BASELINE_NAMES),
        "corrected_parent_counts_exact": counts == profile["expected_counts"],
        "cross_source_parents_excluded_from_project_statistics": all(
            parent["project_id"] is not None for parent in project_statistics
        ),
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
        "material_unobserved_excluded_from_conditioned_models": (
            model["material_unobserved_train_parent_count"]
            == counts["baseline"]["material_unobserved_train_parents"]
            and UNKNOWN_MATERIAL not in model["conditioning_classes"]
            and UNKNOWN_MATERIAL not in model["material_prototypes"]
        ),
        "no_candidate_training_or_selection": True,
        "train_development_identity_disjoint": True,
    }
    if not all(gates.values()):
        raise CorrectedBaselineError("B0R/R0R conjunctive gate failed")
    report = {
        "access": access,
        "artifacts": artifact_bindings,
        "authority": AUTHORITY,
        "counts": counts,
        "decision": outcome,
        "gates": gates,
        "next_actions": [
            "v44_c1_descriptor_source_growth",
            "v44_v0s_validator_source_search",
        ],
        "result": {
            "baseline_best": baseline_summary["best_baseline"],
            "baseline_global_median_parent_rmse": metrics["global_prototype"][
                "coarse_supported"
            ]["median_parent_rmse"],
            "cross_project_material_median_relative_improvement": raw_lpo[
                "median_relative_improvement"
            ],
            "cross_project_material_paired_bootstrap_95_low": raw_bootstrap["low"],
            "minimum_supported_evaluation_parents": minimum,
            "new_independent_parent_deficit": power["new_independent_parent_deficit"],
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


def prepare_root(path: Path) -> Path:
    if path.is_symlink() or not path.is_dir():
        raise CorrectedBaselineError("C0R root must be a non-symlink directory")
    return path.resolve()


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise CorrectedBaselineError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise CorrectedBaselineError("output must remain outside the repository")
    if resolved.exists():
        raise CorrectedBaselineError(f"refusing to replace existing output: {resolved}")
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
                raise CorrectedBaselineError("B0R/R0R outputs must be flat files")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, corpus_root: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(raw_profile)
    return publish_directory(
        output,
        build_documents(profile_bytes, profile, prepare_root(corpus_root)),
    )


def main() -> None:
    arguments = parse_arguments()
    print(run(arguments.profile, arguments.corpus_root, arguments.output))


if __name__ == "__main__":
    main()
