#!/usr/bin/env python3
"""Audit the preregistered V44 G0B1c0 fixed-size target on synthetic signals."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

import numpy as np
import scipy
from scipy import ndimage, signal

import physical_sound_v41_c0_disclosed_corpus_v1 as c0

PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c0-synthetic-profile.v1"
)
PROTOCOL_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-g0b1c0-noise-robust-target-protocol.v1"
)
TARGET_SCHEMA = (
    "nextengine.experimental-physical-sound-v44-noise-robust-fixed-target.v1"
)
AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c0-synthetic-audit.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v44-g0b1c0-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v44-g0b1c0-synthetic-target-audit.v1.json"
PROTOCOL_PATH = (
    "lab/profiles/physical-sound-v44-g0b1c0-noise-robust-target-protocol.v1.json"
)
OWNER_PATH = "lab/scripts/physical_sound_v44_g0b1c0_noise_robust_target_v1.py"
C0_OWNER_PATH = "lab/scripts/physical_sound_v41_c0_disclosed_corpus_v1.py"

CLAIM = (
    "PREREGISTERED_SYNTHETIC_ONLY_NOISE_ROBUST_FIXED_SIZE_TARGET_AUDIT / "
    "NO_IETEASY_CORPUS_ROLE_PSEL_MODEL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_"
    "RUNTIME_AUTHORITY"
)
HASH_LENGTH = 64
MAX_JSON_BYTES = 4 * 1024 * 1024
MAX_FILE_BYTES = 4 * 1024 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "candidate_training_authority": False,
    "corpus_materialization_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "synthetic_target_audit_authority": True,
    "validator_calibration_authority": False,
}
ACCESS_POLICY = {
    "candidate_access_allowed": False,
    "disclosed_audio_access_allowed": False,
    "model_access_allowed": False,
    "network_allowed": False,
    "outputs_external": True,
    "protected_access_allowed": False,
    "validator_access_allowed": False,
}
FORBIDDEN_COUNTERS = (
    "candidate_values_read",
    "disclosed_audio_values_read",
    "ieteasy_target_values_read",
    "model_values_read",
    "network_requests",
    "protected_values_read",
    "validator_values_read",
)


class SyntheticTargetError(RuntimeError):
    """G0B1c0 cannot publish a trustworthy synthetic target audit."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--protocol", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(
                value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False
            )
            + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise SyntheticTargetError(
            f"cannot serialize canonical JSON: {error}"
        ) from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SyntheticTargetError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise SyntheticTargetError(f"{context} must be an array")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise SyntheticTargetError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise SyntheticTargetError(f"{context} must be an integer >= {minimum}")
    return value


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise SyntheticTargetError(f"{context} must be a lowercase SHA-256")
    return value


def read_regular(path: Path, context: str, maximum: int = MAX_FILE_BYTES) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise SyntheticTargetError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise SyntheticTargetError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context, MAX_JSON_BYTES)
    try:
        value = require_dict(json.loads(data), context)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SyntheticTargetError(f"{context} is not valid UTF-8 JSON") from error
    if data != canonical_json(value):
        raise SyntheticTargetError(f"{context} must be canonical JSON")
    return data, value


def validate_binding_bytes(data: bytes, binding: dict[str, Any], context: str) -> None:
    if set(binding) != {"bytes", "path", "sha256"}:
        raise SyntheticTargetError(f"{context} binding fields changed")
    require_string(binding["path"], f"{context} path")
    if len(data) != require_int(binding["bytes"], f"{context} bytes", 1):
        raise SyntheticTargetError(f"{context} byte count changed")
    if sha256_bytes(data) != require_hash(binding["sha256"], f"{context} hash"):
        raise SyntheticTargetError(f"{context} hash changed")


