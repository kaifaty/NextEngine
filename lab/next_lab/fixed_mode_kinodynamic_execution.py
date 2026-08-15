from __future__ import annotations

import hashlib
import io
import json
import math
import os
import resource
import time
from collections import Counter
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Final

import numpy as np
import osqp
import scipy
from numpy.typing import NDArray
from scipy import sparse

from next_lab.exit_mode_owned_lift_conformance import EXIT_INTERVALS
from next_lab.exit_mode_owned_projected_schedule_execution import (
    CONTROLLER_CATEGORIES,
    derive_eventful_projected_fixed_pd_schedule,
)
from next_lab.fixed_mode_controller_reachable_kinodynamic_formulation import (
    ACTUATED_JOINT_COUNT,
    GENERALIZED_WIDTH,
    MOTOR_INTERVAL_COUNT,
    PHYSICS_INTERVAL_COUNT,
    STATE_NODE_COUNT,
    SUBSTEPS_PER_INTERVAL,
    active_points_for_modes,
)
from next_lab.fixed_mode_kinodynamic_graph_conformance import (
    classify_point_transition,
    physics_address,
)
from next_lab.fixed_mode_kinodynamic_solve_conformance import decide_major_outcome
from next_lab.fixed_mode_kinodynamic_solve_formulation import canonical_json
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    Configuration,
    Kinematics,
    SpatialModel,
    build_spatial_model,
    forward_kinematics,
    generalized_contact_force,
    integrate_configuration,
    inverse_dynamics,
    mass_matrix,
    point_acceleration,
    point_jacobian,
    point_position,
    propagate_motion,
    rotation_exp,
)
from next_lab.motion_math import (
    collider_minimum_y,
    decompose_xzy,
    matrix_to_quaternion,
    q1_30,
    quaternion_to_matrix,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor
from next_lab.tangent_velocity_projection_conformance import (
    _flat_line_audit,
    _projection_passes,
    project_tangent_velocity,
)

EXECUTION_ID: Final = "nextengine.humanoid-fixed-mode-kinodynamic-execution.v1"
CHECK_ID: Final = "TRAIN-4-FIXED-MODE-KINODYNAMIC-EXECUTION"
FRAME_COUNT: Final = MOTOR_INTERVAL_COUNT + 1
POINT_COUNT: Final = 4
POINT_WIDTH: Final = 3
DT: Final = 1.0 / 240.0
FRICTION_NUMERATOR: Final = 52_429
FRICTION_DENOMINATOR: Final = 65_536
FRICTION: Final = FRICTION_NUMERATOR / FRICTION_DENOMINATOR
VALIDATION_COMMANDS_SHA256: Final = (
    "48699fe66b59437dbba733c8acd03d18e22b10a04978a907e238c12302e051fa"
)
CONTACT_IDS: Final = (
    "effector.left-heel",
    "effector.left-forefoot",
    "effector.right-heel",
    "effector.right-forefoot",
)
CACHE_KEYS: Final = (
    "root_position_m",
    "root_orientation_delta_rad",
    "joint_position_rad",
    "velocity",
    "acceleration",
    "reference_root_rotation",
    "metadata_json_utf8",
)
LOADED_CACHE_KEYS: Final = (
    "root_position_m",
    "root_orientation_delta_rad",
    "joint_position_rad",
    "velocity",
    "reference_root_rotation",
    "metadata_json_utf8",
)
CONTROLLER_STAGE_NAMES: Final = (
    "target_soft_limit",
    "target_slew_limit",
    "observed_hard_rom",
    "observed_joint_velocity",
    "static_effort_limit",
    "effort_rate_limit",
    "power_limit",
    "positive_work_limit",
    "pre_clip_envelope",
    "post_charge_work",
)
CONTROLLER_CATEGORY_CODE: Final = {
    value: ordinal for ordinal, value in enumerate(CONTROLLER_CATEGORIES)
}
CONTROLLER_STAGE_CODE: Final = {
    value: ordinal for ordinal, value in enumerate(CONTROLLER_STAGE_NAMES)
}
GRAPH_CATEGORY_CODE: Final = {
    name: ordinal
    for ordinal, name in enumerate(
        (
            "initial_anchor_position",
            "initial_sticking_velocity",
            "continuous_dynamics",
            "pre_event_velocity_definition",
            "manifold_integration",
            "ordinary_velocity_update",
            "activation_momentum_jump",
            "active_anchor_position",
            "active_sticking_velocity",
            "active_sticking_acceleration",
            "force_normal",
            "force_circular_cone",
            "impulse_normal",
            "impulse_circular_cone",
            "joint_rom",
            "joint_velocity",
            "collider_floor",
            "controller_hard_rom",
            "controller_velocity",
            "controller_envelope",
        )
    )
}
ZERO_DOWNSTREAM_COUNTERS: Final = (
    "candidate_artifacts_built",
    "physx_scene_runs",
    "training_runs",
)
REQUIRED_COUNTERS: Final = (
    "real_state_reconstructions",
    "real_state_lift_evaluations",
    "real_controller_schedule_derivations",
    "real_controller_graph_evaluations",
    "real_mass_matrix_assemblies",
    "real_contact_jacobian_assemblies",
    "real_dynamics_residual_evaluations",
    "real_integration_residual_evaluations",
    "real_impulse_residual_evaluations",
    "real_kinodynamic_system_assemblies",
    "real_qp_setups",
    "real_qp_solves",
    "real_kinodynamic_solves",
    "real_factorizations",
    "real_exact_trial_audits",
    "optimizer_steps",
    "candidate_artifacts_built",
    "solver_private_caches_built",
    "physx_scene_runs",
    "training_runs",
)

FloatArray = NDArray[np.float64]
IntArray = NDArray[np.int64]
BoolArray = NDArray[np.bool_]


class ExecutionInvalid(RuntimeError):
    """An R139 fail-closed numeric or identity condition."""


def _stable_invalid_reason(error: Exception, *, prefix: str) -> str:
    if isinstance(error, ExecutionInvalid):
        return str(error)
    return f"{prefix}_{type(error).__name__}: {error}"


@dataclass(frozen=True)
class TrajectoryState:
    root_position: FloatArray
    root_rotation: FloatArray
    joint_position: FloatArray
    velocity: FloatArray
    acceleration: FloatArray
    integer_command: IntArray
    active_force: FloatArray
    activation_impulse: FloatArray


@dataclass(frozen=True)
class ControllerReplay:
    applied_target: FloatArray
    applied_effort: FloatArray
    event_addresses: IntArray
    branch_addresses: NDArray[np.uint8]
    target_branch_addresses: NDArray[np.uint8]
    work_before: FloatArray
    work_after: FloatArray
    observed_position_microradians: FloatArray
    observed_velocity_microradians_per_second: FloatArray
    branch_equality_ambiguity_count: int
    audit: dict[str, Any]


@dataclass(frozen=True)
class GraphInventory:
    interval_modes: NDArray[np.uint8]
    node_modes: NDArray[np.uint8]
    active_points_by_interval: tuple[tuple[int, ...], ...]
    active_points_by_node: tuple[tuple[int, ...], ...]
    force_rows: tuple[tuple[int, int], ...]
    force_row_by_interval_point: dict[tuple[int, int], int]
    impulse_rows: tuple[tuple[int, int], ...]
    impulse_row_by_node_point: dict[tuple[int, int], int]
    anchor_rows: tuple[tuple[int, int], ...]
    anchor_index_by_node_point: IntArray
    graph_index_sha256: str
    transition_replay_sha256: str
    motor_mode_sequence_sha256: str


@dataclass(frozen=True)
class ReferenceReconstruction:
    state: TrajectoryState
    projected_velocity: FloatArray
    projection_delta: FloatArray
    source_controller: ControllerReplay
    anchors: FloatArray
    source_hashes: dict[str, str]
    configuration_hashes: dict[str, str]
    projection_systems: int
    projection_factorizations: int
    projection_solves: int


@dataclass(frozen=True)
class ExactAudit:
    status: str
    invalid_reason: str | None
    controller: ControllerReplay | None
    residual_vector: FloatArray | None
    violation_vector: FloatArray | None
    graph_row_addresses: IntArray | None
    funnel: tuple[float, float] | None
    hard_pass: bool
    category_aggregate: dict[str, Any]
    violated_cones: tuple[dict[str, Any], ...]
    counters: dict[str, int]


@dataclass(frozen=True)
class VariableLayout:
    q: slice
    velocity: slice
    acceleration: slice
    command: slice
    force: slice
    impulse: slice
    applied_target: slice
    applied_effort: slice
    work: slice
    primary_count: int
    base_count: int
    scale: FloatArray
    lower: FloatArray
    upper: FloatArray
    effort_scale: FloatArray
    work_scale: FloatArray

    def q_index(self, node: int, component: int) -> int:
        return self.q.start + node * GENERALIZED_WIDTH + component

    def velocity_index(self, node: int, component: int) -> int:
        return self.velocity.start + node * GENERALIZED_WIDTH + component

    def acceleration_index(self, interval: int, component: int) -> int:
        return self.acceleration.start + interval * GENERALIZED_WIDTH + component

    def command_index(self, interval: int, dof: int) -> int:
        return self.command.start + interval * ACTUATED_JOINT_COUNT + dof

    def force_index(self, row: int, component: int) -> int:
        return self.force.start + row * POINT_WIDTH + component

    def impulse_index(self, row: int, component: int) -> int:
        return self.impulse.start + row * POINT_WIDTH + component

    def target_index(self, interval: int, dof: int) -> int:
        return self.applied_target.start + interval * ACTUATED_JOINT_COUNT + dof

    def effort_index(self, interval: int, dof: int) -> int:
        return self.applied_effort.start + interval * ACTUATED_JOINT_COUNT + dof

    def work_index(self, interval: int, dof: int) -> int:
        return self.work.start + interval * ACTUATED_JOINT_COUNT + dof


@dataclass(frozen=True)
class LocalModel:
    layout: VariableLayout
    physical_matrix: sparse.csc_matrix
    physical_residual: FloatArray
    physical_funnel_multiplier: FloatArray
    physical_upper: BoolArray
    physical_addresses: IntArray
    hard_matrix: sparse.csc_matrix
    hard_lower: FloatArray
    hard_upper: FloatArray
    coefficient_sha256: str
    controller_branch_sha256: str
    controller_branch_ambiguity_count: int
    counters: dict[str, int]
    radius: float


@dataclass(frozen=True)
class QpPassResult:
    ordinal: int
    status: str
    iterations: int
    primal_residual: float
    dual_residual: float
    objective: float
    maximum_elastic: float
    mean_elastic: float
    solution: FloatArray
    variable_count: int
    constraint_count: int
    nonzero_count: int
    problem_sha256: str


@dataclass(frozen=True)
class MajorStep:
    step: FloatArray
    passes: tuple[QpPassResult, ...]
    model_maximum: float
    model_mean: float
    active_trust_boundary: bool


@dataclass(frozen=True)
class ExecutionOutcome:
    status: str
    feasibility: str
    termination: str
    invalid_reason: str | None
    final_state: TrajectoryState | None
    final_audit: ExactAudit | None
    anchors: FloatArray | None
    history: tuple[dict[str, Any], ...]
    accepted_anchor_hashes: tuple[dict[str, Any], ...]
    source_gate: dict[str, Any]
    counters: dict[str, int]
    resource_usage: dict[str, Any]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _array_sha256(value: NDArray[Any]) -> str:
    return hashlib.sha256(np.ascontiguousarray(value).tobytes(order="C")).hexdigest()


def _typed_array_identity(value: NDArray[Any]) -> dict[str, Any]:
    array = np.ascontiguousarray(value)
    return {
        "sha256": hashlib.sha256(array.tobytes(order="C")).hexdigest(),
        "shape": list(array.shape),
        "dtype": str(array.dtype),
    }


def _aggregate_array_identity(values: Mapping[str, NDArray[Any]]) -> str:
    rows = {name: _typed_array_identity(values[name]) for name in sorted(values)}
    return hashlib.sha256(canonical_json(rows)).hexdigest()


def _configuration_at(cache: Mapping[str, NDArray[Any]], frame: int) -> Configuration:
    return Configuration(
        root_position=np.asarray(cache["root_position_m"][frame], dtype=np.float64),
        root_rotation=(
            rotation_exp(
                np.asarray(cache["root_orientation_delta_rad"][frame], dtype=np.float64)
            )
            @ np.asarray(cache["reference_root_rotation"][frame], dtype=np.float64)
        ),
        joint_positions=np.asarray(
            cache["joint_position_rad"][frame], dtype=np.float64
        ),
    )


def _rotation_log(rotation: FloatArray) -> FloatArray:
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
        raise ExecutionInvalid("SO3_PRINCIPAL_LOG_AMBIGUOUS")
    return angle / (2.0 * math.sin(angle)) * vee


def _interpolate_configuration(
    cache: Mapping[str, NDArray[Any]], interval: int, substep: int
) -> Configuration:
    fraction = substep / SUBSTEPS_PER_INTERVAL
    left = _configuration_at(cache, interval)
    right = _configuration_at(cache, interval + 1)
    tangent = _rotation_log(right.root_rotation @ left.root_rotation.T)
    return Configuration(
        root_position=(1.0 - fraction) * left.root_position
        + fraction * right.root_position,
        root_rotation=rotation_exp(fraction * tangent) @ left.root_rotation,
        joint_positions=(1.0 - fraction) * left.joint_positions
        + fraction * right.joint_positions,
    )


def _discrete_velocity(left: Configuration, right: Configuration) -> FloatArray:
    return np.concatenate(
        (
            (right.root_position - left.root_position) / DT,
            _rotation_log(right.root_rotation @ left.root_rotation.T) / DT,
            (right.joint_positions - left.joint_positions) / DT,
        )
    )


def _contact_points(descriptor: Mapping[str, Any]) -> tuple[dict[str, Any], ...]:
    bodies = {
        str(row["body_id"]): int(row["body_slot"])
        for row in descriptor.get("bodies", ())
    }
    effectors = {
        str(row["effector_id"]): row for row in descriptor.get("effectors", ())
    }
    points: list[dict[str, Any]] = []
    for ordinal, effector_id in enumerate(CONTACT_IDS):
        effector = effectors.get(effector_id)
        if effector is None or str(effector["body_id"]) not in bodies:
            raise ExecutionInvalid("CONTACT_POINT_DESCRIPTOR_IDENTITY_DIFFERS")
        points.append(
            {
                "point_ordinal": ordinal,
                "effector_id": effector_id,
                "body_id": str(effector["body_id"]),
                "body_slot": bodies[str(effector["body_id"])],
                "local_translation_metres": (
                    np.asarray(
                        effector["local_translation_micrometres"], dtype=np.float64
                    )
                    / 1_000_000.0
                ),
            }
        )
    return tuple(points)


def _point_jacobian(
    model: SpatialModel,
    kinematics: Kinematics,
    point: Mapping[str, Any],
) -> FloatArray:
    return point_jacobian(
        model,
        kinematics,
        int(point["body_slot"]),
        np.asarray(point["local_translation_metres"], dtype=np.float64),
    )


def _point_position(kinematics: Kinematics, point: Mapping[str, Any]) -> FloatArray:
    return point_position(
        kinematics,
        int(point["body_slot"]),
        np.asarray(point["local_translation_metres"], dtype=np.float64),
    )


def build_graph_inventory(
    contact_modes: NDArray[np.uint8], *, require_production_inventory: bool = True
) -> GraphInventory:
    if contact_modes.shape != (FRAME_COUNT, 2) or contact_modes.dtype != np.uint8:
        raise ExecutionInvalid("V9_CONTACT_MODE_SHAPE_OR_DTYPE_DIFFERS")
    interval_modes = np.array(contact_modes[:MOTOR_INTERVAL_COUNT], copy=True)
    if np.any(~np.isin(interval_modes, (0, 2, 3))):
        raise ExecutionInvalid("FIXED_CONTACT_MODE_VALUE_DIFFERS")
    motor_active = tuple(
        tuple(active_points_for_modes([int(value) for value in row]))
        for row in interval_modes
    )
    active_by_interval = tuple(
        motor_active[interval // SUBSTEPS_PER_INTERVAL]
        for interval in range(PHYSICS_INTERVAL_COUNT)
    )
    node_modes = np.empty((STATE_NODE_COUNT, 2), dtype=np.uint8)
    for node in range(STATE_NODE_COUNT):
        motor = min(node // SUBSTEPS_PER_INTERVAL, MOTOR_INTERVAL_COUNT - 1)
        node_modes[node] = interval_modes[motor]
    active_by_node = tuple(
        tuple(active_points_for_modes([int(value) for value in row]))
        for row in node_modes
    )

    force_rows = tuple(
        (interval, point)
        for interval, active in enumerate(active_by_interval)
        for point in active
    )
    force_lookup = {address: row for row, address in enumerate(force_rows)}
    impulse_rows: list[tuple[int, int]] = []
    transition_replay: list[dict[str, Any]] = []
    for boundary in range(MOTOR_INTERVAL_COUNT - 1):
        left_modes = [int(value) for value in interval_modes[boundary]]
        right_modes = [int(value) for value in interval_modes[boundary + 1]]
        transition = classify_point_transition(left_modes, right_modes)
        if transition["event"] == "ORDINARY_CONTINUOUS_BOUNDARY":
            continue
        node = (boundary + 1) * SUBSTEPS_PER_INTERVAL
        transition_replay.append(
            {
                "motor_boundary": boundary,
                "physics_node": node,
                **transition,
            }
        )
        impulse_rows.extend(
            (node, int(point)) for point in transition["activated_point_ordinals"]
        )
    impulse_tuple = tuple(impulse_rows)
    impulse_lookup = {address: row for row, address in enumerate(impulse_tuple)}

    anchor_rows: list[tuple[int, int]] = []
    anchor_index = np.full((STATE_NODE_COUNT, POINT_COUNT), -1, dtype=np.int64)
    active_anchor: dict[int, int] = {}
    for node in range(STATE_NODE_COUNT):
        current = set(active_by_node[node])
        for point in sorted(set(active_anchor) - current):
            del active_anchor[point]
        for point in sorted(current):
            if point not in active_anchor:
                active_anchor[point] = len(anchor_rows)
                anchor_rows.append((node, point))
            anchor_index[node, point] = active_anchor[point]

    graph_rows = [
        physics_address(interval, substep)
        for interval in range(MOTOR_INTERVAL_COUNT)
        for substep in range(SUBSTEPS_PER_INTERVAL)
    ]
    mode_hash = hashlib.sha256(
        canonical_json([[int(value) for value in row] for row in interval_modes])
    ).hexdigest()
    inventory = GraphInventory(
        interval_modes=interval_modes,
        node_modes=node_modes,
        active_points_by_interval=active_by_interval,
        active_points_by_node=active_by_node,
        force_rows=force_rows,
        force_row_by_interval_point=force_lookup,
        impulse_rows=impulse_tuple,
        impulse_row_by_node_point=impulse_lookup,
        anchor_rows=tuple(anchor_rows),
        anchor_index_by_node_point=anchor_index,
        graph_index_sha256=hashlib.sha256(canonical_json(graph_rows)).hexdigest(),
        transition_replay_sha256=hashlib.sha256(
            canonical_json(transition_replay)
        ).hexdigest(),
        motor_mode_sequence_sha256=mode_hash,
    )
    if require_production_inventory and (
        len(inventory.force_rows) != 4_956
        or len(inventory.impulse_rows) != 18
        or len(inventory.anchor_rows) != 20
        or len(transition_replay) != 29
    ):
        raise ExecutionInvalid("FIXED_GRAPH_INVENTORY_DIFFERS")
    return inventory


def _record_event_addresses(
    rows: list[tuple[int, int, int, int]],
    mask: NDArray[np.bool_],
    *,
    interval: int,
    substep: int,
    category: str,
    stage: str,
) -> None:
    if mask.shape != (ACTUATED_JOINT_COUNT,) or mask.dtype != np.bool_:
        raise ExecutionInvalid("CONTROLLER_EVENT_MASK_DIFFERS")
    collocation = interval * SUBSTEPS_PER_INTERVAL + substep
    rows.extend(
        (
            collocation,
            int(dof),
            CONTROLLER_CATEGORY_CODE[category],
            CONTROLLER_STAGE_CODE[stage],
        )
        for dof in np.flatnonzero(mask)
    )


def _column(rows: Sequence[Mapping[str, Any]], key: str, column: int) -> FloatArray:
    return np.asarray([row[key][column] for row in rows], dtype=np.float64)


def _branch_code(value: float, lower: float, upper: float) -> tuple[int, bool]:
    if value < lower:
        return 0, False
    if value == lower:
        return 1, True
    if value < upper:
        return 2, False
    if value == upper:
        return 3, True
    return 4, False


def _selected_extreme(
    values: Sequence[tuple[int, float]], *, maximum: bool
) -> tuple[int, float, bool]:
    finite = [(code, value) for code, value in values if math.isfinite(value)]
    if not finite:
        raise ExecutionInvalid("CONTROLLER_LIMITER_BOUND_ABSENT")
    selected_value = (
        max(value for _, value in finite)
        if maximum
        else min(value for _, value in finite)
    )
    selected = [code for code, value in finite if value == selected_value]
    return (
        (selected[0] if len(selected) == 1 else 255),
        selected_value,
        len(selected) > 1,
    )


def replay_exact_controller(
    *,
    integer_command: IntArray,
    joint_position: FloatArray,
    velocity: FloatArray,
    descriptor: Mapping[str, Any],
) -> ControllerReplay:
    """Replay the R133 order from trial q/v and signed integer commands."""

    if (
        integer_command.shape != (MOTOR_INTERVAL_COUNT, ACTUATED_JOINT_COUNT)
        or integer_command.dtype != np.int64
        or joint_position.shape != (STATE_NODE_COUNT, ACTUATED_JOINT_COUNT)
        or joint_position.dtype != np.float64
        or velocity.shape != (STATE_NODE_COUNT, GENERALIZED_WIDTH)
        or velocity.dtype != np.float64
        or not np.all(np.isfinite(joint_position))
        or not np.all(np.isfinite(velocity))
    ):
        raise ExecutionInvalid("EXACT_CONTROLLER_INPUT_SHAPE_DTYPE_DIFFERS")
    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    expected_order = list(range(ACTUATED_JOINT_COUNT))
    if [int(row["dof_ordinal"]) for row in joints] != expected_order or [
        int(row["dof_ordinal"]) for row in actuators
    ] != expected_order:
        raise ExecutionInvalid("CONTROLLER_DESCRIPTOR_ORDER_DIFFERS")

    hard_min = _column(joints, "hard_limit_microradians", 0)
    hard_max = _column(joints, "hard_limit_microradians", 1)
    soft_min = _column(joints, "soft_limit_microradians", 0)
    soft_max = _column(joints, "soft_limit_microradians", 1)
    maximum_velocity = np.asarray(
        [row["maximum_velocity_microradians_per_second"] for row in joints],
        dtype=np.float64,
    )
    target_delta = np.asarray(
        [
            max(
                abs(int(value))
                for value in row["target_delta_microradians_per_motor_tick"]
            )
            for row in actuators
        ],
        dtype=np.float64,
    )
    stiffness = np.asarray(
        [row["stiffness_q16"] for row in actuators], dtype=np.float64
    )
    damping = np.asarray([row["damping_q16"] for row in actuators], dtype=np.float64)
    effort_min = _column(actuators, "effort_micronewton_metres", 0)
    effort_max = _column(actuators, "effort_micronewton_metres", 1)
    effort_rate = np.asarray(
        [row["maximum_effort_rate_micronewton_metres_per_second"] for row in actuators],
        dtype=np.float64,
    )
    maximum_power = np.asarray(
        [row["maximum_power_microwatts"] for row in actuators], dtype=np.float64
    )
    maximum_work = np.asarray(
        [row["maximum_positive_work_microjoules_per_motor_tick"] for row in actuators],
        dtype=np.float64,
    )
    maximum_effort_delta = np.rint(effort_rate / 240.0)

    target_output = np.empty(
        (MOTOR_INTERVAL_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64
    )
    effort_output = np.empty(
        (PHYSICS_INTERVAL_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64
    )
    work_before = np.empty_like(effort_output)
    work_after = np.empty_like(effort_output)
    observed_position = np.rint(joint_position[:-1] * 1_000_000.0)
    observed_velocity = np.rint(velocity[:-1, 6:] * 1_000_000.0)
    target_branches = np.empty(
        (MOTOR_INTERVAL_COUNT, ACTUATED_JOINT_COUNT, 2), dtype=np.uint8
    )
    effort_branches = np.empty(
        (PHYSICS_INTERVAL_COUNT, ACTUATED_JOINT_COUNT, 8), dtype=np.uint8
    )
    counts = {name: 0 for name in CONTROLLER_CATEGORIES}
    channels: dict[str, set[int]] = {name: set() for name in CONTROLLER_CATEGORIES}
    events: list[tuple[int, int, int, int]] = []
    maxima = {
        "hard_rom_excess_microradians": 0.0,
        "velocity_excess_microradians_per_second": 0.0,
        "target_lag_microradians": 0.0,
        "absolute_requested_effort_micronewton_metres": 0.0,
        "absolute_applied_effort_micronewton_metres": 0.0,
        "absolute_power_microwatts": 0.0,
        "positive_work_microjoules_per_motor_tick": 0.0,
    }
    applied_target = integer_command[0].astype(np.float64, copy=True)
    previous_effort = np.zeros(ACTUATED_JOINT_COUNT, dtype=np.float64)
    ambiguities = 0

    for interval in range(MOTOR_INTERVAL_COUNT):
        raw = integer_command[interval].astype(np.float64)
        soft = np.minimum(np.maximum(raw, soft_min), soft_max)
        soft_mask = soft != raw
        counts["target_soft_clamp"] += int(np.count_nonzero(soft_mask))
        channels["target_soft_clamp"].update(
            int(value) for value in np.flatnonzero(soft_mask)
        )
        _record_event_addresses(
            events,
            soft_mask,
            interval=interval,
            substep=0,
            category="target_soft_clamp",
            stage="target_soft_limit",
        )
        for dof in range(ACTUATED_JOINT_COUNT):
            code, equality = _branch_code(
                float(raw[dof]), float(soft_min[dof]), float(soft_max[dof])
            )
            target_branches[interval, dof, 0] = code
            ambiguities += int(equality)

        next_target = np.minimum(
            np.maximum(soft, applied_target - target_delta),
            applied_target + target_delta,
        )
        slew_mask = next_target != soft
        counts["target_slew"] += int(np.count_nonzero(slew_mask))
        channels["target_slew"].update(
            int(value) for value in np.flatnonzero(slew_mask)
        )
        _record_event_addresses(
            events,
            slew_mask,
            interval=interval,
            substep=0,
            category="target_slew",
            stage="target_slew_limit",
        )
        for dof in range(ACTUATED_JOINT_COUNT):
            code, equality = _branch_code(
                float(soft[dof]),
                float(applied_target[dof] - target_delta[dof]),
                float(applied_target[dof] + target_delta[dof]),
            )
            target_branches[interval, dof, 1] = code
            ambiguities += int(equality)
        applied_target = next_target
        target_output[interval] = applied_target
        maxima["target_lag_microradians"] = max(
            maxima["target_lag_microradians"],
            float(np.max(np.abs(applied_target - raw))),
        )

        used_work = np.zeros(ACTUATED_JOINT_COUNT, dtype=np.float64)
        for substep in range(SUBSTEPS_PER_INTERVAL):
            collocation = interval * SUBSTEPS_PER_INTERVAL + substep
            q_observed = observed_position[collocation]
            v_observed = observed_velocity[collocation]
            hard_excess = np.maximum(hard_min - q_observed, q_observed - hard_max)
            velocity_excess = np.abs(v_observed) - maximum_velocity
            hard_mask = hard_excess > 10.0
            velocity_mask = velocity_excess > 0.0
            for category, stage, mask in (
                ("hard_rom_violation", "observed_hard_rom", hard_mask),
                ("velocity_violation", "observed_joint_velocity", velocity_mask),
            ):
                counts[category] += int(np.count_nonzero(mask))
                channels[category].update(int(value) for value in np.flatnonzero(mask))
                _record_event_addresses(
                    events,
                    mask,
                    interval=interval,
                    substep=substep,
                    category=category,
                    stage=stage,
                )
            maxima["hard_rom_excess_microradians"] = max(
                maxima["hard_rom_excess_microradians"],
                float(max(0.0, np.max(hard_excess))),
            )
            maxima["velocity_excess_microradians_per_second"] = max(
                maxima["velocity_excess_microradians_per_second"],
                float(max(0.0, np.max(velocity_excess))),
            )
            requested = np.rint(
                stiffness * (applied_target - q_observed) / 65_536.0
            ) - np.rint(damping * v_observed / 65_536.0)
            maxima["absolute_requested_effort_micronewton_metres"] = max(
                maxima["absolute_requested_effort_micronewton_metres"],
                float(np.max(np.abs(requested))),
            )
            rate_min = previous_effort - maximum_effort_delta
            rate_max = previous_effort + maximum_effort_delta
            absolute_velocity = np.abs(v_observed)
            moving = absolute_velocity > 0.0
            power_bound = np.full(ACTUATED_JOINT_COUNT, np.inf, dtype=np.float64)
            power_bound[moving] = np.floor(
                maximum_power[moving] * 1_000_000.0 / absolute_velocity[moving]
            )
            work_before[collocation] = used_work
            remaining_work = maximum_work - used_work
            work_bound = np.full(ACTUATED_JOINT_COUNT, np.inf, dtype=np.float64)
            work_bound[moving] = np.floor(
                remaining_work[moving] * 240.0 * 1_000_000.0 / absolute_velocity[moving]
            )
            lower = np.maximum(np.maximum(effort_min, rate_min), -power_bound)
            upper = np.minimum(np.minimum(effort_max, rate_max), power_bound)
            lower = np.where(v_observed < 0.0, np.maximum(lower, -work_bound), lower)
            upper = np.where(v_observed > 0.0, np.minimum(upper, work_bound), upper)
            if np.any(lower > upper):
                raise ExecutionInvalid("CONTROLLER_UNORDERED_LOWER_UPPER_BOUND")

            masks = (
                (
                    "static_effort_clamp",
                    "static_effort_limit",
                    (requested < effort_min) | (requested > effort_max),
                ),
                (
                    "effort_rate_clamp",
                    "effort_rate_limit",
                    (requested < rate_min) | (requested > rate_max),
                ),
                (
                    "power_clamp",
                    "power_limit",
                    (requested < -power_bound) | (requested > power_bound),
                ),
                (
                    "positive_work_clamp",
                    "positive_work_limit",
                    ((v_observed > 0.0) & (requested > work_bound))
                    | ((v_observed < 0.0) & (requested < -work_bound)),
                ),
            )
            for category, stage, mask in masks:
                counts[category] += int(np.count_nonzero(mask))
                channels[category].update(int(value) for value in np.flatnonzero(mask))
                _record_event_addresses(
                    events,
                    mask,
                    interval=interval,
                    substep=substep,
                    category=category,
                    stage=stage,
                )
            infeasible = (remaining_work < 0.0) | (lower > upper)
            counts["infeasible_effort_envelope"] += int(np.count_nonzero(infeasible))
            channels["infeasible_effort_envelope"].update(
                int(value) for value in np.flatnonzero(infeasible)
            )
            _record_event_addresses(
                events,
                infeasible,
                interval=interval,
                substep=substep,
                category="infeasible_effort_envelope",
                stage="pre_clip_envelope",
            )

            current_effort = np.minimum(np.maximum(requested, lower), upper)
            charge = np.ceil(
                np.maximum(current_effort * v_observed, 0.0) / (240.0 * 1_000_000.0)
            )
            used_work = used_work + charge
            work_after[collocation] = used_work
            post_infeasible = used_work > maximum_work
            counts["infeasible_effort_envelope"] += int(
                np.count_nonzero(post_infeasible)
            )
            channels["infeasible_effort_envelope"].update(
                int(value) for value in np.flatnonzero(post_infeasible)
            )
            _record_event_addresses(
                events,
                post_infeasible,
                interval=interval,
                substep=substep,
                category="infeasible_effort_envelope",
                stage="post_charge_work",
            )

            for dof in range(ACTUATED_JOINT_COUNT):
                lower_values = [
                    (0, float(effort_min[dof])),
                    (1, float(rate_min[dof])),
                    (2, float(-power_bound[dof])),
                ]
                upper_values = [
                    (0, float(effort_max[dof])),
                    (1, float(rate_max[dof])),
                    (2, float(power_bound[dof])),
                ]
                if v_observed[dof] < 0.0:
                    lower_values.append((3, float(-work_bound[dof])))
                elif v_observed[dof] > 0.0:
                    upper_values.append((3, float(work_bound[dof])))
                lower_source, _, lower_equal = _selected_extreme(
                    lower_values, maximum=True
                )
                upper_source, _, upper_equal = _selected_extreme(
                    upper_values, maximum=False
                )
                final_code, final_equal = _branch_code(
                    float(requested[dof]), float(lower[dof]), float(upper[dof])
                )
                charge_code = (
                    1
                    if current_effort[dof] * v_observed[dof] > 0.0
                    else 2
                    if current_effort[dof] * v_observed[dof] == 0.0
                    else 0
                )
                effort_branches[collocation, dof] = (
                    lower_source,
                    upper_source,
                    final_code,
                    charge_code,
                    int(lower_equal),
                    int(upper_equal),
                    int(final_equal),
                    int(v_observed[dof] == 0.0),
                )
                ambiguities += int(lower_equal) + int(upper_equal) + int(final_equal)
                ambiguities += int(v_observed[dof] == 0.0)
                ambiguities += int(charge_code == 2)

            previous_effort = current_effort
            effort_output[collocation] = current_effort
            maxima["absolute_applied_effort_micronewton_metres"] = max(
                maxima["absolute_applied_effort_micronewton_metres"],
                float(np.max(np.abs(current_effort))),
            )
            maxima["absolute_power_microwatts"] = max(
                maxima["absolute_power_microwatts"],
                float(np.max(np.abs(current_effort * v_observed))) / 1_000_000.0,
            )
            maxima["positive_work_microjoules_per_motor_tick"] = max(
                maxima["positive_work_microjoules_per_motor_tick"],
                float(np.max(used_work)),
            )

    event_array = (
        np.asarray(events, dtype=np.int64).reshape((-1, 4))
        if events
        else np.empty((0, 4), dtype=np.int64)
    )
    audit = {
        "status": (
            "PASS"
            if counts["hard_rom_violation"] == 0
            and counts["velocity_violation"] == 0
            and counts["infeasible_effort_envelope"] == 0
            else "FAIL"
        ),
        "collocation_count": PHYSICS_INTERVAL_COUNT,
        "activation_counts": counts,
        "activated_dof_ordinals": {
            name: sorted(values) for name, values in channels.items()
        },
        "maxima": maxima,
        "applied_target_float64_sha256": _array_sha256(target_output),
        "applied_effort_float64_sha256": _array_sha256(effort_output),
        "controller_event_address_sha256": _array_sha256(event_array),
        "controller_branch_address_sha256": _array_sha256(effort_branches),
        "target_branch_address_sha256": _array_sha256(target_branches),
        "branch_equality_ambiguity_count": ambiguities,
    }
    return ControllerReplay(
        applied_target=target_output,
        applied_effort=effort_output,
        event_addresses=event_array,
        branch_addresses=effort_branches,
        target_branch_addresses=target_branches,
        work_before=work_before,
        work_after=work_after,
        observed_position_microradians=observed_position,
        observed_velocity_microradians_per_second=observed_velocity,
        branch_equality_ambiguity_count=ambiguities,
        audit=audit,
    )


def load_r120_source_cache(
    path: Path, *, expected_sha256: str
) -> dict[str, NDArray[Any]]:
    """Load only R120 q/v payloads; the stored acceleration is never read."""

    if sha256(path) != expected_sha256:
        raise ExecutionInvalid("R120_CACHE_SHA256_DIFFERS")
    with np.load(path, allow_pickle=False) as archive:
        if tuple(archive.files) != CACHE_KEYS:
            raise ExecutionInvalid("R120_CACHE_KEY_ORDER_DIFFERS")
        cache = {name: np.array(archive[name], copy=True) for name in LOADED_CACHE_KEYS}
    expected_shapes = {
        "root_position_m": (FRAME_COUNT, 3),
        "root_orientation_delta_rad": (FRAME_COUNT, 3),
        "joint_position_rad": (FRAME_COUNT, ACTUATED_JOINT_COUNT),
        "velocity": (FRAME_COUNT, GENERALIZED_WIDTH),
        "reference_root_rotation": (FRAME_COUNT, 3, 3),
        "metadata_json_utf8": (264,),
    }
    if any(cache[name].shape != shape for name, shape in expected_shapes.items()):
        raise ExecutionInvalid("R120_CACHE_SHAPE_DIFFERS")
    if (
        any(cache[name].dtype != np.float64 for name in LOADED_CACHE_KEYS[:-1])
        or cache["metadata_json_utf8"].dtype != np.uint8
        or any(not np.all(np.isfinite(cache[name])) for name in LOADED_CACHE_KEYS[:-1])
    ):
        raise ExecutionInvalid("R120_CACHE_DTYPE_OR_FINITENESS_DIFFERS")
    return cache


def _hybrid_velocity_stencil(
    contact_modes: NDArray[np.uint8],
) -> tuple[IntArray, FloatArray]:
    active = np.zeros((FRAME_COUNT, 2, 2), dtype=np.bool_)
    for frame in range(FRAME_COUNT):
        for side in range(2):
            mode = int(contact_modes[frame, side])
            active[frame, side, 0] = mode in (1, 3)
            active[frame, side, 1] = mode in (2, 3)
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


def _stencil_values(
    values: FloatArray, indices: IntArray, coefficients: FloatArray
) -> FloatArray:
    weights = coefficients.reshape((FRAME_COUNT, 2) + (1,) * (values.ndim - 1))
    return np.sum(values[indices] * weights, axis=1)


def reproduce_r120_array_hashes(
    *,
    cache: Mapping[str, NDArray[Any]],
    v9_arrays: Mapping[str, NDArray[Any]],
) -> dict[str, str]:
    rotations = np.stack(
        [_configuration_at(cache, frame).root_rotation for frame in range(FRAME_COUNT)]
    )
    quaternion = q1_30(
        np.stack([matrix_to_quaternion(rotation) for rotation in rotations])
    )
    source_quaternion = np.asarray(v9_arrays["root_quaternion_q1_30"], dtype=np.int64)
    source_yaw = np.asarray(v9_arrays["root_yaw_urad"], dtype=np.int64)
    if source_quaternion.shape != (FRAME_COUNT, 4) or source_yaw.shape != (
        FRAME_COUNT,
    ):
        raise ExecutionInvalid("V9_ROOT_ORIENTATION_ARRAY_SHAPE_DIFFERS")
    emitted_raw = np.asarray(
        [
            decompose_xzy(
                quaternion_to_matrix(row.astype(np.float64) / float(1 << 30))
            )[2]
            for row in quaternion
        ],
        dtype=np.float64,
    )
    source_raw = np.asarray(
        [
            decompose_xzy(
                quaternion_to_matrix(row.astype(np.float64) / float(1 << 30))
            )[2]
            for row in source_quaternion
        ],
        dtype=np.float64,
    )
    delta = np.arctan2(
        np.sin(emitted_raw - source_raw), np.cos(emitted_raw - source_raw)
    )
    yaw = source_yaw + np.rint(delta * 1_000_000.0).astype(np.int64)
    contact_modes = np.asarray(v9_arrays["contact_modes"], dtype=np.uint8)
    indices, coefficients = _hybrid_velocity_stencil(contact_modes)
    yaw_velocity = np.rint(
        _stencil_values(yaw[:, None].astype(np.float64), indices, coefficients)[:, 0]
    ).astype(np.int64)
    arrays: dict[str, NDArray[Any]] = {
        "root_position_um": np.rint(cache["root_position_m"] * 1_000_000.0).astype(
            np.int64
        ),
        "root_quaternion_q1_30": quaternion,
        "joint_position_urad": np.rint(
            cache["joint_position_rad"] * 1_000_000.0
        ).astype(np.int64),
        "root_linear_velocity_um_s": np.rint(
            cache["velocity"][:, :3] * 1_000_000.0
        ).astype(np.int64),
        "root_yaw_velocity_urad_s": yaw_velocity,
        "joint_velocity_urad_s": np.rint(cache["velocity"][:, 6:] * 1_000_000.0).astype(
            np.int64
        ),
    }
    return {name: _array_sha256(value) for name, value in arrays.items()}


def _validate_contact_rank(jacobian: FloatArray, expected_rank: int) -> None:
    if jacobian.shape[1] != GENERALIZED_WIDTH or not np.all(np.isfinite(jacobian)):
        raise ExecutionInvalid("CONTACT_JACOBIAN_NONFINITE_OR_SHAPE_DIFFERS")
    singular = np.linalg.svd(jacobian, compute_uv=False)
    observed_rank = 0
    for value in singular:
        numeric = float(value)
        if not math.isfinite(numeric) or numeric < 0.0:
            raise ExecutionInvalid("CONTACT_SINGULAR_VALUE_NONFINITE")
        if numeric <= 1.0e-12:
            continue
        if numeric < 1.0e-10:
            raise ExecutionInvalid("CONTACT_SINGULAR_VALUE_INSIDE_RANK_GAP")
        observed_rank += 1
    if observed_rank != expected_rank:
        raise ExecutionInvalid("CONTACT_RANK_OR_ADDITIONAL_NULLITY_DIFFERS")


def _project_velocity_row(
    *,
    model: SpatialModel,
    configuration: Configuration,
    source_velocity: FloatArray,
    active: tuple[int, ...],
    points: tuple[dict[str, Any], ...],
    numeric: Mapping[str, Any],
) -> tuple[FloatArray, FloatArray]:
    if not active:
        return np.array(source_velocity, copy=True), np.zeros(
            GENERALIZED_WIDTH, dtype=np.float64
        )
    mass, kinematics = mass_matrix(model, configuration)
    jacobian = np.vstack(
        [_point_jacobian(model, kinematics, points[point]) for point in active]
    )
    expected_rank = 3 if len(active) == 1 else 5
    _validate_contact_rank(jacobian, expected_rank)
    analysis = project_tangent_velocity(
        mass=mass,
        jacobian=jacobian,
        velocity=source_velocity,
        expected_rank=expected_rank,
        numeric=numeric,
    )
    if analysis.projected_velocity is None or analysis.delta_velocity is None:
        raise ExecutionInvalid("R133_PROJECTED_VELOCITY_ABSENT")
    flat_audit = (
        _flat_line_audit(
            model=model,
            kinematics=kinematics,
            original_velocity=source_velocity,
            projected_velocity=analysis.projected_velocity,
            points=points,
            active=active,
        )
        if len(active) == 2
        else None
    )
    if not _projection_passes(
        analysis=analysis,
        numeric=numeric,
        flat_audit=flat_audit,
    ):
        raise ExecutionInvalid("R133_PROJECTION_CONFORMANCE_GUARD_FAILED")
    return (
        np.asarray(analysis.projected_velocity, dtype=np.float64),
        np.asarray(analysis.delta_velocity, dtype=np.float64),
    )


def _event_addresses_from_r133(events: Sequence[Mapping[str, Any]]) -> IntArray:
    rows = [
        (
            int(row["collocation"]),
            int(row["dof_ordinal"]),
            CONTROLLER_CATEGORY_CODE[str(row["category"])],
            CONTROLLER_STAGE_CODE[str(row["stage"])],
        )
        for row in events
    ]
    return (
        np.asarray(rows, dtype=np.int64).reshape((-1, 4))
        if rows
        else np.empty((0, 4), dtype=np.int64)
    )


def reconstruct_reference(
    *,
    cache: Mapping[str, NDArray[Any]],
    v9_arrays: Mapping[str, NDArray[Any]],
    descriptor: Mapping[str, Any],
    model: SpatialModel,
    points: tuple[dict[str, Any], ...],
    inventory: GraphInventory,
    projection_numeric: Mapping[str, Any],
    expected_r120_hashes: Mapping[str, str],
    expected_r133_hashes: Mapping[str, str],
    r133_report: Mapping[str, Any],
) -> ReferenceReconstruction:
    """Reconstruct R120/R133 and close all array guards before graph call one."""

    r120_hashes = reproduce_r120_array_hashes(cache=cache, v9_arrays=v9_arrays)
    if dict(r120_hashes) != dict(expected_r120_hashes):
        raise ExecutionInvalid("R120_ACCEPTED_ARRAY_HASH_CLOSURE_DIFFERS")

    root_position = np.empty((STATE_NODE_COUNT, 3), dtype=np.float64)
    root_rotation = np.empty((STATE_NODE_COUNT, 3, 3), dtype=np.float64)
    joint_position = np.empty(
        (STATE_NODE_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64
    )
    projected = np.empty((PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
    delta = np.empty_like(projected)
    exit_intervals = frozenset(int(value) for value in EXIT_INTERVALS)
    projection_systems = 0

    for collocation in range(PHYSICS_INTERVAL_COUNT):
        motor, substep = divmod(collocation, SUBSTEPS_PER_INTERVAL)
        configuration = _interpolate_configuration(cache, motor, substep)
        root_position[collocation] = configuration.root_position
        root_rotation[collocation] = configuration.root_rotation
        joint_position[collocation] = configuration.joint_positions
        fraction = substep / SUBSTEPS_PER_INTERVAL
        source_velocity = (
            np.asarray(cache["velocity"][motor], dtype=np.float64)
            if motor in exit_intervals
            else (1.0 - fraction) * np.asarray(cache["velocity"][motor])
            + fraction * np.asarray(cache["velocity"][motor + 1])
        )
        active = inventory.active_points_by_interval[collocation]
        projected[collocation], delta[collocation] = _project_velocity_row(
            model=model,
            configuration=configuration,
            source_velocity=np.asarray(source_velocity, dtype=np.float64),
            active=active,
            points=points,
            numeric=projection_numeric,
        )
        projection_systems += int(bool(active))

    terminal = _configuration_at(cache, FRAME_COUNT - 1)
    root_position[-1] = terminal.root_position
    root_rotation[-1] = terminal.root_rotation
    joint_position[-1] = terminal.joint_positions
    terminal_velocity, _ = _project_velocity_row(
        model=model,
        configuration=terminal,
        source_velocity=np.asarray(cache["velocity"][-1], dtype=np.float64),
        active=inventory.active_points_by_node[-1],
        points=points,
        numeric=projection_numeric,
    )

    source_cache = {"joint_position_rad": np.asarray(cache["joint_position_rad"])}
    r133_exact = derive_eventful_projected_fixed_pd_schedule(
        cache=source_cache,
        descriptor=descriptor,
        projected_velocity=projected,
    )
    integer_command = np.rint(
        np.asarray(cache["joint_position_rad"][:-1]) * 1_000_000.0
    ).astype(np.int64)
    velocity = np.vstack((projected, terminal_velocity[None, :]))
    replay = replay_exact_controller(
        integer_command=integer_command,
        joint_position=joint_position,
        velocity=velocity,
        descriptor=descriptor,
    )
    canonical_events = _event_addresses_from_r133(r133_exact.events)
    if (
        not np.array_equal(
            replay.applied_target, r133_exact.schedule.applied_target_microradians
        )
        or not np.array_equal(
            replay.applied_effort, r133_exact.schedule.applied_effort_micronewton_metres
        )
        or not np.array_equal(replay.event_addresses, canonical_events)
    ):
        raise ExecutionInvalid("EXACT_TRIAL_CONTROLLER_DOES_NOT_REPRODUCE_R133")

    observed_hashes = {
        "projected_velocity": _array_sha256(projected),
        "projection_delta": _array_sha256(delta),
        "applied_target": _array_sha256(replay.applied_target),
        "applied_effort": _array_sha256(replay.applied_effort),
    }
    if observed_hashes != dict(expected_r133_hashes):
        raise ExecutionInvalid("R133_ARRAY_HASH_CLOSURE_DIFFERS")
    schedule = r133_report.get("projected_fixed_pd_schedule_audit", {})
    if (
        schedule.get("status") != "PASS"
        or schedule.get("projected_velocity_float64_sha256")
        != observed_hashes["projected_velocity"]
        or schedule.get("applied_target_float64_sha256")
        != observed_hashes["applied_target"]
        or schedule.get("applied_effort_float64_sha256")
        != observed_hashes["applied_effort"]
    ):
        raise ExecutionInvalid("R133_REPORT_ARRAY_HASH_CLOSURE_DIFFERS")

    acceleration = np.empty(
        (PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH), dtype=np.float64
    )
    for interval in range(PHYSICS_INTERVAL_COUNT):
        left = Configuration(
            root_position[interval], root_rotation[interval], joint_position[interval]
        )
        right = Configuration(
            root_position[interval + 1],
            root_rotation[interval + 1],
            joint_position[interval + 1],
        )
        mapped_velocity = _discrete_velocity(left, right)
        acceleration[interval] = (mapped_velocity - velocity[interval]) / DT

    anchors = np.empty((len(inventory.anchor_rows), POINT_WIDTH), dtype=np.float64)
    for anchor, (node, point) in enumerate(inventory.anchor_rows):
        configuration = Configuration(
            root_position[node], root_rotation[node], joint_position[node]
        )
        anchors[anchor] = _point_position(
            forward_kinematics(model, configuration), points[point]
        )

    state = TrajectoryState(
        root_position=root_position,
        root_rotation=root_rotation,
        joint_position=joint_position,
        velocity=velocity,
        acceleration=acceleration,
        integer_command=integer_command,
        active_force=np.zeros(
            (len(inventory.force_rows), POINT_WIDTH), dtype=np.float64
        ),
        activation_impulse=np.zeros(
            (len(inventory.impulse_rows), POINT_WIDTH), dtype=np.float64
        ),
    )
    return ReferenceReconstruction(
        state=state,
        projected_velocity=projected,
        projection_delta=delta,
        source_controller=replay,
        anchors=anchors,
        source_hashes={**r120_hashes, **observed_hashes},
        configuration_hashes={
            "root_position": _array_sha256(root_position),
            "root_rotation": _array_sha256(root_rotation),
            "joint_position": _array_sha256(joint_position),
            "terminal_velocity": _array_sha256(terminal_velocity),
            "derived_acceleration": _array_sha256(acceleration),
        },
        projection_systems=projection_systems,
        projection_factorizations=projection_systems,
        projection_solves=projection_systems,
    )


def build_contact_anchors(
    *,
    state: TrajectoryState,
    model: SpatialModel,
    points: tuple[dict[str, Any], ...],
    inventory: GraphInventory,
) -> FloatArray:
    anchors = np.empty((len(inventory.anchor_rows), POINT_WIDTH), dtype=np.float64)
    for row, (node, point) in enumerate(inventory.anchor_rows):
        configuration = Configuration(
            state.root_position[node],
            state.root_rotation[node],
            state.joint_position[node],
        )
        anchors[row] = _point_position(
            forward_kinematics(model, configuration), points[point]
        )
    return anchors


class _ExactResidualBuilder:
    def __init__(self) -> None:
        self.residual: list[float] = []
        self.violation: list[float] = []
        self.addresses: list[tuple[int, int, int, int]] = []
        self.counts: Counter[str] = Counter()
        self.maximum: dict[str, float] = {}
        self.hard_pass = True

    def equality(
        self,
        values: NDArray[Any] | Sequence[float],
        *,
        tolerance: float,
        category: str,
        graph_row: int,
        point: int = -1,
    ) -> None:
        array = np.asarray(values, dtype=np.float64).reshape(-1)
        if tolerance <= 0.0 or not np.all(np.isfinite(array)):
            raise ExecutionInvalid("EXACT_EQUALITY_NONFINITE_OR_TOLERANCE_DIFFERS")
        normalized = array / tolerance
        violations = np.abs(normalized)
        self.hard_pass = self.hard_pass and bool(np.all(violations <= 1.0))
        self._extend(
            normalized,
            violations,
            category=category,
            graph_row=graph_row,
            point=point,
        )

    def upper(
        self,
        values: NDArray[Any] | Sequence[float] | float,
        *,
        tolerance: float,
        category: str,
        graph_row: int,
        point: int = -1,
        exact_zero: bool = False,
    ) -> None:
        array = np.asarray(values, dtype=np.float64).reshape(-1)
        if tolerance <= 0.0 or not np.all(np.isfinite(array)):
            raise ExecutionInvalid("EXACT_INEQUALITY_NONFINITE_OR_TOLERANCE_DIFFERS")
        normalized = array / tolerance
        violations = np.maximum(normalized, 0.0)
        self.hard_pass = self.hard_pass and bool(
            np.all(array <= (0.0 if exact_zero else tolerance))
        )
        self._extend(
            normalized,
            violations,
            category=category,
            graph_row=graph_row,
            point=point,
        )

    def _extend(
        self,
        residual: FloatArray,
        violation: FloatArray,
        *,
        category: str,
        graph_row: int,
        point: int,
    ) -> None:
        code = GRAPH_CATEGORY_CODE[category]
        self.residual.extend(float(value) for value in residual)
        self.violation.extend(float(value) for value in violation)
        self.addresses.extend(
            (code, int(graph_row), int(point), component)
            for component in range(len(residual))
        )
        self.counts[category] += len(residual)
        observed = float(np.max(violation, initial=0.0))
        self.maximum[category] = max(self.maximum.get(category, 0.0), observed)

    def arrays(self) -> tuple[FloatArray, FloatArray, IntArray]:
        return (
            np.asarray(self.residual, dtype=np.float64),
            np.asarray(self.violation, dtype=np.float64),
            np.asarray(self.addresses, dtype=np.int64).reshape((-1, 4)),
        )


def _configuration_from_state(state: TrajectoryState, node: int) -> Configuration:
    return Configuration(
        np.asarray(state.root_position[node], dtype=np.float64),
        np.asarray(state.root_rotation[node], dtype=np.float64),
        np.asarray(state.joint_position[node], dtype=np.float64),
    )


def _cone_residual_and_agreement(value: FloatArray) -> tuple[float, float]:
    if value.shape != (POINT_WIDTH,) or not np.all(np.isfinite(value)):
        raise ExecutionInvalid("EXACT_CONE_VALUE_NONFINITE_OR_SHAPE_DIFFERS")
    normal, right, forward = (float(component) for component in value)
    tangential = math.hypot(right, forward)
    norm_inside = normal >= 0.0 and tangential <= FRICTION * normal
    squared_inside = normal >= 0.0 and (
        (right * right + forward * forward) * FRICTION_DENOMINATOR**2
        <= (FRICTION_NUMERATOR * normal) ** 2
    )
    if norm_inside != squared_inside:
        raise ExecutionInvalid("NORM_SQUARED_CONE_CLASSIFICATION_DISAGREEMENT")
    return -normal, tangential - FRICTION * normal


def _all_colliders(
    descriptor: Mapping[str, Any],
) -> tuple[tuple[int, Mapping[str, Any]], ...]:
    rows = tuple(
        (int(body["body_slot"]), collider)
        for body in descriptor.get("bodies", ())
        for collider in body.get("colliders", ())
    )
    if not rows:
        raise ExecutionInvalid("DESCRIPTOR_COLLIDER_INVENTORY_EMPTY")
    return rows


def exact_graph_audit(
    *,
    state: TrajectoryState,
    immutable_initial: TrajectoryState,
    anchors: FloatArray,
    descriptor: Mapping[str, Any],
    model: SpatialModel,
    points: tuple[dict[str, Any], ...],
    inventory: GraphInventory,
) -> ExactAudit:
    counters = {name: 0 for name in REQUIRED_COUNTERS}
    try:
        if (
            not np.array_equal(
                state.root_position[0], immutable_initial.root_position[0]
            )
            or not np.array_equal(
                state.root_rotation[0], immutable_initial.root_rotation[0]
            )
            or not np.array_equal(
                state.joint_position[0], immutable_initial.joint_position[0]
            )
            or not np.array_equal(state.velocity[0], immutable_initial.velocity[0])
        ):
            raise ExecutionInvalid("IMMUTABLE_INITIAL_Q_V_DIFFERS")
        expected_shapes = (
            (state.root_position, (STATE_NODE_COUNT, 3)),
            (state.root_rotation, (STATE_NODE_COUNT, 3, 3)),
            (state.joint_position, (STATE_NODE_COUNT, ACTUATED_JOINT_COUNT)),
            (state.velocity, (STATE_NODE_COUNT, GENERALIZED_WIDTH)),
            (state.acceleration, (PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH)),
            (
                state.integer_command,
                (MOTOR_INTERVAL_COUNT, ACTUATED_JOINT_COUNT),
            ),
            (state.active_force, (len(inventory.force_rows), POINT_WIDTH)),
            (
                state.activation_impulse,
                (len(inventory.impulse_rows), POINT_WIDTH),
            ),
        )
        if any(value.shape != shape for value, shape in expected_shapes):
            raise ExecutionInvalid("EXACT_STATE_SHAPE_DIFFERS")
        if state.integer_command.dtype != np.int64 or any(
            value.dtype != np.float64
            for value, _ in expected_shapes
            if value is not state.integer_command
        ):
            raise ExecutionInvalid("EXACT_STATE_DTYPE_DIFFERS")
        if any(
            not np.all(np.isfinite(value))
            for value, _ in expected_shapes
            if value is not state.integer_command
        ):
            raise ExecutionInvalid("EXACT_STATE_NONFINITE")

        controller = replay_exact_controller(
            integer_command=state.integer_command,
            joint_position=state.joint_position,
            velocity=state.velocity,
            descriptor=descriptor,
        )
        counters["real_controller_schedule_derivations"] = 1
        counters["real_controller_graph_evaluations"] = 1
        builder = _ExactResidualBuilder()

        kinematics: list[Kinematics] = []
        colliders = _all_colliders(descriptor)
        joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
        hard_min = _column(joints, "hard_limit_microradians", 0)
        hard_max = _column(joints, "hard_limit_microradians", 1)
        velocity_max = np.asarray(
            [row["maximum_velocity_microradians_per_second"] for row in joints],
            dtype=np.float64,
        )

        for node in range(STATE_NODE_COUNT):
            configuration = _configuration_from_state(state, node)
            node_kinematics = forward_kinematics(model, configuration)
            kinematics.append(node_kinematics)
            q_microradians = np.rint(state.joint_position[node] * 1_000_000.0)
            v_microradians = np.rint(state.velocity[node, 6:] * 1_000_000.0)
            builder.upper(
                np.maximum(hard_min - q_microradians, q_microradians - hard_max),
                tolerance=10.0,
                category="joint_rom",
                graph_row=node,
                exact_zero=False,
            )
            builder.upper(
                np.abs(v_microradians) - velocity_max,
                tolerance=1.0,
                category="joint_velocity",
                graph_row=node,
                exact_zero=True,
            )
            minimum_collider = min(
                collider_minimum_y(
                    node_kinematics.body_positions[slot],
                    node_kinematics.body_rotations[slot],
                    dict(collider),
                )
                for slot, collider in colliders
            )
            builder.upper(
                -2.0e-6 - minimum_collider,
                tolerance=1.0e-6,
                category="collider_floor",
                graph_row=node,
                exact_zero=True,
            )
        counters["real_state_lift_evaluations"] = STATE_NODE_COUNT

        for point in inventory.active_points_by_node[0]:
            anchor_index = int(inventory.anchor_index_by_node_point[0, point])
            jacobian = _point_jacobian(model, kinematics[0], points[point])
            counters["real_contact_jacobian_assemblies"] += 1
            builder.equality(
                _point_position(kinematics[0], points[point]) - anchors[anchor_index],
                tolerance=1.0e-6,
                category="initial_anchor_position",
                graph_row=0,
                point=point,
            )
            builder.equality(
                jacobian @ state.velocity[0],
                tolerance=1.0e-6,
                category="initial_sticking_velocity",
                graph_row=0,
                point=point,
            )

        violated_cones: list[dict[str, Any]] = []
        for interval in range(PHYSICS_INTERVAL_COUNT):
            left = _configuration_from_state(state, interval)
            right = _configuration_from_state(state, interval + 1)
            active = inventory.active_points_by_interval[interval]
            next_active = inventory.active_points_by_node[interval + 1]
            left_kinematics = kinematics[interval]
            right_kinematics = kinematics[interval + 1]
            jacobians = {
                point: _point_jacobian(model, left_kinematics, points[point])
                for point in active
            }
            counters["real_contact_jacobian_assemblies"] += len(jacobians)
            if active:
                stacked = np.vstack([jacobians[point] for point in active])
                _validate_contact_rank(stacked, 3 if len(active) == 1 else 5)
                counters["real_factorizations"] += 1

            inertial = inverse_dynamics(
                model,
                left,
                state.velocity[interval],
                state.acceleration[interval],
            )
            effort = controller.applied_effort[interval] / 1_000_000.0
            generalized = np.array(inertial, copy=True)
            generalized[6:] -= effort
            force_terms: list[FloatArray] = []
            for point in active:
                force_row = inventory.force_row_by_interval_point[(interval, point)]
                term = generalized_contact_force(
                    jacobians[point], state.active_force[force_row]
                )
                generalized -= term
                force_terms.append(term)
            dynamics_scale = _dynamics_scale(inertial, effort, force_terms)
            builder.equality(
                generalized / dynamics_scale,
                tolerance=1.0e-7,
                category="continuous_dynamics",
                graph_row=interval,
            )
            counters["real_dynamics_residual_evaluations"] += 1

            v_bar = state.velocity[interval] + DT * state.acceleration[interval]
            builder.equality(
                np.zeros(GENERALIZED_WIDTH, dtype=np.float64),
                tolerance=1.0e-7,
                category="pre_event_velocity_definition",
                graph_row=interval,
            )
            predicted = integrate_configuration(left, v_bar, DT)
            integration = np.concatenate(
                (
                    right.root_position - predicted.root_position,
                    _rotation_log(right.root_rotation @ predicted.root_rotation.T),
                    right.joint_positions - predicted.joint_positions,
                )
            )
            builder.equality(
                integration,
                tolerance=1.0e-7,
                category="manifold_integration",
                graph_row=interval,
            )
            counters["real_integration_residual_evaluations"] += 1

            activated = tuple(sorted(set(next_active) - set(active)))
            if activated:
                mass, _ = mass_matrix(model, right)
                counters["real_mass_matrix_assemblies"] += 1
                jump = mass @ (state.velocity[interval + 1] - v_bar)
                impulse_terms: list[FloatArray] = []
                for point in activated:
                    jacobian = _point_jacobian(model, right_kinematics, points[point])
                    counters["real_contact_jacobian_assemblies"] += 1
                    impulse_row = inventory.impulse_row_by_node_point[
                        (interval + 1, point)
                    ]
                    term = generalized_contact_force(
                        jacobian, state.activation_impulse[impulse_row]
                    )
                    jump -= term
                    impulse_terms.append(term)
                jump_scale = max(
                    1.0,
                    float(np.max(np.abs(mass @ state.velocity[interval + 1]))),
                    float(np.max(np.abs(mass @ v_bar))),
                    *(float(np.max(np.abs(value))) for value in impulse_terms),
                )
                builder.equality(
                    jump / jump_scale,
                    tolerance=1.0e-7,
                    category="activation_momentum_jump",
                    graph_row=interval,
                )
                counters["real_impulse_residual_evaluations"] += 1
            else:
                builder.equality(
                    state.velocity[interval + 1] - v_bar,
                    tolerance=1.0e-7,
                    category="ordinary_velocity_update",
                    graph_row=interval,
                )

            for point in next_active:
                anchor_index = int(
                    inventory.anchor_index_by_node_point[interval + 1, point]
                )
                jacobian = _point_jacobian(model, right_kinematics, points[point])
                counters["real_contact_jacobian_assemblies"] += 1
                builder.equality(
                    _point_position(right_kinematics, points[point])
                    - anchors[anchor_index],
                    tolerance=1.0e-6,
                    category="active_anchor_position",
                    graph_row=interval,
                    point=point,
                )
                builder.equality(
                    jacobian @ state.velocity[interval + 1],
                    tolerance=1.0e-6,
                    category="active_sticking_velocity",
                    graph_row=interval,
                    point=point,
                )

            if active:
                motion = propagate_motion(
                    model,
                    left_kinematics,
                    state.velocity[interval],
                    state.acceleration[interval],
                )
                for point in active:
                    builder.equality(
                        point_acceleration(
                            left_kinematics,
                            motion,
                            int(points[point]["body_slot"]),
                            np.asarray(
                                points[point]["local_translation_metres"],
                                dtype=np.float64,
                            ),
                        ),
                        tolerance=1.0e-5,
                        category="active_sticking_acceleration",
                        graph_row=interval,
                        point=point,
                    )

            for point in active:
                force_row = inventory.force_row_by_interval_point[(interval, point)]
                normal_residual, cone_residual = _cone_residual_and_agreement(
                    state.active_force[force_row]
                )
                builder.upper(
                    normal_residual,
                    tolerance=1.0e-7,
                    category="force_normal",
                    graph_row=interval,
                    point=point,
                )
                builder.upper(
                    cone_residual,
                    tolerance=1.0e-7,
                    category="force_circular_cone",
                    graph_row=interval,
                    point=point,
                )
                if normal_residual > 1.0e-7 or cone_residual > 1.0e-7:
                    violated_cones.append(
                        {
                            "graph_row": interval,
                            "cone_type": "continuous_force",
                            "point_ordinal": point,
                            "row": force_row,
                            "right": float(state.active_force[force_row, 1]),
                            "forward": float(state.active_force[force_row, 2]),
                        }
                    )
            for point in activated:
                impulse_row = inventory.impulse_row_by_node_point[(interval + 1, point)]
                normal_residual, cone_residual = _cone_residual_and_agreement(
                    state.activation_impulse[impulse_row]
                )
                builder.upper(
                    normal_residual,
                    tolerance=1.0e-7,
                    category="impulse_normal",
                    graph_row=interval,
                    point=point,
                )
                builder.upper(
                    cone_residual,
                    tolerance=1.0e-7,
                    category="impulse_circular_cone",
                    graph_row=interval,
                    point=point,
                )
                if normal_residual > 1.0e-7 or cone_residual > 1.0e-7:
                    violated_cones.append(
                        {
                            "graph_row": interval,
                            "cone_type": "activation_impulse",
                            "point_ordinal": point,
                            "row": impulse_row,
                            "right": float(state.activation_impulse[impulse_row, 1]),
                            "forward": float(state.activation_impulse[impulse_row, 2]),
                        }
                    )

        controller_counts = controller.audit["activation_counts"]
        builder.upper(
            float(controller_counts["hard_rom_violation"]),
            tolerance=1.0,
            category="controller_hard_rom",
            graph_row=-1,
            exact_zero=True,
        )
        builder.upper(
            float(controller_counts["velocity_violation"]),
            tolerance=1.0,
            category="controller_velocity",
            graph_row=-1,
            exact_zero=True,
        )
        builder.upper(
            float(controller_counts["infeasible_effort_envelope"]),
            tolerance=1.0,
            category="controller_envelope",
            graph_row=-1,
            exact_zero=True,
        )
        residual, violation, addresses = builder.arrays()
        if not np.all(np.isfinite(residual)) or not np.all(np.isfinite(violation)):
            raise ExecutionInvalid("EXACT_RESIDUAL_VECTOR_NONFINITE")
        funnel = (
            float(np.max(violation, initial=0.0)),
            float(np.mean(violation)) if len(violation) else 0.0,
        )
        hard_pass = bool(builder.hard_pass and controller.audit["status"] == "PASS")
        return ExactAudit(
            status="PASS" if hard_pass else "VALID_INFEASIBLE",
            invalid_reason=None,
            controller=controller,
            residual_vector=residual,
            violation_vector=violation,
            graph_row_addresses=addresses,
            funnel=funnel,
            hard_pass=hard_pass,
            category_aggregate={
                "row_counts": dict(sorted(builder.counts.items())),
                "maximum_normalized_violation": dict(sorted(builder.maximum.items())),
                "residual_rows": len(residual),
                "maximum_normalized_violation_overall": funnel[0],
                "mean_normalized_violation_overall": funnel[1],
            },
            violated_cones=tuple(violated_cones),
            counters=counters,
        )
    except Exception as error:
        invalid_reason = _stable_invalid_reason(error, prefix="EXACT_AUDIT_EXCEPTION")
        return ExactAudit(
            status="INVALID",
            invalid_reason=invalid_reason,
            controller=None,
            residual_vector=None,
            violation_vector=None,
            graph_row_addresses=None,
            funnel=None,
            hard_pass=False,
            category_aggregate={},
            violated_cones=(),
            counters=counters,
        )


def build_variable_layout(
    *,
    state: TrajectoryState,
    descriptor: Mapping[str, Any],
    inventory: GraphInventory,
    radius: float,
) -> VariableLayout:
    if not 0.03125 <= radius <= 2.0 or not math.isfinite(radius):
        raise ExecutionInvalid("TRUST_RADIUS_DIFFERS")
    cursor = 0

    def allocate(count: int) -> slice:
        nonlocal cursor
        result = slice(cursor, cursor + count)
        cursor += count
        return result

    q = allocate(STATE_NODE_COUNT * GENERALIZED_WIDTH)
    velocity = allocate(STATE_NODE_COUNT * GENERALIZED_WIDTH)
    acceleration = allocate(PHYSICS_INTERVAL_COUNT * GENERALIZED_WIDTH)
    command = allocate(MOTOR_INTERVAL_COUNT * ACTUATED_JOINT_COUNT)
    force = allocate(len(inventory.force_rows) * POINT_WIDTH)
    impulse = allocate(len(inventory.impulse_rows) * POINT_WIDTH)
    primary_count = cursor
    applied_target = allocate(MOTOR_INTERVAL_COUNT * ACTUATED_JOINT_COUNT)
    applied_effort = allocate(PHYSICS_INTERVAL_COUNT * ACTUATED_JOINT_COUNT)
    work = allocate(PHYSICS_INTERVAL_COUNT * ACTUATED_JOINT_COUNT)
    base_count = cursor
    if primary_count != 311_780:
        raise ExecutionInvalid("PRIMARY_VARIABLE_INVENTORY_DIFFERS")

    joints = sorted(descriptor["joints"], key=lambda row: int(row["dof_ordinal"]))
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    effort_min = _column(actuators, "effort_micronewton_metres", 0)
    effort_max = _column(actuators, "effort_micronewton_metres", 1)
    effort_scale = np.maximum(np.maximum(np.abs(effort_min), np.abs(effort_max)), 1.0)
    work_scale = np.maximum(
        np.asarray(
            [
                row["maximum_positive_work_microjoules_per_motor_tick"]
                for row in actuators
            ],
            dtype=np.float64,
        ),
        1.0,
    )
    q_scale = np.asarray(
        [0.01] * 3 + [0.025] * 3 + [0.025] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )
    velocity_scale = np.asarray(
        [0.25] * 3 + [1.0] * 3 + [2.0] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )
    acceleration_scale = np.asarray(
        [5.0] * 3 + [20.0] * 3 + [20.0] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )
    scale = np.empty(base_count, dtype=np.float64)
    scale[q] = np.tile(q_scale, STATE_NODE_COUNT)
    scale[velocity] = np.tile(velocity_scale, STATE_NODE_COUNT)
    scale[acceleration] = np.tile(acceleration_scale, PHYSICS_INTERVAL_COUNT)
    scale[command] = 25_000.0
    scale[force] = 250.0
    scale[impulse] = 25.0
    scale[applied_target] = 25_000.0
    scale[applied_effort] = np.tile(effort_scale, PHYSICS_INTERVAL_COUNT)
    scale[work] = np.tile(work_scale, PHYSICS_INTERVAL_COUNT)
    if not np.all(np.isfinite(scale)) or np.any(scale <= 0.0):
        raise ExecutionInvalid("VARIABLE_SCALE_NONFINITE_OR_NONPOSITIVE")

    lower = np.full(base_count, -np.inf, dtype=np.float64)
    upper = np.full(base_count, np.inf, dtype=np.float64)
    lower[:primary_count] = -radius
    upper[:primary_count] = radius
    lower[q.start : q.start + GENERALIZED_WIDTH] = 0.0
    upper[q.start : q.start + GENERALIZED_WIDTH] = 0.0
    lower[velocity.start : velocity.start + GENERALIZED_WIDTH] = 0.0
    upper[velocity.start : velocity.start + GENERALIZED_WIDTH] = 0.0

    hard_min = _column(joints, "hard_limit_microradians", 0) / 1_000_000.0
    hard_max = _column(joints, "hard_limit_microradians", 1) / 1_000_000.0
    maximum_velocity = (
        np.asarray(
            [row["maximum_velocity_microradians_per_second"] for row in joints],
            dtype=np.float64,
        )
        / 1_000_000.0
    )
    for node in range(STATE_NODE_COUNT):
        q_base = q.start + node * GENERALIZED_WIDTH + 6
        v_base = velocity.start + node * GENERALIZED_WIDTH + 6
        q_denominator = scale[q_base : q_base + ACTUATED_JOINT_COUNT]
        v_denominator = scale[v_base : v_base + ACTUATED_JOINT_COUNT]
        lower[q_base : q_base + ACTUATED_JOINT_COUNT] = np.maximum(
            lower[q_base : q_base + ACTUATED_JOINT_COUNT],
            (hard_min - state.joint_position[node]) / q_denominator,
        )
        upper[q_base : q_base + ACTUATED_JOINT_COUNT] = np.minimum(
            upper[q_base : q_base + ACTUATED_JOINT_COUNT],
            (hard_max - state.joint_position[node]) / q_denominator,
        )
        lower[v_base : v_base + ACTUATED_JOINT_COUNT] = np.maximum(
            lower[v_base : v_base + ACTUATED_JOINT_COUNT],
            (-maximum_velocity - state.velocity[node, 6:]) / v_denominator,
        )
        upper[v_base : v_base + ACTUATED_JOINT_COUNT] = np.minimum(
            upper[v_base : v_base + ACTUATED_JOINT_COUNT],
            (maximum_velocity - state.velocity[node, 6:]) / v_denominator,
        )
    for row in range(len(inventory.force_rows)):
        index = force.start + row * POINT_WIDTH
        lower[index] = max(lower[index], -state.active_force[row, 0] / scale[index])
    for row in range(len(inventory.impulse_rows)):
        index = impulse.start + row * POINT_WIDTH
        lower[index] = max(
            lower[index], -state.activation_impulse[row, 0] / scale[index]
        )
    if (
        np.any(lower > upper)
        or not np.all(np.isfinite(lower[:primary_count]))
        or not np.all(np.isfinite(upper[:primary_count]))
    ):
        raise ExecutionInvalid("VARIABLE_LOWER_UPPER_BOUND_UNORDERED_OR_NONFINITE")
    return VariableLayout(
        q=q,
        velocity=velocity,
        acceleration=acceleration,
        command=command,
        force=force,
        impulse=impulse,
        applied_target=applied_target,
        applied_effort=applied_effort,
        work=work,
        primary_count=primary_count,
        base_count=base_count,
        scale=scale,
        lower=lower,
        upper=upper,
        effort_scale=effort_scale,
        work_scale=work_scale,
    )


class _SparseChunks:
    def __init__(self) -> None:
        self.rows: list[IntArray] = []
        self.columns: list[IntArray] = []
        self.values: list[FloatArray] = []

    def add_block(
        self,
        *,
        row_start: int,
        columns: IntArray,
        values: FloatArray,
    ) -> None:
        matrix = np.asarray(values, dtype=np.float64)
        column_array = np.asarray(columns, dtype=np.int64)
        if matrix.ndim != 2 or matrix.shape[1] != len(column_array):
            raise ExecutionInvalid("SPARSE_BLOCK_SHAPE_DIFFERS")
        if not np.all(np.isfinite(matrix)):
            raise ExecutionInvalid("SPARSE_BLOCK_NONFINITE")
        row_index, local_column = np.nonzero(np.abs(matrix) > 1.0e-14)
        if len(row_index) == 0:
            return
        self.rows.append(row_index.astype(np.int64) + row_start)
        self.columns.append(column_array[local_column])
        self.values.append(matrix[row_index, local_column])

    def add_entries(
        self, *, rows: IntArray, columns: IntArray, values: FloatArray
    ) -> None:
        row_array = np.asarray(rows, dtype=np.int64)
        column_array = np.asarray(columns, dtype=np.int64)
        value_array = np.asarray(values, dtype=np.float64)
        if (
            row_array.shape != column_array.shape
            or row_array.shape != value_array.shape
            or not np.all(np.isfinite(value_array))
        ):
            raise ExecutionInvalid("SPARSE_ENTRY_SHAPE_OR_FINITE_DIFFERS")
        keep = np.abs(value_array) > 1.0e-14
        if np.any(keep):
            self.rows.append(row_array[keep])
            self.columns.append(column_array[keep])
            self.values.append(value_array[keep])

    def matrix(self, *, row_count: int, column_count: int) -> sparse.csc_matrix:
        if self.rows:
            rows = np.concatenate(self.rows)
            columns = np.concatenate(self.columns)
            values = np.concatenate(self.values)
        else:
            rows = np.empty(0, dtype=np.int64)
            columns = np.empty(0, dtype=np.int64)
            values = np.empty(0, dtype=np.float64)
        if (
            np.any(rows < 0)
            or np.any(rows >= row_count)
            or np.any(columns < 0)
            or np.any(columns >= column_count)
        ):
            raise ExecutionInvalid("SPARSE_ENTRY_INDEX_OUT_OF_RANGE")
        return sparse.coo_matrix(
            (values, (rows, columns)), shape=(row_count, column_count)
        ).tocsc()


class _PhysicalModelBuilder:
    def __init__(self, layout: VariableLayout) -> None:
        self.layout = layout
        self.sparse = _SparseChunks()
        self.residual: list[float] = []
        self.funnel_multiplier: list[float] = []
        self.upper: list[bool] = []
        self.addresses: list[tuple[int, int, int, int]] = []

    def begin(
        self,
        residual: NDArray[Any] | Sequence[float],
        *,
        phase_scale: NDArray[Any] | Sequence[float] | float,
        acceptance_tolerance: float,
        category: str,
        graph_row: int,
        point: int = -1,
        upper: bool = False,
    ) -> tuple[int, FloatArray]:
        raw = np.asarray(residual, dtype=np.float64).reshape(-1)
        scale = np.broadcast_to(
            np.asarray(phase_scale, dtype=np.float64), raw.shape
        ).copy()
        if (
            acceptance_tolerance <= 0.0
            or not np.all(np.isfinite(raw))
            or not np.all(np.isfinite(scale))
            or np.any(scale <= 0.0)
        ):
            raise ExecutionInvalid("PHYSICAL_MODEL_RESIDUAL_OR_SCALE_DIFFERS")
        start = len(self.residual)
        self.residual.extend(float(value) for value in raw / scale)
        self.funnel_multiplier.extend(
            float(value) for value in scale / acceptance_tolerance
        )
        self.upper.extend([upper] * len(raw))
        code = GRAPH_CATEGORY_CODE[category]
        self.addresses.extend(
            (code, int(graph_row), int(point), component)
            for component in range(len(raw))
        )
        return start, scale

    def add_block(
        self,
        *,
        row_start: int,
        columns: IntArray,
        derivative: FloatArray,
        row_scale: FloatArray,
    ) -> None:
        scaled = (
            np.asarray(derivative, dtype=np.float64)
            * self.layout.scale[np.asarray(columns, dtype=np.int64)][None, :]
            / row_scale[:, None]
        )
        self.sparse.add_block(row_start=row_start, columns=columns, values=scaled)

    def finish(
        self,
    ) -> tuple[sparse.csc_matrix, FloatArray, FloatArray, BoolArray, IntArray]:
        row_count = len(self.residual)
        return (
            self.sparse.matrix(
                row_count=row_count, column_count=self.layout.base_count
            ),
            np.asarray(self.residual, dtype=np.float64),
            np.asarray(self.funnel_multiplier, dtype=np.float64),
            np.asarray(self.upper, dtype=np.bool_),
            np.asarray(self.addresses, dtype=np.int64).reshape((-1, 4)),
        )


class _HardModelBuilder:
    def __init__(self, layout: VariableLayout) -> None:
        self.layout = layout
        self.sparse = _SparseChunks()
        self.lower: list[float] = []
        self.upper: list[float] = []

    def add_row(
        self,
        entries: Sequence[tuple[int, float]],
        *,
        lower: float,
        upper: float,
        row_scale: float = 1.0,
    ) -> None:
        if (
            not math.isfinite(lower)
            and lower != -math.inf
            or not math.isfinite(upper)
            and upper != math.inf
            or lower > upper
            or not math.isfinite(row_scale)
            or row_scale <= 0.0
        ):
            raise ExecutionInvalid("HARD_MODEL_BOUND_OR_SCALE_DIFFERS")
        row = len(self.lower)
        columns = np.asarray([column for column, _ in entries], dtype=np.int64)
        values = np.asarray(
            [coefficient for _, coefficient in entries], dtype=np.float64
        )
        if (
            np.any(columns < 0)
            or np.any(columns >= self.layout.base_count)
            or not np.all(np.isfinite(values))
        ):
            raise ExecutionInvalid("HARD_MODEL_ENTRY_NONFINITE_OR_INDEX_DIFFERS")
        if len(columns):
            values = values * self.layout.scale[columns] / row_scale
            self.sparse.add_entries(
                rows=np.full(len(columns), row, dtype=np.int64),
                columns=columns,
                values=values,
            )
        self.lower.append(lower / row_scale)
        self.upper.append(upper / row_scale)

    def finish(self) -> tuple[sparse.csc_matrix, FloatArray, FloatArray]:
        row_count = len(self.lower)
        return (
            self.sparse.matrix(
                row_count=row_count, column_count=self.layout.base_count
            ),
            np.asarray(self.lower, dtype=np.float64),
            np.asarray(self.upper, dtype=np.float64),
        )


def _target_derivative_entries(
    *,
    interval: int,
    dof: int,
    replay: ControllerReplay,
    layout: VariableLayout,
) -> list[tuple[int, float]]:
    soft_code = int(replay.target_branch_addresses[interval, dof, 0])
    slew_code = int(replay.target_branch_addresses[interval, dof, 1])
    soft_command = 1.0 if soft_code == 2 else 0.0
    entries: list[tuple[int, float]] = [(layout.target_index(interval, dof), 1.0)]
    if slew_code == 2 and soft_command:
        entries.append((layout.command_index(interval, dof), -1.0))
    elif slew_code in (0, 4):
        previous = (
            layout.command_index(0, dof)
            if interval == 0
            else layout.target_index(interval - 1, dof)
        )
        entries.append((previous, -1.0))
    return entries


def _effort_selected_derivative(
    *,
    interval: int,
    dof: int,
    state: TrajectoryState,
    replay: ControllerReplay,
    descriptor: Mapping[str, Any],
    layout: VariableLayout,
) -> list[tuple[int, float]]:
    actuators = sorted(descriptor["actuators"], key=lambda row: int(row["dof_ordinal"]))
    actuator = actuators[dof]
    target_coefficient = float(actuator["stiffness_q16"]) / 65_536.0
    q_coefficient = -float(actuator["stiffness_q16"]) * 1_000_000.0 / 65_536.0
    v_coefficient = -float(actuator["damping_q16"]) * 1_000_000.0 / 65_536.0
    requested = [
        (
            layout.target_index(interval // SUBSTEPS_PER_INTERVAL, dof),
            target_coefficient,
        ),
        (layout.q_index(interval, 6 + dof), q_coefficient),
        (layout.velocity_index(interval, 6 + dof), v_coefficient),
    ]
    branch = replay.branch_addresses[interval, dof]
    lower_source = int(branch[0])
    upper_source = int(branch[1])
    final_code = int(branch[2])
    lower_equal = bool(branch[4])
    upper_equal = bool(branch[5])
    final_equal = bool(branch[6])
    if final_equal:
        return []

    def bound_entries(
        source: int, *, upper: bool, equality: bool
    ) -> list[tuple[int, float]]:
        if equality or source in (0, 255):
            return []
        if source == 1:
            return (
                [(layout.effort_index(interval - 1, dof), 1.0)] if interval > 0 else []
            )
        velocity_rad = float(state.velocity[interval, 6 + dof])
        if velocity_rad == 0.0:
            return []
        sign = math.copysign(1.0, velocity_rad)
        absolute = abs(velocity_rad)
        if source == 2:
            power = float(actuator["maximum_power_microwatts"])
            derivative = -power * sign / (absolute * absolute)
            return [
                (
                    layout.velocity_index(interval, 6 + dof),
                    derivative if upper else -derivative,
                )
            ]
        if source == 3:
            remaining = float(
                actuator["maximum_positive_work_microjoules_per_motor_tick"]
            ) - float(replay.work_before[interval, dof])
            velocity_derivative = -remaining * 240.0 * sign / (absolute * absolute)
            work_derivative = -240.0 / absolute
            entries = [
                (
                    layout.velocity_index(interval, 6 + dof),
                    velocity_derivative if upper else -velocity_derivative,
                )
            ]
            if interval % SUBSTEPS_PER_INTERVAL:
                entries.append(
                    (
                        layout.work_index(interval - 1, dof),
                        work_derivative if upper else -work_derivative,
                    )
                )
            return entries
        raise ExecutionInvalid("CONTROLLER_LIMITER_BRANCH_SOURCE_DIFFERS")

    if final_code == 2:
        return requested
    if final_code in (0, 1):
        return bound_entries(lower_source, upper=False, equality=lower_equal)
    if final_code in (3, 4):
        return bound_entries(upper_source, upper=True, equality=upper_equal)
    raise ExecutionInvalid("CONTROLLER_FINAL_BRANCH_CODE_DIFFERS")


def _add_controller_surrogate_rows(
    *,
    hard: _HardModelBuilder,
    state: TrajectoryState,
    replay: ControllerReplay,
    descriptor: Mapping[str, Any],
    layout: VariableLayout,
) -> None:
    for interval in range(MOTOR_INTERVAL_COUNT):
        for dof in range(ACTUATED_JOINT_COUNT):
            hard.add_row(
                _target_derivative_entries(
                    interval=interval, dof=dof, replay=replay, layout=layout
                ),
                lower=0.0,
                upper=0.0,
                row_scale=25_000.0,
            )
    for interval in range(PHYSICS_INTERVAL_COUNT):
        for dof in range(ACTUATED_JOINT_COUNT):
            selected = _effort_selected_derivative(
                interval=interval,
                dof=dof,
                state=state,
                replay=replay,
                descriptor=descriptor,
                layout=layout,
            )
            hard.add_row(
                [
                    (layout.effort_index(interval, dof), 1.0),
                    *((column, -coefficient) for column, coefficient in selected),
                ],
                lower=0.0,
                upper=0.0,
                row_scale=float(layout.effort_scale[dof]),
            )
            work_entries: list[tuple[int, float]] = [
                (layout.work_index(interval, dof), 1.0)
            ]
            if interval % SUBSTEPS_PER_INTERVAL:
                work_entries.append((layout.work_index(interval - 1, dof), -1.0))
            charge_code = int(replay.branch_addresses[interval, dof, 3])
            if charge_code == 1:
                effort = float(replay.applied_effort[interval, dof])
                velocity = float(state.velocity[interval, 6 + dof])
                work_entries.extend(
                    (
                        (layout.effort_index(interval, dof), -velocity / 240.0),
                        (
                            layout.velocity_index(interval, 6 + dof),
                            -effort / 240.0,
                        ),
                    )
                )
            elif charge_code not in (0, 2):
                raise ExecutionInvalid("CONTROLLER_WORK_BRANCH_CODE_DIFFERS")
            hard.add_row(
                work_entries,
                lower=0.0,
                upper=0.0,
                row_scale=float(layout.work_scale[dof]),
            )


def _dynamics_scale(
    inertial: FloatArray,
    effort: FloatArray,
    force_terms: Sequence[FloatArray],
) -> float:
    values = [
        1.0,
        float(np.max(np.abs(inertial), initial=0.0)),
        float(np.max(np.abs(effort), initial=0.0)),
    ]
    values.extend(float(np.max(np.abs(term), initial=0.0)) for term in force_terms)
    return max(values)


def _add_force_cone_rows(
    *,
    physical: _PhysicalModelBuilder,
    state: TrajectoryState,
    inventory: GraphInventory,
    layout: VariableLayout,
    separation_directions: Mapping[tuple[str, int], Sequence[tuple[float, float]]],
) -> None:
    base_directions = tuple(
        (math.cos(math.tau * ordinal / 32), math.sin(math.tau * ordinal / 32))
        for ordinal in range(32)
    )

    def add_cone(
        *,
        cone_type: str,
        row: int,
        value: FloatArray,
        index: Any,
        physical_scale: float,
        graph_row: int,
        point: int,
    ) -> None:
        normal_category = (
            "force_normal" if cone_type == "continuous_force" else "impulse_normal"
        )
        cone_category = (
            "force_circular_cone"
            if cone_type == "continuous_force"
            else "impulse_circular_cone"
        )
        start, scale = physical.begin(
            (-float(value[0]),),
            phase_scale=physical_scale,
            acceptance_tolerance=1.0e-7,
            category=normal_category,
            graph_row=graph_row,
            point=point,
            upper=True,
        )
        physical.add_block(
            row_start=start,
            columns=np.asarray((index(row, 0),), dtype=np.int64),
            derivative=np.asarray(((-1.0,),), dtype=np.float64),
            row_scale=scale,
        )
        directions = [*base_directions]
        directions.extend(separation_directions.get((cone_type, row), ()))
        for right, forward in directions:
            norm = math.hypot(right, forward)
            if not math.isfinite(norm) or norm <= 0.0:
                raise ExecutionInvalid("CONE_SEPARATION_DIRECTION_DIFFERS")
            unit_right = right / norm
            unit_forward = forward / norm
            current = (
                unit_right * float(value[1])
                + unit_forward * float(value[2])
                - FRICTION * float(value[0])
            )
            start, scale = physical.begin(
                (current,),
                phase_scale=physical_scale,
                acceptance_tolerance=1.0e-7,
                category=cone_category,
                graph_row=graph_row,
                point=point,
                upper=True,
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(index(row, 0), index(row, 0) + POINT_WIDTH),
                derivative=np.asarray(
                    ((-FRICTION, unit_right, unit_forward),), dtype=np.float64
                ),
                row_scale=scale,
            )

    for row, ((interval, point), value) in enumerate(
        zip(inventory.force_rows, state.active_force, strict=True)
    ):
        add_cone(
            cone_type="continuous_force",
            row=row,
            value=value,
            index=layout.force_index,
            physical_scale=250.0,
            graph_row=interval,
            point=point,
        )
    for row, ((node, point), value) in enumerate(
        zip(inventory.impulse_rows, state.activation_impulse, strict=True)
    ):
        add_cone(
            cone_type="activation_impulse",
            row=row,
            value=value,
            index=layout.impulse_index,
            physical_scale=25.0,
            graph_row=node - 1,
            point=point,
        )


def assemble_local_model(
    *,
    state: TrajectoryState,
    exact_anchor: ExactAudit,
    anchors: FloatArray,
    descriptor: Mapping[str, Any],
    model: SpatialModel,
    points: tuple[dict[str, Any], ...],
    inventory: GraphInventory,
    radius: float,
    separation_directions: Mapping[tuple[str, int], Sequence[tuple[float, float]]],
) -> LocalModel:
    if exact_anchor.status == "INVALID" or exact_anchor.controller is None:
        raise ExecutionInvalid("LOCAL_MODEL_REQUIRES_VALID_EXACT_ANCHOR")
    layout = build_variable_layout(
        state=state, descriptor=descriptor, inventory=inventory, radius=radius
    )
    physical = _PhysicalModelBuilder(layout)
    hard = _HardModelBuilder(layout)
    counters = {name: 0 for name in REQUIRED_COUNTERS}
    identity = np.eye(GENERALIZED_WIDTH, dtype=np.float64)
    q_scale = np.asarray(
        [0.01] * 3 + [0.025] * 3 + [0.025] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )
    v_scale = np.asarray(
        [0.25] * 3 + [1.0] * 3 + [2.0] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )

    initial_kinematics = forward_kinematics(model, _configuration_from_state(state, 0))
    for point in inventory.active_points_by_node[0]:
        anchor_index = int(inventory.anchor_index_by_node_point[0, point])
        jacobian = _point_jacobian(model, initial_kinematics, points[point])
        counters["real_contact_jacobian_assemblies"] += 1
        start, scale = physical.begin(
            _point_position(initial_kinematics, points[point]) - anchors[anchor_index],
            phase_scale=0.01,
            acceptance_tolerance=1.0e-6,
            category="initial_anchor_position",
            graph_row=0,
            point=point,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.q_index(0, 0), layout.q_index(0, 0) + GENERALIZED_WIDTH
            ),
            derivative=jacobian,
            row_scale=scale,
        )
        start, scale = physical.begin(
            jacobian @ state.velocity[0],
            phase_scale=0.25,
            acceptance_tolerance=1.0e-6,
            category="initial_sticking_velocity",
            graph_row=0,
            point=point,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.velocity_index(0, 0),
                layout.velocity_index(0, 0) + GENERALIZED_WIDTH,
            ),
            derivative=jacobian,
            row_scale=scale,
        )

    controller = exact_anchor.controller
    for interval in range(PHYSICS_INTERVAL_COUNT):
        left = _configuration_from_state(state, interval)
        right = _configuration_from_state(state, interval + 1)
        mass, left_kinematics = mass_matrix(model, left)
        counters["real_mass_matrix_assemblies"] += 1
        active = inventory.active_points_by_interval[interval]
        next_active = inventory.active_points_by_node[interval + 1]
        jacobians = {
            point: _point_jacobian(model, left_kinematics, points[point])
            for point in active
        }
        counters["real_contact_jacobian_assemblies"] += len(jacobians)
        if active:
            _validate_contact_rank(
                np.vstack([jacobians[point] for point in active]),
                3 if len(active) == 1 else 5,
            )
            counters["real_factorizations"] += 1
        inertial = inverse_dynamics(
            model,
            left,
            state.velocity[interval],
            state.acceleration[interval],
        )
        effort = controller.applied_effort[interval] / 1_000_000.0
        residual = np.array(inertial, copy=True)
        residual[6:] -= effort
        force_terms: list[FloatArray] = []
        for point in active:
            force_row = inventory.force_row_by_interval_point[(interval, point)]
            term = generalized_contact_force(
                jacobians[point], state.active_force[force_row]
            )
            residual -= term
            force_terms.append(term)
        dynamics_scale = _dynamics_scale(inertial, effort, force_terms)
        start, scale = physical.begin(
            residual / dynamics_scale,
            phase_scale=1.0,
            acceptance_tolerance=1.0e-7,
            category="continuous_dynamics",
            graph_row=interval,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.acceleration_index(interval, 0),
                layout.acceleration_index(interval, 0) + GENERALIZED_WIDTH,
            ),
            derivative=mass / dynamics_scale,
            row_scale=scale,
        )
        physical.add_block(
            row_start=start + 6,
            columns=np.arange(
                layout.effort_index(interval, 0),
                layout.effort_index(interval, 0) + ACTUATED_JOINT_COUNT,
            ),
            derivative=(-np.eye(ACTUATED_JOINT_COUNT) / 1_000_000.0) / dynamics_scale,
            row_scale=scale[6:],
        )
        for point in active:
            force_row = inventory.force_row_by_interval_point[(interval, point)]
            derivative = -np.column_stack(
                [
                    generalized_contact_force(
                        jacobians[point],
                        np.eye(POINT_WIDTH, dtype=np.float64)[component],
                    )
                    for component in range(POINT_WIDTH)
                ]
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.force_index(force_row, 0),
                    layout.force_index(force_row, 0) + POINT_WIDTH,
                ),
                derivative=derivative / dynamics_scale,
                row_scale=scale,
            )
        counters["real_dynamics_residual_evaluations"] += 1

        v_bar = state.velocity[interval] + DT * state.acceleration[interval]
        predicted = integrate_configuration(left, v_bar, DT)
        integration = np.concatenate(
            (
                right.root_position - predicted.root_position,
                _rotation_log(right.root_rotation @ predicted.root_rotation.T),
                right.joint_positions - predicted.joint_positions,
            )
        )
        start, scale = physical.begin(
            integration,
            phase_scale=q_scale,
            acceptance_tolerance=1.0e-7,
            category="manifold_integration",
            graph_row=interval,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.q_index(interval + 1, 0),
                layout.q_index(interval + 1, 0) + GENERALIZED_WIDTH,
            ),
            derivative=identity,
            row_scale=scale,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.q_index(interval, 0),
                layout.q_index(interval, 0) + GENERALIZED_WIDTH,
            ),
            derivative=-identity,
            row_scale=scale,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.velocity_index(interval, 0),
                layout.velocity_index(interval, 0) + GENERALIZED_WIDTH,
            ),
            derivative=-DT * identity,
            row_scale=scale,
        )
        physical.add_block(
            row_start=start,
            columns=np.arange(
                layout.acceleration_index(interval, 0),
                layout.acceleration_index(interval, 0) + GENERALIZED_WIDTH,
            ),
            derivative=-(DT * DT) * identity,
            row_scale=scale,
        )
        counters["real_integration_residual_evaluations"] += 1

        activated = tuple(sorted(set(next_active) - set(active)))
        if activated:
            right_mass, right_kinematics = mass_matrix(model, right)
            counters["real_mass_matrix_assemblies"] += 1
            jump = right_mass @ (state.velocity[interval + 1] - v_bar)
            impulse_terms: list[FloatArray] = []
            impulse_jacobians: dict[int, FloatArray] = {}
            for point in activated:
                jacobian = _point_jacobian(model, right_kinematics, points[point])
                impulse_jacobians[point] = jacobian
                counters["real_contact_jacobian_assemblies"] += 1
                impulse_row = inventory.impulse_row_by_node_point[(interval + 1, point)]
                term = generalized_contact_force(
                    jacobian, state.activation_impulse[impulse_row]
                )
                jump -= term
                impulse_terms.append(term)
            jump_scale = max(
                1.0,
                float(np.max(np.abs(right_mass @ state.velocity[interval + 1]))),
                float(np.max(np.abs(right_mass @ v_bar))),
                *(float(np.max(np.abs(term))) for term in impulse_terms),
            )
            start, scale = physical.begin(
                jump / jump_scale,
                phase_scale=1.0,
                acceptance_tolerance=1.0e-7,
                category="activation_momentum_jump",
                graph_row=interval,
            )
            for sign, node in ((1.0, interval + 1), (-1.0, interval)):
                physical.add_block(
                    row_start=start,
                    columns=np.arange(
                        layout.velocity_index(node, 0),
                        layout.velocity_index(node, 0) + GENERALIZED_WIDTH,
                    ),
                    derivative=sign * right_mass / jump_scale,
                    row_scale=scale,
                )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.acceleration_index(interval, 0),
                    layout.acceleration_index(interval, 0) + GENERALIZED_WIDTH,
                ),
                derivative=-DT * right_mass / jump_scale,
                row_scale=scale,
            )
            for point, jacobian in impulse_jacobians.items():
                impulse_row = inventory.impulse_row_by_node_point[(interval + 1, point)]
                derivative = -np.column_stack(
                    [
                        generalized_contact_force(
                            jacobian,
                            np.eye(POINT_WIDTH, dtype=np.float64)[component],
                        )
                        for component in range(POINT_WIDTH)
                    ]
                )
                physical.add_block(
                    row_start=start,
                    columns=np.arange(
                        layout.impulse_index(impulse_row, 0),
                        layout.impulse_index(impulse_row, 0) + POINT_WIDTH,
                    ),
                    derivative=derivative / jump_scale,
                    row_scale=scale,
                )
            counters["real_impulse_residual_evaluations"] += 1
        else:
            right_kinematics = forward_kinematics(model, right)
            start, scale = physical.begin(
                state.velocity[interval + 1] - v_bar,
                phase_scale=v_scale,
                acceptance_tolerance=1.0e-7,
                category="ordinary_velocity_update",
                graph_row=interval,
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.velocity_index(interval + 1, 0),
                    layout.velocity_index(interval + 1, 0) + GENERALIZED_WIDTH,
                ),
                derivative=identity,
                row_scale=scale,
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.velocity_index(interval, 0),
                    layout.velocity_index(interval, 0) + GENERALIZED_WIDTH,
                ),
                derivative=-identity,
                row_scale=scale,
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.acceleration_index(interval, 0),
                    layout.acceleration_index(interval, 0) + GENERALIZED_WIDTH,
                ),
                derivative=-DT * identity,
                row_scale=scale,
            )

        for point in next_active:
            anchor_index = int(
                inventory.anchor_index_by_node_point[interval + 1, point]
            )
            jacobian = _point_jacobian(model, right_kinematics, points[point])
            counters["real_contact_jacobian_assemblies"] += 1
            start, scale = physical.begin(
                _point_position(right_kinematics, points[point])
                - anchors[anchor_index],
                phase_scale=0.01,
                acceptance_tolerance=1.0e-6,
                category="active_anchor_position",
                graph_row=interval,
                point=point,
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.q_index(interval + 1, 0),
                    layout.q_index(interval + 1, 0) + GENERALIZED_WIDTH,
                ),
                derivative=jacobian,
                row_scale=scale,
            )
            start, scale = physical.begin(
                jacobian @ state.velocity[interval + 1],
                phase_scale=0.25,
                acceptance_tolerance=1.0e-6,
                category="active_sticking_velocity",
                graph_row=interval,
                point=point,
            )
            physical.add_block(
                row_start=start,
                columns=np.arange(
                    layout.velocity_index(interval + 1, 0),
                    layout.velocity_index(interval + 1, 0) + GENERALIZED_WIDTH,
                ),
                derivative=jacobian,
                row_scale=scale,
            )

        if active:
            motion = propagate_motion(
                model,
                left_kinematics,
                state.velocity[interval],
                state.acceleration[interval],
            )
            for point in active:
                point_value = point_acceleration(
                    left_kinematics,
                    motion,
                    int(points[point]["body_slot"]),
                    np.asarray(points[point]["local_translation_metres"]),
                )
                start, scale = physical.begin(
                    point_value,
                    phase_scale=5.0,
                    acceptance_tolerance=1.0e-5,
                    category="active_sticking_acceleration",
                    graph_row=interval,
                    point=point,
                )
                physical.add_block(
                    row_start=start,
                    columns=np.arange(
                        layout.acceleration_index(interval, 0),
                        layout.acceleration_index(interval, 0) + GENERALIZED_WIDTH,
                    ),
                    derivative=jacobians[point],
                    row_scale=scale,
                )

    _add_controller_surrogate_rows(
        hard=hard,
        state=state,
        replay=controller,
        descriptor=descriptor,
        layout=layout,
    )
    for column in range(layout.primary_count):
        hard.add_row(
            [(column, 1.0)],
            lower=float(layout.lower[column] * layout.scale[column]),
            upper=float(layout.upper[column] * layout.scale[column]),
            row_scale=float(layout.scale[column]),
        )
    _add_force_cone_rows(
        physical=physical,
        state=state,
        inventory=inventory,
        layout=layout,
        separation_directions=separation_directions,
    )
    physical_matrix, physical_residual, multiplier, upper, addresses = physical.finish()
    hard_matrix, hard_lower, hard_upper = hard.finish()
    if (
        not np.all(np.isfinite(physical_matrix.data))
        or not np.all(np.isfinite(hard_matrix.data))
        or np.any(hard_lower > hard_upper)
    ):
        raise ExecutionInvalid("LOCAL_MODEL_NONFINITE_OR_UNORDERED_BOUND")
    digest = hashlib.sha256()
    for value in (
        physical_matrix.indptr,
        physical_matrix.indices,
        physical_matrix.data,
        physical_residual,
        multiplier,
        upper,
        addresses,
        hard_matrix.indptr,
        hard_matrix.indices,
        hard_matrix.data,
        hard_lower,
        hard_upper,
    ):
        digest.update(np.ascontiguousarray(value).tobytes())
    counters["real_kinodynamic_system_assemblies"] = 1
    return LocalModel(
        layout=layout,
        physical_matrix=physical_matrix,
        physical_residual=physical_residual,
        physical_funnel_multiplier=multiplier,
        physical_upper=upper,
        physical_addresses=addresses,
        hard_matrix=hard_matrix,
        hard_lower=hard_lower,
        hard_upper=hard_upper,
        coefficient_sha256=digest.hexdigest(),
        controller_branch_sha256=controller.audit["controller_branch_address_sha256"],
        controller_branch_ambiguity_count=controller.branch_equality_ambiguity_count,
        counters=counters,
        radius=radius,
    )


