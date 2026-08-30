#!/usr/bin/env python3
"""Discover only fresh archive identities for the colored-residual successor."""

from __future__ import annotations

import argparse
import hashlib
import ipaddress
import json
import socket
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-colored-residual-discovery.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-colored-residual-discovery-preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-colored-residual-archive-identity.report.v1"
    ),
    "audit": (
        "nextengine.experimental-realimpact-colored-residual-archive-identity-audit.report.v1"
    ),
}
REVISION = "fresh-pan-piepan-cup-archive-identity-discovery-v1"
PREREGISTRATION_RUNNER_SHA256 = (
    "fe8a12f516ee8f85dc515c23cf2b7f67cc6bcc4d12a2b7f20c7f69d0626cebf5"
)
PREREGISTRATION_MANIFEST_SHA256 = (
    "f03e428ee9b6005b96519869541fd27a301e0343a4e0f800b5d74f770d3e564a"
)
PREREGISTRATION_PREFLIGHT_SHA256 = (
    "4c754f0630d16ade6fd7de3479fc15bb54a193a9f928b72eaf3e1342424da360"
)
MINIMUM_ARCHIVE_BYTES = 64 * 1024 * 1024
OBJECTS = [
    {
        "dataset_object_id": "19_Pan",
        "roster_index": 3,
        "semantic_family_group": "pan",
        "role": "calibration",
        "url": "https://downloads.cs.stanford.edu/viscam/RealImpact/19_Pan.zip",
    },
    {
        "dataset_object_id": "37_PiePan",
        "roster_index": 14,
        "semantic_family_group": "pan",
        "role": "calibration",
        "url": "https://downloads.cs.stanford.edu/viscam/RealImpact/37_PiePan.zip",
    },
    {
        "dataset_object_id": "22_Cup",
        "roster_index": 4,
        "semantic_family_group": "cup",
        "role": "holdout",
        "url": "https://downloads.cs.stanford.edu/viscam/RealImpact/22_Cup.zip",
    },
]


