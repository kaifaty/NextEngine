"""Value-independent full-entry and mutation tests for V25 M0a."""

from __future__ import annotations

import copy
import dataclasses
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import physical_sound_v24_t0_teacher as teacher
import physical_sound_v24_v3_assemble as assembler
import physical_sound_v25_m0a_common as common
import physical_sound_v25_m0a_evaluate as evaluate
import physical_sound_v25_m0a_model as model_lib
import physical_sound_v25_m0a_train as trainer
import test_physical_sound_v24_t0_teacher as teacher_test


def write(path: Path, data: bytes) -> dict[str, str]:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {"path": str(path), "sha256": common.sha256_bytes(data)}


def executable_ref(name: str) -> dict[str, str]:
    path = Path(subprocess.run(["which", name], check=True, capture_output=True, text=True).stdout.strip()).resolve(
        strict=True
    )
    return {"path": str(path), "sha256": common.sha256_file(path)}


def make_recording(path: Path, frequency: int) -> dict[str, str]:
    result = subprocess.run(
        [
            executable_ref("ffmpeg")["path"],
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            f"sine=frequency={frequency}:sample_rate=44100:duration=0.14",
            "-ac",
            "2",
            "-ar",
            "44100",
            "-c:a",
            "aac",
            "-y",
            str(path),
        ],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode(errors="replace"))
    return {"path": str(path), "sha256": common.sha256_file(path)}


def x0_lane(root: Path, mesh_ref: dict[str, str]) -> Path:
    profile = common.execution_profile("contract-fixture-v1")
    lineage = {
        "schema": common.X0_SCHEMA,
        "status": "Validated",
        "profile": "contract-fixture-v1",
        "protocol_sha256": "fixture",
        "sealed_contact": {
            "row_index": 4,
            "decoded_sample_count": 0,
            "waveform_access": "sealed_not_decompressed",
        },
        "model_training_authorized": False,
        "real_material_admission_authorized": False,
        "runtime_authorized": False,
    }
    lineage_ref = write(root / "x0-lineage-report.json", common.canonical_json(lineage))
    mesh = common.parse_mesh(Path(mesh_ref["path"]).read_bytes(), mesh_ref["sha256"])
    vertex_ids = (0, 8, 72, 80)
    rows = []
    for index, vertex_id in enumerate(vertex_ids):
        time = np.arange(profile.transfer_samples, dtype=np.float64) / 48_000.0
        samples = 0.1 * np.exp(-40.0 * time) * np.sin(2.0 * np.pi * (500.0 + 120.0 * index) * time)
        samples[512] += 1.0 + 0.05 * index
        audio_ref = write(root / f"transfer-row-{index:03d}.f32le", samples.astype("<f4").tobytes())
        rows.append(
            {
                "row_id": f"x0-realimpact-blue-bowl-contact-{index:03d}",
                "split_role": "development",
                "sample_role": "query" if index == 3 else "context",
                "corpus_role": "target",
                "evidence_lane": "exact_real_transfer",
                "audio_semantics": "force_deconvolved_transfer_response",
                "source_group_id": "fixture-x0-transfer",
                "family_group_id": "fixture-blue-bowl-glass",
                "object_group_id": "fixture-x0-blue-bowl",
                "recording_parent_id": f"fixture-transfer-{index}",
                "condition_group_id": f"fixture-condition-{index}",
                "mutation_parent_id": None,
                "lineage_report_ids": ["v24-x0-blue-bowl-real"],
                "audio": audio_ref,
                "audio_provenance": lineage_ref,
                "axes": {
                    "material": {"value_id": "glass", "evidence": lineage_ref},
                    "geometry": {
                        "geometry_id": "fixture-blue-bowl",
                        "feature_artifact": mesh_ref,
                        "evidence": lineage_ref,
                    },
                    "impact": {
                        "coordinate_profile": "fixture-metres-v1",
                        "point_metres": mesh.vertices[vertex_id].tolist(),
                        "evidence": lineage_ref,
                    },
                    "listener": {
                        "coordinate_profile": "fixture-metres-v1",
                        "point_metres": [0.2, 0.0, 0.1],
                        "evidence": lineage_ref,
                    },
                },
            }
        )
    for index, recording_id in enumerate(("000", "020", "039")):
        audio_ref = make_recording(root / f"recording-{recording_id}.mp4", 700 + 90 * index)
        rows.append(
            {
                "row_id": f"x0-objectfolder-blue-bowl-recording-{recording_id}",
                "split_role": "development",
                "sample_role": "query" if recording_id == "039" else "context",
                "corpus_role": "target",
                "evidence_lane": "identified_real_recording",
                "audio_semantics": "recorded_impact_waveform",
                "source_group_id": "fixture-x0-recording",
                "family_group_id": "fixture-blue-bowl-glass",
                "object_group_id": "fixture-x0-blue-bowl",
                "recording_parent_id": f"fixture-recording-{recording_id}",
                "condition_group_id": f"fixture-recording-condition-{recording_id}",
                "mutation_parent_id": None,
                "lineage_report_ids": ["v24-x0-blue-bowl-real"],
                "audio": audio_ref,
                "audio_provenance": lineage_ref,
                "axes": {"material": {"value_id": "glass", "evidence": lineage_ref}},
            }
        )
    lane = {
        "schema": assembler.X0_SCHEMA,
        "status": "Validated",
        "claim": assembler.X0_CLAIM,
        "lineage_report": {
            "id": "v24-x0-blue-bowl-real",
            "expected_schema": common.X0_SCHEMA,
            "expected_claim": assembler.X0_CLAIM,
            "artifact": lineage_ref,
        },
        "rows": sorted(rows, key=lambda row: row["row_id"]),
    }
    path = root / "x0-lane-records.json"
    path.write_bytes(common.canonical_json(lane))
    return path


