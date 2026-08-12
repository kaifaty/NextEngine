from __future__ import annotations

import hashlib
from typing import Any

from next_lab.motor_mirror import round_div_ties_even

PROFILE_SHA256 = "ad20d7a4abd5cc8b59069ecdb59161499ce7754953cbff2477f2850395adb42c"
PROFILE_BYTES = bytes.fromhex(PROFILE_SHA256)
Q1_30_ONE = 1 << 30
TARGET_CLAMPED = 1 << 0
EFFORT_CLAMPED = 1 << 1
RATE_CLAMPED = 1 << 2
TARGET_SLEW_CLAMPED = 1 << 3
POWER_CLAMPED = 1 << 4
WORK_CLAMPED = 1 << 5
ACTIVE_IMPULSE = 50_000
BRUSH_CEILING = 250_000
GRACE_SUBSTEPS = 4


def validate_safety_contact_golden(
    golden: dict[str, Any], descriptor: dict[str, Any]
) -> None:
    if golden.get("schema_version") != 1:
        raise ValueError("unsupported safety/contact mirror version")
    if golden.get("mirror_id") != "nextengine.isaac.humanoid-safety-contact-mirror.v1":
        raise ValueError("safety/contact mirror identity mismatch")
    if golden.get("safety_contact_profile_sha256") != PROFILE_SHA256:
        raise ValueError("safety/contact profile hash mismatch")
    for name in ("body_schema_hash", "compiled_descriptor_hash"):
        if golden.get(name) != descriptor.get(name):
            raise ValueError(f"safety/contact {name} mismatch")
    if golden.get("physics_hz") != 240 or golden.get("motor_hz") != 60:
        raise ValueError("safety/contact cadence mismatch")
    if golden.get("action_width") != 23:
        raise ValueError("safety/contact action width mismatch")
    _validate_safety_scenario(golden["safety_scenario"], descriptor, golden)
    _validate_contact_terminal_scenarios(
        golden["contact_terminal_scenarios"], descriptor
    )


