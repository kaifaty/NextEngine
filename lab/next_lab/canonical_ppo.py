"""Canonical motor-lab vector adapter; neural computation may use CUDA.

No physics, reward, safety, command or episode state is owned by this adapter.
The process/slot partition is part of the run identity, not a tuning override.
"""

from __future__ import annotations

import hashlib
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Any

import numpy as np
import torch
from rsl_rl.algorithms import PPO
from tensordict import TensorDict

from next_lab.motor_lab_client import MotorLabClient, normalized_action_to_raw


def observation_scales(descriptor: dict[str, Any]) -> np.ndarray:
    joints = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    ordered = [joints[item["joint_id"]] for item in descriptor["actuators"]]
    position = [max(map(abs, joint["soft_limit_microradians"])) for joint in ordered]
    velocity = [joint["maximum_velocity_microradians_per_second"] for joint in ordered]
    clock_scales = []
    if descriptor.get("observation_width") == 86:
        profiles = descriptor["environment_profiles"]
        if len(profiles) != 1 or not profiles[0]["profile_id"].endswith(
            "forward-start-stop.v6"
        ):
            raise ValueError("86-channel observations require walking V6")
        clock_scales = [1 << 30] * 2
    scales = np.asarray(
        [1 << 30] * 4
        + [2_000_000] * 6
        + position
        + velocity
        + position
        + [1_000_000] * 3
        + [1, 1]
        + clock_scales,
        dtype=np.float64,
    )
    if (
        scales.shape != (84 + len(clock_scales),)
        or not np.isfinite(scales).all()
        or np.any(scales <= 0)
    ):
        raise ValueError("invalid canonical observation scales")
    return scales


def shard_root(run_root: str, shard: int) -> str:
    return hashlib.sha256(
        b"nextengine.canonical-ppo.shard.v1\0"
        + bytes.fromhex(run_root)
        + shard.to_bytes(4, "little")
    ).hexdigest()


