from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import osqp
import scipy
from numpy.typing import NDArray
from scipy import sparse

from next_lab import contact_manifold
from next_lab.contact_manifold import (
    ContactManifoldProjection,
    ContactManifoldTolerances,
    contact_manifold_diagnostics,
    contact_point_mask,
    infer_contact_modes,
)
from next_lab.motion_math import (
    collider_minimum_y,
    decode_q1_30,
    quaternion_to_matrix,
    target_center_of_mass,
    target_effectors,
    target_forward_kinematics,
)


ALGORITHM_ID = "nextengine.dimensionless-contact-trajectory-qp.v8"
STABLE_FOOT_BOX_ALGORITHM_ID = (
    "nextengine.dimensionless-contact-trajectory-qp.v9"
)
SECOND_DIFFERENCE_ALGORITHM_ID = (
    "nextengine.dimensionless-contact-trajectory-qp.v10"
)
EMITTED_ACCELERATION_ALGORITHM_ID = (
    "nextengine.dimensionless-contact-trajectory-qp.v11"
)
SCALAR_COLLIDER_LINEARIZATION = "scalar-minimum.v1"
STABLE_FOOT_BOX_COLLIDER_LINEARIZATION = (
    "stable-contact-role-8-box-vertices.v1"
)
_ALGORITHM_LINEARIZATION = {
    ALGORITHM_ID: SCALAR_COLLIDER_LINEARIZATION,
    STABLE_FOOT_BOX_ALGORITHM_ID: STABLE_FOOT_BOX_COLLIDER_LINEARIZATION,
    SECOND_DIFFERENCE_ALGORITHM_ID: STABLE_FOOT_BOX_COLLIDER_LINEARIZATION,
    EMITTED_ACCELERATION_ALGORITHM_ID: STABLE_FOOT_BOX_COLLIDER_LINEARIZATION,
}
VELOCITY_SEMANTICS = (
    "forward on contact entry; backward on exit; centered otherwise; "
    "entry precedence"
)
_SIDES = ("left", "right")
_BOX_VERTEX_SIGNS = np.asarray(
    (
        (-1.0, -1.0, -1.0),
        (-1.0, -1.0, 1.0),
        (-1.0, 1.0, -1.0),
        (-1.0, 1.0, 1.0),
        (1.0, -1.0, -1.0),
        (1.0, -1.0, 1.0),
        (1.0, 1.0, -1.0),
        (1.0, 1.0, 1.0),
    ),
    dtype=np.float64,
)


@dataclass(frozen=True)
class CoupledTrajectoryClosure:
    """Hash-bound lab-only SQP closure for one complete reference clip."""

    algorithm_id: str
    collider_linearization_policy: str
    numpy_version: str
    scipy_version: str
    osqp_version: str
    minimum_collider_height_micrometres: int
    maximum_root_vertical_velocity_micrometres_per_second: int
    joint_velocity_limit_basis_points: int
    ordered_joint_suffixes: tuple[str, ...]
    joint_bounds_microradians: tuple[tuple[tuple[int, int], ...], ...]
    maximum_outer_iterations: int
    jacobian_probe_microradians: int
    root_variable_scale_micrometres: int
    joint_variable_scale_microradians: int
    objective_regularization: float
    first_difference_regularization: float
    second_difference_regularization: float
    emitted_acceleration_regularization: float
    maximum_solver_iterations: int
    solver_absolute_tolerance: float
    solver_relative_tolerance: float
    solver_polishing_enabled: bool
    solver_adaptive_rho_enabled: bool
    collider_target_margin_micrometres: int
    root_velocity_margin_micrometres_per_second: int
    joint_velocity_margin_numerator: int
    joint_velocity_margin_denominator: int
    normal_residual_margin_micrometres: int
    finite_normal_margin_micrometres: int
    finite_tangential_margin_micrometres: int
    analytic_normal_margin_micrometres_per_second: int
    analytic_tangential_margin_micrometres_per_second: int

    def validate(self) -> None:
        bounds = self.joint_bounds_microradians
        versions = (
            self.numpy_version,
            self.scipy_version,
            self.osqp_version,
        )
        integer_positive = (
            self.maximum_root_vertical_velocity_micrometres_per_second,
            self.joint_velocity_limit_basis_points,
            self.maximum_outer_iterations,
            self.jacobian_probe_microradians,
            self.root_variable_scale_micrometres,
            self.joint_variable_scale_microradians,
            self.maximum_solver_iterations,
            self.collider_target_margin_micrometres,
            self.root_velocity_margin_micrometres_per_second,
            self.joint_velocity_margin_numerator,
            self.joint_velocity_margin_denominator,
            self.normal_residual_margin_micrometres,
            self.finite_normal_margin_micrometres,
            self.finite_tangential_margin_micrometres,
            self.analytic_normal_margin_micrometres_per_second,
            self.analytic_tangential_margin_micrometres_per_second,
        )
        if (
            _ALGORITHM_LINEARIZATION.get(self.algorithm_id)
            != self.collider_linearization_policy
            or versions != (np.__version__, scipy.__version__, osqp.__version__)
            or any(not value for value in versions)
            or isinstance(self.minimum_collider_height_micrometres, bool)
            or not isinstance(self.minimum_collider_height_micrometres, int)
            or self.minimum_collider_height_micrometres > 0
            or any(
                isinstance(value, bool)
                or not isinstance(value, int)
                or value <= 0
                for value in integer_positive
            )
            or not 0 < self.joint_velocity_limit_basis_points <= 10_000
            or not self.ordered_joint_suffixes
            or len(self.ordered_joint_suffixes)
            != len(set(self.ordered_joint_suffixes))
            or len(bounds) != len(_SIDES)
            or any(len(side) != len(self.ordered_joint_suffixes) for side in bounds)
            or any(
                isinstance(value, bool) or not isinstance(value, int)
                for side in bounds
                for pair in side
                for value in pair
            )
            or any(pair[0] > pair[1] for side in bounds for pair in side)
            or not math.isfinite(self.objective_regularization)
            or self.objective_regularization <= 0.0
            or not math.isfinite(self.first_difference_regularization)
            or self.first_difference_regularization < 0.0
            or not math.isfinite(self.second_difference_regularization)
            or self.second_difference_regularization < 0.0
            or not math.isfinite(self.emitted_acceleration_regularization)
            or self.emitted_acceleration_regularization < 0.0
            or not math.isfinite(self.solver_absolute_tolerance)
            or self.solver_absolute_tolerance <= 0.0
            or not math.isfinite(self.solver_relative_tolerance)
            or self.solver_relative_tolerance <= 0.0
            or not isinstance(self.solver_polishing_enabled, bool)
            or not isinstance(self.solver_adaptive_rho_enabled, bool)
            or self.root_velocity_margin_micrometres_per_second
            >= self.maximum_root_vertical_velocity_micrometres_per_second
            or self.joint_velocity_margin_numerator
            >= self.joint_velocity_margin_denominator
            or self.normal_residual_margin_micrometres >= 5_000
            or self.finite_normal_margin_micrometres >= 1_000
            or self.finite_tangential_margin_micrometres >= 2_000
            or self.analytic_normal_margin_micrometres_per_second >= 60_000
            or self.analytic_tangential_margin_micrometres_per_second
            >= 120_000
        ):
            raise ValueError("coupled-trajectory closure is invalid")


