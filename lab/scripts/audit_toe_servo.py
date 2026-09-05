"""FOOT-SERVO-01 diagnostics and independent integer effort replay."""

import argparse
import hashlib
import json
from pathlib import Path


def nearest_even(value, denominator):
    quotient, remainder = divmod(abs(value), denominator)
    increment = 2 * remainder > denominator or (
        2 * remainder == denominator and quotient % 2 == 1
    )
    return (quotient + increment) * (1 if value >= 0 else -1)


def effort(actuator, channel, previous, work):
    """Exact rational reconstruction, not a call to production safety code."""
    velocity = channel["input_velocity_urad_s"]
    request = nearest_even(
        actuator["stiffness_q16"]
        * (channel["target_urad"] - channel["input_position_urad"]),
        65536,
    ) - nearest_even(actuator["damping_q16"] * velocity, 65536)
    limits = actuator["effort_micronewton_metres"]
    delta = nearest_even(
        actuator["maximum_effort_rate_micronewton_metres_per_second"], 240
    )
    low, high = max(limits[0], previous - delta), min(limits[1], previous + delta)
    flags = 2 * (not limits[0] <= request <= limits[1])
    flags |= 4 * (not previous - delta <= request <= previous + delta)
    if velocity:
        power = actuator["maximum_power_microwatts"] * 1_000_000 // abs(velocity)
        remaining = actuator["maximum_positive_work_microjoules_per_motor_tick"] - work
        if remaining < 0:
            raise ValueError("work budget exceeded")
        work_limit = remaining * 240_000_000 // abs(velocity)
        low, high = max(low, -power), min(high, power)
        flags |= 16 * (abs(request) > power)
        if velocity > 0:
            high = min(high, work_limit)
            flags |= 32 * (request > work_limit)
        else:
            low = max(low, -work_limit)
            flags |= 32 * (request < -work_limit)
    if low > high:
        raise ValueError("empty effort intersection")
    applied = min(max(request, low), high)
    charge = (max(0, applied * velocity) + 239_999_999) // 240_000_000
    return request, applied, flags, work + charge


