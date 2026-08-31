#!/usr/bin/env python3
"""Run the preregistered V11-B1 synthetic force/response oracle."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
from pathlib import Path
from typing import Any, Iterable

import numpy as np
from scipy import signal
import scipy

import physical_sound_subband_common_pole_control as pole_core


STUDY_ID = "physical-sound-contact-field-r3a-v11-b1-force-response-oracle"
REVISION = "known-truth-h1-force-response-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v11-b1.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-r3a-v11-b1-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-r3a-v11-b1.report.v1",
}
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v11-b1.model.v1"

PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-r3a-v11-b1-force-response-oracle-protocol-2026-08-31.md"
)
POLE_CORE_PATH = "lab/scripts/physical_sound_subband_common_pole_control.py"

SAMPLE_RATE_HZ = 48_000
TRANSFER_SAMPLES = 96_000
ANALYSIS_FFT_SAMPLES = 262_144
CONTACT_COUNT = 6
TRUTH_FREQUENCIES_HZ = np.asarray(
    [523.0, 911.0, 1_487.0, 2_381.0, 3_769.0, 6_029.0, 9_137.0],
    dtype=np.float64,
)
TRUTH_DECAYS_PER_SECOND = np.asarray(
    [4.0, 6.5, 9.0, 13.0, 19.0, 28.0, 42.0], dtype=np.float64
)
MODE_COUNT = len(TRUTH_FREQUENCIES_HZ)
TRANSFER_PEAK = 0.25

FIT_PROFILES = (
    ("half_sine", (9.0,)),
    ("hann", (19.0,)),
    ("beta", (37.0, 2.0, 4.0)),
    ("half_sine", (61.0,)),
)
DEVELOPMENT_PROFILES = (
    ("hann", (13.0,)),
    ("beta", (47.0, 3.0, 2.0)),
)
HOLDOUT_PROFILES = (
    ("half_sine", (25.0,)),
    ("beta", (53.0, 2.0, 5.0)),
)
IDENTITY_PROFILE = ("beta", (43.0, 2.0, 3.0))

ROLE_SEEDS = {
    "fit": {"force": 2_026_110_101, "response": 2_026_110_102, "room": 2_026_110_103},
    "development": {
        "force": 2_026_110_201,
        "response": 2_026_110_202,
        "room": 2_026_110_203,
    },
    "holdout": {
        "force": 2_026_110_301,
        "response": 2_026_110_302,
        "room": 2_026_110_303,
    },
    "corruption": {
        "force": 2_026_110_401,
        "response": 2_026_110_402,
        "room": 2_026_110_403,
    },
}

FORCE_NOISE_PEAK_RATIO = 2.0e-4
RESPONSE_NOISE_RMS_RATIO = 2.0e-4
ROOM_TAIL_RMS_RATIO = 0.002
ROOM_TAIL_START = 1_920
ROOM_TAIL_POLE = 0.82
ROOM_TAIL_DECAY_PER_SECOND = 18.0

MINIMUM_ANALYSIS_HZ = 200.0
MAXIMUM_ANALYSIS_HZ = 12_000.0
COVERAGE_MAXIMUM_HZ = 10_000.0
CONDITIONING_FLOOR = 1.0e-5
COHERENCE_FLOOR = 0.98
REGULARIZATION_RATIO = 1.0e-8
MASK_TRANSITION_BINS = 4

GABOR_WINDOW_SAMPLES = 1_024
GABOR_HOP_SAMPLES = 32
GABOR_FRAME_COUNT = 256
GABOR_REGION_FLOOR_DB = -35.0
GABOR_NEIGHBORHOOD_BINS = 1
MINIMUM_ORDER_SCORE_MARGIN = 20.0
DUPLICATE_FREQUENCY_HZ = 1.0
MODE_ENERGY_FLOOR_DB = -30.0
MODE_MATCH_RADIUS_HZ = 12.0
MODE_ANALYSIS_SAMPLES = pole_core.SAMPLE_COUNT
MODE_ONSET_SAMPLE = pole_core.ONSET_SAMPLE

GATES = {
    "minimum_valid_coverage": 0.90,
    "truth_match_count": 7,
    "false_positive_count": 0,
    "maximum_frequency_error_hz": 1.0,
    "maximum_decay_error_per_second": 1.5,
    "maximum_relative_decay_error": 0.20,
    "maximum_mean_held_nrmse": 0.06,
    "maximum_held_nrmse": 0.10,
    "maximum_mean_log_spectrum_rmse_db": 1.5,
    "maximum_impulse_nrmse_ratio": 0.65,
    "maximum_raw_modal_nrmse_ratio": 0.75,
    "maximum_direct_division_nrmse_ratio": 1.00,
    "maximum_identity_nrmse": 1.0e-11,
    "maximum_weak_high_band_coverage": 0.35,
    "maximum_low_coherence_median": 0.80,
    "maximum_low_coherence_coverage": 0.50,
    "minimum_double_impact_residual_nrmse": 0.15,
}


class OracleError(RuntimeError):
    """The frozen synthetic oracle or its exact lineage is invalid."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=("freeze", "preflight", "run"))
    parser.add_argument("--manifest", type=Path)
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


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, dict):
        return {str(key): json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def profile_descriptor(profile: tuple[str, tuple[float, ...]]) -> dict[str, Any]:
    return {"family": profile[0], "parameters": list(profile[1])}


def implementation_hashes(root: Path) -> dict[str, str]:
    return {
        "runner": sha256_file(Path(__file__).resolve()),
        "protocol": sha256_file((root / PROTOCOL_PATH).resolve(strict=True)),
        "common_pole_core": sha256_file((root / POLE_CORE_PATH).resolve(strict=True)),
    }


def expected_manifest(root: Path) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "implementation_sha256": implementation_hashes(root),
        "fixture": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "transfer_samples": TRANSFER_SAMPLES,
            "analysis_fft_samples": ANALYSIS_FFT_SAMPLES,
            "contact_count": CONTACT_COUNT,
            "truth_frequencies_hz": TRUTH_FREQUENCIES_HZ.tolist(),
            "truth_decays_per_second": TRUTH_DECAYS_PER_SECOND.tolist(),
            "transfer_peak": TRANSFER_PEAK,
            "residue_magnitude_rule": (
                "(0.38 + 0.035*((5*contact + 7*mode) mod 13))/(1 + 0.04*mode)"
            ),
            "residue_phase_rule": (
                "0.19*contact + 0.23*mode + 0.017*contact*mode radians"
            ),
        },
        "roles": {
            "fit": [profile_descriptor(value) for value in FIT_PROFILES],
            "development": [
                profile_descriptor(value) for value in DEVELOPMENT_PROFILES
            ],
            "holdout": [profile_descriptor(value) for value in HOLDOUT_PROFILES],
            "identity": profile_descriptor(IDENTITY_PROFILE),
            "seeds": ROLE_SEEDS,
            "order": ["fit", "development", "holdout"],
        },
        "observation": {
            "force_noise_peak_ratio": FORCE_NOISE_PEAK_RATIO,
            "response_noise_rms_ratio": RESPONSE_NOISE_RMS_RATIO,
            "room_tail_rms_ratio": ROOM_TAIL_RMS_RATIO,
            "room_tail_start": ROOM_TAIL_START,
            "room_tail_pole": ROOM_TAIL_POLE,
            "room_tail_decay_per_second": ROOM_TAIL_DECAY_PER_SECOND,
        },
        "candidate": {
            "estimator": "pooled_H1_regularized_spectral_deconvolution",
            "minimum_analysis_hz": MINIMUM_ANALYSIS_HZ,
            "maximum_analysis_hz": MAXIMUM_ANALYSIS_HZ,
            "conditioning_floor": CONDITIONING_FLOOR,
            "coherence_floor": COHERENCE_FLOOR,
            "regularization_ratio": REGULARIZATION_RATIO,
            "mask_transition_bins": MASK_TRANSITION_BINS,
            "mode_discovery": {
                "window": "blackmanharris",
                "window_samples": GABOR_WINDOW_SAMPLES,
                "hop_samples": GABOR_HOP_SAMPLES,
                "frame_count": GABOR_FRAME_COUNT,
                "region_floor_db": GABOR_REGION_FLOOR_DB,
                "neighborhood_bins": GABOR_NEIGHBORHOOD_BINS,
                "pencil_lags": pole_core.PENCIL_LAGS,
                "maximum_order": pole_core.MAXIMUM_ORDER,
                "minimum_order_score_margin": MINIMUM_ORDER_SCORE_MARGIN,
                "duplicate_frequency_hz": DUPLICATE_FREQUENCY_HZ,
                "mode_energy_floor_db": MODE_ENERGY_FLOOR_DB,
            },
            "controls": [
                "equal_weight_direct_division",
                "shortest_response_impulse_assumption",
                "input_ignorant_raw_output_modal",
            ],
        },
        "corruptions": {
            "weak_excitation": profile_descriptor(("hann", (1025.0,))),
            "low_coherence_interference_rms_ratio": 0.75,
            "missing_second_impact": {
                "profile": profile_descriptor(("half_sine", (19.0,))),
                "gain": 0.8,
                "delays_samples": [401, 613, 887, 1151],
            },
        },
        "gates": GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "synthetic_only": True,
            "network_allowed": False,
            "real_payload_allowed": False,
            "holdout_after_development_failure_allowed": False,
            "threshold_tuning_after_freeze_allowed": False,
            "quality_admission_runtime_credit_allowed": False,
            "outputs_must_be_external": True,
        },
        "stop_rule": (
            "PASS_KNOWN_TRUTH_FRF opens only B2 zero-decode source feasibility; "
            "any valid gate failure rejects this estimator revision"
        ),
    }


