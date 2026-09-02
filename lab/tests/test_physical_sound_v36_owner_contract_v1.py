from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v36_owner_contract_v1 as contract


def feature_matrices(row_count: int) -> contract.FeatureMatrices:
    return contract.FeatureMatrices(
        decay=contract.frozen_matrix(
            tuple((float(index), 0.25) for index in range(row_count))
        ),
        global_gain=contract.frozen_matrix(
            tuple((float(index), 0.5, -0.5) for index in range(row_count))
        ),
        contact=contract.frozen_matrix(
            tuple((float(index), 0.75, -0.25, 0.125) for index in range(row_count))
        ),
    )


def role_batch(role: contract.RoleKind, prefix: str) -> contract.RoleBatch:
    row_count = 2
    features = feature_matrices(row_count)
    return contract.RoleBatch(
        contract_schema=contract.CONTRACT_SCHEMA,
        role=role,
        row_ids=tuple(f"{prefix}/row-{index}" for index in range(row_count)),
        features=features,
        no_geometry_features=features,
        raw_features=features,
        targets=contract.frozen_matrix(((0.1, 0.2, 0.3), (0.2, 0.3, 0.4))),
        local_keys=contract.frozen_matrix(
            tuple(
                tuple(float(index + column) for column in range(15))
                for index in range(2)
            )
        ),
        geometry_keys=contract.frozen_matrix(((0.1, 0.2, 0.3), (0.2, 0.3, 0.4))),
        contact_keys=contract.frozen_matrix(((0.1, 0.2), (0.2, 0.3))),
        local_partition_ids=(f"{prefix}/partition", f"{prefix}/partition"),
        local_group_ids=(f"{prefix}/group-0", f"{prefix}/group-1"),
        case_spans=(contract.CaseSpan(f"{prefix}/case", 0, 2, "all"),),
        strata=(contract.StratumRows("all", (0, 1)),),
        role_local_geometry_groups=1,
    )


class MemoryProvider:
    def __init__(
        self,
        kind: contract.ProviderKind,
        namespace: str,
        batches: dict[contract.RoleKind, contract.RoleBatch],
    ) -> None:
        self._kind = kind
        self._namespace = namespace
        self._batches = batches
        self.calls: list[contract.RoleKind] = []

    @property
    def kind(self) -> contract.ProviderKind:
        return self._kind

    @property
    def namespace(self) -> str:
        return self._namespace

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapability
    ) -> contract.RoleBatch:
        if (
            capability.provider_kind is not self.kind
            or capability.namespace != self.namespace
        ):
            raise contract.ContractError("test provider capability mismatch")
        self.calls.append(role)
        return self._batches[role]


def complete_trace(
    pipeline: contract.PipelineKind,
    provider: MemoryProvider,
    decision: contract.TerminalDecision = contract.TerminalDecision.PASS,
) -> contract.ExecutionTrace:
    capability = contract.AccessCapability.surrogate(provider.kind, provider.namespace)
    lifecycle = contract.OwnerLifecycle(pipeline, capability)
    for stage in contract.stages_for_pipeline(pipeline):
        role = contract.ROLE_STAGE.get(stage)
        if role is None:
            lifecycle.step(stage)
        else:
            lifecycle.materialize_role(stage, role, provider)
    return lifecycle.finish(decision)


