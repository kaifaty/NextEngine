from __future__ import annotations

import math
import resource
import time
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import Any

import numpy as np
import osqp
from numpy.typing import NDArray
from scipy import sparse

from next_lab import contact_manifold
from next_lab import quantization_aware_kto_solver as inherited
from next_lab.kto_linearization_repair_conformance import (
    AnchorDefinition,
    VariableSpec,
    audit_all_active_function_identity,
    audit_anchor_jacobian,
    nearest_yaw_delta,
    repaired_exact_contact_velocity,
    rotation_exp,
    select_anchor_definitions,
    variable_specs,
)
from next_lab.kto_linearization_repair_conformance import (
    hybrid_velocity_stencil as repaired_hybrid_velocity_stencil,
)
from next_lab.motion_math import (
    collider_minimum_y,
    matrix_to_quaternion,
    q1_30,
    quaternion_to_matrix,
    rotation_axis,
    target_center_of_mass,
    target_effectors,
    target_forward_kinematics,
)
from next_lab.quantization_aware_kto_formulation import classify_tracking_progress

FRAME_COUNT = inherited.FRAME_COUNT
JOINT_COUNT = inherited.JOINT_COUNT
Q_WIDTH = inherited.Q_WIDTH
BLOCK_WIDTH = inherited.BLOCK_WIDTH
ROOT_POSITION = inherited.ROOT_POSITION
ROOT_ORIENTATION = inherited.ROOT_ORIENTATION
JOINT_POSITION = inherited.JOINT_POSITION
FRACTIONS = ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
NORMAL_BOUND_M_S = 0.06
TANGENT_BOUND_M_S = 0.12
TANGENT_BOUND_SQUARED = TANGENT_BOUND_M_S**2
COEFFICIENT_ELISION = 1.0e-14

KtoState = inherited.KtoState
KtoLinearization = inherited.KtoLinearization


@dataclass(frozen=True)
class ContactRowLinearization:
    anchor: AnchorDefinition
    variables: tuple[VariableSpec, ...]
    global_columns: NDArray[np.int64]
    velocity_m_s: NDArray[np.float64]
    vector_jacobian_m_s: NDArray[np.float64]


@dataclass(frozen=True)
class RepairedKtoProblem:
    objective: sparse.csc_matrix
    objective_linear: NDArray[np.float64]
    constraints: sparse.csc_matrix
    lower: NDArray[np.float64]
    upper: NDArray[np.float64]
    variable_scale: NDArray[np.float64]
    constraint_categories: dict[str, int]
    constraint_category_nonzeros: dict[str, int]


@dataclass(frozen=True)
class EmittedAudit:
    report: dict[str, Any]
    arrays: dict[str, NDArray[Any]]
    analytic_velocity_um_s: NDArray[np.float64]


class _ContactKernel:
    """Cached, semantics-preserving evaluator for one R118 contact function."""

    def __init__(
        self,
        *,
        descriptor: dict[str, Any],
        arrays: Mapping[str, NDArray[Any]],
        anchor: AnchorDefinition,
        exact_probe: float,
    ) -> None:
        self.descriptor = descriptor
        self.arrays = arrays
        self.anchor = anchor
        self.exact_probe = exact_probe
        self.required_frames = {anchor.frame, *anchor.stencil_frames}
        self.rotations = {
            frame: quaternion_to_matrix(
                arrays["root_quaternion_q1_30"][frame].astype(np.float64)
                / float(1 << 30)
            )
            for frame in self.required_frames
        }
        self.root = (
            arrays["root_position_um"][anchor.frame].astype(np.float64) / 1_000_000.0
        )
        self.joints = (
            arrays["joint_position_urad"][anchor.frame].astype(np.float64) / 1_000_000.0
        )
        self.root_velocity_um_s = arrays["root_linear_velocity_um_s"][
            anchor.frame
        ].astype(np.float64)
        self.root_yaw_velocity_rad_s = (
            float(arrays["root_yaw_velocity_urad_s"][anchor.frame]) / 1_000_000.0
        )
        self.joint_velocity_rad_s = (
            arrays["joint_velocity_urad_s"][anchor.frame].astype(np.float64)
            / 1_000_000.0
        )
        self.leg_ordinals = tuple(
            int(value)
            for value in contact_manifold._leg_joint_ordinals(descriptor)[anchor.side]
        )
        side = ("left", "right")[anchor.side]
        point = ("heel", "forefoot")[anchor.point]
        self.point_name = f"effector.{side}-{point}"
        self._zero_levers = self._levers(
            current_orientation=np.zeros(3, dtype=np.float64),
            joint_delta=np.zeros(JOINT_COUNT, dtype=np.float64),
        )

    def _point(
        self,
        rotation: NDArray[np.float64],
        joints: NDArray[np.float64],
    ) -> NDArray[np.float64]:
        positions, rotations = target_forward_kinematics(
            self.descriptor,
            self.root,
            matrix_to_quaternion(rotation),
            joints,
        )
        return target_effectors(self.descriptor, positions, rotations)[self.point_name]

    def _levers(
        self,
        *,
        current_orientation: NDArray[np.float64],
        joint_delta: NDArray[np.float64],
    ) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
        rotation = rotation_exp(current_orientation) @ self.rotations[self.anchor.frame]
        joints = self.joints + joint_delta
        baseline = self._point(rotation, joints)
        yaw_point = self._point(
            rotation_axis("Y", self.exact_probe) @ rotation,
            joints,
        )
        yaw = (yaw_point - baseline) * (1_000_000.0 / self.exact_probe)
        joint = np.empty((len(self.leg_ordinals), 3), dtype=np.float64)
        for slot, ordinal in enumerate(self.leg_ordinals):
            candidate = joints.copy()
            candidate[ordinal] += self.exact_probe
            joint[slot] = (self._point(rotation, candidate) - baseline) * (
                1_000_000.0 / self.exact_probe
            )
        return yaw, joint

    def evaluate(
        self,
        *,
        orientation: Mapping[int, NDArray[np.float64]] | None = None,
        joint_delta: NDArray[np.float64] | None = None,
        root_linear_delta: NDArray[np.float64] | None = None,
        joint_velocity_delta: NDArray[np.float64] | None = None,
    ) -> NDArray[np.float64]:
        orientation = orientation or {}
        current_orientation = np.asarray(
            orientation.get(self.anchor.frame, np.zeros(3)), dtype=np.float64
        )
        joint_delta = (
            np.zeros(JOINT_COUNT, dtype=np.float64)
            if joint_delta is None
            else np.asarray(joint_delta, dtype=np.float64)
        )
        if np.any(current_orientation) or np.any(joint_delta):
            yaw_lever, joint_levers = self._levers(
                current_orientation=current_orientation,
                joint_delta=joint_delta,
            )
        else:
            yaw_lever, joint_levers = self._zero_levers
        yaw_rate = self.root_yaw_velocity_rad_s
        for frame, coefficient in zip(
            self.anchor.stencil_frames,
            self.anchor.stencil_coefficients,
            strict=True,
        ):
            delta = np.asarray(orientation.get(frame, np.zeros(3)), dtype=np.float64)
            candidate = rotation_exp(delta) @ self.rotations[frame]
            yaw_rate += coefficient * nearest_yaw_delta(
                candidate, self.rotations[frame]
            )
        result = self.root_velocity_um_s.copy()
        if root_linear_delta is not None:
            result += np.asarray(root_linear_delta, dtype=np.float64) * 1_000_000.0
        result += yaw_lever * yaw_rate
        velocity = self.joint_velocity_rad_s.copy()
        if joint_velocity_delta is not None:
            velocity += np.asarray(joint_velocity_delta, dtype=np.float64)
        for lever, ordinal in zip(joint_levers, self.leg_ordinals, strict=True):
            result += lever * velocity[ordinal]
        return result


