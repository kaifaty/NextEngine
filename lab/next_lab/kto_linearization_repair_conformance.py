from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab import contact_manifold
from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.motion_math import (
    decompose_xzy,
    matrix_to_quaternion,
    quaternion_to_matrix,
    rotation_axis,
    target_effectors,
    target_forward_kinematics,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor

CONFORMANCE_ID = "nextengine.humanoid-kto-linearization-repair-conformance.v1"
CHECK_ID = "TRAIN-4-KTO-LINEARIZATION-REPAIR-CONFORMANCE"
SIDE_NAMES = ("left", "right")
POINT_NAMES = ("heel", "forefoot")
AXIS_NAMES = ("x", "y", "z")
ORIENTATION_AXIS_NAMES = ("world-x", "world-y", "world-z")
FRAME_COUNT = 801
JOINT_COUNT = 23
EXACT_ARRAYS = {
    "center_of_mass_um": ((FRAME_COUNT, 3), "int64"),
    "contact_modes": ((FRAME_COUNT, 2), "uint8"),
    "contacts": ((FRAME_COUNT, 7), "uint8"),
    "effector_position_um": ((FRAME_COUNT, 6, 3), "int64"),
    "joint_position_urad": ((FRAME_COUNT, JOINT_COUNT), "int64"),
    "joint_velocity_urad_s": ((FRAME_COUNT, JOINT_COUNT), "int64"),
    "phase_u16": ((FRAME_COUNT,), "uint16"),
    "reference_frame": ((FRAME_COUNT,), "int64"),
    "root_linear_velocity_um_s": ((FRAME_COUNT, 3), "int64"),
    "root_position_um": ((FRAME_COUNT, 3), "int64"),
    "root_quaternion_q1_30": ((FRAME_COUNT, 4), "int64"),
    "root_yaw_urad": ((FRAME_COUNT,), "int64"),
    "root_yaw_velocity_urad_s": ((FRAME_COUNT,), "int64"),
}
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


@dataclass(frozen=True)
class VariableSpec:
    name: str
    block: str
    kind: str
    frame: int
    index: int
    unit: str
    probe: float


@dataclass(frozen=True)
class AnchorDefinition:
    role: str
    frame: int
    side: int
    point: int
    stencil_frames: tuple[int, int]
    stencil_coefficients: tuple[float, float]

    @property
    def point_id(self) -> str:
        return f"frame-{self.frame}:{SIDE_NAMES[self.side]}-{POINT_NAMES[self.point]}"


def build_kto_linearization_repair_conformance(
    *,
    profile_path: Path,
    r117_report_path: Path,
    r117_profile_path: Path,
    descriptor_bytes: bytes,
    v9_profile_path: Path,
    v9_complete_clip_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    solver_import_audit: Mapping[str, Any],
) -> dict[str, Any]:
    """Audit R117's repaired function and q/v derivative without a solve."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r117_report_path,
            r117_profile_path,
            v9_profile_path,
            v9_complete_clip_path,
            tool_path,
        )
    )
    (
        profile_path,
        r117_report_path,
        r117_profile_path,
        v9_profile_path,
        v9_complete_clip_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R118 conformance input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_solver_import_audit(solver_import_audit)
    r117 = _validate_r117(
        profile=profile,
        report_path=r117_report_path,
        profile_path=r117_profile_path,
    )
    _validate_source_files(
        profile=profile,
        descriptor_bytes=descriptor_bytes,
        v9_profile_path=v9_profile_path,
        v9_complete_clip_path=v9_complete_clip_path,
    )
    validations = _validate_results(profile, validation_results)

    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    arrays, metadata = load_complete_arrays(v9_complete_clip_path)
    active = contact_manifold.contact_point_mask(arrays["contact_modes"])
    stencil_indices, stencil_coefficients = hybrid_velocity_stencil(active)
    exact_probe = float(profile["function_contract"]["lever_probe_radians"])
    baseline = audit_all_active_function_identity(
        descriptor=descriptor,
        arrays=arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        exact_probe=exact_probe,
        tolerance=float(
            profile["numeric_tolerances"][
                "function_identity_absolute_micrometres_per_second_per_component"
            ]
        ),
    )
    anchors = select_anchor_definitions(
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        expected=profile["deterministic_anchors"],
    )
    anchor_audits = [
        audit_anchor_jacobian(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            profile=profile,
        )
        for anchor in anchors
    ]
    discriminators = {
        "all_active_function_identity": baseline["status"] == "PASS",
        "deterministic_anchor_closure": tuple(row["point_id"] for row in anchor_audits)
        == tuple(profile["deterministic_anchors"]),
        "whole_function_jacobian_agreement": all(
            row["vector_jacobian"]["status"] == "PASS" for row in anchor_audits
        ),
        "configuration_nonzero_closure": all(
            row["configuration_dependencies"]["status"] == "PASS"
            for row in anchor_audits
        ),
        "neighbor_yaw_dependency_closure": all(
            row["neighbor_yaw_dependencies"]["status"] == "PASS"
            for row in anchor_audits
        ),
        "explicit_velocity_dependency_closure": all(
            row["velocity_dependencies"]["status"] == "PASS" for row in anchor_audits
        ),
        "norm_squared_tangent_chain_rule": all(
            row["tangential_norm_squared"]["status"] == "PASS" for row in anchor_audits
        ),
        "solver_free_process": solver_import_audit.get("osqp_module_loaded") is False,
        "zero_execution_counters": True,
    }
    passed = all(discriminators.values())
    status = "PASS" if passed else "FAIL"
    gate_decision = profile["decision"]["pass" if passed else "fail"]
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "conformance_id": CONFORMANCE_ID,
        "status": status,
        "claim": profile["claim"],
        "gate_decision": gate_decision,
        "scope": profile["scope"],
        "source_evidence": {
            "r117": {
                "status": r117["status"],
                "gate_decision": r117["gate_decision"],
                "report_sha256": r117["report_sha256"],
                "repository_commit": r117["repository"]["commit"],
            },
            "v9": {
                "artifact_id": metadata["artifact_id"],
                "clip_id": metadata["clip_id"],
                "complete_clip_sha256": sha256(v9_complete_clip_path),
            },
        },
        "function_contract": profile["function_contract"],
        "variable_contract": profile["variable_contract"],
        "all_active_function_identity": baseline,
        "deterministic_anchor_audits": anchor_audits,
        "discriminators": discriminators,
        "solver_import_audit": dict(solver_import_audit),
        "numeric_tolerances": profile["numeric_tolerances"],
        "next_smallest_action": profile["next_smallest_action"][
            "pass" if passed else "fail"
        ],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r117_report_file_sha256": sha256(r117_report_path),
            "r117_profile_sha256": sha256(r117_profile_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "v9_profile_sha256": sha256(v9_profile_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "contact_manifold_module_sha256": sha256(
                Path(contact_manifold.__file__).resolve()
            ),
            "motion_math_module_sha256": sha256(
                Path(quaternion_to_matrix.__code__.co_filename).resolve()
            ),
            "conformance_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "conformance_reports": 1,
        **{counter: 0 for counter in ZERO_EXECUTION_COUNTERS},
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_all_active_function_identity(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    exact_probe: float,
    tolerance: float,
) -> dict[str, Any]:
    root_positions = arrays["root_position_um"].astype(np.float64) / 1_000_000.0
    root_quaternions = arrays["root_quaternion_q1_30"].astype(np.float64) / float(
        1 << 30
    )
    joint_positions = arrays["joint_position_urad"].astype(np.float64) / 1_000_000.0
    exact = contact_manifold._analytic_active_point_velocities(
        descriptor=descriptor,
        root_positions=root_positions,
        root_quaternions=root_quaternions,
        joint_positions=joint_positions,
        root_linear_velocity_um_s=arrays["root_linear_velocity_um_s"],
        root_yaw_velocity_urad_s=arrays["root_yaw_velocity_urad_s"],
        joint_velocity_urad_s=arrays["joint_velocity_urad_s"],
        active=active,
        probe=exact_probe,
    )
    maximum = -1.0
    maximum_identity = ""
    maximum_component = ""
    differing = 0
    compared = 0
    for frame, side, point in np.argwhere(active):
        anchor = AnchorDefinition(
            role="all-active-function-identity",
            frame=int(frame),
            side=int(side),
            point=int(point),
            stencil_frames=tuple(int(value) for value in stencil_indices[int(frame)]),
            stencil_coefficients=tuple(
                float(value) for value in stencil_coefficients[int(frame)]
            ),
        )
        specs = variable_specs(descriptor, anchor, profile_probes=None)
        value = repaired_exact_contact_velocity(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            variables=specs,
            delta=np.zeros(len(specs), dtype=np.float64),
            exact_probe=exact_probe,
        )
        component_delta = np.abs(value - exact[frame, side, point])
        compared += len(component_delta)
        differing += int(np.count_nonzero(component_delta > tolerance))
        local = int(np.argmax(component_delta))
        if float(component_delta[local]) > maximum:
            maximum = float(component_delta[local])
            maximum_identity = anchor.point_id
            maximum_component = AXIS_NAMES[local]
    passed = differing == 0
    return {
        "status": "PASS" if passed else "FAIL",
        "active_point_frame_count": int(np.sum(active)),
        "compared_component_count": compared,
        "differing_component_count": differing,
        "maximum_absolute_component_difference_micrometres_per_second": maximum,
        "maximum_difference_point_id": maximum_identity,
        "maximum_difference_component": maximum_component,
        "absolute_tolerance_micrometres_per_second_per_component": tolerance,
    }


def audit_anchor_jacobian(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    profile: Mapping[str, Any],
) -> dict[str, Any]:
    probes = profile["variable_contract"]["probes"]
    specs = variable_specs(descriptor, anchor, profile_probes=probes)
    exact_probe = float(profile["function_contract"]["lever_probe_radians"])
    zero = np.zeros(len(specs), dtype=np.float64)
    production_value = repaired_exact_contact_velocity(
        descriptor=descriptor,
        arrays=arrays,
        anchor=anchor,
        variables=specs,
        delta=zero,
        exact_probe=exact_probe,
    )
    reference_value = independent_exact_contact_velocity(
        descriptor=descriptor,
        arrays=arrays,
        anchor=anchor,
        variables=specs,
        delta=zero,
        exact_probe=exact_probe,
    )
    production = symmetric_whole_function_jacobian(
        evaluator=repaired_exact_contact_velocity,
        descriptor=descriptor,
        arrays=arrays,
        anchor=anchor,
        variables=specs,
        exact_probe=exact_probe,
    )
    reference = independent_symmetric_reference_jacobian(
        descriptor=descriptor,
        arrays=arrays,
        anchor=anchor,
        variables=specs,
        exact_probe=exact_probe,
    )
    tolerances = profile["numeric_tolerances"]
    absolute = float(
        tolerances["jacobian_absolute_micrometres_per_second_per_variable_unit"]
    )
    relative = float(tolerances["jacobian_relative"])
    allowed = absolute + relative * np.maximum(np.abs(production), np.abs(reference))
    difference = np.abs(production - reference)
    violation = difference > allowed
    variable_rows = []
    for ordinal, spec in enumerate(specs):
        variable_rows.append(
            {
                "name": spec.name,
                "block": spec.block,
                "unit": spec.unit,
                "probe": spec.probe,
                "production_xyz_micrometres_per_second_per_unit": [
                    float(value) for value in production[:, ordinal]
                ],
                "reference_xyz_micrometres_per_second_per_unit": [
                    float(value) for value in reference[:, ordinal]
                ],
                "maximum_absolute_difference": float(np.max(difference[:, ordinal])),
                "component_violation_count": int(
                    np.count_nonzero(violation[:, ordinal])
                ),
                "production_norm": float(np.linalg.norm(production[:, ordinal])),
                "reference_norm": float(np.linalg.norm(reference[:, ordinal])),
            }
        )
    vector_pass = not np.any(violation)
    nonzero_threshold = float(
        tolerances["configuration_nonzero_micrometres_per_second_per_radian"]
    )
    required_configuration = [
        row
        for row in variable_rows
        if row["block"] == "configuration" and row["reference_norm"] > nonzero_threshold
    ]
    missing_configuration = [
        row["name"]
        for row in required_configuration
        if row["production_norm"] <= nonzero_threshold
    ]
    neighbor_frames = sorted(set(anchor.stencil_frames) - {anchor.frame})
    missing_neighbor_frames = []
    for frame in neighbor_frames:
        rows = [
            row
            for row in variable_rows
            if row["name"].startswith(f"q[{frame}].root_orientation.")
        ]
        if not rows or max(row["production_norm"] for row in rows) <= nonzero_threshold:
            missing_neighbor_frames.append(frame)

    root_linear_rows = [
        row
        for row in variable_rows
        if row["name"].startswith(f"v[{anchor.frame}].root_linear.")
    ]
    root_linear_present = len(root_linear_rows) == 3 and all(
        row["production_norm"] > nonzero_threshold for row in root_linear_rows
    )
    leg_ordinals = {
        int(value)
        for value in contact_manifold._leg_joint_ordinals(descriptor)[anchor.side]
    }
    same_side_velocity_rows = [
        row
        for spec, row in zip(specs, variable_rows, strict=True)
        if spec.kind == "joint_velocity" and spec.index in leg_ordinals
    ]
    same_side_present = len(same_side_velocity_rows) == 6 and all(
        row["production_norm"] > nonzero_threshold
        and row["reference_norm"] > nonzero_threshold
        for row in same_side_velocity_rows
    )
    unrelated_velocity_rows = [
        row
        for spec, row in zip(specs, variable_rows, strict=True)
        if spec.block == "velocity"
        and (
            spec.kind == "root_angular_velocity"
            or (spec.kind == "joint_velocity" and spec.index not in leg_ordinals)
        )
    ]
    maximum_unrelated = max(
        (
            max(row["production_norm"], row["reference_norm"])
            for row in unrelated_velocity_rows
        ),
        default=0.0,
    )
    unrelated_zero = maximum_unrelated <= absolute

    scalar = audit_tangential_norm_squared(
        descriptor=descriptor,
        arrays=arrays,
        anchor=anchor,
        variables=specs,
        exact_probe=exact_probe,
        production_velocity=production_value,
        production_vector_jacobian=production,
        absolute_tolerance=float(
            tolerances[
                "tangent_chain_rule_absolute_square_metres_per_square_second_per_unit"
            ]
        ),
        relative_tolerance=relative,
    )
    return {
        "status": (
            "PASS"
            if vector_pass
            and not missing_configuration
            and not missing_neighbor_frames
            and root_linear_present
            and same_side_present
            and unrelated_zero
            and scalar["status"] == "PASS"
            else "FAIL"
        ),
        "role": anchor.role,
        "point_id": anchor.point_id,
        "frame": anchor.frame,
        "side": SIDE_NAMES[anchor.side],
        "point": POINT_NAMES[anchor.point],
        "stencil_frames": list(anchor.stencil_frames),
        "stencil_coefficients_per_second": list(anchor.stencil_coefficients),
        "variable_count": len(specs),
        "production_baseline_velocity_micrometres_per_second": [
            float(value) for value in production_value
        ],
        "independent_baseline_velocity_micrometres_per_second": [
            float(value) for value in reference_value
        ],
        "baseline_function_path_maximum_difference_micrometres_per_second": float(
            np.max(np.abs(production_value - reference_value))
        ),
        "vector_jacobian": {
            "status": "PASS" if vector_pass else "FAIL",
            "maximum_absolute_difference": float(np.max(difference)),
            "maximum_tolerance_ratio": float(np.max(difference / allowed)),
            "violating_component_count": int(np.count_nonzero(violation)),
        },
        "configuration_dependencies": {
            "status": "PASS" if not missing_configuration else "FAIL",
            "independent_nonzero_column_count": len(required_configuration),
            "missing_production_columns": missing_configuration,
            "threshold_micrometres_per_second_per_radian": nonzero_threshold,
        },
        "neighbor_yaw_dependencies": {
            "status": "PASS" if not missing_neighbor_frames else "FAIL",
            "neighbor_frames": neighbor_frames,
            "missing_neighbor_frames": missing_neighbor_frames,
        },
        "velocity_dependencies": {
            "status": (
                "PASS"
                if root_linear_present and same_side_present and unrelated_zero
                else "FAIL"
            ),
            "root_linear_columns_present": root_linear_present,
            "same_side_joint_velocity_columns_present": same_side_present,
            "unrelated_velocity_column_count": len(unrelated_velocity_rows),
            "maximum_unrelated_velocity_derivative_norm": maximum_unrelated,
        },
        "tangential_norm_squared": scalar,
        "variable_rows": variable_rows,
    }


def repaired_exact_contact_velocity(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    variables: Sequence[VariableSpec],
    delta: NDArray[np.float64],
    exact_probe: float,
) -> NDArray[np.float64]:
    """R117 physical-unit extension of the emitted one-sided contact kernel."""

    perturbation = _decode_perturbation(variables, delta, anchor.frame)
    required_frames = {anchor.frame, *anchor.stencil_frames}
    base_rotations = {
        frame: _source_rotation(arrays, frame) for frame in required_frames
    }
    current_rotation = (
        rotation_exp(perturbation["orientation"][anchor.frame])
        @ base_rotations[anchor.frame]
    )
    root_position = (
        arrays["root_position_um"][anchor.frame].astype(np.float64) / 1_000_000.0
        + perturbation["root_translation"]
    )
    joints = (
        arrays["joint_position_urad"][anchor.frame].astype(np.float64) / 1_000_000.0
        + perturbation["joint_position"]
    )
    point_name = f"effector.{SIDE_NAMES[anchor.side]}-{POINT_NAMES[anchor.point]}"
    baseline = _effector_point(
        descriptor, root_position, current_rotation, joints, point_name
    )
    yaw_point = _effector_point(
        descriptor,
        root_position,
        rotation_axis("Y", exact_probe) @ current_rotation,
        joints,
        point_name,
    )
    yaw_lever_um_per_rad = (yaw_point - baseline) * (1_000_000.0 / exact_probe)
    yaw_velocity_rad_s = (
        float(arrays["root_yaw_velocity_urad_s"][anchor.frame]) / 1_000_000.0
    )
    for frame, coefficient in zip(
        anchor.stencil_frames, anchor.stencil_coefficients, strict=True
    ):
        candidate = (
            rotation_exp(perturbation["orientation"][frame]) @ base_rotations[frame]
        )
        yaw_velocity_rad_s += coefficient * nearest_yaw_delta(
            candidate, base_rotations[frame]
        )
    velocity = (
        arrays["root_linear_velocity_um_s"][anchor.frame].astype(np.float64)
        + perturbation["root_linear_velocity"] * 1_000_000.0
    )
    velocity += yaw_lever_um_per_rad * yaw_velocity_rad_s
    joint_velocity = (
        arrays["joint_velocity_urad_s"][anchor.frame].astype(np.float64) / 1_000_000.0
        + perturbation["joint_velocity"]
    )
    for ordinal_value in contact_manifold._leg_joint_ordinals(descriptor)[anchor.side]:
        ordinal = int(ordinal_value)
        candidate_joints = joints.copy()
        candidate_joints[ordinal] += exact_probe
        candidate = _effector_point(
            descriptor,
            root_position,
            current_rotation,
            candidate_joints,
            point_name,
        )
        lever = (candidate - baseline) * (1_000_000.0 / exact_probe)
        velocity += lever * joint_velocity[ordinal]
    return velocity


def independent_exact_contact_velocity(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    variables: Sequence[VariableSpec],
    delta: NDArray[np.float64],
    exact_probe: float,
) -> NDArray[np.float64]:
    """Separately implemented whole-function reference for R118."""

    changes = _decode_perturbation(variables, delta, anchor.frame)
    required_frames = {anchor.frame, *anchor.stencil_frames}
    source_rotations = {
        frame: _source_rotation(arrays, frame) for frame in required_frames
    }
    rotation = (
        rotation_exp(changes["orientation"][anchor.frame])
        @ source_rotations[anchor.frame]
    )
    root = (
        arrays["root_position_um"][anchor.frame].astype(np.float64) / 1_000_000.0
        + changes["root_translation"]
    )
    joint = (
        arrays["joint_position_urad"][anchor.frame].astype(np.float64) / 1_000_000.0
        + changes["joint_position"]
    )
    name = f"effector.{SIDE_NAMES[anchor.side]}-{POINT_NAMES[anchor.point]}"

    def evaluate(
        candidate_rotation: NDArray[np.float64],
        candidate_joint: NDArray[np.float64],
    ) -> NDArray[np.float64]:
        positions, rotations = target_forward_kinematics(
            descriptor,
            root,
            matrix_to_quaternion(candidate_rotation),
            candidate_joint,
        )
        return target_effectors(descriptor, positions, rotations)[name]

    point = evaluate(rotation, joint)
    yaw_lever = (
        evaluate(rotation_axis("Y", exact_probe) @ rotation, joint) - point
    ) * (1_000_000.0 / exact_probe)
    yaw_rate = float(arrays["root_yaw_velocity_urad_s"][anchor.frame]) / 1_000_000.0
    for slot in range(2):
        source_frame = anchor.stencil_frames[slot]
        changed_rotation = (
            rotation_exp(changes["orientation"][source_frame])
            @ source_rotations[source_frame]
        )
        changed_yaw = decompose_xzy(changed_rotation)[2]
        source_yaw = decompose_xzy(source_rotations[source_frame])[2]
        wrapped_delta = math.atan2(
            math.sin(changed_yaw - source_yaw),
            math.cos(changed_yaw - source_yaw),
        )
        yaw_rate += anchor.stencil_coefficients[slot] * wrapped_delta

    result = (
        arrays["root_linear_velocity_um_s"][anchor.frame].astype(np.float64)
        + changes["root_linear_velocity"] * 1_000_000.0
    )
    result += yaw_lever * yaw_rate
    source_joint_velocity = (
        arrays["joint_velocity_urad_s"][anchor.frame].astype(np.float64) / 1_000_000.0
        + changes["joint_velocity"]
    )
    ordinals = contact_manifold._leg_joint_ordinals(descriptor)[anchor.side]
    for ordinal_value in ordinals:
        ordinal = int(ordinal_value)
        shifted = joint.copy()
        shifted[ordinal] += exact_probe
        joint_lever = (evaluate(rotation, shifted) - point) * (
            1_000_000.0 / exact_probe
        )
        result += joint_lever * source_joint_velocity[ordinal]
    return result


def symmetric_whole_function_jacobian(
    *,
    evaluator: Any,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    variables: Sequence[VariableSpec],
    exact_probe: float,
) -> NDArray[np.float64]:
    result = np.empty((3, len(variables)), dtype=np.float64)
    for ordinal, variable in enumerate(variables):
        plus = np.zeros(len(variables), dtype=np.float64)
        minus = np.zeros(len(variables), dtype=np.float64)
        plus[ordinal] = variable.probe
        minus[ordinal] = -variable.probe
        result[:, ordinal] = (
            evaluator(
                descriptor=descriptor,
                arrays=arrays,
                anchor=anchor,
                variables=variables,
                delta=plus,
                exact_probe=exact_probe,
            )
            - evaluator(
                descriptor=descriptor,
                arrays=arrays,
                anchor=anchor,
                variables=variables,
                delta=minus,
                exact_probe=exact_probe,
            )
        ) / (2.0 * variable.probe)
    return result


def independent_symmetric_reference_jacobian(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    variables: Sequence[VariableSpec],
    exact_probe: float,
) -> NDArray[np.float64]:
    """Central difference the separately implemented whole reference function."""

    columns = []
    for ordinal, variable in enumerate(variables):
        positive = np.zeros(len(variables), dtype=np.float64)
        negative = np.zeros(len(variables), dtype=np.float64)
        positive[ordinal] = variable.probe
        negative[ordinal] = -variable.probe
        high = independent_exact_contact_velocity(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            variables=variables,
            delta=positive,
            exact_probe=exact_probe,
        )
        low = independent_exact_contact_velocity(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            variables=variables,
            delta=negative,
            exact_probe=exact_probe,
        )
        columns.append((high - low) / (2.0 * variable.probe))
    return np.stack(columns, axis=1)


def audit_tangential_norm_squared(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    variables: Sequence[VariableSpec],
    exact_probe: float,
    production_velocity: NDArray[np.float64],
    production_vector_jacobian: NDArray[np.float64],
    absolute_tolerance: float,
    relative_tolerance: float,
) -> dict[str, Any]:
    velocity_m_s = production_velocity / 1_000_000.0
    vector_jacobian_m_s = production_vector_jacobian / 1_000_000.0
    chain = 2.0 * (
        velocity_m_s[0] * vector_jacobian_m_s[0]
        + velocity_m_s[2] * vector_jacobian_m_s[2]
    )
    reference = np.empty(len(variables), dtype=np.float64)
    for ordinal, variable in enumerate(variables):
        plus = np.zeros(len(variables), dtype=np.float64)
        minus = np.zeros(len(variables), dtype=np.float64)
        plus[ordinal] = variable.probe
        minus[ordinal] = -variable.probe
        high = (
            independent_exact_contact_velocity(
                descriptor=descriptor,
                arrays=arrays,
                anchor=anchor,
                variables=variables,
                delta=plus,
                exact_probe=exact_probe,
            )
            / 1_000_000.0
        )
        low = (
            independent_exact_contact_velocity(
                descriptor=descriptor,
                arrays=arrays,
                anchor=anchor,
                variables=variables,
                delta=minus,
                exact_probe=exact_probe,
            )
            / 1_000_000.0
        )
        high_value = high[0] * high[0] + high[2] * high[2]
        low_value = low[0] * low[0] + low[2] * low[2]
        reference[ordinal] = (high_value - low_value) / (2.0 * variable.probe)
    difference = np.abs(chain - reference)
    allowed = absolute_tolerance + relative_tolerance * np.maximum(
        np.abs(chain), np.abs(reference)
    )
    passed = bool(np.all(difference <= allowed))
    return {
        "status": "PASS" if passed else "FAIL",
        "constraint_expression": "velocity_x_m_s^2 + velocity_z_m_s^2 <= 0.12^2",
        "constraint_kind": "NORM_SQUARED_NOT_COMPONENT_BOX",
        "bound_square_metres_per_square_second": 0.12**2,
        "baseline_value_square_metres_per_square_second": float(
            velocity_m_s[0] * velocity_m_s[0] + velocity_m_s[2] * velocity_m_s[2]
        ),
        "maximum_absolute_derivative_difference": float(np.max(difference)),
        "violating_variable_count": int(np.count_nonzero(difference > allowed)),
    }


def variable_specs(
    descriptor: Mapping[str, Any],
    anchor: AnchorDefinition,
    profile_probes: Mapping[str, Any] | None,
) -> tuple[VariableSpec, ...]:
    probes = {
        "root_translation_metres": 0.00001,
        "root_orientation_radians": 0.0001,
        "joint_position_radians": 0.0001,
        "root_linear_velocity_metres_per_second": 0.00001,
        "root_angular_velocity_radians_per_second": 0.0001,
        "joint_velocity_radians_per_second": 0.0001,
    }
    if profile_probes is not None:
        probes = {key: float(value) for key, value in profile_probes.items()}
    joints = sorted(descriptor["joints"], key=lambda row: row["dof_ordinal"])
    rows = []
    for axis, name in enumerate(AXIS_NAMES):
        rows.append(
            VariableSpec(
                f"q[{anchor.frame}].root_translation.{name}",
                "configuration",
                "root_translation",
                anchor.frame,
                axis,
                "metre",
                probes["root_translation_metres"],
            )
        )
    orientation_frames = sorted({anchor.frame, *anchor.stencil_frames})
    for frame in orientation_frames:
        for axis, name in enumerate(ORIENTATION_AXIS_NAMES):
            rows.append(
                VariableSpec(
                    f"q[{frame}].root_orientation.{name}",
                    "configuration",
                    "root_orientation",
                    frame,
                    axis,
                    "radian",
                    probes["root_orientation_radians"],
                )
            )
    for joint in joints:
        rows.append(
            VariableSpec(
                f"q[{anchor.frame}].{joint['joint_id']}",
                "configuration",
                "joint_position",
                anchor.frame,
                int(joint["dof_ordinal"]),
                "radian",
                probes["joint_position_radians"],
            )
        )
    for axis, name in enumerate(AXIS_NAMES):
        rows.append(
            VariableSpec(
                f"v[{anchor.frame}].root_linear.{name}",
                "velocity",
                "root_linear_velocity",
                anchor.frame,
                axis,
                "metre_per_second",
                probes["root_linear_velocity_metres_per_second"],
            )
        )
    for axis, name in enumerate(ORIENTATION_AXIS_NAMES):
        rows.append(
            VariableSpec(
                f"v[{anchor.frame}].root_orientation.{name}",
                "velocity",
                "root_angular_velocity",
                anchor.frame,
                axis,
                "radian_per_second",
                probes["root_angular_velocity_radians_per_second"],
            )
        )
    for joint in joints:
        rows.append(
            VariableSpec(
                f"v[{anchor.frame}].{joint['joint_id']}",
                "velocity",
                "joint_velocity",
                anchor.frame,
                int(joint["dof_ordinal"]),
                "radian_per_second",
                probes["joint_velocity_radians_per_second"],
            )
        )
    return tuple(rows)


def select_anchor_definitions(
    *,
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    expected: Sequence[str],
) -> tuple[AnchorDefinition, ...]:
    side_active = np.any(active, axis=2)
    onset = side_active & np.vstack(
        [np.zeros((1, 2), dtype=np.bool_), ~side_active[:-1]]
    )
    ending = side_active & np.vstack(
        [~side_active[1:], np.zeros((1, 2), dtype=np.bool_)]
    )

    def first(role: str, side: int, mask: NDArray[np.bool_]) -> AnchorDefinition:
        frames = np.flatnonzero(mask)
        if len(frames) == 0:
            raise ValueError(f"R118 has no {role} anchor for {SIDE_NAMES[side]}")
        frame = int(frames[0])
        points = np.flatnonzero(active[frame, side])
        if len(points) == 0:
            raise ValueError("R118 selected an inactive point")
        return AnchorDefinition(
            role=f"{SIDE_NAMES[side]}-{role}",
            frame=frame,
            side=side,
            point=int(points[0]),
            stencil_frames=tuple(int(value) for value in stencil_indices[frame]),
            stencil_coefficients=tuple(
                float(value) for value in stencil_coefficients[frame]
            ),
        )

    hotspot = AnchorDefinition(
        role="r115-rc1-hotspot",
        frame=328,
        side=0,
        point=1,
        stencil_frames=tuple(int(value) for value in stencil_indices[328]),
        stencil_coefficients=tuple(float(value) for value in stencil_coefficients[328]),
    )
    if not active[hotspot.frame, hotspot.side, hotspot.point]:
        raise ValueError("R118 hotspot is inactive")
    selected = [hotspot]
    for side in range(2):
        selected.extend(
            (
                first("entry", side, onset[:, side]),
                first("exit", side, ending[:, side]),
                first(
                    "centered",
                    side,
                    side_active[:, side] & (stencil_coefficients[:, 1] == 30.0),
                ),
            )
        )
    if tuple(row.point_id for row in selected) != tuple(expected):
        raise ValueError("R118 deterministic anchor identity differs")
    return tuple(selected)


def hybrid_velocity_stencil(
    active: NDArray[np.bool_],
) -> tuple[NDArray[np.int64], NDArray[np.float64]]:
    """Solver-free copy of the frozen entry/exit-aware 60 Hz stencil."""

    if active.shape != (FRAME_COUNT, 2, 2):
        raise ValueError("R118 contact mask shape differs")
    side_active = np.any(active, axis=2)
    onset = side_active & np.vstack(
        [np.zeros((1, 2), dtype=np.bool_), ~side_active[:-1]]
    )
    ending = side_active & np.vstack(
        [~side_active[1:], np.zeros((1, 2), dtype=np.bool_)]
    )
    indices = np.empty((FRAME_COUNT, 2), dtype=np.int64)
    coefficients = np.empty((FRAME_COUNT, 2), dtype=np.float64)
    for frame in range(FRAME_COUNT):
        if frame == 0:
            indices[frame] = (0, 1)
            coefficients[frame] = (-60.0, 60.0)
        elif frame == FRAME_COUNT - 1:
            indices[frame] = (frame - 1, frame)
            coefficients[frame] = (-60.0, 60.0)
        elif np.any(onset[frame]):
            indices[frame] = (frame, frame + 1)
            coefficients[frame] = (-60.0, 60.0)
        elif np.any(ending[frame]):
            indices[frame] = (frame - 1, frame)
            coefficients[frame] = (-60.0, 60.0)
        else:
            indices[frame] = (frame - 1, frame + 1)
            coefficients[frame] = (-30.0, 30.0)
    return indices, coefficients


def load_complete_arrays(
    path: Path,
) -> tuple[dict[str, NDArray[Any]], dict[str, Any]]:
    with np.load(path, allow_pickle=False) as source:
        arrays = {name: np.asarray(source[name]).copy() for name in source.files}
    metadata = json.loads(arrays.pop("metadata_json_utf8").tobytes().decode("utf-8"))
    actual = {name: (array.shape, str(array.dtype)) for name, array in arrays.items()}
    if actual != EXACT_ARRAYS:
        raise ValueError("R118 V9 complete-clip array schema differs")
    if metadata != {
        "artifact_id": "cmu16-walk-nominal-b--complete",
        "clip_id": "cmu16-walk-nominal-b",
        "effector_ids": [
            "effector.left-forefoot",
            "effector.left-heel",
            "effector.left-palm",
            "effector.right-forefoot",
            "effector.right-heel",
            "effector.right-palm",
        ],
        "frame_first": 0,
        "frame_last": 800,
        "projection_domain": "clip-global",
        "schema_version": 1,
        "source_clip_artifact_sha256": (
            "859bb9f473e1fb4023a004ec8cadb9a5465c81477600fe1e2f54048e0722f82c"
        ),
    }:
        raise ValueError("R118 V9 complete-clip metadata differs")
    return arrays, metadata


def rotation_exp(vector: NDArray[np.float64]) -> NDArray[np.float64]:
    angle = float(np.linalg.norm(vector))
    skew = np.asarray(
        (
            (0.0, -float(vector[2]), float(vector[1])),
            (float(vector[2]), 0.0, -float(vector[0])),
            (-float(vector[1]), float(vector[0]), 0.0),
        ),
        dtype=np.float64,
    )
    if angle < 1.0e-12:
        return np.eye(3, dtype=np.float64) + skew + 0.5 * (skew @ skew)
    return (
        np.eye(3, dtype=np.float64)
        + math.sin(angle) / angle * skew
        + (1.0 - math.cos(angle)) / (angle * angle) * (skew @ skew)
    )


def nearest_yaw_delta(
    candidate_rotation: NDArray[np.float64],
    source_rotation: NDArray[np.float64],
) -> float:
    candidate = decompose_xzy(candidate_rotation)[2]
    source = decompose_xzy(source_rotation)[2]
    return math.atan2(math.sin(candidate - source), math.cos(candidate - source))


def _decode_perturbation(
    variables: Sequence[VariableSpec],
    delta: NDArray[np.float64],
    current_frame: int,
) -> dict[str, Any]:
    if delta.shape != (len(variables),) or not np.all(np.isfinite(delta)):
        raise ValueError("R118 perturbation shape/value differs")
    orientation_frames = {
        spec.frame for spec in variables if spec.kind == "root_orientation"
    }
    result: dict[str, Any] = {
        "root_translation": np.zeros(3, dtype=np.float64),
        "orientation": {
            frame: np.zeros(3, dtype=np.float64) for frame in orientation_frames
        },
        "joint_position": np.zeros(JOINT_COUNT, dtype=np.float64),
        "root_linear_velocity": np.zeros(3, dtype=np.float64),
        "joint_velocity": np.zeros(JOINT_COUNT, dtype=np.float64),
    }
    for value, spec in zip(delta, variables, strict=True):
        if spec.kind == "root_translation":
            result["root_translation"][spec.index] += value
        elif spec.kind == "root_orientation":
            result["orientation"][spec.frame][spec.index] += value
        elif spec.kind == "joint_position":
            result["joint_position"][spec.index] += value
        elif spec.kind == "root_linear_velocity":
            result["root_linear_velocity"][spec.index] += value
        elif spec.kind == "root_angular_velocity":
            pass
        elif spec.kind == "joint_velocity":
            result["joint_velocity"][spec.index] += value
        else:
            raise ValueError(f"unsupported R118 variable kind {spec.kind!r}")
    if current_frame not in result["orientation"]:
        raise ValueError("R118 current orientation variable is absent")
    return result


def _source_rotation(
    arrays: Mapping[str, NDArray[Any]], frame: int
) -> NDArray[np.float64]:
    return quaternion_to_matrix(
        arrays["root_quaternion_q1_30"][frame].astype(np.float64) / float(1 << 30)
    )


def _effector_point(
    descriptor: dict[str, Any],
    root_position: NDArray[np.float64],
    root_rotation: NDArray[np.float64],
    joint_positions: NDArray[np.float64],
    point_name: str,
) -> NDArray[np.float64]:
    positions, rotations = target_forward_kinematics(
        descriptor,
        root_position,
        matrix_to_quaternion(root_rotation),
        joint_positions,
    )
    return target_effectors(descriptor, positions, rotations)[point_name]


def _validate_r117(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r117"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R118 R117 source file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    identities = report.get("identities", {})
    if (
        embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R118_KTO_LINEARIZATION_REPAIR_IMPLEMENTATION_CONFORMANCE_ONLY"
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("formulation_module_sha256")
        != expected["formulation_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or any(int(report.get(counter, -1)) != 0 for counter in ZERO_EXECUTION_COUNTERS)
    ):
        raise ValueError("R118 R117 source report contract differs")
    return report


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    descriptor_bytes: bytes,
    v9_profile_path: Path,
    v9_complete_clip_path: Path,
) -> None:
    expected = profile["source"]
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != expected["current_descriptor_file_sha256"]
        or sha256(v9_profile_path) != expected["v9_profile_sha256"]
        or sha256(v9_complete_clip_path) != expected["v9_complete_clip_sha256"]
        or sha256(Path(contact_manifold.__file__).resolve())
        != expected["contact_manifold_module_sha256"]
        or sha256(Path(quaternion_to_matrix.__code__.co_filename).resolve())
        != expected["motion_math_module_sha256"]
    ):
        raise ValueError("R118 exact source identity differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R118 validation result differs")
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
        raise ValueError("R118 conformance requires a clean repository")


def _validate_solver_import_audit(audit: Mapping[str, Any]) -> None:
    if audit != {
        "osqp_module_loaded": False,
        "solver_module_imported": False,
        "solver_execution_requested": False,
    }:
        raise ValueError("R118 solver-free import audit differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    r117 = source.get("r117", {})
    scope = profile.get("scope", {})
    function = profile.get("function_contract", {})
    tolerances = profile.get("numeric_tolerances", {})
    probes = profile.get("variable_contract", {}).get("probes", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("conformance_id") != CONFORMANCE_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "ExactKernelFullQvDerivativeConformanceOnly"
        or r117.get("report_sha256")
        != "b0a9f07012e0f43c660019df8a1f31e7368f6130312231cf6c2bddb0f59fb7c7"
        or scope
        != {
            "run_id": "R118",
            "clip_id": "cmu16-walk-nominal-b",
            "frame_count": 801,
            "conformance_reports": 1,
            "qp_solves": 0,
            "kto_solves": 0,
            "candidate_construction": False,
            "physx_scene_runs": 0,
            "training": False,
        }
        or function.get("baseline_yaw_extension")
        != "exact emitted V9 root-yaw velocity plus the frozen stencil applied only to continuous nearest-branch yaw deltas"
        or function.get("tangential_constraint")
        != "velocity_x_m_s^2 + velocity_z_m_s^2 <= 0.12^2"
        or function.get("lever_probe_radians") != 0.0001
        or tolerances
        != {
            "function_identity_absolute_micrometres_per_second_per_component": 1.0,
            "jacobian_absolute_micrometres_per_second_per_variable_unit": 5.0,
            "jacobian_relative": 0.0001,
            "configuration_nonzero_micrometres_per_second_per_radian": 1.0,
            "tangent_chain_rule_absolute_square_metres_per_square_second_per_unit": 5e-06,
        }
        or probes
        != {
            "root_translation_metres": 1e-05,
            "root_orientation_radians": 0.0001,
            "joint_position_radians": 0.0001,
            "root_linear_velocity_metres_per_second": 1e-05,
            "root_angular_velocity_radians_per_second": 0.0001,
            "joint_velocity_radians_per_second": 0.0001,
        }
        or tuple(profile.get("deterministic_anchors", ()))
        != (
            "frame-328:left-forefoot",
            "frame-92:left-forefoot",
            "frame-165:left-forefoot",
            "frame-93:left-heel",
            "frame-179:right-heel",
            "frame-76:right-forefoot",
            "frame-1:right-heel",
        )
        or profile.get("decision", {}).get("pass")
        != "PERMIT_SEPARATE_REPORT_ONLY_R119_REPAIRED_KTO_EXECUTION_FORMULATION_ONLY"
        or profile.get("decision", {}).get("fail") != "STOP_AND_RESEARCH"
        or bounded.get("r119_repaired_kto_execution_formulation")
        != "AUTHORIZED_REPORT_ONLY_ON_R118_PASS"
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
        != (
            "ruff_check",
            "ruff_format",
            "solver_free_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("R118 conformance profile differs")
