#!/usr/bin/env python3
"""Build the V44 G0 metadata-only YCB source and published plate controls."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from decimal import Decimal, ROUND_HALF_EVEN
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0-profile.v1"
SOURCE_SCHEMA = "nextengine.experimental-physical-sound-v44-c1-descriptor-source.v1"
PLATE_SCHEMA = "nextengine.experimental-physical-sound-v44-g0-plate-prospects.v1"
INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-v44-g0-source-inventory.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-g0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0-report.v1"
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v43-c0r-role-projection.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v44-g0-structured-source.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v44_g0_structured_source_v1.py"
YCB_FAMILY = "iri-csic-upc-ctu--ycb-impact-sounds"
YCB_PARENT_PREFIX = f"{YCB_FAMILY}--osf-bj5w8-2022-09-27--object-"
DECISION = "G0_STRUCTURED_SOURCE_INCREMENT_REPEATABLE_PSEL_BLOCKED"
CLAIM = (
    "METADATA_ONLY_DESCRIPTOR_AND_NUMERICAL_CONTROL_INCREMENT / "
    "NO_WAVEFORM_TRAINING_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
HASH_LENGTH = 64
MAX_METADATA_BYTES = 16 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "descriptor_source_authority": True,
    "numerical_control_prospect_authority": True,
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
    "outputs_external": True,
    "protected_access_allowed": False,
    "waveform_access_allowed": False,
}
SOURCE_POLICY = {
    "descriptor_fields": ["extent_micrometres", "mass_milligrams"],
    "license_expression": "NOASSERTION",
    "redistribution_policy": "external_research_only",
    "source_root_name": "ycb-object-properties",
}
EXPECTED_RESULT = {
    "c0r_ycb_parents": 29,
    "descriptor_artifacts": 2,
    "descriptor_observations": 26,
    "plate_prospects": 20,
    "priority_plate_prospects": 15,
}
FORBIDDEN_COUNTERS = (
    "audio_files_decoded",
    "feature_values_read",
    "model_values_read",
    "network_requests",
    "pcm_sample_values_decoded",
    "protected_values_read",
    "waveform_bytes_read",
)

# Reviewed bridge from current YCB model IDs to Table I in the CMU paper.
# Two dimensions denote an axisymmetric object and are expanded to [d, d, h].
# A single dimension denotes a near-isotropic object and is expanded to [d, d, d].
YCB_OBJECTS: dict[int, tuple[str, Decimal, tuple[Decimal, ...]]] = {
    2: ("Master Chef Can", Decimal("414"), (Decimal("102"), Decimal("139"))),
    3: ("Cracker Box", Decimal("411"), (Decimal("60"), Decimal("158"), Decimal("210"))),
    4: ("Sugar Box", Decimal("514"), (Decimal("38"), Decimal("89"), Decimal("175"))),
    5: ("Tomato Soup Can", Decimal("349"), (Decimal("66"), Decimal("101"))),
    6: (
        "Mustard Bottle",
        Decimal("603"),
        (Decimal("58"), Decimal("95"), Decimal("190")),
    ),
    7: ("Tuna Fish Can", Decimal("171"), (Decimal("85"), Decimal("33"))),
    8: ("Pudding Box", Decimal("187"), (Decimal("35"), Decimal("110"), Decimal("89"))),
    11: ("Banana", Decimal("66"), (Decimal("36"), Decimal("190"))),
    13: ("Apple", Decimal("68"), (Decimal("75"),)),
    19: ("Pitcher Base", Decimal("178"), (Decimal("108"), Decimal("235"))),
    20: ("Pitcher Lid", Decimal("66"), (Decimal("123"), Decimal("48"))),
    23: ("Wine Glass", Decimal("133"), (Decimal("89"), Decimal("137"))),
    24: ("Bowl", Decimal("147"), (Decimal("159"), Decimal("53"))),
    25: ("Mug", Decimal("118"), (Decimal("80"), Decimal("82"))),
    26: ("Sponge", Decimal("6.2"), (Decimal("72"), Decimal("114"), Decimal("14"))),
    27: ("Skillet", Decimal("950"), (Decimal("270"), Decimal("25"), Decimal("30"))),
    28: ("Skillet Lid", Decimal("652"), (Decimal("270"), Decimal("10"), Decimal("22"))),
    29: ("Plate", Decimal("279"), (Decimal("258"), Decimal("24"))),
    36: ("Wood Block", Decimal("729"), (Decimal("85"), Decimal("85"), Decimal("200"))),
    38: ("Padlock", Decimal("304"), (Decimal("24"), Decimal("47"), Decimal("65"))),
    41: ("Small Marker", Decimal("8.2"), (Decimal("8"), Decimal("135"))),
    48: ("Hammer", Decimal("665"), (Decimal("24"), Decimal("32"), Decimal("135"))),
    51: ("L Clamp", Decimal("125"), (Decimal("125"), Decimal("165"), Decimal("32"))),
    61: ("Foam Brick", Decimal("28"), (Decimal("50"), Decimal("75"), Decimal("50"))),
    68: ("Clear Box", Decimal("302"), (Decimal("292"), Decimal("429"), Decimal("149"))),
    70: ("Colored Wood Block", Decimal("10.8"), (Decimal("26"),)),
}

# IDs 60, 65 and 71 are deliberately excluded: the current corpus label cannot
# be mapped unambiguously to one scalar row in the 2015 paper.
EXPECTED_C0R_YCB_IDS = {
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    11,
    13,
    19,
    20,
    23,
    24,
    25,
    26,
    27,
    28,
    29,
    36,
    38,
    41,
    48,
    51,
    60,
    61,
    65,
    68,
    70,
    71,
}
YCB_SOURCE_TABLE_IDS = {
    **{object_id: object_id for object_id in YCB_OBJECTS},
    61: 60,
    68: 66,
    70: 69,
}
PLATE_MATERIALS = {"G": "Glass", "P": "Plastic", "S": "Steel", "W": "Wood"}
PLATE_SIDE_CENTIMETRES = {
    75: Decimal("8.66"),
    150: Decimal("12.24"),
    300: Decimal("17.32"),
    600: Decimal("24.49"),
    1200: Decimal("34.64"),
}
PLATE_COLUMNS = (
    "density_kg_per_cubic_metre",
    "auditory_loss_factor_times_1e_minus_3",
    "duration_seconds",
    "lowest_component_hertz",
    "attack_loudness_pseudo_sones",
    "mean_loudness_pseudo_sones",
    "early_loudness_slope_pseudo_sones_per_second",
    "late_loudness_slope_pseudo_sones_per_second",
    "attack_spectral_centroid_erb_rate",
    "mean_spectral_centroid_erb_rate",
    "spectral_centroid_slope_erb_rate_per_second",
)


class StructuredSourceError(RuntimeError):
    """G0 cannot publish a trustworthy metadata-only source increment."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--corpus-root", required=True, type=Path)
    parser.add_argument("--raw-root", required=True, type=Path)
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
        raise StructuredSourceError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise StructuredSourceError(f"{context} must be an object")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise StructuredSourceError(f"{context} must be a non-empty string")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise StructuredSourceError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_METADATA_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise StructuredSourceError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise StructuredSourceError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StructuredSourceError(f"{context} is not valid UTF-8 JSON") from error
    return require_dict(value, context)


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise StructuredSourceError(f"{context} binding fields changed")
    expected_path = require_string(binding["path"], f"{context} path")
    relative = Path(expected_path)
    if relative.is_absolute() or ".." in relative.parts:
        raise StructuredSourceError(f"{context} path escapes its root")
    expected_bytes = binding["bytes"]
    if type(expected_bytes) is not int or expected_bytes <= 0:
        raise StructuredSourceError(f"{context} byte count is invalid")
    require_hash(binding["sha256"], f"{context} SHA-256")
    data = read_regular(path, context)
    if len(data) != expected_bytes or sha256_bytes(data) != binding["sha256"]:
        raise StructuredSourceError(f"{context} binding mismatch")
    return data


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    value = parse_json(data, context)
    if canonical_json(value) != data:
        raise StructuredSourceError(f"{context} must use canonical JSON")
    return data, value


