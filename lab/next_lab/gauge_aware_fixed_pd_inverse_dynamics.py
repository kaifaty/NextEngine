from __future__ import annotations

import math
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.fixed_pd_inverse_dynamics_execution import (
    CollocationState,
    point_active,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    GENERALIZED_WIDTH,
    POINT_COUNT,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    SpatialModel,
    generalized_contact_force,
    inverse_dynamics,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    propagate_motion,
)

POINT_FORCE_WIDTH = 3
FRICTION_NUMERATOR = 52_429
FRICTION_DENOMINATOR = 65_536
FRICTION = FRICTION_NUMERATOR / FRICTION_DENOMINATOR


@dataclass(frozen=True)
class GaugeIdentity:
    side: int
    heel_point_ordinal: int
    forefoot_point_ordinal: int
    world_direction: NDArray[np.float64]
    maximum_resultant_force: float
    maximum_resultant_moment: float


@dataclass(frozen=True)
class ReducedLocalSystem:
    matrix: NDArray[np.float64]
    right_hand_side: NDArray[np.float64]
    active_point_ordinals: tuple[int, ...]
    modes: NDArray[np.uint8]
    analytic_gauge_matrix: NDArray[np.float64]
    gauge_identities: tuple[GaugeIdentity, ...]


@dataclass(frozen=True)
class RankAnalysis:
    status: str
    invalid_reason: str | None
    rank: int
    nullity: int
    expected_nullity: int
    relative_singular_values: NDArray[np.float64]
    smallest_retained_relative_singular_value: float | None
    largest_null_relative_singular_value: float | None
    analytic_gauge_scaled_residual: float
    analytic_to_svd_projector_spectral_error: float
    particular_solution: NDArray[np.float64] | None
    scaled_absolute_residual: float | None
    backward_error: float | None
    dynamics_absolute_residual: float | None
    closure_absolute_residual: float | None


@dataclass(frozen=True)
class LineConeInterval:
    status: str
    invalid_reason: str | None
    empty: bool
    lower: float | None
    upper: float | None
    quadratic_coefficients: tuple[float, float, float]


@dataclass(frozen=True)
class GaugeFeasibility:
    status: str
    invalid_reason: str | None
    feasible: bool | None
    lower: float | None
    upper: float | None
    unconstrained_minimum_force_alpha: float | None
    witness_alpha: float | None
    witness_forces: NDArray[np.float64] | None
    minimum_normal_margin_newtons: float | None
    minimum_friction_margin_newtons: float | None


def build_reduced_local_system(
    *,
    model: SpatialModel,
    state: CollocationState,
    effort_newton_metres: NDArray[np.float64],
    modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
) -> ReducedLocalSystem:
    """Build the R125 29/32/35 equality system without deleting contact rows."""

    mass, kinematics = mass_matrix(model, state.configuration)
    zero = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
    bias = inverse_dynamics(model, state.configuration, state.velocity, zero)
    motion = propagate_motion(model, kinematics, state.velocity, zero)
    jacobians = []
    jdot_v = []
    active = np.zeros(POINT_COUNT, dtype=np.bool_)
    for ordinal, point in enumerate(points):
        body_slot = int(point["body_slot"])
        local = np.asarray(point["local_translation_metres"], dtype=np.float64)
        jacobians.append(point_jacobian(model, kinematics, body_slot, local))
        jdot_v.append(point_acceleration(kinematics, motion, body_slot, local))
        active[ordinal] = point_active(modes, ordinal)

    active_ordinals = tuple(int(value) for value in np.flatnonzero(active))
    matrix, right = assemble_reduced_local_system(
        mass=mass,
        bias=bias,
        effort_newton_metres=effort_newton_metres,
        contact_jacobians=np.stack(jacobians),
        contact_jdot_v=np.stack(jdot_v),
        active_point_ordinals=active_ordinals,
        modes=modes,
    )
    gauges, identities = analytic_force_gauges(
        modes=modes,
        points=points,
        active_point_ordinals=active_ordinals,
        body_rotations=kinematics.body_rotations,
        body_positions=kinematics.body_positions,
    )
    return ReducedLocalSystem(
        matrix=matrix,
        right_hand_side=right,
        active_point_ordinals=active_ordinals,
        modes=modes.copy(),
        analytic_gauge_matrix=gauges,
        gauge_identities=identities,
    )


