#!/usr/bin/env python3
"""Freeze one synchronized REALIMPACT broad-band holdout observation."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_realimpact_broadband_holdout_discovery as holdout
import physical_sound_realimpact_broadband_v2_counterfactual as v2
import physical_sound_realimpact_observation_discovery as discovery_core
import scipy

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-broadband-holdout-protocol.manifest.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-broadband-holdout-protocol-preflight.report.v1"
)
STUDY_ID = "physical-sound-realimpact-broadband-independent-holdout"
REVISION = "iron-mortar-impact-zero-condition-zero-broadband-v2"
OBJECT_ID = holdout.OBJECT_ID

TOTAL_ROWS = 3_000
IMPACT_ROW_COUNT = 600
ANALYSIS_ROW_COUNT = 15
REFERENCE_ROW = 7
SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 208_375
SAMPLE_BYTES = 4
ROW_BYTES = SAMPLE_COUNT * SAMPLE_BYTES
DECODED_BYTES = ANALYSIS_ROW_COUNT * ROW_BYTES
NPY_HEADER_BYTES = 128
AUDIO_PREFIX_BYTES = 33_554_432

DISTANCE_RANGE = [157, 547]
MIC_VERTEX_RANGE = [3_044_594, 3_045_171]
ANGLE_RANGE = [2_305_328_734, 2_305_329_119]
AUDIO_PREFIX_RANGE = [3_046_013, 36_600_444]

DISCOVERY_MANIFEST_SHA256 = (
    "931f6a72f16f9a52bc7386d40aad1b5e0d5f674bd0be410ed5d35a5c090bf5e7"
)
DISCOVERY_ACQUISITION_SHA256 = (
    "84308e8e28623d114f1d0002c9a7f5cc94bb151dc78d7aed1be1dbf35d39d080"
)
DISCOVERY_AUDIT_SHA256 = (
    "67c92621d1a58507f92303b709427ecfb6ad0eb972e497621ba9ce8bf4c48a42"
)
DISCOVERY_TAIL_SHA256 = (
    "f8bd43a7f49464bf5b8cac19295deae97816e68970996500010850cca29a13af"
)
DISCOVERY_AUDIO_HEADER_SHA256 = (
    "61ce30bf1d5083e67cb982aa8cf73b1e6a6212618736397f54115761dafd7d16"
)
DISCOVERY_RUNNER_SHA256 = (
    "c59aea1d0fd8ae2dd565929120b207f33326c697d2daacaf82469e0830c679cd"
)
DISCOVERY_CORE_SHA256 = (
    "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
)
V2_RUNNER_SHA256 = "3f1c07b501dec97645cc0f1ad2fec5b3960312eacde88a291b64cc740a056aa5"
V2_REPORT_SHA256 = "f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73"
V2_RESULT_DOC_SHA256 = (
    "c531d97899d2602f13e7d96b8b6fad639bd9911c421c2b8abeb10d05d2673897"
)

ENTRIES = {
    "distance.npy": {
        "name": f"{OBJECT_ID}/preprocessed/distance.npy",
        "crc32": "570b48dd",
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "local_offset": 157,
        "method": 8,
    },
    "micID.npy": {
        "name": f"{OBJECT_ID}/preprocessed/micID.npy",
        "crc32": "082ec0c6",
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "local_offset": 3_044_594,
        "method": 8,
    },
    "vertexID.npy": {
        "name": f"{OBJECT_ID}/preprocessed/vertexID.npy",
        "crc32": "60ca8636",
        "compressed_bytes": 162,
        "uncompressed_bytes": 24_128,
        "local_offset": 3_044_913,
        "method": 8,
    },
    "angle.npy": {
        "name": f"{OBJECT_ID}/preprocessed/angle.npy",
        "crc32": "fabb9a2e",
        "compressed_bytes": 292,
        "uncompressed_bytes": 24_128,
        "local_offset": 2_305_328_734,
        "method": 8,
    },
    "deconvolved_0db.npy": {
        "name": f"{OBJECT_ID}/preprocessed/deconvolved_0db.npy",
        "crc32": "9da9ace4",
        "compressed_bytes": 2_302_282_721,
        "uncompressed_bytes": 2_500_500_128,
        "local_offset": 3_045_909,
        "data_offset": 3_046_013,
        "method": 8,
    },
}

GATES = {
    "minimum_region_count": 1,
    "maximum_region_count": 32,
    "maximum_analysis_bin_count": 96,
    "minimum_duplicate_estimates_removed": 1,
    "minimum_preprune_cluster_count": 6,
    "maximum_preprune_cluster_count": 64,
    "minimum_retained_mode_count": 6,
    "maximum_retained_mode_count": 64,
    "scale_invariant_region_bins": True,
    "minimum_even_partition_match_fraction": 0.50,
    "minimum_odd_partition_match_fraction": 0.50,
    "maximum_full_observed_nrmse": 0.95,
    "maximum_full_damped_to_undamped_nrmse_ratio": 0.95,
    "maximum_prediction_error_ratio_window_a": 0.95,
    "maximum_prediction_error_ratio_window_b": 0.95,
}


class ProtocolError(RuntimeError):
    """The holdout protocol or one of its frozen parents changed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
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
        "runner_sha256": runner_sha256,
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "independent_method_holdout",
            "impact_ordinal": 0,
            "impact_rows": [0, IMPACT_ROW_COUNT - 1],
            "analysis_rows": [0, ANALYSIS_ROW_COUNT - 1],
            "reference_row": REFERENCE_ROW,
            "prior_observation_payload_bytes_read": 0,
            "planter_payload_access_allowed": False,
        },
        "archive": {
            "url": holdout.ARCHIVE_URL,
            "bytes": holdout.ARCHIVE_BYTES,
            "etag": holdout.ARCHIVE_ETAG,
            "last_modified_http": holdout.ARCHIVE_LAST_MODIFIED,
        },
        "discovery": {
            "runner": {
                "path": (
                    "lab/scripts/physical_sound_realimpact_broadband_"
                    "holdout_discovery.py"
                ),
                "sha256": DISCOVERY_RUNNER_SHA256,
            },
            "core": {
                "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
                "sha256": DISCOVERY_CORE_SHA256,
            },
            "manifest": {
                "path": (
                    "../ps2-realimpact-iron-mortar-broadband-holdout-"
                    "discovery-v1/manifest.json"
                ),
                "sha256": DISCOVERY_MANIFEST_SHA256,
            },
            "acquisition_report": {
                "path": (
                    "../ps2-realimpact-iron-mortar-broadband-holdout-"
                    "discovery-v1/acquisition/report.json"
                ),
                "sha256": DISCOVERY_ACQUISITION_SHA256,
            },
            "audit_report": {
                "path": (
                    "../ps2-realimpact-iron-mortar-broadband-holdout-"
                    "discovery-v1/audit-a/report.json"
                ),
                "sha256": DISCOVERY_AUDIT_SHA256,
            },
            "tail": {
                "path": (
                    "../ps2-realimpact-iron-mortar-broadband-holdout-"
                    "discovery-v1/acquisition/tail.bin"
                ),
                "sha256": DISCOVERY_TAIL_SHA256,
                "bytes": holdout.TAIL_BYTES,
            },
            "audio_local_header": {
                "path": (
                    "../ps2-realimpact-iron-mortar-broadband-holdout-"
                    "discovery-v1/acquisition/audio-local-header.bin"
                ),
                "sha256": DISCOVERY_AUDIO_HEADER_SHA256,
                "bytes": holdout.LOCAL_HEADER_BYTES,
            },
        },
        "entries": ENTRIES,
        "requests": {
            "maximum_network_requests": 4,
            "distance_range": DISTANCE_RANGE,
            "mic_vertex_range": MIC_VERTEX_RANGE,
            "angle_range": ANGLE_RANGE,
            "audio_prefix_range": AUDIO_PREFIX_RANGE,
            "audio_prefix_bytes": AUDIO_PREFIX_BYTES,
            "metadata_bytes": 1_355,
            "prefix_growth_allowed": False,
            "retry_allowed": False,
        },
        "decode": {
            "audio_dtype": "<f4",
            "audio_shape": [TOTAL_ROWS, SAMPLE_COUNT],
            "npy_header_bytes": NPY_HEADER_BYTES,
            "decoded_rows": ANALYSIS_ROW_COUNT,
            "decoded_row_bytes": ROW_BYTES,
            "decoded_bytes": DECODED_BYTES,
            "metadata_dtype": "<i8",
            "metadata_shape": [TOTAL_ROWS],
            "required_metadata": [
                "angle.npy",
                "distance.npy",
                "micID.npy",
                "vertexID.npy",
            ],
        },
        "candidate": {
            "runner": {
                "path": (
                    "lab/scripts/physical_sound_realimpact_broadband_v2_"
                    "counterfactual.py"
                ),
                "sha256": V2_RUNNER_SHA256,
            },
            "parent_report": {
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
            "only_holdout_changes": [
                "input-derived onset at five percent of spatial peak",
                "remove parent-specific exact 31-region and 91-bin identity gates",
            ],
            "partial_svd": {
                "rank": 7,
                "solver": "propack",
                "which": "LM",
                "rng_seed_per_bin": 0,
            },
            "discovery_and_fit": {
                "opened_region_ranking": "forbidden",
                "post_estimation_mode_energy_floor_db": -25.0,
                "maximum_amplitude_design_columns": 128,
            },
            "onset": {
                "rule": "first spatial norm sample at or above 5 percent of maximum",
                "minimum_peak_fraction": 0.05,
                "must_leave_60000_post_onset_samples": True,
            },
            "spatial_partitions": v2.v1.SPATIAL_PARTITIONS,
            "partition_matching_tolerance_cents": v2.v1.MATCH_TOLERANCE_CENTS,
            "prediction_calibration_samples": [
                0,
                v2.v1.PREDICTION_CALIBRATION_SAMPLES,
            ],
            "prediction_windows": v2.v1.PREDICTION_WINDOWS,
            "scale_factors": v2.broadband.SCALE_FACTORS,
            "gates": GATES,
        },
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
            "implement and hash-close an execution runner before access; then "
            "stop with authored-clip fallback if acquisition, decode, row "
            "identity, repeat, bounded capacity, scale invariance or any "
            "unchanged V2 gate fails; no retry, prefix growth, object "
            "substitution, physics or Planter access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ProtocolError("holdout protocol manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ProtocolError(f"parse holdout protocol manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ProtocolError("holdout protocol manifest changed")
    return data, manifest


def resolve_reference(
    base: Path, reference: dict[str, Any], label: str
) -> tuple[bytes, Path]:
    path = (base / reference["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise ProtocolError(f"{label} escapes physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != reference["sha256"] or (
        "bytes" in reference and len(data) != reference["bytes"]
    ):
        raise ProtocolError(f"{label} identity changed")
    return data, path


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    references = [
        manifest["discovery"]["runner"],
        manifest["discovery"]["core"],
        manifest["candidate"]["runner"],
        manifest["candidate"]["result_document"],
    ]
    result = []
    for reference in references:
        path = (root / reference["path"]).resolve(strict=True)
        if not path.is_file() or not path.is_relative_to(root):
            raise ProtocolError(f"source escapes repository: {reference['path']}")
        if sha256_file(path) != reference["sha256"]:
            raise ProtocolError(f"bound source changed: {reference['path']}")
        result.append(
            {
                "path": reference["path"],
                "sha256": reference["sha256"],
                "bytes": path.stat().st_size,
            }
        )
    return result


def validate_discovery(base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    references = manifest["discovery"]
    manifest_bytes, _ = resolve_reference(
        base, references["manifest"], "discovery manifest"
    )
    acquisition_bytes, _ = resolve_reference(
        base, references["acquisition_report"], "discovery acquisition"
    )
    audit_bytes, _ = resolve_reference(
        base, references["audit_report"], "discovery audit"
    )
    tail, _ = resolve_reference(base, references["tail"], "discovery tail")
    header, _ = resolve_reference(
        base, references["audio_local_header"], "observation local header"
    )
    acquisition = json.loads(acquisition_bytes)
    audit = json.loads(audit_bytes)
    holdout.configure_core()
    central, entries = discovery_core.discover_from_tail(tail)
    audio = next(
        entry for entry in entries if entry["name"] == holdout.EXPECTED_AUDIO_ENTRY
    )
    audio_header = discovery_core.validate_audio_local_header(header, audio)
    if (
        acquisition.get("decision")
        != "BroadbandIndependentHoldoutArchiveEntryDiscovered"
        or acquisition.get("runner_sha256") != DISCOVERY_RUNNER_SHA256
        or acquisition.get("network_requests") != 2
        or acquisition.get("audio_payload_bytes_read") != 0
        or acquisition.get("planter_audio_payload_bytes_read") != 0
        or acquisition.get("central_directory") != central
        or acquisition.get("entries") != entries
        or acquisition.get("observation_local_header") != audio_header
        or audit.get("decision") != "BroadbandIndependentHoldoutDiscoveryCacheVerified"
        or audit.get("acquisition_report_sha256") != DISCOVERY_ACQUISITION_SHA256
        or audit.get("network_requests") != 0
        or audit.get("audio_payload_bytes_read") != 0
    ):
        raise ProtocolError("holdout discovery lineage changed")
    for name, expected in manifest["entries"].items():
        matches = [entry for entry in entries if entry["name"] == expected["name"]]
        if len(matches) != 1:
            raise ProtocolError(f"required entry is absent: {name}")
        for field in [
            "crc32",
            "compressed_bytes",
            "uncompressed_bytes",
            "local_offset",
            "method",
        ]:
            if matches[0][field] != expected[field]:
                raise ProtocolError(f"entry field changed: {name}.{field}")
    if (
        audio_header["data_offset"] != ENTRIES["deconvolved_0db.npy"]["data_offset"]
        or ENTRIES["deconvolved_0db.npy"]["uncompressed_bytes"]
        != NPY_HEADER_BYTES + TOTAL_ROWS * ROW_BYTES
        or any(
            ENTRIES[name]["uncompressed_bytes"] != NPY_HEADER_BYTES + TOTAL_ROWS * 8
            for name in ["angle.npy", "distance.npy", "micID.npy", "vertexID.npy"]
        )
    ):
        raise ProtocolError("frozen NPY shape arithmetic changed")
    return {
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
        "audit_report_sha256": sha256_bytes(audit_bytes),
        "tail_sha256": sha256_bytes(tail),
        "audio_local_header_sha256": sha256_bytes(header),
        "central_directory_sha256": central["sha256"],
        "entry_count": len(entries),
        "audio_data_offset": audio_header["data_offset"],
    }


def validate_candidate_parent(base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    reference = manifest["candidate"]["parent_report"]
    data, _ = resolve_reference(base, reference, "V2 parent")
    report = json.loads(data)
    if (
        report.get("decision") != reference["required_decision"]
        or report.get("gate", {}).get("passed") is not True
        or report.get("runner_sha256") != V2_RUNNER_SHA256
        or report.get("additional_payload_bytes_read") != 0
        or report.get("planter_payload_bytes_read") != 0
        or report.get("quality_admission_or_runtime_credit") is not False
    ):
        raise ProtocolError("V2 parent lineage changed")
    return {"sha256": sha256_bytes(data), "decision": report["decision"]}


def publish(output: Path, report: dict[str, Any]) -> bytes:
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


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = discovery_core.external_file(root, arguments.manifest, "manifest")
    output = discovery_core.external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "BroadbandIndependentHoldoutObservationProtocolFrozen",
        "claim": (
            "ONE_SYNCHRONIZED_LISTENER_GROUP_PROTOCOL_PREFLIGHT / ZERO_NEW_"
            "NETWORK_OR_MEMBER_PAYLOAD_ACCESS / NO_PHYSICS_QUALITY_ADMISSION_"
            "OR_RUNTIME_CREDIT"
        ),
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "discovery": validate_discovery(base, manifest),
        "candidate_parent": validate_candidate_parent(base, manifest),
        "bound_sources": validate_sources(root, manifest),
        "object": manifest["object"],
        "entries": manifest["entries"],
        "requests": manifest["requests"],
        "decode": manifest["decode"],
        "candidate": manifest["candidate"],
        "runtime": manifest["runtime"],
        "network_requests": 0,
        "object_payload_bytes_read": 0,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_admission_or_runtime_credit": False,
        "next_action": (
            "implement and commit a repeated zero-access execution preflight "
            "before the exact four-request acquisition"
        ),
    }
    report_bytes = publish(output, report)
    print(f"REALIMPACT broad-band holdout protocol: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("object payload bytes read: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
