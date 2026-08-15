from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.kto_linearization_repair_conformance import (
    AnchorDefinition,
    repaired_exact_contact_velocity,
    symmetric_whole_function_jacobian,
)
from next_lab.quantization_aware_kto_solver import (
    _column,
    _effective_joint_limits,
)
from next_lab.repaired_kto_execution import _validate_profile
from next_lab.repaired_kto_solver import (
    BLOCK_WIDTH,
    FRAME_COUNT,
    JOINT_COUNT,
    Q_WIDTH,
    ContactRowLinearization,
    KtoLinearization,
    KtoState,
    _funnel_strictly_decreases,
    _zero_state_reproduction,
    build_repaired_problem,
    optimized_contact_velocity_and_jacobian,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-repaired-kto-execution-r120.v1.json"
R114_PROFILE = (
    ROOT
    / "lab/profiles/humanoid-quantization-aware-kto-execution-formulation-r114.v1.json"
)
R118_PROFILE = (
    ROOT / "lab/profiles/humanoid-kto-linearization-repair-conformance-r118.v1.json"
)
V9_PROFILE = ROOT / "lab/profiles/humanoid-contact-manifold-prototype.v9.json"
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class RepairedKtoExecutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())
        self.descriptor = json.loads(DESCRIPTOR.read_bytes())

    def test_profile_authorizes_one_r120_and_no_downstream_execution(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["kto_process_count"], 1)
        self.assertEqual(self.profile["scope"]["maximum_qp_solves"], 12)
        self.assertEqual(
            self.profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )
        changed = copy.deepcopy(self.profile)
        changed["scope"]["kto_solve_count"] = 2
        with self.assertRaisesRegex(ValueError, "execution profile"):
            _validate_profile(changed)

    def test_optimized_contact_derivative_matches_whole_function(self) -> None:
        r118 = json.loads(R118_PROFILE.read_bytes())
        arrays = _synthetic_arrays(self.descriptor)
        anchor = AnchorDefinition(
            role="unit",
            frame=10,
            side=0,
            point=1,
            stencil_frames=(9, 11),
            stencil_coefficients=(-30.0, 30.0),
        )
        specs, value, optimized = optimized_contact_velocity_and_jacobian(
            descriptor=self.descriptor,
            arrays=arrays,
            anchor=anchor,
            probes=r118["variable_contract"]["probes"],
            exact_probe=1.0e-4,
        )
        whole = symmetric_whole_function_jacobian(
            evaluator=repaired_exact_contact_velocity,
            descriptor=self.descriptor,
            arrays=arrays,
            anchor=anchor,
            variables=specs,
            exact_probe=1.0e-4,
        )
        zero = repaired_exact_contact_velocity(
            descriptor=self.descriptor,
            arrays=arrays,
            anchor=anchor,
            variables=specs,
            delta=np.zeros(len(specs)),
            exact_probe=1.0e-4,
        )
        allowed = 5.0 + 1.0e-4 * np.maximum(np.abs(optimized), np.abs(whole))
        self.assertTrue(np.all(np.abs(optimized - whole) <= allowed))
        self.assertTrue(np.allclose(value, zero, atol=1.0))

    def test_repaired_problem_retires_component_box_and_rebases_trust(self) -> None:
        v9_profile = json.loads(V9_PROFILE.read_bytes())
        r114 = json.loads(R114_PROFILE.read_bytes())
        limits = _effective_joint_limits(self.descriptor, v9_profile)
        joint = np.asarray(
            [
                (row["position_microradians"][0] + row["position_microradians"][1])
                / 2_000_000.0
                for row in limits
            ],
            dtype=np.float64,
        )
        root = np.tile([0.0, 1.0, 0.0], (FRAME_COUNT, 1))
        joints = np.tile(joint, (FRAME_COUNT, 1))
        initial = KtoState(
            root_position_m=root.copy(),
            root_orientation_delta_rad=np.zeros((FRAME_COUNT, 3)),
            joint_position_rad=joints.copy(),
            velocity=np.zeros((FRAME_COUNT, Q_WIDTH)),
            acceleration=np.zeros((FRAME_COUNT, Q_WIDTH)),
        )
        shifted_root = root.copy()
        shifted_root[10, 0] += 0.01
        state = KtoState(
            root_position_m=shifted_root,
            root_orientation_delta_rad=np.zeros((FRAME_COUNT, 3)),
            joint_position_rad=joints.copy(),
            velocity=np.zeros((FRAME_COUNT, Q_WIDTH)),
            acceleration=np.zeros((FRAME_COUNT, Q_WIDTH)),
        )
        v9_joint = np.rint(joints * 1_000_000.0).astype(np.int64)
        v9 = {
            "root_position_um": np.rint(root * 1_000_000.0).astype(np.int64),
            "joint_position_urad": v9_joint,
        }
        v7_joint = v9_joint[238:250].copy()
        v7_joint[3, 5] += 4
        v7 = {
            "reference_frame": np.arange(238, 250, dtype=np.int64),
            "joint_position_urad": v7_joint,
        }
        active = np.zeros((FRAME_COUNT, 2, 2), dtype=np.bool_)
        active[10, 0, 1] = True
        indices, coefficients = _centered_stencil()
        geometry = KtoLinearization(
            sole_position_m=np.zeros((FRAME_COUNT, 2, 2, 3)),
            sole_jacobian=np.zeros((FRAME_COUNT, 2, 2, 3, Q_WIDTH)),
            collider_height_m=np.ones((FRAME_COUNT, 1)),
            collider_jacobian=np.zeros((FRAME_COUNT, 1, Q_WIDTH)),
            analytic_sole_velocity_m_s=np.zeros((FRAME_COUNT, 2, 2, 3)),
        )
        column = _column(10, 1, 0)
        repaired = ContactRowLinearization(
            anchor=AnchorDefinition("unit", 10, 0, 1, (9, 11), (-30.0, 30.0)),
            variables=(),
            global_columns=np.asarray([column], dtype=np.int64),
            velocity_m_s=np.asarray([0.01, 0.0, 0.02]),
            vector_jacobian_m_s=np.asarray([[1.0], [0.5], [-0.25]], dtype=np.float64),
        )
        problem = build_repaired_problem(
            descriptor=self.descriptor,
            state=state,
            initial_state=initial,
            v9_arrays=v9,
            v7_arrays=v7,
            active=active,
            stencil_indices=indices,
            stencil_coefficients=coefficients,
            geometry=geometry,
            repaired_contact=(repaired,),
            effective_limits=limits,
            r114_profile=r114,
        )
        self.assertNotIn("contact_analytic_velocity", problem.constraint_categories)
        self.assertEqual(problem.constraint_categories["contact_analytic_normal"], 1)
        self.assertEqual(
            problem.constraint_categories["contact_analytic_tangential_norm_squared"],
            1,
        )
        trust_start = sum(
            count
            for name, count in problem.constraint_categories.items()
            if name
            not in (
                "endpoint_or_trust_bound",
                "contact_analytic_normal",
                "contact_analytic_tangential_norm_squared",
            )
        )
        trust_row = trust_start + 10 * Q_WIDTH
        self.assertAlmostEqual(problem.lower[trust_row], -1.5)
        self.assertAlmostEqual(problem.upper[trust_row], 0.5)
        self.assertAlmostEqual(
            problem.objective_linear[_column(10, 0, 0)],
            1.0 / (FRAME_COUNT * Q_WIDTH),
        )
        self.assertEqual(problem.constraints.shape[1], FRAME_COUNT * BLOCK_WIDTH)

    def test_zero_identity_and_lexicographic_funnel_are_exact(self) -> None:
        arrays = {"a": np.asarray([1, 2], dtype=np.int64)}
        self.assertEqual(_zero_state_reproduction(arrays, arrays)["status"], "PASS")
        changed = {"a": np.asarray([1, 3], dtype=np.int64)}
        self.assertEqual(
            _zero_state_reproduction(changed, arrays)["differing_arrays"], ["a"]
        )
        self.assertTrue(
            _funnel_strictly_decreases(
                {"maximum": 0.5, "sum": 9.0},
                {"maximum": 0.6, "sum": 1.0},
            )
        )
        self.assertFalse(
            _funnel_strictly_decreases(
                {"maximum": 0.5, "sum": 9.0},
                {"maximum": 0.5, "sum": 8.0},
            )
        )


