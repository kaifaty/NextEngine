from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

FORMULATION_ID = (
    "nextengine.humanoid-fixed-mode-controller-reachable-kinodynamic-formulation.v1"
)
CHECK_ID = "TRAIN-4-FIXED-MODE-CONTROLLER-REACHABLE-KINODYNAMIC-FORMULATION"
MOTOR_INTERVAL_COUNT = 800
SUBSTEPS_PER_INTERVAL = 4
PHYSICS_INTERVAL_COUNT = MOTOR_INTERVAL_COUNT * SUBSTEPS_PER_INTERVAL
STATE_NODE_COUNT = PHYSICS_INTERVAL_COUNT + 1
GENERALIZED_WIDTH = 29
ACTUATED_JOINT_COUNT = 23
POINT_FORCE_WIDTH = 3
MODE_FLIGHT = 0
MODE_FOREFOOT = 2
MODE_FLAT = 3
ZERO_EXECUTION_COUNTERS = (
    "state_reconstructions",
    "state_lift_evaluations",
    "mass_matrix_assemblies",
    "contact_jacobian_assemblies",
    "controller_schedule_derivations",
    "controller_graph_evaluations",
    "dynamics_residual_evaluations",
    "integration_residual_evaluations",
    "impulse_residual_evaluations",
    "kinodynamic_system_assemblies",
    "kinodynamic_solves",
    "factorizations",
    "optimizer_steps",
    "candidate_artifacts_built",
    "solver_private_caches_built",
    "physx_scene_runs",
    "training_runs",
)
BOUNDED_ACCEPTANCE_KEYS = frozenset(
    {
        "r138_kinodynamic_formulation_conformance",
        "r139_kinodynamic_solve_formulation",
        "r136_retry",
        "additional_pointwise_inverse_dynamics",
        "real_controller_graph_evaluation",
        "real_kinodynamic_assembly",
        "kinodynamic_solve",
        "contact_semantics_change",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
)
BRANCH_DECISIONS = {
    "fixed_mode_controller_reachable_trajectory_codesign": (
        "SELECT_R137_REPORT_ONLY_FORMULATION"
    ),
    "changed_contact_semantics": "DEFER_REQUIRES_SEPARATE_CORRESPONDENCE_DECISION",
    "pointwise_schedule_or_force_retry": "REJECT_FROZEN_R136_INFEASIBILITY",
    "free_effort_trajectory_optimization": "REJECT_NOT_CONTROLLER_REACHABLE",
}


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_fixed_mode_controller_reachable_kinodynamic_formulation(
    *,
    profile_path: Path,
    decision_document_path: Path,
    r136_report_path: Path,
    r136_profile_path: Path,
    r136_module_path: Path,
    r136_tool_path: Path,
    r131_report_path: Path,
    r131_profile_path: Path,
    r131_module_path: Path,
    r131_tool_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R137 without reconstructing or evaluating the real trajectory."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            decision_document_path,
            r136_report_path,
            r136_profile_path,
            r136_module_path,
            r136_tool_path,
            r131_report_path,
            r131_profile_path,
            r131_module_path,
            r131_tool_path,
            tool_path,
        )
    )
    (
        profile_path,
        decision_document_path,
        r136_report_path,
        r136_profile_path,
        r136_module_path,
        r136_tool_path,
        r131_report_path,
        r131_profile_path,
        r131_module_path,
        r131_tool_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R137 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    source = profile["source"]
    r136 = _load_bound_report(r136_report_path, source["r136"], "R136")
    r131 = _load_bound_report(r131_report_path, source["r131"], "R131")
    _validate_source_files(
        profile=profile,
        decision_document_path=decision_document_path,
        r136_profile_path=r136_profile_path,
        r136_module_path=r136_module_path,
        r136_tool_path=r136_tool_path,
        r131_profile_path=r131_profile_path,
        r131_module_path=r131_module_path,
        r131_tool_path=r131_tool_path,
    )
    if (
        sha256(Path(__file__).resolve()) != source["formulation_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R137 current formulation identity differs")

    lineage = audit_source_lineage(profile, r136=r136, r131=r131)
    transitions = audit_fixed_mode_transitions(r136["solver_result"]["collocations"])
    _compare_transition_contract(profile, transitions)
    variables = audit_variable_inventory(r136=r136, transitions=transitions)
    if variables != profile["variable_inventory_contract"]:
        raise ValueError("R137 variable inventory differs")
    branch = audit_branch_decision(profile)
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
        "source_lineage_audit": lineage,
        "branch_decision_audit": branch,
        "fixed_mode_transition_audit": transitions,
        "variable_inventory_audit": variables,
        "fixed_contact_semantics_contract": profile["fixed_contact_semantics_contract"],
        "controller_reachability_contract": profile["controller_reachability_contract"],
        "hybrid_transcription_contract": profile["hybrid_transcription_contract"],
        "acceptance_semantics_contract": profile["acceptance_semantics_contract"],
        "r138_implementation_conformance_contract": profile[
            "r138_implementation_conformance_contract"
        ],
        "future_r139_solve_formulation_contract": profile[
            "future_r139_solve_formulation_contract"
        ],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "decision_document_sha256": sha256(decision_document_path),
            "r136_report_file_sha256": sha256(r136_report_path),
            "r136_profile_sha256": sha256(r136_profile_path),
            "r136_module_sha256": sha256(r136_module_path),
            "r136_tool_sha256": sha256(r136_tool_path),
            "r131_report_file_sha256": sha256(r131_report_path),
            "r131_profile_sha256": sha256(r131_profile_path),
            "r131_module_sha256": sha256(r131_module_path),
            "r131_tool_sha256": sha256(r131_tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "source_report_audits": 2,
        "fixed_mode_transition_audits": 1,
        "kinodynamic_formulations": 1,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_source_lineage(
    profile: Mapping[str, Any],
    *,
    r136: Mapping[str, Any],
    r131: Mapping[str, Any],
) -> dict[str, Any]:
    source = profile["source"]
    solver = r136.get("solver_result", {})
    aggregate = solver.get("aggregate", {})
    r136_ids = r136.get("identities", {})
    r131_inventory = r131.get("contact_edge_inventory_audit", {})
    if (
        r136.get("status") != "COMPLETE"
        or r136.get("repository", {}).get("commit")
        != source["r136"]["repository_commit"]
        or r136.get("repository", {}).get("dirty") is not False
        or r136.get("gate_decision")
        != "STOP_VALID_PROJECTED_FIXED_PD_CONE_INFEASIBILITY_FOR_RESEARCH"
        or r136.get("result_transition") != "R136_VALID_INFEASIBLE_RESEARCH_REQUIRED"
        or solver.get("status") != "VALID_COMPLETE"
        or solver.get("feasibility") != "INFEASIBLE"
        or solver.get("invalid_reason") is not None
        or aggregate.get("collocation_rows_recorded") != PHYSICS_INTERVAL_COUNT
        or aggregate.get("numerically_valid_collocations") != PHYSICS_INTERVAL_COUNT
        or aggregate.get("feasible_collocations") != 782
        or aggregate.get("infeasible_collocations") != 2418
        or solver.get("active_point_cones") != 4956
        or r136.get("kinodynamic_solves") != 0
        or r136.get("candidate_artifacts_built") != 0
        or r136.get("physx_scene_runs") != 0
        or r136.get("optimizer_steps") != 0
        or r136.get("training_runs") != 0
        or r136_ids.get("profile_sha256") != source["r136"]["profile_sha256"]
        or r136_ids.get("execution_module_sha256") != source["r136"]["module_sha256"]
        or r136_ids.get("tool_sha256") != source["r136"]["tool_sha256"]
    ):
        raise ValueError("R137 R136 source contract differs")
    if (
        r131.get("status") != "COMPLETE"
        or r131.get("repository", {}).get("commit")
        != source["r131"]["repository_commit"]
        or r131.get("repository", {}).get("dirty") is not False
        or r131.get("formulation_id")
        != "nextengine.humanoid-hybrid-contact-edge-state-lift-formulation.v1"
        or r131_inventory.get("motor_interval_count") != MOTOR_INTERVAL_COUNT
        or r131_inventory.get("collocation_count") != PHYSICS_INTERVAL_COUNT
        or r131_inventory.get("changed_boundary_count") != 29
        or r131_inventory.get("contact_entry_boundary_count") != 9
        or r131_inventory.get("contact_exit_boundary_count") != 9
        or r131_inventory.get("active_mode_change_boundary_count") != 11
        or r131.get("kinodynamic_solves") != 0
        or r131.get("physx_scene_runs") != 0
        or r131.get("training_runs") != 0
        or r131.get("identities", {}).get("profile_sha256")
        != source["r131"]["profile_sha256"]
        or r131.get("identities", {}).get("formulation_module_sha256")
        != source["r131"]["module_sha256"]
        or r131.get("identities", {}).get("tool_sha256")
        != source["r131"]["tool_sha256"]
    ):
        raise ValueError("R137 R131 source contract differs")
    return {
        "status": "PASS",
        "authority_source": "tracked post-R136 R137 research/roadmap decision",
        "r136_report_sha256": r136["report_sha256"],
        "r136_solver_status": solver["status"],
        "r136_feasibility": solver["feasibility"],
        "r136_feasible_collocations": aggregate["feasible_collocations"],
        "r136_infeasible_collocations": aggregate["infeasible_collocations"],
        "r131_report_sha256": r131["report_sha256"],
        "r131_motor_mode_sequence_sha256": r131_inventory["motor_mode_sequence_sha256"],
        "shared_descriptor_sha256": r136_ids["current_descriptor_file_sha256"],
        "shared_dynamics_kernel_sha256": r136_ids["dynamics_kernel_sha256"],
        "shared_v9_complete_clip_sha256": r136_ids["v9_complete_clip_sha256"],
    }


def audit_fixed_mode_transitions(
    collocations: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    if len(collocations) != PHYSICS_INTERVAL_COUNT:
        raise ValueError("R137 source collocation count differs")
    interval_modes: list[list[int]] = []
    interval_points: list[list[int]] = []
    for interval in range(MOTOR_INTERVAL_COUNT):
        start = interval * SUBSTEPS_PER_INTERVAL
        rows = collocations[start : start + SUBSTEPS_PER_INTERVAL]
        owned_modes: list[int] | None = None
        owned_points: list[int] | None = None
        for substep, row in enumerate(rows):
            if (
                row.get("collocation") != start + substep
                or row.get("interval") != interval
                or row.get("substep") != substep
            ):
                raise ValueError("R137 source collocation order differs")
            modes = [int(value) for value in row.get("contact_modes", ())]
            points = [int(value) for value in row.get("active_point_ordinals", ())]
            if len(modes) != 2 or any(
                mode not in (MODE_FLIGHT, MODE_FOREFOOT, MODE_FLAT) for mode in modes
            ):
                raise ValueError("R137 fixed contact mode differs")
            if points != active_points_for_modes(modes):
                raise ValueError("R137 active point ownership differs")
            if owned_modes is None:
                owned_modes = modes
                owned_points = points
            elif modes != owned_modes or points != owned_points:
                raise ValueError("R137 interval contact ownership differs")
        if owned_modes is None or owned_points is None:
            raise ValueError("R137 interval metadata is absent")
        interval_modes.append(owned_modes)
        interval_points.append(owned_points)

    rows: list[dict[str, Any]] = []
    activated_points = 0
    deactivated_points = 0
    activation_boundaries = 0
    deactivation_boundaries = 0
    for boundary in range(MOTOR_INTERVAL_COUNT - 1):
        before = interval_points[boundary]
        after = interval_points[boundary + 1]
        activated = sorted(set(after) - set(before))
        deactivated = sorted(set(before) - set(after))
        if not activated and not deactivated:
            continue
        if activated and deactivated:
            raise ValueError("R137 boundary changes activation and release together")
        if activated:
            activation_boundaries += 1
            activated_points += len(activated)
            event = "RIGID_POINT_ACTIVATION_IMPULSE"
        else:
            deactivation_boundaries += 1
            deactivated_points += len(deactivated)
            event = "ZERO_IMPULSE_POINT_RELEASE"
        rows.append(
            {
                "motor_boundary": boundary,
                "physics_node": (boundary + 1) * SUBSTEPS_PER_INTERVAL,
                "from_modes": interval_modes[boundary],
                "to_modes": interval_modes[boundary + 1],
                "activated_point_ordinals": activated,
                "deactivated_point_ordinals": deactivated,
                "event": event,
            }
        )

    active_force_rows = sum(
        len(points) * SUBSTEPS_PER_INTERVAL for points in interval_points
    )
    initial_points = interval_points[0]
    terminal_points = interval_points[-1]
    return {
        "status": "PASS",
        "motor_intervals": MOTOR_INTERVAL_COUNT,
        "physics_intervals": PHYSICS_INTERVAL_COUNT,
        "state_nodes": STATE_NODE_COUNT,
        "motor_boundary_count": MOTOR_INTERVAL_COUNT - 1,
        "unchanged_motor_boundary_count": MOTOR_INTERVAL_COUNT - 1 - len(rows),
        "changed_motor_boundary_count": len(rows),
        "activation_boundary_count": activation_boundaries,
        "activation_point_count": activated_points,
        "deactivation_boundary_count": deactivation_boundaries,
        "deactivation_point_count": deactivated_points,
        "initial_active_point_count": len(initial_points),
        "terminal_active_point_count": len(terminal_points),
        "contact_point_anchor_count": len(initial_points) + activated_points,
        "active_point_force_rows": active_force_rows,
        "motor_mode_sequence_sha256": hashlib.sha256(
            canonical_json(interval_modes)
        ).hexdigest(),
        "transition_rows": rows,
    }


def active_points_for_modes(modes: Sequence[int]) -> list[int]:
    if len(modes) != 2:
        raise ValueError("R137 contact side count differs")
    points: list[int] = []
    for side, mode in enumerate(modes):
        offset = side * 2
        if mode == MODE_FOREFOOT:
            points.append(offset + 1)
        elif mode == MODE_FLAT:
            points.extend((offset, offset + 1))
        elif mode != MODE_FLIGHT:
            raise ValueError("R137 contact mode differs")
    return points


def audit_variable_inventory(
    *,
    r136: Mapping[str, Any],
    transitions: Mapping[str, Any],
) -> dict[str, Any]:
    active_force_rows = int(transitions["active_point_force_rows"])
    activation_points = int(transitions["activation_point_count"])
    solver = r136.get("solver_result", {})
    if (
        active_force_rows != 4956
        or solver.get("active_point_cones") != active_force_rows
        or activation_points != 18
    ):
        raise ValueError("R137 source variable inventory is incomplete")
    q = STATE_NODE_COUNT * GENERALIZED_WIDTH
    velocity = STATE_NODE_COUNT * GENERALIZED_WIDTH
    acceleration = PHYSICS_INTERVAL_COUNT * GENERALIZED_WIDTH
    command = MOTOR_INTERVAL_COUNT * ACTUATED_JOINT_COUNT
    force = active_force_rows * POINT_FORCE_WIDTH
    impulse = activation_points * POINT_FORCE_WIDTH
    return {
        "status": "PASS",
        "state_nodes": STATE_NODE_COUNT,
        "physics_intervals": PHYSICS_INTERVAL_COUNT,
        "motor_target_rows": MOTOR_INTERVAL_COUNT,
        "generalized_width": GENERALIZED_WIDTH,
        "actuated_joint_count": ACTUATED_JOINT_COUNT,
        "configuration_local_scalars": q,
        "generalized_velocity_scalars": velocity,
        "generalized_acceleration_scalars": acceleration,
        "commanded_target_integer_scalars": command,
        "active_contact_force_scalars": force,
        "activation_impulse_scalars": impulse,
        "primary_decision_scalars": q
        + velocity
        + acceleration
        + command
        + force
        + impulse,
        "derived_applied_target_scalars": command,
        "derived_applied_effort_scalars": (
            PHYSICS_INTERVAL_COUNT * ACTUATED_JOINT_COUNT
        ),
        "free_effort_decision_scalars": 0,
        "contact_anchor_decision_scalars": 0,
    }


def audit_branch_decision(profile: Mapping[str, Any]) -> dict[str, Any]:
    rows = profile.get("branch_decisions", ())
    actual = {row.get("branch"): row.get("decision") for row in rows}
    selected = [
        branch
        for branch, decision in actual.items()
        if decision == "SELECT_R137_REPORT_ONLY_FORMULATION"
    ]
    if (
        len(rows) != len(BRANCH_DECISIONS)
        or actual != BRANCH_DECISIONS
        or selected != ["fixed_mode_controller_reachable_trajectory_codesign"]
    ):
        raise ValueError("R137 branch decision matrix differs")
    return {
        "status": "PASS",
        "branches_compared": len(rows),
        "selected_branch": selected[0],
        "decisions": [dict(row) for row in rows],
    }


def _compare_transition_contract(
    profile: Mapping[str, Any], audit: Mapping[str, Any]
) -> None:
    expected = profile["fixed_mode_transition_contract"]
    comparable = {key: audit.get(key) for key in expected}
    if comparable != expected:
        raise ValueError("R137 fixed-mode transition inventory differs")
    r131_hash = profile["source_contract"]["r131_motor_mode_sequence_sha256"]
    if audit["motor_mode_sequence_sha256"] != r131_hash:
        raise ValueError("R137 R131/R136 mode sequence differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R137 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R137 {label} canonical report identity differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    decision_document_path: Path,
    r136_profile_path: Path,
    r136_module_path: Path,
    r136_tool_path: Path,
    r131_profile_path: Path,
    r131_module_path: Path,
    r131_tool_path: Path,
) -> None:
    source = profile["source"]
    expected = (
        (decision_document_path, source["decision_document_sha256"]),
        (r136_profile_path, source["r136"]["profile_sha256"]),
        (r136_module_path, source["r136"]["module_sha256"]),
        (r136_tool_path, source["r136"]["tool_sha256"]),
        (r131_profile_path, source["r131"]["profile_sha256"]),
        (r131_module_path, source["r131"]["module_sha256"]),
        (r131_tool_path, source["r131"]["tool_sha256"]),
    )
    if any(sha256(path) != digest for path, digest in expected):
        raise ValueError("R137 source file identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    transition = profile.get("fixed_mode_transition_contract", {})
    variables = profile.get("variable_inventory_contract", {})
    controller = profile.get("controller_reachability_contract", {})
    hybrid = profile.get("hybrid_transcription_contract", {})
    r138 = profile.get("r138_implementation_conformance_contract", {})
    r139 = profile.get("future_r139_solve_formulation_contract", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "FixedModeControllerReachableKinodynamicFeasibilityFormulationOnly"
        or scope.get("formulation_id") != "R137"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("source_reports_read") != 2
        or scope.get("source_collocation_metadata_rows_read") != PHYSICS_INTERVAL_COUNT
        or scope.get("kinodynamic_formulations") != 1
        or any(scope.get(counter) != 0 for counter in ZERO_EXECUTION_COUNTERS)
        or transition.get("state_nodes") != STATE_NODE_COUNT
        or transition.get("physics_intervals") != PHYSICS_INTERVAL_COUNT
        or transition.get("changed_motor_boundary_count") != 29
        or transition.get("activation_boundary_count") != 11
        or transition.get("activation_point_count") != 18
        or transition.get("deactivation_boundary_count") != 18
        or transition.get("deactivation_point_count") != 18
        or transition.get("active_point_force_rows") != 4956
        or variables.get("primary_decision_scalars") != 311780
        or variables.get("free_effort_decision_scalars") != 0
        or variables.get("contact_anchor_decision_scalars") != 0
        or controller.get("command_rows") != MOTOR_INTERVAL_COUNT
        or controller.get("effort_rows") != PHYSICS_INTERVAL_COUNT
        or controller.get("effort_status") != "EXACT_DERIVED_NOT_FREE"
        or controller.get("acceptance_replay")
        != "BYTE_EXACT_QUANTIZED_FIXED_PD_REQUIRED"
        or hybrid.get("time_step_seconds") != "1/240"
        or hybrid.get("contact_mode_choice") != "FORBIDDEN"
        or hybrid.get("release_impulse") != "EXACT_ZERO"
        or r138.get("real_state_reconstructions") != 0
        or r138.get("real_controller_schedule_derivations") != 0
        or r138.get("real_kinodynamic_system_assemblies") != 0
        or r138.get("real_kinodynamic_solves") != 0
        or r138.get("pass_transition")
        != "PERMIT_SEPARATE_REPORT_ONLY_R139_KINODYNAMIC_SOLVE_FORMULATION_ONLY"
        or r139.get("authority") != "NOT_GRANTED_UNTIL_EXACT_R138_PASS"
        or r139.get("real_kinodynamic_solve") != "FORBIDDEN"
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r138_kinodynamic_formulation_conformance")
        != "AUTHORIZED_REPORT_ONLY_ON_R137_COMPLETE"
        or any(
            not str(value).startswith("NOT_AUTHORIZED")
            for key, value in bounded.items()
            if key != "r138_kinodynamic_formulation_conformance"
        )
        or profile.get("decision", {}).get("complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_R138_KINODYNAMIC_FORMULATION_CONFORMANCE_ONLY"
        or profile.get("result_transitions", {}).get("complete")
        != "R137_COMPLETE_R138_CONFORMANCE_ONLY"
    ):
        raise ValueError("R137 formulation profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R137 formulation requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R137 validation result differs")
    return [dict(row) for row in results]
