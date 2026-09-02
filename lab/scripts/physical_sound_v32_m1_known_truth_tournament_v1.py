#!/usr/bin/env python3
"""Run the frozen V32 M1 synthetic known-truth tournament exactly once."""

from __future__ import annotations

import argparse
import copy
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
import torch

import physical_sound_v32_m0_physics_locked_residual_v1 as m0


PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-execution-mechanics-profile.v1"
PROFILE_ID = "physical-sound-v32-m1-execution-mechanics-v1"
PROFILE_SHA256 = "789e222aeb1107335632e85a931347fe4c236e372e7cef20e16b00f8ce895cc0"
CLAIM = (
    "SYNTHETIC_KNOWN_TRUTH_TOURNAMENT_ONLY / NO_REAL_MATERIAL_QUALITY_"
    "VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-tournament-report.v1"
EVIDENCE_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-tournament-evidence.v1"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-corpus-manifest.v1"
CONTROL_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-control-report.v1"
FREEZE_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-candidate-freeze.v1"
HOLDOUT_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-method-holdout-report.v1"
COMPARE_SCHEMA = "nextengine.experimental-physical-sound-v32-m1-repeat-comparison.v1"
OWNER_PATH = "lab/scripts/physical_sound_v32_m1_known_truth_tournament_v1.py"
MAX_PROFILE_BYTES = 256 * 1024
WEIGHTS_MAGIC = b"NEXTM1W\0"
PREDICTIONS_MAGIC = b"NEXTM1P\0"
BRANCHES = ("decay", "global_gain", "contact")
TARGET_INDEX = {branch: index for index, branch in enumerate(BRANCHES)}
ORACLE_SCALES = np.asarray([0.20, 0.16, 0.22], dtype=np.float64)
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "external_research_only": True,
    "model_allowed": True,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "validator_release_authority": False,
}


class M1TournamentError(RuntimeError):
    """The frozen M1 owner cannot produce a valid terminal decision."""


@dataclass
class RoleData:
    role: str
    row_ids: list[str]
    features: dict[str, np.ndarray]
    targets: np.ndarray
    cases: list[dict[str, Any]]
    case_count: int
    object_group_count: int


def canonical_json(value: Any) -> bytes:
    try:
        return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()
    except (TypeError, ValueError) as error:
        raise M1TournamentError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_execution_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise M1TournamentError("profile must not be a symlink")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise M1TournamentError("profile size outside bound")
    try:
        profile = json.loads(data)
    except json.JSONDecodeError as error:
        raise M1TournamentError(f"invalid profile JSON: {error}") from error
    if data != canonical_json(profile):
        raise M1TournamentError("profile is not canonical JSON")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise M1TournamentError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
        or profile.get("authority") != AUTHORITY
    ):
        raise M1TournamentError("profile identity or authority mismatch")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    root = repository_root()
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, declared in sorted(profile["dependencies"].items()):
        data = (root / declared["path"]).read_bytes()
        if sha256_bytes(data) != declared["sha256"]:
            raise M1TournamentError(f"dependency drift: {dependency_id}")
        result[dependency_id] = {"bytes": len(data), **declared}
    protocol = profile["protocol"]
    protocol_data = (root / protocol["path"]).read_bytes()
    if sha256_bytes(protocol_data) != protocol["sha256"]:
        raise M1TournamentError("dependency drift: protocol")
    result["m1_protocol"] = {"bytes": len(protocol_data), **protocol}
    return result


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise M1TournamentError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise M1TournamentError("output must be a fresh external path")
    return output


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
    raise M1TournamentError(f"missing P0 family fixture: {family['family']}")