def validate_dependencies(profile: dict[str, Any]) -> None:
    dependencies = profile["dependency_bindings"]
    if not isinstance(dependencies, list) or not dependencies:
        raise StructuredSourceError("dependency bindings must be a non-empty array")
    paths = []
    for raw in dependencies:
        binding = require_dict(raw, "dependency binding")
        path_text = require_string(binding.get("path"), "dependency path")
        paths.append(path_text)
        check_binding(repository_root() / path_text, binding, f"dependency {path_text}")
    if paths != sorted(set(paths)):
        raise StructuredSourceError("dependencies must be path-sorted and unique")


def validate_profile(profile: dict[str, Any]) -> dict[str, Any]:
    required = {
        "access_policy",
        "authority",
        "claim",
        "dependency_bindings",
        "expected_result",
        "input_bindings",
        "profile_id",
        "raw_inputs",
        "schema",
        "source_policy",
    }
    if set(profile) != required or profile.get("schema") != PROFILE_SCHEMA:
        raise StructuredSourceError("G0 profile fields or schema changed")
    if profile.get("access_policy") != ACCESS_POLICY:
        raise StructuredSourceError("G0 access policy changed")
    if profile.get("authority") != AUTHORITY or profile.get("claim") != CLAIM:
        raise StructuredSourceError("G0 authority changed")
    if profile.get("expected_result") != EXPECTED_RESULT:
        raise StructuredSourceError("G0 expected result changed")
    if profile.get("source_policy") != SOURCE_POLICY:
        raise StructuredSourceError("G0 source policy changed")
    require_string(profile.get("profile_id"), "profile ID")
    validate_dependencies(profile)
    inputs = require_dict(profile["input_bindings"], "input bindings")
    if set(inputs) != {"development_projection", "train_projection"}:
        raise StructuredSourceError("G0 corpus input set changed")
    raw_inputs = require_dict(profile["raw_inputs"], "raw inputs")
    if set(raw_inputs) != {
        "plate_paper_pdf",
        "plate_paper_text",
        "ycb_paper_pdf",
        "ycb_paper_text",
    }:
        raise StructuredSourceError("G0 raw input set changed")
    for group, bindings in (("input", inputs), ("raw", raw_inputs)):
        for name, raw in bindings.items():
            binding = require_dict(raw, f"{group} binding {name}")
            if set(binding) != {"bytes", "path", "sha256"}:
                raise StructuredSourceError(f"{group} binding {name} fields changed")
            require_hash(binding["sha256"], f"{group} binding {name} SHA-256")
    return profile


