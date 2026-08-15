from __future__ import annotations

import hashlib
import json
import math
import resource
import time
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from decimal import Decimal, getcontext
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _contact_points,
    load_r120_cache,
)
from next_lab.fixed_pd_inverse_dynamics_execution import (
    collocation_state,
    derive_fixed_pd_schedule,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import FRAME_COUNT
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    build_spatial_model,
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import (
    FRICTION,
    FRICTION_DENOMINATOR,
    FRICTION_NUMERATOR,
    ReducedLocalSystem,
    analyse_reduced_system,
    build_reduced_local_system,
    classify_flat_gauge_feasibility,
    line_cone_interval,
    reduced_column_scale,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

CONFORMANCE_ID = "nextengine.humanoid-redundant-contact-implementation-conformance.v1"
CHECK_ID = "TRAIN-4-REDUNDANT-CONTACT-IMPLEMENTATION-CONFORMANCE"
POINT_FORCE_WIDTH = 3


@dataclass(frozen=True)
class SourceReports:
    r125: dict[str, Any]
    r122: dict[str, Any]
    r121: dict[str, Any]
    r113: dict[str, Any]
    r120: dict[str, Any]


def build_redundant_contact_implementation_conformance(
    *,
    profile_path: Path,
    r125_report_path: Path,
    r125_profile_path: Path,
    r125_module_path: Path,
    r125_tool_path: Path,
    r122_report_path: Path,
    r122_profile_path: Path,
    r122_kernel_path: Path,
    r122_conformance_module_path: Path,
    r122_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r120_report_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> dict[str, Any]:
    """Run R126 conformance without solving a real schedule particular point."""

    start = time.monotonic()
    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r125_report_path,
            r125_profile_path,
            r125_module_path,
            r125_tool_path,
            r122_report_path,
            r122_profile_path,
            r122_kernel_path,
            r122_conformance_module_path,
            r122_tool_path,
            r121_report_path,
            r121_profile_path,
            r113_report_path,
            r113_profile_path,
            r120_report_path,
            r120_cache_path,
            v9_complete_clip_path,
            tool_path,
        )
    )
    (
        profile_path,
        r125_report_path,
        r125_profile_path,
        r125_module_path,
        r125_tool_path,
        r122_report_path,
        r122_profile_path,
        r122_kernel_path,
        r122_conformance_module_path,
        r122_tool_path,
        r121_report_path,
        r121_profile_path,
        r113_report_path,
        r113_profile_path,
        r120_report_path,
        r120_cache_path,
        v9_complete_clip_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R126 conformance input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    reports = _load_and_validate_sources(
        profile=profile,
        r125_report_path=r125_report_path,
        r125_profile_path=r125_profile_path,
        r125_module_path=r125_module_path,
        r125_tool_path=r125_tool_path,
        r122_report_path=r122_report_path,
        r122_profile_path=r122_profile_path,
        r122_kernel_path=r122_kernel_path,
        r122_conformance_module_path=r122_conformance_module_path,
        r122_tool_path=r122_tool_path,
        r121_report_path=r121_report_path,
        r121_profile_path=r121_profile_path,
        r113_report_path=r113_report_path,
        r113_profile_path=r113_profile_path,
        r120_report_path=r120_report_path,
        r120_cache_path=r120_cache_path,
        v9_complete_clip_path=v9_complete_clip_path,
    )
    source = profile["source"]
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != source["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve()) != source["conformance_module_sha256"]
        or sha256(Path(build_reduced_local_system.__code__.co_filename).resolve())
        != source["gauge_aware_kernel_sha256"]
        or sha256(tool_path) != source["tool_sha256"]
    ):
        raise ValueError("R126 current implementation identity differs")

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    r122_profile = json.loads(r122_profile_path.read_bytes())
    cache = load_r120_cache(r120_cache_path, r122_profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R126 V9 contact modes differ")
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=reports.r113)
    fixed_pd = derive_fixed_pd_schedule(cache=cache, descriptor=descriptor)
    if fixed_pd.audit != reports.r121["fixed_pd_schedule_audit"]:
        raise ValueError("R126 fixed-PD schedule differs from R121")
    effort = fixed_pd.applied_effort_micronewton_metres / 1_000_000.0

    anchor_audits = audit_frozen_anchors(
        profile=profile,
        model=model,
        cache=cache,
        contact_modes=contact_modes,
        points=points,
        effort_newton_metres=effort,
    )
    synthetic_rank = audit_synthetic_rank_cases(profile)
    synthetic_cones = audit_synthetic_line_cones(profile)
    resource_audit = audit_maximum_payload(profile)
    passed = bool(
        all(row["status"] == "PASS" for row in anchor_audits)
        and synthetic_rank["status"] == "PASS"
        and synthetic_cones["status"] == "PASS"
        and resource_audit["status"] == "PASS"
    )
    elapsed = time.monotonic() - start
    maximum_threads = _linux_thread_count()
    maximum_rss = _resident_memory_bytes()
    budget = profile["resource_budget"]
    if (
        elapsed > float(budget["maximum_wall_clock_seconds"])
        or maximum_threads > int(budget["thread_count"])
        or maximum_rss > int(budget["maximum_resident_memory_bytes"])
    ):
        passed = False

    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": "PASS" if passed else "FAIL",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["pass" if passed else "fail"],
        "scope": profile["scope"],
        "source_gates": {
            "r125_status": reports.r125["status"],
            "r125_report_sha256": reports.r125["report_sha256"],
            "r122_status": reports.r122["status"],
            "r122_report_sha256": reports.r122["report_sha256"],
            "r121_status": reports.r121["status"],
            "r121_report_sha256": reports.r121["report_sha256"],
            "r113_status": reports.r113["status"],
            "r113_report_sha256": reports.r113["report_sha256"],
            "r120_status": reports.r120["status"],
            "r120_report_sha256": reports.r120["report_sha256"],
        },
        "numeric_contract": profile["numeric_contract"],
        "frozen_anchor_audits": anchor_audits,
        "synthetic_rank_audit": synthetic_rank,
        "synthetic_line_cone_audit": synthetic_cones,
        "resource_audit": resource_audit,
        "fixed_pd_schedule_sha256": _array_sha256(
            fixed_pd.applied_effort_micronewton_metres
        ),
        "validation_results": validations,
        "execution_environment": dict(execution_environment),
        "resource_usage": {
            "status": "PASS" if passed else "FAIL",
            "wall_clock_seconds": elapsed,
            "maximum_resident_memory_bytes": maximum_rss,
            "maximum_observed_os_thread_count": maximum_threads,
            "process_count": 1,
            "child_processes_spawned_during_conformance": 0,
            "randomized_restart_count": 0,
        },
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r125_report_file_sha256": sha256(r125_report_path),
            "r125_profile_sha256": sha256(r125_profile_path),
            "r125_module_sha256": sha256(r125_module_path),
            "r125_tool_sha256": sha256(r125_tool_path),
            "r122_report_file_sha256": sha256(r122_report_path),
            "r122_profile_sha256": sha256(r122_profile_path),
            "r121_report_file_sha256": sha256(r121_report_path),
            "r121_profile_sha256": sha256(r121_profile_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_cache_sha256": sha256(r120_cache_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "r122_kernel_sha256": sha256(r122_kernel_path),
            "r122_conformance_module_sha256": sha256(r122_conformance_module_path),
            "r122_tool_sha256": sha256(r122_tool_path),
            "gauge_aware_kernel_sha256": sha256(
                Path(build_reduced_local_system.__code__.co_filename).resolve()
            ),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "frozen_anchor_singular_value_decompositions": len(anchor_audits),
        "synthetic_singular_value_decompositions": synthetic_rank[
            "singular_value_decompositions"
        ],
        "real_schedule_particular_solutions": 0,
        "synthetic_particular_solutions": synthetic_rank["particular_solutions"],
        "real_schedule_gauge_interval_classifications": 0,
        "synthetic_gauge_interval_classifications": synthetic_cones[
            "gauge_interval_classifications"
        ],
        "r127_execution_runs": 0,
        "local_system_solves": 0,
        "inverse_dynamics_execution_runs": 0,
        "kinodynamic_solves": 0,
        "candidate_artifacts_built": 0,
        "solver_private_caches_built": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "bounded_acceptance": profile["bounded_acceptance"],
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_frozen_anchors(
    *,
    profile: Mapping[str, Any],
    model: Any,
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    effort_newton_metres: NDArray[np.float64],
) -> list[dict[str, Any]]:
    numeric = profile["numeric_contract"]
    rows = []
    for frozen in profile["frozen_anchors"]:
        collocation = int(frozen["collocation"])
        interval, substep = divmod(collocation, 4)
        state = collocation_state(cache, interval, substep)
        system = build_reduced_local_system(
            model=model,
            state=state,
            effort_newton_metres=effort_newton_metres[collocation],
            modes=contact_modes[interval],
            points=points,
        )
        scale = reduced_column_scale(
            model=model,
            cache=cache,
            active_point_count=len(system.active_point_ordinals),
        )
        analysis = analyse_reduced_system(
            system,
            column_scale=scale,
            numeric_contract=numeric,
            compute_particular=False,
        )
        expected_modes = frozen["contact_modes"]
        expected_active = frozen["active_point_ordinals"]
        expected_unknowns = int(frozen["reduced_local_unknown_count"])
        expected_nullity = int(frozen["expected_nullity"])
        wrench_pass = all(
            gauge.maximum_resultant_force
            <= float(numeric["physical_group_absolute_residual"])
            and gauge.maximum_resultant_moment
            <= float(numeric["physical_group_absolute_residual"])
            for gauge in system.gauge_identities
        )
        passed = bool(
            analysis.status == "VALID"
            and interval == int(frozen["interval"])
            and substep == int(frozen["substep"])
            and system.modes.tolist() == expected_modes
            and list(system.active_point_ordinals) == expected_active
            and system.matrix.shape == (expected_unknowns, expected_unknowns)
            and analysis.nullity == expected_nullity
            and analysis.rank == expected_unknowns - expected_nullity
            and analysis.particular_solution is None
            and wrench_pass
        )
        system_digest = hashlib.sha256()
        system_digest.update(np.ascontiguousarray(system.matrix).tobytes())
        system_digest.update(np.ascontiguousarray(system.right_hand_side).tobytes())
        rows.append(
            {
                "status": "PASS" if passed else "FAIL",
                "collocation": collocation,
                "interval": interval,
                "substep": substep,
                "contact_modes": system.modes.tolist(),
                "active_point_ordinals": list(system.active_point_ordinals),
                "matrix_shape": list(system.matrix.shape),
                "rank": analysis.rank,
                "nullity": analysis.nullity,
                "expected_nullity": analysis.expected_nullity,
                "smallest_retained_relative_singular_value": analysis.smallest_retained_relative_singular_value,
                "largest_null_relative_singular_value": analysis.largest_null_relative_singular_value,
                "analytic_gauge_scaled_residual": analysis.analytic_gauge_scaled_residual,
                "analytic_to_svd_null_projector_spectral_error": analysis.analytic_to_svd_projector_spectral_error,
                "maximum_resultant_gauge_force": max(
                    (
                        gauge.maximum_resultant_force
                        for gauge in system.gauge_identities
                    ),
                    default=0.0,
                ),
                "maximum_resultant_gauge_moment": max(
                    (
                        gauge.maximum_resultant_moment
                        for gauge in system.gauge_identities
                    ),
                    default=0.0,
                ),
                "particular_solution_computed": False,
                "gauge_interval_classified": False,
                "local_system_sha256": system_digest.hexdigest(),
                "invalid_reason": analysis.invalid_reason,
            }
        )
    return rows


def audit_synthetic_rank_cases(profile: Mapping[str, Any]) -> dict[str, Any]:
    numeric = profile["numeric_contract"]
    rows: list[dict[str, Any]] = []
    particular_count = 0

    flight_target = np.linspace(-0.4, 0.4, 29, dtype=np.float64)
    flight = _synthetic_system(
        diagonal=np.ones(29, dtype=np.float64),
        right_hand_side=flight_target,
        modes=(0, 0),
        active=(),
        gauges=np.empty((29, 0), dtype=np.float64),
    )
    flight_analysis = analyse_reduced_system(
        flight,
        column_scale=np.ones(29, dtype=np.float64),
        numeric_contract=numeric,
        compute_particular=True,
    )
    particular_count += 1
    rows.append(
        _rank_case_row(
            "flight_full_rank",
            flight_analysis,
            expected_status="VALID",
            expected_rank=29,
            expected_nullity=0,
            expected_solution=flight_target,
        )
    )

    single_diagonal = np.linspace(0.5, 1.0, 32, dtype=np.float64)
    single_target = np.linspace(-0.25, 0.25, 32, dtype=np.float64)
    single = _synthetic_system(
        diagonal=single_diagonal,
        right_hand_side=single_diagonal * single_target,
        modes=(0, 2),
        active=(3,),
        gauges=np.empty((32, 0), dtype=np.float64),
    )
    single_analysis = analyse_reduced_system(
        single,
        column_scale=np.ones(32, dtype=np.float64),
        numeric_contract=numeric,
        compute_particular=True,
    )
    particular_count += 1
    rows.append(
        _rank_case_row(
            "single_point_full_rank",
            single_analysis,
            expected_status="VALID",
            expected_rank=32,
            expected_nullity=0,
            expected_solution=single_target,
        )
    )

    flat_diagonal = np.ones(35, dtype=np.float64)
    flat_diagonal[-1] = 0.0
    flat_target = np.linspace(-0.3, 0.3, 35, dtype=np.float64)
    flat_target[-1] = 0.0
    flat_gauge = np.zeros((35, 1), dtype=np.float64)
    flat_gauge[-1, 0] = 1.0
    flat = _synthetic_system(
        diagonal=flat_diagonal,
        right_hand_side=flat_diagonal * flat_target,
        modes=(0, 3),
        active=(2, 3),
        gauges=flat_gauge,
    )
    flat_analysis = analyse_reduced_system(
        flat,
        column_scale=np.ones(35, dtype=np.float64),
        numeric_contract=numeric,
        compute_particular=True,
    )
    particular_count += 1
    rows.append(
        _rank_case_row(
            "flat_foot_known_one_null",
            flat_analysis,
            expected_status="VALID",
            expected_rank=34,
            expected_nullity=1,
            expected_solution=flat_target,
        )
    )

    extra_diagonal = flat_diagonal.copy()
    extra_diagonal[-2] = 0.0
    extra = _synthetic_system(
        diagonal=extra_diagonal,
        right_hand_side=np.zeros(35, dtype=np.float64),
        modes=(0, 3),
        active=(2, 3),
        gauges=flat_gauge,
    )
    extra_analysis = analyse_reduced_system(
        extra,
        column_scale=np.ones(35, dtype=np.float64),
        numeric_contract=numeric,
        compute_particular=False,
    )
    rows.append(
        _rank_case_row(
            "flat_foot_extra_nullity_rejected",
            extra_analysis,
            expected_status="INVALID",
            expected_rank=33,
            expected_nullity=2,
            expected_reason="UNEXPECTED_REDUCED_SYSTEM_NULLITY",
        )
    )

    ambiguous_diagonal = np.ones(32, dtype=np.float64)
    ambiguous_diagonal[-1] = 1.0e-11
    ambiguous = _synthetic_system(
        diagonal=ambiguous_diagonal,
        right_hand_side=np.zeros(32, dtype=np.float64),
        modes=(0, 2),
        active=(3,),
        gauges=np.empty((32, 0), dtype=np.float64),
    )
    ambiguous_analysis = analyse_reduced_system(
        ambiguous,
        column_scale=np.ones(32, dtype=np.float64),
        numeric_contract=numeric,
        compute_particular=False,
    )
    rows.append(
        _rank_case_row(
            "rank_gap_rejected",
            ambiguous_analysis,
            expected_status="INVALID",
            expected_rank=31,
            expected_nullity=0,
            expected_reason="SVD_RANK_AMBIGUITY_BAND_OCCUPIED",
        )
    )
    return {
        "status": "PASS" if all(row["status"] == "PASS" for row in rows) else "FAIL",
        "cases": rows,
        "singular_value_decompositions": len(rows),
        "particular_solutions": particular_count,
        "real_schedule_particular_solutions": 0,
    }


def audit_synthetic_line_cones(profile: Mapping[str, Any]) -> dict[str, Any]:
    numeric = profile["numeric_contract"]
    boundary = float(numeric["quadratic_boundary_relative_ambiguity"])
    cone_tolerance = float(numeric["cone_absolute_newtons"])
    cases = (
        (
            "interior_feasible",
            np.asarray(((10.0, 0.0, 0.0), (10.0, 0.0, 0.0))),
            True,
            0.0,
        ),
        (
            "gauge_recovers_infeasible_alpha_zero",
            np.asarray(((10.0, 9.0, 0.0), (10.0, -7.0, 0.0))),
            True,
            -8.0,
        ),
        (
            "disjoint_cones",
            np.asarray(((10.0, 9.0, 0.0), (10.0, 9.0, 0.0))),
            False,
            None,
        ),
    )
    direction = np.asarray((0.0, 1.0, 0.0), dtype=np.float64)
    rows = []
    for name, forces, expected_feasible, expected_witness in cases:
        actual = classify_flat_gauge_feasibility(
            forces,
            direction,
            boundary_relative_ambiguity=boundary,
            cone_absolute_newtons=cone_tolerance,
        )
        oracle = _decimal_flat_interval(forces)
        endpoint_error = _endpoint_error(actual, oracle)
        witness_error = (
            None
            if expected_witness is None or actual.witness_alpha is None
            else abs(actual.witness_alpha - expected_witness)
        )
        passed = bool(
            actual.status == "VALID"
            and actual.feasible is expected_feasible
            and endpoint_error <= float(profile["oracle_contract"]["absolute_error"])
            and (
                expected_witness is None
                or (
                    witness_error is not None
                    and witness_error
                    <= float(profile["oracle_contract"]["absolute_error"])
                )
            )
        )
        rows.append(
            {
                "status": "PASS" if passed else "FAIL",
                "case": name,
                "classification": ("FEASIBLE" if actual.feasible else "INFEASIBLE"),
                "interval": [actual.lower, actual.upper],
                "decimal_oracle_interval": oracle,
                "maximum_endpoint_absolute_error": endpoint_error,
                "witness_alpha": actual.witness_alpha,
                "expected_witness_alpha": expected_witness,
                "witness_absolute_error": witness_error,
                "minimum_normal_margin_newtons": actual.minimum_normal_margin_newtons,
                "minimum_friction_margin_newtons": actual.minimum_friction_margin_newtons,
                "alpha_zero_cone_feasible": _forces_in_cones(forces, cone_tolerance),
                "invalid_reason": actual.invalid_reason,
            }
        )

    tangent = line_cone_interval(
        np.asarray((1.0, FRICTION, 0.0), dtype=np.float64),
        np.asarray((0.0, 0.0, 1.0), dtype=np.float64),
        boundary_relative_ambiguity=boundary,
    )
    tangent_passed = bool(
        tangent.status == "INVALID"
        and tangent.invalid_reason == "QUADRATIC_BOUNDARY_AMBIGUITY"
        and _decimal_tangent_discriminant() == 0
    )
    rows.append(
        {
            "status": "PASS" if tangent_passed else "FAIL",
            "case": "exact_tangent_rejected_as_ambiguous",
            "classification": tangent.status,
            "decimal_oracle_discriminant": "0",
            "runtime_quadratic_coefficients": list(tangent.quadratic_coefficients),
            "invalid_reason": tangent.invalid_reason,
        }
    )
    return {
        "status": "PASS" if all(row["status"] == "PASS" for row in rows) else "FAIL",
        "oracle": {
            "implementation": "Python decimal closed-form one-dimensional oracle",
            "decimal_precision": int(profile["oracle_contract"]["decimal_precision"]),
            "friction_ratio": f"{FRICTION_NUMERATOR}/{FRICTION_DENOMINATOR}",
        },
        "cases": rows,
        "gauge_interval_classifications": 4,
        "real_schedule_gauge_interval_classifications": 0,
    }


def audit_maximum_payload(profile: Mapping[str, Any]) -> dict[str, Any]:
    unknowns = int(profile["resource_instrumentation"]["local_unknown_count"])
    matrix = np.empty((unknowns, unknowns), dtype=np.float64)
    right = np.empty(unknowns, dtype=np.float64)
    column_scale = np.empty(unknowns, dtype=np.float64)
    row_scale = np.empty(unknowns, dtype=np.float64)
    gauge = np.empty((unknowns, 1), dtype=np.float64)
    allocated = sum(
        value.nbytes for value in (matrix, right, column_scale, row_scale, gauge)
    )
    expected = int(profile["resource_instrumentation"]["expected_payload_bytes"])
    passed = bool(
        unknowns == 35
        and allocated == expected
        and profile["resource_instrumentation"]["particular_solution"] is False
        and profile["resource_instrumentation"]["gauge_classification"] is False
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "local_unknown_count": unknowns,
        "matrix_shape": list(matrix.shape),
        "analytic_gauge_shape": list(gauge.shape),
        "allocated_payload_bytes": allocated,
        "expected_payload_bytes": expected,
        "factorization_performed_in_resource_probe": False,
        "particular_solution_computed_in_resource_probe": False,
        "gauge_interval_classified_in_resource_probe": False,
    }


def _load_and_validate_sources(
    *,
    profile: Mapping[str, Any],
    r125_report_path: Path,
    r125_profile_path: Path,
    r125_module_path: Path,
    r125_tool_path: Path,
    r122_report_path: Path,
    r122_profile_path: Path,
    r122_kernel_path: Path,
    r122_conformance_module_path: Path,
    r122_tool_path: Path,
    r121_report_path: Path,
    r121_profile_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r120_report_path: Path,
    r120_cache_path: Path,
    v9_complete_clip_path: Path,
) -> SourceReports:
    source = profile["source"]
    r125 = _load_bound_report(r125_report_path, source["r125"], "R125")
    r122 = _load_bound_report(r122_report_path, source["r122"], "R122")
    r121 = _load_bound_report(r121_report_path, source["r121"], "R121")
    r113 = _load_bound_report(r113_report_path, source["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, source["r120"], "R120")
    file_identities = (
        (r125_profile_path, source["r125"]["profile_sha256"]),
        (r125_module_path, source["r125"]["module_sha256"]),
        (r125_tool_path, source["r125"]["tool_sha256"]),
        (r122_profile_path, source["r122"]["profile_sha256"]),
        (r122_kernel_path, source["r122"]["kernel_sha256"]),
        (
            r122_conformance_module_path,
            source["r122"]["conformance_module_sha256"],
        ),
        (r122_tool_path, source["r122"]["tool_sha256"]),
        (r121_profile_path, source["r121"]["profile_sha256"]),
        (r113_profile_path, source["r113"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (v9_complete_clip_path, source["v9_complete_clip_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in file_identities):
        raise ValueError("R126 bound source file identity differs")
    if (
        r125.get("status") != "COMPLETE"
        or r125.get("gate_decision")
        != "PERMIT_R126_REDUNDANT_CONTACT_IMPLEMENTATION_CONFORMANCE_ONLY"
        or r125.get("repository", {}).get("commit")
        != source["r125"]["repository_commit"]
        or r125.get("repository", {}).get("dirty") is not False
        or r125.get("local_system_reconstructions") != 0
        or r125.get("singular_value_decompositions") != 0
        or r125.get("local_system_solves") != 0
        or r122.get("status") != "PASS"
        or r122.get("gate_decision")
        != "PERMIT_R123_SINGLE_BOUNDED_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or r122.get("repository", {}).get("commit")
        != source["r122"]["repository_commit"]
        or r121.get("status") != "COMPLETE"
        or r121.get("report_sha256")
        != r122.get("source_gates", {}).get("r121_report_sha256")
        or r113.get("status") != "PASS"
        or r113.get("report_sha256")
        != r122.get("source_gates", {}).get("r113_report_sha256")
        or r120.get("status") != "PASS"
        or r120.get("report_sha256")
        != r122.get("source_gates", {}).get("r120_report_sha256")
        or r122.get("r123_local_system_solves") != 0
        or r122.get("inverse_dynamics_execution_runs") != 0
    ):
        raise ValueError("R126 bound source report contract differs")
    return SourceReports(r125, r122, r121, r113, r120)


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"R126 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"R126 {label} canonical report identity differs")
    return report


def _synthetic_system(
    *,
    diagonal: NDArray[np.float64],
    right_hand_side: NDArray[np.float64],
    modes: tuple[int, int],
    active: tuple[int, ...],
    gauges: NDArray[np.float64],
) -> ReducedLocalSystem:
    return ReducedLocalSystem(
        matrix=np.diag(diagonal),
        right_hand_side=right_hand_side,
        active_point_ordinals=active,
        modes=np.asarray(modes, dtype=np.uint8),
        analytic_gauge_matrix=gauges,
        gauge_identities=(),
    )


def _rank_case_row(
    name: str,
    analysis: Any,
    *,
    expected_status: str,
    expected_rank: int,
    expected_nullity: int,
    expected_solution: NDArray[np.float64] | None = None,
    expected_reason: str | None = None,
) -> dict[str, Any]:
    solution_error = None
    if expected_solution is not None and analysis.particular_solution is not None:
        solution_error = float(
            np.max(np.abs(analysis.particular_solution - expected_solution))
        )
    passed = bool(
        analysis.status == expected_status
        and analysis.rank == expected_rank
        and analysis.nullity == expected_nullity
        and analysis.invalid_reason == expected_reason
        and (
            expected_solution is None
            or (solution_error is not None and solution_error <= 1.0e-12)
        )
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "case": name,
        "analysis_status": analysis.status,
        "rank": analysis.rank,
        "nullity": analysis.nullity,
        "expected_nullity": analysis.expected_nullity,
        "smallest_retained_relative_singular_value": analysis.smallest_retained_relative_singular_value,
        "largest_null_relative_singular_value": analysis.largest_null_relative_singular_value,
        "analytic_gauge_scaled_residual": analysis.analytic_gauge_scaled_residual,
        "analytic_to_svd_null_projector_spectral_error": analysis.analytic_to_svd_projector_spectral_error,
        "particular_solution_maximum_absolute_error": solution_error,
        "invalid_reason": analysis.invalid_reason,
    }


def _decimal_flat_interval(forces: NDArray[np.float64]) -> list[float | None]:
    getcontext().prec = 80
    mu = Decimal(FRICTION_NUMERATOR) / Decimal(FRICTION_DENOMINATOR)
    intervals = []
    for point, direction in ((0, Decimal(1)), (1, Decimal(-1))):
        normal = Decimal(str(float(forces[point, 0])))
        right = Decimal(str(float(forces[point, 1])))
        bound = mu * normal
        roots = ((-bound - right) / direction, (bound - right) / direction)
        intervals.append((min(roots), max(roots)))
    lower = max(interval[0] for interval in intervals)
    upper = min(interval[1] for interval in intervals)
    if lower > upper:
        return [None, None]
    return [float(lower), float(upper)]


def _decimal_tangent_discriminant() -> Decimal:
    getcontext().prec = 80
    numerator = Decimal(FRICTION_NUMERATOR)
    denominator = Decimal(FRICTION_DENOMINATOR)
    normal = denominator
    right = numerator
    mu2 = (numerator / denominator) ** 2
    quadratic = Decimal(1)
    linear = Decimal(0)
    constant = right * right - mu2 * normal * normal
    return linear * linear - Decimal(4) * quadratic * constant


def _endpoint_error(actual: Any, oracle: list[float | None]) -> float:
    if oracle == [None, None]:
        return 0.0 if actual.feasible is False else math.inf
    if actual.lower is None or actual.upper is None:
        return math.inf
    assert oracle[0] is not None and oracle[1] is not None
    return max(abs(actual.lower - oracle[0]), abs(actual.upper - oracle[1]))


def _forces_in_cones(forces: NDArray[np.float64], tolerance: float) -> bool:
    return bool(
        np.all(forces[:, 0] >= -tolerance)
        and np.all(
            FRICTION * forces[:, 0] - np.hypot(forces[:, 1], forces[:, 2]) >= -tolerance
        )
    )


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    numeric = profile.get("numeric_contract", {})
    budget = profile.get("resource_budget", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    anchors = profile.get("frozen_anchors", [])
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnly"
        or scope.get("run_id") != "R126"
        or scope.get("frozen_anchor_count") != 7
        or scope.get("real_schedule_particular_solutions") != 0
        or scope.get("real_schedule_gauge_classifications") != 0
        or scope.get("r127_execution") is not False
        or scope.get("training") is not False
        or len(anchors) != 7
        or [row.get("collocation") for row in anchors]
        != [0, 952, 976, 996, 1312, 2504, 3199]
        or len(source.get("current_descriptor_file_sha256", "")) != 64
        or len(source.get("gauge_aware_kernel_sha256", "")) != 64
        or len(source.get("conformance_module_sha256", "")) != 64
        or len(source.get("tool_sha256", "")) != 64
        or numeric.get("svd_null_relative_maximum") != 1.0e-12
        or numeric.get("svd_retained_relative_minimum") != 1.0e-10
        or numeric.get("analytic_gauge_scaled_residual") != 1.0e-10
        or numeric.get("analytic_to_svd_null_projector_spectral_error") != 1.0e-8
        or numeric.get("scaled_absolute_residual") != 1.0e-9
        or numeric.get("backward_error") != 1.0e-10
        or numeric.get("physical_group_absolute_residual") != 1.0e-7
        or numeric.get("cone_absolute_newtons") != 1.0e-7
        or numeric.get("quadratic_boundary_relative_ambiguity") != 1.0e-12
        or budget.get("process_count") != 1
        or budget.get("thread_count") != 1
        or budget.get("maximum_wall_clock_seconds") != 600
        or budget.get("maximum_resident_memory_bytes") != 8 * 1024**3
        or decision.get("pass")
        != "PERMIT_R127_SINGLE_BOUNDED_GAUGE_AWARE_FIXED_PD_EXECUTION_ONLY"
        or decision.get("fail") != "STOP_INVALID_REDUNDANT_CONTACT_IMPLEMENTATION"
        or bounded.get("r127_execution") != "AUTHORIZED_ON_EXACT_R126_PASS_ONLY"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "r123_retry",
                "additional_inverse_dynamics_execution",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != (
            "ruff_check",
            "ruff_format",
            "single_thread_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("R126 conformance profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R126 conformance requires a clean repository")


def _validate_execution_environment(
    profile: Mapping[str, Any], actual: Mapping[str, str]
) -> None:
    if (
        dict(actual) != profile["single_thread_environment"]
        or _linux_thread_count() != 1
    ):
        raise ValueError("R126 single-thread execution environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R126 validation results differ")
    return normalized


def _array_sha256(array: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(array).tobytes()).hexdigest()


def _resident_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _linux_thread_count() -> int:
    status = Path("/proc/self/status").read_text(encoding="utf-8")
    for line in status.splitlines():
        if line.startswith("Threads:"):
            return int(line.split(":", 1)[1].strip())
    raise ValueError("R126 cannot read Linux thread count")
