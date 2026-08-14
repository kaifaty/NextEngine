from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_boundary_velocity_counterfactual import (
    ARRAY_NAME,
    CASE_ORDINAL,
    COUNTERFACTUAL_CHECK,
    COUNTERFACTUAL_ID,
    DOF_ORDINAL,
    FRAME_OFFSET,
    JOINT_ID,
    SOURCE_CASE_ORDINAL,
    V7_PROTOTYPE_ID,
    V7_VALUE,
    V9_PROTOTYPE_ID,
    V9_VALUE,
    build_boundary_velocity_counterfactual,
    canonical_json,
    sha256,
)
from next_lab.contact_boundary_velocity_vector_counterfactual import (
    CHANGED_DOF_ORDINALS,
    COUNTERFACTUAL_CHECK as VECTOR_COUNTERFACTUAL_CHECK,
    COUNTERFACTUAL_ID as VECTOR_COUNTERFACTUAL_ID,
    DELTA_VECTOR_SHA256,
    V7_VECTOR_SHA256,
    V9_VECTOR_SHA256,
    build_boundary_velocity_vector_counterfactual,
)
from next_lab.contact_manifold_physx import load_contact_prototype_cases


_V9_VECTOR = (
    90, -253830, 0, -2486820, 0, -53280, -98520, -1500, 0, 1520550,
    -717720, -90, 105840, 432870, -78990, 44040, -242490, -500160, 0,
    107700, -178380, -218430, 0,
)
_V7_VECTOR = (
    120, -254040, 0, -2463960, 0, -40080, -409800, 2040, 0, 1426500,
    -754500, 0, 110700, 423780, -80880, 43200, -233820, -533280, 0,
    109260, -186480, -216240, 0,
)