def assemble_reduced_local_system(
    *,
    mass: NDArray[np.float64],
    bias: NDArray[np.float64],
    effort_newton_metres: NDArray[np.float64],
    contact_jacobians: NDArray[np.float64],
    contact_jdot_v: NDArray[np.float64],
    active_point_ordinals: Sequence[int],
    modes: NDArray[np.uint8],
) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
    active = tuple(int(value) for value in active_point_ordinals)
    if (
        mass.shape != (GENERALIZED_WIDTH, GENERALIZED_WIDTH)
        or bias.shape != (GENERALIZED_WIDTH,)
        or effort_newton_metres.shape != (ACTUATOR_COUNT,)
        or contact_jacobians.shape
        != (POINT_COUNT, POINT_FORCE_WIDTH, GENERALIZED_WIDTH)
        or contact_jdot_v.shape != (POINT_COUNT, POINT_FORCE_WIDTH)
        or modes.shape != (2,)
        or len(set(active)) != len(active)
        or tuple(sorted(active)) != active
        or any(not 0 <= point < POINT_COUNT for point in active)
        or any(
            not np.all(np.isfinite(value))
            for value in (
                mass,
                bias,
                effort_newton_metres,
                contact_jacobians,
                contact_jdot_v,
            )
        )
    ):
        raise ValueError("R125 reduced local-system input differs")
    expected = tuple(
        point for point in range(POINT_COUNT) if point_active(modes, point)
    )
    if active != expected:
        raise ValueError("R125 active point order differs from contact modes")

    unknown_count = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * len(active)
    matrix = np.zeros((unknown_count, unknown_count), dtype=np.float64)
    right = np.zeros(unknown_count, dtype=np.float64)
    matrix[:GENERALIZED_WIDTH, :GENERALIZED_WIDTH] = mass
    right[:GENERALIZED_WIDTH] = -bias
    right[6:GENERALIZED_WIDTH] += effort_newton_metres

    for local_ordinal, point in enumerate(active):
        columns = slice(
            GENERALIZED_WIDTH + POINT_FORCE_WIDTH * local_ordinal,
            GENERALIZED_WIDTH + POINT_FORCE_WIDTH * (local_ordinal + 1),
        )
        generalized_columns = np.column_stack(
            [
                generalized_contact_force(
                    contact_jacobians[point],
                    np.eye(POINT_FORCE_WIDTH, dtype=np.float64)[axis],
                )
                for axis in range(POINT_FORCE_WIDTH)
            ]
        )
        matrix[:GENERALIZED_WIDTH, columns] = -generalized_columns
        rows = slice(
            GENERALIZED_WIDTH + POINT_FORCE_WIDTH * local_ordinal,
            GENERALIZED_WIDTH + POINT_FORCE_WIDTH * (local_ordinal + 1),
        )
        matrix[rows, :GENERALIZED_WIDTH] = contact_jacobians[point]
        right[rows] = -contact_jdot_v[point]
    return matrix, right


