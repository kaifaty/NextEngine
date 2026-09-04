#!/usr/bin/env python3
"""Finite report-only composition of the already demonstrated single-foot action."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import torch
from next_lab.canonical_ppo import CanonicalVecEnv
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from next_lab.walking_action_basis import mirror_action_tape, walking_action_basis_tape

ROOT = Path(__file__).resolve().parents[2]


def mirror_target(target):
    raw = np.rint(np.asarray(target, dtype=np.float64) * (1 << 30)).astype(np.int64)
    return mirror_action_tape(raw).astype(np.float64) / (1 << 30)


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--headless", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = require_external_path(args.output, ROOT, label="report", must_exist=False)
    if output.exists():
        raise ValueError("output must be fresh")
    descriptor = json.loads(args.descriptor.read_text())
    if (
        sha256_file(args.descriptor)
        != "0f4610fe25d52cd128b651a1533544f45bf20c855dd18d917e82eaf65fdcdb03"
    ):
        raise ValueError("V5 descriptor hash mismatch")
    left = walking_action_basis_tape()[1].astype(np.float64) / (1 << 30)
    prep, peak = left[54], left[104]
    cases, tapes = [], []
    joints_by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    joints = [
        joints_by_id[actuator["joint_id"]] for actuator in descriptor["actuators"]
    ]
    for duration in (0.5, 1.0, 1.5):
        for amplitude in (0.5, 0.75, 1.0):
            targets = [np.zeros(23)]
            current = targets[0]
            # Return the lifted foot before transferring weight to the other side.
            for _ in range(8):
                for side in (0, 1):
                    side_prep = prep if side == 0 else mirror_target(prep)
                    side_peak = prep + amplitude * (peak - prep)
                    if side:
                        side_peak = mirror_target(side_peak)
                    for target, ticks in (
                        (side_prep, 40),
                        (side_prep, 15),
                        (side_peak, 15),
                        (side_peak, 35),
                        (side_prep, 25),
                    ):
                        length = max(1, round(ticks * duration))
                        targets.extend(
                            current + (target - current) * step / length
                            for step in range(1, length + 1)
                        )
                        current = target
            tapes.append(np.asarray(targets[1:601], dtype=np.float32))
            cases.append(
                {
                    "duration_scale": duration,
                    "swing_scale": amplitude,
                    "ticks": 0,
                    "longest_single_support_ticks": [0, 0],
                    "support_switches": 0,
                }
            )
    env = CanonicalVecEnv(
        args.headless.resolve(),
        descriptor,
        num_envs=len(cases),
        shards=1,
        run_root="32" * 32,
        device="cpu",
    )
    active = np.ones(len(cases), dtype=bool)
    runs = np.zeros((len(cases), 2), dtype=int)
    last = [None] * len(cases)
    try:
        for tick in range(600):
            env.step(torch.from_numpy(np.stack([tape[tick] for tape in tapes])))
            for slot in np.flatnonzero(active):
                record, result = cases[slot], env.last_steps[slot]
                record["ticks"] += 1
                for side in (0, 1):
                    single = (
                        result.contact_flags[side]
                        and not result.contact_flags[1 - side]
                    )
                    runs[slot, side] = runs[slot, side] + 1 if single else 0
                    record["longest_single_support_ticks"][side] = max(
                        record["longest_single_support_ticks"][side],
                        int(runs[slot, side]),
                    )
                    if runs[slot, side] == 8:
                        if last[slot] is not None and last[slot] != side:
                            record["support_switches"] += 1
                        last[slot] = side
                record["forward_m"] = float(result.root_position_micrometres[2] / 1e6)
                record["terminal"] = result.terminal_reason_id
                if result.terminated or result.truncated:
                    joint_facts = []
                    for index, joint in enumerate(joints):
                        position = int(result.joint_position_microradians[index])
                        velocity = int(
                            result.joint_velocity_microradians_per_second[index]
                        )
                        low, high = joint["hard_limit_microradians"]
                        joint_facts.append(
                            {
                                "joint_id": joint["joint_id"],
                                "position_urad": position,
                                "hard_rom_excess_urad": max(
                                    0, low - position, position - high
                                ),
                                "velocity_urad_per_second": velocity,
                                "velocity_limit_ratio": abs(velocity)
                                / joint["maximum_velocity_microradians_per_second"],
                            }
                        )
                    record["terminal_joints"] = joint_facts
                    active[slot] = False
            if not active.any():
                break
    finally:
        env.close()
    report = {
        "schema": "nextengine.canonical-alternation-probe.v1",
        "claim": "finite composition diagnostic, not learned gait or general infeasibility",
        "maximum_ticks": 600,
        "headless_sha256": sha256_file(args.headless),
        "descriptor_sha256": sha256_file(args.descriptor),
        "tool_sha256": sha256_file(Path(__file__)),
        "cases": cases,
    }
    atomic_write_json(output, report)
    print(json.dumps(cases, indent=2))


if __name__ == "__main__":
    main()