def _validate_safety_scenario(
    scenario: dict[str, Any], descriptor: dict[str, Any], golden: dict[str, Any]
) -> None:
    actuators = descriptor["actuators"]
    joints = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    references = _int_list(scenario["reference_targets_microradians"], 23)
    residuals = _int_list(scenario["normalized_residuals_q1_30"], 23)
    states = [_int_list(value, 2) for value in scenario["joint_states"]]
    envelopes = scenario["skill_envelopes"]
    if len(states) != 23 or len(envelopes) != 23:
        raise ValueError("safety scenario width mismatch")
    targets = [joints[actuator["joint_id"]]["neutral_position_microradians"] for actuator in actuators]
    previous_efforts = [0] * 23
    work = [0] * 23
    final_targets: list[dict[str, Any]] = []
    final_tick_efforts: list[list[dict[str, Any]]] = []
    for _ in range(int(scenario["motor_ticks"])):
        next_targets: list[int] = []
        final_targets = []
        for index, actuator in enumerate(actuators):
            joint = joints[actuator["joint_id"]]
            envelope = envelopes[index]
            if envelope["joint_id"] != actuator["joint_id"]:
                raise ValueError("skill envelope joint order mismatch")
            residual = residuals[index]
            if not -Q1_30_ONE <= residual <= Q1_30_ONE:
                raise ValueError("normalized residual outside Q1.30")
            candidate = references[index] + round_div_ties_even(
                residual * actuator["residual_scale_microradians"], Q1_30_ONE
            )
            envelope_minimum = max(
                joint["hard_limit_microradians"][0],
                joint["soft_limit_microradians"][0],
                envelope["minimum_microradians"],
            )
            envelope_maximum = min(
                joint["hard_limit_microradians"][1],
                joint["soft_limit_microradians"][1],
                envelope["maximum_microradians"],
            )
            slew_minimum = targets[index] + actuator[
                "target_delta_microradians_per_motor_tick"
            ][0]
            slew_maximum = targets[index] + actuator[
                "target_delta_microradians_per_motor_tick"
            ][1]
            minimum = max(envelope_minimum, slew_minimum)
            maximum = min(envelope_maximum, slew_maximum)
            if minimum > maximum:
                raise ValueError("empty target envelope")
            target = min(max(candidate, minimum), maximum)
            flags = 0
            if not envelope_minimum <= candidate <= envelope_maximum:
                flags |= TARGET_CLAMPED
            if not slew_minimum <= candidate <= slew_maximum:
                flags |= TARGET_SLEW_CLAMPED
            next_targets.append(target)
            final_targets.append(
                {
                    "actuator_id": actuator["actuator_id"],
                    "target_microradians": target,
                    "clamp_flags": flags,
                }
            )
        targets = next_targets
        work = [0] * 23
        final_tick_efforts = []
        for _ in range(4):
            next_efforts: list[int] = []
            substep: list[dict[str, Any]] = []
            for index, actuator in enumerate(actuators):
                position, velocity = states[index]
                joint = joints[actuator["joint_id"]]
                if not joint["hard_limit_microradians"][0] <= position <= joint[
                    "hard_limit_microradians"
                ][1]:
                    raise ValueError("golden joint outside hard ROM")
                if abs(velocity) > joint["maximum_velocity_microradians_per_second"]:
                    raise ValueError("golden joint outside velocity bound")
                requested = round_div_ties_even(
                    actuator["stiffness_q16"] * (targets[index] - position), 65_536
                ) - round_div_ties_even(actuator["damping_q16"] * velocity, 65_536)
                effort, flags = _intersect_effort(
                    actuator, requested, previous_efforts[index], velocity, work[index]
                )
                charge = _positive_work_charge(effort, velocity)
                work[index] += charge
                if work[index] > actuator[
                    "maximum_positive_work_microjoules_per_motor_tick"
                ]:
                    raise ValueError("positive work exceeded")
                next_efforts.append(effort)
                substep.append(
                    {
                        "actuator_id": actuator["actuator_id"],
                        "effort_micronewton_metres": effort,
                        "clamp_flags": flags,
                    }
                )
            previous_efforts = next_efforts
            final_tick_efforts.append(substep)
    checkpoint = {
        "applied_targets_microradians": targets,
        "previous_efforts_micronewton_metres": previous_efforts,
        "positive_work_microjoules": work,
        "completed_substeps": 4,
        "motor_tick_prepared": True,
    }
    if final_targets != scenario["final_applied_targets"]:
        raise ValueError("safety target golden mismatch")
    if final_tick_efforts != scenario["final_tick_efforts"]:
        raise ValueError("safety effort golden mismatch")
    if checkpoint != scenario["final_checkpoint"]:
        raise ValueError("safety checkpoint golden mismatch")
    root = _safety_checkpoint_root(
        descriptor, golden, targets, previous_efforts, work, 4, True
    )
    if root != scenario["safety_checkpoint_root"]:
        raise ValueError("safety checkpoint root mismatch")


def _intersect_effort(
    actuator: dict[str, Any], requested: int, previous: int, velocity: int, used_work: int
) -> tuple[int, int]:
    effort_minimum, effort_maximum = actuator["effort_micronewton_metres"]
    maximum_delta = round_div_ties_even(
        actuator["maximum_effort_rate_micronewton_metres_per_second"], 240
    )
    rate_minimum, rate_maximum = previous - maximum_delta, previous + maximum_delta
    velocity_abs = abs(velocity)
    power_maximum = (
        (1 << 127) - 1
        if velocity_abs == 0
        else actuator["maximum_power_microwatts"] * 1_000_000 // velocity_abs
    )
    remaining_work = (
        actuator["maximum_positive_work_microjoules_per_motor_tick"] - used_work
    )
    work_maximum = (
        (1 << 127) - 1
        if velocity_abs == 0
        else remaining_work * 240 * 1_000_000 // velocity_abs
    )
    minimum = max(effort_minimum, rate_minimum, -power_maximum)
    maximum = min(effort_maximum, rate_maximum, power_maximum)
    if velocity > 0:
        maximum = min(maximum, work_maximum)
    elif velocity < 0:
        minimum = max(minimum, -work_maximum)
    if minimum > maximum:
        raise ValueError("empty effort envelope")
    effort = min(max(requested, minimum), maximum)
    flags = 0
    if not effort_minimum <= requested <= effort_maximum:
        flags |= EFFORT_CLAMPED
    if not rate_minimum <= requested <= rate_maximum:
        flags |= RATE_CLAMPED
    if not -power_maximum <= requested <= power_maximum:
        flags |= POWER_CLAMPED
    if (velocity > 0 and requested > work_maximum) or (
        velocity < 0 and requested < -work_maximum
    ):
        flags |= WORK_CLAMPED
    return effort, flags


