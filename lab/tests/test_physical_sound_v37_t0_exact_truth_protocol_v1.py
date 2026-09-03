from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

import numpy as np

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v37_t0_exact_truth_protocol_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v37-t0-exact-truth-protocol.v1.json"
SEAL = LAB / "profiles" / "physical-sound-v37-t0-official-input-seal.v1.json"
C0_PROFILE = (
    LAB / "profiles" / "physical-sound-v37-c0-query-surface-structural-cost.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0
import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_t0_exact_truth_protocol_v1 as t0


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class ExactTruthProtocolTests(unittest.TestCase):
    context: t0.LoadedContext

    @classmethod
    def setUpClass(cls) -> None:
        cls.context = t0.load_context(PROFILE)

    def test_profile_prf_coefficients_and_prior_seal_are_exact(self) -> None:
        commitment = t0.coefficient_commitment(self.context.profile)
        self.assertEqual(commitment["coefficient_count"], 72)
        self.assertEqual(
            commitment["coefficient_root_sha256"],
            "07932a8069157b0b62eb49fe166f94f857d5b7d717a1fea86636d18091070ace",
        )
        self.assertEqual(
            t0.coefficient_unit(
                370021,
                "local.theta_radians",
                0,
                "nextengine.physical-sound.v37.t0.coefficient.v1",
            ),
            0.5222345820976055,
        )
        self.assertEqual(
            self.context.e0_seal["seal_sha256"],
            "608f901938c4031e63353a9231e005bb1040ac21c50ed3c7746e01525e211d7f",
        )

    def test_official_row_metadata_is_target_free_and_matches_structural_rows(
        self,
    ) -> None:
        commitments = t0.metadata_commitments(self.context.f0_profile)
        self.assertEqual(
            commitments["all_roles_root_sha256"],
            "48508bf0226e4d3894d577bc0f77affb7f71d4bc6b0c4864cd5b6a4c52bdf9b0",
        )
        self.assertEqual(
            commitments["roles"],
            {
                "development": {
                    "root_sha256": "c75e9bbedf97fd45f8b905928e3a516f62b0ca3a3629f87c38852889f9f6bd00",
                    "row_count": 4320,
                },
                "method_holdout": {
                    "root_sha256": "95d5a300151ea1a2dee7f129c9770ed9729ff3a2ae298dfc782b819de47af1df",
                    "row_count": 4320,
                },
                "train": {
                    "root_sha256": "9906ea4490669558691503b9aca5d5291117229eb3143cfcb051bd97a9c7bda0",
                    "row_count": 6480,
                },
            },
        )
        c0_context = c0.load_context(C0_PROFILE)
        structural = c0.build_role(c0_context, "development").batch
        metadata = t0.official_role_metadata(self.context.f0_profile, "development")
        self.assertFalse(structural.has_targets)
        self.assertEqual(tuple(row.row_id for row in metadata), structural.row_ids)

    def test_artificial_truth_is_bounded_exact_and_mutation_sensitive(self) -> None:
        conformance = t0.artificial_conformance(self.context.profile)
        self.assertTrue(conformance["artificial_only"])
        self.assertTrue(conformance["permutation_exact"])
        self.assertTrue(conformance["mode_mutation_observed"])
        self.assertTrue(conformance["topology_mutation_observed"])
        self.assertTrue(conformance["mixture_mutation_observed"])
        self.assertEqual(
            conformance["target_root_sha256"],
            "6dbfe6ada1cd8b995e7960df3d18281acf5d8b6f58c0117f4c5ac23ceb9d758c",
        )
        self.assertEqual(conformance["official_access"], t0.ZERO_ACCESS)
        batch, metadata = t0.artificial_fixture()
        result = t0.evaluate_targets(self.context.profile, batch, metadata)
        self.assertFalse(result.targets.flags.writeable)
        self.assertLessEqual(float(np.max(np.abs(result.targets[:, 0]))), 0.25)
        self.assertLessEqual(float(np.max(np.abs(result.targets[:, 1]))), 0.20)
        self.assertLessEqual(float(np.max(np.abs(result.targets[:, 2]))), 0.25)

    def test_ambiguous_protocol_metadata_and_labelled_input_are_rejected(self) -> None:
        invalid = copy.deepcopy(self.context.profile)
        invalid["protocol"]["coefficient_prf"]["purpose"] = "unspecified"
        with self.assertRaisesRegex(t0.T0ProtocolError, "PRF contract drift"):
            t0.validate_protocol(invalid)
        batch, metadata = t0.artificial_fixture()
        with self.assertRaisesRegex(t0.T0ProtocolError, "canonical batch row order"):
            t0.evaluate_targets(self.context.profile, batch, tuple(reversed(metadata)))
        labelled = replace(
            batch,
            targets=contract.frozen_float_matrix(
                tuple((0.0, 0.0, 0.0) for _ in range(batch.row_count))
            ),
        )
        with self.assertRaisesRegex(t0.T0ProtocolError, "unlabelled batch"):
            t0.evaluate_targets(self.context.profile, labelled, metadata)

    def test_checked_composite_seal_binds_current_owner_and_both_protocols(
        self,
    ) -> None:
        coefficients = t0.coefficient_commitment(self.context.profile)
        metadata = t0.metadata_commitments(self.context.f0_profile)
        conformance = t0.artificial_conformance(self.context.profile)
        expected = t0.composite_seal(self.context, coefficients, metadata, conformance)
        actual = t0.validate_checked_official_input_seal(expected, SEAL)
        self.assertEqual(
            actual["seal_sha256"],
            "7bae65cc0290c3ec54eedef5d9ef2dd3d8e4ab1881945db71c8c80dcd1361573",
        )
        self.assertEqual(
            actual["composite_seal"]["prior_e0_execution_seal_sha256"],
            self.context.e0_seal["seal_sha256"],
        )
        self.assertEqual(actual["composite_seal"]["forbidden_access_count"], 0)

    def test_complete_cli_twice_is_exact_and_opens_no_official_values(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-t0-repeat-") as raw:
            root = Path(raw)
            outputs = [root / "a", root / "b"]
            stdout: list[bytes] = []
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
                self.assertEqual(completed.returncode, 0, completed.stdout.decode())
                self.assertEqual(completed.stderr, b"")
                stdout.append(completed.stdout)
            self.assertEqual(stdout[0], stdout[1])
            self.assertEqual(artifact_tree(outputs[0]), artifact_tree(outputs[1]))
            self.assertEqual(
                set(artifact_tree(outputs[0])),
                {
                    "artificial-conformance.json",
                    "coefficient-commitment.json",
                    "evidence.json",
                    "official-input-seal.json",
                    "report.json",
                    "row-metadata-commitments.json",
                },
            )
            report = json.loads((outputs[0] / "report.json").read_text())
            self.assertEqual(report["decision"], "T0_EXACT_TRUTH_PROTOCOL_PASS")
            self.assertFalse(report["official_values_opened"])
            self.assertEqual(report["official_access"], t0.ZERO_ACCESS)

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-t0-failure-") as raw:
            root = Path(raw)
            corrupt = copy.deepcopy(self.context.profile)
            corrupt["protocol"]["contact"]["bound"] = "0.26"
            corrupt_path = root / "corrupt.json"
            corrupt_path.write_bytes(t0.canonical_json(corrupt))
            output = root / "output"
            with self.assertRaisesRegex(t0.T0ProtocolError, "profile hash mismatch"):
                t0.run(corrupt_path, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v37-t0-")
                    for path in root.iterdir()
                )
            )
            output.mkdir()
            with self.assertRaisesRegex(t0.T0ProtocolError, "fresh external path"):
                t0.run(PROFILE, output)


if __name__ == "__main__":
    unittest.main()
