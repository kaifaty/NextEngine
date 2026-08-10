import unittest

import numpy as np

from next_lab.correspondence import evaluate_correspondence


def trajectory(offset: float = 0.0) -> dict[str, np.ndarray]:
    return {
        "joint_position_rad": np.full((2, 3, 23), offset, dtype=np.float32),
        "root_position_m": np.full((2, 3, 3), offset, dtype=np.float32),
        "root_velocity_mps": np.full((2, 3, 3), offset, dtype=np.float32),
        "contact_occupancy": np.ones((2, 3, 2), dtype=np.bool_),
        "done_tick": np.array([3, 3], dtype=np.int64),
    }


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


if __name__ == "__main__":
    unittest.main()