def _collider_linearization_rows(
    colliders: tuple[tuple[int, dict[str, Any]], ...],
    policy: str,
) -> tuple[tuple[int, dict[str, Any], NDArray[np.float64] | None], ...]:
    """Build stable local floor samples while preserving V8 scalar rows."""

    if policy not in _ALGORITHM_LINEARIZATION.values():
        raise ValueError("unsupported collider linearization policy")
    rows: list[tuple[int, dict[str, Any], NDArray[np.float64] | None]] = []
    for slot, collider in colliders:
        geometry = collider["geometry"]
        use_stable_vertices = (
            policy == STABLE_FOOT_BOX_COLLIDER_LINEARIZATION
            and geometry["kind"] == "box"
            and int(collider["contact_role"]) == 8
        )
        if not use_stable_vertices:
            rows.append((slot, collider, None))
            continue
        local_rotation = quaternion_to_matrix(
            decode_q1_30(collider["local_rotation_q1_30"])
        )
        local_center = (
            np.asarray(
                collider["local_translation_micrometres"], dtype=np.float64
            )
            / 1_000_000.0
        )
        half_extents = (
            np.asarray(
                geometry["half_extents_micrometres"], dtype=np.float64
            )
            / 1_000_000.0
        )
        for signs in _BOX_VERTEX_SIGNS:
            rows.append(
                (
                    slot,
                    collider,
                    local_center + local_rotation @ (signs * half_extents),
                )
            )
    return tuple(rows)


def _collider_linearization_values(
    positions: NDArray[np.float64],
    rotations: NDArray[np.float64],
    rows: tuple[
        tuple[int, dict[str, Any], NDArray[np.float64] | None], ...
    ],
) -> NDArray[np.float64]:
    """Evaluate scalar rows or stable body-local support vertices."""

    return np.asarray(
        [
            (
                collider_minimum_y(
                    positions[slot], rotations[slot], collider
                )
                if local_point is None
                else float(
                    (
                        positions[slot]
                        + rotations[slot] @ local_point
                    )[1]
                )
            )
            for slot, collider, local_point in rows
        ],
        dtype=np.float64,
    )


