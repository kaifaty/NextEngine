#!/usr/bin/env python3
"""Diagnose the rejected R2 field without changing its frozen decision."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import wave
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_listener_field_r2_evaluate as evaluation
import physical_sound_listener_field_r2_phase_aligned as phase

REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-failure-"
    "diagnostic.report.v1"
)
EVALUATION_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "evaluation.report.v1"
)
EVALUATION_MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "evaluation.manifest.v1"
)
TRAINING_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "training.report.v1"
)
BASELINE_SCHEMA = (
    "nextengine.experimental-physical-sound-transfer-field-baseline.report.v1"
)
CANDIDATE_ID = "listener_tanh_rank4_modal_latent_v1"
SPEED_OF_SOUND_METRES_PER_SECOND = 343.0
RELATIVE_MAGNITUDE_FLOOR_DB = -120.0


class DiagnosticError(RuntimeError):
    """The bounded post-rejection diagnostic failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--training-manifest", required=True, type=Path)
    parser.add_argument("--evaluation-manifest", required=True, type=Path)
    parser.add_argument("--evaluation-report", required=True, type=Path)
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


def read_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.resolve(strict=True).read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiagnosticError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise DiagnosticError(f"{label} must be one JSON object")
    return data, value


def resolve_ref(directory: Path, value: Any, label: str) -> tuple[Path, bytes]:
    if (
        not isinstance(value, dict)
        or set(value) != {"path", "sha256"}
        or not isinstance(value["path"], str)
        or not isinstance(value["sha256"], str)
    ):
        raise DiagnosticError(f"invalid {label} reference")
    unresolved = Path(value["path"])
    path = unresolved if unresolved.is_absolute() else directory / unresolved
    path = path.resolve(strict=True)
    data = path.read_bytes()
    if sha256_bytes(data) != value["sha256"]:
        raise DiagnosticError(f"{label} hash changed")
    return path, data


def read_pcm16(path: Path) -> tuple[int, np.ndarray]:
    with wave.open(str(path), "rb") as source:
        if (
            source.getnchannels() != 1
            or source.getsampwidth() != 2
            or source.getcomptype() != "NONE"
        ):
            raise DiagnosticError(f"WAV must be mono PCM16: {path}")
        sample_rate = source.getframerate()
        count = source.getnframes()
        payload = source.readframes(count)
    if len(payload) != 2 * count:
        raise DiagnosticError(f"truncated WAV: {path}")
    return sample_rate, np.frombuffer(payload, dtype="<i2").astype(np.float64) / 32768.0


