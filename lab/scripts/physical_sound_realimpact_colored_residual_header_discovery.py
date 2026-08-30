#!/usr/bin/env python3
"""Discover colored-residual observation payload ranges from local headers."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import shutil
import tempfile
import zipfile
from pathlib import Path
from typing import Any

import physical_sound_realimpact_colored_residual_tail_discovery as tail_discovery
import physical_sound_realimpact_observation_discovery as discovery

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-colored-residual-header-discovery.manifest.v1"
)
REPORT_SCHEMAS = {
    "fixture": (
        "nextengine.experimental-realimpact-colored-residual-header-fixture.report.v1"
    ),
    "preflight": (
        "nextengine.experimental-realimpact-colored-residual-header-preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-colored-residual-header-acquisition.report.v1"
    ),
    "audit": (
        "nextengine.experimental-realimpact-colored-residual-header-audit.report.v1"
    ),
}
REVISION = "fresh-pan-piepan-cup-local-header-only-discovery-v1"
TAIL_RUNNER_SHA256 = "5987b04807fe21177fd017165ef299a87ca8ddac6fa14f818209b67644a8fb74"
DISCOVERY_CORE_SHA256 = (
    "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
)
TAIL_BYTES = 65_536
LOCAL_HEADER_BYTES = 30
EXPECTED_ENTRY_COUNT = 12
EXPECTED_OBJECTS = [
    ("19_Pan", "calibration", "pan", 3),
    ("37_PiePan", "calibration", "pan", 14),
    ("22_Cup", "holdout", "cup", 4),
]


class HeaderDiscoveryError(RuntimeError):
    """The frozen ZIP local-header discovery boundary failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage",
        required=True,
        choices=["fixture", "manifest", "preflight", "acquire", "audit"],
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--tail-audit", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--preflight", type=Path)
    parser.add_argument("--acquisition", type=Path)
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


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise HeaderDiscoveryError(f"{label} must be an external file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise HeaderDiscoveryError(f"{label} must be an external directory: {resolved}")
    return resolved


def external_output(root: Path, path: Path, *, directory: bool) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise HeaderDiscoveryError(f"output must remain external: {resolved}")
    if directory:
        if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
            raise HeaderDiscoveryError(
                f"output directory is not absent/empty: {resolved}"
            )
    elif resolved.exists():
        raise HeaderDiscoveryError(f"refusing to replace output: {resolved}")
    return resolved


def validate_sources(root: Path) -> list[dict[str, Any]]:
    references = [
        {
            "path": "lab/scripts/physical_sound_realimpact_colored_residual_tail_discovery.py",
            "sha256": TAIL_RUNNER_SHA256,
        },
        {
            "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
            "sha256": DISCOVERY_CORE_SHA256,
        },
    ]
    result = []
    for reference in references:
        path = (root / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != reference["sha256"]
        ):
            raise HeaderDiscoveryError(f"bound source changed: {reference['path']}")
        result.append({**reference, "bytes": path.stat().st_size})
    return result


def validate_tail_object(observed: dict[str, Any], expected: tuple[Any, ...]) -> None:
    object_id, role, family, roster_index = expected
    identity = observed.get("archive_identity", {})
    entry = observed.get("observation_entry", {})
    header_range = observed.get("next_local_header_range", [])
    expected_name = f"{object_id}/preprocessed/deconvolved_0db.npy"
    if (
        observed.get("dataset_object_id") != object_id
        or observed.get("role") != role
        or observed.get("semantic_family_group") != family
        or observed.get("roster_index") != roster_index
        or not identity.get("url")
        or identity.get("bytes", 0) < TAIL_BYTES
        or not identity.get("etag")
        or not identity.get("last_modified_http")
        or identity.get("accept_ranges") != "bytes"
        or observed.get("central_directory", {}).get("entry_count")
        != EXPECTED_ENTRY_COUNT
        or entry.get("name") != expected_name
        or entry.get("method") != 8
        or entry.get("compressed_bytes", 0) <= 0
        or entry.get("uncompressed_bytes", 0) <= 0
        or header_range
        != [entry.get("local_offset"), entry.get("local_offset", -1) + 29]
        or header_range[0] < 0
        or header_range[1] >= identity.get("bytes", 0)
    ):
        raise HeaderDiscoveryError(f"tail audit object changed: {observed}")


def read_tail_audit(
    root: Path, path: Path
) -> tuple[bytes, dict[str, Any], list[dict[str, Any]]]:
    resolved = external_file(root, path, "ZIP-tail audit")
    data = resolved.read_bytes()
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise HeaderDiscoveryError(f"parse ZIP-tail audit: {error}") from error
    if (
        report.get("schema")
        != "nextengine.experimental-realimpact-colored-residual-tail-audit.report.v1"
        or report.get("decision")
        != "RealImpactColoredResidualObservationLocalHeaderRangesDiscovered"
        or report.get("runner_sha256") != TAIL_RUNNER_SHA256
        or report.get("network_requests") != 0
        or report.get("tail_bytes_read_from_cache")
        != TAIL_BYTES * len(EXPECTED_OBJECTS)
        or report.get("local_header_bytes_read") != 0
        or report.get("member_payload_bytes_read") != 0
        or report.get("shadow_payload_bytes_read") != 0
        or report.get("quality_domain_or_runtime_admission") is not False
        or report.get("authored_clip_fallback_required") is not True
    ):
        raise HeaderDiscoveryError("ZIP-tail audit lineage changed")
    objects = report.get("objects", [])
    if len(objects) != len(EXPECTED_OBJECTS):
        raise HeaderDiscoveryError("ZIP-tail audit object count changed")
    for observed, expected in zip(objects, EXPECTED_OBJECTS, strict=True):
        validate_tail_object(observed, expected)
    return data, report, objects


def expected_manifest(
    runner_sha256: str,
    tail_audit_sha256: str,
    objects: list[dict[str, Any]],
) -> dict[str, Any]:
    selected = []
    for item in objects:
        selected.append(
            {
                "dataset_object_id": item["dataset_object_id"],
                "role": item["role"],
                "semantic_family_group": item["semantic_family_group"],
                "roster_index": item["roster_index"],
                "archive_identity": item["archive_identity"],
                "observation_entry": item["observation_entry"],
                "local_header_range": item.get(
                    "next_local_header_range", item.get("local_header_range")
                ),
            }
        )
    return {
        "schema": MANIFEST_SCHEMA,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_colored_residual_header_discovery.py",
            "sha256": runner_sha256,
        },
        "sources": [
            {
                "path": "lab/scripts/physical_sound_realimpact_colored_residual_tail_discovery.py",
                "sha256": TAIL_RUNNER_SHA256,
            },
            {
                "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
                "sha256": DISCOVERY_CORE_SHA256,
            },
        ],
        "lineage": {
            "tail_audit_sha256": tail_audit_sha256,
            "tail_decision": (
                "RealImpactColoredResidualObservationLocalHeaderRangesDiscovered"
            ),
        },
        "claim": (
            "THREE_EXACT_30_BYTE_LOCAL_HEADERS_ONLY / ZERO_MEMBER_OR_SHADOW_"
            "PAYLOAD / NO_QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_PRODUCTION_CREDIT"
        ),
        "objects": selected,
        "access": {
            "requests": 3,
            "requests_per_object": 1,
            "local_header_bytes_per_object": LOCAL_HEADER_BYTES,
            "total_range_bytes": LOCAL_HEADER_BYTES * len(selected),
            "member_payload_bytes": 0,
            "shadow_requests": 0,
            "retry_or_range_growth": False,
        },
        "next_stage_if_supported": (
            "freeze one exact compressed observation payload range per object"
        ),
        "authored_clip_fallback_required": True,
    }


