#!/usr/bin/env python3
"""Fit and evaluate the V41 B0 grouped disclosed-corpus baselines."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import tempfile
from collections import defaultdict
from pathlib import Path
from typing import Any

import numpy as np

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v41-b0-profile.v1"
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-role-projection.v1"
FEATURE_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-acoustic-target.v1"
REPRESENTATION_SCHEMA = (
    "nextengine.experimental-physical-sound-v41-b0-representation.v1"
)
MODEL_SCHEMA = "nextengine.experimental-physical-sound-v41-b0-models.v1"
PREDICTION_SCHEMA = "nextengine.experimental-physical-sound-v41-b0-predictions.v1"
METRICS_SCHEMA = "nextengine.experimental-physical-sound-v41-b0-metrics.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v41-b0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v41-b0-report.v1"
DECISION = "B0_GROUPED_BASELINE_SURFACE_FROZEN_M0_AUTHORIZED"

TRAIN_ROLE = "generator_train"
DEVELOPMENT_ROLE = "generator_development"
BASELINE_NAMES = (
    "global_prototype",
    "material_modal_prototype",
    "nearest_local_medoid",
    "pointwise_mlp",
    "retrieval_copy",
    "ridge",
)
MAX_JSON_BYTES = 8 * 1024 * 1024
HASH_LENGTH = 64

AUTHORITY = {
    "admission_authority": False,
    "authorizes_only": ["v41_m0_recipe_model_freeze"],
    "baseline_fit_authority": True,
    "candidate_model_training_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_access_authority": False,
}

REPRESENTATION_POLICY = {
    "band_count": 24,
    "conditioning": "coarse_material_class_one_hot_only",
    "global_features": [
        "log2_bandwidth_hz",
        "log2_centroid_hz",
        "log2_rolloff_95_hz",
        "log_crest_factor",
        "log_decay_t60_seconds",
        "log_flatness",
        "log_peak_offset_ms_plus_0_1",
        "log_temporal_centroid_seconds",
        "zero_crossing_rate",
    ],
    "modal_histogram_bins": 24,
    "modal_max_hz": 18000.0,
    "modal_min_hz": 80.0,
    "parent_aggregation": "masked_arithmetic_mean_per_physical_parent",
    "scaler": "train_parent_mean_std_with_unit_constant_floor",
    "scaler_floor": 1e-06,
    "spectral_band_db_clip": [-160.0, 20.0],
    "target_dimension": 105,
    "transient_bins": 48,
    "transient_db_clip": [-160.0, 20.0],
}

BASELINE_POLICY = {
    "global_prototype": {
        "kind": "masked_train_parent_mean",
    },
    "material_modal_prototype": {
        "fallback": "global_prototype_and_ood",
        "kind": "masked_coarse_material_train_parent_mean",
    },
    "nearest_local_medoid": {
        "distance": "masked_standardized_rmse",
        "fallback": "global_prototype_and_ood",
        "kind": "coarse_material_train_medoid",
    },
    "pointwise_mlp": {
        "adam_beta1": 0.9,
        "adam_beta2": 0.999,
        "adam_epsilon": 1e-08,
        "full_batch_steps": 500,
        "hidden_width": 32,
        "kind": "one_hidden_layer_tanh_full_batch_numpy_float64",
        "learning_rate": 0.01,
        "seed": 4100,
        "weight_decay": 0.0001,
    },
    "retrieval_copy": {
        "fallback": "global_prototype_and_ood",
        "kind": "lexicographic_first_train_parent_within_coarse_material",
    },
    "ridge": {
        "alpha": 1.0,
        "bias_regularized": False,
        "kind": "closed_form_multioutput_coarse_material_ridge",
    },
}

EVALUATION_POLICY = {
    "aggregation": "equal_weight_per_physical_parent",
    "candidate_or_hyperparameter_selection_allowed": False,
    "exact_label_support_reported_separately": True,
    "metric": "masked_train_standardized_rmse",
    "ood_behavior": "FallbackOutOfDomain",
    "quality_scope": "coarse_material_supported_development_parents_only",
    "ranking": "minimum_parent_median_rmse_then_baseline_name",
    "steel_claim": "coarse_metallic_transfer_only_not_steel_specialization",
    "validator_access_allowed": False,
}


class BaselineError(RuntimeError):
    """B0 cannot publish a leakage-free grouped baseline surface."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise BaselineError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise BaselineError(f"{context} must be an array")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise BaselineError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int | None = None) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise BaselineError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or (maximum is not None and size > maximum):
        raise BaselineError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise BaselineError(f"{context} is not valid UTF-8 JSON") from error
    return require_dict(value, context)


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    expected_bytes = binding.get("bytes")
    require_hash(binding.get("sha256"), f"{context} SHA-256")
    if type(expected_bytes) is not int or expected_bytes <= 0:
        raise BaselineError(f"{context} binding byte count is invalid")
    data = read_regular(path, context, MAX_JSON_BYTES)
    if len(data) != expected_bytes or sha256_bytes(data) != binding["sha256"]:
        raise BaselineError(f"{context} bytes or SHA-256 changed")
    return data


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, "B0 profile", MAX_JSON_BYTES)
    value = parse_json(data, "B0 profile")
    if canonical_json(value) != data:
        raise BaselineError("B0 profile must use canonical JSON")
    return data, value