def _tracking_quadratic(
    *,
    model: LocalModel,
    state: TrajectoryState,
    reference: TrajectoryState,
    inventory: GraphInventory,
) -> tuple[sparse.csc_matrix, FloatArray]:
    layout = model.layout
    diagonal = np.zeros(layout.base_count, dtype=np.float64)
    linear = np.zeros(layout.base_count, dtype=np.float64)
    off_rows: list[int] = []
    off_columns: list[int] = []
    off_values: list[float] = []
    q_group_scale = np.asarray(
        [0.01] * 3 + [0.025] * 3 + [0.025] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )
    v_group_scale = np.asarray(
        [0.25] * 3 + [1.0] * 3 + [2.0] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )
    a_group_scale = np.asarray(
        [5.0] * 3 + [20.0] * 3 + [20.0] * ACTUATED_JOINT_COUNT,
        dtype=np.float64,
    )

    def square(index: int, coefficient: float, offset: float, weight: float) -> None:
        diagonal[index] += 2.0 * weight * coefficient * coefficient
        linear[index] += 2.0 * weight * offset * coefficient

    q_weight = 1.0 / (STATE_NODE_COUNT * GENERALIZED_WIDTH)
    v_weight = 0.25 / (STATE_NODE_COUNT * GENERALIZED_WIDTH)
    command_weight = 0.5 / (MOTOR_INTERVAL_COUNT * ACTUATED_JOINT_COUNT)
    for node in range(STATE_NODE_COUNT):
        q_error = np.concatenate(
            (
                state.root_position[node] - reference.root_position[node],
                _rotation_log(
                    state.root_rotation[node] @ reference.root_rotation[node].T
                ),
                state.joint_position[node] - reference.joint_position[node],
            )
        )
        terminal = 16.0 if node == STATE_NODE_COUNT - 1 else 1.0
        for component in range(GENERALIZED_WIDTH):
            index = layout.q_index(node, component)
            square(
                index,
                layout.scale[index] / q_group_scale[component],
                float(q_error[component] / q_group_scale[component]),
                q_weight * terminal,
            )
            v_index = layout.velocity_index(node, component)
            square(
                v_index,
                layout.scale[v_index] / v_group_scale[component],
                float(
                    (
                        state.velocity[node, component]
                        - reference.velocity[node, component]
                    )
                    / v_group_scale[component]
                ),
                v_weight * terminal,
            )
    for interval in range(MOTOR_INTERVAL_COUNT):
        for dof in range(ACTUATED_JOINT_COUNT):
            index = layout.command_index(interval, dof)
            square(
                index,
                layout.scale[index] / 25_000.0,
                float(
                    state.integer_command[interval, dof]
                    - reference.integer_command[interval, dof]
                )
                / 25_000.0,
                command_weight,
            )

    def difference(
        left: int,
        right: int,
        *,
        left_value: float,
        right_value: float,
        denominator: float,
        weight: float,
    ) -> None:
        left_coefficient = -layout.scale[left] / denominator
        right_coefficient = layout.scale[right] / denominator
        offset = (right_value - left_value) / denominator
        diagonal[left] += 2.0 * weight * left_coefficient * left_coefficient
        diagonal[right] += 2.0 * weight * right_coefficient * right_coefficient
        linear[left] += 2.0 * weight * offset * left_coefficient
        linear[right] += 2.0 * weight * offset * right_coefficient
        off_rows.append(min(left, right))
        off_columns.append(max(left, right))
        off_values.append(2.0 * weight * left_coefficient * right_coefficient)

    acceleration_weight = 0.01 / ((PHYSICS_INTERVAL_COUNT - 1) * GENERALIZED_WIDTH)
    for interval in range(1, PHYSICS_INTERVAL_COUNT):
        for component in range(GENERALIZED_WIDTH):
            left = layout.acceleration_index(interval - 1, component)
            right = layout.acceleration_index(interval, component)
            difference(
                left,
                right,
                left_value=float(state.acceleration[interval - 1, component]),
                right_value=float(state.acceleration[interval, component]),
                denominator=float(a_group_scale[component]),
                weight=acceleration_weight,
            )

    force_weight = 0.0001 / max(len(inventory.force_rows) * POINT_WIDTH, 1)
    for row in range(len(inventory.force_rows)):
        for component in range(POINT_WIDTH):
            index = layout.force_index(row, component)
            square(
                index,
                layout.scale[index] / 250.0,
                float(state.active_force[row, component] / 250.0),
                force_weight,
            )
    force_difference_weight = 0.0001 / max(
        (len(inventory.force_rows) - 1) * POINT_WIDTH, 1
    )
    for row in range(1, len(inventory.force_rows)):
        previous_address = inventory.force_rows[row - 1]
        current_address = inventory.force_rows[row]
        if previous_address[1] != current_address[1]:
            continue
        for component in range(POINT_WIDTH):
            left = layout.force_index(row - 1, component)
            right = layout.force_index(row, component)
            difference(
                left,
                right,
                left_value=float(state.active_force[row - 1, component]),
                right_value=float(state.active_force[row, component]),
                denominator=250.0,
                weight=force_difference_weight,
            )
    impulse_weight = 0.0001 / max(len(inventory.impulse_rows) * POINT_WIDTH, 1)
    for row in range(len(inventory.impulse_rows)):
        for component in range(POINT_WIDTH):
            index = layout.impulse_index(row, component)
            square(
                index,
                layout.scale[index] / 25.0,
                float(state.activation_impulse[row, component] / 25.0),
                impulse_weight,
            )

    rows = np.asarray(off_rows, dtype=np.int64)
    columns = np.asarray(off_columns, dtype=np.int64)
    values = np.asarray(off_values, dtype=np.float64)
    quadratic = sparse.diags(diagonal, format="csc")
    if len(values):
        quadratic = (
            quadratic
            + sparse.coo_matrix(
                (values, (rows, columns)),
                shape=(layout.base_count, layout.base_count),
            ).tocsc()
        )
    return sparse.triu(quadratic, format="csc"), linear


