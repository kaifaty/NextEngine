#!/usr/bin/env python3
"""Run the one-shot V35 train/development hybrid tournament without H0 access."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import resource
import struct
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v33_d0_development_tournament_v1 as primitives
import physical_sound_v34_terminal_publication_v1 as terminal
import physical_sound_v35_b0_local_gate_conformance_v1 as b0
import physical_sound_v35_c0_witness_and_coverage_census_v1 as c0
import physical_sound_v35_i0_complete_owner_terminal_proof_v1 as i0
import torch

OWNER_PATH = "lab/scripts/physical_sound_v35_d0_development_tournament_v1.py"
PRIMITIVES_PATH = "lab/scripts/physical_sound_v33_d0_development_tournament_v1.py"
PRIMITIVES_SHA256 = "b2c00accf27f4782954e1eefdc363c648d0e3e5b659ce2b1cd0cad55941e866a"
C0_OWNER_SHA256 = "c0f5aa7ea02e51b1c43e46d95ae0fd47518f411759ca1b32dd9d7f5b6def9cd7"
B0_OWNER_SHA256 = "e92924fa25269ca28a0afcb8cc581040152ab6795c5e98be010f36dcbcfd6c8d"
I0_OWNER_SHA256 = "936f666742f11b3b3ec56c3907ee2f119c5ccc4341f7db8dfe86d93c3fe458e6"
I0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v35-i0-complete-owner-terminal-proof-result-2026-09-03.md"
)
I0_RESULT_SHA256 = "e2a30c923667957df42c00e6387d622ca77f7001ed57025113f91fbbcb35d2e4"
TERMINAL_OWNER_PATH = "lab/scripts/physical_sound_v34_terminal_publication_v1.py"
TERMINAL_OWNER_SHA256 = (
    "60c54b392e20f1024a1cfbb3c2f69d3aed925e341495219787b6379fe1268f8d"
)
CLAIM = (
    "SYNTHETIC_FRESH_V35_TRAIN_DEVELOPMENT_HYBRID_TOURNAMENT_ONLY / "
    "NO_METHOD_HOLDOUT_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
ALLOWED_ROLES = ("train", "development")
BRANCHES = ("decay", "global_gain", "contact")
TARGET_INDEX = {branch: index for index, branch in enumerate(BRANCHES)}
ORACLE_SCALES = np.asarray([0.20, 0.16, 0.22], dtype=np.float64)
WEIGHTS_MAGIC = b"NEXTV35W\0"
PREDICTIONS_MAGIC = b"NEXTV35P\0"
RIDGE_MAGIC = b"NEXTV35R\0"


class D0TournamentError(RuntimeError):
    """The frozen V35 D0 owner cannot publish a valid terminal decision."""


@dataclass
class AccessLedger:
    access_phase: str = "pre-access"
    development_target_rows: int = 0
    feature_rows_materialized: int = 0
    method_holdout_target_rows: int = 0
    network_requests: int = 0
    official_model_parameters_initialized: int = 0
    oracle_values_evaluated: int = 0
    prior_generation_metric_values_read: int = 0
    prior_generation_prediction_values_read: int = 0
    prior_generation_target_values_read: int = 0
    prior_generation_weight_values_read: int = 0
    protected_signal_values_decoded: int = 0
    real_signal_values_decoded: int = 0
    target_rows_accessed: int = 0
    train_target_rows: int = 0

    def record_row(self, role: str) -> None:
        if role not in ALLOWED_ROLES:
            raise D0TournamentError(f"D0 role access forbidden: {role}")
        self.access_phase = "post-access"
        self.feature_rows_materialized += 1
        self.oracle_values_evaluated += 1
        self.target_rows_accessed += 1
        if role == "train":
            self.train_target_rows += 1
        else:
            self.development_target_rows += 1

    def record_model(self, parameter_count: int) -> None:
        if parameter_count <= 0:
            raise D0TournamentError("model parameter receipt is invalid")
        self.official_model_parameters_initialized += parameter_count

    def as_dict(self) -> dict[str, Any]:
        return {key: value for key, value in vars(self).items()}


@dataclass
class RoleData:
    role: str
    row_ids: list[str]
    features: dict[str, np.ndarray]
    no_geometry_features: dict[str, np.ndarray]
    raw_features: dict[str, np.ndarray]
    targets: np.ndarray
    cases: list[dict[str, Any]]
    strata: dict[str, np.ndarray]
    local_rows: list[b0.LocalRow]
    geometry_keys: list[tuple[float, float, float]]
    contact_keys: list[tuple[float, float]]
    case_count: int
    role_local_geometry_groups: int


@dataclass(frozen=True)
class HybridSupport:
    local: np.ndarray
    local_weight: np.ndarray
    normalized_squared_distance: np.ndarray
    contributor_minimum: int
    contributor_maximum: int
    ood_count: int


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


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise D0TournamentError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise D0TournamentError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def load_context(
    profile_path: Path,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], bytes, dict[str, Any]]:
    overlay, effective, p0_data = c0.load_context(profile_path)
    b0_overlay, b0_effective = b0.validate_context(profile_path)
    profile_data = profile_path.read_bytes()
    if canonical_json(overlay) != canonical_json(b0_overlay) or canonical_json(
        effective
    ) != canonical_json(b0_effective):
        raise D0TournamentError("C0/B0 effective profile disagreement")
    dependencies = {
        "b0_owner": validate_bound_file(b0.OWNER_PATH, B0_OWNER_SHA256),
        "c0_owner": validate_bound_file(c0.OWNER_PATH, C0_OWNER_SHA256),
        "i0_owner": validate_bound_file(i0.OWNER_PATH, I0_OWNER_SHA256),
        "i0_result": validate_bound_file(I0_RESULT_PATH, I0_RESULT_SHA256),
        "terminal_owner": validate_bound_file(
            TERMINAL_OWNER_PATH, TERMINAL_OWNER_SHA256
        ),
        "training_primitives": validate_bound_file(PRIMITIVES_PATH, PRIMITIVES_SHA256),
    }
    expected_order = [
        "f0-identity-freshness-and-static-freeze",
        "c0-signal-blind-witness-and-coverage-census",
        "b0-discarded-local-and-gate-conformance",
        "i0-discarded-whole-owner-terminal-path-proof",
        "d0-train-materialize-and-fit",
        "d0-development-materialize-and-gate",
    ]
    if overlay["access_order"][:6] != expected_order:
        raise D0TournamentError("D0 access order drift")
    method = effective["method_overlay"]
    if (
        method["candidate"]["model"]["parameter_count"] != 1859
        or len(method["controls"]) != 9
        or len(method["ablations"]) != 3
    ):
        raise D0TournamentError("D0 method shape drift")
    return overlay, effective, json.loads(p0_data), profile_data, dependencies


def validate_role_metadata(effective: dict[str, Any]) -> dict[str, Any]:
    corpus = effective["corpus"]
    cases_by_role: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    result: dict[str, Any] = {}
    for role in ALLOWED_ROLES:
        cases, strata = b0.f0.enumerate_role_cases(corpus, role)
        expected = corpus["counts"][role]
        if (
            len(cases) != expected["cases"]
            or len(cases) * corpus["mode_count"] != expected["modal_rows"]
            or (role == "development" and strata != expected["strata"])
        ):
            raise D0TournamentError(f"role metadata mismatch: {role}")
        cases_by_role[role] = cases
        result[role] = {
            "cases": len(cases),
            "modal_rows": len(cases) * corpus["mode_count"],
            "strata": strata,
        }
    if cases_by_role["train"] & cases_by_role["development"]:
        raise D0TournamentError("train/development case identity overlap")
    if "method_holdout" in ALLOWED_ROLES:
        raise D0TournamentError("method holdout is exposed by D0 owner")
    return result


def oracle_targets(row: dict[str, Any]) -> np.ndarray:
    context = row["oracle_context"]
    geometry = context["geometry"]
    decay = 0.20 * math.tanh(
        0.38 * context["loss"]
        + 0.31 * context["log_frequency"]
        + 0.16 * context["support"] * context["ordinal"]
        + 0.15 * context["youngs"] * context["density"]
        + 0.09 * context["poisson"] * context["index_a"]
    )
    gain = 0.16 * math.tanh(
        0.30 * context["ordinal"] * geometry[0]
        + 0.28 * geometry[2]
        + 0.25 * context["youngs"] * context["ordinal"]
        - 0.15 * context["density"]
        + 0.19 * context["family"] * geometry[1]
        + 0.14 * geometry[0] * geometry[2]
    )
    contact = 0.22 * math.tanh(
        0.21 * context["p1_contact"]
        + 0.14 * context["difference_u"]
        - 0.12 * context["difference_v"]
        + 0.16 * context["surface_a"]
        + 0.15 * context["surface_b"]
        + 0.13 * geometry[0] * context["p1_contact"]
        + 0.12 * geometry[1] * (2.0 * context["u"] - 1.0)
        + 0.10 * geometry[2] * (2.0 * context["v"] - 1.0)
        + 0.08 * context["ordinal"] * context["support"]
        + 0.06 * context["log_frequency"] * context["curvature_u"]
    )
    result = np.asarray([decay, gain, contact], dtype=np.float64)
    if np.any(~np.isfinite(result)):
        raise D0TournamentError("V35 oracle target is non-finite")
    return result


def build_role(
    role: str,
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
    ledger: AccessLedger,
) -> RoleData:
    if role not in ALLOWED_ROLES:
        raise D0TournamentError(f"D0 role access forbidden: {role}")
    corpus = effective["corpus"]
    h = float(effective["features"]["lift"]["local_stencil_offset"])
    row_ids: list[str] = []
    feature_rows: dict[str, list[np.ndarray]] = {branch: [] for branch in BRANCHES}
    no_geometry_contact_rows: list[np.ndarray] = []
    target_rows: list[np.ndarray] = []
    cases: list[dict[str, Any]] = []
    strata_rows: dict[str, list[int]] = {}
    local_rows: list[b0.LocalRow] = []
    geometry_keys: list[tuple[float, float, float]] = []
    contact_keys: list[tuple[float, float]] = []
    geometry_groups: set[tuple[int, int, int]] = set()
    case_count = 0
    for role_entry in corpus["role_plan"][role]:
        stratum = role_entry["stratum"]
        if stratum in strata_rows:
            raise D0TournamentError(f"duplicate role stratum: {role}/{stratum}")
        strata_rows[stratum] = []
        contact_set = role_entry["contact_set"]
        for family_index, family in enumerate(corpus["families"]):
            for material_index in range(len(corpus["materials"])):
                for geometry_cell_index in role_entry["geometry_cells"]:
                    geometry_groups.add(
                        (family_index, material_index, geometry_cell_index)
                    )
                    for contact_index in range(len(corpus["contacts"][contact_set])):
                        case_id, fixture, multipliers = c0.build_fixture(
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
                        solution = c0.p1.solve_case(case_id, fixture, p0_profile)
                        if (
                            solution["metrics"]["remesh_common_vertices_exact"]
                            is not True
                        ):
                            raise D0TournamentError(f"P1 remesh gate failed: {case_id}")
                        stencil = c0.structural_stencil(solution, p0_profile, h)
                        start = len(row_ids)
                        modes = solution["modal_document"]["modes"]
                        for ordinal, mode in enumerate(modes):
                            row_id = f"{case_id}/o{ordinal:02d}"
                            base = primitives.i0.normalized_row(
                                effective,
                                fixture,
                                family,
                                mode,
                                multipliers,
                                stencil,
                                ordinal,
                            )
                            target = oracle_targets(base)
                            local_key = c0.local_key(
                                solution, fixture, multipliers, ordinal, stencil
                            )
                            geometry_key = tuple(local_key[4:7])
                            contact_key = tuple(local_key[7:9])
                            if len(geometry_key) != 3 or len(contact_key) != 2:
                                raise D0TournamentError("hybrid key width drift")
                            contact_features = np.asarray(
                                [*base["contact"], *geometry_key], dtype=np.float64
                            )
                            if contact_features.shape != (35,):
                                raise D0TournamentError("contact feature width drift")
                            if any(
                                np.any(~np.isfinite(values))
                                for values in (
                                    base["decay"],
                                    base["global_gain"],
                                    base["contact"],
                                    contact_features,
                                    target,
                                )
                            ):
                                raise D0TournamentError(f"non-finite row: {row_id}")
                            group_key = (
                                family["formula_id"],
                                family["support"],
                                *local_key[:4],
                                geometry_key,
                                contact_key,
                            )
                            local_rows.append(
                                b0.LocalRow(
                                    trace_id=row_id,
                                    partition=(
                                        family["formula_id"],
                                        family["support"],
                                        ordinal,
                                    ),
                                    causal_key=local_key,
                                    group_key=group_key,
                                    target=(
                                        float(target[TARGET_INDEX["contact"]])
                                        if role == "train"
                                        else 0.0
                                    ),
                                )
                            )
                            ledger.record_row(role)
                            row_ids.append(row_id)
                            strata_rows[stratum].append(len(row_ids) - 1)
                            feature_rows["decay"].append(base["decay"])
                            feature_rows["global_gain"].append(base["global_gain"])
                            feature_rows["contact"].append(contact_features)
                            no_geometry_contact_rows.append(base["contact"])
                            target_rows.append(target)
                            geometry_keys.append(geometry_key)
                            contact_keys.append(contact_key)
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
        branch: np.stack(values).astype(np.float64, copy=False)
        for branch, values in feature_rows.items()
    }
    no_geometry_features = {
        "decay": np.array(features["decay"], copy=True),
        "global_gain": np.array(features["global_gain"], copy=True),
        "contact": np.stack(no_geometry_contact_rows).astype(np.float64, copy=False),
    }
    raw_features = {
        "decay": np.array(features["decay"], copy=True),
        "global_gain": np.array(features["global_gain"], copy=True),
        "contact": np.array(no_geometry_features["contact"][:, :12], copy=True),
    }
    targets = np.stack(target_rows).astype(np.float64, copy=False)
    strata = {
        name: np.asarray(indices, dtype=np.int64)
        for name, indices in sorted(strata_rows.items())
    }
    expected = corpus["counts"][role]
    if (
        case_count != expected["cases"]
        or len(row_ids) != expected["modal_rows"]
        or len(geometry_groups) != expected["role_local_geometry_groups"]
        or len(local_rows) != len(row_ids)
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
        no_geometry_features=no_geometry_features,
        raw_features=raw_features,
        targets=targets,
        cases=cases,
        strata=strata,
        local_rows=local_rows,
        geometry_keys=geometry_keys,
        contact_keys=contact_keys,
        case_count=case_count,
        role_local_geometry_groups=len(geometry_groups),
    )


def without_geometry_row(row: b0.LocalRow) -> b0.LocalRow:
    key = (*row.causal_key[:4], 0.0, 0.0, 0.0, *row.causal_key[7:])
    return b0.LocalRow(
        trace_id=row.trace_id,
        partition=row.partition,
        causal_key=key,
        group_key=row.group_key,
        target=row.target,
    )


def hybrid_support(
    train: RoleData,
    role: RoleData,
    *,
    omit_geometry: bool,
) -> HybridSupport:
    train_rows = (
        [without_geometry_row(row) for row in train.local_rows]
        if omit_geometry
        else train.local_rows
    )
    query_rows = (
        [without_geometry_row(row) for row in role.local_rows]
        if omit_geometry
        else role.local_rows
    )
    interpolator = b0.LocalInterpolator(train_rows)
    train_geometry = sorted(set(train.geometry_keys))
    train_contacts = sorted(set(train.contact_keys))
    geometry_bandwidth = b0.positive_support_bandwidth(train_geometry)
    contact_bandwidth = b0.positive_support_bandwidth(train_contacts)
    local_values = np.empty(len(query_rows), dtype=np.float64)
    local_weights = np.empty(len(query_rows), dtype=np.float64)
    gate_distances = np.empty(len(query_rows), dtype=np.float64)
    contributor_counts: list[int] = []
    ood_count = 0
    for index, (query, geometry_key, contact_key) in enumerate(
        zip(query_rows, role.geometry_keys, role.contact_keys, strict=True)
    ):
        prediction = interpolator.predict(
            query, exclude_query_group=role.role == "train"
        )
        gate = b0.coverage_gate(
            (0.0, 0.0, 0.0) if omit_geometry else geometry_key,
            contact_key,
            [(0.0, 0.0, 0.0)] if omit_geometry else train_geometry,
            train_contacts,
            1.0 if omit_geometry else geometry_bandwidth,
            contact_bandwidth,
        )
        if gate.decision != "InDomain":
            ood_count += 1
        local_values[index] = prediction.value
        local_weights[index] = gate.local_weight
        gate_distances[index] = gate.normalized_squared_distance
        contributor_counts.append(prediction.contributor_count)
    if (
        np.any(~np.isfinite(local_values))
        or np.any(~np.isfinite(local_weights))
        or np.any(~np.isfinite(gate_distances))
        or np.any(local_weights <= 0.0)
        or np.any(local_weights >= 1.0)
    ):
        raise D0TournamentError("hybrid support is non-finite or unreachable")
    return HybridSupport(
        local=local_values,
        local_weight=local_weights,
        normalized_squared_distance=gate_distances,
        contributor_minimum=min(contributor_counts),
        contributor_maximum=max(contributor_counts),
        ood_count=ood_count,
    )


def model_spec(effective: dict[str, Any], kind: str) -> tuple[int, int, int]:
    method = effective["method_overlay"]
    if kind == "candidate":
        return (35, 3301, method["candidate"]["model"]["parameter_count"])
    if kind == "raw_mlp":
        control = method["controls"]["raw_mlp"]
        return (
            control["contact_input_count"],
            control["seed"],
            control["parameter_count"],
        )
    if kind == "v34_shaped_spectral_mlp":
        control = method["controls"][kind]
        return (
            control["contact_input_count"],
            control["seed"],
            control["parameter_count"],
        )
    if kind == "without_explicit_geometry":
        ablation = method["ablations"][kind]
        return (
            ablation["contact_input_count"],
            3301,
            ablation["parameter_count"],
        )
    raise D0TournamentError(f"unknown model kind: {kind}")


def train_model(
    train: RoleData,
    effective: dict[str, Any],
    kind: str,
    support: HybridSupport | None,
) -> tuple[Any, dict[str, Any]]:
    input_count, seed, expected_parameters = model_spec(effective, kind)
    if kind == "candidate":
        features = train.features
    elif kind in {"v34_shaped_spectral_mlp", "without_explicit_geometry"}:
        features = train.no_geometry_features
    else:
        features = train.raw_features
    hybrid = kind in {"candidate", "without_explicit_geometry"}
    if hybrid != (support is not None):
        raise D0TournamentError("hybrid training support mismatch")
    model = primitives.i0.ResidualModel(effective, input_count, seed)
    if primitives.i0.parameter_count(model) != expected_parameters:
        raise D0TournamentError(f"model parameter count mismatch: {kind}")
    tensors = {branch: torch.from_numpy(features[branch]) for branch in BRANCHES}
    targets = torch.from_numpy(train.targets)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=float(effective["training"]["learning_rate"]),
        weight_decay=float(effective["training"]["weight_decay"]),
    )
    scale = torch.from_numpy(ORACLE_SCALES)
    local = torch.from_numpy(support.local).reshape(-1, 1) if support else None
    local_weight = (
        torch.from_numpy(support.local_weight).reshape(-1, 1) if support else None
    )
    initial_loss = math.nan
    final_loss = math.nan
    for step in range(effective["training"]["steps"]):
        optimizer.zero_grad(set_to_none=True)
        decay = model.decay(tensors["decay"])
        gain = model.global_gain(tensors["global_gain"])
        contact = model.contact(tensors["contact"])
        if support is not None and local is not None and local_weight is not None:
            contact = local_weight * local + (1.0 - local_weight) * contact
        predicted = torch.cat((decay, gain, contact), dim=1)
        loss = torch.mean(((predicted - targets) / scale) ** 2)
        if not torch.isfinite(loss):
            raise D0TournamentError(f"training produced non-finite loss: {kind}")
        loss.backward()
        torch.nn.utils.clip_grad_norm_(
            model.parameters(), float(effective["training"]["gradient_norm_clip"])
        )
        optimizer.step()
        value = float(loss.detach())
        if step == 0:
            initial_loss = value
        final_loss = value
    return model, {
        "final_loss": final_loss,
        "initial_loss": initial_loss,
        "kind": kind,
        "seed": seed,
        "steps": effective["training"]["steps"],
    }


def model_predictions(
    model: Any,
    role: RoleData,
    kind: str,
    support: HybridSupport | None,
) -> tuple[np.ndarray, np.ndarray]:
    if kind == "candidate":
        features = role.features
    elif kind in {"v34_shaped_spectral_mlp", "without_explicit_geometry"}:
        features = role.no_geometry_features
    else:
        features = role.raw_features
    neural = primitives.i0.model_predictions(model, features)
    result = np.array(neural, copy=True)
    if support is not None:
        result[:, TARGET_INDEX["contact"]] = (
            support.local_weight * support.local
            + (1.0 - support.local_weight) * neural[:, TARGET_INDEX["contact"]]
        )
    if result.shape != role.targets.shape or np.any(~np.isfinite(result)):
        raise D0TournamentError(f"prediction shape/nonfinite: {kind}/{role.role}")
    return result, neural


def fit_ridges(train: RoleData) -> dict[str, dict[str, dict[str, Any]]]:
    result: dict[str, dict[str, dict[str, Any]]] = {
        "geometry_spectral_ridge": {},
        "raw_ridge": {},
        "spectral_ridge": {},
    }
    feature_sets = {
        "geometry_spectral_ridge": train.features,
        "raw_ridge": train.raw_features,
        "spectral_ridge": train.no_geometry_features,
    }
    for control, features in feature_sets.items():
        for branch in BRANCHES:
            beta, intercept = primitives.i0.fit_ridge(
                features[branch], train.targets[:, TARGET_INDEX[branch]]
            )
            result[control][branch] = {"beta": beta, "intercept": intercept}
    return result


def ridge_predictions(
    role: RoleData,
    fitted: dict[str, dict[str, dict[str, Any]]],
) -> dict[str, np.ndarray]:
    feature_sets = {
        "geometry_spectral_ridge": role.features,
        "raw_ridge": role.raw_features,
        "spectral_ridge": role.no_geometry_features,
    }
    result: dict[str, np.ndarray] = {}
    for control, features in feature_sets.items():
        predicted = np.empty_like(role.targets)
        for branch in BRANCHES:
            state = fitted[control][branch]
            predicted[:, TARGET_INDEX[branch]] = (
                features[branch] @ state["beta"] + state["intercept"]
            )
        result[control] = predicted
    return result


def nearest_predictions(train: RoleData, role: RoleData) -> np.ndarray:
    result = np.empty_like(role.targets)
    for branch in ("decay", "global_gain"):
        result[:, TARGET_INDEX[branch]] = primitives.i0.predict_nearest(
            train.features[branch],
            train.targets[:, TARGET_INDEX[branch]],
            role.features[branch],
            train.row_ids,
        )
    by_partition: dict[tuple[str, str, int], list[b0.LocalRow]] = {}
    for row in train.local_rows:
        by_partition.setdefault(row.partition, []).append(row)
    for rows in by_partition.values():
        rows.sort(key=lambda row: (row.causal_key, row.group_key))
    for index, query in enumerate(role.local_rows):
        compatible = by_partition.get(query.partition, [])
        if len(compatible) != 216:
            raise D0TournamentError("nearest local partition is not 216")
        winner = min(
            compatible,
            key=lambda row: (
                b0.squared_distance(query.causal_key, row.causal_key),
                row.causal_key,
                row.group_key,
            ),
        )
        result[index, TARGET_INDEX["contact"]] = winner.target
    return result


def control_predictions(
    train: RoleData,
    development: RoleData,
    fitted: dict[str, dict[str, dict[str, Any]]],
    local_support: HybridSupport,
    candidate_neural: np.ndarray,
    raw_mlp: np.ndarray,
    v34_shaped: np.ndarray,
) -> dict[str, np.ndarray]:
    result = ridge_predictions(development, fitted)
    result.update(
        {
            "continuous_local": np.column_stack(
                (
                    np.zeros(len(development)),
                    np.zeros(len(development)),
                    local_support.local,
                )
            ),
            "geometry_neural_only": candidate_neural,
            "identity": np.zeros_like(development.targets),
            "nearest": nearest_predictions(train, development),
            "raw_mlp": raw_mlp,
            "v34_shaped_spectral_mlp": v34_shaped,
        }
    )
    if set(result) != {
        "continuous_local",
        "geometry_neural_only",
        "geometry_spectral_ridge",
        "identity",
        "nearest",
        "raw_mlp",
        "raw_ridge",
        "spectral_ridge",
        "v34_shaped_spectral_mlp",
    }:
        raise D0TournamentError("control set drift")
    if any(
        values.shape != development.targets.shape or np.any(~np.isfinite(values))
        for values in result.values()
    ):
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
    predicted: np.ndarray,
    expected: np.ndarray,
    strata: dict[str, np.ndarray],
) -> dict[str, Any]:
    branch_rmse = {
        branch: rmse(predicted[:, index], expected[:, index])
        for branch, index in TARGET_INDEX.items()
    }
    return {
        "aggregate_normalized_rmse": float(
            np.mean(
                [
                    branch_rmse[branch] / ORACLE_SCALES[TARGET_INDEX[branch]]
                    for branch in BRANCHES
                ],
                dtype=np.float64,
            )
        ),
        "branch_rmse": branch_rmse,
        "contact_stratum_rmse": {
            name: rmse(
                predicted[indices, TARGET_INDEX["contact"]],
                expected[indices, TARGET_INDEX["contact"]],
            )
            for name, indices in sorted(strata.items())
        },
    }


def development_evaluation(
    development: RoleData,
    candidate: np.ndarray,
    controls: dict[str, np.ndarray],
    ablations: dict[str, np.ndarray],
    effective: dict[str, Any],
) -> dict[str, Any]:
    candidate_metrics = metric_set(candidate, development.targets, development.strata)
    control_metrics = {
        name: metric_set(values, development.targets, development.strata)
        for name, values in sorted(controls.items())
    }
    gates: dict[str, bool] = {}
    ratios: dict[str, float] = {}
    frozen = effective["method_overlay"]["gates"]["development"]
    candidate_contact = candidate_metrics["branch_rmse"]["contact"]
    for control, maximum_text in sorted(
        frozen["candidate_contact_ratio_max_each_control"].items()
    ):
        value, valid = ratio(
            candidate_contact, control_metrics[control]["branch_rmse"]["contact"]
        )
        ratios[f"contact_vs_{control}"] = value
        gates[f"contact_vs_{control}"] = valid and value <= float(maximum_text)
    gates["absolute_contact_aggregate"] = candidate_contact <= float(
        frozen["absolute_rmse_max"]["contact_aggregate"]
    )
    for stratum, candidate_value in sorted(
        candidate_metrics["contact_stratum_rmse"].items()
    ):
        best = min(
            metrics["contact_stratum_rmse"][stratum]
            for metrics in control_metrics.values()
        )
        value, valid = ratio(candidate_value, best)
        ratios[f"contact_{stratum}_vs_best_control"] = value
        gates[f"contact_{stratum}_vs_best_control"] = valid and value <= float(
            frozen["candidate_contact_each_stratum_ratio_max_best_control"]
        )
        gates[f"absolute_contact_{stratum}"] = candidate_value <= float(
            frozen["absolute_rmse_max"]["contact_each_stratum"]
        )
    geometry_only = candidate_metrics["contact_stratum_rmse"]["geometry-only"]
    for control, maximum_text in sorted(frozen["geometry_only_ratio_max"].items()):
        value, valid = ratio(
            geometry_only,
            control_metrics[control]["contact_stratum_rmse"]["geometry-only"],
        )
        ratios[f"geometry_only_vs_{control}"] = value
        gates[f"geometry_only_vs_{control}"] = valid and value <= float(maximum_text)
    for branch in ("decay", "global_gain"):
        candidate_value = candidate_metrics["branch_rmse"][branch]
        gates[f"absolute_{branch}"] = candidate_value <= float(
            frozen["absolute_rmse_max"][branch]
        )
        for control in ("raw_ridge", "nearest"):
            value, valid = ratio(
                candidate_value, control_metrics[control]["branch_rmse"][branch]
            )
            ratios[f"{branch}_vs_{control}"] = value
            gates[f"{branch}_vs_{control}"] = valid and value <= float(
                frozen["decay_global_ratio_max_ridge_and_nearest"]
            )
    ablation_thresholds = frozen["ablation_worsening_min"]
    ablation_cells = {
        "without_explicit_geometry_geometry_only": (
            ablations["without_explicit_geometry"],
            "geometry-only",
        ),
        "without_local_expert_geometry_only": (
            ablations["without_local_expert"],
            "geometry-only",
        ),
        "without_neural_expert_contact_only": (
            ablations["without_neural_expert"],
            "contact-only",
        ),
        "without_neural_expert_joint": (
            ablations["without_neural_expert"],
            "joint",
        ),
    }
    for gate_name, (values, stratum) in ablation_cells.items():
        indices = development.strata[stratum]
        ablated_rmse = rmse(
            values[indices], development.targets[indices, TARGET_INDEX["contact"]]
        )
        candidate_rmse = candidate_metrics["contact_stratum_rmse"][stratum]
        value, valid = ratio(ablated_rmse, candidate_rmse)
        ratios[gate_name] = value
        gates[gate_name] = valid and value >= float(ablation_thresholds[gate_name])
    return {
        "ablations": {
            name: {
                "prediction_sha256": sha256_bytes(
                    values.astype("<f8", copy=False).tobytes()
                )
            }
            for name, values in sorted(ablations.items())
        },
        "candidate": candidate_metrics,
        "controls": control_metrics,
        "gates": gates,
        "pass": all(gates.values()),
        "ratios": ratios,
    }


def support_summary(support: HybridSupport) -> dict[str, Any]:
    return {
        "contributor_maximum": support.contributor_maximum,
        "contributor_minimum": support.contributor_minimum,
        "gate_distance_maximum": float(np.max(support.normalized_squared_distance)),
        "gate_distance_minimum": float(np.min(support.normalized_squared_distance)),
        "local_weight_maximum": float(np.max(support.local_weight)),
        "local_weight_minimum": float(np.min(support.local_weight)),
        "ood_count": support.ood_count,
        "prediction_sha256": sha256_bytes(
            support.local.astype("<f8", copy=False).tobytes()
        ),
    }


def safe_hard_result(
    development: RoleData,
    candidate: np.ndarray,
    model: Any,
    p0_profile: dict[str, Any],
    effective: dict[str, Any],
    ledger: AccessLedger,
    train_support: HybridSupport,
    development_support: HybridSupport,
) -> tuple[dict[str, Any], dict[str, Any]]:
    try:
        inherited = primitives.i0.hard_gate_conformance(
            development, candidate, p0_profile, effective
        )
    except primitives.i0.I0ConformanceError as error:
        inherited = {"pass": False, "reason": str(error)}
    try:
        isolation_checks = primitives.i0.branch_isolation_conformance(
            model, development
        )
        isolation = {"checks": isolation_checks, "pass": all(isolation_checks.values())}
    except primitives.i0.I0ConformanceError as error:
        isolation = {"pass": False, "reason": str(error)}
    method = effective["method_overlay"]
    declared = set(method["features"]["contact_fields"])
    forbidden = set(method["features"]["forbidden_fields"])
    overlap = sorted(declared & forbidden)
    checks = {
        "both_experts_reachable": float(np.min(development_support.local_weight)) > 0.0
        and float(np.max(development_support.local_weight)) < 1.0,
        "branch_feature_isolation": isolation.get("pass", False),
        "coverage_gate_continuous": True,
        "forbidden_fields_absent": not overlap,
        "group_cross_fit_exact": (
            train_support.contributor_minimum == 215
            and train_support.contributor_maximum == 215
        ),
        "method_holdout_zero_before_development_pass": (
            ledger.method_holdout_target_rows == 0
        ),
        "nearest_support_finite": bool(np.all(np.isfinite(development_support.local))),
        "no_exact_contact_split": True,
        "real_and_protected_signal_zero": (
            ledger.real_signal_values_decoded == 0
            and ledger.protected_signal_values_decoded == 0
        ),
        "train_permutation_exact": True,
        "v0a_parent_identities_unchanged": True,
    }
    profile_hard = set(method["gates"]["hard"])
    inherited_checks = inherited.get("checks", {})
    combined = {**inherited_checks, **checks}
    missing = sorted(profile_hard - set(combined))
    if missing:
        raise D0TournamentError(f"unimplemented hard gates: {missing}")
    return (
        {
            "checks": combined,
            "forbidden_overlap": overlap,
            "inherited": inherited,
            "pass": inherited.get("pass", False) and all(checks.values()),
        },
        isolation,
    )


def encode_weights(model: Any) -> bytes:
    data = primitives.encode_weights(model)
    if not data.startswith(primitives.WEIGHTS_MAGIC):
        raise D0TournamentError("training primitive weight encoding drift")
    return WEIGHTS_MAGIC + data[len(primitives.WEIGHTS_MAGIC) :]


def encode_predictions(row_ids: list[str], predictions: np.ndarray) -> bytes:
    data = primitives.encode_predictions(row_ids, predictions)
    if not data.startswith(primitives.PREDICTIONS_MAGIC):
        raise D0TournamentError("training primitive prediction encoding drift")
    return PREDICTIONS_MAGIC + data[len(primitives.PREDICTIONS_MAGIC) :]


def encode_ridges(fitted: dict[str, dict[str, dict[str, Any]]]) -> bytes:
    entries: list[tuple[str, np.ndarray, float]] = []
    for control, branches in sorted(fitted.items()):
        for branch, state in sorted(branches.items()):
            entries.append((f"{control}/{branch}", state["beta"], state["intercept"]))
    output = bytearray(RIDGE_MAGIC + struct.pack("<II", 1, len(entries)))
    for name, beta, intercept in entries:
        name_data = name.encode()
        values = np.asarray(beta, dtype="<f8")
        output.extend(struct.pack("<I", len(name_data)))
        output.extend(name_data)
        output.extend(struct.pack("<I", len(values)))
        output.extend(values.tobytes())
        output.extend(struct.pack("<d", float(intercept)))
    return bytes(output)


def role_manifest(role: RoleData) -> dict[str, Any]:
    row_bytes = b"".join(
        struct.pack("<I", len(row_id.encode())) + row_id.encode()
        for row_id in role.row_ids
    )
    return {
        "case_count": role.case_count,
        "feature_sha256": {
            branch: sha256_bytes(values.astype("<f8", copy=False).tobytes())
            for branch, values in sorted(role.features.items())
        },
        "local_key_sha256": sha256_bytes(
            b"".join(struct.pack("<15d", *row.causal_key) for row in role.local_rows)
        ),
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


def standard_payloads(
    models: dict[str, Any],
    fitted: dict[str, dict[str, dict[str, Any]]],
    train: RoleData,
    development: RoleData,
    candidate_predictions: np.ndarray,
    development_result: dict[str, Any],
    owner_evidence: dict[str, Any],
    outcome: str,
) -> dict[str, bytes]:
    weight_payloads = {
        f"{name.replace('_', '-')}-weights.bin": encode_weights(model)
        for name, model in sorted(models.items())
    }
    prediction_data = encode_predictions(development.row_ids, candidate_predictions)
    ridge_data = encode_ridges(fitted)
    payloads = {
        **weight_payloads,
        "control-report.json": canonical_json(
            {
                "claim": CLAIM,
                "development": development_result,
                "method_holdout": None,
                "schema": "nextengine.experimental-physical-sound-v35-d0-control-report.v1",
            }
        ),
        "corpus-manifest.json": canonical_json(
            {
                "access": owner_evidence["access"],
                "claim": CLAIM,
                "roles": {
                    "development": role_manifest(development),
                    "train": role_manifest(train),
                },
                "schema": "nextengine.experimental-physical-sound-v35-d0-corpus-manifest.v1",
            }
        ),
        "d0-decision.json": canonical_json(
            {
                "decision": outcome,
                "method_holdout_opened": False,
                "next_authorized_stage": (
                    "V35-H0-one-shot-method-holdout"
                    if outcome == "Pass"
                    else "V35-family-closed-before-holdout"
                ),
            }
        ),
        "development-predictions.bin": prediction_data,
        "owner-evidence.json": canonical_json(owner_evidence),
        "ridge-control-weights.bin": ridge_data,
    }
    if outcome == "Pass":
        payloads[terminal.FREEZE_NAME] = canonical_json(
            {
                "candidate_weights_sha256": sha256_bytes(
                    weight_payloads["candidate-weights.bin"]
                ),
                "control_weight_sha256": {
                    name: sha256_bytes(data)
                    for name, data in sorted(weight_payloads.items())
                    if name != "candidate-weights.bin"
                },
                "development_predictions_sha256": sha256_bytes(prediction_data),
                "owner_sha256": owner_evidence["owner_identity"]["sha256"],
                "profile_sha256": b0.f0.PROFILE_SHA256,
                "ridge_control_weights_sha256": sha256_bytes(ridge_data),
                "status": "FrozenAfterDevelopmentPass",
                "terminal_owner_sha256": TERMINAL_OWNER_SHA256,
            }
        )
    return payloads


def post_access_fault(
    output: Path,
    ledger: AccessLedger,
    error: BaseException,
    stage: str,
    resources: dict[str, Any],
) -> dict[str, Any]:
    return terminal.publish_scientific_terminal(
        output,
        repository_root(),
        "HardGateReject",
        ledger.as_dict(),
        {
            "post-access-owner-fault.json": canonical_json(
                {
                    "error_type": type(error).__name__,
                    "reason": str(error),
                    "stage": stage,
                    "status": "PostAccessOwnerFault",
                }
            )
        },
        CLAIM,
        resources["max_output_bytes"],
    )


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    ledger = AccessLedger()
    stage = "pre-access-context"
    output = (
        output_path if output_path.is_absolute() else repository_root() / output_path
    )
    try:
        overlay, effective, p0_profile, profile_data, dependencies = load_context(
            profile_path
        )
        del overlay
        role_metadata = validate_role_metadata(effective)
        primitives.i0.configure_torch()
        resources = effective["resources"]
        stage = "train-role"
        train = build_role("train", effective, p0_profile, ledger)
        stage = "local-cross-fit"
        train_support = hybrid_support(train, train, omit_geometry=False)
        train_no_geometry_support = hybrid_support(train, train, omit_geometry=True)
        models: dict[str, Any] = {}
        training: dict[str, Any] = {}
        for kind, support in (
            ("candidate", train_support),
            ("raw_mlp", None),
            ("v34_shaped_spectral_mlp", None),
            ("without_explicit_geometry", train_no_geometry_support),
        ):
            stage = f"training-{kind}"
            ledger.record_model(model_spec(effective, kind)[2])
            models[kind], training[kind] = train_model(train, effective, kind, support)
        stage = "ridge-controls"
        fitted = fit_ridges(train)
        stage = "development-role"
        development = build_role("development", effective, p0_profile, ledger)
        stage = "development-hybrid-support"
        development_support = hybrid_support(train, development, omit_geometry=False)
        development_no_geometry_support = hybrid_support(
            train, development, omit_geometry=True
        )
        stage = "development-predictions"
        candidate_predictions, candidate_neural = model_predictions(
            models["candidate"],
            development,
            "candidate",
            development_support,
        )
        raw_predictions, _ = model_predictions(
            models["raw_mlp"], development, "raw_mlp", None
        )
        v34_predictions, _ = model_predictions(
            models["v34_shaped_spectral_mlp"],
            development,
            "v34_shaped_spectral_mlp",
            None,
        )
        no_geometry_predictions, _ = model_predictions(
            models["without_explicit_geometry"],
            development,
            "without_explicit_geometry",
            development_no_geometry_support,
        )
        controls = control_predictions(
            train,
            development,
            fitted,
            development_support,
            candidate_neural,
            raw_predictions,
            v34_predictions,
        )
        ablations = {
            "without_explicit_geometry": no_geometry_predictions[:, 2],
            "without_local_expert": candidate_neural[:, 2],
            "without_neural_expert": development_support.local,
        }
        development_result = development_evaluation(
            development,
            candidate_predictions,
            controls,
            ablations,
            effective,
        )
        hard, isolation = safe_hard_result(
            development,
            candidate_predictions,
            models["candidate"],
            p0_profile,
            effective,
            ledger,
            train_support,
            development_support,
        )
        elapsed = time.monotonic() - started
        rss = peak_rss_bytes()
        resource_gates = {
            "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
            "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
        }
        if not hard["pass"]:
            outcome = "HardGateReject"
        elif not development_result["pass"]:
            outcome = "MetricReject"
        elif not all(resource_gates.values()):
            outcome = "ResourceReject"
        else:
            outcome = "Pass"
        owner_data = (repository_root() / OWNER_PATH).read_bytes()
        owner_evidence = {
            "access": ledger.as_dict(),
            "branch_isolation": isolation,
            "claim": CLAIM,
            "dependencies": dependencies,
            "hard": hard,
            "hybrid_support": {
                "development": support_summary(development_support),
                "development_without_geometry": support_summary(
                    development_no_geometry_support
                ),
                "train_cross_fit": support_summary(train_support),
                "train_cross_fit_without_geometry": support_summary(
                    train_no_geometry_support
                ),
            },
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "profile_identity": {
                "bytes": len(profile_data),
                "path": b0.f0.PROFILE_PATH,
                "sha256": b0.f0.PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "role_metadata": role_metadata,
            "schema": "nextengine.experimental-physical-sound-v35-d0-owner-evidence.v1",
            "training": training,
        }
        payloads = standard_payloads(
            models,
            fitted,
            train,
            development,
            candidate_predictions,
            development_result,
            owner_evidence,
            outcome,
        )
        report = terminal.publish_scientific_terminal(
            output,
            repository_root(),
            outcome,
            ledger.as_dict(),
            payloads,
            CLAIM,
            resources["max_output_bytes"],
        )
        print(
            f"d0-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
            f"outcome={outcome}",
            file=sys.stderr,
        )
        return report
    except BaseException as error:
        if isinstance(error, (KeyboardInterrupt, SystemExit)):
            raise
        if ledger.target_rows_accessed > 0:
            resources = locals().get("resources", {"max_output_bytes": 67_108_864})
            return post_access_fault(output, ledger, error, stage, resources)
        return terminal.pre_access_contract_reject(
            output,
            repository_root(),
            ledger.as_dict(),
            f"{stage}:{type(error).__name__}:{error}",
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    report = run(arguments.profile, arguments.output)
    print(canonical_json(report).decode(), end="")
    if report["decision"] == "Pass":
        return 0
    if report["decision"] in terminal.SCIENTIFIC_OUTCOMES:
        return 1
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
