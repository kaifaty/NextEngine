from __future__ import annotations

import json
import os
import re
import struct
from pathlib import Path
from typing import Any, Iterable

import torch

from next_lab.motor_mirror import (
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    BIOMECHANICS_STANDING_PROFILE_ID,
    BOUNDED_STANDING_PROFILE_ID,
    CURRICULUM_LOCOMOTION_PROFILE_ID,
    FLAT_LOCOMOTION_PROFILE_ID,
    Q1_30_ONE,
    RATE_CLAMPED,
    STANDING_PROFILE_ID,
    biomechanics_forward_start_stop_command_schedule_v2,
    curriculum_locomotion_command_schedule,
    derive_purpose_seed,
    flat_locomotion_command_schedule,
    select_environment_profile,
    select_biomechanics_training_profile,
    validate_biomechanics_forward_start_stop_descriptor,
    validate_biomechanics_standing_descriptor,
    validate_current_biomechanics_descriptor,
    validate_descriptor,
)
from next_lab.usd_translation import validate_translation_bundle

BOUNDED_STANDING_REWARD_COEFFICIENTS_Q16 = (
    65_536,
    32_768,
    16_384,
    -6_554,
    -1_311,
    -3_277,
    -6_554,
    -131_072,
)

try:
    import isaaclab.sim as sim_utils
    from isaaclab.actuators import ImplicitActuatorCfg
    from isaaclab.assets import Articulation, ArticulationCfg
    from isaaclab.envs import DirectRLEnv, DirectRLEnvCfg
    from isaaclab.scene import InteractiveSceneCfg
    from isaaclab.sensors import ContactSensor, ContactSensorCfg
    from isaaclab.utils import configclass

    ISAAC_LAB_AVAILABLE = True
except ImportError:
    ISAAC_LAB_AVAILABLE = False

LOCOMOTION_REWARD_COEFFICIENTS_Q16 = (
    98_304,
    32_768,
    32_768,
    16_384,
    -3_277,
    -3_277,
    -1_311,
    -3_277,
    -6_554,
    -131_072,
)
CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16 = (
    131_072,
    32_768,
    65_536,
    32_768,
    -6_554,
    -6_554,
    -655,
    -1_311,
    -13_107,
    16_384,
    -655_360,
)
BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2 = (
    131_072,
    16_384,
    -32_768,
    -16_384,
    -6_554,
    -6_554,
    -1_311,
    -3_277,
    -6_554,
    16_384,
    -655_360,
)
BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3 = (
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2
)
FROZEN_STAGE0_V1_BODY_SCHEMA_HASH = (
    "13f01daf349cddd84f6c3a068cf9da9ff1307a727949ffa9232ec2e02cbc9bd5"
)
FROZEN_STAGE0_V1_ROOT_HEIGHT_MICROMETRES = 1_050_000
FROZEN_STAGE0_V1_GROUND_PENETRATION_MICROMETRES = -45_000
MICRO_SCALE = 1_000_000
ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS = 50_000
CONTACT_BRUSH_CEILING_MICRONEWTON_SECONDS = 250_000
LOW_IMPULSE_GRACE_SUBSTEPS = 4
OBSERVED_HARD_ROM_QUANTIZATION_TOLERANCE_MICRORADIANS = 10
OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND = 1_000


def is_locomotion_profile(profile_id: str) -> bool:
    return profile_id in {
        FLAT_LOCOMOTION_PROFILE_ID,
        CURRICULUM_LOCOMOTION_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    }


def is_standing_profile(profile_id: str) -> bool:
    return profile_id in {
        STANDING_PROFILE_ID,
        BOUNDED_STANDING_PROFILE_ID,
        BIOMECHANICS_STANDING_PROFILE_ID,
    }


def is_biomechanics_standing_profile(profile_id: str) -> bool:
    return profile_id == BIOMECHANICS_STANDING_PROFILE_ID


def is_biomechanics_profile(profile_id: str) -> bool:
    return profile_id in {
        BIOMECHANICS_STANDING_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    }


def fall_height_threshold_micrometres(profile_id: str) -> int:
    return (
        450_000
        if is_locomotion_profile(profile_id)
        or is_biomechanics_standing_profile(profile_id)
        else 250_000
    )


def require_finite_tensor(
    name: str, value: torch.Tensor, *, asynchronous: bool = False
) -> None:
    finite = torch.isfinite(value).all()
    if asynchronous and value.device.type == "cuda":
        torch._assert_async(finite, f"non-finite Isaac tensor: {name}")
    elif not bool(finite.item()):
        raise RuntimeError(f"non-finite Isaac tensor: {name}")


