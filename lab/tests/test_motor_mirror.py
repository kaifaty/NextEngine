from pathlib import Path
import unittest

import torch

from next_lab.isaac_env import (
    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
    engine_vector_from_isaac_tensor,
    fixed_pd_tensor,
    isaac_prim_name,
    isaac_root_state_from_descriptor,
    locomotion_reward_q16_tensor,
    precompute_flat_command_schedules,
    rotate_world_to_root_local_q1_30_tensor,
    round_div_ties_even_tensor,
)
from next_lab.motor_mirror import (
    derive_purpose_seed,
    flat_locomotion_command_schedule,
    load_json,
    rotate_world_to_root_local_q1_30,
    validate_golden,
)


FIXTURE = Path(__file__).parent / "fixtures/stage0_motor_mirror_v2.json"


class MotorMirrorTests(unittest.TestCase):
    def test_semantic_joint_id_maps_to_usd_prim_name(self) -> None:
        self.assertEqual(isaac_prim_name("joint.left-ankle-roll"), "joint_left_ankle_roll")

    def test_rust_golden_matches_python_seed_and_pd(self) -> None:
        validate_golden(load_json(FIXTURE))

    def test_torch_integer_pd_matches_rust_golden(self) -> None:
        target = torch.full((2, 23), 1_000_000, dtype=torch.int64)
        zero = torch.zeros_like(target)
        first, flags = fixed_pd_tensor(target, zero, zero, zero)
        second, second_flags = fixed_pd_tensor(target, zero, zero, first)
        self.assertTrue(torch.all(first == 25_000_000))
        self.assertTrue(torch.all(second == 50_000_000))
        self.assertTrue(torch.all(flags == 4))
        self.assertTrue(torch.all(second_flags == 4))

    def test_tensor_ties_to_even_is_sign_symmetric(self) -> None:
        values = torch.tensor([5, 7, -5, -7], dtype=torch.int64)
        actual = round_div_ties_even_tensor(values, 2)
        torch.testing.assert_close(actual, torch.tensor([2, 4, -2, -4], dtype=torch.int64))

    def test_torch_root_local_transform_matches_python_golden(self) -> None:
        golden = load_json(FIXTURE)["root_local_transform"]
        quaternion = torch.tensor([golden["quaternion_xyzw_q1_30"]], dtype=torch.int64)
        vector = torch.tensor([golden["world_vector_raw"]], dtype=torch.int64)
        actual = rotate_world_to_root_local_q1_30_tensor(quaternion, vector)
        self.assertEqual(actual[0].tolist(), golden["root_local_vector_raw"])
        self.assertEqual(
            rotate_world_to_root_local_q1_30(quaternion[0].tolist(), vector[0].tolist()),
            tuple(golden["root_local_vector_raw"]),
        )

    def test_partial_schedule_precomputation_is_slot_and_ordinal_bound(self) -> None:
        run_root = bytes(range(32))
        actual = precompute_flat_command_schedules(run_root, [17, 5], [3, 9])
        self.assertEqual(actual.shape, (2, 1_201, 3))
        seed = derive_purpose_seed(run_root, 17, 3, "randomization.command")
        self.assertEqual(actual[0].tolist(), [list(row) for row in flat_locomotion_command_schedule(seed)])
        self.assertFalse(torch.equal(actual[0], actual[1]))

    def test_isaac_coordinate_and_quaternion_ordering_are_explicit(self) -> None:
        vector = engine_vector_from_isaac_tensor(torch.tensor([[1.0, 2.0, 3.0]]))
        torch.testing.assert_close(vector, torch.tensor([[1.0, 3.0, -2.0]]))
        quaternion = engine_quaternion_xyzw_from_isaac_wxyz_tensor(
            torch.tensor([[4.0, 1.0, 2.0, 3.0]])
        )
        torch.testing.assert_close(quaternion, torch.tensor([[1.0, 3.0, -2.0, 4.0]]))

        root_state = isaac_root_state_from_descriptor(
            {
                "bodies": [
                    {
                        "parent_body_id": None,
                        "local_bind_translation_micrometres": [
                            1_000_000,
                            2_000_000,
                            3_000_000,
                        ],
                        "local_bind_rotation_q1_30": [
                            1 << 29,
                            1 << 28,
                            -(1 << 27),
                            1 << 30,
                        ],
                    }
                ]
            }
        )
        self.assertEqual(root_state[:7], (1.0, -3.0, 2.0, 1.0, 0.5, 0.125, 0.25))
        self.assertEqual(root_state[7:], (0.0,) * 6)

    def test_locomotion_reward_uses_applied_action_and_is_yaw_invariant(self) -> None:
        half_sqrt_q30 = 759_250_125
        quaternions = torch.tensor(
            [[0, 0, 0, 1 << 30], [0, half_sqrt_q30, 0, half_sqrt_q30]],
            dtype=torch.int64,
        )
        zeros3 = torch.zeros((2, 3), dtype=torch.int64)
        zeros23 = torch.zeros((2, 23), dtype=torch.int64)
        components, _ = locomotion_reward_q16_tensor(
            quaternion_xyzw_q1_30=quaternions,
            root_height_micrometres=torch.full((2,), 1_050_000, dtype=torch.int64),
            local_linear_velocity_raw=zeros3,
            local_angular_velocity_raw=zeros3,
            vertical_velocity_raw=torch.zeros(2, dtype=torch.int64),
            command_raw=zeros3,
            effort_sum_raw=torch.zeros(2, dtype=torch.int64),
            applied_action_raw=zeros23,
            previous_applied_action_raw=zeros23,
            contacting_foot_slip_sum_raw=torch.zeros(2, dtype=torch.int64),
            contacting_foot_count=torch.zeros(2, dtype=torch.int64),
            fell=torch.zeros(2, dtype=torch.bool),
        )
        self.assertEqual(components[0, 2].item(), components[1, 2].item())
        self.assertEqual(components[0, 7].item(), 0)

        applied = zeros23.clone()
        applied[:, 0] = 1_000_000
        rate_components, _ = locomotion_reward_q16_tensor(
            quaternion_xyzw_q1_30=quaternions,
            root_height_micrometres=torch.full((2,), 1_050_000, dtype=torch.int64),
            local_linear_velocity_raw=zeros3,
            local_angular_velocity_raw=zeros3,
            vertical_velocity_raw=torch.zeros(2, dtype=torch.int64),
            command_raw=zeros3,
            effort_sum_raw=torch.zeros(2, dtype=torch.int64),
            applied_action_raw=applied,
            previous_applied_action_raw=zeros23,
            contacting_foot_slip_sum_raw=torch.zeros(2, dtype=torch.int64),
            contacting_foot_count=torch.zeros(2, dtype=torch.int64),
            fell=torch.zeros(2, dtype=torch.bool),
        )
        self.assertGreater(rate_components[0, 7].item(), 0)


if __name__ == "__main__":
    unittest.main()