def validated_inputs(
    training_manifest_path: Path, manifest_path: Path, report_path: Path
) -> dict[str, Any]:
    training_manifest_bytes, training_manifest = read_json(
        training_manifest_path, "training manifest"
    )
    manifest_bytes, manifest = read_json(manifest_path, "evaluation manifest")
    report_bytes, report = read_json(report_path, "evaluation report")
    if (
        manifest.get("schema") != EVALUATION_MANIFEST_SCHEMA
        or report.get("schema") != EVALUATION_SCHEMA
        or report.get("decision") != "RejectListenerField"
        or report.get("selected_candidate_id") is not None
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise DiagnosticError("evaluation is not the frozen phase-aligned rejection")
    directory = manifest_path.parent
    training_path, training_bytes = resolve_ref(
        directory, manifest["training_report"], "training report"
    )
    baseline_path, baseline_bytes = resolve_ref(
        directory, manifest["baseline_report"], "baseline report"
    )
    training = json.loads(training_bytes)
    baseline = json.loads(baseline_bytes)
    if (
        training.get("schema") != TRAINING_SCHEMA
        or training_manifest.get("schema") != phase.MANIFEST_SCHEMA
        or training.get("manifest_sha256")
        != sha256_bytes(training_manifest_bytes)
        or training.get("query_audio_bytes_read") != 0
        or training.get("method_holdout_or_shadow_bytes_read") != 0
        or baseline.get("schema") != BASELINE_SCHEMA
    ):
        raise DiagnosticError("training or baseline boundary changed")
    candidate = next(
        (
            candidate
            for candidate in training.get("candidates", [])
            if candidate.get("candidate_id") == CANDIDATE_ID
        ),
        None,
    )
    evaluated = next(
        (
            candidate
            for candidate in report.get("candidates", [])
            if candidate.get("candidate_id") == CANDIDATE_ID
        ),
        None,
    )
    if candidate is None or evaluated is None or evaluated.get(
        "passes_frozen_r2_rule"
    ):
        raise DiagnosticError("rank-4 rejected candidate is unavailable")
    return {
        "training_manifest_path": training_manifest_path,
        "training_manifest_bytes": training_manifest_bytes,
        "training_manifest": training_manifest,
        "manifest_path": manifest_path,
        "manifest_bytes": manifest_bytes,
        "manifest": manifest,
        "report_bytes": report_bytes,
        "report": report,
        "training_path": training_path,
        "training_bytes": training_bytes,
        "training": training,
        "candidate": candidate,
        "baseline_path": baseline_path,
        "baseline_bytes": baseline_bytes,
        "baseline": baseline,
    }


def spatial_profile(training: dict[str, Any], sample_rate: int) -> dict[str, float]:
    coordinates = sorted(
        float(row["listener_point_metres"][2]) for row in training["context_rows"]
    )
    spacings = [right - left for left, right in zip(coordinates, coordinates[1:])]
    if not spacings or min(spacings) <= 0.0:
        raise DiagnosticError("context microphone spacing is invalid")
    spacing = min(spacings)
    if any(abs(value - spacing) > 1.0e-12 for value in spacings):
        raise DiagnosticError("diagnostic requires a uniform context line")
    alias_frequency = SPEED_OF_SOUND_METRES_PER_SECOND / (2.0 * spacing)
    if alias_frequency >= sample_rate / 2.0:
        raise DiagnosticError("spatial alias frequency must be below Nyquist")
    return {
        "uniform_context_spacing_metres": spacing,
        "speed_of_sound_metres_per_second": SPEED_OF_SOUND_METRES_PER_SECOND,
        "spatial_nyquist_frequency_hz": alias_frequency,
    }


def gain_matched_band_error(
    candidate_magnitude: np.ndarray,
    reference_magnitude: np.ndarray,
    mask: np.ndarray,
) -> float:
    peak = float(np.max(reference_magnitude))
    if peak <= 0.0:
        raise DiagnosticError("reference spectrum is silent")
    floor = peak * 10.0 ** (RELATIVE_MAGNITUDE_FLOOR_DB / 20.0)
    selected = mask & (reference_magnitude >= floor)
    if not np.any(selected):
        raise DiagnosticError("frequency band has no supported reference bins")
    reference_db = 20.0 * np.log10(np.maximum(reference_magnitude[selected], floor))
    candidate_db = 20.0 * np.log10(np.maximum(candidate_magnitude[selected], floor))
    delta = candidate_db - reference_db
    gain = float(np.mean(delta))
    return math.sqrt(float(np.mean((delta - gain) ** 2)))


def spectrum(
    candidate: np.ndarray,
    reference: np.ndarray,
    sample_rate: int,
    alias_frequency: float,
) -> tuple[dict[str, float], float]:
    if candidate.shape != reference.shape:
        raise DiagnosticError("candidate and reference grids differ")
    window = np.hanning(reference.size)
    candidate_fft = np.abs(np.fft.rfft(candidate * window))
    reference_fft = np.abs(np.fft.rfft(reference * window))
    frequencies = np.fft.rfftfreq(reference.size, d=1.0 / sample_rate)
    bands = {
        "below_spatial_nyquist": frequencies <= alias_frequency,
        "one_to_four_times_spatial_nyquist": (frequencies > alias_frequency)
        & (frequencies <= 4.0 * alias_frequency),
        "above_four_times_spatial_nyquist": frequencies > 4.0 * alias_frequency,
    }
    errors = {
        name: gain_matched_band_error(candidate_fft, reference_fft, mask)
        for name, mask in bands.items()
    }
    weights = np.ones(reference_fft.size, dtype=np.float64)
    if weights.size > 2:
        weights[1:-1] = 2.0
    energy = weights * reference_fft**2
    high_fraction = float(np.sum(energy[frequencies > alias_frequency]) / np.sum(energy))
    return errors, high_fraction


def waveform_nrmse_db(candidate: np.ndarray, reference: np.ndarray) -> float:
    denominator = float(np.sum(reference**2))
    if denominator <= 0.0:
        raise DiagnosticError("reference waveform is silent")
    ratio = math.sqrt(float(np.sum((candidate - reference) ** 2)) / denominator)
    return -240.0 if ratio <= 1.0e-12 else 20.0 * math.log10(ratio)


def checked_audio(
    path: Path, expected_hash: str, expected_size: int, label: str
) -> tuple[int, np.ndarray]:
    resolved = path.resolve(strict=True)
    if sha256_file(resolved) != expected_hash or resolved.stat().st_size != expected_size:
        raise DiagnosticError(f"{label} identity changed")
    return read_pcm16(resolved)


def diagnostic_rows(inputs: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, float]]:
    query_refs = {
        binding["row_id"]: binding["reference_audio"]
        for binding in inputs["manifest"]["query_bindings"]
    }
    candidate_records = {
        record["row_id"]: record for record in inputs["candidate"]["predictions"]
    }
    baseline_rows = {row["row_id"]: row for row in inputs["baseline"]["rows"]}
    if set(query_refs) != set(candidate_records) or set(query_refs) != set(baseline_rows):
        raise DiagnosticError("diagnostic row identities differ")
    first_reference = Path(next(iter(query_refs.values()))["path"])
    sample_rate, _ = read_pcm16(first_reference)
    profile = spatial_profile(inputs["training"], sample_rate)
    alias_frequency = profile["spatial_nyquist_frequency_hz"]
    rows = []
    for row_id in sorted(query_refs):
        reference_path, reference_bytes = resolve_ref(
            inputs["manifest_path"].parent,
            query_refs[row_id],
            "query reference",
        )
        reference_rate, reference = read_pcm16(reference_path)
        candidate_record = candidate_records[row_id]
        candidate_path = inputs["training_path"].parent / candidate_record[
            "prediction_file"
        ]
        candidate_rate, candidate = checked_audio(
            candidate_path,
            candidate_record["prediction_sha256"],
            candidate_record["prediction_byte_count"],
            "phase candidate",
        )
        linear = baseline_rows[row_id]["linear_segment"]
        linear_path = inputs["baseline_path"].parent / linear["prediction_file"]
        linear_rate, linear_samples = checked_audio(
            linear_path,
            linear["prediction_sha256"],
            linear["prediction_byte_count"],
            "linear control",
        )
        if candidate_rate != reference_rate or linear_rate != reference_rate:
            raise DiagnosticError("diagnostic sample rates differ")
        candidate_errors, high_fraction = spectrum(
            candidate, reference, reference_rate, alias_frequency
        )
        linear_errors, control_high_fraction = spectrum(
            linear_samples, reference, reference_rate, alias_frequency
        )
        if abs(high_fraction - control_high_fraction) > 1.0e-15:
            raise DiagnosticError("reference energy calculation is inconsistent")
        rows.append(
            {
                "row_id": row_id,
                "reference_audio_sha256": sha256_bytes(reference_bytes),
                "reference_energy_fraction_above_spatial_nyquist": high_fraction,
                "phase_rank4_gain_matched_log_spectrum_rmse_db": candidate_errors,
                "linear_gain_matched_log_spectrum_rmse_db": linear_errors,
                "phase_rank4_better_by_band": {
                    band: candidate_errors[band] < linear_errors[band]
                    for band in candidate_errors
                },
            }
        )
    return rows, profile


