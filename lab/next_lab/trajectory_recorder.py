from __future__ import annotations

import os
from pathlib import Path
from typing import Sequence

import numpy as np

from next_lab.motor_lab_client import (
    FLAT_LOCOMOTION_PROFILE_ID,
    EnvironmentDescriptor,
    MotorLabClient,
    StepResult,
)


def record_canonical_cpu_trajectories(
    headless_executable: str | Path | Sequence[str | Path],
    training_store: Path,
    run_root: str,
    slots: int,
    episodes_per_slot: int,
    profile_id: str = FLAT_LOCOMOTION_PROFILE_ID,
    output_name: str | None = None,
) -> Path:
    store = require_external_training_store(training_store)
    if slots <= 0 or episodes_per_slot <= 0:
        raise ValueError("slots and episodes_per_slot must be positive")
    with MotorLabClient(headless_executable, profile_id, slots, run_root) as client:
        descriptor = client.descriptor
        resets = client.reset(list(range(slots)))
        episode_ordinals = np.zeros(slots, dtype=np.uint64)
        for reset in resets:
            episode_ordinals[reset.vector_slot] = reset.episode_ordinal
        completed = np.zeros(slots, dtype=np.int64)
        records: list[StepResult] = []
        maximum_records = slots * episodes_per_slot * (descriptor.maximum_episode_steps + 1)
        while np.any(completed < episodes_per_slot):
            actions = np.zeros((slots, descriptor.action_width), dtype=np.int64)
            steps = client.step(episode_ordinals, actions)
            reset_slots = []
            for step in steps:
                if completed[step.vector_slot] < episodes_per_slot:
                    records.append(step)
                if step.terminated or step.truncated:
                    if completed[step.vector_slot] < episodes_per_slot:
                        completed[step.vector_slot] += 1
                    reset_slots.append(step.vector_slot)
            if len(records) > maximum_records:
                raise RuntimeError("trajectory recorder exceeded the manifest episode bound")
            if np.all(completed >= episodes_per_slot):
                break
            if reset_slots:
                for reset in client.reset(reset_slots):
                    episode_ordinals[reset.vector_slot] = reset.episode_ordinal
        return _write_npz_v2(store, output_name, run_root, descriptor, records)


def require_external_training_store(candidate: Path) -> Path:
    repository_root = Path(__file__).resolve().parents[2]
    resolved = candidate.resolve()
    if resolved == repository_root or repository_root in resolved.parents:
        raise ValueError("training artifacts require a store outside the repository")
    return resolved


def _write_npz_v2(
    store: Path,
    output_name: str | None,
    run_root: str,
    descriptor: EnvironmentDescriptor,
    records: list[StepResult],
) -> Path:
    if not records:
        raise ValueError("trajectory recorder produced no steps")
    name = output_name or f"canonical_cpu_{run_root[:16]}.npz"
    if Path(name).name != name or not name.endswith(".npz"):
        raise ValueError("output_name must be a plain .npz filename")
    manifest_hash = descriptor.manifest_hash.hex()
    output_directory = store / "trajectories-v2" / manifest_hash
    output_directory.mkdir(parents=True, exist_ok=True)
    output_path = output_directory / name
    temporary_path = output_path.with_suffix(".npz.tmp")
    reward_ids = np.asarray(
        [component.component_id for component in descriptor.reward_components], dtype=np.str_
    )
    payload = {
        "schema_version": np.asarray(2, dtype=np.uint16),
        "profile_id": np.asarray(descriptor.profile_id, dtype=np.str_),
        "manifest_hash": np.asarray(descriptor.manifest_hash.hex(), dtype=np.str_),
        "observation_layout_hash": np.asarray(
            descriptor.observation_layout_hash.hex(), dtype=np.str_
        ),
        "action_layout_hash": np.asarray(descriptor.action_layout_hash.hex(), dtype=np.str_),
        "command_schedule_profile_hash": np.asarray(
            descriptor.command_schedule_profile_hash.hex(), dtype=np.str_
        ),
        "reward_profile_hash": np.asarray(descriptor.reward_profile_hash.hex(), dtype=np.str_),
        "termination_profile_hash": np.asarray(
            descriptor.termination_profile_hash.hex(), dtype=np.str_
        ),
        "rng_derivation_profile_hash": np.asarray(
            descriptor.rng_derivation_profile_hash.hex(), dtype=np.str_
        ),
        "correspondence_profile_hash": np.asarray(
            descriptor.correspondence_profile_hash.hex(), dtype=np.str_
        ),
        "run_root": np.asarray(run_root, dtype=np.str_),
        "reward_component_ids": reward_ids,
        "episode_ordinal": np.asarray([row.episode_ordinal for row in records], dtype=np.uint64),
        "vector_slot": np.asarray([row.vector_slot for row in records], dtype=np.uint32),
        "motor_tick": np.asarray([row.motor_tick for row in records], dtype=np.uint64),
        "command_raw": _stack(records, "command_raw"),
        "next_command_raw": _stack(records, "next_command_raw"),
        "applied_action_raw": _stack(records, "applied_action_raw"),
        "observation_raw": _stack(records, "observation_raw"),
        "reward_components_q16": _stack(records, "reward_components_raw"),
        "reward_total_q16": np.asarray(
            [row.reward_total_q16 for row in records], dtype=np.int64
        ),
        "terminated": np.asarray([row.terminated for row in records], dtype=np.bool_),
        "truncated": np.asarray([row.truncated for row in records], dtype=np.bool_),
        "terminal_reason_id": np.asarray(
            [row.terminal_reason_id or "" for row in records], dtype=np.str_
        ),
        "physics_root": np.asarray([row.physics_root for row in records], dtype="S32"),
        "motor_root": np.asarray([row.motor_root for row in records], dtype="S32"),
        "step_root": np.asarray([row.step_root for row in records], dtype="S32"),
        "root_position_micrometres": _stack(records, "root_position_micrometres"),
        "root_quaternion_q1_30_xyzw": _stack(records, "root_quaternion_q1_30_xyzw"),
        "root_linear_velocity_micrometres_per_second": _stack(
            records, "root_linear_velocity_micrometres_per_second"
        ),
        "root_angular_velocity_microradians_per_second": _stack(
            records, "root_angular_velocity_microradians_per_second"
        ),
        "joint_position_microradians": _stack(records, "joint_position_microradians"),
        "joint_velocity_microradians_per_second": _stack(
            records, "joint_velocity_microradians_per_second"
        ),
        "contact_flags": _stack(records, "contact_flags").astype(np.bool_),
    }
    try:
        with temporary_path.open("wb") as output:
            np.savez_compressed(output, **payload)
        os.replace(temporary_path, output_path)
    finally:
        temporary_path.unlink(missing_ok=True)
    return output_path


def _stack(records: list[StepResult], field: str) -> np.ndarray:
    return np.stack([getattr(row, field) for row in records])
