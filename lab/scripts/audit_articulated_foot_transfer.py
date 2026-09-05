"""Bounded loaded-foot transfer oracle; not a balance or learning admission gate."""

import argparse
import hashlib
import json
import math
from pathlib import Path

# Frozen observed-state tolerance from motor/safety_control.rs, not a new limit.
OBSERVED_VELOCITY_TOLERANCE_URAD_S = 1_000


def velocity_violations(frame, limits):
    return [
        {"joint_id": limits[j["ordinal"]]["joint_id"], **j}
        for j in frame["joints"]
        if j["ordinal"] in limits
        and abs(j["velocity_urad_s"])
        > limits[j["ordinal"]]["maximum_velocity_microradians_per_second"]
        + OBSERVED_VELOCITY_TOLERANCE_URAD_S
    ]


def rotate(q, v):
    """Independent scalar quaternion cross-product transform, diagnostic f64."""
    norm = math.sqrt(sum(float(a) ** 2 for a in q))
    if not math.isfinite(norm) or norm == 0:
        raise ValueError("invalid quaternion")
    x, y, z, w = [a / norm for a in q]
    a, b, c = v
    tx, ty, tz = 2 * (y * c - z * b), 2 * (z * a - x * c), 2 * (x * b - y * a)
    return [
        a + w * tx + y * tz - z * ty,
        b + w * ty + z * tx - x * tz,
        c + w * tz + x * ty - y * tx,
    ]


def sole_heights(body, link):
    if len(body["colliders"]) != 1:
        raise ValueError("expected one segment collider")
    shape = body["colliders"][0]
    if shape["geometry"]["kind"] != "box":
        raise ValueError("expected box segment")
    hx, hy, hz = [x / 1e6 for x in shape["geometry"]["half_extents_micrometres"]]
    heights = []
    # First two are rear/heel edge. Include body AND collider rotations/offsets.
    for z in [-hz, hz]:
        for x in [-hx, hx]:
            corner = rotate(shape["local_rotation_q1_30"], [x, -hy, z])
            corner = [
                v + o / 1e6
                for v, o in zip(
                    corner, shape["local_translation_micrometres"], strict=True
                )
            ]
            heights.append(
                link["position_um"][1] / 1e6 + rotate(link["rotation_q1_30"], corner)[1]
            )
    return heights


def vertical_impulse(frame, actor):
    total = 0
    for contact in frame["contacts"]:
        if contact["actors"] == [1, actor]:
            total += contact["impulse_uns"][1]
        elif contact["actors"] == [actor, 1]:
            total -= contact["impulse_uns"][1]
    return abs(total) / 1e6


def longest(flags):
    result = current = 0
    for flag in flags:
        current = current + 1 if flag else 0
        result = max(result, current)
    return result


