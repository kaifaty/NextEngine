"""Atomic terminal publication for the V37 query-surface owner."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

import physical_sound_v37_query_surface_contract_v1 as contract

PUBLISHER_SCHEMA = "nextengine.experimental-physical-sound-v37-terminal-publisher.v1"
TERMINAL_NAME = "terminal.json"


class PublicationError(RuntimeError):
    """A V37 terminal could not be validated or published atomically."""


class PublicationFailpoint(str, Enum):
    AFTER_STAGING_CREATE = "after-staging-create"
    AFTER_FIRST_PAYLOAD = "after-first-payload"
    BEFORE_TERMINAL_DESCRIPTOR = "before-terminal-descriptor"
    BEFORE_ATOMIC_REPLACE = "before-atomic-replace"


@dataclass(frozen=True, slots=True)
class PublicationReceipt:
    output: str
    file_count: int
    total_bytes: int
    tree_sha256: str
    terminal_sha256: str
    decision_root_sha256: str


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise PublicationError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def access_record(access: contract.AccessLedgerV1) -> dict[str, int]:
    return {
        "development_target_rows": access.development_target_rows,
        "forbidden_access_count": access.forbidden_access_count,
        "method_holdout_target_rows": access.method_holdout_target_rows,
        "official_d0_target_rows": access.official_d0_target_rows,
        "official_h0_target_rows": access.official_h0_target_rows,
        "provider_calls": access.provider_calls,
        "structural_rows": access.structural_rows,
        "target_rows_accessed": access.target_rows_accessed,
        "train_target_rows": access.train_target_rows,
    }


def trace_record(trace: contract.ExecutionTraceV1) -> dict[str, object]:
    return contract.trace_record(trace)


def validate_trace(trace: contract.ExecutionTraceV1) -> None:
    if trace.pipeline is not contract.pipeline_for_provider(trace.provider_kind):
        raise PublicationError("trace pipeline/provider mismatch")
    access = access_record(trace.access)
    if any(value < 0 for value in access.values()):
        raise PublicationError("trace contains a negative access count")
    if trace.access.forbidden_access_count != 0:
        raise PublicationError("trace contains forbidden access")
    if trace.pipeline is contract.PipelineKind.D0:
        if trace.access.method_holdout_target_rows != 0:
            raise PublicationError("D0 trace contains method-holdout role access")
    elif trace.access.development_target_rows != 0:
        raise PublicationError("H0 trace contains development role access")
    if not contract.provider_is_official(trace.provider_kind) and (
        trace.access.official_d0_target_rows != 0
        or trace.access.official_h0_target_rows != 0
    ):
        raise PublicationError("non-official trace contains official access")
    if trace.provider_kind is contract.ProviderKind.OFFICIAL_D0 and (
        trace.access.official_d0_target_rows != trace.access.target_rows_accessed
        or trace.access.official_h0_target_rows != 0
    ):
        raise PublicationError("official D0 access receipt mismatch")
    if trace.provider_kind is contract.ProviderKind.OFFICIAL_H0 and (
        trace.access.official_h0_target_rows != trace.access.target_rows_accessed
        or trace.access.official_d0_target_rows != 0
    ):
        raise PublicationError("official H0 access receipt mismatch")
    if not trace.events or trace.events[-1].stage is not (
        contract.LifecycleStage.TERMINAL_PUBLICATION
    ):
        raise PublicationError("trace lacks terminal publication event")
    if any(event.ordinal != index for index, event in enumerate(trace.events)):
        raise PublicationError("trace ordinal mutation")
    if any(
        contract.STAGE_CALLABLES.get(event.stage) != event.callable_id
        for event in trace.events
    ):
        raise PublicationError("trace callable mutation")
    expected = contract.stages_for_provider(trace.provider_kind)
    actual = tuple(event.stage for event in trace.events[:-1])
    if actual != expected[: len(actual)]:
        raise PublicationError("trace stage sequence mutation")
    scientific = {
        contract.TerminalDecision.PASS,
        contract.TerminalDecision.METRIC_REJECT,
        contract.TerminalDecision.HARD_GATE_REJECT,
        contract.TerminalDecision.RESOURCE_REJECT,
    }
    if trace.terminal in scientific:
        if trace.access.provider_calls != 2 or trace.access.train_target_rows <= 0:
            raise PublicationError("scientific terminal requires two complete roles")
        if trace.pipeline is contract.PipelineKind.D0:
            if trace.access.development_target_rows <= 0:
                raise PublicationError("D0 scientific terminal lacks development")
        elif trace.access.method_holdout_target_rows <= 0:
            raise PublicationError("H0 scientific terminal lacks holdout")
        try:
            contract.assert_complete_trace(trace)
        except contract.ContractError as error:
            raise PublicationError(str(error)) from error
    elif trace.terminal is contract.TerminalDecision.OWNER_FAULT:
        if trace.access.target_rows_accessed <= 0:
            raise PublicationError("OwnerFault requires prior target access")
    else:
        raise PublicationError("terminal is not publishable")


def unexpected_exception_decision(
    access: contract.AccessLedgerV1,
) -> contract.TerminalDecision:
    if access.forbidden_access_count != 0:
        raise PublicationError("cannot classify a failure after forbidden access")
    if access.target_rows_accessed == 0:
        if access.provider_calls != 0:
            raise PublicationError("zero-row provider call cannot be classified")
        return contract.TerminalDecision.CONTRACT_REJECT
    return contract.TerminalDecision.OWNER_FAULT


def decision_root_sha256(
    trace: contract.ExecutionTraceV1,
    claim: str,
    owner_sha256: str,
    profile_sha256: str,
    candidate_weights_sha256: str | None,
) -> str:
    if not claim or not all(
        contract.is_sha256(value) for value in (owner_sha256, profile_sha256)
    ):
        raise PublicationError("decision root identity is invalid")
    if candidate_weights_sha256 is not None and not contract.is_sha256(
        candidate_weights_sha256
    ):
        raise PublicationError("candidate weight identity is invalid")
    return sha256_bytes(
        canonical_json(
            {
                "candidate_weights_sha256": candidate_weights_sha256,
                "claim": claim,
                "owner_sha256": owner_sha256,
                "profile_sha256": profile_sha256,
                "trace": trace_record(trace),
            }
        )
    )


def validate_output(output: Path, forbidden_root: Path) -> tuple[Path, Path]:
    root = forbidden_root.resolve(strict=True)
    unresolved = output if output.is_absolute() else root / output
    if unresolved.is_symlink() or unresolved.exists():
        raise PublicationError("output must be a fresh non-symlink path")
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise PublicationError("output must stay outside the forbidden root")
    staging = parent / f".{resolved.name}.nextengine-v37-terminal-staging"
    if staging.exists() or staging.is_symlink():
        raise PublicationError("owned staging path already exists")
    return resolved, staging


def validate_payloads(
    decision: contract.TerminalDecision,
    disposition: contract.CandidateDispositionV1,
    payloads: dict[str, bytes],
) -> list[tuple[str, bytes]]:
    if disposition.decision is not decision:
        raise PublicationError("disposition and trace decision differ")
    names = set(payloads)
    if decision is contract.TerminalDecision.PASS:
        required = {"candidate-bundle.json", "candidate-freeze.json", "evidence.json"}
        if names != required or disposition.freeze_document != payloads.get(
            "candidate-freeze.json"
        ):
            raise PublicationError("Pass payload set or freeze document is invalid")
    elif decision in {
        contract.TerminalDecision.METRIC_REJECT,
        contract.TerminalDecision.HARD_GATE_REJECT,
        contract.TerminalDecision.RESOURCE_REJECT,
    }:
        required = {"evidence.json", "rejected-candidate.json"}
        if names != required or disposition.rejected_evidence_document != payloads.get(
            "rejected-candidate.json"
        ):
            raise PublicationError("reject payload set or evidence is invalid")
    elif decision is contract.TerminalDecision.OWNER_FAULT:
        if names != {"owner-fault.json"}:
            raise PublicationError("OwnerFault may publish only owner-fault evidence")
    else:
        raise PublicationError("terminal is not publishable")
    ordered: list[tuple[str, bytes]] = []
    for name, data in sorted(payloads.items()):
        if Path(name).name != name or name == TERMINAL_NAME or not name:
            raise PublicationError("payload name must be a nonterminal leaf")
        if not isinstance(data, bytes) or not data:
            raise PublicationError("payload must be nonempty bytes")
        ordered.append((name, data))
    return ordered


def payload_manifest(payloads: list[tuple[str, bytes]]) -> dict[str, object]:
    return {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in payloads
    }


def tree_sha256(files: list[tuple[str, bytes]]) -> str:
    return sha256_bytes(
        canonical_json(
            [
                {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}
                for name, data in sorted(files)
            ]
        )
    )


def publish_terminal(
    output: Path,
    forbidden_root: Path,
    trace: contract.ExecutionTraceV1,
    claim: str,
    owner_sha256: str,
    profile_sha256: str,
    candidate_weights_sha256: str | None,
    disposition: contract.CandidateDispositionV1,
    payloads: dict[str, bytes],
    maximum_output_bytes: int,
    *,
    failpoint: PublicationFailpoint | None = None,
) -> PublicationReceipt:
    validate_trace(trace)
    if maximum_output_bytes <= 0:
        raise PublicationError("maximum output bytes must be positive")
    output_path, staging = validate_output(output, forbidden_root)
    ordered_payloads = validate_payloads(trace.terminal, disposition, payloads)
    decision_root = decision_root_sha256(
        trace, claim, owner_sha256, profile_sha256, candidate_weights_sha256
    )
    terminal_data = canonical_json(
        {
            "claim": claim,
            "decision_root_sha256": decision_root,
            "payloads": payload_manifest(ordered_payloads),
            "schema": PUBLISHER_SCHEMA,
            "trace": trace_record(trace),
        }
    )
    complete_files = [*ordered_payloads, (TERMINAL_NAME, terminal_data)]
    total_bytes = sum(len(data) for _, data in complete_files)
    if total_bytes > maximum_output_bytes:
        raise PublicationError("terminal output exceeds byte budget")

    staging.mkdir(mode=0o700)
    try:
        if failpoint is PublicationFailpoint.AFTER_STAGING_CREATE:
            raise PublicationError("injected publication failure: after-staging-create")
        for index, (name, data) in enumerate(ordered_payloads):
            (staging / name).write_bytes(data)
            if index == 0 and failpoint is PublicationFailpoint.AFTER_FIRST_PAYLOAD:
                raise PublicationError(
                    "injected publication failure: after-first-payload"
                )
        if failpoint is PublicationFailpoint.BEFORE_TERMINAL_DESCRIPTOR:
            raise PublicationError(
                "injected publication failure: before-terminal-descriptor"
            )
        (staging / TERMINAL_NAME).write_bytes(terminal_data)
        if failpoint is PublicationFailpoint.BEFORE_ATOMIC_REPLACE:
            raise PublicationError(
                "injected publication failure: before-atomic-replace"
            )
        os.replace(staging, output_path)
    except BaseException:
        if staging.is_dir() and not staging.is_symlink():
            shutil.rmtree(staging)
        raise
    return PublicationReceipt(
        output=str(output_path),
        file_count=len(complete_files),
        total_bytes=total_bytes,
        tree_sha256=tree_sha256(complete_files),
        terminal_sha256=sha256_bytes(terminal_data),
        decision_root_sha256=decision_root,
    )
