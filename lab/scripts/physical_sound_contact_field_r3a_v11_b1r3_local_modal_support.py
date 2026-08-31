#!/usr/bin/env python3
"""Run the preregistered V11-B1R3 local modal-support oracle."""

from __future__ import annotations

import argparse
import json
import math
import shutil
from pathlib import Path
from typing import Any

import numpy as np
from scipy import signal
import scipy

import physical_sound_contact_field_r3a_v11_b1_force_response_oracle as base


STUDY_ID = "physical-sound-contact-field-r3a-v11-b1r3-local-modal-support"
REVISION = "noise-calibrated-gtls-local-modal-support-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v11-b1r3.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-r3a-v11-b1r3-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-r3a-v11-b1r3.report.v1",
}
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v11-b1r3.model.v1"

PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-r3a-v11-b1r3-local-modal-support-protocol-2026-08-31.md"
)
PARENT_RUNNER_PATH = (
    "lab/scripts/"
    "physical_sound_contact_field_r3a_v11_b1r2_noise_aware_gtls.py"
)
BASE_RUNNER_PATH = (
    "lab/scripts/"
    "physical_sound_contact_field_r3a_v11_b1_force_response_oracle.py"
)
PARENT_RESULT_PATH = (
    "docs/development/"
    "physical-sound-r3a-v11-b1r2-noise-aware-gtls-result-2026-08-31.md"
)
POLE_CORE_PATH = "lab/scripts/physical_sound_subband_common_pole_control.py"

TRUTH_FREQUENCIES_HZ = np.asarray(
    [613.0, 1_097.0, 1_777.0, 2_819.0, 4_433.0, 6_643.0, 8_000.06103515625],
    dtype=np.float64,
)
TRUTH_DECAYS_PER_SECOND = np.asarray(
    [5.25, 8.25, 11.75, 16.5, 24.0, 34.5, 49.0], dtype=np.float64
)
FIT_PROFILES = (
    ("half_sine", (11.0,)),
    ("hann", (23.0,)),
    ("beta", (41.0, 3.0, 4.0)),
    ("half_sine", (67.0,)),
)
DEVELOPMENT_PROFILES = (("hann", (15.0,)), ("beta", (49.0, 4.0, 3.0)))
HOLDOUT_PROFILES = (("half_sine", (29.0,)), ("beta", (59.0, 3.0, 5.0)))
COMB_NOTCH_PROFILE = ("positive_comb_notch", ())
WEAK_PROFILE = ("hann", (1025.0,))
LOW_COHERENCE_PROFILE = ("half_sine", (23.0,))
FIT_REPEATS = 4
ROLE_SEEDS = {
    "fit": {"force": 2_026_113_101, "response": 2_026_113_102, "room": 2_026_113_103},
    "development": {
        "force": 2_026_113_201,
        "response": 2_026_113_202,
        "room": 2_026_113_203,
    },
    "holdout": {
        "force": 2_026_113_301,
        "response": 2_026_113_302,
        "room": 2_026_113_303,
    },
    "corruption": {
        "force": 2_026_113_401,
        "response": 2_026_113_402,
        "room": 2_026_113_403,
    },
}

MAXIMUM_DISCOVERY_HZ = 10_000.0
DISCOVERY_INPUT_SNR_FLOOR = 3.0
DISCOVERY_RELATIVE_SIGNAL_POWER_FLOOR = 1.0e-8
LOCAL_INPUT_SNR_FLOOR = 10.0
LOCAL_DISCOVERY_MASK_FRACTION = 0.50
MODE_SUPPORT_COHERENCE = 0.90
RESIDUE_CONFIDENCE_COHERENCE = 0.85
MINIMUM_SUPPORTING_CONTACTS = 3
MINIMUM_CONFIDENT_RESIDUES = 3
EIGENVECTOR_INPUT_FLOOR = 1.0e-12

GATES = {
    "supported_truth_mode_count": 7,
    "unsupported_truth_mode_count": 0,
    "false_positive_count": 0,
    "minimum_supporting_contacts": MINIMUM_SUPPORTING_CONTACTS,
    "minimum_confident_residues": MINIMUM_CONFIDENT_RESIDUES,
    "maximum_frequency_error_hz": 1.0,
    "maximum_decay_error_per_second": 1.5,
    "maximum_relative_decay_error": 0.20,
    "maximum_mean_held_nrmse": 0.08,
    "maximum_held_nrmse": 0.15,
    "maximum_mean_log_spectrum_rmse_db": 2.0,
    "maximum_impulse_nrmse_ratio": 0.70,
    "maximum_raw_modal_nrmse_ratio": 0.80,
    "maximum_raw_h1_nrmse_ratio": 1.50,
    "maximum_gtls_over_best_corrected_ratio": 1.05,
    "maximum_identity_nrmse": 1.0e-11,
    "comb_supported_truth_mode_count": 6,
    "comb_unsupported_truth_indices": [6],
    "maximum_low_coherence_residual_nrmse": 0.20,
    "minimum_double_impact_residual_nrmse": 0.15,
}

OracleError = base.OracleError


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=("freeze", "preflight", "run"))
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def implementation_hashes(root: Path) -> dict[str, str]:
    paths = {
        "runner": Path(__file__).resolve(),
        "protocol": (root / PROTOCOL_PATH).resolve(strict=True),
        "parent_runner": (root / PARENT_RUNNER_PATH).resolve(strict=True),
        "base_runner": (root / BASE_RUNNER_PATH).resolve(strict=True),
        "parent_result": (root / PARENT_RESULT_PATH).resolve(strict=True),
        "common_pole_core": (root / POLE_CORE_PATH).resolve(strict=True),
    }
    return {name: base.sha256_file(path) for name, path in paths.items()}


def profile_descriptor(profile: tuple[str, tuple[float, ...]]) -> dict[str, Any]:
    return {"family": profile[0], "parameters": list(profile[1])}