def validate_dependencies(profile: dict[str, Any]) -> None:
    root = repository_root()
    seen: set[str] = set()
    for index, raw in enumerate(
        require_list(profile.get("dependency_bindings"), "dependency bindings")
    ):
        binding = require_dict(raw, f"dependency {index}")
        path_text = require_string(binding.get("path"), f"dependency {index} path")
        relative = Path(path_text)
        if relative.is_absolute() or ".." in relative.parts or path_text in seen:
            raise SyntheticTargetError("dependency path is invalid or duplicated")
        seen.add(path_text)
        data = read_regular(root / relative, f"dependency {path_text}")
        validate_binding_bytes(data, binding, f"dependency {path_text}")
    if not {OWNER_PATH, PROTOCOL_PATH, C0_OWNER_PATH}.issubset(seen):
        raise SyntheticTargetError("required dependency binding is missing")


def validate_environment(profile: dict[str, Any]) -> None:
    environment = require_dict(profile.get("environment"), "environment")
    if set(environment) != {"numpy_version", "scipy_version"}:
        raise SyntheticTargetError("environment fields changed")
    if environment["numpy_version"] != np.__version__:
        raise SyntheticTargetError("NumPy version changed")
    if environment["scipy_version"] != scipy.__version__:
        raise SyntheticTargetError("SciPy version changed")


