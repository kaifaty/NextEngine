#!/usr/bin/env python3
"""Discover one unopened REALIMPACT object without reading audio payload bytes."""

from __future__ import annotations

import argparse
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import shutil
import socket
import struct
from typing import Any
import urllib.error
import urllib.parse
import urllib.request


MANIFEST_SCHEMA = "nextengine.experimental-realimpact-observation-discovery.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-observation-discovery-preflight.report.v1",
    "acquire": "nextengine.experimental-realimpact-observation-discovery-acquisition.report.v1",
    "audit": "nextengine.experimental-realimpact-observation-discovery-audit.report.v1",
}
STUDY_ID = "physical-sound-realimpact-observation-first-discriminator"
OBJECT_ID = "78_CeramicCup"
ARCHIVE_URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/78_CeramicCup.zip"
ARCHIVE_BYTES = 2_320_964_262
ARCHIVE_ETAG = '"6433ded5-8a571aa6"'
ARCHIVE_LAST_MODIFIED = "Mon, 10 Apr 2023 10:03:01 GMT"
TAIL_BYTES = 65_536
TAIL_START = ARCHIVE_BYTES - TAIL_BYTES
LOCAL_HEADER_BYTES = 30
EXPECTED_ENTRY_COUNT = 12
EXPECTED_AUDIO_ENTRY = f"{OBJECT_ID}/preprocessed/deconvolved_0db.npy"
ROSTER_SHA256 = "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"


