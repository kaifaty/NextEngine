from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Iterable

SEED_DOMAIN = b"nextengine.motor-episode-seed.v1\0"
PHYSICS_HZ = 240
MOTOR_HZ = 60
SUBSTEPS = 4
TARGET_CLAMPED = 1 << 0
EFFORT_CLAMPED = 1 << 1
RATE_CLAMPED = 1 << 2


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


def validate_golden(golden: dict[str, Any], descriptor_bytes: bytes | None = None) -> None:
    if golden.get("schema_version") != 1:
        raise ValueError("unsupported motor mirror golden version")
    actuator_ids = require_unique_strings(golden.get("ordered_actuator_ids"), "actuators")
    reward_ids = require_unique_strings(golden.get("reward_component_ids"), "rewards")
    if len(actuator_ids) != golden.get("pd_channel_count") or len(actuator_ids) != 23:
        raise ValueError("golden actuator count mismatch")
    if len(reward_ids) != 8:
        raise ValueError("golden reward component count mismatch")
    seed_input = golden["seed_input"]
    run_root = bytes.fromhex(seed_input["run_root"])
    for record in golden["purpose_seeds"]:
        actual = derive_purpose_seed(
            run_root,
            seed_input["episode_ordinal"],
            seed_input["vector_slot"],
            record["purpose_id"],
        ).hex()
        if actual != record["seed"]:
            raise ValueError(f"seed mismatch for {record['purpose_id']}")
    pd_input = golden["pd_input"]
    first, first_flags = fixed_pd_substep(
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
        previous_effort_micronewton_metres=0,
    )
    second, second_flags = fixed_pd_substep(
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
        previous_effort_micronewton_metres=first,
    )
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
    if descriptor.get("schema_version") != 1:
        raise ValueError("unsupported mirror descriptor")
    if descriptor.get("physics_hz") != PHYSICS_HZ or descriptor.get("motor_hz") != MOTOR_HZ:
        raise ValueError("unsupported Stage 0 cadence")
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


def require_unique_strings(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or not value or not all(isinstance(item, str) for item in value):
        raise ValueError(f"{label} must be a non-empty string array")
    if len(set(value)) != len(value):
        raise ValueError(f"{label} contains duplicates")
    return value


def unsigned_sum(values: Iterable[int]) -> int:
    return min(sum(abs(value) for value in values), (1 << 63) - 1)
