from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_a0_query_surface_contract_v1 as a0
import physical_sound_v37_query_surface_contract_v1 as contract


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class A0QuerySurfaceConformanceTests(unittest.TestCase):
    def test_conformance_is_target_free_and_all_mutations_pass(self) -> None:
        contract_data, conformance, evidence = a0.run_conformance()
        self.assertEqual(contract_data["schema"], contract.CONTRACT_SCHEMA)
        checks = conformance["checks"]
        mutations = conformance["mutations"]
        self.assertIsInstance(checks, dict)
        self.assertIsInstance(mutations, dict)
        self.assertTrue(all(checks.values()))
        self.assertEqual(tuple(mutations), a0.profile_mutation_ids(a0.load_profile()))
        self.assertTrue(all(mutations.values()))
        access = evidence["official_access"]
        self.assertIsInstance(access, dict)
        self.assertTrue(all(value == 0 for value in access.values()))

    def test_profile_freezes_both_structural_lifecycles(self) -> None:
        _, conformance, _ = a0.run_conformance()
        profile = a0.load_profile()
        d0 = conformance["structural_d0_trace"]
        h0 = conformance["structural_h0_trace"]
        self.assertIsInstance(d0, dict)
        self.assertIsInstance(h0, dict)
        self.assertEqual(
            tuple(event["stage"] for event in d0["events"]),
            a0.profile_stages(profile, "structural-d0"),
        )
        self.assertEqual(
            tuple(event["stage"] for event in h0["events"]),
            a0.profile_stages(profile, "structural-h0"),
        )

    def test_owner_publishes_four_files_twice_exactly(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-a0-") as temporary:
            root = Path(temporary)
            output_a = root / "a"
            output_b = root / "b"
            report_a = a0.run(output_a)
            report_b = a0.run(output_b)
            self.assertEqual(report_a, report_b)
            self.assertEqual(artifact_tree(output_a), artifact_tree(output_b))
            self.assertEqual(
                set(artifact_tree(output_a)),
                {"conformance.json", "contract.json", "evidence.json", "report.json"},
            )
            stored = json.loads((output_a / "report.json").read_text())
            self.assertEqual(stored["decision"], "PreflightPass")
            self.assertEqual(
                stored["next_authorized_stage"],
                "V37-F0-fresh-operator-truth-and-role-freeze",
            )

    def test_owner_refuses_existing_output_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v37-a0-existing-"
        ) as temporary:
            output = Path(temporary) / "existing"
            output.mkdir()
            sentinel = output / "sentinel"
            sentinel.write_text("keep")
            with self.assertRaisesRegex(a0.A0ConformanceError, "already exists"):
                a0.run(output)
            self.assertEqual(sentinel.read_text(), "keep")

    def test_bound_file_hash_drift_fails_before_output(self) -> None:
        with self.assertRaisesRegex(a0.A0ConformanceError, "bound file drift"):
            a0.bound_file(a0.V36_RESULT_PATH, "0" * 64)


if __name__ == "__main__":
    unittest.main()
