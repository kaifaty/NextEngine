#!/usr/bin/env python3
"""Evaluate V46 D0 synthetic-teacher provenance without opening payload values."""

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

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v46-d0-profile.v1"
SOURCE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v46-d0-source-preflight.v1"
)
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v46-d0-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v46-d0-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v46-d0-source-preflight.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v46_d0_source_preflight_v1.py"
PROTOCOL_PATH = (
    "docs/development/physical-sound-v46-d0-synthetic-source-preflight-protocol-"
    "2026-09-03.md"
)
R0_PROFILE_PATH = "lab/profiles/physical-sound-v45-r0-source-claim-ledger.v1.json"
T0_PROFILE_PATH = "lab/profiles/physical-sound-v45-t0-recipe-v3.v1.json"

STUDY_ID = "physical-sound-v46-d0-synthetic-source-preflight"
CLAIM = (
    "METADATA_AND_GENERATION_PROVENANCE_PREFLIGHT_ONLY / NO_DATASET_PAYLOAD_"
    "MODAL_VALUE_AUDIO_MODEL_TRAINING_VALIDATOR_PROTECTED_COOKER_DEMO_OR_"
    "RUNTIME_AUTHORITY"
)
TRUSTED_DECISION = "SyntheticTeacherTrusted"
UNTRUSTED_DECISION = "SyntheticTeacherUntrusted"
HASH_LENGTH = 64
GIT_HASH_LENGTH = 40
MAX_PROFILE_BYTES = 2 * 1024 * 1024
MAX_EXTERNAL_FILE_BYTES = 8 * 1024 * 1024
MAX_SIBLINGS = 200_000
MAX_ROOT_ENTRIES = 2_000

AUTHORITY = {
    "admission_authority": False,
    "authorizes_only": "v46_d1_modal_teacher_rows_for_trusted_sources",
    "corpus_increment_authority": False,
    "model_training_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "real_acoustic_parent_credit": 0,
    "runtime_consumer_allowed": False,
    "validator_authority": False,
}

ZERO_PAYLOAD_COUNTERS = (
    "archive_range_bytes_read",
    "audio_files_decoded",
    "dataset_payload_files_read",
    "modal_numeric_values_decoded",
    "model_values_read",
    "pcm_sample_values_decoded",
    "protected_values_read",
    "waveform_bytes_read",
)


