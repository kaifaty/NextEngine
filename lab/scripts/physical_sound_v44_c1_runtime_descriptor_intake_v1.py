#!/usr/bin/env python3
"""Freeze the V44 C1 runtime-descriptor contract and metadata-only intake."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-intake-profile.v1"
CONTRACT_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-c1-runtime-descriptor-contract.v1"
)
SOURCE_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-descriptor-source.v1"
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-role-projection.v1"
RECORDS_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-descriptor-records.v1"
PROVENANCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-c1-descriptor-provenance.v1"
)
COVERAGE_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-descriptor-coverage.v1"
SOURCE_INVENTORY_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-c1-source-inventory.v1"
)
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-report.v1"

TRAIN_ROLE = "generator_train"
DEVELOPMENT_ROLE = "generator_development"
ROLES = (DEVELOPMENT_ROLE, TRAIN_ROLE)
UNKNOWN_MATERIAL = "Unknown"
DECISION = "C1_DESCRIPTOR_CONTRACT_FROZEN_G0_AUTHORIZED"
MAX_JSON_BYTES = 32 * 1024 * 1024
HASH_LENGTH = 64

FIELD_ORDER = (
    "material_label",
    "shape_topology",
    "extent_micrometres",
    "characteristic_wall_thickness_micrometres",
    "cavity_kind",
    "opening_kind",
    "mass_milligrams",
    "support_condition",
    "impact_zone",
)
OBJECT_FIELDS = frozenset(
    {
        "material_label",
        "shape_topology",
        "extent_micrometres",
        "characteristic_wall_thickness_micrometres",
        "cavity_kind",
        "opening_kind",
        "mass_milligrams",
    }
)
CONDITION_FIELDS = frozenset({"support_condition", "impact_zone"})
EXPECTED_FIELD_SHAPES = {
    "cavity_kind": ("physical_parent", "enum", "enum"),
    "characteristic_wall_thickness_micrometres": (
        "physical_parent",
        "integer",
        "micrometre",
    ),
    "extent_micrometres": ("physical_parent", "integer_vector", "micrometre"),
    "impact_zone": ("record", "enum", "enum"),
    "mass_milligrams": ("physical_parent", "integer", "milligram"),
    "material_label": ("physical_parent", "enum", "enum"),
    "opening_kind": ("physical_parent", "enum", "enum"),
    "shape_topology": ("physical_parent", "enum", "enum"),
    "support_condition": ("record", "enum", "enum"),
}
FORBIDDEN_MODEL_TOKENS = frozenset(
    {
        "artifact",
        "audio",
        "dataset",
        "evidence",
        "family",
        "file",
        "filename",
        "network",
        "path",
        "project",
        "provenance",
        "publisher",
        "record_id",
        "source",
        "url",
        "waveform",
    }
)

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": ["v44_g0_descriptor_source_growth"],
    "candidate_training_authority": False,
    "descriptor_contract_freeze_authority": True,
    "descriptor_intake_authority": True,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}
CONTRACT_AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "experimental_lab_contract": True,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_calibration_authority": False,
}
CONFIDENCE_POLICY = {
    "maximum_ppm": 1000000,
    "minimum_ppm": 0,
    "missing_confidence_mask": False,
    "missing_confidence_sentinel_ppm": 0,
    "semantics": "source_reported_or_adapter_derived_confidence_not_truth_probability",
}
OUTPUT_POLICY = {
    "maximum_artifact_bytes": 33554432,
    "maximum_artifact_bytes_per_source": 67108864,
    "maximum_artifacts_per_source": 256,
    "maximum_source_inputs": 64,
    "publication": "atomic_external_directory_only",
}


class DescriptorIntakeError(RuntimeError):
    """C1 cannot publish a trustworthy runtime-descriptor surface."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--source-root", action="append", default=[], type=Path)
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
        raise DescriptorIntakeError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise DescriptorIntakeError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise DescriptorIntakeError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise DescriptorIntakeError(f"{context} must be a non-empty string")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise DescriptorIntakeError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_JSON_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise DescriptorIntakeError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise DescriptorIntakeError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise DescriptorIntakeError(f"{context} is not valid UTF-8 JSON") from error
    return require_dict(value, context)


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise DescriptorIntakeError(f"{context} binding fields changed")
    relative = Path(require_string(binding["path"], f"{context} path"))
    if relative.is_absolute() or ".." in relative.parts:
        raise DescriptorIntakeError(f"{context} path escapes its root")
    expected_bytes = binding["bytes"]
    if type(expected_bytes) is not int or expected_bytes <= 0:
        raise DescriptorIntakeError(f"{context} byte count is invalid")
    require_hash(binding["sha256"], f"{context} SHA-256")
    data = read_regular(path, context)
    if len(data) != expected_bytes or sha256_bytes(data) != binding["sha256"]:
        raise DescriptorIntakeError(f"{context} bytes or SHA-256 changed")
    return data


def load_bound_json(
    root: Path, binding: dict[str, Any], context: str
) -> tuple[bytes, dict[str, Any]]:
    data = check_binding(root / binding["path"], binding, context)
    value = parse_json(data, context)
    if canonical_json(value) != data:
        raise DescriptorIntakeError(f"{context} must remain canonical")
    return data, value


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    value = parse_json(data, context)
    if canonical_json(value) != data:
        raise DescriptorIntakeError(f"{context} must remain canonical")
    return data, value


def validate_dependencies(profile: dict[str, Any]) -> None:
    bindings = require_list(profile.get("dependency_bindings"), "dependencies")
    paths = []
    for raw in bindings:
        binding = require_dict(raw, "dependency")
        path_text = require_string(binding.get("path"), "dependency path")
        paths.append(path_text)
        check_binding(
            repository_root() / path_text,
            binding,
            f"dependency {path_text}",
        )
    if paths != sorted(set(paths)):
        raise DescriptorIntakeError("dependencies must be path-sorted and unique")


