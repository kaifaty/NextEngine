#!/usr/bin/env python3
"""Validate the dense R2B complex-field data boundary before any optimizer step."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import struct
import subprocess
import sys
import wave
from pathlib import Path
from typing import Any

import numpy as np

MANIFEST_SCHEMA = "nextengine.experimental-realimpact-dense-listener-block.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r2b-dense-preflight.report.v1"
RUST_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-validator.manifest.v1"
RUST_REPORT_SCHEMA = "nextengine.experimental-physical-sound-validator.report.v1"
PROFILE = "green-goblet-dense-listener-block-v3"
REPRESENTATION_ID = "complex-stft-sqrt-periodic-hann-2048-hop512-v1"
ROW_COUNT = 600
COLUMN_COUNT = 40
CONTEXT_ROWS = 420
QUERY_ROWS = 180
MICROPHONES_PER_COLUMN = 15
SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 208_323
FFT_LENGTH = 2_048
HOP_LENGTH = 512
QUERY_ANGLES = (40, 100, 160)
BRACKET_ANGLES = (20, 60, 80, 120, 140, 180)
TARGET_PEAK = 0.92
DB_FLOOR = -240.0
PRIMARY_ENDPOINTS = (
    "mean_absolute_rms_level_error_db",
    "p95_absolute_rms_level_error_db",
    "mean_gain_matched_multiresolution_log_spectrum_rmse_db",
    "p95_gain_matched_multiresolution_log_spectrum_rmse_db",
    "mean_normalized_waveform_rmse_db",
)
CONTROL_IDS = (
    "nearest_context_azimuth_same_distance_and_microphone",
    "linear_bracketing_azimuth_same_distance_and_microphone",
    "log_magnitude_shortest_arc_phase_bracketing_complex_stft",
)
METRIC_SOURCE_PATHS = (
    "tools/xtask/src/physical_sound_eval_command.rs",
    "tools/xtask/src/physical_sound_eval_command/audio_analysis.rs",
    "tools/xtask/src/physical_sound_eval_command/audio_analysis/amplitude_envelope.rs",
)


class PreflightError(RuntimeError):
    """The frozen R2B preflight boundary failed closed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def read_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise PreflightError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise PreflightError(f"{label} must contain one JSON object")
    return data, value


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise PreflightError(f"{label} must be an external file: {resolved}")
    return resolved


def prepare_output(root: Path, argument: Path) -> tuple[Path, Path]:
    unresolved = argument if argument.is_absolute() else root / argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise PreflightError(f"output must be a new external directory: {output}")
    staging = parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        raise PreflightError(f"staging directory already exists: {staging}")
    staging.mkdir()
    return output, staging


def expected_representation() -> dict[str, Any]:
    return {
        "id": REPRESENTATION_ID,
        "target": "complex_pressure_rfft",
        "sample_dtype": "float32le",
        "feature_dtype": "complex64le",
        "fft_length": FFT_LENGTH,
        "window_length": FFT_LENGTH,
        "hop_length": HOP_LENGTH,
        "window": "sqrt_periodic_hann",
        "centering": "zero_pad_half_window_each_side",
        "inverse": "irfft_overlap_add_divide_window_square_then_crop",
        "shared_normalization": (
            "context_420_rows_global_peak_to_0.92_pcm16_full_scale_"
            "applied_unchanged_to_query"
        ),
        "split_unit": "complete_15_microphone_gantry_column",
        "query_group": "all_columns_at_azimuth_degrees_40_100_160",
        "context_group": "all_columns_at_remaining_published_azimuths",
        "classical_controls": list(CONTROL_IDS),
        "inverse_float_nrmse_db_max": -140.0,
        "inverse_max_absolute_error_max": 1.0e-6,
        "inverse_pcm_requirement": "maximum_absolute_difference_lte_1_lsb",
        "optimizer_authorized": False,
    }