def load_manifest(
    root: Path, path: Path, runner_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "local-header manifest")
    data = resolved.read_bytes()
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise HeaderDiscoveryError(f"parse local-header manifest: {error}") from error
    if "tail_audit_external_path" in manifest:
        raise HeaderDiscoveryError("manifest must not persist an external path")
    expected = expected_manifest(
        runner_sha256,
        manifest.get("lineage", {}).get("tail_audit_sha256", ""),
        manifest.get("objects", []),
    )
    if manifest != expected:
        raise HeaderDiscoveryError("local-header manifest changed")
    return data, manifest


def configure_core(item: dict[str, Any]) -> None:
    identity = item["archive_identity"]
    discovery.OBJECT_ID = item["dataset_object_id"]
    discovery.ARCHIVE_URL = identity["url"]
    discovery.ARCHIVE_BYTES = identity["bytes"]
    discovery.ARCHIVE_ETAG = identity["etag"]
    discovery.ARCHIVE_LAST_MODIFIED = identity["last_modified_http"]
    discovery.LOCAL_HEADER_BYTES = LOCAL_HEADER_BYTES
    discovery.EXPECTED_AUDIO_ENTRY = item["observation_entry"]["name"]


def parse_header(item: dict[str, Any], header: bytes) -> dict[str, Any]:
    configure_core(item)
    try:
        parsed = discovery.validate_audio_local_header(
            header, item["observation_entry"]
        )
    except discovery.DiscoveryError as error:
        raise HeaderDiscoveryError(
            f"parse {item['dataset_object_id']} local header: {error}"
        ) from error
    compressed_bytes = item["observation_entry"]["compressed_bytes"]
    payload_end = parsed["data_offset"] + compressed_bytes - 1
    if payload_end >= item["archive_identity"]["bytes"]:
        raise HeaderDiscoveryError("compressed observation payload range is invalid")
    return {
        "dataset_object_id": item["dataset_object_id"],
        "role": item["role"],
        "semantic_family_group": item["semantic_family_group"],
        "roster_index": item["roster_index"],
        "archive_identity": item["archive_identity"],
        "observation_entry": item["observation_entry"],
        "local_header": parsed,
        "next_compressed_payload_range": [parsed["data_offset"], payload_end],
    }


