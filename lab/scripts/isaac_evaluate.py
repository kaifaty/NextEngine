#!/usr/bin/env python3
"""Evaluate one closed RSL-RL checkpoint without exploration noise."""

from __future__ import annotations

import argparse
import copy
import json
import math
import os
import statistics
import subprocess
import traceback
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from isaaclab.app import AppLauncher

from next_lab.isaac_training import (
    EVALUATION_MANIFEST_SCHEMA,
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    atomic_write_json,
    parse_gpu_memory_csv,
    require_external_path,
    sha256_file,
    validate_closed_checkpoint,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--seed", type=int)
    parser.add_argument("--episodes", type=int)
    parser.add_argument("--num-envs", type=int)
    parser.add_argument("--max-steps", type=int)
    parser.add_argument("--evaluation-device")
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    profile = IsaacTrainingProfile.load(args.profile.resolve())
    evaluation = profile.evaluation
    seed = evaluation["seeds"][0] if args.seed is None else args.seed
    episodes = evaluation["episodes"] if args.episodes is None else args.episodes
    num_envs = evaluation["num_envs"] if args.num_envs is None else args.num_envs
    max_steps = evaluation["max_steps"] if args.max_steps is None else args.max_steps
    if min(episodes, num_envs, max_steps) <= 0:
        raise ValueError("evaluation counts must be positive")
    config = ResolvedTrainingConfig.from_profile(
        profile,
        num_envs=num_envs,
        steps_per_env=1,
        iterations=1,
        seed=seed,
        device=args.evaluation_device,
    )
    descriptor = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")
    checkpoint = require_external_path(
        args.checkpoint, REPOSITORY_ROOT, label="checkpoint"
    )
    output_root = require_external_path(
        args.output_root, REPOSITORY_ROOT, label="evaluation root", must_exist=False
    )
    parent = validate_closed_checkpoint(checkpoint)
    validate_parent_artifacts(parent, profile, descriptor, usd)
    gpu = query_gpu(config.device)
    if gpu["memory_free_mib"] < profile.min_free_gpu_memory_mib:
        raise RuntimeError(
            "insufficient free GPU memory: "
            f"{gpu['memory_free_mib']} MiB available, "
            f"{profile.min_free_gpu_memory_mib} MiB required"
        )

    evaluation_id = (
        datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
        + f"-seed{seed}-"
        + sha256_file(checkpoint)[:12]
    )
    evaluation_dir = output_root / evaluation_id
    evaluation_dir.mkdir(parents=True, exist_ok=False)
    manifest_path = evaluation_dir / "evaluation-manifest.json"
    manifest: dict[str, Any] = {
        "schema": EVALUATION_MANIFEST_SCHEMA,
        "status": "running",
        "evaluation_id": evaluation_id,
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "profile_id": profile.profile_id,
        "profile_hash": profile.profile_hash,
        "environment_profile_id": profile.environment_profile_id,
        "seed": seed,
        "run_root_hex": config.run_root_hex,
        "num_envs": num_envs,
        "requested_episodes": episodes,
        "maximum_episode_steps": max_steps,
        "checkpoint": {
            "path": str(checkpoint),
            "sha256": sha256_file(checkpoint),
            "parent_run_id": parent.get("run_id"),
        },
        "gpu_preflight": gpu,
    }
    atomic_write_json(manifest_path, manifest)

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
        from next_lab.isaac_rl import GuardedOnPolicyRunner, build_agent_cfg

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = num_envs
        env_cfg.seed = seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.sim.device = config.device
        environment = NextEngineHumanoidDirectEnv(
            env_cfg,
            descriptor_path=str(descriptor),
        )
        if max_steps != environment.max_episode_length:
            raise ValueError(
                "evaluation maximum_episode_steps must match the engine environment profile: "
                f"expected {environment.max_episode_length}, got {max_steps}"
            )
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)
        runner = GuardedOnPolicyRunner(
            wrapped,
            copy.deepcopy(build_agent_cfg(config).to_dict()),
            log_dir=None,
            device=config.device,
        )
        runner.load(str(checkpoint), load_optimizer=False, map_location=config.device)
        policy = runner.get_inference_policy(device=config.device)
        observations, _ = wrapped.reset()

        current_returns = torch.zeros(num_envs, device=config.device)
        current_lengths = torch.zeros(num_envs, dtype=torch.int64, device=config.device)
        component_count = len(environment.profile["reward_components"])
        current_components = torch.zeros(
            (num_envs, component_count), device=config.device
        )
        episode_returns: list[float] = []
        episode_lengths: list[int] = []
        episode_component_means: list[list[float]] = []
        terminated_count = 0
        truncated_count = 0
        step_budget = max_steps * (math.ceil(episodes / num_envs) + 1)

        with torch.inference_mode():
            for _ in range(step_budget):
                actions = policy(observations)
                require_finite("evaluation action", actions)
                observations, rewards, dones, _ = wrapped.step(actions)
                require_finite("evaluation observation", observations["policy"])
                require_finite("evaluation reward", rewards)
                component_values = (
                    environment.reward_components_q16.to(torch.float32) / 65_536.0
                )
                require_finite("evaluation reward components", component_values)
                current_returns.add_(rewards)
                current_lengths.add_(1)
                current_components.add_(component_values)

                done_ids = dones.nonzero(as_tuple=False).squeeze(-1)
                if len(done_ids) == 0:
                    continue
                for env_id in done_ids.tolist():
                    if len(episode_returns) >= episodes:
                        break
                    length = int(current_lengths[env_id].item())
                    episode_returns.append(float(current_returns[env_id].item()))
                    episode_lengths.append(length)
                    episode_component_means.append(
                        (current_components[env_id] / length).cpu().tolist()
                    )
                    terminated_count += int(environment.reset_terminated[env_id].item())
                    truncated_count += int(environment.reset_time_outs[env_id].item())
                current_returns[done_ids] = 0.0
                current_lengths[done_ids] = 0
                current_components[done_ids] = 0.0
                if len(episode_returns) >= episodes:
                    break

        if len(episode_returns) != episodes:
            raise RuntimeError(
                f"evaluation step budget exhausted after {len(episode_returns)} episodes"
            )
        component_metrics = {}
        for index, component in enumerate(environment.profile["reward_components"]):
            values = [episode[index] for episode in episode_component_means]
            component_metrics[component["component_id"]] = summary(values)
        manifest.update(
            {
                "status": "completed",
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "episodes": episodes,
                "returns": summary(episode_returns),
                "lengths": summary(episode_lengths),
                "termination": {
                    "terminated": terminated_count,
                    "truncated": truncated_count,
                },
                "reward_component_mean_per_step": component_metrics,
                "gpu_postflight": query_gpu(config.device),
            }
        )
        atomic_write_json(manifest_path, manifest)
        print(json.dumps(manifest, indent=2, sort_keys=True), flush=True)
    except BaseException as error:
        manifest.update(
            {
                "status": "failed",
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "error": {
                    "type": type(error).__name__,
                    "message": str(error)[:2000],
                },
            }
        )
        atomic_write_json(manifest_path, manifest)
        traceback.print_exc()
        raise
    finally:
        if wrapped is not None:
            wrapped.close()
        if simulation_app is not None:
            simulation_app.close()


