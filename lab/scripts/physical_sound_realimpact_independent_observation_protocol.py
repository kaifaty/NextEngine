#!/usr/bin/env python3
"""Freeze the independent REALIMPACT one-impact observation protocol."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from typing import Any

import numpy as np
import scipy

import physical_sound_realimpact_independent_observation_discovery as independent
import physical_sound_realimpact_observation_discovery as discovery_core


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-independent-observation-protocol.manifest.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-independent-observation-protocol-"
    "preflight.report.v1"
)
STUDY_ID = "physical-sound-realimpact-independent-observation-discriminator"
REVISION = "iron-skillet-one-impact-salience-observation-v1"
OBJECT_ID = independent.OBJECT_ID

TOTAL_ROWS = 3_000
ROW_COUNT = 600
REFERENCE_ROW = 7
SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 230_549
SAMPLE_BYTES = 4
ROW_BYTES = SAMPLE_COUNT * SAMPLE_BYTES
DECODED_BYTES = ROW_COUNT * ROW_BYTES
NPY_HEADER_BYTES = 128
PREFIX_BYTES = 536_870_912

EARLY_CONDITION_RANGE = [159, 865]
DISTANCE_RANGE = [611_263, 611_654]
VERTEX_ID_RANGE = [2_393_992_534, 2_393_992_794]
AUDIO_PREFIX_RANGE = [2_938_027, 2_938_027 + PREFIX_BYTES - 1]

DISCOVERY_MANIFEST_SHA256 = (
    "a77f6d9750890af5bddbcc3d53bf79ae14d67f55f5c104a7fdaf22673599357f"
)
DISCOVERY_ACQUISITION_SHA256 = (
    "b9f659b42e42137d80d456c8a87e8ddd771b8182490e40b27c72c33b69b16f6c"
)
DISCOVERY_AUDIT_SHA256 = (
    "a3bd84f1ccb743ecbee3d68b88e9bfeb309fa8351204db20a1a787f2538a8dab"
)
DISCOVERY_TAIL_SHA256 = (
    "d5626b4cdaf2c1f136710d5dbb17e487b5385411ef04abf940c8652ca666752a"
)
DISCOVERY_AUDIO_HEADER_SHA256 = (
    "cd26f6df95bd5acfbc0b4133abd16fede81d723b2586d33a0b29e05e31af0f64"
)
DISCOVERY_RUNNER_SHA256 = (
    "27b11de8c97f2ba7ee92d2b1c0dce02cee1ea732dc9a31e9426a5ffe181f70f1"
)
DISCOVERY_CORE_SHA256 = (
    "3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74"
)
SALIENCE_REPORT_SHA256 = (
    "fa94071059c3e3f6dd3e71eed187affe08c0ca36c120cd483b8c2e8da9560b3e"
)
SALIENCE_RUNNER_SHA256 = (
    "62043142a91e5bd4fe815a0b11c76678935a867d57dae90262336c59b5004ed2"
)
ADAPTIVE_RUNNER_SHA256 = (
    "fe59b3d6c8845ed9433bab6f225625d53485f4245e7dac5968c4952eb8459c20"
)
MULTIOUTPUT_RUNNER_SHA256 = (
    "f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e"
)
EXTRACTOR_RUNNER_SHA256 = (
    "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
)

ENTRIES = {
    "angle.npy": {
        "name": f"{OBJECT_ID}/preprocessed/angle.npy",
        "crc32": "fabb9a2e",
        "compressed_bytes": 292,
        "uncompressed_bytes": 24_128,
        "local_offset": 159,
        "method": 8,
    },
    "micID.npy": {
        "name": f"{OBJECT_ID}/preprocessed/micID.npy",
        "crc32": "082ec0c6",
        "compressed_bytes": 225,
        "uncompressed_bytes": 24_128,
        "local_offset": 546,
        "method": 8,
    },
    "distance.npy": {
        "name": f"{OBJECT_ID}/preprocessed/distance.npy",
        "crc32": "570b48dd",
        "compressed_bytes": 294,
        "uncompressed_bytes": 24_128,
        "local_offset": 611_263,
        "method": 8,
    },
    "vertexID.npy": {
        "name": f"{OBJECT_ID}/preprocessed/vertexID.npy",
        "crc32": "9ab5b6ac",
        "compressed_bytes": 163,
        "uncompressed_bytes": 24_128,
        "local_offset": 2_393_992_534,
        "method": 8,
    },
    "deconvolved_0db.npy": {
        "name": f"{OBJECT_ID}/preprocessed/deconvolved_0db.npy",
        "crc32": "3669823f",
        "compressed_bytes": 2_391_054_312,
        "uncompressed_bytes": 2_766_588_128,
        "local_offset": 2_937_922,
        "data_offset": 2_938_027,
        "method": 8,
    },
}

GATES = {
    "minimum_selected_modes": 6,
    "maximum_selected_modes": 16,
    "minimum_onset_to_tail_persistence_recall": 0.50,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_valid_adaptive_fit_fraction": 0.75,
    "minimum_decaying_mode_fraction": 0.50,
    "minimum_median_fit_r_squared": 0.95,
    "scale_invariant_bin_selection": True,
}


class ProtocolError(RuntimeError):
    """The independent observation protocol or its frozen lineage changed."""


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
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "development",
            "impact_ordinal": 0,
            "rows": [0, ROW_COUNT - 1],
            "reference_row": REFERENCE_ROW,
            "prior_observation_payload_bytes_read": 0,
            "planter_payload_access_allowed": False,
        },
        "archive": {
            "url": independent.ARCHIVE_URL,
            "bytes": independent.ARCHIVE_BYTES,
            "etag": independent.ARCHIVE_ETAG,
            "last_modified_http": independent.ARCHIVE_LAST_MODIFIED,
        },
        "discovery": {
            "runner": {
                "path": (
                    "lab/scripts/"
                    "physical_sound_realimpact_independent_observation_discovery.py"
                ),
                "sha256": DISCOVERY_RUNNER_SHA256,
            },
            "core": {
                "path": "lab/scripts/physical_sound_realimpact_observation_discovery.py",
                "sha256": DISCOVERY_CORE_SHA256,
            },
            "manifest": {
                "path": "discovery-manifest.json",
                "sha256": DISCOVERY_MANIFEST_SHA256,
            },
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
                "bytes": independent.TAIL_BYTES,
            },
            "audio_local_header": {
                "path": "discovery-acquisition/audio-local-header.bin",
                "sha256": DISCOVERY_AUDIO_HEADER_SHA256,
                "bytes": independent.LOCAL_HEADER_BYTES,
            },
        },
        "entries": ENTRIES,
        "requests": {
            "maximum_network_requests": 4,
            "early_condition_range": EARLY_CONDITION_RANGE,
            "distance_range": DISTANCE_RANGE,
            "vertex_id_range": VERTEX_ID_RANGE,
            "audio_prefix_range": AUDIO_PREFIX_RANGE,
            "audio_prefix_bytes": PREFIX_BYTES,
            "metadata_bytes": 1_360,
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
            "salience": {
                "path": "lab/scripts/physical_sound_salience_selector_control.py",
                "sha256": SALIENCE_RUNNER_SHA256,
                "revision": "relative-spatial-persistence-salience-v1",
                "parent_report": {
                    "path": "../ps2-salience-selector-control-v1/run-a/report.json",
                    "sha256": SALIENCE_REPORT_SHA256,
                },
            },
            "adaptive": {
                "path": "lab/scripts/physical_sound_adaptive_decay_control.py",
                "sha256": ADAPTIVE_RUNNER_SHA256,
                "revision": "spatial-adaptive-rms-envelope-v1",
            },
            "diagnostic_comparator": {
                "path": "lab/scripts/physical_sound_multioutput_decay_control.py",
                "sha256": MULTIOUTPUT_RUNNER_SHA256,
                "admission_role": False,
            },
            "reference_extractor": {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": EXTRACTOR_RUNNER_SHA256,
            },
            "scale_factors": [0.125, 1.0, 8.0],
            "gates": GATES,
        },
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
            "implement and hash-close an execution runner before access; then stop with "
            "authored-clip fallback if acquisition, decode, row identity, scale "
            "invariance or any unchanged observation gate fails; no retry, prefix "
            "growth, object substitution, physics or Planter access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ProtocolError("independent observation manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ProtocolError(f"parse independent observation manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ProtocolError("independent observation manifest changed")
    return data, manifest


def resolve_reference(
    base: Path, ref: dict[str, Any], label: str
) -> tuple[bytes, Path]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise ProtocolError(f"{label} escapes physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"] or (
        "bytes" in ref and len(data) != ref["bytes"]
    ):
        raise ProtocolError(f"{label} identity changed")
    return data, path


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [
        manifest["discovery"]["runner"],
        manifest["discovery"]["core"],
        manifest["candidate"]["salience"],
        manifest["candidate"]["adaptive"],
        manifest["candidate"]["diagnostic_comparator"],
        manifest["candidate"]["reference_extractor"],
    ]
    result = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_file() or not path.is_relative_to(root):
            raise ProtocolError(f"source escapes repository: {ref['path']}")
        if sha256_file(path) != ref["sha256"]:
            raise ProtocolError(f"bound source changed: {ref['path']}")
        result.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return result


def validate_discovery(base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    refs = manifest["discovery"]
    manifest_bytes, _ = resolve_reference(base, refs["manifest"], "discovery manifest")
    acquisition_bytes, _ = resolve_reference(
        base, refs["acquisition_report"], "discovery acquisition"
    )
    audit_bytes, _ = resolve_reference(base, refs["audit_report"], "discovery audit")
    tail, _ = resolve_reference(base, refs["tail"], "discovery tail")
    header, _ = resolve_reference(
        base, refs["audio_local_header"], "observation local header"
    )
    acquisition = json.loads(acquisition_bytes)
    audit = json.loads(audit_bytes)
    independent.configure_core()
    central, entries = discovery_core.discover_from_tail(tail)
    audio = next(
        entry for entry in entries if entry["name"] == independent.EXPECTED_AUDIO_ENTRY
    )
    audio_header = discovery_core.validate_audio_local_header(header, audio)
    if (
        acquisition.get("decision")
        != "IndependentObjectArchiveAndObservationEntryDiscovered"
        or acquisition.get("runner_sha256") != DISCOVERY_RUNNER_SHA256
        or acquisition.get("network_requests") != 2
        or acquisition.get("audio_payload_bytes_read") != 0
        or acquisition.get("planter_audio_payload_bytes_read") != 0
        or acquisition.get("central_directory") != central
        or acquisition.get("entries") != entries
        or acquisition.get("observation_local_header") != audio_header
        or audit.get("decision") != "IndependentObservationDiscoveryCacheVerified"
        or audit.get("acquisition_report_sha256") != DISCOVERY_ACQUISITION_SHA256
        or audit.get("network_requests") != 0
        or audit.get("audio_payload_bytes_read") != 0
    ):
        raise ProtocolError("independent discovery lineage changed")
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


def validate_salience_parent(base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    ref = manifest["candidate"]["salience"]["parent_report"]
    data, _ = resolve_reference(base, ref, "salience parent")
    report = json.loads(data)
    if (
        report.get("decision") != "SalienceSelectorSyntheticControlSupported"
        or report.get("gate", {}).get("passed") is not True
        or report.get("runner_sha256") != SALIENCE_RUNNER_SHA256
        or report.get("real_payload_bytes_read") != 0
        or report.get("planter_payload_bytes_read") != 0
    ):
        raise ProtocolError("salience parent lineage changed")
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
    manifest_path = independent.discovery.external_file(root, arguments.manifest, "manifest")
    output = independent.discovery.external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "IndependentOneImpactObservationProtocolFrozen",
        "claim": (
            "ONE_IMPACT_OBSERVATION_PROTOCOL_PREFLIGHT / ZERO_NEW_NETWORK_OR_MEMBER_"
            "PAYLOAD_ACCESS / NO_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "discovery": validate_discovery(base, manifest),
        "salience_parent": validate_salience_parent(base, manifest),
        "bound_sources": validate_sources(root, manifest),
        "frozen_requests": manifest["requests"],
        "frozen_decode": manifest["decode"],
        "frozen_candidate": manifest["candidate"],
        "runtime": manifest["runtime"],
        "network_requests": 0,
        "object_payload_bytes_read": 0,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_admission_or_runtime_credit": False,
        "next_action": (
            "implement and hash-close the execution runner; rerun protocol preflight "
            "before any of the four frozen requests"
        ),
    }
    report_bytes = publish(output, report)
    print(f"Independent observation protocol preflight: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("object payload bytes read: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
