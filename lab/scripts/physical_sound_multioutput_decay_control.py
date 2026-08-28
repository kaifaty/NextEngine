#!/usr/bin/env python3
"""Evaluate a frozen synthetic multi-output modal-decay estimator control."""

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

import physical_sound_realimpact_pitcher_calibration as single


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-multioutput-decay-control.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-physical-sound-multioutput-decay-control-preflight.report.v1",
    "run": "nextengine.experimental-physical-sound-multioutput-decay-control.report.v1",
}
STUDY_ID = "physical-sound-multioutput-observation-decay-control"
REVISION = "spatial-modal-power-v1"
CHANNEL_COUNT = 15
SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 192_000
ONSET_SAMPLE = 240
NODE_CHANNEL = 7
NODE_DIRECT_GAIN = 0.01
NODE_DELAYED_GAIN = 0.30
NODE_DELAY_SAMPLES = 16_800
FREQUENCIES_HZ = [
    311.0,
    433.0,
    587.0,
    751.0,
    947.0,
    1_187.0,
    1_451.0,
    1_783.0,
    2_153.0,
    2_591.0,
    3_083.0,
    3_659.0,
    4_327.0,
    5_101.0,
    6_011.0,
    7_013.0,
]
AMPLITUDE_DECAY_PER_SECOND = [1.15 + 0.09 * index for index in range(16)]
EXTRACTOR_PYTHON_SHA256 = "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
TRANSFER_DSP_SHA256 = "131bbf42d01e633b6cac0c4e0340178f3f5a0a83324bb64630d8423da8179ca4"
FIXTURE_REPORT_SHA256 = "2e3db2d3f231d69b67b2a3a094f83fbc7ed10144a02f4c424d75bbbb2f85572f"
FIXTURE_SAMPLE_SHA256 = "c8316f81da79b9823cbeaf1c0c49ee1708d2112fca8b4905a792546fd3f40477"

GATES = {
    "selected_mode_count": 16,
    "minimum_persistent_mode_recall": 0.75,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_decaying_mode_fraction": 0.75,
    "maximum_median_tail_prediction_rmse_db": 24.0,
    "maximum_median_ground_truth_decay_error_db_per_second": 3.0,
    "maximum_single_output_decaying_mode_fraction": 0.49,
    "minimum_decaying_fraction_improvement": 0.25,
}


