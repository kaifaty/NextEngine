#!/usr/bin/env python3
"""Run V33 I0 conformance on discarded non-official modal fixtures."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
import shutil
import struct
import tempfile
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v31_p1_modal_owner_v1 as p1
import physical_sound_v33_f0_profile_freeze_v1 as f0
import torch
from torch import nn

PROFILE_SHA256 = "4c1869101af3ec2264771e17af5c2e652e5861e36c3a8ab61333e588db88fd52"
F0_OWNER_SHA256 = "b3c8ee4cceac36361b36ce95269388c4cc039dadd87e86bcf3fbc32f0c77bb2e"
F0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v33-f0-fresh-role-spectral-freeze-result-2026-09-02.md"
)
F0_RESULT_SHA256 = "32946b14ce0506d4207038d271a98e9de5131fc8cbc090878a31e6cd3bb5c392"
OWNER_PATH = "lab/scripts/physical_sound_v33_i0_mode_local_spectral_owner_v1.py"
CLAIM = (
    "SYNTHETIC_NONOFFICIAL_MODE_LOCAL_SPECTRAL_CONFORMANCE_ONLY / "
    "NO_OFFICIAL_ROLE_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_"
    "COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
CONFORMANCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v33-i0-owner-conformance.v1"
)
EVIDENCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v33-i0-owner-conformance-evidence.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v33-i0-owner-conformance-report.v1"
)
BRANCHES = ("decay", "global_gain", "contact")
TARGET_INDEX = {branch: index for index, branch in enumerate(BRANCHES)}
ORACLE_SCALES = np.asarray([0.20, 0.16, 0.22], dtype=np.float64)
NONOFFICIAL_MATERIAL = {
    "density_kg_m3": "5100",
    "loss_rate_per_second": "12.5",
    "material_id": "i0-discarded-synthetic",
    "poisson_ratio": "0.29",
    "youngs_modulus_pa": "101000000000",
}
NONOFFICIAL_GEOMETRY_MULTIPLIERS = ("1.03", "0.97", "1.01")
NONOFFICIAL_CONTACTS = (
    ("0", "0.37"),
    ("1", "0.63"),
    ("0.41", "0"),
    ("0.67", "1"),
    ("0.53", "0.51"),
    ("0.25", "0.75"),
)
ZERO_OFFICIAL_ACCESS = {
    "development_target_rows": 0,
    "method_holdout_target_rows": 0,
    "network_requests": 0,
    "official_model_training_steps": 0,
    "official_train_target_rows": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
}
_TORCH_CONFIGURED = False


class I0ConformanceError(RuntimeError):
    """The frozen I0 owner failed before any official role was opened."""


@dataclass
class NonOfficialData:
    row_ids: list[str]
    features: dict[str, np.ndarray]
    raw_features: dict[str, np.ndarray]
    targets: np.ndarray
    cases: list[dict[str, Any]]
    boundary_probes: list[dict[str, Any]]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise I0ConformanceError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise I0ConformanceError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise I0ConformanceError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    profile, data = f0.load_profile(path)
    if sha256_bytes(data) != PROFILE_SHA256:
        raise I0ConformanceError("F0 profile identity drift")
    if profile["access_order"][:2] != [
        "f0-identity-and-static-freeze",
        "i0-discarded-nonofficial-conformance",
    ]:
        raise I0ConformanceError("I0 access order drift")
    if profile["authority"]["model_allowed_after_f0"] is not True:
        raise I0ConformanceError("I0 model conformance is not authorized")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    dependencies = f0.validate_dependencies(profile)
    v32_profile = f0.load_v32_profile(profile)
    f0.validate_freshness_and_counts(profile, v32_profile)
    f0.validate_features_models_and_gates(profile, v32_profile)
    dependencies["f0_owner"] = validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256)
    dependencies["f0_result"] = validate_bound_file(F0_RESULT_PATH, F0_RESULT_SHA256)
    return dependencies


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise I0ConformanceError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise I0ConformanceError("output must be a fresh external path")
    return output


def decimal_product(value: str, multiplier: str) -> str:
    return format(Decimal(value) * Decimal(multiplier), "f")


def validate_nonofficial_isolation(
    profile: dict[str, Any], v32_profile: dict[str, Any]
) -> dict[str, int]:
    f0_materials = {canonical_json(value) for value in profile["corpus"]["materials"]}
    v32_materials = {
        canonical_json(value) for value in v32_profile["corpus"]["materials"]
    }
    material = canonical_json(
        {
            key: value
            for key, value in NONOFFICIAL_MATERIAL.items()
            if key != "material_id"
        }
    )
    normalized_f0_materials = {
        canonical_json(
            {key: value for key, value in row.items() if key != "material_id"}
        )
        for row in profile["corpus"]["materials"]
    }
    normalized_v32_materials = {
        canonical_json(
            {key: value for key, value in row.items() if key != "material_id"}
        )
        for row in v32_profile["corpus"]["materials"]
    }
    if material in normalized_f0_materials or material in normalized_v32_materials:
        raise I0ConformanceError("non-official material collides with an official row")
    if canonical_json(NONOFFICIAL_MATERIAL) in f0_materials | v32_materials:
        raise I0ConformanceError("non-official material identity reused")

    geometry = tuple(NONOFFICIAL_GEOMETRY_MULTIPLIERS)
    old_geometry = {
        tuple(value) for value in v32_profile["corpus"]["geometry_multiplier_cells"]
    }
    new_geometry = {
        tuple(value) for value in profile["corpus"]["geometry_multiplier_cells"]
    }
    if geometry in old_geometry or geometry in new_geometry:
        raise I0ConformanceError("non-official geometry cell reused")

    official_contacts = {
        tuple(value)
        for values in profile["corpus"]["contacts"].values()
        for value in values
    }
    official_contacts |= {tuple(value) for value in v32_profile["corpus"]["contacts"]}
    if set(NONOFFICIAL_CONTACTS) & official_contacts:
        raise I0ConformanceError("non-official contact reused")
    return {
        "contacts": len(NONOFFICIAL_CONTACTS),
        "geometry_cells": 1,
        "materials": 1,
    }


def base_fixture_for_family(
    family: dict[str, Any], p0_profile: dict[str, Any]
) -> dict[str, Any]:
    for fixture in p0_profile["fixtures"]:
        if fixture["family"] == family["family"]:
            result = copy.deepcopy(fixture)
            result["formula_id"] = family["formula_id"]
            result["support"] = family["support"]
            return result
    raise I0ConformanceError(f"missing P0 family fixture: {family['family']}")


def build_nonofficial_fixture(
    profile: dict[str, Any],
    p0_profile: dict[str, Any],
    family_index: int,
    contact_index: int,
) -> tuple[dict[str, Any], tuple[float, float, float]]:
    family = profile["corpus"]["families"][family_index]
    fixture = base_fixture_for_family(family, p0_profile)
    fixture["fixture_id"] = f"i0-discarded-f{family_index}-c{contact_index}"
    fixture["material"] = {
        **NONOFFICIAL_MATERIAL,
        "material_id": "synthetic-elastic-reference",
    }
    for field, multiplier in zip(
        family["geometry_component_order"],
        NONOFFICIAL_GEOMETRY_MULTIPLIERS,
        strict=True,
    ):
        fixture["geometry"][field] = decimal_product(
            fixture["geometry"][field], multiplier
        )
    u, v = NONOFFICIAL_CONTACTS[contact_index]
    fixture["contact"] = {"normal_impulse_ns": "1", "u": u, "v": v}
    fixture["mode_count"] = profile["corpus"]["mode_count"]
    return fixture, tuple(float(value) for value in NONOFFICIAL_GEOMETRY_MULTIPLIERS)


def stencil_coordinates(u: float, v: float, h: float) -> dict[str, tuple[float, float]]:
    return {
        "u_minus": (max(0.0, u - h), v),
        "u_plus": (min(1.0, u + h), v),
        "v_minus": (u, max(0.0, v - h)),
        "v_plus": (u, min(1.0, v + h)),
    }


def stencil_values(
    solution: dict[str, Any], p0_profile: dict[str, Any], h: float
) -> tuple[dict[str, tuple[float, float]], dict[str, np.ndarray]]:
    validated = solution["validated"]
    u = float(validated["contact"]["u"])
    v = float(validated["contact"]["v"])
    coordinates = stencil_coordinates(u, v, h)
    values = {
        name: p1.participation(
            validated, p0_profile, solution["indices"], point[0], point[1]
        )[0]
        for name, point in coordinates.items()
    }
    return coordinates, values


def normalized_row(
    profile: dict[str, Any],
    fixture: dict[str, Any],
    family: dict[str, Any],
    mode: dict[str, Any],
    geometry_multipliers: tuple[float, float, float],
    stencil: dict[str, np.ndarray],
    ordinal_index: int,
) -> dict[str, Any]:
    material = fixture["material"]
    ordinal = 2.0 * float(mode["ordinal"]) / 9.0 - 1.0
    log_frequency = max(
        -1.5, min(1.5, math.log2(float(mode["frequency_hz"]) / 1000.0) / 4.0)
    )
    index_a = 2.0 * (float(mode["family_index_a"]) - 1.0) / 9.0 - 1.0
    index_b = (
        -1.0
        if mode["family_index_b"] == 0
        else 2.0 * (float(mode["family_index_b"]) - 1.0) / 9.0 - 1.0
    )
    mode_features = [ordinal, log_frequency, index_a, index_b]
    material_features = [
        math.log2(float(material["youngs_modulus_pa"]) / 83_000_000_000.0) / 2.0,
        math.log2(float(material["density_kg_m3"]) / 3100.0) / 2.0,
        (float(material["poisson_ratio"]) - 0.275) / 0.05,
        (float(material["loss_rate_per_second"]) - 15.0) / 6.0,
    ]
    is_plate = fixture["family"] == "rectangular_plate"
    family_features = [float(is_plate), float(not is_plate)]
    support_features = [
        float(fixture["support"] == "simply-supported-all-edges"),
        float(fixture["support"] == "cantilever-clamped-u0"),
        float(fixture["support"] == "simply-supported-both-ends"),
    ]
    geometry = [math.log(value) / math.log(1.25) for value in geometry_multipliers]
    u = float(fixture["contact"]["u"])
    v = float(fixture["contact"]["v"])
    base_contact = (
        mode_features
        + family_features
        + support_features
        + [2.0 * u - 1.0, 2.0 * v - 1.0, float(mode["contact_participation"])]
    )
    fourier: list[float] = []
    for order in profile["features"]["lift"]["fourier_orders"]:
        fourier.extend(
            [
                math.sin(order * math.pi * u),
                math.cos(order * math.pi * u),
                math.sin(order * math.pi * v),
                math.cos(order * math.pi * v),
            ]
        )
    stencil_features = [
        float(stencil[name][ordinal_index])
        for name in profile["features"]["lift"]["local_stencil_order"]
    ]
    p1_contact = float(mode["contact_participation"])
    difference_u = stencil_features[1] - stencil_features[0]
    difference_v = stencil_features[3] - stencil_features[2]
    curvature_u = stencil_features[1] + stencil_features[0] - 2.0 * p1_contact
    return {
        "contact": np.asarray(
            base_contact + fourier + stencil_features, dtype=np.float64
        ),
        "decay": np.asarray(
            mode_features + material_features + support_features, dtype=np.float64
        ),
        "global_gain": np.asarray(
            mode_features + material_features + family_features + geometry,
            dtype=np.float64,
        ),
        "oracle_context": {
            "curvature_u": curvature_u,
            "density": material_features[1],
            "difference_u": difference_u,
            "difference_v": difference_v,
            "family": float(family["family_code"]),
            "geometry": geometry,
            "index_a": index_a,
            "log_frequency": log_frequency,
            "loss": material_features[3],
            "ordinal": ordinal,
            "p1_contact": p1_contact,
            "poisson": material_features[2],
            "support": float(family["support_code"]),
            "surface_a": math.sin(math.pi * u) * math.cos(2.0 * math.pi * v),
            "surface_b": math.cos(3.0 * math.pi * u) * math.sin(math.pi * v),
            "u": u,
            "youngs": material_features[0],
        },
    }


def oracle_targets(row: dict[str, Any]) -> np.ndarray:
    context = row["oracle_context"]
    decay = 0.20 * math.tanh(
        0.42 * context["loss"]
        + 0.28 * context["log_frequency"]
        + 0.18 * context["support"] * context["ordinal"]
        + 0.13 * context["youngs"] * context["density"]
        + 0.11 * context["poisson"] * context["index_a"]
    )
    gain = 0.16 * math.tanh(
        0.34 * context["ordinal"] * context["geometry"][0]
        + 0.29 * context["geometry"][2]
        + 0.22 * context["youngs"] * context["ordinal"]
        - 0.18 * context["density"]
        + 0.17 * context["family"] * context["geometry"][1]
        + 0.12 * context["geometry"][0] * context["geometry"][2]
    )
    contact = 0.22 * math.tanh(
        0.26 * context["p1_contact"]
        + 0.18 * context["difference_u"]
        - 0.14 * context["difference_v"]
        + 0.20 * context["surface_a"]
        + 0.17 * context["surface_b"]
        + 0.12 * context["ordinal"] * context["support"]
        + 0.10 * context["family"] * (2.0 * context["u"] - 1.0)
        + 0.08 * context["log_frequency"] * context["curvature_u"]
    )
    return np.asarray([decay, gain, contact], dtype=np.float64)


def build_nonofficial_data(
    profile: dict[str, Any], p0_profile: dict[str, Any]
) -> NonOfficialData:
    h = float(profile["features"]["lift"]["local_stencil_offset"])
    row_ids: list[str] = []
    feature_rows: dict[str, list[np.ndarray]] = {branch: [] for branch in BRANCHES}
    target_rows: list[np.ndarray] = []
    cases: list[dict[str, Any]] = []
    boundary_probes: list[dict[str, Any]] = []
    for family_index, family in enumerate(profile["corpus"]["families"]):
        for contact_index in range(len(NONOFFICIAL_CONTACTS)):
            case_id = f"i0/f{family_index}/c{contact_index}"
            fixture, multipliers = build_nonofficial_fixture(
                profile, p0_profile, family_index, contact_index
            )
            solution = p1.solve_case(case_id, fixture, p0_profile)
            if solution["metrics"]["remesh_common_vertices_exact"] is not True:
                raise I0ConformanceError(f"P1 remesh gate failed: {case_id}")
            coordinates, stencil = stencil_values(solution, p0_profile, h)
            direct = {
                name: p1.participation(
                    solution["validated"],
                    p0_profile,
                    solution["indices"],
                    point[0],
                    point[1],
                )[0]
                for name, point in coordinates.items()
            }
            if any(not np.array_equal(stencil[name], direct[name]) for name in stencil):
                raise I0ConformanceError(f"local stencil owner drift: {case_id}")
            boundary_probes.append(
                {
                    "case_id": case_id,
                    "coordinates": {
                        name: [point[0], point[1]]
                        for name, point in sorted(coordinates.items())
                    },
                    "u_distances": [
                        float(fixture["contact"]["u"]) - coordinates["u_minus"][0],
                        coordinates["u_plus"][0] - float(fixture["contact"]["u"]),
                    ],
                    "v_distances": [
                        float(fixture["contact"]["v"]) - coordinates["v_minus"][1],
                        coordinates["v_plus"][1] - float(fixture["contact"]["v"]),
                    ],
                }
            )
            start = len(row_ids)
            modes = solution["modal_document"]["modes"]
            for ordinal_index, mode in enumerate(modes):
                row_id = f"{case_id}/o{ordinal_index:02d}"
                row = normalized_row(
                    profile,
                    fixture,
                    family,
                    mode,
                    multipliers,
                    stencil,
                    ordinal_index,
                )
                if row["contact"].shape != (32,):
                    raise I0ConformanceError(f"contact feature width drift: {row_id}")
                if row["decay"].shape != (11,) or row["global_gain"].shape != (13,):
                    raise I0ConformanceError(f"inherited feature width drift: {row_id}")
                target = oracle_targets(row)
                if any(np.any(~np.isfinite(row[branch])) for branch in BRANCHES):
                    raise I0ConformanceError(f"non-finite feature: {row_id}")
                if np.any(~np.isfinite(target)):
                    raise I0ConformanceError(f"non-finite target: {row_id}")
                row_ids.append(row_id)
                for branch in BRANCHES:
                    feature_rows[branch].append(row[branch])
                target_rows.append(target)
            cases.append(
                {
                    "case_id": case_id,
                    "end": len(row_ids),
                    "modes": modes,
                    "remesh_common_vertices_exact": True,
                    "start": start,
                }
            )
    features = {
        branch: np.stack(feature_rows[branch]).astype(np.float64, copy=False)
        for branch in BRANCHES
    }
    raw_features = {
        branch: np.array(values, copy=True) for branch, values in features.items()
    }
    raw_features["contact"] = np.array(features["contact"][:, :12], copy=True)
    targets = np.stack(target_rows).astype(np.float64, copy=False)
    if len(cases) != 18 or len(row_ids) != 180 or targets.shape != (180, 3):
        raise I0ConformanceError("non-official fixture count closure mismatch")
    return NonOfficialData(
        row_ids=row_ids,
        features=features,
        raw_features=raw_features,
        targets=targets,
        cases=cases,
        boundary_probes=boundary_probes,
    )


class BoundScale(nn.Module):
    def __init__(self, bound: float) -> None:
        super().__init__()
        self.bound = bound

    def forward(self, values: torch.Tensor) -> torch.Tensor:
        return values * self.bound


class ResidualModel(nn.Module):
    def __init__(
        self, profile: dict[str, Any], contact_input_count: int, seed: int
    ) -> None:
        super().__init__()
        self.decay = self._head(11, 0.25)
        self.global_gain = self._head(13, 0.20)
        self.contact = self._head(contact_input_count, 0.25)
        self._initialize(seed)

    @staticmethod
    def _head(input_count: int, bound: float) -> nn.Sequential:
        return nn.Sequential(
            nn.Linear(input_count, 16, dtype=torch.float64),
            nn.SiLU(),
            nn.Linear(16, 16, dtype=torch.float64),
            nn.SiLU(),
            nn.Linear(16, 1, dtype=torch.float64),
            nn.Tanh(),
            BoundScale(bound),
        )

    def _initialize(self, seed: int) -> None:
        generator = np.random.Generator(np.random.PCG64(seed))
        with torch.no_grad():
            for name, parameter in sorted(self.named_parameters()):
                if name.endswith("bias"):
                    parameter.zero_()
                else:
                    fan_out, fan_in = parameter.shape
                    limit = math.sqrt(6.0 / float(fan_in + fan_out))
                    values = generator.uniform(
                        -limit, limit, size=tuple(parameter.shape)
                    )
                    parameter.copy_(torch.from_numpy(values))


def parameter_count(model: nn.Module) -> int:
    return sum(parameter.numel() for parameter in model.parameters())


def configure_torch() -> None:
    global _TORCH_CONFIGURED
    torch.use_deterministic_algorithms(True)
    if not _TORCH_CONFIGURED:
        torch.set_num_threads(1)
        torch.set_num_interop_threads(1)
        _TORCH_CONFIGURED = True


def model_predictions(
    model: ResidualModel, features: dict[str, np.ndarray]
) -> np.ndarray:
    with torch.no_grad():
        result = np.column_stack(
            [
                getattr(model, branch)(torch.from_numpy(features[branch]))
                .numpy()
                .reshape(-1)
                for branch in BRANCHES
            ]
        )
    if np.any(~np.isfinite(result)):
        raise I0ConformanceError("model prediction is non-finite")
    return result


def train_smoke(
    model: ResidualModel,
    features: dict[str, np.ndarray],
    targets: np.ndarray,
    profile: dict[str, Any],
) -> dict[str, float | int | bool]:
    tensors = {branch: torch.from_numpy(features[branch]) for branch in BRANCHES}
    target_tensor = torch.from_numpy(targets)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=float(profile["training"]["learning_rate"]),
        weight_decay=float(profile["training"]["weight_decay"]),
    )
    scale = torch.from_numpy(ORACLE_SCALES)
    losses: list[float] = []
    for _ in range(3):
        optimizer.zero_grad(set_to_none=True)
        predicted = torch.cat(
            [getattr(model, branch)(tensors[branch]) for branch in BRANCHES], dim=1
        )
        loss = torch.mean(((predicted - target_tensor) / scale) ** 2)
        if not torch.isfinite(loss):
            raise I0ConformanceError("gradient smoke produced non-finite loss")
        loss.backward()
        torch.nn.utils.clip_grad_norm_(
            model.parameters(), float(profile["training"]["gradient_norm_clip"])
        )
        optimizer.step()
        losses.append(float(loss.detach()))
    return {
        "final_loss": losses[-1],
        "initial_loss": losses[0],
        "official_metric": False,
        "steps": len(losses),
        "values_discarded": True,
    }


def fit_ridge(train_x: np.ndarray, train_y: np.ndarray) -> tuple[np.ndarray, float]:
    feature_mean = np.mean(train_x, axis=0, dtype=np.float64)
    target_mean = float(np.mean(train_y, dtype=np.float64))
    centered_x = train_x - feature_mean
    centered_y = train_y - target_mean
    matrix = centered_x.T @ centered_x
    matrix.flat[:: matrix.shape[0] + 1] += 0.000001
    beta = np.linalg.solve(matrix, centered_x.T @ centered_y)
    intercept = target_mean - float(feature_mean @ beta)
    if np.any(~np.isfinite(beta)) or not math.isfinite(intercept):
        raise I0ConformanceError("ridge control is non-finite")
    return beta, intercept


def predict_nearest(
    train_x: np.ndarray,
    train_y: np.ndarray,
    eval_x: np.ndarray,
    train_row_ids: list[str],
) -> np.ndarray:
    result = np.empty(len(eval_x), dtype=np.float64)
    for eval_index, row in enumerate(eval_x):
        delta = train_x - row
        distances = np.einsum("ij,ij->i", delta, delta, dtype=np.float64)
        minimum = float(np.min(distances))
        candidates = np.flatnonzero(distances == minimum)
        winner = min(candidates, key=lambda index: train_row_ids[int(index)])
        result[eval_index] = train_y[int(winner)]
    return result


def control_conformance(data: NonOfficialData) -> dict[str, Any]:
    predictions = {
        "identity": np.zeros_like(data.targets),
        "nearest": np.empty_like(data.targets),
        "raw_ridge": np.empty_like(data.targets),
        "spectral_ridge": np.empty_like(data.targets),
    }
    for branch in BRANCHES:
        index = TARGET_INDEX[branch]
        raw_x = data.raw_features[branch]
        full_x = data.features[branch]
        raw_beta, raw_intercept = fit_ridge(raw_x, data.targets[:, index])
        full_beta, full_intercept = fit_ridge(full_x, data.targets[:, index])
        predictions["raw_ridge"][:, index] = raw_x @ raw_beta + raw_intercept
        predictions["spectral_ridge"][:, index] = full_x @ full_beta + full_intercept
        predictions["nearest"][:, index] = predict_nearest(
            full_x,
            data.targets[:, index],
            full_x,
            data.row_ids,
        )
    if not np.array_equal(predictions["identity"], np.zeros_like(data.targets)):
        raise I0ConformanceError("identity control drift")
    if not np.array_equal(predictions["nearest"], data.targets):
        raise I0ConformanceError("nearest self-control or tie-break drift")
    if any(np.any(~np.isfinite(value)) for value in predictions.values()):
        raise I0ConformanceError("control prediction is non-finite")
    return {
        name: {
            "finite": True,
            "prediction_sha256": sha256_bytes(
                value.astype("<f8", copy=False).tobytes()
            ),
            "shape": list(value.shape),
        }
        for name, value in sorted(predictions.items())
    }


def compose_modes(
    modes: list[dict[str, Any]], corrections: np.ndarray
) -> list[dict[str, float | int]]:
    result: list[dict[str, float | int]] = []
    for mode, correction in zip(modes, corrections, strict=True):
        decay_log = float(np.clip(correction[0], -0.25, 0.25))
        gain_log = float(np.clip(correction[1], -0.20, 0.20))
        contact_log = float(np.clip(correction[2], -0.25, 0.25))
        contact = float(mode["contact_participation"]) * math.exp(contact_log)
        signed_gain = contact * float(mode["pickup_participation"]) * math.exp(gain_log)
        result.append(
            {
                "decay_per_second": float(mode["decay_per_second"])
                * math.exp(decay_log),
                "frequency_hz": mode["frequency_hz"],
                "ordinal": mode["ordinal"],
                "signed_gain": signed_gain,
            }
        )
    return result


def render_corrected_modes(
    modes: list[dict[str, float | int]], p0_profile: dict[str, Any], impulse: float
) -> np.ndarray:
    numeric = p0_profile["numeric_profile"]
    seconds = np.arange(numeric["render_frames"], dtype=np.float64) / float(
        numeric["sample_rate_hz"]
    )
    samples = np.zeros(len(seconds), dtype=np.float64)
    for mode in modes:
        samples += (
            float(mode["signed_gain"])
            * np.exp(-float(mode["decay_per_second"]) * seconds)
            * np.sin(2.0 * math.pi * float(mode["frequency_hz"]) * seconds)
        )
    return impulse * float(numeric["render_scale_per_ns"]) * samples


def hard_gate_conformance(
    data: NonOfficialData,
    predictions: np.ndarray,
    p0_profile: dict[str, Any],
    profile: dict[str, Any],
) -> dict[str, Any]:
    checks = {
        "correction_bounds_exact": True,
        "finite_output": bool(np.all(np.isfinite(predictions))),
        "frequency_and_order_bit_exact": True,
        "impulse_probe_ratios": True,
        "nodal_zero_exact": True,
        "positive_decay": True,
        "remesh_common_contact_exact": True,
        "render_peak_strict_max": True,
        "signed_gain_sign_preserved": True,
    }
    bounds = np.asarray([0.25, 0.20, 0.25], dtype=np.float64)
    checks["correction_bounds_exact"] = bool(np.all(np.abs(predictions) <= bounds))
    maximum_peak = 0.0
    contact_nodal_modes = 0
    for case in data.cases:
        original = case["modes"]
        correction = predictions[case["start"] : case["end"]]
        composed = compose_modes(original, correction)
        checks["remesh_common_contact_exact"] &= bool(
            case["remesh_common_vertices_exact"]
        )
        for before, after in zip(original, composed, strict=True):
            checks["frequency_and_order_bit_exact"] &= (
                struct.pack("<d", float(before["frequency_hz"]))
                == struct.pack("<d", float(after["frequency_hz"]))
                and before["ordinal"] == after["ordinal"]
            )
            checks["positive_decay"] &= float(after["decay_per_second"]) > 0.0
            if float(before["contact_participation"]) == 0.0:
                contact_nodal_modes += 1
                checks["nodal_zero_exact"] &= float(after["signed_gain"]) == 0.0
            if float(before["signed_gain"]) != 0.0:
                checks["signed_gain_sign_preserved"] &= math.copysign(
                    1.0, float(before["signed_gain"])
                ) == math.copysign(1.0, float(after["signed_gain"]))
        unit = render_corrected_modes(composed, p0_profile, 1.0)
        half = render_corrected_modes(composed, p0_profile, 0.5)
        double = render_corrected_modes(composed, p0_profile, 2.0)
        checks["finite_output"] &= bool(np.all(np.isfinite(unit)))
        checks["impulse_probe_ratios"] &= bool(
            np.array_equal(half, unit * 0.5) and np.array_equal(double, unit * 2.0)
        )
        peak = float(np.max(np.abs(unit)))
        maximum_peak = max(maximum_peak, peak)
        checks["render_peak_strict_max"] &= peak < float(
            profile["gates"]["hard"]["render_peak_strict_max"]
        )
    if contact_nodal_modes == 0:
        checks["nodal_zero_exact"] = False
    if not all(checks.values()):
        failed = [name for name, passed in checks.items() if not passed]
        raise I0ConformanceError(f"P1 hard gate failed: {','.join(failed)}")
    return {
        "checks": checks,
        "maximum_render_peak": maximum_peak,
        "contact_nodal_modes": contact_nodal_modes,
        "pass": True,
    }


def branch_isolation_conformance(
    model: ResidualModel, data: NonOfficialData
) -> dict[str, bool]:
    baseline = model_predictions(model, data.features)
    contact_changed = {
        branch: np.array(values, copy=True) for branch, values in data.features.items()
    }
    contact_changed["contact"][:, 12:] = 0.0
    contact_prediction = model_predictions(model, contact_changed)
    material_changed = {
        branch: np.array(values, copy=True) for branch, values in data.features.items()
    }
    material_changed["decay"][:, 4:8] = 0.0
    material_changed["global_gain"][:, 4:8] = 0.0
    material_prediction = model_predictions(model, material_changed)
    result = {
        "contact_change_leaves_decay_exact": np.array_equal(
            baseline[:, 0], contact_prediction[:, 0]
        ),
        "contact_change_leaves_gain_exact": np.array_equal(
            baseline[:, 1], contact_prediction[:, 1]
        ),
        "material_change_leaves_contact_exact": np.array_equal(
            baseline[:, 2], material_prediction[:, 2]
        ),
    }
    if not all(result.values()):
        raise I0ConformanceError("branch isolation failed")
    return result


def boundary_conformance(data: NonOfficialData, h: float) -> dict[str, Any]:
    expected = {
        "i0/f0/c0": {"u_distances": [0.0, h]},
        "i0/f0/c1": {"u_distances": [h, 0.0]},
        "i0/f0/c2": {"v_distances": [0.0, h]},
        "i0/f0/c3": {"v_distances": [h, 0.0]},
    }
    probes = {probe["case_id"]: probe for probe in data.boundary_probes}
    for case_id, fields in expected.items():
        for field, values in fields.items():
            if probes[case_id][field] != values:
                raise I0ConformanceError(f"clamped stencil distance drift: {case_id}")
    return {
        "clamped_edge_probes": 4,
        "direct_p1_sample_comparisons": len(data.boundary_probes) * 4,
        "local_stencil_offset": h,
        "status": "Pass",
    }


def forbidden_field_conformance(profile: dict[str, Any]) -> dict[str, Any]:
    fields = {
        value
        for values in profile["features"]["branch_fields"].values()
        for value in values
    }
    forbidden = set(profile["features"]["forbidden_fields"])
    overlap = sorted(fields & forbidden)
    if overlap:
        raise I0ConformanceError(f"forbidden feature exposed: {overlap}")
    return {"forbidden_count": len(forbidden), "overlap": overlap, "status": "Pass"}


def build_conformance(profile: dict[str, Any], profile_data: bytes) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    v32_profile = f0.load_v32_profile(profile)
    isolation = validate_nonofficial_isolation(profile, v32_profile)
    p0_profile, _ = p1.p0.load_profile(
        repository_root() / profile["parent"]["p0_profile"]["path"]
    )
    data = build_nonofficial_data(profile, p0_profile)
    configure_torch()

    candidate = ResidualModel(
        profile,
        contact_input_count=profile["model"]["heads"]["contact"]["input_count"],
        seed=profile["training"]["candidate_seed"],
    )
    raw_mlp = ResidualModel(
        profile,
        contact_input_count=profile["controls"]["raw_mlp"]["contact_input_count"],
        seed=profile["training"]["raw_mlp_control_seed"],
    )
    if parameter_count(candidate) != profile["model"]["parameter_count"]:
        raise I0ConformanceError("candidate parameter count mismatch")
    if parameter_count(raw_mlp) != profile["controls"]["raw_mlp"]["parameter_count"]:
        raise I0ConformanceError("raw MLP parameter count mismatch")

    candidate_smoke = train_smoke(candidate, data.features, data.targets, profile)
    raw_smoke = train_smoke(raw_mlp, data.raw_features, data.targets, profile)
    candidate_predictions = model_predictions(candidate, data.features)
    raw_predictions = model_predictions(raw_mlp, data.raw_features)
    if candidate_predictions.shape != (180, 3) or raw_predictions.shape != (180, 3):
        raise I0ConformanceError("model prediction shape mismatch")
    controls = control_conformance(data)
    controls["raw_mlp"] = {
        "finite": True,
        "prediction_sha256": sha256_bytes(
            raw_predictions.astype("<f8", copy=False).tobytes()
        ),
        "shape": list(raw_predictions.shape),
    }

    hard = hard_gate_conformance(data, candidate_predictions, p0_profile, profile)
    branch_isolation = branch_isolation_conformance(candidate, data)
    boundary = boundary_conformance(
        data, float(profile["features"]["lift"]["local_stencil_offset"])
    )
    forbidden = forbidden_field_conformance(profile)
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    return {
        "access": {
            **ZERO_OFFICIAL_ACCESS,
            "nonofficial_cases_materialized": len(data.cases),
            "nonofficial_model_training_steps": 6,
            "nonofficial_target_rows_materialized": len(data.row_ids),
        },
        "boundary_stencil": boundary,
        "branch_isolation": branch_isolation,
        "claim": CLAIM,
        "controls": controls,
        "dependencies": dependencies,
        "forbidden_fields": forbidden,
        "hard": hard,
        "model": {
            "candidate_parameter_count": parameter_count(candidate),
            "candidate_prediction_sha256": sha256_bytes(
                candidate_predictions.astype("<f8", copy=False).tobytes()
            ),
            "contact_feature_count": data.features["contact"].shape[1],
            "decay_feature_count": data.features["decay"].shape[1],
            "global_gain_feature_count": data.features["global_gain"].shape[1],
            "raw_contact_feature_count": data.raw_features["contact"].shape[1],
            "raw_mlp_parameter_count": parameter_count(raw_mlp),
        },
        "nonofficial_fixture_isolation": isolation,
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "profile_identity": {
            "bytes": len(profile_data),
            "path": "lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json",
            "sha256": sha256_bytes(profile_data),
        },
        "schema": CONFORMANCE_SCHEMA,
        "smoke": {"candidate": candidate_smoke, "raw_mlp": raw_smoke},
        "status": "Pass",
    }


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    path = root / name
    path.write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v33-i0-", dir=output.parent))
    try:
        conformance = build_conformance(profile, profile_data)
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        evidence = {
            "access": conformance["access"],
            "claim": CLAIM,
            "conformance": conformance_ref,
            "owner_identity": conformance["owner_identity"],
            "profile_identity": conformance["profile_identity"],
            "schema": EVIDENCE_SCHEMA,
            "status": "Pass",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "access": conformance["access"],
            "authored_fallback_required": True,
            "claim": CLAIM,
            "decision": "I0_OWNER_CONFORMANCE_PASS",
            "evidence": evidence_ref,
            "next_authorized_stage": "V33-D0-fresh-development-tournament",
            "official_values_opened": False,
            "schema": REPORT_SCHEMA,
            "status": "Pass",
        }
        write_bytes(staging, "report.json", canonical_json(report))
        files = sorted(path.name for path in staging.iterdir() if path.is_file())
        if files != ["conformance.json", "evidence.json", "report.json"]:
            raise I0ConformanceError("I0 publication closure mismatch")
        total_bytes = sum((staging / name).stat().st_size for name in files)
        if total_bytes > profile["resources"]["max_output_bytes"]:
            raise I0ConformanceError("I0 output resource bound exceeded")
        os.replace(staging, output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.profile, arguments.output)
    except Exception as error:  # noqa: BLE001
        print(f"physical-sound-v33-i0: ContractReject: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
