"""Atomic terminal publisher shared by every sealed V36 owner path."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

import physical_sound_v36_owner_contract_v1 as contract

PUBLISHER_SCHEMA = "nextengine.experimental-physical-sound-v36-terminal-publisher.v1"
TERMINAL_NAME = "terminal.json"


class PublicationError(RuntimeError):
    """A V36 terminal could not be validated or published atomically."""


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


def unexpected_exception_decision(
    access: contract.AccessLedger,
) -> contract.TerminalDecision:
    """Convert an unexpected owner exception without creating retry ambiguity."""

    if access.forbidden_access_count != 0:
        raise PublicationError("cannot classify an exception after forbidden access")
    if access.target_rows_accessed == 0:
        if access.provider_calls != 0:
            raise PublicationError("zero-row provider call cannot be classified")
        return contract.TerminalDecision.CONTRACT_REJECT
    if access.provider_calls <= 0:
        raise PublicationError("post-access exception lacks a provider call")
    return contract.TerminalDecision.OWNER_FAULT


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise PublicationError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


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


def trace_record(trace: contract.ExecutionTrace) -> dict[str, object]:
    return {
        "access": access_record(trace.access),
        "events": [
            {
                "callable_id": event.callable_id,
                "ordinal": event.ordinal,
                "stage": event.stage.value,
            }
            for event in trace.events
        ],
        "pipeline": trace.pipeline.value,
        "provider_kind": trace.provider_kind.value,
        "terminal": trace.terminal.value,
        "topology_sha256": trace.topology_sha256,
    }


def validate_trace(trace: contract.ExecutionTrace) -> None:
    if trace.pipeline is not contract.pipeline_for_provider(trace.provider_kind):
        raise PublicationError("trace pipeline/provider mismatch")
    if trace.access.forbidden_access_count != 0:
        raise PublicationError("trace contains forbidden access")
    access_values = access_record(trace.access)
    if any(value < 0 for value in access_values.values()):
        raise PublicationError("trace contains a negative access count")
    if trace.pipeline is contract.PipelineKind.D0:
        if trace.access.method_holdout_target_rows != 0:
            raise PublicationError("D0 trace contains method-holdout role access")
    elif trace.access.development_target_rows != 0:
        raise PublicationError("H0 trace contains development role access")
    if trace.provider_kind in (
        contract.ProviderKind.SURROGATE_D0,
        contract.ProviderKind.SURROGATE_H0,
    ) and (
        trace.access.official_d0_target_rows != 0
        or trace.access.official_h0_target_rows != 0
    ):
        raise PublicationError("surrogate trace contains official role access")
    if trace.provider_kind is contract.ProviderKind.OFFICIAL_D0 and (
        trace.access.official_d0_target_rows != trace.access.target_rows_accessed
        or trace.access.official_h0_target_rows != 0
    ):
        raise PublicationError("official D0 receipt does not cover its target access")
    if trace.provider_kind is contract.ProviderKind.OFFICIAL_H0 and (
        trace.access.official_h0_target_rows != trace.access.target_rows_accessed
        or trace.access.official_d0_target_rows != 0
    ):
        raise PublicationError("official H0 receipt does not cover its target access")
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

    expected_prefix = contract.stages_for_pipeline(trace.pipeline)
    actual_prefix = tuple(event.stage for event in trace.events[:-1])
    if actual_prefix != expected_prefix[: len(actual_prefix)]:
        raise PublicationError("trace stage sequence mutation")
    scientific = {
        contract.TerminalDecision.PASS,
        contract.TerminalDecision.METRIC_REJECT,
        contract.TerminalDecision.HARD_GATE_REJECT,
        contract.TerminalDecision.RESOURCE_REJECT,
    }
    if trace.terminal in scientific:
        if trace.access.provider_calls != 2:
            raise PublicationError(
                "scientific terminal requires exactly two role opens"
            )
        if trace.access.train_target_rows <= 0:
            raise PublicationError("scientific terminal lacks train role access")
        if trace.pipeline is contract.PipelineKind.D0:
            if trace.access.development_target_rows <= 0:
                raise PublicationError(
                    "D0 scientific terminal lacks development access"
                )
        elif trace.access.method_holdout_target_rows <= 0:
            raise PublicationError("H0 scientific terminal lacks holdout access")
        try:
            contract.assert_complete_trace(trace)
        except contract.ContractError as error:
            raise PublicationError(str(error)) from error
    elif trace.terminal is contract.TerminalDecision.OWNER_FAULT:
        if trace.access.target_rows_accessed <= 0 or trace.access.provider_calls <= 0:
            raise PublicationError("OwnerFault requires post-access trace")
    elif trace.terminal is contract.TerminalDecision.CONTRACT_REJECT:
        if trace.access.target_rows_accessed != 0 or trace.access.provider_calls != 0:
            raise PublicationError("ContractReject requires zero target access")
    else:
        raise PublicationError("unknown terminal decision")


def validate_output(output: Path, forbidden_root: Path) -> tuple[Path, Path]:
    root = forbidden_root.resolve(strict=True)
    unresolved = output if output.is_absolute() else root / output
    if unresolved.is_symlink() or unresolved.exists():
        raise PublicationError("output must be a fresh non-symlink path")
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise PublicationError("output must stay outside the forbidden root")
    staging = parent / f".{resolved.name}.nextengine-v36-terminal-staging"
    if staging.exists() or staging.is_symlink():
        raise PublicationError("owned staging path already exists")
    return resolved, staging


def validate_payloads(payloads: dict[str, bytes]) -> list[tuple[str, bytes]]:
    if not payloads:
        raise PublicationError("terminal payload set must be nonempty")
    result: list[tuple[str, bytes]] = []
    for name, data in sorted(payloads.items()):
        if Path(name).name != name or name == TERMINAL_NAME or not name:
            raise PublicationError("payload name must be a nonterminal leaf")
        if not isinstance(data, bytes) or not data:
            raise PublicationError("payload must be nonempty bytes")
        result.append((name, data))
    return result


def payload_manifest(payloads: list[tuple[str, bytes]]) -> dict[str, object]:
    return {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in payloads
    }


def build_terminal(
    trace: contract.ExecutionTrace,
    claim: str,
    payloads: list[tuple[str, bytes]],
) -> bytes:
    if not claim:
        raise PublicationError("terminal claim must be nonempty")
    return canonical_json(
        {
            "claim": claim,
            "payloads": payload_manifest(payloads),
            "schema": PUBLISHER_SCHEMA,
            "trace": trace_record(trace),
        }
    )


def tree_sha256(files: list[tuple[str, bytes]]) -> str:
    manifest = [
        {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}
        for name, data in sorted(files)
    ]
    return sha256_bytes(canonical_json(manifest))


def publish_terminal(
    output: Path,
    forbidden_root: Path,
    trace: contract.ExecutionTrace,
    claim: str,
    payloads: dict[str, bytes],
    maximum_output_bytes: int,
    *,
    failpoint: PublicationFailpoint | None = None,
) -> PublicationReceipt:
    validate_trace(trace)
    if trace.terminal is contract.TerminalDecision.CONTRACT_REJECT:
        raise PublicationError("ContractReject must leave output absent")
    if maximum_output_bytes <= 0:
        raise PublicationError("maximum output bytes must be positive")
    output_path, staging = validate_output(output, forbidden_root)
    ordered_payloads = validate_payloads(payloads)
    terminal_data = build_terminal(trace, claim, ordered_payloads)
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
    )
