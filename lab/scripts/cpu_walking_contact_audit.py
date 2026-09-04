#!/usr/bin/env python3
"""Replay a closed evaluation; invoke with ``python -m lab.scripts.cpu_walking_contact_audit``."""

from __future__ import annotations

import argparse
import json
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
        pose = {
            link["body_token"]: np.asarray(link["position_um"]) / 1e6
            for link in trace["frames"][0][tick]["links"]
        }
        links = {link["body_token"]: link for link in trace["frames"][0][tick]["links"]}
        bodies = descriptor["bodies"]
        origin = pose[bodies[0]["body_token"]]
        for row, horizontal in enumerate((0, 2)):
            ax = axes[row, column]
            for body in bodies:
                parent = body["parent_body_slot"]
                if parent is None:
                    continue
                a, b = pose[bodies[parent]["body_token"]], pose[body["body_token"]]
                color = (
                    "#2684d9"
                    if ".left-" in body["body_id"]
                    else "#e6699b"
                    if ".right-" in body["body_id"]
                    else "#d8a600"
                )
                ax.plot(
                    [
                        a[horizontal] - origin[horizontal],
                        b[horizontal] - origin[horizontal],
                    ],
                    [a[1], b[1]],
                    color=color,
                    linewidth=3,
                )
                if body["body_id"] in ("body.left-ankle-roll", "body.right-ankle-roll"):
                    corners, _, _, _ = foot_box(body, links[body["body_token"]])
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
                            corners[[first, second], horizontal] - origin[horizontal],
                            corners[[first, second], 1],
                            color=color,
                            linewidth=1,
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
    fig.suptitle(
        "Native body origins + actual sole collider boxes (not a surface mesh)"
    )
    fig.tight_layout()
    fig.savefig(output / "native-frames.png", dpi=130)
    plt.close(fig)

    # A close-up is necessary: full-body scaling hides millimetre heel/toe gaps.
    fig, axes = plt.subplots(2, 6, figsize=(15, 5), sharey=True)
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
                ylim=(-8, 100),
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