def validate_manifest(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> tuple[Path, list[dict[str, Any]]]:
    required = {
        "schema",
        "status",
        "profile",
        "frozen_profile_sha256",
        "source_repository_revision",
        "dataset_object_id",
        "object_id",
        "geometry_revision",
        "impact_position_id",
        "impact_position_metres",
        "sample_rate_hz",
        "sample_count_per_row",
        "row_count",
        "column_count",
        "context_row_count",
        "query_row_count",
        "block_payload",
        "columns",
        "rows",
        "metadata_array_sha256",
        "representation_preflight",
        "prerequisites",
        "allowed_claims",
        "prohibited_claims",
        "unavailable_components",
    }
    if set(manifest) != required:
        raise PreflightError("dense acquisition manifest fields changed")
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("status") != "development_dense_fixed_impact_block"
        or manifest.get("profile") != PROFILE
        or manifest.get("sample_rate_hz") != SAMPLE_RATE_HZ
        or manifest.get("sample_count_per_row") != SAMPLE_COUNT
        or manifest.get("row_count") != ROW_COUNT
        or manifest.get("column_count") != COLUMN_COUNT
        or manifest.get("context_row_count") != CONTEXT_ROWS
        or manifest.get("query_row_count") != QUERY_ROWS
        or manifest.get("representation_preflight") != expected_representation()
    ):
        raise PreflightError("dense acquisition manifest semantics changed")
    block_ref = manifest.get("block_payload")
    if (
        not isinstance(block_ref, dict)
        or set(block_ref) != {"path", "sha256", "byte_count"}
        or not isinstance(block_ref["path"], str)
        or not isinstance(block_ref["sha256"], str)
        or len(block_ref["sha256"]) != 64
        or block_ref["byte_count"] != ROW_COUNT * SAMPLE_COUNT * 4
    ):
        raise PreflightError("dense block file reference changed")
    unresolved = Path(block_ref["path"])
    block_path = unresolved if unresolved.is_absolute() else manifest_path.parent / unresolved
    block_path = external_file(root, block_path, "dense listener block")
    if block_path.stat().st_size != block_ref["byte_count"]:
        raise PreflightError("dense listener block byte count changed")
    if sha256_file(block_path) != block_ref["sha256"]:
        raise PreflightError("dense listener block hash changed")
    rows = manifest.get("rows")
    if not isinstance(rows, list) or len(rows) != ROW_COUNT:
        raise PreflightError("dense row projection changed")
    expected_indices = list(range(ROW_COUNT))
    if [row.get("row_index") for row in rows] != expected_indices:
        raise PreflightError("dense rows must remain source-ordered")
    for row in rows:
        index = row["row_index"]
        expected_role = "query" if row.get("azimuth_degrees") in QUERY_ANGLES else "context"
        if (
            row.get("split_role") != expected_role
            or row.get("microphone_id") != index % MICROPHONES_PER_COLUMN
            or row.get("payload_offset_bytes") != index * SAMPLE_COUNT * 4
            or row.get("sample_count") != SAMPLE_COUNT
            or not isinstance(row.get("raw_f32le_sha256"), str)
            or len(row["raw_f32le_sha256"]) != 64
        ):
            raise PreflightError(f"dense row projection changed at row {index}")
    if sum(row["split_role"] == "context" for row in rows) != CONTEXT_ROWS:
        raise PreflightError("context row count changed")
    if sum(row["split_role"] == "query" for row in rows) != QUERY_ROWS:
        raise PreflightError("query row count changed")
    return block_path, rows


