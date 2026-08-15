from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

FORMULATION_ID = "nextengine.humanoid-fixed-mode-kinodynamic-solve-formulation.v1"
CHECK_ID = "TRAIN-4-FIXED-MODE-KINODYNAMIC-SOLVE-FORMULATION"
MOTOR_INTERVAL_COUNT = 800
PHYSICS_INTERVAL_COUNT = 3200
STATE_NODE_COUNT = 3201
MAJOR_ITERATION_LIMIT = 6
QP_PASSES_PER_MAJOR_ITERATION = 3
TRIAL_FRACTIONS_PER_MAJOR_ITERATION = 7
QP_SOLVE_LIMIT = MAJOR_ITERATION_LIMIT * QP_PASSES_PER_MAJOR_ITERATION
EXACT_TRIAL_AUDIT_LIMIT = MAJOR_ITERATION_LIMIT * TRIAL_FRACTIONS_PER_MAJOR_ITERATION
ZERO_EXECUTION_COUNTERS = (
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
        "r140_kinodynamic_solve_implementation_conformance",
        "r141_bounded_kinodynamic_solve",
        "r136_retry",
        "r137_retry",
        "r138_retry",
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
EXPECTED_R120_ARRAY_HASHES = {
    "joint_position_urad": (
        "5d02a91bd401df0402920426b9ae728d6de7c80544a65c113e7cf28fa5c4befb"
    ),
    "joint_velocity_urad_s": (
        "d2cae9f31ccdb36b313de918c4c0565eec2ed8f39da3b5a79d0ae2afcd3aca34"
    ),
    "root_linear_velocity_um_s": (
        "3c898243cbc227c56579c2c1a96905ddebdf36b58ece6ff1d578c53bd9492268"
    ),
    "root_position_um": (
        "9e3d3cb27d8aa872b1e8c6daa4ecb2420713aef4fba35d654c6153e1b04a71f8"
    ),
    "root_quaternion_q1_30": (
        "9b8a83dd33774d0e759237ec7b51997e7404b32d08ee0538ca0bba4abcafb92d"
    ),
    "root_yaw_velocity_urad_s": (
        "656f9d52fa7ef6adac9dd4b337c280a7b53e0bed7a415c254be6d8251b44b0f6"
    ),
}
EXPECTED_TRACKING_WEIGHTS = {
    "configuration": 1,
    "generalized_velocity": 0.25,
    "integer_command_target": 0.5,
}
EXPECTED_REGULARIZATION_WEIGHTS = {
    "acceleration_variation": 0.01,
    "force_magnitude_and_variation": 0.0001,
    "impulse_magnitude": 0.0001,
}
EXPECTED_PHYSICAL_SCALES = {
    "configuration_root_translation_metres": 0.01,
    "configuration_root_rotation_radians": 0.025,
    "configuration_joint_radians": 0.025,
    "velocity_root_linear_metres_per_second": 0.25,
    "velocity_root_angular_radians_per_second": 1,
    "velocity_joint_radians_per_second": 2,
    "acceleration_root_linear_metres_per_second_squared": 5,
    "acceleration_root_angular_radians_per_second_squared": 20,
    "acceleration_joint_radians_per_second_squared": 20,
    "integer_command_microradians": 25000,
    "active_force_newtons": 250,
    "activation_impulse_newton_seconds": 25,
}
EXPECTED_NUMERIC_INVALID_CONDITIONS = {
    "non-finite coefficient residual or iterate",
    "unordered lower and upper bound",
    "unexpected contact rank or additional nullity",
    "singular value strictly inside the rank gap",
    "norm versus squared-cone classification disagreement",
    "solver status or residual ambiguity",
    "counter budget or hash closure mismatch",
}


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_fixed_mode_kinodynamic_solve_formulation(
    *,
    profile_path: Path,
    decision_document_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r133_report_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r138_report_path: Path,
    r138_profile_path: Path,
    r138_module_path: Path,
    r138_tool_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R139 without reading source arrays or evaluating the real graph."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            decision_document_path,
            r120_report_path,
            r120_profile_path,
            r133_report_path,
            r133_profile_path,
            r133_module_path,
            r133_tool_path,
            r138_report_path,
            r138_profile_path,
            r138_module_path,
            r138_tool_path,
            tool_path,
        )
    )
    (
        profile_path,
        decision_document_path,
        r120_report_path,
        r120_profile_path,
        r133_report_path,
        r133_profile_path,
        r133_module_path,
        r133_tool_path,
        r138_report_path,
        r138_profile_path,
        r138_module_path,
        r138_tool_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R139 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    source = profile["source"]
    r120 = _load_bound_report(r120_report_path, source["r120"], "R120")
    r133 = _load_bound_report(r133_report_path, source["r133"], "R133")
    r138 = _load_bound_report(r138_report_path, source["r138"], "R138")
    _validate_source_files(
        profile=profile,
        decision_document_path=decision_document_path,
        r120_profile_path=r120_profile_path,
        r133_profile_path=r133_profile_path,
        r133_module_path=r133_module_path,
        r133_tool_path=r133_tool_path,
        r138_profile_path=r138_profile_path,
        r138_module_path=r138_module_path,
        r138_tool_path=r138_tool_path,
    )
    if (
        sha256(Path(__file__).resolve()) != source["formulation_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R139 current formulation identity differs")

    lineage = audit_source_lineage(profile, r120=r120, r133=r133, r138=r138)
    formulation = audit_solve_formulation(profile)
    numeric_budget = audit_numeric_and_budget_contract(profile)
    required_hashes = audit_required_hash_contract(profile)
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
        "solve_formulation_audit": formulation,
        "numeric_and_budget_audit": numeric_budget,
        "required_hash_audit": required_hashes,
        "source_reconstruction_contract": profile["source_reconstruction_contract"],
        "initial_and_endpoint_contract": profile["initial_and_endpoint_contract"],
        "objective_hierarchy_contract": profile["objective_hierarchy_contract"],
        "normalization_and_trust_region_contract": profile[
            "normalization_and_trust_region_contract"
        ],
        "candidate_generator_contract": profile["candidate_generator_contract"],
        "exact_acceptance_contract": profile["exact_acceptance_contract"],
        "numeric_validity_contract": profile["numeric_validity_contract"],
        "future_execution_budget_contract": profile["future_execution_budget_contract"],
        "required_hash_contract": profile["required_hash_contract"],
        "r140_implementation_conformance_contract": profile[
            "r140_implementation_conformance_contract"
        ],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "decision_document_sha256": sha256(decision_document_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_profile_sha256": sha256(r120_profile_path),
            "r133_report_file_sha256": sha256(r133_report_path),
            "r133_profile_sha256": sha256(r133_profile_path),
            "r133_module_sha256": sha256(r133_module_path),
            "r133_tool_sha256": sha256(r133_tool_path),
            "r138_report_file_sha256": sha256(r138_report_path),
            "r138_profile_sha256": sha256(r138_profile_path),
            "r138_module_sha256": sha256(r138_module_path),
            "r138_tool_sha256": sha256(r138_tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "source_report_audits": 3,
        "source_array_payloads_read": 0,
        "solve_formulations": 1,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_source_lineage(
    profile: Mapping[str, Any],
    *,
    r120: Mapping[str, Any],
    r133: Mapping[str, Any],
    r138: Mapping[str, Any],
) -> dict[str, Any]:
    source = profile["source"]
    r120_solver = r120.get("solver_result", {})
    r120_accepted = r120_solver.get("accepted_exact_result", {})
    r120_hashes = r120_accepted.get("emitted_hashes", {})
    r120_arrays = r120_hashes.get("arrays", {})
    r120_cache = r120.get("solver_private_warm_start_cache", {})
    if (
        r120.get("status") != "PASS"
        or r120.get("repository", {}).get("commit")
        != source["r120"]["repository_commit"]
        or r120.get("repository", {}).get("dirty") is not False
        or r120.get("repository", {}).get("dirty_paths") != []
        or r120.get("scope", {}).get("run_id") != "R120"
        or r120.get("scope", {}).get("clip_id") != "cmu16-walk-nominal-b"
        or r120.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R121_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_FORMULATION_ONLY"
        or r120_solver.get("status") != "PASS"
        or r120_accepted.get("status") != "PASS"
        or r120_hashes.get("aggregate_sha256")
        != source["r120"]["accepted_emitted_aggregate_sha256"]
        or r120_cache.get("status") != "EMITTED_TRANSIENT_SOLVER_PRIVATE"
        or r120_cache.get("sha256") != source["r120"]["cache_sha256"]
        or r120.get("identities", {}).get("execution_profile_sha256")
        != source["r120"]["profile_sha256"]
        or any(
            r120_arrays.get(name, {}).get("sha256") != digest
            for name, digest in EXPECTED_R120_ARRAY_HASHES.items()
        )
        or any(
            r120.get(counter) != 0
            for counter in (
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "optimizer_steps",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "training_runs",
            )
        )
    ):
        raise ValueError("R139 R120 source contract differs")

    schedule = r133.get("projected_fixed_pd_schedule_audit", {})
    r133_ids = r133.get("identities", {})
    if (
        r133.get("status") != "PASS"
        or r133.get("invalid_reason") is not None
        or r133.get("repository", {}).get("commit")
        != source["r133"]["repository_commit"]
        or r133.get("repository", {}).get("dirty") is not False
        or r133.get("repository", {}).get("dirty_paths") != []
        or r133.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R134_PROJECTED_INVERSE_DYNAMICS_FORMULATION_ONLY"
        or r133.get("result_transition") != "R133_PASS_R134_FORMULATION_ONLY"
        or schedule.get("status") != "PASS"
        or schedule.get("collocation_count") != PHYSICS_INTERVAL_COUNT
        or schedule.get("projected_velocity_float64_sha256")
        != source["r133"]["projected_velocity_float64_sha256"]
        or schedule.get("applied_target_float64_sha256")
        != source["r133"]["applied_target_float64_sha256"]
        or schedule.get("applied_effort_float64_sha256")
        != source["r133"]["applied_effort_float64_sha256"]
        or r133_ids.get("profile_sha256") != source["r133"]["profile_sha256"]
        or r133_ids.get("execution_module_sha256") != source["r133"]["module_sha256"]
        or r133_ids.get("tool_sha256") != source["r133"]["tool_sha256"]
        or r133_ids.get("r120_cache_sha256") != source["r120"]["cache_sha256"]
        or r133.get("source_gates", {}).get("r120_report_sha256")
        != source["r120"]["report_sha256"]
        or any(
            r133.get(counter) != 0
            for counter in (
                "inverse_dynamics_system_assemblies",
                "kinodynamic_solves",
                "optimizer_steps",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "training_runs",
            )
        )
    ):
        raise ValueError("R139 R133 source contract differs")

    r138_ids = r138.get("identities", {})
    index = r138.get("index_map_audit", {})
    transitions = r138.get("transition_replay_audit", {})
    if (
        r138.get("status") != "PASS"
        or r138.get("repository", {}).get("commit")
        != source["r138"]["repository_commit"]
        or r138.get("repository", {}).get("dirty") is not False
        or r138.get("repository", {}).get("dirty_paths") != []
        or r138.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R139_KINODYNAMIC_SOLVE_FORMULATION_ONLY"
        or r138.get("result_transition") != "R138_PASS_R139_FORMULATION_ONLY"
        or index.get("status") != "PASS"
        or index.get("motor_intervals") != MOTOR_INTERVAL_COUNT
        or index.get("physics_intervals") != PHYSICS_INTERVAL_COUNT
        or index.get("state_nodes") != STATE_NODE_COUNT
        or index.get("ordered_address_sha256") != source["r138"]["graph_index_sha256"]
        or transitions.get("status") != "PASS"
        or transitions.get("changed_motor_boundaries") != 29
        or transitions.get("activation_points") != 18
        or transitions.get("deactivation_points") != 18
        or transitions.get("ordered_transition_replay_sha256")
        != source["r138"]["transition_replay_sha256"]
        or r138_ids.get("profile_sha256") != source["r138"]["profile_sha256"]
        or r138_ids.get("conformance_module_sha256") != source["r138"]["module_sha256"]
        or r138_ids.get("tool_sha256") != source["r138"]["tool_sha256"]
        or any(
            r138.get(counter) != 0
            for counter in ZERO_EXECUTION_COUNTERS
            if counter in r138
        )
    ):
        raise ValueError("R139 R138 source contract differs")

    return {
        "status": "PASS",
        "source_reports_read": 3,
        "source_array_payloads_read": 0,
        "r120_report_sha256": r120["report_sha256"],
        "r120_cache_sha256_from_report_metadata": r120_cache["sha256"],
        "r120_accepted_emitted_aggregate_sha256": r120_hashes["aggregate_sha256"],
        "r120_initial_array_hashes": {
            name: r120_arrays[name]["sha256"] for name in EXPECTED_R120_ARRAY_HASHES
        },
        "r133_report_sha256": r133["report_sha256"],
        "r133_projected_velocity_float64_sha256": schedule[
            "projected_velocity_float64_sha256"
        ],
        "r133_applied_target_float64_sha256": schedule["applied_target_float64_sha256"],
        "r133_applied_effort_float64_sha256": schedule["applied_effort_float64_sha256"],
        "r131_motor_mode_sequence_sha256": source["r131_motor_mode_sequence_sha256"],
        "r138_report_sha256": r138["report_sha256"],
        "r138_graph_index_sha256": index["ordered_address_sha256"],
        "r138_transition_replay_sha256": transitions[
            "ordered_transition_replay_sha256"
        ],
    }


def audit_solve_formulation(profile: Mapping[str, Any]) -> dict[str, Any]:
    source = profile["source_reconstruction_contract"]
    endpoint = profile["initial_and_endpoint_contract"]
    objective = profile["objective_hierarchy_contract"]
    trust = profile["normalization_and_trust_region_contract"]
    candidate = profile["candidate_generator_contract"]
    acceptance = profile["exact_acceptance_contract"]
    passes = objective["lexicographic_qp_passes"]
    surrogate = candidate.get("controller_surrogate", {})
    cone = candidate.get("friction_cone_model", {})
    if (
        source.get("r139_source_report_reads") != 3
        or source.get("r139_source_array_payload_reads") != 0
        or source.get("r120_cache_access_in_r139") != "FORBIDDEN_REPORT_METADATA_ONLY"
        or source.get("r136_cache_acceleration_or_force_import") != "FORBIDDEN"
        or source.get("r136_pointwise_witness_import") != "FORBIDDEN"
        or source.get("future_before_first_graph_call")
        != "ALL_REQUIRED_HASHES_MUST_REPRODUCE_EXACTLY"
        or source.get("source_identity_failure") != "INVALID_STOP_WITHOUT_RESTART"
        or endpoint.get("initial_configuration", {}).get("source")
        != "R120_ACCEPTED_EXACT_ROW_0"
        or endpoint.get("initial_configuration", {}).get("immutable") is not True
        or endpoint.get("initial_velocity", {}).get("source")
        != "R133_PROJECTED_VELOCITY_ROW_0"
        or endpoint.get("initial_velocity", {}).get("immutable") is not True
        or endpoint.get("terminal", {}).get("byte_fixed") is not False
        or endpoint.get("terminal", {}).get("periodic") is not False
        or endpoint.get("terminal", {}).get("all_hard_constraints_required") is not True
        or endpoint.get("terminal", {}).get("tracking_weight_multiplier") != 16
        or endpoint.get("interior_r120_r133_role")
        != "INITIALIZATION_AND_DEVIATION_REFERENCE_ONLY_NOT_HARD_EQUALITY"
        or endpoint.get("acceleration_initialization")
        != "DERIVE_ONLY_FROM_RECONSTRUCTED_Q_V_REFERENCE_AND_FROZEN_DISCRETE_MAP"
        or endpoint.get("continuous_force_initialization_newtons") != 0
        or endpoint.get("activation_impulse_initialization_newton_seconds") != 0
        or endpoint.get("pointwise_inverse_dynamics_initialization") != "FORBIDDEN"
        or objective.get("hard_feasibility_priority")
        != "LEXICOGRAPHICALLY_ABOVE_ALL_TRACKING_AND_REGULARIZATION"
        or [row.get("pass") for row in passes] != [1, 2, 3]
        or [row.get("objective") for row in passes]
        != [
            "MINIMIZE_MAXIMUM_NORMALIZED_ELASTIC_VIOLATION",
            "MINIMIZE_MEAN_NORMALIZED_ELASTIC_VIOLATION",
            "MINIMIZE_TRACKING_AND_REGULARIZATION",
        ]
        or passes[1].get("maximum_optimum_hold_tolerance") != 1e-8
        or passes[2].get("maximum_optimum_hold_tolerance") != 1e-8
        or passes[2].get("mean_optimum_hold_tolerance") != 1e-8
        or objective.get("temporary_elastics_in_emitted_state") != "FORBIDDEN"
        or objective.get("mean_squared_tracking_weights") != EXPECTED_TRACKING_WEIGHTS
        or objective.get("terminal_q_v_tracking_multiplier") != 16
        or objective.get("regularization_weights") != EXPECTED_REGULARIZATION_WEIGHTS
        or objective.get("first_exact_hard_constraint_pass") != "TERMINATE"
        or objective.get("post_feasibility_quality_polishing") != "FORBIDDEN"
        or trust.get("componentwise_physical_scales") != EXPECTED_PHYSICAL_SCALES
        or trust.get("trial_fractions")
        != ["1", "1/2", "1/4", "1/8", "1/16", "1/32", "1/64"]
        or trust.get("exact_pass_priority") != "ACCEPT_AND_TERMINATE"
        or trust.get("exact_funnel_order")
        != ["maximum_normalized_violation", "mean_normalized_violation"]
        or trust.get("exact_funnel_minimum_lexicographic_decrease") != 1e-6
        or trust.get("expansion_requires_active_trust_boundary") is not True
        or trust.get("no_accepted_trial")
        != "CONTRACT_AND_RESTORE_PREVIOUS_EXACT_ANCHOR"
        or trust.get("failure_at_minimum_radius")
        != "VALID_STOP_AND_RESEARCH_WITHOUT_TUNING_OR_RESTART"
        or candidate.get("algorithm") != "FIXED_MODE_SPARSE_MULTIPLE_SHOOTING_SCVX_SQP"
        or candidate.get("qp_solver") != "OSQP"
        or candidate.get("qp_form") != "SPARSE_L_LE_A_X_LE_U"
        or candidate.get("authority") != "CANDIDATE_GENERATION_ONLY"
        or candidate.get("contact_mode_choice") != "FORBIDDEN"
        or surrogate.get("type") != "CONTINUOUS_FIXED_BRANCH_LOCAL_MODEL"
        or surrogate.get("integer_rounding") != "REMOVED_FOR_LOCAL_MODEL_ONLY"
        or surrogate.get("branch_equality_derivative") != 0
        or surrogate.get("branch_equality_ambiguity_count") != "REQUIRED"
        or surrogate.get("branch_address_hash") != "REQUIRED"
        or surrogate.get("free_effort") != "FORBIDDEN"
        or surrogate.get("acceptance_authority") != "NONE"
        or cone.get("base_angular_half_spaces") != 32
        or cone.get("separation_update")
        != "AT_MOST_ONE_DETERMINISTIC_DIRECTION_PER_EXACTLY_VIOLATED_CONE_PER_ACCEPTED_ANCHOR"
        or cone.get("separation_direction_order")
        != "GRAPH_ROW_THEN_CONE_TYPE_THEN_POINT_ORDINAL"
        or cone.get("polyhedral_elastic_authority") != "NONE"
        or cone.get("acceptance_authority") != "EXACT_CIRCULAR_CONE_ONLY"
        or acceptance.get("oracle") != "EXACT_NONLINEAR_FIXED_MODE_GRAPH_ONLY"
        or acceptance.get("controller_replay")
        != "EXACT_R133_ORDER_TIES_TO_EVEN_FROM_TRIAL_Q_V_AND_INTEGER_COMMAND"
        or acceptance.get("surrogate_effort_authority") != "NONE_DISCARDED"
        or acceptance.get("changed_float_command_rounding_to_same_integer")
        != "NO_COMMAND_PROGRESS"
        or acceptance.get("polyhedral_cone_authority") != "NONE"
        or acceptance.get("force_and_impulse_cone")
        != "EXACT_INDIVIDUAL_CIRCULAR_COULOMB_CONES"
        or acceptance.get("funnel")
        != "LEXICOGRAPHIC_MAXIMUM_THEN_MEAN_NORMALIZED_EXACT_VIOLATION"
        or acceptance.get("qp_slack_or_status_as_feasibility") != "FORBIDDEN"
        or acceptance.get("first_exact_pass") != "TERMINATE_WITHOUT_POLISHING"
        or acceptance.get("accepted_anchor_required_hashes")
        != "ALL_REQUIRED_EVERY_ACCEPTED_ANCHOR_HASHES"
        or acceptance.get("floating_trial_state_as_output") != "FORBIDDEN"
    ):
        raise ValueError("R139 solve formulation contract differs")
    return {
        "status": "PASS",
        "algorithm": candidate["algorithm"],
        "qp_solver": candidate["qp_solver"],
        "lexicographic_qp_passes_per_major_iteration": len(passes),
        "trial_fractions_per_major_iteration": len(trust["trial_fractions"]),
        "initial_configuration_source": endpoint["initial_configuration"]["source"],
        "initial_velocity_source": endpoint["initial_velocity"]["source"],
        "terminal_byte_fixed": endpoint["terminal"]["byte_fixed"],
        "terminal_periodic": endpoint["terminal"]["periodic"],
        "exact_controller_is_sole_acceptance_authority": True,
        "exact_circular_cone_is_sole_acceptance_authority": True,
    }


def audit_numeric_and_budget_contract(profile: Mapping[str, Any]) -> dict[str, Any]:
    numeric = profile["numeric_validity_contract"]
    trust = profile["normalization_and_trust_region_contract"]
    budget = profile["future_execution_budget_contract"]
    residuals = numeric["exact_acceptance_tolerances"]
    if (
        numeric.get("evaluation_dtype") != "float64"
        or numeric.get("rank_policy", {}).get("null_singular_value_maximum") != 1e-12
        or numeric.get("rank_policy", {}).get("retained_singular_value_minimum")
        != 1e-10
        or numeric.get("rank_policy", {}).get("expected_contact_ranks") != [0, 3, 5]
        or numeric.get("rank_policy", {}).get("rank_gap_values") != "INVALID"
        or numeric.get("rank_policy", {}).get("additional_nullity") != "INVALID"
        or numeric.get("coefficient_elision_absolute_maximum") != 1e-14
        or numeric.get("osqp", {}).get("eps_abs") != 1e-5
        or numeric.get("osqp", {}).get("eps_rel") != 1e-5
        or numeric.get("osqp", {}).get("accepted_statuses") != ["solved"]
        or numeric.get("osqp", {}).get("maximum_primal_or_dual_residual") != 5e-5
        or numeric.get("osqp", {}).get("polish") is not True
        or numeric.get("osqp", {}).get("adaptive_rho") is not True
        or numeric.get("osqp", {}).get("termination_authority") != "QP_CANDIDATE_ONLY"
        or residuals.get("scaled_dynamics") != 1e-7
        or residuals.get("manifold_integration") != 1e-7
        or residuals.get("ordinary_velocity_update") != 1e-7
        or residuals.get("momentum_jump") != 1e-7
        or residuals.get("anchor_position_metres") != 1e-6
        or residuals.get("sticking_velocity_metres_per_second") != 1e-6
        or residuals.get("sticking_acceleration_metres_per_second_squared") != 1e-5
        or residuals.get("force_circular_cone_newtons") != 1e-7
        or residuals.get("impulse_circular_cone_newton_seconds") != 1e-7
        or residuals.get("cone_classification_relative_ambiguity") != 1e-12
        or residuals.get("existing_descriptor_and_controller_integer_checks")
        != "EXACT_UNCHANGED"
        or set(numeric.get("invalid_without_restart", ()))
        != EXPECTED_NUMERIC_INVALID_CONDITIONS
        or trust.get("radius", {}).get("initial") != 1
        or trust.get("radius", {}).get("minimum") != 0.03125
        or trust.get("radius", {}).get("maximum") != 2
        or trust.get("actual_to_model_minimum_acceptance_ratio") != 0.1
        or trust.get("contraction_ratio_threshold") != 0.25
        or trust.get("expansion_ratio_threshold") != 0.75
        or trust.get("contraction_factor") != 0.5
        or trust.get("expansion_factor") != 2
        or budget.get("clean_processes") != 1
        or budget.get("observed_threads") != 1
        or budget.get("seed") != 0
        or budget.get("major_iteration_limit") != MAJOR_ITERATION_LIMIT
        or budget.get("lexicographic_qp_passes_per_major_iteration")
        != QP_PASSES_PER_MAJOR_ITERATION
        or budget.get("qp_solve_limit") != QP_SOLVE_LIMIT
        or budget.get("trial_fractions_per_major_iteration")
        != TRIAL_FRACTIONS_PER_MAJOR_ITERATION
        or budget.get("exact_trial_audit_limit") != EXACT_TRIAL_AUDIT_LIMIT
        or budget.get("osqp_iteration_limit_per_qp") != 100000
        or budget.get("wall_time_seconds") != 14400
        or budget.get("peak_rss_bytes") != 17179869184
        or budget.get("restart") != "FORBIDDEN"
        or budget.get("randomized_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
        or budget.get("within_process_factorization_reuse")
        != "TRANSIENT_AND_HASH_COUNTED"
        or budget.get("r136_cache_load") != "FORBIDDEN"
        or budget.get("budget_exhaustion")
        != "VALID_STOP_AND_RESEARCH_NO_TOLERANCE_CHANGE"
    ):
        raise ValueError("R139 numeric or resource budget contract differs")
    return {
        "status": "PASS",
        "evaluation_dtype": numeric["evaluation_dtype"],
        "contact_rank_gap": numeric["rank_policy"],
        "trust_radius": trust["radius"],
        "major_iteration_limit": MAJOR_ITERATION_LIMIT,
        "qp_solve_limit": QP_SOLVE_LIMIT,
        "exact_trial_audit_limit": EXACT_TRIAL_AUDIT_LIMIT,
        "wall_time_seconds": budget["wall_time_seconds"],
        "peak_rss_bytes": budget["peak_rss_bytes"],
    }


def audit_required_hash_contract(profile: Mapping[str, Any]) -> dict[str, Any]:
    required = profile["required_hash_contract"]
    source = profile["source"]
    pre_graph = required.get("before_first_real_graph_call", {})
    accepted = required.get("every_accepted_anchor", [])
    output = required.get("future_execution_output", [])
    counters = required.get("required_counter_closure", [])
    if (
        pre_graph
        != {
            "r120_report": source["r120"]["report_sha256"],
            "r120_cache": source["r120"]["cache_sha256"],
            "r120_accepted_arrays": source["r120"]["accepted_emitted_aggregate_sha256"],
            "r133_report": source["r133"]["report_sha256"],
            "r133_projected_velocity": source["r133"][
                "projected_velocity_float64_sha256"
            ],
            "r133_applied_target": source["r133"]["applied_target_float64_sha256"],
            "r133_applied_effort": source["r133"]["applied_effort_float64_sha256"],
            "r131_mode_sequence": source["r131_motor_mode_sequence_sha256"],
            "r138_graph_index": source["r138"]["graph_index_sha256"],
            "r138_transition_replay": source["r138"]["transition_replay_sha256"],
        }
        or set(accepted)
        != {
            "exact_q_array",
            "exact_v_array",
            "integer_command_array",
            "exact_applied_target_array",
            "exact_applied_effort_array",
            "controller_event_address_array",
            "controller_branch_address_array",
            "exact_funnel_vector",
        }
        or set(output)
        != {
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
        or set(counters) != set(ZERO_EXECUTION_COUNTERS)
    ):
        raise ValueError("R139 required hash contract differs")
    return {
        "status": "PASS",
        "before_first_real_graph_call_hashes": len(pre_graph),
        "every_accepted_anchor_hashes": len(accepted),
        "future_execution_output_hashes": len(output),
        "required_counter_closure": len(counters),
    }


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R139 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R139 {label} canonical report identity differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    decision_document_path: Path,
    r120_profile_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r138_profile_path: Path,
    r138_module_path: Path,
    r138_tool_path: Path,
) -> None:
    source = profile["source"]
    expected = (
        (decision_document_path, source["decision_document_sha256"]),
        (r120_profile_path, source["r120"]["profile_sha256"]),
        (r133_profile_path, source["r133"]["profile_sha256"]),
        (r133_module_path, source["r133"]["module_sha256"]),
        (r133_tool_path, source["r133"]["tool_sha256"]),
        (r138_profile_path, source["r138"]["profile_sha256"]),
        (r138_module_path, source["r138"]["module_sha256"]),
        (r138_tool_path, source["r138"]["tool_sha256"]),
    )
    if any(sha256(path) != digest for path, digest in expected):
        raise ValueError("R139 source file identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    r140 = profile.get("r140_implementation_conformance_contract", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "FixedModeKinodynamicSolveFormulationOnly"
        or source.get("decision_document_sha256")
        != "01caa6be6b7705126b6813ed7e4df148db951e66847d93b820605641b38c5f21"
        or source.get("decision_repository_commit")
        != "813632fe4eef3ad34e8f5200237f01c8e8c91555"
        or source.get("r120", {}).get("report_sha256")
        != "dfcb05e006467ee30bab70aac00f4408acce26782fbdbf5d1cea89821c06953b"
        or source.get("r120", {}).get("report_file_sha256")
        != "35e35581b4062ce3048cb564a7856787efdd59e1fa12b64eef9526200ea4f2fc"
        or source.get("r120", {}).get("profile_sha256")
        != "3dfa2f1b8357cd3452481c9518e8d1ca0ce5c0bb664b3a024fc5ce2653837d55"
        or source.get("r120", {}).get("cache_sha256")
        != "e305fc5888a1cf1dff238c32ac07707a284b215bb49d25920dfb5ee8f097afc5"
        or source.get("r120", {}).get("accepted_emitted_aggregate_sha256")
        != "06e6113d862dcf6acd827962b8a580013f9e3689902b90c9189efaab641841cf"
        or source.get("r120", {}).get("repository_commit")
        != "39708da5036a2f5a2f3bd4f1b4d2f840efe17088"
        or source.get("r133", {}).get("report_sha256")
        != "f1fad2ca3c7abd49acaefd9fcd37873d02fb1d289a3081fd2039b0b1192fd3b6"
        or source.get("r133", {}).get("report_file_sha256")
        != "2ddef1cfae193eb0e32f4d001aba6a8b744056b700fa2ecbe533434bc8e5f7c2"
        or source.get("r133", {}).get("profile_sha256")
        != "20cccf2ffd413ae56148b95ca4cbd67c616f1ee396f2bee995a078ab1685a6a7"
        or source.get("r133", {}).get("module_sha256")
        != "60dbf107a19c795d6c6ffcbbecc4bf0db81dcf0c5efa5adc6a1a43f6d35f0c15"
        or source.get("r133", {}).get("tool_sha256")
        != "427bc05222dc767da3285a58815fa6fa1275ebd6bd3945e43fcb1a89df18f156"
        or source.get("r133", {}).get("projected_velocity_float64_sha256")
        != "e57bcb75629d403e04cc6d3197e8e2dada19ac0c0ef4c920083f2276c33bfb17"
        or source.get("r133", {}).get("applied_target_float64_sha256")
        != "195810f7e9c20b15ec6ddf165c8873f905a96414fe055a0dae7a2b45382be1a3"
        or source.get("r133", {}).get("applied_effort_float64_sha256")
        != "5ad7ac66c4d394ef5d06d111d86f58120069d42e61d1adf1ac82ca1d1d42a1ff"
        or source.get("r133", {}).get("repository_commit")
        != "03f8e0bec8cfc6cb662452b971cd1548f03051dc"
        or source.get("r138", {}).get("report_sha256")
        != "c2343b289a11e7794d4f092b261f18dc6f4776b265854893cd3403fc53daea6c"
        or source.get("r138", {}).get("report_file_sha256")
        != "cec76311db0c0c4d4cb693b806a1fae344eb3e79e24161aa4081c37a5e258d83"
        or source.get("r138", {}).get("profile_sha256")
        != "54e6291852997450f4acd2e15f9fb6e499a16196b9aea53295ffef74a979b216"
        or source.get("r138", {}).get("module_sha256")
        != "ba244e4540f15d354a2709dbb47b72e919b380c2420af96058e51060fa49e306"
        or source.get("r138", {}).get("tool_sha256")
        != "db4146dfa181b8020d55e561a10bc6da51cb99f3da41813bdebf74416fe9fdb4"
        or source.get("r138", {}).get("graph_index_sha256")
        != "844ed3a2e727712fedff2c0cd858c30639c8956e054f79f2012d67277535639b"
        or source.get("r138", {}).get("transition_replay_sha256")
        != "9f289c31ecfe2ef18b4beed4559982c3da25f7244eb6b17c022ac6000edc322a"
        or source.get("r138", {}).get("repository_commit")
        != "4745c0aa1273d3d256f1a468820813272b917a11"
        or source.get("r131_motor_mode_sequence_sha256")
        != "ab30a912550e06e1219b1fab87e0d93044eba6b0cdf8e7377f7b27a0e1915ca4"
        or scope.get("formulation_id") != "R139"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("source_reports_read") != 3
        or scope.get("source_array_payloads_read") != 0
        or scope.get("solve_formulations") != 1
        or any(scope.get(counter) != 0 for counter in ZERO_EXECUTION_COUNTERS)
        or r140.get("claim") != "FixedModeKinodynamicSolveImplementationConformanceOnly"
        or r140.get("real_state_reconstructions") != 0
        or r140.get("real_controller_graph_evaluations") != 0
        or r140.get("real_kinodynamic_system_assemblies") != 0
        or r140.get("real_qp_solves") != 0
        or r140.get("real_kinodynamic_solves") != 0
        or r140.get("candidate_artifacts") != 0
        or r140.get("physx_scene_runs") != 0
        or r140.get("training_runs") != 0
        or len(r140.get("required_checks", ())) != 8
        or r140.get("pass_transition")
        != "PERMIT_EXPLICIT_ROADMAP_DECISION_FOR_ONE_BOUNDED_R141_ONLY"
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r140_kinodynamic_solve_implementation_conformance")
        != "AUTHORIZED_REPORT_ONLY_ON_R139_COMPLETE"
        or any(
            not str(value).startswith("NOT_AUTHORIZED")
            for key, value in bounded.items()
            if key != "r140_kinodynamic_solve_implementation_conformance"
        )
        or profile.get("decision", {}).get("complete")
        != "PERMIT_SEPARATE_REPORT_ONLY_R140_KINODYNAMIC_SOLVE_IMPLEMENTATION_CONFORMANCE_ONLY"
        or profile.get("result_transitions", {}).get("complete")
        != "R139_COMPLETE_R140_CONFORMANCE_ONLY"
        or profile.get("result_transitions", {}).get("invalid")
        != "R139_INVALID_STOP_WITHOUT_EXECUTION"
    ):
        raise ValueError("R139 formulation profile differs")
    audit_solve_formulation(profile)
    audit_numeric_and_budget_contract(profile)
    audit_required_hash_contract(profile)


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R139 formulation requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R139 validation result differs")
    return [dict(row) for row in results]