def prepare_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise OracleError(f"output must remain outside repository: {resolved}")
    if resolved.exists():
        if not resolved.is_dir() or any(resolved.iterdir()):
            raise OracleError(f"output must be absent or empty: {resolved}")
    else:
        resolved.mkdir()
    return resolved


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise OracleError(f"{label} must be an external regular file: {resolved}")
    return resolved


def load_manifest(root: Path, path: Path) -> tuple[bytes, dict[str, Any]]:
    resolved = require_external_file(root, path, "B1 manifest")
    payload = resolved.read_bytes()
    if len(payload) > 256 * 1024:
        raise OracleError("B1 manifest exceeds 256 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleError("cannot parse B1 manifest") from error
    if payload != canonical_json(value):
        raise OracleError("B1 manifest is not canonical JSON")
    if value != expected_manifest(root):
        raise OracleError("B1 manifest or implementation lineage changed")
    return payload, value


def write_atomic(path: Path, payload: bytes) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_bytes(payload)
    temporary.replace(path)


def write_npy(path: Path, values: np.ndarray) -> str:
    temporary = path.with_suffix(path.suffix + ".tmp")
    with temporary.open("wb") as handle:
        np.save(handle, np.ascontiguousarray(values), allow_pickle=False)
    temporary.replace(path)
    return sha256_file(path)


def force_profile(profile: tuple[str, tuple[float, ...]]) -> np.ndarray:
    family, parameters = profile
    count = int(parameters[0])
    if count <= 0 or float(count) != parameters[0]:
        raise OracleError("force profile sample count must be a positive integer")
    if family == "half_sine":
        index = np.arange(count, dtype=np.float64)
        active = np.sin(math.pi * (index + 0.5) / count)
    elif family == "hann":
        active = np.hanning(count + 2)[1:-1]
    elif family == "beta":
        if len(parameters) != 3:
            raise OracleError("beta force requires count, a and b")
        x = (np.arange(count, dtype=np.float64) + 0.5) / count
        active = np.power(x, parameters[1]) * np.power(1.0 - x, parameters[2])
    else:
        raise OracleError(f"unknown force profile: {family}")
    total = float(np.sum(active))
    if not math.isfinite(total) or total <= 0.0:
        raise OracleError("force profile has invalid sum")
    result = np.zeros(TRANSFER_SAMPLES, dtype=np.float64)
    result[:count] = active / total
    return result


def exact_transfers() -> np.ndarray:
    time = np.arange(TRANSFER_SAMPLES, dtype=np.float64) / SAMPLE_RATE_HZ
    transfers = np.zeros((CONTACT_COUNT, TRANSFER_SAMPLES), dtype=np.float64)
    for contact in range(CONTACT_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
        ):
            gain = (0.38 + 0.035 * ((5 * contact + 7 * mode) % 13)) / (
                1.0 + 0.04 * mode
            )
            phase = 0.19 * contact + 0.23 * mode + 0.017 * contact * mode
            transfers[contact] += (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
    peak = float(np.max(np.abs(transfers)))
    if not math.isfinite(peak) or peak <= 0.0:
        raise OracleError("exact transfer fixture has invalid peak")
    transfers *= TRANSFER_PEAK / peak
    return transfers


def fft_convolve(force: np.ndarray, transfer_spectrum: np.ndarray) -> np.ndarray:
    spectrum = np.fft.rfft(force, n=ANALYSIS_FFT_SAMPLES)
    return np.fft.irfft(
        spectrum * transfer_spectrum, n=ANALYSIS_FFT_SAMPLES
    )[:TRANSFER_SAMPLES]


def child_generators(seed: int, count: int) -> list[np.random.Generator]:
    return [
        np.random.Generator(np.random.PCG64(child))
        for child in np.random.SeedSequence(seed).spawn(count)
    ]


def room_tail(generator: np.random.Generator, target_rms: float) -> np.ndarray:
    excitation = generator.standard_normal(TRANSFER_SAMPLES - ROOM_TAIL_START)
    coloured = signal.lfilter([1.0], [1.0, -ROOM_TAIL_POLE], excitation)
    time = np.arange(len(coloured), dtype=np.float64) / SAMPLE_RATE_HZ
    coloured *= np.exp(-ROOM_TAIL_DECAY_PER_SECOND * time)
    result = np.zeros(TRANSFER_SAMPLES, dtype=np.float64)
    result[ROOM_TAIL_START:] = coloured
    observed_rms = math.sqrt(float(np.mean(result * result)))
    if not math.isfinite(observed_rms) or observed_rms <= 0.0:
        raise OracleError("room-tail generator returned zero RMS")
    return result * (target_rms / observed_rms)


def generate_clean_role(
    role: str,
    profiles: tuple[tuple[str, tuple[float, ...]], ...],
    transfers: np.ndarray,
) -> dict[str, Any]:
    trial_count = CONTACT_COUNT * len(profiles)
    seeds = ROLE_SEEDS[role]
    force_rngs = child_generators(seeds["force"], trial_count)
    response_rngs = child_generators(seeds["response"], trial_count)
    room_rngs = child_generators(seeds["room"], trial_count)
    transfer_spectra = np.fft.rfft(transfers, n=ANALYSIS_FFT_SAMPLES, axis=1)
    trials = []
    for contact in range(CONTACT_COUNT):
        for profile_index, profile in enumerate(profiles):
            trial_index = contact * len(profiles) + profile_index
            exact_force = force_profile(profile)
            clean_response = fft_convolve(exact_force, transfer_spectra[contact])
            response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
            if response_rms <= 0.0 or not math.isfinite(response_rms):
                raise OracleError("clean response has invalid RMS")
            observed_force = exact_force + force_rngs[trial_index].normal(
                0.0,
                FORCE_NOISE_PEAK_RATIO * float(np.max(exact_force)),
                TRANSFER_SAMPLES,
            )
            observed_response = clean_response + response_rngs[trial_index].normal(
                0.0,
                RESPONSE_NOISE_RMS_RATIO * response_rms,
                TRANSFER_SAMPLES,
            )
            observed_response += room_tail(
                room_rngs[trial_index], ROOM_TAIL_RMS_RATIO * response_rms
            )
            trials.append(
                {
                    "contact": contact,
                    "profile_index": profile_index,
                    "profile": profile_descriptor(profile),
                    "exact_force": exact_force,
                    "observed_force": observed_force,
                    "clean_response": clean_response,
                    "observed_response": observed_response,
                }
            )
    return {"role": role, "profiles": profiles, "trials": trials}


def frequency_band(maximum_hz: float = MAXIMUM_ANALYSIS_HZ) -> np.ndarray:
    frequencies = np.fft.rfftfreq(ANALYSIS_FFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    return (frequencies >= MINIMUM_ANALYSIS_HZ) & (frequencies <= maximum_hz)


def taper_mask(mask: np.ndarray) -> np.ndarray:
    weights = mask.astype(np.float64)
    padded = np.pad(mask.astype(np.int8), (1, 1))
    changes = np.flatnonzero(np.diff(padded))
    for start, stop in changes.reshape(-1, 2):
        length = stop - start
        edge = min(MASK_TRANSITION_BINS, length // 2)
        if edge <= 0:
            continue
        ramp = np.sin(
            math.pi * (np.arange(edge, dtype=np.float64) + 1.0) / (2.0 * (edge + 1.0))
        ) ** 2
        weights[start : start + edge] *= ramp
        weights[stop - edge : stop] *= ramp[::-1]
    return weights


def estimate_transfer(trials: list[dict[str, Any]]) -> dict[str, np.ndarray]:
    x = np.stack(
        [np.fft.rfft(item["observed_force"], n=ANALYSIS_FFT_SAMPLES) for item in trials]
    )
    y = np.stack(
        [
            np.fft.rfft(item["observed_response"], n=ANALYSIS_FFT_SAMPLES)
            for item in trials
        ]
    )
    sxx = np.sum(np.abs(x) ** 2, axis=0)
    syx = np.sum(y * np.conj(x), axis=0)
    syy = np.sum(np.abs(y) ** 2, axis=0)
    band = frequency_band()
    maximum = float(np.max(sxx[band]))
    if maximum <= 0.0 or not math.isfinite(maximum):
        raise OracleError("force spectrum has invalid power")
    conditioning = sxx / maximum
    coherence = np.zeros_like(sxx, dtype=np.float64)
    denominator = sxx * syy
    nonzero = denominator > 0.0
    coherence[nonzero] = np.minimum(
        1.0, np.abs(syx[nonzero]) ** 2 / denominator[nonzero]
    )
    valid = band & (conditioning >= CONDITIONING_FLOOR) & (coherence >= COHERENCE_FLOOR)
    weights = taper_mask(valid)
    transfer = syx / (sxx + REGULARIZATION_RATIO * maximum)
    transfer *= weights

    eligible = np.abs(x) ** 2 >= CONDITIONING_FLOOR * np.max(np.abs(x) ** 2, axis=1)[:, None]
    divisions = np.zeros_like(y)
    np.divide(y, x, out=divisions, where=eligible)
    counts = np.sum(eligible, axis=0)
    direct = np.zeros_like(syx)
    np.divide(np.sum(divisions, axis=0), counts, out=direct, where=counts > 0)
    direct_valid = band & (counts > 0)
    direct *= taper_mask(direct_valid)
    return {
        "transfer": transfer,
        "valid": valid,
        "conditioning": conditioning,
        "coherence": coherence,
        "direct": direct,
        "direct_valid": direct_valid,
    }


def fit_estimators(fit_role: dict[str, Any]) -> dict[str, Any]:
    by_contact = []
    for contact in range(CONTACT_COUNT):
        trials = [item for item in fit_role["trials"] if item["contact"] == contact]
        if len(trials) != len(FIT_PROFILES):
            raise OracleError("fit contact trial count changed")
        by_contact.append(estimate_transfer(trials))
    h1 = np.stack([item["transfer"] for item in by_contact])
    valid = np.stack([item["valid"] for item in by_contact])
    conditioning = np.stack([item["conditioning"] for item in by_contact])
    coherence = np.stack([item["coherence"] for item in by_contact])
    direct = np.stack([item["direct"] for item in by_contact])

    shortest_index = min(
        range(len(FIT_PROFILES)), key=lambda index: FIT_PROFILES[index][1][0]
    )
    impulse = []
    raw_outputs = []
    band = frequency_band()
    band_weights = taper_mask(band)
    for contact in range(CONTACT_COUNT):
        trials = [item for item in fit_role["trials"] if item["contact"] == contact]
        shortest = trials[shortest_index]["observed_response"]
        impulse.append(
            np.fft.rfft(shortest, n=ANALYSIS_FFT_SAMPLES) * band_weights
        )
        raw_outputs.append(
            np.mean(np.stack([item["observed_response"] for item in trials]), axis=0)
        )
    impulse_spectra = np.stack(impulse)
    raw_modes = identify_modes(np.stack(raw_outputs))
    raw_modal_spectra = np.fft.rfft(
        raw_modes["reconstruction"], n=ANALYSIS_FFT_SAMPLES, axis=1
    ) * band_weights[None, :]
    return {
        "h1": h1,
        "valid": valid,
        "conditioning": conditioning,
        "coherence": coherence,
        "direct": direct,
        "impulse": impulse_spectra,
        "raw_modal": raw_modal_spectra,
        "raw_mode_report": raw_modes,
    }


def gabor_coefficients(channels: np.ndarray) -> np.ndarray:
    shifted = np.zeros((len(channels), MODE_ANALYSIS_SAMPLES), dtype=np.float64)
    retained = MODE_ANALYSIS_SAMPLES - MODE_ONSET_SAMPLE
    shifted[:, MODE_ONSET_SAMPLE:] = channels[:, :retained]
    result = []
    for channel in shifted:
        _, _, coefficients = signal.stft(
            channel,
            fs=SAMPLE_RATE_HZ,
            window="blackmanharris",
            nperseg=GABOR_WINDOW_SAMPLES,
            noverlap=GABOR_WINDOW_SAMPLES - GABOR_HOP_SAMPLES,
            nfft=GABOR_WINDOW_SAMPLES,
            boundary=None,
            padded=False,
            return_onesided=True,
        )
        result.append(coefficients[:, :GABOR_FRAME_COUNT])
    return np.stack(result, axis=2)


def discover_modes(coefficients: np.ndarray) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    energy = np.sum(np.abs(coefficients) ** 2, axis=(1, 2))
    bin_hz = SAMPLE_RATE_HZ / GABOR_WINDOW_SAMPLES
    first = math.ceil(MINIMUM_ANALYSIS_HZ / bin_hz)
    last = math.floor(MAXIMUM_ANALYSIS_HZ / bin_hz)
    maximum = max(float(np.max(energy[first : last + 1])), 1.0e-300)
    regions = []
    for index in range(first, last + 1):
        relative_db = 10.0 * math.log10(max(float(energy[index]), 1.0e-300) / maximum)
        if (
            energy[index] > energy[index - 1]
            and energy[index] >= energy[index + 1]
            and relative_db >= GABOR_REGION_FLOOR_DB
        ):
            regions.append(
                {"bin": index, "frequency_hz": index * bin_hz, "relative_energy_db": relative_db}
            )
    analysis_bins = sorted(
        {
            index
            for region in regions
            for index in range(
                region["bin"] - GABOR_NEIGHBORHOOD_BINS,
                region["bin"] + GABOR_NEIGHBORHOOD_BINS + 1,
            )
            if first <= index <= last
        }
    )
    raw = []
    analyses = []
    for index in analysis_bins:
        analysis = pole_core.estimate_common_poles(coefficients[index], index * bin_hz)
        accepted = analysis["order_score_margin"] >= MINIMUM_ORDER_SCORE_MARGIN
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
        if not groups or item["frequency_hz"] - groups[-1][-1]["frequency_hz"] > DUPLICATE_FREQUENCY_HZ:
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
    time = np.arange(TRANSFER_SAMPLES, dtype=np.float64) / SAMPLE_RATE_HZ
    if not clusters:
        return {
            "discovery": discovery,
            "modes": [],
            "reconstruction": np.zeros_like(channels),
            "coefficients": np.zeros((0, CONTACT_COUNT), dtype=np.float64),
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
        if relative_db < MODE_ENERGY_FLOOR_DB:
            continue
        representative = cluster["representative"]
        residues = []
        for contact in range(CONTACT_COUNT):
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
    return {
        "discovery": discovery,
        "modes": retained,
        "reconstruction": reconstruction,
        "coefficients": fitted,
    }


def match_truth(modes: list[dict[str, Any]]) -> dict[str, Any]:
    pairs = []
    for candidate_index, mode in enumerate(modes):
        for truth_index, frequency in enumerate(TRUTH_FREQUENCIES_HZ):
            error = abs(mode["frequency_hz"] - frequency)
            if error <= MODE_MATCH_RADIUS_HZ:
                pairs.append((error, candidate_index, truth_index))
    pairs.sort()
    assignments: list[int | None] = [None] * len(modes)
    used = [False] * MODE_COUNT
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
                "decay_error_per_second": abs(mode["decay_per_second"] - expected_decay),
                "relative_decay_error": abs(mode["decay_per_second"] - expected_decay)
                / expected_decay,
            }
        )
    return {
        "truth_count": MODE_COUNT,
        "candidate_count": len(modes),
        "truth_match_count": len(comparisons),
        "false_positive_count": len(modes) - len(comparisons),
        "comparisons": comparisons,
    }


def normalized_rms_error(expected: np.ndarray, observed: np.ndarray) -> float:
    denominator = math.sqrt(float(np.mean(expected * expected)))
    if denominator <= 0.0 or not math.isfinite(denominator):
        raise OracleError("NRMSE reference has invalid RMS")
    return math.sqrt(float(np.mean((expected - observed) ** 2))) / denominator


def log_spectrum_rmse_db(expected: np.ndarray, observed: np.ndarray) -> float:
    expected_spectrum = np.abs(np.fft.rfft(expected, n=ANALYSIS_FFT_SAMPLES))
    observed_spectrum = np.abs(np.fft.rfft(observed, n=ANALYSIS_FFT_SAMPLES))
    band = frequency_band()
    maximum = max(float(np.max(expected_spectrum[band])), 1.0e-300)
    active = band & (expected_spectrum >= maximum * 1.0e-3)
    if not np.any(active):
        raise OracleError("log-spectrum metric has no active bins")
    expected_db = 20.0 * np.log10(np.maximum(expected_spectrum[active], maximum * 1.0e-8))
    observed_db = 20.0 * np.log10(np.maximum(observed_spectrum[active], maximum * 1.0e-8))
    offset = float(np.mean(expected_db - observed_db))
    return math.sqrt(float(np.mean((expected_db - (observed_db + offset)) ** 2)))


def coverage(valid: np.ndarray, maximum_hz: float) -> float:
    band = frequency_band(maximum_hz)
    return float(np.mean(valid[band]))


def evaluate_held(
    role: dict[str, Any], estimators: dict[str, Any], modes: dict[str, Any]
) -> dict[str, Any]:
    metrics = []
    for trial in role["trials"]:
        contact = trial["contact"]
        force_spectrum = np.fft.rfft(trial["exact_force"], n=ANALYSIS_FFT_SAMPLES)
        predictions = {}
        for name in ("h1", "direct", "impulse", "raw_modal"):
            predictions[name] = np.fft.irfft(
                force_spectrum * estimators[name][contact], n=ANALYSIS_FFT_SAMPLES
            )[:TRANSFER_SAMPLES]
        metrics.append(
            {
                "contact": contact,
                "profile_index": trial["profile_index"],
                "nrmse": {
                    name: normalized_rms_error(trial["clean_response"], prediction)
                    for name, prediction in predictions.items()
                },
                "h1_log_spectrum_rmse_db": log_spectrum_rmse_db(
                    trial["clean_response"], predictions["h1"]
                ),
            }
        )
    means = {
        name: float(np.mean([item["nrmse"][name] for item in metrics]))
        for name in ("h1", "direct", "impulse", "raw_modal")
    }
    candidate_values = [item["nrmse"]["h1"] for item in metrics]
    spectrum_values = [item["h1_log_spectrum_rmse_db"] for item in metrics]
    truth = match_truth(modes["modes"])
    frequency_errors = [item["frequency_error_hz"] for item in truth["comparisons"]]
    damping_errors = [item["decay_error_per_second"] for item in truth["comparisons"]]
    relative_errors = [item["relative_decay_error"] for item in truth["comparisons"]]
    coverages = [coverage(value, COVERAGE_MAXIMUM_HZ) for value in estimators["valid"]]
    residue_nonzero = all(
        all(item["magnitude"] > 0.0 for item in mode["residues"])
        for mode in modes["modes"]
    )
    return {
        "role": role["role"],
        "trial_count": len(metrics),
        "trials": metrics,
        "mean_nrmse": means,
        "maximum_h1_nrmse": max(candidate_values),
        "mean_h1_log_spectrum_rmse_db": float(np.mean(spectrum_values)),
        "candidate_over_impulse_nrmse": means["h1"] / means["impulse"],
        "candidate_over_raw_modal_nrmse": means["h1"] / means["raw_modal"],
        "candidate_over_direct_nrmse": means["h1"] / means["direct"],
        "valid_coverage_by_contact": coverages,
        "minimum_valid_coverage": min(coverages),
        "truth": truth,
        "maximum_frequency_error_hz": max(frequency_errors, default=math.inf),
        "maximum_decay_error_per_second": max(damping_errors, default=math.inf),
        "maximum_relative_decay_error": max(relative_errors, default=math.inf),
        "all_residues_nonzero": residue_nonzero,
    }


def evaluate_identity(transfers: np.ndarray) -> dict[str, Any]:
    impulse = np.zeros(TRANSFER_SAMPLES, dtype=np.float64)
    impulse[0] = 1.0
    exact = np.fft.rfft(transfers[0], n=ANALYSIS_FFT_SAMPLES)
    observed = np.fft.rfft(impulse, n=ANALYSIS_FFT_SAMPLES) * exact
    recovered = observed / np.fft.rfft(impulse, n=ANALYSIS_FFT_SAMPLES)
    held = force_profile(IDENTITY_PROFILE)
    expected = fft_convolve(held, exact)
    actual = fft_convolve(held, recovered)
    return {"nrmse": normalized_rms_error(expected, actual), "finite": bool(np.all(np.isfinite(actual)))}


def corruption_trials(
    kind: str, role_offset: int, transfer: np.ndarray
) -> list[dict[str, Any]]:
    repeat_count = 4
    seeds = ROLE_SEEDS["corruption"]
    force_rngs = child_generators(seeds["force"] + role_offset, repeat_count)
    response_rngs = child_generators(seeds["response"] + role_offset, repeat_count)
    room_rngs = child_generators(seeds["room"] + role_offset, repeat_count)
    transfer_spectrum = np.fft.rfft(transfer, n=ANALYSIS_FFT_SAMPLES)
    if kind == "weak_excitation":
        base = force_profile(("hann", (1025.0,)))
    else:
        base = force_profile(("half_sine", (19.0,)))
    trials = []
    delays = (401, 613, 887, 1151)
    for index in range(repeat_count):
        true_force = base.copy()
        if kind == "missing_second_impact":
            delayed = np.zeros_like(base)
            delay = delays[index]
            active = force_profile(("half_sine", (19.0,)))[:19]
            delayed[delay : delay + len(active)] = 0.8 * active
            true_force += delayed
        clean_response = fft_convolve(true_force, transfer_spectrum)
        response_rms = math.sqrt(float(np.mean(clean_response * clean_response)))
        observed_force = base + force_rngs[index].normal(
            0.0, FORCE_NOISE_PEAK_RATIO * float(np.max(base)), TRANSFER_SAMPLES
        )
        observed_response = clean_response + response_rngs[index].normal(
            0.0, RESPONSE_NOISE_RMS_RATIO * response_rms, TRANSFER_SAMPLES
        )
        observed_response += room_tail(room_rngs[index], ROOM_TAIL_RMS_RATIO * response_rms)
        if kind == "low_coherence":
            interference = response_rngs[index].standard_normal(TRANSFER_SAMPLES)
            interference = signal.lfilter([1.0], [1.0, -0.91], interference)
            interference_rms = math.sqrt(float(np.mean(interference * interference)))
            observed_response += interference * (0.75 * response_rms / interference_rms)
        trials.append(
            {
                "contact": 0,
                "profile_index": index,
                "observed_force": observed_force,
                "observed_response": observed_response,
                "clean_response": clean_response,
                "exact_force": true_force,
            }
        )
    return trials


def evaluate_corruption(kind: str, role_offset: int, transfer: np.ndarray) -> dict[str, Any]:
    trials = corruption_trials(kind, role_offset, transfer)
    estimate = estimate_transfer(trials)
    band = frequency_band()
    high_band = frequency_band(MAXIMUM_ANALYSIS_HZ) & ~frequency_band(4_000.0)
    median_coherence = float(np.median(estimate["coherence"][band]))
    valid_coverage = float(np.mean(estimate["valid"][band]))
    high_coverage = float(np.mean(estimate["valid"][high_band]))
    residuals = []
    for trial in trials:
        predicted = np.fft.irfft(
            np.fft.rfft(trial["observed_force"], n=ANALYSIS_FFT_SAMPLES)
            * estimate["transfer"],
            n=ANALYSIS_FFT_SAMPLES,
        )[:TRANSFER_SAMPLES]
        residuals.append(normalized_rms_error(trial["observed_response"], predicted))
    median_residual = float(np.median(residuals))
    if kind == "weak_excitation" and high_coverage <= GATES["maximum_weak_high_band_coverage"]:
        decision = "OOD_WEAK_EXCITATION"
    elif kind == "low_coherence" and (
        median_coherence <= GATES["maximum_low_coherence_median"]
        or valid_coverage <= GATES["maximum_low_coherence_coverage"]
    ):
        decision = "OOD_LOW_COHERENCE"
    elif kind == "missing_second_impact" and median_residual >= GATES["minimum_double_impact_residual_nrmse"]:
        decision = "OOD_MODEL_MISMATCH"
    else:
        decision = "INVALIDLY_ADMITTED"
    return {
        "kind": kind,
        "repeat_count": len(trials),
        "median_band_coherence": median_coherence,
        "valid_coverage": valid_coverage,
        "high_band_valid_coverage": high_coverage,
        "median_reconstruction_nrmse": median_residual,
        "decision": decision,
        "admitted_mode_count": 0 if decision != "INVALIDLY_ADMITTED" else None,
    }


def check(name: str, observed: Any, relation: str, threshold: Any) -> dict[str, Any]:
    if relation == "<=":
        passed = observed <= threshold
    elif relation == ">=":
        passed = observed >= threshold
    elif relation == "==":
        passed = observed == threshold
    else:
        raise OracleError(f"unknown gate relation: {relation}")
    return {"name": name, "observed": observed, "relation": relation, "threshold": threshold, "passed": bool(passed)}


def evaluation_gates(prefix: str, value: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        check(f"{prefix}.minimum_valid_coverage", value["minimum_valid_coverage"], ">=", GATES["minimum_valid_coverage"]),
        check(f"{prefix}.truth_match_count", value["truth"]["truth_match_count"], "==", GATES["truth_match_count"]),
        check(f"{prefix}.false_positive_count", value["truth"]["false_positive_count"], "==", GATES["false_positive_count"]),
        check(f"{prefix}.maximum_frequency_error_hz", value["maximum_frequency_error_hz"], "<=", GATES["maximum_frequency_error_hz"]),
        check(f"{prefix}.maximum_decay_error_per_second", value["maximum_decay_error_per_second"], "<=", GATES["maximum_decay_error_per_second"]),
        check(f"{prefix}.maximum_relative_decay_error", value["maximum_relative_decay_error"], "<=", GATES["maximum_relative_decay_error"]),
        check(f"{prefix}.mean_held_nrmse", value["mean_nrmse"]["h1"], "<=", GATES["maximum_mean_held_nrmse"]),
        check(f"{prefix}.maximum_held_nrmse", value["maximum_h1_nrmse"], "<=", GATES["maximum_held_nrmse"]),
        check(f"{prefix}.mean_log_spectrum_rmse_db", value["mean_h1_log_spectrum_rmse_db"], "<=", GATES["maximum_mean_log_spectrum_rmse_db"]),
        check(f"{prefix}.impulse_nrmse_ratio", value["candidate_over_impulse_nrmse"], "<=", GATES["maximum_impulse_nrmse_ratio"]),
        check(f"{prefix}.raw_modal_nrmse_ratio", value["candidate_over_raw_modal_nrmse"], "<=", GATES["maximum_raw_modal_nrmse_ratio"]),
        check(f"{prefix}.direct_division_nrmse_ratio", value["candidate_over_direct_nrmse"], "<=", GATES["maximum_direct_division_nrmse_ratio"]),
        check(f"{prefix}.all_residues_nonzero", value["all_residues_nonzero"], "==", True),
    ]


def common_report(manifest_payload: bytes, manifest: dict[str, Any]) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_payload),
        "implementation_sha256": manifest["implementation_sha256"],
        "runtime": manifest["runtime"],
        "network_requests": 0,
        "real_payload_bytes_read": 0,
        "product_credit": {
            "real_quality": False,
            "validator": False,
            "atlas": False,
            "runtime": False,
        },
    }


def freeze(root: Path, output: Path) -> None:
    manifest = expected_manifest(root)
    write_atomic(output / "manifest.json", canonical_json(manifest))


def preflight(manifest_payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    report = {
        "schema": REPORT_SCHEMAS["preflight"],
        **common_report(manifest_payload, manifest),
        "decision": "B1_ORACLE_FROZEN",
        "fixture_samples_generated": 0,
        "fit_trials_evaluated": 0,
        "development_trials_evaluated": 0,
        "holdout_trials_evaluated": 0,
        "gate": {"passed": True, "checks": []},
    }
    write_atomic(output / "report.json", canonical_json(report))


def finite_tree(value: Any) -> bool:
    if isinstance(value, dict):
        return all(finite_tree(item) for item in value.values())
    if isinstance(value, (list, tuple)):
        return all(finite_tree(item) for item in value)
    if isinstance(value, (float, np.floating)):
        return math.isfinite(float(value))
    return True


def run_oracle(manifest_payload: bytes, manifest: dict[str, Any], output: Path) -> None:
    transfers = exact_transfers()
    fit_role = generate_clean_role("fit", FIT_PROFILES, transfers)
    estimators = fit_estimators(fit_role)
    candidate_impulses = np.fft.irfft(
        estimators["h1"], n=ANALYSIS_FFT_SAMPLES, axis=1
    )[:, :TRANSFER_SAMPLES]
    modes = identify_modes(candidate_impulses)
    development_role = generate_clean_role("development", DEVELOPMENT_PROFILES, transfers)
    development = evaluate_held(development_role, estimators, modes)
    identity = evaluate_identity(transfers)
    development_corruptions = {
        "weak_excitation": evaluate_corruption("weak_excitation", 100, transfers[0]),
        "low_coherence": evaluate_corruption("low_coherence", 200, transfers[0]),
    }
    checks = evaluation_gates("development", development)
    checks.extend(
        [
            check("identity.nrmse", identity["nrmse"], "<=", GATES["maximum_identity_nrmse"]),
            check("identity.finite", identity["finite"], "==", True),
            check("development.weak_excitation_decision", development_corruptions["weak_excitation"]["decision"], "==", "OOD_WEAK_EXCITATION"),
            check("development.weak_admitted_modes", development_corruptions["weak_excitation"]["admitted_mode_count"], "==", 0),
            check("development.low_coherence_decision", development_corruptions["low_coherence"]["decision"], "==", "OOD_LOW_COHERENCE"),
            check("development.low_coherence_admitted_modes", development_corruptions["low_coherence"]["admitted_mode_count"], "==", 0),
        ]
    )
    development_passed = all(item["passed"] for item in checks)

    holdout = None
    holdout_corruptions = None
    if development_passed:
        holdout_role = generate_clean_role("holdout", HOLDOUT_PROFILES, transfers)
        holdout = evaluate_held(holdout_role, estimators, modes)
        holdout_corruptions = {
            "weak_excitation": evaluate_corruption("weak_excitation", 300, transfers[0]),
            "missing_second_impact": evaluate_corruption(
                "missing_second_impact", 400, transfers[0]
            ),
        }
        checks.extend(evaluation_gates("holdout", holdout))
        checks.extend(
            [
                check("holdout.weak_excitation_decision", holdout_corruptions["weak_excitation"]["decision"], "==", "OOD_WEAK_EXCITATION"),
                check("holdout.weak_admitted_modes", holdout_corruptions["weak_excitation"]["admitted_mode_count"], "==", 0),
                check("holdout.double_impact_decision", holdout_corruptions["missing_second_impact"]["decision"], "==", "OOD_MODEL_MISMATCH"),
                check("holdout.double_impact_admitted_modes", holdout_corruptions["missing_second_impact"]["admitted_mode_count"], "==", 0),
            ]
        )

    arrays = {
        "h1_transfer.npy": estimators["h1"].astype("<c16"),
        "conditioning.npy": estimators["conditioning"].astype("<f8"),
        "coherence.npy": estimators["coherence"].astype("<f8"),
        "valid_mask.npy": estimators["valid"].astype(np.bool_),
        "modal_reconstruction.npy": modes["reconstruction"].astype("<f8"),
    }
    array_hashes = {name: write_npy(output / name, values) for name, values in arrays.items()}
    mode_records = modes["modes"]
    model = {
        "schema": MODEL_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_payload),
        "array_sha256": array_hashes,
        "modes": mode_records,
        "candidate": manifest["candidate"],
        "product_credit": False,
    }
    if not finite_tree(model):
        raise OracleError("B1 model contains nonfinite values")
    model_payload = canonical_json(model)
    write_atomic(output / "model.json", model_payload)

    passed = all(item["passed"] for item in checks)
    decision = "PASS_KNOWN_TRUTH_FRF" if passed else "REJECT_ESTIMATOR"
    report = {
        "schema": REPORT_SCHEMAS["run"],
        **common_report(manifest_payload, manifest),
        "decision": decision,
        "role_access_order": ["fit", "development"] + (["holdout"] if holdout is not None else []),
        "sample_accounting": {
            "fixture_transfer_samples": int(transfers.size),
            "fit_trials": len(fit_role["trials"]),
            "development_trials": len(development_role["trials"]),
            "holdout_trials": 0 if holdout is None else CONTACT_COUNT * len(HOLDOUT_PROFILES),
            "real_samples": 0,
        },
        "identity": identity,
        "mode_discovery": {
            "region_count": len(modes["discovery"]["regions"]),
            "analysis_bin_count": len(modes["discovery"]["analysis_bins"]),
            "raw_estimate_count": modes["discovery"]["raw_estimate_count"],
            "retained_mode_count": len(mode_records),
            "regions": modes["discovery"]["regions"],
        },
        "development": development,
        "development_corruptions": development_corruptions,
        "holdout": holdout,
        "holdout_corruptions": holdout_corruptions,
        "model_sha256": sha256_bytes(model_payload),
        "array_sha256": array_hashes,
        "gate": {"passed": passed, "checks": checks},
        "next_authorized_step": (
            "B2_ZERO_DECODE_SOURCE_FEASIBILITY" if passed else "NEW_ESTIMATOR_HYPOTHESIS_ONLY"
        ),
    }
    if not finite_tree(report):
        raise OracleError("B1 report contains nonfinite values")
    write_atomic(output / "report.json", canonical_json(report))


def main() -> int:
    arguments = parse_arguments()
    root = repository_root()
    output = prepare_output(root, arguments.output)
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
