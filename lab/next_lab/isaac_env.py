from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import Any, Iterable

import torch

from next_lab.motor_mirror import (
    FLAT_LOCOMOTION_PROFILE_ID,
    Q1_30_ONE,
    RATE_CLAMPED,
    STANDING_PROFILE_ID,
    derive_purpose_seed,
    flat_locomotion_command_schedule,
    select_environment_profile,
    validate_descriptor,
)

try:
    import isaaclab.sim as sim_utils
    from isaaclab.actuators import ImplicitActuatorCfg
    from isaaclab.assets import Articulation, ArticulationCfg
    from isaaclab.envs import DirectRLEnv, DirectRLEnvCfg
    from isaaclab.scene import InteractiveSceneCfg
    from isaaclab.sensors import ContactSensor, ContactSensorCfg
    from isaaclab.utils import configclass

    ISAAC_LAB_AVAILABLE = True
except ImportError:
    ISAAC_LAB_AVAILABLE = False

LOCOMOTION_REWARD_COEFFICIENTS_Q16 = (
    98_304,
    32_768,
    32_768,
    16_384,
    -3_277,
    -3_277,
    -1_311,
    -3_277,
    -6_554,
    -131_072,
)


def require_finite_tensor(name: str, value: torch.Tensor) -> None:
    if not torch.isfinite(value).all():
        raise RuntimeError(f"non-finite Isaac tensor: {name}")