class SourcePreflightError(RuntimeError):
    """D0 cannot publish a trustworthy provenance decision."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--metadata", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(
                value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False
            )
            + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise SourcePreflightError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SourcePreflightError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise SourcePreflightError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise SourcePreflightError(f"{context} must be a non-empty string")
    return value


def require_bool(value: Any, context: str) -> bool:
    if not isinstance(value, bool):
        raise SourcePreflightError(f"{context} must be a boolean")
    return value


def require_nonnegative_int(value: Any, context: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise SourcePreflightError(f"{context} must be a non-negative integer")
    return value


def require_hash(value: Any, context: str, length: int = HASH_LENGTH) -> str:
    text = require_string(value, context)
    if len(text) != length or any(char not in "0123456789abcdef" for char in text):
        raise SourcePreflightError(f"{context} must be a lowercase hexadecimal hash")
    return text


def require_sorted_strings(value: Any, context: str) -> list[str]:
    strings = [
        require_string(item, f"{context} item") for item in require_list(value, context)
    ]
    if strings != sorted(set(strings)):
        raise SourcePreflightError(f"{context} must be sorted and unique")
    return strings


def read_regular(
    path: Path, context: str, maximum: int = MAX_EXTERNAL_FILE_BYTES
) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise SourcePreflightError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise SourcePreflightError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_json(path: Path, context: str) -> tuple[bytes, Any]:
    data = read_regular(path, context)
    try:
        return data, json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourcePreflightError(f"{context} is not valid UTF-8 JSON") from error


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data, value = read_json(path, context)
    document = require_dict(value, context)
    if data != canonical_json(document):
        raise SourcePreflightError(f"{context} is not canonical JSON")
    return data, document


def validate_dependency(value: Any, context: str) -> dict[str, Any]:
    binding = require_dict(value, context)
    if set(binding) != {"bytes", "path", "sha256"}:
        raise SourcePreflightError(f"{context} fields changed")
    relative = Path(require_string(binding["path"], f"{context}.path"))
    count = require_nonnegative_int(binding["bytes"], f"{context}.bytes")
    digest = require_hash(binding["sha256"], f"{context}.sha256")
    if relative.is_absolute() or ".." in relative.parts or count == 0:
        raise SourcePreflightError(f"{context} path or byte count is invalid")
    data = read_regular(repository_root() / relative, context, MAX_PROFILE_BYTES)
    if len(data) != count or sha256_bytes(data) != digest:
        raise SourcePreflightError(f"{context} binding mismatch")
    return binding


def validate_external_filename(value: Any, context: str) -> str:
    name = require_string(value, context)
    path = Path(name)
    if path.is_absolute() or len(path.parts) != 1 or name in {".", ".."}:
        raise SourcePreflightError(f"{context} must be one external basename")
    return name


def validate_sample_plan(value: Any, context: str) -> dict[str, Any]:
    plan = require_dict(value, context)
    if set(plan) != {"access_condition", "maximum_bytes", "selection"}:
        raise SourcePreflightError(f"{context} fields changed")
    if plan["access_condition"] != "all_generation_provenance_gates_pass":
        raise SourcePreflightError(f"{context} access condition changed")
    if require_nonnegative_int(plan["maximum_bytes"], f"{context}.maximum_bytes") == 0:
        raise SourcePreflightError(f"{context} maximum bytes must be positive")
    require_string(plan["selection"], f"{context}.selection")
    return plan


def validate_source(value: Any, context: str) -> dict[str, Any]:
    source = require_dict(value, context)
    expected_fields = {
        "alias_component",
        "expected",
        "expected_decision",
        "metadata_files",
        "provenance_rule",
        "repository_id",
        "revision",
        "sample_plan",
        "source_id",
    }
    if set(source) != expected_fields:
        raise SourcePreflightError(f"{context} fields changed")
    source_id = require_string(source["source_id"], f"{context}.source_id")
    if source_id not in {"nisr-v5", "vibraverse"}:
        raise SourcePreflightError(f"{context} source is unsupported")
    require_string(source["alias_component"], f"{context}.alias_component")
    require_string(source["repository_id"], f"{context}.repository_id")
    require_hash(source["revision"], f"{context}.revision", GIT_HASH_LENGTH)
    files = require_dict(source["metadata_files"], f"{context}.metadata_files")
    expected_file_fields = {"readme", "revision", "root_tree"}
    if source_id == "nisr-v5":
        expected_file_fields.add("generation_link_headers")
    else:
        expected_file_fields.add("material_parameters")
    if set(files) != expected_file_fields:
        raise SourcePreflightError(f"{context} metadata files changed")
    for key, name in files.items():
        validate_external_filename(name, f"{context}.metadata_files.{key}")

    expected = require_dict(source["expected"], f"{context}.expected")
    if set(expected) != {
        "generation_path_candidates",
        "readme_required_literals",
        "readme_sha256",
        "root_entry_count",
        "sibling_count",
        "sibling_path_root_sha256",
        "stable_tags",
    } | ({"generation_link_http_status"} if source_id == "nisr-v5" else {"material_parameters_sha256"}):
        raise SourcePreflightError(f"{context} expected fields changed")
    require_nonnegative_int(expected["sibling_count"], f"{context}.sibling_count")
    require_hash(
        expected["sibling_path_root_sha256"], f"{context}.sibling_path_root_sha256"
    )
    require_nonnegative_int(expected["root_entry_count"], f"{context}.root_entry_count")
    require_hash(expected["readme_sha256"], f"{context}.readme_sha256")
    require_sorted_strings(expected["stable_tags"], f"{context}.stable_tags")
    require_sorted_strings(
        expected["generation_path_candidates"],
        f"{context}.generation_path_candidates",
    )
    require_sorted_strings(
        expected["readme_required_literals"], f"{context}.readme_required_literals"
    )
    if source_id == "nisr-v5":
        status = require_nonnegative_int(
            expected["generation_link_http_status"],
            f"{context}.generation_link_http_status",
        )
        if status < 100 or status > 599:
            raise SourcePreflightError(f"{context} HTTP status is invalid")
    else:
        require_hash(
            expected["material_parameters_sha256"],
            f"{context}.material_parameters_sha256",
        )

    rule = require_dict(source["provenance_rule"], f"{context}.provenance_rule")
    if set(rule) != {
        "artifact_to_generator_revision_binding_required",
        "executable_or_complete_generation_manifest_required",
        "upstream_asset_identity_required",
    } or not all(require_bool(item, f"{context}.provenance_rule") for item in rule.values()):
        raise SourcePreflightError(f"{context} provenance rule changed")
    validate_sample_plan(source["sample_plan"], f"{context}.sample_plan")
    decision = require_dict(source["expected_decision"], f"{context}.expected_decision")
    if set(decision) != {"decision", "reason"}:
        raise SourcePreflightError(f"{context} expected decision fields changed")
    if decision["decision"] != UNTRUSTED_DECISION:
        raise SourcePreflightError(f"{context} expected decision must fail closed")
    require_string(decision["reason"], f"{context}.expected_decision.reason")
    return source


def validate_profile(profile: dict[str, Any]) -> list[dict[str, Any]]:
    if set(profile) != {
        "access_policy",
        "authority",
        "claim",
        "dependency_bindings",
        "expected_result",
        "schema",
        "sources",
        "study_id",
    }:
        raise SourcePreflightError("profile fields changed")
    if (
        profile["schema"] != PROFILE_SCHEMA
        or profile["study_id"] != STUDY_ID
        or profile["claim"] != CLAIM
        or profile["authority"] != AUTHORITY
    ):
        raise SourcePreflightError("profile identity or authority changed")
    if profile["access_policy"] != {
        "bulk_download_allowed": False,
        "dataset_payload_access_allowed_before_provenance_pass": False,
        "metadata_network_requests_declared": 8,
        "outputs_external": True,
        "separate_source_decisions": True,
    }:
        raise SourcePreflightError("access policy changed")
    dependencies = [
        validate_dependency(item, f"dependency {index}")
        for index, item in enumerate(
            require_list(profile["dependency_bindings"], "dependency bindings")
        )
    ]
    dependency_paths = [item["path"] for item in dependencies]
    required_dependencies = {
        OWNER_PATH,
        PROTOCOL_PATH,
        R0_PROFILE_PATH,
        T0_PROFILE_PATH,
    }
    if dependency_paths != sorted(set(dependency_paths)) or not required_dependencies.issubset(
        dependency_paths
    ):
        raise SourcePreflightError("dependencies must be sorted, unique and complete")
    sources = [
        validate_source(item, f"source {index}")
        for index, item in enumerate(require_list(profile["sources"], "sources"))
    ]
    if [item["source_id"] for item in sources] != ["nisr-v5", "vibraverse"]:
        raise SourcePreflightError("source order or identities changed")
    expected_result = require_dict(profile["expected_result"], "expected result")
    if expected_result != {
        "payload_access_counters_zero": True,
        "source_decisions": {
            "nisr-v5": {
                "decision": UNTRUSTED_DECISION,
                "reason": "DeclaredGenerationDocumentUnavailable",
            },
            "vibraverse": {
                "decision": UNTRUSTED_DECISION,
                "reason": "GenerationLineageIncomplete",
            },
        },
        "trusted_source_count": 0,
    }:
        raise SourcePreflightError("expected result changed")
    return sources


def sibling_path_root(paths: list[str]) -> str:
    material = "".join(f"{path}\n" for path in sorted(paths)).encode()
    return sha256_bytes(material)


def parse_revision(
    data: bytes, source: dict[str, Any], context: str
) -> tuple[dict[str, Any], list[str]]:
    try:
        document = require_dict(json.loads(data), context)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourcePreflightError(f"{context} is not valid JSON") from error
    for field in ("id", "sha", "private", "gated", "tags", "siblings"):
        if field not in document:
            raise SourcePreflightError(f"{context} misses {field}")
    if document["id"] != source["repository_id"] or document["sha"] != source["revision"]:
        raise SourcePreflightError(f"{context} repository identity changed")
    if document["private"] is not False or document["gated"] is not False:
        raise SourcePreflightError(f"{context} is private or gated")
    tags = sorted(set(require_string(item, f"{context} tag") for item in require_list(document["tags"], f"{context} tags")))
    if not set(source["expected"]["stable_tags"]).issubset(tags):
        raise SourcePreflightError(f"{context} stable tags changed")
    siblings = require_list(document["siblings"], f"{context} siblings")
    if len(siblings) > MAX_SIBLINGS:
        raise SourcePreflightError(f"{context} sibling resource bound exceeded")
    paths = []
    for index, value in enumerate(siblings):
        item = require_dict(value, f"{context} sibling {index}")
        path = require_string(item.get("rfilename"), f"{context} sibling {index} path")
        if Path(path).is_absolute() or ".." in Path(path).parts:
            raise SourcePreflightError(f"{context} sibling path is unsafe")
        paths.append(path)
    if len(paths) != len(set(paths)):
        raise SourcePreflightError(f"{context} sibling paths are duplicated")
    expected = source["expected"]
    if len(paths) != expected["sibling_count"]:
        raise SourcePreflightError(f"{context} sibling count changed")
    root = sibling_path_root(paths)
    if root != expected["sibling_path_root_sha256"]:
        raise SourcePreflightError(f"{context} sibling path root changed")
    return document, paths


def parse_root_tree(data: bytes, source: dict[str, Any], context: str) -> list[str]:
    try:
        entries = require_list(json.loads(data), context)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourcePreflightError(f"{context} is not valid JSON") from error
    if len(entries) > MAX_ROOT_ENTRIES:
        raise SourcePreflightError(f"{context} root entry resource bound exceeded")
    paths = []
    for index, value in enumerate(entries):
        item = require_dict(value, f"{context} entry {index}")
        if item.get("type") not in {"file", "directory"}:
            raise SourcePreflightError(f"{context} entry type is invalid")
        path = require_string(item.get("path"), f"{context} entry path")
        require_hash(item.get("oid"), f"{context} entry oid", GIT_HASH_LENGTH)
        require_nonnegative_int(item.get("size"), f"{context} entry size")
        paths.append(path)
    if len(paths) != len(set(paths)):
        raise SourcePreflightError(f"{context} root paths are duplicated")
    if len(paths) != source["expected"]["root_entry_count"]:
        raise SourcePreflightError(f"{context} root entry count changed")
    return paths


def final_http_status(data: bytes, context: str) -> int:
    try:
        text = data.decode("iso-8859-1")
    except UnicodeDecodeError as error:
        raise SourcePreflightError(f"{context} headers are invalid") from error
    statuses = [int(match) for match in re.findall(r"(?im)^HTTP/\S+\s+(\d{3})\s*$", text)]
    if not statuses:
        raise SourcePreflightError(f"{context} has no HTTP status")
    return statuses[-1]


def evaluate_source(
    source: dict[str, Any], metadata_directory: Path
) -> tuple[dict[str, Any], int]:
    files = source["metadata_files"]
    loaded: dict[str, bytes] = {}
    for role, name in files.items():
        loaded[role] = read_regular(
            metadata_directory / name, f"{source['source_id']} {role}"
        )
    revision, sibling_paths = parse_revision(
        loaded["revision"], source, f"{source['source_id']} revision"
    )
    root_paths = parse_root_tree(
        loaded["root_tree"], source, f"{source['source_id']} root tree"
    )
    readme_hash = sha256_bytes(loaded["readme"])
    if readme_hash != source["expected"]["readme_sha256"]:
        raise SourcePreflightError(f"{source['source_id']} README hash changed")
    try:
        readme = loaded["readme"].decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourcePreflightError(
            f"{source['source_id']} README is not UTF-8"
        ) from error
    if any(
        literal not in readme
        for literal in source["expected"]["readme_required_literals"]
    ):
        raise SourcePreflightError(f"{source['source_id']} README semantics changed")

    candidate_paths = [
        path
        for path in sibling_paths
        if any(
            path == candidate or path.startswith(f"{candidate}/")
            for candidate in source["expected"]["generation_path_candidates"]
        )
    ]
    gates = {
        "artifact_to_generator_revision_binding": False,
        "bounded_sample_preregistered": True,
        "dataset_payload_remained_closed": True,
        "executable_or_complete_generation_manifest": False,
        "metadata_integrity": True,
        "public_exact_revision": True,
        "upstream_asset_identity": False,
    }
    observations: dict[str, Any] = {
        "generation_candidate_paths": sorted(candidate_paths),
        "readme_sha256": readme_hash,
        "repository_id": revision["id"],
        "revision": revision["sha"],
        "root_entry_count": len(root_paths),
        "sibling_count": len(sibling_paths),
        "sibling_path_root_sha256": sibling_path_root(sibling_paths),
    }

    if source["source_id"] == "nisr-v5":
        status = final_http_status(
            loaded["generation_link_headers"], "NISR generation link"
        )
        if status != source["expected"]["generation_link_http_status"]:
            raise SourcePreflightError("NISR generation-link observation changed")
        observations["declared_generation_link_http_status"] = status
        reason = "DeclaredGenerationDocumentUnavailable"
    else:
        material_hash = sha256_bytes(loaded["material_parameters"])
        if material_hash != source["expected"]["material_parameters_sha256"]:
            raise SourcePreflightError("VibraVerse material table hash changed")
        observations["material_parameters_sha256"] = material_hash
        observations["data_shard_count"] = sum(
            path.endswith(".tar") for path in sibling_paths
        )
        reason = "GenerationLineageIncomplete"

    trusted = all(gates.values())
    decision = TRUSTED_DECISION if trusted else UNTRUSTED_DECISION
    expected = source["expected_decision"]
    if decision != expected["decision"] or reason != expected["reason"]:
        raise SourcePreflightError(f"{source['source_id']} decision changed")
    report = {
        "alias_component": source["alias_component"],
        "authority": AUTHORITY,
        "decision": decision,
        "gates": gates,
        "observations": observations,
        "reason": reason,
        "sample": {
            "accessed": False,
            "maximum_bytes": source["sample_plan"]["maximum_bytes"],
            "selection": source["sample_plan"]["selection"],
            "state": "NotOpenedBecauseGenerationProvenanceFailed",
        },
        "schema": SOURCE_REPORT_SCHEMA,
        "source_id": source["source_id"],
    }
    return report, sum(len(value) for value in loaded.values())


def validate_external_metadata(path: Path, sources: list[dict[str, Any]]) -> None:
    root = repository_root().resolve()
    resolved = path.resolve()
    if path.is_symlink() or not path.is_dir():
        raise SourcePreflightError("metadata must be a regular external directory")
    if resolved == root or root in resolved.parents:
        raise SourcePreflightError("metadata must stay outside the repository")
    expected_names = sorted(
        name for source in sources for name in source["metadata_files"].values()
    )
    if expected_names != sorted(set(expected_names)):
        raise SourcePreflightError("metadata filenames must be globally unique")
    actual_names = sorted(item.name for item in path.iterdir() if item.is_file())
    if actual_names != expected_names:
        raise SourcePreflightError("external metadata inventory changed")
    if any(item.is_symlink() or not item.is_file() for item in path.iterdir()):
        raise SourcePreflightError("external metadata contains a non-regular entry")


def validate_output(path: Path) -> None:
    root = repository_root().resolve()
    resolved = path.resolve()
    if resolved == root or root in resolved.parents:
        raise SourcePreflightError("output must stay outside the repository")
    if path.exists():
        raise SourcePreflightError("output must not already exist")
    if not path.parent.is_dir():
        raise SourcePreflightError("output parent must exist")


def publish(output: Path, documents: dict[str, Any]) -> None:
    validate_output(output)
    temporary = Path(tempfile.mkdtemp(prefix=f".{output.name}-", dir=output.parent))
    try:
        for name, document in sorted(documents.items()):
            (temporary / name).write_bytes(canonical_json(document))
        os.replace(temporary, output)
    except Exception:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def run(profile_path: Path, metadata: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "profile")
    sources = validate_profile(profile)
    validate_external_metadata(metadata, sources)
    validate_output(output)

    source_reports = []
    metadata_bytes_read = 0
    for source in sources:
        report, byte_count = evaluate_source(source, metadata)
        source_reports.append(report)
        metadata_bytes_read += byte_count

    decisions = {
        report["source_id"]: {
            "decision": report["decision"],
            "reason": report["reason"],
        }
        for report in source_reports
    }
    trusted_count = sum(
        report["decision"] == TRUSTED_DECISION for report in source_reports
    )
    counters = {counter: 0 for counter in ZERO_PAYLOAD_COUNTERS}
    counters.update(
        {
            "external_metadata_bytes_read": metadata_bytes_read,
            "external_metadata_files_read": sum(
                len(source["metadata_files"]) for source in sources
            ),
            "metadata_network_requests_declared": profile["access_policy"][
                "metadata_network_requests_declared"
            ],
        }
    )
    access = {
        "counters": counters,
        "dataset_payload_access_authorized": False,
        "schema": ACCESS_SCHEMA,
    }
    gates = {
        "decisions_are_source_separate": len(decisions) == len(sources),
        "expected_source_decisions": decisions
        == profile["expected_result"]["source_decisions"],
        "no_bulk_download": counters["archive_range_bytes_read"] == 0,
        "no_dataset_payload_access": all(
            counters[counter] == 0 for counter in ZERO_PAYLOAD_COUNTERS
        ),
        "no_trusted_teacher_without_complete_provenance": trusted_count == 0,
        "profile_and_owner_bound": True,
    }
    report = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": "NoTrustedSyntheticTeacher",
        "gates": gates,
        "profile": {
            "bytes": len(profile_bytes),
            "sha256": sha256_bytes(profile_bytes),
        },
        "schema": REPORT_SCHEMA,
        "source_decisions": decisions,
        "study_id": STUDY_ID,
        "trusted_source_count": trusted_count,
    }
    if not all(gates.values()):
        raise SourcePreflightError("D0 publication gates failed")
    publish(
        output,
        {
            "access.json": access,
            "report.json": report,
            "sources.json": {
                "schema": SOURCE_REPORT_SCHEMA,
                "sources": source_reports,
            },
        },
    )


def main() -> int:
    arguments = parse_arguments()
    try:
        run(arguments.profile, arguments.metadata, arguments.output)
    except SourcePreflightError as error:
        print(f"physical-sound V46 D0 source preflight failed: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
