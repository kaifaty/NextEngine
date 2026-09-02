from __future__ import annotations

import copy
import json
import os
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v31_p1_modal_owner_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v31-p0-causal-baseline.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v31_p1_modal_owner_v1 as p1  # noqa: E402


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class DeterministicOwnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_text(encoding="utf-8"))

    def test_full_owner_cli_twice_is_byte_exact_and_complete(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v31-p1-repeat-") as temporary:
            root = Path(temporary)
            outputs = [root / "run-a", root / "run-b"]
            environment = os.environ.copy()
            environment.update(
                {
                    "OMP_NUM_THREADS": "1",
                    "OPENBLAS_NUM_THREADS": "1",
                    "MKL_NUM_THREADS": "1",
                    "NUMEXPR_NUM_THREADS": "1",
                }
            )
            stdout: list[bytes] = []
            for output in outputs:
                completed = subprocess.run(
                    [
                        sys.executable,
                        str(SCRIPT),
                        "--profile",
                        str(PROFILE),
                        "--output",
                        str(output),
                    ],
                    cwd=ROOT,
                    env=environment,
                    check=False,
                    capture_output=True,
                )
                self.assertEqual(completed.returncode, 0, completed.stderr.decode())
                self.assertEqual(completed.stderr, b"")
                stdout.append(completed.stdout)

            first = all_files(outputs[0])
            second = all_files(outputs[1])
            self.assertEqual(first, second)
            self.assertEqual(stdout[0], stdout[1])
            self.assertEqual(len(first), 56)

            report = json.loads(first["report.json"])
            evidence = json.loads(first["evidence.json"])
            self.assertEqual(report["status"], "Pass")
            self.assertEqual(report["decision"], "P1_DETERMINISTIC_MODAL_OWNER_PASS")
            self.assertEqual(report["case_count"], 9)
            self.assertEqual(report["fallback_probe_count"], 6)
            self.assertEqual(report["gates"]["isolated_interventions"], "7/7 Pass")
            self.assertEqual(report["gates"]["remesh_pairs"], "9/9 Pass")
            self.assertEqual(report["next_authorized_stage"], "V31-T0-truth-and-mutation-library")
            self.assertEqual(evidence["access"], p1.ZERO_ACCESS)
            self.assertEqual(
                evidence["owner_identity"]["sha256"],
                p1.sha256_bytes(SCRIPT.read_bytes()),
            )
            self.assertLess(
                evidence["deterministic_memory_bound_bytes"],
                self.profile["resources"]["max_peak_rss_bytes"],
            )
            self.assertLess(
                report["output_bytes_before_report"],
                self.profile["resources"]["max_output_bytes"],
            )

    def test_modal_wav_and_mesh_payloads_match_frozen_contract(self) -> None:
        plate = copy.deepcopy(self.profile["fixtures"][0])
        solved = p1.solve_case("plate", plate, self.profile)
        modes = solved["modal_document"]["modes"]
        self.assertEqual(len(modes), 10)
        self.assertEqual(
            [mode["frequency_hz"] for mode in modes],
            sorted(mode["frequency_hz"] for mode in modes),
        )
        self.assertEqual(
            list(modes[0]),
            sorted(self.profile["modal_output"]["record_fields"]),
        )

        mesh = solved["fine_mesh"]
        self.assertEqual(mesh[:8], p1.MESH_MAGIC)
        self.assertEqual(struct.unpack_from("<III", mesh, 8), (1, 825, 1536))
        gains = p1.encode_gains(solved["fine_field"])
        self.assertEqual(gains[:8], p1.GAIN_MAGIC)
        self.assertEqual(struct.unpack_from("<III", gains, 8), (1, 825, 10))

        wav = p1.encode_float32_wav(
            solved["samples"], self.profile["numeric_profile"]["sample_rate_hz"]
        )
        self.assertEqual(wav[:4], b"RIFF")
        self.assertEqual(wav[8:12], b"WAVE")
        self.assertEqual(struct.unpack_from("<H", wav, 20)[0], 3)
        self.assertEqual(struct.unpack_from("<I", wav, 24)[0], 48_000)
        self.assertEqual((len(wav) - 44) // 4, 144_000)
        self.assertLess(float(np.max(np.abs(solved["samples"]))), 1.0)

    def test_every_causal_intervention_and_analytic_control_passes(self) -> None:
        cases = p1.build_case_inputs(self.profile)
        solutions = {
            case_id: p1.solve_case(case_id, fixture, self.profile)
            for case_id, fixture, _ in cases
        }
        controls = p1.intervention_controls(self.profile, solutions)
        self.assertEqual(len(controls), 7)
        self.assertTrue(all(control["status"] == "Pass" for control in controls))
        self.assertLessEqual(
            max(control["maximum_frequency_ratio_absolute_error"] for control in controls),
            p1.RELATIVE_TOLERANCE,
        )
        by_axis = {control["axis"]: control for control in controls}
        self.assertEqual(by_axis["impulse"]["gain_result"], "linear-ratio:2-exact")
        self.assertEqual(by_axis["contact"]["gain_result"], "target-mode-zero-exact")
        self.assertEqual(by_axis["support"]["compared_modes"], 1)

        analytic = p1.analytic_controls(self.profile, solutions)
        self.assertEqual(analytic["status"], "Pass")
        self.assertEqual(analytic["maximum_plate_boundary_displacement_error"], 0.0)
        self.assertEqual(analytic["maximum_beam_clamp_displacement_error"], 0.0)
        self.assertLessEqual(
            analytic["maximum_beam_characteristic_residual"],
            p1.RELATIVE_TOLERANCE,
        )
        for solved in solutions.values():
            self.assertTrue(solved["metrics"]["remesh_common_vertices_exact"])
            self.assertLess(
                solved["metrics"]["sample_peak"],
                solved["metrics"]["analytic_peak_bound"] + p1.RELATIVE_TOLERANCE,
            )
            self.assertLessEqual(
                solved["metrics"]["final_modal_envelope_energy"],
                solved["metrics"]["initial_modal_envelope_energy"],
            )

    def test_plate_and_cantilever_frequencies_match_inherited_v24_teacher(self) -> None:
        material = p1.v24.Material(
            "synthetic-elastic-reference",
            2700.0,
            69_000_000_000.0,
            0.33,
            8.0,
            0.0,
        )
        plate_recipe = p1.v24.Recipe(
            "plate", "train", "plate", material, (0.24, 0.18, 0.003)
        )
        beam_recipe = p1.v24.Recipe(
            "beam", "train", "beam", material, (0.30, 0.04, 0.004)
        )
        plate_expected, plate_indices = p1.v24.plate_modes(plate_recipe, 10)
        beam_expected, beam_indices = p1.v24.beam_modes(beam_recipe, 10)
        plate_actual = p1.solve_case(
            "plate", copy.deepcopy(self.profile["fixtures"][0]), self.profile
        )
        beam_actual = p1.solve_case(
            "beam", copy.deepcopy(self.profile["fixtures"][1]), self.profile
        )
        np.testing.assert_array_equal(plate_actual["frequencies"], plate_expected)
        np.testing.assert_array_equal(plate_actual["indices"], plate_indices)
        np.testing.assert_array_equal(beam_actual["frequencies"], beam_expected)
        np.testing.assert_array_equal(beam_actual["indices"], beam_indices)

    def test_all_declared_out_of_domain_reasons_select_authored_fallback(self) -> None:
        probes = p1.fallback_probes(self.profile)
        self.assertEqual(
            {probe["reason_code"] for probe in probes},
            set(self.profile["fallback"]["reason_codes"]),
        )
        for probe in probes:
            self.assertEqual(probe["status"], "FallbackOutOfDomain")
            self.assertEqual(
                probe["authored_clip_class"], "engine.authored-impact-clip"
            )
            self.assertFalse(probe["partial_output_published"])

    def test_profile_dependency_and_numeric_corruptions_fail_closed(self) -> None:
        with mock.patch.object(p1, "P0_OWNER_SHA256", "0" * 64):
            with self.assertRaisesRegex(p1.ModalOwnerError, "dependency drift"):
                p1.validate_dependencies()

        plate = copy.deepcopy(self.profile["fixtures"][0])
        plate["material"]["density_kg_m3"] = "Infinity"
        result = p1.attempt_case("bad-number", plate, self.profile)
        self.assertEqual(result["reason_code"], "InvalidNumericInput")

        with tempfile.TemporaryDirectory(prefix="nextengine-v31-p1-profile-") as temporary:
            root = Path(temporary)
            changed = copy.deepcopy(self.profile)
            changed["resources"]["max_modes"] += 1
            profile = root / "profile.json"
            profile.write_bytes(p1.canonical_json(changed))
            output = root / "output"
            with self.assertRaisesRegex(p1.ModalOwnerError, "P0 contract rejected"):
                p1.run(profile, output)
            self.assertFalse(output.exists())

    def test_payload_and_artifact_corruption_is_rejected(self) -> None:
        with self.assertRaisesRegex(p1.ModalOwnerError, "non-finite"):
            p1.encode_float32_wav(np.asarray([0.0, np.nan]), 48_000)
        with self.assertRaisesRegex(p1.ModalOwnerError, "clipped"):
            p1.encode_float32_wav(np.asarray([0.0, 1.0]), 48_000)
        with self.assertRaisesRegex(p1.ModalOwnerError, "gain payload"):
            p1.encode_gains(np.asarray([[np.inf]]))

        with tempfile.TemporaryDirectory(prefix="nextengine-v31-p1-artifact-") as temporary:
            root = Path(temporary)
            artifact = p1.write_bytes(root, "artifact.bin", b"exact")
            corrupt = dict(artifact, sha256="0" * 64)
            with self.assertRaisesRegex(p1.ModalOwnerError, "artifact drift"):
                p1.verify_artifacts(root, [corrupt])

    def test_output_is_external_fresh_and_failed_build_publishes_nothing(self) -> None:
        with self.assertRaisesRegex(p1.ModalOwnerError, "fresh external"):
            p1.run(PROFILE, ROOT / "p1-illegal-output")

        with tempfile.TemporaryDirectory(prefix="nextengine-v31-p1-atomic-") as temporary:
            root = Path(temporary)
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(p1.ModalOwnerError, "fresh external"):
                p1.run(PROFILE, occupied)

            output = root / "never-published"
            with mock.patch.object(
                p1, "build_into", side_effect=p1.ModalOwnerError("forced")
            ):
                with self.assertRaisesRegex(p1.ModalOwnerError, "forced"):
                    p1.run(PROFILE, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(path.name.startswith(".nextengine-v31-p1-") for path in root.iterdir())
            )


if __name__ == "__main__":
    unittest.main()
