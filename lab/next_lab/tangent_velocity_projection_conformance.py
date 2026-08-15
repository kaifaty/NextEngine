from __future__ import annotations

import hashlib
import json
import resource
import time
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_state_consistency_formulation import canonical_json, sha256
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _contact_points,
    load_r120_cache,
)
from next_lab.fixed_pd_inverse_dynamics_execution import collocation_state
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    build_spatial_model,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    propagate_motion,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

CONFORMANCE_ID = "nextengine.humanoid-tangent-velocity-projection-conformance.v1"
CHECK_ID = "TRAIN-4-TANGENT-VELOCITY-PROJECTION-CONFORMANCE"
GENERALIZED_WIDTH = 29
SUBSTEPS_PER_INTERVAL = 4
BOUNDED_ACCEPTANCE = {
    "r130_projected_fixed_pd_execution": "AUTHORIZED_ONE_EXECUTION_ONLY_ON_R129_PASS",
    "additional_projection_conformance": "NOT_AUTHORIZED",
    "r127_retry": "NOT_AUTHORIZED",
    "additional_inverse_dynamics_execution": "NOT_AUTHORIZED",
    "kinodynamic_solve": "NOT_AUTHORIZED",
    "candidate_artifact": "NOT_AUTHORIZED",
    "physx": "NOT_AUTHORIZED",
    "all_17": "NOT_AUTHORIZED",
    "full_v19": "NOT_AUTHORIZED",
    "training": "NOT_AUTHORIZED",
}


@dataclass(frozen=True)
class ProjectionAnalysis:
    status: str
    invalid_reason: str | None
    projected_velocity: NDArray[np.float64] | None
    delta_velocity: NDArray[np.float64] | None
    multiplier: NDArray[np.float64] | None
    rank: int
    nullity: int
    smallest_retained_relative_singular_value: float | None
    largest_null_relative_singular_value: float | None
    maximum_active_velocity_absolute: float | None
    scaled_kkt_residual: float | None
    closed_form_to_kkt_maximum_absolute: float | None
    projection_idempotence_error: float | None
    mass_orthogonality_relative_error: float | None
    kinetic_energy_before_joules: float | None
    kinetic_energy_after_joules: float | None
    kinetic_energy_increase_joules: float | None
    gauge_velocity_effect_maximum_absolute: float | None


