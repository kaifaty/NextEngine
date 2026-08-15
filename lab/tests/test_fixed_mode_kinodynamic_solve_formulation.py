from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.fixed_mode_kinodynamic_solve_formulation import (
    EXACT_TRIAL_AUDIT_LIMIT,
    MAJOR_ITERATION_LIMIT,
    QP_PASSES_PER_MAJOR_ITERATION,
    QP_SOLVE_LIMIT,
    TRIAL_FRACTIONS_PER_MAJOR_ITERATION,
    _validate_profile,
    audit_numeric_and_budget_contract,
    audit_required_hash_contract,
    audit_solve_formulation,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-fixed-mode-kinodynamic-solve-formulation-r139.v1.json"
)


class FixedModeKinodynamicSolveFormulationTests(unittest.TestCase):
    def test_profile_authorizes_only_r140_report_only_conformance(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r140_kinodynamic_solve_implementation_conformance"],
            "AUTHORIZED_REPORT_ONLY_ON_R139_COMPLETE",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r140_kinodynamic_solve_implementation_conformance"
            )
        )
        self.assertEqual(
            profile["result_transitions"]["complete"],
            "R139_COMPLETE_R140_CONFORMANCE_ONLY",
        )

    def test_profile_rejects_premature_real_solve_authority(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        invalid = deepcopy(profile)
        invalid["bounded_acceptance"]["kinodynamic_solve"] = "AUTHORIZED"
        with self.assertRaisesRegex(ValueError, "formulation profile differs"):
            _validate_profile(invalid)

    def test_source_reconstruction_forbids_r136_payload_and_freezes_initial_state(
        self,
    ) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_solve_formulation(profile)
        source = profile["source_reconstruction_contract"]
        endpoint = profile["initial_and_endpoint_contract"]
        self.assertEqual(source["r139_source_array_payload_reads"], 0)
        self.assertEqual(
            source["r120_cache_access_in_r139"],
            "FORBIDDEN_REPORT_METADATA_ONLY",
        )
        self.assertEqual(source["r136_cache_acceleration_or_force_import"], "FORBIDDEN")
        self.assertEqual(
            audit["initial_configuration_source"], "R120_ACCEPTED_EXACT_ROW_0"
        )
        self.assertEqual(
            audit["initial_velocity_source"], "R133_PROJECTED_VELOCITY_ROW_0"
        )
        self.assertTrue(endpoint["initial_configuration"]["immutable"])
        self.assertTrue(endpoint["initial_velocity"]["immutable"])
        self.assertFalse(endpoint["terminal"]["byte_fixed"])
        self.assertFalse(endpoint["terminal"]["periodic"])
        self.assertTrue(endpoint["terminal"]["all_hard_constraints_required"])

    def test_objective_is_three_pass_lexicographic_feasibility_first(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        objective = profile["objective_hierarchy_contract"]
        passes = objective["lexicographic_qp_passes"]
        self.assertEqual([row["pass"] for row in passes], [1, 2, 3])
        self.assertEqual(
            [row["objective"] for row in passes],
            [
                "MINIMIZE_MAXIMUM_NORMALIZED_ELASTIC_VIOLATION",
                "MINIMIZE_MEAN_NORMALIZED_ELASTIC_VIOLATION",
                "MINIMIZE_TRACKING_AND_REGULARIZATION",
            ],
        )
        self.assertEqual(passes[1]["maximum_optimum_hold_tolerance"], 1e-8)
        self.assertEqual(passes[2]["maximum_optimum_hold_tolerance"], 1e-8)
        self.assertEqual(passes[2]["mean_optimum_hold_tolerance"], 1e-8)
        self.assertEqual(objective["first_exact_hard_constraint_pass"], "TERMINATE")

    def test_continuous_controller_is_candidate_only_and_exact_replay_accepts(
        self,
    ) -> None:
        profile = json.loads(PROFILE.read_bytes())
        surrogate = profile["candidate_generator_contract"]["controller_surrogate"]
        exact = profile["exact_acceptance_contract"]
        self.assertEqual(surrogate["branch_equality_derivative"], 0)
        self.assertEqual(surrogate["free_effort"], "FORBIDDEN")
        self.assertEqual(surrogate["acceptance_authority"], "NONE")
        self.assertEqual(
            exact["controller_replay"],
            "EXACT_R133_ORDER_TIES_TO_EVEN_FROM_TRIAL_Q_V_AND_INTEGER_COMMAND",
        )
        self.assertEqual(exact["surrogate_effort_authority"], "NONE_DISCARDED")
        self.assertEqual(
            exact["changed_float_command_rounding_to_same_integer"],
            "NO_COMMAND_PROGRESS",
        )

    def test_trust_and_future_budget_formulas_are_closed(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_numeric_and_budget_contract(profile)
        trust = profile["normalization_and_trust_region_contract"]
        self.assertEqual(
            trust["radius"], {"initial": 1, "minimum": 1 / 32, "maximum": 2}
        )
        self.assertEqual(
            len(trust["trial_fractions"]), TRIAL_FRACTIONS_PER_MAJOR_ITERATION
        )
        self.assertEqual(
            QP_SOLVE_LIMIT, MAJOR_ITERATION_LIMIT * QP_PASSES_PER_MAJOR_ITERATION
        )
        self.assertEqual(
            EXACT_TRIAL_AUDIT_LIMIT,
            MAJOR_ITERATION_LIMIT * TRIAL_FRACTIONS_PER_MAJOR_ITERATION,
        )
        self.assertEqual(audit["qp_solve_limit"], 18)
        self.assertEqual(audit["exact_trial_audit_limit"], 42)

    def test_polyhedral_cone_never_has_acceptance_authority(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        cone = profile["candidate_generator_contract"]["friction_cone_model"]
        exact = profile["exact_acceptance_contract"]
        self.assertEqual(cone["base_angular_half_spaces"], 32)
        self.assertIn("AT_MOST_ONE_DETERMINISTIC_DIRECTION", cone["separation_update"])
        self.assertEqual(cone["acceptance_authority"], "EXACT_CIRCULAR_CONE_ONLY")
        self.assertEqual(exact["polyhedral_cone_authority"], "NONE")

    def test_numeric_rank_gap_and_ambiguity_policy_fail_closed(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        numeric = profile["numeric_validity_contract"]
        rank = numeric["rank_policy"]
        self.assertEqual(rank["null_singular_value_maximum"], 1e-12)
        self.assertEqual(rank["retained_singular_value_minimum"], 1e-10)
        self.assertEqual(rank["expected_contact_ranks"], [0, 3, 5])
        self.assertEqual(rank["rank_gap_values"], "INVALID")
        self.assertIn(
            "norm versus squared-cone classification disagreement",
            numeric["invalid_without_restart"],
        )
        invalid = deepcopy(profile)
        invalid["numeric_validity_contract"]["rank_policy"][
            "retained_singular_value_minimum"
        ] = 1e-9
        with self.assertRaisesRegex(ValueError, "numeric or resource budget"):
            audit_numeric_and_budget_contract(invalid)

    def test_required_hash_categories_and_counter_closure_are_complete(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_required_hash_contract(profile)
        self.assertEqual(audit["before_first_real_graph_call_hashes"], 10)
        self.assertEqual(audit["every_accepted_anchor_hashes"], 8)
        self.assertEqual(audit["future_execution_output_hashes"], 14)
        self.assertEqual(audit["required_counter_closure"], 20)
        invalid = deepcopy(profile)
        invalid["required_hash_contract"]["future_execution_output"].remove(
            "exact_applied_effort"
        )
        with self.assertRaisesRegex(ValueError, "required hash contract"):
            audit_required_hash_contract(invalid)


if __name__ == "__main__":
    unittest.main()
