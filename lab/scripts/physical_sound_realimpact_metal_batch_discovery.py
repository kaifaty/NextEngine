#!/usr/bin/env python3
"""Freeze and discover an object-grouped REALIMPACT metal batch."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

import physical_sound_realimpact_observation_discovery as discovery

MANIFEST_SCHEMA = "nextengine.experimental-realimpact-metal-batch-discovery.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-metal-batch-discovery-preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-metal-batch-discovery-acquisition.report.v1"
    ),
    "audit": (
        "nextengine.experimental-realimpact-metal-batch-discovery-audit.report.v1"
    ),
}
STUDY_ID = "physical-sound-realimpact-object-grouped-metal-batch"
REVISION = "explicit-metal-name-families-report-only-v1"
ROSTER_COMMIT = "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987"
ROSTER_SHA256 = "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
CORE_SHA256 = "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
TAIL_BYTES = 65_536
LOCAL_HEADER_BYTES = 30
EXPECTED_ENTRY_COUNT = 12

REGISTRY_REFERENCES = [
    {
        "label": "modal_observation_registry",
        "path": "../ps2-modal-observation-registry-v0/build-a/registry.json",
        "sha256": ("e0bec857edee2af087ff970631dcc8c389803c3fcde24b2947edbce224e09e98"),
        "bytes": 277_667,
    },
    {
        "label": "modal_observation_registry_report",
        "path": "../ps2-modal-observation-registry-v0/build-a/report.json",
        "sha256": ("34a7f209e446cb0261bff4231f7401899501000543eb554501f10ddd80cdaed0"),
        "bytes": 2_609,
    },
]

SOURCE_REPORTS = [
    {
        "label": "iron_skillet_broadband_report",
        "path": (
            "../ps2-realimpact-iron-skillet-broadband-v2-counterfactual-v1/"
            "analysis-a/report.json"
        ),
        "sha256": ("f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73"),
        "bytes": 924_196,
    },
    {
        "label": "shape_spatial_development_report",
        "path": ("../ps2-realimpact-shape-spatial-v1/development-a/report.json"),
        "sha256": ("3d18358b50442ee9b171bbacc154f7713e3240efa6f4d50f61f2598b7ac4962f"),
        "bytes": 532_236,
    },
    {
        "label": "iron_mortar_broadband_report",
        "path": (
            "../ps2-realimpact-iron-mortar-broadband-holdout-v1/analysis-a/report.json"
        ),
        "sha256": ("64bc63aa0cd02cfe9ad3959670102e883893f7d53a1427fddb5f660a8951d0cc"),
        "bytes": 696_263,
    },
]

EXISTING_OBSERVATIONS = [
    {
        "dataset_object_id": "17_IronSkillet",
        "family_group": "iron_skillet",
        "role": "development",
        "path": (
            "../ps2-realimpact-iron-skillet-observation-v1/observation-decode/"
            "iron-skillet-impact000-rows000-599.f32le"
        ),
        "sha256": ("e26d1df1d547d059bbcbc57453c522b8c23402461feec09ff6149bc37a9c7eeb"),
        "bytes": 553_317_600,
        "dtype": "<f4",
        "stored_rows": 600,
        "samples_per_row": 230_549,
        "batch_rows": list(range(15)),
    },
    {
        "dataset_object_id": "67_IronPlate",
        "family_group": "iron_plate",
        "role": "development",
        "path": (
            "../ps2-realimpact-shape-spatial-v1/development-a/"
            "67_IronPlate-base-selected-block.f32le"
        ),
        "sha256": ("19dce5da73487ee81bd44b7272258fcf4c926afe9db8bf29967dfe2a75351c14"),
        "bytes": 12_522_540,
        "dtype": "<f4",
        "stored_rows": 15,
        "samples_per_row": 208_709,
        "batch_rows": list(range(15)),
    },
    {
        "dataset_object_id": "90_MetalLadle",
        "family_group": "metal_ladle",
        "role": "development",
        "path": (
            "../ps2-realimpact-shape-spatial-v1/development-a/"
            "90_MetalLadle-base-selected-block.f32le"
        ),
        "sha256": ("be38f1eadd302ea3957d5c7296bff4c3dd7ff9fb550129b2c86b00b08648dcb5"),
        "bytes": 12_497_100,
        "dtype": "<f4",
        "stored_rows": 15,
        "samples_per_row": 208_285,
        "batch_rows": list(range(15)),
    },
    {
        "dataset_object_id": "43_IronMortar",
        "family_group": "iron_mortar",
        "role": "calibration",
        "path": (
            "../ps2-realimpact-iron-mortar-broadband-holdout-v1/decode/"
            "iron-mortar-impact000-condition000-mics00-14.f32le"
        ),
        "sha256": ("19108dc93b4b3ee5fcc43fd1777454e25cfa99043483d93e636f68c69928d948"),
        "bytes": 12_502_500,
        "dtype": "<f4",
        "stored_rows": 15,
        "samples_per_row": 208_375,
        "batch_rows": list(range(15)),
    },
]

NEW_OBJECTS = [
    {
        "dataset_object_id": "86_MetalHoledSpoon",
        "roster_index": 40,
        "family_group": "metal_holed_spoon",
        "role": "calibration",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/"
                "86_MetalHoledSpoon.zip"
            ),
            "bytes": 2_310_510_455,
            "etag": '"6433e1b7-89b79777"',
            "last_modified_http": "Mon, 10 Apr 2023 10:15:19 GMT",
            "accept_ranges": "bytes",
            "head_requests_before_freeze": 1,
        },
    },
    {
        "dataset_object_id": "91_MetalSpoon",
        "roster_index": 43,
        "family_group": "metal_spoon",
        "role": "holdout",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/91_MetalSpoon.zip"
            ),
            "bytes": 2_312_590_327,
            "etag": '"6433e360-89d753f7"',
            "last_modified_http": "Mon, 10 Apr 2023 10:22:24 GMT",
            "accept_ranges": "bytes",
            "head_requests_before_freeze": 2,
        },
    },
    {
        "dataset_object_id": "89_MetalSpatula",
        "roster_index": 41,
        "family_group": "metal_spatula",
        "role": "shadow",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/"
                "89_MetalSpatula.zip"
            ),
            "bytes": 2_308_676_719,
            "etag": '"6433e245-899b9c6f"',
            "last_modified_http": "Mon, 10 Apr 2023 10:17:41 GMT",
            "accept_ranges": "bytes",
            "head_requests_before_freeze": 2,
        },
    },
    {
        "dataset_object_id": "92_MetalSpatula",
        "roster_index": 44,
        "family_group": "metal_spatula",
        "role": "shadow",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/"
                "92_MetalSpatula.zip"
            ),
            "bytes": 2_313_811_870,
            "etag": '"6433e3e8-89e9f79e"',
            "last_modified_http": "Mon, 10 Apr 2023 10:24:40 GMT",
            "accept_ranges": "bytes",
            "head_requests_before_freeze": 2,
        },
    },
]


class BatchDiscoveryError(RuntimeError):
    """The frozen metal batch or archive-discovery boundary changed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage", required=True, choices=["manifest", "preflight", "acquire", "audit"]
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--preflight", type=Path)
    parser.add_argument("--cache", type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner": {
            "path": ("lab/scripts/physical_sound_realimpact_metal_batch_discovery.py"),
            "sha256": runner_sha256,
        },
        "core": {
            "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
            "sha256": CORE_SHA256,
        },
        "parent_registry": {
            "references": REGISTRY_REFERENCES,
            "required_decision": "ModalObservationRegistryV0BuiltFallbackOnly",
            "quality_domain_runtime_admission_required_false": True,
        },
        "source_reports": SOURCE_REPORTS,
        "official_roster": {
            "repository_commit": ROSTER_COMMIT,
            "object_names_sha256": ROSTER_SHA256,
            "selection_rule": (
                "all exact official-roster names containing Iron or Metal; "
                "all equal normalized name families remain in one role"
            ),
            "eligible_object_ids": [
                "17_IronSkillet",
                "43_IronMortar",
                "67_IronPlate",
                "86_MetalHoledSpoon",
                "89_MetalSpatula",
                "90_MetalLadle",
                "91_MetalSpoon",
                "92_MetalSpatula",
            ],
        },
        "existing_observations": EXISTING_OBSERVATIONS,
        "new_objects": NEW_OBJECTS,
        "partition": {
            "development": [
                "17_IronSkillet",
                "67_IronPlate",
                "90_MetalLadle",
            ],
            "calibration": ["43_IronMortar", "86_MetalHoledSpoon"],
            "holdout": ["91_MetalSpoon"],
            "shadow": ["89_MetalSpatula", "92_MetalSpatula"],
            "family_group_disjoint": True,
            "opened_seed_objects_forbidden_in_holdout_or_shadow": True,
            "statistical_release_credit": False,
        },
        "requests": {
            "maximum_network_requests": 2 * len(NEW_OBJECTS),
            "tail_bytes_per_archive": TAIL_BYTES,
            "audio_local_header_bytes_per_archive": LOCAL_HEADER_BYTES,
            "audio_or_metadata_member_payload_bytes_allowed": 0,
            "retry_or_object_substitution_allowed": False,
        },
        "next_protocol": {
            "sample_rate_hz": 48_000,
            "analysis_rows_per_object": list(range(15)),
            "minimum_samples_per_row": 32_768,
            "calibration_stop_sample": 8_192,
            "evaluation_windows": [[8_192, 16_384], [16_384, 32_768]],
            "candidate_families": [
                "modal_only_common_pole_v2",
                "modal_plus_parametric_transient_v0",
                "modal_plus_seeded_subband_residual_v0",
            ],
            "candidate_parameters_and_gates_frozen_before_audio_payload": True,
            "per_object_human_admission_allowed": False,
            "automatic_fallback_required": True,
        },
        "data_policy": {
            "central_directory_and_audio_local_header_only": True,
            "new_audio_or_metadata_member_payload_allowed": False,
            "planter_payload_allowed": False,
            "physics_solver_allowed": False,
            "quality_domain_or_runtime_admission_allowed": False,
        },
        "stop_rule": (
            "discover and audit only the four frozen new archive identities; "
            "then freeze exact metadata/audio ranges plus candidate parameters "
            "and gates before any new member payload access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise BatchDiscoveryError("metal batch manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise BatchDiscoveryError(f"parse metal batch manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise BatchDiscoveryError("metal batch manifest changed")
    return data, manifest


def resolve_reference(base: Path, reference: dict[str, Any]) -> Path:
    path = (base / reference["path"]).resolve(strict=True)
    store = base.parent.resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(store):
        raise BatchDiscoveryError(f"reference escapes physical-sound store: {path}")
    if path.stat().st_size != reference["bytes"]:
        raise BatchDiscoveryError(f"reference byte length changed: {reference['path']}")
    if sha256_file(path) != reference["sha256"]:
        raise BatchDiscoveryError(f"reference hash changed: {reference['path']}")
    return path


def validate_core(root: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    reference = manifest["core"]
    path = (root / reference["path"]).resolve(strict=True)
    if (
        not path.is_file()
        or not path.is_relative_to(root)
        or sha256_file(path) != reference["sha256"]
    ):
        raise BatchDiscoveryError("bound discovery core changed")
    return {
        "path": reference["path"],
        "sha256": reference["sha256"],
        "bytes": path.stat().st_size,
    }


def validate_fixed_references(
    base: Path, manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    references = [
        *manifest["parent_registry"]["references"],
        *manifest["source_reports"],
    ]
    result = []
    for reference in references:
        resolve_reference(base, reference)
        result.append(reference)
    registry_report_path = resolve_reference(
        base, manifest["parent_registry"]["references"][1]
    )
    registry_report = json.loads(registry_report_path.read_bytes())
    if (
        registry_report.get("decision")
        != manifest["parent_registry"]["required_decision"]
        or registry_report.get("all_quality_domain_runtime_admissions_disabled")
        is not True
        or registry_report.get("all_fallbacks_required") is not True
    ):
        raise BatchDiscoveryError("parent modal registry state changed")
    return result


def validate_existing_observations(
    base: Path, manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    result = []
    for reference in manifest["existing_observations"]:
        resolve_reference(base, reference)
        if (
            reference["bytes"]
            != reference["stored_rows"] * reference["samples_per_row"] * 4
            or reference["batch_rows"] != list(range(15))
            or reference["samples_per_row"] < 32_768
        ):
            raise BatchDiscoveryError("existing observation shape changed")
        result.append(reference)
    return result


def expected_preflight(
    manifest_bytes: bytes,
    runner_sha256: str,
    core: dict[str, Any],
    fixed_references: list[dict[str, Any]],
    observations: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "study_id": STUDY_ID,
        "revision": REVISION,
        "decision": "RealImpactMetalBatchRolesAndDiscoveryInputsFrozen",
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "bound_core": core,
        "fixed_references": fixed_references,
        "existing_observations": observations,
        "partition": expected_manifest(runner_sha256)["partition"],
        "new_objects": NEW_OBJECTS,
        "existing_waveform_bytes_hashed": sum(item["bytes"] for item in observations),
        "report_and_registry_bytes_hashed": sum(
            item["bytes"] for item in fixed_references
        ),
        "network_requests": 0,
        "new_object_member_payload_bytes_read": 0,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_domain_or_runtime_admission": False,
        "next_action": (
            "read each exact archive tail and observation local header once; "
            "open no member payload"
        ),
    }


def configure_archive(item: dict[str, Any]) -> None:
    archive = item["archive"]
    object_id = item["dataset_object_id"]
    discovery.OBJECT_ID = object_id
    discovery.ARCHIVE_URL = archive["url"]
    discovery.ARCHIVE_BYTES = archive["bytes"]
    discovery.ARCHIVE_ETAG = archive["etag"]
    discovery.ARCHIVE_LAST_MODIFIED = archive["last_modified_http"]
    discovery.TAIL_BYTES = TAIL_BYTES
    discovery.TAIL_START = archive["bytes"] - TAIL_BYTES
    discovery.LOCAL_HEADER_BYTES = LOCAL_HEADER_BYTES
    discovery.EXPECTED_ENTRY_COUNT = EXPECTED_ENTRY_COUNT
    discovery.EXPECTED_AUDIO_ENTRY = f"{object_id}/preprocessed/deconvolved_0db.npy"
    discovery.ROSTER_SHA256 = ROSTER_SHA256


def discover_new_object(
    item: dict[str, Any],
) -> tuple[dict[str, Any], dict[str, bytes]]:
    configure_archive(item)
    archive = item["archive"]
    tail_start = archive["bytes"] - TAIL_BYTES
    tail, tail_response = discovery.range_get(tail_start, TAIL_BYTES)
    central, entries = discovery.discover_from_tail(tail)
    audio = next(
        entry
        for entry in entries
        if entry["name"]
        == f"{item['dataset_object_id']}/preprocessed/deconvolved_0db.npy"
    )
    header, header_response = discovery.range_get(
        audio["local_offset"], LOCAL_HEADER_BYTES
    )
    audio_header = discovery.validate_audio_local_header(header, audio)
    prefix = item["dataset_object_id"]
    return (
        {
            "dataset_object_id": item["dataset_object_id"],
            "family_group": item["family_group"],
            "role": item["role"],
            "archive": archive,
            "tail_response": tail_response,
            "central_directory": central,
            "entries": entries,
            "observation_entry": audio,
            "observation_local_header": audio_header,
            "audio_local_header_response": header_response,
        },
        {
            f"{prefix}-tail.bin": tail,
            f"{prefix}-audio-local-header.bin": header,
        },
    )


def load_preflight(
    path: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise BatchDiscoveryError(f"parse metal batch preflight: {error}") from error
    expected = expected_preflight(
        manifest_bytes,
        runner_sha256,
        validate_core(Path(__file__).resolve().parents[2], manifest),
        [
            *manifest["parent_registry"]["references"],
            *manifest["source_reports"],
        ],
        manifest["existing_observations"],
    )
    if report != expected:
        raise BatchDiscoveryError("metal batch preflight changed")
    return data, report


def audit_cache(
    cache: Path,
    acquisition: dict[str, Any],
    manifest_bytes: bytes,
    runner_sha256: str,
) -> list[dict[str, Any]]:
    if (
        acquisition.get("schema") != REPORT_SCHEMAS["acquire"]
        or acquisition.get("decision") != "RealImpactMetalBatchArchivesDiscovered"
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("network_requests") != 8
        or acquisition.get("new_object_member_payload_bytes_read") != 0
    ):
        raise BatchDiscoveryError("cached batch acquisition changed")
    audited = []
    for item, observed in zip(NEW_OBJECTS, acquisition.get("objects", []), strict=True):
        configure_archive(item)
        prefix = item["dataset_object_id"]
        tail = (cache / f"{prefix}-tail.bin").read_bytes()
        header = (cache / f"{prefix}-audio-local-header.bin").read_bytes()
        central, entries = discovery.discover_from_tail(tail)
        audio = next(
            entry
            for entry in entries
            if entry["name"] == f"{prefix}/preprocessed/deconvolved_0db.npy"
        )
        audio_header = discovery.validate_audio_local_header(header, audio)
        if (
            observed.get("dataset_object_id") != prefix
            or observed.get("central_directory") != central
            or observed.get("entries") != entries
            or observed.get("observation_entry") != audio
            or observed.get("observation_local_header") != audio_header
            or observed.get("tail_response", {}).get("sha256") != sha256_bytes(tail)
            or observed.get("audio_local_header_response", {}).get("sha256")
            != sha256_bytes(header)
        ):
            raise BatchDiscoveryError(f"cached discovery changed: {prefix}")
        audited.append(
            {
                "dataset_object_id": prefix,
                "family_group": item["family_group"],
                "role": item["role"],
                "tail_sha256": sha256_bytes(tail),
                "audio_local_header_sha256": sha256_bytes(header),
                "central_directory": central,
                "observation_entry": audio,
                "observation_local_header": audio_header,
            }
        )
    return audited


def write_new_file(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise BatchDiscoveryError(f"refusing to replace existing output: {path}")
    descriptor, staging_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    staging = Path(staging_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(staging, path)
    finally:
        staging.unlink(missing_ok=True)


def publish_directory(output: Path, files: dict[str, bytes]) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise BatchDiscoveryError(f"refusing to replace existing output: {output}")
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, data in files.items():
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        os.replace(staging, output)
    finally:
        if staging.exists():
            shutil.rmtree(staging)


def main() -> None:
    arguments = parse_arguments()
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "manifest":
        if any(
            value is not None
            for value in [arguments.manifest, arguments.preflight, arguments.cache]
        ):
            raise BatchDiscoveryError("manifest stage accepts only --output")
        write_new_file(
            arguments.output, canonical_json(expected_manifest(runner_sha256))
        )
        return
    if arguments.manifest is None:
        raise BatchDiscoveryError(f"{arguments.stage} stage requires --manifest")
    manifest_bytes, manifest = load_manifest(arguments.manifest, runner_sha256)
    root = Path(__file__).resolve().parents[2]
    base = arguments.manifest.parent
    if arguments.stage == "preflight":
        if arguments.preflight is not None or arguments.cache is not None:
            raise BatchDiscoveryError("preflight accepts no preflight/cache input")
        core = validate_core(root, manifest)
        references = validate_fixed_references(base, manifest)
        observations = validate_existing_observations(base, manifest)
        report = expected_preflight(
            manifest_bytes, runner_sha256, core, references, observations
        )
        publish_directory(arguments.output, {"report.json": canonical_json(report)})
        return
    if arguments.preflight is None:
        raise BatchDiscoveryError(f"{arguments.stage} stage requires --preflight")
    preflight_bytes, _ = load_preflight(
        arguments.preflight, manifest_bytes, manifest, runner_sha256
    )
    if arguments.stage == "acquire":
        if arguments.cache is not None:
            raise BatchDiscoveryError("acquire does not accept --cache")
        objects = []
        artifacts = {}
        for item in manifest["new_objects"]:
            observed, observed_artifacts = discover_new_object(item)
            objects.append(observed)
            artifacts.update(observed_artifacts)
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "study_id": STUDY_ID,
            "revision": REVISION,
            "decision": "RealImpactMetalBatchArchivesDiscovered",
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "runner_sha256": runner_sha256,
            "preflight_sha256": sha256_bytes(preflight_bytes),
            "partition": manifest["partition"],
            "objects": objects,
            "network_requests": 2 * len(objects),
            "new_object_member_payload_bytes_read": 0,
            "planter_payload_bytes_read": 0,
            "physics_solver_runs": 0,
            "quality_domain_or_runtime_admission": False,
            "next_action": (
                "audit this cache twice, then freeze exact member ranges, "
                "candidate parameters and gates before payload access"
            ),
        }
        artifacts["report.json"] = canonical_json(report)
        publish_directory(arguments.output, artifacts)
        return
    if arguments.cache is None:
        raise BatchDiscoveryError("audit stage requires --cache")
    acquisition_bytes = (arguments.cache / "report.json").read_bytes()
    try:
        acquisition = json.loads(acquisition_bytes)
    except json.JSONDecodeError as error:
        raise BatchDiscoveryError(f"parse batch acquisition: {error}") from error
    audited = audit_cache(
        arguments.cache,
        acquisition,
        manifest_bytes,
        runner_sha256,
    )
    report = {
        "schema": REPORT_SCHEMAS["audit"],
        "study_id": STUDY_ID,
        "revision": REVISION,
        "decision": "RealImpactMetalBatchDiscoveryCacheVerified",
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "preflight_sha256": sha256_bytes(preflight_bytes),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "partition": manifest["partition"],
        "objects": audited,
        "network_requests": 0,
        "new_object_member_payload_bytes_read": 0,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_domain_or_runtime_admission": False,
        "next_action": (
            "freeze exact member ranges, candidate parameters and gates before "
            "new payload access"
        ),
    }
    publish_directory(arguments.output, {"report.json": canonical_json(report)})


if __name__ == "__main__":
    main()
