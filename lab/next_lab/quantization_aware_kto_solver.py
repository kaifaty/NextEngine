from __future__ import annotations

import hashlib
import json
import math
import time
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Any

import numpy as np
import osqp
from numpy.typing import NDArray
from scipy import sparse

from next_lab import contact_manifold
from next_lab.contact_manifold import (
    ContactManifoldTolerances,
    contact_manifold_diagnostics,
    contact_point_mask,
)
from next_lab.contact_trajectory import hybrid_velocity_stencil
from next_lab.motion_math import (
    collider_minimum_y,
    decompose_xzy,
    matrix_to_quaternion,
    q1_30,
    quaternion_to_matrix,
    target_center_of_mass,
    target_effectors,
    target_forward_kinematics,
)
from next_lab.quantization_aware_kto_formulation import (
    classify_tracking_progress,
)

FRAME_COUNT = 801
JOINT_COUNT = 23
Q_WIDTH = 29
BLOCK_WIDTH = 87
ROOT_POSITION = slice(0, 3)
ROOT_ORIENTATION = slice(3, 6)
JOINT_POSITION = slice(6, 29)
VELOCITY_OFFSET = 29
ACCELERATION_OFFSET = 58


@dataclass(frozen=True)
class KtoState:
    root_position_m: NDArray[np.float64]
    root_orientation_delta_rad: NDArray[np.float64]
    joint_position_rad: NDArray[np.float64]
    velocity: NDArray[np.float64]
    acceleration: NDArray[np.float64]


@dataclass(frozen=True)
class KtoLinearization:
    sole_position_m: NDArray[np.float64]
    sole_jacobian: NDArray[np.float64]
    collider_height_m: NDArray[np.float64]
    collider_jacobian: NDArray[np.float64]
    analytic_sole_velocity_m_s: NDArray[np.float64]


@dataclass(frozen=True)
class KtoProblem:
    objective: sparse.csc_matrix
    objective_linear: NDArray[np.float64]
    constraints: sparse.csc_matrix
    lower: NDArray[np.float64]
    upper: NDArray[np.float64]
    variable_scale: NDArray[np.float64]
    constraint_categories: dict[str, int]


