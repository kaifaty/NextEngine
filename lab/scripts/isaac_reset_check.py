#!/usr/bin/env python3
"""Force a fall and verify that the NextEngine Isaac environment fully resets."""

from __future__ import annotations

import argparse
import json
import os
import traceback
from pathlib import Path
from typing import Any

from isaaclab.app import AppLauncher

from next_lab.isaac_training import (
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    require_external_path,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--num-envs", type=int, default=16)
    parser.add_argument("--seed", type=int, default=1001)
    parser.add_argument("--survival-steps", type=int, default=10)
    parser.add_argument("--max-root-linear-speed", type=float, default=1.0)
    parser.add_argument("--max-root-angular-speed", type=float, default=2.0)
    parser.add_argument("--max-joint-speed", type=float, default=5.0)
    parser.add_argument("--max-root-height-overshoot", type=float, default=0.02)
    parser.add_argument("--check-device")
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.survival_steps <= 0:
        raise ValueError("survival-steps must be positive")
    for name in (
        "max_root_linear_speed",
        "max_root_angular_speed",
        "max_joint_speed",
        "max_root_height_overshoot",
    ):
        if getattr(args, name) <= 0.0:
            raise ValueError(f"{name.replace('_', '-')} must be positive")
    profile = IsaacTrainingProfile.load(args.profile.resolve())
    config = ResolvedTrainingConfig.from_profile(
        profile,
        num_envs=args.num_envs,
        steps_per_env=1,
        iterations=1,
        seed=args.seed,
        device=args.check_device,
    )
    descriptor = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")

    simulation_app = None
    wrapped = None
    try:
        os.environ["NEXTENGINE_HUMANOID_USD"] = str(usd)
        args.device = config.device
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch
        from isaaclab_rl.rsl_rl import RslRlVecEnvWrapper

        from next_lab.isaac_env import (
            NextEngineHumanoidDirectEnv,
            NextEngineHumanoidDirectEnvCfg,
        )

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = config.num_envs
        env_cfg.seed = config.seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.episode_ordinal_start = profile.evaluation.get("episode_ordinal_start", 0)
        env_cfg.sim.device = config.device
        environment = NextEngineHumanoidDirectEnv(
            env_cfg,
            descriptor_path=str(descriptor),
        )
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)

        expected_root = environment.robot.data.default_root_state.clone()
        expected_root[:, :3] += environment.scene.env_origins
        expected_joint_position = environment.robot.data.default_joint_pos.clone()
        expected_joint_velocity = environment.robot.data.default_joint_vel.clone()
        initial_errors = state_errors(
            environment,
            expected_root,
            expected_joint_position,
            expected_joint_velocity,
        )
        require_below_tolerance("initial reset", initial_errors)

        forced_root = expected_root.clone()
        forced_root[:, 2] = environment.scene.env_origins[:, 2] + 0.2
        forced_root[:, 7:] = 0.5
        lower = environment.robot.data.soft_joint_pos_limits[:, :, 0]
        upper = environment.robot.data.soft_joint_pos_limits[:, :, 1]
        forced_joint_position = torch.clamp(
            expected_joint_position + 0.05,
            min=lower,
            max=upper,
        )
        forced_joint_velocity = torch.full_like(expected_joint_velocity, 0.25)
        environment.robot.write_root_state_to_sim(forced_root)
        environment.robot.write_joint_state_to_sim(
            forced_joint_position, forced_joint_velocity
        )

        zero_actions = torch.zeros(
            (config.num_envs, 23), dtype=torch.float32, device=config.device
        )
        _, _, dones, _ = wrapped.step(zero_actions)
        if not torch.all(dones > 0) or not torch.all(environment.reset_terminated):
            raise RuntimeError("forced fall did not terminate every environment slot")
        if torch.any(environment.episode_length_buf != 0):
            raise RuntimeError("episode length was not cleared by automatic reset")

        automatic_reset_errors = state_errors(
            environment,
            expected_root,
            expected_joint_position,
            expected_joint_velocity,
        )
        require_below_tolerance("automatic reset", automatic_reset_errors)
        motion = {
            "maximum_joint_speed_rps": 0.0,
            "maximum_root_angular_speed_rps": 0.0,
            "maximum_root_height_overshoot_m": 0.0,
            "maximum_root_linear_speed_mps": 0.0,
        }
        for _ in range(args.survival_steps):
            _, _, dones, _ = wrapped.step(zero_actions)
            if torch.any(dones > 0):
                raise RuntimeError("zero-action state terminated immediately after reset")
            motion["maximum_joint_speed_rps"] = max(
                motion["maximum_joint_speed_rps"],
                float(torch.max(torch.abs(environment.robot.data.joint_vel)).item()),
            )
            motion["maximum_root_angular_speed_rps"] = max(
                motion["maximum_root_angular_speed_rps"],
                float(
                    torch.max(
                        torch.linalg.vector_norm(
                            environment.robot.data.root_ang_vel_w, dim=-1
                        )
                    ).item()
                ),
            )
            motion["maximum_root_linear_speed_mps"] = max(
                motion["maximum_root_linear_speed_mps"],
                float(
                    torch.max(
                        torch.linalg.vector_norm(
                            environment.robot.data.root_lin_vel_w, dim=-1
                        )
                    ).item()
                ),
            )
            motion["maximum_root_height_overshoot_m"] = max(
                motion["maximum_root_height_overshoot_m"],
                float(
                    torch.max(
                        environment.robot.data.root_pos_w[:, 2] - expected_root[:, 2]
                    ).item()
                ),
            )
        require_motion_envelope(args, motion)

        print(
            json.dumps(
                {
                    "status": "passed",
                    "num_envs": config.num_envs,
                    "seed": config.seed,
                    "reset_root_height_m": float(expected_root[0, 2].item()),
                    "authored_ground_clearance_m": environment.authored_ground_clearance_m,
                    "descriptor_initial_deviation": (
                        environment.reset_template_initial_deviation
                    ),
                    "initial_errors": initial_errors,
                    "automatic_reset_errors": automatic_reset_errors,
                    "post_reset_zero_action_steps": args.survival_steps,
                    "post_reset_motion": motion,
                    "episode_ordinals": environment._episode_ordinals.cpu().tolist(),
                },
                indent=2,
                sort_keys=True,
            ),
            flush=True,
        )
    except BaseException:
        traceback.print_exc()
        raise
    finally:
        if wrapped is not None:
            wrapped.close()
        if simulation_app is not None:
            simulation_app.close()