class ControlError(RuntimeError):
    """The frozen synthetic control contract or numeric invariant failed."""


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
        raise ControlError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise ControlError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise ControlError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "synthetic_fixture": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "sample_count": SAMPLE_COUNT,
            "channel_count": CHANNEL_COUNT,
            "onset_sample": ONSET_SAMPLE,
            "frequencies_hz": FREQUENCIES_HZ,
            "amplitude_decay_per_second": AMPLITUDE_DECAY_PER_SECOND,
            "participation_rule": "channel 7 direct gain 0.01; other gains 0.45 + 0.04*((7*channel + 3*mode) mod 11); deterministic per-channel/mode phase",
            "node_channel": NODE_CHANNEL,
            "node_direct_gain": NODE_DIRECT_GAIN,
            "node_delayed_gain": NODE_DELAYED_GAIN,
            "node_delay_samples": NODE_DELAY_SAMPLES,
            "node_delay_seconds": NODE_DELAY_SAMPLES / SAMPLE_RATE_HZ,
            "sample_format": "f64le-channel-major",
        },
        "estimator": {
            "id": "spatial-modal-power-15-v1",
            "frequency_selection": "unchanged injective-modal-16-fft65536-v2 over sum of per-channel windowed FFT power",
            "decay_tracking": "unchanged V2 windows/bins/regression over sum of per-channel three-bin modal power",
            "single_output_control": "unchanged V2 extractor on channel 7",
            "no_cross_channel_phase_fit": True,
        },
        "implementation": {
            "python_source": {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": EXTRACTOR_PYTHON_SHA256,
            },
            "rust_dsp_source": {
                "path": "tools/xtask/src/physical_sound_registry_command/transfer_calibration/dsp.rs",
                "sha256": TRANSFER_DSP_SHA256,
            },
            "rust_fixture": {
                "report_path": "../ps2-realimpact-geometry-spatial-transfer-v1/extractor-parity-fixture-a/report.json",
                "report_sha256": FIXTURE_REPORT_SHA256,
                "sample_path": "../ps2-realimpact-geometry-spatial-transfer-v1/extractor-parity-fixture-a/fixture.f64le",
                "sample_sha256": FIXTURE_SAMPLE_SHA256,
            },
        },
        "gates": GATES,
        "data_policy": {
            "synthetic_only": True,
            "realimpact_or_other_real_payload_allowed": False,
            "network_allowed": False,
            "threshold_tuning_allowed": False,
            "production_or_validator_credit_allowed": False,
            "physics_or_planter_access_allowed": False,
        },
        "stop_rule": "publish support or rejection once; on rejection do not reuse real rows, on support freeze a separate read-only Ceramic counterfactual before real analysis",
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ControlError("multi-output manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ControlError(f"parse multi-output manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ControlError("multi-output control manifest changed")
    return data, manifest


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [manifest["implementation"]["python_source"], manifest["implementation"]["rust_dsp_source"]]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or sha256_file(path) != ref["sha256"]:
            raise ControlError(f"bound source changed: {ref['path']}")
        reports.append({"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size})
    return reports


def validate_fixture(base: Path, manifest: dict[str, Any]) -> tuple[dict[str, Any], dict[str, float]]:
    ref = manifest["implementation"]["rust_fixture"]
    report = (base / ref["report_path"]).resolve(strict=True)
    sample = (base / ref["sample_path"]).resolve(strict=True)
    if (
        not report.is_relative_to(base.parent)
        or not sample.is_relative_to(base.parent)
        or sha256_file(report) != ref["report_sha256"]
        or sha256_file(sample) != ref["sample_sha256"]
        or report.parent != sample.parent
    ):
        raise ControlError("Rust parity fixture identity changed")
    return single.validate_fixture(report.parent)


def participation(channel: int, mode: int) -> float:
    if channel == NODE_CHANNEL:
        return NODE_DIRECT_GAIN
    return 0.45 + 0.04 * ((7 * channel + 3 * mode) % 11)


def generate_fixture() -> np.ndarray:
    channels = np.zeros((CHANNEL_COUNT, SAMPLE_COUNT), dtype=np.float64)
    time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE, dtype=np.float64) / SAMPLE_RATE_HZ
    for channel in range(CHANNEL_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, strict=True)
        ):
            phase = 0.113 * channel + 0.173 * mode + 0.019 * channel * mode
            channels[channel, ONSET_SAMPLE:] += (
                participation(channel, mode)
                * np.exp(-decay * time)
                * np.sin(2.0 * math.pi * frequency * time + phase)
                / (1.0 + 0.19 * mode)
            )
    delayed_time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE - NODE_DELAY_SAMPLES, dtype=np.float64) / SAMPLE_RATE_HZ
    delayed_start = ONSET_SAMPLE + NODE_DELAY_SAMPLES
    for mode, (frequency, decay) in enumerate(
        zip(FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, strict=True)
    ):
        phase = 0.113 * NODE_CHANNEL + 0.173 * mode + 0.019 * NODE_CHANNEL * mode + 0.41
        channels[NODE_CHANNEL, delayed_start:] += (
            NODE_DELAYED_GAIN
            * np.exp(-decay * delayed_time)
            * np.sin(2.0 * math.pi * frequency * delayed_time + phase)
            / (1.0 + 0.19 * mode)
        )
    return channels


def spatial_power_spectrum(channels: np.ndarray, start: int, size: int) -> np.ndarray:
    return np.sum(
        np.stack([single.power_spectrum(channel, start, size) for channel in channels]),
        axis=0,
    )


def spatial_modal_peaks(channels: np.ndarray, start: int) -> list[single.SpectralPeak]:
    power = spatial_power_spectrum(channels, start, single.FFT_SIZE)
    bin_hz = SAMPLE_RATE_HZ / single.FFT_SIZE
    first = math.ceil(single.MINIMUM_FREQUENCY_HZ / bin_hz)
    last = min(math.floor(single.MAXIMUM_FREQUENCY_HZ / bin_hz), len(power) - 2)
    maximum = max(single.SAMPLE_EPSILON, float(np.max(power[first : last + 1])))
    candidates = []
    for index in range(first, last + 1):
        if power[index] <= power[index - 1] or power[index] < power[index + 1]:
            continue
        relative = 10.0 * math.log10(max(float(power[index]), single.SAMPLE_EPSILON) / maximum)
        if relative >= single.PEAK_FLOOR_DB:
            candidates.append(
                single.SpectralPeak(single.interpolated_frequency(power, index, bin_hz), relative)
            )
    candidates.sort(key=lambda value: (-value.relative_level_db, value.frequency_hz))
    selected: list[single.SpectralPeak] = []
    for candidate in candidates:
        if all(
            abs(candidate.frequency_hz - other.frequency_hz) >= single.MINIMUM_SEPARATION_HZ
            and abs(single.cents_between(candidate.frequency_hz, other.frequency_hz))
            >= single.PEAK_SEPARATION_CENTS
            for other in selected
        ):
            selected.append(candidate)
            if len(selected) == single.MODE_LIMIT:
                break
    if len(selected) < 4:
        raise ControlError("spatial estimator found fewer than four modes")
    return sorted(selected, key=lambda value: value.frequency_hz)


