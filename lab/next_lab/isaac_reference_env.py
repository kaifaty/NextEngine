from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import Any

import numpy as np
import torch

from next_lab.isaac_env import (
    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
    engine_vector_from_isaac_tensor,
    isaac_actuator_limits_from_descriptor,
    require_finite_tensor,
)
from next_lab.motor_mirror import validate_biomechanics_descriptor
from next_lab.reference_tracker import (
    ACTION_CHANNELS,
    OBSERVATION_CHANNELS,
    REFERENCE_OFFSETS,
    ReferenceCorpus,
    ReferenceTrackerProfile,
    derive_named_seed,
)

try:
    import isaaclab.sim as sim_utils
    from isaaclab.actuators import ImplicitActuatorCfg
    from isaaclab.assets import Articulation, ArticulationCfg
    from isaaclab.envs import DirectRLEnv, DirectRLEnvCfg
    from isaaclab.scene import InteractiveSceneCfg
    from isaaclab.sensors import ContactSensor, ContactSensorCfg
    from isaaclab.utils import configclass
    from isaaclab.utils.math import quat_apply

    ISAAC_LAB_AVAILABLE = True
except ImportError:
    ISAAC_LAB_AVAILABLE = False


Q1_30 = 1 << 30
OBSERVED_HARD_ROM_TOLERANCE_MICRORADIANS = 10
PHYSICS_SUBSTEPS_PER_SECOND = 240
MICRO_SCALE = 1_000_000