class ComplexTransform:
    def __init__(self, sample_count: int) -> None:
        periodic_hann = np.hanning(FFT_LENGTH + 1)[:-1]
        self.window = np.sqrt(periodic_hann).astype(np.float64)
        self.left_padding = FFT_LENGTH // 2
        self.frame_count = math.ceil(sample_count / HOP_LENGTH) + 1
        self.total_samples = (self.frame_count - 1) * HOP_LENGTH + FFT_LENGTH
        self.right_padding = self.total_samples - self.left_padding - sample_count
        self.denominator = np.zeros(self.total_samples, dtype=np.float64)
        squared = self.window * self.window
        for frame in range(self.frame_count):
            start = frame * HOP_LENGTH
            self.denominator[start : start + FFT_LENGTH] += squared
        if np.any(self.denominator[self.left_padding : -self.right_padding] <= 1.0e-18):
            raise PreflightError("complex transform has an uncovered source sample")

    def analyze(self, signal: np.ndarray) -> np.ndarray:
        padded = np.pad(signal, (self.left_padding, self.right_padding))
        frames = np.lib.stride_tricks.sliding_window_view(padded, FFT_LENGTH)[
            ::HOP_LENGTH
        ]
        if frames.shape != (self.frame_count, FFT_LENGTH):
            raise PreflightError("complex transform frame grid changed")
        return np.fft.rfft(frames * self.window, axis=1).astype("<c8")

    def synthesize(self, spectrum: np.ndarray) -> np.ndarray:
        if spectrum.shape != (self.frame_count, FFT_LENGTH // 2 + 1):
            raise PreflightError("complex spectrum grid changed")
        frames = np.fft.irfft(spectrum.astype(np.complex128), n=FFT_LENGTH, axis=1)
        output = np.zeros(self.total_samples, dtype=np.float64)
        for frame, values in enumerate(frames):
            start = frame * HOP_LENGTH
            output[start : start + FFT_LENGTH] += values * self.window
        output = np.divide(
            output,
            self.denominator,
            out=np.zeros_like(output),
            where=self.denominator > 1.0e-18,
        )
        return output[self.left_padding : self.left_padding + SAMPLE_COUNT]


def quantize_pcm(signal: np.ndarray) -> np.ndarray:
    if signal.shape != (SAMPLE_COUNT,) or not np.isfinite(signal).all():
        raise PreflightError("PCM input is not a finite source-length signal")
    return np.clip(np.rint(signal * 32768.0), -32768, 32767).astype("<i2")


def wav_bytes(pcm: np.ndarray) -> bytes:
    payload = pcm.astype("<i2", copy=False).tobytes()
    return b"".join(
        (
            b"RIFF",
            struct.pack("<I", 36 + len(payload)),
            b"WAVEfmt ",
            struct.pack(
                "<IHHIIHH",
                16,
                1,
                1,
                SAMPLE_RATE_HZ,
                SAMPLE_RATE_HZ * 2,
                2,
                16,
            ),
            b"data",
            struct.pack("<I", len(payload)),
            payload,
        )
    )


def nrmse_db(candidate: np.ndarray, reference: np.ndarray) -> float:
    reference_energy = float(np.sum(reference * reference))
    if reference_energy <= 0.0:
        raise PreflightError("reference signal is silent")
    error_energy = float(np.sum((candidate - reference) ** 2))
    ratio = math.sqrt(error_energy / reference_energy)
    return DB_FLOOR if ratio <= 1.0e-12 else max(DB_FLOOR, 20.0 * math.log10(ratio))


def nearest_rank(values: list[float], percentile: int) -> float:
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered) / 100.0))
    return ordered[rank - 1]


def aggregate(rows: list[dict[str, Any]]) -> dict[str, float]:
    level = [row["absolute_rms_level_error_db"] for row in rows]
    spectrum = [
        row["gain_matched_multiresolution_log_spectrum_rmse_db"] for row in rows
    ]
    waveform = [row["normalized_waveform_rmse_db"] for row in rows]
    return {
        "mean_absolute_rms_level_error_db": sum(level) / len(level),
        "p95_absolute_rms_level_error_db": nearest_rank(level, 95),
        "mean_gain_matched_multiresolution_log_spectrum_rmse_db": sum(spectrum)
        / len(spectrum),
        "p95_gain_matched_multiresolution_log_spectrum_rmse_db": nearest_rank(
            spectrum, 95
        ),
        "mean_normalized_waveform_rmse_db": sum(waveform) / len(waveform),
    }


def read_pcm16(path: Path) -> np.ndarray:
    with wave.open(str(path), "rb") as source:
        if (
            source.getnchannels() != 1
            or source.getsampwidth() != 2
            or source.getframerate() != SAMPLE_RATE_HZ
            or source.getnframes() != SAMPLE_COUNT
            or source.getcomptype() != "NONE"
        ):
            raise PreflightError(f"WAV grid changed: {path}")
        payload = source.readframes(SAMPLE_COUNT)
    return np.frombuffer(payload, dtype="<i2").astype(np.float64) / 32768.0


