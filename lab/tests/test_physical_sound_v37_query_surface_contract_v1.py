from __future__ import annotations

import sys
import unittest
from dataclasses import replace
from pathlib import Path

import numpy as np

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_a0_query_surface_contract_v1 as a0
import physical_sound_v37_query_surface_contract_v1 as contract

NAMES = ("contact-u", "contact-v", "mode-order-normalized")
HASHES = tuple(character * 64 for character in "1234")


class QuerySurfaceContractTests(unittest.TestCase):
    def test_surface_container_has_explicit_counts_and_immutable_storage(self) -> None:
        fields = a0.sample_fields()
        self.assertEqual(fields.field_count, 2)
        self.assertEqual(fields.probe_count, 6)
        self.assertEqual(fields.directed_edge_count, 12)
        self.assertFalse(hasattr(fields, "__len__"))
        with self.assertRaises(TypeError):
            len(fields)  # type: ignore[arg-type]
        for values in (
            fields.field_offsets,
            fields.probe_positions,
            fields.probe_normals,
            fields.probe_area_weights,
            fields.probe_mode_values,
            fields.edge_offsets,
            fields.edge_indices,
        ):
            self.assertFalse(values.flags.writeable)
            with self.assertRaises(ValueError):
                values.flat[0] = 99

    def test_surface_root_is_independent_of_all_input_enumeration(self) -> None:
        canonical = a0.sample_fields()
        permuted = a0.sample_fields(permuted=True)
        self.assertEqual(canonical.field_ids, permuted.field_ids)
        self.assertEqual(canonical.probe_ids, permuted.probe_ids)
        self.assertEqual(canonical.root_sha256, permuted.root_sha256)

    def test_query_root_canonicalizes_rows_fields_and_feature_columns(self) -> None:
        canonical = a0.sample_batch(contract.RoleKind.TRAIN, NAMES)
        permuted = a0.sample_batch(contract.RoleKind.TRAIN, NAMES, permuted=True)
        self.assertEqual(canonical.row_ids, permuted.row_ids)
        self.assertEqual(canonical.context_feature_names, NAMES)
        self.assertEqual(
            canonical.structural_root_sha256, permuted.structural_root_sha256
        )
        np.testing.assert_array_equal(
            canonical.context_features, permuted.context_features
        )
        self.assertIsNone(canonical.targets)
        self.assertIsNone(canonical.target_root_sha256)

    def test_query_rejects_writable_storage_and_noncanonical_direct_names(self) -> None:
        batch = a0.sample_batch(contract.RoleKind.TRAIN, NAMES)
        with self.assertRaisesRegex(contract.ContractError, "immutable finite"):
            replace(
                batch,
                query_positions=np.array(batch.query_positions, copy=True),
            )
        with self.assertRaisesRegex(contract.ContractError, "invalid or noncanonical"):
            replace(batch, context_feature_names=tuple(reversed(NAMES)))

    def test_surface_rejects_offset_csr_and_normal_mutations(self) -> None:
        results = a0.mutation_suite(NAMES)
        for mutation in (
            "asymmetric-csr",
            "cross-field-csr",
            "field-offset-origin",
            "nonunit-probe-normal",
        ):
            self.assertTrue(results[mutation], mutation)

    def test_structural_d0_and_h0_paths_are_complete_and_target_free(self) -> None:
        d0 = a0.complete_structural_trace(contract.ProviderKind.STRUCTURAL_D0, NAMES)
        h0 = a0.complete_structural_trace(contract.ProviderKind.STRUCTURAL_H0, NAMES)
        contract.assert_complete_trace(d0)
        contract.assert_complete_trace(h0)
        self.assertNotEqual(d0.topology_sha256, h0.topology_sha256)
        self.assertEqual(d0.access.target_rows_accessed, 0)
        self.assertEqual(h0.access.target_rows_accessed, 0)
        self.assertEqual(d0.access.structural_rows, 4)
        self.assertEqual(h0.access.structural_rows, 4)

    def test_provider_kinds_have_separate_structural_and_scientific_topologies(
        self,
    ) -> None:
        self.assertNotEqual(
            contract.stages_for_provider(contract.ProviderKind.STRUCTURAL_D0),
            contract.stages_for_provider(contract.ProviderKind.SURROGATE_D0),
        )
        self.assertEqual(
            contract.stages_for_provider(contract.ProviderKind.SURROGATE_D0),
            contract.stages_for_provider(contract.ProviderKind.OFFICIAL_D0),
        )
        self.assertNotEqual(
            contract.expected_topology_sha256(contract.ProviderKind.STRUCTURAL_D0),
            contract.expected_topology_sha256(contract.ProviderKind.SURROGATE_D0),
        )

    def test_capability_namespace_and_pipeline_cannot_cross(self) -> None:
        with self.assertRaisesRegex(contract.ContractError, "structural provider"):
            contract.AccessCapabilityV1.structural(
                contract.ProviderKind.SURROGATE_D0, "structural-invalid"
            )
        with self.assertRaisesRegex(contract.ContractError, "namespace or seal"):
            contract.AccessCapabilityV1.surrogate(
                contract.ProviderKind.SURROGATE_D0, "v37-invalid"
            )
        capability = contract.AccessCapabilityV1.structural(
            contract.ProviderKind.STRUCTURAL_H0, "structural-h0"
        )
        with self.assertRaisesRegex(contract.ContractError, "does not match"):
            contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)

    def test_surrogate_provider_cannot_return_target_free_batch(self) -> None:
        namespace = "discarded-target-free"
        capability = contract.AccessCapabilityV1.surrogate(
            contract.ProviderKind.SURROGATE_D0, namespace
        )
        provider = a0.MemoryStructuralProvider(
            contract.ProviderKind.SURROGATE_D0,
            namespace,
            {contract.RoleKind.TRAIN: a0.sample_batch(contract.RoleKind.TRAIN, NAMES)},
        )
        lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        with self.assertRaisesRegex(contract.ContractError, "target state"):
            lifecycle.materialize_role(
                contract.LifecycleStage.TRAIN_ROLE,
                contract.RoleKind.TRAIN,
                provider,
            )

    def test_terminal_rules_separate_preflight_science_and_fault(self) -> None:
        capability = contract.AccessCapabilityV1.structural(
            contract.ProviderKind.STRUCTURAL_D0, "structural-terminal"
        )
        lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
        with self.assertRaisesRegex(contract.ContractError, "scientific terminal"):
            lifecycle.finish(contract.TerminalDecision.METRIC_REJECT)
        with self.assertRaisesRegex(contract.ContractError, "post-target"):
            contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability).finish(
                contract.TerminalDecision.OWNER_FAULT
            )

    def test_candidate_documents_are_mutually_exclusive(self) -> None:
        owner_hash, profile_hash, terminal_hash, weights_hash = HASHES
        passed = contract.candidate_disposition(
            contract.TerminalDecision.PASS,
            owner_sha256=owner_hash,
            profile_sha256=profile_hash,
            terminal_sha256=terminal_hash,
            candidate_weights_sha256=weights_hash,
        )
        rejected = contract.candidate_disposition(
            contract.TerminalDecision.RESOURCE_REJECT,
            owner_sha256=owner_hash,
            profile_sha256=profile_hash,
            terminal_sha256=terminal_hash,
            candidate_weights_sha256=weights_hash,
        )
        fault = contract.candidate_disposition(
            contract.TerminalDecision.OWNER_FAULT,
            owner_sha256=owner_hash,
            profile_sha256=profile_hash,
            terminal_sha256=terminal_hash,
            candidate_weights_sha256=None,
        )
        self.assertTrue(passed.candidate_bundle_authority)
        self.assertIsNotNone(passed.freeze_document)
        self.assertIsNone(passed.rejected_evidence_document)
        self.assertFalse(rejected.candidate_bundle_authority)
        self.assertIsNone(rejected.freeze_document)
        self.assertIsNotNone(rejected.rejected_evidence_document)
        self.assertIsNone(fault.freeze_document)
        self.assertIsNone(fault.rejected_evidence_document)

    def test_h0_accepts_only_exact_qso_pass_freeze(self) -> None:
        owner_hash, profile_hash, terminal_hash, weights_hash = HASHES
        passed = contract.candidate_disposition(
            contract.TerminalDecision.PASS,
            owner_sha256=owner_hash,
            profile_sha256=profile_hash,
            terminal_sha256=terminal_hash,
            candidate_weights_sha256=weights_hash,
        )
        document = contract.validate_h0_candidate_freeze(
            passed,
            expected_owner_sha256=owner_hash,
            expected_profile_sha256=profile_hash,
            expected_terminal_sha256=terminal_hash,
            expected_weights_sha256=weights_hash,
        )
        self.assertEqual(
            document["status"], contract.CandidateStatus.FROZEN_AFTER_PASS.value
        )
        rejected = contract.candidate_disposition(
            contract.TerminalDecision.METRIC_REJECT,
            owner_sha256=owner_hash,
            profile_sha256=profile_hash,
            terminal_sha256=terminal_hash,
            candidate_weights_sha256=weights_hash,
        )
        with self.assertRaisesRegex(contract.ContractError, "authoritative D0 Pass"):
            contract.validate_h0_candidate_freeze(
                rejected,
                expected_owner_sha256=owner_hash,
                expected_profile_sha256=profile_hash,
                expected_terminal_sha256=terminal_hash,
                expected_weights_sha256=weights_hash,
            )
        self.assertTrue(a0.mutation_suite(NAMES)["legacy-v36-reject-as-h0-freeze"])

    def test_boolean_is_not_an_integer_index(self) -> None:
        with self.assertRaisesRegex(contract.ContractError, "non-integer"):
            contract.frozen_int_vector((0, True, 2))


if __name__ == "__main__":
    unittest.main()