def validate_contract(contract: dict[str, Any]) -> dict[str, Any]:
    required = {
        "authority",
        "confidence_policy",
        "contract_id",
        "evidence_source_contract",
        "field_order",
        "fields",
        "missingness_policy",
        "model_input_policy",
        "schema",
    }
    if set(contract) != required or contract.get("schema") != CONTRACT_SCHEMA:
        raise DescriptorIntakeError("descriptor contract fields or schema changed")
    if tuple(contract["field_order"]) != FIELD_ORDER:
        raise DescriptorIntakeError("descriptor field order changed")
    if contract.get("authority") != CONTRACT_AUTHORITY:
        raise DescriptorIntakeError("descriptor contract authority changed")
    if contract.get("confidence_policy") != CONFIDENCE_POLICY:
        raise DescriptorIntakeError("descriptor confidence policy changed")
    fields = require_dict(contract["fields"], "descriptor fields")
    if set(fields) != set(FIELD_ORDER):
        raise DescriptorIntakeError("descriptor field set changed")
    for name, (scope, kind, unit) in EXPECTED_FIELD_SHAPES.items():
        spec = require_dict(fields[name], f"field {name}")
        if (
            spec.get("scope") != scope
            or spec.get("value_kind") != kind
            or spec.get("unit") != unit
        ):
            raise DescriptorIntakeError(f"descriptor field {name} shape changed")
        require_string(spec.get("semantics"), f"field {name} semantics")
        if kind == "enum":
            allowed = require_list(spec.get("allowed_values"), f"field {name} enum")
            if not allowed or allowed != sorted(set(allowed)):
                raise DescriptorIntakeError(f"field {name} enum is not canonical")
        elif kind == "integer":
            if (
                type(spec.get("minimum")) is not int
                or type(spec.get("maximum")) is not int
                or spec["minimum"] > spec["maximum"]
            ):
                raise DescriptorIntakeError(f"field {name} bounds are invalid")
        elif kind == "integer_vector":
            if (
                type(spec.get("length")) is not int
                or spec["length"] <= 0
                or type(spec.get("item_minimum")) is not int
                or type(spec.get("item_maximum")) is not int
                or spec["item_minimum"] > spec["item_maximum"]
            ):
                raise DescriptorIntakeError(f"field {name} vector bounds are invalid")
    evidence = require_dict(
        contract["evidence_source_contract"], "evidence source contract"
    )
    if (
        evidence.get("manifest_schema") != SOURCE_SCHEMA
        or set(evidence.get("object_fields_require_null_record_id", []))
        != OBJECT_FIELDS
        or set(evidence.get("condition_fields_require_record_id", []))
        != CONDITION_FIELDS
        or evidence.get("artifact_path_policy")
        != "relative_regular_non_symlink_hash_bound"
        or evidence.get("artifact_url_policy") != "https_only"
        or evidence.get("source_conflict_policy")
        != "different_values_publish_masked_source_conflict_never_select_by_order"
        or evidence.get("material_conflict_policy")
        != "c0r_material_disagreement_rejects_and_c0r_quarantine_cannot_be_overridden"
        or set(evidence.get("forbidden_artifact_media_prefixes", []))
        != {"audio/", "video/"}
        or set(evidence.get("redistribution_policies", []))
        != {"external_research_only", "redistributable_with_notice"}
    ):
        raise DescriptorIntakeError("evidence source field scopes changed")
    model = require_dict(contract["model_input_policy"], "model input policy")
    if (
        set(model.get("allowed_keys", []))
        != {"confidence_mask", "confidence_ppm", "observed_mask", "values"}
        or set(model.get("forbidden_key_tokens", [])) != FORBIDDEN_MODEL_TOKENS
        or not model.get("project_and_capture_identity_forbidden")
    ):
        raise DescriptorIntakeError("model-input isolation policy changed")
    missing = require_dict(contract["missingness_policy"], "missingness policy")
    if set(missing.get("allowed_reasons", [])) != {
        "not_applicable",
        "not_published",
        "outside_contract",
        "source_conflict",
    } or not missing.get(
        "unknown_axes_are_never_inferred_from_audio_dataset_name_filename_or_project_identity"
    ):
        raise DescriptorIntakeError("missingness policy changed")
    return contract


