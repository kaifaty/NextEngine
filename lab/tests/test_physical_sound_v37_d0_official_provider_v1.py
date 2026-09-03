from __future__ import annotations

import sys
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_d0_official_provider_v1 as official
import physical_sound_v37_query_surface_contract_v1 as contract


class OfficialD0ProviderTests(unittest.TestCase):
    def test_pre_access_verifies_all_three_seals_without_opening_values(self) -> None:
        verified = official.verify_pre_access()
        self.assertEqual(
            verified.d0r_seal["seal_sha256"], official.D0R_SEAL_PAYLOAD_SHA256
        )
        self.assertEqual(
            verified.t0_seal["seal_sha256"], official.T0_SEAL_PAYLOAD_SHA256
        )
        self.assertEqual(
            verified.t0_context.e0_seal["seal_sha256"],
            official.E0_SEAL_PAYLOAD_SHA256,
        )
        for key, value in verified.receipt.items():
            if key.endswith(("_rows", "_evaluated")) or key in {
                "network_requests",
                "official_capabilities_issued",
                "protected_signal_values_decoded",
                "real_signal_values_decoded",
            }:
                self.assertEqual(value, 0, key)

    def test_prepared_capability_is_exact_and_still_target_free(self) -> None:
        _verified, capability, provider = official.prepare_official_d0()
        self.assertIs(capability.provider_kind, contract.ProviderKind.OFFICIAL_D0)
        self.assertEqual(capability.namespace, official.OFFICIAL_D0_NAMESPACE)
        self.assertEqual(
            capability.execution_seal_sha256,
            official.D0R_SEAL_PAYLOAD_SHA256,
        )
        evidence = provider.evidence()
        self.assertEqual(evidence["official_capabilities_issued"], 1)
        self.assertEqual(evidence["official_d0_target_rows"], 0)
        self.assertEqual(evidence["official_truth_rows_evaluated"], 0)

    def test_development_before_train_rejects_without_truth_evaluation(self) -> None:
        _verified, capability, provider = official.prepare_official_d0()
        with self.assertRaisesRegex(contract.ContractError, "before train"):
            provider.materialize(contract.RoleKind.DEVELOPMENT, capability)
        self.assertEqual(provider.evidence()["official_truth_rows_evaluated"], 0)

    def test_capability_mutation_rejects_before_truth_evaluation(self) -> None:
        verified, _capability, provider = official.prepare_official_d0()
        wrong = contract.AccessCapabilityV1.official(
            contract.ProviderKind.OFFICIAL_D0,
            "v37-mutated",
            official.D0R_SEAL_PAYLOAD_SHA256,
        )
        with self.assertRaisesRegex(contract.ContractError, "identity mismatch"):
            provider.materialize(contract.RoleKind.TRAIN, wrong)
        self.assertEqual(provider.evidence()["official_truth_rows_evaluated"], 0)
        self.assertEqual(verified.receipt["official_capabilities_issued"], 0)

    def test_bound_file_rejects_identity_drift(self) -> None:
        with self.assertRaisesRegex(official.OfficialProviderError, "bound file drift"):
            official.bound_file(official.D0R_SEAL_PATH, "0" * 64)


if __name__ == "__main__":
    unittest.main()
