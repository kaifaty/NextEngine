from __future__ import annotations

import copy
import json
import sys
import unittest
from dataclasses import replace
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import numpy as np
import physical_sound_v36_d0_official_provider_v1 as d0
import physical_sound_v36_e0_full_surrogate_seal_v1 as e0
import physical_sound_v36_owner_contract_v1 as contract


def matrix(values: list[list[float]]) -> contract.FloatMatrix:
    return e0.immutable_matrix(np.asarray(values, dtype=np.float64))


def one_row_batch(*, coordinate_mutation: bool = False) -> contract.RoleBatch:
    ordinal_index = 3
    ordinal = 2.0 * ordinal_index / 9.0 - 1.0
    local = [
        0.1,
        -0.2,
        0.3,
        0.4,
        0.1,
        -0.1,
        0.2,
        0.2,
        -0.4,
        0.15,
        0.25,
        0.1,
        0.3,
        -0.2,
        0.4,
    ]
    decay = [ordinal, local[9], 0.2, -0.4, *local[0:4], 1.0, 0.0, 0.0]
    if coordinate_mutation:
        decay[1] += 0.01
    gain = [ordinal, local[9], 0.2, -0.4, *local[0:4], 1.0, 0.0, *local[4:7]]
    contact = [0.0] * 35
    contact[0] = ordinal
    contact[6:9] = [1.0, 0.0, 0.0]
    contact[9] = local[7]
    contact[10] = local[8]
    contact[11] = local[10]
    contact[32:35] = local[4:7]
    features = e0.feature_matrices(
        {
            "decay": np.asarray([decay], dtype=np.float64),
            "global_gain": np.asarray([gain], dtype=np.float64),
            "contact": np.asarray([contact], dtype=np.float64),
        }
    )
    return contract.RoleBatch(
        contract_schema=contract.CONTRACT_SCHEMA,
        role=contract.RoleKind.TRAIN,
        row_ids=("fixture:mode-03",),
        features=features,
        no_geometry_features=features,
        raw_features=features,
        targets=matrix([[0.0, 0.0, 0.0]]),
        local_keys=matrix([local]),
        geometry_keys=matrix([local[4:7]]),
        contact_keys=matrix([[local[7], local[8]]]),
        local_partition_ids=(
            '["kirchhoff-love-simply-supported-v1","simply-supported-all-edges",3]',
        ),
        local_group_ids=("fixture",),
        case_spans=(contract.CaseSpan("fixture", 0, 1, "train"),),
        strata=(contract.StratumRows("train", (0,)),),
        role_local_geometry_groups=1,
    )


