from __future__ import annotations

import unittest

import torch

from next_lab.isaac_env import (
    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
    engine_vector_from_isaac_tensor,
)
from next_lab.isaac_reference_env import (
    _engine_to_isaac_vector,
    _engine_xyzw_to_isaac_wxyz,
    _quaternion_conjugate_xyzw,
    _quaternion_multiply_xyzw,
    _rotate_inverse_xyzw,
)


class IsaacReferenceEnvironmentTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
