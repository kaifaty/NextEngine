#!/usr/bin/env python3
"""Record matched standing-policy trajectories in canonical CPU PhysX."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import torch

from cpu_evaluate_biomechanics_standing import (
    REPOSITORY_ROOT,
    load_actor,
    observation_scales,
    policy_observation,
)
from next_lab.isaac_training import (
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    load_active_training_generation,
    require_external_path,
    sha256_file,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
)
from next_lab.motor_lab_client import (
    BIOMECHANICS_STANDING_PROFILE_ID,
    MotorLabClient,
    normalized_action_to_raw,
)
from next_lab.motor_mirror import select_biomechanics_standing_profile
from next_lab.policy_correspondence import trajectory_metadata, write_policy_trajectory


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generation-index", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--action-tape", type=Path)
    parser.add_argument("--headless", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--episodes", type=int, default=256)
    parser.add_argument("--motor-steps", type=int, default=600)
    parser.add_argument(
        "--action-source", choices=("policy", "zero"), default="policy"
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if not 1 <= args.episodes <= 256 or args.motor_steps <= 0:
        raise ValueError("episodes must be in [1,256] and motor-steps must be positive")
    generation = load_active_training_generation(
        require_external_path(
            args.generation_index,
            REPOSITORY_ROOT,
            label="active training generation",
        )
    )
    training_profile = IsaacTrainingProfile.load(args.profile.resolve())
    run_root = ResolvedTrainingConfig.from_profile(training_profile).run_root_hex
    descriptor_path = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd_path = require_external_path(
        args.usd, REPOSITORY_ROOT, label="derived humanoid USD"
    )
    checkpoint_path = require_external_path(
        args.checkpoint, REPOSITORY_ROOT, label="checkpoint"
    )
    output_path = require_external_path(
        args.output, REPOSITORY_ROOT, label="CPU trajectory", must_exist=False
    )
    generation.manifest.require_input(
        training_profile, sha256_file(descriptor_path), sha256_file(usd_path)
    )
    parent = validate_closed_checkpoint(
        checkpoint_path, generation.manifest.generation_id
    )
    validate_checkpoint_artifacts(
        parent, training_profile, descriptor_path, usd_path
    )
    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    profile = select_biomechanics_standing_profile(
        descriptor, BIOMECHANICS_STANDING_PROFILE_ID
    )
    metadata = trajectory_metadata(
        profile,
        checkpoint_sha256=sha256_file(checkpoint_path),
        training_generation_manifest_hash=generation.manifest.manifest_hash,
        run_root=run_root,
    )
    actor, mean, std = load_actor(checkpoint_path)
    scales = observation_scales(descriptor)

    shape = (args.episodes, args.motor_steps)
    joints = np.empty((*shape, 23), dtype=np.float32)
    positions = np.empty((*shape, 3), dtype=np.float32)
    velocities = np.empty((*shape, 3), dtype=np.float32)
    contacts = np.empty((*shape, 2), dtype=np.bool_)
    rewards = np.empty(shape, dtype=np.int64)
    commands = np.empty((*shape, 3), dtype=np.int64)
    action_tape = np.empty((*shape, 23), dtype=np.int64)
    replay_action_tape = None
    if args.action_tape is not None:
        action_tape_path = require_external_path(
            args.action_tape, REPOSITORY_ROOT, label="canonical GPU action tape"
        )
        with np.load(action_tape_path, allow_pickle=False) as tape:
            replay_action_tape = np.asarray(tape["action_raw"], dtype=np.int64)
            for key, expected in metadata.items():
                if not np.array_equal(tape[key], expected):
                    raise ValueError(f"GPU action tape identity mismatch: {key}")
        if replay_action_tape.shape != (*shape, 23):
            raise ValueError(
                f"GPU action tape shape mismatch: expected {(*shape, 23)}, "
                f"got {replay_action_tape.shape}"
            )
    done_ticks = np.full(args.episodes, args.motor_steps, dtype=np.int64)

    with MotorLabClient(
        args.headless.resolve(),
        BIOMECHANICS_STANDING_PROFILE_ID,
        args.episodes,
        run_root,
    ) as client:
        if args.motor_steps > client.descriptor.maximum_episode_steps:
            raise ValueError("motor-steps exceeds the environment episode bound")
        resets = client.reset(list(range(args.episodes)))
        ordinals = np.empty(args.episodes, dtype=np.uint64)
        observations = np.empty((args.episodes, 84), dtype=np.float32)
        for reset in resets:
            slot = reset.vector_slot
            ordinals[slot] = reset.episode_ordinal
            observations[slot] = policy_observation(
                reset.observation_raw, scales
            ).numpy()[0]

        with torch.inference_mode():
            for tick in range(args.motor_steps):
                normalized = (
                    torch.from_numpy(observations) - mean
                ) / (std + 1.0e-2)
                if replay_action_tape is not None:
                    action_raw = replay_action_tape[:, tick]
                else:
                    actions = (
                        actor(normalized).numpy()
                        if args.action_source == "policy"
                        else np.zeros((args.episodes, 23), dtype=np.float32)
                    )
                    action_raw = normalized_action_to_raw(actions, q1_30=True)
                steps = client.step(ordinals, action_raw)
                for step in steps:
                    slot = step.vector_slot
                    if step.terminated or step.truncated:
                        raise RuntimeError(
                            f"CPU slot {slot} ended at tick {step.motor_tick}: "
                            f"{step.terminal_reason_id}"
                        )
                    joints[slot, tick] = (
                        step.joint_position_microradians.astype(np.float64) / 1_000_000.0
                    )
                    positions[slot, tick] = (
                        step.root_position_micrometres.astype(np.float64) / 1_000_000.0
                    )
                    velocities[slot, tick] = (
                        step.root_linear_velocity_micrometres_per_second.astype(np.float64)
                        / 1_000_000.0
                    )
                    contacts[slot, tick] = step.contact_flags
                    rewards[slot, tick] = step.reward_total_q16
                    commands[slot, tick] = step.command_raw
                    action_tape[slot, tick] = action_raw[slot]
                    observations[slot] = policy_observation(
                        step.observation_raw, scales
                    ).numpy()[0]

    path = write_policy_trajectory(
        output_path,
        metadata=metadata,
        joint_position_rad=joints,
        root_position_m=positions,
        root_velocity_mps=velocities,
        contact_occupancy=contacts,
        done_tick=done_ticks,
        reward_total_q16=rewards,
        command_raw=commands,
        action_raw=action_tape,
    )
    print(path)


if __name__ == "__main__":
    main()