def aggregate(rows: list[dict[str, Any]]) -> dict[str, Any]:
    bands = list(rows[0]["phase_rank4_gain_matched_log_spectrum_rmse_db"])
    return {
        "mean_reference_energy_fraction_above_spatial_nyquist": float(
            np.mean(
                [
                    row["reference_energy_fraction_above_spatial_nyquist"]
                    for row in rows
                ]
            )
        ),
        "bands": {
            band: {
                "phase_rank4_mean_rmse_db": float(
                    np.mean(
                        [
                            row["phase_rank4_gain_matched_log_spectrum_rmse_db"][band]
                            for row in rows
                        ]
                    )
                ),
                "linear_mean_rmse_db": float(
                    np.mean(
                        [
                            row["linear_gain_matched_log_spectrum_rmse_db"][band]
                            for row in rows
                        ]
                    )
                ),
                "phase_rank4_improved_row_count": sum(
                    row["phase_rank4_better_by_band"][band] for row in rows
                ),
                "row_count": len(rows),
            }
            for band in bands
        },
    }


def read_training_context(inputs: dict[str, Any]) -> tuple[int, np.ndarray, list[str]]:
    signals = []
    row_ids = []
    sample_rate = None
    sample_count = None
    directory = inputs["training_manifest_path"].parent
    for binding in inputs["training_manifest"]["context_bindings"]:
        path, _ = resolve_ref(directory, binding["audio"], "training context")
        rate, signal = read_pcm16(path)
        if sample_rate is None:
            sample_rate = rate
            sample_count = signal.size
        elif rate != sample_rate or signal.size != sample_count:
            raise DiagnosticError("training context PCM grid changed")
        signals.append(signal)
        row_ids.append(binding["row_id"])
    assert sample_rate is not None
    return sample_rate, np.stack(signals), row_ids