def _synthetic_arrays(descriptor: dict[str, object]) -> dict[str, np.ndarray]:
    joints = sorted(descriptor["joints"], key=lambda row: row["dof_ordinal"])
    midpoint = np.asarray(
        [sum(row["soft_limit_microradians"]) // 2 for row in joints], dtype=np.int64
    )
    quaternion = np.zeros((FRAME_COUNT, 4), dtype=np.int64)
    quaternion[:, 3] = 1 << 30
    joint_velocity = np.zeros((FRAME_COUNT, JOINT_COUNT), dtype=np.int64)
    joint_velocity[:, :6] = np.asarray([10_000, -20_000, 15_000, 8_000, -9_000, 7_000])
    return {
        "root_position_um": np.tile([0, 1_000_000, 0], (FRAME_COUNT, 1)).astype(
            np.int64
        ),
        "root_quaternion_q1_30": quaternion,
        "joint_position_urad": np.tile(midpoint, (FRAME_COUNT, 1)),
        "root_linear_velocity_um_s": np.tile(
            [20_000, 0, 10_000], (FRAME_COUNT, 1)
        ).astype(np.int64),
        "root_yaw_velocity_urad_s": np.full(FRAME_COUNT, 30_000, dtype=np.int64),
        "joint_velocity_urad_s": joint_velocity,
    }


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