class DiscoveryError(RuntimeError):
    """The frozen discovery contract or a bounded source invariant failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--cache", type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise DiscoveryError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise DiscoveryError(f"{label} must be an external directory: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise DiscoveryError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise DiscoveryError(f"output must be absent or empty: {resolved}")
    return resolved


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 64 * 1024:
        raise DiscoveryError("discovery manifest exceeds 64 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiscoveryError(f"parse discovery manifest: {error}") from error
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("study_id") != STUDY_ID
        or manifest.get("protocol_revision") != "observation-first-metadata-discovery-v1"
        or manifest.get("runner_sha256") != runner_sha256
        or manifest.get("object")
        != {
            "dataset_object_id": OBJECT_ID,
            "role": "development",
            "prior_payload_bytes_opened": 0,
        }
        or manifest.get("official_roster")
        != {
            "repository_commit": "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987",
            "object_names_sha256": ROSTER_SHA256,
            "contains_dataset_object_id": True,
        }
        or manifest.get("archive")
        != {
            "url": ARCHIVE_URL,
            "bytes": ARCHIVE_BYTES,
            "etag": ARCHIVE_ETAG,
            "last_modified_http": ARCHIVE_LAST_MODIFIED,
        }
        or manifest.get("requests")
        != {
            "maximum_network_requests": 2,
            "tail_range": [TAIL_START, ARCHIVE_BYTES - 1],
            "tail_bytes": TAIL_BYTES,
            "audio_local_header_bytes": LOCAL_HEADER_BYTES,
            "audio_payload_bytes_allowed": 0,
        }
        or manifest.get("data_policy")
        != {
            "central_directory_and_audio_local_header_only": True,
            "metadata_or_geometry_payload_access_allowed": False,
            "audio_payload_access_allowed": False,
            "planter_payload_access_allowed": False,
            "physics_comparison_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        }
        or manifest.get("stop_rule")
        != "publish exact archive/entry discovery, then freeze a separate metadata and observation manifest before any payload access"
    ):
        raise DiscoveryError("discovery manifest contract changed")
    return data, manifest


def ensure_public_archive_host() -> None:
    parsed = urllib.parse.urlparse(ARCHIVE_URL)
    if parsed.scheme != "https" or parsed.hostname is None or parsed.username is not None:
        raise DiscoveryError("archive URL is not a credential-free HTTPS URL")
    addresses = {
        item[4][0]
        for item in socket.getaddrinfo(parsed.hostname, 443, type=socket.SOCK_STREAM)
    }
    if not addresses or any(not ipaddress.ip_address(value).is_global for value in addresses):
        raise DiscoveryError(f"archive host did not resolve only to public addresses: {addresses}")


def range_get(start: int, length: int) -> tuple[bytes, dict[str, str | int]]:
    if start < 0 or length <= 0 or start + length > ARCHIVE_BYTES:
        raise DiscoveryError("bounded archive range is invalid")
    ensure_public_archive_host()
    end = start + length - 1
    request = urllib.request.Request(
        ARCHIVE_URL,
        headers={"Range": f"bytes={start}-{end}", "Accept-Encoding": "identity"},
        method="GET",
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            data = response.read(length + 1)
            status = response.status
            final_url = response.geturl()
            content_range = response.headers.get("Content-Range")
            content_length = response.headers.get("Content-Length")
            etag = response.headers.get("ETag")
            last_modified = response.headers.get("Last-Modified")
    except urllib.error.HTTPError as error:
        raise DiscoveryError(f"archive range request failed: HTTP {error.code}") from error
    expected_range = f"bytes {start}-{end}/{ARCHIVE_BYTES}"
    if (
        status != 206
        or final_url != ARCHIVE_URL
        or content_range != expected_range
        or content_length != str(length)
        or etag != ARCHIVE_ETAG
        or last_modified != ARCHIVE_LAST_MODIFIED
        or len(data) != length
    ):
        raise DiscoveryError(
            "archive range response identity changed: "
            f"{status}, {final_url}, {content_range}, {content_length}, {etag}, {last_modified}, {len(data)}"
        )
    return data, {
        "start": start,
        "end": end,
        "bytes": length,
        "content_range": content_range,
        "etag": etag,
        "last_modified_http": last_modified,
        "sha256": sha256_bytes(data),
    }


def parse_eocd(tail: bytes) -> tuple[int, int, int]:
    marker = b"PK\x05\x06"
    offset = tail.rfind(marker)
    if offset < 0 or offset + 22 > len(tail):
        raise DiscoveryError("ZIP end-of-central-directory record is absent")
    (
        signature,
        disk,
        central_disk,
        disk_entries,
        total_entries,
        central_bytes,
        central_offset,
        comment_bytes,
    ) = struct.unpack_from("<4s4H2LH", tail, offset)
    if (
        signature != marker
        or disk != 0
        or central_disk != 0
        or disk_entries != EXPECTED_ENTRY_COUNT
        or total_entries != EXPECTED_ENTRY_COUNT
        or comment_bytes != len(tail) - offset - 22
        or central_offset + central_bytes > ARCHIVE_BYTES
    ):
        raise DiscoveryError("ZIP end-of-central-directory contract changed")
    return central_offset, central_bytes, total_entries


def parse_central_directory(data: bytes, count: int) -> list[dict[str, Any]]:
    entries = []
    offset = 0
    for _ in range(count):
        if offset + 46 > len(data):
            raise DiscoveryError("ZIP central directory is truncated")
        values = struct.unpack_from("<4s6H3L5H2L", data, offset)
        if values[0] != b"PK\x01\x02":
            raise DiscoveryError("ZIP central-directory signature changed")
        flags, method = values[3], values[4]
        crc32, compressed, uncompressed = values[7], values[8], values[9]
        name_bytes, extra_bytes, comment_bytes = values[10], values[11], values[12]
        disk_start, local_offset = values[13], values[16]
        record_end = offset + 46 + name_bytes + extra_bytes + comment_bytes
        if record_end > len(data) or disk_start != 0:
            raise DiscoveryError("ZIP central-directory entry is invalid")
        raw_name = data[offset + 46 : offset + 46 + name_bytes]
        try:
            name = raw_name.decode("utf-8")
        except UnicodeDecodeError as error:
            raise DiscoveryError("ZIP entry name is not UTF-8") from error
        entries.append(
            {
                "name": name,
                "flags": flags,
                "method": method,
                "crc32": f"{crc32:08x}",
                "compressed_bytes": compressed,
                "uncompressed_bytes": uncompressed,
                "local_offset": local_offset,
            }
        )
        offset = record_end
    if offset != len(data):
        raise DiscoveryError("ZIP central-directory byte count changed")
    return entries


def discover_from_tail(tail: bytes) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    central_offset, central_bytes, entry_count = parse_eocd(tail)
    relative = central_offset - TAIL_START
    if relative < 0 or relative + central_bytes > len(tail):
        raise DiscoveryError("bounded tail does not contain the complete central directory")
    central = tail[relative : relative + central_bytes]
    entries = parse_central_directory(central, entry_count)
    matches = [entry for entry in entries if entry["name"] == EXPECTED_AUDIO_ENTRY]
    if len(matches) != 1 or matches[0]["method"] != 8:
        raise DiscoveryError("expected deflated observation entry is absent")
    return {
        "offset": central_offset,
        "bytes": central_bytes,
        "sha256": sha256_bytes(central),
        "entry_count": entry_count,
    }, entries


def validate_audio_local_header(header: bytes, audio: dict[str, Any]) -> dict[str, Any]:
    if len(header) != LOCAL_HEADER_BYTES:
        raise DiscoveryError("audio local header byte count changed")
    values = struct.unpack("<4s5H3L2H", header)
    if values[0] != b"PK\x03\x04":
        raise DiscoveryError("audio local-header signature changed")
    flags, method, name_bytes, extra_bytes = values[2], values[3], values[9], values[10]
    expected_name_bytes = len(EXPECTED_AUDIO_ENTRY.encode("utf-8"))
    if (
        flags != audio["flags"]
        or method != audio["method"]
        or name_bytes != expected_name_bytes
    ):
        raise DiscoveryError("audio local-header fields differ from the central directory")
    data_offset = audio["local_offset"] + LOCAL_HEADER_BYTES + name_bytes + extra_bytes
    if data_offset <= audio["local_offset"] or data_offset >= ARCHIVE_BYTES:
        raise DiscoveryError("audio payload offset is invalid")
    return {
        "local_offset": audio["local_offset"],
        "header_bytes_read": LOCAL_HEADER_BYTES,
        "header_sha256": sha256_bytes(header),
        "name_bytes": name_bytes,
        "extra_bytes": extra_bytes,
        "data_offset": data_offset,
    }


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "protocol_revision": "observation-first-metadata-discovery-v1",
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
        "prior_object_payload_bytes_opened": 0,
        "audio_payload_bytes_read": 0,
        "planter_audio_payload_bytes_read": 0,
    }


def preflight(manifest_bytes: bytes, runner_sha256: str) -> tuple[dict[str, Any], dict[str, bytes]]:
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "CeramicCupObservationDiscoveryPreflightSupported",
        "claim": "LOCAL_HASH_AND_RANGE_PREFLIGHT_ONLY / NO_NETWORK_OR_PAYLOAD_ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "frozen_requests": {
            "tail_range": [TAIL_START, ARCHIVE_BYTES - 1],
            "tail_bytes": TAIL_BYTES,
            "audio_local_header_bytes": LOCAL_HEADER_BYTES,
            "maximum_network_requests": 2,
        },
        "network_requests": 0,
        "next_action": "acquire the exact tail and 30-byte audio local header once; open no payload member",
    }
    return report, {}


def acquire(manifest_bytes: bytes, runner_sha256: str) -> tuple[dict[str, Any], dict[str, bytes]]:
    tail, tail_response = range_get(TAIL_START, TAIL_BYTES)
    central, entries = discover_from_tail(tail)
    audio = next(entry for entry in entries if entry["name"] == EXPECTED_AUDIO_ENTRY)
    header, header_response = range_get(audio["local_offset"], LOCAL_HEADER_BYTES)
    audio_header = validate_audio_local_header(header, audio)
    report = {
        "schema": REPORT_SCHEMAS["acquire"],
        "status": "Validated",
        "decision": "CeramicCupArchiveAndObservationEntryDiscovered",
        "claim": "CENTRAL_DIRECTORY_AND_AUDIO_LOCAL_HEADER_ONLY / ZERO_AUDIO_PAYLOAD_BYTES / NO_METADATA_GEOMETRY_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "tail_response": tail_response,
        "central_directory": central,
        "entries": entries,
        "observation_entry": audio,
        "observation_local_header": audio_header,
        "audio_local_header_response": header_response,
        "network_requests": 2,
        "next_action": "repeat from this immutable cache, then freeze exact metadata/geometry and one-impact observation access before any member payload is opened",
    }
    return report, {"tail.bin": tail, "audio-local-header.bin": header}


def audit_cache(
    manifest_bytes: bytes, runner_sha256: str, cache: Path
) -> tuple[dict[str, Any], dict[str, bytes]]:
    acquisition_bytes = (cache / "report.json").read_bytes()
    acquisition = json.loads(acquisition_bytes)
    tail = (cache / "tail.bin").read_bytes()
    header = (cache / "audio-local-header.bin").read_bytes()
    if (
        acquisition.get("schema") != REPORT_SCHEMAS["acquire"]
        or acquisition.get("decision") != "CeramicCupArchiveAndObservationEntryDiscovered"
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("network_requests") != 2
        or acquisition.get("audio_payload_bytes_read") != 0
        or acquisition.get("planter_audio_payload_bytes_read") != 0
        or acquisition.get("tail_response", {}).get("sha256") != sha256_bytes(tail)
        or acquisition.get("observation_local_header", {}).get("header_sha256")
        != sha256_bytes(header)
    ):
        raise DiscoveryError("cached acquisition lineage changed")
    central, entries = discover_from_tail(tail)
    audio = next(entry for entry in entries if entry["name"] == EXPECTED_AUDIO_ENTRY)
    audio_header = validate_audio_local_header(header, audio)
    if (
        central != acquisition.get("central_directory")
        or entries != acquisition.get("entries")
        or audio != acquisition.get("observation_entry")
        or audio_header != acquisition.get("observation_local_header")
    ):
        raise DiscoveryError("cached discovery values changed")
    report = {
        "schema": REPORT_SCHEMAS["audit"],
        "status": "Validated",
        "decision": "CeramicCupObservationDiscoveryCacheVerified",
        "claim": "OFFLINE_CACHE_AUDIT_ONLY / ZERO_NEW_NETWORK_OR_PAYLOAD_ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "tail_sha256": sha256_bytes(tail),
        "audio_local_header_sha256": sha256_bytes(header),
        "central_directory": central,
        "observation_entry": audio,
        "observation_local_header": audio_header,
        "network_requests": 0,
        "next_action": "freeze exact metadata/geometry and one-impact observation access before any member payload is opened",
    }
    return report, {}


def publish(output: Path, report: dict[str, Any], artifacts: dict[str, bytes]) -> bytes:
    report_bytes = canonical_json(report)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        for name, data in artifacts.items():
            (staging / name).write_bytes(data)
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report_bytes


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = external_file(root, arguments.manifest, "manifest")
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_bytes(Path(__file__).read_bytes())
    manifest_bytes, _ = load_manifest(manifest_path, runner_sha256)
    if arguments.stage == "preflight":
        if arguments.cache is not None:
            raise DiscoveryError("preflight does not accept --cache")
        report, artifacts = preflight(manifest_bytes, runner_sha256)
    elif arguments.stage == "acquire":
        if arguments.cache is not None:
            raise DiscoveryError("acquire does not accept --cache")
        report, artifacts = acquire(manifest_bytes, runner_sha256)
    else:
        if arguments.cache is None:
            raise DiscoveryError("audit requires --cache")
        cache = external_directory(root, arguments.cache, "cache")
        report, artifacts = audit_cache(manifest_bytes, runner_sha256, cache)
    report_bytes = publish(output, report, artifacts)
    print(f"REALIMPACT observation discovery {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report['network_requests']}")
    print("audio payload bytes read: 0")
    print("Planter audio payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
