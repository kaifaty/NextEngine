from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

import torch

from next_lab.motor_mirror import RATE_CLAMPED, validate_descriptor

try:
    import isaaclab.sim as sim_utils
    from isaaclab.assets import Articulation, ArticulationCfg
    from isaaclab.envs import DirectRLEnv, DirectRLEnvCfg
    from isaaclab.scene import InteractiveSceneCfg
    from isaaclab.sensors import ContactSensor, ContactSensorCfg
    from isaaclab.utils import configclass

    ISAAC_LAB_AVAILABLE = True
except ImportError:
    ISAAC_LAB_AVAILABLE = False


def round_div_ties_even_tensor(numerator: torch.Tensor, denominator: int) -> torch.Tensor:
    if denominator <= 0 or numerator.dtype != torch.int64:
        raise ValueError("integer ties-to-even requires int64 and a positive denominator")
    absolute = torch.abs(numerator)
    quotient = torch.div(absolute, denominator, rounding_mode="floor")
    remainder = torch.remainder(absolute, denominator)
    increment = (remainder * 2 > denominator) | (
        (remainder * 2 == denominator) & (torch.remainder(quotient, 2) == 1)
    )
    rounded = quotient + increment.to(torch.int64)
    return torch.where(numerator < 0, -rounded, rounded)


def fixed_pd_tensor(
    target: torch.Tensor,
    position: torch.Tensor,
    velocity: torch.Tensor,
    previous_effort: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor]:
    for value in (target, position, velocity, previous_effort):
        if value.dtype != torch.int64:
            raise ValueError("fixed PD inputs must be int64")
    target_limited = torch.clamp(target, -1_500_000, 1_500_000)
    proportional = round_div_ties_even_tensor((80 * 65_536) * (target_limited - position), 65_536)
    damping = round_div_ties_even_tensor((4 * 65_536) * velocity, 65_536)
    requested = proportional - damping
    effort_limited = torch.clamp(requested, -150_000_000, 150_000_000)
    maximum_delta = 25_000_000
    effort = torch.minimum(
        torch.maximum(effort_limited, previous_effort - maximum_delta),
        previous_effort + maximum_delta,
    )
    flags = torch.zeros_like(effort)
    flags |= (target_limited != target).to(torch.int64)
    flags |= ((effort_limited != requested).to(torch.int64) << 1)
    flags |= ((effort != effort_limited).to(torch.int64) * RATE_CLAMPED)
    return effort, flags