def _prim(identifier: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", identifier)


def _engine_to_isaac_vector(value: torch.Tensor) -> torch.Tensor:
    if value.shape[-1] != 3:
        raise ValueError("engine vector must have three components")
    x, y, z = value.unbind(dim=-1)
    return torch.stack((x, -z, y), dim=-1)


def _engine_xyzw_to_isaac_wxyz(value: torch.Tensor) -> torch.Tensor:
    if value.shape[-1] != 4:
        raise ValueError("engine quaternion must have four components")
    x, y, z, w = value.unbind(dim=-1)
    return torch.stack((w, x, -z, y), dim=-1)


def _normalized_xyzw(value: torch.Tensor) -> torch.Tensor:
    norm = torch.linalg.vector_norm(value, dim=-1, keepdim=True)
    valid = torch.all(torch.isfinite(norm) & (norm > 1.0e-12))
    if value.device.type == "cuda":
        torch._assert_async(valid, "invalid reference quaternion")
    elif not bool(valid.item()):
        raise RuntimeError("invalid reference quaternion")
    result = value / norm
    return torch.where(result[..., 3:4] < 0.0, -result, result)


def _rotate_inverse_xyzw(vector: torch.Tensor, quaternion: torch.Tensor) -> torch.Tensor:
    quaternion = _normalized_xyzw(quaternion.to(vector.dtype))
    xyz = quaternion[..., :3]
    w = quaternion[..., 3:4]
    first = torch.linalg.cross(xyz, vector, dim=-1)
    second = torch.linalg.cross(xyz, first - w * vector, dim=-1)
    return vector + 2.0 * second


def _quaternion_multiply_xyzw(left: torch.Tensor, right: torch.Tensor) -> torch.Tensor:
    dtype = torch.promote_types(left.dtype, right.dtype)
    left = left.to(dtype)
    right = right.to(dtype)
    left_xyz, left_w = left[..., :3], left[..., 3:4]
    right_xyz, right_w = right[..., :3], right[..., 3:4]
    xyz = (
        left_w * right_xyz
        + right_w * left_xyz
        + torch.linalg.cross(left_xyz, right_xyz, dim=-1)
    )
    w = left_w * right_w - torch.sum(left_xyz * right_xyz, dim=-1, keepdim=True)
    return _normalized_xyzw(torch.cat((xyz, w), dim=-1))


def _quaternion_conjugate_xyzw(value: torch.Tensor) -> torch.Tensor:
    return torch.cat((-value[..., :3], value[..., 3:4]), dim=-1)


def _round_int64(value: torch.Tensor) -> torch.Tensor:
    finite = torch.isfinite(value).all()
    if value.device.type == "cuda":
        torch._assert_async(finite, "non-finite value before canonical rounding")
    elif not bool(finite.item()):
        raise RuntimeError("non-finite value before canonical rounding")
    return torch.round(value).to(torch.int64)


def _select_curriculum_episode(
    *,
    run_root: bytes,
    episode_ordinal: int,
    vector_slot: int,
    clip_frame_counts: tuple[int, ...],
    horizon_motor_ticks: int,
) -> tuple[int, int, int]:
    if (
        len(run_root) != 32
        or episode_ordinal < 0
        or vector_slot < 0
        or not clip_frame_counts
        or horizon_motor_ticks <= 0
    ):
        raise ValueError("invalid reference curriculum episode identity")
    clip_seed = derive_named_seed(
        run_root,
        "train",
        episode_ordinal,
        vector_slot,
        "randomization.reference-clip",
    )
    clip_index = int.from_bytes(clip_seed[:8], "little") % len(clip_frame_counts)
    valid_start_count = clip_frame_counts[clip_index] - horizon_motor_ticks
    if valid_start_count <= 0:
        raise ValueError("curriculum horizon exceeds a selected clip")
    phase_seed = derive_named_seed(
        run_root,
        "train",
        episode_ordinal,
        vector_slot,
        "randomization.reference-phase",
    )
    start_frame = int.from_bytes(phase_seed[:8], "little") % valid_start_count
    return clip_index, start_frame, start_frame + horizon_motor_ticks


def _advance_contact_grace(
    previous_substeps: torch.Tensor,
    raw_contact: torch.Tensor,
    *,
    physics_substeps_per_motor_tick: int,
    grace_physics_substeps: int,
) -> tuple[torch.Tensor, torch.Tensor]:
    if physics_substeps_per_motor_tick <= 0 or grace_physics_substeps < 0:
        raise ValueError("invalid forbidden-contact grace cadence")
    accumulated = torch.where(
        raw_contact,
        previous_substeps + physics_substeps_per_motor_tick,
        torch.zeros_like(previous_substeps),
    )
    return accumulated, raw_contact & (accumulated > grace_physics_substeps)


def _soft_rom_excursion_cost_tensor(
    position: torch.Tensor,
    soft_minimum: torch.Tensor,
    soft_maximum: torch.Tensor,
    hard_minimum: torch.Tensor,
    hard_maximum: torch.Tensor,
) -> torch.Tensor:
    lower_span = soft_minimum - hard_minimum
    upper_span = hard_maximum - soft_maximum
    lower = torch.where(
        lower_span > 0,
        (soft_minimum - position) / torch.clamp(lower_span, min=1.0),
        torch.zeros_like(position),
    )
    upper = torch.where(
        upper_span > 0,
        (position - soft_maximum) / torch.clamp(upper_span, min=1.0),
        torch.zeros_like(position),
    )
    return torch.clamp(torch.maximum(lower, upper), min=0.0, max=1.0).amax(
        dim=-1
    )


def _canonical_pd_requested_effort_tensor(
    target_microradians: torch.Tensor,
    position_microradians: torch.Tensor,
    velocity_microradians_per_second: torch.Tensor,
    stiffness_q16: torch.Tensor,
    damping_q16: torch.Tensor,
) -> torch.Tensor:
    proportional = torch.round(
        stiffness_q16
        * (target_microradians - position_microradians)
        / 65_536.0
    )
    damping = torch.round(
        damping_q16 * velocity_microradians_per_second / 65_536.0
    )
    return proportional - damping


def _intersect_effort_limits_tensor(
    requested_effort_micronewton_metres: torch.Tensor,
    previous_effort_micronewton_metres: torch.Tensor,
    velocity_microradians_per_second: torch.Tensor,
    used_positive_work_microjoules: torch.Tensor,
    minimum_effort_micronewton_metres: torch.Tensor,
    maximum_effort_micronewton_metres: torch.Tensor,
    maximum_effort_rate_micronewton_metres_per_second: torch.Tensor,
    maximum_power_microwatts: torch.Tensor,
    maximum_positive_work_microjoules_per_motor_tick: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    values = (
        requested_effort_micronewton_metres,
        previous_effort_micronewton_metres,
        velocity_microradians_per_second,
        used_positive_work_microjoules,
        minimum_effort_micronewton_metres,
        maximum_effort_micronewton_metres,
        maximum_effort_rate_micronewton_metres_per_second,
        maximum_power_microwatts,
        maximum_positive_work_microjoules_per_motor_tick,
    )
    if any(value.dtype != torch.float64 for value in values):
        raise ValueError("canonical effort intersection requires float64 tensors")
    velocity_abs = torch.abs(velocity_microradians_per_second)
    nonzero_velocity = velocity_abs > 0.0
    unlimited = torch.full_like(velocity_abs, torch.inf)
    maximum_delta = torch.round(
        maximum_effort_rate_micronewton_metres_per_second
        / PHYSICS_SUBSTEPS_PER_SECOND
    )
    rate_minimum = previous_effort_micronewton_metres - maximum_delta
    rate_maximum = previous_effort_micronewton_metres + maximum_delta
    power_maximum = torch.where(
        nonzero_velocity,
        torch.floor(
            maximum_power_microwatts * MICRO_SCALE
            / torch.clamp(velocity_abs, min=1.0)
        ),
        unlimited,
    )
    remaining_work = (
        maximum_positive_work_microjoules_per_motor_tick
        - used_positive_work_microjoules
    )
    work_maximum = torch.where(
        nonzero_velocity,
        torch.floor(
            remaining_work
            * PHYSICS_SUBSTEPS_PER_SECOND
            * MICRO_SCALE
            / torch.clamp(velocity_abs, min=1.0)
        ),
        unlimited,
    )
    minimum = torch.maximum(
        torch.maximum(minimum_effort_micronewton_metres, rate_minimum),
        -power_maximum,
    )
    maximum = torch.minimum(
        torch.minimum(maximum_effort_micronewton_metres, rate_maximum),
        power_maximum,
    )
    maximum = torch.where(
        velocity_microradians_per_second > 0.0,
        torch.minimum(maximum, work_maximum),
        maximum,
    )
    minimum = torch.where(
        velocity_microradians_per_second < 0.0,
        torch.maximum(minimum, -work_maximum),
        minimum,
    )
    infeasible = (remaining_work < 0.0) | (minimum > maximum)
    effort = torch.minimum(
        torch.maximum(requested_effort_micronewton_metres, minimum), maximum
    )
    positive_power = torch.clamp(
        effort * velocity_microradians_per_second, min=0.0
    )
    charge = torch.ceil(
        positive_power
        / (PHYSICS_SUBSTEPS_PER_SECOND * MICRO_SCALE)
    )
    next_work = used_positive_work_microjoules + charge
    infeasible |= next_work > maximum_positive_work_microjoules_per_motor_tick
    return effort, next_work, infeasible


if ISAAC_LAB_AVAILABLE:

    @configclass
    class NextEngineReferenceDirectEnvCfg(DirectRLEnvCfg):
        decimation = 4
        episode_length_s = 2.0
        action_space = ACTION_CHANNELS
        observation_space = OBSERVATION_CHANNELS
        state_space = 0
        fixed_clip_id = "cmu104-start-right"
        fixed_start_frame = 0
        fixed_horizon_motor_ticks = 64
        eligible_clip_ids: tuple[str, ...] = ()
        phase_randomization = False
        rng_run_root_hex = ""
        sim = sim_utils.SimulationCfg(dt=1.0 / 240.0, render_interval=4)
        scene = InteractiveSceneCfg(num_envs=64, env_spacing=3.0, replicate_physics=True)
        asset = ArticulationCfg(
            prim_path="/World/envs/env_.*/Humanoid",
            spawn=sim_utils.UsdFileCfg(
                usd_path=os.environ.get("NEXTENGINE_HUMANOID_USD", ""),
                activate_contact_sensors=True,
                articulation_props=sim_utils.ArticulationRootPropertiesCfg(
                    enabled_self_collisions=True,
                    solver_position_iteration_count=8,
                    solver_velocity_iteration_count=4,
                ),
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
        contacts = ContactSensorCfg(
            prim_path="/World/envs/env_.*/Humanoid/Bodies/body_.*",
            update_period=0.0,
            history_length=1,
            track_air_time=False,
        )


    class NextEngineReferenceDirectEnv(DirectRLEnv):
        cfg: NextEngineReferenceDirectEnvCfg

        def __init__(
            self,
            cfg: NextEngineReferenceDirectEnvCfg,
            *,
            descriptor_path: str,
            profile_path: str,
            corpus_root: str,
            gate_report_path: str,
            **kwargs: Any,
        ) -> None:
            self.descriptor = json.loads(Path(descriptor_path).read_text(encoding="utf-8"))
            validate_biomechanics_descriptor(self.descriptor)
            self.reference_profile = ReferenceTrackerProfile.load(Path(profile_path))
            self.reference_corpus = ReferenceCorpus(
                self.reference_profile,
                Path(corpus_root),
                Path(gate_report_path),
            )
            clip_ids = tuple(cfg.eligible_clip_ids) or (cfg.fixed_clip_id,)
            if not clip_ids or len(clip_ids) != len(set(clip_ids)):
                raise ValueError("reference curriculum clip IDs must be non-empty and unique")
            self.reference_clips = tuple(
                self.reference_corpus.load_clip(clip_id) for clip_id in clip_ids
            )
            self.reference_clip = self.reference_clips[0]
            if any(clip.split != "train" for clip in self.reference_clips):
                raise ValueError("reference training clips must belong to the train split")
            if cfg.fixed_horizon_motor_ticks <= 0:
                raise ValueError("reference horizon must be positive")
            if cfg.phase_randomization:
                if len(cfg.rng_run_root_hex) != 64:
                    raise ValueError("phase-randomized curriculum requires a 256-bit run root")
                self._rng_run_root = bytes.fromhex(cfg.rng_run_root_hex)
            else:
                if len(self.reference_clips) != 1:
                    raise ValueError("multiple clips require deterministic phase randomization")
                if not 0 <= cfg.fixed_start_frame < self.reference_clip.frame_count - 1:
                    raise ValueError("fixed start frame is outside the reference clip")
                terminal_frame = cfg.fixed_start_frame + cfg.fixed_horizon_motor_ticks
                if terminal_frame >= self.reference_clip.frame_count:
                    raise ValueError("fixed horizon exceeds the reference clip")
                self._rng_run_root = b""
            self._episode_ordinal_by_env = [0] * cfg.scene.num_envs
            self._validate_channel_closure()
            limits = isaac_actuator_limits_from_descriptor(self.descriptor)
            actuator_cfg = cfg.asset.actuators["engine_effort"]
            actuator_cfg.effort_limit_sim = limits["effort_limit_sim"]
            actuator_cfg.velocity_limit_sim = limits["velocity_limit_sim"]
            cfg.episode_length_s = (cfg.fixed_horizon_motor_ticks + 1) / 60.0
            self._action = torch.zeros((cfg.scene.num_envs, ACTION_CHANNELS))
            self._applied_target = torch.zeros_like(self._action)
            self._previous_applied_target = torch.zeros_like(self._action)
            self._previous_effort = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.float64
            )
            self._applied_effort = torch.zeros_like(self._previous_effort)
            self._positive_work = torch.zeros_like(self._previous_effort)
            self._substep_hard_rom_violation = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self._substep_joint_safety_violation = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self._substep_velocity_violation = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self._substep_effort_envelope_violation = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self._substep_hard_rom_excess_by_action_channel = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.float64
            )
            self._substep_velocity_excess_by_action_channel = torch.zeros_like(
                self._substep_hard_rom_excess_by_action_channel
            )
            self._cursor = torch.full(
                (cfg.scene.num_envs,), cfg.fixed_start_frame, dtype=torch.int64
            )
            self._clip_index = torch.zeros(cfg.scene.num_envs, dtype=torch.int64)
            self._terminal_frame = torch.full(
                (cfg.scene.num_envs,),
                cfg.fixed_start_frame + cfg.fixed_horizon_motor_ticks,
                dtype=torch.int64,
            )
            self._episode_start_frame = self._cursor.clone()
            self._episode_clip_index = self._clip_index.clone()
            self._tracking_loss_ticks = torch.zeros(cfg.scene.num_envs, dtype=torch.int64)
            self._forbidden_contact_substeps = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self._forbidden_contact_grace_substeps = int(
                self.reference_profile.document["termination"]
                ["forbidden_contact_grace_physics_substeps"]
            )
            self._failure_terminal = torch.zeros(cfg.scene.num_envs, dtype=torch.bool)
            self._success_terminal = torch.zeros(cfg.scene.num_envs, dtype=torch.bool)
            self.last_step_failure = torch.zeros(cfg.scene.num_envs, dtype=torch.bool)
            self.last_step_success = torch.zeros(cfg.scene.num_envs, dtype=torch.bool)
            self.last_step_failure_tracking_lost = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_failure_hard_rom = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_failure_joint_safety = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_failure_joint_velocity = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_failure_effort_envelope = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_failure_forbidden_contact = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_failure_non_finite = torch.zeros(
                cfg.scene.num_envs, dtype=torch.bool
            )
            self.last_step_episode_start_frame = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self.last_step_episode_clip_index = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self.last_step_hard_rom_action_channel = torch.full(
                (cfg.scene.num_envs,), -1, dtype=torch.int64
            )
            self.last_step_hard_rom_excess_microradians = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self.last_step_hard_rom_excess_by_action_channel = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_velocity_action_channel = torch.full(
                (cfg.scene.num_envs,), -1, dtype=torch.int64
            )
            self.last_step_velocity_excess_microradians_per_second = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self.last_step_velocity_excess_by_action_channel = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_action_joint_position_microradians = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_pre_physics_action_joint_position_microradians = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_action_joint_velocity_microradians_per_second = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_pre_physics_action_joint_velocity_microradians_per_second = (
                torch.zeros(
                    (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
                )
            )
            self.last_step_applied_target_microradians = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_previous_applied_target_microradians = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_command_reference_target_microradians = torch.zeros(
                (cfg.scene.num_envs, ACTION_CHANNELS), dtype=torch.int64
            )
            self.last_step_reference_frame = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self.last_step_episode_elapsed_motor_ticks = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self.last_step_forbidden_contact_mask = torch.zeros(
                cfg.scene.num_envs, dtype=torch.int64
            )
            self._reward_component_ids = tuple(
                component["id"]
                for component in self.reference_profile.document["reward"]["components"]
            )
            reward_component_count = len(self._reward_component_ids)
            self._episode_reward_sum = torch.zeros(cfg.scene.num_envs)
            self._episode_component_sums = torch.zeros(
                (cfg.scene.num_envs, reward_component_count)
            )
            self.reward_components = torch.zeros(
                (cfg.scene.num_envs, reward_component_count)
            )
            self._canonical_cache: dict[str, torch.Tensor] | None = None
            self.last_raw_observation = torch.zeros(
                (cfg.scene.num_envs, OBSERVATION_CHANNELS), dtype=torch.int64
            )
            super().__init__(cfg, **kwargs)
            self._resolve_articulation_layout()
            self._load_reference_tensors()
            self._load_control_tensors()
            self._load_observation_scale()
            self._forbidden_contact_weights = torch.tensor(
                [1, 2, 4, 8, 16], dtype=torch.int64, device=self.device
            )
            self._reward_coefficients = torch.tensor(
                [
                    int(component["coefficient_q16"]) / 65_536.0
                    for component in self.reference_profile.document["reward"][
                        "components"
                    ]
                ],
                dtype=torch.float64,
                device=self.device,
            )
            for name in (
                "_action",
                "_applied_target",
                "_previous_applied_target",
                "_previous_effort",
                "_applied_effort",
                "_positive_work",
                "_substep_hard_rom_violation",
                "_substep_joint_safety_violation",
                "_substep_velocity_violation",
                "_substep_effort_envelope_violation",
                "_substep_hard_rom_excess_by_action_channel",
                "_substep_velocity_excess_by_action_channel",
                "_cursor",
                "_clip_index",
                "_terminal_frame",
                "_episode_start_frame",
                "_episode_clip_index",
                "_tracking_loss_ticks",
                "_forbidden_contact_substeps",
                "_failure_terminal",
                "_success_terminal",
                "last_step_failure",
                "last_step_success",
                "last_step_failure_tracking_lost",
                "last_step_failure_hard_rom",
                "last_step_failure_joint_safety",
                "last_step_failure_joint_velocity",
                "last_step_failure_effort_envelope",
                "last_step_failure_forbidden_contact",
                "last_step_failure_non_finite",
                "last_step_episode_start_frame",
                "last_step_episode_clip_index",
                "last_step_hard_rom_action_channel",
                "last_step_hard_rom_excess_microradians",
                "last_step_hard_rom_excess_by_action_channel",
                "last_step_velocity_action_channel",
                "last_step_velocity_excess_microradians_per_second",
                "last_step_velocity_excess_by_action_channel",
                "last_step_action_joint_position_microradians",
                "last_step_pre_physics_action_joint_position_microradians",
                "last_step_action_joint_velocity_microradians_per_second",
                "last_step_pre_physics_action_joint_velocity_microradians_per_second",
                "last_step_applied_target_microradians",
                "last_step_previous_applied_target_microradians",
                "last_step_command_reference_target_microradians",
                "last_step_reference_frame",
                "last_step_episode_elapsed_motor_ticks",
                "last_step_forbidden_contact_mask",
                "_episode_reward_sum",
                "_episode_component_sums",
                "reward_components",
                "last_raw_observation",
            ):
                setattr(self, name, getattr(self, name).to(self.device))

        def _validate_channel_closure(self) -> None:
            observation = self.reference_profile.document["observation"]
            action = self.reference_profile.document["action"]
            for clip in self.reference_clips:
                metadata = clip.metadata
                if (
                    metadata["joint_ids"] != observation["joint_state_order"]
                    or metadata["effector_ids"] != observation["reference_effector_order"]
                    or metadata["contact_ids"] != observation["contact_order"]
                    or self.descriptor["ordered_actuator_ids"]
                    != action["ordered_actuator_ids"]
                ):
                    raise ValueError("reference/descriptor channel order mismatch")

        def reset_episode_sequence(self, episode_ordinal: int = 0) -> None:
            if episode_ordinal < 0:
                raise ValueError("episode sequence ordinal must be non-negative")
            self._episode_ordinal_by_env = [episode_ordinal] * self.num_envs

        def _setup_scene(self) -> None:
            self.robot = Articulation(self.cfg.asset)
            self.contacts = ContactSensor(self.cfg.contacts)
            self.scene.articulations["humanoid"] = self.robot
            self.scene.sensors["contacts"] = self.contacts
            sim_utils.spawn_ground_plane("/World/ground", sim_utils.GroundPlaneCfg())
            self.scene.clone_environments(copy_from_source=False)

        def _resolve_articulation_layout(self) -> None:
            observation = self.reference_profile.document["observation"]
            action = self.reference_profile.document["action"]
            dof_names = [_prim(value) for value in observation["joint_state_order"]]
            action_joint_names = [
                _prim("joint." + value.removeprefix("actuator."))
                for value in action["ordered_actuator_ids"]
            ]
            self._dof_joint_ids, resolved_dof = self.robot.find_joints(
                dof_names, preserve_order=True
            )
            self._action_joint_ids, resolved_action = self.robot.find_joints(
                action_joint_names, preserve_order=True
            )
            if resolved_dof != dof_names or resolved_action != action_joint_names:
                raise RuntimeError("Isaac joint order does not close reference/action layouts")
            body_names = [_prim(body["body_id"]) for body in self.descriptor["bodies"]]
            self._body_ids, resolved_bodies = self.robot.find_bodies(
                body_names, preserve_order=True
            )
            if resolved_bodies != body_names:
                raise RuntimeError("Isaac body order does not close the descriptor")
            body_id_by_name = dict(zip(resolved_bodies, self._body_ids, strict=True))
            effector_by_id = {
                effector["effector_id"]: effector for effector in self.descriptor["effectors"]
            }
            effector_records = [
                effector_by_id[value]
                for value in observation["reference_effector_order"]
            ]
            self._effector_body_ids = [
                body_id_by_name[_prim(record["body_id"])] for record in effector_records
            ]
            self._effector_local_isaac = _engine_to_isaac_vector(
                torch.tensor(
                    [record["local_translation_micrometres"] for record in effector_records],
                    dtype=torch.float64,
                    device=self.device,
                )
                / 1_000_000.0
            ).to(torch.float32)
            contact_names = list(self.contacts.body_names)
            contact_index = {name: index for index, name in enumerate(contact_names)}
            required = (
                "body_left_ankle_roll",
                "body_right_ankle_roll",
                "body_left_elbow",
                "body_right_elbow",
                "body_left_knee",
                "body_right_knee",
            )
            if any(name not in contact_index for name in required):
                raise RuntimeError("Isaac contact sensor does not expose required bodies")
            self._contact_role_indices = torch.tensor(
                [contact_index[name] for name in required],
                dtype=torch.int64,
                device=self.device,
            )
            excluded = set(required)
            self._body_contact_indices = torch.tensor(
                [index for index, name in enumerate(contact_names) if name not in excluded],
                dtype=torch.int64,
                device=self.device,
            )

        def _load_reference_tensors(self) -> None:
            self._reference: dict[str, torch.Tensor] = {}
            maximum_frames = max(clip.frame_count for clip in self.reference_clips)
            for name in self.reference_clip.arrays:
                padded: list[np.ndarray] = []
                for clip in self.reference_clips:
                    array = np.array(clip.arrays[name], copy=True)
                    if array.dtype == np.uint16 or array.dtype == np.uint8:
                        array = array.astype(np.int64)
                    missing = maximum_frames - clip.frame_count
                    if missing:
                        array = np.concatenate(
                            (array, np.repeat(array[-1:], missing, axis=0)), axis=0
                        )
                    padded.append(array)
                self._reference[name] = torch.from_numpy(np.stack(padded)).to(
                    self.device
                )
            self._reference_lengths = torch.tensor(
                [clip.frame_count for clip in self.reference_clips],
                dtype=torch.int64,
                device=self.device,
            )

        def _reference_at(
            self,
            name: str,
            frame: torch.Tensor | None = None,
            env_ids: torch.Tensor | None = None,
        ) -> torch.Tensor:
            clip_index = self._clip_index if env_ids is None else self._clip_index[env_ids]
            if frame is None:
                selected_frame = self._cursor if env_ids is None else self._cursor[env_ids]
            else:
                selected_frame = frame
            return self._reference[name][clip_index, selected_frame]

        def _load_control_tensors(self) -> None:
            action = self.reference_profile.document["action"]
            actuator_by_id = {
                actuator["actuator_id"]: actuator for actuator in self.descriptor["actuators"]
            }
            joint_by_dof = {
                int(joint["dof_ordinal"]): joint for joint in self.descriptor["joints"]
            }
            records = [actuator_by_id[value] for value in action["ordered_actuator_ids"]]
            mapping = action["reference_joint_dof_ordinal_by_action_channel"]
            self._action_to_dof = torch.tensor(mapping, dtype=torch.int64, device=self.device)
            self._residual_scale = self._tensor(records, "residual_scale_microradians")
            self._target_delta = self._tensor_range_max(
                records, "target_delta_microradians_per_motor_tick"
            )
            self._stiffness_q16 = self._tensor(records, "stiffness_q16")
            self._damping_q16 = self._tensor(records, "damping_q16")
            self._minimum_effort = self._tensor_range_min(
                records, "effort_micronewton_metres"
            )
            self._maximum_effort = self._tensor_range_max(
                records, "effort_micronewton_metres"
            )
            self._maximum_effort_rate = self._tensor(
                records, "maximum_effort_rate_micronewton_metres_per_second"
            )
            self._maximum_power = self._tensor(records, "maximum_power_microwatts")
            self._maximum_positive_work = self._tensor(
                records, "maximum_positive_work_microjoules_per_motor_tick"
            )
            self._maximum_velocity_action = torch.tensor(
                [
                    joint_by_dof[dof]["maximum_velocity_microradians_per_second"]
                    for dof in mapping
                ],
                dtype=torch.float64,
                device=self.device,
            )
            self._hard_minimum = torch.tensor(
                [joint_by_dof[dof]["hard_limit_microradians"][0] for dof in mapping],
                dtype=torch.float64,
                device=self.device,
            )
            self._hard_maximum = torch.tensor(
                [joint_by_dof[dof]["hard_limit_microradians"][1] for dof in mapping],
                dtype=torch.float64,
                device=self.device,
            )
            self._soft_minimum = torch.tensor(
                [joint_by_dof[dof]["soft_limit_microradians"][0] for dof in mapping],
                dtype=torch.float64,
                device=self.device,
            )
            self._soft_maximum = torch.tensor(
                [joint_by_dof[dof]["soft_limit_microradians"][1] for dof in mapping],
                dtype=torch.float64,
                device=self.device,
            )
            self._soft_span_dof = torch.tensor(
                [
                    joint_by_dof[dof]["soft_limit_microradians"][1]
                    - joint_by_dof[dof]["soft_limit_microradians"][0]
                    for dof in range(ACTION_CHANNELS)
                ],
                dtype=torch.float64,
                device=self.device,
            )
            self._maximum_velocity_dof = torch.tensor(
                [
                    joint_by_dof[dof]["maximum_velocity_microradians_per_second"]
                    for dof in range(ACTION_CHANNELS)
                ],
                dtype=torch.float64,
                device=self.device,
            )
            self._body_mass = torch.tensor(
                [body["mass_microkilograms"] for body in self.descriptor["bodies"]],
                dtype=torch.float64,
                device=self.device,
            )
            self._body_mass /= torch.sum(self._body_mass)

        def _tensor(self, records: list[dict[str, Any]], name: str) -> torch.Tensor:
            return torch.tensor(
                [record[name] for record in records],
                dtype=torch.float64,
                device=self.device,
            )

        def _tensor_range_max(
            self, records: list[dict[str, Any]], name: str
        ) -> torch.Tensor:
            return torch.tensor(
                [max(abs(record[name][0]), abs(record[name][1])) for record in records],
                dtype=torch.float64,
                device=self.device,
            )

        def _tensor_range_min(
            self, records: list[dict[str, Any]], name: str
        ) -> torch.Tensor:
            return torch.tensor(
                [record[name][0] for record in records],
                dtype=torch.float64,
                device=self.device,
            )

        def _load_observation_scale(self) -> None:
            joint_position_scale = torch.maximum(
                torch.abs(self._soft_minimum), torch.abs(self._soft_maximum)
            )
            joint_position_scale_dof = torch.empty_like(joint_position_scale)
            joint_position_scale_dof[self._action_to_dof] = joint_position_scale
            scales: list[torch.Tensor] = [
                torch.full((4,), Q1_30, device=self.device),
                torch.full((3,), 2_000_000, device=self.device),
                torch.full((3,), 2_000_000, device=self.device),
                joint_position_scale_dof,
                self._maximum_velocity_dof,
                joint_position_scale_dof,
                torch.ones(7, device=self.device),
                torch.full((1,), 65_535, device=self.device),
            ]
            for _ in REFERENCE_OFFSETS:
                scales.extend(
                    (
                        torch.full((3,), 1_000_000, device=self.device),
                        torch.full((4,), Q1_30, device=self.device),
                        torch.full((3,), 2_000_000, device=self.device),
                        torch.full((3,), 2_000_000, device=self.device),
                        torch.full((3,), 1_000_000, device=self.device),
                        joint_position_scale_dof,
                        self._maximum_velocity_dof,
                        torch.full((18,), 1_000_000, device=self.device),
                        torch.ones(7, device=self.device),
                    )
                )
            self._observation_scale = torch.cat(scales).to(torch.float32)
            if self._observation_scale.shape != (OBSERVATION_CHANNELS,):
                raise RuntimeError("reference observation scale width mismatch")

        def _pre_physics_step(self, actions: torch.Tensor) -> None:
            self.extras.pop("log", None)
            self._canonical_cache = None
            if actions.shape != (self.num_envs, ACTION_CHANNELS):
                raise ValueError("reference action batch has the wrong shape")
            require_finite_tensor("reference_actions", actions, asynchronous=True)
            self.last_step_pre_physics_action_joint_position_microradians.copy_(
                torch.round(
                    self.robot.data.joint_pos[:, self._action_joint_ids]
                    * 1_000_000.0
                ).to(torch.int64)
            )
            self.last_step_pre_physics_action_joint_velocity_microradians_per_second.copy_(
                torch.round(
                    self.robot.data.joint_vel[:, self._action_joint_ids]
                    * 1_000_000.0
                ).to(torch.int64)
            )
            self._positive_work.zero_()
            self._substep_hard_rom_violation.zero_()
            self._substep_joint_safety_violation.zero_()
            self._substep_velocity_violation.zero_()
            self._substep_effort_envelope_violation.zero_()
            self._substep_hard_rom_excess_by_action_channel.zero_()
            self._substep_velocity_excess_by_action_channel.zero_()
            self._action.copy_(torch.clamp(actions, -1.0, 1.0))
            current_reference_dof = self._reference_at("joint_position_urad")
            current_reference_action = current_reference_dof[:, self._action_to_dof]
            self.last_step_command_reference_target_microradians.copy_(
                current_reference_action
            )
            residual = torch.round(
                self._action.to(torch.float64) * self._residual_scale
            )
            candidate = current_reference_action.to(torch.float64) + residual
            candidate = torch.minimum(
                torch.maximum(candidate, self._soft_minimum), self._soft_maximum
            )
            self._previous_applied_target.copy_(self._applied_target)
            lower = self._applied_target.to(torch.float64) - self._target_delta
            upper = self._applied_target.to(torch.float64) + self._target_delta
            applied = torch.minimum(torch.maximum(candidate, lower), upper)
            self._applied_target.copy_(applied.to(torch.float32))
            self._cursor.add_(1)

        def _apply_action(self) -> None:
            position = torch.round(
                self.robot.data.joint_pos[:, self._action_joint_ids].to(torch.float64)
                * MICRO_SCALE
            )
            velocity = torch.round(
                self.robot.data.joint_vel[:, self._action_joint_ids].to(torch.float64)
                * MICRO_SCALE
            )
            hard_rom_excess = torch.maximum(
                torch.maximum(
                    self._hard_minimum - position, torch.zeros_like(position)
                ),
                torch.maximum(
                    position - self._hard_maximum, torch.zeros_like(position)
                ),
            )
            self._substep_hard_rom_excess_by_action_channel.copy_(
                torch.maximum(
                    self._substep_hard_rom_excess_by_action_channel,
                    hard_rom_excess,
                )
            )
            hard_rom_violation = torch.any(
                hard_rom_excess
                > OBSERVED_HARD_ROM_TOLERANCE_MICRORADIANS,
                dim=-1,
            )
            velocity_excess = torch.clamp(
                torch.abs(velocity) - self._maximum_velocity_action,
                min=0.0,
            )
            self._substep_velocity_excess_by_action_channel.copy_(
                torch.maximum(
                    self._substep_velocity_excess_by_action_channel,
                    velocity_excess,
                )
            )
            velocity_violation = torch.any(velocity_excess > 0.0, dim=-1)
            requested = _canonical_pd_requested_effort_tensor(
                self._applied_target.to(torch.float64),
                position,
                velocity,
                self._stiffness_q16,
                self._damping_q16,
            )
            effort, next_work, infeasible_channel = _intersect_effort_limits_tensor(
                requested,
                self._previous_effort,
                velocity,
                self._positive_work,
                self._minimum_effort,
                self._maximum_effort,
                self._maximum_effort_rate,
                self._maximum_power,
                self._maximum_positive_work,
            )
            joint_safety_violation = velocity_violation | torch.any(
                infeasible_channel, dim=-1
            )
            blocked = (
                self._substep_hard_rom_violation
                | self._substep_joint_safety_violation
                | hard_rom_violation
                | joint_safety_violation
            )
            publish = ~blocked
            published_effort = torch.where(publish[:, None], effort, 0.0)
            self._previous_effort.copy_(
                torch.where(publish[:, None], effort, self._previous_effort)
            )
            self._positive_work.copy_(
                torch.where(publish[:, None], next_work, self._positive_work)
            )
            self._applied_effort.copy_(published_effort)
            self._substep_hard_rom_violation |= hard_rom_violation
            self._substep_joint_safety_violation |= joint_safety_violation
            self._substep_velocity_violation |= velocity_violation
            self._substep_effort_envelope_violation |= torch.any(
                infeasible_channel, dim=-1
            )
            self.robot.set_joint_effort_target(
                published_effort.to(torch.float32) / MICRO_SCALE,
                joint_ids=self._action_joint_ids,
            )

        def _canonical_current(self) -> dict[str, torch.Tensor]:
            if self._canonical_cache is not None:
                return self._canonical_cache
            data = self.robot.data
            for name, value in (
                ("root_pos_w", data.root_pos_w),
                ("root_quat_w", data.root_quat_w),
                ("root_lin_vel_w", data.root_lin_vel_w),
                ("root_ang_vel_w", data.root_ang_vel_w),
                ("joint_pos", data.joint_pos),
                ("joint_vel", data.joint_vel),
            ):
                require_finite_tensor(name, value, asynchronous=True)
            root_world_isaac = data.root_pos_w - self.scene.env_origins
            root_position = engine_vector_from_isaac_tensor(root_world_isaac)
            root_quaternion = _normalized_xyzw(
                engine_quaternion_xyzw_from_isaac_wxyz_tensor(data.root_quat_w)
            )
            linear_world = engine_vector_from_isaac_tensor(data.root_lin_vel_w)
            angular_world = engine_vector_from_isaac_tensor(data.root_ang_vel_w)
            joint_position = data.joint_pos[:, self._dof_joint_ids]
            joint_velocity = data.joint_vel[:, self._dof_joint_ids]
            body_com_isaac = data.body_com_pos_w[:, self._body_ids] - self.scene.env_origins[:, None]
            center_of_mass_isaac = torch.sum(
                body_com_isaac.to(torch.float64) * self._body_mass[None, :, None],
                dim=1,
            ).to(torch.float32)
            center_of_mass = engine_vector_from_isaac_tensor(center_of_mass_isaac)
            effector_body_position = data.body_pos_w[:, self._effector_body_ids]
            effector_body_rotation = data.body_quat_w[:, self._effector_body_ids]
            local = self._effector_local_isaac[None].expand(self.num_envs, -1, -1)
            effector_isaac = effector_body_position + quat_apply(effector_body_rotation, local)
            effector_isaac -= self.scene.env_origins[:, None]
            effectors = engine_vector_from_isaac_tensor(effector_isaac)
            contact_force = torch.linalg.vector_norm(self.contacts.data.net_forces_w, dim=-1)
            contacts_by_body = contact_force > 1.0
            role_contacts = contacts_by_body[:, self._contact_role_indices]
            body_contact = torch.any(
                contacts_by_body[:, self._body_contact_indices], dim=-1, keepdim=True
            )
            contacts = torch.cat((role_contacts, body_contact), dim=-1)
            self._canonical_cache = {
                "root_position_um": _round_int64(root_position * 1_000_000.0),
                "root_quaternion_q1_30": _round_int64(root_quaternion * Q1_30),
                "root_linear_velocity_um_s": _round_int64(linear_world * 1_000_000.0),
                "root_angular_velocity_urad_s": _round_int64(angular_world * 1_000_000.0),
                "joint_position_urad": _round_int64(joint_position * 1_000_000.0),
                "joint_velocity_urad_s": _round_int64(joint_velocity * 1_000_000.0),
                "center_of_mass_um": _round_int64(center_of_mass * 1_000_000.0),
                "effector_position_um": _round_int64(effectors * 1_000_000.0),
                "contacts": contacts.to(torch.int64),
                "root_quaternion": root_quaternion,
            }
            return self._canonical_cache

        def _get_observations(self) -> dict[str, torch.Tensor]:
            current = self._canonical_current()
            quaternion = current["root_quaternion"]
            previous_target_dof = torch.empty_like(self._previous_applied_target)
            previous_target_dof[:, self._action_to_dof] = self._previous_applied_target
            values: list[torch.Tensor] = [
                current["root_quaternion_q1_30"],
                _round_int64(
                    _rotate_inverse_xyzw(
                        current["root_linear_velocity_um_s"].to(torch.float64), quaternion
                    )
                ),
                _round_int64(
                    _rotate_inverse_xyzw(
                        current["root_angular_velocity_urad_s"].to(torch.float64), quaternion
                    )
                ),
                current["joint_position_urad"],
                current["joint_velocity_urad_s"],
                _round_int64(previous_target_dof),
                current["contacts"],
                self._reference_at("phase_u16")[:, None].to(torch.int64),
            ]
            for offset in REFERENCE_OFFSETS:
                frame = torch.minimum(
                    self._cursor + offset,
                    self._reference_lengths[self._clip_index] - 1,
                )
                reference_quaternion = _normalized_xyzw(
                    self._reference_at("root_quaternion_q1_30", frame).to(torch.float64)
                    / Q1_30
                )
                relative_rotation = _quaternion_multiply_xyzw(
                    _quaternion_conjugate_xyzw(quaternion), reference_quaternion
                )
                yaw = self._reference_at("root_yaw_velocity_urad_s", frame)
                reference_angular = torch.stack(
                    (torch.zeros_like(yaw), yaw, torch.zeros_like(yaw)), dim=-1
                )
                relative_effectors = self._reference_at("effector_position_um", frame) - current[
                    "root_position_um"
                ][:, None]
                values.extend(
                    (
                        _round_int64(
                            _rotate_inverse_xyzw(
                                (
                                    self._reference_at("root_position_um", frame)
                                    - current["root_position_um"]
                                ).to(torch.float64),
                                quaternion,
                            )
                        ),
                        _round_int64(relative_rotation * Q1_30),
                        _round_int64(
                            _rotate_inverse_xyzw(
                                self._reference_at("root_linear_velocity_um_s", frame).to(
                                    torch.float64
                                ),
                                quaternion,
                            )
                        ),
                        _round_int64(
                            _rotate_inverse_xyzw(
                                reference_angular.to(torch.float64), quaternion
                            )
                        ),
                        _round_int64(
                            _rotate_inverse_xyzw(
                                (
                                    self._reference_at("center_of_mass_um", frame)
                                    - current["root_position_um"]
                                ).to(torch.float64),
                                quaternion,
                            )
                        ),
                        self._reference_at("joint_position_urad", frame),
                        self._reference_at("joint_velocity_urad_s", frame),
                        _round_int64(
                            _rotate_inverse_xyzw(
                                relative_effectors.to(torch.float64),
                                quaternion[:, None].expand(-1, 6, -1),
                            )
                        ).reshape(self.num_envs, 18),
                        self._reference_at("contacts", frame).to(torch.int64),
                    )
                )
            raw = torch.cat(values, dim=-1)
            if raw.shape != (self.num_envs, OBSERVATION_CHANNELS):
                raise RuntimeError("reference observation width mismatch")
            self.last_raw_observation.copy_(raw)
            policy = raw.to(torch.float32) / self._observation_scale
            require_finite_tensor(
                "reference_policy_observation", policy, asynchronous=True
            )
            return {"policy": policy}

        def _get_dones(self) -> tuple[torch.Tensor, torch.Tensor]:
            current = self._canonical_current()
            frame = torch.minimum(
                self._cursor, self._reference_lengths[self._clip_index] - 1
            )
            reference_position = self._reference_at("root_position_um", frame)
            position_error = torch.linalg.vector_norm(
                (current["root_position_um"] - reference_position).to(torch.float64), dim=-1
            )
            current_quaternion = current["root_quaternion_q1_30"].to(torch.float64) / Q1_30
            reference_quaternion = (
                self._reference_at("root_quaternion_q1_30", frame).to(torch.float64)
                / Q1_30
            )
            orientation_dot = torch.abs(
                torch.sum(
                    _normalized_xyzw(current_quaternion)
                    * _normalized_xyzw(reference_quaternion),
                    dim=-1,
                )
            )
            lost = (position_error > 750_000.0) | (
                orientation_dot < 929_887_697.0 / Q1_30
            )
            self._tracking_loss_ticks.copy_(
                torch.where(lost, self._tracking_loss_ticks + 1, 0)
            )
            action_position = current["joint_position_urad"][
                :, self._action_to_dof
            ].to(torch.float64)
            action_velocity = current["joint_velocity_urad_s"][
                :, self._action_to_dof
            ].to(torch.float64)
            hard_rom_excess = torch.maximum(
                torch.maximum(self._hard_minimum - action_position, torch.zeros_like(action_position)),
                torch.maximum(action_position - self._hard_maximum, torch.zeros_like(action_position)),
            )
            hard_rom_excess = torch.maximum(
                hard_rom_excess,
                self._substep_hard_rom_excess_by_action_channel,
            )
            maximum_hard_rom_excess, hard_rom_channel = torch.max(
                hard_rom_excess, dim=-1
            )
            hard_rom = (
                maximum_hard_rom_excess
                > OBSERVED_HARD_ROM_TOLERANCE_MICRORADIANS
            ) | self._substep_hard_rom_violation
            velocity_excess = torch.maximum(
                torch.clamp(
                    torch.abs(action_velocity) - self._maximum_velocity_action,
                    min=0.0,
                ),
                self._substep_velocity_excess_by_action_channel,
            )
            maximum_velocity_excess, velocity_channel = torch.max(
                velocity_excess, dim=-1
            )
            velocity_violation = (
                maximum_velocity_excess > 0.0
            ) | self._substep_velocity_violation
            effort_envelope_violation = (
                self._substep_effort_envelope_violation
            )
            joint_safety = (
                velocity_violation
                | effort_envelope_violation
                | self._substep_joint_safety_violation
            )
            forbidden_contact_raw = torch.any(
                current["contacts"][:, 2:] != 0, dim=-1
            )
            forbidden_contact_mask = torch.sum(
                (current["contacts"][:, 2:] != 0).to(torch.int64)
                * self._forbidden_contact_weights[None],
                dim=-1,
            )
            forbidden_contact_substeps, forbidden_contact = _advance_contact_grace(
                self._forbidden_contact_substeps,
                forbidden_contact_raw,
                physics_substeps_per_motor_tick=self.cfg.decimation,
                grace_physics_substeps=self._forbidden_contact_grace_substeps,
            )
            self._forbidden_contact_substeps.copy_(forbidden_contact_substeps)
            non_finite = ~torch.isfinite(self.robot.data.root_state_w).all(dim=-1) | ~torch.isfinite(
                self.robot.data.joint_pos
            ).all(dim=-1)
            tracking_lost = self._tracking_loss_ticks >= 4
            self._failure_terminal.copy_(
                tracking_lost
                | hard_rom
                | joint_safety
                | forbidden_contact
                | non_finite
            )
            self._success_terminal.copy_(
                (self._cursor >= self._terminal_frame) & ~self._failure_terminal
            )
            self.last_step_success.copy_(self._success_terminal)
            self.last_step_failure.copy_(self._failure_terminal)
            self.last_step_failure_tracking_lost.copy_(tracking_lost)
            self.last_step_failure_hard_rom.copy_(hard_rom)
            self.last_step_failure_joint_safety.copy_(joint_safety)
            self.last_step_failure_joint_velocity.copy_(velocity_violation)
            self.last_step_failure_effort_envelope.copy_(
                effort_envelope_violation
            )
            self.last_step_failure_forbidden_contact.copy_(forbidden_contact)
            self.last_step_failure_non_finite.copy_(non_finite)
            self.last_step_episode_start_frame.copy_(self._episode_start_frame)
            self.last_step_episode_clip_index.copy_(self._episode_clip_index)
            self.last_step_hard_rom_action_channel.copy_(
                torch.where(
                    hard_rom,
                    hard_rom_channel,
                    torch.full_like(hard_rom_channel, -1),
                )
            )
            self.last_step_hard_rom_excess_microradians.copy_(
                torch.round(maximum_hard_rom_excess).to(torch.int64)
            )
            self.last_step_hard_rom_excess_by_action_channel.copy_(
                torch.round(hard_rom_excess).to(torch.int64)
            )
            self.last_step_velocity_action_channel.copy_(
                torch.where(
                    velocity_violation,
                    velocity_channel,
                    torch.full_like(velocity_channel, -1),
                )
            )
            self.last_step_velocity_excess_microradians_per_second.copy_(
                torch.round(maximum_velocity_excess).to(torch.int64)
            )
            self.last_step_velocity_excess_by_action_channel.copy_(
                torch.round(velocity_excess).to(torch.int64)
            )
            self.last_step_action_joint_position_microradians.copy_(
                torch.round(action_position).to(torch.int64)
            )
            self.last_step_action_joint_velocity_microradians_per_second.copy_(
                torch.round(action_velocity).to(torch.int64)
            )
            self.last_step_applied_target_microradians.copy_(
                torch.round(self._applied_target).to(torch.int64)
            )
            self.last_step_previous_applied_target_microradians.copy_(
                torch.round(self._previous_applied_target).to(torch.int64)
            )
            self.last_step_reference_frame.copy_(frame)
            self.last_step_episode_elapsed_motor_ticks.copy_(
                frame - self._episode_start_frame
            )
            self.last_step_forbidden_contact_mask.copy_(forbidden_contact_mask)
            terminated = self._success_terminal | self._failure_terminal
            timed_out = self.episode_length_buf >= self.max_episode_length - 1
            return terminated, timed_out & ~terminated

        def _get_rewards(self) -> torch.Tensor:
            current = self._canonical_current()
            frame = torch.minimum(
                self._cursor, self._reference_lengths[self._clip_index] - 1
            )
            reference_quaternion = _normalized_xyzw(
                self._reference_at("root_quaternion_q1_30", frame).to(torch.float64)
                / Q1_30
            )
            current_quaternion = _normalized_xyzw(
                current["root_quaternion_q1_30"].to(torch.float64) / Q1_30
            )
            orientation = torch.abs(
                torch.sum(current_quaternion * reference_quaternion, dim=-1)
            ).square()
            root_height = self._similarity(
                torch.abs(
                    current["root_position_um"][:, 1]
                    - self._reference_at("root_position_um", frame)[:, 1]
                ),
                250_000.0,
            )
            root_linear = self._similarity(
                torch.linalg.vector_norm(
                    (
                        current["root_linear_velocity_um_s"]
                        - self._reference_at("root_linear_velocity_um_s", frame)
                    ).to(torch.float64),
                    dim=-1,
                ),
                2_000_000.0,
            )
            reference_yaw = self._reference_at("root_yaw_velocity_urad_s", frame)
            reference_angular = torch.stack(
                (torch.zeros_like(reference_yaw), reference_yaw, torch.zeros_like(reference_yaw)),
                dim=-1,
            )
            root_angular = self._similarity(
                torch.linalg.vector_norm(
                    (current["root_angular_velocity_urad_s"] - reference_angular).to(
                        torch.float64
                    ),
                    dim=-1,
                ),
                2_000_000.0,
            )
            joint_pose = 1.0 - torch.clamp(
                torch.mean(
                    torch.abs(
                        current["joint_position_urad"].to(torch.float64)
                        - self._reference_at("joint_position_urad", frame).to(torch.float64)
                    )
                    / self._soft_span_dof,
                    dim=-1,
                ),
                max=1.0,
            )
            joint_velocity = 1.0 - torch.clamp(
                torch.mean(
                    torch.abs(
                        current["joint_velocity_urad_s"].to(torch.float64)
                        - self._reference_at("joint_velocity_urad_s", frame).to(torch.float64)
                    )
                    / self._maximum_velocity_dof,
                    dim=-1,
                ),
                max=1.0,
            )
            center_of_mass = self._similarity(
                torch.linalg.vector_norm(
                    (
                        current["center_of_mass_um"]
                        - self._reference_at("center_of_mass_um", frame)
                    ).to(torch.float64),
                    dim=-1,
                ),
                250_000.0,
            )
            effectors = self._similarity(
                torch.mean(
                    torch.linalg.vector_norm(
                        (
                            current["effector_position_um"]
                            - self._reference_at("effector_position_um", frame)
                        ).to(torch.float64),
                        dim=-1,
                    ),
                    dim=-1,
                ),
                300_000.0,
            )
            contact_match = torch.mean(
                (
                    current["contacts"] == self._reference_at("contacts", frame)
                ).to(torch.float64),
                dim=-1,
            )
            foot_velocity = engine_vector_from_isaac_tensor(
                self.robot.data.body_lin_vel_w[
                    :, [self._effector_body_ids[0], self._effector_body_ids[3]], :
                ]
            )
            planar_speed = torch.linalg.vector_norm(
                foot_velocity[..., [0, 2]].to(torch.float64), dim=-1
            ) * 1_000_000.0
            active_sole = current["contacts"][:, :2].to(torch.float64)
            sole_count = torch.sum(active_sole, dim=-1)
            sole_slip = torch.where(
                sole_count > 0,
                torch.sum(planar_speed * active_sole, dim=-1) / torch.clamp(sole_count, min=1),
                0.0,
            )
            sole_slip = torch.clamp(sole_slip / 2_000_000.0, max=1.0)
            effort_cost = torch.clamp(
                torch.mean(
                    torch.abs(self._applied_effort.to(torch.float64)) / self._maximum_effort,
                    dim=-1,
                ),
                max=1.0,
            )
            target_rate = torch.clamp(
                torch.mean(
                    torch.abs(
                        self._applied_target.to(torch.float64)
                        - self._previous_applied_target.to(torch.float64)
                    )
                    / self._target_delta,
                    dim=-1,
                ),
                max=1.0,
            )
            action_position = current["joint_position_urad"][
                :, self._action_to_dof
            ].to(torch.float64)
            component_values = {
                "reward.reference-root-orientation": orientation,
                "reward.reference-root-height": root_height,
                "reward.reference-root-linear-velocity": root_linear,
                "reward.reference-root-angular-velocity": root_angular,
                "reward.reference-joint-pose": joint_pose,
                "reward.reference-joint-velocity": joint_velocity,
                "reward.reference-center-of-mass": center_of_mass,
                "reward.reference-effectors": effectors,
                "reward.reference-contacts": contact_match,
                "reward.contacting-sole-slip-cost": sole_slip,
                "reward.normalized-applied-effort-cost": effort_cost,
                "reward.applied-target-rate-cost": target_rate,
                "reward.soft-rom-excursion-cost": _soft_rom_excursion_cost_tensor(
                    action_position,
                    self._soft_minimum,
                    self._soft_maximum,
                    self._hard_minimum,
                    self._hard_maximum,
                ),
                "reward.predictive-rom-excursion-cost": _soft_rom_excursion_cost_tensor(
                    action_position
                    + current["joint_velocity_urad_s"][:, self._action_to_dof].to(
                        torch.float64
                    )
                    / 60.0,
                    self._soft_minimum,
                    self._soft_maximum,
                    self._hard_minimum,
                    self._hard_maximum,
                ),
                "reward.terminal-failure": self._failure_terminal.to(torch.float64),
            }
            components = torch.stack(
                tuple(component_values[value] for value in self._reward_component_ids),
                dim=-1,
            )
            reward = torch.sum(
                components * self._reward_coefficients, dim=-1
            ).to(torch.float32)
            self.reward_components.copy_(components.to(torch.float32))
            self._episode_reward_sum.add_(reward)
            self._episode_component_sums.add_(components.to(torch.float32))
            return reward

        @staticmethod
        def _similarity(error: torch.Tensor, normalization: float) -> torch.Tensor:
            return 1.0 - torch.clamp(error.to(torch.float64) / normalization, min=0.0, max=1.0)

        def _reset_idx(self, env_ids: torch.Tensor | None) -> None:
            self._canonical_cache = None
            if env_ids is None:
                env_ids = self.robot._ALL_INDICES
            completed = self.episode_length_buf[env_ids] > 0
            if torch.any(completed):
                completed_ids = env_ids[completed]
                length = self.episode_length_buf[completed_ids].to(torch.float32)
                self.extras["log"] = {
                    "Episode/return": self._episode_reward_sum[completed_ids],
                    "Episode/length": length,
                    "Episode/reference_complete": self._success_terminal[completed_ids].to(
                        torch.float32
                    ),
                    "Episode/failure": self._failure_terminal[completed_ids].to(torch.float32),
                }
            super()._reset_idx(env_ids)
            frame_values: list[int] = []
            clip_values: list[int] = []
            terminal_values: list[int] = []
            if self.cfg.phase_randomization:
                for env_id in env_ids.detach().cpu().tolist():
                    episode_ordinal = self._episode_ordinal_by_env[env_id]
                    clip_index, frame_value, terminal_value = (
                        _select_curriculum_episode(
                            run_root=self._rng_run_root,
                            episode_ordinal=episode_ordinal,
                            vector_slot=env_id,
                            clip_frame_counts=tuple(
                                clip.frame_count for clip in self.reference_clips
                            ),
                            horizon_motor_ticks=self.cfg.fixed_horizon_motor_ticks,
                        )
                    )
                    clip_values.append(clip_index)
                    frame_values.append(frame_value)
                    terminal_values.append(terminal_value)
                    self._episode_ordinal_by_env[env_id] += 1
            else:
                clip_values = [0] * len(env_ids)
                frame_values = [self.cfg.fixed_start_frame] * len(env_ids)
                terminal_values = [
                    self.cfg.fixed_start_frame + self.cfg.fixed_horizon_motor_ticks
                ] * len(env_ids)
            frame = torch.tensor(frame_values, dtype=torch.int64, device=self.device)
            self._clip_index[env_ids] = torch.tensor(
                clip_values, dtype=torch.int64, device=self.device
            )
            self._terminal_frame[env_ids] = torch.tensor(
                terminal_values, dtype=torch.int64, device=self.device
            )
            self._episode_start_frame[env_ids] = frame
            self._episode_clip_index[env_ids] = self._clip_index[env_ids]
            root_position_engine = (
                self._reference_at("root_position_um", frame, env_ids).to(torch.float32)
                / 1_000_000.0
            )
            root_position = _engine_to_isaac_vector(root_position_engine)
            root_position += self.scene.env_origins[env_ids]
            root_quaternion_engine = (
                self._reference_at("root_quaternion_q1_30", frame, env_ids).to(
                    torch.float32
                )
                / Q1_30
            )
            root_quaternion = _engine_xyzw_to_isaac_wxyz(root_quaternion_engine)
            root_linear_engine = (
                self._reference_at("root_linear_velocity_um_s", frame, env_ids).to(
                    torch.float32
                )
                / 1_000_000.0
            )
            root_linear = _engine_to_isaac_vector(root_linear_engine)
            yaw = (
                self._reference_at("root_yaw_velocity_urad_s", frame, env_ids).to(
                    torch.float32
                )
                / 1_000_000.0
            )
            root_angular_engine = torch.stack(
                (torch.zeros_like(yaw), yaw, torch.zeros_like(yaw)), dim=-1
            )
            root_angular = _engine_to_isaac_vector(root_angular_engine)
            root_state = torch.cat(
                (root_position, root_quaternion, root_linear, root_angular), dim=-1
            )
            joint_position = (
                self._reference_at("joint_position_urad", frame, env_ids).to(
                    torch.float32
                )
                / 1_000_000.0
            )
            joint_velocity = (
                self._reference_at("joint_velocity_urad_s", frame, env_ids).to(
                    torch.float32
                )
                / 1_000_000.0
            )
            self.robot.write_root_state_to_sim(root_state, env_ids)
            self.robot.write_joint_state_to_sim(
                joint_position,
                joint_velocity,
                joint_ids=self._dof_joint_ids,
                env_ids=env_ids,
            )
            self.robot.set_joint_effort_target(
                torch.zeros((len(env_ids), ACTION_CHANNELS), device=self.device),
                joint_ids=self._action_joint_ids,
                env_ids=env_ids,
            )
            reference_action = self._reference_at(
                "joint_position_urad", frame, env_ids
            )[:, self._action_to_dof].to(torch.float32)
            self._action[env_ids] = 0.0
            self._applied_target[env_ids] = reference_action
            self._previous_applied_target[env_ids] = reference_action
            self._previous_effort[env_ids] = 0.0
            self._applied_effort[env_ids] = 0.0
            self._positive_work[env_ids] = 0.0
            self._substep_hard_rom_violation[env_ids] = False
            self._substep_joint_safety_violation[env_ids] = False
            self._substep_velocity_violation[env_ids] = False
            self._substep_effort_envelope_violation[env_ids] = False
            self._substep_hard_rom_excess_by_action_channel[env_ids] = 0.0
            self._substep_velocity_excess_by_action_channel[env_ids] = 0.0
            self._cursor[env_ids] = frame
            self._tracking_loss_ticks[env_ids] = 0
            self._forbidden_contact_substeps[env_ids] = 0
            self._failure_terminal[env_ids] = False
            self._success_terminal[env_ids] = False
            self._episode_reward_sum[env_ids] = 0.0
            self._episode_component_sums[env_ids] = 0.0
            self.reward_components[env_ids] = 0.0
            self.contacts.reset(env_ids)

else:

    class NextEngineReferenceDirectEnvCfg:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")


    class NextEngineReferenceDirectEnv:
        def __init__(self, *_: Any, **__: Any) -> None:
            raise RuntimeError("pinned Isaac Lab profile is not installed")
