from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v45_r0_source_claim_ledger_v1 as r0  # noqa: E402

PROFILE = ROOT / r0.PROFILE_PATH
SCRIPT = ROOT / r0.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def directory_bytes(path: Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.iterdir())}


def load_profile() -> dict[str, object]:
    return json.loads(PROFILE.read_text())


def write_profile(path: Path, profile: dict[str, object]) -> None:
    path.write_bytes(r0.canonical_json(profile))


class SourceClaimLedgerTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_binds_owner(self) -> None:
        profile_bytes, profile = r0.read_canonical_json(PROFILE, "tracked profile")
        sources = r0.validate_profile(profile)
        owner = next(
            item
            for item in profile["dependency_bindings"]
            if item["path"] == r0.OWNER_PATH
        )
        self.assertEqual(len(SCRIPT.read_bytes()), owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])
        self.assertEqual(profile_bytes, r0.canonical_json(profile))
        self.assertEqual(10, len(sources))

    def test_run_repeats_exactly_with_zero_authority_and_access(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output_a = root / "output-a"
            output_b = root / "output-b"
            r0.run(PROFILE, output_a)
            r0.run(PROFILE, output_b)
            self.assertEqual(directory_bytes(output_a), directory_bytes(output_b))
            report = json.loads((output_a / "report.json").read_text())
            access = json.loads((output_a / "access.json").read_text())
            self.assertEqual(r0.DECISION, report["decision"])
            self.assertTrue(all(report["gates"].values()))
            self.assertTrue(all(value == 0 for value in access["counters"].values()))
            self.assertFalse(report["authority"]["model_training_authority"])
            self.assertFalse(report["authority"]["validator_calibration_authority"])

    def test_lane_counts_preserve_non_interchangeable_evidence(self) -> None:
        profile = load_profile()
        result = profile["expected_result"]
        self.assertEqual(2, result["prospective_lane_counts"]["modal_teacher"])
        self.assertEqual(2, result["prospective_lane_counts"]["structural_transfer"])
        self.assertEqual(2, result["prospective_lane_counts"]["real_acoustic"])
        self.assertEqual(2, result["prospective_lane_counts"]["validator_calibration"])
        self.assertEqual(0, result["prospective_lane_counts"]["protected_admission"])
        self.assertEqual(0, result["physical_parent_credit"])
        self.assertEqual(0, result["project_independence_credit"])

    def test_synthetic_source_cannot_claim_real_or_validator_lane(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = load_profile()
            source = next(
                item for item in profile["sources"] if item["source_id"] == "nisr-v5"
            )
            source["prospective_lanes"] = ["modal_teacher", "real_acoustic"]
            path = root / "profile.json"
            write_profile(path, profile)
            with self.assertRaisesRegex(r0.SourceClaimLedgerError, "exceed"):
                r0.run(path, root / "output")
            self.assertFalse((root / "output").exists())

    def test_parent_credit_mutation_fails_without_publication(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = load_profile()
            profile["sources"][0]["physical_parent_credit"] = 1
            profile["expected_result"]["physical_parent_credit"] = 1
            path = root / "profile.json"
            write_profile(path, profile)
            with self.assertRaisesRegex(r0.SourceClaimLedgerError, "premature"):
                r0.run(path, root / "output")
            self.assertFalse((root / "output").exists())

    def test_dependency_mutation_fails_without_publication(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = load_profile()
            profile["dependency_bindings"][0]["sha256"] = "0" * 64
            path = root / "profile.json"
            write_profile(path, profile)
            with self.assertRaisesRegex(r0.SourceClaimLedgerError, "binding mismatch"):
                r0.run(path, root / "output")
            self.assertFalse((root / "output").exists())

    def test_output_and_import_guards(self) -> None:
        with self.assertRaisesRegex(r0.SourceClaimLedgerError, "outside"):
            r0.prepare_output(ROOT / "forbidden-v45-r0-output")
        source = SCRIPT.read_text()
        for forbidden in (
            "import librosa",
            "import numpy",
            "import requests",
            "import soundfile",
            "import torch",
            "import urllib",
            "import wave",
        ):
            self.assertNotIn(forbidden, source)

    def test_unknown_field_and_unsorted_sources_fail_closed(self) -> None:
        profile = load_profile()
        changed = copy.deepcopy(profile)
        changed["sources"][0]["unexpected"] = True
        with self.assertRaisesRegex(r0.SourceClaimLedgerError, "fields changed"):
            r0.validate_profile(changed)
        changed = copy.deepcopy(profile)
        changed["sources"][0], changed["sources"][1] = (
            changed["sources"][1],
            changed["sources"][0],
        )
        with self.assertRaisesRegex(r0.SourceClaimLedgerError, "source IDs"):
            r0.validate_profile(changed)


if __name__ == "__main__":
    unittest.main()
