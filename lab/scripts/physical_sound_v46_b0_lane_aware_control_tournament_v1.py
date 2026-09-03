#!/usr/bin/env python3
"""Run the V46 B0 lane-aware control tournament outside the repository."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import tempfile
from collections import defaultdict
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v41_b0_grouped_baselines_v1 as b0
import physical_sound_v44_b0r_r0r_corrected_baseline_v1 as b0r

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v46-b0-profile.v1"
LANE_SCHEMA = "nextengine.experimental-physical-sound-v46-b0-lane-matrix.v1"
METRICS_SCHEMA = "nextengine.experimental-physical-sound-v46-b0-control-metrics.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v46-b0-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v46-b0-report.v1"

D1_ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-access.v1"
D1_INDEX_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-corpus-index.v1"
D1_REPORT_SCHEMA = "nextengine.experimental-physical-sound-v46-d1-report.v1"
CLATTER_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-prior.v1"
HISTORICAL_METRICS_SCHEMA = "nextengine.experimental-physical-sound-v44-b0r-metrics.v1"
HISTORICAL_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-b0r-r0r-report.v1"
)

OWNER_PATH = "lab/scripts/physical_sound_v46_b0_lane_aware_control_tournament_v1.py"
PROTOCOL_PATH = (
    "docs/development/physical-sound-v46-b0-lane-aware-control-tournament-"
    "protocol-2026-09-03.md"
)
STUDY_ID = "physical-sound-v46-b0-lane-aware-control-tournament"
CLAIM = (
    "DISCLOSED_LANE_COMPATIBILITY_AND_CONTROL_FLOOR_ONLY / NO_TEACHER_MODEL_"
    "VALIDATOR_PROTECTED_COOKER_DEMO_PUBLIC_CONTRACT_OR_RUNTIME_AUTHORITY"
)
DECISIONS = ("ControlFloorFrozen", "NoUsefulTeacher")
CONTROL_NAMES = ("global_prototype", "retrieval_copy", "ridge")
HASH_LENGTH = 64
MAX_JSON_BYTES = 2 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authorizes_only": [
        "v46_e0_real_source_power",
        "v46_v0_validator_source_growth",
    ],
    "baseline_comparison_authority": True,
    "candidate_model_training_authority": False,
    "control_floor_product_authority": False,
    "model_selection_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_access_authority": False,
}

ACCESS_POLICY = {
    "allowed_target_contract": "c0r_acoustic_pseudo_target",
    "allowed_target_roles": ["generator_development", "generator_train"],
    "c0r_target_values_allowed_after_seal": True,
    "clatter_control_metadata_allowed": True,
    "external_inputs_must_be_flat": True,
    "ieteasy_target_values_allowed": False,
    "network_allowed": False,
    "outputs_external": True,
    "pcm_or_waveform_values_allowed": False,
    "protected_values_allowed": False,
    "validator_values_allowed": False,
}

EVALUATION_POLICY = {
    "aggregation_order": ["record_to_physical_parent", "parent_to_project"],
    "candidate_or_hyperparameter_selection_allowed": False,
    "control_ranking": (
        "minimum_supported_parent_median_rmse_then_project_balanced_mean_rmse_then_name"
    ),
    "metric": "masked_train_standardized_rmse",
    "project_weight": "equal_after_parent_mean",
    "target_contract_crossing_allowed": False,
    "waveform_vote_allowed": False,
}

MATERIAL_TAXONOMY = {
    "Aluminium": "metallic",
    "Ceramic": "ceramic",
    "Foam": "foam",
    "Glass": "glass",
    "Hard Plastic": "polymer",
    "Iron": "metallic",
    "Metal": "metallic",
    "Other Plastic": "polymer",
    "Paper": "paper",
    "Plastic": "polymer",
    "Polycarbonate": "polymer",
    "Soft Plastic": "polymer",
    "Steel": "metallic",
    "Wood": "wood",
}

ZERO_ACCESS_COUNTERS = (
    "candidate_training_steps",
    "generated_render_bytes_read",
    "ieteasy_target_bytes_read",
    "model_or_checkpoint_bytes_read",
    "network_requests",
    "pcm_or_waveform_bytes_read",
    "protected_bytes_read",
    "validator_projection_bytes_read",
    "validator_target_bytes_read",
)


class ControlTournamentError(RuntimeError):
    """B0 cannot publish a trustworthy lane-aware comparison."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--inputs", required=True, type=Path)
    parser.add_argument("--c0r-root", required=True, type=Path)
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
        raise ControlTournamentError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def rounded(value: float, digits: int = 9) -> float:
    if not math.isfinite(value):
        raise ControlTournamentError("derived value is non-finite")
    result = round(float(value), digits)
    return 0.0 if result == 0 else result


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ControlTournamentError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise ControlTournamentError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise ControlTournamentError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise ControlTournamentError(f"{context} must be an integer >= {minimum}")
    return value


