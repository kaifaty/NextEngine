#!/usr/bin/env python3
"""Contract-only whole-entry tests for the V24 T0 analytic teacher."""

from __future__ import annotations

import json
import os
import struct
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest import mock

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v24_t0_teacher as teacher


def manifest_value() -> dict[str, object]:
    return {
        "schema": teacher.MANIFEST_SCHEMA,
        "study_id": teacher.STUDY_ID,
        "protocol_revision": teacher.PROTOCOL_REVISION,
        "protocol_sha256": teacher.PROTOCOL_SHA256,
        "profile": "contract-fixture-v1",
        "claim": teacher.CLAIM,
        "data_policy": {
            "external_output_only": True,
            "network_allowed": False,
            "real_signal_allowed": False,
            "model_training_authorized": False,
            "runtime_authorized": False,
        },
    }


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class PhysicalSoundV24T0TeacherTests(unittest.TestCase):
    def test_full_cli_twice_is_exact_and_emits_v3_ready_lane_records(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-t0-test-") as temporary:
            root = Path(temporary)
            manifest = root / "manifest.json"
            manifest.write_bytes(teacher.canonical_json(manifest_value()))
            first = root / "run-a"
            second = root / "run-b"
            environment = os.environ.copy()
            environment.update(
                {
                    "OMP_NUM_THREADS": "1",
                    "OPENBLAS_NUM_THREADS": "1",
                    "MKL_NUM_THREADS": "1",
                    "NUMEXPR_NUM_THREADS": "1",
                }
            )
            command = [
                sys.executable,
                str(SCRIPTS / "physical_sound_v24_t0_teacher.py"),
                "--manifest",
                str(manifest),
                "--output",
            ]
            first_result = subprocess.run(
                [*command, str(first)],
                check=False,
                capture_output=True,
                text=True,
                env=environment,
            )
            self.assertEqual(first_result.returncode, 0, first_result.stderr)
            second_result = subprocess.run(
                [*command, str(second)],
                check=False,
                capture_output=True,
                text=True,
                env=environment,
            )
            self.assertEqual(second_result.returncode, 0, second_result.stderr)
            self.assertEqual(all_files(first), all_files(second))
            self.assertEqual(len(all_files(first)), 38)

            report = json.loads((first / "report.json").read_bytes())
            self.assertEqual(report["status"], "Validated")
            self.assertEqual(report["object_count"], 5)
            self.assertEqual(report["row_count"], 10)
            self.assertEqual(report["role_counts"], {role: 2 for role in teacher.ROLES})
            self.assertFalse(report["model_training_authorized"])
            self.assertFalse(report["real_material_authorized"])
            self.assertFalse(report["runtime_authorized"])

            lane = json.loads((first / "lane-records.json").read_bytes())
            self.assertEqual(lane["status"], "Validated")
            self.assertEqual(len(lane["rows"]), 10)
            self.assertEqual(
                [row["row_id"] for row in lane["rows"]],
                sorted(row["row_id"] for row in lane["rows"]),
            )
            for row in lane["rows"]:
                self.assertEqual(row["evidence_lane"], "synthetic_teacher")
                self.assertEqual(row["audio_semantics"], "synthetic_modal_render")
                self.assertEqual(
                    row["axes"]["teacher_target"]["representation_id"],
                    teacher.TEACHER_REPRESENTATION,
                )
                self.assertEqual(row["axes"]["teacher_target"]["mode_count"], 2)
                self.assertIsNone(row["mutation_parent_id"])

            evidence = json.loads((first / "teacher-evidence.json").read_bytes())
            self.assertLessEqual(evidence["beam_characteristic_maximum_residual"], 1.0e-12)
            self.assertTrue(
                all(item["remesh_common_vertices_exact"] for item in evidence["objects"])
            )
            self.assertTrue(all(item["force_linearity_exact"] for item in evidence["objects"]))

    def test_binary_headers_mesh_binding_and_float_wav_are_exact(self) -> None:
        selected = teacher.profile("contract-fixture-v1")
        recipe = selected.recipes[0]
        frequencies, decay, indices = teacher.modal_solution(recipe, selected.mode_count)
        vertices, triangles, uv = teacher.mesh(recipe, selected.fine_grid)
        mesh_bytes = teacher.encode_mesh(vertices, triangles)
        gains, bounds = teacher.normalized_gains(recipe, indices, frequencies, uv)
        mode_bytes = teacher.encode_modes(frequencies, decay, indices)
        gain_bytes = teacher.encode_gains(teacher.sha256_bytes(mesh_bytes), gains)
        samples = teacher.render(frequencies, decay, gains[0], bounds, selected.duration_seconds)
        wav = teacher.encode_float32_wav(samples)

        self.assertEqual(mesh_bytes[:8], b"NEMESH01")
        self.assertEqual(struct.unpack_from("<III", mesh_bytes, 8), (1, 81, 128))
        self.assertEqual(mode_bytes[:8], b"NEMODT01")
        self.assertEqual(struct.unpack_from("<II", mode_bytes, 8), (1, 2))
        self.assertEqual(gain_bytes[:8], b"NEGAIN01")
        self.assertEqual(struct.unpack_from("<III", gain_bytes, 8), (1, 81, 2))
        self.assertEqual(gain_bytes[20:52], bytes.fromhex(teacher.sha256_bytes(mesh_bytes)))
        teacher.validate_gain_binding(mesh_bytes, gain_bytes)
        corrupt_binding = bytearray(gain_bytes)
        corrupt_binding[20] ^= 1
        with self.assertRaisesRegex(ValueError, "mesh hash mismatch"):
            teacher.validate_gain_binding(mesh_bytes, bytes(corrupt_binding))
        self.assertEqual(wav[:4], b"RIFF")
        self.assertEqual(wav[8:12], b"WAVE")
        self.assertEqual(struct.unpack_from("<H", wav, 20)[0], 3)
        self.assertLess(float(np.max(np.abs(samples))), 0.95)
        with self.assertRaisesRegex(ValueError, "unsorted"):
            teacher.encode_modes(frequencies[::-1], decay[::-1], indices[::-1])
        with self.assertRaisesRegex(ValueError, "unsorted"):
            teacher.encode_modes(frequencies[:0], decay[:0], indices[:0])
        corrupt_gains = gains.copy()
        corrupt_gains[0, 0] = np.nan
        with self.assertRaisesRegex(ValueError, "contact-gain"):
            teacher.encode_gains(teacher.sha256_bytes(mesh_bytes), corrupt_gains)
        corrupt_samples = samples.copy()
        corrupt_samples[0] = np.inf
        with self.assertRaisesRegex(ValueError, "non-finite"):
            teacher.encode_float32_wav(corrupt_samples)

    def test_analytic_scaling_boundaries_roots_and_contact_variation_hold(self) -> None:
        selected = teacher.profile("contract-fixture-v1")
        for recipe in selected.recipes:
            controls = teacher.analytic_controls(recipe, selected.mode_count)
            self.assertLessEqual(controls["maximum_scaling_relative_error"], 1.0e-12)
            self.assertLessEqual(
                controls["maximum_plate_boundary_absolute_error"], 1.0e-12
            )
            self.assertLessEqual(controls["maximum_beam_clamp_absolute_error"], 1.0e-12)
            frequencies, _, indices = teacher.modal_solution(recipe, selected.mode_count)
            contact_uv = np.asarray(
                [[row[2], row[3]] for row in selected.contacts], dtype=np.float64
            )
            gains, _ = teacher.normalized_gains(recipe, indices, frequencies, contact_uv)
            self.assertTrue(bool(np.any(np.ptp(gains, axis=0) > 1.0e-12)))
        self.assertLessEqual(max(teacher.root_residuals(10)), 1.0e-12)

    def test_manifest_output_recipe_and_root_corruptions_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-t0-negative-") as temporary:
            root = Path(temporary)
            manifest = root / "manifest.json"
            value = manifest_value()
            value["protocol_sha256"] = "0" * 64
            manifest.write_bytes(teacher.canonical_json(value))
            with self.assertRaisesRegex(ValueError, "frozen protocol"):
                teacher.run(manifest, root / "output")

            occupied = root / "occupied"
            occupied.mkdir()
            value["protocol_sha256"] = teacher.PROTOCOL_SHA256
            manifest.write_bytes(teacher.canonical_json(value))
            with self.assertRaisesRegex(ValueError, "new external path"):
                teacher.run(manifest, occupied)

            bad_recipe = replace(
                teacher.profile("contract-fixture-v1").recipes[0],
                dimensions=(0.08, 0.06, -0.002),
            )
            with self.assertRaises(ValueError):
                teacher.modal_solution(bad_recipe, 2)

            selected = teacher.profile("contract-fixture-v1")
            material = replace(selected.recipes[0].material, density=2701.0)
            material_recipe = replace(selected.recipes[0], material=material)
            with self.assertRaisesRegex(ValueError, "profile drift"):
                teacher.validate_profile(
                    replace(selected, recipes=(material_recipe, *selected.recipes[1:]))
                )
            with self.assertRaisesRegex(ValueError, "profile drift"):
                teacher.validate_profile(replace(selected, mode_count=3))
            contact = selected.contacts[0]
            changed_contact = (contact[0], "wrong-role", contact[2], contact[3])
            with self.assertRaisesRegex(ValueError, "profile drift"):
                teacher.validate_profile(
                    replace(selected, contacts=(changed_contact, *selected.contacts[1:]))
                )

            roots = ("1.0", *teacher.BEAM_ROOTS[1:])
            with mock.patch.object(teacher, "BEAM_ROOTS", roots):
                with self.assertRaisesRegex(ValueError, "root table"):
                    teacher.run(manifest, root / "root-corrupt-output")
            with mock.patch.object(teacher, "FORMULA_IDS", ("wrong-formula",)):
                with self.assertRaisesRegex(ValueError, "constants drift"):
                    teacher.run(manifest, root / "formula-corrupt-output")
            with mock.patch.dict(teacher.SUPPORT_IDS, {"plate": "free"}, clear=True):
                with self.assertRaisesRegex(ValueError, "constants drift"):
                    teacher.run(manifest, root / "support-corrupt-output")

    def test_remesh_and_artifact_hash_corruptions_fail_closed(self) -> None:
        selected = teacher.profile("contract-fixture-v1")
        recipe = selected.recipes[0]
        frequencies, _, indices = teacher.modal_solution(recipe, selected.mode_count)
        _, _, coarse_uv = teacher.mesh(recipe, selected.coarse_grid)
        _, _, fine_uv = teacher.mesh(recipe, selected.fine_grid)
        coarse_gains, coarse_bounds = teacher.normalized_gains(
            recipe, indices, frequencies, coarse_uv
        )
        fine_gains, fine_bounds = teacher.normalized_gains(
            recipe, indices, frequencies, fine_uv
        )
        corrupt_fine = fine_gains.copy()
        corrupt_fine[0, 0] += 1.0
        with self.assertRaisesRegex(ValueError, "remesh truth drift"):
            teacher.validate_remesh(
                coarse_uv,
                coarse_gains,
                coarse_bounds,
                fine_uv,
                corrupt_fine,
                fine_bounds,
            )

        with tempfile.TemporaryDirectory(prefix="nextengine-v24-t0-hash-") as temporary:
            root = Path(temporary)
            artifact = teacher.write_bytes(root, "artifact.bin", b"exact")
            corrupt = dict(artifact, sha256="0" * 64)
            with self.assertRaisesRegex(ValueError, "artifact hash mismatch"):
                teacher.verify_artifacts(root, [corrupt])

    def test_failed_build_removes_owned_staging_and_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-t0-atomic-") as temporary:
            root = Path(temporary)
            manifest = root / "manifest.json"
            manifest.write_bytes(teacher.canonical_json(manifest_value()))
            output = root / "never-published"
            with mock.patch.object(teacher, "build_into", side_effect=ValueError("forced")):
                with self.assertRaisesRegex(ValueError, "forced"):
                    teacher.run(manifest, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(path.name.startswith(".nextengine-v24-t0-") for path in root.iterdir())
            )


if __name__ == "__main__":
    unittest.main()