def optimized_contact_velocity_and_jacobian(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    anchor: AnchorDefinition,
    probes: Mapping[str, Any],
    exact_probe: float,
) -> tuple[tuple[VariableSpec, ...], NDArray[np.float64], NDArray[np.float64]]:
    """Evaluate the R118 function and omit only independently tiny/zero work."""

    specs = variable_specs(descriptor, anchor, profile_probes=probes)
    kernel = _ContactKernel(
        descriptor=descriptor,
        arrays=arrays,
        anchor=anchor,
        exact_probe=exact_probe,
    )
    baseline = kernel.evaluate()
    jacobian = np.zeros((3, len(specs)), dtype=np.float64)
    leg_ordinals = set(kernel.leg_ordinals)
    for column, spec in enumerate(specs):
        if spec.kind == "root_linear_velocity":
            jacobian[spec.index, column] = 1_000_000.0
            continue
        if spec.kind == "joint_velocity":
            if spec.index in leg_ordinals:
                slot = kernel.leg_ordinals.index(spec.index)
                jacobian[:, column] = kernel._zero_levers[1][slot]
            continue
        if spec.kind in ("root_translation", "root_angular_velocity"):
            continue
        if spec.kind == "joint_position" and spec.index not in leg_ordinals:
            continue

        orientation_plus: dict[int, NDArray[np.float64]] = {}
        orientation_minus: dict[int, NDArray[np.float64]] = {}
        joint_plus = np.zeros(JOINT_COUNT, dtype=np.float64)
        joint_minus = np.zeros(JOINT_COUNT, dtype=np.float64)
        if spec.kind == "root_orientation":
            orientation_plus[spec.frame] = np.zeros(3, dtype=np.float64)
            orientation_minus[spec.frame] = np.zeros(3, dtype=np.float64)
            orientation_plus[spec.frame][spec.index] = spec.probe
            orientation_minus[spec.frame][spec.index] = -spec.probe
        elif spec.kind == "joint_position":
            joint_plus[spec.index] = spec.probe
            joint_minus[spec.index] = -spec.probe
        else:
            raise ValueError(f"unsupported repaired contact variable {spec.kind!r}")
        high = kernel.evaluate(
            orientation=orientation_plus,
            joint_delta=joint_plus,
        )
        low = kernel.evaluate(
            orientation=orientation_minus,
            joint_delta=joint_minus,
        )
        jacobian[:, column] = (high - low) / (2.0 * spec.probe)
    return specs, baseline, jacobian


def build_repaired_contact_linearization(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    probes: Mapping[str, Any],
    exact_probe: float,
) -> tuple[ContactRowLinearization, ...]:
    rows = []
    for frame, side, point in np.argwhere(active):
        frame = int(frame)
        anchor = AnchorDefinition(
            role="r120-active-contact",
            frame=frame,
            side=int(side),
            point=int(point),
            stencil_frames=tuple(int(value) for value in stencil_indices[frame]),
            stencil_coefficients=tuple(
                float(value) for value in stencil_coefficients[frame]
            ),
        )
        specs, velocity_um_s, jacobian_um_s = optimized_contact_velocity_and_jacobian(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            probes=probes,
            exact_probe=exact_probe,
        )
        rows.append(
            ContactRowLinearization(
                anchor=anchor,
                variables=specs,
                global_columns=np.asarray(
                    [_global_column(spec) for spec in specs], dtype=np.int64
                ),
                velocity_m_s=velocity_um_s / 1_000_000.0,
                vector_jacobian_m_s=jacobian_um_s / 1_000_000.0,
            )
        )
    return tuple(rows)


def _global_column(spec: VariableSpec) -> int:
    if spec.kind == "root_translation":
        return inherited._column(spec.frame, 0, spec.index)
    if spec.kind == "root_orientation":
        return inherited._column(spec.frame, 0, 3 + spec.index)
    if spec.kind == "joint_position":
        return inherited._column(spec.frame, 0, 6 + spec.index)
    if spec.kind == "root_linear_velocity":
        return inherited._column(spec.frame, 1, spec.index)
    if spec.kind == "root_angular_velocity":
        return inherited._column(spec.frame, 1, 3 + spec.index)
    if spec.kind == "joint_velocity":
        return inherited._column(spec.frame, 1, 6 + spec.index)
    raise ValueError(f"unsupported repaired contact variable {spec.kind!r}")