def _positive_work_charge(effort: int, velocity: int) -> int:
    product = effort * velocity
    return 0 if product <= 0 else (product + 240_000_000 - 1) // 240_000_000


def _safety_checkpoint_root(
    descriptor: dict[str, Any],
    golden: dict[str, Any],
    targets: list[int],
    efforts: list[int],
    work: list[int],
    completed_substeps: int,
    prepared: bool,
) -> str:
    payload = bytearray(b"nextengine.humanoid-safety-checkpoint.v1\0")
    payload.extend(PROFILE_BYTES)
    payload.extend(bytes.fromhex(golden["body_schema_hash"]))
    payload.extend(bytes.fromhex(golden["compiled_descriptor_hash"]))
    actuators = descriptor["actuators"]
    payload.extend(_u64(len(actuators)))
    for index, actuator in enumerate(actuators):
        payload.extend(_text(actuator["actuator_id"]))
        payload.extend(_text(actuator["joint_id"]))
        payload.extend(_i64(targets[index]))
        payload.extend(_i64(efforts[index]))
        payload.extend(_u64(work[index]))
    payload.append(completed_substeps)
    payload.append(int(prepared))
    return hashlib.sha256(payload).hexdigest()


def _validate_contact_terminal_scenarios(
    scenarios: list[dict[str, Any]], descriptor: dict[str, Any]
) -> None:
    if [scenario["name"] for scenario in scenarios] != [
        "locomotion-hand-grace",
        "locomotion-knee-material",
        "locomotion-torso-material",
        "getup-knee-support",
        "locomotion-head-impact",
        "locomotion-self-collision",
        "locomotion-timeout",
    ]:
        raise ValueError("contact/terminal scenario closure mismatch")
    shape_roles, shape_actors = _shape_maps(descriptor)
    for scenario in scenarios:
        continuity: dict[tuple[int, int, int, int], int] = {}
        profile = int(scenario["skill_profile"])
        maximum_ticks = int(scenario["maximum_episode_motor_ticks"])
        root = scenario["root"]
        for tick in scenario["ticks"]:
            frames: list[dict[str, Any]] = []
            for contacts in tick["contact_substeps"]:
                frame, continuity = _classify_contact_frame(
                    contacts, profile, continuity, shape_roles, shape_actors
                )
                frames.append(frame)
            if frames != tick["expected_frames"]:
                raise ValueError(f"contact frame mismatch in {scenario['name']}")
            decision = _terminal_decision(
                profile, maximum_ticks, int(tick["motor_tick"]), root, frames
            )
            if decision != tick["expected_decision"]:
                raise ValueError(f"terminal decision mismatch in {scenario['name']}")


def _shape_maps(
    descriptor: dict[str, Any]
) -> tuple[dict[int, int], dict[int, int]]:
    roles: dict[int, int] = {}
    actors: dict[int, int] = {}
    for body in descriptor["bodies"]:
        for collider in body["colliders"]:
            shape = int(collider["shape_token"])
            roles[shape] = int(collider["contact_role"])
            actors[shape] = int(body["body_token"])
    return roles, actors


