from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    CACHE_KEYS,
    COLLOCATION_COUNT,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
    MOTOR_INTERVAL_COUNT,
    SUBSTEPS_PER_INTERVAL,
    affine_sample,
    audit_contact_schedule,
    audit_fixed_pd_schedule,
    audit_system_inventory,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    Configuration,
    SpatialModel,
    build_spatial_model,
    forward_kinematics,
    generalized_contact_force,
    integrate_configuration,
    inverse_dynamics,
    inverse_dynamics_mass_matrix,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    point_position,
    propagate_motion,
    rotation_exp,
)
from next_lab.motion_math import (
    decode_q1_30,
    matrix_to_quaternion,
    quaternion_to_matrix,
    target_forward_kinematics,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

CONFORMANCE_ID = (
    "nextengine.humanoid-fixed-pd-inverse-dynamics-implementation-conformance.v1"
)
CHECK_ID = "TRAIN-4-FIXED-PD-INVERSE-DYNAMICS-IMPLEMENTATION-CONFORMANCE"
ANCHORS = (0, 238, 244, 249, 328, 626, 800)
CONTACT_IDS = (
    "effector.left-heel",
    "effector.left-forefoot",
    "effector.right-heel",
    "effector.right-forefoot",
)
ZERO_EXECUTION_COUNTERS = (
    "r123_local_system_solves",
    "inverse_dynamics_execution_runs",
    "kinodynamic_solves",
    "candidate_artifacts_built",
    "solver_private_warm_start_caches",
    "physx_scene_runs",
    "optimizer_steps",
    "training_runs",
)


def build_fixed_pd_inverse_dynamics_conformance(
    *,
    profile_path: Path,
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
    solver_import_audit: Mapping[str, Any],
) -> dict[str, Any]:
    """Run R122 numerical conformance without constructing an R123 system."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
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
        raise FileNotFoundError("R122 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_solver_import_audit(solver_import_audit)
    validations = _validate_results(profile, validation_results)
    r121 = _load_bound_report(
        r121_report_path, r121_profile_path, profile["source"]["r121"]
    )
    r113 = _load_bound_report(
        r113_report_path, r113_profile_path, profile["source"]["r113"]
    )
    r120 = _load_r120(r120_report_path, profile["source"]["r120"])
    _validate_source_reports(profile=profile, r121=r121, r113=r113, r120=r120)
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["current_descriptor_file_sha256"]
    ):
        raise ValueError("R122 descriptor identity differs")
    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    model = build_spatial_model(descriptor)
    model_audit = audit_spatial_model_identity(model=model, descriptor=descriptor)
    cache = load_r120_cache(r120_cache_path, profile)
    with np.load(v9_complete_clip_path, allow_pickle=False) as archive:
        contact_modes = np.array(archive["contact_modes"], copy=True)
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ValueError("R122 V9 contact modes differ")
    points = _contact_points(descriptor=descriptor, r113=r113)
    anchor_audits = [
        audit_anchor(
            frame=frame,
            model=model,
            descriptor=descriptor,
            cache=cache,
            contact_modes=contact_modes,
            points=points,
            profile=profile,
        )
        for frame in ANCHORS
    ]
    point_force_audit = audit_point_force_projection(
        model=model,
        descriptor=descriptor,
        cache=cache,
        points=points,
        r113=r113,
    )
    r121_reproduction = audit_r121_reproduction(
        profile=profile,
        descriptor=descriptor,
        cache=cache,
        r121=r121,
        r113=r113,
        r120=r120,
        v9_complete_clip_path=v9_complete_clip_path,
    )
    resource_audit = audit_one_local_system_resource(profile)
    discriminators = {
        "descriptor_to_spatial_model_identity": model_audit["status"] == "PASS",
        "seven_anchor_closure": tuple(row["frame"] for row in anchor_audits) == ANCHORS,
        "mass_matrix_symmetry_and_spd": all(
            row["mass_matrix"]["status"] == "PASS" for row in anchor_audits
        ),
        "inverse_forward_dynamics_round_trip": all(
            row["dynamics_round_trip"]["status"] == "PASS" for row in anchor_audits
        ),
        "contact_position_and_jacobian": all(
            row["contact_kinematics"]["status"] == "PASS" for row in anchor_audits
        ),
        "active_contact_jdot_v": all(
            row["jdot_v"]["status"] == "PASS" for row in anchor_audits
        ),
        "r113_point_force_projection": point_force_audit["status"] == "PASS",
        "r121_schedule_reproduction": r121_reproduction["status"] == "PASS",
        "one_local_system_instrumented_without_execution": resource_audit["status"]
        == "PASS",
        "solver_free_process": all(
            value is False for value in solver_import_audit.values()
        ),
        "zero_r123_execution_counters": True,
    }
    passed = all(discriminators.values())
    status = "PASS" if passed else "FAIL"
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": status,
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["pass" if passed else "fail"],
        "scope": profile["scope"],
        "source_gates": {
            "r121_status": r121["status"],
            "r121_report_sha256": r121["report_sha256"],
            "r113_status": r113["status"],
            "r113_report_sha256": r113["report_sha256"],
            "r120_status": r120["status"],
            "r120_report_sha256": r120["report_sha256"],
        },
        "kernel_contract": profile["kernel_contract"],
        "spatial_model_identity_audit": model_audit,
        "anchor_audits": anchor_audits,
        "point_force_projection_audit": point_force_audit,
        "r121_reproduction_audit": r121_reproduction,
        "resource_instrumentation_audit": resource_audit,
        "discriminators": discriminators,
        "numeric_tolerances": profile["numeric_tolerances"],
        "next_smallest_action": profile["next_smallest_action"][
            "pass" if passed else "fail"
        ],
        "validation_results": validations,
        "solver_import_audit": dict(solver_import_audit),
        "identities": {
            "profile_sha256": sha256(profile_path),
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
            "motion_math_module_sha256": sha256(
                Path(target_forward_kinematics.__code__.co_filename).resolve()
            ),
            "kernel_module_sha256": sha256(
                Path(build_spatial_model.__code__.co_filename).resolve()
            ),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "conformance_reports": 1,
        "conformance_forward_dynamics_probes": len(ANCHORS),
        "inverse_dynamics_evaluations": len(ANCHORS) * (GENERALIZED_WIDTH + 3),
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_spatial_model_identity(
    *, model: SpatialModel, descriptor: Mapping[str, Any]
) -> dict[str, Any]:
    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    joint_axes = np.stack([joint.axis for joint in model.joints_by_dof])
    axis_norm_error = float(np.max(np.abs(np.linalg.norm(joint_axes, axis=1) - 1.0)))
    total_mass = float(np.sum(model.masses))
    inertia_pass = all(row["status"] == "PASS" for row in model.inertia_identity)
    hierarchy_pass = True
    frame_axis_pass = True
    for dof, joint in enumerate(model.joints_by_dof):
        source = joints[dof]
        hierarchy_pass &= bool(
            joint.dof == dof
            and joint.joint_id == source["joint_id"]
            and joint.parent == int(source["parent_body_slot"])
            and joint.child == int(source["child_body_slot"])
        )
        source_axis = decode_q1_30(source["axis_q1_30"])
        source_axis /= np.linalg.norm(source_axis)
        frame_axis_pass &= bool(
            np.array_equal(
                joint.parent_translation,
                np.asarray(
                    source["parent_frame"]["translation_micrometres"],
                    dtype=np.float64,
                )
                / 1_000_000.0,
            )
            and np.array_equal(
                joint.child_translation,
                np.asarray(
                    source["child_frame"]["translation_micrometres"],
                    dtype=np.float64,
                )
                / 1_000_000.0,
            )
            and np.array_equal(
                joint.parent_rotation,
                quaternion_to_matrix(
                    decode_q1_30(source["parent_frame"]["rotation_q1_30"])
                ),
            )
            and np.array_equal(
                joint.child_rotation,
                quaternion_to_matrix(
                    decode_q1_30(source["child_frame"]["rotation_q1_30"])
                ),
            )
            and np.array_equal(joint.axis, source_axis)
        )
    gravity_pass = bool(np.array_equal(model.gravity, np.asarray((0.0, -9.81, 0.0))))
    passed = bool(
        len(model.body_ids) == 24
        and len(model.joints_by_dof) == 23
        and abs(total_mass - 75.337) <= 1.0e-12
        and inertia_pass
        and hierarchy_pass
        and frame_axis_pass
        and axis_norm_error <= 2.0e-9
        and gravity_pass
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "body_count": len(model.body_ids),
        "joint_count": len(model.joints_by_dof),
        "total_mass_kilograms": total_mass,
        "computational_inertia": (
            "descriptor solver principal inertia reconstructed through its "
            "principal frame"
        ),
        "authoritative_tensor_role": (
            "identity source bounded by each declared solver projection error"
        ),
        "maximum_actual_inertia_projection_error_microkilogram_metre_squared": max(
            row["maximum_projection_error_microkilogram_metre_squared"]
            for row in model.inertia_identity
        ),
        "maximum_declared_inertia_projection_error_microkilogram_metre_squared": max(
            row["declared_maximum_error_microkilogram_metre_squared"]
            for row in model.inertia_identity
        ),
        "inertia_body_audits": list(model.inertia_identity),
        "joint_hierarchy_identity": hierarchy_pass,
        "joint_frame_and_semantic_axis_identity": frame_axis_pass,
        "maximum_semantic_axis_norm_error": axis_norm_error,
        "gravity_metres_per_second_squared": model.gravity.tolist(),
        "gravity_identity": gravity_pass,
        "generalized_order": (
            "engine-world root linear xyz, engine-world root angular xyz, "
            "descriptor dof ordinal 0..22"
        ),
    }


def audit_anchor(
    *,
    frame: int,
    model: SpatialModel,
    descriptor: dict[str, Any],
    cache: Mapping[str, NDArray[Any]],
    contact_modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    profile: Mapping[str, Any],
) -> dict[str, Any]:
    configuration = configuration_at(cache, frame)
    velocity = np.asarray(cache["velocity"][frame], dtype=np.float64)
    acceleration = np.asarray(cache["acceleration"][frame], dtype=np.float64)
    matrix, kinematics = mass_matrix(model, configuration)
    inverse_matrix = inverse_dynamics_mass_matrix(model, configuration, velocity)
    symmetry = float(
        np.linalg.norm(matrix - matrix.T) / max(np.linalg.norm(matrix), 1.0)
    )
    matrix_agreement = float(
        np.max(np.abs(matrix - inverse_matrix))
        / max(float(np.max(np.abs(matrix))), 1.0)
    )
    eigenvalues = np.linalg.eigvalsh((matrix + matrix.T) * 0.5)
    try:
        np.linalg.cholesky((matrix + matrix.T) * 0.5)
        cholesky_pass = True
    except np.linalg.LinAlgError:
        cholesky_pass = False
    symmetry_tolerance = float(
        profile["numeric_tolerances"]["mass_matrix_relative_symmetry"]
    )
    round_trip_tolerance = float(
        profile["numeric_tolerances"]["dynamics_round_trip_scaled_absolute"]
    )
    mass_pass = bool(
        symmetry <= symmetry_tolerance
        and matrix_agreement <= round_trip_tolerance
        and cholesky_pass
        and float(eigenvalues[0]) > 0.0
    )

    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    bias = inverse_dynamics(model, configuration, velocity, zero)
    generalized_force = inverse_dynamics(model, configuration, velocity, acceleration)
    recovered = np.linalg.solve(matrix, generalized_force - bias)
    round_trip_error = float(
        np.max(np.abs(recovered - acceleration))
        / max(float(np.max(np.abs(acceleration))), 1.0)
    )
    round_trip_pass = round_trip_error <= round_trip_tolerance

    reference_positions, reference_rotations = target_forward_kinematics(
        descriptor,
        configuration.root_position,
        matrix_to_quaternion(configuration.root_rotation),
        configuration.joint_positions,
    )
    body_position_error = float(
        np.max(np.abs(kinematics.body_positions - reference_positions))
    )
    body_rotation_error = float(
        np.max(np.abs(kinematics.body_rotations - reference_rotations))
    )
    contact_rows = []
    jdot_rows = []
    motion = propagate_motion(model, kinematics, velocity, zero)
    for point_ordinal, point in enumerate(points):
        body_slot = int(point["body_slot"])
        local = np.asarray(point["local_translation_metres"], dtype=np.float64)
        implementation_position = point_position(kinematics, body_slot, local)
        reference_position = (
            reference_positions[body_slot] + reference_rotations[body_slot] @ local
        )
        implementation_jacobian = point_jacobian(model, kinematics, body_slot, local)
        reference_jacobian = finite_difference_frozen_jacobian(
            descriptor=descriptor,
            configuration=configuration,
            body_slot=body_slot,
            local_point=local,
            probe=float(profile["numeric_probes"]["contact_jacobian_radians"]),
        )
        absolute_error = float(
            np.max(np.abs(implementation_jacobian - reference_jacobian))
        )
        relative_error = absolute_error / max(
            float(np.max(np.abs(reference_jacobian))), 1.0
        )
        position_error = float(
            np.max(np.abs(implementation_position - reference_position))
        )
        jacobian_pass = bool(
            np.allclose(
                implementation_jacobian,
                reference_jacobian,
                atol=float(profile["numeric_tolerances"]["contact_jacobian_absolute"]),
                rtol=float(profile["numeric_tolerances"]["contact_jacobian_relative"]),
            )
        )
        contact_rows.append(
            {
                "point_ordinal": point_ordinal,
                "effector_id": point["effector_id"],
                "maximum_position_absolute_metres": position_error,
                "maximum_jacobian_absolute": absolute_error,
                "maximum_jacobian_relative_scaled": relative_error,
                "status": (
                    "PASS"
                    if position_error
                    <= float(profile["numeric_tolerances"]["contact_position_absolute"])
                    and jacobian_pass
                    else "FAIL"
                ),
            }
        )
        if _point_active(contact_modes[frame], point_ordinal):
            analytic = point_acceleration(kinematics, motion, body_slot, local)
            finite_difference = finite_difference_frozen_jdot_v(
                descriptor=descriptor,
                configuration=configuration,
                velocity=velocity,
                body_slot=body_slot,
                local_point=local,
                probe_seconds=float(profile["numeric_probes"]["jdot_v_seconds"]),
            )
            error = float(np.max(np.abs(analytic - finite_difference)))
            jdot_rows.append(
                {
                    "point_ordinal": point_ordinal,
                    "effector_id": point["effector_id"],
                    "analytic_metres_per_second_squared": analytic.tolist(),
                    "finite_difference_metres_per_second_squared": (
                        finite_difference.tolist()
                    ),
                    "maximum_absolute_metres_per_second_squared": error,
                    "status": (
                        "PASS"
                        if error
                        <= float(
                            profile["numeric_tolerances"][
                                "jdot_v_absolute_metres_per_second_squared"
                            ]
                        )
                        else "FAIL"
                    ),
                }
            )
    contact_pass = bool(
        body_position_error
        <= float(profile["numeric_tolerances"]["contact_position_absolute"])
        and body_rotation_error
        <= float(profile["numeric_tolerances"]["body_rotation_absolute"])
        and all(row["status"] == "PASS" for row in contact_rows)
    )
    jdot_pass = all(row["status"] == "PASS" for row in jdot_rows)
    return {
        "frame": frame,
        "contact_modes": contact_modes[frame].tolist(),
        "mass_matrix": {
            "status": "PASS" if mass_pass else "FAIL",
            "relative_symmetry_error": symmetry,
            "inverse_dynamics_matrix_relative_scaled_error": matrix_agreement,
            "minimum_eigenvalue": float(eigenvalues[0]),
            "maximum_eigenvalue": float(eigenvalues[-1]),
            "condition_number": float(np.linalg.cond(matrix)),
            "cholesky_succeeded": cholesky_pass,
        },
        "dynamics_round_trip": {
            "status": "PASS" if round_trip_pass else "FAIL",
            "scaled_absolute_error": round_trip_error,
            "acceleration_scale": max(float(np.max(np.abs(acceleration))), 1.0),
        },
        "contact_kinematics": {
            "status": "PASS" if contact_pass else "FAIL",
            "maximum_body_position_absolute_metres": body_position_error,
            "maximum_body_rotation_absolute": body_rotation_error,
            "points": contact_rows,
        },
        "jdot_v": {
            "status": "PASS" if jdot_pass else "FAIL",
            "active_point_count": len(jdot_rows),
            "points": jdot_rows,
        },
    }


def finite_difference_frozen_jacobian(
    *,
    descriptor: dict[str, Any],
    configuration: Configuration,
    body_slot: int,
    local_point: NDArray[np.float64],
    probe: float,
) -> NDArray[np.float64]:
    jacobian = np.empty((3, GENERALIZED_WIDTH), dtype=np.float64)
    for column in range(GENERALIZED_WIDTH):
        direction = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
        direction[column] = 1.0
        positive = integrate_configuration(configuration, direction, probe)
        negative = integrate_configuration(configuration, direction, -probe)
        jacobian[:, column] = (
            _frozen_point_position(descriptor, positive, body_slot, local_point)
            - _frozen_point_position(descriptor, negative, body_slot, local_point)
        ) / (2.0 * probe)
    return jacobian


def finite_difference_frozen_jdot_v(
    *,
    descriptor: dict[str, Any],
    configuration: Configuration,
    velocity: NDArray[np.float64],
    body_slot: int,
    local_point: NDArray[np.float64],
    probe_seconds: float,
) -> NDArray[np.float64]:
    positive = integrate_configuration(configuration, velocity, probe_seconds)
    negative = integrate_configuration(configuration, velocity, -probe_seconds)
    center = _frozen_point_position(descriptor, configuration, body_slot, local_point)
    return (
        _frozen_point_position(descriptor, positive, body_slot, local_point)
        - 2.0 * center
        + _frozen_point_position(descriptor, negative, body_slot, local_point)
    ) / (probe_seconds * probe_seconds)


def audit_point_force_projection(
    *,
    model: SpatialModel,
    descriptor: Mapping[str, Any],
    cache: Mapping[str, NDArray[Any]],
    points: tuple[dict[str, Any], ...],
    r113: Mapping[str, Any],
) -> dict[str, Any]:
    identity = r113["point_contact_force_identity"]
    configuration = configuration_at(cache, ANCHORS[4])
    kinematics = forward_kinematics(model, configuration)
    force_nrf = np.asarray((123.0, -4.0, 7.0), dtype=np.float64)
    world_force = force_nrf[[1, 0, 2]]
    rows = []
    for point in points:
        body_slot = int(point["body_slot"])
        local = np.asarray(point["local_translation_metres"], dtype=np.float64)
        point_world = point_position(kinematics, body_slot, local)
        jacobian = point_jacobian(model, kinematics, body_slot, local)
        projected = generalized_contact_force(jacobian, force_nrf)
        independent = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
        independent[:3] = world_force
        independent[3:6] = np.cross(
            point_world - kinematics.body_positions[0], world_force
        )
        for dof in model.ancestors_by_body[body_slot]:
            independent[6 + dof] = float(
                kinematics.joint_axes[dof]
                @ np.cross(point_world - kinematics.joint_origins[dof], world_force)
            )
        error = float(np.max(np.abs(projected - independent)))
        rows.append(
            {
                "point_ordinal": point["point_ordinal"],
                "effector_id": point["effector_id"],
                "maximum_generalized_force_absolute_newtons_or_newton_metres": error,
                "status": "PASS" if error <= 1.0e-12 else "FAIL",
            }
        )
    contract_pass = bool(
        identity["force_component_order"]
        == ["normal_newtons", "right_newtons", "forward_newtons"]
        and identity["world_force_mapping"]
        == "[right_newtons, normal_newtons, forward_newtons] in engine XYZ"
        and tuple(row["effector_id"] for row in points) == CONTACT_IDS
        and all(
            point["body_id"]
            == next(
                effector["body_id"]
                for effector in descriptor["effectors"]
                if effector["effector_id"] == point["effector_id"]
            )
            for point in points
        )
    )
    passed = contract_pass and all(row["status"] == "PASS" for row in rows)
    return {
        "status": "PASS" if passed else "FAIL",
        "force_component_order": identity["force_component_order"],
        "world_force_mapping": identity["world_force_mapping"],
        "probe_force_normal_right_forward_newtons": force_nrf.tolist(),
        "probe_world_force_xyz_newtons": world_force.tolist(),
        "application_point_and_order_identity": contract_pass,
        "point_audits": rows,
    }


def audit_r121_reproduction(
    *,
    profile: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    cache: Mapping[str, NDArray[Any]],
    r121: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
    v9_complete_clip_path: Path,
) -> dict[str, Any]:
    contact = audit_contact_schedule(
        v9_complete_clip_path,
        expected_sha256=profile["source"]["v9_complete_clip_sha256"],
        point_identity=r113["point_contact_force_identity"],
        r120=r120,
    )
    inventory = audit_system_inventory(
        active_point_collocations=contact["active_point_collocation_count"],
        inactive_point_collocations=contact["inactive_point_collocation_count"],
    )
    fixed_pd = audit_fixed_pd_schedule(cache=cache, descriptor=descriptor)
    lift = audit_affine_lift(cache)
    contact_pass = contact == r121["contact_schedule_audit"]
    inventory_pass = inventory == r121["system_inventory_audit"]
    fixed_pd_pass = fixed_pd == r121["fixed_pd_schedule_audit"]
    passed = bool(
        contact_pass and inventory_pass and fixed_pd_pass and lift["status"] == "PASS"
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "affine_lift": lift,
        "contact_schedule_exact_reproduction": contact_pass,
        "contact_schedule": contact,
        "system_inventory_exact_reproduction": inventory_pass,
        "system_inventory": inventory,
        "fixed_pd_schedule_exact_reproduction": fixed_pd_pass,
        "fixed_pd_schedule": fixed_pd,
    }


def audit_affine_lift(cache: Mapping[str, NDArray[Any]]) -> dict[str, Any]:
    digest = hashlib.sha256()
    maximum_orientation_arc = 0.0
    exact_left_reanchors = 0
    for interval in range(MOTOR_INTERVAL_COUNT):
        left_rotation = (
            rotation_exp(cache["root_orientation_delta_rad"][interval])
            @ cache["reference_root_rotation"][interval]
        )
        right_rotation = (
            rotation_exp(cache["root_orientation_delta_rad"][interval + 1])
            @ cache["reference_root_rotation"][interval + 1]
        )
        tangent = rotation_log(right_rotation @ left_rotation.T)
        maximum_orientation_arc = max(
            maximum_orientation_arc, float(np.linalg.norm(tangent))
        )
        for substep in range(SUBSTEPS_PER_INTERVAL):
            fraction = substep / SUBSTEPS_PER_INTERVAL
            samples = (
                affine_sample(
                    cache["root_position_m"][interval],
                    cache["root_position_m"][interval + 1],
                    substep,
                ),
                rotation_exp(fraction * tangent) @ left_rotation,
                affine_sample(
                    cache["joint_position_rad"][interval],
                    cache["joint_position_rad"][interval + 1],
                    substep,
                ),
                affine_sample(
                    cache["velocity"][interval],
                    cache["velocity"][interval + 1],
                    substep,
                ),
                affine_sample(
                    cache["acceleration"][interval],
                    cache["acceleration"][interval + 1],
                    substep,
                ),
            )
            for sample in samples:
                digest.update(np.ascontiguousarray(sample).tobytes())
            if substep == 0 and all(
                np.array_equal(sample, expected)
                for sample, expected in zip(
                    (samples[0], samples[2], samples[3], samples[4]),
                    (
                        cache["root_position_m"][interval],
                        cache["joint_position_rad"][interval],
                        cache["velocity"][interval],
                        cache["acceleration"][interval],
                    ),
                    strict=True,
                )
            ):
                exact_left_reanchors += 1
    passed = bool(
        exact_left_reanchors == MOTOR_INTERVAL_COUNT
        and maximum_orientation_arc < math.pi
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "collocation_count": COLLOCATION_COUNT,
        "substeps_per_motor_interval": SUBSTEPS_PER_INTERVAL,
        "exact_left_reanchor_count": exact_left_reanchors,
        "maximum_principal_orientation_arc_radians": maximum_orientation_arc,
        "ordered_float64_lift_sha256": digest.hexdigest(),
        "kinematic_derivative_identity_claimed": False,
        "integrated_state_carried_between_intervals": False,
    }


def audit_one_local_system_resource(profile: Mapping[str, Any]) -> dict[str, Any]:
    unknowns = int(profile["resource_instrumentation"]["local_unknown_count"])
    matrix = np.empty((unknowns, unknowns), dtype=np.float64)
    right_hand_side = np.empty(unknowns, dtype=np.float64)
    allocated = matrix.nbytes + right_hand_side.nbytes
    expected = unknowns * unknowns * 8 + unknowns * 8
    passed = bool(
        unknowns == 64
        and allocated == expected
        and profile["resource_instrumentation"]["r123_system_solve"] is False
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "local_unknown_count": unknowns,
        "matrix_shape": list(matrix.shape),
        "right_hand_side_shape": list(right_hand_side.shape),
        "allocated_payload_bytes": allocated,
        "factorization_performed": False,
        "solve_performed": False,
        "r123_local_system_solves": 0,
    }


def load_r120_cache(path: Path, profile: Mapping[str, Any]) -> dict[str, NDArray[Any]]:
    if sha256(path) != profile["source"]["r120"]["cache_sha256"]:
        raise ValueError("R122 R120 cache identity differs")
    with np.load(path, allow_pickle=False) as archive:
        if tuple(archive.files) != CACHE_KEYS:
            raise ValueError("R122 cache keys differ")
        cache = {key: np.array(archive[key], copy=True) for key in CACHE_KEYS}
    expected_shapes = {
        "root_position_m": (FRAME_COUNT, 3),
        "root_orientation_delta_rad": (FRAME_COUNT, 3),
        "joint_position_rad": (FRAME_COUNT, ACTUATOR_COUNT),
        "velocity": (FRAME_COUNT, GENERALIZED_WIDTH),
        "acceleration": (FRAME_COUNT, GENERALIZED_WIDTH),
        "reference_root_rotation": (FRAME_COUNT, 3, 3),
        "metadata_json_utf8": (264,),
    }
    if any(cache[key].shape != shape for key, shape in expected_shapes.items()):
        raise ValueError("R122 cache shape differs")
    if (
        any(cache[key].dtype != np.float64 for key in CACHE_KEYS[:-1])
        or cache["metadata_json_utf8"].dtype != np.uint8
    ):
        raise ValueError("R122 cache dtype differs")
    return cache


def configuration_at(cache: Mapping[str, NDArray[Any]], frame: int) -> Configuration:
    return Configuration(
        root_position=np.asarray(cache["root_position_m"][frame], dtype=np.float64),
        root_rotation=rotation_exp(
            np.asarray(cache["root_orientation_delta_rad"][frame], dtype=np.float64)
        )
        @ np.asarray(cache["reference_root_rotation"][frame], dtype=np.float64),
        joint_positions=np.asarray(
            cache["joint_position_rad"][frame], dtype=np.float64
        ),
    )


def rotation_log(rotation: NDArray[np.float64]) -> NDArray[np.float64]:
    cosine = float(np.clip((np.trace(rotation) - 1.0) * 0.5, -1.0, 1.0))
    angle = math.acos(cosine)
    vee = np.asarray(
        (
            rotation[2, 1] - rotation[1, 2],
            rotation[0, 2] - rotation[2, 0],
            rotation[1, 0] - rotation[0, 1],
        ),
        dtype=np.float64,
    )
    if angle < 1.0e-10:
        return 0.5 * vee
    if math.pi - angle < 1.0e-7:
        raise ValueError("R122 SO(3) principal log is ambiguous")
    return angle / (2.0 * math.sin(angle)) * vee


def _frozen_point_position(
    descriptor: dict[str, Any],
    configuration: Configuration,
    body_slot: int,
    local_point: NDArray[np.float64],
) -> NDArray[np.float64]:
    positions, rotations = target_forward_kinematics(
        descriptor,
        configuration.root_position,
        matrix_to_quaternion(configuration.root_rotation),
        configuration.joint_positions,
    )
    return positions[body_slot] + rotations[body_slot] @ local_point


def _contact_points(
    *, descriptor: Mapping[str, Any], r113: Mapping[str, Any]
) -> tuple[dict[str, Any], ...]:
    identity = r113["point_contact_force_identity"]
    ordered = identity["ordered_points"]
    applications = identity["application_point_identity"]
    body_slots = {
        body["body_id"]: int(body["body_slot"]) for body in descriptor["bodies"]
    }
    effectors = {
        effector["effector_id"]: effector for effector in descriptor["effectors"]
    }
    points = []
    for ordinal, (point, application) in enumerate(
        zip(ordered, applications, strict=True)
    ):
        effector = effectors[point["effector_id"]]
        if (
            int(point["point_ordinal"]) != ordinal
            or int(application["point_ordinal"]) != ordinal
            or point["effector_id"] != application["effector_id"]
            or point["body_id"] != application["body_id"]
            or effector["body_id"] != point["body_id"]
            or effector["local_translation_micrometres"]
            != application["local_translation_micrometres"]
        ):
            raise ValueError("R122 R113 point application identity differs")
        points.append(
            {
                "point_ordinal": ordinal,
                "effector_id": point["effector_id"],
                "body_id": point["body_id"],
                "body_slot": body_slots[point["body_id"]],
                "local_translation_metres": (
                    np.asarray(
                        application["local_translation_micrometres"],
                        dtype=np.float64,
                    )
                    / 1_000_000.0
                ).tolist(),
            }
        )
    if tuple(row["effector_id"] for row in points) != CONTACT_IDS:
        raise ValueError("R122 R113 point order differs")
    return tuple(points)


def _point_active(modes: NDArray[np.uint8], point_ordinal: int) -> bool:
    side = point_ordinal // 2
    point = point_ordinal % 2
    return int(modes[side]) in ((1, 3) if point == 0 else (2, 3))


def _load_bound_report(
    report_path: Path, profile_path: Path, expected: Mapping[str, Any]
) -> dict[str, Any]:
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R122 bound report file identity differs")
    report = json.loads(report_path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError("R122 bound canonical report identity differs")
    return report


def _load_r120(path: Path, expected: Mapping[str, Any]) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ValueError("R122 R120 report file identity differs")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ValueError("R122 R120 canonical report identity differs")
    return report


def _validate_source_reports(
    *,
    profile: Mapping[str, Any],
    r121: Mapping[str, Any],
    r113: Mapping[str, Any],
    r120: Mapping[str, Any],
) -> None:
    expected = profile["source"]
    if (
        r121.get("check") != "TRAIN-4-FIXED-PD-INVERSE-DYNAMICS-EXECUTION-FORMULATION"
        or r121.get("status") != "COMPLETE"
        or r121.get("gate_decision")
        != "PERMIT_R122_FIXED_PD_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY"
        or r121.get("repository", {}).get("commit")
        != expected["r121"]["repository_commit"]
        or r121.get("repository", {}).get("dirty") is not False
        or r121.get("identities", {}).get("formulation_module_sha256")
        != expected["r121"]["formulation_module_sha256"]
        or r121.get("identities", {}).get("tool_sha256")
        != expected["r121"]["tool_sha256"]
        or r113.get("status") != "PASS"
        or r113.get("model_identity_result", {}).get(
            "single_backend_neutral_dynamics_model_available"
        )
        is not True
        or r120.get("status") != "PASS"
        or r120.get("solver_result", {}).get("accepted_exact_result", {}).get("status")
        != "PASS"
        or r121.get("identities", {}).get("r120_cache_sha256")
        != expected["r120"]["cache_sha256"]
        or any(
            int(r121.get(counter, -1)) != 0
            for counter in (
                "solver_runs",
                "local_system_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "solver_private_warm_start_caches",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R122 source report contract differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R122 conformance requires a clean repository")


def _validate_solver_import_audit(audit: Mapping[str, Any]) -> None:
    expected = {
        "osqp_module_loaded": False,
        "scipy_module_loaded": False,
        "pinocchio_module_loaded": False,
        "r123_execution_module_imported": False,
        "r123_system_solve_requested": False,
    }
    if dict(audit) != expected:
        raise ValueError("R122 solver import audit differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R122 validation results differ")
    return normalized


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    tolerances = profile.get("numeric_tolerances", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnly"
        or scope.get("run_id") != "R122"
        or scope.get("required_anchors") != list(ANCHORS)
        or scope.get("r123_local_system_solves") != 0
        or scope.get("inverse_dynamics_execution") is not False
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training") is not False
        or source.get("current_descriptor_file_sha256")
        != "7928fe23affaf9dd16a0c82db1d7da85e61af2ad0f071f9423c6df1121ba50a3"
        or source.get("v9_complete_clip_sha256")
        != "16cacfc0d6a5a57a7e2e2fad75faef1681e9b17c02cd3b78bafc7bb4ed3b5b7d"
        or source.get("kernel_module_sha256")
        != "220e2db2113196031082eb9fa0b7e3d8614582438da7aaee3f2b6a986bac2a02"
        or source.get("motion_math_module_sha256")
        != "2866ef76bc7e056b4cfdd3bd60378ef5c7813d0b3003eaccf9bc5207b8b79d68"
        or tolerances.get("mass_matrix_relative_symmetry") != 1.0e-10
        or tolerances.get("dynamics_round_trip_scaled_absolute") != 1.0e-9
        or tolerances.get("contact_jacobian_absolute") != 1.0e-7
        or tolerances.get("contact_jacobian_relative") != 1.0e-6
        or tolerances.get("jdot_v_absolute_metres_per_second_squared") != 1.0e-5
        or decision.get("pass")
        != "PERMIT_R123_SINGLE_BOUNDED_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_ONLY"
        or decision.get("fail") != "STOP_INVALID_DYNAMICS_IMPLEMENTATION"
        or bounded.get("r123_inverse_dynamics_execution")
        != "AUTHORIZED_ON_EXACT_R122_PASS_ONLY"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_kto_solve",
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
            "solver_free_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("R122 conformance profile differs")