def _qp_identity(
    *,
    quadratic: sparse.csc_matrix,
    linear: FloatArray,
    matrix: sparse.csc_matrix,
    lower: FloatArray,
    upper: FloatArray,
) -> str:
    digest = hashlib.sha256()
    for value in (
        quadratic.indptr,
        quadratic.indices,
        quadratic.data,
        linear,
        matrix.indptr,
        matrix.indices,
        matrix.data,
        lower,
        upper,
    ):
        digest.update(np.ascontiguousarray(value).tobytes())
    return digest.hexdigest()


def _solve_osqp_pass(
    *,
    ordinal: int,
    quadratic: sparse.csc_matrix,
    linear: FloatArray,
    matrix: sparse.csc_matrix,
    lower: FloatArray,
    upper: FloatArray,
    base_count: int,
    physical_matrix: sparse.csc_matrix,
    physical_residual: FloatArray,
    physical_upper: BoolArray,
) -> QpPassResult:
    if (
        ordinal not in (1, 2, 3)
        or quadratic.shape != (len(linear), len(linear))
        or matrix.shape != (len(lower), len(linear))
        or len(lower) != len(upper)
        or np.any(lower > upper)
        or not np.all(np.isfinite(lower) | np.isneginf(lower))
        or not np.all(np.isfinite(upper) | np.isposinf(upper))
        or not np.all(np.isfinite(quadratic.data))
        or not np.all(np.isfinite(linear))
        or not np.all(np.isfinite(matrix.data))
        or physical_matrix.shape != (len(physical_residual), base_count)
        or physical_upper.shape != physical_residual.shape
        or physical_upper.dtype != np.bool_
        or not np.all(np.isfinite(physical_matrix.data))
        or not np.all(np.isfinite(physical_residual))
    ):
        raise ExecutionInvalid("QP_SCHEMA_NONFINITE_OR_SHAPE_DIFFERS")
    solver = osqp.OSQP()
    try:
        solver.setup(
            P=quadratic,
            q=linear,
            A=matrix,
            l=lower,
            u=upper,
            verbose=False,
            eps_abs=1.0e-5,
            eps_rel=1.0e-5,
            max_iter=100_000,
            polishing=True,
            adaptive_rho=True,
        )
        result = solver.solve(raise_error=False)
    except Exception as error:
        raise ExecutionInvalid(
            f"OSQP_SETUP_OR_SOLVE_EXCEPTION_{type(error).__name__}"
        ) from error
    status = str(result.info.status).lower()
    primal = float(result.info.prim_res)
    dual = float(result.info.dual_res)
    if (
        status != "solved"
        or result.x is None
        or not np.all(np.isfinite(result.x))
        or not math.isfinite(primal)
        or not math.isfinite(dual)
        or primal < 0.0
        or dual < 0.0
        or primal > 5.0e-5
        or dual > 5.0e-5
    ):
        raise ExecutionInvalid("OSQP_STATUS_OR_RESIDUAL_AMBIGUITY")
    solution = np.asarray(result.x, dtype=np.float64)
    physical = physical_residual + physical_matrix @ solution[:base_count]
    elastic = np.where(physical_upper, np.maximum(physical, 0.0), np.abs(physical))
    maximum = float(np.max(elastic, initial=0.0))
    mean = float(np.mean(elastic)) if len(elastic) else 0.0
    return QpPassResult(
        ordinal=ordinal,
        status=status,
        iterations=int(result.info.iter),
        primal_residual=primal,
        dual_residual=dual,
        objective=float(result.info.obj_val),
        maximum_elastic=maximum,
        mean_elastic=mean,
        solution=solution,
        variable_count=len(linear),
        constraint_count=matrix.shape[0],
        nonzero_count=matrix.nnz,
        problem_sha256=_qp_identity(
            quadratic=quadratic,
            linear=linear,
            matrix=matrix,
            lower=lower,
            upper=upper,
        ),
    )


