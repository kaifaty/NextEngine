import struct
import tempfile
import unittest
from pathlib import Path

import numpy as np

from next_lab.motor_lab_client import (
    EnvironmentDescriptor,
    FLAT_LOCOMOTION_PROFILE_ID,
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
    def test_normalized_adapter_clamps_and_uses_ties_to_even(self) -> None:
        values = np.asarray([[0.0000005, 0.0000015, -2.0]], dtype=np.float64)
        self.assertEqual(
            normalized_action_to_raw(values, action_width=3).tolist(),
            [[0, 2, -1_000_000]],
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
