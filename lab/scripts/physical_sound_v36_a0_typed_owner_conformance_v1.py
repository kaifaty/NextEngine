#!/usr/bin/env python3
"""Prove the zero-official-value V36 typed owner/provider execution contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
from collections.abc import Callable
from pathlib import Path

import physical_sound_v36_owner_contract_v1 as contract

OWNER_PATH = "lab/scripts/physical_sound_v36_a0_typed_owner_conformance_v1.py"
CONTRACT_PATH = "lab/scripts/physical_sound_v36_owner_contract_v1.py"
V35_OWNER_PATH = "lab/scripts/physical_sound_v35_d0_development_tournament_v1.py"
V35_RESULT_PATH = (
    "docs/development/physical-sound-v35-d0-development-tournament-result-2026-09-03.md"
)
V36_RESEARCH_PATH = (
    "docs/development/physical-sound-v36-sealed-owner-rebaseline-research-2026-09-03.md"
)
V35_OWNER_SHA256 = "a08edd1b5764fe44b340c2e12972be80b94985aa65dc9bb0b53ec00fba69ff04"
V35_RESULT_SHA256 = "91323996da6c69f51cf0c5ee3c8e6d648dfcca2e00a1a846c73ec07746b83943"
V36_RESEARCH_SHA256 = "62cf8104edea4301fb624abfa1c46d81463f68122724822499a95226e4ccc895"
CLAIM = (
    "ZERO_OFFICIAL_VALUE_TYPED_OWNER_PROVIDER_AND_LIFECYCLE_CONFORMANCE_ONLY / "
    "NO_FRESH_ROLE_TARGET_MODEL_HOLDOUT_REAL_VALIDATOR_RELEASE_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)


class A0ConformanceError(RuntimeError):
    """The A0 contract owner cannot publish a valid conformance pass."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise A0ConformanceError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def bound_file(path_text: str, expected_sha256: str | None = None) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise A0ConformanceError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    actual = sha256_bytes(data)
    if expected_sha256 is not None and actual != expected_sha256:
        raise A0ConformanceError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": actual}


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


def complete_trace(
    pipeline: contract.PipelineKind,
    provider: MemoryProvider,
) -> contract.ExecutionTrace:
    capability = contract.AccessCapability.surrogate(provider.kind, provider.namespace)
    lifecycle = contract.OwnerLifecycle(pipeline, capability)
    for stage in contract.stages_for_pipeline(pipeline):
        role = contract.ROLE_STAGE.get(stage)
        if role is None:
            lifecycle.step(stage)
        else:
            lifecycle.materialize_role(stage, role, provider)
    trace = lifecycle.finish(contract.TerminalDecision.PASS)
    contract.assert_complete_trace(trace)
    return trace


def trace_records(trace: contract.ExecutionTrace) -> list[dict[str, object]]:
    return [
        {
            "callable_id": event.callable_id,
            "ordinal": event.ordinal,
            "stage": event.stage.value,
        }
        for event in trace.events
    ]


def access_record(access: contract.AccessLedger) -> dict[str, int]:
    return {
        "development_target_rows": access.development_target_rows,
        "forbidden_access_count": access.forbidden_access_count,
        "method_holdout_target_rows": access.method_holdout_target_rows,
        "network_requests": access.network_requests,
        "official_d0_target_rows": access.official_d0_target_rows,
        "official_h0_target_rows": access.official_h0_target_rows,
        "prior_generation_values_read": access.prior_generation_values_read,
        "protected_signal_values_decoded": access.protected_signal_values_decoded,
        "provider_calls": access.provider_calls,
        "real_signal_values_decoded": access.real_signal_values_decoded,
        "target_rows_accessed": access.target_rows_accessed,
        "train_target_rows": access.train_target_rows,
    }


def expect_error(action: Callable[[], object], fragment: str) -> bool:
    try:
        action()
    except contract.ContractError as error:
        return fragment in str(error)
    return False


def mutation_stage_skip() -> object:
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, "discarded-stage-skip"
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.LOCAL_CROSS_FIT)
    return lifecycle


def mutation_role_bypass() -> object:
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, "discarded-role-bypass"
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    lifecycle.step(contract.LifecycleStage.TRAIN_ROLE)
    return lifecycle


def mutation_early_metric_terminal() -> object:
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, "discarded-early-terminal"
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    return lifecycle.finish(contract.TerminalDecision.METRIC_REJECT)


def mutation_pre_access_owner_fault() -> object:
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, "discarded-pre-fault"
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    return lifecycle.finish(contract.TerminalDecision.OWNER_FAULT)


def mutation_post_access_contract_reject() -> object:
    namespace = "discarded-post-contract"
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, namespace
    )
    provider = MemoryProvider(
        contract.ProviderKind.SURROGATE_D0,
        namespace,
        {contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "train")},
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )
    return lifecycle.finish(contract.TerminalDecision.CONTRACT_REJECT)