def validate_profile(profile: dict[str, Any]) -> dict[str, Any]:
    required = {
        "authority",
        "contract_binding",
        "corpus_identity",
        "dependency_bindings",
        "expected_counts",
        "input_bindings",
        "output_policy",
        "priority_materials",
        "profile_id",
        "source_inputs",
        "schema",
    }
    if set(profile) != required or profile.get("schema") != PROFILE_SCHEMA:
        raise DescriptorIntakeError("C1 profile fields or schema changed")
    if profile.get("authority") != AUTHORITY:
        raise DescriptorIntakeError("C1 authority changed")
    if profile.get("output_policy") != OUTPUT_POLICY:
        raise DescriptorIntakeError("C1 output/resource policy changed")
    require_string(profile.get("profile_id"), "profile ID")
    validate_dependencies(profile)
    contract_binding = require_dict(profile["contract_binding"], "contract binding")
    if set(contract_binding) != {"bytes", "path", "sha256"}:
        raise DescriptorIntakeError("contract binding fields changed")
    contract_path = Path(require_string(contract_binding.get("path"), "contract path"))
    if contract_path.is_absolute() or ".." in contract_path.parts:
        raise DescriptorIntakeError("contract path escapes the repository")
    source_inputs = require_list(profile["source_inputs"], "source inputs")
    if len(source_inputs) > OUTPUT_POLICY["maximum_source_inputs"]:
        raise DescriptorIntakeError("too many descriptor source inputs")
    names = []
    for raw in source_inputs:
        source = require_dict(raw, "source input")
        if set(source) != {"manifest", "root_name"}:
            raise DescriptorIntakeError("source input fields changed")
        name = require_string(source["root_name"], "source root name")
        if Path(name).name != name:
            raise DescriptorIntakeError("source root name must be one path component")
        manifest = require_dict(source["manifest"], "source manifest binding")
        if set(manifest) != {"bytes", "path", "sha256"}:
            raise DescriptorIntakeError("source manifest binding fields changed")
        if manifest.get("path") != "manifest.json":
            raise DescriptorIntakeError("source manifest path must be manifest.json")
        require_hash(manifest.get("sha256"), "source manifest SHA-256")
        if type(manifest.get("bytes")) is not int or manifest["bytes"] <= 0:
            raise DescriptorIntakeError("source manifest byte count is invalid")
        names.append(name)
    if names != sorted(set(names)):
        raise DescriptorIntakeError("source inputs must be root-name sorted and unique")
    expected = require_dict(profile["expected_counts"], "expected counts")
    if set(expected) != {
        "corpus_parents",
        "corpus_records",
        "descriptor_complete_parents",
        "descriptor_complete_records",
        "material_observed_parents",
        "material_observed_records",
        "source_artifacts",
        "source_inputs",
        "source_observations",
    }:
        raise DescriptorIntakeError("expected count fields changed")
    for key, value in expected.items():
        if type(value) is not int or value < 0:
            raise DescriptorIntakeError(f"expected count {key} is invalid")
    if profile.get("priority_materials") != ["Glass", "Steel", "Wood"]:
        raise DescriptorIntakeError("priority material order changed")
    if set(require_dict(profile["input_bindings"], "C0R input bindings")) != {
        "corpus_profile",
        "corpus_report",
        "development_projection",
        "lineage_repair",
        "train_projection",
    }:
        raise DescriptorIntakeError("C0R input binding set changed")
    identity = require_dict(profile["corpus_identity"], "C0R identity")
    if set(identity) != {
        "manifest_root_sha256",
        "report_decision",
        "report_schema",
    }:
        raise DescriptorIntakeError("C0R identity fields changed")
    require_hash(identity["manifest_root_sha256"], "C0R manifest root")
    return profile


def load_contract(profile: dict[str, Any]) -> tuple[bytes, dict[str, Any]]:
    binding = require_dict(profile["contract_binding"], "contract binding")
    path = repository_root() / binding["path"]
    data = check_binding(path, binding, "descriptor contract")
    value = parse_json(data, "descriptor contract")
    if canonical_json(value) != data:
        raise DescriptorIntakeError("descriptor contract must remain canonical")
    return data, validate_contract(value)


def prepare_root(path: Path, context: str) -> Path:
    if path.is_symlink() or not path.is_dir():
        raise DescriptorIntakeError(f"{context} must be a non-symlink directory")
    return path.resolve()


def load_corpus_inputs(
    root: Path, profile: dict[str, Any]
) -> tuple[int, dict[str, dict[str, Any]], list[dict[str, Any]]]:
    bindings = require_dict(profile["input_bindings"], "C0R input bindings")
    values: dict[str, dict[str, Any]] = {}
    total = 0
    for name in (
        "corpus_profile",
        "corpus_report",
        "lineage_repair",
        "train_projection",
        "development_projection",
    ):
        binding = require_dict(bindings.get(name), f"C0R {name} binding")
        data, value = load_bound_json(root, binding, f"C0R {name}")
        values[name] = value
        total += len(data)
    identity = require_dict(profile["corpus_identity"], "C0R identity")
    report = values["corpus_report"]
    if (
        report.get("schema") != identity.get("report_schema")
        or report.get("decision") != identity.get("report_decision")
        or require_dict(report.get("artifacts"), "C0R report artifacts").get(
            "manifest_root_sha256"
        )
        != identity.get("manifest_root_sha256")
    ):
        raise DescriptorIntakeError("C0R report identity changed")
    if (
        values["corpus_profile"].get("schema")
        != "nextengine.experimental-physical-sound-v43-d2-c0r-profile.v1"
        or values["lineage_repair"].get("schema")
        != "nextengine.experimental-physical-sound-v43-d2-repair-record.v1"
    ):
        raise DescriptorIntakeError("C0R repair/profile identity changed")

    rows = []
    for name, role in (
        ("train_projection", TRAIN_ROLE),
        ("development_projection", DEVELOPMENT_ROLE),
    ):
        projection = values[name]
        projection_rows = require_list(projection.get("rows"), f"C0R {role} rows")
        if (
            projection.get("schema") != PROJECTION_SCHEMA
            or projection.get("role") != role
            or projection.get("record_count") != len(projection_rows)
            or projection.get("rows_root_sha256")
            != sha256_bytes(canonical_json(projection_rows))
        ):
            raise DescriptorIntakeError(f"C0R {role} projection changed")
        for raw in projection_rows:
            row = require_dict(raw, "C0R projection row")
            if set(row) != {
                "acoustic_target",
                "axis_mask",
                "canonical_pcm",
                "family_component_id",
                "family_id",
                "material_label",
                "physical_parent_id",
                "record_id",
            }:
                raise DescriptorIntakeError("C0R projection row fields changed")
            rows.append({**row, "role": role})
    validate_corpus_rows(rows, profile)
    return total, values, rows


