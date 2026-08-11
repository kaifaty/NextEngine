from __future__ import annotations

import json
import math
import statistics
from pathlib import Path
from typing import Any

import torch
from isaaclab_rl.rsl_rl import (
    RslRlOnPolicyRunnerCfg,
    RslRlPpoActorCriticCfg,
    RslRlPpoAlgorithmCfg,
)
from rsl_rl.runners import OnPolicyRunner

from next_lab.isaac_training import ResolvedTrainingConfig


def build_agent_cfg(config: ResolvedTrainingConfig) -> RslRlOnPolicyRunnerCfg:
    policy = config.profile.policy
    algorithm = config.profile.algorithm
    return RslRlOnPolicyRunnerCfg(
        seed=config.seed,
        device=config.device,
        num_steps_per_env=config.steps_per_env,
        max_iterations=config.iterations,
        obs_groups={"policy": ["policy"], "critic": ["policy"]},
        clip_actions=1.0,
        save_interval=config.save_interval,
        experiment_name=config.profile.profile_id,
        policy=RslRlPpoActorCriticCfg(**policy),
        algorithm=RslRlPpoAlgorithmCfg(**algorithm),
    )


class GuardedOnPolicyRunner(OnPolicyRunner):
    """RSL-RL runner that fails before checkpointing non-finite policy state."""

    def __init__(self, *args: Any, metrics_path: Path | None = None, **kwargs: Any) -> None:
        self.metrics_path = metrics_path
        super().__init__(*args, **kwargs)

    def log(self, locs: dict[str, Any], width: int = 80, pad: int = 35) -> None:
        losses = {
            name: _finite_scalar(f"loss.{name}", value)
            for name, value in locs["loss_dict"].items()
        }
        for name, parameter in self.alg.policy.named_parameters():
            if not torch.isfinite(parameter).all():
                raise RuntimeError(f"non-finite policy parameter after update: {name}")

        super().log(locs, width=width, pad=pad)
        if self.metrics_path is None:
            return
        collection_size = self.num_steps_per_env * self.env.num_envs * self.gpu_world_size
        iteration_time = locs["collection_time"] + locs["learn_time"]
        record: dict[str, Any] = {
            "iteration": int(locs["it"]),
            "losses": losses,
            "learning_rate": _finite_scalar("learning_rate", self.alg.learning_rate),
            "mean_action_noise_std": _finite_scalar(
                "mean_action_noise_std", self.alg.policy.action_std.mean()
            ),
            "steps_per_second": collection_size / iteration_time,
            "total_timesteps": self.tot_timesteps,
            "collection_seconds": locs["collection_time"],
            "learning_seconds": locs["learn_time"],
        }
        if locs["rewbuffer"]:
            record["mean_episode_return"] = statistics.mean(locs["rewbuffer"])
            record["mean_episode_length"] = statistics.mean(locs["lenbuffer"])
        self.metrics_path.parent.mkdir(parents=True, exist_ok=True)
        with self.metrics_path.open("a", encoding="utf-8") as output:
            output.write(json.dumps(record, sort_keys=True) + "\n")


def _finite_scalar(name: str, value: Any) -> float:
    if isinstance(value, torch.Tensor):
        if value.numel() != 1:
            raise RuntimeError(f"training metric is not scalar: {name}")
        result = float(value.detach().item())
    else:
        result = float(value)
    if not math.isfinite(result):
        raise RuntimeError(f"non-finite training metric: {name}")
    return result