def solve_lexicographic_qps(
    *,
    model: LocalModel,
    state: TrajectoryState,
    reference: TrajectoryState,
    inventory: GraphInventory,
    counters: dict[str, int],
) -> MajorStep:
    base_count = model.layout.base_count
    physical_count = len(model.physical_residual)
    physical = model.physical_matrix
    equality_rows = np.flatnonzero(~model.physical_upper)
    upper_rows = np.flatnonzero(model.physical_upper)
    equality = physical[equality_rows].tocsc()
    upper_inequality = physical[upper_rows].tocsc()
    equality_residual = model.physical_residual[equality_rows]
    upper_residual = model.physical_residual[upper_rows]
    equality_maximum = sparse.csc_matrix(
        -np.ones((len(equality_rows), 1), dtype=np.float64)
    )
    upper_maximum = sparse.csc_matrix(-np.ones((len(upper_rows), 1), dtype=np.float64))
    zero_hard_column = sparse.csc_matrix((model.hard_matrix.shape[0], 1))
    first_matrix = sparse.vstack(
        (
            sparse.hstack((equality, equality_maximum), format="csc"),
            sparse.hstack((-equality, equality_maximum), format="csc"),
            sparse.hstack((upper_inequality, upper_maximum), format="csc"),
            sparse.hstack((model.hard_matrix, zero_hard_column), format="csc"),
            sparse.csc_matrix(
                (
                    np.asarray([1.0]),
                    (np.asarray([0]), np.asarray([base_count])),
                ),
                shape=(1, base_count + 1),
            ),
        ),
        format="csc",
    )
    first_lower = np.concatenate(
        (
            np.full(len(equality_rows), -np.inf),
            np.full(len(equality_rows), -np.inf),
            np.full(len(upper_rows), -np.inf),
            model.hard_lower,
            np.asarray([0.0]),
        )
    )
    first_upper = np.concatenate(
        (
            -equality_residual,
            equality_residual,
            -upper_residual,
            model.hard_upper,
            np.asarray([np.inf]),
        )
    )
    first_linear = np.zeros(base_count + 1, dtype=np.float64)
    first_linear[-1] = 1.0
    counters["real_qp_setups"] += 1
    counters["real_qp_solves"] += 1
    counters["real_factorizations"] += 1
    first = _solve_osqp_pass(
        ordinal=1,
        quadratic=sparse.csc_matrix((base_count + 1, base_count + 1)),
        linear=first_linear,
        matrix=first_matrix,
        lower=first_lower,
        upper=first_upper,
        base_count=base_count,
        physical_matrix=physical,
        physical_residual=model.physical_residual,
        physical_upper=model.physical_upper,
    )
    maximum_hold = first.maximum_elastic + 1.0e-8

    negative_identity = -sparse.eye(physical_count, format="csc")
    equality_elastic = negative_identity[equality_rows].tocsc()
    upper_elastic = negative_identity[upper_rows].tocsc()
    zero_hard_elastic = sparse.csc_matrix((model.hard_matrix.shape[0], physical_count))
    elastic_identity = sparse.eye(physical_count, format="csc")
    second_matrix = sparse.vstack(
        (
            sparse.hstack((equality, equality_elastic), format="csc"),
            sparse.hstack((-equality, equality_elastic), format="csc"),
            sparse.hstack((upper_inequality, upper_elastic), format="csc"),
            sparse.hstack((model.hard_matrix, zero_hard_elastic), format="csc"),
            sparse.hstack(
                (
                    sparse.csc_matrix((physical_count, base_count)),
                    elastic_identity,
                ),
                format="csc",
            ),
        ),
        format="csc",
    )
    second_lower = np.concatenate(
        (
            np.full(len(equality_rows), -np.inf),
            np.full(len(equality_rows), -np.inf),
            np.full(len(upper_rows), -np.inf),
            model.hard_lower,
            np.zeros(physical_count),
        )
    )
    second_upper = np.concatenate(
        (
            -equality_residual,
            equality_residual,
            -upper_residual,
            model.hard_upper,
            np.full(physical_count, maximum_hold),
        )
    )
    second_linear = np.zeros(base_count + physical_count, dtype=np.float64)
    second_linear[base_count:] = 1.0 / max(physical_count, 1)
    counters["real_qp_setups"] += 1
    counters["real_qp_solves"] += 1
    counters["real_factorizations"] += 1
    second = _solve_osqp_pass(
        ordinal=2,
        quadratic=sparse.csc_matrix(
            (base_count + physical_count, base_count + physical_count)
        ),
        linear=second_linear,
        matrix=second_matrix,
        lower=second_lower,
        upper=second_upper,
        base_count=base_count,
        physical_matrix=physical,
        physical_residual=model.physical_residual,
        physical_upper=model.physical_upper,
    )
    mean_hold = second.mean_elastic + 1.0e-8

    mean_row = sparse.hstack(
        (
            sparse.csc_matrix((1, base_count)),
            sparse.csc_matrix(np.ones((1, physical_count), dtype=np.float64)),
        ),
        format="csc",
    )
    third_matrix = sparse.vstack((second_matrix, mean_row), format="csc")
    third_lower = np.concatenate((second_lower, np.asarray([-np.inf])))
    third_upper = np.concatenate(
        (second_upper, np.asarray([mean_hold * physical_count]))
    )
    base_quadratic, base_linear = _tracking_quadratic(
        model=model, state=state, reference=reference, inventory=inventory
    )
    third_quadratic = sparse.block_diag(
        (
            base_quadratic,
            sparse.csc_matrix((physical_count, physical_count)),
        ),
        format="csc",
    )
    third_linear = np.concatenate((base_linear, np.zeros(physical_count)))
    counters["real_qp_setups"] += 1
    counters["real_qp_solves"] += 1
    counters["real_factorizations"] += 1
    third = _solve_osqp_pass(
        ordinal=3,
        quadratic=third_quadratic,
        linear=third_linear,
        matrix=third_matrix,
        lower=third_lower,
        upper=third_upper,
        base_count=base_count,
        physical_matrix=physical,
        physical_residual=model.physical_residual,
        physical_upper=model.physical_upper,
    )
    step = third.solution[:base_count]
    model_residual = model.physical_residual + physical @ step
    elastic = np.where(
        model.physical_upper,
        np.maximum(model_residual, 0.0),
        np.abs(model_residual),
    )
    normalized = elastic * model.physical_funnel_multiplier
    return MajorStep(
        step=step,
        passes=(first, second, third),
        model_maximum=float(np.max(normalized, initial=0.0)),
        model_mean=float(np.mean(normalized)) if len(normalized) else 0.0,
        active_trust_boundary=bool(
            np.max(np.abs(step[: model.layout.primary_count]), initial=0.0)
            >= 0.999999 * model.radius
        ),
    )