def require_hash(value: Any, context: str) -> str:
    text = require_string(value, context)
    if len(text) != HASH_LENGTH or any(
        character not in "0123456789abcdef" for character in text
    ):
        raise ControlTournamentError(f"{context} must be a lowercase SHA-256")
    return text


def read_regular(path: Path, context: str, maximum: int = MAX_JSON_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ControlTournamentError(f"{context} must be a regular non-symlink file")
    count = path.stat().st_size
    if count <= 0 or count > maximum:
        raise ControlTournamentError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ControlTournamentError(f"{context} is not valid UTF-8 JSON") from error
    return data, require_dict(value, context)


def validate_binding(value: Any, context: str, filename: bool) -> dict[str, Any]:
    binding = require_dict(value, context)
    path_key = "filename" if filename else "path"
    required = {"bytes", path_key, "sha256"}
    if filename:
        required |= {"schema", "source_id"}
    if set(binding) != required:
        raise ControlTournamentError(f"{context} fields changed")
    path = Path(require_string(binding[path_key], f"{context}.{path_key}"))
    if path.is_absolute() or ".." in path.parts:
        raise ControlTournamentError(f"{context} path is unsafe")
    if filename and len(path.parts) != 1:
        raise ControlTournamentError(f"{context} filename must be flat")
    require_int(binding["bytes"], f"{context}.bytes", 1)
    require_hash(binding["sha256"], f"{context}.sha256")
    if filename:
        require_string(binding["schema"], f"{context}.schema")
        require_string(binding["source_id"], f"{context}.source_id")
    return binding


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    data = read_regular(path, context)
    if len(data) != binding["bytes"] or sha256_bytes(data) != binding["sha256"]:
        raise ControlTournamentError(f"{context} binding mismatch")
    return data


def validate_profile(profile: dict[str, Any]) -> dict[str, Any]:
    if set(profile) != {
        "access_policy",
        "authority",
        "claim",
        "dependency_bindings",
        "environment",
        "evaluation_policy",
        "expected",
        "external_inputs",
        "material_taxonomy",
        "schema",
        "study_id",
    }:
        raise ControlTournamentError("profile fields changed")
    if (
        profile["schema"] != PROFILE_SCHEMA
        or profile["study_id"] != STUDY_ID
        or profile["claim"] != CLAIM
        or profile["authority"] != AUTHORITY
        or profile["access_policy"] != ACCESS_POLICY
        or profile["evaluation_policy"] != EVALUATION_POLICY
        or profile["material_taxonomy"] != MATERIAL_TAXONOMY
    ):
        raise ControlTournamentError("profile identity, policy or authority changed")
    if profile["environment"] != {"numpy_version": np.__version__}:
        raise ControlTournamentError("NumPy environment changed")

    dependencies = [
        validate_binding(item, f"dependency {index}", False)
        for index, item in enumerate(
            require_list(profile["dependency_bindings"], "dependency bindings")
        )
    ]
    paths = [item["path"] for item in dependencies]
    if paths != sorted(set(paths)) or not {OWNER_PATH, PROTOCOL_PATH}.issubset(paths):
        raise ControlTournamentError("dependencies must be sorted, unique and complete")
    for binding in dependencies:
        data = check_binding(
            repository_root() / binding["path"],
            binding,
            f"dependency {binding['path']}",
        )
        if not data:
            raise ControlTournamentError("empty dependency")

    inputs = [
        validate_binding(item, f"external input {index}", True)
        for index, item in enumerate(
            require_list(profile["external_inputs"], "external inputs")
        )
    ]
    names = [item["filename"] for item in inputs]
    required_names = [
        "clatter-prior.json",
        "d1-access.json",
        "d1-corpus-index.json",
        "d1-report.json",
        "historical-baseline-metrics.json",
        "historical-report.json",
    ]
    if names != required_names:
        raise ControlTournamentError("external input inventory changed")

    expected = require_dict(profile["expected"], "expected")
    required_expected = {
        "clatter_control_groups",
        "clatter_control_rows",
        "c0r_development_parents",
        "c0r_development_projects",
        "c0r_development_records",
        "c0r_supported_development_parents",
        "c0r_target_bytes",
        "c0r_target_objects",
        "c0r_train_parents",
        "c0r_train_projects",
        "c0r_train_records",
        "d1_real_records",
        "ieteasy_train_records",
        "modal_teacher_rows",
        "structural_transfer_rows",
    }
    if set(expected) != required_expected:
        raise ControlTournamentError("expected fields changed")
    for key, value in expected.items():
        require_int(value, f"expected.{key}")
    return profile


def load_external_inputs(
    directory: Path, profile: dict[str, Any]
) -> tuple[dict[str, dict[str, Any]], int]:
    root = repository_root().resolve()
    resolved = directory.resolve()
    if directory.is_symlink() or not directory.is_dir():
        raise ControlTournamentError("inputs must be a regular directory")
    if resolved == root or root in resolved.parents:
        raise ControlTournamentError("inputs must stay outside the repository")
    bindings = profile["external_inputs"]
    expected_names = [item["filename"] for item in bindings]
    if sorted(item.name for item in directory.iterdir()) != expected_names:
        raise ControlTournamentError("external input filenames changed")
    documents = {}
    total = 0
    for binding in bindings:
        name = binding["filename"]
        data = check_binding(directory / name, binding, f"external input {name}")
        try:
            document = require_dict(json.loads(data), f"external input {name}")
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ControlTournamentError(
                f"external input {name} is invalid JSON"
            ) from error
        if document.get("schema") != binding["schema"]:
            raise ControlTournamentError(f"external input {name} schema changed")
        documents[name] = document
        total += len(data)
    return documents, total


def numeric_leaf_count(value: Any) -> int:
    if isinstance(value, bool):
        return 0
    if isinstance(value, (int, float)):
        return 1
    if isinstance(value, list):
        return sum(numeric_leaf_count(item) for item in value)
    if isinstance(value, dict):
        return sum(numeric_leaf_count(item) for item in value.values())
    return 0


def clatter_groups(document: dict[str, Any]) -> tuple[list[dict[str, Any]], int]:
    if (
        document.get("schema") != CLATTER_SCHEMA
        or document.get("source_lane") != "empirical_prior"
    ):
        raise ControlTournamentError("Clatter prior identity changed")
    observed = require_list(document.get("observed_fields"), "Clatter fields")
    groups: dict[str, list[str]] = defaultdict(list)
    scalar_count = 0
    filenames = []
    for index, raw in enumerate(require_list(document.get("rows"), "Clatter rows")):
        row = require_dict(raw, f"Clatter row {index}")
        filename = require_string(row.get("filename"), f"Clatter row {index} filename")
        recipe = require_dict(row.get("recipe"), f"Clatter row {index} recipe")
        target = require_dict(row.get("target"), f"Clatter row {index} target")
        if target.get("observed_fields") != observed:
            raise ControlTournamentError("Clatter observed fields changed")
        if sha256_bytes(canonical_json(recipe)) != require_hash(
            row.get("recipe_sha256"), "Clatter recipe hash"
        ) or sha256_bytes(canonical_json(target)) != require_hash(
            row.get("target_sha256"), "Clatter target hash"
        ):
            raise ControlTournamentError("Clatter inline hash mismatch")
        modal = require_dict(
            require_dict(recipe.get("heads"), "Clatter heads").get("modal"),
            "Clatter modal head",
        )
        digest = sha256_bytes(canonical_json(modal))
        groups[digest].append(filename)
        scalar_count += numeric_leaf_count(modal)
        filenames.append(filename)
    if filenames != sorted(set(filenames)):
        raise ControlTournamentError("Clatter filenames changed")
    return (
        [
            {
                "group_id": f"clatter-modal-{digest}",
                "member_count": len(members),
                "members": sorted(members),
                "modal_head_sha256": digest,
            }
            for digest, members in sorted(groups.items())
        ],
        scalar_count,
    )


def validate_d1(
    documents: dict[str, dict[str, Any]], expected: dict[str, int]
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    access = documents["d1-access.json"]
    report = documents["d1-report.json"]
    index = documents["d1-corpus-index.json"]
    if access.get("schema") != D1_ACCESS_SCHEMA:
        raise ControlTournamentError("D1 access schema changed")
    counters = require_dict(access.get("counters"), "D1 access counters")
    for name in (
        "content_objects_opened",
        "model_values_read",
        "pcm_sample_values_decoded",
        "protected_values_read",
        "real_target_values_decoded",
        "waveform_bytes_read",
    ):
        if counters.get(name) != 0:
            raise ControlTournamentError("D1 opened forbidden content")
    if (
        report.get("schema") != D1_REPORT_SCHEMA
        or report.get("decision") != "CorpusIndexCompiled"
        or not all(require_dict(report.get("gates"), "D1 gates").values())
    ):
        raise ControlTournamentError("D1 report did not pass")
    if (
        index.get("schema") != D1_INDEX_SCHEMA
        or index.get("authority", {}).get("authorizes_only")
        != "v46_b0_control_tournament"
    ):
        raise ControlTournamentError("D1 index authority changed")
    real_rows = require_list(index.get("real_rows"), "D1 real rows")
    if len(real_rows) != expected["d1_real_records"]:
        raise ControlTournamentError("D1 real record count changed")
    controls = require_dict(index.get("control_lanes"), "D1 controls")
    modal_rows = require_list(
        require_dict(controls.get("modal_teacher"), "D1 modal lane").get("rows"),
        "D1 modal rows",
    )
    structural_rows = require_list(
        require_dict(controls.get("structural_transfer"), "D1 structural lane").get(
            "rows"
        ),
        "D1 structural rows",
    )
    if (
        len(modal_rows) != expected["modal_teacher_rows"]
        or len(structural_rows) != expected["structural_transfer_rows"]
    ):
        raise ControlTournamentError("D1 teacher lanes changed")
    c0r_rows = [row for row in real_rows if row.get("source_artifact") == "c0r"]
    iet_rows = [
        row for row in real_rows if row.get("source_artifact") == "ieteasy_increment"
    ]
    if len(iet_rows) != expected["ieteasy_train_records"] or any(
        row.get("role") != "generator_train" for row in iet_rows
    ):
        raise ControlTournamentError("IETeasy inventory changed")
    return index, c0r_rows, iet_rows


def validate_clatter_against_d1(
    index: dict[str, Any], groups: list[dict[str, Any]], expected: dict[str, int]
) -> None:
    empirical = require_dict(
        require_dict(index.get("control_lanes"), "D1 controls").get("empirical_prior"),
        "D1 empirical lane",
    )
    indexed = require_list(empirical.get("groups"), "D1 Clatter groups")
    compact = [
        {
            "group_id": item.get("group_id"),
            "member_count": len(require_list(item.get("members"), "D1 group members")),
            "modal_head_sha256": item.get("modal_head_sha256"),
        }
        for item in indexed
    ]
    derived = [
        {
            "group_id": item["group_id"],
            "member_count": item["member_count"],
            "modal_head_sha256": item["modal_head_sha256"],
        }
        for item in groups
    ]
    if compact != derived or len(groups) != expected["clatter_control_groups"]:
        raise ControlTournamentError("Clatter groups differ from D1")
    if sum(item["member_count"] for item in groups) != expected["clatter_control_rows"]:
        raise ControlTournamentError("Clatter row count changed")


def c0r_rows_by_role(
    rows: list[dict[str, Any]], expected: dict[str, int]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, str]]:
    selected: dict[str, list[dict[str, Any]]] = {
        "generator_train": [],
        "generator_development": [],
    }
    parent_projects: dict[str, str] = {}
    seen_records = set()
    parent_roles: dict[str, str] = {}
    for raw in rows:
        row = require_dict(raw, "D1 C0R row")
        role = row.get("role")
        if role == "validator_calibration":
            continue
        if role not in selected:
            raise ControlTournamentError("C0R role changed")
        if row.get("target_contract") != ACCESS_POLICY["allowed_target_contract"]:
            raise ControlTournamentError("C0R target contract changed")
        record_id = require_string(row.get("record_id"), "C0R record ID")
        parent_id = require_string(row.get("physical_parent_id"), "C0R parent ID")
        project_id = require_string(row.get("project_id"), "C0R project ID")
        if record_id in seen_records:
            raise ControlTournamentError("C0R record ID is duplicated")
        seen_records.add(record_id)
        if parent_roles.setdefault(parent_id, role) != role:
            raise ControlTournamentError("C0R parent crosses roles")
        if parent_projects.setdefault(parent_id, project_id) != project_id:
            raise ControlTournamentError("C0R parent crosses projects")
        selected[role].append(
            {
                "acoustic_target": require_dict(row.get("target"), "C0R target"),
                "axis_mask": require_dict(
                    row.get("observation_mask"), "C0R observation mask"
                ),
                "canonical_pcm": require_dict(row.get("pcm"), "C0R PCM"),
                "family_component_id": require_string(
                    row.get("source_component_id"), "C0R component"
                ),
                "family_id": project_id,
                "material_label": require_string(
                    row.get("material_label"), "C0R material"
                ),
                "physical_parent_id": parent_id,
                "record_id": record_id,
            }
        )
    train = sorted(selected["generator_train"], key=lambda item: item["record_id"])
    development = sorted(
        selected["generator_development"], key=lambda item: item["record_id"]
    )
    if (
        len(train) != expected["c0r_train_records"]
        or len(development) != expected["c0r_development_records"]
    ):
        raise ControlTournamentError("C0R role counts changed")
    return train, development, parent_projects


def prepare_external_root(path: Path, context: str) -> Path:
    repository = repository_root().resolve()
    resolved = path.resolve()
    if path.is_symlink() or not path.is_dir():
        raise ControlTournamentError(f"{context} must be a regular directory")
    if resolved == repository or repository in resolved.parents:
        raise ControlTournamentError(f"{context} must stay outside the repository")
    return resolved


def project_summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
    by_project: dict[str, list[float]] = defaultdict(list)
    for row in rows:
        by_project[row["project_id"]].append(row["aggregate_rmse"])
    if not by_project:
        raise ControlTournamentError("no supported projects remain")
    per_project = {
        project: {
            "mean_parent_rmse": rounded(float(np.mean(values))),
            "median_parent_rmse": rounded(float(np.median(values))),
            "parent_count": len(values),
        }
        for project, values in sorted(by_project.items())
    }
    return {
        "per_project": per_project,
        "project_balanced_mean_rmse": rounded(
            float(
                np.mean([value["mean_parent_rmse"] for value in per_project.values()])
            )
        ),
        "project_count": len(per_project),
    }


def evaluate_control(
    name: str,
    development: list[dict[str, Any]],
    prediction: np.ndarray,
    mean: np.ndarray,
    scale: np.ndarray,
    train_materials: set[str],
    train_classes: set[str],
    parent_projects: dict[str, str],
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    rows = []
    for index, parent in enumerate(development):
        truth = b0.standardized(parent, mean, scale)
        mask = parent["target_mask"]
        categories = {}
        for category, (start, stop) in b0.metric_groups().items():
            categories[category] = rounded(
                b0.parent_distance(
                    truth[start:stop], prediction[index, start:stop], mask[start:stop]
                )
            )
        rows.append(
            {
                "aggregate_rmse": rounded(
                    b0.parent_distance(truth, prediction[index], mask)
                ),
                "category_rmse": categories,
                "coarse_material": parent["coarse_material"],
                "coarse_material_supported": parent["coarse_material"] in train_classes,
                "exact_material_supported": parent["material_label"] in train_materials,
                "material_label": parent["material_label"],
                "parent_id": parent["parent_id"],
                "project_id": parent_projects[parent["parent_id"]],
            }
        )
    supported = [row for row in rows if row["coarse_material_supported"]]
    summary_rows = [
        {key: value for key, value in row.items() if key != "project_id"}
        for row in supported
    ]
    return {
        "coarse_supported": b0.summarize_errors(summary_rows),
        "project_grouped": project_summary(supported),
        "control_name": name,
        "ood_parent_count": len(rows) - len(supported),
    }, rows


def fit_controls(
    train: list[dict[str, Any]],
    development: list[dict[str, Any]],
    parent_projects: dict[str, str],
) -> tuple[dict[str, Any], dict[str, list[dict[str, Any]]]]:
    mean, scale = b0.fit_scaler(train)
    train_targets = np.stack([b0.standardized(parent, mean, scale) for parent in train])
    train_masks = np.stack([parent["target_mask"] for parent in train])
    global_prototype = b0.masked_mean(list(train_targets), list(train_masks))
    conditioned = [
        index for index, parent in enumerate(train) if parent["material_observed"]
    ]
    classes = b0.conditioning_classes(MATERIAL_TAXONOMY)
    train_inputs = b0r.material_inputs([train[index] for index in conditioned], classes)
    development_inputs = b0r.material_inputs(development, classes)
    ridge_bias, ridge_weights = b0.fit_ridge(
        train_inputs, train_targets[conditioned], train_masks[conditioned]
    )
    by_class: dict[str, list[int]] = defaultdict(list)
    for index in conditioned:
        by_class[train[index]["coarse_material"]].append(index)
    retrieval = {
        material: min(indices, key=lambda index: train[index]["parent_id"])
        for material, indices in by_class.items()
    }
    predictions = {
        "global_prototype": np.repeat(
            global_prototype[np.newaxis, :], len(development), axis=0
        ),
        "retrieval_copy": np.stack(
            [
                train_targets[retrieval[parent["coarse_material"]]]
                if parent["coarse_material"] in retrieval
                else global_prototype
                for parent in development
            ]
        ),
        "ridge": b0.ridge_predict(development_inputs, ridge_bias, ridge_weights),
    }
    train_materials = {
        parent["material_label"] for parent in train if parent["material_observed"]
    }
    metrics = {}
    details = {}
    for name in CONTROL_NAMES:
        metrics[name], details[name] = evaluate_control(
            name,
            development,
            predictions[name],
            mean,
            scale,
            train_materials,
            set(by_class),
            parent_projects,
        )
    ranking = sorted(
        CONTROL_NAMES,
        key=lambda name: (
            metrics[name]["coarse_supported"]["median_parent_rmse"],
            metrics[name]["project_grouped"]["project_balanced_mean_rmse"],
            name,
        ),
    )
    return {
        "baseline_metrics": metrics,
        "ranking": ranking,
        "real_control_floor": {
            "control_name": ranking[0],
            "metric": metrics[ranking[0]]["coarse_supported"],
            "project_grouped": metrics[ranking[0]]["project_grouped"],
            "target_contract": ACCESS_POLICY["allowed_target_contract"],
        },
    }, details


def validate_historical(
    documents: dict[str, dict[str, Any]], metrics: dict[str, Any]
) -> None:
    historical = documents["historical-baseline-metrics.json"]
    report = documents["historical-report.json"]
    if historical.get("schema") != HISTORICAL_METRICS_SCHEMA:
        raise ControlTournamentError("historical metrics schema changed")
    if (
        report.get("schema") != HISTORICAL_REPORT_SCHEMA
        or report.get("decision") != "CorrectedCorpusSignalInsufficient"
        or report.get("access", {}).get("validator_projection_bytes_read") != 0
        or report.get("access", {}).get("pcm_bytes_read") != 0
    ):
        raise ControlTournamentError("historical report identity changed")
    reference = require_dict(historical.get("baseline_metrics"), "historical metrics")
    for name in CONTROL_NAMES:
        previous = require_dict(reference.get(name), f"historical {name}").get(
            "coarse_supported"
        )
        current = metrics["baseline_metrics"][name]["coarse_supported"]
        if previous != current:
            raise ControlTournamentError(f"{name} no longer reproduces V44 B0R")
    if metrics["ranking"][0] != "global_prototype":
        raise ControlTournamentError("historical real control floor changed")


def lane_matrix(clatter: list[dict[str, Any]], index: dict[str, Any]) -> dict[str, Any]:
    controls = index["control_lanes"]
    rows = [
        {
            "control_family": "c0r_simple_controls",
            "evaluation_state": "scoreable",
            "observed_contract": "c0r_acoustic_pseudo_target",
            "reason": "train_and_disjoint_development_share_exact_target_contract",
            "role": "real_acoustic_control_floor",
        },
        {
            "control_family": "ieteasy_ndac75",
            "evaluation_state": "not_scoreable",
            "observed_contract": (
                "nextengine.experimental-physical-sound-v44-noise-robust-fixed-target.v1"
            ),
            "reason": "train_only_without_same_contract_development_project",
            "role": "real_acoustic_inventory_only",
        },
        {
            "control_family": "v31_analytic_modal_owner",
            "evaluation_state": "not_scoreable",
            "observed_contract": "v31_plate_beam_modal_control",
            "reason": "required_geometry_contact_inputs_and_lossless_recipe_v3_mapping_absent",
            "role": "analytic_control_only",
        },
        {
            "control_family": "clatter_empirical_prior",
            "evaluation_state": "not_scoreable",
            "group_count": len(clatter),
            "observed_contract": "recipe_v3_five_modal_fields",
            "reason": "no_independent_recipe_v3_modal_truth",
            "role": "empirical_prior_control_only",
        },
        {
            "control_family": "external_modal_teacher",
            "evaluation_state": "unavailable",
            "observed_contract": "recipe_v3_modal_teacher",
            "reason": controls["modal_teacher"]["reason"],
            "role": "teacher",
        },
        {
            "control_family": "structural_transfer",
            "evaluation_state": "unavailable",
            "observed_contract": "recipe_v3_structural_transfer",
            "reason": controls["structural_transfer"]["reason"],
            "role": "teacher",
        },
    ]
    return {
        "cross_contract_comparison_count": 0,
        "rows": rows,
        "schema": LANE_SCHEMA,
        "scoreable_non_real_teacher_count": sum(
            row["evaluation_state"] == "scoreable"
            and row["role"] != "real_acoustic_control_floor"
            for row in rows
        ),
    }


def decide(matrix: dict[str, Any]) -> str:
    decision = (
        "ControlFloorFrozen"
        if matrix["scoreable_non_real_teacher_count"] > 0
        else "NoUsefulTeacher"
    )
    if decision not in DECISIONS:
        raise ControlTournamentError("decision is outside the frozen set")
    return decision


def prepare_output(path: Path) -> Path:
    repository = repository_root().resolve()
    destination = path.resolve()
    if destination == repository or repository in destination.parents:
        raise ControlTournamentError("output must stay outside the repository")
    if destination.exists():
        raise ControlTournamentError("output already exists")
    destination.parent.mkdir(parents=True, exist_ok=True)
    return destination


def publish(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for name, data in sorted(files.items()):
            if Path(name).name != name:
                raise ControlTournamentError("outputs must be flat")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def build_documents(
    profile_bytes: bytes,
    profile: dict[str, Any],
    documents: dict[str, dict[str, Any]],
    external_input_bytes: int,
    c0r_root: Path,
) -> dict[str, bytes]:
    expected = profile["expected"]
    index, c0r_rows, iet_rows = validate_d1(documents, expected)
    groups, clatter_scalars = clatter_groups(documents["clatter-prior.json"])
    validate_clatter_against_d1(index, groups, expected)
    train_rows, development_rows, parent_projects = c0r_rows_by_role(c0r_rows, expected)
    train, train_bytes = b0r.aggregate_parents(train_rows, c0r_root, MATERIAL_TAXONOMY)
    development, development_bytes = b0r.aggregate_parents(
        development_rows, c0r_root, MATERIAL_TAXONOMY
    )
    if (
        len(train) != expected["c0r_train_parents"]
        or len(development) != expected["c0r_development_parents"]
        or len({parent_projects[parent["parent_id"]] for parent in train})
        != expected["c0r_train_projects"]
        or len({parent_projects[parent["parent_id"]] for parent in development})
        != expected["c0r_development_projects"]
    ):
        raise ControlTournamentError("C0R grouped counts changed")
    target_objects = len(
        {row["acoustic_target"]["sha256"] for row in train_rows + development_rows}
    )
    target_bytes = train_bytes + development_bytes
    if (
        target_objects != expected["c0r_target_objects"]
        or target_bytes != expected["c0r_target_bytes"]
    ):
        raise ControlTournamentError("C0R target access envelope changed")

    metrics, details = fit_controls(train, development, parent_projects)
    supported = metrics["real_control_floor"]["metric"]["parent_count"]
    if supported != expected["c0r_supported_development_parents"]:
        raise ControlTournamentError("supported development count changed")
    validate_historical(documents, metrics)
    metrics_document = {
        **metrics,
        "details": details,
        "evaluation_policy": EVALUATION_POLICY,
        "schema": METRICS_SCHEMA,
    }
    matrix = lane_matrix(groups, index)
    decision = decide(matrix)
    access = {
        "c0r_development_target_bytes_read": development_bytes,
        "c0r_development_target_objects_opened": len(development_rows),
        "c0r_train_target_bytes_read": train_bytes,
        "c0r_train_target_objects_opened": len(train_rows),
        "clatter_control_numeric_scalars_read": clatter_scalars,
        "external_input_bytes_read": external_input_bytes,
        "profile_bytes_read": len(profile_bytes),
        "schema": ACCESS_SCHEMA,
        **{name: 0 for name in ZERO_ACCESS_COUNTERS},
    }
    gates = {
        "all_bindings_exact": True,
        "c0r_group_counts_exact": True,
        "c0r_target_access_bounded": True,
        "clatter_groups_match_d1": True,
        "decision_mapping_exclusive": decision in DECISIONS,
        "forbidden_access_zero": all(
            access[name] == 0 for name in ZERO_ACCESS_COUNTERS
        ),
        "historical_simple_controls_reproduced": True,
        "lane_contracts_not_crossed": matrix["cross_contract_comparison_count"] == 0,
        "project_and_parent_aggregation_explicit": True,
        "teacher_usefulness_independently_scoreable": (
            matrix["scoreable_non_real_teacher_count"] > 0
        ),
    }
    files = {
        "access.json": canonical_json(access),
        "control-metrics.json": canonical_json(metrics_document),
        "lane-matrix.json": canonical_json(matrix),
        "profile.json": profile_bytes,
    }
    artifacts = {
        name: {"bytes": len(data), "sha256": sha256_bytes(data)}
        for name, data in sorted(files.items())
    }
    report = {
        "artifacts": artifacts,
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": decision,
        "gates": gates,
        "next": (
            "v46_e0_real_source_power_and_v0_validator_source_growth; m0_remains_closed"
        ),
        "real_control_floor": metrics["real_control_floor"],
        "schema": REPORT_SCHEMA,
        "study_id": STUDY_ID,
        "teacher_state": {
            "scoreable_non_real_teacher_count": matrix[
                "scoreable_non_real_teacher_count"
            ],
            "trusted_modal_teacher_rows": expected["modal_teacher_rows"],
            "structural_transfer_rows": expected["structural_transfer_rows"],
        },
    }
    files["report.json"] = canonical_json(report)
    return files


def run(profile_path: Path, inputs: Path, c0r_root: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_json(profile_path, "B0 profile")
    if canonical_json(raw_profile) != profile_bytes:
        raise ControlTournamentError("B0 profile must be canonical JSON")
    profile = validate_profile(raw_profile)
    documents, external_input_bytes = load_external_inputs(inputs, profile)
    root = prepare_external_root(c0r_root, "C0R corpus root")
    return publish(
        output,
        build_documents(profile_bytes, profile, documents, external_input_bytes, root),
    )


def main() -> None:
    arguments = parse_arguments()
    print(
        run(arguments.profile, arguments.inputs, arguments.c0r_root, arguments.output)
    )


if __name__ == "__main__":
    main()
