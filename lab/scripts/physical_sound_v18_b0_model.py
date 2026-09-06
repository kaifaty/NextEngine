#!/usr/bin/env python3
"""Truth-blind deterministic degree-two ridge for Physical Sound V18 B0."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any, Iterable

import numpy as np

import physical_sound_v18_b0_common as common


BASE_FEATURE_NAMES = (
    "material:Steel",
    "material:Wood",
    "material:Glass",
    "topology:Plate",
    "topology:Cylinder",
    "topology:Bowl",
    "topology:RolledSheet",
    "support:Free",
    "support:BaseClamped",
    "log_aspect",
    "log_slenderness",
)
POLYNOMIAL_FEATURE_NAMES = (
    ("intercept",)
    + tuple(f"linear:{name}" for name in BASE_FEATURE_NAMES)
    + tuple(
        f"quadratic:{BASE_FEATURE_NAMES[left]}*{BASE_FEATURE_NAMES[right]}"
        for left in range(len(BASE_FEATURE_NAMES))
        for right in range(left, len(BASE_FEATURE_NAMES))
    )
)
TARGET_NAMES = tuple(f"log_frequency_ratio:{mode}" for mode in range(8)) + tuple(
    f"log_damping_multiplier:{mode}" for mode in range(8)
)


@dataclass(frozen=True)
class BaselineInput:
    object_id: str
    role: str
    stratum: str
    cell: int
    material: str
    topology: str
    support: str
    length_m: float
    aspect: float
    slenderness: float
    wall_m: float
    frequency_scale: float
    damping_scale: float

    def record(self) -> dict[str, Any]:
        return {
            "aspect": self.aspect,
            "cell": self.cell,
            "damping_scale": self.damping_scale,
            "frequency_scale": self.frequency_scale,
            "length_m": self.length_m,
            "material": self.material,
            "object_id": self.object_id,
            "role": self.role,
            "slenderness": self.slenderness,
            "stratum": self.stratum,
            "support": self.support,
            "topology": self.topology,
            "wall_m": self.wall_m,
        }


@dataclass(frozen=True)
class TrainingExample:
    inputs: BaselineInput
    frequency_ratio: np.ndarray
    damping_multiplier: np.ndarray

    def record(self) -> dict[str, Any]:
        return {
            "damping_multiplier": self.damping_multiplier,
            "frequency_ratio": self.frequency_ratio,
            "inputs": self.inputs.record(),
        }


@dataclass(frozen=True)
class RidgeArtifact:
    mean: np.ndarray
    scale: np.ndarray
    coefficients: np.ndarray
    training_identity_root: str
    protocol_sha256: str
    implementation_hashes: dict[str, str]

    def metadata(
        self, normalization_sha256: str, coefficients_sha256: str
    ) -> dict[str, Any]:
        return {
            "base_feature_names": BASE_FEATURE_NAMES,
            "coefficients": {
                "dtype": "<f8",
                "file": "coefficients.npy",
                "sha256": coefficients_sha256,
                "shape": [78, 16],
            },
            "implementation_hashes": self.implementation_hashes,
            "normalization": {
                "dtype": "<f8",
                "file": "normalization.npy",
                "row_order": ["mean", "population_standard_deviation"],
                "sha256": normalization_sha256,
                "shape": [2, 11],
            },
            "polynomial_feature_names": POLYNOMIAL_FEATURE_NAMES,
            "protocol_sha256": self.protocol_sha256,
            "regularization": {
                "identity_width": 78,
                "lambda": common.RIDGE_LAMBDA,
                "regularizes_intercept": True,
            },
            "schema": common.MODEL_SCHEMA,
            "target_names": TARGET_NAMES,
            "training_identity_root": self.training_identity_root,
        }


def input_from_row(row: common.BRow) -> BaselineInput:
    return BaselineInput(
        object_id=row.object_id,
        role=row.role,
        stratum=row.stratum,
        cell=row.cell,
        material=row.material,
        topology=row.topology,
        support=row.support,
        length_m=row.length_m,
        aspect=row.aspect,
        slenderness=row.slenderness,
        wall_m=row.wall_m,
        frequency_scale=row.frequency_scale,
        damping_scale=row.damping_scale,
    )


def training_example(row: common.BRow) -> TrainingExample:
    if row.role != "train":
        raise common.B0Error("B0 training example requires a train row")
    return TrainingExample(
        inputs=input_from_row(row),
        frequency_ratio=np.asarray(row.frequencies / row.frequency_scale),
        damping_multiplier=np.asarray(row.damping / row.damping_scale),
    )


def _check_input(value: BaselineInput) -> None:
    if value.material not in common.MATERIAL_ORDER:
        raise common.B0Error(f"unknown B0 material: {value.material}")
    if value.topology not in common.TOPOLOGY_ORDER:
        raise common.B0Error(f"unknown B0 topology: {value.topology}")
    if value.support not in common.SUPPORT_ORDER:
        raise common.B0Error(f"unknown B0 support: {value.support}")
    continuous = np.asarray(
        [
            value.length_m,
            value.aspect,
            value.slenderness,
            value.wall_m,
            value.frequency_scale,
            value.damping_scale,
        ],
        dtype=np.float64,
    )
    if not np.isfinite(continuous).all() or np.any(continuous <= 0.0):
        raise common.B0Error("B0 input continuous values must be finite and positive")


def base_features(value: BaselineInput) -> np.ndarray:
    _check_input(value)
    return np.asarray(
        [float(value.material == item) for item in common.MATERIAL_ORDER]
        + [float(value.topology == item) for item in common.TOPOLOGY_ORDER]
        + [float(value.support == item) for item in common.SUPPORT_ORDER]
        + [math.log(value.aspect), math.log(value.slenderness)],
        dtype=np.float64,
    )


def continuous_ood_features(value: BaselineInput) -> np.ndarray:
    _check_input(value)
    return np.asarray(
        [math.log(value.aspect), math.log(value.slenderness)], dtype=np.float64
    )


def polynomial_features(standardized: np.ndarray) -> np.ndarray:
    standardized = np.asarray(standardized, dtype=np.float64)
    if standardized.shape != (11,) or not np.isfinite(standardized).all():
        raise common.B0Error("B0 standardized feature vector must be finite width 11")
    products = [
        standardized[left] * standardized[right]
        for left in range(11)
        for right in range(left, 11)
    ]
    result = np.asarray([1.0, *standardized, *products], dtype=np.float64)
    if result.shape != (78,) or len(POLYNOMIAL_FEATURE_NAMES) != 78:
        raise common.B0Error("B0 polynomial design order changed")
    return result


def _training_root(examples: Iterable[TrainingExample]) -> str:
    return common.sha256_bytes(
        common.canonical_json([example.record() for example in examples])
    )


def fit_ridge(
    examples: tuple[TrainingExample, ...],
    protocol_sha256: str,
    implementation_hashes: dict[str, str],
) -> RidgeArtifact:
    if len(examples) != 72:
        raise common.B0Error("B0 ridge requires exactly 72 training examples")
    if any(example.inputs.role != "train" for example in examples):
        raise common.B0Error("B0 ridge received a non-training role")
    if len({example.inputs.object_id for example in examples}) != 72:
        raise common.B0Error("B0 ridge training identities are not unique")
    base = np.stack([base_features(example.inputs) for example in examples])
    mean = np.mean(base, axis=0)
    scale = np.std(base, axis=0, ddof=0)
    if (
        mean.shape != (11,)
        or scale.shape != (11,)
        or not np.isfinite(mean).all()
        or not np.isfinite(scale).all()
        or np.any(scale <= 0.0)
    ):
        raise common.B0Error("B0 train-only normalizer is invalid")
    design = np.stack(
        [polynomial_features((features - mean) / scale) for features in base]
    )
    frequency = np.stack(
        [np.asarray(example.frequency_ratio, dtype=np.float64) for example in examples]
    )
    damping = np.stack(
        [
            np.asarray(example.damping_multiplier, dtype=np.float64)
            for example in examples
        ]
    )
    if frequency.shape != (72, 8) or damping.shape != (72, 8):
        raise common.B0Error("B0 training target shape changed")
    if (
        not np.isfinite(frequency).all()
        or not np.isfinite(damping).all()
        or np.any(frequency <= 0.0)
        or np.any(damping <= 0.0)
    ):
        raise common.B0Error("B0 training target is non-finite or non-positive")
    targets = np.concatenate((np.log(frequency), np.log(damping)), axis=1)
    system = design.T @ design + common.RIDGE_LAMBDA * np.eye(78)
    right_hand_side = design.T @ targets
    coefficients = np.linalg.solve(system, right_hand_side)
    if coefficients.shape != (78, 16) or not np.isfinite(coefficients).all():
        raise common.B0Error("B0 ridge solution is invalid")
    return RidgeArtifact(
        mean=mean,
        scale=scale,
        coefficients=coefficients,
        training_identity_root=_training_root(examples),
        protocol_sha256=protocol_sha256,
        implementation_hashes=dict(sorted(implementation_hashes.items())),
    )


def predict(
    artifact: RidgeArtifact, value: BaselineInput
) -> tuple[np.ndarray, np.ndarray]:
    standardized = (base_features(value) - artifact.mean) / artifact.scale
    log_targets = polynomial_features(standardized) @ artifact.coefficients
    if log_targets.shape != (16,) or not np.isfinite(log_targets).all():
        raise common.B0Error("B0 log prediction is invalid")
    frequency = np.exp(log_targets[:8]) * value.frequency_scale
    damping = np.exp(log_targets[8:]) * value.damping_scale
    if not np.isfinite(frequency).all() or not np.isfinite(damping).all():
        raise common.B0Error("B0 prediction is non-finite")
    return frequency, damping


def encode_artifact(artifact: RidgeArtifact) -> dict[str, bytes]:
    normalization = common.array_bytes(np.stack((artifact.mean, artifact.scale)))
    coefficients = common.array_bytes(artifact.coefficients)
    metadata = artifact.metadata(
        common.sha256_bytes(normalization), common.sha256_bytes(coefficients)
    )
    return {
        "coefficients.npy": coefficients,
        "model.json": common.canonical_json(metadata),
        "normalization.npy": normalization,
    }


def decode_artifact(files: dict[str, bytes]) -> RidgeArtifact:
    expected = {"coefficients.npy", "model.json", "normalization.npy"}
    if set(files) != expected:
        raise common.B0Error(f"B0 model artifact file set changed: {sorted(files)}")
    try:
        metadata = json_loads(files["model.json"])
    except (UnicodeDecodeError, ValueError, TypeError) as error:
        raise common.B0Error("invalid B0 model metadata") from error
    if metadata.get("schema") != common.MODEL_SCHEMA:
        raise common.B0Error("B0 model schema changed")
    if metadata.get("protocol_sha256") != common.PROTOCOL_SHA256:
        raise common.B0Error("B0 model protocol hash changed")
    if tuple(metadata.get("base_feature_names", ())) != BASE_FEATURE_NAMES:
        raise common.B0Error("B0 base feature order changed")
    if tuple(metadata.get("polynomial_feature_names", ())) != POLYNOMIAL_FEATURE_NAMES:
        raise common.B0Error("B0 polynomial feature order changed")
    if tuple(metadata.get("target_names", ())) != TARGET_NAMES:
        raise common.B0Error("B0 target order changed")
    normalization_record = metadata.get("normalization", {})
    coefficient_record = metadata.get("coefficients", {})
    if {
        key: value for key, value in normalization_record.items() if key != "sha256"
    } != {
        "dtype": "<f8",
        "file": "normalization.npy",
        "row_order": ["mean", "population_standard_deviation"],
        "shape": [2, 11],
    }:
        raise common.B0Error("B0 normalization metadata changed")
    if {
        key: value for key, value in coefficient_record.items() if key != "sha256"
    } != {
        "dtype": "<f8",
        "file": "coefficients.npy",
        "shape": [78, 16],
    }:
        raise common.B0Error("B0 coefficient metadata changed")
    if common.sha256_bytes(files["normalization.npy"]) != normalization_record.get(
        "sha256"
    ):
        raise common.B0Error("B0 normalization hash mismatch")
    if common.sha256_bytes(files["coefficients.npy"]) != coefficient_record.get(
        "sha256"
    ):
        raise common.B0Error("B0 coefficient hash mismatch")
    normalization = common.array_from_bytes(files["normalization.npy"], (2, 11))
    coefficients = common.array_from_bytes(files["coefficients.npy"], (78, 16))
    if np.any(normalization[1] <= 0.0):
        raise common.B0Error("B0 decoded normalization scale is invalid")
    regularization = metadata.get("regularization", {})
    if regularization != {
        "identity_width": 78,
        "lambda": common.RIDGE_LAMBDA,
        "regularizes_intercept": True,
    }:
        raise common.B0Error("B0 regularization record changed")
    implementation_hashes = metadata.get("implementation_hashes")
    if not isinstance(implementation_hashes, dict):
        raise common.B0Error("B0 implementation hashes are missing")
    if implementation_hashes != common.implementation_hashes():
        raise common.B0Error("B0 implementation hash mismatch")
    training_identity_root = metadata.get("training_identity_root")
    if not isinstance(training_identity_root, str) or len(training_identity_root) != 64:
        raise common.B0Error("B0 training identity root is invalid")
    return RidgeArtifact(
        mean=normalization[0],
        scale=normalization[1],
        coefficients=coefficients,
        training_identity_root=training_identity_root,
        protocol_sha256=metadata["protocol_sha256"],
        implementation_hashes=dict(sorted(implementation_hashes.items())),
    )


def json_loads(value: bytes) -> dict[str, Any]:
    import json

    result = json.loads(value.decode("utf-8"))
    if not isinstance(result, dict):
        raise TypeError("B0 JSON root must be an object")
    return result
