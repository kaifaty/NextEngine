#!/usr/bin/env python3
"""Run the preregistered V11-B1R2 noise-aware GTLS modal oracle."""

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


STUDY_ID = "physical-sound-contact-field-r3a-v11-b1r2-noise-aware-gtls"
REVISION = "noise-calibrated-gtls-common-modal-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v11-b1r2.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-r3a-v11-b1r2-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-r3a-v11-b1r2.report.v1",
}
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v11-b1r2.model.v1"

PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-r3a-v11-b1r2-noise-aware-gtls-protocol-2026-08-31.md"
)
PARENT_RUNNER_PATH = (
    "lab/scripts/"
    "physical_sound_contact_field_r3a_v11_b1_force_response_oracle.py"
)
PARENT_RESEARCH_PATH = (
    "docs/development/"
    "physical-sound-r3a-v11-b1r-precheck-rejection-and-frf-noise-research-"
    "2026-08-31.md"
)
POLE_CORE_PATH = "lab/scripts/physical_sound_subband_common_pole_control.py"

TRUTH_FREQUENCIES_HZ = np.asarray(
    [557.0, 983.0, 1_597.0, 2_539.0, 4_051.0, 6_451.0, 9_769.0],
    dtype=np.float64,
)
TRUTH_DECAYS_PER_SECOND = np.asarray(
    [4.5, 7.5, 10.5, 15.0, 22.0, 32.0, 47.0], dtype=np.float64
)
FIT_REPEATS = 4
ROLE_SEEDS = {
    "fit": {"force": 2_026_112_201, "response": 2_026_112_202, "room": 2_026_112_203},
    "development": {
        "force": 2_026_112_301,
        "response": 2_026_112_302,
        "room": 2_026_112_303,
    },
    "holdout": {
        "force": 2_026_112_401,
        "response": 2_026_112_402,
        "room": 2_026_112_403,
    },
    "corruption": {
        "force": 2_026_112_501,
        "response": 2_026_112_502,
        "room": 2_026_112_503,
    },
}

INPUT_SNR_FLOOR = 100.0
RELATIVE_SIGNAL_POWER_FLOOR = 1.0e-5
MODE_SUPPORT_COHERENCE = 0.90
RESIDUE_CONFIDENCE_COHERENCE = 0.85
MINIMUM_SUPPORTING_CONTACTS = 3
MINIMUM_CONFIDENT_RESIDUES = 3
EIGENVECTOR_INPUT_FLOOR = 1.0e-12

GATES = {
    "minimum_force_coverage": 0.98,
    "truth_match_count": 7,
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
    "maximum_weak_high_band_coverage": 0.35,
    "maximum_low_coherence_median": 0.80,
    "maximum_low_coherence_coverage": 0.50,
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
        "parent_research": (root / PARENT_RESEARCH_PATH).resolve(strict=True),
        "common_pole_core": (root / POLE_CORE_PATH).resolve(strict=True),
    }
    return {name: base.sha256_file(path) for name, path in paths.items()}


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
                "(0.31 + 0.045*((7*contact + 9*mode + 2) mod 13))/(1 + 0.03*mode)"
            ),
            "residue_phase_rule": (
                "0.113 + 0.27*contact + 0.21*mode + 0.031*contact*mode radians"
            ),
        },
        "roles": {
            "fit": {
                "profiles": [
                    base.profile_descriptor(value) for value in base.FIT_PROFILES
                ],
                "repeats": FIT_REPEATS,
                "trial_count": base.CONTACT_COUNT
                * len(base.FIT_PROFILES)
                * FIT_REPEATS,
            },
            "development": [
                base.profile_descriptor(value) for value in base.DEVELOPMENT_PROFILES
            ],
            "holdout": [
                base.profile_descriptor(value) for value in base.HOLDOUT_PROFILES
            ],
            "seeds": ROLE_SEEDS,
            "order": ["noise_calibration_and_fit", "development", "holdout"],
        },
        "noise_calibration": {
            "force_noise_peak_ratio": base.FORCE_NOISE_PEAK_RATIO,
            "response_noise_rms_ratio": base.RESPONSE_NOISE_RMS_RATIO,
            "room_tail_rms_ratio": base.ROOM_TAIL_RMS_RATIO,
            "calibration": "exact separately retained noise components for each fit trial",
        },
        "candidate": {
            "estimator": "noise_corrected_dominant_covariance_GTLS",
            "corrected_covariance": "[[Gxx,conj(Gyx)],[Gyx,Gyy]]",
            "dominant_direction": "H=v_y/v_x",
            "input_snr_floor": INPUT_SNR_FLOOR,
            "relative_signal_power_floor": RELATIVE_SIGNAL_POWER_FLOOR,
            "relative_signal_reference": "maximum corrected Gxx over 0..12000 Hz",
            "mode_support_coherence": MODE_SUPPORT_COHERENCE,
            "residue_confidence_coherence": RESIDUE_CONFIDENCE_COHERENCE,
            "minimum_supporting_contacts": MINIMUM_SUPPORTING_CONTACTS,
            "minimum_confident_residues": MINIMUM_CONFIDENT_RESIDUES,
            "selected_transfer": "explicit common-modal reconstruction only",
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
            "weak_excitation": base.profile_descriptor(("hann", (1025.0,))),
            "low_coherence_interference_rms_ratio": 0.75,
            "missing_second_impact": {
                "profile": base.profile_descriptor(("half_sine", (19.0,))),
                "gain": 0.8,
                "delays_samples": [401, 613, 887, 1151],
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
            "any valid gate failure rejects this noise-aware revision"
        ),
    }


