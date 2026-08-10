from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Iterable

SEED_DOMAIN = b"nextengine.motor-episode-seed.v1\0"
COMMAND_COUNTER_DOMAIN = b"nextengine.motor-command-counter.v1\0"
COMMAND_SCHEDULE_DOMAIN = b"nextengine.motor-command-schedule.v1\0"
STANDING_PROFILE_ID = "nextengine.motor.env.humanoid-standing.v1"
FLAT_LOCOMOTION_PROFILE_ID = "nextengine.motor.env.humanoid-flat-command.v1"
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
    schedule = [(0, 0, 0)]
    target = (0, 0, 0)
    for tick in range(1, 1_201):
        if tick < 60:
            target = (0, 0, 0)
        elif tick == 60 or (tick - 60) % 120 == 0:
            segment = (tick - 60) // 120
            selector = _command_counter(command_seed, segment, 0) % 10_000
            right = _bounded_counter(command_seed, segment, 1, -2_000_000, 2_000_000)
            forward = _bounded_counter(command_seed, segment, 2, -1_500_000, 3_000_000)
            yaw = _bounded_counter(command_seed, segment, 3, -1_500_000, 1_500_000)
            if selector < 2_500:
                target = (0, 0, 0)
            elif selector < 6_000:
                target = (right, forward, 0)
            elif selector < 8_000:
                target = (0, 0, yaw)
            else:
                target = (right, forward, yaw)
        previous = schedule[-1]
        schedule.append(
            (
                _move_towards(previous[0], target[0], 50_000),
                _move_towards(previous[1], target[1], 50_000),
                _move_towards(previous[2], target[2], 25_000),
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
    diagonal = lambda left, right: Q1_30_ONE - coefficient(
        2 * (left * left + right * right)
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
        round_div_ties_even(sum(matrix[row][column] * vector[row] for row in range(3)), Q1_30_ONE)
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


def select_environment_profile(descriptor: dict[str, Any], profile_id: str) -> dict[str, Any]:
    validate_descriptor(descriptor)
    profiles = descriptor["environment_profiles"]
    for profile in profiles:
        if profile["profile_id"] == profile_id:
            return profile
    raise ValueError(f"unsupported environment profile: {profile_id}")


def validate_golden(golden: dict[str, Any], descriptor_bytes: bytes | None = None) -> None:
    if golden.get("schema_version") != 2:
        raise ValueError("unsupported motor mirror golden version")
    actuator_ids = require_unique_strings(golden.get("ordered_actuator_ids"), "actuators")
    if len(actuator_ids) != golden.get("pd_channel_count") or len(actuator_ids) != 23:
        raise ValueError("golden actuator count mismatch")
    reward_profiles = golden.get("reward_component_ids")
    if not isinstance(reward_profiles, dict):
        raise ValueError("golden reward profiles are missing")
    expected_rewards = {
        STANDING_PROFILE_ID: STANDING_REWARD_COMPONENT_IDS,
        FLAT_LOCOMOTION_PROFILE_ID: LOCOMOTION_REWARD_COMPONENT_IDS,
    }
    for profile_id, expected in expected_rewards.items():
        actual = require_unique_strings(reward_profiles.get(profile_id), f"rewards:{profile_id}")
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
            raise ValueError(f"command schedule sample mismatch at tick {sample['tick']}")

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
    if descriptor_bytes is not None and sha256_bytes(descriptor_bytes) != golden["descriptor_sha256"]:
        raise ValueError("Rust descriptor hash mismatch")


def validate_descriptor(descriptor: dict[str, Any]) -> None:
    if descriptor.get("schema_version") != 2:
        raise ValueError("unsupported mirror descriptor")
    if descriptor.get("physics_hz") != PHYSICS_HZ or descriptor.get("motor_hz") != MOTOR_HZ:
        raise ValueError("unsupported Stage 0 cadence")
    if descriptor.get("observation_width") != 84 or descriptor.get("action_width") != 23:
        raise ValueError("canonical tensor width mismatch")
    ordered_bodies = require_unique_strings(descriptor.get("ordered_body_ids"), "bodies")
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
    } != {STANDING_PROFILE_ID, FLAT_LOCOMOTION_PROFILE_ID}:
        raise ValueError("canonical environment profiles do not close")
    for profile in profiles:
        for field in (
            "manifest_hash",
            "observation_layout_hash",
            "action_layout_hash",
            "command_schedule_profile_hash",
            "reward_profile_hash",
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
            else LOCOMOTION_REWARD_COMPONENT_IDS
        )
        actual = tuple(component.get("component_id") for component in profile.get("reward_components", []))
        if actual != expected:
            raise ValueError(f"reward component order mismatch for {profile['profile_id']}")
        if profile["profile_id"] == STANDING_PROFILE_ID:
            if profile.get("velocity_frame") != "world" or profile.get("maximum_episode_steps") != 3_600:
                raise ValueError("standing profile semantics changed")
        else:
            if profile.get("velocity_frame") != "root-local" or profile.get("maximum_episode_steps") != 1_200:
                raise ValueError("locomotion profile semantics mismatch")
            command = profile.get("command_profile", {})
            if (
                command.get("warmup_ticks") != 60
                or command.get("segment_ticks") != 120
                or command.get("episode_ticks") != 1_200
                or command.get("mode_weights_basis_points") != [2_500, 3_500, 2_000, 2_000]
            ):
                raise ValueError("locomotion command profile mismatch")


def require_unique_strings(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or not value or not all(isinstance(item, str) for item in value):
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
    if not isinstance(value, str) or len(value) != 64 or any(
        character not in "0123456789abcdef" for character in value
    ):
        raise ValueError(f"{label} must be a lowercase SHA-256 hash")
    return value