def read_query_references(
    inputs: dict[str, Any], sample_rate: int, sample_count: int
) -> tuple[np.ndarray, list[str]]:
    signals = []
    row_ids = []
    for binding in inputs["manifest"]["query_bindings"]:
        path, _ = resolve_ref(
            inputs["manifest_path"].parent,
            binding["reference_audio"],
            "query reference",
        )
        rate, signal = read_pcm16(path)
        if rate != sample_rate or signal.size != sample_count:
            raise DiagnosticError("query reference PCM grid changed")
        signals.append(signal)
        row_ids.append(binding["row_id"])
    return np.stack(signals), row_ids


def eigenspace(
    signals: np.ndarray, rank: int
) -> tuple[np.ndarray, np.ndarray, int, float]:
    mean = np.mean(signals, axis=0)
    centered = signals - mean
    eigenvalues, eigenvectors = np.linalg.eigh(centered @ centered.T)
    order = np.argsort(eigenvalues)[::-1]
    eigenvalues = np.maximum(eigenvalues[order], 0.0)
    eigenvectors = eigenvectors[:, order]
    threshold = max(float(eigenvalues[0]) * 1.0e-12, 1.0e-18)
    available = int(np.sum(eigenvalues > threshold))
    actual_rank = min(rank, available)
    if actual_rank < 1:
        raise DiagnosticError("context field has no non-constant rank")
    singular = np.sqrt(np.maximum(eigenvalues[:actual_rank], 1.0e-24))
    basis = (eigenvectors[:, :actual_rank].T @ centered) / singular[:, None]
    explained = float(
        np.sum(eigenvalues[:actual_rank]) / np.sum(eigenvalues[:available])
    )
    return mean, basis, available, explained


def field_energy_profile(signals: np.ndarray) -> dict[str, Any]:
    centered = signals - np.mean(signals, axis=0)
    eigenvalues = np.linalg.eigvalsh(centered @ centered.T)[::-1]
    eigenvalues = np.maximum(eigenvalues, 0.0)
    threshold = max(float(eigenvalues[0]) * 1.0e-12, 1.0e-18)
    available = int(np.sum(eigenvalues > threshold))
    total = float(np.sum(eigenvalues[:available]))
    return {
        "available_centered_rank": available,
        "rank4_cumulative_energy_fraction": float(np.sum(eigenvalues[:4]) / total),
        "rank7_cumulative_energy_fraction": float(np.sum(eigenvalues[:7]) / total),
    }


