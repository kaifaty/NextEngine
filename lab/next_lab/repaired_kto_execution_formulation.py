from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

from next_lab.contact_target_knot_formulation import canonical_json, sha256

FORMULATION_ID = "nextengine.humanoid-repaired-kto-execution-formulation.v1"
CHECK_ID = "TRAIN-4-REPAIRED-KTO-EXECUTION-FORMULATION"
FRACTIONS = ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
R114_TOTAL_CONSTRAINT_ROWS = 131_180
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


def build_repaired_kto_execution_formulation(
    *,
    profile_path: Path,
    r118_report_path: Path,
    r118_profile_path: Path,
    r117_report_path: Path,
    r117_profile_path: Path,
    r114_report_path: Path,
    r114_profile_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    solver_import_audit: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R119 without importing or executing the repaired solver."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r118_report_path,
            r118_profile_path,
            r117_report_path,
            r117_profile_path,
            r114_report_path,
            r114_profile_path,
            tool_path,
        )
    )
    (
        profile_path,
        r118_report_path,
        r118_profile_path,
        r117_report_path,
        r117_profile_path,
        r114_report_path,
        r114_profile_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R119 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_solver_import_audit(solver_import_audit)
    r118 = _validate_r118(
        profile=profile,
        report_path=r118_report_path,
        profile_path=r118_profile_path,
    )
    r117 = _validate_r117(
        profile=profile,
        report_path=r117_report_path,
        profile_path=r117_profile_path,
    )
    r114 = _validate_r114(
        profile=profile,
        report_path=r114_report_path,
        profile_path=r114_profile_path,
    )
    bound_implementations = _validate_bound_implementations(profile)
    _validate_cross_source_lineage(r118=r118, r117=r117, r114=r114)
    validations = _validate_results(profile, validation_results)

    active_count = int(r118["all_active_function_identity"]["active_point_frame_count"])
    inventory = repaired_contact_row_inventory(active_count)
    expected_inventory = {
        "active_point_frame_count": profile["repaired_contact_row_contract"][
            "active_point_frame_count"
        ],
        "retired_component_row_count": profile["repaired_contact_row_contract"][
            "retired_r115_component_row_count"
        ],
        "normal_row_count": profile["repaired_contact_row_contract"][
            "normal_row_count"
        ],
        "tangential_norm_squared_row_count": profile["repaired_contact_row_contract"][
            "tangential_norm_squared_row_count"
        ],
        "replacement_row_count": profile["repaired_contact_row_contract"][
            "replacement_row_count"
        ],
        "resulting_total_constraint_row_count": profile[
            "repaired_contact_row_contract"
        ]["resulting_total_constraint_row_count"],
    }
    if inventory != expected_inventory:
        raise ValueError("R119 repaired contact-row inventory differs")
    anchor_projections = [
        project_anchor_contact_rows(anchor)
        for anchor in r118["deterministic_anchor_audits"]
    ]
    if not all(row["status"] == "PASS" for row in anchor_projections):
        raise ValueError("R119 anchor row projection differs")
    budget_audit = audit_execution_budget(profile["execution_budget"])
    bridge = r117["historical_bridge_policy_audit"]
    historical_bridge = {
        "role": bridge["role"],
        "selected_fraction": bridge["selected_fraction"],
        "selected_analytic_tangent_excess_micrometres": bridge[
            "selected_analytic_tangent_excess_micrometres"
        ],
        "selected_tracking_improvement_basis_points": bridge[
            "selected_tracking_improvement_basis_points"
        ],
        "r115_arrays_reconstructed": bridge["r115_arrays_reconstructed"],
        "r115_direction_reused": bridge["r115_direction_reused"],
        "execution_input": False,
    }
    if historical_bridge != {
        "role": "METRIC_ONLY_POLICY_DISCRIMINATOR_NOT_A_REUSABLE_ITERATE",
        "selected_fraction": "1/32",
        "selected_analytic_tangent_excess_micrometres": 368,
        "selected_tracking_improvement_basis_points": 615,
        "r115_arrays_reconstructed": False,
        "r115_direction_reused": False,
        "execution_input": False,
    }:
        raise ValueError("R119 historical bridge disposition differs")

    discriminators = {
        "r118_pass_identity": r118["status"] == "PASS",
        "r117_repair_contract_complete": r117["status"] == "COMPLETE",
        "r114_revision_2_inherited": r114["formulation_revision"] == 2,
        "bound_implementation_identity": all(
            row["status"] == "PASS" for row in bound_implementations
        ),
        "repaired_contact_row_inventory": inventory == expected_inventory,
        "anchor_scalar_row_projection": len(anchor_projections) == 7
        and all(row["status"] == "PASS" for row in anchor_projections),
        "quantized_anchor_rebase_explicit": (
            "current anchor's exact emitted"
            in profile["anchor_rebase_contract"]["zero_perturbation_identity"]
            and "rederive every" in profile["anchor_rebase_contract"]["accepted_rebase"]
        ),
        "historical_r115_state_not_reused": not historical_bridge[
            "r115_arrays_reconstructed"
        ]
        and not historical_bridge["r115_direction_reused"]
        and not historical_bridge["execution_input"],
        "budget_closure": budget_audit["status"] == "PASS",
        "solver_free_process": not any(solver_import_audit.values()),
        "zero_execution_counters": True,
    }
    if not all(discriminators.values()):
        raise ValueError("R119 formulation discriminator differs")

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "formulation_id": FORMULATION_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "scope": profile["scope"],
        "source_evidence": {
            "r118": {
                "status": r118["status"],
                "gate_decision": r118["gate_decision"],
                "report_sha256": r118["report_sha256"],
                "active_point_frame_count": active_count,
                "anchor_count": len(r118["deterministic_anchor_audits"]),
                "maximum_function_difference_micrometres_per_second": r118[
                    "all_active_function_identity"
                ]["maximum_absolute_component_difference_micrometres_per_second"],
            },
            "r117": {
                "status": r117["status"],
                "gate_decision": r117["gate_decision"],
                "report_sha256": r117["report_sha256"],
            },
            "r114_revision_2": {
                "status": r114["status"],
                "formulation_revision": r114["formulation_revision"],
                "report_sha256": r114["report_sha256"],
            },
        },
        "bound_implementation_identity_audit": bound_implementations,
        "inheritance_contract": profile["inheritance_contract"],
        "repaired_contact_row_contract": profile["repaired_contact_row_contract"],
        "repaired_contact_row_inventory_audit": inventory,
        "anchor_scalar_row_projection_audits": anchor_projections,
        "anchor_rebase_contract": profile["anchor_rebase_contract"],
        "nonlinear_execution_contract": profile["nonlinear_execution_contract"],
        "historical_bridge_disposition": historical_bridge,
        "pre_solver_contract": profile["pre_solver_contract"],
        "execution_budget": profile["execution_budget"],
        "execution_budget_audit": budget_audit,
        "required_diagnostics": profile["required_diagnostics"],
        "result_transitions": profile["result_transitions"],
        "discriminators": discriminators,
        "solver_import_audit": dict(solver_import_audit),
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r118_report_file_sha256": sha256(r118_report_path),
            "r118_profile_sha256": sha256(r118_profile_path),
            "r117_report_file_sha256": sha256(r117_report_path),
            "r117_profile_sha256": sha256(r117_profile_path),
            "r114_report_file_sha256": sha256(r114_report_path),
            "r114_profile_sha256": sha256(r114_profile_path),
            "r118_conformance_module_sha256": r118["identities"][
                "conformance_module_sha256"
            ],
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "repaired_execution_formulations": 1,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def repaired_contact_row_inventory(active_point_frame_count: int) -> dict[str, int]:
    if active_point_frame_count <= 0:
        raise ValueError("R119 active-point count must be positive")
    retired = 3 * active_point_frame_count
    normal = active_point_frame_count
    tangent = active_point_frame_count
    replacement = normal + tangent
    return {
        "active_point_frame_count": active_point_frame_count,
        "retired_component_row_count": retired,
        "normal_row_count": normal,
        "tangential_norm_squared_row_count": tangent,
        "replacement_row_count": replacement,
        "resulting_total_constraint_row_count": (
            R114_TOTAL_CONSTRAINT_ROWS - retired + replacement
        ),
    }


def project_anchor_contact_rows(anchor: Mapping[str, Any]) -> dict[str, Any]:
    velocity_um_s = tuple(
        float(value)
        for value in anchor["production_baseline_velocity_micrometres_per_second"]
    )
    if len(velocity_um_s) != 3 or any(
        not math.isfinite(value) for value in velocity_um_s
    ):
        raise ValueError("R119 anchor velocity differs")
    velocity_m_s = tuple(value / 1_000_000.0 for value in velocity_um_s)
    rows = anchor["variable_rows"]
    normal_coefficients = []
    tangent_coefficients = []
    configuration_tangent_nonzero = 0
    velocity_tangent_nonzero = 0
    for row in rows:
        vector = tuple(
            float(value) / 1_000_000.0
            for value in row["production_xyz_micrometres_per_second_per_unit"]
        )
        if len(vector) != 3 or any(not math.isfinite(value) for value in vector):
            raise ValueError("R119 anchor vector derivative differs")
        normal_coefficients.append(vector[1])
        tangent = 2.0 * (velocity_m_s[0] * vector[0] + velocity_m_s[2] * vector[2])
        tangent_coefficients.append(tangent)
        if abs(tangent) > 1.0e-14:
            if row["block"] == "configuration":
                configuration_tangent_nonzero += 1
            elif row["block"] == "velocity":
                velocity_tangent_nonzero += 1
            else:
                raise ValueError("R119 anchor variable block differs")
    tangent_value = velocity_m_s[0] ** 2 + velocity_m_s[2] ** 2
    finite = all(
        math.isfinite(value)
        for value in (*normal_coefficients, *tangent_coefficients, tangent_value)
    )
    passed = bool(
        anchor.get("status") == "PASS"
        and finite
        and len(rows) in (61, 64)
        and abs(velocity_m_s[1]) <= 0.06 + 1.0e-12
        and tangent_value <= 0.0144 + 1.0e-12
        and configuration_tangent_nonzero > 0
        and velocity_tangent_nonzero > 0
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "point_id": anchor["point_id"],
        "variable_count": len(rows),
        "normal_baseline_metres_per_second": velocity_m_s[1],
        "normal_lower_rhs_metres_per_second": -0.06 - velocity_m_s[1],
        "normal_upper_rhs_metres_per_second": 0.06 - velocity_m_s[1],
        "normal_coefficient_nonzero_count": sum(
            abs(value) > 1.0e-14 for value in normal_coefficients
        ),
        "tangential_baseline_square_metres_per_square_second": tangent_value,
        "tangential_upper_rhs_square_metres_per_square_second": (
            0.0144 - tangent_value
        ),
        "tangential_coefficient_nonzero_count": sum(
            abs(value) > 1.0e-14 for value in tangent_coefficients
        ),
        "tangential_configuration_nonzero_count": configuration_tangent_nonzero,
        "tangential_velocity_nonzero_count": velocity_tangent_nonzero,
        "maximum_absolute_normal_coefficient": max(
            (abs(value) for value in normal_coefficients), default=0.0
        ),
        "maximum_absolute_tangential_coefficient": max(
            (abs(value) for value in tangent_coefficients), default=0.0
        ),
    }


def audit_execution_budget(budget: Mapping[str, Any]) -> dict[str, Any]:
    fractions = tuple(budget.get("line_search_fractions", FRACTIONS))
    major = int(budget.get("maximum_major_iterations", -1))
    qp = int(budget.get("maximum_qp_solves", -1))
    audits = int(budget.get("maximum_exact_emission_audits", -1))
    passed = bool(
        fractions == FRACTIONS
        and major == 12
        and qp == 12
        and audits == major * len(fractions) == 72
        and int(budget.get("kto_process_count", -1)) == 1
        and int(budget.get("kto_solve_count", -1)) == 1
        and int(budget.get("thread_count", -1)) == 1
        and int(budget.get("randomized_restart_count", -1)) == 0
        and budget.get("resume_or_warm_restart") == "FORBIDDEN"
        and budget.get("manual_intervention") == "FORBIDDEN"
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "maximum_major_iterations": major,
        "maximum_qp_solves": qp,
        "fraction_count_per_major_iteration": len(fractions),
        "derived_maximum_exact_emission_audits": major * len(fractions),
        "declared_maximum_exact_emission_audits": audits,
        "one_outer_execution_not_twelve_kto_solves": (
            int(budget.get("kto_solve_count", -1)) == 1 and qp == 12
        ),
    }


def _validate_r118(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r118"]
    report = _load_bound_report(report_path, profile_path, expected)
    identities = report.get("identities", {})
    baseline = report.get("all_active_function_identity", {})
    anchors = report.get("deterministic_anchor_audits", ())
    if (
        report.get("check") != "TRAIN-4-KTO-LINEARIZATION-REPAIR-CONFORMANCE"
        or report.get("status") != "PASS"
        or report.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R119_REPAIRED_KTO_EXECUTION_FORMULATION_ONLY"
        or baseline.get("status") != "PASS"
        or baseline.get("active_point_frame_count") != 1241
        or baseline.get("differing_component_count") != 0
        or len(anchors) != 7
        or any(anchor.get("status") != "PASS" for anchor in anchors)
        or not all(report.get("discriminators", {}).values())
        or identities.get("conformance_module_sha256")
        != expected["conformance_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or any(int(report.get(counter, -1)) != 0 for counter in ZERO_EXECUTION_COUNTERS)
    ):
        raise ValueError("R119 R118 source contract differs")
    return report


def _validate_r117(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r117"]
    report = _load_bound_report(report_path, profile_path, expected)
    identities = report.get("identities", {})
    if (
        report.get("check") != "TRAIN-4-KTO-LINEARIZATION-REPAIR-FORMULATION"
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R118_KTO_LINEARIZATION_REPAIR_IMPLEMENTATION_CONFORMANCE_ONLY"
        or report.get("scope", {}).get("decision_scalar_count") != 69687
        or identities.get("formulation_module_sha256")
        != expected["formulation_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or any(int(report.get(counter, -1)) != 0 for counter in ZERO_EXECUTION_COUNTERS)
    ):
        raise ValueError("R119 R117 source contract differs")
    return report


def _validate_r114(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r114"]
    report = _load_bound_report(report_path, profile_path, expected)
    identities = report.get("identities", {})
    if (
        report.get("check") != "TRAIN-4-QUANTIZATION-AWARE-KTO-EXECUTION-FORMULATION"
        or report.get("status") != "COMPLETE"
        or report.get("formulation_revision") != 2
        or report.get("gate_decision")
        != "PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY"
        or identities.get("formulation_module_sha256")
        != expected["formulation_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(counter, -1)) != 0
            for counter in ZERO_EXECUTION_COUNTERS
            if counter != "qp_solves"
        )
    ):
        raise ValueError("R119 R114 source contract differs")
    return report


def _load_bound_report(
    report_path: Path,
    profile_path: Path,
    expected: Mapping[str, Any],
) -> dict[str, Any]:
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R119 source file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError("R119 source canonical identity differs")
    return report


def _validate_cross_source_lineage(
    *, r118: Mapping[str, Any], r117: Mapping[str, Any], r114: Mapping[str, Any]
) -> None:
    if (
        r118.get("source_evidence", {}).get("r117", {}).get("report_sha256")
        != r117.get("report_sha256")
        or r117.get("source_evidence", {})
        .get("r114_revision_2", {})
        .get("report_sha256")
        != r114.get("report_sha256")
        or r118.get("identities", {}).get("r117_profile_sha256")
        != r117.get("identities", {}).get("profile_sha256")
        or r117.get("identities", {}).get("r114_profile_sha256")
        != r114.get("identities", {}).get("profile_sha256")
    ):
        raise ValueError("R119 cross-source lineage differs")


def _validate_bound_implementations(
    profile: Mapping[str, Any],
) -> list[dict[str, str]]:
    module_directory = Path(__file__).resolve().parent
    lab_directory = module_directory.parent
    expected = profile["source"]
    sources = (
        (
            "r118_conformance_module",
            module_directory / "kto_linearization_repair_conformance.py",
            expected["r118"]["conformance_module_sha256"],
        ),
        (
            "r118_tool",
            lab_directory / "scripts/audit_kto_linearization_repair_conformance.py",
            expected["r118"]["tool_sha256"],
        ),
        (
            "r117_formulation_module",
            module_directory / "kto_linearization_repair_formulation.py",
            expected["r117"]["formulation_module_sha256"],
        ),
        (
            "r117_tool",
            lab_directory / "scripts/formulate_kto_linearization_repair.py",
            expected["r117"]["tool_sha256"],
        ),
        (
            "r114_formulation_module",
            module_directory / "quantization_aware_kto_formulation.py",
            expected["r114"]["formulation_module_sha256"],
        ),
        (
            "r114_tool",
            lab_directory / "scripts/formulate_quantization_aware_kto.py",
            expected["r114"]["tool_sha256"],
        ),
    )
    results = []
    for source_id, path, expected_hash in sources:
        actual = sha256(path)
        if actual != expected_hash:
            raise ValueError(f"R119 bound implementation {source_id} differs")
        results.append(
            {
                "id": source_id,
                "status": "PASS",
                "sha256": actual,
            }
        )
    return results


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R119 validation result differs")
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
        raise ValueError("R119 formulation requires a clean repository")


def _validate_solver_import_audit(audit: Mapping[str, Any]) -> None:
    if audit != {
        "osqp_module_loaded": False,
        "solver_module_imported": False,
        "execution_module_imported": False,
        "solver_execution_requested": False,
    }:
        raise ValueError("R119 solver-free import audit differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    rows = profile.get("repaired_contact_row_contract", {})
    probes = rows.get("derivative_probes", {})
    rebase = profile.get("anchor_rebase_contract", {})
    iteration = profile.get("nonlinear_execution_contract", {})
    tolerances = profile.get("pre_solver_contract", {}).get(
        "conformance_tolerances", {}
    )
    budget = profile.get("execution_budget", {})
    transitions = profile.get("result_transitions", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "RepairedExactKernelKtoExecutionFormulationOnly"
        or source.get("r118", {}).get("report_sha256")
        != "23d9d556d8be630f7f2e9fe9907f394b74186ef545d0c46f7484f0efc1df45ca"
        or source.get("r117", {}).get("report_sha256")
        != "b0a9f07012e0f43c660019df8a1f31e7368f6130312231cf6c2bddb0f59fb7c7"
        or source.get("r114", {}).get("report_sha256")
        != "7a735320509a303f9feacba79087f2042d451d526b57585d4fe4041603b46ae4"
        or scope
        != {
            "run_id": "R119",
            "clip_id": "cmu16-walk-nominal-b",
            "frame_count": 801,
            "decision_scalar_count": 69687,
            "repaired_execution_formulations": 1,
            "qp_solves": 0,
            "kto_solves": 0,
            "candidate_construction": False,
            "physx_scene_runs": 0,
            "training": False,
        }
        or tuple(profile.get("inheritance_contract", {}).get("replaced_only", ()))
        != (
            "analytic contact continuous function",
            "complete analytic contact q/v derivative and scalar chain rule",
            "nonlinear iterate bridge/restoration globalization",
            "analytic contact row diagnostics",
        )
        or rows.get("active_point_frame_count") != 1241
        or rows.get("retired_r115_component_row_count") != 3723
        or rows.get("normal_row_count") != 1241
        or rows.get("tangential_norm_squared_row_count") != 1241
        or rows.get("replacement_row_count") != 2482
        or rows.get("resulting_total_constraint_row_count") != 129939
        or rows.get("normal_row_scale") != 0.06
        or rows.get("tangential_row_scale") != 0.0144
        or rows.get("tangential_function") != "g(x) = vx(x)^2 + vz(x)^2"
        or rows.get("coefficient_elision_absolute_physical") != 1e-14
        or probes
        != {
            "root_translation_metres": 1e-05,
            "root_orientation_radians": 0.0001,
            "joint_position_radians": 0.0001,
            "root_linear_velocity_metres_per_second": 1e-05,
            "root_angular_velocity_radians_per_second": 0.0001,
            "joint_velocity_radians_per_second": 0.0001,
            "exact_kernel_lever_radians": 0.0001,
        }
        or "current anchor's exact emitted"
        not in rebase.get("zero_perturbation_identity", "")
        or "rederive every" not in rebase.get("accepted_rebase", "")
        or tuple(iteration.get("line_search_fractions", ())) != FRACTIONS
        or iteration.get("single_bridge", {}).get("maximum_count") != 1
        or "before every QP setup" not in iteration.get("local_conformance_guard", "")
        or tolerances
        != {
            "function_identity_absolute_micrometres_per_second_per_component": 1.0,
            "jacobian_absolute_micrometres_per_second_per_variable_unit": 5.0,
            "jacobian_relative": 0.0001,
            "configuration_nonzero_micrometres_per_second_per_radian": 1.0,
            "tangent_chain_rule_absolute_square_metres_per_square_second_per_unit": 5e-06,
        }
        or budget.get("maximum_major_iterations") != 12
        or budget.get("maximum_qp_solves") != 12
        or tuple(budget.get("line_search_fractions", ())) != FRACTIONS
        or budget.get("maximum_exact_emission_audits") != 72
        or budget.get("maximum_wall_clock_seconds") != 14400
        or budget.get("maximum_resident_memory_bytes") != 17179869184
        or budget.get("randomized_restart_count") != 0
        or transitions.get("r119_complete")
        != "PERMIT_R120_SINGLE_BOUNDED_REPAIRED_KTO_EXECUTION_ONLY"
        or profile.get("decision", {}).get("complete")
        != "PERMIT_R120_SINGLE_BOUNDED_REPAIRED_KTO_EXECUTION_ONLY"
        or bounded.get("r120_repaired_kto_execution")
        != "AUTHORIZED_ONE_IN_MEMORY_EXECUTION_ONLY_ON_R119_COMPLETE"
        or bounded.get("r120_solver_private_warm_start_cache")
        != "AUTHORIZED_ONLY_ON_R120_EXACT_PASS"
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
        or bounded.get("r121_inverse_dynamics_formulation")
        != "NOT_AUTHORIZED_UNTIL_R120_EXACT_PASS"
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
        raise ValueError("R119 formulation profile differs")
