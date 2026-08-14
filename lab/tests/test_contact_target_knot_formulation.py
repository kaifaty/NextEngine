from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from typing import Any

import numpy as np

from next_lab.contact_target_knot_formulation import (
    ACTION_IDS,
    CHECK_ID,
    COEFFICIENT_GRID_BASIS_POINTS,
    FORMULATION_ID,
    IMMUTABLE_FRAME_OFFSETS,
    KNOT_FRAME_OFFSETS,
    apply_anchor,
    array_sha256,
    build_target_knot_formulation,
    canonical_json,
    interpolate_coefficients,
    round_divide_ties_to_even,
    sha256,
)


class ContactTargetKnotFormulationTests(unittest.TestCase):
    def test_hash_closed_inputs_enumerate_unique_read_only_lattice(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = _fixture(Path(temporary))
            report = build_target_knot_formulation(
                profile_path=fixture["profile"],
                source_audit_path=fixture["source_audit"],
                r102_audit_path=fixture["r102"],
                v7_manifest_path=fixture["v7"],
                v9_manifest_path=fixture["v9"],
                tool_path=Path(__file__),
                repository={"commit": "a" * 40, "dirty": False},
            )

            self.assertEqual(report["check"], CHECK_ID)
            self.assertEqual(report["status"], "COMPLETE")
            self.assertEqual(
                report["gate_decision"],
                "PERMIT_R104_OFFLINE_LATTICE_PREFLIGHT_ONLY",
            )
            self.assertEqual(len(report["candidate_lattice"]), 27)
            self.assertEqual(len(report["stage_1_prefix_lattice"]), 9)
            self.assertEqual(report["candidate_artifacts_built"], 0)
            self.assertEqual(report["physx_runs"], 0)
            self.assertEqual(
                report["bounded_acceptance"]["candidate_search"],
                "NOT_AUTHORIZED",
            )
            hashes = {row["target_sha256"] for row in report["candidate_lattice"]}
            self.assertEqual(len(hashes), 27)

    def test_r102_file_tampering_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = _fixture(Path(temporary))
            report = json.loads(fixture["r102"].read_bytes())
            report["findings"]["boundary_state_is_sufficient"] = True
            _write_json(fixture["r102"], report)

            with self.assertRaisesRegex(ValueError, "R102 report file"):
                build_target_knot_formulation(
                    profile_path=fixture["profile"],
                    source_audit_path=fixture["source_audit"],
                    r102_audit_path=fixture["r102"],
                    v7_manifest_path=fixture["v7"],
                    v9_manifest_path=fixture["v9"],
                    tool_path=Path(__file__),
                    repository={"commit": "a" * 40, "dirty": False},
                )

    def test_separate_control_target_semantics_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = _fixture(Path(temporary))
            profile = json.loads(fixture["profile"].read_bytes())
            profile["reference_semantics"][
                "separate_control_target_array_allowed"
            ] = True
            _write_json(fixture["profile"], profile)

            with self.assertRaisesRegex(ValueError, "profile is invalid"):
                build_target_knot_formulation(
                    profile_path=fixture["profile"],
                    source_audit_path=fixture["source_audit"],
                    r102_audit_path=fixture["r102"],
                    v7_manifest_path=fixture["v7"],
                    v9_manifest_path=fixture["v9"],
                    tool_path=Path(__file__),
                    repository={"commit": "a" * 40, "dirty": False},
                )

    def test_integer_interpolation_preserves_prefix_and_ties_to_even(self) -> None:
        alpha = interpolate_coefficients((0, 5_000, 10_000))
        self.assertEqual(alpha[:2].tolist(), [0, 0])
        self.assertEqual(alpha[2], 0)
        self.assertEqual(alpha[6], 5_000)
        self.assertEqual(alpha[11], 10_000)
        self.assertEqual(round_divide_ties_to_even(1, 2), 0)
        self.assertEqual(round_divide_ties_to_even(3, 2), 2)
        self.assertEqual(round_divide_ties_to_even(-1, 2), 0)
        self.assertEqual(round_divide_ties_to_even(-3, 2), -2)

        baseline = np.zeros((12, 23), dtype=np.int64)
        delta = np.ones((12, 23), dtype=np.int64)
        candidate = apply_anchor(baseline, delta, alpha)
        np.testing.assert_array_equal(candidate[:2], baseline[:2])


def _fixture(root: Path) -> dict[str, Path]:
    source_audit = root / "source-audit.json"
    _write_json(source_audit, {"check": "fixture"})

    r102 = root / "r102.json"
    r102_report = {
        "audit_id": "nextengine.humanoid-contact-native-rollout-audit.v1",
        "status": "COMPLETE",
        "findings": {
            "boundary_state_is_sufficient": False,
            "manual_boundary_substitution_disposition": "EXHAUSTED",
        },
        "future_candidate_contract": {"status": "FORMULATION_ONLY"},
        "bounded_acceptance": {"candidate_search": "NOT_AUTHORIZED"},
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "trajectory_mutations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
    }
    r102_report["report_sha256"] = hashlib.sha256(
        canonical_json(r102_report)
    ).hexdigest()
    _write_json(r102, r102_report)

    v9_target = np.zeros((12, 23), dtype=np.int64)
    v7_target = v9_target.copy()
    for frame in range(2, 12):
        v7_target[frame, 6] = frame * 10
        v7_target[frame, 10] = -frame
    v7 = _manifest(
        root=root,
        role="v7",
        prototype_id="nextengine.humanoid-contact-manifold-prototype.v7",
        source_audit=source_audit,
        target=v7_target,
        complete_target=None,
    )
    complete_target = np.zeros((801, 23), dtype=np.int64)
    complete_target[238:250] = v9_target
    v9 = _manifest(
        root=root,
        role="v9",
        prototype_id="nextengine.humanoid-contact-manifold-prototype.v9",
        source_audit=source_audit,
        target=v9_target,
        complete_target=complete_target,
    )
    v7_manifest = json.loads(v7.read_bytes())
    v9_manifest = json.loads(v9.read_bytes())
    delta = v7_target - v9_target
    profile = root / "profile.json"
    _write_json(
        profile,
        {
            "schema_version": 1,
            "formulation_id": FORMULATION_ID,
            "status": "FrozenResearchOnly",
            "claim": "OptimizerFreeReferenceTargetKnotFormulationOnly",
            "source": {
                "audit_sha256": sha256(source_audit),
                "r102": {
                    "audit_id": r102_report["audit_id"],
                    "report_sha256": r102_report["report_sha256"],
                    "report_file_sha256": sha256(r102),
                },
                "v7": _profile_source(v7, v7_manifest),
                "v9": _profile_source(v9, v9_manifest),
            },
            "scope": {
                "case_count": 1,
                "clip_id": "cmu16-walk-nominal-b",
                "source_case_ordinal": 7967,
                "frame_first": 238,
                "frame_last": 249,
                "case_frame_count": 12,
                "complete_clip_frame_count": 801,
            },
            "reference_semantics": {
                "candidate_target_equals_reference_joint_position": True,
                "separate_control_target_array_allowed": False,
                "realized_rollout_state_can_replace_reference": False,
                "frame_0_state_mutable": False,
                "controller_semantics_mutable": False,
            },
            "formulation": {
                "action_channel_ids": list(ACTION_IDS),
                "immutable_frame_offsets": list(IMMUTABLE_FRAME_OFFSETS),
                "knot_frame_offsets": list(KNOT_FRAME_OFFSETS),
                "coefficient_scale_basis_points": 10_000,
                "coefficient_grid_basis_points": list(
                    COEFFICIENT_GRID_BASIS_POINTS
                ),
                "interpolation": (
                    "piecewise-linear coefficient with integer "
                    "ties-to-even rounding"
                ),
            },
            "expected_anchor": {
                "v7_target_sha256": array_sha256(v7_target),
                "v9_target_sha256": array_sha256(v9_target),
                "full_delta_sha256": array_sha256(delta),
                "mutable_delta_sha256": array_sha256(delta[2:]),
                "full_changed_element_count": int(np.count_nonzero(delta)),
                "mutable_changed_element_count": int(
                    np.count_nonzero(delta[2:])
                ),
                "mutable_support_action_ordinals": [6, 10],
                "maximum_absolute_mutable_delta_microradians": 110,
            },
            "offline_preflight": {"next_run_id": "R104"},
            "conditional_native_evaluation": {"status": "NOT_AUTHORIZED"},
            "decision": {
                "complete": "PERMIT_R104_OFFLINE_LATTICE_PREFLIGHT_ONLY",
                "offline_candidate_preflight": "AUTHORIZED",
                "candidate_search": "NOT_AUTHORIZED",
                "physx": "NOT_AUTHORIZED",
                "all_17": "NOT_AUTHORIZED",
                "full_v19": "NOT_AUTHORIZED",
                "training": "NOT_AUTHORIZED",
            },
        },
    )
    return {
        "profile": profile,
        "source_audit": source_audit,
        "r102": r102,
        "v7": v7,
        "v9": v9,
    }


def _manifest(
    *,
    root: Path,
    role: str,
    prototype_id: str,
    source_audit: Path,
    target: np.ndarray,
    complete_target: np.ndarray | None,
) -> Path:
    directory = root / role
    cases = directory / "cases"
    cases.mkdir(parents=True)
    artifact = cases / "case.npz"
    np.savez(
        artifact,
        joint_position_urad=target,
        reference_frame=np.arange(238, 250, dtype=np.int64),
    )
    manifest: dict[str, Any] = {
        "schema_version": 1,
        "status": "PASS",
        "prototype_id": prototype_id,
        "scope": {"case_scope": "all"},
        "identities": {
            "source_audit_sha256": sha256(source_audit),
            "prototype_profile_sha256": hashlib.sha256(role.encode()).hexdigest(),
        },
        "cases": [
            {
                "clip_id": "cmu16-walk-nominal-b",
                "frame_first": 238,
                "frame_last": 249,
                "source_case_ordinal": 7967,
                "baseline_status": "PASS",
                "baseline_reasons": [],
                "target_failure_categories": [],
                "artifact": _artifact_record(artifact, directory),
            }
        ],
        "optimizer_steps": 0,
        "training_runs": 0,
    }
    if complete_target is not None:
        clips = directory / "clips"
        clips.mkdir()
        complete = clips / "complete.npz"
        np.savez(complete, joint_position_urad=complete_target)
        manifest["complete_clips"] = [
            {
                "clip_id": "cmu16-walk-nominal-b",
                "frame_first": 0,
                "frame_last": 800,
                "solve_count": 1,
                "projection_diagnostics": {"status": "PASS"},
                "artifact": _artifact_record(complete, directory),
            }
        ]
    manifest["manifest_sha256"] = hashlib.sha256(
        canonical_json(manifest)
    ).hexdigest()
    path = directory / "prototype-manifest.json"
    _write_json(path, manifest)
    return path


def _profile_source(path: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    result = {
        "prototype_id": manifest["prototype_id"],
        "manifest_sha256": manifest["manifest_sha256"],
        "manifest_file_sha256": sha256(path),
        "prototype_profile_sha256": manifest["identities"][
            "prototype_profile_sha256"
        ],
        "case_artifact_sha256": manifest["cases"][0]["artifact"]["sha256"],
    }
    if "complete_clips" in manifest:
        result["complete_clip_artifact_sha256"] = manifest["complete_clips"][0][
            "artifact"
        ]["sha256"]
    return result


def _artifact_record(path: Path, root: Path) -> dict[str, Any]:
    return {
        "relative_path": str(path.relative_to(root)),
        "sha256": sha256(path),
        "bytes": path.stat().st_size,
    }


def _write_json(path: Path, value: Any) -> None:
    path.write_bytes(
        (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    )


if __name__ == "__main__":
    unittest.main()
