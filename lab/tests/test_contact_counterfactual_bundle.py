from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_counterfactual_bundle import (
    BUNDLE_CHECK,
    BUNDLE_ID,
    build_counterfactual_bundle,
    canonical_json,
    sha256,
)
from next_lab.contact_manifold_physx import load_contact_prototype_cases


class ContactCounterfactualBundleTests(unittest.TestCase):
    def test_bundle_copies_two_hash_closed_cases(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            audit_path = root / "source-audit.json"
            audit = {
                "check": (
                    "TRAIN-4-ISAAC-EXHAUSTIVE-DYNAMIC-REFERENCE-FEASIBILITY"
                ),
                "results": {
                    "phase_results": [
                        _source_row(25, "contact-clip", 25),
                        _source_row(7967, "derivative-clip", 238),
                    ]
                },
            }
            _write_json(audit_path, audit)

            specifications = []
            manifest_paths = []
            for role, r95_ordinal, source_ordinal, clip_id, frame_first in (
                ("contact-reserve", 2, 25, "contact-clip", 25),
                (
                    "emitted-acceleration",
                    10,
                    7967,
                    "derivative-clip",
                    238,
                ),
            ):
                manifest_path, specification = _source_manifest(
                    root=root,
                    source_audit_path=audit_path,
                    role=role,
                    r95_ordinal=r95_ordinal,
                    source_ordinal=source_ordinal,
                    clip_id=clip_id,
                    frame_first=frame_first,
                )
                manifest_paths.append(manifest_path)
                specifications.append(specification)

            profile_path = root / "bundle-profile.json"
            profile = {
                "schema_version": 1,
                "bundle_id": BUNDLE_ID,
                "status": "FrozenResearchOnly",
                "source_audit_sha256": sha256(audit_path),
                "ordered_counterfactuals": specifications,
                "execution": {
                    "fresh_scene_case_count": 2,
                    "indexed_partial_reset_enabled": False,
                    "all_17_enabled": False,
                    "optimizer_steps": 0,
                    "training_runs": 0,
                },
            }
            _write_json(profile_path, profile)
            staging = root / "staging"
            staging.mkdir()
            report = build_counterfactual_bundle(
                profile_path=profile_path,
                source_audit_path=audit_path,
                source_manifest_paths=manifest_paths,
                staging_directory=staging,
                tool_path=Path(__file__),
                repository={"commit": "test", "dirty": False},
            )
            self.assertEqual(report["check"], BUNDLE_CHECK)
            self.assertEqual(report["scope"]["ordered_r95_case_ordinals"], [2, 10])
            self.assertEqual(
                [row["counterfactual_role"] for row in report["cases"]],
                ["contact-reserve", "emitted-acceleration"],
            )

            bundle_path = staging / "prototype-manifest.json"
            _write_json(bundle_path, report)
            _, _, cases = load_contact_prototype_cases(
                manifest_path=bundle_path,
                source_audit_path=audit_path,
            )
            self.assertEqual(
                tuple(case.source_case_ordinal for case in cases), (25, 7967)
            )


def _source_row(ordinal: int, clip_id: str, frame_first: int) -> dict[str, object]:
    return {
        "case_ordinal": ordinal,
        "clip_id": clip_id,
        "split": "train",
        "start_frame": frame_first,
        "terminal_frame": frame_first + 11,
        "status": "PASS",
        "first_required_safety_violation": None,
    }


def _source_manifest(
    *,
    root: Path,
    source_audit_path: Path,
    role: str,
    r95_ordinal: int,
    source_ordinal: int,
    clip_id: str,
    frame_first: int,
) -> tuple[Path, dict[str, object]]:
    directory = root / role
    cases = directory / "cases"
    cases.mkdir(parents=True)
    artifact_path = cases / f"{clip_id}.npz"
    frame_count = 12
    quaternion = np.zeros((frame_count, 4), dtype=np.int64)
    quaternion[:, 3] = 1 << 30
    np.savez(
        artifact_path,
        center_of_mass_um=np.zeros((frame_count, 3), dtype=np.int64),
        contacts=np.zeros((frame_count, 7), dtype=np.uint8),
        effector_position_um=np.zeros((frame_count, 6, 3), dtype=np.int64),
        joint_position_urad=np.zeros((frame_count, 1), dtype=np.int64),
        joint_velocity_urad_s=np.zeros((frame_count, 1), dtype=np.int64),
        phase_u16=np.zeros(frame_count, dtype=np.uint16),
        reference_frame=np.arange(
            frame_first, frame_first + frame_count, dtype=np.int64
        ),
        root_linear_velocity_um_s=np.zeros((frame_count, 3), dtype=np.int64),
        root_position_um=np.zeros((frame_count, 3), dtype=np.int64),
        root_quaternion_q1_30=quaternion,
        root_yaw_velocity_urad_s=np.zeros(frame_count, dtype=np.int64),
    )
    artifact_hash = sha256(artifact_path)
    profile_hash = hashlib.sha256(f"profile-{role}".encode()).hexdigest()
    manifest = {
        "schema_version": 1,
        "check": "TRAIN-4-CONTACT-MANIFOLD-PROTOTYPE-BUILD",
        "status": "PASS",
        "prototype_id": f"prototype-{role}",
        "scope": {
            "case_count": 1,
            "case_scope": "discriminator",
            "clip_ids": [clip_id],
        },
        "identities": {
            "source_audit_sha256": sha256(source_audit_path),
            "prototype_profile_sha256": profile_hash,
            "corpus_manifest_sha256": "a" * 64,
            "corpus_manifest_file_sha256": "b" * 64,
            "descriptor_sha256": "c" * 64,
        },
        "cases": [
            {
                "clip_id": clip_id,
                "split": "train",
                "frame_first": frame_first,
                "frame_last": frame_first + 11,
                "source_case_ordinal": source_ordinal,
                "baseline_status": "PASS",
                "baseline_reasons": [],
                "target_failure_categories": [],
                "exact_complete_clip_slice_status": "PASS",
                "artifact": {
                    "relative_path": str(artifact_path.relative_to(directory)),
                    "sha256": artifact_hash,
                    "bytes": artifact_path.stat().st_size,
                },
            }
        ],
        "complete_clips": [
            {
                "solve_count": 1,
                "projection_diagnostics": {
                    "status": "PASS",
                    "contact_point_deletion_count": 0,
                },
            }
        ],
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
    }
    manifest["manifest_sha256"] = hashlib.sha256(
        canonical_json(manifest)
    ).hexdigest()
    manifest_path = directory / "prototype-manifest.json"
    _write_json(manifest_path, manifest)
    specification = {
        "role": role,
        "r95_case_ordinal": r95_ordinal,
        "source_case_ordinal": source_ordinal,
        "prototype_id": manifest["prototype_id"],
        "manifest_sha256": manifest["manifest_sha256"],
        "manifest_file_sha256": sha256(manifest_path),
        "prototype_profile_sha256": profile_hash,
        "case_artifact_sha256": artifact_hash,
    }
    return manifest_path, specification


def _write_json(path: Path, value: object) -> None:
    path.write_bytes(
        (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    )


if __name__ == "__main__":
    unittest.main()