def spatial_level_tracks(
    channels: np.ndarray, onset: int, peaks: list[single.SpectralPeak]
) -> list[list[tuple[float, float]]]:
    first = single.DAMPING_START_MS * SAMPLE_RATE_HZ // 1_000
    last = single.DAMPING_END_MS * SAMPLE_RATE_HZ // 1_000
    bin_hz = SAMPLE_RATE_HZ / single.DAMPING_FFT_SIZE
    bins = [round(value.frequency_hz / bin_hz) for value in peaks]
    tracks: list[list[tuple[float, float]]] = [[] for _ in peaks]
    for offset in range(first, last + 1, single.DAMPING_HOP_SIZE):
        power = spatial_power_spectrum(channels, onset + offset, single.DAMPING_FFT_SIZE)
        time = (offset + single.DAMPING_FFT_SIZE // 2) / SAMPLE_RATE_HZ
        for track, bin_index in zip(tracks, bins, strict=True):
            lower = max(0, bin_index - 1)
            upper = min(len(power) - 1, bin_index + 1)
            energy = float(np.sum(power[lower : upper + 1]))
            track.append((time, 10.0 * math.log10(max(energy, single.SAMPLE_EPSILON))))
    return tracks


def analyze_spatial(channels: np.ndarray) -> dict[str, Any]:
    spatial_peak = np.sqrt(np.sum(channels * channels, axis=0))
    peak = float(np.max(spatial_peak))
    indices = np.flatnonzero(spatial_peak >= peak * single.ONSET_PEAK_FRACTION)
    if not len(indices):
        raise ControlError("spatial fixture onset was not found")
    onset = int(indices[0])
    required = onset + single.DAMPING_END_MS * SAMPLE_RATE_HZ // 1_000 + single.DAMPING_FFT_SIZE
    if required > SAMPLE_COUNT:
        raise ControlError("spatial fixture is too short")
    fit_peaks = spatial_modal_peaks(channels, onset)
    tail_peaks = spatial_modal_peaks(
        channels, onset + single.TAIL_START_MS * SAMPLE_RATE_HZ // 1_000
    )
    assignments = single.injective_matches(fit_peaks, tail_peaks)
    tracks = spatial_level_tracks(channels, onset, fit_peaks)
    modes = []
    for fit_peak, tail_index, track in zip(fit_peaks, assignments, tracks, strict=True):
        fit = [value for value in track if value[0] < single.DAMPING_SPLIT_MS / 1_000.0]
        tail = [value for value in track if value[0] >= single.DAMPING_SPLIT_MS / 1_000.0]
        fit_slope = single.linear_slope(fit)
        tail_slope = single.linear_slope(tail)
        matched = tail_peaks[tail_index] if tail_index is not None else None
        modes.append(
            {
                "frequency_hz": fit_peak.frequency_hz,
                "relative_level_db": fit_peak.relative_level_db,
                "matched_tail_frequency_hz": matched.frequency_hz if matched else None,
                "frequency_error_cents": abs(single.cents_between(fit_peak.frequency_hz, matched.frequency_hz)) if matched else None,
                "fit_decay_db_per_second": fit_slope,
                "tail_decay_db_per_second": tail_slope,
                "tail_prediction_rmse_db": single.anchored_tail_rmse(tail, fit_slope),
            }
        )
    persistent_errors = [mode["frequency_error_cents"] for mode in modes if mode["frequency_error_cents"] is not None]
    persistent_count = len(persistent_errors)
    selected_count = len(modes)
    persistent_recall = persistent_count / selected_count
    frequency_error = single.median(persistent_errors) if persistent_errors else 80.0
    decay_fraction = sum(mode["fit_decay_db_per_second"] < -1.0 for mode in modes) / selected_count
    tail_rmse = single.median([mode["tail_prediction_rmse_db"] for mode in modes])
    return {
        "onset_sample": onset,
        "selected_mode_count": selected_count,
        "persistent_mode_count": persistent_count,
        "persistent_mode_recall": persistent_recall,
        "median_frequency_error_cents": frequency_error,
        "decaying_mode_fraction": decay_fraction,
        "median_tail_prediction_rmse_db": tail_rmse,
        "modes": modes,
    }


def source_errors(analysis: dict[str, Any]) -> dict[str, Any]:
    modes = analysis["modes"]
    if len(modes) != len(FREQUENCIES_HZ):
        return {"complete": False, "median_frequency_error_cents": None, "median_decay_error_db_per_second": None, "per_mode": []}
    reports = []
    for mode, frequency, decay in zip(modes, FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, strict=True):
        expected_decay = -20.0 * decay / math.log(10.0)
        reports.append(
            {
                "expected_frequency_hz": frequency,
                "observed_frequency_hz": mode["frequency_hz"],
                "frequency_error_cents": abs(single.cents_between(frequency, mode["frequency_hz"])),
                "expected_decay_db_per_second": expected_decay,
                "observed_decay_db_per_second": mode["fit_decay_db_per_second"],
                "decay_error_db_per_second": abs(mode["fit_decay_db_per_second"] - expected_decay),
            }
        )
    return {
        "complete": True,
        "median_frequency_error_cents": single.median([item["frequency_error_cents"] for item in reports]),
        "median_decay_error_db_per_second": single.median([item["decay_error_db_per_second"] for item in reports]),
        "per_mode": reports,
    }


def check(name: str, observed: float, relation: str, threshold: float) -> dict[str, Any]:
    passed = observed >= threshold if relation == ">=" else observed <= threshold if relation == "<=" else observed == threshold
    return {"name": name, "observed": observed, "relation": relation, "threshold": threshold, "passed": passed}


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
        "production_or_validator_credit": False,
    }


