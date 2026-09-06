#!/usr/bin/env python3
"""Contract tests for deterministic V24 T0+X0 V3 assembly."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v24_v3_assemble as assembler


def write(path: Path, data: bytes) -> dict[str, str]:
    path.write_bytes(data)
    return {"path": path.name, "sha256": assembler.sha256_bytes(data)}


def lanes(root: Path) -> tuple[Path, Path]:
    evidence = write(root / "evidence.json", b'{"fixture":true}\n')
    geometry = write(root / "geometry.bin", b"geometry")
    modal = write(root / "modal.bin", b"modal")
    gain = write(root / "gain.bin", b"gain")
    t0_claim = assembler.T0_CLAIM
    x0_claim = assembler.X0_CLAIM
    t0_lineage = write(
        root / "t0-lineage.json",
        assembler.canonical_json({"schema": "t0.fixture.v1", "status": "Validated", "claim": t0_claim}),
    )
    x0_lineage = write(
        root / "x0-lineage.json",
        assembler.canonical_json({"schema": "x0.fixture.v1", "status": "Validated", "claim": x0_claim}),
    )
    rows = []
    for role_index, role in enumerate(assembler.ROLES):
        for sample_index, sample_role in enumerate(("context", "query")):
            audio = write(root / f"t0-{role}-{sample_index}.wav", bytes([role_index, sample_index]))
            rows.append(
                {
                    "row_id": f"t0-{role}-{sample_index}",
                    "split_role": role,
                    "sample_role": sample_role,
                    "corpus_role": "target",
                    "evidence_lane": "synthetic_teacher",
                    "audio_semantics": "synthetic_modal_render",
                    "source_group_id": f"t0-source-{role}",
                    "family_group_id": "analytic-plate",
                    "object_group_id": f"t0-object-{role}",
                    "recording_parent_id": f"t0-recording-{role}-{sample_index}",
                    "condition_group_id": f"t0-condition-{role}-{sample_index}",
                    "mutation_parent_id": None,
                    "lineage_report_ids": ["t0-lineage"],
                    "audio": audio,
                    "audio_provenance": t0_lineage,
                    "axes": {
                        "material": {"value_id": "elastic-a", "evidence": evidence},
                        "geometry": {"geometry_id": f"geometry-{role}", "feature_artifact": geometry, "evidence": evidence},
                        "support": {"value_id": "simply-supported", "evidence": evidence},
                        "impact": {"coordinate_profile": "fixture", "point_metres": [0.0, 0.0, 0.0], "outward_normal": [0.0, 0.0, 1.0], "evidence": evidence},
                        "listener": {"coordinate_profile": "fixture", "point_metres": [0.0, 0.0, 1.0], "evidence": evidence},
                        "excitation": {"impulse_newton_seconds": 1.0, "evidence": evidence},
                        "teacher_target": {
                            "representation_id": "sorted-modal-contact-field-v1",
                            "mode_count": 2,
                            "modal_parameters": modal,
                            "contact_gain_field": gain,
                            "evidence": evidence,
                        },
                    },
                }
            )
    rows.sort(key=lambda row: row["row_id"])
    t0 = {
        "schema": assembler.T0_SCHEMA,
        "status": "Validated",
        "claim": t0_claim,
        "lineage_report": {
            "id": "t0-lineage",
            "expected_schema": "t0.fixture.v1",
            "expected_claim": t0_claim,
            "artifact": t0_lineage,
        },
        "rows": rows,
    }
    x0_rows = []
    for index in range(7):
        transfer = index < 4
        audio = write(root / f"x0-{index}.bin", bytes([100 + index]))
        axes = {"material": {"value_id": "glass", "evidence": x0_lineage}}
        if transfer:
            axes.update(
                {
                    "geometry": {"geometry_id": "blue-bowl", "feature_artifact": geometry, "evidence": x0_lineage},
                    "impact": {"coordinate_profile": "fixture", "point_metres": [float(index), 0.0, 0.0], "evidence": x0_lineage},
                    "listener": {"coordinate_profile": "fixture", "point_metres": [0.0, 0.0, 1.0], "evidence": x0_lineage},
                }
            )
        x0_rows.append(
            {
                "row_id": f"x0-{index}",
                "split_role": "development",
                "sample_role": "query" if index in {3, 6} else "context",
                "corpus_role": "target",
                "evidence_lane": "exact_real_transfer" if transfer else "identified_real_recording",
                "audio_semantics": "force_deconvolved_transfer_response" if transfer else "recorded_impact_waveform",
                "source_group_id": "x0-transfer" if transfer else "x0-recording",
                "family_group_id": "blue-bowl-glass",
                "object_group_id": "x0-blue-bowl",
                "recording_parent_id": f"x0-recording-{index}",
                "condition_group_id": f"x0-condition-{index}",
                "mutation_parent_id": None,
                "lineage_report_ids": ["x0-lineage"],
                "audio": audio,
                "audio_provenance": x0_lineage,
                "axes": axes,
            }
        )
    x0 = {
        "schema": assembler.X0_SCHEMA,
        "status": "Validated",
        "claim": x0_claim,
        "lineage_report": {
            "id": "x0-lineage",
            "expected_schema": "x0.fixture.v1",
            "expected_claim": x0_claim,
            "artifact": x0_lineage,
        },
        "rows": x0_rows,
    }
    t0_path = root / "t0-lane.json"
    x0_path = root / "x0-lane.json"
    t0_path.write_bytes(assembler.canonical_json(t0))
    x0_path.write_bytes(assembler.canonical_json(x0))
    return t0_path, x0_path


class PhysicalSoundV24V3AssembleTests(unittest.TestCase):
    def test_full_entry_twice_is_exact_and_hash_closes_all_refs(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-v3-test-") as temporary:
            root = Path(temporary)
            t0, x0 = lanes(root)
            first = root / "run-a"
            second = root / "run-b"
            assembler.run(t0, x0, first)
            assembler.run(t0, x0, second)
            self.assertEqual(
                {path.name: path.read_bytes() for path in first.iterdir()},
                {path.name: path.read_bytes() for path in second.iterdir()},
            )
            manifest = json.loads((first / "combined-manifest.json").read_bytes())
            report = json.loads((first / "report.json").read_bytes())
            self.assertEqual(manifest["schema"], assembler.MANIFEST_SCHEMA)
            self.assertEqual(len(manifest["rows"]), 17)
            self.assertEqual(len(manifest["lineage_reports"]), 2)
            self.assertEqual(report["lane_counts"], {
                "synthetic_teacher": 10,
                "exact_real_transfer": 4,
                "identified_real_recording": 3,
            })
            self.assertFalse(report["model_training_authorized"])
            for row in manifest["rows"]:
                self.assertTrue(Path(row["audio"]["path"]).is_absolute())

    def test_lane_identity_hash_coverage_duplicates_and_output_reject(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-v3-negative-") as temporary:
            root = Path(temporary)
            t0, x0 = lanes(root)
            value = json.loads(t0.read_bytes())
            value["claim"] = "wrong"
            bad_identity = root / "bad-identity.json"
            bad_identity.write_bytes(assembler.canonical_json(value))
            with self.assertRaisesRegex(ValueError, "identity changed"):
                assembler.run(bad_identity, x0, root / "identity-output")

            value = json.loads(t0.read_bytes())
            value["rows"][0]["audio"]["sha256"] = "0" * 64
            bad_hash = root / "bad-hash.json"
            bad_hash.write_bytes(assembler.canonical_json(value))
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                assembler.run(bad_hash, x0, root / "hash-output")

            value = json.loads(t0.read_bytes())
            value["rows"] = [row for row in value["rows"] if row["split_role"] != "calibration"]
            missing_role = root / "missing-role.json"
            missing_role.write_bytes(assembler.canonical_json(value))
            with self.assertRaisesRegex(ValueError, "every role"):
                assembler.run(missing_role, x0, root / "coverage-output")

            value = json.loads(x0.read_bytes())
            value["rows"][1]["row_id"] = value["rows"][0]["row_id"]
            duplicate = root / "duplicate.json"
            duplicate.write_bytes(assembler.canonical_json(value))
            with self.assertRaises(ValueError):
                assembler.run(t0, duplicate, root / "duplicate-output")

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(ValueError, "new external path"):
                assembler.run(t0, x0, occupied)

    def test_failed_build_cleans_staging_and_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-v3-atomic-") as temporary:
            root = Path(temporary)
            t0, x0 = lanes(root)
            output = root / "never-published"
            with mock.patch.object(assembler, "build_into", side_effect=ValueError("forced")):
                with self.assertRaisesRegex(ValueError, "forced"):
                    assembler.run(t0, x0, output)
            self.assertFalse(output.exists())
            self.assertFalse(any(path.name.startswith(".nextengine-v24-v3-") for path in root.iterdir()))


if __name__ == "__main__":
    unittest.main()
