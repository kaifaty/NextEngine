from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.kto_linearization_research import (
    _validate_profile,
    analyze_analytic_row_dependencies,
    analyze_r115_fractions,
    compare_velocity_models,
)
from next_lab.quantization_aware_kto_solver import BLOCK_WIDTH, Q_WIDTH
from scipy import sparse

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-post-r115-kto-linearization-research.v1.json"


class KtoLinearizationResearchTests(unittest.TestCase):
    def test_profile_is_report_only_and_blocks_every_execution(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertEqual(
            profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )
        self.assertEqual(
            profile["bounded_acceptance"]["r116_inverse_dynamics_formulation"],
            "NOT_AUTHORIZED",
        )

    def test_fraction_audit_isolates_analytic_tangent(self) -> None:
        report = {"solver_result": {"exact_audits": _fraction_rows()}}
        analysis = analyze_r115_fractions(report)
        self.assertTrue(analysis["small_fraction_isolation"])
        self.assertGreater(
            analysis["analytic_tangent_affine_r_squared"],
            0.999,
        )
        self.assertEqual(analysis["one_thirty_second_analytic_excess_micrometres"], 368)
        self.assertEqual(
            analysis["one_thirty_second_tracking_improvement_basis_points"],
            615,
        )

    def test_analytic_row_dependency_detects_missing_q_columns(self) -> None:
        matrix = sparse.coo_matrix(
            (
                np.ones(4),
                (
                    np.asarray([0, 1, 2, 3]),
                    np.asarray([0, Q_WIDTH, Q_WIDTH + 1, BLOCK_WIDTH]),
                ),
            ),
            shape=(4, 2 * BLOCK_WIDTH),
        ).tocsc()
        analysis = analyze_analytic_row_dependencies(
            matrix=matrix,
            category_counts={"before": 1, "contact_analytic_velocity": 2, "after": 1},
        )
        self.assertEqual(analysis["configuration_nonzero_count"], 0)
        self.assertEqual(analysis["velocity_nonzero_count"], 2)
        self.assertEqual(analysis["acceleration_nonzero_count"], 0)

    def test_velocity_kernel_comparison_retains_hotspot_identity(self) -> None:
        active = np.zeros((2, 2, 2), dtype=np.bool_)
        active[1, 0, 1] = True
        exact = np.zeros((2, 2, 2, 3), dtype=np.float64)
        model = np.zeros_like(exact)
        exact[1, 0, 1, 0] = 120_000.0
        model[1, 0, 1, 0] = 90_000.0
        analysis = compare_velocity_models(
            model_velocity_um_s=model,
            exact_velocity_um_s=exact,
            active=active,
        )
        self.assertEqual(analysis["exact_maximum_tangential_step_micrometres"], 2000)
        self.assertEqual(analysis["exact_tangent_hotspot"], [1, 0, 1])
        self.assertEqual(analysis["exact_tangent_hotspot_id"], "frame-1:left-forefoot")
        self.assertEqual(analysis["kernel_identity"], "MISMATCH_DIAGNOSTIC_ONLY")


def _fraction_rows() -> list[dict[str, object]]:
    fractions = ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
    analytic = (16753, 9225, 5498, 3673, 2791, 2368)
    emitted = (
        1_847_946,
        4_287_192_374,
        9_643_656_850,
        13_125_104_123,
        15_066_975_034,
        16_087_896_233,
    )
    rows = []
    for index, fraction in enumerate(fractions):
        small = index >= 2
        rows.append(
            {
                "fraction": fraction,
                "failure_reasons": ["contact"] if small else ["contact", "collider"],
                "contact": {
                    "status": "FAIL",
                    "maximum_tangential_step_micrometres": 1990 if small else 2012,
                    "maximum_normal_step_micrometres": 982,
                    "maximum_analytic_tangential_step_micrometres": analytic[index],
                    "maximum_analytic_normal_step_micrometres": 994,
                },
                "minimum_collider_height_micrometres": 37 if small else -166,
                "maximum_root_vertical_velocity_micrometres_per_second": 199800,
                "maximum_joint_velocity_basis_points": 2500,
                "tracking_progress": {
                    "status": "PASS",
                    "baseline_squared_distance_microradians_squared": 17_142_428_727,
                    "emitted_squared_distance_microradians_squared": emitted[index],
                },
            }
        )
    return rows


if __name__ == "__main__":
    unittest.main()
