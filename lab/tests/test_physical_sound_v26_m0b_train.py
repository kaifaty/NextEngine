"""Conformance tests for the V26 M0b surface/alignment preprocessing owner."""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import physical_sound_v25_m0a_common as base
import physical_sound_v26_m0b_alignment as alignment
import physical_sound_v26_m0b_common as contract
import physical_sound_v26_m0b_surface as surface
import physical_sound_v26_m0b_train as trainer
import test_physical_sound_v25_m0a_train as inherited_test


def write(path: Path, data: bytes) -> dict[str, str]:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {"path": str(path), "sha256": base.sha256_bytes(data)}


def _off_vertex_point(row: dict[str, object], face_index: int) -> list[float]:
    mesh_ref = row["axes"]["geometry"]["feature_artifact"]  # type: ignore[index]
    mesh = base.parse_mesh(Path(mesh_ref["path"]).read_bytes(), mesh_ref["sha256"])
    face = mesh.triangles[face_index % len(mesh.triangles)]
    vertices = mesh.vertices[np.asarray(face, dtype=np.int64)]
    point = vertices[0] * 0.2 + vertices[1] * 0.3 + vertices[2] * 0.5
    if np.any(np.all(mesh.vertices == point, axis=1)):
        raise AssertionError("fixture point unexpectedly remained a mesh vertex")
    return point.tolist()


def fixture(root: Path) -> tuple[Path, dict[str, object]]:
    _, inherited_manifest = inherited_test.fixture(root)
    manifest = copy.deepcopy(inherited_manifest)
    combined_path = Path(manifest["artifacts"]["combined_manifest"]["path"])
    combined = json.loads(combined_path.read_bytes())
    synthetic = [
        row
        for row in combined["rows"]
        if row["evidence_lane"] == "synthetic_teacher"
        and row["split_role"] == "train"
        and row["sample_role"] == "context"
    ]
    if not synthetic:
        raise AssertionError("fixture has no synthetic training context")
    synthetic[0]["axes"]["impact"]["point_metres"] = _off_vertex_point(synthetic[0], 3)

    transfer = sorted(
        (
            row
            for row in combined["rows"]
            if row["evidence_lane"] == "exact_real_transfer"
        ),
        key=lambda row: row["row_id"],
    )
    if len(transfer) != 4:
        raise AssertionError("fixture transfer row count changed")
    profile = base.execution_profile("contract-fixture-v1")
    for index, (row, peak) in enumerate(zip(transfer, (87, 73, 39, 69), strict=True)):
        row["axes"]["impact"]["point_metres"] = _off_vertex_point(row, 11 + index * 19)
        samples = np.zeros(profile.transfer_samples, dtype="<f4")
        samples[1_000 + index] = np.float32(-0.125)
        samples[peak] = np.float32(-1.0 if index % 2 else 1.0)
        source_path = Path(row["audio"]["path"])
        row["audio"] = write(source_path, samples.tobytes())

    changed_combined = root / "m0b-combined-manifest.json"
    combined_ref = write(changed_combined, base.canonical_json(combined))
    manifest.update(
        {
            "schema": contract.MANIFEST_SCHEMA,
            "protocol_sha256": contract.PROTOCOL_SHA256,
            "implementation_root_sha256": contract.implementation_root_sha256(),
            "inherited_implementation_hashes": contract.INHERITED_IMPLEMENTATION_HASHES,
        }
    )
    manifest["artifacts"]["combined_manifest"] = combined_ref
    path = root / "m0b-manifest.json"
    path.write_bytes(base.canonical_json(manifest))
    return path, manifest