def apply_trial_step(
    *,
    state: TrajectoryState,
    model: LocalModel,
    step: FloatArray,
    fraction: float,
) -> TrajectoryState:
    if (
        step.shape != (model.layout.base_count,)
        or not np.all(np.isfinite(step))
        or fraction not in (1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625)
    ):
        raise ExecutionInvalid("TRIAL_STEP_OR_FRACTION_DIFFERS")
    layout = model.layout
    physical = fraction * step * layout.scale
    q_delta = physical[layout.q].reshape((STATE_NODE_COUNT, GENERALIZED_WIDTH))
    root_position = state.root_position + q_delta[:, :3]
    root_rotation = np.empty_like(state.root_rotation)
    for node in range(STATE_NODE_COUNT):
        root_rotation[node] = (
            rotation_exp(q_delta[node, 3:6]) @ state.root_rotation[node]
        )
    joint_position = state.joint_position + q_delta[:, 6:]
    velocity = state.velocity + physical[layout.velocity].reshape(
        (STATE_NODE_COUNT, GENERALIZED_WIDTH)
    )
    acceleration = state.acceleration + physical[layout.acceleration].reshape(
        (PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH)
    )
    floating_command = state.integer_command.astype(np.float64) + physical[
        layout.command
    ].reshape((MOTOR_INTERVAL_COUNT, ACTUATED_JOINT_COUNT))
    if np.any(floating_command < np.iinfo(np.int64).min) or np.any(
        floating_command > np.iinfo(np.int64).max
    ):
        raise ExecutionInvalid("TRIAL_INTEGER_COMMAND_OVERFLOW")
    integer_command = np.rint(floating_command).astype(np.int64)
    active_force = state.active_force + physical[layout.force].reshape(
        state.active_force.shape
    )
    activation_impulse = state.activation_impulse + physical[layout.impulse].reshape(
        state.activation_impulse.shape
    )
    return TrajectoryState(
        root_position=np.asarray(root_position, dtype=np.float64),
        root_rotation=root_rotation,
        joint_position=np.asarray(joint_position, dtype=np.float64),
        velocity=np.asarray(velocity, dtype=np.float64),
        acceleration=np.asarray(acceleration, dtype=np.float64),
        integer_command=integer_command,
        active_force=np.asarray(active_force, dtype=np.float64),
        activation_impulse=np.asarray(activation_impulse, dtype=np.float64),
    )


