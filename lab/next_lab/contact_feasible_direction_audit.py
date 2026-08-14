from __future__ import annotations

import hashlib
import json
import math
from collections import Counter
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, Mapping

import numpy as np
import osqp
import scipy
from numpy.typing import NDArray
from scipy import sparse

from next_lab import contact_manifold, contact_trajectory
from next_lab.contact_target_knot_formulation import (
    FRAME_FIRST,
    FRAME_LAST,
    KNOT_FRAME_OFFSETS,
    SOURCE_CASE_ORDINAL,
    _load_arrays,
    _load_case,
    _load_complete_v9_clip,
    _load_manifest,
    _target_array,
    _validate_matched_case,
    apply_anchor,
    array_sha256,
    canonical_json,
    interpolate_coefficients,
    sha256,
)
from next_lab.contact_target_knot_preflight import (
    CHECK_ID as R104_CHECK_ID,
    PREFLIGHT_ID as R104_PREFLIGHT_ID,
    _load_artifact,
    _validate_complete_arrays,
    _validate_r103,
)
from next_lab.motion_math import target_effectors, target_forward_kinematics
from scripts.build_contact_manifold_prototype import (
    _tolerances,
    _trajectory_closure,
)


AUDIT_ID = "nextengine.humanoid-contact-feasible-direction-audit.v1"
CHECK_ID = "TRAIN-4-CONTACT-FEASIBLE-DIRECTION-AUDIT"
BASIS_COEFFICIENTS = (
    (10_000, 0, 0),
    (0, 10_000, 0),
    (0, 0, 10_000),
)
BASIS_IDS = ("anchor-offset-02", "anchor-offset-06", "anchor-offset-11")


@dataclass(frozen=True)
class Linearization:
    matrix: sparse.csc_matrix
    lower: NDArray[np.float64]
    upper: NDArray[np.float64]
    row_categories: tuple[str, ...]
    local_scale: NDArray[np.float64]
    selected_dof_ordinals: NDArray[np.int64]
    selected_joint_ids: tuple[str, ...]
    local_variable_count: int
    frame_count: int


