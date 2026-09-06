from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v36_a0_typed_owner_conformance_v1 as a0
import physical_sound_v36_owner_contract_v1 as contract


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class TypedOwnerConformanceTests(unittest.TestCase):
    def test_contract_and_mutation_conformance_have_zero_official_access(self) -> None:
        contract_data, conformance, evidence = a0.run_conformance()
        self.assertEqual(contract_data["schema"], contract.CONTRACT_SCHEMA)
        checks = conformance["checks"]
        mutations = conformance["mutations"]
        self.assertIsInstance(checks, dict)
        self.assertIsInstance(mutations, dict)
        self.assertTrue(all(checks.values()))
        self.assertEqual(len(mutations), 10)
        self.assertTrue(all(mutations.values()))
        official = evidence["official_access"]
        self.assertIsInstance(official, dict)
        self.assertTrue(all(value == 0 for value in official.values()))

    def test_complete_d0_and_h0_topologies_are_distinct_and_frozen(self) -> None:
        contract_data, conformance, _ = a0.run_conformance()
        pipelines = contract_data["pipelines"]
        self.assertIsInstance(pipelines, dict)
        self.assertNotEqual(
            pipelines["d0"]["topology_sha256"],
            pipelines["h0"]["topology_sha256"],
        )
        self.assertEqual(
            conformance["d0_topology_sha256"],
            contract.expected_topology_sha256(contract.PipelineKind.D0),
        )
        self.assertEqual(
            conformance["h0_topology_sha256"],
            contract.expected_topology_sha256(contract.PipelineKind.H0),
        )

    def test_owner_publishes_four_files_twice_exactly(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v36-a0-") as temporary:
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
            self.assertEqual(stored["decision"], "Pass")
            self.assertEqual(
                stored["next_authorized_stage"],
                "V36-F0-fresh-role-and-unchanged-science-freeze",
            )

    def test_owner_refuses_existing_output_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-a0-existing-"
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
            a0.bound_file(a0.V35_OWNER_PATH, "0" * 64)


if __name__ == "__main__":
    unittest.main()
