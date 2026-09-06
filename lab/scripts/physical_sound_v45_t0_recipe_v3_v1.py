#!/usr/bin/env python3
"""Freeze the V45 T0 value-free Recipe V3 and masked-target contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-profile.v1"
CONTRACT_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-recipe-v3.v1"
FIXTURES_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-fixtures.v1"
TARGET_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-masked-target.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v45-t0-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v45-t0-recipe-v3.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v45_t0_recipe_v3_v1.py"
DECISION = "T0_RECIPE_V3_MASKED_TARGET_CONTRACT_REPEATABLE"
CLAIM = (
    "VALUE_FREE_RECIPE_V3_AND_MASKED_TARGET_CONTRACT / NO_EXTERNAL_NUMERIC_"
    "PAYLOAD_TRAINING_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)

SAMPLE_RATE_HZ = 48_000
NYQUIST_MILLIHZ_EXCLUSIVE = 24_000_000
MAX_MODES = 32
TARGET_DIMENSION = 189
ENERGY_BUDGET_PPM = 1_000_000
LANES = (
    "empirical_prior",
    "modal_teacher",
    "protected_admission",
    "real_acoustic",
    "structural_transfer",
    "validator_calibration",
)
TRAINING_LANES = {
    "empirical_prior",
    "modal_teacher",
    "real_acoustic",
    "structural_transfer",
}
DESCRIPTORS = (
    "contact_location",
    "density",
    "geometry_extents",
    "impact_strength",
    "material_class",
    "poisson_ratio",
    "support_class",
    "youngs_modulus",
)
FORBIDDEN_COUNTERS = (
    "audio_files_decoded",
    "external_numeric_values_decoded",
    "feature_values_read",
    "model_values_read",
    "network_requests",
    "pcm_sample_values_decoded",
    "protected_values_read",
    "validator_values_read",
)

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "external_payload_authority": False,
    "product_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "schema_design_authority": True,
    "validator_authority": False,
}
ACCESS_POLICY = {
    "external_numeric_payload_allowed": False,
    "model_access_allowed": False,
    "network_allowed_at_build": False,
    "outputs_external": True,
    "pcm_or_feature_access_allowed": False,
    "protected_access_allowed": False,
    "validator_access_allowed": False,
}

# name, head, offset, count, unit, minimum, maximum
LAYOUT = (
    ("modal.mode_count", "modal", 0, 1, "count", 1, MAX_MODES),
    ("modal.base_frequency_millihz", "modal", 1, 1, "mHz", 20_000, 20_000_000),
    ("modal.frequency_ratio_ppm", "modal", 2, 32, "ppm", 1_000_000, 256_000_000),
    ("modal.rt60_milliseconds", "modal", 34, 32, "ms", 5, 60_000),
    ("modal.participation_ppm", "modal", 66, 32, "ppm", 0, 1_000_000),
    ("modal.uncertainty_ppm", "modal", 98, 32, "ppm", 0, 1_000_000),
    ("excitation.onset_samples", "excitation", 130, 1, "sample", 0, 4_095),
    ("excitation.contact_duration_samples", "excitation", 131, 1, "sample", 1, 960),
    ("excitation.impact_gain_ppm", "excitation", 132, 1, "ppm", 0, 2_000_000),
    ("excitation.location_modifier_ppm", "excitation", 133, 8, "ppm", -1_000_000, 1_000_000),
    ("excitation.support_modifier_ppm", "excitation", 141, 4, "ppm", -1_000_000, 1_000_000),
    ("excitation.uncertainty_ppm", "excitation", 145, 1, "ppm", 0, 1_000_000),
    ("radiation.band_energy_ppm", "radiation", 146, 24, "ppm", 0, 1_000_000),
    ("radiation.spectral_tilt_millidb_per_octave", "radiation", 170, 1, "mdB/octave", -24_000, 24_000),
    ("radiation.residual_envelope_ppm", "radiation", 171, 16, "ppm", 0, 1_000_000),
    ("radiation.uncertainty_ppm", "radiation", 187, 1, "ppm", 0, 1_000_000),
    ("support.ood_score_ppm", "support", 188, 1, "ppm", 0, 1_000_000),
)
FIELD_NAMES = tuple(row[0] for row in LAYOUT)
FIELD_BY_NAME = {row[0]: row for row in LAYOUT}

LANE_FIELDS = {
    "empirical_prior": tuple(name for name in FIELD_NAMES if name.startswith("modal.")),
    "modal_teacher": tuple(name for name in FIELD_NAMES if name.startswith("modal.")),
    "structural_transfer": (
        "modal.base_frequency_millihz",
        "modal.frequency_ratio_ppm",
        "modal.rt60_milliseconds",
        "modal.participation_ppm",
        "excitation.onset_samples",
        "excitation.contact_duration_samples",
        "excitation.impact_gain_ppm",
        "excitation.location_modifier_ppm",
        "excitation.support_modifier_ppm",
        "excitation.uncertainty_ppm",
        "support.ood_score_ppm",
    ),
    "real_acoustic": FIELD_NAMES,
    "validator_calibration": (),
    "protected_admission": (),
}


class RecipeContractError(RuntimeError):
    """The T0 profile, recipe, target or publication is invalid."""


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
            json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False)
            + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise RecipeContractError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise RecipeContractError(f"{context} must be an object")
    return value


def require_int(value: Any, context: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise RecipeContractError(f"{context} must be an integer")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise RecipeContractError(f"{context} must be a non-empty string")
    return value


def require_int_array(value: Any, count: int, context: str) -> list[int]:
    if not isinstance(value, list) or len(value) != count:
        raise RecipeContractError(f"{context} must contain exactly {count} integers")
    return [require_int(item, f"{context}[{index}]") for index, item in enumerate(value)]


def clamp(value: int, minimum: int, maximum: int) -> int:
    return min(max(value, minimum), maximum)


def normalize_budget(values: list[int], budget: int = ENERGY_BUDGET_PPM) -> list[int]:
    nonnegative = [max(value, 0) for value in values]
    total = sum(nonnegative)
    if total <= budget:
        return nonnegative
    scaled = [(value * budget) // total for value in nonnegative]
    remainders = [(value * budget) % total for value in nonnegative]
    missing = budget - sum(scaled)
    for index in sorted(range(len(values)), key=lambda item: (-remainders[item], item))[:missing]:
        scaled[index] += 1
    return scaled


def read_regular(path: Path, context: str, maximum: int = 2 * 1024 * 1024) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise RecipeContractError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise RecipeContractError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RecipeContractError(f"{context} is not valid UTF-8 JSON") from error
    document = require_dict(value, context)
    if data != canonical_json(document):
        raise RecipeContractError(f"{context} is not canonical JSON")
    return data, document


def validate_dependency(value: Any, context: str) -> dict[str, Any]:
    binding = require_dict(value, context)
    if set(binding) != {"bytes", "path", "sha256"}:
        raise RecipeContractError(f"{context} fields changed")
    relative = Path(require_string(binding["path"], f"{context}.path"))
    if relative.is_absolute() or ".." in relative.parts:
        raise RecipeContractError(f"{context} path escapes the repository")
    count = require_int(binding["bytes"], f"{context}.bytes")
    digest = require_string(binding["sha256"], f"{context}.sha256")
    if count <= 0 or len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
        raise RecipeContractError(f"{context} has an invalid binding")
    data = read_regular(repository_root() / relative, context)
    if len(data) != count or sha256_bytes(data) != digest:
        raise RecipeContractError(f"{context} binding mismatch")
    return binding


def expected_layout() -> list[dict[str, Any]]:
    return [
        {
            "count": count,
            "head": head,
            "maximum": maximum,
            "minimum": minimum,
            "name": name,
            "offset": offset,
            "unit": unit,
        }
        for name, head, offset, count, unit, minimum, maximum in LAYOUT
    ]


def validate_profile(profile: dict[str, Any]) -> list[dict[str, Any]]:
    if set(profile) != {
        "access_policy", "authority", "claim", "decision", "dependency_bindings",
        "fixtures", "lane_fields", "layout", "projection", "schema",
    }:
        raise RecipeContractError("profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM or profile["decision"] != DECISION:
        raise RecipeContractError("profile identity changed")
    if profile["authority"] != AUTHORITY or profile["access_policy"] != ACCESS_POLICY:
        raise RecipeContractError("profile authority or access policy changed")
    if profile["layout"] != expected_layout():
        raise RecipeContractError("recipe layout changed")
    if profile["lane_fields"] != {lane: list(LANE_FIELDS[lane]) for lane in LANES}:
        raise RecipeContractError("lane field policy changed")
    if profile["projection"] != {
        "canonical_json": "utf8-sort-keys-indent2-lf",
        "energy_budget_ppm": ENERGY_BUDGET_PPM,
        "integer_only": True,
        "max_modes": MAX_MODES,
        "missing_targets_contribute_loss": False,
        "mode_order": "ascending-frequency-ratio-then-input-ordinal",
        "nyquist_millihz_exclusive": NYQUIST_MILLIHZ_EXCLUSIVE,
        "padding": "zero-value-zero-mask",
        "rounding": "integer-floor-with-largest-remainder-energy-normalization",
        "sample_rate_hz": SAMPLE_RATE_HZ,
        "source_lanes_interchangeable": False,
        "target_dimension": TARGET_DIMENSION,
    }:
        raise RecipeContractError("projection policy changed")
    bindings = profile["dependency_bindings"]
    if not isinstance(bindings, list) or not bindings:
        raise RecipeContractError("dependency bindings must be non-empty")
    checked = [validate_dependency(item, f"dependency {index}") for index, item in enumerate(bindings)]
    paths = [item["path"] for item in checked]
    if paths != sorted(set(paths)) or OWNER_PATH not in paths:
        raise RecipeContractError("dependency bindings must be sorted, unique and bind the owner")
    fixtures = profile["fixtures"]
    if not isinstance(fixtures, list) or not fixtures:
        raise RecipeContractError("fixtures must be non-empty")
    ids = [require_string(require_dict(item, "fixture").get("fixture_id"), "fixture_id") for item in fixtures]
    if ids != sorted(set(ids)):
        raise RecipeContractError("fixture IDs must be sorted and unique")
    return fixtures


def _head(value: Any, keys: set[str], context: str) -> dict[str, Any]:
    result = require_dict(value, context)
    if set(result) != keys:
        raise RecipeContractError(f"{context} fields changed")
    return result


def project_recipe(raw_value: Any) -> dict[str, Any]:
    raw = require_dict(raw_value, "recipe")
    if set(raw) != {"descriptor_mask", "heads", "recipe_id", "source_lane"}:
        raise RecipeContractError("recipe fields changed")
    recipe_id = require_string(raw["recipe_id"], "recipe_id")
    lane = require_string(raw["source_lane"], "source_lane")
    if lane not in LANES:
        raise RecipeContractError("source lane is unknown")
    descriptors = raw["descriptor_mask"]
    if not isinstance(descriptors, list) or descriptors != sorted(set(descriptors)):
        raise RecipeContractError("descriptor mask must be sorted and unique")
    if any(item not in DESCRIPTORS for item in descriptors):
        raise RecipeContractError("descriptor mask contains an unknown descriptor")
    heads = _head(raw["heads"], {"excitation", "modal", "radiation", "support"}, "heads")

    modal = _head(
        heads["modal"],
        {"base_frequency_millihz", "frequency_ratio_ppm", "mode_count", "participation_ppm", "rt60_milliseconds", "uncertainty_ppm"},
        "modal head",
    )
    count = require_int(modal["mode_count"], "mode_count")
    if not 1 <= count <= MAX_MODES:
        raise RecipeContractError("mode_count exceeds the resource envelope")
    base = clamp(require_int(modal["base_frequency_millihz"], "base frequency"), 20_000, 20_000_000)
    ratios = require_int_array(modal["frequency_ratio_ppm"], count, "frequency ratios")
    rt60 = require_int_array(modal["rt60_milliseconds"], count, "rt60")
    participation = require_int_array(modal["participation_ppm"], count, "participation")
    modal_uncertainty = require_int_array(modal["uncertainty_ppm"], count, "modal uncertainty")
    maximum_ratio = ((NYQUIST_MILLIHZ_EXCLUSIVE - 1) * 1_000_000) // base
    if maximum_ratio < 1_000_000 + count - 1:
        raise RecipeContractError("base frequency leaves no ordered mode domain below Nyquist")
    rows = sorted(
        zip(ratios, rt60, participation, modal_uncertainty, range(count), strict=True),
        key=lambda item: (item[0], item[4]),
    )
    projected_ratios: list[int] = []
    projected_rt60: list[int] = []
    projected_participation: list[int] = []
    projected_uncertainty: list[int] = []
    for ratio, decay, gain, uncertainty, _ordinal in rows:
        ratio = clamp(ratio, 1_000_000, maximum_ratio)
        if projected_ratios:
            ratio = max(ratio, projected_ratios[-1] + 1)
        if ratio > maximum_ratio:
            raise RecipeContractError("mode ordering cannot fit below Nyquist")
        projected_ratios.append(ratio)
        projected_rt60.append(clamp(decay, 5, 60_000))
        projected_participation.append(clamp(gain, 0, ENERGY_BUDGET_PPM))
        projected_uncertainty.append(clamp(uncertainty, 0, ENERGY_BUDGET_PPM))
    projected_participation = normalize_budget(projected_participation)

    excitation = _head(
        heads["excitation"],
        {"contact_duration_samples", "impact_gain_ppm", "location_modifier_ppm", "onset_samples", "support_modifier_ppm", "uncertainty_ppm"},
        "excitation head",
    )
    location = require_int_array(excitation["location_modifier_ppm"], 8, "location modifiers")
    support_modifiers = require_int_array(excitation["support_modifier_ppm"], 4, "support modifiers")
    projected_excitation = {
        "contact_duration_samples": clamp(require_int(excitation["contact_duration_samples"], "contact duration"), 1, 960),
        "impact_gain_ppm": clamp(require_int(excitation["impact_gain_ppm"], "impact gain"), 0, 2_000_000),
        "location_modifier_ppm": [clamp(value, -1_000_000, 1_000_000) for value in location],
        "onset_samples": clamp(require_int(excitation["onset_samples"], "onset"), 0, 4_095),
        "support_modifier_ppm": [clamp(value, -1_000_000, 1_000_000) for value in support_modifiers],
        "uncertainty_ppm": clamp(require_int(excitation["uncertainty_ppm"], "excitation uncertainty"), 0, 1_000_000),
    }

    radiation = _head(
        heads["radiation"],
        {"band_energy_ppm", "residual_envelope_ppm", "spectral_tilt_millidb_per_octave", "uncertainty_ppm"},
        "radiation head",
    )
    bands = require_int_array(radiation["band_energy_ppm"], 24, "radiation bands")
    residual = require_int_array(radiation["residual_envelope_ppm"], 16, "residual envelope")
    projected_radiation = {
        "band_energy_ppm": normalize_budget(bands),
        "residual_envelope_ppm": normalize_budget(residual),
        "spectral_tilt_millidb_per_octave": clamp(require_int(radiation["spectral_tilt_millidb_per_octave"], "spectral tilt"), -24_000, 24_000),
        "uncertainty_ppm": clamp(require_int(radiation["uncertainty_ppm"], "radiation uncertainty"), 0, 1_000_000),
    }
    support = _head(heads["support"], {"ood_score_ppm"}, "support head")
    projected = {
        "descriptor_mask": descriptors,
        "heads": {
            "excitation": projected_excitation,
            "modal": {
                "base_frequency_millihz": base,
                "frequency_ratio_ppm": projected_ratios,
                "mode_count": count,
                "participation_ppm": projected_participation,
                "rt60_milliseconds": projected_rt60,
                "uncertainty_ppm": projected_uncertainty,
            },
            "radiation": projected_radiation,
            "support": {"ood_score_ppm": clamp(require_int(support["ood_score_ppm"], "OOD score"), 0, 1_000_000)},
        },
        "recipe_id": recipe_id,
        "source_lane": lane,
    }
    frequencies = [(base * ratio) // 1_000_000 for ratio in projected_ratios]
    if frequencies != sorted(set(frequencies)) or any(value >= NYQUIST_MILLIHZ_EXCLUSIVE for value in frequencies):
        raise RecipeContractError("projected modes are not unique, ordered and below Nyquist")
    return projected


def flatten_recipe(recipe: dict[str, Any]) -> list[int]:
    heads = recipe["heads"]
    modal = heads["modal"]
    count = modal["mode_count"]
    vector = [0] * TARGET_DIMENSION
    vector[0] = count
    vector[1] = modal["base_frequency_millihz"]
    for offset, values in (
        (2, modal["frequency_ratio_ppm"]),
        (34, modal["rt60_milliseconds"]),
        (66, modal["participation_ppm"]),
        (98, modal["uncertainty_ppm"]),
    ):
        vector[offset : offset + count] = values
    excitation = heads["excitation"]
    vector[130] = excitation["onset_samples"]
    vector[131] = excitation["contact_duration_samples"]
    vector[132] = excitation["impact_gain_ppm"]
    vector[133:141] = excitation["location_modifier_ppm"]
    vector[141:145] = excitation["support_modifier_ppm"]
    vector[145] = excitation["uncertainty_ppm"]
    radiation = heads["radiation"]
    vector[146:170] = radiation["band_energy_ppm"]
    vector[170] = radiation["spectral_tilt_millidb_per_octave"]
    vector[171:187] = radiation["residual_envelope_ppm"]
    vector[187] = radiation["uncertainty_ppm"]
    vector[188] = heads["support"]["ood_score_ppm"]
    return vector


def recipe_from_vector(
    vector_value: Any,
    *,
    descriptor_mask: list[str],
    recipe_id: str,
    source_lane: str,
) -> dict[str, Any]:
    """Decode the fixed vector and reapply the sole canonical projector."""
    vector = require_int_array(vector_value, TARGET_DIMENSION, "recipe vector")
    count = vector[0]
    if not 1 <= count <= MAX_MODES:
        raise RecipeContractError("encoded mode_count exceeds the resource envelope")
    return project_recipe(
        {
            "descriptor_mask": descriptor_mask,
            "heads": {
                "excitation": {
                    "contact_duration_samples": vector[131],
                    "impact_gain_ppm": vector[132],
                    "location_modifier_ppm": vector[133:141],
                    "onset_samples": vector[130],
                    "support_modifier_ppm": vector[141:145],
                    "uncertainty_ppm": vector[145],
                },
                "modal": {
                    "base_frequency_millihz": vector[1],
                    "frequency_ratio_ppm": vector[2 : 2 + count],
                    "mode_count": count,
                    "participation_ppm": vector[66 : 66 + count],
                    "rt60_milliseconds": vector[34 : 34 + count],
                    "uncertainty_ppm": vector[98 : 98 + count],
                },
                "radiation": {
                    "band_energy_ppm": vector[146:170],
                    "residual_envelope_ppm": vector[171:187],
                    "spectral_tilt_millidb_per_octave": vector[170],
                    "uncertainty_ppm": vector[187],
                },
                "support": {"ood_score_ppm": vector[188]},
            },
            "recipe_id": recipe_id,
            "source_lane": source_lane,
        }
    )


def make_target(recipe: dict[str, Any], observed_value: Any) -> dict[str, Any]:
    observed = observed_value
    if not isinstance(observed, list) or observed != sorted(set(observed)):
        raise RecipeContractError("observed fields must be sorted and unique")
    if any(field not in FIELD_BY_NAME for field in observed):
        raise RecipeContractError("observed fields contain an unknown field")
    lane = recipe["source_lane"]
    if lane not in TRAINING_LANES and observed:
        raise RecipeContractError("validator or protected lane cannot create generator targets")
    if any(field not in LANE_FIELDS[lane] for field in observed):
        raise RecipeContractError("observed field exceeds its source lane")
    values = flatten_recipe(recipe)
    mask = [0] * TARGET_DIMENSION
    count = recipe["heads"]["modal"]["mode_count"]
    for field in observed:
        _name, _head_name, offset, width, _unit, _minimum, _maximum = FIELD_BY_NAME[field]
        active_width = count if field.startswith("modal.") and width == MAX_MODES else width
        mask[offset : offset + active_width] = [1] * active_width
    values = [value if present else 0 for value, present in zip(values, mask, strict=True)]
    return {
        "active_mode_count": count,
        "descriptor_mask": recipe["descriptor_mask"],
        "mask": mask,
        "observed_fields": observed,
        "recipe_id": recipe["recipe_id"],
        "schema": TARGET_SCHEMA,
        "source_lane": lane,
        "values": values,
    }


def validate_target(value: Any) -> dict[str, Any]:
    target = require_dict(value, "target")
    if set(target) != {
        "active_mode_count", "descriptor_mask", "mask", "observed_fields",
        "recipe_id", "schema", "source_lane", "values",
    }:
        raise RecipeContractError("target fields changed")
    if target["schema"] != TARGET_SCHEMA:
        raise RecipeContractError("target schema changed")
    count = require_int(target["active_mode_count"], "target active_mode_count")
    if not 1 <= count <= MAX_MODES:
        raise RecipeContractError("target active_mode_count exceeds the resource envelope")
    require_string(target["recipe_id"], "target recipe_id")
    lane = require_string(target["source_lane"], "target source_lane")
    if lane not in LANES:
        raise RecipeContractError("target source lane is unknown")
    descriptors = target["descriptor_mask"]
    if not isinstance(descriptors, list) or descriptors != sorted(set(descriptors)):
        raise RecipeContractError("target descriptor mask must be sorted and unique")
    if any(item not in DESCRIPTORS for item in descriptors):
        raise RecipeContractError("target descriptor mask contains an unknown descriptor")
    observed = target["observed_fields"]
    if not isinstance(observed, list) or observed != sorted(set(observed)):
        raise RecipeContractError("target observed fields must be sorted and unique")
    if lane not in TRAINING_LANES and observed:
        raise RecipeContractError("validator or protected lane cannot create generator targets")
    if any(field not in FIELD_BY_NAME or field not in LANE_FIELDS[lane] for field in observed):
        raise RecipeContractError("target observed field exceeds its source lane")
    values = require_int_array(target["values"], TARGET_DIMENSION, "target values")
    mask = require_int_array(target["mask"], TARGET_DIMENSION, "target mask")
    expected_mask = [0] * TARGET_DIMENSION
    for field in observed:
        _name, _head_name, offset, width, _unit, _minimum, _maximum = FIELD_BY_NAME[field]
        active_width = count if field.startswith("modal.") and width == MAX_MODES else width
        expected_mask[offset : offset + active_width] = [1] * active_width
    if mask != expected_mask:
        raise RecipeContractError("target mask does not match observed fields")
    if any(value != 0 for value, present in zip(values, mask, strict=True) if not present):
        raise RecipeContractError("missing target coordinates must be zero")
    return target


def masked_squared_error(prediction: Any, target: Any) -> dict[str, int]:
    predicted = require_int_array(prediction, TARGET_DIMENSION, "prediction")
    item = validate_target(target)
    values = item["values"]
    mask = item["mask"]
    return {
        "observed_coordinates": sum(mask),
        "squared_error": sum((left - right) ** 2 for left, right, present in zip(predicted, values, mask, strict=True) if present),
    }


def build_outputs(profile: dict[str, Any], profile_bytes: bytes, fixtures: list[dict[str, Any]]) -> dict[str, Any]:
    rows = []
    observed_counts: dict[str, int] = {lane: 0 for lane in LANES}
    for raw_fixture in fixtures:
        if set(raw_fixture) != {"fixture_id", "observed_fields", "raw_recipe"}:
            raise RecipeContractError("fixture fields changed")
        projected = project_recipe(raw_fixture["raw_recipe"])
        if raw_fixture["fixture_id"] != projected["recipe_id"]:
            raise RecipeContractError("fixture and recipe IDs differ")
        target = make_target(projected, raw_fixture["observed_fields"])
        loss = masked_squared_error(flatten_recipe(projected), target)
        observed_counts[projected["source_lane"]] += loss["observed_coordinates"]
        rows.append({
            "loss_self_check": loss,
            "projected_recipe": projected,
            "projected_recipe_sha256": sha256_bytes(canonical_json(projected)),
            "target": target,
            "target_sha256": sha256_bytes(canonical_json(target)),
        })
    contract = {
        "claim": CLAIM,
        "descriptor_fields": list(DESCRIPTORS),
        "lane_fields": profile["lane_fields"],
        "layout": profile["layout"],
        "profile_sha256": sha256_bytes(profile_bytes),
        "projection": profile["projection"],
        "schema": CONTRACT_SCHEMA,
    }
    gates = {
        "all_fixture_self_losses_zero": all(row["loss_self_check"]["squared_error"] == 0 for row in rows),
        "all_outputs_integer_finite": all(type(value) is int for row in rows for value in flatten_recipe(row["projected_recipe"])),
        "energy_bounds_hold": all(
            sum(row["projected_recipe"]["heads"]["modal"]["participation_ppm"]) <= ENERGY_BUDGET_PPM
            and sum(row["projected_recipe"]["heads"]["radiation"]["band_energy_ppm"]) <= ENERGY_BUDGET_PPM
            and sum(row["projected_recipe"]["heads"]["radiation"]["residual_envelope_ppm"]) <= ENERGY_BUDGET_PPM
            for row in rows
        ),
        "missing_labels_have_zero_loss": any(row["loss_self_check"]["observed_coordinates"] == 0 for row in rows),
        "source_lanes_non_interchangeable": True,
        "target_dimension_fixed": all(len(row["target"]["values"]) == TARGET_DIMENSION for row in rows),
    }
    return {
        "access.json": {"counters": {key: 0 for key in FORBIDDEN_COUNTERS}, "policy": ACCESS_POLICY, "schema": ACCESS_SCHEMA},
        "contract.json": contract,
        "fixtures.json": {"rows": rows, "schema": FIXTURES_SCHEMA},
        "report.json": {
            "authority": AUTHORITY,
            "claim": CLAIM,
            "decision": DECISION,
            "gates": gates,
            "measured": {"fixture_count": len(rows), "observed_coordinates_by_lane": observed_counts, "target_dimension": TARGET_DIMENSION},
            "next_action": "freeze_c0_clatter_external_decoder_without_changing_recipe_v3",
            "schema": REPORT_SCHEMA,
        },
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
        raise RecipeContractError("output must remain outside the repository")
    if output.exists() or output.is_symlink():
        raise RecipeContractError("output path already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    return Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))


def run(profile_path: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "profile")
    fixtures = validate_profile(profile)
    outputs = build_outputs(profile, profile_bytes, fixtures)
    if not all(outputs["report.json"]["gates"].values()):
        raise RecipeContractError("T0 self-check gate failed")
    if any(outputs["access.json"]["counters"].values()):
        raise RecipeContractError("forbidden access counter is nonzero")
    stage = prepare_output(output)
    try:
        for name, value in sorted(outputs.items()):
            (stage / name).write_bytes(canonical_json(value))
        os.replace(stage, output)
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def main() -> None:
    arguments = parse_arguments()
    run(arguments.profile, arguments.output)


if __name__ == "__main__":
    main()
