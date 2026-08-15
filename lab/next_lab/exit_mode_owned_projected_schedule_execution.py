from __future__ import annotations

import hashlib
import json
import resource
import time
from collections import Counter
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from itertools import pairwise
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_state_consistency_formulation import canonical_json, sha256
from next_lab.exit_mode_owned_lift_conformance import EXIT_INTERVALS
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _contact_points,
    load_r120_cache,
)
from next_lab.fixed_pd_inverse_dynamics_execution import (
    FixedPdSchedule,
    _column,
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
from next_lab.motor_mirror import validate_current_biomechanics_descriptor
from next_lab.tangent_projected_fixed_pd_execution import (
    _active_projection_row,
    _flight_projection_row,
    aggregate_projection_rows,
)
from next_lab.tangent_velocity_projection_conformance import (
    _flat_line_audit,
    _projection_passes,
    project_tangent_velocity,
)

EXECUTION_ID = "nextengine.humanoid-exit-mode-owned-projected-schedule.v1"
CHECK_ID = "TRAIN-4-EXIT-MODE-OWNED-PROJECTED-SCHEDULE"
CONTROLLER_CATEGORIES = (
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
BOUNDED_ACCEPTANCE = {
    "r133_retry": "NOT_AUTHORIZED",
    "r134_projected_inverse_dynamics_formulation": "AUTHORIZED_ON_R133_PASS_ONLY",
    "additional_projected_schedule_execution": "NOT_AUTHORIZED",
    "inverse_dynamics_execution": "NOT_AUTHORIZED",
    "kinodynamic_solve": "NOT_AUTHORIZED",
    "candidate_artifact": "NOT_AUTHORIZED",
    "physx": "NOT_AUTHORIZED",
    "all_17": "NOT_AUTHORIZED",
    "full_v19": "NOT_AUTHORIZED",
    "training": "NOT_AUTHORIZED",
}
ZERO_DOWNSTREAM_COUNTERS = (
    "inverse_dynamics_system_assemblies",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


@dataclass(frozen=True)
class EventfulFixedPdSchedule:
    schedule: FixedPdSchedule
    events: tuple[dict[str, Any], ...]
    event_audit: dict[str, Any]


@dataclass(frozen=True)
class ProjectedScheduleOutcome:
    status: str
    invalid_reason: str | None
    projection_rows: tuple[dict[str, Any], ...]
    projection_aggregate: dict[str, Any]
    schedule: EventfulFixedPdSchedule | None
    projection_systems: int
    projection_factorizations: int
    projection_solves: int
    controller_schedule_derivations: int
    changed_base_velocity_rows: int
    r132_anchor_rows_reproduced: int
    ordered_system_sha256: str
    resource_usage: dict[str, Any]


def execute_and_build_exit_mode_owned_projected_schedule_report(
    *,
    profile_path: Path,
    r132_report_path: Path,
    r132_profile_path: Path,
    r132_module_path: Path,
    r132_tool_path: Path,
    r129_report_path: Path,
    r129_profile_path: Path,
    projection_module_path: Path,
    r129_tool_path: Path,
    r130_report_path: Path,
    r130_profile_path: Path,
    r130_execution_module_path: Path,
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
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> dict[str, Any]:
    """Consume the sole R133 authority and stop before inverse dynamics."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r132_report_path,
            r132_profile_path,
            r132_module_path,
            r132_tool_path,
            r129_report_path,
            r129_profile_path,
            projection_module_path,
            r129_tool_path,
            r130_report_path,
            r130_profile_path,
            r130_execution_module_path,
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
            tool_path,
        )
    )
    (
        profile_path,
        r132_report_path,
        r132_profile_path,
        r132_module_path,
        r132_tool_path,
        r129_report_path,
        r129_profile_path,
        projection_module_path,
        r129_tool_path,
        r130_report_path,
        r130_profile_path,
        r130_execution_module_path,
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
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R133 execution input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    source = profile["source"]
    r132 = _load_bound_report(r132_report_path, source["r132"], "R132")
    r129 = _load_bound_report(r129_report_path, source["r129"], "R129")
    r130 = _load_bound_report(r130_report_path, source["r130"], "R130")
    r121 = _load_bound_report(r121_report_path, source["r121"], "R121")
    r113 = _load_bound_report(r113_report_path, source["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, source["r120"], "R120")
    _validate_source_files(
        profile=profile,
        r132_profile_path=r132_profile_path,
        r132_module_path=r132_module_path,
        r132_tool_path=r132_tool_path,
        r129_profile_path=r129_profile_path,
        projection_module_path=projection_module_path,
        r129_tool_path=r129_tool_path,
        r130_profile_path=r130_profile_path,
        r130_execution_module_path=r130_execution_module_path,
        r121_profile_path=r121_profile_path,
        r113_profile_path=r113_profile_path,
        r120_profile_path=r120_profile_path,
        r120_cache_path=r120_cache_path,
        v9_complete_clip_path=v9_complete_clip_path,
        collocation_lift_module_path=collocation_lift_module_path,
        dynamics_kernel_path=dynamics_kernel_path,
    )
    _validate_source_contracts(
        profile=profile,
        r132=r132,
        r129=r129,
        r130=r130,
        r121=r121,
        r113=r113,
        r120=r120,
    )
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != source["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve()) != source["execution_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R133 current implementation identity differs")

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R133 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=r113)

    outcome = execute_exit_mode_owned_projected_schedule(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        descriptor=descriptor,
        r132=r132,
        r130=r130,
    )
    decision_key = "pass" if outcome.status == "PASS" else "fail"
    schedule_audit = (
        None if outcome.schedule is None else outcome.schedule.schedule.audit
    )
    event_audit = None if outcome.schedule is None else outcome.schedule.event_audit
    events = () if outcome.schedule is None else outcome.schedule.events
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": outcome.status,
        "claim": profile["claim"],
        "gate_decision": profile["decision"][decision_key],
        "result_transition": profile["result_transitions"][decision_key],
        "invalid_reason": outcome.invalid_reason,
        "scope": profile["scope"],
        "source_gates": {
            "r132_status": r132["status"],
            "r132_gate_decision": r132["gate_decision"],
            "r132_report_sha256": r132["report_sha256"],
            "r129_status": r129["status"],
            "r129_report_sha256": r129["report_sha256"],
            "r130_status": r130["status"],
            "r130_projection_status": r130["projection_result"]["status"],
            "r130_report_sha256": r130["report_sha256"],
            "r121_status": r121["status"],
            "r121_schedule_status": r121["fixed_pd_schedule_audit"]["status"],
            "r121_report_sha256": r121["report_sha256"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "edge_lift_contract": profile["edge_lift_contract"],
        "numeric_contract": profile["numeric_contract"],
        "acceptance_contract": profile["acceptance_contract"],
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
            "ordered_projection_system_float64_sha256": (outcome.ordered_system_sha256),
            "aggregate": outcome.projection_aggregate,
            "collocations": list(outcome.projection_rows),
        },
        "edge_lift_audit": {
            "status": (
                "PASS"
                if outcome.changed_base_velocity_rows == 27
                and outcome.r132_anchor_rows_reproduced == 36
                else "FAIL"
            ),
            "exit_intervals": list(EXIT_INTERVALS),
            "changed_base_velocity_rows": outcome.changed_base_velocity_rows,
            "r132_anchor_rows_reproduced": outcome.r132_anchor_rows_reproduced,
        },
        "projected_fixed_pd_schedule_audit": schedule_audit,
        "controller_activation_event_audit": event_audit,
        "controller_activation_events": list(events),
        "resource_usage": outcome.resource_usage,
        "execution_environment": dict(execution_environment),
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r132_report_file_sha256": sha256(r132_report_path),
            "r132_profile_sha256": sha256(r132_profile_path),
            "r132_module_sha256": sha256(r132_module_path),
            "r132_tool_sha256": sha256(r132_tool_path),
            "r129_report_file_sha256": sha256(r129_report_path),
            "r129_profile_sha256": sha256(r129_profile_path),
            "projection_module_sha256": sha256(projection_module_path),
            "r129_tool_sha256": sha256(r129_tool_path),
            "r130_report_file_sha256": sha256(r130_report_path),
            "r130_profile_sha256": sha256(r130_profile_path),
            "r130_execution_module_sha256": sha256(r130_execution_module_path),
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
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "projected_schedule_execution_runs": 1,
        "state_lift_evaluations": len(outcome.projection_rows),
        "state_projection_systems": outcome.projection_systems,
        "projection_factorizations": outcome.projection_factorizations,
        "projection_solves": outcome.projection_solves,
        "controller_schedule_derivations": outcome.controller_schedule_derivations,
        **{counter: 0 for counter in ZERO_DOWNSTREAM_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def execute_exit_mode_owned_projected_schedule(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    descriptor: Mapping[str, Any],
    r132: Mapping[str, Any],
    r130: Mapping[str, Any],
) -> ProjectedScheduleOutcome:
    """Project all 3200 rows once and derive exactly one controller schedule."""

    numeric = profile["numeric_contract"]
    budget = profile["execution_budget"]
    expected = profile["expected_inventory"]
    np.random.seed(int(budget["random_seed"]))
    started = time.monotonic()
    maximum_rss = _resident_memory_bytes()
    maximum_threads = _linux_thread_count()
    digest = hashlib.sha256()
    projected_velocity = np.empty(
        (COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64
    )
    delta_velocity = np.empty((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    baseline_rows = {
        int(row["collocation"]): row
        for row in r130["projection_result"]["collocations"]
    }
    anchor_rows = {
        int(row["collocation"]): row for row in r132["conformance_result"]["rows"]
    }
    if len(baseline_rows) != COLLOCATION_COUNT or len(anchor_rows) != 36:
        raise ValueError("R133 source projection row inventory differs")

    rows: list[dict[str, Any]] = []
    projection_systems = 0
    factorizations = 0
    solves = 0
    changed_rows = 0
    anchors_reproduced = 0
    invalid_reason: str | None = None
    exit_intervals = frozenset(int(value) for value in EXIT_INTERVALS)
    for collocation in range(COLLOCATION_COUNT):
        interval, substep = divmod(collocation, SUBSTEPS_PER_INTERVAL)
        state = collocation_state(cache, interval, substep)
        source_velocity = np.asarray(state.velocity, dtype=np.float64)
        selected_velocity = (
            np.array(cache["velocity"][interval], copy=True)
            if interval in exit_intervals
            else np.array(source_velocity, copy=True)
        )
        changed = not np.array_equal(source_velocity, selected_velocity)
        changed_rows += int(changed)
        modes = contact_modes[interval]
        active = tuple(
            ordinal for ordinal in range(POINT_COUNT) if point_active(modes, ordinal)
        )
        baseline = baseline_rows[collocation]
        baseline_identity = bool(
            baseline.get("status") == "PASS"
            and baseline.get("interval") == interval
            and baseline.get("substep") == substep
            and baseline.get("contact_modes") == modes.tolist()
            and baseline.get("active_point_ordinals") == list(active)
            and baseline.get("original_velocity_sha256")
            == _array_sha256(source_velocity)
        )
        digest.update(np.ascontiguousarray(modes).tobytes())
        digest.update(np.ascontiguousarray(selected_velocity).tobytes())
        if not active:
            projected = np.array(selected_velocity, copy=True)
            delta = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
            row = _flight_projection_row(
                collocation=collocation,
                interval=interval,
                substep=substep,
                modes=modes,
                velocity=selected_velocity,
            )
        else:
            matrix, kinematics = mass_matrix(model, state.configuration)
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
            digest.update(np.ascontiguousarray(matrix).tobytes())
            digest.update(np.ascontiguousarray(jacobian).tobytes())
            analysis = project_tangent_velocity(
                mass=matrix,
                jacobian=jacobian,
                velocity=selected_velocity,
                expected_rank=3 if len(active) == 1 else 5,
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
                        original_velocity=selected_velocity,
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
            row = _active_projection_row(
                collocation=collocation,
                interval=interval,
                substep=substep,
                modes=modes,
                active=active,
                analysis=analysis,
                jacobian=jacobian,
                original_velocity=selected_velocity,
                projected_velocity=projected,
                flat_audit=flat_audit,
                passed=passed,
            )
        anchor = anchor_rows.get(collocation)
        anchor_identity = bool(
            anchor is not None
            and anchor["selected_base_velocity_sha256"]
            == _array_sha256(selected_velocity)
            and anchor["selected_projected_velocity_sha256"] == _array_sha256(projected)
            and anchor["contact_modes"] == modes.tolist()
            and anchor["active_point_ordinals"] == list(active)
        )
        if anchor_identity:
            anchors_reproduced += 1
        row.update(
            {
                "source_affine_velocity_sha256": _array_sha256(source_velocity),
                "selected_base_velocity_sha256": _array_sha256(selected_velocity),
                "base_velocity_changed": changed,
                "edge_lift": (
                    "exit_mode_owned_left_velocity_trace_hold"
                    if interval in exit_intervals
                    else "ordinary_affine_velocity"
                ),
                "r130_source_row_identity": baseline_identity,
                "r132_anchor_row": anchor is not None,
                "r132_anchor_identity": anchor_identity if anchor is not None else None,
            }
        )
        if not baseline_identity:
            row["status"] = "FAIL"
            invalid_reason = "R130_SOURCE_ROW_IDENTITY_DIFFERS"
        elif anchor is not None and not anchor_identity:
            row["status"] = "FAIL"
            invalid_reason = "R132_ANCHOR_REPRODUCTION_DIFFERS"
        elif row["status"] != "PASS":
            invalid_reason = "PROJECTION_CONFORMANCE_GUARD_FAILED"
        rows.append(row)
        if invalid_reason is not None:
            break
        projected_velocity[collocation] = projected
        delta_velocity[collocation] = delta
        if collocation % 64 == 63:
            resource_passed, maximum_rss, maximum_threads, reason = (
                _check_resource_budget(
                    budget=budget,
                    started=started,
                    maximum_rss=maximum_rss,
                    maximum_threads=maximum_threads,
                )
            )
            if not resource_passed:
                invalid_reason = reason
                break

    if len(rows) == COLLOCATION_COUNT and (
        projection_systems != int(expected["projection_systems"])
        or factorizations != int(expected["projection_factorizations"])
        or solves != int(expected["projection_solves"])
        or changed_rows != int(expected["changed_base_velocity_rows"])
        or anchors_reproduced != int(expected["r132_anchor_rows"])
    ):
        invalid_reason = invalid_reason or "R133_COUNTER_CLOSURE_DIFFERS"
    elif len(rows) != COLLOCATION_COUNT and invalid_reason is None:
        invalid_reason = "R133_PROJECTION_SCHEDULE_CLOSURE_DIFFERS"

    eventful: EventfulFixedPdSchedule | None = None
    controller_derivations = 0
    if invalid_reason is None:
        eventful = derive_eventful_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=descriptor,
            projected_velocity=projected_velocity,
        )
        controller_derivations = 1
        if not _schedule_acceptance_passes(profile, eventful):
            invalid_reason = "PROJECTED_FIXED_PD_SCHEDULE_FAILED"

    resource_passed, maximum_rss, maximum_threads, reason = _check_resource_budget(
        budget=budget,
        started=started,
        maximum_rss=maximum_rss,
        maximum_threads=maximum_threads,
    )
    if not resource_passed:
        invalid_reason = invalid_reason or reason
    complete_projection = len(rows) == COLLOCATION_COUNT
    aggregate = aggregate_projection_rows(
        rows,
        projected_velocity=projected_velocity if complete_projection else None,
        delta_velocity=delta_velocity if complete_projection else None,
    )
    elapsed = time.monotonic() - started
    return ProjectedScheduleOutcome(
        status="PASS" if invalid_reason is None else "FAIL",
        invalid_reason=invalid_reason,
        projection_rows=tuple(rows),
        projection_aggregate=aggregate,
        schedule=eventful,
        projection_systems=projection_systems,
        projection_factorizations=factorizations,
        projection_solves=solves,
        controller_schedule_derivations=controller_derivations,
        changed_base_velocity_rows=changed_rows,
        r132_anchor_rows_reproduced=anchors_reproduced,
        ordered_system_sha256=digest.hexdigest(),
        resource_usage={
            "status": "PASS" if resource_passed else "FAIL",
            "invalid_reason": None if resource_passed else reason,
            "wall_clock_seconds": elapsed,
            "maximum_resident_memory_bytes": maximum_rss,
            "maximum_thread_count": maximum_threads,
            "process_count": 1,
            "random_seed": int(budget["random_seed"]),
            "randomized_restart_count": 0,
            "manual_intervention": "FORBIDDEN_AND_NOT_USED",
        },
    )


def derive_eventful_projected_fixed_pd_schedule(
    *,
    cache: Mapping[str, NDArray[Any]],
    descriptor: Mapping[str, Any],
    projected_velocity: NDArray[np.float64],
) -> EventfulFixedPdSchedule:
    """Reproduce R130 fixed PD while recording every activation address."""

    if projected_velocity.shape != (COLLOCATION_COUNT, GENERALIZED_WIDTH) or not np.all(
        np.isfinite(projected_velocity)
    ):
        raise ValueError("R133 projected fixed-PD velocity differs")
    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    if [int(row["dof_ordinal"]) for row in joints] != list(range(ACTUATOR_COUNT)) or [
        int(row["dof_ordinal"]) for row in actuators
    ] != list(range(ACTUATOR_COUNT)):
        raise ValueError("R133 projected fixed-PD descriptor order differs")
    position = np.asarray(cache["joint_position_rad"], dtype=np.float64)
    if position.shape != (FRAME_COUNT, ACTUATOR_COUNT) or not np.all(
        np.isfinite(position)
    ):
        raise ValueError("R133 projected fixed-PD position differs")
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
    counts = {name: 0 for name in CONTROLLER_CATEGORIES}
    channels: dict[str, set[int]] = {name: set() for name in CONTROLLER_CATEGORIES}
    events: list[dict[str, Any]] = []
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
        _record_controller_events(
            mask=target != position_urad[interval],
            category="target_soft_clamp",
            counts=counts,
            channels=channels,
            events=events,
            interval=interval,
            substep=0,
            stage="target_soft_limit",
            details={
                "source_target_microradians": position_urad[interval],
                "selected_target_microradians": target,
                "lower_bound_microradians": soft_min,
                "upper_bound_microradians": soft_max,
            },
        )
        next_target = np.minimum(
            np.maximum(target, applied_target - target_delta),
            applied_target + target_delta,
        )
        _record_controller_events(
            mask=next_target != target,
            category="target_slew",
            counts=counts,
            channels=channels,
            events=events,
            interval=interval,
            substep=0,
            stage="target_slew_limit",
            details={
                "requested_target_microradians": target,
                "selected_target_microradians": next_target,
                "previous_target_microradians": applied_target,
                "maximum_delta_microradians": target_delta,
            },
        )
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
            _record_controller_events(
                mask=hard_excess > 10.0,
                category="hard_rom_violation",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="observed_hard_rom",
                details={
                    "observed_position_microradians": observed_position,
                    "lower_bound_microradians": hard_min,
                    "upper_bound_microradians": hard_max,
                    "excess_microradians": hard_excess,
                },
            )
            _record_controller_events(
                mask=velocity_excess > 0.0,
                category="velocity_violation",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="observed_joint_velocity",
                details={
                    "observed_velocity_microradians_per_second": observed_velocity,
                    "maximum_velocity_microradians_per_second": maximum_velocity,
                    "excess_microradians_per_second": velocity_excess,
                },
            )
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
            common_effort = {
                "requested_effort_micronewton_metres": requested,
                "observed_velocity_microradians_per_second": observed_velocity,
            }
            _record_controller_events(
                mask=(requested < effort_min) | (requested > effort_max),
                category="static_effort_clamp",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="static_effort_limit",
                details={
                    **common_effort,
                    "lower_bound_micronewton_metres": effort_min,
                    "upper_bound_micronewton_metres": effort_max,
                },
            )
            _record_controller_events(
                mask=(requested < rate_min) | (requested > rate_max),
                category="effort_rate_clamp",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="effort_rate_limit",
                details={
                    **common_effort,
                    "lower_bound_micronewton_metres": rate_min,
                    "upper_bound_micronewton_metres": rate_max,
                },
            )
            _record_controller_events(
                mask=(requested < -power_bound) | (requested > power_bound),
                category="power_clamp",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="power_limit",
                details={
                    **common_effort,
                    "absolute_effort_bound_micronewton_metres": power_bound,
                },
            )
            work_mask = ((observed_velocity > 0.0) & (requested > work_bound)) | (
                (observed_velocity < 0.0) & (requested < -work_bound)
            )
            _record_controller_events(
                mask=work_mask,
                category="positive_work_clamp",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="positive_work_limit",
                details={
                    **common_effort,
                    "absolute_effort_bound_micronewton_metres": work_bound,
                    "remaining_work_microjoules": remaining_work,
                },
            )
            infeasible = (remaining_work < 0.0) | (lower > upper)
            _record_controller_events(
                mask=infeasible,
                category="infeasible_effort_envelope",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="pre_clip_envelope",
                details={
                    "lower_bound_micronewton_metres": lower,
                    "upper_bound_micronewton_metres": upper,
                    "remaining_work_microjoules": remaining_work,
                },
            )
            current_effort = np.minimum(np.maximum(requested, lower), upper)
            charge = np.ceil(
                np.maximum(current_effort * observed_velocity, 0.0)
                / (240.0 * 1_000_000.0)
            )
            used_work += charge
            _record_controller_events(
                mask=used_work > maximum_work,
                category="infeasible_effort_envelope",
                counts=counts,
                channels=channels,
                events=events,
                interval=interval,
                substep=substep,
                stage="post_charge_work",
                details={
                    "used_work_microjoules": used_work,
                    "maximum_work_microjoules": maximum_work,
                },
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
    event_audit = audit_controller_events(events=events, counts=counts)
    return EventfulFixedPdSchedule(
        schedule=FixedPdSchedule(target_output, effort_output, audit),
        events=tuple(events),
        event_audit=event_audit,
    )


def audit_controller_events(
    *, events: Sequence[Mapping[str, Any]], counts: Mapping[str, int]
) -> dict[str, Any]:
    observed = Counter(str(row.get("category")) for row in events)
    count_closure = all(observed[name] == int(counts[name]) for name in counts)
    addressed = all(
        type(row.get("collocation")) is int
        and 0 <= int(row["collocation"]) < COLLOCATION_COUNT
        and type(row.get("interval")) is int
        and int(row["interval"]) == int(row["collocation"]) // SUBSTEPS_PER_INTERVAL
        and type(row.get("substep")) is int
        and int(row["substep"]) == int(row["collocation"]) % SUBSTEPS_PER_INTERVAL
        and type(row.get("dof_ordinal")) is int
        and 0 <= int(row["dof_ordinal"]) < ACTUATOR_COUNT
        and row.get("category") in CONTROLLER_CATEGORIES
        and isinstance(row.get("stage"), str)
        and bool(row["stage"])
        for row in events
    )
    ordered = all(
        int(left["collocation"]) <= int(right["collocation"])
        for left, right in pairwise(events)
    )
    return {
        "status": "PASS" if count_closure and addressed and ordered else "FAIL",
        "event_count": len(events),
        "category_counts": {name: observed[name] for name in CONTROLLER_CATEGORIES},
        "activation_count_closure": count_closure,
        "all_events_row_addressed": addressed,
        "execution_order_preserved": ordered,
        "ordered_event_canonical_sha256": hashlib.sha256(
            canonical_json(list(events))
        ).hexdigest(),
    }


def _record_controller_events(
    *,
    mask: NDArray[np.bool_],
    category: str,
    counts: dict[str, int],
    channels: dict[str, set[int]],
    events: list[dict[str, Any]],
    interval: int,
    substep: int,
    stage: str,
    details: Mapping[str, NDArray[Any]],
) -> None:
    if mask.shape != (ACTUATOR_COUNT,) or mask.dtype != np.bool_:
        raise ValueError("R133 controller activation mask differs")
    ordinals = np.flatnonzero(mask)
    counts[category] += int(ordinals.size)
    channels[category].update(int(value) for value in ordinals)
    collocation = interval * SUBSTEPS_PER_INTERVAL + substep
    for ordinal_value in ordinals:
        ordinal = int(ordinal_value)
        event: dict[str, Any] = {
            "category": category,
            "stage": stage,
            "collocation": collocation,
            "interval": interval,
            "substep": substep,
            "dof_ordinal": ordinal,
        }
        for name, values in details.items():
            if values.shape != (ACTUATOR_COUNT,):
                raise ValueError("R133 controller event detail width differs")
            value = float(values[ordinal])
            if not np.isfinite(value):
                raise ValueError("R133 controller event detail is nonfinite")
            event[name] = value
        events.append(event)


def _schedule_acceptance_passes(
    profile: Mapping[str, Any], eventful: EventfulFixedPdSchedule
) -> bool:
    audit = eventful.schedule.audit
    counts = audit["activation_counts"]
    channels = audit["activated_dof_ordinals"]
    maxima = audit["maxima"]
    expected = profile["controller_lineage_contract"]
    acceptance = profile["acceptance_contract"]
    return bool(
        audit["status"] == "PASS"
        and audit["collocation_count"] == COLLOCATION_COUNT
        and counts["hard_rom_violation"] == 0
        and counts["velocity_violation"] == 0
        and counts["infeasible_effort_envelope"] == 0
        and counts["target_soft_clamp"] == expected["target_soft_clamp_count"]
        and channels["target_soft_clamp"] == expected["target_soft_clamp_dof_ordinals"]
        and counts["target_slew"] == expected["target_slew_count"]
        and channels["target_slew"] == expected["target_slew_dof_ordinals"]
        and maxima["target_lag_microradians"]
        == expected["maximum_target_lag_microradians"]
        and audit["applied_target_float64_sha256"]
        == acceptance["required_applied_target_float64_sha256"]
        and eventful.event_audit["status"] == "PASS"
    )


def _array_sha256(value: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r132_profile_path: Path,
    r132_module_path: Path,
    r132_tool_path: Path,
    r129_profile_path: Path,
    projection_module_path: Path,
    r129_tool_path: Path,
    r130_profile_path: Path,
    r130_execution_module_path: Path,
    r121_profile_path: Path,
    r113_profile_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    collocation_lift_module_path: Path,
    dynamics_kernel_path: Path,
) -> None:
    source = profile["source"]
    checks = (
        (r132_profile_path, source["r132"]["profile_sha256"]),
        (r132_module_path, source["r132"]["module_sha256"]),
        (r132_tool_path, source["r132"]["tool_sha256"]),
        (r129_profile_path, source["r129"]["profile_sha256"]),
        (projection_module_path, source["r129"]["projection_module_sha256"]),
        (r129_tool_path, source["r129"]["tool_sha256"]),
        (r130_profile_path, source["r130"]["profile_sha256"]),
        (
            r130_execution_module_path,
            source["r130"]["execution_module_sha256"],
        ),
        (r121_profile_path, source["r121"]["profile_sha256"]),
        (r113_profile_path, source["r113"]["profile_sha256"]),
        (r120_profile_path, source["r120"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (v9_complete_clip_path, source["v9_complete_clip_sha256"]),
        (
            collocation_lift_module_path,
            source["collocation_lift_module_sha256"],
        ),
        (dynamics_kernel_path, source["dynamics_kernel_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in checks):
        raise ValueError("R133 source file identity differs")


def _validate_source_contracts(
    *,
    profile: Mapping[str, Any],
    r132: Mapping[str, Any],
    r129: Mapping[str, Any],
    r130: Mapping[str, Any],
    r121: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    target_hash = profile["acceptance_contract"][
        "required_applied_target_float64_sha256"
    ]
    if (
        r132.get("status") != "PASS"
        or r132.get("gate_decision")
        != "PERMIT_ONE_BOUNDED_R133_EXIT_MODE_OWNED_PROJECTED_SCHEDULE_EXECUTION"
        or r132.get("result_transition") != "R132_PASS_R133_PROJECTED_SCHEDULE_ONLY"
        or r129.get("status") != "PASS"
        or r130.get("status") != "INVALID"
        or r130.get("projection_result", {}).get("status") != "PASS"
        or r130.get("projected_fixed_pd_schedule_audit", {}).get(
            "applied_target_float64_sha256"
        )
        != target_hash
        or r121.get("status") != "COMPLETE"
        or r121.get("fixed_pd_schedule_audit", {}).get("status") != "PASS"
        or r113.get("status") != "PASS"
        or r120.get("status") != "PASS"
    ):
        raise ValueError("R133 source gate differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R133 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R133 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    expected = profile.get("expected_inventory", {})
    acceptance = profile.get("acceptance_contract", {})
    numeric = profile.get("numeric_contract", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleReportOnlyExecution"
        or profile.get("claim") != "ExitModeOwnedFullProjectedFixedPdScheduleOnly"
        or scope.get("run_id") != "R133"
        or scope.get("collocation_count") != COLLOCATION_COUNT
        or scope.get("maximum_projection_systems") != 2640
        or scope.get("maximum_controller_schedule_derivations") != 1
        or scope.get("inverse_dynamics_system_assemblies") != 0
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or expected.get("flight_collocations") != 560
        or expected.get("single_point_collocations") != 324
        or expected.get("flat_foot_collocations") != 2316
        or expected.get("projection_systems") != 2640
        or expected.get("projection_factorizations") != 2640
        or expected.get("projection_solves") != 2640
        or expected.get("changed_base_velocity_rows") != 27
        or expected.get("r132_anchor_rows") != 36
        or acceptance.get("required_projection_rows") != COLLOCATION_COUNT
        or acceptance.get("required_hard_rom_violation_count") != 0
        or acceptance.get("required_velocity_violation_count") != 0
        or acceptance.get("required_infeasible_effort_envelope_count") != 0
        or acceptance.get("required_controller_event_address_closure") is not True
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
        or profile.get("decision", {}).get("pass")
        != "PERMIT_SEPARATE_REPORT_ONLY_R134_PROJECTED_INVERSE_DYNAMICS_FORMULATION_ONLY"
        or profile.get("decision", {}).get("fail")
        != "STOP_AND_RESEARCH_WITHOUT_EXECUTION"
        or profile.get("result_transitions", {}).get("pass")
        != "R133_PASS_R134_FORMULATION_ONLY"
        or profile.get("result_transitions", {}).get("fail")
        != "R133_FAIL_RESEARCH_WITHOUT_EXECUTION"
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
        raise ValueError("R133 execution profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R133 execution requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], environment: Mapping[str, str]
) -> None:
    if dict(environment) != profile["single_thread_environment"]:
        raise ValueError("R133 single-thread environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R133 execution validation differs")
    return normalized


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


def _resident_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _linux_thread_count() -> int:
    task = Path("/proc/self/task")
    return len(tuple(task.iterdir())) if task.is_dir() else 1
