#!/usr/bin/env python3
"""Stress the NextEngine Isaac mirror with zero, random, and policy actions."""

from __future__ import annotations

import argparse
import copy
import json
import os
from pathlib import Path
import sys
import traceback
from typing import Any

from isaaclab.app import AppLauncher

from next_lab.isaac_training import (
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    latest_closed_checkpoint,
    require_external_path,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--runs-root", type=Path)
    parser.add_argument("--checkpoint", type=Path)
    parser.add_argument("--num-envs", type=int, default=32)
    parser.add_argument("--steps", type=int, default=1_200)
    parser.add_argument("--seed", type=int, default=1002)
    parser.add_argument("--random-action-scale", type=float, default=1.0)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.num_envs <= 0 or args.steps <= 0:
        raise ValueError("num-envs and steps must be positive")
    if not 0.0 < args.random_action_scale <= 1.0:
        raise ValueError("random-action-scale must be in (0, 1]")

    profile = IsaacTrainingProfile.load(args.profile.resolve())
    config = ResolvedTrainingConfig.from_profile(
        profile,
        num_envs=args.num_envs,
        steps_per_env=1,
        iterations=1,
        save_interval=1,
        seed=args.seed,
    )
    descriptor = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")
    checkpoint = resolve_checkpoint(args)
    parent = validate_closed_checkpoint(checkpoint)
    validate_checkpoint_artifacts(parent, profile, descriptor, usd)

    simulation_app = None
    wrapped = None
    failed = False
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
        from next_lab.isaac_rl import GuardedOnPolicyRunner, build_agent_cfg

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = config.num_envs
        env_cfg.seed = config.seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.sim.device = config.device
        environment = NextEngineHumanoidDirectEnv(env_cfg, descriptor_path=str(descriptor))
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)

        runner = GuardedOnPolicyRunner(
            wrapped,
            copy.deepcopy(build_agent_cfg(config).to_dict()),
            log_dir=None,
            device=config.device,
        )
        runner.load(str(checkpoint), load_optimizer=False, map_location=config.device)
        policy = runner.get_inference_policy(device=config.device)
        generator = torch.Generator(device=config.device)
        generator.manual_seed(config.seed ^ 0x5A17)

        reports = []
        for source in ("zero", "random", "policy"):
            print(
                f"[stability] action_source={source} steps={args.steps}",
                flush=True,
            )
            observations, _ = wrapped.reset()
            report = {
                "action_source": source,
                "maximum_abs_joint_velocity_rps": 0.0,
                "maximum_abs_root_velocity": 0.0,
                "resets": 0,
                "steps": args.steps,
            }
            with torch.no_grad():
                for _ in range(args.steps):
                    if source == "zero":
                        actions = torch.zeros(
                            (config.num_envs, 23), device=config.device
                        )
                    elif source == "random":
                        actions = (
                            torch.rand(
                                (config.num_envs, 23),
                                device=config.device,
                                generator=generator,
                            )
                            * 2.0
                            - 1.0
                        ) * args.random_action_scale
                    else:
                        actions = policy(observations)
                    require_finite("actions", actions)
                    observations, rewards, dones, _ = wrapped.step(actions)
                    require_finite("observations", observations)
                    require_finite("rewards", rewards)
                    report["resets"] += int(torch.sum(dones).item())
                    report["maximum_abs_joint_velocity_rps"] = max(
                        report["maximum_abs_joint_velocity_rps"],
                        float(torch.max(torch.abs(environment.robot.data.joint_vel)).item()),
                    )
                    root_velocity = environment.robot.data.root_state_w[:, 7:]
                    report["maximum_abs_root_velocity"] = max(
                        report["maximum_abs_root_velocity"],
                        float(torch.max(torch.abs(root_velocity)).item()),
                    )
            reports.append(report)

        actuator = environment.robot.actuators["engine_effort"]
        result = {
            "checkpoint": str(checkpoint),
            "configured_effort_limits_nm": sorted(
                set(actuator.effort_limit_sim.detach().cpu().flatten().tolist())
            ),
            "configured_velocity_limits_rps": sorted(
                set(actuator.velocity_limit_sim.detach().cpu().flatten().tolist())
            ),
            "num_envs": config.num_envs,
            "reports": reports,
            "seed": config.seed,
            "status": "passed",
        }
        print(json.dumps(result, indent=2, sort_keys=True), flush=True)
    except BaseException:
        traceback.print_exc()
        failed = True
    finally:
        if failed:
            sys.stdout.flush()
            sys.stderr.flush()
            os._exit(1)
        if wrapped is not None:
            wrapped.close()
        if simulation_app is not None:
            simulation_app.close()


def resolve_checkpoint(args: argparse.Namespace) -> Path:
    if args.checkpoint is not None:
        return require_external_path(
            args.checkpoint, REPOSITORY_ROOT, label="stability checkpoint"
        )
    if args.runs_root is None:
        raise ValueError("either --checkpoint or --runs-root is required")
    runs_root = require_external_path(
        args.runs_root, REPOSITORY_ROOT, label="training runs root"
    )
    return latest_closed_checkpoint(runs_root)


def require_finite(name: str, value: Any) -> None:
    import torch

    if torch.is_tensor(value):
        if not torch.isfinite(value).all():
            raise RuntimeError(f"non-finite stability {name}")
        return
    if hasattr(value, "items"):
        for key, child in value.items():
            require_finite(f"{name}.{key}", child)
        return
    raise TypeError(f"unsupported stability value for {name}: {type(value)!r}")


if __name__ == "__main__":
    main()
