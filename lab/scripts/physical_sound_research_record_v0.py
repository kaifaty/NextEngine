#!/usr/bin/env python3
"""Validate and build synthetic Physical Sound Research Record V0 fixtures."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from pathlib import Path
from typing import Any

RECORD_SCHEMA = "nextengine.experimental-physical-sound-research-record.v0"
LEDGER_SCHEMA = "nextengine.experimental-physical-sound-exposure-ledger.v0"
DESCRIPTOR_SCHEMA = (
    "nextengine.experimental-physical-sound-research-record-schema.v0"
)
FIXTURE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-research-record-fixture.report.v0"
)
VALIDATION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-research-record-validation.report.v0"
)
MAX_RECORD_BYTES = 1024 * 1024

CLAIM_KINDS = ("canonical_impact_field", "measured_transfer_field")
AXIS_NAMES = (
    "canonical_excitation",
    "common_timebase",
    "contact_position",
    "force_calibration",
    "force_frequency_coverage",
    "geometry",
    "listener_condition",
    "microphone_calibration",
    "object_identity",
    "raw_force",
    "recorded_response",
    "support_condition",
)
COMMON_REQUIRED_AXES = {
    "contact_position",
    "geometry",
    "listener_condition",
    "object_identity",
    "recorded_response",
}
REQUIRED_AXES = {
    "canonical_impact_field": COMMON_REQUIRED_AXES | {"canonical_excitation"},
    "measured_transfer_field": COMMON_REQUIRED_AXES
    | {"common_timebase", "force_frequency_coverage", "raw_force"},
}
AXIS_STATES = {"known", "absent", "not_applicable"}
PROGRESSIVE_STATES = {"SourceQualified", "FormulaValidated", "AtlasAdmitted"}
TERMINAL_STATES = {"FallbackOnly", "FallbackOutOfDomain"}
LIFECYCLE_STATES = PROGRESSIVE_STATES | TERMINAL_STATES
ALLOWED_TRANSITIONS = {
    (None, "SourceQualified", "qualify_source"),
    ("SourceQualified", "FormulaValidated", "formula_validated"),
    ("SourceQualified", "FallbackOnly", "formula_tournament_rejected"),
    ("SourceQualified", "FallbackOutOfDomain", "source_out_of_domain"),
    ("FormulaValidated", "AtlasAdmitted", "atlas_admitted"),
    ("FormulaValidated", "FallbackOnly", "validator_rejected"),
    ("FormulaValidated", "FallbackOutOfDomain", "validator_out_of_domain"),
}
VALIDATOR_DECISIONS = {"NotRun", "Pass", "Reject", "FallbackOutOfDomain"}
HASH_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}\Z")

TOP_LEVEL_KEYS = {
    "schema",
    "record_id",
    "revision",
    "previous_record_sha256",
    "claim_kind",
    "lifecycle",
    "source",
    "axes",
    "exposure_ledger",
    "evidence",
    "validator",
    "fallback",
    "public_contract",
    "runtime_consumer_allowed",
}
LIFECYCLE_KEYS = {"state", "previous_state", "transition", "reason_code"}
SOURCE_KEYS = {
    "project_id",
    "project_revision",
    "object_id",
    "object_group_id",
    "published_material_label",
    "source_manifest_sha256",
    "provenance_sha256",
}
AXIS_KEYS = {"axis", "state", "evidence_sha256"}
LEDGER_KEYS = {
    "schema",
    "sha256",
    "role_root_sha256",
    "sample_identity_count",
    "signal_values_decoded",
    "protected_signal_values_decoded",
}
EVIDENCE_KEYS = {
    "preprocessing_sha256",
    "coverage_certificate_sha256",
    "formula_sha256",
    "held_metrics_sha256",
    "validator_release_sha256",
    "validator_votes_sha256",
    "cooker_manifest_sha256",
}
VALIDATOR_KEYS = {"decision", "independent"}
FALLBACK_KEYS = {"required", "kind", "identity_sha256", "reason_code"}


class ResearchRecordError(RuntimeError):
    """The record violates the frozen V0 protocol."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["fixture", "validate"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--record", type=Path)
    parser.add_argument("--previous", type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def synthetic_hash(label: str) -> str:
    return sha256_bytes(f"physical-sound-m1a-synthetic:{label}".encode())


def canonical_json(value: Any) -> bytes:
    try:
        encoded = json.dumps(value, indent=2, sort_keys=True, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise ResearchRecordError(f"cannot encode canonical JSON: {error}") from error
    return (encoded + "\n").encode("utf-8")


def canonical_sha256(record: dict[str, Any]) -> str:
    return sha256_bytes(canonical_json(record))


def reject_constant(value: str) -> None:
    raise ResearchRecordError(f"non-finite JSON number is forbidden: {value}")


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ResearchRecordError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def decode_canonical_json(data: bytes) -> Any:
    if len(data) > MAX_RECORD_BYTES:
        raise ResearchRecordError("record exceeds 1 MiB")
    try:
        value = json.loads(
            data,
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_constant,
        )
    except UnicodeDecodeError as error:
        raise ResearchRecordError(f"record is not UTF-8: {error}") from error
    except json.JSONDecodeError as error:
        raise ResearchRecordError(f"cannot parse record JSON: {error}") from error
    if canonical_json(value) != data:
        raise ResearchRecordError("record JSON is not canonical")
    return value


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ResearchRecordError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        missing = sorted(expected - actual)
        unknown = sorted(actual - expected)
        raise ResearchRecordError(
            f"{context} keys changed; missing={missing}, unknown={unknown}"
        )
    return value


def require_identifier(value: Any, context: str) -> str:
    if not isinstance(value, str) or not IDENTIFIER_PATTERN.fullmatch(value):
        raise ResearchRecordError(f"{context} must be a bounded identifier")
    return value


def require_text(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 256:
        raise ResearchRecordError(f"{context} must be non-empty bounded text")
    return value


def require_hash(value: Any, context: str) -> str:
    if not isinstance(value, str) or not HASH_PATTERN.fullmatch(value):
        raise ResearchRecordError(f"{context} must be a lowercase SHA-256")
    return value


def require_nullable_hash(value: Any, context: str) -> str | None:
    if value is None:
        return None
    return require_hash(value, context)


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise ResearchRecordError(f"{context} must be a non-negative integer")
    return value


def validate_source(value: Any) -> dict[str, Any]:
    source = require_exact_keys(value, SOURCE_KEYS, "source")
    for key in (
        "project_id",
        "project_revision",
        "object_id",
        "object_group_id",
    ):
        require_identifier(source[key], f"source.{key}")
    require_text(source["published_material_label"], "source.published_material_label")
    require_hash(source["source_manifest_sha256"], "source.source_manifest_sha256")
    require_hash(source["provenance_sha256"], "source.provenance_sha256")
    return source


def validate_axes(value: Any, claim_kind: str) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        raise ResearchRecordError("axes must be an array")
    if len(value) != len(AXIS_NAMES):
        raise ResearchRecordError("axes must contain the complete frozen vocabulary")
    names: list[str] = []
    by_name: dict[str, dict[str, Any]] = {}
    for index, raw_axis in enumerate(value):
        axis = require_exact_keys(raw_axis, AXIS_KEYS, f"axes[{index}]")
        name = axis["axis"]
        if name not in AXIS_NAMES:
            raise ResearchRecordError(f"unknown axis: {name}")
        if name in by_name:
            raise ResearchRecordError(f"duplicate axis: {name}")
        state = axis["state"]
        if state not in AXIS_STATES:
            raise ResearchRecordError(f"unknown axis state for {name}: {state}")
        if state == "known":
            require_hash(axis["evidence_sha256"], f"axis {name} evidence")
        elif axis["evidence_sha256"] is not None:
            raise ResearchRecordError(f"non-known axis {name} must have null evidence")
        names.append(name)
        by_name[name] = axis
    if tuple(names) != AXIS_NAMES:
        raise ResearchRecordError("axes are not in frozen sorted order")
    for name in REQUIRED_AXES[claim_kind]:
        if by_name[name]["state"] != "known":
            raise ResearchRecordError(
                f"claim {claim_kind} requires known axis {name}"
            )
    return value


def validate_ledger(value: Any) -> dict[str, Any]:
    ledger = require_exact_keys(value, LEDGER_KEYS, "exposure_ledger")
    if ledger["schema"] != LEDGER_SCHEMA:
        raise ResearchRecordError("unknown exposure-ledger schema")
    require_hash(ledger["sha256"], "exposure_ledger.sha256")
    require_hash(ledger["role_root_sha256"], "exposure_ledger.role_root_sha256")
    for key in (
        "sample_identity_count",
        "signal_values_decoded",
        "protected_signal_values_decoded",
    ):
        require_nonnegative_integer(ledger[key], f"exposure_ledger.{key}")
    if ledger["protected_signal_values_decoded"] > ledger["signal_values_decoded"]:
        raise ResearchRecordError("protected decoded values exceed total decoded values")
    return ledger


def validate_evidence(value: Any) -> dict[str, Any]:
    evidence = require_exact_keys(value, EVIDENCE_KEYS, "evidence")
    for key in sorted(EVIDENCE_KEYS):
        require_nullable_hash(evidence[key], f"evidence.{key}")
    return evidence


def validate_validator(value: Any) -> dict[str, Any]:
    validator = require_exact_keys(value, VALIDATOR_KEYS, "validator")
    if validator["decision"] not in VALIDATOR_DECISIONS:
        raise ResearchRecordError("unknown validator decision")
    if type(validator["independent"]) is not bool:
        raise ResearchRecordError("validator.independent must be boolean")
    if validator["decision"] == "NotRun" and validator["independent"]:
        raise ResearchRecordError("a validator that did not run cannot be independent")
    if validator["decision"] != "NotRun" and not validator["independent"]:
        raise ResearchRecordError("a validator decision must be independent")
    return validator


def validate_fallback(value: Any) -> dict[str, Any]:
    fallback = require_exact_keys(value, FALLBACK_KEYS, "fallback")
    if fallback["required"] is not True:
        raise ResearchRecordError("authored fallback must remain required")
    if fallback["kind"] != "authored_clip":
        raise ResearchRecordError("fallback must be an authored clip")
    require_hash(fallback["identity_sha256"], "fallback.identity_sha256")
    require_identifier(fallback["reason_code"], "fallback.reason_code")
    return fallback


def require_present(evidence: dict[str, Any], keys: tuple[str, ...], state: str) -> None:
    for key in keys:
        if evidence[key] is None:
            raise ResearchRecordError(f"{state} requires evidence.{key}")


def require_absent(evidence: dict[str, Any], keys: tuple[str, ...], state: str) -> None:
    for key in keys:
        if evidence[key] is not None:
            raise ResearchRecordError(f"{state} forbids evidence.{key}")


def validate_state_evidence(record: dict[str, Any]) -> None:
    state = record["lifecycle"]["state"]
    transition = record["lifecycle"]["transition"]
    evidence = record["evidence"]
    validator = record["validator"]
    formula_keys = ("formula_sha256", "held_metrics_sha256")
    validator_keys = ("validator_release_sha256", "validator_votes_sha256")
    cooker_key = ("cooker_manifest_sha256",)

    if state == "SourceQualified":
        require_absent(evidence, formula_keys + validator_keys + cooker_key, state)
        if validator != {"decision": "NotRun", "independent": False}:
            raise ResearchRecordError("SourceQualified forbids validator evidence")
        if record["exposure_ledger"]["signal_values_decoded"] != 0:
            raise ResearchRecordError("SourceQualified requires zero decoded signal")
        if record["exposure_ledger"]["protected_signal_values_decoded"] != 0:
            raise ResearchRecordError("SourceQualified requires zero protected signal")
    elif state == "FormulaValidated":
        require_present(evidence, formula_keys, state)
        require_absent(evidence, validator_keys + cooker_key, state)
        if validator != {"decision": "NotRun", "independent": False}:
            raise ResearchRecordError("FormulaValidated has not run the validator")
    elif state == "AtlasAdmitted":
        require_present(evidence, formula_keys + validator_keys + cooker_key, state)
        if validator != {"decision": "Pass", "independent": True}:
            raise ResearchRecordError(
                "AtlasAdmitted requires an independent passing validator"
            )
    elif state == "FallbackOnly":
        require_absent(evidence, cooker_key, state)
        if transition == "formula_tournament_rejected":
            require_present(evidence, formula_keys, state)
            require_absent(evidence, validator_keys, state)
            if validator != {"decision": "NotRun", "independent": False}:
                raise ResearchRecordError("tournament rejection cannot cite a validator")
        elif transition == "validator_rejected":
            require_present(evidence, formula_keys + validator_keys, state)
            if validator != {"decision": "Reject", "independent": True}:
                raise ResearchRecordError("validator rejection requires independent Reject")
    elif state == "FallbackOutOfDomain":
        require_absent(evidence, cooker_key, state)
        if transition == "source_out_of_domain":
            require_absent(evidence, formula_keys + validator_keys, state)
            if validator != {"decision": "NotRun", "independent": False}:
                raise ResearchRecordError("source OOD cannot cite a validator")
        elif transition == "validator_out_of_domain":
            require_present(evidence, formula_keys + validator_keys, state)
            if validator != {
                "decision": "FallbackOutOfDomain",
                "independent": True,
            }:
                raise ResearchRecordError(
                    "validator OOD requires independent FallbackOutOfDomain"
                )


def validate_record(value: Any) -> dict[str, Any]:
    record = require_exact_keys(value, TOP_LEVEL_KEYS, "record")
    if record["schema"] != RECORD_SCHEMA:
        raise ResearchRecordError("unknown research-record schema")
    require_identifier(record["record_id"], "record_id")
    revision = require_nonnegative_integer(record["revision"], "revision")
    previous_hash = require_nullable_hash(
        record["previous_record_sha256"], "previous_record_sha256"
    )
    claim_kind = record["claim_kind"]
    if claim_kind not in CLAIM_KINDS:
        raise ResearchRecordError("unknown claim kind")
    lifecycle = require_exact_keys(record["lifecycle"], LIFECYCLE_KEYS, "lifecycle")
    state = lifecycle["state"]
    previous_state = lifecycle["previous_state"]
    transition = lifecycle["transition"]
    if state not in LIFECYCLE_STATES:
        raise ResearchRecordError("unknown lifecycle state")
    if previous_state is not None and previous_state not in LIFECYCLE_STATES:
        raise ResearchRecordError("unknown previous lifecycle state")
    if (previous_state, state, transition) not in ALLOWED_TRANSITIONS:
        raise ResearchRecordError("lifecycle transition is not allowed")
    require_identifier(lifecycle["reason_code"], "lifecycle.reason_code")
    if previous_state is None:
        if revision != 0 or previous_hash is not None:
            raise ResearchRecordError("initial record must be revision 0 without parent")
    elif revision == 0 or previous_hash is None:
        raise ResearchRecordError("successor record requires revision and parent hash")
    validate_source(record["source"])
    validate_axes(record["axes"], claim_kind)
    validate_ledger(record["exposure_ledger"])
    validate_evidence(record["evidence"])
    validate_validator(record["validator"])
    validate_fallback(record["fallback"])
    if record["public_contract"] is not False:
        raise ResearchRecordError("research record cannot be a public contract")
    if record["runtime_consumer_allowed"] is not False:
        raise ResearchRecordError("research record cannot authorize runtime use")
    validate_state_evidence(record)
    return record


def fallback_identity(record: dict[str, Any]) -> dict[str, Any]:
    return {
        "required": record["fallback"]["required"],
        "kind": record["fallback"]["kind"],
        "identity_sha256": record["fallback"]["identity_sha256"],
    }


def validate_transition(
    previous: dict[str, Any], current: dict[str, Any]
) -> dict[str, Any]:
    validate_record(previous)
    validate_record(current)
    previous_state = previous["lifecycle"]["state"]
    if previous_state in TERMINAL_STATES:
        raise ResearchRecordError("terminal lifecycle state cannot be reopened")
    if current["lifecycle"]["previous_state"] != previous_state:
        raise ResearchRecordError("previous lifecycle state does not match parent")
    if current["previous_record_sha256"] != canonical_sha256(previous):
        raise ResearchRecordError("previous record hash does not match parent")
    if current["revision"] != previous["revision"] + 1:
        raise ResearchRecordError("successor revision is not parent revision plus one")
    invariant_fields = ("record_id", "claim_kind", "source", "axes")
    for field in invariant_fields:
        if current[field] != previous[field]:
            raise ResearchRecordError(f"successor changed invariant {field}")
    if fallback_identity(current) != fallback_identity(previous):
        raise ResearchRecordError("successor changed authored fallback identity")
    return current


def load_record(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    value = decode_canonical_json(data)
    return data, validate_record(value)


def schema_descriptor() -> dict[str, Any]:
    return {
        "schema": DESCRIPTOR_SCHEMA,
        "record_schema": RECORD_SCHEMA,
        "exposure_ledger_schema": LEDGER_SCHEMA,
        "compatibility": {
            "current_version_roundtrip": True,
            "migration_supported": False,
            "unknown_schema_policy": "reject_without_rewrite",
        },
        "canonical_json": {
            "encoding": "utf-8",
            "indent": 2,
            "keys": "sorted",
            "final_newline": True,
            "non_finite_numbers_allowed": False,
            "maximum_record_bytes": MAX_RECORD_BYTES,
        },
        "claim_kinds": list(CLAIM_KINDS),
        "axes": list(AXIS_NAMES),
        "axis_states": sorted(AXIS_STATES),
        "progressive_states": sorted(PROGRESSIVE_STATES),
        "terminal_states": sorted(TERMINAL_STATES),
        "transitions": [
            {
                "previous": previous,
                "current": current,
                "transition": transition,
            }
            for previous, current, transition in sorted(
                ALLOWED_TRANSITIONS, key=lambda item: (item[0] or "", item[1], item[2])
            )
        ],
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "fallback_required": True,
    }


def synthetic_source() -> dict[str, Any]:
    return {
        "project_id": "synthetic-m1a",
        "project_revision": "frozen-v0",
        "object_id": "synthetic-vessel-001",
        "object_group_id": "synthetic-vessel-family",
        "published_material_label": "synthetic glass-like fixture; not a claim",
        "source_manifest_sha256": synthetic_hash("source-manifest"),
        "provenance_sha256": synthetic_hash("provenance"),
    }


def synthetic_axes(claim_kind: str) -> list[dict[str, Any]]:
    required = REQUIRED_AXES[claim_kind]
    axes = []
    for name in AXIS_NAMES:
        if name in required or name == "support_condition":
            state = "known"
            evidence_hash: str | None = synthetic_hash(f"axis:{name}")
        elif claim_kind == "canonical_impact_field" and name.startswith("force_"):
            state = "not_applicable"
            evidence_hash = None
        elif claim_kind == "canonical_impact_field" and name in {
            "common_timebase",
            "raw_force",
        }:
            state = "not_applicable"
            evidence_hash = None
        else:
            state = "absent"
            evidence_hash = None
        axes.append(
            {"axis": name, "state": state, "evidence_sha256": evidence_hash}
        )
    return axes


def synthetic_ledger(label: str = "source") -> dict[str, Any]:
    return {
        "schema": LEDGER_SCHEMA,
        "sha256": synthetic_hash(f"ledger:{label}"),
        "role_root_sha256": synthetic_hash("ledger:role-root"),
        "sample_identity_count": 5,
        "signal_values_decoded": 0,
        "protected_signal_values_decoded": 0,
    }


def empty_evidence() -> dict[str, Any]:
    return {key: None for key in EVIDENCE_KEYS}


def synthetic_fallback(reason_code: str) -> dict[str, Any]:
    return {
        "required": True,
        "kind": "authored_clip",
        "identity_sha256": synthetic_hash("authored-fallback"),
        "reason_code": reason_code,
    }


def source_qualified_record(
    claim_kind: str = "canonical_impact_field",
) -> dict[str, Any]:
    record = {
        "schema": RECORD_SCHEMA,
        "record_id": f"m1a-{claim_kind}",
        "revision": 0,
        "previous_record_sha256": None,
        "claim_kind": claim_kind,
        "lifecycle": {
            "state": "SourceQualified",
            "previous_state": None,
            "transition": "qualify_source",
            "reason_code": "synthetic_source_qualified",
        },
        "source": synthetic_source(),
        "axes": synthetic_axes(claim_kind),
        "exposure_ledger": synthetic_ledger(),
        "evidence": empty_evidence(),
        "validator": {"decision": "NotRun", "independent": False},
        "fallback": synthetic_fallback("source_qualified_fallback"),
        "public_contract": False,
        "runtime_consumer_allowed": False,
    }
    return validate_record(record)


def successor(
    previous: dict[str, Any], state: str, transition: str, reason_code: str
) -> dict[str, Any]:
    record = json.loads(canonical_json(previous))
    record["revision"] = previous["revision"] + 1
    record["previous_record_sha256"] = canonical_sha256(previous)
    record["lifecycle"] = {
        "state": state,
        "previous_state": previous["lifecycle"]["state"],
        "transition": transition,
        "reason_code": reason_code,
    }
    record["fallback"]["reason_code"] = reason_code
    return record


def formula_validated_record(source: dict[str, Any]) -> dict[str, Any]:
    record = successor(
        source,
        "FormulaValidated",
        "formula_validated",
        "synthetic_formula_validated",
    )
    record["evidence"].update(
        {
            "preprocessing_sha256": synthetic_hash("preprocessing"),
            "coverage_certificate_sha256": synthetic_hash("coverage"),
            "formula_sha256": synthetic_hash("formula"),
            "held_metrics_sha256": synthetic_hash("held-metrics"),
        }
    )
    return validate_transition(source, record)


def atlas_admitted_record(formula: dict[str, Any]) -> dict[str, Any]:
    record = successor(
        formula, "AtlasAdmitted", "atlas_admitted", "synthetic_atlas_admitted"
    )
    record["evidence"].update(
        {
            "validator_release_sha256": synthetic_hash("validator-release"),
            "validator_votes_sha256": synthetic_hash("validator-votes"),
            "cooker_manifest_sha256": synthetic_hash("cooker-manifest"),
        }
    )
    record["validator"] = {"decision": "Pass", "independent": True}
    return validate_transition(formula, record)


def fallback_only_record(source: dict[str, Any]) -> dict[str, Any]:
    record = successor(
        source,
        "FallbackOnly",
        "formula_tournament_rejected",
        "synthetic_formula_tournament_rejected",
    )
    record["evidence"].update(
        {
            "preprocessing_sha256": synthetic_hash("fallback-preprocessing"),
            "coverage_certificate_sha256": synthetic_hash("fallback-coverage"),
            "formula_sha256": synthetic_hash("rejected-formula"),
            "held_metrics_sha256": synthetic_hash("rejected-held-metrics"),
        }
    )
    return validate_transition(source, record)


def fallback_ood_record(source: dict[str, Any]) -> dict[str, Any]:
    record = successor(
        source,
        "FallbackOutOfDomain",
        "source_out_of_domain",
        "synthetic_source_out_of_domain",
    )
    return validate_transition(source, record)


def fixture_records() -> dict[str, dict[str, Any]]:
    source = source_qualified_record()
    formula = formula_validated_record(source)
    return {
        "source-qualified.json": source,
        "formula-validated.json": formula,
        "atlas-admitted.json": atlas_admitted_record(formula),
        "fallback-only.json": fallback_only_record(source),
        "fallback-out-of-domain.json": fallback_ood_record(source),
    }


def prepare_output(output: Path) -> Path:
    resolved = output.resolve()
    root = repository_root().resolve(strict=True)
    if resolved == root or resolved.is_relative_to(root):
        raise ResearchRecordError("output must remain outside the repository")
    if resolved.exists():
        raise ResearchRecordError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish_directory(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for name, data in sorted(files.items()):
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def build_fixture(output: Path) -> Path:
    records = fixture_records()
    files = {name: canonical_json(record) for name, record in records.items()}
    files["schema.json"] = canonical_json(schema_descriptor())
    report = {
        "schema": FIXTURE_REPORT_SCHEMA,
        "decision": "M1A_RESEARCH_RECORD_V0_FIXTURE_PASS",
        "record_schema": RECORD_SCHEMA,
        "files": [
            {"path": name, "sha256": sha256_bytes(data), "bytes": len(data)}
            for name, data in sorted(files.items())
        ],
        "counters": {
            "network_requests": 0,
            "source_bytes_read": 0,
            "signal_values_decoded": 0,
            "protected_signal_values_decoded": 0,
        },
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "opens_only": "M1b_exposure_ledger_builder",
    }
    files["report.json"] = canonical_json(report)
    return publish_directory(output, files)


def validate_file(record_path: Path, previous_path: Path | None, output: Path) -> Path:
    data, record = load_record(record_path)
    if record["revision"] == 0:
        if previous_path is not None:
            raise ResearchRecordError("initial record must not supply --previous")
    else:
        if previous_path is None:
            raise ResearchRecordError("successor record requires --previous")
        _, previous = load_record(previous_path)
        validate_transition(previous, record)
    report = {
        "schema": VALIDATION_REPORT_SCHEMA,
        "decision": "CURRENT_V0_CANONICAL_ROUNDTRIP_PASS",
        "record_schema": RECORD_SCHEMA,
        "record_sha256": sha256_bytes(data),
        "record_bytes": len(data),
        "previous_record_supplied": previous_path is not None,
        "migration_performed": False,
        "public_contract": False,
        "runtime_consumer_allowed": False,
    }
    return publish_directory(
        output,
        {"record.json": canonical_json(record), "report.json": canonical_json(report)},
    )


def main() -> None:
    arguments = parse_arguments()
    if arguments.stage == "fixture":
        if arguments.record is not None or arguments.previous is not None:
            raise ResearchRecordError("fixture stage does not accept record inputs")
        destination = build_fixture(arguments.output)
    else:
        if arguments.record is None:
            raise ResearchRecordError("validate stage requires --record")
        destination = validate_file(
            arguments.record, arguments.previous, arguments.output
        )
    print(destination)


if __name__ == "__main__":
    main()