def build_fixture(
    m0_profile: dict[str, Any],
    p0_profile: dict[str, Any],
    family_index: int,
    material_index: int,
    geometry_cell_index: int,
    contact_index: int,
) -> tuple[dict[str, Any], tuple[float, float, float]]:
    corpus = m0_profile["corpus"]
    family = corpus["families"][family_index]
    fixture = base_fixture_for_family(family, p0_profile)
    material = corpus["materials"][material_index]
    fixture["fixture_id"] = (
        f"m1-f{family_index}-m{material_index}-g{geometry_cell_index}-c{contact_index}"
    )
    fixture["material"] = {
        "density_kg_m3": material["density_kg_m3"],
        "loss_rate_per_second": material["loss_rate_per_second"],
        "material_id": "synthetic-elastic-reference",
        "poisson_ratio": material["poisson_ratio"],
        "youngs_modulus_pa": material["youngs_modulus_pa"],
    }
    multipliers_raw = corpus["geometry_multiplier_cells"][geometry_cell_index]
    for field, multiplier in zip(
        family["geometry_component_order"], multipliers_raw, strict=True
    ):
        fixture["geometry"][field] = decimal_product(
            fixture["geometry"][field], multiplier
        )
    u, v = corpus["contacts"][contact_index]
    fixture["contact"] = {
        "normal_impulse_ns": corpus["impulse_ns"],
        "u": u,
        "v": v,
    }
    fixture["mode_count"] = corpus["mode_count"]
    multipliers = tuple(float(value) for value in multipliers_raw)
    return fixture, multipliers  # type: ignore[return-value]


def build_role(
    role: str, m0_profile: dict[str, Any], p0_profile: dict[str, Any]
) -> RoleData:
    corpus = m0_profile["corpus"]
    if role not in corpus["role_geometry_cells"]:
        raise M1TournamentError(f"unknown corpus role: {role}")
    row_ids: list[str] = []
    feature_rows: dict[str, list[np.ndarray]] = {branch: [] for branch in BRANCHES}
    target_rows: list[np.ndarray] = []
    cases: list[dict[str, Any]] = []
    case_count = 0
    object_groups: set[str] = set()
    for family_index in range(len(corpus["families"])):
        for material_index in range(len(corpus["materials"])):
            for geometry_cell_index in corpus["role_geometry_cells"][role]:
                group_id = f"{role}/f{family_index}/m{material_index}/g{geometry_cell_index}"
                object_groups.add(group_id)
                for contact_index in range(len(corpus["contacts"])):
                    case_id = f"{group_id}/c{contact_index}"
                    fixture, multipliers = build_fixture(
                        m0_profile,
                        p0_profile,
                        family_index,
                        material_index,
                        geometry_cell_index,
                        contact_index,
                    )
                    solution = m0.p1.solve_case(case_id, fixture, p0_profile)
                    if solution["metrics"]["remesh_common_vertices_exact"] is not True:
                        raise M1TournamentError(f"P1 remesh gate failed: {case_id}")
                    modes = solution["modal_document"]["modes"]
                    start = len(row_ids)
                    for ordinal, mode in enumerate(modes):
                        row_id = f"{case_id}/o{ordinal}"
                        row = m0.normalized_row(fixture, mode, multipliers)
                        target = m0.oracle_targets(row)
                        if any(np.any(~np.isfinite(row[branch])) for branch in BRANCHES):
                            raise M1TournamentError(f"non-finite feature row: {row_id}")
                        if np.any(~np.isfinite(target)):
                            raise M1TournamentError(f"non-finite target row: {row_id}")
                        row_ids.append(row_id)
                        for branch in BRANCHES:
                            feature_rows[branch].append(row[branch])
                        target_rows.append(target)
                    cases.append(
                        {
                            "case_id": case_id,
                            "end": len(row_ids),
                            "modes": modes,
                            "start": start,
                        }
                    )
                    case_count += 1
    features = {
        branch: np.stack(feature_rows[branch]).astype(np.float64, copy=False)
        for branch in BRANCHES
    }
    targets = np.stack(target_rows).astype(np.float64, copy=False)
    expected = corpus["counts"][role]
    if (
        case_count != expected["cases"]
        or len(row_ids) != expected["modal_rows"]
        or len(object_groups) != expected["object_groups"]
    ):
        raise M1TournamentError(f"role count mismatch: {role}")
    return RoleData(
        role=role,
        row_ids=row_ids,
        features=features,
        targets=targets,
        cases=cases,
        case_count=case_count,
        object_group_count=len(object_groups),
    )


