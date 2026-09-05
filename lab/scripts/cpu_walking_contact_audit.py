#!/usr/bin/env python3
"""Replay a closed evaluation; invoke with ``python -m lab.scripts.cpu_walking_contact_audit``."""

from __future__ import annotations

import argparse
import json
from fractions import Fraction
from pathlib import Path

import numpy as np
from next_lab.canonical_ppo import shard_root
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from next_lab.motion_math import collider_minimum_y, quaternion_to_matrix
from next_lab.motor_lab_client import normalized_action_to_raw

from lab.scripts.canonical_walking_ppo import seed_root
from lab.scripts.native_body_geometry import physical_geometry

ROOT = Path(__file__).resolve().parents[2]


def inputs(run, seed):
    manifest_path = run / "run-manifest.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest["status"] != "completed":
        raise ValueError("evaluation requires a completed closed run")
    if seed not in manifest["profile"]["evaluation"]["seeds"]:
        raise ValueError("seed is not in the declared evaluation matrix")
    descriptor_path = Path(manifest["descriptor_path"])
    if sha256_file(descriptor_path) != manifest["descriptor_sha256"]:
        raise ValueError("descriptor hash mismatch")
    evaluation_path = run / f"evaluation-{seed}.npz"
    if sha256_file(evaluation_path) != manifest["artifacts"][evaluation_path.name]:
        raise ValueError("evaluation hash mismatch")
    with np.load(evaluation_path, allow_pickle=False) as data:
        evaluation = {key: data[key].copy() for key in data.files}
    return (
        manifest_path,
        manifest,
        json.loads(descriptor_path.read_text()),
        evaluation_path,
        evaluation,
    )


def analyze(trace, descriptor, evaluation):
    frames = trace["frames"][0]
    bodies = descriptor["bodies"]
    root_token = bodies[0]["body_token"]
    poses = [{link["body_token"]: link for link in frame["links"]} for frame in frames]
    for field, key, scale in (
        ("position_um", "root_position_m", 1_000_000),
        ("rotation_q1_30", "root_quaternion_xyzw", 1 << 30),
        ("linear_velocity_um_s", "root_velocity_mps", 1_000_000),
    ):
        np.testing.assert_array_equal(
            [pose[root_token][field] for pose in poses],
            np.rint(evaluation[key] * scale).astype(np.int64),
            err_msg=f"native replay mismatch: {field}",
        )
    dofs = [actuator["dof_ordinal"] for actuator in descriptor["actuators"]]
    np.testing.assert_array_equal(
        np.asarray([frame["joint_position_urad"] for frame in frames])[:, dofs],
        np.rint(evaluation["joint_position_rad"] * 1_000_000).astype(np.int64),
    )
    np.testing.assert_array_equal(
        [frame["contact_flags"] for frame in frames], evaluation["contact_occupancy"]
    )
    np.testing.assert_array_equal(
        [frame["command_raw"] for frame in frames],
        np.rint(evaluation["command"] * 1_000_000).astype(np.int64),
    )
    clearance, impulses = foot_measurements(frames, descriptor)
    moving = np.any(evaluation["command"] != 0, axis=1)
    contacts = evaluation["contact_occupancy"]
    report = {
        "exact_native_replay": True,
        "ticks": len(frames),
        "moving_ticks": int(moving.sum()),
        "measurement": "post-step native collider minimum Y; impulses from last physical substep, not a whole-tick support test",
        "feet": [],
    }
    for side, name in enumerate(("left", "right")):
        values = clearance[moving, side]
        report["feet"].append(
            {
                "side": name,
                "moving_clearance_min_m": float(values.min()) if len(values) else None,
                "moving_clearance_max_m": float(values.max()) if len(values) else None,
                "moving_clearance_p95_m": float(np.percentile(values, 95))
                if len(values)
                else None,
                "moving_clearance_counts_report_only": {
                    str(level): int(np.sum(values > level))
                    for level in (0.0, 0.005, 0.02, 0.04)
                },
                "moving_contact_bit_with_zero_last_substep_vertical_impulse": int(
                    np.sum(moving & contacts[:, side] & (impulses[:, side] == 0))
                ),
                "moving_contact_bit_with_clearance_above_5mm": int(
                    np.sum(moving & contacts[:, side] & (clearance[:, side] > 0.005))
                ),
            }
        )
    return report, clearance, impulses


