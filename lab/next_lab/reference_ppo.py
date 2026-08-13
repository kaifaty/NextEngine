from __future__ import annotations

import hashlib
import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

import torch
from torch import nn
from torch.distributions import Normal


@dataclass(frozen=True)
class TinyReferencePpoProfile:
    document: Mapping[str, Any]
    sha256: str

    @classmethod
    def load(cls, path: Path) -> "TinyReferencePpoProfile":
        payload = path.read_bytes()
        document = json.loads(payload)
        if "base_profile_sha256" in document:
            overlay = document
            variant_kind = overlay.get("variant", {}).get("kind")
            if variant_kind in {
                "replace-environment.v1",
                "replace-environment-and-corpus.v1",
            }:
                if (
                    overlay.get("schema_version") != 1
                    or overlay.get("status") != "Frozen"
                    or overlay.get("base_profile_id")
                    != "nextengine.training.humanoid-reference-ppo-tiny.v1"
                    or overlay.get("base_profile_sha256")
                    != "ac34da5ab39a069e8de9ecd19730bfed2c4dce9ac8753c7a03bbfdb99c8df55b"
                ):
                    raise ValueError("invalid reference PPO profile variant")
                base_name = "humanoid-reference-ppo-tiny.v1.json"
            elif (
                overlay.get("variant", {}).get("kind")
                == "replace-environment-and-initialization.v1"
            ):
                if (
                    overlay.get("schema_version") != 1
                    or overlay.get("status") != "Frozen"
                    or overlay.get("base_profile_id")
                    != "nextengine.training.humanoid-reference-ppo-curriculum-stage.v1"
                    or overlay.get("base_profile_sha256")
                    != "d97c93adb39a32694fc2f882732e59561949c726e5827c13a844f8edb45d01d1"
                ):
                    raise ValueError("invalid reference PPO curriculum variant")
                base_name = "humanoid-reference-ppo-curriculum-start-phase.v1.json"
            else:
                raise ValueError("invalid reference PPO profile variant kind")
            base_path = path.with_name(base_name)
            base_payload = base_path.read_bytes()
            if hashlib.sha256(base_payload).hexdigest() != overlay["base_profile_sha256"]:
                raise ValueError("reference PPO variant base hash mismatch")
            document = json.loads(base_payload)
            document["profile_id"] = overlay["profile_id"]
            document["environment_profile_id"] = overlay["variant"][
                "environment_profile_id"
            ]
            document["environment_profile_sha256"] = overlay["variant"][
                "environment_profile_sha256"
            ]
            if variant_kind == "replace-environment-and-corpus.v1":
                corpus_manifest_sha256 = overlay["variant"].get(
                    "corpus_manifest_sha256"
                )
                if (
                    not isinstance(corpus_manifest_sha256, str)
                    or len(corpus_manifest_sha256) != 64
                ):
                    raise ValueError("invalid reference PPO corpus replacement")
                document["corpus_manifest_sha256"] = corpus_manifest_sha256
            if "initialization" in overlay["variant"]:
                document["initialization"] = overlay["variant"]["initialization"]
                document["scope"]["stage_id"] = overlay["variant"]["stage_id"]
        profile_id = document.get("profile_id")
        fixed_tiny_profile_ids = {
            "nextengine.training.humanoid-reference-ppo-tiny.v1",
            "nextengine.training.humanoid-reference-ppo-tiny-soft-rom-cost.v1",
            "nextengine.training.humanoid-reference-ppo-tiny-predictive-rom-cost.v1",
            "nextengine.training.humanoid-reference-ppo-tiny-physics-velocity-guard.v4",
        }
        isolated_curriculum_profile_ids = {
            "nextengine.training.humanoid-reference-ppo-curriculum-stage.v2",
            "nextengine.training.humanoid-reference-ppo-curriculum-stage.v3",
            "nextengine.training.humanoid-reference-ppo-curriculum-stage.v4",
        }
        if (
            document.get("schema_version") != 1
            or profile_id
            not in {
                *fixed_tiny_profile_ids,
                "nextengine.training.humanoid-reference-ppo-curriculum-stage.v1",
                *isolated_curriculum_profile_ids,
            }
            or document.get("status") != "Frozen"
        ):
            raise ValueError("unsupported reference PPO profile")
        execution = document["execution"]
        scope = document["scope"]
        ppo = document["ppo"]
        distribution = document["policy_distribution"]
        if (
            execution["num_envs"] <= 0
            or execution["rollout_steps_per_env"] <= 0
            or execution["iterations"] <= 0
            or execution["cublas_workspace_config"] not in {":4096:8", ":16:8"}
            or scope["split"] != "train"
            or scope["horizon_motor_ticks"] <= 0
            or scope["reset_mode"] != "exact_reference"
            or ppo["minibatches"] <= 0
            or ppo["update_epochs"] <= 0
            or execution["num_envs"] * execution["rollout_steps_per_env"]
            % ppo["minibatches"]
            != 0
            or distribution["kind"] != "diagonal-normal-followed-by-tanh"
            or not distribution["minimum_log_standard_deviation"]
            < distribution["initial_log_standard_deviation"]
            < distribution["maximum_log_standard_deviation"]
            or ppo["terminated_bootstrap"] is not False
            or ppo["truncated_bootstrap"] is not True
        ):
            raise ValueError("invalid tiny reference PPO bounds")
        if profile_id in fixed_tiny_profile_ids:
            if scope["phase_randomization"] is not False or "clip_id" not in scope:
                raise ValueError("invalid fixed tiny reference scope")
        else:
            initialization = document.get("initialization", {})
            eligible = scope.get("eligible_clip_ids")
            if (
                scope["phase_randomization"] is not True
                or not isinstance(eligible, list)
                or not eligible
                or len(eligible) != len(set(eligible))
                or len(scope.get("rng_run_root_hex", "")) != 64
                or initialization.get("mode") != "model-weights-only"
                or len(initialization.get("checkpoint_sha256", "")) != 64
            ):
                raise ValueError("invalid phase-randomized curriculum scope")
        evaluation_num_envs = document["evaluation"].get("num_envs")
        if evaluation_num_envs is not None and (
            evaluation_num_envs <= 0
            or evaluation_num_envs > execution["num_envs"]
        ):
            raise ValueError("invalid deterministic evaluation cohort")
        evaluation_matrix = document["evaluation"].get(
            "episode_matrix", "completion-order-v1"
        )
        if evaluation_matrix not in {
            "completion-order-v1",
            "fixed-vector-waves-v1",
        }:
            raise ValueError("invalid deterministic evaluation matrix")
        if profile_id in isolated_curriculum_profile_ids and (
            evaluation_num_envs is None
            or evaluation_matrix != "fixed-vector-waves-v1"
            or not execution.get("reset_episode_sequence_before_training", False)
        ):
            raise ValueError("V2 curriculum requires an isolated evaluation matrix")
        return cls(document=document, sha256=hashlib.sha256(payload).hexdigest())


