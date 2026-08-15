from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.fixed_pd_inverse_dynamics_conformance import (
    _contact_points,
    configuration_at,
    load_r120_cache,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    build_spatial_model,
    forward_kinematics,
    point_acceleration,
    propagate_motion,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

RESEARCH_ID = "nextengine.humanoid-post-r127-contact-state-consistency-research.v1"
CHECK_ID = "TRAIN-4-POST-R127-CONTACT-STATE-CONSISTENCY-RESEARCH"
GENERALIZED_WIDTH = 29
NOT_AUTHORIZED_KEYS = frozenset(
    {
        "r127_retry",
        "additional_inverse_dynamics_execution",
        "contact_state_consistency_execution",
        "kinodynamic_solve",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
)


def build_contact_state_consistency_research(
    *,
    profile_path: Path,
    r127_report_path: Path,
    r127_profile_path: Path,
    r127_execution_module_path: Path,
    r127_tool_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    dynamics_kernel_path: Path,
    conformance_module_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Hash-close the R127 RHS failure with one kinematic identity audit."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r127_report_path,
            r127_profile_path,
            r127_execution_module_path,
            r127_tool_path,
            r113_report_path,
            r113_profile_path,
            r120_report_path,
            r120_profile_path,
            r120_cache_path,
            dynamics_kernel_path,
            conformance_module_path,
            tool_path,
        )
    )
    (
        profile_path,
        r127_report_path,
        r127_profile_path,
        r127_execution_module_path,
        r127_tool_path,
        r113_report_path,
        r113_profile_path,
        r120_report_path,
        r120_profile_path,
        r120_cache_path,
        dynamics_kernel_path,
        conformance_module_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("post-R127 research input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r127 = _load_bound_report(r127_report_path, profile["source"]["r127"], "R127")
    r113 = _load_bound_report(r113_report_path, profile["source"]["r113"], "R113")
    r120 = _load_bound_report(r120_report_path, profile["source"]["r120"], "R120")
    _validate_sources(
        profile=profile,
        r127=r127,
        r127_profile_path=r127_profile_path,
        r127_execution_module_path=r127_execution_module_path,
        r127_tool_path=r127_tool_path,
        r113=r113,
        r113_profile_path=r113_profile_path,
        r120=r120,
        r120_profile_path=r120_profile_path,
        r120_cache_path=r120_cache_path,
        dynamics_kernel_path=dynamics_kernel_path,
        conformance_module_path=conformance_module_path,
    )
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["current_descriptor_file_sha256"]
        or sha256(Path(__file__).resolve())
        != profile["source"]["research_module_sha256"]
        or sha256(tool_path) != profile["source"]["tool_sha256"]
    ):
        raise ValueError("post-R127 current research identity differs")
    validations = _validate_results(profile, validation_results)

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    configuration = configuration_at(cache, 0)
    velocity = np.asarray(cache["velocity"][0], dtype=np.float64)
    model = build_spatial_model(descriptor)
    points = _contact_points(descriptor=descriptor, r113=r113)
    heel = points[2]
    forefoot = points[3]
    body_slot = int(heel["body_slot"])
    if body_slot != int(forefoot["body_slot"]):
        raise ValueError("post-R127 active points do not share one body")
    heel_local = np.asarray(heel["local_translation_metres"], dtype=np.float64)
    forefoot_local = np.asarray(forefoot["local_translation_metres"], dtype=np.float64)
    kinematics = forward_kinematics(model, configuration)
    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    motion = propagate_motion(model, kinematics, velocity, zero)
    heel_jdot_v = point_acceleration(kinematics, motion, body_slot, heel_local)
    forefoot_jdot_v = point_acceleration(kinematics, motion, body_slot, forefoot_local)
    rotation = kinematics.body_rotations[body_slot]
    heel_velocity = _point_velocity(
        body_slot=body_slot,
        local_point=heel_local,
        body_rotation=rotation,
        linear_velocity=motion.linear_velocity,
        angular_velocity=motion.angular_velocity,
    )
    forefoot_velocity = _point_velocity(
        body_slot=body_slot,
        local_point=forefoot_local,
        body_rotation=rotation,
        linear_velocity=motion.linear_velocity,
        angular_velocity=motion.angular_velocity,
    )
    audit = audit_rigid_line_compatibility(
        heel_local=heel_local,
        forefoot_local=forefoot_local,
        body_rotation=rotation,
        body_angular_velocity=motion.angular_velocity[body_slot],
        heel_jdot_v=heel_jdot_v,
        forefoot_jdot_v=forefoot_jdot_v,
        heel_velocity=heel_velocity,
        forefoot_velocity=forefoot_velocity,
    )
    first = r127["solver_result"]["collocations"][0]
    tolerance = float(profile["discriminators"]["identity_absolute_tolerance"])
    expected = profile["discriminators"]["expected_kinematics"]
    discriminators = {
        "r127_is_clean_single_consumed_invalid_execution": True,
        "first_failure_has_expected_rank_and_nullity": (
            first["rank"] == 34
            and first["nullity"] == 1
            and first["expected_nullity"] == 1
        ),
        "first_failure_is_rhs_residual_not_rank_or_gauge": (
            first["invalid_reason"] == "SCALED_EQUALITY_RESIDUAL_EXCEEDED"
            and first["particular_scaled_absolute_residual"]
            > profile["discriminators"]["scaled_residual_limit"]
            and first["analytic_gauge_scaled_residual"]
            <= profile["discriminators"]["analytic_gauge_residual_limit"]
            and first["analytic_to_svd_null_projector_spectral_error"]
            <= profile["discriminators"]["projector_error_limit"]
        ),
        "same_body_line_is_nonzero": (
            heel["body_id"] == forefoot["body_id"] and audit["separation_metres"] > 0.0
        ),
        "both_flat_points_have_nonzero_world_velocity": (
            audit["heel_speed_metres_per_second"]
            > profile["discriminators"]["minimum_point_speed_metres_per_second"]
            and audit["forefoot_speed_metres_per_second"]
            > profile["discriminators"]["minimum_point_speed_metres_per_second"]
        ),
        "foot_rotation_has_nonzero_component_perpendicular_to_line": (
            audit["perpendicular_angular_speed_radians_per_second"]
            > profile["discriminators"][
                "minimum_perpendicular_angular_speed_radians_per_second"
            ]
        ),
        "observed_line_compatibility_is_nonzero": (
            abs(audit["observed_line_compatibility_metres_per_second_squared"])
            > profile["discriminators"][
                "minimum_incompatibility_metres_per_second_squared"
            ]
        ),
        "centripetal_identity_matches": (audit["identity_absolute_error"] <= tolerance),
        "frozen_kinematics_match_predeclared_values": all(
            abs(float(audit[key]) - float(value)) <= tolerance
            for key, value in expected.items()
        ),
        "research_assembles_or_solves_no_dynamics_system": True,
    }
    confirmed = all(discriminators.values())
    finding = (
        "CONFIRMED_FLAT_STICKING_STATE_ACCELERATION_INCOMPATIBILITY"
        if confirmed
        else "INCONCLUSIVE_RETAIN_STOP_WITHOUT_EXECUTION"
    )
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "research_id": RESEARCH_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "finding": finding,
        "gate_decision": profile["decision"][
            "confirmed" if confirmed else "inconclusive"
        ],
        "scope": profile["scope"],
        "r127_result": {
            "status": r127["status"],
            "gate_decision": r127["gate_decision"],
            "report_sha256": r127["report_sha256"],
            "invalid_reason": r127["solver_result"]["invalid_reason"],
            "first_collocation": first,
            "singular_value_decompositions": r127["singular_value_decompositions"],
            "particular_solutions": r127["particular_solutions"],
            "gauge_interval_classifications": r127["gauge_interval_classifications"],
        },
        "kinematic_compatibility_audit": audit,
        "discriminators": discriminators,
        "hypothesis_disposition": profile["hypothesis_disposition"],
        "research_basis": profile["research_basis"],
        "repair_alternatives": profile["repair_alternatives"],
        "interpretation": {
            "supported": "the frozen right-foot q/v state is off the two-point FlatSticking acceleration manifold because rigid centripetal relative acceleration makes the redundant closure RHS incompatible",
            "not_supported": "R127 provides no pointwise cone-feasibility result and does not show a remaining force-nullspace implementation defect",
            "required_successor_scope": "separately formulate q/v contact-manifold projection, mode correction, or an explicit discrete/compliant contact model before any new execution",
        },
        "next_smallest_action": profile["next_smallest_action"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r127_report_file_sha256": sha256(r127_report_path),
            "r127_profile_sha256": sha256(r127_profile_path),
            "r127_execution_module_sha256": sha256(r127_execution_module_path),
            "r127_tool_sha256": sha256(r127_tool_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "r120_report_file_sha256": sha256(r120_report_path),
            "r120_profile_sha256": sha256(r120_profile_path),
            "r120_cache_sha256": sha256(r120_cache_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "dynamics_kernel_sha256": sha256(dynamics_kernel_path),
            "conformance_module_sha256": sha256(conformance_module_path),
            "research_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "research_audits": 1,
        "kinematic_state_audits": 1,
        "contact_jacobian_assemblies": 0,
        "local_system_reconstructions": 0,
        "singular_value_decompositions": 0,
        "particular_solutions": 0,
        "gauge_interval_classifications": 0,
        "local_system_solves": 0,
        "solver_runs": 0,
        "additional_inverse_dynamics_execution_runs": 0,
        "inverse_dynamics_evaluations": 0,
        "kinodynamic_solves": 0,
        "candidate_artifacts_built": 0,
        "solver_private_caches_built": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_rigid_line_compatibility(
    *,
    heel_local: NDArray[np.float64],
    forefoot_local: NDArray[np.float64],
    body_rotation: NDArray[np.float64],
    body_angular_velocity: NDArray[np.float64],
    heel_jdot_v: NDArray[np.float64],
    forefoot_jdot_v: NDArray[np.float64],
    heel_velocity: NDArray[np.float64],
    forefoot_velocity: NDArray[np.float64],
) -> dict[str, Any]:
    values = (
        heel_local,
        forefoot_local,
        body_rotation,
        body_angular_velocity,
        heel_jdot_v,
        forefoot_jdot_v,
        heel_velocity,
        forefoot_velocity,
    )
    if (
        heel_local.shape != (3,)
        or forefoot_local.shape != (3,)
        or body_rotation.shape != (3, 3)
        or any(value.shape != (3,) for value in values[3:])
        or any(not np.all(np.isfinite(value)) for value in values)
    ):
        raise ValueError("post-R127 rigid-line audit input differs")
    world_separation = body_rotation @ (forefoot_local - heel_local)
    length = float(np.linalg.norm(world_separation))
    if length <= 0.0:
        raise ValueError("post-R127 rigid-line separation is zero")
    direction = world_separation / length
    perpendicular = np.cross(body_angular_velocity, direction)
    perpendicular_speed = float(np.linalg.norm(perpendicular))
    observed = float(direction @ (forefoot_jdot_v - heel_jdot_v))
    predicted = -length * perpendicular_speed * perpendicular_speed
    return {
        "separation_metres": length,
        "world_line_direction": direction.tolist(),
        "body_angular_velocity_radians_per_second": body_angular_velocity.tolist(),
        "perpendicular_angular_speed_radians_per_second": perpendicular_speed,
        "heel_jdot_v_metres_per_second_squared": heel_jdot_v.tolist(),
        "forefoot_jdot_v_metres_per_second_squared": forefoot_jdot_v.tolist(),
        "observed_line_compatibility_metres_per_second_squared": observed,
        "centripetal_identity_prediction_metres_per_second_squared": predicted,
        "identity_absolute_error": abs(observed - predicted),
        "heel_world_velocity_metres_per_second": heel_velocity.tolist(),
        "forefoot_world_velocity_metres_per_second": forefoot_velocity.tolist(),
        "heel_speed_metres_per_second": float(np.linalg.norm(heel_velocity)),
        "forefoot_speed_metres_per_second": float(np.linalg.norm(forefoot_velocity)),
        "two_zero_point_accelerations_are_compatible": observed == 0.0,
    }


def _point_velocity(
    *,
    body_slot: int,
    local_point: NDArray[np.float64],
    body_rotation: NDArray[np.float64],
    linear_velocity: NDArray[np.float64],
    angular_velocity: NDArray[np.float64],
) -> NDArray[np.float64]:
    return linear_velocity[body_slot] + np.cross(
        angular_velocity[body_slot], body_rotation @ local_point
    )


def _validate_sources(
    *,
    profile: Mapping[str, Any],
    r127: Mapping[str, Any],
    r127_profile_path: Path,
    r127_execution_module_path: Path,
    r127_tool_path: Path,
    r113: Mapping[str, Any],
    r113_profile_path: Path,
    r120: Mapping[str, Any],
    r120_profile_path: Path,
    r120_cache_path: Path,
    dynamics_kernel_path: Path,
    conformance_module_path: Path,
) -> None:
    source = profile["source"]
    identities = (
        (r127_profile_path, source["r127"]["profile_sha256"]),
        (r127_execution_module_path, source["r127"]["execution_module_sha256"]),
        (r127_tool_path, source["r127"]["tool_sha256"]),
        (r113_profile_path, source["r113"]["profile_sha256"]),
        (r120_profile_path, source["r120"]["profile_sha256"]),
        (r120_cache_path, source["r120"]["cache_sha256"]),
        (dynamics_kernel_path, source["dynamics_kernel_sha256"]),
        (conformance_module_path, source["conformance_module_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in identities):
        raise ValueError("post-R127 bound source file identity differs")
    rows = r127.get("solver_result", {}).get("collocations", [])
    first = rows[0] if len(rows) == 1 else {}
    r127_identities = r127.get("identities", {})
    r113_identities = r113.get("identities", {})
    r120_identities = r120.get("identities", {})
    if (
        r127.get("status") != "INVALID"
        or r127.get("gate_decision") != "STOP_INVALID_EVIDENCE_WITHOUT_RESTART"
        or r127.get("result_transition") != "R127_CONSUMED_INVALID_NO_RETRY"
        or r127.get("repository", {}).get("commit")
        != source["r127"]["repository_commit"]
        or r127.get("repository", {}).get("dirty") is not False
        or r127.get("singular_value_decompositions") != 1
        or r127.get("particular_solutions") != 1
        or r127.get("gauge_interval_classifications") != 0
        or r127.get("local_system_solves") != 0
        or first.get("collocation") != 0
        or first.get("contact_modes") != [0, 3]
        or first.get("active_point_ordinals") != [2, 3]
        or first.get("invalid_reason") != "SCALED_EQUALITY_RESIDUAL_EXCEEDED"
        or r127.get("solver_private_cache", {}).get("status")
        != "NOT_EMITTED_INVALID_EXECUTION"
        or r127.get("bounded_acceptance", {}).get("r127_retry") != "NOT_AUTHORIZED"
        or any(row.get("status") != "PASS" for row in r127["validation_results"])
        or r127_identities.get("profile_sha256") != source["r127"]["profile_sha256"]
        or r127_identities.get("execution_module_sha256")
        != source["r127"]["execution_module_sha256"]
        or r127_identities.get("tool_sha256") != source["r127"]["tool_sha256"]
        or r127_identities.get("r113_report_file_sha256")
        != source["r113"]["report_file_sha256"]
        or r127_identities.get("r120_report_file_sha256")
        != source["r120"]["report_file_sha256"]
        or r127_identities.get("r120_cache_sha256") != source["r120"]["cache_sha256"]
        or r127_identities.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
        or r113.get("status") != "PASS"
        or r113_identities.get("profile_sha256") != source["r113"]["profile_sha256"]
        or r113_identities.get("mirror_v2_sha256")
        != source["current_descriptor_file_sha256"]
        or r120.get("status") != "PASS"
        or r120_identities.get("execution_profile_sha256")
        != source["r120"]["profile_sha256"]
        or r120_identities.get("current_descriptor_file_sha256")
        != source["current_descriptor_file_sha256"]
    ):
        raise ValueError("post-R127 source report contract differs")


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError(f"post-R127 {label} report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError(f"post-R127 {label} canonical report identity differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    method = profile.get("method", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("research_id") != RESEARCH_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "PostR127FlatStickingStateConsistencyAuditOnly"
        or scope.get("research_cycle_id") != "R127-RC1"
        or scope.get("source_r127_execution_runs") != 1
        or scope.get("research_audits") != 1
        or scope.get("kinematic_state_audits") != 1
        or scope.get("contact_jacobian_assemblies") != 0
        or scope.get("local_system_reconstructions") != 0
        or scope.get("singular_value_decompositions") != 0
        or scope.get("particular_solutions") != 0
        or scope.get("gauge_interval_classifications") != 0
        or scope.get("local_system_solves") != 0
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or method.get("matrix_factorization_or_solve_policy") != "FORBIDDEN"
        or profile.get("decision", {}).get("confirmed")
        != "PERMIT_SEPARATE_REPORT_ONLY_CONTACT_STATE_CONSISTENCY_FORMULATION_ONLY"
        or profile.get("decision", {}).get("inconclusive")
        != "STOP_AND_RESEARCH_WITHOUT_EXECUTION"
        or frozenset(bounded) != NOT_AUTHORIZED_KEYS
        or any(value != "NOT_AUTHORIZED" for value in bounded.values())
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != (
            "ruff_check",
            "ruff_format",
            "no_solver_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("post-R127 research profile differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("post-R127 research requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("post-R127 research validation differs")
    return normalized
