from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v41_d1_alias_component_roster_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v41-d1-alias-component-roster.v1.json"
D0_PROFILE = LAB / "profiles" / "physical-sound-v40-d0-disclosed-roster.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v41_d1_alias_component_roster_v1 as d1


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def read_json(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text())
    assert isinstance(value, dict), f"expected object in {path}"
    return value


class AliasComponentRosterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls._d0_temp = tempfile.TemporaryDirectory(
            prefix="nextengine-v41-d1-d0-fixture-"
        )
        d0_output = Path(cls._d0_temp.name) / "d0"
        d1.d0.run(D0_PROFILE, d0_output)
        cls.d0_roster_path = d0_output / "disclosed-roster.json"
        cls.profile_bytes, raw_profile = d1.read_canonical_json(
            PROFILE, "V41 D1 profile", d1.MAX_PROFILE_BYTES
        )
        roadmap_binding = next(
            row
            for row in raw_profile["dependency_bindings"]
            if row["binding_id"] == "v41-roadmap"
        )
        current_bound_file = d1.bound_repository_file

        def historical_dependency(path_text: str) -> dict[str, object]:
            if path_text == d1.DEPENDENCY_PATHS["v41-roadmap"]:
                return {
                    key: roadmap_binding[key] for key in ("bytes", "path", "sha256")
                }
            return current_bound_file(path_text)

        cls._roadmap_patch = mock.patch.object(
            d1, "bound_repository_file", side_effect=historical_dependency
        )
        cls._roadmap_patch.start()
        cls.profile, cls.dependencies = d1.validate_profile(raw_profile)
        cls.d0_bytes, cls.d0_roster = d1.load_d0_roster(cls.d0_roster_path)

    @classmethod
    def tearDownClass(cls) -> None:
        cls._roadmap_patch.stop()
        cls._d0_temp.cleanup()

    def test_exact_alias_component_role_repair(self) -> None:
        repaired = d1.repair_roster(self.d0_roster)
        self.assertEqual(repaired["result"], d1.EXPECTED_RESULT)
        self.assertEqual(
            repaired["role_counts"],
            {
                "generator_development": 2,
                "generator_train": 5,
                "validator_calibration": 2,
            },
        )
        rows = {row["family_id"]: row for row in repaired["rows"]}
        component = d1.ALIAS_COMPONENTS[0]["component_id"]
        for family in d1.ALIAS_COMPONENTS[0]["family_ids"]:
            self.assertEqual(rows[family]["family_component_id"], component)
            self.assertEqual(rows[family]["role"], "generator_train")
        self.assertEqual(rows["zisen-shao--av-msf"]["role"], "validator_calibration")

    def test_source_d0_contains_the_detected_cross_role_alias(self) -> None:
        original = {row["family_id"]: row["role"] for row in self.d0_roster["families"]}
        components = d1.family_component_map(set(original))
        original_by_component = d1.role_by_component(original, components)
        alias = d1.ALIAS_COMPONENTS[0]["component_id"]
        self.assertEqual(
            original_by_component[alias],
            {"generator_train", "validator_calibration"},
        )
        repaired = d1.repair_roster(self.d0_roster)
        for row in repaired["component_roles"]:
            self.assertEqual(len(set(row["family_ids"])), len(row["family_ids"]))

    def test_owner_a_b_is_byte_exact_atomic_and_zero_signal(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-d1-repeat-") as temp:
            root = Path(temp)
            outputs = [root / "a", root / "b"]
            for output in outputs:
                d1.run(PROFILE, self.d0_roster_path, output)
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
            self.assertEqual(report["decision"], d1.DECISION)
            self.assertTrue(all(report["gates"].values()))
            for counter in d1.FORBIDDEN_COUNTERS:
                self.assertEqual(report["access"][counter], 0)
            self.assertFalse(report["authority"]["payload_access_authority"])
            self.assertFalse(report["authority"]["model_training_authority"])

    def test_profile_role_alias_and_dependency_mutations_reject(self) -> None:
        mutations: list[tuple[dict[str, object], str]] = []
        role = copy.deepcopy(self.profile)
        role["assignments"][6]["role"] = "validator_calibration"
        mutations.append((role, "assignments"))
        alias = copy.deepcopy(self.profile)
        alias["alias_components"][0]["family_ids"].pop()
        mutations.append((alias, "alias"))
        authority = copy.deepcopy(self.profile)
        authority["authority"]["payload_access_authority"] = True
        mutations.append((authority, "authority"))
        dependency = copy.deepcopy(self.profile)
        dependency["dependency_bindings"][0]["sha256"] = "0" * 64
        mutations.append((dependency, "dependency drift"))
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-d1-mutation-") as temp:
            root = Path(temp)
            for index, (mutation, expected) in enumerate(mutations):
                profile = root / f"profile-{index}.json"
                profile.write_bytes(d1.canonical_json(mutation))
                with (
                    self.subTest(expected=expected),
                    self.assertRaisesRegex(d1.AliasRosterError, expected),
                ):
                    d1.run(
                        profile,
                        self.d0_roster_path,
                        root / f"output-{index}",
                    )

    def test_d0_roster_mutation_rejects_before_publication(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-d1-d0-") as temp:
            root = Path(temp)
            mutated = copy.deepcopy(self.d0_roster)
            mutated["families"][0]["role"] = "generator_train"
            source = root / "d0.json"
            source.write_bytes(d1.canonical_json(mutated))
            output = root / "output"
            with self.assertRaisesRegex(d1.AliasRosterError, "bytes or SHA-256"):
                d1.run(PROFILE, source, output)
            self.assertFalse(output.exists())

    def test_noncanonical_and_output_guards_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-d1-boundary-") as temp:
            root = Path(temp)
            noncanonical = root / "profile.json"
            noncanonical.write_text(json.dumps(self.profile))
            with self.assertRaisesRegex(d1.AliasRosterError, "canonical JSON"):
                d1.run(
                    noncanonical,
                    self.d0_roster_path,
                    root / "noncanonical-output",
                )

            inside = ROOT / "target" / "v41-d1-forbidden"
            with self.assertRaisesRegex(d1.AliasRosterError, "outside"):
                d1.prepare_output(inside)

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(d1.AliasRosterError, "refusing to replace"):
                d1.prepare_output(occupied)

            linked = root / "linked"
            linked.symlink_to(root / "actual", target_is_directory=True)
            with self.assertRaisesRegex(d1.AliasRosterError, "symlink"):
                d1.prepare_output(linked / "result")

    def test_late_publication_failure_leaves_no_partial_result(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-d1-atomic-") as temp:
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

            with (
                mock.patch.object(Path, "write_bytes", fail_third_write),
                self.assertRaisesRegex(OSError, "injected"),
            ):
                d1.run(PROFILE, self.d0_roster_path, output)
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
