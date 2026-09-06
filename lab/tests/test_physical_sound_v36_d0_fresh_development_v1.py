from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v36_d0_fresh_development_v1 as d0
import physical_sound_v36_d0_official_provider_v1 as provider
import physical_sound_v36_owner_contract_v1 as contract


class FreshDevelopmentTests(unittest.TestCase):
    def test_preparation_issues_capability_but_opens_zero_targets(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v36-d0-pre-") as temporary:
            output = Path(temporary) / "output"
            prepared = d0.prepare(Path(d0.PROFILE_PATH), output)
            report = d0.pre_access_report(prepared)
            self.assertEqual(report["status"], "PreparedBeforeFreshRoleMaterialization")
            self.assertEqual(report["official_capabilities_issued"], 1)
            self.assertEqual(report["fresh_v36_target_rows"], 0)
            self.assertEqual(report["official_d0_target_rows"], 0)
            self.assertFalse(output.exists())
            self.assertIs(
                prepared.capability.provider_kind,
                contract.ProviderKind.OFFICIAL_D0,
            )

    def test_two_value_free_preparations_are_canonical_exact(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-d0-exact-"
        ) as temporary:
            root = Path(temporary)
            first = d0.canonical_json(
                d0.pre_access_report(d0.prepare(Path(d0.PROFILE_PATH), root / "a"))
            )
            second = d0.canonical_json(
                d0.pre_access_report(d0.prepare(Path(d0.PROFILE_PATH), root / "b"))
            )
            self.assertEqual(first, second)

    def test_profile_mutation_rejects_before_preparation(self) -> None:
        source = Path(d0.PROFILE_PATH).read_bytes()
        profile = json.loads(source)
        profile["one_shot"]["processes"] = 3
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-d0-profile-"
        ) as temporary:
            path = Path(temporary) / "profile.json"
            path.write_bytes(d0.canonical_json(profile))
            with self.assertRaisesRegex(d0.D0RunnerError, "hash-drifted"):
                d0.load_profile(path)

    def test_repository_output_rejects_before_capability_issue(self) -> None:
        output = d0.repository_root() / "forbidden-d0-output"
        with mock.patch.object(provider, "prepare_official_d0") as prepare:
            with self.assertRaisesRegex(
                d0.publisher.PublicationError,
                "outside",
            ):
                d0.prepare(Path(d0.PROFILE_PATH), output)
            prepare.assert_not_called()

    def test_bound_dependency_drift_rejects(self) -> None:
        declaration = d0.load_profile(Path(d0.PROFILE_PATH))[0]["dependencies"][
            "official_provider"
        ]
        with self.assertRaisesRegex(d0.D0RunnerError, "bound file drift"):
            d0.bound_file(declaration["path"], "0" * 64)

    def test_runner_has_one_numeric_owner_call_and_no_truth_formula(self) -> None:
        source = (d0.repository_root() / d0.OWNER_PATH).read_text()
        self.assertEqual(source.count("numeric.execute_owner("), 1)
        self.assertNotIn("def official_targets", source)
        self.assertNotIn("math.tanh", source)

    def test_contract_reject_is_zero_access_and_has_no_artifact_authority(self) -> None:
        report = d0.contract_reject(d0.D0RunnerError("fixture"))
        self.assertEqual(report["decision"], "ContractReject")
        self.assertEqual(report["fresh_v36_target_rows"], 0)
        self.assertEqual(report["official_d0_target_rows"], 0)
        self.assertEqual(report["status"], "D0_PRE_ACCESS_CONTRACT_REJECT")


if __name__ == "__main__":
    unittest.main()
