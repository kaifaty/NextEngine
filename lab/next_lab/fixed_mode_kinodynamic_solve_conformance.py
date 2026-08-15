from __future__ import annotations

import hashlib
import json
import math
import os
from collections.abc import Mapping, Sequence
from decimal import ROUND_HALF_EVEN, Decimal
from pathlib import Path
from typing import Any

CONFORMANCE_ID = "nextengine.humanoid-fixed-mode-kinodynamic-solve-conformance.v1"
CHECK_ID = "TRAIN-4-FIXED-MODE-KINODYNAMIC-SOLVE-CONFORMANCE"
NULL_SINGULAR_VALUE_MAXIMUM = 1e-12
RETAINED_SINGULAR_VALUE_MINIMUM = 1e-10
EXPECTED_CONTACT_RANKS = frozenset({0, 3, 5})
COEFFICIENT_ELISION_MAXIMUM = 1e-14
SOLVER_RESIDUAL_MAXIMUM = 5e-5
TRUST_RADIUS_MINIMUM = Decimal("0.03125")
TRUST_RADIUS_MAXIMUM = Decimal(2)
TRUST_CONTRACTION_FACTOR = Decimal("0.5")
TRUST_EXPANSION_FACTOR = Decimal(2)
MINIMUM_FUNNEL_DECREASE = Decimal("0.000001")
MINIMUM_ACCEPTANCE_RATIO = Decimal("0.1")
CONTRACTION_RATIO_THRESHOLD = Decimal("0.25")
EXPANSION_RATIO_THRESHOLD = Decimal("0.75")
QP_OPTIMUM_HOLD_TOLERANCE = Decimal("0.00000001")
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
    "real_qp_setups",
    "real_qp_solves",
    "real_kinodynamic_solves",
    "real_factorizations",
    "real_exact_trial_audits",
    "optimizer_steps",
    "candidate_artifacts_built",
    "solver_private_caches_built",
    "physx_scene_runs",
    "training_runs",
)
BOUNDED_ACCEPTANCE_KEYS = frozenset(
    {
        "r141_bounded_kinodynamic_solve_roadmap_decision",
        "r141_bounded_kinodynamic_solve",
        "r136_retry",
        "r137_retry",
        "r138_retry",
        "r139_retry",
        "additional_pointwise_inverse_dynamics",
        "real_state_reconstruction",
        "real_controller_graph_evaluation",
        "real_kinodynamic_assembly",
        "qp_solve",
        "kinodynamic_solve",
        "contact_semantics_change",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
)
REQUIRED_PRE_GRAPH_HASH_KEYS = frozenset(
    {
        "r120_report",
        "r120_cache",
        "r120_accepted_arrays",
        "r133_report",
        "r133_projected_velocity",
        "r133_applied_target",
        "r133_applied_effort",
        "r131_mode_sequence",
        "r138_graph_index",
        "r138_transition_replay",
    }
)
REQUIRED_OUTPUT_HASH_KEYS = frozenset(
    {
        "q",
        "v",
        "a",
        "integer_command",
        "exact_applied_target",
        "exact_applied_effort",
        "active_force",
        "activation_impulse",
        "contact_anchor",
        "graph_row_address",
        "controller_event_address",
        "controller_branch_address",
        "exact_residual_vector",
        "exact_funnel_vector",
    }
)


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_fixed_mode_kinodynamic_solve_conformance(
    *,
    profile_path: Path,
    authority_document_path: Path,
    r139_report_path: Path,
    r139_profile_path: Path,
    r139_module_path: Path,
    r139_tool_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Conform the R139 method with synthetic and metadata-only cases."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            authority_document_path,
            r139_report_path,
            r139_profile_path,
            r139_module_path,
            r139_tool_path,
            tool_path,
        )
    )
    (
        profile_path,
        authority_document_path,
        r139_report_path,
        r139_profile_path,
        r139_module_path,
        r139_tool_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R140 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    source = profile["source"]
    r139 = _load_bound_report(r139_report_path, source["r139"])
    _validate_source_files(
        profile=profile,
        authority_document_path=authority_document_path,
        r139_profile_path=r139_profile_path,
        r139_module_path=r139_module_path,
        r139_tool_path=r139_tool_path,
    )
    if (
        sha256(Path(__file__).resolve()) != source["conformance_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R140 current conformance identity differs")

    source_audit = audit_r139_source(profile, r139=r139)
    reconstruction_audit = audit_reconstruction_gate(profile, r139=r139)
    qp_audit = audit_lexicographic_qp_schema(profile)
    trust_audit = audit_trust_funnel_cases(profile)
    controller_audit = audit_controller_boundary_cases(profile)
    cone_audit = audit_friction_cone_cases(profile)
    numeric_audit = audit_numeric_invalid_cases(profile)
    budget_audit = audit_budget_and_hash_closure(profile, r139=r139)
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
        "reconstruction_gate_audit": reconstruction_audit,
        "lexicographic_qp_schema_audit": qp_audit,
        "trust_funnel_audit": trust_audit,
        "controller_boundary_audit": controller_audit,
        "friction_cone_audit": cone_audit,
        "numeric_invalid_audit": numeric_audit,
        "budget_and_hash_closure_audit": budget_audit,
        "conformed_implementation_contract": profile[
            "conformed_implementation_contract"
        ],
        "acceptance_boundary": profile["acceptance_boundary"],
        "future_r141_execution_contract": profile["future_r141_execution_contract"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "authority_document_sha256": sha256(authority_document_path),
            "r139_report_file_sha256": sha256(r139_report_path),
            "r139_profile_sha256": sha256(r139_profile_path),
            "r139_module_sha256": sha256(r139_module_path),
            "r139_tool_sha256": sha256(r139_tool_path),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "source_report_audits": 1,
        "source_array_payloads_read": 0,
        "synthetic_reconstruction_gate_cases": reconstruction_audit["case_count"],
        "synthetic_qp_schema_cases": qp_audit["case_count"],
        "synthetic_trust_cases": trust_audit["case_count"],
        "synthetic_rounding_cases": controller_audit["rounding_case_count"],
        "synthetic_branch_cases": controller_audit["branch_case_count"],
        "synthetic_cone_base_rows": cone_audit["base_half_space_rows"],
        "synthetic_cone_classification_cases": cone_audit["classification_case_count"],
        "synthetic_cone_separation_cases": cone_audit["separation_case_count"],
        "synthetic_numeric_valid_cases": numeric_audit["valid_case_count"],
        "synthetic_numeric_invalid_cases": numeric_audit["invalid_case_count"],
        **{counter: 0 for counter in ZERO_REAL_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_reconstruction_gate(
    expected: Mapping[str, str],
    observed: Mapping[str, str],
    *,
    real_graph_calls: int,
) -> dict[str, Any]:
    if type(real_graph_calls) is not int or real_graph_calls != 0:
        raise ValueError("R140 graph call preceded source hash closure")
    if (
        set(expected) != REQUIRED_PRE_GRAPH_HASH_KEYS
        or set(observed) != REQUIRED_PRE_GRAPH_HASH_KEYS
        or any(
            not isinstance(value, str) or len(value) != 64
            for value in (*expected.values(), *observed.values())
        )
        or dict(observed) != dict(expected)
    ):
        raise ValueError("R140 pre-graph source hash closure differs")
    rows = [{"identity": key, "sha256": expected[key]} for key in sorted(expected)]
    return {
        "status": "PASS",
        "identity_count": len(rows),
        "real_graph_calls_before_closure": 0,
        "ordered_identity_sha256": hashlib.sha256(canonical_json(rows)).hexdigest(),
    }


def audit_reconstruction_gate(
    profile: Mapping[str, Any], *, r139: Mapping[str, Any]
) -> dict[str, Any]:
    contract = profile["reconstruction_gate_contract"]
    expected = contract["required_source_hashes"]
    source_required = r139.get("required_hash_contract", {}).get(
        "before_first_real_graph_call", {}
    )
    if source_required != expected:
        raise ValueError("R140 R139 reconstruction hashes differ")
    passing = validate_reconstruction_gate(expected, expected, real_graph_calls=0)
    rejected: list[dict[str, str]] = []

    changed = dict(expected)
    changed["r120_cache"] = "0" * 64
    for case, observed, graph_calls, expected_error in (
        (
            "mismatched_source_hash",
            changed,
            0,
            "R140 pre-graph source hash closure differs",
        ),
        (
            "premature_real_graph_call",
            expected,
            1,
            "R140 graph call preceded source hash closure",
        ),
    ):
        try:
            validate_reconstruction_gate(
                expected, observed, real_graph_calls=graph_calls
            )
        except ValueError as error:
            if str(error) != expected_error:
                raise ValueError("R140 reconstruction rejection differs") from error
            rejected.append({"case": case, "error": str(error)})
        else:
            raise ValueError("R140 malformed reconstruction case was accepted")
    if len(rejected) != 2:
        raise ValueError("R140 reconstruction case inventory differs")
    return {
        **passing,
        "case_count": 3,
        "passing_case_count": 1,
        "rejected_case_count": len(rejected),
        "rejected_cases": rejected,
    }


def build_lexicographic_qp_schema(
    pass_ordinal: int,
    *,
    maximum_optimum: str = "0.75",
    mean_optimum: str = "0.25",
) -> dict[str, Any]:
    if type(pass_ordinal) is not int or pass_ordinal not in (1, 2, 3):
        raise ValueError("R140 QP pass ordinal differs")
    maximum = Decimal(maximum_optimum)
    mean = Decimal(mean_optimum)
    if not maximum.is_finite() or not mean.is_finite() or maximum < 0 or mean < 0:
        raise ValueError("R140 QP optimum differs")
    objectives = {
        1: "MINIMIZE_MAXIMUM_NORMALIZED_ELASTIC_VIOLATION",
        2: "MINIMIZE_MEAN_NORMALIZED_ELASTIC_VIOLATION",
        3: "MINIMIZE_TRACKING_AND_REGULARIZATION",
    }
    holds: list[dict[str, str]] = []
    if pass_ordinal >= 2:
        holds.append(
            {
                "quantity": "maximum_normalized_elastic_violation",
                "relation": "<=",
                "upper_bound": _decimal_string(maximum + QP_OPTIMUM_HOLD_TOLERANCE),
            }
        )
    if pass_ordinal >= 3:
        holds.append(
            {
                "quantity": "mean_normalized_elastic_violation",
                "relation": "<=",
                "upper_bound": _decimal_string(mean + QP_OPTIMUM_HOLD_TOLERANCE),
            }
        )
    return {
        "pass": pass_ordinal,
        "sparse_form": "L_LE_A_X_LE_U",
        "objective": objectives[pass_ordinal],
        "optimum_hold_rows": holds,
        "temporary_elastics_emitted": False,
        "acceptance_authority": "NONE_CANDIDATE_ONLY",
    }


def audit_lexicographic_qp_schema(profile: Mapping[str, Any]) -> dict[str, Any]:
    contract = profile["lexicographic_qp_schema_contract"]
    cases = [build_lexicographic_qp_schema(ordinal) for ordinal in (1, 2, 3)]
    if cases != contract["expected_passes"]:
        raise ValueError("R140 lexicographic QP schema differs")
    try:
        build_lexicographic_qp_schema(4)
    except ValueError as error:
        if str(error) != "R140 QP pass ordinal differs":
            raise ValueError("R140 malformed QP rejection differs") from error
    else:
        raise ValueError("R140 malformed QP pass was accepted")
    return {
        "status": "PASS",
        "case_count": len(cases),
        "malformed_case_count": 1,
        "ordered_schema_sha256": hashlib.sha256(canonical_json(cases)).hexdigest(),
        "passes": cases,
    }


def decide_major_outcome(
    *,
    radius: str,
    exact_pass: bool,
    exact_maximum_decrease: str,
    exact_mean_decrease: str,
    actual_to_model_ratio: str,
    active_trust_boundary: bool,
) -> dict[str, Any]:
    current = Decimal(radius)
    maximum_decrease = Decimal(exact_maximum_decrease)
    mean_decrease = Decimal(exact_mean_decrease)
    ratio = Decimal(actual_to_model_ratio)
    if (
        not current.is_finite()
        or not maximum_decrease.is_finite()
        or not mean_decrease.is_finite()
        or not ratio.is_finite()
        or not TRUST_RADIUS_MINIMUM <= current <= TRUST_RADIUS_MAXIMUM
    ):
        raise ValueError("R140 trust input differs")
    if exact_pass:
        return {
            "decision": "ACCEPT_AND_TERMINATE_EXACT_PASS",
            "next_radius": _decimal_string(current),
            "restore_previous_exact_anchor": False,
        }

    lexicographic_decrease = maximum_decrease >= MINIMUM_FUNNEL_DECREASE or (
        maximum_decrease >= 0 and mean_decrease >= MINIMUM_FUNNEL_DECREASE
    )
    accepted = lexicographic_decrease and ratio >= MINIMUM_ACCEPTANCE_RATIO
    if not accepted:
        if current == TRUST_RADIUS_MINIMUM:
            return {
                "decision": "STOP_AND_RESEARCH_AT_MINIMUM_RADIUS",
                "next_radius": _decimal_string(current),
                "restore_previous_exact_anchor": True,
            }
        contracted = max(TRUST_RADIUS_MINIMUM, current * TRUST_CONTRACTION_FACTOR)
        return {
            "decision": "REJECT_RESTORE_AND_CONTRACT",
            "next_radius": _decimal_string(contracted),
            "restore_previous_exact_anchor": True,
        }
    if ratio < CONTRACTION_RATIO_THRESHOLD:
        next_radius = max(TRUST_RADIUS_MINIMUM, current * TRUST_CONTRACTION_FACTOR)
        decision = "ACCEPT_AND_CONTRACT"
    elif ratio >= EXPANSION_RATIO_THRESHOLD and active_trust_boundary:
        next_radius = min(TRUST_RADIUS_MAXIMUM, current * TRUST_EXPANSION_FACTOR)
        decision = "ACCEPT_AND_EXPAND"
    else:
        next_radius = current
        decision = "ACCEPT_AND_HOLD"
    return {
        "decision": decision,
        "next_radius": _decimal_string(next_radius),
        "restore_previous_exact_anchor": False,
    }


def audit_trust_funnel_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    contract = profile["trust_funnel_conformance_contract"]
    observed = []
    for row in contract["cases"]:
        outcome = decide_major_outcome(
            radius=row["radius"],
            exact_pass=row["exact_pass"],
            exact_maximum_decrease=row["exact_maximum_decrease"],
            exact_mean_decrease=row["exact_mean_decrease"],
            actual_to_model_ratio=row["actual_to_model_ratio"],
            active_trust_boundary=row["active_trust_boundary"],
        )
        actual = {"case": row["case"], **outcome}
        expected = {
            key: value
            for key, value in row.items()
            if key
            not in {
                "radius",
                "exact_pass",
                "exact_maximum_decrease",
                "exact_mean_decrease",
                "actual_to_model_ratio",
                "active_trust_boundary",
            }
        }
        if actual != expected:
            raise ValueError("R140 trust/funnel outcome differs")
        observed.append(actual)
    return {
        "status": "PASS",
        "case_count": len(observed),
        "ordered_outcome_sha256": hashlib.sha256(canonical_json(observed)).hexdigest(),
        "cases": observed,
    }


def round_fraction_ties_even(numerator: int, denominator: int) -> int:
    if type(numerator) is not int or type(denominator) is not int or denominator <= 0:
        raise ValueError("R140 rational command differs")
    sign = -1 if numerator < 0 else 1
    quotient, remainder = divmod(abs(numerator), denominator)
    doubled = remainder * 2
    if doubled < denominator:
        rounded = quotient
    elif doubled > denominator:
        rounded = quotient + 1
    else:
        rounded = quotient if quotient % 2 == 0 else quotient + 1
    return sign * rounded


def round_decimal_ties_even(numerator: int, denominator: int) -> int:
    if type(numerator) is not int or type(denominator) is not int or denominator <= 0:
        raise ValueError("R140 decimal command differs")
    value = Decimal(numerator) / Decimal(denominator)
    return int(value.quantize(Decimal(1), rounding=ROUND_HALF_EVEN))


def fixed_branch_derivative(value: str, lower: str, upper: str) -> dict[str, Any]:
    numeric = Decimal(value)
    lower_bound = Decimal(lower)
    upper_bound = Decimal(upper)
    if (
        not numeric.is_finite()
        or not lower_bound.is_finite()
        or not upper_bound.is_finite()
        or lower_bound > upper_bound
    ):
        raise ValueError("R140 controller branch input differs")
    if numeric < lower_bound:
        branch, derivative, ambiguous = "LOWER_CLAMP", 0, False
    elif numeric == lower_bound:
        branch, derivative, ambiguous = "LOWER_EQUALITY", 0, True
    elif numeric < upper_bound:
        branch, derivative, ambiguous = "INTERIOR", 1, False
    elif numeric == upper_bound:
        branch, derivative, ambiguous = "UPPER_EQUALITY", 0, True
    else:
        branch, derivative, ambiguous = "UPPER_CLAMP", 0, False
    return {
        "branch": branch,
        "derivative": derivative,
        "ambiguity_recorded": ambiguous,
    }


def audit_controller_boundary_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    contract = profile["controller_boundary_conformance_contract"]
    rounding = []
    for row in contract["rounding_cases"]:
        rational = round_fraction_ties_even(row["numerator"], row["denominator"])
        decimal = round_decimal_ties_even(row["numerator"], row["denominator"])
        if rational != decimal or rational != row["expected_integer"]:
            raise ValueError("R140 ties-to-even differential differs")
        rounding.append(
            {
                "numerator": row["numerator"],
                "denominator": row["denominator"],
                "exact_integer": rational,
                "surrogate_equals_exact_integer": (
                    row["numerator"] == rational * row["denominator"]
                ),
            }
        )
    branches = []
    for row in contract["branch_cases"]:
        actual = {
            "case": row["case"],
            **fixed_branch_derivative(row["value"], row["lower"], row["upper"]),
        }
        expected = {
            key: value
            for key, value in row.items()
            if key not in {"value", "lower", "upper"}
        }
        if actual != expected:
            raise ValueError("R140 fixed controller branch differs")
        branches.append(actual)
    return {
        "status": "PASS",
        "rounding_case_count": len(rounding),
        "rounding_differential_disagreements": 0,
        "surrogate_exact_difference_count": sum(
            not row["surrogate_equals_exact_integer"] for row in rounding
        ),
        "branch_case_count": len(branches),
        "branch_equality_ambiguity_count": sum(
            row["ambiguity_recorded"] for row in branches
        ),
        "free_effort_scalars": 0,
        "ordered_rounding_sha256": hashlib.sha256(canonical_json(rounding)).hexdigest(),
        "ordered_branch_sha256": hashlib.sha256(canonical_json(branches)).hexdigest(),
        "rounding_cases": rounding,
        "branch_cases": branches,
    }


def base_cone_half_space_rows(count: int = 32) -> list[dict[str, Any]]:
    if type(count) is not int or count != 32:
        raise ValueError("R140 base cone half-space count differs")
    rows = []
    for ordinal in range(count):
        angle = math.tau * ordinal / count
        rows.append(
            {
                "ordinal": ordinal,
                "angle_turn_numerator": ordinal,
                "angle_turn_denominator": count,
                "right_coefficient_float64_hex": math.cos(angle).hex(),
                "forward_coefficient_float64_hex": math.sin(angle).hex(),
            }
        )
    return rows


def classify_circular_cone(
    normal: int,
    right: int,
    forward: int,
    *,
    friction_numerator: int = 52429,
    friction_denominator: int = 65536,
) -> dict[str, Any]:
    if any(type(value) is not int for value in (normal, right, forward)):
        raise ValueError("R140 circular cone input differs")
    if normal < 0:
        return {"inside": False, "normal_nonnegative": False}
    tangential_scaled_squared = (
        right * right + forward * forward
    ) * friction_denominator**2
    capacity_scaled_squared = (friction_numerator * normal) ** 2
    return {
        "inside": tangential_scaled_squared <= capacity_scaled_squared,
        "normal_nonnegative": True,
    }


def build_separation_directions(
    violations: Sequence[Mapping[str, Any]],
) -> list[dict[str, Any]]:
    cone_type_order = {"continuous_force": 0, "activation_impulse": 1}
    owned: dict[tuple[int, int, int], dict[str, Any]] = {}
    for row in violations:
        graph_row = row.get("graph_row")
        cone_type = row.get("cone_type")
        point_ordinal = row.get("point_ordinal")
        right = row.get("right")
        forward = row.get("forward")
        if (
            type(graph_row) is not int
            or graph_row < 0
            or cone_type not in cone_type_order
            or type(point_ordinal) is not int
            or point_ordinal < 0
            or type(right) is not int
            or type(forward) is not int
        ):
            raise ValueError("R140 separation violation input differs")
        squared = right * right + forward * forward
        norm = math.isqrt(squared)
        if norm == 0 or norm * norm != squared:
            raise ValueError("R140 separation direction normalization differs")
        key = (graph_row, cone_type_order[cone_type], point_ordinal)
        direction = {
            "graph_row": graph_row,
            "cone_type": cone_type,
            "point_ordinal": point_ordinal,
            "right_numerator": right,
            "forward_numerator": forward,
            "normalizer": norm,
        }
        if key in owned and owned[key] != direction:
            raise ValueError("R140 multiple separation directions for one cone")
        owned[key] = direction
    return [owned[key] for key in sorted(owned)]


def audit_friction_cone_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    contract = profile["friction_cone_conformance_contract"]
    base_rows = base_cone_half_space_rows()
    base_hash = hashlib.sha256(canonical_json(base_rows)).hexdigest()
    if base_hash != contract["base_half_space_rows_sha256"]:
        raise ValueError("R140 base cone row identity differs")
    classifications = []
    for row in contract["classification_cases"]:
        actual = {
            "case": row["case"],
            **classify_circular_cone(row["normal"], row["right"], row["forward"]),
        }
        expected = {
            key: value
            for key, value in row.items()
            if key not in {"normal", "right", "forward"}
        }
        if actual != expected:
            raise ValueError("R140 exact circular cone classification differs")
        classifications.append(actual)
    separation = build_separation_directions(contract["violated_cones"])
    if separation != contract["expected_separation_directions"]:
        raise ValueError("R140 deterministic cone separation differs")
    return {
        "status": "PASS",
        "base_half_space_rows": len(base_rows),
        "base_half_space_rows_sha256": base_hash,
        "classification_case_count": len(classifications),
        "separation_case_count": len(separation),
        "exact_circular_cone_acceptance_authority": True,
        "polyhedral_acceptance_authority": False,
        "classification_cases": classifications,
        "separation_directions": separation,
    }


def classify_contact_rank(singular_values: Sequence[float]) -> int:
    rank = 0
    for value in singular_values:
        if not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
            raise ValueError("R140 non-finite or negative singular value")
        if value <= NULL_SINGULAR_VALUE_MAXIMUM:
            continue
        if value < RETAINED_SINGULAR_VALUE_MINIMUM:
            raise ValueError("R140 singular value lies inside rank gap")
        rank += 1
    if rank not in EXPECTED_CONTACT_RANKS:
        raise ValueError("R140 unexpected contact rank")
    return rank


def validate_numeric_coefficients(values: Sequence[float]) -> list[float]:
    result = []
    for value in values:
        if not isinstance(value, (int, float)) or not math.isfinite(value):
            raise ValueError("R140 non-finite numeric coefficient")
        result.append(
            0.0 if abs(value) <= COEFFICIENT_ELISION_MAXIMUM else float(value)
        )
    return result


def validate_bounds(rows: Sequence[Sequence[float]]) -> None:
    for row in rows:
        if len(row) != 2:
            raise ValueError("R140 bound row width differs")
        lower, upper = row
        if (
            not isinstance(lower, (int, float))
            or not isinstance(upper, (int, float))
            or not math.isfinite(lower)
            or not math.isfinite(upper)
        ):
            raise ValueError("R140 non-finite bound")
        if lower > upper:
            raise ValueError("R140 unordered bound")


def require_cone_classification_agreement(
    *, norm_classification: bool, squared_classification: bool
) -> None:
    if (
        type(norm_classification) is not bool
        or type(squared_classification) is not bool
    ):
        raise ValueError("R140 cone classification type differs")
    if norm_classification != squared_classification:
        raise ValueError("R140 cone classification ambiguity")


def validate_solver_candidate(
    *, status: str, primal_residual: float, dual_residual: float
) -> None:
    if status != "solved":
        raise ValueError("R140 solver status ambiguity")
    if (
        not math.isfinite(primal_residual)
        or not math.isfinite(dual_residual)
        or primal_residual < 0
        or dual_residual < 0
        or primal_residual > SOLVER_RESIDUAL_MAXIMUM
        or dual_residual > SOLVER_RESIDUAL_MAXIMUM
    ):
        raise ValueError("R140 solver residual ambiguity")


def audit_numeric_invalid_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    contract = profile["numeric_invalid_conformance_contract"]
    valid = [
        {"case": "rank_0", "result": classify_contact_rank([0.0, 1e-13])},
        {"case": "rank_3", "result": classify_contact_rank([1e-4, 1e-5, 1e-6])},
        {
            "case": "rank_5",
            "result": classify_contact_rank([1e-2, 1e-3, 1e-4, 1e-5, 1e-6]),
        },
    ]
    validate_bounds([(-1.0, 1.0), (0.0, 0.0)])
    valid.append({"case": "ordered_bounds", "result": "PASS"})
    validate_solver_candidate(status="solved", primal_residual=4e-5, dual_residual=5e-5)
    valid.append({"case": "solver_candidate", "result": "PASS"})
    coefficient_result = validate_numeric_coefficients([1e-15, -1e-14, 2e-14])
    if coefficient_result != [0.0, 0.0, 2e-14]:
        raise ValueError("R140 coefficient elision differs")

    rejected = []
    invalid_calls = (
        (
            "rank_gap",
            lambda: classify_contact_rank([1e-11]),
            "R140 singular value lies inside rank gap",
        ),
        (
            "unexpected_rank",
            lambda: classify_contact_rank([1.0] * 4),
            "R140 unexpected contact rank",
        ),
        (
            "nonfinite_singular",
            lambda: classify_contact_rank([math.nan]),
            "R140 non-finite or negative singular value",
        ),
        (
            "nonfinite_coefficient",
            lambda: validate_numeric_coefficients([math.inf]),
            "R140 non-finite numeric coefficient",
        ),
        (
            "unordered_bound",
            lambda: validate_bounds([(1.0, -1.0)]),
            "R140 unordered bound",
        ),
        (
            "cone_ambiguity",
            lambda: require_cone_classification_agreement(
                norm_classification=True, squared_classification=False
            ),
            "R140 cone classification ambiguity",
        ),
        (
            "solver_status",
            lambda: validate_solver_candidate(
                status="solved inaccurate", primal_residual=0.0, dual_residual=0.0
            ),
            "R140 solver status ambiguity",
        ),
        (
            "solver_residual",
            lambda: validate_solver_candidate(
                status="solved", primal_residual=5.1e-5, dual_residual=0.0
            ),
            "R140 solver residual ambiguity",
        ),
    )
    for case, call, expected_error in invalid_calls:
        try:
            call()
        except ValueError as error:
            if str(error) != expected_error:
                raise ValueError("R140 numeric invalid error differs") from error
            rejected.append({"case": case, "error": str(error)})
        else:
            raise ValueError("R140 numeric invalid case was accepted")
    if [row["case"] for row in valid] != contract["valid_cases"] or [
        row["case"] for row in rejected
    ] != contract["invalid_cases"]:
        raise ValueError("R140 numeric case inventory differs")
    return {
        "status": "PASS",
        "valid_case_count": len(valid),
        "invalid_case_count": len(rejected),
        "coefficient_elision_case_count": 1,
        "valid_cases": valid,
        "invalid_cases": rejected,
    }


def audit_budget_and_hash_closure(
    profile: Mapping[str, Any], *, r139: Mapping[str, Any]
) -> dict[str, Any]:
    contract = profile["budget_and_hash_conformance_contract"]
    budget = r139.get("future_execution_budget_contract", {})
    required = r139.get("required_hash_contract", {})
    if (
        budget != contract["future_execution_budget"]
        or budget.get("major_iteration_limit")
        * budget.get("lexicographic_qp_passes_per_major_iteration")
        != budget.get("qp_solve_limit")
        or budget.get("major_iteration_limit")
        * budget.get("trial_fractions_per_major_iteration")
        != budget.get("exact_trial_audit_limit")
        or set(required.get("future_execution_output", ())) != REQUIRED_OUTPUT_HASH_KEYS
        or set(required.get("required_counter_closure", ())) != set(ZERO_REAL_COUNTERS)
        or required.get("before_first_real_graph_call")
        != profile["reconstruction_gate_contract"]["required_source_hashes"]
    ):
        raise ValueError("R140 budget or hash closure differs")
    return {
        "status": "PASS",
        "major_iteration_limit": budget["major_iteration_limit"],
        "qp_solve_limit": budget["qp_solve_limit"],
        "exact_trial_audit_limit": budget["exact_trial_audit_limit"],
        "osqp_iteration_limit_per_qp": budget["osqp_iteration_limit_per_qp"],
        "wall_time_seconds": budget["wall_time_seconds"],
        "peak_rss_bytes": budget["peak_rss_bytes"],
        "pre_graph_hash_count": len(required["before_first_real_graph_call"]),
        "accepted_anchor_hash_count": len(required["every_accepted_anchor"]),
        "output_hash_count": len(required["future_execution_output"]),
        "counter_closure_count": len(required["required_counter_closure"]),
    }


def audit_r139_source(
    profile: Mapping[str, Any], *, r139: Mapping[str, Any]
) -> dict[str, Any]:
    source = profile["source"]["r139"]
    ids = r139.get("identities", {})
    solve = r139.get("solve_formulation_audit", {})
    budget = r139.get("numeric_and_budget_audit", {})
    if (
        r139.get("status") != "COMPLETE"
        or r139.get("claim") != "FixedModeKinodynamicSolveFormulationOnly"
        or r139.get("repository", {}).get("commit") != source["repository_commit"]
        or r139.get("repository", {}).get("dirty") is not False
        or r139.get("repository", {}).get("dirty_paths") != []
        or r139.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R140_KINODYNAMIC_SOLVE_IMPLEMENTATION_CONFORMANCE_ONLY"
        or r139.get("result_transition") != "R139_COMPLETE_R140_CONFORMANCE_ONLY"
        or r139.get("source_report_audits") != 3
        or r139.get("source_array_payloads_read") != 0
        or solve.get("status") != "PASS"
        or solve.get("algorithm") != "FIXED_MODE_SPARSE_MULTIPLE_SHOOTING_SCVX_SQP"
        or solve.get("lexicographic_qp_passes_per_major_iteration") != 3
        or solve.get("trial_fractions_per_major_iteration") != 7
        or solve.get("exact_controller_is_sole_acceptance_authority") is not True
        or solve.get("exact_circular_cone_is_sole_acceptance_authority") is not True
        or budget.get("status") != "PASS"
        or budget.get("qp_solve_limit") != 18
        or budget.get("exact_trial_audit_limit") != 42
        or r139.get("bounded_acceptance", {}).get(
            "r140_kinodynamic_solve_implementation_conformance"
        )
        != "AUTHORIZED_REPORT_ONLY_ON_R139_COMPLETE"
        or any(r139.get(counter) != 0 for counter in ZERO_REAL_COUNTERS)
        or ids.get("profile_sha256") != source["profile_sha256"]
        or ids.get("formulation_module_sha256") != source["module_sha256"]
        or ids.get("tool_sha256") != source["tool_sha256"]
    ):
        raise ValueError("R140 R139 source contract differs")
    return {
        "status": "PASS",
        "r139_report_sha256": r139["report_sha256"],
        "r139_transition": r139["result_transition"],
        "source_reports_read_by_r139": r139["source_report_audits"],
        "source_array_payloads_read_by_r139": r139["source_array_payloads_read"],
        "algorithm": solve["algorithm"],
        "qp_solve_limit_future_only": budget["qp_solve_limit"],
        "exact_trial_audit_limit_future_only": budget["exact_trial_audit_limit"],
    }


def _load_bound_report(path: Path, expected: Mapping[str, Any]) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError("R140 R139 report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError("R140 R139 canonical report identity differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    authority_document_path: Path,
    r139_profile_path: Path,
    r139_module_path: Path,
    r139_tool_path: Path,
) -> None:
    source = profile["source"]
    expected = (
        (authority_document_path, source["authority_document_sha256"]),
        (r139_profile_path, source["r139"]["profile_sha256"]),
        (r139_module_path, source["r139"]["module_sha256"]),
        (r139_tool_path, source["r139"]["tool_sha256"]),
    )
    if any(sha256(path) != digest for path, digest in expected):
        raise ValueError("R140 source file identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    future = profile.get("future_r141_execution_contract", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "FixedModeKinodynamicSolveImplementationConformanceOnly"
        or source.get("authority_document_sha256")
        != "0e2c12115383fe85a293fce93550a2639ba84cee718a1f947fdea0e7ac30902f"
        or source.get("authority_repository_commit")
        != "8383e5edd787b60ebd3b171985392244085fee02"
        or source.get("r139", {}).get("report_sha256")
        != "fc76b3072ca9323a2b5dfea45bb9f5c74de57e485a80ed01a7a8ecd6651fec0c"
        or source.get("r139", {}).get("report_file_sha256")
        != "efd6968fac2a66144b43c8422d798b025a878d0e5588e85d8a3414b745b465de"
        or source.get("r139", {}).get("profile_sha256")
        != "55326991ba7fc025238a2a6ef3f288984c1dea14533cf8b50f9e6ad897f57853"
        or source.get("r139", {}).get("module_sha256")
        != "cc84ee8cbc36c7e4591746963e266cdb76969276d835694fc9b3614f4f84a25f"
        or source.get("r139", {}).get("tool_sha256")
        != "c78bed4d6cbfd5e6a24d7cc618a1a300197216d80be15197c5f5bcb33f3f67e2"
        or source.get("r139", {}).get("repository_commit")
        != "0a71362e6d1e77378bbf53662f3ccf6cec223f77"
        or scope.get("conformance_id") != "R140"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("source_reports_read") != 1
        or scope.get("source_array_payloads_read") != 0
        or scope.get("synthetic_reconstruction_gate_cases") != 3
        or scope.get("synthetic_qp_schema_cases") != 3
        or scope.get("synthetic_trust_cases") != 7
        or scope.get("synthetic_rounding_cases") != 6
        or scope.get("synthetic_branch_cases") != 5
        or scope.get("synthetic_cone_base_rows") != 32
        or scope.get("synthetic_cone_classification_cases") != 3
        or scope.get("synthetic_cone_separation_cases") != 3
        or scope.get("synthetic_numeric_valid_cases") != 5
        or scope.get("synthetic_numeric_invalid_cases") != 8
        or any(scope.get(counter) != 0 for counter in ZERO_REAL_COUNTERS)
        or set(
            profile.get("reconstruction_gate_contract", {}).get(
                "required_source_hashes", {}
            )
        )
        != REQUIRED_PRE_GRAPH_HASH_KEYS
        or len(
            profile.get("lexicographic_qp_schema_contract", {}).get(
                "expected_passes", ()
            )
        )
        != 3
        or len(profile.get("trust_funnel_conformance_contract", {}).get("cases", ()))
        != 7
        or len(
            profile.get("controller_boundary_conformance_contract", {}).get(
                "rounding_cases", ()
            )
        )
        != 6
        or len(
            profile.get("controller_boundary_conformance_contract", {}).get(
                "branch_cases", ()
            )
        )
        != 5
        or len(
            profile.get("friction_cone_conformance_contract", {}).get(
                "classification_cases", ()
            )
        )
        != 3
        or len(
            profile.get("numeric_invalid_conformance_contract", {}).get(
                "invalid_cases", ()
            )
        )
        != 8
        or future.get("authority") != "NOT_GRANTED_BY_R140"
        or future.get("required_preconditions")
        != ["EXACT_R140_PASS", "EXPLICIT_LATER_ROADMAP_UPDATE"]
        or future.get("single_bounded_execution") != "R141_ONLY"
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r141_bounded_kinodynamic_solve_roadmap_decision")
        != "AUTHORIZED_DECISION_ONLY_ON_R140_PASS"
        or any(
            not str(value).startswith("NOT_AUTHORIZED")
            for key, value in bounded.items()
            if key != "r141_bounded_kinodynamic_solve_roadmap_decision"
        )
        or profile.get("decision", {}).get("pass")
        != "PERMIT_EXPLICIT_ROADMAP_DECISION_FOR_ONE_BOUNDED_R141_ONLY"
        or profile.get("result_transitions", {}).get("pass")
        != "R140_PASS_R141_ROADMAP_DECISION_ONLY"
        or profile.get("result_transitions", {}).get("invalid")
        != "R140_INVALID_STOP_WITHOUT_EXECUTION"
    ):
        raise ValueError("R140 conformance profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R140 conformance requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R140 validation result differs")
    return [dict(row) for row in results]


def _decimal_string(value: Decimal) -> str:
    if not value.is_finite():
        raise ValueError("R140 non-finite decimal")
    return format(value.normalize(), "f")


def _linux_thread_count() -> int:
    task_directory = Path(f"/proc/{os.getpid()}/task")
    if not task_directory.is_dir():
        raise RuntimeError("R140 requires Linux /proc thread accounting")
    return sum(1 for child in task_directory.iterdir() if child.name.isdigit())