def validate_corpus_rows(rows: list[dict[str, Any]], profile: dict[str, Any]) -> None:
    expected = profile["expected_counts"]
    if len(rows) != expected["corpus_records"]:
        raise DescriptorIntakeError("C0R record count changed")
    ids = [require_string(row.get("record_id"), "record ID") for row in rows]
    if len(ids) != len(set(ids)):
        raise DescriptorIntakeError("C0R record IDs overlap")
    parent_roles: dict[str, set[str]] = defaultdict(set)
    component_roles: dict[str, set[str]] = defaultdict(set)
    parent_materials: dict[str, set[tuple[str, bool]]] = defaultdict(set)
    for row in rows:
        parent = require_string(row.get("physical_parent_id"), "physical parent ID")
        family = require_string(row.get("family_id"), "source family ID")
        component = require_string(
            row.get("family_component_id"), "source component ID"
        )
        if not family:
            raise DescriptorIntakeError("source family ID is empty")
        axes = require_dict(row.get("axis_mask"), "C0R axis mask")
        observed = axes.get("material_identity")
        if type(observed) is not bool:
            raise DescriptorIntakeError("C0R material mask is not boolean")
        material = row.get("material_label")
        if observed:
            require_string(material, "C0R material label")
            if material == UNKNOWN_MATERIAL:
                raise DescriptorIntakeError("observed C0R material is Unknown")
        elif material != UNKNOWN_MATERIAL:
            raise DescriptorIntakeError("unobserved C0R material is not Unknown")
        parent_roles[parent].add(row["role"])
        component_roles[component].add(row["role"])
        parent_materials[parent].add((material, observed))
    if any(len(roles) != 1 for roles in parent_roles.values()):
        raise DescriptorIntakeError("physical parent crosses generator roles")
    if any(len(roles) != 1 for roles in component_roles.values()):
        raise DescriptorIntakeError("source component crosses generator roles")
    if any(len(states) != 1 for states in parent_materials.values()):
        raise DescriptorIntakeError("physical parent material state is inconsistent")
    if len(parent_roles) != expected["corpus_parents"]:
        raise DescriptorIntakeError("C0R parent count changed")
    observed_records = sum(
        require_dict(row["axis_mask"], "C0R axis mask")["material_identity"]
        for row in rows
    )
    observed_parents = sum(
        next(iter(states))[1] for states in parent_materials.values()
    )
    if (
        observed_records != expected["material_observed_records"]
        or observed_parents != expected["material_observed_parents"]
    ):
        raise DescriptorIntakeError("C0R material-observed counts changed")


def validate_field_value(name: str, value: Any, contract: dict[str, Any]) -> Any:
    spec = contract["fields"][name]
    kind = spec["value_kind"]
    if kind == "enum":
        if not isinstance(value, str) or value not in spec["allowed_values"]:
            raise DescriptorIntakeError(f"field {name} enum value is invalid")
        return value
    if kind == "integer":
        if type(value) is not int or value < spec["minimum"] or value > spec["maximum"]:
            raise DescriptorIntakeError(f"field {name} integer value is invalid")
        return value
    if kind == "integer_vector":
        if not isinstance(value, list) or len(value) != spec["length"]:
            raise DescriptorIntakeError(f"field {name} vector shape is invalid")
        if any(
            type(item) is not int
            or item < spec["item_minimum"]
            or item > spec["item_maximum"]
            for item in value
        ):
            raise DescriptorIntakeError(f"field {name} vector item is invalid")
        return list(value)
    raise DescriptorIntakeError(f"field {name} has unknown value kind")


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def source_roots_by_name(roots: list[Path], profile: dict[str, Any]) -> dict[str, Path]:
    expected = {entry["root_name"]: entry for entry in profile["source_inputs"]}
    supplied: dict[str, Path] = {}
    for root in roots:
        prepared = prepare_root(root, "descriptor source root")
        name = prepared.name
        if name in supplied:
            raise DescriptorIntakeError("descriptor source root names overlap")
        supplied[name] = prepared
    if set(supplied) != set(expected):
        raise DescriptorIntakeError("descriptor source roots do not match profile")
    return supplied