def complex_interpolate(lower: np.ndarray, upper: np.ndarray) -> np.ndarray:
    magnitude_floor = 1.0e-12
    lower_magnitude = np.maximum(np.abs(lower).astype(np.float64), magnitude_floor)
    upper_magnitude = np.maximum(np.abs(upper).astype(np.float64), magnitude_floor)
    magnitude = np.exp(0.5 * (np.log(lower_magnitude) + np.log(upper_magnitude)))
    phase = np.angle(lower) + 0.5 * np.angle(upper * np.conj(lower))
    return (magnitude * np.exp(1j * phase)).astype("<c8")


def metric_sources(root: Path) -> list[dict[str, Any]]:
    records = []
    for relative in METRIC_SOURCE_PATHS:
        path = (root / relative).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file():
            raise PreflightError(f"metric source escaped repository: {relative}")
        records.append(
            {
                "path": relative,
                "sha256": sha256_file(path),
                "byte_count": path.stat().st_size,
            }
        )
    return records


def run_rust_metrics(
    root: Path,
    control_id: str,
    predictions: list[dict[str, Any]],
    work: Path,
) -> dict[str, Any]:
    entries = []
    paths: dict[str, tuple[Path, Path]] = {}
    for record in predictions:
        entry_id = f"row-{record['row_index']:04}"
        candidate = Path(record["prediction_path"])
        reference = Path(record["reference_path"])
        entries.append(
            {
                "id": entry_id,
                "object_id": "realimpact-green-goblet",
                "material": "glass-vessel-published-label",
                "impact_position": "fixed-mesh-vertex-31676",
                "force_band": "force-deconvolved-transfer",
                "expected_signal": "impact",
                "candidate": {
                    "path": str(candidate),
                    "sha256": sha256_file(candidate),
                },
                "reference": {
                    "path": str(reference),
                    "sha256": sha256_file(reference),
                },
            }
        )
        paths[entry_id] = (candidate, reference)
    value = {
        "schema": RUST_MANIFEST_SCHEMA,
        "split": f"r2b-{control_id}",
        "relations": [],
        "entries": entries,
    }
    manifest_bytes = canonical_json(value)
    manifest_path = work / f"{control_id}.manifest.json"
    manifest_path.write_bytes(manifest_bytes)
    output = work / f"{control_id}.rust-eval"
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "physical-sound-eval",
            "--manifest",
            str(manifest_path),
            "--output",
            str(output),
        ],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise PreflightError(
            f"Rust metrics failed for {control_id}: "
            f"{completed.stdout}\n{completed.stderr}"
        )
    _, rust_report = read_json(output / "report.json", "Rust metric report")
    if rust_report.get("schema") != RUST_REPORT_SCHEMA:
        raise PreflightError("Rust metric report schema changed")
    rows = []
    for entry in rust_report.get("entries", []):
        matched = entry.get("matched")
        entry_id = entry.get("id")
        if matched is None or entry_id not in paths:
            raise PreflightError("Rust metric report omitted a query match")
        candidate, reference = paths[entry_id]
        rows.append(
            {
                "row_index": int(entry_id.rsplit("-", 1)[1]),
                "absolute_rms_level_error_db": abs(matched["raw_rms_delta_db"]),
                "gain_matched_multiresolution_log_spectrum_rmse_db": matched[
                    "gain_matched_multiresolution_log_spectrum_rmse_db"
                ],
                "normalized_waveform_rmse_db": nrmse_db(
                    read_pcm16(candidate), read_pcm16(reference)
                ),
            }
        )
    rows.sort(key=lambda row: row["row_index"])
    if len(rows) != QUERY_ROWS:
        raise PreflightError(f"Rust metrics returned {len(rows)} rows, expected {QUERY_ROWS}")
    normalized_bytes = canonical_json(rows)
    return {
        "control_id": control_id,
        "query_count": len(rows),
        "normalized_metric_rows_sha256": sha256_bytes(normalized_bytes),
        "aggregate": aggregate(rows),
        "rows": rows,
    }


