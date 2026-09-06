from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v40_d0_disclosed_roster_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v40-d0-disclosed-roster.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v40_d0_disclosed_roster_v1 as d0


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def read_json(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise AssertionError(f"expected object in {path}")
    return value


class DisclosedRosterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile_bytes, raw = d0.read_canonical_profile(PROFILE)
        cls.profile, cls.dependencies = d0.validate_profile(raw)
        cls.prerequisites = d0.load_prerequisites(cls.dependencies)

    def test_exact_permanent_disclosed_role_freeze(self) -> None:
        frozen = d0.freeze_roster(self.prerequisites)
        self.assertEqual(frozen["result"], d0.EXPECTED_RESULT)
        self.assertEqual(
            frozen["role_counts"],
            {
                "generator_development": 2,
                "generator_train": 5,
                "validator_calibration": 2,
            },
        )
        self.assertEqual(len(frozen["rows"]), 9)
        self.assertTrue(all(row["permanent_disclosed"] for row in frozen["rows"]))
        self.assertTrue(all(not row["protected"] for row in frozen["rows"]))
        self.assertTrue(
            all(row["current_run_signal_access"] == "none" for row in frozen["rows"])
        )
        self.assertEqual(
            [row["family_id"] for row in frozen["rows"]],
            sorted(row["family_id"] for row in frozen["rows"]),
        )

    def test_historical_partition_rule_is_preserved(self) -> None:
        frozen = d0.freeze_roster(self.prerequisites)
        role_map = d0.ASSIGNMENT_POLICY["historical_partition_role_map"]
        for row in frozen["rows"]:
            self.assertEqual(row["role"], role_map[row["historical_partition"]])
        role_roots = [row["role_root_sha256"] for row in frozen["role_commitments"]]
        self.assertEqual(len(role_roots), len(set(role_roots)))
        self.assertEqual(
            [row["role"] for row in frozen["role_commitments"]],
            list(d0.DISCLOSED_ROLES),
        )

    def test_i0_clean_pool_remains_unspent_and_alias_fails_closed(self) -> None:
        frozen = d0.freeze_roster(self.prerequisites)
        clean = {
            row["family_id"]
            for row in self.prerequisites["i0_solved"]["classifications"]
            if row["protected_eligible"]
        }
        disclosed = {row["family_id"] for row in frozen["rows"]}
        self.assertFalse(clean & disclosed)
        self.assertEqual(frozen["result"]["clean_project_families_spent"], 0)

        mutation = copy.deepcopy(self.prerequisites)
        mutation["i0_solved"]["classifications"].append(
            {
                "family_id": next(iter(disclosed)),
                "protected_eligible": True,
            }
        )
        with self.assertRaisesRegex(
            d0.DisclosedRosterError, "consumes a clean project family"
        ):
            d0.freeze_roster(mutation)

    def test_cli_a_b_is_byte_exact_atomic_and_zero_access(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v40-d0-repeat-") as temp:
            root = Path(temp)
            outputs = [root / "a", root / "b"]
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
                    check=False,
                    capture_output=True,
                )
                self.assertEqual(completed.returncode, 0, completed.stderr.decode())
                self.assertEqual(completed.stderr, b"")

            self.assertEqual(artifact_tree(outputs[0]), artifact_tree(outputs[1]))
            self.assertEqual(
                set(artifact_tree(outputs[0])),
                {
                    "access-ledger.json",
                    "disclosed-roster.json",
                    "profile.json",
                    "report.json",
                },
            )
            report = read_json(outputs[0] / "report.json")
            self.assertEqual(report["decision"], d0.DECISION)
            self.assertTrue(all(report["gates"].values()))
            for counter in d0.FORBIDDEN_COUNTERS:
                self.assertEqual(report["access"][counter], 0)
            self.assertFalse(report["authority"]["model_training_authority"])
            self.assertFalse(report["authority"]["payload_access_authority"])
            self.assertFalse(
                report["authority"]["protected_role_assignment_authority"]
            )

            ledger = read_json(outputs[0] / "access-ledger.json")
            self.assertEqual(len(ledger["entries"]), 9)
            for entry in ledger["entries"]:
                self.assertFalse(entry["protected"])
                self.assertTrue(all(value == 0 for value in entry["counters"].values()))

    def test_assignment_policy_authority_and_dependency_mutations_reject(self) -> None:
        mutations = []
        assignment = copy.deepcopy(self.profile)
        assignment["assignments"][0]["role"] = "generator_train"
        mutations.append((assignment, "historical partition role map"))
        count = copy.deepcopy(self.profile)
        count["assignment_policy"]["expected_role_counts"]["generator_train"] = 4
        mutations.append((count, "assignment_policy"))
        authority = copy.deepcopy(self.profile)
        authority["authority"]["model_training_authority"] = True
        mutations.append((authority, "authority"))
        dependency = copy.deepcopy(self.profile)
        dependency["dependency_bindings"][0]["sha256"] = "0" * 64
        mutations.append((dependency, "dependency drift"))

        with tempfile.TemporaryDirectory(prefix="nextengine-v40-d0-mutation-") as temp:
            root = Path(temp)
            for index, (mutation, expected) in enumerate(mutations):
                profile = root / f"profile-{index}.json"
                profile.write_bytes(d0.canonical_json(mutation))
                output = root / f"output-{index}"
                with self.subTest(expected=expected):
                    with self.assertRaisesRegex(d0.DisclosedRosterError, expected):
                        d0.run(profile, output)
                    self.assertFalse(output.exists())

    def test_noncanonical_duplicate_json_and_output_guards_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v40-d0-boundary-") as temp:
            root = Path(temp)
            noncanonical = root / "noncanonical.json"
            noncanonical.write_text(json.dumps(self.profile))
            with self.assertRaisesRegex(d0.DisclosedRosterError, "canonical JSON"):
                d0.run(noncanonical, root / "noncanonical-output")

            duplicate = root / "duplicate.json"
            duplicate.write_bytes(b'{"schema": "one", "schema": "two"}\n')
            with self.assertRaisesRegex(d0.DisclosedRosterError, "duplicate JSON key"):
                d0.run(duplicate, root / "duplicate-output")

            inside_repository = ROOT / "target" / "v40-d0-forbidden"
            with self.assertRaisesRegex(d0.DisclosedRosterError, "outside"):
                d0.prepare_output(inside_repository)

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(d0.DisclosedRosterError, "refusing to replace"):
                d0.prepare_output(occupied)

            linked = root / "linked"
            linked.symlink_to(root / "actual", target_is_directory=True)
            with self.assertRaisesRegex(d0.DisclosedRosterError, "symlink"):
                d0.prepare_output(linked / "result")

    def test_late_publication_failure_leaves_no_partial_result(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v40-d0-atomic-") as temp:
            root = Path(temp)
            output = root / "result"
            writes = 0
            original = Path.write_bytes

            def fail_third_write(path: Path, data: bytes) -> int:
                nonlocal writes
                writes += 1
                if writes == 3:
                    raise OSError("injected late publication failure")
                return original(path, data)

            with mock.patch.object(Path, "write_bytes", fail_third_write):
                with self.assertRaisesRegex(OSError, "injected"):
                    d0.run(PROFILE, output)
            self.assertFalse(output.exists())
            self.assertEqual(list(root.iterdir()), [])

    def test_owner_has_no_network_signal_or_model_dependency(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "import requests",
            "import torch",
            "import torchaudio",
            "import librosa",
            "import soundfile",
            "import scipy",
            "import socket",
            "import urllib",
            "import wave",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
