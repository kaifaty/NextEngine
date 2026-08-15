from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.exit_mode_owned_projected_schedule_execution import (
    derive_eventful_projected_fixed_pd_schedule,
)
from next_lab.fixed_mode_kinodynamic_execution import (
    ACTUATED_JOINT_COUNT,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
    PHYSICS_INTERVAL_COUNT,
    STATE_NODE_COUNT,
    _actual_to_model_ratio,
    _branch_code,
    _cone_residual_and_agreement,
    _event_addresses_from_r133,
    _load_bound_report,
    _selected_extreme,
    _solve_osqp_pass,
    _update_separation_directions,
    _validate_profile,
    build_graph_inventory,
    replay_exact_controller,
)
from scipy import sparse

ROOT = Path(__file__).resolve().parents[2]
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"
PROFILE = ROOT / "lab/profiles/humanoid-fixed-mode-kinodynamic-execution-r141.v1.json"


class FixedModeKinodynamicExecutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.descriptor = json.loads(DESCRIPTOR.read_bytes())

    def _controller_case(
        self,
        position: np.ndarray,
        velocity: np.ndarray,
        *,
        descriptor: dict | None = None,
    ) -> None:
        selected_descriptor = descriptor or self.descriptor
        canonical = derive_eventful_projected_fixed_pd_schedule(
            cache={"joint_position_rad": position},
            descriptor=selected_descriptor,
            projected_velocity=velocity,
        )
        q = np.empty((STATE_NODE_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64)
        for collocation in range(PHYSICS_INTERVAL_COUNT):
            motor, substep = divmod(collocation, 4)
            fraction = substep / 4.0
            q[collocation] = (1.0 - fraction) * position[motor] + fraction * position[
                motor + 1
            ]
        q[-1] = position[-1]
        state_velocity = np.vstack(
            (velocity, np.zeros((1, GENERALIZED_WIDTH), dtype=np.float64))
        )
        command = np.rint(position[:-1] * 1_000_000.0).astype(np.int64)
        replay = replay_exact_controller(
            integer_command=command,
            joint_position=q,
            velocity=state_velocity,
            descriptor=selected_descriptor,
        )
        self.assertTrue(
            np.array_equal(
                replay.applied_target,
                canonical.schedule.applied_target_microradians,
            )
        )
        self.assertTrue(
            np.array_equal(
                replay.applied_effort,
                canonical.schedule.applied_effort_micronewton_metres,
            )
        )
        self.assertTrue(
            np.array_equal(
                replay.event_addresses,
                _event_addresses_from_r133(canonical.events),
            )
        )

    def test_exact_controller_reproduces_zero_case(self) -> None:
        self._controller_case(
            np.zeros((FRAME_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64),
            np.zeros((PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH), dtype=np.float64),
        )

    def test_exact_controller_reproduces_ties_even_case(self) -> None:
        position = np.zeros((FRAME_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64)
        position[:, 0] = 0.5e-6
        position[:, 1] = 1.5e-6
        position[:, 2] = -1.5e-6
        self._controller_case(
            position,
            np.zeros((PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH), dtype=np.float64),
        )

    def test_exact_controller_reproduces_stateful_work_limiter(self) -> None:
        descriptor = copy.deepcopy(self.descriptor)
        position = np.zeros((FRAME_COUNT, ACTUATED_JOINT_COUNT), dtype=np.float64)
        position[1:, 0] = 0.2
        velocity = np.zeros(
            (PHYSICS_INTERVAL_COUNT, GENERALIZED_WIDTH), dtype=np.float64
        )
        velocity[:, 6] = -0.5
        actuator = next(
            row for row in descriptor["actuators"] if int(row["dof_ordinal"]) == 0
        )
        actuator["maximum_positive_work_microjoules_per_motor_tick"] = 1
        self._controller_case(position, velocity, descriptor=descriptor)

    def test_branch_equalities_have_explicit_ambiguity(self) -> None:
        self.assertEqual(_branch_code(-2.0, -1.0, 1.0), (0, False))
        self.assertEqual(_branch_code(-1.0, -1.0, 1.0), (1, True))
        self.assertEqual(_branch_code(0.0, -1.0, 1.0), (2, False))
        self.assertEqual(_branch_code(1.0, -1.0, 1.0), (3, True))
        self.assertEqual(_branch_code(2.0, -1.0, 1.0), (4, False))
        self.assertEqual(
            _selected_extreme(((0, 1.0), (1, 1.0)), maximum=True),
            (255, 1.0, True),
        )

    def test_graph_inventory_is_deterministic_and_fail_closed(self) -> None:
        modes = np.zeros((FRAME_COUNT, 2), dtype=np.uint8)
        modes[:4, 0] = 3
        modes[4:8, 0] = 2
        first = build_graph_inventory(modes, require_production_inventory=False)
        second = build_graph_inventory(modes, require_production_inventory=False)
        self.assertEqual(first.graph_index_sha256, second.graph_index_sha256)
        self.assertEqual(
            first.transition_replay_sha256, second.transition_replay_sha256
        )
        self.assertGreater(len(first.force_rows), 0)
        with self.assertRaisesRegex(RuntimeError, "FIXED_GRAPH_INVENTORY_DIFFERS"):
            build_graph_inventory(modes)

    def test_exact_circular_cone_is_not_replaced_by_polyhedron(self) -> None:
        normal, cone = _cone_residual_and_agreement(
            np.asarray((10.0, 3.0, 4.0), dtype=np.float64)
        )
        self.assertLessEqual(normal, 0.0)
        self.assertLessEqual(cone, 0.0)
        _, outside = _cone_residual_and_agreement(
            np.asarray((5.0, 3.0, 4.0), dtype=np.float64)
        )
        self.assertGreater(outside, 0.0)

    def test_separation_order_and_ratio_are_deterministic(self) -> None:
        target: dict[tuple[str, int], list[tuple[float, float]]] = {}
        additions = _update_separation_directions(
            target,
            (
                {
                    "graph_row": 7,
                    "cone_type": "activation_impulse",
                    "point_ordinal": 3,
                    "row": 1,
                    "right": 0.0,
                    "forward": 5.0,
                },
                {
                    "graph_row": 2,
                    "cone_type": "continuous_force",
                    "point_ordinal": 1,
                    "row": 4,
                    "right": 3.0,
                    "forward": 4.0,
                },
            ),
        )
        self.assertEqual([row["graph_row"] for row in additions], [2, 7])
        self.assertEqual(
            _actual_to_model_ratio(
                current=(10.0, 5.0), trial=(8.0, 4.0), predicted=(6.0, 3.0)
            ),
            0.5,
        )

    def test_osqp_candidate_requires_exact_solved_residual(self) -> None:
        result = _solve_osqp_pass(
            ordinal=3,
            quadratic=sparse.csc_matrix(np.asarray(((1.0,),))),
            linear=np.zeros(1, dtype=np.float64),
            matrix=sparse.csc_matrix(np.asarray(((1.0,),))),
            lower=np.asarray((1.0,), dtype=np.float64),
            upper=np.asarray((1.0,), dtype=np.float64),
            base_count=1,
            physical_matrix=sparse.csc_matrix(np.asarray(((1.0,),))),
            physical_residual=np.asarray((-1.0,), dtype=np.float64),
            physical_upper=np.asarray((False,), dtype=np.bool_),
        )
        self.assertEqual(result.status, "solved")
        self.assertLessEqual(result.maximum_elastic, 1.0e-8)
        self.assertEqual(result.problem_sha256, result.problem_sha256.lower())

    def test_osqp_candidate_preserves_one_sided_elastic_and_rejects_nan(self) -> None:
        result = _solve_osqp_pass(
            ordinal=1,
            quadratic=sparse.csc_matrix(np.asarray(((1.0,),))),
            linear=np.zeros(1, dtype=np.float64),
            matrix=sparse.csc_matrix(np.asarray(((1.0,),))),
            lower=np.asarray((0.0,), dtype=np.float64),
            upper=np.asarray((0.0,), dtype=np.float64),
            base_count=1,
            physical_matrix=sparse.csc_matrix((1, 1)),
            physical_residual=np.asarray((-2.0,), dtype=np.float64),
            physical_upper=np.asarray((True,), dtype=np.bool_),
        )
        self.assertEqual(result.maximum_elastic, 0.0)
        with self.assertRaisesRegex(RuntimeError, "QP_SCHEMA_NONFINITE"):
            _solve_osqp_pass(
                ordinal=1,
                quadratic=sparse.csc_matrix(np.asarray(((1.0,),))),
                linear=np.zeros(1, dtype=np.float64),
                matrix=sparse.csc_matrix(np.asarray(((1.0,),))),
                lower=np.asarray((np.nan,), dtype=np.float64),
                upper=np.asarray((0.0,), dtype=np.float64),
                base_count=1,
                physical_matrix=sparse.csc_matrix((1, 1)),
                physical_residual=np.asarray((0.0,), dtype=np.float64),
                physical_upper=np.asarray((True,), dtype=np.bool_),
            )

    def test_profile_freezes_one_execution_and_no_downstream_authority(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertEqual(profile["execution_budget"]["qp_solve_limit"], 18)
        self.assertEqual(profile["execution_budget"]["exact_trial_audit_limit"], 42)
        self.assertTrue(
            all(
                value == "NOT_AUTHORIZED"
                for key, value in profile["bounded_acceptance"].items()
                if key != "r142_report_only_decision"
            )
        )

    def test_bound_report_rejects_file_mismatch_before_payload_use(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "X_REPORT_FILE_SHA256_DIFFERS"):
            _load_bound_report(
                PROFILE,
                {
                    "report_file_sha256": "0" * 64,
                    "report_sha256": "1" * 64,
                },
                "X",
            )


if __name__ == "__main__":
    unittest.main()