def foot_measurements(frames, descriptor):
    """Measure shape clearance independently of the contact-presence flags."""
    bodies = descriptor["bodies"]
    poses = [{link["body_token"]: link for link in frame["links"]} for frame in frames]
    feet = [
        next(body for body in bodies if body["body_id"] == f"body.{side}-ankle-roll")
        for side in ("left", "right")
    ]
    clearance = np.empty((len(frames), 2))
    impulses = np.zeros((len(frames), 2))
    for tick, (frame, pose) in enumerate(zip(frames, poses, strict=True)):
        for side, foot in enumerate(feet):
            link = pose[foot["body_token"]]
            clearance[tick, side] = min(
                collider_minimum_y(
                    np.asarray(link["position_um"]) / 1e6,
                    quaternion_to_matrix(
                        np.asarray(link["rotation_q1_30"]) / (1 << 30)
                    ),
                    collider,
                )
                for collider in foot["colliders"]
            )
            # Frozen canonical ground actor token is 1. Keep all original contact bits.
            pair = {1, foot["body_token"]}
            impulses[tick, side] = sum(
                abs(contact["impulse_uns"][1])
                for contact in frame["contacts"]
                if set(contact["actor_tokens"]) == pair
            )
    return clearance, impulses


def classified_support(frames, descriptor):
    """Report existing classified load over actual substeps, never a gait gate.

    Positive vertical impulse is an exact measurement, not a newly selected
    minimum support force. Partial/zero-step terminal ticks cannot establish
    full-tick support. Recorded classifier-padding is forbidden here.
    """
    tokens = [
        next(
            b["body_token"]
            for b in descriptor["bodies"]
            if b["body_id"] == f"body.{side}-ankle-roll"
        )
        for side in ("left", "right")
    ]
    result = []
    for frame in frames:
        count = frame["completed_physics_substeps"]
        substeps = frame["classified_contact_substeps"]
        if type(count) is not int or not 0 <= count <= 4 or len(substeps) != count:
            raise ValueError("actual substep count mismatch")
        impulses = []
        active = []
        for ordinal, substep in enumerate(substeps):
            if substep["ordinal"] != ordinal:
                raise ValueError("substep order mismatch")
            loads = [0, 0]
            present = [False, False]
            for contact in substep["contacts"]:
                if contact["class"] != "SoleSupport":
                    continue
                for side, token in enumerate(tokens):
                    if set(contact["actor_tokens"]) == {1, token}:
                        vertical = contact["impulse_uns"][1]
                        if type(vertical) is not int:
                            raise ValueError("vertical impulse must be integer")
                        present[side] = True
                        loads[side] += abs(vertical)
            impulses.append(loads)
            active.append(present)
        exclusive = [
            (0 if load[0] > 0 else 1) if (load[0] > 0) != (load[1] > 0) else None
            for load in impulses
        ]
        result.append(
            {
                "tick": frame["tick"],
                "actual_substeps": count,
                "active_sole_contacts_by_substep": active,
                "vertical_impulse_uns_by_substep": impulses,
                "exclusive_positive_load_side_all_four_substeps_report_only": exclusive[
                    0
                ]
                if count == 4
                and exclusive[0] is not None
                and all(side == exclusive[0] for side in exclusive)
                else None,
            }
        )
    return result


def foot_box(body, link):
    """World-space eight box corners: first four are the sole, heel then toe."""
    if len(body["colliders"]) != 1:
        raise ValueError("sole audit requires one box per foot")
    collider = body["colliders"][0]
    if collider["geometry"]["kind"] != "box":
        raise ValueError("sole audit requires a box collider")
    rotation = quaternion_to_matrix(np.asarray(link["rotation_q1_30"]) / (1 << 30))
    local_rotation = quaternion_to_matrix(
        np.asarray(collider["local_rotation_q1_30"]) / (1 << 30)
    )
    center = np.asarray(link["position_um"]) / 1e6 + rotation @ (
        np.asarray(collider["local_translation_micrometres"]) / 1e6
    )
    rotation = rotation @ local_rotation
    half = np.asarray(collider["geometry"]["half_extents_micrometres"]) / 1e6
    signs = np.array(
        [
            [-1, -1, -1],
            [1, -1, -1],
            [1, -1, 1],
            [-1, -1, 1],
            [-1, 1, -1],
            [1, 1, -1],
            [1, 1, 1],
            [-1, 1, 1],
        ]
    )
    return center + (signs * half) @ rotation.T, center, rotation, half


