import struct
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock

import numpy as np
from next_lab.motor_lab_client import (
    FLAT_LOCOMOTION_PROFILE_ID,
    EnvironmentDescriptor,
    MotorLabClient,
    MotorLabProtocolError,
    RewardComponent,
    StepResult,
    normalized_action_to_raw,
)
from next_lab.trajectory_recorder import (
    _write_npz_v2,
    require_external_training_store,
)


class MotorLabClientTests(unittest.TestCase):
    def test_step_parser_uses_advertised_joint_width_and_rejects_old_tail(self):
        def payload(width, tail_width):
            data = bytearray(struct.pack("<IQIQ", 1, 1, 0, 1))
            data.extend(bytes(6 * 8))

            def vector(values):
                data.extend(struct.pack("<I", len(values)))
                data.extend(np.asarray(values, dtype="<i8").tobytes())

            vector(np.zeros(width, dtype=np.int64))
            vector(np.arange(19 + 3 * width))
            data.extend(struct.pack("<IqBBB", 0, 0, 0, 0, 0))
            data.extend(bytes(3 * 32 + 13 * 8))
            vector(np.arange(10, 10 + tail_width))
            vector(np.arange(10 + width, 10 + width + tail_width))
            vector([1, 0])
            return bytes(data)

        for width in (23, 25):
            client = object.__new__(MotorLabClient)
            client.descriptor = SimpleNamespace(
                maximum_slots=1,
                action_width=width,
                observation_width=19 + 3 * width,
                reward_components=(),
            )
            result = client._parse_steps(payload(width, width))[0]
            np.testing.assert_array_equal(
                result.joint_position_microradians,
                result.observation_raw[10 : 10 + width],
            )
            np.testing.assert_array_equal(
                result.joint_velocity_microradians_per_second,
                result.observation_raw[10 + width : 10 + 2 * width],
            )
            with self.assertRaisesRegex(MotorLabProtocolError, "JOINT_TELEMETRY_WIDTH"):
                client._parse_steps(payload(width, width - 2))

    def test_periodic_walking_actions_are_q1_30_not_microradians(self):
        for version in (6, 7, 8, 9):
            with self.subTest(version=version):
                width = 25 if version == 9 else 23
                client = object.__new__(MotorLabClient)
                client.descriptor = SimpleNamespace(
                    profile_id=f"nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v{version}",
                    action_width=width,
                )
                client.step = Mock(return_value=[])
                client.step_normalized([1], np.full((1, width), 0.5))
                np.testing.assert_array_equal(
                    client.step.call_args.args[1], np.full((1, width), 1 << 29)
                )

    def test_normalized_adapter_clamps_and_uses_ties_to_even(self) -> None:
        values = np.asarray([[0.0000005, 0.0000015, -2.0]], dtype=np.float64)
        self.assertEqual(
            normalized_action_to_raw(values, action_width=3).tolist(),
            [[0, 2, -1_000_000]],
        )
        self.assertEqual(
            normalized_action_to_raw(
                [[0.5, -2.0]], action_width=2, q1_30=True
            ).tolist(),
            [[1 << 29, -(1 << 30)]],
        )
        with self.assertRaisesRegex(ValueError, "finite"):
            normalized_action_to_raw([[float("nan")]], action_width=1)

    def test_descriptor_parser_rejects_trailing_bytes(self) -> None:
        payload = bytearray()
        profile = FLAT_LOCOMOTION_PROFILE_ID.encode()
        payload.extend(struct.pack("<I", len(profile)))
        payload.extend(profile)
        payload.extend(bytes(range(32)) * 8)
        payload.extend(struct.pack("<IIQIIII", 1, 256, 1200, 240, 60, 84, 23))
        payload.extend(struct.pack("<I", 1))
        reward = b"reward.test"
        payload.extend(struct.pack("<I", len(reward)))
        payload.extend(reward)
        payload.extend(struct.pack("<qqq", 65_536, 0, 65_536))
        client = object.__new__(MotorLabClient)
        descriptor = client._parse_descriptor(bytes(payload))
        self.assertEqual(descriptor.observation_width, 84)
        with self.assertRaisesRegex(MotorLabProtocolError, "TRAILING"):
            client._parse_descriptor(bytes(payload) + b"x")

    def test_npz_v2_contains_raw_authority_facts_in_external_store(self) -> None:
        descriptor = EnvironmentDescriptor(
            profile_id=FLAT_LOCOMOTION_PROFILE_ID,
            manifest_hash=b"a" * 32,
            observation_layout_hash=b"b" * 32,
            action_layout_hash=b"c" * 32,
            command_schedule_profile_hash=b"d" * 32,
            reward_profile_hash=b"e" * 32,
            termination_profile_hash=b"f" * 32,
            rng_derivation_profile_hash=b"g" * 32,
            correspondence_profile_hash=b"h" * 32,
            slots=1,
            maximum_slots=256,
            maximum_episode_steps=1200,
            physics_hz=240,
            motor_hz=60,
            observation_width=84,
            action_width=23,
            reward_components=(RewardComponent("reward.test", 65_536, 0, 65_536),),
        )
        row = StepResult(
            episode_ordinal=1,
            vector_slot=0,
            motor_tick=1,
            command_raw=np.zeros(3, dtype=np.int64),
            next_command_raw=np.zeros(3, dtype=np.int64),
            applied_action_raw=np.zeros(23, dtype=np.int64),
            observation_raw=np.zeros(84, dtype=np.int64),
            reward_components_raw=np.zeros(1, dtype=np.int64),
            reward_total_q16=0,
            terminated=False,
            truncated=True,
            terminal_reason_id="terminal.timeout",
            physics_root=b"i" * 32,
            motor_root=b"j" * 32,
            step_root=b"k" * 32,
            root_position_micrometres=np.zeros(3, dtype=np.int64),
            root_quaternion_q1_30_xyzw=np.zeros(4, dtype=np.int64),
            root_linear_velocity_micrometres_per_second=np.zeros(3, dtype=np.int64),
            root_angular_velocity_microradians_per_second=np.zeros(3, dtype=np.int64),
            joint_position_microradians=np.zeros(23, dtype=np.int64),
            joint_velocity_microradians_per_second=np.zeros(23, dtype=np.int64),
            contact_flags=np.zeros(2, dtype=np.bool_),
        )
        with tempfile.TemporaryDirectory() as directory:
            path = _write_npz_v2(
                Path(directory), "fixture.npz", "01" * 32, descriptor, [row]
            )
            with np.load(path, allow_pickle=False) as archive:
                self.assertEqual(int(archive["schema_version"]), 2)
                self.assertEqual(archive["observation_raw"].shape, (1, 84))
                self.assertEqual(archive["joint_position_microradians"].shape, (1, 23))
                self.assertTrue(bool(archive["truncated"][0]))

    def test_repository_is_never_an_artifact_store(self) -> None:
        repository_root = Path(__file__).resolve().parents[2]
        with self.assertRaisesRegex(ValueError, "outside"):
            require_external_training_store(repository_root / "training-runs")


if __name__ == "__main__":
    unittest.main()