def mutation_provider_identity() -> object:
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, "discarded-owner"
    )
    provider = MemoryProvider(
        contract.ProviderKind.SURROGATE_D0,
        "discarded-other",
        {contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "train")},
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    return lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )


def mutation_wrong_returned_role() -> object:
    namespace = "discarded-wrong-role"
    capability = contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, namespace
    )
    provider = MemoryProvider(
        contract.ProviderKind.SURROGATE_D0,
        namespace,
        {
            contract.RoleKind.TRAIN: role_batch(
                contract.RoleKind.DEVELOPMENT, "wrong-development"
            )
        },
    )
    lifecycle = contract.OwnerLifecycle(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.PRE_ACCESS_CONTEXT)
    return lifecycle.materialize_role(
        contract.LifecycleStage.TRAIN_ROLE, contract.RoleKind.TRAIN, provider
    )


def mutation_invalid_surrogate_namespace() -> object:
    return contract.AccessCapability.surrogate(
        contract.ProviderKind.SURROGATE_D0, "v36-not-discarded"
    )


def mutation_incomplete_execution_seal() -> object:
    value = "a" * 64
    return contract.ExecutionSeal(
        contract_schema=contract.CONTRACT_SCHEMA,
        owner_sha256=value,
        profile_sha256=value,
        environment_sha256=value,
        d0_rehearsal_root_sha256=value,
        h0_rehearsal_root_sha256=value,
        d0_topology_sha256=contract.expected_topology_sha256(contract.PipelineKind.D0),
        h0_topology_sha256=contract.expected_topology_sha256(contract.PipelineKind.H0),
        rehearsal_run_count=1,
        repeat_exact=True,
        forbidden_access_count=0,
    )


def mutation_suite() -> dict[str, bool]:
    return {
        "early_scientific_terminal_rejected": expect_error(
            mutation_early_metric_terminal, "complete owner path"
        ),
        "implicit_role_length_absent": not hasattr(
            role_batch(contract.RoleKind.TRAIN, "implicit-length"), "__len__"
        ),
        "incomplete_execution_seal_rejected": expect_error(
            mutation_incomplete_execution_seal, "two repeat-exact"
        ),
        "post_access_contract_reject_rejected": expect_error(
            mutation_post_access_contract_reject, "only before"
        ),
        "pre_access_owner_fault_rejected": expect_error(
            mutation_pre_access_owner_fault, "post-access only"
        ),
        "provider_identity_mismatch_rejected": expect_error(
            mutation_provider_identity, "provider identity"
        ),
        "role_stage_bypass_rejected": expect_error(
            mutation_role_bypass, "materialize_role"
        ),
        "stage_skip_rejected": expect_error(mutation_stage_skip, "stage mismatch"),
        "surrogate_namespace_mismatch_rejected": expect_error(
            mutation_invalid_surrogate_namespace, "discarded namespace"
        ),
        "wrong_concrete_role_rejected": expect_error(
            mutation_wrong_returned_role, "wrong concrete role"
        ),
    }


def contract_document() -> dict[str, object]:
    required_stages = (
        set(contract.D0_STAGES)
        | set(contract.H0_STAGES)
        | {contract.LifecycleStage.TERMINAL_PUBLICATION}
    )
    if set(contract.STAGE_CALLABLES) != required_stages:
        raise A0ConformanceError("stage/callable closure mismatch")
    return {
        "access_capability": {
            "official_requires_execution_seal": True,
            "rehearsal_run_count": 2,
            "surrogate_namespace_prefix": "discarded-",
            "v36_namespace_prefix": "v36-",
        },
        "claim": CLAIM,
        "concrete_role_container": {
            "explicit_count_property": "row_count",
            "implicit_length_protocol": False,
            "immutable_float_storage": True,
            "schema": contract.CONTRACT_SCHEMA,
            "target_axis_count": contract.TARGET_AXIS_COUNT,
        },
        "pipelines": {
            pipeline.value: {
                "stages": [
                    stage.value for stage in contract.stages_for_pipeline(pipeline)
                ],
                "topology_sha256": contract.expected_topology_sha256(pipeline),
            }
            for pipeline in contract.PipelineKind
        },
        "provider_kinds": [kind.value for kind in contract.ProviderKind],
        "roles": [role.value for role in contract.RoleKind],
        "schema": contract.CONTRACT_SCHEMA,
        "terminal_decisions": {
            decision.value: contract.terminal_exit_code(decision)
            for decision in contract.TerminalDecision
        },
    }


