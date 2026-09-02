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
SCRIPT = SCRIPTS / "physical_sound_v32_t0_truth_mutations_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v32-t0-truth-mutations.v1.json"
P0_PROFILE = LAB / "profiles" / "physical-sound-v31-p0-causal-baseline.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v32_t0_truth_mutations_v1 as t0  # noqa: E402


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def decode_float32_wav(data: bytes) -> np.ndarray:
    if data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise AssertionError("not a RIFF/WAVE payload")
    if struct.unpack_from("<H", data, 20)[0] != 3:
        raise AssertionError("not IEEE float WAV")
    if struct.unpack_from("<I", data, 24)[0] != 48_000:
        raise AssertionError("unexpected sample rate")
    if data[36:40] != b"data":
        raise AssertionError("unexpected WAV chunk layout")
    return np.frombuffer(data[44:], dtype="<f4").astype(np.float64)


class TruthMutationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = t0.load_profile(PROFILE)
        cls.p0_profile, _ = t0.p1.p0.load_profile(P0_PROFILE)
        cls.cases = t0.p1.build_case_inputs(cls.p0_profile)
        cls.solutions = {
            case_id: t0.p1.solve_case(case_id, fixture, cls.p0_profile)
            for case_id, fixture, _ in cls.cases
        }
        cls.clean_wavs = {
            case_id: t0.p1.encode_float32_wav(
                solution["samples"], 48_000
            )
            for case_id, solution in cls.solutions.items()
        }
        cls.mutations = t0.build_mutations(
            cls.profile,
            cls.p0_profile,
            cls.solutions,
            cls.clean_wavs,
            t0.sha256_bytes(cls.profile_data),
        )
        cls.mutations_by_id = {
            record["record_id"]: (record, wav) for record, wav in cls.mutations
        }

    def test_full_cli_twice_is_byte_exact_and_complete(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-t0-repeat-") as temporary:
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
            self.assertEqual(len(first), 35)

            report = json.loads(first["report.json"])
            evidence = json.loads(first["evidence.json"])
            release = json.loads(first["truth-release.json"])
            self.assertEqual(report["status"], "Pass")
            self.assertEqual(report["decision"], "T0_TRUTH_MUTATION_LIBRARY_PASS")
            self.assertEqual(report["record_count"], 16)
            self.assertEqual(report["expected_pass_count"], 9)
            self.assertEqual(report["expected_reject_count"], 7)
            self.assertEqual(
                report["next_authorized_stage"],
                "V32-V0-validator-mechanics-protocol",
            )
            self.assertEqual(evidence["access"], t0.ZERO_ACCESS)
            self.assertEqual(len(evidence["operation_checks"]), 16)
            self.assertTrue(
                all(item["status"] == "Pass" for item in evidence["operation_checks"])
            )
            self.assertEqual(
                release["decision_counts"],
                {"OutOfDomain": 0, "Pass": 9, "Reject": 7},
            )
            self.assertEqual(len(release["records"]), 16)
            self.assertEqual(
                evidence["owner_identity"]["sha256"],
                t0.sha256_bytes(SCRIPT.read_bytes()),
            )
            self.assertLess(
                evidence["deterministic_memory_bound_bytes"],
                self.profile["resources"]["max_peak_rss_bytes"],
            )

    def test_all_clean_audio_is_exact_p1_output(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-t0-clean-") as temporary:
            output = Path(temporary) / "release"
            t0.run(PROFILE, output)
            for case_id in t0.EXPECTED_CLEAN_CASES:
                wav = (output / "records" / case_id / "audio.wav").read_bytes()
                record = json.loads(
                    (output / "records" / case_id / "record.json").read_text()
                )
                self.assertEqual(wav, self.clean_wavs[case_id])
                self.assertEqual(record["kind"], "Clean")
                self.assertEqual(record["expected"], {"decision": "Pass", "reason_codes": []})
                self.assertEqual(record["audio"]["sha256"], t0.sha256_bytes(wav))
                self.assertTrue(record["invariant_checks"]["clean_p1_audio_exact"])

    def test_wrong_decay_and_frozen_carrier_are_exact(self) -> None:
        plate = self.solutions["baseline-plate"]
        wrong, wrong_wav = self.mutations_by_id["mutation-wrong-decay"]
        self.assertEqual(
            [mode["decay_per_second"] for mode in wrong["modal"]["modes"]],
            [2.0] * 10,
        )
        expected_wrong = t0.render_modal(
            plate["frequencies"],
            plate["decay"] * 0.25,
            plate["signed_gains"],
            plate["validated"]["contact"]["normal_impulse_ns"],
            48_000,
            144_000,
            0.01,
        ).astype("<f4")
        np.testing.assert_array_equal(decode_float32_wav(wrong_wav), expected_wrong)

        frozen, frozen_wav = self.mutations_by_id["mutation-frozen-carrier"]
        frozen_samples = decode_float32_wav(frozen_wav)
        ordinals = np.arange(144_000, dtype=np.int64)
        prefix = t0.undamped_carrier(
            plate["frequencies"],
            plate["signed_gains"],
            np.arange(256, dtype=np.int64),
            48_000,
        )
        expected_frozen = (
            0.01
            * prefix[np.remainder(ordinals, 256)]
            * np.exp(-8.0 * ordinals.astype(np.float64) / 48_000.0)
        ).astype("<f4")
        np.testing.assert_array_equal(frozen_samples, expected_frozen)
        self.assertEqual(frozen["expected"]["reason_codes"], ["TemporalEvolutionFrozen"])

    def test_shuffled_envelope_and_mode_collapse_are_exact(self) -> None:
        plate = self.solutions["baseline-plate"]
        shuffled, shuffled_wav = self.mutations_by_id["mutation-shuffled-envelope"]
        operation = shuffled["mutation"]["operation"]
        permutation = operation["block_permutation"]
        envelope = np.exp(-8.0 * np.arange(144_000, dtype=np.float64) / 48_000.0)
        shuffled_envelope = np.concatenate(
            [envelope[index * 12_000 : (index + 1) * 12_000] for index in permutation]
        )
        carrier = t0.undamped_carrier(
            plate["frequencies"],
            plate["signed_gains"],
            np.arange(144_000, dtype=np.int64),
            48_000,
        )
        expected = (0.01 * carrier * shuffled_envelope).astype("<f4")
        np.testing.assert_array_equal(decode_float32_wav(shuffled_wav), expected)

        collapsed, collapsed_wav = self.mutations_by_id["mutation-mode-collapse"]
        self.assertEqual(len(collapsed["modal"]["modes"]), 1)
        selected = collapsed["invariant_checks"]["selected_ordinal"]
        self.assertEqual(selected, 1)
        expected_collapsed = t0.render_modal(
            plate["frequencies"][selected : selected + 1],
            plate["decay"][selected : selected + 1],
            plate["signed_gains"][selected : selected + 1],
            1.0,
            48_000,
            144_000,
            0.01,
        ).astype("<f4")
        np.testing.assert_array_equal(
            decode_float32_wav(collapsed_wav), expected_collapsed
        )

    def test_copy_clipping_and_provenance_mutations_are_exact(self) -> None:
        copied, copied_wav = self.mutations_by_id["mutation-spectral-copy"]
        self.assertEqual(copied_wav, self.clean_wavs["baseline-plate"])
        self.assertNotEqual(copied_wav, self.clean_wavs["baseline-beam"])
        self.assertEqual(copied["target_case_id"], "baseline-beam")

        clipped, clipped_wav = self.mutations_by_id["mutation-clipping"]
        clipped_samples = decode_float32_wav(clipped_wav)
        self.assertEqual(float(np.max(np.abs(clipped_samples))), 1.0)
        self.assertGreater(clipped["invariant_checks"]["clipped_sample_count"], 0)
        self.assertEqual(clipped["expected"]["reason_codes"], ["PcmClipping"])

        provenance, provenance_wav = self.mutations_by_id[
            "mutation-provenance-mismatch"
        ]
        self.assertEqual(provenance_wav, self.clean_wavs["baseline-plate"])
        self.assertEqual(
            provenance["lineage"]["declared_parent_audio_sha256"], "0" * 64
        )
        self.assertEqual(
            provenance["lineage"]["actual_parent_audio_sha256"],
            t0.sha256_bytes(provenance_wav),
        )

    def test_expected_decision_and_reason_matrix_is_exact(self) -> None:
        actual = [
            (record["record_id"], record["expected"]["reason_codes"][0])
            for record, _ in self.mutations
        ]
        self.assertEqual(actual, t0.EXPECTED_MUTATIONS)
        self.assertTrue(
            all(record["expected"]["decision"] == "Reject" for record, _ in self.mutations)
        )
        self.assertEqual(
            [item["expected_reason_code"] for item in self.profile["mutations"]],
            self.profile["mutation_reason_codes"],
        )

    def test_profile_and_dependency_corruption_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-t0-profile-") as temporary:
            root = Path(temporary)
            changed = copy.deepcopy(self.profile)
            changed["resources"]["max_record_count"] += 1
            changed_path = root / "changed.json"
            changed_path.write_bytes(t0.canonical_json(changed))
            with self.assertRaisesRegex(t0.TruthMutationError, "profile hash mismatch"):
                t0.load_profile(changed_path)

            duplicate = root / "duplicate.json"
            duplicate.write_bytes(b'{"schema": 1, "schema": 2}\n')
            with self.assertRaisesRegex(t0.TruthMutationError, "duplicate JSON key"):
                t0.load_profile(duplicate)

            nonfinite = root / "nonfinite.json"
            nonfinite.write_bytes(b'{"value": NaN}\n')
            with self.assertRaisesRegex(t0.TruthMutationError, "non-finite JSON"):
                t0.load_profile(nonfinite)

        with mock.patch.object(t0, "P1_OWNER_SHA256", "0" * 64):
            with self.assertRaisesRegex(
                t0.TruthMutationError, "declared dependency mismatch"
            ):
                t0.validate_dependencies(self.profile)

    def test_wav_and_artifact_corruption_is_rejected(self) -> None:
        with self.assertRaisesRegex(t0.TruthMutationError, "empty or non-finite"):
            t0.encode_float32_wav(
                np.asarray([0.0, np.nan]), 48_000, allow_full_scale=False
            )
        with self.assertRaisesRegex(t0.TruthMutationError, "peak contract"):
            t0.encode_float32_wav(
                np.asarray([0.0, 1.0]), 48_000, allow_full_scale=False
            )
        allowed = t0.encode_float32_wav(
            np.asarray([-1.0, 1.0]), 48_000, allow_full_scale=True
        )
        np.testing.assert_array_equal(decode_float32_wav(allowed), [-1.0, 1.0])

        with tempfile.TemporaryDirectory(prefix="nextengine-v32-t0-artifact-") as temporary:
            root = Path(temporary)
            artifact = t0.write_bytes(root, "artifact.bin", b"exact")
            with self.assertRaisesRegex(t0.TruthMutationError, "artifact drift"):
                t0.verify_artifacts(root, [dict(artifact, sha256="0" * 64)])

    def test_output_is_external_fresh_and_failure_is_atomic(self) -> None:
        with self.assertRaisesRegex(t0.TruthMutationError, "fresh external"):
            t0.run(PROFILE, ROOT / "t0-illegal-output")

        with tempfile.TemporaryDirectory(prefix="nextengine-v32-t0-atomic-") as temporary:
            root = Path(temporary)
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(t0.TruthMutationError, "fresh external"):
                t0.run(PROFILE, occupied)

            output = root / "never-published"
            with mock.patch.object(
                t0, "build_into", side_effect=t0.TruthMutationError("forced")
            ):
                with self.assertRaisesRegex(t0.TruthMutationError, "forced"):
                    t0.run(PROFILE, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(path.name.startswith(".nextengine-v32-t0-") for path in root.iterdir())
            )


if __name__ == "__main__":
    unittest.main()