def validate_parent_artifacts(
    parent: dict[str, Any],
    profile: IsaacTrainingProfile,
    descriptor: Path,
    usd: Path,
) -> None:
    training = parent.get("training_config")
    artifacts = parent.get("artifacts")
    if not isinstance(training, dict) or training.get("profile_hash") != profile.profile_hash:
        raise ValueError("checkpoint training profile does not match evaluation profile")
    if not isinstance(artifacts, dict):
        raise ValueError("checkpoint manifest has no artifact closure")
    expected = {
        "descriptor": sha256_file(descriptor),
        "usd": sha256_file(usd),
    }
    for name, digest in expected.items():
        record = artifacts.get(name)
        if not isinstance(record, dict) or record.get("sha256") != digest:
            raise ValueError(f"checkpoint {name} does not match evaluation artifact")


def require_finite(name: str, value: Any) -> None:
    import torch

    if not torch.isfinite(value).all():
        raise RuntimeError(f"non-finite {name}")


def summary(values: list[float] | list[int]) -> dict[str, float | int]:
    return {
        "count": len(values),
        "mean": statistics.mean(values),
        "standard_deviation": statistics.pstdev(values),
        "minimum": min(values),
        "maximum": max(values),
    }


def query_gpu(device: str) -> dict[str, int | str]:
    index = int(device.removeprefix("cuda:"))
    result = subprocess.run(
        [
            "nvidia-smi",
            "--query-gpu=index,name,memory.total,memory.free",
            "--format=csv,noheader,nounits",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    for line in result.stdout.splitlines():
        record = parse_gpu_memory_csv(line)
        if record["index"] == index:
            return record
    raise RuntimeError(f"CUDA device is not reported by nvidia-smi: {device}")


if __name__ == "__main__":
    main()
