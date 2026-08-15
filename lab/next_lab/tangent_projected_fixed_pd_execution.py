from __future__ import annotations

import hashlib
import io
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
from next_lab.fixed_pd_inverse_dynamics_execution import (
    CollocationState,
    FixedPdSchedule,
    _column,
    _record_mask,
    collocation_state,
    point_active,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    COLLOCATION_COUNT,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
    MOTOR_INTERVAL_COUNT,
    POINT_COUNT,
    SUBSTEPS_PER_INTERVAL,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    SpatialModel,
    build_spatial_model,
    mass_matrix,
    point_jacobian,
)
from next_lab.gauge_aware_fixed_pd_execution import (
    _collocation_row,
    aggregate_execution_rows,
    solve_gauge_aware_collocation,
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import (
    build_reduced_local_system,
    reduced_column_scale,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor
from next_lab.tangent_velocity_projection_conformance import (
    _analysis_metrics,
    _flat_line_audit,
    _projection_passes,
    project_tangent_velocity,
)

EXECUTION_ID = "nextengine.humanoid-tangent-projected-fixed-pd-execution.v1"
CHECK_ID = "TRAIN-4-TANGENT-PROJECTED-FIXED-PD-EXECUTION"
POINT_FORCE_WIDTH = 3
ZERO_DOWNSTREAM_COUNTERS = (
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)
BOUNDED_ACCEPTANCE = {
    "r123_retry": "NOT_AUTHORIZED",
    "r127_retry": "NOT_AUTHORIZED",
    "r129_retry": "NOT_AUTHORIZED",
    "r130_retry": "NOT_AUTHORIZED",
    "additional_projected_fixed_pd_execution": "NOT_AUTHORIZED",
    "additional_projection_execution": "NOT_AUTHORIZED",
    "additional_inverse_dynamics_execution": "NOT_AUTHORIZED",
    "projected_state_kinodynamic_formulation": "AUTHORIZED_REPORT_ONLY_ON_VALID_FEASIBLE_R130",
    "kinodynamic_solve": "NOT_AUTHORIZED",
    "candidate_artifact": "NOT_AUTHORIZED",
    "physx": "NOT_AUTHORIZED",
    "all_17": "NOT_AUTHORIZED",
    "full_v19": "NOT_AUTHORIZED",
    "training": "NOT_AUTHORIZED",
}
CONTROLLER_LINEAGE = {
    "hard_rom_violation_count": 0,
    "hard_rom_violation_dof_ordinals": [],
    "maximum_hard_rom_excess_microradians": 0.0,
    "target_soft_clamp_count": 0,
    "target_soft_clamp_dof_ordinals": [],
    "target_slew_count": 1,
    "target_slew_dof_ordinals": [6],
    "maximum_target_lag_microradians": 1672.0,
}


@dataclass(frozen=True)
class ProjectedExecutionOutcome:
    status: str
    feasibility: str | None
    invalid_reason: str | None
    projection_systems: int
    projection_factorizations: int
    projection_solves: int
    controller_schedule_derivations: int
    inverse_dynamics_system_assemblies: int
    inverse_dynamics_singular_value_decompositions: int
    inverse_dynamics_particular_solutions: int
    gauge_interval_classifications: int
    projection_rows: tuple[dict[str, Any], ...]
    solver_rows: tuple[dict[str, Any], ...]
    projection_aggregate: dict[str, Any]
    solver_aggregate: dict[str, Any]
    fixed_pd_schedule: FixedPdSchedule | None
    arrays: dict[str, NDArray[Any]] | None
    resource_usage: dict[str, Any]
    ordered_projection_system_sha256: str
    ordered_reduced_system_sha256: str


def execute_and_build_tangent_projected_fixed_pd_report(
    *,
    profile_path: Path,
    r129_report_path: Path,
    r129_profile_path: Path,
    projection_module_path: Path,
    r129_tool_path: Path,
    r128_report_path: Path,
    r128_profile_path: Path,
    r126_report_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    collocation_lift_module_path: Path,
    dynamics_kernel_path: Path,
    gauge_execution_module_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> tuple[dict[str, Any], bytes | None]:
    """Consume the sole R130 authority and return its report/cache bytes."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r129_report_path,
            r129_profile_path,
            projection_module_path,
            r129_tool_path,
            r128_report_path,
            r128_profile_path,
            r126_report_path,
            r126_profile_path,
            gauge_aware_kernel_path,
            r126_conformance_module_path,
            r126_tool_path,
            r121_report_path,
            r121_profile_path,
            r113_report_path,
            r113_profile_path,
            r120_report_path,
            r120_profile_path,
            r120_cache_path,
            v9_complete_clip_path,
            collocation_lift_module_path,
            dynamics_kernel_path,
            gauge_execution_module_path,
            tool_path,
        )
    )
    (
        profile_path,
        r129_report_path,
        r129_profile_path,
        projection_module_path,
        r129_tool_path,
        r128_report_path,
        r128_profile_path,
        r126_report_path,
        r126_profile_path,
        gauge_aware_kernel_path,
        r126_conformance_module_path,
        r126_tool_path,
        r121_report_path,
        r121_profile_path,
        r113_report_path,
        r113_profile_path,
        r120_report_path,
        r120_profile_path,
        r120_cache_path,
        v9_complete_clip_path,
        collocation_lift_module_path,
        dynamics_kernel_path,
        gauge_execution_module_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R130 execution input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    r129 = _load_bound_report(r129_report_path, profile["source"]["r129"], "R129")
    r128 = _load_bound_report(r128_report_path, profile["source"]["r128"], "R128")
    r126 = _load_bound_report(r126_report_path, profile["source"]["r126"], "R126")
    r121 = _load_bound_report(r121_report_path, profile["source"]["r121"], "R121")
    r113 = _load_bound_report(r113_report_path, profile["source"]["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, profile["source"]["r120"], "R120")
    _validate_source_files(
        profile=profile,
        r129_profile_path=r129_profile_path,
        projection_module_path=projection_module_path,
        r129_tool_path=r129_tool_path,
        r128_profile_path=r128_profile_path,
        r126_profile_path=r126_profile_path,
        gauge_aware_kernel_path=gauge_aware_kernel_path,
        r126_conformance_module_path=r126_conformance_module_path,
        r126_tool_path=r126_tool_path,
        r121_profile_path=r121_profile_path,
        r113_profile_path=r113_profile_path,
        r120_profile_path=r120_profile_path,
        r120_cache_path=r120_cache_path,
        v9_complete_clip_path=v9_complete_clip_path,
        collocation_lift_module_path=collocation_lift_module_path,
        dynamics_kernel_path=dynamics_kernel_path,
        gauge_execution_module_path=gauge_execution_module_path,
    )
    _validate_source_contracts(
        profile=profile,
        r129=r129,
        r128=r128,
        r126=r126,
        r121=r121,
        r113=r113,
        r120=r120,
    )
    source = profile["source"]
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != source["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve()) != source["execution_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R130 current implementation identity differs")

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R130 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=r113)

    outcome = execute_tangent_projected_fixed_pd(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        descriptor=descriptor,
    )
    cache_bytes, cache_identity = encode_solver_private_cache(
        outcome=outcome,
        profile_sha256=sha256(profile_path),
        r129_report_sha256=r129["report_sha256"],
        r126_report_sha256=r126["report_sha256"],
    )
    valid = outcome.status == "VALID_COMPLETE"
    if valid and outcome.feasibility == "FEASIBLE":
        decision_key = "valid_feasible"
    elif valid and outcome.feasibility == "INFEASIBLE":
        decision_key = "valid_infeasible"
    else:
        decision_key = "invalid"
    fixed_pd_audit = (
        None if outcome.fixed_pd_schedule is None else outcome.fixed_pd_schedule.audit
    )
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": "COMPLETE" if valid else "INVALID",
        "claim": profile["claim"],
        "gate_decision": profile["decision"][decision_key],
        "scope": profile["scope"],
        "source_gates": {
            "r129_status": r129["status"],
            "r129_report_sha256": r129["report_sha256"],
            "r128_status": r128["status"],
            "r128_report_sha256": r128["report_sha256"],
            "r126_status": r126["status"],
            "r126_report_sha256": r126["report_sha256"],
            "r121_status": r121["status"],
            "r121_report_sha256": r121["report_sha256"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "numeric_contract": profile["numeric_contract"],
        "projection_result": {
            "status": (
                "PASS"
                if len(outcome.projection_rows) == COLLOCATION_COUNT
                and all(row["status"] == "PASS" for row in outcome.projection_rows)
                else "FAIL"
            ),
            "projection_systems": outcome.projection_systems,
            "projection_factorizations": outcome.projection_factorizations,
            "projection_solves": outcome.projection_solves,
            "ordered_projection_system_float64_sha256": outcome.ordered_projection_system_sha256,
            "aggregate": outcome.projection_aggregate,
            "collocations": list(outcome.projection_rows),
        },
        "projected_fixed_pd_schedule_audit": fixed_pd_audit,
        "solver_result": {
            "status": outcome.status,
            "feasibility": outcome.feasibility,
            "invalid_reason": outcome.invalid_reason,
            "inverse_dynamics_system_assemblies": outcome.inverse_dynamics_system_assemblies,
            "singular_value_decompositions": outcome.inverse_dynamics_singular_value_decompositions,
            "particular_solutions": outcome.inverse_dynamics_particular_solutions,
            "gauge_interval_classifications": outcome.gauge_interval_classifications,
            "ordered_reduced_system_float64_sha256": outcome.ordered_reduced_system_sha256,
            "aggregate": outcome.solver_aggregate,
            "collocations": list(outcome.solver_rows),
        },
        "solver_private_cache": cache_identity,
        "resource_usage": outcome.resource_usage,
        "execution_environment": dict(execution_environment),
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r129_report_file_sha256": sha256(r129_report_path),
            "r129_profile_sha256": sha256(r129_profile_path),
            "projection_module_sha256": sha256(projection_module_path),
            "r129_tool_sha256": sha256(r129_tool_path),
            "r128_report_file_sha256": sha256(r128_report_path),
            "r128_profile_sha256": sha256(r128_profile_path),
            "r126_report_file_sha256": sha256(r126_report_path),
            "r126_profile_sha256": sha256(r126_profile_path),
            "gauge_aware_kernel_sha256": sha256(gauge_aware_kernel_path),
            "r126_conformance_module_sha256": sha256(r126_conformance_module_path),
            "r126_tool_sha256": sha256(r126_tool_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_profile_sha256": sha256(r120_profile_path),
            "r120_cache_sha256": sha256(r120_cache_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "collocation_lift_module_sha256": sha256(collocation_lift_module_path),
            "dynamics_kernel_sha256": sha256(dynamics_kernel_path),
            "gauge_execution_module_sha256": sha256(gauge_execution_module_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "result_transition": profile["result_transition"][decision_key],
        "bounded_acceptance": profile["bounded_acceptance"],
        "projected_fixed_pd_execution_runs": 1,
        "state_projection_systems": outcome.projection_systems,
        "projection_factorizations": outcome.projection_factorizations,
        "projection_solves": outcome.projection_solves,
        "controller_schedule_derivations": outcome.controller_schedule_derivations,
        "inverse_dynamics_system_assemblies": outcome.inverse_dynamics_system_assemblies,
        "inverse_dynamics_singular_value_decompositions": outcome.inverse_dynamics_singular_value_decompositions,
        "inverse_dynamics_particular_solutions": outcome.inverse_dynamics_particular_solutions,
        "force_gauge_interval_classifications": outcome.gauge_interval_classifications,
        "local_system_solves": 0,
        **{counter: 0 for counter in ZERO_DOWNSTREAM_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report, cache_bytes


def execute_tangent_projected_fixed_pd(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    descriptor: Mapping[str, Any],
) -> ProjectedExecutionOutcome:
    """Consume all 3200 R130 collocations once without integration or retry."""

    budget = profile["execution_budget"]
    numeric = profile["numeric_contract"]
    expected = profile["expected_inventory"]
    np.random.seed(int(budget["random_seed"]))
    started = time.monotonic()
    maximum_rss = _resident_memory_bytes()
    maximum_threads = _linux_thread_count()
    resource_passed = True
    projection_digest = hashlib.sha256()
    reduced_digest = hashlib.sha256()
    projected_velocity = np.empty(
        (COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64
    )
    delta_velocity = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    interval_index = np.repeat(
        np.arange(MOTOR_INTERVAL_COUNT, dtype=np.int64), SUBSTEPS_PER_INTERVAL
    )
    substep_index = np.tile(
        np.arange(SUBSTEPS_PER_INTERVAL, dtype=np.int64), MOTOR_INTERVAL_COUNT
    )
    projection_rows: list[dict[str, Any]] = []
    solver_rows: list[dict[str, Any]] = []
    projection_systems = 0
    projection_factorizations = 0
    projection_solves = 0
    controller_derivations = 0
    system_assemblies = 0
    decompositions = 0
    particulars = 0
    gauge_classifications = 0
    invalid_reason: str | None = None

    for collocation in range(COLLOCATION_COUNT):
        interval = int(interval_index[collocation])
        substep = int(substep_index[collocation])
        base_state = collocation_state(cache, interval, substep)
        modes = contact_modes[interval]
        active = tuple(
            point for point in range(POINT_COUNT) if point_active(modes, point)
        )
        projection_digest.update(np.ascontiguousarray(modes).tobytes())
        projection_digest.update(np.ascontiguousarray(base_state.velocity).tobytes())
        if not active:
            projected = np.array(base_state.velocity, copy=True)
            delta = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
            projection_rows.append(
                _flight_projection_row(
                    collocation=collocation,
                    interval=interval,
                    substep=substep,
                    modes=modes,
                    velocity=base_state.velocity,
                )
            )
        else:
            matrix, kinematics = mass_matrix(model, base_state.configuration)
            jacobian = np.vstack(
                [
                    point_jacobian(
                        model,
                        kinematics,
                        int(points[point]["body_slot"]),
                        np.asarray(
                            points[point]["local_translation_metres"],
                            dtype=np.float64,
                        ),
                    )
                    for point in active
                ]
            )
            projection_systems += 1
            projection_factorizations += 1
            projection_solves += 1
            projection_digest.update(np.ascontiguousarray(matrix).tobytes())
            projection_digest.update(np.ascontiguousarray(jacobian).tobytes())
            expected_rank = 3 if len(active) == 1 else 5
            analysis = project_tangent_velocity(
                mass=matrix,
                jacobian=jacobian,
                velocity=base_state.velocity,
                expected_rank=expected_rank,
                numeric=numeric,
            )
            if analysis.projected_velocity is None or analysis.delta_velocity is None:
                projected = np.full(GENERALIZED_WIDTH, np.nan, dtype=np.float64)
                delta = projected.copy()
                flat_audit = None
                passed = False
            else:
                projected = analysis.projected_velocity
                delta = analysis.delta_velocity
                flat_audit = (
                    _flat_line_audit(
                        model=model,
                        kinematics=kinematics,
                        original_velocity=base_state.velocity,
                        projected_velocity=projected,
                        points=points,
                        active=active,
                    )
                    if len(active) == 2
                    else None
                )
                passed = _projection_passes(
                    analysis=analysis,
                    numeric=numeric,
                    flat_audit=flat_audit,
                )
            projection_rows.append(
                _active_projection_row(
                    collocation=collocation,
                    interval=interval,
                    substep=substep,
                    modes=modes,
                    active=active,
                    analysis=analysis,
                    jacobian=jacobian,
                    original_velocity=base_state.velocity,
                    projected_velocity=projected,
                    flat_audit=flat_audit,
                    passed=passed,
                )
            )
            if not passed:
                invalid_reason = (
                    analysis.invalid_reason or "PROJECTION_CONFORMANCE_GUARD_FAILED"
                )
                break
        projected_velocity[collocation] = projected
        delta_velocity[collocation] = delta
        if collocation % 64 == 63:
            resource_passed, maximum_rss, maximum_threads, resource_reason = (
                _check_resource_budget(
                    budget=budget,
                    started=started,
                    maximum_rss=maximum_rss,
                    maximum_threads=maximum_threads,
                )
            )
            if not resource_passed:
                invalid_reason = resource_reason
                break

    if len(projection_rows) == COLLOCATION_COUNT and (
        projection_systems != int(expected["projection_systems"])
        or projection_factorizations != int(expected["projection_factorizations"])
        or projection_solves != int(expected["projection_solves"])
    ):
        invalid_reason = invalid_reason or "PROJECTION_COUNTER_CLOSURE_DIFFERS"
    if len(projection_rows) != COLLOCATION_COUNT and invalid_reason is None:
        invalid_reason = "PROJECTION_SCHEDULE_CLOSURE_DIFFERS"

    fixed_pd: FixedPdSchedule | None = None
    if invalid_reason is None:
        fixed_pd = derive_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=descriptor,
            projected_velocity=projected_velocity,
        )
        controller_derivations = 1
        if not _projected_controller_lineage_passes(profile, fixed_pd.audit):
            invalid_reason = "PROJECTED_FIXED_PD_SCHEDULE_INVALID"

    acceleration = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    point_force = np.zeros(
        (COLLOCATION_COUNT, POINT_COUNT, POINT_FORCE_WIDTH), dtype=np.float64
    )
    interval_lower = np.full(COLLOCATION_COUNT, np.nan, dtype=np.float64)
    interval_upper = np.full(COLLOCATION_COUNT, np.nan, dtype=np.float64)
    witness_alpha = np.full(COLLOCATION_COUNT, np.nan, dtype=np.float64)
    collocation_feasible = np.empty(COLLOCATION_COUNT, dtype=np.uint8)
    effort = (
        None
        if fixed_pd is None
        else fixed_pd.applied_effort_micronewton_metres / 1_000_000.0
    )
    if invalid_reason is None:
        assert effort is not None
        for collocation in range(COLLOCATION_COUNT):
            interval = int(interval_index[collocation])
            substep = int(substep_index[collocation])
            base_state = collocation_state(cache, interval, substep)
            projected_state = CollocationState(
                configuration=base_state.configuration,
                velocity=projected_velocity[collocation],
                warm_acceleration=base_state.warm_acceleration,
            )
            system = build_reduced_local_system(
                model=model,
                state=projected_state,
                effort_newton_metres=effort[collocation],
                modes=contact_modes[interval],
                points=points,
            )
            system_assemblies += 1
            reduced_digest.update(np.ascontiguousarray(system.matrix).tobytes())
            reduced_digest.update(
                np.ascontiguousarray(system.right_hand_side).tobytes()
            )
            column_scale = reduced_column_scale(
                model=model,
                cache=cache,
                active_point_count=len(system.active_point_ordinals),
            )
            solution = solve_gauge_aware_collocation(
                system,
                column_scale=column_scale,
                numeric_contract=numeric,
            )
            decompositions += 1
            if solution.particular_solution is not None:
                particulars += 1
            if solution.gauge_feasibility is not None:
                gauge_classifications += 1
            solver_rows.append(
                _collocation_row(
                    collocation=collocation,
                    interval=interval,
                    substep=substep,
                    system=system,
                    solution=solution,
                )
            )
            if solution.status != "VALID":
                invalid_reason = solution.invalid_reason
                break
            assert solution.selected_solution is not None
            selected = solution.selected_solution
            acceleration[collocation] = selected[:GENERALIZED_WIDTH]
            active_forces = selected[GENERALIZED_WIDTH:].reshape(
                len(system.active_point_ordinals), POINT_FORCE_WIDTH
            )
            for local, point in enumerate(system.active_point_ordinals):
                point_force[collocation, point] = active_forces[local]
            collocation_feasible[collocation] = int(solution.feasibility == "FEASIBLE")
            if solution.gauge_feasibility is not None:
                gauge = solution.gauge_feasibility
                if gauge.lower is not None:
                    interval_lower[collocation] = gauge.lower
                if gauge.upper is not None:
                    interval_upper[collocation] = gauge.upper
                if gauge.witness_alpha is not None:
                    witness_alpha[collocation] = gauge.witness_alpha
            if collocation % 64 == 63:
                resource_passed, maximum_rss, maximum_threads, resource_reason = (
                    _check_resource_budget(
                        budget=budget,
                        started=started,
                        maximum_rss=maximum_rss,
                        maximum_threads=maximum_threads,
                    )
                )
                if not resource_passed:
                    invalid_reason = resource_reason
                    break

    if len(solver_rows) == COLLOCATION_COUNT and (
        system_assemblies != int(expected["inverse_dynamics_system_assemblies"])
        or decompositions
        != int(expected["inverse_dynamics_singular_value_decompositions"])
        or particulars != int(expected["inverse_dynamics_particular_solutions"])
        or gauge_classifications
        != int(expected["force_gauge_interval_classifications"])
    ):
        invalid_reason = invalid_reason or "INVERSE_DYNAMICS_COUNTER_CLOSURE_DIFFERS"
    if (
        fixed_pd is not None
        and fixed_pd.audit["status"] == "PASS"
        and len(solver_rows) != COLLOCATION_COUNT
        and invalid_reason is None
    ):
        invalid_reason = "INVERSE_DYNAMICS_SCHEDULE_CLOSURE_DIFFERS"

    resource_passed, maximum_rss, maximum_threads, resource_reason = (
        _check_resource_budget(
            budget=budget,
            started=started,
            maximum_rss=maximum_rss,
            maximum_threads=maximum_threads,
        )
    )
    if not resource_passed:
        invalid_reason = resource_reason
    valid = invalid_reason is None and len(solver_rows) == COLLOCATION_COUNT
    feasibility: str | None = None
    arrays: dict[str, NDArray[Any]] | None = None
    if valid:
        feasibility = (
            "FEASIBLE" if bool(np.all(collocation_feasible == 1)) else "INFEASIBLE"
        )
        assert fixed_pd is not None and effort is not None
        arrays = {
            "projected_generalized_velocity": projected_velocity,
            "projection_delta_velocity": delta_velocity,
            "generalized_acceleration": acceleration,
            "applied_target_microradians": fixed_pd.applied_target_microradians,
            "applied_effort_newton_metres": effort,
            "point_force_normal_right_forward_newtons": point_force,
            "gauge_interval_lower": interval_lower,
            "gauge_interval_upper": interval_upper,
            "gauge_witness_alpha": witness_alpha,
            "collocation_feasible": collocation_feasible,
            "interval_index": interval_index,
            "substep_index": substep_index,
        }
    elapsed = time.monotonic() - started
    projection_aggregate = aggregate_projection_rows(
        projection_rows,
        projected_velocity=projected_velocity
        if len(projection_rows) == COLLOCATION_COUNT
        else None,
        delta_velocity=delta_velocity
        if len(projection_rows) == COLLOCATION_COUNT
        else None,
    )
    solver_aggregate = aggregate_execution_rows(solver_rows, arrays)
    return ProjectedExecutionOutcome(
        status="VALID_COMPLETE" if valid else "INVALID",
        feasibility=feasibility,
        invalid_reason=invalid_reason,
        projection_systems=projection_systems,
        projection_factorizations=projection_factorizations,
        projection_solves=projection_solves,
        controller_schedule_derivations=controller_derivations,
        inverse_dynamics_system_assemblies=system_assemblies,
        inverse_dynamics_singular_value_decompositions=decompositions,
        inverse_dynamics_particular_solutions=particulars,
        gauge_interval_classifications=gauge_classifications,
        projection_rows=tuple(projection_rows),
        solver_rows=tuple(solver_rows),
        projection_aggregate=projection_aggregate,
        solver_aggregate=solver_aggregate,
        fixed_pd_schedule=fixed_pd,
        arrays=arrays,
        resource_usage={
            "status": "PASS" if resource_passed else "FAIL",
            "wall_clock_seconds": elapsed,
            "maximum_resident_memory_bytes": maximum_rss,
            "maximum_observed_os_thread_count": maximum_threads,
            "execution_process_count": 1,
            "child_processes_spawned_during_execution": 0,
            "random_seed": int(budget["random_seed"]),
            "randomized_restart_count": 0,
            "resume_or_warm_restart": "FORBIDDEN_AND_NOT_USED",
            "manual_intervention": "FORBIDDEN_AND_NOT_USED",
        },
        ordered_projection_system_sha256=projection_digest.hexdigest(),
        ordered_reduced_system_sha256=reduced_digest.hexdigest(),
    )


def derive_projected_fixed_pd_schedule(
    *,
    cache: Mapping[str, NDArray[Any]],
    descriptor: Mapping[str, Any],
    projected_velocity: NDArray[np.float64],
) -> FixedPdSchedule:
    """Recompute the R121 sequential controller from projected velocity."""

    if projected_velocity.shape != (COLLOCATION_COUNT, GENERALIZED_WIDTH) or not np.all(
        np.isfinite(projected_velocity)
    ):
        raise ValueError("R130 projected fixed-PD velocity differs")
    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    if [int(row["dof_ordinal"]) for row in joints] != list(range(ACTUATOR_COUNT)) or [
        int(row["dof_ordinal"]) for row in actuators
    ] != list(range(ACTUATOR_COUNT)):
        raise ValueError("R130 projected fixed-PD descriptor order differs")
    position = np.asarray(cache["joint_position_rad"], dtype=np.float64)
    if position.shape != (FRAME_COUNT, ACTUATOR_COUNT) or not np.all(
        np.isfinite(position)
    ):
        raise ValueError("R130 projected fixed-PD position differs")
    position_urad = np.rint(position * 1_000_000.0)
    hard_min = _column(joints, "hard_limit_microradians", 0)
    hard_max = _column(joints, "hard_limit_microradians", 1)
    soft_min = _column(joints, "soft_limit_microradians", 0)
    soft_max = _column(joints, "soft_limit_microradians", 1)
    maximum_velocity = np.asarray(
        [row["maximum_velocity_microradians_per_second"] for row in joints],
        dtype=np.float64,
    )
    target_delta = np.asarray(
        [
            max(
                abs(int(value))
                for value in row["target_delta_microradians_per_motor_tick"]
            )
            for row in actuators
        ],
        dtype=np.float64,
    )
    stiffness = np.asarray(
        [row["stiffness_q16"] for row in actuators], dtype=np.float64
    )
    damping = np.asarray([row["damping_q16"] for row in actuators], dtype=np.float64)
    effort_min = _column(actuators, "effort_micronewton_metres", 0)
    effort_max = _column(actuators, "effort_micronewton_metres", 1)
    effort_rate = np.asarray(
        [row["maximum_effort_rate_micronewton_metres_per_second"] for row in actuators],
        dtype=np.float64,
    )
    maximum_power = np.asarray(
        [row["maximum_power_microwatts"] for row in actuators], dtype=np.float64
    )
    maximum_work = np.asarray(
        [row["maximum_positive_work_microjoules_per_motor_tick"] for row in actuators],
        dtype=np.float64,
    )
    maximum_effort_delta = np.rint(effort_rate / 240.0)
    target_output = np.empty((MOTOR_INTERVAL_COUNT, ACTUATOR_COUNT), dtype=np.float64)
    effort_output = np.empty((COLLOCATION_COUNT, ACTUATOR_COUNT), dtype=np.float64)
    count_names = (
        "hard_rom_violation",
        "velocity_violation",
        "target_soft_clamp",
        "target_slew",
        "static_effort_clamp",
        "effort_rate_clamp",
        "power_clamp",
        "positive_work_clamp",
        "infeasible_effort_envelope",
    )
    counts = {name: 0 for name in count_names}
    channels: dict[str, set[int]] = {name: set() for name in count_names}
    maxima = {
        "hard_rom_excess_microradians": 0.0,
        "velocity_excess_microradians_per_second": 0.0,
        "target_lag_microradians": 0.0,
        "absolute_requested_effort_micronewton_metres": 0.0,
        "absolute_applied_effort_micronewton_metres": 0.0,
        "absolute_power_microwatts": 0.0,
        "positive_work_microjoules_per_motor_tick": 0.0,
    }
    applied_target = position_urad[0].copy()
    previous_effort = np.zeros(ACTUATOR_COUNT, dtype=np.float64)
    for interval in range(MOTOR_INTERVAL_COUNT):
        target = np.minimum(np.maximum(position_urad[interval], soft_min), soft_max)
        _record_mask(
            target != position_urad[interval], "target_soft_clamp", counts, channels
        )
        next_target = np.minimum(
            np.maximum(target, applied_target - target_delta),
            applied_target + target_delta,
        )
        _record_mask(next_target != target, "target_slew", counts, channels)
        applied_target = next_target
        target_output[interval] = applied_target
        maxima["target_lag_microradians"] = max(
            maxima["target_lag_microradians"],
            float(np.max(np.abs(applied_target - position_urad[interval]))),
        )
        used_work = np.zeros(ACTUATOR_COUNT, dtype=np.float64)
        for substep in range(SUBSTEPS_PER_INTERVAL):
            collocation = interval * SUBSTEPS_PER_INTERVAL + substep
            fraction = substep / SUBSTEPS_PER_INTERVAL
            observed_position = np.rint(
                (
                    (1.0 - fraction) * position[interval]
                    + fraction * position[interval + 1]
                )
                * 1_000_000.0
            )
            observed_velocity = np.rint(
                projected_velocity[collocation, 6:] * 1_000_000.0
            )
            hard_excess = np.maximum(
                hard_min - observed_position, observed_position - hard_max
            )
            velocity_excess = np.abs(observed_velocity) - maximum_velocity
            _record_mask(hard_excess > 10.0, "hard_rom_violation", counts, channels)
            _record_mask(velocity_excess > 0.0, "velocity_violation", counts, channels)
            maxima["hard_rom_excess_microradians"] = max(
                maxima["hard_rom_excess_microradians"],
                float(max(0.0, np.max(hard_excess))),
            )
            maxima["velocity_excess_microradians_per_second"] = max(
                maxima["velocity_excess_microradians_per_second"],
                float(max(0.0, np.max(velocity_excess))),
            )
            requested = np.rint(
                stiffness * (applied_target - observed_position) / 65_536.0
            ) - np.rint(damping * observed_velocity / 65_536.0)
            maxima["absolute_requested_effort_micronewton_metres"] = max(
                maxima["absolute_requested_effort_micronewton_metres"],
                float(np.max(np.abs(requested))),
            )
            rate_min = previous_effort - maximum_effort_delta
            rate_max = previous_effort + maximum_effort_delta
            absolute_velocity = np.abs(observed_velocity)
            moving = absolute_velocity > 0.0
            power_bound = np.full(ACTUATOR_COUNT, np.inf, dtype=np.float64)
            power_bound[moving] = np.floor(
                maximum_power[moving] * 1_000_000.0 / absolute_velocity[moving]
            )
            remaining_work = maximum_work - used_work
            work_bound = np.full(ACTUATOR_COUNT, np.inf, dtype=np.float64)
            work_bound[moving] = np.floor(
                remaining_work[moving] * 240.0 * 1_000_000.0 / absolute_velocity[moving]
            )
            lower = np.maximum(np.maximum(effort_min, rate_min), -power_bound)
            upper = np.minimum(np.minimum(effort_max, rate_max), power_bound)
            lower = np.where(
                observed_velocity < 0.0, np.maximum(lower, -work_bound), lower
            )
            upper = np.where(
                observed_velocity > 0.0, np.minimum(upper, work_bound), upper
            )
            _record_mask(
                (requested < effort_min) | (requested > effort_max),
                "static_effort_clamp",
                counts,
                channels,
            )
            _record_mask(
                (requested < rate_min) | (requested > rate_max),
                "effort_rate_clamp",
                counts,
                channels,
            )
            _record_mask(
                (requested < -power_bound) | (requested > power_bound),
                "power_clamp",
                counts,
                channels,
            )
            _record_mask(
                ((observed_velocity > 0.0) & (requested > work_bound))
                | ((observed_velocity < 0.0) & (requested < -work_bound)),
                "positive_work_clamp",
                counts,
                channels,
            )
            infeasible = (remaining_work < 0.0) | (lower > upper)
            _record_mask(infeasible, "infeasible_effort_envelope", counts, channels)
            current_effort = np.minimum(np.maximum(requested, lower), upper)
            charge = np.ceil(
                np.maximum(current_effort * observed_velocity, 0.0)
                / (240.0 * 1_000_000.0)
            )
            used_work += charge
            _record_mask(
                used_work > maximum_work,
                "infeasible_effort_envelope",
                counts,
                channels,
            )
            previous_effort = current_effort
            effort_output[collocation] = current_effort
            maxima["absolute_applied_effort_micronewton_metres"] = max(
                maxima["absolute_applied_effort_micronewton_metres"],
                float(np.max(np.abs(current_effort))),
            )
            maxima["absolute_power_microwatts"] = max(
                maxima["absolute_power_microwatts"],
                float(np.max(np.abs(current_effort * observed_velocity))) / 1_000_000.0,
            )
            maxima["positive_work_microjoules_per_motor_tick"] = max(
                maxima["positive_work_microjoules_per_motor_tick"],
                float(np.max(used_work)),
            )
    audit = {
        "status": (
            "PASS"
            if counts["hard_rom_violation"] == 0
            and counts["velocity_violation"] == 0
            and counts["infeasible_effort_envelope"] == 0
            else "FAIL"
        ),
        "collocation_count": COLLOCATION_COUNT,
        "activation_counts": counts,
        "activated_dof_ordinals": {
            name: sorted(values) for name, values in channels.items()
        },
        "maxima": maxima,
        "projected_velocity_float64_sha256": _array_sha256(projected_velocity),
        "applied_target_float64_sha256": _array_sha256(target_output),
        "applied_effort_float64_sha256": _array_sha256(effort_output),
        "claim_ceiling": "PROJECTED_FIXED_PD_INPUT_SCHEDULE_ONLY_NO_INTEGRATION",
    }
    return FixedPdSchedule(target_output, effort_output, audit)


def aggregate_projection_rows(
    rows: Sequence[Mapping[str, Any]],
    *,
    projected_velocity: NDArray[np.float64] | None,
    delta_velocity: NDArray[np.float64] | None,
) -> dict[str, Any]:
    passed = [row for row in rows if row["status"] == "PASS"]
    active = [row for row in passed if row["active_point_ordinals"]]
    flat = [row for row in active if len(row["active_point_ordinals"]) == 2]
    result: dict[str, Any] = {
        "collocation_rows_recorded": len(rows),
        "passing_collocations": len(passed),
        "flight_identity_collocations": sum(
            not row["active_point_ordinals"] for row in passed
        ),
        "single_point_projection_collocations": sum(
            len(row["active_point_ordinals"]) == 1 for row in passed
        ),
        "flat_foot_projection_collocations": len(flat),
        "first_failing_collocation": next(
            (int(row["collocation"]) for row in rows if row["status"] != "PASS"),
            None,
        ),
    }
    projection_fields = (
        "maximum_active_velocity_before_metres_per_second",
        "maximum_active_velocity_after_metres_per_second",
        "maximum_generalized_velocity_correction",
        "mass_metric_velocity_correction",
        "scaled_kkt_residual",
        "closed_form_to_kkt_maximum_absolute",
        "projection_idempotence_error",
        "mass_orthogonality_relative_error",
        "kinetic_energy_increase_joules",
        "gauge_velocity_effect_maximum_absolute",
    )
    for field in projection_fields:
        values = [
            float(row["projection"][field])
            for row in active
            if row["projection"].get(field) is not None
        ]
        output_field = field if field.startswith("maximum_") else f"maximum_{field}"
        result[output_field] = max(values) if values else None
    line_values = [
        abs(
            float(
                row["flat_rigid_line_audit"][
                    "projected_line_compatibility_metres_per_second_squared"
                ]
            )
        )
        for row in flat
        if row.get("flat_rigid_line_audit") is not None
    ]
    result["maximum_absolute_projected_flat_rigid_line_compatibility"] = (
        max(line_values) if line_values else None
    )
    if projected_velocity is not None and delta_velocity is not None:
        result["array_sha256"] = {
            "projected_generalized_velocity": _array_sha256(projected_velocity),
            "projection_delta_velocity": _array_sha256(delta_velocity),
        }
    return result


def encode_solver_private_cache(
    *,
    outcome: ProjectedExecutionOutcome,
    profile_sha256: str,
    r129_report_sha256: str,
    r126_report_sha256: str,
) -> tuple[bytes | None, dict[str, Any]]:
    if outcome.arrays is None:
        return None, {
            "status": "NOT_EMITTED_INVALID_EXECUTION",
            "candidate_or_corpus_authority": False,
        }
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r130-tangent-projected-fixed-pd-solver-private.v1",
        "candidate_or_corpus_authority": False,
        "collocation_count": COLLOCATION_COUNT,
        "feasibility": outcome.feasibility,
        "profile_sha256": profile_sha256,
        "r129_report_sha256": r129_report_sha256,
        "r126_report_sha256": r126_report_sha256,
    }
    arrays = {
        **outcome.arrays,
        "metadata_json_utf8": np.frombuffer(
            canonical_json(metadata), dtype=np.uint8
        ).copy(),
    }
    buffer = io.BytesIO()
    np.savez(buffer, **arrays)
    payload = buffer.getvalue()
    return payload, {
        "status": "EMITTED_SOLVER_PRIVATE_TRANSIENT",
        "file_name": "solver-private-r130-tangent-projected-fixed-pd.npz",
        "file_sha256": hashlib.sha256(payload).hexdigest(),
        "file_bytes": len(payload),
        "candidate_or_corpus_authority": False,
        "arrays": {
            name: {
                "shape": list(value.shape),
                "dtype": str(value.dtype),
                "sha256": _array_sha256(value),
            }
            for name, value in arrays.items()
        },
        "metadata": metadata,
    }


def _flight_projection_row(
    *,
    collocation: int,
    interval: int,
    substep: int,
    modes: NDArray[np.uint8],
    velocity: NDArray[np.float64],
) -> dict[str, Any]:
    energy = None
    return {
        "status": "PASS",
        "collocation": collocation,
        "interval": interval,
        "substep": substep,
        "contact_modes": modes.tolist(),
        "active_point_ordinals": [],
        "original_velocity_sha256": _array_sha256(velocity),
        "projected_velocity_sha256": _array_sha256(velocity),
        "projection": {
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
            "kinetic_energy_before_joules": energy,
            "kinetic_energy_after_joules": energy,
            "kinetic_energy_increase_joules": 0.0,
            "gauge_velocity_effect_maximum_absolute": 0.0,
        },
        "flat_rigid_line_audit": None,
        "invalid_reason": None,
    }


def _active_projection_row(
    *,
    collocation: int,
    interval: int,
    substep: int,
    modes: NDArray[np.uint8],
    active: tuple[int, ...],
    analysis: Any,
    jacobian: NDArray[np.float64],
    original_velocity: NDArray[np.float64],
    projected_velocity: NDArray[np.float64],
    flat_audit: Mapping[str, Any] | None,
    passed: bool,
) -> dict[str, Any]:
    return {
        "status": "PASS" if passed else "FAIL",
        "collocation": collocation,
        "interval": interval,
        "substep": substep,
        "contact_modes": modes.tolist(),
        "active_point_ordinals": list(active),
        "original_velocity_sha256": _array_sha256(original_velocity),
        "projected_velocity_sha256": _array_sha256(projected_velocity),
        "projection": _analysis_metrics(analysis, jacobian, original_velocity),
        "flat_rigid_line_audit": flat_audit,
        "invalid_reason": analysis.invalid_reason,
    }


def _check_resource_budget(
    *,
    budget: Mapping[str, Any],
    started: float,
    maximum_rss: int,
    maximum_threads: int,
) -> tuple[bool, int, int, str | None]:
    rss = max(maximum_rss, _resident_memory_bytes())
    threads = max(maximum_threads, _linux_thread_count())
    if time.monotonic() - started > float(budget["maximum_wall_clock_seconds"]):
        return False, rss, threads, "WALL_CLOCK_BUDGET_EXHAUSTED"
    if rss > int(budget["maximum_resident_memory_bytes"]):
        return False, rss, threads, "RESIDENT_MEMORY_BUDGET_EXHAUSTED"
    if threads > int(budget["thread_count"]):
        return False, rss, threads, "THREAD_BUDGET_EXCEEDED"
    return True, rss, threads, None


def _projected_controller_lineage_passes(
    profile: Mapping[str, Any], audit: Mapping[str, Any]
) -> bool:
    counts = audit.get("activation_counts", {})
    channels = audit.get("activated_dof_ordinals", {})
    maxima = audit.get("maxima", {})
    expected = profile.get("controller_lineage_contract", {})
    return bool(
        expected == CONTROLLER_LINEAGE
        and audit.get("status") == "PASS"
        and audit.get("collocation_count") == COLLOCATION_COUNT
        and counts.get("hard_rom_violation") == expected["hard_rom_violation_count"]
        and channels.get("hard_rom_violation")
        == expected["hard_rom_violation_dof_ordinals"]
        and maxima.get("hard_rom_excess_microradians")
        == expected["maximum_hard_rom_excess_microradians"]
        and counts.get("target_soft_clamp") == expected["target_soft_clamp_count"]
        and channels.get("target_soft_clamp")
        == expected["target_soft_clamp_dof_ordinals"]
        and counts.get("target_slew") == expected["target_slew_count"]
        and channels.get("target_slew") == expected["target_slew_dof_ordinals"]
        and maxima.get("target_lag_microradians")
        == expected["maximum_target_lag_microradians"]
    )


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r129_profile_path: Path,
    projection_module_path: Path,
    r129_tool_path: Path,
    r128_profile_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    r121_profile_path: Path,
    r113_profile_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    collocation_lift_module_path: Path,
    dynamics_kernel_path: Path,
    gauge_execution_module_path: Path,
) -> None:
    source = profile["source"]
    identities = (
        (r129_profile_path, source["r129"]["profile_sha256"]),
        (projection_module_path, source["r129"]["projection_module_sha256"]),
        (r129_tool_path, source["r129"]["tool_sha256"]),
        (r128_profile_path, source["r128"]["profile_sha256"]),
        (r126_profile_path, source["r126"]["profile_sha256"]),
        (gauge_aware_kernel_path, source["r126"]["gauge_aware_kernel_sha256"]),
        (
            r126_conformance_module_path,
            source["r126"]["conformance_module_sha256"],
        ),
        (r126_tool_path, source["r126"]["tool_sha256"]),
        (r121_profile_path, source["r121"]["profile_sha256"]),
        (r113_profile_path, source["r113"]["profile_sha256"]),
        (r120_profile_path, source["r120"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (v9_complete_clip_path, source["v9_complete_clip_sha256"]),
        (collocation_lift_module_path, source["collocation_lift_module_sha256"]),
        (dynamics_kernel_path, source["dynamics_kernel_sha256"]),
        (gauge_execution_module_path, source["gauge_execution_module_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in identities):
        raise ValueError("R130 bound source file identity differs")


def _validate_source_contracts(
    *,
    profile: Mapping[str, Any],
    r129: Mapping[str, Any],
    r128: Mapping[str, Any],
    r126: Mapping[str, Any],
    r121: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    source = profile["source"]
    r129_id = r129.get("identities", {})
    r128_id = r128.get("identities", {})
    r126_id = r126.get("identities", {})
    r121_id = r121.get("identities", {})
    r113_id = r113.get("identities", {})
    r120_id = r120.get("identities", {})
    future = r128.get("future_r130_execution_contract", {})
    expected = profile["expected_inventory"]
    if (
        r129.get("status") != "PASS"
        or r129.get("gate_decision")
        != "PERMIT_R130_SINGLE_BOUNDED_TANGENT_PROJECTED_FIXED_PD_EXECUTION_ONLY"
        or r129.get("repository", {}).get("commit")
        != source["r129"]["repository_commit"]
        or r129.get("repository", {}).get("dirty") is not False
        or r129_id.get("profile_sha256") != source["r129"]["profile_sha256"]
        or r129_id.get("conformance_module_sha256")
        != source["r129"]["projection_module_sha256"]
        or r129_id.get("tool_sha256") != source["r129"]["tool_sha256"]
        or r129.get("real_anchor_audits") != 7
        or r129.get("real_state_projections") != 6
        or r129.get("full_schedule_projections") != 0
        or r129.get("inverse_dynamics_evaluations") != 0
        or r129.get("bounded_acceptance", {}).get("r130_projected_fixed_pd_execution")
        != "AUTHORIZED_ONE_EXECUTION_ONLY_ON_R129_PASS"
        or r128.get("status") != "COMPLETE"
        or r128.get("report_sha256") != source["r128"]["report_sha256"]
        or r128_id.get("profile_sha256") != source["r128"]["profile_sha256"]
        or future.get("collocation_count") != COLLOCATION_COUNT
        or future.get("authorization") != "NOT_AUTHORIZED_UNTIL_EXACT_R129_PASS"
        or future.get("maximum_projection_systems") != expected["projection_systems"]
        or future.get("maximum_projection_factorizations")
        != expected["projection_factorizations"]
        or future.get("maximum_projection_solves") != expected["projection_solves"]
        or future.get("maximum_inverse_dynamics_singular_value_decompositions")
        != expected["inverse_dynamics_singular_value_decompositions"]
        or future.get("maximum_inverse_dynamics_particular_solutions")
        != expected["inverse_dynamics_particular_solutions"]
        or future.get("maximum_force_gauge_interval_classifications")
        != expected["force_gauge_interval_classifications"]
        or future.get("controller_policy")
        != "recompute one complete sequential fixed-PD schedule from projected velocity; never reuse R121 effort bytes"
        or r126.get("status") != "PASS"
        or r126.get("repository", {}).get("commit")
        != source["r126"]["repository_commit"]
        or r126.get("repository", {}).get("dirty") is not False
        or r126_id.get("profile_sha256") != source["r126"]["profile_sha256"]
        or r126_id.get("gauge_aware_kernel_sha256")
        != source["r126"]["gauge_aware_kernel_sha256"]
        or r126_id.get("conformance_module_sha256")
        != source["r126"]["conformance_module_sha256"]
        or r126_id.get("tool_sha256") != source["r126"]["tool_sha256"]
        or r126.get("real_schedule_particular_solutions") != 0
        or r126.get("real_schedule_gauge_interval_classifications") != 0
        or r121.get("status") != "COMPLETE"
        or r121.get("repository", {}).get("commit")
        != source["r121"]["repository_commit"]
        or r121.get("repository", {}).get("dirty") is not False
        or r121_id.get("profile_sha256") != source["r121"]["profile_sha256"]
        or r121.get("fixed_pd_schedule_audit", {}).get("status") != "PASS"
        or r113.get("status") != "PASS"
        or r113.get("repository", {}).get("commit")
        != source["r113"]["repository_commit"]
        or r113.get("repository", {}).get("dirty") is not False
        or r113_id.get("profile_sha256") != source["r113"]["profile_sha256"]
        or r120.get("status") != "PASS"
        or r120.get("repository", {}).get("commit")
        != source["r120"]["repository_commit"]
        or r120.get("repository", {}).get("dirty") is not False
        or r120_id.get("execution_profile_sha256") != source["r120"]["profile_sha256"]
        or r120_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
        or r126_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
    ):
        raise ValueError("R130 source report contract differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R130 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R130 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    expected = profile.get("expected_inventory", {})
    budget = profile.get("execution_budget", {})
    numeric = profile.get("numeric_contract", {})
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleExecution"
        or profile.get("claim")
        != "Stage2PointwiseTangentProjectedFixedPdConeFeasibilityOnly"
        or scope.get("run_id") != "R130"
        or scope.get("motor_interval_count") != MOTOR_INTERVAL_COUNT
        or scope.get("collocation_count") != COLLOCATION_COUNT
        or scope.get("generalized_velocity_width") != GENERALIZED_WIDTH
        or scope.get("generalized_acceleration_width") != GENERALIZED_WIDTH
        or scope.get("maximum_reduced_local_unknown_count") != 35
        or scope.get("maximum_flat_foot_gauge_dimension") != 1
        or scope.get("position_projection") is not False
        or scope.get("integration") is not False
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or expected.get("flight_collocations") != 560
        or expected.get("single_point_collocations") != 324
        or expected.get("flat_foot_collocations") != 2316
        or expected.get("active_point_collocations") != 4956
        or expected.get("projection_systems") != 2640
        or expected.get("projection_factorizations") != 2640
        or expected.get("projection_solves") != 2640
        or expected.get("inverse_dynamics_system_assemblies") != 3200
        or expected.get("inverse_dynamics_singular_value_decompositions") != 3200
        or expected.get("inverse_dynamics_particular_solutions") != 3200
        or expected.get("force_gauge_interval_classifications") != 2316
        or expected.get("maximum_reduced_local_unknown_count") != 35
        or expected.get("maximum_flat_foot_gauge_dimension") != 1
        or budget.get("process_count") != 1
        or budget.get("thread_count") != 1
        or budget.get("maximum_projection_systems") != 2640
        or budget.get("maximum_projection_factorizations") != 2640
        or budget.get("maximum_projection_solves") != 2640
        or budget.get("maximum_inverse_dynamics_system_assemblies") != 3200
        or budget.get("maximum_inverse_dynamics_singular_value_decompositions") != 3200
        or budget.get("maximum_inverse_dynamics_particular_solutions") != 3200
        or budget.get("maximum_force_gauge_interval_classifications") != 2316
        or budget.get("maximum_local_unknown_count") != 35
        or budget.get("maximum_flat_foot_gauge_dimension") != 1
        or budget.get("maximum_wall_clock_seconds") != 7200
        or budget.get("maximum_resident_memory_bytes") != 8 * 1024**3
        or budget.get("random_seed") != 0
        or budget.get("randomized_restart_count") != 0
        or budget.get("resume_or_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
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
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or numeric.get("analytic_gauge_scaled_residual") != 1.0e-10
        or numeric.get("analytic_to_svd_null_projector_spectral_error") != 1.0e-8
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("physical_group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or numeric.get("quadratic_boundary_relative_ambiguity") != 1.0e-12
        or decision.get("valid_feasible")
        != "PERMIT_SEPARATE_REPORT_ONLY_PROJECTED_STATE_KINODYNAMIC_FORMULATION_ONLY"
        or decision.get("valid_infeasible")
        != "STOP_VALID_PROJECTED_FIXED_PD_CONE_INFEASIBILITY_FOR_RESEARCH"
        or decision.get("invalid") != "STOP_INVALID_EVIDENCE_WITHOUT_RESTART"
        or profile.get("controller_lineage_contract") != CONTROLLER_LINEAGE
        or profile.get("result_transition", {}).get("valid_feasible")
        != "R130_CONSUMED_VALID_FEASIBLE_POINTWISE_RESULT"
        or profile.get("result_transition", {}).get("valid_infeasible")
        != "R130_CONSUMED_VALID_INFEASIBLE_POINTWISE_RESULT"
        or profile.get("result_transition", {}).get("invalid")
        != "R130_CONSUMED_INVALID_NO_RETRY"
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
        raise ValueError("R130 execution profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R130 execution requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], actual: Mapping[str, str]
) -> None:
    if (
        dict(actual) != profile["single_thread_environment"]
        or _linux_thread_count() != 1
    ):
        raise ValueError("R130 single-thread execution environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R130 validation results differ")
    return normalized


def _array_sha256(array: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(array).tobytes()).hexdigest()


def _resident_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _linux_thread_count() -> int:
    status = Path("/proc/self/status").read_text(encoding="utf-8")
    for line in status.splitlines():
        if line.startswith("Threads:"):
            return int(line.split(":", 1)[1].strip())
    raise ValueError("R130 cannot read Linux thread count")