def configure_torch() -> None:
    torch.use_deterministic_algorithms(True)
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)


def train_candidate(
    train: RoleData, m0_profile: dict[str, Any]
) -> tuple[m0.ResidualModel, dict[str, float]]:
    configure_torch()
    model = m0.ResidualModel(m0_profile)
    if m0.parameter_count(model) != m0_profile["model"]["parameter_count"]:
        raise M1TournamentError("candidate parameter count mismatch")
    tensors = {
        branch: torch.from_numpy(train.features[branch]) for branch in BRANCHES
    }
    targets = torch.from_numpy(train.targets)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=float(m0_profile["training"]["learning_rate"]),
        weight_decay=float(m0_profile["training"]["weight_decay"]),
    )
    scale = torch.from_numpy(ORACLE_SCALES)
    initial_loss = math.nan
    final_loss = math.nan
    for step in range(m0_profile["training"]["steps"]):
        optimizer.zero_grad(set_to_none=True)
        predicted = torch.cat(
            [getattr(model, branch)(tensors[branch]) for branch in BRANCHES], dim=1
        )
        loss = torch.mean(((predicted - targets) / scale) ** 2)
        if not torch.isfinite(loss):
            raise M1TournamentError("candidate training produced non-finite loss")
        loss.backward()
        torch.nn.utils.clip_grad_norm_(
            model.parameters(), float(m0_profile["training"]["gradient_norm_clip"])
        )
        optimizer.step()
        value = float(loss.detach())
        if step == 0:
            initial_loss = value
        final_loss = value
    return model, {"final_loss": final_loss, "initial_loss": initial_loss}


def candidate_predictions(model: m0.ResidualModel, role: RoleData) -> np.ndarray:
    with torch.no_grad():
        result = np.column_stack(
            [
                getattr(model, branch)(torch.from_numpy(role.features[branch]))
                .numpy()
                .reshape(-1)
                for branch in BRANCHES
            ]
        )
    if result.shape != role.targets.shape or np.any(~np.isfinite(result)):
        raise M1TournamentError(f"invalid candidate predictions: {role.role}")
    return result


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
        raise M1TournamentError("ridge fit is non-finite")
    return beta, intercept


def predict_nearest(
    train_x: np.ndarray, train_y: np.ndarray, eval_x: np.ndarray
) -> np.ndarray:
    result = np.empty(len(eval_x), dtype=np.float64)
    for index, row in enumerate(eval_x):
        delta = train_x - row
        distances = np.einsum("ij,ij->i", delta, delta, dtype=np.float64)
        result[index] = train_y[int(np.argmin(distances))]
    return result


def fit_controls(train: RoleData) -> dict[str, dict[str, Any]]:
    controls: dict[str, dict[str, Any]] = {}
    for branch in BRANCHES:
        target = train.targets[:, TARGET_INDEX[branch]]
        beta, intercept = fit_ridge(train.features[branch], target)
        controls[branch] = {"beta": beta, "intercept": intercept}
    return controls


def control_predictions(
    train: RoleData, role: RoleData, fitted: dict[str, dict[str, Any]]
) -> dict[str, np.ndarray]:
    result = {
        "identity": np.zeros_like(role.targets),
        "nearest": np.empty_like(role.targets),
        "ridge": np.empty_like(role.targets),
    }
    for branch in BRANCHES:
        index = TARGET_INDEX[branch]
        result["nearest"][:, index] = predict_nearest(
            train.features[branch], train.targets[:, index], role.features[branch]
        )
        result["ridge"][:, index] = (
            role.features[branch] @ fitted[branch]["beta"]
            + fitted[branch]["intercept"]
        )
    return result


def rmse(predicted: np.ndarray, expected: np.ndarray) -> float:
    return float(np.sqrt(np.mean((predicted - expected) ** 2, dtype=np.float64)))


