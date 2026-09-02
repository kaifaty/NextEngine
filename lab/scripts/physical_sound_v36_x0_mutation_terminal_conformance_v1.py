#!/usr/bin/env python3
"""Prove V36 container, lifecycle, trace, and atomic terminal conformance."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import tempfile
from collections.abc import Callable, Sized
from dataclasses import replace
from pathlib import Path
from typing import cast

import physical_sound_v36_owner_contract_v1 as contract
import physical_sound_v36_terminal_publisher_v1 as publisher

OWNER_PATH = "lab/scripts/physical_sound_v36_x0_mutation_terminal_conformance_v1.py"
CONTRACT_PATH = "lab/scripts/physical_sound_v36_owner_contract_v1.py"
PUBLISHER_PATH = "lab/scripts/physical_sound_v36_terminal_publisher_v1.py"
PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-v36-x0-mutation-terminal-conformance-protocol-2026-09-03.md"
)
A0_RESULT_PATH = (
    "docs/development/physical-sound-v36-a0-typed-owner-contract-result-2026-09-03.md"
)
C0_OWNER_PATH = "lab/scripts/physical_sound_v36_c0_fresh_structural_census_v1.py"
C0_PROFILE_PATH = "lab/profiles/physical-sound-v36-c0-fresh-structural-census.v1.json"
C0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v36-c0-fresh-structural-census-result-2026-09-03.md"
)
V35_B0_RESULT_PATH = (
    "docs/development/physical-sound-v35-b0-local-gate-conformance-result-2026-09-03.md"
)
V35_I0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v35-i0-complete-owner-terminal-proof-result-2026-09-03.md"
)

CONTRACT_SHA256 = "a93d22f4432a29cdb54bcad241b68099f9f0a56c7601e374f5a64a96a20eaac8"
PUBLISHER_SHA256 = "b142b2e11d3e4dcd104520639708cd64e080a8eced0cb9903eb4a16448efae81"
PROTOCOL_SHA256 = "d8c2a92ab6609e52a8a0ddf45b9ea2353744bb91ea5a11cb08abac305238276f"
A0_RESULT_SHA256 = "5233c90adf7990a1679d8c3a07f5a3664b662d0753e8f26e9d5bec31c9dbe7aa"
C0_OWNER_SHA256 = "936dc1401c5dc0a394397997cc861d1e18172c37b605036d35530dc9eb1afe7b"
C0_PROFILE_SHA256 = "55d29722836976ad0fce22d362452564a49ffbbc905722f725001ee6845be48b"
C0_RESULT_SHA256 = "86a6cef80772ee5571f4ade28c8eac2b451bbbec2451f4e33b120ca16b58dc2d"
V35_B0_RESULT_SHA256 = (
    "494494d5ce4bb8317461672e7d57a81967a5ab368930766b72c5a329f52211fc"
)
V35_I0_RESULT_SHA256 = (
    "e2a30c923667957df42c00e6387d622ca77f7001ed57025113f91fbbcb35d2e4"
)

MAXIMUM_OUTPUT_BYTES = 64 * 1024 * 1024
CLAIM = (
    "DISCARDED_TYPED_CONTAINER_LIFECYCLE_TRACE_AND_ATOMIC_TERMINAL_CONFORMANCE_ONLY / "
    "NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_"
    "COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)


class X0ConformanceError(RuntimeError):
    """The X0 owner cannot publish a valid conformance pass."""


class InjectedOwnerError(RuntimeError):
    """A deterministic unexpected owner failure used only by X0."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise X0ConformanceError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def bound_file(path_text: str, expected_sha256: str | None) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise X0ConformanceError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    actual_sha256 = sha256_bytes(data)
    if expected_sha256 is not None and actual_sha256 != expected_sha256:
        raise X0ConformanceError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": actual_sha256}


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
        row_ids=(f"{prefix}/row-0", f"{prefix}/row-1"),
        features=features,
        no_geometry_features=features,
        raw_features=features,
        targets=contract.frozen_matrix(((0.1, 0.2, 0.3), (0.2, 0.3, 0.4))),
        local_keys=contract.frozen_matrix(
            tuple(
                tuple(
                    float(index + column) for column in range(contract.LOCAL_KEY_WIDTH)
                )
                for index in range(row_count)
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

    @property
    def kind(self) -> contract.ProviderKind:
        return self._kind

    @property
    def namespace(self) -> str:
        return self._namespace

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapability
    ) -> contract.RoleBatch:
        if capability.provider_kind is not self.kind:
            raise contract.ContractError("memory provider kind mismatch")
        if capability.namespace != self.namespace:
            raise contract.ContractError("memory provider namespace mismatch")
        return self._batches[role]


def provider_for(pipeline: contract.PipelineKind, label: str) -> MemoryProvider:
    if pipeline is contract.PipelineKind.D0:
        kind = contract.ProviderKind.SURROGATE_D0
        roles = (contract.RoleKind.TRAIN, contract.RoleKind.DEVELOPMENT)
    else:
        kind = contract.ProviderKind.SURROGATE_H0
        roles = (contract.RoleKind.TRAIN, contract.RoleKind.METHOD_HOLDOUT)
    namespace = f"discarded-x0-{label}-{pipeline.value}"
    return MemoryProvider(
        kind,
        namespace,
        {
            role: role_batch(role, f"{label}/{pipeline.value}/{role.value}")
            for role in roles
        },
    )


def complete_trace(
    pipeline: contract.PipelineKind,
    decision: contract.TerminalDecision,
    label: str,
) -> contract.ExecutionTrace:
    provider = provider_for(pipeline, label)
    capability = contract.AccessCapability.surrogate(provider.kind, provider.namespace)
    lifecycle = contract.OwnerLifecycle(pipeline, capability)
    for stage in contract.stages_for_pipeline(pipeline):
        role = contract.ROLE_STAGE.get(stage)
        if role is None:
            lifecycle.step(stage)
        else:
            lifecycle.materialize_role(stage, role, provider)
    trace = lifecycle.finish(decision)
    publisher.validate_trace(trace)
    return trace


def expect_contract_error(action: Callable[[], object], fragment: str) -> bool:
    try:
        action()
    except contract.ContractError as error:
        return fragment in str(error)
    return False


def expect_publication_error(action: Callable[[], object], fragment: str) -> bool:
    try:
        action()
    except publisher.PublicationError as error:
        return fragment in str(error)
    return False


def expect_type_error(action: Callable[[], object]) -> bool:
    try:
        action()
    except TypeError:
        return True
    return False


def lifecycle_after_train(label: str) -> tuple[contract.OwnerLifecycle, MemoryProvider]:
    provider = provider_for(contract.PipelineKind.D0, label)
    capability = contract.AccessCapability.surrogate(provider.kind, provider.namespace)
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )
    return lifecycle, provider