def _classify_contact_frame(
    raw_contacts: list[dict[str, Any]],
    profile: int,
    previous: dict[tuple[int, int, int, int], int],
    shape_roles: dict[int, int],
    shape_actors: dict[int, int],
) -> tuple[dict[str, Any], dict[tuple[int, int, int, int], int]]:
    aggregates: dict[tuple[int, int, int, int], dict[str, Any]] = {}
    for contact in raw_contacts:
        first = (int(contact["actor_a_token"]), int(contact["shape_a_token"]))
        second = (int(contact["actor_b_token"]), int(contact["shape_b_token"]))
        sign = 1
        if first > second:
            first, second = second, first
            sign = -1
        pair = first + second
        aggregate = aggregates.setdefault(
            pair,
            {
                "impulse": [0, 0, 0],
                "separation": int(contact["separation_micrometres"]),
            },
        )
        for axis, value in enumerate(contact["impulse_micronewton_seconds"]):
            aggregate["impulse"][axis] += int(value) * sign
        aggregate["separation"] = min(
            aggregate["separation"], int(contact["separation_micrometres"])
        )
    continuity: dict[tuple[int, int, int, int], int] = {}
    records: list[dict[str, Any]] = []
    for pair in sorted(aggregates):
        aggregate = aggregates[pair]
        primary, secondary, self_contact = _roles_for_pair(
            pair, shape_roles, shape_actors
        )
        impulse = aggregate["impulse"]
        magnitude_squared = sum(value * value for value in impulse)
        active = aggregate["separation"] < 0 or magnitude_squared >= ACTIVE_IMPULSE**2
        if not active:
            continue
        consecutive = previous.get(pair, 0) + 1
        continuity[pair] = consecutive
        material = magnitude_squared > BRUSH_CEILING**2 or consecutive > GRACE_SUBSTEPS
        hard_limit = min(_hard_limit(primary), _hard_limit(secondary) if secondary else 1 << 63)
        hard = magnitude_squared > hard_limit * hard_limit
        contact_class = _contact_class(primary, self_contact, material, profile)
        records.append(
            {
                "pair": list(pair),
                "primary_role": primary,
                "secondary_role": secondary,
                "impulse_micronewton_seconds": impulse,
                "impulse_magnitude_squared": magnitude_squared,
                "minimum_separation_micrometres": aggregate["separation"],
                "consecutive_active_substeps": consecutive,
                "material": material,
                "hard_impact_violation": hard,
                "class": contact_class,
            }
        )
    return (
        {
            "classification_root": _classification_root(profile, records),
            "continuity_root": _continuity_root(continuity),
            "contacts": records,
        },
        continuity,
    )


def _roles_for_pair(
    pair: tuple[int, int, int, int],
    shape_roles: dict[int, int],
    shape_actors: dict[int, int],
) -> tuple[int, int | None, bool]:
    actor_a, shape_a, actor_b, shape_b = pair
    a_ground, b_ground = actor_a == 1, actor_b == 1
    if a_ground != b_ground:
        ground_shape, actor, shape = (
            (shape_a, actor_b, shape_b) if a_ground else (shape_b, actor_a, shape_a)
        )
        if ground_shape != 0 or shape_actors.get(shape) != actor:
            raise ValueError("invalid ground contact pair")
        return shape_roles[shape], None, False
    if a_ground or actor_a == actor_b:
        raise ValueError("invalid self contact pair")
    if shape_actors.get(shape_a) != actor_a or shape_actors.get(shape_b) != actor_b:
        raise ValueError("contact actor/shape mismatch")
    return shape_roles[shape_a], shape_roles[shape_b], True


def _hard_limit(role: int) -> int:
    if role == 8:
        return 6_000_000
    if role in {1, 2, 4, 5, 6}:
        return 4_000_000
    if role in {7, 9, 10, 11}:
        return 3_000_000
    if role == 3:
        return 1_000_000
    raise ValueError("unknown contact role")


def _contact_class(role: int, self_contact: bool, material: bool, profile: int) -> int:
    if self_contact:
        return 6 if material else 2
    if role == 8:
        return 1
    if profile == 1:
        return 3 if material else 2
    if profile == 2 and role in {10, 11}:
        return 4
    if profile == 3 and role in {6, 10, 11}:
        return 5
    if profile in {2, 3}:
        return 2
    raise ValueError("unknown skill contact profile")


