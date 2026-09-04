from __future__ import annotations

import hashlib
import json
from collections.abc import Iterable
from pathlib import Path
from typing import Any

from next_lab.biomechanics_material_lineage import (
    BIOMECHANICS_TRANSLATOR_ID_V2,
    validate_current_material_lineage,
)

SEED_DOMAIN = b"nextengine.motor-episode-seed.v1\0"
COMMAND_COUNTER_DOMAIN = b"nextengine.motor-command-counter.v1\0"
COMMAND_SCHEDULE_DOMAIN = b"nextengine.motor-command-schedule.v1\0"
STANDING_PROFILE_ID = "nextengine.motor.env.humanoid-standing.v1"
BOUNDED_STANDING_PROFILE_ID = "nextengine.motor.env.humanoid-standing.v2"
BIOMECHANICS_STANDING_PROFILE_ID = (
    "nextengine.motor.env.humanoid-biomechanics-standing.v2"
)
FLAT_LOCOMOTION_PROFILE_ID = "nextengine.motor.env.humanoid-flat-command.v1"
CURRICULUM_LOCOMOTION_PROFILE_ID = (
    "nextengine.motor.env.humanoid-flat-command-curriculum.v2"
)
CURRENT_TRANSLATOR_VERSION = "nextengine.isaac-usda-translator.v3"
LEGACY_TRANSLATOR_PROFILE_ID = "nextengine.isaac-translator.v2"
BIOMECHANICS_TRANSLATOR_ID = "nextengine.isaac.biomechanics-mirror.v1"
PHYSICS_HZ = 240
MOTOR_HZ = 60
SUBSTEPS = 4
Q1_30_ONE = 1 << 30
TARGET_CLAMPED = 1 << 0
EFFORT_CLAMPED = 1 << 1
RATE_CLAMPED = 1 << 2

STANDING_REWARD_COMPONENT_IDS = (
    "reward.upright",
    "reward.root-height-tracking",
    "reward.standing-pose-tracking",
    "reward.velocity-penalty",
    "reward.effort-penalty",
    "reward.action-rate-penalty",
    "reward.foot-slip-penalty",
    "reward.fall-terminal",
)
BOUNDED_STANDING_REWARD_COMPONENT_IDS = (
    "reward.upright-yaw-invariant",
    "reward.root-height-tracking",
    "reward.standing-pose-tracking-normalized",
    "reward.root-motion-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-action-rate-cost",
    "reward.contacting-foot-tangential-slip-cost",
    "reward.fall-component",
)
BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS = (
    "reward.upright-yaw-invariant",
    "reward.root-height-tracking",
    "reward.procedural-standing-pose-tracking-normalized",
    "reward.root-motion-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-target-rate-cost",
    "reward.contacting-sole-tangential-slip-cost",
    "reward.fall-component",
)
LOCOMOTION_REWARD_COMPONENT_IDS = (
    "reward.planar-command-tracking",
    "reward.yaw-rate-tracking",
    "reward.upright-yaw-invariant",
    "reward.root-height-tracking",
    "reward.vertical-velocity-cost",
    "reward.roll-pitch-rate-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-action-rate-cost",
    "reward.contacting-foot-tangential-slip-cost",
    "reward.fall-component",
)
CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS = (
    *LOCOMOTION_REWARD_COMPONENT_IDS[:-1],
    "reward.command-conditioned-support",
    "reward.fall-component",
)

FLAT_COMMAND_PROFILE_V1 = {
    "warmup_ticks": 60,
    "segment_ticks": 120,
    "episode_ticks": 1_200,
    "mode_weights_basis_points": (2_500, 3_500, 2_000, 2_000),
    "right_velocity_range_raw": (-2_000_000, 2_000_000),
    "forward_velocity_range_raw": (-1_500_000, 3_000_000),
    "yaw_rate_range_raw": (-1_500_000, 1_500_000),
    "linear_rate_limit_raw_per_second_squared": 3_000_000,
    "yaw_rate_limit_raw_per_second_squared": 1_500_000,
}