if ISAAC_LAB_AVAILABLE:

    @configclass
    class NextEngineHumanoidDirectEnvCfg(DirectRLEnvCfg):
        decimation = 4
        episode_length_s = 60.0
        action_space = 23
        observation_space = 84
        state_space = 0
        sim = sim_utils.SimulationCfg(dt=1.0 / 240.0, render_interval=4)
        scene = InteractiveSceneCfg(num_envs=4_096, env_spacing=3.0, replicate_physics=True)
        asset = ArticulationCfg(
            prim_path="/World/envs/env_.*/Humanoid",
            spawn=sim_utils.UsdFileCfg(usd_path=os.environ.get("NEXTENGINE_HUMANOID_USD", "")),
        )
        feet = ContactSensorCfg(
            prim_path="/World/envs/env_.*/Humanoid/Bodies/body_.*_ankle_roll",
            update_period=0.0,
            history_length=1,
            track_air_time=True,
        )
        reward_coefficients = (1.0, 1.0, 1.0, 1.0, 0.000001, 0.000001, 1.0, 1.0)


    class NextEngineHumanoidDirectEnv(DirectRLEnv):
        cfg: NextEngineHumanoidDirectEnvCfg

        def __init__(self, cfg: NextEngineHumanoidDirectEnvCfg, descriptor_path: str, **kwargs: Any):
            descriptor = json.loads(Path(descriptor_path).read_text(encoding="utf-8"))
            validate_descriptor(descriptor)
            self.descriptor = descriptor
            self._action = torch.zeros((cfg.scene.num_envs, 23), dtype=torch.int64)
            self._previous_action = torch.zeros_like(self._action)
            self._previous_effort = torch.zeros_like(self._action)
            self.reward_components = torch.zeros((cfg.scene.num_envs, 8))
            super().__init__(cfg, **kwargs)

        def _setup_scene(self) -> None:
            self.robot = Articulation(self.cfg.asset)
            self.feet = ContactSensor(self.cfg.feet)
            self.scene.articulations["humanoid"] = self.robot
            self.scene.sensors["feet"] = self.feet
            sim_utils.spawn_ground_plane("/World/ground", sim_utils.GroundPlaneCfg())
            self.scene.clone_environments(copy_from_source=False)

        def _pre_physics_step(self, actions: torch.Tensor) -> None:
            if actions.shape != (self.num_envs, 23) or not torch.isfinite(actions).all():
                raise ValueError("invalid Stage 0 action batch")
            self._previous_action.copy_(self._action)
            self._action.copy_(torch.round(torch.clamp(actions, -1.0, 1.0) * 1_000_000).to(torch.int64))

        def _apply_action(self) -> None:
            position = torch.round(self.robot.data.joint_pos * 1_000_000).to(torch.int64)
            velocity = torch.round(self.robot.data.joint_vel * 1_000_000).to(torch.int64)
            effort, _ = fixed_pd_tensor(self._action, position, velocity, self._previous_effort)
            self._previous_effort.copy_(effort)
            self.robot.set_joint_effort_target(effort.to(torch.float32) / 1_000_000.0)

        def _get_observations(self) -> dict[str, torch.Tensor]:
            data = self.robot.data
            contacts = torch.linalg.vector_norm(self.feet.data.net_forces_w, dim=-1) > 1.0e-6
            command = torch.zeros((self.num_envs, 3), device=self.device)
            policy = torch.cat(
                (
                    data.root_quat_w,
                    data.root_lin_vel_b,
                    data.root_ang_vel_b,
                    data.joint_pos,
                    data.joint_vel,
                    self._action.to(torch.float32) / 1_000_000.0,
                    command,
                    contacts.to(torch.float32),
                ),
                dim=-1,
            )
            return {"policy": policy}

        def _get_rewards(self) -> torch.Tensor:
            data = self.robot.data
            upright = torch.abs(data.root_quat_w[:, 0])
            root_height = -torch.abs(data.root_pos_w[:, 2] - 1.05)
            standing_pose = -torch.sum(torch.abs(data.joint_pos), dim=-1)
            velocity = -torch.sum(torch.abs(data.root_lin_vel_b), dim=-1) - torch.sum(
                torch.abs(data.root_ang_vel_b), dim=-1
            )
            effort = -torch.sum(torch.abs(self._previous_effort), dim=-1).to(torch.float32)
            action_rate = -torch.sum(torch.abs(self._action - self._previous_action), dim=-1).to(torch.float32)
            foot_slip = torch.zeros_like(upright)
            fall = -(data.root_pos_w[:, 2] <= 0.25).to(torch.float32)
            self.reward_components = torch.stack(
                (upright, root_height, standing_pose, velocity, effort, action_rate, foot_slip, fall),
                dim=-1,
            )
            coefficients = torch.tensor(self.cfg.reward_coefficients, device=self.device)
            return torch.sum(self.reward_components * coefficients, dim=-1)

        def _get_dones(self) -> tuple[torch.Tensor, torch.Tensor]:
            fallen = self.robot.data.root_pos_w[:, 2] <= 0.25
            timed_out = self.episode_length_buf >= self.max_episode_length - 1
            return fallen, timed_out

        def _reset_idx(self, env_ids: torch.Tensor | None) -> None:
            super()._reset_idx(env_ids)
            if env_ids is None:
                env_ids = self.robot._ALL_INDICES
            self._action[env_ids] = 0
            self._previous_action[env_ids] = 0
            self._previous_effort[env_ids] = 0

else:

    class NextEngineHumanoidDirectEnvCfg:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")


    class NextEngineHumanoidDirectEnv:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")
