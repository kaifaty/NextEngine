from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v37-x0-complete-owner-terminal.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_terminal_publisher_v1 as publisher
import physical_sound_v37_x0_complete_owner_terminal_v1 as x0


class CompleteOwnerTerminalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, data = x0.load_profile(PROFILE)
        cls.profile_sha256 = x0.sha256_bytes(data)

    def test_profile_closes_dependencies_and_forbids_official_authority(self) -> None:
        self.assertEqual(self.profile_sha256, x0.PROFILE_SHA256)
        authority = x0.object_map(self.profile["authority"], "authority")
        self.assertFalse(authority["official_capability_allowed"])
        self.assertFalse(authority["scientific_target_allowed"])
        self.assertTrue(authority["surrogate_targets_allowed"])

    def test_natural_d0_pass_is_the_only_freeze_shape_and_h0_consumes_it(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-x0-test-") as raw:
            root = Path(raw)
            d0 = x0.run_owner(
                self.profile,
                self.profile_sha256,
                root,
                contract.PipelineKind.D0,
                x0.RunFixture.NATURAL,
                label="d0-pass",
            )
            self.assertEqual(d0.decision, contract.TerminalDecision.PASS)
            self.assertLess(d0.metrics["candidate_rmse"], d0.metrics["p1_control_rmse"])
            self.assertEqual(
                {path.name for path in (root / "d0-pass").iterdir()},
                {
                    "candidate-bundle.json",
                    "candidate-freeze.json",
                    "evidence.json",
                    "terminal.json",
                },
            )
            h0 = x0.run_owner(
                self.profile,
                self.profile_sha256,
                root,
                contract.PipelineKind.H0,
                x0.RunFixture.NATURAL,
                d0_pass_output=root / "d0-pass",
                label="h0-pass",
            )
            self.assertEqual(h0.decision, contract.TerminalDecision.PASS)
            contract.assert_complete_trace(h0.trace)

    def test_every_reject_shape_excludes_freeze_and_candidate_bundle(self) -> None:
        fixtures = {
            x0.RunFixture.METRIC_REJECT: contract.TerminalDecision.METRIC_REJECT,
            x0.RunFixture.HARD_GATE_REJECT: contract.TerminalDecision.HARD_GATE_REJECT,
            x0.RunFixture.RESOURCE_REJECT: contract.TerminalDecision.RESOURCE_REJECT,
        }
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-x0-test-") as raw:
            root = Path(raw)
            for fixture, expected in fixtures.items():
                result = x0.run_owner(
                    self.profile,
                    self.profile_sha256,
                    root,
                    contract.PipelineKind.D0,
                    fixture,
                )
                self.assertEqual(result.decision, expected)
                self.assertIsNotNone(result.receipt)
                output = Path(result.receipt.output)  # type: ignore[union-attr]
                self.assertEqual(
                    {path.name for path in output.iterdir()},
                    {"evidence.json", "rejected-candidate.json", "terminal.json"},
                )

    def test_pre_access_fault_is_absent_and_post_access_fault_is_evidence_only(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-x0-test-") as raw:
            root = Path(raw)
            pre = x0.run_owner(
                self.profile,
                self.profile_sha256,
                root,
                contract.PipelineKind.D0,
                x0.RunFixture.PRE_ACCESS_FAULT,
            )
            self.assertEqual(pre.decision, contract.TerminalDecision.CONTRACT_REJECT)
            self.assertIsNone(pre.receipt)
            self.assertFalse((root / "d0-pre-access-fault").exists())
            post = x0.run_owner(
                self.profile,
                self.profile_sha256,
                root,
                contract.PipelineKind.D0,
                x0.RunFixture.POST_ACCESS_FAULT,
            )
            self.assertEqual(post.decision, contract.TerminalDecision.OWNER_FAULT)
            self.assertEqual(
                {path.name for path in Path(post.receipt.output).iterdir()},  # type: ignore[union-attr]
                {"owner-fault.json", "terminal.json"},
            )

    def test_every_publication_failpoint_leaves_no_partial_tree(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-x0-test-") as raw:
            root = Path(raw)
            for point in publisher.PublicationFailpoint:
                label = f"point-{point.value}"
                with self.assertRaisesRegex(
                    publisher.PublicationError, "injected publication failure"
                ):
                    x0.run_owner(
                        self.profile,
                        self.profile_sha256,
                        root,
                        contract.PipelineKind.D0,
                        x0.RunFixture.METRIC_REJECT,
                        failpoint=point,
                        label=label,
                    )
                self.assertFalse((root / label).exists())
                self.assertFalse(
                    (root / f".{label}.nextengine-v37-terminal-staging").exists()
                )

    def test_full_conformance_matrix_passes_without_official_access(self) -> None:
        result = x0.conformance_suite(self.profile, self.profile_sha256)
        self.assertTrue(result["all_pass"])
        official = x0.object_map(result["official_access"], "official access")
        self.assertTrue(all(value == 0 for value in official.values()))
        self.assertEqual(len(x0.object_map(result["terminals"], "terminals")), 8)


if __name__ == "__main__":
    unittest.main()