CURRICULUM_COMMAND_STAGES_V2 = (
    {
        "first_episode_ordinal": 0,
        "warmup_ticks": 120,
        "segment_ticks": 240,
        "episode_ticks": 1_200,
        "mode_weights_basis_points": (4_000, 6_000, 0, 0),
        "right_velocity_range_raw": (0, 0),
        "forward_velocity_range_raw": (0, 750_000),
        "yaw_rate_range_raw": (0, 0),
        "linear_rate_limit_raw_per_second_squared": 1_000_000,
        "yaw_rate_limit_raw_per_second_squared": 500_000,
    },
    {
        "first_episode_ordinal": 32,
        "warmup_ticks": 90,
        "segment_ticks": 180,
        "episode_ticks": 1_200,
        "mode_weights_basis_points": (2_500, 5_500, 1_500, 500),
        "right_velocity_range_raw": (-350_000, 350_000),
        "forward_velocity_range_raw": (0, 1_250_000),
        "yaw_rate_range_raw": (-600_000, 600_000),
        "linear_rate_limit_raw_per_second_squared": 1_500_000,
        "yaw_rate_limit_raw_per_second_squared": 750_000,
    },
    {
        "first_episode_ordinal": 96,
        "warmup_ticks": 60,
        "segment_ticks": 120,
        "episode_ticks": 1_200,
        "mode_weights_basis_points": (1_500, 4_500, 1_500, 2_500),
        "right_velocity_range_raw": (-1_000_000, 1_000_000),
        "forward_velocity_range_raw": (-500_000, 2_000_000),
        "yaw_rate_range_raw": (-1_000_000, 1_000_000),
        "linear_rate_limit_raw_per_second_squared": 2_000_000,
        "yaw_rate_limit_raw_per_second_squared": 1_000_000,
    },
)


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("expected a JSON object")
    return value


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def derive_purpose_seed(
    run_root: bytes,
    episode_ordinal: int,
    vector_slot: int,
    purpose_id: str,
) -> bytes:
    if len(run_root) != 32:
        raise ValueError("run_root must contain exactly 32 bytes")
    if not 0 <= episode_ordinal <= (1 << 64) - 1:
        raise ValueError("episode_ordinal is outside u64")
    if not 0 <= vector_slot <= (1 << 32) - 1:
        raise ValueError("vector_slot is outside u32")
    purpose = purpose_id.encode("utf-8")
    preimage = b"".join(
        (
            SEED_DOMAIN,
            run_root,
            episode_ordinal.to_bytes(8, "little"),
            vector_slot.to_bytes(4, "little"),
            len(purpose).to_bytes(4, "little"),
            purpose,
        )
    )
    return hashlib.sha256(preimage).digest()


def round_div_ties_even(numerator: int, denominator: int) -> int:
    if denominator <= 0:
        raise ValueError("denominator must be positive")
    absolute_quotient, remainder = divmod(abs(numerator), denominator)
    if remainder * 2 > denominator or (
        remainder * 2 == denominator and absolute_quotient % 2 == 1
    ):
        absolute_quotient += 1
    return absolute_quotient if numerator >= 0 else -absolute_quotient


def _command_counter(command_seed: bytes, segment_index: int, lane: int) -> int:
    if len(command_seed) != 32:
        raise ValueError("command_seed must contain exactly 32 bytes")
    payload = b"".join(
        (
            COMMAND_COUNTER_DOMAIN,
            command_seed,
            segment_index.to_bytes(8, "little"),
            lane.to_bytes(4, "little"),
        )
    )
    return int.from_bytes(hashlib.sha256(payload).digest()[:8], "little")


def _bounded_counter(
    command_seed: bytes, segment_index: int, lane: int, minimum: int, maximum: int
) -> int:
    return minimum + _command_counter(command_seed, segment_index, lane) % (
        maximum - minimum + 1
    )


def _move_towards(current: int, target: int, maximum_delta: int) -> int:
    if current < target:
        return min(current + maximum_delta, target)
    return max(current - maximum_delta, target)


def flat_locomotion_command_schedule(command_seed: bytes) -> list[tuple[int, int, int]]:
    """Build the canonical 1,201-sample command schedule using engine integer rules."""
    return _command_schedule(command_seed, FLAT_COMMAND_PROFILE_V1)


def curriculum_locomotion_command_schedule(
    command_seed: bytes, episode_ordinal: int
) -> list[tuple[int, int, int]]:
    if not 0 <= episode_ordinal <= (1 << 64) - 1:
        raise ValueError("episode_ordinal is outside u64")
    stage = next(
        stage
        for stage in reversed(CURRICULUM_COMMAND_STAGES_V2)
        if episode_ordinal >= stage["first_episode_ordinal"]
    )
    return _command_schedule(command_seed, stage)


def _command_schedule(
    command_seed: bytes, profile: dict[str, Any]
) -> list[tuple[int, int, int]]:
    schedule = [(0, 0, 0)]
    target = (0, 0, 0)
    warmup_ticks = int(profile["warmup_ticks"])
    segment_ticks = int(profile["segment_ticks"])
    episode_ticks = int(profile["episode_ticks"])
    weights = tuple(int(value) for value in profile["mode_weights_basis_points"])
    ranges = (
        tuple(int(value) for value in profile["right_velocity_range_raw"]),
        tuple(int(value) for value in profile["forward_velocity_range_raw"]),
        tuple(int(value) for value in profile["yaw_rate_range_raw"]),
    )
    linear_delta = int(profile["linear_rate_limit_raw_per_second_squared"]) // MOTOR_HZ
    yaw_delta = int(profile["yaw_rate_limit_raw_per_second_squared"]) // MOTOR_HZ
    thresholds = (weights[0], weights[0] + weights[1], sum(weights[:3]))
    for tick in range(1, episode_ticks + 1):
        if tick < warmup_ticks:
            target = (0, 0, 0)
        elif tick == warmup_ticks or (tick - warmup_ticks) % segment_ticks == 0:
            segment = (tick - warmup_ticks) // segment_ticks
            selector = _command_counter(command_seed, segment, 0) % 10_000
            right = _bounded_counter(command_seed, segment, 1, *ranges[0])
            forward = _bounded_counter(command_seed, segment, 2, *ranges[1])
            yaw = _bounded_counter(command_seed, segment, 3, *ranges[2])
            if selector < thresholds[0]:
                target = (0, 0, 0)
            elif selector < thresholds[1]:
                target = (right, forward, 0)
            elif selector < thresholds[2]:
                target = (0, 0, yaw)
            else:
                target = (right, forward, yaw)
        previous = schedule[-1]
        schedule.append(
            (
                _move_towards(previous[0], target[0], linear_delta),
                _move_towards(previous[1], target[1], linear_delta),
                _move_towards(previous[2], target[2], yaw_delta),
            )
        )
    return schedule


