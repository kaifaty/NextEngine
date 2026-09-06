#!/usr/bin/env python3
"""Run the preregistered V12-C1 acquisition-coverage modal oracle."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any

import numpy as np
import scipy

import physical_sound_contact_field_r3a_v11_b1_force_response_oracle as base
import physical_sound_contact_field_r3a_v11_b1r3_local_modal_support as parent


STUDY_ID = "physical-sound-r3a-v12-c1-acquisition-coverage"
REVISION = "force-ensemble-certificate-common-pole-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c1.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-r3a-v12-c1-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-r3a-v12-c1.report.v1",
}
CERTIFICATE_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v12-c1-force-certificate.v1"
)
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c1.model.v1"

PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-r3a-v12-c1-acquisition-coverage-oracle-protocol-2026-08-31.md"
)
RESEARCH_PATH = (
    "docs/development/"
    "physical-sound-r3a-v12-c1-coverage-and-source-research-2026-08-31.md"
)
PARENT_RUNNER_PATH = (
    "lab/scripts/physical_sound_contact_field_r3a_v11_b1r3_local_modal_support.py"
)
PARENT_RESULT_PATH = (
    "docs/development/"
    "physical-sound-r3a-v11-b1r3-local-modal-support-result-2026-08-31.md"
)
B1R2_RUNNER_PATH = (
    "lab/scripts/physical_sound_contact_field_r3a_v11_b1r2_noise_aware_gtls.py"
)
BASE_RUNNER_PATH = (
    "lab/scripts/physical_sound_contact_field_r3a_v11_b1_force_response_oracle.py"
)
POLE_CORE_PATH = "lab/scripts/physical_sound_subband_common_pole_control.py"

TRUTH_FREQUENCIES_HZ = np.asarray(
    [719.0, 1_493.0, 2_579.0, 4_013.0, 6_000.0, 7_877.0, 9_251.0],
    dtype=np.float64,
)
TRUTH_DECAYS_PER_SECOND = np.asarray(
    [4.75, 7.5, 10.5, 15.0, 21.5, 30.0, 41.0], dtype=np.float64
)
FIT_PROFILES = (
    ("half_sine", (3.0,)),
    ("half_sine", (5.0,)),
    ("hann", (7.0,)),
    ("beta", (9.0, 2.0, 3.0)),
    ("half_sine", (13.0,)),
)
DEVELOPMENT_PROFILES = (
    ("half_sine", (11.0,)),
    ("beta", (15.0, 3.0, 4.0)),
)
HOLDOUT_PROFILES = (("hann", (9.0,)), ("beta", (17.0, 4.0, 3.0)))
NOTCH_BASE_PROFILES = (
    ("half_sine", (3.0,)),
    ("half_sine", (5.0,)),
    ("hann", (7.0,)),
)
WEAK_PROFILES = tuple(("hann", (float(value),)) for value in (769, 833, 897, 961, 1025))
FIT_REPEATS = 4
ROLE_SEEDS = {
    "fit": {"force": 2_026_120_101, "response": 2_026_120_102, "room": 2_026_120_103},
    "development": {
        "force": 2_026_120_201,
        "response": 2_026_120_202,
        "room": 2_026_120_203,
    },
    "holdout": {
        "force": 2_026_120_301,
        "response": 2_026_120_302,
        "room": 2_026_120_303,
    },
    "development_controls": {
        "force": 2_026_120_401,
        "response": 2_026_120_402,
        "room": 2_026_120_403,
    },
    "holdout_controls": {
        "force": 2_026_120_501,
        "response": 2_026_120_502,
        "room": 2_026_120_503,
    },
}

MINIMUM_CERTIFICATE_HZ = 200.0
MAXIMUM_CERTIFICATE_HZ = 9_500.0
CERTIFICATE_REFERENCE_HZ = 12_000.0
CERTIFICATE_INPUT_SNR = 25.0
CERTIFICATE_RELATIVE_POWER = 0.005
MINIMUM_PROFILE_SUPPORT_COUNT = 3
MINIMUM_LOCAL_CERTIFICATE_FRACTION = 0.90
MODE_SUPPORT_COHERENCE = 0.90
RESIDUE_CONFIDENCE_COHERENCE = 0.85
MINIMUM_SUPPORTING_CONTACTS = 3
MINIMUM_CONFIDENT_RESIDUES = 3
NOTCH_TRUTH_INDEX = 4
NOTCH_DELAY_SAMPLES = 4

GATES = {
    "truth_mode_count": 7,
    "false_positive_count": 0,
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
    "maximum_query_notch_nrmse": 0.08,
    "maximum_query_notch_target_energy_ratio": 0.02,
    "minimum_low_coherence_residual_nrmse": 0.20,
    "minimum_missing_impact_residual_nrmse": 0.15,
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
        "research": (root / RESEARCH_PATH).resolve(strict=True),
        "parent_runner": (root / PARENT_RUNNER_PATH).resolve(strict=True),
        "parent_result": (root / PARENT_RESULT_PATH).resolve(strict=True),
        "b1r2_runner": (root / B1R2_RUNNER_PATH).resolve(strict=True),
        "base_runner": (root / BASE_RUNNER_PATH).resolve(strict=True),
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
        "force_certificate": {
            "sample_rate_hz": base.SAMPLE_RATE_HZ,
            "sample_count": base.TRANSFER_SAMPLES,
            "fft_sample_count": base.ANALYSIS_FFT_SAMPLES,
            "profiles": [profile_descriptor(value) for value in FIT_PROFILES],
            "contact_count": base.CONTACT_COUNT,
            "repeats": FIT_REPEATS,
            "minimum_hz": MINIMUM_CERTIFICATE_HZ,
            "maximum_hz": MAXIMUM_CERTIFICATE_HZ,
            "reference_maximum_hz": CERTIFICATE_REFERENCE_HZ,
            "minimum_input_snr": CERTIFICATE_INPUT_SNR,
            "minimum_relative_power": CERTIFICATE_RELATIVE_POWER,
            "minimum_profile_support_count": MINIMUM_PROFILE_SUPPORT_COUNT,
            "minimum_local_fraction": MINIMUM_LOCAL_CERTIFICATE_FRACTION,
            "leave_one_profile_out_required": True,
            "response_or_truth_access_allowed": False,
        },
        "fixture": {
            "contact_count": base.CONTACT_COUNT,
            "truth_frequencies_hz": TRUTH_FREQUENCIES_HZ.tolist(),
            "truth_decays_per_second": TRUTH_DECAYS_PER_SECOND.tolist(),
            "transfer_peak": base.TRANSFER_PEAK,
            "residue_magnitude_rule": (
                "(0.33 + 0.039*((13*contact + 5*mode + 7) mod 19))/(1 + 0.018*mode)"
            ),
            "residue_phase_rule": (
                "0.211 + 0.17*contact + 0.137*mode + 0.029*contact*mode radians"
            ),
        },
        "roles": {
            "fit_profiles": [profile_descriptor(value) for value in FIT_PROFILES],
            "development_profiles": [
                profile_descriptor(value) for value in DEVELOPMENT_PROFILES
            ],
            "holdout_profiles": [profile_descriptor(value) for value in HOLDOUT_PROFILES],
            "fit_repeats": FIT_REPEATS,
            "seeds": ROLE_SEEDS,
            "order": [
                "force_certificate",
                "truth_fixture_and_fit_response",
                "development",
                "holdout",
            ],
        },
        "observation": {
            "force_sensor_noise_peak_ratio": base.FORCE_NOISE_PEAK_RATIO,
            "response_sensor_noise_rms_ratio": base.RESPONSE_NOISE_RMS_RATIO,
            "unmeasured_room_tail_rms_ratio": base.ROOM_TAIL_RMS_RATIO,
            "calibrated_components": ["force_sensor_noise", "response_sensor_noise"],
            "uncalibrated_components": ["room_disturbance", "dynamic_interference"],
        },
        "candidate": {
            "estimator": "noise_corrected_dominant_covariance_GTLS",
            "mode_discovery": "B1R3_Gabor_common_pole",
            "selected_transfer": "explicit acquisition-certified common-modal reconstruction",
            "minimum_supporting_contacts": MINIMUM_SUPPORTING_CONTACTS,
            "minimum_confident_residues": MINIMUM_CONFIDENT_RESIDUES,
            "mode_support_coherence": MODE_SUPPORT_COHERENCE,
            "residue_confidence_coherence": RESIDUE_CONFIDENCE_COHERENCE,
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
        "controls": {
            "acquisition_notch": {
                "base_profiles": [profile_descriptor(value) for value in NOTCH_BASE_PROFILES],
                "delay_samples": NOTCH_DELAY_SAMPLES,
                "truth_index": NOTCH_TRUTH_INDEX,
                "expected_decision": "OOD_ACQUISITION_HOLE",
            },
            "query_notch": {
                "base_profile": profile_descriptor(NOTCH_BASE_PROFILES[0]),
                "delay_samples": NOTCH_DELAY_SAMPLES,
                "expected_decision": "VALID_QUERY",
            },
            "weak_profiles": [profile_descriptor(value) for value in WEAK_PROFILES],
            "low_coherence": {
                "profile": profile_descriptor(("half_sine", (7.0,))),
                "interference_rms_ratio": 0.75,
            },
            "missing_impact": {
                "measured_profile": profile_descriptor(("half_sine", (13.0,))),
                "missing_profile": profile_descriptor(("half_sine", (17.0,))),
                "gain": 0.8,
                "delays_samples": [373, 619, 907, 1231],
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
            "PASS_KNOWN_TRUTH_FRF opens C3 only with independent C2 source pass; "
            "any valid estimator gate failure closes this common-pole revision"
        ),
    }


def load_manifest(root: Path, path: Path) -> tuple[bytes, dict[str, Any]]:
    resolved = base.require_external_file(root, path, "V12-C1 manifest")
    payload = resolved.read_bytes()
    if len(payload) > 256 * 1024:
        raise OracleError("V12-C1 manifest exceeds 256 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleError("cannot parse V12-C1 manifest") from error
    if payload != base.canonical_json(value):
        raise OracleError("V12-C1 manifest is not canonical JSON")
    if value != expected_manifest(root):
        raise OracleError("V12-C1 manifest or implementation lineage changed")
    return payload, value


def frequencies_hz() -> np.ndarray:
    return np.fft.rfftfreq(
        base.ANALYSIS_FFT_SAMPLES, 1.0 / base.SAMPLE_RATE_HZ
    )


def certificate_band() -> np.ndarray:
    frequencies = frequencies_hz()
    return (frequencies >= MINIMUM_CERTIFICATE_HZ) & (
        frequencies <= MAXIMUM_CERTIFICATE_HZ
    )


def modal_neighborhood(frequency_hz: float, decay_per_second: float) -> np.ndarray:
    radius = max(24.0, 4.0 * decay_per_second / (2.0 * math.pi))
    return np.abs(frequencies_hz() - frequency_hz) <= radius


def notched_profile(base_profile: tuple[str, tuple[float, ...]]) -> np.ndarray:
    active_count = int(base_profile[1][0])
    active = base.force_profile(base_profile)[:active_count]
    result = np.zeros(base.TRANSFER_SAMPLES, dtype=np.float64)
    result[:active_count] += active
    result[NOTCH_DELAY_SAMPLES : NOTCH_DELAY_SAMPLES + active_count] += active
    total = float(np.sum(result))
    if total <= 0.0 or not math.isfinite(total) or np.any(result < 0.0):
        raise OracleError("V12-C1 notched force profile is invalid")
    result /= total
    return result


def force_profile(
    profile: tuple[str, tuple[float, ...]], *, notch: bool = False
) -> np.ndarray:
    return notched_profile(profile) if notch else base.force_profile(profile)


def generate_force_role(
    role: str,
    profiles: tuple[tuple[str, tuple[float, ...]], ...],
    repeats: int,
    force_seed: int,
    *,
    notch: bool = False,
) -> dict[str, Any]:
    count = base.CONTACT_COUNT * len(profiles) * repeats
    generators = base.child_generators(force_seed, count)
    trials = []
    for contact in range(base.CONTACT_COUNT):
        for profile_index, profile in enumerate(profiles):
            exact_force = force_profile(profile, notch=notch)
            for repeat in range(repeats):
                index = (contact * len(profiles) + profile_index) * repeats + repeat
                sensor_noise = generators[index].normal(
                    0.0,
                    base.FORCE_NOISE_PEAK_RATIO * float(np.max(exact_force)),
                    base.TRANSFER_SAMPLES,
                )
                trials.append(
                    {
                        "contact": contact,
                        "profile_index": profile_index,
                        "repeat": repeat,
                        "exact_force": exact_force,
                        "force_sensor_noise": sensor_noise,
                        "observed_force": exact_force + sensor_noise,
                    }
                )
    return {
        "role": role,
        "profiles": profiles,
        "repeats": repeats,
        "notch": notch,
        "trials": trials,
    }


def force_certificate(
    role: dict[str, Any], *, minimum_supporters: int = MINIMUM_PROFILE_SUPPORT_COUNT
) -> dict[str, Any]:
    profiles = role["profiles"]
    profile_reports = []
    support_masks = []
    input_snrs = []
    relative_powers = []
    reference_band = frequencies_hz() <= CERTIFICATE_REFERENCE_HZ
    for profile_index, profile in enumerate(profiles):
        trials = [
            item for item in role["trials"] if item["profile_index"] == profile_index
        ]
        expected = base.CONTACT_COUNT * role["repeats"]
        if len(trials) != expected:
            raise OracleError("V12-C1 force certificate profile count changed")
        observed = np.stack(
            [
                np.fft.rfft(item["observed_force"], n=base.ANALYSIS_FFT_SAMPLES)
                for item in trials
            ]
        )
        noise = np.stack(
            [
                np.fft.rfft(item["force_sensor_noise"], n=base.ANALYSIS_FFT_SAMPLES)
                for item in trials
            ]
        )
        gxx = np.maximum(
            np.sum(np.abs(observed) ** 2, axis=0)
            - np.sum(np.abs(noise) ** 2, axis=0),
            0.0,
        )
        nxx = np.sum(np.abs(noise) ** 2, axis=0)
        maximum = float(np.max(gxx[reference_band]))
        if maximum <= 0.0 or not math.isfinite(maximum):
            raise OracleError("V12-C1 force certificate has invalid corrected power")
        input_snr = gxx / np.maximum(nxx, 1.0e-30)
        relative_power = gxx / maximum
        support = (
            certificate_band()
            & (input_snr >= CERTIFICATE_INPUT_SNR)
            & (relative_power >= CERTIFICATE_RELATIVE_POWER)
        )
        support_masks.append(support)
        input_snrs.append(input_snr)
        relative_powers.append(relative_power)
        profile_reports.append(
            {
                "profile_index": profile_index,
                "profile": profile_descriptor(profile),
                "supported_band_fraction": float(np.mean(support[certificate_band()])),
                "minimum_band_input_snr": float(np.min(input_snr[certificate_band()])),
                "minimum_band_relative_power": float(
                    np.min(relative_power[certificate_band()])
                ),
            }
        )
    support_stack = np.stack(support_masks)
    counts = np.sum(support_stack, axis=0)
    certified = certificate_band() & (counts >= minimum_supporters)
    truth = []
    for index, (frequency, decay) in enumerate(
        zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
    ):
        local = modal_neighborhood(float(frequency), float(decay))
        fraction = float(np.mean(certified[local]))
        truth.append(
            {
                "truth_index": index,
                "frequency_hz": float(frequency),
                "decay_per_second": float(decay),
                "certificate_fraction": fraction,
                "minimum_profile_support_count": int(np.min(counts[local])),
                "median_profile_support_count": float(np.median(counts[local])),
                "certified": fraction >= MINIMUM_LOCAL_CERTIFICATE_FRACTION,
            }
        )
    supported = [item["truth_index"] for item in truth if item["certified"]]
    return {
        "schema": CERTIFICATE_SCHEMA,
        "role": role["role"],
        "minimum_supporters": minimum_supporters,
        "profile_reports": profile_reports,
        "supported_band_fraction": float(np.mean(certified[certificate_band()])),
        "minimum_band_profile_support_count": int(np.min(counts[certificate_band()])),
        "truth_neighborhoods": truth,
        "supported_truth_indices": supported,
        "unsupported_truth_indices": [
            index for index in range(len(truth)) if index not in supported
        ],
        "support_masks": support_stack,
        "support_count": counts,
        "certified_mask": certified,
        "input_snr": np.stack(input_snrs),
        "relative_power": np.stack(relative_powers),
    }


def certificate_json(certificate: dict[str, Any]) -> dict[str, Any]:
    return {
        name: value
        for name, value in certificate.items()
        if name
        not in {
            "support_masks",
            "support_count",
            "certified_mask",
            "input_snr",
            "relative_power",
        }
    }


def ordinary_certificate_checks(certificate: dict[str, Any]) -> list[dict[str, Any]]:
    checks = [
        base.check(
            "certificate.supported_band_fraction",
            certificate["supported_band_fraction"],
            "==",
            1.0,
        ),
        base.check(
            "certificate.minimum_band_profile_support_count",
            certificate["minimum_band_profile_support_count"],
            ">=",
            MINIMUM_PROFILE_SUPPORT_COUNT,
        ),
        base.check(
            "certificate.supported_truth_indices",
            certificate["supported_truth_indices"],
            "==",
            list(range(len(TRUTH_FREQUENCIES_HZ))),
        ),
    ]
    for omitted in range(len(FIT_PROFILES)):
        remaining = np.delete(certificate["support_masks"], omitted, axis=0)
        certified = certificate_band() & (
            np.sum(remaining, axis=0) >= MINIMUM_PROFILE_SUPPORT_COUNT
        )
        for index, (frequency, decay) in enumerate(
            zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
        ):
            local = modal_neighborhood(float(frequency), float(decay))
            checks.append(
                base.check(
                    f"certificate.leave_one_out.{omitted}.truth.{index}.fraction",
                    float(np.mean(certified[local])),
                    ">=",
                    MINIMUM_LOCAL_CERTIFICATE_FRACTION,
                )
            )
    return checks


def exact_transfers() -> tuple[np.ndarray, np.ndarray]:
    time = np.arange(base.TRANSFER_SAMPLES, dtype=np.float64) / base.SAMPLE_RATE_HZ
    transfers = np.zeros((base.CONTACT_COUNT, base.TRANSFER_SAMPLES), dtype=np.float64)
    components = np.zeros(
        (base.CONTACT_COUNT, len(TRUTH_FREQUENCIES_HZ), base.TRANSFER_SAMPLES),
        dtype=np.float64,
    )
    for contact in range(base.CONTACT_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
        ):
            gain = (0.33 + 0.039 * ((13 * contact + 5 * mode + 7) % 19)) / (
                1.0 + 0.018 * mode
            )
            phase = 0.211 + 0.17 * contact + 0.137 * mode + 0.029 * contact * mode
            component = (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
            components[contact, mode] = component
            transfers[contact] += component
    peak = float(np.max(np.abs(transfers)))
    if peak <= 0.0 or not math.isfinite(peak):
        raise OracleError("V12-C1 exact transfer has invalid peak")
    scale = base.TRANSFER_PEAK / peak
    return transfers * scale, components * scale


def attach_responses(
    force_role: dict[str, Any],
    transfers: np.ndarray,
    response_seed: int,
    room_seed: int,
    *,
    interference_rms_ratio: float = 0.0,
) -> dict[str, Any]:
    count = len(force_role["trials"])
    response_generators = base.child_generators(response_seed, count)
    room_generators = base.child_generators(room_seed, count)
    transfer_spectra = np.fft.rfft(
        transfers, n=base.ANALYSIS_FFT_SAMPLES, axis=1
    )
    trials = []
    for index, item in enumerate(force_role["trials"]):
        clean_response = base.fft_convolve(
            item["exact_force"], transfer_spectra[item["contact"]]
        )
        response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
        if response_rms <= 0.0 or not math.isfinite(response_rms):
            raise OracleError("V12-C1 clean response has invalid RMS")
        response_sensor_noise = response_generators[index].normal(
            0.0,
            base.RESPONSE_NOISE_RMS_RATIO * response_rms,
            base.TRANSFER_SAMPLES,
        )
        room = base.room_tail(
            room_generators[index], base.ROOM_TAIL_RMS_RATIO * response_rms
        )
        interference = np.zeros(base.TRANSFER_SAMPLES, dtype=np.float64)
        if interference_rms_ratio > 0.0:
            interference = base.room_tail(
                room_generators[index], interference_rms_ratio * response_rms
            )
        trials.append(
            {
                **item,
                "clean_response": clean_response,
                "response_sensor_noise": response_sensor_noise,
                "room_disturbance": room + interference,
                "observed_response": clean_response
                + response_sensor_noise
                + room
                + interference,
            }
        )
    return {**force_role, "trials": trials}


def reconstruct_modes(modes: list[dict[str, Any]]) -> np.ndarray:
    time = np.arange(base.TRANSFER_SAMPLES, dtype=np.float64) / base.SAMPLE_RATE_HZ
    result = np.zeros((base.CONTACT_COUNT, base.TRANSFER_SAMPLES), dtype=np.float64)
    for mode in modes:
        envelope = np.exp(-mode["decay_per_second"] * time)
        phase = 2.0 * math.pi * mode["frequency_hz"] * time
        cosine = envelope * np.cos(phase)
        sine = envelope * np.sin(phase)
        for contact, residue in enumerate(mode["residues"]):
            result[contact] += residue["cosine"] * cosine + residue["sine"] * sine
    return result


def support_for_mode(
    mode: dict[str, Any],
    certificate: dict[str, Any],
    coherence: np.ndarray,
) -> dict[str, Any]:
    local = modal_neighborhood(mode["frequency_hz"], mode["decay_per_second"])
    certificate_fraction = float(np.mean(certificate["certified_mask"][local]))
    contacts = []
    for contact in range(base.CONTACT_COUNT):
        median_coherence = float(np.median(coherence[contact, local]))
        contacts.append(
            {
                "contact": contact,
                "median_corrected_coherence": median_coherence,
                "supports_pole": certificate_fraction
                >= MINIMUM_LOCAL_CERTIFICATE_FRACTION
                and median_coherence >= MODE_SUPPORT_COHERENCE,
                "residue_confident": certificate_fraction
                >= MINIMUM_LOCAL_CERTIFICATE_FRACTION
                and median_coherence >= RESIDUE_CONFIDENCE_COHERENCE,
                "residue_uncertainty": 1.0 - median_coherence,
            }
        )
    supporting = sum(item["supports_pole"] for item in contacts)
    confident = sum(item["residue_confident"] for item in contacts)
    return {
        "frequency_hz": mode["frequency_hz"],
        "decay_per_second": mode["decay_per_second"],
        "certificate_fraction": certificate_fraction,
        "supporting_contact_count": supporting,
        "confident_residue_count": confident,
        "contacts": contacts,
        "retained": supporting >= MINIMUM_SUPPORTING_CONTACTS
        and confident >= MINIMUM_CONFIDENT_RESIDUES,
    }


def fit_estimators(
    fit_role: dict[str, Any],
    certificate: dict[str, Any],
) -> dict[str, Any]:
    estimates = []
    raw_outputs = []
    impulses = []
    profile_lengths = [int(profile[1][0]) for profile in fit_role["profiles"]]
    shortest_index = min(range(len(profile_lengths)), key=profile_lengths.__getitem__)
    certified_taper = base.taper_mask(certificate["certified_mask"])
    for contact in range(base.CONTACT_COUNT):
        trials = [item for item in fit_role["trials"] if item["contact"] == contact]
        expected = len(fit_role["profiles"]) * fit_role["repeats"]
        if len(trials) != expected:
            raise OracleError("V12-C1 fit contact trial count changed")
        estimate = parent.estimate_noise_aware(trials)
        for name in ("gtls", "raw_h1", "corrected_h1", "corrected_h2", "direct"):
            estimate[name] = estimate[name] * certified_taper
        estimate["gtls_valid"] = estimate["gtls_valid"] & certificate["certified_mask"]
        estimate["discovery_valid"] = certificate["certified_mask"].copy()
        estimates.append(estimate)
        shortest = [
            item for item in trials if item["profile_index"] == shortest_index
        ]
        impulses.append(
            np.fft.rfft(
                np.mean(
                    np.stack([item["observed_response"] for item in shortest]),
                    axis=0,
                ),
                n=base.ANALYSIS_FFT_SAMPLES,
            )
            * certified_taper
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
    discovered = parent.identify_modes(gtls_impulses)
    in_band_modes = [
        mode
        for mode in discovered["modes"]
        if MINIMUM_CERTIFICATE_HZ <= mode["frequency_hz"] <= MAXIMUM_CERTIFICATE_HZ
    ]
    support = [
        support_for_mode(mode, certificate, stacked["corrected_coherence"])
        for mode in in_band_modes
    ]
    modes = [
        mode
        for mode, report in zip(in_band_modes, support, strict=True)
        if report["retained"]
    ]
    retained_support = [item for item in support if item["retained"]]
    modal_reconstruction = reconstruct_modes(modes)
    modal = np.fft.rfft(
        modal_reconstruction, n=base.ANALYSIS_FFT_SAMPLES, axis=1
    )
    raw_modes = parent.identify_modes(np.stack(raw_outputs))
    raw_modal = np.fft.rfft(
        raw_modes["reconstruction"], n=base.ANALYSIS_FFT_SAMPLES, axis=1
    ) * certified_taper[None, :]
    return {
        **stacked,
        "candidate": modal,
        "impulse": np.stack(impulses),
        "raw_modal": raw_modal,
        "modes": {
            "discovery": discovered["discovery"],
            "modes": modes,
            "reconstruction": modal_reconstruction,
            "initial_candidate_count": len(in_band_modes),
        },
        "support": support,
        "retained_support": retained_support,
        "negative_eigenvalue_count": sum(
            item["negative_eigenvalue_count"] for item in estimates
        ),
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
        decay = float(TRUTH_DECAYS_PER_SECOND[assignment])
        comparisons.append(
            {
                "truth_index": assignment,
                "expected_frequency_hz": float(TRUTH_FREQUENCIES_HZ[assignment]),
                "observed_frequency_hz": mode["frequency_hz"],
                "frequency_error_hz": abs(
                    mode["frequency_hz"] - float(TRUTH_FREQUENCIES_HZ[assignment])
                ),
                "expected_decay_per_second": decay,
                "observed_decay_per_second": mode["decay_per_second"],
                "decay_error_per_second": abs(mode["decay_per_second"] - decay),
                "relative_decay_error": abs(mode["decay_per_second"] - decay) / decay,
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
        force = np.fft.rfft(item["exact_force"], n=base.ANALYSIS_FFT_SAMPLES)
        predictions = {
            name: np.fft.irfft(
                force * estimators[name][contact], n=base.ANALYSIS_FFT_SAMPLES
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
    truth = match_truth(estimators["modes"]["modes"])
    frequency_errors = [item["frequency_error_hz"] for item in truth["comparisons"]]
    decay_errors = [item["decay_error_per_second"] for item in truth["comparisons"]]
    relative_errors = [item["relative_decay_error"] for item in truth["comparisons"]]
    supporting = [item["supporting_contact_count"] for item in estimators["retained_support"]]
    confident = [item["confident_residue_count"] for item in estimators["retained_support"]]
    candidate_values = [item["nrmse"]["candidate"] for item in trials]
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
        "candidate_over_impulse_nrmse": mean_nrmse["candidate"] / mean_nrmse["impulse"],
        "candidate_over_raw_modal_nrmse": mean_nrmse["candidate"]
        / mean_nrmse["raw_modal"],
        "candidate_over_raw_h1_nrmse": mean_nrmse["candidate"] / mean_nrmse["raw_h1"],
        "gtls_over_best_corrected_nrmse": mean_nrmse["gtls"] / best_corrected,
        "truth": truth,
        "supported_truth_indices": truth["matched_truth_indices"],
        "unsupported_truth_indices": truth["unmatched_truth_indices"],
        "supported_truth_mode_count": truth["truth_match_count"],
        "unsupported_truth_mode_count": len(truth["unmatched_truth_indices"]),
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


def remove_profile_role(role: dict[str, Any], omitted: int) -> dict[str, Any]:
    profiles = tuple(
        profile for index, profile in enumerate(role["profiles"]) if index != omitted
    )
    trials = []
    for item in role["trials"]:
        if item["profile_index"] == omitted:
            continue
        trials.append(
            {
                **item,
                "profile_index": item["profile_index"]
                - (1 if item["profile_index"] > omitted else 0),
            }
        )
    return {**role, "profiles": profiles, "trials": trials}


def leave_one_out_results(fit_role: dict[str, Any]) -> list[dict[str, Any]]:
    results = []
    for omitted in range(len(FIT_PROFILES)):
        role = remove_profile_role(fit_role, omitted)
        certificate = force_certificate(role)
        estimators = fit_estimators(role, certificate)
        truth = match_truth(estimators["modes"]["modes"])
        results.append(
            {
                "omitted_profile_index": omitted,
                "omitted_profile": profile_descriptor(FIT_PROFILES[omitted]),
                "certificate": certificate_json(certificate),
                "retained_mode_count": len(estimators["modes"]["modes"]),
                "truth": truth,
            }
        )
    return results


def acquisition_notch_control(
    transfers: np.ndarray, seeds: dict[str, int]
) -> dict[str, Any]:
    force_role = generate_force_role(
        "control_acquisition_notch",
        NOTCH_BASE_PROFILES,
        FIT_REPEATS,
        seeds["force"],
        notch=True,
    )
    certificate = force_certificate(force_role)
    role = attach_responses(
        force_role, transfers, seeds["response"], seeds["room"]
    )
    estimators = fit_estimators(role, certificate)
    truth = match_truth(estimators["modes"]["modes"])
    expected = [index for index in range(len(TRUTH_FREQUENCIES_HZ)) if index != NOTCH_TRUTH_INDEX]
    decision = (
        "OOD_ACQUISITION_HOLE"
        if certificate["supported_truth_indices"] == expected
        and truth["matched_truth_indices"] == expected
        and truth["false_positive_count"] == 0
        else "INVALID_ACQUISITION_HOLE_RESULT"
    )
    return {
        "kind": "acquisition_notch",
        "trial_count": len(role["trials"]),
        "certificate": certificate_json(certificate),
        "truth": truth,
        "decision": decision,
        "complete_domain_admitted_mode_count": 0,
    }


def weak_acquisition_control(force_seed: int) -> dict[str, Any]:
    role = generate_force_role(
        "control_weak_acquisition", WEAK_PROFILES, FIT_REPEATS, force_seed
    )
    certificate = force_certificate(role)
    incomplete = len(certificate["supported_truth_indices"]) < len(TRUTH_FREQUENCIES_HZ)
    return {
        "kind": "weak_acquisition",
        "trial_count": len(role["trials"]),
        "certificate": certificate_json(certificate),
        "decision": "OOD_WEAK_EXCITATION" if incomplete else "INVALIDLY_CERTIFIED",
        "complete_domain_admitted_mode_count": 0 if incomplete else None,
    }


def query_notch_control(
    estimators: dict[str, Any],
    transfers: np.ndarray,
    components: np.ndarray,
    seeds: dict[str, int],
) -> dict[str, Any]:
    force_role = generate_force_role(
        "control_query_notch",
        (NOTCH_BASE_PROFILES[0],),
        1,
        seeds["force"] + 10,
        notch=True,
    )
    role = attach_responses(
        force_role, transfers, seeds["response"] + 10, seeds["room"] + 10
    )
    nrmse = []
    target_ratios = []
    for item in role["trials"]:
        contact = item["contact"]
        force_spectrum = np.fft.rfft(
            item["exact_force"], n=base.ANALYSIS_FFT_SAMPLES
        )
        prediction = np.fft.irfft(
            force_spectrum * estimators["candidate"][contact],
            n=base.ANALYSIS_FFT_SAMPLES,
        )[: base.TRANSFER_SAMPLES]
        nrmse.append(base.normalized_rms_error(item["clean_response"], prediction))
        modal_rms = []
        for mode in range(len(TRUTH_FREQUENCIES_HZ)):
            response = base.fft_convolve(
                item["exact_force"],
                np.fft.rfft(
                    components[contact, mode], n=base.ANALYSIS_FFT_SAMPLES
                ),
            )
            modal_rms.append(math.sqrt(float(np.mean(response * response))))
        target_ratios.append(
            modal_rms[NOTCH_TRUTH_INDEX]
            / max(math.sqrt(sum(value * value for value in modal_rms)), 1.0e-30)
        )
    mean_nrmse = float(np.mean(nrmse))
    maximum_target_ratio = max(target_ratios)
    passed = (
        mean_nrmse <= GATES["maximum_query_notch_nrmse"]
        and maximum_target_ratio
        <= GATES["maximum_query_notch_target_energy_ratio"]
    )
    return {
        "kind": "query_notch",
        "trial_count": len(role["trials"]),
        "mean_clean_response_nrmse": mean_nrmse,
        "maximum_clean_response_nrmse": max(nrmse),
        "maximum_target_mode_energy_ratio": maximum_target_ratio,
        "decision": "VALID_QUERY" if passed else "INVALID_QUERY_NOTCH_RESULT",
        "object_model_mode_count": len(estimators["modes"]["modes"]),
    }


def observed_residuals(role: dict[str, Any], estimators: dict[str, Any]) -> list[float]:
    residuals = []
    for item in role["trials"]:
        contact = item["contact"]
        prediction = np.fft.irfft(
            np.fft.rfft(item["observed_force"], n=base.ANALYSIS_FFT_SAMPLES)
            * estimators["candidate"][contact],
            n=base.ANALYSIS_FFT_SAMPLES,
        )[: base.TRANSFER_SAMPLES]
        residuals.append(
            base.normalized_rms_error(item["observed_response"], prediction)
        )
    return residuals


def low_coherence_control(
    estimators: dict[str, Any], transfers: np.ndarray, seeds: dict[str, int]
) -> dict[str, Any]:
    force_role = generate_force_role(
        "control_low_coherence",
        (("half_sine", (7.0,)),),
        1,
        seeds["force"] + 20,
    )
    role = attach_responses(
        force_role,
        transfers,
        seeds["response"] + 20,
        seeds["room"] + 20,
        interference_rms_ratio=0.75,
    )
    residuals = observed_residuals(role, estimators)
    median = float(np.median(residuals))
    return {
        "kind": "low_coherence",
        "trial_count": len(role["trials"]),
        "median_observed_response_reconstruction_nrmse": median,
        "decision": (
            "OOD_LOW_COHERENCE"
            if median >= GATES["minimum_low_coherence_residual_nrmse"]
            else "INVALIDLY_ADMITTED"
        ),
        "complete_domain_admitted_mode_count": 0,
    }


def missing_impact_control(
    estimators: dict[str, Any], transfer: np.ndarray, seeds: dict[str, int]
) -> dict[str, Any]:
    count = 4
    force_generators = base.child_generators(seeds["force"] + 30, count)
    response_generators = base.child_generators(seeds["response"] + 30, count)
    room_generators = base.child_generators(seeds["room"] + 30, count)
    measured = base.force_profile(("half_sine", (13.0,)))
    missing = base.force_profile(("half_sine", (17.0,)))[:17]
    delays = (373, 619, 907, 1231)
    transfer_spectrum = np.fft.rfft(transfer, n=base.ANALYSIS_FFT_SAMPLES)
    trials = []
    for index, delay in enumerate(delays):
        true_force = measured.copy()
        true_force[delay : delay + len(missing)] += 0.8 * missing
        clean_response = base.fft_convolve(true_force, transfer_spectrum)
        response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
        force_noise = force_generators[index].normal(
            0.0,
            base.FORCE_NOISE_PEAK_RATIO * float(np.max(measured)),
            base.TRANSFER_SAMPLES,
        )
        response_noise = response_generators[index].normal(
            0.0,
            base.RESPONSE_NOISE_RMS_RATIO * response_rms,
            base.TRANSFER_SAMPLES,
        )
        room = base.room_tail(
            room_generators[index], base.ROOM_TAIL_RMS_RATIO * response_rms
        )
        trials.append(
            {
                "contact": 0,
                "observed_force": measured + force_noise,
                "observed_response": clean_response + response_noise + room,
            }
        )
    residuals = []
    for item in trials:
        prediction = np.fft.irfft(
            np.fft.rfft(item["observed_force"], n=base.ANALYSIS_FFT_SAMPLES)
            * estimators["candidate"][0],
            n=base.ANALYSIS_FFT_SAMPLES,
        )[: base.TRANSFER_SAMPLES]
        residuals.append(
            base.normalized_rms_error(item["observed_response"], prediction)
        )
    median = float(np.median(residuals))
    return {
        "kind": "missing_impact",
        "trial_count": count,
        "median_observed_response_reconstruction_nrmse": median,
        "decision": (
            "OOD_MODEL_MISMATCH"
            if median >= GATES["minimum_missing_impact_residual_nrmse"]
            else "INVALIDLY_ADMITTED"
        ),
        "complete_domain_admitted_mode_count": 0,
    }


def evaluation_gates(prefix: str, value: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        base.check(
            f"{prefix}.supported_truth_mode_count",
            value["supported_truth_mode_count"],
            "==",
            GATES["truth_mode_count"],
        ),
        base.check(
            f"{prefix}.unsupported_truth_mode_count",
            value["unsupported_truth_mode_count"],
            "==",
            0,
        ),
        base.check(
            f"{prefix}.false_positive_count",
            value["truth"]["false_positive_count"],
            "==",
            GATES["false_positive_count"],
        ),
        base.check(
            f"{prefix}.minimum_supporting_contacts",
            value["minimum_supporting_contacts"],
            ">=",
            MINIMUM_SUPPORTING_CONTACTS,
        ),
        base.check(
            f"{prefix}.minimum_confident_residues",
            value["minimum_confident_residues"],
            ">=",
            MINIMUM_CONFIDENT_RESIDUES,
        ),
        base.check(
            f"{prefix}.maximum_frequency_error_hz",
            value["maximum_frequency_error_hz"],
            "<=",
            GATES["maximum_frequency_error_hz"],
        ),
        base.check(
            f"{prefix}.maximum_decay_error_per_second",
            value["maximum_decay_error_per_second"],
            "<=",
            GATES["maximum_decay_error_per_second"],
        ),
        base.check(
            f"{prefix}.maximum_relative_decay_error",
            value["maximum_relative_decay_error"],
            "<=",
            GATES["maximum_relative_decay_error"],
        ),
        base.check(
            f"{prefix}.mean_held_nrmse",
            value["mean_nrmse"]["candidate"],
            "<=",
            GATES["maximum_mean_held_nrmse"],
        ),
        base.check(
            f"{prefix}.maximum_held_nrmse",
            value["maximum_candidate_nrmse"],
            "<=",
            GATES["maximum_held_nrmse"],
        ),
        base.check(
            f"{prefix}.mean_log_spectrum_rmse_db",
            value["mean_candidate_log_spectrum_rmse_db"],
            "<=",
            GATES["maximum_mean_log_spectrum_rmse_db"],
        ),
        base.check(
            f"{prefix}.impulse_nrmse_ratio",
            value["candidate_over_impulse_nrmse"],
            "<=",
            GATES["maximum_impulse_nrmse_ratio"],
        ),
        base.check(
            f"{prefix}.raw_modal_nrmse_ratio",
            value["candidate_over_raw_modal_nrmse"],
            "<=",
            GATES["maximum_raw_modal_nrmse_ratio"],
        ),
        base.check(
            f"{prefix}.raw_h1_nrmse_ratio",
            value["candidate_over_raw_h1_nrmse"],
            "<=",
            GATES["maximum_raw_h1_nrmse_ratio"],
        ),
        base.check(
            f"{prefix}.gtls_best_corrected_ratio",
            value["gtls_over_best_corrected_nrmse"],
            "<=",
            GATES["maximum_gtls_over_best_corrected_ratio"],
        ),
        base.check(
            f"{prefix}.all_residues_nonzero",
            value["all_residues_nonzero"],
            "==",
            True,
        ),
    ]


def leave_one_out_gates(results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    checks = []
    expected = list(range(len(TRUTH_FREQUENCIES_HZ)))
    for item in results:
        prefix = f"leave_one_out.{item['omitted_profile_index']}"
        checks.extend(
            [
                base.check(
                    f"{prefix}.certificate.supported_truth_indices",
                    item["certificate"]["supported_truth_indices"],
                    "==",
                    expected,
                ),
                base.check(
                    f"{prefix}.truth.matched_truth_indices",
                    item["truth"]["matched_truth_indices"],
                    "==",
                    expected,
                ),
                base.check(
                    f"{prefix}.truth.false_positive_count",
                    item["truth"]["false_positive_count"],
                    "==",
                    0,
                ),
            ]
        )
    return checks


def control_gates(prefix: str, controls: dict[str, Any]) -> list[dict[str, Any]]:
    acquisition = controls["acquisition_notch"]
    query = controls["query_notch"]
    weak = controls["weak_acquisition"]
    low = controls["low_coherence"]
    expected_notch = [
        index for index in range(len(TRUTH_FREQUENCIES_HZ)) if index != NOTCH_TRUTH_INDEX
    ]
    return [
        base.check(
            f"{prefix}.acquisition_notch.certificate.supported_truth_indices",
            acquisition["certificate"]["supported_truth_indices"],
            "==",
            expected_notch,
        ),
        base.check(
            f"{prefix}.acquisition_notch.truth.matched_truth_indices",
            acquisition["truth"]["matched_truth_indices"],
            "==",
            expected_notch,
        ),
        base.check(
            f"{prefix}.acquisition_notch.false_positive_count",
            acquisition["truth"]["false_positive_count"],
            "==",
            0,
        ),
        base.check(
            f"{prefix}.acquisition_notch.decision",
            acquisition["decision"],
            "==",
            "OOD_ACQUISITION_HOLE",
        ),
        base.check(
            f"{prefix}.query_notch.decision", query["decision"], "==", "VALID_QUERY"
        ),
        base.check(
            f"{prefix}.query_notch.object_model_mode_count",
            query["object_model_mode_count"],
            "==",
            7,
        ),
        base.check(
            f"{prefix}.weak.decision",
            weak["decision"],
            "==",
            "OOD_WEAK_EXCITATION",
        ),
        base.check(
            f"{prefix}.low_coherence.decision",
            low["decision"],
            "==",
            "OOD_LOW_COHERENCE",
        ),
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
            "ml": False,
            "validator": False,
            "atlas": False,
            "runtime": False,
        },
    }


def write_certificate_artifacts(
    output: Path,
    payload: bytes,
    certificate: dict[str, Any],
) -> tuple[dict[str, str], bytes]:
    arrays = {
        "force_profile_support.npy": certificate["support_masks"].astype(np.bool_),
        "force_support_count.npy": certificate["support_count"].astype("<i8"),
        "force_certified_mask.npy": certificate["certified_mask"].astype(np.bool_),
        "force_input_snr.npy": certificate["input_snr"].astype("<f8"),
        "force_relative_power.npy": certificate["relative_power"].astype("<f8"),
    }
    hashes = {
        name: base.write_npy(output / name, value) for name, value in arrays.items()
    }
    document = {
        **certificate_json(certificate),
        "manifest_sha256": base.sha256_bytes(payload),
        "array_sha256": hashes,
        "truth_or_response_samples_read": 0,
    }
    if not base.finite_tree(document):
        raise OracleError("V12-C1 force certificate contains nonfinite values")
    certificate_payload = base.canonical_json(document)
    base.write_atomic(output / "certificate.json", certificate_payload)
    return hashes, certificate_payload


def freeze(root: Path, output: Path) -> None:
    base.write_atomic(output / "manifest.json", base.canonical_json(expected_manifest(root)))


def preflight(payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        **common_report(payload, manifest),
        "decision": "C1_ACQUISITION_COVERAGE_ORACLE_FROZEN",
        "force_samples_generated": 0,
        "truth_samples_generated": 0,
        "response_samples_generated": 0,
        "fit_trials_evaluated": 0,
        "development_trials_evaluated": 0,
        "holdout_trials_evaluated": 0,
        "gate": {"passed": True, "checks": []},
    }
    base.write_atomic(output / "report.json", base.canonical_json(report))


def run_oracle(payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    force_role = generate_force_role(
        "fit", FIT_PROFILES, FIT_REPEATS, ROLE_SEEDS["fit"]["force"]
    )
    certificate = force_certificate(force_role)
    certificate_checks = ordinary_certificate_checks(certificate)
    certificate_passed = all(item["passed"] for item in certificate_checks)
    certificate_array_hashes, certificate_payload = write_certificate_artifacts(
        output, payload, certificate
    )
    certificate_sha256 = base.sha256_bytes(certificate_payload)
    if not certificate_passed:
        report = {
            "schema": REPORT_SCHEMAS["run"],
            **common_report(payload, manifest),
            "decision": "DATA_INSUFFICIENT_FORCE_COVERAGE",
            "role_access_order": ["force_certificate"],
            "force_certificate": certificate_json(certificate),
            "certificate_sha256": certificate_sha256,
            "certificate_array_sha256": certificate_array_hashes,
            "sample_accounting": {
                "force_certificate_trials": len(force_role["trials"]),
                "force_sensor_noise_samples": len(force_role["trials"])
                * base.TRANSFER_SAMPLES,
                "truth_transfer_samples": 0,
                "response_samples": 0,
                "development_trials": 0,
                "holdout_trials": 0,
                "real_samples": 0,
                "parent_holdout_samples": 0,
            },
            "gate": {"passed": False, "checks": certificate_checks},
            "next_authorized_step": "ACQUISITION_DESIGN_RECONSIDERATION_ONLY",
        }
        base.write_atomic(output / "report.json", base.canonical_json(report))
        return

    transfers, components = exact_transfers()
    fit_role = attach_responses(
        force_role,
        transfers,
        ROLE_SEEDS["fit"]["response"],
        ROLE_SEEDS["fit"]["room"],
    )
    estimators = fit_estimators(fit_role, certificate)
    leave_one_out = leave_one_out_results(fit_role)
    development_force = generate_force_role(
        "development",
        DEVELOPMENT_PROFILES,
        1,
        ROLE_SEEDS["development"]["force"],
    )
    development_role = attach_responses(
        development_force,
        transfers,
        ROLE_SEEDS["development"]["response"],
        ROLE_SEEDS["development"]["room"],
    )
    development = evaluate_held(development_role, estimators)
    development_controls = {
        "acquisition_notch": acquisition_notch_control(
            transfers, ROLE_SEEDS["development_controls"]
        ),
        "query_notch": query_notch_control(
            estimators, transfers, components, ROLE_SEEDS["development_controls"]
        ),
        "weak_acquisition": weak_acquisition_control(
            ROLE_SEEDS["development_controls"]["force"] + 40
        ),
        "low_coherence": low_coherence_control(
            estimators, transfers, ROLE_SEEDS["development_controls"]
        ),
    }
    identity = base.evaluate_identity(transfers)
    checks = list(certificate_checks)
    checks.extend(evaluation_gates("development", development))
    checks.extend(leave_one_out_gates(leave_one_out))
    checks.extend(control_gates("development", development_controls))
    checks.extend(
        [
            base.check(
                "identity.nrmse",
                identity["nrmse"],
                "<=",
                GATES["maximum_identity_nrmse"],
            ),
            base.check("identity.finite", identity["finite"], "==", True),
        ]
    )
    development_passed = all(item["passed"] for item in checks)

    holdout = None
    holdout_controls = None
    if development_passed:
        holdout_force = generate_force_role(
            "holdout",
            HOLDOUT_PROFILES,
            1,
            ROLE_SEEDS["holdout"]["force"],
        )
        holdout_role = attach_responses(
            holdout_force,
            transfers,
            ROLE_SEEDS["holdout"]["response"],
            ROLE_SEEDS["holdout"]["room"],
        )
        holdout = evaluate_held(holdout_role, estimators)
        holdout_controls = {
            "acquisition_notch": acquisition_notch_control(
                transfers, ROLE_SEEDS["holdout_controls"]
            ),
            "query_notch": query_notch_control(
                estimators, transfers, components, ROLE_SEEDS["holdout_controls"]
            ),
            "weak_acquisition": weak_acquisition_control(
                ROLE_SEEDS["holdout_controls"]["force"] + 40
            ),
            "low_coherence": low_coherence_control(
                estimators, transfers, ROLE_SEEDS["holdout_controls"]
            ),
            "missing_impact": missing_impact_control(
                estimators, transfers[0], ROLE_SEEDS["holdout_controls"]
            ),
        }
        checks.extend(evaluation_gates("holdout", holdout))
        checks.extend(control_gates("holdout", holdout_controls))
        checks.extend(
            [
                base.check(
                    "holdout.missing_impact.decision",
                    holdout_controls["missing_impact"]["decision"],
                    "==",
                    "OOD_MODEL_MISMATCH",
                ),
                base.check(
                    "holdout.missing_impact.admitted_modes",
                    holdout_controls["missing_impact"][
                        "complete_domain_admitted_mode_count"
                    ],
                    "==",
                    0,
                ),
            ]
        )

    arrays = {
        "modal_transfer.npy": estimators["candidate"].astype("<c16"),
        "gtls_transfer.npy": estimators["gtls"].astype("<c16"),
        "corrected_coherence.npy": estimators["corrected_coherence"].astype("<f8"),
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
        "certificate_sha256": certificate_sha256,
        "array_sha256": array_hashes,
        "modes": estimators["modes"]["modes"],
        "modal_support": estimators["retained_support"],
        "negative_corrected_covariance_eigenvalue_count": estimators[
            "negative_eigenvalue_count"
        ],
        "candidate": manifest["candidate"],
        "product_credit": False,
    }
    if not base.finite_tree(model):
        raise OracleError("V12-C1 model contains nonfinite values")
    model_payload = base.canonical_json(model)
    base.write_atomic(output / "model.json", model_payload)

    passed = all(item["passed"] for item in checks)
    development_control_trials = sum(
        item["trial_count"] for item in development_controls.values()
    )
    holdout_control_trials = (
        0
        if holdout_controls is None
        else sum(item["trial_count"] for item in holdout_controls.values())
    )
    report = {
        "schema": REPORT_SCHEMAS["run"],
        **common_report(payload, manifest),
        "decision": (
            "PASS_KNOWN_TRUTH_FRF"
            if passed
            else "REJECT_COVERAGE_CERTIFIED_COMMON_POLE"
        ),
        "role_access_order": [
            "force_certificate",
            "truth_fixture_and_fit_response",
            "development",
        ]
        + (["holdout"] if holdout is not None else []),
        "sample_accounting": {
            "force_certificate_trials": len(force_role["trials"]),
            "fit_trials": len(fit_role["trials"]),
            "fit_force_sensor_noise_samples": len(fit_role["trials"])
            * base.TRANSFER_SAMPLES,
            "fit_response_sensor_noise_samples": len(fit_role["trials"])
            * base.TRANSFER_SAMPLES,
            "fit_unmeasured_room_samples": len(fit_role["trials"])
            * base.TRANSFER_SAMPLES,
            "truth_transfer_samples": int(transfers.size),
            "leave_one_out_fit_trial_evaluations": sum(
                (len(FIT_PROFILES) - 1) * FIT_REPEATS * base.CONTACT_COUNT
                for _ in range(len(FIT_PROFILES))
            ),
            "development_trials": len(development_role["trials"]),
            "development_control_trials": development_control_trials,
            "holdout_trials": 0 if holdout is None else base.CONTACT_COUNT * len(HOLDOUT_PROFILES),
            "holdout_control_trials": holdout_control_trials,
            "real_samples": 0,
            "parent_holdout_samples": 0,
        },
        "force_certificate": certificate_json(certificate),
        "certificate_sha256": certificate_sha256,
        "certificate_array_sha256": certificate_array_hashes,
        "identity": identity,
        "mode_discovery": {
            "region_count": len(estimators["modes"]["discovery"]["regions"]),
            "analysis_bin_count": len(
                estimators["modes"]["discovery"]["analysis_bins"]
            ),
            "raw_estimate_count": estimators["modes"]["discovery"][
                "raw_estimate_count"
            ],
            "initial_candidate_count": estimators["modes"]["initial_candidate_count"],
            "retained_mode_count": len(estimators["modes"]["modes"]),
            "regions": estimators["modes"]["discovery"]["regions"],
        },
        "leave_one_profile_out": leave_one_out,
        "development": development,
        "development_controls": development_controls,
        "holdout": holdout,
        "holdout_controls": holdout_controls,
        "model_sha256": base.sha256_bytes(model_payload),
        "array_sha256": array_hashes,
        "gate": {"passed": passed, "checks": checks},
        "next_authorized_step": (
            "C3_SOURCE_ROLE_FREEZE_IF_C2_PASSED"
            if passed
            else "CLOSE_COMMON_POLE_AND_PREREGISTER_ONE_LOCAL_RATIONAL_COMPARISON"
        ),
    }
    if not base.finite_tree(report):
        raise OracleError("V12-C1 report contains nonfinite values")
    base.write_atomic(output / "report.json", base.canonical_json(report))


def main() -> int:
    arguments = parse_arguments()
    root = base.repository_root()
    output = base.prepare_output(root, arguments.output)
    if arguments.stage == "freeze":
        if arguments.manifest is not None:
            raise OracleError("V12-C1 freeze does not accept --manifest")
        freeze(root, output)
        return 0
    if arguments.manifest is None:
        raise OracleError("V12-C1 preflight/run requires --manifest")
    payload, manifest = load_manifest(root, arguments.manifest)
    if arguments.stage == "preflight":
        preflight(payload, manifest, output)
    else:
        run_oracle(payload, manifest, output)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleError as error:
        raise SystemExit(f"V12-C1 oracle: {error}") from error