def expected_manifest(root: Path) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "implementation_sha256": implementation_hashes(root),
        "fixture": {
            "sample_rate_hz": base.SAMPLE_RATE_HZ,
            "transfer_samples": base.TRANSFER_SAMPLES,
            "analysis_fft_samples": base.ANALYSIS_FFT_SAMPLES,
            "contact_count": base.CONTACT_COUNT,
            "truth_frequencies_hz": TRUTH_FREQUENCIES_HZ.tolist(),
            "truth_decays_per_second": TRUTH_DECAYS_PER_SECOND.tolist(),
            "transfer_peak": base.TRANSFER_PEAK,
            "residue_magnitude_rule": (
                "(0.29 + 0.041*((11*contact + 7*mode + 5) mod 17))/(1 + 0.025*mode)"
            ),
            "residue_phase_rule": (
                "0.173 + 0.23*contact + 0.19*mode + 0.037*contact*mode radians"
            ),
        },
        "roles": {
            "fit": {
                "profiles": [profile_descriptor(value) for value in FIT_PROFILES],
                "repeats": FIT_REPEATS,
                "trial_count": base.CONTACT_COUNT * len(FIT_PROFILES) * FIT_REPEATS,
            },
            "development": [profile_descriptor(value) for value in DEVELOPMENT_PROFILES],
            "holdout": [profile_descriptor(value) for value in HOLDOUT_PROFILES],
            "seeds": ROLE_SEEDS,
            "order": ["sensor_calibration_and_fit", "development", "holdout"],
        },
        "noise_calibration": {
            "force_sensor_noise_peak_ratio": base.FORCE_NOISE_PEAK_RATIO,
            "response_sensor_noise_rms_ratio": base.RESPONSE_NOISE_RMS_RATIO,
            "unmeasured_room_tail_rms_ratio": base.ROOM_TAIL_RMS_RATIO,
            "calibrated_components": ["force_sensor_noise", "response_sensor_noise"],
            "uncalibrated_components": ["room_disturbance", "dynamic_interference"],
        },
        "candidate": {
            "estimator": "noise_corrected_dominant_covariance_GTLS",
            "discovery_maximum_hz": MAXIMUM_DISCOVERY_HZ,
            "discovery_input_snr_floor": DISCOVERY_INPUT_SNR_FLOOR,
            "discovery_relative_signal_power_floor": DISCOVERY_RELATIVE_SIGNAL_POWER_FLOOR,
            "local_input_snr_floor": LOCAL_INPUT_SNR_FLOOR,
            "local_discovery_mask_fraction": LOCAL_DISCOVERY_MASK_FRACTION,
            "mode_support_coherence": MODE_SUPPORT_COHERENCE,
            "residue_confidence_coherence": RESIDUE_CONFIDENCE_COHERENCE,
            "minimum_supporting_contacts": MINIMUM_SUPPORTING_CONTACTS,
            "minimum_confident_residues": MINIMUM_CONFIDENT_RESIDUES,
            "selected_transfer": "explicit locally-supported common-modal reconstruction",
            "broad_band_coverage_is_gate": False,
            "waveform_residual_allowed": False,
            "controls": [
                "raw_H1",
                "noise_corrected_H1",
                "noise_corrected_H2",
                "equal_weight_direct_division",
                "shortest_response_impulse_assumption",
                "input_ignorant_raw_output_modal",
            ],
        },
        "corruptions": {
            "partial_support": profile_descriptor(COMB_NOTCH_PROFILE),
            "weak_excitation": profile_descriptor(WEAK_PROFILE),
            "low_coherence": {
                "profile": profile_descriptor(LOW_COHERENCE_PROFILE),
                "uncalibrated_interference_rms_ratio": 0.75,
            },
            "missing_second_impact": {
                "profile": profile_descriptor(("half_sine", (23.0,))),
                "gain": 0.8,
                "delays_samples": [389, 617, 941, 1207],
            },
        },
        "gates": GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "fresh_synthetic_only": True,
            "network_allowed": False,
            "real_payload_allowed": False,
            "parent_holdout_allowed": False,
            "holdout_after_development_failure_allowed": False,
            "threshold_tuning_after_freeze_allowed": False,
            "quality_admission_runtime_credit_allowed": False,
            "outputs_must_be_external": True,
        },
        "stop_rule": (
            "PASS_KNOWN_TRUTH_FRF opens only B2 zero-decode source feasibility; "
            "any valid gate failure rejects this local-support revision"
        ),
    }


