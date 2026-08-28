#!/usr/bin/env python3
"""Run the frozen broad-band common-pole candidate on existing Iron rows."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
from typing import Any

import numpy as np
import scipy

import physical_sound_broadband_common_pole_control as broadband


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-broadband-counterfactual.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-broadband-counterfactual-"
        "preflight.report.v1"
    ),
    "analyze": (
        "nextengine.experimental-realimpact-broadband-counterfactual.report.v1"
    ),
}
STUDY_ID = "physical-sound-realimpact-broadband-counterfactual"
REVISION = "iron-skillet-broadband-common-pole-counterfactual-v1"

BROADBAND_RUNNER_SHA256 = (
    "317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d"
)
BROADBAND_REPORT_SHA256 = (
    "dcd832525fbd1955c9122fc52579e4a8ddd10ab76bc5d79393f8f9df9d270094"
)
BROADBAND_RESULT_DOC_SHA256 = (
    "08a83bed456ebe2f963924303f226170a5bb145d964920d34463259f021b736d"
)
IRON_DECODE_REPORT_SHA256 = (
    "47acdc35449390f639cda34990520c5f74cb8627d0932079345ca35735ffc099"
)
IRON_BLOCK_SHA256 = (
    "e26d1df1d547d059bbcbc57453c522b8c23402461feec09ff6149bc37a9c7eeb"
)
IRON_DIAGNOSTIC_REPORT_SHA256 = (
    "02551f2995764197575d5dd30bb36bd44da5452e2af79533b43922f44fec48d1"
)
IRON_DIAGNOSTIC_DOC_SHA256 = (
    "a236f27be0dd5264abed82dd2ea52e71dc34b929ccf1f6efcd10486dc75cf5e9"
)

SAMPLE_RATE_HZ = 48_000
SOURCE_ROW_COUNT = 600
SOURCE_SAMPLE_COUNT = 230_549
ANALYSIS_ROWS = list(range(15))
ONSET_SAMPLE = 29
OBSERVATION_SAMPLES = 60_000
PREDICTION_CALIBRATION_SAMPLES = 8_192
PREDICTION_WINDOWS = [
    [8_192, 16_384],
    [16_384, 32_768],
]
SPATIAL_PARTITIONS = {
    "even_microphones": list(range(0, 15, 2)),
    "odd_microphones": list(range(1, 15, 2)),
}
MATCH_TOLERANCE_CENTS = 40.0

GATES = {
    "minimum_region_count": 1,
    "maximum_region_count": 16,
    "maximum_analysis_bin_count": 48,
    "minimum_duplicate_estimates_removed": 1,
    "minimum_preprune_cluster_count": 6,
    "maximum_preprune_cluster_count": 64,
    "minimum_retained_mode_count": 6,
    "maximum_retained_mode_count": 32,
    "scale_invariant_region_bins": True,
    "minimum_even_partition_match_fraction": 0.50,
    "minimum_odd_partition_match_fraction": 0.50,
    "maximum_full_observed_nrmse": 0.95,
    "maximum_full_damped_to_undamped_nrmse_ratio": 0.95,
    "maximum_prediction_error_ratio_window_a": 0.95,
    "maximum_prediction_error_ratio_window_b": 0.95,
}


class CounterfactualError(RuntimeError):
    """The frozen counterfactual or one of its exact parents changed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
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
        "parents": {
            "broadband_runner": {
                "path": "lab/scripts/physical_sound_broadband_common_pole_control.py",
                "sha256": BROADBAND_RUNNER_SHA256,
            },
            "broadband_report": {
                "path": "../ps2-broadband-common-pole-control-v1/run-a/report.json",
                "sha256": BROADBAND_REPORT_SHA256,
            },
            "broadband_result_document": {
                "path": (
                    "docs/development/physical-sound-broadband-common-pole-"
                    "control-result-ps2-2026-08-28.md"
                ),
                "sha256": BROADBAND_RESULT_DOC_SHA256,
            },
            "iron_decode_report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-observation-v1/"
                    "observation-decode/report.json"
                ),
                "sha256": IRON_DECODE_REPORT_SHA256,
            },
            "iron_decoded_block": {
                "path": (
                    "../ps2-realimpact-iron-skillet-observation-v1/"
                    "observation-decode/"
                    "iron-skillet-impact000-rows000-599.f32le"
                ),
                "sha256": IRON_BLOCK_SHA256,
                "bytes": 553_317_600,
                "dtype": "<f4",
                "shape": [SOURCE_ROW_COUNT, SOURCE_SAMPLE_COUNT],
            },
            "iron_selector_tail_report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-selector-tail-diagnostic-v1/"
                    "analysis-a/report.json"
                ),
                "sha256": IRON_DIAGNOSTIC_REPORT_SHA256,
            },
            "iron_selector_tail_result_document": {
                "path": (
                    "docs/development/physical-sound-realimpact-selector-tail-"
                    "diagnostic-result-ps2-2026-08-28.md"
                ),
                "sha256": IRON_DIAGNOSTIC_DOC_SHA256,
            },
        },
        "observation": {
            "dataset_object_id": "17_IronSkillet",
            "impact_ordinal": 0,
            "role": "opened_development_counterfactual_only",
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "source_shape": [SOURCE_ROW_COUNT, SOURCE_SAMPLE_COUNT],
            "analysis_rows": ANALYSIS_ROWS,
            "onset_sample": ONSET_SAMPLE,
            "post_onset_observation_samples": OBSERVATION_SAMPLES,
            "sample_format": "f32le-row-major",
            "normalization": "none",
        },
        "candidate": {
            "id": broadband.REVISION,
            "source_sha256": BROADBAND_RUNNER_SHA256,
            "gabor": {
                "window": "scipy blackmanharris",
                "window_samples": broadband.WINDOW_SAMPLES,
                "hop_samples": broadband.HOP_SAMPLES,
                "fft_samples": broadband.WINDOW_SAMPLES,
                "frame_count": broadband.GABOR_FRAME_COUNT,
                "boundary": None,
                "padded": False,
            },
            "discovery": {
                "minimum_frequency_hz": broadband.MINIMUM_FREQUENCY_HZ,
                "maximum_frequency_hz": broadband.MAXIMUM_FREQUENCY_HZ,
                "relative_local_peak_floor_db": broadband.REGION_ENERGY_FLOOR_DB,
                "neighborhood_radius_bins": broadband.NEIGHBORHOOD_RADIUS_BINS,
                "maximum_region_count": GATES["maximum_region_count"],
                "maximum_analysis_bin_count": GATES[
                    "maximum_analysis_bin_count"
                ],
            },
            "common_poles": {
                "minimum_order_score_margin": (
                    broadband.MINIMUM_ORDER_SCORE_MARGIN
                ),
                "duplicate_frequency_hz": broadband.DUPLICATE_FREQUENCY_HZ,
                "maximum_preprune_cluster_count": GATES[
                    "maximum_preprune_cluster_count"
                ],
            },
            "amplitudes": {
                "fit": (
                    "joint all-output least-squares cosine/sine coefficients "
                    "over the frozen 60000-sample post-onset observation"
                ),
                "mode_energy_floor_db": broadband.MODE_ENERGY_FLOOR_DB,
                "pruning_order": "after pole estimation and amplitude fit",
            },
            "scale_factors": broadband.SCALE_FACTORS,
        },
        "controls": {
            "spatial_partitions": SPATIAL_PARTITIONS,
            "partition_matching_tolerance_cents": MATCH_TOLERANCE_CENTS,
            "partition_matching": (
                "injective minimum-cents matching of retained full-output "
                "frequencies to unpruned partition estimates"
            ),
            "undamped_ablation": (
                "same discovered frequencies and joint fit, every decay set to zero"
            ),
            "prediction": {
                "amplitude_calibration_samples": [
                    0,
                    PREDICTION_CALIBRATION_SAMPLES,
                ],
                "holdout_windows": PREDICTION_WINDOWS,
                "error_ratio": "damped squared error divided by undamped squared error",
            },
        },
        "gates": GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "existing_decoded_rows_only": True,
            "new_network_or_payload_access_allowed": False,
            "threshold_or_variant_tuning_after_preflight_allowed": False,
            "new_object_or_retry_allowed": False,
            "physics_solver_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish support or rejection twice on the hash-bound existing Iron "
            "rows; on failure do not tune; on support only freeze an independent "
            "internet-sourced real holdout before any domain or quality claim"
        ),
    }


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise CounterfactualError(
            f"{label} must be an external regular file: {resolved}"
        )
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise CounterfactualError(f"output must remain outside repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise CounterfactualError(f"output must be absent or empty: {resolved}")
    return resolved


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise CounterfactualError("counterfactual manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise CounterfactualError(f"parse counterfactual manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise CounterfactualError("counterfactual manifest changed")
    return data, manifest


def checked_store_file(base: Path, reference: dict[str, Any], label: str) -> Path:
    path = (base / reference["path"]).resolve(strict=True)
    store = base.parent.resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(store):
        raise CounterfactualError(f"{label} escapes physical-sound store")
    if sha256_file(path) != reference["sha256"]:
        raise CounterfactualError(f"{label} hash changed")
    return path


def checked_repo_file(
    root: Path, reference: dict[str, Any], label: str
) -> Path:
    path = (root / reference["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(root):
        raise CounterfactualError(f"{label} escapes repository")
    if sha256_file(path) != reference["sha256"]:
        raise CounterfactualError(f"{label} hash changed")
    return path


def validate_lineage(
    root: Path, base: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    parents = manifest["parents"]
    broad_runner = checked_repo_file(
        root, parents["broadband_runner"], "broad-band runner"
    )
    broad_doc = checked_repo_file(
        root, parents["broadband_result_document"], "broad-band result document"
    )
    iron_doc = checked_repo_file(
        root,
        parents["iron_selector_tail_result_document"],
        "Iron diagnostic result document",
    )
    broad_report_path = checked_store_file(
        base, parents["broadband_report"], "broad-band report"
    )
    decode_report_path = checked_store_file(
        base, parents["iron_decode_report"], "Iron decode report"
    )
    decoded_block_path = checked_store_file(
        base, parents["iron_decoded_block"], "Iron decoded block"
    )
    diagnostic_path = checked_store_file(
        base, parents["iron_selector_tail_report"], "Iron diagnostic report"
    )
    if decoded_block_path.stat().st_size != parents["iron_decoded_block"]["bytes"]:
        raise CounterfactualError("Iron decoded block size changed")

    broad_report = json.loads(broad_report_path.read_bytes())
    decode_report = json.loads(decode_report_path.read_bytes())
    diagnostic = json.loads(diagnostic_path.read_bytes())
    if (
        broad_report.get("decision")
        != "BroadbandCommonPoleSyntheticControlSupported"
        or broad_report.get("gate", {}).get("passed") is not True
        or broad_report.get("real_payload_bytes_read") != 0
        or broad_report.get("network_requests") != 0
        or broad_report.get("physics_solver_runs") != 0
        or broad_report.get("planter_payload_bytes_read") != 0
        or broad_report.get("quality_admission_or_runtime_credit") is not False
    ):
        raise CounterfactualError("broad-band synthetic parent is not admissible")
    if (
        decode_report.get("decision") != "IndependentImpactZeroObservationDecoded"
        or decode_report.get("dataset_object_id") != "17_IronSkillet"
        or decode_report.get("impact_ordinal") != 0
        or decode_report.get("decoded_sha256") != IRON_BLOCK_SHA256
        or decode_report.get("sample_count") != SOURCE_SAMPLE_COUNT
        or decode_report.get("row_count") != SOURCE_ROW_COUNT
        or decode_report.get("analysis_rows") != [0, 14]
        or decode_report.get("reference_row") != 7
        or decode_report.get("additional_audio_payload_bytes_read") != 0
        or decode_report.get("network_requests") != 0
    ):
        raise CounterfactualError("Iron decode lineage changed")
    if (
        diagnostic.get("decision") != "FixedTailTimingMismatchSupported"
        or diagnostic.get("onset_sample") != ONSET_SAMPLE
        or diagnostic.get("analysis_rows") != ANALYSIS_ROWS
        or diagnostic.get("parents", {}).get("decoded_block_sha256")
        != IRON_BLOCK_SHA256
        or diagnostic.get("additional_payload_bytes_read") != 0
        or diagnostic.get("network_requests") != 0
        or diagnostic.get("physics_solver_runs") != 0
        or diagnostic.get("planter_payload_bytes_read") != 0
        or diagnostic.get("quality_admission_or_runtime_credit") is not False
    ):
        raise CounterfactualError("Iron selector/tail lineage changed")
    return {
        "broadband_runner_sha256": sha256_file(broad_runner),
        "broadband_result_document_sha256": sha256_file(broad_doc),
        "broadband_report_sha256": sha256_file(broad_report_path),
        "broadband_decision": broad_report["decision"],
        "iron_decode_report_sha256": sha256_file(decode_report_path),
        "iron_decoded_block_sha256": sha256_file(decoded_block_path),
        "iron_decoded_block_bytes": decoded_block_path.stat().st_size,
        "iron_selector_tail_report_sha256": sha256_file(diagnostic_path),
        "iron_selector_tail_result_document_sha256": sha256_file(iron_doc),
        "iron_selector_tail_decision": diagnostic["decision"],
        "decoded_block_path": str(decoded_block_path),
    }


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": "17_IronSkillet",
        "impact_ordinal": 0,
        "role": "opened_development_counterfactual_only",
        "analysis_rows": ANALYSIS_ROWS,
        "onset_sample": ONSET_SAMPLE,
        "additional_payload_bytes_read": 0,
        "network_requests": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    lineage = validate_lineage(root, base, manifest)
    lineage.pop("decoded_block_path")
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "ExistingIronBroadbandCounterfactualFrozen",
        "claim": (
            "EXISTING_IRON_READ_ONLY_PREFLIGHT / NO_NEW_PAYLOAD_OBJECT_PHYSICS_"
            "PLANTER_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "lineage": lineage,
        "observation": manifest["observation"],
        "candidate": manifest["candidate"],
        "controls": manifest["controls"],
        "gates": manifest["gates"],
        "runtime": manifest["runtime"],
        "next_action": (
            "commit this preflight, then analyze the existing Iron rows twice"
        ),
    }


def load_observation(path: Path) -> np.ndarray:
    mapped = np.memmap(
        path,
        dtype="<f4",
        mode="r",
        shape=(SOURCE_ROW_COUNT, SOURCE_SAMPLE_COUNT),
        order="C",
    )
    stop = ONSET_SAMPLE + OBSERVATION_SAMPLES
    channels = np.asarray(mapped[ANALYSIS_ROWS, :stop], dtype=np.float64)
    if channels.shape != (len(ANALYSIS_ROWS), stop):
        raise CounterfactualError("Iron analysis slice shape changed")
    if not np.all(np.isfinite(channels)):
        raise CounterfactualError("Iron analysis slice contains non-finite values")
    return channels


def representatives(clusters: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [item["representative"] for item in clusters]


def design_matrix(
    modes: list[dict[str, Any]],
    start: int,
    stop: int,
    *,
    undamped: bool,
) -> np.ndarray:
    time = np.arange(start, stop, dtype=np.float64) / SAMPLE_RATE_HZ
    columns = []
    for mode in modes:
        decay = 0.0 if undamped else mode["decay_per_second"]
        envelope = np.exp(-decay * time)
        phase = 2.0 * math.pi * mode["frequency_hz"] * time
        columns.extend([envelope * np.cos(phase), envelope * np.sin(phase)])
    if not columns:
        raise CounterfactualError("cannot build a modal design with zero modes")
    return np.stack(columns, axis=1)


def fit_coefficients(
    channels: np.ndarray,
    modes: list[dict[str, Any]],
    fit_start: int,
    fit_stop: int,
    *,
    undamped: bool,
) -> np.ndarray:
    design = design_matrix(modes, fit_start, fit_stop, undamped=undamped)
    target = channels[:, fit_start:fit_stop].T
    return np.linalg.lstsq(design, target, rcond=None)[0]


def reconstruct(
    modes: list[dict[str, Any]],
    coefficients: np.ndarray,
    start: int,
    stop: int,
    *,
    undamped: bool,
    retained_indices: set[int] | None = None,
) -> np.ndarray:
    design = design_matrix(modes, start, stop, undamped=undamped)
    output = np.zeros((coefficients.shape[1], stop - start), dtype=np.float64)
    indices = (
        range(len(modes)) if retained_indices is None else sorted(retained_indices)
    )
    for index in indices:
        output += (
            design[:, 2 * index, None] * coefficients[2 * index][None, :]
            + design[:, 2 * index + 1, None]
            * coefficients[2 * index + 1][None, :]
        ).T
    return output


def fit_and_prune(
    post_onset: np.ndarray, clusters: list[dict[str, Any]]
) -> tuple[list[dict[str, Any]], np.ndarray, np.ndarray]:
    modes = representatives(clusters)
    coefficients = fit_coefficients(
        post_onset,
        modes,
        0,
        OBSERVATION_SAMPLES,
        undamped=False,
    )
    design = design_matrix(
        modes, 0, OBSERVATION_SAMPLES, undamped=False
    )
    energies = []
    for index in range(len(modes)):
        component = (
            design[:, 2 * index, None] * coefficients[2 * index][None, :]
            + design[:, 2 * index + 1, None]
            * coefficients[2 * index + 1][None, :]
        )
        energies.append(float(np.sum(component * component)))
    maximum = max(energies)
    reports = []
    retained_indices: set[int] = set()
    for index, (cluster, energy) in enumerate(
        zip(clusters, energies, strict=True)
    ):
        relative_db = 10.0 * math.log10(max(energy, 1.0e-300) / maximum)
        retained = relative_db >= broadband.MODE_ENERGY_FLOOR_DB
        if retained:
            retained_indices.add(index)
        amplitudes = []
        for channel in range(post_onset.shape[0]):
            value = complex(
                float(coefficients[2 * index, channel]),
                -float(coefficients[2 * index + 1, channel]),
            )
            amplitudes.append(
                {
                    "real": value.real,
                    "imaginary": value.imag,
                    "magnitude": abs(value),
                    "phase_radians": math.atan2(value.imag, value.real),
                }
            )
        reports.append(
            {
                **cluster,
                "mode_energy": energy,
                "relative_mode_energy_db": relative_db,
                "retained": retained,
                "complex_amplitudes": amplitudes,
            }
        )
    candidate = reconstruct(
        modes,
        coefficients,
        0,
        OBSERVATION_SAMPLES,
        undamped=False,
        retained_indices=retained_indices,
    )
    undamped_coefficients = fit_coefficients(
        post_onset,
        modes,
        0,
        OBSERVATION_SAMPLES,
        undamped=True,
    )
    undamped = reconstruct(
        modes,
        undamped_coefficients,
        0,
        OBSERVATION_SAMPLES,
        undamped=True,
        retained_indices=retained_indices,
    )
    return reports, candidate, undamped


def normalized_rms_error(expected: np.ndarray, observed: np.ndarray) -> float:
    denominator = math.sqrt(float(np.mean(expected * expected)))
    if denominator <= 1.0e-300:
        raise CounterfactualError("reconstruction reference has zero RMS")
    return math.sqrt(float(np.mean((expected - observed) ** 2))) / denominator


def squared_error(expected: np.ndarray, observed: np.ndarray) -> float:
    return float(np.sum((expected - observed) ** 2))


def prediction_control(
    post_onset: np.ndarray, retained_modes: list[dict[str, Any]]
) -> dict[str, Any]:
    damped_coefficients = fit_coefficients(
        post_onset,
        retained_modes,
        0,
        PREDICTION_CALIBRATION_SAMPLES,
        undamped=False,
    )
    undamped_coefficients = fit_coefficients(
        post_onset,
        retained_modes,
        0,
        PREDICTION_CALIBRATION_SAMPLES,
        undamped=True,
    )
    windows = []
    for start, stop in PREDICTION_WINDOWS:
        expected = post_onset[:, start:stop]
        damped = reconstruct(
            retained_modes,
            damped_coefficients,
            start,
            stop,
            undamped=False,
        )
        undamped = reconstruct(
            retained_modes,
            undamped_coefficients,
            start,
            stop,
            undamped=True,
        )
        damped_error = squared_error(expected, damped)
        undamped_error = squared_error(expected, undamped)
        windows.append(
            {
                "start_sample": start,
                "stop_sample": stop,
                "start_ms": start * 1000.0 / SAMPLE_RATE_HZ,
                "stop_ms": stop * 1000.0 / SAMPLE_RATE_HZ,
                "damped_nrmse": normalized_rms_error(expected, damped),
                "undamped_nrmse": normalized_rms_error(expected, undamped),
                "damped_squared_error": damped_error,
                "undamped_squared_error": undamped_error,
                "damped_to_undamped_error_ratio": (
                    damped_error / max(undamped_error, 1.0e-300)
                ),
            }
        )
    return {
        "calibration_start_sample": 0,
        "calibration_stop_sample": PREDICTION_CALIBRATION_SAMPLES,
        "calibration_stop_ms": (
            PREDICTION_CALIBRATION_SAMPLES * 1000.0 / SAMPLE_RATE_HZ
        ),
        "windows": windows,
    }


def cents_error(first: float, second: float) -> float:
    return 1200.0 * abs(math.log2(first / second))


def injective_match(
    expected_modes: list[dict[str, Any]],
    observed_modes: list[dict[str, Any]],
) -> dict[str, Any]:
    pairs = []
    for expected_index, expected in enumerate(expected_modes):
        for observed_index, observed in enumerate(observed_modes):
            error = cents_error(
                expected["frequency_hz"], observed["frequency_hz"]
            )
            if error <= MATCH_TOLERANCE_CENTS:
                pairs.append((error, expected_index, observed_index))
    pairs.sort()
    assignments: list[int | None] = [None] * len(expected_modes)
    used_observed: set[int] = set()
    errors = []
    for error, expected_index, observed_index in pairs:
        if (
            assignments[expected_index] is None
            and observed_index not in used_observed
        ):
            assignments[expected_index] = observed_index
            used_observed.add(observed_index)
            errors.append(error)
    matched = sum(item is not None for item in assignments)
    return {
        "expected_mode_count": len(expected_modes),
        "observed_mode_count": len(observed_modes),
        "matched_mode_count": matched,
        "match_fraction": matched / max(len(expected_modes), 1),
        "maximum_match_error_cents": max(errors) if errors else None,
        "assignments": assignments,
    }


def analyze_partition(
    coefficients: np.ndarray,
    discovery: dict[str, Any],
    output_indices: list[int],
    retained_modes: list[dict[str, Any]],
) -> dict[str, Any]:
    raw, analyses = broadband.extract_raw_estimates(
        coefficients[:, :, output_indices], discovery
    )
    clusters = broadband.cluster_duplicates(raw)
    return {
        "output_indices": output_indices,
        "raw_estimate_count": len(raw),
        "cluster_count": len(clusters),
        "clusters": clusters,
        "bin_analyses": analyses,
        "retained_full_output_matching": injective_match(
            retained_modes, representatives(clusters)
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


def capacity_rejection(
    common: dict[str, Any],
    lineage: dict[str, Any],
    discovery: dict[str, Any],
    scale_invariance: dict[str, Any],
    reason: str,
    checks: list[dict[str, Any]],
) -> dict[str, Any]:
    lineage.pop("decoded_block_path")
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": "ExistingIronBroadbandCounterfactualRejected",
        "claim": (
            "EXISTING_IRON_READ_ONLY_CAPACITY_REJECTION / NO_MATERIAL_QUALITY_"
            "ADMISSION_PHYSICS_PLANTER_OR_RUNTIME_CREDIT"
        ),
        **common,
        "lineage": lineage,
        "existing_decoded_payload_bytes_analyzed": (
            len(ANALYSIS_ROWS)
            * (ONSET_SAMPLE + OBSERVATION_SAMPLES)
            * np.dtype("<f4").itemsize
        ),
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "scale_invariance": scale_invariance,
        "analysis_skipped_due_to_capacity": True,
        "capacity_rejection_reason": reason,
        "gate": {"passed": False, "checks": checks},
        "next_action": (
            "preserve the rejection and research a bounded selector on synthetic "
            "and existing-block controls; do not tune Iron or open new payload"
        ),
    }


def run_counterfactual(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    lineage = validate_lineage(root, base, manifest)
    block_path = Path(lineage["decoded_block_path"])
    channels = load_observation(block_path)
    post_onset = channels[:, ONSET_SAMPLE : ONSET_SAMPLE + OBSERVATION_SAMPLES]
    gabor_stop = broadband.WINDOW_SAMPLES + (
        broadband.GABOR_FRAME_COUNT - 1
    ) * broadband.HOP_SAMPLES
    coefficients = broadband.gabor_coefficients(channels[:, :gabor_stop])
    discovery = broadband.discover_regions(coefficients)
    scale_invariance = broadband.discovery_scale_invariance(coefficients)
    common = common_report(manifest_bytes, runner_sha256)

    capacity_checks = [
        check(
            "minimum_region_count",
            len(discovery["regions"]),
            ">=",
            GATES["minimum_region_count"],
        ),
        check(
            "maximum_region_count",
            len(discovery["regions"]),
            "<=",
            GATES["maximum_region_count"],
        ),
        check(
            "maximum_analysis_bin_count",
            len(discovery["analysis_bins"]),
            "<=",
            GATES["maximum_analysis_bin_count"],
        ),
    ]
    if not all(item["passed"] for item in capacity_checks):
        return capacity_rejection(
            common,
            lineage,
            discovery,
            scale_invariance,
            "input-driven discovery exceeded the frozen bounded analysis envelope",
            capacity_checks,
        )

    raw, bin_analyses = broadband.extract_raw_estimates(coefficients, discovery)
    clusters = broadband.cluster_duplicates(raw)
    cluster_checks = [
        *capacity_checks,
        check(
            "minimum_preprune_cluster_count",
            len(clusters),
            ">=",
            GATES["minimum_preprune_cluster_count"],
        ),
        check(
            "maximum_preprune_cluster_count",
            len(clusters),
            "<=",
            GATES["maximum_preprune_cluster_count"],
        ),
    ]
    if not all(item["passed"] for item in cluster_checks):
        return capacity_rejection(
            common,
            lineage,
            discovery,
            scale_invariance,
            "common-pole cluster count left the frozen fit envelope",
            cluster_checks,
        )

    fitted, candidate_reconstruction, undamped_reconstruction = fit_and_prune(
        post_onset, clusters
    )
    retained = [item for item in fitted if item["retained"]]
    retained_modes = [item["representative"] for item in retained]
    duplicate_estimates_removed = len(raw) - len(clusters)
    candidate_nrmse = normalized_rms_error(
        post_onset, candidate_reconstruction
    )
    undamped_nrmse = normalized_rms_error(
        post_onset, undamped_reconstruction
    )
    full_ratio = candidate_nrmse / max(undamped_nrmse, 1.0e-300)
    prediction = prediction_control(post_onset, retained_modes)
    partitions = {
        name: analyze_partition(
            coefficients, discovery, indices, retained_modes
        )
        for name, indices in SPATIAL_PARTITIONS.items()
    }
    even_fraction = partitions["even_microphones"][
        "retained_full_output_matching"
    ]["match_fraction"]
    odd_fraction = partitions["odd_microphones"][
        "retained_full_output_matching"
    ]["match_fraction"]
    checks = [
        *capacity_checks,
        check(
            "duplicate_estimates_removed",
            duplicate_estimates_removed,
            ">=",
            GATES["minimum_duplicate_estimates_removed"],
        ),
        check(
            "minimum_preprune_cluster_count",
            len(clusters),
            ">=",
            GATES["minimum_preprune_cluster_count"],
        ),
        check(
            "maximum_preprune_cluster_count",
            len(clusters),
            "<=",
            GATES["maximum_preprune_cluster_count"],
        ),
        check(
            "minimum_retained_mode_count",
            len(retained),
            ">=",
            GATES["minimum_retained_mode_count"],
        ),
        check(
            "maximum_retained_mode_count",
            len(retained),
            "<=",
            GATES["maximum_retained_mode_count"],
        ),
        check(
            "scale_invariant_region_bins",
            scale_invariance["passed"],
            "==",
            GATES["scale_invariant_region_bins"],
        ),
        check(
            "even_partition_match_fraction",
            even_fraction,
            ">=",
            GATES["minimum_even_partition_match_fraction"],
        ),
        check(
            "odd_partition_match_fraction",
            odd_fraction,
            ">=",
            GATES["minimum_odd_partition_match_fraction"],
        ),
        check(
            "full_observed_nrmse",
            candidate_nrmse,
            "<=",
            GATES["maximum_full_observed_nrmse"],
        ),
        check(
            "full_damped_to_undamped_nrmse_ratio",
            full_ratio,
            "<=",
            GATES["maximum_full_damped_to_undamped_nrmse_ratio"],
        ),
        check(
            "prediction_error_ratio_window_a",
            prediction["windows"][0]["damped_to_undamped_error_ratio"],
            "<=",
            GATES["maximum_prediction_error_ratio_window_a"],
        ),
        check(
            "prediction_error_ratio_window_b",
            prediction["windows"][1]["damped_to_undamped_error_ratio"],
            "<=",
            GATES["maximum_prediction_error_ratio_window_b"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    lineage.pop("decoded_block_path")
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": (
            "ExistingIronBroadbandCounterfactualSupported"
            if passed
            else "ExistingIronBroadbandCounterfactualRejected"
        ),
        "claim": (
            "EXISTING_IRON_READ_ONLY_COMMON_POLE_METHOD_TRANSFER_ONLY / "
            "NO_MATERIAL_IDENTITY_PERCEPTUAL_QUALITY_DOMAIN_ADMISSION_PHYSICS_"
            "PLANTER_OR_RUNTIME_CREDIT"
        ),
        **common,
        "lineage": lineage,
        "existing_decoded_payload_bytes_analyzed": (
            len(ANALYSIS_ROWS)
            * (ONSET_SAMPLE + OBSERVATION_SAMPLES)
            * np.dtype("<f4").itemsize
        ),
        "input_slice_sha256": sha256_bytes(
            channels.astype("<f8", copy=False).tobytes()
        ),
        "gabor_coefficients_sha256": sha256_bytes(
            coefficients.astype("<c16", copy=False).tobytes()
        ),
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "scale_invariance": scale_invariance,
        "bin_analyses": bin_analyses,
        "raw_estimate_count": len(raw),
        "duplicate_estimates_removed": duplicate_estimates_removed,
        "fitted_clusters": fitted,
        "retained_mode_count": len(retained),
        "pruned_mode_count": len(fitted) - len(retained),
        "spatial_partitions": partitions,
        "reconstruction": {
            "observation_samples": OBSERVATION_SAMPLES,
            "candidate_nrmse": candidate_nrmse,
            "undamped_nrmse": undamped_nrmse,
            "damped_to_undamped_nrmse_ratio": full_ratio,
        },
        "prediction_control": prediction,
        "gate": {"passed": passed, "checks": checks},
        "analysis_skipped_due_to_capacity": False,
        "next_action": (
            "freeze an independent internet-sourced real holdout before any "
            "domain or quality claim"
            if passed
            else "preserve the rejection and research the failed gate without "
            "tuning Iron or opening new payload"
        ),
    }


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
    manifest_path = external_file(root, arguments.manifest, "manifest")
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    report = (
        preflight(root, base, manifest_bytes, manifest, runner_sha256)
        if arguments.stage == "preflight"
        else run_counterfactual(
            root, base, manifest_bytes, manifest, runner_sha256
        )
    )
    report_bytes = publish(output, report)
    print(f"Existing-Iron broad-band {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("additional payload bytes read: 0")
    print("network requests: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
