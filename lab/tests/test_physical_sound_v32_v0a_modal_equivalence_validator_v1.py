from __future__ import annotations

import copy
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v32_v0a_modal_equivalence_validator_v1.py"
PROFILE = (
    LAB / "profiles" / "physical-sound-v32-v0a-modal-equivalence-validator.v1.json"
)
T0_PROFILE = LAB / "profiles" / "physical-sound-v32-t0-truth-mutations.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v32_t0_truth_mutations_v1 as t0  # noqa: E402
import physical_sound_v32_v0_validator_mechanics_v1 as v0  # noqa: E402
import physical_sound_v32_v0a_modal_equivalence_validator_v1 as v0a  # noqa: E402


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class ModalEquivalenceValidatorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(prefix="nextengine-v32-v0a-tests-")
        cls.root = Path(cls.temporary.name)
        cls.truth = cls.root / "truth"
        t0.run(T0_PROFILE, cls.truth)
        cls.profile, cls.profile_data = v0a.load_profile(PROFILE)
        cls.records, cls.audio, cls.truth_identity = v0.load_truth(
            cls.truth, cls.profile
        )
        cls.clean_ids = cls.profile["truth"]["clean_case_ids"]
        cls.clean_records = {
            record_id: cls.records[record_id] for record_id in cls.clean_ids
        }
        cls.clean_audio = {
            record_id: cls.audio[record_id] for record_id in cls.clean_ids
        }
        cls.clean_record_sha256 = {
            record_id: cls.truth_identity["record_sha256"][record_id]
            for record_id in cls.clean_ids
        }

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def validate(
        self, record: dict[str, object], audio: bytes
    ) -> tuple[dict[str, object] | None, dict[str, object]]:
        return v0a.validate_candidate(
            self.profile,
            record,
            audio,
            self.clean_records,
            self.clean_audio,
            self.clean_record_sha256,
        )

    def test_full_cli_twice_is_byte_exact_complete_and_hash_closed(self) -> None:
        outputs = [self.root / "validation-a", self.root / "validation-b"]
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
                    "--truth",
                    str(self.truth),
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
        release = json.loads(first["validator-mechanics-release.json"])
        self.assertEqual(
            report["decision"],
            "V0A_MODAL_EQUIVALENCE_VALIDATOR_MECHANICS_PASS",
        )
        self.assertEqual(
            release["decision_counts"],
            {"OutOfDomain": 0, "Pass": 9, "Reject": 7},
        )
        self.assertEqual(
            release["reason_counts"],
            {
                "EnvelopeOrderMismatch": 1,
                "ModalCoverageCollapsed": 1,
                "PcmClipping": 1,
                "PhysicalDecayMismatch": 1,
                "ProvenanceMismatch": 1,
                "RetrievalCopyDetected": 1,
                "TemporalEvolutionFrozen": 1,
            },
        )
        self.assertEqual(evidence["access"], v0a.ZERO_ACCESS)
        self.assertEqual(evidence["cache"]["reverse_order_warm_hits"], 16)
        self.assertEqual(evidence["owner_identity"]["path"], v0a.OWNER_PATH)
        self.assertEqual(
            evidence["owner_identity"]["sha256"],
            v0a.sha256_bytes(SCRIPT.read_bytes()),
        )
        self.assertEqual(
            evidence["dependencies"]["v0_core"]["sha256"],
            v0a.V0_CORE_SHA256,
        )
        self.assertLess(
            evidence["deterministic_memory_bound_bytes"],
            self.profile["resources"]["max_peak_rss_bytes"],
        )

    def test_clean_aliases_pass_but_plate_to_beam_copy_rejects(self) -> None:
        for record_id in (
            "intervention-density",
            "intervention-uniform-scale",
            "intervention-thickness",
            "intervention-youngs-modulus",
        ):
            features, decision = self.validate(
                self.records[record_id], self.audio[record_id]
            )
            self.assertIsNotNone(features)
            self.assertEqual(decision["decision"], "Pass")
            self.assertEqual(decision["reason_codes"], [])
            self.assertEqual(features["retrieval"]["copied_clean_target_ids"], [])

        record_id = "mutation-spectral-copy"
        features, decision = self.validate(
            self.records[record_id], self.audio[record_id]
        )
        self.assertEqual(decision["decision"], "Reject")
        self.assertEqual(decision["reason_codes"], ["RetrievalCopyDetected"])
        self.assertEqual(
            features["retrieval"]["copied_clean_target_ids"], ["baseline-plate"]
        )

    def test_ignored_labels_and_old_core_identity_are_invariant(self) -> None:
        record = self.records["baseline-plate"]
        forged = copy.deepcopy(record)
        forged.update(
            {
                "expected": {"decision": "Forged"},
                "invariant_checks": {"forged": False},
                "kind": "Forged",
                "mutation": {"operation": "forged"},
                "source_case_id": "forged",
            }
        )
        first = self.validate(record, self.audio["baseline-plate"])
        second = self.validate(forged, self.audio["baseline-plate"])
        self.assertEqual(v0a.canonical_json(first), v0a.canonical_json(second))
        self.assertEqual(
            v0.PROFILE_SHA256,
            "8865cf8ca52f76816ad538c1ebedd44ba8ee68a6a96e1f1980a50cd35ebe4709",
        )
        self.assertEqual(v0.PROFILE_ID, "physical-sound-v32-v0-validator-mechanics-v1")

    def test_cache_domain_is_successor_owned(self) -> None:
        record = self.records["baseline-plate"]
        view_sha256 = v0.sha256_bytes(
            v0.canonical_json(v0.candidate_view(record, self.profile))
        )
        audio_sha256 = v0.sha256_bytes(self.audio["baseline-plate"])
        target_sha256 = self.clean_record_sha256["baseline-plate"]
        successor = v0a.feature_cache_key(
            self.profile, view_sha256, audio_sha256, target_sha256
        )
        v0_profile, _ = v0.load_profile(
            LAB / "profiles" / "physical-sound-v32-v0-validator-mechanics.v1.json"
        )
        rejected = v0.feature_cache_key(
            v0_profile, view_sha256, audio_sha256, target_sha256
        )
        self.assertNotEqual(successor, rejected)

    def test_ood_modal_parameter_and_integrity_probes(self) -> None:
        plate = self.records["baseline-plate"]
        ood = copy.deepcopy(plate)
        ood["record_id"] = "ood"
        ood["target_case_id"] = "unknown-target"
        features, decision = self.validate(ood, self.audio["baseline-plate"])
        self.assertIsNone(features)
        self.assertEqual(decision["decision"], "OutOfDomain")
        self.assertEqual(decision["reason_codes"], ["UnsupportedTargetCase"])

        modal = copy.deepcopy(plate)
        modal["record_id"] = "modal-parameter-probe"
        modal["modal"]["modes"][0]["frequency_hz"] += 1.0
        _, decision = self.validate(modal, self.audio["baseline-plate"])
        self.assertEqual(decision["reason_codes"], ["ModalParameterMismatch"])

        identity = copy.deepcopy(plate)
        identity["record_id"] = "integrity-probe"
        identity["audio"]["sha256"] = "0" * 64
        _, decision = self.validate(identity, self.audio["baseline-plate"])
        self.assertEqual(decision["reason_codes"], ["IntegrityMismatch"])

        malformed = copy.deepcopy(plate)
        malformed["record_id"] = "malformed-wav-probe"
        features, decision = self.validate(malformed, b"not-a-wave")
        self.assertIn("integrity_error", features)
        self.assertEqual(decision["reason_codes"], ["IntegrityMismatch"])

    def test_reason_precedence_remains_provenance_before_clipping(self) -> None:
        record = copy.deepcopy(self.records["mutation-clipping"])
        record["record_id"] = "precedence-probe"
        record["lineage"] = {
            "actual_parent_audio_sha256": v0.sha256_bytes(
                self.audio["baseline-plate"]
            ),
            "declared_parent_audio_sha256": "0" * 64,
        }
        _, decision = self.validate(record, self.audio["mutation-clipping"])
        self.assertEqual(decision["reason_codes"], ["ProvenanceMismatch"])

    def test_profile_dependency_truth_and_atomic_failures_reject(self) -> None:
        bad_profile = self.root / "bad-profile.json"
        profile = copy.deepcopy(self.profile)
        profile["revision"] = 2
        bad_profile.write_bytes(v0a.canonical_json(profile))
        output = self.root / "must-not-publish"
        with self.assertRaisesRegex(v0a.ValidatorMechanicsError, "profile hash mismatch"):
            v0a.run(bad_profile, self.truth, output)
        self.assertFalse(output.exists())
        self.assertFalse(
            any(path.name.startswith(".nextengine-v32-v0a-") for path in self.root.iterdir())
        )

        bad_dependencies = copy.deepcopy(self.profile)
        bad_dependencies["parent"]["v0_core"]["sha256"] = "0" * 64
        with self.assertRaisesRegex(
            v0a.ValidatorMechanicsError, "declared dependency mismatch"
        ):
            v0a.validate_dependencies(bad_dependencies)

        corrupt_truth = self.root / "corrupt-truth"
        shutil.copytree(self.truth, corrupt_truth)
        report = corrupt_truth / "report.json"
        report.write_bytes(report.read_bytes() + b" ")
        with self.assertRaisesRegex(v0a.ValidatorMechanicsError, "T0 report drift"):
            v0a.run(PROFILE, corrupt_truth, self.root / "corrupt-output")
        self.assertFalse((self.root / "corrupt-output").exists())

    def test_source_does_not_import_truth_or_generator_owners(self) -> None:
        source = SCRIPT.read_text()
        self.assertNotIn("import physical_sound_v32_t0", source)
        self.assertNotIn("import physical_sound_v31_p1", source)
        self.assertNotIn("import physical_sound_v31_p0", source)


if __name__ == "__main__":
    unittest.main()
