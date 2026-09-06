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
SCRIPT = SCRIPTS / "physical_sound_v40_i0_project_exposure_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v40-i0-project-exposure.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v40_i0_project_exposure_v1 as i0


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


class ProjectExposureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile_bytes, raw = i0.read_canonical_profile(PROFILE)
        cls.profile, cls.dependencies = i0.validate_profile(raw)
        _, _, cls.source_solved = i0.load_source_frontier(cls.dependencies)

    def test_exact_audit_quarantines_opened_projects_and_recomputes_power(self) -> None:
        solved = i0.solve_audit(self.source_solved)
        self.assertEqual(solved["observed"], i0.EXPECTED_RESULT)
        self.assertEqual(
            {
                row["project_revision_id"]
                for row in solved["quarantined_projects"]
            },
            {i0.OBJECTFOLDER_PROJECT, i0.YCB_PROJECT},
        )
        self.assertEqual(
            solved["clean_power"],
            {
                "exact_steel_groups": 9,
                "non_metal_groups": 7,
                "project_revisions": 9,
            },
        )
        self.assertEqual(
            solved["quarantined_power"],
            {
                "exact_steel_groups": 23,
                "non_metal_groups": 70,
                "project_revisions": 2,
            },
        )

    def test_clean_partition_preserves_five_reserves_and_exact_deficits(self) -> None:
        solved = i0.solve_audit(self.source_solved)
        partition = solved["clean_partition"]
        self.assertFalse(partition["feasible"])
        frontier = partition["best_frontier"]
        self.assertEqual(frontier["reserved_unprotected_project_count"], 5)
        self.assertEqual(
            (
                frontier["role_a"]["exact_steel_deficit"],
                frontier["role_a"]["non_metal_deficit"],
            ),
            (13, 34),
        )
        self.assertEqual(
            (
                frontier["role_b"]["exact_steel_deficit"],
                frontier["role_b"]["non_metal_deficit"],
            ),
            (13, 31),
        )
        self.assertEqual(
            solved["clean_projects_sha256"], i0.EXPECTED_CLEAN_PROJECTS_SHA256
        )
        self.assertEqual(
            solved["clean_partition_sha256"],
            i0.EXPECTED_CLEAN_PARTITION_SHA256,
        )

    def test_known_disclosed_family_roster_is_complete_and_protected_false(self) -> None:
        families = self.profile["family_policies"]
        self.assertEqual(len(families), 9)
        self.assertEqual(
            [row["family_id"] for row in families],
            sorted(row["family_id"] for row in families),
        )
        self.assertTrue(
            all(row["disposition"] == "permanent_disclosed" for row in families)
        )
        self.assertTrue(
            all(row["protected_power_allowed"] is False for row in families)
        )
        self.assertIn(
            "samuel-clarke--realimpact", {row["family_id"] for row in families}
        )
        self.assertIn("zisen-shao--av-msf", {row["family_id"] for row in families})

    def test_cli_a_b_is_byte_exact_atomic_and_zero_access(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v40-i0-repeat-") as temp:
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
                    "clean-frontier.json",
                    "profile.json",
                    "project-ledger.json",
                    "report.json",
                },
            )
            report = read_json(outputs[0] / "report.json")
            self.assertEqual(report["decision"], i0.DECISION)
            self.assertTrue(all(report["gates"].values()))
            for counter in i0.FORBIDDEN_COUNTERS:
                self.assertEqual(report["access"][counter], 0)

    def test_unknown_origin_fails_closed_into_quarantine(self) -> None:
        source = {
            "current_projects": [
                {
                    "exact_steel_groups": 99,
                    "non_metal_groups": 99,
                    "origin": "unknown-provider",
                    "project_revision_id": "unknown-project-revision",
                }
            ]
        }
        classified = i0.classify_projects(source)
        self.assertEqual(classified["clean_projects"], [])
        self.assertEqual(classified["quarantined_projects"], [])
        self.assertEqual(
            classified["unknown_projects"][0]["project_revision_id"],
            "unknown-project-revision",
        )
        self.assertFalse(classified["classifications"][0]["protected_eligible"])

    def test_policy_authority_family_and_dependency_mutations_reject(self) -> None:
        mutations = []
        threshold = copy.deepcopy(self.profile)
        threshold["minimums"]["protected_exact_steel_groups_per_role"] = 15
        mutations.append((threshold, "minimums"))
        authority = copy.deepcopy(self.profile)
        authority["authority"]["role_assignment_authority"] = True
        mutations.append((authority, "authority"))
        family = copy.deepcopy(self.profile)
        family["family_policies"][0]["protected_power_allowed"] = True
        mutations.append((family, "opened family cannot allow protected power"))
        dependency = copy.deepcopy(self.profile)
        dependency["dependency_bindings"][0]["sha256"] = "0" * 64
        mutations.append((dependency, "dependency drift"))

        with tempfile.TemporaryDirectory(prefix="nextengine-v40-i0-mutation-") as temp:
            root = Path(temp)
            for index, (mutation, expected) in enumerate(mutations):
                profile = root / f"profile-{index}.json"
                profile.write_bytes(i0.canonical_json(mutation))
                output = root / f"output-{index}"
                with self.subTest(expected=expected):
                    with self.assertRaisesRegex(i0.ProjectExposureError, expected):
                        i0.run(profile, output)
                    self.assertFalse(output.exists())

    def test_noncanonical_duplicate_json_and_output_guards_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v40-i0-boundary-") as temp:
            root = Path(temp)
            noncanonical = root / "noncanonical.json"
            noncanonical.write_text(json.dumps(self.profile))
            with self.assertRaisesRegex(i0.ProjectExposureError, "canonical JSON"):
                i0.run(noncanonical, root / "noncanonical-output")

            duplicate = root / "duplicate.json"
            duplicate.write_bytes(b'{"schema": "one", "schema": "two"}\n')
            with self.assertRaisesRegex(i0.ProjectExposureError, "duplicate JSON key"):
                i0.run(duplicate, root / "duplicate-output")

            in_repository = ROOT / "target" / "v40-i0-forbidden"
            with self.assertRaisesRegex(i0.ProjectExposureError, "outside"):
                i0.prepare_output(in_repository)

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(i0.ProjectExposureError, "replace existing"):
                i0.run(PROFILE, occupied)

            actual = root / "actual"
            actual.mkdir()
            linked = root / "linked"
            linked.symlink_to(actual, target_is_directory=True)
            with self.assertRaisesRegex(i0.ProjectExposureError, "symlinks"):
                i0.run(PROFILE, linked / "output")

    def test_late_publication_failure_leaves_no_partial_result(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v40-i0-atomic-") as temp:
            root = Path(temp)
            destination = root / "publication"
            original_write_bytes = Path.write_bytes

            def fail_report(path: Path, data: bytes) -> int:
                if path.name == "report.json":
                    raise OSError("injected publication failure")
                return original_write_bytes(path, data)

            with mock.patch.object(Path, "write_bytes", autospec=True) as writer:
                writer.side_effect = fail_report
                with self.assertRaisesRegex(OSError, "injected publication failure"):
                    i0.run(PROFILE, destination)
            self.assertFalse(destination.exists())
            self.assertFalse(
                any(path.name.startswith(".publication.") for path in root.iterdir())
            )

    def test_owner_has_no_network_signal_or_model_dependency(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "import numpy",
            "import requests",
            "import socket",
            "import torch",
            "urllib.request",
            "wave.open",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