def build_tangent_velocity_projection_conformance(
    *,
    profile_path: Path,
    r128_report_path: Path,
    r128_profile_path: Path,
    r128_module_path: Path,
    r128_tool_path: Path,
    r122_report_path: Path,
    r122_profile_path: Path,
    dynamics_kernel_path: Path,
    dynamics_conformance_module_path: Path,
    r122_tool_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    collocation_lift_module_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> dict[str, Any]:
    """Conform the R128 projection on seven anchors without inverse dynamics."""

    started = time.monotonic()
    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r128_report_path,
            r128_profile_path,
            r128_module_path,
            r128_tool_path,
            r122_report_path,
            r122_profile_path,
            dynamics_kernel_path,
            dynamics_conformance_module_path,
            r122_tool_path,
            r113_report_path,
            r113_profile_path,
            r120_report_path,
            r120_profile_path,
            r120_cache_path,
            v9_complete_clip_path,
            collocation_lift_module_path,
            tool_path,
        )
    )
    (
        profile_path,
        r128_report_path,
        r128_profile_path,
        r128_module_path,
        r128_tool_path,
        r122_report_path,
        r122_profile_path,
        dynamics_kernel_path,
        dynamics_conformance_module_path,
        r122_tool_path,
        r113_report_path,
        r113_profile_path,
        r120_report_path,
        r120_profile_path,
        r120_cache_path,
        v9_complete_clip_path,
        collocation_lift_module_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R129 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    r128 = _load_bound_report(r128_report_path, profile["source"]["r128"], "R128")
    r122 = _load_bound_report(r122_report_path, profile["source"]["r122"], "R122")
    r113 = _load_bound_report(r113_report_path, profile["source"]["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, profile["source"]["r120"], "R120")
    _validate_source_files(
        profile=profile,
        r128_profile_path=r128_profile_path,
        r128_module_path=r128_module_path,
        r128_tool_path=r128_tool_path,
        r122_profile_path=r122_profile_path,
        dynamics_kernel_path=dynamics_kernel_path,
        dynamics_conformance_module_path=dynamics_conformance_module_path,
        r122_tool_path=r122_tool_path,
        r113_profile_path=r113_profile_path,
        r120_profile_path=r120_profile_path,
        r120_cache_path=r120_cache_path,
        v9_complete_clip_path=v9_complete_clip_path,
        collocation_lift_module_path=collocation_lift_module_path,
    )
    _validate_source_contracts(
        profile=profile, r128=r128, r122=r122, r113=r113, r120=r120
    )
    source = profile["source"]
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != source["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve()) != source["conformance_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R129 current implementation identity differs")

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (801, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R129 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=r113)
    anchor_rows = audit_frozen_anchors(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
    )
    synthetic = audit_synthetic_cases(profile)
    projection_passed = bool(
        all(row["status"] == "PASS" for row in anchor_rows)
        and synthetic["status"] == "PASS"
    )
    elapsed = time.monotonic() - started
    maximum_threads = _linux_thread_count()
    maximum_rss = _resident_memory_bytes()
    budget = profile["resource_budget"]
    resource_passed = not (
        elapsed > float(budget["maximum_wall_clock_seconds"])
        or maximum_threads > int(budget["thread_count"])
        or maximum_rss > int(budget["maximum_resident_memory_bytes"])
    )
    passed = projection_passed and resource_passed

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": "PASS" if passed else "FAIL",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["pass" if passed else "fail"],
        "scope": profile["scope"],
        "source_gates": {
            "r128_status": r128["status"],
            "r128_report_sha256": r128["report_sha256"],
            "r122_status": r122["status"],
            "r122_report_sha256": r122["report_sha256"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "numeric_contract": profile["numeric_contract"],
        "frozen_anchor_audits": anchor_rows,
        "synthetic_projection_audit": synthetic,
        "validation_results": validations,
        "execution_environment": dict(execution_environment),
        "resource_usage": {
            "status": "PASS" if resource_passed else "FAIL",
            "wall_clock_seconds": elapsed,
            "maximum_resident_memory_bytes": maximum_rss,
            "maximum_observed_os_thread_count": maximum_threads,
            "process_count": 1,
            "child_processes_spawned_during_conformance": 0,
            "randomized_restart_count": 0,
        },
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r128_report_file_sha256": sha256(r128_report_path),
            "r128_profile_sha256": sha256(r128_profile_path),
            "r128_module_sha256": sha256(r128_module_path),
            "r128_tool_sha256": sha256(r128_tool_path),
            "r122_report_file_sha256": sha256(r122_report_path),
            "r122_profile_sha256": sha256(r122_profile_path),
            "dynamics_kernel_sha256": sha256(dynamics_kernel_path),
            "dynamics_conformance_module_sha256": sha256(
                dynamics_conformance_module_path
            ),
            "r122_tool_sha256": sha256(r122_tool_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_profile_sha256": sha256(r120_profile_path),
            "r120_cache_sha256": sha256(r120_cache_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "collocation_lift_module_sha256": sha256(collocation_lift_module_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "real_anchor_audits": 7,
        "real_state_projections": 6,
        "real_mass_matrix_assemblies": 7,
        "real_contact_jacobian_assemblies": 6,
        "real_projection_factorizations": 6,
        "real_projection_solves": 6,
        "full_schedule_projections": 0,
        "inverse_dynamics_evaluations": 0,
        "inverse_dynamics_solves": 0,
        "force_gauge_interval_classifications": 0,
        "kinodynamic_solves": 0,
        "candidate_artifacts_built": 0,
        "solver_private_caches_built": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def project_tangent_velocity(
    *,
    mass: NDArray[np.float64],
    jacobian: NDArray[np.float64],
    velocity: NDArray[np.float64],
    expected_rank: int,
    numeric: Mapping[str, Any],
) -> ProjectionAnalysis:
    rows = jacobian.shape[0]
    if (
        mass.shape != (GENERALIZED_WIDTH, GENERALIZED_WIDTH)
        or jacobian.ndim != 2
        or jacobian.shape[1] != GENERALIZED_WIDTH
        or velocity.shape != (GENERALIZED_WIDTH,)
        or not 0 < rows <= GENERALIZED_WIDTH
        or not 0 <= expected_rank <= rows
        or any(not np.all(np.isfinite(value)) for value in (mass, jacobian, velocity))
    ):
        raise ValueError("R129 projection input differs")
    symmetry_error = float(
        np.linalg.norm(mass - mass.T, ord=2)
        / max(float(np.linalg.norm(mass, ord=2)), 1.0)
    )
    if symmetry_error > float(numeric["maximum_mass_matrix_relative_symmetry"]):
        return _invalid_projection("MASS_MATRIX_NOT_SYMMETRIC")
    symmetric = 0.5 * (mass + mass.T)
    try:
        lower = np.linalg.cholesky(symmetric)
    except np.linalg.LinAlgError:
        return _invalid_projection("MASS_MATRIX_NOT_POSITIVE_DEFINITE")
    whitened_jacobian = np.linalg.solve(lower, jacobian.T).T
    left, singular, right_t = np.linalg.svd(whitened_jacobian, full_matrices=True)
    largest = float(singular[0]) if singular.size else 0.0
    if not np.isfinite(largest) or largest <= 0.0:
        return _invalid_projection("ZERO_OR_NONFINITE_PROJECTION_SCALE")
    relative = singular / largest
    retained_minimum = float(numeric["rank_revealing_relative_retained_minimum"])
    null_maximum = float(numeric["rank_revealing_relative_null_maximum"])
    ambiguous = (relative > null_maximum) & (relative < retained_minimum)
    if np.any(ambiguous):
        return _invalid_projection("AMBIGUOUS_PROJECTION_RANK_GAP")
    rank = int(np.count_nonzero(relative >= retained_minimum))
    nullity = rows - rank
    smallest_retained = float(relative[rank - 1]) if rank else None
    largest_null = float(relative[rank]) if rank < singular.size else 0.0
    if rank != expected_rank:
        return _invalid_projection(
            "PROJECTION_RANK_DIFFERS", rank=rank, nullity=nullity
        )

    right = right_t.T
    row_basis = right[:, :rank]
    whitened_velocity = lower.T @ velocity
    delta_whitened = -row_basis @ (row_basis.T @ whitened_velocity)
    projected_direct = np.linalg.solve(lower.T, whitened_velocity + delta_whitened)
    rhs = jacobian @ velocity
    retained_left = left[:, :rank]
    multiplier = retained_left @ ((retained_left.T @ rhs) / np.square(singular[:rank]))
    delta_kkt = -np.linalg.solve(lower.T, whitened_jacobian.T @ multiplier)
    projected_kkt = velocity + delta_kkt
    delta = projected_direct - velocity
    active_residual = jacobian @ projected_direct
    stationarity = symmetric @ delta + jacobian.T @ multiplier
    scale = max(
        float(np.max(np.abs(symmetric @ delta))),
        float(np.max(np.abs(jacobian.T @ multiplier))),
        float(np.max(np.abs(rhs))),
        1.0,
    )
    kkt_residual = (
        max(
            float(np.max(np.abs(stationarity))),
            float(np.max(np.abs(active_residual))),
        )
        / scale
    )
    closed_to_kkt = float(np.max(np.abs(projected_direct - projected_kkt)))
    projected_whitened = lower.T @ projected_direct
    second_delta = -row_basis @ (row_basis.T @ projected_whitened)
    projected_twice = np.linalg.solve(lower.T, projected_whitened + second_delta)
    idempotence = float(np.max(np.abs(projected_twice - projected_direct)))
    whitened_projector = np.eye(GENERALIZED_WIDTH) - row_basis @ row_basis.T
    projector = np.linalg.solve(lower.T, whitened_projector @ lower.T)
    orthogonality = float(
        np.linalg.norm(projector.T @ symmetric - symmetric @ projector, ord=2)
        / max(float(np.linalg.norm(symmetric, ord=2)), 1.0)
    )
    energy_before = float(0.5 * velocity @ symmetric @ velocity)
    energy_after = float(0.5 * projected_direct @ symmetric @ projected_direct)
    gauge_effect = 0.0
    if nullity:
        gauge_basis = left[:, rank:]
        gauge_delta = np.linalg.solve(lower.T, whitened_jacobian.T @ gauge_basis)
        gauge_effect = float(np.max(np.abs(gauge_delta)))
    return ProjectionAnalysis(
        status="VALID",
        invalid_reason=None,
        projected_velocity=projected_direct,
        delta_velocity=delta,
        multiplier=multiplier,
        rank=rank,
        nullity=nullity,
        smallest_retained_relative_singular_value=smallest_retained,
        largest_null_relative_singular_value=largest_null,
        maximum_active_velocity_absolute=float(np.max(np.abs(active_residual))),
        scaled_kkt_residual=kkt_residual,
        closed_form_to_kkt_maximum_absolute=closed_to_kkt,
        projection_idempotence_error=idempotence,
        mass_orthogonality_relative_error=orthogonality,
        kinetic_energy_before_joules=energy_before,
        kinetic_energy_after_joules=energy_after,
        kinetic_energy_increase_joules=energy_after - energy_before,
        gauge_velocity_effect_maximum_absolute=gauge_effect,
    )


def audit_frozen_anchors(
    *,
    profile: Mapping[str, Any],
    model: Any,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
) -> list[dict[str, Any]]:
    numeric = profile["numeric_contract"]
    rows = []
    for frozen in profile["frozen_anchors"]:
        collocation = int(frozen["collocation"])
        interval, substep = divmod(collocation, SUBSTEPS_PER_INTERVAL)
        state = collocation_state(cache, interval, substep)
        matrix, kinematics = mass_matrix(model, state.configuration)
        modes = contact_modes[interval]
        active = tuple(
            ordinal for ordinal in range(len(points)) if _point_active(modes, ordinal)
        )
        expected_modes = tuple(int(value) for value in frozen["contact_modes"])
        expected_rank = int(frozen["expected_rank"])
        expected_nullity = int(frozen["expected_multiplier_nullity"])
        if not active:
            projected = np.array(state.velocity, copy=True)
            analysis = None
            metrics = {
                "rank": 0,
                "multiplier_nullity": 0,
                "smallest_retained_relative_singular_value": None,
                "largest_null_relative_singular_value": None,
                "maximum_active_velocity_before_metres_per_second": 0.0,
                "maximum_active_velocity_after_metres_per_second": 0.0,
                "maximum_generalized_velocity_correction": 0.0,
                "mass_metric_velocity_correction": 0.0,
                "scaled_kkt_residual": 0.0,
                "closed_form_to_kkt_maximum_absolute": 0.0,
                "projection_idempotence_error": 0.0,
                "mass_orthogonality_relative_error": 0.0,
                "kinetic_energy_before_joules": float(
                    0.5 * state.velocity @ matrix @ state.velocity
                ),
                "kinetic_energy_after_joules": float(
                    0.5 * state.velocity @ matrix @ state.velocity
                ),
                "kinetic_energy_increase_joules": 0.0,
                "gauge_velocity_effect_maximum_absolute": 0.0,
            }
            flat_audit = None
            valid = True
        else:
            jacobian = np.vstack(
                [
                    point_jacobian(
                        model,
                        kinematics,
                        int(points[ordinal]["body_slot"]),
                        np.asarray(
                            points[ordinal]["local_translation_metres"],
                            dtype=np.float64,
                        ),
                    )
                    for ordinal in active
                ]
            )
            analysis = project_tangent_velocity(
                mass=matrix,
                jacobian=jacobian,
                velocity=state.velocity,
                expected_rank=expected_rank,
                numeric=numeric,
            )
            if analysis.projected_velocity is None or analysis.delta_velocity is None:
                projected = np.full(GENERALIZED_WIDTH, np.nan, dtype=np.float64)
                metrics = _analysis_metrics(analysis, jacobian, state.velocity)
                flat_audit = None
                valid = False
            else:
                projected = analysis.projected_velocity
                metrics = _analysis_metrics(analysis, jacobian, state.velocity)
                flat_audit = (
                    _flat_line_audit(
                        model=model,
                        kinematics=kinematics,
                        original_velocity=state.velocity,
                        projected_velocity=projected,
                        points=points,
                        active=active,
                    )
                    if len(active) == 2
                    else None
                )
                valid = _projection_passes(
                    analysis=analysis, numeric=numeric, flat_audit=flat_audit
                )
        passed = bool(
            valid
            and tuple(int(value) for value in modes) == expected_modes
            and interval == int(frozen["interval"])
            and substep == int(frozen["substep"])
            and list(active) == frozen["active_point_ordinals"]
            and metrics["rank"] == expected_rank
            and metrics["multiplier_nullity"] == expected_nullity
        )
        rows.append(
            {
                "status": "PASS" if passed else "FAIL",
                "collocation": collocation,
                "interval": interval,
                "substep": substep,
                "contact_modes": modes.tolist(),
                "active_point_ordinals": list(active),
                "mass_matrix_minimum_eigenvalue": float(
                    np.linalg.eigvalsh(0.5 * (matrix + matrix.T))[0]
                ),
                "original_velocity_sha256": _array_sha256(state.velocity),
                "projected_velocity_sha256": _array_sha256(projected),
                "projection": metrics,
                "flat_rigid_line_audit": flat_audit,
                "invalid_reason": analysis.invalid_reason if analysis else None,
            }
        )
    return rows


def audit_synthetic_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    numeric = profile["numeric_contract"]
    mass = np.diag(np.linspace(0.5, 3.0, GENERALIZED_WIDTH)).astype(np.float64)
    velocity = np.linspace(-0.7, 0.9, GENERALIZED_WIDTH, dtype=np.float64)
    single_jacobian = np.zeros((3, GENERALIZED_WIDTH), dtype=np.float64)
    single_jacobian[:, :3] = np.eye(3)
    single = project_tangent_velocity(
        mass=mass,
        jacobian=single_jacobian,
        velocity=velocity,
        expected_rank=3,
        numeric=numeric,
    )
    single_pass = _projection_passes(analysis=single, numeric=numeric, flat_audit=None)

    separation = np.asarray((0.0, 0.0, 0.215), dtype=np.float64)
    flat_jacobian = np.zeros((6, GENERALIZED_WIDTH), dtype=np.float64)
    flat_jacobian[:3, :3] = np.eye(3)
    flat_jacobian[3:, :3] = np.eye(3)
    flat_jacobian[3:, 3:6] = -_skew(separation)
    flat = project_tangent_velocity(
        mass=mass,
        jacobian=flat_jacobian,
        velocity=velocity,
        expected_rank=5,
        numeric=numeric,
    )
    flat_pass = bool(
        _projection_passes(analysis=flat, numeric=numeric, flat_audit=None)
        and flat.nullity == 1
    )
    idempotence_pass = bool(
        flat.projection_idempotence_error is not None
        and flat.projection_idempotence_error
        <= float(numeric["maximum_projection_idempotence_error"])
    )

    gap_jacobian = np.zeros((3, GENERALIZED_WIDTH), dtype=np.float64)
    gap_jacobian[0, 0] = 1.0
    gap_jacobian[1, 1] = 1.0
    gap_jacobian[2, 2] = 5.0e-11
    gap = project_tangent_velocity(
        mass=np.eye(GENERALIZED_WIDTH, dtype=np.float64),
        jacobian=gap_jacobian,
        velocity=velocity,
        expected_rank=3,
        numeric=numeric,
    )
    try:
        project_tangent_velocity(
            mass=mass,
            jacobian=single_jacobian,
            velocity=np.full(GENERALIZED_WIDTH, np.nan, dtype=np.float64),
            expected_rank=3,
            numeric=numeric,
        )
        nonfinite_rejected = False
    except ValueError:
        nonfinite_rejected = True
    fail_closed_pass = bool(
        gap.status == "INVALID"
        and gap.invalid_reason == "AMBIGUOUS_PROJECTION_RANK_GAP"
        and nonfinite_rejected
    )
    cases = [
        {"case": "full_rank_single_point", "status": "PASS" if single_pass else "FAIL"},
        {"case": "same_body_flat_rank_five", "status": "PASS" if flat_pass else "FAIL"},
        {
            "case": "projection_idempotence",
            "status": "PASS" if idempotence_pass else "FAIL",
        },
        {
            "case": "rank_gap_and_nonfinite_fail_closed",
            "status": "PASS" if fail_closed_pass else "FAIL",
        },
    ]
    return {
        "status": "PASS" if all(row["status"] == "PASS" for row in cases) else "FAIL",
        "cases": cases,
        "synthetic_projection_factorizations": 3,
        "synthetic_projection_solves": 2,
    }


def _analysis_metrics(
    analysis: ProjectionAnalysis,
    jacobian: NDArray[np.float64],
    original_velocity: NDArray[np.float64],
) -> dict[str, Any]:
    delta = analysis.delta_velocity
    return {
        "rank": analysis.rank,
        "multiplier_nullity": analysis.nullity,
        "smallest_retained_relative_singular_value": analysis.smallest_retained_relative_singular_value,
        "largest_null_relative_singular_value": analysis.largest_null_relative_singular_value,
        "maximum_active_velocity_before_metres_per_second": float(
            np.max(np.abs(jacobian @ original_velocity))
        ),
        "maximum_active_velocity_after_metres_per_second": analysis.maximum_active_velocity_absolute,
        "maximum_generalized_velocity_correction": (
            float(np.max(np.abs(delta))) if delta is not None else None
        ),
        "mass_metric_velocity_correction": (
            float(
                np.sqrt(
                    max(
                        2.0
                        * (
                            float(analysis.kinetic_energy_before_joules)
                            - float(analysis.kinetic_energy_after_joules)
                        ),
                        0.0,
                    )
                )
            )
            if analysis.kinetic_energy_before_joules is not None
            and analysis.kinetic_energy_after_joules is not None
            else None
        ),
        "scaled_kkt_residual": analysis.scaled_kkt_residual,
        "closed_form_to_kkt_maximum_absolute": analysis.closed_form_to_kkt_maximum_absolute,
        "projection_idempotence_error": analysis.projection_idempotence_error,
        "mass_orthogonality_relative_error": analysis.mass_orthogonality_relative_error,
        "kinetic_energy_before_joules": analysis.kinetic_energy_before_joules,
        "kinetic_energy_after_joules": analysis.kinetic_energy_after_joules,
        "kinetic_energy_increase_joules": analysis.kinetic_energy_increase_joules,
        "gauge_velocity_effect_maximum_absolute": analysis.gauge_velocity_effect_maximum_absolute,
    }


def _projection_passes(
    *,
    analysis: ProjectionAnalysis,
    numeric: Mapping[str, Any],
    flat_audit: Mapping[str, Any] | None,
) -> bool:
    return bool(
        analysis.status == "VALID"
        and analysis.maximum_active_velocity_absolute is not None
        and analysis.maximum_active_velocity_absolute
        <= float(numeric["maximum_active_point_velocity_absolute_metres_per_second"])
        and analysis.scaled_kkt_residual is not None
        and analysis.scaled_kkt_residual
        <= float(numeric["maximum_scaled_kkt_residual"])
        and analysis.closed_form_to_kkt_maximum_absolute is not None
        and analysis.closed_form_to_kkt_maximum_absolute
        <= float(numeric["maximum_closed_form_to_kkt_absolute"])
        and analysis.projection_idempotence_error is not None
        and analysis.projection_idempotence_error
        <= float(numeric["maximum_projection_idempotence_error"])
        and analysis.mass_orthogonality_relative_error is not None
        and analysis.mass_orthogonality_relative_error
        <= float(numeric["maximum_mass_orthogonality_relative_error"])
        and analysis.kinetic_energy_increase_joules is not None
        and analysis.kinetic_energy_increase_joules
        <= float(numeric["maximum_kinetic_energy_increase_joules"])
        and analysis.gauge_velocity_effect_maximum_absolute is not None
        and analysis.gauge_velocity_effect_maximum_absolute
        <= float(numeric["maximum_gauge_velocity_effect_absolute"])
        and (
            flat_audit is None
            or abs(
                float(
                    flat_audit["projected_line_compatibility_metres_per_second_squared"]
                )
            )
            <= float(
                numeric[
                    "maximum_flat_rigid_line_compatibility_metres_per_second_squared"
                ]
            )
        )
    )


def _flat_line_audit(
    *,
    model: Any,
    kinematics: Any,
    original_velocity: NDArray[np.float64],
    projected_velocity: NDArray[np.float64],
    points: tuple[dict[str, Any], ...],
    active: tuple[int, ...],
) -> dict[str, Any]:
    first, second = (points[ordinal] for ordinal in active)
    if int(first["body_slot"]) != int(second["body_slot"]):
        raise ValueError("R129 flat points do not share one body")
    body_slot = int(first["body_slot"])
    first_local = np.asarray(first["local_translation_metres"], dtype=np.float64)
    second_local = np.asarray(second["local_translation_metres"], dtype=np.float64)
    world_separation = kinematics.body_rotations[body_slot] @ (
        second_local - first_local
    )
    direction = world_separation / np.linalg.norm(world_separation)
    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    original_motion = propagate_motion(model, kinematics, original_velocity, zero)
    projected_motion = propagate_motion(model, kinematics, projected_velocity, zero)

    def compatibility(motion: Any) -> tuple[float, float, float]:
        first_acceleration = point_acceleration(
            kinematics, motion, body_slot, first_local
        )
        second_acceleration = point_acceleration(
            kinematics, motion, body_slot, second_local
        )
        observed = float(direction @ (second_acceleration - first_acceleration))
        perpendicular = np.cross(motion.angular_velocity[body_slot], direction)
        predicted = -float(np.linalg.norm(world_separation)) * float(
            perpendicular @ perpendicular
        )
        return observed, predicted, abs(observed - predicted)

    original = compatibility(original_motion)
    projected = compatibility(projected_motion)
    return {
        "original_line_compatibility_metres_per_second_squared": original[0],
        "original_centripetal_prediction_metres_per_second_squared": original[1],
        "original_identity_absolute_error": original[2],
        "projected_line_compatibility_metres_per_second_squared": projected[0],
        "projected_centripetal_prediction_metres_per_second_squared": projected[1],
        "projected_identity_absolute_error": projected[2],
    }


def _invalid_projection(
    reason: str, *, rank: int = 0, nullity: int = 0
) -> ProjectionAnalysis:
    return ProjectionAnalysis(
        status="INVALID",
        invalid_reason=reason,
        projected_velocity=None,
        delta_velocity=None,
        multiplier=None,
        rank=rank,
        nullity=nullity,
        smallest_retained_relative_singular_value=None,
        largest_null_relative_singular_value=None,
        maximum_active_velocity_absolute=None,
        scaled_kkt_residual=None,
        closed_form_to_kkt_maximum_absolute=None,
        projection_idempotence_error=None,
        mass_orthogonality_relative_error=None,
        kinetic_energy_before_joules=None,
        kinetic_energy_after_joules=None,
        kinetic_energy_increase_joules=None,
        gauge_velocity_effect_maximum_absolute=None,
    )


def _skew(vector: NDArray[np.float64]) -> NDArray[np.float64]:
    x, y, z = vector
    return np.asarray(((0.0, -z, y), (z, 0.0, -x), (-y, x, 0.0)))


def _point_active(modes: NDArray[np.uint8], point_ordinal: int) -> bool:
    side = point_ordinal // 2
    point = point_ordinal % 2
    return int(modes[side]) in ((1, 3) if point == 0 else (2, 3))


def _array_sha256(value: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r128_profile_path: Path,
    r128_module_path: Path,
    r128_tool_path: Path,
    r122_profile_path: Path,
    dynamics_kernel_path: Path,
    dynamics_conformance_module_path: Path,
    r122_tool_path: Path,
    r113_profile_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    collocation_lift_module_path: Path,
) -> None:
    source = profile["source"]
    identities = (
        (r128_profile_path, source["r128"]["profile_sha256"]),
        (r128_module_path, source["r128"]["module_sha256"]),
        (r128_tool_path, source["r128"]["tool_sha256"]),
        (r122_profile_path, source["r122"]["profile_sha256"]),
        (dynamics_kernel_path, source["r122"]["kernel_sha256"]),
        (
            dynamics_conformance_module_path,
            source["r122"]["conformance_module_sha256"],
        ),
        (r122_tool_path, source["r122"]["tool_sha256"]),
        (r113_profile_path, source["r113"]["profile_sha256"]),
        (r120_profile_path, source["r120"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (v9_complete_clip_path, source["v9_complete_clip_sha256"]),
        (collocation_lift_module_path, source["collocation_lift_module_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in identities):
        raise ValueError("R129 bound source file identity differs")


def _validate_source_contracts(
    *,
    profile: Mapping[str, Any],
    r128: Mapping[str, Any],
    r122: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    source = profile["source"]
    r128_id = r128.get("identities", {})
    r122_id = r122.get("identities", {})
    r113_id = r113.get("identities", {})
    r120_id = r120.get("identities", {})
    inherited_numeric = r128.get("numeric_contract", {})
    conformance = r128.get("r129_conformance_contract", {})
    r122_anchor_audits = r122.get("anchor_audits", ())
    r128_anchors = tuple(
        (
            row.get("collocation"),
            tuple(row.get("contact_modes", ())),
            row.get("expected_rank"),
            row.get("expected_multiplier_nullity"),
        )
        for row in conformance.get("frozen_anchors", ())
    )
    r129_anchors = tuple(
        (
            row.get("collocation"),
            tuple(row.get("contact_modes", ())),
            row.get("expected_rank"),
            row.get("expected_multiplier_nullity"),
        )
        for row in profile.get("frozen_anchors", ())
    )
    if (
        r128.get("status") != "COMPLETE"
        or r128.get("gate_decision")
        != "PERMIT_R129_REPORT_ONLY_TANGENT_PROJECTION_CONFORMANCE_ONLY"
        or r128.get("alternative_decision_audit", {}).get("selected_alternative")
        != "mass_metric_tangent_velocity_projection"
        or any(
            r128.get(key) != 0
            for key in (
                "state_projections",
                "mass_matrix_assemblies",
                "contact_jacobian_assemblies",
                "projection_factorizations",
                "projection_solves",
                "inverse_dynamics_evaluations",
                "physx_scene_runs",
                "training_runs",
            )
        )
        or r128.get("repository", {}).get("commit")
        != source["r128"]["repository_commit"]
        or r128.get("repository", {}).get("dirty") is not False
        or r128_id.get("profile_sha256") != source["r128"]["profile_sha256"]
        or r128_id.get("formulation_module_sha256") != source["r128"]["module_sha256"]
        or r128_id.get("tool_sha256") != source["r128"]["tool_sha256"]
        or r128_anchors != r129_anchors
        or conformance.get("maximum_real_mass_matrix_assemblies") != 7
        or conformance.get("maximum_real_contact_jacobian_assemblies") != 6
        or conformance.get("maximum_real_projection_factorizations") != 6
        or conformance.get("maximum_real_projection_solves") != 6
        or conformance.get("maximum_full_schedule_projection_count") != 0
        or conformance.get("maximum_real_inverse_dynamics_evaluations") != 0
        or any(
            inherited_numeric.get(key) != profile["numeric_contract"].get(key)
            for key in (
                "rank_revealing_relative_null_maximum",
                "rank_revealing_relative_retained_minimum",
                "maximum_active_point_velocity_absolute_metres_per_second",
                "maximum_scaled_kkt_residual",
                "maximum_projection_idempotence_error",
                "maximum_mass_orthogonality_relative_error",
                "maximum_kinetic_energy_increase_joules",
            )
        )
        or r122.get("status") != "PASS"
        or r122.get("gate_decision")
        != "PERMIT_R123_SINGLE_BOUNDED_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or r122.get("repository", {}).get("commit")
        != source["r122"]["repository_commit"]
        or r122.get("repository", {}).get("dirty") is not False
        or r122_id.get("profile_sha256") != source["r122"]["profile_sha256"]
        or r122_id.get("kernel_module_sha256") != source["r122"]["kernel_sha256"]
        or r122_id.get("conformance_module_sha256")
        != source["r122"]["conformance_module_sha256"]
        or r122_id.get("tool_sha256") != source["r122"]["tool_sha256"]
        or len(r122_anchor_audits) != 7
        or any(not _r122_anchor_audit_passes(row) for row in r122_anchor_audits)
        or r113.get("status") != "PASS"
        or r113_id.get("profile_sha256") != source["r113"]["profile_sha256"]
        or r120.get("status") != "PASS"
        or r120_id.get("execution_profile_sha256") != source["r120"]["profile_sha256"]
        or r120_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
        or r122_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
    ):
        raise ValueError("R129 source report contract differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R129 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R129 {label} canonical report identity differs")
    return report


def _r122_anchor_audit_passes(row: Mapping[str, Any]) -> bool:
    contact = row.get("contact_kinematics", {})
    jdot_v = row.get("jdot_v", {})
    contact_points = contact.get("points", ())
    active_points = jdot_v.get("points", ())
    return bool(
        contact.get("status") == "PASS"
        and row.get("dynamics_round_trip", {}).get("status") == "PASS"
        and jdot_v.get("status") == "PASS"
        and row.get("mass_matrix", {}).get("status") == "PASS"
        and len(contact_points) == 4
        and all(point.get("status") == "PASS" for point in contact_points)
        and len(active_points) == jdot_v.get("active_point_count")
        and all(point.get("status") == "PASS" for point in active_points)
    )


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    numeric = profile.get("numeric_contract", {})
    expected_anchors = (
        (0, 0, 0, (0, 3), (2, 3), 5, 1),
        (952, 238, 0, (0, 2), (3,), 3, 0),
        (976, 244, 0, (0, 2), (3,), 3, 0),
        (996, 249, 0, (0, 0), (), 0, 0),
        (1312, 328, 0, (2, 0), (1,), 3, 0),
        (2504, 626, 0, (3, 0), (0, 1), 5, 1),
        (3199, 799, 3, (3, 0), (0, 1), 5, 1),
    )
    actual_anchors = tuple(
        (
            row.get("collocation"),
            row.get("interval"),
            row.get("substep"),
            tuple(row.get("contact_modes", ())),
            tuple(row.get("active_point_ordinals", ())),
            row.get("expected_rank"),
            row.get("expected_multiplier_nullity"),
        )
        for row in profile.get("frozen_anchors", ())
    )
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnlyConformance"
        or profile.get("claim")
        != "SevenAnchorMassMetricTangentVelocityProjectionConformanceOnly"
        or scope.get("run_id") != "R129"
        or scope.get("frozen_anchor_count") != 7
        or scope.get("maximum_real_state_projections") != 6
        or scope.get("maximum_real_mass_matrix_assemblies") != 7
        or scope.get("maximum_real_contact_jacobian_assemblies") != 6
        or scope.get("maximum_real_projection_factorizations") != 6
        or scope.get("maximum_real_projection_solves") != 6
        or scope.get("full_schedule_projections") != 0
        or scope.get("inverse_dynamics_evaluations") != 0
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or actual_anchors != expected_anchors
        or numeric.get("rank_revealing_relative_null_maximum") != 1.0e-12
        or numeric.get("rank_revealing_relative_retained_minimum") != 1.0e-10
        or numeric.get("maximum_mass_matrix_relative_symmetry") != 1.0e-10
        or numeric.get("maximum_active_point_velocity_absolute_metres_per_second")
        != 1.0e-10
        or numeric.get("maximum_scaled_kkt_residual") != 1.0e-10
        or numeric.get("maximum_closed_form_to_kkt_absolute") != 1.0e-10
        or numeric.get("maximum_projection_idempotence_error") != 1.0e-10
        or numeric.get("maximum_mass_orthogonality_relative_error") != 1.0e-10
        or numeric.get("maximum_kinetic_energy_increase_joules") != 1.0e-12
        or numeric.get("maximum_gauge_velocity_effect_absolute") != 1.0e-10
        or numeric.get(
            "maximum_flat_rigid_line_compatibility_metres_per_second_squared"
        )
        != 1.0e-10
        or profile.get("decision", {}).get("pass")
        != "PERMIT_R130_SINGLE_BOUNDED_TANGENT_PROJECTED_FIXED_PD_EXECUTION_ONLY"
        or profile.get("decision", {}).get("fail")
        != "STOP_INVALID_PROJECTION_IMPLEMENTATION"
        or profile.get("bounded_acceptance") != BOUNDED_ACCEPTANCE
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != (
            "ruff_check",
            "ruff_format",
            "single_thread_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("R129 conformance profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R129 conformance requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], environment: Mapping[str, str]
) -> None:
    if dict(environment) != profile["single_thread_environment"]:
        raise ValueError("R129 single-thread environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R129 conformance validation differs")
    return normalized


def _linux_thread_count() -> int:
    task = Path("/proc/self/task")
    return len(tuple(task.iterdir())) if task.is_dir() else 1


def _resident_memory_bytes() -> int:
    usage = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return int(usage) * 1024