def state_errors(
    environment: Any,
    expected_root: Any,
    expected_joint_position: Any,
    expected_joint_velocity: Any,
) -> dict[str, float]:
    import torch

    values = {
        "root_pose": torch.max(
            torch.abs(environment.robot.data.root_state_w[:, :7] - expected_root[:, :7])
        ),
        "root_velocity": torch.max(
            torch.abs(environment.robot.data.root_state_w[:, 7:] - expected_root[:, 7:])
        ),
        "joint_position": torch.max(
            torch.abs(environment.robot.data.joint_pos - expected_joint_position)
        ),
        "joint_velocity": torch.max(
            torch.abs(environment.robot.data.joint_vel - expected_joint_velocity)
        ),
    }
    return {name: float(value.item()) for name, value in values.items()}


def require_below_tolerance(label: str, errors: dict[str, float]) -> None:
    tolerance = 1.0e-6
    failed = {name: value for name, value in errors.items() if value > tolerance}
    if failed:
        raise RuntimeError(f"{label} state mismatch: {failed}")


def require_motion_envelope(args: argparse.Namespace, motion: dict[str, float]) -> None:
    limits = {
        "maximum_joint_speed_rps": args.max_joint_speed,
        "maximum_root_angular_speed_rps": args.max_root_angular_speed,
        "maximum_root_height_overshoot_m": args.max_root_height_overshoot,
        "maximum_root_linear_speed_mps": args.max_root_linear_speed,
    }
    failed = {
        name: {"actual": motion[name], "maximum": maximum}
        for name, maximum in limits.items()
        if motion[name] > maximum
    }
    if failed:
        raise RuntimeError(f"post-reset motion envelope exceeded: {failed}")


if __name__ == "__main__":
    main()