def load_bound(root: Path, binding: dict[str, Any], context: str) -> bytes:
    if root.is_symlink() or not root.is_dir():
        raise StructuredSourceError(f"{context} root must be a non-symlink directory")
    relative = Path(binding["path"])
    target = root / relative
    if target.is_symlink() or not target.resolve().is_relative_to(root.resolve()):
        raise StructuredSourceError(f"{context} path escapes its root")
    return check_binding(target, binding, context)


def c0r_ycb_ids(profile: dict[str, Any], corpus_root: Path) -> set[int]:
    identifiers: set[int] = set()
    for name, role in (
        ("development_projection", "generator_development"),
        ("train_projection", "generator_train"),
    ):
        data = load_bound(corpus_root, profile["input_bindings"][name], name)
        projection = parse_json(data, name)
        if (
            projection.get("schema") != PROJECTION_SCHEMA
            or projection.get("role") != role
            or not isinstance(projection.get("rows"), list)
        ):
            raise StructuredSourceError(f"{name} contract changed")
        for row in projection["rows"]:
            if not isinstance(row, dict):
                raise StructuredSourceError(f"{name} row is invalid")
            if row.get("family_id") != YCB_FAMILY:
                continue
            parent = require_string(row.get("physical_parent_id"), "YCB parent ID")
            if not parent.startswith(YCB_PARENT_PREFIX):
                raise StructuredSourceError("YCB parent namespace changed")
            suffix = parent.removeprefix(YCB_PARENT_PREFIX)
            if not suffix.isdigit():
                raise StructuredSourceError("YCB parent object ID is invalid")
            identifiers.add(int(suffix))
    if identifiers != EXPECTED_C0R_YCB_IDS:
        raise StructuredSourceError("C0R YCB parent identity set changed")
    return identifiers


def decimal_to_int(value: Decimal) -> int:
    return int(value.quantize(Decimal("1"), rounding=ROUND_HALF_EVEN))