def synthetic_archive() -> tuple[bytes, dict[str, Any]]:
    object_id = "fixture_Pan"
    buffer = io.BytesIO(b"\x00" * 70_000)
    buffer.seek(0, io.SEEK_END)

    def write_entry(archive: zipfile.ZipFile, name: str, data: bytes) -> None:
        entry = zipfile.ZipInfo(name, date_time=(2023, 4, 10, 0, 0, 0))
        entry.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(entry, data)

    with zipfile.ZipFile(buffer, mode="a", compression=zipfile.ZIP_DEFLATED) as archive:
        write_entry(
            archive,
            f"{object_id}/preprocessed/deconvolved_0db.npy",
            b"fixture-observation" * 64,
        )
        for index in range(EXPECTED_ENTRY_COUNT - 1):
            write_entry(
                archive,
                f"{object_id}/preprocessed/fixture-{index:02d}.npy",
                f"fixture-{index}".encode() * 16,
            )
    data = buffer.getvalue()
    item = {
        "dataset_object_id": object_id,
        "role": "fixture",
        "semantic_family_group": "fixture_pan",
        "roster_index": -1,
        "url": "https://example.invalid/fixture_Pan.zip",
        "bytes": len(data),
        "etag": '"fixture"',
        "last_modified_http": "Mon, 10 Apr 2023 00:00:00 GMT",
        "accept_ranges": "bytes",
        "response_body_bytes": 0,
        "tail_range": [len(data) - TAIL_BYTES, len(data) - 1],
        "tail_bytes": TAIL_BYTES,
        "expected_entry_count": EXPECTED_ENTRY_COUNT,
        "expected_observation_entry": (f"{object_id}/preprocessed/deconvolved_0db.npy"),
    }
    return data, tail_discovery.parse_tail(item, data[-TAIL_BYTES:])