def coverage(rows: list[dict[str, Any]]) -> dict[str, Any]:
    context = np.asarray(
        [row["listener_position_metres"] for row in rows if row["split_role"] == "context"],
        dtype=np.float64,
    )
    query = np.asarray(
        [row["listener_position_metres"] for row in rows if row["split_role"] == "query"],
        dtype=np.float64,
    )
    nearest = []
    for point in query:
        nearest.append(float(np.min(np.linalg.norm(context - point, axis=1))))
    all_points = np.concatenate((context, query), axis=0)
    return {
        "context_unique_positions": int(len(np.unique(context, axis=0))),
        "query_unique_positions": int(len(np.unique(query, axis=0))),
        "coordinate_min_metres": np.min(all_points, axis=0).tolist(),
        "coordinate_max_metres": np.max(all_points, axis=0).tolist(),
        "query_nearest_context_distance_metres": {
            "minimum": min(nearest),
            "mean": sum(nearest) / len(nearest),
            "maximum": max(nearest),
        },
        "published_grid": {
            "azimuth_degrees": list(range(0, 181, 20)),
            "distance_offsets_millimetres": [0, 333, 666, 1000],
            "microphone_z_spacing_metres": 0.13,
        },
    }


def future_candidate_protocol() -> dict[str, Any]:
    return {
        "status": "preregistered_after_data_readiness_only",
        "shared_model": (
            "coordinate_time_frequency_mlp_outputs_real_and_imaginary_pressure"
        ),
        "inputs": [
            "listener_position_metres",
            "impact_position_metres",
            "frame_time_seconds",
            "frequency_hz",
        ],
        "outputs": ["pressure_real", "pressure_imaginary"],
        "candidate_ids": [
            "dense_complex_field_data_only_v1",
            "dense_complex_field_helmholtz_v1",
        ],
        "data_loss": "context_only_complex_stft_l1_plus_log_magnitude_l1",
        "physics_ablation": {
            "data_only_weight": 0.0,
            "helmholtz_weight": 0.0001,
            "equation": "laplacian_p_plus_(2pi_frequency_over_343)^2_p_equals_zero",
            "frequency_band_hz": [93.75, 12_000.0],
            "collocation": (
                "deterministic_midpoints_between_context_azimuth_planes_at_"
                "published_distance_and_microphone_coordinates"
            ),
            "query_audio_used": False,
        },
        "selection_rule": (
            "strictly_beat_every_frozen_classical_control_on_all_five_primary_"
            "aggregates"
        ),
        "method_holdout_and_admission_shadow": "sealed",
    }


