from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

FORMULATION_ID = "nextengine.humanoid-redundant-contact-feasibility-formulation.v1"
CHECK_ID = "TRAIN-4-REDUNDANT-CONTACT-FEASIBILITY-FORMULATION"
MOTOR_INTERVAL_COUNT = 800
SUBSTEPS_PER_INTERVAL = 4
COLLOCATION_COUNT = MOTOR_INTERVAL_COUNT * SUBSTEPS_PER_INTERVAL
GENERALIZED_WIDTH = 29
POINT_FORCE_WIDTH = 3

MODE_SPECS = (
    ("flat_flight", 2, 1),
    ("flight_flat", 2, 1),
    ("flight_flight", 0, 0),
    ("flight_forefoot", 1, 0),
    ("forefoot_flight", 1, 0),
)


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_redundant_contact_feasibility_formulation(
    *,
    profile_path: Path,
    research_report_path: Path,
    research_profile_path: Path,
    research_module_path: Path,
    research_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    r121_module_path: Path,
    r121_tool_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R125 without assembling or factoring a dynamics system."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            research_report_path,
            research_profile_path,
            research_module_path,
            research_tool_path,
            r121_report_path,
            r121_profile_path,
            r121_module_path,
            r121_tool_path,
            tool_path,
        )
    )
    (
        profile_path,
        research_report_path,
        research_profile_path,
        research_module_path,
        research_tool_path,
        r121_report_path,
        r121_profile_path,
        r121_module_path,
        r121_tool_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R125 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    research = validate_research(
        profile=profile,
        report_path=research_report_path,
        research_profile_path=research_profile_path,
        research_module_path=research_module_path,
        research_tool_path=research_tool_path,
    )
    r121 = validate_r121(
        profile=profile,
        report_path=r121_report_path,
        formulation_profile_path=r121_profile_path,
        formulation_module_path=r121_module_path,
        formulation_tool_path=r121_tool_path,
    )
    inventory = audit_mode_inventory(r121)
    if inventory != profile["mode_inventory_contract"]:
        raise ValueError("R125 mode inventory differs")
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
            "r123_rc1_status": research["status"],
            "r123_rc1_finding": research["finding"],
            "r123_rc1_report_sha256": research["report_sha256"],
            "r123_status": research["r123_result"]["status"],
            "r123_local_system_solves": research["r123_result"]["local_system_solves"],
            "r121_status": r121["status"],
            "r121_report_sha256": r121["report_sha256"],
        },
        "preserved_invariants": profile["preserved_invariants"],
        "mode_inventory_audit": inventory,
        "reduced_local_system_contract": profile["reduced_local_system_contract"],
        "analytic_force_gauge_contract": profile["analytic_force_gauge_contract"],
        "rank_revealing_particular_solution_contract": profile[
            "rank_revealing_particular_solution_contract"
        ],
        "friction_gauge_interval_contract": profile["friction_gauge_interval_contract"],
        "feasibility_classification_contract": profile[
            "feasibility_classification_contract"
        ],
        "numeric_contract": profile["numeric_contract"],
        "implementation_conformance_contract": profile[
            "implementation_conformance_contract"
        ],
        "future_single_execution_budget": profile["future_single_execution_budget"],
        "future_execution_output": profile["future_execution_output"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "research_report_file_sha256": sha256(research_report_path),
            "research_profile_sha256": sha256(research_profile_path),
            "research_module_sha256": sha256(research_module_path),
            "research_tool_sha256": sha256(research_tool_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "r121_module_sha256": sha256(r121_module_path),
            "r121_tool_sha256": sha256(r121_tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "redundant_contact_feasibility_formulations": 1,
        "research_audits": 0,
        "local_system_reconstructions": 0,
        "singular_value_decompositions": 0,
        "particular_solutions": 0,
        "gauge_interval_classifications": 0,
        "local_system_solves": 0,
        "solver_runs": 0,
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


def audit_mode_inventory(r121: Mapping[str, Any]) -> dict[str, Any]:
    source = r121.get("contact_schedule_audit", {})
    mode_counts = source.get("mode_counts_over_motor_intervals", {})
    if set(mode_counts) != {row[0] for row in MODE_SPECS}:
        raise ValueError("R125 source mode inventory differs")
    rows = []
    for mode, active_points, flat_feet in MODE_SPECS:
        intervals = int(mode_counts[mode])
        collocations = intervals * SUBSTEPS_PER_INTERVAL
        unknown_count = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * active_points
        nullity = flat_feet
        rows.append(
            {
                "mode": mode,
                "motor_intervals": intervals,
                "collocations": collocations,
                "active_point_count": active_points,
                "flat_foot_gauge_dimension": flat_feet,
                "reduced_local_unknown_count": unknown_count,
                "algebraic_equality_row_count": unknown_count,
                "expected_independent_rank": unknown_count - nullity,
                "friction_cone_count": collocations * active_points,
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
            row["collocations"] for row in rows if row["flat_foot_gauge_dimension"] == 1
        ),
        "active_point_collocations": sum(row["friction_cone_count"] for row in rows),
        "friction_cones": sum(row["friction_cone_count"] for row in rows),
        "reduced_decision_scalars": sum(
            row["collocations"] * row["reduced_local_unknown_count"] for row in rows
        ),
        "algebraic_equality_rows": sum(
            row["collocations"] * row["algebraic_equality_row_count"] for row in rows
        ),
        "expected_independent_equality_rank": sum(
            row["collocations"] * row["expected_independent_rank"] for row in rows
        ),
        "exact_force_gauge_scalars": sum(
            row["collocations"] * row["flat_foot_gauge_dimension"] for row in rows
        ),
        "maximum_reduced_local_unknown_count": max(
            row["reduced_local_unknown_count"] for row in rows
        ),
        "maximum_flat_foot_gauge_dimension": max(
            row["flat_foot_gauge_dimension"] for row in rows
        ),
    }
    if (
        aggregate["motor_intervals"] != MOTOR_INTERVAL_COUNT
        or aggregate["collocations"] != COLLOCATION_COUNT
        or aggregate["active_point_collocations"]
        != source.get("active_point_collocation_count")
        or aggregate["friction_cones"]
        != r121.get("equation_contract", {}).get("friction_second_order_cone_count")
    ):
        raise ValueError("R125 derived inventory does not close R121")
    return {"status": "PASS", "modes": rows, "aggregate": aggregate}


def scaled_line_cone_coefficients(
    *,
    force_nrf: Sequence[int],
    direction_nrf: Sequence[int],
    friction_numerator: int = 52429,
    friction_denominator: int = 65536,
) -> dict[str, int]:
    """Return integer coefficients for den²||t||²-num²n² <= 0 on a line."""

    if len(force_nrf) != 3 or len(direction_nrf) != 3:
        raise ValueError("line-cone vectors must have three components")
    if friction_numerator <= 0 or friction_denominator <= 0:
        raise ValueError("friction ratio must be positive")
    n0, right0, forward0 = (int(value) for value in force_nrf)
    dn, dright, dforward = (int(value) for value in direction_nrf)
    den2 = friction_denominator * friction_denominator
    num2 = friction_numerator * friction_numerator
    return {
        "quadratic": den2 * (dright * dright + dforward * dforward) - num2 * dn * dn,
        "linear": 2 * (den2 * (right0 * dright + forward0 * dforward) - num2 * n0 * dn),
        "constant": den2 * (right0 * right0 + forward0 * forward0) - num2 * n0 * n0,
        "normal_intercept": n0,
        "normal_slope": dn,
    }


def validate_research(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    research_profile_path: Path,
    research_module_path: Path,
    research_tool_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r123_rc1"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(research_profile_path) != expected["profile_sha256"]
        or sha256(research_module_path) != expected["module_sha256"]
        or sha256(research_tool_path) != expected["tool_sha256"]
    ):
        raise ValueError("R125 research source file identity differs")
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R123-RC1")
    geometry = report.get("exact_geometry_audit", {})
    identities = report.get("identities", {})
    if (
        report.get("status") != "COMPLETE"
        or report.get("finding") != "CONFIRMED_REDUNDANT_FLAT_FOOT_FORCE_GAUGE"
        or report.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R125_REDUNDANT_CONTACT_FEASIBILITY_FORMULATION_ONLY"
        or geometry.get("same_body") is not True
        or geometry.get("separation_micrometres") != [0, 0, 215000]
        or geometry.get("resultant_force_is_exact_zero") is not True
        or geometry.get("resultant_moment_is_exact_zero") is not True
        or geometry.get("constraint_rank_upper_bound") != 5
        or geometry.get("minimum_force_nullity") != 1
        or not all(report.get("discriminators", {}).values())
        or report.get("local_system_reconstructions") != 0
        or report.get("singular_value_decompositions") != 0
        or report.get("local_system_solves") != 0
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("profile_sha256") != expected["profile_sha256"]
        or identities.get("research_module_sha256") != expected["module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
    ):
        raise ValueError("R125 research source contract differs")
    return report


def validate_r121(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    formulation_profile_path: Path,
    formulation_module_path: Path,
    formulation_tool_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r121"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(formulation_profile_path) != expected["profile_sha256"]
        or sha256(formulation_module_path) != expected["module_sha256"]
        or sha256(formulation_tool_path) != expected["tool_sha256"]
    ):
        raise ValueError("R125 R121 source file identity differs")
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R121")
    identities = report.get("identities", {})
    if (
        report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R122_FIXED_PD_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY"
        or report.get("scope", {}).get("physics_collocation_count") != COLLOCATION_COUNT
        or report.get("scope", {}).get("generalized_velocity_width")
        != GENERALIZED_WIDTH
        or report.get("contact_schedule_audit", {}).get("status") != "PASS"
        or report.get("system_inventory_audit", {}).get(
            "friction_second_order_cone_count"
        )
        != 4956
        or report.get("local_system_solves") != 0
        or report.get("inverse_dynamics_solves") != 0
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("profile_sha256") != expected["profile_sha256"]
        or identities.get("formulation_module_sha256") != expected["module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
    ):
        raise ValueError("R125 R121 source contract differs")
    return report


def _validate_canonical_report(
    report: Mapping[str, Any], expected_sha256: str, source_name: str
) -> None:
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected_sha256
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError(f"R125 {source_name} canonical identity differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R125 validation results differ")
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
        raise ValueError("R125 requires a clean repository")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    numeric = profile.get("numeric_contract", {})
    conformance = profile.get("implementation_conformance_contract", {})
    budget = profile.get("future_single_execution_budget", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "GaugeAwareStage2PointwiseFixedPdConeFeasibilityFormulationOnly"
        or scope.get("run_id") != "R125"
        or scope.get("physics_collocation_count") != COLLOCATION_COUNT
        or scope.get("formulation_runs") != 1
        or scope.get("local_system_reconstructions") != 0
        or scope.get("local_system_solves") != 0
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or conformance.get("run_id") != "R126"
        or conformance.get("execution_in_r126") is not False
        or budget.get("run_id") != "R127"
        or budget.get("maximum_local_unknown_count") != 35
        or budget.get("maximum_flat_foot_gauge_dimension") != 1
        or profile.get("decision", {}).get("complete")
        != "PERMIT_R126_REDUNDANT_CONTACT_IMPLEMENTATION_CONFORMANCE_ONLY"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "r123_retry",
                "r124_formulation",
                "r127_execution",
                "additional_kto_solve",
                "additional_inverse_dynamics_solve",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or bounded.get("r126_implementation_conformance")
        != "AUTHORIZED_REPORT_ONLY_ON_R125_COMPLETE"
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
        raise ValueError("R125 profile differs")