def container_mutations() -> dict[str, bool]:
    batch = role_batch(contract.RoleKind.TRAIN, "container")
    writable_targets = batch.targets.copy()
    one_row_targets = contract.frozen_matrix(((0.1, 0.2, 0.3),))
    wrong_width_targets = contract.frozen_matrix(((0.1, 0.2), (0.2, 0.3)))
    one_row_features = feature_matrices(1)
    return {
        "bad_contract_schema_rejected": expect_contract_error(
            lambda: replace(batch, contract_schema="wrong-schema"), "schema mismatch"
        ),
        "broken_case_span_rejected": expect_contract_error(
            lambda: replace(
                batch,
                case_spans=(contract.CaseSpan("bad/case", 1, 2, "all"),),
            ),
            "cover rows contiguously",
        ),
        "duplicate_row_ids_rejected": expect_contract_error(
            lambda: replace(batch, row_ids=("duplicate", "duplicate")), "unique"
        ),
        "empty_matrix_rejected": expect_contract_error(
            lambda: contract.frozen_matrix(()), "nonempty"
        ),
        "empty_row_id_rejected": expect_contract_error(
            lambda: replace(batch, row_ids=("container/row-0", "")), "must not be empty"
        ),
        "feature_row_count_rejected": expect_contract_error(
            lambda: replace(batch, features=one_row_features), "row count mismatch"
        ),
        "implicit_length_runtime_rejected": expect_type_error(
            lambda: len(cast(Sized, batch))
        ),
        "local_identity_count_rejected": expect_contract_error(
            lambda: replace(batch, local_partition_ids=("only-one",)),
            "identity count mismatch",
        ),
        "nonfinite_matrix_rejected": expect_contract_error(
            lambda: contract.frozen_matrix(((math.nan, 0.0),)), "non-finite"
        ),
        "overlapping_strata_rejected": expect_contract_error(
            lambda: replace(
                batch,
                strata=(
                    contract.StratumRows("all", (0, 1)),
                    contract.StratumRows("overlap", (1,)),
                ),
            ),
            "partition every role row",
        ),
        "target_row_count_rejected": expect_contract_error(
            lambda: replace(batch, targets=one_row_targets), "row count mismatch"
        ),
        "target_width_rejected": expect_contract_error(
            lambda: replace(batch, targets=wrong_width_targets), "width mismatch"
        ),
        "writable_matrix_rejected": expect_contract_error(
            lambda: replace(batch, targets=writable_targets), "immutable"
        ),
    }