def fixture_report(runner_sha256: str) -> dict[str, Any]:
    archive, tail_object = synthetic_archive()
    item = {
        "dataset_object_id": tail_object["dataset_object_id"],
        "role": tail_object["role"],
        "semantic_family_group": tail_object["semantic_family_group"],
        "roster_index": tail_object["roster_index"],
        "archive_identity": tail_object["archive_identity"],
        "observation_entry": tail_object["observation_entry"],
        "local_header_range": tail_object["next_local_header_range"],
    }
    start, end = item["local_header_range"]
    header = archive[start : end + 1]
    parsed = parse_header(item, header)
    payload_range = parsed["next_compressed_payload_range"]
    checks = {
        "local_header_is_exactly_bounded": len(header) == LOCAL_HEADER_BYTES,
        "header_matches_central_directory": (
            parsed["local_header"]["local_offset"]
            == item["observation_entry"]["local_offset"]
        ),
        "payload_range_is_derived_not_read": (
            payload_range[1] - payload_range[0] + 1
            == item["observation_entry"]["compressed_bytes"]
        ),
        "payload_range_stays_in_archive": (
            0 <= payload_range[0] <= payload_range[1] < len(archive)
        ),
    }
    if not all(checks.values()):
        raise HeaderDiscoveryError(f"synthetic local-header fixture failed: {checks}")
    return {
        "schema": REPORT_SCHEMAS["fixture"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualHeaderParserFixtureSupported",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "claim": (
            "SYNTHETIC_ZIP_LOCAL_HEADER_CONTROL_ONLY / ZERO_NETWORK_OR_MEMBER_PAYLOAD"
        ),
        "header_sha256": sha256_bytes(header),
        "parsed": parsed,
        "checks": checks,
        "network_requests": 0,
        "range_requests": 0,
        "local_header_bytes_read": LOCAL_HEADER_BYTES,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": "bind a successful real ZIP-tail audit before real header preflight",
    }


def preflight_report(
    root: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    fixture = fixture_report(runner_sha256)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualExactLocalHeaderRangesFrozen",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "claim": manifest["claim"],
        "sources": validate_sources(root),
        "lineage": manifest["lineage"],
        "objects": manifest["objects"],
        "access": manifest["access"],
        "fixture_report_sha256": sha256_bytes(canonical_json(fixture)),
        "network_requests": 0,
        "range_requests": 0,
        "local_header_bytes_read": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "commit this repeated preflight, then read exactly three headers once"
        ),
    }


def validate_preflight(
    root: Path,
    path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "local-header preflight")
    data = resolved.read_bytes()
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise HeaderDiscoveryError(f"parse local-header preflight: {error}") from error
    if report != preflight_report(root, manifest_bytes, manifest, runner_sha256):
        raise HeaderDiscoveryError("local-header preflight changed")
    return data, report


def acquire_headers(
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    output: Path,
) -> tuple[dict[str, Any], bytes]:
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent)
    )
    objects = []
    attempted = 0
    failure = None
    try:
        for item in manifest["objects"]:
            attempted += 1
            configure_core(item)
            start, end = item["local_header_range"]
            header, response = discovery.range_get(start, end - start + 1)
            name = f"{item['dataset_object_id']}-local-header.bin"
            (staging / name).write_bytes(header)
            objects.append(
                {
                    "dataset_object_id": item["dataset_object_id"],
                    "local_header_path": name,
                    "local_header_sha256": sha256_bytes(header),
                    "response": response,
                }
            )
    except (OSError, RuntimeError) as error:
        failure = f"{type(error).__name__}: {error}"
    supported = failure is None and len(objects) == len(EXPECTED_OBJECTS)
    report = {
        "schema": REPORT_SCHEMAS["acquire"],
        "status": "Validated",
        "decision": (
            "RealImpactColoredResidualExactLocalHeadersAcquired"
            if supported
            else "RealImpactColoredResidualLocalHeaderAcquisitionRejected"
        ),
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "claim": manifest["claim"],
        "network_requests_attempted": attempted,
        "objects": objects,
        "failure": failure,
        "additional_requests_allowed": 0,
        "local_header_bytes_read": sum(item["response"]["bytes"] for item in objects),
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "audit immutable headers twice before any member payload request"
            if supported
            else "stop without retry, range growth or member access"
        ),
    }
    report_bytes = canonical_json(report)
    try:
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report, report_bytes