def _classification_root(profile: int, records: list[dict[str, Any]]) -> str:
    payload = bytearray(b"nextengine.humanoid-contact-frame.v1\0")
    payload.extend(PROFILE_BYTES)
    payload.append(profile)
    payload.extend(_u64(len(records)))
    for record in records:
        for value in record["pair"]:
            payload.extend(_u64(value))
        payload.append(record["primary_role"])
        payload.append(record["secondary_role"] or 0)
        for value in record["impulse_micronewton_seconds"]:
            payload.extend(_i64(value))
        payload.extend(int(record["impulse_magnitude_squared"]).to_bytes(16, "little"))
        payload.extend(_i64(record["minimum_separation_micrometres"]))
        payload.extend(_u64(record["consecutive_active_substeps"]))
        payload.append(int(record["material"]))
        payload.append(int(record["hard_impact_violation"]))
        payload.append(record["class"])
    return hashlib.sha256(payload).hexdigest()


def _continuity_root(continuity: dict[tuple[int, int, int, int], int]) -> str:
    payload = bytearray(b"nextengine.humanoid-contact-continuity.v1\0")
    payload.extend(PROFILE_BYTES)
    payload.extend(_u64(len(continuity)))
    for pair in sorted(continuity):
        for value in pair:
            payload.extend(_u64(value))
        payload.extend(_u64(continuity[pair]))
    return hashlib.sha256(payload).hexdigest()


def _terminal_decision(
    profile: int,
    maximum_ticks: int,
    motor_tick: int,
    root: dict[str, Any],
    frames: list[dict[str, Any]],
) -> dict[str, Any]:
    hard = any(contact["hard_impact_violation"] for frame in frames for contact in frame["contacts"])
    self_collision = any(contact["class"] == 6 for frame in frames for contact in frame["contacts"])
    forbidden = profile == 1 and any(
        contact["class"] == 3 for frame in frames for contact in frame["contacts"]
    )
    x, height, z = root["position_micrometres"]
    qx, _, qz, _ = root["rotation_q1_30"]
    tilt = (1 << 60) - 2 * (qx * qx + qz * qz) <= (1 << 59)
    reason = (
        3
        if hard
        else 4
        if self_collision
        else 5
        if forbidden
        else 6
        if abs(x) >= 90_000_000 or abs(z) >= 90_000_000
        else 7
        if height <= 450_000 or tilt
        else 8
        if motor_tick >= maximum_ticks
        else None
    )
    disposition = 0 if reason is None else 2 if reason == 8 else 1
    reason_ids = {
        3: "terminal.contact-impact",
        4: "terminal.self-collision",
        5: "terminal.forbidden-locomotion-contact",
        6: "terminal.world-bounds",
        7: "terminal.fall",
        8: "terminal.timeout",
    }
    payload = bytearray(b"nextengine.humanoid-terminal-decision.v1\0")
    payload.extend(PROFILE_BYTES)
    payload.append(profile)
    payload.extend(_u64(maximum_ticks))
    payload.extend(_u64(motor_tick))
    payload.append(0)
    payload.append(0)
    payload.append(1)
    payload.extend(_u64(root["actor_token"]))
    for value in root["position_micrometres"]:
        payload.extend(_i64(value))
    for value in root["rotation_q1_30"]:
        payload.extend(_i64(value))
    payload.extend(_u64(len(frames)))
    for frame in frames:
        payload.extend(bytes.fromhex(frame["classification_root"]))
        payload.extend(bytes.fromhex(frame["continuity_root"]))
    payload.append(disposition)
    payload.append(reason or 0)
    return {
        "disposition": disposition,
        "reason": reason,
        "reason_id": reason_ids.get(reason),
        "decision_root": hashlib.sha256(payload).hexdigest(),
    }


def _int_list(value: Any, width: int | None = None) -> list[int]:
    if not isinstance(value, list) or (width is not None and len(value) != width):
        raise ValueError("expected integer array")
    if not all(isinstance(item, int) for item in value):
        raise ValueError("expected integer array")
    return value


def _i64(value: int) -> bytes:
    return int(value).to_bytes(8, "little", signed=True)


def _u64(value: int) -> bytes:
    return int(value).to_bytes(8, "little", signed=False)


def _text(value: str) -> bytes:
    encoded = value.encode("utf-8")
    return len(encoded).to_bytes(4, "little") + encoded