def audit_local_conformance(
    *,
    descriptor: dict[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    r118_profile: Mapping[str, Any],
) -> dict[str, Any]:
    tolerances = r118_profile["numeric_tolerances"]
    exact_probe = float(r118_profile["function_contract"]["lever_probe_radians"])
    identity = audit_all_active_function_identity(
        descriptor=descriptor,
        arrays=arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        exact_probe=exact_probe,
        tolerance=float(
            tolerances[
                "function_identity_absolute_micrometres_per_second_per_component"
            ]
        ),
    )
    anchors = select_anchor_definitions(
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        expected=tuple(r118_profile["deterministic_anchors"]),
    )
    absolute = float(
        tolerances["jacobian_absolute_micrometres_per_second_per_variable_unit"]
    )
    relative = float(tolerances["jacobian_relative"])
    tangent_absolute = float(
        tolerances[
            "tangent_chain_rule_absolute_square_metres_per_square_second_per_unit"
        ]
    )
    summaries = []
    for anchor in anchors:
        reference = audit_anchor_jacobian(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            profile=r118_profile,
        )
        specs, velocity, jacobian = optimized_contact_velocity_and_jacobian(
            descriptor=descriptor,
            arrays=arrays,
            anchor=anchor,
            probes=r118_profile["variable_contract"]["probes"],
            exact_probe=exact_probe,
        )
        production = np.stack(
            [
                np.asarray(
                    row["production_xyz_micrometres_per_second_per_unit"],
                    dtype=np.float64,
                )
                for row in reference["variable_rows"]
            ],
            axis=1,
        )
        difference = np.abs(jacobian - production)
        allowed = absolute + relative * np.maximum(np.abs(jacobian), np.abs(production))
        baseline = np.asarray(
            reference["production_baseline_velocity_micrometres_per_second"],
            dtype=np.float64,
        )
        baseline_difference = float(np.max(np.abs(velocity - baseline)))
        optimized_chain = 2.0 * (
            (velocity[0] / 1_000_000.0) * (jacobian[0] / 1_000_000.0)
            + (velocity[2] / 1_000_000.0) * (jacobian[2] / 1_000_000.0)
        )
        production_chain = 2.0 * (
            (baseline[0] / 1_000_000.0) * (production[0] / 1_000_000.0)
            + (baseline[2] / 1_000_000.0) * (production[2] / 1_000_000.0)
        )
        chain_difference = np.abs(optimized_chain - production_chain)
        chain_allowed = tangent_absolute + relative * np.maximum(
            np.abs(optimized_chain), np.abs(production_chain)
        )
        passed = bool(
            reference["status"] == "PASS"
            and len(specs) == reference["variable_count"]
            and baseline_difference
            <= float(
                tolerances[
                    "function_identity_absolute_micrometres_per_second_per_component"
                ]
            )
            and np.all(difference <= allowed)
            and np.all(chain_difference <= chain_allowed)
        )
        summaries.append(
            {
                "status": "PASS" if passed else "FAIL",
                "point_id": anchor.point_id,
                "variable_count": len(specs),
                "r118_reference_status": reference["status"],
                "maximum_baseline_difference_micrometres_per_second": (
                    baseline_difference
                ),
                "maximum_vector_jacobian_difference_micrometres_per_second_per_unit": float(
                    np.max(difference)
                ),
                "maximum_vector_jacobian_tolerance_ratio": float(
                    np.max(difference / allowed)
                ),
                "maximum_tangent_chain_difference": float(np.max(chain_difference)),
                "maximum_tangent_chain_tolerance_ratio": float(
                    np.max(chain_difference / chain_allowed)
                ),
            }
        )
    passed = identity["status"] == "PASS" and all(
        row["status"] == "PASS" for row in summaries
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "all_active_function_identity": identity,
        "deterministic_anchor_count": len(summaries),
        "optimized_anchor_audits": summaries,
    }


def build_repaired_problem(
    *,
    descriptor: Mapping[str, Any],
    state: KtoState,
    initial_state: KtoState,
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    geometry: KtoLinearization,
    repaired_contact: Sequence[ContactRowLinearization],
    effective_limits: list[dict[str, Any]],
    r114_profile: Mapping[str, Any],
) -> RepairedKtoProblem:
    """Patch only R119's four allowed boundaries in the inherited R114 QP."""

    base = inherited._build_problem(
        descriptor=descriptor,
        state=state,
        v9_arrays=v9_arrays,
        v7_arrays=v7_arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        linearization=geometry,
        effective_limits=effective_limits,
        r114_profile=r114_profile,
    )
    category_ranges: dict[str, tuple[int, int]] = {}
    cursor = 0
    for name, count in base.constraint_categories.items():
        category_ranges[name] = (cursor, cursor + int(count))
        cursor += int(count)
    if cursor != base.constraints.shape[0]:
        raise ValueError("R120 inherited constraint inventory is not contiguous")
    analytic_range = category_ranges.get("contact_analytic_velocity")
    trust_range = category_ranges.get("endpoint_or_trust_bound")
    if (
        analytic_range is None
        or analytic_range[1] - analytic_range[0] != 3 * int(np.sum(active))
        or trust_range is None
        or trust_range[1] - trust_range[0] != FRAME_COUNT * Q_WIDTH
        or len(repaired_contact) != int(np.sum(active))
    ):
        raise ValueError("R120 inherited analytic/trust row boundary differs")

    lower = base.lower.copy()
    upper = base.upper.copy()
    q_offset = _configuration_offset(state=state, v9_arrays=v9_arrays)
    q_scale = np.asarray([0.02] * 3 + [0.1] * 26, dtype=np.float64)
    trust_cursor = trust_range[0]
    for frame in range(FRAME_COUNT):
        for local in range(Q_WIDTH):
            normalized = float(q_offset[frame, local] / q_scale[local])
            bound = 0.0 if frame in (0, FRAME_COUNT - 1) else 1.0
            lower[trust_cursor] = -bound - normalized
            upper[trust_cursor] = bound - normalized
            trust_cursor += 1

    objective_linear = base.objective_linear.copy()
    q_mean = 1.0 / (FRAME_COUNT * Q_WIDTH)
    a_mean = q_mean
    acceleration_scale = q_scale * 3600.0
    acceleration_offset = state.acceleration - initial_state.acceleration
    for frame in range(FRAME_COUNT):
        for local in range(Q_WIDTH):
            objective_linear[inherited._column(frame, 0, local)] += (
                2.0 * q_mean * q_offset[frame, local] / q_scale[local]
            )
            objective_linear[inherited._column(frame, 2, local)] += (
                2.0
                * a_mean
                * acceleration_offset[frame, local]
                / acceleration_scale[local]
            )
    frames = np.asarray(v7_arrays["reference_frame"], dtype=np.int64)
    v9_joint = np.asarray(v9_arrays["joint_position_urad"], dtype=np.int64)[frames]
    v7_joint = np.asarray(v7_arrays["joint_position_urad"], dtype=np.int64)
    baseline_squared = (
        r114_profile["tracking_progress_gate"][
            "v9_to_v7_squared_l2_distance_microradians_squared"
        ]
        / 1_000_000_000_000.0
    )
    for local_frame, frame in enumerate(frames):
        frame = int(frame)
        for joint in range(JOINT_COUNT):
            if int(v7_joint[local_frame, joint] - v9_joint[local_frame, joint]) == 0:
                continue
            current = float(q_offset[frame, 6 + joint])
            objective_linear[inherited._column(frame, 0, 6 + joint)] += (
                2.0 * q_scale[6 + joint] * current / baseline_squared
            )

    keep = np.ones(base.constraints.shape[0], dtype=np.bool_)
    keep[analytic_range[0] : analytic_range[1]] = False
    inherited_constraints = base.constraints[keep].tocsc()
    inherited_lower = lower[keep]
    inherited_upper = upper[keep]

    new_rows: list[int] = []
    new_columns: list[int] = []
    new_values: list[float] = []
    new_lower: list[float] = []
    new_upper: list[float] = []
    normal_nonzeros = 0
    tangent_nonzeros = 0

    def add_repaired_row(
        columns: NDArray[np.int64],
        coefficients: NDArray[np.float64],
        low: float,
        high: float,
        scale: float,
    ) -> int:
        row = len(new_lower)
        selected = np.abs(coefficients) > COEFFICIENT_ELISION
        selected_columns = columns[selected]
        selected_coefficients = coefficients[selected]
        if not np.all(np.isfinite(selected_coefficients)):
            raise ValueError("R120 repaired contact coefficient is nonfinite")
        for column, coefficient in zip(
            selected_columns, selected_coefficients, strict=True
        ):
            new_rows.append(row)
            new_columns.append(int(column))
            new_values.append(
                float(coefficient) * float(base.variable_scale[int(column)]) / scale
            )
        new_lower.append(low / scale)
        new_upper.append(high / scale)
        return int(np.count_nonzero(selected))

    for row in repaired_contact:
        normal = row.vector_jacobian_m_s[1]
        normal_nonzeros += add_repaired_row(
            row.global_columns,
            normal,
            -NORMAL_BOUND_M_S - float(row.velocity_m_s[1]),
            NORMAL_BOUND_M_S - float(row.velocity_m_s[1]),
            NORMAL_BOUND_M_S,
        )
    for row in repaired_contact:
        tangent = 2.0 * (
            float(row.velocity_m_s[0]) * row.vector_jacobian_m_s[0]
            + float(row.velocity_m_s[2]) * row.vector_jacobian_m_s[2]
        )
        current = float(row.velocity_m_s[0] ** 2 + row.velocity_m_s[2] ** 2)
        tangent_nonzeros += add_repaired_row(
            row.global_columns,
            tangent,
            -np.inf,
            TANGENT_BOUND_SQUARED - current,
            TANGENT_BOUND_SQUARED,
        )
    replacement = sparse.coo_matrix(
        (
            np.asarray(new_values, dtype=np.float64),
            (np.asarray(new_rows), np.asarray(new_columns)),
        ),
        shape=(len(new_lower), base.constraints.shape[1]),
    ).tocsc()
    constraints = sparse.vstack((inherited_constraints, replacement), format="csc")
    final_lower = np.concatenate(
        (inherited_lower, np.asarray(new_lower, dtype=np.float64))
    )
    final_upper = np.concatenate(
        (inherited_upper, np.asarray(new_upper, dtype=np.float64))
    )
    categories = {
        name: count
        for name, count in base.constraint_categories.items()
        if name != "contact_analytic_velocity"
    }
    categories["contact_analytic_normal"] = len(repaired_contact)
    categories["contact_analytic_tangential_norm_squared"] = len(repaired_contact)
    category_nonzeros = {}
    for name, (start, end) in category_ranges.items():
        if name != "contact_analytic_velocity":
            category_nonzeros[name] = int(base.constraints[start:end].nnz)
    category_nonzeros["contact_analytic_normal"] = normal_nonzeros
    category_nonzeros["contact_analytic_tangential_norm_squared"] = tangent_nonzeros
    if (
        constraints.shape[0] != sum(categories.values())
        or constraints.shape[1] != FRAME_COUNT * BLOCK_WIDTH
        or not np.all(np.isfinite(constraints.data))
        or not np.all(np.isfinite(final_lower[np.isfinite(final_lower)]))
        or not np.all(np.isfinite(final_upper[np.isfinite(final_upper)]))
        or np.any(final_lower > final_upper)
    ):
        raise ValueError("R120 repaired sparse problem is invalid")
    return RepairedKtoProblem(
        objective=base.objective,
        objective_linear=objective_linear,
        constraints=constraints,
        lower=final_lower,
        upper=final_upper,
        variable_scale=base.variable_scale,
        constraint_categories=categories,
        constraint_category_nonzeros=category_nonzeros,
    )


def _configuration_offset(
    *, state: KtoState, v9_arrays: Mapping[str, NDArray[Any]]
) -> NDArray[np.float64]:
    result = np.empty((FRAME_COUNT, Q_WIDTH), dtype=np.float64)
    result[:, ROOT_POSITION] = state.root_position_m - (
        np.asarray(v9_arrays["root_position_um"], dtype=np.float64) / 1_000_000.0
    )
    result[:, ROOT_ORIENTATION] = state.root_orientation_delta_rad
    result[:, JOINT_POSITION] = state.joint_position_rad - (
        np.asarray(v9_arrays["joint_position_urad"], dtype=np.float64) / 1_000_000.0
    )
    return result


def emit_and_audit(
    *,
    descriptor: dict[str, Any],
    state: KtoState,
    reference_rotations: NDArray[np.float64],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    effective_limits: list[dict[str, Any]],
    tolerances: contact_manifold.ContactManifoldTolerances,
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    identity_anchor_arrays: Mapping[str, NDArray[Any]] | None = None,
) -> EmittedAudit:
    source_root_m = (
        np.asarray(v9_arrays["root_position_um"], dtype=np.float64) / 1_000_000.0
    )
    source_joint_rad = (
        np.asarray(v9_arrays["joint_position_urad"], dtype=np.float64) / 1_000_000.0
    )
    maximum_root_step = float(
        np.max(np.linalg.norm(state.root_position_m - source_root_m, axis=1))
    )
    maximum_orientation_step = float(
        np.max(np.linalg.norm(state.root_orientation_delta_rad, axis=1))
    )
    maximum_joint_step = float(
        np.max(np.abs(state.joint_position_rad - source_joint_rad))
    )
    if (
        not all(
            np.all(np.isfinite(array))
            for array in (
                state.root_position_m,
                state.root_orientation_delta_rad,
                state.joint_position_rad,
                state.velocity,
                state.acceleration,
            )
        )
        or maximum_orientation_step > math.pi / 2.0
    ):
        return EmittedAudit(
            report={
                "status": "FAIL",
                "failure_reasons": ["nonfinite_or_orientation_chart"],
                "non_analytic_exact_gates_pass": False,
                "analytic_tangent_pass": False,
                "tangent_funnel": {"maximum": 1.0e300, "sum": 1.0e300},
                "emitted_hashes": {},
            },
            arrays={},
            analytic_velocity_um_s=np.empty((0,), dtype=np.float64),
        )
    root_um = np.rint(state.root_position_m * 1_000_000.0).astype(np.int64)
    joint_urad = np.rint(state.joint_position_rad * 1_000_000.0).astype(np.int64)
    rotations = inherited._compose_rotations(
        reference_rotations, state.root_orientation_delta_rad
    )
    if identity_anchor_arrays is not None:
        identity_root = np.asarray(
            identity_anchor_arrays["root_position_um"], dtype=np.int64
        )
        identity_joint = np.asarray(
            identity_anchor_arrays["joint_position_urad"], dtype=np.int64
        )
        identity_quaternion = np.asarray(
            identity_anchor_arrays["root_quaternion_q1_30"], dtype=np.int64
        )
        identity_rotations = np.stack(
            [
                quaternion_to_matrix(value.astype(np.float64) / float(1 << 30))
                for value in identity_quaternion
            ]
        )
        if (
            not np.array_equal(root_um, identity_root)
            or not np.array_equal(joint_urad, identity_joint)
            or float(np.max(np.abs(rotations - identity_rotations))) > 1.0e-10
        ):
            raise ValueError("R120 zero-coordinate identity anchor differs")
        root_um = identity_root.copy()
        joint_urad = identity_joint.copy()
        quaternion_q1_30 = identity_quaternion.copy()
        yaw_urad = np.asarray(
            identity_anchor_arrays["root_yaw_urad"], dtype=np.int64
        ).copy()
        root_velocity = np.asarray(
            identity_anchor_arrays["root_linear_velocity_um_s"], dtype=np.int64
        ).copy()
        joint_velocity = np.asarray(
            identity_anchor_arrays["joint_velocity_urad_s"], dtype=np.int64
        ).copy()
        yaw_velocity = np.asarray(
            identity_anchor_arrays["root_yaw_velocity_urad_s"], dtype=np.int64
        ).copy()
    else:
        quaternion_q1_30 = q1_30(
            np.stack([matrix_to_quaternion(rotation) for rotation in rotations])
        )
        yaw_urad = inherited._lift_root_yaw(
            emitted_quaternion_q1_30=quaternion_q1_30,
            source_quaternion_q1_30=np.asarray(
                v9_arrays["root_quaternion_q1_30"], dtype=np.int64
            ),
            source_yaw_urad=np.asarray(v9_arrays["root_yaw_urad"], dtype=np.int64),
        )
        root_velocity = np.rint(
            inherited._stencil_values(
                root_um.astype(np.float64), stencil_indices, stencil_coefficients
            )
        ).astype(np.int64)
        joint_velocity = np.rint(
            inherited._stencil_values(
                joint_urad.astype(np.float64), stencil_indices, stencil_coefficients
            )
        ).astype(np.int64)
        yaw_velocity = np.rint(
            inherited._stencil_values(
                yaw_urad[:, None].astype(np.float64),
                stencil_indices,
                stencil_coefficients,
            )[:, 0]
        ).astype(np.int64)
    metadata = inherited._metadata(v9_arrays)
    effector_ids = tuple(metadata["effector_ids"])
    effectors = np.empty((FRAME_COUNT, len(effector_ids), 3), dtype=np.float64)
    centers = np.empty((FRAME_COUNT, 3), dtype=np.float64)
    colliders, _ = contact_manifold._collider_inventory(descriptor)
    collider_heights = np.empty((FRAME_COUNT, len(colliders)), dtype=np.float64)
    decoded_quaternions = quaternion_q1_30.astype(np.float64) / float(1 << 30)
    quantized_root = root_um.astype(np.float64) / 1_000_000.0
    quantized_joints = joint_urad.astype(np.float64) / 1_000_000.0
    for frame in range(FRAME_COUNT):
        positions, body_rotations = target_forward_kinematics(
            descriptor,
            quantized_root[frame],
            decoded_quaternions[frame],
            quantized_joints[frame],
        )
        values = target_effectors(descriptor, positions, body_rotations)
        effectors[frame] = np.stack([values[value] for value in effector_ids])
        centers[frame] = target_center_of_mass(descriptor, positions, body_rotations)
        collider_heights[frame] = np.asarray(
            [
                collider_minimum_y(positions[slot], body_rotations[slot], collider)
                for slot, collider in colliders
            ]
        )
    effector_um = np.rint(effectors * 1_000_000.0).astype(np.int64)
    center_um = np.rint(centers * 1_000_000.0).astype(np.int64)
    modes = np.asarray(v9_arrays["contact_modes"], dtype=np.uint8)
    active = contact_manifold.contact_point_mask(modes)
    analytic = contact_manifold._analytic_active_point_velocities(
        descriptor=descriptor,
        root_positions=quantized_root,
        root_quaternions=decoded_quaternions,
        joint_positions=quantized_joints,
        root_linear_velocity_um_s=root_velocity,
        root_yaw_velocity_urad_s=yaw_velocity,
        joint_velocity_urad_s=joint_velocity,
        active=active,
        probe=1.0e-4,
    )
    contact = _contact_metrics(
        effector_ids=effector_ids,
        effector_position_um=effector_um,
        active=active,
        analytic_velocity_um_s=analytic,
        tolerances=tolerances,
    )
    maximum_joint_basis_points = 0
    effective_rom_violation = 0
    descriptor_rom_violation = 0
    joints = sorted(descriptor["joints"], key=lambda row: row["dof_ordinal"])
    for ordinal, joint in enumerate(joints):
        maximum_velocity = int(joint["maximum_velocity_microradians_per_second"])
        maximum_joint_basis_points = max(
            maximum_joint_basis_points,
            int(
                np.ceil(
                    np.max(np.abs(joint_velocity[:, ordinal]))
                    * 10_000.0
                    / maximum_velocity
                )
            ),
        )
        source_minimum, source_maximum = joint["soft_limit_microradians"]
        effective_minimum, effective_maximum = effective_limits[ordinal][
            "position_microradians"
        ]
        descriptor_rom_violation = max(
            descriptor_rom_violation,
            int(np.max(np.maximum(source_minimum - joint_urad[:, ordinal], 0))),
            int(np.max(np.maximum(joint_urad[:, ordinal] - source_maximum, 0))),
        )
        effective_rom_violation = max(
            effective_rom_violation,
            int(np.max(np.maximum(effective_minimum - joint_urad[:, ordinal], 0))),
            int(np.max(np.maximum(joint_urad[:, ordinal] - effective_maximum, 0))),
        )
    minimum_collider = int(np.floor(np.min(collider_heights) * 1_000_000.0 + 1e-9))
    maximum_root_vertical_velocity = int(np.max(np.abs(root_velocity[:, 1])))
    endpoints_equal = all(
        np.array_equal(candidate[[0, -1]], np.asarray(v9_arrays[name])[[0, -1]])
        for name, candidate in (
            ("root_position_um", root_um),
            ("root_quaternion_q1_30", quaternion_q1_30),
            ("joint_position_urad", joint_urad),
        )
    )
    frames = np.asarray(v7_arrays["reference_frame"], dtype=np.int64)
    progress = classify_tracking_progress(
        v9_joint_position_urad=np.asarray(
            v9_arrays["joint_position_urad"], dtype=np.int64
        )[frames],
        v7_joint_position_urad=np.asarray(
            v7_arrays["joint_position_urad"], dtype=np.int64
        ),
        emitted_joint_position_urad=joint_urad[frames],
    )
    baseline_distance = int(progress["baseline_squared_distance_microradians_squared"])
    emitted_distance = int(progress["emitted_squared_distance_microradians_squared"])
    progress["improvement_basis_points"] = (
        (baseline_distance - emitted_distance) * 10_000 // baseline_distance
    )
    emitted = {
        "root_position_um": root_um,
        "root_quaternion_q1_30": quaternion_q1_30,
        "root_yaw_urad": yaw_urad,
        "root_linear_velocity_um_s": root_velocity,
        "root_yaw_velocity_urad_s": yaw_velocity,
        "joint_position_urad": joint_urad,
        "joint_velocity_urad_s": joint_velocity,
        "effector_position_um": effector_um,
        "center_of_mass_um": center_um,
        "contact_modes": modes,
        "contacts": np.asarray(v9_arrays["contacts"], dtype=np.uint8),
        "phase_u16": np.asarray(v9_arrays["phase_u16"], dtype=np.uint16),
        "reference_frame": np.asarray(v9_arrays["reference_frame"], dtype=np.int64),
    }
    gates = {
        "normal_residual": contact["maximum_normal_residual_micrometres"]
        <= tolerances.maximum_normal_residual_micrometres,
        "finite_tangent": contact["maximum_tangential_step_micrometres"]
        <= tolerances.maximum_tangential_step_micrometres,
        "finite_normal": contact["maximum_normal_step_micrometres"]
        <= tolerances.maximum_normal_step_micrometres,
        "analytic_tangent": contact["maximum_analytic_tangential_step_micrometres"]
        <= tolerances.maximum_tangential_step_micrometres,
        "analytic_normal": contact["maximum_analytic_normal_step_micrometres"]
        <= tolerances.maximum_normal_step_micrometres,
        "collider": minimum_collider >= -2,
        "root_vertical_velocity": maximum_root_vertical_velocity <= 200_060,
        "joint_velocity": maximum_joint_basis_points <= 2500,
        "descriptor_rom": descriptor_rom_violation == 0,
        "effective_rom": effective_rom_violation == 0,
        "endpoint_identity": endpoints_equal,
        "tracking_progress": progress["status"] == "PASS",
        "root_step": maximum_root_step <= 0.02 + 1.0e-12,
        "orientation_step": maximum_orientation_step <= 0.1 + 1.0e-12,
        "joint_step": maximum_joint_step <= 0.1 + 1.0e-12,
    }
    non_analytic_pass = all(
        value for name, value in gates.items() if name != "analytic_tangent"
    )
    passed = all(gates.values())
    return EmittedAudit(
        report={
            "status": "PASS" if passed else "FAIL",
            "failure_reasons": [name for name, value in gates.items() if not value],
            "exact_gates": gates,
            "non_analytic_exact_gates_pass": non_analytic_pass,
            "analytic_tangent_pass": gates["analytic_tangent"],
            "tangent_funnel": contact["analytic_tangent_funnel"],
            "contact": contact,
            "minimum_collider_height_micrometres": minimum_collider,
            "maximum_root_vertical_velocity_micrometres_per_second": (
                maximum_root_vertical_velocity
            ),
            "maximum_joint_velocity_basis_points": maximum_joint_basis_points,
            "maximum_descriptor_rom_violation_microradians": (descriptor_rom_violation),
            "maximum_effective_rom_violation_microradians": effective_rom_violation,
            "endpoint_identity": "PASS" if endpoints_equal else "FAIL",
            "tracking_progress": progress,
            "maximum_root_translation_step_metres": maximum_root_step,
            "maximum_root_orientation_step_radians": maximum_orientation_step,
            "maximum_joint_position_step_radians": maximum_joint_step,
            "changed_root_position_cell_count": int(
                np.count_nonzero(root_um - np.asarray(v9_arrays["root_position_um"]))
            ),
            "changed_root_quaternion_cell_count": int(
                np.count_nonzero(
                    quaternion_q1_30 - np.asarray(v9_arrays["root_quaternion_q1_30"])
                )
            ),
            "changed_joint_position_cell_count": int(
                np.count_nonzero(
                    joint_urad - np.asarray(v9_arrays["joint_position_urad"])
                )
            ),
            "emitted_hashes": inherited._array_hashes(emitted),
        },
        arrays=emitted,
        analytic_velocity_um_s=analytic,
    )


def _contact_metrics(
    *,
    effector_ids: tuple[str, ...],
    effector_position_um: NDArray[np.int64],
    active: NDArray[np.bool_],
    analytic_velocity_um_s: NDArray[np.float64],
    tolerances: contact_manifold.ContactManifoldTolerances,
) -> dict[str, Any]:
    sole = effector_position_um[
        :, contact_manifold._sole_effector_indices(effector_ids)
    ]
    normal_residual = np.abs(sole[..., 1])
    shared = active[1:] & active[:-1]
    step = sole[1:] - sole[:-1]
    tangential_step = np.linalg.norm(step[..., (0, 2)], axis=-1)
    normal_step = np.abs(step[..., 1])
    analytic_tangent = (
        np.linalg.norm(analytic_velocity_um_s[..., (0, 2)], axis=-1) / 60.0
    )
    analytic_normal = np.abs(analytic_velocity_um_s[..., 1]) / 60.0
    maximum_normal = int(contact_manifold._masked_max(normal_residual, active))
    maximum_tangent_step = int(
        np.ceil(contact_manifold._masked_max(tangential_step, shared))
    )
    maximum_normal_step = int(contact_manifold._masked_max(normal_step, shared))
    maximum_analytic_tangent = int(
        np.ceil(contact_manifold._masked_max(analytic_tangent, active))
    )
    maximum_analytic_normal = int(
        np.ceil(contact_manifold._masked_max(analytic_normal, active))
    )
    excess = np.maximum(
        analytic_tangent[active]
        - float(tolerances.maximum_tangential_step_micrometres),
        0.0,
    ) / float(tolerances.maximum_tangential_step_micrometres)
    return {
        "status": (
            "PASS"
            if maximum_normal <= tolerances.maximum_normal_residual_micrometres
            and maximum_tangent_step <= tolerances.maximum_tangential_step_micrometres
            and maximum_normal_step <= tolerances.maximum_normal_step_micrometres
            and maximum_analytic_tangent
            <= tolerances.maximum_tangential_step_micrometres
            and maximum_analytic_normal <= tolerances.maximum_normal_step_micrometres
            else "FAIL"
        ),
        "active_point_frame_count": int(np.sum(active)),
        "shared_active_point_step_count": int(np.sum(shared)),
        "maximum_normal_residual_micrometres": maximum_normal,
        "maximum_tangential_step_micrometres": maximum_tangent_step,
        "maximum_normal_step_micrometres": maximum_normal_step,
        "maximum_analytic_tangential_step_micrometres": maximum_analytic_tangent,
        "maximum_analytic_normal_step_micrometres": maximum_analytic_normal,
        "analytic_tangent_funnel": {
            "maximum": float(np.max(excess, initial=0.0)),
            "sum": float(np.sum(excess)),
        },
        "tolerances": {
            "maximum_tangential_step_micrometres": (
                tolerances.maximum_tangential_step_micrometres
            ),
            "maximum_normal_step_micrometres": (
                tolerances.maximum_normal_step_micrometres
            ),
            "maximum_normal_residual_micrometres": (
                tolerances.maximum_normal_residual_micrometres
            ),
        },
    }


def reanchor_state(
    *,
    arrays: Mapping[str, NDArray[Any]],
    reference_rotations: NDArray[np.float64],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
) -> KtoState:
    root = np.asarray(arrays["root_position_um"], dtype=np.float64) / 1_000_000.0
    joints = np.asarray(arrays["joint_position_urad"], dtype=np.float64) / 1_000_000.0
    emitted_rotations = np.stack(
        [
            quaternion_to_matrix(np.asarray(value, dtype=np.float64) / float(1 << 30))
            for value in arrays["root_quaternion_q1_30"]
        ]
    )
    orientation = np.stack(
        [
            inherited._rotation_log(emitted @ reference.T)
            for emitted, reference in zip(
                emitted_rotations, reference_rotations, strict=True
            )
        ]
    )
    velocity, acceleration = inherited._derive_velocity_acceleration(
        root_position_m=root,
        root_orientation_delta_rad=orientation,
        joint_position_rad=joints,
        reference_rotations=reference_rotations,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
    )
    return KtoState(root, orientation, joints, velocity, acceleration)


def _funnel_strictly_decreases(
    candidate: Mapping[str, float], current: Mapping[str, float]
) -> bool:
    return (float(candidate["maximum"]), float(candidate["sum"])) < (
        float(current["maximum"]),
        float(current["sum"]),
    )


def solve_repaired_kto(
    *,
    descriptor: dict[str, Any],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    v9_profile: Mapping[str, Any],
    r114_profile: Mapping[str, Any],
    r118_profile: Mapping[str, Any],
    r119_profile: Mapping[str, Any],
) -> tuple[dict[str, Any], dict[str, NDArray[Any]] | None]:
    """Run R120's sole repaired KTO process after a solver-free preflight."""

    started = time.monotonic()
    budget = r119_profile["execution_budget"]
    qp_reports: list[dict[str, Any]] = []
    accepted_anchors: list[dict[str, Any]] = []
    exact_audit_count = 0
    bridge_used = False
    try:
        if (
            tuple(budget.get("line_search_fractions", ())) != FRACTIONS
            or int(budget.get("maximum_major_iterations", -1)) != 12
            or int(budget.get("maximum_qp_solves", -1)) != 12
            or int(budget.get("maximum_exact_emission_audits", -1)) != 72
            or int(budget.get("kto_process_count", -1)) != 1
            or int(budget.get("kto_solve_count", -1)) != 1
        ):
            raise ValueError("R120 execution budget differs")
        inherited._validate_solver_inputs(
            descriptor, v9_arrays, v7_arrays, r114_profile
        )
        modes = np.asarray(v9_arrays["contact_modes"], dtype=np.uint8)
        active = contact_manifold.contact_point_mask(modes)
        stencil_indices, stencil_coefficients = repaired_hybrid_velocity_stencil(active)
        inherited_indices, inherited_coefficients = inherited.hybrid_velocity_stencil(
            active
        )
        if not np.array_equal(stencil_indices, inherited_indices) or not np.array_equal(
            stencil_coefficients, inherited_coefficients
        ):
            raise ValueError("R120 inherited and repaired hybrid stencils differ")
        reference_rotations = np.stack(
            [
                quaternion_to_matrix(
                    np.asarray(value, dtype=np.float64) / float(1 << 30)
                )
                for value in v9_arrays["root_quaternion_q1_30"]
            ]
        )
        initial_state = inherited._initial_state(
            v9_arrays=v9_arrays,
            reference_rotations=reference_rotations,
            stencil_indices=stencil_indices,
            stencil_coefficients=stencil_coefficients,
        )
        state = initial_state
        anchor_arrays = {name: np.asarray(value) for name, value in v9_arrays.items()}
        effective_limits = inherited._effective_joint_limits(descriptor, v9_profile)
        tolerances = inherited._contact_tolerances(v9_profile)
        zero = emit_and_audit(
            descriptor=descriptor,
            state=state,
            reference_rotations=reference_rotations,
            v9_arrays=v9_arrays,
            v7_arrays=v7_arrays,
            effective_limits=effective_limits,
            tolerances=tolerances,
            stencil_indices=stencil_indices,
            stencil_coefficients=stencil_coefficients,
            identity_anchor_arrays=v9_arrays,
        )
        zero_reproduction = _zero_state_reproduction(zero.arrays, v9_arrays)
        guard, geometry, contact_rows, problem = _prepare_iteration(
            descriptor=descriptor,
            state=state,
            initial_state=initial_state,
            anchor_arrays=anchor_arrays,
            v9_arrays=v9_arrays,
            v7_arrays=v7_arrays,
            active=active,
            stencil_indices=stencil_indices,
            stencil_coefficients=stencil_coefficients,
            reference_rotations=reference_rotations,
            effective_limits=effective_limits,
            r114_profile=r114_profile,
            r118_profile=r118_profile,
            r119_profile=r119_profile,
        )
        preflight = _preflight_report(
            active=active,
            zero=zero,
            zero_reproduction=zero_reproduction,
            guard=guard,
            problem=problem,
            r119_profile=r119_profile,
        )
        if preflight["status"] != "PASS":
            raise ValueError("R120 pre-solver discriminator failed")
        if time.monotonic() - started > float(budget["maximum_wall_clock_seconds"]):
            raise ValueError("R120 pre-solver wall-clock budget exhausted")
        if _peak_memory_bytes() > int(budget["maximum_resident_memory_bytes"]):
            raise ValueError("R120 pre-solver resident-memory budget exhausted")
    except Exception as error:
        return (
            {
                "status": "INVALID",
                "termination": "pre_solver_invalid",
                "invalid_reason": f"{type(error).__name__}: {error}",
                "elapsed_seconds": time.monotonic() - started,
                "kto_solves": 0,
                "major_iterations": 0,
                "qp_solves": 0,
                "exact_emission_audits": 0,
                "bridge_used": False,
                "preflight": locals().get("preflight"),
                "accepted_anchors": [],
                "qp_iterations": [],
                "resource_budget_satisfied": False,
            },
            None,
        )

    current_funnel = zero.report["tangent_funnel"]
    accepted_anchors.append(
        {
            "anchor_number": 0,
            "source": "byte_exact_v9",
            "emitted_hashes": zero.report["emitted_hashes"],
            "tangent_funnel": current_funnel,
        }
    )
    maximum_major = int(budget["maximum_major_iterations"])
    for major in range(1, maximum_major + 1):
        if major > 1:
            try:
                guard, geometry, contact_rows, problem = _prepare_iteration(
                    descriptor=descriptor,
                    state=state,
                    initial_state=initial_state,
                    anchor_arrays=anchor_arrays,
                    v9_arrays=v9_arrays,
                    v7_arrays=v7_arrays,
                    active=active,
                    stencil_indices=stencil_indices,
                    stencil_coefficients=stencil_coefficients,
                    reference_rotations=reference_rotations,
                    effective_limits=effective_limits,
                    r114_profile=r114_profile,
                    r118_profile=r118_profile,
                    r119_profile=r119_profile,
                )
                if guard["status"] != "PASS":
                    raise ValueError("accepted-anchor local conformance failed")
            except Exception as error:
                return (
                    _terminal_result(
                        status="INVALID",
                        termination="accepted_reanchor_invalid",
                        started=started,
                        preflight=preflight,
                        qp_reports=qp_reports,
                        exact_audit_count=exact_audit_count,
                        bridge_used=bridge_used,
                        accepted_anchors=accepted_anchors,
                        budget=budget,
                        invalid_reason=f"{type(error).__name__}: {error}",
                    ),
                    None,
                )
        elapsed = time.monotonic() - started
        if elapsed > float(budget["maximum_wall_clock_seconds"]) or (
            _peak_memory_bytes() > int(budget["maximum_resident_memory_bytes"])
        ):
            return (
                _terminal_result(
                    status="FAIL",
                    termination="resource_budget_exhausted",
                    started=started,
                    preflight=preflight,
                    qp_reports=qp_reports,
                    exact_audit_count=exact_audit_count,
                    bridge_used=bridge_used,
                    accepted_anchors=accepted_anchors,
                    budget=budget,
                ),
                None,
            )

        solver = osqp.OSQP()
        try:
            solver.setup(
                P=problem.objective,
                q=problem.objective_linear,
                A=problem.constraints,
                l=problem.lower,
                u=problem.upper,
                verbose=False,
                eps_abs=float(budget["osqp_absolute_tolerance"]),
                eps_rel=float(budget["osqp_relative_tolerance"]),
                max_iter=int(budget["maximum_osqp_iterations_per_subproblem"]),
                polishing=bool(budget["osqp_polishing_enabled"]),
                adaptive_rho=bool(budget["osqp_adaptive_rho_enabled"]),
            )
        except Exception as error:
            return (
                _terminal_result(
                    status="FAIL",
                    termination="qp_setup_failure",
                    started=started,
                    preflight=preflight,
                    qp_reports=qp_reports,
                    exact_audit_count=exact_audit_count,
                    bridge_used=bridge_used,
                    accepted_anchors=accepted_anchors,
                    budget=budget,
                    invalid_reason=f"{type(error).__name__}: {error}",
                ),
                None,
            )
        try:
            solved = solver.solve(raise_error=False)
        except Exception as error:
            qp_reports.append(
                {
                    "major_iteration": major,
                    "accepted_anchor_number": len(accepted_anchors) - 1,
                    "local_conformance_guard": guard,
                    "status": "exception",
                    "error": f"{type(error).__name__}: {error}",
                    "fraction_audits": [],
                }
            )
            return (
                _terminal_result(
                    status="FAIL",
                    termination="qp_solve_failure",
                    started=started,
                    preflight=preflight,
                    qp_reports=qp_reports,
                    exact_audit_count=exact_audit_count,
                    bridge_used=bridge_used,
                    accepted_anchors=accepted_anchors,
                    budget=budget,
                ),
                None,
            )
        status = str(solved.info.status)
        qp = {
            "major_iteration": major,
            "accepted_anchor_number": len(accepted_anchors) - 1,
            "local_conformance_guard": guard,
            "status": status,
            "iterations": int(solved.info.iter),
            "primal_residual": float(solved.info.prim_res),
            "dual_residual": float(solved.info.dual_res),
            "objective": float(solved.info.obj_val),
            "variable_count": int(problem.constraints.shape[1]),
            "constraint_count": int(problem.constraints.shape[0]),
            "constraint_nonzero_count": int(problem.constraints.nnz),
            "constraint_categories": problem.constraint_categories,
            "constraint_category_nonzeros": problem.constraint_category_nonzeros,
            "bridge_used_before_iteration": bridge_used,
            "funnel_before": current_funnel,
            "fraction_audits": [],
            "accepted_fraction": None,
            "accepted_mode": None,
        }
        qp_reports.append(qp)
        if solved.x is None or not status.lower().startswith("solved"):
            return (
                _terminal_result(
                    status="FAIL",
                    termination="qp_failure",
                    started=started,
                    preflight=preflight,
                    qp_reports=qp_reports,
                    exact_audit_count=exact_audit_count,
                    bridge_used=bridge_used,
                    accepted_anchors=accepted_anchors,
                    budget=budget,
                ),
                None,
            )
        normalized_step = np.asarray(solved.x, dtype=np.float64).reshape(
            FRAME_COUNT, BLOCK_WIDTH
        )
        physical_step = normalized_step * problem.variable_scale.reshape(
            FRAME_COUNT, BLOCK_WIDTH
        )
        eligible: list[tuple[tuple[float, ...], str, EmittedAudit, dict[str, Any]]] = []
        for ordinal, fraction_text in enumerate(FRACTIONS):
            if exact_audit_count >= int(budget["maximum_exact_emission_audits"]):
                break
            fraction = inherited._parse_fraction(fraction_text)
            candidate = inherited._candidate_state(
                state=state,
                physical_step=physical_step,
                fraction=fraction,
                reference_rotations=reference_rotations,
                stencil_indices=stencil_indices,
                stencil_coefficients=stencil_coefficients,
            )
            emitted = emit_and_audit(
                descriptor=descriptor,
                state=candidate,
                reference_rotations=reference_rotations,
                v9_arrays=v9_arrays,
                v7_arrays=v7_arrays,
                effective_limits=effective_limits,
                tolerances=tolerances,
                stencil_indices=stencil_indices,
                stencil_coefficients=stencil_coefficients,
            )
            exact_audit_count += 1
            fraction_report = _fraction_report(
                fraction_text=fraction_text,
                fraction=fraction,
                emitted=emitted,
                repaired_contact=contact_rows,
                physical_step=physical_step,
                active=active,
                descriptor=descriptor,
                anchor_arrays=anchor_arrays,
                exact_probe=float(
                    r118_profile["function_contract"]["lever_probe_radians"]
                ),
                current_funnel=current_funnel,
            )
            qp["fraction_audits"].append(fraction_report)
            if emitted.report["status"] == "PASS":
                for skipped in FRACTIONS[ordinal + 1 :]:
                    qp["fraction_audits"].append(
                        {
                            "fraction": skipped,
                            "status": "NOT_AUDITED_AFTER_FIRST_EXACT_PASS",
                        }
                    )
                qp["accepted_fraction"] = fraction_text
                qp["accepted_mode"] = "exact_pass"
                accepted_anchors.append(
                    _accepted_anchor_record(
                        anchor_number=len(accepted_anchors),
                        major=major,
                        fraction=fraction_text,
                        mode="exact_pass",
                        emitted=emitted,
                    )
                )
                accepted_state = reanchor_state(
                    arrays=emitted.arrays,
                    reference_rotations=reference_rotations,
                    stencil_indices=stencil_indices,
                    stencil_coefficients=stencil_coefficients,
                )
                result = _terminal_result(
                    status="PASS",
                    termination="first_exact_quantized_pass_with_nonzero_tracking_progress",
                    started=started,
                    preflight=preflight,
                    qp_reports=qp_reports,
                    exact_audit_count=exact_audit_count,
                    bridge_used=bridge_used,
                    accepted_anchors=accepted_anchors,
                    budget=budget,
                    accepted_exact=emitted.report,
                )
                return result, _warm_start_cache(
                    state=accepted_state,
                    exact=emitted.report,
                    reference_rotations=reference_rotations,
                )
            if not emitted.report.get("non_analytic_exact_gates_pass", False):
                continue
            funnel = emitted.report["tangent_funnel"]
            progress_bp = int(
                emitted.report["tracking_progress"]["improvement_basis_points"]
            )
            if not bridge_used:
                key = (
                    float(
                        emitted.report["contact"][
                            "maximum_analytic_tangential_step_micrometres"
                        ]
                        - 2000
                    ),
                    float(-progress_bp),
                    float(ordinal),
                )
                eligible.append((key, "single_bridge", emitted, fraction_report))
            elif _funnel_strictly_decreases(funnel, current_funnel):
                key = (
                    float(funnel["maximum"]),
                    float(funnel["sum"]),
                    float(-progress_bp),
                    float(ordinal),
                )
                eligible.append((key, "restoration", emitted, fraction_report))
        if not eligible:
            return (
                _terminal_result(
                    status="FAIL",
                    termination="no_eligible_bridge_or_restoration_fraction",
                    started=started,
                    preflight=preflight,
                    qp_reports=qp_reports,
                    exact_audit_count=exact_audit_count,
                    bridge_used=bridge_used,
                    accepted_anchors=accepted_anchors,
                    budget=budget,
                ),
                None,
            )
        _, mode, accepted, accepted_fraction_report = min(
            eligible, key=lambda row: row[0]
        )
        fraction_text = str(accepted_fraction_report["fraction"])
        qp["accepted_fraction"] = fraction_text
        qp["accepted_mode"] = mode
        if mode == "single_bridge":
            bridge_used = True
        state = reanchor_state(
            arrays=accepted.arrays,
            reference_rotations=reference_rotations,
            stencil_indices=stencil_indices,
            stencil_coefficients=stencil_coefficients,
        )
        anchor_arrays = accepted.arrays
        current_funnel = accepted.report["tangent_funnel"]
        accepted_anchors.append(
            _accepted_anchor_record(
                anchor_number=len(accepted_anchors),
                major=major,
                fraction=fraction_text,
                mode=mode,
                emitted=accepted,
            )
        )
    return (
        _terminal_result(
            status="FAIL",
            termination="maximum_major_iterations_reached",
            started=started,
            preflight=preflight,
            qp_reports=qp_reports,
            exact_audit_count=exact_audit_count,
            bridge_used=bridge_used,
            accepted_anchors=accepted_anchors,
            budget=budget,
        ),
        None,
    )


def _prepare_iteration(
    *,
    descriptor: dict[str, Any],
    state: KtoState,
    initial_state: KtoState,
    anchor_arrays: Mapping[str, NDArray[Any]],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    reference_rotations: NDArray[np.float64],
    effective_limits: list[dict[str, Any]],
    r114_profile: Mapping[str, Any],
    r118_profile: Mapping[str, Any],
    r119_profile: Mapping[str, Any],
) -> tuple[
    dict[str, Any],
    KtoLinearization,
    tuple[ContactRowLinearization, ...],
    RepairedKtoProblem,
]:
    guard = audit_local_conformance(
        descriptor=descriptor,
        arrays=anchor_arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        r118_profile=r118_profile,
    )
    if guard["status"] != "PASS":
        raise ValueError("R120 local conformance guard failed")
    geometry = inherited._linearize_geometry(
        descriptor=descriptor,
        state=state,
        reference_rotations=reference_rotations,
        effector_ids=tuple(inherited._metadata(v9_arrays)["effector_ids"]),
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        r114_profile=r114_profile,
    )
    repaired_rows = build_repaired_contact_linearization(
        descriptor=descriptor,
        arrays=anchor_arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        probes=r119_profile["repaired_contact_row_contract"]["derivative_probes"],
        exact_probe=float(
            r119_profile["repaired_contact_row_contract"]["derivative_probes"][
                "exact_kernel_lever_radians"
            ]
        ),
    )
    problem = build_repaired_problem(
        descriptor=descriptor,
        state=state,
        initial_state=initial_state,
        v9_arrays=v9_arrays,
        v7_arrays=v7_arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        geometry=geometry,
        repaired_contact=repaired_rows,
        effective_limits=effective_limits,
        r114_profile=r114_profile,
    )
    return guard, geometry, repaired_rows, problem


def _zero_state_reproduction(
    emitted: Mapping[str, NDArray[Any]], v9_arrays: Mapping[str, NDArray[Any]]
) -> dict[str, Any]:
    names = tuple(sorted(name for name in emitted if name != "metadata_json_utf8"))
    differing = [
        name
        for name in names
        if name not in v9_arrays
        or not np.array_equal(np.asarray(emitted[name]), np.asarray(v9_arrays[name]))
    ]
    return {
        "status": "PASS" if not differing else "FAIL",
        "compared_array_count": len(names),
        "differing_arrays": differing,
    }


def _preflight_report(
    *,
    active: NDArray[np.bool_],
    zero: EmittedAudit,
    zero_reproduction: Mapping[str, Any],
    guard: Mapping[str, Any],
    problem: RepairedKtoProblem,
    r119_profile: Mapping[str, Any],
) -> dict[str, Any]:
    active_count = int(np.sum(active))
    row_contract = r119_profile["repaired_contact_row_contract"]
    zero_gates = zero.report.get("exact_gates", {})
    zero_gate_identity = bool(
        zero_gates
        and not zero_gates.get("tracking_progress", True)
        and all(
            value for name, value in zero_gates.items() if name != "tracking_progress"
        )
    )
    checks = {
        "active_point_count": active_count
        == int(row_contract["active_point_frame_count"]),
        "all_active_function_and_anchor_conformance": guard.get("status") == "PASS",
        "zero_state_exact_emission_reproduces_v9": zero_reproduction.get("status")
        == "PASS",
        "zero_state_all_gates_except_required_progress": zero_gate_identity,
        "retired_component_box_rows": "contact_analytic_velocity"
        not in problem.constraint_categories,
        "normal_row_inventory": problem.constraint_categories.get(
            "contact_analytic_normal"
        )
        == int(row_contract["normal_row_count"]),
        "tangent_norm_squared_row_inventory": problem.constraint_categories.get(
            "contact_analytic_tangential_norm_squared"
        )
        == int(row_contract["tangential_norm_squared_row_count"]),
        "total_row_inventory": problem.constraints.shape[0]
        == int(row_contract["resulting_total_constraint_row_count"]),
        "decision_scalar_inventory": problem.constraints.shape[1]
        == FRAME_COUNT * BLOCK_WIDTH,
        "finite_matrix": bool(np.all(np.isfinite(problem.constraints.data))),
        "ordered_bounds": bool(np.all(problem.lower <= problem.upper)),
        "r115_runtime_state_inputs_absent": True,
    }
    return {
        "status": "PASS" if all(checks.values()) else "FAIL",
        "checks": checks,
        "zero_state_reproduction": dict(zero_reproduction),
        "local_conformance_guard": dict(guard),
        "problem": {
            "variable_count": int(problem.constraints.shape[1]),
            "constraint_count": int(problem.constraints.shape[0]),
            "constraint_nonzero_count": int(problem.constraints.nnz),
            "constraint_categories": problem.constraint_categories,
            "constraint_category_nonzeros": problem.constraint_category_nonzeros,
            "lower_above_upper_count": int(
                np.count_nonzero(problem.lower > problem.upper)
            ),
        },
        "r115_runtime_inputs": [],
        "solver_setup_calls": 0,
        "qp_solves": 0,
    }


def _model_contact_values(
    *,
    repaired_contact: Sequence[ContactRowLinearization],
    physical_step: NDArray[np.float64],
    fraction: float,
) -> tuple[NDArray[np.float64], NDArray[np.float64], dict[str, float]]:
    flattened = physical_step.reshape(-1)
    values = np.empty((len(repaired_contact), 3), dtype=np.float64)
    tangent_scalar = np.empty(len(repaired_contact), dtype=np.float64)
    for ordinal, row in enumerate(repaired_contact):
        local = fraction * flattened[row.global_columns]
        values[ordinal] = row.velocity_m_s + row.vector_jacobian_m_s @ local
        coefficients = 2.0 * (
            row.velocity_m_s[0] * row.vector_jacobian_m_s[0]
            + row.velocity_m_s[2] * row.vector_jacobian_m_s[2]
        )
        tangent_scalar[ordinal] = (
            row.velocity_m_s[0] ** 2 + row.velocity_m_s[2] ** 2 + coefficients @ local
        )
    tangent = np.sqrt(np.maximum(tangent_scalar, 0.0))
    excess = np.maximum(tangent - TANGENT_BOUND_M_S, 0.0) / TANGENT_BOUND_M_S
    return (
        values,
        tangent_scalar,
        {
            "maximum": float(np.max(excess, initial=0.0)),
            "sum": float(np.sum(excess)),
        },
    )


def _fraction_report(
    *,
    fraction_text: str,
    fraction: float,
    emitted: EmittedAudit,
    repaired_contact: Sequence[ContactRowLinearization],
    physical_step: NDArray[np.float64],
    active: NDArray[np.bool_],
    descriptor: dict[str, Any],
    anchor_arrays: Mapping[str, NDArray[Any]],
    exact_probe: float,
    current_funnel: Mapping[str, float],
) -> dict[str, Any]:
    _model_values, model_tangent_scalar, model_funnel = _model_contact_values(
        repaired_contact=repaired_contact,
        physical_step=physical_step,
        fraction=fraction,
    )
    actual_funnel = emitted.report["tangent_funnel"]
    active_indices = np.argwhere(active)
    hotspot = None
    if emitted.analytic_velocity_um_s.shape == (*active.shape, 3):
        exact_values = emitted.analytic_velocity_um_s[active] / 1_000_000.0
        exact_tangent = np.linalg.norm(exact_values[:, (0, 2)], axis=1)
        hotspot_ordinal = int(np.argmax(exact_tangent))
        row = repaired_contact[hotspot_ordinal]
        frame, side, point = (int(value) for value in active_indices[hotspot_ordinal])
        flattened = physical_step.reshape(-1)
        local_delta = fraction * flattened[row.global_columns]
        continuous = (
            repaired_exact_contact_velocity(
                descriptor=descriptor,
                arrays=anchor_arrays,
                anchor=row.anchor,
                variables=row.variables,
                delta=local_delta,
                exact_probe=exact_probe,
            )
            / 1_000_000.0
        )
        tangent_coefficients = 2.0 * (
            row.velocity_m_s[0] * row.vector_jacobian_m_s[0]
            + row.velocity_m_s[2] * row.vector_jacobian_m_s[2]
        )
        q_nonzero = sum(
            abs(float(value)) > COEFFICIENT_ELISION
            for value, spec in zip(tangent_coefficients, row.variables, strict=True)
            if spec.block == "configuration"
        )
        v_nonzero = sum(
            abs(float(value)) > COEFFICIENT_ELISION
            for value, spec in zip(tangent_coefficients, row.variables, strict=True)
            if spec.block == "velocity"
        )
        hotspot = {
            "frame": frame,
            "side": ("left", "right")[side],
            "point": ("heel", "forefoot")[point],
            "row_identity": "analytic_tangential_norm_squared",
            "baseline_velocity_metres_per_second": row.velocity_m_s.tolist(),
            "configuration_nonzero_count": q_nonzero,
            "velocity_nonzero_count": v_nonzero,
            "vector_derivative_frobenius_norm": float(
                np.linalg.norm(row.vector_jacobian_m_s)
            ),
            "tangent_derivative_norm": float(np.linalg.norm(tangent_coefficients)),
            "qp_predicted_scalar_square_metres_per_square_second": float(
                model_tangent_scalar[hotspot_ordinal]
            ),
            "continuous_evaluated_scalar_square_metres_per_square_second": float(
                continuous[0] ** 2 + continuous[2] ** 2
            ),
            "emitted_exact_scalar_square_metres_per_square_second": float(
                exact_values[hotspot_ordinal, 0] ** 2
                + exact_values[hotspot_ordinal, 2] ** 2
            ),
            "selected_fraction": fraction_text,
        }
    return {
        "fraction": fraction_text,
        "status": emitted.report["status"],
        "failure_reasons": emitted.report["failure_reasons"],
        "non_analytic_exact_gates_pass": emitted.report[
            "non_analytic_exact_gates_pass"
        ],
        "analytic_tangent_pass": emitted.report["analytic_tangent_pass"],
        "tracking_progress": emitted.report.get("tracking_progress"),
        "emitted_hashes": emitted.report.get("emitted_hashes"),
        "funnel_before": dict(current_funnel),
        "modeled_funnel": model_funnel,
        "emitted_exact_funnel": actual_funnel,
        "modeled_reduction": {
            "maximum": float(current_funnel["maximum"])
            - float(model_funnel["maximum"]),
            "sum": float(current_funnel["sum"]) - float(model_funnel["sum"]),
        },
        "actual_reduction": {
            "maximum": float(current_funnel["maximum"])
            - float(actual_funnel["maximum"]),
            "sum": float(current_funnel["sum"]) - float(actual_funnel["sum"]),
        },
        "contact": emitted.report.get("contact"),
        "contact_hotspot": hotspot,
    }


def _accepted_anchor_record(
    *,
    anchor_number: int,
    major: int,
    fraction: str,
    mode: str,
    emitted: EmittedAudit,
) -> dict[str, Any]:
    return {
        "anchor_number": anchor_number,
        "accepted_at_major_iteration": major,
        "fraction": fraction,
        "mode": mode,
        "emitted_hashes": emitted.report["emitted_hashes"],
        "tangent_funnel": emitted.report["tangent_funnel"],
        "tracking_progress": emitted.report["tracking_progress"],
    }


def _terminal_result(
    *,
    status: str,
    termination: str,
    started: float,
    preflight: Mapping[str, Any],
    qp_reports: Sequence[Mapping[str, Any]],
    exact_audit_count: int,
    bridge_used: bool,
    accepted_anchors: Sequence[Mapping[str, Any]],
    budget: Mapping[str, Any],
    invalid_reason: str | None = None,
    accepted_exact: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    elapsed = time.monotonic() - started
    resource_ok = bool(
        elapsed <= float(budget["maximum_wall_clock_seconds"])
        and _peak_memory_bytes() <= int(budget["maximum_resident_memory_bytes"])
    )
    result = {
        "status": status,
        "termination": termination,
        "elapsed_seconds": elapsed,
        "kto_solves": 1,
        "major_iterations": len(qp_reports),
        "qp_solves": len(qp_reports),
        "exact_emission_audits": exact_audit_count,
        "bridge_used": bridge_used,
        "preflight": dict(preflight),
        "accepted_anchors": [dict(row) for row in accepted_anchors],
        "qp_iterations": [dict(row) for row in qp_reports],
        "accepted_exact_result": (
            dict(accepted_exact) if accepted_exact is not None else None
        ),
        "resource_budget_satisfied": resource_ok,
    }
    if invalid_reason is not None:
        result["invalid_reason"] = invalid_reason
    return result


def _warm_start_cache(
    *,
    state: KtoState,
    exact: Mapping[str, Any],
    reference_rotations: NDArray[np.float64],
) -> dict[str, NDArray[Any]]:
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r120-repaired-kto-solver-private.v1",
        "frame_count": FRAME_COUNT,
        "qva_scalar_count": FRAME_COUNT * BLOCK_WIDTH,
        "emitted_aggregate_sha256": exact["emitted_hashes"]["aggregate_sha256"],
        "candidate_or_corpus_authority": False,
    }
    encoded = inherited.json.dumps(
        metadata, sort_keys=True, separators=(",", ":")
    ).encode()
    return {
        "root_position_m": state.root_position_m,
        "root_orientation_delta_rad": state.root_orientation_delta_rad,
        "joint_position_rad": state.joint_position_rad,
        "velocity": state.velocity,
        "acceleration": state.acceleration,
        "reference_root_rotation": reference_rotations,
        "metadata_json_utf8": np.frombuffer(encoded, dtype=np.uint8).copy(),
    }


def _peak_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024
