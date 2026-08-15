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

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _contact_points,
    load_r120_cache,
)
from next_lab.fixed_pd_inverse_dynamics_execution import (
    FixedPdSchedule,
    audit_contact_cones,
    collocation_state,
    derive_fixed_pd_schedule,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
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
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import (
    GaugeFeasibility,
    RankAnalysis,
    ReducedLocalSystem,
    analyse_reduced_system,
    build_reduced_local_system,
    classify_flat_gauge_feasibility,
    reduced_column_scale,
    scale_reduced_system,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

EXECUTION_ID = "nextengine.humanoid-gauge-aware-fixed-pd-execution.v1"
CHECK_ID = "TRAIN-4-GAUGE-AWARE-FIXED-PD-EXECUTION"
POINT_FORCE_WIDTH = 3
ZERO_DOWNSTREAM_COUNTERS = (
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


@dataclass(frozen=True)
class CollocationSolution:
    status: str
    feasibility: str | None
    invalid_reason: str | None
    selected_solution: NDArray[np.float64] | None
    particular_solution: NDArray[np.float64] | None
    rank_analysis: RankAnalysis
    gauge_feasibility: GaugeFeasibility | None
    cone_audit: dict[str, Any] | None
    particular_cone_audit: dict[str, Any] | None
    selected_scaled_absolute_residual: float | None
    selected_backward_error: float | None
    selected_dynamics_absolute_residual: float | None
    selected_closure_absolute_residual: float | None


@dataclass(frozen=True)
class ExecutionOutcome:
    status: str
    feasibility: str | None
    invalid_reason: str | None
    singular_value_decompositions: int
    particular_solutions: int
    gauge_interval_classifications: int
    rows: tuple[dict[str, Any], ...]
    aggregate: dict[str, Any]
    arrays: dict[str, NDArray[Any]] | None
    resource_usage: dict[str, Any]
    ordered_reduced_system_sha256: str


def execute_and_build_gauge_aware_fixed_pd_report(
    *,
    profile_path: Path,
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
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> tuple[dict[str, Any], bytes | None]:
    """Consume the sole R127 authority and return its report/cache bytes."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
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
            r120_cache_path,
            v9_complete_clip_path,
            tool_path,
        )
    )
    (
        profile_path,
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
        r120_cache_path,
        v9_complete_clip_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R127 execution input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    r126 = _load_bound_report(r126_report_path, profile["source"]["r126"], "R126")
    r121 = _load_bound_report(r121_report_path, profile["source"]["r121"], "R121")
    r113 = _load_bound_report(r113_report_path, profile["source"]["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, profile["source"]["r120"], "R120")
    _validate_sources(
        profile=profile,
        r126=r126,
        r126_profile_path=r126_profile_path,
        gauge_aware_kernel_path=gauge_aware_kernel_path,
        r126_conformance_module_path=r126_conformance_module_path,
        r126_tool_path=r126_tool_path,
        r121=r121,
        r121_profile_path=r121_profile_path,
        r113=r113,
        r113_profile_path=r113_profile_path,
        r120=r120,
        r120_cache_path=r120_cache_path,
        v9_complete_clip_path=v9_complete_clip_path,
    )
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve())
        != profile["source"]["execution_module_sha256"]
        or sha256(tool_path) != profile["source"]["tool_sha256"]
    ):
        raise ValueError("R127 current execution identity differs")

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R127 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=r113)
    fixed_pd = derive_fixed_pd_schedule(cache=cache, descriptor=descriptor)
    if fixed_pd.audit != r121["fixed_pd_schedule_audit"]:
        raise ValueError("R127 fixed-PD schedule differs from R121")

    outcome = execute_gauge_aware_fixed_pd(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        fixed_pd=fixed_pd,
    )
    cache_bytes, cache_identity = encode_solver_private_cache(
        outcome=outcome,
        profile_sha256=sha256(profile_path),
        r126_report_sha256=r126["report_sha256"],
    )
    valid = outcome.status == "VALID_COMPLETE"
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": "COMPLETE" if valid else "INVALID",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["valid_complete" if valid else "invalid"],
        "scope": profile["scope"],
        "source_gates": {
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
        "fixed_pd_schedule_audit": fixed_pd.audit,
        "solver_result": {
            "status": outcome.status,
            "feasibility": outcome.feasibility,
            "invalid_reason": outcome.invalid_reason,
            "singular_value_decompositions": outcome.singular_value_decompositions,
            "particular_solutions": outcome.particular_solutions,
            "gauge_interval_classifications": outcome.gauge_interval_classifications,
            "ordered_reduced_system_float64_sha256": outcome.ordered_reduced_system_sha256,
            "aggregate": outcome.aggregate,
            "collocations": list(outcome.rows),
        },
        "solver_private_cache": cache_identity,
        "resource_usage": outcome.resource_usage,
        "execution_environment": dict(execution_environment),
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
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
            "r120_cache_sha256": sha256(r120_cache_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "result_transition": profile["result_transition"][
            "valid_complete" if valid else "invalid"
        ],
        "bounded_acceptance": profile["bounded_acceptance"],
        "solver_runs": 1,
        "gauge_aware_fixed_pd_execution_runs": 1,
        "singular_value_decompositions": outcome.singular_value_decompositions,
        "particular_solutions": outcome.particular_solutions,
        "gauge_interval_classifications": outcome.gauge_interval_classifications,
        "local_system_solves": 0,
        **{counter: 0 for counter in ZERO_DOWNSTREAM_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report, cache_bytes


def execute_gauge_aware_fixed_pd(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    fixed_pd: FixedPdSchedule,
) -> ExecutionOutcome:
    """Consume all 3200 R127 collocations once; no retry path exists."""

    budget = profile["execution_budget"]
    numeric = profile["numeric_contract"]
    np.random.seed(int(budget["random_seed"]))
    start = time.monotonic()
    maximum_rss = _resident_memory_bytes()
    maximum_threads = _linux_thread_count()
    system_digest = hashlib.sha256()
    effort = fixed_pd.applied_effort_micronewton_metres / 1_000_000.0
    acceleration = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    point_force = np.zeros(
        (COLLOCATION_COUNT, POINT_COUNT, POINT_FORCE_WIDTH), dtype=np.float64
    )
    interval_lower = np.full(COLLOCATION_COUNT, np.nan, dtype=np.float64)
    interval_upper = np.full(COLLOCATION_COUNT, np.nan, dtype=np.float64)
    witness_alpha = np.full(COLLOCATION_COUNT, np.nan, dtype=np.float64)
    collocation_feasible = np.empty(COLLOCATION_COUNT, dtype=np.uint8)
    interval_index = np.repeat(
        np.arange(MOTOR_INTERVAL_COUNT, dtype=np.int64), SUBSTEPS_PER_INTERVAL
    )
    substep_index = np.tile(
        np.arange(SUBSTEPS_PER_INTERVAL, dtype=np.int64), MOTOR_INTERVAL_COUNT
    )
    rows: list[dict[str, Any]] = []
    decompositions = 0
    particulars = 0
    gauge_classifications = 0
    invalid_reason: str | None = None

    for collocation in range(COLLOCATION_COUNT):
        interval = int(interval_index[collocation])
        substep = int(substep_index[collocation])
        state = collocation_state(cache, interval, substep)
        system = build_reduced_local_system(
            model=model,
            state=state,
            effort_newton_metres=effort[collocation],
            modes=contact_modes[interval],
            points=points,
        )
        system_digest.update(np.ascontiguousarray(system.matrix).tobytes())
        system_digest.update(np.ascontiguousarray(system.right_hand_side).tobytes())
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
        if solution.status != "VALID":
            invalid_reason = solution.invalid_reason
            rows.append(
                _collocation_row(
                    collocation=collocation,
                    interval=interval,
                    substep=substep,
                    system=system,
                    solution=solution,
                )
            )
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
        rows.append(
            _collocation_row(
                collocation=collocation,
                interval=interval,
                substep=substep,
                system=system,
                solution=solution,
            )
        )
        if collocation % 64 == 63:
            elapsed = time.monotonic() - start
            maximum_rss = max(maximum_rss, _resident_memory_bytes())
            maximum_threads = max(maximum_threads, _linux_thread_count())
            if elapsed > float(budget["maximum_wall_clock_seconds"]):
                invalid_reason = "WALL_CLOCK_BUDGET_EXHAUSTED"
                break
            if maximum_rss > int(budget["maximum_resident_memory_bytes"]):
                invalid_reason = "RESIDENT_MEMORY_BUDGET_EXHAUSTED"
                break
            if maximum_threads > int(budget["thread_count"]):
                invalid_reason = "THREAD_BUDGET_EXCEEDED"
                break

    elapsed = time.monotonic() - start
    maximum_rss = max(maximum_rss, _resident_memory_bytes())
    maximum_threads = max(maximum_threads, _linux_thread_count())
    expected = profile["expected_inventory"]
    if decompositions > int(budget["maximum_singular_value_decompositions"]):
        invalid_reason = "SVD_BUDGET_EXCEEDED"
    if particulars > int(budget["maximum_particular_solutions"]):
        invalid_reason = "PARTICULAR_SOLUTION_BUDGET_EXCEEDED"
    if gauge_classifications > int(budget["maximum_gauge_interval_classifications"]):
        invalid_reason = "GAUGE_CLASSIFICATION_BUDGET_EXCEEDED"
    if len(rows) == COLLOCATION_COUNT and (
        decompositions != int(expected["singular_value_decompositions"])
        or particulars != int(expected["particular_solutions"])
        or gauge_classifications != int(expected["gauge_interval_classifications"])
    ):
        invalid_reason = invalid_reason or "FULL_SCHEDULE_COUNTER_CLOSURE_DIFFERS"
    if len(rows) != COLLOCATION_COUNT and invalid_reason is None:
        invalid_reason = "COLLOCATION_CLOSURE_DIFFERS"

    valid = invalid_reason is None and len(rows) == COLLOCATION_COUNT
    feasibility = None
    arrays: dict[str, NDArray[Any]] | None = None
    if valid:
        feasibility = (
            "FEASIBLE" if bool(np.all(collocation_feasible == 1)) else "INFEASIBLE"
        )
        arrays = {
            "generalized_acceleration": acceleration,
            "applied_effort_newton_metres": effort,
            "point_force_normal_right_forward_newtons": point_force,
            "gauge_interval_lower": interval_lower,
            "gauge_interval_upper": interval_upper,
            "gauge_witness_alpha": witness_alpha,
            "collocation_feasible": collocation_feasible,
            "interval_index": interval_index,
            "substep_index": substep_index,
        }
    aggregate = aggregate_execution_rows(rows, arrays)
    return ExecutionOutcome(
        status="VALID_COMPLETE" if valid else "INVALID",
        feasibility=feasibility,
        invalid_reason=invalid_reason,
        singular_value_decompositions=decompositions,
        particular_solutions=particulars,
        gauge_interval_classifications=gauge_classifications,
        rows=tuple(rows),
        aggregate=aggregate,
        arrays=arrays,
        resource_usage={
            "status": "PASS" if valid else "FAIL",
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
        ordered_reduced_system_sha256=system_digest.hexdigest(),
    )


def solve_gauge_aware_collocation(
    system: ReducedLocalSystem,
    *,
    column_scale: NDArray[np.float64],
    numeric_contract: Mapping[str, Any],
) -> CollocationSolution:
    analysis = analyse_reduced_system(
        system,
        column_scale=column_scale,
        numeric_contract=numeric_contract,
        compute_particular=True,
    )
    if analysis.status != "VALID" or analysis.particular_solution is None:
        return _invalid_solution(
            system,
            analysis,
            analysis.invalid_reason or "PARTICULAR_SOLUTION_ABSENT",
        )
    particular = analysis.particular_solution
    active_count = len(system.active_point_ordinals)
    particular_forces = particular[GENERALIZED_WIDTH:].reshape(
        active_count, POINT_FORCE_WIDTH
    )
    active_mask = np.zeros(POINT_COUNT, dtype=np.bool_)
    active_mask[list(system.active_point_ordinals)] = True
    particular_four = np.zeros((POINT_COUNT, POINT_FORCE_WIDTH), dtype=np.float64)
    for local, point in enumerate(system.active_point_ordinals):
        particular_four[point] = particular_forces[local]
    particular_cone = audit_contact_cones(
        particular_four,
        active=active_mask,
        tolerance_newtons=float(numeric_contract["cone_absolute_newtons"]),
    )

    selected = particular.copy()
    gauge_result: GaugeFeasibility | None = None
    if active_count == 0:
        feasibility = "FEASIBLE"
    elif active_count == 1:
        feasibility = (
            "FEASIBLE" if particular_cone["friction_cone_feasible"] else "INFEASIBLE"
        )
    elif active_count == 2 and system.analytic_gauge_matrix.shape[1] == 1:
        direction = system.analytic_gauge_matrix[
            GENERALIZED_WIDTH : GENERALIZED_WIDTH + POINT_FORCE_WIDTH, 0
        ]
        gauge_result = classify_flat_gauge_feasibility(
            particular_forces,
            direction,
            boundary_relative_ambiguity=float(
                numeric_contract["quadratic_boundary_relative_ambiguity"]
            ),
            cone_absolute_newtons=float(numeric_contract["cone_absolute_newtons"]),
        )
        if gauge_result.status != "VALID" or gauge_result.feasible is None:
            return _invalid_solution(
                system,
                analysis,
                gauge_result.invalid_reason or "GAUGE_CLASSIFICATION_INVALID",
                particular_cone=particular_cone,
                gauge_result=gauge_result,
            )
        feasibility = "FEASIBLE" if gauge_result.feasible else "INFEASIBLE"
        if gauge_result.feasible:
            assert gauge_result.witness_alpha is not None
            selected += system.analytic_gauge_matrix[:, 0] * gauge_result.witness_alpha
    else:
        return _invalid_solution(
            system,
            analysis,
            "ACTIVE_POINT_OR_GAUGE_DIMENSION_DIFFERS",
            particular_cone=particular_cone,
        )

    equality = audit_selected_equality(
        system,
        selected_solution=selected,
        column_scale=column_scale,
    )
    invalid = None
    if equality["scaled_absolute_residual"] > float(
        numeric_contract["scaled_absolute_residual"]
    ):
        invalid = "SELECTED_SCALED_EQUALITY_RESIDUAL_EXCEEDED"
    elif equality["backward_error"] > float(numeric_contract["backward_error"]):
        invalid = "SELECTED_BACKWARD_ERROR_EXCEEDED"
    elif max(
        equality["dynamics_absolute_residual"],
        equality["closure_absolute_residual"],
    ) > float(numeric_contract["physical_group_absolute_residual"]):
        invalid = "SELECTED_PHYSICAL_GROUP_RESIDUAL_EXCEEDED"

    selected_forces = selected[GENERALIZED_WIDTH:].reshape(
        active_count, POINT_FORCE_WIDTH
    )
    selected_four = np.zeros((POINT_COUNT, POINT_FORCE_WIDTH), dtype=np.float64)
    for local, point in enumerate(system.active_point_ordinals):
        selected_four[point] = selected_forces[local]
    cone = audit_contact_cones(
        selected_four,
        active=active_mask,
        tolerance_newtons=float(numeric_contract["cone_absolute_newtons"]),
    )
    if feasibility == "FEASIBLE" and not cone["friction_cone_feasible"]:
        invalid = invalid or "FEASIBLE_WITNESS_DIRECT_CONE_RECHECK_FAILED"
    return CollocationSolution(
        status="VALID" if invalid is None else "INVALID",
        feasibility=feasibility if invalid is None else None,
        invalid_reason=invalid,
        selected_solution=selected,
        particular_solution=particular,
        rank_analysis=analysis,
        gauge_feasibility=gauge_result,
        cone_audit=cone,
        particular_cone_audit=particular_cone,
        selected_scaled_absolute_residual=equality["scaled_absolute_residual"],
        selected_backward_error=equality["backward_error"],
        selected_dynamics_absolute_residual=equality["dynamics_absolute_residual"],
        selected_closure_absolute_residual=equality["closure_absolute_residual"],
    )


def audit_selected_equality(
    system: ReducedLocalSystem,
    *,
    selected_solution: NDArray[np.float64],
    column_scale: NDArray[np.float64],
) -> dict[str, float]:
    if selected_solution.shape != system.right_hand_side.shape:
        raise ValueError("R127 selected solution shape differs")
    scaled_matrix, scaled_right, _ = scale_reduced_system(system, column_scale)
    scaled_solution = selected_solution / column_scale
    scaled_residual = float(
        np.max(np.abs(scaled_matrix @ scaled_solution - scaled_right))
    )
    residual = system.matrix @ selected_solution - system.right_hand_side
    denominator = np.maximum(
        np.abs(system.matrix) @ np.abs(selected_solution)
        + np.abs(system.right_hand_side),
        1.0,
    )
    return {
        "scaled_absolute_residual": scaled_residual,
        "backward_error": float(np.max(np.abs(residual) / denominator)),
        "dynamics_absolute_residual": float(
            np.max(np.abs(residual[:GENERALIZED_WIDTH]))
        ),
        "closure_absolute_residual": (
            float(np.max(np.abs(residual[GENERALIZED_WIDTH:])))
            if residual.size > GENERALIZED_WIDTH
            else 0.0
        ),
    }


def aggregate_execution_rows(
    rows: Sequence[Mapping[str, Any]],
    arrays: Mapping[str, NDArray[Any]] | None,
) -> dict[str, Any]:
    valid = [row for row in rows if row["status"] == "VALID"]
    feasible = [row for row in valid if row["feasibility"] == "FEASIBLE"]
    infeasible = [row for row in valid if row["feasibility"] == "INFEASIBLE"]
    flat = [row for row in valid if row["gauge_interval_classified"]]
    result: dict[str, Any] = {
        "collocation_rows_recorded": len(rows),
        "numerically_valid_collocations": len(valid),
        "feasible_collocations": len(feasible),
        "infeasible_collocations": len(infeasible),
        "first_infeasible_collocation": (
            int(infeasible[0]["collocation"]) if infeasible else None
        ),
        "flat_foot_collocations": len(flat),
        "nonempty_flat_gauge_intervals": sum(
            row["feasibility"] == "FEASIBLE" for row in flat
        ),
        "empty_flat_gauge_intervals": sum(
            row["feasibility"] == "INFEASIBLE" for row in flat
        ),
        "particular_force_cone_infeasible_collocations": sum(
            row.get("particular_force_cone_feasible") is False for row in valid
        ),
    }
    for field in (
        "analytic_gauge_scaled_residual",
        "analytic_to_svd_null_projector_spectral_error",
        "selected_scaled_absolute_residual",
        "selected_backward_error",
        "selected_dynamics_absolute_residual",
        "selected_closure_absolute_residual",
        "maximum_absolute_acceleration",
        "maximum_absolute_point_force_newtons",
        "maximum_absolute_gauge_witness_alpha",
    ):
        values = [float(row[field]) for row in valid if row.get(field) is not None]
        result[f"maximum_{field}"] = max(values) if values else None
    normal = [
        (float(row["minimum_active_normal_margin_newtons"]), int(row["collocation"]))
        for row in feasible
        if row.get("minimum_active_normal_margin_newtons") is not None
    ]
    friction = [
        (
            float(row["minimum_active_friction_margin_newtons"]),
            int(row["collocation"]),
        )
        for row in feasible
        if row.get("minimum_active_friction_margin_newtons") is not None
    ]
    result["minimum_feasible_normal_margin"] = (
        {"newtons": min(normal)[0], "collocation": min(normal)[1]} if normal else None
    )
    result["minimum_feasible_friction_margin"] = (
        {"newtons": min(friction)[0], "collocation": min(friction)[1]}
        if friction
        else None
    )
    if arrays is not None:
        result["solution_array_sha256"] = {
            name: _array_sha256(value) for name, value in arrays.items()
        }
    return result


def encode_solver_private_cache(
    *,
    outcome: ExecutionOutcome,
    profile_sha256: str,
    r126_report_sha256: str,
) -> tuple[bytes | None, dict[str, Any]]:
    if outcome.arrays is None:
        return None, {
            "status": "NOT_EMITTED_INVALID_EXECUTION",
            "candidate_or_corpus_authority": False,
        }
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r127-gauge-aware-fixed-pd-solver-private.v1",
        "candidate_or_corpus_authority": False,
        "collocation_count": COLLOCATION_COUNT,
        "feasibility": outcome.feasibility,
        "profile_sha256": profile_sha256,
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
        "file_name": "solver-private-r127-gauge-aware-fixed-pd.npz",
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


def _collocation_row(
    *,
    collocation: int,
    interval: int,
    substep: int,
    system: ReducedLocalSystem,
    solution: CollocationSolution,
) -> dict[str, Any]:
    rank = solution.rank_analysis
    gauge = solution.gauge_feasibility
    cone = solution.cone_audit or {}
    particular_cone = solution.particular_cone_audit or {}
    selected = solution.selected_solution
    return {
        "status": solution.status,
        "feasibility": solution.feasibility,
        "collocation": collocation,
        "interval": interval,
        "substep": substep,
        "contact_modes": system.modes.tolist(),
        "active_point_ordinals": list(system.active_point_ordinals),
        "reduced_local_unknown_count": int(system.matrix.shape[1]),
        "rank": rank.rank,
        "nullity": rank.nullity,
        "expected_nullity": rank.expected_nullity,
        "smallest_retained_relative_singular_value": rank.smallest_retained_relative_singular_value,
        "largest_null_relative_singular_value": rank.largest_null_relative_singular_value,
        "analytic_gauge_scaled_residual": rank.analytic_gauge_scaled_residual,
        "analytic_to_svd_null_projector_spectral_error": rank.analytic_to_svd_projector_spectral_error,
        "particular_scaled_absolute_residual": rank.scaled_absolute_residual,
        "particular_backward_error": rank.backward_error,
        "selected_scaled_absolute_residual": solution.selected_scaled_absolute_residual,
        "selected_backward_error": solution.selected_backward_error,
        "selected_dynamics_absolute_residual": solution.selected_dynamics_absolute_residual,
        "selected_closure_absolute_residual": solution.selected_closure_absolute_residual,
        "particular_force_cone_feasible": particular_cone.get("friction_cone_feasible"),
        "friction_cone_feasible": cone.get("friction_cone_feasible"),
        "minimum_active_normal_margin_newtons": cone.get(
            "minimum_active_normal_margin_newtons"
        ),
        "minimum_active_friction_margin_newtons": cone.get(
            "minimum_active_friction_margin_newtons"
        ),
        "gauge_interval_classified": gauge is not None,
        "gauge_interval_lower": None if gauge is None else gauge.lower,
        "gauge_interval_upper": None if gauge is None else gauge.upper,
        "gauge_witness_alpha": None if gauge is None else gauge.witness_alpha,
        "maximum_absolute_gauge_witness_alpha": (
            None
            if gauge is None or gauge.witness_alpha is None
            else abs(gauge.witness_alpha)
        ),
        "maximum_absolute_acceleration": (
            None
            if selected is None
            else float(np.max(np.abs(selected[:GENERALIZED_WIDTH])))
        ),
        "maximum_absolute_point_force_newtons": (
            None
            if selected is None or selected.size == GENERALIZED_WIDTH
            else float(np.max(np.abs(selected[GENERALIZED_WIDTH:])))
        ),
        "invalid_reason": solution.invalid_reason,
    }


def _invalid_solution(
    system: ReducedLocalSystem,
    analysis: RankAnalysis,
    reason: str,
    *,
    particular_cone: dict[str, Any] | None = None,
    gauge_result: GaugeFeasibility | None = None,
) -> CollocationSolution:
    return CollocationSolution(
        status="INVALID",
        feasibility=None,
        invalid_reason=reason,
        selected_solution=None,
        particular_solution=analysis.particular_solution,
        rank_analysis=analysis,
        gauge_feasibility=gauge_result,
        cone_audit=None,
        particular_cone_audit=particular_cone,
        selected_scaled_absolute_residual=None,
        selected_backward_error=None,
        selected_dynamics_absolute_residual=None,
        selected_closure_absolute_residual=None,
    )


def _validate_sources(
    *,
    profile: Mapping[str, Any],
    r126: Mapping[str, Any],
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    r121: Mapping[str, Any],
    r121_profile_path: Path,
    r113: Mapping[str, Any],
    r113_profile_path: Path,
    r120: Mapping[str, Any],
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
) -> None:
    source = profile["source"]
    identities = (
        (r126_profile_path, source["r126"]["profile_sha256"]),
        (gauge_aware_kernel_path, source["r126"]["gauge_aware_kernel_sha256"]),
        (
            r126_conformance_module_path,
            source["r126"]["conformance_module_sha256"],
        ),
        (r126_tool_path, source["r126"]["tool_sha256"]),
        (r121_profile_path, source["r121"]["profile_sha256"]),
        (r113_profile_path, source["r113"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (v9_complete_clip_path, source["v9_complete_clip_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in identities):
        raise ValueError("R127 bound source file identity differs")
    if (
        r126.get("status") != "PASS"
        or r126.get("gate_decision")
        != "PERMIT_R127_SINGLE_BOUNDED_GAUGE_AWARE_FIXED_PD_EXECUTION_ONLY"
        or r126.get("repository", {}).get("commit")
        != source["r126"]["repository_commit"]
        or r126.get("repository", {}).get("dirty") is not False
        or r126.get("frozen_anchor_singular_value_decompositions") != 7
        or r126.get("real_schedule_particular_solutions") != 0
        or r126.get("real_schedule_gauge_interval_classifications") != 0
        or r126.get("r127_execution_runs") != 0
        or r121.get("status") != "COMPLETE"
        or r121.get("report_sha256")
        != r126.get("source_gates", {}).get("r121_report_sha256")
        or r113.get("status") != "PASS"
        or r113.get("report_sha256")
        != r126.get("source_gates", {}).get("r113_report_sha256")
        or r120.get("status") != "PASS"
        or r120.get("report_sha256")
        != r126.get("source_gates", {}).get("r120_report_sha256")
    ):
        raise ValueError("R127 bound source report contract differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R127 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R127 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    budget = profile.get("execution_budget", {})
    numeric = profile.get("numeric_contract", {})
    expected = profile.get("expected_inventory", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleExecution"
        or scope.get("run_id") != "R127"
        or scope.get("collocation_count") != COLLOCATION_COUNT
        or scope.get("maximum_reduced_local_unknown_count") != 35
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or expected.get("singular_value_decompositions") != COLLOCATION_COUNT
        or expected.get("particular_solutions") != COLLOCATION_COUNT
        or expected.get("gauge_interval_classifications") != 2316
        or budget.get("process_count") != 1
        or budget.get("thread_count") != 1
        or budget.get("maximum_singular_value_decompositions") != COLLOCATION_COUNT
        or budget.get("maximum_particular_solutions") != COLLOCATION_COUNT
        or budget.get("maximum_gauge_interval_classifications") != 2316
        or budget.get("maximum_local_unknown_count") != 35
        or budget.get("maximum_wall_clock_seconds") != 7200
        or budget.get("maximum_resident_memory_bytes") != 8 * 1024**3
        or budget.get("random_seed") != 0
        or budget.get("randomized_restart_count") != 0
        or budget.get("resume_or_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or numeric.get("analytic_gauge_scaled_residual") != 1.0e-10
        or numeric.get("analytic_to_svd_null_projector_spectral_error") != 1.0e-8
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("physical_group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or numeric.get("quadratic_boundary_relative_ambiguity") != 1.0e-12
        or decision.get("valid_complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_FULL_KINODYNAMIC_FORMULATION_ONLY"
        or decision.get("invalid") != "STOP_INVALID_EVIDENCE_WITHOUT_RESTART"
        or bounded.get("r127_retry") != "NOT_AUTHORIZED"
        or bounded.get("full_kinodynamic_formulation")
        != "AUTHORIZED_REPORT_ONLY_ON_VALID_R127_COMPLETION"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_inverse_dynamics_execution",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
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
        raise ValueError("R127 execution profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R127 execution requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], actual: Mapping[str, str]
) -> None:
    if (
        dict(actual) != profile["single_thread_environment"]
        or _linux_thread_count() != 1
    ):
        raise ValueError("R127 single-thread execution environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R127 validation results differ")
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
    raise ValueError("R127 cannot read Linux thread count")
