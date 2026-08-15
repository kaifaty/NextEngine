from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.gauge_aware_fixed_pd_execution import (
    ExecutionOutcome,
    _validate_profile,
    audit_selected_equality,
    encode_solver_private_cache,
    solve_gauge_aware_collocation,
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import ReducedLocalSystem

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-gauge-aware-fixed-pd-execution-r127.v1.json"


class GaugeAwareFixedPdExecutionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())
        cls.numeric = cls.profile["numeric_contract"]

    def test_profile_is_one_bounded_execution_without_retry(self) -> None:
        _validate_profile(self.profile)
        budget = self.profile["execution_budget"]
        self.assertEqual(budget["maximum_singular_value_decompositions"], 3200)
        self.assertEqual(budget["maximum_particular_solutions"], 3200)
        self.assertEqual(budget["maximum_gauge_interval_classifications"], 2316)
        self.assertEqual(
            self.profile["bounded_acceptance"]["r127_retry"], "NOT_AUTHORIZED"
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["training"], "NOT_AUTHORIZED"
        )

    def test_flight_and_single_point_classification(self) -> None:
        flight_target = np.linspace(-0.2, 0.2, 29, dtype=np.float64)
        flight = algebraic_system(
            np.eye(29),
            flight_target,
            modes=(0, 0),
            active=(),
            gauge=np.empty((29, 0)),
        )
        solved_flight = solve_gauge_aware_collocation(
            flight,
            column_scale=np.ones(29),
            numeric_contract=self.numeric,
        )
        self.assertEqual(solved_flight.status, "VALID")
        self.assertEqual(solved_flight.feasibility, "FEASIBLE")
        np.testing.assert_allclose(solved_flight.selected_solution, flight_target)

        single_target = np.zeros(32, dtype=np.float64)
        single_target[-3:] = (10.0, 1.0, 0.0)
        single = algebraic_system(
            np.eye(32),
            single_target,
            modes=(0, 2),
            active=(3,),
            gauge=np.empty((32, 0)),
        )
        solved_single = solve_gauge_aware_collocation(
            single,
            column_scale=np.ones(32),
            numeric_contract=self.numeric,
        )
        self.assertEqual(solved_single.status, "VALID")
        self.assertEqual(solved_single.feasibility, "FEASIBLE")
        self.assertTrue(solved_single.cone_audit["friction_cone_feasible"])

        single_target[-3:] = (1.0, 2.0, 0.0)
        infeasible = algebraic_system(
            np.eye(32),
            single_target,
            modes=(0, 2),
            active=(3,),
            gauge=np.empty((32, 0)),
        )
        solved_infeasible = solve_gauge_aware_collocation(
            infeasible,
            column_scale=np.ones(32),
            numeric_contract=self.numeric,
        )
        self.assertEqual(solved_infeasible.status, "VALID")
        self.assertEqual(solved_infeasible.feasibility, "INFEASIBLE")

    def test_flat_gauge_feasible_and_disjoint_are_both_numerically_valid(self) -> None:
        feasible = flat_projector_system(right_force=1.0)
        solved_feasible = solve_gauge_aware_collocation(
            feasible,
            column_scale=np.ones(35),
            numeric_contract=self.numeric,
        )
        self.assertEqual(solved_feasible.status, "VALID")
        self.assertEqual(solved_feasible.feasibility, "FEASIBLE")
        self.assertTrue(solved_feasible.gauge_feasibility.feasible)
        self.assertTrue(solved_feasible.cone_audit["friction_cone_feasible"])

        disjoint = flat_projector_system(right_force=9.0)
        solved_disjoint = solve_gauge_aware_collocation(
            disjoint,
            column_scale=np.ones(35),
            numeric_contract=self.numeric,
        )
        self.assertEqual(solved_disjoint.status, "VALID")
        self.assertEqual(solved_disjoint.feasibility, "INFEASIBLE")
        self.assertFalse(solved_disjoint.gauge_feasibility.feasible)
        self.assertFalse(solved_disjoint.cone_audit["friction_cone_feasible"])

    def test_selected_gauge_witness_preserves_all_equalities(self) -> None:
        system = flat_projector_system(right_force=7.9)
        solution = solve_gauge_aware_collocation(
            system,
            column_scale=np.ones(35),
            numeric_contract=self.numeric,
        )
        self.assertEqual(solution.status, "VALID")
        self.assertEqual(solution.feasibility, "FEASIBLE")
        audit = audit_selected_equality(
            system,
            selected_solution=solution.selected_solution,
            column_scale=np.ones(35),
        )
        self.assertLessEqual(audit["scaled_absolute_residual"], 1.0e-12)
        self.assertLessEqual(audit["dynamics_absolute_residual"], 1.0e-12)
        self.assertLessEqual(audit["closure_absolute_residual"], 1.0e-12)

    def test_solver_private_cache_is_non_authoritative_and_hash_bound(self) -> None:
        arrays = {
            "generalized_acceleration": np.zeros((2, 29), dtype=np.float64),
            "collocation_feasible": np.ones(2, dtype=np.uint8),
        }
        outcome = ExecutionOutcome(
            status="VALID_COMPLETE",
            feasibility="FEASIBLE",
            invalid_reason=None,
            singular_value_decompositions=2,
            particular_solutions=2,
            gauge_interval_classifications=0,
            rows=(),
            aggregate={},
            arrays=arrays,
            resource_usage={},
            ordered_reduced_system_sha256="0" * 64,
        )
        payload, identity = encode_solver_private_cache(
            outcome=outcome,
            profile_sha256="1" * 64,
            r126_report_sha256="2" * 64,
        )
        self.assertIsNotNone(payload)
        self.assertFalse(identity["candidate_or_corpus_authority"])
        self.assertEqual(identity["status"], "EMITTED_SOLVER_PRIVATE_TRANSIENT")
        self.assertEqual(identity["metadata"]["feasibility"], "FEASIBLE")


def algebraic_system(
    matrix: np.ndarray,
    right: np.ndarray,
    *,
    modes: tuple[int, int],
    active: tuple[int, ...],
    gauge: np.ndarray,
) -> ReducedLocalSystem:
    return ReducedLocalSystem(
        matrix=np.asarray(matrix, dtype=np.float64),
        right_hand_side=np.asarray(right, dtype=np.float64),
        active_point_ordinals=active,
        modes=np.asarray(modes, dtype=np.uint8),
        analytic_gauge_matrix=np.asarray(gauge, dtype=np.float64),
        gauge_identities=(),
    )


def flat_projector_system(*, right_force: float) -> ReducedLocalSystem:
    gauge = np.zeros((35, 1), dtype=np.float64)
    gauge[30, 0] = 1.0
    gauge[33, 0] = -1.0
    unit = gauge[:, 0] / np.linalg.norm(gauge[:, 0])
    matrix = np.eye(35, dtype=np.float64) - np.outer(unit, unit)
    target = np.zeros(35, dtype=np.float64)
    target[29:32] = (10.0, right_force, 0.0)
    target[32:35] = (10.0, right_force, 0.0)
    return algebraic_system(
        matrix,
        matrix @ target,
        modes=(0, 3),
        active=(2, 3),
        gauge=gauge,
    )


if __name__ == "__main__":
    unittest.main()
