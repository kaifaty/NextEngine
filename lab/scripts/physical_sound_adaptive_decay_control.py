#!/usr/bin/env python3
"""Evaluate a source-faithful adaptive modal-decay estimator on known truth."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
from typing import Any

import numpy as np
from scipy import signal
import scipy

import physical_sound_multioutput_decay_control as multioutput
import physical_sound_realimpact_pitcher_calibration as single


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-adaptive-decay-control.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-adaptive-decay-control-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-adaptive-decay-control.report.v1",
}
STUDY_ID = "physical-sound-adaptive-modal-decay-control"
REVISION = "spatial-adaptive-rms-envelope-v1"
CHANNEL_COUNT = 15
SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 384_000
ONSET_SAMPLE = 240
DIRECT_GAIN = 0.04
DELAYED_GAIN = 0.50
DELAY_SAMPLES = [26_400 + 2_400 * (index % 8) for index in range(16)]
NOISE_STANDARD_DEVIATION = 2.0e-6
NOISE_SEED = 20_260_828
FILTER_WIDTH_HZ = 20.0
FILTER_ORDER = 4
ENVELOPE_ETA_SECONDS = 0.01
NOISE_FLOOR_TAIL_FRACTION = 0.10
START_DYNAMIC_FRACTION = 0.90
END_DYNAMIC_FRACTION = 0.10
MINIMUM_DYNAMIC_RANGE_DB = 20.0
MINIMUM_FIT_SECONDS = 0.05

FREQUENCIES_HZ = multioutput.FREQUENCIES_HZ
AMPLITUDE_DECAY_PER_SECOND = multioutput.AMPLITUDE_DECAY_PER_SECOND
EXTRACTOR_PYTHON_SHA256 = multioutput.EXTRACTOR_PYTHON_SHA256
MULTIOUTPUT_PYTHON_SHA256 = (
    "f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e"
)
SYNTHETIC_REPORT_SHA256 = (
    "a099f50d017e4f86ac7e2519663c457622e755ddd4a1969dcbd629fc0e0e8bea"
)
CERAMIC_REJECTION_SHA256 = (
    "9947c427a3416b3a3635c43ea24e074df84ce4db97f0a22cca3be8c93ac96cbf"
)

GATES = {
    "selected_mode_count": 16,
    "minimum_valid_adaptive_fit_fraction": 0.75,
    "maximum_median_frequency_error_cents": 40.0,
    "maximum_median_adaptive_decay_error_db_per_second": 3.0,
    "minimum_median_fit_r_squared": 0.95,
    "minimum_fixed_window_decay_error_db_per_second": 6.0,
    "minimum_decay_error_reduction_db_per_second": 5.0,
}


class AdaptiveControlError(RuntimeError):
    """The frozen adaptive-control contract or numeric invariant failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
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
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise AdaptiveControlError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise AdaptiveControlError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise AdaptiveControlError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "parents": {
            "synthetic_multioutput_control": {
                "path": "../ps2-multioutput-decay-control-v1/run-a/report.json",
                "sha256": SYNTHETIC_REPORT_SHA256,
            },
            "ceramic_multioutput_rejection": {
                "path": "../ps2-realimpact-ceramic-cup-observation-v1/multioutput-analysis-a/report.json",
                "sha256": CERAMIC_REJECTION_SHA256,
            },
        },
        "synthetic_fixture": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "sample_count": SAMPLE_COUNT,
            "channel_count": CHANNEL_COUNT,
            "onset_sample": ONSET_SAMPLE,
            "frequencies_hz": FREQUENCIES_HZ,
            "amplitude_decay_per_second": AMPLITUDE_DECAY_PER_SECOND,
            "direct_gain": DIRECT_GAIN,
            "delayed_gain": DELAYED_GAIN,
            "mode_delay_samples": DELAY_SAMPLES,
            "participation_rule": "0.45 + 0.04*((7*channel + 3*mode) mod 11); deterministic phase",
            "noise": {
                "generator": "numpy.PCG64",
                "seed": NOISE_SEED,
                "standard_deviation": NOISE_STANDARD_DEVIATION,
            },
            "sample_format": "f64le-channel-major",
        },
        "candidate": {
            "id": "spatial-adaptive-rms-envelope-v1",
            "frequency_selection": "unchanged spatial-modal-power-15-v1 peak selection",
            "mode_filter": {
                "implementation": "causal Butterworth low-pass then high-pass per output, matching audio_dspy modal_tools",
                "width_hz": FILTER_WIDTH_HZ,
                "order": FILTER_ORDER,
            },
            "envelope": {
                "implementation": "sqrt of one-pole-smoothed summed per-output filtered power",
                "eta_seconds": ENVELOPE_ETA_SECONDS,
            },
            "fit_interval": {
                "peak": "maximum normalized RMS envelope",
                "noise_floor": "maximum dB envelope over final 10 percent",
                "start_dynamic_fraction": START_DYNAMIC_FRACTION,
                "end_dynamic_fraction": END_DYNAMIC_FRACTION,
                "minimum_dynamic_range_db": MINIMUM_DYNAMIC_RANGE_DB,
                "minimum_fit_seconds": MINIMUM_FIT_SECONDS,
            },
            "source_provenance": {
                "realimpact_commit": "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987",
                "realimpact_notebook": "https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/modes_dsp_sweep.ipynb",
                "audio_dspy_commit": "2ad0b05f81b014c27612f6f91087265ee52e9238",
                "audio_dspy_modal_tools": "https://raw.githubusercontent.com/jatinchowdhury18/audio_dspy/2ad0b05f81b014c27612f6f91087265ee52e9238/audio_dspy/modal_tools.py",
            },
        },
        "comparator": {
            "id": "spatial-modal-power-15-v1-fixed-window",
            "source": {
                "path": "lab/scripts/physical_sound_multioutput_decay_control.py",
                "sha256": MULTIOUTPUT_PYTHON_SHA256,
            },
        },
        "reference_extractor": {
            "source": {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": EXTRACTOR_PYTHON_SHA256,
            },
        },
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "gates": GATES,
        "data_policy": {
            "synthetic_only": True,
            "real_payload_allowed": False,
            "network_allowed": False,
            "threshold_tuning_allowed": False,
            "physics_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": "publish support or rejection once; on rejection do not weaken gates, on support freeze a separate Ceramic adaptive counterfactual before real reuse",
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise AdaptiveControlError("adaptive control manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise AdaptiveControlError(f"parse adaptive control manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise AdaptiveControlError("adaptive control manifest changed")
    return data, manifest


def resolve_report(base: Path, ref: dict[str, Any], label: str) -> tuple[bytes, dict[str, Any]]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise AdaptiveControlError(f"{label} escapes external physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise AdaptiveControlError(f"{label} identity changed")
    return data, json.loads(data)


def validate_parents(base: Path, manifest: dict[str, Any]) -> dict[str, str]:
    synthetic_bytes, synthetic = resolve_report(
        base, manifest["parents"]["synthetic_multioutput_control"], "synthetic control"
    )
    ceramic_bytes, ceramic = resolve_report(
        base, manifest["parents"]["ceramic_multioutput_rejection"], "Ceramic rejection"
    )
    if (
        synthetic.get("decision") != "MultiOutputSpatialDecayControlSupported"
        or synthetic.get("gate", {}).get("passed") is not True
        or synthetic.get("real_payload_bytes_read") != 0
        or ceramic.get("decision") != "CeramicCupMultiOutputCounterfactualRejected"
        or ceramic.get("gate", {}).get("passed") is not False
        or ceramic.get("candidate_analysis", {}).get("decaying_mode_fraction") != 0.1875
        or ceramic.get("network_requests") != 0
        or ceramic.get("physics_solver_runs") != 0
        or ceramic.get("planter_payload_bytes_read") != 0
    ):
        raise AdaptiveControlError("adaptive control parent lineage changed")
    return {
        "synthetic_multioutput_control_sha256": sha256_bytes(synthetic_bytes),
        "ceramic_multioutput_rejection_sha256": sha256_bytes(ceramic_bytes),
    }


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [manifest["comparator"]["source"], manifest["reference_extractor"]["source"]]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or sha256_file(path) != ref["sha256"]:
            raise AdaptiveControlError(f"bound source changed: {ref['path']}")
        reports.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return reports


def participation(channel: int, mode: int) -> float:
    return 0.45 + 0.04 * ((7 * channel + 3 * mode) % 11)


def generate_fixture() -> np.ndarray:
    channels = np.zeros((CHANNEL_COUNT, SAMPLE_COUNT), dtype=np.float64)
    direct_time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE, dtype=np.float64) / SAMPLE_RATE_HZ
    for channel in range(CHANNEL_COUNT):
        for mode, (frequency, decay, delay) in enumerate(
            zip(FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, DELAY_SAMPLES, strict=True)
        ):
            phase = 0.113 * channel + 0.173 * mode + 0.019 * channel * mode
            scale = participation(channel, mode) / (1.0 + 0.19 * mode)
            channels[channel, ONSET_SAMPLE:] += (
                DIRECT_GAIN
                * scale
                * np.exp(-decay * direct_time)
                * np.sin(2.0 * math.pi * frequency * direct_time + phase)
            )
            delayed_start = ONSET_SAMPLE + delay
            delayed_time = (
                np.arange(SAMPLE_COUNT - delayed_start, dtype=np.float64) / SAMPLE_RATE_HZ
            )
            channels[channel, delayed_start:] += (
                DELAYED_GAIN
                * scale
                * np.exp(-decay * delayed_time)
                * np.sin(2.0 * math.pi * frequency * delayed_time + phase + 0.41)
            )
    noise = np.random.Generator(np.random.PCG64(NOISE_SEED)).standard_normal(channels.shape)
    channels += NOISE_STANDARD_DEVIATION * noise
    return channels


def filtered_spatial_power(channels: np.ndarray, frequency_hz: float) -> np.ndarray:
    low = (frequency_hz - FILTER_WIDTH_HZ / 2.0) / (SAMPLE_RATE_HZ / 2.0)
    high = (frequency_hz + FILTER_WIDTH_HZ / 2.0) / (SAMPLE_RATE_HZ / 2.0)
    highpass_b, highpass_a = signal.butter(FILTER_ORDER, low, btype="highpass")
    lowpass_b, lowpass_a = signal.butter(FILTER_ORDER, high, btype="lowpass")
    power = np.zeros(SAMPLE_COUNT, dtype=np.float64)
    for channel in channels:
        filtered = signal.lfilter(lowpass_b, lowpass_a, channel)
        filtered = signal.lfilter(highpass_b, highpass_a, filtered)
        power += filtered * filtered
    return power


def rms_envelope(power: np.ndarray) -> np.ndarray:
    pole = math.exp(-1.0 / (ENVELOPE_ETA_SECONDS * SAMPLE_RATE_HZ))
    smoothed = signal.lfilter([1.0 - pole], [1.0, -pole], power)
    envelope = np.sqrt(np.maximum(smoothed, single.SAMPLE_EPSILON))
    return envelope / max(float(np.max(envelope)), single.SAMPLE_EPSILON)


def first_below(values: np.ndarray, start: int, threshold: float) -> int | None:
    indices = np.flatnonzero(values[start:] < threshold)
    return start + int(indices[0]) if len(indices) else None


def adaptive_fit(channels: np.ndarray, peak: single.SpectralPeak) -> dict[str, Any]:
    envelope = rms_envelope(filtered_spatial_power(channels, peak.frequency_hz))
    levels = 20.0 * np.log10(np.maximum(envelope, single.SAMPLE_EPSILON))
    peak_sample = int(np.argmax(levels))
    tail_start = int(SAMPLE_COUNT * (1.0 - NOISE_FLOOR_TAIL_FRACTION))
    noise_floor_db = float(np.max(levels[tail_start:]))
    peak_db = float(levels[peak_sample])
    dynamic_range_db = peak_db - noise_floor_db
    start_threshold = noise_floor_db + dynamic_range_db * START_DYNAMIC_FRACTION
    end_threshold = noise_floor_db + dynamic_range_db * END_DYNAMIC_FRACTION
    fit_start = first_below(levels, peak_sample, start_threshold)
    fit_end = first_below(levels, fit_start or peak_sample, end_threshold)
    minimum_samples = round(MINIMUM_FIT_SECONDS * SAMPLE_RATE_HZ)
    valid = (
        dynamic_range_db >= MINIMUM_DYNAMIC_RANGE_DB
        and fit_start is not None
        and fit_end is not None
        and fit_end - fit_start >= minimum_samples
    )
    if not valid:
        return {
            "frequency_hz": peak.frequency_hz,
            "valid": False,
            "peak_sample": peak_sample,
            "noise_floor_db": noise_floor_db,
            "dynamic_range_db": dynamic_range_db,
            "fit_start_sample": fit_start,
            "fit_end_sample": fit_end,
            "fit_decay_db_per_second": None,
            "fit_r_squared": None,
        }
    assert fit_start is not None and fit_end is not None
    x = np.arange(fit_start, fit_end, dtype=np.float64) / SAMPLE_RATE_HZ
    y = levels[fit_start:fit_end]
    slope, intercept = np.polyfit(x, y, 1)
    predicted = slope * x + intercept
    residual = float(np.sum((y - predicted) ** 2))
    total = float(np.sum((y - np.mean(y)) ** 2))
    r_squared = 1.0 - residual / total if total > single.SAMPLE_EPSILON else 0.0
    return {
        "frequency_hz": peak.frequency_hz,
        "valid": True,
        "peak_sample": peak_sample,
        "noise_floor_db": noise_floor_db,
        "dynamic_range_db": dynamic_range_db,
        "fit_start_sample": fit_start,
        "fit_end_sample": fit_end,
        "fit_decay_db_per_second": float(slope),
        "fit_r_squared": r_squared,
    }


def analyze_adaptive(channels: np.ndarray) -> dict[str, Any]:
    norm = np.sqrt(np.sum(channels * channels, axis=0))
    peak = float(np.max(norm))
    onset_indices = np.flatnonzero(norm >= peak * single.ONSET_PEAK_FRACTION)
    if not len(onset_indices):
        raise AdaptiveControlError("synthetic fixture onset was not found")
    onset = int(onset_indices[0])
    peaks = multioutput.spatial_modal_peaks(channels, onset)
    modes = [adaptive_fit(channels, peak) for peak in peaks]
    valid_modes = [mode for mode in modes if mode["valid"]]
    comparisons = []
    if len(modes) == len(FREQUENCIES_HZ):
        for mode, frequency, decay in zip(
            modes, FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, strict=True
        ):
            expected_decay = -20.0 * decay / math.log(10.0)
            comparisons.append(
                {
                    "expected_frequency_hz": frequency,
                    "observed_frequency_hz": mode["frequency_hz"],
                    "frequency_error_cents": abs(
                        single.cents_between(frequency, mode["frequency_hz"])
                    ),
                    "expected_decay_db_per_second": expected_decay,
                    "observed_decay_db_per_second": mode["fit_decay_db_per_second"],
                    "decay_error_db_per_second": (
                        abs(mode["fit_decay_db_per_second"] - expected_decay)
                        if mode["valid"]
                        else None
                    ),
                }
            )
    valid_comparisons = [item for item in comparisons if item["decay_error_db_per_second"] is not None]
    return {
        "onset_sample": onset,
        "selected_mode_count": len(modes),
        "valid_fit_count": len(valid_modes),
        "valid_fit_fraction": len(valid_modes) / len(modes),
        "median_frequency_error_cents": (
            single.median([item["frequency_error_cents"] for item in comparisons])
            if comparisons
            else None
        ),
        "median_decay_error_db_per_second": (
            single.median([item["decay_error_db_per_second"] for item in valid_comparisons])
            if valid_comparisons
            else None
        ),
        "median_fit_r_squared": (
            single.median([mode["fit_r_squared"] for mode in valid_modes])
            if valid_modes
            else None
        ),
        "modes": modes,
        "ground_truth": comparisons,
    }


def check(name: str, observed: float, relation: str, threshold: float) -> dict[str, Any]:
    if relation == ">=":
        passed = observed >= threshold
    elif relation == "<=":
        passed = observed <= threshold
    else:
        passed = observed == threshold
    return {
        "name": name,
        "observed": observed,
        "relation": relation,
        "threshold": threshold,
        "passed": passed,
    }


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "network_requests": 0,
        "real_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents = validate_parents(base, manifest)
    sources = validate_sources(root, manifest)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "AdaptiveDecaySyntheticControlFrozen",
        "claim": "SYNTHETIC_SOURCE_FAITHFUL_ADAPTIVE_DECAY_PREFLIGHT_ONLY / NO_REAL_DATA_PHYSICS_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "bound_sources": sources,
        "synthetic_fixture": manifest["synthetic_fixture"],
        "candidate": manifest["candidate"],
        "comparator": manifest["comparator"],
        "runtime": manifest["runtime"],
        "gates": GATES,
        "next_action": "commit this preflight, then execute the deterministic adaptive-decay control twice",
    }


def run_control(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents = validate_parents(base, manifest)
    validate_sources(root, manifest)
    channels = generate_fixture()
    fixture_sha256 = sha256_bytes(channels.astype("<f8", copy=False).tobytes(order="C"))
    adaptive = analyze_adaptive(channels)
    fixed = multioutput.analyze_spatial(channels)
    fixed_truth = multioutput.source_errors(fixed)
    adaptive_error = (
        adaptive["median_decay_error_db_per_second"]
        if adaptive["median_decay_error_db_per_second"] is not None
        else 1.0e9
    )
    fixed_error = (
        fixed_truth["median_decay_error_db_per_second"]
        if fixed_truth["complete"]
        else 1.0e9
    )
    error_reduction = fixed_error - adaptive_error
    checks = [
        check("selected_mode_count", adaptive["selected_mode_count"], "==", GATES["selected_mode_count"]),
        check(
            "valid_adaptive_fit_fraction",
            adaptive["valid_fit_fraction"],
            ">=",
            GATES["minimum_valid_adaptive_fit_fraction"],
        ),
        check(
            "median_frequency_error_cents",
            (
                adaptive["median_frequency_error_cents"]
                if adaptive["median_frequency_error_cents"] is not None
                else 1.0e9
            ),
            "<=",
            GATES["maximum_median_frequency_error_cents"],
        ),
        check(
            "median_adaptive_decay_error_db_per_second",
            adaptive_error,
            "<=",
            GATES["maximum_median_adaptive_decay_error_db_per_second"],
        ),
        check(
            "median_fit_r_squared",
            (
                adaptive["median_fit_r_squared"]
                if adaptive["median_fit_r_squared"] is not None
                else 0.0
            ),
            ">=",
            GATES["minimum_median_fit_r_squared"],
        ),
        check("fixed_window_truth_complete", float(fixed_truth["complete"]), "==", 1.0),
        check(
            "fixed_window_decay_error_db_per_second",
            fixed_error,
            ">=",
            GATES["minimum_fixed_window_decay_error_db_per_second"],
        ),
        check(
            "decay_error_reduction_db_per_second",
            error_reduction,
            ">=",
            GATES["minimum_decay_error_reduction_db_per_second"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["run"],
        "status": "Validated",
        "decision": (
            "AdaptiveDecaySyntheticControlSupported"
            if passed
            else "AdaptiveDecaySyntheticControlRejected"
        ),
        "claim": "SYNTHETIC_SOURCE_FAITHFUL_ADAPTIVE_DECAY_CONTROL_ONLY / NO_REAL_DATA_PHYSICS_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "synthetic_fixture_sha256": fixture_sha256,
        "synthetic_fixture_bytes": channels.nbytes,
        "adaptive_analysis": adaptive,
        "fixed_window_analysis": fixed,
        "fixed_window_ground_truth": fixed_truth,
        "decay_error_reduction_db_per_second": error_reduction,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze a separate read-only Ceramic adaptive counterfactual"
            if passed
            else "reject this estimator revision; do not weaken gates or reuse Ceramic"
        ),
    }


def publish(output: Path, report: dict[str, Any]) -> bytes:
    report_bytes = canonical_json(report)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report_bytes


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = external_file(root, arguments.manifest, "manifest")
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    report = (
        preflight(root, base, manifest_bytes, manifest, runner_sha256)
        if arguments.stage == "preflight"
        else run_control(root, base, manifest_bytes, manifest, runner_sha256)
    )
    report_bytes = publish(output, report)
    print(f"Adaptive decay control {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("real payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