class CanonicalVecEnv:
    def __init__(
        self,
        executable: Path,
        descriptor: dict[str, Any],
        *,
        num_envs: int,
        shards: int,
        run_root: str,
        device: str,
        client_factory=MotorLabClient,
    ) -> None:
        if shards <= 0 or num_envs <= 0 or num_envs % shards:
            raise ValueError("positive environment count must be divisible by shards")
        self.num_envs = num_envs
        self.num_actions = 23
        self.device = device
        self.scales = observation_scales(descriptor)
        # Only the explicitly selected command-only environment is accepted.
        matches = [
            item
            for item in descriptor["environment_profiles"]
            if item["profile_id"].endswith(
                ("forward-start-stop.v5", "forward-start-stop.v6")
            )
        ]
        if len(matches) != 1:
            raise ValueError(
                "descriptor must contain exactly one walking V5/V6 environment"
            )
        expected = matches[0]
        self.max_episode_length = expected["maximum_episode_steps"]
        self.cfg = {"environment_profile_id": expected["profile_id"]}
        self.slots_per_shard = num_envs // shards
        self.clients: list[MotorLabClient] = []
        self.pool = ThreadPoolExecutor(max_workers=shards)
        self.raw = np.empty((num_envs, len(self.scales)), dtype=np.int64)
        self.ordinals = np.empty(num_envs, dtype=np.int64)
        self.episode_length_buf = torch.zeros(num_envs, dtype=torch.long, device=device)
        self.last_steps: list[Any] = []
        try:
            for index in range(shards):
                client = client_factory(
                    executable,
                    expected["profile_id"],
                    self.slots_per_shard,
                    shard_root(run_root, index),
                )
                self.clients.append(client)
                for name in (
                    "manifest_hash",
                    "observation_layout_hash",
                    "action_layout_hash",
                    "command_schedule_profile_hash",
                    "reward_profile_hash",
                    "termination_profile_hash",
                    "rng_derivation_profile_hash",
                    "correspondence_profile_hash",
                ):
                    if getattr(client.descriptor, name).hex() != expected[name]:
                        raise ValueError(f"canonical descriptor mismatch: {name}")
                for name in (
                    "observation_width",
                    "action_width",
                    "maximum_episode_steps",
                    "physics_hz",
                    "motor_hz",
                ):
                    value = expected[name] if name in expected else descriptor[name]
                    if getattr(client.descriptor, name) != value:
                        raise ValueError(f"canonical descriptor mismatch: {name}")
                if client.descriptor.slots != self.slots_per_shard:
                    raise ValueError("canonical slot count mismatch")
                self._reset(index, list(range(self.slots_per_shard)))
        except BaseException:
            self.close()
            raise

    def _reset(self, shard: int, slots: list[int]) -> None:
        # Native responses use (episode ordinal, vector slot), not slot order.
        results = sorted(
            self.clients[shard].reset(slots), key=lambda item: item.vector_slot
        )
        if [item.vector_slot for item in results] != slots:
            raise RuntimeError("canonical reset order/count mismatch")
        for item in results:
            index = shard * self.slots_per_shard + item.vector_slot
            self.raw[index] = item.observation_raw
            self.ordinals[index] = item.episode_ordinal

    def _observations(self, raw: np.ndarray) -> TensorDict:
        values = (raw.astype(np.float64) / self.scales).astype(np.float32)
        if not np.isfinite(values).all():
            raise RuntimeError("non-finite canonical observation")
        return TensorDict(
            {"policy": torch.as_tensor(values, device=self.device)},
            batch_size=[self.num_envs],
        )

    def get_observations(self) -> TensorDict:
        return self._observations(self.raw)

    def step(self, actions: torch.Tensor):
        if tuple(actions.shape) != (self.num_envs, self.num_actions):
            raise ValueError("canonical action batch shape mismatch")
        # Validate the entire batch before advancing any process.
        raw_actions = normalized_action_to_raw(
            actions.detach().cpu().numpy(), 23, q1_30=True
        )
        futures = []
        for shard, client in enumerate(self.clients):
            part = slice(
                shard * self.slots_per_shard, (shard + 1) * self.slots_per_shard
            )
            futures.append(
                self.pool.submit(client.step, self.ordinals[part], raw_actions[part])
            )
        results_by_shard = [
            sorted(future.result(), key=lambda item: item.vector_slot)
            for future in futures
        ]
        results = []
        for shard, batch in enumerate(results_by_shard):
            if [item.vector_slot for item in batch] != list(
                range(self.slots_per_shard)
            ):
                raise RuntimeError("canonical step order/count mismatch")
            for item in batch:
                index = shard * self.slots_per_shard + item.vector_slot
                if item.episode_ordinal != self.ordinals[index]:
                    raise RuntimeError("canonical episode identity mismatch")
            results.extend(batch)
        self.last_steps = results
        self.raw[:] = np.stack([item.observation_raw for item in results])
        # Capture final observations before autoreset, never alias mutable raw state.
        terminal_observation = self.get_observations()
        rewards = torch.tensor(
            [item.reward_total_q16 / 65536 for item in results],
            dtype=torch.float32,
            device=self.device,
        )
        dones = torch.tensor(
            [item.terminated or item.truncated for item in results],
            dtype=torch.bool,
            device=self.device,
        )
        timeouts = torch.tensor(
            [item.truncated and not item.terminated for item in results],
            dtype=torch.bool,
            device=self.device,
        )
        self.episode_length_buf[:] = torch.tensor(
            [item.motor_tick for item in results], device=self.device
        )
        for shard, batch in enumerate(results_by_shard):
            slots = [
                item.vector_slot for item in batch if item.terminated or item.truncated
            ]
            if slots:
                self._reset(shard, slots)
        self.episode_length_buf[dones] = 0
        return (
            self.get_observations(),
            rewards,
            dones,
            {
                "time_outs": timeouts,
                "terminal_observation": terminal_observation,
            },
        )

    def close(self) -> None:
        self.pool.shutdown(wait=True)
        errors = []
        for client in self.clients:
            try:
                client.close()
            except Exception as error:  # noqa: BLE001 -- close every worker, then propagate
                errors.append(error)
        self.clients.clear()
        if errors:
            raise RuntimeError("canonical worker cleanup failed") from errors[0]


class TerminalObservationPPO(PPO):
    """Use V(final observation), not V(previous/reset observation), on time limits."""

    def process_env_step(self, obs, rewards, dones, extras):
        extras = dict(extras)
        timeouts = extras.pop("time_outs")
        final_obs = extras.pop("terminal_observation")
        with torch.no_grad():
            final_values = self.policy.evaluate(final_obs).squeeze(-1)
            corrected_rewards = rewards + self.gamma * final_values * timeouts
        super().process_env_step(obs, corrected_rewards, dones, extras)
