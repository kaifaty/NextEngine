from __future__ import annotations

import hashlib
import json
import resource
import time
from collections.abc import Iterable, Mapping, Sequence
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
    collocation_state,
    point_active,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    FRAME_COUNT,
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
from next_lab.tangent_velocity_projection_conformance import (
    _analysis_metrics,
    _projection_passes,
    project_tangent_velocity,
)

CONFORMANCE_ID = "nextengine.humanoid-exit-mode-owned-lift-conformance.v1"
CHECK_ID = "TRAIN-4-EXIT-MODE-OWNED-LIFT-CONFORMANCE"
EXIT_INTERVALS = (76, 165, 244, 337, 412, 501, 575, 676, 744)
HOTSPOT_INTERVALS = (76, 744)
MODE_FLIGHT = 0
MODE_FOREFOOT = 2
MODE_FLAT = 3
EDGE_CLASSES = frozenset({"ordinary", "entry", "active_mode_change", "exit"})
BOUNDED_ACCEPTANCE = {
    "r132_retry": "NOT_AUTHORIZED",
    "r133_projected_schedule_execution": "AUTHORIZED_ON_R132_PASS_ONLY",
    "additional_edge_lift_conformance": "NOT_AUTHORIZED",
    "additional_projection_execution": "NOT_AUTHORIZED_EXCEPT_ONE_R133_ON_PASS",
    "additional_inverse_dynamics_execution": "NOT_AUTHORIZED",
    "kinodynamic_solve": "NOT_AUTHORIZED",
    "candidate_artifact": "NOT_AUTHORIZED",
    "physx": "NOT_AUTHORIZED",
    "all_17": "NOT_AUTHORIZED",
    "full_v19": "NOT_AUTHORIZED",
    "training": "NOT_AUTHORIZED",
}
ZERO_DOWNSTREAM_COUNTERS = (
    "controller_schedule_derivations",
    "inverse_dynamics_system_assemblies",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


@dataclass(frozen=True)
class ExitLiftConformanceOutcome:
    status: str
    invalid_reason: str | None
    rows: tuple[dict[str, Any], ...]
    aggregate: dict[str, Any]
    ordered_system_sha256: str
    resource_usage: dict[str, Any]


def build_exit_mode_owned_lift_conformance_report(
    *,
    profile_path: Path,
    r131_report_path: Path,
    r131_profile_path: Path,
    r131_module_path: Path,
    r131_tool_path: Path,
    r129_report_path: Path,
    r129_profile_path: Path,
    projection_module_path: Path,
    r129_tool_path: Path,
    r130_report_path: Path,
    r130_profile_path: Path,
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
    """Consume the sole bounded R132 authority and emit report-only evidence."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r131_report_path,
            r131_profile_path,
            r131_module_path,
            r131_tool_path,
            r129_report_path,
            r129_profile_path,
            projection_module_path,
            r129_tool_path,
            r130_report_path,
            r130_profile_path,
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
        r131_report_path,
        r131_profile_path,
        r131_module_path,
        r131_tool_path,
        r129_report_path,
        r129_profile_path,
        projection_module_path,
        r129_tool_path,
        r130_report_path,
        r130_profile_path,
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
        raise FileNotFoundError("R132 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    source = profile["source"]
    r131 = _load_bound_report(r131_report_path, source["r131"], "R131")
    r129 = _load_bound_report(r129_report_path, source["r129"], "R129")
    r130 = _load_bound_report(r130_report_path, source["r130"], "R130")
    r113 = _load_bound_report(r113_report_path, source["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, source["r120"], "R120")
    _validate_source_files(
        profile=profile,
        r131_profile_path=r131_profile_path,
        r131_module_path=r131_module_path,
        r131_tool_path=r131_tool_path,
        r129_profile_path=r129_profile_path,
        projection_module_path=projection_module_path,
        r129_tool_path=r129_tool_path,
        r130_profile_path=r130_profile_path,
        r113_profile_path=r113_profile_path,
        r120_profile_path=r120_profile_path,
        r120_cache_path=r120_cache_path,
        v9_complete_clip_path=v9_complete_clip_path,
        collocation_lift_module_path=collocation_lift_module_path,
        dynamics_kernel_path=dynamics_kernel_path,
    )
    _validate_source_contracts(
        profile=profile,
        r131=r131,
        r129=r129,
        r130=r130,
        r113=r113,
        r120=r120,
    )
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != source["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve()) != source["conformance_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R132 current implementation identity differs")

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R132 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=r113)

    outcome = conform_exit_mode_owned_lift(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        descriptor=descriptor,
        r130=r130,
    )
    passed = outcome.status == "PASS"
    decision_key = "pass" if passed else "fail"
    target_hash = r130["projected_fixed_pd_schedule_audit"][
        "applied_target_float64_sha256"
    ]
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": outcome.status,
        "claim": profile["claim"],
        "gate_decision": profile["decision"][decision_key],
        "result_transition": profile["result_transitions"][decision_key],
        "invalid_reason": outcome.invalid_reason,
        "scope": profile["scope"],
        "source_gates": {
            "r131_status": r131["status"],
            "r131_gate_decision": r131["gate_decision"],
            "r131_report_sha256": r131["report_sha256"],
            "r129_status": r129["status"],
            "r129_report_sha256": r129["report_sha256"],
            "r130_status": r130["status"],
            "r130_projection_status": r130["projection_result"]["status"],
            "r130_report_sha256": r130["report_sha256"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "edge_lift_contract": profile["edge_lift_contract"],
        "numeric_contract": profile["numeric_contract"],
        "acceptance_contract": profile["acceptance_contract"],
        "synthetic_edge_classification_audit": audit_synthetic_edge_cases(),
        "target_lineage_audit": {
            "status": "PASS",
            "source": "R130 position-only target schedule",
            "applied_target_float64_sha256": target_hash,
            "expected_applied_target_float64_sha256": profile[
                "target_lineage_contract"
            ]["applied_target_float64_sha256"],
            "target_values_materialized_by_r132": False,
            "controller_schedule_derivations": 0,
            "preservation_basis": (
                "R132 changes only the selected generalized velocity before "
                "projection; it does not mutate configuration, target, gains or limits"
            ),
        },
        "conformance_result": {
            "status": outcome.status,
            "invalid_reason": outcome.invalid_reason,
            "ordered_projection_system_float64_sha256": (outcome.ordered_system_sha256),
            "aggregate": outcome.aggregate,
            "rows": list(outcome.rows),
        },
        "diagnostic_interpretation_contract": profile[
            "diagnostic_interpretation_contract"
        ],
        "resource_usage": outcome.resource_usage,
        "execution_environment": dict(execution_environment),
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r131_report_file_sha256": sha256(r131_report_path),
            "r131_profile_sha256": sha256(r131_profile_path),
            "r131_module_sha256": sha256(r131_module_path),
            "r131_tool_sha256": sha256(r131_tool_path),
            "r129_report_file_sha256": sha256(r129_report_path),
            "r129_profile_sha256": sha256(r129_profile_path),
            "projection_module_sha256": sha256(projection_module_path),
            "r129_tool_sha256": sha256(r129_tool_path),
            "r130_report_file_sha256": sha256(r130_report_path),
            "r130_profile_sha256": sha256(r130_profile_path),
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
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "real_anchor_rows": len(outcome.rows),
        "state_lift_evaluations": len(outcome.rows),
        "mass_matrix_assemblies": len(outcome.rows),
        "contact_jacobian_assemblies": len(outcome.rows),
        "projection_factorizations": len(outcome.rows),
        "projection_solves": len(outcome.rows),
        "full_schedule_projections": 0,
        **{counter: 0 for counter in ZERO_DOWNSTREAM_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def conform_exit_mode_owned_lift(
    *,
    profile: Mapping[str, Any],
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    descriptor: Mapping[str, Any],
    r130: Mapping[str, Any],
) -> ExitLiftConformanceOutcome:
    """Evaluate exactly 36 exit rows; never derive a controller schedule."""

    numeric = profile["numeric_contract"]
    budget = profile["resource_budget"]
    expected_intervals = tuple(
        int(value) for value in profile["edge_lift_contract"]["exit_intervals"]
    )
    started = time.monotonic()
    maximum_rss = _resident_memory_bytes()
    maximum_threads = _linux_thread_count()
    digest = hashlib.sha256()
    baseline_rows = {
        int(row["collocation"]): row
        for row in r130["projection_result"]["collocations"]
    }
    if len(baseline_rows) != 3200:
        raise ValueError("R132 R130 projection row inventory differs")
    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    if [int(row["dof_ordinal"]) for row in joints] != list(range(ACTUATOR_COUNT)):
        raise ValueError("R132 descriptor joint order differs")
    limits = np.asarray(
        [row["maximum_velocity_microradians_per_second"] for row in joints],
        dtype=np.int64,
    )
    joint_ids = [str(row["joint_id"]) for row in joints]

    rows: list[dict[str, Any]] = []
    for interval in expected_intervals:
        current_modes = contact_modes[interval]
        next_modes = contact_modes[interval + 1]
        edge_class = classify_contact_edge(current_modes, next_modes)
        active = tuple(
            ordinal
            for ordinal in range(POINT_COUNT)
            if point_active(current_modes, ordinal)
        )
        for substep in range(SUBSTEPS_PER_INTERVAL):
            collocation = interval * SUBSTEPS_PER_INTERVAL + substep
            state = collocation_state(cache, interval, substep)
            source_velocity = np.asarray(state.velocity, dtype=np.float64)
            selected_velocity = select_edge_owned_velocity(
                left=np.asarray(cache["velocity"][interval], dtype=np.float64),
                right=np.asarray(cache["velocity"][interval + 1], dtype=np.float64),
                substep=substep,
                edge_class=edge_class,
            )
            configuration_hash = _configuration_sha256(state.configuration)
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
            digest.update(np.ascontiguousarray(current_modes).tobytes())
            digest.update(np.ascontiguousarray(next_modes).tobytes())
            digest.update(np.ascontiguousarray(selected_velocity).tobytes())
            digest.update(np.ascontiguousarray(matrix).tobytes())
            digest.update(np.ascontiguousarray(jacobian).tobytes())
            analysis = project_tangent_velocity(
                mass=matrix,
                jacobian=jacobian,
                velocity=selected_velocity,
                expected_rank=3,
                numeric=numeric,
            )
            projected = analysis.projected_velocity
            projection_passed = _projection_passes(
                analysis=analysis,
                numeric=numeric,
                flat_audit=None,
            )
            baseline = baseline_rows[collocation]
            selected_is_left = np.array_equal(
                selected_velocity, cache["velocity"][interval]
            )
            source_is_left_at_zero = substep != 0 or np.array_equal(
                source_velocity, cache["velocity"][interval]
            )
            baseline_identity = bool(
                baseline.get("status") == "PASS"
                and baseline.get("interval") == interval
                and baseline.get("substep") == substep
                and baseline.get("contact_modes") == current_modes.tolist()
                and baseline.get("active_point_ordinals") == list(active)
                and baseline.get("original_velocity_sha256")
                == _array_sha256(source_velocity)
            )
            passed = bool(
                edge_class == "exit"
                and len(active) == 1
                and selected_is_left
                and source_is_left_at_zero
                and baseline_identity
                and projection_passed
                and analysis.rank == 3
                and analysis.nullity == 0
            )
            row = _conformance_row(
                status="PASS" if passed else "FAIL",
                interval=interval,
                substep=substep,
                current_modes=current_modes,
                next_modes=next_modes,
                active=active,
                configuration_hash=configuration_hash,
                source_velocity=source_velocity,
                selected_velocity=selected_velocity,
                projected_velocity=projected,
                analysis=analysis,
                jacobian=jacobian,
                baseline=baseline,
                limits=limits,
                joint_ids=joint_ids,
                edge_class=edge_class,
                selected_is_left=selected_is_left,
                baseline_identity=baseline_identity,
            )
            rows.append(row)
            maximum_rss = max(maximum_rss, _resident_memory_bytes())
            maximum_threads = max(maximum_threads, _linux_thread_count())

    elapsed = time.monotonic() - started
    aggregate = aggregate_exit_lift_rows(rows, expected_intervals=expected_intervals)
    resource_status, resource_reason = _resource_status(
        budget=budget,
        elapsed=elapsed,
        maximum_rss=maximum_rss,
        maximum_threads=maximum_threads,
    )
    invalid_reason = next(
        (
            f"ROW_{row['collocation']}_CONFORMANCE_FAILED"
            for row in rows
            if row["status"] != "PASS"
        ),
        None,
    )
    if aggregate["closure_status"] != "PASS":
        invalid_reason = invalid_reason or "R132_ROW_CLOSURE_DIFFERS"
    if resource_status != "PASS":
        invalid_reason = invalid_reason or resource_reason
    status = "PASS" if invalid_reason is None else "FAIL"
    aggregate["status"] = status
    return ExitLiftConformanceOutcome(
        status=status,
        invalid_reason=invalid_reason,
        rows=tuple(rows),
        aggregate=aggregate,
        ordered_system_sha256=digest.hexdigest(),
        resource_usage={
            "status": resource_status,
            "invalid_reason": resource_reason,
            "wall_clock_seconds": elapsed,
            "maximum_resident_memory_bytes": maximum_rss,
            "maximum_thread_count": maximum_threads,
            "process_count": 1,
        },
    )


def classify_contact_edge(
    current_modes: Sequence[int], next_modes: Sequence[int]
) -> str:
    current = _validated_modes(current_modes)
    following = _validated_modes(next_modes)
    side_classes: list[str] = []
    for before, after in zip(current, following, strict=True):
        if before == after:
            continue
        if before == MODE_FLIGHT:
            side_classes.append("entry")
        elif after == MODE_FLIGHT:
            side_classes.append("exit")
        else:
            side_classes.append("active_mode_change")
    if not side_classes:
        return "ordinary"
    unique = set(side_classes)
    if len(unique) != 1:
        raise ValueError("R132 simultaneous mixed contact edge differs")
    return side_classes[0]


def select_edge_owned_velocity(
    *,
    left: NDArray[np.float64],
    right: NDArray[np.float64],
    substep: int,
    edge_class: str,
) -> NDArray[np.float64]:
    if (
        left.shape != right.shape
        or left.ndim != 1
        or left.dtype != np.float64
        or right.dtype != np.float64
        or not np.all(np.isfinite(left))
        or not np.all(np.isfinite(right))
    ):
        raise ValueError("R132 velocity endpoints differ")
    if type(substep) is not int or not 0 <= substep < SUBSTEPS_PER_INTERVAL:
        raise ValueError("R132 velocity-lift substep differs")
    if edge_class not in EDGE_CLASSES:
        raise ValueError("R132 velocity-lift edge class differs")
    if edge_class == "exit":
        return np.array(left, copy=True)
    fraction = substep / SUBSTEPS_PER_INTERVAL
    return (1.0 - fraction) * left + fraction * right


def audit_synthetic_edge_cases() -> dict[str, Any]:
    cases = (
        ("ordinary", (0, 0), (0, 0), "ordinary"),
        ("entry", (0, 0), (0, 2), "entry"),
        ("active_mode_change", (0, 3), (0, 2), "active_mode_change"),
        ("exit", (2, 0), (0, 0), "exit"),
    )
    rows = []
    for name, current, following, expected in cases:
        actual = classify_contact_edge(current, following)
        rows.append({"case": name, "status": "PASS" if actual == expected else "FAIL"})
    try:
        classify_contact_edge((1, 0), (0, 0))
        malformed_rejected = False
    except ValueError:
        malformed_rejected = True
    rows.append(
        {
            "case": "malformed_edge",
            "status": "PASS" if malformed_rejected else "FAIL",
        }
    )
    return {
        "status": "PASS" if all(row["status"] == "PASS" for row in rows) else "FAIL",
        "cases": rows,
        "state_projection_systems": 0,
    }


def aggregate_exit_lift_rows(
    rows: Sequence[Mapping[str, Any]], *, expected_intervals: Sequence[int]
) -> dict[str, Any]:
    expected_collocations = [
        interval * SUBSTEPS_PER_INTERVAL + substep
        for interval in expected_intervals
        for substep in range(SUBSTEPS_PER_INTERVAL)
    ]
    actual_collocations = [int(row["collocation"]) for row in rows]
    changed = [row for row in rows if row["base_velocity_changed"]]
    substep_zero = [row for row in rows if row["substep"] == 0]
    projection_values = [row["projection"] for row in rows]
    baseline_corrections = [
        float(row["r130_affine_baseline"]["maximum_generalized_velocity_correction"])
        for row in rows
    ]
    selected_corrections = [
        float(row["projection"]["maximum_generalized_velocity_correction"])
        for row in rows
        if row["projection"]["maximum_generalized_velocity_correction"] is not None
    ]
    improved = [
        row
        for row in rows
        if row["diagnostic_comparison"]["correction_delta_from_r130"] is not None
        and row["diagnostic_comparison"]["correction_delta_from_r130"] < 0.0
    ]
    violations = [
        row
        for row in rows
        if row["joint_velocity_diagnostic"]["projected_violation_count"] > 0
    ]
    closure = bool(
        actual_collocations == expected_collocations
        and len(rows) == 36
        and len(changed) <= 27
        and len(substep_zero) == 9
        and all(not row["base_velocity_changed"] for row in substep_zero)
        and all(row["status"] == "PASS" for row in rows)
    )
    return {
        "closure_status": "PASS" if closure else "FAIL",
        "row_count": len(rows),
        "passing_row_count": sum(row["status"] == "PASS" for row in rows),
        "exit_interval_count": len({int(row["interval"]) for row in rows}),
        "changed_base_velocity_row_count": len(changed),
        "unchanged_substep_zero_row_count": sum(
            not row["base_velocity_changed"] for row in substep_zero
        ),
        "rank_three_row_count": sum(row["projection"]["rank"] == 3 for row in rows),
        "maximum_active_velocity_after_metres_per_second": _optional_maximum(
            value["maximum_active_velocity_after_metres_per_second"]
            for value in projection_values
        ),
        "maximum_scaled_kkt_residual": _optional_maximum(
            value["scaled_kkt_residual"] for value in projection_values
        ),
        "maximum_projection_idempotence_error": _optional_maximum(
            value["projection_idempotence_error"] for value in projection_values
        ),
        "maximum_selected_generalized_velocity_correction": _optional_maximum(
            selected_corrections
        ),
        "maximum_r130_affine_generalized_velocity_correction": max(
            baseline_corrections
        ),
        "rows_with_strictly_lower_correction_than_r130": len(improved),
        "rows_with_projected_joint_velocity_violation": len(violations),
        "projected_joint_velocity_violation_collocations": [
            int(row["collocation"]) for row in violations
        ],
        "maximum_projected_joint_velocity_excess_microradians_per_second": max(
            int(
                row["joint_velocity_diagnostic"][
                    "maximum_projected_excess_microradians_per_second"
                ]
            )
            for row in rows
        ),
        "diagnostic_values_are_acceptance_gates": False,
    }


def _conformance_row(
    *,
    status: str,
    interval: int,
    substep: int,
    current_modes: NDArray[np.uint8],
    next_modes: NDArray[np.uint8],
    active: tuple[int, ...],
    configuration_hash: str,
    source_velocity: NDArray[np.float64],
    selected_velocity: NDArray[np.float64],
    projected_velocity: NDArray[np.float64] | None,
    analysis: Any,
    jacobian: NDArray[np.float64],
    baseline: Mapping[str, Any],
    limits: NDArray[np.int64],
    joint_ids: Sequence[str],
    edge_class: str,
    selected_is_left: bool,
    baseline_identity: bool,
) -> dict[str, Any]:
    collocation = interval * SUBSTEPS_PER_INTERVAL + substep
    source_joint = _quantized_joint_velocity(source_velocity)
    selected_joint = _quantized_joint_velocity(selected_velocity)
    projected_joint = (
        None
        if projected_velocity is None
        else _quantized_joint_velocity(projected_velocity)
    )
    correction_joint = (
        None
        if projected_velocity is None
        else np.rint(
            (projected_velocity[6:] - selected_velocity[6:]) * 1_000_000.0
        ).astype(np.int64)
    )
    violation_mask = (
        np.zeros(ACTUATOR_COUNT, dtype=np.bool_)
        if projected_joint is None
        else np.abs(projected_joint) > limits
    )
    excess = (
        np.zeros(ACTUATOR_COUNT, dtype=np.int64)
        if projected_joint is None
        else np.maximum(np.abs(projected_joint) - limits, 0)
    )
    metrics = _analysis_metrics(analysis, jacobian, selected_velocity)
    baseline_correction = float(
        baseline["projection"]["maximum_generalized_velocity_correction"]
    )
    selected_correction = metrics["maximum_generalized_velocity_correction"]
    return {
        "status": status,
        "collocation": collocation,
        "interval": interval,
        "substep": substep,
        "edge_class": edge_class,
        "contact_modes": current_modes.tolist(),
        "next_contact_modes": next_modes.tolist(),
        "active_point_ordinals": list(active),
        "configuration_identity": {
            "status": "PASS",
            "source_affine_configuration_sha256": configuration_hash,
            "selected_configuration_sha256": configuration_hash,
            "byte_identical": True,
        },
        "source_affine_velocity_sha256": _array_sha256(source_velocity),
        "selected_base_velocity_sha256": _array_sha256(selected_velocity),
        "selected_projected_velocity_sha256": (
            None if projected_velocity is None else _array_sha256(projected_velocity)
        ),
        "selected_base_is_exact_left_knot": selected_is_left,
        "base_velocity_changed": not np.array_equal(source_velocity, selected_velocity),
        "r130_affine_baseline_identity": baseline_identity,
        "joint_velocity_evidence_microradians_per_second": {
            "dof_ordinals": list(range(ACTUATOR_COUNT)),
            "source_affine": _integer_list(source_joint),
            "selected_base": _integer_list(selected_joint),
            "selected_projected": (
                None if projected_joint is None else _integer_list(projected_joint)
            ),
            "projection_correction": (
                None if correction_joint is None else _integer_list(correction_joint)
            ),
        },
        "joint_velocity_diagnostic": {
            "acceptance_gate": False,
            "projected_violation_count": int(np.count_nonzero(violation_mask)),
            "projected_violation_dof_ordinals": np.flatnonzero(violation_mask)
            .astype(int)
            .tolist(),
            "projected_violation_joint_ids": [
                joint_ids[int(ordinal)] for ordinal in np.flatnonzero(violation_mask)
            ],
            "maximum_projected_excess_microradians_per_second": int(np.max(excess)),
            "right_ankle_roll_dof_ordinal": 11,
            "right_ankle_roll_limit_microradians_per_second": int(limits[11]),
            "right_ankle_roll_source_affine_microradians_per_second": int(
                source_joint[11]
            ),
            "right_ankle_roll_selected_base_microradians_per_second": int(
                selected_joint[11]
            ),
            "right_ankle_roll_selected_projected_microradians_per_second": (
                None if projected_joint is None else int(projected_joint[11])
            ),
        },
        "projection": metrics,
        "projection_invalid_reason": analysis.invalid_reason,
        "r130_affine_baseline": {
            "original_velocity_sha256": baseline["original_velocity_sha256"],
            "projected_velocity_sha256": baseline["projected_velocity_sha256"],
            "maximum_generalized_velocity_correction": baseline_correction,
            "maximum_active_velocity_before_metres_per_second": baseline["projection"][
                "maximum_active_velocity_before_metres_per_second"
            ],
        },
        "diagnostic_comparison": {
            "acceptance_gate": False,
            "correction_delta_from_r130": (
                None
                if selected_correction is None
                else float(selected_correction) - baseline_correction
            ),
            "correction_ratio_to_r130": (
                None
                if selected_correction is None or baseline_correction == 0.0
                else float(selected_correction) / baseline_correction
            ),
            "hotspot_interval": interval in HOTSPOT_INTERVALS,
        },
    }


def _validated_modes(modes: Sequence[int]) -> tuple[int, int]:
    if len(modes) != 2:
        raise ValueError("R132 contact mode width differs")
    result = tuple(int(value) for value in modes)
    if any(value not in (MODE_FLIGHT, MODE_FOREFOOT, MODE_FLAT) for value in result):
        raise ValueError("R132 contact mode value differs")
    return result  # type: ignore[return-value]


def _quantized_joint_velocity(
    velocity: NDArray[np.float64],
) -> NDArray[np.int64]:
    return np.rint(velocity[6:] * 1_000_000.0).astype(np.int64)


def _integer_list(values: NDArray[np.int64]) -> list[int]:
    return [int(value) for value in values]


def _optional_maximum(values: Iterable[Any]) -> float | None:
    available = [float(value) for value in values if value is not None]
    return max(available) if available else None


def _configuration_sha256(configuration: Any) -> str:
    digest = hashlib.sha256()
    for value in (
        configuration.root_position,
        configuration.root_rotation,
        configuration.joint_positions,
    ):
        digest.update(np.ascontiguousarray(value, dtype=np.float64).tobytes())
    return digest.hexdigest()


def _array_sha256(value: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r131_profile_path: Path,
    r131_module_path: Path,
    r131_tool_path: Path,
    r129_profile_path: Path,
    projection_module_path: Path,
    r129_tool_path: Path,
    r130_profile_path: Path,
    r113_profile_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    collocation_lift_module_path: Path,
    dynamics_kernel_path: Path,
) -> None:
    source = profile["source"]
    checks = (
        (r131_profile_path, source["r131"]["profile_sha256"]),
        (r131_module_path, source["r131"]["module_sha256"]),
        (r131_tool_path, source["r131"]["tool_sha256"]),
        (r129_profile_path, source["r129"]["profile_sha256"]),
        (projection_module_path, source["r129"]["projection_module_sha256"]),
        (r129_tool_path, source["r129"]["tool_sha256"]),
        (r130_profile_path, source["r130"]["profile_sha256"]),
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
        raise ValueError("R132 source file identity differs")


def _validate_source_contracts(
    *,
    profile: Mapping[str, Any],
    r131: Mapping[str, Any],
    r129: Mapping[str, Any],
    r130: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    target_hash = profile["target_lineage_contract"]["applied_target_float64_sha256"]
    if (
        r131.get("status") != "COMPLETE"
        or r131.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R132_EXIT_MODE_OWNED_LIFT_CONFORMANCE_ONLY"
        or r131.get("result_transition") != "R131_COMPLETE_R132_CONFORMANCE_ONLY"
        or r129.get("status") != "PASS"
        or r129.get("gate_decision")
        != "PERMIT_R130_SINGLE_BOUNDED_TANGENT_PROJECTED_FIXED_PD_EXECUTION_ONLY"
        or r130.get("status") != "INVALID"
        or r130.get("projection_result", {}).get("status") != "PASS"
        or r130.get("projected_fixed_pd_schedule_audit", {}).get("status") != "FAIL"
        or r130.get("projected_fixed_pd_schedule_audit", {}).get(
            "applied_target_float64_sha256"
        )
        != target_hash
        or r113.get("status") != "PASS"
        or r120.get("status") != "PASS"
    ):
        raise ValueError("R132 source gate differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R132 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R132 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    edge = profile.get("edge_lift_contract", {})
    acceptance = profile.get("acceptance_contract", {})
    numeric = profile.get("numeric_contract", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnlyConformance"
        or profile.get("claim") != "ExitModeOwnedLiftImplementationConformanceOnly"
        or scope.get("run_id") != "R132"
        or scope.get("real_anchor_interval_count") != 9
        or scope.get("maximum_real_anchor_rows") != 36
        or scope.get("maximum_real_projection_systems") != 36
        or scope.get("full_schedule_projections") != 0
        or scope.get("controller_schedule_derivations") != 0
        or scope.get("inverse_dynamics_system_assemblies") != 0
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or tuple(edge.get("exit_intervals", ())) != EXIT_INTERVALS
        or tuple(edge.get("anchor_substeps", ())) != tuple(range(SUBSTEPS_PER_INTERVAL))
        or edge.get("maximum_changed_base_velocity_rows") != 27
        or acceptance.get("required_passing_rows") != 36
        or acceptance.get("required_rank") != 3
        or acceptance.get("required_multiplier_nullity") != 0
        or acceptance.get("required_active_point_count_per_row") != 1
        or acceptance.get("maximum_changed_base_velocity_rows") != 27
        or acceptance.get("required_unchanged_substep_zero_rows") != 9
        or acceptance.get("joint_velocity_diagnostic_is_gate") is not False
        or acceptance.get("r130_correction_improvement_is_gate") is not False
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
        != "PERMIT_ONE_BOUNDED_R133_EXIT_MODE_OWNED_PROJECTED_SCHEDULE_EXECUTION"
        or profile.get("decision", {}).get("fail")
        != "STOP_AND_RESEARCH_WITHOUT_EXECUTION"
        or profile.get("result_transitions", {}).get("pass")
        != "R132_PASS_R133_PROJECTED_SCHEDULE_ONLY"
        or profile.get("result_transitions", {}).get("fail")
        != "R132_FAIL_RESEARCH_WITHOUT_EXECUTION"
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
        raise ValueError("R132 conformance profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R132 conformance requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], environment: Mapping[str, str]
) -> None:
    if dict(environment) != profile["single_thread_environment"]:
        raise ValueError("R132 single-thread environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R132 conformance validation differs")
    return normalized


def _resource_status(
    *,
    budget: Mapping[str, Any],
    elapsed: float,
    maximum_rss: int,
    maximum_threads: int,
) -> tuple[str, str | None]:
    if elapsed > float(budget["maximum_wall_clock_seconds"]):
        return "FAIL", "WALL_CLOCK_BUDGET_EXHAUSTED"
    if maximum_rss > int(budget["maximum_resident_memory_bytes"]):
        return "FAIL", "RESIDENT_MEMORY_BUDGET_EXHAUSTED"
    if maximum_threads > int(budget["thread_count"]):
        return "FAIL", "THREAD_BUDGET_EXCEEDED"
    return "PASS", None


def _resident_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _linux_thread_count() -> int:
    task = Path("/proc/self/task")
    return len(tuple(task.iterdir())) if task.is_dir() else 1
