from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256

FORMULATION_ID = (
    "nextengine.humanoid-fixed-pd-inverse-dynamics-execution-formulation.v1"
)
CHECK_ID = "TRAIN-4-FIXED-PD-INVERSE-DYNAMICS-EXECUTION-FORMULATION"
FRAME_COUNT = 801
MOTOR_INTERVAL_COUNT = 800
SUBSTEPS_PER_INTERVAL = 4
COLLOCATION_COUNT = MOTOR_INTERVAL_COUNT * SUBSTEPS_PER_INTERVAL
GENERALIZED_WIDTH = 29
ACTUATOR_COUNT = 23
POINT_COUNT = 4
POINT_FORCE_WIDTH = 3
LOCAL_UNKNOWN_COUNT = GENERALIZED_WIDTH + ACTUATOR_COUNT + POINT_COUNT * 3
CACHE_KEYS = (
    "root_position_m",
    "root_orientation_delta_rad",
    "joint_position_rad",
    "velocity",
    "acceleration",
    "reference_root_rotation",
    "metadata_json_utf8",
)
ZERO_EXECUTION_COUNTERS = (
    "solver_runs",
    "local_system_solves",
    "inverse_dynamics_solves",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "solver_private_warm_start_caches",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


def build_fixed_pd_inverse_dynamics_execution_formulation(
    *,
    profile_path: Path,
    r108_report_path: Path,
    r108_profile_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    solver_import_audit: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R121 and audit its inputs without building dynamics equations."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r108_report_path,
            r108_profile_path,
            r113_report_path,
            r113_profile_path,
            r120_report_path,
            r120_profile_path,
            r120_cache_path,
            v9_complete_clip_path,
            tool_path,
        )
    )
    (
        profile_path,
        r108_report_path,
        r108_profile_path,
        r113_report_path,
        r113_profile_path,
        r120_report_path,
        r120_profile_path,
        r120_cache_path,
        v9_complete_clip_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R121 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_solver_import_audit(solver_import_audit)
    r108 = _validate_r108(
        profile=profile,
        report_path=r108_report_path,
        profile_path=r108_profile_path,
    )
    r113 = _validate_r113(
        profile=profile,
        report_path=r113_report_path,
        profile_path=r113_profile_path,
    )
    r120 = _validate_r120(
        profile=profile,
        report_path=r120_report_path,
        profile_path=r120_profile_path,
    )
    _validate_cross_source_lineage(r108=r108, r113=r113, r120=r120)
    descriptor = _validate_descriptor(
        profile=profile,
        descriptor_bytes=descriptor_bytes,
        r113=r113,
        r120=r120,
    )
    controller_sources = _validate_controller_sources(profile)
    cache, cache_audit = _validate_cache(
        profile=profile,
        cache_path=r120_cache_path,
        r120=r120,
    )
    contact_audit = audit_contact_schedule(
        v9_complete_clip_path,
        expected_sha256=profile["source"]["v9_complete_clip_sha256"],
        point_identity=r113["point_contact_force_identity"],
        r120=r120,
    )
    inventory = audit_system_inventory(
        active_point_collocations=contact_audit["active_point_collocation_count"],
        inactive_point_collocations=contact_audit["inactive_point_collocation_count"],
    )
    if inventory != _profile_inventory(profile):
        raise ValueError("R121 system inventory differs")
    fixed_pd_audit = audit_fixed_pd_schedule(cache=cache, descriptor=descriptor)
    _validate_fixed_pd_audit(profile, fixed_pd_audit)
    budget_audit = audit_future_execution_budget(
        profile["future_single_execution_budget"]
    )
    if budget_audit["status"] != "PASS":
        raise ValueError("R121 future execution budget differs")
    validations = _validate_results(profile, validation_results)

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "formulation_id": FORMULATION_ID,
        "scope": profile["scope"],
        "frozen_invariants": profile["frozen_invariants"],
        "source_gates": {
            "r108_status": r108["status"],
            "r108_report_sha256": r108["report_sha256"],
            "r108_stage_2_acceptance_authority": r108["stage_contracts"][
                "stage_2_fixed_pd_inverse_dynamics"
            ]["acceptance_authority"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r113_model_identity_satisfied": r113["model_identity_result"][
                "single_backend_neutral_dynamics_model_available"
            ],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
            "r120_accepted_fraction": r120["solver_result"]["accepted_anchors"][-1][
                "fraction"
            ],
            "r120_exact_pass": (
                r120["solver_result"]["accepted_exact_result"]["status"] == "PASS"
            ),
        },
        "state_lift_contract": profile["state_lift_contract"],
        "decision_variables": profile["decision_variables"],
        "equation_contract": profile["equation_contract"],
        "fixed_pd_contract": profile["fixed_pd_contract"],
        "contact_schedule_contract": profile["contact_schedule_contract"],
        "cache_identity_audit": cache_audit,
        "descriptor_inventory": _descriptor_inventory(descriptor),
        "controller_source_identity": controller_sources,
        "fixed_pd_schedule_audit": fixed_pd_audit,
        "contact_schedule_audit": contact_audit,
        "system_inventory_audit": inventory,
        "implementation_conformance_contract": profile[
            "implementation_conformance_contract"
        ],
        "future_single_execution_budget": profile["future_single_execution_budget"],
        "future_single_execution_budget_audit": budget_audit,
        "future_execution_output": profile["future_execution_output"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "solver_import_audit": dict(solver_import_audit),
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r108_report_file_sha256": sha256(r108_report_path),
            "r108_profile_sha256": sha256(r108_profile_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_profile_sha256": sha256(r120_profile_path),
            "r120_cache_sha256": sha256(r120_cache_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "inverse_dynamics_execution_formulations": 1,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_contact_schedule(
    clip_path: Path,
    *,
    expected_sha256: str,
    point_identity: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> dict[str, Any]:
    if sha256(clip_path) != expected_sha256:
        raise ValueError("R121 V9 clip identity differs")
    with np.load(clip_path, allow_pickle=False) as archive:
        modes = np.array(archive["contact_modes"], copy=True)
    emitted = r120["solver_result"]["accepted_exact_result"]["emitted_hashes"]
    expected_mode = emitted["arrays"]["contact_modes"]
    if (
        modes.shape != (FRAME_COUNT, 2)
        or modes.dtype != np.uint8
        or _array_sha256(modes) != expected_mode["sha256"]
        or expected_mode["shape"] != [FRAME_COUNT, 2]
        or expected_mode["dtype"] != "uint8"
    ):
        raise ValueError("R121 contact-mode source differs")
    points = point_identity.get("ordered_points", ())
    if len(points) != POINT_COUNT or [row.get("point_ordinal") for row in points] != [
        0,
        1,
        2,
        3,
    ]:
        raise ValueError("R121 ordered point identity differs")

    active_motor_points = 0
    mode_counts: dict[str, int] = {}
    names = {
        (0, 0): "flight_flight",
        (0, 2): "flight_forefoot",
        (0, 3): "flight_flat",
        (2, 0): "forefoot_flight",
        (3, 0): "flat_flight",
    }
    for row in modes[:MOTOR_INTERVAL_COUNT]:
        key = (int(row[0]), int(row[1]))
        if key not in names:
            raise ValueError("R121 contact-mode pair differs")
        name = names[key]
        mode_counts[name] = mode_counts.get(name, 0) + 1
        active_motor_points += sum(
            int(row[int(point["side_index"])])
            in tuple(int(value) for value in point["active_mode_bits"])
            for point in points
        )
    active = active_motor_points * SUBSTEPS_PER_INTERVAL
    total = COLLOCATION_COUNT * POINT_COUNT
    return {
        "status": "PASS",
        "source_schedule": "BYTE_EXACT_V9_UNCHANGED_BY_R120",
        "mode_counts_over_motor_intervals": mode_counts,
        "motor_active_point_count": active_motor_points,
        "active_point_collocation_count": active,
        "inactive_point_collocation_count": total - active,
        "total_point_collocation_count": total,
        "last_frame_is_endpoint_only": True,
    }


def audit_system_inventory(
    *, active_point_collocations: int, inactive_point_collocations: int
) -> dict[str, int]:
    if (
        active_point_collocations < 0
        or inactive_point_collocations < 0
        or active_point_collocations + inactive_point_collocations
        != COLLOCATION_COUNT * POINT_COUNT
    ):
        raise ValueError("R121 contact collocation inventory is invalid")
    accelerations = COLLOCATION_COUNT * GENERALIZED_WIDTH
    efforts = COLLOCATION_COUNT * ACTUATOR_COUNT
    point_forces = COLLOCATION_COUNT * POINT_COUNT * POINT_FORCE_WIDTH
    contact_active_rows = active_point_collocations * POINT_FORCE_WIDTH
    inactive_rows = inactive_point_collocations * POINT_FORCE_WIDTH
    total = accelerations + efforts + point_forces
    return {
        "physics_collocation_count": COLLOCATION_COUNT,
        "local_unknown_count": LOCAL_UNKNOWN_COUNT,
        "generalized_acceleration_scalar_count": accelerations,
        "applied_effort_scalar_count": efforts,
        "point_force_scalar_count": point_forces,
        "total_decision_scalar_count": total,
        "rigid_body_equality_row_count": accelerations,
        "fixed_pd_identity_row_count": efforts,
        "active_contact_acceleration_closure_row_count": contact_active_rows,
        "inactive_contact_zero_force_row_count": inactive_rows,
        "total_equality_row_count": total,
        "friction_second_order_cone_count": active_point_collocations,
    }


def audit_fixed_pd_schedule(
    *, cache: Mapping[str, NDArray[Any]], descriptor: Mapping[str, Any]
) -> dict[str, Any]:
    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    if (
        [int(row["dof_ordinal"]) for row in joints] != list(range(ACTUATOR_COUNT))
        or [int(row["dof_ordinal"]) for row in actuators] != list(range(ACTUATOR_COUNT))
        or any(
            actuator["joint_id"] != joint["joint_id"]
            for actuator, joint in zip(actuators, joints, strict=True)
        )
    ):
        raise ValueError("R121 descriptor actuator order differs")

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
            target != position_urad[interval],
            "target_soft_clamp",
            counts,
            channels,
        )
        next_target = np.minimum(
            np.maximum(target, applied_target - target_delta),
            applied_target + target_delta,
        )
        _record_mask(next_target != target, "target_slew", counts, channels)
        applied_target = next_target
        maxima["target_lag_microradians"] = max(
            maxima["target_lag_microradians"],
            float(np.max(np.abs(applied_target - position_urad[interval]))),
        )
        used_work = np.zeros(ACTUATOR_COUNT, dtype=np.float64)

        for substep in range(SUBSTEPS_PER_INTERVAL):
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
            _record_mask(
                hard_excess > 10.0,
                "hard_rom_violation",
                counts,
                channels,
            )
            _record_mask(
                velocity_excess > 0.0,
                "velocity_violation",
                counts,
                channels,
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
            nonzero_velocity = absolute_velocity > 0.0
            power_bound = np.full(ACTUATOR_COUNT, np.inf, dtype=np.float64)
            power_bound[nonzero_velocity] = np.floor(
                maximum_power[nonzero_velocity]
                * 1_000_000.0
                / absolute_velocity[nonzero_velocity]
            )
            remaining_work = maximum_work - used_work
            work_bound = np.full(ACTUATOR_COUNT, np.inf, dtype=np.float64)
            work_bound[nonzero_velocity] = np.floor(
                remaining_work[nonzero_velocity]
                * 240.0
                * 1_000_000.0
                / absolute_velocity[nonzero_velocity]
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
            _record_mask(
                infeasible,
                "infeasible_effort_envelope",
                counts,
                channels,
            )
            effort = np.minimum(np.maximum(requested, lower), upper)
            charge = np.ceil(
                np.maximum(effort * observed_velocity, 0.0) / (240.0 * 1_000_000.0)
            )
            used_work += charge
            _record_mask(
                used_work > maximum_work,
                "infeasible_effort_envelope",
                counts,
                channels,
            )
            previous_effort = effort
            maxima["absolute_applied_effort_micronewton_metres"] = max(
                maxima["absolute_applied_effort_micronewton_metres"],
                float(np.max(np.abs(effort))),
            )
            maxima["absolute_power_microwatts"] = max(
                maxima["absolute_power_microwatts"],
                float(np.max(np.abs(effort * observed_velocity)) / 1_000_000.0),
            )
            maxima["positive_work_microjoules_per_motor_tick"] = max(
                maxima["positive_work_microjoules_per_motor_tick"],
                float(np.max(used_work)),
            )

    return {
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


def audit_future_execution_budget(budget: Mapping[str, Any]) -> dict[str, Any]:
    passed = bool(
        budget.get("run_id") == "R123"
        and budget.get("process_count") == 1
        and budget.get("thread_count") == 1
        and budget.get("maximum_local_system_solves") == COLLOCATION_COUNT
        and budget.get("maximum_local_unknown_count") == LOCAL_UNKNOWN_COUNT
        and budget.get("maximum_wall_clock_seconds") == 7200
        and budget.get("maximum_resident_memory_bytes") == 8 * 1024**3
        and budget.get("random_seed") == 0
        and budget.get("randomized_restart_count") == 0
        and budget.get("resume_or_warm_restart") == "FORBIDDEN"
        and budget.get("manual_intervention") == "FORBIDDEN"
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "one_process": budget.get("process_count") == 1,
        "maximum_local_system_solves": budget.get("maximum_local_system_solves"),
        "maximum_local_unknown_count": budget.get("maximum_local_unknown_count"),
        "maximum_wall_clock_seconds": budget.get("maximum_wall_clock_seconds"),
        "maximum_resident_memory_bytes": budget.get("maximum_resident_memory_bytes"),
        "restart_count": budget.get("randomized_restart_count"),
    }


def affine_sample(
    left: NDArray[np.float64], right: NDArray[np.float64], substep: int
) -> NDArray[np.float64]:
    if (
        left.shape != right.shape
        or substep < 0
        or substep >= SUBSTEPS_PER_INTERVAL
        or not np.all(np.isfinite(left))
        or not np.all(np.isfinite(right))
    ):
        raise ValueError("R121 affine sample input differs")
    fraction = substep / SUBSTEPS_PER_INTERVAL
    return (1.0 - fraction) * left + fraction * right


def _validate_r108(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r108"]
    report = _load_bound_report(report_path, profile_path, expected)
    stage = report.get("stage_contracts", {}).get(
        "stage_2_fixed_pd_inverse_dynamics", {}
    )
    identities = report.get("identities", {})
    if (
        report.get("check") != "TRAIN-4-PROGRESSIVE-KINODYNAMIC-FORMULATION"
        or report.get("formulation_id")
        != "nextengine.humanoid-progressive-kinodynamic-formulation.v1"
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R109_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        or stage.get("acceptance_authority") is not False
        or stage.get("decision_variables")
        != "240 Hz generalized acceleration, 23 actuator efforts and scheduled contact wrenches"
        or report.get("stage_transitions", {}).get("inverse_dynamics_complete")
        != "PERMIT_SEPARATE_FULL_KINODYNAMIC_EXECUTION_FORMULATION_ONLY"
        or identities.get("formulation_module_sha256")
        != expected["formulation_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(counter, -1)) != 0
            for counter in (
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "physx_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R121 R108 source contract differs")
    return report


def _validate_r113(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r113"]
    report = _load_bound_report(report_path, profile_path, expected)
    identities = report.get("identities", {})
    model = report.get("model_identity_result", {})
    point = report.get("point_contact_force_identity", {})
    if (
        report.get("check") != "TRAIN-4-CLEAN-DYNAMICS-MODEL-IDENTITY-PREFLIGHT"
        or report.get("status") != "PASS"
        or report.get("gate_decision")
        != "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY"
        or model.get("single_backend_neutral_dynamics_model_available") is not True
        or model.get("point_force_contract_satisfied") is not True
        or model.get("blocking_reason_count") != 0
        or point.get("status") != "FROZEN_SOLVER_PRIVATE_POINT_FORCE_COORDINATES"
        or identities.get("preflight_module_sha256")
        != expected["preflight_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(counter, -1)) != 0
            for counter in (
                "solver_runs",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R121 R113 source contract differs")
    return report


def _validate_r120(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r120"]
    report = _load_bound_report(report_path, profile_path, expected)
    exact = report.get("solver_result", {}).get("accepted_exact_result", {})
    cache = report.get("solver_private_warm_start_cache", {})
    bounded = report.get("bounded_acceptance", {})
    identities = report.get("identities", {})
    if (
        report.get("check") != "TRAIN-4-REPAIRED-KTO-EXECUTION"
        or report.get("status") != "PASS"
        or report.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R121_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_FORMULATION_ONLY"
        or exact.get("status") != "PASS"
        or not all(exact.get("exact_gates", {}).values())
        or exact.get("emitted_hashes", {}).get("aggregate_sha256")
        != expected["emitted_aggregate_sha256"]
        or cache.get("status") != "EMITTED_TRANSIENT_SOLVER_PRIVATE"
        or cache.get("sha256") != expected["cache_sha256"]
        or cache.get("candidate_or_corpus_authority") is not False
        or report.get("kto_processes") != 1
        or report.get("kto_solves") != 1
        or report.get("qp_solves") != 1
        or report.get("in_memory_emitted_iterates") != 6
        or any(
            int(report.get(counter, -1)) != 0
            for counter in (
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
        or bounded.get("r121_inverse_dynamics_formulation")
        != "AUTHORIZED_ONLY_ON_R120_EXACT_PASS"
        or bounded.get("inverse_dynamics_solve") != "NOT_AUTHORIZED"
        or identities.get("execution_module_sha256")
        != expected["execution_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
    ):
        raise ValueError("R121 R120 source contract differs")
    return report


def _load_bound_report(
    report_path: Path, profile_path: Path, expected: Mapping[str, Any]
) -> dict[str, Any]:
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R121 source file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError("R121 source canonical identity differs")
    return report


def _validate_cross_source_lineage(
    *, r108: Mapping[str, Any], r113: Mapping[str, Any], r120: Mapping[str, Any]
) -> None:
    shared = r113.get("shared_identity", {})
    descriptor = r120.get("descriptor_lineage", {})
    stage = r108.get("stage_contracts", {}).get("stage_2_fixed_pd_inverse_dynamics", {})
    if (
        shared.get("compiled_descriptor_hash")
        != descriptor.get("compiled_descriptor_hash")
        or shared.get("body_schema_hash") != descriptor.get("body_schema_hash")
        or shared.get("material_lineage_hash")
        != descriptor.get("material_lineage_hash")
        or shared.get("motor_hz") != 60
        or shared.get("physics_hz") != 240
        or stage.get("inputs")
        != "stage-1 q/v/a warm start plus hash-bound body/joint/actuator/material/gravity model"
    ):
        raise ValueError("R121 cross-source lineage differs")


def _validate_descriptor(
    *,
    profile: Mapping[str, Any],
    descriptor_bytes: bytes,
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> dict[str, Any]:
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["current_descriptor_file_sha256"]
    ):
        raise ValueError("R121 descriptor file identity differs")
    descriptor = json.loads(descriptor_bytes)
    bodies = descriptor.get("bodies", ())
    joints = descriptor.get("joints", ())
    actuators = descriptor.get("actuators", ())
    shared = r113["shared_identity"]
    lineage = r120["descriptor_lineage"]
    if (
        descriptor.get("schema_version") != 2
        or descriptor.get("body_count") != 24
        or len(bodies) != 24
        or len(joints) != 23
        or len(actuators) != 23
        or descriptor.get("motor_hz") != 60
        or descriptor.get("physics_hz") != 240
        or descriptor.get("body_schema_hash") != shared["body_schema_hash"]
        or descriptor.get("compiled_descriptor_hash")
        != shared["compiled_descriptor_hash"]
        or descriptor.get("material_lineage_hash") != shared["material_lineage_hash"]
        or descriptor.get("compiled_descriptor_hash")
        != lineage["compiled_descriptor_hash"]
        or sum(int(row["mass_microkilograms"]) for row in bodies)
        != shared["total_mass_microkilograms"]
        or sorted(int(row["dof_ordinal"]) for row in joints) != list(range(23))
        or sorted(int(row["dof_ordinal"]) for row in actuators) != list(range(23))
    ):
        raise ValueError("R121 descriptor inventory differs")
    return descriptor


def _validate_controller_sources(profile: Mapping[str, Any]) -> list[dict[str, str]]:
    lab_root = Path(__file__).resolve().parents[1]
    expected = profile["source"]["tracked_controller_sha256"]
    sources = (
        ("motor_mirror", lab_root / "next_lab/motor_mirror.py"),
        ("isaac_reference_env", lab_root / "next_lab/isaac_reference_env.py"),
    )
    rows = []
    for source_id, path in sources:
        actual = sha256(path)
        if actual != expected[source_id]:
            raise ValueError(f"R121 tracked controller source {source_id} differs")
        rows.append({"id": source_id, "status": "PASS", "sha256": actual})
    return rows


def _validate_cache(
    *, profile: Mapping[str, Any], cache_path: Path, r120: Mapping[str, Any]
) -> tuple[dict[str, NDArray[Any]], dict[str, Any]]:
    expected = profile["source"]["r120"]
    if sha256(cache_path) != expected["cache_sha256"]:
        raise ValueError("R121 R120 cache identity differs")
    with np.load(cache_path, allow_pickle=False) as archive:
        if tuple(archive.files) != CACHE_KEYS:
            raise ValueError("R121 R120 cache keys differ")
        cache = {name: np.array(archive[name], copy=True) for name in CACHE_KEYS}
    shapes = {
        "root_position_m": (FRAME_COUNT, 3),
        "root_orientation_delta_rad": (FRAME_COUNT, 3),
        "joint_position_rad": (FRAME_COUNT, ACTUATOR_COUNT),
        "velocity": (FRAME_COUNT, GENERALIZED_WIDTH),
        "acceleration": (FRAME_COUNT, GENERALIZED_WIDTH),
        "reference_root_rotation": (FRAME_COUNT, 3, 3),
        "metadata_json_utf8": (264,),
    }
    if any(cache[name].shape != shape for name, shape in shapes.items()):
        raise ValueError("R121 R120 cache shape differs")
    if (
        any(cache[name].dtype != np.float64 for name in CACHE_KEYS[:-1])
        or cache["metadata_json_utf8"].dtype != np.uint8
        or any(not np.all(np.isfinite(cache[name])) for name in CACHE_KEYS[:-1])
    ):
        raise ValueError("R121 R120 cache dtype or finiteness differs")
    metadata = json.loads(cache["metadata_json_utf8"].tobytes())
    if metadata != {
        "cache_id": "nextengine.humanoid-r120-repaired-kto-solver-private.v1",
        "candidate_or_corpus_authority": False,
        "emitted_aggregate_sha256": expected["emitted_aggregate_sha256"],
        "frame_count": FRAME_COUNT,
        "qva_scalar_count": FRAME_COUNT * 87,
        "schema_version": 1,
    }:
        raise ValueError("R121 R120 cache metadata differs")
    rotations = cache["reference_root_rotation"]
    identity = np.eye(3, dtype=np.float64)
    orthogonality = np.max(
        np.abs(np.matmul(rotations, np.swapaxes(rotations, 1, 2)) - identity)
    )
    determinant_error = np.max(np.abs(np.linalg.det(rotations) - 1.0))
    if orthogonality > 1e-12 or determinant_error > 1e-12:
        raise ValueError("R121 reference rotation identity differs")

    emitted = r120["solver_result"]["accepted_exact_result"]["emitted_hashes"]
    projected = {
        "root_position_um": np.rint(cache["root_position_m"] * 1_000_000.0).astype(
            np.int64
        ),
        "joint_position_urad": np.rint(
            cache["joint_position_rad"] * 1_000_000.0
        ).astype(np.int64),
        "root_linear_velocity_um_s": np.rint(
            cache["velocity"][:, :3] * 1_000_000.0
        ).astype(np.int64),
        "joint_velocity_urad_s": np.rint(cache["velocity"][:, 6:] * 1_000_000.0).astype(
            np.int64
        ),
    }
    hash_rows = []
    for name, array in projected.items():
        actual = _array_sha256(array)
        expected_array = emitted["arrays"][name]
        if (
            actual != expected_array["sha256"]
            or list(array.shape) != expected_array["shape"]
            or str(array.dtype) != expected_array["dtype"]
        ):
            raise ValueError(f"R121 cache projection {name} differs")
        hash_rows.append({"array": name, "status": "PASS", "sha256": actual})
    return cache, {
        "status": "PASS",
        "cache_id": metadata["cache_id"],
        "candidate_or_corpus_authority": False,
        "qva_scalar_count": metadata["qva_scalar_count"],
        "cache_file_sha256": sha256(cache_path),
        "emitted_aggregate_sha256": metadata["emitted_aggregate_sha256"],
        "emitted_projection_hashes": hash_rows,
        "maximum_reference_rotation_orthogonality_error": float(orthogonality),
        "maximum_reference_rotation_determinant_error": float(determinant_error),
    }


def _descriptor_inventory(descriptor: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "status": "PASS",
        "schema_version": descriptor["schema_version"],
        "body_count": len(descriptor["bodies"]),
        "joint_count": len(descriptor["joints"]),
        "actuator_count": len(descriptor["actuators"]),
        "collider_count": sum(len(row["colliders"]) for row in descriptor["bodies"]),
        "total_mass_microkilograms": sum(
            int(row["mass_microkilograms"]) for row in descriptor["bodies"]
        ),
        "body_schema_hash": descriptor["body_schema_hash"],
        "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
        "material_lineage_hash": descriptor["material_lineage_hash"],
        "motor_hz": descriptor["motor_hz"],
        "physics_hz": descriptor["physics_hz"],
    }


def _profile_inventory(profile: Mapping[str, Any]) -> dict[str, int]:
    variables = profile["decision_variables"]
    equations = profile["equation_contract"]
    return {
        "physics_collocation_count": profile["scope"]["physics_collocation_count"],
        "local_unknown_count": variables["per_physics_collocation"]["total"],
        "generalized_acceleration_scalar_count": variables[
            "total_generalized_acceleration_scalars"
        ],
        "applied_effort_scalar_count": variables["total_applied_effort_scalars"],
        "point_force_scalar_count": variables["total_point_force_scalars"],
        "total_decision_scalar_count": variables["total_decision_scalars"],
        "rigid_body_equality_row_count": (
            profile["scope"]["physics_collocation_count"]
            * equations["rigid_body_equality_rows_per_collocation"]
        ),
        "fixed_pd_identity_row_count": (
            profile["scope"]["physics_collocation_count"]
            * equations["fixed_pd_effort_identity_rows_per_collocation"]
        ),
        "active_contact_acceleration_closure_row_count": equations[
            "active_point_acceleration_closure_scalar_rows"
        ],
        "inactive_contact_zero_force_row_count": equations[
            "inactive_point_zero_force_scalar_rows"
        ],
        "total_equality_row_count": equations["total_equality_rows"],
        "friction_second_order_cone_count": equations[
            "friction_second_order_cone_count"
        ],
    }


def _validate_fixed_pd_audit(
    profile: Mapping[str, Any], audit: Mapping[str, Any]
) -> None:
    expected = profile["fixed_pd_contract"]["expected_formulation_audit"]
    counts = audit.get("activation_counts", {})
    channels = audit.get("activated_dof_ordinals", {})
    maxima = audit.get("maxima", {})
    if (
        audit.get("status") != expected["status"]
        or audit.get("collocation_count") != COLLOCATION_COUNT
        or counts != expected["activation_counts"]
        or channels.get("target_slew") != expected["target_slew_dof_ordinals"]
        or channels.get("effort_rate_clamp")
        != expected["effort_rate_clamp_dof_ordinals"]
        or any(
            channels.get(name) != []
            for name in (
                "hard_rom_violation",
                "velocity_violation",
                "target_soft_clamp",
                "static_effort_clamp",
                "power_clamp",
                "positive_work_clamp",
                "infeasible_effort_envelope",
            )
        )
        or maxima.get("target_lag_microradians")
        != expected["maximum_target_lag_microradians"]
        or maxima.get("absolute_requested_effort_micronewton_metres")
        != expected["maximum_absolute_requested_effort_micronewton_metres"]
        or maxima.get("absolute_applied_effort_micronewton_metres")
        != expected["maximum_absolute_applied_effort_micronewton_metres"]
        or maxima.get("positive_work_microjoules_per_motor_tick")
        != expected["maximum_positive_work_microjoules_per_motor_tick"]
    ):
        raise ValueError("R121 fixed-PD formulation audit differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R121 validation result differs")
    return normalized


def _validate_repository(repository: Mapping[str, Any]) -> None:
    commit = repository.get("commit")
    if (
        not isinstance(commit, str)
        or len(commit) != 40
        or any(character not in "0123456789abcdef" for character in commit)
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R121 formulation requires a clean repository")


def _validate_solver_import_audit(audit: Mapping[str, Any]) -> None:
    if audit != {
        "osqp_module_loaded": False,
        "pinocchio_module_loaded": False,
        "inverse_dynamics_solver_module_imported": False,
        "inverse_dynamics_execution_module_imported": False,
        "solver_execution_requested": False,
    }:
        raise ValueError("R121 solver-free import audit differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    lift = profile.get("state_lift_contract", {})
    variables = profile.get("decision_variables", {})
    local = variables.get("per_physics_collocation", {})
    equations = profile.get("equation_contract", {})
    contacts = profile.get("contact_schedule_contract", {})
    fixed_pd = profile.get("fixed_pd_contract", {})
    conformance = profile.get("implementation_conformance_contract", {})
    budget = profile.get("future_single_execution_budget", {})
    transitions = profile.get("result_transitions", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "FixedPdInverseDynamicsExecutionFormulationOnly"
        or source.get("r108", {}).get("report_sha256")
        != "4ab1ccbc697fdf97efadc9e53ca6f2605956000927c88ed76f1960917590f9e8"
        or source.get("r113", {}).get("report_sha256")
        != "3ac92ae2ca508234a52d77f0414ad5557f1164028e51a3938cc045ac4c5147cf"
        or source.get("r120", {}).get("report_sha256")
        != "dfcb05e006467ee30bab70aac00f4408acce26782fbdbf5d1cea89821c06953b"
        or source.get("r120", {}).get("cache_sha256")
        != "e305fc5888a1cf1dff238c32ac07707a284b215bb49d25920dfb5ee8f097afc5"
        or scope
        != {
            "run_id": "R121",
            "clip_id": "cmu16-walk-nominal-b",
            "frame_count": 801,
            "motor_interval_count": 800,
            "physics_substeps_per_motor_interval": 4,
            "physics_collocation_count": 3200,
            "body_count": 24,
            "joint_count": 23,
            "actuator_count": 23,
            "scheduled_point_count": 4,
            "generalized_velocity_width": 29,
            "motor_hz": 60,
            "physics_hz": 240,
            "inverse_dynamics_execution_formulations": 1,
            "inverse_dynamics_solves": 0,
            "candidate_construction": False,
            "physx_scene_runs": 0,
            "training": False,
        }
        or tuple(lift.get("collocation_abscissae", ())) != ("0", "1/4", "1/2", "3/4")
        or "independent affine interpolation"
        not in lift.get("translation_and_joint_configuration", "")
        or "independent affine interpolation"
        not in lift.get("linear_angular_and_joint_velocity", "")
        or lift.get("kinematic_derivative_identity") != "NOT_CLAIMED_IN_STAGE_2"
        or lift.get("discrete_integration_constraint")
        != "ABSENT_BY_DESIGN_AND_RESERVED_FOR_STAGE_3_FULL_KINODYNAMICS"
        or local
        != {
            "generalized_acceleration": 29,
            "applied_actuator_effort": 23,
            "ordered_point_force": 12,
            "total": 64,
        }
        or variables.get("total_decision_scalars") != 204800
        or variables.get("configuration_or_velocity_variables") != 0
        or equations.get("total_equality_rows") != 204800
        or equations.get("friction_second_order_cone_count") != 4956
        or equations.get("impact_or_complementarity_variables") != 0
        or fixed_pd.get("observed_hard_rom_tolerance_microradians") != 10
        or fixed_pd.get("expected_formulation_audit", {})
        .get("activation_counts", {})
        .get("effort_rate_clamp")
        != 641
        or contacts.get("motor_active_point_count") != 1239
        or contacts.get("active_point_collocation_count") != 4956
        or contacts.get("inactive_point_collocation_count") != 7844
        or conformance.get("next_run_id") != "R122"
        or conformance.get("execution_in_r122") is not False
        or conformance.get("required_anchors") != [0, 238, 244, 249, 328, 626, 800]
        or conformance.get("pass_transition")
        != "PERMIT_R123_SINGLE_BOUNDED_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or audit_future_execution_budget(budget)["status"] != "PASS"
        or transitions.get("r121_complete")
        != "PERMIT_R122_FIXED_PD_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY"
        or transitions.get("r123_valid_complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_R124_FULL_KINODYNAMIC_EXECUTION_FORMULATION_ONLY"
        or profile.get("decision", {}).get("complete")
        != "PERMIT_R122_FIXED_PD_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY"
        or bounded.get("r122_implementation_conformance")
        != "AUTHORIZED_REPORT_ONLY_ON_R121_COMPLETE"
        or bounded.get("r123_inverse_dynamics_execution")
        != "NOT_AUTHORIZED_UNTIL_R122_PASS"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_kto_solve",
                "inverse_dynamics_solve",
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
            "solver_free_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("R121 formulation profile differs")


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