def lifecycle_mutations() -> dict[str, bool]:
    def pipeline_provider_mismatch() -> object:
        capability = contract.AccessCapability.surrogate(
            contract.ProviderKind.SURROGATE_H0, "discarded-x0-pipeline-mismatch"
        )
        return contract.OwnerLifecycle(contract.PipelineKind.D0, capability)

    def stage_skip() -> object:
        provider = provider_for(contract.PipelineKind.D0, "stage-skip")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.LOCAL_CROSS_FIT)
        return lifecycle

    def role_stage_bypass() -> object:
        provider = provider_for(contract.PipelineKind.D0, "role-bypass")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        lifecycle.step(contract.LifecycleStage.TRAIN_ROLE)
        return lifecycle

    def stage_role_mismatch() -> object:
        provider = provider_for(contract.PipelineKind.D0, "role-mismatch")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        return lifecycle.materialize_role(
            contract.LifecycleStage.TRAIN_ROLE,
            contract.RoleKind.METHOD_HOLDOUT,
            provider,
        )

    def capability_role_leak() -> object:
        return contract.AccessCapability(
            provider_kind=contract.ProviderKind.SURROGATE_D0,
            namespace="discarded-x0-role-leak",
            allowed_roles=(
                contract.RoleKind.TRAIN,
                contract.RoleKind.METHOD_HOLDOUT,
            ),
            execution_seal=None,
        )

    def provider_identity_mismatch() -> object:
        owner_provider = provider_for(contract.PipelineKind.D0, "provider-owner")
        other_provider = provider_for(contract.PipelineKind.D0, "provider-other")
        capability = contract.AccessCapability.surrogate(
            owner_provider.kind, owner_provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        return lifecycle.materialize_role(
            contract.LifecycleStage.TRAIN_ROLE,
            contract.RoleKind.TRAIN,
            other_provider,
        )

    def wrong_returned_role() -> object:
        namespace = "discarded-x0-wrong-return"
        provider = MemoryProvider(
            contract.ProviderKind.SURROGATE_D0,
            namespace,
            {
                contract.RoleKind.TRAIN: role_batch(
                    contract.RoleKind.DEVELOPMENT, "wrong-return"
                )
            },
        )
        capability = contract.AccessCapability.surrogate(provider.kind, namespace)
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        return lifecycle.materialize_role(
            contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
        )

    def double_open() -> object:
        lifecycle, provider = lifecycle_after_train("double-open")
        return lifecycle.materialize_role(
            contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
        )

    def early_scientific_terminal() -> object:
        provider = provider_for(contract.PipelineKind.D0, "early-scientific")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        return lifecycle.finish(contract.TerminalDecision.METRIC_REJECT)

    def pre_access_owner_fault() -> object:
        provider = provider_for(contract.PipelineKind.D0, "pre-fault")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        return lifecycle.finish(contract.TerminalDecision.OWNER_FAULT)

    def post_access_contract_reject() -> object:
        lifecycle, _ = lifecycle_after_train("post-contract")
        return lifecycle.finish(contract.TerminalDecision.CONTRACT_REJECT)

    def terminal_reuse() -> object:
        provider = provider_for(contract.PipelineKind.D0, "terminal-reuse")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.finish(contract.TerminalDecision.CONTRACT_REJECT)
        return lifecycle.finish(contract.TerminalDecision.CONTRACT_REJECT)

    def after_terminal_step() -> object:
        provider = provider_for(contract.PipelineKind.D0, "after-terminal")
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
        lifecycle.finish(contract.TerminalDecision.CONTRACT_REJECT)
        lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
        return lifecycle

    return {
        "capability_role_leak_rejected": expect_contract_error(
            capability_role_leak, "role set"
        ),
        "double_role_open_rejected": expect_contract_error(double_open, "only once"),
        "early_scientific_terminal_rejected": expect_contract_error(
            early_scientific_terminal, "complete owner path"
        ),
        "execution_after_terminal_rejected": expect_contract_error(
            after_terminal_step, "cannot advance"
        ),
        "pipeline_provider_mismatch_rejected": expect_contract_error(
            pipeline_provider_mismatch, "does not match"
        ),
        "post_access_contract_reject_rejected": expect_contract_error(
            post_access_contract_reject, "only before target access"
        ),
        "pre_access_owner_fault_rejected": expect_contract_error(
            pre_access_owner_fault, "post-access only"
        ),
        "provider_identity_mismatch_rejected": expect_contract_error(
            provider_identity_mismatch, "provider identity"
        ),
        "role_stage_bypass_rejected": expect_contract_error(
            role_stage_bypass, "materialize_role"
        ),
        "stage_role_mismatch_rejected": expect_contract_error(
            stage_role_mismatch, "do not match"
        ),
        "stage_skip_rejected": expect_contract_error(stage_skip, "stage mismatch"),
        "terminal_reuse_rejected": expect_contract_error(
            terminal_reuse, "already published"
        ),
        "wrong_returned_role_rejected": expect_contract_error(
            wrong_returned_role, "wrong concrete role"
        ),
    }


def trace_mutations() -> dict[str, bool]:
    trace = complete_trace(
        contract.PipelineKind.D0, contract.TerminalDecision.PASS, "trace-base"
    )
    first = trace.events[0]
    ordinal_trace = replace(
        trace, events=(replace(first, ordinal=7), *trace.events[1:])
    )
    callable_trace = replace(
        trace,
        events=(replace(first, callable_id="mutated.callable"), *trace.events[1:]),
    )
    missing_terminal_trace = replace(trace, events=trace.events[:-1])
    reordered_trace = replace(
        trace,
        events=(
            replace(trace.events[1], ordinal=0),
            replace(trace.events[0], ordinal=1),
            *trace.events[2:],
        ),
    )
    forbidden_trace = replace(trace, access=replace(trace.access, network_requests=1))
    role_leak_trace = replace(
        trace, access=replace(trace.access, method_holdout_target_rows=1)
    )
    official_leak_trace = replace(
        trace, access=replace(trace.access, official_d0_target_rows=1)
    )
    bad_call_count_trace = replace(
        trace, access=replace(trace.access, provider_calls=1)
    )
    pipeline_trace = replace(trace, pipeline=contract.PipelineKind.H0)
    contract_reject_after_access = replace(
        trace, terminal=contract.TerminalDecision.CONTRACT_REJECT
    )
    owner_fault_before_access = contract.ExecutionTrace(
        pipeline=contract.PipelineKind.D0,
        provider_kind=contract.ProviderKind.SURROGATE_D0,
        events=(
            contract.TraceEvent(
                ordinal=0,
                stage=contract.LifecycleStage.TERMINAL_PUBLICATION,
                callable_id=contract.STAGE_CALLABLES[
                    contract.LifecycleStage.TERMINAL_PUBLICATION
                ],
            ),
        ),
        terminal=contract.TerminalDecision.OWNER_FAULT,
        access=contract.AccessLedger(),
    )
    negative_access_trace = replace(
        trace, access=replace(trace.access, train_target_rows=-1)
    )
    return {
        "callable_and_topology_mutation_rejected": expect_publication_error(
            lambda: publisher.validate_trace(callable_trace), "callable mutation"
        ),
        "contract_reject_after_access_rejected": expect_publication_error(
            lambda: publisher.validate_trace(contract_reject_after_access),
            "zero target access",
        ),
        "forbidden_access_rejected": expect_publication_error(
            lambda: publisher.validate_trace(forbidden_trace), "forbidden access"
        ),
        "missing_terminal_event_rejected": expect_publication_error(
            lambda: publisher.validate_trace(missing_terminal_trace), "lacks terminal"
        ),
        "negative_access_rejected": expect_publication_error(
            lambda: publisher.validate_trace(negative_access_trace), "negative access"
        ),
        "ordinal_mutation_rejected": expect_publication_error(
            lambda: publisher.validate_trace(ordinal_trace), "ordinal mutation"
        ),
        "owner_fault_before_access_rejected": expect_publication_error(
            lambda: publisher.validate_trace(owner_fault_before_access),
            "post-access trace",
        ),
        "pipeline_provider_mutation_rejected": expect_publication_error(
            lambda: publisher.validate_trace(pipeline_trace), "pipeline/provider"
        ),
        "provider_call_count_mutation_rejected": expect_publication_error(
            lambda: publisher.validate_trace(bad_call_count_trace), "two role opens"
        ),
        "role_leak_receipt_rejected": expect_publication_error(
            lambda: publisher.validate_trace(role_leak_trace), "method-holdout"
        ),
        "stage_reorder_rejected": expect_publication_error(
            lambda: publisher.validate_trace(reordered_trace), "stage sequence mutation"
        ),
        "surrogate_official_receipt_rejected": expect_publication_error(
            lambda: publisher.validate_trace(official_leak_trace),
            "official role access",
        ),
    }


def mutation_suite() -> dict[str, object]:
    containers = container_mutations()
    lifecycle = lifecycle_mutations()
    traces = trace_mutations()
    return {
        "all_pass": all((*containers.values(), *lifecycle.values(), *traces.values())),
        "container": containers,
        "lifecycle": lifecycle,
        "trace": traces,
        "total_mutations": len(containers) + len(lifecycle) + len(traces),
    }


def receipt_record(receipt: publisher.PublicationReceipt) -> dict[str, object]:
    return {
        "file_count": receipt.file_count,
        "terminal_sha256": receipt.terminal_sha256,
        "total_bytes": receipt.total_bytes,
        "tree_sha256": receipt.tree_sha256,
    }


def publish_discarded_terminal(
    output: Path,
    pipeline: contract.PipelineKind,
    decision: contract.TerminalDecision,
    label: str,
    *,
    failpoint: publisher.PublicationFailpoint | None = None,
) -> publisher.PublicationReceipt:
    trace = complete_trace(pipeline, decision, label)
    payload = canonical_json(
        {
            "decision": decision.value,
            "pipeline": pipeline.value,
            "schema": "nextengine.experimental-physical-sound-v36-x0-probe.v1",
        }
    )
    return publisher.publish_terminal(
        output,
        repository_root(),
        trace,
        CLAIM,
        {"evidence.json": payload},
        MAXIMUM_OUTPUT_BYTES,
        failpoint=failpoint,
    )


def classify_unexpected_exception(
    output: Path,
    *,
    after_access: bool,
) -> tuple[contract.ExecutionTrace, publisher.PublicationReceipt | None]:
    provider = provider_for(contract.PipelineKind.D0, "unexpected")
    capability = contract.AccessCapability.surrogate(provider.kind, provider.namespace)
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    try:
        if after_access:
            lifecycle.materialize_role(
                contract.LifecycleStage.TRAIN_ROLE,
                contract.RoleKind.TRAIN,
                provider,
            )
        raise InjectedOwnerError("deterministic X0 unexpected exception")
    except InjectedOwnerError:
        decision = publisher.unexpected_exception_decision(lifecycle.access)
        trace = lifecycle.finish(decision)
        publisher.validate_trace(trace)
        if decision is contract.TerminalDecision.CONTRACT_REJECT:
            return trace, None
        receipt = publisher.publish_terminal(
            output,
            repository_root(),
            trace,
            CLAIM,
            {
                "owner-fault.json": canonical_json(
                    {
                        "error_class": "InjectedOwnerError",
                        "schema": (
                            "nextengine.experimental-physical-sound-v36-x0-owner-fault.v1"
                        ),
                    }
                )
            },
            MAXIMUM_OUTPUT_BYTES,
        )
        return trace, receipt


def publication_suite() -> dict[str, object]:
    scientific = (
        contract.TerminalDecision.PASS,
        contract.TerminalDecision.METRIC_REJECT,
        contract.TerminalDecision.HARD_GATE_REJECT,
        contract.TerminalDecision.RESOURCE_REJECT,
    )
    with tempfile.TemporaryDirectory(
        prefix="nextengine-v36-x0-publication-"
    ) as temporary:
        root = Path(temporary)
        complete: dict[str, object] = {}
        for pipeline in contract.PipelineKind:
            for decision in scientific:
                key = f"{pipeline.value}/{decision.value}"
                receipt = publish_discarded_terminal(
                    root / f"complete-{pipeline.value}-{decision.value}",
                    pipeline,
                    decision,
                    f"complete-{pipeline.value}-{decision.value}",
                )
                complete[key] = receipt_record(receipt)

        failpoints: dict[str, bool] = {}
        for failpoint in publisher.PublicationFailpoint:
            output = root / f"failpoint-{failpoint.value}"
            staging = output.parent / (
                f".{output.name}.nextengine-v36-terminal-staging"
            )
            try:
                publish_discarded_terminal(
                    output,
                    contract.PipelineKind.D0,
                    contract.TerminalDecision.METRIC_REJECT,
                    f"failpoint-{failpoint.value}",
                    failpoint=failpoint,
                )
            except publisher.PublicationError as error:
                failed = "injected publication failure" in str(error)
            else:
                failed = False
            failpoints[failpoint.value] = (
                failed and not output.exists() and not staging.exists()
            )

        trace = complete_trace(
            contract.PipelineKind.D0,
            contract.TerminalDecision.PASS,
            "publication-mutations",
        )
        occupied = root / "occupied"
        occupied.mkdir()
        (occupied / "sentinel").write_text("keep")
        symlink_target = root / "symlink-target"
        symlink_target.mkdir()
        symlink_output = root / "symlink-output"
        symlink_output.symlink_to(symlink_target, target_is_directory=True)
        stale_output = root / "stale-output"
        stale_staging = root / ".stale-output.nextengine-v36-terminal-staging"
        stale_staging.mkdir()
        contract_reject_lifecycle = contract.OwnerLifecycle(
            contract.PipelineKind.D0,
            contract.AccessCapability.surrogate(
                contract.ProviderKind.SURROGATE_D0,
                "discarded-x0-contract-reject-publication",
            ),
        )
        contract_reject_trace = contract_reject_lifecycle.finish(
            contract.TerminalDecision.CONTRACT_REJECT
        )
        checks = {
            "byte_budget_overflow_rejected": expect_publication_error(
                lambda: publisher.publish_terminal(
                    root / "budget",
                    repository_root(),
                    trace,
                    CLAIM,
                    {"payload.json": b"payload"},
                    1,
                ),
                "byte budget",
            ),
            "contract_reject_output_absent": expect_publication_error(
                lambda: publisher.publish_terminal(
                    root / "contract-reject",
                    repository_root(),
                    contract_reject_trace,
                    CLAIM,
                    {"payload.json": b"payload"},
                    MAXIMUM_OUTPUT_BYTES,
                ),
                "must leave output absent",
            )
            and not (root / "contract-reject").exists(),
            "invalid_payload_leaf_rejected": expect_publication_error(
                lambda: publisher.publish_terminal(
                    root / "invalid-leaf",
                    repository_root(),
                    trace,
                    CLAIM,
                    {"../escape.json": b"payload"},
                    MAXIMUM_OUTPUT_BYTES,
                ),
                "nonterminal leaf",
            ),
            "occupied_output_preserved": expect_publication_error(
                lambda: publisher.publish_terminal(
                    occupied,
                    repository_root(),
                    trace,
                    CLAIM,
                    {"payload.json": b"payload"},
                    MAXIMUM_OUTPUT_BYTES,
                ),
                "fresh non-symlink",
            )
            and (occupied / "sentinel").read_text() == "keep",
            "repository_output_rejected": expect_publication_error(
                lambda: publisher.publish_terminal(
                    repository_root() / ".never-created-v36-x0-output",
                    repository_root(),
                    trace,
                    CLAIM,
                    {"payload.json": b"payload"},
                    MAXIMUM_OUTPUT_BYTES,
                ),
                "outside the forbidden root",
            ),
            "stale_staging_rejected": expect_publication_error(
                lambda: publisher.publish_terminal(
                    stale_output,
                    repository_root(),
                    trace,
                    CLAIM,
                    {"payload.json": b"payload"},
                    MAXIMUM_OUTPUT_BYTES,
                ),
                "staging path already exists",
            ),
            "symlink_output_rejected": expect_publication_error(
                lambda: publisher.publish_terminal(
                    symlink_output,
                    repository_root(),
                    trace,
                    CLAIM,
                    {"payload.json": b"payload"},
                    MAXIMUM_OUTPUT_BYTES,
                ),
                "fresh non-symlink",
            ),
        }

        pre_output = root / "pre-access-exception"
        pre_trace, pre_receipt = classify_unexpected_exception(
            pre_output, after_access=False
        )
        post_output = root / "post-access-exception"
        post_trace, post_receipt = classify_unexpected_exception(
            post_output, after_access=True
        )
        exceptions: dict[str, object] = {
            "post_access": {
                "decision": post_trace.terminal.value,
                "published": post_output.is_dir(),
                "receipt": (
                    receipt_record(post_receipt) if post_receipt is not None else None
                ),
            },
            "pre_access": {
                "decision": pre_trace.terminal.value,
                "output_absent": not pre_output.exists(),
                "receipt_absent": pre_receipt is None,
            },
        }
        return {
            "all_pass": (
                len(complete) == 8
                and all(failpoints.values())
                and all(checks.values())
                and pre_trace.terminal is contract.TerminalDecision.CONTRACT_REJECT
                and pre_receipt is None
                and not pre_output.exists()
                and post_trace.terminal is contract.TerminalDecision.OWNER_FAULT
                and post_receipt is not None
                and post_output.is_dir()
            ),
            "checks": checks,
            "complete_scientific_terminals": complete,
            "exception_conversion": exceptions,
            "failpoints": failpoints,
        }


def run_conformance() -> tuple[
    dict[str, object], dict[str, object], dict[str, object], dict[str, object]
]:
    bindings = {
        "a0_result": bound_file(A0_RESULT_PATH, A0_RESULT_SHA256),
        "c0_owner": bound_file(C0_OWNER_PATH, C0_OWNER_SHA256),
        "c0_profile": bound_file(C0_PROFILE_PATH, C0_PROFILE_SHA256),
        "c0_result": bound_file(C0_RESULT_PATH, C0_RESULT_SHA256),
        "contract": bound_file(CONTRACT_PATH, CONTRACT_SHA256),
        "protocol": bound_file(PROTOCOL_PATH, PROTOCOL_SHA256),
        "publisher": bound_file(PUBLISHER_PATH, PUBLISHER_SHA256),
        "v35_b0_result": bound_file(V35_B0_RESULT_PATH, V35_B0_RESULT_SHA256),
        "v35_i0_result": bound_file(V35_I0_RESULT_PATH, V35_I0_RESULT_SHA256),
    }
    mutations = mutation_suite()
    publication = publication_suite()
    if mutations["all_pass"] is not True:
        raise X0ConformanceError("mutation matrix failed")
    if publication["all_pass"] is not True:
        raise X0ConformanceError("publication matrix failed")
    contract_data: dict[str, object] = {
        "claim": CLAIM,
        "container_schema": contract.CONTRACT_SCHEMA,
        "exception_rule": {
            "post_access": contract.TerminalDecision.OWNER_FAULT.value,
            "pre_access": contract.TerminalDecision.CONTRACT_REJECT.value,
        },
        "negative_type_fixture": (
            "lab/tests/typecheck/physical_sound_v36_invalid_implicit_len.py"
        ),
        "pipelines": {
            pipeline.value: {
                "provider": provider_for(pipeline, "contract").kind.value,
                "stages": [
                    stage.value for stage in contract.stages_for_pipeline(pipeline)
                ],
                "topology_sha256": contract.expected_topology_sha256(pipeline),
            }
            for pipeline in contract.PipelineKind
        },
        "publication_failpoints": [
            point.value for point in publisher.PublicationFailpoint
        ],
        "publisher_schema": publisher.PUBLISHER_SCHEMA,
        "schema": "nextengine.experimental-physical-sound-v36-x0-contract.v1",
        "scientific_terminals": [
            decision.value
            for decision in (
                contract.TerminalDecision.PASS,
                contract.TerminalDecision.METRIC_REJECT,
                contract.TerminalDecision.HARD_GATE_REJECT,
                contract.TerminalDecision.RESOURCE_REJECT,
            )
        ],
    }
    evidence: dict[str, object] = {
        "bindings": bindings,
        "claim": CLAIM,
        "mypy_profile": {
            "negative_fixture": (
                "lab/tests/typecheck/physical_sound_v36_invalid_implicit_len.py"
            ),
            "numpy": "2.5.2",
            "options": [
                "--strict",
                "--disallow-any-explicit",
                "--disallow-any-generics",
                "--warn-unused-ignores",
            ],
            "tool": "mypy",
            "version": "2.0.0",
        },
        "official_access": {
            "fresh_v36_target_rows": 0,
            "model_parameters_initialized": 0,
            "network_requests": 0,
            "official_capabilities_issued": 0,
            "prior_generation_values_read": 0,
            "protected_signal_values_decoded": 0,
            "real_signal_values_decoded": 0,
        },
        "owner": bound_file(OWNER_PATH, None),
        "schema": "nextengine.experimental-physical-sound-v36-x0-evidence.v1",
    }
    return contract_data, mutations, publication, evidence


def run(output: Path) -> dict[str, object]:
    contract_data, mutations, publication, evidence = run_conformance()
    initial = {
        "contract.json": canonical_json(contract_data),
        "evidence.json": canonical_json(evidence),
        "mutations.json": canonical_json(mutations),
        "publication.json": canonical_json(publication),
    }
    report: dict[str, object] = {
        "artifacts": {
            name: {"bytes": len(data), "sha256": sha256_bytes(data)}
            for name, data in sorted(initial.items())
        },
        "claim": CLAIM,
        "decision": "Pass",
        "next_authorized_stage": "V36-E0-full-surrogate-rehearsal-and-execution-seal",
        "official_access": evidence["official_access"],
        "schema": "nextengine.experimental-physical-sound-v36-x0-report.v1",
        "status": "X0_MUTATION_AND_TERMINAL_CONFORMANCE_PASS",
    }
    payloads = {**initial, "report.json": canonical_json(report)}
    top_level_trace = complete_trace(
        contract.PipelineKind.D0,
        contract.TerminalDecision.PASS,
        "top-level-publication",
    )
    publisher.publish_terminal(
        output,
        repository_root(),
        top_level_trace,
        CLAIM,
        payloads,
        MAXIMUM_OUTPUT_BYTES,
    )
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    output = arguments.output.absolute()
    report = run(output)
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
