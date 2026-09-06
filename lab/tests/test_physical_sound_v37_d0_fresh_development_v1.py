from __future__ import annotations

import copy
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v37-d0-fresh-development.v1.json"
SCRIPT = SCRIPTS / "physical_sound_v37_d0_fresh_development_v1.py"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_d0_fresh_development_v1 as d0
import physical_sound_v37_d0_official_provider_v1 as official
import physical_sound_v37_query_surface_contract_v1 as contract


class FreshDevelopmentTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.context = d0.load_context(PROFILE)

    def test_profile_binds_provider_ready_owner_and_one_shot_access(self) -> None:
        self.assertEqual(d0.sha256_bytes(self.context.profile_data), d0.PROFILE_SHA256)
        self.assertEqual(
            self.context.profile["execution_seals"]["d0r"]["payload_sha256"],
            official.D0R_SEAL_PAYLOAD_SHA256,
        )
        self.assertEqual(
            self.context.profile["expected_access_per_process"][
                "official_d0_target_rows"
            ],
            10800,
        )
        self.assertFalse(
            self.context.profile["one_shot"][
                "repeat_or_tune_after_post_access_terminal"
            ]
        )

    def test_preflight_opens_no_official_values(self) -> None:
        report = d0.preflight(PROFILE)
        self.assertEqual(report["status"], "D0_PREFLIGHT_PASS_ZERO_OFFICIAL_VALUES")
        access = report["official_access"]
        self.assertIsInstance(access, dict)
        self.assertEqual(access["official_capabilities_issued"], 0)
        self.assertEqual(access["official_d0_target_rows"], 0)
        self.assertEqual(access["official_truth_rows_evaluated"], 0)

    def test_prepare_issues_capability_but_does_not_materialize_targets(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-d0-prepare-") as raw:
            output = Path(raw) / "output"
            prepared = d0.prepare(PROFILE, output)
            self.assertFalse(output.exists())
            self.assertIs(
                prepared.capability.provider_kind, contract.ProviderKind.OFFICIAL_D0
            )
            evidence = prepared.provider.evidence()
            self.assertEqual(evidence["official_capabilities_issued"], 1)
            self.assertEqual(evidence["official_d0_target_rows"], 0)
            self.assertEqual(evidence["official_truth_rows_evaluated"], 0)

    def test_profile_mutation_rejects_before_capability_or_target_access(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-d0-profile-") as raw:
            mutated = copy.deepcopy(self.context.profile)
            mutated["one_shot"]["processes"] = 3
            path = Path(raw) / "mutated.json"
            path.write_bytes(d0.canonical_json(mutated))
            with self.assertRaisesRegex(d0.D0RunnerError, "profile size or identity"):
                d0.load_context(path)

    def test_preflight_cli_is_byte_exact_and_target_free(self) -> None:
        outputs: list[bytes] = []
        for _ in range(2):
            completed = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--profile",
                    str(PROFILE),
                    "--preflight",
                ],
                cwd=ROOT,
                check=False,
                capture_output=True,
            )
            self.assertEqual(completed.returncode, 0, completed.stdout.decode())
            self.assertEqual(completed.stderr, b"")
            outputs.append(completed.stdout)
        self.assertEqual(outputs[0], outputs[1])


if __name__ == "__main__":
    unittest.main()