def isaac_prim_name(identifier: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", identifier)


def isaac_actuator_limits_from_descriptor(
    descriptor: dict[str, Any],
) -> dict[str, dict[str, float]]:
    """Map engine-owned per-joint safety limits into Isaac actuator parameters."""
    velocities: dict[str, float] = {}
    for joint in descriptor["joints"]:
        raw = joint["maximum_velocity_microradians_per_second"]
        if not isinstance(raw, int) or isinstance(raw, bool) or raw <= 0:
            raise ValueError("descriptor joint velocity limits must be positive integers")
        velocities[isaac_prim_name(joint["joint_id"])] = raw / 1_000_000.0

    efforts: dict[str, float] = {}
    for actuator in descriptor["actuators"]:
        raw = actuator["maximum_effort_micronewton_metres"]
        if not isinstance(raw, int) or isinstance(raw, bool) or raw <= 0:
            raise ValueError("descriptor actuator effort limits must be positive integers")
        efforts[isaac_prim_name(actuator["joint_id"])] = raw / 1_000_000.0

    if velocities.keys() != efforts.keys():
        raise ValueError("descriptor joints and actuators must cover the same joint IDs")
    return {
        "effort_limit_sim": efforts,
        "velocity_limit_sim": velocities,
    }


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


def ratio_q16_tensor(value: torch.Tensor, maximum: int) -> torch.Tensor:
    if value.dtype != torch.int64 or maximum <= 0 or torch.any(value < 0):
        raise ValueError("Q16 ratio requires non-negative int64 values and a positive maximum")
    return round_div_ties_even_tensor(torch.clamp(value, max=maximum) * 65_536, maximum)


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


def engine_vector_from_isaac_tensor(vector: torch.Tensor) -> torch.Tensor:
    """Map Isaac (+X,+Y,+Z) to engine (+right,+up,+forward)."""
    if vector.shape[-1] != 3:
        raise ValueError("vector must end in three components")
    return torch.stack((vector[..., 0], vector[..., 2], -vector[..., 1]), dim=-1)


def engine_quaternion_xyzw_from_isaac_wxyz_tensor(quaternion: torch.Tensor) -> torch.Tensor:
    if quaternion.shape[-1] != 4:
        raise ValueError("quaternion must end in four components")
    w, x, y, z = quaternion.unbind(dim=-1)
    return torch.stack((x, z, -y, w), dim=-1)


def isaac_root_state_from_descriptor(descriptor: dict[str, Any]) -> tuple[float, ...]:
    roots = [body for body in descriptor["bodies"] if body["parent_body_id"] is None]
    if len(roots) != 1:
        raise ValueError("descriptor must declare exactly one root body")
    root = roots[0]
    x, up, forward = root["local_bind_translation_micrometres"]
    qx, qy, qz, qw = root["local_bind_rotation_q1_30"]
    scale = float(Q1_30_ONE)
    return (
        x / 1_000_000.0,
        -forward / 1_000_000.0,
        up / 1_000_000.0,
        qw / scale,
        qx / scale,
        -qz / scale,
        qy / scale,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    )


def rotate_world_to_root_local_q1_30_tensor(
    quaternion_xyzw_q1_30: torch.Tensor, world_vector: torch.Tensor
) -> torch.Tensor:
    if (
        quaternion_xyzw_q1_30.dtype != torch.int64
        or world_vector.dtype != torch.int64
        or quaternion_xyzw_q1_30.shape[:-1] != world_vector.shape[:-1]
        or quaternion_xyzw_q1_30.shape[-1] != 4
        or world_vector.shape[-1] != 3
    ):
        raise ValueError("root-local transform requires matching int64 [...,4] and [...,3]")
    if torch.any(torch.abs(quaternion_xyzw_q1_30) > Q1_30_ONE):
        raise OverflowError("quaternion is outside Q1.30")
    x, y, z, w = quaternion_xyzw_q1_30.unbind(dim=-1)

    def coefficient(value: torch.Tensor) -> torch.Tensor:
        return round_div_ties_even_tensor(value, Q1_30_ONE)

    def diagonal(left: torch.Tensor, right: torch.Tensor) -> torch.Tensor:
        return Q1_30_ONE - coefficient(2 * (left * left + right * right))

    rows = (
        (
            diagonal(y, z),
            coefficient(2 * (x * y - z * w)),
            coefficient(2 * (x * z + y * w)),
        ),
        (
            coefficient(2 * (x * y + z * w)),
            diagonal(x, z),
            coefficient(2 * (y * z - x * w)),
        ),
        (
            coefficient(2 * (x * z - y * w)),
            coefficient(2 * (y * z + x * w)),
            diagonal(x, y),
        ),
    )
    output = []
    for column in range(3):
        dot = sum(rows[row][column] * world_vector[..., row] for row in range(3))
        output.append(round_div_ties_even_tensor(dot, Q1_30_ONE))
    return torch.stack(output, dim=-1)


def precompute_flat_command_schedules(
    run_root: bytes,
    episode_ordinals: Iterable[int],
    vector_slots: Iterable[int],
) -> torch.Tensor:
    ordinals = list(episode_ordinals)
    slots = list(vector_slots)
    if len(ordinals) != len(slots):
        raise ValueError("episode ordinals and vector slots must have equal length")
    schedules = [
        flat_locomotion_command_schedule(
            derive_purpose_seed(run_root, ordinal, slot, "randomization.command")
        )
        for ordinal, slot in zip(ordinals, slots, strict=True)
    ]
    return torch.tensor(schedules, dtype=torch.int64, device="cpu")


def locomotion_reward_q16_tensor(
    *,
    quaternion_xyzw_q1_30: torch.Tensor,
    root_height_micrometres: torch.Tensor,
    local_linear_velocity_raw: torch.Tensor,
    local_angular_velocity_raw: torch.Tensor,
    vertical_velocity_raw: torch.Tensor,
    command_raw: torch.Tensor,
    effort_sum_raw: torch.Tensor,
    applied_action_raw: torch.Tensor,
    previous_applied_action_raw: torch.Tensor,
    contacting_foot_slip_sum_raw: torch.Tensor,
    contacting_foot_count: torch.Tensor,
    fell: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor]:
    """Evaluate the ten canonical locomotion components and weighted Q16 total."""
    planar_error = torch.abs(local_linear_velocity_raw[:, 0] - command_raw[:, 0]) + torch.abs(
        local_linear_velocity_raw[:, 2] - command_raw[:, 1]
    )
    planar = 65_536 - ratio_q16_tensor(planar_error, 6_500_000)
    yaw = 65_536 - ratio_q16_tensor(
        torch.abs(local_angular_velocity_raw[:, 1] - command_raw[:, 2]), 3_000_000
    )
    x = quaternion_xyzw_q1_30[:, 0]
    z = quaternion_xyzw_q1_30[:, 2]
    tilt_reduction = round_div_ties_even_tensor(2 * (x * x + z * z), Q1_30_ONE)
    upright_q30 = torch.clamp(Q1_30_ONE - tilt_reduction, 0, Q1_30_ONE)
    upright = ratio_q16_tensor(upright_q30, Q1_30_ONE)
    height = 65_536 - ratio_q16_tensor(
        torch.abs(root_height_micrometres - 1_050_000), 600_000
    )
    vertical = ratio_q16_tensor(torch.abs(vertical_velocity_raw), 3_000_000)
    roll_pitch = ratio_q16_tensor(
        torch.abs(local_angular_velocity_raw[:, 0])
        + torch.abs(local_angular_velocity_raw[:, 2]),
        6_000_000,
    )
    effort = ratio_q16_tensor(effort_sum_raw, 23 * 4 * 150_000_000)
    action_rate = ratio_q16_tensor(
        torch.sum(torch.abs(applied_action_raw - previous_applied_action_raw), dim=-1),
        23 * 2_000_000,
    )
    slip_denominator = contacting_foot_count * 4_000_000
    slip = torch.zeros_like(contacting_foot_slip_sum_raw)
    # The denominator is per environment; calculate the exact ratio without float math.
    nonzero = slip_denominator > 0
    if torch.any(nonzero):
        bounded = torch.minimum(contacting_foot_slip_sum_raw[nonzero], slip_denominator[nonzero])
        numerator = bounded * 65_536
        quotient = torch.div(numerator, slip_denominator[nonzero], rounding_mode="floor")
        remainder = torch.remainder(numerator, slip_denominator[nonzero])
        increment = (remainder * 2 > slip_denominator[nonzero]) | (
            (remainder * 2 == slip_denominator[nonzero]) & (quotient % 2 == 1)
        )
        slip[nonzero] = quotient + increment.to(torch.int64)
    fall = fell.to(torch.int64) * 65_536
    components = torch.stack(
        (planar, yaw, upright, height, vertical, roll_pitch, effort, action_rate, slip, fall),
        dim=-1,
    )
    coefficients = torch.tensor(
        LOCOMOTION_REWARD_COEFFICIENTS_Q16,
        dtype=torch.int64,
        device=components.device,
    )
    total = torch.sum(round_div_ties_even_tensor(components * coefficients, 65_536), dim=-1)
    return components, total


if ISAAC_LAB_AVAILABLE:

    @configclass
    class NextEngineHumanoidDirectEnvCfg(DirectRLEnvCfg):
        decimation = 4
        episode_length_s = 20.0
        action_space = 23
        observation_space = 84
        state_space = 0
        environment_profile_id = FLAT_LOCOMOTION_PROFILE_ID
        run_root_hex = "00" * 32
        sim = sim_utils.SimulationCfg(dt=1.0 / 240.0, render_interval=4)
        scene = InteractiveSceneCfg(num_envs=4_096, env_spacing=3.0, replicate_physics=True)
        asset = ArticulationCfg(
            prim_path="/World/envs/env_.*/Humanoid",
            spawn=sim_utils.UsdFileCfg(
                usd_path=os.environ.get("NEXTENGINE_HUMANOID_USD", ""),
                activate_contact_sensors=True,
            ),
            actuators={
                "engine_effort": ImplicitActuatorCfg(
                    joint_names_expr=[".*"],
                    stiffness=0.0,
                    damping=0.0,
                    effort_limit_sim=None,
                    velocity_limit_sim=None,
                )
            },
        )
        feet = ContactSensorCfg(
            prim_path="/World/envs/env_.*/Humanoid/Bodies/body_.*_ankle_roll",
            update_period=0.0,
            history_length=1,
            track_air_time=True,
        )
        standing_reward_coefficients = (1.0,) * 8


    class NextEngineHumanoidDirectEnv(DirectRLEnv):
        cfg: NextEngineHumanoidDirectEnvCfg

        def __init__(self, cfg: NextEngineHumanoidDirectEnvCfg, descriptor_path: str, **kwargs: Any):
            descriptor = json.loads(Path(descriptor_path).read_text(encoding="utf-8"))
            validate_descriptor(descriptor)
            self.profile = select_environment_profile(descriptor, cfg.environment_profile_id)
            self.descriptor = descriptor
            self.isaac_actuator_limits = isaac_actuator_limits_from_descriptor(descriptor)
            actuator_cfg = cfg.asset.actuators["engine_effort"]
            actuator_cfg.effort_limit_sim = self.isaac_actuator_limits["effort_limit_sim"]
            actuator_cfg.velocity_limit_sim = self.isaac_actuator_limits["velocity_limit_sim"]
            self.run_root = bytes.fromhex(cfg.run_root_hex)
            if len(self.run_root) != 32:
                raise ValueError("run_root_hex must encode 32 bytes")
            cfg.episode_length_s = self.profile["maximum_episode_steps"] / 60.0
            self._action = torch.zeros((cfg.scene.num_envs, 23), dtype=torch.int64)
            self._previous_action = torch.zeros_like(self._action)
            self._previous_effort = torch.zeros_like(self._action)
            self._effort_sum = torch.zeros(cfg.scene.num_envs, dtype=torch.int64)
            self._episode_ordinals = torch.full((cfg.scene.num_envs,), -1, dtype=torch.int64)
            self._command_schedule = torch.zeros((cfg.scene.num_envs, 1_201, 3), dtype=torch.int64)
            component_count = len(self.profile["reward_components"])
            self.reward_components_q16 = torch.zeros(
                (cfg.scene.num_envs, component_count), dtype=torch.int64
            )
            self._episode_reward_sum = torch.zeros(
                cfg.scene.num_envs, dtype=torch.float32
            )
            self._episode_component_sums = torch.zeros(
                (cfg.scene.num_envs, component_count), dtype=torch.float32
            )
            super().__init__(cfg, **kwargs)
            canonical_joint_names = [
                isaac_prim_name(actuator["joint_id"])
                for actuator in self.descriptor["actuators"]
            ]
            self._canonical_joint_ids, resolved_joint_names = self.robot.find_joints(
                canonical_joint_names, preserve_order=True
            )
            if resolved_joint_names != canonical_joint_names:
                raise RuntimeError("Isaac articulation does not match canonical actuator order")
            self._foot_body_ids, _ = self.robot.find_bodies(
                self.feet.body_names, preserve_order=True
            )
            if len(self._foot_body_ids) != 2:
                raise RuntimeError("descriptor declares exactly two foot effectors")
            self._capture_reset_defaults()
            for name in (
                "_action",
                "_previous_action",
                "_previous_effort",
                "_effort_sum",
                "_episode_ordinals",
                "_command_schedule",
                "reward_components_q16",
                "_episode_reward_sum",
                "_episode_component_sums",
            ):
                setattr(self, name, getattr(self, name).to(self.device))

        def _capture_reset_defaults(self) -> None:
            """Close the engine-authored pose into deterministic reset templates."""
            root_template = torch.tensor(
                isaac_root_state_from_descriptor(self.descriptor),
                dtype=torch.float32,
                device=self.device,
            ).unsqueeze(0)
            initial_root_state = self.robot.data.root_state_w.clone()
            initial_root_state[:, :3] -= self.scene.env_origins
            root_pose_deviation = torch.max(
                torch.abs(
                    initial_root_state[:, :7]
                    - root_template[:, :7].expand_as(initial_root_state[:, :7])
                )
            )
            initial_root_velocity = torch.max(torch.abs(initial_root_state[:, 7:]))
            if root_pose_deviation > 0.01:
                raise RuntimeError(
                    "initial PhysX root pose does not match the engine descriptor: "
                    f"maximum deviation {float(root_pose_deviation.item())}"
                )

            joint_template = self.robot.data.default_joint_pos[0].unsqueeze(0)
            joint_deviation = torch.max(
                torch.abs(
                    self.robot.data.joint_pos
                    - joint_template.expand_as(self.robot.data.joint_pos)
                )
            )

            self.reset_template_initial_deviation = {
                "root_pose": float(root_pose_deviation.item()),
                "root_velocity": float(initial_root_velocity.item()),
                "joint_position": float(joint_deviation.item()),
            }
            self.robot.data.default_root_state.copy_(
                root_template.expand_as(self.robot.data.default_root_state)
            )
            self.robot.data.default_joint_pos.copy_(
                joint_template.expand_as(self.robot.data.default_joint_pos)
            )
            self.robot.data.default_joint_vel.zero_()

        def _setup_scene(self) -> None:
            self.robot = Articulation(self.cfg.asset)
            self.feet = ContactSensor(self.cfg.feet)
            self.scene.articulations["humanoid"] = self.robot
            self.scene.sensors["feet"] = self.feet
            sim_utils.spawn_ground_plane("/World/ground", sim_utils.GroundPlaneCfg())
            self.scene.clone_environments(copy_from_source=False)

        def _pre_physics_step(self, actions: torch.Tensor) -> None:
            self.extras.pop("log", None)
            if actions.shape != (self.num_envs, 23) or not torch.isfinite(actions).all():
                raise ValueError("invalid canonical action batch")
            self._previous_action.copy_(self._action)
            self._action.copy_(
                torch.round(torch.clamp(actions, -1.0, 1.0) * 1_000_000).to(torch.int64)
            )
            self._effort_sum.zero_()

        def _apply_action(self) -> None:
            joint_position = self.robot.data.joint_pos[:, self._canonical_joint_ids]
            joint_velocity = self.robot.data.joint_vel[:, self._canonical_joint_ids]
            require_finite_tensor("joint_pos", joint_position)
            require_finite_tensor("joint_vel", joint_velocity)
            position = torch.round(joint_position * 1_000_000).to(torch.int64)
            velocity = torch.round(joint_velocity * 1_000_000).to(torch.int64)
            effort, _ = fixed_pd_tensor(self._action, position, velocity, self._previous_effort)
            self._previous_effort.copy_(effort)
            self._effort_sum.add_(torch.sum(torch.abs(effort), dim=-1))
            self.robot.set_joint_effort_target(
                effort.to(torch.float32) / 1_000_000.0,
                joint_ids=self._canonical_joint_ids,
            )

        def _canonical_facts(self) -> tuple[torch.Tensor, ...]:
            data = self.robot.data
            require_finite_tensor("root_quat_w", data.root_quat_w)
            require_finite_tensor("root_lin_vel_w", data.root_lin_vel_w)
            require_finite_tensor("root_ang_vel_w", data.root_ang_vel_w)
            require_finite_tensor("foot_net_forces_w", self.feet.data.net_forces_w)
            quaternion = engine_quaternion_xyzw_from_isaac_wxyz_tensor(data.root_quat_w)
            quaternion_raw = torch.round(quaternion * Q1_30_ONE).to(torch.int64)
            linear_world = engine_vector_from_isaac_tensor(data.root_lin_vel_w)
            angular_world = engine_vector_from_isaac_tensor(data.root_ang_vel_w)
            linear_raw = torch.round(linear_world * 1_000_000).to(torch.int64)
            angular_raw = torch.round(angular_world * 1_000_000).to(torch.int64)
            if self.profile["profile_id"] == FLAT_LOCOMOTION_PROFILE_ID:
                linear_raw = rotate_world_to_root_local_q1_30_tensor(quaternion_raw, linear_raw)
                angular_raw = rotate_world_to_root_local_q1_30_tensor(quaternion_raw, angular_raw)
            contacts = torch.linalg.vector_norm(self.feet.data.net_forces_w, dim=-1) > 1.0e-6
            if contacts.shape[-1] != 2:
                raise RuntimeError("descriptor declares exactly two foot effectors")
            return quaternion, quaternion_raw, linear_raw, angular_raw, contacts

        def _current_command(self) -> torch.Tensor:
            if self.profile["profile_id"] == STANDING_PROFILE_ID:
                return torch.zeros((self.num_envs, 3), dtype=torch.int64, device=self.device)
            ticks = torch.clamp(self.episode_length_buf, 0, 1_200).to(torch.int64)
            envs = torch.arange(self.num_envs, device=self.device)
            return self._command_schedule[envs, ticks]

        def _get_observations(self) -> dict[str, torch.Tensor]:
            quaternion, _, linear_raw, angular_raw, contacts = self._canonical_facts()
            command = self._current_command()
            policy = torch.cat(
                (
                    quaternion,
                    linear_raw.to(torch.float32) / 1_000_000.0,
                    angular_raw.to(torch.float32) / 1_000_000.0,
                    self.robot.data.joint_pos[:, self._canonical_joint_ids],
                    self.robot.data.joint_vel[:, self._canonical_joint_ids],
                    self._action.to(torch.float32) / 1_000_000.0,
                    command.to(torch.float32) / 1_000_000.0,
                    contacts.to(torch.float32),
                ),
                dim=-1,
            )
            if policy.shape[-1] != 84:
                raise RuntimeError("canonical observation width mismatch")
            require_finite_tensor("policy_observation", policy)
            return {"policy": policy}

        def _get_rewards(self) -> torch.Tensor:
            if self.profile["profile_id"] == STANDING_PROFILE_ID:
                return self._standing_rewards()
            _, quaternion_raw, linear_raw, angular_raw, contacts = self._canonical_facts()
            require_finite_tensor("root_pos_w", self.robot.data.root_pos_w)
            require_finite_tensor("body_lin_vel_w", self.robot.data.body_lin_vel_w)
            root_height = torch.round(self.robot.data.root_pos_w[:, 2] * 1_000_000).to(torch.int64)
            vertical_velocity = torch.round(
                self.robot.data.root_lin_vel_w[:, 2] * 1_000_000
            ).to(torch.int64)
            fallen = root_height <= 450_000
            foot_velocity = engine_vector_from_isaac_tensor(
                self.robot.data.body_lin_vel_w[:, self._foot_body_ids, :]
            )
            slip_per_foot = torch.round(
                (torch.abs(foot_velocity[..., 0]) + torch.abs(foot_velocity[..., 2])) * 1_000_000
            ).to(torch.int64)
            slip_sum = torch.sum(slip_per_foot * contacts.to(torch.int64), dim=-1)
            components, total = locomotion_reward_q16_tensor(
                quaternion_xyzw_q1_30=quaternion_raw,
                root_height_micrometres=root_height,
                local_linear_velocity_raw=linear_raw,
                local_angular_velocity_raw=angular_raw,
                vertical_velocity_raw=vertical_velocity,
                command_raw=self._current_command(),
                effort_sum_raw=self._effort_sum,
                applied_action_raw=self._action,
                previous_applied_action_raw=self._previous_action,
                contacting_foot_slip_sum_raw=slip_sum,
                contacting_foot_count=torch.sum(contacts.to(torch.int64), dim=-1),
                fell=fallen,
            )
            self.reward_components_q16.copy_(components)
            reward = total.to(torch.float32) / 65_536.0
            component_values = components.to(torch.float32) / 65_536.0
            self._accumulate_episode_metrics(reward, component_values)
            return reward

        def _standing_rewards(self) -> torch.Tensor:
            data = self.robot.data
            require_finite_tensor("standing_root_quat_w", data.root_quat_w)
            require_finite_tensor("standing_root_pos_w", data.root_pos_w)
            require_finite_tensor("standing_joint_pos", data.joint_pos)
            require_finite_tensor("standing_root_lin_vel_w", data.root_lin_vel_w)
            require_finite_tensor("standing_root_ang_vel_w", data.root_ang_vel_w)
            upright = torch.abs(data.root_quat_w[:, 0])
            root_height = -torch.abs(data.root_pos_w[:, 2] - 1.05)
            standing_pose = -torch.sum(torch.abs(data.joint_pos), dim=-1)
            velocity = -torch.sum(torch.abs(data.root_lin_vel_w), dim=-1) - torch.sum(
                torch.abs(data.root_ang_vel_w), dim=-1
            )
            effort = -torch.sum(torch.abs(self._previous_effort), dim=-1).to(torch.float32)
            action_rate = -torch.sum(torch.abs(self._action - self._previous_action), dim=-1).to(
                torch.float32
            )
            foot_slip = torch.zeros_like(upright)
            fall = -(data.root_pos_w[:, 2] <= 0.25).to(torch.float32)
            components = torch.stack(
                (upright, root_height, standing_pose, velocity, effort, action_rate, foot_slip, fall),
                dim=-1,
            )
            coefficients = torch.tensor(
                self.cfg.standing_reward_coefficients, device=self.device
            )
            reward = torch.sum(components * coefficients, dim=-1)
            require_finite_tensor("standing_reward", reward)
            self._accumulate_episode_metrics(reward, components)
            return reward

        def _accumulate_episode_metrics(
            self, reward: torch.Tensor, components: torch.Tensor
        ) -> None:
            require_finite_tensor("reward", reward)
            require_finite_tensor("reward_components", components)
            self._episode_reward_sum.add_(reward)
            self._episode_component_sums.add_(components)

        def _get_dones(self) -> tuple[torch.Tensor, torch.Tensor]:
            root = self.robot.data.root_pos_w
            require_finite_tensor("done_root_pos_w", root)
            threshold = 0.45 if self.profile["profile_id"] == FLAT_LOCOMOTION_PROFILE_ID else 0.25
            terminated = root[:, 2] <= threshold
            if self.profile["profile_id"] == FLAT_LOCOMOTION_PROFILE_ID:
                displacement = root[:, :2] - self.scene.env_origins[:, :2]
                terminated |= (torch.abs(displacement[:, 0]) >= 90.0) | (
                    torch.abs(displacement[:, 1]) >= 90.0
                )
            timed_out = self.episode_length_buf >= self.max_episode_length - 1
            return terminated, timed_out

        def _reset_idx(self, env_ids: torch.Tensor | None) -> None:
            if env_ids is None:
                env_ids = self.robot._ALL_INDICES
            completed = self.episode_length_buf[env_ids] > 0
            if torch.any(completed):
                completed_ids = env_ids[completed]
                lengths = self.episode_length_buf[completed_ids].to(torch.float32)
                log: dict[str, torch.Tensor] = {
                    "Episode/return": self._episode_reward_sum[completed_ids],
                    "Episode/length": lengths,
                    "Episode/terminated": self.reset_terminated[completed_ids].to(
                        torch.float32
                    ),
                    "Episode/truncated": self.reset_time_outs[completed_ids].to(
                        torch.float32
                    ),
                }
                for index, component in enumerate(self.profile["reward_components"]):
                    component_id = component["component_id"].removeprefix("reward.")
                    log[f"Episode_Component/{component_id}"] = (
                        self._episode_component_sums[completed_ids, index] / lengths
                    )
                self.extras["log"] = log
            super()._reset_idx(env_ids)
            root_state = self.robot.data.default_root_state[env_ids].clone()
            root_state[:, :3] += self.scene.env_origins[env_ids]
            joint_position = self.robot.data.default_joint_pos[env_ids].clone()
            joint_velocity = self.robot.data.default_joint_vel[env_ids].clone()
            self.robot.write_root_state_to_sim(root_state, env_ids)
            self.robot.write_joint_state_to_sim(
                joint_position, joint_velocity, env_ids=env_ids
            )
            self.robot.set_joint_effort_target(
                torch.zeros_like(joint_position), env_ids=env_ids
            )
            self._action[env_ids] = 0
            self._previous_action[env_ids] = 0
            self._previous_effort[env_ids] = 0
            self._effort_sum[env_ids] = 0
            self._episode_reward_sum[env_ids] = 0.0
            self._episode_component_sums[env_ids] = 0.0
            self._episode_ordinals[env_ids] += 1
            if self.profile["profile_id"] == FLAT_LOCOMOTION_PROFILE_ID:
                ids = env_ids.detach().cpu().tolist()
                ordinals = self._episode_ordinals[env_ids].detach().cpu().tolist()
                schedules = precompute_flat_command_schedules(self.run_root, ordinals, ids)
                self._command_schedule[env_ids] = schedules.to(self.device)

else:

    class NextEngineHumanoidDirectEnvCfg:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")


    class NextEngineHumanoidDirectEnv:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")
