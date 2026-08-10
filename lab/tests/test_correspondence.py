import unittest

import numpy as np

from next_lab.correspondence import evaluate_correspondence


def trajectory(offset: float = 0.0) -> dict[str, np.ndarray]:
    value = {
        "joint_position_rad": np.full((2, 3, 23), offset, dtype=np.float32),
        "root_position_m": np.full((2, 3, 3), offset, dtype=np.float32),
        "root_velocity_mps": np.full((2, 3, 3), offset, dtype=np.float32),
        "contact_occupancy": np.ones((2, 3, 2), dtype=np.bool_),
        "done_tick": np.array([3, 3], dtype=np.int64),
        "reward_total_q16": np.zeros((2, 3), dtype=np.int64),
        "command_raw": np.zeros((2, 3, 3), dtype=np.int64),
        "profile_id": np.asarray("nextengine.motor.env.humanoid-flat-command.v1"),
        "reward_component_ids": np.asarray([f"reward.{index}" for index in range(10)]),
    }
    for key in (
        "manifest_hash",
        "observation_layout_hash",
        "action_layout_hash",
        "command_schedule_profile_hash",
        "reward_profile_hash",
        "termination_profile_hash",
        "rng_derivation_profile_hash",
        "correspondence_profile_hash",
    ):
        value[key] = np.asarray("12" * 32)
    return value


class CorrespondenceTests(unittest.TestCase):
    def test_equal_trajectories_pass_all_gates(self) -> None:
        report = evaluate_correspondence(
            trajectory(), trajectory(), minimum_episodes=2, minimum_motor_steps=3
        )
        self.assertEqual(report["status"], "passed")
        self.assertTrue(all(report["gates"].values()))

    def test_excess_root_error_fails_without_relaxing_threshold(self) -> None:
        report = evaluate_correspondence(
            trajectory(), trajectory(0.06), minimum_episodes=2, minimum_motor_steps=3
        )
        self.assertEqual(report["status"], "failed")
        self.assertFalse(report["gates"]["root_position"])

    def test_sample_floor_is_mandatory(self) -> None:
        with self.assertRaisesRegex(ValueError, "sample floor"):
            evaluate_correspondence(trajectory(), trajectory())

    def test_command_and_profile_hashes_are_byte_exact_gates(self) -> None:
        gpu = trajectory()
        gpu["command_raw"] = gpu["command_raw"].copy()
        gpu["command_raw"][0, 0, 0] = 1
        report = evaluate_correspondence(
            trajectory(), gpu, minimum_episodes=2, minimum_motor_steps=3
        )
        self.assertEqual(report["status"], "failed")
        self.assertFalse(report["gates"]["exact:command_raw"])

    def test_reward_total_mae_threshold_is_fixed(self) -> None:
        gpu = trajectory()
        gpu["reward_total_q16"] = np.full((2, 3), 4_000, dtype=np.int64)
        report = evaluate_correspondence(
            trajectory(), gpu, minimum_episodes=2, minimum_motor_steps=3
        )
        self.assertEqual(report["status"], "failed")
        self.assertFalse(report["gates"]["reward_total"])


if __name__ == "__main__":
    unittest.main()