class DiscoveryError(RuntimeError):
    """The frozen identity discovery boundary failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage", required=True, choices=["manifest", *sorted(REPORT_SCHEMAS)]
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--preregistration-manifest", type=Path)
    parser.add_argument("--preregistration-preflight", type=Path)
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
        raise DiscoveryError(f"{label} must be an external file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise DiscoveryError(f"output must be a new external file: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_colored_residual_discovery.py",
            "sha256": runner_sha256,
        },
        "preregistration": {
            "runner_path": (
                "lab/scripts/physical_sound_realimpact_colored_residual_preregistration.py"
            ),
            "runner_sha256": PREREGISTRATION_RUNNER_SHA256,
            "manifest_sha256": PREREGISTRATION_MANIFEST_SHA256,
            "preflight_sha256": PREREGISTRATION_PREFLIGHT_SHA256,
            "required_decision": "RealImpactColoredResidualSuccessorProtocolFrozen",
        },
        "claim": (
            "FRESH_ARCHIVE_IDENTITY_ONLY / ZERO_RANGE_OR_MEMBER_PAYLOAD / NO_"
            "MATERIAL_QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_PRODUCTION_CREDIT"
        ),
        "objects": OBJECTS,
        "request_profile": {
            "method": "HEAD",
            "requests": 3,
            "requests_per_object": 1,
            "redirect_final_url_must_equal_requested_url": True,
            "required_status": 200,
            "required_accept_ranges": "bytes",
            "minimum_content_length": MINIMUM_ARCHIVE_BYTES,
            "required_identity_headers": ["ETag", "Last-Modified"],
            "retry": False,
            "response_body_bytes": 0,
        },
        "next_stage_if_supported": (
            "freeze identities, then preregister exact ZIP tail/local-header ranges"
        ),
        "forbidden": {
            "range_get": True,
            "zip_tail_or_header_read": True,
            "member_payload_read": True,
            "shadow_request": True,
            "retry_or_object_substitution": True,
            "quality_domain_or_runtime_admission": True,
        },
        "authored_clip_fallback_required": True,
    }


def read_exact_json(
    root: Path, path: Path, label: str, expected_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    data = external_file(root, path, label).read_bytes()
    observed = sha256_bytes(data)
    if observed != expected_sha256:
        raise DiscoveryError(f"{label} hash changed: {observed}")
    try:
        parsed = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiscoveryError(f"parse {label}: {error}") from error
    return data, parsed


def validate_preregistration(
    root: Path, prereg_manifest_path: Path, prereg_preflight_path: Path
) -> dict[str, Any]:
    runner = (
        root
        / "lab/scripts/physical_sound_realimpact_colored_residual_preregistration.py"
    )
    if sha256_file(runner) != PREREGISTRATION_RUNNER_SHA256:
        raise DiscoveryError("colored-residual preregistration runner changed")
    manifest_bytes, manifest = read_exact_json(
        root,
        prereg_manifest_path,
        "colored-residual preregistration manifest",
        PREREGISTRATION_MANIFEST_SHA256,
    )
    preflight_bytes, preflight = read_exact_json(
        root,
        prereg_preflight_path,
        "colored-residual preregistration preflight",
        PREREGISTRATION_PREFLIGHT_SHA256,
    )
    if (
        manifest.get("runner", {}).get("sha256") != PREREGISTRATION_RUNNER_SHA256
        or preflight.get("decision")
        != "RealImpactColoredResidualSuccessorProtocolFrozen"
        or preflight.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or preflight.get("network_requests") != 0
        or preflight.get("new_member_payload_bytes_read") != 0
        or preflight.get("shadow_payload_bytes_read") != 0
        or preflight.get("quality_domain_or_runtime_admission") is not False
        or preflight.get("authored_clip_fallback_required") is not True
    ):
        raise DiscoveryError("colored-residual preregistration lineage changed")
    expected_fresh = [
        {
            "dataset_object_id": item["dataset_object_id"],
            "roster_index": item["roster_index"],
            "semantic_family_group": item["semantic_family_group"],
            "role": item["role"],
            "archive_url": item["url"],
            "archive_identity": "DISCOVERY_REQUIRED_BEFORE_MEMBER_ACCESS",
        }
        for item in OBJECTS
    ]
    if preflight.get("fresh_objects") != expected_fresh:
        raise DiscoveryError("fresh object roster changed")
    return {
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "preflight_sha256": sha256_bytes(preflight_bytes),
    }


def load_manifest(
    root: Path, path: Path, runner_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "identity discovery manifest")
    data = resolved.read_bytes()
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiscoveryError(f"parse identity discovery manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise DiscoveryError("identity discovery manifest changed")
    return data, manifest


def fixture_report() -> dict[str, Any]:
    fixture = {
        "status": 200,
        "final_url": OBJECTS[0]["url"],
        "content_length": str(2_000_000_000),
        "accept_ranges": "bytes",
        "etag": '"fixture-etag"',
        "last_modified": "Mon, 10 Apr 2023 00:00:00 GMT",
    }
    validated = validate_identity_response(OBJECTS[0], fixture)
    checks = {
        "content_length_parsed": validated["bytes"] == 2_000_000_000,
        "headers_preserved": validated["etag"] == '"fixture-etag"'
        and validated["last_modified_http"] == "Mon, 10 Apr 2023 00:00:00 GMT",
        "body_bytes_zero": validated["response_body_bytes"] == 0,
    }
    if not all(checks.values()):
        raise DiscoveryError(f"identity fixture failed: {checks}")
    return {"response": validated, "checks": checks}


def preflight_report(
    root: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    prereg_manifest_path: Path,
    prereg_preflight_path: Path,
) -> dict[str, Any]:
    lineage = validate_preregistration(
        root, prereg_manifest_path, prereg_preflight_path
    )
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualArchiveIdentityDiscoveryFrozen",
        "claim": manifest["claim"],
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "lineage": lineage,
        "objects": manifest["objects"],
        "request_profile": manifest["request_profile"],
        "fixture": fixture_report(),
        "network_requests": 0,
        "range_requests": 0,
        "response_body_bytes": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": "commit this repeated preflight, then make exactly three HEAD requests once",
    }


def validate_preflight(
    root: Path,
    path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    prereg_manifest_path: Path,
    prereg_preflight_path: Path,
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "identity discovery preflight")
    data = resolved.read_bytes()
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiscoveryError(f"parse identity discovery preflight: {error}") from error
    expected = preflight_report(
        root,
        manifest_bytes,
        manifest,
        runner_sha256,
        prereg_manifest_path,
        prereg_preflight_path,
    )
    if report != expected:
        raise DiscoveryError("identity discovery preflight changed")
    return data, report


def ensure_public_archive_host(url: str) -> None:
    parsed = urllib.parse.urlparse(url)
    if (
        parsed.scheme != "https"
        or parsed.hostname is None
        or parsed.username is not None
        or parsed.password is not None
    ):
        raise DiscoveryError("archive URL is not credential-free HTTPS")
    addresses = {
        item[4][0]
        for item in socket.getaddrinfo(parsed.hostname, 443, type=socket.SOCK_STREAM)
    }
    if not addresses or any(
        not ipaddress.ip_address(value).is_global for value in addresses
    ):
        raise DiscoveryError(f"archive host did not resolve only publicly: {addresses}")


def validate_identity_response(
    item: dict[str, Any], observed: dict[str, Any]
) -> dict[str, Any]:
    try:
        content_length = int(observed["content_length"])
    except (KeyError, TypeError, ValueError) as error:
        raise DiscoveryError("archive Content-Length is absent or invalid") from error
    if (
        observed.get("status") != 200
        or observed.get("final_url") != item["url"]
        or content_length < MINIMUM_ARCHIVE_BYTES
        or observed.get("accept_ranges") != "bytes"
        or not observed.get("etag")
        or not observed.get("last_modified")
    ):
        raise DiscoveryError(f"archive identity response changed: {observed}")
    return {
        "dataset_object_id": item["dataset_object_id"],
        "roster_index": item["roster_index"],
        "semantic_family_group": item["semantic_family_group"],
        "role": item["role"],
        "url": item["url"],
        "bytes": content_length,
        "etag": observed["etag"],
        "last_modified_http": observed["last_modified"],
        "accept_ranges": observed["accept_ranges"],
        "response_body_bytes": 0,
    }


def head_identity(item: dict[str, Any]) -> dict[str, Any]:
    ensure_public_archive_host(item["url"])
    request = urllib.request.Request(
        item["url"], headers={"Accept-Encoding": "identity"}, method="HEAD"
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            observed = {
                "status": response.status,
                "final_url": response.geturl(),
                "content_length": response.headers.get("Content-Length"),
                "accept_ranges": response.headers.get("Accept-Ranges"),
                "etag": response.headers.get("ETag"),
                "last_modified": response.headers.get("Last-Modified"),
            }
    except urllib.error.HTTPError as error:
        raise DiscoveryError(f"archive HEAD failed: HTTP {error.code}") from error
    return validate_identity_response(item, observed)


def acquisition_report(
    manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    identities = []
    attempted = 0
    failure = None
    try:
        for item in manifest["objects"]:
            attempted += 1
            identities.append(head_identity(item))
    except (OSError, RuntimeError) as error:
        failure = f"{type(error).__name__}: {error}"
    supported = failure is None and len(identities) == len(OBJECTS)
    return {
        "schema": REPORT_SCHEMAS["acquire"],
        "status": "Validated",
        "decision": (
            "RealImpactColoredResidualArchiveIdentitiesDiscovered"
            if supported
            else "RealImpactColoredResidualArchiveIdentityDiscoveryRejected"
        ),
        "claim": manifest["claim"],
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "network_requests_attempted": attempted,
        "identities": identities,
        "failure": failure,
        "additional_requests_allowed": 0,
        "range_requests": 0,
        "response_body_bytes": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "audit this immutable identity report twice before any range request"
            if supported
            else "stop without retry, range request or object substitution"
        ),
    }


def audit_report(
    root: Path,
    acquisition_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    acquisition_file = external_file(root, acquisition_path, "identity acquisition")
    acquisition_bytes = acquisition_file.read_bytes()
    try:
        acquisition = json.loads(acquisition_bytes)
    except json.JSONDecodeError as error:
        raise DiscoveryError(f"parse identity acquisition: {error}") from error
    if (
        acquisition.get("schema") != REPORT_SCHEMAS["acquire"]
        or acquisition.get("decision")
        != "RealImpactColoredResidualArchiveIdentitiesDiscovered"
        or acquisition.get("runner_sha256") != runner_sha256
        or acquisition.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or acquisition.get("network_requests_attempted") != len(OBJECTS)
        or acquisition.get("range_requests") != 0
        or acquisition.get("response_body_bytes") != 0
        or acquisition.get("member_payload_bytes_read") != 0
        or acquisition.get("shadow_payload_bytes_read") != 0
        or acquisition.get("quality_domain_or_runtime_admission") is not False
        or acquisition.get("authored_clip_fallback_required") is not True
    ):
        raise DiscoveryError("identity acquisition lineage changed")
    expected_ids = [item["dataset_object_id"] for item in manifest["objects"]]
    observed_ids = [item["dataset_object_id"] for item in acquisition["identities"]]
    if observed_ids != expected_ids or len(set(observed_ids)) != len(expected_ids):
        raise DiscoveryError("identity acquisition object order changed")
    for expected, observed in zip(
        manifest["objects"], acquisition["identities"], strict=True
    ):
        validate_identity_response(
            expected,
            {
                "status": 200,
                "final_url": observed["url"],
                "content_length": str(observed["bytes"]),
                "accept_ranges": observed["accept_ranges"],
                "etag": observed["etag"],
                "last_modified": observed["last_modified_http"],
            },
        )
    return {
        "schema": REPORT_SCHEMAS["audit"],
        "status": "Validated",
        "decision": "RealImpactColoredResidualArchiveIdentitiesVerified",
        "claim": manifest["claim"],
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "identities": acquisition["identities"],
        "network_requests": 0,
        "range_requests": 0,
        "response_body_bytes": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "freeze exact ZIP tail and observation local-header ranges before "
            "any range or member access"
        ),
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "manifest":
        if any(
            value is not None
            for value in [
                arguments.manifest,
                arguments.preregistration_manifest,
                arguments.preregistration_preflight,
                arguments.preflight,
                arguments.acquisition,
            ]
        ):
            raise DiscoveryError("manifest stage accepts only --output")
        data = canonical_json(expected_manifest(runner_sha256))
        output.write_bytes(data)
        print(f"colored residual identity discovery manifest: {output}")
        print(f"manifest sha256: {sha256_bytes(data)}")
        return 0
    if (
        arguments.manifest is None
        or arguments.preregistration_manifest is None
        or arguments.preregistration_preflight is None
    ):
        raise DiscoveryError("non-manifest stages require all frozen manifests")
    manifest_bytes, manifest = load_manifest(root, arguments.manifest, runner_sha256)
    if arguments.stage == "preflight":
        if arguments.preflight is not None or arguments.acquisition is not None:
            raise DiscoveryError("preflight accepts no later-stage input")
        report = preflight_report(
            root,
            manifest_bytes,
            manifest,
            runner_sha256,
            arguments.preregistration_manifest,
            arguments.preregistration_preflight,
        )
    elif arguments.stage == "acquire":
        if arguments.preflight is None or arguments.acquisition is not None:
            raise DiscoveryError("acquire requires only --preflight")
        validate_preflight(
            root,
            arguments.preflight,
            manifest_bytes,
            manifest,
            runner_sha256,
            arguments.preregistration_manifest,
            arguments.preregistration_preflight,
        )
        report = acquisition_report(manifest_bytes, manifest, runner_sha256)
    else:
        if arguments.preflight is not None or arguments.acquisition is None:
            raise DiscoveryError("audit requires only --acquisition")
        report = audit_report(
            root,
            arguments.acquisition,
            manifest_bytes,
            manifest,
            runner_sha256,
        )
    report_bytes = canonical_json(report)
    output.write_bytes(report_bytes)
    print(f"colored residual identity discovery {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print("range requests: 0")
    print("member payload bytes read: 0")
    print("shadow payload bytes read: 0")
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
