from __future__ import annotations

import hashlib
import io
import json
import math
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
    _load_bound_report,
    _load_r120,
    configuration_at,
    load_r120_cache,
    rotation_log,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    COLLOCATION_COUNT,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
    LOCAL_UNKNOWN_COUNT,
    MOTOR_INTERVAL_COUNT,
    POINT_COUNT,
    SUBSTEPS_PER_INTERVAL,
    affine_sample,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    Configuration,
    SpatialModel,
    build_spatial_model,
    generalized_contact_force,
    inverse_dynamics,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    propagate_motion,
    rotation_exp,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

EXECUTION_ID = "nextengine.humanoid-fixed-pd-inverse-dynamics-execution.v1"
CHECK_ID = "TRAIN-4-FIXED-PD-INVERSE-DYNAMICS-EXECUTION"
POINT_FORCE_WIDTH = 3
FRICTION_Q16 = 52_429
FRICTION = FRICTION_Q16 / 65_536.0
ZERO_DOWNSTREAM_COUNTERS = (
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


@dataclass(frozen=True)
class FixedPdSchedule:
    applied_target_microradians: NDArray[np.float64]
    applied_effort_micronewton_metres: NDArray[np.float64]
    audit: dict[str, Any]


@dataclass(frozen=True)
class CollocationState:
    configuration: Configuration
    velocity: NDArray[np.float64]
    warm_acceleration: NDArray[np.float64]


@dataclass(frozen=True)
class LocalSystem:
    matrix: NDArray[np.float64]
    right_hand_side: NDArray[np.float64]
    active_points: NDArray[np.bool_]
    modes: NDArray[np.uint8]


@dataclass(frozen=True)
class LocalSolution:
    status: str
    solution: NDArray[np.float64] | None
    solve_attempted: bool
    singular_value_condition_number: float
    scaled_absolute_residual: float | None
    backward_error: float | None
    dynamics_absolute_residual: float | None
    effort_absolute_residual: float | None
    point_absolute_residual: float | None
    invalid_reason: str | None


@dataclass(frozen=True)
class ExecutionOutcome:
    status: str
    feasibility: str | None
    invalid_reason: str | None
    local_system_solves: int
    singular_value_decompositions: int
    collocation_rows: tuple[dict[str, Any], ...]
    aggregate: dict[str, Any]
    arrays: dict[str, NDArray[Any]] | None
    resource_usage: dict[str, Any]
    local_system_sha256: str


def execute_and_build_fixed_pd_inverse_dynamics_report(
    *,
    profile_path: Path,
    r122_report_path: Path,
    r122_profile_path: Path,
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
    """Validate the exact lineage, consume R123 once, and build its report."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r122_report_path,
            r122_profile_path,
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
        r122_report_path,
        r122_profile_path,
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
        raise FileNotFoundError("R123 execution input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    r122 = _load_bound_report(
        r122_report_path, r122_profile_path, profile["source"]["r122"]
    )
    r121 = _load_bound_report(
        r121_report_path, r121_profile_path, profile["source"]["r121"]
    )
    r113 = _load_bound_report(
        r113_report_path, r113_profile_path, profile["source"]["r113"]
    )
    r120 = _load_r120(r120_report_path, profile["source"]["r120"])
    _validate_source_reports(
        profile=profile,
        r122=r122,
        r121=r121,
        r113=r113,
        r120=r120,
    )
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["current_descriptor_file_sha256"]
    ):
        raise ValueError("R123 descriptor identity differs")
    if sha256(Path(__file__).resolve()) != profile["source"]["execution_module_sha256"]:
        raise ValueError("R123 execution module identity differs")
    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    model = build_spatial_model(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R123 V9 contact modes differ")
    points = _contact_points(descriptor=descriptor, r113=r113)
    fixed_pd = derive_fixed_pd_schedule(cache=cache, descriptor=descriptor)
    if fixed_pd.audit != r121["fixed_pd_schedule_audit"]:
        raise ValueError("R123 fixed-PD schedule differs from R121")

    outcome = execute_fixed_pd_inverse_dynamics(
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
        r122_report_sha256=r122["report_sha256"],
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
            "r122_status": r122["status"],
            "r122_report_sha256": r122["report_sha256"],
            "r121_status": r121["status"],
            "r121_report_sha256": r121["report_sha256"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "system_contract": profile["system_contract"],
        "numeric_contract": profile["numeric_contract"],
        "fixed_pd_schedule_audit": fixed_pd.audit,
        "solver_result": {
            "status": outcome.status,
            "feasibility": outcome.feasibility,
            "invalid_reason": outcome.invalid_reason,
            "local_system_solves": outcome.local_system_solves,
            "singular_value_decompositions": outcome.singular_value_decompositions,
            "aggregate": outcome.aggregate,
            "collocations": list(outcome.collocation_rows),
            "ordered_local_system_float64_sha256": outcome.local_system_sha256,
        },
        "solver_private_cache": cache_identity,
        "resource_usage": outcome.resource_usage,
        "execution_environment": dict(execution_environment),
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r122_report_file_sha256": sha256(r122_report_path),
            "r122_profile_sha256": sha256(r122_profile_path),
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
            "dynamics_kernel_sha256": sha256(
                Path(build_spatial_model.__code__.co_filename).resolve()
            ),
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "result_transition": profile["result_transition"][
            "valid_complete" if valid else "invalid"
        ],
        "bounded_acceptance": profile["bounded_acceptance"],
        "solver_runs": 1,
        "inverse_dynamics_execution_runs": 1,
        "local_system_solves": outcome.local_system_solves,
        **{counter: 0 for counter in ZERO_DOWNSTREAM_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report, cache_bytes


def execute_fixed_pd_inverse_dynamics(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    fixed_pd: FixedPdSchedule,
) -> ExecutionOutcome:
    """Consume the single bounded R123 loop. No retry path exists here."""

    budget = profile["execution_budget"]
    numeric = profile["numeric_contract"]
    np.random.seed(int(budget["random_seed"]))
    start = time.monotonic()
    maximum_rss = _resident_memory_bytes()
    maximum_threads = _linux_thread_count()
    system_digest = hashlib.sha256()
    acceleration = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    effort = fixed_pd.applied_effort_micronewton_metres / 1_000_000.0
    point_force = np.empty(
        (COLLOCATION_COUNT, POINT_COUNT, POINT_FORCE_WIDTH), dtype=np.float64
    )
    interval_index = np.repeat(
        np.arange(MOTOR_INTERVAL_COUNT, dtype=np.int64), SUBSTEPS_PER_INTERVAL
    )
    substep_index = np.tile(
        np.arange(SUBSTEPS_PER_INTERVAL, dtype=np.int64), MOTOR_INTERVAL_COUNT
    )
    column_scale = execution_column_scale(
        model=model, cache=cache, descriptor_effort=effort
    )
    rows: list[dict[str, Any]] = []
    solves = 0
    decompositions = 0
    invalid_reason: str | None = None

    for collocation in range(COLLOCATION_COUNT):
        interval = int(interval_index[collocation])
        substep = int(substep_index[collocation])
        state = collocation_state(cache, interval, substep)
        modes = contact_modes[interval]
        local = build_local_system(
            model=model,
            state=state,
            effort_newton_metres=effort[collocation],
            modes=modes,
            points=points,
        )
        system_digest.update(np.ascontiguousarray(local.matrix).tobytes())
        system_digest.update(np.ascontiguousarray(local.right_hand_side).tobytes())
        solution = solve_local_system(
            local,
            column_scale=column_scale,
            numeric_contract=numeric,
        )
        decompositions += 1
        if solution.solve_attempted:
            solves += 1
        if solution.status != "VALID":
            invalid_reason = solution.invalid_reason
            rows.append(
                _invalid_collocation_row(
                    collocation=collocation,
                    interval=interval,
                    substep=substep,
                    modes=modes,
                    active=local.active_points,
                    solution=solution,
                )
            )
            break
        assert solution.solution is not None
        vector = solution.solution
        acceleration[collocation] = vector[:GENERALIZED_WIDTH]
        solved_effort = vector[GENERALIZED_WIDTH : GENERALIZED_WIDTH + ACTUATOR_COUNT]
        forces = vector[GENERALIZED_WIDTH + ACTUATOR_COUNT :].reshape(
            POINT_COUNT, POINT_FORCE_WIDTH
        )
        point_force[collocation] = forces
        cone = audit_contact_cones(
            forces,
            active=local.active_points,
            tolerance_newtons=float(numeric["cone_absolute_newtons"]),
        )
        effort_error = float(np.max(np.abs(solved_effort - effort[collocation])))
        row = {
            "collocation": collocation,
            "interval": interval,
            "substep": substep,
            "contact_modes": modes.tolist(),
            "active_point_ordinals": np.flatnonzero(local.active_points).tolist(),
            "singular_value_condition_number": (
                solution.singular_value_condition_number
            ),
            "scaled_absolute_residual": solution.scaled_absolute_residual,
            "backward_error": solution.backward_error,
            "dynamics_absolute_residual": solution.dynamics_absolute_residual,
            "effort_absolute_residual": solution.effort_absolute_residual,
            "point_absolute_residual": solution.point_absolute_residual,
            "maximum_fixed_effort_identity_error_newton_metres": effort_error,
            "maximum_absolute_acceleration": float(
                np.max(np.abs(acceleration[collocation]))
            ),
            "maximum_absolute_warm_acceleration_delta": float(
                np.max(np.abs(acceleration[collocation] - state.warm_acceleration))
            ),
            "maximum_absolute_point_force_newtons": float(np.max(np.abs(forces))),
            **cone,
        }
        rows.append(row)
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
    if solves > int(budget["maximum_local_system_solves"]):
        invalid_reason = "LOCAL_SYSTEM_SOLVE_BUDGET_EXCEEDED"
    if len(rows) == COLLOCATION_COUNT and solves != COLLOCATION_COUNT:
        invalid_reason = invalid_reason or "LOCAL_SYSTEM_SOLVE_COUNT_DIFFERS"
    if len(rows) != COLLOCATION_COUNT and invalid_reason is None:
        invalid_reason = "COLLOCATION_CLOSURE_DIFFERS"

    valid = invalid_reason is None and len(rows) == COLLOCATION_COUNT
    feasibility = None
    arrays: dict[str, NDArray[Any]] | None = None
    if valid:
        feasible = all(bool(row["friction_cone_feasible"]) for row in rows)
        feasibility = "FEASIBLE" if feasible else "INFEASIBLE"
        arrays = {
            "generalized_acceleration": acceleration,
            "applied_effort_newton_metres": effort,
            "point_force_normal_right_forward_newtons": point_force,
            "interval_index": interval_index,
            "substep_index": substep_index,
        }
    aggregate = aggregate_execution_rows(
        rows=rows,
        warm_acceleration=cache["acceleration"],
        solved_acceleration=acceleration if valid else None,
        solved_point_force=point_force if valid else None,
    )
    return ExecutionOutcome(
        status="VALID_COMPLETE" if valid else "INVALID",
        feasibility=feasibility,
        invalid_reason=invalid_reason,
        local_system_solves=solves,
        singular_value_decompositions=decompositions,
        collocation_rows=tuple(rows),
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
        local_system_sha256=system_digest.hexdigest(),
    )


def build_local_system(
    *,
    model: SpatialModel,
    state: CollocationState,
    effort_newton_metres: NDArray[np.float64],
    modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
) -> LocalSystem:
    matrix, kinematics = mass_matrix(model, state.configuration)
    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    bias = inverse_dynamics(model, state.configuration, state.velocity, zero)
    motion = propagate_motion(model, kinematics, state.velocity, zero)
    jacobians = []
    jdot_v = []
    active = np.zeros(POINT_COUNT, dtype=np.bool_)
    for ordinal, point in enumerate(points):
        body_slot = int(point["body_slot"])
        local = np.asarray(point["local_translation_metres"], dtype=np.float64)
        jacobians.append(point_jacobian(model, kinematics, body_slot, local))
        jdot_v.append(point_acceleration(kinematics, motion, body_slot, local))
        active[ordinal] = point_active(modes, ordinal)
    return assemble_local_system(
        mass=matrix,
        bias=bias,
        effort_newton_metres=effort_newton_metres,
        contact_jacobians=np.stack(jacobians),
        contact_jdot_v=np.stack(jdot_v),
        active_points=active,
        modes=modes,
    )


def assemble_local_system(
    *,
    mass: NDArray[np.float64],
    bias: NDArray[np.float64],
    effort_newton_metres: NDArray[np.float64],
    contact_jacobians: NDArray[np.float64],
    contact_jdot_v: NDArray[np.float64],
    active_points: NDArray[np.bool_],
    modes: NDArray[np.uint8],
) -> LocalSystem:
    if (
        mass.shape != (GENERALIZED_WIDTH, GENERALIZED_WIDTH)
        or bias.shape != (GENERALIZED_WIDTH,)
        or effort_newton_metres.shape != (ACTUATOR_COUNT,)
        or contact_jacobians.shape
        != (POINT_COUNT, POINT_FORCE_WIDTH, GENERALIZED_WIDTH)
        or contact_jdot_v.shape != (POINT_COUNT, POINT_FORCE_WIDTH)
        or active_points.shape != (POINT_COUNT,)
        or modes.shape != (2,)
        or any(
            not np.all(np.isfinite(value))
            for value in (
                mass,
                bias,
                effort_newton_metres,
                contact_jacobians,
                contact_jdot_v,
            )
        )
    ):
        raise ValueError("R123 local-system input differs")
    matrix = np.zeros((LOCAL_UNKNOWN_COUNT, LOCAL_UNKNOWN_COUNT), dtype=np.float64)
    right = np.zeros(LOCAL_UNKNOWN_COUNT, dtype=np.float64)
    matrix[:GENERALIZED_WIDTH, :GENERALIZED_WIDTH] = mass
    for dof in range(ACTUATOR_COUNT):
        matrix[6 + dof, GENERALIZED_WIDTH + dof] = -1.0
    force_offset = GENERALIZED_WIDTH + ACTUATOR_COUNT
    for point in range(POINT_COUNT):
        force_slice = slice(
            force_offset + point * POINT_FORCE_WIDTH,
            force_offset + (point + 1) * POINT_FORCE_WIDTH,
        )
        columns = np.column_stack(
            [
                generalized_contact_force(
                    contact_jacobians[point], np.eye(POINT_FORCE_WIDTH)[axis]
                )
                for axis in range(POINT_FORCE_WIDTH)
            ]
        )
        matrix[:GENERALIZED_WIDTH, force_slice] = -columns
    right[:GENERALIZED_WIDTH] = -bias

    effort_row = GENERALIZED_WIDTH
    effort_column = GENERALIZED_WIDTH
    matrix[
        effort_row : effort_row + ACTUATOR_COUNT,
        effort_column : effort_column + ACTUATOR_COUNT,
    ] = np.eye(ACTUATOR_COUNT, dtype=np.float64)
    right[effort_row : effort_row + ACTUATOR_COUNT] = effort_newton_metres

    point_row = GENERALIZED_WIDTH + ACTUATOR_COUNT
    for point in range(POINT_COUNT):
        rows = slice(
            point_row + point * POINT_FORCE_WIDTH,
            point_row + (point + 1) * POINT_FORCE_WIDTH,
        )
        if active_points[point]:
            matrix[rows, :GENERALIZED_WIDTH] = contact_jacobians[point]
            right[rows] = -contact_jdot_v[point]
        else:
            columns = slice(
                force_offset + point * POINT_FORCE_WIDTH,
                force_offset + (point + 1) * POINT_FORCE_WIDTH,
            )
            matrix[rows, columns] = np.eye(POINT_FORCE_WIDTH, dtype=np.float64)
    return LocalSystem(matrix, right, active_points.copy(), modes.copy())


def solve_local_system(
    system: LocalSystem,
    *,
    column_scale: NDArray[np.float64],
    numeric_contract: Mapping[str, Any],
) -> LocalSolution:
    if (
        column_scale.shape != (LOCAL_UNKNOWN_COUNT,)
        or not np.all(np.isfinite(column_scale))
        or np.any(column_scale <= 0.0)
    ):
        raise ValueError("R123 column scale differs")
    scaled_columns = system.matrix * column_scale[np.newaxis, :]
    row_scale = np.maximum(
        np.maximum(
            np.max(np.abs(scaled_columns), axis=1), np.abs(system.right_hand_side)
        ),
        1.0,
    )
    scaled_matrix = scaled_columns / row_scale[:, np.newaxis]
    scaled_right = system.right_hand_side / row_scale
    singular_values = np.linalg.svd(scaled_matrix, compute_uv=False)
    smallest = float(singular_values[-1])
    condition = math.inf if smallest <= 0.0 else float(singular_values[0]) / smallest
    if not np.isfinite(condition) or condition > float(
        numeric_contract["maximum_scaled_condition_number"]
    ):
        return LocalSolution(
            "INVALID",
            None,
            False,
            condition,
            None,
            None,
            None,
            None,
            None,
            "SCALED_LOCAL_SYSTEM_ILL_CONDITIONED",
        )
    try:
        scaled_solution = np.linalg.solve(scaled_matrix, scaled_right)
    except np.linalg.LinAlgError:
        return LocalSolution(
            "INVALID",
            None,
            True,
            condition,
            None,
            None,
            None,
            None,
            None,
            "LOCAL_SYSTEM_SOLVE_FAILED",
        )
    solution = column_scale * scaled_solution
    if not np.all(np.isfinite(solution)):
        return LocalSolution(
            "INVALID",
            None,
            True,
            condition,
            None,
            None,
            None,
            None,
            None,
            "NONFINITE_LOCAL_SOLUTION",
        )
    scaled_residual = float(
        np.max(np.abs(scaled_matrix @ scaled_solution - scaled_right))
    )
    residual = system.matrix @ solution - system.right_hand_side
    denominator = np.maximum(
        np.abs(system.matrix) @ np.abs(solution) + np.abs(system.right_hand_side),
        1.0,
    )
    backward = float(np.max(np.abs(residual) / denominator))
    dynamics = float(np.max(np.abs(residual[:GENERALIZED_WIDTH])))
    effort = float(
        np.max(np.abs(residual[GENERALIZED_WIDTH : GENERALIZED_WIDTH + ACTUATOR_COUNT]))
    )
    point = float(np.max(np.abs(residual[GENERALIZED_WIDTH + ACTUATOR_COUNT :])))
    invalid = None
    if scaled_residual > float(numeric_contract["scaled_absolute_residual"]):
        invalid = "SCALED_EQUALITY_RESIDUAL_EXCEEDED"
    elif backward > float(numeric_contract["backward_error"]):
        invalid = "BACKWARD_ERROR_EXCEEDED"
    elif max(dynamics, effort, point) > float(
        numeric_contract["group_absolute_residual"]
    ):
        invalid = "PHYSICAL_GROUP_RESIDUAL_EXCEEDED"
    return LocalSolution(
        "VALID" if invalid is None else "INVALID",
        solution,
        True,
        condition,
        scaled_residual,
        backward,
        dynamics,
        effort,
        point,
        invalid,
    )


def audit_contact_cones(
    forces: NDArray[np.float64],
    *,
    active: NDArray[np.bool_],
    tolerance_newtons: float,
) -> dict[str, Any]:
    if forces.shape != (POINT_COUNT, POINT_FORCE_WIDTH) or active.shape != (
        POINT_COUNT,
    ):
        raise ValueError("R123 contact cone input differs")
    normal_margins = []
    friction_margins = []
    violations = []
    maximum_inactive = 0.0
    for point in range(POINT_COUNT):
        normal, right, forward = (float(value) for value in forces[point])
        if active[point]:
            normal_margin = normal
            friction_margin = FRICTION * normal - math.hypot(right, forward)
            normal_margins.append(normal_margin)
            friction_margins.append(friction_margin)
            if (
                normal_margin < -tolerance_newtons
                or friction_margin < -tolerance_newtons
            ):
                violations.append(point)
        else:
            maximum_inactive = max(
                maximum_inactive, float(np.max(np.abs(forces[point])))
            )
    return {
        "minimum_active_normal_margin_newtons": (
            min(normal_margins) if normal_margins else None
        ),
        "minimum_active_friction_margin_newtons": (
            min(friction_margins) if friction_margins else None
        ),
        "violating_active_point_ordinals": violations,
        "maximum_inactive_point_force_newtons": maximum_inactive,
        "friction_cone_feasible": not violations,
    }


def derive_fixed_pd_schedule(
    *, cache: Mapping[str, NDArray[Any]], descriptor: Mapping[str, Any]
) -> FixedPdSchedule:
    """Reproduce the exact R121 target/effort state machine and retain outputs."""

    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    if [int(row["dof_ordinal"]) for row in joints] != list(range(ACTUATOR_COUNT)) or [
        int(row["dof_ordinal"]) for row in actuators
    ] != list(range(ACTUATOR_COUNT)):
        raise ValueError("R123 fixed-PD descriptor order differs")
    position = np.asarray(cache["joint_position_rad"], dtype=np.float64)
    velocity = np.asarray(cache["velocity"], dtype=np.float64)[:, 6:]
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
                (
                    (1.0 - fraction) * velocity[interval]
                    + fraction * velocity[interval + 1]
                )
                * 1_000_000.0
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
                float(np.max(np.abs(current_effort * observed_velocity)) / 1_000_000.0),
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
        "claim_ceiling": "FIXED_PD_INPUT_SCHEDULE_PREFLIGHT_ONLY_NO_DYNAMICS",
    }
    return FixedPdSchedule(target_output, effort_output, audit)


def collocation_state(
    cache: Mapping[str, NDArray[Any]], interval: int, substep: int
) -> CollocationState:
    if not 0 <= interval < MOTOR_INTERVAL_COUNT or not 0 <= substep < 4:
        raise ValueError("R123 collocation index differs")
    fraction = substep / SUBSTEPS_PER_INTERVAL
    left = configuration_at(cache, interval)
    right = configuration_at(cache, interval + 1)
    tangent = rotation_log(right.root_rotation @ left.root_rotation.T)
    return CollocationState(
        configuration=Configuration(
            root_position=affine_sample(
                left.root_position, right.root_position, substep
            ),
            root_rotation=rotation_exp(fraction * tangent) @ left.root_rotation,
            joint_positions=affine_sample(
                left.joint_positions, right.joint_positions, substep
            ),
        ),
        velocity=affine_sample(
            cache["velocity"][interval], cache["velocity"][interval + 1], substep
        ),
        warm_acceleration=affine_sample(
            cache["acceleration"][interval],
            cache["acceleration"][interval + 1],
            substep,
        ),
    )


def execution_column_scale(
    *,
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    descriptor_effort: NDArray[np.float64],
) -> NDArray[np.float64]:
    acceleration = np.maximum(np.max(np.abs(cache["acceleration"]), axis=0), 1.0)
    effort = np.maximum(np.max(np.abs(descriptor_effort), axis=0), 1.0)
    force = max(float(np.sum(model.masses)) * abs(float(model.gravity[1])), 1.0)
    return np.concatenate(
        (
            acceleration,
            effort,
            np.full(POINT_COUNT * POINT_FORCE_WIDTH, force, dtype=np.float64),
        )
    )


def aggregate_execution_rows(
    *,
    rows: Sequence[Mapping[str, Any]],
    warm_acceleration: NDArray[Any],
    solved_acceleration: NDArray[np.float64] | None,
    solved_point_force: NDArray[np.float64] | None,
) -> dict[str, Any]:
    valid_rows = [row for row in rows if row.get("status") != "INVALID"]
    cone_feasible = [
        row for row in valid_rows if row.get("friction_cone_feasible") is True
    ]
    cone_infeasible = [
        row for row in valid_rows if row.get("friction_cone_feasible") is False
    ]
    result: dict[str, Any] = {
        "collocation_rows_recorded": len(rows),
        "numerically_valid_collocations": len(valid_rows),
        "friction_cone_feasible_collocations": len(cone_feasible),
        "friction_cone_infeasible_collocations": len(cone_infeasible),
        "friction_cone_violating_point_count": sum(
            len(row.get("violating_active_point_ordinals", ())) for row in valid_rows
        ),
    }
    for key in (
        "singular_value_condition_number",
        "scaled_absolute_residual",
        "backward_error",
        "dynamics_absolute_residual",
        "effort_absolute_residual",
        "point_absolute_residual",
        "maximum_fixed_effort_identity_error_newton_metres",
        "maximum_absolute_acceleration",
        "maximum_absolute_warm_acceleration_delta",
        "maximum_absolute_point_force_newtons",
        "maximum_inactive_point_force_newtons",
    ):
        values = [float(row[key]) for row in valid_rows if row.get(key) is not None]
        result[f"maximum_{key}"] = max(values) if values else None
    normal = [
        (float(row["minimum_active_normal_margin_newtons"]), int(row["collocation"]))
        for row in valid_rows
        if row.get("minimum_active_normal_margin_newtons") is not None
    ]
    friction = [
        (
            float(row["minimum_active_friction_margin_newtons"]),
            int(row["collocation"]),
        )
        for row in valid_rows
        if row.get("minimum_active_friction_margin_newtons") is not None
    ]
    result["minimum_active_normal_margin"] = (
        {"newtons": min(normal)[0], "collocation": min(normal)[1]} if normal else None
    )
    result["minimum_active_friction_margin"] = (
        {"newtons": min(friction)[0], "collocation": min(friction)[1]}
        if friction
        else None
    )
    if solved_acceleration is not None and solved_point_force is not None:
        warm_collocation = np.stack(
            [
                affine_sample(warm_acceleration[i], warm_acceleration[i + 1], substep)
                for i in range(MOTOR_INTERVAL_COUNT)
                for substep in range(SUBSTEPS_PER_INTERVAL)
            ]
        )
        result["acceleration_root_mean_square"] = float(
            np.sqrt(np.mean(solved_acceleration * solved_acceleration))
        )
        delta = solved_acceleration - warm_collocation
        result["warm_acceleration_delta_root_mean_square"] = float(
            np.sqrt(np.mean(delta * delta))
        )
        result["solution_array_sha256"] = {
            "generalized_acceleration": _array_sha256(solved_acceleration),
            "point_force_normal_right_forward_newtons": _array_sha256(
                solved_point_force
            ),
        }
    return result


def encode_solver_private_cache(
    *,
    outcome: ExecutionOutcome,
    profile_sha256: str,
    r122_report_sha256: str,
) -> tuple[bytes | None, dict[str, Any]]:
    if outcome.arrays is None:
        return None, {
            "status": "NOT_EMITTED_INVALID_EXECUTION",
            "candidate_or_corpus_authority": False,
        }
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r123-fixed-pd-inverse-dynamics-solver-private.v1",
        "candidate_or_corpus_authority": False,
        "collocation_count": COLLOCATION_COUNT,
        "feasibility": outcome.feasibility,
        "profile_sha256": profile_sha256,
        "r122_report_sha256": r122_report_sha256,
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
        "file_name": "solver-private-r123-fixed-pd-inverse-dynamics.npz",
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


def point_active(modes: NDArray[np.uint8], point_ordinal: int) -> bool:
    side = point_ordinal // 2
    point = point_ordinal % 2
    return int(modes[side]) in ((1, 3) if point == 0 else (2, 3))


def _invalid_collocation_row(
    *,
    collocation: int,
    interval: int,
    substep: int,
    modes: NDArray[np.uint8],
    active: NDArray[np.bool_],
    solution: LocalSolution,
) -> dict[str, Any]:
    return {
        "status": "INVALID",
        "collocation": collocation,
        "interval": interval,
        "substep": substep,
        "contact_modes": modes.tolist(),
        "active_point_ordinals": np.flatnonzero(active).tolist(),
        "singular_value_condition_number": solution.singular_value_condition_number,
        "scaled_absolute_residual": solution.scaled_absolute_residual,
        "backward_error": solution.backward_error,
        "dynamics_absolute_residual": solution.dynamics_absolute_residual,
        "effort_absolute_residual": solution.effort_absolute_residual,
        "point_absolute_residual": solution.point_absolute_residual,
        "invalid_reason": solution.invalid_reason,
    }


def _validate_source_reports(
    *,
    profile: Mapping[str, Any],
    r122: Mapping[str, Any],
    r121: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    source = profile["source"]
    if (
        r122.get("check")
        != "TRAIN-4-FIXED-PD-INVERSE-DYNAMICS-IMPLEMENTATION-CONFORMANCE"
        or r122.get("status") != "PASS"
        or r122.get("gate_decision")
        != "PERMIT_R123_SINGLE_BOUNDED_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or r122.get("repository", {}).get("commit")
        != source["r122"]["repository_commit"]
        or r122.get("repository", {}).get("dirty") is not False
        or r122.get("identities", {}).get("kernel_module_sha256")
        != source["r122"]["kernel_module_sha256"]
        or r122.get("identities", {}).get("conformance_module_sha256")
        != source["r122"]["conformance_module_sha256"]
        or r122.get("identities", {}).get("tool_sha256")
        != source["r122"]["tool_sha256"]
        or r122.get("r123_local_system_solves") != 0
        or r122.get("inverse_dynamics_execution_runs") != 0
        or r121.get("status") != "COMPLETE"
        or r121.get("future_single_execution_budget") != profile["execution_budget"]
        or r121.get("future_execution_output") != profile["execution_output"]
        or r113.get("status") != "PASS"
        or r120.get("status") != "PASS"
        or r120.get("solver_result", {}).get("accepted_exact_result", {}).get("status")
        != "PASS"
    ):
        raise ValueError("R123 source report contract differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    budget = profile.get("execution_budget", {})
    numeric = profile.get("numeric_contract", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleExecution"
        or scope.get("run_id") != "R123"
        or scope.get("collocation_count") != COLLOCATION_COUNT
        or scope.get("local_unknown_count") != LOCAL_UNKNOWN_COUNT
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or budget.get("process_count") != 1
        or budget.get("thread_count") != 1
        or budget.get("maximum_local_system_solves") != COLLOCATION_COUNT
        or budget.get("maximum_local_unknown_count") != LOCAL_UNKNOWN_COUNT
        or budget.get("maximum_wall_clock_seconds") != 7200
        or budget.get("maximum_resident_memory_bytes") != 8 * 1024**3
        or budget.get("random_seed") != 0
        or budget.get("randomized_restart_count") != 0
        or budget.get("resume_or_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
        or numeric.get("maximum_scaled_condition_number") != 1.0e12
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or decision.get("valid_complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_R124_FULL_KINODYNAMIC_EXECUTION_FORMULATION_ONLY"
        or decision.get("invalid") != "STOP_INVALID_EVIDENCE_WITHOUT_RESTART"
        or bounded.get("r123_retry") != "NOT_AUTHORIZED"
        or bounded.get("r124_formulation")
        != "AUTHORIZED_REPORT_ONLY_ON_VALID_R123_COMPLETION"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_kto_solve",
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
        raise ValueError("R123 execution profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R123 execution requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], actual: Mapping[str, str]
) -> None:
    expected = profile["single_thread_environment"]
    if dict(actual) != expected or _linux_thread_count() != 1:
        raise ValueError("R123 single-thread execution environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R123 validation results differ")
    return normalized


def _column(
    records: Sequence[Mapping[str, Any]], field: str, index: int
) -> NDArray[np.float64]:
    return np.asarray([row[field][index] for row in records], dtype=np.float64)


def _record_mask(
    mask: NDArray[np.bool_],
    name: str,
    counts: dict[str, int],
    channels: dict[str, set[int]],
) -> None:
    counts[name] += int(np.count_nonzero(mask))
    channels[name].update(int(value) for value in np.flatnonzero(mask))


def _array_sha256(array: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(array).tobytes()).hexdigest()


def _resident_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _linux_thread_count() -> int:
    status = Path("/proc/self/status").read_text(encoding="utf-8")
    for line in status.splitlines():
        if line.startswith("Threads:"):
            return int(line.split(":", 1)[1].strip())
    raise ValueError("R123 cannot read Linux thread count")