def load_manifest(root: Path, path: Path) -> tuple[bytes, dict[str, Any]]:
    resolved = base.require_external_file(root, path, "B1R2 manifest")
    payload = resolved.read_bytes()
    if len(payload) > 256 * 1024:
        raise OracleError("B1R2 manifest exceeds 256 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleError("cannot parse B1R2 manifest") from error
    if payload != base.canonical_json(value):
        raise OracleError("B1R2 manifest is not canonical JSON")
    if value != expected_manifest(root):
        raise OracleError("B1R2 manifest or implementation lineage changed")
    return payload, value


def exact_transfers() -> np.ndarray:
    time = np.arange(base.TRANSFER_SAMPLES, dtype=np.float64) / base.SAMPLE_RATE_HZ
    transfers = np.zeros(
        (base.CONTACT_COUNT, base.TRANSFER_SAMPLES), dtype=np.float64
    )
    for contact in range(base.CONTACT_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
        ):
            gain = (0.31 + 0.045 * ((7 * contact + 9 * mode + 2) % 13)) / (
                1.0 + 0.03 * mode
            )
            phase = 0.113 + 0.27 * contact + 0.21 * mode + 0.031 * contact * mode
            transfers[contact] += (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
    peak = float(np.max(np.abs(transfers)))
    if not math.isfinite(peak) or peak <= 0.0:
        raise OracleError("B1R2 exact transfer has invalid peak")
    transfers *= base.TRANSFER_PEAK / peak
    return transfers


def generate_role(
    role: str,
    profiles: tuple[tuple[str, tuple[float, ...]], ...],
    transfers: np.ndarray,
    repeats: int,
) -> dict[str, Any]:
    count = base.CONTACT_COUNT * len(profiles) * repeats
    seeds = ROLE_SEEDS[role]
    force_rngs = base.child_generators(seeds["force"], count)
    response_rngs = base.child_generators(seeds["response"], count)
    room_rngs = base.child_generators(seeds["room"], count)
    spectra = np.fft.rfft(transfers, n=base.ANALYSIS_FFT_SAMPLES, axis=1)
    trials = []
    for contact in range(base.CONTACT_COUNT):
        for profile_index, profile in enumerate(profiles):
            for repeat in range(repeats):
                index = (contact * len(profiles) + profile_index) * repeats + repeat
                exact_force = base.force_profile(profile)
                clean_response = base.fft_convolve(exact_force, spectra[contact])
                response_rms = math.sqrt(
                    float(np.mean(clean_response * clean_response))
                )
                if response_rms <= 0.0 or not math.isfinite(response_rms):
                    raise OracleError("B1R2 clean response has invalid RMS")
                force_noise = force_rngs[index].normal(
                    0.0,
                    base.FORCE_NOISE_PEAK_RATIO * float(np.max(exact_force)),
                    base.TRANSFER_SAMPLES,
                )
                response_noise = response_rngs[index].normal(
                    0.0,
                    base.RESPONSE_NOISE_RMS_RATIO * response_rms,
                    base.TRANSFER_SAMPLES,
                )
                response_noise += base.room_tail(
                    room_rngs[index], base.ROOM_TAIL_RMS_RATIO * response_rms
                )
                trials.append(
                    {
                        "contact": contact,
                        "profile_index": profile_index,
                        "repeat": repeat,
                        "profile": base.profile_descriptor(profile),
                        "exact_force": exact_force,
                        "force_noise": force_noise,
                        "observed_force": exact_force + force_noise,
                        "clean_response": clean_response,
                        "response_noise": response_noise,
                        "observed_response": clean_response + response_noise,
                    }
                )
    return {"role": role, "profiles": profiles, "repeats": repeats, "trials": trials}


def safe_divide(
    numerator: np.ndarray, denominator: np.ndarray, valid: np.ndarray
) -> np.ndarray:
    result = np.zeros_like(numerator, dtype=np.complex128)
    np.divide(numerator, denominator, out=result, where=valid)
    return result


def estimate_noise_aware(trials: list[dict[str, Any]]) -> dict[str, Any]:
    transform = lambda key: np.stack(  # noqa: E731
        [
            np.fft.rfft(item[key], n=base.ANALYSIS_FFT_SAMPLES)
            for item in trials
        ]
    )
    x = transform("observed_force")
    y = transform("observed_response")
    nx = transform("force_noise")
    ny = transform("response_noise")
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
        raise OracleError("B1R2 corrected force signal has invalid power")
    input_snr = gxx / np.maximum(nxx, 1.0e-30)
    relative_signal = gxx / maximum_signal
    force_valid = (
        base.frequency_band()
        & (input_snr >= INPUT_SNR_FLOOR)
        & (relative_signal >= RELATIVE_SIGNAL_POWER_FLOOR)
    )

    covariance = np.empty((len(gxx), 2, 2), dtype=np.complex128)
    covariance[:, 0, 0] = gxx
    covariance[:, 0, 1] = np.conj(gyx)
    covariance[:, 1, 0] = gyx
    covariance[:, 1, 1] = gyy
    eigenvalues, eigenvectors = np.linalg.eigh(covariance)
    direction = eigenvectors[:, :, 1]
    vector_valid = np.abs(direction[:, 0]) > EIGENVECTOR_INPUT_FLOOR
    gtls_valid = force_valid & vector_valid
    gtls = safe_divide(direction[:, 1], direction[:, 0], gtls_valid)
    gtls *= base.taper_mask(gtls_valid)

    coherence_denominator = np.maximum(gxx * gyy, 1.0e-30)
    raw_coherence = np.abs(gyx) ** 2 / coherence_denominator
    corrected_coherence = np.clip(raw_coherence, 0.0, 1.0)
    raw_h1 = safe_divide(
        gyx_observed,
        gxx_observed,
        base.frequency_band() & (gxx_observed > 0.0),
    )
    corrected_h1 = safe_divide(gyx, gxx, force_valid & (gxx > 0.0))
    corrected_h2 = safe_divide(
        gyy,
        np.conj(gyx),
        force_valid & (np.abs(gyx) > 1.0e-30),
    )
    raw_h1 *= base.taper_mask(base.frequency_band())
    corrected_h1 *= base.taper_mask(force_valid)
    corrected_h2 *= base.taper_mask(force_valid)

    trial_power = np.abs(x) ** 2
    eligible = trial_power >= base.CONDITIONING_FLOOR * np.max(
        trial_power, axis=1
    )[:, None]
    divisions = np.zeros_like(y)
    np.divide(y, x, out=divisions, where=eligible)
    counts = np.sum(eligible, axis=0)
    direct = safe_divide(np.sum(divisions, axis=0), counts, counts > 0)
    direct *= base.taper_mask(base.frequency_band() & (counts > 0))
    return {
        "gtls": gtls,
        "gtls_valid": gtls_valid,
        "force_valid": force_valid,
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
    return {
        "truth_count": len(TRUTH_FREQUENCIES_HZ),
        "candidate_count": len(modes),
        "truth_match_count": len(comparisons),
        "false_positive_count": len(modes) - len(comparisons),
        "comparisons": comparisons,
    }


def modal_support(
    modes: list[dict[str, Any]], input_snr: np.ndarray, coherence: np.ndarray
) -> list[dict[str, Any]]:
    frequencies = np.fft.rfftfreq(
        base.ANALYSIS_FFT_SAMPLES, 1.0 / base.SAMPLE_RATE_HZ
    )
    reports = []
    for mode_index, mode in enumerate(modes):
        radius = max(
            20.0, 4.0 * mode["decay_per_second"] / (2.0 * math.pi)
        )
        neighborhood = np.abs(frequencies - mode["frequency_hz"]) <= radius
        contacts = []
        for contact in range(base.CONTACT_COUNT):
            median_snr = float(np.median(input_snr[contact, neighborhood]))
            median_coherence = float(np.median(coherence[contact, neighborhood]))
            contacts.append(
                {
                    "contact": contact,
                    "median_input_snr": median_snr,
                    "median_corrected_coherence": median_coherence,
                    "supports_pole": median_snr >= INPUT_SNR_FLOOR
                    and median_coherence >= MODE_SUPPORT_COHERENCE,
                    "residue_confident": median_snr >= INPUT_SNR_FLOOR
                    and median_coherence >= RESIDUE_CONFIDENCE_COHERENCE,
                    "residue_uncertainty": 1.0 - median_coherence,
                }
            )
        reports.append(
            {
                "mode_index": mode_index,
                "frequency_hz": mode["frequency_hz"],
                "decay_per_second": mode["decay_per_second"],
                "neighborhood_radius_hz": radius,
                "supporting_contact_count": sum(
                    item["supports_pole"] for item in contacts
                ),
                "confident_residue_count": sum(
                    item["residue_confident"] for item in contacts
                ),
                "contacts": contacts,
            }
        )
    return reports


def fit_estimators(fit_role: dict[str, Any]) -> dict[str, Any]:
    estimates = []
    raw_outputs = []
    impulses = []
    shortest_index = min(
        range(len(base.FIT_PROFILES)),
        key=lambda index: base.FIT_PROFILES[index][1][0],
    )
    analysis_weights = base.taper_mask(base.frequency_band())
    for contact in range(base.CONTACT_COUNT):
        trials = [item for item in fit_role["trials"] if item["contact"] == contact]
        if len(trials) != len(base.FIT_PROFILES) * FIT_REPEATS:
            raise OracleError("B1R2 fit contact trial count changed")
        estimates.append(estimate_noise_aware(trials))
        shortest_trials = [
            item for item in trials if item["profile_index"] == shortest_index
        ]
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
            "force_valid",
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
    modes = base.identify_modes(gtls_impulses)
    support = modal_support(
        modes["modes"], stacked["input_snr"], stacked["corrected_coherence"]
    )
    modal = np.fft.rfft(
        modes["reconstruction"], n=base.ANALYSIS_FFT_SAMPLES, axis=1
    )
    for contact in range(base.CONTACT_COUNT):
        modal[contact] *= base.taper_mask(stacked["force_valid"][contact])
    raw_modes = base.identify_modes(np.stack(raw_outputs))
    raw_modal = np.fft.rfft(
        raw_modes["reconstruction"], n=base.ANALYSIS_FFT_SAMPLES, axis=1
    ) * analysis_weights[None, :]
    return {
        **stacked,
        "candidate": modal,
        "impulse": np.stack(impulses),
        "raw_modal": raw_modal,
        "modes": modes,
        "support": support,
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
        force_spectrum = np.fft.rfft(
            item["exact_force"], n=base.ANALYSIS_FFT_SAMPLES
        )
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
                    name: base.normalized_rms_error(
                        item["clean_response"], predictions[name]
                    )
                    for name in names
                },
                "candidate_log_spectrum_rmse_db": base.log_spectrum_rmse_db(
                    item["clean_response"], predictions["candidate"]
                ),
            }
        )
    mean_nrmse = {
        name: float(np.mean([item["nrmse"][name] for item in trials]))
        for name in names
    }
    candidate_values = [item["nrmse"]["candidate"] for item in trials]
    truth = match_truth(estimators["modes"]["modes"])
    frequency_errors = [item["frequency_error_hz"] for item in truth["comparisons"]]
    decay_errors = [item["decay_error_per_second"] for item in truth["comparisons"]]
    relative_errors = [item["relative_decay_error"] for item in truth["comparisons"]]
    force_coverages = [
        base.coverage(item, base.COVERAGE_MAXIMUM_HZ)
        for item in estimators["force_valid"]
    ]
    supporting = [item["supporting_contact_count"] for item in estimators["support"]]
    confident = [item["confident_residue_count"] for item in estimators["support"]]
    best_corrected = min(mean_nrmse["corrected_h1"], mean_nrmse["corrected_h2"])
    return {
        "role": role["role"],
        "trial_count": len(trials),
        "trials": trials,
        "mean_nrmse": mean_nrmse,
        "maximum_candidate_nrmse": max(candidate_values),
        "mean_candidate_log_spectrum_rmse_db": float(
            np.mean([item["candidate_log_spectrum_rmse_db"] for item in trials])
        ),
        "candidate_over_impulse_nrmse": mean_nrmse["candidate"]
        / mean_nrmse["impulse"],
        "candidate_over_raw_modal_nrmse": mean_nrmse["candidate"]
        / mean_nrmse["raw_modal"],
        "candidate_over_raw_h1_nrmse": mean_nrmse["candidate"]
        / mean_nrmse["raw_h1"],
        "gtls_over_best_corrected_nrmse": mean_nrmse["gtls"] / best_corrected,
        "force_coverage_by_contact": force_coverages,
        "minimum_force_coverage": min(force_coverages),
        "truth": truth,
        "maximum_frequency_error_hz": max(frequency_errors, default=math.inf),
        "maximum_decay_error_per_second": max(decay_errors, default=math.inf),
        "maximum_relative_decay_error": max(relative_errors, default=math.inf),
        "minimum_supporting_contacts": min(supporting, default=0),
        "minimum_confident_residues": min(confident, default=0),
        "all_residues_nonzero": all(
            all(residue["magnitude"] > 0.0 for residue in mode["residues"])
            for mode in estimators["modes"]["modes"]
        ),
    }


def generate_corruption(
    kind: str, offset: int, transfer: np.ndarray
) -> list[dict[str, Any]]:
    count = 4
    seeds = ROLE_SEEDS["corruption"]
    force_rngs = base.child_generators(seeds["force"] + offset, count)
    response_rngs = base.child_generators(seeds["response"] + offset, count)
    room_rngs = base.child_generators(seeds["room"] + offset, count)
    transfer_spectrum = np.fft.rfft(transfer, n=base.ANALYSIS_FFT_SAMPLES)
    profile = ("hann", (1025.0,)) if kind == "weak_excitation" else ("half_sine", (19.0,))
    measured = base.force_profile(profile)
    delays = (401, 613, 887, 1151)
    trials = []
    for index in range(count):
        true_force = measured.copy()
        if kind == "missing_second_impact":
            active = base.force_profile(("half_sine", (19.0,)))[:19]
            true_force[delays[index] : delays[index] + 19] += 0.8 * active
        clean_response = base.fft_convolve(true_force, transfer_spectrum)
        response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
        force_noise = force_rngs[index].normal(
            0.0,
            base.FORCE_NOISE_PEAK_RATIO * float(np.max(measured)),
            base.TRANSFER_SAMPLES,
        )
        response_noise = response_rngs[index].normal(
            0.0,
            base.RESPONSE_NOISE_RMS_RATIO * response_rms,
            base.TRANSFER_SAMPLES,
        )
        response_noise += base.room_tail(
            room_rngs[index], base.ROOM_TAIL_RMS_RATIO * response_rms
        )
        if kind == "low_coherence":
            interference = response_rngs[index].standard_normal(base.TRANSFER_SAMPLES)
            interference = signal.lfilter([1.0], [1.0, -0.91], interference)
            interference_rms = math.sqrt(float(np.mean(interference * interference)))
            interference *= 0.75 * response_rms / interference_rms
            response_noise += interference
        trials.append(
            {
                "contact": 0,
                "profile_index": index,
                "repeat": 0,
                "exact_force": true_force,
                "force_noise": force_noise,
                "observed_force": measured + force_noise,
                "clean_response": clean_response,
                "response_noise": response_noise,
                "observed_response": clean_response + response_noise,
            }
        )
    return trials


def evaluate_corruption(kind: str, offset: int, transfer: np.ndarray) -> dict[str, Any]:
    trials = generate_corruption(kind, offset, transfer)
    estimate = estimate_noise_aware(trials)
    frequencies = np.fft.rfftfreq(
        base.ANALYSIS_FFT_SAMPLES, 1.0 / base.SAMPLE_RATE_HZ
    )
    high_band = (frequencies > 4_000.0) & (
        frequencies <= base.MAXIMUM_ANALYSIS_HZ
    )
    band = base.frequency_band()
    high_coverage = float(np.mean(estimate["force_valid"][high_band]))
    median_coherence = float(np.median(estimate["corrected_coherence"][band]))
    coherent_coverage = float(
        np.mean(estimate["corrected_coherence"][band] >= MODE_SUPPORT_COHERENCE)
    )
    residuals = []
    for item in trials:
        prediction = np.fft.irfft(
            np.fft.rfft(item["observed_force"], n=base.ANALYSIS_FFT_SAMPLES)
            * estimate["gtls"],
            n=base.ANALYSIS_FFT_SAMPLES,
        )[: base.TRANSFER_SAMPLES]
        residuals.append(
            base.normalized_rms_error(item["observed_response"], prediction)
        )
    median_residual = float(np.median(residuals))
    if (
        kind == "weak_excitation"
        and high_coverage <= GATES["maximum_weak_high_band_coverage"]
    ):
        decision = "OOD_WEAK_EXCITATION"
    elif kind == "low_coherence" and (
        median_coherence <= GATES["maximum_low_coherence_median"]
        or coherent_coverage <= GATES["maximum_low_coherence_coverage"]
    ):
        decision = "OOD_LOW_COHERENCE"
    elif (
        kind == "missing_second_impact"
        and median_residual >= GATES["minimum_double_impact_residual_nrmse"]
    ):
        decision = "OOD_MODEL_MISMATCH"
    else:
        decision = "INVALIDLY_ADMITTED"
    return {
        "kind": kind,
        "repeat_count": len(trials),
        "high_band_force_coverage": high_coverage,
        "median_corrected_coherence": median_coherence,
        "coherent_coverage": coherent_coverage,
        "median_reconstruction_nrmse": median_residual,
        "decision": decision,
        "admitted_mode_count": 0 if decision != "INVALIDLY_ADMITTED" else None,
    }


def evaluation_gates(prefix: str, value: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        base.check(f"{prefix}.minimum_force_coverage", value["minimum_force_coverage"], ">=", GATES["minimum_force_coverage"]),
        base.check(f"{prefix}.truth_match_count", value["truth"]["truth_match_count"], "==", GATES["truth_match_count"]),
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
        "decision": "B1R2_NOISE_AWARE_ORACLE_FROZEN",
        "fixture_samples_generated": 0,
        "noise_samples_generated": 0,
        "fit_trials_evaluated": 0,
        "development_trials_evaluated": 0,
        "holdout_trials_evaluated": 0,
        "gate": {"passed": True, "checks": []},
    }
    base.write_atomic(output / "report.json", base.canonical_json(report))


def run_oracle(payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    transfers = exact_transfers()
    fit_role = generate_role("fit", base.FIT_PROFILES, transfers, FIT_REPEATS)
    estimators = fit_estimators(fit_role)
    development_role = generate_role(
        "development", base.DEVELOPMENT_PROFILES, transfers, 1
    )
    development = evaluate_held(development_role, estimators)
    identity = base.evaluate_identity(transfers)
    development_corruptions = {
        "weak_excitation": evaluate_corruption("weak_excitation", 100, transfers[0]),
        "low_coherence": evaluate_corruption("low_coherence", 200, transfers[0]),
    }
    checks = evaluation_gates("development", development)
    checks.extend(
        [
            base.check("identity.nrmse", identity["nrmse"], "<=", GATES["maximum_identity_nrmse"]),
            base.check("identity.finite", identity["finite"], "==", True),
            base.check("development.weak_excitation_decision", development_corruptions["weak_excitation"]["decision"], "==", "OOD_WEAK_EXCITATION"),
            base.check("development.weak_admitted_modes", development_corruptions["weak_excitation"]["admitted_mode_count"], "==", 0),
            base.check("development.low_coherence_decision", development_corruptions["low_coherence"]["decision"], "==", "OOD_LOW_COHERENCE"),
            base.check("development.low_coherence_admitted_modes", development_corruptions["low_coherence"]["admitted_mode_count"], "==", 0),
        ]
    )
    development_passed = all(item["passed"] for item in checks)
    holdout = None
    holdout_corruptions = None
    if development_passed:
        holdout_role = generate_role("holdout", base.HOLDOUT_PROFILES, transfers, 1)
        holdout = evaluate_held(holdout_role, estimators)
        holdout_corruptions = {
            "weak_excitation": evaluate_corruption("weak_excitation", 300, transfers[0]),
            "missing_second_impact": evaluate_corruption(
                "missing_second_impact", 400, transfers[0]
            ),
        }
        checks.extend(evaluation_gates("holdout", holdout))
        checks.extend(
            [
                base.check("holdout.weak_excitation_decision", holdout_corruptions["weak_excitation"]["decision"], "==", "OOD_WEAK_EXCITATION"),
                base.check("holdout.weak_admitted_modes", holdout_corruptions["weak_excitation"]["admitted_mode_count"], "==", 0),
                base.check("holdout.double_impact_decision", holdout_corruptions["missing_second_impact"]["decision"], "==", "OOD_MODEL_MISMATCH"),
                base.check("holdout.double_impact_admitted_modes", holdout_corruptions["missing_second_impact"]["admitted_mode_count"], "==", 0),
            ]
        )

    arrays = {
        "modal_transfer.npy": estimators["candidate"].astype("<c16"),
        "gtls_transfer.npy": estimators["gtls"].astype("<c16"),
        "input_snr.npy": estimators["input_snr"].astype("<f8"),
        "corrected_coherence.npy": estimators["corrected_coherence"].astype("<f8"),
        "force_valid_mask.npy": estimators["force_valid"].astype(np.bool_),
        "modal_reconstruction.npy": estimators["modes"]["reconstruction"].astype("<f8"),
    }
    array_hashes = {
        name: base.write_npy(output / name, value) for name, value in arrays.items()
    }
    model = {
        "schema": MODEL_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": base.sha256_bytes(payload),
        "array_sha256": array_hashes,
        "modes": estimators["modes"]["modes"],
        "modal_support": estimators["support"],
        "negative_corrected_covariance_eigenvalue_count": estimators[
            "negative_eigenvalue_count"
        ],
        "candidate": manifest["candidate"],
        "product_credit": False,
    }
    if not base.finite_tree(model):
        raise OracleError("B1R2 model contains nonfinite values")
    model_payload = base.canonical_json(model)
    base.write_atomic(output / "model.json", model_payload)

    passed = all(item["passed"] for item in checks)
    report = {
        "schema": REPORT_SCHEMAS["run"],
        **common_report(payload, manifest),
        "decision": "PASS_KNOWN_TRUTH_FRF" if passed else "REJECT_NOISE_AWARE_FRF",
        "role_access_order": ["noise_calibration_and_fit", "development"]
        + (["holdout"] if holdout is not None else []),
        "sample_accounting": {
            "fixture_transfer_samples": int(transfers.size),
            "fit_trials": len(fit_role["trials"]),
            "fit_force_noise_samples": len(fit_role["trials"])
            * base.TRANSFER_SAMPLES,
            "fit_response_noise_samples": len(fit_role["trials"])
            * base.TRANSFER_SAMPLES,
            "development_trials": len(development_role["trials"]),
            "holdout_trials": 0
            if holdout is None
            else base.CONTACT_COUNT * len(base.HOLDOUT_PROFILES),
            "real_samples": 0,
            "parent_holdout_samples": 0,
        },
        "identity": identity,
        "mode_discovery": {
            "region_count": len(estimators["modes"]["discovery"]["regions"]),
            "analysis_bin_count": len(
                estimators["modes"]["discovery"]["analysis_bins"]
            ),
            "raw_estimate_count": estimators["modes"]["discovery"][
                "raw_estimate_count"
            ],
            "retained_mode_count": len(estimators["modes"]["modes"]),
            "regions": estimators["modes"]["discovery"]["regions"],
        },
        "development": development,
        "development_corruptions": development_corruptions,
        "holdout": holdout,
        "holdout_corruptions": holdout_corruptions,
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
        raise OracleError("B1R2 report contains nonfinite values")
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