def run_preflight(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    block_path: Path,
    rows: list[dict[str, Any]],
    staging: Path,
) -> dict[str, Any]:
    mapped = np.memmap(
        block_path,
        dtype="<f4",
        mode="r",
        shape=(ROW_COUNT, SAMPLE_COUNT),
    )
    context_peak = max(row["peak_abs"] for row in rows if row["split_role"] == "context")
    query_peak = max(row["peak_abs"] for row in rows if row["split_role"] == "query")
    if not math.isfinite(context_peak) or context_peak <= 0.0:
        raise PreflightError("context-only normalization peak is invalid")
    normalization_scale = TARGET_PEAK / context_peak
    if query_peak * normalization_scale >= 1.0:
        raise PreflightError("context-only scale makes a query row non-cookable")

    transform = ComplexTransform(SAMPLE_COUNT)
    references = staging / "references"
    predictions_root = staging / "controls"
    references.mkdir()
    predictions_root.mkdir()
    context_features: dict[tuple[int, int, int], np.ndarray] = {}
    representation_rows = []
    query_reference_records: dict[int, dict[str, Any]] = {}
    max_abs_error = 0.0
    worst_nrmse_db = DB_FLOOR
    max_pcm_difference = 0
    pcm_mismatch_samples = 0

    for row in rows:
        index = row["row_index"]
        source = np.asarray(mapped[index], dtype=np.float64)
        if not np.isfinite(source).all():
            raise PreflightError(f"source row {index} contains a non-finite sample")
        source_bytes = np.asarray(mapped[index], dtype="<f4").tobytes()
        if sha256_bytes(source_bytes) != row["raw_f32le_sha256"]:
            raise PreflightError(f"source row hash changed at {index}")
        actual_peak = float(np.max(np.abs(source)))
        actual_rms = float(np.sqrt(np.mean(source * source)))
        if not math.isclose(actual_peak, row["peak_abs"], rel_tol=0.0, abs_tol=1.0e-12):
            raise PreflightError(f"source row peak changed at {index}")
        if not math.isclose(actual_rms, row["rms"], rel_tol=1.0e-12, abs_tol=1.0e-12):
            raise PreflightError(f"source row RMS changed at {index}")
        normalized = source * normalization_scale
        spectrum = transform.analyze(normalized)
        reconstructed = transform.synthesize(spectrum)
        error = reconstructed - normalized
        row_max_error = float(np.max(np.abs(error)))
        row_nrmse_db = nrmse_db(reconstructed, normalized)
        direct_pcm = quantize_pcm(normalized)
        reconstructed_pcm = quantize_pcm(reconstructed)
        pcm_difference = np.abs(
            reconstructed_pcm.astype(np.int32) - direct_pcm.astype(np.int32)
        )
        row_pcm_max = int(np.max(pcm_difference))
        row_pcm_mismatches = int(np.count_nonzero(pcm_difference))
        if (
            row_max_error > 1.0e-6
            or row_nrmse_db > -140.0
            or row_pcm_max > 1
        ):
            raise PreflightError(f"complex inverse gate failed at row {index}")
        max_abs_error = max(max_abs_error, row_max_error)
        worst_nrmse_db = max(worst_nrmse_db, row_nrmse_db)
        max_pcm_difference = max(max_pcm_difference, row_pcm_max)
        pcm_mismatch_samples += row_pcm_mismatches
        representation_rows.append(
            {
                "row_index": index,
                "split_role": row["split_role"],
                "source_sha256": row["raw_f32le_sha256"],
                "complex_stft_sha256": sha256_bytes(
                    spectrum.astype("<c8", copy=False).tobytes()
                ),
                "roundtrip_max_absolute_error": row_max_error,
                "roundtrip_nrmse_db": row_nrmse_db,
                "roundtrip_max_pcm_difference_lsb": row_pcm_max,
                "roundtrip_pcm_mismatch_samples": row_pcm_mismatches,
            }
        )
        angle = row["azimuth_degrees"]
        key = (
            angle,
            row["gantry_distance_offset_millimetres"],
            row["microphone_id"],
        )
        if row["split_role"] == "context" and angle in BRACKET_ANGLES:
            context_features[key] = spectrum
        if row["split_role"] == "query":
            path = references / f"row-{index:04}.wav"
            data = wav_bytes(direct_pcm)
            path.write_bytes(data)
            query_reference_records[index] = {
                "path": path,
                "sha256": sha256_bytes(data),
                "byte_count": len(data),
            }

    if len(context_features) != 360 or len(query_reference_records) != QUERY_ROWS:
        raise PreflightError("context feature cache or query reference coverage changed")

    prediction_sets: dict[str, list[dict[str, Any]]] = {
        control_id: [] for control_id in CONTROL_IDS
    }
    for row in rows:
        if row["split_role"] != "query":
            continue
        index = row["row_index"]
        angle = row["azimuth_degrees"]
        lower_angle = angle - 20
        upper_angle = angle + 20
        distance = row["gantry_distance_offset_millimetres"]
        microphone = row["microphone_id"]
        lower_index = ((lower_angle // 20) * 4 + [0, 333, 666, 1000].index(distance)) * 15 + microphone
        upper_index = ((upper_angle // 20) * 4 + [0, 333, 666, 1000].index(distance)) * 15 + microphone
        lower = np.asarray(mapped[lower_index], dtype=np.float64) * normalization_scale
        upper = np.asarray(mapped[upper_index], dtype=np.float64) * normalization_scale
        control_signals = {
            CONTROL_IDS[0]: lower,
            CONTROL_IDS[1]: 0.5 * (lower + upper),
            CONTROL_IDS[2]: transform.synthesize(
                complex_interpolate(
                    context_features[(lower_angle, distance, microphone)],
                    context_features[(upper_angle, distance, microphone)],
                )
            ),
        }
        reference = query_reference_records[index]
        for control_id, signal in control_signals.items():
            peak = float(np.max(np.abs(signal)))
            if not math.isfinite(peak) or peak >= 1.0:
                raise PreflightError(f"control {control_id} row {index} is not cookable")
            directory = predictions_root / control_id
            directory.mkdir(exist_ok=True)
            path = directory / f"row-{index:04}.wav"
            data = wav_bytes(quantize_pcm(signal))
            path.write_bytes(data)
            prediction_sets[control_id].append(
                {
                    "row_index": index,
                    "prediction_path": str(path),
                    "prediction_sha256": sha256_bytes(data),
                    "prediction_byte_count": len(data),
                    "reference_path": str(reference["path"]),
                    "reference_sha256": reference["sha256"],
                }
            )

    work = staging / ".metric-work"
    work.mkdir()
    controls = [
        run_rust_metrics(root, control_id, prediction_sets[control_id], work)
        for control_id in CONTROL_IDS
    ]
    shutil.rmtree(work)
    for records in prediction_sets.values():
        for record in records:
            record["prediction_path"] = str(
                Path(record["prediction_path"]).relative_to(staging)
            )
            record["reference_path"] = str(Path(record["reference_path"]).relative_to(staging))

    query_audio_bytes = QUERY_ROWS * SAMPLE_COUNT * 4
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "ReadyForComplexFieldTraining",
        "claim": (
            "DENSE_COMPLEX_FIELD_DATA_AND_REPRESENTATION_READY_ONLY / NO_TRAINED_"
            "MODEL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "profile": PROFILE,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": sha256_file(Path(__file__).resolve(strict=True)),
        "block_payload_sha256": manifest["block_payload"]["sha256"],
        "environment": {
            "python": sys.version.split()[0],
            "numpy": np.__version__,
            "device": "cpu",
            "threads": 1,
        },
        "metric_sources": metric_sources(root),
        "split": {
            "unit": "complete_15_microphone_gantry_column",
            "context_rows": CONTEXT_ROWS,
            "query_rows": QUERY_ROWS,
            "query_angles_degrees": list(QUERY_ANGLES),
            "query_audio_rows_read": QUERY_ROWS,
            "query_audio_bytes_read": query_audio_bytes,
            "query_rows_used_for_normalization": 0,
            "query_rows_cached_for_candidate_fit": 0,
            "method_holdout_or_shadow_bytes_read": 0,
        },
        "coverage": coverage(rows),
        "normalization": {
            "fit_role": "context_only",
            "context_peak_abs": context_peak,
            "query_peak_abs_report_only": query_peak,
            "target_peak_fraction": TARGET_PEAK,
            "scale": normalization_scale,
            "query_peak_after_context_scale": query_peak * normalization_scale,
        },
        "representation": {
            **expected_representation(),
            "frame_count": transform.frame_count,
            "frequency_bin_count": FFT_LENGTH // 2 + 1,
            "left_padding_samples": transform.left_padding,
            "right_padding_samples": transform.right_padding,
            "max_roundtrip_absolute_error": max_abs_error,
            "worst_roundtrip_nrmse_db": worst_nrmse_db,
            "max_roundtrip_pcm_difference_lsb": max_pcm_difference,
            "roundtrip_pcm_mismatch_samples": pcm_mismatch_samples,
            "row_feature_records_sha256": sha256_bytes(
                canonical_json(representation_rows)
            ),
            "rows": representation_rows,
        },
        "controls": controls,
        "control_prediction_records": prediction_sets,
        "future_candidate_protocol": future_candidate_protocol(),
        "optimizer_steps": 0,
        "training_authorized": True,
        "quality_or_admission_authorized": False,
    }


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve(strict=True).parents[2]
    manifest_path = external_file(root, arguments.manifest, "dense acquisition manifest")
    manifest_bytes, manifest = read_json(manifest_path, "dense acquisition manifest")
    block_path, rows = validate_manifest(root, manifest_path, manifest)
    output, staging = prepare_output(root, arguments.output)
    try:
        report = run_preflight(
            root,
            manifest_path,
            manifest_bytes,
            manifest,
            block_path,
            rows,
            staging,
        )
        report_bytes = canonical_json(report)
        (staging / "preflight-report.json").write_bytes(report_bytes)
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    print(report_bytes.decode(), end="")


if __name__ == "__main__":
    main()
