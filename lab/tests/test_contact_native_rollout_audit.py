from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from typing import Any

from next_lab.contact_native_rollout_audit import (
    AUDIT_ID,
    build_native_rollout_audit,
    canonical_json,
)


class ContactNativeRolloutAuditTests(unittest.TestCase):
    def test_hash_closed_sources_build_report_only_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, reports = _fixture(root)
            report = build_native_rollout_audit(
                profile_path=profile_path,
                source_report_paths=reports,
                tool_path=Path(__file__),
                repository={"commit": "a" * 40, "dirty": False, "dirty_paths": []},
            )

            self.assertEqual(report["status"], "COMPLETE")
            self.assertEqual(report["gate_decision"], "STOP_AND_RESEARCH")
            self.assertEqual(report["physx_runs"], 0)
            self.assertEqual(report["candidate_evaluations"], 0)
            self.assertEqual(
                report["bounded_acceptance"]["candidate_search"],
                "NOT_AUTHORIZED",
            )
            self.assertTrue(
                report["findings"]["r101_applied_targets_match_v9"]
            )

    def test_worker_tampering_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, reports = _fixture(root)
            worker_path = root / "r101" / "fresh-scene" / "results" / "case.json"
            worker = json.loads(worker_path.read_bytes())
            worker["native_dynamics_trace"]["samples"][0][
                "requested_effort_micronewton_metres"
            ][0] += 1
            worker_path.write_bytes(_pretty(worker))

            with self.assertRaisesRegex(ValueError, "worker identity"):
                build_native_rollout_audit(
                    profile_path=profile_path,
                    source_report_paths=reports,
                    tool_path=Path(__file__),
                    repository={
                        "commit": "a" * 40,
                        "dirty": False,
                        "dirty_paths": [],
                    },
                )

    def test_predeclared_expected_finding_is_enforced(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, reports = _fixture(root)
            profile = json.loads(profile_path.read_bytes())
            profile["expected"] = {
                "findings": {"r101_applied_targets_match_v9": False}
            }
            profile_path.write_bytes(_pretty(profile))

            with self.assertRaisesRegex(ValueError, "expected.findings"):
                build_native_rollout_audit(
                    profile_path=profile_path,
                    source_report_paths=reports,
                    tool_path=Path(__file__),
                    repository={
                        "commit": "a" * 40,
                        "dirty": False,
                        "dirty_paths": [],
                    },
                )


def _fixture(root: Path) -> tuple[Path, dict[str, Path]]:
    roles = {
        "v7": ("v7-passing-control", 1),
        "v9": ("v9-baseline", 2),
        "r100": ("v9-v7-boundary-velocity", 3),
        "r101": ("v9-v7-boundary-velocity-vector", 1),
    }
    reports: dict[str, Path] = {}
    source: dict[str, Any] = {}
    for index, (role, (comparison_role, velocity)) in enumerate(roles.items(), 1):
        report_path, source_row = _write_source(
            root=root,
            role=role,
            comparison_role=comparison_role,
            velocity=velocity,
            identity_character=f"{index:x}",
        )
        reports[role] = report_path
        source[role] = source_row
    profile = {
        "schema_version": 1,
        "audit_id": AUDIT_ID,
        "status": "FrozenResearchOnly",
        "claim": "OptimizerFreeReportOnlyNativeRolloutAudit",
        "source": source,
        "scope": {
            "input_roles": ["v7", "v9", "r100", "r101"],
            "case_count": 1,
            "clip_id": "cmu16-walk-nominal-b",
            "start_frame": 238,
            "source_case_ordinal": 7967,
        },
        "measurement": {
            "comparison_prefix_substeps": 20,
            "remote_comparison_motor_tick": 5,
            "left_roll_action_id": "actuator.left-ankle-roll",
            "right_pitch_action_id": "actuator.right-ankle-pitch",
            "remote_contact_pair_id": "ground:body.right-ankle-roll",
        },
        "expected": {},
        "future_candidate_contract": {
            "status": "FORMULATION_ONLY",
            "candidate_search": "NOT_AUTHORIZED",
            "required_safety_priority": "LEXICOGRAPHIC_REJECT_BEFORE_TRACKING",
            "fresh_scene_required": True,
            "physical_substep_trace_required": True,
            "frame_0_state_mutable": False,
            "controller_semantics_mutable": False,
            "safety_limits_mutable": False,
            "reset_semantics_mutable": False,
        },
        "decision": {
            "complete": "STOP_AND_RESEARCH",
            "candidate_search": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
    }
    profile_path = root / "profile.json"
    profile_path.write_bytes(_pretty(profile))
    return profile_path, reports


def _write_source(
    *,
    root: Path,
    role: str,
    comparison_role: str,
    velocity: int,
    identity_character: str,
) -> tuple[Path, dict[str, Any]]:
    source_root = root / role
    worker_path = source_root / "fresh-scene" / "results" / "case.json"
    worker_path.parent.mkdir(parents=True)
    action_ids = ["actuator.left-ankle-roll", "actuator.right-ankle-pitch"]
    pair_ids = ["ground:body.right-ankle-roll"]
    samples = [
        {
            "motor_tick": index // 4 + 1,
            "physics_substep": index % 4,
            "applied_target_microradians": [0, 0],
            "command_reference_target_microradians": [0, 0],
            "pre_physics_joint_position_microradians": [0, -index],
            "pre_physics_joint_velocity_microradians_per_second": [
                velocity,
                index,
            ],
            "published_effort_micronewton_metres": [0, index],
            "requested_effort_micronewton_metres": [0, index + 1],
            "episode_contact_pair_maximum_impulse_micronewton_seconds": [
                index
            ],
            "motor_tick_contact_pair_maximum_impulse_micronewton_seconds": [
                index
            ],
            "hard_impact_by_contact_pair": [False],
        }
        for index in range(20)
    ]
    trace = {
        "schema_version": 1,
        "enabled": True,
        "evidence_role": "report-only",
        "comparison_role": comparison_role,
        "action_channel_ids": action_ids,
        "contact_pair_ids": pair_ids,
        "observed_motor_ticks": 5,
        "physics_substeps_per_motor_tick": 4,
        "expected_sample_count": 20,
        "sample_count": 20,
        "coverage_complete": True,
        "samples": samples,
    }
    trace["trace_sha256"] = hashlib.sha256(canonical_json(trace)).hexdigest()
    phase_result = {
        "status": "PASS",
        "first_required_safety_violation": None,
    }
    worker = {
        "phase_result": phase_result,
        "contact_pair_hard_limits_micronewton_seconds": [6_000_000],
        "native_dynamics_trace": trace,
        "optimizer_steps": 0,
        "training_runs": 0,
    }
    worker_path.write_bytes(_pretty(worker))
    worker_sha256 = _sha256(worker_path)
    manifest_sha256 = identity_character * 64
    probe_profile_sha256 = identity_character * 64
    report = {
        "check": "TRAIN-4-ISAAC-NATIVE-DYNAMICS-TRACE",
        "status": "COMPLETE",
        "claim": "OptimizerFreeReportOnlyNativeDynamicsTrace",
        "gate_decision": "STOP_AND_RESEARCH",
        "manifest_sha256": manifest_sha256,
        "identities": {"probe_profile_sha256": probe_profile_sha256},
        "optimizer_steps": 0,
        "training_runs": 0,
        "indexed_partial_reset": {"status": "NOT_RUN"},
        "repository": {
            "commit": identity_character * 40,
            "dirty": False,
            "dirty_paths": [],
        },
        "fresh_scene": {
            "results": {"phase_results": [phase_result]},
            "workers": [
                {
                    "case_ordinal": 0,
                    "report_relative_path": "fresh-scene/results/case.json",
                    "report_sha256": worker_sha256,
                    "native_dynamics_trace": {
                        "sample_count": 20,
                        "trace_sha256": trace["trace_sha256"],
                    },
                }
            ],
        },
    }
    report_path = source_root / "reset-probe-report.json"
    report_path.write_bytes(_pretty(report))
    return report_path, {
        "role": role,
        "manifest_sha256": manifest_sha256,
        "manifest_file_sha256": _sha256(report_path),
        "probe_profile_sha256": probe_profile_sha256,
        "worker_report_sha256": worker_sha256,
        "trace_sha256": trace["trace_sha256"],
        "worker_case_ordinal": 0,
        "sample_count": 20,
        "comparison_role": comparison_role,
    }


def _pretty(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    unittest.main()
