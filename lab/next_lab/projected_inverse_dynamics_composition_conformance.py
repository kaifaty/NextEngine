from __future__ import annotations

import hashlib
import inspect
import json
import resource
import time
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.gauge_aware_fixed_pd_execution import solve_gauge_aware_collocation
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import ReducedLocalSystem
from next_lab.projected_inverse_dynamics_composition import (
    ARRAY_SHAPES,
    assemble_projected_reduced_local_system,
    projected_reduced_column_scale,
    verify_r133_array_hashes,
)

CONFORMANCE_ID = (
    "nextengine.humanoid-projected-inverse-dynamics-composition-conformance.v1"
)
CHECK_ID = "TRAIN-4-PROJECTED-INVERSE-DYNAMICS-COMPOSITION-CONFORMANCE"
GENERALIZED_WIDTH = 29
ACTUATOR_COUNT = 23
POINT_COUNT = 4
POINT_FORCE_WIDTH = 3
COLLOCATION_COUNT = 3200
ZERO_REAL_COUNTERS = (
    "real_state_lift_evaluations",
    "real_state_projection_systems",
    "real_projection_factorizations",
    "real_projection_solves",
    "real_controller_schedule_derivations",
    "real_inverse_dynamics_system_assemblies",
    "real_particular_solutions",
    "real_gauge_interval_classifications",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "solver_private_caches_built",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)