def validate_dependencies(profile: dict[str, Any]) -> None:
    bindings = require_list(profile.get("dependency_bindings"), "dependencies")
    paths = []
    for raw in bindings:
        binding = require_dict(raw, "dependency")
        if set(binding) != {"bytes", "path", "sha256"}:
            raise BaselineError("dependency fields changed")
        path_text = binding["path"]
        if not isinstance(path_text, str) or not path_text:
            raise BaselineError("dependency path is invalid")
        paths.append(path_text)
        check_binding(repository_root() / path_text, binding, f"dependency {path_text}")
    if paths != sorted(set(paths)):
        raise BaselineError("dependency bindings must be path-sorted and unique")


def validate_profile(value: dict[str, Any]) -> dict[str, Any]:
    required = {
        "authority",
        "baseline_policy",
        "dependency_bindings",
        "environment",
        "evaluation_policy",
        "expected_counts",
        "input_bindings",
        "material_taxonomy",
        "profile_id",
        "representation_policy",
        "schema",
    }
    if set(value) != required:
        raise BaselineError("B0 profile fields changed")
    if value["schema"] != PROFILE_SCHEMA or not isinstance(value["profile_id"], str):
        raise BaselineError("unknown B0 profile identity")
    for key, expected in (
        ("authority", AUTHORITY),
        ("baseline_policy", BASELINE_POLICY),
        ("evaluation_policy", EVALUATION_POLICY),
        ("representation_policy", REPRESENTATION_POLICY),
    ):
        if value[key] != expected:
            raise BaselineError(f"B0 {key} changed")
    environment = require_dict(value["environment"], "environment")
    if environment != {"numpy_version": np.__version__}:
        raise BaselineError("B0 NumPy environment changed")
    counts = require_dict(value["expected_counts"], "expected counts")
    expected_count_keys = {
        "coarse_supported_development_parents",
        "development_parents",
        "development_records",
        "exact_supported_development_parents",
        "train_parents",
        "train_records",
    }
    if set(counts) != expected_count_keys or any(
        type(counts[key]) is not int or counts[key] < 0 for key in counts
    ):
        raise BaselineError("B0 expected counts changed")
    taxonomy = require_dict(value["material_taxonomy"], "material taxonomy")
    if not taxonomy or list(taxonomy) != sorted(taxonomy):
        raise BaselineError("material taxonomy must be sorted and non-empty")
    if any(
        not isinstance(key, str) or not isinstance(item, str)
        for key, item in taxonomy.items()
    ):
        raise BaselineError("material taxonomy values must be strings")
    inputs = require_dict(value["input_bindings"], "input bindings")
    if set(inputs) != {
        "corpus_card",
        "corpus_profile",
        "corpus_report",
        "development_projection",
        "train_projection",
    }:
        raise BaselineError("B0 input bindings changed")
    for name, raw in inputs.items():
        binding = require_dict(raw, f"input binding {name}")
        if set(binding) != {"bytes", "path", "sha256"}:
            raise BaselineError("input binding fields changed")
        path = Path(binding["path"])
        if path.is_absolute() or ".." in path.parts:
            raise BaselineError("input binding path escapes the corpus root")
        require_hash(binding["sha256"], f"input binding {name}")
    validate_dependencies(value)
    return value


def rounded(value: float, digits: int = 9) -> float:
    if not math.isfinite(value):
        raise BaselineError("derived baseline value is non-finite")
    result = round(float(value), digits)
    return 0.0 if result == 0 else result


