from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

RESEARCH_ID = "nextengine.humanoid-post-r130-projected-schedule-research.v1"
CHECK_ID = "TRAIN-4-POST-R130-PROJECTED-SCHEDULE-FAILURE-RESEARCH"
NOT_AUTHORIZED_KEYS = frozenset(
    {
        "r130_retry",
        "additional_projected_fixed_pd_execution",
        "additional_projection_execution",
        "hybrid_contact_edge_state_lift_execution",
        "additional_inverse_dynamics_execution",
        "kinodynamic_solve",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
)


def build_projected_schedule_failure_research(
    *,
    profile_path: Path,
    r130_report_path: Path,
    r130_profile_path: Path,
    r130_execution_module_path: Path,
    r130_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Hash-close the immutable R130 schedule failure without recomputation."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r130_report_path,
            r130_profile_path,
            r130_execution_module_path,
            r130_tool_path,
            r121_report_path,
            r121_profile_path,
            tool_path,
        )
    )
    (
        profile_path,
        r130_report_path,
        r130_profile_path,
        r130_execution_module_path,
        r130_tool_path,
        r121_report_path,
        r121_profile_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("post-R130 research input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r130 = _load_bound_report(r130_report_path, profile["source"]["r130"], "R130")
    r121 = _load_bound_report(r121_report_path, profile["source"]["r121"], "R121")
    _validate_source_files(
        profile=profile,
        r130_profile_path=r130_profile_path,
        r130_execution_module_path=r130_execution_module_path,
        r130_tool_path=r130_tool_path,
        r121_profile_path=r121_profile_path,
    )
    descriptor_sha256 = hashlib.sha256(descriptor_bytes).hexdigest()
    if (
        descriptor_sha256 != profile["source"]["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve())
        != profile["source"]["research_module_sha256"]
        or sha256(tool_path) != profile["source"]["tool_sha256"]
    ):
        raise ValueError("post-R130 current research identity differs")
    validations = _validate_results(profile, validation_results)

    descriptor = json.loads(descriptor_bytes)
    failed_channel = _descriptor_channel(
        descriptor, int(profile["discriminators"]["expected_failed_dof_ordinal"])
    )
    projection_audit = analyze_projection_hotspots(
        r130["projection_result"]["collocations"],
        hotspot_count=int(profile["discriminators"]["hotspot_count"]),
    )
    r121_schedule = r121["fixed_pd_schedule_audit"]
    r130_schedule = r130["projected_fixed_pd_schedule_audit"]
    discriminators = _build_discriminators(
        profile=profile,
        r130=r130,
        r121_schedule=r121_schedule,
        r130_schedule=r130_schedule,
        failed_channel=failed_channel,
        projection_audit=projection_audit,
    )
    confirmed = all(discriminators.values())
    finding = (
        "CONFIRMED_PROJECTED_SCHEDULE_ACTUATOR_CONFLICT_WITH_HYBRID_EXIT_HOTSPOTS"
        if confirmed
        else "INCONCLUSIVE_RETAIN_STOP_WITHOUT_EXECUTION"
    )
    peak_velocity = int(
        failed_channel["joint"]["maximum_velocity_microradians_per_second"]
    ) + int(r130_schedule["maxima"]["velocity_excess_microradians_per_second"])
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "research_id": RESEARCH_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "finding": finding,
        "gate_decision": profile["decision"][
            "confirmed" if confirmed else "inconclusive"
        ],
        "scope": profile["scope"],
        "r130_result": {
            "status": r130["status"],
            "gate_decision": r130["gate_decision"],
            "result_transition": r130["result_transition"],
            "report_sha256": r130["report_sha256"],
            "invalid_reason": r130["solver_result"]["invalid_reason"],
            "projected_fixed_pd_execution_runs": r130[
                "projected_fixed_pd_execution_runs"
            ],
            "state_projection_systems": r130["state_projection_systems"],
            "inverse_dynamics_system_assemblies": r130[
                "inverse_dynamics_system_assemblies"
            ],
            "force_gauge_interval_classifications": r130[
                "force_gauge_interval_classifications"
            ],
            "solver_private_cache": r130["solver_private_cache"],
        },
        "controller_schedule_ab": {
            "baseline_r121": r121_schedule,
            "projected_r130": r130_schedule,
            "unchanged_target_lineage": {
                "hard_rom_violation_count": r130_schedule["activation_counts"][
                    "hard_rom_violation"
                ],
                "target_soft_clamp_count": r130_schedule["activation_counts"][
                    "target_soft_clamp"
                ],
                "target_slew_count": r130_schedule["activation_counts"]["target_slew"],
                "target_slew_dof_ordinals": r130_schedule["activated_dof_ordinals"][
                    "target_slew"
                ],
                "maximum_target_lag_microradians": r130_schedule["maxima"][
                    "target_lag_microradians"
                ],
            },
            "new_blocking_events": {
                "velocity_violation_count": r130_schedule["activation_counts"][
                    "velocity_violation"
                ],
                "velocity_violation_dof_ordinals": r130_schedule[
                    "activated_dof_ordinals"
                ]["velocity_violation"],
                "infeasible_effort_envelope_count": r130_schedule["activation_counts"][
                    "infeasible_effort_envelope"
                ],
                "infeasible_effort_envelope_dof_ordinals": r130_schedule[
                    "activated_dof_ordinals"
                ]["infeasible_effort_envelope"],
                "implied_peak_absolute_velocity_microradians_per_second": peak_velocity,
            },
        },
        "failed_descriptor_channel": failed_channel,
        "projection_hotspot_audit": projection_audit,
        "evidence_boundary": {
            "confirmed": "the otherwise numerically valid R130 tangent projection creates an invalid fixed-PD input schedule on descriptor DoF 11 before inverse dynamics",
            "supported_hotspot": "the four largest recorded generalized corrections cluster at substeps 2/3 of two right-forefoot intervals immediately followed by flight",
            "not_available": "R130 emitted per-row projected-velocity hashes but no values or controller-event row indices, so exact row-to-velocity-violation identity is not claimed",
            "not_supported": "R130 provides no inverse-dynamics equality or point-force cone feasibility result",
        },
        "discriminators": discriminators,
        "hypothesis_disposition": profile["hypothesis_disposition"],
        "research_basis": profile["research_basis"],
        "repair_alternatives": profile["repair_alternatives"],
        "next_smallest_action": profile["next_smallest_action"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r130_report_file_sha256": sha256(r130_report_path),
            "r130_profile_sha256": sha256(r130_profile_path),
            "r130_execution_module_sha256": sha256(r130_execution_module_path),
            "r130_tool_sha256": sha256(r130_tool_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "current_descriptor_file_sha256": descriptor_sha256,
            "research_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "research_audits": 1,
        "descriptor_channel_lookups": 1,
        "state_projection_systems": 0,
        "projection_factorizations": 0,
        "projection_solves": 0,
        "inverse_dynamics_system_assemblies": 0,
        "singular_value_decompositions": 0,
        "particular_solutions": 0,
        "gauge_interval_classifications": 0,
        "local_system_solves": 0,
        "solver_runs": 0,
        "kinodynamic_solves": 0,
        "candidate_artifacts_built": 0,
        "solver_private_caches_built": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def analyze_projection_hotspots(
    collocations: Sequence[Mapping[str, Any]], *, hotspot_count: int
) -> dict[str, Any]:
    if hotspot_count <= 0:
        raise ValueError("hotspot_count must be positive")
    if not collocations:
        raise ValueError("projection collocations are absent")
    by_collocation: dict[int, Mapping[str, Any]] = {}
    projected: list[Mapping[str, Any]] = []
    for expected_ordinal, row in enumerate(collocations):
        ordinal = int(row.get("collocation", -1))
        if ordinal != expected_ordinal or ordinal in by_collocation:
            raise ValueError("projection collocation order differs")
        by_collocation[ordinal] = row
        if row.get("projection") is not None:
            projected.append(row)
    if len(projected) < hotspot_count:
        raise ValueError("too few projected collocations")
    ordered = sorted(
        projected,
        key=lambda row: float(
            row["projection"]["maximum_generalized_velocity_correction"]
        ),
        reverse=True,
    )
    hotspots = [
        _normalize_hotspot(row=row, by_collocation=by_collocation)
        for row in ordered[:hotspot_count]
    ]
    return {
        "collocation_rows_read": len(collocations),
        "projected_rows_ranked": len(projected),
        "hotspot_count": hotspot_count,
        "hotspots": hotspots,
        "next_largest_generalized_velocity_correction": float(
            ordered[hotspot_count]["projection"][
                "maximum_generalized_velocity_correction"
            ]
        )
        if len(ordered) > hotspot_count
        else None,
        "distinct_hotspot_intervals": sorted(
            {int(row["interval"]) for row in hotspots}
        ),
        "all_hotspots_are_contact_exit_rows": all(
            row["contact_exit_before_next_interval"] for row in hotspots
        ),
        "exact_controller_event_row_mapping_available": False,
    }


def _normalize_hotspot(
    *, row: Mapping[str, Any], by_collocation: Mapping[int, Mapping[str, Any]]
) -> dict[str, Any]:
    interval = int(row["interval"])
    next_boundary_ordinal = (interval + 1) * 4
    next_boundary = by_collocation.get(next_boundary_ordinal)
    if next_boundary is None or int(next_boundary.get("substep", -1)) != 0:
        raise ValueError("projection next interval boundary is absent")
    contact_modes = [int(value) for value in row["contact_modes"]]
    next_modes = [int(value) for value in next_boundary["contact_modes"]]
    active_to_flight = any(
        current != 0 and following == 0
        for current, following in zip(contact_modes, next_modes, strict=True)
    )
    projection = row["projection"]
    return {
        "collocation": int(row["collocation"]),
        "interval": interval,
        "substep": int(row["substep"]),
        "contact_modes": contact_modes,
        "active_point_ordinals": [int(value) for value in row["active_point_ordinals"]],
        "maximum_generalized_velocity_correction": float(
            projection["maximum_generalized_velocity_correction"]
        ),
        "maximum_active_velocity_before_metres_per_second": float(
            projection["maximum_active_velocity_before_metres_per_second"]
        ),
        "next_interval_contact_modes": next_modes,
        "contact_exit_before_next_interval": active_to_flight,
    }


def _descriptor_channel(descriptor: Mapping[str, Any], ordinal: int) -> dict[str, Any]:
    joints = [
        row for row in descriptor.get("joints", ()) if row.get("dof_ordinal") == ordinal
    ]
    actuators = [
        row
        for row in descriptor.get("actuators", ())
        if row.get("dof_ordinal") == ordinal
    ]
    if len(joints) != 1 or len(actuators) != 1:
        raise ValueError("descriptor failed channel identity differs")
    joint = joints[0]
    actuator = actuators[0]
    if joint.get("joint_id") != actuator.get("joint_id"):
        raise ValueError("descriptor joint/actuator channel differs")
    return {
        "dof_ordinal": ordinal,
        "joint": {
            "joint_id": joint["joint_id"],
            "anatomical_semantic_id": joint["anatomical_semantic_id"],
            "maximum_velocity_microradians_per_second": int(
                joint["maximum_velocity_microradians_per_second"]
            ),
            "hard_limit_microradians": [
                int(value) for value in joint["hard_limit_microradians"]
            ],
        },
        "actuator": {
            "actuator_id": actuator["actuator_id"],
            "joint_id": actuator["joint_id"],
            "stiffness_q16": int(actuator["stiffness_q16"]),
            "damping_q16": int(actuator["damping_q16"]),
            "effort_micronewton_metres": [
                int(value) for value in actuator["effort_micronewton_metres"]
            ],
            "maximum_effort_rate_micronewton_metres_per_second": int(
                actuator["maximum_effort_rate_micronewton_metres_per_second"]
            ),
            "maximum_power_microwatts": int(actuator["maximum_power_microwatts"]),
            "maximum_positive_work_microjoules_per_motor_tick": int(
                actuator["maximum_positive_work_microjoules_per_motor_tick"]
            ),
        },
    }


def _build_discriminators(
    *,
    profile: Mapping[str, Any],
    r130: Mapping[str, Any],
    r121_schedule: Mapping[str, Any],
    r130_schedule: Mapping[str, Any],
    failed_channel: Mapping[str, Any],
    projection_audit: Mapping[str, Any],
) -> dict[str, bool]:
    expected = profile["discriminators"]
    r130_counts = r130_schedule["activation_counts"]
    r130_channels = r130_schedule["activated_dof_ordinals"]
    r121_counts = r121_schedule["activation_counts"]
    peak_velocity = int(
        failed_channel["joint"]["maximum_velocity_microradians_per_second"]
    ) + int(r130_schedule["maxima"]["velocity_excess_microradians_per_second"])
    tolerance = float(expected["numeric_absolute_tolerance"])
    expected_hotspots = expected["expected_hotspots"]
    actual_hotspots = projection_audit["hotspots"]
    hotspots_match = len(actual_hotspots) == len(expected_hotspots) and all(
        _hotspot_matches(actual, wanted, tolerance=tolerance)
        for actual, wanted in zip(actual_hotspots, expected_hotspots, strict=True)
    )
    return {
        "r130_is_clean_single_consumed_invalid_execution": (
            r130["repository"]["commit"]
            == profile["source"]["r130"]["repository_commit"]
            and r130["repository"]["dirty"] is False
            and r130["projected_fixed_pd_execution_runs"] == 1
            and r130["status"] == "INVALID"
            and r130["gate_decision"] == "STOP_INVALID_EVIDENCE_WITHOUT_RESTART"
            and r130["result_transition"] == "R130_CONSUMED_INVALID_NO_RETRY"
            and r130["solver_result"]["invalid_reason"]
            == "PROJECTED_FIXED_PD_SCHEDULE_INVALID"
        ),
        "projection_completed_and_passed_before_controller_failure": (
            r130["projection_result"]["status"] == "PASS"
            and r130["projection_result"]["aggregate"]["collocation_rows_recorded"]
            == expected["expected_projection_collocation_count"]
            and r130["projection_result"]["aggregate"]["passing_collocations"]
            == expected["expected_projection_collocation_count"]
            and r130["projection_result"]["aggregate"]["first_failing_collocation"]
            is None
            and r130["state_projection_systems"]
            == expected["expected_projection_system_count"]
        ),
        "r121_preprojection_schedule_is_valid": (
            r121_schedule["status"] == "PASS"
            and r121_counts["hard_rom_violation"] == 0
            and r121_counts["velocity_violation"] == 0
            and r121_counts["infeasible_effort_envelope"] == 0
        ),
        "target_and_rom_lineage_remains_unchanged": (
            r130_counts["hard_rom_violation"] == 0
            and r130_counts["target_soft_clamp"] == 0
            and r130_counts["target_slew"] == r121_counts["target_slew"] == 1
            and r130_channels["target_slew"]
            == r121_schedule["activated_dof_ordinals"]["target_slew"]
            == [6]
            and r130_schedule["maxima"]["target_lag_microradians"]
            == r121_schedule["maxima"]["target_lag_microradians"]
            == 1672.0
        ),
        "projected_schedule_has_expected_blocking_counts": (
            r130_schedule["status"] == "FAIL"
            and r130_counts["velocity_violation"]
            == expected["expected_velocity_violation_count"]
            and r130_counts["infeasible_effort_envelope"]
            == expected["expected_infeasible_effort_envelope_count"]
            and r130_channels["velocity_violation"]
            == [expected["expected_failed_dof_ordinal"]]
            and r130_channels["infeasible_effort_envelope"]
            == [expected["expected_failed_dof_ordinal"]]
        ),
        "failed_channel_is_exact_right_ankle_roll_descriptor_channel": (
            failed_channel["dof_ordinal"] == expected["expected_failed_dof_ordinal"]
            and failed_channel["joint"]["joint_id"]
            == expected["expected_failed_joint_id"]
            and failed_channel["actuator"]["actuator_id"]
            == expected["expected_failed_actuator_id"]
            and failed_channel["joint"]["maximum_velocity_microradians_per_second"]
            == expected["expected_maximum_velocity_microradians_per_second"]
        ),
        "peak_velocity_excess_reconstructs_expected_absolute_speed": (
            r130_schedule["maxima"]["velocity_excess_microradians_per_second"]
            == expected["expected_peak_velocity_excess_microradians_per_second"]
            and peak_velocity
            == expected[
                "expected_implied_peak_absolute_velocity_microradians_per_second"
            ]
        ),
        "four_predeclared_projection_hotspots_match": hotspots_match,
        "hotspots_form_two_right_forefoot_exit_pairs": (
            projection_audit["distinct_hotspot_intervals"] == [76, 744]
            and projection_audit["all_hotspots_are_contact_exit_rows"] is True
            and all(
                row["contact_modes"] == [0, 2]
                and row["active_point_ordinals"] == [3]
                and row["next_interval_contact_modes"] == [0, 0]
                and row["substep"] in (2, 3)
                for row in actual_hotspots
            )
            and min(
                row["maximum_generalized_velocity_correction"]
                for row in actual_hotspots
            )
            > expected["hotspot_minimum_generalized_velocity_correction"]
            and projection_audit["next_largest_generalized_velocity_correction"]
            < expected["next_largest_correction_maximum"]
        ),
        "exact_row_to_controller_event_mapping_is_not_claimed": (
            projection_audit["exact_controller_event_row_mapping_available"] is False
            and profile["method"]["causal_claim"].startswith(
                "confirm the projected schedule actuator conflict"
            )
        ),
        "inverse_dynamics_and_downstream_work_never_started": (
            r130["inverse_dynamics_system_assemblies"]
            == expected["expected_inverse_dynamics_system_count"]
            and r130["inverse_dynamics_singular_value_decompositions"] == 0
            and r130["inverse_dynamics_particular_solutions"] == 0
            and r130["force_gauge_interval_classifications"] == 0
            and r130["kinodynamic_solves"] == 0
            and r130["physx_scene_runs"] == 0
            and r130["training_runs"] == 0
            and r130["solver_private_cache"]["status"]
            == "NOT_EMITTED_INVALID_EXECUTION"
        ),
        "research_reconstructs_or_solves_no_state_or_dynamics_system": True,
    }


def _hotspot_matches(
    actual: Mapping[str, Any], wanted: Mapping[str, Any], *, tolerance: float
) -> bool:
    exact_keys = (
        "collocation",
        "interval",
        "substep",
        "contact_modes",
        "active_point_ordinals",
        "next_interval_contact_modes",
    )
    numeric_keys = (
        "maximum_generalized_velocity_correction",
        "maximum_active_velocity_before_metres_per_second",
    )
    return all(actual[key] == wanted[key] for key in exact_keys) and all(
        abs(float(actual[key]) - float(wanted[key])) <= tolerance
        for key in numeric_keys
    )


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    method = profile.get("method", {})
    bounded = profile.get("bounded_acceptance", {})
    expected = profile.get("discriminators", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("research_id") != RESEARCH_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "PostR130ProjectedFixedPdScheduleFailureResearchOnly"
        or set(source)
        != {
            "r130",
            "r121",
            "current_descriptor_file_sha256",
            "research_module_sha256",
            "tool_sha256",
        }
        or scope.get("research_cycle_id") != "R130-RC1"
        or scope.get("source_r130_execution_runs") != 1
        or scope.get("source_projection_collocations_read") != 3200
        or scope.get("source_controller_schedule_audits_read") != 2
        or scope.get("research_audits") != 1
        or scope.get("descriptor_channel_lookups") != 1
        or any(
            scope.get(key) != 0
            for key in (
                "state_projection_systems",
                "projection_factorizations",
                "projection_solves",
                "inverse_dynamics_system_assemblies",
                "singular_value_decompositions",
                "particular_solutions",
                "gauge_interval_classifications",
                "kinodynamic_solves",
                "physx_scene_runs",
            )
        )
        or scope.get("candidate_construction") is not False
        or scope.get("training") is not False
        or method.get("matrix_factorization_or_solve_policy") != "FORBIDDEN"
        or set(bounded) != NOT_AUTHORIZED_KEYS
        or any(value != "NOT_AUTHORIZED" for value in bounded.values())
        or expected.get("expected_projection_collocation_count") != 3200
        or expected.get("expected_projection_system_count") != 2640
        or expected.get("expected_inverse_dynamics_system_count") != 0
        or expected.get("expected_velocity_violation_count") != 4
        or expected.get("expected_infeasible_effort_envelope_count") != 2
        or expected.get("expected_failed_dof_ordinal") != 11
        or expected.get("hotspot_count") != 4
        or len(expected.get("expected_hotspots", ())) != 4
        or profile.get("decision", {}).get("confirmed")
        != "PERMIT_SEPARATE_REPORT_ONLY_R131_HYBRID_CONTACT_EDGE_STATE_LIFT_FORMULATION_ONLY"
        or profile.get("decision", {}).get("inconclusive")
        != "STOP_AND_RESEARCH_WITHOUT_EXECUTION"
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != (
            "ruff_check",
            "ruff_format",
            "no_numeric_or_solver_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("post-R130 research profile differs")


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r130_profile_path: Path,
    r130_execution_module_path: Path,
    r130_tool_path: Path,
    r121_profile_path: Path,
) -> None:
    source = profile["source"]
    if (
        sha256(r130_profile_path) != source["r130"]["profile_sha256"]
        or sha256(r130_execution_module_path)
        != source["r130"]["execution_module_sha256"]
        or sha256(r130_tool_path) != source["r130"]["tool_sha256"]
        or sha256(r121_profile_path) != source["r121"]["profile_sha256"]
    ):
        raise ValueError("post-R130 bound source file identity differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"{label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"{label} canonical report identity differs")
    return report


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("post-R130 research requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("post-R130 validation results differ")
    return normalized


def canonical_json(value: Mapping[str, Any]) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