def metric_set(predicted: np.ndarray, expected: np.ndarray) -> dict[str, Any]:
    branches = {
        branch: rmse(predicted[:, index], expected[:, index])
        for branch, index in TARGET_INDEX.items()
    }
    aggregate = float(
        np.mean(
            [branches[branch] / ORACLE_SCALES[TARGET_INDEX[branch]] for branch in BRANCHES],
            dtype=np.float64,
        )
    )
    return {"aggregate_normalized_rmse": aggregate, "branch_rmse": branches}


def ratio(numerator: float, denominator: float) -> tuple[float, bool]:
    if denominator == 0.0:
        return (0.0, numerator == 0.0)
    value = numerator / denominator
    return (value, math.isfinite(value))


def development_evaluation(
    model: m0.ResidualModel,
    role: RoleData,
    candidate: np.ndarray,
    controls: dict[str, np.ndarray],
    m0_profile: dict[str, Any],
) -> dict[str, Any]:
    candidate_metrics = metric_set(candidate, role.targets)
    control_metrics = {
        control: metric_set(predicted, role.targets)
        for control, predicted in sorted(controls.items())
    }
    gates: dict[str, bool] = {}
    ratios: dict[str, float] = {}
    dev_gates = m0_profile["gates"]["development"]
    for control, metrics in sorted(control_metrics.items()):
        value, valid = ratio(
            candidate_metrics["aggregate_normalized_rmse"],
            metrics["aggregate_normalized_rmse"],
        )
        ratios[f"aggregate_vs_{control}"] = value
        gates[f"aggregate_vs_{control}"] = valid and value <= float(
            dev_gates["aggregate_ratio_max_each_control"]
        )
    for branch in BRANCHES:
        candidate_rmse = candidate_metrics["branch_rmse"][branch]
        gates[f"absolute_{branch}"] = candidate_rmse <= float(
            dev_gates["absolute_rmse_max"][branch]
        )
        for control in ("ridge", "nearest"):
            value, valid = ratio(
                candidate_rmse, control_metrics[control]["branch_rmse"][branch]
            )
            ratios[f"{branch}_vs_{control}"] = value
            gates[f"{branch}_vs_{control}"] = valid and value <= float(
                dev_gates["branch_ratio_max_ridge_and_nearest"]
            )
    material_inputs = {
        branch: np.array(role.features[branch], copy=True)
        for branch in ("decay", "global_gain")
    }
    for inputs in material_inputs.values():
        inputs[:, 4:8] = 0.0
    with torch.no_grad():
        material_rmse = []
        for branch in ("decay", "global_gain"):
            predicted = getattr(model, branch)(torch.from_numpy(material_inputs[branch]))
            material_rmse.append(
                rmse(
                    predicted.numpy().reshape(-1),
                    role.targets[:, TARGET_INDEX[branch]],
                )
            )
        contact_input = np.array(role.features["contact"], copy=True)
        contact_input[:, 9:12] = 0.0
        contact_prediction = model.contact(torch.from_numpy(contact_input)).numpy().reshape(-1)
    full_material_rmse = np.mean(
        [
            candidate_metrics["branch_rmse"]["decay"],
            candidate_metrics["branch_rmse"]["global_gain"],
        ],
        dtype=np.float64,
    )
    material_ratio, material_valid = ratio(
        float(np.mean(material_rmse, dtype=np.float64)), float(full_material_rmse)
    )
    contact_ratio, contact_valid = ratio(
        rmse(contact_prediction, role.targets[:, TARGET_INDEX["contact"]]),
        candidate_metrics["branch_rmse"]["contact"],
    )
    ratios["material_zero_ablation"] = material_ratio
    ratios["contact_zero_ablation"] = contact_ratio
    gates["material_zero_ablation"] = material_valid and material_ratio >= float(
        dev_gates["material_zero_ablation_worsening_min"]
    )
    gates["contact_zero_ablation"] = contact_valid and contact_ratio >= float(
        dev_gates["contact_zero_ablation_worsening_min"]
    )
    return {
        "candidate": candidate_metrics,
        "controls": control_metrics,
        "gates": gates,
        "pass": all(gates.values()),
        "ratios": ratios,
    }