def project_reference_coupled_trajectory(
    *,
    descriptor: dict[str, Any],
    effector_ids: tuple[str, ...],
    root_position_um: NDArray[np.int64],
    root_quaternion_q1_30: NDArray[np.int64],
    root_yaw_velocity_urad_s: NDArray[np.int64],
    joint_position_urad: NDArray[np.int64],
    effector_position_um: NDArray[np.int64],
    contacts: NDArray[np.uint8],
    support_state: NDArray[np.int64],
    frame_first: int,
    frame_last: int,
    tolerances: ContactManifoldTolerances,
    closure: CoupledTrajectoryClosure,
) -> ContactManifoldProjection:
    """Solve contact, collider, ROM and velocity bounds in one clip-wide SQP."""

    tolerances.validate()
    closure.validate()
    frame_count, joint_count = joint_position_urad.shape
    if (
        frame_count < 3
        or frame_first != 0
        or frame_last != frame_count - 1
        or root_position_um.shape != (frame_count, 3)
        or root_quaternion_q1_30.shape != (frame_count, 4)
        or root_yaw_velocity_urad_s.shape != (frame_count,)
        or effector_position_um.shape != (frame_count, len(effector_ids), 3)
        or contacts.ndim != 2
        or contacts.shape[0] != frame_count
        or contacts.shape[1] < 2
        or support_state.shape != (frame_count,)
        or joint_count != len(descriptor.get("joints", ()))
    ):
        raise ValueError("coupled-trajectory source shape mismatch")

    modes = infer_contact_modes(
        effector_ids=effector_ids,
        effector_position_um=effector_position_um,
        support_state=support_state,
        tolerances=tolerances,
    )
    active = contact_point_mask(modes)
    stencil_indices, stencil_coefficients = hybrid_velocity_stencil(active)
    sole_indices = contact_manifold._sole_effector_indices(effector_ids)
    joint_lookup = {
        joint["joint_id"]: int(joint["dof_ordinal"])
        for joint in descriptor["joints"]
    }
    selected_ids = tuple(
        tuple(f"joint.{side}-{suffix}" for suffix in closure.ordered_joint_suffixes)
        for side in _SIDES
    )
    if any(
        joint_id not in joint_lookup
        for side_ids in selected_ids
        for joint_id in side_ids
    ):
        raise ValueError("coupled-trajectory joint identity mismatch")
    selected = np.asarray(
        [[joint_lookup[joint_id] for joint_id in side] for side in selected_ids],
        dtype=np.int64,
    )
    selected_flat = selected.reshape(-1)
    selected_id_flat = tuple(value for side in selected_ids for value in side)
    local_variable_count = 3 + len(selected_flat)
    variable_count = frame_count * local_variable_count
    local_scale = np.concatenate(
        (
            np.full(
                3,
                closure.root_variable_scale_micrometres / 1_000_000.0,
                dtype=np.float64,
            ),
            np.full(
                len(selected_flat),
                closure.joint_variable_scale_microradians / 1_000_000.0,
                dtype=np.float64,
            ),
        )
    )
    all_colliders, _ = contact_manifold._collider_inventory(descriptor)
    collider_linearization_rows = _collider_linearization_rows(
        all_colliders, closure.collider_linearization_policy
    )
    collider_count = len(collider_linearization_rows)
    stable_vertex_row_count = sum(
        local_point is not None
        for _, _, local_point in collider_linearization_rows
    )
    collider_floor = closure.minimum_collider_height_micrometres / 1_000_000.0
    collider_target = (
        closure.collider_target_margin_micrometres / 1_000_000.0
    )
    root_velocity_bound = (
        closure.maximum_root_vertical_velocity_micrometres_per_second
        - closure.root_velocity_margin_micrometres_per_second
    ) / 1_000_000.0
    bound_map = {
        joint_id: closure.joint_bounds_microradians[side][column]
        for side, side_ids in enumerate(selected_ids)
        for column, joint_id in enumerate(side_ids)
    }
    joint_minimum = np.asarray(
        [bound_map[value][0] for value in selected_id_flat], dtype=np.float64
    ) / 1_000_000.0
    joint_maximum = np.asarray(
        [bound_map[value][1] for value in selected_id_flat], dtype=np.float64
    ) / 1_000_000.0
    joint_by_ordinal = {
        int(joint["dof_ordinal"]): joint for joint in descriptor["joints"]
    }
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
            * closure.joint_velocity_margin_numerator
            / closure.joint_velocity_margin_denominator
            for ordinal in selected_flat
        ],
        dtype=np.float64,
    )
    root = root_position_um.astype(np.float64) / 1_000_000.0
    joints = joint_position_urad.astype(np.float64) / 1_000_000.0
    quaternions = root_quaternion_q1_30.astype(np.float64) / float(1 << 30)

    def velocity(values: NDArray[np.float64]) -> NDArray[np.float64]:
        weights = stencil_coefficients.reshape(
            (frame_count, 2) + (1,) * (values.ndim - 1)
        )
        return np.sum(values[stencil_indices] * weights, axis=1)

    def collider_values(
        positions: NDArray[np.float64], rotations: NDArray[np.float64]
    ) -> NDArray[np.float64]:
        return _collider_linearization_values(
            positions, rotations, collider_linearization_rows
        )

    def linearization() -> tuple[
        NDArray[np.float64],
        NDArray[np.float64],
        NDArray[np.float64],
        NDArray[np.float64],
        NDArray[np.float64],
    ]:
        effectors = np.empty((frame_count, 2, 2, 3), dtype=np.float64)
        effector_jacobian = np.zeros(
            (frame_count, 2, 2, 3, local_variable_count), dtype=np.float64
        )
        effector_jacobian[..., 0, 0] = 1.0
        effector_jacobian[..., 1, 1] = 1.0
        effector_jacobian[..., 2, 2] = 1.0
        collider_heights = np.empty(
            (frame_count, collider_count), dtype=np.float64
        )
        collider_jacobian = np.zeros(
            (frame_count, collider_count, local_variable_count), dtype=np.float64
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
            collider_heights[frame] = collider_values(positions, rotations)
            for column, ordinal in enumerate(selected_flat, start=3):
                candidate = joints[frame].copy()
                candidate[int(ordinal)] += probe
                candidate_positions, candidate_rotations = (
                    target_forward_kinematics(
                        descriptor,
                        root[frame],
                        quaternions[frame],
                        candidate,
                    )
                )
                candidate_effectors = target_effectors(
                    descriptor, candidate_positions, candidate_rotations
                )
                for side in range(2):
                    for point in range(2):
                        effector_jacobian[frame, side, point, :, column] = (
                            candidate_effectors[
                                effector_ids[int(sole_indices[side, point])]
                            ]
                            - effectors[frame, side, point]
                        ) / probe
                collider_jacobian[frame, :, column] = (
                    collider_values(candidate_positions, candidate_rotations)
                    - collider_heights[frame]
                ) / probe
        root_um = np.rint(root * 1_000_000.0).astype(np.int64)
        joint_um = np.rint(joints * 1_000_000.0).astype(np.int64)
        root_velocity_um = np.rint(velocity(root_um.astype(np.float64))).astype(
            np.int64
        )
        joint_velocity_um = np.rint(
            velocity(joint_um.astype(np.float64))
        ).astype(np.int64)
        analytic_velocity = contact_manifold._analytic_active_point_velocities(
            descriptor=descriptor,
            root_positions=root,
            root_quaternions=quaternions,
            joint_positions=joints,
            root_linear_velocity_um_s=root_velocity_um,
            root_yaw_velocity_urad_s=root_yaw_velocity_urad_s,
            joint_velocity_urad_s=joint_velocity_um,
            active=active,
            probe=probe,
        ) / 1_000_000.0
        return (
            effectors,
            effector_jacobian,
            collider_heights,
            collider_jacobian,
            analytic_velocity,
        )

    emitted_acceleration_operator = (
        emitted_joint_acceleration_operator(
            frame_count=frame_count,
            local_variable_count=local_variable_count,
            joint_local_indices=np.arange(
                3, local_variable_count, dtype=np.int64
            ),
            stencil_indices=stencil_indices,
            stencil_coefficients=stencil_coefficients,
        )
        if closure.emitted_acceleration_regularization > 0.0
        else None
    )
    objective = _objective_matrix(
        frame_count=frame_count,
        local_variable_count=local_variable_count,
        objective_regularization=closure.objective_regularization,
        first_difference_regularization=closure.first_difference_regularization,
        second_difference_regularization=(
            closure.second_difference_regularization
        ),
        emitted_acceleration_regularization=(
            closure.emitted_acceleration_regularization
        ),
        emitted_acceleration_operator=emitted_acceleration_operator,
    )
    history: list[dict[str, Any]] = []
    final_state: dict[str, Any] | None = None
    final_root_um: NDArray[np.int64] | None = None
    final_joint_um: NDArray[np.int64] | None = None
    final_effector_um: NDArray[np.int64] | None = None
    final_center_um: NDArray[np.int64] | None = None
    final_root_velocity_um: NDArray[np.int64] | None = None
    final_joint_velocity_um: NDArray[np.int64] | None = None
    last_categories: dict[str, int] = {}
    termination = "maximum_outer_iterations"
    for outer in range(1, closure.maximum_outer_iterations + 1):
        (
            sole_effectors,
            effector_jacobian,
            collider_heights,
            collider_jacobian,
            analytic_velocity,
        ) = linearization()
        matrix, lower, upper, categories = _build_qp(
            frame_count=frame_count,
            local_variable_count=local_variable_count,
            variable_count=variable_count,
            local_scale=local_scale,
            root_values=root,
            joint_values=joints,
            effectors=sole_effectors,
            effector_jacobian=effector_jacobian,
            collider_heights=collider_heights,
            collider_jacobian=collider_jacobian,
            analytic_velocity=analytic_velocity,
            active=active,
            stencil_indices=stencil_indices,
            stencil_coefficients=stencil_coefficients,
            collider_target=collider_target,
            root_velocity_bound=root_velocity_bound,
            joint_velocity_bounds=joint_velocity_bounds,
            selected_flat=selected_flat,
            joint_minimum=joint_minimum,
            joint_maximum=joint_maximum,
            tolerances=tolerances,
            closure=closure,
        )
        last_categories = categories
        objective_linear = np.zeros(variable_count, dtype=np.float64)
        if emitted_acceleration_operator is not None:
            normalized_state = np.concatenate(
                (
                    root
                    / (
                        closure.root_variable_scale_micrometres
                        / 1_000_000.0
                    ),
                    joints[:, selected_flat]
                    / (
                        closure.joint_variable_scale_microradians
                        / 1_000_000.0
                    ),
                ),
                axis=1,
            ).reshape(-1)
            current_acceleration = (
                emitted_acceleration_operator @ normalized_state
            )
            objective_linear = np.asarray(
                closure.emitted_acceleration_regularization
                * emitted_acceleration_operator.T
                @ current_acceleration
            ).reshape(-1)
        solver = osqp.OSQP()
        solver.setup(
            P=objective,
            q=objective_linear,
            A=matrix,
            l=lower,
            u=upper,
            verbose=False,
            eps_abs=closure.solver_absolute_tolerance,
            eps_rel=closure.solver_relative_tolerance,
            max_iter=closure.maximum_solver_iterations,
            polishing=closure.solver_polishing_enabled,
            adaptive_rho=closure.solver_adaptive_rho_enabled,
        )
        result = solver.solve(raise_error=False)
        status = str(result.info.status)
        iteration: dict[str, Any] = {
            "outer_iteration": outer,
            "qp_status": status,
            "qp_iterations": int(result.info.iter),
            "qp_primal_residual": float(result.info.prim_res),
            "qp_dual_residual": float(result.info.dual_res),
            "constraint_count": int(matrix.shape[0]),
            "constraint_nonzero_count": int(matrix.nnz),
            "constraint_categories": categories,
        }
        if result.x is None or not status.lower().startswith("solved"):
            history.append(iteration)
            termination = "solver_failure"
            break
        normalized_increment = result.x.reshape(
            frame_count, local_variable_count
        )
        physical_increment = normalized_increment * local_scale[None, :]
        root += physical_increment[:, :3]
        joints[:, selected_flat] += physical_increment[:, 3:]
        (
            final_state,
            final_root_um,
            final_joint_um,
            final_effector_um,
            final_center_um,
            final_root_velocity_um,
            final_joint_velocity_um,
        ) = _exact_state(
            descriptor=descriptor,
            effector_ids=effector_ids,
            root_values=root,
            quaternion_q1_30=root_quaternion_q1_30,
            yaw_velocity_urad_s=root_yaw_velocity_urad_s,
            joint_values=joints,
            modes=modes,
            velocity=velocity,
            collider_floor=collider_floor,
            tolerances=tolerances,
            closure=closure,
        )
        iteration.update(
            {
                "maximum_root_update_micrometres": int(
                    np.ceil(
                        np.max(np.linalg.norm(physical_increment[:, :3], axis=1))
                        * 1_000_000.0
                    )
                ),
                "maximum_joint_update_microradians": int(
                    np.ceil(np.max(np.abs(physical_increment[:, 3:])) * 1_000_000.0)
                ),
                "exact_status": final_state["status"],
                "minimum_collider_height_micrometres": final_state[
                    "minimum_collider_height_micrometres"
                ],
                "maximum_normal_residual_micrometres": final_state[
                    "contact"
                ]["maximum_normal_residual_micrometres"],
            }
        )
        history.append(iteration)
        if final_state["status"] == "PASS":
            termination = "exact_quantized_pass"
            break

    if final_state is None:
        (
            final_state,
            final_root_um,
            final_joint_um,
            final_effector_um,
            final_center_um,
            final_root_velocity_um,
            final_joint_velocity_um,
        ) = _exact_state(
            descriptor=descriptor,
            effector_ids=effector_ids,
            root_values=root,
            quaternion_q1_30=root_quaternion_q1_30,
            yaw_velocity_urad_s=root_yaw_velocity_urad_s,
            joint_values=joints,
            modes=modes,
            velocity=velocity,
            collider_floor=collider_floor,
            tolerances=tolerances,
            closure=closure,
        )
    assert final_root_um is not None
    assert final_joint_um is not None
    assert final_effector_um is not None
    assert final_center_um is not None
    assert final_root_velocity_um is not None
    assert final_joint_velocity_um is not None

    solved_contacts = contacts.copy()
    solved_contacts[:, :2] = np.any(active, axis=2).astype(np.uint8)
    root_correction = final_root_um - root_position_um
    joint_correction = final_joint_um - joint_position_urad
    overall_status = (
        "PASS"
        if final_state["status"] == "PASS"
        and termination == "exact_quantized_pass"
        else "FAIL"
    )
    diagnostics = {
        **final_state["contact"],
        "active_contact_status": final_state["contact"]["status"],
        "collider_closure_status": final_state["collider_closure_status"],
        "minimum_collider_height_micrometres": final_state[
            "minimum_collider_height_micrometres"
        ],
        "violating_collider_sample_count": final_state[
            "violating_collider_sample_count"
        ],
        "maximum_root_vertical_velocity_micrometres_per_second": final_state[
            "maximum_root_vertical_velocity_micrometres_per_second"
        ],
        "maximum_joint_velocity_basis_points": final_state[
            "maximum_joint_velocity_basis_points"
        ],
        "maximum_soft_rom_violation_microradians": final_state[
            "maximum_soft_rom_violation_microradians"
        ],
        "contact_point_deletion_count": 0,
        "contact_point_policy": (
            "all point-consistent source modes are frozen before the solve"
        ),
        "status": overall_status,
        "frame_first": frame_first,
        "frame_last": frame_last,
        "maximum_root_correction_micrometres": int(
            np.max(np.linalg.norm(root_correction, axis=1))
        ),
        "maximum_root_correction_step_micrometres": int(
            np.max(np.linalg.norm(np.diff(root_correction, axis=0), axis=1))
        ),
        "maximum_joint_correction_microradians": int(
            np.max(np.abs(joint_correction))
        ),
        "mode_counts": {
            contact_manifold.CONTACT_MODE_NAMES[value]: int(np.sum(modes == value))
            for value in range(len(contact_manifold.CONTACT_MODE_NAMES))
        },
        "trajectory_solver": {
            "algorithm_id": closure.algorithm_id,
            "objective_regularization": closure.objective_regularization,
            "first_difference_regularization": (
                closure.first_difference_regularization
            ),
            "second_difference_regularization": (
                closure.second_difference_regularization
            ),
            "emitted_acceleration_regularization": (
                closure.emitted_acceleration_regularization
            ),
            "emitted_acceleration_semantics": (
                "first difference of the frozen hybrid 60 Hz velocity "
                "stencil over normalized selected-joint final poses"
            ),
            "termination": termination,
            "outer_iterations_completed": len(history),
            "maximum_outer_iterations": closure.maximum_outer_iterations,
            "velocity_semantics": VELOCITY_SEMANTICS,
            "collider_linearization_policy": (
                closure.collider_linearization_policy
            ),
            "exact_collider_count": len(all_colliders),
            "collider_linearization_row_count_per_frame": collider_count,
            "stable_vertex_row_count_per_frame": stable_vertex_row_count,
            "constraint_categories": last_categories,
            "post_root_velocity_closure": False,
            "post_joint_velocity_projection": False,
            "all_collider_samples_constrained": True,
            "runtime_versions": {
                "numpy": np.__version__,
                "scipy": scipy.__version__,
                "osqp": osqp.__version__,
            },
            "history": history,
        },
    }
    return ContactManifoldProjection(
        frame_first=frame_first,
        frame_last=frame_last,
        root_position_um=final_root_um,
        root_linear_velocity_um_s=final_root_velocity_um,
        root_yaw_velocity_urad_s=root_yaw_velocity_urad_s.copy(),
        joint_position_urad=final_joint_um,
        joint_velocity_urad_s=final_joint_velocity_um,
        center_of_mass_um=final_center_um,
        effector_position_um=final_effector_um,
        contacts=solved_contacts,
        contact_modes=modes,
        diagnostics=diagnostics,
    )


def hybrid_velocity_stencil(
    active: NDArray[np.bool_],
) -> tuple[NDArray[np.int64], NDArray[np.float64]]:
    """Return the fixed 60 Hz entry/exit-aware two-sample stencil."""

    if active.ndim != 3 or active.shape[1:] != (2, 2) or len(active) < 3:
        raise ValueError("hybrid velocity contact mask is invalid")
    frame_count = len(active)
    side_active = np.any(active, axis=2)
    onset = side_active & np.vstack(
        [np.zeros((1, 2), dtype=np.bool_), ~side_active[:-1]]
    )
    ending = side_active & np.vstack(
        [~side_active[1:], np.zeros((1, 2), dtype=np.bool_)]
    )
    indices = np.empty((frame_count, 2), dtype=np.int64)
    coefficients = np.empty((frame_count, 2), dtype=np.float64)
    for frame in range(frame_count):
        if frame == 0:
            indices[frame] = (0, 1)
            coefficients[frame] = (-60.0, 60.0)
        elif frame == frame_count - 1:
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


def emitted_joint_acceleration_operator(
    *,
    frame_count: int,
    local_variable_count: int,
    joint_local_indices: NDArray[np.int64],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
) -> sparse.csc_matrix:
    """Map normalized poses to differences of emitted 60 Hz velocity."""

    local_indices = np.asarray(joint_local_indices, dtype=np.int64)
    if (
        frame_count < 3
        or local_variable_count <= 0
        or local_indices.ndim != 1
        or len(local_indices) == 0
        or len(set(int(value) for value in local_indices)) != len(local_indices)
        or np.any(local_indices < 0)
        or np.any(local_indices >= local_variable_count)
        or stencil_indices.shape != (frame_count, 2)
        or stencil_coefficients.shape != (frame_count, 2)
    ):
        raise ValueError("emitted-acceleration operator identity is invalid")
    joint_count = len(local_indices)
    rows: list[int] = []
    columns: list[int] = []
    values: list[float] = []
    for frame in range(1, frame_count):
        for joint_column, local in enumerate(local_indices):
            row = (frame - 1) * joint_count + joint_column
            for sign, velocity_frame in ((-1.0, frame - 1), (1.0, frame)):
                for sample in range(2):
                    source_frame = int(stencil_indices[velocity_frame, sample])
                    rows.append(row)
                    columns.append(
                        source_frame * local_variable_count + int(local)
                    )
                    values.append(
                        sign
                        * float(stencil_coefficients[velocity_frame, sample])
                        / 60.0
                    )
    return sparse.coo_matrix(
        (values, (rows, columns)),
        shape=(
            (frame_count - 1) * joint_count,
            frame_count * local_variable_count,
        ),
    ).tocsc()


def _objective_matrix(
    *,
    frame_count: int,
    local_variable_count: int,
    objective_regularization: float,
    first_difference_regularization: float,
    second_difference_regularization: float,
    emitted_acceleration_regularization: float,
    emitted_acceleration_operator: sparse.csc_matrix | None,
) -> sparse.csc_matrix:
    variable_count = frame_count * local_variable_count
    result = objective_regularization * sparse.eye(variable_count, format="csc")
    if first_difference_regularization > 0.0:
        rows = np.repeat(np.arange((frame_count - 1) * local_variable_count), 2)
        columns = np.empty(len(rows), dtype=np.int64)
        values = np.empty(len(rows), dtype=np.float64)
        cursor = 0
        for frame in range(frame_count - 1):
            for local in range(local_variable_count):
                columns[cursor : cursor + 2] = (
                    frame * local_variable_count + local,
                    (frame + 1) * local_variable_count + local,
                )
                values[cursor : cursor + 2] = (-1.0, 1.0)
                cursor += 2
        difference = sparse.coo_matrix(
            (values, (rows, columns)),
            shape=((frame_count - 1) * local_variable_count, variable_count),
        ).tocsc()
        result = result + first_difference_regularization * (
            difference.T @ difference
        )
    if second_difference_regularization > 0.0:
        rows = np.repeat(
            np.arange((frame_count - 2) * local_variable_count), 3
        )
        columns = np.empty(len(rows), dtype=np.int64)
        values = np.empty(len(rows), dtype=np.float64)
        cursor = 0
        for frame in range(frame_count - 2):
            for local in range(local_variable_count):
                columns[cursor : cursor + 3] = (
                    frame * local_variable_count + local,
                    (frame + 1) * local_variable_count + local,
                    (frame + 2) * local_variable_count + local,
                )
                values[cursor : cursor + 3] = (1.0, -2.0, 1.0)
                cursor += 3
        curvature = sparse.coo_matrix(
            (values, (rows, columns)),
            shape=((frame_count - 2) * local_variable_count, variable_count),
        ).tocsc()
        result = result + second_difference_regularization * (
            curvature.T @ curvature
        )
    if emitted_acceleration_regularization > 0.0:
        if emitted_acceleration_operator is None:
            raise ValueError("emitted-acceleration operator is absent")
        result = result + emitted_acceleration_regularization * (
            emitted_acceleration_operator.T @ emitted_acceleration_operator
        )
    elif emitted_acceleration_operator is not None:
        raise ValueError("unexpected emitted-acceleration operator")
    return sparse.triu(result, format="csc")


def _build_qp(
    *,
    frame_count: int,
    local_variable_count: int,
    variable_count: int,
    local_scale: NDArray[np.float64],
    root_values: NDArray[np.float64],
    joint_values: NDArray[np.float64],
    effectors: NDArray[np.float64],
    effector_jacobian: NDArray[np.float64],
    collider_heights: NDArray[np.float64],
    collider_jacobian: NDArray[np.float64],
    analytic_velocity: NDArray[np.float64],
    active: NDArray[np.bool_],
    stencil_indices: NDArray[np.int64],
    stencil_coefficients: NDArray[np.float64],
    collider_target: float,
    root_velocity_bound: float,
    joint_velocity_bounds: NDArray[np.float64],
    selected_flat: NDArray[np.int64],
    joint_minimum: NDArray[np.float64],
    joint_maximum: NDArray[np.float64],
    tolerances: ContactManifoldTolerances,
    closure: CoupledTrajectoryClosure,
) -> tuple[sparse.csc_matrix, NDArray[np.float64], NDArray[np.float64], dict[str, int]]:
    rows: list[int] = []
    columns: list[int] = []
    data: list[float] = []
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
            local_column = column % local_variable_count
            rows.append(row)
            columns.append(column)
            data.append(coefficient * local_scale[local_column] / row_scale)
        lower.append(low / row_scale)
        upper.append(high / row_scale)
        categories[category] = categories.get(category, 0) + 1

    def local_entries(
        frame: int, values: NDArray[np.float64]
    ) -> list[tuple[int, float]]:
        base = frame * local_variable_count
        return [
            (base + column, float(value))
            for column, value in enumerate(values)
            if abs(float(value)) > 1.0e-14
        ]

    normal_residual_bound = (
        tolerances.maximum_normal_residual_micrometres
        - closure.normal_residual_margin_micrometres
    ) / 1_000_000.0
    finite_normal_bound = (
        tolerances.maximum_normal_step_micrometres
        - closure.finite_normal_margin_micrometres
    ) / 1_000_000.0
    finite_tangent_component_bound = (
        tolerances.maximum_tangential_step_micrometres
        - closure.finite_tangential_margin_micrometres
    ) / 1_000_000.0 / math.sqrt(2.0)
    analytic_normal_bound = (
        tolerances.maximum_normal_step_micrometres * 60
        - closure.analytic_normal_margin_micrometres_per_second
    ) / 1_000_000.0
    analytic_tangent_component_bound = (
        tolerances.maximum_tangential_step_micrometres * 60
        - closure.analytic_tangential_margin_micrometres_per_second
    ) / 1_000_000.0 / math.sqrt(2.0)
    for frame, side, point in np.argwhere(active):
        frame = int(frame)
        side = int(side)
        point = int(point)
        current = float(effectors[frame, side, point, 1])
        add_row(
            local_entries(frame, effector_jacobian[frame, side, point, 1]),
            -normal_residual_bound - current,
            normal_residual_bound - current,
            normal_residual_bound,
            "contact_normal_residual",
        )
    shared = active[1:] & active[:-1]
    component_bounds = (
        finite_tangent_component_bound,
        finite_normal_bound,
        finite_tangent_component_bound,
    )
    for frame_minus_one, side, point in np.argwhere(shared):
        previous = int(frame_minus_one)
        frame = previous + 1
        side = int(side)
        point = int(point)
        difference = effectors[frame, side, point] - effectors[previous, side, point]
        for component, bound in enumerate(component_bounds):
            entries = local_entries(
                frame, effector_jacobian[frame, side, point, component]
            )
            entries.extend(
                local_entries(
                    previous,
                    -effector_jacobian[previous, side, point, component],
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
        analytic_tangent_component_bound,
        analytic_normal_bound,
        analytic_tangent_component_bound,
    )
    for frame, side, point in np.argwhere(active):
        frame = int(frame)
        side = int(side)
        point = int(point)
        for component, bound in enumerate(analytic_bounds):
            entries: list[tuple[int, float]] = []
            for slot in range(2):
                sample = int(stencil_indices[frame, slot])
                coefficient = float(stencil_coefficients[frame, slot])
                entries.append(
                    (sample * local_variable_count + component, coefficient)
                )
                for local_joint, jacobian_value in enumerate(
                    effector_jacobian[frame, side, point, component, 3:], start=3
                ):
                    if abs(float(jacobian_value)) > 1.0e-14:
                        entries.append(
                            (
                                sample * local_variable_count + local_joint,
                                coefficient * float(jacobian_value),
                            )
                        )
            current = float(analytic_velocity[frame, side, point, component])
            add_row(
                entries,
                -bound - current,
                bound - current,
                bound,
                "contact_analytic_velocity",
            )
    for frame in range(frame_count):
        for collider in range(len(collider_heights[frame])):
            current = float(collider_heights[frame, collider])
            add_row(
                local_entries(frame, collider_jacobian[frame, collider]),
                collider_target - current,
                np.inf,
                0.005,
                "collider_floor",
            )

    def velocity(values: NDArray[np.float64]) -> NDArray[np.float64]:
        weights = stencil_coefficients.reshape(
            (frame_count, 2) + (1,) * (values.ndim - 1)
        )
        return np.sum(values[stencil_indices] * weights, axis=1)

    root_velocity = velocity(root_values[:, 1:2])[:, 0]
    for frame in range(frame_count):
        entries = [
            (
                int(stencil_indices[frame, slot]) * local_variable_count + 1,
                float(stencil_coefficients[frame, slot]),
            )
            for slot in range(2)
        ]
        current = float(root_velocity[frame])
        add_row(
            entries,
            -root_velocity_bound - current,
            root_velocity_bound - current,
            root_velocity_bound,
            "root_vertical_velocity",
        )
    selected_joint_velocity = velocity(joint_values[:, selected_flat])
    for frame in range(frame_count):
        for joint_column, bound in enumerate(joint_velocity_bounds):
            entries = [
                (
                    int(stencil_indices[frame, slot]) * local_variable_count
                    + 3
                    + joint_column,
                    float(stencil_coefficients[frame, slot]),
                )
                for slot in range(2)
            ]
            current = float(selected_joint_velocity[frame, joint_column])
            add_row(
                entries,
                -float(bound) - current,
                float(bound) - current,
                float(bound),
                "joint_velocity",
            )
    for frame in range(frame_count):
        for joint_column in range(len(selected_flat)):
            current = float(joint_values[frame, selected_flat[joint_column]])
            add_row(
                [
                    (
                        frame * local_variable_count + 3 + joint_column,
                        1.0,
                    )
                ],
                float(joint_minimum[joint_column]) - current,
                float(joint_maximum[joint_column]) - current,
                0.10,
                "joint_rom",
            )
    matrix = sparse.coo_matrix(
        (np.asarray(data), (np.asarray(rows), np.asarray(columns))),
        shape=(len(lower), variable_count),
    ).tocsc()
    return (
        matrix,
        np.asarray(lower, dtype=np.float64),
        np.asarray(upper, dtype=np.float64),
        categories,
    )


def _exact_state(
    *,
    descriptor: dict[str, Any],
    effector_ids: tuple[str, ...],
    root_values: NDArray[np.float64],
    quaternion_q1_30: NDArray[np.int64],
    yaw_velocity_urad_s: NDArray[np.int64],
    joint_values: NDArray[np.float64],
    modes: NDArray[np.uint8],
    velocity: Any,
    collider_floor: float,
    tolerances: ContactManifoldTolerances,
    closure: CoupledTrajectoryClosure,
) -> tuple[
    dict[str, Any],
    NDArray[np.int64],
    NDArray[np.int64],
    NDArray[np.int64],
    NDArray[np.int64],
    NDArray[np.int64],
    NDArray[np.int64],
]:
    root_um = np.rint(root_values * 1_000_000.0).astype(np.int64)
    joint_um = np.rint(joint_values * 1_000_000.0).astype(np.int64)
    frame_count = len(root_um)
    quaternions = quaternion_q1_30.astype(np.float64) / float(1 << 30)
    quantized_root = root_um.astype(np.float64) / 1_000_000.0
    quantized_joints = joint_um.astype(np.float64) / 1_000_000.0
    all_colliders, _ = contact_manifold._collider_inventory(descriptor)
    effectors = np.empty(
        (frame_count, len(effector_ids), 3), dtype=np.float64
    )
    centers = np.empty((frame_count, 3), dtype=np.float64)
    collider_heights = np.empty(
        (frame_count, len(all_colliders)), dtype=np.float64
    )
    for frame in range(frame_count):
        positions, rotations = target_forward_kinematics(
            descriptor,
            quantized_root[frame],
            quaternions[frame],
            quantized_joints[frame],
        )
        values = target_effectors(descriptor, positions, rotations)
        effectors[frame] = np.stack(
            [values[value] for value in effector_ids]
        )
        centers[frame] = target_center_of_mass(
            descriptor, positions, rotations
        )
        collider_heights[frame] = np.asarray(
            [
                collider_minimum_y(
                    positions[slot], rotations[slot], collider
                )
                for slot, collider in all_colliders
            ],
            dtype=np.float64,
        )
    effector_um = np.rint(effectors * 1_000_000.0).astype(np.int64)
    center_um = np.rint(centers * 1_000_000.0).astype(np.int64)
    root_velocity_um = np.rint(velocity(root_um.astype(np.float64))).astype(
        np.int64
    )
    joint_velocity_um = np.rint(
        velocity(joint_um.astype(np.float64))
    ).astype(np.int64)
    contact = contact_manifold_diagnostics(
        descriptor=descriptor,
        effector_ids=effector_ids,
        root_position_um=root_um,
        root_quaternion_q1_30=quaternion_q1_30,
        root_linear_velocity_um_s=root_velocity_um,
        root_yaw_velocity_urad_s=yaw_velocity_urad_s,
        joint_position_urad=joint_um,
        joint_velocity_urad_s=joint_velocity_um,
        effector_position_um=effector_um,
        contact_modes=modes,
        tolerances=tolerances,
    )
    maximum_joint_basis_points = 0
    maximum_soft_rom_violation = 0
    for joint in descriptor["joints"]:
        ordinal = int(joint["dof_ordinal"])
        maximum_velocity = int(
            joint["maximum_velocity_microradians_per_second"]
        )
        maximum_joint_basis_points = max(
            maximum_joint_basis_points,
            int(
                np.ceil(
                    np.max(np.abs(joint_velocity_um[:, ordinal]))
                    * 10_000.0
                    / maximum_velocity
                )
            ),
        )
        minimum, maximum = (
            int(value) for value in joint["soft_limit_microradians"]
        )
        maximum_soft_rom_violation = max(
            maximum_soft_rom_violation,
            int(np.max(np.maximum(minimum - joint_um[:, ordinal], 0))),
            int(np.max(np.maximum(joint_um[:, ordinal] - maximum, 0))),
        )
    minimum_collider_height = int(
        np.floor(np.min(collider_heights) * 1_000_000.0 + 1.0e-9)
    )
    maximum_root_vertical_velocity = int(
        np.max(np.abs(root_velocity_um[:, 1]))
    )
    collider_status = (
        "PASS"
        if minimum_collider_height
        >= closure.minimum_collider_height_micrometres
        and maximum_root_vertical_velocity
        <= closure.maximum_root_vertical_velocity_micrometres_per_second
        and maximum_joint_basis_points
        <= closure.joint_velocity_limit_basis_points
        and maximum_soft_rom_violation == 0
        else "FAIL"
    )
    state = {
        "contact": contact,
        "collider_closure_status": collider_status,
        "minimum_collider_height_micrometres": minimum_collider_height,
        "violating_collider_sample_count": int(
            np.sum(collider_heights < collider_floor)
        ),
        "maximum_root_vertical_velocity_micrometres_per_second": (
            maximum_root_vertical_velocity
        ),
        "maximum_joint_velocity_basis_points": maximum_joint_basis_points,
        "maximum_soft_rom_violation_microradians": maximum_soft_rom_violation,
    }
    state["status"] = (
        "PASS"
        if contact["status"] == "PASS" and collider_status == "PASS"
        else "FAIL"
    )
    return (
        state,
        root_um,
        joint_um,
        effector_um,
        center_um,
        root_velocity_um,
        joint_velocity_um,
    )
