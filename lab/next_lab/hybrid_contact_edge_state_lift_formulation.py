from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

FORMULATION_ID = "nextengine.humanoid-hybrid-contact-edge-state-lift-formulation.v1"
CHECK_ID = "TRAIN-4-HYBRID-CONTACT-EDGE-STATE-LIFT-FORMULATION"
MOTOR_INTERVAL_COUNT = 800
SUBSTEPS_PER_INTERVAL = 4
COLLOCATION_COUNT = MOTOR_INTERVAL_COUNT * SUBSTEPS_PER_INTERVAL
MODE_FLIGHT = 0
MODE_FOREFOOT = 2
MODE_FLAT = 3
ALTERNATIVE_DECISIONS = {
    "current_affine_cross_mode_velocity_lift": (
        "REJECT_AS_R130_EXECUTION_INPUT_RETAIN_AS_BASELINE"
    ),
    "exit_mode_owned_left_velocity_trace_hold": "SELECT_R132_BOUNDED_CONFORMANCE",
    "instantaneous_actuator_constrained_tangent_projection": (
        "DEFER_AFTER_EDGE_CAUSAL_DISCRIMINATOR"
    ),
    "full_trajectory_kinodynamic_retargeting": (
        "STRONG_FALLBACK_IF_LOCAL_DISCRIMINATOR_FAILS"
    ),
    "sliding_compliant_or_impact_contact_successor": (
        "DEFER_TO_CONTACT_SEMANTICS_AND_NATIVE_CORRESPONDENCE"
    ),
}
BOUNDED_ACCEPTANCE_KEYS = frozenset(
    {
        "r132_edge_lift_conformance",
        "r133_projected_schedule_execution",
        "r130_retry",
        "additional_projected_fixed_pd_execution",
        "additional_projection_execution",
        "additional_inverse_dynamics_execution",
        "kinodynamic_solve",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
)


def build_hybrid_contact_edge_state_lift_formulation(
    *,
    profile_path: Path,
    r130_rc1_report_path: Path,
    r130_rc1_profile_path: Path,
    r130_rc1_module_path: Path,
    r130_rc1_tool_path: Path,
    r130_report_path: Path,
    r130_profile_path: Path,
    r130_execution_module_path: Path,
    r130_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    collocation_lift_module_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R131 without evaluating a state lift or numerical system."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r130_rc1_report_path,
            r130_rc1_profile_path,
            r130_rc1_module_path,
            r130_rc1_tool_path,
            r130_report_path,
            r130_profile_path,
            r130_execution_module_path,
            r130_tool_path,
            r121_report_path,
            r121_profile_path,
            collocation_lift_module_path,
            tool_path,
        )
    )
    (
        profile_path,
        r130_rc1_report_path,
        r130_rc1_profile_path,
        r130_rc1_module_path,
        r130_rc1_tool_path,
        r130_report_path,
        r130_profile_path,
        r130_execution_module_path,
        r130_tool_path,
        r121_report_path,
        r121_profile_path,
        collocation_lift_module_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R131 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    source = profile["source"]
    r130_rc1 = _load_bound_report(r130_rc1_report_path, source["r130_rc1"], "R130-RC1")
    r130 = _load_bound_report(r130_report_path, source["r130"], "R130")
    r121 = _load_bound_report(r121_report_path, source["r121"], "R121")
    _validate_source_files(
        profile=profile,
        r130_rc1_profile_path=r130_rc1_profile_path,
        r130_rc1_module_path=r130_rc1_module_path,
        r130_rc1_tool_path=r130_rc1_tool_path,
        r130_profile_path=r130_profile_path,
        r130_execution_module_path=r130_execution_module_path,
        r130_tool_path=r130_tool_path,
        r121_profile_path=r121_profile_path,
        collocation_lift_module_path=collocation_lift_module_path,
    )
    _validate_source_contracts(
        profile=profile,
        r130_rc1=r130_rc1,
        r130=r130,
        r121=r121,
    )
    if (
        sha256(Path(__file__).resolve()) != source["formulation_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R131 current formulation identity differs")

    inventory = audit_contact_edges(
        r130["projection_result"]["collocations"],
        hotspot_intervals=profile["contact_edge_inventory_contract"][
            "right_forefoot_exit_hotspot_intervals"
        ],
    )
    if inventory != profile["contact_edge_inventory_contract"]:
        raise ValueError("R131 contact-edge inventory differs")
    alternatives = audit_alternative_decisions(profile)
    validations = _validate_results(profile, validation_results)

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "formulation_id": FORMULATION_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "result_transition": profile["result_transitions"]["complete"],
        "scope": profile["scope"],
        "source_gates": {
            "r130_rc1_status": r130_rc1["status"],
            "r130_rc1_finding": r130_rc1["finding"],
            "r130_rc1_report_sha256": r130_rc1["report_sha256"],
            "r130_status": r130["status"],
            "r130_invalid_reason": r130["solver_result"]["invalid_reason"],
            "r130_result_transition": r130["result_transition"],
            "r130_report_sha256": r130["report_sha256"],
            "r121_status": r121["status"],
            "r121_schedule_status": r121["fixed_pd_schedule_audit"]["status"],
            "r121_report_sha256": r121["report_sha256"],
        },
        "diagnosis_contract": profile["diagnosis_contract"],
        "current_cross_mode_lift_contract": profile["current_cross_mode_lift_contract"],
        "contact_edge_inventory_audit": inventory,
        "alternative_decision_audit": alternatives,
        "selected_edge_lift_contract": profile["selected_edge_lift_contract"],
        "r132_conformance_contract": profile["r132_conformance_contract"],
        "future_r133_schedule_contract": profile["future_r133_schedule_contract"],
        "research_basis": profile["research_basis"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r130_rc1_report_file_sha256": sha256(r130_rc1_report_path),
            "r130_rc1_profile_sha256": sha256(r130_rc1_profile_path),
            "r130_rc1_module_sha256": sha256(r130_rc1_module_path),
            "r130_rc1_tool_sha256": sha256(r130_rc1_tool_path),
            "r130_report_file_sha256": sha256(r130_report_path),
            "r130_profile_sha256": sha256(r130_profile_path),
            "r130_execution_module_sha256": sha256(r130_execution_module_path),
            "r130_tool_sha256": sha256(r130_tool_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "collocation_lift_module_sha256": sha256(collocation_lift_module_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "contact_edge_inventory_audits": 1,
        "hybrid_state_lift_formulations": 1,
        "state_lift_evaluations": 0,
        "mass_matrix_assemblies": 0,
        "contact_jacobian_assemblies": 0,
        "projection_factorizations": 0,
        "projection_solves": 0,
        "controller_schedule_derivations": 0,
        "inverse_dynamics_system_assemblies": 0,
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


def velocity_lift_endpoint_weights(
    *, substep: int, contact_exit: bool
) -> dict[str, int]:
    if type(substep) is not int or not 0 <= substep < SUBSTEPS_PER_INTERVAL:
        raise ValueError("R131 velocity-lift substep differs")
    if type(contact_exit) is not bool:
        raise ValueError("R131 velocity-lift edge predicate differs")
    return {
        "left_numerator": SUBSTEPS_PER_INTERVAL
        if contact_exit
        else SUBSTEPS_PER_INTERVAL - substep,
        "right_numerator": 0 if contact_exit else substep,
        "denominator": SUBSTEPS_PER_INTERVAL,
    }


def audit_contact_edges(
    collocations: Sequence[Mapping[str, Any]],
    *,
    hotspot_intervals: Sequence[int],
) -> dict[str, Any]:
    if len(collocations) != COLLOCATION_COUNT:
        raise ValueError("R131 projection collocation count differs")
    interval_modes: list[list[int]] = []
    for interval in range(MOTOR_INTERVAL_COUNT):
        start = interval * SUBSTEPS_PER_INTERVAL
        rows = collocations[start : start + SUBSTEPS_PER_INTERVAL]
        canonical_modes: list[int] | None = None
        canonical_points: list[int] | None = None
        for substep, row in enumerate(rows):
            if (
                row.get("collocation") != start + substep
                or row.get("interval") != interval
                or row.get("substep") != substep
            ):
                raise ValueError("R131 projection collocation order differs")
            modes = [int(value) for value in row.get("contact_modes", ())]
            points = [int(value) for value in row.get("active_point_ordinals", ())]
            if len(modes) != 2 or any(
                mode not in (MODE_FLIGHT, MODE_FOREFOOT, MODE_FLAT) for mode in modes
            ):
                raise ValueError("R131 contact mode differs")
            if points != _active_points_for_modes(modes):
                raise ValueError("R131 active point ownership differs")
            if canonical_modes is None:
                canonical_modes = modes
                canonical_points = points
            elif modes != canonical_modes or points != canonical_points:
                raise ValueError("R131 interval contact ownership differs")
        if canonical_modes is None:
            raise ValueError("R131 interval rows are absent")
        interval_modes.append(canonical_modes)

    entry_intervals: set[int] = set()
    exit_intervals: set[int] = set()
    active_mode_change_intervals: set[int] = set()
    changed_boundaries: set[int] = set()
    simultaneous_changes = 0
    side_counts = {
        "left": {"entry_count": 0, "exit_count": 0, "active_mode_change_count": 0},
        "right": {
            "entry_count": 0,
            "exit_count": 0,
            "active_mode_change_count": 0,
        },
    }
    transition_lookup: dict[tuple[int, int], tuple[int, int]] = {}
    for interval in range(MOTOR_INTERVAL_COUNT - 1):
        current = interval_modes[interval]
        following = interval_modes[interval + 1]
        changed_sides = 0
        for side, side_name in enumerate(("left", "right")):
            before = current[side]
            after = following[side]
            if before == after:
                continue
            changed_sides += 1
            changed_boundaries.add(interval)
            transition_lookup[(interval, side)] = (before, after)
            if before == MODE_FLIGHT and after != MODE_FLIGHT:
                entry_intervals.add(interval)
                side_counts[side_name]["entry_count"] += 1
            elif before != MODE_FLIGHT and after == MODE_FLIGHT:
                exit_intervals.add(interval)
                side_counts[side_name]["exit_count"] += 1
            elif before != MODE_FLIGHT and after != MODE_FLIGHT:
                active_mode_change_intervals.add(interval)
                side_counts[side_name]["active_mode_change_count"] += 1
            else:
                raise ValueError("R131 contact transition is unclassified")
        if changed_sides > 1:
            simultaneous_changes += 1

    normalized_hotspots = [int(value) for value in hotspot_intervals]
    if len(normalized_hotspots) != len(set(normalized_hotspots)):
        raise ValueError("R131 hotspot interval identity differs")
    if any(
        transition_lookup.get((interval, 1)) != (MODE_FOREFOOT, MODE_FLIGHT)
        for interval in normalized_hotspots
    ):
        raise ValueError("R131 hotspot is not a right-forefoot exit")

    exit_rows = len(exit_intervals) * SUBSTEPS_PER_INTERVAL
    return {
        "motor_interval_count": MOTOR_INTERVAL_COUNT,
        "collocation_count": COLLOCATION_COUNT,
        "boundary_count": MOTOR_INTERVAL_COUNT - 1,
        "unchanged_boundary_count": MOTOR_INTERVAL_COUNT - 1 - len(changed_boundaries),
        "changed_boundary_count": len(changed_boundaries),
        "simultaneous_side_change_boundary_count": simultaneous_changes,
        "contact_entry_boundary_count": sum(
            counts["entry_count"] for counts in side_counts.values()
        ),
        "contact_exit_boundary_count": sum(
            counts["exit_count"] for counts in side_counts.values()
        ),
        "active_mode_change_boundary_count": sum(
            counts["active_mode_change_count"] for counts in side_counts.values()
        ),
        "left": side_counts["left"],
        "right": side_counts["right"],
        "entry_intervals": sorted(entry_intervals),
        "exit_intervals": sorted(exit_intervals),
        "active_mode_change_intervals": sorted(active_mode_change_intervals),
        "right_forefoot_exit_hotspot_intervals": normalized_hotspots,
        "selected_exit_collocation_rows": exit_rows,
        "selected_changed_base_velocity_rows": len(exit_intervals)
        * (SUBSTEPS_PER_INTERVAL - 1),
        "motor_mode_sequence_sha256": hashlib.sha256(
            canonical_json(interval_modes)
        ).hexdigest(),
    }


def audit_alternative_decisions(profile: Mapping[str, Any]) -> dict[str, Any]:
    rows = profile.get("alternative_decisions", ())
    actual = {row.get("alternative"): row.get("decision") for row in rows}
    selected = [
        alternative
        for alternative, decision in actual.items()
        if decision == "SELECT_R132_BOUNDED_CONFORMANCE"
    ]
    if (
        len(rows) != len(ALTERNATIVE_DECISIONS)
        or actual != ALTERNATIVE_DECISIONS
        or selected != ["exit_mode_owned_left_velocity_trace_hold"]
    ):
        raise ValueError("R131 alternative decision matrix differs")
    return {
        "alternatives_compared": len(rows),
        "selected_alternative": selected[0],
        "decisions": [dict(row) for row in rows],
    }


def _active_points_for_modes(modes: Sequence[int]) -> list[int]:
    points: list[int] = []
    for side, mode in enumerate(modes):
        offset = side * 2
        if mode == MODE_FOREFOOT:
            points.append(offset + 1)
        elif mode == MODE_FLAT:
            points.extend((offset, offset + 1))
    return points


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    inventory = profile.get("contact_edge_inventory_contract", {})
    selected = profile.get("selected_edge_lift_contract", {})
    r132 = profile.get("r132_conformance_contract", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "HybridContactEdgeStateLiftFormulationOnly"
        or set(source)
        != {
            "r130_rc1",
            "r130",
            "r121",
            "collocation_lift_module_sha256",
            "formulation_module_sha256",
            "tool_sha256",
        }
        or scope.get("formulation_id") != "R131"
        or scope.get("source_reports_read") != 3
        or scope.get("source_projection_collocations_read") != COLLOCATION_COUNT
        or scope.get("contact_edge_inventory_audits") != 1
        or scope.get("hybrid_state_lift_formulations") != 1
        or any(
            scope.get(key) != 0
            for key in (
                "state_lift_evaluations",
                "mass_matrix_assemblies",
                "contact_jacobian_assemblies",
                "projection_factorizations",
                "projection_solves",
                "controller_schedule_derivations",
                "inverse_dynamics_system_assemblies",
                "kinodynamic_solves",
                "physx_scene_runs",
            )
        )
        or scope.get("candidate_construction") is not False
        or scope.get("training") is not False
        or inventory.get("motor_interval_count") != MOTOR_INTERVAL_COUNT
        or inventory.get("collocation_count") != COLLOCATION_COUNT
        or inventory.get("changed_boundary_count") != 29
        or inventory.get("contact_exit_boundary_count") != 9
        or inventory.get("selected_exit_collocation_rows") != 36
        or inventory.get("selected_changed_base_velocity_rows") != 27
        or inventory.get("right_forefoot_exit_hotspot_intervals") != [76, 744]
        or selected.get("name") != "exit_mode_owned_left_velocity_trace_hold"
        or selected.get("selected_intervals") != inventory.get("exit_intervals")
        or selected.get("purpose")
        != "CAUSAL_DIAGNOSTIC_NOT_PRODUCTION_STATE_INTEGRATOR"
        or selected.get("kinematic_derivative_identity") != "NOT_CLAIMED"
        or selected.get("discrete_integration_constraint") != "ABSENT"
        or r132.get("maximum_real_anchor_rows") != 36
        or r132.get("maximum_real_projection_systems") != 36
        or r132.get("maximum_changed_base_velocity_rows") != 27
        or r132.get("full_schedule_projection") != "FORBIDDEN"
        or r132.get("controller_schedule_derivation") != "FORBIDDEN"
        or profile.get("decision", {}).get("complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_R132_EXIT_MODE_OWNED_LIFT_CONFORMANCE_ONLY"
        or profile.get("result_transitions", {}).get("complete")
        != "R131_COMPLETE_R132_CONFORMANCE_ONLY"
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r132_edge_lift_conformance")
        != "AUTHORIZED_REPORT_ONLY_ON_R131_COMPLETE"
        or any(
            not value.startswith("NOT_AUTHORIZED")
            for key, value in bounded.items()
            if key != "r132_edge_lift_conformance"
        )
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
        raise ValueError("R131 formulation profile differs")
    audit_alternative_decisions(profile)


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r130_rc1_profile_path: Path,
    r130_rc1_module_path: Path,
    r130_rc1_tool_path: Path,
    r130_profile_path: Path,
    r130_execution_module_path: Path,
    r130_tool_path: Path,
    r121_profile_path: Path,
    collocation_lift_module_path: Path,
) -> None:
    source = profile["source"]
    if (
        sha256(r130_rc1_profile_path) != source["r130_rc1"]["profile_sha256"]
        or sha256(r130_rc1_module_path) != source["r130_rc1"]["module_sha256"]
        or sha256(r130_rc1_tool_path) != source["r130_rc1"]["tool_sha256"]
        or sha256(r130_profile_path) != source["r130"]["profile_sha256"]
        or sha256(r130_execution_module_path)
        != source["r130"]["execution_module_sha256"]
        or sha256(r130_tool_path) != source["r130"]["tool_sha256"]
        or sha256(r121_profile_path) != source["r121"]["profile_sha256"]
        or sha256(collocation_lift_module_path)
        != source["collocation_lift_module_sha256"]
    ):
        raise ValueError("R131 bound source file identity differs")


def _validate_source_contracts(
    *,
    profile: Mapping[str, Any],
    r130_rc1: Mapping[str, Any],
    r130: Mapping[str, Any],
    r121: Mapping[str, Any],
) -> None:
    source = profile["source"]
    r130_counts = r130.get("projected_fixed_pd_schedule_audit", {}).get(
        "activation_counts", {}
    )
    r121_lift = r121.get("state_lift_contract", {})
    if (
        r130_rc1.get("repository", {}).get("commit")
        != source["r130_rc1"]["repository_commit"]
        or r130_rc1.get("status") != "COMPLETE"
        or r130_rc1.get("finding")
        != "CONFIRMED_PROJECTED_SCHEDULE_ACTUATOR_CONFLICT_WITH_HYBRID_EXIT_HOTSPOTS"
        or r130_rc1.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R131_HYBRID_CONTACT_EDGE_STATE_LIFT_FORMULATION_ONLY"
        or not all(r130_rc1.get("discriminators", {}).values())
        or r130_rc1.get("projection_solves") != 0
        or r130_rc1.get("inverse_dynamics_system_assemblies") != 0
        or r130.get("repository", {}).get("commit")
        != source["r130"]["repository_commit"]
        or r130.get("status") != "INVALID"
        or r130.get("result_transition") != "R130_CONSUMED_INVALID_NO_RETRY"
        or r130.get("solver_result", {}).get("invalid_reason")
        != "PROJECTED_FIXED_PD_SCHEDULE_INVALID"
        or r130.get("projection_result", {}).get("status") != "PASS"
        or r130.get("projection_result", {})
        .get("aggregate", {})
        .get("passing_collocations")
        != COLLOCATION_COUNT
        or r130_counts.get("velocity_violation") != 4
        or r130_counts.get("infeasible_effort_envelope") != 2
        or r130.get("inverse_dynamics_system_assemblies") != 0
        or r121.get("repository", {}).get("commit")
        != source["r121"]["repository_commit"]
        or r121.get("status") != "COMPLETE"
        or r121.get("fixed_pd_schedule_audit", {}).get("status") != "PASS"
        or tuple(r121_lift.get("collocation_abscissae", ()))
        != ("0", "1/4", "1/2", "3/4")
        or "independent affine interpolation"
        not in r121_lift.get("translation_and_joint_configuration", "")
        or "independent affine interpolation"
        not in r121_lift.get("linear_angular_and_joint_velocity", "")
        or r121_lift.get("kinematic_derivative_identity") != "NOT_CLAIMED_IN_STAGE_2"
    ):
        raise ValueError("R131 source gate differs")


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
        raise ValueError("R131 formulation requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R131 validation results differ")
    return normalized


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