def preflight(
    root: Path, base: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    sources = validate_sources(root, manifest)
    fixture_report, parity = validate_fixture(base, manifest)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "MultiOutputDecayControlFrozen",
        "claim": "SYNTHETIC_MULTI_OUTPUT_PREFLIGHT_ONLY / NO_REAL_DATA_PHYSICS_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "bound_sources": sources,
        "fixture_report_sha256": FIXTURE_REPORT_SHA256,
        "fixture_analysis": fixture_report["analysis"],
        "fixture_parity": parity,
        "synthetic_fixture": manifest["synthetic_fixture"],
        "estimator": manifest["estimator"],
        "gates": GATES,
        "next_action": "commit this preflight, then execute the deterministic synthetic control twice before any real-row reuse",
    }


def run_control(
    root: Path, base: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    validate_sources(root, manifest)
    _, parity = validate_fixture(base, manifest)
    channels = generate_fixture()
    fixture_sha256 = sha256_bytes(channels.astype("<f8", copy=False).tobytes(order="C"))
    spatial = analyze_spatial(channels)
    comparator = single.analyze_v2(np.asarray(channels[NODE_CHANNEL], dtype=np.float64))
    truth = source_errors(spatial)
    checks = [
        check("selected_mode_count", spatial["selected_mode_count"], "==", GATES["selected_mode_count"]),
        check("persistent_mode_recall", spatial["persistent_mode_recall"], ">=", GATES["minimum_persistent_mode_recall"]),
        check("median_frequency_error_cents", spatial["median_frequency_error_cents"], "<=", GATES["maximum_median_frequency_error_cents"]),
        check("decaying_mode_fraction", spatial["decaying_mode_fraction"], ">=", GATES["minimum_decaying_mode_fraction"]),
        check("median_tail_prediction_rmse_db", spatial["median_tail_prediction_rmse_db"], "<=", GATES["maximum_median_tail_prediction_rmse_db"]),
        check("ground_truth_complete", float(truth["complete"]), "==", 1.0),
        check("median_ground_truth_decay_error_db_per_second", truth["median_decay_error_db_per_second"] if truth["complete"] else 1.0e9, "<=", GATES["maximum_median_ground_truth_decay_error_db_per_second"]),
        check("single_output_decaying_mode_fraction", comparator["decaying_mode_fraction"], "<=", GATES["maximum_single_output_decaying_mode_fraction"]),
        check("decaying_fraction_improvement", spatial["decaying_mode_fraction"] - comparator["decaying_mode_fraction"], ">=", GATES["minimum_decaying_fraction_improvement"]),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["run"],
        "status": "Validated",
        "decision": "MultiOutputSpatialDecayControlSupported" if passed else "MultiOutputSpatialDecayControlRejected",
        "claim": "SYNTHETIC_MULTI_OUTPUT_DECAY_CONTROL_ONLY / NO_REAL_DATA_PHYSICS_ADMISSION_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "extractor_fixture_parity": parity,
        "synthetic_fixture_sha256": fixture_sha256,
        "synthetic_fixture_bytes": channels.nbytes,
        "spatial_analysis": spatial,
        "single_output_node_control": comparator,
        "ground_truth_comparison": truth,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze a separate read-only Ceramic multi-output counterfactual; retain zero admission or mechanics credit"
            if passed
            else "reject this estimator revision; do not reuse real rows or weaken the frozen synthetic gates"
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
    print(f"Multi-output decay control {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("real payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
