from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.exit_mode_owned_projected_inverse_dynamics_formulation import (
    _validate_profile,
    audit_system_inventory,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-exit-mode-owned-projected-inverse-dynamics-formulation-r134.v1.json"
)


def _inventory_sources() -> tuple[dict[str, object], dict[str, object]]:
    r133: dict[str, object] = {
        "projection_result": {
            "aggregate": {
                "flight_identity_collocations": 560,
                "single_point_projection_collocations": 324,
                "flat_foot_projection_collocations": 2316,
            }
        }
    }
    r121: dict[str, object] = {
        "system_inventory_audit": {"friction_second_order_cone_count": 4956}
    }
    return r133, r121


class ProjectedInverseDynamicsFormulationTests(unittest.TestCase):
    def test_profile_authorizes_only_r135_report_only_conformance(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertEqual(
            bounded["r135_projected_inverse_dynamics_implementation_conformance"],
            "AUTHORIZED_REPORT_ONLY_ON_R134_COMPLETE",
        )
        self.assertTrue(
            all(
                value.startswith("NOT_AUTHORIZED")
                for key, value in bounded.items()
                if key != "r135_projected_inverse_dynamics_implementation_conformance"
            )
        )

    def test_profile_rejects_premature_r136_execution_authority(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        invalid = deepcopy(profile)
        invalid["bounded_acceptance"]["r136_projected_inverse_dynamics_execution"] = (
            "AUTHORIZED"
        )
        with self.assertRaisesRegex(ValueError, "formulation profile differs"):
            _validate_profile(invalid)

    def test_warm_acceleration_is_scale_and_diagnostic_only(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        contract = profile["acceleration_semantics_contract"]
        self.assertEqual(
            contract["r120_acceleration"],
            "NONAUTHORITATIVE_SCALE_AND_DIAGNOSTIC_REFERENCE_ONLY",
        )
        self.assertIn("may not enter", contract["right_hand_side_prohibition"])
        self.assertIn("h(q,v)", contract["recomputed_velocity_terms"])

    def test_inventory_reduces_fixed_effort_system_to_29_32_35(self) -> None:
        r133, r121 = _inventory_sources()
        audit = audit_system_inventory(r133=r133, r121=r121)
        self.assertEqual(audit["collocations"], 3200)
        self.assertEqual(audit["reduced_decision_scalars"], 107668)
        self.assertEqual(audit["algebraic_equality_rows"], 107668)
        self.assertEqual(audit["expected_independent_equality_rank"], 105352)
        self.assertEqual(audit["exact_force_gauge_scalars"], 2316)

    def test_inventory_rejects_incomplete_collocation_partition(self) -> None:
        r133, r121 = _inventory_sources()
        r133["projection_result"]["aggregate"][  # type: ignore[index]
            "flat_foot_projection_collocations"
        ] = 2315
        with self.assertRaisesRegex(ValueError, "source inventory is incomplete"):
            audit_system_inventory(r133=r133, r121=r121)

    def test_future_execution_must_reproduce_r133_before_id(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        lineage = profile["state_and_effort_lineage_contract"]
        future = profile["future_r136_execution_contract"]
        self.assertIn("match every R133 array hash", lineage["reconstruction_policy"])
        self.assertIn(
            "byte hashes must match before ID begins", future["upstream_reconstruction"]
        )
        self.assertEqual(future["authority"], "NOT_GRANTED_UNTIL_EXACT_R135_PASS")


if __name__ == "__main__":
    unittest.main()