def fixture(root: Path) -> tuple[Path, dict[str, object]]:
    t0_manifest = root / "t0-manifest.json"
    t0_manifest.write_bytes(teacher.canonical_json(teacher_test.manifest_value()))
    t0_root = root / "t0"
    teacher.run(t0_manifest, t0_root)
    t0_lane = t0_root / "lane-records.json"
    first_t0_row = json.loads(t0_lane.read_bytes())["rows"][0]
    source_mesh = t0_root / first_t0_row["axes"]["geometry"]["feature_artifact"]["path"]
    x0_root = root / "x0"
    x0_root.mkdir()
    mesh_ref = write(x0_root / "mesh.bin", source_mesh.read_bytes())
    x0_path = x0_lane(x0_root, mesh_ref)
    assembly = root / "assembly"
    assembler.run(t0_lane, x0_path, assembly)
    manifest = {
        "schema": common.MANIFEST_SCHEMA,
        "profile": "contract-fixture-v1",
        "protocol_sha256": common.PROTOCOL_SHA256,
        "implementation_root_sha256": common.implementation_root_sha256(),
        "artifacts": {
            "combined_manifest": {
                "path": str(assembly / "combined-manifest.json"),
                "sha256": common.sha256_file(assembly / "combined-manifest.json"),
            },
            "teacher_evidence": {
                "path": str(t0_root / "teacher-evidence.json"),
                "sha256": common.sha256_file(t0_root / "teacher-evidence.json"),
            },
            "x0_lineage": {
                "path": str(x0_root / "x0-lineage-report.json"),
                "sha256": common.sha256_file(x0_root / "x0-lineage-report.json"),
            },
            "ffmpeg": executable_ref("ffmpeg"),
            "ffprobe": executable_ref("ffprobe"),
        },
        "t0_root": str(t0_root),
        "x0_root": str(x0_root),
        "mlflow_root": str(root / "mlflow"),
        "data_policy": {
            "external_output_only": True,
            "cpu_only": True,
            "network_allowed": False,
            "admission_shadow_access_allowed": False,
            "runtime_authorized": False,
            "mlflow_remote_allowed": False,
        },
    }
    manifest_path = root / "m0a-manifest.json"
    manifest_path.write_bytes(common.canonical_json(manifest))
    return manifest_path, manifest