class PhysicalSoundV26M0bTests(unittest.TestCase):
    def test_alignment_fixture_and_opened_context_facts_repeat_exactly(self) -> None:
        first = alignment.conformance_report("a" * 64)
        second = alignment.conformance_report("a" * 64)
        self.assertEqual(first, second)
        self.assertEqual(first["valid_case_count"], 9)
        self.assertEqual(first["rejection_case_count"], 8)
        official = base.execution_profile("official-v1")
        expected = {
            87: (425, 143_575),
            73: (439, 143_561),
            39: (473, 143_527),
        }
        for peak, (leading, copied) in expected.items():
            source = np.zeros(official.transfer_samples, dtype="<f4")
            source[peak] = np.float32(1.0)
            result = alignment.align_transfer(source.tobytes(), official)
            self.assertEqual(result.source_peak_index, peak)
            self.assertEqual(result.leading_padding_samples, leading)
            self.assertEqual(result.copied_source_samples, copied)
            self.assertEqual(result.trailing_padding_samples, 0)
            self.assertEqual(int(np.argmax(np.abs(result.samples))), 512)

    def test_surface_fixture_is_representation_invariant_and_fail_closed(self) -> None:
        first = surface.official_shape_report("b" * 64)
        second = surface.official_shape_report("b" * 64)
        self.assertEqual(first, second)
        self.assertEqual(first["query_evaluation_count"], 48)
        self.assertEqual(first["coarse_refined_pair_count"], 24)
        self.assertEqual(first["invariance_fixture_count"], 4)
        self.assertGreaterEqual(first["mutation_fixture_count"], 15)
        self.assertLessEqual(
            first["maximum_affine_pair_difference"], surface.AFFINE_PAIR_LIMIT
        )
        surface.validate_report(first, "b" * 64)
        for key, value in (
            ("query_evaluation_count", 47),
            ("implementation_root_sha256", "0" * 64),
            ("model_values_opened", True),
        ):
            corrupt = copy.deepcopy(first)
            corrupt[key] = value
            with self.assertRaises(base.M0Error):
                surface.validate_report(corrupt, "b" * 64)

    def test_full_cli_twice_is_byte_exact_and_owns_nine_artifacts(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v26-m0b-full-"
        ) as temporary:
            root = Path(temporary)
            manifest_path, _ = fixture(root)
            command = [
                sys.executable,
                str(SCRIPTS / "physical_sound_v26_m0b_train.py"),
                "--manifest",
                str(manifest_path),
                "--output",
            ]
            environment = os.environ.copy()
            environment.update(
                {
                    name: "1"
                    for name in (
                        "OMP_NUM_THREADS",
                        "OPENBLAS_NUM_THREADS",
                        "MKL_NUM_THREADS",
                        "NUMEXPR_NUM_THREADS",
                    )
                }
            )
            first = subprocess.run(
                [*command, str(root / "run-a")],
                check=False,
                capture_output=True,
                text=True,
                env=environment,
            )
            second = subprocess.run(
                [*command, str(root / "run-b")],
                check=False,
                capture_output=True,
                text=True,
                env=environment,
            )
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertEqual(second.returncode, 0, second.stderr)
            first_files = inherited_test.files(root / "run-a")
            second_files = inherited_test.files(root / "run-b")
            self.assertEqual(first_files, second_files)
            self.assertEqual(len(first_files), 9)
            self.assertIn("surface-query-report.json", first_files)
            report = json.loads((root / "run-a/report.json").read_bytes())
            self.assertEqual(report["decision"], "PASS")
            self.assertEqual(report["protocol_sha256"], contract.PROTOCOL_SHA256)
            self.assertTrue(report["internal_candidate_ab_byte_exact"])
            self.assertFalse(report["admission_shadow_opened"])
            self.assertFalse(report["realimpact_2407_opened"])
            surface_report = json.loads(
                (root / "run-a/surface-query-report.json").read_bytes()
            )
            self.assertEqual(surface_report["query_evaluation_count"], 48)
            preprocess = json.loads(
                (root / "run-a/preprocess-manifest.json").read_bytes()
            )
            self.assertFalse(preprocess["nearest_vertex_gain_lookup_used"])
            self.assertFalse(preprocess["unpadded_alignment_used"])
            self.assertEqual(preprocess["alignment_record_count"], 3)
            observed_peaks = sorted(
                item["source_peak_index"] for item in preprocess["alignment_records"]
            )
            self.assertEqual(observed_peaks, [39, 73, 87])
            self.assertEqual(preprocess["alignment_conformance"]["valid_case_count"], 9)

    def test_manifest_output_and_atomic_mutations_reject(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v26-m0b-negative-"
        ) as temporary:
            root = Path(temporary)
            manifest_path, manifest = fixture(root)
            bad = copy.deepcopy(manifest)
            bad["protocol_sha256"] = "0" * 64
            bad_path = root / "bad-protocol.json"
            bad_path.write_bytes(base.canonical_json(bad))
            with self.assertRaisesRegex(base.M0Error, "protocol hash"):
                trainer.run(bad_path, root / "bad-protocol-output")
            bad = copy.deepcopy(manifest)
            bad["implementation_root_sha256"] = "0" * 64
            bad_path = root / "bad-implementation.json"
            bad_path.write_bytes(base.canonical_json(bad))
            with self.assertRaisesRegex(base.M0Error, "implementation root"):
                trainer.run(bad_path, root / "bad-implementation-output")
            bad = copy.deepcopy(manifest)
            bad["inherited_implementation_hashes"] = {
                **contract.INHERITED_IMPLEMENTATION_HASHES,
                "physical_sound_v25_m0a_train.py": "0" * 64,
            }
            bad_path = root / "bad-inherited.json"
            bad_path.write_bytes(base.canonical_json(bad))
            with self.assertRaisesRegex(base.M0Error, "inherited implementation"):
                trainer.run(bad_path, root / "bad-inherited-output")
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(base.M0Error, "new external path"):
                trainer.run(manifest_path, occupied)
            output = root / "never-published"
            original = trainer._mlflow_log
            try:
                trainer._mlflow_log = lambda *_args, **_kwargs: (_ for _ in ()).throw(
                    base.M0Error("forced M0b MLflow failure")
                )
                with self.assertRaisesRegex(base.M0Error, "forced M0b MLflow failure"):
                    trainer.run(manifest_path, output)
            finally:
                trainer._mlflow_log = original
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v25-m0a-")
                    for path in root.iterdir()
                )
            )


if __name__ == "__main__":
    unittest.main()
