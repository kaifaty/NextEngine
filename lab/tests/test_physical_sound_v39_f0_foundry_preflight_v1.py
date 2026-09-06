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
SCRIPT = SCRIPTS / "physical_sound_v39_f0_foundry_preflight_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v39-f0-foundry.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v39_f0_foundry_preflight_v1 as f0


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


class FoundryPreflightTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile_bytes, cls.profile = f0.load_and_validate_profile(PROFILE)

    def fixture_documents(
        self, root: Path
    ) -> tuple[bytes, dict[str, object], bytes, dict[str, object]]:
        fixture = root / "fixture"
        f0.run_fixture(PROFILE, fixture)
        plan_bytes = (fixture / "role-plan.json").read_bytes()
        ledger_bytes = (fixture / "access-ledger.json").read_bytes()
        return (
            plan_bytes,
            read_json(fixture / "role-plan.json"),
            ledger_bytes,
            read_json(fixture / "access-ledger.json"),
        )

    def test_fixture_and_validate_cli_are_repeat_exact_and_target_free(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-repeat-"
        ) as temporary:
            root = Path(temporary)
            outputs = [root / "a", root / "b"]
            for output in outputs:
                completed = subprocess.run(
                    [
                        sys.executable,
                        str(SCRIPT),
                        "--stage",
                        "fixture",
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
                    "descriptor.json",
                    "profile.json",
                    "report.json",
                    "role-plan.json",
                },
            )
            report = read_json(outputs[0] / "report.json")
            self.assertEqual(report["decision"], "MechanicsReady")
            self.assertEqual(
                report["next_authorized_stage"],
                "V39-F1-source-frontier-automation",
            )
            self.assertEqual(report["access_counters"], f0.zero_counters())
            self.assertEqual(report["counts"]["project_count"], 6)
            self.assertTrue(all(report["invariants"].values()))
            self.assertFalse(report["authority"]["model_training_authority"])
            self.assertFalse(report["authority"]["admission_authority"])

            validated = root / "validated"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--stage",
                    "validate",
                    "--profile",
                    str(PROFILE),
                    "--role-plan",
                    str(outputs[0] / "role-plan.json"),
                    "--access-ledger",
                    str(outputs[0] / "access-ledger.json"),
                    "--output",
                    str(validated),
                ],
                cwd=ROOT,
                check=False,
                capture_output=True,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr.decode())
            self.assertEqual(artifact_tree(outputs[0]), artifact_tree(validated))

    def test_role_partition_rejects_project_object_and_recording_leaks(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-role-leak-"
        ) as temporary:
            root = Path(temporary)
            plan_bytes, plan, _, _ = self.fixture_documents(root)
            profile_hash = f0.sha256_bytes(self.profile_bytes)

            duplicate_project = copy.deepcopy(plan)
            left = duplicate_project["projects"][0]
            right = duplicate_project["projects"][1]
            for key in ("publisher_id", "project_id", "revision_id"):
                right[key] = left[key]
            right["project_parent_sha256"] = left["project_parent_sha256"]
            duplicate_project["projects"].sort(
                key=lambda item: item["project_parent_sha256"]
            )
            with self.assertRaisesRegex(f0.FoundryPreflightError, "project revision"):
                f0.validate_role_plan(
                    f0.canonical_json(duplicate_project),
                    duplicate_project,
                    profile_hash,
                )

            duplicate_object = copy.deepcopy(plan)
            duplicate_object["projects"][1]["object_parent_sha256s"] = copy.deepcopy(
                duplicate_object["projects"][0]["object_parent_sha256s"]
            )
            with self.assertRaisesRegex(f0.FoundryPreflightError, "object parent"):
                f0.validate_role_plan(
                    f0.canonical_json(duplicate_object),
                    duplicate_object,
                    profile_hash,
                )

            duplicate_recording = copy.deepcopy(plan)
            duplicate_recording["projects"][1]["recording_parent_sha256s"] = (
                copy.deepcopy(
                    duplicate_recording["projects"][0]["recording_parent_sha256s"]
                )
            )
            with self.assertRaisesRegex(f0.FoundryPreflightError, "recording parent"):
                f0.validate_role_plan(
                    f0.canonical_json(duplicate_recording),
                    duplicate_recording,
                    profile_hash,
                )

            self.assertEqual(
                f0.validate_role_plan(plan_bytes, plan, profile_hash),
                plan,
            )

    def test_role_flags_sealing_and_complete_roster_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-role-policy-"
        ) as temporary:
            plan_bytes, plan, _, _ = self.fixture_documents(Path(temporary))
            del plan_bytes
            profile_hash = f0.sha256_bytes(self.profile_bytes)

            mutations = []
            wrong_protected = copy.deepcopy(plan)
            wrong_protected["projects"][0]["protected"] = not wrong_protected[
                "projects"
            ][0]["protected"]
            mutations.append((wrong_protected, "protected flag"))

            wrong_permanence = copy.deepcopy(plan)
            wrong_permanence["projects"][0]["permanent_disclosed"] = not (
                wrong_permanence["projects"][0]["permanent_disclosed"]
            )
            mutations.append((wrong_permanence, "permanence"))

            opened_signal = copy.deepcopy(plan)
            opened_signal["projects"][0]["signal_state"] = "decoded"
            mutations.append((opened_signal, "signal must remain sealed"))

            missing_role = copy.deepcopy(plan)
            missing_role["projects"] = missing_role["projects"][:-1]
            mutations.append((missing_role, "omits roles"))

            for mutation, message in mutations:
                with self.subTest(message=message):
                    with self.assertRaisesRegex(f0.FoundryPreflightError, message):
                        f0.validate_role_plan(
                            f0.canonical_json(mutation),
                            mutation,
                            profile_hash,
                        )

    def test_access_ledger_rejects_every_forbidden_access_class(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-access-"
        ) as temporary:
            plan_bytes, plan, ledger_bytes, ledger = self.fixture_documents(
                Path(temporary)
            )
            self.assertEqual(
                f0.validate_access_ledger(ledger_bytes, ledger, plan_bytes, plan),
                ledger,
            )

            for counter in (
                "network_requests",
                "model_target_values_read",
                "protected_signal_values_decoded",
                "pcm_sample_values_decoded",
                "source_artifact_bytes_read",
            ):
                mutation = copy.deepcopy(ledger)
                mutation["entries"][0]["counters"][counter] = 1
                mutation["counters"][counter] = 1
                with self.subTest(counter=counter):
                    with self.assertRaisesRegex(
                        f0.FoundryPreflightError, "access must remain zero"
                    ):
                        f0.validate_access_ledger(
                            f0.canonical_json(mutation), mutation, plan_bytes, plan
                        )

            wrong_role = copy.deepcopy(ledger)
            wrong_role["entries"][0]["role"] = "generator_train"
            if plan["projects"][0]["role"] == "generator_train":
                wrong_role["entries"][0]["role"] = "validator_qualification"
            with self.assertRaisesRegex(f0.FoundryPreflightError, "ledger role"):
                f0.validate_access_ledger(
                    f0.canonical_json(wrong_role), wrong_role, plan_bytes, plan
                )

            missing_entry = copy.deepcopy(ledger)
            missing_entry["entries"] = missing_entry["entries"][:-1]
            with self.assertRaisesRegex(f0.FoundryPreflightError, "cover every project"):
                f0.validate_access_ledger(
                    f0.canonical_json(missing_entry), missing_entry, plan_bytes, plan
                )

    def test_terminal_state_cannot_freeze_or_open_protected_roles(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-terminal-"
        ) as temporary:
            plan_bytes, plan, _, ledger = self.fixture_documents(Path(temporary))
            mutations = []
            frozen = copy.deepcopy(ledger)
            frozen["terminal"]["candidate_frozen"] = True
            mutations.append(frozen)
            validator_frozen = copy.deepcopy(ledger)
            validator_frozen["terminal"]["validator_release_frozen"] = True
            mutations.append(validator_frozen)
            opened = copy.deepcopy(ledger)
            opened["terminal"]["protected_roles_opened"] = [
                "validator_qualification"
            ]
            mutations.append(opened)
            admitted = copy.deepcopy(ledger)
            admitted["terminal"]["outcome"] = "Pass"
            mutations.append(admitted)

            for mutation in mutations:
                with self.assertRaisesRegex(f0.FoundryPreflightError, "terminal state"):
                    f0.validate_access_ledger(
                        f0.canonical_json(mutation), mutation, plan_bytes, plan
                    )

    def test_profile_budget_authority_and_dependency_drift_reject_before_output(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-profile-"
        ) as temporary:
            root = Path(temporary)
            mutations = []
            budget = copy.deepcopy(self.profile)
            budget["experiment_budget"]["train_step_limit"] += 1
            mutations.append((budget, "experiment_budget"))
            authority = copy.deepcopy(self.profile)
            authority["authority"]["model_training_authority"] = True
            mutations.append((authority, "authority"))
            dependency = copy.deepcopy(self.profile)
            dependency["dependency_bindings"][0]["sha256"] = "0" * 64
            mutations.append((dependency, "bound file drift"))

            for index, (mutation, message) in enumerate(mutations):
                profile = root / f"profile-{index}.json"
                profile.write_bytes(f0.canonical_json(mutation))
                output = root / f"output-{index}"
                with self.subTest(message=message):
                    with self.assertRaisesRegex(f0.FoundryPreflightError, message):
                        f0.run_fixture(profile, output)
                    self.assertFalse(output.exists())

    def test_noncanonical_duplicate_json_and_output_guards_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-boundary-"
        ) as temporary:
            root = Path(temporary)
            noncanonical = root / "noncanonical.json"
            noncanonical.write_text(json.dumps(self.profile))
            with self.assertRaisesRegex(f0.FoundryPreflightError, "canonical JSON"):
                f0.run_fixture(noncanonical, root / "noncanonical-output")

            duplicate = root / "duplicate.json"
            duplicate.write_bytes(b'{"schema": "one", "schema": "two"}\n')
            with self.assertRaisesRegex(f0.FoundryPreflightError, "duplicate JSON key"):
                f0.run_fixture(duplicate, root / "duplicate-output")

            in_repository = ROOT / "target" / "v39-f0-forbidden"
            with self.assertRaisesRegex(f0.FoundryPreflightError, "outside"):
                f0.prepare_output(in_repository)

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(f0.FoundryPreflightError, "replace existing"):
                f0.run_fixture(PROFILE, occupied)

            actual = root / "actual"
            actual.mkdir()
            linked = root / "linked"
            linked.symlink_to(actual, target_is_directory=True)
            with self.assertRaisesRegex(f0.FoundryPreflightError, "symlinks"):
                f0.run_fixture(PROFILE, linked / "output")

    def test_late_publication_failure_leaves_no_partial_directory(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v39-f0-atomic-"
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
                    f0.run_fixture(PROFILE, destination)
            self.assertFalse(destination.exists())
            self.assertFalse(
                any(path.name.startswith(".publication.") for path in root.iterdir())
            )

    def test_owner_source_has_no_signal_model_or_network_dependencies(self) -> None:
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