def validate_source_manifest(
    root_name: str,
    root: Path,
    source_input: dict[str, Any],
    contract: dict[str, Any],
    parent_records: dict[str, set[str]],
    base_material: dict[str, str | None],
    quarantined_material_parents: set[str],
) -> tuple[int, int, dict[str, Any], list[dict[str, Any]]]:
    manifest_bytes, manifest = load_bound_json(
        root, source_input["manifest"], f"descriptor source {root_name} manifest"
    )
    required = {
        "artifacts",
        "declared_revision",
        "landing_page_url",
        "license_expression",
        "observations",
        "project_id",
        "publisher_id",
        "redistribution_policy",
        "schema",
        "source_component_id",
        "source_id",
        "terms_url",
    }
    if set(manifest) != required or manifest.get("schema") != SOURCE_SCHEMA:
        raise DescriptorIntakeError("descriptor source manifest fields changed")
    for key in (
        "declared_revision",
        "license_expression",
        "project_id",
        "publisher_id",
        "source_component_id",
        "source_id",
    ):
        require_string(manifest.get(key), f"descriptor source {key}")
    if (
        manifest["redistribution_policy"]
        not in contract["evidence_source_contract"]["redistribution_policies"]
    ):
        raise DescriptorIntakeError("descriptor source redistribution policy changed")
    for key in ("landing_page_url", "terms_url"):
        value = manifest[key]
        if value is not None and (
            not isinstance(value, str) or not value.startswith("https://")
        ):
            raise DescriptorIntakeError(f"descriptor source {key} must use HTTPS")
    if manifest["landing_page_url"] is None:
        raise DescriptorIntakeError("descriptor source landing page is required")

    media_types = set(contract["evidence_source_contract"]["artifact_media_types"])
    artifacts: dict[str, dict[str, Any]] = {}
    artifact_bytes = 0
    for raw in require_list(manifest["artifacts"], "descriptor source artifacts"):
        artifact = require_dict(raw, "descriptor source artifact")
        if set(artifact) != {
            "artifact_id",
            "bytes",
            "media_type",
            "path",
            "sha256",
            "source_url",
        }:
            raise DescriptorIntakeError("descriptor source artifact fields changed")
        artifact_id = require_string(artifact["artifact_id"], "artifact ID")
        if artifact_id in artifacts:
            raise DescriptorIntakeError("descriptor artifact IDs overlap")
        media_type = require_string(artifact["media_type"], "artifact media type")
        if media_type not in media_types or any(
            media_type.startswith(prefix)
            for prefix in contract["evidence_source_contract"][
                "forbidden_artifact_media_prefixes"
            ]
        ):
            raise DescriptorIntakeError("descriptor artifact media type is forbidden")
        url = require_string(artifact["source_url"], "artifact source URL")
        if not url.startswith("https://"):
            raise DescriptorIntakeError("descriptor artifact URL must use HTTPS")
        relative = Path(require_string(artifact["path"], "artifact path"))
        if relative.is_absolute() or ".." in relative.parts:
            raise DescriptorIntakeError("descriptor artifact path escapes source root")
        artifact_binding = {
            "bytes": artifact["bytes"],
            "path": artifact["path"],
            "sha256": artifact["sha256"],
        }
        artifact_path = root / relative
        if has_symlink_component(
            artifact_path
        ) or not artifact_path.resolve().is_relative_to(root):
            raise DescriptorIntakeError(
                "descriptor artifact path escapes through symlink"
            )
        data = check_binding(
            artifact_path, artifact_binding, f"descriptor artifact {artifact_id}"
        )
        artifact_bytes += len(data)
        artifacts[artifact_id] = artifact
    if not artifacts:
        raise DescriptorIntakeError("descriptor source has no evidence artifacts")
    if (
        len(artifacts) > OUTPUT_POLICY["maximum_artifacts_per_source"]
        or artifact_bytes > OUTPUT_POLICY["maximum_artifact_bytes_per_source"]
    ):
        raise DescriptorIntakeError("descriptor source artifact budget exceeded")

    observations = []
    observation_ids = set()
    for raw in require_list(manifest["observations"], "descriptor observations"):
        observation = require_dict(raw, "descriptor observation")
        if set(observation) != {
            "fields",
            "observation_id",
            "physical_parent_id",
            "record_id",
        }:
            raise DescriptorIntakeError("descriptor observation fields changed")
        observation_id = require_string(observation["observation_id"], "observation ID")
        if observation_id in observation_ids:
            raise DescriptorIntakeError("descriptor observation IDs overlap")
        observation_ids.add(observation_id)
        parent_id = require_string(
            observation["physical_parent_id"], "observation physical parent"
        )
        if parent_id not in parent_records:
            raise DescriptorIntakeError("descriptor observation parent is outside C0R")
        record_id = observation["record_id"]
        if record_id is not None and (
            not isinstance(record_id, str) or record_id not in parent_records[parent_id]
        ):
            raise DescriptorIntakeError(
                "descriptor observation record is outside parent"
            )
        fields = require_dict(observation["fields"], "observation fields")
        if not fields or not set(fields).issubset(FIELD_ORDER):
            raise DescriptorIntakeError("descriptor observation field set is invalid")
        normalized_fields = {}
        for name, raw_claim in fields.items():
            if (name in OBJECT_FIELDS and record_id is not None) or (
                name in CONDITION_FIELDS and record_id is None
            ):
                raise DescriptorIntakeError(
                    "descriptor observation field scope is invalid"
                )
            claim = require_dict(raw_claim, f"observation field {name}")
            if set(claim) != {
                "confidence_observed",
                "confidence_ppm",
                "evidence_artifact_ids",
                "value",
            }:
                raise DescriptorIntakeError("descriptor field claim shape changed")
            value = validate_field_value(name, claim["value"], contract)
            confidence_observed = claim["confidence_observed"]
            confidence = claim["confidence_ppm"]
            if type(confidence_observed) is not bool:
                raise DescriptorIntakeError("confidence mask is not boolean")
            if confidence_observed:
                policy = contract["confidence_policy"]
                if (
                    type(confidence) is not int
                    or confidence < policy["minimum_ppm"]
                    or confidence > policy["maximum_ppm"]
                ):
                    raise DescriptorIntakeError("observed confidence is invalid")
            elif confidence is not None:
                raise DescriptorIntakeError("masked confidence must be null")
            evidence_ids = require_list(
                claim["evidence_artifact_ids"], "field evidence artifact IDs"
            )
            if (
                not evidence_ids
                or evidence_ids != sorted(set(evidence_ids))
                or not all(item in artifacts for item in evidence_ids)
            ):
                raise DescriptorIntakeError("field evidence references are invalid")
            if name == "material_label":
                if parent_id in quarantined_material_parents:
                    raise DescriptorIntakeError(
                        "C0R material quarantine cannot be overridden in C1"
                    )
                if value != base_material[parent_id]:
                    raise DescriptorIntakeError(
                        "descriptor material disagrees with C0R material"
                    )
            normalized_fields[name] = {
                "confidence_observed": confidence_observed,
                "confidence_ppm": confidence,
                "evidence_artifact_ids": evidence_ids,
                "value": value,
            }
        observations.append(
            {
                "fields": normalized_fields,
                "observation_id": observation_id,
                "physical_parent_id": parent_id,
                "record_id": record_id,
            }
        )
    if not observations:
        raise DescriptorIntakeError("descriptor source has no observations")
    inventory = {
        "artifact_count": len(artifacts),
        "artifact_payload_bytes": artifact_bytes,
        "declared_revision": manifest["declared_revision"],
        "landing_page_url": manifest["landing_page_url"],
        "license_expression": manifest["license_expression"],
        "manifest_bytes": len(manifest_bytes),
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "observation_count": len(observations),
        "project_id": manifest["project_id"],
        "publisher_id": manifest["publisher_id"],
        "redistribution_policy": manifest["redistribution_policy"],
        "root_name": root_name,
        "source_component_id": manifest["source_component_id"],
        "source_id": manifest["source_id"],
        "terms_url": manifest["terms_url"],
    }
    return len(manifest_bytes), artifact_bytes, inventory, observations


