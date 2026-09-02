from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v36_owner_contract_v1 as contract
import physical_sound_v36_terminal_publisher_v1 as publisher
import physical_sound_v36_x0_mutation_terminal_conformance_v1 as x0


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class MutationTerminalConformanceTests(unittest.TestCase):
    def test_all_38_declared_mutations_fail_closed(self) -> None:
        mutations = x0.mutation_suite()
        self.assertIs(mutations["all_pass"], True)
        self.assertEqual(mutations["total_mutations"], 38)
        for group_name in ("container", "lifecycle", "trace"):
            group = mutations[group_name]
            self.assertIsInstance(group, dict)
            self.assertTrue(all(group.values()), group_name)

    def test_terminal_matrix_covers_all_paths_and_failpoints(self) -> None:
        result = x0.publication_suite()
        self.assertIs(result["all_pass"], True)
        complete = result["complete_scientific_terminals"]
        failpoints = result["failpoints"]
        checks = result["checks"]
        self.assertIsInstance(complete, dict)
        self.assertIsInstance(failpoints, dict)
        self.assertIsInstance(checks, dict)
        self.assertEqual(len(complete), 8)
        self.assertEqual(
            set(failpoints), {point.value for point in publisher.PublicationFailpoint}
        )
        self.assertTrue(all(failpoints.values()))
        self.assertTrue(all(checks.values()))

    def test_exception_conversion_is_access_sensitive_and_atomic(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-x0-exception-"
        ) as temporary:
            root = Path(temporary)
            pre_output = root / "pre"
            pre_trace, pre_receipt = x0.classify_unexpected_exception(
                pre_output, after_access=False
            )
            self.assertIs(pre_trace.terminal, contract.TerminalDecision.CONTRACT_REJECT)
            self.assertIsNone(pre_receipt)
            self.assertFalse(pre_output.exists())

            post_output = root / "post"
            post_trace, post_receipt = x0.classify_unexpected_exception(
                post_output, after_access=True
            )
            self.assertIs(post_trace.terminal, contract.TerminalDecision.OWNER_FAULT)
            self.assertIsNotNone(post_receipt)
            self.assertEqual(
                set(artifact_tree(post_output)), {"owner-fault.json", "terminal.json"}
            )

    def test_complete_scientific_traces_have_exact_role_receipts(self) -> None:
        d0 = x0.complete_trace(
            contract.PipelineKind.D0,
            contract.TerminalDecision.PASS,
            "test-d0",
        )
        h0 = x0.complete_trace(
            contract.PipelineKind.H0,
            contract.TerminalDecision.PASS,
            "test-h0",
        )
        self.assertEqual(d0.access.provider_calls, 2)
        self.assertEqual(d0.access.train_target_rows, 2)
        self.assertEqual(d0.access.development_target_rows, 2)
        self.assertEqual(d0.access.method_holdout_target_rows, 0)
        self.assertEqual(h0.access.provider_calls, 2)
        self.assertEqual(h0.access.train_target_rows, 2)
        self.assertEqual(h0.access.development_target_rows, 0)
        self.assertEqual(h0.access.method_holdout_target_rows, 2)

    def test_owner_publishes_six_files_twice_exactly(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-x0-owner-"
        ) as temporary:
            root = Path(temporary)
            output_a = root / "a"
            output_b = root / "b"
            report_a = x0.run(output_a)
            report_b = x0.run(output_b)
            self.assertEqual(report_a, report_b)
            self.assertEqual(artifact_tree(output_a), artifact_tree(output_b))
            self.assertEqual(
                set(artifact_tree(output_a)),
                {
                    "contract.json",
                    "evidence.json",
                    "mutations.json",
                    "publication.json",
                    "report.json",
                    "terminal.json",
                },
            )
            terminal = json.loads((output_a / "terminal.json").read_text())
            self.assertEqual(terminal["schema"], publisher.PUBLISHER_SCHEMA)
            self.assertEqual(terminal["trace"]["terminal"], "Pass")
            self.assertEqual(terminal["trace"]["access"]["forbidden_access_count"], 0)

    def test_owner_refuses_existing_output_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-x0-existing-"
        ) as temporary:
            output = Path(temporary) / "existing"
            output.mkdir()
            sentinel = output / "sentinel"
            sentinel.write_text("keep")
            with self.assertRaisesRegex(
                publisher.PublicationError, "fresh non-symlink"
            ):
                x0.run(output)
            self.assertEqual(sentinel.read_text(), "keep")

    def test_conformance_issues_no_official_capability_or_value_access(self) -> None:
        _, _, _, evidence = x0.run_conformance()
        official = evidence["official_access"]
        self.assertIsInstance(official, dict)
        self.assertTrue(all(value == 0 for value in official.values()))

    def test_bound_dependency_drift_fails_before_output(self) -> None:
        with self.assertRaisesRegex(x0.X0ConformanceError, "bound file drift"):
            x0.bound_file(x0.C0_RESULT_PATH, "0" * 64)


if __name__ == "__main__":
    unittest.main()
