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
from next_lab.exit_mode_owned_lift_conformance import EXIT_INTERVALS
from next_lab.exit_mode_owned_projected_schedule_execution import (
    derive_eventful_projected_fixed_pd_schedule,
)
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _contact_points,
    load_r120_cache,
)
from next_lab.fixed_pd_inverse_dynamics_execution import (
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
    Configuration,
    SpatialModel,
    build_spatial_model,
    inverse_dynamics,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    propagate_motion,
)
from next_lab.gauge_aware_fixed_pd_execution import (
    _collocation_row,
    aggregate_execution_rows,
    solve_gauge_aware_collocation,
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import (
    POINT_FORCE_WIDTH,
    ReducedLocalSystem,
    analytic_force_gauges,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor
from next_lab.projected_inverse_dynamics_composition import (
    ARRAY_SHAPES as R135_ARRAY_SHAPES,
)
from next_lab.projected_inverse_dynamics_composition import (
    assemble_projected_reduced_local_system,
    projected_reduced_column_scale,
)
from next_lab.tangent_velocity_projection_conformance import (
    _flat_line_audit,
    _projection_passes,
    project_tangent_velocity,
)

EXECUTION_ID = "nextengine.humanoid-projected-fixed-pd-inverse-dynamics.v1"
CHECK_ID = "TRAIN-4-PROJECTED-FIXED-PD-INVERSE-DYNAMICS"
R133_ARRAY_SHAPES = {
    "projected_generalized_velocity": (COLLOCATION_COUNT, GENERALIZED_WIDTH),
    "projection_delta_velocity": (COLLOCATION_COUNT, GENERALIZED_WIDTH),
    "applied_target_microradians": (MOTOR_INTERVAL_COUNT, ACTUATOR_COUNT),
    "applied_effort_micronewton_metres": (COLLOCATION_COUNT, ACTUATOR_COUNT),
}
ZERO_DOWNSTREAM_COUNTERS = (
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


@dataclass(frozen=True)
class R133Reconstruction:
    status: str
    invalid_reason: str | None
    root_position: NDArray[np.float64] | None
    root_rotation: NDArray[np.float64] | None
    joint_position: NDArray[np.float64] | None
    arrays: dict[str, NDArray[np.float64]] | None
    array_sha256: dict[str, str] | None
    configuration_sha256: dict[str, str] | None
    schedule_audit: dict[str, Any] | None
    event_audit: dict[str, Any] | None
    state_lift_evaluations: int
    projection_systems: int
    projection_factorizations: int
    projection_solves: int
    controller_schedule_derivations: int


@dataclass(frozen=True)
class ProjectedInverseDynamicsOutcome:
    status: str
    feasibility: str | None
    invalid_reason: str | None
    reconstruction: R133Reconstruction
    inverse_dynamics_system_assemblies: int
    singular_value_decompositions: int
    particular_solutions: int
    gauge_interval_classifications: int
    flight_collocations: int
    single_point_collocations: int
    flat_foot_collocations: int
    equality_rows: int
    expected_independent_rank: int
    active_point_cones: int
    rows: tuple[dict[str, Any], ...]
    aggregate: dict[str, Any]
    arrays: dict[str, NDArray[Any]] | None
    resource_usage: dict[str, Any]
    ordered_reduced_system_sha256: str


def execute_and_build_projected_fixed_pd_inverse_dynamics_report(
    *,
    profile_path: Path,
    r135_report_path: Path,
    r135_profile_path: Path,
    r135_conformance_module_path: Path,
    r135_tool_path: Path,
    r134_report_path: Path,
    r134_profile_path: Path,
    r134_module_path: Path,
    r134_tool_path: Path,
    r133_report_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r126_report_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    gauge_execution_module_path: Path,
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
    contact_conformance_module_path: Path,
    collocation_lift_module_path: Path,
    exit_mode_lift_module_path: Path,
    projection_module_path: Path,
    dynamics_kernel_path: Path,
    composition_module_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> tuple[dict[str, Any], bytes | None]:
    """Consume the sole R136 authority and build its report/cache bytes."""

    named_paths = {
        "profile": profile_path,
        "r135_report": r135_report_path,
        "r135_profile": r135_profile_path,
        "r135_conformance_module": r135_conformance_module_path,
        "r135_tool": r135_tool_path,
        "r134_report": r134_report_path,
        "r134_profile": r134_profile_path,
        "r134_module": r134_module_path,
        "r134_tool": r134_tool_path,
        "r133_report": r133_report_path,
        "r133_profile": r133_profile_path,
        "r133_module": r133_module_path,
        "r133_tool": r133_tool_path,
        "r126_report": r126_report_path,
        "r126_profile": r126_profile_path,
        "gauge_aware_kernel": gauge_aware_kernel_path,
        "gauge_execution_module": gauge_execution_module_path,
        "r126_conformance_module": r126_conformance_module_path,
        "r126_tool": r126_tool_path,
        "r121_report": r121_report_path,
        "r121_profile": r121_profile_path,
        "r113_report": r113_report_path,
        "r113_profile": r113_profile_path,
        "r120_report": r120_report_path,
        "r120_profile": r120_profile_path,
        "r120_cache": r120_cache_path,
        "v9_complete_clip": v9_complete_clip_path,
        "contact_conformance_module": contact_conformance_module_path,
        "collocation_lift_module": collocation_lift_module_path,
        "exit_mode_lift_module": exit_mode_lift_module_path,
        "projection_module": projection_module_path,
        "dynamics_kernel": dynamics_kernel_path,
        "composition_module": composition_module_path,
        "tool": tool_path,
    }
    paths = {name: path.resolve() for name, path in named_paths.items()}
    absent = [name for name, path in paths.items() if not path.is_file()]
    if absent:
        raise FileNotFoundError(f"R136 execution input is absent: {absent}")

    profile = json.loads(paths["profile"].read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    reports = {
        label: _load_bound_report(
            paths[f"{label.lower()}_report"], profile["source"][label.lower()], label
        )
        for label in ("R135", "R134", "R133", "R126", "R121", "R113", "R120")
    }
    _validate_sources(
        profile=profile,
        reports=reports,
        paths=paths,
        descriptor_bytes=descriptor_bytes,
    )

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(paths["r120_cache"], profile)
    with np.load(paths["v9_complete_clip"], allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R136 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=reports["R113"])
    outcome = execute_projected_fixed_pd_inverse_dynamics(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        descriptor=descriptor,
        r133=reports["R133"],
    )
    cache_bytes, cache_identity = encode_solver_private_cache(
        outcome=outcome,
        profile_sha256=sha256(paths["profile"]),
        r135_report_sha256=reports["R135"]["report_sha256"],
    )
    valid = outcome.status == "VALID_COMPLETE"
    decision_key = (
        "valid_feasible"
        if valid and outcome.feasibility == "FEASIBLE"
        else "valid_infeasible"
        if valid
        else "invalid"
    )
    reconstruction = outcome.reconstruction
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": "COMPLETE" if valid else "INVALID",
        "claim": profile["claim"],
        "gate_decision": profile["decision"][decision_key],
        "result_transition": profile["result_transitions"][decision_key],
        "scope": profile["scope"],
        "source_gates": {
            label.lower(): {
                "status": reports[label]["status"],
                "report_sha256": reports[label]["report_sha256"],
            }
            for label in reports
        },
        "r135_target_shape_compatibility_audit": r135_target_shape_compatibility_audit(),
        "r133_reconstruction": {
            "status": reconstruction.status,
            "invalid_reason": reconstruction.invalid_reason,
            "array_sha256": reconstruction.array_sha256,
            "configuration_sha256": reconstruction.configuration_sha256,
            "schedule_audit": reconstruction.schedule_audit,
            "event_audit": reconstruction.event_audit,
            "hash_guard_status": (
                "PASS" if reconstruction.array_sha256 is not None else "NOT_REACHED"
            ),
            "inverse_dynamics_system_assemblies_at_hash_verification": (
                0 if reconstruction.array_sha256 is not None else None
            ),
            "state_lift_evaluations": reconstruction.state_lift_evaluations,
            "projection_systems": reconstruction.projection_systems,
            "projection_factorizations": reconstruction.projection_factorizations,
            "projection_solves": reconstruction.projection_solves,
            "controller_schedule_derivations": reconstruction.controller_schedule_derivations,
            "independent_projected_schedule_result_claimed": False,
        },
        "solver_result": {
            "status": outcome.status,
            "feasibility": outcome.feasibility,
            "invalid_reason": outcome.invalid_reason,
            "inverse_dynamics_system_assemblies": outcome.inverse_dynamics_system_assemblies,
            "singular_value_decompositions": outcome.singular_value_decompositions,
            "particular_solutions": outcome.particular_solutions,
            "gauge_interval_classifications": outcome.gauge_interval_classifications,
            "flight_collocations": outcome.flight_collocations,
            "single_point_collocations": outcome.single_point_collocations,
            "flat_foot_collocations": outcome.flat_foot_collocations,
            "equality_rows": outcome.equality_rows,
            "expected_independent_rank": outcome.expected_independent_rank,
            "active_point_cones": outcome.active_point_cones,
            "ordered_reduced_system_float64_sha256": outcome.ordered_reduced_system_sha256,
            "aggregate": outcome.aggregate,
            "collocations": list(outcome.rows),
        },
        "solver_private_cache": cache_identity,
        "numeric_contract": profile["inverse_dynamics_numeric_contract"],
        "resource_usage": outcome.resource_usage,
        "execution_environment": dict(execution_environment),
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(paths["profile"]),
            **{
                f"{name}_sha256": sha256(path)
                for name, path in paths.items()
                if name not in {"profile", "tool"}
            },
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(paths["tool"]),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "projected_inverse_dynamics_execution_runs": 1,
        "state_lift_evaluations": reconstruction.state_lift_evaluations,
        "state_projection_systems": reconstruction.projection_systems,
        "projection_factorizations": reconstruction.projection_factorizations,
        "projection_solves": reconstruction.projection_solves,
        "controller_schedule_derivations": reconstruction.controller_schedule_derivations,
        "inverse_dynamics_system_assemblies": outcome.inverse_dynamics_system_assemblies,
        "singular_value_decompositions": outcome.singular_value_decompositions,
        "particular_solutions": outcome.particular_solutions,
        "gauge_interval_classifications": outcome.gauge_interval_classifications,
        **{counter: 0 for counter in ZERO_DOWNSTREAM_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report, cache_bytes


def execute_projected_fixed_pd_inverse_dynamics(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    descriptor: Mapping[str, Any],
    r133: Mapping[str, Any],
) -> ProjectedInverseDynamicsOutcome:
    """Reconstruct R133 once, then classify all pointwise R136 systems once."""

    budget = profile["execution_budget"]
    numeric = profile["inverse_dynamics_numeric_contract"]
    np.random.seed(int(budget["random_seed"]))
    started = time.monotonic()
    reconstruction = reconstruct_r133_inputs(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        descriptor=descriptor,
        r133=r133,
        started=started,
    )
    if reconstruction.status != "VALID" or reconstruction.arrays is None:
        return _empty_invalid_outcome(
            reconstruction=reconstruction,
            reason=reconstruction.invalid_reason or "R133_RECONSTRUCTION_INVALID",
            started=started,
            budget=budget,
        )

    assert reconstruction.root_position is not None
    assert reconstruction.root_rotation is not None
    assert reconstruction.joint_position is not None
    arrays = reconstruction.arrays
    projected_velocity = arrays["projected_generalized_velocity"]
    effort_microunits = arrays["applied_effort_micronewton_metres"]
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
    system_digest = hashlib.sha256()
    assemblies = 0
    decompositions = 0
    particulars = 0
    gauge_classifications = 0
    equality_rows = 0
    independent_rank = 0
    active_cones = 0
    contact_count_histogram = {0: 0, 1: 0, 2: 0}
    invalid_reason: str | None = None
    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    total_mass = float(np.sum(model.masses))
    gravity_y = float(model.gravity[1])

    for collocation in range(COLLOCATION_COUNT):
        interval = int(interval_index[collocation])
        substep = int(substep_index[collocation])
        configuration = Configuration(
            root_position=reconstruction.root_position[collocation],
            root_rotation=reconstruction.root_rotation[collocation],
            joint_positions=reconstruction.joint_position[collocation],
        )
        velocity = projected_velocity[collocation]
        modes = contact_modes[interval]
        active = tuple(
            ordinal for ordinal in range(POINT_COUNT) if point_active(modes, ordinal)
        )
        if len(active) not in contact_count_histogram:
            invalid_reason = "ACTIVE_POINT_COUNT_DIFFERS"
            break
        contact_count_histogram[len(active)] += 1
        mass, kinematics = mass_matrix(model, configuration)
        bias = inverse_dynamics(model, configuration, velocity, zero)
        motion = propagate_motion(model, kinematics, velocity, zero)
        jacobians = []
        jdot_v = []
        for point in points:
            body_slot = int(point["body_slot"])
            local = np.asarray(point["local_translation_metres"], dtype=np.float64)
            jacobians.append(point_jacobian(model, kinematics, body_slot, local))
            jdot_v.append(point_acceleration(kinematics, motion, body_slot, local))
        matrix, right = assemble_projected_reduced_local_system(
            mass=mass,
            bias_from_projected_velocity=bias,
            applied_effort_micronewton_metres=effort_microunits[collocation],
            contact_jacobians=np.stack(jacobians),
            contact_jdot_v_from_projected_velocity=np.stack(jdot_v),
            active_point_ordinals=active,
            modes=modes,
        )
        gauges, identities = analytic_force_gauges(
            modes=modes,
            points=points,
            active_point_ordinals=active,
            body_rotations=kinematics.body_rotations,
            body_positions=kinematics.body_positions,
        )
        system = ReducedLocalSystem(
            matrix=matrix,
            right_hand_side=right,
            active_point_ordinals=active,
            modes=modes.copy(),
            analytic_gauge_matrix=gauges,
            gauge_identities=identities,
        )
        assemblies += 1
        equality_rows += int(matrix.shape[0])
        independent_rank += int(matrix.shape[0] - gauges.shape[1])
        active_cones += len(active)
        system_digest.update(np.ascontiguousarray(matrix).tobytes())
        system_digest.update(np.ascontiguousarray(right).tobytes())
        if matrix.shape[1] > int(budget["maximum_local_unknown_count"]):
            invalid_reason = "LOCAL_UNKNOWN_BUDGET_EXCEEDED"
            break
        if gauges.shape[1] > int(budget["maximum_flat_foot_gauge_dimension"]):
            invalid_reason = "GAUGE_DIMENSION_BUDGET_EXCEEDED"
            break
        column_scale = projected_reduced_column_scale(
            r120_acceleration_knots=np.asarray(cache["acceleration"]),
            total_body_mass_kilograms=total_mass,
            gravity_y_metres_per_second_squared=gravity_y,
            active_point_count=len(active),
        )
        solution = solve_gauge_aware_collocation(
            system,
            column_scale=column_scale,
            numeric_contract=numeric,
        )
        decompositions += 1
        particulars += int(solution.particular_solution is not None)
        gauge_classifications += int(solution.gauge_feasibility is not None)
        row = _collocation_row(
            collocation=collocation,
            interval=interval,
            substep=substep,
            system=system,
            solution=solution,
        )
        row["projected_velocity_sha256"] = _array_sha256(velocity)
        row["applied_effort_micronewton_metres_sha256"] = _array_sha256(
            effort_microunits[collocation]
        )
        rows.append(row)
        if solution.status != "VALID" or solution.selected_solution is None:
            invalid_reason = solution.invalid_reason or "R136_COLLOCATION_INVALID"
            break

        selected = solution.selected_solution
        acceleration[collocation] = selected[:GENERALIZED_WIDTH]
        active_forces = selected[GENERALIZED_WIDTH:].reshape(
            len(active), POINT_FORCE_WIDTH
        )
        for local_ordinal, point_ordinal in enumerate(active):
            point_force[collocation, point_ordinal] = active_forces[local_ordinal]
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
            invalid_reason = _resource_invalid_reason(budget, started)
            if invalid_reason is not None:
                break

    expected = profile["expected_inventory"]
    counters = {
        "inverse_dynamics_system_assemblies": assemblies,
        "singular_value_decompositions": decompositions,
        "particular_solutions": particulars,
        "gauge_interval_classifications": gauge_classifications,
        "flight_collocations": contact_count_histogram[0],
        "single_point_collocations": contact_count_histogram[1],
        "flat_foot_collocations": contact_count_histogram[2],
        "equality_rows": equality_rows,
        "expected_independent_rank": independent_rank,
        "active_point_cones": active_cones,
    }
    if len(rows) == COLLOCATION_COUNT and any(
        counters[name] != int(expected[name]) for name in counters
    ):
        invalid_reason = invalid_reason or "R136_COUNTER_CLOSURE_DIFFERS"
    if len(rows) != COLLOCATION_COUNT and invalid_reason is None:
        invalid_reason = "R136_COLLOCATION_CLOSURE_DIFFERS"
    invalid_reason = invalid_reason or _budget_counter_invalid_reason(
        budget=budget,
        reconstruction=reconstruction,
        counters=counters,
    )
    invalid_reason = invalid_reason or _resource_invalid_reason(budget, started)
    valid = invalid_reason is None and len(rows) == COLLOCATION_COUNT
    output_arrays: dict[str, NDArray[Any]] | None = None
    feasibility: str | None = None
    if valid:
        feasibility = (
            "FEASIBLE" if bool(np.all(collocation_feasible == 1)) else "INFEASIBLE"
        )
        output_arrays = {
            **arrays,
            "root_position_metres": reconstruction.root_position,
            "root_rotation_matrix": reconstruction.root_rotation,
            "joint_position_radians": reconstruction.joint_position,
            "applied_effort_newton_metres": effort_microunits / 1_000_000.0,
            "generalized_acceleration": acceleration,
            "point_force_normal_right_forward_newtons": point_force,
            "gauge_interval_lower": interval_lower,
            "gauge_interval_upper": interval_upper,
            "gauge_witness_alpha": witness_alpha,
            "collocation_feasible": collocation_feasible,
            "interval_index": interval_index,
            "substep_index": substep_index,
        }
    aggregate = aggregate_execution_rows(rows, output_arrays)
    elapsed = time.monotonic() - started
    return ProjectedInverseDynamicsOutcome(
        status="VALID_COMPLETE" if valid else "INVALID",
        feasibility=feasibility,
        invalid_reason=invalid_reason,
        reconstruction=reconstruction,
        inverse_dynamics_system_assemblies=assemblies,
        singular_value_decompositions=decompositions,
        particular_solutions=particulars,
        gauge_interval_classifications=gauge_classifications,
        flight_collocations=contact_count_histogram[0],
        single_point_collocations=contact_count_histogram[1],
        flat_foot_collocations=contact_count_histogram[2],
        equality_rows=equality_rows,
        expected_independent_rank=independent_rank,
        active_point_cones=active_cones,
        rows=tuple(rows),
        aggregate=aggregate,
        arrays=output_arrays,
        resource_usage=_resource_usage(
            valid=valid, elapsed=elapsed, budget=budget, invalid_reason=invalid_reason
        ),
        ordered_reduced_system_sha256=system_digest.hexdigest(),
    )


def reconstruct_r133_inputs(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    descriptor: Mapping[str, Any],
    r133: Mapping[str, Any],
    started: float,
) -> R133Reconstruction:
    """Perform the sole in-memory R133 input reconstruction owned by R136."""

    numeric = profile["projection_numeric_contract"]
    budget = profile["execution_budget"]
    root_position = np.empty((COLLOCATION_COUNT, 3), dtype=np.float64)
    root_rotation = np.empty((COLLOCATION_COUNT, 3, 3), dtype=np.float64)
    joint_position = np.empty((COLLOCATION_COUNT, ACTUATOR_COUNT), dtype=np.float64)
    projected = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    delta = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    lifts = 0
    projection_systems = 0
    factorizations = 0
    solves = 0
    invalid_reason: str | None = None
    exit_intervals = frozenset(int(value) for value in EXIT_INTERVALS)

    for collocation in range(COLLOCATION_COUNT):
        interval, substep = divmod(collocation, SUBSTEPS_PER_INTERVAL)
        state = collocation_state(cache, interval, substep)
        lifts += 1
        root_position[collocation] = state.configuration.root_position
        root_rotation[collocation] = state.configuration.root_rotation
        joint_position[collocation] = state.configuration.joint_positions
        source_velocity = np.asarray(state.velocity, dtype=np.float64)
        selected_velocity = (
            np.array(cache["velocity"][interval], copy=True)
            if interval in exit_intervals
            else np.array(source_velocity, copy=True)
        )
        modes = contact_modes[interval]
        active = tuple(
            ordinal for ordinal in range(POINT_COUNT) if point_active(modes, ordinal)
        )
        if not active:
            projected_row = selected_velocity
            delta_row = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
        else:
            mass, kinematics = mass_matrix(model, state.configuration)
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
            projection_systems += 1
            factorizations += 1
            solves += 1
            analysis = project_tangent_velocity(
                mass=mass,
                jacobian=jacobian,
                velocity=selected_velocity,
                expected_rank=3 if len(active) == 1 else 5,
                numeric=numeric,
            )
            if analysis.projected_velocity is None or analysis.delta_velocity is None:
                invalid_reason = "R133_PROJECTION_RESULT_ABSENT"
                break
            projected_row = analysis.projected_velocity
            delta_row = analysis.delta_velocity
            flat_audit = (
                _flat_line_audit(
                    model=model,
                    kinematics=kinematics,
                    original_velocity=selected_velocity,
                    projected_velocity=projected_row,
                    points=points,
                    active=active,
                )
                if len(active) == 2
                else None
            )
            if not _projection_passes(
                analysis=analysis,
                numeric=numeric,
                flat_audit=flat_audit,
            ):
                invalid_reason = "R133_PROJECTION_CONFORMANCE_GUARD_FAILED"
                break
        projected[collocation] = projected_row
        delta[collocation] = delta_row
        if collocation % 64 == 63:
            invalid_reason = _resource_invalid_reason(budget, started)
            if invalid_reason is not None:
                break

    schedule = None
    event_audit = None
    controller_derivations = 0
    array_hashes = None
    configuration_hashes = None
    arrays = None
    if invalid_reason is None and lifts == COLLOCATION_COUNT:
        eventful = derive_eventful_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=descriptor,
            projected_velocity=projected,
        )
        controller_derivations = 1
        schedule = eventful.schedule.audit
        event_audit = eventful.event_audit
        arrays = {
            "projected_generalized_velocity": projected,
            "projection_delta_velocity": delta,
            "applied_target_microradians": eventful.schedule.applied_target_microradians,
            "applied_effort_micronewton_metres": eventful.schedule.applied_effort_micronewton_metres,
        }
        try:
            array_hashes = validate_r133_reconstruction_hashes(
                arrays=arrays,
                expected_sha256=profile["r133_array_hash_contract"],
                inverse_dynamics_system_assemblies=0,
            )
        except ValueError as error:
            invalid_reason = str(error)
        if schedule != r133.get("projected_fixed_pd_schedule_audit"):
            invalid_reason = invalid_reason or "R133_SCHEDULE_AUDIT_DIFFERS"
        if event_audit != r133.get("controller_activation_event_audit"):
            invalid_reason = invalid_reason or "R133_EVENT_AUDIT_DIFFERS"
        configuration_hashes = {
            "root_position_metres": _array_sha256(root_position),
            "root_rotation_matrix": _array_sha256(root_rotation),
            "joint_position_radians": _array_sha256(joint_position),
        }
    elif invalid_reason is None:
        invalid_reason = "R133_STATE_LIFT_CLOSURE_DIFFERS"

    expected = profile["expected_inventory"]
    if invalid_reason is None and (
        projection_systems != int(expected["projection_systems"])
        or factorizations != int(expected["projection_factorizations"])
        or solves != int(expected["projection_solves"])
        or controller_derivations != int(expected["controller_schedule_derivations"])
    ):
        invalid_reason = "R133_RECONSTRUCTION_COUNTER_CLOSURE_DIFFERS"
    if invalid_reason is not None:
        arrays = None
    return R133Reconstruction(
        status="VALID" if invalid_reason is None else "INVALID",
        invalid_reason=invalid_reason,
        root_position=root_position if invalid_reason is None else None,
        root_rotation=root_rotation if invalid_reason is None else None,
        joint_position=joint_position if invalid_reason is None else None,
        arrays=arrays,
        array_sha256=array_hashes,
        configuration_sha256=configuration_hashes,
        schedule_audit=schedule,
        event_audit=event_audit,
        state_lift_evaluations=lifts,
        projection_systems=projection_systems,
        projection_factorizations=factorizations,
        projection_solves=solves,
        controller_schedule_derivations=controller_derivations,
    )


def validate_r133_reconstruction_hashes(
    *,
    arrays: Mapping[str, NDArray[Any]],
    expected_sha256: Mapping[str, str],
    inverse_dynamics_system_assemblies: int,
) -> dict[str, str]:
    """Validate the real R133 shapes/hashes before the first R136 ID system."""

    if inverse_dynamics_system_assemblies != 0:
        raise ValueError("R133 hashes were not checked before inverse dynamics")
    if set(arrays) != set(R133_ARRAY_SHAPES) or set(expected_sha256) != set(
        R133_ARRAY_SHAPES
    ):
        raise ValueError("R133 reconstructed array inventory differs")
    actual: dict[str, str] = {}
    for name, shape in R133_ARRAY_SHAPES.items():
        value = np.asarray(arrays[name])
        expected = expected_sha256[name]
        if (
            value.dtype != np.float64
            or value.shape != shape
            or not np.all(np.isfinite(value))
            or not isinstance(expected, str)
            or len(expected) != 64
        ):
            raise ValueError(f"R133 reconstructed {name} shape or dtype differs")
        actual[name] = _array_sha256(value)
        if actual[name] != expected:
            raise ValueError(f"R133 reconstructed {name} hash differs")
    return actual


def r135_target_shape_compatibility_audit() -> dict[str, Any]:
    """Record the bounded R135 synthetic target-shape metadata defect."""

    declared = tuple(R135_ARRAY_SHAPES["applied_target_microradians"])
    actual = tuple(R133_ARRAY_SHAPES["applied_target_microradians"])
    return {
        "status": (
            "PASS_LOCAL_INPUT_GUARD_REPAIRED"
            if declared == (COLLOCATION_COUNT, ACTUATOR_COUNT)
            and actual == (MOTOR_INTERVAL_COUNT, ACTUATOR_COUNT)
            else "FAIL"
        ),
        "r135_synthetic_declared_shape": list(declared),
        "r133_real_schedule_shape": list(actual),
        "dynamics_input_role": "NONE_TARGET_IS_PROVENANCE_ONLY",
        "repair": "R136 independently checks the real 800x23 target hash before ID",
        "frozen_r135_module_modified": False,
    }


def encode_solver_private_cache(
    *,
    outcome: ProjectedInverseDynamicsOutcome,
    profile_sha256: str,
    r135_report_sha256: str,
) -> tuple[bytes | None, dict[str, Any]]:
    if outcome.arrays is None:
        return None, {
            "status": "NOT_EMITTED_INVALID_EXECUTION",
            "candidate_or_corpus_authority": False,
        }
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r136-projected-fixed-pd-solver-private.v1",
        "candidate_or_corpus_authority": False,
        "collocation_count": COLLOCATION_COUNT,
        "feasibility": outcome.feasibility,
        "profile_sha256": profile_sha256,
        "r135_report_sha256": r135_report_sha256,
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
        "file_name": "solver-private-r136-projected-fixed-pd.npz",
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


def _empty_invalid_outcome(
    *,
    reconstruction: R133Reconstruction,
    reason: str,
    started: float,
    budget: Mapping[str, Any],
) -> ProjectedInverseDynamicsOutcome:
    elapsed = time.monotonic() - started
    aggregate = aggregate_execution_rows((), None)
    return ProjectedInverseDynamicsOutcome(
        status="INVALID",
        feasibility=None,
        invalid_reason=reason,
        reconstruction=reconstruction,
        inverse_dynamics_system_assemblies=0,
        singular_value_decompositions=0,
        particular_solutions=0,
        gauge_interval_classifications=0,
        flight_collocations=0,
        single_point_collocations=0,
        flat_foot_collocations=0,
        equality_rows=0,
        expected_independent_rank=0,
        active_point_cones=0,
        rows=(),
        aggregate=aggregate,
        arrays=None,
        resource_usage=_resource_usage(
            valid=False, elapsed=elapsed, budget=budget, invalid_reason=reason
        ),
        ordered_reduced_system_sha256=hashlib.sha256().hexdigest(),
    )


def _budget_counter_invalid_reason(
    *,
    budget: Mapping[str, Any],
    reconstruction: R133Reconstruction,
    counters: Mapping[str, int],
) -> str | None:
    checks = (
        (
            reconstruction.state_lift_evaluations,
            "maximum_state_lift_evaluations",
            "STATE_LIFT_BUDGET_EXCEEDED",
        ),
        (
            reconstruction.projection_systems,
            "maximum_projection_systems",
            "PROJECTION_SYSTEM_BUDGET_EXCEEDED",
        ),
        (
            reconstruction.projection_factorizations,
            "maximum_projection_factorizations",
            "PROJECTION_FACTORIZATION_BUDGET_EXCEEDED",
        ),
        (
            reconstruction.projection_solves,
            "maximum_projection_solves",
            "PROJECTION_SOLVE_BUDGET_EXCEEDED",
        ),
        (
            reconstruction.controller_schedule_derivations,
            "maximum_controller_schedule_derivations",
            "CONTROLLER_SCHEDULE_BUDGET_EXCEEDED",
        ),
        (
            counters["inverse_dynamics_system_assemblies"],
            "maximum_inverse_dynamics_system_assemblies",
            "INVERSE_DYNAMICS_ASSEMBLY_BUDGET_EXCEEDED",
        ),
        (
            counters["singular_value_decompositions"],
            "maximum_singular_value_decompositions",
            "SVD_BUDGET_EXCEEDED",
        ),
        (
            counters["particular_solutions"],
            "maximum_particular_solutions",
            "PARTICULAR_SOLUTION_BUDGET_EXCEEDED",
        ),
        (
            counters["gauge_interval_classifications"],
            "maximum_gauge_interval_classifications",
            "GAUGE_CLASSIFICATION_BUDGET_EXCEEDED",
        ),
    )
    for actual, key, reason in checks:
        if actual > int(budget[key]):
            return reason
    return None


def _resource_invalid_reason(budget: Mapping[str, Any], started: float) -> str | None:
    if time.monotonic() - started > float(budget["maximum_wall_clock_seconds"]):
        return "WALL_CLOCK_BUDGET_EXHAUSTED"
    if _resident_memory_bytes() > int(budget["maximum_resident_memory_bytes"]):
        return "RESIDENT_MEMORY_BUDGET_EXHAUSTED"
    if _linux_thread_count() > int(budget["thread_count"]):
        return "THREAD_BUDGET_EXCEEDED"
    return None


def _resource_usage(
    *,
    valid: bool,
    elapsed: float,
    budget: Mapping[str, Any],
    invalid_reason: str | None,
) -> dict[str, Any]:
    return {
        "status": "PASS" if valid else "FAIL",
        "invalid_reason": None if valid else invalid_reason,
        "wall_clock_seconds": elapsed,
        "maximum_resident_memory_bytes": _resident_memory_bytes(),
        "maximum_observed_os_thread_count": _linux_thread_count(),
        "execution_process_count": 1,
        "child_processes_spawned_during_execution": 0,
        "random_seed": int(budget["random_seed"]),
        "randomized_restart_count": 0,
        "resume_or_warm_restart": "FORBIDDEN_AND_NOT_USED",
        "manual_intervention": "FORBIDDEN_AND_NOT_USED",
    }


def _validate_sources(
    *,
    profile: Mapping[str, Any],
    reports: Mapping[str, Mapping[str, Any]],
    paths: Mapping[str, Path],
    descriptor_bytes: bytes,
) -> None:
    expected_status = {
        "R135": "PASS",
        "R134": "COMPLETE",
        "R133": "PASS",
        "R126": "PASS",
        "R121": "COMPLETE",
        "R113": "PASS",
        "R120": "PASS",
    }
    if any(
        reports[label].get("status") != status
        for label, status in expected_status.items()
    ):
        raise ValueError("R136 upstream status differs")
    if (
        reports["R135"].get("gate_decision")
        != "PERMIT_ONE_R136_PROJECTED_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or reports["R135"].get("result_transition")
        != "R135_PASS_R136_SINGLE_EXECUTION_ONLY"
    ):
        raise ValueError("R136 authority gate differs")
    direct_identities = {
        "r135_profile": profile["source"]["r135"]["profile_sha256"],
        "r135_conformance_module": profile["source"]["r135"][
            "conformance_module_sha256"
        ],
        "r135_tool": profile["source"]["r135"]["tool_sha256"],
        "r134_profile": profile["source"]["r134"]["profile_sha256"],
        "r134_module": profile["source"]["r134"]["module_sha256"],
        "r134_tool": profile["source"]["r134"]["tool_sha256"],
        "r133_profile": profile["source"]["r133"]["profile_sha256"],
        "r133_module": profile["source"]["r133"]["module_sha256"],
        "r133_tool": profile["source"]["r133"]["tool_sha256"],
        "r126_profile": profile["source"]["r126"]["profile_sha256"],
        "gauge_aware_kernel": profile["source"]["r126"]["gauge_aware_kernel_sha256"],
        "r126_conformance_module": profile["source"]["r126"][
            "conformance_module_sha256"
        ],
        "r126_tool": profile["source"]["r126"]["tool_sha256"],
        "r121_profile": profile["source"]["r121"]["profile_sha256"],
        "r113_profile": profile["source"]["r113"]["profile_sha256"],
        "r120_profile": profile["source"]["r120"]["profile_sha256"],
        "r120_cache": profile["source"]["r120"]["cache_sha256"],
        "v9_complete_clip": profile["source"]["v9_complete_clip_sha256"],
        "contact_conformance_module": profile["source"][
            "contact_conformance_module_sha256"
        ],
        "collocation_lift_module": profile["source"]["collocation_lift_module_sha256"],
        "exit_mode_lift_module": profile["source"]["exit_mode_lift_module_sha256"],
        "projection_module": profile["source"]["projection_module_sha256"],
        "dynamics_kernel": profile["source"]["dynamics_kernel_sha256"],
        "composition_module": profile["source"]["composition_module_sha256"],
        "gauge_execution_module": profile["source"]["gauge_execution_module_sha256"],
    }
    for name, expected in direct_identities.items():
        if sha256(paths[name]) != expected:
            raise ValueError(f"R136 {name} identity differs")
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve())
        != profile["source"]["execution_module_sha256"]
        or sha256(paths["tool"]) != profile["source"]["tool_sha256"]
    ):
        raise ValueError("R136 current implementation identity differs")
    if r135_target_shape_compatibility_audit()["status"] != (
        "PASS_LOCAL_INPUT_GUARD_REPAIRED"
    ):
        raise ValueError("R136 R135 target-shape compatibility audit failed")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R136 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R136 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    expected = profile.get("expected_inventory", {})
    budget = profile.get("execution_budget", {})
    projection_numeric = profile.get("projection_numeric_contract", {})
    numeric = profile.get("inverse_dynamics_numeric_contract", {})
    compatibility = profile.get("r135_target_shape_compatibility_contract", {})
    source = profile.get("source", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleExecution"
        or scope.get("run_id") != "R136"
        or scope.get("collocation_count") != COLLOCATION_COUNT
        or scope.get("pointwise_inverse_dynamics_only") is not True
        or scope.get("integration") is not False
        or scope.get("impact_or_release_impulse") is not False
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or expected.get("state_lift_evaluations") != COLLOCATION_COUNT
        or expected.get("projection_systems") != 2640
        or expected.get("projection_factorizations") != 2640
        or expected.get("projection_solves") != 2640
        or expected.get("controller_schedule_derivations") != 1
        or expected.get("inverse_dynamics_system_assemblies") != COLLOCATION_COUNT
        or expected.get("singular_value_decompositions") != COLLOCATION_COUNT
        or expected.get("particular_solutions") != COLLOCATION_COUNT
        or expected.get("gauge_interval_classifications") != 2316
        or expected.get("equality_rows") != 107668
        or expected.get("expected_independent_rank") != 105352
        or expected.get("active_point_cones") != 4956
        or budget.get("process_count") != 1
        or budget.get("thread_count") != 1
        or budget.get("maximum_state_lift_evaluations") != COLLOCATION_COUNT
        or budget.get("maximum_projection_systems") != 2640
        or budget.get("maximum_projection_factorizations") != 2640
        or budget.get("maximum_projection_solves") != 2640
        or budget.get("maximum_controller_schedule_derivations") != 1
        or budget.get("maximum_inverse_dynamics_system_assemblies") != COLLOCATION_COUNT
        or budget.get("maximum_singular_value_decompositions") != COLLOCATION_COUNT
        or budget.get("maximum_particular_solutions") != COLLOCATION_COUNT
        or budget.get("maximum_gauge_interval_classifications") != 2316
        or budget.get("maximum_local_unknown_count") != 35
        or budget.get("maximum_flat_foot_gauge_dimension") != 1
        or budget.get("maximum_wall_clock_seconds") != 7200
        or budget.get("maximum_resident_memory_bytes") != 8 * 1024**3
        or budget.get("random_seed") != 0
        or budget.get("randomized_restart_count") != 0
        or budget.get("resume_or_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
        or projection_numeric
        != {
            "float_dtype": "float64",
            "rank_revealing_relative_null_maximum": 1.0e-12,
            "rank_revealing_relative_retained_minimum": 1.0e-10,
            "maximum_mass_matrix_relative_symmetry": 1.0e-10,
            "maximum_active_point_velocity_absolute_metres_per_second": 1.0e-10,
            "maximum_scaled_kkt_residual": 1.0e-10,
            "maximum_closed_form_to_kkt_absolute": 1.0e-10,
            "maximum_projection_idempotence_error": 1.0e-10,
            "maximum_mass_orthogonality_relative_error": 1.0e-10,
            "maximum_kinetic_energy_increase_joules": 1.0e-12,
            "maximum_gauge_velocity_effect_absolute": 1.0e-10,
            "maximum_flat_rigid_line_compatibility_metres_per_second_squared": 1.0e-10,
            "ambiguity_policy": "FAIL_WITHOUT_THRESHOLD_TUNING_OR_RETRY",
        }
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or numeric.get("analytic_gauge_scaled_residual") != 1.0e-10
        or numeric.get("analytic_to_svd_null_projector_spectral_error") != 1.0e-8
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("physical_group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or numeric.get("quadratic_boundary_relative_ambiguity") != 1.0e-12
        or compatibility.get("r135_declared_target_shape") != [3200, 23]
        or compatibility.get("r133_real_target_shape") != [800, 23]
        or profile.get("r133_array_hash_contract")
        != {
            "projected_generalized_velocity": "e57bcb75629d403e04cc6d3197e8e2dada19ac0c0ef4c920083f2276c33bfb17",
            "projection_delta_velocity": "6308386172a7341885acb46b51ef68189fcd5be1a152db7a72f1dab8d0e86760",
            "applied_target_microradians": "195810f7e9c20b15ec6ddf165c8873f905a96414fe055a0dae7a2b45382be1a3",
            "applied_effort_micronewton_metres": "5ad7ac66c4d394ef5d06d111d86f58120069d42e61d1adf1ac82ca1d1d42a1ff",
        }
        or profile.get("decision")
        != {
            "valid_feasible": "PERMIT_SEPARATE_REPORT_ONLY_PROJECTED_STATE_KINODYNAMIC_FORMULATION_ONLY",
            "valid_infeasible": "STOP_VALID_PROJECTED_FIXED_PD_CONE_INFEASIBILITY_FOR_RESEARCH",
            "invalid": "STOP_INVALID_EVIDENCE_WITHOUT_RESTART",
        }
        or profile.get("result_transitions")
        != {
            "valid_feasible": "R136_VALID_FEASIBLE_FORMULATION_ONLY",
            "valid_infeasible": "R136_VALID_INFEASIBLE_RESEARCH_REQUIRED",
            "invalid": "R136_CONSUMED_INVALID_NO_RETRY",
        }
        or profile.get("bounded_acceptance")
        != {
            "r136_retry": "NOT_AUTHORIZED",
            "additional_projected_schedule_execution": "NOT_AUTHORIZED",
            "additional_inverse_dynamics_execution": "NOT_AUTHORIZED",
            "projected_state_kinodynamic_formulation": "AUTHORIZED_REPORT_ONLY_ON_VALID_FEASIBLE_R136",
            "kinodynamic_solve": "NOT_AUTHORIZED",
            "candidate_artifact": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        }
        or not isinstance(source.get("execution_module_sha256"), str)
        or not isinstance(source.get("tool_sha256"), str)
        or len(source["execution_module_sha256"]) != 64
        or len(source["tool_sha256"]) != 64
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
        raise ValueError("R136 execution profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R136 execution requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], actual: Mapping[str, str]
) -> None:
    if (
        dict(actual) != profile["single_thread_environment"]
        or _linux_thread_count() != 1
    ):
        raise ValueError("R136 single-thread execution environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R136 validation results differ")
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
    raise ValueError("R136 Linux thread count is unavailable")