def frozen_oracle_metrics(
    root: Path,
    output: Path,
    rank: int,
    row_ids: list[str],
    predictions: np.ndarray,
    sample_rate: int,
    inputs: dict[str, Any],
) -> dict[str, Any]:
    references = {
        binding["row_id"]: Path(binding["reference_audio"]["path"])
        for binding in inputs["manifest"]["query_bindings"]
    }
    prediction_directory = output / f"query-informed-oracle-rank{rank}"
    prediction_directory.mkdir()
    entries = []
    paths = {}
    for row_id, samples in zip(row_ids, predictions, strict=True):
        candidate_path = prediction_directory / f"{row_id}.wav"
        candidate_bytes = phase.base.encode_wav(samples, sample_rate)
        candidate_path.write_bytes(candidate_bytes)
        reference_path = references[row_id].resolve(strict=True)
        entry_id = row_id.rsplit("-", 1)[-1]
        entries.append(
            {
                "id": entry_id,
                "object_id": "realimpact-green-goblet",
                "material": "glass-vessel-published-label",
                "impact_position": "fixed-mesh-vertex-31676",
                "force_band": "force-deconvolved-transfer",
                "expected_signal": "impact",
                "candidate": {
                    "path": str(candidate_path),
                    "sha256": sha256_bytes(candidate_bytes),
                },
                "reference": {
                    "path": str(reference_path),
                    "sha256": sha256_file(reference_path),
                },
            }
        )
        paths[entry_id] = (candidate_path, reference_path, row_id)
    manifest_value = {
        "schema": evaluation.RUST_MANIFEST_SCHEMA,
        "split": f"r2-query-informed-oracle-rank{rank}",
        "relations": [],
        "entries": entries,
    }
    manifest_bytes = canonical_json(manifest_value)
    manifest_path = output / f"query-informed-oracle-rank{rank}.rust-manifest.json"
    manifest_path.write_bytes(manifest_bytes)
    rust_bytes, rust_report = evaluation.run_rust_evaluator(
        root, manifest_path, output / f"query-informed-oracle-rank{rank}.rust-eval"
    )
    rows = []
    for entry in rust_report["entries"]:
        matched = entry.get("matched")
        if matched is None:
            raise DiagnosticError("Rust oracle evaluator omitted matched metrics")
        candidate_path, reference_path, row_id = paths[entry["id"]]
        rows.append(
            {
                "row_id": row_id,
                "absolute_rms_level_error_db": abs(matched["raw_rms_delta_db"]),
                "gain_matched_multiresolution_log_spectrum_rmse_db": matched[
                    "gain_matched_multiresolution_log_spectrum_rmse_db"
                ],
                "normalized_waveform_rmse_db": evaluation.normalized_waveform_rmse_db(
                    candidate_path, reference_path
                ),
            }
        )
    rows.sort(key=lambda row: row["row_id"])
    candidate = {
        "candidate_id": f"query_informed_subspace_oracle_rank{rank}",
        "rank": rank,
        "metric_input_content_sha256": sha256_bytes(
            canonical_json(
                [
                    {
                        "row_id": paths[entry["id"]][2],
                        "candidate_sha256": entry["candidate"]["sha256"],
                        "reference_sha256": entry["reference"]["sha256"],
                    }
                    for entry in entries
                ]
            )
        ),
        "normalized_rust_metric_rows_sha256": sha256_bytes(canonical_json(rows)),
        "query_count": len(rows),
        "aggregate": evaluation.aggregate(rows),
        "rows": rows,
    }
    return evaluation.comparison(candidate, inputs["baseline"]["controls"])