def analytic_force_gauges(
    *,
    modes: NDArray[np.uint8],
    points: tuple[dict[str, Any], ...],
    active_point_ordinals: tuple[int, ...],
    body_rotations: NDArray[np.float64],
    body_positions: NDArray[np.float64],
) -> tuple[NDArray[np.float64], tuple[GaugeIdentity, ...]]:
    if modes.shape != (2,) or len(points) != POINT_COUNT:
        raise ValueError("R125 analytic gauge input differs")
    unknown_count = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * len(active_point_ordinals)
    active_index = {point: index for index, point in enumerate(active_point_ordinals)}
    columns: list[NDArray[np.float64]] = []
    identities: list[GaugeIdentity] = []
    for side in range(2):
        if int(modes[side]) != 3:
            continue
        heel = side * 2
        forefoot = heel + 1
        if heel not in active_index or forefoot not in active_index:
            raise ValueError("R125 flat-foot active points differ")
        heel_point = points[heel]
        forefoot_point = points[forefoot]
        body_slot = int(heel_point["body_slot"])
        if body_slot != int(forefoot_point["body_slot"]):
            raise ValueError("R125 flat-foot points do not share a rigid body")
        heel_local = np.asarray(
            heel_point["local_translation_metres"], dtype=np.float64
        )
        forefoot_local = np.asarray(
            forefoot_point["local_translation_metres"], dtype=np.float64
        )
        local_direction = forefoot_local - heel_local
        norm = float(np.linalg.norm(local_direction))
        if not np.isfinite(norm) or norm <= 0.0:
            raise ValueError("R125 flat-foot point separation differs")
        world_direction = body_rotations[body_slot] @ (local_direction / norm)
        direction_nrf = world_direction[[1, 0, 2]]
        gauge = np.zeros(unknown_count, dtype=np.float64)
        heel_offset = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * active_index[heel]
        forefoot_offset = GENERALIZED_WIDTH + POINT_FORCE_WIDTH * active_index[forefoot]
        gauge[heel_offset : heel_offset + POINT_FORCE_WIDTH] = direction_nrf
        gauge[forefoot_offset : forefoot_offset + POINT_FORCE_WIDTH] = -direction_nrf
        columns.append(gauge)

        heel_world = body_positions[body_slot] + body_rotations[body_slot] @ heel_local
        forefoot_world = (
            body_positions[body_slot] + body_rotations[body_slot] @ forefoot_local
        )
        resultant_force = world_direction - world_direction
        resultant_moment = np.cross(heel_world, world_direction) + np.cross(
            forefoot_world, -world_direction
        )
        identities.append(
            GaugeIdentity(
                side=side,
                heel_point_ordinal=heel,
                forefoot_point_ordinal=forefoot,
                world_direction=world_direction,
                maximum_resultant_force=float(np.max(np.abs(resultant_force))),
                maximum_resultant_moment=float(np.max(np.abs(resultant_moment))),
            )
        )
    gauge_matrix = (
        np.column_stack(columns)
        if columns
        else np.empty((unknown_count, 0), dtype=np.float64)
    )
    return gauge_matrix, tuple(identities)


def reduced_column_scale(
    *,
    model: SpatialModel,
    cache: Mapping[str, NDArray[Any]],
    active_point_count: int,
) -> NDArray[np.float64]:
    if not 0 <= active_point_count <= POINT_COUNT:
        raise ValueError("R125 active point count differs")
    acceleration = np.maximum(
        np.max(np.abs(np.asarray(cache["acceleration"], dtype=np.float64)), axis=0),
        1.0,
    )
    force = max(float(np.sum(model.masses)) * abs(float(model.gravity[1])), 1.0)
    return np.concatenate(
        (
            acceleration,
            np.full(
                active_point_count * POINT_FORCE_WIDTH,
                force,
                dtype=np.float64,
            ),
        )
    )


def scale_reduced_system(
    system: ReducedLocalSystem,
    column_scale: NDArray[np.float64],
) -> tuple[NDArray[np.float64], NDArray[np.float64], NDArray[np.float64]]:
    unknown_count = system.matrix.shape[1]
    if (
        system.matrix.shape != (unknown_count, unknown_count)
        or system.right_hand_side.shape != (unknown_count,)
        or column_scale.shape != (unknown_count,)
        or not np.all(np.isfinite(system.matrix))
        or not np.all(np.isfinite(system.right_hand_side))
        or not np.all(np.isfinite(column_scale))
        or np.any(column_scale <= 0.0)
    ):
        raise ValueError("R125 scaled reduced system differs")
    scaled_columns = system.matrix * column_scale[np.newaxis, :]
    row_scale = np.maximum(
        np.maximum(
            np.max(np.abs(scaled_columns), axis=1),
            np.abs(system.right_hand_side),
        ),
        1.0,
    )
    return (
        scaled_columns / row_scale[:, np.newaxis],
        system.right_hand_side / row_scale,
        row_scale,
    )