def base_material_state(
    rows: list[dict[str, Any]], projection_hashes: dict[str, str]
) -> tuple[
    dict[str, str | None],
    set[str],
    dict[str, dict[str, Any]],
    dict[tuple[str, str | None, str], list[dict[str, Any]]],
]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        grouped[row["physical_parent_id"]].append(row)
    materials: dict[str, str | None] = {}
    quarantined = set()
    evidence = {}
    candidates: dict[tuple[str, str | None, str], list[dict[str, Any]]] = defaultdict(
        list
    )
    for parent_id in sorted(grouped):
        members = grouped[parent_id]
        material, observed = next(
            iter(
                {
                    (row["material_label"], row["axis_mask"]["material_identity"])
                    for row in members
                }
            )
        )
        evidence_id = f"c0r:parent:{parent_id}:material_label"
        entry = {
            "evidence_id": evidence_id,
            "kind": "c0r_declared_material" if observed else "c0r_material_quarantine",
            "physical_parent_id": parent_id,
            "projection_sha256": sorted(
                {projection_hashes[row["role"]] for row in members}
            ),
            "source_component_ids": sorted(
                {row["family_component_id"] for row in members}
            ),
            "source_family_ids": sorted({row["family_id"] for row in members}),
        }
        evidence[evidence_id] = entry
        if observed:
            materials[parent_id] = material
            candidates[(parent_id, None, "material_label")].append(
                {
                    "confidence_observed": False,
                    "confidence_ppm": None,
                    "evidence_refs": [evidence_id],
                    "value": material,
                }
            )
        else:
            materials[parent_id] = None
            quarantined.add(parent_id)
    return materials, quarantined, evidence, candidates


def add_source_candidates(
    inventory: dict[str, Any],
    observations: list[dict[str, Any]],
    evidence: dict[str, dict[str, Any]],
    candidates: dict[tuple[str, str | None, str], list[dict[str, Any]]],
) -> None:
    for observation in observations:
        for field, claim in observation["fields"].items():
            evidence_id = f"source:{inventory['root_name']}:{observation['observation_id']}:{field}"
            evidence[evidence_id] = {
                "artifact_ids": claim["evidence_artifact_ids"],
                "evidence_id": evidence_id,
                "kind": "descriptor_source_claim",
                "observation_id": observation["observation_id"],
                "physical_parent_id": observation["physical_parent_id"],
                "project_id": inventory["project_id"],
                "publisher_id": inventory["publisher_id"],
                "record_id": observation["record_id"],
                "source_component_id": inventory["source_component_id"],
                "source_id": inventory["source_id"],
            }
            key = (
                observation["physical_parent_id"],
                observation["record_id"],
                field,
            )
            candidates[key].append(
                {
                    "confidence_observed": claim["confidence_observed"],
                    "confidence_ppm": claim["confidence_ppm"],
                    "evidence_refs": [evidence_id],
                    "value": claim["value"],
                }
            )


def resolve_field(
    claims: list[dict[str, Any]],
    forced_conflict_refs: list[str] | None = None,
) -> tuple[Any, bool, int, bool, dict[str, Any]]:
    if forced_conflict_refs is not None:
        return (
            None,
            False,
            0,
            False,
            {
                "evidence_refs": sorted(forced_conflict_refs),
                "missing_reason": "source_conflict",
                "state": "missing",
            },
        )
    if not claims:
        return (
            None,
            False,
            0,
            False,
            {
                "evidence_refs": [],
                "missing_reason": "not_published",
                "state": "missing",
            },
        )
    grouped: dict[bytes, list[dict[str, Any]]] = defaultdict(list)
    for claim in claims:
        grouped[canonical_json(claim["value"])].append(claim)
    refs = sorted(
        {reference for claim in claims for reference in claim["evidence_refs"]}
    )
    if len(grouped) != 1:
        return (
            None,
            False,
            0,
            False,
            {
                "evidence_refs": refs,
                "missing_reason": "source_conflict",
                "state": "missing",
            },
        )
    value = claims[0]["value"]
    confidence_is_observed = all(claim["confidence_observed"] for claim in claims)
    confidence = (
        min(claim["confidence_ppm"] for claim in claims)
        if confidence_is_observed
        else 0
    )
    return (
        value,
        True,
        confidence,
        confidence_is_observed,
        {"evidence_refs": refs, "missing_reason": None, "state": "observed"},
    )


def build_descriptor_records(
    rows: list[dict[str, Any]],
    quarantined: set[str],
    evidence: dict[str, dict[str, Any]],
    candidates: dict[tuple[str, str | None, str], list[dict[str, Any]]],
) -> list[dict[str, Any]]:
    records = []
    for row in sorted(rows, key=lambda item: item["record_id"]):
        parent_id = row["physical_parent_id"]
        values = {}
        observed_mask = {}
        confidence_ppm = {}
        confidence_mask = {}
        field_state = {}
        for field in FIELD_ORDER:
            key = (
                (parent_id, None, field)
                if field in OBJECT_FIELDS
                else (parent_id, row["record_id"], field)
            )
            forced = None
            if field == "material_label" and parent_id in quarantined:
                forced = [f"c0r:parent:{parent_id}:material_label"]
            value, observed, confidence, confidence_observed, state = resolve_field(
                candidates.get(key, []), forced
            )
            for reference in state["evidence_refs"]:
                if reference not in evidence:
                    raise DescriptorIntakeError(
                        "descriptor evidence reference is absent"
                    )
            values[field] = value
            observed_mask[field] = observed
            confidence_ppm[field] = confidence
            confidence_mask[field] = confidence_observed
            field_state[field] = state
        model_input = {
            "confidence_mask": confidence_mask,
            "confidence_ppm": confidence_ppm,
            "observed_mask": observed_mask,
            "values": values,
        }
        validate_model_input(model_input)
        records.append(
            {
                "descriptor_id": f"c1:{row['record_id']}",
                "field_state": field_state,
                "model_input": model_input,
                "physical_parent_id": parent_id,
                "record_id": row["record_id"],
                "role": row["role"],
            }
        )
    return records