class OwnerContractTests(unittest.TestCase):
    def test_role_batch_has_explicit_count_and_no_length_protocol(self) -> None:
        batch = role_batch(contract.RoleKind.DEVELOPMENT, "discarded-development")
        self.assertEqual(batch.row_count, 2)
        self.assertEqual(batch.case_count, 1)
        self.assertFalse(hasattr(batch, "__len__"))
        with self.assertRaises(TypeError):
            len(batch)  # type: ignore[arg-type]
        with self.assertRaises(ValueError):
            batch.targets[0, 0] = 9.0

    def test_role_batch_rejects_writable_or_misshaped_storage(self) -> None:
        batch = role_batch(contract.RoleKind.TRAIN, "discarded-train")
        writable = np.array(batch.targets, copy=True)
        with self.assertRaisesRegex(contract.ContractError, "immutable"):
            contract.RoleBatch(
                contract_schema=batch.contract_schema,
                role=batch.role,
                row_ids=batch.row_ids,
                features=batch.features,
                no_geometry_features=batch.no_geometry_features,
                raw_features=batch.raw_features,
                targets=writable,
                local_keys=batch.local_keys,
                geometry_keys=batch.geometry_keys,
                contact_keys=batch.contact_keys,
                local_partition_ids=batch.local_partition_ids,
                local_group_ids=batch.local_group_ids,
                case_spans=batch.case_spans,
                strata=batch.strata,
                role_local_geometry_groups=batch.role_local_geometry_groups,
            )
        with self.assertRaisesRegex(contract.ContractError, "targets width"):
            contract.RoleBatch(
                contract_schema=batch.contract_schema,
                role=batch.role,
                row_ids=batch.row_ids,
                features=batch.features,
                no_geometry_features=batch.no_geometry_features,
                raw_features=batch.raw_features,
                targets=contract.frozen_matrix(((0.1, 0.2), (0.2, 0.3))),
                local_keys=batch.local_keys,
                geometry_keys=batch.geometry_keys,
                contact_keys=batch.contact_keys,
                local_partition_ids=batch.local_partition_ids,
                local_group_ids=batch.local_group_ids,
                case_spans=batch.case_spans,
                strata=batch.strata,
                role_local_geometry_groups=batch.role_local_geometry_groups,
            )

    def test_d0_surrogate_uses_one_complete_frozen_topology(self) -> None:
        provider = MemoryProvider(
            contract.ProviderKind.SURROGATE_D0,
            "discarded-d0",
            {
                contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "train"),
                contract.RoleKind.DEVELOPMENT: role_batch(
                    contract.RoleKind.DEVELOPMENT, "development"
                ),
            },
        )
        trace = complete_trace(contract.PipelineKind.D0, provider)
        contract.assert_complete_trace(trace)
        self.assertEqual(
            provider.calls, [contract.RoleKind.TRAIN, contract.RoleKind.DEVELOPMENT]
        )
        self.assertEqual(trace.access.train_target_rows, 2)
        self.assertEqual(trace.access.development_target_rows, 2)
        self.assertEqual(trace.access.method_holdout_target_rows, 0)
        self.assertEqual(trace.access.official_d0_target_rows, 0)
        self.assertEqual(
            trace.topology_sha256,
            contract.expected_topology_sha256(contract.PipelineKind.D0),
        )

    def test_h0_surrogate_uses_one_complete_frozen_topology(self) -> None:
        provider = MemoryProvider(
            contract.ProviderKind.SURROGATE_H0,
            "discarded-h0",
            {
                contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "train"),
                contract.RoleKind.METHOD_HOLDOUT: role_batch(
                    contract.RoleKind.METHOD_HOLDOUT, "holdout"
                ),
            },
        )
        trace = complete_trace(contract.PipelineKind.H0, provider)
        contract.assert_complete_trace(trace)
        self.assertEqual(
            provider.calls, [contract.RoleKind.TRAIN, contract.RoleKind.METHOD_HOLDOUT]
        )
        self.assertEqual(trace.access.method_holdout_target_rows, 2)
        self.assertEqual(trace.access.official_h0_target_rows, 0)
        self.assertEqual(
            trace.topology_sha256,
            contract.expected_topology_sha256(contract.PipelineKind.H0),
        )

    def test_lifecycle_rejects_stage_skip_and_role_stage_bypass(self) -> None:
        capability = contract.AccessCapability.surrogate(
            contract.ProviderKind.SURROGATE_D0, "discarded-order"
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        with self.assertRaisesRegex(contract.ContractError, "stage mismatch"):
            lifecycle.step(contract.LifecycleStage.LOCAL_CROSS_FIT)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        with self.assertRaisesRegex(contract.ContractError, "materialize_role"):
            lifecycle.step(contract.LifecycleStage.TRAIN_ROLE)

    def test_provider_identity_and_returned_role_are_enforced(self) -> None:
        capability = contract.AccessCapability.surrogate(
            contract.ProviderKind.SURROGATE_D0, "discarded-owner"
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        mismatch = MemoryProvider(
            contract.ProviderKind.SURROGATE_D0,
            "discarded-other",
            {contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "train")},
        )
        with self.assertRaisesRegex(contract.ContractError, "provider identity"):
            lifecycle.materialize_role(
                contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, mismatch
            )
        wrong_role = MemoryProvider(
            contract.ProviderKind.SURROGATE_D0,
            "discarded-owner",
            {
                contract.RoleKind.TRAIN: role_batch(
                    contract.RoleKind.DEVELOPMENT, "development"
                )
            },
        )
        with self.assertRaisesRegex(contract.ContractError, "wrong concrete role"):
            lifecycle.materialize_role(
                contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, wrong_role
            )

    def test_terminal_rules_distinguish_pre_and_post_access_faults(self) -> None:
        capability = contract.AccessCapability.surrogate(
            contract.ProviderKind.SURROGATE_D0, "discarded-terminal"
        )
        pre_access = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        rejected = pre_access.finish(contract.TerminalDecision.CONTRACT_REJECT)
        self.assertEqual(rejected.access.target_rows_accessed, 0)
        with self.assertRaisesRegex(contract.ContractError, "post-access only"):
            contract.OwnerLifecycle(contract.PipelineKind.D0, capability).finish(
                contract.TerminalDecision.OWNER_FAULT
            )
        early = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        with self.assertRaisesRegex(contract.ContractError, "complete owner path"):
            early.finish(contract.TerminalDecision.METRIC_REJECT)

        provider = MemoryProvider(
            contract.ProviderKind.SURROGATE_D0,
            "discarded-terminal",
            {contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "train")},
        )
        post_access = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        post_access.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        post_access.materialize_role(
            contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
        )
        fault = post_access.finish(contract.TerminalDecision.OWNER_FAULT)
        self.assertEqual(fault.access.target_rows_accessed, 2)
        with self.assertRaisesRegex(contract.ContractError, "only before"):
            post_access = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
            post_access.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
            post_access.materialize_role(
                contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
            )
            post_access.finish(contract.TerminalDecision.CONTRACT_REJECT)

    def test_official_capability_requires_complete_repeat_exact_seal(self) -> None:
        valid_hash = "a" * 64
        seal = contract.ExecutionSeal(
            contract_schema=contract.CONTRACT_SCHEMA,
            owner_sha256=valid_hash,
            profile_sha256=valid_hash,
            environment_sha256=valid_hash,
            d0_rehearsal_root_sha256=valid_hash,
            h0_rehearsal_root_sha256=valid_hash,
            d0_topology_sha256=contract.expected_topology_sha256(
                contract.PipelineKind.D0
            ),
            h0_topology_sha256=contract.expected_topology_sha256(
                contract.PipelineKind.H0
            ),
            rehearsal_run_count=2,
            repeat_exact=True,
            forbidden_access_count=0,
        )
        capability = contract.AccessCapability.official(
            contract.ProviderKind.OFFICIAL_D0, "v36-fresh-d0", seal
        )
        self.assertIs(capability.execution_seal, seal)
        with self.assertRaisesRegex(contract.ContractError, "two repeat-exact"):
            contract.ExecutionSeal(
                contract_schema=contract.CONTRACT_SCHEMA,
                owner_sha256=valid_hash,
                profile_sha256=valid_hash,
                environment_sha256=valid_hash,
                d0_rehearsal_root_sha256=valid_hash,
                h0_rehearsal_root_sha256=valid_hash,
                d0_topology_sha256=contract.expected_topology_sha256(
                    contract.PipelineKind.D0
                ),
                h0_topology_sha256=contract.expected_topology_sha256(
                    contract.PipelineKind.H0
                ),
                rehearsal_run_count=1,
                repeat_exact=True,
                forbidden_access_count=0,
            )

    def test_provider_pipeline_and_namespace_cannot_cross(self) -> None:
        with self.assertRaisesRegex(contract.ContractError, "discarded namespace"):
            contract.AccessCapability.surrogate(
                contract.ProviderKind.SURROGATE_D0, "v36-not-discarded"
            )
        capability = contract.AccessCapability.surrogate(
            contract.ProviderKind.SURROGATE_H0, "discarded-h0"
        )
        with self.assertRaisesRegex(contract.ContractError, "does not match"):
            contract.OwnerLifecycle(contract.PipelineKind.D0, capability)

    def test_terminal_exit_codes_cover_every_declared_decision(self) -> None:
        codes = {
            decision: contract.terminal_exit_code(decision)
            for decision in contract.TerminalDecision
        }
        self.assertEqual(codes[contract.TerminalDecision.PASS], 0)
        self.assertEqual(codes[contract.TerminalDecision.CONTRACT_REJECT], 2)
        self.assertTrue(
            all(
                code == 1
                for decision, code in codes.items()
                if decision
                not in (
                    contract.TerminalDecision.PASS,
                    contract.TerminalDecision.CONTRACT_REJECT,
                )
            )
        )


if __name__ == "__main__":
    unittest.main()