def rounded_array(value: np.ndarray, digits: int = 9) -> list[float]:
    return [rounded(item, digits) for item in value.reshape(-1)]


def safe_feature_path(corpus_root: Path, relative: str) -> Path:
    path = Path(relative)
    if (
        path.is_absolute()
        or ".." in path.parts
        or path.parts[:2]
        != (
            "objects",
            "features",
        )
    ):
        raise BaselineError("feature path escapes the C0 feature namespace")
    resolved = (corpus_root / path).resolve()
    if not resolved.is_relative_to(corpus_root.resolve()):
        raise BaselineError("feature path escapes the corpus root")
    return resolved


def load_projection(
    corpus_root: Path, binding: dict[str, Any], role: str
) -> tuple[bytes, dict[str, Any]]:
    data = check_binding(corpus_root / binding["path"], binding, f"{role} projection")
    projection = parse_json(data, f"{role} projection")
    if canonical_json(projection) != data:
        raise BaselineError("C0 projection must remain canonical")
    if projection.get("schema") != PROJECTION_SCHEMA or projection.get("role") != role:
        raise BaselineError("C0 projection identity changed")
    rows = require_list(projection.get("rows"), "projection rows")
    if projection.get("record_count") != len(rows):
        raise BaselineError("C0 projection record count changed")
    if projection.get("rows_root_sha256") != sha256_bytes(canonical_json(rows)):
        raise BaselineError("C0 projection row root changed")
    if len({row.get("record_id") for row in rows}) != len(rows):
        raise BaselineError("projection record IDs are not unique")
    return data, projection


def load_feature(
    corpus_root: Path, reference: dict[str, Any]
) -> tuple[int, dict[str, Any]]:
    reference = require_dict(reference, "acoustic target reference")
    if set(reference) != {"bytes", "path", "sha256"}:
        raise BaselineError("acoustic target reference fields changed")
    path = safe_feature_path(corpus_root, reference["path"])
    data = check_binding(path, reference, "C0 acoustic target")
    feature = parse_json(data, "C0 acoustic target")
    if canonical_json(feature) != data or feature.get("schema") != FEATURE_SCHEMA:
        raise BaselineError("C0 acoustic target identity or canonical form changed")
    return len(data), feature


def representation_names() -> list[str]:
    names = [f"spectral_band_db_{index:02d}" for index in range(24)]
    names.extend(f"transient_envelope_db_{index:02d}" for index in range(48))
    names.extend(f"modal_histogram_{index:02d}" for index in range(24))
    names.extend(REPRESENTATION_POLICY["global_features"])
    if len(names) != REPRESENTATION_POLICY["target_dimension"]:
        raise BaselineError("target representation dimension drift")
    return names


def feature_vector(feature: dict[str, Any]) -> tuple[np.ndarray, np.ndarray]:
    spectral = require_dict(feature.get("spectral"), "feature spectral")
    time = require_dict(feature.get("time"), "feature time")
    bands = np.asarray(spectral.get("band_energy_db"), dtype=np.float64)
    transient = np.asarray(feature.get("transient_envelope_db"), dtype=np.float64)
    if bands.shape != (24,) or transient.shape != (48,):
        raise BaselineError("C0 band or transient target shape changed")
    bands = np.clip(bands, *REPRESENTATION_POLICY["spectral_band_db_clip"])
    transient = np.clip(transient, *REPRESENTATION_POLICY["transient_db_clip"])

    edges = np.geomspace(
        REPRESENTATION_POLICY["modal_min_hz"],
        REPRESENTATION_POLICY["modal_max_hz"],
        REPRESENTATION_POLICY["modal_histogram_bins"] + 1,
    )
    histogram = np.zeros(
        REPRESENTATION_POLICY["modal_histogram_bins"], dtype=np.float64
    )
    for raw in require_list(feature.get("modes"), "feature modes"):
        mode = require_dict(raw, "feature mode")
        frequency = float(mode["frequency_hz"])
        index = int(np.searchsorted(edges, frequency, side="right") - 1)
        if 0 <= index < len(histogram):
            histogram[index] += 10.0 ** (float(mode["relative_level_db"]) / 20.0)
    total = float(np.sum(histogram))
    if total > 0:
        histogram = np.log1p(100.0 * histogram / total)

    decay = time.get("decay_t20_extrapolated_t60_seconds")
    globals_ = np.array(
        [
            math.log2(max(float(spectral["bandwidth_hz"]), 1e-9)),
            math.log2(max(float(spectral["centroid_hz"]), 1e-9)),
            math.log2(max(float(spectral["rolloff_95_hz"]), 1e-9)),
            math.log(max(float(time["crest_factor"]), 1e-9)),
            math.log(max(float(decay), 1e-9)) if decay is not None else 0.0,
            math.log(max(float(spectral["flatness"]), 1e-12)),
            math.log(max(float(time["peak_offset_ms"]) + 0.1, 1e-9)),
            math.log(max(float(time["temporal_centroid_seconds"]), 1e-9)),
            float(time["zero_crossing_rate"]),
        ],
        dtype=np.float64,
    )
    vector = np.concatenate((bands, transient, histogram, globals_))
    mask = np.ones_like(vector, dtype=bool)
    mask[96 + 4] = decay is not None
    if vector.shape != (REPRESENTATION_POLICY["target_dimension"],):
        raise BaselineError("C0 acoustic target vector dimension changed")
    if not np.all(np.isfinite(vector)):
        raise BaselineError("C0 acoustic target vector is non-finite")
    return vector, mask


