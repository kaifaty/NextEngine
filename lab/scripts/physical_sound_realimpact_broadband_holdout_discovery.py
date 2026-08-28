#!/usr/bin/env python3
"""Discover the frozen REALIMPACT broad-band holdout without payload access."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

import physical_sound_realimpact_observation_discovery as discovery

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-broadband-holdout-discovery.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-broadband-holdout-discovery-"
        "preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-broadband-holdout-discovery-"
        "acquisition.report.v1"
    ),
    "audit": (
        "nextengine.experimental-realimpact-broadband-holdout-discovery-audit.report.v1"
    ),
}
STUDY_ID = "physical-sound-realimpact-broadband-independent-holdout"
REVISION = "iron-mortar-object-disjoint-archive-discovery-v1"
OBJECT_ID = "43_IronMortar"
ARCHIVE_URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/43_IronMortar.zip"
ARCHIVE_BYTES = 2_305_935_628
ARCHIVE_ETAG = '"6433e2e6-8971c90c"'
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 10:20:22 GMT"
TAIL_BYTES = 65_536
TAIL_START = ARCHIVE_BYTES - TAIL_BYTES
LOCAL_HEADER_BYTES = 30
EXPECTED_ENTRY_COUNT = 12
EXPECTED_AUDIO_ENTRY = f"{OBJECT_ID}/preprocessed/deconvolved_0db.npy"
ROSTER_SHA256 = "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
CORE_PYTHON_SHA256 = "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
V2_REPORT_SHA256 = "f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73"
V2_RESULT_DOC_SHA256 = (
    "c531d97899d2602f13e7d96b8b6fad639bd9911c421c2b8abeb10d05d2673897"
)


class HoldoutDiscoveryError(RuntimeError):
    """The independent holdout discovery contract or lineage changed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
    parser.add_argument("--output", required=True, type=Path)
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


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "core": {
            "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
            "sha256": CORE_PYTHON_SHA256,
        },
        "parent": {
            "report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-broadband-v2-"
                    "counterfactual-v1/analysis-a/report.json"
                ),
                "sha256": V2_REPORT_SHA256,
                "required_decision": ("ExistingIronBroadbandV2MethodTransferSupported"),
            },
            "result_document": {
                "path": (
                    "docs/development/physical-sound-realimpact-broadband-"
                    "v2-result-ps2-2026-08-28.md"
                ),
                "sha256": V2_RESULT_DOC_SHA256,
            },
        },
        "selection": {
            "dataset_object_id": OBJECT_ID,
            "role": "independent_method_holdout",
            "official_roster": {
                "repository_commit": ("fca2bd6cbb7e9f96ac61328d2a0d51594bf01987"),
                "object_names_sha256": ROSTER_SHA256,
                "zero_based_index": 16,
            },
            "rule": (
                "first official-roster entry after the Iron Skillet parent "
                "whose exact name contains Iron or Metal, excludes Pan and "
                "Skillet families, and has zero exact repository and external-"
                "experiment references at selection time"
            ),
            "candidate_audit": [
                {
                    "dataset_object_id": "17_IronSkillet",
                    "decision": "exclude_parent_and_skillet_family",
                },
                {
                    "dataset_object_id": "43_IronMortar",
                    "decision": "select_first_eligible",
                },
                {
                    "dataset_object_id": "67_IronPlate",
                    "decision": "not_reached_and_previously_opened",
                },
                {
                    "dataset_object_id": "90_MetalLadle",
                    "decision": "not_reached_and_previously_opened",
                },
            ],
            "exact_reference_matches_before_freeze": {
                "repository": 0,
                "external_experiment_store": 0,
            },
            "selection_commit": "8397f7927851a07daed352393adcfbd4137b2c15",
            "prior_holdout_payload_bytes_opened": 0,
            "object_disjoint_from_parent": True,
            "name_family_disjoint_from_parent": True,
            "material_or_domain_credit": False,
            "planter_remains_reserved": True,
        },
        "archive": {
            "url": ARCHIVE_URL,
            "bytes": ARCHIVE_BYTES,
            "etag": ARCHIVE_ETAG,
            "last_modified_http": ARCHIVE_LAST_MODIFIED,
            "accept_ranges": "bytes",
            "metadata_requests_before_freeze": 1,
            "payload_bytes_read_before_freeze": 0,
        },
        "requests": {
            "maximum_future_network_requests": 2,
            "tail_range": [TAIL_START, ARCHIVE_BYTES - 1],
            "tail_bytes": TAIL_BYTES,
            "audio_local_header_bytes": LOCAL_HEADER_BYTES,
            "audio_payload_bytes_allowed": 0,
        },
        "data_policy": {
            "central_directory_and_audio_local_header_only": True,
            "metadata_or_geometry_payload_access_allowed": False,
            "audio_payload_access_allowed": False,
            "planter_payload_access_allowed": False,
            "physics_comparison_allowed": False,
            "threshold_tuning_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish exact archive and observation-entry discovery, audit the "
            "immutable cache twice, then freeze a separate one-impact V2 "
            "holdout manifest before any member payload access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 64 * 1024:
        raise HoldoutDiscoveryError("holdout discovery manifest exceeds 64 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise HoldoutDiscoveryError(
            f"parse holdout discovery manifest: {error}"
        ) from error
    if manifest != expected_manifest(runner_sha256):
        raise HoldoutDiscoveryError("holdout discovery manifest changed")
    return data, manifest


def configure_core() -> None:
    discovery.OBJECT_ID = OBJECT_ID
    discovery.ARCHIVE_URL = ARCHIVE_URL
    discovery.ARCHIVE_BYTES = ARCHIVE_BYTES
    discovery.ARCHIVE_ETAG = ARCHIVE_ETAG
    discovery.ARCHIVE_LAST_MODIFIED = ARCHIVE_LAST_MODIFIED
    discovery.TAIL_BYTES = TAIL_BYTES
    discovery.TAIL_START = TAIL_START
    discovery.LOCAL_HEADER_BYTES = LOCAL_HEADER_BYTES
    discovery.EXPECTED_ENTRY_COUNT = EXPECTED_ENTRY_COUNT
    discovery.EXPECTED_AUDIO_ENTRY = EXPECTED_AUDIO_ENTRY
    discovery.ROSTER_SHA256 = ROSTER_SHA256


def validate_core(root: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    reference = manifest["core"]
    path = (root / reference["path"]).resolve(strict=True)
    if not path.is_relative_to(root) or sha256_file(path) != reference["sha256"]:
        raise HoldoutDiscoveryError("bound discovery core changed")
    return {
        "path": reference["path"],
        "sha256": reference["sha256"],
        "bytes": path.stat().st_size,
    }


def validate_parent(root: Path, base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    report_reference = manifest["parent"]["report"]
    report_path = (base / report_reference["path"]).resolve(strict=True)
    if not report_path.is_file() or not report_path.is_relative_to(base.parent):
        raise HoldoutDiscoveryError("V2 report escapes physical-sound store")
    report_bytes = report_path.read_bytes()
    report = json.loads(report_bytes)
    if (
        sha256_bytes(report_bytes) != report_reference["sha256"]
        or report.get("decision") != report_reference["required_decision"]
        or report.get("gate", {}).get("passed") is not True
        or report.get("additional_payload_bytes_read") != 0
        or report.get("network_requests") != 0
        or report.get("physics_solver_runs") != 0
        or report.get("planter_payload_bytes_read") != 0
        or report.get("quality_admission_or_runtime_credit") is not False
    ):
        raise HoldoutDiscoveryError("Iron V2 support parent changed")

    doc_reference = manifest["parent"]["result_document"]
    doc_path = (root / doc_reference["path"]).resolve(strict=True)
    if (
        not doc_path.is_file()
        or not doc_path.is_relative_to(root)
        or sha256_file(doc_path) != doc_reference["sha256"]
    ):
        raise HoldoutDiscoveryError("Iron V2 result document changed")
    return {
        "report_path": report_reference["path"],
        "report_sha256": report_reference["sha256"],
        "decision": report["decision"],
        "result_document_path": doc_reference["path"],
        "result_document_sha256": doc_reference["sha256"],
    }


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "role": "independent_method_holdout",
        "archive": {
            "url": ARCHIVE_URL,
            "bytes": ARCHIVE_BYTES,
            "etag": ARCHIVE_ETAG,
            "last_modified_http": ARCHIVE_LAST_MODIFIED,
        },
        "prior_metadata_requests": 1,
        "prior_holdout_payload_bytes_opened": 0,
        "audio_payload_bytes_read": 0,
        "planter_audio_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_admission_or_runtime_credit": False,
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> tuple[dict[str, Any], dict[str, bytes]]:
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "BroadbandIndependentHoldoutDiscoveryPreflightSupported",
        "claim": (
            "METADATA_ONLY_OBJECT_DISJOINT_HOLDOUT_PREREGISTRATION / NO_"
            "OBJECT_PAYLOAD_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": validate_parent(root, base, manifest),
        "bound_core": validate_core(root, manifest),
        "selection": manifest["selection"],
        "frozen_requests": manifest["requests"],
        "network_requests": 0,
        "next_action": (
            "acquire only the exact archive tail and 30-byte observation local "
            "header once; open no member payload"
        ),
    }
    return report, {}


def acquire(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> tuple[dict[str, Any], dict[str, bytes]]:
    parent = validate_parent(root, base, manifest)
    validate_core(root, manifest)
    tail, tail_response = discovery.range_get(TAIL_START, TAIL_BYTES)
    central, entries = discovery.discover_from_tail(tail)
    audio = next(entry for entry in entries if entry["name"] == EXPECTED_AUDIO_ENTRY)
    header, header_response = discovery.range_get(
        audio["local_offset"], LOCAL_HEADER_BYTES
    )
    audio_header = discovery.validate_audio_local_header(header, audio)
    report = {
        "schema": REPORT_SCHEMAS["acquire"],
        "status": "Validated",
        "decision": "BroadbandIndependentHoldoutArchiveEntryDiscovered",
        "claim": (
            "CENTRAL_DIRECTORY_AND_AUDIO_LOCAL_HEADER_ONLY / ZERO_MEMBER_"
            "PAYLOAD_BYTES / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": parent,
        "tail_response": tail_response,
        "central_directory": central,
        "entries": entries,
        "observation_entry": audio,
        "observation_local_header": audio_header,
        "audio_local_header_response": header_response,
        "network_requests": 2,
        "next_action": (
            "audit this immutable cache twice, then freeze one-impact V2 holdout access"
        ),
    }
    return report, {"tail.bin": tail, "audio-local-header.bin": header}


def audit_cache(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    cache: Path,
) -> tuple[dict[str, Any], dict[str, bytes]]:
    parent = validate_parent(root, base, manifest)
    validate_core(root, manifest)
    acquisition_bytes = (cache / "report.json").read_bytes()
    acquisition = json.loads(acquisition_bytes)
    tail = (cache / "tail.bin").read_bytes()
    header = (cache / "audio-local-header.bin").read_bytes()
    if (
        acquisition.get("schema") != REPORT_SCHEMAS["acquire"]
        or acquisition.get("decision")
        != "BroadbandIndependentHoldoutArchiveEntryDiscovered"
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("network_requests") != 2
        or acquisition.get("audio_payload_bytes_read") != 0
        or acquisition.get("planter_audio_payload_bytes_read") != 0
        or acquisition.get("tail_response", {}).get("sha256") != sha256_bytes(tail)
        or acquisition.get("observation_local_header", {}).get("header_sha256")
        != sha256_bytes(header)
    ):
        raise HoldoutDiscoveryError("cached holdout acquisition changed")
    central, entries = discovery.discover_from_tail(tail)
    audio = next(entry for entry in entries if entry["name"] == EXPECTED_AUDIO_ENTRY)
    audio_header = discovery.validate_audio_local_header(header, audio)
    if (
        central != acquisition.get("central_directory")
        or entries != acquisition.get("entries")
        or audio != acquisition.get("observation_entry")
        or audio_header != acquisition.get("observation_local_header")
    ):
        raise HoldoutDiscoveryError("cached holdout discovery values changed")
    report = {
        "schema": REPORT_SCHEMAS["audit"],
        "status": "Validated",
        "decision": "BroadbandIndependentHoldoutDiscoveryCacheVerified",
        "claim": (
            "OFFLINE_CACHE_AUDIT_ONLY / ZERO_NEW_NETWORK_OR_MEMBER_PAYLOAD_"
            "ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": parent,
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "tail_sha256": sha256_bytes(tail),
        "audio_local_header_sha256": sha256_bytes(header),
        "central_directory": central,
        "observation_entry": audio,
        "observation_local_header": audio_header,
        "network_requests": 0,
        "next_action": "freeze one-impact V2 holdout access before payload",
    }
    return report, {}


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = discovery.external_file(root, arguments.manifest, "manifest")
    output = discovery.external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    configure_core()
    if arguments.stage == "preflight":
        if arguments.cache is not None:
            raise HoldoutDiscoveryError("preflight does not accept --cache")
        report, artifacts = preflight(
            root, base, manifest_bytes, manifest, runner_sha256
        )
    elif arguments.stage == "acquire":
        if arguments.cache is not None:
            raise HoldoutDiscoveryError("acquire does not accept --cache")
        report, artifacts = acquire(root, base, manifest_bytes, manifest, runner_sha256)
    else:
        if arguments.cache is None:
            raise HoldoutDiscoveryError("audit requires --cache")
        cache = discovery.external_directory(root, arguments.cache, "cache")
        report, artifacts = audit_cache(
            root, base, manifest_bytes, manifest, runner_sha256, cache
        )
    report_bytes = discovery.publish(output, report, artifacts)
    print(f"REALIMPACT broad-band holdout discovery {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report['network_requests']}")
    print("holdout audio payload bytes read: 0")
    print("Planter audio payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
