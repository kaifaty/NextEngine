from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = (
    LAB / "profiles" / "physical-sound-v34-f0-target-safe-spectral-recovery.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v34_t0_terminal_path_proof_v1 as t0
import physical_sound_v34_terminal_publication_v1 as terminal


class TerminalPathProofTests(unittest.TestCase):
    def test_pass_and_reject_freeze_contracts_publish_atomically(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v34-terminal-"
        ) as temporary:
            root = Path(temporary)
            access = t0.discarded_access("post-access")
            passed = root / "pass"
            report = terminal.publish_scientific_terminal(
                passed,
                ROOT,
                "Pass",
                access,
                {
                    "payload.bin": b"discarded",
                    terminal.FREEZE_NAME: terminal.canonical_json({"discarded": True}),
                },
                t0.CLAIM,
                1_000_000,
            )
            self.assertEqual(report["decision"], "Pass")
            self.assertTrue(report["candidate_frozen"])
            self.assertEqual(
                {path.name for path in passed.iterdir()},
                {
                    terminal.FREEZE_NAME,
                    "evidence.json",
                    "payload.bin",
                    "report.json",
                },
            )
            rejected = root / "reject"
            report = terminal.publish_scientific_terminal(
                rejected,
                ROOT,
                "MetricReject",
                access,
                {"payload.bin": b"discarded"},
                t0.CLAIM,
                1_000_000,
            )
            self.assertEqual(report["decision"], "MetricReject")
            self.assertFalse(report["candidate_frozen"])
            self.assertNotIn(terminal.FREEZE_NAME, {p.name for p in rejected.iterdir()})

    def test_candidate_freeze_mismatch_rejects_before_output(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v34-terminal-"
        ) as temporary:
            root = Path(temporary)
            with self.assertRaisesRegex(
                terminal.TerminalPublicationError, "freeze/outcome mismatch"
            ):
                terminal.publish_scientific_terminal(
                    root / "bad-pass",
                    ROOT,
                    "Pass",
                    t0.discarded_access("post-access"),
                    {"payload.bin": b"discarded"},
                    t0.CLAIM,
                    1_000_000,
                )
            self.assertFalse((root / "bad-pass").exists())
            with self.assertRaisesRegex(
                terminal.TerminalPublicationError, "freeze/outcome mismatch"
            ):
                terminal.publish_scientific_terminal(
                    root / "bad-reject",
                    ROOT,
                    "HardGateReject",
                    t0.discarded_access("post-access"),
                    {terminal.FREEZE_NAME: b"forbidden"},
                    t0.CLAIM,
                    1_000_000,
                )
            self.assertFalse((root / "bad-reject").exists())

    def test_injected_post_access_failures_leave_no_partial_output(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v34-terminal-"
        ) as temporary:
            root = Path(temporary)
            for failpoint in ("after-first-payload", "after-evidence"):
                output = root / failpoint
                with self.assertRaisesRegex(
                    terminal.TerminalPublicationError, "injected"
                ):
                    terminal.publish_scientific_terminal(
                        output,
                        ROOT,
                        "MetricReject",
                        t0.discarded_access("post-access"),
                        {"payload-a.bin": b"a", "payload-b.bin": b"b"},
                        t0.CLAIM,
                        1_000_000,
                        failpoint=failpoint,
                    )
                self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v34-terminal-")
                    for path in root.iterdir()
                )
            )

    def test_pre_access_reject_has_zero_access_and_no_output(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v34-terminal-"
        ) as temporary:
            output = Path(temporary) / "must-not-exist"
            receipt = terminal.pre_access_contract_reject(
                output,
                ROOT,
                t0.discarded_access("pre-access"),
                "discarded-invalid-profile",
            )
            self.assertEqual(receipt["decision"], "ContractReject")
            self.assertFalse(receipt["output_published"])
            self.assertEqual(receipt["access"]["target_rows_accessed"], 0)
            self.assertFalse(output.exists())
            bad = t0.discarded_access("pre-access")
            bad["target_rows_accessed"] = 1
            with self.assertRaisesRegex(
                terminal.TerminalPublicationError, "pre-access reject"
            ):
                terminal.pre_access_contract_reject(
                    output, ROOT, bad, "discarded-invalid-profile"
                )

    def test_t0_context_import_boundary_and_full_shapes_are_closed(self) -> None:
        _, effective, profile_data = t0.load_context(PROFILE)
        self.assertEqual(t0.sha256_bytes(profile_data), t0.f0.PROFILE_SHA256)
        self.assertEqual(t0.validate_import_boundary()["forbidden_imports"], [])
        self.assertEqual(effective["model"]["parameter_count"], 1811)
        self.assertEqual(
            len(t0.discarded_payloads("MetricReject")["discarded-targets.bin"]),
            8 + 8 + 10_800 * 3 * 8,
        )
        self.assertEqual(
            len(t0.discarded_payloads("MetricReject")["discarded-predictions.bin"]),
            8 + 8 + 4_320 * 3 * 8,
        )
        self.assertIn(terminal.FREEZE_NAME, t0.discarded_payloads("Pass"))
        self.assertNotIn(terminal.FREEZE_NAME, t0.discarded_payloads("HardGateReject"))
        self.assertEqual(
            json.loads(
                t0.discarded_payloads("ResourceReject")["resource-reject-receipt.json"]
            )["reason"],
            "injected-resource-reject",
        )


if __name__ == "__main__":
    unittest.main()