def holdout_evaluation(
    role: RoleData,
    candidate: np.ndarray,
    controls: dict[str, np.ndarray],
    m0_profile: dict[str, Any],
) -> dict[str, Any]:
    candidate_metrics = metric_set(candidate, role.targets)
    control_metrics = {
        control: metric_set(predicted, role.targets)
        for control, predicted in sorted(controls.items())
    }
    gates: dict[str, bool] = {}
    ratios: dict[str, float] = {}
    holdout_gates = m0_profile["gates"]["method_holdout"]
    best_aggregate = min(
        metrics["aggregate_normalized_rmse"] for metrics in control_metrics.values()
    )
    value, valid = ratio(candidate_metrics["aggregate_normalized_rmse"], best_aggregate)
    ratios["aggregate_vs_best_control"] = value
    gates["aggregate_vs_best_control"] = valid and value <= float(
        holdout_gates["aggregate_ratio_max_best_control"]
    )
    for branch in BRANCHES:
        best = min(metrics["branch_rmse"][branch] for metrics in control_metrics.values())
        value, valid = ratio(candidate_metrics["branch_rmse"][branch], best)
        ratios[f"{branch}_vs_best_control"] = value
        gates[f"{branch}_vs_best_control"] = valid and value <= float(
            holdout_gates["branch_ratio_max_best_control"]
        )
    return {
        "candidate": candidate_metrics,
        "controls": control_metrics,
        "gates": gates,
        "pass": all(gates.values()),
        "ratios": ratios,
    }


def render_corrected_modes(
    modes: list[dict[str, float | int]], p0_profile: dict[str, Any], impulse: float
) -> np.ndarray:
    numeric = p0_profile["numeric_profile"]
    seconds = np.arange(numeric["render_frames"], dtype=np.float64) / float(
        numeric["sample_rate_hz"]
    )
    samples = np.zeros(len(seconds), dtype=np.float64)
    for mode in modes:
        samples += float(mode["signed_gain"]) * np.exp(
            -float(mode["decay_per_second"]) * seconds
        ) * np.sin(2.0 * math.pi * float(mode["frequency_hz"]) * seconds)
    return impulse * float(numeric["render_scale_per_ns"]) * samples


def hard_gate_evaluation(
    role: RoleData,
    predictions: np.ndarray,
    p0_profile: dict[str, Any],
    m0_profile: dict[str, Any],
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
    maximum_peak = 0.0
    bounds = np.asarray([0.25, 0.20, 0.25], dtype=np.float64)
    checks["correction_bounds_exact"] = bool(np.all(np.abs(predictions) <= bounds))
    for case in role.cases:
        original = case["modes"]
        correction = predictions[case["start"] : case["end"]]
        composed = m0.compose_modes(original, correction)
        for before, after in zip(original, composed, strict=True):
            checks["frequency_and_order_bit_exact"] &= (
                struct.pack("<d", float(before["frequency_hz"]))
                == struct.pack("<d", float(after["frequency_hz"]))
                and before["ordinal"] == after["ordinal"]
            )
            checks["positive_decay"] &= float(after["decay_per_second"]) > 0.0
            if float(before["signed_gain"]) == 0.0:
                checks["nodal_zero_exact"] &= float(after["signed_gain"]) == 0.0
            else:
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
            m0_profile["gates"]["hard"]["render_peak_strict_max"]
        )
    return {"checks": checks, "maximum_render_peak": maximum_peak, "pass": all(checks.values())}


def encode_weights(model: m0.ResidualModel) -> bytes:
    output = bytearray(WEIGHTS_MAGIC + struct.pack("<II", 1, len(model.state_dict())))
    for name, tensor in sorted(model.state_dict().items()):
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
        raise M1TournamentError("invalid prediction serialization input")
    output = bytearray(PREDICTIONS_MAGIC + struct.pack("<II", 1, len(row_ids)))
    for row_id, values in zip(row_ids, predictions, strict=True):
        row_data = row_id.encode()
        output.extend(struct.pack("<I", len(row_data)))
        output.extend(row_data)
        output.extend(values.astype("<f8", copy=False).tobytes())
    return bytes(output)


