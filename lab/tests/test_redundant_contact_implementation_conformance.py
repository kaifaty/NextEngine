from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.fixed_pd_inverse_dynamics_execution import (
    CollocationState,
    assemble_local_system,
)
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    GENERALIZED_WIDTH,
)
from next_lab.fixed_pd_inverse_dynamics_kernel import (
    Configuration,
    build_spatial_model,
)
from next_lab.gauge_aware_fixed_pd_inverse_dynamics import (
    FRICTION,
    analyse_reduced_system,
    assemble_reduced_local_system,
    build_reduced_local_system,
    classify_flat_gauge_feasibility,
    line_cone_interval,
)
from next_lab.redundant_contact_implementation_conformance import (
    _validate_profile,
    audit_maximum_payload,
    audit_synthetic_line_cones,
    audit_synthetic_rank_cases,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-redundant-contact-implementation-conformance-r126.v1.json"
)
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"
POINT_IDS = (
    "effector.left-heel",
    "effector.left-forefoot",
    "effector.right-heel",
    "effector.right-forefoot",
)


class RedundantContactImplementationConformanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_r127_on_exact_pass(self) -> None:
        _validate_profile(self.profile)
        self.assertFalse(self.profile["scope"]["r127_execution"])
        self.assertEqual(self.profile["scope"]["real_schedule_particular_solutions"], 0)
        self.assertEqual(
            self.profile["decision"]["pass"],
            "PERMIT_R127_SINGLE_BOUNDED_GAUGE_AWARE_FIXED_PD_EXECUTION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["training"], "NOT_AUTHORIZED"
        )

    def test_effort_substitution_and_inactive_elimination_are_exact(self) -> None:
        generator = np.random.default_rng(7)
        dense = generator.normal(size=(GENERALIZED_WIDTH, GENERALIZED_WIDTH))
        mass = dense.T @ dense + np.eye(GENERALIZED_WIDTH)
        bias = generator.normal(size=GENERALIZED_WIDTH)
        effort = generator.normal(size=ACTUATOR_COUNT)
        jacobians = generator.normal(size=(4, 3, GENERALIZED_WIDTH))
        jdot_v = generator.normal(size=(4, 3))
        modes = np.asarray((0, 3), dtype=np.uint8)
        active_mask = np.asarray((False, False, True, True), dtype=np.bool_)
        full = assemble_local_system(
            mass=mass,
            bias=bias,
            effort_newton_metres=effort,
            contact_jacobians=jacobians,
            contact_jdot_v=jdot_v,
            active_points=active_mask,
            modes=modes,
        )
        reduced_matrix, reduced_right = assemble_reduced_local_system(
            mass=mass,
            bias=bias,
            effort_newton_metres=effort,
            contact_jacobians=jacobians,
            contact_jdot_v=jdot_v,
            active_point_ordinals=(2, 3),
            modes=modes,
        )

        force_offset = GENERALIZED_WIDTH + ACTUATOR_COUNT
        selected_rows = [*range(GENERALIZED_WIDTH)]
        selected_columns = [*range(GENERALIZED_WIDTH)]
        for point in (2, 3):
            selected_rows.extend(
                range(force_offset + 3 * point, force_offset + 3 * (point + 1))
            )
            selected_columns.extend(
                range(force_offset + 3 * point, force_offset + 3 * (point + 1))
            )
        expected_matrix = full.matrix[np.ix_(selected_rows, selected_columns)]
        effort_columns = list(
            range(GENERALIZED_WIDTH, GENERALIZED_WIDTH + ACTUATOR_COUNT)
        )
        expected_right = (
            full.right_hand_side[selected_rows]
            - full.matrix[np.ix_(selected_rows, effort_columns)] @ effort
        )
        np.testing.assert_array_equal(reduced_matrix, expected_matrix)
        np.testing.assert_array_equal(reduced_right, expected_right)

    def test_real_geometry_gauge_has_zero_matrix_product_and_wrench(self) -> None:
        descriptor = json.loads(DESCRIPTOR.read_bytes())
        body_slots = {
            row["body_id"]: int(row["body_slot"]) for row in descriptor["bodies"]
        }
        effectors = {row["effector_id"]: row for row in descriptor["effectors"]}
        points = tuple(
            {
                "point_ordinal": ordinal,
                "effector_id": effector_id,
                "body_id": effectors[effector_id]["body_id"],
                "body_slot": body_slots[effectors[effector_id]["body_id"]],
                "local_translation_metres": (
                    np.asarray(
                        effectors[effector_id]["local_translation_micrometres"],
                        dtype=np.float64,
                    )
                    / 1_000_000.0
                ).tolist(),
            }
            for ordinal, effector_id in enumerate(POINT_IDS)
        )
        model = build_spatial_model(descriptor)
        state = CollocationState(
            configuration=Configuration(
                root_position=np.asarray((0.0, 1.0, 0.0), dtype=np.float64),
                root_rotation=np.eye(3, dtype=np.float64),
                joint_positions=np.zeros(23, dtype=np.float64),
            ),
            velocity=np.zeros(GENERALIZED_WIDTH, dtype=np.float64),
            warm_acceleration=np.zeros(GENERALIZED_WIDTH, dtype=np.float64),
        )
        system = build_reduced_local_system(
            model=model,
            state=state,
            effort_newton_metres=np.zeros(ACTUATOR_COUNT, dtype=np.float64),
            modes=np.asarray((0, 3), dtype=np.uint8),
            points=points,
        )
        self.assertEqual(system.matrix.shape, (35, 35))
        self.assertEqual(system.analytic_gauge_matrix.shape, (35, 1))
        np.testing.assert_allclose(
            system.matrix @ system.analytic_gauge_matrix,
            np.zeros((35, 1)),
            atol=1.0e-12,
        )
        self.assertLessEqual(
            system.gauge_identities[0].maximum_resultant_force, 1.0e-15
        )
        self.assertLessEqual(
            system.gauge_identities[0].maximum_resultant_moment, 1.0e-15
        )

    def test_synthetic_rank_and_oracle_suites_pass(self) -> None:
        rank = audit_synthetic_rank_cases(self.profile)
        cones = audit_synthetic_line_cones(self.profile)
        self.assertEqual(rank["status"], "PASS")
        self.assertEqual(rank["singular_value_decompositions"], 5)
        self.assertEqual(rank["particular_solutions"], 3)
        self.assertEqual(cones["status"], "PASS")
        self.assertEqual(cones["gauge_interval_classifications"], 4)

    def test_gauge_can_recover_alpha_zero_and_disjoint_case_is_infeasible(self) -> None:
        numeric = self.profile["numeric_contract"]
        direction = np.asarray((0.0, 1.0, 0.0), dtype=np.float64)
        recovered = classify_flat_gauge_feasibility(
            np.asarray(((10.0, 9.0, 0.0), (10.0, -7.0, 0.0))),
            direction,
            boundary_relative_ambiguity=numeric[
                "quadratic_boundary_relative_ambiguity"
            ],
            cone_absolute_newtons=numeric["cone_absolute_newtons"],
        )
        self.assertEqual(recovered.status, "VALID")
        self.assertTrue(recovered.feasible)
        self.assertEqual(recovered.witness_alpha, -8.0)
        self.assertGreater(recovered.minimum_friction_margin_newtons, 0.0)

        disjoint = classify_flat_gauge_feasibility(
            np.asarray(((10.0, 9.0, 0.0), (10.0, 9.0, 0.0))),
            direction,
            boundary_relative_ambiguity=numeric[
                "quadratic_boundary_relative_ambiguity"
            ],
            cone_absolute_newtons=numeric["cone_absolute_newtons"],
        )
        self.assertEqual(disjoint.status, "VALID")
        self.assertFalse(disjoint.feasible)

    def test_tangent_and_extra_nullity_are_invalid_not_repaired(self) -> None:
        numeric = self.profile["numeric_contract"]
        tangent = line_cone_interval(
            np.asarray((1.0, FRICTION, 0.0), dtype=np.float64),
            np.asarray((0.0, 0.0, 1.0), dtype=np.float64),
            boundary_relative_ambiguity=numeric[
                "quadratic_boundary_relative_ambiguity"
            ],
        )
        self.assertEqual(tangent.status, "INVALID")
        self.assertEqual(tangent.invalid_reason, "QUADRATIC_BOUNDARY_AMBIGUITY")

        matrix = np.eye(35, dtype=np.float64)
        matrix[-2:, -2:] = 0.0
        gauge = np.zeros((35, 1), dtype=np.float64)
        gauge[-1, 0] = 1.0
        system = build_test_system(matrix, gauge)
        analysis = analyse_reduced_system(
            system,
            column_scale=np.ones(35, dtype=np.float64),
            numeric_contract=numeric,
            compute_particular=False,
        )
        self.assertEqual(analysis.status, "INVALID")
        self.assertEqual(analysis.invalid_reason, "UNEXPECTED_REDUCED_SYSTEM_NULLITY")

    def test_resource_probe_is_exact_and_does_not_solve(self) -> None:
        audit = audit_maximum_payload(self.profile)
        self.assertEqual(audit["status"], "PASS")
        self.assertEqual(audit["matrix_shape"], [35, 35])
        self.assertFalse(audit["particular_solution_computed_in_resource_probe"])
        self.assertFalse(audit["gauge_interval_classified_in_resource_probe"])


def build_test_system(matrix: np.ndarray, gauge: np.ndarray) -> object:
    from next_lab.gauge_aware_fixed_pd_inverse_dynamics import ReducedLocalSystem

    return ReducedLocalSystem(
        matrix=matrix,
        right_hand_side=np.zeros(matrix.shape[0], dtype=np.float64),
        active_point_ordinals=(2, 3),
        modes=np.asarray((0, 3), dtype=np.uint8),
        analytic_gauge_matrix=gauge,
        gauge_identities=(),
    )


if __name__ == "__main__":
    unittest.main()