def aggregate_parents(
    rows: list[dict[str, Any]],
    corpus_root: Path,
    taxonomy: dict[str, str],
) -> tuple[list[dict[str, Any]], int]:
    grouped: dict[str, list[tuple[dict[str, Any], np.ndarray, np.ndarray]]] = (
        defaultdict(list)
    )
    feature_bytes = 0
    seen_targets: set[str] = set()
    for row in rows:
        material = row.get("material_label")
        if material not in taxonomy:
            raise BaselineError(f"material is absent from taxonomy: {material}")
        reference = row.get("acoustic_target")
        digest = require_dict(reference, "acoustic target").get("sha256")
        if digest not in seen_targets:
            count, feature = load_feature(corpus_root, reference)
            feature_bytes += count
            seen_targets.add(digest)
        else:
            _, feature = load_feature(corpus_root, reference)
        vector, mask = feature_vector(feature)
        grouped[row["physical_parent_id"]].append((row, vector, mask))

    parents = []
    for parent_id in sorted(grouped):
        members = grouped[parent_id]
        materials = {row["material_label"] for row, _, _ in members}
        components = {row["family_component_id"] for row, _, _ in members}
        if len(materials) != 1 or len(components) != 1:
            raise BaselineError("one C0 parent crosses material or component identity")
        vectors = np.stack([vector for _, vector, _ in members])
        masks = np.stack([mask for _, _, mask in members])
        counts = np.sum(masks, axis=0)
        aggregate_mask = counts > 0
        aggregate = np.zeros(vectors.shape[1], dtype=np.float64)
        aggregate[aggregate_mask] = (
            np.sum(np.where(masks, vectors, 0.0), axis=0)[aggregate_mask]
            / counts[aggregate_mask]
        )
        material = next(iter(materials))
        parents.append(
            {
                "coarse_material": taxonomy[material],
                "family_component_id": next(iter(components)),
                "material_label": material,
                "parent_id": parent_id,
                "record_count": len(members),
                "record_ids": sorted(row["record_id"] for row, _, _ in members),
                "target": aggregate,
                "target_mask": aggregate_mask,
            }
        )
    return parents, feature_bytes


def validate_split(
    train_rows: list[dict[str, Any]], development_rows: list[dict[str, Any]]
) -> None:
    train_records = {row["record_id"] for row in train_rows}
    development_records = {row["record_id"] for row in development_rows}
    if train_records & development_records:
        raise BaselineError("record identity crosses train and development")
    train_parents = {row["physical_parent_id"] for row in train_rows}
    development_parents = {row["physical_parent_id"] for row in development_rows}
    if train_parents & development_parents:
        raise BaselineError("physical parent crosses train and development")
    train_components = {row["family_component_id"] for row in train_rows}
    development_components = {row["family_component_id"] for row in development_rows}
    if train_components & development_components:
        raise BaselineError("family component crosses train and development")


