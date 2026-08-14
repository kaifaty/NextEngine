from __future__ import annotations

import copy
import io
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.motion_math import quaternion_q1_30, rotation_axis
from next_lab.quantization_aware_kto_execution import (
    _serialize_cache,
    _validate_profile,
)
from next_lab.quantization_aware_kto_solver import (
    ACCELERATION_OFFSET,
    BLOCK_WIDTH,
    FRAME_COUNT,
    JOINT_COUNT,
    Q_WIDTH,
    KtoLinearization,
    KtoState,
    _array_hashes,
    _build_problem,
    _derive_velocity_acceleration,
    _effective_joint_limits,
    _lift_root_yaw,
    _rotation_exp,
    _rotation_log,
)

ROOT = Path(__file__).resolve().parents[2]
EXECUTION_PROFILE = (
    ROOT / "lab/profiles/humanoid-quantization-aware-kto-execution-r115.v1.json"
)
R114_PROFILE = (
    ROOT
    / "lab/profiles/humanoid-quantization-aware-kto-execution-formulation-r114.v1.json"
)
V9_PROFILE = ROOT / "lab/profiles/humanoid-contact-manifold-prototype.v9.json"
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class QuantizationAwareKtoExecutionTests(unittest.TestCase):
    def test_profile_allows_one_solve_and_no_downstream_execution(self) -> None:
        profile = json.loads(EXECUTION_PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertEqual(profile["scope"]["kto_solve_count"], 1)
        self.assertEqual(
            profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )
        self.assertEqual(
            profile["bounded_acceptance"]["inverse_dynamics_solve"],
            "NOT_AUTHORIZED",
        )

        changed = copy.deepcopy(profile)
        changed["scope"]["kto_solve_count"] = 2
        with self.assertRaisesRegex(ValueError, "execution profile"):
            _validate_profile(changed)

    def test_so3_exp_log_and_hybrid_qva_are_consistent(self) -> None:
        vector = np.asarray([0.03, -0.07, 0.11], dtype=np.float64)
        self.assertTrue(np.allclose(_rotation_log(_rotation_exp(vector)), vector))

        frames = np.arange(FRAME_COUNT, dtype=np.float64)
        root = np.zeros((FRAME_COUNT, 3), dtype=np.float64)
        root[:, 0] = frames / 60.0
        orientation = np.zeros((FRAME_COUNT, 3), dtype=np.float64)
        joints = np.zeros((FRAME_COUNT, JOINT_COUNT), dtype=np.float64)
        references = np.repeat(np.eye(3)[None, :, :], FRAME_COUNT, axis=0)
        indices = np.empty((FRAME_COUNT, 2), dtype=np.int64)
        coefficients = np.empty((FRAME_COUNT, 2), dtype=np.float64)
        indices[0] = (0, 1)
        coefficients[0] = (-60.0, 60.0)
        indices[-1] = (FRAME_COUNT - 2, FRAME_COUNT - 1)
        coefficients[-1] = (-60.0, 60.0)
        for frame in range(1, FRAME_COUNT - 1):
            indices[frame] = (frame - 1, frame + 1)
            coefficients[frame] = (-30.0, 30.0)
        velocity, acceleration = _derive_velocity_acceleration(
            root_position_m=root,
            root_orientation_delta_rad=orientation,
            joint_position_rad=joints,
            reference_rotations=references,
            stencil_indices=indices,
            stencil_coefficients=coefficients,
        )
        self.assertTrue(np.allclose(velocity[:, 0], 1.0))
        self.assertTrue(np.allclose(velocity[:, 1:], 0.0))
        self.assertTrue(np.allclose(acceleration, 0.0))

    def test_sparse_problem_has_full_r114_qva_inventory(self) -> None:
        descriptor = json.loads(DESCRIPTOR.read_bytes())
        v9_profile = json.loads(V9_PROFILE.read_bytes())
        r114 = json.loads(R114_PROFILE.read_bytes())
        limits = _effective_joint_limits(descriptor, v9_profile)
        joint = np.asarray(
            [
                (row["position_microradians"][0] + row["position_microradians"][1])
                / 2_000_000.0
                for row in limits
            ],
            dtype=np.float64,
        )
        state = KtoState(
            root_position_m=np.tile([0.0, 1.0, 0.0], (FRAME_COUNT, 1)),
            root_orientation_delta_rad=np.zeros((FRAME_COUNT, 3)),
            joint_position_rad=np.tile(joint, (FRAME_COUNT, 1)),
            velocity=np.zeros((FRAME_COUNT, Q_WIDTH)),
            acceleration=np.zeros((FRAME_COUNT, Q_WIDTH)),
        )
        v9_joint = np.rint(state.joint_position_rad * 1_000_000.0).astype(np.int64)
        v9 = {"joint_position_urad": v9_joint}
        v7_joint = v9_joint[238:250].copy()
        v7_joint[3, 5] += 4
        v7 = {
            "reference_frame": np.arange(238, 250, dtype=np.int64),
            "joint_position_urad": v7_joint,
        }
        indices, coefficients = _centered_stencil()
        linearization = KtoLinearization(
            sole_position_m=np.zeros((FRAME_COUNT, 2, 2, 3)),
            sole_jacobian=np.zeros((FRAME_COUNT, 2, 2, 3, Q_WIDTH)),
            collider_height_m=np.ones((FRAME_COUNT, 1)),
            collider_jacobian=np.zeros((FRAME_COUNT, 1, Q_WIDTH)),
            analytic_sole_velocity_m_s=np.zeros((FRAME_COUNT, 2, 2, 3)),
        )
        problem = _build_problem(
            descriptor=descriptor,
            state=state,
            v9_arrays=v9,
            v7_arrays=v7,
            active=np.zeros((FRAME_COUNT, 2, 2), dtype=np.bool_),
            stencil_indices=indices,
            stencil_coefficients=coefficients,
            linearization=linearization,
            effective_limits=limits,
            r114_profile=r114,
        )
        self.assertEqual(problem.constraints.shape[1], 69_687)
        self.assertEqual(problem.objective.shape, (69_687, 69_687))
        self.assertEqual(
            problem.constraint_categories["q_to_v_equality"], FRAME_COUNT * Q_WIDTH
        )
        self.assertEqual(
            problem.constraint_categories["v_to_a_equality"], FRAME_COUNT * Q_WIDTH
        )
        self.assertEqual(problem.variable_scale[ACCELERATION_OFFSET], 72.0)
        self.assertEqual(BLOCK_WIDTH, 87)

    def test_root_yaw_lift_preserves_the_unwrapped_v9_branch(self) -> None:
        angles = np.linspace(3.0, 3.3, FRAME_COUNT, dtype=np.float64)
        source_quaternion = np.stack(
            [quaternion_q1_30(rotation_axis("Y", float(angle))) for angle in angles]
        )
        source_yaw = np.rint(angles * 1_000_000.0).astype(np.int64)
        lifted = _lift_root_yaw(
            emitted_quaternion_q1_30=source_quaternion,
            source_quaternion_q1_30=source_quaternion,
            source_yaw_urad=source_yaw,
        )
        self.assertTrue(np.array_equal(lifted, source_yaw))

    def test_array_hash_and_cache_are_solver_private(self) -> None:
        arrays = {
            "z": np.asarray([1, 2], dtype=np.int64),
            "a": np.asarray([[3.0]], dtype=np.float64),
        }
        first = _array_hashes(arrays)
        second = _array_hashes(dict(reversed(tuple(arrays.items()))))
        self.assertEqual(first, second)
        cache_bytes = _serialize_cache(arrays)
        with np.load(io.BytesIO(cache_bytes), allow_pickle=False) as cache:
            self.assertEqual(set(cache.files), {"a", "z"})
            self.assertTrue(np.array_equal(cache["z"], arrays["z"]))


def _centered_stencil() -> tuple[np.ndarray, np.ndarray]:
    indices = np.empty((FRAME_COUNT, 2), dtype=np.int64)
    coefficients = np.empty((FRAME_COUNT, 2), dtype=np.float64)
    indices[0] = (0, 1)
    coefficients[0] = (-60.0, 60.0)
    indices[-1] = (FRAME_COUNT - 2, FRAME_COUNT - 1)
    coefficients[-1] = (-60.0, 60.0)
    for frame in range(1, FRAME_COUNT - 1):
        indices[frame] = (frame - 1, frame + 1)
        coefficients[frame] = (-30.0, 30.0)
    return indices, coefficients


if __name__ == "__main__":
    unittest.main()
