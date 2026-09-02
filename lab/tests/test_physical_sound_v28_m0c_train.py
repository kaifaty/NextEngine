"""Conformance tests for the V28 M0c prefix-bounded execution owner."""

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

import physical_sound_v24_t0_teacher as teacher
import physical_sound_v25_m0a_common as base
import physical_sound_v25_m0a_train as inherited_train
import physical_sound_v26_m0b_common as spent_contract
import physical_sound_v28_m0c_common as contract
import physical_sound_v28_m0c_model as model_lib
import physical_sound_v28_m0c_resource as resource_oracle
import physical_sound_v28_m0c_train as trainer
import test_physical_sound_v25_m0a_train as inherited_test
import test_physical_sound_v26_m0b_train as m0b_test


def _replace_reference(
    value: object, old: dict[str, str], new: dict[str, str]
) -> object:
    if isinstance(value, dict):
        if value == old:
            return copy.deepcopy(new)
        return {key: _replace_reference(item, old, new) for key, item in value.items()}
    if isinstance(value, list):
        return [_replace_reference(item, old, new) for item in value]
    return value


def _pad_teacher_fixture(manifest: dict[str, object]) -> None:
    t0_root = Path(manifest["t0_root"])
    combined_path = Path(manifest["artifacts"]["combined_manifest"]["path"])
    combined = json.loads(combined_path.read_bytes())
    teacher_rows = [
        row for row in combined["rows"] if row["evidence_lane"] == "synthetic_teacher"
    ]
    for row in teacher_rows:
        audio_path = Path(row["audio"]["path"])
        sample_rate, samples = base.decode_float_wav(audio_path.read_bytes())
        if sample_rate != model_lib.SAMPLE_RATE_HZ or len(samples) >= 4_096:
            raise AssertionError("M0c fixture teacher audio shape changed")
        padded = np.zeros(4_096, dtype=np.float32)
        padded[: len(samples)] = samples
        audio_bytes = teacher.encode_float32_wav(padded)
        audio_path.write_bytes(audio_bytes)
        row["audio"]["sha256"] = base.sha256_bytes(audio_bytes)

    evidence_path = Path(manifest["artifacts"]["teacher_evidence"]["path"])
    evidence = json.loads(evidence_path.read_bytes())
    artifacts = []
    for item in evidence["objects"]:
        object_id = item["object_id"]
        relatives = [
            f"objects/{object_id}/mesh-coarse.bin",
            f"objects/{object_id}/mesh.bin",
            f"objects/{object_id}/modal-parameters.bin",
            f"objects/{object_id}/contact-gain-field-coarse.bin",
            f"objects/{object_id}/contact-gain-field.bin",
        ]
        relatives.extend(
            sorted(
                Path(row["audio"]["path"])
                .resolve(strict=True)
                .relative_to(t0_root.resolve(strict=True))
                .as_posix()
                for row in teacher_rows
                if row["object_group_id"] == f"v24-t0-object-{object_id}"
            )
        )
        for relative in relatives:
            data = (t0_root / relative).read_bytes()
            artifacts.append(
                {
                    "path": relative,
                    "sha256": base.sha256_bytes(data),
                    "byte_count": len(data),
                }
            )
    if len(artifacts) != evidence["artifact_count"]:
        raise AssertionError("M0c fixture teacher artifact count changed")
    evidence["artifact_root_sha256"] = base.sha256_bytes(base.canonical_json(artifacts))
    old_evidence_ref = {
        "path": str(evidence_path),
        "sha256": manifest["artifacts"]["teacher_evidence"]["sha256"],
    }
    evidence_bytes = base.canonical_json(evidence)
    evidence_path.write_bytes(evidence_bytes)
    new_evidence_ref = {
        "path": str(evidence_path),
        "sha256": base.sha256_bytes(evidence_bytes),
    }
    combined = _replace_reference(combined, old_evidence_ref, new_evidence_ref)
    combined_bytes = base.canonical_json(combined)
    combined_path.write_bytes(combined_bytes)
    manifest["artifacts"]["teacher_evidence"] = new_evidence_ref
    manifest["artifacts"]["combined_manifest"] = {
        "path": str(combined_path),
        "sha256": base.sha256_bytes(combined_bytes),
    }


