from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from next_lab.contact_target_knot_formulation import (
    CHECK_ID as R103_CHECK_ID,
    FORMULATION_ID,
    canonical_json,
    sha256,
)
from next_lab.contact_target_knot_preflight import (
    ZERO_CONTROL_ID,
    _validate_r103,
    candidate_failure_reasons,
    preflight_summary,
    worst_limit_utilization_basis_points,
)


_LIMITS = {
    "maximum_normal_residual_micrometres": 5_000,
    "maximum_normal_step_micrometres": 1_000,
    "maximum_tangential_step_micrometres": 2_000,
    "maximum_analytic_normal_step_micrometres": 1_000,
    "maximum_analytic_tangential_step_micrometres": 2_000,
    "minimum_collider_height_micrometres": -2,
    "maximum_root_vertical_velocity_micrometres_per_second": 200_060,
    "maximum_joint_velocity_basis_points": 2_500,
    "maximum_soft_rom_violation_microradians": 0,
}


class ContactTargetKnotPreflightTests(unittest.TestCase):
    def test_failure_reasons_are_exact_and_not_compensable(self) -> None:
        metrics = _metrics()
        metrics.update(
            {
                "maximum_normal_residual_micrometres": 5_001,
                "maximum_tangential_step_micrometres": 2_001,
                "maximum_joint_velocity_basis_points": 2_531,
                "minimum_collider_height_micrometres": -3,
                "maximum_soft_rom_violation_microradians": 1,
            }
        )
        self.assertEqual(
            candidate_failure_reasons(metrics=metrics, limits=_LIMITS),
            [
                "contact_normal_residual",
                "contact_finite_tangential",
                "joint_velocity",
                "collider_floor",
                "soft_rom",
            ],
        )
        self.assertEqual(
            worst_limit_utilization_basis_points(
                metrics=metrics, limits=_LIMITS
            ),
            10_124,
        )

    def test_only_zero_control_pass_keeps_native_unauthorized(self) -> None:
        rows = [_row(ZERO_CONTROL_ID, "PASS", 10_000, [])]
        for index in range(1, 27):
            rows.append(
                _row(
                    f"candidate-{index:02d}",
                    "FAIL",
                    10_200 + index,
                    ["joint_velocity"],
                )
            )
        rows[1]["worst_limit_utilization_basis_points"] = 10_124
        summary = preflight_summary(rows)

        self.assertEqual(summary["pass_count"], 1)
        self.assertEqual(summary["nonzero_pass_count"], 0)
        self.assertEqual(summary["nonzero_fail_count"], 26)
        self.assertEqual(
            summary["best_failed_nonzero_candidate"]["candidate_id"],
            "candidate-01",
        )

    def test_r103_canonical_or_file_tampering_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "r103.json"
            report = _r103_report()
            _write_json(path, report)
            expected = {
                "report_sha256": report["report_sha256"],
                "report_file_sha256": sha256(path),
                "profile_sha256": "b" * 64,
            }
            loaded = _validate_r103(report_path=path, expected=expected)
            self.assertEqual(len(loaded["candidate_lattice"]), 27)

            report["candidate_lattice"][1]["target_sha256"] = "f" * 64
            _write_json(path, report)
            with self.assertRaisesRegex(ValueError, "R103 report file"):
                _validate_r103(report_path=path, expected=expected)


def _metrics() -> dict[str, int | str]:
    return {
        "exact_state_status": "PASS",
        "active_contact_status": "PASS",
        "collider_closure_status": "PASS",
        "maximum_normal_residual_micrometres": 4_900,
        "maximum_normal_step_micrometres": 900,
        "maximum_tangential_step_micrometres": 1_900,
        "maximum_analytic_normal_step_micrometres": 900,
        "maximum_analytic_tangential_step_micrometres": 1_900,
        "minimum_collider_height_micrometres": 10,
        "maximum_root_vertical_velocity_micrometres_per_second": 190_000,
        "maximum_joint_velocity_basis_points": 2_400,
        "maximum_soft_rom_violation_microradians": 0,
    }


def _row(
    candidate_id: str, status: str, utilization: int, reasons: list[str]
) -> dict[str, object]:
    return {
        "candidate_id": candidate_id,
        "status": status,
        "failure_reasons": reasons,
        "worst_limit_utilization_basis_points": utilization,
        "maximum_absolute_target_correction_microradians": 10,
        "metrics": _metrics(),
    }


def _r103_report() -> dict[str, object]:
    report: dict[str, object] = {
        "check": R103_CHECK_ID,
        "formulation_id": FORMULATION_ID,
        "status": "COMPLETE",
        "gate_decision": "PERMIT_R104_OFFLINE_LATTICE_PREFLIGHT_ONLY",
        "identities": {"profile_sha256": "b" * 64},
        "candidate_lattice": [
            {
                "candidate_id": (
                    ZERO_CONTROL_ID if index == 0 else f"candidate-{index:02d}"
                ),
                "target_sha256": hashlib.sha256(str(index).encode()).hexdigest(),
            }
            for index in range(27)
        ],
        "bounded_acceptance": {
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
        },
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": 0,
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