def validate_model_input(model_input: dict[str, Any]) -> None:
    if set(model_input) != {
        "confidence_mask",
        "confidence_ppm",
        "observed_mask",
        "values",
    }:
        raise DescriptorIntakeError("model-input fields changed")
    if any(set(model_input[key]) != set(FIELD_ORDER) for key in model_input):
        raise DescriptorIntakeError("model-input descriptor field set changed")

    def visit(value: Any) -> None:
        if isinstance(value, dict):
            for key, nested in value.items():
                lowered = key.lower()
                if any(token in lowered for token in FORBIDDEN_MODEL_TOKENS):
                    raise DescriptorIntakeError(
                        "forbidden identity entered model input"
                    )
                visit(nested)
        elif isinstance(value, list):
            for nested in value:
                visit(nested)

    visit(model_input)


def coverage_report(
    records: list[dict[str, Any]], rows: list[dict[str, Any]], profile: dict[str, Any]
) -> dict[str, Any]:
    parents = sorted({record["physical_parent_id"] for record in records})
    parent_roles = {
        parent: next(
            record["role"]
            for record in records
            if record["physical_parent_id"] == parent
        )
        for parent in parents
    }
    per_field = {}
    for field in FIELD_ORDER:
        selected = [
            record
            for record in records
            if record["model_input"]["observed_mask"][field]
        ]
        missing = Counter(
            record["field_state"][field]["missing_reason"]
            for record in records
            if not record["model_input"]["observed_mask"][field]
        )
        per_field[field] = {
            "confidence_observed_records": sum(
                record["model_input"]["confidence_mask"][field] for record in selected
            ),
            "missing_reason_record_counts": dict(sorted(missing.items())),
            "observed_parents": len(
                {record["physical_parent_id"] for record in selected}
            ),
            "observed_records": len(selected),
        }
    object_complete_parents = [
        parent
        for parent in parents
        if all(
            any(
                record["model_input"]["observed_mask"][field]
                for record in records
                if record["physical_parent_id"] == parent
            )
            for field in OBJECT_FIELDS
        )
    ]
    condition_complete = [
        record
        for record in records
        if all(
            record["model_input"]["observed_mask"][field] for field in CONDITION_FIELDS
        )
    ]
    complete = [
        record
        for record in records
        if all(record["model_input"]["observed_mask"].values())
    ]
    priority = {}
    for material in profile["priority_materials"]:
        material_parents = sorted(
            {
                row["physical_parent_id"]
                for row in rows
                if row["material_label"] == material
                and row["axis_mask"]["material_identity"]
            }
        )
        priority[material] = {
            "descriptor_complete_records": sum(
                record["physical_parent_id"] in material_parents for record in complete
            ),
            "development_parents": sum(
                parent_roles[parent] == DEVELOPMENT_ROLE for parent in material_parents
            ),
            "object_complete_parents": sum(
                parent in object_complete_parents for parent in material_parents
            ),
            "total_parents": len(material_parents),
            "train_parents": sum(
                parent_roles[parent] == TRAIN_ROLE for parent in material_parents
            ),
        }
    return {
        "condition_complete_records": len(condition_complete),
        "descriptor_complete_parents": len(
            {record["physical_parent_id"] for record in complete}
        ),
        "descriptor_complete_records": len(complete),
        "field_order": list(FIELD_ORDER),
        "object_complete_parents": len(object_complete_parents),
        "per_field": per_field,
        "priority_materials": priority,
        "role_parent_counts": {
            role: sum(parent_roles[parent] == role for parent in parents)
            for role in ROLES
        },
        "role_record_counts": Counter(record["role"] for record in records),
        "schema": COVERAGE_SCHEMA,
        "total_parents": len(parents),
        "total_records": len(records),
    }


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    contract_bytes: bytes,
    contract: dict[str, Any],
    corpus_root: Path,
    source_roots: list[Path],
) -> dict[str, bytes]:
    corpus_bytes, _corpus_values, rows = load_corpus_inputs(corpus_root, profile)
    parent_records: dict[str, set[str]] = defaultdict(set)
    for row in rows:
        parent_records[row["physical_parent_id"]].add(row["record_id"])
    projection_hashes = {
        TRAIN_ROLE: profile["input_bindings"]["train_projection"]["sha256"],
        DEVELOPMENT_ROLE: profile["input_bindings"]["development_projection"]["sha256"],
    }
    base_material, quarantined, evidence, candidates = base_material_state(
        rows, projection_hashes
    )
    roots = source_roots_by_name(source_roots, profile)
    inputs_by_name = {entry["root_name"]: entry for entry in profile["source_inputs"]}
    inventories = []
    source_manifest_bytes = 0
    source_artifact_bytes = 0
    source_observations = 0
    for root_name in sorted(roots):
        manifest_count, artifact_count, inventory, observations = (
            validate_source_manifest(
                root_name,
                roots[root_name],
                inputs_by_name[root_name],
                contract,
                parent_records,
                base_material,
                quarantined,
            )
        )
        source_manifest_bytes += manifest_count
        source_artifact_bytes += artifact_count
        source_observations += len(observations)
        inventories.append(inventory)
        add_source_candidates(inventory, observations, evidence, candidates)

    descriptor_records = build_descriptor_records(
        rows, quarantined, evidence, candidates
    )
    coverage = coverage_report(descriptor_records, rows, profile)
    expected = profile["expected_counts"]
    actual = {
        "corpus_parents": coverage["total_parents"],
        "corpus_records": coverage["total_records"],
        "descriptor_complete_parents": coverage["descriptor_complete_parents"],
        "descriptor_complete_records": coverage["descriptor_complete_records"],
        "material_observed_parents": coverage["per_field"]["material_label"][
            "observed_parents"
        ],
        "material_observed_records": coverage["per_field"]["material_label"][
            "observed_records"
        ],
        "source_artifacts": sum(item["artifact_count"] for item in inventories),
        "source_inputs": len(inventories),
        "source_observations": source_observations,
    }
    if actual != expected:
        raise DescriptorIntakeError(
            f"C1 expected counts changed: expected {expected}, got {actual}"
        )

    records_document = {
        "contract_sha256": sha256_bytes(contract_bytes),
        "record_count": len(descriptor_records),
        "records": descriptor_records,
        "records_root_sha256": sha256_bytes(canonical_json(descriptor_records)),
        "schema": RECORDS_SCHEMA,
    }
    provenance_document = {
        "entries": [evidence[key] for key in sorted(evidence)],
        "entry_count": len(evidence),
        "schema": PROVENANCE_SCHEMA,
    }
    source_inventory = {
        "schema": SOURCE_INVENTORY_SCHEMA,
        "source_count": len(inventories),
        "sources": inventories,
    }
    access = {
        "acoustic_feature_bytes_read": 0,
        "candidate_model_bytes_read": 0,
        "corpus_metadata_bytes_read": corpus_bytes,
        "descriptor_source_artifact_bytes_read": source_artifact_bytes,
        "descriptor_source_manifest_bytes_read": source_manifest_bytes,
        "network_requests": 0,
        "pcm_bytes_read": 0,
        "protected_bytes_read": 0,
        "validator_projection_bytes_read": 0,
    }
    access_document = {
        "access": access,
        "authority": AUTHORITY,
        "profile_sha256": sha256_bytes(profile_bytes),
        "schema": ACCESS_SCHEMA,
    }
    artifacts = {
        "access-ledger.json": canonical_json(access_document),
        "coverage.json": canonical_json(coverage),
        "descriptor-contract.json": contract_bytes,
        "descriptor-records.json": canonical_json(records_document),
        "profile.json": profile_bytes,
        "provenance-ledger.json": canonical_json(provenance_document),
        "source-inventory.json": canonical_json(source_inventory),
    }
    artifact_bindings = {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in sorted(artifacts.items())
    }
    gates = {
        "all_c0r_inputs_hash_verified": True,
        "contract_is_hash_bound_and_canonical": True,
        "corpus_expected_counts_exact": True,
        "descriptor_counts_exact": actual == expected,
        "generator_roles_parent_and_component_disjoint": True,
        "material_quarantine_preserved": len(quarantined)
        == profile["expected_counts"]["corpus_parents"]
        - profile["expected_counts"]["material_observed_parents"],
        "model_inputs_exclude_capture_identity": True,
        "no_audio_feature_validator_candidate_protected_or_network_access": all(
            access[key] == 0
            for key in (
                "acoustic_feature_bytes_read",
                "candidate_model_bytes_read",
                "network_requests",
                "pcm_bytes_read",
                "protected_bytes_read",
                "validator_projection_bytes_read",
            )
        ),
        "unknown_fields_are_explicitly_masked": all(
            state["state"] == "observed"
            or (
                state["state"] == "missing"
                and state["missing_reason"]
                in contract["missingness_policy"]["allowed_reasons"]
            )
            for record in descriptor_records
            for state in record["field_state"].values()
        ),
    }
    if not all(gates.values()):
        raise DescriptorIntakeError("C1 conjunctive gate failed")
    report = {
        "access": access,
        "artifacts": artifact_bindings,
        "authority": AUTHORITY,
        "decision": DECISION,
        "gates": gates,
        "next_actions": ["v44_g0_descriptor_source_growth"],
        "result": {
            **actual,
            "field_observed_parent_counts": {
                field: coverage["per_field"][field]["observed_parents"]
                for field in FIELD_ORDER
            },
            "field_observed_record_counts": {
                field: coverage["per_field"][field]["observed_records"]
                for field in FIELD_ORDER
            },
        },
        "schema": REPORT_SCHEMA,
    }
    artifacts["report.json"] = canonical_json(report)
    return artifacts


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise DescriptorIntakeError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise DescriptorIntakeError("output must remain outside the repository")
    if resolved.exists():
        raise DescriptorIntakeError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish_directory(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for name, data in sorted(files.items()):
            if Path(name).name != name:
                raise DescriptorIntakeError("C1 outputs must be flat files")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(
    profile_path: Path,
    corpus_root: Path,
    source_roots: list[Path],
    output: Path,
) -> Path:
    profile_bytes, raw_profile = read_canonical_json(profile_path, "C1 profile")
    profile = validate_profile(raw_profile)
    contract_bytes, contract = load_contract(profile)
    return publish_directory(
        output,
        build_documents(
            profile_bytes,
            profile,
            contract_bytes,
            contract,
            prepare_root(corpus_root, "C0R root"),
            source_roots,
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    print(
        run(
            arguments.profile,
            arguments.corpus_root,
            arguments.source_root,
            arguments.output,
        )
    )


if __name__ == "__main__":
    main()