class ContactBoundaryVelocityCounterfactualTests(unittest.TestCase):
    def test_builder_changes_exactly_one_boundary_velocity_cell(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            audit_path = root / "source-audit.json"
            source_ordinals = [
                SOURCE_CASE_ORDINAL if ordinal == CASE_ORDINAL else ordinal
                for ordinal in range(17)
            ]
            source_frames = [
                238 if ordinal == CASE_ORDINAL else ordinal * 20
                for ordinal in range(17)
            ]
            _write_json(
                audit_path,
                {
                    "check": (
                        "TRAIN-4-ISAAC-EXHAUSTIVE-"
                        "DYNAMIC-REFERENCE-FEASIBILITY"
                    ),
                    "results": {
                        "phase_results": [
                            {
                                "case_ordinal": source_ordinal,
                                "clip_id": "clip",
                                "split": "train",
                                "start_frame": frame_first,
                                "terminal_frame": frame_first + 11,
                                "status": "PASS",
                                "first_required_safety_violation": None,
                            }
                            for source_ordinal, frame_first in zip(
                                source_ordinals, source_frames, strict=True
                            )
                        ]
                    },
                },
            )
            v7_path = _source_manifest(
                root=root,
                source_audit_path=audit_path,
                prototype_id=V7_PROTOTYPE_ID,
                initial_velocity_vector=_V7_VECTOR,
                source_ordinals=source_ordinals,
                source_frames=source_frames,
            )
            v9_path = _source_manifest(
                root=root,
                source_audit_path=audit_path,
                prototype_id=V9_PROTOTYPE_ID,
                initial_velocity_vector=_V9_VECTOR,
                source_ordinals=source_ordinals,
                source_frames=source_frames,
            )
            v7 = json.loads(v7_path.read_bytes())
            v9 = json.loads(v9_path.read_bytes())
            profile_path = root / "profile.json"
            _write_json(
                profile_path,
                {
                    "schema_version": 1,
                    "counterfactual_id": COUNTERFACTUAL_ID,
                    "status": "FrozenResearchOnly",
                    "source": {
                        "audit_sha256": sha256(audit_path),
                        "v7": _profile_source(v7_path, v7),
                        "v9": _profile_source(v9_path, v9),
                    },
                    "edit": {
                        "array": ARRAY_NAME,
                        "frame_offset": FRAME_OFFSET,
                        "dof_ordinal": DOF_ORDINAL,
                        "joint_id": JOINT_ID,
                        "source_value_microradians_per_second": V9_VALUE,
                        "replacement_value_microradians_per_second": V7_VALUE,
                        "delta_microradians_per_second": V7_VALUE - V9_VALUE,
                    },
                    "execution": {
                        "generated_case_count": 1,
                        "fresh_scene_runs": 0,
                        "physx_runs": 0,
                        "indexed_partial_reset_enabled": False,
                        "all_17_enabled": False,
                        "optimizer_steps": 0,
                        "training_runs": 0,
                    },
                },
            )
            staging = root / "staging"
            staging.mkdir()
            report = build_boundary_velocity_counterfactual(
                profile_path=profile_path,
                source_audit_path=audit_path,
                v7_manifest_path=v7_path,
                v9_manifest_path=v9_path,
                staging_directory=staging,
                tool_path=Path(__file__),
                repository={"commit": "test", "dirty": False},
            )

            self.assertEqual(report["check"], COUNTERFACTUAL_CHECK)
            self.assertEqual(
                report["gate_decision"],
                "PERMIT_EXACTLY_ONE_R100_FRESH_TRACE_ONLY",
            )
            self.assertEqual(
                report["offline_verification"]["changed_array_element_count"],
                1,
            )
            artifact_path = staging / report["cases"][0]["artifact"][
                "relative_path"
            ]
            with np.load(artifact_path, allow_pickle=False) as candidate:
                self.assertEqual(
                    int(candidate[ARRAY_NAME][FRAME_OFFSET, DOF_ORDINAL]),
                    V7_VALUE,
                )
            manifest_path = staging / "prototype-manifest.json"
            _write_json(manifest_path, report)
            _, _, cases = load_contact_prototype_cases(
                manifest_path=manifest_path,
                source_audit_path=audit_path,
            )
            self.assertEqual(len(cases), 1)
            self.assertEqual(cases[0].source_case_ordinal, SOURCE_CASE_ORDINAL)

            vector_profile_path = root / "vector-profile.json"
            _write_json(
                vector_profile_path,
                {
                    "schema_version": 1,
                    "counterfactual_id": VECTOR_COUNTERFACTUAL_ID,
                    "status": "FrozenResearchOnly",
                    "source": {
                        "audit_sha256": sha256(audit_path),
                        "v7": _profile_source(v7_path, v7),
                        "v9": _profile_source(v9_path, v9),
                    },
                    "edit": {
                        "array": ARRAY_NAME,
                        "frame_offset": FRAME_OFFSET,
                        "dof_scope": "all-ordered-joints",
                        "changed_dof_ordinals": list(CHANGED_DOF_ORDINALS),
                        "changed_dof_count": len(CHANGED_DOF_ORDINALS),
                        "source_vector_sha256": V9_VECTOR_SHA256,
                        "replacement_vector_sha256": V7_VECTOR_SHA256,
                        "delta_vector_sha256": DELTA_VECTOR_SHA256,
                    },
                    "execution": {
                        "generated_case_count": 1,
                        "fresh_scene_runs": 0,
                        "physx_runs": 0,
                        "indexed_partial_reset_enabled": False,
                        "all_17_enabled": False,
                        "optimizer_steps": 0,
                        "training_runs": 0,
                    },
                },
            )
            vector_staging = root / "vector-staging"
            vector_staging.mkdir()
            vector_report = build_boundary_velocity_vector_counterfactual(
                profile_path=vector_profile_path,
                source_audit_path=audit_path,
                v7_manifest_path=v7_path,
                v9_manifest_path=v9_path,
                staging_directory=vector_staging,
                tool_path=Path(__file__),
                repository={"commit": "test", "dirty": False},
            )
            self.assertEqual(vector_report["check"], VECTOR_COUNTERFACTUAL_CHECK)
            self.assertEqual(
                vector_report["offline_verification"][
                    "changed_array_element_count"
                ],
                18,
            )
            vector_artifact = vector_staging / vector_report["cases"][0][
                "artifact"
            ]["relative_path"]
            with np.load(vector_artifact, allow_pickle=False) as candidate:
                np.testing.assert_array_equal(
                    candidate[ARRAY_NAME][FRAME_OFFSET], _V7_VECTOR
                )


def _source_manifest(
    *,
    root: Path,
    source_audit_path: Path,
    prototype_id: str,
    initial_velocity_vector: tuple[int, ...],
    source_ordinals: list[int],
    source_frames: list[int],
) -> Path:
    directory = root / prototype_id.rsplit(".", 1)[-1]
    case_directory = directory / "cases"
    case_directory.mkdir(parents=True)
    records = []
    for ordinal, (source_ordinal, frame_first) in enumerate(
        zip(source_ordinals, source_frames, strict=True)
    ):
        artifact_path = case_directory / f"case-{ordinal:02d}.npz"
        arrays = _arrays(frame_first)
        if ordinal == CASE_ORDINAL:
            arrays[ARRAY_NAME][FRAME_OFFSET] = initial_velocity_vector
        np.savez(artifact_path, **arrays)
        records.append(
            {
                "clip_id": "clip",
                "split": "train",
                "frame_first": frame_first,
                "frame_last": frame_first + 11,
                "source_case_ordinal": source_ordinal,
                "baseline_status": "PASS",
                "baseline_reasons": [],
                "target_failure_categories": [],
                "exact_complete_clip_slice_status": "PASS",
                "projection_diagnostics": {
                    "status": "PASS",
                    "final_contact_reprojection_dropped_point_count": 0,
                },
                "artifact": {
                    "relative_path": str(artifact_path.relative_to(directory)),
                    "sha256": sha256(artifact_path),
                    "bytes": artifact_path.stat().st_size,
                },
            }
        )
    profile_hash = hashlib.sha256(prototype_id.encode()).hexdigest()
    manifest = {
        "schema_version": 1,
        "check": "TRAIN-4-CONTACT-MANIFOLD-PROTOTYPE-BUILD",
        "status": "PASS",
        "prototype_id": prototype_id,
        "scope": {
            "case_count": 17,
            "case_scope": "all",
            "clip_ids": ["clip"],
            "failure_case_count": 7,
            "control_case_count": 10,
        },
        "identities": {
            "source_audit_sha256": sha256(source_audit_path),
            "prototype_profile_sha256": profile_hash,
            "corpus_manifest_sha256": "a" * 64,
            "corpus_manifest_file_sha256": "b" * 64,
            "descriptor_sha256": "c" * 64,
        },
        "cases": records,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
    }
    manifest["manifest_sha256"] = hashlib.sha256(
        canonical_json(manifest)
    ).hexdigest()
    manifest_path = directory / "prototype-manifest.json"
    _write_json(manifest_path, manifest)
    return manifest_path


def _arrays(frame_first: int) -> dict[str, np.ndarray]:
    frame_count = 12
    quaternion = np.zeros((frame_count, 4), dtype=np.int64)
    quaternion[:, 3] = 1 << 30
    return {
        "center_of_mass_um": np.zeros((frame_count, 3), dtype=np.int64),
        "contact_modes": np.full((frame_count, 2), "FLIGHT"),
        "contacts": np.zeros((frame_count, 7), dtype=np.uint8),
        "effector_position_um": np.zeros(
            (frame_count, 6, 3), dtype=np.int64
        ),
        "joint_position_urad": np.zeros((frame_count, 23), dtype=np.int64),
        "joint_velocity_urad_s": np.zeros((frame_count, 23), dtype=np.int64),
        "metadata_json_utf8": np.frombuffer(b"{}", dtype=np.uint8),
        "phase_u16": np.zeros(frame_count, dtype=np.uint16),
        "reference_frame": np.arange(
            frame_first, frame_first + frame_count, dtype=np.int64
        ),
        "root_linear_velocity_um_s": np.zeros(
            (frame_count, 3), dtype=np.int64
        ),
        "root_position_um": np.zeros((frame_count, 3), dtype=np.int64),
        "root_quaternion_q1_30": quaternion,
        "root_yaw_urad": np.zeros(frame_count, dtype=np.int64),
        "root_yaw_velocity_urad_s": np.zeros(frame_count, dtype=np.int64),
    }


def _profile_source(path: Path, manifest: dict[str, object]) -> dict[str, object]:
    return {
        "prototype_id": manifest["prototype_id"],
        "manifest_sha256": manifest["manifest_sha256"],
        "manifest_file_sha256": sha256(path),
        "prototype_profile_sha256": manifest["identities"][
            "prototype_profile_sha256"
        ],
        "case_artifact_sha256": manifest["cases"][CASE_ORDINAL]["artifact"][
            "sha256"
        ],
    }


def _write_json(path: Path, value: object) -> None:
    path.write_bytes(
        (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    )


if __name__ == "__main__":
    unittest.main()
