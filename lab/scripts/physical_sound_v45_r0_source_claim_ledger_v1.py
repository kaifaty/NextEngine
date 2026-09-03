#!/usr/bin/env python3
"""Build the V45 R0 metadata-only source-to-claim ledger."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v45-r0-profile.v1"
LEDGER_SCHEMA = "nextengine.experimental-physical-sound-v45-r0-source-claim-ledger.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v45-r0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v45-r0-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v45-r0-source-claim-ledger.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v45_r0_source_claim_ledger_v1.py"
DECISION = "R0_SOURCE_CLAIM_LEDGER_REPEATABLE_NO_PAYLOAD_AUTHORITY"
CLAIM = (
    "METADATA_ONLY_SOURCE_TO_CLAIM_PLANNING / NO_PAYLOAD_SIGNAL_FEATURE_MODEL_"
    "TRAINING_VALIDATOR_PROTECTED_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)

HASH_LENGTH = 64
MAX_PROFILE_BYTES = 2 * 1024 * 1024
LANES = (
    "empirical_prior",
    "modal_teacher",
    "protected_admission",
    "real_acoustic",
    "structural_transfer",
    "validator_calibration",
)
FORBIDDEN_AUTHORITIES = (
    "corpus_increment",
    "first_pack_selection",
    "generator_training",
    "protected_admission",
    "runtime_or_product",
    "validator_calibration",
)
FORBIDDEN_COUNTERS = (
    "audio_files_decoded",
    "feature_values_read",
    "model_values_read",
    "network_requests",
    "numeric_payload_values_decoded",
    "pcm_sample_values_decoded",
    "protected_values_read",
    "waveform_bytes_read",
)
EVIDENCE_CLASSES = {
    "empirical_prior_source": ("empirical_prior",),
    "excluded_unavailable": (),
    "literature_control": (),
    "metadata_physics_control": ("empirical_prior",),
    "real_acoustic_candidate": ("real_acoustic", "validator_calibration"),
    "structural_transfer": ("structural_transfer",),
    "synthetic_modal_teacher": ("modal_teacher",),
}
ROLE_STATES = {
    "control_only",
    "eligible_after_preflight",
    "excluded_unavailable",
    "metadata_axis_audit_required",
}
COST_CLASSES = {"metadata_only", "small", "medium", "large", "very_large"}

AUTHORITY = {
    "admission_authority": False,
    "authorizes_only": "v45_t0_recipe_v3_design",
    "corpus_increment_authority": False,
    "model_training_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}
ACCESS_POLICY = {
    "audio_decode_allowed": False,
    "feature_access_allowed": False,
    "model_access_allowed": False,
    "network_allowed_at_build": False,
    "numeric_payload_decode_allowed": False,
    "outputs_external": True,
    "protected_access_allowed": False,
    "waveform_access_allowed": False,
}


class SourceClaimLedgerError(RuntimeError):
    """R0 cannot publish a trustworthy metadata-only ledger."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
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
        raise SourceClaimLedgerError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SourceClaimLedgerError(f"{context} must be an object")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise SourceClaimLedgerError(f"{context} must be a non-empty string")
    return value