def analyse_reduced_system(
    system: ReducedLocalSystem,
    *,
    column_scale: NDArray[np.float64],
    numeric_contract: Mapping[str, Any],
    compute_particular: bool,
) -> RankAnalysis:
    scaled_matrix, scaled_right, _ = scale_reduced_system(system, column_scale)
    left, singular_values, right_transpose = np.linalg.svd(
        scaled_matrix, full_matrices=True
    )
    expected_nullity = system.analytic_gauge_matrix.shape[1]
    invalid: str | None = None
    if singular_values.size == 0 or not np.all(np.isfinite(singular_values)):
        return _invalid_rank_analysis(
            "NONFINITE_OR_EMPTY_SINGULAR_SPECTRUM", expected_nullity
        )
    largest = float(singular_values[0])
    if not np.isfinite(largest) or largest <= 0.0:
        return _invalid_rank_analysis("ZERO_SCALED_LOCAL_SYSTEM", expected_nullity)
    relative = singular_values / largest
    null_limit = float(numeric_contract["svd_null_relative_maximum"])
    retained_limit = float(numeric_contract["svd_retained_relative_minimum"])
    ambiguous = (relative > null_limit) & (relative < retained_limit)
    retained = relative >= retained_limit
    null = relative <= null_limit
    rank = int(np.count_nonzero(retained))
    nullity = int(np.count_nonzero(null))
    if np.any(ambiguous) or rank + nullity != len(relative):
        invalid = "SVD_RANK_AMBIGUITY_BAND_OCCUPIED"
    elif nullity != expected_nullity:
        invalid = "UNEXPECTED_REDUCED_SYSTEM_NULLITY"

    analytic_residual = 0.0
    projector_error = 0.0
    if expected_nullity:
        if system.analytic_gauge_matrix.shape != (
            system.matrix.shape[1],
            expected_nullity,
        ):
            invalid = invalid or "ANALYTIC_GAUGE_SHAPE_DIFFERS"
        scaled_gauge = system.analytic_gauge_matrix / column_scale[:, np.newaxis]
        analytic_basis, triangular = np.linalg.qr(scaled_gauge, mode="reduced")
        if not np.all(np.isfinite(analytic_basis)) or np.any(
            np.abs(np.diag(triangular)) <= 0.0
        ):
            invalid = invalid or "ANALYTIC_GAUGE_DEGENERATE"
        analytic_residual = float(np.max(np.abs(scaled_matrix @ analytic_basis)))
        if nullity == expected_nullity:
            svd_basis = right_transpose.T[:, -nullity:]
            projector_error = float(
                np.linalg.norm(
                    analytic_basis @ analytic_basis.T - svd_basis @ svd_basis.T,
                    ord=2,
                )
            )
        else:
            projector_error = math.inf
        if analytic_residual > float(
            numeric_contract["analytic_gauge_scaled_residual"]
        ):
            invalid = invalid or "ANALYTIC_GAUGE_RESIDUAL_EXCEEDED"
        if projector_error > float(
            numeric_contract["analytic_to_svd_null_projector_spectral_error"]
        ):
            invalid = invalid or "ANALYTIC_SVD_NULL_PROJECTOR_MISMATCH"

    particular: NDArray[np.float64] | None = None
    scaled_residual: float | None = None
    backward: float | None = None
    dynamics: float | None = None
    closure: float | None = None
    if compute_particular and invalid is None:
        retained_indices = np.flatnonzero(retained)
        scaled_solution = right_transpose.T[:, retained_indices] @ (
            (left[:, retained_indices].T @ scaled_right)
            / singular_values[retained_indices]
        )
        particular = column_scale * scaled_solution
        if not np.all(np.isfinite(particular)):
            invalid = "NONFINITE_PARTICULAR_SOLUTION"
        else:
            scaled_residual = float(
                np.max(np.abs(scaled_matrix @ scaled_solution - scaled_right))
            )
            residual = system.matrix @ particular - system.right_hand_side
            denominator = np.maximum(
                np.abs(system.matrix) @ np.abs(particular)
                + np.abs(system.right_hand_side),
                1.0,
            )
            backward = float(np.max(np.abs(residual) / denominator))
            dynamics = float(np.max(np.abs(residual[:GENERALIZED_WIDTH])))
            closure = (
                float(np.max(np.abs(residual[GENERALIZED_WIDTH:])))
                if residual.size > GENERALIZED_WIDTH
                else 0.0
            )
            if scaled_residual > float(numeric_contract["scaled_absolute_residual"]):
                invalid = "SCALED_EQUALITY_RESIDUAL_EXCEEDED"
            elif backward > float(numeric_contract["backward_error"]):
                invalid = "BACKWARD_ERROR_EXCEEDED"
            elif max(dynamics, closure) > float(
                numeric_contract["physical_group_absolute_residual"]
            ):
                invalid = "PHYSICAL_GROUP_RESIDUAL_EXCEEDED"

    return RankAnalysis(
        status="VALID" if invalid is None else "INVALID",
        invalid_reason=invalid,
        rank=rank,
        nullity=nullity,
        expected_nullity=expected_nullity,
        relative_singular_values=relative,
        smallest_retained_relative_singular_value=(
            float(np.min(relative[retained])) if rank else None
        ),
        largest_null_relative_singular_value=(
            float(np.max(relative[null])) if nullity else None
        ),
        analytic_gauge_scaled_residual=analytic_residual,
        analytic_to_svd_projector_spectral_error=projector_error,
        particular_solution=particular,
        scaled_absolute_residual=scaled_residual,
        backward_error=backward,
        dynamics_absolute_residual=dynamics,
        closure_absolute_residual=closure,
    )


