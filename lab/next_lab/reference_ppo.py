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
        profile_id = document.get("profile_id")
        if (
            document.get("schema_version") != 1
            or profile_id
            not in {
                "nextengine.training.humanoid-reference-ppo-tiny.v1",
                "nextengine.training.humanoid-reference-ppo-curriculum-stage.v1",
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
        if profile_id == "nextengine.training.humanoid-reference-ppo-tiny.v1":
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

    def evaluate_deterministic(self, episodes: int) -> dict[str, Any]:
        if episodes <= 0:
            raise ValueError("deterministic evaluation requires at least one episode")
        reset_episode_sequence = getattr(
            self.environment, "reset_episode_sequence", None
        )
        if reset_episode_sequence is not None:
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
            "forbidden_contact": 0,
            "non_finite": 0,
        }
        selection_results: dict[str, dict[str, Any]] = {}
        maximum_hard_rom_excess_microradians = 0
        maximum_hard_rom_action_channel = -1
        maximum_hard_rom_selection = ""
        forbidden_contact_mask_counts: dict[str, int] = {}
        executed_motor_steps = 0
        maximum_motor_steps = max(episodes, self.environment.num_envs) * (
            int(self.environment.max_episode_length) + 1
        )
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
                done = terminated | truncated
                for index in torch.nonzero(done, as_tuple=False).squeeze(-1).tolist():
                    if len(completed_returns) >= episodes:
                        break
                    completed_returns.append(float(returns[index].item()))
                    completed_lengths.append(int(lengths[index].item()))
                    reference_complete_count += int(
                        self.environment.last_step_success[index].item()
                    )
                    failure_count += int(self.environment.last_step_failure[index].item())
                    clip_index = int(
                        self.environment.last_step_episode_clip_index[index].item()
                    )
                    start_frame = int(
                        self.environment.last_step_episode_start_frame[index].item()
                    )
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
                                "forbidden_contact": 0,
                                "non_finite": 0,
                            },
                        },
                    )
                    selection["episodes"] += 1
                    selection["reference_complete_count"] += int(
                        self.environment.last_step_success[index].item()
                    )
                    selection["failure_count"] += int(
                        self.environment.last_step_failure[index].item()
                    )
                    for reason, attribute in (
                        (
                            "reference_tracking_lost",
                            "last_step_failure_tracking_lost",
                        ),
                        ("hard_rom", "last_step_failure_hard_rom"),
                        (
                            "forbidden_contact",
                            "last_step_failure_forbidden_contact",
                        ),
                        ("non_finite", "last_step_failure_non_finite"),
                    ):
                        occurred = int(
                            getattr(self.environment, attribute)[index].item()
                        )
                        failure_reason_counts[reason] += occurred
                        selection["failure_reason_counts"][reason] += occurred
                    hard_rom_excess = int(
                        self.environment.last_step_hard_rom_excess_microradians[
                            index
                        ].item()
                    )
                    if (
                        self.environment.last_step_failure_hard_rom[index].item()
                        and hard_rom_excess > maximum_hard_rom_excess_microradians
                    ):
                        maximum_hard_rom_excess_microradians = hard_rom_excess
                        maximum_hard_rom_action_channel = int(
                            self.environment.last_step_hard_rom_action_channel[
                                index
                            ].item()
                        )
                        maximum_hard_rom_selection = selection_id
                    contact_mask = int(
                        self.environment.last_step_forbidden_contact_mask[index].item()
                    )
                    if (
                        self.environment.last_step_failure_forbidden_contact[
                            index
                        ].item()
                        and contact_mask
                    ):
                        mask_key = str(contact_mask)
                        forbidden_contact_mask_counts[mask_key] = (
                            forbidden_contact_mask_counts.get(mask_key, 0) + 1
                        )
                returns[done] = 0.0
                lengths[done] = 0
                observation = observation_map["policy"]
        return {
            "episodes": episodes,
            "reference_complete_count": reference_complete_count,
            "failure_count": failure_count,
            "failure_reason_counts": failure_reason_counts,
            "selection_results": dict(sorted(selection_results.items())),
            "maximum_hard_rom_excess_microradians": maximum_hard_rom_excess_microradians,
            "maximum_hard_rom_action_channel": maximum_hard_rom_action_channel,
            "maximum_hard_rom_selection": maximum_hard_rom_selection,
            "forbidden_contact_mask_counts": dict(
                sorted(forbidden_contact_mask_counts.items())
            ),
            "mean_return": sum(completed_returns) / len(completed_returns),
            "mean_episode_length": sum(completed_lengths) / len(completed_lengths),
            "minimum_episode_length": min(completed_lengths),
            "maximum_episode_length": max(completed_lengths),
        }

    def train(self) -> list[dict[str, Any]]:
        document = self.profile.document
        execution = document["execution"]
        ppo = document["ppo"]
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
            metrics.update(
                {
                    "iteration": iteration_number,
                    "samples": self.samples,
                    "optimizer_steps": self.optimizer_steps,
                    "rollout_mean_reward": float(rollout["reward"].mean().item()),
                    "rollout_reference_complete_count": int(
                        rollout["success"].sum().item()
                    ),
                    "rollout_failure_count": int(rollout["failure"].sum().item()),
                    "action_standard_deviation_mean": float(
                        torch.exp(
                            torch.clamp(
                                self.model.log_std,
                                self.model.minimum_log_std,
                                self.model.maximum_log_std,
                            )
                        )
                        .mean()
                        .item()
                    ),
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
        storage: dict[str, list[torch.Tensor]] = {
            name: []
            for name in (
                "observation",
                "pre_tanh",
                "log_probability",
                "reward",
                "value",
                "terminated",
                "truncated",
                "success",
                "failure",
            )
        }
        for _ in range(steps):
            with torch.no_grad():
                action, pre_tanh, log_probability, value = self.model.sample(observation)
            next_observation, reward, terminated, truncated, _ = self.environment.step(
                action
            )
            if torch.any(truncated):
                raise RuntimeError(
                    "tiny reference run unexpectedly truncated before a valid bootstrap capture"
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
                storage[name].append(value_to_store.detach().clone())
            observation = next_observation["policy"]
        with torch.no_grad():
            next_value = self.model.critic(observation).squeeze(-1)
        result = {name: torch.stack(values) for name, values in storage.items()}
        result["next_observation"] = observation
        advantage = torch.zeros_like(result["reward"])
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
        result["advantage"] = advantage
        result["return"] = advantage + result["value"]
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
                for name, value_to_add in (
                    ("policy_loss", policy_loss),
                    ("value_loss", value_loss),
                    ("entropy_estimate", entropy_mean),
                    ("approximate_kl", approximate_kl),
                    ("clip_fraction", clip_fraction),
                    ("gradient_norm", gradient_norm),
                ):
                    totals[name] += float(value_to_add.detach().item())
                update_count += 1
                if float(approximate_kl.item()) > float(ppo["target_approximate_kl"]):
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


def _require_finite_mapping(value: Mapping[str, Any]) -> None:
    for name, item in value.items():
        if isinstance(item, (int, bool)):
            continue
        if not math.isfinite(float(item)):
            raise RuntimeError(f"non-finite PPO metric: {name}")
