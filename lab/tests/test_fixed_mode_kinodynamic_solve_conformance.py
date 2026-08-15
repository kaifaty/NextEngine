from __future__ import annotations

import json
import math
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.fixed_mode_kinodynamic_solve_conformance import (
    ZERO_REAL_COUNTERS,
    _validate_profile,
    audit_budget_and_hash_closure,
    audit_controller_boundary_cases,
    audit_friction_cone_cases,
    audit_lexicographic_qp_schema,
    audit_numeric_invalid_cases,
    audit_reconstruction_gate,
    audit_trust_funnel_cases,
    build_lexicographic_qp_schema,
    classify_contact_rank,
    decide_major_outcome,
    fixed_branch_derivative,
    round_decimal_ties_even,
    round_fraction_ties_even,
    validate_reconstruction_gate,
    validate_solver_candidate,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-fixed-mode-kinodynamic-solve-conformance-r140.v1.json"
)
R139_PROFILE = (
    ROOT / "lab/profiles/humanoid-fixed-mode-kinodynamic-solve-formulation-r139.v1.json"
)


class FixedModeKinodynamicSolveConformanceTests(unittest.TestCase):
    def test_profile_authorizes_only_r141_roadmap_decision(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r141_bounded_kinodynamic_solve_roadmap_decision"],
            "AUTHORIZED_DECISION_ONLY_ON_R140_PASS",
        )
        self.assertEqual(
            bounded["r141_bounded_kinodynamic_solve"],
            "NOT_AUTHORIZED_UNTIL_EXPLICIT_ROADMAP_UPDATE",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r141_bounded_kinodynamic_solve_roadmap_decision"
            )
        )

    def test_reconstruction_gate_rejects_mismatch_and_premature_graph(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        expected = profile["reconstruction_gate_contract"]["required_source_hashes"]
        passing = validate_reconstruction_gate(expected, expected, real_graph_calls=0)
        self.assertEqual(passing["identity_count"], 10)

        changed = dict(expected)
        changed["r120_cache"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "source hash closure differs"):
            validate_reconstruction_gate(expected, changed, real_graph_calls=0)
        with self.assertRaisesRegex(ValueError, "graph call preceded"):
            validate_reconstruction_gate(expected, expected, real_graph_calls=1)

        audit = audit_reconstruction_gate(
            profile,
            r139={"required_hash_contract": {"before_first_real_graph_call": expected}},
        )
        self.assertEqual(audit["case_count"], 3)
        self.assertEqual(audit["rejected_case_count"], 2)

    def test_three_pass_qp_schema_holds_prior_optima(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_lexicographic_qp_schema(profile)
        self.assertEqual(audit["case_count"], 3)
        self.assertEqual(len(audit["passes"][0]["optimum_hold_rows"]), 0)
        self.assertEqual(len(audit["passes"][1]["optimum_hold_rows"]), 1)
        self.assertEqual(len(audit["passes"][2]["optimum_hold_rows"]), 2)
        self.assertEqual(
            audit["passes"][2]["optimum_hold_rows"][1]["upper_bound"],
            "0.25000001",
        )
        with self.assertRaisesRegex(ValueError, "pass ordinal differs"):
            build_lexicographic_qp_schema(0)

    def test_trust_funnel_cases_cover_accept_reject_restore_and_stop(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_trust_funnel_cases(profile)
        self.assertEqual(audit["case_count"], 7)
        decisions = {row["case"]: row["decision"] for row in audit["cases"]}
        self.assertEqual(
            decisions["exact_pass_priority"], "ACCEPT_AND_TERMINATE_EXACT_PASS"
        )
        self.assertEqual(
            decisions["insufficient_funnel_decrease"],
            "REJECT_RESTORE_AND_CONTRACT",
        )
        self.assertEqual(decisions["accepted_contract"], "ACCEPT_AND_CONTRACT")
        self.assertEqual(decisions["accepted_expand"], "ACCEPT_AND_EXPAND")
        self.assertEqual(
            decisions["minimum_radius_stop"], "STOP_AND_RESEARCH_AT_MINIMUM_RADIUS"
        )
        with self.assertRaisesRegex(ValueError, "trust input differs"):
            decide_major_outcome(
                radius="4",
                exact_pass=False,
                exact_maximum_decrease="1",
                exact_mean_decrease="0",
                actual_to_model_ratio="1",
                active_trust_boundary=True,
            )

    def test_controller_boundary_is_independent_ties_even_and_zero_equality_derivative(
        self,
    ) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_controller_boundary_cases(profile)
        self.assertEqual(audit["rounding_case_count"], 6)
        self.assertEqual(audit["rounding_differential_disagreements"], 0)
        self.assertEqual(audit["surrogate_exact_difference_count"], 6)
        self.assertEqual(audit["branch_equality_ambiguity_count"], 2)
        self.assertEqual(audit["free_effort_scalars"], 0)
        for numerator, expected in (
            (-5, -2),
            (-3, -2),
            (-1, 0),
            (1, 0),
            (3, 2),
            (5, 2),
        ):
            self.assertEqual(round_fraction_ties_even(numerator, 2), expected)
            self.assertEqual(round_decimal_ties_even(numerator, 2), expected)
        equality = fixed_branch_derivative("1", "-1", "1")
        self.assertEqual(equality["derivative"], 0)
        self.assertTrue(equality["ambiguity_recorded"])

    def test_cone_candidate_rows_and_exact_circle_authority_are_separate(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_friction_cone_cases(profile)
        self.assertEqual(audit["base_half_space_rows"], 32)
        self.assertEqual(audit["classification_case_count"], 3)
        self.assertEqual(audit["separation_case_count"], 3)
        self.assertTrue(audit["exact_circular_cone_acceptance_authority"])
        self.assertFalse(audit["polyhedral_acceptance_authority"])
        self.assertEqual(
            [row["graph_row"] for row in audit["separation_directions"]],
            [2, 2, 7],
        )

    def test_numeric_gap_nonfinite_bounds_cone_and_solver_cases_fail_closed(
        self,
    ) -> None:
        profile = json.loads(PROFILE.read_bytes())
        audit = audit_numeric_invalid_cases(profile)
        self.assertEqual(audit["valid_case_count"], 5)
        self.assertEqual(audit["invalid_case_count"], 8)
        self.assertEqual(classify_contact_rank([1e-12, 1e-10, 1e-9, 1e-8]), 3)
        with self.assertRaisesRegex(ValueError, "inside rank gap"):
            classify_contact_rank([1.1e-12])
        with self.assertRaisesRegex(ValueError, "non-finite"):
            classify_contact_rank([math.nan])
        with self.assertRaisesRegex(ValueError, "solver status"):
            validate_solver_candidate(
                status="maximum iterations reached",
                primal_residual=0.0,
                dual_residual=0.0,
            )

    def test_budget_hash_and_zero_counter_closure_match_r139(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        r139_profile = json.loads(R139_PROFILE.read_bytes())
        r139 = {
            "future_execution_budget_contract": r139_profile[
                "future_execution_budget_contract"
            ],
            "required_hash_contract": r139_profile["required_hash_contract"],
        }
        audit = audit_budget_and_hash_closure(profile, r139=r139)
        self.assertEqual(audit["qp_solve_limit"], 6 * 3)
        self.assertEqual(audit["exact_trial_audit_limit"], 6 * 7)
        self.assertEqual(audit["pre_graph_hash_count"], 10)
        self.assertEqual(audit["accepted_anchor_hash_count"], 8)
        self.assertEqual(audit["output_hash_count"], 14)
        self.assertEqual(audit["counter_closure_count"], len(ZERO_REAL_COUNTERS))

    def test_profile_rejects_any_real_work_counter(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        invalid = deepcopy(profile)
        invalid["scope"]["real_qp_setups"] = 1
        with self.assertRaisesRegex(ValueError, "conformance profile differs"):
            _validate_profile(invalid)


if __name__ == "__main__":
    unittest.main()
