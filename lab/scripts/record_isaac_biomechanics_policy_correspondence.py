#!/usr/bin/env python3
"""Record matched standing-policy trajectories in the Isaac GPU mirror."""

from __future__ import annotations

import argparse
import os
from pathlib import Path

import numpy as np
from isaaclab.app import AppLauncher

from next_lab.isaac_training import (
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    load_active_training_generation,
    require_external_path,
    sha256_file,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
)
from next_lab.policy_correspondence import trajectory_metadata, write_policy_trajectory


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generation-index", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--action-tape", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--episodes", type=int, default=256)
    parser.add_argument("--num-envs", type=int, default=128)
    parser.add_argument("--motor-steps", type=int, default=600)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if (
        not 1 <= args.num_envs <= args.episodes <= 256
        or args.episodes % args.num_envs != 0
        or args.motor_steps <= 0
    ):
        raise ValueError(
            "num-envs must divide episodes in [1,256] and motor-steps must be positive"
        )
    generation = load_active_training_generation(
        require_external_path(
            args.generation_index,
            REPOSITORY_ROOT,
            label="active training generation",
        )
    )
    profile = IsaacTrainingProfile.load(args.profile.resolve())
    descriptor_path = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd_path = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")
    checkpoint_path = require_external_path(
        args.checkpoint, REPOSITORY_ROOT, label="checkpoint"
    )
    output_path = require_external_path(
        args.output, REPOSITORY_ROOT, label="GPU trajectory", must_exist=False
    )
    generation.manifest.require_input(
        profile, sha256_file(descriptor_path), sha256_file(usd_path)
    )
    parent = validate_closed_checkpoint(
        checkpoint_path, generation.manifest.generation_id
    )
    validate_checkpoint_artifacts(parent, profile, descriptor_path, usd_path)
    action_tape_path = require_external_path(
        args.action_tape, REPOSITORY_ROOT, label="canonical CPU action tape"
    )
    config = ResolvedTrainingConfig.from_profile(
        profile,
        num_envs=args.num_envs,
        steps_per_env=1,
        iterations=1,
        seed=args.seed,
    )

    os.environ["NEXTENGINE_HUMANOID_USD"] = str(usd_path)
    args.device = config.device
    app_launcher = AppLauncher(args)
    simulation_app = app_launcher.app
    wrapped = None
    try:
        import torch
        from isaaclab_rl.rsl_rl import RslRlVecEnvWrapper

        from next_lab.isaac_env import (
            NextEngineHumanoidDirectEnv,
            NextEngineHumanoidDirectEnvCfg,
            engine_vector_from_isaac_tensor,
        )

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = args.num_envs
        env_cfg.seed = args.seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.episode_ordinal_start = 0
        env_cfg.sim.device = config.device
        environment = NextEngineHumanoidDirectEnv(
            env_cfg, descriptor_path=str(descriptor_path)
        )
        if args.motor_steps > environment.max_episode_length:
            raise ValueError("motor-steps exceeds the environment episode bound")
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)

        shape = (args.episodes, args.motor_steps)
        joints = np.empty((*shape, 23), dtype=np.float32)
        positions = np.empty((*shape, 3), dtype=np.float32)
        velocities = np.empty((*shape, 3), dtype=np.float32)
        contacts = np.empty((*shape, 2), dtype=np.bool_)
        rewards = np.empty(shape, dtype=np.int64)
        commands = np.empty((*shape, 3), dtype=np.int64)
        done_ticks = np.full(args.episodes, args.motor_steps, dtype=np.int64)

        expected_metadata = trajectory_metadata(
            environment.profile,
            checkpoint_sha256=sha256_file(checkpoint_path),
            training_generation_manifest_hash=generation.manifest.manifest_hash,
            run_root=config.run_root_hex,
        )
        with np.load(action_tape_path, allow_pickle=False) as tape:
            action_tape = np.asarray(tape["action_raw"], dtype=np.int64)
            for key, expected in expected_metadata.items():
                if not np.array_equal(tape[key], expected):
                    raise ValueError(f"CPU action tape identity mismatch: {key}")
        if action_tape.shape != (*shape, 23):
            raise ValueError(
                f"CPU action tape shape mismatch: expected {(*shape, 23)}, "
                f"got {action_tape.shape}"
            )

        with torch.inference_mode():
            for batch_start in range(0, args.episodes, args.num_envs):
                batch = slice(batch_start, batch_start + args.num_envs)
                observations, _ = wrapped.reset()
                for tick in range(args.motor_steps):
                    expected_action = torch.from_numpy(action_tape[batch, tick]).to(
                        config.device
                    )
                    actions = expected_action.to(torch.float32) / float(1 << 30)
                    observations, reward, dones, _ = wrapped.step(actions)
                    if not torch.equal(environment._action, expected_action):
                        raise RuntimeError(
                            "GPU did not consume the canonical action tape exactly"
                        )
                    if torch.any(dones):
                        ended = (
                            dones.nonzero(as_tuple=False).squeeze(-1).cpu().tolist()
                        )
                        raise RuntimeError(
                            f"GPU slots ended before {args.motor_steps} ticks: "
                            f"{ended[:8]}"
                        )
                    joints[batch, tick] = (
                        environment.robot.data.joint_pos[
                            :, environment._canonical_joint_ids
                        ]
                        .detach()
                        .cpu()
                        .numpy()
                    )
                    root_position = engine_vector_from_isaac_tensor(
                        environment.robot.data.root_pos_w - environment.scene.env_origins
                    )
                    positions[batch, tick] = root_position.detach().cpu().numpy()
                    _, _, linear_raw, _, contact = environment._canonical_facts()
                    velocities[batch, tick] = (
                        linear_raw.detach().cpu().numpy().astype(np.float64)
                        / 1_000_000.0
                    )
                    contacts[batch, tick] = contact.detach().cpu().numpy()
                    rewards[batch, tick] = np.rint(
                        reward.detach().cpu().numpy().astype(np.float64) * 65_536.0
                    ).astype(np.int64)
                    commands[batch, tick] = (
                        environment._current_command().detach().cpu().numpy()
                    )

        path = write_policy_trajectory(
            output_path,
            metadata=expected_metadata,
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
    finally:
        if wrapped is not None:
            wrapped.close()
        simulation_app.close()


if __name__ == "__main__":
    main()