def lifted_support_report(frames, descriptor):
    """Diagnostic candidate, NOT a replacement for any frozen walking gate.

    Require exclusive classified vertical load over four actual substeps and
    positive canonical post-step minimum height of the other complete foot.
    The latter is a post-step geometric sample, not four substep poses.
    Keep both load-only and lift-qualified counts so unloading cannot be
    mistaken for release. Eight ticks is the existing duration requirement.
    """
    profiles = descriptor["environment_profiles"]
    allowed = {
        f"nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v{v}"
        for v in (7, 8)
    }
    if (
        descriptor["observation_width"] != 88
        or len(profiles) != 1
        or profiles[0]["profile_id"] not in allowed
    ):
        raise ValueError("lifted support requires the V7/V8 sole-height layout")
    if not frames:
        raise ValueError("support report requires nonempty frames")
    rows = classified_support(frames, descriptor)
    heights = []
    for index, frame in enumerate(frames):
        if type(frame["tick"]) is not int or frame["tick"] != index + 1:
            raise ValueError("support report requires contiguous ticks from one")
        raw = frame["observation_raw"]
        if len(raw) != 88 or any(type(value) is not int for value in raw[86:88]):
            raise ValueError("sole heights must be two raw integer micrometre values")
        heights.append(raw[86:88])
    clearance, _ = foot_measurements(frames, descriptor)
    if not np.isfinite(clearance).all():
        raise ValueError("non-finite independent sole geometry")
    # Normalized floating rendering and canonical Q30 projection differ slightly.
    # Report that difference, but verify raw heights with exact integer arithmetic
    # rather than accepting a tuned floating tolerance around the ground plane.
    error_um = np.abs(clearance * 1e6 - np.asarray(heights))
    for frame, height in zip(frames, heights, strict=True):
        links = {link["body_token"]: link for link in frame["links"]}
        for side, name in enumerate(("left", "right")):
            body = next(
                b
                for b in descriptor["bodies"]
                if b["body_id"] == f"body.{name}-ankle-roll"
            )
            if canonical_box_height_um(body, links[body["body_token"]]) != height[side]:
                raise ValueError("native sole height disagrees with exact box geometry")
    for row, height in zip(rows, heights, strict=True):
        side = row["exclusive_positive_load_side_all_four_substeps_report_only"]
        row["post_step_whole_foot_minimum_y_um"] = height
        row["lift_qualified_stance_side_report_only"] = (
            side if side is not None and height[1 - side] > 0 else None
        )

    def summarize(key):
        counts, longest = [0, 0], [0, 0]
        current, length, previous_qualified, switches = None, 0, None, 0
        segments = []
        for row in rows:
            side = row[key]
            if side is None:
                current, length = None, 0
                continue
            counts[side] += 1
            if side != current:
                segments.append({"side": side, "first_tick": row["tick"], "ticks": 0})
            length = length + 1 if side == current else 1
            current = side
            segments[-1]["ticks"] = length
            longest[side] = max(longest[side], length)
            if length == 8:
                if previous_qualified is not None and side != previous_qualified:
                    switches += 1
                previous_qualified = side
        return {
            "ticks_by_stance_side": counts,
            "longest_continuous_ticks_by_stance_side": longest,
            "switches_between_runs_of_at_least_eight_ticks": switches,
            "segments": segments,
        }

    return {
        "status": "report_only",
        "claim": "four-substep exclusive load plus post-step whole-foot release; no gate promotion",
        "ticks": len(frames),
        "maximum_independent_geometry_error_um": float(error_um.max()),
        "exact_canonical_geometry": True,
        "load_only": summarize(
            "exclusive_positive_load_side_all_four_substeps_report_only"
        ),
        "load_and_lift": summarize("lift_qualified_stance_side_report_only"),
        "frames": rows,
    }


