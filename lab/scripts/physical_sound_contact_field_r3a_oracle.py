#!/usr/bin/env python3
"""Run the frozen query-seeing R3A compact-representation oracle."""

from __future__ import annotations

import argparse
import io
import math
import wave
from pathlib import Path
from typing import Any

import numpy as np
from scipy import fft, signal

import physical_sound_contact_field_r3a_common as common

MODAL_COUNT = 32
RESIDUAL_DCT_COUNT = 192
ALTERNATIVE_DCT_COUNT = 256
SCALAR_BUDGET = 512
MODAL_FIT_SAMPLES = 96_000
MODAL_FIT_STRIDE = 4
SPECTRAL_FLOOR_DB = -100.0
PRIMARY_ENDPOINTS = (
    "absolute_rms_level_error_db",
    "gain_matched_multiresolution_log_spectrum_rmse_db",
    "normalized_envelope_rmse",
    "modal_frequency_median_error_cents",
    "decay_t60_relative_error",
)
ABSOLUTE_THRESHOLDS = {
    "absolute_rms_level_error_db": 0.5,
    "gain_matched_multiresolution_log_spectrum_rmse_db": 4.0,
    "normalized_envelope_rmse": 0.20,
    "modal_frequency_median_error_cents": 100.0,
    "decay_t60_relative_error": 0.35,
}


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--contacts", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sparse_dct_reconstruction(
    value: np.ndarray, count: int
) -> tuple[np.ndarray, dict[str, Any]]:
    coefficients = fft.dct(value, type=2, norm="ortho")
    order = np.argsort(-np.abs(coefficients), kind="stable")[:count]
    sparse = np.zeros_like(coefficients)
    sparse[order] = coefficients[order]
    reconstructed = fft.idct(sparse, type=2, norm="ortho")
    return reconstructed, {
        "coefficient_count": count,
        "encoded_scalar_count": count * 2,
        "index_semantics": "integer_dct_bin_plus_float64_value",
    }