def fixture(root: Path) -> tuple[Path, dict[str, object]]:
    _, inherited_manifest = m0b_test.fixture(root)
    manifest = copy.deepcopy(inherited_manifest)
    _pad_teacher_fixture(manifest)
    manifest.update(
        {
            "schema": contract.MANIFEST_SCHEMA,
            "protocol_sha256": contract.PROTOCOL_SHA256,
            "implementation_root_sha256": contract.implementation_root_sha256(),
            "inherited_implementation_hashes": contract.INHERITED_IMPLEMENTATION_HASHES,
        }
    )
    path = root / "m0c-manifest.json"
    path.write_bytes(base.canonical_json(manifest))
    return path, manifest


class PhysicalSoundV28M0cTests(unittest.TestCase):
    def test_exact_prefix_equivalence_and_mutations(self) -> None:
        report = resource_oracle.equivalence_report()
        self.assertEqual(report["status"], "Pass")
        self.assertEqual(report["case_count"], 8)
        self.assertEqual(report["full_frames"], 144_000)
        self.assertEqual(report["prefix_frames"], 4_096)
        self.assertTrue(report["late_target_mutation_exact"])
        self.assertTrue(report["early_target_mutation_detected"])
        self.assertTrue(all(report["invalid_rejections"].values()))
        for row in report["cases"]:
            self.assertTrue(row["render_prefix_exact"])
            self.assertTrue(row["loss_exact"])
            self.assertTrue(row["components_exact"])
            self.assertTrue(row["gradients_exact"])
        self.assertFalse(report["official_values_opened"])
        self.assertFalse(report["model_values_published"])

    def test_resource_fixture_has_frozen_official_training_shape(self) -> None:
        synthetic, transfers, recordings, profile = resource_oracle._resource_fixture()
        self.assertEqual(len(synthetic), 32)
        self.assertEqual(len(transfers), 3)
        self.assertEqual(len(recordings), 2)
        self.assertEqual(profile.synthetic_steps, 1_500)
        self.assertEqual(profile.real_steps, 500)
        self.assertEqual(profile.batch_size, 16)
        self.assertEqual(profile.transfer_window, 144_000)
        self.assertEqual(model_lib.TRAINING_RENDER_FRAMES, 4_096)
        self.assertTrue(all(len(item.waveform) == 144_000 for item in synthetic))

    def test_full_cli_twice_is_byte_exact_and_restores_inherited_owner(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v28-m0c-full-"
        ) as temporary:
            root = Path(temporary)
            manifest_path, _ = fixture(root)
            command = [
                sys.executable,
                str(SCRIPTS / "physical_sound_v28_m0c_train.py"),
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
            report = json.loads((root / "run-a/report.json").read_bytes())
            self.assertEqual(report["schema"], trainer.REPORT_SCHEMA)
            self.assertEqual(report["experiment_id"], contract.EXPERIMENT_ID)
            self.assertEqual(report["protocol_sha256"], contract.PROTOCOL_SHA256)
            self.assertEqual(
                report["implementation_root_sha256"],
                contract.implementation_root_sha256(),
            )
            self.assertEqual(report["decision"], "PASS")
            self.assertTrue(report["internal_candidate_ab_byte_exact"])
            self.assertFalse(report["admission_shadow_opened"])
            self.assertFalse(report["realimpact_2407_opened"])

            original_model = inherited_train.model_lib
            direct_report = trainer.run(manifest_path, root / "run-direct")
            self.assertIs(inherited_train.model_lib, original_model)
            self.assertEqual(direct_report["schema"], trainer.REPORT_SCHEMA)

    def test_manifest_cross_schema_atomic_and_hash_mutations_reject(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v28-m0c-negative-"
        ) as temporary:
            root = Path(temporary)
            manifest_path, manifest = fixture(root)
            with self.assertRaisesRegex(base.M0Error, "fields or schema"):
                spent_contract.load_manifest(manifest_path)

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
                "physical_sound_v26_m0b_train.py": "0" * 64,
            }
            bad_path = root / "bad-inherited.json"
            bad_path.write_bytes(base.canonical_json(bad))
            with self.assertRaisesRegex(base.M0Error, "inherited implementation"):
                trainer.run(bad_path, root / "bad-inherited-output")

            output = root / "never-published"
            original = trainer._mlflow_log
            try:
                trainer._mlflow_log = lambda *_args, **_kwargs: (_ for _ in ()).throw(
                    base.M0Error("forced M0c MLflow failure")
                )
                with self.assertRaisesRegex(base.M0Error, "forced M0c MLflow failure"):
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