def accepted_anchor_hashes(
    *, state: TrajectoryState, audit: ExactAudit
) -> dict[str, Any]:
    if (
        audit.status == "INVALID"
        or audit.controller is None
        or audit.funnel is None
        or audit.residual_vector is None
    ):
        raise ExecutionInvalid("ACCEPTED_ANCHOR_HASH_INPUT_DIFFERS")
    q_hash = _aggregate_array_identity(
        {
            "root_position": state.root_position,
            "root_rotation": state.root_rotation,
            "joint_position": state.joint_position,
        }
    )
    branch_hash = _aggregate_array_identity(
        {
            "target": audit.controller.target_branch_addresses,
            "effort": audit.controller.branch_addresses,
        }
    )
    values = {
        "exact_q_array": q_hash,
        "exact_v_array": _array_sha256(state.velocity),
        "integer_command_array": _array_sha256(state.integer_command),
        "exact_applied_target_array": _array_sha256(audit.controller.applied_target),
        "exact_applied_effort_array": _array_sha256(audit.controller.applied_effort),
        "controller_event_address_array": _array_sha256(
            audit.controller.event_addresses
        ),
        "controller_branch_address_array": branch_hash,
        "exact_funnel_vector": _array_sha256(
            np.asarray(audit.funnel, dtype=np.float64)
        ),
    }
    return values