def load_manifest(root: Path, path: Path) -> tuple[bytes, dict[str, Any]]:
    resolved = base.require_external_file(root, path, "B1R3 manifest")
    payload = resolved.read_bytes()
    if len(payload) > 256 * 1024:
        raise OracleError("B1R3 manifest exceeds 256 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleError("cannot parse B1R3 manifest") from error
    if payload != base.canonical_json(value):
        raise OracleError("B1R3 manifest is not canonical JSON")
    if value != expected_manifest(root):
        raise OracleError("B1R3 manifest or implementation lineage changed")
    return payload, value


def exact_transfers() -> np.ndarray:
    time = np.arange(base.TRANSFER_SAMPLES, dtype=np.float64) / base.SAMPLE_RATE_HZ
    transfers = np.zeros((base.CONTACT_COUNT, base.TRANSFER_SAMPLES), dtype=np.float64)
    for contact in range(base.CONTACT_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
        ):
            gain = (0.29 + 0.041 * ((11 * contact + 7 * mode + 5) % 17)) / (
                1.0 + 0.025 * mode
            )
            phase = 0.173 + 0.23 * contact + 0.19 * mode + 0.037 * contact * mode
            transfers[contact] += (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
    peak = float(np.max(np.abs(transfers)))
    if not math.isfinite(peak) or peak <= 0.0:
        raise OracleError("B1R3 exact transfer has invalid peak")
    transfers *= base.TRANSFER_PEAK / peak
    return transfers


def force_profile(profile: tuple[str, tuple[float, ...]]) -> np.ndarray:
    if profile[0] != COMB_NOTCH_PROFILE[0]:
        return base.force_profile(profile)
    pulse = base.force_profile(("half_sine", (3.0,)))[:3]
    result = np.zeros(base.TRANSFER_SAMPLES, dtype=np.float64)
    for delay, weight in zip((0, 3, 6, 9, 12), (1.0, 4.0, 6.0, 4.0, 1.0), strict=True):
        result[delay : delay + len(pulse)] += weight * pulse
    total = float(np.sum(result))
    if not math.isfinite(total) or total <= 0.0 or np.any(result < 0.0):
        raise OracleError("B1R3 comb-notch force is invalid")
    return result / total


def seed_triplet(offset: int) -> dict[str, int]:
    return {name: value + offset for name, value in ROLE_SEEDS["corruption"].items()}


def generate_role(
    role: str,
    profiles: tuple[tuple[str, tuple[float, ...]], ...],
    transfers: np.ndarray,
    repeats: int,
    *,
    seeds: dict[str, int] | None = None,
    interference_rms_ratio: float = 0.0,
) -> dict[str, Any]:
    count = base.CONTACT_COUNT * len(profiles) * repeats
    selected_seeds = ROLE_SEEDS[role] if seeds is None else seeds
    force_rngs = base.child_generators(selected_seeds["force"], count)
    response_rngs = base.child_generators(selected_seeds["response"], count)
    room_rngs = base.child_generators(selected_seeds["room"], count)
    spectra = np.fft.rfft(transfers, n=base.ANALYSIS_FFT_SAMPLES, axis=1)
    trials = []
    for contact in range(base.CONTACT_COUNT):
        for profile_index, profile in enumerate(profiles):
            for repeat in range(repeats):
                index = (contact * len(profiles) + profile_index) * repeats + repeat
                exact_force = force_profile(profile)
                clean_response = base.fft_convolve(exact_force, spectra[contact])
                response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
                if response_rms <= 0.0 or not math.isfinite(response_rms):
                    raise OracleError("B1R3 clean response has invalid RMS")
                force_sensor_noise = force_rngs[index].normal(
                    0.0,
                    base.FORCE_NOISE_PEAK_RATIO * float(np.max(exact_force)),
                    base.TRANSFER_SAMPLES,
                )
                response_sensor_noise = response_rngs[index].normal(
                    0.0,
                    base.RESPONSE_NOISE_RMS_RATIO * response_rms,
                    base.TRANSFER_SAMPLES,
                )
                room_disturbance = base.room_tail(
                    room_rngs[index], base.ROOM_TAIL_RMS_RATIO * response_rms
                )
                if interference_rms_ratio > 0.0:
                    interference = room_rngs[index].standard_normal(base.TRANSFER_SAMPLES)
                    interference = signal.lfilter([1.0], [1.0, -0.91], interference)
                    interference_rms = math.sqrt(float(np.mean(interference * interference)))
                    if interference_rms <= 0.0 or not math.isfinite(interference_rms):
                        raise OracleError("B1R3 interference has invalid RMS")
                    room_disturbance += (
                        interference
                        * interference_rms_ratio
                        * response_rms
                        / interference_rms
                    )
                trials.append(
                    {
                        "contact": contact,
                        "profile_index": profile_index,
                        "repeat": repeat,
                        "profile": profile_descriptor(profile),
                        "exact_force": exact_force,
                        "force_sensor_noise": force_sensor_noise,
                        "observed_force": exact_force + force_sensor_noise,
                        "clean_response": clean_response,
                        "response_sensor_noise": response_sensor_noise,
                        "room_disturbance": room_disturbance,
                        "observed_response": (
                            clean_response + response_sensor_noise + room_disturbance
                        ),
                    }
                )
    return {"role": role, "profiles": profiles, "repeats": repeats, "trials": trials}


def analysis_band() -> np.ndarray:
    frequencies = np.fft.rfftfreq(
        base.ANALYSIS_FFT_SAMPLES, 1.0 / base.SAMPLE_RATE_HZ
    )
    return (frequencies >= base.MINIMUM_ANALYSIS_HZ) & (
        frequencies <= MAXIMUM_DISCOVERY_HZ
    )


def safe_divide(
    numerator: np.ndarray, denominator: np.ndarray, valid: np.ndarray
) -> np.ndarray:
    result = np.zeros_like(numerator, dtype=np.complex128)
    np.divide(numerator, denominator, out=result, where=valid)
    return result


def estimate_noise_aware(trials: list[dict[str, Any]]) -> dict[str, Any]:
    transform = lambda key: np.stack(  # noqa: E731
        [np.fft.rfft(item[key], n=base.ANALYSIS_FFT_SAMPLES) for item in trials]
    )
    x = transform("observed_force")
    y = transform("observed_response")
    nx = transform("force_sensor_noise")
    ny = transform("response_sensor_noise")
    gxx_observed = np.sum(np.abs(x) ** 2, axis=0)
    gyy_observed = np.sum(np.abs(y) ** 2, axis=0)
    gyx_observed = np.sum(y * np.conj(x), axis=0)
    nxx = np.sum(np.abs(nx) ** 2, axis=0)
    nyy = np.sum(np.abs(ny) ** 2, axis=0)
    nyx = np.sum(ny * np.conj(nx), axis=0)
    gxx = np.maximum(gxx_observed - nxx, 0.0)
    gyy = np.maximum(gyy_observed - nyy, 0.0)
    gyx = gyx_observed - nyx

    frequencies = np.fft.rfftfreq(
        base.ANALYSIS_FFT_SAMPLES, 1.0 / base.SAMPLE_RATE_HZ
    )
    reference_band = frequencies <= base.MAXIMUM_ANALYSIS_HZ
    maximum_signal = float(np.max(gxx[reference_band]))
    if maximum_signal <= 0.0 or not math.isfinite(maximum_signal):
        raise OracleError("B1R3 corrected force signal has invalid power")
    input_snr = gxx / np.maximum(nxx, 1.0e-30)
    relative_signal = gxx / maximum_signal
    discovery_valid = (
        analysis_band()
        & (input_snr >= DISCOVERY_INPUT_SNR_FLOOR)
        & (relative_signal >= DISCOVERY_RELATIVE_SIGNAL_POWER_FLOOR)
    )

    covariance = np.empty((len(gxx), 2, 2), dtype=np.complex128)
    covariance[:, 0, 0] = gxx
    covariance[:, 0, 1] = np.conj(gyx)
    covariance[:, 1, 0] = gyx
    covariance[:, 1, 1] = gyy
    eigenvalues, eigenvectors = np.linalg.eigh(covariance)
    direction = eigenvectors[:, :, 1]
    vector_valid = np.abs(direction[:, 0]) > EIGENVECTOR_INPUT_FLOOR
    gtls_valid = discovery_valid & vector_valid
    gtls = safe_divide(direction[:, 1], direction[:, 0], gtls_valid)
    gtls *= base.taper_mask(gtls_valid)

    raw_coherence = np.abs(gyx) ** 2 / np.maximum(gxx * gyy, 1.0e-30)
    corrected_coherence = np.clip(raw_coherence, 0.0, 1.0)
    raw_h1 = safe_divide(
        gyx_observed,
        gxx_observed,
        analysis_band() & (gxx_observed > 0.0),
    )
    corrected_h1 = safe_divide(gyx, gxx, discovery_valid & (gxx > 0.0))
    corrected_h2 = safe_divide(
        gyy,
        np.conj(gyx),
        discovery_valid & (np.abs(gyx) > 1.0e-30),
    )
    raw_h1 *= base.taper_mask(analysis_band())
    corrected_h1 *= base.taper_mask(discovery_valid)
    corrected_h2 *= base.taper_mask(discovery_valid)

    trial_power = np.abs(x) ** 2
    eligible = trial_power >= base.CONDITIONING_FLOOR * np.max(
        trial_power, axis=1
    )[:, None]
    divisions = np.zeros_like(y)
    np.divide(y, x, out=divisions, where=eligible)
    counts = np.sum(eligible, axis=0)
    direct = safe_divide(np.sum(divisions, axis=0), counts, counts > 0)
    direct *= base.taper_mask(analysis_band() & (counts > 0))
    return {
        "gtls": gtls,
        "gtls_valid": gtls_valid,
        "discovery_valid": discovery_valid,
        "input_snr": input_snr,
        "relative_signal": relative_signal,
        "corrected_coherence": corrected_coherence,
        "raw_coherence": raw_coherence,
        "raw_h1": raw_h1,
        "corrected_h1": corrected_h1,
        "corrected_h2": corrected_h2,
        "direct": direct,
        "eigenvalues": eigenvalues,
        "negative_eigenvalue_count": int(np.sum(eigenvalues[:, 0] < 0.0)),
        "corrected": {"gxx": gxx, "gyy": gyy, "gyx": gyx},
    }


def gabor_coefficients(channels: np.ndarray) -> np.ndarray:
    shifted = np.zeros((len(channels), base.MODE_ANALYSIS_SAMPLES), dtype=np.float64)
    retained = base.MODE_ANALYSIS_SAMPLES - base.MODE_ONSET_SAMPLE
    shifted[:, base.MODE_ONSET_SAMPLE :] = channels[:, :retained]
    result = []
    for channel in shifted:
        _, _, coefficients = signal.stft(
            channel,
            fs=base.SAMPLE_RATE_HZ,
            window="blackmanharris",
            nperseg=base.GABOR_WINDOW_SAMPLES,
            noverlap=base.GABOR_WINDOW_SAMPLES - base.GABOR_HOP_SAMPLES,
            nfft=base.GABOR_WINDOW_SAMPLES,
            boundary=None,
            padded=False,
            return_onesided=True,
        )
        result.append(coefficients[:, : base.GABOR_FRAME_COUNT])
    return np.stack(result, axis=2)


def discover_modes(
    coefficients: np.ndarray,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    energy = np.sum(np.abs(coefficients) ** 2, axis=(1, 2))
    bin_hz = base.SAMPLE_RATE_HZ / base.GABOR_WINDOW_SAMPLES
    first = math.ceil(base.MINIMUM_ANALYSIS_HZ / bin_hz)
    last = math.floor(MAXIMUM_DISCOVERY_HZ / bin_hz)
    maximum = max(float(np.max(energy[first : last + 1])), 1.0e-300)
    regions = []
    for index in range(first, last + 1):
        relative_db = 10.0 * math.log10(max(float(energy[index]), 1.0e-300) / maximum)
        if (
            energy[index] > energy[index - 1]
            and energy[index] >= energy[index + 1]
            and relative_db >= base.GABOR_REGION_FLOOR_DB
        ):
            regions.append(
                {
                    "bin": index,
                    "frequency_hz": index * bin_hz,
                    "relative_energy_db": relative_db,
                }
            )
    analysis_bins = sorted(
        {
            index
            for region in regions
            for index in range(
                region["bin"] - base.GABOR_NEIGHBORHOOD_BINS,
                region["bin"] + base.GABOR_NEIGHBORHOOD_BINS + 1,
            )
            if first <= index <= last
        }
    )
    raw = []
    analyses = []
    for index in analysis_bins:
        analysis = base.pole_core.estimate_common_poles(coefficients[index], index * bin_hz)
        accepted = analysis["order_score_margin"] >= base.MINIMUM_ORDER_SCORE_MARGIN
        analyses.append(
            {"bin": index, "center_hz": index * bin_hz, "accepted": accepted, "analysis": analysis}
        )
        if accepted:
            for estimate in analysis["estimates"]:
                if 0.0 < estimate["decay_per_second"] <= 200.0:
                    raw.append(
                        {
                            **estimate,
                            "source_bin": index,
                            "order_score_margin": analysis["order_score_margin"],
                            "source_energy": float(energy[index]),
                        }
                    )
    raw.sort(key=lambda item: item["frequency_hz"])
    groups: list[list[dict[str, Any]]] = []
    for item in raw:
        if not groups or item["frequency_hz"] - groups[-1][-1]["frequency_hz"] > base.DUPLICATE_FREQUENCY_HZ:
            groups.append([item])
        else:
            groups[-1].append(item)
    clusters = []
    for members in groups:
        representative = max(
            members,
            key=lambda item: (
                item["order_score_margin"],
                item["source_energy"],
                -item["source_bin"],
            ),
        )
        clusters.append({"members": members, "representative": representative})
    return clusters, {
        "regions": regions,
        "analysis_bins": analysis_bins,
        "raw_estimate_count": len(raw),
        "analysis": analyses,
    }


def identify_modes(channels: np.ndarray) -> dict[str, Any]:
    coefficients = gabor_coefficients(channels)
    clusters, discovery = discover_modes(coefficients)
    time = np.arange(base.TRANSFER_SAMPLES, dtype=np.float64) / base.SAMPLE_RATE_HZ
    if not clusters:
        return {
            "discovery": discovery,
            "modes": [],
            "reconstruction": np.zeros_like(channels),
        }
    columns = []
    for cluster in clusters:
        estimate = cluster["representative"]
        envelope = np.exp(-estimate["decay_per_second"] * time)
        phase = 2.0 * math.pi * estimate["frequency_hz"] * time
        columns.extend((envelope * np.cos(phase), envelope * np.sin(phase)))
    design = np.stack(columns, axis=1)
    fitted = np.linalg.lstsq(design, channels.T, rcond=None)[0]
    energies = []
    components = []
    for index in range(len(clusters)):
        component = (
            design[:, 2 * index, None] * fitted[2 * index][None, :]
            + design[:, 2 * index + 1, None] * fitted[2 * index + 1][None, :]
        ).T
        components.append(component)
        energies.append(float(np.sum(component * component)))
    maximum = max(energies)
    retained = []
    retained_components = []
    for index, (cluster, energy, component) in enumerate(
        zip(clusters, energies, components, strict=True)
    ):
        relative_db = 10.0 * math.log10(max(energy, 1.0e-300) / maximum)
        if relative_db < base.MODE_ENERGY_FLOOR_DB:
            continue
        representative = cluster["representative"]
        residues = []
        for contact in range(base.CONTACT_COUNT):
            cosine = float(fitted[2 * index, contact])
            sine = float(fitted[2 * index + 1, contact])
            residues.append(
                {"cosine": cosine, "sine": sine, "magnitude": math.hypot(cosine, sine)}
            )
        retained.append(
            {
                "frequency_hz": representative["frequency_hz"],
                "decay_per_second": representative["decay_per_second"],
                "relative_energy_db": relative_db,
                "member_count": len(cluster["members"]),
                "residues": residues,
            }
        )
        retained_components.append(component)
    reconstruction = (
        np.sum(np.stack(retained_components), axis=0)
        if retained_components
        else np.zeros_like(channels)
    )
    return {"discovery": discovery, "modes": retained, "reconstruction": reconstruction}


def neighborhood(frequency_hz: float, decay_per_second: float) -> np.ndarray:
    frequencies = np.fft.rfftfreq(
        base.ANALYSIS_FFT_SAMPLES, 1.0 / base.SAMPLE_RATE_HZ
    )
    radius = max(20.0, 4.0 * decay_per_second / (2.0 * math.pi))
    return np.abs(frequencies - frequency_hz) <= radius


def support_for_frequency(
    frequency_hz: float,
    decay_per_second: float,
    input_snr: np.ndarray,
    coherence: np.ndarray,
    discovery_valid: np.ndarray,
) -> dict[str, Any]:
    local = neighborhood(frequency_hz, decay_per_second)
    radius = max(20.0, 4.0 * decay_per_second / (2.0 * math.pi))
    contacts = []
    for contact in range(base.CONTACT_COUNT):
        median_snr = float(np.median(input_snr[contact, local]))
        median_coherence = float(np.median(coherence[contact, local]))
        mask_fraction = float(np.mean(discovery_valid[contact, local]))
        source_supported = (
            median_snr >= LOCAL_INPUT_SNR_FLOOR
            and mask_fraction >= LOCAL_DISCOVERY_MASK_FRACTION
        )
        contacts.append(
            {
                "contact": contact,
                "median_input_snr": median_snr,
                "discovery_mask_fraction": mask_fraction,
                "median_corrected_coherence": median_coherence,
                "supports_source": source_supported,
                "supports_pole": source_supported
                and median_coherence >= MODE_SUPPORT_COHERENCE,
                "residue_confident": source_supported
                and median_coherence >= RESIDUE_CONFIDENCE_COHERENCE,
                "residue_uncertainty": 1.0 - median_coherence,
            }
        )
    return {
        "frequency_hz": frequency_hz,
        "decay_per_second": decay_per_second,
        "neighborhood_radius_hz": radius,
        "source_supporting_contact_count": sum(item["supports_source"] for item in contacts),
        "supporting_contact_count": sum(item["supports_pole"] for item in contacts),
        "confident_residue_count": sum(item["residue_confident"] for item in contacts),
        "contacts": contacts,
    }


def modal_support(
    modes: list[dict[str, Any]],
    input_snr: np.ndarray,
    coherence: np.ndarray,
    discovery_valid: np.ndarray,
) -> list[dict[str, Any]]:
    reports = []
    for mode_index, mode in enumerate(modes):
        report = support_for_frequency(
            mode["frequency_hz"],
            mode["decay_per_second"],
            input_snr,
            coherence,
            discovery_valid,
        )
        report["mode_index"] = mode_index
        report["retained"] = (
            report["supporting_contact_count"] >= MINIMUM_SUPPORTING_CONTACTS
            and report["confident_residue_count"] >= MINIMUM_CONFIDENT_RESIDUES
        )
        reports.append(report)
    return reports


def truth_support(
    input_snr: np.ndarray,
    coherence: np.ndarray,
    discovery_valid: np.ndarray,
) -> dict[str, Any]:
    reports = [
        {
            "truth_index": index,
            **support_for_frequency(
                float(frequency),
                float(TRUTH_DECAYS_PER_SECOND[index]),
                input_snr,
                coherence,
                discovery_valid,
            ),
        }
        for index, frequency in enumerate(TRUTH_FREQUENCIES_HZ)
    ]
    supported = [
        item["truth_index"]
        for item in reports
        if item["supporting_contact_count"] >= MINIMUM_SUPPORTING_CONTACTS
    ]
    unsupported = [index for index in range(len(reports)) if index not in supported]
    return {
        "supported_truth_indices": supported,
        "unsupported_truth_indices": unsupported,
        "supported_truth_mode_count": len(supported),
        "unsupported_truth_mode_count": len(unsupported),
        "modes": reports,
    }


def reconstruct_modes(modes: list[dict[str, Any]]) -> np.ndarray:
    time = np.arange(base.TRANSFER_SAMPLES, dtype=np.float64) / base.SAMPLE_RATE_HZ
    reconstruction = np.zeros((base.CONTACT_COUNT, base.TRANSFER_SAMPLES), dtype=np.float64)
    for mode in modes:
        envelope = np.exp(-mode["decay_per_second"] * time)
        phase = 2.0 * math.pi * mode["frequency_hz"] * time
        cosine = envelope * np.cos(phase)
        sine = envelope * np.sin(phase)
        for contact, residue in enumerate(mode["residues"]):
            reconstruction[contact] += (
                residue["cosine"] * cosine + residue["sine"] * sine
            )
    return reconstruction


def match_truth(modes: list[dict[str, Any]]) -> dict[str, Any]:
    pairs = []
    for candidate_index, mode in enumerate(modes):
        for truth_index, frequency in enumerate(TRUTH_FREQUENCIES_HZ):
            error = abs(mode["frequency_hz"] - frequency)
            if error <= base.MODE_MATCH_RADIUS_HZ:
                pairs.append((error, candidate_index, truth_index))
    pairs.sort()
    assignments: list[int | None] = [None] * len(modes)
    used = [False] * len(TRUTH_FREQUENCIES_HZ)
    for _, candidate_index, truth_index in pairs:
        if assignments[candidate_index] is None and not used[truth_index]:
            assignments[candidate_index] = truth_index
            used[truth_index] = True
    comparisons = []
    for mode, assignment in zip(modes, assignments, strict=True):
        if assignment is None:
            continue
        expected_decay = float(TRUTH_DECAYS_PER_SECOND[assignment])
        comparisons.append(
            {
                "truth_index": assignment,
                "expected_frequency_hz": float(TRUTH_FREQUENCIES_HZ[assignment]),
                "observed_frequency_hz": mode["frequency_hz"],
                "frequency_error_hz": abs(
                    mode["frequency_hz"] - float(TRUTH_FREQUENCIES_HZ[assignment])
                ),
                "expected_decay_per_second": expected_decay,
                "observed_decay_per_second": mode["decay_per_second"],
                "decay_error_per_second": abs(
                    mode["decay_per_second"] - expected_decay
                ),
                "relative_decay_error": abs(
                    mode["decay_per_second"] - expected_decay
                )
                / expected_decay,
            }
        )
    matched = sorted(item["truth_index"] for item in comparisons)
    return {
        "truth_count": len(TRUTH_FREQUENCIES_HZ),
        "candidate_count": len(modes),
        "truth_match_count": len(comparisons),
        "false_positive_count": len(modes) - len(comparisons),
        "comparisons": comparisons,
        "matched_truth_indices": matched,
        "unmatched_truth_indices": [
        index for index in range(len(TRUTH_FREQUENCIES_HZ)) if index not in matched
        ],
    }


def fit_estimators(
    fit_role: dict[str, Any], profiles: tuple[tuple[str, tuple[float, ...]], ...]
) -> dict[str, Any]:
    estimates = []
    raw_outputs = []
    impulses = []
    shortest_index = min(
        range(len(profiles)),
        key=lambda index: (profiles[index][1][0] if profiles[index][1] else 15.0),
    )
    analysis_weights = base.taper_mask(analysis_band())
    for contact in range(base.CONTACT_COUNT):
        trials = [item for item in fit_role["trials"] if item["contact"] == contact]
        if len(trials) != len(profiles) * fit_role["repeats"]:
            raise OracleError("B1R3 fit contact trial count changed")
        estimates.append(estimate_noise_aware(trials))
        shortest_trials = [item for item in trials if item["profile_index"] == shortest_index]
        impulses.append(
            np.fft.rfft(
                np.mean(
                    np.stack([item["observed_response"] for item in shortest_trials]),
                    axis=0,
                ),
                n=base.ANALYSIS_FFT_SAMPLES,
            )
            * analysis_weights
        )
        raw_outputs.append(
            np.mean(np.stack([item["observed_response"] for item in trials]), axis=0)
        )
    stacked = {
        name: np.stack([item[name] for item in estimates])
        for name in (
            "gtls",
            "gtls_valid",
            "discovery_valid",
            "input_snr",
            "relative_signal",
            "corrected_coherence",
            "raw_h1",
            "corrected_h1",
            "corrected_h2",
            "direct",
            "eigenvalues",
        )
    }
    gtls_impulses = np.fft.irfft(
        stacked["gtls"], n=base.ANALYSIS_FFT_SAMPLES, axis=1
    )[:, : base.TRANSFER_SAMPLES]
    discovered = identify_modes(gtls_impulses)
    support = modal_support(
        discovered["modes"],
        stacked["input_snr"],
        stacked["corrected_coherence"],
        stacked["discovery_valid"],
    )
    retained_modes = [
        mode
        for mode, report in zip(discovered["modes"], support, strict=True)
        if report["retained"]
    ]
    retained_support = [item for item in support if item["retained"]]
    modal_reconstruction = reconstruct_modes(retained_modes)
    modal = np.fft.rfft(modal_reconstruction, n=base.ANALYSIS_FFT_SAMPLES, axis=1)
    raw_modes = identify_modes(np.stack(raw_outputs))
    raw_modal = np.fft.rfft(
        raw_modes["reconstruction"], n=base.ANALYSIS_FFT_SAMPLES, axis=1
    ) * analysis_weights[None, :]
    return {
        **stacked,
        "candidate": modal,
        "impulse": np.stack(impulses),
        "raw_modal": raw_modal,
        "modes": {
            "discovery": discovered["discovery"],
            "modes": retained_modes,
            "reconstruction": modal_reconstruction,
            "initial_candidate_count": len(discovered["modes"]),
        },
        "support": support,
        "retained_support": retained_support,
        "truth_support": truth_support(
            stacked["input_snr"],
            stacked["corrected_coherence"],
            stacked["discovery_valid"],
        ),
        "negative_eigenvalue_count": sum(
            item["negative_eigenvalue_count"] for item in estimates
        ),
    }


def evaluate_held(role: dict[str, Any], estimators: dict[str, Any]) -> dict[str, Any]:
    names = (
        "candidate",
        "gtls",
        "raw_h1",
        "corrected_h1",
        "corrected_h2",
        "direct",
        "impulse",
        "raw_modal",
    )
    trials = []
    for item in role["trials"]:
        contact = item["contact"]
        force_spectrum = np.fft.rfft(item["exact_force"], n=base.ANALYSIS_FFT_SAMPLES)
        predictions = {
            name: np.fft.irfft(
                force_spectrum * estimators[name][contact],
                n=base.ANALYSIS_FFT_SAMPLES,
            )[: base.TRANSFER_SAMPLES]
            for name in names
        }
        trials.append(
            {
                "contact": contact,
                "profile_index": item["profile_index"],
                "nrmse": {
                    name: base.normalized_rms_error(item["clean_response"], predictions[name])
                    for name in names
                },
                "candidate_log_spectrum_rmse_db": base.log_spectrum_rmse_db(
                    item["clean_response"], predictions["candidate"]
                ),
            }
        )
    mean_nrmse = {
        name: float(np.mean([item["nrmse"][name] for item in trials])) for name in names
    }
    candidate_values = [item["nrmse"]["candidate"] for item in trials]
    truth = match_truth(estimators["modes"]["modes"])
    source_truth = estimators["truth_support"]
    matched_and_supported = sorted(
        set(truth["matched_truth_indices"]) & set(source_truth["supported_truth_indices"])
    )
    unsupported = [
        index for index in range(len(TRUTH_FREQUENCIES_HZ)) if index not in matched_and_supported
    ]
    frequency_errors = [item["frequency_error_hz"] for item in truth["comparisons"]]
    decay_errors = [item["decay_error_per_second"] for item in truth["comparisons"]]
    relative_errors = [item["relative_decay_error"] for item in truth["comparisons"]]
    supporting = [item["supporting_contact_count"] for item in estimators["retained_support"]]
    confident = [item["confident_residue_count"] for item in estimators["retained_support"]]
    best_corrected = min(mean_nrmse["corrected_h1"], mean_nrmse["corrected_h2"])
    broad_coverages = [float(np.mean(mask[analysis_band()])) for mask in estimators["discovery_valid"]]
    return {
        "role": role["role"],
        "trial_count": len(trials),
        "trials": trials,
        "mean_nrmse": mean_nrmse,
        "maximum_candidate_nrmse": max(candidate_values),
        "mean_candidate_log_spectrum_rmse_db": float(
            np.mean([item["candidate_log_spectrum_rmse_db"] for item in trials])
        ),
        "candidate_over_impulse_nrmse": mean_nrmse["candidate"] / mean_nrmse["impulse"],
        "candidate_over_raw_modal_nrmse": mean_nrmse["candidate"] / mean_nrmse["raw_modal"],
        "candidate_over_raw_h1_nrmse": mean_nrmse["candidate"] / mean_nrmse["raw_h1"],
        "gtls_over_best_corrected_nrmse": mean_nrmse["gtls"] / best_corrected,
        "broad_discovery_coverage_by_contact": broad_coverages,
        "minimum_broad_discovery_coverage": min(broad_coverages),
        "truth": truth,
        "truth_source_support": source_truth,
        "supported_truth_indices": matched_and_supported,
        "unsupported_truth_indices": unsupported,
        "supported_truth_mode_count": len(matched_and_supported),
        "unsupported_truth_mode_count": len(unsupported),
        "maximum_frequency_error_hz": max(frequency_errors, default=1.0e300),
        "maximum_decay_error_per_second": max(decay_errors, default=1.0e300),
        "maximum_relative_decay_error": max(relative_errors, default=1.0e300),
        "minimum_supporting_contacts": min(supporting, default=0),
        "minimum_confident_residues": min(confident, default=0),
        "all_residues_nonzero": all(
            all(residue["magnitude"] > 0.0 for residue in mode["residues"])
            for mode in estimators["modes"]["modes"]
        ),
    }


def control_residuals(role: dict[str, Any], estimators: dict[str, Any]) -> list[float]:
    residuals = []
    for item in role["trials"]:
        contact = item["contact"]
        prediction = np.fft.irfft(
            np.fft.rfft(item["observed_force"], n=base.ANALYSIS_FFT_SAMPLES)
            * estimators["candidate"][contact],
            n=base.ANALYSIS_FFT_SAMPLES,
        )[: base.TRANSFER_SAMPLES]
        residuals.append(base.normalized_rms_error(item["observed_response"], prediction))
    return residuals


def evaluate_domain_control(
    kind: str, offset: int, transfers: np.ndarray
) -> dict[str, Any]:
    if kind == "partial_support":
        profiles = (COMB_NOTCH_PROFILE,)
        interference = 0.0
    elif kind == "weak_excitation":
        profiles = (WEAK_PROFILE,)
        interference = 0.0
    elif kind == "low_coherence":
        profiles = (LOW_COHERENCE_PROFILE,)
        interference = 0.75
    else:
        raise OracleError(f"unknown B1R3 domain control: {kind}")
    role = generate_role(
        f"control_{kind}",
        profiles,
        transfers,
        FIT_REPEATS,
        seeds=seed_triplet(offset),
        interference_rms_ratio=interference,
    )
    estimators = fit_estimators(role, profiles)
    truth = match_truth(estimators["modes"]["modes"])
    source = estimators["truth_support"]
    supported = sorted(
        set(truth["matched_truth_indices"]) & set(source["supported_truth_indices"])
    )
    unsupported = [
        index for index in range(len(TRUTH_FREQUENCIES_HZ)) if index not in supported
    ]
    residuals = control_residuals(role, estimators)
    median_residual = float(np.median(residuals))
    if kind == "partial_support" and supported == [0, 1, 2, 3, 4, 5] and unsupported == [6]:
        decision = "OOD_UNSUPPORTED_MODAL_BAND"
    elif kind == "weak_excitation" and unsupported:
        decision = "OOD_UNSUPPORTED_MODAL_BAND"
    elif kind == "low_coherence" and median_residual >= GATES["maximum_low_coherence_residual_nrmse"]:
        decision = "OOD_LOW_COHERENCE"
    else:
        decision = "INVALIDLY_ADMITTED"
    return {
        "kind": kind,
        "trial_count": len(role["trials"]),
        "supported_truth_indices": supported,
        "unsupported_truth_indices": unsupported,
        "supported_truth_mode_count": len(supported),
        "unsupported_truth_mode_count": len(unsupported),
        "retained_diagnostic_mode_count": len(estimators["modes"]["modes"]),
        "false_positive_count": truth["false_positive_count"],
        "median_observed_response_reconstruction_nrmse": median_residual,
        "broad_discovery_coverage_by_contact": [
            float(np.mean(mask[analysis_band()])) for mask in estimators["discovery_valid"]
        ],
        "truth_source_support": source,
        "decision": decision,
        "complete_domain_admitted_mode_count": 7 if decision == "PASS" else 0,
    }


def evaluate_missing_second_impact(offset: int, transfer: np.ndarray) -> dict[str, Any]:
    count = 4
    seeds = seed_triplet(offset)
    force_rngs = base.child_generators(seeds["force"], count)
    response_rngs = base.child_generators(seeds["response"], count)
    room_rngs = base.child_generators(seeds["room"], count)
    transfer_spectrum = np.fft.rfft(transfer, n=base.ANALYSIS_FFT_SAMPLES)
    measured = force_profile(("half_sine", (23.0,)))
    active = measured[:23]
    delays = (389, 617, 941, 1207)
    trials = []
    for index in range(count):
        true_force = measured.copy()
        true_force[delays[index] : delays[index] + len(active)] += 0.8 * active
        clean_response = base.fft_convolve(true_force, transfer_spectrum)
        response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
        force_sensor_noise = force_rngs[index].normal(
            0.0,
            base.FORCE_NOISE_PEAK_RATIO * float(np.max(measured)),
            base.TRANSFER_SAMPLES,
        )
        response_sensor_noise = response_rngs[index].normal(
            0.0,
            base.RESPONSE_NOISE_RMS_RATIO * response_rms,
            base.TRANSFER_SAMPLES,
        )
        room_disturbance = base.room_tail(
            room_rngs[index], base.ROOM_TAIL_RMS_RATIO * response_rms
        )
        trials.append(
            {
                "contact": 0,
                "profile_index": index,
                "repeat": 0,
                "exact_force": true_force,
                "force_sensor_noise": force_sensor_noise,
                "observed_force": measured + force_sensor_noise,
                "clean_response": clean_response,
                "response_sensor_noise": response_sensor_noise,
                "room_disturbance": room_disturbance,
                "observed_response": clean_response + response_sensor_noise + room_disturbance,
            }
        )
    estimate = estimate_noise_aware(trials)
    residuals = []
    for item in trials:
        prediction = np.fft.irfft(
            np.fft.rfft(item["observed_force"], n=base.ANALYSIS_FFT_SAMPLES)
            * estimate["gtls"],
            n=base.ANALYSIS_FFT_SAMPLES,
        )[: base.TRANSFER_SAMPLES]
        residuals.append(base.normalized_rms_error(item["observed_response"], prediction))
    median_residual = float(np.median(residuals))
    decision = (
        "OOD_MODEL_MISMATCH"
        if median_residual >= GATES["minimum_double_impact_residual_nrmse"]
        else "INVALIDLY_ADMITTED"
    )
    return {
        "kind": "missing_second_impact",
        "trial_count": len(trials),
        "median_observed_response_reconstruction_nrmse": median_residual,
        "decision": decision,
        "complete_domain_admitted_mode_count": 0 if decision != "INVALIDLY_ADMITTED" else None,
    }


def evaluation_gates(prefix: str, value: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        base.check(f"{prefix}.supported_truth_mode_count", value["supported_truth_mode_count"], "==", GATES["supported_truth_mode_count"]),
        base.check(f"{prefix}.unsupported_truth_mode_count", value["unsupported_truth_mode_count"], "==", GATES["unsupported_truth_mode_count"]),
        base.check(f"{prefix}.false_positive_count", value["truth"]["false_positive_count"], "==", GATES["false_positive_count"]),
        base.check(f"{prefix}.minimum_supporting_contacts", value["minimum_supporting_contacts"], ">=", GATES["minimum_supporting_contacts"]),
        base.check(f"{prefix}.minimum_confident_residues", value["minimum_confident_residues"], ">=", GATES["minimum_confident_residues"]),
        base.check(f"{prefix}.maximum_frequency_error_hz", value["maximum_frequency_error_hz"], "<=", GATES["maximum_frequency_error_hz"]),
        base.check(f"{prefix}.maximum_decay_error_per_second", value["maximum_decay_error_per_second"], "<=", GATES["maximum_decay_error_per_second"]),
        base.check(f"{prefix}.maximum_relative_decay_error", value["maximum_relative_decay_error"], "<=", GATES["maximum_relative_decay_error"]),
        base.check(f"{prefix}.mean_held_nrmse", value["mean_nrmse"]["candidate"], "<=", GATES["maximum_mean_held_nrmse"]),
        base.check(f"{prefix}.maximum_held_nrmse", value["maximum_candidate_nrmse"], "<=", GATES["maximum_held_nrmse"]),
        base.check(f"{prefix}.mean_log_spectrum_rmse_db", value["mean_candidate_log_spectrum_rmse_db"], "<=", GATES["maximum_mean_log_spectrum_rmse_db"]),
        base.check(f"{prefix}.impulse_nrmse_ratio", value["candidate_over_impulse_nrmse"], "<=", GATES["maximum_impulse_nrmse_ratio"]),
        base.check(f"{prefix}.raw_modal_nrmse_ratio", value["candidate_over_raw_modal_nrmse"], "<=", GATES["maximum_raw_modal_nrmse_ratio"]),
        base.check(f"{prefix}.raw_h1_nrmse_ratio", value["candidate_over_raw_h1_nrmse"], "<=", GATES["maximum_raw_h1_nrmse_ratio"]),
        base.check(f"{prefix}.gtls_best_corrected_ratio", value["gtls_over_best_corrected_nrmse"], "<=", GATES["maximum_gtls_over_best_corrected_ratio"]),
        base.check(f"{prefix}.all_residues_nonzero", value["all_residues_nonzero"], "==", True),
    ]


def control_gates(prefix: str, controls: dict[str, Any]) -> list[dict[str, Any]]:
    partial = controls["partial_support"]
    weak = controls["weak_excitation"]
    low = controls["low_coherence"]
    return [
        base.check(f"{prefix}.partial.supported_truth_mode_count", partial["supported_truth_mode_count"], "==", GATES["comb_supported_truth_mode_count"]),
        base.check(f"{prefix}.partial.unsupported_truth_indices", partial["unsupported_truth_indices"], "==", GATES["comb_unsupported_truth_indices"]),
        base.check(f"{prefix}.partial.false_positive_count", partial["false_positive_count"], "==", 0),
        base.check(f"{prefix}.partial.decision", partial["decision"], "==", "OOD_UNSUPPORTED_MODAL_BAND"),
        base.check(f"{prefix}.partial.admitted_modes", partial["complete_domain_admitted_mode_count"], "==", 0),
        base.check(f"{prefix}.weak.decision", weak["decision"], "==", "OOD_UNSUPPORTED_MODAL_BAND"),
        base.check(f"{prefix}.weak.admitted_modes", weak["complete_domain_admitted_mode_count"], "==", 0),
        base.check(f"{prefix}.low_coherence.decision", low["decision"], "==", "OOD_LOW_COHERENCE"),
        base.check(f"{prefix}.low_coherence.admitted_modes", low["complete_domain_admitted_mode_count"], "==", 0),
    ]


def common_report(payload: bytes, manifest: dict[str, Any]) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": base.sha256_bytes(payload),
        "implementation_sha256": manifest["implementation_sha256"],
        "runtime": manifest["runtime"],
        "network_requests": 0,
        "real_payload_bytes_read": 0,
        "parent_holdout_samples_read": 0,
        "product_credit": {
            "real_quality": False,
            "validator": False,
            "atlas": False,
            "runtime": False,
        },
    }


def freeze(root: Path, output: Path) -> None:
    base.write_atomic(output / "manifest.json", base.canonical_json(expected_manifest(root)))


def preflight(payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        **common_report(payload, manifest),
        "decision": "B1R3_LOCAL_MODAL_SUPPORT_ORACLE_FROZEN",
        "fixture_samples_generated": 0,
        "sensor_noise_samples_generated": 0,
        "unmeasured_disturbance_samples_generated": 0,
        "fit_trials_evaluated": 0,
        "development_trials_evaluated": 0,
        "holdout_trials_evaluated": 0,
        "gate": {"passed": True, "checks": []},
    }
    base.write_atomic(output / "report.json", base.canonical_json(report))


def run_oracle(payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    transfers = exact_transfers()
    fit_role = generate_role("fit", FIT_PROFILES, transfers, FIT_REPEATS)
    estimators = fit_estimators(fit_role, FIT_PROFILES)
    development_role = generate_role("development", DEVELOPMENT_PROFILES, transfers, 1)
    development = evaluate_held(development_role, estimators)
    identity = base.evaluate_identity(transfers)
    development_controls = {
        "partial_support": evaluate_domain_control("partial_support", 100, transfers),
        "weak_excitation": evaluate_domain_control("weak_excitation", 200, transfers),
        "low_coherence": evaluate_domain_control("low_coherence", 300, transfers),
    }
    checks = evaluation_gates("development", development)
    checks.extend(control_gates("development", development_controls))
    checks.extend(
        [
            base.check("identity.nrmse", identity["nrmse"], "<=", GATES["maximum_identity_nrmse"]),
            base.check("identity.finite", identity["finite"], "==", True),
        ]
    )
    development_passed = all(item["passed"] for item in checks)
    holdout = None
    holdout_controls = None
    if development_passed:
        holdout_role = generate_role("holdout", HOLDOUT_PROFILES, transfers, 1)
        holdout = evaluate_held(holdout_role, estimators)
        holdout_controls = {
            "partial_support": evaluate_domain_control("partial_support", 400, transfers),
            "weak_excitation": evaluate_domain_control("weak_excitation", 500, transfers),
            "low_coherence": evaluate_domain_control("low_coherence", 600, transfers),
            "missing_second_impact": evaluate_missing_second_impact(700, transfers[0]),
        }
        checks.extend(evaluation_gates("holdout", holdout))
        checks.extend(control_gates("holdout", holdout_controls))
        checks.extend(
            [
                base.check("holdout.double_impact_decision", holdout_controls["missing_second_impact"]["decision"], "==", "OOD_MODEL_MISMATCH"),
                base.check("holdout.double_impact_admitted_modes", holdout_controls["missing_second_impact"]["complete_domain_admitted_mode_count"], "==", 0),
            ]
        )

    arrays = {
        "modal_transfer.npy": estimators["candidate"].astype("<c16"),
        "gtls_transfer.npy": estimators["gtls"].astype("<c16"),
        "input_snr.npy": estimators["input_snr"].astype("<f8"),
        "corrected_coherence.npy": estimators["corrected_coherence"].astype("<f8"),
        "discovery_valid_mask.npy": estimators["discovery_valid"].astype(np.bool_),
        "modal_reconstruction.npy": estimators["modes"]["reconstruction"].astype("<f8"),
    }
    array_hashes = {name: base.write_npy(output / name, value) for name, value in arrays.items()}
    model = {
        "schema": MODEL_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": base.sha256_bytes(payload),
        "array_sha256": array_hashes,
        "modes": estimators["modes"]["modes"],
        "modal_support": estimators["retained_support"],
        "truth_source_support": estimators["truth_support"],
        "negative_corrected_covariance_eigenvalue_count": estimators["negative_eigenvalue_count"],
        "candidate": manifest["candidate"],
        "product_credit": False,
    }
    if not base.finite_tree(model):
        raise OracleError("B1R3 model contains nonfinite values")
    model_payload = base.canonical_json(model)
    base.write_atomic(output / "model.json", model_payload)

    passed = all(item["passed"] for item in checks)
    report = {
        "schema": REPORT_SCHEMAS["run"],
        **common_report(payload, manifest),
        "decision": "PASS_KNOWN_TRUTH_FRF" if passed else "REJECT_LOCAL_MODAL_SUPPORT",
        "role_access_order": ["sensor_calibration_and_fit", "development"]
        + (["holdout"] if holdout is not None else []),
        "sample_accounting": {
            "fixture_transfer_samples": int(transfers.size),
            "fit_trials": len(fit_role["trials"]),
            "fit_force_sensor_noise_samples": len(fit_role["trials"]) * base.TRANSFER_SAMPLES,
            "fit_response_sensor_noise_samples": len(fit_role["trials"]) * base.TRANSFER_SAMPLES,
            "fit_unmeasured_room_samples": len(fit_role["trials"]) * base.TRANSFER_SAMPLES,
            "development_trials": len(development_role["trials"]),
            "development_control_trials": sum(item["trial_count"] for item in development_controls.values()),
            "holdout_trials": 0 if holdout is None else base.CONTACT_COUNT * len(HOLDOUT_PROFILES),
            "holdout_control_trials": 0 if holdout_controls is None else sum(item["trial_count"] for item in holdout_controls.values()),
            "real_samples": 0,
            "parent_holdout_samples": 0,
        },
        "identity": identity,
        "mode_discovery": {
            "region_count": len(estimators["modes"]["discovery"]["regions"]),
            "analysis_bin_count": len(estimators["modes"]["discovery"]["analysis_bins"]),
            "raw_estimate_count": estimators["modes"]["discovery"]["raw_estimate_count"],
            "initial_candidate_count": estimators["modes"]["initial_candidate_count"],
            "retained_mode_count": len(estimators["modes"]["modes"]),
            "regions": estimators["modes"]["discovery"]["regions"],
        },
        "development": development,
        "development_controls": development_controls,
        "holdout": holdout,
        "holdout_controls": holdout_controls,
        "model_sha256": base.sha256_bytes(model_payload),
        "array_sha256": array_hashes,
        "gate": {"passed": passed, "checks": checks},
        "next_authorized_step": (
            "B2_ZERO_DECODE_SOURCE_FEASIBILITY"
            if passed
            else "NEW_ESTIMATOR_HYPOTHESIS_ONLY"
        ),
    }
    if not base.finite_tree(report):
        raise OracleError("B1R3 report contains nonfinite values")
    base.write_atomic(output / "report.json", base.canonical_json(report))


def main() -> int:
    arguments = parse_arguments()
    root = base.repository_root()
    output = base.prepare_output(root, arguments.output)
    try:
        if arguments.stage == "freeze":
            if arguments.manifest is not None:
                raise OracleError("freeze stage does not accept --manifest")
            freeze(root, output)
            return 0
        if arguments.manifest is None:
            raise OracleError(f"{arguments.stage} stage requires --manifest")
        payload, manifest = load_manifest(root, arguments.manifest)
        if arguments.stage == "preflight":
            preflight(payload, manifest, output)
        else:
            run_oracle(payload, manifest, output)
        return 0
    except Exception:
        shutil.rmtree(output, ignore_errors=True)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