BOUNDED_ACCEPTANCE_KEYS = frozenset(
    {
        "r135_retry",
        "r136_projected_inverse_dynamics_execution",
        "r123_retry",
        "r127_retry",
        "r129_retry",
        "r130_retry",
        "r132_retry",
        "r133_retry",
        "r134_retry",
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


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_projected_inverse_dynamics_composition_conformance(
    *,
    profile_path: Path,
    r134_report_path: Path,
    r134_profile_path: Path,
    r134_module_path: Path,
    r134_tool_path: Path,
    r133_report_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r126_report_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    composition_module_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Conform R135 with synthetic systems and zero real schedule work."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r134_report_path,
            r134_profile_path,
            r134_module_path,
            r134_tool_path,
            r133_report_path,
            r133_profile_path,
            r133_module_path,
            r133_tool_path,
            r126_report_path,
            r126_profile_path,
            gauge_aware_kernel_path,
            r126_conformance_module_path,
            r126_tool_path,
            composition_module_path,
            tool_path,
        )
    )
    (
        profile_path,
        r134_report_path,
        r134_profile_path,
        r134_module_path,
        r134_tool_path,
        r133_report_path,
        r133_profile_path,
        r133_module_path,
        r133_tool_path,
        r126_report_path,
        r126_profile_path,
        gauge_aware_kernel_path,
        r126_conformance_module_path,
        r126_tool_path,
        composition_module_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R135 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile)
    source = profile["source"]
    r134 = _load_bound_report(r134_report_path, source["r134"], "R134")
    r133 = _load_bound_report(r133_report_path, source["r133"], "R133")
    r126 = _load_bound_report(r126_report_path, source["r126"], "R126")
    _validate_source_files(
        profile=profile,
        r134_profile_path=r134_profile_path,
        r134_module_path=r134_module_path,
        r134_tool_path=r134_tool_path,
        r133_profile_path=r133_profile_path,
        r133_module_path=r133_module_path,
        r133_tool_path=r133_tool_path,
        r126_profile_path=r126_profile_path,
        gauge_aware_kernel_path=gauge_aware_kernel_path,
        r126_conformance_module_path=r126_conformance_module_path,
        r126_tool_path=r126_tool_path,
        composition_module_path=composition_module_path,
    )
    if (
        sha256(Path(__file__).resolve()) != source["conformance_module_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R135 current conformance identity differs")
    source_audit = audit_source_lineage(profile, r134=r134, r133=r133, r126=r126)
    validations = _validate_results(profile, validation_results)

    started = time.monotonic()
    rss_before = _maximum_resident_memory_bytes()
    threads_before = _linux_thread_count()
    api_audit = audit_composition_api()
    block_audit = audit_synthetic_block_composition()
    warm_audit = audit_warm_scale_independence()
    solver_audit = audit_synthetic_rank_and_cones(profile["numeric_contract"])
    hash_audit = audit_synthetic_hash_guards()
    elapsed = time.monotonic() - started
    resource_audit = {
        "status": "PASS",
        "wall_clock_seconds": elapsed,
        "maximum_resident_memory_bytes": max(
            rss_before, _maximum_resident_memory_bytes()
        ),
        "maximum_observed_os_thread_count": max(threads_before, _linux_thread_count()),
        "execution_process_count": 1,
        "child_processes_spawned_during_conformance": 0,
    }
    budget = profile["resource_budget"]
    if (
        resource_audit["wall_clock_seconds"] > budget["maximum_wall_clock_seconds"]
        or resource_audit["maximum_resident_memory_bytes"]
        > budget["maximum_resident_memory_bytes"]
        or resource_audit["maximum_observed_os_thread_count"] > budget["thread_count"]
    ):
        resource_audit["status"] = "FAIL"

    audits = (api_audit, block_audit, warm_audit, solver_audit, hash_audit)
    passed = all(audit["status"] == "PASS" for audit in audits) and (
        resource_audit["status"] == "PASS"
    )
    status = "PASS" if passed else "FAIL"
    transition_key = "pass" if passed else "fail"
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": status,
        "claim": profile["claim"],
        "gate_decision": profile["decision"][transition_key],
        "result_transition": profile["result_transitions"][transition_key],
        "scope": profile["scope"],
        "source_lineage_audit": source_audit,
        "composition_api_audit": api_audit,
        "synthetic_block_composition_audit": block_audit,
        "warm_acceleration_scale_audit": warm_audit,
        "synthetic_rank_and_cone_audit": solver_audit,
        "synthetic_r133_hash_guard_audit": hash_audit,
        "numeric_contract": profile["numeric_contract"],
        "future_r136_execution_contract": profile["future_r136_execution_contract"],
        "resource_audit": resource_audit,
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r134_report_file_sha256": sha256(r134_report_path),
            "r134_profile_sha256": sha256(r134_profile_path),
            "r134_module_sha256": sha256(r134_module_path),
            "r134_tool_sha256": sha256(r134_tool_path),
            "r133_report_file_sha256": sha256(r133_report_path),
            "r133_profile_sha256": sha256(r133_profile_path),
            "r133_module_sha256": sha256(r133_module_path),
            "r133_tool_sha256": sha256(r133_tool_path),
            "r126_report_file_sha256": sha256(r126_report_path),
            "r126_profile_sha256": sha256(r126_profile_path),
            "gauge_aware_kernel_sha256": sha256(gauge_aware_kernel_path),
            "r126_conformance_module_sha256": sha256(r126_conformance_module_path),
            "r126_tool_sha256": sha256(r126_tool_path),
            "composition_module_sha256": sha256(composition_module_path),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "composition_conformance_runs": 1,
        "synthetic_system_compositions": block_audit["case_count"],
        "synthetic_singular_value_decompositions": solver_audit[
            "singular_value_decompositions"
        ],
        "synthetic_particular_solutions": solver_audit["particular_solutions"],
        "synthetic_gauge_interval_classifications": solver_audit[
            "gauge_interval_classifications"
        ],
        "synthetic_hash_guard_cases": hash_audit["case_count"],
        **{counter: 0 for counter in ZERO_REAL_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_composition_api() -> dict[str, Any]:
    assembly = inspect.signature(assemble_projected_reduced_local_system)
    scale = inspect.signature(projected_reduced_column_scale)
    assembly_names = list(assembly.parameters)
    scale_names = list(scale.parameters)
    expected_assembly = [
        "mass",
        "bias_from_projected_velocity",
        "applied_effort_micronewton_metres",
        "contact_jacobians",
        "contact_jdot_v_from_projected_velocity",
        "active_point_ordinals",
        "modes",
    ]
    expected_scale = [
        "r120_acceleration_knots",
        "total_body_mass_kilograms",
        "gravity_y_metres_per_second_squared",
        "active_point_count",
    ]
    passed = (
        assembly_names == expected_assembly
        and scale_names == expected_scale
        and all(
            parameter.kind is inspect.Parameter.KEYWORD_ONLY
            for parameter in assembly.parameters.values()
        )
        and "warm" not in " ".join(assembly_names)
        and "r120_acceleration_knots" in scale_names
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "assembly_keyword_order": assembly_names,
        "scale_keyword_order": scale_names,
        "warm_acceleration_absent_from_system_api": "warm"
        not in " ".join(assembly_names),
        "projected_velocity_terms_explicit": (
            "bias_from_projected_velocity" in assembly_names
            and "contact_jdot_v_from_projected_velocity" in assembly_names
        ),
    }


def audit_synthetic_block_composition() -> dict[str, Any]:
    cases = (
        ("flight_29", (0, 0), ()),
        ("single_32", (2, 0), (1,)),
        ("flat_35", (0, 3), (2, 3)),
    )
    rows = []
    for ordinal, (name, modes, active) in enumerate(cases):
        mass, bias, effort, jacobians, jdot_v = _synthetic_composition_inputs(ordinal)
        matrix, right = assemble_projected_reduced_local_system(
            mass=mass,
            bias_from_projected_velocity=bias,
            applied_effort_micronewton_metres=effort,
            contact_jacobians=jacobians,
            contact_jdot_v_from_projected_velocity=jdot_v,
            active_point_ordinals=active,
            modes=np.asarray(modes, dtype=np.uint8),
        )
        expected_matrix, expected_right = _independent_block_oracle(
            mass=mass,
            bias=bias,
            effort_micronewton_metres=effort,
            jacobians=jacobians,
            jdot_v=jdot_v,
            active=active,
        )
        matrix_error = float(np.max(np.abs(matrix - expected_matrix)))
        right_error = float(np.max(np.abs(right - expected_right)))
        expected_width = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * len(active)
        rows.append(
            {
                "case": name,
                "status": "PASS"
                if matrix.shape == (expected_width, expected_width)
                and right.shape == (expected_width,)
                and matrix_error == 0.0
                and right_error == 0.0
                else "FAIL",
                "matrix_shape": list(matrix.shape),
                "active_point_ordinals": list(active),
                "maximum_matrix_oracle_error": matrix_error,
                "maximum_right_hand_side_oracle_error": right_error,
            }
        )

    mass, bias, effort, jacobians, jdot_v = _synthetic_composition_inputs(9)
    base_matrix, base_right = assemble_projected_reduced_local_system(
        mass=mass,
        bias_from_projected_velocity=bias,
        applied_effort_micronewton_metres=effort,
        contact_jacobians=jacobians,
        contact_jdot_v_from_projected_velocity=jdot_v,
        active_point_ordinals=(1,),
        modes=np.asarray((2, 0), dtype=np.uint8),
    )
    changed_bias = bias + np.linspace(0.1, 0.3, GENERALIZED_WIDTH)
    changed_jdot = jdot_v.copy()
    changed_jdot[1] += (0.4, 0.5, 0.6)
    changed_matrix, changed_right = assemble_projected_reduced_local_system(
        mass=mass,
        bias_from_projected_velocity=changed_bias,
        applied_effort_micronewton_metres=effort,
        contact_jacobians=jacobians,
        contact_jdot_v_from_projected_velocity=changed_jdot,
        active_point_ordinals=(1,),
        modes=np.asarray((2, 0), dtype=np.uint8),
    )
    velocity_input_audit = {
        "status": "PASS"
        if np.array_equal(base_matrix, changed_matrix)
        and np.allclose(
            changed_right[:GENERALIZED_WIDTH] - base_right[:GENERALIZED_WIDTH],
            -(changed_bias - bias),
            rtol=0.0,
            atol=0.0,
        )
        and np.allclose(
            changed_right[GENERALIZED_WIDTH:] - base_right[GENERALIZED_WIDTH:],
            -(changed_jdot[1] - jdot_v[1]),
            rtol=0.0,
            atol=0.0,
        )
        else "FAIL",
        "matrix_unchanged": bool(np.array_equal(base_matrix, changed_matrix)),
        "bias_delta_routed_to_dynamics_rhs": True,
        "jdot_v_delta_routed_to_closure_rhs": True,
    }
    return {
        "status": "PASS"
        if all(row["status"] == "PASS" for row in rows)
        and velocity_input_audit["status"] == "PASS"
        else "FAIL",
        "case_count": len(rows),
        "cases": rows,
        "projected_velocity_input_audit": velocity_input_audit,
    }


def audit_warm_scale_independence() -> dict[str, Any]:
    mass, bias, effort, jacobians, jdot_v = _synthetic_composition_inputs(17)
    matrix, right = assemble_projected_reduced_local_system(
        mass=mass,
        bias_from_projected_velocity=bias,
        applied_effort_micronewton_metres=effort,
        contact_jacobians=jacobians,
        contact_jdot_v_from_projected_velocity=jdot_v,
        active_point_ordinals=(2, 3),
        modes=np.asarray((0, 3), dtype=np.uint8),
    )
    low = np.zeros((2, GENERALIZED_WIDTH), dtype=np.float64)
    high = np.tile(np.linspace(2.0, 4.0, GENERALIZED_WIDTH, dtype=np.float64), (2, 1))
    low_scale = projected_reduced_column_scale(
        r120_acceleration_knots=low,
        total_body_mass_kilograms=75.337,
        gravity_y_metres_per_second_squared=-9.81,
        active_point_count=2,
    )
    high_scale = projected_reduced_column_scale(
        r120_acceleration_knots=high,
        total_body_mass_kilograms=75.337,
        gravity_y_metres_per_second_squared=-9.81,
        active_point_count=2,
    )
    matrix_again, right_again = assemble_projected_reduced_local_system(
        mass=mass,
        bias_from_projected_velocity=bias,
        applied_effort_micronewton_metres=effort,
        contact_jacobians=jacobians,
        contact_jdot_v_from_projected_velocity=jdot_v,
        active_point_ordinals=(2, 3),
        modes=np.asarray((0, 3), dtype=np.uint8),
    )
    passed = (
        np.array_equal(matrix, matrix_again)
        and np.array_equal(right, right_again)
        and np.all(low_scale[:GENERALIZED_WIDTH] == 1.0)
        and np.array_equal(high_scale[:GENERALIZED_WIDTH], high[0])
        and np.array_equal(
            low_scale[GENERALIZED_WIDTH:], high_scale[GENERALIZED_WIDTH:]
        )
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "matrix_byte_identical_across_warm_scale_inputs": bool(
            np.array_equal(matrix, matrix_again)
        ),
        "right_hand_side_byte_identical_across_warm_scale_inputs": bool(
            np.array_equal(right, right_again)
        ),
        "acceleration_scale_changes": bool(
            not np.array_equal(
                low_scale[:GENERALIZED_WIDTH], high_scale[:GENERALIZED_WIDTH]
            )
        ),
        "force_scale_unchanged": bool(
            np.array_equal(
                low_scale[GENERALIZED_WIDTH:], high_scale[GENERALIZED_WIDTH:]
            )
        ),
    }


def audit_synthetic_rank_and_cones(
    numeric_contract: Mapping[str, Any],
) -> dict[str, Any]:
    rows: list[dict[str, Any]] = []
    systems = (
        (
            "flight_feasible",
            _identity_system(29, modes=(0, 0), active=()),
            "VALID",
            "FEASIBLE",
        ),
        (
            "single_feasible",
            _single_point_system(normal=10.0, right=1.0),
            "VALID",
            "FEASIBLE",
        ),
        (
            "single_infeasible",
            _single_point_system(normal=1.0, right=2.0),
            "VALID",
            "INFEASIBLE",
        ),
        (
            "flat_gauge_feasible",
            _flat_projector_system(right_force=1.0),
            "VALID",
            "FEASIBLE",
        ),
        (
            "flat_gauge_infeasible",
            _flat_projector_system(right_force=9.0),
            "VALID",
            "INFEASIBLE",
        ),
        (
            "rank_gap_invalid",
            _rank_gap_system(),
            "INVALID",
            None,
        ),
        (
            "inconsistent_rhs_invalid",
            _inconsistent_flat_system(),
            "INVALID",
            None,
        ),
    )
    particular_count = 0
    gauge_count = 0
    for name, system, expected_status, expected_feasibility in systems:
        solution = solve_gauge_aware_collocation(
            system,
            column_scale=np.ones(system.matrix.shape[1], dtype=np.float64),
            numeric_contract=numeric_contract,
        )
        particular_count += int(solution.particular_solution is not None)
        gauge_count += int(solution.gauge_feasibility is not None)
        passed = solution.status == expected_status and (
            expected_feasibility is None or solution.feasibility == expected_feasibility
        )
        rows.append(
            {
                "case": name,
                "status": "PASS" if passed else "FAIL",
                "analysis_status": solution.status,
                "feasibility": solution.feasibility,
                "invalid_reason": solution.invalid_reason,
                "rank": solution.rank_analysis.rank,
                "nullity": solution.rank_analysis.nullity,
            }
        )
    return {
        "status": "PASS" if all(row["status"] == "PASS" for row in rows) else "FAIL",
        "cases": rows,
        "singular_value_decompositions": len(rows),
        "particular_solutions": particular_count,
        "gauge_interval_classifications": gauge_count,
    }


def audit_synthetic_hash_guards() -> dict[str, Any]:
    arrays = {
        name: np.zeros(shape, dtype=np.float64) for name, shape in ARRAY_SHAPES.items()
    }
    expected = {
        name: hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()
        for name, value in arrays.items()
    }
    verified = verify_r133_array_hashes(arrays, expected)
    rejected = False
    mutated = {name: value.copy() for name, value in arrays.items()}
    mutated["projected_generalized_velocity"][0, 0] = 1.0
    try:
        verify_r133_array_hashes(mutated, expected)
    except ValueError as error:
        rejected = "hash differs" in str(error)
    passed = verified == expected and rejected
    return {
        "status": "PASS" if passed else "FAIL",
        "case_count": 2,
        "matching_synthetic_inventory_accepted": verified == expected,
        "one_scalar_mutation_rejected": rejected,
        "real_r133_array_values_read": 0,
    }


def audit_source_lineage(
    profile: Mapping[str, Any],
    *,
    r134: Mapping[str, Any],
    r133: Mapping[str, Any],
    r126: Mapping[str, Any],
) -> dict[str, Any]:
    source = profile["source"]
    r134_ids = r134.get("identities", {})
    r133_hashes = profile["r133_array_hash_contract"]
    r133_projection = r133.get("projection_result", {}).get("aggregate", {})
    r133_schedule = r133.get("projected_fixed_pd_schedule_audit", {})
    if (
        r134.get("status") != "COMPLETE"
        or r134.get("gate_decision")
        != "PERMIT_SEPARATE_REPORT_ONLY_R135_PROJECTED_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY"
        or r134.get("result_transition") != "R134_COMPLETE_R135_CONFORMANCE_ONLY"
        or r134.get("repository", {}).get("commit")
        != source["r134"]["repository_commit"]
        or r134.get("repository", {}).get("dirty") is not False
        or r134.get("inverse_dynamics_system_assemblies") != 0
        or r134.get("state_lift_evaluations") != 0
        or r134.get("state_projection_systems") != 0
        or r134.get("controller_schedule_derivations") != 0
        or r134.get("bounded_acceptance", {}).get(
            "r135_projected_inverse_dynamics_implementation_conformance"
        )
        != "AUTHORIZED_REPORT_ONLY_ON_R134_COMPLETE"
        or r134.get("future_r136_execution_contract")
        != profile["future_r136_execution_contract"]
        or r134.get("system_inventory_audit") != profile["system_inventory_contract"]
    ):
        raise ValueError("R135 R134 source contract differs")
    if (
        r133.get("status") != "PASS"
        or r133.get("report_sha256")
        != r134.get("source_lineage_audit", {}).get("r133_report_sha256")
        or r133_schedule.get("status") != "PASS"
        or r133_projection.get("array_sha256", {}).get("projected_generalized_velocity")
        != r133_hashes["projected_generalized_velocity"]
        or r133_projection.get("array_sha256", {}).get("projection_delta_velocity")
        != r133_hashes["projection_delta_velocity"]
        or r133_schedule.get("applied_target_float64_sha256")
        != r133_hashes["applied_target_microradians"]
        or r133_schedule.get("applied_effort_float64_sha256")
        != r133_hashes["applied_effort_micronewton_metres"]
        or r133.get("inverse_dynamics_system_assemblies") != 0
        or r126.get("status") != "PASS"
        or r126.get("report_sha256")
        != r134.get("source_lineage_audit", {}).get("r126_report_sha256")
        or r126.get("real_schedule_particular_solutions") != 0
        or r134_ids.get("r133_report_file_sha256")
        != source["r133"]["report_file_sha256"]
        or r134_ids.get("r126_report_file_sha256")
        != source["r126"]["report_file_sha256"]
    ):
        raise ValueError("R135 upstream source contract differs")
    return {
        "status": "PASS",
        "r134_report_sha256": r134["report_sha256"],
        "r133_report_sha256": r133["report_sha256"],
        "r133_projected_velocity_sha256": r133_schedule[
            "projected_velocity_float64_sha256"
        ],
        "r133_projection_delta_sha256": r133_hashes["projection_delta_velocity"],
        "r133_applied_target_sha256": r133_schedule["applied_target_float64_sha256"],
        "r133_applied_effort_sha256": r133_schedule["applied_effort_float64_sha256"],
        "r126_report_sha256": r126["report_sha256"],
        "r126_gauge_kernel_conformance": "PASS",
    }


def _synthetic_composition_inputs(
    salt: int,
) -> tuple[
    NDArray[np.float64],
    NDArray[np.float64],
    NDArray[np.float64],
    NDArray[np.float64],
    NDArray[np.float64],
]:
    mass = np.diag(np.linspace(1.0, 3.8, GENERALIZED_WIDTH, dtype=np.float64))
    bias = np.linspace(-2.0, 2.0, GENERALIZED_WIDTH, dtype=np.float64) + salt
    effort = np.arange(ACTUATOR_COUNT, dtype=np.float64) * 1000.0 + float(
        salt * 1_000_000
    )
    jacobians = np.arange(
        POINT_COUNT * POINT_FORCE_WIDTH * GENERALIZED_WIDTH, dtype=np.float64
    ).reshape(POINT_COUNT, POINT_FORCE_WIDTH, GENERALIZED_WIDTH)
    jacobians = (jacobians + salt) / 10_000.0
    jdot_v = (
        np.arange(POINT_COUNT * POINT_FORCE_WIDTH, dtype=np.float64).reshape(
            POINT_COUNT, POINT_FORCE_WIDTH
        )
        + salt
    ) / 100.0
    return mass, bias, effort, jacobians, jdot_v


def _independent_block_oracle(
    *,
    mass: NDArray[np.float64],
    bias: NDArray[np.float64],
    effort_micronewton_metres: NDArray[np.float64],
    jacobians: NDArray[np.float64],
    jdot_v: NDArray[np.float64],
    active: Sequence[int],
) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
    unknowns = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * len(active)
    matrix = np.zeros((unknowns, unknowns), dtype=np.float64)
    right = np.zeros(unknowns, dtype=np.float64)
    matrix[:GENERALIZED_WIDTH, :GENERALIZED_WIDTH] = mass
    right[:GENERALIZED_WIDTH] = -bias
    right[6:GENERALIZED_WIDTH] += effort_micronewton_metres / 1_000_000.0
    for local, point in enumerate(active):
        offset = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * local
        columns = slice(offset, offset + POINT_FORCE_WIDTH)
        matrix[:GENERALIZED_WIDTH, columns] = -jacobians[point][[1, 0, 2], :].T
        matrix[columns, :GENERALIZED_WIDTH] = jacobians[point]
        right[columns] = -jdot_v[point]
    return matrix, right


def _identity_system(
    width: int, *, modes: tuple[int, int], active: tuple[int, ...]
) -> ReducedLocalSystem:
    target = np.linspace(-0.2, 0.2, width, dtype=np.float64)
    return _algebraic_system(
        np.eye(width, dtype=np.float64),
        target,
        modes=modes,
        active=active,
        gauge=np.empty((width, 0), dtype=np.float64),
    )


def _single_point_system(*, normal: float, right: float) -> ReducedLocalSystem:
    target = np.zeros(32, dtype=np.float64)
    target[-3:] = (normal, right, 0.0)
    return _algebraic_system(
        np.eye(32, dtype=np.float64),
        target,
        modes=(0, 2),
        active=(3,),
        gauge=np.empty((32, 0), dtype=np.float64),
    )


def _flat_projector_system(*, right_force: float) -> ReducedLocalSystem:
    gauge = np.zeros((35, 1), dtype=np.float64)
    gauge[30, 0] = 1.0
    gauge[33, 0] = -1.0
    unit = gauge[:, 0] / np.linalg.norm(gauge[:, 0])
    matrix = np.eye(35, dtype=np.float64) - np.outer(unit, unit)
    target = np.zeros(35, dtype=np.float64)
    target[29:32] = (10.0, right_force, 0.0)
    target[32:35] = (10.0, right_force, 0.0)
    return _algebraic_system(
        matrix,
        matrix @ target,
        modes=(0, 3),
        active=(2, 3),
        gauge=gauge,
    )


def _rank_gap_system() -> ReducedLocalSystem:
    diagonal = np.ones(32, dtype=np.float64)
    diagonal[-1] = 1.0e-11
    return _algebraic_system(
        np.diag(diagonal),
        np.zeros(32, dtype=np.float64),
        modes=(0, 2),
        active=(3,),
        gauge=np.empty((32, 0), dtype=np.float64),
    )


def _inconsistent_flat_system() -> ReducedLocalSystem:
    system = _flat_projector_system(right_force=1.0)
    right = system.right_hand_side.copy()
    right += system.analytic_gauge_matrix[:, 0]
    return _algebraic_system(
        system.matrix,
        right,
        modes=(0, 3),
        active=(2, 3),
        gauge=system.analytic_gauge_matrix,
    )


def _algebraic_system(
    matrix: NDArray[np.float64],
    right: NDArray[np.float64],
    *,
    modes: tuple[int, int],
    active: tuple[int, ...],
    gauge: NDArray[np.float64],
) -> ReducedLocalSystem:
    return ReducedLocalSystem(
        matrix=np.asarray(matrix, dtype=np.float64),
        right_hand_side=np.asarray(right, dtype=np.float64),
        active_point_ordinals=active,
        modes=np.asarray(modes, dtype=np.uint8),
        analytic_gauge_matrix=np.asarray(gauge, dtype=np.float64),
        gauge_identities=(),
    )


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R135 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R135 {label} canonical report identity differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r134_profile_path: Path,
    r134_module_path: Path,
    r134_tool_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r126_profile_path: Path,
    gauge_aware_kernel_path: Path,
    r126_conformance_module_path: Path,
    r126_tool_path: Path,
    composition_module_path: Path,
) -> None:
    source = profile["source"]
    expected = (
        (r134_profile_path, source["r134"]["profile_sha256"]),
        (r134_module_path, source["r134"]["module_sha256"]),
        (r134_tool_path, source["r134"]["tool_sha256"]),
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
        (composition_module_path, source["composition_module_sha256"]),
    )
    if any(sha256(path) != digest for path, digest in expected):
        raise ValueError("R135 source file identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    bounded = profile.get("bounded_acceptance", {})
    expected = profile.get("expected_synthetic_inventory", {})
    numeric = profile.get("numeric_contract", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnlyConformance"
        or profile.get("claim")
        != "ProjectedInverseDynamicsCompositionImplementationConformanceOnly"
        or scope.get("run_id") != "R135"
        or scope.get("source_reports_read") != 3
        or scope.get("real_collocations_read") != 0
        or any(scope.get(counter) != 0 for counter in ZERO_REAL_COUNTERS)
        or expected.get("system_compositions") != 3
        or expected.get("singular_value_decompositions") != 7
        or expected.get("particular_solutions") != 6
        or expected.get("gauge_interval_classifications") != 2
        or expected.get("hash_guard_cases") != 2
        or expected.get("real_system_compositions") != 0
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("physical_group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or profile.get("future_r136_execution_contract", {}).get("authority")
        != "NOT_GRANTED_UNTIL_EXACT_R135_PASS"
        or profile.get("future_r136_execution_contract", {}).get("collocation_count")
        != COLLOCATION_COUNT
        or set(bounded) != BOUNDED_ACCEPTANCE_KEYS
        or bounded.get("r136_projected_inverse_dynamics_execution")
        != "AUTHORIZED_ONE_EXECUTION_ONLY_ON_R135_PASS"
        or any(
            value != "NOT_AUTHORIZED"
            for key, value in bounded.items()
            if key != "r136_projected_inverse_dynamics_execution"
        )
        or profile.get("decision", {}).get("pass")
        != "PERMIT_ONE_R136_PROJECTED_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or profile.get("result_transitions", {}).get("pass")
        != "R135_PASS_R136_SINGLE_EXECUTION_ONLY"
    ):
        raise ValueError("R135 conformance profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R135 conformance requires a clean repository")


def _validate_execution_environment(profile: Mapping[str, Any]) -> None:
    import os

    expected = profile["single_thread_environment"]
    if any(os.environ.get(name) != value for name, value in expected.items()):
        raise ValueError("R135 single-thread environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R135 validation result differs")
    return [dict(row) for row in results]


def _linux_thread_count() -> int:
    status = Path("/proc/self/status")
    if not status.is_file():
        return 1
    for line in status.read_text(encoding="utf-8").splitlines():
        if line.startswith("Threads:"):
            return int(line.split(":", 1)[1].strip())
    raise ValueError("R135 Linux thread count is absent")


def _maximum_resident_memory_bytes() -> int:
    maximum = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return maximum * 1024
