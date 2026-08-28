#!/usr/bin/env python3
"""Freeze payload access and candidate rules for the REALIMPACT metal batch."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
from pathlib import Path
from typing import Any

MANIFEST_SCHEMA = "nextengine.experimental-realimpact-metal-batch-protocol.manifest.v1"
PREFLIGHT_SCHEMA = (
    "nextengine.experimental-realimpact-metal-batch-protocol-preflight.report.v1"
)
STUDY_ID = "physical-sound-realimpact-object-grouped-metal-batch"
REVISION = "payload-decoder-and-residual-candidates-v1"
SAMPLE_RATE_HZ = 48_000
AUDIO_PREFIX_BYTES = 33_554_432
NPY_HEADER_BYTES = 128
ANALYSIS_ROWS = list(range(15))
ANALYSIS_SAMPLES = 60_000
CALIBRATION_STOP_SAMPLE = 8_192
EVALUATION_WINDOWS = [[8_192, 16_384], [16_384, 32_768]]

DISCOVERY_REFERENCES = [
    {
        "label": "discovery_manifest",
        "path": ("../ps2-realimpact-metal-batch-discovery-v1/manifest.json"),
        "sha256": ("873d82e3ea887f89c05fbb9ada924b43f6087062c7394b1be047e2ca54e23ae4"),
        "bytes": 9_011,
    },
    {
        "label": "discovery_preflight",
        "path": ("../ps2-realimpact-metal-batch-discovery-v1/preflight-a/report.json"),
        "sha256": ("578e143ca137ead516c43d5320df986b4926b3855662322c5325c69ae3eaec31"),
        "bytes": 7_255,
        "required_decision": "RealImpactMetalBatchRolesAndDiscoveryInputsFrozen",
    },
    {
        "label": "discovery_acquisition",
        "path": ("../ps2-realimpact-metal-batch-discovery-v1/acquisition/report.json"),
        "sha256": ("d0a051ebbe39dffdf4ae8f01a4109a439c4f0a6e89b89d7933e85829a34b0468"),
        "bytes": 22_037,
        "required_decision": "RealImpactMetalBatchArchivesDiscovered",
    },
    {
        "label": "discovery_audit",
        "path": ("../ps2-realimpact-metal-batch-discovery-v1/audit-a/report.json"),
        "sha256": ("5790f8cc1db0fc5472915b89a0a8236611c274d2c5cc3a817e9ed4d0152a1219"),
        "bytes": 5_767,
        "required_decision": "RealImpactMetalBatchDiscoveryCacheVerified",
    },
]

SOURCE_REFERENCES = [
    {
        "path": "lab/scripts/physical_sound_broadband_common_pole_control.py",
        "sha256": ("317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d"),
    },
    {
        "path": "lab/scripts/physical_sound_dense_broadband_scaling_control.py",
        "sha256": ("8e4cc18ba25f90adb43483e5d4ff1ea7be46869e55c28ebd7c9bb5831b29fbbe"),
    },
    {
        "path": (
            "lab/scripts/physical_sound_realimpact_broadband_v2_counterfactual.py"
        ),
        "sha256": ("3f1c07b501dec97645cc0f1ad2fec5b3960312eacde88a291b64cc740a056aa5"),
    },
    {
        "path": "lab/scripts/physical_sound_modal_observation_registry.py",
        "sha256": ("a958c7955f5194046398c02d8cc1bd308e9acae0fb4f6ee5455c7327c07c064b"),
    },
]

PARTITION = {
    "development": ["17_IronSkillet", "67_IronPlate", "90_MetalLadle"],
    "calibration": ["43_IronMortar", "86_MetalHoledSpoon"],
    "holdout": ["91_MetalSpoon"],
    "shadow": ["89_MetalSpatula", "92_MetalSpatula"],
    "family_group_disjoint": True,
    "opened_seed_objects_forbidden_in_holdout_or_shadow": True,
    "statistical_release_credit": False,
}

OBJECTS = [
    {
        "dataset_object_id": "86_MetalHoledSpoon",
        "family_group": "metal_holed_spoon",
        "role": "calibration",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/"
                "86_MetalHoledSpoon.zip"
            ),
            "bytes": 2_310_510_455,
            "etag": '"6433e1b7-89b79777"',
            "last_modified_http": "Mon, 10 Apr 2023 10:15:19 GMT",
        },
        "audio_entry": {
            "name": "86_MetalHoledSpoon/preprocessed/deconvolved_0db.npy",
            "crc32": "0620272f",
            "compressed_bytes": 2_308_176_023,
            "uncompressed_bytes": 2_504_832_128,
            "local_offset": 366,
            "data_offset": 475,
            "shape": [3_000, 208_736],
        },
        "metadata_entries": [
            ["distance.npy", "570b48dd", 294, 24_128, 2_310_507_171],
            ["angle.npy", "fabb9a2e", 292, 24_128, 2_310_507_567],
            ["vertexID.npy", "13e1b8bf", 162, 24_128, 2_310_507_958],
            ["micID.npy", "082ec0c6", 225, 24_128, 2_310_508_222],
        ],
        "metadata_ranges": [[2_310_507_171, 2_310_508_545]],
    },
    {
        "dataset_object_id": "91_MetalSpoon",
        "family_group": "metal_spoon",
        "role": "holdout",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/91_MetalSpoon.zip"
            ),
            "bytes": 2_312_590_327,
            "etag": '"6433e360-89d753f7"',
            "last_modified_http": "Mon, 10 Apr 2023 10:22:24 GMT",
        },
        "audio_entry": {
            "name": "91_MetalSpoon/preprocessed/deconvolved_0db.npy",
            "crc32": "32a0ec83",
            "compressed_bytes": 2_310_790_746,
            "uncompressed_bytes": 2_503_020_128,
            "local_offset": 157,
            "data_offset": 261,
            "shape": [3_000, 208_585],
        },
        "metadata_entries": [
            ["vertexID.npy", "56bb5371", 166, 24_128, 2_310_791_007],
            ["angle.npy", "fabb9a2e", 292, 24_128, 2_310_791_270],
            ["distance.npy", "570b48dd", 294, 24_128, 2_312_588_312],
            ["micID.npy", "082ec0c6", 225, 24_128, 2_312_588_703],
        ],
        "metadata_ranges": [
            [2_310_791_007, 2_310_791_655],
            [2_312_588_312, 2_312_589_021],
        ],
    },
    {
        "dataset_object_id": "89_MetalSpatula",
        "family_group": "metal_spatula",
        "role": "shadow",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/"
                "89_MetalSpatula.zip"
            ),
            "bytes": 2_308_676_719,
            "etag": '"6433e245-899b9c6f"',
            "last_modified_http": "Mon, 10 Apr 2023 10:17:41 GMT",
        },
        "audio_entry": {
            "name": "89_MetalSpatula/preprocessed/deconvolved_0db.npy",
            "crc32": "80350931",
            "compressed_bytes": 2_306_531_327,
            "uncompressed_bytes": 2_503_368_128,
            "local_offset": 2_140_544,
            "data_offset": 2_140_650,
            "shape": [3_000, 208_614],
        },
        "metadata_entries": [
            ["vertexID.npy", "f7e3298b", 161, 24_128, 705],
            ["distance.npy", "570b48dd", 294, 24_128, 965],
            ["micID.npy", "082ec0c6", 225, 24_128, 1_554],
            ["angle.npy", "fabb9a2e", 292, 24_128, 2_308_671_977],
        ],
        "metadata_ranges": [
            [705, 1_357],
            [1_554, 1_874],
            [2_308_671_977, 2_308_672_364],
        ],
    },
    {
        "dataset_object_id": "92_MetalSpatula",
        "family_group": "metal_spatula",
        "role": "shadow",
        "archive": {
            "url": (
                "https://downloads.cs.stanford.edu/viscam/RealImpact/"
                "92_MetalSpatula.zip"
            ),
            "bytes": 2_313_811_870,
            "etag": '"6433e3e8-89e9f79e"',
            "last_modified_http": "Mon, 10 Apr 2023 10:24:40 GMT",
        },
        "audio_entry": {
            "name": "92_MetalSpatula/preprocessed/deconvolved_0db.npy",
            "crc32": "5d422cab",
            "compressed_bytes": 2_311_492_574,
            "uncompressed_bytes": 2_503_008_128,
            "local_offset": 2_317_861,
            "data_offset": 2_317_967,
            "shape": [3_000, 208_584],
        },
        "metadata_entries": [
            ["angle.npy", "fabb9a2e", 292, 24_128, 357],
            ["distance.npy", "570b48dd", 294, 24_128, 745],
            ["micID.npy", "082ec0c6", 225, 24_128, 1_717_010],
            ["vertexID.npy", "6ee2a866", 165, 24_128, 2_317_597],
        ],
        "metadata_ranges": [
            [357, 1_137],
            [1_717_010, 1_717_330],
            [2_317_597, 2_317_860],
        ],
    },
]

BAND_EDGES_HZ = [0, 375, 750, 1_500, 3_000, 6_000, 9_000, 12_000, 24_000]
DECAY_GRID_PER_SECOND = [10, 20, 40, 80, 160, 320, 640, 1_280]

CANDIDATES = [
    {
        "candidate_id": "modal_only_common_pole_v2",
        "kind": "baseline",
        "definition": {
            "partial_svd_rank": 7,
            "opened_region_ranking": "forbidden",
            "all_input_derived_regions_analyzed": True,
            "amplitude_calibration_stop_sample": CALIBRATION_STOP_SAMPLE,
            "prediction_windows": EVALUATION_WINDOWS,
        },
    },
    {
        "candidate_id": "modal_plus_parametric_transient_v0",
        "kind": "candidate",
        "definition": {
            "input": "modal_only_residual",
            "band_edges_hz": BAND_EDGES_HZ,
            "fit_samples": [0, 4_096],
            "render_stop_sample": 8_192,
            "envelope": "one_nonnegative_exponential_per_band",
            "decay_grid_per_second": DECAY_GRID_PER_SECOND,
            "spatial_gain": "per_listener_band_rms_ratio_on_fit_samples",
            "excitation": "sha256_counter_box_muller_f64",
            "seed_fields": ["manifest_sha256", "dataset_object_id", "band_index"],
        },
    },
    {
        "candidate_id": "modal_plus_seeded_subband_residual_v0",
        "kind": "candidate",
        "definition": {
            "input": "modal_only_residual",
            "band_edges_hz": BAND_EDGES_HZ,
            "fit_samples": [0, CALIBRATION_STOP_SAMPLE],
            "render_stop_sample": 32_768,
            "envelope": (
                "nonnegative_two_exponential_sum_per_band_with_lexicographic_"
                "minimum_sse_tie_break"
            ),
            "decay_grid_per_second": DECAY_GRID_PER_SECOND,
            "spatial_gain": "per_listener_band_rms_ratio_on_fit_samples",
            "excitation": "sha256_counter_box_muller_f64",
            "seed_fields": ["manifest_sha256", "dataset_object_id", "band_index"],
        },
    },
]

METRICS = {
    "gating": [
        "log_band_envelope_mae_db",
        "listener_window_energy_mae_db",
        "spectral_flatness_absolute_error_db",
    ],
    "diagnostic_only": ["waveform_nrmse", "damped_to_undamped_error_ratio"],
    "frame_samples": 512,
    "hop_samples": 256,
    "relative_floor_db": -80.0,
}

SELECTION_GATES = {
    "calibration": {
        "maximum_median_log_band_envelope_error_ratio": 0.85,
        "maximum_median_listener_energy_error_ratio": 0.95,
        "maximum_median_flatness_error_ratio": 0.95,
        "maximum_any_object_metric_ratio": 1.10,
        "minimum_improved_object_fraction": 0.50,
        "tie_break": ("lexicographic by the three aggregate ratios then candidate_id"),
    },
    "holdout": {
        "maximum_each_gating_metric_ratio": 0.95,
        "maximum_any_gating_metric_ratio": 1.05,
    },
    "shadow_family": {
        "maximum_family_median_each_gating_metric_ratio": 0.95,
        "maximum_any_object_gating_metric_ratio": 1.10,
    },
    "quality_domain_runtime_admission": False,
    "fallback_on_any_failure": "authored_clip_required",
}


class ProtocolError(RuntimeError):
    """A frozen batch payload or candidate protocol invariant changed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["manifest", "preflight"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
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


def access_summary() -> dict[str, Any]:
    metadata_ranges = [value for item in OBJECTS for value in item["metadata_ranges"]]
    return {
        "maximum_network_requests": len(metadata_ranges) + len(OBJECTS),
        "metadata_ranges": len(metadata_ranges),
        "metadata_range_bytes": sum(end - start + 1 for start, end in metadata_ranges),
        "audio_prefixes": len(OBJECTS),
        "audio_prefix_bytes_each": AUDIO_PREFIX_BYTES,
        "audio_prefix_bytes_total": AUDIO_PREFIX_BYTES * len(OBJECTS),
        "retry_allowed": False,
        "prefix_growth_allowed": False,
        "object_substitution_allowed": False,
    }


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_metal_batch_protocol.py",
            "sha256": runner_sha256,
        },
        "discovery": DISCOVERY_REFERENCES,
        "sources": SOURCE_REFERENCES,
        "partition": PARTITION,
        "objects": OBJECTS,
        "access": access_summary(),
        "decode": {
            "audio_dtype": "<f4",
            "npy_header_bytes": NPY_HEADER_BYTES,
            "decoded_rows": ANALYSIS_ROWS,
            "analysis_samples": ANALYSIS_SAMPLES,
            "metadata_dtype": "<i8",
            "metadata_shape": [3_000],
            "required_metadata": [
                "angle.npy",
                "distance.npy",
                "micID.npy",
                "vertexID.npy",
            ],
            "condition_rule": (
                "rows 0..14 must share angle, distance and vertex and expose "
                "ordered microphone ids 0..14"
            ),
        },
        "candidates": CANDIDATES,
        "metrics": METRICS,
        "selection_gates": SELECTION_GATES,
        "leakage_policy": {
            "candidate_selection_roles": ["development", "calibration"],
            "candidate_parameters_fit_roles": ["development", "calibration"],
            "holdout_opened_after_single_candidate_selection": True,
            "shadow_opened_after_holdout_report_frozen": True,
            "threshold_tuning_after_holdout_or_shadow": False,
            "per_object_human_admission": False,
        },
        "data_policy": {
            "new_payload_allowed_only_by_exact_ranges": True,
            "planter_payload_allowed": False,
            "physics_solver_allowed": False,
            "quality_domain_or_runtime_admission_allowed": False,
            "authored_clip_fallback_required": True,
        },
        "stop_rule": (
            "freeze a repeated execution preflight before network access; stop "
            "without retry or prefix growth on any identity, decode, method, "
            "selection, holdout or shadow failure; never promote quality, "
            "domain or runtime credit"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 256 * 1024:
        raise ProtocolError("metal batch protocol manifest exceeds 256 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ProtocolError(f"parse metal batch protocol manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ProtocolError("metal batch payload/candidate manifest changed")
    return data, manifest


def resolve_external(base: Path, reference: dict[str, Any]) -> tuple[bytes, Path]:
    path = (base / reference["path"]).resolve(strict=True)
    store = base.parent.resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(store):
        raise ProtocolError(f"discovery reference escapes store: {path}")
    data = path.read_bytes()
    if len(data) != reference["bytes"] or sha256_bytes(data) != reference["sha256"]:
        raise ProtocolError(f"discovery reference changed: {reference['label']}")
    if required := reference.get("required_decision"):
        try:
            report = json.loads(data)
        except json.JSONDecodeError as error:
            raise ProtocolError(
                f"parse discovery reference: {reference['label']}"
            ) from error
        if report.get("decision") != required:
            raise ProtocolError(f"discovery decision changed: {reference['label']}")
        if (
            report.get("new_object_member_payload_bytes_read", 0) != 0
            or report.get("quality_domain_or_runtime_admission", False) is not False
        ):
            raise ProtocolError(f"discovery boundary changed: {reference['label']}")
    return data, path


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    result = []
    for reference in manifest["sources"]:
        path = (root / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != reference["sha256"]
        ):
            raise ProtocolError(f"bound source changed: {reference['path']}")
        result.append(
            {
                "path": reference["path"],
                "sha256": reference["sha256"],
                "bytes": path.stat().st_size,
            }
        )
    return result


def validate_protocol_shape(manifest: dict[str, Any]) -> None:
    object_ids = [item["dataset_object_id"] for item in manifest["objects"]]
    if object_ids != [
        "86_MetalHoledSpoon",
        "91_MetalSpoon",
        "89_MetalSpatula",
        "92_MetalSpatula",
    ]:
        raise ProtocolError("new object order changed")
    for item in manifest["objects"]:
        audio = item["audio_entry"]
        if (
            audio["uncompressed_bytes"]
            != NPY_HEADER_BYTES + audio["shape"][0] * audio["shape"][1] * 4
            or audio["shape"][0] != 3_000
            or audio["shape"][1] < ANALYSIS_SAMPLES
            or audio["data_offset"] + AUDIO_PREFIX_BYTES
            > audio["data_offset"] + audio["compressed_bytes"]
        ):
            raise ProtocolError(
                f"audio shape/range changed: {item['dataset_object_id']}"
            )
        ranges = item["metadata_ranges"]
        if len(item["metadata_entries"]) != 4 or any(
            start < 0 or end < start or end >= item["archive"]["bytes"]
            for start, end in ranges
        ):
            raise ProtocolError(f"metadata access changed: {item['dataset_object_id']}")
        audio_payload_end = audio["data_offset"] + audio["compressed_bytes"] - 1
        if any(
            not (end < audio["data_offset"] or start > audio_payload_end)
            for start, end in ranges
        ):
            raise ProtocolError(
                f"metadata range overlaps audio payload: {item['dataset_object_id']}"
            )
    if access_summary() != manifest["access"]:
        raise ProtocolError("access summary changed")


def expected_preflight(
    manifest_bytes: bytes,
    runner_sha256: str,
    discovery: list[dict[str, Any]],
    sources: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "schema": PREFLIGHT_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "decision": "RealImpactMetalBatchPayloadAndCandidateProtocolFrozen",
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "discovery": discovery,
        "sources": sources,
        "partition": PARTITION,
        "objects": [
            {
                "dataset_object_id": item["dataset_object_id"],
                "family_group": item["family_group"],
                "role": item["role"],
                "shape": item["audio_entry"]["shape"],
                "metadata_ranges": item["metadata_ranges"],
                "audio_prefix_range": [
                    item["audio_entry"]["data_offset"],
                    item["audio_entry"]["data_offset"] + AUDIO_PREFIX_BYTES - 1,
                ],
            }
            for item in OBJECTS
        ],
        "access": access_summary(),
        "candidate_ids": [item["candidate_id"] for item in CANDIDATES],
        "gating_metrics": METRICS["gating"],
        "selection_gates": SELECTION_GATES,
        "network_requests": 0,
        "new_member_payload_bytes_read": 0,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_domain_or_runtime_admission": False,
        "next_action": (
            "implement and freeze a repeated execution preflight; then acquire "
            "only the exact metadata and audio-prefix ranges"
        ),
    }


def write_new_file(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise ProtocolError(f"refusing to replace existing output: {path}")
    descriptor, staging_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    staging = Path(staging_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(staging, path)
    finally:
        staging.unlink(missing_ok=True)


def main() -> None:
    arguments = parse_arguments()
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "manifest":
        if arguments.manifest is not None:
            raise ProtocolError("manifest stage accepts only --output")
        write_new_file(
            arguments.output, canonical_json(expected_manifest(runner_sha256))
        )
        return
    if arguments.manifest is None:
        raise ProtocolError("preflight stage requires --manifest")
    manifest_bytes, manifest = load_manifest(arguments.manifest, runner_sha256)
    root = Path(__file__).resolve().parents[2]
    validate_protocol_shape(manifest)
    discovery = []
    for reference in manifest["discovery"]:
        data, path = resolve_external(arguments.manifest.parent, reference)
        discovery.append(
            {
                "label": reference["label"],
                "path": reference["path"],
                "sha256": sha256_bytes(data),
                "bytes": path.stat().st_size,
                "decision": reference.get("required_decision"),
            }
        )
    sources = validate_sources(root, manifest)
    report = expected_preflight(
        manifest_bytes,
        runner_sha256,
        discovery,
        sources,
    )
    write_new_file(arguments.output, canonical_json(report))


if __name__ == "__main__":
    main()