def audit_report(
    root: Path,
    acquisition_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    directory = external_directory(root, acquisition_path, "local-header acquisition")
    report_path = directory / "report.json"
    acquisition_bytes = report_path.read_bytes()
    try:
        acquisition = json.loads(acquisition_bytes)
    except json.JSONDecodeError as error:
        raise HeaderDiscoveryError(
            f"parse local-header acquisition: {error}"
        ) from error
    if (
        acquisition.get("decision")
        != "RealImpactColoredResidualExactLocalHeadersAcquired"
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("network_requests_attempted") != len(EXPECTED_OBJECTS)
        or acquisition.get("local_header_bytes_read")
        != LOCAL_HEADER_BYTES * len(EXPECTED_OBJECTS)
        or acquisition.get("member_payload_bytes_read") != 0
        or acquisition.get("shadow_payload_bytes_read") != 0
        or acquisition.get("quality_domain_or_runtime_admission") is not False
        or acquisition.get("authored_clip_fallback_required") is not True
    ):
        raise HeaderDiscoveryError("local-header acquisition lineage changed")
    parsed = []
    for item, acquired in zip(
        manifest["objects"], acquisition.get("objects", []), strict=True
    ):
        if acquired.get("dataset_object_id") != item["dataset_object_id"]:
            raise HeaderDiscoveryError("local-header acquisition object order changed")
        path = directory / acquired["local_header_path"]
        header = path.read_bytes()
        if sha256_bytes(header) != acquired["local_header_sha256"]:
            raise HeaderDiscoveryError("cached local-header hash changed")
        parsed.append(parse_header(item, header))
    return {
        "schema": REPORT_SCHEMAS["audit"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualCompressedPayloadRangesDiscovered",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "claim": manifest["claim"],
        "objects": parsed,
        "network_requests": 0,
        "local_header_bytes_read_from_cache": LOCAL_HEADER_BYTES * len(parsed),
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "freeze exact compressed payload ranges in a new manifest before "
            "any member access"
        ),
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "fixture":
        if any(
            value is not None
            for value in [
                arguments.tail_audit,
                arguments.manifest,
                arguments.preflight,
                arguments.acquisition,
            ]
        ):
            raise HeaderDiscoveryError("fixture accepts only --output")
        output = external_output(root, arguments.output, directory=False)
        report = fixture_report(runner_sha256)
        report_bytes = canonical_json(report)
        output.write_bytes(report_bytes)
    elif arguments.stage == "manifest":
        if (
            arguments.tail_audit is None
            or arguments.manifest is not None
            or arguments.preflight is not None
            or arguments.acquisition is not None
        ):
            raise HeaderDiscoveryError("manifest requires only --tail-audit")
        output = external_output(root, arguments.output, directory=False)
        tail_bytes, _, objects = read_tail_audit(root, arguments.tail_audit)
        report_bytes = canonical_json(
            expected_manifest(runner_sha256, sha256_bytes(tail_bytes), objects)
        )
        output.write_bytes(report_bytes)
        print(f"colored residual local-header manifest: {output}")
        print(f"manifest sha256: {sha256_bytes(report_bytes)}")
        return 0
    else:
        if arguments.manifest is None or arguments.tail_audit is not None:
            raise HeaderDiscoveryError(
                "non-manifest stages require only --manifest lineage"
            )
        manifest_bytes, manifest = load_manifest(
            root, arguments.manifest, runner_sha256
        )
        if arguments.stage == "preflight":
            if arguments.preflight is not None or arguments.acquisition is not None:
                raise HeaderDiscoveryError("preflight accepts no later-stage input")
            output = external_output(root, arguments.output, directory=False)
            report = preflight_report(root, manifest_bytes, manifest, runner_sha256)
            report_bytes = canonical_json(report)
            output.write_bytes(report_bytes)
        elif arguments.stage == "acquire":
            if arguments.preflight is None or arguments.acquisition is not None:
                raise HeaderDiscoveryError("acquire requires only --preflight")
            validate_preflight(
                root,
                arguments.preflight,
                manifest_bytes,
                manifest,
                runner_sha256,
            )
            output = external_output(root, arguments.output, directory=True)
            report, report_bytes = acquire_headers(
                manifest_bytes, manifest, runner_sha256, output
            )
        else:
            if arguments.preflight is not None or arguments.acquisition is None:
                raise HeaderDiscoveryError("audit requires only --acquisition")
            output = external_output(root, arguments.output, directory=False)
            report = audit_report(
                root,
                arguments.acquisition,
                manifest_bytes,
                manifest,
                runner_sha256,
            )
            report_bytes = canonical_json(report)
            output.write_bytes(report_bytes)
    print(f"colored residual local-header {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print(f"local header bytes read: {report.get('local_header_bytes_read', 0)}")
    print("member payload bytes read: 0")
    print("shadow payload bytes read: 0")
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
