#!/usr/bin/env python3
"""Discover colored-residual observation entries from bounded ZIP tails only."""

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

import physical_sound_realimpact_observation_discovery as discovery

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-colored-residual-tail-discovery.manifest.v1"
)
REPORT_SCHEMAS = {
    "fixture": (
        "nextengine.experimental-realimpact-colored-residual-tail-fixture.report.v1"
    ),
    "preflight": (
        "nextengine.experimental-realimpact-colored-residual-tail-preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-colored-residual-tail-acquisition.report.v1"
    ),
    "audit": (
        "nextengine.experimental-realimpact-colored-residual-tail-audit.report.v1"
    ),
}
REVISION = "fresh-pan-piepan-cup-tail-only-discovery-v1"
IDENTITY_RUNNER_SHA256 = (
    "898d1af13bf4dda4c3efed77b611ad757ccfe71f0548532e29c1a38c1a6c7c58"
)
IDENTITY_MANIFEST_SHA256 = (
    "55be5ed2e71657a344251ccf6068720b0eb0a7f329866e680ca8fffc61b01abe"
)
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


class TailDiscoveryError(RuntimeError):
    """The frozen ZIP-tail discovery boundary failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage",
        required=True,
        choices=["fixture", "manifest", "preflight", "acquire", "audit"],
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--identity-audit", type=Path)
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
        raise TailDiscoveryError(f"{label} must be an external file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise TailDiscoveryError(f"{label} must be an external directory: {resolved}")
    return resolved


def external_output(root: Path, path: Path, *, directory: bool) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise TailDiscoveryError(f"output must remain external: {resolved}")
    if directory:
        if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
            raise TailDiscoveryError(
                f"output directory is not absent/empty: {resolved}"
            )
    elif resolved.exists():
        raise TailDiscoveryError(f"refusing to replace output: {resolved}")
    return resolved


def validate_sources(root: Path) -> list[dict[str, Any]]:
    references = [
        {
            "path": "lab/scripts/physical_sound_realimpact_colored_residual_discovery.py",
            "sha256": IDENTITY_RUNNER_SHA256,
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
            raise TailDiscoveryError(f"bound source changed: {reference['path']}")
        result.append({**reference, "bytes": path.stat().st_size})
    return result


def read_identity_audit(
    root: Path, path: Path
) -> tuple[bytes, dict[str, Any], list[dict[str, Any]]]:
    resolved = external_file(root, path, "archive identity audit")
    data = resolved.read_bytes()
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise TailDiscoveryError(f"parse archive identity audit: {error}") from error
    if (
        report.get("schema")
        != "nextengine.experimental-realimpact-colored-residual-archive-identity-audit.report.v1"
        or report.get("decision")
        != "RealImpactColoredResidualArchiveIdentitiesVerified"
        or report.get("runner_sha256") != IDENTITY_RUNNER_SHA256
        or report.get("manifest_sha256") != IDENTITY_MANIFEST_SHA256
        or report.get("network_requests") != 0
        or report.get("range_requests") != 0
        or report.get("response_body_bytes") != 0
        or report.get("member_payload_bytes_read") != 0
        or report.get("shadow_payload_bytes_read") != 0
        or report.get("quality_domain_or_runtime_admission") is not False
        or report.get("authored_clip_fallback_required") is not True
    ):
        raise TailDiscoveryError("archive identity audit lineage changed")
    identities = report.get("identities", [])
    if len(identities) != len(EXPECTED_OBJECTS):
        raise TailDiscoveryError("archive identity count changed")
    for observed, expected in zip(identities, EXPECTED_OBJECTS, strict=True):
        object_id, role, family, roster_index = expected
        if (
            observed.get("dataset_object_id") != object_id
            or observed.get("role") != role
            or observed.get("semantic_family_group") != family
            or observed.get("roster_index") != roster_index
            or observed.get("bytes", 0) < TAIL_BYTES
            or observed.get("accept_ranges") != "bytes"
            or not observed.get("etag")
            or not observed.get("last_modified_http")
            or observed.get("response_body_bytes") != 0
        ):
            raise TailDiscoveryError(f"archive identity changed: {observed}")
    return data, report, identities


def expected_manifest(
    runner_sha256: str,
    identity_audit_sha256: str,
    identities: list[dict[str, Any]],
) -> dict[str, Any]:
    objects = []
    for item in identities:
        start = item["bytes"] - TAIL_BYTES
        objects.append(
            {
                **item,
                "tail_range": [start, item["bytes"] - 1],
                "tail_bytes": TAIL_BYTES,
                "expected_entry_count": EXPECTED_ENTRY_COUNT,
                "expected_observation_entry": (
                    f"{item['dataset_object_id']}/preprocessed/deconvolved_0db.npy"
                ),
            }
        )
    return {
        "schema": MANIFEST_SCHEMA,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_colored_residual_tail_discovery.py",
            "sha256": runner_sha256,
        },
        "sources": [
            {
                "path": "lab/scripts/physical_sound_realimpact_colored_residual_discovery.py",
                "sha256": IDENTITY_RUNNER_SHA256,
            },
            {
                "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
                "sha256": DISCOVERY_CORE_SHA256,
            },
        ],
        "lineage": {
            "identity_manifest_sha256": IDENTITY_MANIFEST_SHA256,
            "identity_audit_sha256": identity_audit_sha256,
            "identity_decision": "RealImpactColoredResidualArchiveIdentitiesVerified",
        },
        "claim": (
            "THREE_EXACT_ZIP_TAILS_AND_CENTRAL_DIRECTORY_ONLY / ZERO_LOCAL_"
            "HEADER_OR_MEMBER_PAYLOAD / NO_QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_"
            "PRODUCTION_CREDIT"
        ),
        "objects": objects,
        "access": {
            "requests": 3,
            "requests_per_object": 1,
            "range_bytes_per_object": TAIL_BYTES,
            "total_range_bytes": TAIL_BYTES * len(objects),
            "local_header_requests": 0,
            "member_payload_bytes": 0,
            "shadow_requests": 0,
            "retry_or_prefix_growth": False,
        },
        "next_stage_if_supported": (
            "freeze one exact 30-byte observation local-header range per object"
        ),
        "authored_clip_fallback_required": True,
    }


def load_manifest(
    root: Path, path: Path, runner_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "tail discovery manifest")
    data = resolved.read_bytes()
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise TailDiscoveryError(f"parse tail discovery manifest: {error}") from error
    if "identity_audit_external_path" in manifest:
        raise TailDiscoveryError("manifest must not persist an external path")
    lineage = manifest.get("lineage", {})
    identities = [
        {
            key: value
            for key, value in item.items()
            if key
            not in {
                "tail_range",
                "tail_bytes",
                "expected_entry_count",
                "expected_observation_entry",
            }
        }
        for item in manifest.get("objects", [])
    ]
    expected = expected_manifest(
        runner_sha256, lineage.get("identity_audit_sha256", ""), identities
    )
    if manifest != expected:
        raise TailDiscoveryError("tail discovery manifest changed")
    return data, manifest


def configure_core(item: dict[str, Any]) -> None:
    discovery.OBJECT_ID = item["dataset_object_id"]
    discovery.ARCHIVE_URL = item["url"]
    discovery.ARCHIVE_BYTES = item["bytes"]
    discovery.ARCHIVE_ETAG = item["etag"]
    discovery.ARCHIVE_LAST_MODIFIED = item["last_modified_http"]
    discovery.TAIL_BYTES = TAIL_BYTES
    discovery.TAIL_START = item["tail_range"][0]
    discovery.LOCAL_HEADER_BYTES = LOCAL_HEADER_BYTES
    discovery.EXPECTED_ENTRY_COUNT = EXPECTED_ENTRY_COUNT
    discovery.EXPECTED_AUDIO_ENTRY = item["expected_observation_entry"]


def parse_tail(item: dict[str, Any], tail: bytes) -> dict[str, Any]:
    if len(tail) != TAIL_BYTES:
        raise TailDiscoveryError("tail byte count changed")
    configure_core(item)
    try:
        central, entries = discovery.discover_from_tail(tail)
    except discovery.DiscoveryError as error:
        raise TailDiscoveryError(
            f"parse {item['dataset_object_id']} tail: {error}"
        ) from error
    matches = [
        entry
        for entry in entries
        if entry["name"] == item["expected_observation_entry"]
    ]
    if len(matches) != 1:
        raise TailDiscoveryError("expected observation entry is not unique")
    observation = matches[0]
    local_offset = observation["local_offset"]
    if local_offset < 0 or local_offset + LOCAL_HEADER_BYTES > item["bytes"]:
        raise TailDiscoveryError("observation local-header range is invalid")
    return {
        "dataset_object_id": item["dataset_object_id"],
        "role": item["role"],
        "semantic_family_group": item["semantic_family_group"],
        "roster_index": item["roster_index"],
        "archive_identity": {
            "url": item["url"],
            "bytes": item["bytes"],
            "etag": item["etag"],
            "last_modified_http": item["last_modified_http"],
            "accept_ranges": item["accept_ranges"],
        },
        "tail_sha256": sha256_bytes(tail),
        "central_directory": central,
        "entries": entries,
        "observation_entry": observation,
        "next_local_header_range": [
            local_offset,
            local_offset + LOCAL_HEADER_BYTES - 1,
        ],
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
    if len(data) <= TAIL_BYTES:
        raise TailDiscoveryError("synthetic ZIP does not exercise a bounded tail")
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
    return data[-TAIL_BYTES:], item


def fixture_report(runner_sha256: str) -> dict[str, Any]:
    tail, item = synthetic_archive()
    parsed = parse_tail(item, tail)
    checks = {
        "tail_is_exactly_bounded": len(tail) == TAIL_BYTES,
        "all_entries_parsed": parsed["central_directory"]["entry_count"]
        == EXPECTED_ENTRY_COUNT,
        "observation_is_deflated": parsed["observation_entry"]["method"] == 8,
        "local_header_range_is_derived_not_read": (
            parsed["next_local_header_range"][1]
            - parsed["next_local_header_range"][0]
            + 1
            == LOCAL_HEADER_BYTES
        ),
    }
    if not all(checks.values()):
        raise TailDiscoveryError(f"synthetic tail fixture failed: {checks}")
    return {
        "schema": REPORT_SCHEMAS["fixture"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualTailParserFixtureSupported",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "claim": (
            "SYNTHETIC_ZIP_TAIL_AND_CENTRAL_DIRECTORY_CONTROL_ONLY / ZERO_"
            "NETWORK_LOCAL_HEADER_OR_MEMBER_PAYLOAD"
        ),
        "fixture_tail_sha256": sha256_bytes(tail),
        "parsed": parsed,
        "checks": checks,
        "network_requests": 0,
        "range_requests": 0,
        "local_header_bytes_read": 0,
        "member_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": "bind a successful real identity audit before any real tail preflight",
    }


def preflight_report(
    root: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    fixture = fixture_report(runner_sha256)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualExactTailRangesFrozen",
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
        "next_action": "commit this repeated preflight, then read exactly three frozen tails once",
    }


def validate_preflight(
    root: Path,
    path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "tail discovery preflight")
    data = resolved.read_bytes()
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise TailDiscoveryError(f"parse tail discovery preflight: {error}") from error
    expected = preflight_report(root, manifest_bytes, manifest, runner_sha256)
    if report != expected:
        raise TailDiscoveryError("tail discovery preflight changed")
    return data, report


def acquire_tails(
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
            tail, response = discovery.range_get(item["tail_range"][0], TAIL_BYTES)
            name = f"{item['dataset_object_id']}-tail.bin"
            (staging / name).write_bytes(tail)
            objects.append(
                {
                    "dataset_object_id": item["dataset_object_id"],
                    "tail_path": name,
                    "tail_sha256": sha256_bytes(tail),
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
            "RealImpactColoredResidualExactTailsAcquired"
            if supported
            else "RealImpactColoredResidualTailAcquisitionRejected"
        ),
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "claim": manifest["claim"],
        "network_requests_attempted": attempted,
        "objects": objects,
        "failure": failure,
        "additional_requests_allowed": 0,
        "tail_bytes_read": sum(item["response"]["bytes"] for item in objects),
        "local_header_bytes_read": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "audit the immutable tails twice before any local-header request"
            if supported
            else "stop without retry, prefix growth, header or member access"
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
    directory = external_directory(root, acquisition_path, "tail acquisition")
    report_path = directory / "report.json"
    acquisition_bytes = report_path.read_bytes()
    try:
        acquisition = json.loads(acquisition_bytes)
    except json.JSONDecodeError as error:
        raise TailDiscoveryError(f"parse tail acquisition: {error}") from error
    if (
        acquisition.get("decision") != "RealImpactColoredResidualExactTailsAcquired"
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("network_requests_attempted") != len(EXPECTED_OBJECTS)
        or acquisition.get("tail_bytes_read") != TAIL_BYTES * len(EXPECTED_OBJECTS)
        or acquisition.get("local_header_bytes_read") != 0
        or acquisition.get("member_payload_bytes_read") != 0
        or acquisition.get("shadow_payload_bytes_read") != 0
        or acquisition.get("quality_domain_or_runtime_admission") is not False
        or acquisition.get("authored_clip_fallback_required") is not True
    ):
        raise TailDiscoveryError("tail acquisition lineage changed")
    parsed = []
    for item, acquired in zip(
        manifest["objects"], acquisition.get("objects", []), strict=True
    ):
        if acquired.get("dataset_object_id") != item["dataset_object_id"]:
            raise TailDiscoveryError("tail acquisition object order changed")
        path = directory / acquired["tail_path"]
        tail = path.read_bytes()
        if sha256_bytes(tail) != acquired["tail_sha256"]:
            raise TailDiscoveryError("cached tail hash changed")
        parsed.append(parse_tail(item, tail))
    return {
        "schema": REPORT_SCHEMAS["audit"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualObservationLocalHeaderRangesDiscovered",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "claim": manifest["claim"],
        "objects": parsed,
        "network_requests": 0,
        "tail_bytes_read_from_cache": TAIL_BYTES * len(parsed),
        "local_header_bytes_read": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "freeze these exact 30-byte local-header ranges in a new manifest "
            "before any header or member access"
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
                arguments.identity_audit,
                arguments.manifest,
                arguments.preflight,
                arguments.acquisition,
            ]
        ):
            raise TailDiscoveryError("fixture accepts only --output")
        output = external_output(root, arguments.output, directory=False)
        report = fixture_report(runner_sha256)
        report_bytes = canonical_json(report)
        output.write_bytes(report_bytes)
    elif arguments.stage == "manifest":
        if (
            arguments.identity_audit is None
            or arguments.manifest is not None
            or arguments.preflight is not None
            or arguments.acquisition is not None
        ):
            raise TailDiscoveryError("manifest requires only --identity-audit")
        output = external_output(root, arguments.output, directory=False)
        identity_bytes, _, identities = read_identity_audit(
            root, arguments.identity_audit
        )
        report_bytes = canonical_json(
            expected_manifest(runner_sha256, sha256_bytes(identity_bytes), identities)
        )
        output.write_bytes(report_bytes)
        print(f"colored residual tail manifest: {output}")
        print(f"manifest sha256: {sha256_bytes(report_bytes)}")
        return 0
    else:
        if arguments.manifest is None or arguments.identity_audit is not None:
            raise TailDiscoveryError(
                "non-manifest stages require only --manifest lineage"
            )
        manifest_bytes, manifest = load_manifest(
            root, arguments.manifest, runner_sha256
        )
        if arguments.stage == "preflight":
            if arguments.preflight is not None or arguments.acquisition is not None:
                raise TailDiscoveryError("preflight accepts no later-stage input")
            output = external_output(root, arguments.output, directory=False)
            report = preflight_report(root, manifest_bytes, manifest, runner_sha256)
            report_bytes = canonical_json(report)
            output.write_bytes(report_bytes)
        elif arguments.stage == "acquire":
            if arguments.preflight is None or arguments.acquisition is not None:
                raise TailDiscoveryError("acquire requires only --preflight")
            validate_preflight(
                root,
                arguments.preflight,
                manifest_bytes,
                manifest,
                runner_sha256,
            )
            output = external_output(root, arguments.output, directory=True)
            report, report_bytes = acquire_tails(
                manifest_bytes, manifest, runner_sha256, output
            )
        else:
            if arguments.preflight is not None or arguments.acquisition is None:
                raise TailDiscoveryError("audit requires only --acquisition")
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
    print(f"colored residual tail {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print("local header bytes read: 0")
    print("member payload bytes read: 0")
    print("shadow payload bytes read: 0")
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
