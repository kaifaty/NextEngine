#!/usr/bin/env python3
"""Fit the frozen V8 explicit-modal representation on ObjectFolder object 91."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
import platform
import shutil
import struct
import sys
import tarfile
from pathlib import Path
from typing import Any

import numpy as np
import scipy
from scipy import fft, signal

import physical_sound_contact_field_r3a_v4_common as endpoint
import physical_sound_contact_field_r3a_v8_real_source_inventory as source

STUDY_ID = "physical-sound-contact-field-r3a-v8-object91-real-fit"
REVISION = "measured-force-explicit-modal-phase-residual-fit-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8-real-fit.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8-real-fit.report.v1"
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8-real-fit.model.v1"

INVENTORY_MANIFEST_SHA256 = "e1651b64b3867b2823accc1923b69c8437887b0acd90f6ced953526ebd8d147b"
FIT_CONTACT_IDS = ("18", "12", "4")
FIT_MEMBER_FILENAMES = ("mic.wav", "Force.wav", "metadata.yaml", "striking_force.yaml")
SAMPLE_RATE_HZ = 48_000
SOURCE_SAMPLES = 288_000
ANALYSIS_SAMPLES = 144_000
ALIGNMENT_SAMPLE = 512
BASELINE_SAMPLES = 12_000
MODE_COUNT = 64
RESIDUAL_BIN_COUNT = 14_500
ENVELOPE_BLOCKS = 512
TRANSIENT_SAMPLES = 2_048
MAXIMUM_SHARED_BYTES = 4 * 1024 * 1024
MAXIMUM_CONTACT_BYTES = 64 * 1024

CAPACITIES = (
    {"id": "global-damping", "per_contact_damping": False},
    {"id": "per-contact-damping", "per_contact_damping": True},
)

REQUIRED_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "openblas_num_threads": "1",
    "omp_num_threads": "1",
    "mkl_num_threads": "1",
}


class FitError(RuntimeError):
    """The frozen V8 real-fit boundary was violated."""


def canonical_json(value: Any) -> bytes:
    def ready(item: Any) -> Any:
        if isinstance(item, np.generic):
            return item.item()
        if isinstance(item, np.ndarray):
            return item.tolist()
        if isinstance(item, dict):
            return {key: ready(child) for key, child in item.items()}
        if isinstance(item, (list, tuple)):
            return [ready(child) for child in item]
        return item

    return (json.dumps(ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise FitError(f"{label} must be an external file")
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
                raise FitError(f"V8 real-fit environment changed for {key}")
    return observed


def implementation_hashes(directory: Path) -> dict[str, str]:
    files = {
        "fit": directory / "physical_sound_contact_field_r3a_v8_real_fit.py",
        "inventory": directory
        / "physical_sound_contact_field_r3a_v8_real_source_inventory.py",
        "endpoints": directory / "physical_sound_contact_field_r3a_v4_common.py",
    }
    return {key: sha256_file(path) for key, path in files.items()}


def validate_inventory_manifest(root: Path, path: Path) -> dict[str, Any]:
    resolved = require_external_file(root, path, "V8 inventory manifest")
    payload = resolved.read_bytes()
    if sha256_bytes(payload) != INVENTORY_MANIFEST_SHA256:
        raise FitError("V8 inventory manifest hash changed")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FitError("cannot parse V8 inventory manifest") from error
    if canonical_json(value) != payload:
        raise FitError("V8 inventory manifest is not canonical")
    if value.get("schema") != source.MANIFEST_SCHEMA:
        raise FitError("V8 inventory manifest schema changed")
    if value.get("waveform_sample_values_decoded") != 0:
        raise FitError("V8 inventory manifest already decoded waveform samples")
    roles = {item["contact_id"]: item["role"] for item in value["contacts"]}
    if roles != {"18": "fit", "12": "fit", "4": "fit", "20": "development", "27": "sealed"}:
        raise FitError("V8 inventory contact roles changed")
    return value


def fit_member_paths() -> set[str]:
    return {
        f"{source.OBJECT_ID}/audio/{contact_id}/{filename}"
        for contact_id in FIT_CONTACT_IDS
        for filename in FIT_MEMBER_FILENAMES
    }


def extract_fit_member_bytes(prefix: Path) -> dict[str, bytes]:
    expected_paths = fit_member_paths()
    found: dict[str, bytes] = {}
    try:
        with prefix.open("rb") as raw, tarfile.open(fileobj=raw, mode="r|gz") as archive:
            for member in archive:
                if member.name not in expected_paths:
                    continue
                if member.name in found or not member.isfile():
                    raise FitError(f"invalid fit member: {member.name}")
                payload = archive.extractfile(member)
                if payload is None:
                    raise FitError(f"cannot read fit member: {member.name}")
                value = payload.read()
                expected_bytes, expected_sha256 = source.EXPECTED_COMMITMENTS[member.name]
                if len(value) != expected_bytes or sha256_bytes(value) != expected_sha256:
                    raise FitError(f"fit member commitment changed: {member.name}")
                found[member.name] = value
                if len(found) == len(expected_paths):
                    break
    except (tarfile.TarError, EOFError, OSError) as error:
        raise FitError("cannot extract fit members") from error
    missing = sorted(expected_paths - set(found))
    if missing:
        raise FitError(f"fit members missing: {missing}")
    return found


def decode_pcm16(payload: bytes) -> np.ndarray:
    source.parse_pcm_header(payload[:64], len(payload))
    samples = np.frombuffer(payload, dtype="<i2", offset=44)
    if samples.shape != (SOURCE_SAMPLES,):
        raise FitError("fit PCM sample count changed")
    return samples.astype(np.float64) / 32768.0


def force_onset(force: np.ndarray) -> tuple[int, float]:
    absolute = np.abs(force)
    median = float(np.median(absolute))
    mad = float(np.median(np.abs(absolute - median)))
    threshold = max(0.05 * float(np.max(absolute)), median + 10.0 * mad)
    indices = np.flatnonzero(absolute >= threshold)
    if indices.size == 0:
        raise FitError("force onset threshold was never crossed")
    return int(indices[0]), threshold


def shift_and_crop(value: np.ndarray, shift: int) -> np.ndarray:
    shifted = np.zeros_like(value)
    if shift >= 0:
        shifted[shift:] = value[: value.size - shift]
    else:
        shifted[:shift] = value[-shift:]
    return np.ascontiguousarray(shifted[:ANALYSIS_SAMPLES], dtype=np.float64)


def preprocess_contact(audio: np.ndarray, force: np.ndarray) -> tuple[np.ndarray, np.ndarray, dict[str, Any]]:
    if audio.shape != (SOURCE_SAMPLES,) or force.shape != (SOURCE_SAMPLES,):
        raise FitError("source contact shape changed")
    audio_centered = audio - np.median(audio[:BASELINE_SAMPLES])
    force_centered = force - np.median(force[:BASELINE_SAMPLES])
    onset, threshold = force_onset(force_centered)
    shift = ALIGNMENT_SAMPLE - onset
    aligned_audio = shift_and_crop(audio_centered, shift)
    aligned_force = shift_and_crop(force_centered, shift)
    force_peak = float(np.max(np.abs(aligned_force)))
    if not math.isfinite(force_peak) or force_peak <= 0.0:
        raise FitError("aligned force peak is invalid")
    aligned_force /= force_peak
    return aligned_audio, aligned_force, {
        "source_onset_sample": onset,
        "alignment_shift_samples": shift,
        "force_threshold": threshold,
        "force_peak_before_normalization": force_peak,
    }


def load_fit_contacts(members: dict[str, bytes]) -> tuple[np.ndarray, np.ndarray, list[dict[str, Any]], float]:
    audio_rows = []
    force_rows = []
    preprocessing = []
    for contact_id in FIT_CONTACT_IDS:
        base = f"{source.OBJECT_ID}/audio/{contact_id}"
        audio = decode_pcm16(members[f"{base}/mic.wav"])
        force = decode_pcm16(members[f"{base}/Force.wav"])
        aligned_audio, aligned_force, descriptor = preprocess_contact(audio, force)
        descriptor["contact_id"] = contact_id
        audio_rows.append(aligned_audio)
        force_rows.append(aligned_force)
        preprocessing.append(descriptor)
    audio_values = np.stack(audio_rows)
    scale = 0.92 / float(np.max(np.abs(audio_values)))
    if not math.isfinite(scale) or scale <= 0.0:
        raise FitError("fit-only audio scale is invalid")
    return (
        np.ascontiguousarray(audio_values * scale, dtype=np.float64),
        np.ascontiguousarray(np.stack(force_rows), dtype=np.float64),
        preprocessing,
        scale,
    )


def contact_damping(target: np.ndarray, frequencies_hz: np.ndarray) -> np.ndarray:
    _, times, stft = signal.stft(
        target[ALIGNMENT_SAMPLE:],
        fs=SAMPLE_RATE_HZ,
        window="hann",
        nperseg=endpoint.MODAL_STFT_SAMPLES,
        noverlap=endpoint.MODAL_STFT_SAMPLES - endpoint.MODAL_STFT_HOP,
        boundary=None,
        padded=False,
    )
    stft_frequencies = np.fft.rfftfreq(endpoint.MODAL_STFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    magnitude = np.abs(stft)
    result = []
    for frequency in frequencies_hz:
        bin_index = int(np.argmin(np.abs(stft_frequencies - frequency)))
        envelope = magnitude[bin_index]
        maximum = max(float(np.max(envelope)), 1.0e-30)
        start = int(np.argmax(envelope))
        usable = np.arange(envelope.size) >= start
        usable &= times <= min(float(times[-1]), endpoint.MODAL_DAMPING_MAXIMUM_SECONDS)
        usable &= envelope >= maximum * endpoint.MODAL_DAMPING_FLOOR_RATIO
        if np.count_nonzero(usable) >= 6:
            slope = np.polyfit(
                times[usable], np.log(np.maximum(envelope[usable], 1.0e-30)), 1
            )[0]
            damping = float(
                np.clip(
                    -slope,
                    endpoint.MODAL_DAMPING_MINIMUM_PER_SECOND,
                    endpoint.MODAL_DAMPING_MAXIMUM_PER_SECOND,
                )
            )
        else:
            damping = 20.0
        result.append(damping)
    return np.asarray(result, dtype=np.float64)


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


def convolved_mode_basis(force: np.ndarray, frequency: float, damping: float) -> tuple[np.ndarray, np.ndarray]:
    time = np.arange(ANALYSIS_SAMPLES, dtype=np.float64) / SAMPLE_RATE_HZ
    envelope = np.exp(-damping * time)
    phase = 2.0 * math.pi * frequency * time
    length = fft.next_fast_len(2 * ANALYSIS_SAMPLES - 1)
    force_spectrum = fft.rfft(force, length)
    cosine = fft.irfft(force_spectrum * fft.rfft(envelope * np.cos(phase), length), length)[:ANALYSIS_SAMPLES]
    sine = fft.irfft(force_spectrum * fft.rfft(envelope * np.sin(phase), length), length)[:ANALYSIS_SAMPLES]
    return cosine, sine


def fit_modal_gains(
    targets: np.ndarray,
    forces: np.ndarray,
    frequencies_hz: np.ndarray,
    damping_by_contact: np.ndarray,
    rank_order: np.ndarray,
) -> tuple[np.ndarray, np.ndarray]:
    residuals = targets.copy()
    gains = np.zeros((targets.shape[0], frequencies_hz.size, 2), dtype=np.float64)
    for mode_index in rank_order:
        for contact_index in range(targets.shape[0]):
            cosine, sine = convolved_mode_basis(
                forces[contact_index],
                float(frequencies_hz[mode_index]),
                float(damping_by_contact[contact_index, mode_index]),
            )
            view = residuals[contact_index]
            gram = np.asarray(
                [
                    [np.dot(cosine, cosine), np.dot(cosine, sine)],
                    [np.dot(cosine, sine), np.dot(sine, sine)],
                ],
                dtype=np.float64,
            )
            rhs = np.asarray([np.dot(cosine, view), np.dot(sine, view)])
            gain = np.linalg.solve(gram + np.eye(2) * 1.0e-12, rhs)
            gains[contact_index, mode_index] = gain
            view -= gain[0] * cosine + gain[1] * sine
    return gains, residuals


def synthesize_modal(
    forces: np.ndarray,
    frequencies_hz: np.ndarray,
    damping_by_contact: np.ndarray,
    gains: np.ndarray,
) -> np.ndarray:
    result = np.zeros((forces.shape[0], ANALYSIS_SAMPLES), dtype=np.float64)
    for contact_index in range(forces.shape[0]):
        for mode_index, frequency in enumerate(frequencies_hz):
            cosine, sine = convolved_mode_basis(
                forces[contact_index],
                float(frequency),
                float(damping_by_contact[contact_index, mode_index]),
            )
            result[contact_index] += (
                gains[contact_index, mode_index, 0] * cosine
                + gains[contact_index, mode_index, 1] * sine
            )
    return result


def select_residual_bins(residuals: np.ndarray, count: int = RESIDUAL_BIN_COUNT) -> np.ndarray:
    spectra = np.fft.rfft(residuals, axis=1)
    frequencies = np.fft.rfftfreq(ANALYSIS_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    eligible = np.flatnonzero(
        (frequencies >= endpoint.EVALUATION_MIN_HZ)
        & (frequencies <= endpoint.EVALUATION_MAX_HZ)
    )
    summary = np.sqrt(np.mean(np.square(np.abs(spectra[:, eligible])), axis=0))
    if eligible.size < count:
        raise FitError("not enough eligible residual bins")
    order = np.argsort(-summary, kind="stable")[:count]
    return np.sort(eligible[order]).astype("<u4")


def block_envelope(target: np.ndarray, candidate: np.ndarray, block_count: int = ENVELOPE_BLOCKS) -> tuple[np.ndarray, np.ndarray]:
    edges = np.linspace(0, target.size, block_count + 1, dtype=np.int64)
    gains = np.empty(block_count, dtype=np.float64)
    centers = np.empty(block_count, dtype=np.float64)
    for index, (start, stop) in enumerate(zip(edges[:-1], edges[1:], strict=True)):
        view = candidate[start:stop]
        denominator = float(np.dot(view, view))
        gain = float(np.dot(target[start:stop], view)) / max(denominator, 1.0e-30)
        gains[index] = np.clip(gain, 0.0, 4.0)
        centers[index] = 0.5 * (start + stop - 1)
    quantized = gains.astype("<f2")
    decoded = quantized.astype(np.float64)
    curve = np.interp(np.arange(target.size), centers, decoded, left=decoded[0], right=decoded[-1])
    return quantized, curve


def encode_contact(
    target: np.ndarray,
    modal: np.ndarray,
    residual_spectrum: np.ndarray,
    selected_bins: np.ndarray,
    modal_quantized: np.ndarray,
    modal_scale: np.float32,
    damping_quantized: np.ndarray | None,
) -> tuple[np.ndarray, bytes, dict[str, Any]]:
    selected = residual_spectrum[selected_bins]
    complex_values = np.column_stack([selected.real, selected.imag])
    spectral_q, spectral_scale, spectral_decoded = quantize_int16(complex_values)
    sparse = np.zeros(ANALYSIS_SAMPLES // 2 + 1, dtype=np.complex128)
    sparse[selected_bins] = spectral_decoded[:, 0] + 1j * spectral_decoded[:, 1]
    candidate = modal + np.fft.irfft(sparse, n=ANALYSIS_SAMPLES)

    envelope_q, envelope_curve = block_envelope(target, candidate)
    candidate *= envelope_curve
    transient = target[:TRANSIENT_SAMPLES] - candidate[:TRANSIENT_SAMPLES]
    transient_q, transient_scale, transient_decoded = quantize_int16(transient)
    candidate[:TRANSIENT_SAMPLES] += transient_decoded
    output_scale = np.float32(
        np.dot(target, candidate) / max(float(np.dot(candidate, candidate)), 1.0e-30)
    )
    candidate *= float(output_scale)

    parts = [
        struct.pack("<f", float(modal_scale)),
        modal_quantized.tobytes(order="C"),
    ]
    if damping_quantized is not None:
        parts.append(damping_quantized.tobytes(order="C"))
    parts.extend(
        [
            struct.pack("<f", float(spectral_scale)),
            spectral_q.tobytes(order="C"),
            envelope_q.tobytes(order="C"),
            struct.pack("<f", float(transient_scale)),
            transient_q.tobytes(order="C"),
            struct.pack("<f", float(output_scale)),
        ]
    )
    record = b"".join(parts)
    return candidate, record, {
        "encoded_bytes": len(record),
        "record_sha256": sha256_bytes(record),
        "prediction_sha256": sha256_bytes(
            np.ascontiguousarray(candidate, dtype="<f8").tobytes()
        ),
        "spectral_scale": float(spectral_scale),
        "transient_scale": float(transient_scale),
        "output_scale": float(output_scale),
    }


def fit_capacity(
    staging: Path,
    capacity: dict[str, Any],
    targets: np.ndarray,
    forces: np.ndarray,
    poles: np.ndarray,
) -> tuple[dict[str, Any], dict[str, Any]]:
    capacity_id = capacity["id"]
    frequencies = poles[:, 0].astype(np.float32).astype(np.float64)
    global_damping = poles[:, 1].astype(np.float32).astype(np.float64)
    if capacity["per_contact_damping"]:
        raw_damping = np.stack(
            [contact_damping(target, frequencies) for target in targets]
        )
        damping_quantized = raw_damping.astype("<f2")
        damping_by_contact = damping_quantized.astype(np.float64)
    else:
        damping_quantized = None
        damping_by_contact = np.broadcast_to(
            global_damping[None, :], (targets.shape[0], frequencies.size)
        ).copy()

    raw_gains, _ = fit_modal_gains(
        targets, forces, frequencies, damping_by_contact, np.argsort(poles[:, 2])
    )
    gains_quantized = []
    gain_scales = []
    gains_decoded = []
    for gains in raw_gains:
        quantized, scale, decoded = quantize_float16_scaled(gains)
        gains_quantized.append(quantized)
        gain_scales.append(scale)
        gains_decoded.append(decoded)
    gains_quantized_array = np.stack(gains_quantized)
    gains_decoded_array = np.stack(gains_decoded)
    modal = synthesize_modal(
        forces, frequencies, damping_by_contact, gains_decoded_array
    )
    residuals = targets - modal
    selected_bins = select_residual_bins(residuals)
    residual_spectra = np.fft.rfft(residuals, axis=1)

    poles_payload = np.column_stack([frequencies, global_damping]).astype("<f4")
    poles_path = staging / f"{capacity_id}-poles.npy"
    bins_path = staging / f"{capacity_id}-residual-bins.npy"
    np.save(poles_path, poles_payload, allow_pickle=False)
    np.save(bins_path, selected_bins, allow_pickle=False)
    shared_bytes = poles_payload.nbytes + selected_bins.nbytes

    contacts = []
    maximum_record_bytes = 0
    all_absolute = True
    for contact_index, contact_id in enumerate(FIT_CONTACT_IDS):
        damping_for_record = (
            damping_quantized[contact_index]
            if damping_quantized is not None
            else None
        )
        candidate, record, record_descriptor = encode_contact(
            targets[contact_index],
            modal[contact_index],
            residual_spectra[contact_index],
            selected_bins,
            gains_quantized_array[contact_index],
            gain_scales[contact_index],
            damping_for_record,
        )
        record_path = staging / f"{capacity_id}-contact-{contact_id}.bin"
        record_path.write_bytes(record)
        metrics = endpoint.metrics(targets[contact_index], candidate)
        within = {
            name: metrics[name] <= endpoint.ABSOLUTE_THRESHOLDS[name]
            for name in endpoint.PRIMARY_ENDPOINTS
        }
        finite = bool(np.isfinite(candidate).all()) and all(
            math.isfinite(value) for value in metrics.values()
        )
        maximum_record_bytes = max(maximum_record_bytes, len(record))
        all_absolute &= finite and all(within.values())
        contacts.append(
            {
                "contact_id": contact_id,
                "metrics": metrics,
                "within_absolute_threshold": within,
                "finite": finite,
                **record_descriptor,
            }
        )

    budget_gate = {
        "shared_decoder": shared_bytes <= MAXIMUM_SHARED_BYTES,
        "contact_record": maximum_record_bytes <= MAXIMUM_CONTACT_BYTES,
    }
    passed = all_absolute and all(budget_gate.values())
    artifact = {
        "capacity_id": capacity_id,
        "poles_path": poles_path.name,
        "poles_sha256": sha256_file(poles_path),
        "residual_bins_path": bins_path.name,
        "residual_bins_sha256": sha256_file(bins_path),
        "record_sha256": {
            contact["contact_id"]: contact["record_sha256"] for contact in contacts
        },
        "shared_decoder_bytes": shared_bytes,
        "maximum_contact_record_bytes": maximum_record_bytes,
        "fit_gate_passed": passed,
    }
    report = {
        "capacity_id": capacity_id,
        "fit_gate_passed": passed,
        "all_fit_contacts_within_absolute_thresholds": all_absolute,
        "budget_gate": budget_gate,
        "shared_decoder_bytes": shared_bytes,
        "maximum_contact_record_bytes": maximum_record_bytes,
        "fit_contacts": contacts,
    }
    return artifact, report


def build_manifest(
    environment: dict[str, Any], implementation: dict[str, str]
) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenFitOnly",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "inventory_manifest_sha256": INVENTORY_MANIFEST_SHA256,
        "source_prefix_sha256": source.PREFIX_SHA256,
        "object_id": source.OBJECT_ID,
        "fit_contacts": list(FIT_CONTACT_IDS),
        "development_contact": "20",
        "sealed_contact": "27",
        "preprocessing": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "source_samples": SOURCE_SAMPLES,
            "analysis_samples": ANALYSIS_SAMPLES,
            "baseline_samples": BASELINE_SAMPLES,
            "alignment_sample": ALIGNMENT_SAMPLE,
            "force_onset": "max_0.05_peak_or_median_abs_plus_10_mad",
            "force_normalization": "per_contact_peak_one",
            "audio_normalization": "fit_group_peak_0.92",
            "resampling": "forbidden",
        },
        "representation": {
            "mode_count": MODE_COUNT,
            "frequency_band_hz": [endpoint.EVALUATION_MIN_HZ, endpoint.EVALUATION_MAX_HZ],
            "measured_force_excitation": True,
            "residual_bin_count": RESIDUAL_BIN_COUNT,
            "envelope_blocks": ENVELOPE_BLOCKS,
            "transient_samples": TRANSIENT_SAMPLES,
            "capacities": list(CAPACITIES),
        },
        "primary_endpoints": list(endpoint.PRIMARY_ENDPOINTS),
        "absolute_thresholds": endpoint.ABSOLUTE_THRESHOLDS,
        "maximum_shared_bytes": MAXIMUM_SHARED_BYTES,
        "maximum_contact_bytes": MAXIMUM_CONTACT_BYTES,
        "capacity_selection": "global_if_all_pass_else_per_contact_if_all_pass_else_reject",
        "environment": environment,
        "implementation_sha256": implementation,
        "fit_waveform_sample_values_decoded": 0,
        "development_waveform_sample_values_decoded": 0,
        "sealed_waveform_sample_values_decoded": 0,
        "real_development_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise FitError("V8 real-fit output must stay outside the repository")
    if resolved.exists():
        raise FitError("V8 real-fit output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def run(root: Path, inventory_path: Path, prefix_path: Path, output_path: Path) -> Path:
    validate_inventory_manifest(root, inventory_path)
    prefix = require_external_file(root, prefix_path, "ObjectFolder prefix")
    if prefix.stat().st_size != source.PREFIX_BYTES or sha256_file(prefix) != source.PREFIX_SHA256:
        raise FitError("ObjectFolder prefix identity changed")
    environment = environment_identity()
    implementation = implementation_hashes(Path(__file__).resolve().parent)
    manifest = build_manifest(environment, implementation)

    members = extract_fit_member_bytes(prefix)
    targets, forces, preprocessing, fit_scale = load_fit_contacts(members)
    maximum_poles = endpoint.estimate_object_poles(targets, MODE_COUNT)
    poles = endpoint.selected_poles(maximum_poles, MODE_COUNT)

    output, staging = prepare_output(root, output_path)
    try:
        artifacts = []
        capacity_reports = []
        for capacity in CAPACITIES:
            artifact, report = fit_capacity(staging, capacity, targets, forces, poles)
            artifacts.append(artifact)
            capacity_reports.append(report)
        eligible = [
            item["capacity_id"] for item in capacity_reports if item["fit_gate_passed"]
        ]
        selected = next(
            (capacity["id"] for capacity in CAPACITIES if capacity["id"] in eligible),
            None,
        )
        decision = (
            "READY_FOR_V8_REAL_DEVELOPMENT_EVALUATION"
            if selected is not None
            else "REJECT_V8_REAL_FIT_REPRESENTATION"
        )
        manifest["fit_waveform_sample_values_decoded"] = (
            len(FIT_CONTACT_IDS) * 2 * SOURCE_SAMPLES
        )
        manifest_bytes = canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        model = {
            "schema": MODEL_SCHEMA,
            "status": "Frozen",
            "decision": decision,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "selected_capacity": selected,
            "fit_audio_scale": fit_scale,
            "preprocessing": preprocessing,
            "capacities": artifacts,
            "development_waveform_sample_values_decoded": 0,
            "sealed_waveform_sample_values_decoded": 0,
        }
        model_bytes = canonical_json(model)
        (staging / "model.json").write_bytes(model_bytes)
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "model_sha256": sha256_bytes(model_bytes),
            "capacities_eligible_for_development": eligible,
            "selected_capacity": selected,
            "capacity_results": capacity_reports,
            "source_sample_values_decoded": len(FIT_CONTACT_IDS) * 2 * SOURCE_SAMPLES,
            "fit_analysis_sample_values_used": len(FIT_CONTACT_IDS) * 2 * ANALYSIS_SAMPLES,
            "development_waveform_sample_values_decoded": 0,
            "sealed_waveform_sample_values_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "real_quality_credit": False,
            "real_development_authorized": selected is not None,
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
    parser.add_argument("--archive-prefix", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    output = run(
        repository_root(),
        arguments.inventory_manifest,
        arguments.archive_prefix,
        arguments.output,
    )
    report = json.loads((output / "report.json").read_bytes())
    print(f"R3A V8 ObjectFolder Real fit: {output}")
    print(f"decision: {report['decision']}")
    print(f"selected capacity: {report['selected_capacity']}")
    print(f"manifest sha256: {sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