def subspace_oracle(
    root: Path, output: Path, inputs: dict[str, Any], alias_frequency: float
) -> dict[str, Any]:
    sample_rate, context, context_ids = read_training_context(inputs)
    query, query_ids = read_query_references(inputs, sample_rate, context.shape[1])
    geometry = inputs["training"]["phase_geometry"]
    context_delays = phase.delays_for(context_ids, geometry)
    query_delays = phase.delays_for(query_ids, geometry)
    aligned_context = phase.align_context(
        context, context_delays, sample_rate, geometry
    )
    aligned_query = phase.align_context(query, query_delays, sample_rate, geometry)
    profiles = {}
    for rank in (4, 7):
        mean, basis, available, explained = eigenspace(aligned_context, rank)
        coefficients = (aligned_query - mean) @ basis.T
        aligned_prediction = mean + coefficients @ basis
        prediction = phase.restore_delay(
            aligned_prediction,
            query_delays,
            sample_rate,
            context.shape[1],
            geometry,
        )
        frozen_metrics = frozen_oracle_metrics(
            root,
            output,
            rank,
            query_ids,
            prediction,
            sample_rate,
            inputs,
        )
        rows = []
        for row_id, candidate, reference in zip(
            query_ids, prediction, query, strict=True
        ):
            errors, _ = spectrum(
                candidate, reference, sample_rate, alias_frequency
            )
            rows.append(
                {
                    "row_id": row_id,
                    "gain_matched_log_spectrum_rmse_db": errors,
                    "normalized_waveform_rmse_db": waveform_nrmse_db(
                        candidate, reference
                    ),
                }
            )
        profiles[f"rank{rank}"] = {
            "available_context_centered_rank": available,
            "context_cumulative_energy_fraction": explained,
            "mean_normalized_waveform_rmse_db": float(
                np.mean([row["normalized_waveform_rmse_db"] for row in rows])
            ),
            "mean_gain_matched_log_spectrum_rmse_db_by_band": {
                band: float(
                    np.mean(
                        [
                            row["gain_matched_log_spectrum_rmse_db"][band]
                            for row in rows
                        ]
                    )
                )
                for band in rows[0]["gain_matched_log_spectrum_rmse_db"]
            },
            "frozen_r2_metric_comparison": frozen_metrics,
            "rows": rows,
        }
    return {
        "purpose": (
            "query-informed_orthogonal_projection_lower_bound_only_not_an_"
            "inference_candidate"
        ),
        "context_field": field_energy_profile(aligned_context),
        "all_fifteen_row_field": field_energy_profile(
            np.concatenate((aligned_context, aligned_query), axis=0)
        ),
        "profiles": profiles,
    }


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve(strict=True).parents[2]
    training_manifest_path = arguments.training_manifest.resolve(strict=True)
    manifest_path = arguments.evaluation_manifest.resolve(strict=True)
    report_path = arguments.evaluation_report.resolve(strict=True)
    if (
        training_manifest_path.is_relative_to(root)
        or manifest_path.is_relative_to(root)
        or report_path.is_relative_to(root)
    ):
        raise DiagnosticError("diagnostic evidence must remain external")
    output_parent = arguments.output.parent.resolve(strict=True)
    output = output_parent / arguments.output.name
    if output.is_relative_to(root) or output.exists():
        raise DiagnosticError("diagnostic output must be a new external directory")
    output.mkdir()
    inputs = validated_inputs(training_manifest_path, manifest_path, report_path)
    rows, spatial = diagnostic_rows(inputs)
    oracle = subspace_oracle(
        root, output, inputs, spatial["spatial_nyquist_frequency_hz"]
    )
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Diagnosed",
        "decision": "ResearchEvidenceOnly",
        "claim": (
            "POST_REJECTION_SPATIAL_SAMPLING_DIAGNOSTIC_ONLY / NO_MODEL_"
            "SELECTION_THRESHOLD_CHANGE_OR_ADMISSION_AUTHORITY"
        ),
        "runner_sha256": sha256_file(Path(__file__).resolve(strict=True)),
        "phase_implementation_sha256": sha256_file(
            Path(phase.__file__).resolve(strict=True)
        ),
        "training_manifest_sha256": sha256_bytes(
            inputs["training_manifest_bytes"]
        ),
        "evaluation_manifest_sha256": sha256_bytes(inputs["manifest_bytes"]),
        "evaluation_report_sha256": sha256_bytes(inputs["report_bytes"]),
        "training_report_sha256": sha256_bytes(inputs["training_bytes"]),
        "baseline_report_sha256": sha256_bytes(inputs["baseline_bytes"]),
        "candidate_id": CANDIDATE_ID,
        "frequency_profile": {
            **spatial,
            "bands": [
                "below_spatial_nyquist",
                "one_to_four_times_spatial_nyquist",
                "above_four_times_spatial_nyquist",
            ],
            "analysis": "full_record_hann_rfft_log_magnitude",
            "gain_match": "per_row_per_band_mean_log_delta_removed",
            "relative_magnitude_floor_db": RELATIVE_MAGNITUDE_FLOOR_DB,
        },
        "aggregate": aggregate(rows),
        "rows": rows,
        "subspace_oracle": oracle,
        "query_audio_rows_read": len(rows),
        "method_holdout_or_shadow_bytes_read": 0,
        "selection_or_runtime_authorized": False,
    }
    report_bytes = canonical_json(report)
    (output / "diagnostic-report.json").write_bytes(report_bytes)
    print(report_bytes.decode(), end="")


if __name__ == "__main__":
    main()
