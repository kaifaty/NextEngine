#!/usr/bin/env python3
"""Run the preregistered V12-C4a one-shot real force/FRF fit."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
import platform
import shutil
import tarfile
import wave
from pathlib import Path
from typing import Any

import numpy as np
import scipy
from scipy import signal

import physical_sound_contact_field_r3a_v11_b1_force_response_oracle as base
import physical_sound_contact_field_r3a_v11_b1r3_local_modal_support as poles
import physical_sound_contact_field_r3a_v12_c3_source_role_inventory as c3


STUDY_ID = "physical-sound-contact-field-r3a-v12-c4a-real-frf-fit"
REVISION = "object41-force-certified-one-shot-common-pole-v2"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c4a.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-r3a-v12-c4a-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-r3a-v12-c4a.report.v1",
}
CERTIFICATE_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c4a-force-certificate.v1"
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c4a-model.v1"

PROTOCOL_PATH = Path("docs/development/physical-sound-r3a-v12-c4-object41-real-frf-fit-protocol-2026-08-31.md")
RESEARCH_PATH = Path("docs/development/physical-sound-r3a-v12-c4-real-frf-fit-research-2026-08-31.md")
C1_RUNNER_PATH = Path("lab/scripts/physical_sound_contact_field_r3a_v12_c1_acquisition_coverage.py")
POLE_RUNNER_PATH = Path("lab/scripts/physical_sound_contact_field_r3a_v11_b1r3_local_modal_support.py")
C3_RUNNER_PATH = Path("lab/scripts/physical_sound_contact_field_r3a_v12_c3_source_role_inventory.py")

C3_MANIFEST_SHA256 = "a6dd159e417624f4db087cfc7289e72ba985a4fde6f88167bcc51a7277a766b3"
C3_REPORT_SHA256 = "a300199fe2c327694483c5613415243eecb897977c2ef3a7da77e1c6b92220d7"
RAW_PREFIX_SHA256 = "93ad4484e39a511074cfc48861a3206c9e098df1527569f62ca44c3bf8ab55eb"
FIT_CONTACTS = c3.EXPECTED_ROLE_IDS["estimator_fit"]
PROTECTED_CONTACTS = tuple(sorted(set(c3.CONTACT_IDS) - set(FIT_CONTACTS)))

SAMPLE_RATE_HZ = 48_000
SOURCE_SAMPLES = 288_000
BASELINE_SAMPLES = 12_000
ONSET_SAMPLE = 512
RETAINED_SAMPLES = 192_000
FFT_SAMPLES = 262_144
NOISE_WINDOW = 2_048
NOISE_HOP = 1_024
MINIMUM_HZ = 200.0
MAXIMUM_HZ = 9_500.0
REFERENCE_HZ = 12_000.0
MINIMUM_INPUT_SNR = 25.0
MINIMUM_RELATIVE_POWER = 0.005
MINIMUM_SUPPORTERS = 4
MINIMUM_REFIT_SUPPORTERS = 3
MINIMUM_CERTIFIED_FRACTION = 0.40
MINIMUM_CERTIFIED_WIDTH_HZ = 3_000.0
MINIMUM_CONTIGUOUS_WIDTH_HZ = 400.0
MINIMUM_LOCAL_CERTIFIED_FRACTION = 0.90
QUARTERS = tuple(tuple(FIT_CONTACTS[offset : offset + 4]) for offset in range(0, 16, 4))

GATES = {
    "minimum_mode_count": 6,
    "maximum_mode_count": 64,
    "minimum_stable_fraction": 0.75,
    "minimum_residue_contacts": 8,
    "maximum_mean_nrmse": 0.20,
    "maximum_contact_nrmse": 0.35,
    "maximum_spectrum_rmse_db": 4.0,
    "maximum_envelope_nrmse": 0.20,
    "maximum_impulse_ratio": 0.80,
    "maximum_raw_h1_ratio": 1.50,
    "maximum_output_modal_ratio": 1.05,
    "minimum_permuted_ratio": 1.05,
}


class FitError(RuntimeError):
    """The frozen C4a boundary or numeric contract was violated."""


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
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def counted_sha256_file(path: Path) -> tuple[str, int]:
    digest = hashlib.sha256()
    count = 0
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
            count += len(block)
    return digest.hexdigest(), count


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def implementation_hashes() -> dict[str, str]:
    root = repository_root()
    paths = {
        "protocol": root / PROTOCOL_PATH,
        "research": root / RESEARCH_PATH,
        "runner": Path(__file__).resolve(),
        "c1_runner": root / C1_RUNNER_PATH,
        "pole_runner": root / POLE_RUNNER_PATH,
        "c3_runner": root / C3_RUNNER_PATH,
    }
    return {name: sha256_file(path) for name, path in paths.items()}


def prepare_output(path: Path) -> tuple[Path, Path]:
    root = repository_root()
    output = path.resolve()
    if output.is_relative_to(root):
        raise FitError("C4a output must stay outside the repository")
    if output.exists():
        raise FitError("C4a output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return output, staging


def write_json(path: Path, value: Any) -> str:
    payload = canonical_json(value)
    path.write_bytes(payload)
    return sha256_bytes(payload)


def expected_manifest() -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "implementation_sha256": implementation_hashes(),
        "parent": {
            "c3_manifest_sha256": C3_MANIFEST_SHA256,
            "c3_inventory_report_sha256": C3_REPORT_SHA256,
            "raw_prefix_sha256": RAW_PREFIX_SHA256,
            "raw_prefix_bytes": c3.RAW_PREFIX_BYTES,
            "role_order_root": c3.ROLE_ORDER_ROOT,
        },
        "object": {
            "id": c3.OBJECT_ID,
            "name": c3.OBJECT_NAME,
            "published_material": c3.PUBLISHED_MATERIAL,
            "claim": "canonical_recorded_setup_normalized_instrument_counts",
        },
        "roles": {
            "fit": list(FIT_CONTACTS),
            "protected": list(PROTECTED_CONTACTS),
            "quarters": [list(value) for value in QUARTERS],
        },
        "sample_budget": {
            "microphone_decoded": len(FIT_CONTACTS) * SOURCE_SAMPLES,
            "force_decoded": len(FIT_CONTACTS) * SOURCE_SAMPLES,
            "total_decoded": len(FIT_CONTACTS) * SOURCE_SAMPLES * 2,
            "retained_float64": len(FIT_CONTACTS) * RETAINED_SAMPLES * 2,
            "protected_decoded": 0,
        },
        "preprocessing": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "source_samples": SOURCE_SAMPLES,
            "baseline_samples": BASELINE_SAMPLES,
            "onset_sample": ONSET_SAMPLE,
            "retained_samples": RETAINED_SAMPLES,
            "fft_samples": FFT_SAMPLES,
            "threshold_peak_ratio": 0.05,
            "threshold_mad_multiplier": 10.0,
            "double_impact_peak_ratio": 0.5,
            "double_impact_gap_samples": 240,
            "maximum_tail_rms_ratio": 0.10,
            "resample_filter_window_normalization_allowed": False,
        },
        "certificate": {
            "minimum_hz": MINIMUM_HZ,
            "maximum_hz": MAXIMUM_HZ,
            "reference_hz": REFERENCE_HZ,
            "minimum_input_snr": MINIMUM_INPUT_SNR,
            "minimum_relative_power": MINIMUM_RELATIVE_POWER,
            "minimum_supporters": MINIMUM_SUPPORTERS,
            "minimum_refit_supporters": MINIMUM_REFIT_SUPPORTERS,
            "minimum_fraction": MINIMUM_CERTIFIED_FRACTION,
            "minimum_width_hz": MINIMUM_CERTIFIED_WIDTH_HZ,
            "minimum_contiguous_width_hz": MINIMUM_CONTIGUOUS_WIDTH_HZ,
            "noise_window": NOISE_WINDOW,
            "noise_hop": NOISE_HOP,
            "response_access_allowed": False,
        },
        "candidate": {
            "transfer": "certificate_masked_noise_regularized_one_shot_H1",
            "poles": "C1_Gabor_common_pole",
            "residues": "fixed_pole_contact_local_least_squares",
            "coherence": "NOT_APPLICABLE_SINGLE_RECORD",
            "waveform_residual_allowed": False,
            "controls": [
                "raw_one_shot_H1",
                "shortest_response_impulse_assumption",
                "input_ignorant_output_modal",
                "cyclic_force_permutation",
                "baseline_only_zero_force",
            ],
            "control_definitions": {
                "impulse": "aligned_microphone_as_unit_impulse_transfer_target_band_taper",
                "output_modal": "aligned_microphone_common_poles_as_transfer",
                "zero_force": "centered_12000_sample_baseline_then_zero_fill",
            },
        },
        "gates": GATES,
        "environment": {
            "python": platform.python_version(),
            "python_implementation": platform.python_implementation(),
            "numpy": np.__version__,
            "scipy": scipy.__version__,
        },
        "data_policy": {
            "network_allowed": False,
            "outputs_external": True,
            "development_holdout_validator_shadow_allowed": False,
            "force_pcm_before_microphone_pcm": True,
            "next_authorized_step": "C4A_ZERO_READ_PREFLIGHT",
        },
    }


def validate_manifest(path: Path) -> tuple[bytes, dict[str, Any]]:
    try:
        payload = path.read_bytes()
        manifest = json.loads(payload)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FitError("cannot read C4a manifest") from error
    if payload != canonical_json(manifest):
        raise FitError("C4a manifest is not canonical JSON")
    if manifest != expected_manifest():
        raise FitError("C4a manifest or implementation changed")
    return payload, manifest


def freeze(output_path: Path) -> Path:
    output, staging = prepare_output(output_path)
    write_json(staging / "manifest.json", expected_manifest())
    staging.replace(output)
    return output


def preflight(manifest_path: Path, output_path: Path) -> Path:
    payload, manifest = validate_manifest(manifest_path)
    output, staging = prepare_output(output_path)
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        "study_id": STUDY_ID,
        "revision": REVISION,
        "stage": "preflight",
        "decision": "C4A_REAL_FRF_FIT_FROZEN",
        "manifest_sha256": sha256_bytes(payload),
        "implementation_sha256": manifest["implementation_sha256"],
        "roles": manifest["roles"],
        "sample_budget": manifest["sample_budget"],
        "coherence": "NOT_APPLICABLE_SINGLE_RECORD",
        "counters": {
            "network_requests": 0,
            "source_bytes_read": 0,
            "microphone_samples_decoded": 0,
            "force_samples_decoded": 0,
            "protected_samples_decoded": 0,
        },
        "fit_decode_authorized": False,
        "protected_decode_authorized": False,
        "next_authorized_step": "C4A_REPEATED_ZERO_READ_PREFLIGHT",
    }
    write_json(staging / "report.json", report)
    staging.replace(output)
    return output


def decode_pcm16_wave(payload: bytes) -> np.ndarray:
    try:
        with wave.open(io.BytesIO(payload), "rb") as stream:
            header = {
                "channels": stream.getnchannels(),
                "sample_width": stream.getsampwidth(),
                "sample_rate": stream.getframerate(),
                "frames": stream.getnframes(),
                "compression": stream.getcomptype(),
            }
            frames = stream.readframes(stream.getnframes())
    except (wave.Error, EOFError) as error:
        raise FitError("cannot decode fit WAV") from error
    expected = {
        "channels": 1,
        "sample_width": 2,
        "sample_rate": SAMPLE_RATE_HZ,
        "frames": SOURCE_SAMPLES,
        "compression": "NONE",
    }
    if header != expected or len(frames) != SOURCE_SAMPLES * 2:
        raise FitError("fit WAV header or payload changed")
    return np.frombuffer(frames, dtype="<i2").astype(np.float64) / 32768.0


def preprocess_force(force: np.ndarray) -> dict[str, Any]:
    if force.shape != (SOURCE_SAMPLES,) or not np.all(np.isfinite(force)):
        raise FitError("fit force sample count or finiteness changed")
    src = force - np.median(force[:BASELINE_SAMPLES])
    absolute = np.abs(src)
    baseline_absolute = absolute[:BASELINE_SAMPLES]
    baseline_median = float(np.median(baseline_absolute))
    baseline_mad = float(np.median(np.abs(baseline_absolute - baseline_median)))
    peak = float(np.max(absolute))
    threshold = max(0.05 * peak, baseline_median + 10.0 * baseline_mad)
    indices = np.flatnonzero(absolute >= threshold)
    if peak <= 0.0 or not math.isfinite(peak) or len(indices) == 0:
        raise FitError("OOD_FORCE_EVENT: missing force onset")
    onset = int(indices[0])
    start = onset - ONSET_SAMPLE
    end = start + RETAINED_SAMPLES
    if end > SOURCE_SAMPLES:
        raise FitError("OOD_FORCE_EVENT: fixed crop exceeds source")
    shifted_force = np.zeros(RETAINED_SAMPLES, dtype=np.float64)
    source_start = max(start, 0)
    target_start = max(-start, 0)
    count = min(SOURCE_SAMPLES - source_start, RETAINED_SAMPLES - target_start)
    shifted_force[target_start : target_start + count] = src[source_start : source_start + count]

    below = absolute < threshold
    gap = 0
    second = False
    for index in range(onset + 1, SOURCE_SAMPLES):
        gap = gap + 1 if below[index] else 0
        if gap >= 240:
            tail_start = index + 1
            second = bool(np.any(absolute[tail_start:] >= 0.5 * peak))
            break
    if second:
        raise FitError("OOD_FORCE_EVENT: double impact")
    return {
        "force": shifted_force,
        "baseline_force": src[:BASELINE_SAMPLES].copy(),
        "onset": onset,
        "crop_start": start,
        "threshold": threshold,
        "peak": peak,
        "baseline_mad": baseline_mad,
    }


def preprocess_microphone(microphone: np.ndarray, force_record: dict[str, Any]) -> dict[str, Any]:
    if microphone.shape != (SOURCE_SAMPLES,) or not np.all(np.isfinite(microphone)):
        raise FitError("fit microphone sample count or finiteness changed")
    mic = microphone - np.median(microphone[:BASELINE_SAMPLES])
    start = int(force_record["crop_start"])
    shifted = np.zeros(RETAINED_SAMPLES, dtype=np.float64)
    source_start = max(start, 0)
    target_start = max(-start, 0)
    count = min(SOURCE_SAMPLES - source_start, RETAINED_SAMPLES - target_start)
    shifted[target_start : target_start + count] = mic[source_start : source_start + count]
    early_rms = math.sqrt(
        float(np.mean(shifted[ONSET_SAMPLE : ONSET_SAMPLE + 24_000] ** 2))
    )
    tail_rms = math.sqrt(float(np.mean(shifted[-24_000:] ** 2)))
    tail_ratio = tail_rms / max(early_rms, 1.0e-30)
    if not math.isfinite(tail_ratio):
        raise FitError("fit microphone tail ratio is nonfinite")
    return {"microphone": shifted, "tail_rms_ratio": tail_ratio}


def preprocess_pair(microphone: np.ndarray, force: np.ndarray) -> dict[str, Any]:
    """Test helper that preserves the force-before-response access order."""
    force_record = preprocess_force(force)
    return {**force_record, **preprocess_microphone(microphone, force_record)}


def baseline_noise_power(baseline: np.ndarray) -> np.ndarray:
    window = signal.windows.hann(NOISE_WINDOW, sym=False)
    power = []
    for start in range(0, len(baseline) - NOISE_WINDOW + 1, NOISE_HOP):
        spectrum = np.fft.rfft(baseline[start : start + NOISE_WINDOW] * window, n=FFT_SAMPLES)
        power.append(np.abs(spectrum) ** 2 / np.sum(window * window) * RETAINED_SAMPLES)
    if not power:
        raise FitError("force baseline has no noise windows")
    median_power = np.median(np.stack(power), axis=0)
    median = float(np.median(baseline))
    mad = float(np.median(np.abs(baseline - median)))
    white_floor = RETAINED_SAMPLES * (1.4826 * mad) ** 2
    return np.maximum(median_power, white_floor)


def band_mask() -> np.ndarray:
    frequencies = np.fft.rfftfreq(FFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    return (frequencies >= MINIMUM_HZ) & (frequencies <= MAXIMUM_HZ)


def certificate_metrics(mask: np.ndarray) -> dict[str, float]:
    frequencies = np.fft.rfftfreq(FFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    target = band_mask()
    bin_hz = SAMPLE_RATE_HZ / FFT_SAMPLES
    longest = 0
    current = 0
    for value in mask[target]:
        current = current + 1 if value else 0
        longest = max(longest, current)
    return {
        "supported_fraction": float(np.mean(mask[target])),
        "cumulative_width_hz": float(np.sum(mask[target]) * bin_hz),
        "longest_contiguous_width_hz": float(longest * bin_hz),
        "minimum_hz": float(frequencies[target][0]),
        "maximum_hz": float(frequencies[target][-1]),
    }


def certificate_passes(metrics: dict[str, float]) -> bool:
    return (
        metrics["supported_fraction"] >= MINIMUM_CERTIFIED_FRACTION
        and metrics["cumulative_width_hz"] >= MINIMUM_CERTIFIED_WIDTH_HZ
        and metrics["longest_contiguous_width_hz"] >= MINIMUM_CONTIGUOUS_WIDTH_HZ
    )


def force_certificate(records: list[dict[str, Any]]) -> dict[str, Any]:
    if len(records) != len(FIT_CONTACTS):
        raise FitError("force certificate fit-contact count changed")
    supports = []
    snrs = []
    relative = []
    noises = []
    contact_valid = []
    frequencies = np.fft.rfftfreq(FFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    reference = frequencies <= REFERENCE_HZ
    for record in records:
        observed = np.abs(np.fft.rfft(record["force"], n=FFT_SAMPLES)) ** 2
        noise = baseline_noise_power(record["baseline_force"])
        corrected = np.maximum(observed - noise, 0.0)
        maximum = float(np.max(corrected[reference]))
        if maximum <= 0.0 or not math.isfinite(maximum):
            snr = np.zeros_like(corrected)
            rel = np.zeros_like(corrected)
            support = np.zeros_like(corrected, dtype=bool)
            contact_valid.append(False)
        else:
            snr = corrected / np.maximum(noise, 1.0e-30)
            rel = corrected / maximum
            support = (
                band_mask()
                & (snr >= MINIMUM_INPUT_SNR)
                & (rel >= MINIMUM_RELATIVE_POWER)
            )
            contact_valid.append(True)
        supports.append(support)
        snrs.append(snr)
        relative.append(rel)
        noises.append(noise)
    stack = np.stack(supports)
    counts = np.sum(stack, axis=0)
    certified = band_mask() & (counts >= MINIMUM_SUPPORTERS)
    refits = []
    refit_masks = []
    for quarter_index in range(4):
        remaining = np.delete(stack, slice(quarter_index * 4, quarter_index * 4 + 4), axis=0)
        mask = band_mask() & (np.sum(remaining, axis=0) >= MINIMUM_REFIT_SUPPORTERS)
        refit_masks.append(mask)
        metrics = certificate_metrics(mask)
        refits.append({"omitted_quarter": quarter_index, **metrics, "passed": certificate_passes(metrics)})
    metrics = certificate_metrics(certified)
    return {
        "schema": CERTIFICATE_SCHEMA,
        **metrics,
        "passed": certificate_passes(metrics) and all(item["passed"] for item in refits),
        "valid_contact_count": int(sum(contact_valid)),
        "contact_valid": contact_valid,
        "leave_quarter_out": refits,
        "refit_masks": np.stack(refit_masks),
        "support_count": counts,
        "certified_mask": certified,
        "input_snr": np.stack(snrs),
        "relative_power": np.stack(relative),
        "noise_power": np.stack(noises),
    }


def fit_common_modes(impulses: np.ndarray) -> dict[str, Any]:
    analysis = impulses[:, : base.TRANSFER_SAMPLES]
    coefficients = poles.gabor_coefficients(analysis)
    clusters, discovery = poles.discover_modes(coefficients)
    time = np.arange(RETAINED_SAMPLES, dtype=np.float64) / SAMPLE_RATE_HZ
    if not clusters:
        return {"modes": [], "reconstruction": np.zeros_like(impulses), "discovery": discovery}
    columns = []
    representatives = []
    for cluster in clusters:
        mode = cluster["representative"]
        if not (MINIMUM_HZ <= mode["frequency_hz"] <= MAXIMUM_HZ):
            continue
        envelope = np.exp(-mode["decay_per_second"] * time)
        phase = 2.0 * math.pi * mode["frequency_hz"] * time
        columns.extend((envelope * np.cos(phase), envelope * np.sin(phase)))
        representatives.append((cluster, mode))
    if not columns:
        return {"modes": [], "reconstruction": np.zeros_like(impulses), "discovery": discovery}
    design = np.stack(columns, axis=1)
    fitted = np.linalg.lstsq(design, impulses.T, rcond=None)[0]
    components = []
    energies = []
    for index in range(len(representatives)):
        component = (
            design[:, 2 * index, None] * fitted[2 * index][None, :]
            + design[:, 2 * index + 1, None] * fitted[2 * index + 1][None, :]
        ).T
        components.append(component)
        energies.append(float(np.sum(component * component)))
    maximum = max(energies, default=1.0e-300)
    modes = []
    retained = []
    for index, ((cluster, mode), component, energy) in enumerate(zip(representatives, components, energies, strict=True)):
        relative_db = 10.0 * math.log10(max(energy, 1.0e-300) / maximum)
        if relative_db < base.MODE_ENERGY_FLOOR_DB:
            continue
        residues = []
        magnitudes = []
        for contact in range(len(impulses)):
            cosine = float(fitted[2 * index, contact])
            sine = float(fitted[2 * index + 1, contact])
            magnitude = math.hypot(cosine, sine)
            magnitudes.append(magnitude)
            residues.append({"cosine": cosine, "sine": sine, "magnitude": magnitude})
        modes.append({
            "frequency_hz": float(mode["frequency_hz"]),
            "decay_per_second": float(mode["decay_per_second"]),
            "relative_energy_db": relative_db,
            "member_count": len(cluster["members"]),
            "residues": residues,
        })
        retained.append(component)
    if modes:
        strongest_by_contact = np.max(
            np.asarray(
                [
                    [residue["magnitude"] for residue in mode["residues"]]
                    for mode in modes
                ],
                dtype=np.float64,
            ),
            axis=0,
        )
        floors = strongest_by_contact * 10.0 ** (-35.0 / 20.0)
        for mode in modes:
            mode["non_negligible_residue_contacts"] = int(
                sum(
                    residue["magnitude"] >= floors[contact]
                    for contact, residue in enumerate(mode["residues"])
                )
            )
    reconstruction = np.sum(np.stack(retained), axis=0) if retained else np.zeros_like(impulses)
    return {"modes": modes, "reconstruction": reconstruction, "discovery": discovery}


def match_stability(full_modes: list[dict[str, Any]], refit_modes: list[list[dict[str, Any]]]) -> list[dict[str, Any]]:
    reports = []
    for index, mode in enumerate(full_modes):
        matches = 0
        detail = []
        for quarter, candidates in enumerate(refit_modes):
            eligible = [
                candidate for candidate in candidates
                if abs(candidate["frequency_hz"] - mode["frequency_hz"]) <= 2.0
                and abs(candidate["decay_per_second"] - mode["decay_per_second"])
                <= max(2.0, 0.25 * mode["decay_per_second"])
            ]
            unique = len(eligible) == 1
            matches += int(unique)
            detail.append({"quarter": quarter, "unique_match": unique})
        reports.append(
            {
                "mode_index": index,
                "matching_refits": matches,
                "refit_stable": matches >= 3,
                "refits": detail,
            }
        )
    return reports


def local_certificate_fraction(
    frequency_hz: float, decay_per_second: float, certified_mask: np.ndarray
) -> tuple[float, float]:
    frequencies = np.fft.rfftfreq(FFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    radius = max(24.0, 4.0 * decay_per_second / (2.0 * math.pi))
    local = np.abs(frequencies - frequency_hz) <= radius
    return float(np.mean(certified_mask[local])), radius


def reconstruct_modes(modes: list[dict[str, Any]], contact_count: int) -> np.ndarray:
    time = np.arange(RETAINED_SAMPLES, dtype=np.float64) / SAMPLE_RATE_HZ
    result = np.zeros((contact_count, RETAINED_SAMPLES), dtype=np.float64)
    for mode in modes:
        envelope = np.exp(-mode["decay_per_second"] * time)
        phase = 2.0 * math.pi * mode["frequency_hz"] * time
        cosine = envelope * np.cos(phase)
        sine = envelope * np.sin(phase)
        for contact, residue in enumerate(mode["residues"]):
            result[contact] += residue["cosine"] * cosine + residue["sine"] * sine
    return result


def baseline_only_records(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    result = []
    for record in records:
        force = np.zeros(RETAINED_SAMPLES, dtype=np.float64)
        force[:BASELINE_SAMPLES] = record["baseline_force"]
        result.append({"force": force, "baseline_force": record["baseline_force"]})
    return result


def validate_model_numbers(model: dict[str, Any]) -> None:
    arrays = model["arrays"]
    expected_shapes = {
        "regularized_transfer": (len(FIT_CONTACTS), FFT_SAMPLES // 2 + 1),
        "modal_transfer": (len(FIT_CONTACTS), RETAINED_SAMPLES),
        "prediction": (len(FIT_CONTACTS), RETAINED_SAMPLES),
    }
    for name, expected in expected_shapes.items():
        values = arrays[name]
        if values.shape != expected or not np.all(np.isfinite(values)):
            raise FitError(f"model array {name} is nonfinite or has the wrong shape")
    frequencies = [mode["frequency_hz"] for mode in model["full_modes"]]
    if any(
        not math.isfinite(mode["frequency_hz"])
        or not math.isfinite(mode["decay_per_second"])
        or mode["decay_per_second"] <= 0.0
        or not MINIMUM_HZ <= mode["frequency_hz"] <= MAXIMUM_HZ
        for mode in model["full_modes"]
    ):
        raise FitError("model pole bounds or finiteness changed")
    if frequencies != sorted(frequencies) or any(
        right - left <= base.DUPLICATE_FREQUENCY_HZ
        for left, right in zip(frequencies, frequencies[1:], strict=False)
    ):
        raise FitError("model pole order or duplicate separation changed")
    if not base.finite_tree(public_model(model)):
        raise FitError("model report contains nonfinite values")


def normalized_envelope_error(expected: np.ndarray, observed: np.ndarray) -> float:
    expected_envelope = np.abs(signal.hilbert(expected))
    observed_envelope = np.abs(signal.hilbert(observed))
    scale = max(float(np.max(expected_envelope)), 1.0e-30)
    return math.sqrt(float(np.mean(((expected_envelope - observed_envelope) / scale) ** 2)))


def predict_from_transfer(forces: np.ndarray, transfers: np.ndarray) -> np.ndarray:
    return np.stack([
        signal.fftconvolve(force, transfer, mode="full")[:RETAINED_SAMPLES]
        for force, transfer in zip(forces, transfers, strict=True)
    ])


def metrics_for(expected: np.ndarray, observed: np.ndarray) -> dict[str, Any]:
    rows = []
    for target, candidate in zip(expected, observed, strict=True):
        rows.append({
            "nrmse": base.normalized_rms_error(target, candidate),
            "spectrum_rmse_db": base.log_spectrum_rmse_db(target, candidate),
            "envelope_nrmse": normalized_envelope_error(target, candidate),
        })
    return {
        "contacts": rows,
        "mean_nrmse": float(np.mean([row["nrmse"] for row in rows])),
        "maximum_nrmse": float(np.max([row["nrmse"] for row in rows])),
        "mean_spectrum_rmse_db": float(np.mean([row["spectrum_rmse_db"] for row in rows])),
        "mean_envelope_nrmse": float(np.mean([row["envelope_nrmse"] for row in rows])),
        "median_nrmse": float(np.median([row["nrmse"] for row in rows])),
    }


def fit_model(records: list[dict[str, Any]], certificate: dict[str, Any]) -> dict[str, Any]:
    forces = np.stack([item["force"] for item in records])
    microphones = np.stack([item["microphone"] for item in records])
    x = np.fft.rfft(forces, n=FFT_SAMPLES, axis=1)
    y = np.fft.rfft(microphones, n=FFT_SAMPLES, axis=1)
    noise = certificate["noise_power"]
    taper = base.taper_mask(certificate["certified_mask"])
    unmasked_regularized = y * np.conj(x) / np.maximum(
        np.abs(x) ** 2 + noise, 1.0e-30
    )
    regularized = unmasked_regularized * taper[None, :]
    raw_h1 = np.zeros_like(regularized)
    np.divide(y, x, out=raw_h1, where=np.abs(x) > 1.0e-30)
    raw_h1 *= taper[None, :]
    impulses = np.fft.irfft(regularized, n=FFT_SAMPLES, axis=1)[:, :RETAINED_SAMPLES]
    full = fit_common_modes(impulses)
    refits = []
    for quarter_index in range(4):
        keep = [index for index in range(16) if index // 4 != quarter_index]
        refit_transfer = unmasked_regularized[keep] * base.taper_mask(
            certificate["refit_masks"][quarter_index]
        )[None, :]
        refit_impulses = np.fft.irfft(
            refit_transfer, n=FFT_SAMPLES, axis=1
        )[:, :RETAINED_SAMPLES]
        refits.append(fit_common_modes(refit_impulses)["modes"])
    stability = match_stability(full["modes"], refits)
    for mode, report in zip(full["modes"], stability, strict=True):
        local_fraction, radius = local_certificate_fraction(
            mode["frequency_hz"],
            mode["decay_per_second"],
            certificate["certified_mask"],
        )
        residue_supported = (
            mode["non_negligible_residue_contacts"] >= GATES["minimum_residue_contacts"]
        )
        report.update(
            {
                "local_certified_fraction": local_fraction,
                "local_radius_hz": radius,
                "residue_supported": residue_supported,
                "stable": (
                    report["refit_stable"]
                    and local_fraction >= MINIMUM_LOCAL_CERTIFIED_FRACTION
                    and residue_supported
                ),
            }
        )
        mode.update(
            {
                "local_certified_fraction": local_fraction,
                "local_radius_hz": radius,
                "stable": report["stable"],
            }
        )
    stable_modes = [mode for mode in full["modes"] if mode["stable"]]
    stable_transfer = reconstruct_modes(stable_modes, len(records))
    candidate = predict_from_transfer(forces, stable_transfer)
    raw_transfer = np.fft.irfft(raw_h1, n=FFT_SAMPLES, axis=1)[:, :RETAINED_SAMPLES]
    raw_prediction = predict_from_transfer(forces, raw_transfer)
    target_weights = base.taper_mask(band_mask())
    impulse_transfer = np.fft.irfft(
        np.fft.rfft(microphones, n=FFT_SAMPLES, axis=1) * target_weights[None, :],
        n=FFT_SAMPLES,
        axis=1,
    )[:, :RETAINED_SAMPLES]
    impulse_prediction = predict_from_transfer(forces, impulse_transfer)
    output_modal_fit = fit_common_modes(microphones)
    output_modal_prediction = predict_from_transfer(
        forces, output_modal_fit["reconstruction"]
    )
    permuted_forces = np.roll(forces, 1, axis=0)
    permuted_prediction = predict_from_transfer(permuted_forces, stable_transfer)
    baseline_certificate = force_certificate(baseline_only_records(records))
    model = {
        "schema": MODEL_SCHEMA,
        "full_modes": full["modes"],
        "stable_modes": stable_modes,
        "stability": stability,
        "full_candidate_count": len(full["modes"]),
        "stable_mode_count": len(stable_modes),
        "stable_fraction": len(stable_modes) / max(len(full["modes"]), 1),
        "metrics": {
            "candidate": metrics_for(microphones, candidate),
            "raw_h1": metrics_for(microphones, raw_prediction),
            "impulse": metrics_for(microphones, impulse_prediction),
            "output_modal": metrics_for(microphones, output_modal_prediction),
            "force_permuted": metrics_for(microphones, permuted_prediction),
        },
        "controls": {
            "output_modal_mode_count": len(output_modal_fit["modes"]),
            "baseline_only_zero_force": {
                **public_certificate(baseline_certificate),
                "admitted_poles": 0,
                "decision": (
                    "INVALID_ZERO_FORCE_ADMISSION"
                    if baseline_certificate["passed"]
                    else "DATA_INSUFFICIENT_FORCE_COVERAGE"
                ),
            },
        },
        "arrays": {
            "regularized_transfer": regularized,
            "modal_transfer": stable_transfer,
            "prediction": candidate,
        },
    }
    validate_model_numbers(model)
    return model


def load_fit_payloads(
    prefix: Path, report_path: Path
) -> tuple[list[dict[str, Any]], dict[str, int]]:
    try:
        prefix_bytes = prefix.stat().st_size
        prefix_hash, prefix_hash_bytes = counted_sha256_file(prefix)
    except OSError as error:
        raise FitError("cannot read bounded raw prefix") from error
    if prefix_bytes != c3.RAW_PREFIX_BYTES or prefix_hash != RAW_PREFIX_SHA256:
        raise FitError("raw prefix identity changed")
    report_payload = report_path.read_bytes()
    if sha256_bytes(report_payload) != C3_REPORT_SHA256:
        raise FitError("C3 report identity changed")
    report = json.loads(report_payload)
    if report.get("manifest_sha256") != C3_MANIFEST_SHA256:
        raise FitError("C3 report no longer binds the frozen manifest")
    records = {int(item["contact_id"]): item for item in report["records"]}
    expected = {
        f"{c3.OBJECT_ID}/audio/{contact}/{name}": (contact, name)
        for contact in FIT_CONTACTS for name in ("mic.wav", "Force.wav")
    }
    payloads: dict[tuple[int, str], bytes] = {}
    try:
        with prefix.open("rb") as raw:
            try:
                with tarfile.open(fileobj=raw, mode="r|gz") as archive:
                    for member in archive:
                        identity = expected.get(member.name)
                        if identity is None:
                            continue
                        stream = archive.extractfile(member)
                        if stream is None or identity in payloads or not member.isfile():
                            raise FitError("fit member missing or duplicated")
                        payload = stream.read()
                        source_key = "raw_microphone" if identity[1] == "mic.wav" else "raw_force"
                        expected_hash = records[identity[0]][source_key]["sha256"]
                        if sha256_bytes(payload) != expected_hash:
                            raise FitError("fit member hash changed")
                        payloads[identity] = payload
                        if len(payloads) == len(expected):
                            break
            except (tarfile.ReadError, EOFError):
                pass
            bytes_read = raw.tell()
    except OSError as error:
        raise FitError("cannot read bounded raw prefix") from error
    if len(payloads) != len(expected):
        raise FitError("fit members are incomplete")
    result = []
    for contact in FIT_CONTACTS:
        result.append(
            {
                "contact": contact,
                "microphone_payload": payloads[(contact, "mic.wav")],
                "force_payload": payloads[(contact, "Force.wav")],
            }
        )
    return result, {
        "prefix_identity_bytes_read": prefix_hash_bytes,
        "prefix_archive_bytes_read": bytes_read,
        "c3_report_bytes_read": len(report_payload),
        "selected_member_bytes": sum(map(len, payloads.values())),
    }


def public_certificate(value: dict[str, Any]) -> dict[str, Any]:
    excluded = {
        "support_count",
        "certified_mask",
        "refit_masks",
        "input_snr",
        "relative_power",
        "noise_power",
    }
    return {key: child for key, child in value.items() if key not in excluded}


def public_model(value: dict[str, Any]) -> dict[str, Any]:
    return {key: child for key, child in value.items() if key != "arrays"}


def run(manifest_path: Path, prefix: Path, c3_report: Path, output_path: Path) -> Path:
    payload, manifest = validate_manifest(manifest_path)
    output, staging = prepare_output(output_path)
    loaded, read_counts = load_fit_payloads(
        prefix.resolve(strict=True), c3_report.resolve(strict=True)
    )
    force_records = []
    for item in loaded:
        try:
            processed = preprocess_force(decode_pcm16_wave(item["force_payload"]))
        except FitError as error:
            if not str(error).startswith("OOD_FORCE_EVENT"):
                raise
            report = {
                "schema": REPORT_SCHEMAS["run"],
                "study_id": STUDY_ID,
                "revision": REVISION,
                "stage": "run",
                "decision": "OOD_FORCE_EVENT",
                "manifest_sha256": sha256_bytes(payload),
                "failed_contact": item["contact"],
                "microphone_fit_executed": False,
                "counters": {
                    "network_requests": 0,
                    "source_bytes_read": (
                        read_counts["prefix_identity_bytes_read"]
                        + read_counts["prefix_archive_bytes_read"]
                    ),
                    **read_counts,
                    "microphone_samples_decoded": 0,
                    "force_samples_decoded": (len(force_records) + 1) * SOURCE_SAMPLES,
                    "protected_samples_decoded": 0,
                },
            }
            write_json(staging / "report.json", report)
            staging.replace(output)
            return output
        force_records.append({"contact": item["contact"], **processed})
    certificate = force_certificate(force_records)
    certificate_hash = write_json(staging / "certificate.json", public_certificate(certificate))
    counters = {
        "network_requests": 0,
        "source_bytes_read": (
            read_counts["prefix_identity_bytes_read"]
            + read_counts["prefix_archive_bytes_read"]
        ),
        **read_counts,
        "microphone_samples_decoded": 0,
        "force_samples_decoded": len(force_records) * SOURCE_SAMPLES,
        "protected_samples_decoded": 0,
    }
    if not certificate["passed"]:
        report = {
            "schema": REPORT_SCHEMAS["run"], "study_id": STUDY_ID, "revision": REVISION,
            "stage": "run", "decision": "DATA_INSUFFICIENT_FORCE_COVERAGE",
            "manifest_sha256": sha256_bytes(payload), "certificate_sha256": certificate_hash,
            "microphone_fit_executed": False, "counters": counters,
            "protected_roles_opened": False,
            "coherence": "NOT_APPLICABLE_SINGLE_RECORD",
        }
        write_json(staging / "report.json", report)
        staging.replace(output)
        return output
    records = []
    for item, force_record in zip(loaded, force_records, strict=True):
        microphone = preprocess_microphone(
            decode_pcm16_wave(item["microphone_payload"]), force_record
        )
        records.append({**force_record, **microphone})
    counters["microphone_samples_decoded"] = len(records) * SOURCE_SAMPLES
    if any(item["tail_rms_ratio"] > 0.10 for item in records):
        decision = "DATA_INSUFFICIENT_RESPONSE_DURATION"
        model = None
        checks = []
    else:
        model = fit_model(records, certificate)
        metrics = model["metrics"]
        candidate = metrics["candidate"]
        ratios = {
            "candidate_over_impulse": candidate["mean_nrmse"]
            / max(metrics["impulse"]["mean_nrmse"], 1.0e-30),
            "candidate_over_raw_h1": candidate["mean_nrmse"]
            / max(metrics["raw_h1"]["mean_nrmse"], 1.0e-30),
            "candidate_over_output_modal": candidate["mean_nrmse"]
            / max(metrics["output_modal"]["mean_nrmse"], 1.0e-30),
        }
        permutation_ratio = metrics["force_permuted"]["median_nrmse"] / max(candidate["median_nrmse"], 1.0e-30)
        checks = [
            {
                "name": "stable_mode_count",
                "value": model["stable_mode_count"],
                "passed": GATES["minimum_mode_count"]
                <= model["stable_mode_count"]
                <= GATES["maximum_mode_count"],
            },
            {
                "name": "stable_fraction",
                "value": model["stable_fraction"],
                "passed": model["stable_fraction"] >= GATES["minimum_stable_fraction"],
            },
            {
                "name": "minimum_residue_contacts",
                "value": min(
                    (
                        mode["non_negligible_residue_contacts"]
                        for mode in model["stable_modes"]
                    ),
                    default=0,
                ),
                "passed": bool(model["stable_modes"])
                and all(
                    mode["non_negligible_residue_contacts"]
                    >= GATES["minimum_residue_contacts"]
                    for mode in model["stable_modes"]
                ),
            },
            {
                "name": "mean_nrmse",
                "value": candidate["mean_nrmse"],
                "passed": candidate["mean_nrmse"] <= GATES["maximum_mean_nrmse"],
            },
            {
                "name": "maximum_contact_nrmse",
                "value": candidate["maximum_nrmse"],
                "passed": candidate["maximum_nrmse"]
                <= GATES["maximum_contact_nrmse"],
            },
            {
                "name": "mean_spectrum_rmse_db",
                "value": candidate["mean_spectrum_rmse_db"],
                "passed": candidate["mean_spectrum_rmse_db"]
                <= GATES["maximum_spectrum_rmse_db"],
            },
            {
                "name": "mean_envelope_nrmse",
                "value": candidate["mean_envelope_nrmse"],
                "passed": candidate["mean_envelope_nrmse"]
                <= GATES["maximum_envelope_nrmse"],
            },
            {
                "name": "candidate_over_impulse",
                "value": ratios["candidate_over_impulse"],
                "passed": ratios["candidate_over_impulse"]
                <= GATES["maximum_impulse_ratio"],
            },
            {
                "name": "candidate_over_raw_h1",
                "value": ratios["candidate_over_raw_h1"],
                "passed": ratios["candidate_over_raw_h1"]
                <= GATES["maximum_raw_h1_ratio"],
            },
            {
                "name": "candidate_over_output_modal",
                "value": ratios["candidate_over_output_modal"],
                "passed": ratios["candidate_over_output_modal"]
                <= GATES["maximum_output_modal_ratio"],
            },
            {
                "name": "force_permuted_over_candidate",
                "value": permutation_ratio,
                "passed": permutation_ratio >= GATES["minimum_permuted_ratio"],
            },
            {
                "name": "baseline_only_zero_force",
                "value": model["controls"]["baseline_only_zero_force"]["admitted_poles"],
                "passed": not model["controls"]["baseline_only_zero_force"]["passed"],
            },
        ]
        if not checks[-2]["passed"]:
            decision = "OOD_FORCE_NOT_DISCRIMINATIVE"
        else:
            decision = (
                "READY_FOR_C4B_DEVELOPMENT_PROTOCOL"
                if all(item["passed"] for item in checks)
                else "REJECT_C4A_REAL_COMMON_POLE"
            )
        model["ratios"] = ratios
        model["permutation_ratio"] = permutation_ratio
        model["gate"] = {"passed": all(item["passed"] for item in checks), "checks": checks}
        for name, array in model["arrays"].items():
            np.save(staging / f"{name}.npy", array, allow_pickle=False)
        write_json(staging / "model.json", public_model(model))
    report = {
        "schema": REPORT_SCHEMAS["run"], "study_id": STUDY_ID, "revision": REVISION,
        "stage": "run", "decision": decision, "manifest_sha256": sha256_bytes(payload),
        "certificate_sha256": certificate_hash,
        "preprocessing": [{"contact": item["contact"], "onset": item["onset"], "tail_rms_ratio": item["tail_rms_ratio"]} for item in records],
        "model": None if model is None else public_model(model), "counters": counters,
        "gate": {"passed": bool(checks) and all(item["passed"] for item in checks), "checks": checks},
        "protected_roles_opened": False, "coherence": "NOT_APPLICABLE_SINGLE_RECORD",
    }
    write_json(staging / "report.json", report)
    staging.replace(output)
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=("freeze", "preflight", "run"))
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--raw-prefix", type=Path)
    parser.add_argument("--c3-report", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    if args.stage == "freeze":
        freeze(args.output)
    elif args.stage == "preflight":
        if args.manifest is None:
            raise FitError("preflight requires --manifest")
        preflight(args.manifest, args.output)
    else:
        if args.manifest is None or args.raw_prefix is None or args.c3_report is None:
            raise FitError("run requires --manifest, --raw-prefix and --c3-report")
        run(args.manifest, args.raw_prefix, args.c3_report, args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