def line_cone_interval(
    force_nrf: NDArray[np.float64],
    direction_nrf: NDArray[np.float64],
    *,
    boundary_relative_ambiguity: float,
) -> LineConeInterval:
    force = np.asarray(force_nrf, dtype=np.float64)
    direction = np.asarray(direction_nrf, dtype=np.float64)
    if (
        force.shape != (POINT_FORCE_WIDTH,)
        or direction.shape != (POINT_FORCE_WIDTH,)
        or not np.all(np.isfinite(force))
        or not np.all(np.isfinite(direction))
        or boundary_relative_ambiguity <= 0.0
    ):
        raise ValueError("R125 line-cone input differs")
    n0, right0, forward0 = (float(value) for value in force)
    dn, dright, dforward = (float(value) for value in direction)
    mu2 = FRICTION * FRICTION
    quadratic = dright * dright + dforward * dforward - mu2 * dn * dn
    linear = 2.0 * (right0 * dright + forward0 * dforward - mu2 * n0 * dn)
    constant = right0 * right0 + forward0 * forward0 - mu2 * n0 * n0
    coefficients = (quadratic, linear, constant)

    polynomial = _quadratic_nonpositive_intervals(
        quadratic,
        linear,
        constant,
        boundary_relative_ambiguity,
    )
    if polynomial is None:
        return LineConeInterval(
            "INVALID",
            "QUADRATIC_BOUNDARY_AMBIGUITY",
            False,
            None,
            None,
            coefficients,
        )
    normal = _normal_halfspace(n0, dn)
    feasible = _intersect_interval_sets(polynomial, normal)
    if not feasible:
        return LineConeInterval("VALID", None, True, None, None, coefficients)
    if len(feasible) != 1:
        return LineConeInterval(
            "INVALID",
            "NONCONVEX_LINE_CONE_CLASSIFICATION",
            False,
            None,
            None,
            coefficients,
        )
    lower, upper = feasible[0]
    return LineConeInterval(
        "VALID", None, False, float(lower), float(upper), coefficients
    )


