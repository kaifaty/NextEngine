from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.fixed_pd_inverse_dynamics_execution import (
    ExecutionOutcome,
    LocalSystem,
    _validate_profile,
    assemble_local_system,
    audit_contact_cones,
    collocation_state,
    derive_fixed_pd_schedule,
    encode_solver_private_cache,
    solve_local_system,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    COLLOCATION_COUNT,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
    LOCAL_UNKNOWN_COUNT,
    POINT_COUNT,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-fixed-pd-inverse-dynamics-execution-r123.v1.json"
)
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class FixedPdInverseDynamicsExecutionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())
        cls.descriptor = json.loads(DESCRIPTOR.read_bytes())

    def test_profile_freezes_exactly_one_r123_and_only_r124_report_next(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(
            self.profile["execution_budget"]["maximum_local_system_solves"],
            3200,
        )
        self.assertEqual(self.profile["execution_budget"]["thread_count"], 1)
        self.assertEqual(
            self.profile["decision"]["valid_complete"],
            "PERMIT_SEPARATE_REPORT_ONLY_R124_FULL_KINODYNAMIC_EXECUTION_FORMULATION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["r123_retry"], "NOT_AUTHORIZED"
        )

    def test_local_system_layout_preserves_effort_and_force_signs(self) -> None:
        mass = np.eye(GENERALIZED_WIDTH, dtype=np.float64)
        bias = np.linspace(-2.0, 3.0, GENERALIZED_WIDTH, dtype=np.float64)
        effort = np.linspace(-4.0, 5.0, ACTUATOR_COUNT, dtype=np.float64)
        jacobians = np.zeros((POINT_COUNT, 3, GENERALIZED_WIDTH), dtype=np.float64)
        jacobians[0, :, :3] = np.eye(3, dtype=np.float64)
        jdot_v = np.zeros((POINT_COUNT, 3), dtype=np.float64)
        active = np.zeros(POINT_COUNT, dtype=np.bool_)
        system = assemble_local_system(
            mass=mass,
            bias=bias,
            effort_newton_metres=effort,
            contact_jacobians=jacobians,
            contact_jdot_v=jdot_v,
            active_points=active,
            modes=np.asarray((0, 0), dtype=np.uint8),
        )
        force = GENERALIZED_WIDTH + ACTUATOR_COUNT
        self.assertEqual(system.matrix[1, force], -1.0)
        self.assertEqual(system.matrix[0, force + 1], -1.0)
        self.assertEqual(system.matrix[2, force + 2], -1.0)
        np.testing.assert_array_equal(system.right_hand_side[:GENERALIZED_WIDTH], -bias)
        np.testing.assert_array_equal(
            system.right_hand_side[
                GENERALIZED_WIDTH : GENERALIZED_WIDTH + ACTUATOR_COUNT
            ],
            effort,
        )

        solution = solve_local_system(
            system,
            column_scale=np.ones(LOCAL_UNKNOWN_COUNT, dtype=np.float64),
            numeric_contract=self.profile["numeric_contract"],
        )
        self.assertEqual(solution.status, "VALID")
        assert solution.solution is not None
        np.testing.assert_allclose(
            solution.solution[GENERALIZED_WIDTH : GENERALIZED_WIDTH + ACTUATOR_COUNT],
            effort,
            atol=1.0e-12,
        )
        np.testing.assert_allclose(solution.solution[force:], 0.0, atol=1.0e-12)
        np.testing.assert_allclose(
            system.matrix @ solution.solution,
            system.right_hand_side,
            atol=1.0e-12,
        )

    def test_active_rows_bind_acceleration_while_inactive_rows_zero_force(self) -> None:
        jacobians = np.zeros((POINT_COUNT, 3, GENERALIZED_WIDTH), dtype=np.float64)
        jacobians[0, :, :3] = np.eye(3, dtype=np.float64)
        jdot_v = np.arange(12, dtype=np.float64).reshape(POINT_COUNT, 3)
        active = np.asarray((True, False, False, False), dtype=np.bool_)
        system = assemble_local_system(
            mass=np.eye(GENERALIZED_WIDTH, dtype=np.float64),
            bias=np.zeros(GENERALIZED_WIDTH, dtype=np.float64),
            effort_newton_metres=np.zeros(ACTUATOR_COUNT, dtype=np.float64),
            contact_jacobians=jacobians,
            contact_jdot_v=jdot_v,
            active_points=active,
            modes=np.asarray((1, 0), dtype=np.uint8),
        )
        row = GENERALIZED_WIDTH + ACTUATOR_COUNT
        force = GENERALIZED_WIDTH + ACTUATOR_COUNT
        np.testing.assert_array_equal(system.matrix[row : row + 3, :3], np.eye(3))
        np.testing.assert_array_equal(system.right_hand_side[row : row + 3], -jdot_v[0])
        np.testing.assert_array_equal(
            system.matrix[row + 3 : row + 6, force + 3 : force + 6],
            np.eye(3),
        )

    def test_active_contact_solution_recovers_normal_right_forward_order(self) -> None:
        jacobians = np.zeros((POINT_COUNT, 3, GENERALIZED_WIDTH), dtype=np.float64)
        jacobians[0, :, :3] = np.eye(3, dtype=np.float64)
        jdot_v = np.zeros((POINT_COUNT, 3), dtype=np.float64)
        jdot_v[0] = (1.0, 2.0, 3.0)
        bias = np.zeros(GENERALIZED_WIDTH, dtype=np.float64)
        bias[:3] = (2.0, 12.0, 4.0)
        system = assemble_local_system(
            mass=np.eye(GENERALIZED_WIDTH, dtype=np.float64),
            bias=bias,
            effort_newton_metres=np.zeros(ACTUATOR_COUNT, dtype=np.float64),
            contact_jacobians=jacobians,
            contact_jdot_v=jdot_v,
            active_points=np.asarray((True, False, False, False), dtype=np.bool_),
            modes=np.asarray((1, 0), dtype=np.uint8),
        )
        solution = solve_local_system(
            system,
            column_scale=np.ones(LOCAL_UNKNOWN_COUNT, dtype=np.float64),
            numeric_contract=self.profile["numeric_contract"],
        )
        self.assertEqual(solution.status, "VALID")
        assert solution.solution is not None
        np.testing.assert_allclose(solution.solution[:3], (-1.0, -2.0, -3.0))
        force = GENERALIZED_WIDTH + ACTUATOR_COUNT
        np.testing.assert_allclose(
            solution.solution[force : force + 3], (10.0, 1.0, 1.0)
        )
        cone = audit_contact_cones(
            solution.solution[force:].reshape(POINT_COUNT, 3),
            active=system.active_points,
            tolerance_newtons=1.0e-7,
        )
        self.assertTrue(cone["friction_cone_feasible"])

    def test_condition_guard_rejects_unreliable_system_without_solve(self) -> None:
        matrix = np.eye(LOCAL_UNKNOWN_COUNT, dtype=np.float64)
        matrix[-1, -1] = 1.0e-14
        solution = solve_local_system(
            LocalSystem(
                matrix,
                np.zeros(LOCAL_UNKNOWN_COUNT, dtype=np.float64),
                np.zeros(POINT_COUNT, dtype=np.bool_),
                np.zeros(2, dtype=np.uint8),
            ),
            column_scale=np.ones(LOCAL_UNKNOWN_COUNT, dtype=np.float64),
            numeric_contract=self.profile["numeric_contract"],
        )
        self.assertEqual(solution.status, "INVALID")
        self.assertIsNone(solution.solution)
        self.assertEqual(solution.invalid_reason, "SCALED_LOCAL_SYSTEM_ILL_CONDITIONED")

    def test_contact_cone_classifies_raw_margins_without_projection(self) -> None:
        forces = np.asarray(
            (
                (100.0, 30.0, 40.0),
                (-0.5, 0.0, 0.0),
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0),
            ),
            dtype=np.float64,
        )
        audit = audit_contact_cones(
            forces,
            active=np.asarray((True, True, False, False), dtype=np.bool_),
            tolerance_newtons=1.0e-7,
        )
        self.assertFalse(audit["friction_cone_feasible"])
        self.assertEqual(audit["violating_active_point_ordinals"], [1])
        self.assertEqual(audit["minimum_active_normal_margin_newtons"], -0.5)
        np.testing.assert_array_equal(forces[0], (100.0, 30.0, 40.0))

    def test_zero_state_fixed_pd_schedule_is_retained_at_all_collocations(self) -> None:
        cache = _zero_cache()
        schedule = derive_fixed_pd_schedule(cache=cache, descriptor=self.descriptor)
        self.assertEqual(
            schedule.applied_effort_micronewton_metres.shape,
            (COLLOCATION_COUNT, ACTUATOR_COUNT),
        )
        np.testing.assert_array_equal(schedule.applied_effort_micronewton_metres, 0.0)
        self.assertEqual(schedule.audit["status"], "PASS")
        self.assertEqual(
            sum(schedule.audit["activation_counts"].values()),
            0,
        )

    def test_collocation_lift_and_solver_cache_have_no_candidate_authority(
        self,
    ) -> None:
        cache = _zero_cache()
        cache["root_position_m"][1] = (4.0, 8.0, 12.0)
        cache["joint_position_rad"][1] = 4.0
        cache["velocity"][1] = 8.0
        cache["acceleration"][1] = 12.0
        state = collocation_state(cache, 0, 2)
        np.testing.assert_array_equal(
            state.configuration.root_position, (2.0, 4.0, 6.0)
        )
        np.testing.assert_array_equal(state.configuration.joint_positions, 2.0)
        np.testing.assert_array_equal(state.velocity, 4.0)
        np.testing.assert_array_equal(state.warm_acceleration, 6.0)

        arrays = {
            "generalized_acceleration": np.zeros((COLLOCATION_COUNT, 29)),
            "applied_effort_newton_metres": np.zeros((COLLOCATION_COUNT, 23)),
            "point_force_normal_right_forward_newtons": np.zeros(
                (COLLOCATION_COUNT, 4, 3)
            ),
            "interval_index": np.repeat(np.arange(800), 4).astype(np.int64),
            "substep_index": np.tile(np.arange(4), 800).astype(np.int64),
        }
        outcome = ExecutionOutcome(
            status="VALID_COMPLETE",
            feasibility="FEASIBLE",
            invalid_reason=None,
            local_system_solves=COLLOCATION_COUNT,
            singular_value_decompositions=COLLOCATION_COUNT,
            collocation_rows=(),
            aggregate={},
            arrays=arrays,
            resource_usage={},
            local_system_sha256="0" * 64,
        )
        payload, identity = encode_solver_private_cache(
            outcome=outcome,
            profile_sha256="1" * 64,
            r122_report_sha256="2" * 64,
        )
        self.assertIsNotNone(payload)
        self.assertFalse(identity["candidate_or_corpus_authority"])
        self.assertEqual(identity["status"], "EMITTED_SOLVER_PRIVATE_TRANSIENT")


def _zero_cache() -> dict[str, np.ndarray]:
    rotations = np.repeat(np.eye(3, dtype=np.float64)[np.newaxis], FRAME_COUNT, axis=0)
    return {
        "root_position_m": np.zeros((FRAME_COUNT, 3), dtype=np.float64),
        "root_orientation_delta_rad": np.zeros((FRAME_COUNT, 3), dtype=np.float64),
        "joint_position_rad": np.zeros((FRAME_COUNT, ACTUATOR_COUNT), dtype=np.float64),
        "velocity": np.zeros((FRAME_COUNT, GENERALIZED_WIDTH), dtype=np.float64),
        "acceleration": np.zeros((FRAME_COUNT, GENERALIZED_WIDTH), dtype=np.float64),
        "reference_root_rotation": rotations,
    }


if __name__ == "__main__":
    unittest.main()