def files(root: Path) -> dict[str, bytes]:
    return {str(path.relative_to(root)): path.read_bytes() for path in sorted(root.rglob("*")) if path.is_file()}


class PhysicalSoundV25M0aTests(unittest.TestCase):
    def test_full_cli_twice_is_byte_exact_and_keeps_protected_roles_sealed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v25-m0a-full-") as temporary:
            root = Path(temporary)
            manifest_path, _ = fixture(root)
            command = [
                sys.executable,
                str(SCRIPTS / "physical_sound_v25_m0a_train.py"),
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
            self.assertEqual(files(root / "run-a"), files(root / "run-b"))
            self.assertEqual(len(files(root / "run-a")), 8)
            report = json.loads((root / "run-a/report.json").read_bytes())
            self.assertEqual(report["decision"], "PASS")
            self.assertTrue(report["internal_candidate_ab_byte_exact"])
            self.assertLessEqual(report["parameter_count"], 40_000)
            self.assertEqual(report["final_step"], 6)
            self.assertFalse(report["admission_shadow_opened"])
            self.assertFalse(report["realimpact_2407_opened"])
            self.assertTrue(all(item["split_role"] != "admission_shadow" for item in report["access_log"]))
            preprocess = json.loads((root / "run-a/preprocess-manifest.json").read_bytes())
            self.assertFalse(preprocess["x0_physical_parameters_known"])
            self.assertFalse(preprocess["x0_support_known"])
            weights = common.decode_canonical_tensors((root / "run-a/candidate-weights.bin").read_bytes())
            predictions = common.decode_canonical_tensors((root / "run-a/candidate-predictions.bin").read_bytes())
            self.assertTrue(weights)
            self.assertTrue(predictions)
            self.assertTrue((root / "mlflow").is_dir())

    def test_model_and_material_coordinates_match_the_frozen_contract(self) -> None:
        environment = model_lib.configure_determinism()
        candidate = model_lib.ContactModalField()
        self.assertEqual(model_lib.validate_model(candidate), 23_142)
        self.assertEqual(environment["device"], "cpu")
        prediction = candidate(torch_zeros((2, 38)), torch_zeros((2, 24)))
        frequency = prediction.frequencies_hz.detach().numpy()
        self.assertTrue(np.all(np.diff(frequency, axis=1) > 0.0))
        self.assertGreaterEqual(float(np.min(frequency)), 20.0)
        self.assertLessEqual(float(np.max(frequency)), 18_000.0)
        base = common.material_support_features("elastic-a", "simply-supported-all-edges")
        physics = common.MATERIAL_PHYSICS["elastic-a"]
        youngs = common.material_support_features(
            "elastic-a",
            "simply-supported-all-edges",
            physical_override=(physics[0] * 4.0, *physics[1:]),
        )
        density = common.material_support_features(
            "elastic-a",
            "simply-supported-all-edges",
            physical_override=(physics[0], physics[1] * 4.0, *physics[2:]),
        )
        self.assertEqual(np.flatnonzero(base != youngs).tolist(), [5])
        self.assertEqual(np.flatnonzero(base != density).tolist(), [6])
        glass = common.material_support_features("glass", None)
        self.assertEqual(glass[:5].tolist(), [0.0, 0.0, 0.0, 1.0, 1.0])
        self.assertTrue(np.array_equal(glass[5:11], np.zeros(6, dtype=np.float32)))
        self.assertTrue(np.array_equal(glass[11:14], np.zeros(3, dtype=np.float32)))

    def test_tensor_container_rejects_order_dtype_finiteness_and_corruption(
        self,
    ) -> None:
        tensors = {
            "a": np.arange(6, dtype="<f4").reshape(2, 3),
            "b": np.asarray([7], dtype="<i8"),
        }
        data = common.canonical_tensor_bytes(tensors)
        decoded = common.decode_canonical_tensors(data)
        self.assertTrue(np.array_equal(decoded["a"], tensors["a"]))
        self.assertTrue(np.array_equal(decoded["b"], tensors["b"]))
        with self.assertRaisesRegex(common.M0Error, "sorted"):
            common.canonical_tensor_bytes({"b": tensors["b"], "a": tensors["a"]})
        with self.assertRaisesRegex(common.M0Error, "dtype"):
            common.canonical_tensor_bytes({"a": np.ones(2, dtype=np.uint8)})
        with self.assertRaisesRegex(common.M0Error, "non-finite"):
            common.canonical_tensor_bytes({"a": np.asarray([np.nan], dtype="<f4")})
        with self.assertRaises(common.M0Error):
            common.decode_canonical_tensors(data + b"x")
        corrupt = bytearray(data)
        corrupt[0] ^= 1
        with self.assertRaisesRegex(common.M0Error, "header"):
            common.decode_canonical_tensors(bytes(corrupt))

    def test_binary_parsers_contact_binding_nuisance_gain_and_denominator_fail_closed(
        self,
    ) -> None:
        selected = teacher.profile("contract-fixture-v1")
        recipe = selected.recipes[0]
        frequency, decay, families = teacher.modal_solution(recipe, selected.mode_count)
        vertices, triangles, uv = teacher.mesh(recipe, selected.fine_grid)
        mesh_bytes = teacher.encode_mesh(vertices, triangles)
        mesh = common.parse_mesh(mesh_bytes)
        modes_bytes = teacher.encode_modes(frequency, decay, families)
        modes = common.parse_modes(modes_bytes, common.execution_profile("contract-fixture-v1"))
        gains, _ = teacher.normalized_gains(recipe, families, frequency, uv)
        gain_bytes = teacher.encode_gains(mesh.sha256, gains)
        field = common.parse_gain_field(gain_bytes, mesh, modes)
        self.assertEqual(field.values.shape, (len(vertices), selected.mode_count))
        with self.assertRaisesRegex(common.M0Error, "header"):
            common.parse_mesh(b"bad")
        corrupt = bytearray(gain_bytes)
        corrupt[20] ^= 1
        with self.assertRaisesRegex(common.M0Error, "binding"):
            common.parse_gain_field(bytes(corrupt), mesh, modes)
        with self.assertRaisesRegex(common.M0Error, "exact mesh vertex"):
            common.nearest_vertex(mesh, [100.0, 100.0, 100.0])
        import torch

        generator = torch.Generator(device="cpu").manual_seed(17)
        predicted = torch.randn(4096, generator=generator) * 0.1
        target = predicted * 3.0
        gain = model_lib.closed_form_log_gain(predicted, target)
        self.assertAlmostEqual(float(gain), math_log(3.0), places=4)
        self.assertEqual(evaluate.safe_ratio(0.0, 0.0), 1.0)
        self.assertTrue(np.isinf(evaluate.safe_ratio(1.0, 0.0)))

    def test_seed_thread_final_step_and_source_seal_mutations_reject(self) -> None:
        with self.assertRaisesRegex(common.M0Error, "seed"):
            model_lib.configure_determinism(model_lib.SEED + 1)
        with (
            mock.patch.dict(os.environ, {"OMP_NUM_THREADS": "2"}),
            self.assertRaisesRegex(common.M0Error, "OMP_NUM_THREADS"),
        ):
            model_lib.configure_determinism()
        profile = common.execution_profile("contract-fixture-v1")
        self.assertEqual(profile.synthetic_steps + profile.real_steps, 6)
        changed = dataclasses.replace(profile, synthetic_steps=5)
        self.assertNotEqual(changed.synthetic_steps + changed.real_steps, 6)
        with tempfile.TemporaryDirectory(prefix="nextengine-v25-m0a-seal-") as temporary:
            root = Path(temporary)
            _, manifest = fixture(root)
            source = Path(manifest["artifacts"]["x0_lineage"]["path"])
            value = json.loads(source.read_bytes())
            value["sealed_contact"]["decoded_sample_count"] = 1
            changed_source = root / "x0-lineage-opened.json"
            changed_ref = write(changed_source, common.canonical_json(value))
            bad = copy.deepcopy(manifest)
            bad["artifacts"]["x0_lineage"] = changed_ref
            bad_path = root / "bad-seal.json"
            bad_path.write_bytes(common.canonical_json(bad))
            with self.assertRaisesRegex(common.M0Error, "seal"):
                trainer.run(bad_path, root / "bad-seal-output")
            bad = copy.deepcopy(manifest)
            bad["artifacts"]["combined_manifest"]["sha256"] = "0" * 64
            bad_path = root / "bad-source-hash.json"
            bad_path.write_bytes(common.canonical_json(bad))
            with self.assertRaisesRegex(common.M0Error, "hash mismatch"):
                trainer.run(bad_path, root / "bad-source-output")
            combined_path = Path(manifest["artifacts"]["combined_manifest"]["path"])
            combined = json.loads(combined_path.read_bytes())
            combined["lineage_reports"][0]["artifact"] = manifest["artifacts"]["x0_lineage"]
            changed_combined = root / "combined-lineage-rebound.json"
            changed_ref = write(changed_combined, common.canonical_json(combined))
            bad = copy.deepcopy(manifest)
            bad["artifacts"]["combined_manifest"] = changed_ref
            bad_path = root / "bad-lineage-binding.json"
            bad_path.write_bytes(common.canonical_json(bad))
            with self.assertRaisesRegex(common.M0Error, "lineage bindings"):
                trainer.run(bad_path, root / "bad-lineage-binding-output")
            bad = copy.deepcopy(manifest)
            bad["mlflow_root"] = str(Path.cwd() / "forbidden-mlflow")
            bad_path = root / "bad-mlflow-root.json"
            bad_path.write_bytes(common.canonical_json(bad))
            with self.assertRaisesRegex(common.M0Error, "external"):
                trainer.run(bad_path, root / "bad-mlflow-output")

    def test_role_order_hashes_output_and_atomic_publish_fail_closed(self) -> None:
        vault = common.EvidenceVault({"rows": []})
        with self.assertRaisesRegex(common.M0Error, "before candidate freeze"):
            vault.synthetic_method_holdout()
        with self.assertRaisesRegex(common.M0Error, "forbidden"):
            vault.admission_shadow()
        with tempfile.TemporaryDirectory(prefix="nextengine-v25-m0a-negative-") as temporary:
            root = Path(temporary)
            manifest_path, manifest = fixture(root)
            bad = copy.deepcopy(manifest)
            bad["protocol_sha256"] = "0" * 64
            bad_path = root / "bad-protocol.json"
            bad_path.write_bytes(common.canonical_json(bad))
            with self.assertRaisesRegex(common.M0Error, "protocol hash"):
                trainer.run(bad_path, root / "bad-output")
            bad = copy.deepcopy(manifest)
            bad["implementation_root_sha256"] = "0" * 64
            bad_path = root / "bad-implementation.json"
            bad_path.write_bytes(common.canonical_json(bad))
            with self.assertRaisesRegex(common.M0Error, "implementation hash"):
                trainer.run(bad_path, root / "bad-implementation-output")
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(common.M0Error, "new external path"):
                trainer.run(manifest_path, occupied)
            output = root / "never-published"
            with (
                mock.patch.object(
                    trainer,
                    "_mlflow_log",
                    side_effect=common.M0Error("forced mlflow failure"),
                ),
                self.assertRaisesRegex(common.M0Error, "forced mlflow failure"),
            ):
                trainer.run(manifest_path, output)
            self.assertFalse(output.exists())
            self.assertFalse(any(path.name.startswith(".nextengine-v25-m0a-") for path in root.iterdir()))


def torch_zeros(shape: tuple[int, ...]):
    import torch

    return torch.zeros(shape, dtype=torch.float32)


def math_log(value: float) -> float:
    import math

    return math.log(value)


if __name__ == "__main__":
    unittest.main()
