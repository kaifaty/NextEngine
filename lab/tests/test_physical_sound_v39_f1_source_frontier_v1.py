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
SCRIPT = SCRIPTS / "physical_sound_v39_f1_source_frontier_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v39-f1-source-frontier.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v39_f1_source_frontier_v1 as f1


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def read_json(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise AssertionError(f"expected an object in {path}")
    return value


class SourceFrontierTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile_bytes, raw = f1.read_canonical_profile(PROFILE)
        cls.profile = f1.validate_profile(cls.profile_bytes, raw)

    def test_profile_replays_exact_v30_and_ieteasy_frontiers(self) -> None:
        solved = f1.solve_frontiers(self.profile)
        self.assertEqual(f1.role_b_deficits(solved["baseline_partition"]), (6, 27, 5))
        self.assertEqual(f1.role_b_deficits(solved["current_partition"]), (6, 23, 5))
        self.assertEqual(solved["current_power"], f1.BASELINE_POWER | {
            "non_metal_groups": 77,
            "project_revisions": 11,
        })
        self.assertEqual(solved["observed"], f1.EXPECTED_RESULT)
        self.assertFalse(solved["current_partition"]["feasible"])

    def test_cli_a_b_is_byte_exact_and_zero_access(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f1-repeat-"
        ) as temporary:
            root = Path(temporary)
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
                    "classification.json",
                    "frontier.json",
                    "profile.json",
                    "report.json",
                },
            )
            report = read_json(outputs[0] / "report.json")
            self.assertEqual(report["decision"], "ImprovedFrontier")
            self.assertEqual(report["result"], f1.EXPECTED_RESULT)
            self.assertTrue(all(report["gates"].values()))
            for counter in f1.FORBIDDEN_COUNTERS:
                self.assertEqual(report["access"][counter], 0)
            self.assertFalse(report["authority"]["role_assignment_authority"])
            self.assertFalse(report["authority"]["payload_access_authority"])

    def test_ieteasy_collapses_repetitions_and_preserves_material_power(self) -> None:
        solved = f1.solve_frontiers(self.profile)
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f1-classification-"
        ) as temporary:
            output = Path(temporary) / "run"
            f1.run(PROFILE, output)
            classification = read_json(output / "classification.json")
        self.assertEqual(classification["counts"]["physical_sample_groups"], 15)
        self.assertEqual(classification["counts"]["recording_repetitions_per_sample"], 10)
        self.assertEqual(
            classification["counts"]["recording_rows_not_independent_groups"], 150
        )
        self.assertEqual(classification["counts"]["exact_steel_groups"], 0)
        self.assertEqual(classification["counts"]["non_metal_groups"], 4)
        relations = {
            row["source_label"]: row["eligible_relation"]
            for row in solved["normalized_samples"]
        }
        self.assertEqual(relations["AISI 304 steel"], "ineligible_other_metal")
        self.assertEqual(relations["C45E steel"], "ineligible_other_metal")
        self.assertEqual(relations["Polizene"], "non_metal_candidate")

    def test_material_inflation_and_sample_identity_mutations_reject(self) -> None:
        increment = copy.deepcopy(self.profile["increment"])
        qualified = next(
            row for row in increment["samples"] if row["source_label"] == "AISI 304 steel"
        )
        qualified["power_relation"] = "exact_steel"
        with self.assertRaisesRegex(f1.SourceFrontierError, "material inflation"):
            f1.validate_increment(increment)

        relabelled = copy.deepcopy(self.profile["increment"])
        relabelled["samples"][0]["source_label"] = "Steel"
        with self.assertRaisesRegex(f1.SourceFrontierError, "unknown IETeasy"):
            f1.validate_increment(relabelled)

        duplicate = copy.deepcopy(self.profile["increment"])
        duplicate["samples"][-1] = copy.deepcopy(duplicate["samples"][0])
        with self.assertRaisesRegex(f1.SourceFrontierError, "unique and sorted"):
            f1.validate_increment(duplicate)

        repeated_as_groups = copy.deepcopy(self.profile["increment"])
        repeated_as_groups["recording_repetitions_per_sample"] = 150
        with self.assertRaisesRegex(f1.SourceFrontierError, "repetition count"):
            f1.validate_increment(repeated_as_groups)

        geometry_drift = copy.deepcopy(self.profile["increment"])
        geometry_drift["samples"][0]["mass_g"] += 1
        with self.assertRaisesRegex(f1.SourceFrontierError, "evidence hash drift"):
            f1.validate_increment(geometry_drift)

    def test_duplicate_project_parent_and_baseline_power_drift_reject(self) -> None:
        duplicate = copy.deepcopy(self.profile)
        duplicate["increment"]["project_revision_id"] = duplicate["baseline"][
            "projects"
        ][0]["project_revision_id"]
        with self.assertRaisesRegex(f1.SourceFrontierError, "project parent"):
            f1.validate_increment(duplicate["increment"])

        power_drift = copy.deepcopy(self.profile)
        power_drift["baseline"]["projects"][0]["exact_steel_groups"] += 1
        with self.assertRaisesRegex(f1.SourceFrontierError, "projection drift"):
            f1.validate_baseline(power_drift["baseline"])

        duplicate_baseline = copy.deepcopy(self.profile)
        duplicate_baseline["baseline"]["projects"][1] = copy.deepcopy(
            duplicate_baseline["baseline"]["projects"][0]
        )
        with self.assertRaisesRegex(f1.SourceFrontierError, "unique and sorted"):
            f1.validate_baseline(duplicate_baseline["baseline"])

    def test_lead_disposition_power_and_batch_size_cannot_inflate(self) -> None:
        inflated = copy.deepcopy(self.profile["leads"])
        inflated[0]["declared_exact_steel_groups"] = 6
        with self.assertRaisesRegex(f1.SourceFrontierError, "lead power"):
            f1.validate_leads(inflated)

        admitted = copy.deepcopy(self.profile["leads"])
        admitted[0]["disposition"] = "ADMITTED_METADATA_INCREMENT"
        with self.assertRaisesRegex(f1.SourceFrontierError, "disposition"):
            f1.validate_leads(admitted)

        oversized = copy.deepcopy(self.profile["leads"])
        oversized.append(copy.deepcopy(oversized[-1]))
        with self.assertRaisesRegex(f1.SourceFrontierError, "three-lead"):
            f1.validate_leads(oversized)

    def test_policy_authority_receipt_and_dependency_drift_publish_nothing(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f1-profile-"
        ) as temporary:
            root = Path(temporary)
            mutations = []
            threshold = copy.deepcopy(self.profile)
            threshold["minimums"]["protected_exact_steel_groups_per_role"] = 15
            mutations.append((threshold, "minimums"))
            authority = copy.deepcopy(self.profile)
            authority["authority"]["role_assignment_authority"] = True
            mutations.append((authority, "authority"))
            receipt = copy.deepcopy(self.profile)
            receipt["baseline"]["audit_receipt"]["sha256"] = "0" * 64
            mutations.append((receipt, "audit receipt"))
            dependency = copy.deepcopy(self.profile)
            dependency["dependency_bindings"][0]["sha256"] = "0" * 64
            mutations.append((dependency, "dependency drift"))

            for index, (mutation, message) in enumerate(mutations):
                profile = root / f"profile-{index}.json"
                profile.write_bytes(f1.canonical_json(mutation))
                output = root / f"output-{index}"
                with self.subTest(message=message):
                    with self.assertRaisesRegex(f1.SourceFrontierError, message):
                        f1.run(profile, output)
                    self.assertFalse(output.exists())

    def test_noncanonical_duplicate_json_and_output_guards_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f1-boundary-"
        ) as temporary:
            root = Path(temporary)
            noncanonical = root / "noncanonical.json"
            noncanonical.write_text(json.dumps(self.profile))
            with self.assertRaisesRegex(f1.SourceFrontierError, "canonical JSON"):
                f1.run(noncanonical, root / "noncanonical-output")

            duplicate = root / "duplicate.json"
            duplicate.write_bytes(b'{"schema": "one", "schema": "two"}\n')
            with self.assertRaisesRegex(f1.SourceFrontierError, "duplicate JSON key"):
                f1.run(duplicate, root / "duplicate-output")

            in_repository = ROOT / "target" / "v39-f1-forbidden"
            with self.assertRaisesRegex(f1.SourceFrontierError, "outside"):
                f1.prepare_output(in_repository)

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(f1.SourceFrontierError, "replace existing"):
                f1.run(PROFILE, occupied)

            actual = root / "actual"
            actual.mkdir()
            linked = root / "linked"
            linked.symlink_to(actual, target_is_directory=True)
            with self.assertRaisesRegex(f1.SourceFrontierError, "symlinks"):
                f1.run(PROFILE, linked / "output")

    def test_late_publication_failure_is_atomic(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f1-atomic-"
        ) as temporary:
            root = Path(temporary)
            destination = root / "publication"
            original_write_bytes = Path.write_bytes

            def fail_report(path: Path, data: bytes) -> int:
                if path.name == "report.json":
                    raise OSError("injected publication failure")
                return original_write_bytes(path, data)

            with mock.patch.object(Path, "write_bytes", autospec=True) as writer:
                writer.side_effect = fail_report
                with self.assertRaisesRegex(OSError, "injected publication failure"):
                    f1.run(PROFILE, destination)
            self.assertFalse(destination.exists())
            self.assertFalse(
                any(path.name.startswith(".publication.") for path in root.iterdir())
            )

    def test_owner_source_has_no_network_signal_or_model_dependency(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "import numpy",
            "import requests",
            "import socket",
            "import torch",
            "subprocess",
            "urllib.request",
            "wave.open",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
