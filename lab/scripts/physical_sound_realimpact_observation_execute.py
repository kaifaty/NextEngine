#!/usr/bin/env python3
"""Execute the frozen REALIMPACT Ceramic Cup observation-admission protocol."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import struct
import tempfile
from typing import Any
import urllib.error
import urllib.request
import zlib

import numpy as np

import physical_sound_realimpact_observation_discovery as discovery
import physical_sound_realimpact_pitcher_calibration as extractor


MANIFEST_SCHEMA = "nextengine.experimental-realimpact-observation-execution.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-observation-execution-preflight.report.v1",
    "acquire": "nextengine.experimental-realimpact-observation-acquisition.report.v1",
    "decode": "nextengine.experimental-realimpact-observation-decode.report.v1",
    "analyze": "nextengine.experimental-realimpact-observation-admission.report.v1",
}
STUDY_ID = "physical-sound-realimpact-observation-first-discriminator"
PROTOCOL_REVISION = "ceramic-cup-observation-admission-v1"
OBJECT_ID = "78_CeramicCup"
SAMPLE_RATE_HZ = 48_000
TOTAL_ROWS = 3_000
ROW_COUNT = 600
REFERENCE_ROW = 7
SAMPLE_COUNT = 208_895
ROW_BYTES = SAMPLE_COUNT * 4
DECODED_BYTES = ROW_COUNT * ROW_BYTES
NPY_HEADER_BYTES = 128
PREFIX_BYTES = 536_870_912

DISCOVERY_RUNNER_SHA256 = "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
DISCOVERY_ACQUISITION_SHA256 = "2b183dae20e38350098ce7b986a356fd34033a1ebdd83d207cc311a2de80782b"
DISCOVERY_AUDIT_SHA256 = "cd68ba79b73aceef579a8243674b1448ccc3248fb074d82486e796551f641b0b"
DISCOVERY_TAIL_SHA256 = "eef6c5388784c32e8943e236c54928b1c56d2a3c4f194c12697552bc39ff351c"
DISCOVERY_AUDIO_HEADER_SHA256 = "a1b86037a7f3903541fc0c88b46577fa9685653b0201c4cb125c4a21fadcb47c"
EXTRACTOR_PYTHON_SHA256 = "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
TRANSFER_CALIBRATION_SHA256 = "2624656ebefbdf20d210ddef3138824219ab8044435dc5440365908a720cbf80"
TRANSFER_DSP_SHA256 = "131bbf42d01e633b6cac0c4e0340178f3f5a0a83324bb64630d8423da8179ca4"
FIXTURE_REPORT_SHA256 = "2e3db2d3f231d69b67b2a3a094f83fbc7ed10144a02f4c424d75bbbb2f85572f"
FIXTURE_SAMPLE_SHA256 = "c8316f81da79b9823cbeaf1c0c49ee1708d2112fca8b4905a792546fd3f40477"

SPATIAL_RANGE = (614_950, 618_705)
CONDITION_RANGE = (2_505_276, 2_506_630)
AUDIO_RANGE = (2_506_735, 2_506_735 + PREFIX_BYTES - 1)

ENTRIES = {
    "vertexXYZ.npy": {
        "name": f"{OBJECT_ID}/preprocessed/vertexXYZ.npy",
        "crc32": "ad958323",
        "compressed_bytes": 441,
        "uncompressed_bytes": 72_128,
        "local_offset": 614_950,
        "method": 8,
    },
    "listenerXYZ.npy": {
        "name": f"{OBJECT_ID}/preprocessed/listenerXYZ.npy",
        "crc32": "ee426e91",
        "compressed_bytes": 2_923,
        "uncompressed_bytes": 72_128,
        "local_offset": 615_683,
        "method": 8,
    },
    "angle.npy": {
        "name": f"{OBJECT_ID}/preprocessed/angle.npy",
        "crc32": "fabb9a2e",
        "compressed_bytes": 292,
        "uncompressed_bytes": 24_128,
        "local_offset": 2_505_276,
        "method": 8,
    },
    "distance.npy": {
        "name": f"{OBJECT_ID}/preprocessed/distance.npy",
        "crc32": "570b48dd",
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "local_offset": 2_505_662,
        "method": 8,
    },
    "micID.npy": {
        "name": f"{OBJECT_ID}/preprocessed/micID.npy",
        "crc32": "082ec0c6",
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "local_offset": 2_506_053,
        "method": 8,
    },
    "vertexID.npy": {
        "name": f"{OBJECT_ID}/preprocessed/vertexID.npy",
        "crc32": "dc5e8a5b",
        "compressed_bytes": 162,
        "uncompressed_bytes": 24_128,
        "local_offset": 2_506_372,
        "method": 8,
    },
    "deconvolved_0db.npy": {
        "name": f"{OBJECT_ID}/preprocessed/deconvolved_0db.npy",
        "crc32": "460b3ca0",
        "compressed_bytes": 2_318_456_222,
        "uncompressed_bytes": 2_506_740_128,
        "local_offset": 2_506_631,
        "data_offset": 2_506_735,
        "method": 8,
    },
}

THRESHOLDS = {
    "minimum_selected_modes": 6,
    "minimum_persistent_mode_recall": 0.50,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_decaying_mode_fraction": 0.50,
    "maximum_median_tail_prediction_rmse_db": 24.0,
}


class ObservationError(RuntimeError):
    """The frozen observation contract or a bounded source invariant failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--input", type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise ObservationError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise ObservationError(f"{label} must be an external directory: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise ObservationError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise ObservationError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "runner_sha256": runner_sha256,
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "development",
            "impact_ordinal": 0,
            "rows": [0, ROW_COUNT - 1],
            "reference_row": REFERENCE_ROW,
            "planter_payload_access_allowed": False,
        },
        "archive": {
            "url": discovery.ARCHIVE_URL,
            "bytes": discovery.ARCHIVE_BYTES,
            "etag": discovery.ARCHIVE_ETAG,
            "last_modified_http": discovery.ARCHIVE_LAST_MODIFIED,
        },
        "discovery": {
            "runner_sha256": DISCOVERY_RUNNER_SHA256,
            "acquisition_report": {
                "path": "discovery-acquisition/report.json",
                "sha256": DISCOVERY_ACQUISITION_SHA256,
            },
            "audit_report": {
                "path": "discovery-audit-a/report.json",
                "sha256": DISCOVERY_AUDIT_SHA256,
            },
            "tail": {
                "path": "discovery-acquisition/tail.bin",
                "sha256": DISCOVERY_TAIL_SHA256,
                "bytes": discovery.TAIL_BYTES,
            },
            "audio_local_header": {
                "path": "discovery-acquisition/audio-local-header.bin",
                "sha256": DISCOVERY_AUDIO_HEADER_SHA256,
                "bytes": discovery.LOCAL_HEADER_BYTES,
            },
        },
        "entries": ENTRIES,
        "requests": {
            "maximum_network_requests": 3,
            "spatial_metadata_range": list(SPATIAL_RANGE),
            "condition_metadata_range": list(CONDITION_RANGE),
            "audio_prefix_range": list(AUDIO_RANGE),
            "audio_prefix_bytes": PREFIX_BYTES,
            "prefix_growth_allowed": False,
            "retry_allowed": False,
        },
        "decode": {
            "audio_dtype": "<f4",
            "audio_shape": [TOTAL_ROWS, SAMPLE_COUNT],
            "npy_header_bytes": NPY_HEADER_BYTES,
            "decoded_rows": ROW_COUNT,
            "decoded_row_bytes": ROW_BYTES,
            "decoded_bytes": DECODED_BYTES,
            "metadata_dtype": {"coordinates": "<f8", "identifiers": "<i8"},
            "metadata_shapes": {"coordinates": [TOTAL_ROWS, 3], "identifiers": [TOTAL_ROWS]},
        },
        "extractor": {
            "profile_id": "injective-modal-16-fft65536-v2",
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "python_source": {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": EXTRACTOR_PYTHON_SHA256,
            },
            "rust_sources": [
                {
                    "path": "tools/xtask/src/physical_sound_registry_command/transfer_calibration.rs",
                    "sha256": TRANSFER_CALIBRATION_SHA256,
                },
                {
                    "path": "tools/xtask/src/physical_sound_registry_command/transfer_calibration/dsp.rs",
                    "sha256": TRANSFER_DSP_SHA256,
                },
            ],
            "rust_fixture": {
                "report_path": "../ps2-realimpact-geometry-spatial-transfer-v1/extractor-parity-fixture-a/report.json",
                "report_sha256": FIXTURE_REPORT_SHA256,
                "sample_path": "../ps2-realimpact-geometry-spatial-transfer-v1/extractor-parity-fixture-a/fixture.f64le",
                "sample_sha256": FIXTURE_SAMPLE_SHA256,
            },
            "thresholds": THRESHOLDS,
            "threshold_selection": "unchanged REALIMPACT transfer-calibration V2 holdout gates",
        },
        "data_policy": {
            "metadata_and_spatial_coordinates_allowed": True,
            "one_impact_observation_rows_allowed": True,
            "additional_audio_or_planter_payload_allowed": False,
            "physics_solver_allowed": False,
            "threshold_or_extractor_tuning_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": "stop with authored-clip fallback if acquisition, decode, row identity, parity or any unchanged V2 observation gate fails; run no physics and open no Planter payload",
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ObservationError("observation manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ObservationError(f"parse observation manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ObservationError("observation execution manifest contract changed")
    return data, manifest


def resolve_reference(base: Path, ref: dict[str, Any], label: str) -> tuple[Path, bytes]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise ObservationError(f"{label} escapes the external physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise ObservationError(f"{label} identity changed")
    if "bytes" in ref and len(data) != ref["bytes"]:
        raise ObservationError(f"{label} byte count changed")
    return path, data


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [manifest["extractor"]["python_source"], *manifest["extractor"]["rust_sources"]]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_file() or not path.is_relative_to(root):
            raise ObservationError(f"bound source escapes repository: {path}")
        actual = sha256_file(path)
        if actual != ref["sha256"]:
            raise ObservationError(f"bound source changed: {ref['path']}")
        reports.append({"path": ref["path"], "sha256": actual, "bytes": path.stat().st_size})
    return reports


def validate_discovery(base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    refs = manifest["discovery"]
    acquisition_path, acquisition_bytes = resolve_reference(
        base, refs["acquisition_report"], "discovery acquisition report"
    )
    _, audit_bytes = resolve_reference(base, refs["audit_report"], "discovery audit report")
    _, tail = resolve_reference(base, refs["tail"], "discovery ZIP tail")
    _, header = resolve_reference(base, refs["audio_local_header"], "audio local header")
    acquisition = json.loads(acquisition_bytes)
    audit = json.loads(audit_bytes)
    central, entries = discovery.discover_from_tail(tail)
    audio = next(entry for entry in entries if entry["name"] == discovery.EXPECTED_AUDIO_ENTRY)
    audio_header = discovery.validate_audio_local_header(header, audio)
    if (
        acquisition.get("decision") != "CeramicCupArchiveAndObservationEntryDiscovered"
        or acquisition.get("runner_sha256") != DISCOVERY_RUNNER_SHA256
        or acquisition.get("network_requests") != 2
        or acquisition.get("audio_payload_bytes_read") != 0
        or acquisition.get("planter_audio_payload_bytes_read") != 0
        or acquisition.get("central_directory") != central
        or acquisition.get("entries") != entries
        or acquisition.get("observation_local_header") != audio_header
        or audit.get("decision") != "CeramicCupObservationDiscoveryCacheVerified"
        or audit.get("acquisition_report_sha256") != DISCOVERY_ACQUISITION_SHA256
        or audit.get("network_requests") != 0
    ):
        raise ObservationError("discovery cache lineage changed")
    for name, expected in ENTRIES.items():
        matches = [entry for entry in entries if entry["name"] == expected["name"]]
        if len(matches) != 1:
            raise ObservationError(f"discovered entry is absent: {name}")
        actual = matches[0]
        for field in ["crc32", "compressed_bytes", "uncompressed_bytes", "local_offset", "method"]:
            if actual[field] != expected[field]:
                raise ObservationError(f"discovered {name} field changed: {field}")
    return {
        "acquisition_report_path": str(acquisition_path),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "audit_report_sha256": sha256_bytes(audit_bytes),
        "tail_sha256": sha256_bytes(tail),
        "audio_local_header_sha256": sha256_bytes(header),
        "entry_count": len(entries),
        "audio_data_offset": audio_header["data_offset"],
    }


def fixture_directory(base: Path, manifest: dict[str, Any]) -> Path:
    report_ref = manifest["extractor"]["rust_fixture"]
    report = (base / report_ref["report_path"]).resolve(strict=True)
    sample = (base / report_ref["sample_path"]).resolve(strict=True)
    if not report.is_relative_to(base.parent) or not sample.is_relative_to(base.parent):
        raise ObservationError("extractor fixture escapes external physical-sound store")
    if sha256_file(report) != report_ref["report_sha256"] or sha256_file(sample) != report_ref["sample_sha256"]:
        raise ObservationError("extractor fixture identity changed")
    if report.parent != sample.parent:
        raise ObservationError("extractor fixture files do not share a directory")
    return report.parent


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "role": "development",
        "impact_ordinal": 0,
        "rows": [0, ROW_COUNT - 1],
        "reference_row": REFERENCE_ROW,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def preflight(
    root: Path, base: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> tuple[dict[str, Any], dict[str, bytes]]:
    discovered = validate_discovery(base, manifest)
    sources = validate_sources(root, manifest)
    fixture = fixture_directory(base, manifest)
    fixture_report, parity = extractor.validate_fixture(fixture)
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "CeramicCupObservationProtocolFrozen",
        "claim": "OBSERVATION_ONLY_PROTOCOL_PREFLIGHT / ZERO_NEW_NETWORK_OR_PAYLOAD_ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "discovery": discovered,
        "bound_sources": sources,
        "extractor_fixture_report_sha256": FIXTURE_REPORT_SHA256,
        "extractor_fixture_analysis": fixture_report["analysis"],
        "extractor_fixture_parity": parity,
        "frozen_requests": manifest["requests"],
        "frozen_decode": manifest["decode"],
        "frozen_thresholds": THRESHOLDS,
        "network_requests": 0,
        "object_payload_bytes_read": 0,
        "next_action": "commit this preflight, then execute the exact three-request observation acquisition once",
    }
    return report, {}


def download_range(start: int, end: int, target: Path) -> dict[str, Any]:
    length = end - start + 1
    if start < 0 or length <= 0 or end >= discovery.ARCHIVE_BYTES:
        raise ObservationError("bounded observation range is invalid")
    discovery.ensure_public_archive_host()
    request = urllib.request.Request(
        discovery.ARCHIVE_URL,
        headers={"Range": f"bytes={start}-{end}", "Accept-Encoding": "identity"},
        method="GET",
    )
    digest = hashlib.sha256()
    count = 0
    try:
        with urllib.request.urlopen(request, timeout=60) as response, target.open("wb") as output:
            status = response.status
            final_url = response.geturl()
            content_range = response.headers.get("Content-Range")
            content_length = response.headers.get("Content-Length")
            etag = response.headers.get("ETag")
            last_modified = response.headers.get("Last-Modified")
            expected_range = f"bytes {start}-{end}/{discovery.ARCHIVE_BYTES}"
            if (
                status != 206
                or final_url != discovery.ARCHIVE_URL
                or content_range != expected_range
                or content_length != str(length)
                or etag != discovery.ARCHIVE_ETAG
                or last_modified != discovery.ARCHIVE_LAST_MODIFIED
            ):
                raise ObservationError(
                    "archive range response identity changed: "
                    f"{status}, {final_url}, {content_range}, {content_length}, {etag}, {last_modified}"
                )
            while count < length:
                block = response.read(min(1024 * 1024, length - count))
                if not block:
                    break
                digest.update(block)
                output.write(block)
                count += len(block)
            if response.read(1):
                raise ObservationError("archive range response exceeded the frozen length")
    except urllib.error.HTTPError as error:
        raise ObservationError(f"archive range request failed: HTTP {error.code}") from error
    if count != length:
        raise ObservationError(f"archive range response truncated: expected {length}, got {count}")
    return {
        "start": start,
        "end": end,
        "bytes": count,
        "content_range": f"bytes {start}-{end}/{discovery.ARCHIVE_BYTES}",
        "etag": discovery.ARCHIVE_ETAG,
        "last_modified_http": discovery.ARCHIVE_LAST_MODIFIED,
        "sha256": digest.hexdigest(),
    }


def acquire(
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    output: Path,
) -> dict[str, Any]:
    discovered = validate_discovery(base, manifest)
    requests = [
        ("spatial-metadata-records.bin", SPATIAL_RANGE, "metadata"),
        ("condition-metadata-records.bin", CONDITION_RANGE, "metadata"),
        ("observation-prefix-512m.deflate", AUDIO_RANGE, "audio"),
    ]
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    responses = []
    attempted = 0
    try:
        for name, (start, end), role in requests:
            attempted += 1
            response = download_range(start, end, staging / name)
            responses.append({"artifact": name, "role": role, **response})
    except Exception as error:
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "status": "Validated",
            "decision": "CeramicCupObservationAcquisitionRejected",
            "claim": "IMMUTABLE_BOUNDED_ACQUISITION_FAILURE / NO_RETRY_OR_PREFIX_GROWTH / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
            **common_report(manifest_bytes, runner_sha256),
            "discovery": discovered,
            "network_requests_attempted": attempted,
            "completed_responses": responses,
            "failure": f"{type(error).__name__}: {error}",
            "additional_requests_allowed": 0,
            "audio_payload_bytes_read": (
                (staging / "observation-prefix-512m.deflate").stat().st_size
                if (staging / "observation-prefix-512m.deflate").is_file()
                else 0
            ),
            "next_action": "stop with authored-clip fallback; do not retry, grow the prefix, run physics or open Planter",
        }
    else:
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "status": "Validated",
            "decision": "CeramicCupObservationInputsAcquired",
            "claim": "TWO_METADATA_BLOCKS_AND_ONE_FROZEN_AUDIO_PREFIX_ONLY / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
            **common_report(manifest_bytes, runner_sha256),
            "discovery": discovered,
            "responses": responses,
            "network_requests": len(responses),
            "metadata_payload_bytes_read": (
                SPATIAL_RANGE[1] - SPATIAL_RANGE[0] + 1
                + CONDITION_RANGE[1]
                - CONDITION_RANGE[0]
                + 1
            ),
            "audio_payload_bytes_read": PREFIX_BYTES,
            "additional_requests_allowed": 0,
            "next_action": "decode only metadata arrays and impact-zero rows 0 through 599 from this immutable cache",
        }
    try:
        (staging / "report.json").write_bytes(canonical_json(report))
        if output.exists():
            output.rmdir()
        staging.rename(output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def decode_local_record(block: bytes, block_start: int, expected: dict[str, Any]) -> bytes:
    offset = expected["local_offset"] - block_start
    if offset < 0 or offset + 30 > len(block):
        raise ObservationError(f"local header is outside acquired block: {expected['name']}")
    values = struct.unpack_from("<4s5H3L2H", block, offset)
    signature, flags, method = values[0], values[2], values[3]
    crc32, compressed, uncompressed = values[6], values[7], values[8]
    name_bytes, extra_bytes = values[9], values[10]
    name_start = offset + 30
    data_start = name_start + name_bytes + extra_bytes
    data_end = data_start + compressed
    if data_end > len(block):
        raise ObservationError(f"compressed member is outside acquired block: {expected['name']}")
    try:
        name = block[name_start : name_start + name_bytes].decode("utf-8")
    except UnicodeDecodeError as error:
        raise ObservationError("local member name is not UTF-8") from error
    if (
        signature != b"PK\x03\x04"
        or flags != 0
        or method != expected["method"]
        or f"{crc32:08x}" != expected["crc32"]
        or compressed != expected["compressed_bytes"]
        or uncompressed != expected["uncompressed_bytes"]
        or name != expected["name"]
    ):
        raise ObservationError(f"local member contract changed: {expected['name']}")
    try:
        payload = zlib.decompress(block[data_start:data_end], -15)
    except zlib.error as error:
        raise ObservationError(f"decompress metadata member {name}: {error}") from error
    if len(payload) != uncompressed or f"{zlib.crc32(payload) & 0xffffffff:08x}" != expected["crc32"]:
        raise ObservationError(f"metadata member integrity changed: {name}")
    return payload


def load_npy(payload: bytes, shape: tuple[int, ...], dtype: str, label: str) -> np.ndarray:
    stream = io.BytesIO(payload)
    try:
        array = np.load(stream, allow_pickle=False)
    except (ValueError, OSError) as error:
        raise ObservationError(f"decode {label}: {error}") from error
    if stream.tell() != len(payload) or array.shape != shape or array.dtype.str != dtype:
        raise ObservationError(
            f"{label} NPY contract changed: shape={array.shape}, dtype={array.dtype.str}, consumed={stream.tell()}"
        )
    if not np.all(np.isfinite(array)):
        raise ObservationError(f"{label} contains non-finite values")
    return array


def validate_metadata(arrays: dict[str, np.ndarray]) -> dict[str, Any]:
    rows = slice(0, ROW_COUNT)
    vertex_ids = arrays["vertexID.npy"][rows]
    vertex_xyz = arrays["vertexXYZ.npy"][rows]
    angles = arrays["angle.npy"][rows]
    distances = arrays["distance.npy"][rows]
    microphones = arrays["micID.npy"][rows]
    listener_xyz = arrays["listenerXYZ.npy"][rows]
    if (
        len(np.unique(vertex_ids)) != 1
        or not np.all(vertex_xyz == vertex_xyz[0])
        or len(np.unique(angles)) != 10
        or len(np.unique(distances)) != 4
        or len(np.unique(np.stack([angles, distances], axis=1), axis=0)) != 40
    ):
        raise ObservationError("impact-zero metadata product changed")
    expected_microphones = np.tile(np.arange(15, dtype=np.int64), 40)
    if not np.array_equal(microphones, expected_microphones):
        raise ObservationError("impact-zero microphone row order changed")
    for start in range(0, ROW_COUNT, 15):
        if (
            not np.all(angles[start : start + 15] == angles[start])
            or not np.all(distances[start : start + 15] == distances[start])
        ):
            raise ObservationError("impact-zero angle/distance group order changed")
    if microphones[REFERENCE_ROW] != REFERENCE_ROW:
        raise ObservationError("frozen reference row is not microphone 7")
    return {
        "row_count": ROW_COUNT,
        "vertex_id": int(vertex_ids[0]),
        "vertex_xyz_metres": [float(value) for value in vertex_xyz[0]],
        "angle_values_degrees": [int(value) for value in np.unique(angles)],
        "distance_values_millimetres": [int(value) for value in np.unique(distances)],
        "microphone_values": [int(value) for value in np.unique(microphones)],
        "condition_pair_count": 40,
        "listener_position_count": len(np.unique(listener_xyz, axis=0)),
        "reference_row": {
            "row": REFERENCE_ROW,
            "angle_degrees": int(angles[REFERENCE_ROW]),
            "distance_millimetres": int(distances[REFERENCE_ROW]),
            "microphone_id": int(microphones[REFERENCE_ROW]),
            "listener_xyz_metres": [float(value) for value in listener_xyz[REFERENCE_ROW]],
        },
    }


def validate_npy_header(header: bytes) -> None:
    if (
        len(header) != NPY_HEADER_BYTES
        or header[:6] != b"\x93NUMPY"
        or header[6:8] != b"\x01\x00"
        or int.from_bytes(header[8:10], "little") != 118
    ):
        raise ObservationError("Ceramic Cup audio NPY header identity changed")
    text = header[10:].decode("latin1")
    if (
        "'descr': '<f4'" not in text
        or "'fortran_order': False" not in text
        or f"'shape': ({TOTAL_ROWS}, {SAMPLE_COUNT})" not in text
        or not text.endswith("\n")
    ):
        raise ObservationError(f"Ceramic Cup audio NPY descriptor changed: {text!r}")


def decode_audio_prefix(prefix: Path, target: Path) -> tuple[str, str]:
    decoder = zlib.decompressobj(-15)
    header = bytearray()
    digest = hashlib.sha256()
    decoded = 0
    target_total = NPY_HEADER_BYTES + DECODED_BYTES
    with prefix.open("rb") as source, target.open("wb") as output:
        while len(header) + decoded < target_total:
            compressed = source.read(1024 * 1024)
            if not compressed:
                break
            pending = compressed
            while pending and len(header) + decoded < target_total:
                remaining = target_total - len(header) - decoded
                block = decoder.decompress(pending, remaining)
                pending = decoder.unconsumed_tail
                if len(header) < NPY_HEADER_BYTES:
                    take = min(NPY_HEADER_BYTES - len(header), len(block))
                    header.extend(block[:take])
                    block = block[take:]
                if block:
                    digest.update(block)
                    output.write(block)
                    decoded += len(block)
                if not pending:
                    break
    validate_npy_header(bytes(header))
    if decoded != DECODED_BYTES:
        raise ObservationError(
            f"frozen prefix decoded {decoded} row bytes, expected {DECODED_BYTES}"
        )
    return sha256_bytes(bytes(header)), digest.hexdigest()


def validate_acquisition(
    manifest_bytes: bytes, acquisition: Path
) -> tuple[bytes, dict[str, Any], dict[str, Path]]:
    report_bytes = (acquisition / "report.json").read_bytes()
    report = json.loads(report_bytes)
    paths = {
        "spatial": acquisition / "spatial-metadata-records.bin",
        "condition": acquisition / "condition-metadata-records.bin",
        "audio": acquisition / "observation-prefix-512m.deflate",
    }
    if (
        report.get("schema") != REPORT_SCHEMAS["acquire"]
        or report.get("decision") != "CeramicCupObservationInputsAcquired"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("network_requests") != 3
        or report.get("audio_payload_bytes_read") != PREFIX_BYTES
        or report.get("planter_payload_bytes_read") != 0
        or any(not path.is_file() for path in paths.values())
        or paths["spatial"].stat().st_size != SPATIAL_RANGE[1] - SPATIAL_RANGE[0] + 1
        or paths["condition"].stat().st_size != CONDITION_RANGE[1] - CONDITION_RANGE[0] + 1
        or paths["audio"].stat().st_size != PREFIX_BYTES
    ):
        raise ObservationError("observation acquisition lineage changed")
    response_hashes = {item["artifact"]: item["sha256"] for item in report["responses"]}
    if any(sha256_file(path) != response_hashes.get(path.name) for path in paths.values()):
        raise ObservationError("observation acquisition artifact identity changed")
    return report_bytes, report, paths


def decode(
    manifest_bytes: bytes, runner_sha256: str, acquisition: Path, output: Path
) -> dict[str, Any]:
    acquisition_bytes, acquisition_report, paths = validate_acquisition(manifest_bytes, acquisition)
    spatial = paths["spatial"].read_bytes()
    condition = paths["condition"].read_bytes()
    arrays = {
        "vertexXYZ.npy": load_npy(
            decode_local_record(spatial, SPATIAL_RANGE[0], ENTRIES["vertexXYZ.npy"]),
            (TOTAL_ROWS, 3),
            "<f8",
            "vertexXYZ",
        ),
        "listenerXYZ.npy": load_npy(
            decode_local_record(spatial, SPATIAL_RANGE[0], ENTRIES["listenerXYZ.npy"]),
            (TOTAL_ROWS, 3),
            "<f8",
            "listenerXYZ",
        ),
    }
    for name in ["angle.npy", "distance.npy", "micID.npy", "vertexID.npy"]:
        arrays[name] = load_npy(
            decode_local_record(condition, CONDITION_RANGE[0], ENTRIES[name]),
            (TOTAL_ROWS,),
            "<i8",
            name.removesuffix(".npy"),
        )
    metadata = validate_metadata(arrays)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    decoded_path = staging / "ceramic-cup-impact000-rows000-599.f32le"
    try:
        npy_header_sha256, decoded_sha256 = decode_audio_prefix(paths["audio"], decoded_path)
        report = {
            "schema": REPORT_SCHEMAS["decode"],
            "status": "Validated",
            "decision": "CeramicCupImpactZeroObservationDecoded",
            "claim": "METADATA_AND_IMPACT_ZERO_ROWS_ONLY / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
            **common_report(manifest_bytes, runner_sha256),
            "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
            "acquisition_audio_prefix_sha256": acquisition_report["responses"][2]["sha256"],
            "metadata": metadata,
            "npy_header_sha256": npy_header_sha256,
            "decoded_path": decoded_path.name,
            "decoded_sha256": decoded_sha256,
            "decoded_bytes": DECODED_BYTES,
            "row_count": ROW_COUNT,
            "sample_count": SAMPLE_COUNT,
            "network_requests": 0,
            "additional_audio_payload_bytes_read": 0,
            "next_action": "run the unchanged V2 observation admission twice from this immutable decoded block",
        }
        report_bytes = canonical_json(report)
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def validate_decode(
    manifest_bytes: bytes, decode_dir: Path
) -> tuple[bytes, dict[str, Any], Path]:
    report_bytes = (decode_dir / "report.json").read_bytes()
    report = json.loads(report_bytes)
    decoded_path = decode_dir / report.get("decoded_path", "")
    if (
        report.get("schema") != REPORT_SCHEMAS["decode"]
        or report.get("decision") != "CeramicCupImpactZeroObservationDecoded"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("decoded_bytes") != DECODED_BYTES
        or report.get("row_count") != ROW_COUNT
        or report.get("sample_count") != SAMPLE_COUNT
        or report.get("network_requests") != 0
        or report.get("planter_payload_bytes_read") != 0
        or not decoded_path.is_file()
        or decoded_path.stat().st_size != DECODED_BYTES
        or sha256_file(decoded_path) != report.get("decoded_sha256")
    ):
        raise ObservationError("decoded observation lineage changed")
    return report_bytes, report, decoded_path


def gate_check(metric: str, observed: float, relation: str, threshold: float) -> dict[str, Any]:
    passed = observed >= threshold if relation == ">=" else observed <= threshold
    return {
        "metric": metric,
        "observed": observed,
        "relation": relation,
        "threshold": threshold,
        "passed": passed,
    }


def analyze(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    decode_dir: Path,
) -> tuple[dict[str, Any], dict[str, bytes]]:
    decode_bytes, decode_report, decoded_path = validate_decode(manifest_bytes, decode_dir)
    validate_sources(root, manifest)
    fixture = fixture_directory(base, manifest)
    fixture_report, parity = extractor.validate_fixture(fixture)
    rows = np.memmap(decoded_path, mode="r", dtype="<f4", shape=(ROW_COUNT, SAMPLE_COUNT))
    samples = np.asarray(rows[REFERENCE_ROW], dtype=np.float64)
    extraction = extractor.analyze_v2(samples)
    checks = [
        gate_check("selected_mode_count", extraction["selected_mode_count"], ">=", THRESHOLDS["minimum_selected_modes"]),
        gate_check("persistent_mode_recall", extraction["persistent_mode_recall"], ">=", THRESHOLDS["minimum_persistent_mode_recall"]),
        gate_check("median_frequency_error_cents", extraction["median_frequency_error_cents"], "<=", THRESHOLDS["maximum_median_frequency_error_cents"]),
        gate_check("decaying_mode_fraction", extraction["decaying_mode_fraction"], ">=", THRESHOLDS["minimum_decaying_mode_fraction"]),
        gate_check("median_tail_prediction_rmse_db", extraction["median_tail_prediction_rmse_db"], "<=", THRESHOLDS["maximum_median_tail_prediction_rmse_db"]),
    ]
    passed = all(check["passed"] for check in checks)
    report = {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": "CeramicCupObservationAdmitted" if passed else "CeramicCupObservationRejected",
        "claim": "UNCHANGED_V2_OBSERVATION_ADMISSION_ONLY / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "decode_report_sha256": sha256_bytes(decode_bytes),
        "decoded_sha256": decode_report["decoded_sha256"],
        "reference_condition": decode_report["metadata"]["reference_row"],
        "extractor_profile_id": manifest["extractor"]["profile_id"],
        "extractor_fixture_report_sha256": FIXTURE_REPORT_SHA256,
        "extractor_fixture_parity": parity,
        "observation_analysis": extraction,
        "observation_gate": {"passed": passed, "checks": checks},
        "diagnostics": {
            "selected_modes_below_500_hz": sum(
                mode["frequency_hz"] < 500.0 for mode in extraction["modes"]
            ),
            "selected_modes_at_or_above_500_hz": sum(
                mode["frequency_hz"] >= 500.0 for mode in extraction["modes"]
            ),
        },
        "network_requests": 0,
        "additional_audio_payload_bytes_read": 0,
        "next_action": (
            "freeze a vector shell/hollow-volume mechanics discriminator without changing this observation"
            if passed
            else "stop this object with authored-clip fallback; do not run physics, tune the extractor or open Planter"
        ),
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
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    if arguments.stage == "preflight":
        if arguments.input is not None:
            raise ObservationError("preflight does not accept --input")
        report, artifacts = preflight(root, base, manifest_bytes, manifest, runner_sha256)
        report_bytes = publish(output, report, artifacts)
    elif arguments.stage == "acquire":
        if arguments.input is not None:
            raise ObservationError("acquire does not accept --input")
        report = acquire(base, manifest_bytes, manifest, runner_sha256, output)
        report_bytes = (output / "report.json").read_bytes()
    elif arguments.stage == "decode":
        if arguments.input is None:
            raise ObservationError("decode requires --input acquisition directory")
        input_dir = external_directory(root, arguments.input, "acquisition input")
        report = decode(manifest_bytes, runner_sha256, input_dir, output)
        report_bytes = (output / "report.json").read_bytes()
    else:
        if arguments.input is None:
            raise ObservationError("analyze requires --input decode directory")
        input_dir = external_directory(root, arguments.input, "decode input")
        report, artifacts = analyze(
            root, base, manifest_bytes, manifest, runner_sha256, input_dir
        )
        report_bytes = publish(output, report, artifacts)
    print(f"REALIMPACT Ceramic Cup observation {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print(f"audio payload bytes read: {report.get('audio_payload_bytes_read', 0)}")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
