from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np
from scipy import sparse

from next_lab.contact_feasible_direction_audit import (
    analyze_binding_rows,
    build_raw_basis,
    constraint_violations,
    project_direction,
    projection_is_useful,
    projection_metrics,
    _validate_r104,
)
from next_lab.contact_target_knot_formulation import canonical_json, sha256


class ContactFeasibleDirectionAuditTests(unittest.TestCase):
    def test_raw_basis_maps_descriptor_ordinals_into_v9_leg_order(self) -> None:
        baseline = np.zeros((12, 23), dtype=np.int64)
        anchor_delta = np.zeros_like(baseline)
        anchor_delta[11, 6] = 1_000
        selected = np.asarray([0, 1, 3, 4, 5, 6, 7, 9, 10, 11])
        scale = np.asarray([0.02] * 3 + [0.1] * 10)
        raw = build_raw_basis(
            coefficients=(0, 0, 10_000),
            v9_target=baseline,
            anchor_delta=anchor_delta,
            support_frame_first=240,
            support_frame_last=249,
            local_variable_count=13,
            local_scale=scale,
            selected_dof_ordinals=selected,
        ).reshape(10, 13)

        self.assertEqual(np.count_nonzero(raw), 1)
        self.assertAlmostEqual(raw[-1, 8], 0.01)

    def test_projection_preserves_closest_feasible_direction(self) -> None:
        matrix = sparse.eye(2, format="csc")
        lower = np.asarray([-1.0, -1.0])
        upper = np.asarray([1.0, 1.0])
        raw = np.asarray([2.0, 0.5])
        projected, solve = project_direction(
            raw=raw,
            matrix=matrix,
            lower=lower,
            upper=upper,
            solver={
                "absolute_tolerance": 1.0e-9,
                "relative_tolerance": 1.0e-9,
                "maximum_iterations": 10_000,
                "polishing_enabled": True,
                "adaptive_rho_enabled": False,
            },
        )
        np.testing.assert_allclose(projected, [1.0, 0.5], atol=1.0e-8)
        self.assertTrue(solve["status"].lower().startswith("solved"))

        metrics = projection_metrics(
            raw=raw,
            projected=projected,
            local_scale=np.asarray([0.02, 0.1]),
            support_frame_count=1,
        )
        self.assertEqual(metrics["anchor_component_basis_points"], 5294)
        self.assertEqual(metrics["quantized_nonzero_joint_cell_count"], 0)

    def test_violation_and_binding_rows_retain_categories_and_rank(self) -> None:
        matrix = sparse.csc_matrix(
            np.asarray([[1.0, 0.0], [0.0, 1.0], [1.0, 1.0]])
        )
        lower = np.asarray([-1.0, -1.0, -2.0])
        upper = np.asarray([0.0, 0.04, 2.0])
        categories = ("contact", "velocity", "contact")
        analysis = analyze_binding_rows(
            matrix=matrix,
            lower=lower,
            upper=upper,
            row_categories=categories,
            binding_tolerance=1.0e-9,
            near_binding_tolerance=0.05,
            rank_relative_tolerance=1.0e-10,
        )
        self.assertEqual(analysis["binding_row_count"], 1)
        self.assertEqual(analysis["near_binding_row_count"], 2)
        self.assertEqual(analysis["near_binding_rank"], 2)
        self.assertEqual(
            analysis["near_binding_category_counts"],
            {"contact": 1, "velocity": 1},
        )

        violations = constraint_violations(
            matrix=matrix,
            lower=lower,
            upper=upper,
            value=np.asarray([0.2, 0.1]),
            row_categories=categories,
            tolerance=1.0e-9,
        )
        self.assertEqual(violations["row_count"], 2)
        self.assertEqual(
            violations["category_counts"],
            {"contact": 1, "velocity": 1},
        )

    def test_useful_projection_threshold_is_predeclared(self) -> None:
        thresholds = {
            "minimum_anchor_component_basis_points": 1000,
            "minimum_cosine_similarity_basis_points": 5000,
            "minimum_quantized_nonzero_joint_cell_count": 1,
        }
        metrics = {
            "anchor_component_basis_points": 1000,
            "cosine_similarity_basis_points": 5000,
            "quantized_nonzero_joint_cell_count": 1,
        }
        self.assertTrue(
            projection_is_useful(metrics=metrics, thresholds=thresholds)
        )
        metrics["anchor_component_basis_points"] = 999
        self.assertFalse(
            projection_is_useful(metrics=metrics, thresholds=thresholds)
        )

    def test_r104_canonical_or_file_tampering_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "r104.json"
            report = _r104_report()
            _write_json(path, report)
            expected = {
                "report_sha256": report["report_sha256"],
                "report_file_sha256": sha256(path),
                "profile_sha256": "a" * 64,
            }
            loaded = _validate_r104(report_path=path, expected=expected)
            self.assertEqual(loaded["summary"]["nonzero_fail_count"], 26)

            report["summary"]["nonzero_fail_count"] = 25
            _write_json(path, report)
            with self.assertRaisesRegex(ValueError, "R104 report file"):
                _validate_r104(report_path=path, expected=expected)


def _r104_report() -> dict[str, object]:
    report: dict[str, object] = {
        "check": "TRAIN-4-CONTACT-TARGET-KNOT-OFFLINE-PREFLIGHT",
        "preflight_id": "nextengine.humanoid-contact-target-knot-preflight.v1",
        "status": "COMPLETE",
        "gate_decision": "STOP_AND_RESEARCH",
        "identities": {
            "profile_sha256": "a" * 64,
            "r103_report_sha256": "b" * 64,
        },
        "summary": {
            "candidate_count": 27,
            "nonzero_pass_count": 0,
            "nonzero_fail_count": 26,
        },
        "bounded_acceptance": {
            "r105_native_candidate_evaluation": "NOT_AUTHORIZED"
        },
        "candidate_artifacts_built": 0,
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def _write_json(path: Path, value: object) -> None:
    path.write_bytes(
        (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    )


if __name__ == "__main__":
    unittest.main()