def role_manifest(role: RoleData) -> dict[str, Any]:
    row_bytes = b"".join(
        struct.pack("<I", len(value.encode())) + value.encode() for value in role.row_ids
    )
    return {
        "case_count": role.case_count,
        "feature_sha256": {
            branch: sha256_bytes(role.features[branch].astype("<f8", copy=False).tobytes())
            for branch in BRANCHES
        },
        "modal_row_count": len(role.row_ids),
        "object_group_count": role.object_group_count,
        "row_id_sha256": sha256_bytes(row_bytes),
        "target_sha256": sha256_bytes(role.targets.astype("<f8", copy=False).tobytes()),
    }


def artifact_ref(name: str, data: bytes) -> dict[str, Any]:
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def peak_rss_bytes() -> int:
    # Linux reports ru_maxrss in KiB.
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    execution_profile, execution_profile_data = load_execution_profile(profile_path)
    dependencies = validate_dependencies(execution_profile)
    root = repository_root()
    m0_profile, m0_profile_data = m0.load_profile(
        root / execution_profile["dependencies"]["m0_profile"]["path"]
    )
    p0_profile, _ = m0.p1.p0.load_profile(
        root / m0_profile["parent"]["p0_profile"]["path"]
    )
    output = external_output(output_path)

    train = build_role("train", m0_profile, p0_profile)
    fitted_controls = fit_controls(train)
    model, training = train_candidate(train, m0_profile)
    development = build_role("development", m0_profile, p0_profile)
    development_predictions = candidate_predictions(model, development)
    development_controls = control_predictions(train, development, fitted_controls)
    development_result = development_evaluation(
        model,
        development,
        development_predictions,
        development_controls,
        m0_profile,
    )
    development_hard = hard_gate_evaluation(
        development, development_predictions, p0_profile, m0_profile
    )

    materialized_roles = {"development": development, "train": train}
    holdout_result: dict[str, Any] | None = None
    holdout_hard: dict[str, Any] | None = None
    holdout: RoleData | None = None
    holdout_predictions: np.ndarray | None = None
    decision = "REJECT_HARD_GATE"
    if development_hard["pass"]:
        decision = "REJECT_DEVELOPMENT"
        if development_result["pass"]:
            holdout = build_role("method_holdout", m0_profile, p0_profile)
            materialized_roles["method_holdout"] = holdout
            holdout_predictions = candidate_predictions(model, holdout)
            holdout_controls = control_predictions(train, holdout, fitted_controls)
            holdout_result = holdout_evaluation(
                holdout, holdout_predictions, holdout_controls, m0_profile
            )
            holdout_hard = hard_gate_evaluation(
                holdout, holdout_predictions, p0_profile, m0_profile
            )
            decision = "REJECT_HARD_GATE"
            if holdout_hard["pass"]:
                decision = (
                    "PASS_KNOWN_TRUTH_TOURNAMENT"
                    if holdout_result["pass"]
                    else "REJECT_METHOD_HOLDOUT"
                )

    owner_data = Path(__file__).read_bytes()
    corpus_manifest = {
        "access": {
            "development_rows_materialized": len(development.row_ids),
            "method_holdout_rows_materialized": 0 if holdout is None else len(holdout.row_ids),
            "network_requests": 0,
            "protected_signal_values_decoded": 0,
            "real_signal_values_decoded": 0,
        },
        "claim": CLAIM,
        "m0_profile_sha256": sha256_bytes(m0_profile_data),
        "roles": {
            role: role_manifest(data) for role, data in sorted(materialized_roles.items())
        },
        "schema": CORPUS_SCHEMA,
    }
    control_report = {
        "claim": CLAIM,
        "development": development_result,
        "method_holdout": holdout_result,
        "schema": CONTROL_SCHEMA,
    }
    payloads: dict[str, bytes] = {
        "candidate-weights.bin": encode_weights(model),
        "control-report.json": canonical_json(control_report),
        "corpus-manifest.json": canonical_json(corpus_manifest),
        "development-predictions.bin": encode_predictions(
            development.row_ids, development_predictions
        ),
    }
    if development_result["pass"] and development_hard["pass"]:
        payloads["candidate-freeze.json"] = canonical_json(
            {
                "candidate_weights_sha256": sha256_bytes(payloads["candidate-weights.bin"]),
                "claim": CLAIM,
                "development_predictions_sha256": sha256_bytes(
                    payloads["development-predictions.bin"]
                ),
                "schema": FREEZE_SCHEMA,
                "status": "FrozenAfterDevelopmentPass",
            }
        )
    if holdout_result is not None and holdout_hard is not None:
        payloads["method-holdout-report.json"] = canonical_json(
            {
                "claim": CLAIM,
                "hard": holdout_hard,
                "metrics": holdout_result,
                "prediction_sha256": sha256_bytes(
                    holdout_predictions.astype("<f8", copy=False).tobytes()
                ),
                "schema": HOLDOUT_SCHEMA,
            }
        )

    elapsed = time.monotonic() - started
    rss = peak_rss_bytes()
    resources = m0_profile["resources"]
    resource_flags = {
        "output_within_64_mib": True,
        "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
        "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
    }
    if not all(resource_flags.values()):
        decision = "REJECT_RESOURCE"
    scientific_pass = decision == "PASS_KNOWN_TRUTH_TOURNAMENT"
    evidence = {
        "artifacts": {
            name: artifact_ref(name, data) for name, data in sorted(payloads.items())
        },
        "claim": CLAIM,
        "dependencies": dependencies,
        "hard": {
            "development": development_hard,
            "method_holdout": holdout_hard,
            "t0_mutation_reason_identities_unchanged": True,
            "v0a_parent_identities_unchanged": True,
        },
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "profile_identity": {
            "bytes": len(execution_profile_data),
            "sha256": sha256_bytes(execution_profile_data),
        },
        "resources": resource_flags,
        "schema": EVIDENCE_SCHEMA,
        "training": training,
    }
    payloads["evidence.json"] = canonical_json(evidence)
    report = {
        "access": corpus_manifest["access"],
        "authored_fallback_required": True,
        "claim": CLAIM,
        "decision": decision,
        "evidence_sha256": sha256_bytes(payloads["evidence.json"]),
        "next_authorized_stage": (
            "await-fresh-real-s1-roles" if scientific_pass else "compact-residual-family-closed"
        ),
        "schema": REPORT_SCHEMA,
        "status": "Pass" if scientific_pass else "Reject",
    }
    payloads["report.json"] = canonical_json(report)
    total_output_bytes = sum(len(data) for data in payloads.values())
    if total_output_bytes > resources["max_output_bytes"]:
        raise M1TournamentError("output resource bound exceeded before publication")
    expected_files = 8 if holdout is not None else 6
    if len(payloads) != expected_files:
        raise M1TournamentError("publication file-count closure mismatch")
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v32-m1-", dir=output.parent))
    try:
        for name, data in sorted(payloads.items()):
            (staging / name).write_bytes(data)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    print(
        f"m1-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
        f"output_bytes={total_output_bytes}",
        file=os.sys.stderr,
    )
    return report


def compare_runs(path_a: Path, path_b: Path) -> dict[str, Any]:
    if not path_a.is_dir() or not path_b.is_dir():
        raise M1TournamentError("comparison inputs must be artifact directories")
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
    decision = "REPEAT_EXACT" if exact else "REJECT_NONDETERMINISTIC"
    return {
        "artifacts": artifacts,
        "claim": CLAIM,
        "decision": decision,
        "schema": COMPARE_SCHEMA,
        "status": "Pass" if exact else "Reject",
    }


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
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
                raise M1TournamentError("both comparison paths are required")
            result = compare_runs(arguments.compare_a, arguments.compare_b)
            print(canonical_json(result).decode(), end="")
            return 0 if result["status"] == "Pass" else 1
        if arguments.profile is None or arguments.output is None:
            raise M1TournamentError("profile and output are required")
        report = run(arguments.profile, arguments.output)
    except Exception as error:  # noqa: BLE001
        print(f"physical-sound-v32-m1: ContractReject: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