def fit_scaler(parents: list[dict[str, Any]]) -> tuple[np.ndarray, np.ndarray]:
    dimension = REPRESENTATION_POLICY["target_dimension"]
    mean = np.zeros(dimension, dtype=np.float64)
    scale = np.ones(dimension, dtype=np.float64)
    for index in range(dimension):
        values = [
            parent["target"][index]
            for parent in parents
            if parent["target_mask"][index]
        ]
        if not values:
            raise BaselineError("train parents do not observe a target dimension")
        mean[index] = float(np.mean(values))
        deviation = float(np.std(values))
        if deviation >= REPRESENTATION_POLICY["scaler_floor"]:
            scale[index] = deviation
    return mean, scale


def standardized(
    parent: dict[str, Any], mean: np.ndarray, scale: np.ndarray
) -> np.ndarray:
    return (parent["target"] - mean) / scale


def masked_mean(values: list[np.ndarray], masks: list[np.ndarray]) -> np.ndarray:
    stacked = np.stack(values)
    mask = np.stack(masks)
    counts = np.sum(mask, axis=0)
    result = np.zeros(stacked.shape[1], dtype=np.float64)
    observed = counts > 0
    result[observed] = (
        np.sum(np.where(mask, stacked, 0.0), axis=0)[observed] / counts[observed]
    )
    return result


def conditioning_classes(taxonomy: dict[str, str]) -> list[str]:
    return sorted(set(taxonomy.values()))


def input_matrix(parents: list[dict[str, Any]], classes: list[str]) -> np.ndarray:
    lookup = {name: index for index, name in enumerate(classes)}
    matrix = np.zeros((len(parents), len(classes)), dtype=np.float64)
    for row, parent in enumerate(parents):
        matrix[row, lookup[parent["coarse_material"]]] = 1.0
    return matrix


