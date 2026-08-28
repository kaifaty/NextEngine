#!/usr/bin/env python3
"""Execute the frozen independent REALIMPACT one-impact observation protocol."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import tempfile
from typing import Any
import zlib

import numpy as np
from scipy import signal
import scipy

import physical_sound_adaptive_decay_control as adaptive
import physical_sound_multioutput_decay_control as multioutput
import physical_sound_realimpact_independent_observation_discovery as independent
import physical_sound_realimpact_independent_observation_protocol as protocol
import physical_sound_realimpact_observation_execute as legacy
import physical_sound_realimpact_pitcher_calibration as single
import physical_sound_salience_selector_control as salience


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-independent-observation-execution.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-independent-observation-execution-"
        "preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-independent-observation-acquisition."
        "report.v1"
    ),
    "decode": (
        "nextengine.experimental-realimpact-independent-observation-decode.report.v1"
    ),
    "analyze": (
        "nextengine.experimental-realimpact-independent-observation-analysis.report.v1"
    ),
}
STUDY_ID = protocol.STUDY_ID
REVISION = "iron-skillet-one-impact-salience-execution-v1"
OBJECT_ID = protocol.OBJECT_ID

PROTOCOL_MANIFEST_SHA256 = (
    "573ff0d6f50619aebe222314b9161d2f6ab066dc2720f5b0aa1f1870ec168c93"
)
PROTOCOL_PREFLIGHT_SHA256 = (
    "19c1f57aadbbc104cda8d0c665f9b0047ef947bb2b9a07006d7a454ae57b2680"
)
PROTOCOL_RUNNER_SHA256 = (
    "7385241d75abe219430267ca21fe82ab15b6706b3368b67cb15127f6c7838273"
)

ACQUISITION_ARTIFACTS = [
    ("early-condition-records.bin", protocol.EARLY_CONDITION_RANGE, "metadata"),
    ("distance-record.bin", protocol.DISTANCE_RANGE, "metadata"),
    ("vertex-id-record.bin", protocol.VERTEX_ID_RANGE, "metadata"),
    ("observation-prefix-512m.deflate", protocol.AUDIO_PREFIX_RANGE, "audio"),
]
DECODED_NAME = "iron-skillet-impact000-rows000-599.f32le"


class ExecutionError(RuntimeError):
    """The independent execution contract or a bounded invariant failed."""


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


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "protocol": {
            "runner": {
                "path": (
                    "lab/scripts/"
                    "physical_sound_realimpact_independent_observation_protocol.py"
                ),
                "sha256": PROTOCOL_RUNNER_SHA256,
            },
            "manifest": {
                "path": "observation-protocol-manifest.json",
                "sha256": PROTOCOL_MANIFEST_SHA256,
            },
            "preflight": {
                "path": "observation-protocol-preflight-a/report.json",
                "sha256": PROTOCOL_PREFLIGHT_SHA256,
            },
        },
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "development",
            "impact_ordinal": 0,
            "rows": [0, protocol.ROW_COUNT - 1],
            "analysis_rows": [0, 14],
            "reference_row": protocol.REFERENCE_ROW,
            "prior_observation_payload_bytes_read": 0,
            "planter_payload_access_allowed": False,
        },
        "archive": {
            "url": independent.ARCHIVE_URL,
            "bytes": independent.ARCHIVE_BYTES,
            "etag": independent.ARCHIVE_ETAG,
            "last_modified_http": independent.ARCHIVE_LAST_MODIFIED,
        },
        "entries": protocol.ENTRIES,
        "requests": {
            "maximum_network_requests": 4,
            "artifacts": [
                {"name": name, "range": value, "role": role}
                for name, value, role in ACQUISITION_ARTIFACTS
            ],
            "audio_prefix_bytes": protocol.PREFIX_BYTES,
            "metadata_bytes": 1_360,
            "prefix_growth_allowed": False,
            "retry_allowed": False,
        },
        "decode": {
            "audio_dtype": "<f4",
            "audio_shape": [protocol.TOTAL_ROWS, protocol.SAMPLE_COUNT],
            "npy_header_bytes": protocol.NPY_HEADER_BYTES,
            "decoded_rows": protocol.ROW_COUNT,
            "decoded_row_bytes": protocol.ROW_BYTES,
            "decoded_bytes": protocol.DECODED_BYTES,
            "metadata_dtype": "<i8",
            "metadata_shape": [protocol.TOTAL_ROWS],
        },
        "analysis": {
            "sample_rate_hz": protocol.SAMPLE_RATE_HZ,
            "onset": {
                "rule": "first spatial norm sample at or above 5 percent of maximum",
                "minimum_peak_fraction": single.ONSET_PEAK_FRACTION,
            },
            "salience_revision": salience.REVISION,
            "adaptive_revision": adaptive.REVISION,
            "diagnostic_comparator": "spatial-modal-power-15-v1-top16-fixed-separation",
            "scale_factors": salience.SCALE_FACTORS,
            "gates": protocol.GATES,
        },
        "sources": [
            {
                "path": "lab/scripts/physical_sound_salience_selector_control.py",
                "sha256": protocol.SALIENCE_RUNNER_SHA256,
            },
            {
                "path": "lab/scripts/physical_sound_adaptive_decay_control.py",
                "sha256": protocol.ADAPTIVE_RUNNER_SHA256,
            },
            {
                "path": "lab/scripts/physical_sound_multioutput_decay_control.py",
                "sha256": protocol.MULTIOUTPUT_RUNNER_SHA256,
            },
            {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": protocol.EXTRACTOR_RUNNER_SHA256,
            },
            {
                "path": "lab/scripts/physical_sound_realimpact_observation_execute.py",
                "sha256": (
                    "bad26592744ce90d7d7bf54382924be100b2a1e438c95844adaa5681759360e2"
                ),
            },
        ],
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "condition_metadata_allowed": True,
            "one_impact_observation_rows_allowed": True,
            "additional_audio_or_planter_payload_allowed": False,
            "physics_solver_allowed": False,
            "threshold_or_selector_tuning_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "commit a repeated zero-access execution preflight before acquire; then stop "
            "with authored-clip fallback if acquisition, decode, row identity, repeat, "
            "scale invariance or any unchanged gate fails; no retry, prefix growth, "
            "object substitution, physics or Planter access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ExecutionError("independent execution manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ExecutionError(f"parse independent execution manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ExecutionError("independent execution manifest changed")
    return data, manifest


def resolve_reference(
    base: Path, ref: dict[str, Any], label: str
) -> tuple[bytes, Path]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise ExecutionError(f"{label} escapes physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise ExecutionError(f"{label} identity changed")
    return data, path


def validate_protocol(root: Path, base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    refs = manifest["protocol"]
    runner_path = (root / refs["runner"]["path"]).resolve(strict=True)
    if (
        not runner_path.is_file()
        or not runner_path.is_relative_to(root)
        or sha256_file(runner_path) != refs["runner"]["sha256"]
    ):
        raise ExecutionError("protocol runner identity changed")
    manifest_bytes, _ = resolve_reference(base, refs["manifest"], "protocol manifest")
    preflight_bytes, _ = resolve_reference(base, refs["preflight"], "protocol preflight")
    preflight = json.loads(preflight_bytes)
    if (
        preflight.get("decision") != "IndependentOneImpactObservationProtocolFrozen"
        or preflight.get("runner_sha256") != PROTOCOL_RUNNER_SHA256
        or preflight.get("manifest_sha256") != PROTOCOL_MANIFEST_SHA256
        or preflight.get("network_requests") != 0
        or preflight.get("object_payload_bytes_read") != 0
        or preflight.get("planter_payload_bytes_read") != 0
    ):
        raise ExecutionError("protocol preflight lineage changed")
    return {
        "runner_sha256": refs["runner"]["sha256"],
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "preflight_sha256": sha256_bytes(preflight_bytes),
        "decision": preflight["decision"],
    }


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    result = []
    for ref in manifest["sources"]:
        path = (root / ref["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != ref["sha256"]
        ):
            raise ExecutionError(f"bound source changed: {ref['path']}")
        result.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return result


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "role": "development",
        "impact_ordinal": 0,
        "rows": [0, protocol.ROW_COUNT - 1],
        "analysis_rows": [0, 14],
        "reference_row": protocol.REFERENCE_ROW,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def publish_small(output: Path, report: dict[str, Any]) -> bytes:
    report_bytes = canonical_json(report)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report_bytes


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "IndependentObservationExecutionPreflightSupported",
        "claim": (
            "EXECUTION_RUNNER_AND_PROTOCOL_PREFLIGHT / ZERO_NEW_NETWORK_OR_MEMBER_"
            "PAYLOAD_ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "protocol": validate_protocol(root, base, manifest),
        "bound_sources": validate_sources(root, manifest),
        "frozen_requests": manifest["requests"],
        "frozen_decode": manifest["decode"],
        "frozen_analysis": manifest["analysis"],
        "runtime": manifest["runtime"],
        "network_requests": 0,
        "object_payload_bytes_read": 0,
        "next_action": (
            "commit this repeated preflight, then execute the exact four-request "
            "acquisition once"
        ),
    }


def acquire(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    output: Path,
) -> dict[str, Any]:
    lineage = validate_protocol(root, base, manifest)
    validate_sources(root, manifest)
    independent.configure_core()
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    responses = []
    attempted = 0
    try:
        for name, (start, end), role in ACQUISITION_ARTIFACTS:
            attempted += 1
            response = legacy.download_range(start, end, staging / name)
            responses.append({"artifact": name, "role": role, **response})
    except Exception as error:
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "status": "Validated",
            "decision": "IndependentObservationAcquisitionRejected",
            "claim": (
                "IMMUTABLE_BOUNDED_ACQUISITION_FAILURE / NO_RETRY_OR_PREFIX_GROWTH / "
                "NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
            ),
            **common_report(manifest_bytes, runner_sha256),
            "protocol": lineage,
            "network_requests_attempted": attempted,
            "completed_responses": responses,
            "failure": f"{type(error).__name__}: {error}",
            "additional_requests_allowed": 0,
            "audio_payload_bytes_read": (
                (staging / "observation-prefix-512m.deflate").stat().st_size
                if (staging / "observation-prefix-512m.deflate").is_file()
                else 0
            ),
            "next_action": (
                "stop with authored-clip fallback; do not retry, grow the prefix, "
                "substitute an object, run physics or access Planter"
            ),
        }
    else:
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "status": "Validated",
            "decision": "IndependentObservationInputsAcquired",
            "claim": (
                "THREE_CONDITION_BLOCKS_AND_ONE_FROZEN_AUDIO_PREFIX_ONLY / "
                "NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
            ),
            **common_report(manifest_bytes, runner_sha256),
            "protocol": lineage,
            "responses": responses,
            "network_requests": len(responses),
            "metadata_payload_bytes_read": 1_360,
            "audio_payload_bytes_read": protocol.PREFIX_BYTES,
            "additional_requests_allowed": 0,
            "next_action": (
                "decode only four condition arrays and impact-zero rows 0 through 599"
            ),
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


def validate_acquisition(
    manifest_bytes: bytes, acquisition: Path
) -> tuple[bytes, dict[str, Any], dict[str, Path]]:
    report_bytes = (acquisition / "report.json").read_bytes()
    report = json.loads(report_bytes)
    paths = {name: acquisition / name for name, _, _ in ACQUISITION_ARTIFACTS}
    if (
        report.get("schema") != REPORT_SCHEMAS["acquire"]
        or report.get("decision") != "IndependentObservationInputsAcquired"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("network_requests") != 4
        or report.get("audio_payload_bytes_read") != protocol.PREFIX_BYTES
        or report.get("planter_payload_bytes_read") != 0
        or any(not path.is_file() for path in paths.values())
    ):
        raise ExecutionError("independent acquisition lineage changed")
    responses = {item["artifact"]: item for item in report["responses"]}
    for name, (start, end), _ in ACQUISITION_ARTIFACTS:
        path = paths[name]
        expected_bytes = end - start + 1
        response = responses.get(name, {})
        if (
            path.stat().st_size != expected_bytes
            or response.get("start") != start
            or response.get("end") != end
            or response.get("bytes") != expected_bytes
            or response.get("sha256") != sha256_file(path)
        ):
            raise ExecutionError(f"acquired artifact changed: {name}")
    return report_bytes, report, paths


def validate_metadata(arrays: dict[str, np.ndarray]) -> dict[str, Any]:
    rows = slice(0, protocol.ROW_COUNT)
    vertex_ids = arrays["vertexID.npy"][rows]
    angles = arrays["angle.npy"][rows]
    distances = arrays["distance.npy"][rows]
    microphones = arrays["micID.npy"][rows]
    pairs = np.stack([angles, distances], axis=1)
    expected_microphones = np.tile(np.arange(15, dtype=np.int64), 40)
    if (
        len(np.unique(vertex_ids)) != 1
        or len(np.unique(angles)) != 10
        or len(np.unique(distances)) != 4
        or len(np.unique(pairs, axis=0)) != 40
        or not np.array_equal(microphones, expected_microphones)
    ):
        raise ExecutionError("impact-zero condition metadata changed")
    for start in range(0, protocol.ROW_COUNT, 15):
        if (
            not np.all(angles[start : start + 15] == angles[start])
            or not np.all(distances[start : start + 15] == distances[start])
        ):
            raise ExecutionError("impact-zero condition grouping changed")
    if microphones[protocol.REFERENCE_ROW] != protocol.REFERENCE_ROW:
        raise ExecutionError("reference row is not microphone 7")
    return {
        "row_count": protocol.ROW_COUNT,
        "vertex_id": int(vertex_ids[0]),
        "angle_values_degrees": [int(value) for value in np.unique(angles)],
        "distance_values_millimetres": [int(value) for value in np.unique(distances)],
        "microphone_values": [int(value) for value in np.unique(microphones)],
        "condition_pair_count": 40,
        "reference_row": {
            "row": protocol.REFERENCE_ROW,
            "angle_degrees": int(angles[protocol.REFERENCE_ROW]),
            "distance_millimetres": int(distances[protocol.REFERENCE_ROW]),
            "microphone_id": int(microphones[protocol.REFERENCE_ROW]),
        },
    }


def validate_audio_header(header: bytes) -> None:
    if (
        len(header) != protocol.NPY_HEADER_BYTES
        or header[:6] != b"\x93NUMPY"
        or header[6:8] != b"\x01\x00"
        or int.from_bytes(header[8:10], "little") != 118
    ):
        raise ExecutionError("Iron Skillet audio NPY header changed")
    text = header[10:].decode("latin1")
    if (
        "'descr': '<f4'" not in text
        or "'fortran_order': False" not in text
        or f"'shape': ({protocol.TOTAL_ROWS}, {protocol.SAMPLE_COUNT})" not in text
        or not text.endswith("\n")
    ):
        raise ExecutionError(f"Iron Skillet audio NPY descriptor changed: {text!r}")


def decode_audio_prefix(prefix: Path, target: Path) -> tuple[str, str]:
    decoder = zlib.decompressobj(-15)
    header = bytearray()
    digest = hashlib.sha256()
    decoded = 0
    target_total = protocol.NPY_HEADER_BYTES + protocol.DECODED_BYTES
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
                if len(header) < protocol.NPY_HEADER_BYTES:
                    take = min(protocol.NPY_HEADER_BYTES - len(header), len(block))
                    header.extend(block[:take])
                    block = block[take:]
                if block:
                    digest.update(block)
                    output.write(block)
                    decoded += len(block)
                if not pending:
                    break
    validate_audio_header(bytes(header))
    if decoded != protocol.DECODED_BYTES:
        raise ExecutionError(
            f"frozen prefix decoded {decoded} row bytes, expected {protocol.DECODED_BYTES}"
        )
    return sha256_bytes(bytes(header)), digest.hexdigest()


def decode(
    manifest_bytes: bytes,
    runner_sha256: str,
    acquisition: Path,
    output: Path,
) -> dict[str, Any]:
    acquisition_bytes, acquisition_report, paths = validate_acquisition(
        manifest_bytes, acquisition
    )
    early = paths["early-condition-records.bin"].read_bytes()
    distance = paths["distance-record.bin"].read_bytes()
    vertex = paths["vertex-id-record.bin"].read_bytes()
    arrays = {
        "angle.npy": legacy.load_npy(
            legacy.decode_local_record(
                early, protocol.EARLY_CONDITION_RANGE[0], protocol.ENTRIES["angle.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "angle",
        ),
        "micID.npy": legacy.load_npy(
            legacy.decode_local_record(
                early, protocol.EARLY_CONDITION_RANGE[0], protocol.ENTRIES["micID.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "micID",
        ),
        "distance.npy": legacy.load_npy(
            legacy.decode_local_record(
                distance, protocol.DISTANCE_RANGE[0], protocol.ENTRIES["distance.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "distance",
        ),
        "vertexID.npy": legacy.load_npy(
            legacy.decode_local_record(
                vertex, protocol.VERTEX_ID_RANGE[0], protocol.ENTRIES["vertexID.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "vertexID",
        ),
    }
    metadata = validate_metadata(arrays)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    decoded_path = staging / DECODED_NAME
    try:
        header_sha256, decoded_sha256 = decode_audio_prefix(
            paths["observation-prefix-512m.deflate"], decoded_path
        )
        report = {
            "schema": REPORT_SCHEMAS["decode"],
            "status": "Validated",
            "decision": "IndependentImpactZeroObservationDecoded",
            "claim": (
                "CONDITION_METADATA_AND_IMPACT_ZERO_ROWS_ONLY / "
                "NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
            ),
            **common_report(manifest_bytes, runner_sha256),
            "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
            "acquisition_audio_prefix_sha256": acquisition_report["responses"][3][
                "sha256"
            ],
            "metadata": metadata,
            "npy_header_sha256": header_sha256,
            "decoded_path": decoded_path.name,
            "decoded_sha256": decoded_sha256,
            "decoded_bytes": protocol.DECODED_BYTES,
            "row_count": protocol.ROW_COUNT,
            "sample_count": protocol.SAMPLE_COUNT,
            "network_requests": 0,
            "additional_audio_payload_bytes_read": 0,
            "next_action": (
                "run the unchanged source-derived observation analysis twice"
            ),
        }
        (staging / "report.json").write_bytes(canonical_json(report))
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
        or report.get("decision") != "IndependentImpactZeroObservationDecoded"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("decoded_bytes") != protocol.DECODED_BYTES
        or report.get("row_count") != protocol.ROW_COUNT
        or report.get("sample_count") != protocol.SAMPLE_COUNT
        or report.get("additional_audio_payload_bytes_read") != 0
        or report.get("planter_payload_bytes_read") != 0
        or not decoded_path.is_file()
        or decoded_path.stat().st_size != protocol.DECODED_BYTES
        or sha256_file(decoded_path) != report.get("decoded_sha256")
    ):
        raise ExecutionError("independent decode lineage changed")
    return report_bytes, report, decoded_path


def first_below(values: np.ndarray, start: int, threshold: float) -> int | None:
    indices = np.flatnonzero(values[start:] < threshold)
    return start + int(indices[0]) if len(indices) else None


def adaptive_fit(channels: np.ndarray, peak: single.SpectralPeak) -> dict[str, Any]:
    sample_count = channels.shape[1]
    low = (peak.frequency_hz - adaptive.FILTER_WIDTH_HZ / 2.0) / (
        protocol.SAMPLE_RATE_HZ / 2.0
    )
    high = (peak.frequency_hz + adaptive.FILTER_WIDTH_HZ / 2.0) / (
        protocol.SAMPLE_RATE_HZ / 2.0
    )
    highpass_b, highpass_a = signal.butter(adaptive.FILTER_ORDER, low, btype="highpass")
    lowpass_b, lowpass_a = signal.butter(adaptive.FILTER_ORDER, high, btype="lowpass")
    power = np.zeros(sample_count, dtype=np.float64)
    for channel in channels:
        filtered = signal.lfilter(lowpass_b, lowpass_a, channel)
        filtered = signal.lfilter(highpass_b, highpass_a, filtered)
        power += filtered * filtered
    pole = math.exp(
        -1.0 / (adaptive.ENVELOPE_ETA_SECONDS * protocol.SAMPLE_RATE_HZ)
    )
    smoothed = signal.lfilter([1.0 - pole], [1.0, -pole], power)
    envelope = np.sqrt(np.maximum(smoothed, single.SAMPLE_EPSILON))
    envelope /= max(float(np.max(envelope)), single.SAMPLE_EPSILON)
    levels = 20.0 * np.log10(np.maximum(envelope, single.SAMPLE_EPSILON))
    peak_sample = int(np.argmax(levels))
    tail_start = int(sample_count * (1.0 - adaptive.NOISE_FLOOR_TAIL_FRACTION))
    noise_floor_db = float(np.max(levels[tail_start:]))
    dynamic_range_db = float(levels[peak_sample] - noise_floor_db)
    start_threshold = noise_floor_db + dynamic_range_db * adaptive.START_DYNAMIC_FRACTION
    end_threshold = noise_floor_db + dynamic_range_db * adaptive.END_DYNAMIC_FRACTION
    fit_start = first_below(levels, peak_sample, start_threshold)
    fit_end = first_below(levels, fit_start or peak_sample, end_threshold)
    minimum_samples = round(adaptive.MINIMUM_FIT_SECONDS * protocol.SAMPLE_RATE_HZ)
    valid = (
        dynamic_range_db >= adaptive.MINIMUM_DYNAMIC_RANGE_DB
        and fit_start is not None
        and fit_end is not None
        and fit_end - fit_start >= minimum_samples
    )
    if not valid:
        return {
            "frequency_hz": peak.frequency_hz,
            "valid": False,
            "peak_sample": peak_sample,
            "noise_floor_db": noise_floor_db,
            "dynamic_range_db": dynamic_range_db,
            "fit_start_sample": fit_start,
            "fit_end_sample": fit_end,
            "fit_decay_db_per_second": None,
            "fit_r_squared": None,
        }
    assert fit_start is not None and fit_end is not None
    x = np.arange(fit_start, fit_end, dtype=np.float64) / protocol.SAMPLE_RATE_HZ
    y = levels[fit_start:fit_end]
    slope, intercept = np.polyfit(x, y, 1)
    predicted = slope * x + intercept
    residual = float(np.sum((y - predicted) ** 2))
    total = float(np.sum((y - np.mean(y)) ** 2))
    r_squared = 1.0 - residual / total if total > single.SAMPLE_EPSILON else 0.0
    return {
        "frequency_hz": peak.frequency_hz,
        "valid": True,
        "peak_sample": peak_sample,
        "noise_floor_db": noise_floor_db,
        "dynamic_range_db": dynamic_range_db,
        "fit_start_sample": fit_start,
        "fit_end_sample": fit_end,
        "fit_decay_db_per_second": float(slope),
        "fit_r_squared": r_squared,
    }


def detect_onset(channels: np.ndarray) -> int:
    spatial_norm = np.sqrt(np.sum(channels * channels, axis=0))
    peak = float(np.max(spatial_norm))
    indices = np.flatnonzero(spatial_norm >= peak * single.ONSET_PEAK_FRACTION)
    if not len(indices):
        raise ExecutionError("independent observation onset was not found")
    onset = int(indices[0])
    tail = onset + salience.TAIL_START_MS * protocol.SAMPLE_RATE_HZ // 1_000
    if tail + single.FFT_SIZE > channels.shape[1]:
        raise ExecutionError("independent observation is too short for tail selection")
    return onset


def candidate_analysis(channels: np.ndarray, onset: int) -> dict[str, Any]:
    local_onset, salient_onset = salience.source_derived_candidates(channels, onset)
    tail_start = onset + salience.TAIL_START_MS * protocol.SAMPLE_RATE_HZ // 1_000
    local_tail, salient_tail = salience.source_derived_candidates(channels, tail_start)
    persistent, assignments = salience.persistent_candidates(salient_onset, salient_tail)
    errors = [
        abs(
            single.cents_between(
                salient_onset[index]["frequency_hz"],
                salient_tail[assignment]["frequency_hz"],
            )
        )
        for index, assignment in enumerate(assignments)
        if assignment is not None
    ]
    peaks = [
        single.SpectralPeak(item["frequency_hz"], item["relative_level_db"])
        for item in persistent
    ]
    fits = [adaptive_fit(channels, peak) for peak in peaks]
    valid = [mode for mode in fits if mode["valid"]]
    selected_count = len(persistent)
    return {
        "local_onset_candidate_count": len(local_onset),
        "salient_onset_candidate_count": len(salient_onset),
        "local_tail_candidate_count": len(local_tail),
        "salient_tail_candidate_count": len(salient_tail),
        "selected_mode_count": selected_count,
        "persistent_bin_indices": [item["bin_index"] for item in persistent],
        "persistent_frequencies_hz": [item["frequency_hz"] for item in persistent],
        "onset_to_tail_assignments": assignments,
        "persistence_recall": (
            selected_count / len(salient_onset) if salient_onset else 0.0
        ),
        "median_frequency_error_cents": single.median(errors) if errors else None,
        "valid_adaptive_fit_count": len(valid),
        "valid_adaptive_fit_fraction": len(valid) / selected_count if selected_count else 0.0,
        "decaying_mode_fraction": (
            sum(
                mode["valid"] and mode["fit_decay_db_per_second"] < -1.0
                for mode in fits
            )
            / selected_count
            if selected_count
            else 0.0
        ),
        "median_fit_r_squared": (
            single.median([mode["fit_r_squared"] for mode in valid]) if valid else None
        ),
        "modes": fits,
    }


def scale_invariance(channels: np.ndarray, onset: int) -> dict[str, Any]:
    variants = []
    signatures = []
    for scale in salience.SCALE_FACTORS:
        local_onset, salient_onset = salience.source_derived_candidates(
            channels * scale, onset
        )
        _, salient_tail = salience.source_derived_candidates(
            channels * scale,
            onset + salience.TAIL_START_MS * protocol.SAMPLE_RATE_HZ // 1_000,
        )
        persistent, _ = salience.persistent_candidates(salient_onset, salient_tail)
        signature = [item["bin_index"] for item in persistent]
        signatures.append(signature)
        variants.append(
            {
                "scale": scale,
                "local_onset_candidate_count": len(local_onset),
                "persistent_bin_indices": signature,
                "signature_sha256": sha256_bytes(canonical_json(signature)),
            }
        )
    return {
        "passed": all(signature == signatures[0] for signature in signatures[1:]),
        "variants": variants,
    }


def comparator_analysis(channels: np.ndarray, onset: int) -> dict[str, Any]:
    peaks = multioutput.spatial_modal_peaks(channels, onset)
    modes = [adaptive_fit(channels, peak) for peak in peaks]
    valid = [mode for mode in modes if mode["valid"]]
    return {
        "admission_role": False,
        "selected_mode_count": len(peaks),
        "frequencies_hz": [peak.frequency_hz for peak in peaks],
        "valid_adaptive_fit_fraction": len(valid) / len(peaks),
        "decaying_mode_fraction": sum(
            mode["valid"] and mode["fit_decay_db_per_second"] < -1.0 for mode in modes
        )
        / len(peaks),
        "modes": modes,
    }


def check(name: str, observed: Any, relation: str, threshold: Any) -> dict[str, Any]:
    if relation == ">=":
        passed = observed >= threshold
    elif relation == "<=":
        passed = observed <= threshold
    else:
        passed = observed == threshold
    return {
        "name": name,
        "observed": observed,
        "relation": relation,
        "threshold": threshold,
        "passed": passed,
    }


def analyze(
    manifest_bytes: bytes,
    runner_sha256: str,
    decode_dir: Path,
) -> dict[str, Any]:
    decode_bytes, decode_report, decoded_path = validate_decode(
        manifest_bytes, decode_dir
    )
    rows = np.memmap(
        decoded_path,
        dtype="<f4",
        mode="r",
        shape=(protocol.ROW_COUNT, protocol.SAMPLE_COUNT),
    )
    channels = np.asarray(rows[:15], dtype=np.float64)
    onset = detect_onset(channels)
    candidate = candidate_analysis(channels, onset)
    scales = scale_invariance(channels, onset)
    comparator = comparator_analysis(channels, onset)
    gates = protocol.GATES
    frequency_error = (
        candidate["median_frequency_error_cents"]
        if candidate["median_frequency_error_cents"] is not None
        else 1.0e9
    )
    fit_r_squared = (
        candidate["median_fit_r_squared"]
        if candidate["median_fit_r_squared"] is not None
        else 0.0
    )
    checks = [
        check(
            "minimum_selected_modes",
            candidate["selected_mode_count"],
            ">=",
            gates["minimum_selected_modes"],
        ),
        check(
            "maximum_selected_modes",
            candidate["selected_mode_count"],
            "<=",
            gates["maximum_selected_modes"],
        ),
        check(
            "onset_to_tail_persistence_recall",
            candidate["persistence_recall"],
            ">=",
            gates["minimum_onset_to_tail_persistence_recall"],
        ),
        check(
            "median_frequency_error_cents",
            frequency_error,
            "<=",
            gates["maximum_median_frequency_error_cents"],
        ),
        check(
            "valid_adaptive_fit_fraction",
            candidate["valid_adaptive_fit_fraction"],
            ">=",
            gates["minimum_valid_adaptive_fit_fraction"],
        ),
        check(
            "decaying_mode_fraction",
            candidate["decaying_mode_fraction"],
            ">=",
            gates["minimum_decaying_mode_fraction"],
        ),
        check(
            "median_fit_r_squared",
            fit_r_squared,
            ">=",
            gates["minimum_median_fit_r_squared"],
        ),
        check(
            "scale_invariant_bin_selection",
            scales["passed"],
            "==",
            gates["scale_invariant_bin_selection"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": (
            "IndependentObservationMethodTransferSupported"
            if passed
            else "IndependentObservationMethodTransferRejected"
        ),
        "claim": (
            "INDEPENDENT_ONE_OBJECT_OBSERVATION_METHOD_TRANSFER_ONLY / "
            "NO_MATERIAL_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "decode_report_sha256": sha256_bytes(decode_bytes),
        "decoded_payload_sha256": decode_report["decoded_sha256"],
        "decoded_payload_bytes_analyzed": protocol.DECODED_BYTES,
        "additional_payload_bytes_read": 0,
        "network_requests": 0,
        "onset_sample": onset,
        "candidate_analysis": candidate,
        "scale_invariance": scales,
        "diagnostic_comparator": comparator,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze an object-disjoint calibration/holdout protocol before mechanics"
            if passed
            else "reject real transfer; do not tune, retry, substitute objects or run physics"
        ),
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = independent.discovery.external_file(root, arguments.manifest, "manifest")
    output = independent.discovery.external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    independent.configure_core()
    if arguments.stage == "preflight":
        if arguments.input is not None:
            raise ExecutionError("preflight does not accept --input")
        report = preflight(root, base, manifest_bytes, manifest, runner_sha256)
        report_bytes = publish_small(output, report)
    elif arguments.stage == "acquire":
        if arguments.input is not None:
            raise ExecutionError("acquire does not accept --input")
        report = acquire(
            root, base, manifest_bytes, manifest, runner_sha256, output
        )
        report_bytes = canonical_json(report)
    elif arguments.stage == "decode":
        if arguments.input is None:
            raise ExecutionError("decode requires --input acquisition directory")
        acquisition = independent.discovery.external_directory(
            root, arguments.input, "acquisition"
        )
        report = decode(manifest_bytes, runner_sha256, acquisition, output)
        report_bytes = canonical_json(report)
    else:
        if arguments.input is None:
            raise ExecutionError("analyze requires --input decode directory")
        decode_dir = independent.discovery.external_directory(
            root, arguments.input, "decode"
        )
        report = analyze(manifest_bytes, runner_sha256, decode_dir)
        report_bytes = publish_small(output, report)
    print(f"Independent observation execution {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print(f"audio payload bytes read: {report.get('audio_payload_bytes_read', 0)}")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
