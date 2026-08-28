#!/usr/bin/env python3
"""Execute the frozen REALIMPACT broad-band independent holdout."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
import zlib
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_broadband_common_pole_control as broadband
import physical_sound_dense_broadband_scaling_control as dense
import physical_sound_realimpact_broadband_counterfactual as v1
import physical_sound_realimpact_broadband_holdout_discovery as holdout
import physical_sound_realimpact_broadband_holdout_protocol as protocol
import physical_sound_realimpact_observation_execute as legacy
import scipy

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-broadband-holdout-execution.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-broadband-holdout-execution-"
        "preflight.report.v1"
    ),
    "acquire": (
        "nextengine.experimental-realimpact-broadband-holdout-execution-"
        "acquisition.report.v1"
    ),
    "decode": (
        "nextengine.experimental-realimpact-broadband-holdout-execution-"
        "decode.report.v1"
    ),
    "analyze": (
        "nextengine.experimental-realimpact-broadband-holdout-execution-"
        "analysis.report.v1"
    ),
}
STUDY_ID = protocol.STUDY_ID
REVISION = "iron-mortar-broadband-v2-independent-execution-v1"
OBJECT_ID = protocol.OBJECT_ID

PROTOCOL_RUNNER_SHA256 = (
    "219c62797b3ccb0494da5079f30922ea150c3ac759f8bcd24293cfc8911fae85"
)
PROTOCOL_MANIFEST_SHA256 = (
    "73c218edb57f42aa6b6ee9adae6f7ff84e6b1158e1ab47069c156b68f9caa824"
)
PROTOCOL_PREFLIGHT_SHA256 = (
    "c6943d129b8be337390ffcde1cd5aa4ad4eb1bceee83c5912ea88e063f1734fd"
)

SOURCE_HASHES = {
    "lab/scripts/physical_sound_realimpact_broadband_holdout_protocol.py": (
        PROTOCOL_RUNNER_SHA256
    ),
    "lab/scripts/physical_sound_realimpact_broadband_v2_counterfactual.py": (
        "3f1c07b501dec97645cc0f1ad2fec5b3960312eacde88a291b64cc740a056aa5"
    ),
    "lab/scripts/physical_sound_dense_broadband_scaling_control.py": (
        "8e4cc18ba25f90adb43483e5d4ff1ea7be46869e55c28ebd7c9bb5831b29fbbe"
    ),
    "lab/scripts/physical_sound_broadband_common_pole_control.py": (
        "317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d"
    ),
    "lab/scripts/physical_sound_realimpact_broadband_counterfactual.py": (
        "63d46bf476798e540cfc726a0e2bca58beaacc0f829ffff7ecc2b1083efb2e76"
    ),
    "lab/scripts/physical_sound_realimpact_observation_execute.py": (
        "bad26592744ce90d7d7bf54382924be100b2a1e438c95844adaa5681759360e2"
    ),
}

ACQUISITION_ARTIFACTS = [
    ("distance-record.bin", protocol.DISTANCE_RANGE, "metadata"),
    ("mic-vertex-records.bin", protocol.MIC_VERTEX_RANGE, "metadata"),
    ("angle-record.bin", protocol.ANGLE_RANGE, "metadata"),
    ("observation-prefix-32m.deflate", protocol.AUDIO_PREFIX_RANGE, "audio"),
]
DECODED_NAME = "iron-mortar-impact000-condition000-mics00-14.f32le"


class ExecutionError(RuntimeError):
    """The holdout execution contract or a bounded invariant failed."""


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
                    "lab/scripts/physical_sound_realimpact_broadband_holdout_"
                    "protocol.py"
                ),
                "sha256": PROTOCOL_RUNNER_SHA256,
            },
            "manifest": {
                "path": "manifest.json",
                "sha256": PROTOCOL_MANIFEST_SHA256,
            },
            "preflight": {
                "path": "protocol-preflight-a/report.json",
                "sha256": PROTOCOL_PREFLIGHT_SHA256,
            },
        },
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "independent_method_holdout",
            "impact_ordinal": 0,
            "impact_rows": [0, protocol.IMPACT_ROW_COUNT - 1],
            "analysis_rows": [0, protocol.ANALYSIS_ROW_COUNT - 1],
            "reference_row": protocol.REFERENCE_ROW,
            "prior_observation_payload_bytes_read": 0,
            "planter_payload_access_allowed": False,
        },
        "archive": {
            "url": holdout.ARCHIVE_URL,
            "bytes": holdout.ARCHIVE_BYTES,
            "etag": holdout.ARCHIVE_ETAG,
            "last_modified_http": holdout.ARCHIVE_LAST_MODIFIED,
        },
        "entries": protocol.ENTRIES,
        "requests": {
            "maximum_network_requests": 4,
            "artifacts": [
                {"name": name, "range": value, "role": role}
                for name, value, role in ACQUISITION_ARTIFACTS
            ],
            "audio_prefix_bytes": protocol.AUDIO_PREFIX_BYTES,
            "metadata_bytes": 1_355,
            "prefix_growth_allowed": False,
            "retry_allowed": False,
        },
        "decode": {
            "audio_dtype": "<f4",
            "audio_shape": [protocol.TOTAL_ROWS, protocol.SAMPLE_COUNT],
            "npy_header_bytes": protocol.NPY_HEADER_BYTES,
            "decoded_rows": protocol.ANALYSIS_ROW_COUNT,
            "decoded_row_bytes": protocol.ROW_BYTES,
            "decoded_bytes": protocol.DECODED_BYTES,
            "metadata_dtype": "<i8",
            "metadata_shape": [protocol.TOTAL_ROWS],
        },
        "analysis": {
            "sample_rate_hz": protocol.SAMPLE_RATE_HZ,
            "observation_samples": v1.OBSERVATION_SAMPLES,
            "onset": {
                "rule": "first spatial norm sample at or above 5 percent of maximum",
                "minimum_peak_fraction": 0.05,
            },
            "partial_svd_rank": dense.PARTIAL_SVD_RANK,
            "opened_region_ranking": "forbidden",
            "scale_factors": broadband.SCALE_FACTORS,
            "spatial_partitions": v1.SPATIAL_PARTITIONS,
            "prediction_calibration_samples": v1.PREDICTION_CALIBRATION_SAMPLES,
            "prediction_windows": v1.PREDICTION_WINDOWS,
            "gates": protocol.GATES,
        },
        "sources": [
            {"path": path, "sha256": digest} for path, digest in SOURCE_HASHES.items()
        ],
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "condition_metadata_allowed": True,
            "one_synchronized_listener_group_allowed": True,
            "additional_audio_or_planter_payload_allowed": False,
            "physics_solver_allowed": False,
            "threshold_or_variant_tuning_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "commit a repeated zero-access execution preflight before acquire; "
            "then stop with authored-clip fallback if acquisition, decode, row "
            "identity, repeat, bounded capacity, scale invariance or any frozen "
            "gate fails; no retry, prefix growth, object substitution, physics "
            "or Planter access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ExecutionError("holdout execution manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ExecutionError(f"parse holdout execution manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ExecutionError("holdout execution manifest changed")
    return data, manifest


def resolve_reference(
    base: Path, reference: dict[str, Any], label: str
) -> tuple[bytes, Path]:
    path = (base / reference["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise ExecutionError(f"{label} escapes physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != reference["sha256"]:
        raise ExecutionError(f"{label} identity changed")
    return data, path


def validate_protocol(
    root: Path, base: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    references = manifest["protocol"]
    runner_path = (root / references["runner"]["path"]).resolve(strict=True)
    if (
        not runner_path.is_file()
        or not runner_path.is_relative_to(root)
        or sha256_file(runner_path) != references["runner"]["sha256"]
    ):
        raise ExecutionError("bound protocol runner changed")
    protocol_manifest_bytes, protocol_manifest_path = resolve_reference(
        base, references["manifest"], "protocol manifest"
    )
    protocol_preflight_bytes, _ = resolve_reference(
        base, references["preflight"], "protocol preflight"
    )
    protocol_manifest = json.loads(protocol_manifest_bytes)
    protocol_preflight = json.loads(protocol_preflight_bytes)
    if (
        protocol_manifest != protocol.expected_manifest(PROTOCOL_RUNNER_SHA256)
        or protocol_preflight.get("decision")
        != "BroadbandIndependentHoldoutObservationProtocolFrozen"
        or protocol_preflight.get("manifest_sha256") != PROTOCOL_MANIFEST_SHA256
        or protocol_preflight.get("runner_sha256") != PROTOCOL_RUNNER_SHA256
        or protocol_preflight.get("network_requests") != 0
        or protocol_preflight.get("object_payload_bytes_read") != 0
        or protocol_preflight.get("planter_payload_bytes_read") != 0
        or protocol_preflight.get("quality_admission_or_runtime_credit") is not False
    ):
        raise ExecutionError("holdout protocol lineage changed")
    return {
        "runner_sha256": sha256_file(runner_path),
        "manifest_sha256": sha256_bytes(protocol_manifest_bytes),
        "preflight_sha256": sha256_bytes(protocol_preflight_bytes),
        "decision": protocol_preflight["decision"],
        "manifest_path": str(protocol_manifest_path),
    }


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    result = []
    for reference in manifest["sources"]:
        path = (root / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != reference["sha256"]
        ):
            raise ExecutionError(f"bound source changed: {reference['path']}")
        result.append(
            {
                "path": reference["path"],
                "sha256": reference["sha256"],
                "bytes": path.stat().st_size,
            }
        )
    return result


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "impact_ordinal": 0,
        "role": "independent_method_holdout",
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_admission_or_runtime_credit": False,
        "opened_region_ranking_used": False,
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
        "decision": "BroadbandIndependentHoldoutExecutionFrozen",
        "claim": (
            "HASH_CLOSED_EXECUTION_PREFLIGHT / ZERO_NETWORK_OR_OBJECT_PAYLOAD_"
            "ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
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
    holdout.configure_core()
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent)
    )
    responses = []
    attempted = 0
    try:
        for name, (start, end), role in ACQUISITION_ARTIFACTS:
            attempted += 1
            response = legacy.download_range(start, end, staging / name)
            responses.append({"artifact": name, "role": role, **response})
    except (OSError, RuntimeError) as error:
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "status": "Validated",
            "decision": "BroadbandIndependentHoldoutAcquisitionRejected",
            "claim": (
                "IMMUTABLE_BOUNDED_ACQUISITION_FAILURE / NO_RETRY_OR_PREFIX_"
                "GROWTH / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
            ),
            **common_report(manifest_bytes, runner_sha256),
            "protocol": lineage,
            "network_requests_attempted": attempted,
            "completed_responses": responses,
            "failure": f"{type(error).__name__}: {error}",
            "additional_requests_allowed": 0,
            "audio_payload_bytes_read": (
                (staging / "observation-prefix-32m.deflate").stat().st_size
                if (staging / "observation-prefix-32m.deflate").is_file()
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
            "decision": "BroadbandIndependentHoldoutInputsAcquired",
            "claim": (
                "THREE_CONDITION_BLOCKS_AND_ONE_FROZEN_AUDIO_PREFIX_ONLY / "
                "NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
            ),
            **common_report(manifest_bytes, runner_sha256),
            "protocol": lineage,
            "responses": responses,
            "network_requests": len(responses),
            "metadata_payload_bytes_read": 1_355,
            "audio_payload_bytes_read": protocol.AUDIO_PREFIX_BYTES,
            "additional_requests_allowed": 0,
            "next_action": (
                "decode only four condition arrays and the first synchronized "
                "15-listener group"
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
        or report.get("decision") != "BroadbandIndependentHoldoutInputsAcquired"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("network_requests") != 4
        or report.get("audio_payload_bytes_read") != protocol.AUDIO_PREFIX_BYTES
        or report.get("planter_payload_bytes_read") != 0
        or any(not path.is_file() for path in paths.values())
    ):
        raise ExecutionError("holdout acquisition lineage changed")
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
    rows = slice(0, protocol.IMPACT_ROW_COUNT)
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
    for start in range(0, protocol.IMPACT_ROW_COUNT, 15):
        if not np.all(angles[start : start + 15] == angles[start]) or not np.all(
            distances[start : start + 15] == distances[start]
        ):
            raise ExecutionError("impact-zero condition grouping changed")
    if microphones[protocol.REFERENCE_ROW] != protocol.REFERENCE_ROW:
        raise ExecutionError("reference row is not microphone 7")
    return {
        "impact_row_count": protocol.IMPACT_ROW_COUNT,
        "analysis_row_count": protocol.ANALYSIS_ROW_COUNT,
        "vertex_id": int(vertex_ids[0]),
        "angle_values_degrees": [int(value) for value in np.unique(angles)],
        "distance_values_millimetres": [int(value) for value in np.unique(distances)],
        "microphone_values": [int(value) for value in np.unique(microphones)],
        "condition_pair_count": 40,
        "analysis_condition": {
            "rows": [0, protocol.ANALYSIS_ROW_COUNT - 1],
            "angle_degrees": int(angles[0]),
            "distance_millimetres": int(distances[0]),
            "microphone_ids": [
                int(value) for value in microphones[: protocol.ANALYSIS_ROW_COUNT]
            ],
        },
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
        raise ExecutionError("Iron Mortar audio NPY header changed")
    text = header[10:].decode("latin1")
    if (
        "'descr': '<f4'" not in text
        or "'fortran_order': False" not in text
        or f"'shape': ({protocol.TOTAL_ROWS}, {protocol.SAMPLE_COUNT})" not in text
        or not text.endswith("\n")
    ):
        raise ExecutionError(f"Iron Mortar audio NPY descriptor changed: {text!r}")


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
            f"frozen prefix decoded {decoded} row bytes, expected "
            f"{protocol.DECODED_BYTES}"
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
    distance = paths["distance-record.bin"].read_bytes()
    mic_vertex = paths["mic-vertex-records.bin"].read_bytes()
    angle = paths["angle-record.bin"].read_bytes()
    arrays = {
        "distance.npy": legacy.load_npy(
            legacy.decode_local_record(
                distance, protocol.DISTANCE_RANGE[0], protocol.ENTRIES["distance.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "distance",
        ),
        "micID.npy": legacy.load_npy(
            legacy.decode_local_record(
                mic_vertex, protocol.MIC_VERTEX_RANGE[0], protocol.ENTRIES["micID.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "micID",
        ),
        "vertexID.npy": legacy.load_npy(
            legacy.decode_local_record(
                mic_vertex,
                protocol.MIC_VERTEX_RANGE[0],
                protocol.ENTRIES["vertexID.npy"],
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "vertexID",
        ),
        "angle.npy": legacy.load_npy(
            legacy.decode_local_record(
                angle, protocol.ANGLE_RANGE[0], protocol.ENTRIES["angle.npy"]
            ),
            (protocol.TOTAL_ROWS,),
            "<i8",
            "angle",
        ),
    }
    metadata = validate_metadata(arrays)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent)
    )
    decoded_path = staging / DECODED_NAME
    try:
        header_sha256, decoded_sha256 = decode_audio_prefix(
            paths["observation-prefix-32m.deflate"], decoded_path
        )
        report = {
            "schema": REPORT_SCHEMAS["decode"],
            "status": "Validated",
            "decision": "BroadbandIndependentHoldoutObservationDecoded",
            "claim": (
                "CONDITION_METADATA_AND_ONE_SYNCHRONIZED_LISTENER_GROUP_ONLY / "
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
            "row_count": protocol.ANALYSIS_ROW_COUNT,
            "sample_count": protocol.SAMPLE_COUNT,
            "network_requests": 0,
            "additional_audio_payload_bytes_read": 0,
            "next_action": "run the frozen V2 holdout analysis twice",
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
        or report.get("decision") != "BroadbandIndependentHoldoutObservationDecoded"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("decoded_bytes") != protocol.DECODED_BYTES
        or report.get("row_count") != protocol.ANALYSIS_ROW_COUNT
        or report.get("sample_count") != protocol.SAMPLE_COUNT
        or report.get("additional_audio_payload_bytes_read") != 0
        or report.get("planter_payload_bytes_read") != 0
        or not decoded_path.is_file()
        or decoded_path.stat().st_size != protocol.DECODED_BYTES
        or sha256_file(decoded_path) != report.get("decoded_sha256")
    ):
        raise ExecutionError("holdout decode lineage changed")
    return report_bytes, report, decoded_path


def detect_onset(channels: np.ndarray) -> int:
    spatial_norm = np.linalg.norm(channels, axis=0)
    maximum = float(np.max(spatial_norm))
    if not np.isfinite(maximum) or maximum <= 0.0:
        raise ExecutionError("holdout spatial norm has no finite positive peak")
    candidates = np.flatnonzero(spatial_norm >= 0.05 * maximum)
    if not len(candidates):
        raise ExecutionError("holdout onset is absent")
    onset = int(candidates[0])
    if onset + v1.OBSERVATION_SAMPLES > channels.shape[1]:
        raise ExecutionError("holdout onset leaves fewer than 60000 samples")
    return onset


def analyze_partition(
    coefficients: np.ndarray,
    discovery: dict[str, Any],
    output_indices: list[int],
    retained_modes: list[dict[str, Any]],
) -> dict[str, Any]:
    raw, analyses = dense.extract_partial_estimates(
        coefficients[:, :, output_indices], discovery
    )
    clusters = broadband.cluster_duplicates(raw)
    return {
        "output_indices": output_indices,
        "raw_estimate_count": len(raw),
        "cluster_count": len(clusters),
        "clusters": clusters,
        "bin_analyses": analyses,
        "retained_full_output_matching": v1.injective_match(
            retained_modes, v1.representatives(clusters)
        ),
    }


def check(name: str, observed: Any, relation: str, threshold: Any) -> dict[str, Any]:
    if relation == "<=":
        passed = observed <= threshold
    elif relation == ">=":
        passed = observed >= threshold
    else:
        passed = observed == threshold
    return {
        "name": name,
        "observed": observed,
        "relation": relation,
        "threshold": threshold,
        "passed": passed,
    }


def rejected_analysis(
    *,
    common: dict[str, Any],
    decode_sha256: str,
    onset: int,
    discovery: dict[str, Any],
    scale_invariance: dict[str, Any],
    checks: list[dict[str, Any]],
    stage: str,
    reason: str,
    details: dict[str, Any] | None = None,
) -> dict[str, Any]:
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": "BroadbandIndependentHoldoutMethodTransferRejected",
        "claim": (
            "INDEPENDENT_REAL_HOLDOUT_METHOD_OR_CAPACITY_REJECTION / NO_"
            "MATERIAL_IDENTITY_PERCEPTUAL_QUALITY_DOMAIN_ADMISSION_PHYSICS_"
            "PLANTER_OR_RUNTIME_CREDIT"
        ),
        **common,
        "decode_report_sha256": decode_sha256,
        "onset_sample": onset,
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "all_discovered_regions_analyzed": False,
        "scale_invariance": scale_invariance,
        "rejection_stage": stage,
        "rejection_reason": reason,
        "details": details or {},
        "network_requests": 0,
        "additional_audio_payload_bytes_read": 0,
        "gate": {"passed": False, "checks": checks},
        "next_action": (
            "preserve independent rejection and authored-clip fallback; do not "
            "retune, retry, grow the prefix or substitute another object"
        ),
    }


def analyze(
    manifest_bytes: bytes,
    runner_sha256: str,
    decode_dir: Path,
) -> dict[str, Any]:
    decode_bytes, decode_report, decoded_path = validate_decode(
        manifest_bytes, decode_dir
    )
    mapped = np.memmap(
        decoded_path,
        dtype="<f4",
        mode="r",
        shape=(protocol.ANALYSIS_ROW_COUNT, protocol.SAMPLE_COUNT),
        order="C",
    )
    channels = np.asarray(mapped, dtype=np.float64)
    if not np.all(np.isfinite(channels)):
        raise ExecutionError("decoded holdout contains non-finite values")
    onset = detect_onset(channels)
    observation = channels[:, onset : onset + v1.OBSERVATION_SAMPLES]
    gabor_stop = (
        broadband.WINDOW_SAMPLES
        + (broadband.GABOR_FRAME_COUNT - 1) * broadband.HOP_SAMPLES
    )
    coefficients = broadband.gabor_coefficients(observation[:, :gabor_stop])
    discovery = broadband.discover_regions(coefficients)
    scale_invariance = broadband.discovery_scale_invariance(coefficients)
    common = {
        **common_report(manifest_bytes, runner_sha256),
        "decoded_observation_sha256": decode_report["decoded_sha256"],
        "input_slice_sha256": sha256_bytes(
            observation.astype("<f8", copy=False).tobytes()
        ),
        "gabor_coefficients_sha256": sha256_bytes(
            coefficients.astype("<c16", copy=False).tobytes()
        ),
    }
    gates = protocol.GATES
    capacity_checks = [
        check(
            "minimum_region_count",
            len(discovery["regions"]),
            ">=",
            gates["minimum_region_count"],
        ),
        check(
            "maximum_region_count",
            len(discovery["regions"]),
            "<=",
            gates["maximum_region_count"],
        ),
        check(
            "maximum_analysis_bin_count",
            len(discovery["analysis_bins"]),
            "<=",
            gates["maximum_analysis_bin_count"],
        ),
        check(
            "scale_invariant_region_bins",
            scale_invariance["passed"],
            "==",
            gates["scale_invariant_region_bins"],
        ),
    ]
    if not all(item["passed"] for item in capacity_checks):
        return rejected_analysis(
            common=common,
            decode_sha256=sha256_bytes(decode_bytes),
            onset=onset,
            discovery=discovery,
            scale_invariance=scale_invariance,
            checks=capacity_checks,
            stage="discovery",
            reason="independent discovery left the frozen 32/96 envelope",
        )

    raw, bin_analyses = dense.extract_partial_estimates(coefficients, discovery)
    clusters = broadband.cluster_duplicates(raw)
    duplicate_estimates_removed = len(raw) - len(clusters)
    cluster_checks = [
        *capacity_checks,
        check(
            "duplicate_estimates_removed",
            duplicate_estimates_removed,
            ">=",
            gates["minimum_duplicate_estimates_removed"],
        ),
        check(
            "minimum_preprune_cluster_count",
            len(clusters),
            ">=",
            gates["minimum_preprune_cluster_count"],
        ),
        check(
            "maximum_preprune_cluster_count",
            len(clusters),
            "<=",
            gates["maximum_preprune_cluster_count"],
        ),
    ]
    if not all(item["passed"] for item in cluster_checks):
        return rejected_analysis(
            common=common,
            decode_sha256=sha256_bytes(decode_bytes),
            onset=onset,
            discovery=discovery,
            scale_invariance=scale_invariance,
            checks=cluster_checks,
            stage="pole_estimation",
            reason="independent partial-SVD clusters left the fit envelope",
            details={
                "bin_analyses": bin_analyses,
                "raw_estimate_count": len(raw),
                "duplicate_estimates_removed": duplicate_estimates_removed,
                "preprune_cluster_count": len(clusters),
            },
        )

    fitted, candidate_reconstruction, undamped_reconstruction = v1.fit_and_prune(
        observation, clusters
    )
    retained = [item for item in fitted if item["retained"]]
    retained_modes = [item["representative"] for item in retained]
    retained_checks = [
        *cluster_checks,
        check(
            "minimum_retained_mode_count",
            len(retained),
            ">=",
            gates["minimum_retained_mode_count"],
        ),
        check(
            "maximum_retained_mode_count",
            len(retained),
            "<=",
            gates["maximum_retained_mode_count"],
        ),
    ]
    if not all(item["passed"] for item in retained_checks):
        return rejected_analysis(
            common=common,
            decode_sha256=sha256_bytes(decode_bytes),
            onset=onset,
            discovery=discovery,
            scale_invariance=scale_invariance,
            checks=retained_checks,
            stage="amplitude_pruning",
            reason="independent retained modes left the frozen envelope",
            details={
                "bin_analyses": bin_analyses,
                "raw_estimate_count": len(raw),
                "duplicate_estimates_removed": duplicate_estimates_removed,
                "fitted_clusters": fitted,
                "retained_mode_count": len(retained),
            },
        )

    candidate_nrmse = v1.normalized_rms_error(observation, candidate_reconstruction)
    undamped_nrmse = v1.normalized_rms_error(observation, undamped_reconstruction)
    full_ratio = candidate_nrmse / max(undamped_nrmse, 1.0e-300)
    prediction = v1.prediction_control(observation, retained_modes)
    partitions = {
        name: analyze_partition(coefficients, discovery, indices, retained_modes)
        for name, indices in v1.SPATIAL_PARTITIONS.items()
    }
    even_fraction = partitions["even_microphones"]["retained_full_output_matching"][
        "match_fraction"
    ]
    odd_fraction = partitions["odd_microphones"]["retained_full_output_matching"][
        "match_fraction"
    ]
    checks = [
        *retained_checks,
        check(
            "even_partition_match_fraction",
            even_fraction,
            ">=",
            gates["minimum_even_partition_match_fraction"],
        ),
        check(
            "odd_partition_match_fraction",
            odd_fraction,
            ">=",
            gates["minimum_odd_partition_match_fraction"],
        ),
        check(
            "full_observed_nrmse",
            candidate_nrmse,
            "<=",
            gates["maximum_full_observed_nrmse"],
        ),
        check(
            "full_damped_to_undamped_nrmse_ratio",
            full_ratio,
            "<=",
            gates["maximum_full_damped_to_undamped_nrmse_ratio"],
        ),
        check(
            "prediction_error_ratio_window_a",
            prediction["windows"][0]["damped_to_undamped_error_ratio"],
            "<=",
            gates["maximum_prediction_error_ratio_window_a"],
        ),
        check(
            "prediction_error_ratio_window_b",
            prediction["windows"][1]["damped_to_undamped_error_ratio"],
            "<=",
            gates["maximum_prediction_error_ratio_window_b"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": (
            "BroadbandIndependentHoldoutMethodTransferSupported"
            if passed
            else "BroadbandIndependentHoldoutMethodTransferRejected"
        ),
        "claim": (
            "OBJECT_DISJOINT_REAL_HOLDOUT_COMMON_POLE_METHOD_TRANSFER_ONLY / "
            "NO_MATERIAL_IDENTITY_PERCEPTUAL_QUALITY_DOMAIN_ADMISSION_PHYSICS_"
            "PLANTER_OR_RUNTIME_CREDIT"
        ),
        **common,
        "decode_report_sha256": sha256_bytes(decode_bytes),
        "onset_sample": onset,
        "onset_ms": onset * 1000.0 / protocol.SAMPLE_RATE_HZ,
        "observation_samples": v1.OBSERVATION_SAMPLES,
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "all_discovered_regions_analyzed": len(bin_analyses)
        == len(discovery["analysis_bins"]),
        "scale_invariance": scale_invariance,
        "bin_analyses": bin_analyses,
        "raw_estimate_count": len(raw),
        "duplicate_estimates_removed": duplicate_estimates_removed,
        "fitted_clusters": fitted,
        "retained_mode_count": len(retained),
        "pruned_mode_count": len(fitted) - len(retained),
        "spatial_partitions": partitions,
        "reconstruction": {
            "candidate_nrmse": candidate_nrmse,
            "undamped_nrmse": undamped_nrmse,
            "damped_to_undamped_nrmse_ratio": full_ratio,
        },
        "prediction_control": prediction,
        "deterministic_work_envelope": {
            "region_count": len(discovery["regions"]),
            "analysis_bin_count": len(discovery["analysis_bins"]),
            "partial_svd_rank": dense.PARTIAL_SVD_RANK,
            "preprune_cluster_count": len(clusters),
            "amplitude_design_columns": 2 * len(clusters),
        },
        "network_requests": 0,
        "additional_audio_payload_bytes_read": 0,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze a second object-disjoint real holdout before broader method "
            "credit; keep material, quality and domain admission separate"
            if passed
            else "preserve independent rejection and authored-clip fallback; do "
            "not retune, retry or substitute another object"
        ),
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = holdout.discovery.external_file(
        root, arguments.manifest, "manifest"
    )
    output = holdout.discovery.external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    if arguments.stage == "preflight":
        if arguments.input is not None:
            raise ExecutionError("preflight does not accept --input")
        report = preflight(root, base, manifest_bytes, manifest, runner_sha256)
        report_bytes = publish_small(output, report)
    elif arguments.stage == "acquire":
        if arguments.input is not None:
            raise ExecutionError("acquire does not accept --input")
        report = acquire(root, base, manifest_bytes, manifest, runner_sha256, output)
        report_bytes = canonical_json(report)
    elif arguments.stage == "decode":
        if arguments.input is None:
            raise ExecutionError("decode requires --input acquisition")
        acquisition = holdout.discovery.external_directory(
            root, arguments.input, "acquisition"
        )
        report = decode(manifest_bytes, runner_sha256, acquisition, output)
        report_bytes = canonical_json(report)
    else:
        if arguments.input is None:
            raise ExecutionError("analyze requires --input decode")
        decode_dir = holdout.discovery.external_directory(
            root, arguments.input, "decode"
        )
        report = analyze(manifest_bytes, runner_sha256, decode_dir)
        report_bytes = publish_small(output, report)
    print(f"REALIMPACT broad-band holdout {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print(f"audio payload bytes read: {report.get('audio_payload_bytes_read', 0)}")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
