#!/usr/bin/env python3
"""Run the frozen walking action-basis tape in canonical CPU PhysX."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path

import numpy as np

from next_lab.isaac_training import atomic_write_json, require_external_path
from next_lab.motor_lab_client import MotorLabClient
from next_lab.walking_action_basis import (
    ACTION_BASIS_CASES,
    ACTION_BASIS_PROFILE_ID,
    action_tape_sha256,
    evaluate_action_basis,
    walking_action_basis_tape,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--headless", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    headless = args.headless.resolve()
    if not headless.is_file():
        raise FileNotFoundError(f"headless executable does not exist: {headless}")
    output = require_external_path(
        args.output,
        REPOSITORY_ROOT,
        label="walking action-basis CPU report",
        must_exist=False,
    )
    trajectory_path = output.with_suffix(".npz")
    if output.suffix != ".json" or output.exists() or trajectory_path.exists():
        raise ValueError("audit output must be a new .json path with a new .npz companion")
    tape = walking_action_basis_tape()
    case_count, motor_steps, action_width = tape.shape
    root_position = np.empty((case_count, motor_steps, 3), dtype=np.float64)
    root_velocity = np.empty_like(root_position)
    joint_position = np.empty((case_count, motor_steps, action_width), dtype=np.float64)
    targets = np.empty_like(tape)
    contacts = np.empty((case_count, motor_steps, 2), dtype=np.bool_)
    done = np.zeros((case_count, motor_steps), dtype=np.bool_)
    terminal_reasons = np.full(case_count, "", dtype="<U128")

    with MotorLabClient(
        headless,
        ACTION_BASIS_PROFILE_ID,
        case_count,
        bytes.fromhex(action_tape_sha256(tape)),
    ) as client:
        resets = client.reset(list(range(case_count)))
        ordinals = np.asarray(
            [value.episode_ordinal for value in sorted(resets, key=lambda value: value.vector_slot)],
            dtype=np.uint64,
        )
        for tick in range(motor_steps):
            steps = client.step(ordinals, tape[:, tick])
            for step in steps:
                slot = step.vector_slot
                root_position[slot, tick] = (
                    step.root_position_micrometres.astype(np.float64) / 1_000_000.0
                )
                root_velocity[slot, tick] = (
                    step.root_linear_velocity_micrometres_per_second.astype(np.float64)
                    / 1_000_000.0
                )
                joint_position[slot, tick] = (
                    step.joint_position_microradians.astype(np.float64) / 1_000_000.0
                )
                contacts[slot, tick] = step.contact_flags
                targets[slot, tick] = step.observation_raw[56:79]
                done[slot, tick] = step.terminated or step.truncated
                if done[slot, tick] and not terminal_reasons[slot]:
                    terminal_reasons[slot] = step.terminal_reason_id or "unknown"
            if np.any(done[:, tick]):
                break

    root_position, root_velocity, joint_position, contacts, done, targets = (
        value[:, :tick + 1] for value in
        (root_position, root_velocity, joint_position, contacts, done, targets)
    )
    gate = evaluate_action_basis(
        contact_occupancy=contacts,
        root_position_m=root_position,
        done=done,
    )
    report = {
        "schema_version": 1,
        **gate,
        "plane": "canonical-cpu-physx",
        "action_tape_sha256": action_tape_sha256(tape),
        "manifest_hash": client.descriptor.manifest_hash.hex(),
        "action_layout_hash": client.descriptor.action_layout_hash.hex(),
        "terminal_reasons": dict(zip(ACTION_BASIS_CASES, terminal_reasons.tolist())),
        "trajectory_path": str(trajectory_path),
        "observed_motor_steps": tick + 1,
        "headless_sha256": _sha256(headless),
        "tool_sha256": _sha256(Path(__file__).resolve()),
        "tape_module_sha256": _sha256(REPOSITORY_ROOT / "lab/next_lab/walking_action_basis.py"),
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = trajectory_path.with_suffix(".npz.tmp")
    with temporary.open("wb") as stream:
        np.savez_compressed(
            stream,
            profile_id=np.asarray(ACTION_BASIS_PROFILE_ID),
            case_ids=np.asarray(ACTION_BASIS_CASES),
            action_raw=tape,
            applied_targets_raw=targets,
            root_position_m=root_position,
            root_velocity_mps=root_velocity,
            joint_position_rad=joint_position,
            contact_occupancy=contacts,
            done=done,
            manifest_hash=np.frombuffer(client.descriptor.manifest_hash, dtype=np.uint8),
            action_layout_hash=np.frombuffer(client.descriptor.action_layout_hash, dtype=np.uint8),
        )
    os.replace(temporary, trajectory_path)
    report["trajectory_sha256"] = _sha256(trajectory_path)
    atomic_write_json(output, report)
    print(json.dumps(report, indent=2, sort_keys=True))
    if gate["status"] != "passed":
        raise SystemExit(4)


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


if __name__ == "__main__":
    main()
