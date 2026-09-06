"""Atomic terminal publication shared by V34 T0 and future official owners."""

from __future__ import annotations

import hashlib
import json
import shutil
import tempfile
from pathlib import Path
from typing import Any

SCIENTIFIC_OUTCOMES = (
    "Pass",
    "MetricReject",
    "HardGateReject",
    "ResourceReject",
)
FREEZE_NAME = "candidate-freeze.json"


class TerminalPublicationError(RuntimeError):
    """A terminal outcome cannot be published under the frozen contract."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise TerminalPublicationError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def validate_fresh_output(output: Path, repository_root: Path) -> Path:
    root = repository_root.resolve(strict=True)
    unresolved = output if output.is_absolute() else root / output
    if unresolved.is_symlink():
        raise TerminalPublicationError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise TerminalPublicationError("output must be a fresh external path")
    return resolved


def validate_access(access: dict[str, Any], expected_phase: str) -> None:
    if access.get("access_phase") != expected_phase:
        raise TerminalPublicationError("access phase mismatch")
    rows = access.get("target_rows_accessed")
    if not isinstance(rows, int) or rows < 0:
        raise TerminalPublicationError("target-row receipt is invalid")
    if expected_phase == "pre-access" and rows != 0:
        raise TerminalPublicationError("pre-access reject has target access")
    if expected_phase == "post-access" and rows <= 0:
        raise TerminalPublicationError("post-access terminal lacks access receipt")
    for key, value in access.items():
        if key.endswith(("_values_decoded", "_requests")) and (
            not isinstance(value, int) or value < 0
        ):
            raise TerminalPublicationError(f"invalid access counter: {key}")


def artifact_ref(name: str, data: bytes) -> dict[str, Any]:
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def pre_access_contract_reject(
    output: Path,
    repository_root: Path,
    access: dict[str, Any],
    reason: str,
) -> dict[str, Any]:
    resolved = validate_fresh_output(output, repository_root)
    validate_access(access, "pre-access")
    if not reason or resolved.exists():
        raise TerminalPublicationError("invalid pre-access reject")
    return {
        "access": access,
        "decision": "ContractReject",
        "output_published": False,
        "reason": reason,
        "schema": "nextengine.experimental-physical-sound-v34-pre-access-contract-reject.v1",
        "status": "Reject",
    }


def publish_scientific_terminal(
    output: Path,
    repository_root: Path,
    outcome: str,
    access: dict[str, Any],
    payloads: dict[str, bytes],
    claim: str,
    maximum_output_bytes: int,
    *,
    failpoint: str | None = None,
) -> dict[str, Any]:
    resolved = validate_fresh_output(output, repository_root)
    if outcome not in SCIENTIFIC_OUTCOMES:
        raise TerminalPublicationError("unknown scientific terminal outcome")
    validate_access(access, "post-access")
    if (
        not claim
        or not isinstance(maximum_output_bytes, int)
        or maximum_output_bytes <= 0
    ):
        raise TerminalPublicationError("publication bound or claim is invalid")
    if not payloads or any(
        not name or Path(name).name != name or not isinstance(data, bytes)
        for name, data in payloads.items()
    ):
        raise TerminalPublicationError("terminal payload set is invalid")
    has_freeze = FREEZE_NAME in payloads
    if (outcome == "Pass") != has_freeze:
        raise TerminalPublicationError("candidate freeze/outcome mismatch")
    if any(name in {"evidence.json", "report.json"} for name in payloads):
        raise TerminalPublicationError("payload shadows owner artifact")

    payload_refs = {
        name: artifact_ref(name, data) for name, data in sorted(payloads.items())
    }
    evidence = {
        "access": access,
        "artifacts": payload_refs,
        "atomic_staging": True,
        "claim": claim,
        "outcome": outcome,
        "schema": "nextengine.experimental-physical-sound-v34-scientific-terminal-evidence.v1",
    }
    evidence_data = canonical_json(evidence)
    report = {
        "access": access,
        "candidate_frozen": has_freeze,
        "claim": claim,
        "decision": outcome,
        "evidence_sha256": sha256_bytes(evidence_data),
        "schema": "nextengine.experimental-physical-sound-v34-scientific-terminal-report.v1",
        "status": "Pass" if outcome == "Pass" else "Reject",
    }
    report_data = canonical_json(report)
    total = sum(map(len, payloads.values())) + len(evidence_data) + len(report_data)
    if total > maximum_output_bytes:
        raise TerminalPublicationError("terminal output exceeds publication bound")

    staging = Path(
        tempfile.mkdtemp(prefix=".nextengine-v34-terminal-", dir=resolved.parent)
    )
    try:
        for index, (name, data) in enumerate(sorted(payloads.items())):
            (staging / name).write_bytes(data)
            if failpoint == "after-first-payload" and index == 0:
                raise TerminalPublicationError("injected publication failure")
        (staging / "evidence.json").write_bytes(evidence_data)
        if failpoint == "after-evidence":
            raise TerminalPublicationError("injected publication failure")
        (staging / "report.json").write_bytes(report_data)
        staging.replace(resolved)
        return report
    except Exception:
        shutil.rmtree(staging, ignore_errors=True)
        raise
