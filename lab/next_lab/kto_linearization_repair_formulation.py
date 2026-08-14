from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

from next_lab.contact_target_knot_formulation import canonical_json, sha256

FORMULATION_ID = "nextengine.humanoid-kto-linearization-repair-formulation.v1"
CHECK_ID = "TRAIN-4-KTO-LINEARIZATION-REPAIR-FORMULATION"
FRACTIONS = ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
ZERO_EXECUTION_COUNTERS = (
    "solver_runs",
    "qp_solves",
    "kto_solves",
    "inverse_dynamics_solves",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "solver_private_warm_start_caches",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


def build_kto_linearization_repair_formulation(
    *,
    profile_path: Path,
    r115_rc1_report_path: Path,
    r115_rc1_profile_path: Path,
    r115_report_path: Path,
    r115_profile_path: Path,
    r114_report_path: Path,
    r114_profile_path: Path,
    contact_manifold_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R117's repair contract without implementing or solving it."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r115_rc1_report_path,
            r115_rc1_profile_path,
            r115_report_path,
            r115_profile_path,
            r114_report_path,
            r114_profile_path,
            contact_manifold_path,
            tool_path,
        )
    )
    (
        profile_path,
        r115_rc1_report_path,
        r115_rc1_profile_path,
        r115_report_path,
        r115_profile_path,
        r114_report_path,
        r114_profile_path,
        contact_manifold_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R117 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    rc1 = _validate_rc1(
        profile=profile,
        report_path=r115_rc1_report_path,
        profile_path=r115_rc1_profile_path,
    )
    r115 = _validate_r115(
        profile=profile,
        report_path=r115_report_path,
        profile_path=r115_profile_path,
    )
    r114 = _validate_r114(
        profile=profile,
        report_path=r114_report_path,
        profile_path=r114_profile_path,
    )
    if (
        sha256(contact_manifold_path)
        != profile["source"]["contact_manifold_module_sha256"]
    ):
        raise ValueError("R117 exact contact-kernel identity differs")
    validations = _validate_results(profile, validation_results)
    bridge_audit = select_historical_bridge(r115["solver_result"]["exact_audits"])
    if (
        bridge_audit["selected_fraction"] != "1/32"
        or bridge_audit["selected_analytic_tangent_excess_micrometres"] != 368
        or bridge_audit["selected_tracking_improvement_basis_points"] != 615
    ):
        raise ValueError("R117 historical bridge discriminator differs")

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "formulation_id": FORMULATION_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "scope": profile["scope"],
        "source_evidence": {
            "r115_rc1": {
                "status": rc1["status"],
                "finding": rc1["finding"],
                "gate_decision": rc1["gate_decision"],
                "report_sha256": rc1["report_sha256"],
                "analytic_row_count": rc1["analytic_row_dependency_audit"]["row_count"],
                "analytic_configuration_nonzero_count": rc1[
                    "analytic_row_dependency_audit"
                ]["configuration_nonzero_count"],
                "exact_hotspot_nonzero_configuration_variable_count": rc1[
                    "hotspot_configuration_sensitivity"
                ]["nonzero_variable_count"],
                "exact_hotspot": rc1["hotspot_configuration_sensitivity"]["point_id"],
            },
            "r115": {
                "status": r115["status"],
                "gate_decision": r115["gate_decision"],
                "termination": r115["solver_result"]["termination"],
                "report_sha256": r115["report_sha256"],
                "qp_solves": r115["qp_solves"],
                "exact_emission_audits": r115["in_memory_emitted_iterates"],
                "candidate_artifacts_built": r115["candidate_artifacts_built"],
            },
            "r114_revision_2": {
                "status": r114["status"],
                "gate_decision": r114["gate_decision"],
                "report_sha256": r114["report_sha256"],
                "formulation_revision": r114["formulation_revision"],
            },
        },
        "research_resolution": profile["research_resolution"],
        "inherited_r114_contract": profile["inherited_r114_contract"],
        "exact_contact_function": profile["exact_contact_function"],
        "complete_derivative_contract": profile["complete_derivative_contract"],
        "nonlinear_iteration_contract": profile["nonlinear_iteration_contract"],
        "required_diagnostics": profile["required_diagnostics"],
        "historical_bridge_policy_audit": bridge_audit,
        "r118_conformance_contract": profile["r118_conformance_contract"],
        "research_basis": profile["research_basis"],
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r115_rc1_report_file_sha256": sha256(r115_rc1_report_path),
            "r115_rc1_profile_sha256": sha256(r115_rc1_profile_path),
            "r115_report_file_sha256": sha256(r115_report_path),
            "r115_profile_sha256": sha256(r115_profile_path),
            "r114_report_file_sha256": sha256(r114_report_path),
            "r114_profile_sha256": sha256(r114_profile_path),
            "contact_manifold_module_sha256": sha256(contact_manifold_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "formulation_reports": 1,
        "research_audits": 0,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def select_historical_bridge(
    audits: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    """Apply R117's bridge ordering to metric-only R115 summaries."""

    if tuple(row.get("fraction") for row in audits) != FRACTIONS:
        raise ValueError("R117 bridge fraction inventory differs")
    rows = []
    eligible_rows = []
    for ordinal, audit in enumerate(audits):
        contact = audit.get("contact", {})
        progress = audit.get("tracking_progress", {})
        baseline = int(
            progress.get("baseline_squared_distance_microradians_squared", 0)
        )
        emitted = int(progress.get("emitted_squared_distance_microradians_squared", 0))
        improvement = (baseline - emitted) * 10_000 // baseline if baseline > 0 else -1
        excess = (
            int(contact.get("maximum_analytic_tangential_step_micrometres", -1)) - 2000
        )
        eligible = _historical_bridge_eligible(audit)
        row = {
            "fraction": audit["fraction"],
            "eligible": eligible,
            "analytic_tangent_excess_micrometres": excess,
            "tracking_improvement_basis_points": improvement,
            "failure_reasons": audit.get("failure_reasons"),
        }
        rows.append(row)
        if eligible:
            eligible_rows.append((excess, -improvement, ordinal, row))
    if not eligible_rows:
        raise ValueError("R117 historical bridge has no eligible fraction")
    selected = min(eligible_rows)[-1]
    return {
        "role": "METRIC_ONLY_POLICY_DISCRIMINATOR_NOT_A_REUSABLE_ITERATE",
        "rows": rows,
        "eligible_fraction_count": len(eligible_rows),
        "selected_fraction": selected["fraction"],
        "selected_analytic_tangent_excess_micrometres": selected[
            "analytic_tangent_excess_micrometres"
        ],
        "selected_tracking_improvement_basis_points": selected[
            "tracking_improvement_basis_points"
        ],
        "r115_arrays_reconstructed": False,
        "r115_direction_reused": False,
    }


def funnel_strictly_decreases(
    current: Sequence[float], candidate: Sequence[float]
) -> bool:
    """Return the exact lexicographic R117 restoration predicate."""

    current_pair = _funnel_pair(current)
    candidate_pair = _funnel_pair(candidate)
    return candidate_pair < current_pair


def _historical_bridge_eligible(audit: Mapping[str, Any]) -> bool:
    contact = audit.get("contact", {})
    progress = audit.get("tracking_progress", {})
    return bool(
        audit.get("status") == "FAIL"
        and audit.get("failure_reasons") == ["contact"]
        and contact.get("status") == "FAIL"
        and int(contact.get("maximum_normal_residual_micrometres", 5001)) <= 5000
        and int(contact.get("maximum_normal_step_micrometres", 1001)) <= 1000
        and int(contact.get("maximum_tangential_step_micrometres", 2001)) <= 2000
        and int(contact.get("maximum_analytic_normal_step_micrometres", 1001)) <= 1000
        and int(contact.get("maximum_analytic_tangential_step_micrometres", 0)) > 2000
        and int(audit.get("minimum_collider_height_micrometres", -3)) >= -2
        and int(
            audit.get("maximum_root_vertical_velocity_micrometres_per_second", 200061)
        )
        <= 200060
        and int(audit.get("maximum_joint_velocity_basis_points", 2501)) <= 2500
        and int(audit.get("maximum_descriptor_rom_violation_microradians", 1)) == 0
        and int(audit.get("maximum_effective_rom_violation_microradians", 1)) == 0
        and audit.get("endpoint_identity") == "PASS"
        and progress.get("status") == "PASS"
        and progress.get("strict_distance_decrease") is True
        and progress.get("strict_positive_directional_dot") is True
        and int(progress.get("changed_joint_position_cell_count", 0)) > 0
    )


def _funnel_pair(values: Sequence[float]) -> tuple[float, float]:
    if len(values) != 2:
        raise ValueError("R117 funnel must contain maximum and sum")
    pair = tuple(float(value) for value in values)
    if any(not math.isfinite(value) or value < 0.0 for value in pair):
        raise ValueError("R117 funnel values must be finite and nonnegative")
    if pair[1] + 1.0e-15 < pair[0]:
        raise ValueError("R117 funnel sum cannot be smaller than its maximum")
    return pair


def _validate_rc1(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r115_rc1"]
    _validate_file_identities(report_path, profile_path, expected)
    report = _load_canonical_report(report_path, expected["report_sha256"])
    dependency = report.get("analytic_row_dependency_audit", {})
    sensitivity = report.get("hotspot_configuration_sensitivity", {})
    if (
        report.get("check") != "TRAIN-4-POST-R115-KTO-LINEARIZATION-RESEARCH"
        or report.get("status") != "COMPLETE"
        or report.get("finding")
        != "CONFIRMED_ANALYTIC_CONTACT_LINEARIZATION_IDENTITY_GAP"
        or report.get("gate_decision")
        != "PERMIT_REPORT_ONLY_KTO_LINEARIZATION_REPAIR_FORMULATION_ONLY"
        or dependency.get("row_count") != 3723
        or dependency.get("configuration_nonzero_count") != 0
        or sensitivity.get("point_id") != "frame-328:left-forefoot"
        or sensitivity.get("nonzero_variable_count") != 9
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or any(int(report.get(counter, -1)) != 0 for counter in ZERO_EXECUTION_COUNTERS)
    ):
        raise ValueError("R117 R115-RC1 source contract differs")
    identities = report.get("identities", {})
    if (
        identities.get("research_module_sha256") != expected["research_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
    ):
        raise ValueError("R117 R115-RC1 implementation identity differs")
    return report


def _validate_r115(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r115"]
    _validate_file_identities(report_path, profile_path, expected)
    report = _load_canonical_report(report_path, expected["report_sha256"])
    solver = report.get("solver_result", {})
    identities = report.get("identities", {})
    if (
        report.get("check") != "TRAIN-4-QUANTIZATION-AWARE-KTO-EXECUTION"
        or report.get("status") != "FAIL"
        or report.get("gate_decision") != "STOP_AND_RESEARCH"
        or solver.get("termination") != "no_exact_progress_step"
        or report.get("qp_solves") != 1
        or report.get("kto_solves") != 1
        or report.get("in_memory_emitted_iterates") != 6
        or report.get("candidate_artifacts_built") != 0
        or report.get("physx_scene_runs") != 0
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("execution_module_sha256")
        != expected["execution_module_sha256"]
        or identities.get("solver_module_sha256") != expected["solver_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
    ):
        raise ValueError("R117 R115 source contract differs")
    return report


def _validate_r114(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r114"]
    _validate_file_identities(report_path, profile_path, expected)
    report = _load_canonical_report(report_path, expected["report_sha256"])
    if (
        report.get("check") != "TRAIN-4-QUANTIZATION-AWARE-KTO-EXECUTION-FORMULATION"
        or report.get("status") != "COMPLETE"
        or report.get("formulation_revision") != 2
        or report.get("gate_decision")
        != "PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY"
        or report.get("kto_solves") != 0
        or report.get("candidate_artifacts_built") != 0
        or report.get("physx_scene_runs") != 0
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
    ):
        raise ValueError("R117 R114 source contract differs")
    return report


def _validate_file_identities(
    report_path: Path, profile_path: Path, expected: Mapping[str, Any]
) -> None:
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R117 source file identity differs")


def _load_canonical_report(path: Path, expected_hash: str) -> dict[str, Any]:
    report = json.loads(path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected_hash
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError("R117 source report is not canonical")
    return report


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R117 validation result differs")
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
        raise ValueError("R117 formulation requires a clean repository")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    exact = profile.get("exact_contact_function", {})
    derivative = profile.get("complete_derivative_contract", {})
    iteration = profile.get("nonlinear_iteration_contract", {})
    bounds = iteration.get("bounds", {})
    r118 = profile.get("r118_conformance_contract", {})
    tolerances = r118.get("numeric_tolerances", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "ExactKernelKtoLinearizationRepairFormulationOnly"
        or scope.get("run_id") != "R117"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("frame_count") != 801
        or scope.get("decision_scalar_count") != 69687
        or scope.get("qp_solves") != 0
        or scope.get("kto_solves") != 0
        or scope.get("physx_scene_runs") != 0
        or exact.get("tangential_constraint")
        != "velocity_x_m_s^2 + velocity_z_m_s^2 <= 0.12^2; this is the differentiable equivalent of the exact 2000-micrometre step norm at 60 Hz, not a component box"
        or len(derivative.get("configuration_terms", ())) != 3
        or len(derivative.get("velocity_terms", ())) != 3
        or tuple(iteration.get("line_search_fractions", ())) != FRACTIONS
        or iteration.get("single_bridge", {}).get("maximum_count") != 1
        or bounds.get("maximum_major_iterations") != 12
        or bounds.get("maximum_qp_solves") != 12
        or bounds.get("maximum_exact_emission_audits") != 72
        or bounds.get("randomized_restarts") != 0
        or r118.get("scope")
        != "report-only implementation and numerical conformance; zero QP and KTO solves"
        or tolerances
        != {
            "function_identity_absolute_micrometres_per_second_per_component": 1.0,
            "jacobian_absolute_micrometres_per_second_per_variable_unit": 5.0,
            "jacobian_relative": 0.0001,
            "configuration_nonzero_micrometres_per_second_per_radian": 1.0,
        }
        or len(r118.get("deterministic_anchor_requirements", ())) != 4
        or profile.get("decision", {}).get("complete")
        != "PERMIT_R118_KTO_LINEARIZATION_REPAIR_IMPLEMENTATION_CONFORMANCE_ONLY"
        or bounded.get("r118_linearization_repair_implementation_conformance")
        != "AUTHORIZED_REPORT_ONLY"
        or bounded.get("r119_repaired_kto_execution_formulation")
        != "NOT_AUTHORIZED_UNTIL_R118_PASS"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_kto_solve",
                "r116_inverse_dynamics_formulation",
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
        != ("ruff_check", "ruff_format", "lab_full", "motor", "host_check")
    ):
        raise ValueError("R117 formulation profile differs")