def extent_micrometres(dimensions: tuple[Decimal, ...]) -> list[int]:
    if len(dimensions) == 1:
        values = (dimensions[0], dimensions[0], dimensions[0])
    elif len(dimensions) == 2:
        values = (dimensions[0], dimensions[0], dimensions[1])
    elif len(dimensions) == 3:
        values = dimensions
    else:
        raise StructuredSourceError("reviewed YCB extent has invalid arity")
    return [decimal_to_int(value * 1000) for value in values]


def claim(value: Any) -> dict[str, Any]:
    return {
        "confidence_observed": False,
        "confidence_ppm": None,
        "evidence_artifact_ids": ["ycb-paper-pdf", "ycb-paper-text"],
        "value": value,
    }


def descriptor_manifest(raw: dict[str, bytes]) -> dict[str, Any]:
    artifacts = []
    for artifact_id, key, media_type, filename in (
        (
            "ycb-paper-pdf",
            "ycb_paper_pdf",
            "application/pdf",
            "ycb-object-model-set-icar-2015.pdf",
        ),
        (
            "ycb-paper-text",
            "ycb_paper_text",
            "text/plain",
            "ycb-object-model-set-icar-2015.txt",
        ),
    ):
        data = raw[key]
        artifacts.append(
            {
                "artifact_id": artifact_id,
                "bytes": len(data),
                "media_type": media_type,
                "path": filename,
                "sha256": sha256_bytes(data),
                "source_url": "https://www.ri.cmu.edu/pub_files/2015/7/ICAR-FINAL.pdf",
            }
        )
    observations = []
    for object_id, (_name, mass_grams, dimensions) in sorted(YCB_OBJECTS.items()):
        source_table_id = YCB_SOURCE_TABLE_IDS[object_id]
        observations.append(
            {
                "fields": {
                    "extent_micrometres": claim(extent_micrometres(dimensions)),
                    "mass_milligrams": claim(decimal_to_int(mass_grams * 1000)),
                },
                "observation_id": (
                    f"ycb-corpus-object-{object_id:03d}-"
                    f"source-table-{source_table_id:03d}-mass-extents"
                ),
                "physical_parent_id": f"{YCB_PARENT_PREFIX}{object_id}",
                "record_id": None,
            }
        )
    return {
        "artifacts": artifacts,
        "declared_revision": "icar-2015-table-i-reviewed-transcription-v1",
        "landing_page_url": "https://www.ri.cmu.edu/publications/ycb-object-model-set-towards-common-benchmarks-manipulation-research/",
        "license_expression": SOURCE_POLICY["license_expression"],
        "observations": observations,
        "project_id": "cmu-ycb-object-and-model-set",
        "publisher_id": "carnegie-mellon-university-robotics-institute",
        "redistribution_policy": SOURCE_POLICY["redistribution_policy"],
        "schema": SOURCE_SCHEMA,
        "source_component_id": "source-component.cmu-ycb-object-properties.v1",
        "source_id": "calli-etal-ycb-object-model-set-icar-2015",
        "terms_url": None,
    }


