from __future__ import annotations

import unittest

import torch

from next_lab.isaac_env import (
    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
    engine_vector_from_isaac_tensor,
)
from next_lab.isaac_reference_env import (
    _advance_contact_grace,
    _engine_to_isaac_vector,
    _engine_xyzw_to_isaac_wxyz,
    _normalized_xyzw,
    _quaternion_conjugate_xyzw,
    _quaternion_multiply_xyzw,
    _rotate_inverse_xyzw,
    _select_curriculum_episode,
)


class IsaacReferenceEnvironmentTests(unittest.TestCase):
    def test_invalid_quaternion_still_fails_synchronously_on_cpu(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "invalid reference quaternion"):
            _normalized_xyzw(torch.zeros((1, 4)))

    def test_engine_isaac_vector_and_quaternion_mappings_are_inverses(self) -> None:
        vector = torch.tensor([[1.25, -2.5, 3.75]], dtype=torch.float64)
        torch.testing.assert_close(
            engine_vector_from_isaac_tensor(_engine_to_isaac_vector(vector)), vector
        )
        quaternion = torch.tensor([[0.1, -0.2, 0.3, 0.92736185]], dtype=torch.float64)
        quaternion /= torch.linalg.vector_norm(quaternion, dim=-1, keepdim=True)
        torch.testing.assert_close(
            engine_quaternion_xyzw_from_isaac_wxyz_tensor(
                _engine_xyzw_to_isaac_wxyz(quaternion)
            ),
            quaternion,
        )

    def test_root_inverse_rotation_and_relative_identity_are_closed(self) -> None:
        half = 2.0**-0.5
        yaw_quaternion = torch.tensor([[0.0, half, 0.0, half]], dtype=torch.float64)
        forward_world = torch.tensor([[1.0, 0.0, 0.0]], dtype=torch.float64)
        local = _rotate_inverse_xyzw(forward_world, yaw_quaternion)
        torch.testing.assert_close(
            local,
            torch.tensor([[0.0, 0.0, 1.0]], dtype=torch.float64),
            atol=1.0e-12,
            rtol=0.0,
        )
        identity = _quaternion_multiply_xyzw(
            _quaternion_conjugate_xyzw(yaw_quaternion), yaw_quaternion
        )
        torch.testing.assert_close(
            identity,
            torch.tensor([[0.0, 0.0, 0.0, 1.0]], dtype=torch.float64),
            atol=1.0e-12,
            rtol=0.0,
        )

    def test_curriculum_episode_selection_is_deterministic_and_bounded(self) -> None:
        root = bytes.fromhex("12" * 32)
        selections = [
            _select_curriculum_episode(
                run_root=root,
                episode_ordinal=episode,
                vector_slot=slot,
                clip_frame_counts=(90, 120, 180),
                horizon_motor_ticks=32,
            )
            for episode in range(3)
            for slot in range(8)
        ]
        self.assertEqual(
            selections[0],
            _select_curriculum_episode(
                run_root=root,
                episode_ordinal=0,
                vector_slot=0,
                clip_frame_counts=(90, 120, 180),
                horizon_motor_ticks=32,
            ),
        )
        self.assertGreater(len(set(selections)), 1)
        for clip_index, start_frame, terminal_frame in selections:
            self.assertIn(clip_index, range(3))
            self.assertGreaterEqual(start_frame, 0)
            self.assertEqual(terminal_frame - start_frame, 32)
            self.assertLess(terminal_frame, (90, 120, 180)[clip_index])

    def test_forbidden_contact_grace_expires_after_declared_physics_substeps(
        self,
    ) -> None:
        accumulated = torch.zeros(3, dtype=torch.int64)
        raw = torch.tensor([True, False, True])
        accumulated, terminal = _advance_contact_grace(
            accumulated,
            raw,
            physics_substeps_per_motor_tick=4,
            grace_physics_substeps=4,
        )
        torch.testing.assert_close(accumulated, torch.tensor([4, 0, 4]))
        self.assertFalse(torch.any(terminal))
        accumulated, terminal = _advance_contact_grace(
            accumulated,
            torch.tensor([True, True, False]),
            physics_substeps_per_motor_tick=4,
            grace_physics_substeps=4,
        )
        torch.testing.assert_close(accumulated, torch.tensor([8, 4, 0]))
        torch.testing.assert_close(terminal, torch.tensor([True, False, False]))


if __name__ == "__main__":
    unittest.main()
