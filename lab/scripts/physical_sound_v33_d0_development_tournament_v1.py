#!/usr/bin/env python3
"""Run the frozen V33 D0 train/development tournament without holdout access."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import resource
import shutil
import struct
import tempfile
import time
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v33_i0_mode_local_spectral_owner_v1 as i0
import torch

PROFILE_SHA256 = i0.PROFILE_SHA256
I0_OWNER_SHA256 = "8a12af9417feae0e92da6950ffc7b86327902849647e959a1ad01b4e94a6f5b0"
I0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v33-i0-mode-local-spectral-owner-result-2026-09-02.md"
)
I0_RESULT_SHA256 = "8fedab7970b5135136cecc68ff06900d509d71489daa836d2a4b5c16823cd61a"
OWNER_PATH = "lab/scripts/physical_sound_v33_d0_development_tournament_v1.py"
CLAIM = (
    "SYNTHETIC_FRESH_TRAIN_DEVELOPMENT_TOURNAMENT_ONLY / "
    "NO_METHOD_HOLDOUT_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_"
    "COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v33-d0-corpus-manifest.v1"
CONTROL_SCHEMA = "nextengine.experimental-physical-sound-v33-d0-control-report.v1"
EVIDENCE_SCHEMA = "nextengine.experimental-physical-sound-v33-d0-evidence.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v33-d0-report.v1"
FREEZE_SCHEMA = "nextengine.experimental-physical-sound-v33-d0-candidate-freeze.v1"
COMPARE_SCHEMA = "nextengine.experimental-physical-sound-v33-d0-repeat-comparison.v1"
WEIGHTS_MAGIC = b"NEXTV33W\0"
PREDICTIONS_MAGIC = b"NEXTV33P\0"
BRANCHES = i0.BRANCHES
TARGET_INDEX = i0.TARGET_INDEX
ORACLE_SCALES = i0.ORACLE_SCALES
ALLOWED_ROLES = ("train", "development")


class D0TournamentError(RuntimeError):
    """The frozen D0 owner cannot publish a valid terminal decision."""


@dataclass
class RoleData:
    role: str
    row_ids: list[str]
    features: dict[str, np.ndarray]
    raw_features: dict[str, np.ndarray]
    targets: np.ndarray
    cases: list[dict[str, Any]]
    strata: dict[str, np.ndarray]
    case_count: int
    role_local_geometry_groups: int


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise D0TournamentError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    profile, data = i0.load_profile(path)
    if sha256_bytes(data) != PROFILE_SHA256:
        raise D0TournamentError("F0 profile identity drift")
    expected_prefix = [
        "f0-identity-and-static-freeze",
        "i0-discarded-nonofficial-conformance",
        "d0-train-materialize-and-fit",
        "d0-development-materialize-and-gate",
    ]
    if profile["access_order"][:4] != expected_prefix:
        raise D0TournamentError("D0 access order drift")
    return profile, data


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise D0TournamentError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise D0TournamentError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    dependencies = i0.validate_dependencies(profile)
    dependencies["i0_owner"] = validate_bound_file(i0.OWNER_PATH, I0_OWNER_SHA256)
    dependencies["i0_result"] = validate_bound_file(I0_RESULT_PATH, I0_RESULT_SHA256)
    return dependencies


def external_output(path: Path) -> Path:
    try:
        return i0.external_output(path)
    except i0.I0ConformanceError as error:
        raise D0TournamentError(str(error)) from error


def decimal_product(value: str, multiplier: str) -> str:
    return format(Decimal(value) * Decimal(multiplier), "f")


def validate_role_metadata(profile: dict[str, Any]) -> dict[str, Any]:
    corpus = profile["corpus"]
    role_cases: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    result: dict[str, Any] = {}
    for role in ALLOWED_ROLES:
        cases, strata = i0.f0.enumerate_role_cases(corpus, role)
        expected = corpus["counts"][role]
        if len(cases) != expected["cases"]:
            raise D0TournamentError(f"case count mismatch: {role}")
        if len(cases) * corpus["mode_count"] != expected["modal_rows"]:
            raise D0TournamentError(f"modal row count mismatch: {role}")
        if role == "development" and strata != expected["strata"]:
            raise D0TournamentError("development stratum algebra mismatch")
        role_cases[role] = cases
        result[role] = {
            "cases": len(cases),
            "modal_rows": len(cases) * corpus["mode_count"],
            "strata": strata,
        }
    if role_cases["train"] & role_cases["development"]:
        raise D0TournamentError("train/development case identity overlap")
    return result


def base_fixture_for_family(
    family: dict[str, Any], p0_profile: dict[str, Any]
) -> dict[str, Any]:
    return i0.base_fixture_for_family(family, p0_profile)


def build_fixture(
    profile: dict[str, Any],
    p0_profile: dict[str, Any],
    family_index: int,
    material_index: int,
    geometry_cell_index: int,
    contact_set: str,
    contact_index: int,
) -> tuple[dict[str, Any], tuple[float, float, float]]:
    corpus = profile["corpus"]
    family = corpus["families"][family_index]
    fixture = base_fixture_for_family(family, p0_profile)
    fixture["fixture_id"] = (
        f"v33-f{family_index}-m{material_index}-g{geometry_cell_index}-"
        f"{contact_set}-c{contact_index}"
    )
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
    return fixture, tuple(float(value) for value in multiplier_text)


def build_role(
    role: str, profile: dict[str, Any], p0_profile: dict[str, Any]
) -> RoleData:
    if role not in ALLOWED_ROLES:
        raise D0TournamentError(f"D0 role access forbidden: {role}")
    corpus = profile["corpus"]
    h = float(profile["features"]["lift"]["local_stencil_offset"])
    row_ids: list[str] = []
    feature_rows: dict[str, list[np.ndarray]] = {branch: [] for branch in BRANCHES}
    target_rows: list[np.ndarray] = []
    cases: list[dict[str, Any]] = []
    stratum_rows: dict[str, list[int]] = {}
    geometry_groups: set[tuple[int, int, int]] = set()
    case_count = 0
    for role_entry in corpus["role_plan"][role]:
        stratum = role_entry["stratum"]
        if stratum in stratum_rows:
            raise D0TournamentError(f"duplicate role stratum: {role}/{stratum}")
        stratum_rows[stratum] = []
        contact_set = role_entry["contact_set"]
        for family_index, family in enumerate(corpus["families"]):
            for material_index in range(len(corpus["materials"])):
                for geometry_cell_index in role_entry["geometry_cells"]:
                    geometry_groups.add(
                        (family_index, material_index, geometry_cell_index)
                    )
                    for contact_index in range(len(corpus["contacts"][contact_set])):
                        case_id = (
                            f"{role}/{stratum}/f{family_index}/m{material_index}/"
                            f"g{geometry_cell_index:02d}/{contact_set}/c{contact_index:02d}"
                        )
                        fixture, multipliers = build_fixture(
                            profile,
                            p0_profile,
                            family_index,
                            material_index,
                            geometry_cell_index,
                            contact_set,
                            contact_index,
                        )
                        solution = i0.p1.solve_case(case_id, fixture, p0_profile)
                        if (
                            solution["metrics"]["remesh_common_vertices_exact"]
                            is not True
                        ):
                            raise D0TournamentError(f"P1 remesh gate failed: {case_id}")
                        _, stencil = i0.stencil_values(solution, p0_profile, h)
                        start = len(row_ids)
                        modes = solution["modal_document"]["modes"]
                        for ordinal_index, mode in enumerate(modes):
                            row_id = f"{case_id}/o{ordinal_index:02d}"
                            row = i0.normalized_row(
                                profile,
                                fixture,
                                family,
                                mode,
                                multipliers,
                                stencil,
                                ordinal_index,
                            )
                            target = i0.oracle_targets(row)
                            if any(
                                np.any(~np.isfinite(row[branch])) for branch in BRANCHES
                            ) or np.any(~np.isfinite(target)):
                                raise D0TournamentError(f"non-finite row: {row_id}")
                            row_ids.append(row_id)
                            stratum_rows[stratum].append(len(row_ids) - 1)
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
                                "stratum": stratum,
                            }
                        )
                        case_count += 1
    features = {
        branch: np.stack(feature_rows[branch]).astype(np.float64, copy=False)
        for branch in BRANCHES
    }
    raw_features = {
        branch: np.array(values, copy=True) for branch, values in features.items()
    }
    raw_features["contact"] = np.array(features["contact"][:, :12], copy=True)
    targets = np.stack(target_rows).astype(np.float64, copy=False)
    strata = {
        name: np.asarray(indices, dtype=np.int64)
        for name, indices in sorted(stratum_rows.items())
    }
    expected = corpus["counts"][role]
    if (
        case_count != expected["cases"]
        or len(row_ids) != expected["modal_rows"]
        or len(geometry_groups) != expected["role_local_geometry_groups"]
    ):
        raise D0TournamentError(f"role count closure mismatch: {role}")
    if role == "development":
        for stratum, indices in strata.items():
            if len(indices) != expected["strata"][stratum]["modal_rows"]:
                raise D0TournamentError(f"development stratum row mismatch: {stratum}")
    return RoleData(
        role=role,
        row_ids=row_ids,
        features=features,
        raw_features=raw_features,
        targets=targets,
        cases=cases,
        strata=strata,
        case_count=case_count,
        role_local_geometry_groups=len(geometry_groups),
    )


def train_model(
    data: RoleData,
    profile: dict[str, Any],
    contact_input_count: int,
    seed: int,
    raw: bool,
) -> tuple[i0.ResidualModel, dict[str, Any]]:
    features = data.raw_features if raw else data.features
    model = i0.ResidualModel(profile, contact_input_count, seed)
    expected_parameters = (
        profile["controls"]["raw_mlp"]["parameter_count"]
        if raw
        else profile["model"]["parameter_count"]
    )
    if i0.parameter_count(model) != expected_parameters:
        raise D0TournamentError("model parameter count mismatch")
    tensors = {branch: torch.from_numpy(features[branch]) for branch in BRANCHES}
    targets = torch.from_numpy(data.targets)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=float(profile["training"]["learning_rate"]),
        weight_decay=float(profile["training"]["weight_decay"]),
    )
    scale = torch.from_numpy(ORACLE_SCALES)
    initial_loss = math.nan
    final_loss = math.nan
    for step in range(profile["training"]["steps"]):
        optimizer.zero_grad(set_to_none=True)
        predicted = torch.cat(
            [getattr(model, branch)(tensors[branch]) for branch in BRANCHES], dim=1
        )
        loss = torch.mean(((predicted - targets) / scale) ** 2)
        if not torch.isfinite(loss):
            raise D0TournamentError("training produced non-finite loss")
        loss.backward()
        torch.nn.utils.clip_grad_norm_(
            model.parameters(), float(profile["training"]["gradient_norm_clip"])
        )
        optimizer.step()
        value = float(loss.detach())
        if step == 0:
            initial_loss = value
        final_loss = value
    return model, {
        "final_loss": final_loss,
        "initial_loss": initial_loss,
        "seed": seed,
        "steps": profile["training"]["steps"],
    }


def model_predictions(model: i0.ResidualModel, role: RoleData, raw: bool) -> np.ndarray:
    features = role.raw_features if raw else role.features
    result = i0.model_predictions(model, features)
    if result.shape != role.targets.shape:
        raise D0TournamentError(f"prediction shape mismatch: {role.role}")
    return result


def fit_controls(train: RoleData) -> dict[str, dict[str, dict[str, Any]]]:
    result: dict[str, dict[str, dict[str, Any]]] = {
        "raw_ridge": {},
        "spectral_ridge": {},
    }
    for branch in BRANCHES:
        index = TARGET_INDEX[branch]
        for control, features in (
            ("raw_ridge", train.raw_features),
            ("spectral_ridge", train.features),
        ):
            beta, intercept = i0.fit_ridge(features[branch], train.targets[:, index])
            result[control][branch] = {"beta": beta, "intercept": intercept}
    return result


def control_predictions(
    train: RoleData,
    role: RoleData,
    fitted: dict[str, dict[str, dict[str, Any]]],
    raw_mlp_predictions: np.ndarray,
) -> dict[str, np.ndarray]:
    result = {
        "identity": np.zeros_like(role.targets),
        "nearest": np.empty_like(role.targets),
        "raw_mlp": raw_mlp_predictions,
        "raw_ridge": np.empty_like(role.targets),
        "spectral_ridge": np.empty_like(role.targets),
    }
    for branch in BRANCHES:
        index = TARGET_INDEX[branch]
        result["nearest"][:, index] = i0.predict_nearest(
            train.features[branch],
            train.targets[:, index],
            role.features[branch],
            train.row_ids,
        )
        result["raw_ridge"][:, index] = (
            role.raw_features[branch] @ fitted["raw_ridge"][branch]["beta"]
            + fitted["raw_ridge"][branch]["intercept"]
        )
        result["spectral_ridge"][:, index] = (
            role.features[branch] @ fitted["spectral_ridge"][branch]["beta"]
            + fitted["spectral_ridge"][branch]["intercept"]
        )
    if any(np.any(~np.isfinite(values)) for values in result.values()):
        raise D0TournamentError("control prediction is non-finite")
    return result


def rmse(predicted: np.ndarray, expected: np.ndarray) -> float:
    return float(np.sqrt(np.mean((predicted - expected) ** 2, dtype=np.float64)))


def ratio(numerator: float, denominator: float) -> tuple[float, bool]:
    if denominator == 0.0:
        return (0.0, numerator == 0.0)
    value = numerator / denominator
    return (value, math.isfinite(value))


def metric_set(
    predicted: np.ndarray, expected: np.ndarray, strata: dict[str, np.ndarray]
) -> dict[str, Any]:
    branch_rmse = {
        branch: rmse(predicted[:, index], expected[:, index])
        for branch, index in TARGET_INDEX.items()
    }
    aggregate = float(
        np.mean(
            [
                branch_rmse[branch] / ORACLE_SCALES[TARGET_INDEX[branch]]
                for branch in BRANCHES
            ],
            dtype=np.float64,
        )
    )
    contact_strata = {
        name: rmse(
            predicted[indices, TARGET_INDEX["contact"]],
            expected[indices, TARGET_INDEX["contact"]],
        )
        for name, indices in sorted(strata.items())
    }
    return {
        "aggregate_normalized_rmse": aggregate,
        "branch_rmse": branch_rmse,
        "contact_stratum_rmse": contact_strata,
    }


def development_evaluation(
    model: i0.ResidualModel,
    role: RoleData,
    candidate: np.ndarray,
    controls: dict[str, np.ndarray],
    profile: dict[str, Any],
) -> dict[str, Any]:
    candidate_metrics = metric_set(candidate, role.targets, role.strata)
    control_metrics = {
        name: metric_set(values, role.targets, role.strata)
        for name, values in sorted(controls.items())
    }
    gates: dict[str, bool] = {}
    ratios: dict[str, float] = {}
    frozen = profile["gates"]["development"]

    for control, maximum_text in sorted(
        frozen["aggregate_ratio_max_each_control"].items()
    ):
        value, valid = ratio(
            candidate_metrics["aggregate_normalized_rmse"],
            control_metrics[control]["aggregate_normalized_rmse"],
        )
        key = f"aggregate_vs_{control}"
        ratios[key] = value
        gates[key] = valid and value <= float(maximum_text)

    contact_rmse = candidate_metrics["branch_rmse"]["contact"]
    for control, maximum_text in sorted(
        frozen["contact_ratio_max_each_control"].items()
    ):
        value, valid = ratio(
            contact_rmse, control_metrics[control]["branch_rmse"]["contact"]
        )
        key = f"contact_vs_{control}"
        ratios[key] = value
        gates[key] = valid and value <= float(maximum_text)
    gates["absolute_contact_aggregate"] = contact_rmse <= float(
        frozen["absolute_rmse_max"]["contact_aggregate"]
    )

    stratum_controls = ("raw_mlp", "nearest", "spectral_ridge")
    for stratum, candidate_value in sorted(
        candidate_metrics["contact_stratum_rmse"].items()
    ):
        best = min(
            control_metrics[control]["contact_stratum_rmse"][stratum]
            for control in stratum_controls
        )
        value, valid = ratio(candidate_value, best)
        ratios[f"contact_{stratum}_vs_best_control"] = value
        gates[f"contact_{stratum}_vs_best_control"] = valid and value <= float(
            frozen["contact_each_stratum_ratio_max_best_control"]
        )
        gates[f"absolute_contact_{stratum}"] = candidate_value <= float(
            frozen["absolute_rmse_max"]["contact_each_stratum"]
        )

    for branch in ("decay", "global_gain"):
        candidate_value = candidate_metrics["branch_rmse"][branch]
        gates[f"absolute_{branch}"] = candidate_value <= float(
            frozen["absolute_rmse_max"][branch]
        )
        for control in ("raw_ridge", "nearest"):
            value, valid = ratio(
                candidate_value, control_metrics[control]["branch_rmse"][branch]
            )
            key = f"{branch}_vs_{control}"
            ratios[key] = value
            gates[key] = valid and value <= float(
                frozen["decay_global_ratio_max_ridge_and_nearest"]
            )

    contact_ablations: dict[str, np.ndarray] = {}
    for name, column_slice in (
        ("full_lift_zero", slice(12, 32)),
        ("stencil_zero", slice(28, 32)),
        ("contact_context_zero", slice(9, 32)),
    ):
        features = {
            branch: np.array(values, copy=True)
            for branch, values in role.features.items()
        }
        features["contact"][:, column_slice] = 0.0
        contact_ablations[name] = i0.model_predictions(model, features)[:, 2]
    ablation_gates = {
        "full_lift_zero": "full_lift_zero_ablation_worsening_min",
        "stencil_zero": "stencil_zero_ablation_worsening_min",
        "contact_context_zero": "contact_context_zero_ablation_worsening_min",
    }
    for name, predictions in sorted(contact_ablations.items()):
        value, valid = ratio(
            rmse(predictions, role.targets[:, TARGET_INDEX["contact"]]),
            contact_rmse,
        )
        ratios[name] = value
        gates[name] = valid and value >= float(frozen[ablation_gates[name]])

    material_features = {
        branch: np.array(values, copy=True) for branch, values in role.features.items()
    }
    material_features["decay"][:, 4:8] = 0.0
    material_features["global_gain"][:, 4:8] = 0.0
    material_predictions = i0.model_predictions(model, material_features)
    full_material_rmse = float(
        np.mean(
            [
                candidate_metrics["branch_rmse"]["decay"],
                candidate_metrics["branch_rmse"]["global_gain"],
            ],
            dtype=np.float64,
        )
    )
    ablated_material_rmse = float(
        np.mean(
            [
                rmse(material_predictions[:, 0], role.targets[:, 0]),
                rmse(material_predictions[:, 1], role.targets[:, 1]),
            ],
            dtype=np.float64,
        )
    )
    value, valid = ratio(ablated_material_rmse, full_material_rmse)
    ratios["material_zero"] = value
    gates["material_zero"] = valid and value >= float(
        frozen["material_zero_ablation_worsening_min"]
    )

    return {
        "candidate": candidate_metrics,
        "controls": control_metrics,
        "gates": gates,
        "pass": all(gates.values()),
        "ratios": ratios,
    }


def encode_weights(model: i0.ResidualModel) -> bytes:
    state = model.state_dict()
    output = bytearray(WEIGHTS_MAGIC + struct.pack("<II", 1, len(state)))
    for name, tensor in sorted(state.items()):
        name_data = name.encode()
        values = tensor.detach().cpu().numpy().astype("<f8", copy=False)
        output.extend(struct.pack("<I", len(name_data)))
        output.extend(name_data)
        output.extend(struct.pack("<I", values.ndim))
        output.extend(struct.pack(f"<{values.ndim}I", *values.shape))
        output.extend(values.tobytes())
    return bytes(output)


def encode_predictions(row_ids: list[str], predictions: np.ndarray) -> bytes:
    if predictions.shape != (len(row_ids), 3) or np.any(~np.isfinite(predictions)):
        raise D0TournamentError("invalid prediction serialization input")
    output = bytearray(PREDICTIONS_MAGIC + struct.pack("<II", 1, len(row_ids)))
    for row_id, values in zip(row_ids, predictions, strict=True):
        row_data = row_id.encode()
        output.extend(struct.pack("<I", len(row_data)))
        output.extend(row_data)
        output.extend(values.astype("<f8", copy=False).tobytes())
    return bytes(output)


def role_manifest(role: RoleData) -> dict[str, Any]:
    row_bytes = b"".join(
        struct.pack("<I", len(row_id.encode())) + row_id.encode()
        for row_id in role.row_ids
    )
    return {
        "case_count": role.case_count,
        "feature_sha256": {
            branch: sha256_bytes(
                role.features[branch].astype("<f8", copy=False).tobytes()
            )
            for branch in BRANCHES
        },
        "modal_row_count": len(role.row_ids),
        "raw_contact_feature_sha256": sha256_bytes(
            role.raw_features["contact"].astype("<f8", copy=False).tobytes()
        ),
        "role_local_geometry_groups": role.role_local_geometry_groups,
        "row_id_sha256": sha256_bytes(row_bytes),
        "stratum_modal_rows": {
            name: len(indices) for name, indices in sorted(role.strata.items())
        },
        "target_sha256": sha256_bytes(role.targets.astype("<f8", copy=False).tobytes()),
    }


def artifact_ref(name: str, data: bytes) -> dict[str, Any]:
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    profile, profile_data = load_profile(profile_path)
    dependencies = validate_dependencies(profile)
    metadata = validate_role_metadata(profile)
    p0_profile, _ = i0.p1.p0.load_profile(
        repository_root() / profile["parent"]["p0_profile"]["path"]
    )
    output = external_output(output_path)
    i0.configure_torch()

    train = build_role("train", profile, p0_profile)
    candidate, candidate_training = train_model(
        train,
        profile,
        profile["model"]["heads"]["contact"]["input_count"],
        profile["training"]["candidate_seed"],
        raw=False,
    )
    raw_mlp, raw_training = train_model(
        train,
        profile,
        profile["controls"]["raw_mlp"]["contact_input_count"],
        profile["training"]["raw_mlp_control_seed"],
        raw=True,
    )
    fitted_controls = fit_controls(train)

    development = build_role("development", profile, p0_profile)
    candidate_predictions = model_predictions(candidate, development, raw=False)
    raw_mlp_predictions = model_predictions(raw_mlp, development, raw=True)
    controls = control_predictions(
        train, development, fitted_controls, raw_mlp_predictions
    )
    development_result = development_evaluation(
        candidate, development, candidate_predictions, controls, profile
    )
    hard = i0.hard_gate_conformance(
        development, candidate_predictions, p0_profile, profile
    )
    branch_isolation = i0.branch_isolation_conformance(candidate, development)
    forbidden = i0.forbidden_field_conformance(profile)

    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    owner_identity = {
        "bytes": len(owner_data),
        "path": OWNER_PATH,
        "sha256": sha256_bytes(owner_data),
    }
    profile_identity = {
        "bytes": len(profile_data),
        "path": "lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json",
        "sha256": sha256_bytes(profile_data),
    }
    access = {
        "development_target_rows": len(development.row_ids),
        "method_holdout_target_rows": 0,
        "network_requests": 0,
        "official_model_training_steps": 2 * profile["training"]["steps"],
        "official_train_target_rows": len(train.row_ids),
        "protected_signal_values_decoded": 0,
        "real_signal_values_decoded": 0,
    }
    corpus_manifest = {
        "access": access,
        "claim": CLAIM,
        "metadata": metadata,
        "roles": {
            "development": role_manifest(development),
            "train": role_manifest(train),
        },
        "schema": CORPUS_SCHEMA,
    }
    control_report = {
        "claim": CLAIM,
        "development": development_result,
        "method_holdout": None,
        "schema": CONTROL_SCHEMA,
    }
    payloads: dict[str, bytes] = {
        "candidate-weights.bin": encode_weights(candidate),
        "control-report.json": canonical_json(control_report),
        "corpus-manifest.json": canonical_json(corpus_manifest),
        "development-predictions.bin": encode_predictions(
            development.row_ids, candidate_predictions
        ),
        "raw-mlp-control-weights.bin": encode_weights(raw_mlp),
    }

    elapsed = time.monotonic() - started
    rss = peak_rss_bytes()
    resources = profile["resources"]
    resource_flags = {
        "output_within_64_mib": True,
        "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
        "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
    }
    decision = "PASS_DEVELOPMENT"
    if not hard["pass"] or not all(branch_isolation.values()):
        decision = "REJECT_HARD_GATE"
    elif not development_result["pass"]:
        decision = "REJECT_DEVELOPMENT"
    if not all(resource_flags.values()):
        decision = "REJECT_RESOURCE"

    if decision == "PASS_DEVELOPMENT":
        payloads["candidate-freeze.json"] = canonical_json(
            {
                "candidate_weights_sha256": sha256_bytes(
                    payloads["candidate-weights.bin"]
                ),
                "claim": CLAIM,
                "development_predictions_sha256": sha256_bytes(
                    payloads["development-predictions.bin"]
                ),
                "owner_sha256": owner_identity["sha256"],
                "profile_sha256": profile_identity["sha256"],
                "raw_mlp_control_weights_sha256": sha256_bytes(
                    payloads["raw-mlp-control-weights.bin"]
                ),
                "schema": FREEZE_SCHEMA,
                "status": "FrozenAfterDevelopmentPass",
            }
        )

    evidence = {
        "artifacts": {
            name: artifact_ref(name, data) for name, data in sorted(payloads.items())
        },
        "branch_isolation": branch_isolation,
        "claim": CLAIM,
        "dependencies": dependencies,
        "forbidden_fields": forbidden,
        "hard": {
            "development": hard,
            "method_holdout": None,
            "t0_mutation_reason_identities_unchanged": True,
            "v0a_parent_identities_unchanged": True,
        },
        "owner_identity": owner_identity,
        "profile_identity": profile_identity,
        "resources": resource_flags,
        "schema": EVIDENCE_SCHEMA,
        "training": {"candidate": candidate_training, "raw_mlp": raw_training},
    }
    payloads["evidence.json"] = canonical_json(evidence)
    scientific_pass = decision == "PASS_DEVELOPMENT"
    report = {
        "access": access,
        "authored_fallback_required": True,
        "claim": CLAIM,
        "decision": decision,
        "evidence_sha256": sha256_bytes(payloads["evidence.json"]),
        "method_holdout_opened": False,
        "next_authorized_stage": (
            "V33-H0-one-shot-method-holdout"
            if scientific_pass
            else "V33-family-closed-before-holdout"
        ),
        "schema": REPORT_SCHEMA,
        "status": "Pass" if scientific_pass else "Reject",
    }
    payloads["report.json"] = canonical_json(report)
    total_output_bytes = sum(len(data) for data in payloads.values())
    if total_output_bytes > resources["max_output_bytes"]:
        raise D0TournamentError("output resource bound exceeded before publication")
    expected_files = 8 if scientific_pass else 7
    if len(payloads) != expected_files:
        raise D0TournamentError("D0 publication file-count closure mismatch")

    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v33-d0-", dir=output.parent))
    try:
        for name, data in sorted(payloads.items()):
            (staging / name).write_bytes(data)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    print(
        f"d0-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
        f"output_bytes={total_output_bytes}",
        file=os.sys.stderr,
    )
    return report


def compare_runs(path_a: Path, path_b: Path) -> dict[str, Any]:
    if not path_a.is_dir() or not path_b.is_dir():
        raise D0TournamentError("comparison inputs must be artifact directories")
    files_a = sorted(path.name for path in path_a.iterdir() if path.is_file())
    files_b = sorted(path.name for path in path_b.iterdir() if path.is_file())
    exact = files_a == files_b
    artifacts: dict[str, Any] = {}
    for name in sorted(set(files_a) | set(files_b)):
        data_a = (path_a / name).read_bytes() if name in files_a else b""
        data_b = (path_b / name).read_bytes() if name in files_b else b""
        matched = name in files_a and name in files_b and data_a == data_b
        exact &= matched
        artifacts[name] = {
            "a_sha256": sha256_bytes(data_a) if name in files_a else None,
            "b_sha256": sha256_bytes(data_b) if name in files_b else None,
            "matched": matched,
        }
    return {
        "artifacts": artifacts,
        "claim": CLAIM,
        "decision": "REPEAT_EXACT" if exact else "REJECT_NONDETERMINISTIC",
        "schema": COMPARE_SCHEMA,
        "status": "Pass" if exact else "Reject",
    }


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--compare-a", type=Path)
    parser.add_argument("--compare-b", type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        if arguments.compare_a is not None or arguments.compare_b is not None:
            if arguments.compare_a is None or arguments.compare_b is None:
                raise D0TournamentError("both comparison paths are required")
            result = compare_runs(arguments.compare_a, arguments.compare_b)
            print(canonical_json(result).decode(), end="")
            return 0 if result["status"] == "Pass" else 1
        if arguments.profile is None or arguments.output is None:
            raise D0TournamentError("profile and output are required")
        report = run(arguments.profile, arguments.output)
    except Exception as error:  # noqa: BLE001
        print(f"physical-sound-v33-d0: ContractReject: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