def parse_plate_table(text: str) -> list[dict[str, Any]]:
    pattern = re.compile(
        r"^\s*([SGWP])\s+(75|150|300|600|1200)\s+"
        + r"\s+".join([r"([+\-−]?\d+(?:\.\d+)?)"] * 11)
        + r"\s*$"
    )
    parsed: dict[tuple[str, int], list[Decimal]] = {}
    for line in text.splitlines():
        match = pattern.match(line)
        if not match:
            continue
        material = match.group(1)
        area = int(match.group(2))
        values = [Decimal(value.replace("−", "-")) for value in match.groups()[2:]]
        key = (material, area)
        if key in parsed:
            raise StructuredSourceError("published plate table contains duplicate rows")
        parsed[key] = values
    expected = {
        (material, area)
        for material in PLATE_MATERIALS
        for area in PLATE_SIDE_CENTIMETRES
    }
    if set(parsed) != expected:
        raise StructuredSourceError("published plate table row set changed")
    rows = []
    for (material_code, area), values in sorted(parsed.items()):
        source_values = dict(zip(PLATE_COLUMNS, values, strict=True))
        side = PLATE_SIDE_CENTIMETRES[area]
        density = source_values["density_kg_per_cubic_metre"]
        target_scale = Decimal("1000000")
        rows.append(
            {
                "acoustic_targets_fixed_point": {
                    "attack_loudness_micro_pseudo_sones": decimal_to_int(
                        source_values["attack_loudness_pseudo_sones"] * target_scale
                    ),
                    "attack_spectral_centroid_micro_erb_rate": decimal_to_int(
                        source_values["attack_spectral_centroid_erb_rate"]
                        * target_scale
                    ),
                    "auditory_loss_factor_micro": decimal_to_int(
                        source_values["auditory_loss_factor_times_1e_minus_3"] * 1000
                    ),
                    "duration_microseconds": decimal_to_int(
                        source_values["duration_seconds"] * target_scale
                    ),
                    "early_loudness_slope_micro_pseudo_sones_per_second": decimal_to_int(
                        source_values["early_loudness_slope_pseudo_sones_per_second"]
                        * target_scale
                    ),
                    "late_loudness_slope_micro_pseudo_sones_per_second": decimal_to_int(
                        source_values["late_loudness_slope_pseudo_sones_per_second"]
                        * target_scale
                    ),
                    "lowest_component_millihertz": decimal_to_int(
                        source_values["lowest_component_hertz"] * 1000
                    ),
                    "mean_loudness_micro_pseudo_sones": decimal_to_int(
                        source_values["mean_loudness_pseudo_sones"] * target_scale
                    ),
                    "mean_spectral_centroid_micro_erb_rate": decimal_to_int(
                        source_values["mean_spectral_centroid_erb_rate"] * target_scale
                    ),
                    "spectral_centroid_slope_micro_erb_rate_per_second": decimal_to_int(
                        source_values["spectral_centroid_slope_erb_rate_per_second"]
                        * target_scale
                    ),
                },
                "area_square_centimetres": area,
                "descriptor": {
                    "cavity_kind": "none",
                    "characteristic_wall_thickness_micrometres": 2000,
                    "extent_micrometres": [
                        decimal_to_int(side * 10000),
                        decimal_to_int(side * 10000),
                        2000,
                    ],
                    "impact_zone": "face",
                    "mass_milligrams": decimal_to_int(
                        density * Decimal(area) * Decimal("0.2")
                    ),
                    "material_label": PLATE_MATERIALS[material_code],
                    "opening_kind": "none",
                    "shape_topology": "thin_plate",
                    "support_condition": "suspended",
                },
                "priority_material": material_code in {"G", "S", "W"},
                "prospect_id": f"giordano-mcadams-2006-{material_code.lower()}-{area:04d}",
                "source_material_code": material_code,
            }
        )
    return rows


def load_raw(profile: dict[str, Any], raw_root: Path) -> dict[str, bytes]:
    raw = {
        name: load_bound(raw_root, binding, f"raw input {name}")
        for name, binding in sorted(profile["raw_inputs"].items())
    }
    ycb_text = raw["ycb_paper_text"].decode("utf-8")
    plate_text = raw["plate_paper_text"].decode("utf-8")
    for marker in (
        "Table I: Object Set Items and Properties",
        "Wine glass",
        "Wood Block",
    ):
        if marker not in ycb_text:
            raise StructuredSourceError("YCB paper extraction markers changed")
    for marker in (
        "2-mm-thick square",
        "45 cm from the center",
        "TABLE I. Acoustical descriptors",
    ):
        if marker not in plate_text:
            raise StructuredSourceError("plate paper extraction markers changed")
    return raw