def isaac_prim_name(identifier: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", identifier)


def _hard_impact_limit_for_role(contact_role: int) -> int:
    if contact_role == 8:
        return 6_000_000
    if contact_role in {1, 2, 4, 5, 6}:
        return 4_000_000
    if contact_role in {7, 9, 10, 11}:
        return 3_000_000
    if contact_role == 3:
        return 1_000_000
    raise ValueError(f"unsupported biomechanics contact role: {contact_role}")


def _contact_body_projections(
    descriptor: dict[str, Any],
) -> tuple[tuple[str, int, int], ...]:
    projections: list[tuple[str, int, int]] = []
    for body in descriptor["bodies"]:
        candidates = []
        for collider in body["colliders"]:
            role = int(collider["contact_role"])
            shape = int(collider["shape_token"])
            candidates.append((_hard_impact_limit_for_role(role), role, shape))
        if candidates:
            hard_limit, role, _ = min(candidates)
            projections.append((str(body["body_id"]), role, hard_limit))
    if not projections or len({body_id for body_id, _, _ in projections}) != len(
        projections
    ):
        raise ValueError("invalid contact-bearing body projection")
    return tuple(projections)


def _contact_pair_layout(
    projections: tuple[tuple[str, int, int], ...],
) -> dict[str, tuple[int | bool | str, ...]]:
    body_count = len(projections)
    sensor: list[int] = []
    filter_index: list[int] = []
    primary_role: list[int] = []
    hard_limit: list[int] = []
    self_contact: list[bool] = []
    pair_id: list[str] = []
    for body_index, (_, role, limit) in enumerate(projections):
        sensor.append(body_index)
        filter_index.append(0)
        primary_role.append(role)
        hard_limit.append(limit)
        self_contact.append(False)
        pair_id.append(f"ground:{projections[body_index][0]}")
    for first in range(body_count):
        for second in range(first + 1, body_count):
            sensor.append(first)
            filter_index.append(1 + second)
            primary_role.append(projections[first][1])
            hard_limit.append(min(projections[first][2], projections[second][2]))
            self_contact.append(True)
            pair_id.append(f"{projections[first][0]}:{projections[second][0]}")
    return {
        "sensor": tuple(sensor),
        "filter": tuple(filter_index),
        "primary_role": tuple(primary_role),
        "hard_limit": tuple(hard_limit),
        "self_contact": tuple(self_contact),
        "pair_id": tuple(pair_id),
    }


def _integer_vector_threshold(
    impulse: torch.Tensor,
    threshold: torch.Tensor | int,
    *,
    inclusive: bool,
) -> torch.Tensor:
    if impulse.dtype != torch.int64 or impulse.shape[-1] != 3:
        raise ValueError("canonical contact impulse must be int64 xyz")
    limit = torch.as_tensor(threshold, dtype=torch.int64, device=impulse.device)
    while limit.ndim < impulse.ndim - 1:
        limit = limit.unsqueeze(0)
    absolute = torch.abs(impulse)
    bounded = torch.minimum(absolute, limit[..., None])
    magnitude_squared = torch.sum(bounded * bounded, dim=-1)
    limit_squared = limit * limit
    if inclusive:
        return torch.any(absolute >= limit[..., None], dim=-1) | (
            magnitude_squared >= limit_squared
        )
    return torch.any(absolute > limit[..., None], dim=-1) | (
        magnitude_squared > limit_squared
    )


def _classify_contact_pairs_tensor(
    impulse_micronewton_seconds: torch.Tensor,
    minimum_separation_micrometres: torch.Tensor,
    previous_continuity: torch.Tensor,
    hard_limits: torch.Tensor,
    self_contact: torch.Tensor,
    primary_roles: torch.Tensor,
) -> dict[str, torch.Tensor]:
    if (
        impulse_micronewton_seconds.dtype != torch.int64
        or minimum_separation_micrometres.dtype != torch.int64
        or previous_continuity.dtype != torch.int64
        or impulse_micronewton_seconds.shape[:-1]
        != minimum_separation_micrometres.shape
        or minimum_separation_micrometres.shape != previous_continuity.shape
        or hard_limits.shape != minimum_separation_micrometres.shape[-1:]
        or self_contact.shape != hard_limits.shape
        or primary_roles.shape != hard_limits.shape
    ):
        raise ValueError("contact pair tensor layout mismatch")
    active = (minimum_separation_micrometres < 0) | _integer_vector_threshold(
        impulse_micronewton_seconds,
        ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS,
        inclusive=True,
    )
    continuity = torch.where(
        active, previous_continuity + 1, torch.zeros_like(previous_continuity)
    )
    material = active & (
        _integer_vector_threshold(
            impulse_micronewton_seconds,
            CONTACT_BRUSH_CEILING_MICRONEWTON_SECONDS,
            inclusive=False,
        )
        | (continuity > LOW_IMPULSE_GRACE_SUBSTEPS)
    )
    hard_impact = active & _integer_vector_threshold(
        impulse_micronewton_seconds, hard_limits, inclusive=False
    )
    return {
        "active": active,
        "continuity": continuity,
        "material": material,
        "hard_impact": hard_impact,
        "self_collision": material & self_contact[None],
        "forbidden_locomotion": material
        & ~self_contact[None]
        & (primary_roles[None] != 8),
    }


def _minimum_contact_separation_tensor(
    separation_metres: torch.Tensor,
    contact_count: torch.Tensor,
    contact_start: torch.Tensor,
) -> torch.Tensor:
    if (
        separation_metres.ndim != 2
        or separation_metres.shape[1] != 1
        or contact_count.shape != contact_start.shape
        or contact_count.dtype != torch.int32
        or contact_start.dtype != torch.int32
    ):
        raise ValueError("PhysX contact detail tensor layout mismatch")
    flat_count = contact_count.reshape(-1).to(torch.int64)
    flat_start = contact_start.reshape(-1).to(torch.int64)
    expected_start = torch.cumsum(flat_count, dim=0) - flat_count
    packed = torch.all((flat_count == 0) | (flat_start == expected_start))
    if packed.device.type == "cuda":
        torch._assert_async(
            packed, "PhysX contact detail buffer is not canonical packed order"
        )
    elif not bool(packed.item()):
        raise RuntimeError("PhysX contact detail buffer is not canonical packed order")
    owner = torch.repeat_interleave(
        torch.arange(flat_count.numel(), device=flat_count.device), flat_count
    )
    used = separation_metres.reshape(-1)[: owner.shape[0]]
    canonical = torch.round(used.to(torch.float64) * MICRO_SCALE).to(torch.int64)
    output = torch.full(
        (flat_count.numel(),),
        torch.iinfo(torch.int64).max,
        dtype=torch.int64,
        device=flat_count.device,
    )
    output.scatter_reduce_(0, owner, canonical, reduce="amin", include_self=True)
    return output.reshape(contact_count.shape)


def isaac_actuator_limits_from_descriptor(
    descriptor: dict[str, Any],
) -> dict[str, dict[str, float]]:
    """Map engine-owned per-joint safety limits into Isaac actuator parameters."""
    velocities: dict[str, float] = {}
    for joint in descriptor["joints"]:
        raw = joint["maximum_velocity_microradians_per_second"]
        if not isinstance(raw, int) or isinstance(raw, bool) or raw <= 0:
            raise ValueError("descriptor joint velocity limits must be positive integers")
        velocities[isaac_prim_name(joint["joint_id"])] = raw / 1_000_000.0

    efforts: dict[str, float] = {}
    for actuator in descriptor["actuators"]:
        if "effort_micronewton_metres" in actuator:
            effort_range = actuator["effort_micronewton_metres"]
            if (
                not isinstance(effort_range, list)
                or len(effort_range) != 2
                or not all(
                    isinstance(value, int) and not isinstance(value, bool)
                    for value in effort_range
                )
                or not effort_range[0] < 0 < effort_range[1]
            ):
                raise ValueError("descriptor actuator effort range is invalid")
            raw = max(abs(effort_range[0]), abs(effort_range[1]))
        else:
            raw = actuator["maximum_effort_micronewton_metres"]
        if not isinstance(raw, int) or isinstance(raw, bool) or raw <= 0:
            raise ValueError("descriptor actuator effort limits must be positive integers")
        efforts[isaac_prim_name(actuator["joint_id"])] = raw / 1_000_000.0

    if velocities.keys() != efforts.keys():
        raise ValueError("descriptor joints and actuators must cover the same joint IDs")
    return {
        "effort_limit_sim": efforts,
        "velocity_limit_sim": velocities,
    }


def validate_biomechanics_training_descriptor(descriptor: dict[str, Any]) -> None:
    training_descriptor_id = descriptor.get("training_descriptor_id")
    if training_descriptor_id == "nextengine.isaac.humanoid-biomechanics-standing.v2":
        validate_biomechanics_standing_descriptor(descriptor)
    elif training_descriptor_id in {
        "nextengine.isaac.humanoid-biomechanics-forward-start-stop.v1",
        "nextengine.isaac.humanoid-biomechanics-forward-start-stop.v2",
        "nextengine.isaac.humanoid-biomechanics-forward-start-stop.v3",
        "nextengine.isaac.humanoid-biomechanics-forward-start-stop.v4",
        "nextengine.isaac.humanoid-biomechanics-forward-start-stop.v5",
    }:
        validate_biomechanics_forward_start_stop_descriptor(descriptor)
    else:
        raise ValueError("unsupported biomechanics training descriptor")


def authored_ground_clearance_metres(descriptor: dict[str, Any]) -> float:
    """Return the lowest authored collider point above the flat ground plane."""
    if descriptor.get("translator_id") == "nextengine.isaac.biomechanics-mirror.v2":
        validate_biomechanics_training_descriptor(descriptor)
        minimum: int | None = None
        for body in descriptor["bodies"]:
            world_height = round(
                _f32_from_bits(body["initial_translation_f32_bits"][1]) * 1_000_000
            )
            for collider in body["colliders"]:
                if collider.get("contact_role") != 8:
                    continue
                geometry = collider["geometry"]
                if geometry.get("kind") != "box":
                    raise ValueError("sole support collider must be a box")
                bottom = (
                    world_height
                    + collider["local_translation_micrometres"][1]
                    - geometry["half_extents_micrometres"][1]
                )
                minimum = bottom if minimum is None else min(minimum, bottom)
        if minimum is None:
            raise ValueError("biomechanics descriptor contains no sole colliders")
        return minimum / 1_000_000.0
    bodies = {body["body_id"]: body for body in descriptor["bodies"]}
    world_heights: dict[str, int] = {}
    while len(world_heights) < len(bodies):
        before = len(world_heights)
        for body_id, body in bodies.items():
            if body_id in world_heights:
                continue
            parent_id = body["parent_body_id"]
            if parent_id is None:
                parent_height = 0
            elif parent_id in world_heights:
                parent_height = world_heights[parent_id]
            else:
                continue
            world_heights[body_id] = (
                parent_height + body["local_bind_translation_micrometres"][1]
            )
        if len(world_heights) == before:
            raise ValueError("descriptor body hierarchy does not resolve")

    minimum: int | None = None
    for body_id, body in bodies.items():
        for collider in body["colliders"]:
            geometry = collider["geometry"]
            if geometry["kind"] == "sphere":
                vertical_extent = geometry["radius_micrometres"]
            elif geometry["kind"] == "capsule":
                vertical_extent = geometry["radius_micrometres"]
            elif geometry["kind"] == "box":
                vertical_extent = geometry["half_extents_micrometres"][1]
            else:
                raise ValueError("unsupported collider geometry for ground clearance")
            bottom = (
                world_heights[body_id]
                + collider["local_translation_micrometres"][1]
                - vertical_extent
            )
            minimum = bottom if minimum is None else min(minimum, bottom)
    if minimum is None:
        raise ValueError("descriptor contains no colliders")
    return minimum / 1_000_000.0


def authored_root_height_micrometres(descriptor: dict[str, Any]) -> int:
    if descriptor.get("translator_id") == "nextengine.isaac.biomechanics-mirror.v2":
        validate_biomechanics_training_descriptor(descriptor)
        roots = [
            body for body in descriptor["bodies"] if body["parent_body_slot"] is None
        ]
        if len(roots) != 1:
            raise ValueError("descriptor must contain exactly one root body")
        return round(
            _f32_from_bits(roots[0]["initial_translation_f32_bits"][1])
            * 1_000_000
        )
    roots = [body for body in descriptor["bodies"] if body["parent_body_id"] is None]
    if len(roots) != 1:
        raise ValueError("descriptor must contain exactly one root body")
    value = roots[0]["local_bind_translation_micrometres"][1]
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError("descriptor root height must be an integer")
    return value


def require_compatible_authored_ground_clearance(descriptor: dict[str, Any]) -> float:
    """Accept tangent poses plus the exact immutable Stage 0 V1 legacy pose."""
    clearance = authored_ground_clearance_metres(descriptor)
    if clearance >= -1.0e-6:
        return clearance
    if is_frozen_stage0_v1_legacy_pose(descriptor, clearance):
        return clearance
    raise ValueError(
        "descriptor authored pose penetrates the flat ground: "
        f"{clearance:.6f} m"
    )


def is_frozen_stage0_v1_legacy_pose(
    descriptor: dict[str, Any], clearance: float | None = None
) -> bool:
    actual_clearance = (
        authored_ground_clearance_metres(descriptor)
        if clearance is None
        else clearance
    )
    return (
        descriptor.get("body_schema_hash") == FROZEN_STAGE0_V1_BODY_SCHEMA_HASH
        and authored_root_height_micrometres(descriptor)
        == FROZEN_STAGE0_V1_ROOT_HEIGHT_MICROMETRES
        and round(actual_clearance * 1_000_000.0)
        == FROZEN_STAGE0_V1_GROUND_PENETRATION_MICROMETRES
    )


def round_div_ties_even_tensor(numerator: torch.Tensor, denominator: int) -> torch.Tensor:
    if denominator <= 0 or numerator.dtype != torch.int64:
        raise ValueError("integer ties-to-even requires int64 and a positive denominator")
    absolute = torch.abs(numerator)
    quotient = torch.div(absolute, denominator, rounding_mode="floor")
    remainder = torch.remainder(absolute, denominator)
    increment = (remainder * 2 > denominator) | (
        (remainder * 2 == denominator) & (torch.remainder(quotient, 2) == 1)
    )
    rounded = quotient + increment.to(torch.int64)
    return torch.where(numerator < 0, -rounded, rounded)


def ratio_q16_tensor(value: torch.Tensor, maximum: int) -> torch.Tensor:
    if value.dtype != torch.int64 or maximum <= 0 or torch.any(value < 0):
        raise ValueError("Q16 ratio requires non-negative int64 values and a positive maximum")
    return round_div_ties_even_tensor(torch.clamp(value, max=maximum) * 65_536, maximum)


def dense_tracking_q16_tensor(error: torch.Tensor, normalization: int) -> torch.Tensor:
    if error.dtype != torch.int64 or normalization <= 0 or torch.any(error < 0):
        raise ValueError("dense Q16 tracking requires non-negative int64 errors")
    bounded = torch.clamp(error, max=normalization * 4_096)
    normalization_squared = normalization * normalization
    denominator = normalization_squared + bounded * bounded
    numerator = torch.full_like(denominator, normalization_squared * 65_536)
    quotient = torch.div(numerator, denominator, rounding_mode="floor")
    remainder = torch.remainder(numerator, denominator)
    increment = (remainder * 2 > denominator) | (
        (remainder * 2 == denominator) & (quotient % 2 == 1)
    )
    cauchy = quotient + increment.to(torch.int64)
    return round_div_ties_even_tensor(cauchy * cauchy, 65_536)


def fixed_pd_tensor(
    target: torch.Tensor,
    position: torch.Tensor,
    velocity: torch.Tensor,
    previous_effort: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor]:
    for value in (target, position, velocity, previous_effort):
        if value.dtype != torch.int64:
            raise ValueError("fixed PD inputs must be int64")
    target_limited = torch.clamp(target, -1_500_000, 1_500_000)
    proportional = round_div_ties_even_tensor((80 * 65_536) * (target_limited - position), 65_536)
    damping = round_div_ties_even_tensor((4 * 65_536) * velocity, 65_536)
    requested = proportional - damping
    effort_limited = torch.clamp(requested, -150_000_000, 150_000_000)
    maximum_delta = 25_000_000
    effort = torch.minimum(
        torch.maximum(effort_limited, previous_effort - maximum_delta),
        previous_effort + maximum_delta,
    )
    flags = torch.zeros_like(effort)
    flags |= (target_limited != target).to(torch.int64)
    flags |= ((effort_limited != requested).to(torch.int64) << 1)
    flags |= ((effort != effort_limited).to(torch.int64) * RATE_CLAMPED)
    return effort, flags


def biomechanics_procedural_standing_targets_tensor(
    *,
    actuator_joint_ids: tuple[str, ...],
    neutral_targets_microradians: torch.Tensor,
    root_quaternion_xyzw_q1_30: torch.Tensor,
    root_forward_micrometres: torch.Tensor,
    root_angular_velocity_microradians_per_second: torch.Tensor,
    root_forward_velocity_micrometres_per_second: torch.Tensor,
    include_root_forward_position_feedback: bool = True,
) -> torch.Tensor:
    if (
        neutral_targets_microradians.dtype != torch.int64
        or neutral_targets_microradians.shape != (len(actuator_joint_ids),)
        or root_quaternion_xyzw_q1_30.dtype != torch.int64
        or root_quaternion_xyzw_q1_30.shape[-1] != 4
    ):
        raise ValueError("invalid procedural standing tensor layout")
    pitch_proxy = round_div_ties_even_tensor(
        root_quaternion_xyzw_q1_30[:, 0] * 2_000_000, Q1_30_ONE
    )
    ankle_pitch = (
        -140_000
        + round_div_ties_even_tensor(pitch_proxy, 2)
        + round_div_ties_even_tensor(
            root_angular_velocity_microradians_per_second[:, 0], 20
        )
        + (
            round_div_ties_even_tensor(root_forward_micrometres, 10)
            if include_root_forward_position_feedback
            else 0
        )
        + round_div_ties_even_tensor(root_forward_velocity_micrometres_per_second, 50)
    )
    targets = neutral_targets_microradians[None].expand(
        root_quaternion_xyzw_q1_30.shape[0], -1
    ).clone()
    for index, joint_id in enumerate(actuator_joint_ids):
        if joint_id.endswith("-knee"):
            targets[:, index] = 100_000
        elif joint_id.endswith("-ankle-pitch"):
            targets[:, index] = ankle_pitch
    return targets


def biomechanics_fixed_pd_safety_tensor(
    *,
    target_microradians: torch.Tensor,
    position_microradians: torch.Tensor,
    velocity_microradians_per_second: torch.Tensor,
    previous_effort_micronewton_metres: torch.Tensor,
    used_positive_work_microjoules: torch.Tensor,
    stiffness_q16: torch.Tensor,
    damping_q16: torch.Tensor,
    effort_minimum: torch.Tensor,
    effort_maximum: torch.Tensor,
    maximum_effort_rate_per_second: torch.Tensor,
    maximum_power_microwatts: torch.Tensor,
    maximum_positive_work_microjoules: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    values = (
        target_microradians,
        position_microradians,
        velocity_microradians_per_second,
        previous_effort_micronewton_metres,
        used_positive_work_microjoules,
        stiffness_q16,
        damping_q16,
        effort_minimum,
        effort_maximum,
        maximum_effort_rate_per_second,
        maximum_power_microwatts,
        maximum_positive_work_microjoules,
    )
    if any(value.dtype != torch.int64 for value in values):
        raise ValueError("biomechanics safety tensors must use int64")
    requested = round_div_ties_even_tensor(
        stiffness_q16 * (target_microradians - position_microradians), 65_536
    ) - round_div_ties_even_tensor(
        damping_q16 * velocity_microradians_per_second, 65_536
    )
    maximum_delta = round_div_ties_even_tensor(
        maximum_effort_rate_per_second, 240
    )
    velocity_abs = torch.abs(velocity_microradians_per_second)
    nonzero = velocity_abs > 0
    unlimited = torch.full_like(velocity_abs, torch.iinfo(torch.int64).max)
    power_maximum = torch.where(
        nonzero,
        torch.div(
            maximum_power_microwatts * 1_000_000,
            torch.clamp(velocity_abs, min=1),
            rounding_mode="floor",
        ),
        unlimited,
    )
    remaining_work = maximum_positive_work_microjoules - used_positive_work_microjoules
    work_maximum = torch.where(
        nonzero,
        torch.div(
            torch.clamp(remaining_work, min=0) * 240 * 1_000_000,
            torch.clamp(velocity_abs, min=1),
            rounding_mode="floor",
        ),
        unlimited,
    )
    minimum = torch.maximum(
        torch.maximum(effort_minimum, previous_effort_micronewton_metres - maximum_delta),
        -power_maximum,
    )
    maximum = torch.minimum(
        torch.minimum(effort_maximum, previous_effort_micronewton_metres + maximum_delta),
        power_maximum,
    )
    maximum = torch.where(
        velocity_microradians_per_second > 0,
        torch.minimum(maximum, work_maximum),
        maximum,
    )
    minimum = torch.where(
        velocity_microradians_per_second < 0,
        torch.maximum(minimum, -work_maximum),
        minimum,
    )
    infeasible = (remaining_work < 0) | (minimum > maximum)
    effort = torch.minimum(torch.maximum(requested, minimum), maximum)
    positive_power = torch.clamp(
        effort * velocity_microradians_per_second, min=0
    )
    charge = torch.div(
        positive_power + 240_000_000 - 1,
        240_000_000,
        rounding_mode="floor",
    )
    next_work = used_positive_work_microjoules + charge
    infeasible |= next_work > maximum_positive_work_microjoules
    return effort, next_work, infeasible


def engine_vector_from_isaac_tensor(vector: torch.Tensor) -> torch.Tensor:
    """Map Isaac (+X,+Y,+Z) to engine (+right,+up,+forward)."""
    if vector.shape[-1] != 3:
        raise ValueError("vector must end in three components")
    return torch.stack((vector[..., 0], vector[..., 2], -vector[..., 1]), dim=-1)


def engine_quaternion_xyzw_from_isaac_wxyz_tensor(quaternion: torch.Tensor) -> torch.Tensor:
    if quaternion.shape[-1] != 4:
        raise ValueError("quaternion must end in four components")
    w, x, y, z = quaternion.unbind(dim=-1)
    return torch.stack((x, z, -y, w), dim=-1)


def isaac_root_state_from_descriptor(descriptor: dict[str, Any]) -> tuple[float, ...]:
    if descriptor.get("translator_id") == "nextengine.isaac.biomechanics-mirror.v2":
        validate_biomechanics_training_descriptor(descriptor)
        roots = [
            body for body in descriptor["bodies"] if body["parent_body_slot"] is None
        ]
        if len(roots) != 1:
            raise ValueError("descriptor must declare exactly one root body")
        root = roots[0]
        x, up, forward = tuple(
            _f32_from_bits(value) for value in root["initial_translation_f32_bits"]
        )
        qx, qy, qz, qw = tuple(
            _f32_from_bits(value) for value in root["initial_rotation_f32_bits"]
        )
        return (x, -forward, up, qw, qx, -qz, qy, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    roots = [body for body in descriptor["bodies"] if body["parent_body_id"] is None]
    if len(roots) != 1:
        raise ValueError("descriptor must declare exactly one root body")
    root = roots[0]
    x, up, forward = root["local_bind_translation_micrometres"]
    qx, qy, qz, qw = root["local_bind_rotation_q1_30"]
    scale = float(Q1_30_ONE)
    return (
        x / 1_000_000.0,
        -forward / 1_000_000.0,
        up / 1_000_000.0,
        qw / scale,
        qx / scale,
        -qz / scale,
        qy / scale,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    )


def _f32_from_bits(value: int) -> float:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value < 2**32:
        raise ValueError("f32 bit pattern must be a u32")
    result = struct.unpack("<f", struct.pack("<I", value))[0]
    if not torch.isfinite(torch.tensor(result)):
        raise ValueError("f32 bit pattern must be finite")
    return result


def rotate_world_to_root_local_q1_30_tensor(
    quaternion_xyzw_q1_30: torch.Tensor, world_vector: torch.Tensor
) -> torch.Tensor:
    if (
        quaternion_xyzw_q1_30.dtype != torch.int64
        or world_vector.dtype != torch.int64
        or quaternion_xyzw_q1_30.shape[:-1] != world_vector.shape[:-1]
        or quaternion_xyzw_q1_30.shape[-1] != 4
        or world_vector.shape[-1] != 3
    ):
        raise ValueError("root-local transform requires matching int64 [...,4] and [...,3]")
    if torch.any(torch.abs(quaternion_xyzw_q1_30) > Q1_30_ONE):
        raise OverflowError("quaternion is outside Q1.30")
    x, y, z, w = quaternion_xyzw_q1_30.unbind(dim=-1)

    def coefficient(value: torch.Tensor) -> torch.Tensor:
        return round_div_ties_even_tensor(value, Q1_30_ONE)

    def diagonal(left: torch.Tensor, right: torch.Tensor) -> torch.Tensor:
        return Q1_30_ONE - coefficient(2 * (left * left + right * right))

    rows = (
        (
            diagonal(y, z),
            coefficient(2 * (x * y - z * w)),
            coefficient(2 * (x * z + y * w)),
        ),
        (
            coefficient(2 * (x * y + z * w)),
            diagonal(x, z),
            coefficient(2 * (y * z - x * w)),
        ),
        (
            coefficient(2 * (x * z - y * w)),
            coefficient(2 * (y * z + x * w)),
            diagonal(x, y),
        ),
    )
    output = []
    for column in range(3):
        dot = sum(rows[row][column] * world_vector[..., row] for row in range(3))
        output.append(round_div_ties_even_tensor(dot, Q1_30_ONE))
    return torch.stack(output, dim=-1)


def precompute_flat_command_schedules(
    run_root: bytes,
    episode_ordinals: Iterable[int],
    vector_slots: Iterable[int],
) -> torch.Tensor:
    ordinals = list(episode_ordinals)
    slots = list(vector_slots)
    if len(ordinals) != len(slots):
        raise ValueError("episode ordinals and vector slots must have equal length")
    schedules = [
        flat_locomotion_command_schedule(
            derive_purpose_seed(run_root, ordinal, slot, "randomization.command")
        )
        for ordinal, slot in zip(ordinals, slots, strict=True)
    ]
    return torch.tensor(schedules, dtype=torch.int64, device="cpu")


def precompute_command_schedules(
    run_root: bytes,
    episode_ordinals: Iterable[int],
    vector_slots: Iterable[int],
    profile_id: str,
) -> torch.Tensor:
    ordinals = list(episode_ordinals)
    slots = list(vector_slots)
    if len(ordinals) != len(slots):
        raise ValueError("episode ordinals and vector slots must have equal length")
    if profile_id == FLAT_LOCOMOTION_PROFILE_ID:
        return precompute_flat_command_schedules(run_root, ordinals, slots)
    if profile_id in {
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    }:
        schedule = biomechanics_forward_start_stop_command_schedule_v2()
        return torch.tensor(
            [schedule for _ in ordinals], dtype=torch.int64, device="cpu"
        )
    if profile_id not in {
        CURRICULUM_LOCOMOTION_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
    }:
        raise ValueError(f"profile has no engine command schedule: {profile_id}")
    schedules = [
        curriculum_locomotion_command_schedule(
            derive_purpose_seed(run_root, ordinal, slot, "randomization.command"),
            0 if profile_id == BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID else ordinal,
        )
        for ordinal, slot in zip(ordinals, slots, strict=True)
    ]
    return torch.tensor(schedules, dtype=torch.int64, device="cpu")


def locomotion_reward_q16_tensor(
    *,
    quaternion_xyzw_q1_30: torch.Tensor,
    root_height_micrometres: torch.Tensor,
    target_root_height_micrometres: int,
    local_linear_velocity_raw: torch.Tensor,
    local_angular_velocity_raw: torch.Tensor,
    vertical_velocity_raw: torch.Tensor,
    command_raw: torch.Tensor,
    effort_sum_raw: torch.Tensor,
    applied_action_raw: torch.Tensor,
    previous_applied_action_raw: torch.Tensor,
    contacting_foot_slip_sum_raw: torch.Tensor,
    contacting_foot_count: torch.Tensor,
    fell: torch.Tensor,
    profile_id: str = FLAT_LOCOMOTION_PROFILE_ID,
    effort_normalization: int = 23 * 4 * 150_000_000,
    action_rate_normalization: int = 23 * 2_000_000,
) -> tuple[torch.Tensor, torch.Tensor]:
    """Evaluate the canonical locomotion components and weighted Q16 total."""
    planar_error = torch.abs(local_linear_velocity_raw[:, 0] - command_raw[:, 0]) + torch.abs(
        local_linear_velocity_raw[:, 2] - command_raw[:, 1]
    )
    forward_start_stop_v2 = profile_id in {
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    }
    forward_start_stop_v3 = (
        profile_id
        in {
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
        }
    )
    curriculum = profile_id in {
        CURRICULUM_LOCOMOTION_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    }
    if not is_locomotion_profile(profile_id):
        raise ValueError(f"unsupported locomotion reward profile: {profile_id}")
    planar_normalization = (
        500_000 if forward_start_stop_v2 else (2_500_000 if curriculum else 6_500_000)
    )
    yaw_normalization = (
        500_000 if forward_start_stop_v2 else (1_500_000 if curriculum else 3_000_000)
    )
    yaw_error = torch.abs(local_angular_velocity_raw[:, 1] - command_raw[:, 2])
    if forward_start_stop_v3:
        planar = dense_tracking_q16_tensor(planar_error, planar_normalization)
        yaw = dense_tracking_q16_tensor(yaw_error, yaw_normalization)
    else:
        planar = 65_536 - ratio_q16_tensor(planar_error, planar_normalization)
        yaw = 65_536 - ratio_q16_tensor(yaw_error, yaw_normalization)
    if curriculum and not forward_start_stop_v3:
        planar = round_div_ties_even_tensor(planar * planar, 65_536)
        yaw = round_div_ties_even_tensor(yaw * yaw, 65_536)
    x = quaternion_xyzw_q1_30[:, 0]
    z = quaternion_xyzw_q1_30[:, 2]
    tilt_reduction = round_div_ties_even_tensor(2 * (x * x + z * z), Q1_30_ONE)
    upright_q30 = torch.clamp(Q1_30_ONE - tilt_reduction, 0, Q1_30_ONE)
    upright = ratio_q16_tensor(upright_q30, Q1_30_ONE)
    height_error = ratio_q16_tensor(
        torch.abs(root_height_micrometres - target_root_height_micrometres),
        400_000 if curriculum else 600_000,
    )
    if forward_start_stop_v2:
        upright = 65_536 - upright
        height = height_error
    else:
        height = 65_536 - height_error
    vertical = ratio_q16_tensor(
        torch.abs(vertical_velocity_raw), 2_000_000 if curriculum else 3_000_000
    )
    roll_pitch = ratio_q16_tensor(
        torch.abs(local_angular_velocity_raw[:, 0])
        + torch.abs(local_angular_velocity_raw[:, 2]),
        4_000_000 if curriculum else 6_000_000,
    )
    effort = ratio_q16_tensor(effort_sum_raw, effort_normalization)
    action_rate = ratio_q16_tensor(
        torch.sum(torch.abs(applied_action_raw - previous_applied_action_raw), dim=-1),
        action_rate_normalization,
    )
    slip_denominator = contacting_foot_count * (2_000_000 if curriculum else 4_000_000)
    slip = torch.zeros_like(contacting_foot_slip_sum_raw)
    # The denominator is per environment; calculate the exact ratio without float math.
    nonzero = slip_denominator > 0
    if torch.any(nonzero):
        bounded = torch.minimum(contacting_foot_slip_sum_raw[nonzero], slip_denominator[nonzero])
        numerator = bounded * 65_536
        quotient = torch.div(numerator, slip_denominator[nonzero], rounding_mode="floor")
        remainder = torch.remainder(numerator, slip_denominator[nonzero])
        increment = (remainder * 2 > slip_denominator[nonzero]) | (
            (remainder * 2 == slip_denominator[nonzero]) & (quotient % 2 == 1)
        )
        slip[nonzero] = quotient + increment.to(torch.int64)
    fall = fell.to(torch.int64) * 65_536
    base_components = (planar, yaw, upright, height, vertical, roll_pitch, effort, action_rate, slip)
    if curriculum:
        moving = torch.any(command_raw != 0, dim=-1)
        support = torch.where(
            (moving & (contacting_foot_count == 1))
            | (~moving & (contacting_foot_count == 2)),
            65_536,
            0,
        ).to(torch.int64)
        components = torch.stack((*base_components, support, fall), dim=-1)
        coefficients_q16 = (
            BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3
            if forward_start_stop_v3
            else (
                BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2
                if forward_start_stop_v2
                else CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16
            )
        )
    else:
        components = torch.stack((*base_components, fall), dim=-1)
        coefficients_q16 = LOCOMOTION_REWARD_COEFFICIENTS_Q16
    coefficients = torch.tensor(
        coefficients_q16,
        dtype=torch.int64,
        device=components.device,
    )
    total = torch.sum(round_div_ties_even_tensor(components * coefficients, 65_536), dim=-1)
    return components, total


def bounded_standing_reward_q16_tensor(
    *,
    quaternion_xyzw_q1_30: torch.Tensor,
    root_height_micrometres: torch.Tensor,
    target_root_height_micrometres: int,
    joint_position_raw: torch.Tensor,
    linear_velocity_raw: torch.Tensor,
    angular_velocity_raw: torch.Tensor,
    effort_sum_raw: torch.Tensor,
    applied_action_raw: torch.Tensor,
    previous_applied_action_raw: torch.Tensor,
    contacting_foot_slip_sum_raw: torch.Tensor,
    contacting_foot_count: torch.Tensor,
    fell: torch.Tensor,
    joint_reference_raw: torch.Tensor | None = None,
    joint_pose_normalization: int = 23 * 1_500_000,
    effort_normalization: int = 23 * 4 * 150_000_000,
    action_rate_normalization: int = 23 * 2_000_000,
) -> tuple[torch.Tensor, torch.Tensor]:
    """Evaluate the engine-owned bounded standing V2 reward in exact Q16."""
    x = quaternion_xyzw_q1_30[:, 0]
    z = quaternion_xyzw_q1_30[:, 2]
    tilt_reduction = round_div_ties_even_tensor(2 * (x * x + z * z), Q1_30_ONE)
    upright_q30 = torch.clamp(Q1_30_ONE - tilt_reduction, 0, Q1_30_ONE)
    upright = ratio_q16_tensor(upright_q30, Q1_30_ONE)
    height = 65_536 - ratio_q16_tensor(
        torch.abs(root_height_micrometres - target_root_height_micrometres),
        600_000,
    )
    reference = (
        torch.zeros_like(joint_position_raw)
        if joint_reference_raw is None
        else joint_reference_raw
    )
    if reference.shape != joint_position_raw.shape:
        raise ValueError("standing joint reference shape mismatch")
    pose = 65_536 - ratio_q16_tensor(
        torch.sum(torch.abs(joint_position_raw - reference), dim=-1),
        joint_pose_normalization,
    )
    linear_motion = ratio_q16_tensor(
        torch.sum(torch.abs(linear_velocity_raw), dim=-1),
        3 * 3_000_000,
    )
    angular_motion = ratio_q16_tensor(
        torch.sum(torch.abs(angular_velocity_raw), dim=-1),
        3 * 6_000_000,
    )
    root_motion = torch.maximum(linear_motion, angular_motion)
    effort = ratio_q16_tensor(effort_sum_raw, effort_normalization)
    action_rate = ratio_q16_tensor(
        torch.sum(torch.abs(applied_action_raw - previous_applied_action_raw), dim=-1),
        action_rate_normalization,
    )
    slip_denominator = contacting_foot_count * 4_000_000
    slip = torch.zeros_like(contacting_foot_slip_sum_raw)
    nonzero = slip_denominator > 0
    if torch.any(nonzero):
        bounded = torch.minimum(contacting_foot_slip_sum_raw[nonzero], slip_denominator[nonzero])
        numerator = bounded * 65_536
        quotient = torch.div(numerator, slip_denominator[nonzero], rounding_mode="floor")
        remainder = torch.remainder(numerator, slip_denominator[nonzero])
        increment = (remainder * 2 > slip_denominator[nonzero]) | (
            (remainder * 2 == slip_denominator[nonzero]) & (quotient % 2 == 1)
        )
        slip[nonzero] = quotient + increment.to(torch.int64)
    fall = fell.to(torch.int64) * 65_536
    components = torch.stack(
        (upright, height, pose, root_motion, effort, action_rate, slip, fall),
        dim=-1,
    )
    coefficients = torch.tensor(
        BOUNDED_STANDING_REWARD_COEFFICIENTS_Q16,
        dtype=torch.int64,
        device=components.device,
    )
    total = torch.sum(
        round_div_ties_even_tensor(components * coefficients, 65_536), dim=-1
    )
    return components, total


if ISAAC_LAB_AVAILABLE:

    @configclass
    class NextEngineHumanoidDirectEnvCfg(DirectRLEnvCfg):
        decimation = 4
        episode_length_s = 20.0
        action_space = 23
        observation_space = 84
        state_space = 0
        environment_profile_id = CURRICULUM_LOCOMOTION_PROFILE_ID
        episode_ordinal_start = 0
        run_root_hex = "00" * 32
        sim = sim_utils.SimulationCfg(dt=1.0 / 240.0, render_interval=4)
        scene = InteractiveSceneCfg(num_envs=4_096, env_spacing=3.0, replicate_physics=True)
        asset = ArticulationCfg(
            prim_path="/World/envs/env_.*/Humanoid",
            spawn=sim_utils.UsdFileCfg(
                usd_path=os.environ.get("NEXTENGINE_HUMANOID_USD", ""),
                activate_contact_sensors=True,
                articulation_props=sim_utils.ArticulationRootPropertiesCfg(
                    enabled_self_collisions=False,
                    solver_position_iteration_count=8,
                    solver_velocity_iteration_count=2,
                ),
            ),
            actuators={
                "engine_effort": ImplicitActuatorCfg(
                    joint_names_expr=[".*"],
                    stiffness=0.0,
                    damping=0.0,
                    effort_limit_sim=None,
                    velocity_limit_sim=None,
                )
            },
        )
        feet = ContactSensorCfg(
            prim_path="/World/envs/env_.*/Humanoid/Bodies/body_.*_ankle_roll",
            update_period=0.0,
            history_length=1,
            track_air_time=True,
        )
        standing_reward_coefficients = (1.0,) * 8


    class NextEngineHumanoidDirectEnv(DirectRLEnv):
        cfg: NextEngineHumanoidDirectEnvCfg

        def __init__(self, cfg: NextEngineHumanoidDirectEnvCfg, descriptor_path: str, **kwargs: Any):
            descriptor = json.loads(Path(descriptor_path).read_text(encoding="utf-8"))
            self._biomechanics_standing = is_biomechanics_profile(
                cfg.environment_profile_id
            )
            if self._biomechanics_standing:
                if cfg.environment_profile_id == BIOMECHANICS_STANDING_PROFILE_ID:
                    validate_biomechanics_standing_descriptor(descriptor)
                else:
                    validate_biomechanics_forward_start_stop_descriptor(descriptor)
                self.profile = select_biomechanics_training_profile(
                    descriptor, cfg.environment_profile_id
                )
                self._contact_projections = _contact_body_projections(descriptor)
                self._contact_layout = _contact_pair_layout(self._contact_projections)
                humanoid_path = Path(cfg.asset.spawn.usd_path).resolve()
                validate_translation_bundle(
                    descriptor,
                    humanoid_path.with_name("translation-manifest.json"),
                    humanoid_usd_path=humanoid_path,
                    ground_usd_path=humanoid_path.with_name("ground.usda"),
                )
                cfg.asset.spawn.articulation_props.enabled_self_collisions = True
                cfg.asset.spawn.articulation_props.solver_velocity_iteration_count = 4
            else:
                validate_descriptor(descriptor)
                self.profile = select_environment_profile(
                    descriptor, cfg.environment_profile_id
                )
            self.descriptor = descriptor
            self.authored_ground_clearance_m = (
                require_compatible_authored_ground_clearance(descriptor)
            )
            self.authored_root_height_micrometres = authored_root_height_micrometres(
                descriptor
            )
            self.isaac_actuator_limits = isaac_actuator_limits_from_descriptor(descriptor)
            actuator_cfg = cfg.asset.actuators["engine_effort"]
            actuator_cfg.effort_limit_sim = self.isaac_actuator_limits["effort_limit_sim"]
            actuator_cfg.velocity_limit_sim = self.isaac_actuator_limits["velocity_limit_sim"]
            self.run_root = bytes.fromhex(cfg.run_root_hex)
            if len(self.run_root) != 32:
                raise ValueError("run_root_hex must encode 32 bytes")
            cfg.episode_length_s = self.profile["maximum_episode_steps"] / 60.0
            self._action = torch.zeros((cfg.scene.num_envs, 23), dtype=torch.int64)
            self._previous_action = torch.zeros_like(self._action)
            self._previous_effort = torch.zeros_like(self._action)
            self._effort_sum = torch.zeros(cfg.scene.num_envs, dtype=torch.int64)
            self._applied_target = torch.zeros_like(self._action)
            self._previous_applied_target = torch.zeros_like(self._action)
            self._reference_target = torch.zeros_like(self._action)
            self._positive_work = torch.zeros_like(self._action)
            self._joint_safety_violation = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            if self._biomechanics_standing:
                pair_count = len(self._contact_layout["sensor"])
                self._contact_continuity = torch.zeros(
                    (cfg.scene.num_envs, pair_count), dtype=torch.int64
                )
                self._self_collision_pair_violation = torch.zeros(
                    (cfg.scene.num_envs, pair_count), dtype=torch.bool
                )
                self._self_collision_pair_impulse = torch.zeros(
                    (cfg.scene.num_envs, pair_count, 3), dtype=torch.int64
                )
                self._self_collision_pair_separation = torch.full(
                    (cfg.scene.num_envs, pair_count),
                    torch.iinfo(torch.int64).max,
                    dtype=torch.int64,
                )
                self._contact_physics_substeps_started = 0
                self._hard_impact_violation = torch.zeros(
                    cfg.scene.num_envs, dtype=torch.bool
                )
                self._self_collision_violation = torch.zeros_like(
                    self._hard_impact_violation
                )
                self._forbidden_contact_violation = torch.zeros_like(
                    self._hard_impact_violation
                )
                self.last_step_failure_joint_safety = torch.zeros_like(
                    self._hard_impact_violation
                )
                self._joint_safety_violation_mask = torch.zeros_like(
                    self._action, dtype=torch.bool
                )
                self._joint_observed_violation_mask = torch.zeros_like(
                    self._joint_safety_violation_mask
                )
                self._joint_infeasible_mask = torch.zeros_like(
                    self._joint_safety_violation_mask
                )
                self._joint_peak_absolute_velocity = torch.zeros_like(self._action)
                self.last_step_joint_safety_mask = torch.zeros_like(
                    self._joint_safety_violation_mask
                )
                self.last_step_joint_observed_violation_mask = torch.zeros_like(
                    self._joint_safety_violation_mask
                )
                self.last_step_joint_infeasible_mask = torch.zeros_like(
                    self._joint_safety_violation_mask
                )
                self.last_step_joint_position = torch.zeros_like(self._action)
                self.last_step_joint_velocity = torch.zeros_like(self._action)
                self.last_step_joint_peak_absolute_velocity = torch.zeros_like(
                    self._action
                )
                self.last_step_root_position = torch.zeros(
                    (cfg.scene.num_envs, 3), dtype=torch.int64
                )
                self.last_step_root_quaternion = torch.zeros(
                    (cfg.scene.num_envs, 4), dtype=torch.int64
                )
                self.last_step_root_linear_velocity = torch.zeros(
                    (cfg.scene.num_envs, 3), dtype=torch.int64
                )
                self.last_step_root_angular_velocity = torch.zeros(
                    (cfg.scene.num_envs, 3), dtype=torch.int64
                )
                self.last_step_failure_hard_impact = torch.zeros_like(
                    self._hard_impact_violation
                )
                self.last_step_failure_self_collision = torch.zeros_like(
                    self._hard_impact_violation
                )
                self.last_step_failure_forbidden_contact = torch.zeros_like(
                    self._hard_impact_violation
                )
                self.last_step_failure_fall = torch.zeros_like(
                    self._hard_impact_violation
                )
                self.last_step_self_collision_pair_mask = torch.zeros_like(
                    self._self_collision_pair_violation
                )
                self.last_step_contact_pair_impulse = torch.zeros(
                    (cfg.scene.num_envs, pair_count, 3), dtype=torch.int64
                )
                self.last_step_contact_pair_separation = torch.full(
                    (cfg.scene.num_envs, pair_count),
                    torch.iinfo(torch.int64).max,
                    dtype=torch.int64,
                )
            if not 0 <= cfg.episode_ordinal_start < 2**63:
                raise ValueError("episode_ordinal_start must be a non-negative 63-bit integer")
            self._episode_ordinals = torch.full(
                (cfg.scene.num_envs,),
                cfg.episode_ordinal_start - 1,
                dtype=torch.int64,
            )
            self._command_schedule = torch.zeros((cfg.scene.num_envs, 1_201, 3), dtype=torch.int64)
            component_count = len(self.profile["reward_components"])
            self.reward_components_q16 = torch.zeros(
                (cfg.scene.num_envs, component_count), dtype=torch.int64
            )
            self._episode_reward_sum = torch.zeros(
                cfg.scene.num_envs, dtype=torch.float32
            )
            self._episode_component_sums = torch.zeros(
                (cfg.scene.num_envs, component_count), dtype=torch.float32
            )
            super().__init__(cfg, **kwargs)
            canonical_joint_names = [
                isaac_prim_name(actuator["joint_id"])
                for actuator in self.descriptor["actuators"]
            ]
            self._canonical_joint_ids, resolved_joint_names = self.robot.find_joints(
                canonical_joint_names, preserve_order=True
            )
            if resolved_joint_names != canonical_joint_names:
                raise RuntimeError("Isaac articulation does not match canonical actuator order")
            self._foot_body_ids, _ = self.robot.find_bodies(
                self.feet.body_names, preserve_order=True
            )
            if len(self._foot_body_ids) != 2:
                raise RuntimeError("descriptor declares exactly two foot effectors")
            if self._biomechanics_standing:
                self._load_biomechanics_control_tensors()
                self._setup_contact_pair_view()
            self._capture_reset_defaults()
            for name in (
                "_action",
                "_previous_action",
                "_previous_effort",
                "_effort_sum",
                "_applied_target",
                "_previous_applied_target",
                "_reference_target",
                "_positive_work",
                "_joint_safety_violation",
                "_episode_ordinals",
                "_command_schedule",
                "reward_components_q16",
                "_episode_reward_sum",
                "_episode_component_sums",
            ):
                setattr(self, name, getattr(self, name).to(self.device))
            if self._biomechanics_standing:
                for name in (
                    "_contact_continuity",
                    "_self_collision_pair_violation",
                    "_self_collision_pair_impulse",
                    "_self_collision_pair_separation",
                    "_hard_impact_violation",
                    "_self_collision_violation",
                    "_forbidden_contact_violation",
                    "last_step_failure_joint_safety",
                    "_joint_safety_violation_mask",
                    "_joint_observed_violation_mask",
                    "_joint_infeasible_mask",
                    "_joint_peak_absolute_velocity",
                    "last_step_joint_safety_mask",
                    "last_step_joint_observed_violation_mask",
                    "last_step_joint_infeasible_mask",
                    "last_step_joint_position",
                    "last_step_joint_velocity",
                    "last_step_joint_peak_absolute_velocity",
                    "last_step_root_position",
                    "last_step_root_quaternion",
                    "last_step_root_linear_velocity",
                    "last_step_root_angular_velocity",
                    "last_step_failure_hard_impact",
                    "last_step_failure_self_collision",
                    "last_step_failure_forbidden_contact",
                    "last_step_failure_fall",
                    "last_step_self_collision_pair_mask",
                    "last_step_contact_pair_impulse",
                    "last_step_contact_pair_separation",
                ):
                    setattr(self, name, getattr(self, name).to(self.device))

        def _load_biomechanics_control_tensors(self) -> None:
            joint_by_id = {
                joint["joint_id"]: joint for joint in self.descriptor["joints"]
            }
            records = self.descriptor["actuators"]
            self._actuator_joint_ids = tuple(
                str(record["joint_id"]) for record in records
            )
            joints = [joint_by_id[joint_id] for joint_id in self._actuator_joint_ids]

            def tensor(values: list[int]) -> torch.Tensor:
                return torch.tensor(values, dtype=torch.int64, device=self.device)

            self._neutral_target = tensor(
                [joint["neutral_position_microradians"] for joint in joints]
            )
            self._soft_minimum = tensor(
                [joint["soft_limit_microradians"][0] for joint in joints]
            )
            self._soft_maximum = tensor(
                [joint["soft_limit_microradians"][1] for joint in joints]
            )
            self._hard_minimum = tensor(
                [joint["hard_limit_microradians"][0] for joint in joints]
            )
            self._hard_maximum = tensor(
                [joint["hard_limit_microradians"][1] for joint in joints]
            )
            self._maximum_velocity = tensor(
                [joint["maximum_velocity_microradians_per_second"] for joint in joints]
            )
            self._joint_position_scale = torch.maximum(
                torch.abs(self._soft_minimum), torch.abs(self._soft_maximum)
            )
            self._residual_scale = tensor(
                [record["residual_scale_microradians"] for record in records]
            )
            self._residual_scale_multiplier_q16 = int(
                self.profile["action"].get("residual_scale_multiplier_q16", 65_536)
            )
            self._target_delta = tensor(
                [
                    max(
                        abs(record["target_delta_microradians_per_motor_tick"][0]),
                        abs(record["target_delta_microradians_per_motor_tick"][1]),
                    )
                    for record in records
                ]
            )
            self._stiffness_q16 = tensor([record["stiffness_q16"] for record in records])
            self._damping_q16 = tensor([record["damping_q16"] for record in records])
            self._effort_minimum = tensor(
                [record["effort_micronewton_metres"][0] for record in records]
            )
            self._effort_maximum = tensor(
                [record["effort_micronewton_metres"][1] for record in records]
            )
            self._maximum_effort_rate = tensor(
                [
                    record["maximum_effort_rate_micronewton_metres_per_second"]
                    for record in records
                ]
            )
            self._maximum_power = tensor(
                [record["maximum_power_microwatts"] for record in records]
            )
            self._maximum_positive_work = tensor(
                [
                    record["maximum_positive_work_microjoules_per_motor_tick"]
                    for record in records
                ]
            )
            normalizations = self.profile["reward_normalizations"]
            self._pose_normalization = int(
                normalizations.get(
                    "joint_pose_soft_rom_span_sum_microradians",
                    sum(
                        joint["soft_limit_microradians"][1]
                        - joint["soft_limit_microradians"][0]
                        for joint in joints
                    ),
                )
            )
            self._effort_normalization = int(
                normalizations["applied_effort_per_motor_tick_micronewton_metres"]
            )
            self._target_rate_normalization = int(
                normalizations["applied_target_rate_microradians_per_motor_tick"]
            )

        def _setup_contact_pair_view(self) -> None:
            body_patterns = [
                f"/World/envs/env_*/Humanoid/Bodies/{isaac_prim_name(body_id)}"
                for body_id, _, _ in self._contact_projections
            ]
            filters = [
                ["/World/ground/Collision", *body_patterns]
                for _ in body_patterns
            ]
            pair_count = len(self._contact_layout["sensor"])
            self._contact_pair_view = (
                self.feet._physics_sim_view.create_rigid_contact_view(
                    body_patterns,
                    filter_patterns=filters,
                    max_contact_data_count=16 * pair_count * self.num_envs,
                )
            )
            expected_sensor_count = len(body_patterns) * self.num_envs
            expected_filter_count = 1 + len(body_patterns)
            if (
                self._contact_pair_view.sensor_count != expected_sensor_count
                or self._contact_pair_view.filter_count != expected_filter_count
            ):
                raise RuntimeError("Isaac rigid-contact tensor layout mismatch")
            self._contact_pair_sensor = torch.tensor(
                self._contact_layout["sensor"], dtype=torch.int64, device=self.device
            )
            self._contact_pair_filter = torch.tensor(
                self._contact_layout["filter"], dtype=torch.int64, device=self.device
            )
            self._contact_pair_hard_limit = torch.tensor(
                self._contact_layout["hard_limit"],
                dtype=torch.int64,
                device=self.device,
            )
            self._contact_pair_self = torch.tensor(
                self._contact_layout["self_contact"],
                dtype=torch.bool,
                device=self.device,
            )
            self._contact_pair_primary_role = torch.tensor(
                self._contact_layout["primary_role"],
                dtype=torch.int64,
                device=self.device,
            )
            self.contact_pair_ids = tuple(self._contact_layout["pair_id"])

        def _capture_reset_defaults(self) -> None:
            """Close the engine-authored pose into deterministic reset templates."""
            root_template = torch.tensor(
                isaac_root_state_from_descriptor(self.descriptor),
                dtype=torch.float32,
                device=self.device,
            ).unsqueeze(0)
            initial_root_state = self.robot.data.root_state_w.clone()
            initial_root_state[:, :3] -= self.scene.env_origins
            root_pose_deviation = torch.max(
                torch.abs(
                    initial_root_state[:, :7]
                    - root_template[:, :7].expand_as(initial_root_state[:, :7])
                )
            )
            initial_root_velocity = torch.max(torch.abs(initial_root_state[:, 7:]))
            maximum_startup_deviation = (
                0.012 if is_frozen_stage0_v1_legacy_pose(self.descriptor) else 0.01
            )
            if root_pose_deviation > maximum_startup_deviation:
                raise RuntimeError(
                    "initial PhysX root pose does not match the engine descriptor: "
                    f"maximum deviation {float(root_pose_deviation.item())}, "
                    f"limit {maximum_startup_deviation}"
                )

            joint_template = self.robot.data.default_joint_pos[0].unsqueeze(0)
            joint_deviation = torch.max(
                torch.abs(
                    self.robot.data.joint_pos
                    - joint_template.expand_as(self.robot.data.joint_pos)
                )
            )

            self.reset_template_initial_deviation = {
                "root_pose": float(root_pose_deviation.item()),
                "root_velocity": float(initial_root_velocity.item()),
                "joint_position": float(joint_deviation.item()),
            }
            self.robot.data.default_root_state.copy_(
                root_template.expand_as(self.robot.data.default_root_state)
            )
            self.robot.data.default_joint_pos.copy_(
                joint_template.expand_as(self.robot.data.default_joint_pos)
            )
            self.robot.data.default_joint_vel.zero_()

        def _setup_scene(self) -> None:
            self.robot = Articulation(self.cfg.asset)
            self.feet = ContactSensor(self.cfg.feet)
            self.scene.articulations["humanoid"] = self.robot
            self.scene.sensors["feet"] = self.feet
            if self._biomechanics_standing:
                ground_path = str(
                    Path(self.cfg.asset.spawn.usd_path).with_name("ground.usda")
                )
                ground = sim_utils.UsdFileCfg(usd_path=ground_path)
                ground.func("/World/ground", ground)
            else:
                sim_utils.spawn_ground_plane("/World/ground", sim_utils.GroundPlaneCfg())
            self.scene.clone_environments(copy_from_source=False)

        def _pre_physics_step(self, actions: torch.Tensor) -> None:
            self.extras.pop("log", None)
            if actions.shape != (self.num_envs, 23) or not torch.isfinite(actions).all():
                raise ValueError("invalid canonical action batch")
            if self._biomechanics_standing:
                if self._contact_physics_substeps_started != 0:
                    raise RuntimeError(
                        "contact substep cadence did not close at motor boundary"
                    )
                _, quaternion_raw, _, _, _ = self._canonical_facts()
                root_relative_isaac = (
                    self.robot.data.root_pos_w - self.scene.env_origins
                )
                root_position_engine = engine_vector_from_isaac_tensor(
                    root_relative_isaac
                )
                root_linear_world_raw = torch.round(
                    engine_vector_from_isaac_tensor(self.robot.data.root_lin_vel_w)
                    * 1_000_000
                ).to(torch.int64)
                root_angular_world_raw = torch.round(
                    engine_vector_from_isaac_tensor(self.robot.data.root_ang_vel_w)
                    * 1_000_000
                ).to(torch.int64)
                root_forward = torch.round(
                    root_position_engine[:, 2] * 1_000_000
                ).to(torch.int64)
                reference = biomechanics_procedural_standing_targets_tensor(
                    actuator_joint_ids=self._actuator_joint_ids,
                    neutral_targets_microradians=self._neutral_target,
                    root_quaternion_xyzw_q1_30=quaternion_raw,
                    root_forward_micrometres=root_forward,
                    root_angular_velocity_microradians_per_second=root_angular_world_raw,
                    root_forward_velocity_micrometres_per_second=root_linear_world_raw[:, 2],
                    include_root_forward_position_feedback=(
                        self.cfg.environment_profile_id
                        not in {
                            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
                            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
                        }
                    ),
                )
                self._previous_action.copy_(self._action)
                self._action.copy_(
                    torch.round(
                        torch.clamp(actions, -1.0, 1.0) * float(Q1_30_ONE)
                    ).to(torch.int64)
                )
                residual = round_div_ties_even_tensor(
                    self._action * self._residual_scale, Q1_30_ONE
                )
                residual = round_div_ties_even_tensor(
                    residual * self._residual_scale_multiplier_q16, 65_536
                )
                candidate = torch.minimum(
                    torch.maximum(reference + residual, self._soft_minimum),
                    self._soft_maximum,
                )
                self._reference_target.copy_(reference)
                self._previous_applied_target.copy_(self._applied_target)
                self._applied_target.copy_(
                    torch.minimum(
                        torch.maximum(
                            candidate, self._applied_target - self._target_delta
                        ),
                        self._applied_target + self._target_delta,
                    )
                )
                self._effort_sum.zero_()
                self._positive_work.zero_()
                self._joint_safety_violation.zero_()
                self._joint_safety_violation_mask.zero_()
                self._joint_observed_violation_mask.zero_()
                self._joint_infeasible_mask.zero_()
                self._joint_peak_absolute_velocity.zero_()
                self._hard_impact_violation.zero_()
                self._self_collision_violation.zero_()
                self._forbidden_contact_violation.zero_()
                self._self_collision_pair_violation.zero_()
                self._self_collision_pair_impulse.zero_()
                self._self_collision_pair_separation.fill_(
                    torch.iinfo(torch.int64).max
                )
                return
            self._previous_action.copy_(self._action)
            self._action.copy_(
                torch.round(torch.clamp(actions, -1.0, 1.0) * 1_000_000).to(torch.int64)
            )
            self._effort_sum.zero_()

        def _apply_action(self) -> None:
            if self._biomechanics_standing:
                if self._contact_physics_substeps_started > 0:
                    self._consume_biomechanics_contact_substep()
                if self._contact_physics_substeps_started >= self.cfg.decimation:
                    raise RuntimeError("too many contact substeps before motor commit")
            joint_position = self.robot.data.joint_pos[:, self._canonical_joint_ids]
            joint_velocity = self.robot.data.joint_vel[:, self._canonical_joint_ids]
            require_finite_tensor("joint_pos", joint_position)
            require_finite_tensor("joint_vel", joint_velocity)
            position = torch.round(joint_position * 1_000_000).to(torch.int64)
            velocity = torch.round(joint_velocity * 1_000_000).to(torch.int64)
            if self._biomechanics_standing:
                observed_violation = (
                    (
                        position
                        < self._hard_minimum
                        - OBSERVED_HARD_ROM_QUANTIZATION_TOLERANCE_MICRORADIANS
                    )
                    | (
                        position
                        > self._hard_maximum
                        + OBSERVED_HARD_ROM_QUANTIZATION_TOLERANCE_MICRORADIANS
                    )
                    | (
                        torch.abs(velocity)
                        > self._maximum_velocity
                        + OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND
                    )
                )
                effort, next_work, infeasible = biomechanics_fixed_pd_safety_tensor(
                    target_microradians=self._applied_target,
                    position_microradians=position,
                    velocity_microradians_per_second=velocity,
                    previous_effort_micronewton_metres=self._previous_effort,
                    used_positive_work_microjoules=self._positive_work,
                    stiffness_q16=self._stiffness_q16,
                    damping_q16=self._damping_q16,
                    effort_minimum=self._effort_minimum,
                    effort_maximum=self._effort_maximum,
                    maximum_effort_rate_per_second=self._maximum_effort_rate,
                    maximum_power_microwatts=self._maximum_power,
                    maximum_positive_work_microjoules=self._maximum_positive_work,
                )
                self._joint_observed_violation_mask |= observed_violation
                self._joint_infeasible_mask |= infeasible
                self._joint_peak_absolute_velocity.copy_(
                    torch.maximum(
                        self._joint_peak_absolute_velocity,
                        torch.abs(velocity),
                    )
                )
                self._joint_safety_violation_mask |= observed_violation | infeasible
                self._joint_safety_violation |= torch.any(
                    self._joint_safety_violation_mask, dim=-1
                )
                published = torch.where(
                    self._joint_safety_violation[:, None],
                    torch.zeros_like(effort),
                    effort,
                )
                self._previous_effort.copy_(
                    torch.where(
                        self._joint_safety_violation[:, None],
                        self._previous_effort,
                        effort,
                    )
                )
                self._positive_work.copy_(
                    torch.where(
                        self._joint_safety_violation[:, None],
                        self._positive_work,
                        next_work,
                    )
                )
                self._effort_sum.add_(torch.sum(torch.abs(published), dim=-1))
                self.robot.set_joint_effort_target(
                    published.to(torch.float32) / 1_000_000.0,
                    joint_ids=self._canonical_joint_ids,
                )
                self._contact_physics_substeps_started += 1
                return
            effort, _ = fixed_pd_tensor(self._action, position, velocity, self._previous_effort)
            self._previous_effort.copy_(effort)
            self._effort_sum.add_(torch.sum(torch.abs(effort), dim=-1))
            self.robot.set_joint_effort_target(
                effort.to(torch.float32) / 1_000_000.0,
                joint_ids=self._canonical_joint_ids,
            )

        def _canonical_facts(self) -> tuple[torch.Tensor, ...]:
            data = self.robot.data
            require_finite_tensor("root_quat_w", data.root_quat_w)
            require_finite_tensor("root_lin_vel_w", data.root_lin_vel_w)
            require_finite_tensor("root_ang_vel_w", data.root_ang_vel_w)
            require_finite_tensor("foot_net_forces_w", self.feet.data.net_forces_w)
            quaternion = engine_quaternion_xyzw_from_isaac_wxyz_tensor(data.root_quat_w)
            quaternion_raw = torch.round(quaternion * Q1_30_ONE).to(torch.int64)
            linear_world = engine_vector_from_isaac_tensor(data.root_lin_vel_w)
            angular_world = engine_vector_from_isaac_tensor(data.root_ang_vel_w)
            linear_raw = torch.round(linear_world * 1_000_000).to(torch.int64)
            angular_raw = torch.round(angular_world * 1_000_000).to(torch.int64)
            if is_locomotion_profile(self.profile["profile_id"]):
                linear_raw = rotate_world_to_root_local_q1_30_tensor(quaternion_raw, linear_raw)
                angular_raw = rotate_world_to_root_local_q1_30_tensor(quaternion_raw, angular_raw)
            contacts = torch.linalg.vector_norm(self.feet.data.net_forces_w, dim=-1) > 1.0e-6
            if contacts.shape[-1] != 2:
                raise RuntimeError("descriptor declares exactly two foot effectors")
            return quaternion, quaternion_raw, linear_raw, angular_raw, contacts

        def _current_command(self) -> torch.Tensor:
            if is_standing_profile(self.profile["profile_id"]):
                return torch.zeros((self.num_envs, 3), dtype=torch.int64, device=self.device)
            ticks = torch.clamp(self.episode_length_buf, 0, 1_200).to(torch.int64)
            envs = torch.arange(self.num_envs, device=self.device)
            return self._command_schedule[envs, ticks]

        def _consume_biomechanics_contact_substep(self) -> None:
            body_count = len(self._contact_projections)
            filter_count = 1 + body_count
            force = self._contact_pair_view.get_contact_force_matrix(dt=self.physics_dt)
            expected_force_shape = (
                body_count * self.num_envs,
                filter_count,
                3,
            )
            if force.shape != expected_force_shape:
                raise RuntimeError("Isaac contact force matrix changed layout")
            force_by_env = force.reshape(
                body_count, self.num_envs, filter_count, 3
            ).permute(1, 0, 2, 3)
            selected_force = force_by_env[
                :, self._contact_pair_sensor, self._contact_pair_filter
            ]
            impulse = torch.round(
                selected_force.to(torch.float64) * self.physics_dt * MICRO_SCALE
            ).to(torch.int64)
            contact_data = self._contact_pair_view.get_contact_data(dt=self.physics_dt)
            minimum_separation = _minimum_contact_separation_tensor(
                contact_data[3], contact_data[4], contact_data[5]
            )
            separation_by_env = minimum_separation.reshape(
                body_count, self.num_envs, filter_count
            ).permute(1, 0, 2)
            selected_separation = separation_by_env[
                :, self._contact_pair_sensor, self._contact_pair_filter
            ]
            self.last_step_contact_pair_impulse.copy_(impulse)
            self.last_step_contact_pair_separation.copy_(selected_separation)
            classification = _classify_contact_pairs_tensor(
                impulse,
                selected_separation,
                self._contact_continuity,
                self._contact_pair_hard_limit,
                self._contact_pair_self,
                self._contact_pair_primary_role,
            )
            self._contact_continuity.copy_(classification["continuity"])
            self._hard_impact_violation |= torch.any(
                classification["hard_impact"], dim=-1
            )
            self._self_collision_violation |= torch.any(
                classification["self_collision"], dim=-1
            )
            new_self_collision = classification["self_collision"]
            self._self_collision_pair_impulse.copy_(
                torch.where(
                    new_self_collision[..., None],
                    impulse,
                    self._self_collision_pair_impulse,
                )
            )
            self._self_collision_pair_separation.copy_(
                torch.where(
                    new_self_collision,
                    selected_separation,
                    self._self_collision_pair_separation,
                )
            )
            self._self_collision_pair_violation |= classification["self_collision"]
            self._forbidden_contact_violation |= torch.any(
                classification["forbidden_locomotion"], dim=-1
            )

        def _get_observations(self) -> dict[str, torch.Tensor]:
            quaternion, _, linear_raw, angular_raw, contacts = self._canonical_facts()
            command = self._current_command()
            if self._biomechanics_standing:
                joint_position_raw = torch.round(
                    self.robot.data.joint_pos[:, self._canonical_joint_ids]
                    * 1_000_000
                ).to(torch.int64)
                joint_velocity_raw = torch.round(
                    self.robot.data.joint_vel[:, self._canonical_joint_ids]
                    * 1_000_000
                ).to(torch.int64)
                policy = torch.cat(
                    (
                        quaternion,
                        linear_raw.to(torch.float32) / 2_000_000.0,
                        angular_raw.to(torch.float32) / 2_000_000.0,
                        joint_position_raw.to(torch.float32)
                        / self._joint_position_scale.to(torch.float32),
                        joint_velocity_raw.to(torch.float32)
                        / self._maximum_velocity.to(torch.float32),
                        self._applied_target.to(torch.float32)
                        / self._joint_position_scale.to(torch.float32),
                        command.to(torch.float32) / 1_000_000.0,
                        contacts.to(torch.float32),
                    ),
                    dim=-1,
                )
                if policy.shape[-1] != 84:
                    raise RuntimeError("biomechanics standing observation width mismatch")
                require_finite_tensor("biomechanics_standing_observation", policy)
                return {"policy": policy}
            policy = torch.cat(
                (
                    quaternion,
                    linear_raw.to(torch.float32) / 1_000_000.0,
                    angular_raw.to(torch.float32) / 1_000_000.0,
                    self.robot.data.joint_pos[:, self._canonical_joint_ids],
                    self.robot.data.joint_vel[:, self._canonical_joint_ids],
                    self._action.to(torch.float32) / 1_000_000.0,
                    command.to(torch.float32) / 1_000_000.0,
                    contacts.to(torch.float32),
                ),
                dim=-1,
            )
            if policy.shape[-1] != 84:
                raise RuntimeError("canonical observation width mismatch")
            require_finite_tensor("policy_observation", policy)
            return {"policy": policy}

        def _get_rewards(self) -> torch.Tensor:
            if self.profile["profile_id"] == STANDING_PROFILE_ID:
                return self._standing_rewards()
            _, quaternion_raw, linear_raw, angular_raw, contacts = self._canonical_facts()
            require_finite_tensor("root_pos_w", self.robot.data.root_pos_w)
            require_finite_tensor("body_lin_vel_w", self.robot.data.body_lin_vel_w)
            root_height = torch.round(self.robot.data.root_pos_w[:, 2] * 1_000_000).to(torch.int64)
            vertical_velocity = torch.round(
                self.robot.data.root_lin_vel_w[:, 2] * 1_000_000
            ).to(torch.int64)
            fallen = root_height <= fall_height_threshold_micrometres(
                self.profile["profile_id"]
            )
            foot_velocity = engine_vector_from_isaac_tensor(
                self.robot.data.body_lin_vel_w[:, self._foot_body_ids, :]
            )
            slip_per_foot = torch.round(
                (torch.abs(foot_velocity[..., 0]) + torch.abs(foot_velocity[..., 2])) * 1_000_000
            ).to(torch.int64)
            slip_sum = torch.sum(slip_per_foot * contacts.to(torch.int64), dim=-1)
            if self.profile["profile_id"] in {
                BOUNDED_STANDING_PROFILE_ID,
                BIOMECHANICS_STANDING_PROFILE_ID,
            }:
                joint_position_raw = torch.round(
                    self.robot.data.joint_pos[:, self._canonical_joint_ids] * 1_000_000
                ).to(torch.int64)
                components, total = bounded_standing_reward_q16_tensor(
                    quaternion_xyzw_q1_30=quaternion_raw,
                    root_height_micrometres=root_height,
                    target_root_height_micrometres=self.authored_root_height_micrometres,
                    joint_position_raw=joint_position_raw,
                    linear_velocity_raw=linear_raw,
                    angular_velocity_raw=angular_raw,
                    effort_sum_raw=self._effort_sum,
                    applied_action_raw=(
                        self._applied_target
                        if self._biomechanics_standing
                        else self._action
                    ),
                    previous_applied_action_raw=(
                        self._previous_applied_target
                        if self._biomechanics_standing
                        else self._previous_action
                    ),
                    contacting_foot_slip_sum_raw=slip_sum,
                    contacting_foot_count=torch.sum(contacts.to(torch.int64), dim=-1),
                    fell=fallen,
                    joint_reference_raw=(
                        self._reference_target
                        if self._biomechanics_standing
                        else None
                    ),
                    joint_pose_normalization=(
                        self._pose_normalization
                        if self._biomechanics_standing
                        else 23 * 1_500_000
                    ),
                    effort_normalization=(
                        self._effort_normalization
                        if self._biomechanics_standing
                        else 23 * 4 * 150_000_000
                    ),
                    action_rate_normalization=(
                        self._target_rate_normalization
                        if self._biomechanics_standing
                        else 23 * 2_000_000
                    ),
                )
                self.reward_components_q16.copy_(components)
                reward = total.to(torch.float32) / 65_536.0
                self._accumulate_episode_metrics(
                    reward, components.to(torch.float32) / 65_536.0
                )
                return reward
            components, total = locomotion_reward_q16_tensor(
                quaternion_xyzw_q1_30=quaternion_raw,
                root_height_micrometres=root_height,
                target_root_height_micrometres=self.authored_root_height_micrometres,
                local_linear_velocity_raw=linear_raw,
                local_angular_velocity_raw=angular_raw,
                vertical_velocity_raw=vertical_velocity,
                command_raw=self._current_command(),
                effort_sum_raw=self._effort_sum,
                applied_action_raw=(
                    self._applied_target if self._biomechanics_standing else self._action
                ),
                previous_applied_action_raw=(
                    self._previous_applied_target
                    if self._biomechanics_standing
                    else self._previous_action
                ),
                contacting_foot_slip_sum_raw=slip_sum,
                contacting_foot_count=torch.sum(contacts.to(torch.int64), dim=-1),
                fell=fallen,
                profile_id=self.profile["profile_id"],
                effort_normalization=(
                    self._effort_normalization
                    if self._biomechanics_standing
                    else 23 * 4 * 150_000_000
                ),
                action_rate_normalization=(
                    self._target_rate_normalization
                    if self._biomechanics_standing
                    else 23 * 2_000_000
                ),
            )
            self.reward_components_q16.copy_(components)
            reward = total.to(torch.float32) / 65_536.0
            component_values = components.to(torch.float32) / 65_536.0
            self._accumulate_episode_metrics(reward, component_values)
            return reward

        def _standing_rewards(self) -> torch.Tensor:
            data = self.robot.data
            require_finite_tensor("standing_root_quat_w", data.root_quat_w)
            require_finite_tensor("standing_root_pos_w", data.root_pos_w)
            require_finite_tensor("standing_joint_pos", data.joint_pos)
            require_finite_tensor("standing_root_lin_vel_w", data.root_lin_vel_w)
            require_finite_tensor("standing_root_ang_vel_w", data.root_ang_vel_w)
            upright = torch.abs(data.root_quat_w[:, 0])
            root_height = -torch.abs(
                data.root_pos_w[:, 2]
                - self.authored_root_height_micrometres / 1_000_000.0
            )
            standing_pose = -torch.sum(torch.abs(data.joint_pos), dim=-1)
            velocity = -torch.sum(torch.abs(data.root_lin_vel_w), dim=-1) - torch.sum(
                torch.abs(data.root_ang_vel_w), dim=-1
            )
            effort = -torch.sum(torch.abs(self._previous_effort), dim=-1).to(torch.float32)
            action_rate = -torch.sum(torch.abs(self._action - self._previous_action), dim=-1).to(
                torch.float32
            )
            foot_slip = torch.zeros_like(upright)
            fall = -(data.root_pos_w[:, 2] <= 0.25).to(torch.float32)
            components = torch.stack(
                (upright, root_height, standing_pose, velocity, effort, action_rate, foot_slip, fall),
                dim=-1,
            )
            coefficients = torch.tensor(
                self.cfg.standing_reward_coefficients, device=self.device
            )
            reward = torch.sum(components * coefficients, dim=-1)
            require_finite_tensor("standing_reward", reward)
            self._accumulate_episode_metrics(reward, components)
            return reward

        def _accumulate_episode_metrics(
            self, reward: torch.Tensor, components: torch.Tensor
        ) -> None:
            require_finite_tensor("reward", reward)
            require_finite_tensor("reward_components", components)
            self._episode_reward_sum.add_(reward)
            self._episode_component_sums.add_(components)

        def _get_dones(self) -> tuple[torch.Tensor, torch.Tensor]:
            root = self.robot.data.root_pos_w
            require_finite_tensor("done_root_pos_w", root)
            threshold = (
                fall_height_threshold_micrometres(self.profile["profile_id"])
                / 1_000_000.0
            )
            fall = root[:, 2] <= threshold
            terminated = fall.clone()
            if self._biomechanics_standing:
                if self._contact_physics_substeps_started != self.cfg.decimation:
                    raise RuntimeError(
                        "motor commit does not contain four contact substeps"
                    )
                self._consume_biomechanics_contact_substep()
                self._contact_physics_substeps_started = 0
                quaternion = engine_quaternion_xyzw_from_isaac_wxyz_tensor(
                    self.robot.data.root_quat_w
                )
                root_relative = self.robot.data.root_pos_w - self.scene.env_origins
                self.last_step_root_position.copy_(
                    torch.round(
                        engine_vector_from_isaac_tensor(root_relative) * 1_000_000
                    ).to(torch.int64)
                )
                self.last_step_root_quaternion.copy_(
                    torch.round(quaternion * Q1_30_ONE).to(torch.int64)
                )
                self.last_step_root_linear_velocity.copy_(
                    torch.round(
                        engine_vector_from_isaac_tensor(
                            self.robot.data.root_lin_vel_w
                        )
                        * 1_000_000
                    ).to(torch.int64)
                )
                self.last_step_root_angular_velocity.copy_(
                    torch.round(
                        engine_vector_from_isaac_tensor(
                            self.robot.data.root_ang_vel_w
                        )
                        * 1_000_000
                    ).to(torch.int64)
                )
                up_y = 1.0 - 2.0 * (
                    quaternion[:, 0] * quaternion[:, 0]
                    + quaternion[:, 2] * quaternion[:, 2]
                )
                planar = root[:, :2] - self.scene.env_origins[:, :2]
                terminated |= (
                    (up_y <= 0.5)
                    | (torch.abs(planar[:, 0]) >= 90.0)
                    | (torch.abs(planar[:, 1]) >= 90.0)
                    | self._joint_safety_violation
                    | self._hard_impact_violation
                    | self._self_collision_violation
                    | self._forbidden_contact_violation
                )
                self.last_step_failure_joint_safety.copy_(
                    self._joint_safety_violation
                )
                self.last_step_joint_safety_mask.copy_(
                    self._joint_safety_violation_mask
                )
                self.last_step_joint_observed_violation_mask.copy_(
                    self._joint_observed_violation_mask
                )
                self.last_step_joint_infeasible_mask.copy_(
                    self._joint_infeasible_mask
                )
                self.last_step_joint_position.copy_(
                    torch.round(
                        self.robot.data.joint_pos[:, self._canonical_joint_ids]
                        * 1_000_000
                    ).to(torch.int64)
                )
                self.last_step_joint_velocity.copy_(
                    torch.round(
                        self.robot.data.joint_vel[:, self._canonical_joint_ids]
                        * 1_000_000
                    ).to(torch.int64)
                )
                self.last_step_joint_peak_absolute_velocity.copy_(
                    self._joint_peak_absolute_velocity
                )
                self.last_step_failure_hard_impact.copy_(
                    self._hard_impact_violation
                )
                self.last_step_failure_self_collision.copy_(
                    self._self_collision_violation
                )
                self.last_step_failure_forbidden_contact.copy_(
                    self._forbidden_contact_violation
                )
                self.last_step_failure_fall.copy_(fall)
                self.last_step_self_collision_pair_mask.copy_(
                    self._self_collision_pair_violation
                )
                self.last_step_contact_pair_impulse.copy_(
                    self._self_collision_pair_impulse
                )
                self.last_step_contact_pair_separation.copy_(
                    self._self_collision_pair_separation
                )
            if is_locomotion_profile(self.profile["profile_id"]):
                displacement = root[:, :2] - self.scene.env_origins[:, :2]
                terminated |= (torch.abs(displacement[:, 0]) >= 90.0) | (
                    torch.abs(displacement[:, 1]) >= 90.0
                )
            timed_out = self.episode_length_buf >= self.max_episode_length - 1
            return terminated, timed_out

        def _reset_idx(self, env_ids: torch.Tensor | None) -> None:
            if env_ids is None:
                env_ids = self.robot._ALL_INDICES
            completed = self.episode_length_buf[env_ids] > 0
            if torch.any(completed):
                completed_ids = env_ids[completed]
                lengths = self.episode_length_buf[completed_ids].to(torch.float32)
                log: dict[str, torch.Tensor] = {
                    "Episode/return": self._episode_reward_sum[completed_ids],
                    "Episode/length": lengths,
                    "Episode/terminated": self.reset_terminated[completed_ids].to(
                        torch.float32
                    ),
                    "Episode/truncated": self.reset_time_outs[completed_ids].to(
                        torch.float32
                    ),
                }
                for index, component in enumerate(self.profile["reward_components"]):
                    component_id = component["component_id"].removeprefix("reward.")
                    log[f"Episode_Component/{component_id}"] = (
                        self._episode_component_sums[completed_ids, index] / lengths
                    )
                if self._biomechanics_standing:
                    for reason in (
                        "joint_safety",
                        "hard_impact",
                        "self_collision",
                        "forbidden_contact",
                        "fall",
                    ):
                        log[f"Episode_Termination/{reason}"] = getattr(
                            self, f"last_step_failure_{reason}"
                        )[completed_ids].to(torch.float32)
                self.extras["log"] = log
            super()._reset_idx(env_ids)
            root_state = self.robot.data.default_root_state[env_ids].clone()
            root_state[:, :3] += self.scene.env_origins[env_ids]
            joint_position = self.robot.data.default_joint_pos[env_ids].clone()
            joint_velocity = self.robot.data.default_joint_vel[env_ids].clone()
            self.robot.write_root_state_to_sim(root_state, env_ids)
            self.robot.write_joint_state_to_sim(
                joint_position, joint_velocity, env_ids=env_ids
            )
            self.robot.set_joint_effort_target(
                torch.zeros_like(joint_position), env_ids=env_ids
            )
            self._action[env_ids] = 0
            self._previous_action[env_ids] = 0
            self._previous_effort[env_ids] = 0
            self._effort_sum[env_ids] = 0
            self._applied_target[env_ids] = 0
            self._previous_applied_target[env_ids] = 0
            self._reference_target[env_ids] = 0
            self._positive_work[env_ids] = 0
            self._joint_safety_violation[env_ids] = False
            if self._biomechanics_standing:
                self._joint_safety_violation_mask[env_ids] = False
                self._joint_observed_violation_mask[env_ids] = False
                self._joint_infeasible_mask[env_ids] = False
                self._joint_peak_absolute_velocity[env_ids] = 0
                self._applied_target[env_ids] = self._neutral_target
                self._previous_applied_target[env_ids] = self._neutral_target
                self._reference_target[env_ids] = self._neutral_target
                self._contact_continuity[env_ids] = 0
                self._self_collision_pair_violation[env_ids] = False
                self._hard_impact_violation[env_ids] = False
                self._self_collision_violation[env_ids] = False
                self._forbidden_contact_violation[env_ids] = False
            self._episode_reward_sum[env_ids] = 0.0
            self._episode_component_sums[env_ids] = 0.0
            self._episode_ordinals[env_ids] += 1
            if is_locomotion_profile(self.profile["profile_id"]):
                ids = env_ids.detach().cpu().tolist()
                ordinals = self._episode_ordinals[env_ids].detach().cpu().tolist()
                schedules = precompute_command_schedules(
                    self.run_root,
                    ordinals,
                    ids,
                    self.profile["profile_id"],
                )
                self._command_schedule[env_ids] = schedules.to(self.device)

else:

    class NextEngineHumanoidDirectEnvCfg:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")


    class NextEngineHumanoidDirectEnv:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")
