from __future__ import annotations

import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np
import torch
from next_lab.canonical_ppo import (
    CanonicalVecEnv,
    TerminalObservationPPO,
    observation_scales,
    shard_root,
)
from rsl_rl.algorithms import PPO
from rsl_rl.modules import ActorCritic
from tensordict import TensorDict


def descriptor(width=23):
    hashes = {
        name: "12" * 32
        for name in (
            "manifest_hash",
            "observation_layout_hash",
            "action_layout_hash",
            "command_schedule_profile_hash",
            "reward_profile_hash",
            "termination_profile_hash",
            "rng_derivation_profile_hash",
            "correspondence_profile_hash",
        )
    }
    return {
        "joints": [
            {
                "joint_id": str(i),
                "soft_limit_microradians": [-100, 200],
                "maximum_velocity_microradians_per_second": 1000,
            }
            for i in range(width)
        ],
        "actuators": [{"joint_id": str(i)} for i in range(width)],
        "observation_width": 15 + 3 * width,
        "action_width": width,
        "physics_hz": 240,
        "motor_hz": 60,
        "environment_profiles": [
            {
                **hashes,
                "profile_id": "test.forward-start-stop.v5",
                "maximum_episode_steps": 1200,
            }
        ],
    }


class FakeClient:
    action_width = 23

    def __init__(self, executable, profile, slots, root):
        data = descriptor(self.action_width)
        expected = data["environment_profiles"][0]
        self.descriptor = SimpleNamespace(
            **{
                key: bytes.fromhex(value) if key.endswith("hash") else value
                for key, value in expected.items()
            },
            slots=slots,
            observation_width=data["observation_width"],
            action_width=self.action_width,
            physics_hz=240,
            motor_hz=60,
        )
        self.ordinals = [-1] * slots
        self.calls = 0
        self.reset_calls = []
        self.closed = False

    def reset(self, slots):
        self.reset_calls.append(slots)
        for slot in slots:
            self.ordinals[slot] += 1
        return [
            SimpleNamespace(
                vector_slot=slot,
                episode_ordinal=self.ordinals[slot],
                observation_raw=np.full(15 + 3 * self.action_width, -50),
            )
            for slot in slots
        ]

    def step(self, ordinals, actions):
        self.calls += 1
        self.actions = actions.copy()
        return [
            SimpleNamespace(
                vector_slot=slot,
                episode_ordinal=int(ordinal),
                observation_raw=np.full(15 + 3 * self.action_width, 100 + slot),
                reward_total_q16=32768,
                motor_tick=1,
                terminated=slot == 0,
                truncated=slot == 1,
            )
            for slot, ordinal in enumerate(ordinals)
        ]

    def close(self):
        self.closed = True


