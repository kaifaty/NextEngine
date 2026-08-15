from __future__ import annotations

import copy
import hashlib
import json
import os
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.exit_mode_owned_projected_schedule_execution import (
    derive_eventful_projected_fixed_pd_schedule,
)
from next_lab.fixed_mode_controller_reachable_kinodynamic_formulation import (
    ACTUATED_JOINT_COUNT,
    GENERALIZED_WIDTH,
    MOTOR_INTERVAL_COUNT,
    PHYSICS_INTERVAL_COUNT,
    STATE_NODE_COUNT,
    SUBSTEPS_PER_INTERVAL,
    active_points_for_modes,
    canonical_json,
)
from next_lab.tangent_projected_fixed_pd_execution import (
    derive_projected_fixed_pd_schedule,
)

CONFORMANCE_ID = "nextengine.humanoid-fixed-mode-kinodynamic-graph-conformance.v1"
CHECK_ID = "TRAIN-4-FIXED-MODE-KINODYNAMIC-GRAPH-CONFORMANCE"
POINT_WIDTH = 3
ZERO_REAL_COUNTERS = (
    "real_state_reconstructions",
    "real_state_lift_evaluations",
    "real_controller_schedule_derivations",
    "real_controller_graph_evaluations",
    "real_mass_matrix_assemblies",
    "real_contact_jacobian_assemblies",
    "real_dynamics_residual_evaluations",
    "real_integration_residual_evaluations",
    "real_impulse_residual_evaluations",
    "real_kinodynamic_system_assemblies",
    "real_kinodynamic_solves",
    "real_factorizations",
    "optimizer_steps",
    "candidate_artifacts_built",
    "solver_private_caches_built",
    "physx_scene_runs",
    "training_runs",
)
BOUNDED_ACCEPTANCE_KEYS = frozenset(
    {
        "r139_kinodynamic_solve_formulation",
        "r136_retry",
        "r137_retry",
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


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_fixed_mode_kinodynamic_graph_conformance(
    *,
    profile_path: Path,
    r137_report_path: Path,
    r137_profile_path: Path,
    r137_module_path: Path,
    r137_tool_path: Path,
    base_controller_module_path: Path,
    projected_controller_module_path: Path,
    eventful_controller_module_path: Path,
    synthetic_descriptor_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Conform R137 graph mechanics without reading a real state/cache."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r137_report_path,
            r137_profile_path,
            r137_module_path,
            r137_tool_path,
            base_controller_module_path,
            projected_controller_module_path,
            eventful_controller_module_path,
            synthetic_descriptor_path,
            tool_path,
        )
    )
    (
        profile_path,
        r137_report_path,
        r137_profile_path,
        r137_module_path,
        r137_tool_path,
        base_controller_module_path,
        projected_controller_module_path,
        eventful_controller_module_path,
        synthetic_descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R138 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    source = profile["source"]
    r137 = _load_bound_report(r137_report_path, source["r137"])
    _validate_source_files(
        profile=profile,
        r137_profile_path=r137_profile_path,
        r137_module_path=r137_module_path,
        r137_tool_path=r137_tool_path,
        base_controller_module_path=base_controller_module_path,
        projected_controller_module_path=projected_controller_module_path,
        eventful_controller_module_path=eventful_controller_module_path,
        synthetic_descriptor_path=synthetic_descriptor_path,
    )
    if (
        sha256(Path(__file__).resolve()) != source["conformance_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R138 current conformance identity differs")

    source_audit = audit_r137_source(profile, r137=r137)
    index_audit = audit_index_map()
    transition_audit = audit_r137_transition_replay(r137=r137)
    layout_audit = audit_synthetic_layout_cases(profile)
    descriptor = json.loads(synthetic_descriptor_path.read_bytes())
    controller_audit = audit_synthetic_controller_cases(profile, descriptor=descriptor)
    validations = _validate_results(profile, validation_results)

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": "PASS",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["pass"],
        "result_transition": profile["result_transitions"]["pass"],
        "scope": profile["scope"],
        "source_lineage_audit": source_audit,
        "index_map_audit": index_audit,
        "transition_replay_audit": transition_audit,
        "synthetic_layout_audit": layout_audit,
        "synthetic_controller_audit": controller_audit,
        "conformed_graph_contract": profile["conformed_graph_contract"],
        "acceptance_boundary": profile["acceptance_boundary"],
        "future_r139_solve_formulation_contract": profile[
            "future_r139_solve_formulation_contract"
        ],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r137_report_file_sha256": sha256(r137_report_path),
            "r137_profile_sha256": sha256(r137_profile_path),
            "r137_module_sha256": sha256(r137_module_path),
            "r137_tool_sha256": sha256(r137_tool_path),
            "base_controller_module_sha256": sha256(base_controller_module_path),
            "projected_controller_module_sha256": sha256(
                projected_controller_module_path
            ),
            "eventful_controller_module_sha256": sha256(
                eventful_controller_module_path
            ),
            "synthetic_descriptor_sha256": sha256(synthetic_descriptor_path),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "source_report_audits": 1,
        "index_map_conformance_audits": 1,
        "transition_replay_conformance_audits": 1,
        "synthetic_layout_cases": layout_audit["passing_case_count"],
        "synthetic_malformed_cases": layout_audit["rejected_malformed_case_count"],
        "synthetic_controller_cases": controller_audit["case_count"],
        "synthetic_controller_schedule_derivations": controller_audit[
            "schedule_derivations"
        ],
        "synthetic_controller_rows_evaluated": controller_audit[
            "controller_rows_evaluated"
        ],
        **{counter: 0 for counter in ZERO_REAL_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def physics_address(motor_interval: int, substep: int) -> dict[str, int]:
    if (
        type(motor_interval) is not int
        or type(substep) is not int
        or not 0 <= motor_interval < MOTOR_INTERVAL_COUNT
        or not 0 <= substep < SUBSTEPS_PER_INTERVAL
    ):
        raise ValueError("R138 motor/physics address differs")
    physics_interval = motor_interval * SUBSTEPS_PER_INTERVAL + substep
    return {
        "motor_interval": motor_interval,
        "substep": substep,
        "physics_interval": physics_interval,
        "left_state_node": physics_interval,
        "right_state_node": physics_interval + 1,
        "command_target_row": motor_interval,
    }


def classify_point_transition(
    from_modes: Sequence[int], to_modes: Sequence[int]
) -> dict[str, Any]:
    before = active_points_for_modes(from_modes)
    after = active_points_for_modes(to_modes)
    activated = sorted(set(after) - set(before))
    deactivated = sorted(set(before) - set(after))
    if activated and deactivated:
        raise ValueError("R138 transition activates and releases together")
    if activated:
        event = "RIGID_POINT_ACTIVATION_IMPULSE"
    elif deactivated:
        event = "ZERO_IMPULSE_POINT_RELEASE"
    else:
        event = "ORDINARY_CONTINUOUS_BOUNDARY"
    return {
        "from_modes": [int(value) for value in from_modes],
        "to_modes": [int(value) for value in to_modes],
        "current_point_ordinals": before,
        "next_point_ordinals": after,
        "activated_point_ordinals": activated,
        "deactivated_point_ordinals": deactivated,
        "event": event,
    }


def assemble_step_layout(
    from_modes: Sequence[int], to_modes: Sequence[int]
) -> dict[str, Any]:
    transition = classify_point_transition(from_modes, to_modes)
    current_count = len(transition["current_point_ordinals"])
    next_count = len(transition["next_point_ordinals"])
    activation_count = len(transition["activated_point_ordinals"])
    return {
        **transition,
        "continuous_force_scalars": current_count * POINT_WIDTH,
        "activation_impulse_scalars": activation_count * POINT_WIDTH,
        "free_effort_scalars": 0,
        "inactive_force_scalars": 0,
        "unscheduled_impulse_scalars": 0,
        "row_blocks": {
            "continuous_dynamics": GENERALIZED_WIDTH,
            "pre_event_velocity": GENERALIZED_WIDTH,
            "configuration_integration": GENERALIZED_WIDTH,
            "velocity_transition": GENERALIZED_WIDTH,
            "active_position_closure": next_count * POINT_WIDTH,
            "active_velocity_closure": next_count * POINT_WIDTH,
            "active_acceleration_closure": current_count * POINT_WIDTH,
        },
        "continuous_force_cones": current_count,
        "activation_impulse_cones": activation_count,
        "release_impulse": "EXACT_ZERO"
        if transition["deactivated_point_ordinals"]
        else "NOT_APPLICABLE",
    }


def audit_index_map() -> dict[str, Any]:
    rows = [
        physics_address(interval, substep)
        for interval in range(MOTOR_INTERVAL_COUNT)
        for substep in range(SUBSTEPS_PER_INTERVAL)
    ]
    physics = [row["physics_interval"] for row in rows]
    target_counts = [0] * MOTOR_INTERVAL_COUNT
    for row in rows:
        target_counts[row["command_target_row"]] += 1
    if (
        physics != list(range(PHYSICS_INTERVAL_COUNT))
        or any(count != SUBSTEPS_PER_INTERVAL for count in target_counts)
        or rows[0]["left_state_node"] != 0
        or rows[-1]["right_state_node"] != STATE_NODE_COUNT - 1
    ):
        raise ValueError("R138 index map conformance differs")
    return {
        "status": "PASS",
        "motor_intervals": MOTOR_INTERVAL_COUNT,
        "physics_intervals": PHYSICS_INTERVAL_COUNT,
        "state_nodes": STATE_NODE_COUNT,
        "unique_physics_intervals": len(set(physics)),
        "target_hold_rows": MOTOR_INTERVAL_COUNT,
        "physics_steps_per_target": SUBSTEPS_PER_INTERVAL,
        "first_address": rows[0],
        "last_address": rows[-1],
        "ordered_address_sha256": hashlib.sha256(canonical_json(rows)).hexdigest(),
    }


def audit_r137_transition_replay(*, r137: Mapping[str, Any]) -> dict[str, Any]:
    source = r137.get("fixed_mode_transition_audit", {})
    rows = source.get("transition_rows", ())
    replayed: list[dict[str, Any]] = []
    activation_points = 0
    deactivation_points = 0
    for row in rows:
        boundary = int(row["motor_boundary"])
        replay = classify_point_transition(row["from_modes"], row["to_modes"])
        expected_node = physics_address(boundary + 1, 0)["left_state_node"]
        if (
            replay["event"] != row["event"]
            or replay["activated_point_ordinals"] != row["activated_point_ordinals"]
            or replay["deactivated_point_ordinals"] != row["deactivated_point_ordinals"]
            or expected_node != row["physics_node"]
        ):
            raise ValueError("R138 R137 transition replay differs")
        activation_points += len(replay["activated_point_ordinals"])
        deactivation_points += len(replay["deactivated_point_ordinals"])
        replayed.append(
            {
                "motor_boundary": boundary,
                "physics_node": expected_node,
                **replay,
            }
        )
    if len(replayed) != 29 or activation_points != 18 or deactivation_points != 18:
        raise ValueError("R138 R137 transition inventory differs")
    return {
        "status": "PASS",
        "changed_motor_boundaries": len(replayed),
        "activation_boundaries": sum(
            row["event"] == "RIGID_POINT_ACTIVATION_IMPULSE" for row in replayed
        ),
        "activation_points": activation_points,
        "zero_impulse_release_boundaries": sum(
            row["event"] == "ZERO_IMPULSE_POINT_RELEASE" for row in replayed
        ),
        "deactivation_points": deactivation_points,
        "ordered_transition_replay_sha256": hashlib.sha256(
            canonical_json(replayed)
        ).hexdigest(),
    }


def audit_synthetic_layout_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    expected = profile["synthetic_layout_contract"]
    cases = []
    for row in expected["cases"]:
        layout = assemble_step_layout(row["from_modes"], row["to_modes"])
        observed = {
            "case": row["case"],
            "event": layout["event"],
            "current_point_ordinals": layout["current_point_ordinals"],
            "next_point_ordinals": layout["next_point_ordinals"],
            "activated_point_ordinals": layout["activated_point_ordinals"],
            "deactivated_point_ordinals": layout["deactivated_point_ordinals"],
            "continuous_force_scalars": layout["continuous_force_scalars"],
            "activation_impulse_scalars": layout["activation_impulse_scalars"],
            "active_position_rows": layout["row_blocks"]["active_position_closure"],
            "active_acceleration_rows": layout["row_blocks"][
                "active_acceleration_closure"
            ],
            "release_impulse": layout["release_impulse"],
            "free_effort_scalars": layout["free_effort_scalars"],
            "inactive_force_scalars": layout["inactive_force_scalars"],
            "unscheduled_impulse_scalars": layout["unscheduled_impulse_scalars"],
        }
        if observed != {
            key: value
            for key, value in row.items()
            if key != "to_modes" and key != "from_modes"
        }:
            raise ValueError("R138 synthetic step-layout case differs")
        cases.append(observed)
    rejected = 0
    for malformed in expected["malformed_cases"]:
        try:
            assemble_step_layout(malformed["from_modes"], malformed["to_modes"])
        except ValueError as error:
            if str(error) != malformed["expected_error"]:
                raise ValueError("R138 malformed layout error differs") from error
            rejected += 1
        else:
            raise ValueError("R138 malformed layout case was accepted")
    return {
        "status": "PASS",
        "passing_case_count": len(cases),
        "rejected_malformed_case_count": rejected,
        "cases": cases,
    }


def audit_synthetic_controller_cases(
    profile: Mapping[str, Any], *, descriptor: Mapping[str, Any]
) -> dict[str, Any]:
    case_contract = profile["synthetic_controller_contract"]
    cases = []
    for name in ("zero", "ties_even", "limiter_state"):
        case_descriptor = copy.deepcopy(descriptor)
        position = np.zeros(
            (MOTOR_INTERVAL_COUNT + 1, ACTUATED_JOINT_COUNT), dtype=np.float64
        )
        velocity = np.zeros(
            (PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH), dtype=np.float64
        )
        if name == "ties_even":
            position[:, 0] = 0.5e-6
            position[:, 1] = 1.5e-6
            position[:, 2] = -1.5e-6
        elif name == "limiter_state":
            position[1:, 0] = 0.2
            velocity[:, 6] = -0.5
            for actuator in case_descriptor["actuators"]:
                if int(actuator["dof_ordinal"]) == 0:
                    actuator["maximum_positive_work_microjoules_per_motor_tick"] = 1
                    break
            else:
                raise ValueError("R138 synthetic actuator zero is absent")

        cache = {"joint_position_rad": position}
        expected = derive_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=case_descriptor,
            projected_velocity=velocity,
        )
        actual = derive_eventful_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=case_descriptor,
            projected_velocity=velocity,
        )
        if (
            not np.array_equal(
                actual.schedule.applied_target_microradians,
                expected.applied_target_microradians,
            )
            or not np.array_equal(
                actual.schedule.applied_effort_micronewton_metres,
                expected.applied_effort_micronewton_metres,
            )
            or actual.schedule.audit != expected.audit
            or actual.event_audit.get("status") != "PASS"
            or actual.event_audit.get("category_counts")
            != expected.audit["activation_counts"]
        ):
            raise ValueError("R138 exact controller differential differs")
        outcome = {
            "case": name,
            "status": expected.audit["status"],
            "activation_counts": expected.audit["activation_counts"],
            "event_count": actual.event_audit["event_count"],
            "first_target_microradians": [
                int(value) for value in expected.applied_target_microradians[0, :3]
            ],
            "maximum_positive_work_microjoules_per_motor_tick": expected.audit[
                "maxima"
            ]["positive_work_microjoules_per_motor_tick"],
            "target_sha256": _array_sha256(expected.applied_target_microradians),
            "effort_sha256": _array_sha256(expected.applied_effort_micronewton_metres),
            "differing_target_scalars": 0,
            "differing_effort_scalars": 0,
        }
        cases.append(outcome)
    comparable = [
        {
            "case": row["case"],
            "status": row["status"],
            "activation_counts": row["activation_counts"],
            "event_count": row["event_count"],
            "first_target_microradians": row["first_target_microradians"],
            "maximum_positive_work_microjoules_per_motor_tick": row[
                "maximum_positive_work_microjoules_per_motor_tick"
            ],
            "differing_target_scalars": row["differing_target_scalars"],
            "differing_effort_scalars": row["differing_effort_scalars"],
        }
        for row in cases
    ]
    if comparable != case_contract["expected_cases"]:
        raise ValueError("R138 synthetic controller case differs")
    return {
        "status": "PASS",
        "case_count": len(cases),
        "schedule_derivations": len(cases) * 2,
        "controller_rows_evaluated": len(cases) * 2 * PHYSICS_INTERVAL_COUNT,
        "target_arrays_compared": len(cases),
        "effort_arrays_compared": len(cases),
        "differing_target_scalars": 0,
        "differing_effort_scalars": 0,
        "cases": cases,
    }


def audit_r137_source(
    profile: Mapping[str, Any], *, r137: Mapping[str, Any]
) -> dict[str, Any]:
    source = profile["source"]["r137"]
    ids = r137.get("identities", {})
    transition = r137.get("fixed_mode_transition_audit", {})
    variables = r137.get("variable_inventory_audit", {})
    if (
        r137.get("status") != "COMPLETE"
        or r137.get("repository", {}).get("commit") != source["repository_commit"]
        or r137.get("repository", {}).get("dirty") is not False
        or r137.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R138_KINODYNAMIC_FORMULATION_CONFORMANCE_ONLY"
        or r137.get("result_transition") != "R137_COMPLETE_R138_CONFORMANCE_ONLY"
        or transition.get("state_nodes") != STATE_NODE_COUNT
        or transition.get("physics_intervals") != PHYSICS_INTERVAL_COUNT
        or transition.get("changed_motor_boundary_count") != 29
        or transition.get("activation_point_count") != 18
        or transition.get("deactivation_point_count") != 18
        or variables.get("primary_decision_scalars") != 311780
        or variables.get("free_effort_decision_scalars") != 0
        or r137.get("bounded_acceptance", {}).get(
            "r138_kinodynamic_formulation_conformance"
        )
        != "AUTHORIZED_REPORT_ONLY_ON_R137_COMPLETE"
        or any(r137.get(counter) != 0 for counter in profile["r137_zero_counters"])
        or ids.get("profile_sha256") != source["profile_sha256"]
        or ids.get("formulation_module_sha256") != source["module_sha256"]
        or ids.get("tool_sha256") != source["tool_sha256"]
    ):
        raise ValueError("R138 R137 source contract differs")
    return {
        "status": "PASS",
        "r137_report_sha256": r137["report_sha256"],
        "r137_transition": r137["result_transition"],
        "state_nodes": transition["state_nodes"],
        "physics_intervals": transition["physics_intervals"],
        "activation_points": transition["activation_point_count"],
        "deactivation_points": transition["deactivation_point_count"],
        "primary_decision_scalars": variables["primary_decision_scalars"],
        "free_effort_decision_scalars": variables["free_effort_decision_scalars"],
    }


def _array_sha256(array: NDArray[Any]) -> str:
    contiguous = np.ascontiguousarray(array)
    digest = hashlib.sha256()
    digest.update(str(contiguous.dtype).encode("ascii"))
    digest.update(canonical_json(list(contiguous.shape)))
    digest.update(contiguous.tobytes(order="C"))
    return digest.hexdigest()


def _load_bound_report(path: Path, expected: Mapping[str, Any]) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError("R138 R137 report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError("R138 R137 canonical report identity differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r137_profile_path: Path,
    r137_module_path: Path,
    r137_tool_path: Path,
    base_controller_module_path: Path,
    projected_controller_module_path: Path,
    eventful_controller_module_path: Path,
    synthetic_descriptor_path: Path,
) -> None:
    source = profile["source"]
    expected = (
        (r137_profile_path, source["r137"]["profile_sha256"]),
        (r137_module_path, source["r137"]["module_sha256"]),
        (r137_tool_path, source["r137"]["tool_sha256"]),
        (base_controller_module_path, source["base_controller_module_sha256"]),
        (
            projected_controller_module_path,
            source["projected_controller_module_sha256"],
        ),
        (
            eventful_controller_module_path,
            source["eventful_controller_module_sha256"],
        ),
        (synthetic_descriptor_path, source["synthetic_descriptor_sha256"]),
    )
    if any(sha256(path) != digest for path, digest in expected):
        raise ValueError("R138 source file identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    layout = profile.get("synthetic_layout_contract", {})
    controller = profile.get("synthetic_controller_contract", {})
    future = profile.get("future_r139_solve_formulation_contract", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "FixedModeKinodynamicGraphImplementationConformanceOnly"
        or scope.get("conformance_id") != "R138"
        or scope.get("source_reports_read") != 1
        or scope.get("synthetic_layout_cases") != 6
        or scope.get("synthetic_malformed_cases") != 1
        or scope.get("synthetic_controller_cases") != 3
        or scope.get("synthetic_controller_schedule_derivations") != 6
        or scope.get("synthetic_controller_rows_evaluated") != 19200
        or any(scope.get(counter) != 0 for counter in ZERO_REAL_COUNTERS)
        or len(layout.get("cases", ())) != 6
        or len(layout.get("malformed_cases", ())) != 1
        or len(controller.get("expected_cases", ())) != 3
        or future.get("authority") != "NOT_GRANTED_UNTIL_EXACT_R138_PASS"
        or future.get("real_kinodynamic_solve") != "FORBIDDEN"
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r139_kinodynamic_solve_formulation")
        != "AUTHORIZED_REPORT_ONLY_ON_R138_PASS"
        or any(
            not str(value).startswith("NOT_AUTHORIZED")
            for key, value in bounded.items()
            if key != "r139_kinodynamic_solve_formulation"
        )
        or profile.get("decision", {}).get("pass")
        != "PERMIT_SEPARATE_REPORT_ONLY_R139_KINODYNAMIC_SOLVE_FORMULATION_ONLY"
        or profile.get("result_transitions", {}).get("pass")
        != "R138_PASS_R139_FORMULATION_ONLY"
    ):
        raise ValueError("R138 conformance profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R138 conformance requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R138 validation result differs")
    return [dict(row) for row in results]


def _linux_thread_count() -> int:
    task_directory = Path(f"/proc/{os.getpid()}/task")
    if not task_directory.is_dir():
        raise RuntimeError("R138 requires Linux /proc thread accounting")
    return sum(1 for child in task_directory.iterdir() if child.name.isdigit())
