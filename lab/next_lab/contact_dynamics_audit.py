from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Mapping, Sequence

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_manifold_physx import (
    ContactPrototypeCase,
    load_case_arrays,
    load_contact_prototype_cases,
)
from next_lab.motion_math import collider_minimum_y, target_forward_kinematics


AUDIT_ID = "nextengine.humanoid-contact-manifold-native-dynamics-audit.v1"
V7_PROTOTYPE_ID = "nextengine.humanoid-contact-manifold-prototype.v7"
V9_PROTOTYPE_ID = "nextengine.humanoid-contact-manifold-prototype.v9"
_SIDES = ("left", "right")
_ROOT_AXES = ("x", "y", "z")


def build_native_dynamics_audit(
    *,
    profile_path: Path,
    source_audit_path: Path,
    descriptor_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
    r94_report_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Build the hash-closed, report-only V7/V9 differential audit."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            source_audit_path,
            descriptor_path,
            v7_manifest_path,
            v9_manifest_path,
            r94_report_path,
            tool_path,
        )
    )
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("native-dynamics audit input is absent")
    (
        profile_path,
        source_audit_path,
        descriptor_path,
        v7_manifest_path,
        v9_manifest_path,
        r94_report_path,
        tool_path,
    ) = paths
    profile = json.loads(profile_path.read_bytes())
    descriptor = json.loads(descriptor_path.read_bytes())
    v7_manifest, _, v7_cases = load_contact_prototype_cases(
        manifest_path=v7_manifest_path,
        source_audit_path=source_audit_path,
    )
    v9_manifest, _, v9_cases = load_contact_prototype_cases(
        manifest_path=v9_manifest_path,
        source_audit_path=source_audit_path,
    )
    r94_report = json.loads(r94_report_path.read_bytes())
    _validate_inputs(
        profile=profile,
        profile_path=profile_path,
        source_audit_path=source_audit_path,
        descriptor=descriptor,
        descriptor_path=descriptor_path,
        v7_manifest=v7_manifest,
        v7_manifest_path=v7_manifest_path,
        v7_cases=v7_cases,
        v9_manifest=v9_manifest,
        v9_manifest_path=v9_manifest_path,
        v9_cases=v9_cases,
        r94_report=r94_report,
        r94_report_path=r94_report_path,
        repository=repository,
    )
    r94_by_ordinal = {
        int(row["case_ordinal"]): row
        for row in r94_report["fresh_scene"]["results"]["phase_results"]
    }
    thresholds = profile["discriminator"]
    case_records = []
    for v7_case, v9_case in zip(v7_cases, v9_cases, strict=True):
        v7_arrays = load_case_arrays(v7_case)
        v9_arrays = load_case_arrays(v9_case)
        _validate_case_pair(v7_case, v9_case, v7_arrays, v9_arrays)
        case_records.append(
            _case_record(
                v7_case=v7_case,
                v9_case=v9_case,
                v7_arrays=v7_arrays,
                v9_arrays=v9_arrays,
                r94_row=r94_by_ordinal[v7_case.ordinal],
                descriptor=descriptor,
                thresholds=thresholds,
            )
        )
    findings = _findings(case_records, thresholds)
    findings["r94_initial_state_verification"] = _r94_initial_state_summary(
        r94_report
    )
    report = {
        "schema_version": 1,
        "check": "TRAIN-4-NATIVE-DYNAMICS-DIFFERENTIAL-AUDIT",
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "audit_id": profile["audit_id"],
        "scope": profile["scope"],
        "method": {
            "measurements": profile["measurements"],
            "discriminator": thresholds,
            "derivative_semantics": (
                "first differences of emitted 60 Hz reference velocities; "
                "jerk is the first difference of that acceleration"
            ),
            "fixed_pd_proxy_semantics": (
                "initial damping uses q_des=q at reset; one-frame-lag proxy "
                "uses the frozen slew-limited reference target with the "
                "previous emitted reference pose and velocity"
            ),
        },
        "identities": {
            "profile_sha256": sha256(profile_path),
            "source_audit_file_sha256": sha256(source_audit_path),
            "descriptor_file_sha256": sha256(descriptor_path),
            "v7_manifest_sha256": v7_manifest["manifest_sha256"],
            "v7_manifest_file_sha256": sha256(v7_manifest_path),
            "v9_manifest_sha256": v9_manifest["manifest_sha256"],
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "r94_report_sha256": r94_report["manifest_sha256"],
            "r94_report_file_sha256": sha256(r94_report_path),
            "tool_sha256": sha256(tool_path),
            "audit_module_sha256": sha256(Path(__file__).resolve()),
        },
        "cases": case_records,
        "findings": findings,
        "bounded_acceptance": {
            "candidate_status": "NOT_EVALUATED",
            "selected_fresh_case_ordinals": findings[
                "selected_counterfactual_case_ordinals"
            ],
            "maximum_selected_fresh_cases": thresholds[
                "maximum_selected_fresh_cases"
            ],
            "full_all17": profile["decision"]["full_all17"],
            "full_v19": profile["decision"]["full_v19"],
            "training": profile["decision"]["training"],
        },
        "optimizer_steps": 0,
        "training_runs": 0,
        "physx_runs": 0,
        "trajectory_mutations": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def fixed_pd_proxy_metrics(
    *,
    joint_position_urad: NDArray[np.int64],
    joint_velocity_urad_s: NDArray[np.int64],
    reference_frame: NDArray[np.int64],
    descriptor: Mapping[str, Any],
) -> dict[str, Any]:
    """Evaluate diagnostic fixed-PD loads without simulating plant dynamics."""

    position = np.asarray(joint_position_urad, dtype=np.int64)
    velocity = np.asarray(joint_velocity_urad_s, dtype=np.int64)
    frames = np.asarray(reference_frame, dtype=np.int64)
    actuators = _ordered_actuators(descriptor, position.shape[1])
    if velocity.shape != position.shape or frames.shape != (len(position),):
        raise ValueError("fixed-PD proxy array shape mismatch")
    stiffness = np.asarray(
        [int(item["stiffness_q16"]) for item in actuators], dtype=np.int64
    )
    damping = np.asarray(
        [int(item["damping_q16"]) for item in actuators], dtype=np.int64
    )
    effort_limit = np.asarray(
        [
            max(abs(int(value)) for value in item["effort_micronewton_metres"])
            for item in actuators
        ],
        dtype=np.int64,
    )
    effort_rate_limit = np.asarray(
        [
            int(item["maximum_effort_rate_micronewton_metres_per_second"])
            for item in actuators
        ],
        dtype=np.int64,
    )
    target_delta = np.asarray(
        [
            max(
                abs(int(value))
                for value in item[
                    "target_delta_microradians_per_motor_tick"
                ]
            )
            for item in actuators
        ],
        dtype=np.int64,
    )
    if np.any(effort_limit <= 0) or np.any(effort_rate_limit <= 0):
        raise ValueError("fixed-PD proxy effort limits are invalid")
    applied_target = position.copy()
    slew_activation_count = 0
    for frame in range(1, len(position)):
        lower = applied_target[frame - 1] - target_delta
        upper = applied_target[frame - 1] + target_delta
        applied_target[frame] = np.minimum(
            np.maximum(position[frame], lower), upper
        )
        slew_activation_count += int(
            np.count_nonzero(applied_target[frame] != position[frame])
        )
    damping_effort = -np.rint(
        damping[None, :] * velocity / 65_536.0
    ).astype(np.int64)
    lag_effort = damping_effort.copy()
    if len(position) > 1:
        lag_effort[1:] = np.rint(
            stiffness[None, :]
            * (applied_target[1:] - position[:-1])
            / 65_536.0
        ).astype(np.int64) - np.rint(
            damping[None, :] * velocity[:-1] / 65_536.0
        ).astype(np.int64)
    damping_utilization = _ceil_ratio(
        np.abs(damping_effort) * 10_000,
        effort_limit[None, :],
    )
    lag_utilization = _ceil_ratio(
        np.abs(lag_effort) * 10_000,
        effort_limit[None, :],
    )
    lag_rate = np.zeros_like(lag_effort)
    if len(lag_effort) > 1:
        lag_rate[1:] = np.abs(np.diff(lag_effort, axis=0)) * int(
            descriptor["motor_hz"]
        )
    lag_rate_utilization = _ceil_ratio(
        lag_rate * 10_000,
        effort_rate_limit[None, :],
    )
    joint_ids = tuple(str(item["joint_id"]) for item in actuators)
    return {
        "initial_damping": _effort_record(
            effort=damping_effort[:1],
            utilization=damping_utilization[:1],
            frames=frames[:1],
            joint_ids=joint_ids,
            value_name="effort_micronewton_metres",
        ),
        "maximum_damping": _effort_record(
            effort=damping_effort,
            utilization=damping_utilization,
            frames=frames,
            joint_ids=joint_ids,
            value_name="effort_micronewton_metres",
        ),
        "maximum_one_frame_lag_requested": _effort_record(
            effort=lag_effort,
            utilization=lag_utilization,
            frames=frames,
            joint_ids=joint_ids,
            value_name="effort_micronewton_metres",
        ),
        "maximum_one_frame_lag_effort_rate": _effort_record(
            effort=lag_rate,
            utilization=lag_rate_utilization,
            frames=frames,
            joint_ids=joint_ids,
            value_name="effort_rate_micronewton_metres_per_second",
        ),
        "target_slew_activation_count": slew_activation_count,
    }


def select_counterfactual_cases(
    case_records: Sequence[Mapping[str, Any]],
    *,
    maximum_count: int,
) -> list[int]:
    """Select orthogonal contact-only and derivative-only fresh cases."""

    contact_only = [
        row
        for row in case_records
        if row["classification"] == "new_control_regression"
        and row["r94"]["early_hard_impact"]
        and not row["comparison"]["derivative_amplified"]
        and row["comparison"]["impacted_foot_geometry_flags"]
    ]
    derivative_only = [
        row
        for row in case_records
        if row["classification"] == "new_control_regression"
        and "hard_impact" not in row["r94"]["reasons"]
        and row["comparison"]["derivative_amplified"]
    ]
    derivative_only.sort(
        key=lambda row: (
            -max(
                row["comparison"][
                    "joint_acceleration_v9_over_v7_basis_points"
                ],
                row["comparison"]["joint_jerk_v9_over_v7_basis_points"],
            ),
            row["case_ordinal"],
        )
    )
    selected: list[int] = []
    for candidates in (contact_only, derivative_only):
        if candidates and len(selected) < maximum_count:
            ordinal = int(candidates[0]["case_ordinal"])
            if ordinal not in selected:
                selected.append(ordinal)
    return selected


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _case_record(
    *,
    v7_case: ContactPrototypeCase,
    v9_case: ContactPrototypeCase,
    v7_arrays: Mapping[str, NDArray[Any]],
    v9_arrays: Mapping[str, NDArray[Any]],
    r94_row: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    thresholds: Mapping[str, Any],
) -> dict[str, Any]:
    v7_metrics = _trajectory_metrics(v7_arrays, descriptor, thresholds)
    v9_metrics = _trajectory_metrics(v9_arrays, descriptor, thresholds)
    r94 = _r94_outcome(r94_row, v7_case)
    classification = _classification(v7_case.baseline_status, r94["status"])
    acceleration_ratio = _ratio_basis_points(
        v9_metrics["joint"]["maximum_acceleration"]["absolute_value"],
        v7_metrics["joint"]["maximum_acceleration"]["absolute_value"],
    )
    jerk_ratio = _ratio_basis_points(
        v9_metrics["joint"]["maximum_jerk"]["absolute_value"],
        v7_metrics["joint"]["maximum_jerk"]["absolute_value"],
    )
    derivative_amplified = max(acceleration_ratio, jerk_ratio) >= int(
        thresholds["derivative_amplification_basis_points"]
    )
    geometry_flags = _impacted_foot_geometry_flags(
        r94=r94,
        v7_metrics=v7_metrics,
        v9_metrics=v9_metrics,
        thresholds=thresholds,
    )
    comparison = {
        "maximum_joint_position_delta_microradians": _difference_record(
            v9_arrays["joint_position_urad"],
            v7_arrays["joint_position_urad"],
            v9_arrays["reference_frame"],
            _joint_ids(descriptor),
        ),
        "maximum_joint_velocity_delta_microradians_per_second": (
            _difference_record(
                v9_arrays["joint_velocity_urad_s"],
                v7_arrays["joint_velocity_urad_s"],
                v9_arrays["reference_frame"],
                _joint_ids(descriptor),
            )
        ),
        "maximum_root_position_delta_micrometres": _difference_record(
            v9_arrays["root_position_um"],
            v7_arrays["root_position_um"],
            v9_arrays["reference_frame"],
            _ROOT_AXES,
        ),
        "maximum_root_velocity_delta_micrometres_per_second": (
            _difference_record(
                v9_arrays["root_linear_velocity_um_s"],
                v7_arrays["root_linear_velocity_um_s"],
                v9_arrays["reference_frame"],
                _ROOT_AXES,
            )
        ),
        "contact_disagreement_count": int(
            np.count_nonzero(v9_arrays["contacts"] != v7_arrays["contacts"])
        ),
        "contact_mode_disagreement_count": int(
            np.count_nonzero(
                v9_arrays["contact_modes"] != v7_arrays["contact_modes"]
            )
        ),
        "joint_acceleration_v9_over_v7_basis_points": acceleration_ratio,
        "joint_jerk_v9_over_v7_basis_points": jerk_ratio,
        "derivative_amplified": derivative_amplified,
        "impacted_foot_geometry_flags": geometry_flags,
    }
    return {
        "case_ordinal": v7_case.ordinal,
        "source_case_ordinal": v7_case.source_case_ordinal,
        "clip_id": v7_case.clip_id,
        "split": v7_case.split,
        "frame_first": v7_case.frame_first,
        "frame_last": v7_case.frame_last,
        "baseline_status": v7_case.baseline_status,
        "baseline_reasons": list(v7_case.baseline_reasons),
        "classification": classification,
        "r94": r94,
        "artifacts": {
            "v7_sha256": v7_case.artifact_sha256,
            "v9_sha256": v9_case.artifact_sha256,
        },
        "v7": v7_metrics,
        "v9": v9_metrics,
        "comparison": comparison,
    }


def _trajectory_metrics(
    arrays: Mapping[str, NDArray[Any]],
    descriptor: Mapping[str, Any],
    thresholds: Mapping[str, Any],
) -> dict[str, Any]:
    frames = np.asarray(arrays["reference_frame"], dtype=np.int64)
    joint_position = np.asarray(arrays["joint_position_urad"], dtype=np.int64)
    joint_velocity = np.asarray(
        arrays["joint_velocity_urad_s"], dtype=np.int64
    )
    root_position = np.asarray(arrays["root_position_um"], dtype=np.int64)
    root_velocity = np.asarray(
        arrays["root_linear_velocity_um_s"], dtype=np.int64
    )
    motor_hz = int(descriptor["motor_hz"])
    if motor_hz != 60 or len(frames) < 3:
        raise ValueError("native-dynamics audit cadence is invalid")
    joint_ids = _joint_ids(descriptor)
    joint_acceleration = np.diff(joint_velocity, axis=0) * motor_hz
    joint_jerk = np.diff(joint_acceleration, axis=0) * motor_hz
    root_acceleration = np.diff(root_velocity, axis=0) * motor_hz
    root_jerk = np.diff(root_acceleration, axis=0) * motor_hz
    target_step = np.diff(joint_position, axis=0)
    foot = _foot_clearance_metrics(
        arrays=arrays,
        descriptor=descriptor,
        thresholds=thresholds,
    )
    return {
        "joint": {
            "maximum_speed": _maximum_record(
                joint_velocity, frames, joint_ids
            ),
            "maximum_acceleration": _maximum_record(
                joint_acceleration, frames[1:], joint_ids
            ),
            "maximum_jerk": _maximum_record(
                joint_jerk, frames[2:], joint_ids
            ),
            "maximum_target_step": _maximum_record(
                target_step, frames[1:], joint_ids
            ),
        },
        "root": {
            "maximum_speed": _maximum_record(
                root_velocity, frames, _ROOT_AXES
            ),
            "maximum_acceleration": _maximum_record(
                root_acceleration, frames[1:], _ROOT_AXES
            ),
            "maximum_jerk": _maximum_record(
                root_jerk, frames[2:], _ROOT_AXES
            ),
            "maximum_position_step": _maximum_record(
                np.diff(root_position, axis=0), frames[1:], _ROOT_AXES
            ),
        },
        "fixed_pd_proxies": fixed_pd_proxy_metrics(
            joint_position_urad=joint_position,
            joint_velocity_urad_s=joint_velocity,
            reference_frame=frames,
            descriptor=descriptor,
        ),
        "foot_clearance": foot,
        "contact_mode_transition_count": int(
            np.count_nonzero(
                np.any(np.diff(arrays["contact_modes"], axis=0) != 0, axis=1)
            )
        ),
    }


def _foot_clearance_metrics(
    *,
    arrays: Mapping[str, NDArray[Any]],
    descriptor: Mapping[str, Any],
    thresholds: Mapping[str, Any],
) -> dict[str, Any]:
    frames = np.asarray(arrays["reference_frame"], dtype=np.int64)
    modes = np.asarray(arrays["contact_modes"], dtype=np.uint8)
    foot_colliders = _foot_colliders(descriptor)
    heights = np.empty((len(frames), len(_SIDES)), dtype=np.int64)
    for frame in range(len(frames)):
        positions, rotations = target_forward_kinematics(
            descriptor,
            np.asarray(arrays["root_position_um"][frame], dtype=np.float64)
            / 1_000_000.0,
            np.asarray(
                arrays["root_quaternion_q1_30"][frame], dtype=np.float64
            )
            / float(1 << 30),
            np.asarray(
                arrays["joint_position_urad"][frame], dtype=np.float64
            )
            / 1_000_000.0,
        )
        for side, colliders in enumerate(foot_colliders):
            minimum = min(
                collider_minimum_y(
                    positions[slot], rotations[slot], collider
                )
                for slot, collider in colliders
            )
            heights[frame, side] = int(
                np.floor(minimum * 1_000_000.0 + 1.0e-9)
            )
    output: dict[str, Any] = {}
    for side, side_id in enumerate(_SIDES):
        values = heights[:, side]
        minimum_index = int(np.argmin(values))
        downward = np.maximum(-np.diff(values), 0)
        downward_index = int(np.argmax(downward)) + 1
        active = modes[:, side] != 0
        flight = ~active
        output[side_id] = {
            "initial_micrometres": int(values[0]),
            "initial_contact_mode": int(modes[0, side]),
            "minimum_micrometres": int(values[minimum_index]),
            "minimum_reference_frame": int(frames[minimum_index]),
            "maximum_downward_step_micrometres": int(
                downward[downward_index - 1]
            ),
            "maximum_downward_step_reference_frame": int(
                frames[downward_index]
            ),
            "active_support_gap_frame_count": int(
                np.count_nonzero(
                    active
                    & (
                        values
                        >= int(thresholds["active_support_gap_micrometres"])
                    )
                )
            ),
            "flight_near_ground_frame_count": int(
                np.count_nonzero(
                    flight
                    & (
                        values
                        <= int(thresholds["flight_near_ground_micrometres"])
                    )
                )
            ),
            "by_frame_micrometres": values.tolist(),
        }
    return output


def _findings(
    case_records: Sequence[Mapping[str, Any]],
    thresholds: Mapping[str, Any],
) -> dict[str, Any]:
    groups: dict[str, list[Mapping[str, Any]]] = {}
    for row in case_records:
        groups.setdefault(str(row["classification"]), []).append(row)
    group_summary = {}
    for name, rows in sorted(groups.items()):
        group_summary[name] = {
            "case_ordinals": [int(row["case_ordinal"]) for row in rows],
            "median_joint_acceleration_v9_over_v7_basis_points": _median_int(
                [
                    int(
                        row["comparison"][
                            "joint_acceleration_v9_over_v7_basis_points"
                        ]
                    )
                    for row in rows
                ]
            ),
            "median_joint_jerk_v9_over_v7_basis_points": _median_int(
                [
                    int(
                        row["comparison"][
                            "joint_jerk_v9_over_v7_basis_points"
                        ]
                    )
                    for row in rows
                ]
            ),
        }
    derivative_amplified = [
        int(row["case_ordinal"])
        for row in case_records
        if row["comparison"]["derivative_amplified"]
    ]
    new_regressions = [
        int(row["case_ordinal"])
        for row in case_records
        if row["classification"] == "new_control_regression"
    ]
    new_regression_derivative = [
        int(row["case_ordinal"])
        for row in case_records
        if row["classification"] == "new_control_regression"
        and row["comparison"]["derivative_amplified"]
    ]
    early_geometry = [
        {
            "case_ordinal": int(row["case_ordinal"]),
            "flags": row["comparison"]["impacted_foot_geometry_flags"],
        }
        for row in case_records
        if row["r94"]["early_hard_impact"]
        and row["comparison"]["impacted_foot_geometry_flags"]
    ]
    selected = select_counterfactual_cases(
        case_records,
        maximum_count=int(thresholds["maximum_selected_fresh_cases"]),
    )
    return {
        "group_summary": group_summary,
        "new_control_regression_case_ordinals": new_regressions,
        "derivative_amplified_case_ordinals": derivative_amplified,
        "derivative_amplified_new_control_regression_case_ordinals": (
            new_regression_derivative
        ),
        "early_impact_contact_geometry_discriminators": early_geometry,
        "selected_counterfactual_case_ordinals": selected,
        "hypothesis_assessment": {
            "H22_dynamic_demand": {
                "status": "SUPPORTED_FOR_BOUNDED_DISCRIMINATOR",
                "evidence_case_ordinals": new_regression_derivative,
            },
            "H23_contact_geometry": {
                "status": "SUPPORTED_FOR_BOUNDED_DISCRIMINATOR",
                "evidence_case_ordinals": [
                    row["case_ordinal"] for row in early_geometry
                ],
            },
            "H24_nonlocal_control_regression": {
                "status": "SUPPORTED_FOR_BOUNDED_DISCRIMINATOR",
                "evidence_case_ordinals": new_regressions,
            },
            "H25_reset_mismatch": {
                "status": "REJECTED_BY_R94",
                "evidence_case_ordinals": list(range(len(case_records))),
            },
        },
    }


def _r94_outcome(
    row: Mapping[str, Any], case: ContactPrototypeCase
) -> dict[str, Any]:
    violation = row.get("first_required_safety_violation")
    if (
        int(row.get("case_ordinal", -1)) != case.ordinal
        or row.get("clip_id") != case.clip_id
        or int(row.get("start_frame", -1)) != case.frame_first
    ):
        raise ValueError("R94 outcome differs from prototype case")
    reasons = sorted(violation.get("reasons", ())) if violation else []
    tick = int(violation["terminal_motor_tick"]) if violation else None
    impacted_sides = sorted(
        {
            side
            for pair in (violation or {}).get("contact_pairs", ())
            for side in _SIDES
            if f".{side}-" in str(pair.get("pair_id", ""))
        }
    )
    return {
        "status": str(row["status"]),
        "reasons": reasons,
        "terminal_motor_tick": tick,
        "reference_frame": (
            int(violation["reference_frame"]) if violation else None
        ),
        "impacted_sides": impacted_sides,
        "early_hard_impact": bool(
            "hard_impact" in reasons and tick is not None and tick <= 2
        ),
    }


def _r94_initial_state_summary(
    report: Mapping[str, Any],
) -> dict[str, int]:
    workers = report.get("fresh_scene", {}).get("workers", ())
    if not workers:
        raise ValueError("R94 initial-state worker evidence is absent")
    after_reset = [
        worker.get("initial_state_verification", {}).get("after_reset", {})
        for worker in workers
    ]
    return {
        "worker_count": len(workers),
        "maximum_root_position_error_micrometres": max(
            int(row["maximum_root_position_error_micrometres"])
            for row in after_reset
        ),
        "maximum_root_linear_velocity_error_micrometres_per_second": max(
            int(
                row[
                    "maximum_root_linear_velocity_error_micrometres_per_second"
                ]
            )
            for row in after_reset
        ),
        "maximum_root_angular_velocity_error_microradians_per_second": max(
            int(
                row[
                    "maximum_root_angular_velocity_error_microradians_per_second"
                ]
            )
            for row in after_reset
        ),
        "maximum_joint_position_error_microradians": max(
            int(row["maximum_joint_position_error_microradians"])
            for row in after_reset
        ),
        "maximum_joint_velocity_error_microradians_per_second": max(
            int(row["maximum_joint_velocity_error_microradians_per_second"])
            for row in after_reset
        ),
        "minimum_root_orientation_absolute_dot_q1_30": min(
            int(row["minimum_root_orientation_absolute_dot_q1_30"])
            for row in after_reset
        ),
    }


def _impacted_foot_geometry_flags(
    *,
    r94: Mapping[str, Any],
    v7_metrics: Mapping[str, Any],
    v9_metrics: Mapping[str, Any],
    thresholds: Mapping[str, Any],
) -> list[dict[str, Any]]:
    flags = []
    for side in r94["impacted_sides"]:
        v7 = v7_metrics["foot_clearance"][side]
        v9 = v9_metrics["foot_clearance"][side]
        mode = int(v9["initial_contact_mode"])
        value = int(v9["initial_micrometres"])
        kind = None
        if mode != 0 and value >= int(
            thresholds["active_support_gap_micrometres"]
        ):
            kind = "active_support_gap"
        elif mode == 0 and value <= int(
            thresholds["flight_near_ground_micrometres"]
        ):
            kind = "flight_near_ground"
        if kind is not None:
            flags.append(
                {
                    "side": side,
                    "kind": kind,
                    "v7_initial_micrometres": int(
                        v7["initial_micrometres"]
                    ),
                    "v9_initial_micrometres": value,
                    "v9_initial_contact_mode": mode,
                }
            )
    return flags


def _validate_inputs(
    *,
    profile: Mapping[str, Any],
    profile_path: Path,
    source_audit_path: Path,
    descriptor: Mapping[str, Any],
    descriptor_path: Path,
    v7_manifest: Mapping[str, Any],
    v7_manifest_path: Path,
    v7_cases: Sequence[ContactPrototypeCase],
    v9_manifest: Mapping[str, Any],
    v9_manifest_path: Path,
    v9_cases: Sequence[ContactPrototypeCase],
    r94_report: Mapping[str, Any],
    r94_report_path: Path,
    repository: Mapping[str, Any],
) -> None:
    identities = profile.get("input_identities", {})
    report_without_hash = dict(r94_report)
    embedded_r94_hash = report_without_hash.pop("manifest_sha256", None)
    phase_results = r94_report.get("fresh_scene", {}).get("results", {}).get(
        "phase_results", ()
    )
    initial_state = _r94_initial_state_summary(r94_report)
    expected_case_count = int(profile.get("scope", {}).get("case_count", -1))
    if (
        profile.get("schema_version") != 1
        or profile.get("audit_id") != AUDIT_ID
        or profile.get("claim") != "OptimizerFreeReportOnlyResearch"
        or sha256(source_audit_path)
        != identities.get("source_audit_file_sha256")
        or sha256(descriptor_path)
        != identities.get("descriptor_file_sha256")
        or v7_manifest.get("prototype_id") != V7_PROTOTYPE_ID
        or v9_manifest.get("prototype_id") != V9_PROTOTYPE_ID
        or v7_manifest.get("manifest_sha256")
        != identities.get("v7_manifest_sha256")
        or sha256(v7_manifest_path)
        != identities.get("v7_manifest_file_sha256")
        or v9_manifest.get("manifest_sha256")
        != identities.get("v9_manifest_sha256")
        or sha256(v9_manifest_path)
        != identities.get("v9_manifest_file_sha256")
        or embedded_r94_hash != identities.get("r94_report_sha256")
        or sha256(r94_report_path)
        != identities.get("r94_report_file_sha256")
        or hashlib.sha256(canonical_json(report_without_hash)).hexdigest()
        != embedded_r94_hash
        or r94_report.get("status") != "FAIL"
        or r94_report.get("gate_decision") != "STOP_AND_RESEARCH"
        or r94_report.get("optimizer_steps") != 0
        or r94_report.get("training_runs") != 0
        or r94_report.get("indexed_partial_reset", {}).get("status")
        != "NOT_RUN"
        or len(v7_cases) != expected_case_count
        or len(v9_cases) != expected_case_count
        or len(phase_results) != expected_case_count
        or initial_state["worker_count"] != expected_case_count
        or initial_state["maximum_root_position_error_micrometres"] > 1
        or initial_state[
            "maximum_root_linear_velocity_error_micrometres_per_second"
        ]
        != 0
        or initial_state[
            "maximum_root_angular_velocity_error_microradians_per_second"
        ]
        > 1
        or initial_state["maximum_joint_position_error_microradians"] != 0
        or initial_state[
            "maximum_joint_velocity_error_microradians_per_second"
        ]
        != 0
        or initial_state["minimum_root_orientation_absolute_dot_q1_30"]
        < (1 << 30) - 1
        or int(descriptor.get("motor_hz", -1))
        != int(profile.get("scope", {}).get("motor_hz", -2))
        or descriptor.get("compiled_descriptor_hash") is None
        or v7_manifest.get("identities", {}).get("descriptor_sha256")
        != sha256(descriptor_path)
        or v9_manifest.get("identities", {}).get("descriptor_sha256")
        != sha256(descriptor_path)
        or profile.get("scope", {}).get("physx_execution") is not False
        or profile.get("scope", {}).get("trajectory_mutation") is not False
        or profile.get("scope", {}).get("optimizer_steps") != 0
        or profile.get("scope", {}).get("training_runs") != 0
        or profile.get("decision", {}).get("full_v19") != "FORBIDDEN"
        or profile.get("decision", {}).get("training") != "FORBIDDEN"
        or profile.get("scope", {}).get("clean_repository_required") is not True
        or repository.get("dirty") is not False
        or not isinstance(repository.get("commit"), str)
    ):
        raise ValueError("native-dynamics audit lineage is invalid")


def _validate_case_pair(
    v7_case: ContactPrototypeCase,
    v9_case: ContactPrototypeCase,
    v7_arrays: Mapping[str, NDArray[Any]],
    v9_arrays: Mapping[str, NDArray[Any]],
) -> None:
    identity = (
        v7_case.ordinal,
        v7_case.source_case_ordinal,
        v7_case.clip_id,
        v7_case.split,
        v7_case.frame_first,
        v7_case.frame_last,
        v7_case.baseline_status,
        v7_case.baseline_reasons,
    )
    other = (
        v9_case.ordinal,
        v9_case.source_case_ordinal,
        v9_case.clip_id,
        v9_case.split,
        v9_case.frame_first,
        v9_case.frame_last,
        v9_case.baseline_status,
        v9_case.baseline_reasons,
    )
    required = (
        "reference_frame",
        "root_position_um",
        "root_quaternion_q1_30",
        "root_linear_velocity_um_s",
        "joint_position_urad",
        "joint_velocity_urad_s",
        "contacts",
        "contact_modes",
    )
    if (
        identity != other
        or any(name not in v7_arrays or name not in v9_arrays for name in required)
        or any(v7_arrays[name].shape != v9_arrays[name].shape for name in required)
        or not np.array_equal(
            v7_arrays["reference_frame"], v9_arrays["reference_frame"]
        )
    ):
        raise ValueError(f"native-dynamics case pair {v7_case.ordinal} differs")


def _ordered_actuators(
    descriptor: Mapping[str, Any], width: int
) -> tuple[Mapping[str, Any], ...]:
    by_dof = {
        int(item["dof_ordinal"]): item for item in descriptor.get("actuators", ())
    }
    if set(by_dof) != set(range(width)):
        raise ValueError("native-dynamics actuator order is invalid")
    return tuple(by_dof[index] for index in range(width))


def _joint_ids(descriptor: Mapping[str, Any]) -> tuple[str, ...]:
    joints = {
        int(item["dof_ordinal"]): str(item["joint_id"])
        for item in descriptor.get("joints", ())
    }
    if set(joints) != set(range(len(joints))):
        raise ValueError("native-dynamics joint order is invalid")
    return tuple(joints[index] for index in range(len(joints)))


def _foot_colliders(
    descriptor: Mapping[str, Any],
) -> tuple[tuple[tuple[int, Mapping[str, Any]], ...], ...]:
    bodies = {str(item["body_id"]): item for item in descriptor.get("bodies", ())}
    output = []
    for side in _SIDES:
        body = bodies.get(f"body.{side}-ankle-roll")
        colliders = (
            tuple(
                (int(body["body_slot"]), collider)
                for collider in body.get("colliders", ())
                if int(collider.get("contact_role", -1)) == 8
            )
            if body is not None
            else ()
        )
        if not colliders:
            raise ValueError("native-dynamics foot collider is absent")
        output.append(colliders)
    return tuple(output)


def _maximum_record(
    values: NDArray[np.integer[Any]],
    frames: NDArray[np.int64],
    labels: Sequence[str],
) -> dict[str, Any]:
    array = np.asarray(values, dtype=np.int64)
    if array.ndim != 2 or array.shape != (len(frames), len(labels)):
        raise ValueError("native-dynamics maximum record shape mismatch")
    index = np.unravel_index(int(np.argmax(np.abs(array))), array.shape)
    value = int(array[index])
    return {
        "absolute_value": abs(value),
        "signed_value": value,
        "relative_frame": int(index[0]),
        "reference_frame": int(frames[index[0]]),
        "channel": str(labels[index[1]]),
    }


def _difference_record(
    left: NDArray[Any],
    right: NDArray[Any],
    frames: NDArray[np.int64],
    labels: Sequence[str],
) -> dict[str, Any]:
    return _maximum_record(
        np.asarray(left, dtype=np.int64) - np.asarray(right, dtype=np.int64),
        np.asarray(frames, dtype=np.int64),
        labels,
    )


def _effort_record(
    *,
    effort: NDArray[np.int64],
    utilization: NDArray[np.int64],
    frames: NDArray[np.int64],
    joint_ids: Sequence[str],
    value_name: str,
) -> dict[str, Any]:
    index = np.unravel_index(
        int(np.argmax(utilization)), utilization.shape
    )
    return {
        "utilization_basis_points": int(utilization[index]),
        f"absolute_{value_name}": abs(int(effort[index])),
        f"signed_{value_name}": int(effort[index]),
        "relative_frame": int(index[0]),
        "reference_frame": int(frames[index[0]]),
        "joint_id": str(joint_ids[index[1]]),
    }


def _classification(baseline_status: str, r94_status: str) -> str:
    if baseline_status == "PASS" and r94_status == "FAIL":
        return "new_control_regression"
    if baseline_status == "FAIL" and r94_status == "PASS":
        return "source_failure_repaired"
    if baseline_status == "FAIL" and r94_status == "FAIL":
        return "source_failure_persisting_or_changed"
    if baseline_status == "PASS" and r94_status == "PASS":
        return "stable_passing_control"
    raise ValueError("native-dynamics outcome status is invalid")


def _ratio_basis_points(numerator: int, denominator: int) -> int:
    if numerator < 0 or denominator < 0:
        raise ValueError("native-dynamics ratio cannot be negative")
    if denominator == 0:
        return 0 if numerator == 0 else 2_147_483_647
    return (numerator * 10_000 + denominator // 2) // denominator


def _ceil_ratio(
    numerator: NDArray[np.int64], denominator: NDArray[np.int64]
) -> NDArray[np.int64]:
    return (numerator + denominator - 1) // denominator


def _median_int(values: Sequence[int]) -> int:
    ordered = sorted(int(value) for value in values)
    if not ordered:
        raise ValueError("native-dynamics median is empty")
    middle = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle] + 1) // 2