def validate_profile(profile: dict[str, Any]) -> None:
    required = {
        "access_policy",
        "authority",
        "claim",
        "dependency_bindings",
        "environment",
        "expected_controls",
        "profile_id",
        "protocol",
        "schema",
    }
    if set(profile) != required:
        raise SyntheticTargetError("execution profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM:
        raise SyntheticTargetError("execution profile schema or claim changed")
    if profile["authority"] != AUTHORITY or profile["access_policy"] != ACCESS_POLICY:
        raise SyntheticTargetError("execution authority or access policy changed")
    expected = require_dict(profile["expected_controls"], "expected controls")
    if set(expected) != {"fixtures", "scale_mutations"}:
        raise SyntheticTargetError("expected control fields changed")
    require_int(expected["fixtures"], "expected fixtures", 1)
    require_int(expected["scale_mutations"], "expected scale mutations", 1)
    validate_dependencies(profile)
    validate_environment(profile)


def validate_fixture(raw: Any, seen: set[str]) -> dict[str, Any]:
    fixture = require_dict(raw, "synthetic fixture")
    required = {
        "decay_rate_millihertz",
        "fixture_id",
        "impulse_amplitude_ppm",
        "mode_amplitudes_ppm",
        "mode_frequencies_hz",
        "noise_amplitude_ppm",
        "noise_decay_rate_millihertz",
        "noise_seed",
    }
    if set(fixture) != required:
        raise SyntheticTargetError("synthetic fixture fields changed")
    fixture_id = require_string(fixture["fixture_id"], "fixture id")
    if fixture_id in seen:
        raise SyntheticTargetError("synthetic fixture id is duplicated")
    seen.add(fixture_id)
    frequencies = require_list(fixture["mode_frequencies_hz"], "mode frequencies")
    amplitudes = require_list(fixture["mode_amplitudes_ppm"], "mode amplitudes")
    if len(frequencies) != len(amplitudes):
        raise SyntheticTargetError("mode frequencies and amplitudes disagree")
    for frequency in frequencies:
        require_int(frequency, "mode frequency", 1)
    for amplitude in amplitudes:
        require_int(amplitude, "mode amplitude", 0)
    for name in (
        "decay_rate_millihertz",
        "impulse_amplitude_ppm",
        "noise_amplitude_ppm",
        "noise_decay_rate_millihertz",
        "noise_seed",
    ):
        require_int(fixture[name], name, 0)
    return fixture


def validate_protocol(protocol: dict[str, Any]) -> None:
    required = {
        "access_policy",
        "algorithm",
        "authority",
        "claim",
        "decision_policy",
        "profile_id",
        "schema",
        "source_basis",
        "synthetic_fixtures",
    }
    if set(protocol) != required:
        raise SyntheticTargetError("protocol fields changed")
    if protocol["schema"] != PROTOCOL_SCHEMA or protocol["claim"] != CLAIM:
        raise SyntheticTargetError("protocol schema or claim changed")
    if protocol["authority"] != AUTHORITY or protocol["access_policy"] != ACCESS_POLICY:
        raise SyntheticTargetError("protocol authority or access policy changed")
    algorithm = require_dict(protocol["algorithm"], "algorithm")
    if algorithm.get("sample_rate_hz") != c0.CORPUS_POLICY["sample_rate_hz"]:
        raise SyntheticTargetError("sample rate is incompatible with C0")
    if algorithm.get("sample_count") != c0.CORPUS_POLICY["segment_samples"]:
        raise SyntheticTargetError("sample count is incompatible with C0")
    if algorithm.get("analysis_start_sample") != c0.CORPUS_POLICY["pretrigger_samples"]:
        raise SyntheticTargetError("analysis onset is incompatible with C0")
    if algorithm.get("band_count") != 24:
        raise SyntheticTargetError("fixed target band count changed")
    if algorithm.get("target_vector_dimension") != 75:
        raise SyntheticTargetError("fixed target vector dimension changed")
    kernel = require_int(
        algorithm.get("local_log_power_median_bins"), "median kernel", 3
    )
    window = require_int(
        algorithm.get("prominence_window_bins"), "prominence window", 3
    )
    if kernel % 2 == 0 or window % 2 == 0:
        raise SyntheticTargetError("local spectral windows must be odd")
    decision = require_dict(protocol["decision_policy"], "decision policy")
    if decision.get("positive_scale_factors_ppm") != [500000, 2000000]:
        raise SyntheticTargetError("positive scale mutations changed")
    fixtures = require_list(protocol["synthetic_fixtures"], "synthetic fixtures")
    seen: set[str] = set()
    for raw in fixtures:
        validate_fixture(raw, seen)
    if seen != {
        "broadband_noise",
        "clean_three_mode",
        "frequency_shift",
        "modal_plus_noise",
    }:
        raise SyntheticTargetError("synthetic fixture roster changed")
    sources = require_list(protocol["source_basis"], "source basis")
    if len(sources) != 3:
        raise SyntheticTargetError("source basis changed")
    for raw in sources:
        source = require_dict(raw, "source basis row")
        if set(source) != {"bounded_claim", "url"}:
            raise SyntheticTargetError("source basis fields changed")
        require_string(source["bounded_claim"], "bounded claim")
        if not require_string(source["url"], "source URL").startswith("https://"):
            raise SyntheticTargetError("source URL must use HTTPS")


def load_protocol(
    profile: dict[str, Any], protocol_path: Path
) -> tuple[bytes, dict[str, Any]]:
    data, protocol = read_canonical_json(protocol_path, "target protocol")
    binding = require_dict(profile["protocol"], "protocol binding")
    validate_binding_bytes(data, binding, "target protocol")
    if binding["path"] != PROTOCOL_PATH:
        raise SyntheticTargetError("protocol path changed")
    validate_protocol(protocol)
    return data, protocol


def largest_remainder_ppm(values: np.ndarray) -> list[int]:
    if values.ndim != 1 or len(values) == 0 or not np.all(np.isfinite(values)):
        raise SyntheticTargetError("energy apportionment input is invalid")
    if np.any(values < 0):
        raise SyntheticTargetError("energy apportionment input is negative")
    total = float(np.sum(values))
    if total <= 0:
        raise SyntheticTargetError("energy apportionment input is empty")
    scaled = values * (1_000_000.0 / total)
    floors = np.floor(scaled).astype(np.int64)
    remainder = 1_000_000 - int(np.sum(floors))
    order = sorted(
        range(len(values)), key=lambda index: (-(scaled[index] - floors[index]), index)
    )
    for index in order[:remainder]:
        floors[index] += 1
    result = [int(value) for value in floors]
    if sum(result) != 1_000_000:
        raise SyntheticTargetError("energy apportionment did not close")
    return result


def ppm(value: float) -> int:
    if not math.isfinite(value):
        raise SyntheticTargetError("derived target value is non-finite")
    return max(0, min(1_000_000, round(value * 1_000_000)))


def extract_target(samples: np.ndarray, protocol: dict[str, Any]) -> dict[str, Any]:
    algorithm = protocol["algorithm"]
    sample_count = algorithm["sample_count"]
    if (
        not isinstance(samples, np.ndarray)
        or samples.dtype != np.float64
        or samples.ndim != 1
        or len(samples) != sample_count
        or not np.all(np.isfinite(samples))
    ):
        raise SyntheticTargetError("target input must be finite mono float64 C0 PCM")
    peak = float(np.max(np.abs(samples)))
    if peak <= 1e-12:
        raise SyntheticTargetError("target input is silent")
    normalized = samples / peak
    analysis = normalized[algorithm["analysis_start_sample"] :]
    window = algorithm["welch_window_samples"]
    frequencies, power = signal.welch(
        analysis,
        fs=algorithm["sample_rate_hz"],
        window=algorithm["welch_window"],
        nperseg=window,
        noverlap=window - algorithm["welch_hop_samples"],
        detrend=False,
        scaling="spectrum",
    )
    power = np.maximum(power, algorithm["spectral_floor"])
    valid = (frequencies >= algorithm["band_min_hz"]) & (
        frequencies < algorithm["band_max_hz"]
    )
    valid_indices = np.flatnonzero(valid)
    total_power = float(np.sum(power[valid]))
    if total_power <= 0 or not math.isfinite(total_power):
        raise SyntheticTargetError("target spectrum has invalid energy")
    log_power = 10.0 * np.log10(power)
    baseline_db = ndimage.median_filter(
        log_power,
        size=algorithm["local_log_power_median_bins"],
        mode="nearest",
    )
    baseline_power = 10.0 ** (baseline_db / 10.0)
    bin_hz = algorithm["sample_rate_hz"] / window
    local_peaks, properties = signal.find_peaks(
        log_power[valid],
        prominence=algorithm["minimum_peak_prominence_db"],
        distance=max(1, round(algorithm["minimum_peak_distance_hz"] / bin_hz)),
        wlen=algorithm["prominence_window_bins"],
    )
    peak_indices = valid_indices[local_peaks]
    peak_excess_mass = (
        np.maximum(power[peak_indices] - baseline_power[peak_indices], 0.0)
        / total_power
    )
    qualified = peak_indices[
        peak_excess_mass >= algorithm["minimum_peak_excess_mass_ppm"] / 1_000_000.0
    ]
    if len(qualified) > algorithm["maximum_qualified_peaks"]:
        raise SyntheticTargetError("qualified peak resource bound exceeded")

    tonal_mask = np.zeros_like(power, dtype=np.bool_)
    radius = algorithm["peak_capture_radius_bins"]
    for index in qualified:
        start = max(int(valid_indices[0]), int(index) - radius)
        stop = min(int(valid_indices[-1]) + 1, int(index) + radius + 1)
        tonal_mask[start:stop] = True
    tonal_power = np.where(tonal_mask, np.maximum(power - baseline_power, 0.0), 0.0)
    residual_power = np.maximum(power - tonal_power, 0.0)

    edges = np.geomspace(
        algorithm["band_min_hz"],
        algorithm["band_max_hz"],
        algorithm["band_count"] + 1,
    )
    tonal_bands = []
    residual_bands = []
    peak_counts = []
    for low, high in zip(edges[:-1], edges[1:], strict=True):
        band = (frequencies >= low) & (frequencies < high)
        tonal_bands.append(float(np.sum(tonal_power[band])))
        residual_bands.append(float(np.sum(residual_power[band])))
        peak_counts.append(
            int(
                np.count_nonzero(
                    (frequencies[qualified] >= low) & (frequencies[qualified] < high)
                )
            )
        )
    apportioned = largest_remainder_ppm(
        np.asarray(tonal_bands + residual_bands, dtype=np.float64)
    )
    count = algorithm["band_count"]
    tonal_ppm = apportioned[:count]
    residual_ppm = apportioned[count:]
    flatness = float(np.exp(np.mean(np.log(power[valid]))) / np.mean(power[valid]))
    target = {
        "band_edges_millihz": [round(float(value) * 1000.0) for value in edges],
        "qualified_peak_count": len(qualified),
        "qualified_peak_count_by_band": peak_counts,
        "residual_energy_ppm_by_band": residual_ppm,
        "schema": TARGET_SCHEMA,
        "spectral_flatness_ppm": ppm(flatness),
        "tonal_excess_energy_ppm_by_band": tonal_ppm,
        "tonal_excess_mass_ppm": sum(tonal_ppm),
        "vector_dimension": algorithm["target_vector_dimension"],
    }
    if len(target_vector(target)) != algorithm["target_vector_dimension"]:
        raise SyntheticTargetError("target vector dimension changed")
    if sum(tonal_ppm) + sum(residual_ppm) != 1_000_000:
        raise SyntheticTargetError("target energy partition did not close")
    if int(np.sum(peak_counts)) != len(qualified):
        raise SyntheticTargetError("target peak-count partition did not close")
    if len(properties["prominences"]) != len(local_peaks):
        raise SyntheticTargetError("peak prominence output changed")
    return target


def target_vector(target: dict[str, Any]) -> list[int]:
    return [
        *target["tonal_excess_energy_ppm_by_band"],
        *target["residual_energy_ppm_by_band"],
        *target["qualified_peak_count_by_band"],
        target["tonal_excess_mass_ppm"],
        target["spectral_flatness_ppm"],
        target["qualified_peak_count"],
    ]


def synthesize_fixture(
    fixture: dict[str, Any], algorithm: dict[str, Any]
) -> np.ndarray:
    count = algorithm["sample_count"]
    start = algorithm["analysis_start_sample"]
    rate = algorithm["sample_rate_hz"]
    samples = np.zeros(count, dtype=np.float64)
    time = np.arange(count - start, dtype=np.float64) / rate
    mode_envelope = np.exp(-fixture["decay_rate_millihertz"] / 1000.0 * time)
    for frequency, amplitude in zip(
        fixture["mode_frequencies_hz"],
        fixture["mode_amplitudes_ppm"],
        strict=True,
    ):
        samples[start:] += (
            amplitude
            / 1_000_000.0
            * mode_envelope
            * np.sin(2.0 * math.pi * frequency * time)
        )
    if fixture["noise_amplitude_ppm"]:
        generator = np.random.default_rng(fixture["noise_seed"])
        noise_envelope = np.exp(-fixture["noise_decay_rate_millihertz"] / 1000.0 * time)
        samples[start:] += (
            fixture["noise_amplitude_ppm"]
            / 1_000_000.0
            * noise_envelope
            * generator.standard_normal(len(time))
        )
    samples[start] += fixture["impulse_amplitude_ppm"] / 1_000_000.0
    return samples


def dominant_tonal_bands(target: dict[str, Any], count: int) -> list[int]:
    values = target["tonal_excess_energy_ppm_by_band"]
    return sorted(
        sorted(range(len(values)), key=lambda index: (-values[index], index))[:count]
    )


def cosine_ppm(left: list[int], right: list[int]) -> int:
    a = np.asarray(left, dtype=np.float64)
    b = np.asarray(right, dtype=np.float64)
    denominator = float(np.linalg.norm(a) * np.linalg.norm(b))
    return ppm(float(np.dot(a, b) / denominator)) if denominator > 0 else 0


def l1_ppm(left: list[int], right: list[int]) -> int:
    return int(
        np.sum(
            np.abs(np.asarray(left, dtype=np.int64) - np.asarray(right, dtype=np.int64))
        )
    )


def build_outputs(profile: dict[str, Any], protocol: dict[str, Any]) -> dict[str, Any]:
    algorithm = protocol["algorithm"]
    decision = protocol["decision_policy"]
    rows = []
    by_id = {}
    generated_samples = 0
    target_extractions = 0
    for raw in protocol["synthetic_fixtures"]:
        fixture = validate_fixture(raw, set())
        samples = synthesize_fixture(fixture, algorithm)
        generated_samples += len(samples)
        target = extract_target(samples, protocol)
        target_extractions += 1
        target_bytes = canonical_json(target)
        row = {
            "dominant_tonal_bands": dominant_tonal_bands(target, 3),
            "fixture_id": fixture["fixture_id"],
            "target": target,
            "target_bytes": len(target_bytes),
            "target_sha256": sha256_bytes(target_bytes),
        }
        rows.append(row)
        by_id[fixture["fixture_id"]] = (samples, target)

    clean = by_id["clean_three_mode"][1]
    noise = by_id["broadband_noise"][1]
    mixture = by_id["modal_plus_noise"][1]
    shifted = by_id["frequency_shift"][1]
    scale_rows = []
    for fixture_id in decision["scale_identity_fixtures"]:
        samples, baseline = by_id[fixture_id]
        baseline_hash = sha256_bytes(canonical_json(baseline))
        for factor in decision["positive_scale_factors_ppm"]:
            scaled = extract_target(samples * (factor / 1_000_000.0), protocol)
            target_extractions += 1
            scale_rows.append(
                {
                    "fixture_id": fixture_id,
                    "scale_ppm": factor,
                    "target_identical": sha256_bytes(canonical_json(scaled))
                    == baseline_hash,
                }
            )

    clean_tonal = clean["tonal_excess_energy_ppm_by_band"]
    mixture_tonal = mixture["tonal_excess_energy_ppm_by_band"]
    shifted_tonal = shifted["tonal_excess_energy_ppm_by_band"]
    metrics = {
        "clean_qualified_peak_count": clean["qualified_peak_count"],
        "clean_residual_energy_ppm": sum(clean["residual_energy_ppm_by_band"]),
        "clean_tonal_excess_energy_ppm": clean["tonal_excess_mass_ppm"],
        "frequency_shift_clean_tonal_cosine_ppm": cosine_ppm(
            clean_tonal, shifted_tonal
        ),
        "mixture_clean_tonal_cosine_ppm": cosine_ppm(clean_tonal, mixture_tonal),
        "mixture_clean_tonal_l1_ppm": l1_ppm(clean_tonal, mixture_tonal),
        "mixture_qualified_peak_count": mixture["qualified_peak_count"],
        "mixture_tonal_excess_energy_ppm": mixture["tonal_excess_mass_ppm"],
        "noise_qualified_peak_count": noise["qualified_peak_count"],
        "noise_residual_energy_ppm": sum(noise["residual_energy_ppm_by_band"]),
        "noise_spectral_flatness_ppm": noise["spectral_flatness_ppm"],
        "noise_tonal_excess_energy_ppm": noise["tonal_excess_mass_ppm"],
    }
    gates = {
        "clean_mode_recovery": (
            metrics["clean_qualified_peak_count"]
            == decision["clean_expected_qualified_peak_count"]
            and dominant_tonal_bands(clean, 3)
            == decision["clean_expected_dominant_bands"]
            and metrics["clean_tonal_excess_energy_ppm"]
            >= decision["clean_minimum_tonal_excess_energy_ppm"]
            and metrics["clean_residual_energy_ppm"]
            <= decision["clean_maximum_residual_energy_ppm"]
        ),
        "frequency_shift_sensitivity": (
            dominant_tonal_bands(shifted, 3)
            == decision["frequency_shift_expected_dominant_bands"]
            and metrics["frequency_shift_clean_tonal_cosine_ppm"]
            <= decision["frequency_shift_maximum_clean_tonal_cosine_ppm"]
        ),
        "mixture_stability": (
            metrics["mixture_qualified_peak_count"]
            == decision["mixture_expected_qualified_peak_count"]
            and dominant_tonal_bands(mixture, 3)
            == decision["clean_expected_dominant_bands"]
            and metrics["mixture_tonal_excess_energy_ppm"]
            >= decision["mixture_minimum_tonal_excess_energy_ppm"]
            and metrics["mixture_clean_tonal_cosine_ppm"]
            >= decision["mixture_minimum_clean_tonal_cosine_ppm"]
            and metrics["mixture_clean_tonal_l1_ppm"]
            <= decision["mixture_maximum_clean_tonal_l1_ppm"]
        ),
        "noise_rejection": (
            metrics["noise_qualified_peak_count"]
            == decision["noise_expected_qualified_peak_count"]
            and metrics["noise_tonal_excess_energy_ppm"]
            <= decision["noise_maximum_tonal_excess_energy_ppm"]
            and metrics["noise_residual_energy_ppm"]
            >= decision["noise_minimum_residual_energy_ppm"]
            and metrics["noise_spectral_flatness_ppm"]
            >= decision["noise_minimum_spectral_flatness_ppm"]
        ),
        "resource_bounds": all(
            row["target_bytes"] <= decision["target_maximum_canonical_json_bytes"]
            and row["target"]["vector_dimension"]
            == decision["target_vector_dimension_required"]
            for row in rows
        ),
        "scale_invariance": all(row["target_identical"] for row in scale_rows),
    }
    passed = all(gates.values())
    terminal = decision["pass_decision"] if passed else decision["failure_decision"]
    audit = {
        "claim": CLAIM,
        "decision": terminal,
        "gates": gates,
        "metrics": metrics,
        "scale_mutations": scale_rows,
        "schema": AUDIT_SCHEMA,
        "synthetic_targets": rows,
    }
    report = {
        "authority": AUTHORITY,
        "claim": CLAIM,
        "decision": terminal,
        "gates": {
            **gates,
            "corpus_materialization_authorized": False,
            "ieteasy_target_access_authorized": passed,
            "psel_pack_eligible": False,
            "runtime_consumer_allowed": False,
            "training_allowed": False,
        },
        "measured": {
            "synthetic_fixtures": len(rows),
            "synthetic_sample_values": generated_samples,
            "target_extractions": target_extractions,
            "target_vector_dimension": algorithm["target_vector_dimension"],
        },
        "next_action": (
            "bind_this_exact_target_owner_and_protocol_then_apply_unchanged_to_"
            "the_15_frozen_ieteasy_records"
            if passed
            else "redesign_the_fixed_size_target_using_only_synthetic_controls_"
            "without_ieteasy_target_access"
        ),
        "schema": REPORT_SCHEMA,
    }
    access = {
        "claim": CLAIM,
        "counters": {
            "candidate_values_read": 0,
            "disclosed_audio_values_read": 0,
            "ieteasy_target_values_read": 0,
            "model_values_read": 0,
            "network_requests": 0,
            "protected_values_read": 0,
            "synthetic_sample_values_generated": generated_samples,
            "target_extractions": target_extractions,
            "validator_values_read": 0,
        },
        "schema": ACCESS_SCHEMA,
    }
    if (
        len(rows) != profile["expected_controls"]["fixtures"]
        or len(scale_rows) != profile["expected_controls"]["scale_mutations"]
    ):
        raise SyntheticTargetError("synthetic control count changed")
    return {"access-ledger.json": access, "audit.json": audit, "report.json": report}


def is_within(child: Path, parent: Path) -> bool:
    try:
        child.relative_to(parent)
        return True
    except ValueError:
        return False


def prepare_output(output: Path) -> None:
    repository = repository_root().resolve()
    resolved = output.resolve(strict=False)
    if is_within(resolved, repository):
        raise SyntheticTargetError("output must be outside the repository")
    if output.is_symlink():
        raise SyntheticTargetError("output must not be a symlink")
    if output.exists():
        raise SyntheticTargetError("output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)


def publish(
    output: Path,
    outputs: dict[str, Any],
    profile_bytes: bytes,
    protocol_bytes: bytes,
) -> None:
    prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, value in sorted(outputs.items()):
            (staging / name).write_bytes(canonical_json(value))
        (staging / "profile.json").write_bytes(profile_bytes)
        (staging / "protocol.json").write_bytes(protocol_bytes)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def run(profile_path: Path, protocol_path: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "execution profile")
    validate_profile(profile)
    protocol_bytes, protocol = load_protocol(profile, protocol_path)
    outputs = build_outputs(profile, protocol)
    for counter in FORBIDDEN_COUNTERS:
        if outputs["access-ledger.json"]["counters"][counter] != 0:
            raise SyntheticTargetError("forbidden access counter is nonzero")
    publish(output, outputs, profile_bytes, protocol_bytes)


def main() -> int:
    arguments = parse_arguments()
    try:
        run(arguments.profile, arguments.protocol, arguments.output)
    except SyntheticTargetError as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