def _add_counters(target: dict[str, int], values: Mapping[str, int]) -> None:
    for name in REQUIRED_COUNTERS:
        target[name] += int(values.get(name, 0))


def _linux_thread_count() -> int:
    directory = Path(f"/proc/{os.getpid()}/task")
    if not directory.is_dir():
        raise ExecutionInvalid("LINUX_PROC_THREAD_ACCOUNTING_ABSENT")
    return sum(1 for child in directory.iterdir() if child.name.isdigit())


def _maximum_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _resource_stop_reason(started: float) -> str | None:
    if _linux_thread_count() != 1:
        raise ExecutionInvalid("OBSERVED_THREAD_COUNT_DIFFERS")
    if _maximum_rss_bytes() > 17_179_869_184:
        return "PEAK_RSS_BUDGET_EXHAUSTED"
    if time.monotonic() - started > 14_400.0:
        return "WALL_TIME_BUDGET_EXHAUSTED"
    return None


def _model_funnel_at_fraction(
    model: LocalModel, step: FloatArray, fraction: float
) -> tuple[float, float]:
    residual = model.physical_residual + model.physical_matrix @ (fraction * step)
    elastic = np.where(
        model.physical_upper, np.maximum(residual, 0.0), np.abs(residual)
    )
    violation = elastic * model.physical_funnel_multiplier
    return (
        float(np.max(violation, initial=0.0)),
        float(np.mean(violation)) if len(violation) else 0.0,
    )


def _actual_to_model_ratio(
    *,
    current: tuple[float, float],
    trial: tuple[float, float],
    predicted: tuple[float, float],
) -> float:
    maximum_actual = current[0] - trial[0]
    maximum_model = current[0] - predicted[0]
    if maximum_model > 0.0:
        return maximum_actual / maximum_model
    mean_actual = current[1] - trial[1]
    mean_model = current[1] - predicted[1]
    return mean_actual / mean_model if mean_model > 0.0 else 0.0


def _update_separation_directions(
    target: dict[tuple[str, int], list[tuple[float, float]]],
    violations: Sequence[Mapping[str, Any]],
) -> list[dict[str, Any]]:
    cone_order = {"continuous_force": 0, "activation_impulse": 1}
    rows = sorted(
        violations,
        key=lambda row: (
            int(row["graph_row"]),
            cone_order[str(row["cone_type"])],
            int(row["point_ordinal"]),
        ),
    )
    additions: list[dict[str, Any]] = []
    seen: set[tuple[str, int]] = set()
    for row in rows:
        cone_type = str(row["cone_type"])
        cone_row = int(row["row"])
        key = (cone_type, cone_row)
        if key in seen:
            continue
        seen.add(key)
        right = float(row["right"])
        forward = float(row["forward"])
        norm = math.hypot(right, forward)
        direction = (1.0, 0.0) if norm == 0.0 else (right / norm, forward / norm)
        existing = target.setdefault(key, [])
        if not any(
            abs(direction[0] - value[0]) <= 1.0e-15
            and abs(direction[1] - value[1]) <= 1.0e-15
            for value in existing
        ):
            existing.append(direction)
            additions.append(
                {
                    "graph_row": int(row["graph_row"]),
                    "cone_type": cone_type,
                    "point_ordinal": int(row["point_ordinal"]),
                    "cone_row": cone_row,
                    "right_float64_hex": direction[0].hex(),
                    "forward_float64_hex": direction[1].hex(),
                }
            )
    return additions


def solve_fixed_mode_kinodynamic_problem(
    *,
    reference: ReferenceReconstruction,
    source_gate: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    model: SpatialModel,
    points: tuple[dict[str, Any], ...],
    inventory: GraphInventory,
    started: float,
) -> ExecutionOutcome:
    counters = {name: 0 for name in REQUIRED_COUNTERS}
    counters["real_state_reconstructions"] = 1
    counters["real_state_lift_evaluations"] = STATE_NODE_COUNT
    counters["real_controller_schedule_derivations"] = 2
    counters["real_mass_matrix_assemblies"] = reference.projection_systems + 1
    counters["real_contact_jacobian_assemblies"] = sum(
        len(active) for active in inventory.active_points_by_interval
    ) + len(inventory.active_points_by_node[-1])
    counters["real_factorizations"] = reference.projection_factorizations + 1
    counters["real_kinodynamic_solves"] = 1
    history: list[dict[str, Any]] = []
    anchor_hash_rows: list[dict[str, Any]] = []
    separation: dict[tuple[str, int], list[tuple[float, float]]] = {}
    state = reference.state
    radius = 1.0
    maximum_thread_count = _linux_thread_count()

    def resource_usage(*, invalid_reason: str | None) -> dict[str, Any]:
        elapsed = time.monotonic() - started
        return {
            "status": "PASS" if invalid_reason is None else "INVALID",
            "invalid_reason": invalid_reason,
            "wall_clock_seconds": elapsed,
            "maximum_resident_memory_bytes": _maximum_rss_bytes(),
            "maximum_observed_thread_count": maximum_thread_count,
            "execution_process_count": 1,
            "random_seed": 0,
            "randomized_restart_count": 0,
            "resume_or_warm_restart": "FORBIDDEN_AND_NOT_USED",
            "manual_intervention": "FORBIDDEN_AND_NOT_USED",
        }

    current_audit = exact_graph_audit(
        state=state,
        immutable_initial=reference.state,
        anchors=reference.anchors,
        descriptor=descriptor,
        model=model,
        points=points,
        inventory=inventory,
    )
    _add_counters(counters, current_audit.counters)
    if current_audit.status == "INVALID":
        return ExecutionOutcome(
            status="INVALID",
            feasibility="INVALID",
            termination="INITIAL_EXACT_AUDIT_INVALID",
            invalid_reason=current_audit.invalid_reason,
            final_state=None,
            final_audit=current_audit,
            anchors=None,
            history=tuple(history),
            accepted_anchor_hashes=(),
            source_gate=dict(source_gate),
            counters=counters,
            resource_usage=resource_usage(invalid_reason=current_audit.invalid_reason),
        )
    anchor_hash_rows.append(
        {"anchor_number": 0, **accepted_anchor_hashes(state=state, audit=current_audit)}
    )
    history.append(
        {
            "major_iteration": 0,
            "event": "INITIAL_EXACT_ANCHOR",
            "status": current_audit.status,
            "funnel": list(current_audit.funnel or ()),
        }
    )
    if current_audit.hard_pass:
        return ExecutionOutcome(
            status="PASS",
            feasibility="FEASIBLE",
            termination="INITIAL_EXACT_HARD_PASS",
            invalid_reason=None,
            final_state=state,
            final_audit=current_audit,
            anchors=reference.anchors,
            history=tuple(history),
            accepted_anchor_hashes=tuple(anchor_hash_rows),
            source_gate=dict(source_gate),
            counters=counters,
            resource_usage=resource_usage(invalid_reason=None),
        )

    trial_fractions = (1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625)
    termination = "MAJOR_ITERATION_BUDGET_EXHAUSTED"
    for major in range(1, 7):
        try:
            stop_reason = _resource_stop_reason(started)
            maximum_thread_count = max(maximum_thread_count, _linux_thread_count())
            if stop_reason is not None:
                termination = stop_reason
                break
            local = assemble_local_model(
                state=state,
                exact_anchor=current_audit,
                anchors=reference.anchors,
                descriptor=descriptor,
                model=model,
                points=points,
                inventory=inventory,
                radius=radius,
                separation_directions=separation,
            )
            _add_counters(counters, local.counters)
            major_step = solve_lexicographic_qps(
                model=local,
                state=state,
                reference=reference.state,
                inventory=inventory,
                counters=counters,
            )
            pass_rows = [
                {
                    "pass": row.ordinal,
                    "status": row.status,
                    "iterations": row.iterations,
                    "primal_residual": row.primal_residual,
                    "dual_residual": row.dual_residual,
                    "objective": row.objective,
                    "maximum_elastic": row.maximum_elastic,
                    "mean_elastic": row.mean_elastic,
                    "variable_count": row.variable_count,
                    "constraint_count": row.constraint_count,
                    "nonzero_count": row.nonzero_count,
                    "problem_sha256": row.problem_sha256,
                }
                for row in major_step.passes
            ]
            accepted = False
            trial_rows: list[dict[str, Any]] = []
            for fraction in trial_fractions:
                if counters["real_exact_trial_audits"] >= 42:
                    termination = "EXACT_TRIAL_AUDIT_BUDGET_EXHAUSTED"
                    break
                stop_reason = _resource_stop_reason(started)
                maximum_thread_count = max(maximum_thread_count, _linux_thread_count())
                if stop_reason is not None:
                    termination = stop_reason
                    break
                trial_state = apply_trial_step(
                    state=state,
                    model=local,
                    step=major_step.step,
                    fraction=fraction,
                )
                trial_audit = exact_graph_audit(
                    state=trial_state,
                    immutable_initial=reference.state,
                    anchors=reference.anchors,
                    descriptor=descriptor,
                    model=model,
                    points=points,
                    inventory=inventory,
                )
                counters["real_exact_trial_audits"] += 1
                _add_counters(counters, trial_audit.counters)
                if trial_audit.status == "INVALID":
                    return ExecutionOutcome(
                        status="INVALID",
                        feasibility="INVALID",
                        termination="EXACT_TRIAL_AUDIT_INVALID",
                        invalid_reason=trial_audit.invalid_reason,
                        final_state=None,
                        final_audit=trial_audit,
                        anchors=None,
                        history=(
                            *history,
                            {
                                "major_iteration": major,
                                "radius": radius,
                                "qp_passes": pass_rows,
                                "trial_fraction": fraction,
                                "invalid_reason": trial_audit.invalid_reason,
                            },
                        ),
                        accepted_anchor_hashes=tuple(anchor_hash_rows),
                        source_gate=dict(source_gate),
                        counters=counters,
                        resource_usage=resource_usage(
                            invalid_reason=trial_audit.invalid_reason
                        ),
                    )
                assert current_audit.funnel is not None
                assert trial_audit.funnel is not None
                predicted = _model_funnel_at_fraction(local, major_step.step, fraction)
                maximum_decrease = current_audit.funnel[0] - trial_audit.funnel[0]
                mean_decrease = current_audit.funnel[1] - trial_audit.funnel[1]
                ratio = _actual_to_model_ratio(
                    current=current_audit.funnel,
                    trial=trial_audit.funnel,
                    predicted=predicted,
                )
                lexicographic = maximum_decrease >= 1.0e-6 or (
                    maximum_decrease >= 0.0 and mean_decrease >= 1.0e-6
                )
                eligible = bool(
                    trial_audit.hard_pass or (lexicographic and ratio >= 0.1)
                )
                trial_row = {
                    "fraction": fraction,
                    "status": trial_audit.status,
                    "exact_funnel": list(trial_audit.funnel),
                    "model_funnel": list(predicted),
                    "exact_maximum_decrease": maximum_decrease,
                    "exact_mean_decrease": mean_decrease,
                    "actual_to_model_ratio": ratio,
                    "integer_command_changed_scalars": int(
                        np.count_nonzero(
                            trial_state.integer_command - state.integer_command
                        )
                    ),
                    "eligible": eligible,
                }
                trial_rows.append(trial_row)
                if not eligible:
                    continue
                outcome = decide_major_outcome(
                    radius=str(radius),
                    exact_pass=trial_audit.hard_pass,
                    exact_maximum_decrease=str(maximum_decrease),
                    exact_mean_decrease=str(mean_decrease),
                    actual_to_model_ratio=str(ratio),
                    active_trust_boundary=major_step.active_trust_boundary,
                )
                if outcome["restore_previous_exact_anchor"]:
                    continue
                state = trial_state
                current_audit = trial_audit
                counters["optimizer_steps"] += 1
                anchor_hash_rows.append(
                    {
                        "anchor_number": len(anchor_hash_rows),
                        "major_iteration": major,
                        "trial_fraction": fraction,
                        **accepted_anchor_hashes(state=state, audit=current_audit),
                    }
                )
                additions = _update_separation_directions(
                    separation, current_audit.violated_cones
                )
                radius = float(outcome["next_radius"])
                trial_row["decision"] = outcome["decision"]
                trial_row["separation_directions_added"] = additions
                accepted = True
                history.append(
                    {
                        "major_iteration": major,
                        "radius_before": local.radius,
                        "radius_after": radius,
                        "local_model_sha256": local.coefficient_sha256,
                        "controller_branch_sha256": local.controller_branch_sha256,
                        "controller_branch_ambiguity_count": local.controller_branch_ambiguity_count,
                        "qp_passes": pass_rows,
                        "trials": trial_rows,
                        "decision": outcome["decision"],
                    }
                )
                if current_audit.hard_pass:
                    return ExecutionOutcome(
                        status="PASS",
                        feasibility="FEASIBLE",
                        termination="EXACT_TRIAL_HARD_PASS",
                        invalid_reason=None,
                        final_state=state,
                        final_audit=current_audit,
                        anchors=reference.anchors,
                        history=tuple(history),
                        accepted_anchor_hashes=tuple(anchor_hash_rows),
                        source_gate=dict(source_gate),
                        counters=counters,
                        resource_usage=resource_usage(invalid_reason=None),
                    )
                break
            if termination != "MAJOR_ITERATION_BUDGET_EXHAUSTED":
                break
            if accepted:
                continue
            if radius == 0.03125:
                termination = "MINIMUM_RADIUS_NO_ACCEPTED_TRIAL"
                history.append(
                    {
                        "major_iteration": major,
                        "radius_before": radius,
                        "radius_after": radius,
                        "local_model_sha256": local.coefficient_sha256,
                        "controller_branch_sha256": local.controller_branch_sha256,
                        "controller_branch_ambiguity_count": local.controller_branch_ambiguity_count,
                        "qp_passes": pass_rows,
                        "trials": trial_rows,
                        "decision": "STOP_AND_RESEARCH_AT_MINIMUM_RADIUS",
                    }
                )
                break
            next_radius = max(0.03125, radius * 0.5)
            history.append(
                {
                    "major_iteration": major,
                    "radius_before": radius,
                    "radius_after": next_radius,
                    "local_model_sha256": local.coefficient_sha256,
                    "controller_branch_sha256": local.controller_branch_sha256,
                    "controller_branch_ambiguity_count": local.controller_branch_ambiguity_count,
                    "qp_passes": pass_rows,
                    "trials": trial_rows,
                    "decision": "REJECT_RESTORE_AND_CONTRACT",
                }
            )
            radius = next_radius
        except Exception as error:
            invalid_reason = _stable_invalid_reason(
                error, prefix="MAJOR_ITERATION_EXCEPTION"
            )
            return ExecutionOutcome(
                status="INVALID",
                feasibility="INVALID",
                termination="NUMERIC_OR_SOLVER_INVALID",
                invalid_reason=invalid_reason,
                final_state=None,
                final_audit=current_audit,
                anchors=None,
                history=tuple(history),
                accepted_anchor_hashes=tuple(anchor_hash_rows),
                source_gate=dict(source_gate),
                counters=counters,
                resource_usage=resource_usage(invalid_reason=invalid_reason),
            )

    return ExecutionOutcome(
        status="COMPLETE",
        feasibility="UNRESOLVED_BOUNDED_STOP",
        termination=termination,
        invalid_reason=None,
        final_state=state,
        final_audit=current_audit,
        anchors=reference.anchors,
        history=tuple(history),
        accepted_anchor_hashes=tuple(anchor_hash_rows),
        source_gate=dict(source_gate),
        counters=counters,
        resource_usage=resource_usage(invalid_reason=None),
    )