class TanhActorCritic(nn.Module):
    def __init__(self, profile: TinyReferencePpoProfile) -> None:
        super().__init__()
        network = profile.document["network"]
        self.actor = _mlp(
            435,
            list(network["actor_hidden_dimensions"]),
            23,
            network,
            float(network["actor_output_gain"]),
        )
        self.critic = _mlp(
            435,
            list(network["critic_hidden_dimensions"]),
            1,
            network,
            float(network["critic_output_gain"]),
        )
        initial = float(
            profile.document["policy_distribution"]["initial_log_standard_deviation"]
        )
        self.log_std = nn.Parameter(torch.full((23,), initial))
        self.minimum_log_std = float(
            profile.document["policy_distribution"]["minimum_log_standard_deviation"]
        )
        self.maximum_log_std = float(
            profile.document["policy_distribution"]["maximum_log_standard_deviation"]
        )

    def distribution(self, observation: torch.Tensor) -> Normal:
        mean = self.actor(observation)
        log_std = torch.clamp(self.log_std, self.minimum_log_std, self.maximum_log_std)
        return Normal(mean, torch.exp(log_std).expand_as(mean))

    def sample(
        self, observation: torch.Tensor
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
        distribution = self.distribution(observation)
        pre_tanh = distribution.rsample()
        action = torch.tanh(pre_tanh)
        log_probability = _transformed_log_probability(distribution, pre_tanh)
        value = self.critic(observation).squeeze(-1)
        return action, pre_tanh, log_probability, value

    def evaluate(
        self, observation: torch.Tensor, pre_tanh: torch.Tensor
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        distribution = self.distribution(observation)
        log_probability = _transformed_log_probability(distribution, pre_tanh)
        entropy_estimate = -log_probability
        value = self.critic(observation).squeeze(-1)
        return log_probability, entropy_estimate, value

    def deterministic(self, observation: torch.Tensor) -> torch.Tensor:
        return torch.tanh(self.actor(observation))


def _mlp(
    input_width: int,
    hidden: list[int],
    output_width: int,
    network: Mapping[str, Any],
    output_gain: float,
) -> nn.Sequential:
    if network["activation"] != "elu" or not hidden:
        raise ValueError("unsupported tiny reference network")
    layers: list[nn.Module] = []
    width = input_width
    for next_width in hidden:
        linear = nn.Linear(width, next_width)
        nn.init.orthogonal_(linear.weight, gain=float(network["orthogonal_hidden_gain"]))
        nn.init.zeros_(linear.bias)
        layers.extend((linear, nn.ELU()))
        width = next_width
    output = nn.Linear(width, output_width)
    nn.init.orthogonal_(output.weight, gain=output_gain)
    nn.init.zeros_(output.bias)
    layers.append(output)
    return nn.Sequential(*layers)


def _transformed_log_probability(
    distribution: Normal, pre_tanh: torch.Tensor
) -> torch.Tensor:
    log_jacobian = 2.0 * (
        math.log(2.0) - pre_tanh - torch.nn.functional.softplus(-2.0 * pre_tanh)
    )
    return torch.sum(distribution.log_prob(pre_tanh) - log_jacobian, dim=-1)


class TinyReferencePpoTrainer:
    def __init__(
        self,
        environment: Any,
        profile: TinyReferencePpoProfile,
        metrics_path: Path,
        performance_recorder: Any | None = None,
    ) -> None:
        self.environment = environment
        self.profile = profile
        self.device = torch.device(profile.document["execution"]["device"])
        self.model = TanhActorCritic(profile).to(self.device)
        ppo = profile.document["ppo"]
        self.optimizer = torch.optim.Adam(
            self.model.parameters(),
            lr=float(ppo["learning_rate"]),
            eps=float(ppo["adam_epsilon"]),
        )
        self.metrics_path = metrics_path
        self.performance_recorder = performance_recorder
        self.optimizer_steps = 0
        self.samples = 0
        rollout_steps = int(profile.document["execution"]["rollout_steps_per_env"])
        num_envs = int(environment.num_envs)
        self._rollout_storage = {
            "observation": torch.empty(
                (rollout_steps, num_envs, 435), device=self.device
            ),
            "pre_tanh": torch.empty(
                (rollout_steps, num_envs, 23), device=self.device
            ),
            "log_probability": torch.empty(
                (rollout_steps, num_envs), device=self.device
            ),
            "reward": torch.empty((rollout_steps, num_envs), device=self.device),
            "value": torch.empty((rollout_steps, num_envs), device=self.device),
            "terminated": torch.empty(
                (rollout_steps, num_envs), dtype=torch.bool, device=self.device
            ),
            "truncated": torch.empty(
                (rollout_steps, num_envs), dtype=torch.bool, device=self.device
            ),
            "success": torch.empty(
                (rollout_steps, num_envs), dtype=torch.bool, device=self.device
            ),
            "failure": torch.empty(
                (rollout_steps, num_envs), dtype=torch.bool, device=self.device
            ),
            "advantage": torch.empty(
                (rollout_steps, num_envs), device=self.device
            ),
            "return": torch.empty((rollout_steps, num_envs), device=self.device),
        }

    def evaluate_deterministic(
        self,
        episodes: int,
        *,
        include_hard_rom_state_samples: bool = False,
        include_terminal_state_samples: bool = False,
    ) -> dict[str, Any]:
        if episodes <= 0:
            raise ValueError("deterministic evaluation requires at least one episode")
        evaluation_num_envs = int(
            self.profile.document["evaluation"].get(
                "num_envs", self.environment.num_envs
            )
        )
        if not 0 < evaluation_num_envs <= self.environment.num_envs:
            raise ValueError("deterministic evaluation cohort exceeds the environment")
        fixed_vector_waves = (
            self.profile.document["evaluation"].get("episode_matrix")
            == "fixed-vector-waves-v1"
        )
        reset_episode_sequence = getattr(
            self.environment, "reset_episode_sequence", None
        )
        if (
            self.profile.document["scope"]["phase_randomization"]
            and reset_episode_sequence is None
        ):
            raise RuntimeError(
                "phase-randomized evaluation requires a resettable episode sequence"
            )
        if reset_episode_sequence is not None:
            if fixed_vector_waves:
                reset_episode_sequence(0)
            else:
                reset_episode_sequence()
        observation, _ = self.environment.reset()
        observation = observation["policy"]
        returns = torch.zeros(self.environment.num_envs, device=self.device)
        lengths = torch.zeros(
            self.environment.num_envs, dtype=torch.int64, device=self.device
        )
        completed_returns: list[float] = []
        completed_lengths: list[int] = []
        reference_complete_count = 0
        failure_count = 0
        failure_reason_counts = {
            "reference_tracking_lost": 0,
            "hard_rom": 0,
            "joint_safety": 0,
            "joint_velocity": 0,
            "effort_envelope": 0,
            "hard_impact": 0,
            "self_collision": 0,
            "forbidden_contact": 0,
            "world_bounds": 0,
            "fall": 0,
            "non_finite": 0,
        }
        selection_results: dict[str, dict[str, Any]] = {}
        maximum_hard_rom_excess_microradians = 0
        maximum_hard_rom_action_channel = -1
        maximum_hard_rom_selection = ""
        hard_rom_action_channel_counts: dict[str, int] = {}
        hard_rom_state_samples: list[dict[str, Any]] = []
        terminal_state_samples: list[dict[str, Any]] = []
        forbidden_contact_mask_counts: dict[str, int] = {}
        executed_motor_steps = 0
        maximum_motor_steps = max(episodes, evaluation_num_envs) * (
            int(self.environment.max_episode_length) + 1
        )
        evaluation_mask = torch.arange(
            self.environment.num_envs, device=self.device
        ) < evaluation_num_envs
        wave_completed = torch.zeros(
            self.environment.num_envs, dtype=torch.bool, device=self.device
        )
        wave_ordinal = 0
        wave_target = min(evaluation_num_envs, episodes)
        wave_completed_count = 0
        with torch.no_grad():
            while len(completed_returns) < episodes:
                if executed_motor_steps >= maximum_motor_steps:
                    raise RuntimeError(
                        "deterministic evaluation exceeded its finite motor-step budget"
                    )
                action = self.model.deterministic(observation)
                observation_map, reward, terminated, truncated, _ = self.environment.step(
                    action
                )
                returns += reward
                lengths += 1
                executed_motor_steps += 1
                all_done = terminated | truncated
                eligible_done = (
                    all_done
                    if evaluation_num_envs == self.environment.num_envs
                    else all_done & evaluation_mask
                )
                done = (
                    eligible_done & ~wave_completed
                    if fixed_vector_waves
                    else eligible_done
                )
                if fixed_vector_waves:
                    wave_completed |= done
                done_indices = torch.nonzero(done, as_tuple=False).squeeze(-1)
                hard_rom_state_batch: dict[str, list[Any]] = {}
                if include_hard_rom_state_samples:
                    required_diagnostics = (
                        "last_step_pre_physics_action_joint_position_microradians",
                        "last_step_action_joint_velocity_microradians_per_second",
                        "last_step_pre_physics_action_joint_velocity_microradians_per_second",
                        "last_step_applied_target_microradians",
                        "last_step_previous_applied_target_microradians",
                        "last_step_command_reference_target_microradians",
                    )
                    missing = [
                        name
                        for name in required_diagnostics
                        if not hasattr(self.environment, name)
                    ]
                    if missing:
                        raise RuntimeError(
                            "hard-ROM state diagnostic is unavailable: "
                            + ", ".join(missing)
                        )
                    hard_rom_state_batch = {
                        "excess": self.environment.last_step_hard_rom_excess_by_action_channel[
                            done_indices
                        ].cpu().tolist(),
                        "position": self.environment.last_step_action_joint_position_microradians[
                            done_indices
                        ].cpu().tolist(),
                        "pre_physics_position": self.environment.last_step_pre_physics_action_joint_position_microradians[
                            done_indices
                        ].cpu().tolist(),
                        "velocity": self.environment.last_step_action_joint_velocity_microradians_per_second[
                            done_indices
                        ].cpu().tolist(),
                        "pre_physics_velocity": self.environment.last_step_pre_physics_action_joint_velocity_microradians_per_second[
                            done_indices
                        ].cpu().tolist(),
                        "applied_target": self.environment.last_step_applied_target_microradians[
                            done_indices
                        ].cpu().tolist(),
                        "previous_applied_target": self.environment.last_step_previous_applied_target_microradians[
                            done_indices
                        ].cpu().tolist(),
                        "command_reference_target": self.environment.last_step_command_reference_target_microradians[
                            done_indices
                        ].cpu().tolist(),
                        "policy_action": action[done_indices].cpu().tolist(),
                        "reference_frame": self.environment.last_step_reference_frame[
                            done_indices
                        ].cpu().tolist(),
                        "elapsed_motor_ticks": self.environment.last_step_episode_elapsed_motor_ticks[
                            done_indices
                        ].cpu().tolist(),
                    }
                terminal_state_batch: dict[str, list[Any]] = {}
                if include_terminal_state_samples:
                    required_terminal_diagnostics = (
                        "last_step_root_position_error_micrometres",
                        "last_step_root_orientation_absolute_dot_q1_30",
                        "last_step_tracking_loss_ticks",
                    )
                    missing = [
                        name
                        for name in required_terminal_diagnostics
                        if not hasattr(self.environment, name)
                    ]
                    if missing:
                        raise RuntimeError(
                            "terminal state diagnostic is unavailable: "
                            + ", ".join(missing)
                        )
                    terminal_state_batch = {
                        "root_position_error": self.environment.last_step_root_position_error_micrometres[
                            done_indices
                        ].cpu().tolist(),
                        "root_orientation_dot": self.environment.last_step_root_orientation_absolute_dot_q1_30[
                            done_indices
                        ].cpu().tolist(),
                        "tracking_loss_ticks": self.environment.last_step_tracking_loss_ticks[
                            done_indices
                        ].cpu().tolist(),
                        "reference_frame": self.environment.last_step_reference_frame[
                            done_indices
                        ].cpu().tolist(),
                        "elapsed_motor_ticks": self.environment.last_step_episode_elapsed_motor_ticks[
                            done_indices
                        ].cpu().tolist(),
                        "policy_action": action[done_indices].cpu().tolist(),
                    }
                completed_rows = torch.stack(
                    (
                        done_indices.to(torch.float64),
                        returns[done_indices].to(torch.float64),
                        lengths[done_indices].to(torch.float64),
                        self.environment.last_step_success[done_indices].to(
                            torch.float64
                        ),
                        self.environment.last_step_failure[done_indices].to(
                            torch.float64
                        ),
                        self.environment.last_step_episode_clip_index[done_indices].to(
                            torch.float64
                        ),
                        self.environment.last_step_episode_start_frame[done_indices].to(
                            torch.float64
                        ),
                        self.environment.last_step_failure_tracking_lost[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_hard_rom[done_indices].to(
                            torch.float64
                        ),
                        self.environment.last_step_failure_joint_safety[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_joint_velocity[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_effort_envelope[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_hard_impact[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_self_collision[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_forbidden_contact[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_world_bounds[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_fall[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_failure_non_finite[done_indices].to(
                            torch.float64
                        ),
                        self.environment.last_step_hard_rom_excess_microradians[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_hard_rom_action_channel[
                            done_indices
                        ].to(torch.float64),
                        self.environment.last_step_forbidden_contact_mask[
                            done_indices
                        ].to(torch.float64),
                    ),
                    dim=-1,
                ).cpu().tolist()
                for completed_index, row in enumerate(completed_rows):
                    if len(completed_returns) >= episodes:
                        break
                    (
                        _,
                        completed_return,
                        completed_length,
                        success,
                        failure,
                        clip_index_value,
                        start_frame_value,
                        tracking_lost,
                        hard_rom,
                        joint_safety,
                        joint_velocity,
                        effort_envelope,
                        hard_impact,
                        self_collision,
                        forbidden_contact,
                        world_bounds,
                        fall,
                        non_finite,
                        hard_rom_excess_value,
                        hard_rom_channel_value,
                        contact_mask_value,
                    ) = row
                    completed_returns.append(float(completed_return))
                    completed_lengths.append(int(completed_length))
                    reference_complete_count += int(success)
                    failure_count += int(failure)
                    clip_index = int(clip_index_value)
                    start_frame = int(start_frame_value)
                    selection_id = (
                        f"{self.environment.reference_clips[clip_index].clip_id}:{start_frame}"
                    )
                    selection = selection_results.setdefault(
                        selection_id,
                        {
                            "episodes": 0,
                            "reference_complete_count": 0,
                            "failure_count": 0,
                            "failure_reason_counts": {
                                "reference_tracking_lost": 0,
                                "hard_rom": 0,
                                "joint_safety": 0,
                                "joint_velocity": 0,
                                "effort_envelope": 0,
                                "hard_impact": 0,
                                "self_collision": 0,
                                "forbidden_contact": 0,
                                "world_bounds": 0,
                                "fall": 0,
                                "non_finite": 0,
                            },
                        },
                    )
                    selection["episodes"] += 1
                    selection["reference_complete_count"] += int(success)
                    selection["failure_count"] += int(failure)
                    if include_terminal_state_samples:
                        terminal_state_samples.append(
                            {
                                "selection_id": selection_id,
                                "episode_length_motor_ticks": int(completed_length),
                                "reference_frame": int(
                                    terminal_state_batch["reference_frame"][
                                        completed_index
                                    ]
                                ),
                                "elapsed_motor_ticks": int(
                                    terminal_state_batch["elapsed_motor_ticks"][
                                        completed_index
                                    ]
                                ),
                                "root_position_error_micrometres": int(
                                    terminal_state_batch["root_position_error"][
                                        completed_index
                                    ]
                                ),
                                "root_orientation_absolute_dot_q1_30": int(
                                    terminal_state_batch["root_orientation_dot"][
                                        completed_index
                                    ]
                                ),
                                "tracking_loss_ticks": int(
                                    terminal_state_batch["tracking_loss_ticks"][
                                        completed_index
                                    ]
                                ),
                                "failure_reasons": [
                                    reason
                                    for reason, occurred in (
                                        ("reference_tracking_lost", tracking_lost),
                                        ("hard_rom", hard_rom),
                                        ("joint_safety", joint_safety),
                                        ("hard_impact", hard_impact),
                                        ("self_collision", self_collision),
                                        ("forbidden_contact", forbidden_contact),
                                        ("world_bounds", world_bounds),
                                        ("fall", fall),
                                        ("non_finite", non_finite),
                                    )
                                    if occurred
                                ],
                                "policy_action": terminal_state_batch["policy_action"][
                                    completed_index
                                ],
                            }
                        )
                    for reason, occurred_value in (
                        ("reference_tracking_lost", tracking_lost),
                        ("hard_rom", hard_rom),
                        ("joint_safety", joint_safety),
                        ("joint_velocity", joint_velocity),
                        ("effort_envelope", effort_envelope),
                        ("hard_impact", hard_impact),
                        ("self_collision", self_collision),
                        ("forbidden_contact", forbidden_contact),
                        ("world_bounds", world_bounds),
                        ("fall", fall),
                        ("non_finite", non_finite),
                    ):
                        occurred = int(occurred_value)
                        failure_reason_counts[reason] += occurred
                        selection["failure_reason_counts"][reason] += occurred
                    hard_rom_excess = int(hard_rom_excess_value)
                    hard_rom_channel = int(hard_rom_channel_value)
                    if hard_rom and hard_rom_channel >= 0:
                        channel_key = str(hard_rom_channel)
                        hard_rom_action_channel_counts[channel_key] = (
                            hard_rom_action_channel_counts.get(channel_key, 0) + 1
                        )
                        if include_hard_rom_state_samples:
                            action_profile = self.environment.reference_profile.document[
                                "action"
                            ]
                            actuator_ids = action_profile["ordered_actuator_ids"]
                            violations: list[dict[str, Any]] = []
                            for channel, excess in enumerate(
                                hard_rom_state_batch["excess"][completed_index]
                            ):
                                if int(excess) <= 10:
                                    continue
                                violations.append(
                                    {
                                        "action_channel": channel,
                                        "actuator_id": actuator_ids[channel],
                                        "hard_rom_excess_microradians": int(excess),
                                        "joint_position_microradians": int(
                                            hard_rom_state_batch["position"][completed_index][
                                                channel
                                            ]
                                        ),
                                        "pre_physics_joint_position_microradians": int(
                                            hard_rom_state_batch[
                                                "pre_physics_position"
                                            ][completed_index][channel]
                                        ),
                                        "joint_velocity_microradians_per_second": int(
                                            hard_rom_state_batch["velocity"][completed_index][
                                                channel
                                            ]
                                        ),
                                        "pre_physics_joint_velocity_microradians_per_second": int(
                                            hard_rom_state_batch[
                                                "pre_physics_velocity"
                                            ][completed_index][channel]
                                        ),
                                        "applied_target_microradians": int(
                                            hard_rom_state_batch["applied_target"][
                                                completed_index
                                            ][channel]
                                        ),
                                        "previous_applied_target_microradians": int(
                                            hard_rom_state_batch[
                                                "previous_applied_target"
                                            ][completed_index][channel]
                                        ),
                                        "command_reference_target_microradians": int(
                                            hard_rom_state_batch[
                                                "command_reference_target"
                                            ][completed_index][channel]
                                        ),
                                        "policy_action": float(
                                            hard_rom_state_batch["policy_action"][
                                                completed_index
                                            ][channel]
                                        ),
                                    }
                                )
                            hard_rom_state_samples.append(
                                {
                                    "selection_id": selection_id,
                                    "episode_length_motor_ticks": int(completed_length),
                                    "reference_frame": int(
                                        hard_rom_state_batch["reference_frame"][
                                            completed_index
                                        ]
                                    ),
                                    "elapsed_motor_ticks": int(
                                        hard_rom_state_batch["elapsed_motor_ticks"][
                                            completed_index
                                        ]
                                    ),
                                    "violations": violations,
                                }
                            )
                    if (
                        hard_rom
                        and hard_rom_excess > maximum_hard_rom_excess_microradians
                    ):
                        maximum_hard_rom_excess_microradians = hard_rom_excess
                        maximum_hard_rom_action_channel = hard_rom_channel
                        maximum_hard_rom_selection = selection_id
                    contact_mask = int(contact_mask_value)
                    if forbidden_contact and contact_mask:
                        mask_key = str(contact_mask)
                        forbidden_contact_mask_counts[mask_key] = (
                            forbidden_contact_mask_counts.get(mask_key, 0) + 1
                        )
                if fixed_vector_waves:
                    wave_completed_count += len(completed_rows)
                returns[all_done] = 0.0
                lengths[all_done] = 0
                observation = observation_map["policy"]
                if (
                    fixed_vector_waves
                    and len(completed_returns) < episodes
                    and wave_completed_count == wave_target
                ):
                    wave_ordinal += 1
                    remaining = episodes - len(completed_returns)
                    wave_target = min(evaluation_num_envs, remaining)
                    evaluation_mask = torch.arange(
                        self.environment.num_envs, device=self.device
                    ) < wave_target
                    wave_completed.zero_()
                    wave_completed_count = 0
                    reset_episode_sequence(wave_ordinal)
                    observation_map, _ = self.environment.reset()
                    observation = observation_map["policy"]
                    returns.zero_()
                    lengths.zero_()
        ordered_selection_results = dict(sorted(selection_results.items()))
        result = {
            "episodes": episodes,
            "reference_complete_count": reference_complete_count,
            "failure_count": failure_count,
            "failure_reason_counts": failure_reason_counts,
            "selection_results": ordered_selection_results,
            "selection_episode_matrix_hash": _selection_episode_matrix_hash(
                ordered_selection_results
            ),
            "maximum_hard_rom_excess_microradians": maximum_hard_rom_excess_microradians,
            "maximum_hard_rom_action_channel": maximum_hard_rom_action_channel,
            "maximum_hard_rom_selection": maximum_hard_rom_selection,
            "hard_rom_action_channel_counts": dict(
                sorted(hard_rom_action_channel_counts.items())
            ),
            "forbidden_contact_mask_counts": dict(
                sorted(forbidden_contact_mask_counts.items())
            ),
            "mean_return": sum(completed_returns) / len(completed_returns),
            "mean_episode_length": sum(completed_lengths) / len(completed_lengths),
            "minimum_episode_length": min(completed_lengths),
            "maximum_episode_length": max(completed_lengths),
        }
        if "num_envs" in self.profile.document["evaluation"]:
            result["vector_envs"] = evaluation_num_envs
            result["episode_matrix"] = self.profile.document["evaluation"].get(
                "episode_matrix", "completion-order-v1"
            )
        if include_hard_rom_state_samples:
            result["hard_rom_state_samples"] = hard_rom_state_samples
        if include_terminal_state_samples:
            result["terminal_state_samples"] = terminal_state_samples
        return result

    def train(self) -> list[dict[str, Any]]:
        document = self.profile.document
        execution = document["execution"]
        ppo = document["ppo"]
        if execution.get("reset_episode_sequence_before_training", False):
            reset_episode_sequence = getattr(
                self.environment, "reset_episode_sequence", None
            )
            if reset_episode_sequence is None:
                raise RuntimeError(
                    "resolved training profile requires a resettable episode sequence"
                )
            reset_episode_sequence()
        observation_map, _ = self.environment.reset()
        observation = observation_map["policy"]
        records: list[dict[str, Any]] = []
        for iteration in range(int(execution["iterations"])):
            iteration_number = iteration + 1
            if self.performance_recorder is not None:
                self.performance_recorder.begin(iteration_number, "rollout")
            rollout = self._collect_rollout(
                observation, int(execution["rollout_steps_per_env"])
            )
            if self.performance_recorder is not None:
                self.performance_recorder.end(iteration_number, "rollout")
            observation = rollout.pop("next_observation")
            if self.performance_recorder is not None:
                self.performance_recorder.begin(iteration_number, "update")
            metrics = self._update(rollout)
            if self.performance_recorder is not None:
                self.performance_recorder.end(iteration_number, "update")
            with torch.no_grad():
                rollout_summary = torch.stack(
                    (
                        rollout["reward"].mean().to(torch.float64),
                        rollout["success"].sum().to(torch.float64),
                        rollout["failure"].sum().to(torch.float64),
                        torch.exp(
                            torch.clamp(
                                self.model.log_std,
                                self.model.minimum_log_std,
                                self.model.maximum_log_std,
                            )
                        )
                        .mean()
                        .to(torch.float64),
                    )
                ).cpu().tolist()
            metrics.update(
                {
                    "iteration": iteration_number,
                    "samples": self.samples,
                    "optimizer_steps": self.optimizer_steps,
                    "rollout_mean_reward": float(rollout_summary[0]),
                    "rollout_reference_complete_count": int(rollout_summary[1]),
                    "rollout_failure_count": int(rollout_summary[2]),
                    "action_standard_deviation_mean": float(rollout_summary[3]),
                }
            )
            _require_finite_mapping(metrics)
            self.metrics_path.parent.mkdir(parents=True, exist_ok=True)
            with self.metrics_path.open("a", encoding="utf-8") as output:
                output.write(json.dumps(metrics, sort_keys=True) + "\n")
            records.append(metrics)
        return records

    def _collect_rollout(
        self, observation: torch.Tensor, steps: int
    ) -> dict[str, torch.Tensor]:
        configured_steps = int(
            self.profile.document["execution"]["rollout_steps_per_env"]
        )
        if steps != configured_steps:
            raise ValueError("rollout steps must match the resolved training profile")
        storage = self._rollout_storage
        for index in range(steps):
            with torch.no_grad():
                action, pre_tanh, log_probability, value = self.model.sample(observation)
            next_observation, reward, terminated, truncated, _ = self.environment.step(
                action
            )
            for name, value_to_store in (
                ("observation", observation),
                ("pre_tanh", pre_tanh),
                ("log_probability", log_probability),
                ("reward", reward),
                ("value", value),
                ("terminated", terminated),
                ("truncated", truncated),
                ("success", self.environment.last_step_success),
                ("failure", self.environment.last_step_failure),
            ):
                storage[name][index].copy_(value_to_store.detach())
            observation = next_observation["policy"]
        if bool(torch.any(storage["truncated"]).item()):
            raise RuntimeError(
                "tiny reference run unexpectedly truncated before a valid bootstrap capture"
            )
        with torch.no_grad():
            next_value = self.model.critic(observation).squeeze(-1)
        result = dict(storage)
        result["next_observation"] = observation
        advantage = result["advantage"]
        advantage.zero_()
        last_advantage = torch.zeros(self.environment.num_envs, device=self.device)
        for index in range(steps - 1, -1, -1):
            nonterminal = (~result["terminated"][index]).to(torch.float32)
            following_value = next_value if index == steps - 1 else result["value"][index + 1]
            delta = (
                result["reward"][index]
                + float(self.profile.document["ppo"]["discount"])
                * following_value
                * nonterminal
                - result["value"][index]
            )
            last_advantage = delta + (
                float(self.profile.document["ppo"]["discount"])
                * float(self.profile.document["ppo"]["gae_lambda"])
                * nonterminal
                * last_advantage
            )
            advantage[index] = last_advantage
        torch.add(advantage, result["value"], out=result["return"])
        self.samples += steps * self.environment.num_envs
        return result

    def _update(self, rollout: Mapping[str, torch.Tensor]) -> dict[str, float]:
        ppo = self.profile.document["ppo"]
        batch_size = rollout["reward"].numel()
        observation = rollout["observation"].reshape(batch_size, 435)
        pre_tanh = rollout["pre_tanh"].reshape(batch_size, 23)
        old_log_probability = rollout["log_probability"].reshape(batch_size)
        old_value = rollout["value"].reshape(batch_size)
        returns = rollout["return"].reshape(batch_size)
        advantage = rollout["advantage"].reshape(batch_size)
        if ppo["normalize_advantages"]:
            advantage = (advantage - advantage.mean()) / (advantage.std() + 1.0e-8)
        minibatches = int(ppo["minibatches"])
        minibatch_size = batch_size // minibatches
        totals = {
            "policy_loss": 0.0,
            "value_loss": 0.0,
            "entropy_estimate": 0.0,
            "approximate_kl": 0.0,
            "clip_fraction": 0.0,
            "gradient_norm": 0.0,
        }
        update_count = 0
        stop = False
        for _ in range(int(ppo["update_epochs"])):
            permutation = torch.randperm(batch_size, device=self.device)
            for start in range(0, batch_size, minibatch_size):
                indices = permutation[start : start + minibatch_size]
                new_log_probability, entropy, value = self.model.evaluate(
                    observation[indices], pre_tanh[indices]
                )
                log_ratio = new_log_probability - old_log_probability[indices]
                ratio = torch.exp(log_ratio)
                unclipped = ratio * advantage[indices]
                clipped = torch.clamp(
                    ratio,
                    1.0 - float(ppo["clip_ratio"]),
                    1.0 + float(ppo["clip_ratio"]),
                ) * advantage[indices]
                policy_loss = -torch.minimum(unclipped, clipped).mean()
                if ppo["clip_value_loss"]:
                    value_clipped = old_value[indices] + torch.clamp(
                        value - old_value[indices],
                        -float(ppo["clip_ratio"]),
                        float(ppo["clip_ratio"]),
                    )
                    value_loss = 0.5 * torch.maximum(
                        (value - returns[indices]).square(),
                        (value_clipped - returns[indices]).square(),
                    ).mean()
                else:
                    value_loss = 0.5 * (value - returns[indices]).square().mean()
                entropy_mean = entropy.mean()
                loss = (
                    policy_loss
                    + float(ppo["value_loss_coefficient"]) * value_loss
                    - float(ppo["entropy_coefficient"]) * entropy_mean
                )
                self.optimizer.zero_grad(set_to_none=True)
                loss.backward()
                gradient_norm = torch.nn.utils.clip_grad_norm_(
                    self.model.parameters(), float(ppo["maximum_gradient_norm"])
                )
                self.optimizer.step()
                with torch.no_grad():
                    self.model.log_std.clamp_(
                        self.model.minimum_log_std, self.model.maximum_log_std
                    )
                self.optimizer_steps += 1
                with torch.no_grad():
                    approximate_kl = torch.mean((ratio - 1.0) - log_ratio)
                    clip_fraction = torch.mean(
                        (torch.abs(ratio - 1.0) > float(ppo["clip_ratio"])).to(
                            torch.float32
                        )
                    )
                metric_values = torch.stack(
                    (
                        policy_loss.detach(),
                        value_loss.detach(),
                        entropy_mean.detach(),
                        approximate_kl.detach(),
                        clip_fraction.detach(),
                        gradient_norm.detach(),
                    )
                ).cpu().tolist()
                for name, value_to_add in zip(totals, metric_values, strict=True):
                    totals[name] += float(value_to_add)
                update_count += 1
                if metric_values[3] > float(ppo["target_approximate_kl"]):
                    stop = True
                    break
            if stop:
                break
        if update_count == 0:
            raise RuntimeError("PPO update performed no optimizer step")
        metrics = {name: value / update_count for name, value in totals.items()}
        metrics["update_minibatches"] = float(update_count)
        metrics["early_stop_kl"] = float(stop)
        return metrics


def _selection_episode_matrix_hash(
    selection_results: Mapping[str, Mapping[str, Any]],
) -> str:
    matrix = [
        {"selection_id": selection_id, "episodes": int(result["episodes"])}
        for selection_id, result in sorted(selection_results.items())
    ]
    return hashlib.sha256(
        json.dumps(
            matrix, ensure_ascii=True, separators=(",", ":"), sort_keys=True
        ).encode("utf-8")
    ).hexdigest()


def _require_finite_mapping(value: Mapping[str, Any]) -> None:
    for name, item in value.items():
        if isinstance(item, (int, bool)):
            continue
        if not math.isfinite(float(item)):
            raise RuntimeError(f"non-finite PPO metric: {name}")
