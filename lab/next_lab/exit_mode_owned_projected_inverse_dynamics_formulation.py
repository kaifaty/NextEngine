from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

FORMULATION_ID = (
    "nextengine.humanoid-exit-mode-owned-projected-inverse-dynamics-formulation.v1"
)
CHECK_ID = "TRAIN-4-EXIT-MODE-OWNED-PROJECTED-INVERSE-DYNAMICS-FORMULATION"
MOTOR_INTERVAL_COUNT = 800
SUBSTEPS_PER_INTERVAL = 4
COLLOCATION_COUNT = MOTOR_INTERVAL_COUNT * SUBSTEPS_PER_INTERVAL
GENERALIZED_WIDTH = 29
POINT_FORCE_WIDTH = 3
BOUNDED_ACCEPTANCE_KEYS = frozenset(
    {
        "r135_projected_inverse_dynamics_implementation_conformance",
        "r136_projected_inverse_dynamics_execution",
        "r123_retry",
        "r127_retry",
        "r129_retry",
        "r130_retry",
        "r132_retry",
        "r133_retry",
        "additional_projected_schedule_execution",
        "additional_inverse_dynamics_execution",
        "kinodynamic_solve",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
)
ZERO_EXECUTION_COUNTERS = (
    "state_lift_evaluations",
    "state_projection_systems",
    "projection_factorizations",
    "projection_solves",
    "controller_schedule_derivations",
    "inverse_dynamics_system_assemblies",
    "singular_value_decompositions",
    "particular_solutions",
    "gauge_interval_classifications",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "solver_private_caches_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_exit_mode_owned_projected_inverse_dynamics_formulation(
    *,
    profile_path: Path,
    r133_report_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r126_report_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    r121_module_path: Path,
    r121_tool_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R134 without reconstructing a state or numerical system."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r133_report_path,
            r133_profile_path,
            r133_module_path,
            r133_tool_path,
            r126_report_path,
            r126_profile_path,
            gauge_aware_kernel_path,
            r126_conformance_module_path,
            r126_tool_path,
            r121_report_path,
            r121_profile_path,
            r121_module_path,
            r121_tool_path,
            tool_path,
        )
    )
    (
        profile_path,
        r133_report_path,
        r133_profile_path,
        r133_module_path,
        r133_tool_path,
        r126_report_path,
        r126_profile_path,
        gauge_aware_kernel_path,
        r126_conformance_module_path,
        r126_tool_path,
        r121_report_path,
        r121_profile_path,
        r121_module_path,
        r121_tool_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R134 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    source = profile["source"]
    r133 = _load_bound_report(r133_report_path, source["r133"], "R133")
    r126 = _load_bound_report(r126_report_path, source["r126"], "R126")
    r121 = _load_bound_report(r121_report_path, source["r121"], "R121")
    _validate_source_files(
        profile=profile,
        r133_profile_path=r133_profile_path,
        r133_module_path=r133_module_path,
        r133_tool_path=r133_tool_path,
        r126_profile_path=r126_profile_path,
        gauge_aware_kernel_path=gauge_aware_kernel_path,
        r126_conformance_module_path=r126_conformance_module_path,
        r126_tool_path=r126_tool_path,
        r121_profile_path=r121_profile_path,
        r121_module_path=r121_module_path,
        r121_tool_path=r121_tool_path,
    )
    if (
        sha256(Path(__file__).resolve()) != source["formulation_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R134 current formulation identity differs")
    lineage = audit_source_lineage(profile, r133=r133, r126=r126, r121=r121)
    inventory = audit_system_inventory(r133=r133, r121=r121)
    if inventory != profile["system_inventory_contract"]:
        raise ValueError("R134 system inventory differs")
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
        "system_inventory_audit": inventory,
        "state_and_effort_lineage_contract": profile[
            "state_and_effort_lineage_contract"
        ],
        "acceleration_semantics_contract": profile["acceleration_semantics_contract"],
        "reduced_local_system_contract": profile["reduced_local_system_contract"],
        "numeric_contract": profile["numeric_contract"],
        "classification_contract": profile["classification_contract"],
        "r135_implementation_conformance_contract": profile[
            "r135_implementation_conformance_contract"
        ],
        "future_r136_execution_contract": profile["future_r136_execution_contract"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r133_report_file_sha256": sha256(r133_report_path),
            "r133_profile_sha256": sha256(r133_profile_path),
            "r133_module_sha256": sha256(r133_module_path),
            "r133_tool_sha256": sha256(r133_tool_path),
            "r126_report_file_sha256": sha256(r126_report_path),
            "r126_profile_sha256": sha256(r126_profile_path),
            "gauge_aware_kernel_sha256": sha256(gauge_aware_kernel_path),
            "r126_conformance_module_sha256": sha256(r126_conformance_module_path),
            "r126_tool_sha256": sha256(r126_tool_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "r121_module_sha256": sha256(r121_module_path),
            "r121_tool_sha256": sha256(r121_tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "projected_inverse_dynamics_formulations": 1,
        "source_report_audits": 3,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_source_lineage(
    profile: Mapping[str, Any],
    *,
    r133: Mapping[str, Any],
    r126: Mapping[str, Any],
    r121: Mapping[str, Any],
) -> dict[str, Any]:
    source = profile["source"]
    r133_ids = r133.get("identities", {})
    r126_ids = r126.get("identities", {})
    r121_ids = r121.get("identities", {})
    projection = r133.get("projection_result", {}).get("aggregate", {})
    schedule = r133.get("projected_fixed_pd_schedule_audit", {})
    events = r133.get("controller_activation_event_audit", {})
    counts = schedule.get("activation_counts", {})
    if (
        r133.get("status") != "PASS"
        or r133.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R134_PROJECTED_INVERSE_DYNAMICS_FORMULATION_ONLY"
        or r133.get("result_transition") != "R133_PASS_R134_FORMULATION_ONLY"
        or r133.get("repository", {}).get("commit")
        != source["r133"]["repository_commit"]
        or r133.get("repository", {}).get("dirty") is not False
        or r133.get("state_lift_evaluations") != COLLOCATION_COUNT
        or r133.get("state_projection_systems") != 2640
        or r133.get("projection_factorizations") != 2640
        or r133.get("projection_solves") != 2640
        or r133.get("controller_schedule_derivations") != 1
        or r133.get("inverse_dynamics_system_assemblies") != 0
        or projection.get("collocation_rows_recorded") != COLLOCATION_COUNT
        or projection.get("passing_collocations") != COLLOCATION_COUNT
        or projection.get("array_sha256", {}).get("projected_generalized_velocity")
        != profile["state_and_effort_lineage_contract"][
            "projected_generalized_velocity_float64_sha256"
        ]
        or schedule.get("status") != "PASS"
        or schedule.get("collocation_count") != COLLOCATION_COUNT
        or schedule.get("applied_effort_float64_sha256")
        != profile["state_and_effort_lineage_contract"][
            "applied_effort_micronewton_metres_float64_sha256"
        ]
        or schedule.get("applied_target_float64_sha256")
        != profile["state_and_effort_lineage_contract"][
            "applied_target_microradians_float64_sha256"
        ]
        or any(
            counts.get(key) != 0
            for key in (
                "hard_rom_violation",
                "velocity_violation",
                "infeasible_effort_envelope",
                "static_effort_clamp",
                "power_clamp",
                "positive_work_clamp",
            )
        )
        or counts.get("effort_rate_clamp") != 1010
        or counts.get("target_slew") != 1
        or events.get("status") != "PASS"
        or events.get("event_count") != 1011
        or events.get("all_events_row_addressed") is not True
        or events.get("execution_order_preserved") is not True
        or r133.get("bounded_acceptance", {}).get(
            "r134_projected_inverse_dynamics_formulation"
        )
        != "AUTHORIZED_ON_R133_PASS_ONLY"
    ):
        raise ValueError("R134 R133 source contract differs")
    if (
        r126.get("status") != "PASS"
        or r126.get("repository", {}).get("commit")
        != source["r126"]["repository_commit"]
        or r126.get("repository", {}).get("dirty") is not False
        or r126.get("frozen_anchor_singular_value_decompositions") != 7
        or r126.get("synthetic_singular_value_decompositions") != 5
        or r126.get("synthetic_particular_solutions") != 3
        or r126.get("synthetic_gauge_interval_classifications") != 4
        or r126.get("real_schedule_particular_solutions") != 0
        or r126.get("real_schedule_gauge_interval_classifications") != 0
        or r126.get("inverse_dynamics_execution_runs") != 0
        or r126.get("source_gates", {}).get("r121_report_sha256")
        != r121.get("report_sha256")
    ):
        raise ValueError("R134 R126 source contract differs")
    if (
        r121.get("status") != "COMPLETE"
        or r121.get("repository", {}).get("commit")
        != source["r121"]["repository_commit"]
        or r121.get("repository", {}).get("dirty") is not False
        or r121.get("fixed_pd_schedule_audit", {}).get("status") != "PASS"
        or r121.get("inverse_dynamics_solves") != 0
        or r121.get("local_system_solves") != 0
    ):
        raise ValueError("R134 R121 source contract differs")
    if (
        r133.get("source_gates", {}).get("r121_report_sha256")
        != r121.get("report_sha256")
        or r133_ids.get("current_descriptor_file_sha256")
        != r126_ids.get("current_descriptor_file_sha256")
        or r133_ids.get("current_descriptor_file_sha256")
        != r121_ids.get("current_descriptor_file_sha256")
        or r133_ids.get("dynamics_kernel_sha256") != r126_ids.get("r122_kernel_sha256")
        or r133_ids.get("v9_complete_clip_sha256")
        != r126_ids.get("v9_complete_clip_sha256")
        or r133_ids.get("v9_complete_clip_sha256")
        != r121_ids.get("v9_complete_clip_sha256")
    ):
        raise ValueError("R134 cross-source lineage differs")
    return {
        "status": "PASS",
        "r133_report_sha256": r133["report_sha256"],
        "r133_projection_status": r133["projection_result"]["status"],
        "r133_schedule_status": schedule["status"],
        "r133_controller_event_count": events["event_count"],
        "r126_report_sha256": r126["report_sha256"],
        "r126_gauge_kernel_conformance": "PASS",
        "r121_report_sha256": r121["report_sha256"],
        "shared_descriptor_sha256": r133_ids["current_descriptor_file_sha256"],
        "shared_dynamics_kernel_sha256": r133_ids["dynamics_kernel_sha256"],
        "shared_v9_complete_clip_sha256": r133_ids["v9_complete_clip_sha256"],
    }


def audit_system_inventory(
    *, r133: Mapping[str, Any], r121: Mapping[str, Any]
) -> dict[str, Any]:
    projection = r133.get("projection_result", {}).get("aggregate", {})
    base = r121.get("system_inventory_audit", {})
    flight = int(projection.get("flight_identity_collocations", -1))
    single = int(projection.get("single_point_projection_collocations", -1))
    flat = int(projection.get("flat_foot_projection_collocations", -1))
    active_points = int(base.get("friction_second_order_cone_count", -1))
    if flight + single + flat != COLLOCATION_COUNT or active_points < 0:
        raise ValueError("R134 source inventory is incomplete")
    reduced = (
        flight * GENERALIZED_WIDTH
        + single * (GENERALIZED_WIDTH + POINT_FORCE_WIDTH)
        + flat * (GENERALIZED_WIDTH + 2 * POINT_FORCE_WIDTH)
    )
    gauges = flat
    return {
        "status": "PASS",
        "motor_intervals": MOTOR_INTERVAL_COUNT,
        "collocations": COLLOCATION_COUNT,
        "flight_collocations": flight,
        "single_point_collocations": single,
        "flat_foot_collocations": flat,
        "active_point_collocations": active_points,
        "friction_cones": active_points,
        "inverse_dynamics_system_assemblies": COLLOCATION_COUNT,
        "singular_value_decompositions": COLLOCATION_COUNT,
        "particular_solutions": COLLOCATION_COUNT,
        "gauge_interval_classifications": gauges,
        "reduced_decision_scalars": reduced,
        "algebraic_equality_rows": reduced,
        "expected_independent_equality_rank": reduced - gauges,
        "exact_force_gauge_scalars": gauges,
        "maximum_reduced_local_unknown_count": 35,
        "maximum_flat_foot_gauge_dimension": 1,
    }


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R134 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R134 {label} canonical report identity differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    r121_profile_path: Path,
    r121_module_path: Path,
    r121_tool_path: Path,
) -> None:
    source = profile["source"]
    expected = (
        (r133_profile_path, source["r133"]["profile_sha256"]),
        (r133_module_path, source["r133"]["module_sha256"]),
        (r133_tool_path, source["r133"]["tool_sha256"]),
        (r126_profile_path, source["r126"]["profile_sha256"]),
        (gauge_aware_kernel_path, source["r126"]["gauge_aware_kernel_sha256"]),
        (
            r126_conformance_module_path,
            source["r126"]["conformance_module_sha256"],
        ),
        (r126_tool_path, source["r126"]["tool_sha256"]),
        (r121_profile_path, source["r121"]["profile_sha256"]),
        (r121_module_path, source["r121"]["module_sha256"]),
        (r121_tool_path, source["r121"]["tool_sha256"]),
    )
    if any(sha256(path) != digest for path, digest in expected):
        raise ValueError("R134 source file identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    inventory = profile.get("system_inventory_contract", {})
    r135 = profile.get("r135_implementation_conformance_contract", {})
    r136 = profile.get("future_r136_execution_contract", {})
    budget = r136.get("execution_budget", {})
    bounded = profile.get("bounded_acceptance", {})
    numeric = profile.get("numeric_contract", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "Stage2ExitModeOwnedProjectedFixedPdConeFeasibilityFormulationOnly"
        or scope.get("formulation_id") != "R134"
        or scope.get("source_reports_read") != 3
        or scope.get("collocation_count") != COLLOCATION_COUNT
        or any(scope.get(counter) != 0 for counter in ZERO_EXECUTION_COUNTERS)
        or inventory.get("collocations") != COLLOCATION_COUNT
        or inventory.get("flight_collocations") != 560
        or inventory.get("single_point_collocations") != 324
        or inventory.get("flat_foot_collocations") != 2316
        or inventory.get("active_point_collocations") != 4956
        or inventory.get("reduced_decision_scalars") != 107668
        or inventory.get("expected_independent_equality_rank") != 105352
        or inventory.get("exact_force_gauge_scalars") != 2316
        or r135.get("real_state_lift_evaluations") != 0
        or r135.get("real_projection_systems") != 0
        or r135.get("real_controller_schedule_derivations") != 0
        or r135.get("real_inverse_dynamics_system_assemblies") != 0
        or r135.get("pass_transition")
        != "PERMIT_ONE_R136_PROJECTED_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or r136.get("authority") != "NOT_GRANTED_UNTIL_EXACT_R135_PASS"
        or r136.get("collocation_count") != COLLOCATION_COUNT
        or budget.get("maximum_projection_systems") != 2640
        or budget.get("maximum_controller_schedule_derivations") != 1
        or budget.get("maximum_inverse_dynamics_system_assemblies") != COLLOCATION_COUNT
        or budget.get("maximum_singular_value_decompositions") != COLLOCATION_COUNT
        or budget.get("maximum_particular_solutions") != COLLOCATION_COUNT
        or budget.get("maximum_gauge_interval_classifications") != 2316
        or budget.get("randomized_restart_count") != 0
        or budget.get("resume_or_warm_restart") != "FORBIDDEN"
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("physical_group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r135_projected_inverse_dynamics_implementation_conformance")
        != "AUTHORIZED_REPORT_ONLY_ON_R134_COMPLETE"
        or bounded.get("r136_projected_inverse_dynamics_execution")
        != "NOT_AUTHORIZED_UNTIL_EXACT_R135_PASS"
        or any(
            not str(value).startswith("NOT_AUTHORIZED")
            for key, value in bounded.items()
            if key != "r135_projected_inverse_dynamics_implementation_conformance"
        )
        or profile.get("decision", {}).get("complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_R135_PROJECTED_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY"
        or profile.get("result_transitions", {}).get("complete")
        != "R134_COMPLETE_R135_CONFORMANCE_ONLY"
    ):
        raise ValueError("R134 formulation profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R134 formulation requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R134 validation result differs")
    return [dict(row) for row in results]
