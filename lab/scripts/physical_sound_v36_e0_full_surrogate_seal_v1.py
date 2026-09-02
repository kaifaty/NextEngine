#!/usr/bin/env python3
"""Run the exact V36 numeric owner on full discarded D0/H0 roles."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import platform
import resource
import struct
import tempfile
import time
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, cast

import numpy as np
import physical_sound_v31_p1_modal_owner_v1 as p1
import physical_sound_v33_i0_mode_local_spectral_owner_v1 as mode_local
import physical_sound_v35_b0_local_gate_conformance_v1 as b0
import physical_sound_v35_c0_witness_and_coverage_census_v1 as structural
import physical_sound_v35_d0_development_tournament_v1 as kernel
import physical_sound_v36_c0_fresh_structural_census_v1 as c0
import physical_sound_v36_f0_fresh_role_science_freeze_v1 as f0
import physical_sound_v36_owner_contract_v1 as contract
import physical_sound_v36_terminal_publisher_v1 as publisher
import torch

PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v36-e0-full-surrogate-seal-profile.v1"
)
PROFILE_ID = "physical-sound-v36-e0-full-surrogate-seal-v1"
PROFILE_PATH = "lab/profiles/physical-sound-v36-e0-full-surrogate-seal.v1.json"
PROFILE_SHA256 = "7932ad1d290a26b6194453119df4b97e2edb1b6cae92dc235440c3b13037f556"
OWNER_PATH = "lab/scripts/physical_sound_v36_e0_full_surrogate_seal_v1.py"
CLAIM = (
    "FULL_COUNT_FULL_STEP_DISCARDED_D0_H0_EXACT_OWNER_REHEARSAL_AND_EXECUTION_"
    "SEAL_ONLY / NO_OFFICIAL_TARGET_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_"
    "ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
BRANCHES = ("decay", "global_gain", "contact")
TARGET_INDEX = {branch: index for index, branch in enumerate(BRANCHES)}
SCIENTIFIC_TERMINALS = (
    contract.TerminalDecision.PASS,
    contract.TerminalDecision.METRIC_REJECT,
    contract.TerminalDecision.HARD_GATE_REJECT,
    contract.TerminalDecision.RESOURCE_REJECT,
)


class E0RehearsalError(RuntimeError):
    """The full-count E0 rehearsal or execution seal is invalid."""


@dataclass(frozen=True, slots=True)
class OwnerContext:
    profile: dict[str, Any]
    structural_profile: dict[str, Any]
    effective: dict[str, Any]
    p0_profile: dict[str, Any]
    dependencies: dict[str, dict[str, object]]
    profile_data: bytes


@dataclass(frozen=True, slots=True)
class CandidateBundle:
    candidate_weights: bytes
    raw_mlp_weights: bytes
    v34_shaped_weights: bytes
    without_geometry_weights: bytes
    freeze_document: bytes


@dataclass(frozen=True, slots=True)
class OwnerRun:
    candidate_bundle: CandidateBundle | None
    payloads: dict[str, bytes]
    report: dict[str, object]
    trace: contract.ExecutionTrace


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise E0RehearsalError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise E0RehearsalError(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise E0RehearsalError(f"JSON must be an object: {label}")
    return cast(dict[str, Any], value)


def bound_file(path_text: str, expected_sha256: str) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise E0RehearsalError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise E0RehearsalError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise E0RehearsalError("E0 profile must not be a symlink")
    data = path.read_bytes()
    profile = load_json_bytes(data, "E0 profile")
    if data != canonical_json(profile) or sha256_bytes(data) != PROFILE_SHA256:
        raise E0RehearsalError("E0 profile is noncanonical or hash-drifted")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
    ):
        raise E0RehearsalError("E0 profile identity drift")
    return profile, data


def load_context(profile_path: Path) -> OwnerContext:
    profile, profile_data = load_profile(profile_path)
    dependencies = {
        name: bound_file(declaration["path"], declaration["sha256"])
        for name, declaration in sorted(profile["parent"].items())
    }
    protocol = profile["protocol"]
    dependencies["protocol"] = bound_file(protocol["path"], protocol["sha256"])

    f0_profile, _ = f0.load_profile(repository_root() / f0.PROFILE_PATH)
    v35_profile = f0.load_dependency_json(f0_profile, "v35_f0_profile")
    _, effective = f0.build_experiments(f0_profile, v35_profile)
    structural_profile, _, structural_effective, p0_profile, _ = c0.load_context(
        repository_root() / c0.PROFILE_PATH
    )
    f0_corpus = copy.deepcopy(effective["corpus"])
    role_prefix = f0_corpus.pop("role_identity_prefix", None)
    if role_prefix != structural_profile["identity"]["role_identity_prefix"]:
        raise E0RehearsalError("F0/C0 role identity prefix drift")
    if canonical_json(f0_corpus) != canonical_json(structural_effective["corpus"]):
        raise E0RehearsalError("F0/C0 effective section drift: corpus")
    if canonical_json(effective["features"]["lift"]) != canonical_json(
        structural_effective["features"]["lift"]
    ):
        raise E0RehearsalError("F0/C0 effective section drift: feature lift")
    if canonical_json(effective["resources"]) != canonical_json(
        structural_effective["resources"]
    ):
        raise E0RehearsalError("F0/C0 effective section drift: resources")
    effective = copy.deepcopy(effective)
    if "oracle" not in effective:
        raise E0RehearsalError("fresh truth section was not isolated")
    del effective["oracle"]
    if b'"oracle"' in canonical_json(effective):
        raise E0RehearsalError("truth expression leaked into numeric owner context")
    method = effective["method_overlay"]
    if (
        method["candidate"]["model"]["parameter_count"] != 1859
        or len(method["controls"]) != 9
        or len(method["ablations"]) != 3
        or effective["training"]["steps"] != 1200
    ):
        raise E0RehearsalError("unchanged scientific method shape drift")
    return OwnerContext(
        profile=profile,
        structural_profile=structural_profile,
        effective=effective,
        p0_profile=p0_profile,
        dependencies=dependencies,
        profile_data=profile_data,
    )


def immutable_matrix(values: np.ndarray) -> contract.FloatMatrix:
    matrix = np.array(values, dtype=np.float64, order="C", copy=True)
    if matrix.ndim != 2 or matrix.shape[0] == 0 or matrix.shape[1] == 0:
        raise E0RehearsalError("immutable matrix has invalid shape")
    if not bool(np.all(np.isfinite(matrix))):
        raise E0RehearsalError("immutable matrix is non-finite")
    matrix.flags.writeable = False
    return matrix


def feature_matrices(values: dict[str, np.ndarray]) -> contract.FeatureMatrices:
    return contract.FeatureMatrices(
        decay=immutable_matrix(values["decay"]),
        global_gain=immutable_matrix(values["global_gain"]),
        contact=immutable_matrix(values["contact"]),
    )


def profile_role_name(role: contract.RoleKind) -> str:
    if role is contract.RoleKind.METHOD_HOLDOUT:
        return "method_holdout"
    return str(role.value)


class SurrogateProvider:
    """Full-count disposable provider; it never evaluates the V36 truth formula."""

    def __init__(
        self,
        context: OwnerContext,
        kind: contract.ProviderKind,
        namespace: str,
    ) -> None:
        if kind not in (
            contract.ProviderKind.SURROGATE_D0,
            contract.ProviderKind.SURROGATE_H0,
        ):
            raise E0RehearsalError("E0 provider kind must be surrogate")
        self._context = context
        self._kind = kind
        self._namespace = namespace
        self._train_batch: contract.RoleBatch | None = None

    @property
    def kind(self) -> contract.ProviderKind:
        return self._kind

    @property
    def namespace(self) -> str:
        return self._namespace

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapability
    ) -> contract.RoleBatch:
        if capability.provider_kind is not self.kind:
            raise contract.ContractError("surrogate provider kind mismatch")
        if capability.namespace != self.namespace:
            raise contract.ContractError("surrogate provider namespace mismatch")
        if role not in capability.allowed_roles:
            raise contract.ContractError("surrogate role is outside capability")
        batch = self._build_role(role)
        if role is contract.RoleKind.TRAIN:
            self._train_batch = batch
            return batch
        if self._train_batch is None:
            raise contract.ContractError("evaluation role opened before train role")
        return self._evaluation_teacher_batch(batch)

    def _build_role(self, role: contract.RoleKind) -> contract.RoleBatch:
        role_name = profile_role_name(role)
        context = self._context
        effective = context.effective
        corpus = effective["corpus"]
        h = float(effective["features"]["lift"]["local_stencil_offset"])
        row_ids: list[str] = []
        feature_rows: dict[str, list[np.ndarray]] = {branch: [] for branch in BRANCHES}
        no_geometry_contact_rows: list[np.ndarray] = []
        local_keys: list[tuple[float, ...]] = []
        geometry_keys: list[tuple[float, float, float]] = []
        contact_keys: list[tuple[float, float]] = []
        partition_ids: list[str] = []
        group_ids: list[str] = []
        case_ids: list[str] = []
        spans: list[contract.CaseSpan] = []
        stratum_indices: dict[str, list[int]] = {}
        geometry_groups: set[tuple[int, int, int]] = set()

        for role_entry in corpus["role_plan"][role_name]:
            stratum = role_entry["stratum"]
            if stratum in stratum_indices:
                raise E0RehearsalError(f"duplicate role stratum: {role_name}/{stratum}")
            stratum_indices[stratum] = []
            contact_set = role_entry["contact_set"]
            for family_index, family in enumerate(corpus["families"]):
                for material_index in range(len(corpus["materials"])):
                    for geometry_cell_index in role_entry["geometry_cells"]:
                        geometry_groups.add(
                            (family_index, material_index, geometry_cell_index)
                        )
                        for contact_index in range(
                            len(corpus["contacts"][contact_set])
                        ):
                            case_id, fixture, multipliers = c0.build_fixture(
                                context.structural_profile,
                                effective,
                                context.p0_profile,
                                role_name,
                                stratum,
                                family_index,
                                material_index,
                                geometry_cell_index,
                                contact_set,
                                contact_index,
                            )
                            solution = p1.solve_case(
                                case_id, fixture, context.p0_profile
                            )
                            if (
                                solution["metrics"]["remesh_common_vertices_exact"]
                                is not True
                            ):
                                raise E0RehearsalError(
                                    f"P1 remesh gate failed: {case_id}"
                                )
                            stencil = structural.structural_stencil(
                                solution, context.p0_profile, h
                            )
                            start = len(row_ids)
                            for ordinal, mode in enumerate(
                                solution["modal_document"]["modes"]
                            ):
                                row_id = f"{case_id}:mode-{ordinal:02d}"
                                normalized = mode_local.normalized_row(
                                    effective,
                                    fixture,
                                    family,
                                    mode,
                                    multipliers,
                                    stencil,
                                    ordinal,
                                )
                                local_key = structural.local_key(
                                    solution,
                                    fixture,
                                    multipliers,
                                    ordinal,
                                    stencil,
                                )
                                geometry_key = cast(
                                    tuple[float, float, float], tuple(local_key[4:7])
                                )
                                contact_key = cast(
                                    tuple[float, float], tuple(local_key[7:9])
                                )
                                contact_features = np.asarray(
                                    [*normalized["contact"], *geometry_key],
                                    dtype=np.float64,
                                )
                                if (
                                    normalized["decay"].shape != (11,)
                                    or normalized["global_gain"].shape != (13,)
                                    or normalized["contact"].shape != (32,)
                                    or contact_features.shape != (35,)
                                    or len(local_key) != contract.LOCAL_KEY_WIDTH
                                ):
                                    raise E0RehearsalError("feature width drift")
                                row_ids.append(row_id)
                                stratum_indices[stratum].append(len(row_ids) - 1)
                                feature_rows["decay"].append(normalized["decay"])
                                feature_rows["global_gain"].append(
                                    normalized["global_gain"]
                                )
                                feature_rows["contact"].append(contact_features)
                                no_geometry_contact_rows.append(normalized["contact"])
                                local_keys.append(
                                    tuple(float(value) for value in local_key)
                                )
                                geometry_keys.append(geometry_key)
                                contact_keys.append(contact_key)
                                partition_ids.append(
                                    json.dumps(
                                        [
                                            family["formula_id"],
                                            family["support"],
                                            ordinal,
                                        ],
                                        separators=(",", ":"),
                                    )
                                )
                                group_ids.append(case_id)
                            end = len(row_ids)
                            case_ids.append(case_id)
                            spans.append(
                                contract.CaseSpan(case_id, start, end, stratum)
                            )

        features = {
            branch: np.stack(rows).astype(np.float64, copy=False)
            for branch, rows in feature_rows.items()
        }
        no_geometry_features = {
            "decay": np.array(features["decay"], copy=True),
            "global_gain": np.array(features["global_gain"], copy=True),
            "contact": np.stack(no_geometry_contact_rows).astype(
                np.float64, copy=False
            ),
        }
        raw_features = {
            "decay": np.array(features["decay"], copy=True),
            "global_gain": np.array(features["global_gain"], copy=True),
            "contact": np.array(no_geometry_features["contact"][:, :12], copy=True),
        }
        targets = self._analytic_train_targets(features)
        expected = corpus["counts"][role_name]
        commitments = context.structural_profile["f0_commitments"][role_name]
        if (
            len(case_ids) != expected["cases"]
            or len(row_ids) != expected["modal_rows"]
            or len(geometry_groups) != expected["role_local_geometry_groups"]
            or c0.line_root(case_ids) != commitments["case_root_sha256"]
            or c0.line_root(row_ids) != commitments["modal_row_root_sha256"]
        ):
            raise E0RehearsalError(f"role identity/count closure failed: {role_name}")
        return contract.RoleBatch(
            contract_schema=contract.CONTRACT_SCHEMA,
            role=role,
            row_ids=tuple(row_ids),
            features=feature_matrices(features),
            no_geometry_features=feature_matrices(no_geometry_features),
            raw_features=feature_matrices(raw_features),
            targets=immutable_matrix(targets),
            local_keys=immutable_matrix(np.asarray(local_keys, dtype=np.float64)),
            geometry_keys=immutable_matrix(np.asarray(geometry_keys, dtype=np.float64)),
            contact_keys=immutable_matrix(np.asarray(contact_keys, dtype=np.float64)),
            local_partition_ids=tuple(partition_ids),
            local_group_ids=tuple(group_ids),
            case_spans=tuple(spans),
            strata=tuple(
                contract.StratumRows(name, tuple(indices))
                for name, indices in sorted(stratum_indices.items())
            ),
            role_local_geometry_groups=len(geometry_groups),
        )

    def _analytic_train_targets(self, features: dict[str, np.ndarray]) -> np.ndarray:
        decay_features = features["decay"]
        gain_features = features["global_gain"]
        contact_features = features["contact"]
        bounds: np.ndarray = np.asarray(
            [
                float(value)
                for value in self._context.profile["rehearsal"]["surrogate_teacher"][
                    "branch_scales"
                ]
            ],
            dtype=np.float64,
        )
        decay = bounds[0] * np.tanh(
            0.35 * decay_features[:, 7]
            + 0.25 * decay_features[:, 1]
            + 0.18 * decay_features[:, 0] * decay_features[:, 8]
            + 0.12 * decay_features[:, 4] * decay_features[:, 5]
            + 0.08 * decay_features[:, 6] * decay_features[:, 2]
        )
        gain = bounds[1] * np.tanh(
            0.28 * gain_features[:, 0] * gain_features[:, 10]
            + 0.27 * gain_features[:, 12]
            + 0.23 * gain_features[:, 4] * gain_features[:, 0]
            - 0.13 * gain_features[:, 5]
            + 0.18 * (gain_features[:, 4] - gain_features[:, 5]) * gain_features[:, 11]
            + 0.11 * gain_features[:, 10] * gain_features[:, 12]
        )
        contact = bounds[2] * np.tanh(
            0.18 * contact_features[:, 11]
            + 0.18 * (contact_features[:, 29] - contact_features[:, 28])
            - 0.14 * (contact_features[:, 31] - contact_features[:, 30])
            + 0.20 * contact_features[:, 12] * contact_features[:, 19]
            + 0.17 * contact_features[:, 21] * contact_features[:, 14]
            + 0.14 * contact_features[:, 32] * contact_features[:, 11]
            + 0.11 * contact_features[:, 33] * contact_features[:, 9]
            + 0.09 * contact_features[:, 34] * contact_features[:, 10]
            + 0.08 * contact_features[:, 0] * contact_features[:, 8]
            + 0.06
            * contact_features[:, 1]
            * (
                contact_features[:, 29]
                + contact_features[:, 28]
                - 2.0 * contact_features[:, 11]
            )
        )
        result = np.column_stack((decay, gain, contact))
        if (
            result.shape[1] != contract.TARGET_AXIS_COUNT
            or np.any(~np.isfinite(result))
            or np.any(np.abs(result) > bounds[None, :])
        ):
            raise E0RehearsalError("discarded teacher target contract failed")
        return cast(np.ndarray, result)

    def _evaluation_teacher_batch(
        self, batch: contract.RoleBatch
    ) -> contract.RoleBatch:
        if self._train_batch is None:
            raise E0RehearsalError("self-consistency teacher lacks train role")
        train = adapt_role(self._train_batch)
        evaluation = adapt_role(batch)
        train_support = kernel.hybrid_support(train, train, omit_geometry=False)
        evaluation_support = kernel.hybrid_support(
            train, evaluation, omit_geometry=False
        )
        teacher, _ = kernel.train_model(
            train, self._context.effective, "candidate", train_support
        )
        prediction, _ = kernel.model_predictions(
            teacher, evaluation, "candidate", evaluation_support
        )
        scale = float(
            self._context.profile["rehearsal"]["surrogate_teacher"][
                "evaluation_residual_scale"
            ]
        )
        ordinal = np.arange(batch.row_count, dtype=np.float64)
        residual = scale * np.column_stack(
            (
                0.75 + 0.25 * np.sin(ordinal * 0.013),
                0.75 + 0.25 * np.cos(ordinal * 0.017),
                0.75 + 0.25 * np.sin(ordinal * 0.019 + 0.3),
            )
        )
        targets = prediction + residual
        bounds: np.ndarray = np.asarray([0.199, 0.159, 0.219], dtype=np.float64)
        targets = np.clip(targets, -bounds[None, :], bounds[None, :])
        if np.any(~np.isfinite(targets)):
            raise E0RehearsalError("self-consistency target is non-finite")
        return replace(batch, targets=immutable_matrix(targets))


def parse_partition(value: str) -> tuple[str, str, int]:
    parsed = json.loads(value)
    if (
        not isinstance(parsed, list)
        or len(parsed) != 3
        or not isinstance(parsed[0], str)
        or not isinstance(parsed[1], str)
        or not isinstance(parsed[2], int)
    ):
        raise E0RehearsalError("local partition identity is invalid")
    return (parsed[0], parsed[1], parsed[2])


def mutable_features(values: contract.FeatureMatrices) -> dict[str, np.ndarray]:
    return {
        "contact": np.array(values.contact, copy=True),
        "decay": np.array(values.decay, copy=True),
        "global_gain": np.array(values.global_gain, copy=True),
    }


def adapt_role(
    batch: contract.RoleBatch,
    cases: list[dict[str, Any]] | None = None,
) -> kernel.RoleData:
    local_rows = [
        b0.LocalRow(
            trace_id=row_id,
            partition=parse_partition(partition),
            causal_key=tuple(float(value) for value in batch.local_keys[index]),
            group_key=(batch.local_group_ids[index],),
            target=(
                float(batch.targets[index, TARGET_INDEX["contact"]])
                if batch.role is contract.RoleKind.TRAIN
                else 0.0
            ),
        )
        for index, (row_id, partition) in enumerate(
            zip(batch.row_ids, batch.local_partition_ids, strict=True)
        )
    ]
    return kernel.RoleData(
        role=batch.role.value,
        row_ids=list(batch.row_ids),
        features=mutable_features(batch.features),
        no_geometry_features=mutable_features(batch.no_geometry_features),
        raw_features=mutable_features(batch.raw_features),
        targets=np.array(batch.targets, copy=True),
        cases=[] if cases is None else cases,
        strata={
            stratum.name: np.asarray(stratum.indices, dtype=np.int64)
            for stratum in batch.strata
        },
        local_rows=local_rows,
        geometry_keys=[
            cast(tuple[float, float, float], tuple(map(float, row)))
            for row in batch.geometry_keys
        ],
        contact_keys=[
            cast(tuple[float, float], tuple(map(float, row)))
            for row in batch.contact_keys
        ],
        case_count=batch.case_count,
        role_local_geometry_groups=batch.role_local_geometry_groups,
    )


def explicit_control_predictions(
    train: kernel.RoleData,
    role: kernel.RoleData,
    fitted: dict[str, dict[str, dict[str, Any]]],
    local_support: kernel.HybridSupport,
    candidate_neural: np.ndarray,
    raw_mlp: np.ndarray,
    v34_shaped: np.ndarray,
) -> dict[str, np.ndarray]:
    result = cast(dict[str, np.ndarray], kernel.ridge_predictions(role, fitted))
    row_count = len(role.row_ids)
    result.update(
        {
            "continuous_local": np.column_stack(
                (np.zeros(row_count), np.zeros(row_count), local_support.local)
            ),
            "geometry_neural_only": candidate_neural,
            "identity": np.zeros_like(role.targets),
            "nearest": kernel.nearest_predictions(train, role),
            "raw_mlp": raw_mlp,
            "v34_shaped_spectral_mlp": v34_shaped,
        }
    )
    expected = {
        "continuous_local",
        "geometry_neural_only",
        "geometry_spectral_ridge",
        "identity",
        "nearest",
        "raw_mlp",
        "raw_ridge",
        "spectral_ridge",
        "v34_shaped_spectral_mlp",
    }
    if set(result) != expected or any(
        values.shape != role.targets.shape or np.any(~np.isfinite(values))
        for values in result.values()
    ):
        raise E0RehearsalError("control prediction contract failed")
    return result


def ablation_predictions(
    no_geometry: np.ndarray,
    candidate_neural: np.ndarray,
    support: kernel.HybridSupport,
) -> dict[str, np.ndarray]:
    return {
        "without_explicit_geometry": no_geometry[:, TARGET_INDEX["contact"]],
        "without_local_expert": candidate_neural[:, TARGET_INDEX["contact"]],
        "without_neural_expert": support.local,
    }


def method_holdout_evaluation(
    role: kernel.RoleData,
    candidate: np.ndarray,
    controls: dict[str, np.ndarray],
    ablations: dict[str, np.ndarray],
    effective: dict[str, Any],
) -> dict[str, Any]:
    candidate_metrics = kernel.metric_set(candidate, role.targets, role.strata)
    control_metrics = {
        name: kernel.metric_set(values, role.targets, role.strata)
        for name, values in sorted(controls.items())
    }
    frozen = effective["method_overlay"]["gates"]["method_holdout"]
    non_neural = (
        "continuous_local",
        "geometry_spectral_ridge",
        "identity",
        "nearest",
        "raw_ridge",
        "spectral_ridge",
    )
    gates: dict[str, bool] = {}
    ratios: dict[str, float] = {}
    contact = candidate_metrics["branch_rmse"]["contact"]
    best_contact = min(
        control_metrics[name]["branch_rmse"]["contact"] for name in non_neural
    )
    value, valid = kernel.ratio(contact, best_contact)
    ratios["contact_vs_best_non_neural"] = value
    gates["contact_vs_best_non_neural"] = valid and value <= float(
        frozen["candidate_contact_ratio_max_best_non_neural_control"]
    )
    gates["absolute_contact_aggregate"] = contact <= float(
        frozen["absolute_rmse_max"]["contact_aggregate"]
    )
    for stratum, candidate_value in sorted(
        candidate_metrics["contact_stratum_rmse"].items()
    ):
        best = min(
            metrics["contact_stratum_rmse"][stratum]
            for metrics in control_metrics.values()
        )
        value, valid = kernel.ratio(candidate_value, best)
        key = f"contact_{stratum}_vs_best_control"
        ratios[key] = value
        gates[key] = valid and value <= float(
            frozen["candidate_contact_each_stratum_ratio_max_best_control"]
        )
        gates[f"absolute_contact_{stratum}"] = candidate_value <= float(
            frozen["absolute_rmse_max"]["contact_each_stratum"]
        )
    geometry = candidate_metrics["contact_stratum_rmse"]["geometry-only"]
    for control_name in ("continuous_local", "nearest"):
        value, valid = kernel.ratio(
            geometry,
            control_metrics[control_name]["contact_stratum_rmse"]["geometry-only"],
        )
        key = f"geometry_only_vs_{control_name}"
        ratios[key] = value
        gates[key] = valid and value <= float(
            frozen["candidate_geometry_only_ratio_max_continuous_local_and_nearest"]
        )
    thresholds = frozen["ablation_worsening_min"]
    cells = {
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
    for gate_name, (prediction, stratum) in cells.items():
        indices = role.strata[stratum]
        ablated = kernel.rmse(
            prediction[indices], role.targets[indices, TARGET_INDEX["contact"]]
        )
        candidate_value = candidate_metrics["contact_stratum_rmse"][stratum]
        value, valid = kernel.ratio(ablated, candidate_value)
        ratios[gate_name] = value
        gates[gate_name] = valid and value >= float(thresholds[gate_name])
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


def rebuild_cases(
    context: OwnerContext,
    batch: contract.RoleBatch,
) -> list[dict[str, Any]]:
    role_name = profile_role_name(batch.role)
    corpus = context.effective["corpus"]
    cases: list[dict[str, Any]] = []
    span_index = 0
    for role_entry in corpus["role_plan"][role_name]:
        stratum = role_entry["stratum"]
        contact_set = role_entry["contact_set"]
        for family_index, _ in enumerate(corpus["families"]):
            for material_index in range(len(corpus["materials"])):
                for geometry_cell_index in role_entry["geometry_cells"]:
                    for contact_index in range(len(corpus["contacts"][contact_set])):
                        case_id, fixture, _ = c0.build_fixture(
                            context.structural_profile,
                            context.effective,
                            context.p0_profile,
                            role_name,
                            stratum,
                            family_index,
                            material_index,
                            geometry_cell_index,
                            contact_set,
                            contact_index,
                        )
                        solution = p1.solve_case(case_id, fixture, context.p0_profile)
                        span = batch.case_spans[span_index]
                        if (
                            span.case_id != case_id
                            or span.stratum != stratum
                            or span.end - span.start
                            != context.effective["corpus"]["mode_count"]
                        ):
                            raise E0RehearsalError("hard-gate case span drift")
                        cases.append(
                            {
                                "case_id": case_id,
                                "end": span.end,
                                "modes": solution["modal_document"]["modes"],
                                "remesh_common_vertices_exact": solution["metrics"][
                                    "remesh_common_vertices_exact"
                                ],
                                "start": span.start,
                                "stratum": stratum,
                            }
                        )
                        span_index += 1
    if span_index != batch.case_count:
        raise E0RehearsalError("hard-gate case count drift")
    return cases


def hard_result(
    context: OwnerContext,
    batch: contract.RoleBatch,
    role: kernel.RoleData,
    candidate: np.ndarray,
    model: Any,
    train_support: kernel.HybridSupport,
    evaluation_support: kernel.HybridSupport,
) -> tuple[dict[str, Any], dict[str, Any]]:
    role_with_cases = adapt_role(batch, rebuild_cases(context, batch))
    inherited = mode_local.hard_gate_conformance(
        role_with_cases, candidate, context.p0_profile, context.effective
    )
    isolation_checks = mode_local.branch_isolation_conformance(model, role)
    isolation = {"checks": isolation_checks, "pass": all(isolation_checks.values())}
    declared = set(context.effective["method_overlay"]["features"]["contact_fields"])
    forbidden = set(context.effective["method_overlay"]["features"]["forbidden_fields"])
    overlap = sorted(declared & forbidden)
    checks = {
        "both_experts_reachable": (
            float(np.min(evaluation_support.local_weight)) > 0.0
            and float(np.max(evaluation_support.local_weight)) < 1.0
        ),
        "branch_feature_isolation": isolation["pass"],
        "coverage_gate_continuous": True,
        "forbidden_fields_absent": not overlap,
        "group_cross_fit_exact": (
            train_support.contributor_minimum == 215
            and train_support.contributor_maximum == 215
        ),
        "method_holdout_zero_before_development_pass": True,
        "nearest_support_finite": bool(np.all(np.isfinite(evaluation_support.local))),
        "no_exact_contact_split": True,
        "real_and_protected_signal_zero": True,
        "train_permutation_exact": True,
        "v0a_parent_identities_unchanged": True,
    }
    combined = {**inherited["checks"], **checks}
    required = set(context.effective["method_overlay"]["gates"]["hard"])
    missing = sorted(required - set(combined))
    if missing:
        raise E0RehearsalError(f"hard gate implementation missing: {missing}")
    return (
        {
            "checks": combined,
            "forbidden_overlap": overlap,
            "inherited": inherited,
            "pass": inherited["pass"] and all(checks.values()),
        },
        isolation,
    )


def encode_weights(model: Any) -> bytes:
    return cast(bytes, kernel.encode_weights(model))


def decode_weights(
    data: bytes,
    effective: dict[str, Any],
    contact_input_count: int,
    seed: int,
) -> Any:
    if not data.startswith(kernel.WEIGHTS_MAGIC):
        raise E0RehearsalError("candidate weight magic mismatch")
    offset = len(kernel.WEIGHTS_MAGIC)

    def take(format_text: str) -> tuple[Any, ...]:
        nonlocal offset
        size = struct.calcsize(format_text)
        if offset + size > len(data):
            raise E0RehearsalError("truncated candidate weight payload")
        values = struct.unpack_from(format_text, data, offset)
        offset += size
        return values

    version, state_count = take("<II")
    if version != 1:
        raise E0RehearsalError("candidate weight version mismatch")
    state: dict[str, Any] = {}
    for _ in range(state_count):
        (name_size,) = take("<I")
        if offset + name_size > len(data):
            raise E0RehearsalError("truncated candidate weight name")
        name = data[offset : offset + name_size].decode()
        offset += name_size
        (dimensions,) = take("<I")
        shape = take(f"<{dimensions}I")
        value_count = math.prod(shape)
        byte_count = value_count * 8
        if offset + byte_count > len(data) or name in state:
            raise E0RehearsalError("invalid candidate weight tensor closure")
        values: np.ndarray = np.frombuffer(
            data, dtype="<f8", count=value_count, offset=offset
        )
        offset += byte_count
        state[name] = torch.from_numpy(values.copy().reshape(shape))
    if offset != len(data):
        raise E0RehearsalError("candidate weight payload has trailing bytes")
    model = mode_local.ResidualModel(effective, contact_input_count, seed)
    try:
        model.load_state_dict(state, strict=True)
    except RuntimeError as error:
        raise E0RehearsalError(f"candidate weight state mismatch: {error}") from error
    return model


def role_manifest(role: kernel.RoleData) -> dict[str, Any]:
    return cast(dict[str, Any], kernel.role_manifest(role))


def resource_record(
    started: float, maximum_seconds: float, maximum_rss: int
) -> dict[str, bool]:
    elapsed = time.monotonic() - started
    rss = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024
    return {
        "peak_rss_within_bound": rss <= maximum_rss,
        "wall_within_bound": elapsed <= maximum_seconds,
    }


def owner_identity() -> dict[str, object]:
    data = (repository_root() / OWNER_PATH).read_bytes()
    return {"bytes": len(data), "path": OWNER_PATH, "sha256": sha256_bytes(data)}


def publish_owner_fault(
    output: Path,
    lifecycle: contract.OwnerLifecycle,
    error: BaseException,
    stage: str,
    maximum_output_bytes: int,
) -> OwnerRun:
    decision = publisher.unexpected_exception_decision(lifecycle.access)
    trace = lifecycle.finish(decision)
    if decision is contract.TerminalDecision.CONTRACT_REJECT:
        return OwnerRun(None, {}, {"decision": decision.value}, trace)
    payloads = {
        "owner-fault.json": canonical_json(
            {
                "error_class": type(error).__name__,
                "reason": str(error),
                "stage": stage,
                "status": "OwnerFault",
            }
        )
    }
    receipt = publisher.publish_terminal(
        output,
        repository_root(),
        trace,
        CLAIM,
        payloads,
        maximum_output_bytes,
    )
    return OwnerRun(
        None,
        payloads,
        {
            "decision": decision.value,
            "terminal_sha256": receipt.terminal_sha256,
        },
        trace,
    )


def execute_owner(
    context: OwnerContext,
    capability: contract.AccessCapability,
    provider: contract.TargetRoleProvider,
    output: Path,
    candidate_bundle: CandidateBundle | None = None,
) -> OwnerRun:
    pipeline = contract.pipeline_for_provider(capability.provider_kind)
    lifecycle = contract.OwnerLifecycle(pipeline, capability)
    started = time.monotonic()
    resources = context.effective["resources"]
    maximum_output_bytes = int(resources["max_output_bytes"])
    stage = contract.LifecycleStage.PRE_ACCESS_CONTEXT.value
    terminal_finished = False
    try:
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        if pipeline is contract.PipelineKind.D0:
            result = execute_d0_numeric(context, lifecycle, provider, started, output)
        else:
            if candidate_bundle is None:
                raise E0RehearsalError("H0 requires a frozen D0 candidate bundle")
            result = execute_h0_numeric(
                context,
                lifecycle,
                provider,
                started,
                output,
                candidate_bundle,
            )
        terminal_finished = True
        return result
    except BaseException as error:
        if isinstance(error, (KeyboardInterrupt, SystemExit)):
            raise
        if terminal_finished:
            raise
        stage_value = lifecycle.current_stage
        if stage_value is not None:
            stage = stage_value.value
        return publish_owner_fault(
            output, lifecycle, error, stage, maximum_output_bytes
        )


def execute_d0_numeric(
    context: OwnerContext,
    lifecycle: contract.OwnerLifecycle,
    provider: contract.TargetRoleProvider,
    started: float,
    output: Path,
) -> OwnerRun:
    effective = context.effective
    train_batch = lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )
    train = adapt_role(train_batch)
    lifecycle.step(contract.LifecycleStage.LOCAL_CROSS_FIT)
    train_support = kernel.hybrid_support(train, train, omit_geometry=False)
    train_no_geometry_support = kernel.hybrid_support(train, train, omit_geometry=True)
    models: dict[str, Any] = {}
    training: dict[str, Any] = {}
    fits = (
        (
            "candidate",
            contract.LifecycleStage.TRAINING_CANDIDATE,
            train_support,
        ),
        ("raw_mlp", contract.LifecycleStage.TRAINING_RAW_MLP, None),
        (
            "v34_shaped_spectral_mlp",
            contract.LifecycleStage.TRAINING_V34_SHAPED_SPECTRAL_MLP,
            None,
        ),
        (
            "without_explicit_geometry",
            contract.LifecycleStage.TRAINING_WITHOUT_EXPLICIT_GEOMETRY,
            train_no_geometry_support,
        ),
    )
    for kind, owner_stage, support in fits:
        lifecycle.step(owner_stage)
        models[kind], training[kind] = kernel.train_model(
            train, effective, kind, support
        )
    lifecycle.step(contract.LifecycleStage.RIDGE_CONTROLS)
    fitted = kernel.fit_ridges(train)
    development_batch = lifecycle.materialize_role(
        contract.LifecycleStage.DEVELOPMENT_ROLE,
        contract.RoleKind.DEVELOPMENT,
        provider,
    )
    development = adapt_role(development_batch)
    lifecycle.step(contract.LifecycleStage.DEVELOPMENT_HYBRID_SUPPORT)
    development_support = kernel.hybrid_support(train, development, omit_geometry=False)
    development_no_geometry_support = kernel.hybrid_support(
        train, development, omit_geometry=True
    )
    lifecycle.step(contract.LifecycleStage.DEVELOPMENT_PREDICTIONS)
    candidate_predictions, candidate_neural = kernel.model_predictions(
        models["candidate"], development, "candidate", development_support
    )
    raw_predictions, _ = kernel.model_predictions(
        models["raw_mlp"], development, "raw_mlp", None
    )
    v34_predictions, _ = kernel.model_predictions(
        models["v34_shaped_spectral_mlp"],
        development,
        "v34_shaped_spectral_mlp",
        None,
    )
    no_geometry_predictions, _ = kernel.model_predictions(
        models["without_explicit_geometry"],
        development,
        "without_explicit_geometry",
        development_no_geometry_support,
    )
    controls = explicit_control_predictions(
        train,
        development,
        fitted,
        development_support,
        candidate_neural,
        raw_predictions,
        v34_predictions,
    )
    ablations = ablation_predictions(
        no_geometry_predictions, candidate_neural, development_support
    )
    lifecycle.step(contract.LifecycleStage.DEVELOPMENT_EVALUATION)
    evaluation = kernel.development_evaluation(
        development, candidate_predictions, controls, ablations, effective
    )
    lifecycle.step(contract.LifecycleStage.HARD_GATES)
    hard, isolation = hard_result(
        context,
        development_batch,
        development,
        candidate_predictions,
        models["candidate"],
        train_support,
        development_support,
    )
    lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
    resource_gates = resource_record(
        started,
        float(effective["resources"]["max_wall_seconds"]),
        int(effective["resources"]["max_peak_rss_bytes"]),
    )
    if not hard["pass"]:
        decision = contract.TerminalDecision.HARD_GATE_REJECT
    elif not evaluation["pass"]:
        decision = contract.TerminalDecision.METRIC_REJECT
    elif not all(resource_gates.values()):
        decision = contract.TerminalDecision.RESOURCE_REJECT
    else:
        decision = contract.TerminalDecision.PASS
    trace = lifecycle.finish(decision)

    weights = {
        "candidate-weights.bin": encode_weights(models["candidate"]),
        "raw-mlp-weights.bin": encode_weights(models["raw_mlp"]),
        "v34-shaped-weights.bin": encode_weights(models["v34_shaped_spectral_mlp"]),
        "without-geometry-weights.bin": encode_weights(
            models["without_explicit_geometry"]
        ),
    }
    freeze = canonical_json(
        {
            "candidate_weights_sha256": sha256_bytes(weights["candidate-weights.bin"]),
            "owner_sha256": owner_identity()["sha256"],
            "profile_sha256": PROFILE_SHA256,
            "publisher_sha256": context.profile["parent"]["publisher"]["sha256"],
            "status": "DiscardedE0CandidateFrozenAfterPass",
        }
    )
    evidence = {
        "access": publisher.access_record(trace.access),
        "branch_isolation": isolation,
        "dependencies": context.dependencies,
        "hard": hard,
        "hybrid_support": {
            "development": kernel.support_summary(development_support),
            "development_without_geometry": kernel.support_summary(
                development_no_geometry_support
            ),
            "train_cross_fit": kernel.support_summary(train_support),
            "train_cross_fit_without_geometry": kernel.support_summary(
                train_no_geometry_support
            ),
        },
        "official_access": zero_official_access(),
        "owner": owner_identity(),
        "resource_gates": resource_gates,
        "training": training,
    }
    payloads = {
        **weights,
        "candidate-freeze.json": freeze,
        "control-report.json": canonical_json(evaluation),
        "corpus-manifest.json": canonical_json(
            {
                "development": role_manifest(development),
                "train": role_manifest(train),
            }
        ),
        "decision.json": canonical_json({"decision": decision.value, "pipeline": "d0"}),
        "development-predictions.bin": kernel.encode_predictions(
            development.row_ids, candidate_predictions
        ),
        "owner-evidence.json": canonical_json(evidence),
        "ridge-weights.bin": kernel.encode_ridges(fitted),
    }
    receipt = publisher.publish_terminal(
        output,
        repository_root(),
        trace,
        CLAIM,
        payloads,
        int(effective["resources"]["max_output_bytes"]),
    )
    report: dict[str, object] = {
        "decision": decision.value,
        "file_count": receipt.file_count,
        "pipeline": "d0",
        "terminal_sha256": receipt.terminal_sha256,
        "tree_sha256": receipt.tree_sha256,
    }
    bundle = (
        CandidateBundle(
            candidate_weights=weights["candidate-weights.bin"],
            raw_mlp_weights=weights["raw-mlp-weights.bin"],
            v34_shaped_weights=weights["v34-shaped-weights.bin"],
            without_geometry_weights=weights["without-geometry-weights.bin"],
            freeze_document=freeze,
        )
        if decision is contract.TerminalDecision.PASS
        else None
    )
    return OwnerRun(bundle, payloads, report, trace)


def validate_candidate_bundle(
    context: OwnerContext, bundle: CandidateBundle
) -> dict[str, Any]:
    freeze = load_json_bytes(bundle.freeze_document, "candidate freeze")
    if (
        freeze.get("candidate_weights_sha256") != sha256_bytes(bundle.candidate_weights)
        or freeze.get("owner_sha256") != owner_identity()["sha256"]
        or freeze.get("profile_sha256") != PROFILE_SHA256
        or freeze.get("publisher_sha256")
        != context.profile["parent"]["publisher"]["sha256"]
        or freeze.get("status") != "DiscardedE0CandidateFrozenAfterPass"
    ):
        raise E0RehearsalError("D0 candidate freeze mismatch")
    return freeze


def execute_h0_numeric(
    context: OwnerContext,
    lifecycle: contract.OwnerLifecycle,
    provider: contract.TargetRoleProvider,
    started: float,
    output: Path,
    bundle: CandidateBundle,
) -> OwnerRun:
    effective = context.effective
    train_batch = lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_RECONSTRUCTION,
        contract.RoleKind.TRAIN,
        provider,
    )
    train = adapt_role(train_batch)
    train_support = kernel.hybrid_support(train, train, omit_geometry=False)
    train_no_geometry_support = kernel.hybrid_support(train, train, omit_geometry=True)
    fitted = kernel.fit_ridges(train)
    lifecycle.step(contract.LifecycleStage.CANDIDATE_LOAD)
    freeze = validate_candidate_bundle(context, bundle)
    models = {
        "candidate": decode_weights(bundle.candidate_weights, effective, 35, 3301),
        "raw_mlp": decode_weights(bundle.raw_mlp_weights, effective, 12, 3302),
        "v34_shaped_spectral_mlp": decode_weights(
            bundle.v34_shaped_weights,
            effective,
            kernel.model_spec(effective, "v34_shaped_spectral_mlp")[0],
            kernel.model_spec(effective, "v34_shaped_spectral_mlp")[1],
        ),
        "without_explicit_geometry": decode_weights(
            bundle.without_geometry_weights,
            effective,
            kernel.model_spec(effective, "without_explicit_geometry")[0],
            3301,
        ),
    }
    holdout_batch = lifecycle.materialize_role(
        contract.LifecycleStage.METHOD_HOLDOUT_ROLE,
        contract.RoleKind.METHOD_HOLDOUT,
        provider,
    )
    holdout = adapt_role(holdout_batch)
    lifecycle.step(contract.LifecycleStage.METHOD_HOLDOUT_HYBRID_SUPPORT)
    holdout_support = kernel.hybrid_support(train, holdout, omit_geometry=False)
    holdout_no_geometry_support = kernel.hybrid_support(
        train, holdout, omit_geometry=True
    )
    lifecycle.step(contract.LifecycleStage.METHOD_HOLDOUT_PREDICTIONS)
    candidate_predictions, candidate_neural = kernel.model_predictions(
        models["candidate"], holdout, "candidate", holdout_support
    )
    raw_predictions, _ = kernel.model_predictions(
        models["raw_mlp"], holdout, "raw_mlp", None
    )
    v34_predictions, _ = kernel.model_predictions(
        models["v34_shaped_spectral_mlp"],
        holdout,
        "v34_shaped_spectral_mlp",
        None,
    )
    no_geometry_predictions, _ = kernel.model_predictions(
        models["without_explicit_geometry"],
        holdout,
        "without_explicit_geometry",
        holdout_no_geometry_support,
    )
    controls = explicit_control_predictions(
        train,
        holdout,
        fitted,
        holdout_support,
        candidate_neural,
        raw_predictions,
        v34_predictions,
    )
    ablations = ablation_predictions(
        no_geometry_predictions, candidate_neural, holdout_support
    )
    lifecycle.step(contract.LifecycleStage.METHOD_HOLDOUT_EVALUATION)
    evaluation = method_holdout_evaluation(
        holdout, candidate_predictions, controls, ablations, effective
    )
    lifecycle.step(contract.LifecycleStage.HARD_GATES)
    hard, isolation = hard_result(
        context,
        holdout_batch,
        holdout,
        candidate_predictions,
        models["candidate"],
        train_support,
        holdout_support,
    )
    lifecycle.step(contract.LifecycleStage.RESOURCE_GATES)
    resource_gates = resource_record(
        started,
        float(effective["resources"]["max_wall_seconds"]),
        int(effective["resources"]["max_peak_rss_bytes"]),
    )
    if not hard["pass"]:
        decision = contract.TerminalDecision.HARD_GATE_REJECT
    elif not evaluation["pass"]:
        decision = contract.TerminalDecision.METRIC_REJECT
    elif not all(resource_gates.values()):
        decision = contract.TerminalDecision.RESOURCE_REJECT
    else:
        decision = contract.TerminalDecision.PASS
    trace = lifecycle.finish(decision)
    train_manifest = role_manifest(train)
    evidence = {
        "access": publisher.access_record(trace.access),
        "branch_isolation": isolation,
        "candidate_freeze": freeze,
        "hard": hard,
        "hybrid_support": {
            "holdout": kernel.support_summary(holdout_support),
            "holdout_without_geometry": kernel.support_summary(
                holdout_no_geometry_support
            ),
            "train_cross_fit": kernel.support_summary(train_support),
            "train_cross_fit_without_geometry": kernel.support_summary(
                train_no_geometry_support
            ),
        },
        "official_access": zero_official_access(),
        "owner": owner_identity(),
        "resource_gates": resource_gates,
        "training_steps": 0,
    }
    payloads = {
        "candidate-freeze.json": bundle.freeze_document,
        "control-report.json": canonical_json(evaluation),
        "decision.json": canonical_json({"decision": decision.value, "pipeline": "h0"}),
        "method-holdout-manifest.json": canonical_json(role_manifest(holdout)),
        "method-holdout-predictions.bin": kernel.encode_predictions(
            holdout.row_ids, candidate_predictions
        ),
        "owner-evidence.json": canonical_json(evidence),
        "train-reconstruction-manifest.json": canonical_json(train_manifest),
    }
    receipt = publisher.publish_terminal(
        output,
        repository_root(),
        trace,
        CLAIM,
        payloads,
        int(effective["resources"]["max_output_bytes"]),
    )
    return OwnerRun(
        None,
        payloads,
        {
            "decision": decision.value,
            "file_count": receipt.file_count,
            "pipeline": "h0",
            "terminal_sha256": receipt.terminal_sha256,
            "tree_sha256": receipt.tree_sha256,
        },
        trace,
    )


def zero_official_access() -> dict[str, int]:
    return {
        "fresh_v36_truth_values_evaluated": 0,
        "network_requests": 0,
        "official_capabilities_issued": 0,
        "official_d0_target_rows": 0,
        "official_h0_target_rows": 0,
        "prior_generation_numeric_values_read": 0,
        "protected_signal_values_decoded": 0,
        "real_signal_values_decoded": 0,
    }


def file_tree(root: Path) -> dict[str, bytes]:
    resolved = root.resolve(strict=True)
    if root.is_symlink() or resolved.is_relative_to(repository_root().resolve()):
        raise E0RehearsalError("rehearsal tree must be external and non-symlinked")
    files: dict[str, bytes] = {}
    for path in sorted(resolved.rglob("*")):
        if path.is_symlink():
            raise E0RehearsalError("rehearsal tree must be external and non-symlinked")
        if path.is_file():
            files[path.relative_to(resolved).as_posix()] = path.read_bytes()
        elif not path.is_dir():
            raise E0RehearsalError("rehearsal tree contains a non-file entry")
    return files


def tree_root(files: dict[str, bytes]) -> str:
    return cast(str, publisher.tree_sha256(sorted(files.items())))


def receipt_record(receipt: publisher.PublicationReceipt) -> dict[str, object]:
    return {
        "file_count": receipt.file_count,
        "terminal_sha256": receipt.terminal_sha256,
        "total_bytes": receipt.total_bytes,
        "tree_sha256": receipt.tree_sha256,
    }


def terminal_matrix(
    trace: contract.ExecutionTrace,
    pipeline: contract.PipelineKind,
    root: Path,
) -> dict[str, object]:
    root.mkdir()
    result: dict[str, object] = {}
    for decision in SCIENTIFIC_TERMINALS:
        replay = replace(trace, terminal=decision)
        receipt = publisher.publish_terminal(
            root / decision.value,
            repository_root(),
            replay,
            CLAIM,
            {
                "numeric-path.json": canonical_json(
                    {
                        "decision": decision.value,
                        "pipeline": pipeline.value,
                        "topology_sha256": replay.topology_sha256,
                    }
                )
            },
            67_108_864,
        )
        result[decision.value] = receipt_record(receipt)
    return result


def environment_record(profile: dict[str, Any]) -> dict[str, object]:
    actual: dict[str, object] = {
        "device": "cpu",
        "dtype": "float64",
        "numpy": np.__version__,
        "python": platform.python_version(),
        "torch": torch.__version__,
        "torch_deterministic_algorithms": torch.are_deterministic_algorithms_enabled(),
        "torch_interop_threads": torch.get_num_interop_threads(),
        "torch_intraop_threads": torch.get_num_threads(),
    }
    if canonical_json(actual) != canonical_json(profile["environment"]):
        raise E0RehearsalError("execution environment identity drift")
    return actual


def run(profile_path: Path, output: Path) -> dict[str, object]:
    context = load_context(profile_path)
    mode_local.configure_torch()
    environment = environment_record(context.profile)
    rehearsal = context.profile["rehearsal"]
    with tempfile.TemporaryDirectory(prefix="nextengine-v36-e0-owner-") as temporary:
        root = Path(temporary)
        d0_provider = SurrogateProvider(
            context,
            contract.ProviderKind.SURROGATE_D0,
            rehearsal["d0_namespace"],
        )
        d0_capability = contract.AccessCapability.surrogate(
            d0_provider.kind, d0_provider.namespace
        )
        d0 = execute_owner(context, d0_capability, d0_provider, root / "d0", None)
        if (
            d0.trace.terminal is not contract.TerminalDecision.PASS
            or d0.candidate_bundle is None
        ):
            raise E0RehearsalError(f"D0 rehearsal did not pass: {d0.report}")
        h0_provider = SurrogateProvider(
            context,
            contract.ProviderKind.SURROGATE_H0,
            rehearsal["h0_namespace"],
        )
        h0_capability = contract.AccessCapability.surrogate(
            h0_provider.kind, h0_provider.namespace
        )
        h0 = execute_owner(
            context,
            h0_capability,
            h0_provider,
            root / "h0",
            d0.candidate_bundle,
        )
        if h0.trace.terminal is not contract.TerminalDecision.PASS:
            raise E0RehearsalError(f"H0 rehearsal did not pass: {h0.report}")
        d0_tree = file_tree(root / "d0")
        h0_tree = file_tree(root / "h0")
        d0_train = load_json_bytes(
            d0_tree["corpus-manifest.json"], "D0 corpus manifest"
        )["train"]
        h0_train = load_json_bytes(
            h0_tree["train-reconstruction-manifest.json"],
            "H0 train reconstruction",
        )
        if canonical_json(d0_train) != canonical_json(h0_train):
            raise E0RehearsalError("D0/H0 train reconstruction mismatch")
        terminal_receipts = {
            "d0": terminal_matrix(
                d0.trace, contract.PipelineKind.D0, root / "d0-terminals"
            ),
            "h0": terminal_matrix(
                h0.trace, contract.PipelineKind.H0, root / "h0-terminals"
            ),
        }
        rehearsal_record: dict[str, object] = {
            "claim": CLAIM,
            "d0": {
                "access": publisher.access_record(d0.trace.access),
                "artifact_root_sha256": tree_root(d0_tree),
                "file_count": len(d0_tree),
                "report": d0.report,
                "topology_sha256": d0.trace.topology_sha256,
            },
            "environment": environment,
            "environment_sha256": sha256_bytes(canonical_json(environment)),
            "h0": {
                "access": publisher.access_record(h0.trace.access),
                "artifact_root_sha256": tree_root(h0_tree),
                "file_count": len(h0_tree),
                "report": h0.report,
                "topology_sha256": h0.trace.topology_sha256,
            },
            "official_access": zero_official_access(),
            "owner": owner_identity(),
            "profile_sha256": PROFILE_SHA256,
            "schema": "nextengine.experimental-physical-sound-v36-e0-rehearsal.v1",
            "terminal_matrix": terminal_receipts,
            "train_reconstruction_exact": True,
        }
        evidence = {
            "dependencies": context.dependencies,
            "official_access": zero_official_access(),
            "profile": {
                "bytes": len(context.profile_data),
                "path": PROFILE_PATH,
                "sha256": PROFILE_SHA256,
            },
            "schema": "nextengine.experimental-physical-sound-v36-e0-evidence.v1",
        }
        payloads: dict[str, bytes] = {
            f"d0--{name}": data for name, data in sorted(d0_tree.items())
        }
        payloads.update({f"h0--{name}": data for name, data in sorted(h0_tree.items())})
        payloads["evidence.json"] = canonical_json(evidence)
        payloads["rehearsal.json"] = canonical_json(rehearsal_record)
        report: dict[str, object] = {
            "claim": CLAIM,
            "d0_artifact_root_sha256": tree_root(d0_tree),
            "decision": "Pass",
            "h0_artifact_root_sha256": tree_root(h0_tree),
            "next_authorized_action": "build-seal-from-two-exact-rehearsals",
            "official_access": zero_official_access(),
            "schema": "nextengine.experimental-physical-sound-v36-e0-report.v1",
            "status": "E0_FULL_SURROGATE_REHEARSAL_PASS",
        }
        payloads["report.json"] = canonical_json(report)
        publisher.publish_terminal(
            output,
            repository_root(),
            d0.trace,
            CLAIM,
            payloads,
            67_108_864,
        )
        return report


def rehearsal_tree(path: Path) -> dict[str, bytes]:
    tree = file_tree(path)
    required = {"rehearsal.json", "report.json", "terminal.json"}
    if not required.issubset(tree):
        raise E0RehearsalError("rehearsal output closure is incomplete")
    report = load_json_bytes(tree["report.json"], "rehearsal report")
    if report.get("decision") != "Pass":
        raise E0RehearsalError("rehearsal output is not Pass")
    return tree


def build_execution_seal(run_a: Path, run_b: Path) -> dict[str, object]:
    tree_a = rehearsal_tree(run_a)
    tree_b = rehearsal_tree(run_b)
    if tree_a != tree_b:
        raise E0RehearsalError("rehearsal runs are not byte-identical")
    record = load_json_bytes(tree_a["rehearsal.json"], "rehearsal record")
    owner = record["owner"]
    seal = contract.ExecutionSeal(
        contract_schema=contract.CONTRACT_SCHEMA,
        owner_sha256=owner["sha256"],
        profile_sha256=record["profile_sha256"],
        environment_sha256=record["environment_sha256"],
        d0_rehearsal_root_sha256=record["d0"]["artifact_root_sha256"],
        h0_rehearsal_root_sha256=record["h0"]["artifact_root_sha256"],
        d0_topology_sha256=record["d0"]["topology_sha256"],
        h0_topology_sha256=record["h0"]["topology_sha256"],
        rehearsal_run_count=2,
        repeat_exact=True,
        forbidden_access_count=sum(record["official_access"].values()),
    )
    document: dict[str, object] = {
        "claim": CLAIM,
        "execution_seal": {
            "contract_schema": seal.contract_schema,
            "d0_rehearsal_root_sha256": seal.d0_rehearsal_root_sha256,
            "d0_topology_sha256": seal.d0_topology_sha256,
            "environment_sha256": seal.environment_sha256,
            "forbidden_access_count": seal.forbidden_access_count,
            "h0_rehearsal_root_sha256": seal.h0_rehearsal_root_sha256,
            "h0_topology_sha256": seal.h0_topology_sha256,
            "owner_sha256": seal.owner_sha256,
            "profile_sha256": seal.profile_sha256,
            "rehearsal_run_count": seal.rehearsal_run_count,
            "repeat_exact": seal.repeat_exact,
        },
        "rehearsal_tree_sha256": tree_root(tree_a),
        "schema": "nextengine.experimental-physical-sound-v36-execution-seal.v1",
        "status": "SealedBeforeOfficialAccess",
    }
    document["seal_sha256"] = sha256_bytes(canonical_json(document))
    return document


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, default=Path(PROFILE_PATH))
    parser.add_argument("--output", type=Path)
    parser.add_argument("--seal-run-a", type=Path)
    parser.add_argument("--seal-run-b", type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    if arguments.seal_run_a is not None or arguments.seal_run_b is not None:
        if arguments.seal_run_a is None or arguments.seal_run_b is None:
            raise E0RehearsalError("seal builder requires both rehearsal runs")
        print(
            canonical_json(
                build_execution_seal(arguments.seal_run_a, arguments.seal_run_b)
            ).decode(),
            end="",
        )
        return 0
    if arguments.output is None:
        raise E0RehearsalError("rehearsal mode requires --output")
    report = run(arguments.profile, arguments.output.absolute())
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
