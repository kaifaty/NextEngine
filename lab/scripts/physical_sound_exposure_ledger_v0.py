#!/usr/bin/env python3
"""Build a deterministic Physical Sound Exposure Ledger V0."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from pathlib import Path, PurePosixPath
from typing import Any

CATALOG_SCHEMA = "nextengine.experimental-physical-sound-exposure-catalog.v0"
LEDGER_SCHEMA = "nextengine.experimental-physical-sound-exposure-ledger.v0"
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-exposure-ledger-build.report.v0"
)
MAX_CATALOG_BYTES = 16 * 1024 * 1024
MAX_ARTIFACT_BYTES = 32 * 1024 * 1024
MAX_ARTIFACTS = 4096
MAX_EXPOSURES = 250_000

PARTITION_POLICIES = {"historical_union", "disjoint_evaluation"}
ARTIFACT_KINDS = {"manifest", "report", "record", "other_json"}
PAYLOAD_KINDS = {
    "microphone",
    "force",
    "derived_response",
    "geometry",
    "metadata",
    "other",
}
ROLES = {
    "source_inventory",
    "estimator_fit",
    "generator_development",
    "representation_holdout",
    "validator_calibration",
    "validator_method_holdout",
    "method_holdout",
    "admission_shadow",
    "historical_unknown",
    "fallback_only",
}
ROLE_SIDES = {
    "source_inventory": "fit_side",
    "estimator_fit": "fit_side",
    "generator_development": "fit_side",
    "representation_holdout": "holdout_side",
    "validator_calibration": "validator_side",
    "validator_method_holdout": "validator_side",
    "method_holdout": "validator_side",
    "admission_shadow": "validator_side",
    "fallback_only": "fallback_side",
    "historical_unknown": "unknown_side",
}
FIT_OR_DEVELOPMENT_ROLES = {
    "source_inventory",
    "estimator_fit",
    "generator_development",
}
ACCESS_KINDS = {
    "metadata_only",
    "payload_hashed",
    "derived_signal_decoded",
    "signal_decoded",
}
DECODED_ACCESS_KINDS = {"derived_signal_decoded", "signal_decoded"}

CATALOG_KEYS = {
    "schema",
    "revision",
    "store_root_id",
    "partition_policy",
    "artifacts",
    "exposures",
    "network_allowed",
    "source_signal_access_allowed",
    "outputs_external",
}
ARTIFACT_KEYS = {
    "artifact_id",
    "relative_path",
    "sha256",
    "bytes",
    "kind",
    "parse_json",
    "schema_hint",
}
EXPOSURE_KEYS = {
    "identity",
    "role",
    "access_kind",
    "values_decoded",
    "protected",
    "evidence",
}
IDENTITY_KEYS = {
    "source_namespace",
    "project_id",
    "object_id",
    "contact_id",
    "listener_id",
    "impact_id",
    "mutation_parent_id",
    "payload_kind",
}
OPTIONAL_IDENTITY_KEYS = {
    "contact_id",
    "listener_id",
    "impact_id",
    "mutation_parent_id",
}
EVIDENCE_KEYS = {"binds", "artifact_id", "json_pointer", "value_sha256"}
EVIDENCE_BINDINGS = IDENTITY_KEYS | {"role", "access_kind", "values_decoded"}

HASH_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,191}\Z")


class ExposureLedgerError(RuntimeError):
    """The catalog or ledger violates the frozen M1b protocol."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["fixture", "build"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--catalog", type=Path)
    parser.add_argument("--store", type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        text = json.dumps(value, indent=2, sort_keys=True, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise ExposureLedgerError(f"cannot encode canonical JSON: {error}") from error
    return (text + "\n").encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def synthetic_hash(label: str) -> str:
    return sha256_bytes(f"physical-sound-m1b-synthetic:{label}".encode())


def reject_constant(value: str) -> None:
    raise ExposureLedgerError(f"non-finite JSON number is forbidden: {value}")


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ExposureLedgerError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_json_bytes(data: bytes, context: str) -> Any:
    try:
        return json.loads(
            data,
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_constant,
        )
    except UnicodeDecodeError as error:
        raise ExposureLedgerError(f"{context} is not UTF-8: {error}") from error
    except json.JSONDecodeError as error:
        raise ExposureLedgerError(f"cannot parse {context}: {error}") from error
    except RecursionError as error:
        raise ExposureLedgerError(f"{context} nesting is too deep") from error


def decode_canonical_catalog(data: bytes) -> dict[str, Any]:
    if len(data) > MAX_CATALOG_BYTES:
        raise ExposureLedgerError("catalog exceeds frozen byte limit")
    value = parse_json_bytes(data, "catalog JSON")
    if canonical_json(value) != data:
        raise ExposureLedgerError("catalog JSON is not canonical")
    if not isinstance(value, dict):
        raise ExposureLedgerError("catalog must be an object")
    return value


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ExposureLedgerError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise ExposureLedgerError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_identifier(value: Any, context: str) -> str:
    if not isinstance(value, str) or not IDENTIFIER_PATTERN.fullmatch(value):
        raise ExposureLedgerError(f"{context} must be a bounded identifier")
    return value


def require_hash(value: Any, context: str) -> str:
    if not isinstance(value, str) or not HASH_PATTERN.fullmatch(value):
        raise ExposureLedgerError(f"{context} must be a lowercase SHA-256")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise ExposureLedgerError(f"{context} must be a non-negative integer")
    return value


def require_bool(value: Any, context: str) -> bool:
    if type(value) is not bool:
        raise ExposureLedgerError(f"{context} must be boolean")
    return value


def normalized_relative_path(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value or "\\" in value:
        raise ExposureLedgerError(f"{context} must be a normalized relative path")
    candidate = PurePosixPath(value)
    if candidate.is_absolute() or any(part in {"", ".", ".."} for part in candidate.parts):
        raise ExposureLedgerError(f"{context} escapes or is not normalized")
    if candidate.as_posix() != value or candidate.suffix != ".json":
        raise ExposureLedgerError(f"{context} must be a normalized JSON path")
    return value


def validate_artifact(value: Any, index: int) -> dict[str, Any]:
    artifact = require_exact_keys(value, ARTIFACT_KEYS, f"artifacts[{index}]")
    require_identifier(artifact["artifact_id"], f"artifacts[{index}].artifact_id")
    normalized_relative_path(
        artifact["relative_path"], f"artifacts[{index}].relative_path"
    )
    require_hash(artifact["sha256"], f"artifacts[{index}].sha256")
    byte_count = require_nonnegative_integer(
        artifact["bytes"], f"artifacts[{index}].bytes"
    )
    if byte_count > MAX_ARTIFACT_BYTES:
        raise ExposureLedgerError("artifact exceeds frozen byte limit")
    if artifact["kind"] not in ARTIFACT_KINDS:
        raise ExposureLedgerError("unknown artifact kind")
    parse = require_bool(artifact["parse_json"], f"artifacts[{index}].parse_json")
    schema_hint = artifact["schema_hint"]
    if schema_hint is not None:
        require_identifier(schema_hint, f"artifacts[{index}].schema_hint")
        if not parse:
            raise ExposureLedgerError("schema_hint requires parse_json")
    return artifact


def validate_identity(value: Any, index: int) -> dict[str, Any]:
    identity = require_exact_keys(value, IDENTITY_KEYS, f"exposures[{index}].identity")
    for key in ("source_namespace", "project_id", "object_id"):
        require_identifier(identity[key], f"exposures[{index}].identity.{key}")
    for key in OPTIONAL_IDENTITY_KEYS:
        if identity[key] is not None:
            require_identifier(identity[key], f"exposures[{index}].identity.{key}")
    if identity["payload_kind"] not in PAYLOAD_KINDS:
        raise ExposureLedgerError("unknown payload kind")
    return identity


def validate_evidence(value: Any, exposure_index: int, evidence_index: int) -> dict[str, Any]:
    evidence = require_exact_keys(
        value,
        EVIDENCE_KEYS,
        f"exposures[{exposure_index}].evidence[{evidence_index}]",
    )
    if evidence["binds"] not in EVIDENCE_BINDINGS:
        raise ExposureLedgerError("unknown evidence binding")
    require_identifier(evidence["artifact_id"], "evidence.artifact_id")
    pointer = evidence["json_pointer"]
    if not isinstance(pointer, str) or not pointer.startswith("/"):
        raise ExposureLedgerError("evidence.json_pointer must be an RFC 6901 pointer")
    if len(pointer) > 1024:
        raise ExposureLedgerError("evidence.json_pointer is too long")
    require_hash(evidence["value_sha256"], "evidence.value_sha256")
    return evidence


def exposure_sort_key(exposure: dict[str, Any]) -> bytes:
    return canonical_json(exposure)


def validate_exposure(value: Any, index: int) -> dict[str, Any]:
    exposure = require_exact_keys(value, EXPOSURE_KEYS, f"exposures[{index}]")
    identity = validate_identity(exposure["identity"], index)
    role = exposure["role"]
    if role not in ROLES:
        raise ExposureLedgerError("unknown exposure role")
    access_kind = exposure["access_kind"]
    if access_kind not in ACCESS_KINDS:
        raise ExposureLedgerError("unknown access kind")
    values = require_nonnegative_integer(
        exposure["values_decoded"], f"exposures[{index}].values_decoded"
    )
    protected = require_bool(exposure["protected"], f"exposures[{index}].protected")
    if access_kind not in DECODED_ACCESS_KINDS and values != 0:
        raise ExposureLedgerError("non-decoded access must have zero values_decoded")
    if protected and role in FIT_OR_DEVELOPMENT_ROLES:
        raise ExposureLedgerError("fit/development exposure cannot be protected")
    raw_evidence = exposure["evidence"]
    if not isinstance(raw_evidence, list) or not raw_evidence:
        raise ExposureLedgerError("exposure evidence must be a non-empty array")
    evidence = [
        validate_evidence(item, index, evidence_index)
        for evidence_index, item in enumerate(raw_evidence)
    ]
    bindings = [item["binds"] for item in evidence]
    if len(bindings) != len(set(bindings)):
        raise ExposureLedgerError("duplicate evidence binding")
    if bindings != sorted(bindings):
        raise ExposureLedgerError("evidence bindings are not sorted")
    required = {"project_id", "object_id"}
    required.update(
        key for key in OPTIONAL_IDENTITY_KEYS if identity[key] is not None
    )
    missing = sorted(required - set(bindings))
    if missing:
        raise ExposureLedgerError(f"unevidenced identity components: {missing}")
    return exposure


def validate_catalog(value: Any) -> dict[str, Any]:
    catalog = require_exact_keys(value, CATALOG_KEYS, "catalog")
    if catalog["schema"] != CATALOG_SCHEMA:
        raise ExposureLedgerError("unknown exposure-catalog schema")
    require_identifier(catalog["revision"], "catalog.revision")
    require_identifier(catalog["store_root_id"], "catalog.store_root_id")
    if catalog["partition_policy"] not in PARTITION_POLICIES:
        raise ExposureLedgerError("unknown partition policy")
    if catalog["network_allowed"] is not False:
        raise ExposureLedgerError("catalog cannot authorize network access")
    if catalog["source_signal_access_allowed"] is not False:
        raise ExposureLedgerError("catalog cannot authorize source signal access")
    if catalog["outputs_external"] is not True:
        raise ExposureLedgerError("catalog outputs must remain external")

    raw_artifacts = catalog["artifacts"]
    if not isinstance(raw_artifacts, list) or not raw_artifacts:
        raise ExposureLedgerError("catalog artifacts must be a non-empty array")
    if len(raw_artifacts) > MAX_ARTIFACTS:
        raise ExposureLedgerError("catalog has too many artifacts")
    artifacts = [validate_artifact(item, index) for index, item in enumerate(raw_artifacts)]
    artifact_ids = [item["artifact_id"] for item in artifacts]
    artifact_paths = [item["relative_path"] for item in artifacts]
    if len(artifact_ids) != len(set(artifact_ids)):
        raise ExposureLedgerError("duplicate artifact_id")
    if len(artifact_paths) != len(set(artifact_paths)):
        raise ExposureLedgerError("duplicate artifact path")
    if artifact_ids != sorted(artifact_ids):
        raise ExposureLedgerError("artifacts are not sorted by artifact_id")

    raw_exposures = catalog["exposures"]
    if not isinstance(raw_exposures, list) or not raw_exposures:
        raise ExposureLedgerError("catalog exposures must be a non-empty array")
    if len(raw_exposures) > MAX_EXPOSURES:
        raise ExposureLedgerError("catalog has too many exposures")
    exposures = [
        validate_exposure(item, index) for index, item in enumerate(raw_exposures)
    ]
    if exposures != sorted(exposures, key=exposure_sort_key):
        raise ExposureLedgerError("exposures are not in canonical sorted order")
    exposure_hashes = [sha256_bytes(canonical_json(item)) for item in exposures]
    if len(exposure_hashes) != len(set(exposure_hashes)):
        raise ExposureLedgerError("duplicate exact exposure")
    if catalog["partition_policy"] == "disjoint_evaluation" and any(
        item["role"] == "historical_unknown" for item in exposures
    ):
        raise ExposureLedgerError("disjoint catalog cannot contain historical_unknown")
    return catalog


def load_catalog(path: Path) -> tuple[bytes, dict[str, Any]]:
    if path.stat().st_size > MAX_CATALOG_BYTES:
        raise ExposureLedgerError("catalog exceeds frozen byte limit")
    data = path.read_bytes()
    return data, validate_catalog(decode_canonical_catalog(data))


def has_symlink_component(root: Path, relative_path: str) -> bool:
    current = root
    for part in PurePosixPath(relative_path).parts:
        current = current / part
        if current.is_symlink():
            return True
    return False


def scalar_count(value: Any) -> int:
    count = 0
    stack = [value]
    while stack:
        current = stack.pop()
        if isinstance(current, dict):
            stack.extend(current.values())
        elif isinstance(current, list):
            stack.extend(current)
        else:
            count += 1
    return count


def read_artifacts(
    catalog: dict[str, Any], store: Path
) -> tuple[dict[str, dict[str, Any]], dict[str, int]]:
    if store.is_symlink():
        raise ExposureLedgerError("store root cannot be a symlink")
    root = store.resolve(strict=True)
    if not root.is_dir():
        raise ExposureLedgerError("store root is not a directory")
    repo = repository_root().resolve(strict=True)
    if root == repo or root.is_relative_to(repo):
        raise ExposureLedgerError("store root must remain outside the repository")
    loaded: dict[str, dict[str, Any]] = {}
    counters = {
        "artifact_bytes_hashed": 0,
        "metadata_files_parsed": 0,
        "metadata_scalar_values_parsed": 0,
        "network_requests": 0,
        "source_bytes_read": 0,
        "signal_values_decoded": 0,
        "protected_signal_values_decoded": 0,
    }
    for artifact in catalog["artifacts"]:
        relative_path = artifact["relative_path"]
        if has_symlink_component(root, relative_path):
            raise ExposureLedgerError(f"artifact path contains symlink: {relative_path}")
        path = (root / relative_path).resolve(strict=True)
        if not path.is_file() or not path.is_relative_to(root):
            raise ExposureLedgerError(f"artifact escapes store: {relative_path}")
        if path.stat().st_size != artifact["bytes"]:
            raise ExposureLedgerError(f"artifact byte length changed: {relative_path}")
        data = path.read_bytes()
        if sha256_bytes(data) != artifact["sha256"]:
            raise ExposureLedgerError(f"artifact hash changed: {relative_path}")
        counters["artifact_bytes_hashed"] += len(data)
        parsed: Any = None
        if artifact["parse_json"]:
            parsed = parse_json_bytes(data, f"artifact {relative_path}")
            counters["metadata_files_parsed"] += 1
            counters["metadata_scalar_values_parsed"] += scalar_count(parsed)
            if artifact["schema_hint"] is not None:
                if not isinstance(parsed, dict) or parsed.get("schema") != artifact["schema_hint"]:
                    raise ExposureLedgerError(
                        f"artifact schema changed: {relative_path}"
                    )
        loaded[artifact["artifact_id"]] = {
            "declaration": artifact,
            "data": data,
            "parsed": parsed,
        }
    return loaded, counters


def decode_pointer_token(token: str) -> str:
    result = []
    index = 0
    while index < len(token):
        character = token[index]
        if character != "~":
            result.append(character)
            index += 1
            continue
        if index + 1 >= len(token) or token[index + 1] not in {"0", "1"}:
            raise ExposureLedgerError("invalid JSON Pointer escape")
        result.append("~" if token[index + 1] == "0" else "/")
        index += 2
    return "".join(result)


def resolve_json_pointer(value: Any, pointer: str) -> Any:
    current = value
    for raw_token in pointer.split("/")[1:]:
        token = decode_pointer_token(raw_token)
        if isinstance(current, dict):
            if token not in current:
                raise ExposureLedgerError(f"unresolved JSON Pointer: {pointer}")
            current = current[token]
        elif isinstance(current, list):
            if not token.isdigit() or (len(token) > 1 and token.startswith("0")):
                raise ExposureLedgerError(f"invalid array token in JSON Pointer: {pointer}")
            index = int(token)
            if index >= len(current):
                raise ExposureLedgerError(f"unresolved JSON Pointer: {pointer}")
            current = current[index]
        else:
            raise ExposureLedgerError(f"JSON Pointer traverses scalar: {pointer}")
    return current


def normalize_bound_identity(value: Any, context: str) -> str:
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        raise ExposureLedgerError(f"{context} must resolve to string or integer")
    result = str(value)
    require_identifier(result, context)
    return result


def verify_binding(exposure: dict[str, Any], binding: str, resolved: Any) -> None:
    if binding in IDENTITY_KEYS:
        expected = exposure["identity"][binding]
        if expected is None:
            if resolved is not None:
                raise ExposureLedgerError(f"evidence does not match null {binding}")
        elif normalize_bound_identity(resolved, f"evidence for {binding}") != expected:
            raise ExposureLedgerError(f"evidence does not match identity.{binding}")
    elif binding == "values_decoded":
        if type(resolved) is not int or resolved != exposure["values_decoded"]:
            raise ExposureLedgerError("evidence does not match values_decoded")
    elif resolved != exposure[binding]:
        raise ExposureLedgerError(f"evidence does not match {binding}")


def verify_exposures(
    catalog: dict[str, Any], artifacts: dict[str, dict[str, Any]]
) -> None:
    for exposure in catalog["exposures"]:
        for evidence in exposure["evidence"]:
            artifact_id = evidence["artifact_id"]
            if artifact_id not in artifacts:
                raise ExposureLedgerError(
                    f"evidence references unknown artifact: {artifact_id}"
                )
            artifact = artifacts[artifact_id]
            if not artifact["declaration"]["parse_json"]:
                raise ExposureLedgerError("evidence references unparsed artifact")
            resolved = resolve_json_pointer(
                artifact["parsed"], evidence["json_pointer"]
            )
            if sha256_bytes(canonical_json(resolved)) != evidence["value_sha256"]:
                raise ExposureLedgerError("evidence pointed value hash changed")
            verify_binding(exposure, evidence["binds"], resolved)


def group_identity(identity: dict[str, Any], kind: str) -> dict[str, Any] | None:
    base = {
        "source_namespace": identity["source_namespace"],
        "project_id": identity["project_id"],
        "object_id": identity["object_id"],
    }
    if kind == "object_parent":
        return base
    if kind == "contact_parent":
        return {**base, "contact_id": identity["contact_id"]}
    if kind == "listener_parent":
        return {
            **base,
            "contact_id": identity["contact_id"],
            "listener_id": identity["listener_id"],
        }
    if kind == "mutation_parent":
        if identity["mutation_parent_id"] is None:
            return None
        return {**base, "mutation_parent_id": identity["mutation_parent_id"]}
    raise ExposureLedgerError(f"unknown group kind: {kind}")


def group_hash(identity: dict[str, Any], kind: str) -> str | None:
    group = group_identity(identity, kind)
    return None if group is None else sha256_bytes(canonical_json(group))


def partition_overlaps(exposures: list[dict[str, Any]]) -> list[dict[str, Any]]:
    groups: dict[tuple[str, str], set[str]] = {}
    for exposure in exposures:
        identity = exposure["identity"]
        side = ROLE_SIDES[exposure["role"]]
        for kind in ("contact_parent", "mutation_parent"):
            digest = group_hash(identity, kind)
            if digest is not None:
                groups.setdefault((kind, digest), set()).add(side)
    return [
        {"kind": kind, "group_sha256": digest, "sides": sorted(sides)}
        for (kind, digest), sides in sorted(groups.items())
        if len(sides) > 1
    ]


def aggregate_entries(exposures: list[dict[str, Any]]) -> list[dict[str, Any]]:
    groups: dict[str, dict[str, Any]] = {}
    for exposure in exposures:
        identity = exposure["identity"]
        identity_hash = sha256_bytes(canonical_json(identity))
        if identity_hash not in groups:
            groups[identity_hash] = {
                "identity_sha256": identity_hash,
                "identity": identity,
                "groups": {
                    kind: group_hash(identity, kind)
                    for kind in (
                        "object_parent",
                        "contact_parent",
                        "listener_parent",
                        "mutation_parent",
                    )
                },
                "roles": set(),
                "exposures": [],
            }
        entry = groups[identity_hash]
        entry["roles"].add(exposure["role"])
        entry["exposures"].append(
            {
                "role": exposure["role"],
                "access_kind": exposure["access_kind"],
                "values_decoded": exposure["values_decoded"],
                "protected": exposure["protected"],
                "evidence_sha256": sha256_bytes(canonical_json(exposure["evidence"])),
            }
        )
    entries = []
    for identity_hash in sorted(groups):
        entry = groups[identity_hash]
        entries.append(
            {
                **entry,
                "roles": sorted(entry["roles"]),
                "exposures": sorted(
                    entry["exposures"], key=lambda item: canonical_json(item)
                ),
            }
        )
    return entries


def group_index(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    groups: dict[tuple[str, str], set[str]] = {}
    for entry in entries:
        for kind, digest in entry["groups"].items():
            if digest is not None:
                groups.setdefault((kind, digest), set()).add(entry["identity_sha256"])
    return [
        {
            "kind": kind,
            "group_sha256": digest,
            "member_identity_sha256": sorted(members),
        }
        for (kind, digest), members in sorted(groups.items())
    ]


def artifact_root(catalog: dict[str, Any]) -> str:
    rows = [
        {
            "artifact_id": item["artifact_id"],
            "relative_path": item["relative_path"],
            "sha256": item["sha256"],
            "bytes": item["bytes"],
        }
        for item in catalog["artifacts"]
    ]
    return sha256_bytes(canonical_json(rows))


def role_root(entries: list[dict[str, Any]]) -> str:
    rows = [
        {"identity_sha256": item["identity_sha256"], "roles": item["roles"]}
        for item in entries
    ]
    return sha256_bytes(canonical_json(rows))


def build_documents(
    catalog_bytes: bytes,
    catalog: dict[str, Any],
    artifacts: dict[str, dict[str, Any]],
    build_access: dict[str, int],
    decision: str,
    opens_only: str,
) -> dict[str, bytes]:
    verify_exposures(catalog, artifacts)
    overlaps = partition_overlaps(catalog["exposures"])
    if catalog["partition_policy"] == "disjoint_evaluation" and overlaps:
        first = overlaps[0]
        raise ExposureLedgerError(
            f"partition leak: {first['kind']} {first['group_sha256']} "
            f"crosses {first['sides']}"
        )
    entries = aggregate_entries(catalog["exposures"])
    historical_signal = sum(
        item["values_decoded"]
        for item in catalog["exposures"]
        if item["access_kind"] in DECODED_ACCESS_KINDS
    )
    historical_protected = sum(
        item["values_decoded"]
        for item in catalog["exposures"]
        if item["access_kind"] in DECODED_ACCESS_KINDS and item["protected"]
    )
    ledger_role_root = role_root(entries)
    ledger = {
        "schema": LEDGER_SCHEMA,
        "revision": catalog["revision"],
        "catalog_sha256": sha256_bytes(catalog_bytes),
        "store_root_id": catalog["store_root_id"],
        "partition_policy": catalog["partition_policy"],
        "artifact_root_sha256": artifact_root(catalog),
        "role_root_sha256": ledger_role_root,
        "entries": entries,
        "group_index": group_index(entries),
        "partition_observations": {
            "cross_side_group_count": len(overlaps),
            "cross_side_groups": overlaps,
        },
        "counters": {
            "artifact_count": len(catalog["artifacts"]),
            "exposure_count": len(catalog["exposures"]),
            "sample_identity_count": len(entries),
            "historical_signal_values_decoded": historical_signal,
            "historical_protected_signal_values_decoded": historical_protected,
        },
        "decision": "EXPOSURE_LEDGER_V0_BUILT",
        "public_contract": False,
        "runtime_consumer_allowed": False,
    }
    ledger_bytes = canonical_json(ledger)
    summary = {
        "schema": LEDGER_SCHEMA,
        "sha256": sha256_bytes(ledger_bytes),
        "role_root_sha256": ledger_role_root,
        "sample_identity_count": len(entries),
        "signal_values_decoded": historical_signal,
        "protected_signal_values_decoded": historical_protected,
    }
    summary_bytes = canonical_json(summary)
    report = {
        "schema": REPORT_SCHEMA,
        "decision": decision,
        "catalog_sha256": sha256_bytes(catalog_bytes),
        "ledger_sha256": sha256_bytes(ledger_bytes),
        "summary_sha256": sha256_bytes(summary_bytes),
        "role_root_sha256": ledger_role_root,
        "partition_policy": catalog["partition_policy"],
        "build_access": build_access,
        "historical_exposure": {
            "signal_values_decoded": historical_signal,
            "protected_signal_values_decoded": historical_protected,
        },
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "opens_only": opens_only,
    }
    return {
        "catalog.json": catalog_bytes,
        "ledger.json": ledger_bytes,
        "summary.json": summary_bytes,
        "report.json": canonical_json(report),
    }


def prepare_output(output: Path) -> Path:
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise ExposureLedgerError("output must remain outside the repository")
    if resolved.exists():
        raise ExposureLedgerError(f"refusing to replace existing output: {resolved}")
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


def build_ledger(
    catalog_path: Path,
    store: Path,
    output: Path,
    *,
    decision: str = "EXPOSURE_LEDGER_V0_BUILD_PASS",
    opens_only: str = "M1c_evidence_evaluation",
) -> Path:
    catalog_bytes, catalog = load_catalog(catalog_path)
    artifacts, build_access = read_artifacts(catalog, store)
    files = build_documents(
        catalog_bytes,
        catalog,
        artifacts,
        build_access,
        decision,
        opens_only,
    )
    return publish_directory(output, files)


def evidence_reference(
    binding: str, artifact_id: str, pointer: str, resolved: Any
) -> dict[str, Any]:
    return {
        "binds": binding,
        "artifact_id": artifact_id,
        "json_pointer": pointer,
        "value_sha256": sha256_bytes(canonical_json(resolved)),
    }


def synthetic_documents(
    *, contact_leak: bool = False, mutation_leak: bool = False
) -> tuple[dict[str, bytes], dict[str, Any]]:
    validator_contact = "contact-fit" if contact_leak else "contact-validator"
    validator_mutation = "mutation-fit" if mutation_leak else "mutation-validator"
    manifest = {
        "schema": "nextengine.synthetic-physical-sound-exposure-source.v0",
        "study_id": "synthetic-exposure-project",
        "object": {"id": "synthetic-object-001"},
        "contacts": [
            {
                "contact_id": "contact-fit",
                "mutation_parent_id": "mutation-fit",
                "role": "estimator_fit",
                "values_decoded": 128,
            },
            {
                "contact_id": "contact-holdout",
                "mutation_parent_id": "mutation-holdout",
                "role": "representation_holdout",
                "values_decoded": 64,
            },
            {
                "contact_id": validator_contact,
                "mutation_parent_id": validator_mutation,
                "role": "validator_method_holdout",
                "values_decoded": 128,
            },
        ],
    }
    manifest_bytes = canonical_json(manifest)
    report = {
        "schema": "nextengine.synthetic-physical-sound-exposure-report.v0",
        "decision": "SYNTHETIC_ONLY",
    }
    report_bytes = canonical_json(report)
    artifact_id = "synthetic-manifest"
    artifacts = [
        {
            "artifact_id": artifact_id,
            "relative_path": "manifest.json",
            "sha256": sha256_bytes(manifest_bytes),
            "bytes": len(manifest_bytes),
            "kind": "manifest",
            "parse_json": True,
            "schema_hint": manifest["schema"],
        },
        {
            "artifact_id": "synthetic-report-hash-only",
            "relative_path": "report.json",
            "sha256": sha256_bytes(report_bytes),
            "bytes": len(report_bytes),
            "kind": "report",
            "parse_json": False,
            "schema_hint": None,
        },
    ]

    exposure_specs = [
        (0, "microphone", False),
        (0, "force", False),
        (1, "derived_response", True),
        (2, "microphone", True),
    ]
    exposures = []
    for contact_index, payload_kind, protected in exposure_specs:
        contact = manifest["contacts"][contact_index]
        identity = {
            "source_namespace": "synthetic-source",
            "project_id": manifest["study_id"],
            "object_id": manifest["object"]["id"],
            "contact_id": contact["contact_id"],
            "listener_id": None,
            "impact_id": None,
            "mutation_parent_id": contact["mutation_parent_id"],
            "payload_kind": payload_kind,
        }
        pointers = {
            "project_id": "/study_id",
            "object_id": "/object/id",
            "contact_id": f"/contacts/{contact_index}/contact_id",
            "mutation_parent_id": (
                f"/contacts/{contact_index}/mutation_parent_id"
            ),
            "role": f"/contacts/{contact_index}/role",
            "values_decoded": f"/contacts/{contact_index}/values_decoded",
        }
        resolved_values = {
            "project_id": manifest["study_id"],
            "object_id": manifest["object"]["id"],
            "contact_id": contact["contact_id"],
            "mutation_parent_id": contact["mutation_parent_id"],
            "role": contact["role"],
            "values_decoded": contact["values_decoded"],
        }
        evidence = [
            evidence_reference(binding, artifact_id, pointer, resolved_values[binding])
            for binding, pointer in sorted(pointers.items())
        ]
        exposures.append(
            {
                "identity": identity,
                "role": contact["role"],
                "access_kind": (
                    "derived_signal_decoded"
                    if payload_kind == "derived_response"
                    else "signal_decoded"
                ),
                "values_decoded": contact["values_decoded"],
                "protected": protected,
                "evidence": evidence,
            }
        )
    catalog = {
        "schema": CATALOG_SCHEMA,
        "revision": "synthetic-exposure-ledger-v0",
        "store_root_id": "synthetic-external-store",
        "partition_policy": "disjoint_evaluation",
        "artifacts": sorted(artifacts, key=lambda item: item["artifact_id"]),
        "exposures": sorted(exposures, key=exposure_sort_key),
        "network_allowed": False,
        "source_signal_access_allowed": False,
        "outputs_external": True,
    }
    return {"manifest.json": manifest_bytes, "report.json": report_bytes}, catalog


def write_synthetic_inputs(
    root: Path, *, contact_leak: bool = False, mutation_leak: bool = False
) -> tuple[Path, Path]:
    store = root / "store"
    store.mkdir(parents=True)
    documents, catalog = synthetic_documents(
        contact_leak=contact_leak, mutation_leak=mutation_leak
    )
    for name, data in documents.items():
        (store / name).write_bytes(data)
    catalog_path = root / "catalog.json"
    catalog_path.write_bytes(canonical_json(catalog))
    return store, catalog_path


def build_fixture(output: Path) -> Path:
    destination = output.resolve()
    prepare_output(destination)
    with tempfile.TemporaryDirectory(
        prefix="physical-sound-m1b-input-", dir=destination.parent
    ) as temporary:
        store, catalog = write_synthetic_inputs(Path(temporary))
        return build_ledger(
            catalog,
            store,
            destination,
            decision="M1B_EXPOSURE_LEDGER_V0_FIXTURE_PASS",
            opens_only="M1c_real_historical_catalog_freeze",
        )


def main() -> None:
    arguments = parse_arguments()
    if arguments.stage == "fixture":
        if arguments.catalog is not None or arguments.store is not None:
            raise ExposureLedgerError("fixture stage does not accept input paths")
        destination = build_fixture(arguments.output)
    else:
        if arguments.catalog is None or arguments.store is None:
            raise ExposureLedgerError("build stage requires --catalog and --store")
        destination = build_ledger(
            arguments.catalog, arguments.store, arguments.output
        )
    print(destination)


if __name__ == "__main__":
    main()