def build_feasible_direction_audit(
    *,
    profile_path: Path,
    source_audit_path: Path,
    r103_report_path: Path,
    r104_report_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
    v9_profile_path: Path,
    descriptor_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Audit R103 bases against the locally relevant public-limit V9 rows."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            source_audit_path,
            r103_report_path,
            r104_report_path,
            v7_manifest_path,
            v9_manifest_path,
            v9_profile_path,
            descriptor_path,
            tool_path,
        )
    )
    (
        profile_path,
        source_audit_path,
        r103_report_path,
        r104_report_path,
        v7_manifest_path,
        v9_manifest_path,
        v9_profile_path,
        descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("feasible-direction audit input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    r103 = _validate_r103(
        report_path=r103_report_path,
        expected=profile["source"]["r103"],
    )
    r104 = _validate_r104(
        report_path=r104_report_path,
        expected=profile["source"]["r104"],
    )
    if (
        r104["identities"]["r103_report_sha256"] != r103["report_sha256"]
        or sha256(source_audit_path) != profile["source"]["audit_sha256"]
        or sha256(v9_profile_path)
        != profile["source"]["v9_profile_sha256"]
        or sha256(descriptor_path)
        != profile["source"]["descriptor_sha256"]
    ):
        raise ValueError("feasible-direction source identity differs")

    v7_manifest = _load_manifest(
        path=v7_manifest_path,
        expected=profile["source"]["v7"],
        source_audit_path=source_audit_path,
    )
    v9_manifest = _load_manifest(
        path=v9_manifest_path,
        expected=profile["source"]["v9"],
        source_audit_path=source_audit_path,
    )
    v7_case, v7_artifact = _load_case(
        manifest=v7_manifest,
        manifest_path=v7_manifest_path,
        expected=profile["source"]["v7"],
    )
    v9_case, v9_artifact = _load_case(
        manifest=v9_manifest,
        manifest_path=v9_manifest_path,
        expected=profile["source"]["v9"],
    )
    _validate_matched_case(v7_case, v9_case)
    complete_record, complete_artifact = _load_complete_v9_clip(
        manifest=v9_manifest,
        manifest_path=v9_manifest_path,
        expected=profile["source"]["v9"],
    )

    v7_target = _target_array(_load_arrays(v7_artifact))
    v9_target = _target_array(_load_arrays(v9_artifact))
    anchor_delta = v7_target - v9_target
    complete_arrays, complete_metadata = _load_artifact(complete_artifact)
    frame_count = int(profile["scope"]["complete_clip_frame_count"])
    _validate_complete_arrays(
        arrays=complete_arrays,
        metadata=complete_metadata,
        frame_count=frame_count,
        v9_target=v9_target,
    )
    descriptor = json.loads(descriptor_path.read_bytes())
    v9_profile = json.loads(v9_profile_path.read_bytes())
    tolerances = _tolerances(v9_profile["projection"])
    closure = _trajectory_closure(v9_profile["projection"])
    if closure is None:
        raise ValueError("V9 coupled-trajectory closure is absent")

    linearization = build_public_limit_linearization(
        descriptor=descriptor,
        effector_ids=tuple(complete_metadata["effector_ids"]),
        arrays=complete_arrays,
        tolerances=tolerances,
        closure=closure,
    )
    complete_zero_violation = maximum_constraint_violation(
        matrix=linearization.matrix,
        lower=linearization.lower,
        upper=linearization.upper,
        value=np.zeros(linearization.matrix.shape[1], dtype=np.float64),
    )
    feasibility_tolerance = float(profile["analysis"]["feasibility_tolerance"])
    support_first = int(profile["analysis"]["support_frame_first"])
    support_last = int(profile["analysis"]["support_frame_last"])
    support_columns = _support_columns(
        frame_first=support_first,
        frame_last=support_last,
        local_variable_count=linearization.local_variable_count,
    )
    local_matrix = linearization.matrix[:, support_columns].tocsc()
    relevant_rows = np.flatnonzero(np.diff(local_matrix.tocsr().indptr) > 0)
    local_matrix = local_matrix[relevant_rows].tocsc()
    local_lower = linearization.lower[relevant_rows]
    local_upper = linearization.upper[relevant_rows]
    local_categories = tuple(
        linearization.row_categories[int(row)] for row in relevant_rows
    )
    zero_violation = maximum_constraint_violation(
        matrix=local_matrix,
        lower=local_lower,
        upper=local_upper,
        value=np.zeros(len(support_columns), dtype=np.float64),
    )
    if zero_violation > feasibility_tolerance:
        raise ValueError("V9 zero direction violates the public-limit row system")

    row_analysis = analyze_binding_rows(
        matrix=local_matrix,
        lower=local_lower,
        upper=local_upper,
        row_categories=local_categories,
        binding_tolerance=float(
            profile["analysis"]["binding_slack_tolerance_normalized"]
        ),
        near_binding_tolerance=float(
            profile["analysis"]["near_binding_slack_normalized"]
        ),
        rank_relative_tolerance=float(
            profile["analysis"]["rank_relative_tolerance"]
        ),
    )

    r103_basis = {
        tuple(int(value) for value in row["knot_coefficients_basis_points"]): row
        for row in r103["candidate_lattice"]
    }
    basis_rows = []
    for basis_id, coefficients in zip(
        BASIS_IDS, BASIS_COEFFICIENTS, strict=True
    ):
        alpha = interpolate_coefficients(coefficients)
        basis_target = apply_anchor(v9_target, anchor_delta, alpha)
        if (
            coefficients not in r103_basis
            or array_sha256(basis_target)
            != r103_basis[coefficients]["target_sha256"]
        ):
            raise ValueError("R103 anchor basis identity differs")
        raw = build_raw_basis(
            coefficients=coefficients,
            v9_target=v9_target,
            anchor_delta=anchor_delta,
            support_frame_first=support_first,
            support_frame_last=support_last,
            local_variable_count=linearization.local_variable_count,
            local_scale=linearization.local_scale,
            selected_dof_ordinals=linearization.selected_dof_ordinals,
        )
        raw_violations = constraint_violations(
            matrix=local_matrix,
            lower=local_lower,
            upper=local_upper,
            value=raw,
            row_categories=local_categories,
            tolerance=float(profile["analysis"]["feasibility_tolerance"]),
        )
        projected, solve = project_direction(
            raw=raw,
            matrix=local_matrix,
            lower=local_lower,
            upper=local_upper,
            solver=profile["analysis"]["solver"],
        )
        metrics = projection_metrics(
            raw=raw,
            projected=projected,
            local_scale=linearization.local_scale,
            support_frame_count=support_last - support_first + 1,
        )
        projected_violation = maximum_constraint_violation(
            matrix=local_matrix,
            lower=local_lower,
            upper=local_upper,
            value=projected,
        )
        useful = projection_is_useful(
            metrics=metrics,
            thresholds=profile["analysis"]["useful_projection"],
        )
        basis_rows.append(
            {
                "basis_id": basis_id,
                "knot_coefficients_basis_points": list(coefficients),
                "raw_direction": {
                    "normalized_l2_norm": _finite_float(np.linalg.norm(raw)),
                    "violating_row_count": raw_violations["row_count"],
                    "violation_category_counts": raw_violations[
                        "category_counts"
                    ],
                    "maximum_normalized_violation": raw_violations[
                        "maximum_normalized_violation"
                    ],
                },
                "projection": metrics,
                "projection_solve": solve,
                "maximum_projected_normalized_constraint_violation": (
                    _finite_float(projected_violation)
                ),
                "useful_projection": useful,
                "candidate_artifact": "NOT_EMITTED",
                "exact_nonlinear_candidate_audit": "NOT_RUN",
            }
        )

    useful_ids = [
        row["basis_id"] for row in basis_rows if row["useful_projection"]
    ]
    decision = (
        profile["decision"]["useful_projection"]
        if useful_ids
        else profile["decision"]["projection_collapse"]
    )
    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": decision,
        "audit_id": AUDIT_ID,
        "scope": profile["scope"],
        "method": profile["method"],
        "analysis_contract": profile["analysis"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "source_audit_sha256": sha256(source_audit_path),
            "r103_report_sha256": r103["report_sha256"],
            "r103_report_file_sha256": sha256(r103_report_path),
            "r104_report_sha256": r104["report_sha256"],
            "r104_report_file_sha256": sha256(r104_report_path),
            "v7_manifest_sha256": v7_manifest["manifest_sha256"],
            "v7_manifest_file_sha256": sha256(v7_manifest_path),
            "v7_case_artifact_sha256": sha256(v7_artifact),
            "v9_manifest_sha256": v9_manifest["manifest_sha256"],
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "v9_case_artifact_sha256": sha256(v9_artifact),
            "v9_complete_clip_artifact_sha256": sha256(complete_artifact),
            "v9_profile_sha256": sha256(v9_profile_path),
            "descriptor_sha256": sha256(descriptor_path),
            "numpy_version": np.__version__,
            "scipy_version": scipy.__version__,
            "osqp_version": osqp.__version__,
            "contact_trajectory_module_sha256": sha256(
                Path(contact_trajectory.__file__).resolve()
            ),
            "tool_sha256": sha256(tool_path),
            "audit_module_sha256": sha256(Path(__file__).resolve()),
        },
        "source_records": {
            "clip_id": complete_record["clip_id"],
            "source_case_ordinal": SOURCE_CASE_ORDINAL,
            "frame_first": FRAME_FIRST,
            "frame_last": FRAME_LAST,
        },
        "linearization": {
            "complete_clip_variable_count": int(
                linearization.matrix.shape[1]
            ),
            "complete_clip_constraint_count": int(
                linearization.matrix.shape[0]
            ),
            "complete_clip_constraint_nonzero_count": int(
                linearization.matrix.nnz
            ),
            "local_variable_count_per_frame": (
                linearization.local_variable_count
            ),
            "local_support_variable_count": len(support_columns),
            "local_relevant_constraint_count": len(relevant_rows),
            "selected_joint_ids": list(linearization.selected_joint_ids),
            "selected_dof_ordinals": (
                linearization.selected_dof_ordinals.tolist()
            ),
            "complete_clip_proxy_zero_direction_maximum_normalized_violation": (
                _finite_float(complete_zero_violation)
            ),
            "complete_clip_proxy_acceptance_authority": False,
            "zero_direction_maximum_normalized_violation": (
                _finite_float(zero_violation)
            ),
            "binding_rows": row_analysis,
        },
        "basis_results": basis_rows,
        "summary": {
            "basis_count": len(basis_rows),
            "raw_violating_basis_count": sum(
                row["raw_direction"]["violating_row_count"] > 0
                for row in basis_rows
            ),
            "useful_projection_count": len(useful_ids),
            "useful_projection_ids": useful_ids,
            "projection_collapse_count": len(basis_rows) - len(useful_ids),
        },
        "bounded_acceptance": {
            "r106_exact_offline_formulation": (
                "AUTHORIZED"
                if useful_ids
                else "NOT_AUTHORIZED"
            ),
            "candidate_artifact": "NOT_AUTHORIZED",
            "exact_nonlinear_candidate_audit": "NOT_AUTHORIZED",
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
        "basis_constructions": len(basis_rows),
        "local_projection_qp_solves": len(basis_rows),
        "candidate_target_constructions": 0,
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": 0,
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def build_public_limit_linearization(
    *,
    descriptor: Mapping[str, Any],
    effector_ids: tuple[str, ...],
    arrays: Mapping[str, NDArray[Any]],
    tolerances: Any,
    closure: Any,
) -> Linearization:
    """Rebuild V9's row/Jacobian family with public, not solver-margin, limits."""

    root_position = np.asarray(arrays["root_position_um"], dtype=np.int64)
    root_quaternion = np.asarray(
        arrays["root_quaternion_q1_30"], dtype=np.int64
    )
    root_yaw_velocity = np.asarray(
        arrays["root_yaw_velocity_urad_s"], dtype=np.int64
    )
    joint_position = np.asarray(arrays["joint_position_urad"], dtype=np.int64)
    modes = np.asarray(arrays["contact_modes"], dtype=np.uint8)
    frame_count, joint_count = joint_position.shape
    if (
        root_position.shape != (frame_count, 3)
        or root_quaternion.shape != (frame_count, 4)
        or root_yaw_velocity.shape != (frame_count,)
        or joint_count != len(descriptor.get("joints", ()))
    ):
        raise ValueError("feasible-direction linearization shape differs")

    active = contact_trajectory.contact_point_mask(modes)
    stencil_indices, stencil_coefficients = (
        contact_trajectory.hybrid_velocity_stencil(active)
    )
    sole_indices = contact_manifold._sole_effector_indices(effector_ids)
    joint_lookup = {
        str(joint["joint_id"]): int(joint["dof_ordinal"])
        for joint in descriptor["joints"]
    }
    selected_ids = tuple(
        f"joint.{side}-{suffix}"
        for side in ("left", "right")
        for suffix in closure.ordered_joint_suffixes
    )
    if any(joint_id not in joint_lookup for joint_id in selected_ids):
        raise ValueError("feasible-direction selected joint identity differs")
    selected = np.asarray(
        [joint_lookup[joint_id] for joint_id in selected_ids], dtype=np.int64
    )
    local_variable_count = 3 + len(selected)
    local_scale = np.concatenate(
        (
            np.full(
                3,
                closure.root_variable_scale_micrometres / 1_000_000.0,
                dtype=np.float64,
            ),
            np.full(
                len(selected),
                closure.joint_variable_scale_microradians / 1_000_000.0,
                dtype=np.float64,
            ),
        )
    )
    root = root_position.astype(np.float64) / 1_000_000.0
    joints = joint_position.astype(np.float64) / 1_000_000.0
    quaternions = root_quaternion.astype(np.float64) / float(1 << 30)
    all_colliders, _ = contact_manifold._collider_inventory(descriptor)
    collider_rows = contact_trajectory._collider_linearization_rows(
        all_colliders, closure.collider_linearization_policy
    )
    effectors = np.empty((frame_count, 2, 2, 3), dtype=np.float64)
    effector_jacobian = np.zeros(
        (frame_count, 2, 2, 3, local_variable_count), dtype=np.float64
    )
    effector_jacobian[..., 0, 0] = 1.0
    effector_jacobian[..., 1, 1] = 1.0
    effector_jacobian[..., 2, 2] = 1.0
    collider_heights = np.empty(
        (frame_count, len(collider_rows)), dtype=np.float64
    )
    collider_jacobian = np.zeros(
        (frame_count, len(collider_rows), local_variable_count), dtype=np.float64
    )
    collider_jacobian[..., 1] = 1.0
    probe = closure.jacobian_probe_microradians / 1_000_000.0
    for frame in range(frame_count):
        positions, rotations = target_forward_kinematics(
            descriptor, root[frame], quaternions[frame], joints[frame]
        )
        values = target_effectors(descriptor, positions, rotations)
        for side in range(2):
            for point in range(2):
                effectors[frame, side, point] = values[
                    effector_ids[int(sole_indices[side, point])]
                ]
        collider_heights[frame] = (
            contact_trajectory._collider_linearization_values(
                positions, rotations, collider_rows
            )
        )
        for local_joint, ordinal in enumerate(selected, start=3):
            candidate = joints[frame].copy()
            candidate[int(ordinal)] += probe
            candidate_positions, candidate_rotations = target_forward_kinematics(
                descriptor,
                root[frame],
                quaternions[frame],
                candidate,
            )
            candidate_effectors = target_effectors(
                descriptor, candidate_positions, candidate_rotations
            )
            for side in range(2):
                for point in range(2):
                    effector_jacobian[
                        frame, side, point, :, local_joint
                    ] = (
                        candidate_effectors[
                            effector_ids[int(sole_indices[side, point])]
                        ]
                        - effectors[frame, side, point]
                    ) / probe
            collider_jacobian[frame, :, local_joint] = (
                contact_trajectory._collider_linearization_values(
                    candidate_positions, candidate_rotations, collider_rows
                )
                - collider_heights[frame]
            ) / probe

    def velocity(values: NDArray[np.float64]) -> NDArray[np.float64]:
        weights = stencil_coefficients.reshape(
            (frame_count, 2) + (1,) * (values.ndim - 1)
        )
        return np.sum(values[stencil_indices] * weights, axis=1)

    root_velocity = np.rint(velocity(root_position.astype(np.float64))).astype(
        np.int64
    )
    joint_velocity = np.rint(
        velocity(joint_position.astype(np.float64))
    ).astype(np.int64)
    analytic_velocity = contact_manifold._analytic_active_point_velocities(
        descriptor=descriptor,
        root_positions=root,
        root_quaternions=quaternions,
        joint_positions=joints,
        root_linear_velocity_um_s=root_velocity,
        root_yaw_velocity_urad_s=root_yaw_velocity,
        joint_velocity_urad_s=joint_velocity,
        active=active,
        probe=probe,
    ) / 1_000_000.0

    joint_by_ordinal = {
        int(joint["dof_ordinal"]): joint for joint in descriptor["joints"]
    }
    bound_map = {
        joint_id: closure.joint_bounds_microradians[side][column]
        for side, side_name in enumerate(("left", "right"))
        for column, joint_id in enumerate(
            f"joint.{side_name}-{suffix}"
            for suffix in closure.ordered_joint_suffixes
        )
    }
    joint_minimum = np.asarray(
        [bound_map[joint_id][0] for joint_id in selected_ids], dtype=np.float64
    ) / 1_000_000.0
    joint_maximum = np.asarray(
        [bound_map[joint_id][1] for joint_id in selected_ids], dtype=np.float64
    ) / 1_000_000.0
    joint_velocity_bounds = np.asarray(
        [
            int(
                joint_by_ordinal[int(ordinal)][
                    "maximum_velocity_microradians_per_second"
                ]
            )
            * closure.joint_velocity_limit_basis_points
            / 10_000.0
            / 1_000_000.0
            for ordinal in selected
        ],
        dtype=np.float64,
    )
    public_closure = replace(
        closure,
        collider_target_margin_micrometres=(
            closure.minimum_collider_height_micrometres
        ),
        root_velocity_margin_micrometres_per_second=0,
        joint_velocity_margin_numerator=1,
        joint_velocity_margin_denominator=1,
        normal_residual_margin_micrometres=0,
        finite_normal_margin_micrometres=0,
        finite_tangential_margin_micrometres=0,
        analytic_normal_margin_micrometres_per_second=0,
        analytic_tangential_margin_micrometres_per_second=0,
    )
    matrix, lower, upper, categories = contact_trajectory._build_qp(
        frame_count=frame_count,
        local_variable_count=local_variable_count,
        variable_count=frame_count * local_variable_count,
        local_scale=local_scale,
        root_values=root,
        joint_values=joints,
        effectors=effectors,
        effector_jacobian=effector_jacobian,
        collider_heights=collider_heights,
        collider_jacobian=collider_jacobian,
        analytic_velocity=analytic_velocity,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        collider_target=(
            closure.minimum_collider_height_micrometres / 1_000_000.0
        ),
        root_velocity_bound=(
            closure.maximum_root_vertical_velocity_micrometres_per_second
            / 1_000_000.0
        ),
        joint_velocity_bounds=joint_velocity_bounds,
        selected_flat=selected,
        joint_minimum=joint_minimum,
        joint_maximum=joint_maximum,
        tolerances=tolerances,
        closure=public_closure,
    )
    row_categories = tuple(
        category
        for category, count in categories.items()
        for _ in range(count)
    )
    if len(row_categories) != matrix.shape[0]:
        raise AssertionError("linearization row categories differ")
    return Linearization(
        matrix=matrix,
        lower=lower,
        upper=upper,
        row_categories=row_categories,
        local_scale=local_scale,
        selected_dof_ordinals=selected,
        selected_joint_ids=selected_ids,
        local_variable_count=local_variable_count,
        frame_count=frame_count,
    )


def build_raw_basis(
    *,
    coefficients: tuple[int, int, int],
    v9_target: NDArray[np.int64],
    anchor_delta: NDArray[np.int64],
    support_frame_first: int,
    support_frame_last: int,
    local_variable_count: int,
    local_scale: NDArray[np.float64],
    selected_dof_ordinals: NDArray[np.int64],
) -> NDArray[np.float64]:
    alpha = interpolate_coefficients(coefficients)
    target = apply_anchor(v9_target, anchor_delta, alpha)
    target_delta = target - v9_target
    support_frame_count = support_frame_last - support_frame_first + 1
    value = np.zeros(
        (support_frame_count, local_variable_count), dtype=np.float64
    )
    for local_frame, source_frame in enumerate(
        range(support_frame_first, support_frame_last + 1)
    ):
        case_offset = source_frame - FRAME_FIRST
        value[local_frame, 3:] = (
            target_delta[case_offset, selected_dof_ordinals] / 1_000_000.0
        ) / local_scale[3:]
    return value.reshape(-1)


def analyze_binding_rows(
    *,
    matrix: sparse.csc_matrix,
    lower: NDArray[np.float64],
    upper: NDArray[np.float64],
    row_categories: tuple[str, ...],
    binding_tolerance: float,
    near_binding_tolerance: float,
    rank_relative_tolerance: float,
) -> dict[str, Any]:
    slack = np.minimum(-lower, upper)
    binding = np.flatnonzero(slack <= binding_tolerance)
    near = np.flatnonzero(slack <= near_binding_tolerance)
    near_matrix = matrix[near].toarray()
    norms = np.linalg.norm(near_matrix, axis=1)
    near_matrix = near_matrix[norms > 0.0]
    singular = (
        np.linalg.svd(near_matrix, compute_uv=False)
        if len(near_matrix)
        else np.empty(0, dtype=np.float64)
    )
    threshold = (
        float(singular[0]) * rank_relative_tolerance
        if len(singular)
        else 0.0
    )
    rank = int(np.count_nonzero(singular > threshold))
    condition = (
        float(singular[0] / singular[rank - 1]) if rank else None
    )
    return {
        "binding_row_count": len(binding),
        "near_binding_row_count": len(near),
        "near_binding_category_counts": _category_counts(
            row_categories, near
        ),
        "near_binding_rank": rank,
        "local_nullity": int(matrix.shape[1] - rank),
        "rank_threshold": _finite_float(threshold),
        "nonzero_spectrum_condition_number": (
            None if condition is None else _finite_float(condition)
        ),
        "largest_singular_value": (
            0.0 if not len(singular) else _finite_float(singular[0])
        ),
        "smallest_retained_singular_value": (
            0.0 if not rank else _finite_float(singular[rank - 1])
        ),
        "minimum_normalized_slack": _finite_float(float(np.min(slack))),
    }


def project_direction(
    *,
    raw: NDArray[np.float64],
    matrix: sparse.csc_matrix,
    lower: NDArray[np.float64],
    upper: NDArray[np.float64],
    solver: Mapping[str, Any],
) -> tuple[NDArray[np.float64], dict[str, Any]]:
    problem = osqp.OSQP()
    problem.setup(
        P=sparse.eye(len(raw), format="csc"),
        q=-raw,
        A=matrix,
        l=lower,
        u=upper,
        verbose=False,
        eps_abs=float(solver["absolute_tolerance"]),
        eps_rel=float(solver["relative_tolerance"]),
        max_iter=int(solver["maximum_iterations"]),
        polishing=bool(solver["polishing_enabled"]),
        adaptive_rho=bool(solver["adaptive_rho_enabled"]),
    )
    result = problem.solve(raise_error=False)
    status = str(result.info.status)
    if result.x is None or not status.lower().startswith("solved"):
        raise ValueError(f"feasible-direction projection failed: {status}")
    projected = np.asarray(result.x, dtype=np.float64)
    return projected, {
        "status": status,
        "iterations": int(result.info.iter),
        "primal_residual": _finite_float(float(result.info.prim_res)),
        "dual_residual": _finite_float(float(result.info.dual_res)),
    }


def projection_metrics(
    *,
    raw: NDArray[np.float64],
    projected: NDArray[np.float64],
    local_scale: NDArray[np.float64],
    support_frame_count: int,
) -> dict[str, Any]:
    raw_norm = float(np.linalg.norm(raw))
    projected_norm = float(np.linalg.norm(projected))
    dot = float(np.dot(raw, projected))
    component = 0.0 if raw_norm == 0.0 else dot / (raw_norm * raw_norm)
    cosine = (
        0.0
        if raw_norm == 0.0 or projected_norm == 0.0
        else dot / (raw_norm * projected_norm)
    )
    physical = projected.reshape(support_frame_count, -1) * local_scale[None, :]
    root_um = np.rint(physical[:, :3] * 1_000_000.0).astype(np.int64)
    joint_urad = np.rint(physical[:, 3:] * 1_000_000.0).astype(np.int64)
    return {
        "normalized_l2_norm": _finite_float(projected_norm),
        "normalized_distance_from_raw": _finite_float(
            np.linalg.norm(projected - raw)
        ),
        "anchor_component_basis_points": int(round(component * 10_000.0)),
        "cosine_similarity_basis_points": int(round(cosine * 10_000.0)),
        "quantized_nonzero_joint_cell_count": int(np.count_nonzero(joint_urad)),
        "quantized_maximum_absolute_joint_correction_microradians": int(
            np.max(np.abs(joint_urad), initial=0)
        ),
        "quantized_maximum_root_correction_micrometres": int(
            np.max(np.abs(root_um), initial=0)
        ),
    }


def projection_is_useful(
    *, metrics: Mapping[str, Any], thresholds: Mapping[str, Any]
) -> bool:
    return (
        int(metrics["anchor_component_basis_points"])
        >= int(thresholds["minimum_anchor_component_basis_points"])
        and int(metrics["cosine_similarity_basis_points"])
        >= int(thresholds["minimum_cosine_similarity_basis_points"])
        and int(metrics["quantized_nonzero_joint_cell_count"])
        >= int(thresholds["minimum_quantized_nonzero_joint_cell_count"])
    )


def constraint_violations(
    *,
    matrix: sparse.csc_matrix,
    lower: NDArray[np.float64],
    upper: NDArray[np.float64],
    value: NDArray[np.float64],
    row_categories: tuple[str, ...],
    tolerance: float,
) -> dict[str, Any]:
    product = np.asarray(matrix @ value).reshape(-1)
    violation = np.maximum(np.maximum(lower - product, product - upper), 0.0)
    rows = np.flatnonzero(violation > tolerance)
    return {
        "row_count": len(rows),
        "category_counts": _category_counts(row_categories, rows),
        "maximum_normalized_violation": _finite_float(
            float(np.max(violation, initial=0.0))
        ),
    }


def maximum_constraint_violation(
    *,
    matrix: sparse.csc_matrix,
    lower: NDArray[np.float64],
    upper: NDArray[np.float64],
    value: NDArray[np.float64],
) -> float:
    product = np.asarray(matrix @ value).reshape(-1)
    return float(
        np.max(
            np.maximum(np.maximum(lower - product, product - upper), 0.0),
            initial=0.0,
        )
    )


def _support_columns(
    *, frame_first: int, frame_last: int, local_variable_count: int
) -> NDArray[np.int64]:
    return np.asarray(
        [
            frame * local_variable_count + local
            for frame in range(frame_first, frame_last + 1)
            for local in range(local_variable_count)
        ],
        dtype=np.int64,
    )


def _category_counts(
    row_categories: tuple[str, ...], rows: NDArray[np.int64]
) -> dict[str, int]:
    return dict(
        sorted(Counter(row_categories[int(row)] for row in rows).items())
    )


def _validate_profile(profile: Mapping[str, Any]) -> None:
    analysis = profile.get("analysis", {})
    solver = analysis.get("solver", {})
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("audit_id") != AUDIT_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim") != "OptimizerFreeLocalFeasibleDirectionAudit"
        or profile.get("scope", {}).get("source_case_ordinal")
        != SOURCE_CASE_ORDINAL
        or profile.get("scope", {}).get("frame_first") != FRAME_FIRST
        or profile.get("scope", {}).get("frame_last") != FRAME_LAST
        or profile.get("scope", {}).get("complete_clip_frame_count") != 801
        or profile.get("scope", {}).get("basis_count") != 3
        or analysis.get("support_frame_first") != FRAME_FIRST + 2
        or analysis.get("support_frame_last") != FRAME_LAST
        or analysis.get("basis_coefficients_basis_points")
        != [list(value) for value in BASIS_COEFFICIENTS]
        or analysis.get("variable_scope")
        != "root XYZ plus V9 selected bilateral ten leg DoFs"
        or analysis.get("public_limits_only") is not True
        or analysis.get("binding_slack_tolerance_normalized") != 1.0e-9
        or analysis.get("near_binding_slack_normalized") != 0.05
        or analysis.get("rank_relative_tolerance") != 1.0e-10
        or analysis.get("feasibility_tolerance") != 1.0e-7
        or analysis.get("useful_projection")
        != {
            "minimum_anchor_component_basis_points": 1000,
            "minimum_cosine_similarity_basis_points": 5000,
            "minimum_quantized_nonzero_joint_cell_count": 1,
        }
        or solver.get("backend_id") != "osqp"
        or solver.get("runtime_versions")
        != {
            "numpy": np.__version__,
            "scipy": scipy.__version__,
            "osqp": osqp.__version__,
        }
        or solver.get("maximum_iterations") != 200_000
        or solver.get("absolute_tolerance") != 1.0e-9
        or solver.get("relative_tolerance") != 1.0e-9
        or solver.get("polishing_enabled") is not True
        or solver.get("adaptive_rho_enabled") is not False
        or profile.get("method", {}).get("candidate_artifact_policy")
        != "emit report only; no candidate target or corpus artifact"
        or decision
        != {
            "useful_projection": (
                "PERMIT_R106_EXACT_OFFLINE_PROJECTED_DIRECTION_FORMULATION_ONLY"
            ),
            "projection_collapse": "SELECT_PROGRESSIVE_KINODYNAMIC_FORMULATION",
        }
    ):
        raise ValueError("feasible-direction audit profile is invalid")


def _validate_r104(
    *, report_path: Path, expected: Mapping[str, Any]
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R104 report file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    summary = report.get("summary", {})
    if (
        report.get("check") != R104_CHECK_ID
        or report.get("preflight_id") != R104_PREFLIGHT_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision") != "STOP_AND_RESEARCH"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("identities", {}).get("profile_sha256")
        != expected.get("profile_sha256")
        or summary.get("candidate_count") != 27
        or summary.get("nonzero_pass_count") != 0
        or summary.get("nonzero_fail_count") != 26
        or report.get("bounded_acceptance", {}).get(
            "r105_native_candidate_evaluation"
        )
        != "NOT_AUTHORIZED"
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "candidate_artifacts_built",
                "physx_runs",
                "candidate_evaluations",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R104 preflight contract differs")
    return report


def _finite_float(value: float) -> float:
    if not math.isfinite(value):
        raise ValueError("non-finite feasible-direction metric")
    return float(f"{value:.12g}")
