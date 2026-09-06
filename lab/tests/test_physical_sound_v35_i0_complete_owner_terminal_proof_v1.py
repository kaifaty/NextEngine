from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v34_terminal_publication_v1 as terminal
import physical_sound_v35_i0_complete_owner_terminal_proof_v1 as i0


class CompleteOwnerTerminalProofTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, cls.effective, _, cls.dependencies = i0.validate_context(PROFILE)
        cls.b0_conformance = i0.b0.build_conformance(PROFILE)
        cls.b0_data = i0.canonical_json(cls.b0_conformance)

    def test_context_imports_and_full_shapes_are_frozen(self) -> None:
        self.assertEqual(i0.validate_import_boundary()["forbidden_imports"], [])
        self.assertEqual(
            set(self.dependencies),
            {
                "b0_owner",
                "b0_result",
                "terminal_owner",
                "v34_t0_result",
            },
        )
        counts = self.effective["corpus"]["counts"]
        self.assertEqual(counts["train"]["modal_rows"], 6480)
        self.assertEqual(counts["development"]["modal_rows"], 4320)
        self.assertEqual(
            self.effective["method_overlay"]["candidate"]["model"]["parameter_count"],
            1859,
        )
        self.assertEqual(self.b0_conformance["status"], "Pass")
        self.assertEqual(
            self.b0_conformance["access"]["official_role_rows_materialized"], 0
        )

    def test_every_terminal_payload_has_the_complete_discarded_contract(self) -> None:
        for outcome in terminal.SCIENTIFIC_OUTCOMES:
            payloads = i0.discarded_payloads(outcome, self.effective, self.b0_data)
            i0.validate_discarded_payloads(payloads, outcome)
            self.assertEqual(terminal.FREEZE_NAME in payloads, outcome == "Pass")
        payloads = i0.discarded_payloads("MetricReject", self.effective, self.b0_data)
        self.assertEqual(
            len(payloads["discarded-development-predictions.bin"]),
            16 + 4320 * 13 * 3 * 8,
        )
        manifest = json.loads(payloads["discarded-control-manifest.json"])
        self.assertEqual(len(manifest["controls"]) + len(manifest["ablations"]) + 1, 13)

    def test_publication_failpoints_and_pre_access_reject_are_atomic(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v35-i0-fail-") as temporary:
            root = Path(temporary)
            probes = i0.failpoint_probes(root, 67_108_864)
            self.assertEqual(set(probes), {"after-first-payload", "after-evidence"})
            self.assertTrue(all(probe["status"] == "Pass" for probe in probes.values()))
            output = root / "pre-access-must-not-exist"
            receipt = terminal.pre_access_contract_reject(
                output,
                ROOT,
                i0.discarded_access("pre-access"),
                "discarded-invalid-profile",
            )
            self.assertEqual(receipt["decision"], "ContractReject")
            self.assertFalse(output.exists())
            bad = i0.discarded_access("pre-access")
            bad["target_rows_accessed"] = 1
            with self.assertRaisesRegex(
                terminal.TerminalPublicationError, "pre-access reject"
            ):
                terminal.pre_access_contract_reject(
                    output, ROOT, bad, "discarded-invalid-profile"
                )

    def test_invalid_shape_phase_freeze_and_occupied_output_fail_closed(self) -> None:
        payloads = i0.discarded_payloads("HardGateReject", self.effective, self.b0_data)
        payloads["discarded-train-targets.bin"] = b"short"
        with self.assertRaisesRegex(i0.I0ProofError, "payload size drift"):
            i0.validate_discarded_payloads(payloads, "HardGateReject")
        with self.assertRaisesRegex(i0.I0ProofError, "unknown discarded access"):
            i0.discarded_access("unknown")
        with tempfile.TemporaryDirectory(prefix="nextengine-v35-i0-bad-") as temporary:
            root = Path(temporary)
            with self.assertRaisesRegex(
                terminal.TerminalPublicationError, "freeze/outcome mismatch"
            ):
                terminal.publish_scientific_terminal(
                    root / "bad-pass",
                    ROOT,
                    "Pass",
                    i0.discarded_access("post-access"),
                    {"discarded.bin": b"no-freeze"},
                    i0.CLAIM,
                    67_108_864,
                )
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(i0.I0ProofError, "fresh external path"):
                i0.external_output(occupied)

    def test_complete_owner_entry_publishes_all_paths_without_official_access(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v35-i0-run-") as temporary:
            output = Path(temporary) / "output"
            report = i0.run(PROFILE, output)
            self.assertEqual(
                report["decision"], "I0_COMPLETE_OWNER_TERMINAL_PATH_PROOF_PASS"
            )
            self.assertFalse(report["official_target_or_model_values_opened"])
            self.assertEqual(
                set(report["terminals"]), set(terminal.SCIENTIFIC_OUTCOMES)
            )
            self.assertTrue((output / "pass" / terminal.FREEZE_NAME).is_file())
            for rejected in ("metric-reject", "hardgate-reject", "resource-reject"):
                self.assertFalse((output / rejected / terminal.FREEZE_NAME).exists())
            evidence = json.loads((output / "evidence.json").read_text())
            self.assertEqual(set(evidence["official_access"].values()), {0})


if __name__ == "__main__":
    unittest.main()