def fit_ridge(
    inputs: np.ndarray, targets: np.ndarray, masks: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    design = np.column_stack((np.ones(len(inputs)), inputs))
    weights = np.zeros((design.shape[1], targets.shape[1]), dtype=np.float64)
    alpha = BASELINE_POLICY["ridge"]["alpha"]
    penalty = np.eye(design.shape[1], dtype=np.float64) * alpha
    penalty[0, 0] = 0.0
    for index in range(targets.shape[1]):
        selected = masks[:, index]
        x = design[selected]
        y = targets[selected, index]
        weights[:, index] = np.linalg.solve(x.T @ x + penalty, x.T @ y)
    return weights[0], weights[1:]


def ridge_predict(
    inputs: np.ndarray, bias: np.ndarray, weights: np.ndarray
) -> np.ndarray:
    return inputs @ weights + bias


def fit_mlp(
    inputs: np.ndarray, targets: np.ndarray, masks: np.ndarray
) -> tuple[dict[str, np.ndarray], float]:
    policy = BASELINE_POLICY["pointwise_mlp"]
    rng = np.random.Generator(np.random.PCG64(policy["seed"]))
    width = policy["hidden_width"]
    parameters = {
        "b1": np.zeros(width, dtype=np.float64),
        "b2": np.zeros(targets.shape[1], dtype=np.float64),
        "w1": rng.normal(
            0.0, 1.0 / math.sqrt(inputs.shape[1]), (inputs.shape[1], width)
        ),
        "w2": rng.normal(0.0, 1.0 / math.sqrt(width), (width, targets.shape[1])),
    }
    first = {name: np.zeros_like(value) for name, value in parameters.items()}
    second = {name: np.zeros_like(value) for name, value in parameters.items()}
    beta1 = policy["adam_beta1"]
    beta2 = policy["adam_beta2"]
    epsilon = policy["adam_epsilon"]
    learning_rate = policy["learning_rate"]
    weight_decay = policy["weight_decay"]
    mask_float = masks.astype(np.float64)
    normalizer = float(np.sum(mask_float))
    final_loss = 0.0
    for step in range(1, policy["full_batch_steps"] + 1):
        hidden = np.tanh(inputs @ parameters["w1"] + parameters["b1"])
        prediction = hidden @ parameters["w2"] + parameters["b2"]
        difference = (prediction - targets) * mask_float
        final_loss = float(np.sum(difference * difference) / normalizer)
        output_gradient = 2.0 * difference / normalizer
        gradients = {
            "b2": np.sum(output_gradient, axis=0),
            "w2": hidden.T @ output_gradient + weight_decay * parameters["w2"],
        }
        hidden_gradient = (output_gradient @ parameters["w2"].T) * (
            1.0 - hidden * hidden
        )
        gradients["b1"] = np.sum(hidden_gradient, axis=0)
        gradients["w1"] = inputs.T @ hidden_gradient + weight_decay * parameters["w1"]
        for name in parameters:
            first[name] = beta1 * first[name] + (1.0 - beta1) * gradients[name]
            second[name] = beta2 * second[name] + (1.0 - beta2) * (
                gradients[name] * gradients[name]
            )
            corrected_first = first[name] / (1.0 - beta1**step)
            corrected_second = second[name] / (1.0 - beta2**step)
            parameters[name] -= (
                learning_rate * corrected_first / (np.sqrt(corrected_second) + epsilon)
            )
    return parameters, final_loss


def mlp_predict(inputs: np.ndarray, parameters: dict[str, np.ndarray]) -> np.ndarray:
    hidden = np.tanh(inputs @ parameters["w1"] + parameters["b1"])
    return hidden @ parameters["w2"] + parameters["b2"]


def parent_distance(left: np.ndarray, right: np.ndarray, mask: np.ndarray) -> float:
    if not np.any(mask):
        raise BaselineError("parent distance has no observed dimensions")
    return float(math.sqrt(np.mean((left[mask] - right[mask]) ** 2)))


def metric_groups() -> dict[str, tuple[int, int]]:
    return {
        "global": (96, 105),
        "modal": (72, 96),
        "spectral": (0, 24),
        "transient": (24, 72),
    }


def percentile(values: list[float], quantile: float) -> float:
    return rounded(float(np.percentile(np.asarray(values), quantile, method="linear")))


def summarize_errors(rows: list[dict[str, Any]]) -> dict[str, Any]:
    if not rows:
        raise BaselineError("cannot summarize an empty evaluation stratum")
    total = [row["aggregate_rmse"] for row in rows]
    categories = {
        name: [row["category_rmse"][name] for row in rows] for name in metric_groups()
    }
    by_material = {}
    for material in sorted({row["coarse_material"] for row in rows}):
        selected = [
            row["aggregate_rmse"] for row in rows if row["coarse_material"] == material
        ]
        by_material[material] = {
            "mean_rmse": rounded(float(np.mean(selected))),
            "median_rmse": percentile(selected, 50),
            "parent_count": len(selected),
        }
    by_label = {}
    for label in sorted({row["material_label"] for row in rows}):
        selected = [
            row["aggregate_rmse"] for row in rows if row["material_label"] == label
        ]
        by_label[label] = {
            "mean_rmse": rounded(float(np.mean(selected))),
            "median_rmse": percentile(selected, 50),
            "parent_count": len(selected),
        }
    return {
        "category_mean_rmse": {
            name: rounded(float(np.mean(values))) for name, values in categories.items()
        },
        "maximum_parent_rmse": rounded(max(total)),
        "mean_parent_rmse": rounded(float(np.mean(total))),
        "median_parent_rmse": percentile(total, 50),
        "p90_parent_rmse": percentile(total, 90),
        "parent_count": len(rows),
        "per_coarse_material": by_material,
        "per_material_label": by_label,
    }


def evaluate_predictions(
    development: list[dict[str, Any]],
    predictions: dict[str, np.ndarray],
    mean: np.ndarray,
    scale: np.ndarray,
    train_materials: set[str],
    train_classes: set[str],
) -> tuple[dict[str, Any], dict[str, Any]]:
    detail: dict[str, Any] = {name: [] for name in BASELINE_NAMES}
    metrics = {}
    for name in BASELINE_NAMES:
        rows = []
        for index, parent in enumerate(development):
            truth = standardized(parent, mean, scale)
            mask = parent["target_mask"]
            category_rmse = {}
            for category, (start, stop) in metric_groups().items():
                local_mask = mask[start:stop]
                category_rmse[category] = rounded(
                    parent_distance(
                        truth[start:stop],
                        predictions[name][index, start:stop],
                        local_mask,
                    )
                )
            aggregate = rounded(parent_distance(truth, predictions[name][index], mask))
            row = {
                "aggregate_rmse": aggregate,
                "category_rmse": category_rmse,
                "coarse_material": parent["coarse_material"],
                "coarse_material_supported": parent["coarse_material"] in train_classes,
                "exact_material_supported": parent["material_label"] in train_materials,
                "material_label": parent["material_label"],
                "parent_id": parent["parent_id"],
                "prediction": rounded_array(predictions[name][index]),
            }
            rows.append(row)
        detail[name] = rows
        coarse = [row for row in rows if row["coarse_material_supported"]]
        exact = [row for row in rows if row["exact_material_supported"]]
        metrics[name] = {
            "coarse_supported": summarize_errors(coarse),
            "exact_label_supported": summarize_errors(exact),
            "ood_parent_count": len(rows) - len(coarse),
        }
    return metrics, detail


def fit_baselines(
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    taxonomy: dict[str, str],
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]:
    mean, scale = fit_scaler(train)
    train_targets = np.stack([standardized(parent, mean, scale) for parent in train])
    train_masks = np.stack([parent["target_mask"] for parent in train])
    global_prototype = masked_mean(list(train_targets), list(train_masks))
    classes = conditioning_classes(taxonomy)
    train_inputs = input_matrix(train, classes)
    development_inputs = input_matrix(development, classes)

    by_class: dict[str, list[int]] = defaultdict(list)
    for index, parent in enumerate(train):
        by_class[parent["coarse_material"]].append(index)
    prototypes = {}
    retrieval = {}
    medoids = {}
    for material in sorted(by_class):
        indices = by_class[material]
        prototypes[material] = masked_mean(
            [train_targets[index] for index in indices],
            [train_masks[index] for index in indices],
        )
        lexical = min(indices, key=lambda index: train[index]["parent_id"])
        retrieval[material] = lexical
        medoid = min(
            indices,
            key=lambda index: (
                parent_distance(
                    train_targets[index], prototypes[material], train_masks[index]
                ),
                train[index]["parent_id"],
            ),
        )
        medoids[material] = medoid

    ridge_bias, ridge_weights = fit_ridge(train_inputs, train_targets, train_masks)
    mlp, final_loss = fit_mlp(train_inputs, train_targets, train_masks)
    predictions = {
        "global_prototype": np.repeat(
            global_prototype[np.newaxis, :], len(development), axis=0
        ),
        "ridge": ridge_predict(development_inputs, ridge_bias, ridge_weights),
        "pointwise_mlp": mlp_predict(development_inputs, mlp),
    }
    material_predictions = []
    retrieval_predictions = []
    medoid_predictions = []
    for parent in development:
        material = parent["coarse_material"]
        material_predictions.append(prototypes.get(material, global_prototype))
        retrieval_predictions.append(
            train_targets[retrieval[material]]
            if material in retrieval
            else global_prototype
        )
        medoid_predictions.append(
            train_targets[medoids[material]]
            if material in medoids
            else global_prototype
        )
    predictions["material_modal_prototype"] = np.stack(material_predictions)
    predictions["retrieval_copy"] = np.stack(retrieval_predictions)
    predictions["nearest_local_medoid"] = np.stack(medoid_predictions)

    train_materials = {parent["material_label"] for parent in train}
    train_classes = set(by_class)
    metrics, prediction_detail = evaluate_predictions(
        development,
        predictions,
        mean,
        scale,
        train_materials,
        train_classes,
    )
    ranking = sorted(
        BASELINE_NAMES,
        key=lambda name: (
            metrics[name]["coarse_supported"]["median_parent_rmse"],
            name,
        ),
    )
    model = {
        "conditioning_classes": classes,
        "global_prototype": rounded_array(global_prototype),
        "material_prototypes": {
            key: rounded_array(value) for key, value in sorted(prototypes.items())
        },
        "mlp": {
            "final_train_masked_mse": rounded(final_loss),
            "parameters": {
                name: {
                    "shape": list(value.shape),
                    "values": rounded_array(value),
                }
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
            "bias": rounded_array(ridge_bias),
            "weights": {
                "shape": list(ridge_weights.shape),
                "values": rounded_array(ridge_weights),
            },
        },
        "scaler": {
            "mean": rounded_array(mean),
            "scale": rounded_array(scale),
        },
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


def verify_counts(
    train_rows: list[dict[str, Any]],
    development_rows: list[dict[str, Any]],
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    profile: dict[str, Any],
) -> dict[str, int]:
    counts = profile["expected_counts"]
    train_materials = {parent["material_label"] for parent in train}
    train_classes = {parent["coarse_material"] for parent in train}
    actual = {
        "coarse_supported_development_parents": sum(
            parent["coarse_material"] in train_classes for parent in development
        ),
        "development_parents": len(development),
        "development_records": len(development_rows),
        "exact_supported_development_parents": sum(
            parent["material_label"] in train_materials for parent in development
        ),
        "train_parents": len(train),
        "train_records": len(train_rows),
    }
    if actual != counts:
        raise BaselineError(f"B0 input count drift: {actual}")
    return actual


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    corpus_root: Path,
) -> dict[str, bytes]:
    inputs = profile["input_bindings"]
    metadata_bytes = 0
    for name in ("corpus_card", "corpus_profile", "corpus_report"):
        data = check_binding(
            corpus_root / inputs[name]["path"], inputs[name], f"C0 {name}"
        )
        metadata_bytes += len(data)
    train_bytes, train_projection = load_projection(
        corpus_root, inputs["train_projection"], TRAIN_ROLE
    )
    development_bytes, development_projection = load_projection(
        corpus_root, inputs["development_projection"], DEVELOPMENT_ROLE
    )
    train_rows = train_projection["rows"]
    development_rows = development_projection["rows"]
    validate_split(train_rows, development_rows)
    taxonomy = profile["material_taxonomy"]
    train, train_feature_bytes = aggregate_parents(train_rows, corpus_root, taxonomy)
    development, development_feature_bytes = aggregate_parents(
        development_rows, corpus_root, taxonomy
    )
    counts = verify_counts(train_rows, development_rows, train, development, profile)
    model, metrics, predictions, summary = fit_baselines(train, development, taxonomy)

    representation = {
        "feature_groups": {
            name: {"start": start, "stop": stop}
            for name, (start, stop) in metric_groups().items()
        },
        "names": representation_names(),
        "policy": REPRESENTATION_POLICY,
        "schema": REPRESENTATION_SCHEMA,
    }
    representation_bytes = canonical_json(representation)
    model_bytes = canonical_json(model)
    predictions_document = {
        "development_parent_count": len(development),
        "evaluation_policy": EVALUATION_POLICY,
        "predictions": predictions,
        "schema": PREDICTION_SCHEMA,
    }
    predictions_bytes = canonical_json(predictions_document)
    metrics_document = {
        "baseline_metrics": metrics,
        "evaluation_policy": EVALUATION_POLICY,
        "schema": METRICS_SCHEMA,
        "summary": summary,
    }
    metrics_bytes = canonical_json(metrics_document)
    access = {
        "candidate_model_bytes_read": 0,
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
    access_bytes = canonical_json(access_document)
    artifacts = {
        "access_ledger.json": access_bytes,
        "metrics.json": metrics_bytes,
        "models.json": model_bytes,
        "predictions.json": predictions_bytes,
        "profile.json": profile_bytes,
        "representation.json": representation_bytes,
    }
    artifact_bindings = {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in sorted(artifacts.items())
    }
    gates = {
        "all_c0_feature_hashes_verified": True,
        "all_six_baselines_executed": set(metrics) == set(BASELINE_NAMES),
        "coarse_support_and_ood_partition_complete": (
            counts["coarse_supported_development_parents"]
            + counts["development_parents"]
            - counts["coarse_supported_development_parents"]
            == counts["development_parents"]
        ),
        "development_did_not_select_candidate_or_hyperparameters": True,
        "grouped_parent_weighting": True,
        "train_development_identity_disjoint": True,
        "validator_protected_pcm_access_zero": (
            access["validator_feature_bytes_read"] == 0
            and access["validator_projection_bytes_read"] == 0
            and access["protected_bytes_read"] == 0
            and access["pcm_bytes_read"] == 0
        ),
    }
    if not all(gates.values()):
        raise BaselineError("B0 conjunctive gate failed")
    report = {
        "access": access,
        "artifacts": artifact_bindings,
        "authority": AUTHORITY,
        "counts": counts,
        "decision": DECISION,
        "gates": gates,
        "input_metadata_bytes_read": metadata_bytes,
        "result": summary,
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


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise BaselineError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise BaselineError("output must remain outside the repository")
    if resolved.exists():
        raise BaselineError(f"refusing to replace existing output: {resolved}")
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
                raise BaselineError("B0 outputs must be flat files")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, corpus_root: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(raw_profile)
    if corpus_root.is_symlink() or not corpus_root.is_dir():
        raise BaselineError("C0 corpus root must be a non-symlink directory")
    return publish_directory(
        output, build_documents(profile_bytes, profile, corpus_root.resolve())
    )


def main() -> None:
    arguments = parse_arguments()
    print(run(arguments.profile, arguments.corpus_root, arguments.output))


if __name__ == "__main__":
    main()
