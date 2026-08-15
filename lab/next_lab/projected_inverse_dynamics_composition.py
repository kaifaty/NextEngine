from __future__ import annotations

import hashlib
from collections.abc import Mapping, Sequence
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    COLLOCATION_COUNT,
    GENERALIZED_WIDTH,
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import (
    POINT_FORCE_WIDTH,
    assemble_reduced_local_system,
)

ARRAY_SHAPES = {
    "projected_generalized_velocity": (COLLOCATION_COUNT, GENERALIZED_WIDTH),
    "projection_delta_velocity": (COLLOCATION_COUNT, GENERALIZED_WIDTH),
    "applied_target_microradians": (COLLOCATION_COUNT, ACTUATOR_COUNT),
    "applied_effort_micronewton_metres": (COLLOCATION_COUNT, ACTUATOR_COUNT),
}


def applied_effort_newton_metres(
    applied_effort_micronewton_metres: NDArray[Any],
) -> NDArray[np.float64]:
    """Convert the exact R133 integer-valued float64 effort to SI units."""

    effort = np.asarray(applied_effort_micronewton_metres)
    if (
        effort.dtype != np.float64
        or effort.shape != (ACTUATOR_COUNT,)
        or not np.all(np.isfinite(effort))
        or not np.array_equal(effort, np.rint(effort))
    ):
        raise ValueError("projected inverse-dynamics effort differs")
    return effort / 1_000_000.0


def assemble_projected_reduced_local_system(
    *,
    mass: NDArray[np.float64],
    bias_from_projected_velocity: NDArray[np.float64],
    applied_effort_micronewton_metres: NDArray[np.float64],
    contact_jacobians: NDArray[np.float64],
    contact_jdot_v_from_projected_velocity: NDArray[np.float64],
    active_point_ordinals: Sequence[int],
    modes: NDArray[np.uint8],
) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
    """Compose the R134 system; warm acceleration is intentionally absent."""

    effort = applied_effort_newton_metres(applied_effort_micronewton_metres)
    return assemble_reduced_local_system(
        mass=np.asarray(mass, dtype=np.float64),
        bias=np.asarray(bias_from_projected_velocity, dtype=np.float64),
        effort_newton_metres=effort,
        contact_jacobians=np.asarray(contact_jacobians, dtype=np.float64),
        contact_jdot_v=np.asarray(
            contact_jdot_v_from_projected_velocity, dtype=np.float64
        ),
        active_point_ordinals=active_point_ordinals,
        modes=np.asarray(modes, dtype=np.uint8),
    )


def projected_reduced_column_scale(
    *,
    r120_acceleration_knots: NDArray[np.float64],
    total_body_mass_kilograms: float,
    gravity_y_metres_per_second_squared: float,
    active_point_count: int,
) -> NDArray[np.float64]:
    """Use R120 acceleration only as the inherited acceleration scale."""

    acceleration = np.asarray(r120_acceleration_knots)
    if (
        acceleration.dtype != np.float64
        or acceleration.ndim != 2
        or acceleration.shape[1] != GENERALIZED_WIDTH
        or acceleration.shape[0] < 1
        or not np.all(np.isfinite(acceleration))
        or not np.isfinite(total_body_mass_kilograms)
        or total_body_mass_kilograms <= 0.0
        or not np.isfinite(gravity_y_metres_per_second_squared)
        or type(active_point_count) is not int
        or not 0 <= active_point_count <= 4
    ):
        raise ValueError("projected inverse-dynamics column-scale input differs")
    acceleration_scale = np.maximum(np.max(np.abs(acceleration), axis=0), 1.0)
    force_scale = max(
        total_body_mass_kilograms * abs(gravity_y_metres_per_second_squared),
        1.0,
    )
    return np.concatenate(
        (
            acceleration_scale,
            np.full(
                active_point_count * POINT_FORCE_WIDTH,
                force_scale,
                dtype=np.float64,
            ),
        )
    )


def verify_r133_array_hashes(
    arrays: Mapping[str, NDArray[Any]], expected_sha256: Mapping[str, str]
) -> dict[str, str]:
    """Fail before ID unless all four reconstructed R133 arrays match."""

    if set(arrays) != set(ARRAY_SHAPES) or set(expected_sha256) != set(ARRAY_SHAPES):
        raise ValueError("R133 reconstructed array inventory differs")
    actual: dict[str, str] = {}
    for name, shape in ARRAY_SHAPES.items():
        value = np.asarray(arrays[name])
        digest = expected_sha256[name]
        if (
            value.dtype != np.float64
            or value.shape != shape
            or not np.all(np.isfinite(value))
            or not isinstance(digest, str)
            or len(digest) != 64
        ):
            raise ValueError(f"R133 reconstructed {name} differs")
        actual[name] = hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()
        if actual[name] != digest:
            raise ValueError(f"R133 reconstructed {name} hash differs")
    return actual