def canonical_box_height_um(body, link):
    """Exact oracle for walking_box_minimum_y_um, including ties-even then floor.

    Do not normalize the stored Q30 quaternion: the canonical rotation helper
    projects its quantized coefficients directly. Floating plots normalize it.
    """
    one = 1 << 30
    if len(body["colliders"]) != 1:
        raise ValueError("canonical sole height requires one box")
    collider = body["colliders"][0]
    if collider["geometry"]["kind"] != "box" or collider["local_rotation_q1_30"] != [
        0,
        0,
        0,
        one,
    ]:
        raise ValueError("canonical sole height requires identity-local-rotation box")
    q = link["rotation_q1_30"]
    if len(q) != 4 or any(type(v) is not int or abs(v) > one for v in q):
        raise ValueError("invalid Q30 quaternion")
    x, y, z, w = q
    row = (
        round(Fraction(2 * (x * y + z * w), one)),
        one - round(Fraction(2 * (x * x + z * z), one)),
        round(Fraction(2 * (y * z - x * w), one)),
    )
    offset = collider["local_translation_micrometres"]
    half = collider["geometry"]["half_extents_micrometres"]
    position = link["position_um"]
    if (
        any(len(v) != 3 for v in (offset, half, position))
        or any(
            type(v) is not int or not -(1 << 63) <= v < (1 << 63)
            for values in (offset, half, position)
            for v in values
        )
        or any(v <= 0 for v in half)
    ):
        raise ValueError("invalid integer box geometry")
    value = (
        position[1]
        + sum(r * o - abs(r) * h for r, o, h in zip(row, offset, half, strict=True))
        // one
    )
    if not -(1 << 63) <= value < (1 << 63):
        raise ValueError("canonical box height overflow")
    return value


def sole_support(frames, descriptor):
    """Report geometry and force-weighted contact location; no change to gait gates."""
    result = []
    moving = np.any([frame["command_raw"] for frame in frames], axis=1)
    for name in ("left", "right"):
        body = next(
            b for b in descriptor["bodies"] if b["body_id"] == f"body.{name}-ankle-roll"
        )
        values = []
        for frame in frames:
            link = next(
                link
                for link in frame["links"]
                if link["body_token"] == body["body_token"]
            )
            corners, center, rotation, half = foot_box(body, link)
            heel, toe = corners[:2].mean(axis=0), corners[2:4].mean(axis=0)
            contacts = [
                c
                for c in frame["contacts"]
                if set(c["actor_tokens"]) == {1, body["body_token"]}
            ]
            impulse = sum(abs(c["impulse_uns"][1]) for c in contacts)
            cop = None
            if impulse:
                cop_world = (
                    sum(
                        np.asarray(c["position_um"]) / 1e6 * abs(c["impulse_uns"][1])
                        for c in contacts
                    )
                    / impulse
                )
                # 0 = geometric heel end, 1 = toe end; no clamping of measured points.
                cop = float(
                    ((rotation.T @ (cop_world - center))[2] + half[2]) / (2 * half[2])
                )
            values.append(
                {
                    "tick": frame["tick"],
                    "heel_min_y_m": float(corners[:2, 1].min()),
                    "toe_min_y_m": float(corners[2:4, 1].min()),
                    "heel_above_toe_m": float(heel[1] - toe[1]),
                    "sole_corner_span_y_m": float(np.ptp(corners[:4, 1])),
                    "toe_down_angle_deg": float(
                        np.degrees(
                            np.arctan2(
                                heel[1] - toe[1], np.linalg.norm((toe - heel)[[0, 2]])
                            )
                        )
                    ),
                    "last_substep_vertical_force_n": impulse * 240 / 1e6,
                    "pressure_heel_to_toe_fraction": cop,
                }
            )
        phases = {}
        for phase, mask in (
            ("zero_command", ~moving),
            ("moving_command", moving),
        ):
            selected = [value for value, keep in zip(values, mask, strict=True) if keep]
            phases[phase] = {
                "ticks": len(selected),
                "maximum_heel_above_toe_m": max(
                    (v["heel_above_toe_m"] for v in selected), default=None
                ),
                "maximum_toe_down_angle_deg": max(
                    (v["toe_down_angle_deg"] for v in selected), default=None
                ),
                "sole_span_below_1mm_ticks_report_only": sum(
                    v["sole_corner_span_y_m"] < 0.001 for v in selected
                ),
                "heel_raised_over_5mm_and_pressure_front_quarter_ticks_report_only": sum(
                    v["heel_above_toe_m"] > 0.005
                    and v["pressure_heel_to_toe_fraction"] is not None
                    and v["pressure_heel_to_toe_fraction"] > 0.75
                    for v in selected
                ),
            }
        chosen = sorted(
            set(
                np.linspace(0, len(frames) - 1, 6, dtype=int).tolist()
                + [int(np.argmax([v["heel_above_toe_m"] for v in values]))]
            )
        )
        result.append(
            {
                "side": name,
                "phases": phases,
                "selected_frames_including_maximum_heel_raise": [
                    values[i] for i in chosen
                ],
            }
        )
    return result


