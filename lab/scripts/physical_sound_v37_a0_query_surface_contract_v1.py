#!/usr/bin/env python3
"""Prove the target-free V37 query-surface contract and disposition rules."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from typing import cast

import numpy as np
import physical_sound_v37_query_surface_contract_v1 as contract

OWNER_PATH = "lab/scripts/physical_sound_v37_a0_query_surface_contract_v1.py"
CONTRACT_PATH = "lab/scripts/physical_sound_v37_query_surface_contract_v1.py"
PROFILE_PATH = "lab/profiles/physical-sound-v37-a0-query-surface-contract.v1.json"
RESEARCH_PATH = (
    "docs/development/physical-sound-v37-query-surface-operator-research-2026-09-03.md"
)
V36_RESULT_PATH = (
    "docs/development/physical-sound-v36-d0-fresh-development-result-2026-09-03.md"
)
RESEARCH_SHA256 = "9970c0c30e4b948be00cc345fee5fc774fa9a86e34abad29276431bdffeba1d2"
V36_RESULT_SHA256 = "35b4dde5ad956ee3d5f4d6596b8850ea9896839e34de32bba78b9534adbdd7f2"
CLAIM = (
    "TARGET_FREE_QUERY_SURFACE_CONTAINER_LIFECYCLE_AND_CANDIDATE_DISPOSITION_"
    "CONFORMANCE_ONLY / NO_FRESH_TRUTH_TARGET_MODEL_OFFICIAL_CAPABILITY_QUALITY_"
    "REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v37-a0-report.v1"
CONFORMANCE_SCHEMA = "nextengine.experimental-physical-sound-v37-a0-conformance.v1"
EVIDENCE_SCHEMA = "nextengine.experimental-physical-sound-v37-a0-evidence.v1"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-a0-query-surface-profile.v1"
)
VALID_HASHES = tuple(character * 64 for character in "1234")


class A0ConformanceError(RuntimeError):
    """The A0 owner cannot publish target-free conformance."""


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


def load_profile() -> dict[str, object]:
    path = repository_root() / PROFILE_PATH
    data = path.read_bytes()
    try:
        parsed = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise A0ConformanceError("A0 profile is invalid JSON") from error
    if not isinstance(parsed, dict):
        raise A0ConformanceError("A0 profile must be an object")
    profile = cast(dict[str, object], parsed)
    if canonical_json(profile) != data:
        raise A0ConformanceError("A0 profile must use canonical JSON")
    if profile.get("schema") != PROFILE_SCHEMA or profile.get("claim") != CLAIM:
        raise A0ConformanceError("A0 profile schema or claim mismatch")
    return profile


def profile_context_names(profile: dict[str, object]) -> tuple[str, ...]:
    section = profile.get("contract")
    if not isinstance(section, dict):
        raise A0ConformanceError("profile contract section is absent")
    names = section.get("context_feature_names")
    if not isinstance(names, list) or not all(
        isinstance(value, str) for value in names
    ):
        raise A0ConformanceError("profile context feature names are invalid")
    return tuple(str(value) for value in names)


def profile_mutation_ids(profile: dict[str, object]) -> tuple[str, ...]:
    values = profile.get("mutation_ids")
    if not isinstance(values, list) or not all(
        isinstance(value, str) for value in values
    ):
        raise A0ConformanceError("profile mutation IDs are invalid")
    result = tuple(str(value) for value in values)
    if tuple(sorted(set(result))) != result:
        raise A0ConformanceError("profile mutation IDs must be canonical")
    return result


def profile_stages(profile: dict[str, object], provider: str) -> tuple[str, ...]:
    section = profile.get("lifecycle")
    if not isinstance(section, dict):
        raise A0ConformanceError("profile lifecycle section is absent")
    values = section.get(provider)
    if not isinstance(values, list) or not all(
        isinstance(value, str) for value in values
    ):
        raise A0ConformanceError(f"profile lifecycle is invalid: {provider}")
    return tuple(str(value) for value in values)


def sample_fields(*, permuted: bool = False) -> contract.SurfaceFieldSetV1:
    field_a = contract.FieldInputV1(
        field_id="field/a",
        probes=(
            contract.ProbeInputV1(
                "field/a/probe-2",
                (0.0, 1.0, 0.0),
                (0.0, 0.0, 1.0),
                0.45,
                -0.25,
                ("field/a/probe-1", "field/a/probe-0"),
            ),
            contract.ProbeInputV1(
                "field/a/probe-0",
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 1.0),
                0.25,
                0.75,
                ("field/a/probe-2", "field/a/probe-1"),
            ),
            contract.ProbeInputV1(
                "field/a/probe-1",
                (1.0, 0.0, 0.0),
                (0.0, 0.0, 1.0),
                0.30,
                0.50,
                ("field/a/probe-0", "field/a/probe-2"),
            ),
        ),
    )
    field_b = contract.FieldInputV1(
        field_id="field/b",
        probes=(
            contract.ProbeInputV1(
                "field/b/probe-1",
                (3.0, 0.0, 0.0),
                (0.0, 0.0, 1.0),
                0.35,
                -0.40,
                ("field/b/probe-2", "field/b/probe-0"),
            ),
            contract.ProbeInputV1(
                "field/b/probe-2",
                (2.0, 1.0, 0.0),
                (0.0, 0.0, 1.0),
                0.40,
                0.20,
                ("field/b/probe-0", "field/b/probe-1"),
            ),
            contract.ProbeInputV1(
                "field/b/probe-0",
                (2.0, 0.0, 0.0),
                (0.0, 0.0, 1.0),
                0.25,
                0.90,
                ("field/b/probe-1", "field/b/probe-2"),
            ),
        ),
    )
    fields: tuple[contract.FieldInputV1, ...] = (field_a, field_b)
    if permuted:
        fields = tuple(
            replace(
                field,
                probes=tuple(
                    replace(probe, neighbor_ids=tuple(reversed(probe.neighbor_ids)))
                    for probe in reversed(field.probes)
                ),
            )
            for field in reversed(fields)
        )
    return contract.canonical_surface_fields(fields)


def query_inputs(
    names: tuple[str, ...], *, permuted_rows: bool = False
) -> tuple[contract.QueryInputV1, ...]:
    values = {
        "query/0": {
            "contact-u": 0.20,
            "contact-v": 0.30,
            "mode-order-normalized": 0.10,
        },
        "query/1": {
            "contact-u": 0.65,
            "contact-v": 0.15,
            "mode-order-normalized": 0.70,
        },
    }
    rows = (
        contract.QueryInputV1(
            "query/0",
            "field/a",
            "field/a/triangle-0",
            (0.20, 0.30, 0.0),
            (0.0, 0.0, 1.0),
            (0.50, 0.20, 0.30),
            tuple(values["query/0"][name] for name in names),
            0.42,
        ),
        contract.QueryInputV1(
            "query/1",
            "field/b",
            "field/b/triangle-0",
            (2.65, 0.15, 0.0),
            (0.0, 0.0, 1.0),
            (0.20, 0.65, 0.15),
            tuple(values["query/1"][name] for name in names),
            -0.17,
        ),
    )
    return tuple(reversed(rows)) if permuted_rows else rows


def sample_batch(
    role: contract.RoleKind,
    names: tuple[str, ...],
    *,
    permuted: bool = False,
) -> contract.SurfaceQueryBatchV1:
    input_names = tuple(reversed(names)) if permuted else names
    return contract.canonical_query_batch(
        role,
        sample_fields(permuted=permuted),
        input_names,
        query_inputs(input_names, permuted_rows=permuted),
    )


class MemoryStructuralProvider:
    def __init__(
        self,
        kind: contract.ProviderKind,
        namespace: str,
        batches: dict[contract.RoleKind, contract.SurfaceQueryBatchV1],
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
        self, role: contract.RoleKind, capability: contract.AccessCapabilityV1
    ) -> contract.SurfaceQueryBatchV1:
        if (
            capability.provider_kind is not self.kind
            or capability.namespace != self.namespace
        ):
            raise contract.ContractError("memory provider capability mismatch")
        self.calls.append(role)
        return self._batches[role]


def complete_structural_trace(
    kind: contract.ProviderKind, names: tuple[str, ...]
) -> contract.ExecutionTraceV1:
    namespace = f"structural-a0-{contract.pipeline_for_provider(kind).value}"
    batches = {role: sample_batch(role, names) for role in contract.allowed_roles(kind)}
    provider = MemoryStructuralProvider(kind, namespace, batches)
    capability = contract.AccessCapabilityV1.structural(kind, namespace)
    lifecycle = contract.OwnerLifecycleV1(
        contract.pipeline_for_provider(kind), capability
    )
    for stage in contract.stages_for_provider(kind):
        role = contract.ROLE_STAGE.get(stage)
        if role is None:
            lifecycle.step(stage)
        else:
            lifecycle.materialize_role(stage, role, provider)
    trace = lifecycle.finish(contract.TerminalDecision.PREFLIGHT_PASS)
    contract.assert_complete_trace(trace)
    if provider.calls != list(contract.allowed_roles(kind)):
        raise A0ConformanceError("structural provider role order drift")
    return trace


def expect_contract_error(action: Callable[[], object], fragment: str) -> bool:
    try:
        action()
    except contract.ContractError as error:
        return fragment in str(error)
    return False


def mutation_asymmetric_csr() -> object:
    fields = sample_fields()
    values = tuple(int(value) for value in fields.edge_indices)
    return replace(
        fields,
        edge_offsets=contract.frozen_int_vector((0, 2, 4, 5, 7, 9, 11)),
        edge_indices=contract.frozen_int_vector((*values[:4], *values[5:])),
    )


def mutation_bad_barycentrics(names: tuple[str, ...]) -> object:
    batch = sample_batch(contract.RoleKind.TRAIN, names)
    values = np.array(batch.query_barycentrics, copy=True)
    values[0] = (0.80, 0.80, 0.0)
    return replace(
        batch,
        query_barycentrics=contract.frozen_float_matrix(
            tuple(tuple(float(value) for value in row) for row in values)
        ),
    )


def mutation_candidate_reject_freeze() -> object:
    return contract.CandidateDispositionV1(
        contract.TerminalDecision.METRIC_REJECT,
        False,
        contract.canonical_json({"invalid": True}),
        None,
    )


def mutation_cross_field_csr() -> object:
    fields = sample_fields()
    values = [int(value) for value in fields.edge_indices]
    values[0:2] = [3, 4]
    return replace(fields, edge_indices=contract.frozen_int_vector(tuple(values)))


def mutation_duplicate_context_name(names: tuple[str, ...]) -> object:
    duplicate = (names[0], names[0], names[2])
    return contract.canonical_query_batch(
        contract.RoleKind.TRAIN,
        sample_fields(),
        duplicate,
        query_inputs(names),
    )


def mutation_early_scientific_terminal() -> object:
    capability = contract.AccessCapabilityV1.structural(
        contract.ProviderKind.STRUCTURAL_D0, "structural-a0-early-terminal"
    )
    lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
    return lifecycle.finish(contract.TerminalDecision.METRIC_REJECT)


def mutation_field_offset_origin() -> object:
    fields = sample_fields()
    return replace(
        fields,
        field_offsets=contract.frozen_int_vector((1, 3, 6)),
    )


def mutation_legacy_v36_reject_as_h0_freeze() -> object:
    owner_hash, profile_hash, terminal_hash, weights_hash = VALID_HASHES
    legacy = contract.CandidateDispositionV1(
        contract.TerminalDecision.PASS,
        True,
        contract.canonical_json(
            {
                "candidate_weights_sha256": weights_hash,
                "contract_schema": (
                    "nextengine.experimental-physical-sound-v36-owner-contract.v1"
                ),
                "decision": "MetricReject",
                "owner_sha256": owner_hash,
                "profile_sha256": profile_hash,
                "status": "OfficialD0CandidateFrozenAfterPass",
                "terminal_sha256": terminal_hash,
            }
        ),
        None,
    )
    return contract.validate_h0_candidate_freeze(
        legacy,
        expected_owner_sha256=owner_hash,
        expected_profile_sha256=profile_hash,
        expected_terminal_sha256=terminal_hash,
        expected_weights_sha256=weights_hash,
    )


def mutation_noncanonical_direct_query(names: tuple[str, ...]) -> object:
    batch = sample_batch(contract.RoleKind.TRAIN, names)
    return replace(batch, context_feature_names=tuple(reversed(names)))


def mutation_nonunit_probe_normal() -> object:
    fields = sample_fields()
    values = np.array(fields.probe_normals, copy=True)
    values[0] = (0.0, 0.0, 2.0)
    return replace(
        fields,
        probe_normals=contract.frozen_float_matrix(
            tuple(tuple(float(value) for value in row) for row in values)
        ),
    )


def mutation_pass_without_weights() -> object:
    owner_hash, profile_hash, terminal_hash, _ = VALID_HASHES
    return contract.candidate_disposition(
        contract.TerminalDecision.PASS,
        owner_sha256=owner_hash,
        profile_sha256=profile_hash,
        terminal_sha256=terminal_hash,
        candidate_weights_sha256=None,
    )


def mutation_stage_skip() -> object:
    capability = contract.AccessCapabilityV1.structural(
        contract.ProviderKind.STRUCTURAL_D0, "structural-a0-stage-skip"
    )
    lifecycle = contract.OwnerLifecycleV1(contract.PipelineKind.D0, capability)
    lifecycle.step(contract.LifecycleStage.FIELD_ENCODING)
    return lifecycle


def mutation_unknown_query_field(names: tuple[str, ...]) -> object:
    rows = query_inputs(names)
    invalid = (replace(rows[0], field_id="field/missing"), rows[1])
    return contract.canonical_query_batch(
        contract.RoleKind.TRAIN, sample_fields(), names, invalid
    )


def mutation_writable_query_storage(names: tuple[str, ...]) -> object:
    batch = sample_batch(contract.RoleKind.TRAIN, names)
    return replace(batch, query_positions=np.array(batch.query_positions, copy=True))


def mutation_suite(names: tuple[str, ...]) -> dict[str, bool]:
    mutations = {
        "asymmetric-csr": expect_contract_error(
            mutation_asymmetric_csr, "adjacency must be symmetric"
        ),
        "bad-barycentrics": expect_contract_error(
            lambda: mutation_bad_barycentrics(names), "barycentrics must"
        ),
        "candidate-reject-freeze": expect_contract_error(
            mutation_candidate_reject_freeze, "candidate disposition"
        ),
        "cross-field-csr": expect_contract_error(
            mutation_cross_field_csr, "cross a field boundary"
        ),
        "duplicate-context-name": expect_contract_error(
            lambda: mutation_duplicate_context_name(names), "nonempty and unique"
        ),
        "early-scientific-terminal": expect_contract_error(
            mutation_early_scientific_terminal, "scientific terminal"
        ),
        "field-offset-origin": expect_contract_error(
            mutation_field_offset_origin, "field offsets"
        ),
        "legacy-v36-reject-as-h0-freeze": expect_contract_error(
            mutation_legacy_v36_reject_as_h0_freeze, "Pass authority mismatch"
        ),
        "noncanonical-direct-query": expect_contract_error(
            lambda: mutation_noncanonical_direct_query(names),
            "invalid or noncanonical",
        ),
        "nonunit-probe-normal": expect_contract_error(
            mutation_nonunit_probe_normal, "unit length"
        ),
        "pass-without-weights": expect_contract_error(
            mutation_pass_without_weights, "weight identity"
        ),
        "stage-skip": expect_contract_error(mutation_stage_skip, "stage mismatch"),
        "unknown-query-field": expect_contract_error(
            lambda: mutation_unknown_query_field(names), "unknown surface field"
        ),
        "writable-query-storage": expect_contract_error(
            lambda: mutation_writable_query_storage(names), "immutable finite"
        ),
    }
    return dict(sorted(mutations.items()))


def disposition_record() -> dict[str, object]:
    owner_hash, profile_hash, terminal_hash, weights_hash = VALID_HASHES
    passed = contract.candidate_disposition(
        contract.TerminalDecision.PASS,
        owner_sha256=owner_hash,
        profile_sha256=profile_hash,
        terminal_sha256=terminal_hash,
        candidate_weights_sha256=weights_hash,
    )
    frozen = contract.validate_h0_candidate_freeze(
        passed,
        expected_owner_sha256=owner_hash,
        expected_profile_sha256=profile_hash,
        expected_terminal_sha256=terminal_hash,
        expected_weights_sha256=weights_hash,
    )
    rejected = contract.candidate_disposition(
        contract.TerminalDecision.METRIC_REJECT,
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
    return {
        "fault": {
            "bundle_authority": fault.candidate_bundle_authority,
            "freeze_document": fault.freeze_document is not None,
            "rejected_evidence_document": fault.rejected_evidence_document is not None,
        },
        "pass": {
            "bundle_authority": passed.candidate_bundle_authority,
            "freeze_document": passed.freeze_document is not None,
            "h0_validation": frozen["status"],
            "rejected_evidence_document": passed.rejected_evidence_document is not None,
        },
        "scientific_reject": {
            "bundle_authority": rejected.candidate_bundle_authority,
            "freeze_document": rejected.freeze_document is not None,
            "rejected_evidence_document": (
                rejected.rejected_evidence_document is not None
            ),
        },
    }


def provider_topology_record(kind: contract.ProviderKind) -> dict[str, object]:
    stages = (
        *contract.stages_for_provider(kind),
        contract.LifecycleStage.TERMINAL_PUBLICATION,
    )
    return {
        "stages": [stage.value for stage in stages],
        "topology_sha256": contract.expected_topology_sha256(kind),
    }


def run_conformance() -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    profile = load_profile()
    names = profile_context_names(profile)
    fields = sample_fields()
    permuted_fields = sample_fields(permuted=True)
    train = sample_batch(contract.RoleKind.TRAIN, names)
    permuted_train = sample_batch(contract.RoleKind.TRAIN, names, permuted=True)
    d0_trace = complete_structural_trace(contract.ProviderKind.STRUCTURAL_D0, names)
    h0_trace = complete_structural_trace(contract.ProviderKind.STRUCTURAL_H0, names)
    mutations = mutation_suite(names)
    if tuple(mutations) != profile_mutation_ids(profile) or not all(mutations.values()):
        raise A0ConformanceError("A0 mutation matrix failed or drifted")
    expected_d0 = profile_stages(profile, "structural-d0")
    expected_h0 = profile_stages(profile, "structural-h0")
    actual_d0 = tuple(event.stage.value for event in d0_trace.events)
    actual_h0 = tuple(event.stage.value for event in h0_trace.events)
    if actual_d0 != expected_d0 or actual_h0 != expected_h0:
        raise A0ConformanceError("structural lifecycle profile drift")

    disposition = disposition_record()
    checks = {
        "candidate_disposition_is_unambiguous": disposition
        == {
            "fault": {
                "bundle_authority": False,
                "freeze_document": False,
                "rejected_evidence_document": False,
            },
            "pass": {
                "bundle_authority": True,
                "freeze_document": True,
                "h0_validation": contract.CandidateStatus.FROZEN_AFTER_PASS.value,
                "rejected_evidence_document": False,
            },
            "scientific_reject": {
                "bundle_authority": False,
                "freeze_document": False,
                "rejected_evidence_document": True,
            },
        },
        "explicit_counts": (
            fields.field_count == 2
            and fields.probe_count == 6
            and fields.directed_edge_count == 12
            and train.row_count == 2
        ),
        "field_enumeration_independent": (
            fields.root_sha256 == permuted_fields.root_sha256
        ),
        "immutable_storage": all(
            not value.flags.writeable
            for value in (
                fields.field_offsets,
                fields.probe_positions,
                fields.probe_normals,
                fields.probe_area_weights,
                fields.probe_mode_values,
                fields.edge_offsets,
                fields.edge_indices,
                train.row_field_indices,
                train.query_positions,
                train.query_normals,
                train.query_barycentrics,
                train.context_features,
                train.p1_contact_values,
            )
        ),
        "query_enumeration_independent": (
            train.structural_root_sha256 == permuted_train.structural_root_sha256
        ),
        "structural_d0_complete": d0_trace.topology_sha256
        == contract.expected_topology_sha256(contract.ProviderKind.STRUCTURAL_D0),
        "structural_h0_complete": h0_trace.topology_sha256
        == contract.expected_topology_sha256(contract.ProviderKind.STRUCTURAL_H0),
        "target_arrays_absent": train.targets is None
        and permuted_train.targets is None,
        "target_rows_zero": (
            d0_trace.access.target_rows_accessed == 0
            and h0_trace.access.target_rows_accessed == 0
        ),
    }
    if not all(checks.values()):
        failed = sorted(name for name, passed in checks.items() if not passed)
        raise A0ConformanceError(f"A0 positive conformance failed: {failed}")

    contract_data: dict[str, object] = {
        "candidate_disposition": disposition,
        "claim": CLAIM,
        "container": {
            "context_feature_names": list(train.context_feature_names),
            "directed_edge_count": fields.directed_edge_count,
            "field_count": fields.field_count,
            "probe_count": fields.probe_count,
            "query_row_count": train.row_count,
            "target_axis_count": contract.TARGET_AXIS_COUNT,
        },
        "provider_topologies": {
            kind.value: provider_topology_record(kind) for kind in contract.ProviderKind
        },
        "roles": [role.value for role in contract.RoleKind],
        "schema": contract.CONTRACT_SCHEMA,
    }
    conformance: dict[str, object] = {
        "checks": checks,
        "claim": CLAIM,
        "field_root_sha256": fields.root_sha256,
        "mutations": mutations,
        "query_root_sha256": train.structural_root_sha256,
        "schema": CONFORMANCE_SCHEMA,
        "structural_d0_trace": contract.trace_record(d0_trace),
        "structural_h0_trace": contract.trace_record(h0_trace),
        "terminal": contract.TerminalDecision.PREFLIGHT_PASS.value,
    }
    evidence: dict[str, object] = {
        "bindings": {
            "contract": bound_file(CONTRACT_PATH),
            "owner": bound_file(OWNER_PATH),
            "profile": bound_file(PROFILE_PATH),
            "research": bound_file(RESEARCH_PATH, RESEARCH_SHA256),
            "v36_terminal_result": bound_file(V36_RESULT_PATH, V36_RESULT_SHA256),
        },
        "claim": CLAIM,
        "official_access": {
            "fresh_v37_target_rows": 0,
            "model_parameters_initialized": 0,
            "network_requests": 0,
            "official_capabilities_constructed": 0,
            "prior_generation_values_read": 0,
            "protected_signal_values_decoded": 0,
            "real_signal_values_decoded": 0,
            "target_arrays_constructed": 0,
            "truth_formulas_evaluated": 0,
        },
        "schema": EVIDENCE_SCHEMA,
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
        "conformance.json": canonical_json(conformance),
        "contract.json": canonical_json(contract_data),
        "evidence.json": canonical_json(evidence),
    }
    report: dict[str, object] = {
        "artifacts": {
            name: {"bytes": len(data), "sha256": sha256_bytes(data)}
            for name, data in sorted(initial.items())
        },
        "claim": CLAIM,
        "decision": contract.TerminalDecision.PREFLIGHT_PASS.value,
        "next_authorized_stage": "V37-F0-fresh-operator-truth-and-role-freeze",
        "official_access": evidence["official_access"],
        "schema": REPORT_SCHEMA,
        "status": "Pass",
    }
    atomic_publish(output, {**initial, "report.json": canonical_json(report)})
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