def analyze(descriptor, trace):
    if (
        trace["probe"]
        not in (
            "nextengine.articulated-foot-transfer.v1",
            "nextengine.articulated-foot-transfer.v2",
        )
        or trace["body_schema_hash"] != descriptor["body_schema_hash"]
    ):
        raise ValueError("profile mismatch")
    frames = trace["frames"]
    if not frames or [f["physics_substep"] for f in frames] != list(
        range(1, len(frames) + 1)
    ):
        raise ValueError("noncontiguous physical frames")
    if any(f["motor_tick"] != (f["physics_substep"] + 3) // 4 for f in frames):
        raise ValueError("motor/physics phase mismatch")
    bodies = {b["body_id"]: b for b in descriptor["bodies"]}
    result = {
        "side": trace["side"],
        "reason": trace["reason"],
        "physics_substeps": len(frames),
        "sides": {},
    }
    limits = {j["dof_ordinal"]: j for j in descriptor.get("joints", [])}
    result["first_velocity_violation"] = None
    for frame in frames:
        violations = velocity_violations(frame, limits)
        if violations:
            result["first_velocity_violation"] = {
                "step": frame["physics_substep"],
                "offset_urad": frame["offset_urad"],
                "joints": violations,
            }
            break
    for side in ["left", "right"]:
        rear, toe = [bodies[f"body.{side}-{name}"] for name in ["ankle-roll", "mtp"]]
        raised, returned, rows = [], [], []
        for frame in frames:
            links = {l["body_token"]: l for l in frame["links"]}
            if len(links) != len(bodies) or set(links) != {
                b["body_token"] for b in bodies.values()
            }:
                raise ValueError("body token mismatch")
            heel = sole_heights(rear, links[rear["body_token"]])[:2]
            front = sole_heights(toe, links[toe["body_token"]])
            rear_load, toe_load = [
                vertical_impulse(frame, b["body_token"]) for b in [rear, toe]
            ]
            tick = frame["motor_tick"]
            raised.append(
                300 < tick <= 540
                and min(heel) > 0.005
                and min(front) >= -0.001
                and max(front) <= 0.002
                and toe_load >= 0.05
            )
            returned.append(
                tick > 540
                and min(heel) >= -0.001
                and max(heel) <= 0.001
                and rear_load >= 0.05
            )
            rows.append(
                {
                    "step": frame["physics_substep"],
                    "heel_min_m": min(heel),
                    "forefoot_min_m": min(front),
                    "forefoot_max_m": max(front),
                    "rear_impulse_Ns": rear_load,
                    "toe_impulse_Ns": toe_load,
                }
            )
        peak = max(rows, key=lambda r: r["heel_min_m"])
        result["sides"][side] = {
            "maximum_heel_height_frame": peak,
            "longest_loaded_heel_raise_substeps": longest(raised),
            "longest_recontact_substeps": longest(returned),
            "maximum_forefoot_sole_height_m": max(r["forefoot_max_m"] for r in rows),
            "loaded_transfer_pass": longest(raised) >= 60
            and longest(returned) >= 240
            and trace["reason"] == "terminal.timeout"
            and len(frames) == 3600,
        }
    return result


def compare_control(trace, baseline):
    if (
        trace["side"] != "none"
        or trace["reason"] != "terminal.timeout"
        or len(trace["frames"]) != 3600
    ):
        raise ValueError("incomplete control")
    if len(baseline["substep_samples"]) < 3600 or len(baseline["samples"]) < 901:
        raise ValueError("incomplete baseline")
    for key in ["body_schema_hash", "compiled_descriptor_hash", "contact_profile_hash"]:
        if trace[key] != baseline[key]:
            raise ValueError(f"control identity: {key}")
    for frame, old in zip(
        trace["frames"], baseline["substep_samples"][:3600], strict=True
    ):
        if frame["physics_substep"] != old["physics_substep"]:
            raise ValueError("control substep mismatch")
        root = next(link for link in frame["links"] if link["body_token"] == 1000)
        for key in [
            "position_um",
            "rotation_q1_30",
            "angular_velocity_urad_s",
            "linear_velocity_um_s",
        ]:
            if root[key] != old[key]:
                raise ValueError(f"control root mismatch: {key}")
        for key in [
            "joints",
            "contacts",
            "applied_efforts_by_dof_unm",
            "anatomical_foot_impulses_uns",
        ]:
            if frame[key] != old[key]:
                raise ValueError(
                    f"control mismatch step{frame['physics_substep']} {key}"
                )
        if frame["physics_substep"] % 4 == 0:
            sample = baseline["samples"][frame["motor_tick"]]
            for key in ["links", "reference_targets_urad", "applied_targets_urad"]:
                if frame[key] != sample[key]:
                    raise ValueError(
                        f"control mismatch tick{frame['motor_tick']} {key}"
                    )
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("descriptor", type=Path)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    descriptor = json.loads(args.descriptor.read_text())
    baseline = json.loads(args.baseline.read_text())
    reports = []
    paths = [args.descriptor, args.baseline, Path(__file__)]
    for side in ["none", "left", "right"]:
        path = args.directory / f"{side}.json"
        trace = json.loads(path.read_text())
        if trace["side"] != side:
            raise ValueError("side mismatch")
        if side == "none":
            compare_control(trace, baseline)
        reports.append(analyze(descriptor, trace))
        paths.append(path)
    print(
        json.dumps(
            {
                "control_exact": True,
                "reports": reports,
                "sha256": {
                    str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths
                },
            },
            indent=2,
            allow_nan=False,
        )
    )


if __name__ == "__main__":
    main()
