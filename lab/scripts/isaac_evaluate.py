#!/usr/bin/env python3
"""Evaluate one closed RSL-RL checkpoint without exploration noise."""

from __future__ import annotations

import argparse
import copy
import json
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
    equal_episode_quota,
    load_active_training_generation,
    parse_gpu_memory_csv,
    require_external_path,
    require_generation_output_path,
    sha256_file,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generation-index", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--seed", type=int)
    parser.add_argument("--episodes", type=int)
    parser.add_argument("--num-envs", type=int)
    parser.add_argument("--max-steps", type=int)
    parser.add_argument("--episode-ordinal-start", type=int)
    parser.add_argument("--evaluation-device")
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    generation_index = require_external_path(
        args.generation_index, REPOSITORY_ROOT, label="active training generation"
    )
    generation = load_active_training_generation(generation_index)
    profile = IsaacTrainingProfile.load(args.profile.resolve())
    evaluation = profile.evaluation
    seed = evaluation["seeds"][0] if args.seed is None else args.seed
    episodes = evaluation["episodes"] if args.episodes is None else args.episodes
    num_envs = evaluation["num_envs"] if args.num_envs is None else args.num_envs
    max_steps = evaluation["max_steps"] if args.max_steps is None else args.max_steps
    episode_ordinal_start = (
        evaluation.get("episode_ordinal_start", 0)
        if args.episode_ordinal_start is None
        else args.episode_ordinal_start
    )
    if min(episodes, num_envs, max_steps) <= 0:
        raise ValueError("evaluation counts must be positive")
    if not 0 <= episode_ordinal_start < 2**63:
        raise ValueError("evaluation episode ordinal start is outside u63")
    episodes_per_slot = equal_episode_quota(episodes, num_envs)
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
    generation.manifest.require_input(profile, sha256_file(descriptor), sha256_file(usd))
    checkpoint = require_external_path(
        args.checkpoint, REPOSITORY_ROOT, label="checkpoint"
    )
    output_root = require_generation_output_path(
        require_external_path(
            args.output_root,
            REPOSITORY_ROOT,
            label="evaluation root",
            must_exist=False,
        ),
        generation,
        label="evaluation root",
    )
    parent = validate_closed_checkpoint(checkpoint, generation.manifest.generation_id)
    validate_checkpoint_artifacts(parent, profile, descriptor, usd)
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
        "training_generation_id": generation.manifest.generation_id,
        "training_generation_manifest_hash": generation.manifest.manifest_hash,
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "repository": repository_state(),
        "profile_id": profile.profile_id,
        "profile_hash": profile.profile_hash,
        "environment_profile_id": profile.environment_profile_id,
        "seed": seed,
        "run_root_hex": config.run_root_hex,
        "num_envs": num_envs,
        "requested_episodes": episodes,
        "sampling": {
            "strategy": "equal-per-slot",
            "episodes_per_slot": episodes_per_slot,
        },
        "maximum_episode_steps": max_steps,
        "episode_ordinal_start": episode_ordinal_start,
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
            rotate_world_to_root_local_q1_30_tensor,
        )
        from next_lab.isaac_rl import GuardedOnPolicyRunner, build_agent_cfg

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = num_envs
        env_cfg.seed = seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.episode_ordinal_start = episode_ordinal_start
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
        current_commanded_forward_distance = torch.zeros(
            num_envs, device=config.device
        )
        current_achieved_forward_distance = torch.zeros(
            num_envs, device=config.device
        )
        current_forward_velocity_error = torch.zeros(
            num_envs, device=config.device
        )
        episode_returns: list[float] = []
        episode_lengths: list[int] = []
        episode_component_means: list[list[float]] = []
        episode_commanded_forward_distances: list[float] = []
        episode_achieved_forward_distances: list[float] = []
        episode_forward_velocity_mae: list[float] = []
        slot_returns: list[list[float]] = [[] for _ in range(num_envs)]
        slot_lengths: list[list[int]] = [[] for _ in range(num_envs)]
        completed_per_slot = torch.zeros(
            num_envs, dtype=torch.int64, device=config.device
        )
        terminated_count = 0
        truncated_count = 0
        terminal_reason_counts = {
            "joint_safety": 0,
            "hard_impact": 0,
            "self_collision": 0,
            "forbidden_contact": 0,
            "fall": 0,
        }
        self_collision_events: list[dict[str, Any]] = []
        step_budget = max_steps * episodes_per_slot

        with torch.inference_mode():
            for _ in range(step_budget):
                applied_command = torch.round(
                    observations["policy"][:, 79:82] * 1_000_000.0
                ).to(torch.int64)
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
                if hasattr(environment, "last_step_root_linear_velocity"):
                    local_linear_velocity = rotate_world_to_root_local_q1_30_tensor(
                        environment.last_step_root_quaternion,
                        environment.last_step_root_linear_velocity,
                    )
                    current_commanded_forward_distance.add_(
                        applied_command[:, 1].to(torch.float32) / 60_000_000.0
                    )
                    current_achieved_forward_distance.add_(
                        local_linear_velocity[:, 2].to(torch.float32) / 60_000_000.0
                    )
                    current_forward_velocity_error.add_(
                        torch.abs(local_linear_velocity[:, 2] - applied_command[:, 1])
                        .to(torch.float32)
                        / 1_000_000.0
                    )

                done_ids = dones.nonzero(as_tuple=False).squeeze(-1)
                if len(done_ids) == 0:
                    continue
                for env_id in done_ids.tolist():
                    if completed_per_slot[env_id] >= episodes_per_slot:
                        continue
                    length = int(current_lengths[env_id].item())
                    episode_return = float(current_returns[env_id].item())
                    episode_returns.append(episode_return)
                    episode_lengths.append(length)
                    episode_component_means.append(
                        (current_components[env_id] / length).cpu().tolist()
                    )
                    episode_commanded_forward_distances.append(
                        float(current_commanded_forward_distance[env_id].item())
                    )
                    episode_achieved_forward_distances.append(
                        float(current_achieved_forward_distance[env_id].item())
                    )
                    episode_forward_velocity_mae.append(
                        float(current_forward_velocity_error[env_id].item()) / length
                    )
                    slot_returns[env_id].append(episode_return)
                    slot_lengths[env_id].append(length)
                    completed_per_slot[env_id] += 1
                    terminated_count += int(environment.reset_terminated[env_id].item())
                    truncated_count += int(environment.reset_time_outs[env_id].item())
                    if hasattr(environment, "last_step_failure_self_collision"):
                        for reason in terminal_reason_counts:
                            field = f"last_step_failure_{reason}"
                            terminal_reason_counts[reason] += int(
                                getattr(environment, field)[env_id].item()
                            )
                        pair_indices = (
                            environment.last_step_self_collision_pair_mask[env_id]
                            .nonzero(as_tuple=False)
                            .squeeze(-1)
                            .detach()
                            .cpu()
                            .tolist()
                        )
                        if pair_indices:
                            self_collision_events.append(
                                {
                                    "episode": len(episode_lengths) - 1,
                                    "vector_slot": env_id,
                                    "motor_tick": length,
                                    "pairs": [
                                        {
                                            "pair_id": environment.contact_pair_ids[index],
                                            "impulse_micronewton_seconds": environment.last_step_contact_pair_impulse[
                                                env_id, index
                                            ]
                                            .detach()
                                            .cpu()
                                            .tolist(),
                                            "separation_micrometres": int(
                                                environment.last_step_contact_pair_separation[
                                                    env_id, index
                                                ].item()
                                            ),
                                        }
                                        for index in pair_indices
                                    ],
                                }
                            )
                current_returns[done_ids] = 0.0
                current_lengths[done_ids] = 0
                current_components[done_ids] = 0.0
                current_commanded_forward_distance[done_ids] = 0.0
                current_achieved_forward_distance[done_ids] = 0.0
                current_forward_velocity_error[done_ids] = 0.0
                if torch.all(completed_per_slot >= episodes_per_slot):
                    break

        completed_counts = completed_per_slot.cpu().tolist()
        if completed_counts != [episodes_per_slot] * num_envs:
            raise RuntimeError(
                "evaluation step budget exhausted before equal per-slot quota: "
                f"minimum={min(completed_counts)}, maximum={max(completed_counts)}"
            )
        component_metrics = {}
        for index, component in enumerate(environment.profile["reward_components"]):
            values = [episode[index] for episode in episode_component_means]
            component_metrics[component["component_id"]] = summary(values)
        movement_metrics = {}
        if episode_commanded_forward_distances:
            movement_metrics = {
                "commanded_forward_distance_metres": summary(
                    episode_commanded_forward_distances
                ),
                "achieved_root_local_forward_distance_metres": summary(
                    episode_achieved_forward_distances
                ),
                "root_local_forward_velocity_mae_metres_per_second": summary(
                    episode_forward_velocity_mae
                ),
            }
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
                    "reason_counts": terminal_reason_counts,
                    "self_collision_events": self_collision_events,
                },
                "slot_balance": {
                    "episode_counts": completed_counts,
                    "episode_lengths": slot_lengths,
                    "mean_returns": summary(
                        [statistics.mean(values) for values in slot_returns]
                    ),
                    "mean_lengths": summary(
                        [statistics.mean(values) for values in slot_lengths]
                    ),
                },
                "reward_component_mean_per_step": component_metrics,
                **movement_metrics,
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


def repository_state() -> dict[str, Any]:
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    status = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return {"root": str(REPOSITORY_ROOT), "commit": commit, "dirty": bool(status)}


if __name__ == "__main__":
    main()