def plot(descriptor, evaluation, trace, clearance, impulses, output):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    times = (np.arange(len(clearance)) + 1) / 60
    fig, axes = plt.subplots(3, 1, figsize=(12, 8), sharex=True)
    for side, name in enumerate(("left", "right")):
        axes[0].plot(times, clearance[:, side] * 100, label=name)
        axes[1].plot(times, impulses[:, side] * 240 / 1e6, label=name)
    axes[0].axhline(0, color="black", linewidth=0.7)
    axes[0].set_ylabel("Collider clearance, cm")
    axes[1].set_ylabel("Last-substep vertical force, N")
    axes[2].plot(
        times, evaluation["root_velocity_mps"][:, 2], label="actual forward speed"
    )
    axes[2].plot(times, evaluation["command"][:, 1], label="command")
    axes[2].set_ylabel("Speed, m/s")
    axes[2].set_xlabel("Time, s")
    for axis in axes:
        axis.legend(loc="upper right")
        axis.grid(alpha=0.2)
    fig.suptitle(
        "Recorded canonical evaluation: clearance is not a contact-presence bit"
    )
    fig.tight_layout()
    fig.savefig(output / "clearance-and-force.png", dpi=130)
    plt.close(fig)

    chosen = np.linspace(0, len(trace["frames"][0]) - 1, 6, dtype=int)
    fig, axes = plt.subplots(2, 6, figsize=(16, 6))
    for column, tick in enumerate(chosen):
        shapes, origins = physical_geometry(descriptor, trace["frames"][0][tick])
        origin = origins["body.pelvis"]
        for row, horizontal in enumerate((0, 2)):
            ax = axes[row, column]
            for shape in shapes:
                segments = shape["segments"]
                ax.plot(
                    (segments[:, :, horizontal] - origin[horizontal]).T,
                    segments[:, :, 1].T,
                    color=shape["color"],
                    linewidth=0.8,
                )
            ax.axhline(0, color="gray", linewidth=1)
            ax.set(
                xlim=(-0.7, 0.7),
                ylim=(-0.05, 2),
                aspect="equal",
                title=f"{(tick + 1) / 60:.2f} s",
            )
            ax.set_xticks([])
            if column:
                ax.set_yticks([])
            else:
                ax.set_ylabel("Front / Y, m" if row == 0 else "Side / Y, m")
    fig.suptitle("All actual physical colliders in native poses (not a skin mesh)")
    fig.tight_layout()
    fig.savefig(output / "native-frames.png", dpi=130)
    plt.close(fig)

    # A close-up is necessary: full-body scaling hides millimetre heel/toe gaps.
    fig, axes = plt.subplots(2, 6, figsize=(15, 5), sharey=True)
    # One shared range must include every selected foot, including the swing.
    # The old fixed 100 mm ceiling cut off lifted toes and hid their shape.
    selected_heights = []
    for tick in chosen:
        links = {link["body_token"]: link for link in trace["frames"][0][tick]["links"]}
        for body in descriptor["bodies"]:
            if body["body_id"] in ("body.left-ankle-roll", "body.right-ankle-roll"):
                corners, _, _, _ = foot_box(body, links[body["body_token"]])
                selected_heights.extend(corners[:, 1] * 1000)
    sole_ylim = (
        min(-8, min(selected_heights) - 5),
        max(100, max(selected_heights) + 5),
    )
    for row, side in enumerate(("left", "right")):
        body = next(
            b for b in descriptor["bodies"] if b["body_id"] == f"body.{side}-ankle-roll"
        )
        color = "#2684d9" if row == 0 else "#e6699b"
        for column, tick in enumerate(chosen):
            frame = trace["frames"][0][tick]
            link = next(
                l for l in frame["links"] if l["body_token"] == body["body_token"]
            )
            corners, center, rotation, _ = foot_box(body, link)
            forward = rotation[:, 2].copy()
            forward[1] = 0
            forward /= np.linalg.norm(forward)
            horizontal = (corners - center) @ forward * 1000
            ax = axes[row, column]
            for first, second in (
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ):
                ax.plot(
                    horizontal[[first, second]],
                    corners[[first, second], 1] * 1000,
                    color=color,
                )
            for contact in frame["contacts"]:
                if (
                    set(contact["actor_tokens"]) == {1, body["body_token"]}
                    and abs(contact["impulse_uns"][1]) > 0
                ):
                    point = np.asarray(contact["position_um"]) / 1e6
                    ax.scatter(
                        (point - center) @ forward * 1000,
                        point[1] * 1000,
                        color="black",
                        s=12,
                    )
            ax.axhline(0, color="gray", linewidth=1)
            ax.set(
                xlim=(-160, 160),
                ylim=sole_ylim,
                aspect="equal",
                title=f"{(tick + 1) / 60:.2f} s",
            )
            if column == 0:
                ax.set_ylabel(f"{side} / height, mm")
            if row == 1:
                ax.set_xlabel("heel <— mm —> toe")
    fig.suptitle(
        "Actual sole boxes: black dots = contacts with nonzero vertical impulse"
    )
    fig.tight_layout()
    fig.savefig(output / "sole-closeups.png", dpi=130)
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("mode", choices=("prepare", "analyze"))
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=1001)
    parser.add_argument("--tape", type=Path, required=True)
    parser.add_argument("--trace", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    run = require_external_path(args.run, ROOT, label="run")
    manifest_path, manifest, descriptor, evaluation_path, evaluation = inputs(
        run, args.seed
    )
    tape = require_external_path(
        args.tape, ROOT, label="tape", must_exist=args.mode != "prepare"
    )
    if args.mode == "prepare":
        if tape.exists():
            raise ValueError("tape must be fresh")
        atomic_write_json(
            tape,
            {
                "profile_id": manifest["profile"]["environment_profile_id"],
                "run_root_bytes": list(
                    bytes.fromhex(shard_root(seed_root(args.seed), 0))
                ),
                "retain_frames": True,
                "action_q1_30": [
                    normalized_action_to_raw(
                        evaluation["action"], 23, q1_30=True
                    ).tolist()
                ],
                "source_run_manifest_sha256": sha256_file(manifest_path),
                "source_evaluation_sha256": sha256_file(evaluation_path),
            },
        )
        return
    if args.trace is None or args.output is None:
        raise ValueError("analysis requires trace and fresh output directory")
    trace_path = require_external_path(args.trace, ROOT, label="trace")
    output = require_external_path(args.output, ROOT, label="audit", must_exist=False)
    if output.exists():
        raise ValueError("audit directory must be fresh")
    data = json.loads(tape.read_text())
    trace = json.loads(trace_path.read_text())
    if data["source_run_manifest_sha256"] != sha256_file(manifest_path) or data[
        "source_evaluation_sha256"
    ] != sha256_file(evaluation_path):
        raise ValueError("source lineage mismatch")
    if (
        trace["action_tape_sha256"] != sha256_file(tape)
        or trace["profile_id"] != data["profile_id"]
        or trace["run_root"] != bytes(data["run_root_bytes"]).hex()
    ):
        raise ValueError("native replay input mismatch")
    report, clearance, impulses = analyze(trace, descriptor, evaluation)
    report["sole_support"] = sole_support(trace["frames"][0], descriptor)
    output.mkdir(parents=True, exist_ok=False)
    plot(descriptor, evaluation, trace, clearance, impulses, output)
    report.update(
        {
            "native_trace_sha256": sha256_file(trace_path),
            "source_evaluation_sha256": sha256_file(evaluation_path),
            "tool_sha256": sha256_file(Path(__file__)),
        }
    )
    atomic_write_json(output / "report.json", report)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
