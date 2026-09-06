#!/usr/bin/env python3
"""Run the frozen query-seeing R3A V2 learned-codec representation oracle."""

from __future__ import annotations

import argparse
import hashlib
import io
import math
import wave
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v2_common as common
from scipy import signal

EVALUATION_MIN_HZ = 120.0
EVALUATION_MAX_HZ = 18_000.0
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
RESAMPLING_CONTROL_THRESHOLDS = {
    "absolute_rms_level_error_db": 0.10,
    "gain_matched_multiresolution_log_spectrum_rmse_db": 0.75,
    "normalized_envelope_rmse": 0.02,
    "modal_frequency_median_error_cents": 10.0,
    "decay_t60_relative_error": 0.05,
}


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--contacts", required=True, type=Path)
    parser.add_argument("--dac-weights", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def configure_torch(torch: Any) -> None:
    torch.set_num_threads(1)
    if torch.get_num_interop_threads() != 1:
        try:
            torch.set_num_interop_threads(1)
        except RuntimeError as error:
            raise common.R3AV2Error("cannot freeze torch interop threads") from error
    torch.use_deterministic_algorithms(True)
    torch.backends.mkldnn.enabled = False


def load_model(weights: Path) -> tuple[Any, Any]:
    common.validate_dac_weights(weights)
    import dac
    import torch

    configure_torch(torch)
    model = dac.DAC.load(str(weights)).to("cpu").eval()
    identity = {
        "sample_rate": int(model.sample_rate),
        "hop_length": int(model.hop_length),
        "n_codebooks": int(model.n_codebooks),
        "codebook_size": int(model.codebook_size),
    }
    expected = {
        "sample_rate": common.DAC_MODEL_SAMPLE_RATE_HZ,
        "hop_length": common.DAC_MODEL_HOP_LENGTH,
        "n_codebooks": common.DAC_MODEL_CODEBOOKS,
        "codebook_size": common.DAC_MODEL_CODEBOOK_SIZE,
    }
    if identity != expected:
        raise common.R3AV2Error(f"loaded DAC model identity changed: {identity}")
    return torch, model


def reconstruct_codes(
    torch: Any, model: Any, value: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    if value.ndim != 1 or value.dtype != np.float32 or not np.all(np.isfinite(value)):
        raise common.R3AV2Error("DAC input must be one finite float32 waveform")
    tensor = torch.from_numpy(np.ascontiguousarray(value)).reshape(1, 1, -1)
    with torch.inference_mode():
        padded = model.preprocess(tensor, common.DAC_MODEL_SAMPLE_RATE_HZ)
        _, codes, _, _, _ = model.encode(
            padded, n_quantizers=common.DAC_MODEL_CODEBOOKS
        )
        quantized = model.quantizer.from_codes(codes)[0]
        decoded = model.decode(quantized)[..., : value.size]
    return (
        np.ascontiguousarray(decoded.reshape(-1).cpu().numpy(), dtype=np.float32),
        np.ascontiguousarray(codes.cpu().numpy(), dtype=np.int64),
    )


def warm_model(torch: Any, model: Any) -> None:
    warmup = np.zeros(8_192, dtype=np.float32)
    reconstructed, codes = reconstruct_codes(torch, model, warmup)
    if reconstructed.shape != warmup.shape or codes.shape[:2] != (1, 9):
        raise common.R3AV2Error("DAC synthetic warm-up failed")


def synthetic_control(weights: Path) -> dict[str, Any]:
    torch, model = load_model(weights)
    warm_model(torch, model)
    sample_count = 16_384
    time = np.arange(sample_count, dtype=np.float32) / common.DAC_MODEL_SAMPLE_RATE_HZ
    value = (
        0.6 * np.exp(-5.0 * time) * np.sin(2.0 * np.pi * 1_320.0 * time)
        + 0.2 * np.exp(-12.0 * time) * np.sin(2.0 * np.pi * 3_780.0 * time)
    ).astype(np.float32)
    first, first_codes = reconstruct_codes(torch, model, value)
    second, second_codes = reconstruct_codes(torch, model, value)
    if not np.array_equal(first, second) or not np.array_equal(
        first_codes, second_codes
    ):
        raise common.R3AV2Error("DAC is not exact after frozen synthetic warm-up")
    return {
        "device": "cpu",
        "torch_deterministic_algorithms": True,
        "mkldnn_enabled": False,
        "intraop_threads": 1,
        "interop_threads": 1,
        "warmup_samples": 8_192,
        "control_samples": sample_count,
        "repeat_policy": "discard_warmup_then_require_exact_two_run_arrays",
        "code_shape": list(first_codes.shape),
        "codes_sha256": hashlib.sha256(first_codes.tobytes()).hexdigest(),
        "decoded_sha256": hashlib.sha256(first.tobytes()).hexdigest(),
        "exact_repeat": True,
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
        frequencies = np.fft.rfftfreq(length, 1.0 / common.SAMPLE_RATE_HZ)
        selected = (frequencies >= EVALUATION_MIN_HZ) & (
            frequencies <= EVALUATION_MAX_HZ
        )
        target_magnitude = np.abs(np.fft.rfft(target_frames, axis=1))[:, selected]
        candidate_magnitude = np.abs(np.fft.rfft(candidate_frames, axis=1))[:, selected]
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
    eligible = (frequencies >= EVALUATION_MIN_HZ) & (frequencies <= EVALUATION_MAX_HZ)
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


def candidate_gate(
    candidate: dict[str, float], baseline: dict[str, float]
) -> dict[str, Any]:
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


def resampling_gate(control: dict[str, float]) -> dict[str, Any]:
    endpoints = {
        endpoint: control[endpoint] <= RESAMPLING_CONTROL_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    return {"passed": all(endpoints.values()), "within_threshold": endpoints}


def resample_to_dac(value: np.ndarray) -> np.ndarray:
    result = signal.resample_poly(
        value,
        up=147,
        down=160,
        window=("kaiser", 5.0),
        padtype="constant",
    )
    return np.ascontiguousarray(result, dtype=np.float32)


def resample_to_source(value: np.ndarray, sample_count: int) -> np.ndarray:
    result = signal.resample_poly(
        value,
        up=160,
        down=147,
        window=("kaiser", 5.0),
        padtype="constant",
    )
    if result.size < sample_count:
        result = np.pad(result, (0, sample_count - result.size))
    return np.ascontiguousarray(result[:sample_count], dtype=np.float64)


def evaluate(
    waveforms: np.ndarray,
    positions: np.ndarray,
    weights: Path,
) -> dict[str, Any]:
    if waveforms.shape != (4, common.SAMPLE_COUNT) or positions.shape != (4, 3):
        raise common.R3AV2Error("oracle requires three fit and one development contact")
    aligned = np.stack([common.align_peak(row) for row in waveforms])
    fit_peak = float(np.max(np.abs(aligned[:3])))
    if not np.isfinite(fit_peak) or fit_peak <= 0.0:
        raise common.R3AV2Error("fit-only normalization peak is invalid")
    normalized = aligned * (0.92 / fit_peak)
    analysis = normalized[:, : common.ANALYSIS_SAMPLES]
    target = analysis[3]
    distances = np.linalg.norm(positions[:3] - positions[3], axis=1)
    nearest_index = int(np.argmin(distances))
    baseline = analysis[nearest_index]

    dac_input = resample_to_dac(target)
    resampling_control = resample_to_source(dac_input, target.size)
    torch, model = load_model(weights)
    warm_model(torch, model)
    first, first_codes = reconstruct_codes(torch, model, dac_input)
    second, second_codes = reconstruct_codes(torch, model, dac_input)
    exact_repeat = np.array_equal(first, second) and np.array_equal(
        first_codes, second_codes
    )
    if not exact_repeat:
        raise common.R3AV2Error("DAC target reconstruction is not exact after warm-up")
    candidate = resample_to_source(second, target.size)

    baseline_metrics = metrics(target, baseline)
    resampling_metrics = metrics(target, resampling_control)
    candidate_metrics = metrics(target, candidate)
    resampling_result = resampling_gate(resampling_metrics)
    codec_result = candidate_gate(candidate_metrics, baseline_metrics)
    if not resampling_result["passed"]:
        decision = "INCONCLUSIVE_RESAMPLING_CONTROL"
    elif not codec_result["passed"]:
        decision = "REJECT_LEARNED_CODEC_REPRESENTATION"
    else:
        decision = "LEARNED_CODEC_REPRESENTATION_SUPPORTED"

    frame_count = int(second_codes.shape[-1])
    code_count = int(second_codes.size)
    bits_per_code = math.ceil(math.log2(common.DAC_MODEL_CODEBOOK_SIZE))
    encoded_bits = code_count * bits_per_code
    return {
        "decision": decision,
        "r3a_exit_decision": (
            "BLOCKED_PENDING_BOUNDED_DETERMINISTIC_COOKER"
            if decision == "LEARNED_CODEC_REPRESENTATION_SUPPORTED"
            else decision
        ),
        "fit_only_scale": 0.92 / fit_peak,
        "nearest_fit_contact_index": nearest_index,
        "nearest_fit_distance_metres": float(distances[nearest_index]),
        "baseline": {"id": "nearest_fit_contact", "metrics": baseline_metrics},
        "resampling_control": {
            "id": "polyphase_48k_44k1_48k_without_codec",
            "metrics": resampling_metrics,
            "gate": resampling_result,
        },
        "learned_codec": {
            "id": "descript_audio_codec_44khz_8kbps_0.0.1",
            "metrics": candidate_metrics,
            "gate": codec_result,
            "device": "cpu",
            "warmup_then_two_exact_repeats": exact_repeat,
            "code_shape": list(second_codes.shape),
            "code_frame_count": frame_count,
            "code_count": code_count,
            "bits_per_code": bits_per_code,
            "ideal_bitpacked_bytes": (encoded_bits + 7) // 8,
            "effective_bits_per_second": encoded_bits
            / (target.size / common.SAMPLE_RATE_HZ),
            "codes_sha256": hashlib.sha256(second_codes.tobytes()).hexdigest(),
            "decoded_44k1_sha256": hashlib.sha256(second.tobytes()).hexdigest(),
            "shared_decoder_weights_bytes": common.DAC_WEIGHTS_BYTES,
            "decoder_role": "report_only_neural_representation_ceiling",
        },
        "ready_for_exact_object_field": False,
        "deterministic_codec_distillation_authorized": decision
        == "LEARNED_CODEC_REPRESENTATION_SUPPORTED",
        "neural_training_authorized": False,
        "signals": {
            "target": target,
            "nearest_fit": baseline,
            "resampling_control": resampling_control,
            "learned_codec": candidate,
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
    root: Path,
    manifest_argument: Path,
    contacts_argument: Path,
    weights_argument: Path,
    output_argument: Path,
) -> Path:
    manifest_path = common.external_file(root, manifest_argument, "R3A V2 manifest")
    manifest_bytes, manifest = common.load_json(manifest_path, "R3A V2 manifest")
    common.validate_manifest(manifest)
    common.validate_implementation(manifest, Path(__file__).resolve().parent)
    weights = common.external_file(root, weights_argument, "DAC weights")
    common.validate_dac_weights(weights)
    from physical_sound_contact_field_r3a_v2_preflight import environment_identity

    if environment_identity() != manifest["dac_dependency"]["environment"]:
        raise common.R3AV2Error("DAC runtime environment changed after preflight")
    contacts_directory = contacts_argument.resolve(strict=True)
    if contacts_directory.is_relative_to(root) or not contacts_directory.is_dir():
        raise common.R3AV2Error("contacts must be an external extraction directory")
    report_bytes, extraction_report = common.load_json(
        contacts_directory / "report.json", "R3A V2 extraction report"
    )
    metadata_bytes, metadata = common.load_json(
        contacts_directory / "contacts-metadata.json", "R3A V2 contacts metadata"
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
        raise common.R3AV2Error("R3A V2 contact extraction lineage changed")
    with (contacts_directory / "contacts.npy").open("rb") as handle:
        waveforms = np.load(handle, allow_pickle=False)
    positions = np.asarray(
        [contact["position_metres"] for contact in metadata["contacts"]],
        dtype=np.float64,
    )
    result = evaluate(np.asarray(waveforms, dtype=np.float64), positions, weights)
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
            "evaluation_band_hz": [EVALUATION_MIN_HZ, EVALUATION_MAX_HZ],
            "primary_endpoints": list(PRIMARY_ENDPOINTS),
            "absolute_thresholds": ABSOLUTE_THRESHOLDS,
            "resampling_control_thresholds": RESAMPLING_CONTROL_THRESHOLDS,
            "result": result,
            "audition_artifact_policy": "independently_peak_normalized_pcm16_diagnostic_only",
            "audition_artifact_sha256": artifact_hashes,
            "authored_clip_fallback_required": True,
            "neural_training_authorized": False,
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
    output = run(
        root,
        arguments.manifest,
        arguments.contacts,
        arguments.dac_weights,
        arguments.output,
    )
    _, report = common.load_json(output / "report.json", "R3A V2 oracle report")
    print(f"R3A V2 learned-codec oracle: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