class OfficialD0ProviderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.verified = d0.verify_execution_seal()

    def test_checked_in_seal_verifies_before_capability_or_target_access(self) -> None:
        receipt = self.verified.receipt
        self.assertEqual(receipt["status"], "VerifiedBeforeOfficialCapability")
        self.assertEqual(receipt["official_capabilities_issued"], 0)
        self.assertEqual(receipt["fresh_v36_target_rows"], 0)
        self.assertEqual(receipt["official_d0_target_rows"], 0)
        self.assertEqual(self.verified.seal.forbidden_access_count, 0)

    def test_any_seal_document_mutation_rejects_fixed_identity(self) -> None:
        source = (d0.repository_root() / d0.SEAL_PATH).read_bytes()
        document = json.loads(source)
        mutations = {
            "owner": ("execution_seal", "owner_sha256", "0" * 64),
            "profile": ("execution_seal", "profile_sha256", "1" * 64),
            "environment": ("execution_seal", "environment_sha256", "2" * 64),
            "d0-root": ("execution_seal", "d0_rehearsal_root_sha256", "3" * 64),
            "h0-root": ("execution_seal", "h0_rehearsal_root_sha256", "4" * 64),
            "repeat": ("execution_seal", "repeat_exact", False),
            "forbidden": ("execution_seal", "forbidden_access_count", 1),
            "status": (None, "status", "Mutated"),
        }
        for name, (container, key, value) in mutations.items():
            with self.subTest(name=name):
                mutated = copy.deepcopy(document)
                target = mutated if container is None else mutated[container]
                target[key] = value
                data = e0.canonical_json(mutated)
                with self.assertRaisesRegex(
                    d0.OfficialProviderError, "document identity drift"
                ):
                    d0.parse_execution_seal(data)

    def test_seal_payload_self_hash_rejects_even_with_recomputed_file_hash(
        self,
    ) -> None:
        source = (d0.repository_root() / d0.SEAL_PATH).read_bytes()
        document = json.loads(source)
        document["execution_seal"]["owner_sha256"] = "0" * 64
        data = e0.canonical_json(document)
        with self.assertRaisesRegex(d0.OfficialProviderError, "payload identity drift"):
            d0.parse_execution_seal(data, e0.sha256_bytes(data))

    def test_prepare_issues_only_exact_official_d0_capability(self) -> None:
        prepared, capability, provider = d0.prepare_official_d0()
        self.assertIs(capability.provider_kind, contract.ProviderKind.OFFICIAL_D0)
        self.assertEqual(capability.namespace, d0.OFFICIAL_D0_NAMESPACE)
        self.assertEqual(capability.execution_seal, prepared.seal)
        self.assertEqual(prepared.receipt["official_capabilities_issued"], 1)
        self.assertEqual(prepared.receipt["fresh_v36_target_rows"], 0)
        self.assertIs(provider.kind, contract.ProviderKind.OFFICIAL_D0)

    def test_provider_rejects_wrong_capability_and_role_order_pre_access(self) -> None:
        prepared, capability, provider = d0.prepare_official_d0()
        surrogate = contract.AccessCapability.surrogate(
            contract.ProviderKind.SURROGATE_D0, "discarded-wrong-capability"
        )
        with self.assertRaisesRegex(contract.ContractError, "identity mismatch"):
            provider.materialize(contract.RoleKind.TRAIN, surrogate)
        with self.assertRaisesRegex(contract.ContractError, "before train"):
            provider.materialize(contract.RoleKind.DEVELOPMENT, capability)
        self.assertEqual(prepared.receipt["fresh_v36_target_rows"], 0)

    def test_truth_formula_matches_declared_binary64_math(self) -> None:
        batch = one_row_batch()
        targets = d0.official_targets(batch, self.verified.truth_contract)
        expected = np.asarray(
            [
                0.048532260842948836,
                0.014403567469926817,
                0.014307886972424503,
            ],
            dtype=np.float64,
        )
        self.assertTrue(np.array_equal(targets[0], expected))
        self.assertFalse(targets.flags.writeable)

    def test_truth_formula_rejects_coordinate_drift(self) -> None:
        with self.assertRaisesRegex(d0.OfficialProviderError, "coordinate drift"):
            d0.official_targets(
                one_row_batch(coordinate_mutation=True), self.verified.truth_contract
            )

    def test_truth_contract_mutation_rejects(self) -> None:
        truth = json.loads(self.verified.truth_contract)
        truth["contact"]["bound"] = "0.23"
        with self.assertRaisesRegex(d0.OfficialProviderError, "truth contract drift"):
            d0.official_targets(one_row_batch(), e0.canonical_json(truth))

    def test_official_capability_rejects_different_seal(self) -> None:
        prepared, capability, provider = d0.prepare_official_d0()
        mutated = replace(prepared.seal, owner_sha256="0" * 64)
        wrong = contract.AccessCapability.official(
            contract.ProviderKind.OFFICIAL_D0,
            d0.OFFICIAL_D0_NAMESPACE,
            mutated,
        )
        with self.assertRaisesRegex(contract.ContractError, "identity mismatch"):
            provider.materialize(contract.RoleKind.TRAIN, wrong)
        self.assertNotEqual(capability.execution_seal, wrong.execution_seal)


if __name__ == "__main__":
    unittest.main()