def require_nonnegative_int(value: Any, context: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise SourceClaimLedgerError(f"{context} must be a non-negative integer")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise SourceClaimLedgerError(f"{context} must be a lowercase SHA-256")
    return value


def require_sorted_strings(value: Any, context: str) -> list[str]:
    if not isinstance(value, list):
        raise SourceClaimLedgerError(f"{context} must be an array")
    strings = [require_string(item, f"{context} item") for item in value]
    if strings != sorted(set(strings)):
        raise SourceClaimLedgerError(f"{context} must be sorted and unique")
    return strings


def read_regular(path: Path, context: str, maximum: int = MAX_PROFILE_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise SourceClaimLedgerError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise SourceClaimLedgerError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourceClaimLedgerError(f"{context} is not valid UTF-8 JSON") from error
    document = require_dict(value, context)
    if data != canonical_json(document):
        raise SourceClaimLedgerError(f"{context} is not canonical JSON")
    return data, document


def validate_dependency(binding: Any, context: str) -> dict[str, Any]:
    item = require_dict(binding, context)
    if set(item) != {"bytes", "path", "sha256"}:
        raise SourceClaimLedgerError(f"{context} fields changed")
    relative = Path(require_string(item["path"], f"{context} path"))
    if relative.is_absolute() or ".." in relative.parts:
        raise SourceClaimLedgerError(f"{context} path escapes the repository")
    expected_bytes = require_nonnegative_int(item["bytes"], f"{context} bytes")
    if expected_bytes == 0:
        raise SourceClaimLedgerError(f"{context} bytes must be positive")
    expected_hash = require_hash(item["sha256"], f"{context} sha256")
    data = read_regular(repository_root() / relative, context)
    if len(data) != expected_bytes or sha256_bytes(data) != expected_hash:
        raise SourceClaimLedgerError(f"{context} binding mismatch")
    return item


def validate_metadata_binding(binding: Any, context: str) -> dict[str, Any]:
    item = require_dict(binding, context)
    if set(item) != {"algorithm", "digest", "name", "unit_count"}:
        raise SourceClaimLedgerError(f"{context} fields changed")
    if item["algorithm"] != "sha256":
        raise SourceClaimLedgerError(f"{context} algorithm must be sha256")
    require_hash(item["digest"], f"{context} digest")
    require_string(item["name"], f"{context} name")
    if require_nonnegative_int(item["unit_count"], f"{context} unit_count") == 0:
        raise SourceClaimLedgerError(f"{context} unit_count must be positive")
    return item


def validate_source(source: Any, context: str) -> dict[str, Any]:
    item = require_dict(source, context)
    expected_fields = {
        "alias_component",
        "cost_class",
        "evidence_class",
        "forbidden_authorities",
        "metadata_bindings",
        "missing_gate",
        "observed_axes",
        "payload_state",
        "physical_parent_credit",
        "project_independence_credit",
        "prospective_lanes",
        "publisher_uri",
        "revision",
        "role_state",
        "source_id",
        "title",
        "usage_terms",
    }
    if set(item) != expected_fields:
        raise SourceClaimLedgerError(f"{context} fields changed")
    require_string(item["source_id"], f"{context} source_id")
    require_string(item["title"], f"{context} title")
    uri = require_string(item["publisher_uri"], f"{context} publisher_uri")
    if not uri.startswith("https://"):
        raise SourceClaimLedgerError(f"{context} publisher_uri must use https")
    require_string(item["alias_component"], f"{context} alias_component")
    require_string(item["payload_state"], f"{context} payload_state")
    require_string(item["missing_gate"], f"{context} missing_gate")

    revision = require_dict(item["revision"], f"{context} revision")
    if set(revision) != {"kind", "value"}:
        raise SourceClaimLedgerError(f"{context} revision fields changed")
    require_string(revision["kind"], f"{context} revision kind")
    require_string(revision["value"], f"{context} revision value")

    evidence_class = require_string(item["evidence_class"], f"{context} evidence")
    if evidence_class not in EVIDENCE_CLASSES:
        raise SourceClaimLedgerError(f"{context} evidence class is unknown")
    lanes = require_sorted_strings(item["prospective_lanes"], f"{context} lanes")
    if tuple(lanes) != tuple(sorted(EVIDENCE_CLASSES[evidence_class])):
        raise SourceClaimLedgerError(f"{context} lanes exceed the evidence class")
    if any(lane not in LANES for lane in lanes):
        raise SourceClaimLedgerError(f"{context} lane is unknown")
    require_sorted_strings(item["observed_axes"], f"{context} observed_axes")
    forbidden = require_sorted_strings(
        item["forbidden_authorities"], f"{context} forbidden_authorities"
    )
    if tuple(forbidden) != FORBIDDEN_AUTHORITIES:
        raise SourceClaimLedgerError(f"{context} must forbid every R0 authority")

    bindings = item["metadata_bindings"]
    if not isinstance(bindings, list):
        raise SourceClaimLedgerError(f"{context} metadata_bindings must be an array")
    checked_bindings = [
        validate_metadata_binding(binding, f"{context} metadata binding {index}")
        for index, binding in enumerate(bindings)
    ]
    binding_names = [binding["name"] for binding in checked_bindings]
    if binding_names != sorted(set(binding_names)):
        raise SourceClaimLedgerError(
            f"{context} metadata binding names must be sorted and unique"
        )
    if evidence_class not in {"excluded_unavailable", "literature_control"} and not bindings:
        raise SourceClaimLedgerError(f"{context} requires a metadata hash root")

    role_state = require_string(item["role_state"], f"{context} role_state")
    if role_state not in ROLE_STATES:
        raise SourceClaimLedgerError(f"{context} role_state is unknown")
    cost_class = require_string(item["cost_class"], f"{context} cost_class")
    if cost_class not in COST_CLASSES:
        raise SourceClaimLedgerError(f"{context} cost_class is unknown")
    if require_nonnegative_int(
        item["physical_parent_credit"], f"{context} physical_parent_credit"
    ) != 0:
        raise SourceClaimLedgerError(f"{context} grants premature parent credit")
    if require_nonnegative_int(
        item["project_independence_credit"],
        f"{context} project_independence_credit",
    ) != 0:
        raise SourceClaimLedgerError(f"{context} grants premature project credit")

    usage = require_dict(item["usage_terms"], f"{context} usage_terms")
    if set(usage) != {"expression", "retention_policy"}:
        raise SourceClaimLedgerError(f"{context} usage_terms fields changed")
    require_string(usage["expression"], f"{context} usage expression")
    require_string(usage["retention_policy"], f"{context} retention policy")
    return item


def validate_profile(profile: dict[str, Any]) -> list[dict[str, Any]]:
    expected_fields = {
        "access_policy",
        "authority",
        "claim",
        "claim_policy",
        "decision",
        "dependency_bindings",
        "expected_result",
        "schema",
        "sources",
    }
    if set(profile) != expected_fields:
        raise SourceClaimLedgerError("profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA:
        raise SourceClaimLedgerError("profile schema changed")
    if profile["claim"] != CLAIM or profile["decision"] != DECISION:
        raise SourceClaimLedgerError("profile claim or decision changed")
    if profile["authority"] != AUTHORITY:
        raise SourceClaimLedgerError("profile authority changed")
    if profile["access_policy"] != ACCESS_POLICY:
        raise SourceClaimLedgerError("profile access policy changed")

    claim_policy = require_dict(profile["claim_policy"], "claim policy")
    if set(claim_policy) != {
        "lane_order",
        "missing_targets_contribute_loss",
        "source_lanes_are_interchangeable",
    }:
        raise SourceClaimLedgerError("claim policy fields changed")
    if tuple(claim_policy["lane_order"]) != LANES:
        raise SourceClaimLedgerError("claim policy lane order changed")
    if claim_policy["missing_targets_contribute_loss"] is not False:
        raise SourceClaimLedgerError("missing targets must not contribute loss")
    if claim_policy["source_lanes_are_interchangeable"] is not False:
        raise SourceClaimLedgerError("source lanes must not be interchangeable")

    dependencies = profile["dependency_bindings"]
    if not isinstance(dependencies, list) or not dependencies:
        raise SourceClaimLedgerError("dependency bindings must be a non-empty array")
    checked_dependencies = [
        validate_dependency(binding, f"dependency {index}")
        for index, binding in enumerate(dependencies)
    ]
    dependency_paths = [binding["path"] for binding in checked_dependencies]
    if dependency_paths != sorted(set(dependency_paths)):
        raise SourceClaimLedgerError("dependency paths must be sorted and unique")
    if OWNER_PATH not in dependency_paths:
        raise SourceClaimLedgerError("profile does not bind its owner")

    raw_sources = profile["sources"]
    if not isinstance(raw_sources, list) or not raw_sources:
        raise SourceClaimLedgerError("sources must be a non-empty array")
    sources = [
        validate_source(source, f"source {index}")
        for index, source in enumerate(raw_sources)
    ]
    source_ids = [source["source_id"] for source in sources]
    if source_ids != sorted(set(source_ids)):
        raise SourceClaimLedgerError("source IDs must be sorted and unique")

    observed = compute_result(sources)
    if profile["expected_result"] != observed:
        raise SourceClaimLedgerError("expected result does not match source census")
    return sources


def compute_result(sources: list[dict[str, Any]]) -> dict[str, Any]:
    evidence_counts = Counter(source["evidence_class"] for source in sources)
    role_counts = Counter(source["role_state"] for source in sources)
    lane_counts: Counter[str] = Counter()
    for source in sources:
        lane_counts.update(source["prospective_lanes"])
    return {
        "decision": DECISION,
        "evidence_class_counts": dict(sorted(evidence_counts.items())),
        "physical_parent_credit": sum(
            source["physical_parent_credit"] for source in sources
        ),
        "project_independence_credit": sum(
            source["project_independence_credit"] for source in sources
        ),
        "prospective_lane_counts": {
            lane: lane_counts.get(lane, 0) for lane in LANES
        },
        "role_state_counts": dict(sorted(role_counts.items())),
        "source_count": len(sources),
    }


def is_within(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def prepare_output(output: Path) -> Path:
    resolved = output.resolve(strict=False)
    if is_within(resolved, repository_root().resolve()):
        raise SourceClaimLedgerError("output must remain outside the repository")
    if output.exists() or output.is_symlink():
        raise SourceClaimLedgerError("output path already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    return Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))


def run(profile_path: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "profile")
    sources = validate_profile(profile)
    stage = prepare_output(output)
    try:
        source_rows = []
        for source in sources:
            row = dict(source)
            row["source_claim_sha256"] = sha256_bytes(canonical_json(source))
            source_rows.append(row)
        ledger = {
            "claim_policy": profile["claim_policy"],
            "profile_sha256": sha256_bytes(profile_bytes),
            "schema": LEDGER_SCHEMA,
            "sources": source_rows,
        }
        access = {
            "counters": {counter: 0 for counter in FORBIDDEN_COUNTERS},
            "policy": ACCESS_POLICY,
            "schema": ACCESS_SCHEMA,
        }
        result = compute_result(sources)
        report = {
            "authority": AUTHORITY,
            "claim": CLAIM,
            "decision": DECISION,
            "gates": {
                "alias_components_explicit": all(
                    bool(source["alias_component"]) for source in sources
                ),
                "all_r0_authorities_forbidden": all(
                    tuple(source["forbidden_authorities"])
                    == FORBIDDEN_AUTHORITIES
                    for source in sources
                ),
                "claim_lanes_partitioned": True,
                "external_payload_access_zero": True,
                "parent_and_project_credit_zero": (
                    result["physical_parent_credit"] == 0
                    and result["project_independence_credit"] == 0
                ),
                "protected_lane_empty": (
                    result["prospective_lane_counts"]["protected_admission"] == 0
                ),
                "source_census_matches": result == profile["expected_result"],
            },
            "result": result,
            "schema": REPORT_SCHEMA,
        }
        (stage / "access.json").write_bytes(canonical_json(access))
        (stage / "ledger.json").write_bytes(canonical_json(ledger))
        (stage / "report.json").write_bytes(canonical_json(report))
        os.replace(stage, output)
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def main() -> None:
    arguments = parse_arguments()
    run(arguments.profile, arguments.output)


if __name__ == "__main__":
    main()
