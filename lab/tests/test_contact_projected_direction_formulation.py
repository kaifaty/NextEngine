from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from next_lab.contact_projected_direction_formulation import (
    SELECTED_BASIS_ID,
    _validate_profile,
    validate_r105,
)
from next_lab.contact_target_knot_formulation import canonical_json, sha256


class ContactProjectedDirectionFormulationTests(unittest.TestCase):
    def test_tracked_profile_freezes_formulation_only(self) -> None:
        path = (
            Path(__file__).resolve().parents[1]
            / "profiles"
            / "humanoid-contact-projected-direction-formulation-r106.v1.json"
        )
        profile = json.loads(path.read_bytes())
        _validate_profile(profile)
        self.assertFalse(
            profile["reconstruction"]["projection_qp_execution_in_r106"]
        )
        self.assertFalse(
            profile["quantization"]["candidate_construction_in_r106"]
        )

    def test_r105_canonical_or_file_tampering_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "r105.json"
            report = _r105_report()
            _write_json(path, report)
            expected = {
                "report_sha256": report["report_sha256"],
                "report_file_sha256": sha256(path),
                "profile_sha256": "a" * 64,
            }
            loaded = validate_r105(report_path=path, expected=expected)
            self.assertEqual(
                loaded["summary"]["useful_projection_ids"],
                [SELECTED_BASIS_ID],
            )

            report["basis_results"][2]["useful_projection"] = False
            _write_json(path, report)
            with self.assertRaisesRegex(ValueError, "R105 report file"):
                validate_r105(report_path=path, expected=expected)


def _r105_report() -> dict[str, object]:
    rows = [
        {
            "basis_id": "anchor-offset-02",
            "useful_projection": False,
            "knot_coefficients_basis_points": [10_000, 0, 0],
            "projection": {},
        },
        {
            "basis_id": "anchor-offset-06",
            "useful_projection": False,
            "knot_coefficients_basis_points": [0, 10_000, 0],
            "projection": {},
        },
        {
            "basis_id": SELECTED_BASIS_ID,
            "useful_projection": True,
            "knot_coefficients_basis_points": [0, 0, 10_000],
            "projection": {
                "anchor_component_basis_points": 9722,
                "cosine_similarity_basis_points": 9872,
                "quantized_nonzero_joint_cell_count": 45,
                "quantized_maximum_absolute_joint_correction_microradians": 5688,
                "quantized_maximum_root_correction_micrometres": 41,
            },
        },
    ]
    report: dict[str, object] = {
        "check": "TRAIN-4-CONTACT-FEASIBLE-DIRECTION-AUDIT",
        "audit_id": "nextengine.humanoid-contact-feasible-direction-audit.v1",
        "status": "COMPLETE",
        "gate_decision": (
            "PERMIT_R106_EXACT_OFFLINE_PROJECTED_DIRECTION_FORMULATION_ONLY"
        ),
        "identities": {"profile_sha256": "a" * 64},
        "repository": {"dirty": False},
        "basis_results": rows,
        "summary": {
            "basis_count": 3,
            "useful_projection_count": 1,
            "useful_projection_ids": [SELECTED_BASIS_ID],
        },
        "linearization": {
            "local_support_variable_count": 130,
            "local_relevant_constraint_count": 600,
            "zero_direction_maximum_normalized_violation": 0.0,
            "complete_clip_proxy_acceptance_authority": False,
            "binding_rows": {
                "near_binding_row_count": 69,
                "near_binding_rank": 69,
                "local_nullity": 61,
            },
        },
        "bounded_acceptance": {
            "r106_exact_offline_formulation": "AUTHORIZED",
            "candidate_artifact": "NOT_AUTHORIZED",
            "exact_nonlinear_candidate_audit": "NOT_AUTHORIZED",
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
        "basis_constructions": 3,
        "local_projection_qp_solves": 3,
        "candidate_target_constructions": 0,
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