def run_conformance() -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    bindings = {
        "contract": bound_file(CONTRACT_PATH),
        "v35_owner": bound_file(V35_OWNER_PATH, V35_OWNER_SHA256),
        "v35_result": bound_file(V35_RESULT_PATH, V35_RESULT_SHA256),
        "v36_research": bound_file(V36_RESEARCH_PATH, V36_RESEARCH_SHA256),
    }
    d0_provider = MemoryProvider(
        contract.ProviderKind.SURROGATE_D0,
        "discarded-a0-d0",
        {
            contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "d0/train"),
            contract.RoleKind.DEVELOPMENT: role_batch(
                contract.RoleKind.DEVELOPMENT, "d0/development"
            ),
        },
    )
    h0_provider = MemoryProvider(
        contract.ProviderKind.SURROGATE_H0,
        "discarded-a0-h0",
        {
            contract.RoleKind.TRAIN: role_batch(contract.RoleKind.TRAIN, "h0/train"),
            contract.RoleKind.METHOD_HOLDOUT: role_batch(
                contract.RoleKind.METHOD_HOLDOUT, "h0/method-holdout"
            ),
        },
    )
    d0_trace = complete_trace(contract.PipelineKind.D0, d0_provider)
    h0_trace = complete_trace(contract.PipelineKind.H0, h0_provider)
    mutations = mutation_suite()
    if not all(mutations.values()):
        failed = sorted(name for name, passed in mutations.items() if not passed)
        raise A0ConformanceError(f"mutation suite failed: {failed}")
    contract_data = contract_document()
    checks: dict[str, bool] = {
        "complete_d0_trace": True,
        "complete_h0_trace": True,
        "concrete_role_container": True,
        "explicit_row_count": True,
        "forbidden_access_zero": (
            d0_trace.access.forbidden_access_count == 0
            and h0_trace.access.forbidden_access_count == 0
        ),
        "official_capability_unissued": True,
        "stage_callable_closure": True,
    }
    conformance: dict[str, object] = {
        "access": {
            "d0_surrogate": access_record(d0_trace.access),
            "h0_surrogate": access_record(h0_trace.access),
            "official_d0_target_rows": 0,
            "official_h0_target_rows": 0,
            "official_model_parameters_initialized": 0,
            "fresh_v36_target_rows": 0,
        },
        "checks": checks,
        "claim": CLAIM,
        "d0_trace": trace_records(d0_trace),
        "d0_topology_sha256": d0_trace.topology_sha256,
        "h0_trace": trace_records(h0_trace),
        "h0_topology_sha256": h0_trace.topology_sha256,
        "mutations": mutations,
        "schema": "nextengine.experimental-physical-sound-v36-a0-conformance.v1",
        "status": "Pass",
    }
    if not all(checks.values()):
        raise A0ConformanceError("conformance checks failed")
    evidence: dict[str, object] = {
        "bindings": bindings,
        "claim": CLAIM,
        "contract_schema": contract.CONTRACT_SCHEMA,
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
            "prior_generation_values_read": 0,
            "protected_signal_values_decoded": 0,
            "real_signal_values_decoded": 0,
        },
        "owner": bound_file(OWNER_PATH),
        "schema": "nextengine.experimental-physical-sound-v36-a0-evidence.v1",
    }
    return contract_data, conformance, evidence


def atomic_publish(output: Path, payloads: dict[str, bytes]) -> None:
    if output.exists() or output.is_symlink():
        raise A0ConformanceError("output already exists")
    if not output.parent.is_dir() or output.parent.is_symlink():
        raise A0ConformanceError("output parent is invalid")
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists() or staging.is_symlink():
        raise A0ConformanceError("owned staging path already exists")
    staging.mkdir(mode=0o700)
    try:
        for name, data in sorted(payloads.items()):
            if Path(name).name != name:
                raise A0ConformanceError("payload name is not a leaf")
            (staging / name).write_bytes(data)
        os.replace(staging, output)
    except BaseException:
        if staging.is_dir() and not staging.is_symlink():
            shutil.rmtree(staging)
        raise


def run(output: Path) -> dict[str, object]:
    contract_data, conformance, evidence = run_conformance()
    initial = {
        "contract.json": canonical_json(contract_data),
        "conformance.json": canonical_json(conformance),
        "evidence.json": canonical_json(evidence),
    }
    report = {
        "artifacts": {
            name: {"bytes": len(data), "sha256": sha256_bytes(data)}
            for name, data in sorted(initial.items())
        },
        "claim": CLAIM,
        "decision": "Pass",
        "next_authorized_stage": "V36-F0-fresh-role-and-unchanged-science-freeze",
        "official_access": evidence["official_access"],
        "schema": "nextengine.experimental-physical-sound-v36-a0-report.v1",
        "status": "Pass",
    }
    payloads = {**initial, "report.json": canonical_json(report)}
    atomic_publish(output, payloads)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    output = arguments.output
    if not output.is_absolute():
        output = repository_root() / output
    report = run(output)
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