def command_schedule_hash(schedule: Iterable[Iterable[int]]) -> str:
    values = [tuple(command) for command in schedule]
    payload = bytearray(COMMAND_SCHEDULE_DOMAIN)
    payload.extend(len(values).to_bytes(8, "little"))
    for command in values:
        if len(command) != 3:
            raise ValueError("command must have three components")
        for value in command:
            payload.extend(int(value).to_bytes(8, "little", signed=True))
    return hashlib.sha256(payload).hexdigest()


def rotate_world_to_root_local_q1_30(
    quaternion_xyzw_q1_30: Iterable[int], world_vector: Iterable[int]
) -> tuple[int, int, int]:
    quaternion = tuple(int(value) for value in quaternion_xyzw_q1_30)
    vector = tuple(int(value) for value in world_vector)
    if len(quaternion) != 4 or len(vector) != 3:
        raise ValueError("root-local transform requires quaternion[4] and vector[3]")
    if any(value < -Q1_30_ONE or value > Q1_30_ONE for value in quaternion):
        raise OverflowError("quaternion is outside Q1.30")
    x, y, z, w = quaternion
    coefficient = lambda value: round_div_ties_even(value, Q1_30_ONE)
    diagonal = lambda left, right: (
        Q1_30_ONE - coefficient(2 * (left * left + right * right))
    )
    matrix = (
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
    result = tuple(
        round_div_ties_even(
            sum(matrix[row][column] * vector[row] for row in range(3)), Q1_30_ONE
        )
        for column in range(3)
    )
    if any(value < -(1 << 63) or value > (1 << 63) - 1 for value in result):
        raise OverflowError("root-local vector is outside i64")
    return result


def fixed_pd_substep(
    *,
    neutral_position_microradians: int,
    residual_target_microradians: int,
    position_microradians: int,
    velocity_microradians_per_second: int,
    stiffness_q16: int,
    damping_q16: int,
    limit_min_microradians: int,
    limit_max_microradians: int,
    maximum_effort_micronewton_metres: int,
    maximum_effort_rate_micronewton_metres_per_second: int,
    previous_effort_micronewton_metres: int,
) -> tuple[int, int]:
    flags = 0
    requested_target = neutral_position_microradians + residual_target_microradians
    target = min(max(requested_target, limit_min_microradians), limit_max_microradians)
    if target != requested_target:
        flags |= TARGET_CLAMPED
    proportional = round_div_ties_even(
        stiffness_q16 * (target - position_microradians), 65_536
    )
    damping = round_div_ties_even(
        damping_q16 * velocity_microradians_per_second, 65_536
    )
    requested_effort = proportional - damping
    effort = min(
        max(requested_effort, -maximum_effort_micronewton_metres),
        maximum_effort_micronewton_metres,
    )
    if effort != requested_effort:
        flags |= EFFORT_CLAMPED
    maximum_delta = round_div_ties_even(
        maximum_effort_rate_micronewton_metres_per_second, PHYSICS_HZ
    )
    rate_limited = min(
        max(effort, previous_effort_micronewton_metres - maximum_delta),
        previous_effort_micronewton_metres + maximum_delta,
    )
    if rate_limited != effort:
        flags |= RATE_CLAMPED
    return rate_limited, flags


def select_environment_profile(
    descriptor: dict[str, Any], profile_id: str
) -> dict[str, Any]:
    validate_descriptor(descriptor)
    profiles = descriptor["environment_profiles"]
    for profile in profiles:
        if profile["profile_id"] == profile_id:
            return profile
    raise ValueError(f"unsupported environment profile: {profile_id}")


def select_biomechanics_standing_profile(
    descriptor: dict[str, Any], profile_id: str
) -> dict[str, Any]:
    validate_biomechanics_standing_descriptor(descriptor)
    if profile_id != BIOMECHANICS_STANDING_PROFILE_ID:
        raise ValueError(f"unsupported biomechanics standing profile: {profile_id}")
    profiles = descriptor.get("environment_profiles")
    if not isinstance(profiles, list) or len(profiles) != 1:
        raise ValueError("biomechanics standing profile closure mismatch")
    profile = profiles[0]
    if not isinstance(profile, dict) or profile.get("profile_id") != profile_id:
        raise ValueError("biomechanics standing identity mismatch")
    validate_biomechanics_standing_profile(profile)
    return profile


def validate_biomechanics_standing_profile(profile: dict[str, Any]) -> None:
    for field in (
        "manifest_hash",
        "observation_layout_hash",
        "action_layout_hash",
        "command_schedule_profile_hash",
        "reward_profile_hash",
        "translator_version_hash",
        "termination_profile_hash",
        "rng_derivation_profile_hash",
        "correspondence_profile_hash",
    ):
        _require_hash(profile.get(field), field)
    if (
        profile.get("profile_id") != BIOMECHANICS_STANDING_PROFILE_ID
        or profile.get("observation_layout_id")
        != "nextengine.motor.observation.humanoid-biomechanics-standing.v2"
        or profile.get("action_layout_id")
        != "nextengine.motor.action.humanoid-biomechanics-standing-residual.v2"
        or profile.get("maximum_episode_steps") != 3_600
        or profile.get("velocity_frame") != "world"
        or profile.get("ground_half_extent_metres") != 50
        or profile.get("target_root_height_micrometres") != 943_500
        or profile.get("command_profile") != {"kind": "zero"}
    ):
        raise ValueError("biomechanics standing semantics mismatch")
    reference = profile.get("standing_reference")
    if reference != {
        "profile_id": "nextengine.motor.procedural-standing.v1",
        "knee_target_microradians": 100_000,
        "ankle_bias_microradians": -140_000,
        "root_target_forward_micrometres": 0,
    }:
        raise ValueError("biomechanics standing reference mismatch")
    observation = profile.get("observation")
    action = profile.get("action")
    if (
        not isinstance(observation, dict)
        or observation.get("channel_count") != 84
        or observation.get("contacts")
        != ["contact.left-sole", "contact.right-sole"]
        or not isinstance(action, dict)
        or action.get("channel_count") != 23
    ):
        raise ValueError("biomechanics standing tensor layout mismatch")
    components = profile.get("reward_components")
    if not isinstance(components, list) or tuple(
        component.get("component_id") for component in components
    ) != BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS:
        raise ValueError("biomechanics standing reward order mismatch")
    if any(
        component.get("minimum_raw") != 0
        or component.get("maximum_raw") != 65_536
        for component in components
    ):
        raise ValueError("biomechanics standing reward bounds mismatch")
    normalizations = profile.get("reward_normalizations")
    expected_normalization_keys = {
        "root_height_micrometres",
        "joint_pose_soft_rom_span_sum_microradians",
        "root_linear_l1_micrometres_per_second",
        "root_angular_l1_microradians_per_second",
        "applied_effort_per_motor_tick_micronewton_metres",
        "applied_target_rate_microradians_per_motor_tick",
        "contacting_sole_slip_micrometres_per_second",
    }
    if (
        not isinstance(normalizations, dict)
        or set(normalizations) != expected_normalization_keys
        or any(
            not isinstance(value, int) or isinstance(value, bool) or value <= 0
            for value in normalizations.values()
        )
    ):
        raise ValueError("biomechanics standing reward normalization mismatch")
    termination = profile.get("termination")
    if not isinstance(termination, dict) or (
        termination.get("pelvis_height_micrometres_inclusive"),
        termination.get("root_tilt_degrees_inclusive"),
        termination.get("world_bound_micrometres_inclusive"),
        termination.get("timeout_ticks"),
    ) != (450_000, 60, 90_000_000, 3_600):
        raise ValueError("biomechanics standing termination mismatch")


def validate_golden(
    golden: dict[str, Any], descriptor_bytes: bytes | None = None
) -> None:
    if golden.get("schema_version") != 2:
        raise ValueError("unsupported motor mirror golden version")
    actuator_ids = require_unique_strings(
        golden.get("ordered_actuator_ids"), "actuators"
    )
    if len(actuator_ids) != golden.get("pd_channel_count") or len(actuator_ids) != 23:
        raise ValueError("golden actuator count mismatch")
    reward_profiles = golden.get("reward_component_ids")
    if not isinstance(reward_profiles, dict):
        raise ValueError("golden reward profiles are missing")
    expected_rewards = {
        STANDING_PROFILE_ID: STANDING_REWARD_COMPONENT_IDS,
        BOUNDED_STANDING_PROFILE_ID: BOUNDED_STANDING_REWARD_COMPONENT_IDS,
        FLAT_LOCOMOTION_PROFILE_ID: LOCOMOTION_REWARD_COMPONENT_IDS,
        CURRICULUM_LOCOMOTION_PROFILE_ID: CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS,
    }
    for profile_id, expected in expected_rewards.items():
        actual = require_unique_strings(
            reward_profiles.get(profile_id), f"rewards:{profile_id}"
        )
        if tuple(actual) != expected:
            raise ValueError(f"reward order mismatch for {profile_id}")
    manifests = golden.get("profile_manifest_hashes")
    if not isinstance(manifests, dict) or set(manifests) != set(expected_rewards):
        raise ValueError("profile manifest hashes do not close")
    for value in manifests.values():
        _require_hash(value, "profile manifest")

    seed_input = golden["seed_input"]
    run_root = bytes.fromhex(seed_input["run_root"])
    seeds: dict[str, bytes] = {}
    for record in golden["purpose_seeds"]:
        actual = derive_purpose_seed(
            run_root,
            seed_input["episode_ordinal"],
            seed_input["vector_slot"],
            record["purpose_id"],
        )
        if actual.hex() != record["seed"]:
            raise ValueError(f"seed mismatch for {record['purpose_id']}")
        seeds[record["purpose_id"]] = actual
    if "randomization.command" not in seeds:
        raise ValueError("command seed is missing")
    schedule = flat_locomotion_command_schedule(seeds["randomization.command"])
    command_golden = golden["command_schedule"]
    if command_schedule_hash(schedule) != command_golden["sha256"]:
        raise ValueError("command schedule hash mismatch")
    for sample in command_golden["samples"]:
        if list(schedule[sample["tick"]]) != sample["command_raw"]:
            raise ValueError(
                f"command schedule sample mismatch at tick {sample['tick']}"
            )
    curriculum_golden = golden["curriculum_command_schedule"]
    curriculum_schedule = curriculum_locomotion_command_schedule(
        seeds["randomization.command"], curriculum_golden["episode_ordinal"]
    )
    if command_schedule_hash(curriculum_schedule) != curriculum_golden["sha256"]:
        raise ValueError("curriculum command schedule hash mismatch")
    for sample in curriculum_golden["samples"]:
        if list(curriculum_schedule[sample["tick"]]) != sample["command_raw"]:
            raise ValueError(
                f"curriculum command schedule sample mismatch at tick {sample['tick']}"
            )

    transform = golden["root_local_transform"]
    actual_local = rotate_world_to_root_local_q1_30(
        transform["quaternion_xyzw_q1_30"], transform["world_vector_raw"]
    )
    if list(actual_local) != transform["root_local_vector_raw"]:
        raise ValueError("root-local transform golden mismatch")

    pd_input = golden["pd_input"]
    first, first_flags = _golden_pd(pd_input, 0)
    second, second_flags = _golden_pd(pd_input, first)
    expected = (
        golden["pd_first_effort"],
        golden["pd_first_flags"],
        golden["pd_second_effort"],
        golden["pd_second_flags"],
    )
    if (first, first_flags, second, second_flags) != expected:
        raise ValueError("fixed-point PD golden mismatch")
    if (
        descriptor_bytes is not None
        and sha256_bytes(descriptor_bytes) != golden["descriptor_sha256"]
    ):
        raise ValueError("Rust descriptor hash mismatch")


def validate_biomechanics_descriptor(descriptor: dict[str, Any]) -> None:
    """Validate the generated TRAIN-2 biomechanics mirror without anatomy defaults."""
    translator_id = descriptor.get("translator_id")
    current_material_lineage = translator_id == BIOMECHANICS_TRANSLATOR_ID_V2
    expected_schema_version = 2 if current_material_lineage else 1
    if descriptor.get("schema_version") != expected_schema_version:
        raise ValueError("unsupported biomechanics mirror descriptor")
    if translator_id not in {
        BIOMECHANICS_TRANSLATOR_ID,
        BIOMECHANICS_TRANSLATOR_ID_V2,
    }:
        raise ValueError("biomechanics translator identity mismatch")
    if (
        descriptor.get("physics_hz") != PHYSICS_HZ
        or descriptor.get("motor_hz") != MOTOR_HZ
    ):
        raise ValueError("unsupported biomechanics cadence")
    if descriptor.get("body_count") != 24 or descriptor.get("action_width") != 23:
        raise ValueError("biomechanics tensor/topology width mismatch")
    _require_hash(descriptor.get("body_schema_hash"), "body_schema_hash")
    _require_hash(
        descriptor.get("compiled_descriptor_hash"), "compiled_descriptor_hash"
    )
    if not isinstance(descriptor.get("body_schema_id"), str) or not isinstance(
        descriptor.get("body_schema_revision"), int
    ):
        raise ValueError("biomechanics schema identity is incomplete")
    if descriptor.get("coordinate_mapping") != {
        "engine_axes": "+X right, +Y up, +Z forward",
        "isaac_from_engine_vector": ["x", "-z", "y"],
        "isaac_quaternion_order": "wxyz",
        "engine_quaternion_order": "xyzw",
    }:
        raise ValueError("biomechanics coordinate mapping mismatch")

    ordered_bodies = require_unique_strings(
        descriptor.get("ordered_body_ids"), "biomechanics bodies"
    )
    ordered_actuators = require_unique_strings(
        descriptor.get("ordered_actuator_ids"), "biomechanics actuators"
    )
    bodies = descriptor.get("bodies")
    joints = descriptor.get("joints")
    actuators = descriptor.get("actuators")
    effectors = descriptor.get("effectors")
    exclusions = descriptor.get("collision_exclusions")
    if not all(
        isinstance(records, list)
        for records in (bodies, joints, actuators, effectors, exclusions)
    ):
        raise ValueError("biomechanics descriptor records must be arrays")
    if len(ordered_bodies) != 24 or len(bodies) != 24:
        raise ValueError("biomechanics body closure mismatch")
    if len(ordered_actuators) != 23 or len(actuators) != 23 or len(joints) != 23:
        raise ValueError("biomechanics joint/action closure mismatch")

    body_tokens: set[int] = set()
    shape_tokens: set[int] = set()
    collider_count = 0
    total_mass = 0
    for slot, body in enumerate(bodies):
        if body.get("body_slot") != slot or body.get("body_id") != ordered_bodies[slot]:
            raise ValueError("biomechanics body construction order mismatch")
        token = body.get("body_token")
        if not isinstance(token, int) or token in body_tokens:
            raise ValueError("biomechanics body token mismatch")
        body_tokens.add(token)
        parent = body.get("parent_body_slot")
        if (slot == 0 and parent is not None) or (
            slot != 0 and (not isinstance(parent, int) or not 0 <= parent < slot)
        ):
            raise ValueError("biomechanics body hierarchy mismatch")
        mass = body.get("mass_microkilograms")
        if not isinstance(mass, int) or mass <= 0:
            raise ValueError("biomechanics body mass mismatch")
        total_mass += mass
        _require_int_vector(body.get("center_of_mass_micrometres"), 3, "center of mass")
        _require_int_vector(
            body.get("authoritative_inertia_tensor_microkilogram_metre_squared"),
            6,
            "authoritative inertia",
        )
        _require_int_vector(
            body.get("solver_principal_inertia_microkilogram_metre_squared"),
            3,
            "principal inertia",
        )
        colliders = body.get("colliders")
        if not isinstance(colliders, list):
            raise ValueError("biomechanics colliders must be an array")
        if bool(body.get("non_colliding_carrier")) != (len(colliders) == 0):
            raise ValueError("biomechanics carrier/collider mismatch")
        collider_count += len(colliders)
        for collider in colliders:
            shape_token = collider.get("shape_token")
            if not isinstance(shape_token, int) or shape_token in shape_tokens:
                raise ValueError("biomechanics shape token mismatch")
            shape_tokens.add(shape_token)
            if not isinstance(collider.get("collider_id"), str):
                raise ValueError("biomechanics collider identity mismatch")
            _require_int_vector(
                collider.get("local_translation_micrometres"), 3, "collider translation"
            )
            _require_int_vector(
                collider.get("local_rotation_q1_30"), 4, "collider rotation"
            )
            _validate_biomechanics_geometry(collider.get("geometry"))
            layer = collider.get("collision_layer")
            mask = collider.get("collision_mask")
            if not isinstance(layer, int) or not 0 <= layer < 64:
                raise ValueError("biomechanics collision layer mismatch")
            if not isinstance(mask, int) or not 0 < mask < 1 << 64:
                raise ValueError("biomechanics collision mask mismatch")
    if total_mass != 75_337_000 or collider_count != 19:
        raise ValueError("biomechanics mass/collider total mismatch")

    joint_ids: set[str] = set()
    dof_ordinals: set[int] = set()
    for joint in joints:
        joint_id = joint.get("joint_id")
        dof = joint.get("dof_ordinal")
        if not isinstance(joint_id, str) or joint_id in joint_ids:
            raise ValueError("biomechanics joint identity mismatch")
        if not isinstance(dof, int) or dof in dof_ordinals or not 0 <= dof < 23:
            raise ValueError("biomechanics DoF ordinal mismatch")
        joint_ids.add(joint_id)
        dof_ordinals.add(dof)
        parent = joint.get("parent_body_slot")
        child = joint.get("child_body_slot")
        if not all(
            isinstance(value, int) and 0 <= value < 24 for value in (parent, child)
        ):
            raise ValueError("biomechanics joint body mapping mismatch")
        axis = _require_int_vector(joint.get("axis_q1_30"), 3, "joint axis")
        if sum(value * value for value in axis) != 1 << 60:
            raise ValueError("biomechanics joint axis is not normalized")
        hard = _require_int_vector(
            joint.get("hard_limit_microradians"), 2, "hard limit"
        )
        soft = _require_int_vector(
            joint.get("soft_limit_microradians"), 2, "soft limit"
        )
        neutral = joint.get("neutral_position_microradians")
        if not (
            hard[0] < hard[1] and hard[0] <= soft[0] <= neutral <= soft[1] <= hard[1]
        ):
            raise ValueError("biomechanics joint limit envelope mismatch")
        for name in ("parent_frame", "child_frame"):
            frame = joint.get(name)
            if not isinstance(frame, dict):
                raise ValueError("biomechanics joint frame mismatch")
            _require_int_vector(frame.get("translation_micrometres"), 3, name)
            _require_int_vector(frame.get("rotation_q1_30"), 4, name)
        _require_int_vector(
            joint.get("solver_parent_rotation_f32_bits"), 4, "solver frame"
        )
        _require_int_vector(
            joint.get("solver_child_rotation_f32_bits"), 4, "solver frame"
        )
    if dof_ordinals != set(range(23)):
        raise ValueError("biomechanics DoF mapping is partial")

    if ordered_actuators != [record.get("actuator_id") for record in actuators]:
        raise ValueError("biomechanics actuator tensor order mismatch")
    if {record.get("joint_id") for record in actuators} != joint_ids:
        raise ValueError("biomechanics actuator/joint mapping mismatch")
    if {record.get("dof_ordinal") for record in actuators} != set(range(23)):
        raise ValueError("biomechanics actuator/DoF mapping mismatch")
    for actuator in actuators:
        effort = _require_int_vector(
            actuator.get("effort_micronewton_metres"), 2, "actuator effort"
        )
        target_delta = _require_int_vector(
            actuator.get("target_delta_microradians_per_motor_tick"),
            2,
            "actuator target delta",
        )
        if not effort[0] < 0 < effort[1] or not target_delta[0] < 0 < target_delta[1]:
            raise ValueError("biomechanics actuator envelope mismatch")

    canonical_exclusions = [tuple(pair) for pair in exclusions]
    if canonical_exclusions != sorted(set(canonical_exclusions)) or any(
        len(pair) != 2 or not 0 <= pair[0] < pair[1] < 24
        for pair in canonical_exclusions
    ):
        raise ValueError("biomechanics collision exclusions are not canonical")
    body_ids = set(ordered_bodies)
    effector_ids = [record.get("effector_id") for record in effectors]
    if len(effector_ids) != len(set(effector_ids)) or any(
        not isinstance(value, str) for value in effector_ids
    ):
        raise ValueError("biomechanics effector identity mismatch")
    if any(record.get("body_id") not in body_ids for record in effectors):
        raise ValueError("biomechanics effector body mapping mismatch")
    if current_material_lineage:
        validate_current_material_lineage(descriptor)


def validate_current_biomechanics_descriptor(descriptor: dict[str, Any]) -> None:
    validate_biomechanics_descriptor(descriptor)
    if descriptor.get("translator_id") != BIOMECHANICS_TRANSLATOR_ID_V2:
        raise ValueError("current biomechanics mirror V2 is required")


def validate_biomechanics_standing_descriptor(descriptor: dict[str, Any]) -> None:
    extras = {"training_descriptor_id", "observation_width", "environment_profiles"}
    base = {key: value for key, value in descriptor.items() if key not in extras}
    validate_current_biomechanics_descriptor(base)
    if (
        set(descriptor) != set(base) | extras
        or descriptor.get("training_descriptor_id")
        != "nextengine.isaac.humanoid-biomechanics-standing.v2"
        or descriptor.get("observation_width") != 84
    ):
        raise ValueError("biomechanics standing descriptor identity mismatch")
    profiles = descriptor.get("environment_profiles")
    if not isinstance(profiles, list) or len(profiles) != 1:
        raise ValueError("biomechanics standing descriptor profile mismatch")
    validate_biomechanics_standing_profile(profiles[0])


def _require_int_vector(value: Any, width: int, label: str) -> list[int]:
    if (
        not isinstance(value, list)
        or len(value) != width
        or not all(isinstance(item, int) for item in value)
    ):
        raise ValueError(f"biomechanics {label} mismatch")
    return value


def _validate_biomechanics_geometry(geometry: Any) -> None:
    if not isinstance(geometry, dict):
        raise ValueError("biomechanics geometry mismatch")
    kind = geometry.get("kind")
    fields = {
        "box": ("half_extents_micrometres", 3),
        "sphere": ("radius_micrometres", None),
        "capsule": ("radius_micrometres", None),
    }
    if kind not in fields:
        raise ValueError("biomechanics geometry kind mismatch")
    field, width = fields[kind]
    value = geometry.get(field)
    if width is None:
        if not isinstance(value, int) or value <= 0:
            raise ValueError("biomechanics geometry dimension mismatch")
    else:
        dimensions = _require_int_vector(value, width, "geometry dimension")
        if any(dimension <= 0 for dimension in dimensions):
            raise ValueError("biomechanics geometry dimension mismatch")
    if kind == "capsule" and (
        not isinstance(geometry.get("half_segment_micrometres"), int)
        or geometry["half_segment_micrometres"] <= 0
    ):
        raise ValueError("biomechanics capsule dimension mismatch")


def validate_descriptor(descriptor: dict[str, Any]) -> None:
    if descriptor.get("schema_version") != 2:
        raise ValueError("unsupported mirror descriptor")
    if (
        descriptor.get("physics_hz") != PHYSICS_HZ
        or descriptor.get("motor_hz") != MOTOR_HZ
    ):
        raise ValueError("unsupported Stage 0 cadence")
    if (
        descriptor.get("observation_width") != 84
        or descriptor.get("action_width") != 23
    ):
        raise ValueError("canonical tensor width mismatch")
    ordered_bodies = require_unique_strings(
        descriptor.get("ordered_body_ids"), "bodies"
    )
    ordered_actuators = require_unique_strings(
        descriptor.get("ordered_actuator_ids"), "actuators"
    )
    body_records = descriptor.get("bodies")
    actuator_records = descriptor.get("actuators")
    if not isinstance(body_records, list) or not isinstance(actuator_records, list):
        raise ValueError("descriptor records must be arrays")
    if set(ordered_bodies) != {record.get("body_id") for record in body_records}:
        raise ValueError("body construction order does not close")
    if ordered_actuators != [record.get("actuator_id") for record in actuator_records]:
        raise ValueError("actuator tensor order mismatch")
    if len(ordered_actuators) != 23:
        raise ValueError("Stage 0 descriptor must contain 23 actuators")
    profiles = descriptor.get("environment_profiles")
    if not isinstance(profiles, list) or {
        profile.get("profile_id") for profile in profiles
    } != {
        STANDING_PROFILE_ID,
        BOUNDED_STANDING_PROFILE_ID,
        FLAT_LOCOMOTION_PROFILE_ID,
        CURRICULUM_LOCOMOTION_PROFILE_ID,
    }:
        raise ValueError("canonical environment profiles do not close")
    body_schema_hash = descriptor.get("body_schema_hash")
    _require_hash(body_schema_hash, "body_schema_hash")
    for profile in profiles:
        for field in (
            "manifest_hash",
            "observation_layout_hash",
            "action_layout_hash",
            "command_schedule_profile_hash",
            "reward_profile_hash",
            "translator_version_hash",
            "termination_profile_hash",
            "rng_derivation_profile_hash",
            "correspondence_profile_hash",
        ):
            _require_hash(profile.get(field), field)
        if len(profile.get("observation_source_ids", [])) != 84:
            raise ValueError("observation source order mismatch")
        expected = (
            STANDING_REWARD_COMPONENT_IDS
            if profile["profile_id"] == STANDING_PROFILE_ID
            else BOUNDED_STANDING_REWARD_COMPONENT_IDS
            if profile["profile_id"] == BOUNDED_STANDING_PROFILE_ID
            else CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS
            if profile["profile_id"] == CURRICULUM_LOCOMOTION_PROFILE_ID
            else LOCOMOTION_REWARD_COMPONENT_IDS
        )
        actual = tuple(
            component.get("component_id")
            for component in profile.get("reward_components", [])
        )
        if actual != expected:
            raise ValueError(
                f"reward component order mismatch for {profile['profile_id']}"
            )
        translator_identity = (
            CURRENT_TRANSLATOR_VERSION
            if profile["profile_id"]
            in {BOUNDED_STANDING_PROFILE_ID, CURRICULUM_LOCOMOTION_PROFILE_ID}
            else LEGACY_TRANSLATOR_PROFILE_ID
        )
        translator_hash = hashlib.sha256(
            translator_identity.encode("utf-8")
            + b"\0"
            + bytes.fromhex(body_schema_hash)
        ).hexdigest()
        if profile["translator_version_hash"] != translator_hash:
            raise ValueError("environment translator identity does not close")
        if profile["profile_id"] in {
            STANDING_PROFILE_ID,
            BOUNDED_STANDING_PROFILE_ID,
        }:
            if (
                profile.get("velocity_frame") != "world"
                or profile.get("maximum_episode_steps") != 3_600
            ):
                raise ValueError("standing profile semantics changed")
        elif profile["profile_id"] == FLAT_LOCOMOTION_PROFILE_ID:
            if (
                profile.get("velocity_frame") != "root-local"
                or profile.get("maximum_episode_steps") != 1_200
            ):
                raise ValueError("locomotion profile semantics mismatch")
            command = profile.get("command_profile", {})
            if (
                command.get("warmup_ticks") != 60
                or command.get("segment_ticks") != 120
                or command.get("episode_ticks") != 1_200
                or command.get("mode_weights_basis_points")
                != [2_500, 3_500, 2_000, 2_000]
            ):
                raise ValueError("locomotion command profile mismatch")
        else:
            if (
                profile.get("velocity_frame") != "root-local"
                or profile.get("maximum_episode_steps") != 1_200
            ):
                raise ValueError("curriculum locomotion profile semantics mismatch")
            command = profile.get("command_profile", {})
            stages = command.get("stages")
            if (
                command.get("kind") != "sha256-counter-episode-curriculum-v2"
                or command.get("episode_ticks") != 1_200
                or not isinstance(stages, list)
                or len(stages) != len(CURRICULUM_COMMAND_STAGES_V2)
            ):
                raise ValueError("curriculum command profile mismatch")
            for actual, expected_stage in zip(
                stages, CURRICULUM_COMMAND_STAGES_V2, strict=True
            ):
                for field in (
                    "first_episode_ordinal",
                    "warmup_ticks",
                    "segment_ticks",
                    "episode_ticks",
                    "mode_weights_basis_points",
                    "right_velocity_range_raw",
                    "forward_velocity_range_raw",
                    "yaw_rate_range_raw",
                    "linear_rate_limit_raw_per_second_squared",
                    "yaw_rate_limit_raw_per_second_squared",
                ):
                    expected_value = expected_stage[field]
                    if isinstance(expected_value, tuple):
                        expected_value = list(expected_value)
                    if actual.get(field) != expected_value:
                        raise ValueError(f"curriculum stage mismatch: {field}")


def require_unique_strings(value: Any, label: str) -> list[str]:
    if (
        not isinstance(value, list)
        or not value
        or not all(isinstance(item, str) for item in value)
    ):
        raise ValueError(f"{label} must be a non-empty string array")
    if len(set(value)) != len(value):
        raise ValueError(f"{label} contains duplicates")
    return value


def unsigned_sum(values: Iterable[int]) -> int:
    return min(sum(abs(value) for value in values), (1 << 63) - 1)


def _golden_pd(pd_input: dict[str, int], previous: int) -> tuple[int, int]:
    return fixed_pd_substep(
        neutral_position_microradians=0,
        residual_target_microradians=pd_input["residual_target_microradians"],
        position_microradians=pd_input["position_microradians"],
        velocity_microradians_per_second=pd_input["velocity_microradians_per_second"],
        stiffness_q16=80 * 65_536,
        damping_q16=4 * 65_536,
        limit_min_microradians=-1_500_000,
        limit_max_microradians=1_500_000,
        maximum_effort_micronewton_metres=150_000_000,
        maximum_effort_rate_micronewton_metres_per_second=6_000_000_000,
        previous_effort_micronewton_metres=previous,
    )


def _require_hash(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise ValueError(f"{label} must be a lowercase SHA-256 hash")
    return value