def classify_flat_gauge_feasibility(
    particular_forces_nrf: NDArray[np.float64],
    gauge_direction_nrf: NDArray[np.float64],
    *,
    boundary_relative_ambiguity: float,
    cone_absolute_newtons: float,
) -> GaugeFeasibility:
    forces = np.asarray(particular_forces_nrf, dtype=np.float64)
    direction = np.asarray(gauge_direction_nrf, dtype=np.float64)
    if (
        forces.shape != (2, POINT_FORCE_WIDTH)
        or direction.shape != (POINT_FORCE_WIDTH,)
        or not np.all(np.isfinite(forces))
        or not np.all(np.isfinite(direction))
        or float(direction @ direction) <= 0.0
        or cone_absolute_newtons < 0.0
    ):
        raise ValueError("R125 flat-gauge feasibility input differs")
    heel = line_cone_interval(
        forces[0],
        direction,
        boundary_relative_ambiguity=boundary_relative_ambiguity,
    )
    forefoot = line_cone_interval(
        forces[1],
        -direction,
        boundary_relative_ambiguity=boundary_relative_ambiguity,
    )
    for interval in (heel, forefoot):
        if interval.status != "VALID":
            return GaugeFeasibility(
                "INVALID",
                interval.invalid_reason,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
    if heel.empty or forefoot.empty:
        return GaugeFeasibility(
            "VALID", None, False, None, None, None, None, None, None, None
        )
    assert heel.lower is not None and heel.upper is not None
    assert forefoot.lower is not None and forefoot.upper is not None
    lower = max(heel.lower, forefoot.lower)
    upper = min(heel.upper, forefoot.upper)
    endpoint_scale = max(
        1.0,
        abs(lower) if math.isfinite(lower) else 1.0,
        abs(upper) if math.isfinite(upper) else 1.0,
    )
    if lower > upper + boundary_relative_ambiguity * endpoint_scale:
        return GaugeFeasibility(
            "VALID", None, False, lower, upper, None, None, None, None, None
        )
    if lower > upper:
        return GaugeFeasibility(
            "INVALID",
            "GAUGE_INTERVAL_ENDPOINT_AMBIGUITY",
            None,
            lower,
            upper,
            None,
            None,
            None,
            None,
            None,
        )

    norm_squared = float(direction @ direction)
    unconstrained = -float(direction @ (forces[0] - forces[1])) / (2.0 * norm_squared)
    witness_alpha = min(max(unconstrained, lower), upper)
    witness = forces.copy()
    witness[0] += witness_alpha * direction
    witness[1] -= witness_alpha * direction
    normal_margins = witness[:, 0]
    friction_margins = FRICTION * witness[:, 0] - np.hypot(witness[:, 1], witness[:, 2])
    minimum_normal = float(np.min(normal_margins))
    minimum_friction = float(np.min(friction_margins))
    if (
        minimum_normal < -cone_absolute_newtons
        or minimum_friction < -cone_absolute_newtons
    ):
        return GaugeFeasibility(
            "INVALID",
            "GAUGE_WITNESS_DIRECT_CONE_RECHECK_FAILED",
            None,
            lower,
            upper,
            unconstrained,
            witness_alpha,
            witness,
            minimum_normal,
            minimum_friction,
        )
    return GaugeFeasibility(
        "VALID",
        None,
        True,
        lower,
        upper,
        unconstrained,
        witness_alpha,
        witness,
        minimum_normal,
        minimum_friction,
    )


def _quadratic_nonpositive_intervals(
    quadratic: float,
    linear: float,
    constant: float,
    ambiguity: float,
) -> list[tuple[float, float]] | None:
    coefficient_scale = max(abs(quadratic), abs(linear), abs(constant), 1.0)
    if abs(quadratic) <= ambiguity * coefficient_scale:
        if quadratic != 0.0:
            return None
        if abs(linear) <= ambiguity * coefficient_scale:
            if linear != 0.0 or abs(constant) <= ambiguity * coefficient_scale:
                return None
            return [(-math.inf, math.inf)] if constant < 0.0 else []
        root = -constant / linear
        return [(-math.inf, root)] if linear > 0.0 else [(root, math.inf)]

    discriminant = linear * linear - 4.0 * quadratic * constant
    discriminant_scale = max(linear * linear, abs(4.0 * quadratic * constant), 1.0)
    if abs(discriminant) <= ambiguity * discriminant_scale:
        return None
    if discriminant < 0.0:
        return [(-math.inf, math.inf)] if quadratic < 0.0 else []
    root = math.sqrt(discriminant)
    first = (-linear - root) / (2.0 * quadratic)
    second = (-linear + root) / (2.0 * quadratic)
    lower, upper = min(first, second), max(first, second)
    if quadratic > 0.0:
        return [(lower, upper)]
    return [(-math.inf, lower), (upper, math.inf)]


def _normal_halfspace(intercept: float, slope: float) -> list[tuple[float, float]]:
    if slope > 0.0:
        return [(-intercept / slope, math.inf)]
    if slope < 0.0:
        return [(-math.inf, -intercept / slope)]
    return [(-math.inf, math.inf)] if intercept >= 0.0 else []


def _intersect_interval_sets(
    left: Sequence[tuple[float, float]],
    right: Sequence[tuple[float, float]],
) -> list[tuple[float, float]]:
    result = []
    for left_lower, left_upper in left:
        for right_lower, right_upper in right:
            lower = max(left_lower, right_lower)
            upper = min(left_upper, right_upper)
            if lower <= upper:
                result.append((lower, upper))
    result.sort()
    merged: list[tuple[float, float]] = []
    for lower, upper in result:
        if merged and lower <= merged[-1][1]:
            merged[-1] = (merged[-1][0], max(merged[-1][1], upper))
        else:
            merged.append((lower, upper))
    return merged


def _invalid_rank_analysis(reason: str, expected_nullity: int) -> RankAnalysis:
    return RankAnalysis(
        status="INVALID",
        invalid_reason=reason,
        rank=0,
        nullity=0,
        expected_nullity=expected_nullity,
        relative_singular_values=np.empty(0, dtype=np.float64),
        smallest_retained_relative_singular_value=None,
        largest_null_relative_singular_value=None,
        analytic_gauge_scaled_residual=math.inf,
        analytic_to_svd_projector_spectral_error=math.inf,
        particular_solution=None,
        scaled_absolute_residual=None,
        backward_error=None,
        dynamics_absolute_residual=None,
        closure_absolute_residual=None,
    )