class CanonicalAdapterTests(unittest.TestCase):
    def test_descriptor_width_order_and_references_fail_before_worker_creation(self):
        mutations = [
            lambda d: d.update(action_width=24),
            lambda d: d.update(action_width=True),
            lambda d: d.update(action_width=0),
            lambda d: d.update(observation_width=85),
            lambda d: d["joints"].append(d["joints"][0].copy()),
            lambda d: d["actuators"][-1].update(joint_id="missing"),
            lambda d: d["actuators"][-1].update(joint_id="0"),
            lambda d: d["environment_profiles"][0].update(action_width=25),
            lambda d: d["environment_profiles"][0].update(observation_width=90),
        ]
        for mutate in mutations:
            data = descriptor()
            mutate(data)
            with (
                self.subTest(data=data),
                patch("next_lab.canonical_ppo.MotorLabClient") as factory,
            ):
                with self.assertRaises(ValueError):
                    self.make_width_env(data, factory)
                factory.assert_not_called()

    def make_width_env(self, data, factory):
        return CanonicalVecEnv(
            Path("unused"),
            data,
            num_envs=2,
            shards=1,
            run_root="00" * 32,
            device="cpu",
            client_factory=factory,
        )

    def test_25_action_layout_preserves_order_and_rejects_old_action_batches(self):
        class WideClient(FakeClient):
            action_width = 25

        data = descriptor(25)
        # Each channel has a distinguishable scale; descriptor actuator order owns it.
        data["actuators"].reverse()
        for i, joint in enumerate(data["joints"]):
            joint["soft_limit_microradians"] = [-100, 200 + i]
            joint["maximum_velocity_microradians_per_second"] = 1000 + i
        env = self.make_width_env(data, WideClient)
        try:
            self.assertEqual(env.num_actions, 25)
            self.assertEqual(env.raw.shape, (2, 90))
            np.testing.assert_array_equal(env.scales[10:35], np.arange(224, 199, -1))
            np.testing.assert_array_equal(env.scales[35:60], np.arange(1024, 999, -1))
            np.testing.assert_array_equal(env.scales[60:85], env.scales[10:35])
            with self.assertRaisesRegex(ValueError, "shape"):
                env.step(torch.zeros(2, 23))
            bad = torch.zeros(2, 25)
            bad[-1, -1] = torch.nan
            with self.assertRaisesRegex(ValueError, "finite"):
                env.step(bad)
            self.assertEqual(env.clients[0].calls, 0)
            action = torch.arange(25, dtype=torch.float32).repeat(2, 1) / 32
            _, _, _, extras = env.step(action)
            np.testing.assert_array_equal(
                env.clients[0].actions, np.tile(np.arange(25) * (1 << 25), (2, 1))
            )
            self.assertEqual(extras["time_outs"].tolist(), [False, True])
            self.assertEqual(env.ordinals.tolist(), [1, 1])
            self.assertTrue(torch.all(extras["terminal_observation"]["policy"] > 0))
        finally:
            env.close()

    def test_25_action_descriptor_rejects_23_action_native_handshake(self):
        clients = []

        def factory(*args):
            client = FakeClient(*args)
            clients.append(client)
            return client

        with self.assertRaisesRegex(ValueError, "descriptor mismatch"):
            self.make_width_env(descriptor(25), factory)
        self.assertTrue(clients[0].closed)
        self.assertEqual(clients[0].reset_calls, [])

    def test_25_action_sole_offsets_follow_joint_blocks(self):
        data = descriptor(25)
        data["observation_width"] = 94
        data["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v8"
        scales = observation_scales(data)
        self.assertEqual(scales.shape, (94,))
        np.testing.assert_array_equal(scales[90:], [1 << 30, 1 << 30, 100_000, 100_000])

    def test_short_reset_observation_cannot_broadcast_over_25_joints(self):
        clients = []

        class ShortClient(FakeClient):
            action_width = 25

            def __init__(self, *args):
                super().__init__(*args)
                clients.append(self)

            def reset(self, slots):
                results = super().reset(slots)
                results[-1].observation_raw = np.zeros(1, dtype=np.int64)
                return results

        with self.assertRaisesRegex(RuntimeError, "observation shape"):
            self.make_width_env(descriptor(25), ShortClient)
        self.assertTrue(clients[0].closed)

    def test_short_step_observation_preserves_last_valid_adapter_state(self):
        class ShortClient(FakeClient):
            action_width = 25

            def step(self, ordinals, actions):
                results = super().step(ordinals, actions)
                for result in results:
                    result.observation_raw = np.zeros(1, dtype=np.int64)
                return results

        env = self.make_width_env(descriptor(25), ShortClient)
        try:
            before = env.raw.copy()
            with self.assertRaisesRegex(RuntimeError, "observation shape"):
                env.step(torch.zeros(2, 25))
            np.testing.assert_array_equal(env.raw, before)
            self.assertEqual(env.last_steps, [])
            self.assertEqual(len(env.clients[0].reset_calls), 1)
        finally:
            env.close()

    def test_lift_return_height_scales_preserve_periodic_channels(self):
        old = descriptor()
        old["observation_width"] = 86
        old["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v6"
        new = descriptor()
        new["observation_width"] = 88
        new["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v7"
        np.testing.assert_array_equal(
            observation_scales(old), observation_scales(new)[:86]
        )
        np.testing.assert_array_equal(observation_scales(new)[86:], [100_000] * 2)
        new["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v6"
        with self.assertRaisesRegex(ValueError, "V7"):
            observation_scales(new)

    def test_lift_return_heights_survive_terminal_observation_before_reset(self):
        for version in (7, 8, 9):
            for width in (23, 25):
                with self.subTest(version=version, width=width):
                    self.check_lift_return_terminal_observation(version, width)

    def check_lift_return_terminal_observation(self, version, width):
        height_offset = 17 + 3 * width

        class LiftClient(FakeClient):
            action_width = width

            def __init__(self, *args):
                super().__init__(*args)
                self.descriptor.observation_width = height_offset + 2

            def reset(self, slots):
                results = super().reset(slots)
                for result in results:
                    result.observation_raw = np.concatenate(
                        (result.observation_raw, [0, 0, 0, 0])
                    )
                return results

            def step(self, ordinals, actions):
                results = super().step(ordinals, actions)
                for result in results:
                    result.observation_raw = np.concatenate(
                        (result.observation_raw, [0, 0, 60_000, -10])
                    )
                return results

        data = descriptor(width)
        data["observation_width"] = height_offset + 2
        data["environment_profiles"][0]["profile_id"] = (
            f"test.forward-start-stop.v{version}"
        )
        env = CanonicalVecEnv(
            Path("unused"),
            data,
            num_envs=2,
            shards=1,
            run_root="00" * 32,
            device="cpu",
            client_factory=LiftClient,
        )
        try:
            self.assertEqual(env.sole_height_offset, height_offset)
            obs, _, _, extras = env.step(torch.zeros(2, width))
            np.testing.assert_array_equal(
                obs["policy"][:, height_offset:], np.zeros((2, 2))
            )
            np.testing.assert_allclose(
                extras["terminal_observation"]["policy"][:, height_offset:],
                [[0.6, -0.0001], [0.6, -0.0001]],
            )
            self.assertEqual(extras["time_outs"].tolist(), [False, True])
        finally:
            env.close()

    def test_periodic_clock_scales_preserve_all_legacy_channels(self):
        old = descriptor()
        new = descriptor()
        new["observation_width"] = 86
        new["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v6"
        np.testing.assert_array_equal(
            observation_scales(old), observation_scales(new)[:84]
        )
        np.testing.assert_array_equal(observation_scales(new)[84:], [1 << 30] * 2)
        new["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v5"
        with self.assertRaisesRegex(ValueError, "V6"):
            observation_scales(new)

    def test_periodic_clock_survives_reset_and_final_observation(self):
        class PeriodicClient(FakeClient):
            def __init__(self, *args):
                super().__init__(*args)
                self.descriptor.observation_width = 86

            def reset(self, slots):
                results = super().reset(slots)
                for result in results:
                    result.observation_raw = np.concatenate(
                        (result.observation_raw, [0, 0])
                    )
                return results

            def step(self, ordinals, actions):
                results = super().step(ordinals, actions)
                for result in results:
                    result.observation_raw = np.concatenate(
                        (result.observation_raw, [1 << 30, -(1 << 30)])
                    )
                return results

        data = descriptor()
        data["observation_width"] = 86
        data["environment_profiles"][0]["profile_id"] = "test.forward-start-stop.v6"
        env = CanonicalVecEnv(
            Path("unused"),
            data,
            num_envs=2,
            shards=1,
            run_root="00" * 32,
            device="cpu",
            client_factory=PeriodicClient,
        )
        try:
            obs, _, _, extras = env.step(torch.zeros(2, 23))
            np.testing.assert_array_equal(obs["policy"][:, 84:], np.zeros((2, 2)))
            np.testing.assert_array_equal(
                extras["terminal_observation"]["policy"][:, 84:], [[1, -1], [1, -1]]
            )
            self.assertEqual(extras["time_outs"].tolist(), [False, True])
        finally:
            env.close()

    def test_float_swing_mirroring_preserves_nonzero_targets(self):
        from next_lab.walking_action_basis import walking_action_basis_tape

        from lab.scripts.cpu_walking_alternation_probe import mirror_target

        target = walking_action_basis_tape()[1, 104].astype(np.float64) / (1 << 30)
        mirrored = mirror_target(target)
        self.assertEqual(np.count_nonzero(target), np.count_nonzero(mirrored))
        self.assertEqual(mirrored[10], target[0])
        np.testing.assert_array_equal(mirror_target(mirrored), target)

    def make_env(self, factory=FakeClient):
        return CanonicalVecEnv(
            Path("unused"),
            descriptor(),
            num_envs=4,
            shards=2,
            run_root="00" * 32,
            device="cpu",
            client_factory=factory,
        )

    def test_terminal_state_survives_autoreset_and_only_timeout_bootstraps(self):
        env = self.make_env()
        try:
            obs, reward, done, extras = env.step(torch.zeros(4, 23))
            np.testing.assert_array_equal(env.raw, -50 * np.ones((4, 84)))
            self.assertTrue(torch.all(obs["policy"] < 0))
            self.assertTrue(torch.all(extras["terminal_observation"]["policy"] > 0))
            self.assertEqual(extras["time_outs"].tolist(), [False, True, False, True])
            self.assertEqual(done.tolist(), [True] * 4)
            self.assertEqual(reward.tolist(), [0.5] * 4)
            self.assertEqual(env.ordinals.tolist(), [1] * 4)
        finally:
            env.close()

    def test_nonfinite_batch_rejected_before_any_worker_advances(self):
        env = self.make_env()
        try:
            action = torch.zeros(4, 23)
            action[-1, -1] = torch.nan
            with self.assertRaisesRegex(ValueError, "finite"):
                env.step(action)
            self.assertEqual([client.calls for client in env.clients], [0, 0])
            env.step(torch.full((4, 23), 2.0))
            for client in env.clients:
                np.testing.assert_array_equal(client.actions, np.full((2, 23), 1 << 30))
        finally:
            env.close()

    def test_descriptor_mismatch_closes_created_worker(self):
        clients = []

        def bad_factory(*args):
            client = FakeClient(*args)
            client.descriptor.reward_profile_hash = bytes(32)
            clients.append(client)
            return client

        with self.assertRaisesRegex(ValueError, "reward_profile_hash"):
            self.make_env(bad_factory)
        self.assertTrue(clients[0].closed)

    def test_response_order_is_not_assumed_to_be_slot_order(self):
        class ReorderedClient(FakeClient):
            def reset(self, slots):
                return list(reversed(super().reset(slots)))

            def step(self, ordinals, actions):
                return list(reversed(super().step(ordinals, actions)))

        env = self.make_env(ReorderedClient)
        try:
            env.step(torch.zeros(4, 23))
            self.assertEqual(
                [item.vector_slot for item in env.last_steps], [0, 1, 0, 1]
            )
            self.assertEqual(
                [int(item.observation_raw[0]) for item in env.last_steps],
                [100, 101, 100, 101],
            )
        finally:
            env.close()

    def test_duplicate_response_slot_is_rejected(self):
        class DuplicateClient(FakeClient):
            def step(self, ordinals, actions):
                values = super().step(ordinals, actions)
                values[1].vector_slot = 0
                return values

        env = self.make_env(DuplicateClient)
        try:
            with self.assertRaisesRegex(RuntimeError, "order/count"):
                env.step(torch.zeros(4, 23))
        finally:
            env.close()

    def test_shard_partition_is_repeatable_and_distinct(self):
        self.assertEqual(shard_root("ab" * 32, 0), shard_root("ab" * 32, 0))
        self.assertNotEqual(shard_root("ab" * 32, 0), shard_root("ab" * 32, 1))

    def test_ppo_uses_final_value_and_never_changes_reported_reward(self):
        algorithm = object.__new__(TerminalObservationPPO)
        algorithm.gamma = 0.9
        algorithm.policy = SimpleNamespace(evaluate=lambda obs: obs["policy"])
        reset = TensorDict({"policy": torch.tensor([[-100.0], [-200.0]])}, [2])
        final = TensorDict({"policy": torch.tensor([[10.0], [20.0]])}, [2])
        reward = torch.tensor([1.0, 2.0])
        extras = {
            "time_outs": torch.tensor([True, False]),
            "terminal_observation": final,
        }
        with patch.object(PPO, "process_env_step") as parent:
            algorithm.process_env_step(reset, reward, torch.ones(2), extras)
        self.assertEqual(parent.call_args.args[1].tolist(), [10.0, 2.0])
        self.assertEqual(reward.tolist(), [1.0, 2.0])
        self.assertNotIn("time_outs", parent.call_args.args[3])
        self.assertIn("time_outs", extras)

    def test_installed_ppo_can_update_with_adapter_contract(self):
        self.check_ppo_update(self.make_env())

    def test_installed_ppo_can_update_with_25_action_adapter(self):
        class WideClient(FakeClient):
            action_width = 25

        self.check_ppo_update(self.make_width_env(descriptor(25), WideClient))

    def check_ppo_update(self, env):
        try:
            obs = env.get_observations()
            policy = ActorCritic(
                obs,
                {"policy": ["policy"], "critic": ["policy"]},
                env.num_actions,
                actor_hidden_dims=[8],
                critic_hidden_dims=[8],
            )
            algorithm = TerminalObservationPPO(
                policy, device="cpu", num_learning_epochs=1, num_mini_batches=1
            )
            algorithm.init_storage("rl", env.num_envs, 2, obs, [env.num_actions])
            with torch.inference_mode():
                for _ in range(2):
                    obs, reward, done, extras = env.step(algorithm.act(obs))
                    algorithm.process_env_step(obs, reward, done, extras)
                algorithm.compute_returns(obs)
            losses = algorithm.update()
            self.assertTrue(all(np.isfinite(value) for value in losses.values()))
        finally:
            env.close()


if __name__ == "__main__":
    unittest.main()