def summarize(trial, descriptor):
    actuators = {a["actuator_id"]: a for a in descriptor["actuators"]}
    limits = {j["dof_ordinal"]: j for j in descriptor["joints"]}
    previous = dict.fromkeys(actuators, 0)
    work = dict.fromkeys(actuators, 0)
    before = {j["ordinal"]: j for j in trial["initial"]["joints"]}
    stats = {
        key: {
            "peak_velocity_urad_s": 0,
            "peak_position_delta_urad": 0,
            "rate_clamped_steps": 0,
            "opposed_request_steps": 0,
        }
        for key in actuators
    }
    first_contact = first_impulse = first_ground = violation = None
    for step, frame in enumerate(trial["frames"], 1):
        if (
            frame["physics_substep"] != step
            or frame["motor_tick"] != (step - 1) // 4 + 1
        ):
            raise ValueError("noncontiguous trace")
        if (step - 1) % 4 == 0:
            work = dict.fromkeys(actuators, 0)
        channels = frame["channels"]
        if [c["actuator"] for c in channels] != descriptor["ordered_actuator_ids"]:
            raise ValueError("channel identity/order")
        after = {j["ordinal"]: j for j in frame["after"]["joints"]}
        if set(after) != set(limits) or len(after) != len(frame["after"]["joints"]):
            raise ValueError("DOF coverage")
        for channel in channels:
            key = channel["actuator"]
            actuator = actuators[key]
            dof = actuator["dof_ordinal"]
            if channel["dof"] != dof or (
                channel["input_position_urad"],
                channel["input_velocity_urad_s"],
            ) != (before[dof]["position_urad"], before[dof]["velocity_urad_s"]):
                raise ValueError("input/previous-state mismatch")
            request, applied, flags, work[key] = effort(
                actuator, channel, previous[key], work[key]
            )
            if (applied, flags) != (channel["effort_unm"], channel["effort_flags"]):
                raise ValueError(f"effort mismatch at {step}, {key}")
            previous[key] = applied
            stats[key]["rate_clamped_steps"] += bool(flags & 4)
            stats[key]["opposed_request_steps"] += request * applied < 0
            stats[key]["peak_velocity_urad_s"] = max(
                stats[key]["peak_velocity_urad_s"], abs(after[dof]["velocity_urad_s"])
            )
            stats[key]["peak_position_delta_urad"] = max(
                stats[key]["peak_position_delta_urad"],
                abs(after[dof]["position_urad"] - before[dof]["position_urad"]),
            )
        for contact in frame["after"]["contacts"]:
            first_contact = first_contact or step
            if any(contact["impulse_uns"]):
                first_impulse = first_impulse or step
            if 1 in contact["actors"]:
                first_ground = first_ground or step
        failures = [
            {"joint": limits[dof]["joint_id"], **j}
            for dof, j in after.items()
            if abs(j["velocity_urad_s"])
            > limits[dof]["maximum_velocity_microradians_per_second"] + 1000
            or j["position_urad"] < limits[dof]["hard_limit_microradians"][0] - 10
            or j["position_urad"] > limits[dof]["hard_limit_microradians"][1] + 10
        ]
        if failures:
            if step != len(trial["frames"]) or not trial["reason"].startswith(
                "joint-safety:"
            ):
                raise ValueError("failure precedence")
            violation = {"step": step, "joints": failures}
        before = after
    return {
        "height_um": trial["height_um"],
        "side": trial["side"],
        "steps": len(trial["frames"]),
        "reason": trial["reason"],
        "first_contact_record_step": first_contact,
        "first_nonzero_impulse_step": first_impulse,
        "first_ground_record_step": first_ground,
        "first_joint_violation": violation,
        "effort_replay_channels": len(trial["frames"]) * len(actuators),
        "channels": stats,
    }


def compare_baseline(control, baseline):
    for frame, old in zip(control["frames"], baseline["substep_samples"], strict=False):
        after = frame["after"]
        assert after["joints"] == old["joints"]
        assert after["contacts"] == old["contacts"]
        for key in (
            "position_um",
            "rotation_q1_30",
            "linear_velocity_um_s",
            "angular_velocity_urad_s",
        ):
            assert after["links"][0][key] == old[key]
        for channel in frame["channels"]:
            assert (
                channel["effort_unm"]
                == old["applied_efforts_by_dof_unm"][channel["dof"]]
            )
        if frame["physics_substep"] % 4 == 0:
            sample = baseline["samples"][frame["motor_tick"]]
            assert after["links"] == sample["links"]
            assert frame["reference_targets_urad"] == sample["reference_targets_urad"]
            assert [c["target_urad"] for c in frame["channels"]] == sample[
                "applied_targets_urad"
            ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("descriptor", "baseline", "trace", "output"):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    descriptor, baseline, trace = [
        json.loads(p.read_text()) for p in (args.descriptor, args.baseline, args.trace)
    ]
    if not (
        trace["probe"] == "FOOT-SERVO-01.v1"
        and trace["body_schema_hash"]
        == descriptor["body_schema_hash"]
        == baseline["body_schema_hash"]
        and trace["compiled_descriptor_hash"] == baseline["compiled_descriptor_hash"]
    ):
        raise ValueError("identity mismatch")
    if [(t["height_um"], t["side"]) for t in trace["trials"]] != [
        (h, s) for h in (0, 10_000_000) for s in ("none", "left", "right")
    ]:
        raise ValueError("trial coverage")
    compare_baseline(trace["trials"][0], baseline)
    report = {
        "probe": trace["probe"],
        "ground_control_exact_steps": len(trace["trials"][0]["frames"]),
        "trials": [summarize(t, descriptor) for t in trace["trials"]],
        "sha256": {
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.descriptor, args.baseline, args.trace, Path(__file__))
        },
    }
    args.output.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
