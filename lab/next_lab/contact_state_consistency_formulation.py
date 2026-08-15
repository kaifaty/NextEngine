from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

FORMULATION_ID = "nextengine.humanoid-contact-state-consistency-formulation.v1"
CHECK_ID = "TRAIN-4-CONTACT-STATE-CONSISTENCY-FORMULATION"
MOTOR_INTERVAL_COUNT = 800
SUBSTEPS_PER_INTERVAL = 4
COLLOCATION_COUNT = MOTOR_INTERVAL_COUNT * SUBSTEPS_PER_INTERVAL
GENERALIZED_WIDTH = 29
POINT_ROW_WIDTH = 3
MODE_SPECS = (
    ("flat_flight", 2, True),
    ("flight_flat", 2, True),
    ("flight_flight", 0, False),
    ("flight_forefoot", 1, False),
    ("forefoot_flight", 1, False),
)
ALTERNATIVE_DECISIONS = {
    "mass_metric_tangent_velocity_projection": "SELECTED_STAGE2_DIAGNOSTIC",
    "contact_mode_correction": "DEFER_SCHEMA_AND_FRICTION_LAW_EXPANSION",
    "discrete_or_compliant_contact": "DEFER_NATIVE_CORRESPONDENCE_FORMULATION",
    "baumgarte_or_residual_relaxation": "REJECT_FOR_R127_REPAIR",
}
BOUNDED_ACCEPTANCE = {
    "r129_projection_conformance": "AUTHORIZED_REPORT_ONLY_ON_R128_COMPLETE",
    "r130_projected_fixed_pd_execution": "NOT_AUTHORIZED_UNTIL_R129_PASS",
    "r127_retry": "NOT_AUTHORIZED",
    "additional_inverse_dynamics_execution": "NOT_AUTHORIZED",
    "kinodynamic_solve": "NOT_AUTHORIZED",
    "candidate_artifact": "NOT_AUTHORIZED",
    "physx": "NOT_AUTHORIZED",
    "all_17": "NOT_AUTHORIZED",
    "full_v19": "NOT_AUTHORIZED",
    "training": "NOT_AUTHORIZED",
}


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_contact_state_consistency_formulation(
    *,
    profile_path: Path,
    r127_rc1_report_path: Path,
    r127_rc1_profile_path: Path,
    r127_rc1_module_path: Path,
    r127_rc1_tool_path: Path,
    r127_report_path: Path,
    r127_profile_path: Path,
    r127_module_path: Path,
    r127_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    r121_module_path: Path,
    r121_tool_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    gauge_kernel_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R128 without projecting a state or assembling a dynamics system."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r127_rc1_report_path,
            r127_rc1_profile_path,
            r127_rc1_module_path,
            r127_rc1_tool_path,
            r127_report_path,
            r127_profile_path,
            r127_module_path,
            r127_tool_path,
            r121_report_path,
            r121_profile_path,
            r121_module_path,
            r121_tool_path,
            r120_report_path,
            r120_profile_path,
            r120_cache_path,
            gauge_kernel_path,
            tool_path,
        )
    )
    (
        profile_path,
        r127_rc1_report_path,
        r127_rc1_profile_path,
        r127_rc1_module_path,
        r127_rc1_tool_path,
        r127_report_path,
        r127_profile_path,
        r127_module_path,
        r127_tool_path,
        r121_report_path,
        r121_profile_path,
        r121_module_path,
        r121_tool_path,
        r120_report_path,
        r120_profile_path,
        r120_cache_path,
        gauge_kernel_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R128 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r127_rc1 = _load_bound_report(
        r127_rc1_report_path, profile["source"]["r127_rc1"], "R127-RC1"
    )
    r127 = _load_bound_report(r127_report_path, profile["source"]["r127"], "R127")
    r121 = _load_bound_report(r121_report_path, profile["source"]["r121"], "R121")
    r120 = _load_bound_report(r120_report_path, profile["source"]["r120"], "R120")
    _validate_source_files(
        profile=profile,
        r127_rc1_profile_path=r127_rc1_profile_path,
        r127_rc1_module_path=r127_rc1_module_path,
        r127_rc1_tool_path=r127_rc1_tool_path,
        r127_profile_path=r127_profile_path,
        r127_module_path=r127_module_path,
        r127_tool_path=r127_tool_path,
        r121_profile_path=r121_profile_path,
        r121_module_path=r121_module_path,
        r121_tool_path=r121_tool_path,
        r120_profile_path=r120_profile_path,
        r120_cache_path=r120_cache_path,
        gauge_kernel_path=gauge_kernel_path,
    )
    _validate_source_contracts(
        profile=profile,
        r127_rc1=r127_rc1,
        r127=r127,
        r121=r121,
        r120=r120,
    )
    source = profile["source"]
    if (
        sha256(Path(__file__).resolve()) != source["formulation_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R128 current formulation identity differs")

    inventory = audit_projection_inventory(r121)
    if inventory != profile["projection_inventory_contract"]:
        raise ValueError("R128 projection inventory differs")
    alternatives = audit_alternative_decisions(profile)
    validations = _validate_results(profile, validation_results)

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "formulation_id": FORMULATION_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "scope": profile["scope"],
        "source_gates": {
            "r127_rc1_status": r127_rc1["status"],
            "r127_rc1_finding": r127_rc1["finding"],
            "r127_rc1_report_sha256": r127_rc1["report_sha256"],
            "r127_status": r127["status"],
            "r127_invalid_reason": r127["solver_result"]["invalid_reason"],
            "r127_report_sha256": r127["report_sha256"],
            "r121_status": r121["status"],
            "r121_report_sha256": r121["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "diagnosis_contract": profile["diagnosis_contract"],
        "alternative_decision_audit": alternatives,
        "selected_projection_contract": profile["selected_projection_contract"],
        "projection_inventory_audit": inventory,
        "preserved_invariants": profile["preserved_invariants"],
        "explicitly_replaced_invariants": profile["explicitly_replaced_invariants"],
        "claim_ceiling": profile["claim_ceiling"],
        "numeric_contract": profile["numeric_contract"],
        "r129_conformance_contract": profile["r129_conformance_contract"],
        "future_r130_execution_contract": profile["future_r130_execution_contract"],
        "research_basis": profile["research_basis"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r127_rc1_report_file_sha256": sha256(r127_rc1_report_path),
            "r127_rc1_profile_sha256": sha256(r127_rc1_profile_path),
            "r127_rc1_module_sha256": sha256(r127_rc1_module_path),
            "r127_rc1_tool_sha256": sha256(r127_rc1_tool_path),
            "r127_report_file_sha256": sha256(r127_report_path),
            "r127_profile_sha256": sha256(r127_profile_path),
            "r127_module_sha256": sha256(r127_module_path),
            "r127_tool_sha256": sha256(r127_tool_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "r121_module_sha256": sha256(r121_module_path),
            "r121_tool_sha256": sha256(r121_tool_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_profile_sha256": sha256(r120_profile_path),
            "r120_cache_sha256": sha256(r120_cache_path),
            "gauge_aware_kernel_sha256": sha256(gauge_kernel_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "contact_state_consistency_formulations": 1,
        "state_projections": 0,
        "mass_matrix_assemblies": 0,
        "contact_jacobian_assemblies": 0,
        "projection_factorizations": 0,
        "projection_solves": 0,
        "inverse_dynamics_evaluations": 0,
        "inverse_dynamics_solves": 0,
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


def projection_system_layout(
    *, active_point_count: int, same_body_flat: bool
) -> dict[str, int]:
    if active_point_count not in (0, 1, 2) or same_body_flat != (
        active_point_count == 2
    ):
        raise ValueError("R128 projection layout differs")
    constraint_rows = POINT_ROW_WIDTH * active_point_count
    independent_rank = constraint_rows - int(same_body_flat)
    return {
        "active_point_count": active_point_count,
        "constraint_row_count": constraint_rows,
        "expected_independent_rank": independent_rank,
        "expected_multiplier_nullity": constraint_rows - independent_rank,
        "projection_kkt_width": GENERALIZED_WIDTH + constraint_rows,
        "projected_velocity_width": GENERALIZED_WIDTH,
    }


def audit_projection_inventory(r121: Mapping[str, Any]) -> dict[str, Any]:
    schedule = r121.get("contact_schedule_audit", {})
    counts = schedule.get("mode_counts_over_motor_intervals", {})
    if set(counts) != {row[0] for row in MODE_SPECS}:
        raise ValueError("R128 source mode inventory differs")
    rows = []
    for mode, active_points, same_body_flat in MODE_SPECS:
        intervals = int(counts[mode])
        collocations = intervals * SUBSTEPS_PER_INTERVAL
        layout = projection_system_layout(
            active_point_count=active_points, same_body_flat=same_body_flat
        )
        rows.append(
            {
                "mode": mode,
                "motor_intervals": intervals,
                "collocations": collocations,
                **layout,
                "projection_required": active_points > 0,
            }
        )
    aggregate = {
        "motor_intervals": sum(row["motor_intervals"] for row in rows),
        "collocations": sum(row["collocations"] for row in rows),
        "flight_collocations": sum(
            row["collocations"] for row in rows if row["active_point_count"] == 0
        ),
        "single_point_collocations": sum(
            row["collocations"] for row in rows if row["active_point_count"] == 1
        ),
        "flat_foot_collocations": sum(
            row["collocations"] for row in rows if row["active_point_count"] == 2
        ),
        "projection_systems": sum(
            row["collocations"] for row in rows if row["projection_required"]
        ),
        "constraint_rows": sum(
            row["collocations"] * row["constraint_row_count"] for row in rows
        ),
        "expected_independent_rank": sum(
            row["collocations"] * row["expected_independent_rank"] for row in rows
        ),
        "redundant_multiplier_gauge_scalars": sum(
            row["collocations"] * row["expected_multiplier_nullity"] for row in rows
        ),
        "maximum_constraint_row_count": max(
            row["constraint_row_count"] for row in rows
        ),
        "maximum_projection_kkt_width": max(
            row["projection_kkt_width"] for row in rows
        ),
    }
    if (
        aggregate["motor_intervals"] != MOTOR_INTERVAL_COUNT
        or aggregate["collocations"] != COLLOCATION_COUNT
        or aggregate["constraint_rows"]
        != r121.get("system_inventory_audit", {}).get(
            "active_contact_acceleration_closure_row_count"
        )
        or aggregate["projection_systems"]
        != schedule.get("active_point_collocation_count", 0)
        - aggregate["flat_foot_collocations"]
    ):
        raise ValueError("R128 derived projection inventory does not close R121")
    return {"status": "PASS", "modes": rows, "aggregate": aggregate}


def audit_alternative_decisions(profile: Mapping[str, Any]) -> dict[str, Any]:
    rows = profile.get("alternative_decisions", [])
    decisions = {row.get("alternative"): row.get("decision") for row in rows}
    if decisions != ALTERNATIVE_DECISIONS or len(rows) != len(decisions):
        raise ValueError("R128 alternative decision matrix differs")
    selected = [
        name for name, decision in decisions.items() if decision.startswith("SELECTED_")
    ]
    if selected != ["mass_metric_tangent_velocity_projection"]:
        raise ValueError("R128 selected repair differs")
    return {
        "status": "PASS",
        "selected_alternative": selected[0],
        "alternatives": [dict(row) for row in rows],
    }


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r127_rc1_profile_path: Path,
    r127_rc1_module_path: Path,
    r127_rc1_tool_path: Path,
    r127_profile_path: Path,
    r127_module_path: Path,
    r127_tool_path: Path,
    r121_profile_path: Path,
    r121_module_path: Path,
    r121_tool_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    gauge_kernel_path: Path,
) -> None:
    source = profile["source"]
    identities = (
        (r127_rc1_profile_path, source["r127_rc1"]["profile_sha256"]),
        (r127_rc1_module_path, source["r127_rc1"]["module_sha256"]),
        (r127_rc1_tool_path, source["r127_rc1"]["tool_sha256"]),
        (r127_profile_path, source["r127"]["profile_sha256"]),
        (r127_module_path, source["r127"]["module_sha256"]),
        (r127_tool_path, source["r127"]["tool_sha256"]),
        (r121_profile_path, source["r121"]["profile_sha256"]),
        (r121_module_path, source["r121"]["module_sha256"]),
        (r121_tool_path, source["r121"]["tool_sha256"]),
        (r120_profile_path, source["r120"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (gauge_kernel_path, source["gauge_aware_kernel_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in identities):
        raise ValueError("R128 bound source file identity differs")


def _validate_source_contracts(
    *,
    profile: Mapping[str, Any],
    r127_rc1: Mapping[str, Any],
    r127: Mapping[str, Any],
    r121: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    source = profile["source"]
    rc1_id = r127_rc1.get("identities", {})
    r127_id = r127.get("identities", {})
    r121_id = r121.get("identities", {})
    r120_id = r120.get("identities", {})
    first_rows = r127.get("solver_result", {}).get("collocations", [])
    first = first_rows[0] if len(first_rows) == 1 else {}
    if (
        r127_rc1.get("status") != "COMPLETE"
        or r127_rc1.get("finding")
        != "CONFIRMED_FLAT_STICKING_STATE_ACCELERATION_INCOMPATIBILITY"
        or r127_rc1.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_CONTACT_STATE_CONSISTENCY_FORMULATION_ONLY"
        or not all(r127_rc1.get("discriminators", {}).values())
        or r127_rc1.get("kinematic_state_audits") != 1
        or any(
            r127_rc1.get(key) != 0
            for key in (
                "local_system_reconstructions",
                "singular_value_decompositions",
                "particular_solutions",
                "local_system_solves",
                "physx_scene_runs",
                "training_runs",
            )
        )
        or r127_rc1.get("repository", {}).get("commit")
        != source["r127_rc1"]["repository_commit"]
        or r127_rc1.get("repository", {}).get("dirty") is not False
        or rc1_id.get("profile_sha256") != source["r127_rc1"]["profile_sha256"]
        or rc1_id.get("research_module_sha256") != source["r127_rc1"]["module_sha256"]
        or rc1_id.get("tool_sha256") != source["r127_rc1"]["tool_sha256"]
        or rc1_id.get("r127_report_file_sha256") != source["r127"]["report_file_sha256"]
        or r127.get("status") != "INVALID"
        or r127.get("result_transition") != "R127_CONSUMED_INVALID_NO_RETRY"
        or r127.get("bounded_acceptance", {}).get("r127_retry") != "NOT_AUTHORIZED"
        or first.get("collocation") != 0
        or first.get("rank") != 34
        or first.get("nullity") != 1
        or first.get("invalid_reason") != "SCALED_EQUALITY_RESIDUAL_EXCEEDED"
        or r127.get("repository", {}).get("commit")
        != source["r127"]["repository_commit"]
        or r127.get("repository", {}).get("dirty") is not False
        or r127_id.get("profile_sha256") != source["r127"]["profile_sha256"]
        or r127_id.get("execution_module_sha256") != source["r127"]["module_sha256"]
        or r127_id.get("tool_sha256") != source["r127"]["tool_sha256"]
        or r121.get("status") != "COMPLETE"
        or r121.get("state_lift_contract", {}).get("kinematic_derivative_identity")
        != "NOT_CLAIMED_IN_STAGE_2"
        or r121.get("state_lift_contract", {}).get("discrete_integration_constraint")
        != "ABSENT_BY_DESIGN_AND_RESERVED_FOR_STAGE_3_FULL_KINODYNAMICS"
        or r121.get("scope", {}).get("physics_collocation_count") != COLLOCATION_COUNT
        or r121.get("repository", {}).get("commit")
        != source["r121"]["repository_commit"]
        or r121.get("repository", {}).get("dirty") is not False
        or r121_id.get("profile_sha256") != source["r121"]["profile_sha256"]
        or r121_id.get("formulation_module_sha256") != source["r121"]["module_sha256"]
        or r121_id.get("tool_sha256") != source["r121"]["tool_sha256"]
        or r120.get("status") != "PASS"
        or r120.get("repository", {}).get("commit")
        != source["r120"]["repository_commit"]
        or r120.get("repository", {}).get("dirty") is not False
        or r120_id.get("execution_profile_sha256") != source["r120"]["profile_sha256"]
        or r120_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
        or r121_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
        or r127_id.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
    ):
        raise ValueError("R128 source report contract differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R128 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R128 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    bounded = profile.get("bounded_acceptance", {})
    conformance = profile.get("r129_conformance_contract", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "Stage2MassMetricTangentVelocityProjectionFormulationOnly"
        or scope.get("run_id") != "R128"
        or scope.get("contact_state_consistency_formulations") != 1
        or any(
            scope.get(key) != 0
            for key in (
                "state_projections",
                "mass_matrix_assemblies",
                "contact_jacobian_assemblies",
                "projection_factorizations",
                "projection_solves",
                "inverse_dynamics_solves",
                "physx_scene_runs",
            )
        )
        or scope.get("candidate_construction") is not False
        or scope.get("training") is not False
        or profile.get("selected_projection_contract", {}).get("selection")
        != "MASS_METRIC_TANGENT_VELOCITY_PROJECTION"
        or conformance.get("run_id") != "R129"
        or conformance.get("execution") is not False
        or profile.get("decision", {}).get("complete")
        != "PERMIT_R129_REPORT_ONLY_TANGENT_PROJECTION_CONFORMANCE_ONLY"
        or bounded != BOUNDED_ACCEPTANCE
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
        raise ValueError("R128 formulation profile differs")
    audit_alternative_decisions(profile)


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R128 formulation requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R128 formulation validation differs")
    return normalized
