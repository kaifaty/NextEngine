#!/usr/bin/env python3
"""Discover the preregistered independent REALIMPACT object without payload access."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

import physical_sound_realimpact_observation_discovery as discovery


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-independent-observation-discovery."
    "manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-independent-observation-discovery-"
        "preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-independent-observation-discovery-"
        "acquisition.report.v1"
    ),
    "audit": (
        "nextengine.experimental-realimpact-independent-observation-discovery-"
        "audit.report.v1"
    ),
}
STUDY_ID = "physical-sound-realimpact-independent-observation-discriminator"
REVISION = "iron-skillet-source-derived-salience-discovery-v1"
OBJECT_ID = "17_IronSkillet"
ARCHIVE_URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/17_IronSkillet.zip"
ARCHIVE_BYTES = 2_393_994_112
ARCHIVE_ETAG = '"6433da59-8eb17380"'
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 09:43:53 GMT"
TAIL_BYTES = 65_536
TAIL_START = ARCHIVE_BYTES - TAIL_BYTES
LOCAL_HEADER_BYTES = 30
EXPECTED_ENTRY_COUNT = 12
EXPECTED_AUDIO_ENTRY = f"{OBJECT_ID}/preprocessed/deconvolved_0db.npy"
ROSTER_SHA256 = "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
CORE_PYTHON_SHA256 = (
    "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
)
SALIENCE_REPORT_SHA256 = (
    "fa94071059c3e3f6dd3e71eed187affe08c0ca36c120cd483b8c2e8da9560b3e"
)


class IndependentDiscoveryError(RuntimeError):
    """The independent-object discovery contract or lineage changed."""


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
            "path": "../ps2-salience-selector-control-v1/run-a/report.json",
            "sha256": SALIENCE_REPORT_SHA256,
            "required_decision": "SalienceSelectorSyntheticControlSupported",
        },
        "selection": {
            "dataset_object_id": OBJECT_ID,
            "role": "development",
            "official_roster": {
                "repository_commit": "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987",
                "object_names_sha256": ROSTER_SHA256,
                "zero_based_index": 2,
                "preceding_entries": ["100_Frisbee", "10_bowl"],
            },
            "rule": (
                "first official-roster entry not previously opened or referenced by the "
                "repository/external experiment inventory at selection time, excluding "
                "reserved objects and 63_SmallPlanterCeramic"
            ),
            "exclusions_before_selection": [
                {
                    "dataset_object_id": "100_Frisbee",
                    "reason": "already referenced by prior calibration planning",
                },
                {
                    "dataset_object_id": "10_bowl",
                    "reason": "already opened by prior development analysis",
                },
            ],
            "exact_reference_matches_before_freeze": {
                "repository": 0,
                "external_experiment_store": 0,
            },
            "selection_commit": "9315a616556dc6f92848f2532ed55f2d7b29561a",
            "prior_object_payload_bytes_opened": 0,
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
            "publish exact archive/entry discovery, audit the immutable cache twice, "
            "then freeze a separate one-impact observation manifest using the unchanged "
            "salience selector before any member payload access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 64 * 1024:
        raise IndependentDiscoveryError("independent discovery manifest exceeds 64 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise IndependentDiscoveryError(f"parse discovery manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise IndependentDiscoveryError("independent discovery manifest changed")
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
    ref = manifest["core"]
    path = (root / ref["path"]).resolve(strict=True)
    if not path.is_relative_to(root) or sha256_file(path) != ref["sha256"]:
        raise IndependentDiscoveryError("bound discovery core changed")
    return {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}


def validate_parent(base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    ref = manifest["parent"]
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise IndependentDiscoveryError("salience parent escapes physical-sound store")
    data = path.read_bytes()
    report = json.loads(data)
    if (
        sha256_bytes(data) != ref["sha256"]
        or report.get("decision") != ref["required_decision"]
        or report.get("gate", {}).get("passed") is not True
        or report.get("real_payload_bytes_read") != 0
        or report.get("physics_solver_runs") != 0
        or report.get("planter_payload_bytes_read") != 0
    ):
        raise IndependentDiscoveryError("salience parent lineage changed")
    return {
        "path": ref["path"],
        "sha256": ref["sha256"],
        "decision": report["decision"],
    }


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "role": "development",
        "archive": {
            "url": ARCHIVE_URL,
            "bytes": ARCHIVE_BYTES,
            "etag": ARCHIVE_ETAG,
            "last_modified_http": ARCHIVE_LAST_MODIFIED,
        },
        "prior_metadata_requests": 1,
        "prior_object_payload_bytes_opened": 0,
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
        "decision": "IndependentObservationDiscoveryPreflightSupported",
        "claim": (
            "METADATA_ONLY_INDEPENDENT_OBJECT_PREREGISTRATION / "
            "NO_OBJECT_PAYLOAD_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": validate_parent(base, manifest),
        "bound_core": validate_core(root, manifest),
        "selection": manifest["selection"],
        "frozen_requests": manifest["requests"],
        "network_requests": 0,
        "next_action": (
            "acquire only the exact archive tail and 30-byte observation local header "
            "once; open no member payload"
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
    parent = validate_parent(base, manifest)
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
        "decision": "IndependentObjectArchiveAndObservationEntryDiscovered",
        "claim": (
            "CENTRAL_DIRECTORY_AND_AUDIO_LOCAL_HEADER_ONLY / ZERO_MEMBER_PAYLOAD_BYTES / "
            "NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
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
            "audit this immutable cache twice, then freeze one-impact observation access"
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
    parent = validate_parent(base, manifest)
    validate_core(root, manifest)
    acquisition_bytes = (cache / "report.json").read_bytes()
    acquisition = json.loads(acquisition_bytes)
    tail = (cache / "tail.bin").read_bytes()
    header = (cache / "audio-local-header.bin").read_bytes()
    if (
        acquisition.get("schema") != REPORT_SCHEMAS["acquire"]
        or acquisition.get("decision")
        != "IndependentObjectArchiveAndObservationEntryDiscovered"
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("network_requests") != 2
        or acquisition.get("audio_payload_bytes_read") != 0
        or acquisition.get("planter_audio_payload_bytes_read") != 0
        or acquisition.get("tail_response", {}).get("sha256") != sha256_bytes(tail)
        or acquisition.get("observation_local_header", {}).get("header_sha256")
        != sha256_bytes(header)
    ):
        raise IndependentDiscoveryError("cached independent acquisition changed")
    central, entries = discovery.discover_from_tail(tail)
    audio = next(entry for entry in entries if entry["name"] == EXPECTED_AUDIO_ENTRY)
    audio_header = discovery.validate_audio_local_header(header, audio)
    if (
        central != acquisition.get("central_directory")
        or entries != acquisition.get("entries")
        or audio != acquisition.get("observation_entry")
        or audio_header != acquisition.get("observation_local_header")
    ):
        raise IndependentDiscoveryError("cached independent discovery values changed")
    report = {
        "schema": REPORT_SCHEMAS["audit"],
        "status": "Validated",
        "decision": "IndependentObservationDiscoveryCacheVerified",
        "claim": (
            "OFFLINE_CACHE_AUDIT_ONLY / ZERO_NEW_NETWORK_OR_MEMBER_PAYLOAD_ACCESS / "
            "NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
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
        "next_action": (
            "freeze one-impact observation access using the unchanged salience selector"
        ),
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
            raise IndependentDiscoveryError("preflight does not accept --cache")
        report, artifacts = preflight(
            root, base, manifest_bytes, manifest, runner_sha256
        )
    elif arguments.stage == "acquire":
        if arguments.cache is not None:
            raise IndependentDiscoveryError("acquire does not accept --cache")
        report, artifacts = acquire(
            root, base, manifest_bytes, manifest, runner_sha256
        )
    else:
        if arguments.cache is None:
            raise IndependentDiscoveryError("audit requires --cache")
        cache = discovery.external_directory(root, arguments.cache, "cache")
        report, artifacts = audit_cache(
            root, base, manifest_bytes, manifest, runner_sha256, cache
        )
    report_bytes = discovery.publish(output, report, artifacts)
    print(f"Independent REALIMPACT observation discovery {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report['network_requests']}")
    print("object payload bytes read: 0")
    print("Planter audio payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