def modal_parameters(value: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    frequencies, times, spectrum = signal.stft(
        value,
        fs=common.SAMPLE_RATE_HZ,
        window="hann",
        nperseg=4_096,
        noverlap=3_072,
        boundary=None,
        padded=False,
    )
    magnitude = np.abs(spectrum)
    summary = np.sqrt(np.mean(np.square(magnitude), axis=1))
    eligible = (frequencies >= 120.0) & (frequencies <= 18_000.0)
    peaks, _ = signal.find_peaks(summary, distance=4)
    peaks = peaks[eligible[peaks]]
    order = peaks[np.argsort(-summary[peaks], kind="stable")[:MODAL_COUNT]]
    order = order[np.argsort(frequencies[order], kind="stable")]
    if order.size == 0:
        raise common.R3AError("target exposes no modal bins")

    damping = []
    for index in order:
        envelope = np.maximum(magnitude[index], np.max(magnitude[index]) * 1.0e-5)
        usable = envelope >= np.max(envelope) * 1.0e-3
        usable &= times <= min(times[-1], 2.5)
        if np.count_nonzero(usable) >= 4:
            slope = np.polyfit(times[usable], np.log(envelope[usable]), 1)[0]
            damping.append(float(np.clip(-slope, 0.25, 200.0)))
        else:
            damping.append(20.0)
    return frequencies[order].astype(np.float64), np.asarray(damping, dtype=np.float64)


def modal_residual_reconstruction(
    value: np.ndarray,
) -> tuple[np.ndarray, dict[str, Any]]:
    frequencies, damping = modal_parameters(value)
    active_modes = frequencies.size
    fit_indices = np.arange(
        common.PEAK_ALIGNMENT_SAMPLE, MODAL_FIT_SAMPLES, MODAL_FIT_STRIDE
    )
    fit_time = (fit_indices - common.PEAK_ALIGNMENT_SAMPLE) / common.SAMPLE_RATE_HZ
    angular = 2.0 * np.pi * frequencies[None, :] * fit_time[:, None]
    decay = np.exp(-fit_time[:, None] * damping[None, :])
    basis = np.concatenate([decay * np.cos(angular), decay * np.sin(angular)], axis=1)
    gains, _, _, _ = np.linalg.lstsq(basis, value[fit_indices], rcond=1.0e-10)

    time = (
        np.maximum(
            np.arange(value.size, dtype=np.float64) - common.PEAK_ALIGNMENT_SAMPLE,
            0.0,
        )
        / common.SAMPLE_RATE_HZ
    )
    modal = np.zeros_like(value)
    for index, (frequency, decay_rate) in enumerate(
        zip(frequencies, damping, strict=True)
    ):
        phase = 2.0 * np.pi * frequency * time
        envelope = np.exp(-decay_rate * time)
        modal += envelope * (
            gains[index] * np.cos(phase) + gains[index + active_modes] * np.sin(phase)
        )
    modal[: common.PEAK_ALIGNMENT_SAMPLE] = 0.0
    residual = value - modal
    residual_reconstruction, residual_record = sparse_dct_reconstruction(
        residual, RESIDUAL_DCT_COUNT
    )
    return modal + residual_reconstruction, {
        "mode_count": MODAL_COUNT,
        "active_mode_count": int(active_modes),
        "mode_scalar_count": MODAL_COUNT * 4,
        "mode_fields": [
            "frequency_hz",
            "damping_nepers_per_second",
            "cos_gain",
            "sin_gain",
        ],
        "residual": residual_record,
        "encoded_scalar_count": MODAL_COUNT * 4
        + residual_record["encoded_scalar_count"],
        "frequency_hz": frequencies.tolist(),
        "damping_nepers_per_second": damping.tolist(),
    }


def rms(value: np.ndarray) -> float:
    return float(np.sqrt(np.mean(np.square(value))))


def level_error(target: np.ndarray, candidate: np.ndarray) -> float:
    return abs(
        20.0 * math.log10(max(rms(candidate), 1.0e-15) / max(rms(target), 1.0e-15))
    )


def log_spectrum_error(target: np.ndarray, candidate: np.ndarray) -> float:
    errors = []
    for length in (2_048, 8_192, 32_768):
        hop = length // 2
        window = np.hanning(length)
        target_frames = np.stack(
            [
                target[start : start + length] * window
                for start in range(0, target.size - length + 1, hop)
            ]
        )
        candidate_frames = np.stack(
            [
                candidate[start : start + length] * window
                for start in range(0, candidate.size - length + 1, hop)
            ]
        )
        target_magnitude = np.abs(np.fft.rfft(target_frames, axis=1))
        candidate_magnitude = np.abs(np.fft.rfft(candidate_frames, axis=1))
        floor = max(
            float(np.max(target_magnitude)) * 10.0 ** (SPECTRAL_FLOOR_DB / 20.0),
            1.0e-15,
        )
        target_db = 20.0 * np.log10(np.maximum(target_magnitude, floor))
        candidate_db = 20.0 * np.log10(np.maximum(candidate_magnitude, floor))
        difference = candidate_db - target_db
        difference -= np.mean(difference)
        errors.append(float(np.sqrt(np.mean(np.square(difference)))))
    return float(np.mean(errors))


def smoothed_envelope(value: np.ndarray) -> np.ndarray:
    envelope = np.abs(signal.hilbert(value))
    width = common.SAMPLE_RATE_HZ // 200
    return np.convolve(envelope, np.ones(width) / width, mode="same")


def envelope_error(target: np.ndarray, candidate: np.ndarray) -> float:
    target_envelope = smoothed_envelope(target)
    candidate_envelope = smoothed_envelope(candidate)
    scale = max(float(np.max(target_envelope)), 1.0e-15)
    return float(
        np.sqrt(np.mean(np.square((candidate_envelope - target_envelope) / scale)))
    )


def dominant_frequencies(value: np.ndarray, count: int = 12) -> np.ndarray:
    magnitude = np.abs(np.fft.rfft(value * np.hanning(value.size)))
    frequencies = np.fft.rfftfreq(value.size, 1.0 / common.SAMPLE_RATE_HZ)
    eligible = (frequencies >= 120.0) & (frequencies <= 18_000.0)
    peaks, _ = signal.find_peaks(
        magnitude, distance=max(1, value.size // common.SAMPLE_RATE_HZ * 8)
    )
    peaks = peaks[eligible[peaks]]
    if peaks.size < count:
        peaks = np.flatnonzero(eligible)
    selected = peaks[np.argsort(-magnitude[peaks], kind="stable")[:count]]
    return np.sort(frequencies[selected])


def modal_frequency_error(target: np.ndarray, candidate: np.ndarray) -> float:
    target_frequencies = dominant_frequencies(target)
    candidate_frequencies = dominant_frequencies(candidate)
    errors = []
    remaining = list(candidate_frequencies)
    for frequency in target_frequencies:
        index = int(np.argmin(np.abs(np.asarray(remaining) - frequency)))
        matched = remaining.pop(index)
        errors.append(abs(1_200.0 * math.log2(matched / frequency)))
    return float(np.median(errors))


def decay_t60(value: np.ndarray) -> float:
    start = common.PEAK_ALIGNMENT_SAMPLE
    energy = np.square(value[start:])
    schroeder = np.cumsum(energy[::-1])[::-1]
    if schroeder[0] <= 0.0:
        return 0.0
    db = 10.0 * np.log10(np.maximum(schroeder / schroeder[0], 1.0e-12))
    times = np.arange(db.size) / common.SAMPLE_RATE_HZ
    usable = (db <= -5.0) & (db >= -35.0)
    if np.count_nonzero(usable) < 16:
        return float(value.size / common.SAMPLE_RATE_HZ)
    slope = np.polyfit(times[usable], db[usable], 1)[0]
    return (
        float(-60.0 / slope)
        if slope < -1.0e-9
        else float(value.size / common.SAMPLE_RATE_HZ)
    )


def metrics(target: np.ndarray, candidate: np.ndarray) -> dict[str, float]:
    target_t60 = decay_t60(target)
    candidate_t60 = decay_t60(candidate)
    return {
        "absolute_rms_level_error_db": level_error(target, candidate),
        "gain_matched_multiresolution_log_spectrum_rmse_db": log_spectrum_error(
            target, candidate
        ),
        "normalized_envelope_rmse": envelope_error(target, candidate),
        "modal_frequency_median_error_cents": modal_frequency_error(target, candidate),
        "decay_t60_relative_error": abs(candidate_t60 - target_t60)
        / max(target_t60, 1.0e-9),
    }


def gate(candidate: dict[str, float], baseline: dict[str, float]) -> dict[str, Any]:
    strictly_beats_baseline = {
        endpoint: candidate[endpoint] < baseline[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    within_absolute_threshold = {
        endpoint: candidate[endpoint] <= ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    normalized_candidate = sum(
        candidate[endpoint] / ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    )
    normalized_baseline = sum(
        baseline[endpoint] / ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    )
    nonregressing_preserved_endpoint = {
        endpoint: strictly_beats_baseline[endpoint]
        or candidate[endpoint] <= 0.25 * ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    passed = (
        sum(strictly_beats_baseline.values()) >= 4
        and all(within_absolute_threshold.values())
        and all(nonregressing_preserved_endpoint.values())
        and normalized_candidate < 0.8 * normalized_baseline
    )
    return {
        "passed": passed,
        "strictly_beats_nearest_fit_by_endpoint": strictly_beats_baseline,
        "strictly_better_endpoint_count": sum(strictly_beats_baseline.values()),
        "within_absolute_threshold_on_every_endpoint": within_absolute_threshold,
        "nonregressing_preserved_endpoint": nonregressing_preserved_endpoint,
        "normalized_candidate_error_sum": normalized_candidate,
        "normalized_baseline_error_sum": normalized_baseline,
        "required_normalized_improvement_ratio": 0.8,
    }


def evaluate(waveforms: np.ndarray, positions: np.ndarray) -> dict[str, Any]:
    if waveforms.shape != (4, common.SAMPLE_COUNT) or positions.shape != (4, 3):
        raise common.R3AError(
            "oracle requires exactly three fit and one development contact"
        )
    aligned = np.stack([common.align_peak(row) for row in waveforms])
    fit_peak = float(np.max(np.abs(aligned[:3])))
    if not np.isfinite(fit_peak) or fit_peak <= 0.0:
        raise common.R3AError("fit-only normalization peak is invalid")
    normalized = aligned * (0.92 / fit_peak)
    analysis = normalized[:, : common.ANALYSIS_SAMPLES]
    target = analysis[3]
    distances = np.linalg.norm(positions[:3] - positions[3], axis=1)
    nearest_index = int(np.argmin(distances))
    baseline = analysis[nearest_index]
    modal_candidate, modal_record = modal_residual_reconstruction(target)
    alternative, alternative_record = sparse_dct_reconstruction(
        target, ALTERNATIVE_DCT_COUNT
    )
    if (
        modal_record["encoded_scalar_count"] != SCALAR_BUDGET
        or alternative_record["encoded_scalar_count"] != SCALAR_BUDGET
    ):
        raise common.R3AError("representation scalar budget changed")
    baseline_metrics = metrics(target, baseline)
    modal_metrics = metrics(target, modal_candidate)
    alternative_metrics = metrics(target, alternative)
    modal_gate = gate(modal_metrics, baseline_metrics)
    alternative_gate = gate(alternative_metrics, baseline_metrics)
    passed = modal_gate["passed"] or alternative_gate["passed"]
    return {
        "decision": "READY_FOR_EXACT_OBJECT_FIELD"
        if passed
        else "REJECT_REPRESENTATION",
        "fit_only_scale": 0.92 / fit_peak,
        "nearest_fit_contact_index": nearest_index,
        "nearest_fit_distance_metres": float(distances[nearest_index]),
        "baseline": {"id": "nearest_fit_contact", "metrics": baseline_metrics},
        "modal_plus_residual": {
            "id": "damped_modes_32_plus_sparse_dct_residual_192_v1",
            "record": modal_record,
            "metrics": modal_metrics,
            "gate": modal_gate,
        },
        "alternative": {
            "id": "sparse_dct_256_v1",
            "record": alternative_record,
            "metrics": alternative_metrics,
            "gate": alternative_gate,
        },
        "signals": {
            "target": target,
            "nearest_fit": baseline,
            "modal_plus_residual": modal_candidate,
            "alternative": alternative,
        },
    }


def pcm16_wav(value: np.ndarray) -> bytes:
    peak = max(float(np.max(np.abs(value))), 1.0e-15)
    samples = np.round(np.clip(value / peak * 0.95, -1.0, 1.0) * 32_767.0).astype("<i2")
    output = io.BytesIO()
    with wave.open(output, "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(common.SAMPLE_RATE_HZ)
        handle.writeframes(samples.tobytes())
    return output.getvalue()


def run(
    root: Path, manifest_argument: Path, contacts_argument: Path, output_argument: Path
) -> Path:
    manifest_path = common.external_file(root, manifest_argument, "R3A manifest")
    manifest_bytes, manifest = common.load_json(manifest_path, "R3A manifest")
    common.validate_manifest(manifest)
    common.validate_implementation(manifest, Path(__file__).resolve().parent)
    contacts_directory = contacts_argument.resolve(strict=True)
    if contacts_directory.is_relative_to(root) or not contacts_directory.is_dir():
        raise common.R3AError("contacts must be an external extraction directory")
    report_bytes, extraction_report = common.load_json(
        contacts_directory / "report.json", "R3A extraction report"
    )
    metadata_bytes, metadata = common.load_json(
        contacts_directory / "contacts-metadata.json", "R3A contacts metadata"
    )
    if (
        extraction_report.get("schema") != common.EXTRACTION_REPORT_SCHEMA
        or extraction_report.get("manifest_sha256")
        != common.sha256_bytes(manifest_bytes)
        or extraction_report.get("sealed_waveform_samples_decoded") != 0
        or extraction_report.get("waveforms_sha256")
        != common.sha256_file(contacts_directory / "contacts.npy")
        or extraction_report.get("contacts_metadata_sha256")
        != common.sha256_bytes(metadata_bytes)
    ):
        raise common.R3AError("R3A contact extraction lineage changed")
    with (contacts_directory / "contacts.npy").open("rb") as handle:
        waveforms = np.load(handle, allow_pickle=False)
    positions = np.asarray(
        [contact["position_metres"] for contact in metadata["contacts"]],
        dtype=np.float64,
    )
    result = evaluate(np.asarray(waveforms, dtype=np.float64), positions)
    signals = result.pop("signals")
    output, staging = common.prepare_output(root, output_argument)
    try:
        artifact_hashes = {}
        for name, value in signals.items():
            filename = f"{name.replace('_', '-')}.wav"
            payload = pcm16_wav(value)
            (staging / filename).write_bytes(payload)
            artifact_hashes[filename] = common.sha256_bytes(payload)
        report = {
            "schema": common.ORACLE_REPORT_SCHEMA,
            "status": "Validated",
            "decision": result["decision"],
            "profile": common.PROFILE_ID,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "extraction_report_sha256": common.sha256_bytes(report_bytes),
            "oracle_runner_sha256": common.sha256_file(Path(__file__).resolve()),
            "query_visibility": "representation_development_only_query_seeing_oracle",
            "field_holdout_waveform_accessed": False,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "sample_rate_hz": common.SAMPLE_RATE_HZ,
            "analysis_samples": common.ANALYSIS_SAMPLES,
            "primary_endpoints": list(PRIMARY_ENDPOINTS),
            "absolute_thresholds": ABSOLUTE_THRESHOLDS,
            "result": result,
            "audition_artifact_sha256": artifact_hashes,
            "neural_training_authorized": result["decision"]
            == "READY_FOR_EXACT_OBJECT_FIELD",
        }
        (staging / "report.json").write_bytes(common.canonical_json(report))
        common.publish_output(output, staging)
    except BaseException:
        for child in staging.iterdir():
            child.unlink()
        staging.rmdir()
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    output = run(root, arguments.manifest, arguments.contacts, arguments.output)
    _, report = common.load_json(output / "report.json", "R3A oracle report")
    print(f"R3A representation oracle: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
