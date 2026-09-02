#!/usr/bin/env python3
"""Run the one-shot V34 train/development tournament without holdout access."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import resource
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v33_d0_development_tournament_v1 as v33
import physical_sound_v34_c0_structural_witness_census_v1 as c0
import physical_sound_v34_f0_target_safe_profile_freeze_v1 as f0
import physical_sound_v34_terminal_publication_v1 as terminal

OWNER_PATH = "lab/scripts/physical_sound_v34_d0_development_tournament_v1.py"
V33_PRIMITIVES_PATH = "lab/scripts/physical_sound_v33_d0_development_tournament_v1.py"
V33_PRIMITIVES_SHA256 = (
    "b2c00accf27f4782954e1eefdc363c648d0e3e5b659ce2b1cd0cad55941e866a"
)
C0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v34-c0-structural-witness-census-result-2026-09-02.md"
)
C0_RESULT_SHA256 = "dbfcf031c4ab59a0c7eb3ca6b1b2bb8be3fef4f479d7a7b36eb36b08a12b964e"
T0_RESULT_PATH = (
    "docs/development/physical-sound-v34-t0-terminal-path-proof-result-2026-09-02.md"
)
T0_RESULT_SHA256 = "00d1ff517b1c08d59a0abbbf08bccb0fb59b17a54e308c0f16de5a54294d81c0"
TERMINAL_OWNER_PATH = "lab/scripts/physical_sound_v34_terminal_publication_v1.py"
TERMINAL_OWNER_SHA256 = (
    "60c54b392e20f1024a1cfbb3c2f69d3aed925e341495219787b6379fe1268f8d"
)
CLAIM = (
    "SYNTHETIC_FRESH_V34_TRAIN_DEVELOPMENT_TOURNAMENT_ONLY / "
    "NO_METHOD_HOLDOUT_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
ALLOWED_ROLES = ("train", "development")
BRANCHES = v33.BRANCHES
TARGET_INDEX = v33.TARGET_INDEX
ORACLE_SCALES = v33.ORACLE_SCALES


class D0TournamentError(RuntimeError):
    """The V34 development owner cannot return a valid terminal decision."""


@dataclass
class AccessLedger:
    access_phase: str = "pre-access"
    development_target_rows: int = 0
    feature_rows_materialized: int = 0
    method_holdout_target_rows: int = 0
    network_requests: int = 0
    official_model_parameters_initialized: int = 0
    oracle_values_evaluated: int = 0
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

    def record_model(self, parameters: int) -> None:
        if parameters <= 0:
            raise D0TournamentError("model parameter receipt is invalid")
        self.official_model_parameters_initialized += parameters

    def as_dict(self) -> dict[str, Any]:
        return {
            "access_phase": self.access_phase,
            "development_target_rows": self.development_target_rows,
            "feature_rows_materialized": self.feature_rows_materialized,
            "method_holdout_target_rows": self.method_holdout_target_rows,
            "network_requests": self.network_requests,
            "official_model_parameters_initialized": self.official_model_parameters_initialized,
            "oracle_values_evaluated": self.oracle_values_evaluated,
            "protected_signal_values_decoded": self.protected_signal_values_decoded,
            "real_signal_values_decoded": self.real_signal_values_decoded,
            "target_rows_accessed": self.target_rows_accessed,
            "train_target_rows": self.train_target_rows,
        }


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
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], bytes]:
    overlay, profile_data = f0.load_profile(profile_path)
    f0.validate_dependencies(overlay)
    v33_profile = f0.load_declared_json(overlay, "v33_profile")
    effective = f0.build_effective_profile(overlay, v33_profile)
    p0_declaration = v33_profile["parent"]["p0_profile"]
    p0_data = (repository_root() / p0_declaration["path"]).read_bytes()
    if sha256_bytes(p0_data) != p0_declaration["sha256"]:
        raise D0TournamentError("P0 profile dependency drift")
    validate_bound_file(V33_PRIMITIVES_PATH, V33_PRIMITIVES_SHA256)
    validate_bound_file(C0_RESULT_PATH, C0_RESULT_SHA256)
    validate_bound_file(T0_RESULT_PATH, T0_RESULT_SHA256)
    validate_bound_file(TERMINAL_OWNER_PATH, TERMINAL_OWNER_SHA256)
    return overlay, effective, json.loads(p0_data), profile_data


def validate_role_metadata(effective: dict[str, Any]) -> dict[str, Any]:
    corpus = effective["corpus"]
    cases_by_role: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    result: dict[str, Any] = {}
    for role in ALLOWED_ROLES:
        cases, strata = f0.enumerate_role_cases(corpus, role)
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
    decay = 0.20 * math.tanh(
        0.40 * context["loss"]
        + 0.30 * context["log_frequency"]
        + 0.17 * context["support"] * context["ordinal"]
        + 0.14 * context["youngs"] * context["density"]
        + 0.10 * context["poisson"] * context["index_a"]
    )
    gain = 0.16 * math.tanh(
        0.32 * context["ordinal"] * context["geometry"][0]
        + 0.27 * context["geometry"][2]
        + 0.24 * context["youngs"] * context["ordinal"]
        - 0.16 * context["density"]
        + 0.18 * context["family"] * context["geometry"][1]
        + 0.13 * context["geometry"][0] * context["geometry"][2]
    )
    contact = 0.22 * math.tanh(
        0.24 * context["p1_contact"]
        + 0.16 * context["difference_u"]
        - 0.15 * context["difference_v"]
        + 0.19 * context["surface_a"]
        + 0.18 * context["surface_b"]
        + 0.11 * context["ordinal"] * context["support"]
        + 0.09 * context["family"] * (2.0 * context["u"] - 1.0)
        + 0.07 * context["log_frequency"] * context["curvature_u"]
    )
    result = np.asarray([decay, gain, contact], dtype=np.float64)
    if np.any(~np.isfinite(result)):
        raise D0TournamentError("V34 oracle target is non-finite")
    return result


def build_role(
    role: str,
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
    ledger: AccessLedger,
) -> v33.RoleData:
    if role not in ALLOWED_ROLES:
        raise D0TournamentError(f"D0 role access forbidden: {role}")
    corpus = effective["corpus"]
    h = float(effective["features"]["lift"]["local_stencil_offset"])
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
                        case_id, fixture = c0.build_fixture(
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
                        _, stencil = v33.i0.stencil_values(solution, p0_profile, h)
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
                            row = v33.i0.normalized_row(
                                effective,
                                fixture,
                                family,
                                mode,
                                multipliers,
                                stencil,
                                ordinal_index,
                            )
                            target = oracle_targets(row)
                            if any(
                                np.any(~np.isfinite(row[branch])) for branch in BRANCHES
                            ):
                                raise D0TournamentError(f"non-finite feature: {row_id}")
                            ledger.record_row(role)
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
    return v33.RoleData(
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


def encode_weights(model: Any) -> bytes:
    data = v33.encode_weights(model)
    if not data.startswith(v33.WEIGHTS_MAGIC):
        raise D0TournamentError("V33 weight primitive encoding drift")
    return b"NEXTV34W\0" + data[len(v33.WEIGHTS_MAGIC) :]


def encode_predictions(row_ids: list[str], predictions: np.ndarray) -> bytes:
    data = v33.encode_predictions(row_ids, predictions)
    if not data.startswith(v33.PREDICTIONS_MAGIC):
        raise D0TournamentError("V33 prediction primitive encoding drift")
    return b"NEXTV34P\0" + data[len(v33.PREDICTIONS_MAGIC) :]


def safe_hard_result(
    development: v33.RoleData,
    predictions: np.ndarray,
    p0_profile: dict[str, Any],
    effective: dict[str, Any],
) -> dict[str, Any]:
    try:
        return v33.i0.hard_gate_conformance(
            development, predictions, p0_profile, effective
        )
    except v33.i0.I0ConformanceError as error:
        return {"pass": False, "reason": str(error)}


def safe_branch_isolation(model: Any, development: v33.RoleData) -> dict[str, Any]:
    try:
        checks = v33.i0.branch_isolation_conformance(model, development)
        return {"checks": checks, "pass": all(checks.values())}
    except v33.i0.I0ConformanceError as error:
        return {"pass": False, "reason": str(error)}


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def standard_payloads(
    candidate: Any,
    raw_mlp: Any,
    train: v33.RoleData,
    development: v33.RoleData,
    candidate_predictions: np.ndarray,
    development_result: dict[str, Any],
    owner_evidence: dict[str, Any],
    outcome: str,
) -> dict[str, bytes]:
    candidate_weights = encode_weights(candidate)
    prediction_data = encode_predictions(development.row_ids, candidate_predictions)
    payloads = {
        "candidate-weights.bin": candidate_weights,
        "control-report.json": canonical_json(
            {
                "claim": CLAIM,
                "development": development_result,
                "method_holdout": None,
                "schema": "nextengine.experimental-physical-sound-v34-d0-control-report.v1",
            }
        ),
        "corpus-manifest.json": canonical_json(
            {
                "access": owner_evidence["access"],
                "claim": CLAIM,
                "roles": {
                    "development": v33.role_manifest(development),
                    "train": v33.role_manifest(train),
                },
                "schema": "nextengine.experimental-physical-sound-v34-d0-corpus-manifest.v1",
            }
        ),
        "d0-decision.json": canonical_json(
            {
                "decision": outcome,
                "method_holdout_opened": False,
                "next_authorized_stage": (
                    "V34-H0-one-shot-method-holdout"
                    if outcome == "Pass"
                    else "V34-family-closed-before-holdout"
                ),
            }
        ),
        "development-predictions.bin": prediction_data,
        "owner-evidence.json": canonical_json(owner_evidence),
        "raw-mlp-control-weights.bin": encode_weights(raw_mlp),
    }
    if outcome == "Pass":
        payloads[terminal.FREEZE_NAME] = canonical_json(
            {
                "candidate_weights_sha256": sha256_bytes(candidate_weights),
                "development_predictions_sha256": sha256_bytes(prediction_data),
                "owner_sha256": owner_evidence["owner_identity"]["sha256"],
                "profile_sha256": f0.PROFILE_SHA256,
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
    payload = canonical_json(
        {
            "error_type": type(error).__name__,
            "reason": str(error),
            "stage": stage,
            "status": "PostAccessOwnerFault",
        }
    )
    return terminal.publish_scientific_terminal(
        output,
        repository_root(),
        "HardGateReject",
        ledger.as_dict(),
        {"post-access-owner-fault.json": payload},
        CLAIM,
        resources["max_output_bytes"],
    )


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    ledger = AccessLedger()
    stage = "pre-access-context"
    output = (
        output_path if output_path.is_absolute() else repository_root() / output_path
    )
    try:
        overlay, effective, p0_profile, profile_data = load_context(profile_path)
        del overlay
        role_metadata = validate_role_metadata(effective)
        v33.i0.configure_torch()
        resources = effective["resources"]
        stage = "train-role"
        train = build_role("train", effective, p0_profile, ledger)
        stage = "candidate-training"
        ledger.record_model(effective["model"]["parameter_count"])
        candidate, candidate_training = v33.train_model(
            train,
            effective,
            effective["model"]["heads"]["contact"]["input_count"],
            effective["training"]["candidate_seed"],
            raw=False,
        )
        stage = "raw-control-training"
        ledger.record_model(effective["controls"]["raw_mlp"]["parameter_count"])
        raw_mlp, raw_training = v33.train_model(
            train,
            effective,
            effective["controls"]["raw_mlp"]["contact_input_count"],
            effective["training"]["raw_mlp_control_seed"],
            raw=True,
        )
        fitted_controls = v33.fit_controls(train)
        stage = "development-role"
        development = build_role("development", effective, p0_profile, ledger)
        stage = "development-evaluation"
        candidate_predictions = v33.model_predictions(candidate, development, raw=False)
        raw_predictions = v33.model_predictions(raw_mlp, development, raw=True)
        controls = v33.control_predictions(
            train, development, fitted_controls, raw_predictions
        )
        development_result = v33.development_evaluation(
            candidate, development, candidate_predictions, controls, effective
        )
        hard = safe_hard_result(
            development, candidate_predictions, p0_profile, effective
        )
        isolation = safe_branch_isolation(candidate, development)
        try:
            forbidden = v33.i0.forbidden_field_conformance(effective)
            forbidden_pass = True
        except v33.i0.I0ConformanceError as error:
            forbidden = {"reason": str(error), "status": "Reject"}
            forbidden_pass = False
        elapsed = time.monotonic() - started
        rss = peak_rss_bytes()
        resource_gates = {
            "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
            "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
        }
        if (
            not hard.get("pass", False)
            or not isolation.get("pass", False)
            or not forbidden_pass
        ):
            outcome = "HardGateReject"
        elif not development_result["pass"]:
            outcome = "MetricReject"
        elif not all(resource_gates.values()):
            outcome = "ResourceReject"
        else:
            outcome = "Pass"
        owner_data = (repository_root() / OWNER_PATH).read_bytes()
        dependencies = {
            "c0_result": validate_bound_file(C0_RESULT_PATH, C0_RESULT_SHA256),
            "t0_result": validate_bound_file(T0_RESULT_PATH, T0_RESULT_SHA256),
            "terminal_owner": validate_bound_file(
                TERMINAL_OWNER_PATH, TERMINAL_OWNER_SHA256
            ),
            "v33_primitives": validate_bound_file(
                V33_PRIMITIVES_PATH, V33_PRIMITIVES_SHA256
            ),
        }
        owner_evidence = {
            "access": ledger.as_dict(),
            "branch_isolation": isolation,
            "claim": CLAIM,
            "dependencies": dependencies,
            "forbidden_fields": forbidden,
            "hard": hard,
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "profile_identity": {
                "bytes": len(profile_data),
                "path": f0.PROFILE_PATH,
                "sha256": f0.PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "role_metadata": role_metadata,
            "schema": "nextengine.experimental-physical-sound-v34-d0-owner-evidence.v1",
            "training": {
                "candidate": candidate_training,
                "raw_mlp": raw_training,
            },
        }
        payloads = standard_payloads(
            candidate,
            raw_mlp,
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
            resources = locals().get(
                "resources",
                {"max_output_bytes": 67_108_864},
            )
            return post_access_fault(output, ledger, error, stage, resources)
        access = ledger.as_dict()
        return terminal.pre_access_contract_reject(
            output,
            repository_root(),
            access,
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
    decision = report["decision"]
    if decision == "Pass":
        return 0
    if decision in terminal.SCIENTIFIC_OUTCOMES:
        return 1
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