def _load_bound_report(
    path: Path, expected: Mapping[str, Any], label: str
) -> dict[str, Any]:
    if sha256(path) != expected["report_file_sha256"]:
        raise ExecutionInvalid(f"{label}_REPORT_FILE_SHA256_DIFFERS")
    report = json.loads(path.read_bytes())
    canonical = dict(report)
    claimed = canonical.pop("report_sha256", None)
    actual = hashlib.sha256(canonical_json(canonical)).hexdigest()
    if claimed != actual or actual != expected["report_sha256"]:
        raise ExecutionInvalid(f"{label}_CANONICAL_REPORT_SHA256_DIFFERS")
    return report


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if (
        not isinstance(repository.get("commit"), str)
        or len(repository["commit"]) != 40
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R141 requires one clean repository identity")


def _validate_execution_environment(
    profile: Mapping[str, Any], environment: Mapping[str, str]
) -> None:
    expected = profile["execution_environment"]
    if dict(environment) != expected or any(
        value != "1" for value in expected.values()
    ):
        raise ValueError("R141 single-thread environment differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R141 validation result differs")
    return [dict(row) for row in results]


def _validate_profile(profile: Mapping[str, Any]) -> None:
    budget = profile.get("execution_budget", {})
    scope = profile.get("scope", {})
    decisions = profile.get("result_transitions", {})
    gate_decisions = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    numeric_stack = profile.get("numeric_stack", {})
    source = profile.get("source", {})
    expected_bounded_keys = {
        "r142_report_only_decision",
        "r141_retry",
        "r136_retry",
        "r137_retry",
        "r138_retry",
        "r139_retry",
        "r140_retry",
        "additional_kinodynamic_solve",
        "contact_semantics_change",
        "candidate_artifact",
        "physx",
        "all_17",
        "full_v19",
        "training",
    }
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenAuthorizedExecution"
        or profile.get("claim") != "SingleBoundedFixedModeKinodynamicExecutionOnly"
        or scope.get("run_id") != "R141"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("motor_intervals") != MOTOR_INTERVAL_COUNT
        or scope.get("physics_intervals") != PHYSICS_INTERVAL_COUNT
        or scope.get("state_nodes") != STATE_NODE_COUNT
        or scope.get("fixed_contact_modes") is not True
        or scope.get("candidate_construction") is not False
        or scope.get("physx_scene_runs") != 0
        or scope.get("training_runs") != 0
        or budget.get("clean_processes") != 1
        or budget.get("observed_threads") != 1
        or budget.get("seed") != 0
        or budget.get("major_iteration_limit") != 6
        or budget.get("lexicographic_qp_passes_per_major_iteration") != 3
        or budget.get("qp_solve_limit") != 18
        or budget.get("trial_fractions_per_major_iteration") != 7
        or budget.get("exact_trial_audit_limit") != 42
        or budget.get("osqp_iteration_limit_per_qp") != 100_000
        or budget.get("wall_time_seconds") != 14_400
        or budget.get("peak_rss_bytes") != 17_179_869_184
        or budget.get("restart") != "FORBIDDEN"
        or budget.get("randomized_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
        or budget.get("r136_cache_load") != "FORBIDDEN"
        or numeric_stack != {"numpy": "2.5.2", "scipy": "1.18.0", "osqp": "1.1.3"}
        or np.__version__ != numeric_stack.get("numpy")
        or scipy.__version__ != numeric_stack.get("scipy")
        or osqp.__version__ != numeric_stack.get("osqp")
        or decisions.get("pass") != "R141_PASS_R142_DECISION_ONLY"
        or decisions.get("bounded_stop") != "R141_VALID_BOUNDED_STOP_RESEARCH_REQUIRED"
        or decisions.get("invalid") != "R141_INVALID_STOP_WITHOUT_RETRY"
        or gate_decisions
        != {
            "pass": "PERMIT_SEPARATE_REPORT_ONLY_R142_CANDIDATE_FORMULATION_DECISION_ONLY",
            "bounded_stop": "STOP_VALID_R141_BOUNDED_KINODYNAMIC_RESEARCH",
            "invalid": "STOP_INVALID_R141_WITHOUT_RETRY",
        }
        or set(bounded) != expected_bounded_keys
        or bounded.get("r142_report_only_decision")
        != "AUTHORIZED_DECISION_ONLY_ON_EXACT_R141_PASS"
        or any(
            value != "NOT_AUTHORIZED"
            for key, value in bounded.items()
            if key != "r142_report_only_decision"
        )
        or profile.get("projection_numeric_contract", {}).get(
            "rank_revealing_relative_null_maximum"
        )
        != 1.0e-12
        or profile.get("projection_numeric_contract", {}).get(
            "rank_revealing_relative_retained_minimum"
        )
        != 1.0e-10
        or set(profile.get("required_pre_graph_hashes", {}))
        != {
            "r120_report",
            "r120_cache",
            "r120_accepted_arrays",
            "r133_report",
            "r133_projected_velocity",
            "r133_applied_target",
            "r133_applied_effort",
            "r131_mode_sequence",
            "r138_graph_index",
            "r138_transition_replay",
        }
        or source.get("authority_repository_commit")
        != "c9a62d66a06ffccd072d3da48b4ffd9938849b25"
        or hashlib.sha256(
            canonical_json(profile.get("validation_commands", []))
        ).hexdigest()
        != VALIDATION_COMMANDS_SHA256
        or profile.get("execution_environment")
        != {
            "BLIS_NUM_THREADS": "1",
            "MKL_NUM_THREADS": "1",
            "NUMEXPR_NUM_THREADS": "1",
            "OMP_NUM_THREADS": "1",
            "OPENBLAS_NUM_THREADS": "1",
            "VECLIB_MAXIMUM_THREADS": "1",
        }
    ):
        raise ValueError("R141 execution profile differs")


def _validate_source_reports(
    *,
    profile: Mapping[str, Any],
    reports: Mapping[str, Mapping[str, Any]],
) -> None:
    source = profile["source"]
    r140 = reports["R140"]
    r139 = reports["R139"]
    r120 = reports["R120"]
    r133 = reports["R133"]
    r138 = reports["R138"]
    if (
        r140.get("status") != "PASS"
        or r140.get("result_transition") != "R140_PASS_R141_ROADMAP_DECISION_ONLY"
        or r140.get("repository", {}).get("commit")
        != source["r140"]["repository_commit"]
        or r139.get("status") != "COMPLETE"
        or r139.get("result_transition") != "R139_COMPLETE_R140_CONFORMANCE_ONLY"
        or r139.get("repository", {}).get("commit")
        != source["r139"]["repository_commit"]
        or r120.get("status") != "PASS"
        or r120.get("repository", {}).get("commit")
        != source["r120"]["repository_commit"]
        or r120.get("solver_result", {})
        .get("accepted_exact_result", {})
        .get("emitted_hashes", {})
        .get("aggregate_sha256")
        != source["r120"]["accepted_emitted_aggregate_sha256"]
        or r133.get("status") != "PASS"
        or r133.get("invalid_reason") is not None
        or r133.get("repository", {}).get("commit")
        != source["r133"]["repository_commit"]
        or r138.get("status") != "PASS"
        or r138.get("repository", {}).get("commit")
        != source["r138"]["repository_commit"]
    ):
        raise ExecutionInvalid("SOURCE_REPORT_STATUS_OR_LINEAGE_DIFFERS")
    if any(
        report.get("repository", {}).get("dirty") is not False
        or report.get("repository", {}).get("dirty_paths") != []
        for report in reports.values()
    ):
        raise ExecutionInvalid("SOURCE_REPORT_REPOSITORY_WAS_DIRTY")


def _encode_solver_private_cache(
    *,
    outcome: ExecutionOutcome,
    inventory: GraphInventory,
    profile_sha256: str,
) -> tuple[bytes | None, dict[str, Any] | None]:
    state = outcome.final_state
    audit = outcome.final_audit
    anchors = outcome.anchors
    if state is None or audit is None or audit.controller is None or anchors is None:
        return None, None
    assert audit.residual_vector is not None
    assert audit.violation_vector is not None
    assert audit.graph_row_addresses is not None
    assert audit.funnel is not None
    branch = np.concatenate(
        (
            audit.controller.target_branch_addresses.reshape(-1),
            audit.controller.branch_addresses.reshape(-1),
        )
    ).astype(np.uint8)
    metadata = {
        "schema_version": 1,
        "cache_id": "nextengine.humanoid-r141-fixed-mode-kinodynamic-solver-private.v1",
        "profile_sha256": profile_sha256,
        "status": outcome.status,
        "feasibility": outcome.feasibility,
        "candidate_or_corpus_authority": False,
        "physx_or_training_authority": False,
    }
    metadata_bytes = canonical_json(metadata)
    payload = io.BytesIO()
    np.savez(
        payload,
        root_position=state.root_position,
        root_rotation=state.root_rotation,
        joint_position=state.joint_position,
        velocity=state.velocity,
        acceleration=state.acceleration,
        integer_command=state.integer_command,
        exact_applied_target=audit.controller.applied_target,
        exact_applied_effort=audit.controller.applied_effort,
        active_force=state.active_force,
        activation_impulse=state.activation_impulse,
        contact_anchor=anchors,
        graph_row_address=audit.graph_row_addresses,
        controller_event_address=audit.controller.event_addresses,
        controller_branch_address=branch,
        exact_residual_vector=audit.residual_vector,
        exact_violation_vector=audit.violation_vector,
        exact_funnel_vector=np.asarray(audit.funnel, dtype=np.float64),
        metadata_json_utf8=np.frombuffer(metadata_bytes, dtype=np.uint8),
    )
    cache_bytes = payload.getvalue()
    output_arrays: dict[str, NDArray[Any]] = {
        "q": np.concatenate(
            (
                state.root_position.reshape(-1),
                state.root_rotation.reshape(-1),
                state.joint_position.reshape(-1),
            )
        ),
        "v": state.velocity,
        "a": state.acceleration,
        "integer_command": state.integer_command,
        "exact_applied_target": audit.controller.applied_target,
        "exact_applied_effort": audit.controller.applied_effort,
        "active_force": state.active_force,
        "activation_impulse": state.activation_impulse,
        "contact_anchor": anchors,
        "graph_row_address": audit.graph_row_addresses,
        "controller_event_address": audit.controller.event_addresses,
        "controller_branch_address": branch,
        "exact_residual_vector": audit.residual_vector,
        "exact_funnel_vector": np.asarray(audit.funnel, dtype=np.float64),
    }
    return cache_bytes, {
        "status": "EMITTED_SOLVER_PRIVATE",
        "file_name": "solver-private-r141-fixed-mode-kinodynamic.npz",
        "file_sha256": hashlib.sha256(cache_bytes).hexdigest(),
        "file_bytes": len(cache_bytes),
        "candidate_or_corpus_authority": False,
        "output_array_identities": {
            name: _typed_array_identity(value) for name, value in output_arrays.items()
        },
        "metadata": metadata,
    }


def execute_and_build_fixed_mode_kinodynamic_report(
    *,
    profile_path: Path,
    authority_document_path: Path,
    r140_report_path: Path,
    r140_profile_path: Path,
    r140_module_path: Path,
    r140_tool_path: Path,
    r139_report_path: Path,
    r139_profile_path: Path,
    r139_module_path: Path,
    r139_tool_path: Path,
    r120_report_path: Path,
    r120_profile_path: Path,
    r120_cache_path: Path,
    r133_report_path: Path,
    r133_profile_path: Path,
    r133_module_path: Path,
    r133_tool_path: Path,
    r138_report_path: Path,
    r138_profile_path: Path,
    r138_module_path: Path,
    r138_tool_path: Path,
    v9_complete_clip_path: Path,
    dynamics_kernel_path: Path,
    projection_module_path: Path,
    descriptor_bytes: bytes,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
    execution_environment: Mapping[str, str],
) -> tuple[dict[str, Any], bytes | None]:
    named_paths = {
        "profile": profile_path,
        "authority_document": authority_document_path,
        "r140_report": r140_report_path,
        "r140_profile": r140_profile_path,
        "r140_module": r140_module_path,
        "r140_tool": r140_tool_path,
        "r139_report": r139_report_path,
        "r139_profile": r139_profile_path,
        "r139_module": r139_module_path,
        "r139_tool": r139_tool_path,
        "r120_report": r120_report_path,
        "r120_profile": r120_profile_path,
        "r120_cache": r120_cache_path,
        "r133_report": r133_report_path,
        "r133_profile": r133_profile_path,
        "r133_module": r133_module_path,
        "r133_tool": r133_tool_path,
        "r138_report": r138_report_path,
        "r138_profile": r138_profile_path,
        "r138_module": r138_module_path,
        "r138_tool": r138_tool_path,
        "v9_complete_clip": v9_complete_clip_path,
        "dynamics_kernel": dynamics_kernel_path,
        "projection_module": projection_module_path,
        "tool": tool_path,
    }
    paths = {name: path.resolve() for name, path in named_paths.items()}
    absent = [name for name, path in paths.items() if not path.is_file()]
    if absent:
        raise FileNotFoundError(f"R141 input is absent: {absent}")
    profile = json.loads(paths["profile"].read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    _validate_execution_environment(profile, execution_environment)
    validations = _validate_results(profile, validation_results)
    source = profile["source"]
    if (
        sha256(Path(__file__).resolve()) != source["execution_module_sha256"]
        or sha256(paths["tool"]) != source["tool_sha256"]
    ):
        raise ValueError("R141 implementation identity differs")
    tracked_expected = (
        (paths["authority_document"], source["authority_document_sha256"]),
        (paths["r140_profile"], source["r140"]["profile_sha256"]),
        (paths["r140_module"], source["r140"]["module_sha256"]),
        (paths["r140_tool"], source["r140"]["tool_sha256"]),
        (paths["r139_profile"], source["r139"]["profile_sha256"]),
        (paths["r139_module"], source["r139"]["module_sha256"]),
        (paths["r139_tool"], source["r139"]["tool_sha256"]),
        (paths["r120_profile"], source["r120"]["profile_sha256"]),
        (paths["r133_profile"], source["r133"]["profile_sha256"]),
        (paths["r133_module"], source["r133"]["module_sha256"]),
        (paths["r133_tool"], source["r133"]["tool_sha256"]),
        (paths["r138_profile"], source["r138"]["profile_sha256"]),
        (paths["r138_module"], source["r138"]["module_sha256"]),
        (paths["r138_tool"], source["r138"]["tool_sha256"]),
        (paths["dynamics_kernel"], source["dynamics_kernel_sha256"]),
        (paths["projection_module"], source["projection_module_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in tracked_expected):
        raise ValueError("R141 tracked source identity differs")
    if hashlib.sha256(descriptor_bytes).hexdigest() != source["descriptor_sha256"]:
        raise ValueError("R141 descriptor identity differs")

    np.random.seed(int(profile["execution_budget"]["seed"]))
    started = time.monotonic()
    reports: dict[str, dict[str, Any]] = {}
    outcome: ExecutionOutcome
    inventory: GraphInventory | None = None
    reference: ReferenceReconstruction | None = None
    try:
        reports = {
            label: _load_bound_report(
                paths[f"{label.lower()}_report"], source[label.lower()], label
            )
            for label in ("R140", "R139", "R120", "R133", "R138")
        }
        _validate_source_reports(profile=profile, reports=reports)
        if sha256(paths["v9_complete_clip"]) != source["v9_complete_clip_sha256"]:
            raise ExecutionInvalid("V9_COMPLETE_CLIP_SHA256_DIFFERS")
        descriptor = json.loads(descriptor_bytes)
        validate_current_biomechanics_descriptor(descriptor)
        model = build_spatial_model(descriptor)
        points = _contact_points(descriptor)
        with np.load(paths["v9_complete_clip"], allow_pickle=False) as archive:
            v9_arrays = {
                name: np.array(archive[name], copy=True)
                for name in (
                    "contact_modes",
                    "root_quaternion_q1_30",
                    "root_yaw_urad",
                )
            }
        inventory = build_graph_inventory(
            np.asarray(v9_arrays["contact_modes"], dtype=np.uint8)
        )
        cache = load_r120_source_cache(
            paths["r120_cache"], expected_sha256=source["r120"]["cache_sha256"]
        )
        expected_r120 = profile["expected_array_hashes"]["r120"]
        expected_r133 = profile["expected_array_hashes"]["r133"]
        reference = reconstruct_reference(
            cache=cache,
            v9_arrays=v9_arrays,
            descriptor=descriptor,
            model=model,
            points=points,
            inventory=inventory,
            projection_numeric=profile["projection_numeric_contract"],
            expected_r120_hashes=expected_r120,
            expected_r133_hashes=expected_r133,
            r133_report=reports["R133"],
        )
        source_gate = {
            "r120_report": reports["R120"]["report_sha256"],
            "r120_cache": sha256(paths["r120_cache"]),
            "r120_accepted_arrays": reports["R120"]["solver_result"][
                "accepted_exact_result"
            ]["emitted_hashes"]["aggregate_sha256"],
            "r133_report": reports["R133"]["report_sha256"],
            "r133_projected_velocity": reference.source_hashes["projected_velocity"],
            "r133_applied_target": reference.source_hashes["applied_target"],
            "r133_applied_effort": reference.source_hashes["applied_effort"],
            "r131_mode_sequence": inventory.motor_mode_sequence_sha256,
            "r138_graph_index": inventory.graph_index_sha256,
            "r138_transition_replay": inventory.transition_replay_sha256,
        }
        if source_gate != profile["required_pre_graph_hashes"]:
            raise ExecutionInvalid("PRE_GRAPH_TEN_CATEGORY_HASH_CLOSURE_DIFFERS")
        outcome = solve_fixed_mode_kinodynamic_problem(
            reference=reference,
            source_gate=source_gate,
            descriptor=descriptor,
            model=model,
            points=points,
            inventory=inventory,
            started=started,
        )
    except Exception as error:
        invalid_reason = _stable_invalid_reason(error, prefix="SOURCE_EXCEPTION")
        counters = {name: 0 for name in REQUIRED_COUNTERS}
        outcome = ExecutionOutcome(
            status="INVALID",
            feasibility="INVALID",
            termination="SOURCE_OR_RECONSTRUCTION_INVALID",
            invalid_reason=invalid_reason,
            final_state=None,
            final_audit=None,
            anchors=None,
            history=(),
            accepted_anchor_hashes=(),
            source_gate={},
            counters=counters,
            resource_usage={
                "status": "INVALID",
                "invalid_reason": invalid_reason,
                "wall_clock_seconds": time.monotonic() - started,
                "maximum_resident_memory_bytes": _maximum_rss_bytes(),
                "maximum_observed_thread_count": _linux_thread_count(),
                "execution_process_count": 1,
                "random_seed": 0,
                "randomized_restart_count": 0,
                "resume_or_warm_restart": "FORBIDDEN_AND_NOT_USED",
                "manual_intervention": "FORBIDDEN_AND_NOT_USED",
            },
        )

    cache_bytes: bytes | None = None
    cache_identity: dict[str, Any] | None = None
    if inventory is not None and outcome.status != "INVALID":
        cache_bytes, cache_identity = _encode_solver_private_cache(
            outcome=outcome,
            inventory=inventory,
            profile_sha256=sha256(paths["profile"]),
        )
    counters = dict(outcome.counters)
    counters["solver_private_caches_built"] = int(cache_bytes is not None)
    if counters["real_qp_solves"] > 18 or counters["real_exact_trial_audits"] > 42:
        raise ValueError("R141 counter budget closure differs")
    decision_key = (
        "pass"
        if outcome.status == "PASS"
        else "bounded_stop"
        if outcome.status == "COMPLETE"
        else "invalid"
    )
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": outcome.status,
        "claim": profile["claim"],
        "gate_decision": profile["decision"][decision_key],
        "result_transition": profile["result_transitions"][decision_key],
        "scope": profile["scope"],
        "feasibility": outcome.feasibility,
        "termination": outcome.termination,
        "invalid_reason": outcome.invalid_reason,
        "source_gate": outcome.source_gate,
        "reference_reconstruction": (
            {
                "status": "PASS",
                "source_hashes": reference.source_hashes,
                "configuration_hashes": reference.configuration_hashes,
                "projection_systems": reference.projection_systems,
                "projection_factorizations": reference.projection_factorizations,
                "projection_solves": reference.projection_solves,
                "r120_acceleration_array_read": False,
                "r136_cache_or_witness_read": False,
            }
            if reference is not None
            else {"status": "INVALID_OR_NOT_REACHED"}
        ),
        "graph_inventory": (
            {
                "force_rows": len(inventory.force_rows),
                "impulse_rows": len(inventory.impulse_rows),
                "anchor_rows": len(inventory.anchor_rows),
                "motor_mode_sequence_sha256": inventory.motor_mode_sequence_sha256,
                "graph_index_sha256": inventory.graph_index_sha256,
                "transition_replay_sha256": inventory.transition_replay_sha256,
            }
            if inventory is not None
            else None
        ),
        "solve_history": list(outcome.history),
        "accepted_anchor_hashes": list(outcome.accepted_anchor_hashes),
        "final_exact_audit": (
            {
                "status": outcome.final_audit.status,
                "invalid_reason": outcome.final_audit.invalid_reason,
                "hard_pass": outcome.final_audit.hard_pass,
                "funnel": list(outcome.final_audit.funnel or ()),
                "category_aggregate": outcome.final_audit.category_aggregate,
                "violated_cones": len(outcome.final_audit.violated_cones),
                "residual_vector_sha256": (
                    _array_sha256(outcome.final_audit.residual_vector)
                    if outcome.final_audit.residual_vector is not None
                    else None
                ),
                "graph_row_address_sha256": (
                    _array_sha256(outcome.final_audit.graph_row_addresses)
                    if outcome.final_audit.graph_row_addresses is not None
                    else None
                ),
            }
            if outcome.final_audit is not None
            else None
        ),
        "solver_private_cache": cache_identity,
        "resource_usage": outcome.resource_usage,
        "counter_closure": counters,
        **counters,
        "validation_results": validations,
        "execution_environment": dict(execution_environment),
        "numeric_stack": {
            "numpy": np.__version__,
            "scipy": scipy.__version__,
            "osqp": osqp.__version__,
        },
        "identities": {
            "profile_sha256": sha256(paths["profile"]),
            "authority_document_sha256": sha256(paths["authority_document"]),
            **{
                f"{label.lower()}_report_file_sha256": sha256(
                    paths[f"{label.lower()}_report"]
                )
                for label in ("R140", "R139", "R120", "R133", "R138")
            },
            "r120_cache_sha256": sha256(paths["r120_cache"]),
            "v9_complete_clip_sha256": sha256(paths["v9_complete_clip"]),
            "descriptor_sha256": hashlib.sha256(descriptor_bytes).hexdigest(),
            "dynamics_kernel_sha256": sha256(paths["dynamics_kernel"]),
            "projection_module_sha256": sha256(paths["projection_module"]),
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(paths["tool"]),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "repository": dict(repository),
        "learned_policy_claim": False,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report, cache_bytes