def solve_quantization_aware_kto(
    *,
    descriptor: dict[str, Any],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    v9_profile: Mapping[str, Any],
    r114_profile: Mapping[str, Any],
) -> tuple[dict[str, Any], dict[str, NDArray[Any]] | None]:
    """Run the single R115 SQP/QP and return metrics plus an optional cache."""

    started = time.monotonic()
    _validate_solver_inputs(descriptor, v9_arrays, v7_arrays, r114_profile)
    modes = np.asarray(v9_arrays["contact_modes"], dtype=np.uint8)
    active = contact_point_mask(modes)
    stencil_indices, stencil_coefficients = hybrid_velocity_stencil(active)
    reference_rotations = np.stack(
        [
            quaternion_to_matrix(np.asarray(value, dtype=np.float64) / float(1 << 30))
            for value in v9_arrays["root_quaternion_q1_30"]
        ]
    )
    state = _initial_state(
        v9_arrays=v9_arrays,
        reference_rotations=reference_rotations,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
    )
    effective_limits = _effective_joint_limits(descriptor, v9_profile)
    tolerances = _contact_tolerances(v9_profile)
    linearization = _linearize_geometry(
        descriptor=descriptor,
        state=state,
        reference_rotations=reference_rotations,
        effector_ids=tuple(_metadata(v9_arrays)["effector_ids"]),
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        r114_profile=r114_profile,
    )
    problem = _build_problem(
        descriptor=descriptor,
        state=state,
        v9_arrays=v9_arrays,
        v7_arrays=v7_arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        linearization=linearization,
        effective_limits=effective_limits,
        r114_profile=r114_profile,
    )
    solver = osqp.OSQP()
    budget = r114_profile["execution_budget"]
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
    solved = solver.solve(raise_error=False)
    qp_status = str(solved.info.status)
    qp = {
        "major_iteration": 1,
        "status": qp_status,
        "iterations": int(solved.info.iter),
        "primal_residual": float(solved.info.prim_res),
        "dual_residual": float(solved.info.dual_res),
        "objective": float(solved.info.obj_val),
        "variable_count": int(problem.constraints.shape[1]),
        "constraint_count": int(problem.constraints.shape[0]),
        "constraint_nonzero_count": int(problem.constraints.nnz),
        "constraint_categories": problem.constraint_categories,
    }
    exact_audits: list[dict[str, Any]] = []
    accepted_state: KtoState | None = None
    accepted_exact: dict[str, Any] | None = None
    accepted_fraction: str | None = None
    if solved.x is not None and qp_status.lower().startswith("solved"):
        normalized_step = np.asarray(solved.x, dtype=np.float64).reshape(
            FRAME_COUNT, BLOCK_WIDTH
        )
        physical_step = normalized_step * problem.variable_scale.reshape(
            FRAME_COUNT, BLOCK_WIDTH
        )
        for fraction_text in budget["line_search_fractions"]:
            fraction = _parse_fraction(fraction_text)
            candidate = _candidate_state(
                state=state,
                physical_step=physical_step,
                fraction=fraction,
                reference_rotations=reference_rotations,
                stencil_indices=stencil_indices,
                stencil_coefficients=stencil_coefficients,
            )
            exact = _emit_and_audit(
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
            exact_audits.append(
                {
                    "fraction": fraction_text,
                    **{
                        key: value
                        for key, value in exact.items()
                        if key != "emitted_hashes"
                    },
                }
            )
            if exact["status"] == "PASS":
                accepted_state = candidate
                accepted_exact = exact
                accepted_fraction = fraction_text
                break
    passed = accepted_state is not None and accepted_exact is not None
    elapsed = time.monotonic() - started
    if elapsed > float(budget["maximum_wall_clock_seconds"]):
        passed = False
        accepted_state = None
        accepted_exact = None
        accepted_fraction = None
    result = {
        "status": "PASS" if passed else "FAIL",
        "termination": (
            "first_exact_quantized_pass_with_nonzero_tracking_progress"
            if passed
            else "no_exact_progress_step"
            if solved.x is not None and qp_status.lower().startswith("solved")
            else "qp_failure"
        ),
        "elapsed_seconds": elapsed,
        "major_iterations": 1,
        "qp_solves": 1,
        "exact_emission_audits": len(exact_audits),
        "accepted_line_search_fraction": accepted_fraction,
        "qp": qp,
        "exact_audits": exact_audits,
        "accepted_exact_result": accepted_exact,
        "resource_budget_satisfied": elapsed
        <= float(budget["maximum_wall_clock_seconds"]),
    }
    cache = (
        _warm_start_cache(
            accepted_state,
            accepted_exact,
            reference_rotations=reference_rotations,
        )
        if passed and accepted_state is not None and accepted_exact is not None
        else None
    )
    return result, cache


def _validate_solver_inputs(
    descriptor: Mapping[str, Any],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    profile: Mapping[str, Any],
) -> None:
    if (
        len(descriptor.get("joints", ())) != JOINT_COUNT
        or profile.get("decision_variables", {}).get("total_scalar_count")
        != FRAME_COUNT * BLOCK_WIDTH
        or np.asarray(v9_arrays.get("root_position_um")).shape != (FRAME_COUNT, 3)
        or np.asarray(v9_arrays.get("root_quaternion_q1_30")).shape != (FRAME_COUNT, 4)
        or np.asarray(v9_arrays.get("joint_position_urad")).shape
        != (FRAME_COUNT, JOINT_COUNT)
        or np.asarray(v7_arrays.get("joint_position_urad")).shape != (12, JOINT_COUNT)
    ):
        raise ValueError("R115 solver input shape differs")


def _initial_state(
    *,
    v9_arrays: Mapping[str, NDArray[Any]],
    reference_rotations: NDArray[np.float64],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
) -> KtoState:
    root = np.asarray(v9_arrays["root_position_um"], dtype=np.float64) / 1_000_000.0
    orientation = np.zeros((FRAME_COUNT, 3), dtype=np.float64)
    joints = (
        np.asarray(v9_arrays["joint_position_urad"], dtype=np.float64) / 1_000_000.0
    )
    velocity, acceleration = _derive_velocity_acceleration(
        root_position_m=root,
        root_orientation_delta_rad=orientation,
        joint_position_rad=joints,
        reference_rotations=reference_rotations,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
    )
    return KtoState(root, orientation, joints, velocity, acceleration)


def _derive_velocity_acceleration(
    *,
    root_position_m: NDArray[np.float64],
    root_orientation_delta_rad: NDArray[np.float64],
    joint_position_rad: NDArray[np.float64],
    reference_rotations: NDArray[np.float64],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
    rotations = _compose_rotations(reference_rotations, root_orientation_delta_rad)
    velocity = np.empty((FRAME_COUNT, Q_WIDTH), dtype=np.float64)
    velocity[:, ROOT_POSITION] = _stencil_values(
        root_position_m, stencil_indices, stencil_coefficients
    )
    for frame in range(FRAME_COUNT):
        left, right = (int(value) for value in stencil_indices[frame])
        rate = float(stencil_coefficients[frame, 1])
        velocity[frame, ROOT_ORIENTATION] = rate * _rotation_log(
            rotations[right] @ rotations[left].T
        )
    velocity[:, JOINT_POSITION] = _stencil_values(
        joint_position_rad, stencil_indices, stencil_coefficients
    )
    acceleration = _stencil_values(velocity, stencil_indices, stencil_coefficients)
    return velocity, acceleration


def _stencil_values(
    values: NDArray[np.float64],
    indices: NDArray[np.int64],
    coefficients: NDArray[np.float64],
) -> NDArray[np.float64]:
    weights = coefficients.reshape((FRAME_COUNT, 2) + (1,) * (values.ndim - 1))
    return np.sum(values[indices] * weights, axis=1)


def _compose_rotations(
    references: NDArray[np.float64], deltas: NDArray[np.float64]
) -> NDArray[np.float64]:
    return np.stack(
        [
            _rotation_exp(delta) @ reference
            for delta, reference in zip(deltas, references, strict=True)
        ]
    )


def _rotation_exp(vector: NDArray[np.float64]) -> NDArray[np.float64]:
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


def _rotation_log(rotation: NDArray[np.float64]) -> NDArray[np.float64]:
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
        raise ValueError("R115 SO(3) principal log is ambiguous")
    return angle / (2.0 * math.sin(angle)) * vee


def _linearize_geometry(
    *,
    descriptor: dict[str, Any],
    state: KtoState,
    reference_rotations: NDArray[np.float64],
    effector_ids: tuple[str, ...],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    r114_profile: Mapping[str, Any],
) -> KtoLinearization:
    rotations = _compose_rotations(
        reference_rotations, state.root_orientation_delta_rad
    )
    sole_indices = contact_manifold._sole_effector_indices(effector_ids)
    colliders, _ = contact_manifold._collider_inventory(descriptor)
    collider_count = len(colliders)
    sole = np.empty((FRAME_COUNT, 2, 2, 3), dtype=np.float64)
    sole_jacobian = np.zeros((FRAME_COUNT, 2, 2, 3, Q_WIDTH), dtype=np.float64)
    collider = np.empty((FRAME_COUNT, collider_count), dtype=np.float64)
    collider_jacobian = np.zeros(
        (FRAME_COUNT, collider_count, Q_WIDTH), dtype=np.float64
    )
    sole_jacobian[..., 0, 0] = 1.0
    sole_jacobian[..., 1, 1] = 1.0
    sole_jacobian[..., 2, 2] = 1.0
    collider_jacobian[..., 1] = 1.0
    budget = r114_profile["execution_budget"]
    orientation_probe = float(budget["finite_difference_root_orientation_radians"])
    joint_probe = float(budget["finite_difference_joint_position_radians"])
    for frame in range(FRAME_COUNT):
        base_sole, base_collider = _frame_geometry(
            descriptor,
            state.root_position_m[frame],
            rotations[frame],
            state.joint_position_rad[frame],
            effector_ids,
            sole_indices,
            colliders,
        )
        sole[frame] = base_sole
        collider[frame] = base_collider
        for axis in range(3):
            offset = np.zeros(3, dtype=np.float64)
            offset[axis] = orientation_probe
            plus_sole, plus_collider = _frame_geometry(
                descriptor,
                state.root_position_m[frame],
                _rotation_exp(offset) @ rotations[frame],
                state.joint_position_rad[frame],
                effector_ids,
                sole_indices,
                colliders,
            )
            minus_sole, minus_collider = _frame_geometry(
                descriptor,
                state.root_position_m[frame],
                _rotation_exp(-offset) @ rotations[frame],
                state.joint_position_rad[frame],
                effector_ids,
                sole_indices,
                colliders,
            )
            sole_jacobian[frame, ..., 3 + axis] = (plus_sole - minus_sole) / (
                2.0 * orientation_probe
            )
            collider_jacobian[frame, :, 3 + axis] = (plus_collider - minus_collider) / (
                2.0 * orientation_probe
            )
        for ordinal in range(JOINT_COUNT):
            plus = state.joint_position_rad[frame].copy()
            minus = state.joint_position_rad[frame].copy()
            plus[ordinal] += joint_probe
            minus[ordinal] -= joint_probe
            plus_sole, plus_collider = _frame_geometry(
                descriptor,
                state.root_position_m[frame],
                rotations[frame],
                plus,
                effector_ids,
                sole_indices,
                colliders,
            )
            minus_sole, minus_collider = _frame_geometry(
                descriptor,
                state.root_position_m[frame],
                rotations[frame],
                minus,
                effector_ids,
                sole_indices,
                colliders,
            )
            sole_jacobian[frame, ..., 6 + ordinal] = (plus_sole - minus_sole) / (
                2.0 * joint_probe
            )
            collider_jacobian[frame, :, 6 + ordinal] = (
                plus_collider - minus_collider
            ) / (2.0 * joint_probe)
    analytic = np.empty_like(sole)
    for frame, side, point in np.argwhere(active):
        analytic[frame, side, point] = (
            sole_jacobian[frame, side, point] @ state.velocity[frame]
        )
    analytic[~active] = 0.0
    return KtoLinearization(
        sole,
        sole_jacobian,
        collider,
        collider_jacobian,
        analytic,
    )


def _frame_geometry(
    descriptor: dict[str, Any],
    root_position: NDArray[np.float64],
    root_rotation: NDArray[np.float64],
    joints: NDArray[np.float64],
    effector_ids: tuple[str, ...],
    sole_indices: NDArray[np.int64],
    colliders: list[tuple[int, Mapping[str, Any]]],
) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
    positions, rotations = target_forward_kinematics(
        descriptor,
        root_position,
        matrix_to_quaternion(root_rotation),
        joints,
    )
    effectors = target_effectors(descriptor, positions, rotations)
    sole = np.empty((2, 2, 3), dtype=np.float64)
    for side in range(2):
        for point in range(2):
            sole[side, point] = effectors[effector_ids[int(sole_indices[side, point])]]
    heights = np.asarray(
        [
            collider_minimum_y(positions[slot], rotations[slot], collider)
            for slot, collider in colliders
        ],
        dtype=np.float64,
    )
    return sole, heights


def _build_problem(
    *,
    descriptor: Mapping[str, Any],
    state: KtoState,
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    linearization: KtoLinearization,
    effective_limits: list[dict[str, Any]],
    r114_profile: Mapping[str, Any],
) -> KtoProblem:
    q_scale = np.asarray([0.02] * 3 + [0.1] * 3 + [0.1] * 23, dtype=np.float64)
    local_scale = np.concatenate((q_scale, q_scale * 60.0, q_scale * 3600.0))
    variable_scale = np.tile(local_scale, FRAME_COUNT)
    variable_count = FRAME_COUNT * BLOCK_WIDTH
    objective_diagonal = np.zeros(variable_count, dtype=np.float64)
    objective_linear = np.zeros(variable_count, dtype=np.float64)
    q_mean = 1.0 / (FRAME_COUNT * Q_WIDTH)
    a_mean = 1.0 / (FRAME_COUNT * Q_WIDTH)
    for frame in range(FRAME_COUNT):
        for local in range(Q_WIDTH):
            q_column = _column(frame, 0, local)
            a_column = _column(frame, 2, local)
            objective_diagonal[q_column] += 2.0 * q_mean
            objective_diagonal[a_column] += 2.0 * a_mean
    frames = np.asarray(v7_arrays["reference_frame"], dtype=np.int64)
    v9_joint = np.asarray(v9_arrays["joint_position_urad"], dtype=np.int64)[frames]
    v7_joint = np.asarray(v7_arrays["joint_position_urad"], dtype=np.int64)
    baseline_rad_squared = (
        r114_profile["tracking_progress_gate"][
            "v9_to_v7_squared_l2_distance_microradians_squared"
        ]
        / 1_000_000_000_000.0
    )
    for local_frame, frame in enumerate(frames):
        for joint in range(JOINT_COUNT):
            delta = int(v7_joint[local_frame, joint] - v9_joint[local_frame, joint])
            if delta == 0:
                continue
            column = _column(int(frame), 0, 6 + joint)
            scale = q_scale[6 + joint]
            target = delta / 1_000_000.0
            objective_diagonal[column] += 2.0 * scale * scale / baseline_rad_squared
            objective_linear[column] += -2.0 * scale * target / baseline_rad_squared
    objective = sparse.diags(objective_diagonal, format="csc")

    rows: list[int] = []
    columns: list[int] = []
    values: list[float] = []
    lower: list[float] = []
    upper: list[float] = []
    categories: dict[str, int] = {}

    def add_row(
        entries: list[tuple[int, float]],
        low: float,
        high: float,
        row_scale: float,
        category: str,
    ) -> None:
        row = len(lower)
        for column, coefficient in entries:
            rows.append(row)
            columns.append(column)
            values.append(coefficient * variable_scale[column] / row_scale)
        lower.append(low / row_scale)
        upper.append(high / row_scale)
        categories[category] = categories.get(category, 0) + 1

    for frame in range(FRAME_COUNT):
        for local in range(Q_WIDTH):
            velocity_entries = [(_column(frame, 1, local), 1.0)]
            for slot in range(2):
                source = int(stencil_indices[frame, slot])
                coefficient = float(stencil_coefficients[frame, slot])
                velocity_entries.append((_column(source, 0, local), -coefficient))
            add_row(
                velocity_entries,
                0.0,
                0.0,
                float(q_scale[local] * 60.0),
                "q_to_v_equality",
            )
            acceleration_entries = [(_column(frame, 2, local), 1.0)]
            for slot in range(2):
                source = int(stencil_indices[frame, slot])
                coefficient = float(stencil_coefficients[frame, slot])
                acceleration_entries.append((_column(source, 1, local), -coefficient))
            add_row(
                acceleration_entries,
                0.0,
                0.0,
                float(q_scale[local] * 3600.0),
                "v_to_a_equality",
            )

    normal_bound = 0.005
    finite_normal_bound = 0.001
    finite_tangent_bound = 0.002 / math.sqrt(2.0)
    for frame, side, point in np.argwhere(active):
        frame = int(frame)
        side = int(side)
        point = int(point)
        jacobian = linearization.sole_jacobian[frame, side, point, 1]
        current = float(linearization.sole_position_m[frame, side, point, 1])
        add_row(
            _local_q_entries(frame, jacobian),
            -normal_bound - current,
            normal_bound - current,
            normal_bound,
            "contact_normal_residual",
        )
    shared = active[1:] & active[:-1]
    component_bounds = (finite_tangent_bound, finite_normal_bound, finite_tangent_bound)
    for previous, side, point in np.argwhere(shared):
        previous = int(previous)
        frame = previous + 1
        side = int(side)
        point = int(point)
        difference = (
            linearization.sole_position_m[frame, side, point]
            - linearization.sole_position_m[previous, side, point]
        )
        for component, bound in enumerate(component_bounds):
            entries = _local_q_entries(
                frame, linearization.sole_jacobian[frame, side, point, component]
            )
            entries.extend(
                _local_q_entries(
                    previous,
                    -linearization.sole_jacobian[previous, side, point, component],
                )
            )
            add_row(
                entries,
                -bound - float(difference[component]),
                bound - float(difference[component]),
                bound,
                "contact_finite_velocity",
            )
    analytic_bounds = (
        0.12 / math.sqrt(2.0),
        0.06,
        0.12 / math.sqrt(2.0),
    )
    for frame, side, point in np.argwhere(active):
        frame = int(frame)
        side = int(side)
        point = int(point)
        for component, bound in enumerate(analytic_bounds):
            jacobian = linearization.sole_jacobian[frame, side, point, component]
            entries = [
                (_column(frame, 1, local), float(value))
                for local, value in enumerate(jacobian)
                if abs(float(value)) > 1.0e-14
            ]
            current = float(
                linearization.analytic_sole_velocity_m_s[frame, side, point, component]
            )
            add_row(
                entries,
                -bound - current,
                bound - current,
                bound,
                "contact_analytic_velocity",
            )
    for frame in range(FRAME_COUNT):
        for collider in range(linearization.collider_height_m.shape[1]):
            current = float(linearization.collider_height_m[frame, collider])
            add_row(
                _local_q_entries(
                    frame, linearization.collider_jacobian[frame, collider]
                ),
                -0.000002 - current,
                np.inf,
                0.005,
                "collider_floor",
            )
    add_velocity_and_rom_constraints(
        add_row=add_row,
        descriptor=descriptor,
        state=state,
        effective_limits=effective_limits,
    )
    for frame in range(FRAME_COUNT):
        for local in range(Q_WIDTH):
            bound = 0.0 if frame in (0, FRAME_COUNT - 1) else 1.0
            add_row(
                [(_column(frame, 0, local), 1.0)],
                -bound * q_scale[local],
                bound * q_scale[local],
                q_scale[local],
                "endpoint_or_trust_bound",
            )
    constraints = sparse.coo_matrix(
        (np.asarray(values), (np.asarray(rows), np.asarray(columns))),
        shape=(len(lower), variable_count),
    ).tocsc()
    return KtoProblem(
        objective=sparse.triu(objective, format="csc"),
        objective_linear=objective_linear,
        constraints=constraints,
        lower=np.asarray(lower, dtype=np.float64),
        upper=np.asarray(upper, dtype=np.float64),
        variable_scale=variable_scale,
        constraint_categories=categories,
    )


def add_velocity_and_rom_constraints(
    *,
    add_row: Any,
    descriptor: Mapping[str, Any],
    state: KtoState,
    effective_limits: list[dict[str, Any]],
) -> None:
    root_bound = 0.200060
    for frame in range(FRAME_COUNT):
        current = float(state.velocity[frame, 1])
        add_row(
            [(_column(frame, 1, 1), 1.0)],
            -root_bound - current,
            root_bound - current,
            root_bound,
            "root_vertical_velocity",
        )
    joint_by_ordinal = {int(row["dof_ordinal"]): row for row in descriptor["joints"]}
    for frame in range(FRAME_COUNT):
        for joint in range(JOINT_COUNT):
            maximum = (
                int(joint_by_ordinal[joint]["maximum_velocity_microradians_per_second"])
                * 2500
                // 10_000
                / 1_000_000.0
            )
            current_velocity = float(state.velocity[frame, 6 + joint])
            add_row(
                [(_column(frame, 1, 6 + joint), 1.0)],
                -maximum - current_velocity,
                maximum - current_velocity,
                maximum,
                "joint_velocity",
            )
            minimum, maximum_position = effective_limits[joint]["position_microradians"]
            current_position = float(state.joint_position_rad[frame, joint])
            add_row(
                [(_column(frame, 0, 6 + joint), 1.0)],
                minimum / 1_000_000.0 - current_position,
                maximum_position / 1_000_000.0 - current_position,
                0.1,
                "joint_rom",
            )


def _candidate_state(
    *,
    state: KtoState,
    physical_step: NDArray[np.float64],
    fraction: float,
    reference_rotations: NDArray[np.float64],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
) -> KtoState:
    root = state.root_position_m + fraction * physical_step[:, ROOT_POSITION]
    orientation = (
        state.root_orientation_delta_rad + fraction * physical_step[:, ROOT_ORIENTATION]
    )
    joints = state.joint_position_rad + fraction * physical_step[:, JOINT_POSITION]
    velocity, acceleration = _derive_velocity_acceleration(
        root_position_m=root,
        root_orientation_delta_rad=orientation,
        joint_position_rad=joints,
        reference_rotations=reference_rotations,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
    )
    return KtoState(root, orientation, joints, velocity, acceleration)


def _emit_and_audit(
    *,
    descriptor: dict[str, Any],
    state: KtoState,
    reference_rotations: NDArray[np.float64],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
    effective_limits: list[dict[str, Any]],
    tolerances: ContactManifoldTolerances,
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
) -> dict[str, Any]:
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
        return _invalid_exact_result("nonfinite_or_orientation_chart")
    root_um = np.rint(state.root_position_m * 1_000_000.0).astype(np.int64)
    joint_urad = np.rint(state.joint_position_rad * 1_000_000.0).astype(np.int64)
    rotations = _compose_rotations(
        reference_rotations, state.root_orientation_delta_rad
    )
    quaternions = np.stack([matrix_to_quaternion(rotation) for rotation in rotations])
    quaternion_q1_30 = q1_30(quaternions)
    yaw_urad = _lift_root_yaw(
        emitted_quaternion_q1_30=quaternion_q1_30,
        source_quaternion_q1_30=np.asarray(
            v9_arrays["root_quaternion_q1_30"], dtype=np.int64
        ),
        source_yaw_urad=np.asarray(v9_arrays["root_yaw_urad"], dtype=np.int64),
    )
    root_velocity = np.rint(
        _stencil_values(
            root_um.astype(np.float64), stencil_indices, stencil_coefficients
        )
    ).astype(np.int64)
    joint_velocity = np.rint(
        _stencil_values(
            joint_urad.astype(np.float64), stencil_indices, stencil_coefficients
        )
    ).astype(np.int64)
    yaw_velocity = np.rint(
        _stencil_values(
            yaw_urad[:, None].astype(np.float64),
            stencil_indices,
            stencil_coefficients,
        )[:, 0]
    ).astype(np.int64)
    effector_ids = tuple(_metadata(v9_arrays)["effector_ids"])
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
    contact = contact_manifold_diagnostics(
        descriptor=descriptor,
        effector_ids=effector_ids,
        root_position_um=root_um,
        root_quaternion_q1_30=quaternion_q1_30,
        root_linear_velocity_um_s=root_velocity,
        root_yaw_velocity_urad_s=yaw_velocity,
        joint_position_urad=joint_urad,
        joint_velocity_urad_s=joint_velocity,
        effector_position_um=effector_um,
        contact_modes=modes,
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
    minimum_collider = int(np.floor(np.min(collider_heights) * 1_000_000.0 + 1.0e-9))
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
    emitted_hashes = _array_hashes(emitted)
    passed = (
        contact["status"] == "PASS"
        and minimum_collider >= -2
        and maximum_root_vertical_velocity <= 200060
        and maximum_joint_basis_points <= 2500
        and descriptor_rom_violation == 0
        and effective_rom_violation == 0
        and endpoints_equal
        and progress["status"] == "PASS"
        and maximum_root_step <= 0.02 + 1.0e-12
        and maximum_orientation_step <= 0.1 + 1.0e-12
        and maximum_joint_step <= 0.1 + 1.0e-12
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "failure_reasons": [
            reason
            for reason, condition in (
                ("contact", contact["status"] != "PASS"),
                ("collider", minimum_collider < -2),
                ("root_vertical_velocity", maximum_root_vertical_velocity > 200060),
                ("joint_velocity", maximum_joint_basis_points > 2500),
                ("descriptor_rom", descriptor_rom_violation != 0),
                ("effective_rom", effective_rom_violation != 0),
                ("endpoint_identity", not endpoints_equal),
                ("tracking_progress", progress["status"] != "PASS"),
                ("root_step", maximum_root_step > 0.02 + 1.0e-12),
                (
                    "orientation_step",
                    maximum_orientation_step > 0.1 + 1.0e-12,
                ),
                ("joint_step", maximum_joint_step > 0.1 + 1.0e-12),
            )
            if condition
        ],
        "contact": contact,
        "minimum_collider_height_micrometres": minimum_collider,
        "maximum_root_vertical_velocity_micrometres_per_second": (
            maximum_root_vertical_velocity
        ),
        "maximum_joint_velocity_basis_points": maximum_joint_basis_points,
        "maximum_descriptor_rom_violation_microradians": descriptor_rom_violation,
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
            np.count_nonzero(joint_urad - np.asarray(v9_arrays["joint_position_urad"]))
        ),
        "emitted_hashes": emitted_hashes,
    }


def _invalid_exact_result(reason: str) -> dict[str, Any]:
    return {
        "status": "FAIL",
        "failure_reasons": [reason],
        "emitted_hashes": {},
    }


def _lift_root_yaw(
    *,
    emitted_quaternion_q1_30: NDArray[np.int64],
    source_quaternion_q1_30: NDArray[np.int64],
    source_yaw_urad: NDArray[np.int64],
) -> NDArray[np.int64]:
    if (
        emitted_quaternion_q1_30.shape != (FRAME_COUNT, 4)
        or source_quaternion_q1_30.shape != (FRAME_COUNT, 4)
        or source_yaw_urad.shape != (FRAME_COUNT,)
    ):
        raise ValueError("R115 root-yaw lift input differs")
    emitted_raw = np.asarray(
        [
            decompose_xzy(
                quaternion_to_matrix(value.astype(np.float64) / float(1 << 30))
            )[2]
            for value in emitted_quaternion_q1_30
        ]
    )
    source_raw = np.asarray(
        [
            decompose_xzy(
                quaternion_to_matrix(value.astype(np.float64) / float(1 << 30))
            )[2]
            for value in source_quaternion_q1_30
        ]
    )
    difference = emitted_raw - source_raw
    delta = np.arctan2(np.sin(difference), np.cos(difference))
    return source_yaw_urad + np.rint(delta * 1_000_000.0).astype(np.int64)


def _array_hashes(arrays: Mapping[str, NDArray[Any]]) -> dict[str, Any]:
    rows = {}
    aggregate = hashlib.sha256()
    for name in sorted(arrays, key=lambda value: value.encode("utf-8")):
        array = np.ascontiguousarray(arrays[name])
        digest = hashlib.sha256(array.tobytes()).hexdigest()
        rows[name] = {
            "sha256": digest,
            "shape": list(array.shape),
            "dtype": str(array.dtype),
        }
        aggregate.update(name.encode("utf-8"))
        aggregate.update(str(array.dtype).encode("ascii"))
        aggregate.update(json.dumps(list(array.shape), separators=(",", ":")).encode())
        aggregate.update(array.tobytes())
    return {"aggregate_sha256": aggregate.hexdigest(), "arrays": rows}


def _warm_start_cache(
    state: KtoState,
    exact: Mapping[str, Any],
    *,
    reference_rotations: NDArray[np.float64],
) -> dict[str, NDArray[Any]]:
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r115-kto-solver-private-warm-start.v1",
        "frame_count": FRAME_COUNT,
        "qva_scalar_count": FRAME_COUNT * BLOCK_WIDTH,
        "emitted_aggregate_sha256": exact["emitted_hashes"]["aggregate_sha256"],
        "candidate_or_corpus_authority": False,
    }
    return {
        "root_position_m": state.root_position_m,
        "root_orientation_delta_rad": state.root_orientation_delta_rad,
        "joint_position_rad": state.joint_position_rad,
        "velocity": state.velocity,
        "acceleration": state.acceleration,
        "reference_root_rotation": reference_rotations,
        "metadata_json_utf8": np.frombuffer(
            json.dumps(metadata, sort_keys=True, separators=(",", ":")).encode(),
            dtype=np.uint8,
        ).copy(),
    }


def _effective_joint_limits(
    descriptor: Mapping[str, Any], v9_profile: Mapping[str, Any]
) -> list[dict[str, Any]]:
    v9_bounds = v9_profile["projection"]["trajectory_closure"][
        "joint_bounds_microradians"
    ]
    result = []
    for joint in sorted(descriptor["joints"], key=lambda row: row["dof_ordinal"]):
        source_minimum, source_maximum = (
            int(value) for value in joint["soft_limit_microradians"]
        )
        local = v9_bounds.get(joint["joint_id"], [source_minimum, source_maximum])
        result.append(
            {
                "joint_id": joint["joint_id"],
                "dof_ordinal": int(joint["dof_ordinal"]),
                "position_microradians": [
                    max(source_minimum, int(local[0])),
                    min(source_maximum, int(local[1])),
                ],
            }
        )
    return result


def _contact_tolerances(profile: Mapping[str, Any]) -> ContactManifoldTolerances:
    source = profile["projection"]
    return ContactManifoldTolerances(
        maximum_tangential_step_micrometres=int(
            source["maximum_tangential_step_micrometres"]
        ),
        maximum_normal_step_micrometres=int(source["maximum_normal_step_micrometres"]),
        maximum_normal_residual_micrometres=int(
            source["maximum_normal_residual_micrometres"]
        ),
        maximum_mode_inference_height_micrometres=int(
            source["maximum_mode_inference_height_micrometres"]
        ),
        maximum_mode_inference_speed_micrometres_per_second=int(
            source["maximum_mode_inference_speed_micrometres_per_second"]
        ),
        maximum_mode_retention_height_micrometres=int(
            source["maximum_mode_retention_height_micrometres"]
        ),
        maximum_mode_retention_speed_micrometres_per_second=int(
            source["maximum_mode_retention_speed_micrometres_per_second"]
        ),
        minimum_mode_on_frames=int(source["minimum_mode_on_frames"]),
        minimum_mode_off_frames=int(source["minimum_mode_off_frames"]),
    )


def _local_q_entries(
    frame: int, coefficients: NDArray[np.float64]
) -> list[tuple[int, float]]:
    return [
        (_column(frame, 0, local), float(value))
        for local, value in enumerate(coefficients)
        if abs(float(value)) > 1.0e-14
    ]


def _column(frame: int, block: int, local: int) -> int:
    return frame * BLOCK_WIDTH + block * Q_WIDTH + local


def _parse_fraction(value: str) -> float:
    if "/" not in value:
        return float(value)
    numerator, denominator = value.split("/", maxsplit=1)
    return int(numerator) / int(denominator)


def _metadata(arrays: Mapping[str, NDArray[Any]]) -> dict[str, Any]:
    return json.loads(
        np.asarray(arrays["metadata_json_utf8"], dtype=np.uint8)
        .tobytes()
        .decode("utf-8")
    )
