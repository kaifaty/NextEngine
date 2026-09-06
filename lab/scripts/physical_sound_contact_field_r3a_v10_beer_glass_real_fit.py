#!/usr/bin/env python3
"""Run the frozen V10 V9-representation fit on ObjectFolder Beer Glass."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import shutil
import struct
import sys
import tarfile
from io import BytesIO
from pathlib import Path
from typing import Any

import numpy as np
import scipy
from scipy import fft, signal

import physical_sound_contact_field_r3a_v10_real_source_inventory as source
import physical_sound_contact_field_r3a_v4_common as endpoint

STUDY_ID = "physical-sound-contact-field-r3a-v10-beer-glass-real-fit"
REVISION = "explicit-modes-dct-noise-band-residual-fit-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-real-fit.manifest.v1"
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-real-fit.model.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-real-fit.report.v1"

INVENTORY_MANIFEST_SHA256 = (
    "8a30cef02684ddbc338d03a36e7cfab3dca2d5273e6d1d1fa885410e56048728"
)
OBJECT_ID = "60"
FIT_CONTACT_IDS = source.OBJECTS[OBJECT_ID]["roles"]["fit"]
DEVELOPMENT_CONTACT_IDS = source.OBJECTS[OBJECT_ID]["roles"]["development"]
QUERY_CONTACT_IDS = source.OBJECTS[OBJECT_ID]["roles"]["exact_object_query"]
HOLDOUT_CONTACT_IDS = source.OBJECTS["22"]["roles"]["representation_holdout"]

SAMPLE_RATE_HZ = 48_000
SOURCE_SAMPLES = 288_000
ANALYSIS_SAMPLES = 144_000
BASELINE_SAMPLES = 12_000
ALIGNMENT_SAMPLE = 512
MODE_COUNT = 64
MINIMUM_MODE_SEPARATION_HZ = 8.0
NOISE_BAND_COUNT = 96
NOISE_LOOP_SAMPLES = 16_384
NOISE_SEED = 20_260_831
RESIDUAL_FRAME_SAMPLES = 2_048
RESIDUAL_FRAME_HOP = 128
RESIDUAL_DCT_COEFFICIENTS = 8
RESIDUAL_LOG_FLOOR = 1.0e-12
TRANSIENT_SAMPLES = 2_048
MAXIMUM_SHARED_BYTES = 4 * 1024 * 1024
MAXIMUM_CONTACT_BYTES = 64 * 1024
EXPECTED_CONTACT_RECORD_BYTES = 5_904
EXPECTED_DECODED_SOURCE_SAMPLES = len(FIT_CONTACT_IDS) * SOURCE_SAMPLES
EXPECTED_RETAINED_SAMPLES = len(FIT_CONTACT_IDS) * ANALYSIS_SAMPLES

REQUIRED_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "openblas_num_threads": "1",
    "omp_num_threads": "1",
    "mkl_num_threads": "1",
}


class FitError(RuntimeError):
    """The frozen V10 Beer Glass fit boundary was violated."""


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def array_sha256(value: np.ndarray, dtype: str = "<f8") -> str:
    return sha256_bytes(np.ascontiguousarray(value, dtype=dtype).tobytes())


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise FitError(f"{label} must be an external regular file")
    return resolved


def environment_identity(require_exact: bool = True) -> dict[str, Any]:
    observed = {
        "python": platform.python_version(),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "byteorder": sys.byteorder,
        "openblas_num_threads": os.environ.get("OPENBLAS_NUM_THREADS", ""),
        "omp_num_threads": os.environ.get("OMP_NUM_THREADS", ""),
        "mkl_num_threads": os.environ.get("MKL_NUM_THREADS", ""),
    }
    if require_exact:
        for key, expected in REQUIRED_ENVIRONMENT.items():
            if observed[key] != expected:
                raise FitError(
                    f"V10 real-fit environment changed for {key}: "
                    f"{observed[key]} != {expected}"
                )
    return observed


def implementation_hashes(directory: Path) -> dict[str, str]:
    files = {
        "fit": directory / Path(__file__).name,
        "inventory": directory
        / "physical_sound_contact_field_r3a_v10_real_source_inventory.py",
        "endpoints": directory / "physical_sound_contact_field_r3a_v4_common.py",
    }
    return {key: sha256_file(path) for key, path in files.items()}


def validate_inventory_manifest(root: Path, path: Path) -> dict[str, Any]:
    resolved = require_external_file(root, path, "V10 inventory manifest")
    payload = resolved.read_bytes()
    if sha256_bytes(payload) != INVENTORY_MANIFEST_SHA256:
        raise FitError("V10 inventory manifest hash changed")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FitError("cannot parse V10 inventory manifest") from error
    if canonical_json(value) != payload:
        raise FitError("V10 inventory manifest is not canonical")
    if value.get("schema") != source.MANIFEST_SCHEMA:
        raise FitError("V10 inventory manifest schema changed")
    if value.get("next_authorized_role") != "fit":
        raise FitError("V10 inventory does not authorize fit")
    counter_names = (
        "waveform_sample_values_decoded",
        "fit_waveform_sample_values_decoded",
        "development_waveform_sample_values_decoded",
        "exact_object_query_waveform_sample_values_decoded",
        "representation_holdout_waveform_sample_values_decoded",
        "method_holdout_waveform_sample_values_decoded",
        "admission_shadow_waveform_sample_values_decoded",
    )
    if any(value.get(name) != 0 for name in counter_names):
        raise FitError("V10 inventory already decoded waveform samples")

    observed = {
        (str(record["object_id"]), record["role"]): tuple(
            sorted(
                int(candidate["contact_id"])
                for candidate in value["contacts"]
                if str(candidate["object_id"]) == str(record["object_id"])
                and candidate["role"] == record["role"]
            )
        )
        for record in value["contacts"]
    }
    expected = {
        (object_id, role): tuple(contacts)
        for object_id, description in source.OBJECTS.items()
        for role, contacts in description["roles"].items()
    }
    if observed != expected:
        raise FitError("V10 inventory roles changed")
    return value


def validate_sources(
    root: Path,
    audio: Path,
    contacts: Path,
    points: Path,
    split: Path,
    scale: Path,
) -> dict[str, Path]:
    arguments = {
        "audio": audio,
        "contacts": contacts,
        "point_cloud": points,
        "split": split,
        "scale": scale,
    }
    result = {}
    for key, argument in arguments.items():
        try:
            result[key] = source.require_source(root, argument, source.SOURCES[key])
        except source.InventoryError as error:
            raise FitError(str(error)) from error
    return result


def fit_commitments(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    commitments = {}
    for record in manifest["contacts"]:
        if str(record["object_id"]) != OBJECT_ID or record["role"] != "fit":
            continue
        contact_id = int(record["contact_id"])
        audio = record.get("audio")
        expected_path = f"audio/{OBJECT_ID}/{contact_id}.wav"
        if (
            audio is None
            or audio.get("path") != expected_path
            or audio.get("descriptor") != source.EXPECTED_WAV_HEADER
            or not record.get("audio_payload_committed")
        ):
            raise FitError(f"fit audio commitment changed: {contact_id}")
        commitments[expected_path] = audio
    expected_paths = {
        f"audio/{OBJECT_ID}/{contact_id}.wav" for contact_id in FIT_CONTACT_IDS
    }
    if set(commitments) != expected_paths:
        raise FitError("fit audio commitment set changed")
    return commitments


def extract_fit_audio(
    archive_path: Path, commitments: dict[str, dict[str, Any]]
) -> dict[int, bytes]:
    selected = set(commitments)
    found: dict[int, bytes] = {}
    try:
        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive:
                if member.name not in selected:
                    continue
                if not member.isfile():
                    raise FitError(f"fit audio member is not a file: {member.name}")
                contact_id = int(Path(member.name).stem)
                if contact_id in found:
                    raise FitError(f"duplicate fit audio member: {member.name}")
                stream = archive.extractfile(member)
                if stream is None:
                    raise FitError(f"cannot read fit audio member: {member.name}")
                payload = stream.read()
                commitment = commitments[member.name]
                if (
                    len(payload) != commitment["bytes"]
                    or sha256_bytes(payload) != commitment["sha256"]
                ):
                    raise FitError(f"fit audio payload changed: {member.name}")
                found[contact_id] = payload
                if len(found) == len(selected):
                    break
    except (tarfile.TarError, EOFError, OSError) as error:
        raise FitError("cannot extract V10 fit audio") from error
    if tuple(sorted(found)) != tuple(FIT_CONTACT_IDS):
        raise FitError("fit audio members are missing")
    return found


def decode_pcm16(payload: bytes) -> np.ndarray:
    try:
        source.parse_pcm_header(payload[:64], len(payload))
    except source.InventoryError as error:
        raise FitError(str(error)) from error
    samples = np.frombuffer(payload, dtype="<i2", offset=44)
    if samples.shape != (SOURCE_SAMPLES,):
        raise FitError("fit PCM sample count changed")
    return samples.astype(np.float64) / 32768.0


def onset_sample(value: np.ndarray) -> tuple[int, float, float, float]:
    absolute = np.abs(value)
    baseline = absolute[:BASELINE_SAMPLES]
    median = float(np.median(baseline))
    mad = float(np.median(np.abs(baseline - median)))
    peak_threshold = 0.06 * float(np.max(absolute))
    noise_threshold = median + 12.0 * mad
    threshold = max(peak_threshold, noise_threshold)
    indices = np.flatnonzero(absolute > threshold)
    if indices.size == 0:
        raise FitError("audio onset threshold was never crossed")
    return int(indices[0]), threshold, peak_threshold, noise_threshold


def shift_and_crop(value: np.ndarray, shift: int) -> np.ndarray:
    shifted = np.zeros_like(value)
    if shift >= 0:
        if shift < value.size:
            shifted[shift:] = value[: value.size - shift]
    elif -shift < value.size:
        shifted[:shift] = value[-shift:]
    return np.ascontiguousarray(shifted[:ANALYSIS_SAMPLES], dtype=np.float64)


def preprocess_contact(value: np.ndarray) -> tuple[np.ndarray, dict[str, Any]]:
    if value.shape != (SOURCE_SAMPLES,) or not np.isfinite(value).all():
        raise FitError("source contact shape or finiteness changed")
    baseline_median = float(np.median(value[:BASELINE_SAMPLES]))
    centered = value - baseline_median
    onset, threshold, peak_threshold, noise_threshold = onset_sample(centered)
    shift = ALIGNMENT_SAMPLE - onset
    aligned = shift_and_crop(centered, shift)
    if aligned.shape != (ANALYSIS_SAMPLES,) or not np.isfinite(aligned).all():
        raise FitError("aligned contact is invalid")
    return aligned, {
        "source_onset_sample": onset,
        "alignment_shift_samples": shift,
        "baseline_median": baseline_median,
        "onset_threshold": threshold,
        "peak_threshold": peak_threshold,
        "noise_threshold": noise_threshold,
    }


def load_fit_targets(
    payloads: dict[int, bytes],
) -> tuple[np.ndarray | None, list[dict[str, Any]], float | None, list[int]]:
    rows = []
    descriptors = []
    failed_contacts = []
    for contact_id in FIT_CONTACT_IDS:
        decoded = decode_pcm16(payloads[contact_id])
        try:
            aligned, descriptor = preprocess_contact(decoded)
        except FitError as error:
            baseline_median = float(np.median(decoded[:BASELINE_SAMPLES]))
            centered = decoded - baseline_median
            absolute = np.abs(centered)
            baseline = absolute[:BASELINE_SAMPLES]
            absolute_median = float(np.median(baseline))
            absolute_mad = float(np.median(np.abs(baseline - absolute_median)))
            peak_threshold = 0.06 * float(np.max(absolute))
            noise_threshold = absolute_median + 12.0 * absolute_mad
            descriptor = {
                "contact_id": contact_id,
                "status": "OnsetThresholdNotCrossed",
                "error": str(error),
                "baseline_median": baseline_median,
                "absolute_peak": float(np.max(absolute)),
                "peak_threshold": peak_threshold,
                "noise_threshold": noise_threshold,
                "onset_threshold": max(peak_threshold, noise_threshold),
            }
            descriptors.append(descriptor)
            failed_contacts.append(contact_id)
            continue
        descriptor["contact_id"] = contact_id
        descriptor["status"] = "Aligned"
        rows.append(aligned)
        descriptors.append(descriptor)
    if failed_contacts:
        return None, descriptors, None, failed_contacts
    values = np.stack(rows)
    peak = float(np.max(np.abs(values)))
    if not math.isfinite(peak) or peak <= 0.0:
        raise FitError("fit-only peak is invalid")
    scale = 0.92 / peak
    return np.ascontiguousarray(values * scale), descriptors, scale, []


def estimate_poles(targets: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    if targets.shape != (len(FIT_CONTACT_IDS), ANALYSIS_SAMPLES):
        raise FitError("pole estimator input shape changed")
    window = np.hanning(ANALYSIS_SAMPLES)
    spectra = np.abs(np.fft.rfft(targets * window[None, :], axis=1))
    summary = np.sqrt(np.mean(np.square(spectra), axis=0))
    frequencies = np.fft.rfftfreq(ANALYSIS_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    eligible = (frequencies >= endpoint.EVALUATION_MIN_HZ) & (
        frequencies <= endpoint.EVALUATION_MAX_HZ
    )
    separation_bins = max(
        1,
        math.ceil(MINIMUM_MODE_SEPARATION_HZ / (frequencies[1] - frequencies[0])),
    )
    peaks, _ = signal.find_peaks(summary, distance=separation_bins)
    peaks = peaks[eligible[peaks]]
    strength_order = peaks[np.argsort(-summary[peaks], kind="stable")]
    if strength_order.size < MODE_COUNT:
        raise FitError("Beer Glass exposes too few separated modal peaks")
    selected = strength_order[:MODE_COUNT]

    tail = targets[:, ALIGNMENT_SAMPLE:]
    _, times, stft = signal.stft(
        tail,
        fs=SAMPLE_RATE_HZ,
        window="hann",
        nperseg=endpoint.MODAL_STFT_SAMPLES,
        noverlap=endpoint.MODAL_STFT_SAMPLES - endpoint.MODAL_STFT_HOP,
        boundary=None,
        padded=False,
        axis=-1,
    )
    stft_frequencies = np.fft.rfftfreq(
        endpoint.MODAL_STFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ
    )
    magnitude = np.sqrt(np.mean(np.square(np.abs(stft)), axis=0))
    rows = []
    for rank, peak in enumerate(selected):
        frequency = float(frequencies[peak])
        bin_index = int(np.argmin(np.abs(stft_frequencies - frequency)))
        envelope = magnitude[bin_index]
        maximum = max(float(np.max(envelope)), 1.0e-30)
        start = int(np.argmax(envelope))
        usable = np.arange(envelope.size) >= start
        usable &= times <= min(float(times[-1]), 2.5)
        usable &= envelope >= maximum * 1.0e-3
        if np.count_nonzero(usable) >= 6:
            slope = np.polyfit(
                times[usable], np.log(np.maximum(envelope[usable], 1.0e-30)), 1
            )[0]
            damping = float(np.clip(-slope, 0.25, 200.0))
        else:
            damping = 20.0
        rows.append((frequency, damping, float(rank)))
    poles = np.asarray(rows, dtype=np.float64)
    order = np.argsort(poles[:, 0], kind="stable")
    poles = np.ascontiguousarray(poles[order])
    if not np.isfinite(poles).all():
        raise FitError("pole estimator produced non-finite values")
    return poles, summary[selected]


def mode_basis(frequency: float, damping: float) -> tuple[np.ndarray, np.ndarray]:
    time = np.arange(ANALYSIS_SAMPLES - ALIGNMENT_SAMPLE, dtype=np.float64)
    time /= SAMPLE_RATE_HZ
    envelope = np.exp(-damping * time)
    phase = 2.0 * math.pi * frequency * time
    return envelope * np.cos(phase), envelope * np.sin(phase)


def fit_modal_gains(targets: np.ndarray, poles: np.ndarray) -> np.ndarray:
    residuals = targets.copy()
    gains = np.zeros((targets.shape[0], MODE_COUNT, 2), dtype=np.float64)
    rank_order = np.argsort(poles[:, 2], kind="stable")
    for mode_index in rank_order:
        cosine, sine = mode_basis(float(poles[mode_index, 0]), float(poles[mode_index, 1]))
        gram = np.asarray(
            [
                [np.dot(cosine, cosine), np.dot(cosine, sine)],
                [np.dot(cosine, sine), np.dot(sine, sine)],
            ]
        )
        view = residuals[:, ALIGNMENT_SAMPLE:]
        rhs = np.column_stack((view @ cosine, view @ sine))
        solved = np.linalg.solve(gram + np.eye(2) * 1.0e-12, rhs.T).T
        gains[:, mode_index] = solved
        view -= solved[:, 0, None] * cosine + solved[:, 1, None] * sine
    return gains


def synthesize_modal(poles: np.ndarray, gains: np.ndarray) -> np.ndarray:
    result = np.zeros((gains.shape[0], ANALYSIS_SAMPLES), dtype=np.float64)
    for mode_index, pole in enumerate(poles):
        cosine, sine = mode_basis(float(pole[0]), float(pole[1]))
        result[:, ALIGNMENT_SAMPLE:] += (
            gains[:, mode_index, 0, None] * cosine
            + gains[:, mode_index, 1, None] * sine
        )
    return result


def quantize_float16_scaled(value: np.ndarray) -> tuple[np.ndarray, np.float32, np.ndarray]:
    maximum = float(np.max(np.abs(value)))
    scale = np.float32(max(maximum, 1.0e-30))
    quantized = np.asarray(value / float(scale), dtype="<f2")
    decoded = quantized.astype(np.float64) * float(scale)
    return quantized, scale, decoded


def quantize_int16(value: np.ndarray) -> tuple[np.ndarray, np.float32, np.ndarray]:
    maximum = float(np.max(np.abs(value)))
    scale = np.float32(max(maximum / 32767.0, 1.0e-30))
    quantized = np.clip(np.rint(value / float(scale)), -32767, 32767).astype("<i2")
    decoded = quantized.astype(np.float64) * float(scale)
    return quantized, scale, decoded


def band_centers_hz() -> np.ndarray:
    return np.geomspace(
        endpoint.EVALUATION_MIN_HZ,
        endpoint.EVALUATION_MAX_HZ,
        NOISE_BAND_COUNT,
        dtype=np.float64,
    ).astype("<f4").astype(np.float64)


def triangular_profiles(centers: np.ndarray, fft_samples: int) -> np.ndarray:
    frequencies = np.fft.rfftfreq(fft_samples, 1.0 / SAMPLE_RATE_HZ)
    log_frequencies = np.log(np.maximum(frequencies, endpoint.EVALUATION_MIN_HZ))
    log_centers = np.log(centers)
    ratio = log_centers[1] - log_centers[0]
    rows = []
    for index, center in enumerate(log_centers):
        lower = log_centers[index - 1] if index else center - ratio
        upper = log_centers[index + 1] if index + 1 < len(centers) else center + ratio
        row = np.zeros_like(frequencies)
        left = (log_frequencies >= lower) & (log_frequencies <= center)
        right = (log_frequencies > center) & (log_frequencies <= upper)
        row[left] = (log_frequencies[left] - lower) / max(center - lower, 1.0e-30)
        row[right] = (upper - log_frequencies[right]) / max(upper - center, 1.0e-30)
        row[(frequencies < endpoint.EVALUATION_MIN_HZ)] = 0.0
        row[(frequencies > endpoint.EVALUATION_MAX_HZ)] = 0.0
        if not np.any(row > 0.0):
            row[int(np.argmin(np.abs(frequencies - math.exp(center))))] = 1.0
        rows.append(row)
    return np.ascontiguousarray(np.stack(rows))


def generate_carriers(centers: np.ndarray) -> np.ndarray:
    profiles = triangular_profiles(centers, NOISE_LOOP_SAMPLES)
    generator = np.random.Generator(np.random.PCG64(NOISE_SEED))
    rows = []
    for profile in profiles:
        phases = generator.uniform(-math.pi, math.pi, profile.size)
        spectrum = profile * np.exp(1j * phases)
        spectrum[0] = 0.0
        spectrum[-1] = 0.0
        value = np.fft.irfft(spectrum, n=NOISE_LOOP_SAMPLES)
        rms = math.sqrt(float(np.mean(np.square(value))))
        if not math.isfinite(rms) or rms <= 0.0:
            raise FitError("deterministic carrier is invalid")
        rows.append(value / rms)
    result = np.ascontiguousarray(np.stack(rows))
    if result.shape != (NOISE_BAND_COUNT, NOISE_LOOP_SAMPLES):
        raise FitError("deterministic carrier shape changed")
    return result


def residual_band_envelopes(
    residual: np.ndarray, centers: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    _, times, stft = signal.stft(
        residual,
        fs=SAMPLE_RATE_HZ,
        window="hann",
        nperseg=RESIDUAL_FRAME_SAMPLES,
        noverlap=RESIDUAL_FRAME_SAMPLES - RESIDUAL_FRAME_HOP,
        boundary=None,
        padded=False,
    )
    profiles = triangular_profiles(centers, RESIDUAL_FRAME_SAMPLES)
    weights = np.square(profiles)
    denominator = np.maximum(np.sum(weights, axis=1), 1.0e-30)
    energy = np.square(np.abs(stft))
    envelopes = np.sqrt((weights @ energy) / denominator[:, None])
    if not np.isfinite(envelopes).all() or np.any(envelopes < 0.0):
        raise FitError("residual band envelope is invalid")
    return envelopes, times * SAMPLE_RATE_HZ


def carrier_calibration(carriers: np.ndarray, centers: np.ndarray) -> np.ndarray:
    profiles = triangular_profiles(centers, RESIDUAL_FRAME_SAMPLES)
    weights = np.square(profiles)
    denominator = np.maximum(np.sum(weights, axis=1), 1.0e-30)
    result = []
    for band_index, carrier in enumerate(carriers):
        _, _, stft = signal.stft(
            carrier,
            fs=SAMPLE_RATE_HZ,
            window="hann",
            nperseg=RESIDUAL_FRAME_SAMPLES,
            noverlap=RESIDUAL_FRAME_SAMPLES - RESIDUAL_FRAME_HOP,
            boundary=None,
            padded=False,
        )
        energy = np.square(np.abs(stft))
        envelope = np.sqrt(
            (weights[band_index] @ energy) / denominator[band_index]
        )
        calibration = float(np.median(envelope))
        if not math.isfinite(calibration) or calibration <= 0.0:
            raise FitError("carrier calibration is invalid")
        result.append(calibration)
    return np.asarray(result, dtype=np.float64)


def encode_residual_envelope(envelopes: np.ndarray) -> tuple[np.ndarray, np.float32, np.ndarray]:
    log_envelopes = np.log(np.maximum(envelopes, RESIDUAL_LOG_FLOOR))
    coefficients = fft.dct(log_envelopes, type=2, norm="ortho", axis=1)[
        :, :RESIDUAL_DCT_COEFFICIENTS
    ]
    quantized, scale, decoded_coefficients = quantize_float16_scaled(coefficients)
    padded = np.zeros_like(log_envelopes)
    padded[:, :RESIDUAL_DCT_COEFFICIENTS] = decoded_coefficients
    decoded = np.exp(fft.idct(padded, type=2, norm="ortho", axis=1))
    if not np.isfinite(decoded).all() or np.any(decoded < 0.0):
        raise FitError("decoded residual envelope is invalid")
    return quantized, scale, decoded


def stationary_envelope(
    quantized: np.ndarray, scale: np.float32, frame_count: int
) -> np.ndarray:
    coefficients = quantized.astype(np.float64) * float(scale)
    padded = np.zeros((NOISE_BAND_COUNT, frame_count), dtype=np.float64)
    padded[:, 0] = coefficients[:, 0]
    return np.exp(fft.idct(padded, type=2, norm="ortho", axis=1))


def synthesize_residual(
    envelopes: np.ndarray,
    frame_positions: np.ndarray,
    carriers: np.ndarray,
    calibration: np.ndarray,
) -> np.ndarray:
    sample_positions = np.arange(ANALYSIS_SAMPLES, dtype=np.float64)
    repeats = math.ceil(ANALYSIS_SAMPLES / NOISE_LOOP_SAMPLES)
    result = np.zeros(ANALYSIS_SAMPLES, dtype=np.float64)
    for band_index in range(NOISE_BAND_COUNT):
        amplitude = np.interp(
            sample_positions,
            frame_positions,
            envelopes[band_index],
            left=envelopes[band_index, 0],
            right=envelopes[band_index, -1],
        )
        carrier = np.tile(carriers[band_index], repeats)[:ANALYSIS_SAMPLES]
        result += carrier * (amplitude / calibration[band_index])
    if not np.isfinite(result).all():
        raise FitError("residual synthesis produced non-finite values")
    return result


def encode_contact(
    target: np.ndarray,
    poles: np.ndarray,
    raw_gains: np.ndarray,
    centers: np.ndarray,
    carriers: np.ndarray,
    calibration: np.ndarray,
) -> tuple[np.ndarray, np.ndarray, bytes, dict[str, Any]]:
    gain_q, gain_scale, gain_decoded = quantize_float16_scaled(raw_gains)
    modal = synthesize_modal(poles, gain_decoded[None, :, :])[0]
    residual = target - modal
    envelopes, positions = residual_band_envelopes(residual, centers)
    residual_q, residual_scale, residual_decoded = encode_residual_envelope(envelopes)
    residual_waveform = synthesize_residual(
        residual_decoded, positions, carriers, calibration
    )
    candidate = modal + residual_waveform

    transient = target[:TRANSIENT_SAMPLES] - candidate[:TRANSIENT_SAMPLES]
    transient_q, transient_scale, transient_decoded = quantize_int16(transient)
    candidate[:TRANSIENT_SAMPLES] += transient_decoded
    denominator = max(float(np.dot(candidate, candidate)), 1.0e-30)
    output_scale = np.float32(np.dot(target, candidate) / denominator)
    candidate *= float(output_scale)

    stationary = synthesize_residual(
        stationary_envelope(residual_q, residual_scale, envelopes.shape[1]),
        positions,
        carriers,
        calibration,
    )
    stationary_candidate = modal + stationary

    record = b"".join(
        [
            struct.pack("<f", float(gain_scale)),
            gain_q.tobytes(order="C"),
            struct.pack("<f", float(residual_scale)),
            residual_q.tobytes(order="C"),
            struct.pack("<f", float(transient_scale)),
            transient_q.tobytes(order="C"),
            struct.pack("<f", float(output_scale)),
        ]
    )
    if len(record) != EXPECTED_CONTACT_RECORD_BYTES:
        raise FitError("contact record byte count changed")
    temporal_variation = bool(np.any(residual_q[:, 1:] != 0))
    descriptor = {
        "encoded_bytes": len(record),
        "record_sha256": sha256_bytes(record),
        "prediction_sha256": array_sha256(candidate),
        "modal_prediction_sha256": array_sha256(modal),
        "stationary_prediction_sha256": array_sha256(stationary_candidate),
        "modal_gain_scale": float(gain_scale),
        "residual_coefficient_scale": float(residual_scale),
        "transient_scale": float(transient_scale),
        "output_scale": float(output_scale),
        "residual_temporal_variation": temporal_variation,
        "residual_frame_count": envelopes.shape[1],
    }
    return candidate, np.stack((modal, stationary_candidate)), record, descriptor


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise FitError("V10 real-fit output must stay outside the repository")
    if resolved.exists():
        raise FitError("V10 real-fit output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def metric_summary(values: list[dict[str, float]]) -> dict[str, dict[str, float]]:
    return {
        name: {
            "minimum": min(item[name] for item in values),
            "median": float(np.median([item[name] for item in values])),
            "maximum": max(item[name] for item in values),
        }
        for name in endpoint.PRIMARY_ENDPOINTS
    }


def build_manifest(
    environment: dict[str, Any], implementation: dict[str, str]
) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenFitOnly",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "inventory_manifest_sha256": INVENTORY_MANIFEST_SHA256,
        "object_id": OBJECT_ID,
        "fit_contacts": list(FIT_CONTACT_IDS),
        "protected_roles": {
            "development": list(DEVELOPMENT_CONTACT_IDS),
            "exact_object_query": list(QUERY_CONTACT_IDS),
            "representation_holdout": list(HOLDOUT_CONTACT_IDS),
        },
        "source_files": source.SOURCES,
        "preprocessing": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "source_samples": SOURCE_SAMPLES,
            "analysis_samples": ANALYSIS_SAMPLES,
            "baseline_samples": BASELINE_SAMPLES,
            "alignment_sample": ALIGNMENT_SAMPLE,
            "centering": "per_contact_first_12000_sample_median",
            "onset": "first_abs_gt_max_0.06_peak_and_baseline_abs_median_plus_12_mad",
            "shift": "zero_pad_without_wrap",
            "normalization": "one_fit_only_scale_to_global_peak_0.92",
            "resampling": "forbidden",
        },
        "representation": {
            "mode_count": MODE_COUNT,
            "minimum_mode_separation_hz": MINIMUM_MODE_SEPARATION_HZ,
            "frequency_band_hz": [
                endpoint.EVALUATION_MIN_HZ,
                endpoint.EVALUATION_MAX_HZ,
            ],
            "direct_onset_excitation": True,
            "noise_band_count": NOISE_BAND_COUNT,
            "noise_loop_samples": NOISE_LOOP_SAMPLES,
            "noise_seed": NOISE_SEED,
            "residual_frame_samples": RESIDUAL_FRAME_SAMPLES,
            "residual_frame_hop": RESIDUAL_FRAME_HOP,
            "residual_window": "hann",
            "residual_boundary": None,
            "residual_padded": False,
            "residual_band_measure": "weighted_stft_rms_with_squared_triangular_weights",
            "residual_carrier_calibration": "median_matching_band_stft_rms",
            "residual_log_floor": RESIDUAL_LOG_FLOOR,
            "residual_dct_type": "orthonormal_dct_ii",
            "residual_dct_coefficients": RESIDUAL_DCT_COEFFICIENTS,
            "transient_samples": TRANSIENT_SAMPLES,
        },
        "primary_endpoints": list(endpoint.PRIMARY_ENDPOINTS),
        "absolute_thresholds": endpoint.ABSOLUTE_THRESHOLDS,
        "maximum_shared_bytes": MAXIMUM_SHARED_BYTES,
        "maximum_contact_bytes": MAXIMUM_CONTACT_BYTES,
        "expected_contact_record_bytes": EXPECTED_CONTACT_RECORD_BYTES,
        "environment": environment,
        "implementation_sha256": implementation,
        "waveform_sample_values_decoded": 0,
        "fit_waveform_sample_values_decoded": 0,
        "development_waveform_sample_values_decoded": 0,
        "exact_object_query_waveform_sample_values_decoded": 0,
        "representation_holdout_waveform_sample_values_decoded": 0,
        "method_holdout_waveform_sample_values_decoded": 0,
        "admission_shadow_waveform_sample_values_decoded": 0,
        "development_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def write_onset_rejection(
    root: Path,
    output_path: Path,
    manifest: dict[str, Any],
    preprocessing: list[dict[str, Any]],
    failed_contacts: list[int],
) -> Path:
    decoded_samples = len(FIT_CONTACT_IDS) * SOURCE_SAMPLES
    protected_counters = {
        "development_waveform_sample_values_decoded": 0,
        "exact_object_query_waveform_sample_values_decoded": 0,
        "representation_holdout_waveform_sample_values_decoded": 0,
        "method_holdout_waveform_sample_values_decoded": 0,
        "admission_shadow_waveform_sample_values_decoded": 0,
    }
    manifest["waveform_sample_values_decoded"] = decoded_samples
    manifest["fit_waveform_sample_values_decoded"] = decoded_samples
    manifest.update(protected_counters)
    manifest_bytes = canonical_json(manifest)
    decision = "REJECT_V9_REAL_REPRESENTATION"
    model = {
        "schema": MODEL_SCHEMA,
        "status": "RejectedBeforeRepresentationFit",
        "decision": decision,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "failure": "FROZEN_ONSET_GATE_NOT_SATISFIED",
        "failed_contact_ids": failed_contacts,
        "preprocessing": preprocessing,
        "shared_decoder_bytes": 0,
        "maximum_contact_record_bytes": 0,
        "contact_records": {},
        **protected_counters,
    }
    model_bytes = canonical_json(model)
    report = {
        "schema": REPORT_SCHEMA,
        "status": "ValidatedRejection",
        "decision": decision,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "model_sha256": sha256_bytes(model_bytes),
        "failure": "FROZEN_ONSET_GATE_NOT_SATISFIED",
        "failed_contact_ids": failed_contacts,
        "failed_contact_count": len(failed_contacts),
        "hard_gates": {
            "decoded_source_sample_count": decoded_samples
            == EXPECTED_DECODED_SOURCE_SAMPLES,
            "all_fit_contacts_have_frozen_onset": False,
            "protected_decode_zero": True,
            "representation_fit_not_started_after_preprocessing_failure": True,
        },
        "source_sample_values_decoded": decoded_samples,
        "fit_analysis_sample_values_used": 0,
        **protected_counters,
        "candidate_metrics_computed": False,
        "method_holdout_accessed": False,
        "admission_shadow_accessed": False,
        "real_quality_credit": False,
        "development_authorized": False,
        "exact_object_query_authorized": False,
        "representation_holdout_authorized": False,
        "r3b_authorized": False,
        "runtime_or_public_contract_changed": False,
        "authored_clip_fallback_required": True,
        "stop_rule": (
            "do_not_change_onset_threshold_or_fit_subset_on_opened_contacts; "
            "a materially different preregistered representation and source revision "
            "is required"
        ),
    }
    output, staging = prepare_output(root, output_path)
    try:
        (staging / "manifest.json").write_bytes(manifest_bytes)
        (staging / "model.json").write_bytes(model_bytes)
        (staging / "report.json").write_bytes(canonical_json(report))
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def run(
    root: Path,
    inventory_path: Path,
    audio_path: Path,
    contacts_path: Path,
    points_path: Path,
    split_path: Path,
    scale_path: Path,
    output_path: Path,
) -> Path:
    inventory = validate_inventory_manifest(root, inventory_path)
    paths = validate_sources(
        root, audio_path, contacts_path, points_path, split_path, scale_path
    )
    environment = environment_identity()
    implementation = implementation_hashes(Path(__file__).resolve().parent)
    manifest = build_manifest(environment, implementation)
    commitments = fit_commitments(inventory)
    payloads = extract_fit_audio(paths["audio"], commitments)
    targets, preprocessing, fit_scale, onset_failures = load_fit_targets(payloads)
    if onset_failures:
        return write_onset_rejection(
            root, output_path, manifest, preprocessing, onset_failures
        )
    if targets is None or fit_scale is None:
        raise FitError("fit preprocessing result is internally inconsistent")
    if targets.size != EXPECTED_RETAINED_SAMPLES:
        raise FitError("retained fit sample accounting changed")

    raw_poles, strengths = estimate_poles(targets)
    poles = raw_poles.copy()
    poles[:, :2] = poles[:, :2].astype("<f4").astype(np.float64)
    gains = fit_modal_gains(targets, poles)
    centers = band_centers_hz()
    carriers = generate_carriers(centers)
    repeated_carriers = generate_carriers(centers)
    if not np.array_equal(carriers, repeated_carriers):
        raise FitError("deterministic carrier regeneration changed")
    calibration = carrier_calibration(carriers, centers)

    output, staging = prepare_output(root, output_path)
    try:
        poles_payload = np.ascontiguousarray(poles[:, :2], dtype="<f4")
        centers_payload = np.ascontiguousarray(centers, dtype="<f4")
        poles_file = staging / "shared-poles.npy"
        centers_file = staging / "shared-band-centers.npy"
        np.save(poles_file, poles_payload, allow_pickle=False)
        np.save(centers_file, centers_payload, allow_pickle=False)
        shared_bytes = poles_payload.nbytes + centers_payload.nbytes

        contacts = []
        candidate_metrics = []
        modal_metrics = []
        stationary_metrics = []
        all_fit_contacts_pass = True
        maximum_record_bytes = 0
        for row, contact_id in enumerate(FIT_CONTACT_IDS):
            candidate, ablations, record, descriptor = encode_contact(
                targets[row], poles, gains[row], centers, carriers, calibration
            )
            repeated, repeated_ablations, repeated_record, repeated_descriptor = (
                encode_contact(
                    targets[row], poles, gains[row], centers, carriers, calibration
                )
            )
            if (
                record != repeated_record
                or not np.array_equal(candidate, repeated)
                or not np.array_equal(ablations, repeated_ablations)
                or descriptor != repeated_descriptor
            ):
                raise FitError(f"contact {contact_id} reconstruction is not exact")
            record_path = staging / f"contact-{contact_id}.bin"
            record_path.write_bytes(record)

            metrics = endpoint.metrics(targets[row], candidate)
            modal_only = endpoint.metrics(targets[row], ablations[0])
            stationary_only = endpoint.metrics(targets[row], ablations[1])
            within = {
                name: metrics[name] <= endpoint.ABSOLUTE_THRESHOLDS[name]
                for name in endpoint.PRIMARY_ENDPOINTS
            }
            finite = (
                np.isfinite(candidate).all()
                and all(math.isfinite(value) for value in metrics.values())
                and all(math.isfinite(value) for value in modal_only.values())
                and all(math.isfinite(value) for value in stationary_only.values())
            )
            contact_pass = (
                bool(finite)
                and all(within.values())
                and descriptor["residual_temporal_variation"]
                and descriptor["encoded_bytes"] <= MAXIMUM_CONTACT_BYTES
            )
            all_fit_contacts_pass &= contact_pass
            maximum_record_bytes = max(maximum_record_bytes, len(record))
            candidate_metrics.append(metrics)
            modal_metrics.append(modal_only)
            stationary_metrics.append(stationary_only)
            contacts.append(
                {
                    "contact_id": contact_id,
                    "coordinate_m": next(
                        item["coordinate"]["descriptor"]["coordinate_m"]
                        for item in inventory["contacts"]
                        if str(item["object_id"]) == OBJECT_ID
                        and int(item["contact_id"]) == contact_id
                    ),
                    "source_audio_sha256": commitments[
                        f"audio/{OBJECT_ID}/{contact_id}.wav"
                    ]["sha256"],
                    "metrics": metrics,
                    "within_absolute_threshold": within,
                    "finite": bool(finite),
                    "fit_gate_passed": bool(contact_pass),
                    "ablations": {
                        "modal_only": modal_only,
                        "stationary_first_temporal_coefficient": stationary_only,
                    },
                    **descriptor,
                }
            )

        decoded_samples = len(payloads) * SOURCE_SAMPLES
        protected_counters = {
            "development_waveform_sample_values_decoded": 0,
            "exact_object_query_waveform_sample_values_decoded": 0,
            "representation_holdout_waveform_sample_values_decoded": 0,
            "method_holdout_waveform_sample_values_decoded": 0,
            "admission_shadow_waveform_sample_values_decoded": 0,
        }
        hard_gates = {
            "decoded_source_sample_count": decoded_samples
            == EXPECTED_DECODED_SOURCE_SAMPLES,
            "retained_fit_sample_count": targets.size == EXPECTED_RETAINED_SAMPLES,
            "protected_decode_zero": all(value == 0 for value in protected_counters.values()),
            "mode_count": poles.shape == (MODE_COUNT, 3),
            "modes_inside_evaluation_band": bool(
                np.all(poles[:, 0] >= endpoint.EVALUATION_MIN_HZ)
                and np.all(poles[:, 0] <= endpoint.EVALUATION_MAX_HZ)
            ),
            "band_count": centers.shape == (NOISE_BAND_COUNT,),
            "bands_inside_evaluation_band": bool(
                np.all(centers >= endpoint.EVALUATION_MIN_HZ)
                and np.all(centers <= endpoint.EVALUATION_MAX_HZ)
            ),
            "shared_decoder_budget": shared_bytes <= MAXIMUM_SHARED_BYTES,
            "contact_record_budget": maximum_record_bytes <= MAXIMUM_CONTACT_BYTES,
            "exact_contact_record_bytes": maximum_record_bytes
            == EXPECTED_CONTACT_RECORD_BYTES,
            "all_fit_contacts_finite": all(item["finite"] for item in contacts),
            "all_fit_contacts_temporally_varying": all(
                item["residual_temporal_variation"] for item in contacts
            ),
            "carrier_regeneration_exact": True,
        }
        passed = all_fit_contacts_pass and all(hard_gates.values())
        decision = (
            "READY_FOR_V9_REAL_DEVELOPMENT"
            if passed
            else "REJECT_V9_REAL_REPRESENTATION"
        )

        manifest["waveform_sample_values_decoded"] = decoded_samples
        manifest["fit_waveform_sample_values_decoded"] = decoded_samples
        manifest.update(protected_counters)
        manifest_bytes = canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        model = {
            "schema": MODEL_SCHEMA,
            "status": "Frozen",
            "decision": decision,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "fit_audio_scale": fit_scale,
            "preprocessing": preprocessing,
            "poles_path": poles_file.name,
            "poles_sha256": sha256_file(poles_file),
            "pole_selection_strength_sha256": array_sha256(strengths),
            "band_centers_path": centers_file.name,
            "band_centers_sha256": sha256_file(centers_file),
            "carrier_bank_sha256": array_sha256(carriers),
            "carrier_calibration_sha256": array_sha256(calibration),
            "shared_decoder_bytes": shared_bytes,
            "contact_records": {
                str(item["contact_id"]): item["record_sha256"] for item in contacts
            },
            "maximum_contact_record_bytes": maximum_record_bytes,
            "development_waveform_sample_values_decoded": 0,
            "exact_object_query_waveform_sample_values_decoded": 0,
            "representation_holdout_waveform_sample_values_decoded": 0,
        }
        model_bytes = canonical_json(model)
        (staging / "model.json").write_bytes(model_bytes)
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "model_sha256": sha256_bytes(model_bytes),
            "all_fit_contacts_passed": bool(all_fit_contacts_pass),
            "hard_gates": hard_gates,
            "fit_contacts": contacts,
            "metric_summary": metric_summary(candidate_metrics),
            "ablation_summary": {
                "modal_only": metric_summary(modal_metrics),
                "stationary_first_temporal_coefficient": metric_summary(
                    stationary_metrics
                ),
            },
            "source_sample_values_decoded": decoded_samples,
            "fit_analysis_sample_values_used": targets.size,
            **protected_counters,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "real_quality_credit": bool(passed),
            "development_authorized": bool(passed),
            "exact_object_query_authorized": False,
            "representation_holdout_authorized": False,
            "r3b_authorized": False,
            "runtime_or_public_contract_changed": False,
            "authored_clip_fallback_required": True,
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory-manifest", required=True, type=Path)
    parser.add_argument("--audio", required=True, type=Path)
    parser.add_argument("--contacts", required=True, type=Path)
    parser.add_argument("--point-cloud", required=True, type=Path)
    parser.add_argument("--split", required=True, type=Path)
    parser.add_argument("--scale", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    output = run(
        repository_root(),
        arguments.inventory_manifest,
        arguments.audio,
        arguments.contacts,
        arguments.point_cloud,
        arguments.split,
        arguments.scale,
        arguments.output,
    )
    report = json.loads((output / "report.json").read_bytes())
    print(f"R3A V10 Beer Glass real fit: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
