#!/usr/bin/env python3
"""Open the V34 method holdout once with the frozen D0 candidate and controls."""

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
import physical_sound_v34_d0_development_tournament_v1 as d0
import physical_sound_v34_terminal_publication_v1 as terminal

OWNER_PATH = "lab/scripts/physical_sound_v34_h0_method_holdout_v1.py"
D0_OWNER_PATH = "lab/scripts/physical_sound_v34_d0_development_tournament_v1.py"
D0_OWNER_SHA256 = "1b0f0968e693fc0716861989f2ccd89407864e2912a15cb45658bde6d47e6add"
D0_RESULT_PATH = (
    "docs/development/physical-sound-v34-d0-development-tournament-result-2026-09-02.md"
)
D0_RESULT_SHA256 = "330f356c95749528a604a95db1932138d310cf1de75dad25661cce320bd1de24"
CLAIM = (
    "SYNTHETIC_FROZEN_CANDIDATE_ONE_SHOT_METHOD_HOLDOUT_ONLY / "
    "NO_RETRAINING_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
ALLOWED_ROLES = ("train", "method_holdout")
D0_ARTIFACT_SHA256 = {
    "candidate-freeze.json": "2bf800a99c8d9a42e2555c6fa818c6ed4149ee23dcf380e7ec3e10c616d605f7",
    "candidate-weights.bin": "831fd0a78d0d470f6ba54605d52eb0cff0b88aac483b1a35de4564f679fd80bd",
    "control-report.json": "04dd843aa9e6112fe1f14691ed8649541074a9e300358de9df96142956fb0a70",
    "corpus-manifest.json": "20c1555f2320a69dc68695589edda072edd0315cdc664c64ea2124b3d15fb6d4",
    "d0-decision.json": "ae74ccc23e6f973afc9e7aad5580d717b84915630072133f8560a5f307ecdeca",
    "development-predictions.bin": "e71f7de4bf58e8888fc8eec1506c1ddd2bed14868420184a1c64dd0d07aed0b2",
    "evidence.json": "7770efe58a83245dd7a80a86aa1a1af03a1d4651859cacb1a7085c6233bcdcd4",
    "owner-evidence.json": "db5b3cd41c56a0c246d2ece78de704ed5933a3fcb0b511a9a0bd3e1bb59c5e0b",
    "raw-mlp-control-weights.bin": "220bf5c18b3436030771b97bb05865eeea0f904d1e825da986e07528ec9f3ba6",
    "report.json": "e4fd015c02b755078f6b5597b2f919d30b6205bd91176af3ceb9d75c68807b83",
}


class H0HoldoutError(RuntimeError):
    """The one-shot method-holdout owner cannot return a valid terminal."""


@dataclass
class AccessLedger:
    access_phase: str = "pre-access"
    disclosed_train_target_rows_rebuilt: int = 0
    feature_rows_materialized: int = 0
    method_holdout_target_rows: int = 0
    model_parameters_loaded: int = 0
    model_training_steps: int = 0
    network_requests: int = 0
    protected_signal_values_decoded: int = 0
    real_signal_values_decoded: int = 0
    target_rows_accessed: int = 0

    def record_row(self, role: str) -> None:
        if role not in ALLOWED_ROLES:
            raise H0HoldoutError(f"H0 role access forbidden: {role}")
        self.access_phase = "post-access"
        self.feature_rows_materialized += 1
        self.target_rows_accessed += 1
        if role == "train":
            self.disclosed_train_target_rows_rebuilt += 1
        else:
            self.method_holdout_target_rows += 1

    def as_dict(self) -> dict[str, Any]:
        return {
            "access_phase": self.access_phase,
            "disclosed_train_target_rows_rebuilt": self.disclosed_train_target_rows_rebuilt,
            "feature_rows_materialized": self.feature_rows_materialized,
            "method_holdout_target_rows": self.method_holdout_target_rows,
            "model_parameters_loaded": self.model_parameters_loaded,
            "model_training_steps": self.model_training_steps,
            "network_requests": self.network_requests,
            "protected_signal_values_decoded": self.protected_signal_values_decoded,
            "real_signal_values_decoded": self.real_signal_values_decoded,
            "target_rows_accessed": self.target_rows_accessed,
        }


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise H0HoldoutError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_d0_artifacts(path: Path) -> tuple[dict[str, bytes], dict[str, Any]]:
    resolved = path.resolve(strict=True)
    if (
        path.is_symlink()
        or not resolved.is_dir()
        or resolved.is_relative_to(repository_root().resolve(strict=True))
    ):
        raise H0HoldoutError("D0 artifacts must be an external non-symlink directory")
    files = {item.name: item for item in resolved.iterdir() if item.is_file()}
    if set(files) != set(D0_ARTIFACT_SHA256):
        raise H0HoldoutError("D0 artifact file closure mismatch")
    payloads: dict[str, bytes] = {}
    identities: dict[str, Any] = {}
    for name, expected in sorted(D0_ARTIFACT_SHA256.items()):
        file = files[name]
        if file.is_symlink():
            raise H0HoldoutError(f"D0 artifact must not be a symlink: {name}")
        data = file.read_bytes()
        if sha256_bytes(data) != expected:
            raise H0HoldoutError(f"D0 artifact hash mismatch: {name}")
        payloads[name] = data
        identities[name] = {"bytes": len(data), "sha256": expected}
    report = json.loads(payloads["report.json"])
    decision = json.loads(payloads["d0-decision.json"])
    freeze = json.loads(payloads["candidate-freeze.json"])
    if (
        report.get("decision") != "Pass"
        or report.get("candidate_frozen") is not True
        or decision.get("next_authorized_stage") != "V34-H0-one-shot-method-holdout"
        or decision.get("method_holdout_opened") is not False
        or freeze.get("candidate_weights_sha256")
        != D0_ARTIFACT_SHA256["candidate-weights.bin"]
        or freeze.get("owner_sha256") != D0_OWNER_SHA256
        or freeze.get("profile_sha256") != d0.f0.PROFILE_SHA256
        or freeze.get("terminal_owner_sha256") != d0.TERMINAL_OWNER_SHA256
    ):
        raise H0HoldoutError("D0 candidate freeze closure mismatch")
    return payloads, identities


def decode_weights(
    data: bytes, effective: dict[str, Any], contact_input_count: int, seed: int
) -> Any:
    magic = b"NEXTV34W\0"
    if not data.startswith(magic):
        raise H0HoldoutError("V34 weight magic mismatch")
    offset = len(magic)

    def take(format_text: str) -> tuple[Any, ...]:
        nonlocal offset
        size = struct.calcsize(format_text)
        if offset + size > len(data):
            raise H0HoldoutError("truncated V34 weight payload")
        values = struct.unpack_from(format_text, data, offset)
        offset += size
        return values

    version, state_count = take("<II")
    if version != 1:
        raise H0HoldoutError("V34 weight version mismatch")
    state: dict[str, Any] = {}
    for _ in range(state_count):
        (name_size,) = take("<I")
        if offset + name_size > len(data):
            raise H0HoldoutError("truncated V34 weight name")
        name = data[offset : offset + name_size].decode()
        offset += name_size
        (dimensions,) = take("<I")
        shape = take(f"<{dimensions}I")
        value_count = math.prod(shape)
        value_bytes = value_count * 8
        if offset + value_bytes > len(data) or name in state:
            raise H0HoldoutError("invalid V34 weight tensor closure")
        values = np.frombuffer(data, dtype="<f8", count=value_count, offset=offset)
        offset += value_bytes
        state[name] = d0.v33.torch.from_numpy(values.copy().reshape(shape))
    if offset != len(data):
        raise H0HoldoutError("V34 weight payload has trailing bytes")
    model = d0.v33.i0.ResidualModel(effective, contact_input_count, seed)
    try:
        model.load_state_dict(state, strict=True)
    except RuntimeError as error:
        raise H0HoldoutError(f"V34 weight state mismatch: {error}") from error
    return model


def build_role(
    role: str,
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
    ledger: AccessLedger,
) -> d0.v33.RoleData:
    if role not in ALLOWED_ROLES:
        raise H0HoldoutError(f"H0 role access forbidden: {role}")
    corpus = effective["corpus"]
    h = float(effective["features"]["lift"]["local_stencil_offset"])
    row_ids: list[str] = []
    feature_rows: dict[str, list[np.ndarray]] = {branch: [] for branch in d0.BRANCHES}
    target_rows: list[np.ndarray] = []
    cases: list[dict[str, Any]] = []
    stratum_rows: dict[str, list[int]] = {}
    geometry_groups: set[tuple[int, int, int]] = set()
    for role_entry in corpus["role_plan"][role]:
        stratum = role_entry["stratum"]
        if stratum in stratum_rows:
            raise H0HoldoutError(f"duplicate role stratum: {role}/{stratum}")
        stratum_rows[stratum] = []
        contact_set = role_entry["contact_set"]
        for family_index, family in enumerate(corpus["families"]):
            for material_index in range(len(corpus["materials"])):
                for geometry_cell_index in role_entry["geometry_cells"]:
                    geometry_groups.add(
                        (family_index, material_index, geometry_cell_index)
                    )
                    for contact_index in range(len(corpus["contacts"][contact_set])):
                        case_id, fixture = d0.c0.build_fixture(
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
                        solution = d0.c0.p1.solve_case(case_id, fixture, p0_profile)
                        if (
                            solution["metrics"]["remesh_common_vertices_exact"]
                            is not True
                        ):
                            raise H0HoldoutError(f"P1 remesh gate failed: {case_id}")
                        _, stencil = d0.v33.i0.stencil_values(solution, p0_profile, h)
                        start = len(row_ids)
                        modes = solution["modal_document"]["modes"]
                        multipliers = tuple(
                            float(value)
                            for value in corpus["geometry_multiplier_cells"][
                                geometry_cell_index
                            ]
                        )
                        for ordinal_index, mode in enumerate(modes):
                            row_id = f"{case_id}/o{ordinal_index:02d}"
                            row = d0.v33.i0.normalized_row(
                                effective,
                                fixture,
                                family,
                                mode,
                                multipliers,
                                stencil,
                                ordinal_index,
                            )
                            target = d0.oracle_targets(row)
                            if any(
                                np.any(~np.isfinite(row[branch]))
                                for branch in d0.BRANCHES
                            ):
                                raise H0HoldoutError(f"non-finite feature: {row_id}")
                            ledger.record_row(role)
                            row_ids.append(row_id)
                            stratum_rows[stratum].append(len(row_ids) - 1)
                            for branch in d0.BRANCHES:
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
    features = {
        branch: np.stack(feature_rows[branch]).astype(np.float64, copy=False)
        for branch in d0.BRANCHES
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
        len(cases) != expected["cases"]
        or len(row_ids) != expected["modal_rows"]
        or len(geometry_groups) != expected["role_local_geometry_groups"]
        or (
            role == "method_holdout"
            and any(
                len(strata[name]) != expected["strata"][name]["modal_rows"]
                for name in strata
            )
        )
    ):
        raise H0HoldoutError(f"role count closure mismatch: {role}")
    return d0.v33.RoleData(
        role=role,
        row_ids=row_ids,
        features=features,
        raw_features=raw_features,
        targets=targets,
        cases=cases,
        strata=strata,
        case_count=len(cases),
        role_local_geometry_groups=len(geometry_groups),
    )


def ratio(numerator: float, denominator: float) -> tuple[float, bool]:
    if denominator == 0.0:
        return (0.0, numerator == 0.0)
    value = numerator / denominator
    return (value, math.isfinite(value))


def holdout_evaluation(
    role: d0.v33.RoleData,
    candidate: np.ndarray,
    controls: dict[str, np.ndarray],
    effective: dict[str, Any],
) -> dict[str, Any]:
    candidate_metrics = d0.v33.metric_set(candidate, role.targets, role.strata)
    control_metrics = {
        name: d0.v33.metric_set(values, role.targets, role.strata)
        for name, values in sorted(controls.items())
    }
    frozen = effective["gates"]["method_holdout"]
    non_neural = ("identity", "nearest", "raw_ridge", "spectral_ridge")
    gates: dict[str, bool] = {}
    ratios: dict[str, float] = {}
    best_aggregate = min(
        control_metrics[name]["aggregate_normalized_rmse"] for name in non_neural
    )
    value, valid = ratio(candidate_metrics["aggregate_normalized_rmse"], best_aggregate)
    ratios["aggregate_vs_best_non_neural"] = value
    gates["aggregate_vs_best_non_neural"] = valid and value <= float(
        frozen["aggregate_ratio_max_best_non_neural_control"]
    )
    for branch in d0.BRANCHES:
        candidate_value = candidate_metrics["branch_rmse"][branch]
        best = min(control_metrics[name]["branch_rmse"][branch] for name in non_neural)
        value, valid = ratio(candidate_value, best)
        key = f"{branch}_vs_best_non_neural"
        ratios[key] = value
        gates[key] = valid and value <= float(
            frozen["branch_ratio_max_best_non_neural_control"]
        )
    contact = candidate_metrics["branch_rmse"]["contact"]
    value, valid = ratio(contact, control_metrics["raw_mlp"]["branch_rmse"]["contact"])
    ratios["contact_vs_raw_mlp"] = value
    gates["contact_vs_raw_mlp"] = valid and value <= float(
        frozen["contact_ratio_max_raw_mlp"]
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
        value, valid = ratio(candidate_value, best)
        key = f"contact_{stratum}_vs_best_control"
        ratios[key] = value
        gates[key] = valid and value <= float(
            frozen["contact_each_stratum_ratio_max_best_control"]
        )
        gates[f"absolute_contact_{stratum}"] = candidate_value <= float(
            frozen["absolute_rmse_max"]["contact_each_stratum"]
        )
    return {
        "candidate": candidate_metrics,
        "controls": control_metrics,
        "gates": gates,
        "pass": all(gates.values()),
        "ratios": ratios,
    }


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def post_access_fault(
    output: Path,
    ledger: AccessLedger,
    error: BaseException,
    stage: str,
    maximum_output_bytes: int,
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
        maximum_output_bytes,
    )


def run(profile_path: Path, d0_artifacts: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    ledger = AccessLedger()
    stage = "pre-access-identity"
    output = (
        output_path if output_path.is_absolute() else repository_root() / output_path
    )
    try:
        payloads, artifact_identities = validate_d0_artifacts(d0_artifacts)
        _, effective, p0_profile, profile_data = d0.load_context(profile_path)
        d0.validate_bound_file(D0_OWNER_PATH, D0_OWNER_SHA256)
        d0.validate_bound_file(D0_RESULT_PATH, D0_RESULT_SHA256)
        resources = effective["resources"]
        d0.v33.i0.configure_torch()
        stage = "disclosed-train-rebuild"
        train = build_role("train", effective, p0_profile, ledger)
        d0_manifest = json.loads(payloads["corpus-manifest.json"])["roles"]["train"]
        if d0.v33.role_manifest(train) != d0_manifest:
            raise H0HoldoutError("rebuilt train role differs from frozen D0 manifest")
        fitted_controls = d0.v33.fit_controls(train)
        stage = "frozen-model-load"
        candidate = decode_weights(
            payloads["candidate-weights.bin"],
            effective,
            effective["model"]["heads"]["contact"]["input_count"],
            effective["training"]["candidate_seed"],
        )
        raw_mlp = decode_weights(
            payloads["raw-mlp-control-weights.bin"],
            effective,
            effective["controls"]["raw_mlp"]["contact_input_count"],
            effective["training"]["raw_mlp_control_seed"],
        )
        ledger.model_parameters_loaded = (
            effective["model"]["parameter_count"]
            + effective["controls"]["raw_mlp"]["parameter_count"]
        )
        stage = "method-holdout-role"
        holdout = build_role("method_holdout", effective, p0_profile, ledger)
        stage = "method-holdout-evaluation"
        candidate_predictions = d0.v33.model_predictions(candidate, holdout, raw=False)
        raw_predictions = d0.v33.model_predictions(raw_mlp, holdout, raw=True)
        controls = d0.v33.control_predictions(
            train, holdout, fitted_controls, raw_predictions
        )
        evaluation = holdout_evaluation(
            holdout, candidate_predictions, controls, effective
        )
        hard = d0.safe_hard_result(
            holdout, candidate_predictions, p0_profile, effective
        )
        isolation = d0.safe_branch_isolation(candidate, holdout)
        elapsed = time.monotonic() - started
        rss = peak_rss_bytes()
        resource_gates = {
            "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
            "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
        }
        if not hard.get("pass", False) or not isolation.get("pass", False):
            outcome = "HardGateReject"
        elif not evaluation["pass"]:
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
            "d0_artifacts": artifact_identities,
            "hard": hard,
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "profile_identity": {
                "bytes": len(profile_data),
                "path": d0.f0.PROFILE_PATH,
                "sha256": d0.f0.PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v34-h0-owner-evidence.v1",
            "training_steps": 0,
        }
        prediction_data = d0.encode_predictions(holdout.row_ids, candidate_predictions)
        terminal_payloads = {
            "control-report.json": canonical_json(
                {
                    "claim": CLAIM,
                    "method_holdout": evaluation,
                    "schema": "nextengine.experimental-physical-sound-v34-h0-control-report.v1",
                }
            ),
            "h0-decision.json": canonical_json(
                {
                    "decision": outcome,
                    "method_holdout_opened": True,
                    "next_authorized_stage": (
                        "V34-SYNTHETIC-CANDIDATE-ADMITTED"
                        if outcome == "Pass"
                        else "V34-family-closed"
                    ),
                }
            ),
            "method-holdout-manifest.json": canonical_json(
                d0.v33.role_manifest(holdout)
            ),
            "method-holdout-predictions.bin": prediction_data,
            "owner-evidence.json": canonical_json(owner_evidence),
        }
        if outcome == "Pass":
            terminal_payloads[terminal.FREEZE_NAME] = payloads["candidate-freeze.json"]
        report = terminal.publish_scientific_terminal(
            output,
            repository_root(),
            outcome,
            ledger.as_dict(),
            terminal_payloads,
            CLAIM,
            resources["max_output_bytes"],
        )
        print(
            f"h0-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
            f"outcome={outcome}",
            file=sys.stderr,
        )
        return report
    except BaseException as error:
        if isinstance(error, (KeyboardInterrupt, SystemExit)):
            raise
        if ledger.target_rows_accessed > 0:
            maximum = locals().get("resources", {}).get("max_output_bytes", 67_108_864)
            return post_access_fault(output, ledger, error, stage, maximum)
        return terminal.pre_access_contract_reject(
            output,
            repository_root(),
            ledger.as_dict(),
            f"{stage}:{type(error).__name__}:{error}",
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--d0-artifacts", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    report = run(arguments.profile, arguments.d0_artifacts, arguments.output)
    print(canonical_json(report).decode(), end="")
    if report["decision"] == "Pass":
        return 0
    if report["decision"] in terminal.SCIENTIFIC_OUTCOMES:
        return 1
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