def build_documents(
    profile_bytes: bytes, profile: dict[str, Any], corpus_root: Path, raw_root: Path
) -> dict[str, bytes]:
    ycb_ids = c0r_ycb_ids(profile, corpus_root)
    raw = load_raw(profile, raw_root)
    plates = parse_plate_table(raw["plate_paper_text"].decode("utf-8"))
    manifest = descriptor_manifest(raw)
    actual = {
        "c0r_ycb_parents": len(ycb_ids),
        "descriptor_artifacts": len(manifest["artifacts"]),
        "descriptor_observations": len(manifest["observations"]),
        "plate_prospects": len(plates),
        "priority_plate_prospects": sum(row["priority_material"] for row in plates),
    }
    if actual != EXPECTED_RESULT:
        raise StructuredSourceError("G0 result counts changed")
    access = {counter: 0 for counter in FORBIDDEN_COUNTERS}
    access.update(
        {
            "metadata_artifacts_read": 4,
            "projection_documents_read": 2,
            "schema": ACCESS_SCHEMA,
        }
    )
    plate_document = {
        "authority": {
            "candidate_training_authority": False,
            "numerical_control_only": True,
            "target_correspondence_required_before_use": True,
            "validator_calibration_authority": False,
            "waveform_evidence_present": False,
        },
        "plates": plates,
        "schema": PLATE_SCHEMA,
        "source": {
            "citation": "Giordano and McAdams, JASA 119(2), 2006",
            "source_url": "https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf",
        },
    }
    inventory = {
        "c0r_ycb_parent_ids": sorted(ycb_ids),
        "descriptor_parent_ids": sorted(YCB_OBJECTS),
        "descriptor_parent_to_source_table_id": {
            str(object_id): YCB_SOURCE_TABLE_IDS[object_id]
            for object_id in sorted(YCB_OBJECTS)
        },
        "excluded_ambiguous_ycb_parent_ids": sorted(
            EXPECTED_C0R_YCB_IDS - set(YCB_OBJECTS)
        ),
        "plate_material_counts": {
            material: sum(
                row["descriptor"]["material_label"] == material for row in plates
            )
            for material in sorted(PLATE_MATERIALS.values())
        },
        "raw_artifacts": [
            {
                "bytes": len(data),
                "input_id": name,
                "sha256": sha256_bytes(data),
                "source_url": (
                    "https://www.ri.cmu.edu/pub_files/2015/7/ICAR-FINAL.pdf"
                    if name.startswith("ycb_")
                    else "https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf"
                ),
            }
            for name, data in sorted(raw.items())
        ],
        "schema": INVENTORY_SCHEMA,
    }
    report = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": DECISION,
        "gates": {
            "c0r_identity_exact": True,
            "metadata_only_access": all(
                access[counter] == 0 for counter in FORBIDDEN_COUNTERS
            ),
            "plate_table_complete": True,
            "psel_power_sufficient": False,
            "repeat_exact_required": True,
            "source_hashes_exact": True,
            "target_correspondence_proven": False,
        },
        "limitations": [
            "plate_control_has_no_published_waveform_binding",
            "plate_control_is_one_publisher_project_and_cannot_satisfy_PSEL",
            "YCB_properties_do_not_publish_wall_thickness_or_record_level_contact_conditions",
            "YCB_60_65_71_are_excluded_as_ambiguous_table_mappings",
        ],
        "next_actions": [
            "run_c1_successor_with_ycb_descriptor_source",
            "research_independent_structured_sources_with_waveform_or_transfer_correspondence",
            "define_target_correspondence_check_before_plate_control_use",
        ],
        "profile_bytes": len(profile_bytes),
        "profile_sha256": sha256_bytes(profile_bytes),
        "result": actual,
        "schema": REPORT_SCHEMA,
    }
    return {
        "access-ledger.json": canonical_json(access),
        "giordano-mcadams-2006-plates.pdf": raw["plate_paper_pdf"],
        "giordano-mcadams-2006-plates.txt": raw["plate_paper_text"],
        "manifest.json": canonical_json(manifest),
        "plate-prospects.json": canonical_json(plate_document),
        "profile.json": profile_bytes,
        "report.json": canonical_json(report),
        "source-inventory.json": canonical_json(inventory),
        "ycb-object-model-set-icar-2015.pdf": raw["ycb_paper_pdf"],
        "ycb-object-model-set-icar-2015.txt": raw["ycb_paper_text"],
    }


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise StructuredSourceError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise StructuredSourceError("output must remain outside the repository")
    if resolved.exists():
        raise StructuredSourceError(f"refusing to replace existing output: {resolved}")
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
                raise StructuredSourceError("G0 outputs must be flat files")
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(profile_path: Path, corpus_root: Path, raw_root: Path, output: Path) -> Path:
    profile_bytes, raw_profile = read_canonical_json(profile_path, "G0 profile")
    profile = validate_profile(raw_profile)
    return publish_directory(
        output,
        build_documents(
            profile_bytes, profile, corpus_root.resolve(), raw_root.resolve()
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    print(
        run(
            arguments.profile,
            arguments.corpus_root,
            arguments.raw_root,
            arguments.output,
        )
    )


if __name__ == "__main__":
    main()
