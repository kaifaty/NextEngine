#!/usr/bin/env python3
"""Run the frozen REALIMPACT Pitcher calibration protocol."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import struct
import sys
from typing import Any

import numpy as np


MANIFEST_SHA256 = "c60621ccf30442ba8fe4c25533325782ef1807c1be98101eae78fb04d18ad4a7"
MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-geometry-spatial-transfer-calibration.manifest.v1"
)
PREFLIGHT_REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-geometry-spatial-transfer-calibration-runner-preflight.report.v1"
)
FIXTURE_REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-transfer-extractor-parity-fixture.report.v1"
)
FIXTURE_REPORT_SHA256 = "2e3db2d3f231d69b67b2a3a094f83fbc7ed10144a02f4c424d75bbbb2f85572f"
FIXTURE_SAMPLE_SHA256 = "c8316f81da79b9823cbeaf1c0c49ee1708d2112fca8b4905a792546fd3f40477"
GEOMETRY_SHA256 = "bcd54087f27167cbde16077479004deb6be2f8857cec5af8f3381e85e9539acc"
GEOMETRY_BYTES = 3_718_246
EXTRACTOR_ID = "injective-modal-16-fft65536-v2"
SAMPLE_RATE_HZ = 48_000
MODE_LIMIT = 16
FFT_SIZE = 65_536
ONSET_PEAK_FRACTION = 0.02
MINIMUM_FREQUENCY_HZ = 250.0
MAXIMUM_FREQUENCY_HZ = 12_000.0
PEAK_FLOOR_DB = -45.0
PEAK_SEPARATION_CENTS = 45.0
MINIMUM_SEPARATION_HZ = 12.0
MATCH_TOLERANCE_CENTS = 40.0
TAIL_START_MS = 900
DAMPING_START_MS = 50
DAMPING_SPLIT_MS = 900
DAMPING_END_MS = 2_400
DAMPING_FFT_SIZE = 16_384
DAMPING_HOP_SIZE = 2_048
SAMPLE_EPSILON = 1.0e-24
RUST_ABSOLUTE_TOLERANCE = 2.0e-7
RUST_RELATIVE_TOLERANCE = 2.0e-10


class CalibrationError(RuntimeError):
    """A frozen calibration contract failed."""


@dataclass(frozen=True)
class SpectralPeak:
    frequency_hz: float
    relative_level_db: float


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--rust-fixture", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--preflight-only", action="store_true")
    return parser.parse_args()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_exact(path: Path, expected_hash: str, label: str) -> bytes:
    try:
        data = path.read_bytes()
    except OSError as error:
        raise CalibrationError(f"read {label} {path}: {error}") from error
    actual = sha256_bytes(data)
    if actual != expected_hash:
        raise CalibrationError(
            f"{label} hash changed: expected {expected_hash}, got {actual}"
        )
    return data


def load_manifest(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_exact(path, MANIFEST_SHA256, "Pitcher calibration manifest")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise CalibrationError(f"parse Pitcher calibration manifest: {error}") from error
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("study_id")
        != "physical-sound-realimpact-geometry-spatial-transfer"
        or manifest.get("phase") != "calibration-preregistration"
        or manifest.get("pitcher_audio", {}).get("compressed_prefix_bytes")
        != 536_870_912
        or manifest.get("opening_protocol", {}).get("planter_audio_allowed") is not False
    ):
        raise CalibrationError("Pitcher calibration manifest contract changed")
    return data, manifest


def read_u8(data: memoryview, offset: int) -> tuple[int, int]:
    if offset + 1 > len(data):
        raise CalibrationError("geometry block ended inside u8")
    return data[offset], offset + 1


def read_u16(data: memoryview, offset: int) -> tuple[int, int]:
    if offset + 2 > len(data):
        raise CalibrationError("geometry block ended inside u16")
    return struct.unpack_from("<H", data, offset)[0], offset + 2


def read_u32(data: memoryview, offset: int) -> tuple[int, int]:
    if offset + 4 > len(data):
        raise CalibrationError("geometry block ended inside u32")
    return struct.unpack_from("<I", data, offset)[0], offset + 4


def read_u64(data: memoryview, offset: int) -> tuple[int, int]:
    if offset + 8 > len(data):
        raise CalibrationError("geometry block ended inside u64")
    return struct.unpack_from("<Q", data, offset)[0], offset + 8


def decode_geometry_block(path: Path) -> dict[str, np.ndarray]:
    data = read_exact(path, GEOMETRY_SHA256, "Pitcher geometry block")
    if len(data) != GEOMETRY_BYTES or data[:8] != b"NEPSGEO1":
        raise CalibrationError("Pitcher geometry block identity changed")
    view = memoryview(data)
    count, offset = read_u32(view, 8)
    arrays: dict[str, np.ndarray] = {}
    for _ in range(count):
        name_length, offset = read_u16(view, offset)
        name = bytes(view[offset : offset + name_length]).decode("ascii")
        offset += name_length
        dtype_length, offset = read_u8(view, offset)
        dtype_id = bytes(view[offset : offset + dtype_length]).decode("ascii")
        offset += dtype_length
        rank, offset = read_u8(view, offset)
        shape = []
        for _ in range(rank):
            dimension, offset = read_u64(view, offset)
            shape.append(dimension)
        payload_bytes, offset = read_u64(view, offset)
        if offset + payload_bytes > len(view):
            raise CalibrationError(f"geometry array {name} exceeds block")
        dtype = {"f64le": np.dtype("<f8"), "i64le": np.dtype("<i8")}.get(dtype_id)
        if dtype is None or math.prod(shape) * dtype.itemsize != payload_bytes:
            raise CalibrationError(f"geometry array {name} layout changed")
        arrays[name] = np.frombuffer(
            view[offset : offset + payload_bytes], dtype=dtype
        ).reshape(shape).copy()
        offset += payload_bytes
    if offset != len(view):
        raise CalibrationError("geometry block has trailing bytes")
    required = {
        "bem_points": (1_024, 3),
        "bem_faces": (2_048, 3),
        "bem_modes": (1_024, 64),
        "eigenvalues": (64,),
        "relative_eigen_residuals": (64,),
    }
    if any(name not in arrays or arrays[name].shape != shape for name, shape in required.items()):
        raise CalibrationError("Pitcher geometry arrays changed")
    return arrays


def cents_between(left_hz: float, right_hz: float) -> float:
    return 1_200.0 * math.log(right_hz / left_hz) / math.log(2.0)


def power_spectrum(samples: np.ndarray, start: int, size: int) -> np.ndarray:
    end = start + size
    if end > len(samples):
        raise CalibrationError("FFT window exceeds parity fixture")
    indices = np.arange(size, dtype=np.float64)
    window = 0.5 - 0.5 * np.cos(2.0 * math.pi * indices / float(size - 1))
    transformed = np.fft.rfft(samples[start:end] * window)
    return transformed.real * transformed.real + transformed.imag * transformed.imag


def interpolated_frequency(power: np.ndarray, index: int, bin_hz: float) -> float:
    left = math.log(max(float(power[index - 1]), SAMPLE_EPSILON))
    center = math.log(max(float(power[index]), SAMPLE_EPSILON))
    right = math.log(max(float(power[index + 1]), SAMPLE_EPSILON))
    denominator = left - 2.0 * center + right
    offset = (
        max(-0.5, min(0.5, 0.5 * (left - right) / denominator))
        if abs(denominator) > 1.0e-12
        else 0.0
    )
    return (index + offset) * bin_hz


def modal_peaks(samples: np.ndarray, start: int) -> list[SpectralPeak]:
    power = power_spectrum(samples, start, FFT_SIZE)
    bin_hz = SAMPLE_RATE_HZ / FFT_SIZE
    first = math.ceil(MINIMUM_FREQUENCY_HZ / bin_hz)
    last = min(math.floor(MAXIMUM_FREQUENCY_HZ / bin_hz), len(power) - 2)
    maximum = max(SAMPLE_EPSILON, float(np.max(power[first : last + 1])))
    candidates = []
    for index in range(first, last + 1):
        if power[index] <= power[index - 1] or power[index] < power[index + 1]:
            continue
        relative = 10.0 * math.log10(max(float(power[index]), SAMPLE_EPSILON) / maximum)
        if relative >= PEAK_FLOOR_DB:
            candidates.append(
                SpectralPeak(interpolated_frequency(power, index, bin_hz), relative)
            )
    candidates.sort(key=lambda value: (-value.relative_level_db, value.frequency_hz))
    selected: list[SpectralPeak] = []
    for candidate in candidates:
        if all(
            abs(candidate.frequency_hz - other.frequency_hz) >= MINIMUM_SEPARATION_HZ
            and abs(cents_between(candidate.frequency_hz, other.frequency_hz))
            >= PEAK_SEPARATION_CENTS
            for other in selected
        ):
            selected.append(candidate)
            if len(selected) == MODE_LIMIT:
                break
    if len(selected) < 4:
        raise CalibrationError("Python parity extractor found fewer than four modes")
    return sorted(selected, key=lambda value: value.frequency_hz)


def injective_matches(
    fit: list[SpectralPeak], tail: list[SpectralPeak]
) -> list[int | None]:
    pairs = []
    for fit_index, fit_peak in enumerate(fit):
        for tail_index, tail_peak in enumerate(tail):
            error = abs(cents_between(fit_peak.frequency_hz, tail_peak.frequency_hz))
            if error <= MATCH_TOLERANCE_CENTS:
                pairs.append((error, fit_index, tail_index))
    pairs.sort()
    matches: list[int | None] = [None] * len(fit)
    used = [False] * len(tail)
    for _, fit_index, tail_index in pairs:
        if matches[fit_index] is None and not used[tail_index]:
            matches[fit_index] = tail_index
            used[tail_index] = True
    return matches


def level_tracks(
    samples: np.ndarray, onset: int, peaks: list[SpectralPeak]
) -> list[list[tuple[float, float]]]:
    first = DAMPING_START_MS * SAMPLE_RATE_HZ // 1_000
    last = DAMPING_END_MS * SAMPLE_RATE_HZ // 1_000
    bin_hz = SAMPLE_RATE_HZ / DAMPING_FFT_SIZE
    bins = [round(value.frequency_hz / bin_hz) for value in peaks]
    tracks: list[list[tuple[float, float]]] = [[] for _ in peaks]
    for offset in range(first, last + 1, DAMPING_HOP_SIZE):
        power = power_spectrum(samples, onset + offset, DAMPING_FFT_SIZE)
        time = (offset + DAMPING_FFT_SIZE // 2) / SAMPLE_RATE_HZ
        for track, bin_index in zip(tracks, bins, strict=True):
            lower = max(0, bin_index - 1)
            upper = min(len(power) - 1, bin_index + 1)
            energy = float(np.sum(power[lower : upper + 1]))
            track.append((time, 10.0 * math.log10(max(energy, SAMPLE_EPSILON))))
    return tracks


def linear_slope(points: list[tuple[float, float]]) -> float:
    if len(points) < 3:
        raise CalibrationError("parity damping regression is underspecified")
    x = np.asarray([value[0] for value in points], dtype=np.float64)
    y = np.asarray([value[1] for value in points], dtype=np.float64)
    mean_x = float(np.sum(x)) / len(points)
    mean_y = float(np.sum(y)) / len(points)
    numerator = float(np.sum((x - mean_x) * (y - mean_y)))
    denominator = float(np.sum((x - mean_x) ** 2))
    return numerator / denominator


def anchored_tail_rmse(points: list[tuple[float, float]], slope: float) -> float:
    anchor_x, anchor_y = points[0]
    squared = [
        (value - (anchor_y + slope * (time - anchor_x))) ** 2 for time, value in points
    ]
    return math.sqrt(math.fsum(squared) / len(squared))


def median(values: list[float]) -> float:
    ordered = sorted(values)
    middle = len(ordered) // 2
    if not ordered:
        raise CalibrationError("median is empty")
    if len(ordered) % 2 == 0:
        return (ordered[middle - 1] + ordered[middle]) * 0.5
    return ordered[middle]


def analyze_v2(samples: np.ndarray) -> dict[str, Any]:
    peak = float(np.max(np.abs(samples)))
    if peak <= 1.0e-12:
        raise CalibrationError("parity fixture has no usable signal")
    matches = np.flatnonzero(np.abs(samples) >= peak * ONSET_PEAK_FRACTION)
    if len(matches) == 0:
        raise CalibrationError("parity fixture onset was not found")
    onset = int(matches[0])
    required_end = onset + DAMPING_END_MS * SAMPLE_RATE_HZ // 1_000 + DAMPING_FFT_SIZE
    if required_end > len(samples):
        raise CalibrationError("parity fixture is too short")
    fit_peaks = modal_peaks(samples, onset)
    tail_peaks = modal_peaks(samples, onset + TAIL_START_MS * SAMPLE_RATE_HZ // 1_000)
    assignments = injective_matches(fit_peaks, tail_peaks)
    tracks = level_tracks(samples, onset, fit_peaks)
    modes = []
    for fit_peak, tail_index, track in zip(fit_peaks, assignments, tracks, strict=True):
        fit = [value for value in track if value[0] < DAMPING_SPLIT_MS / 1_000.0]
        tail = [value for value in track if value[0] >= DAMPING_SPLIT_MS / 1_000.0]
        fit_slope = linear_slope(fit)
        tail_slope = linear_slope(tail)
        matched = tail_peaks[tail_index] if tail_index is not None else None
        modes.append(
            {
                "frequency_hz": fit_peak.frequency_hz,
                "relative_level_db": fit_peak.relative_level_db,
                "matched_tail_frequency_hz": (
                    matched.frequency_hz if matched is not None else None
                ),
                "frequency_error_cents": (
                    abs(cents_between(fit_peak.frequency_hz, matched.frequency_hz))
                    if matched is not None
                    else None
                ),
                "fit_decay_db_per_second": fit_slope,
                "tail_decay_db_per_second": tail_slope,
                "tail_prediction_rmse_db": anchored_tail_rmse(tail, fit_slope),
            }
        )
    persistent_errors = [
        value["frequency_error_cents"]
        for value in modes
        if value["frequency_error_cents"] is not None
    ]
    persistent_count = len(persistent_errors)
    selected_count = len(modes)
    persistent_recall = persistent_count / selected_count
    median_frequency_error = (
        median(persistent_errors) if persistent_errors else MATCH_TOLERANCE_CENTS * 2.0
    )
    decaying_fraction = (
        sum(value["fit_decay_db_per_second"] < -1.0 for value in modes)
        / selected_count
    )
    tail_rmse = median([value["tail_prediction_rmse_db"] for value in modes])
    loss = (
        4.0 * (1.0 - persistent_recall)
        + min(median_frequency_error / MATCH_TOLERANCE_CENTS, 2.0)
        + 2.0 * (1.0 - decaying_fraction)
        + min(tail_rmse / 20.0, 3.0)
    )
    return {
        "onset_sample": onset,
        "selected_mode_count": selected_count,
        "persistent_mode_count": persistent_count,
        "persistent_mode_recall": persistent_recall,
        "median_frequency_error_cents": median_frequency_error,
        "decaying_mode_fraction": decaying_fraction,
        "median_tail_prediction_rmse_db": tail_rmse,
        "calibration_loss": loss,
        "modes": modes,
    }


def compare_analysis(python: dict[str, Any], rust: dict[str, Any]) -> dict[str, float]:
    integer_fields = ["onset_sample", "selected_mode_count", "persistent_mode_count"]
    if any(python[field] != rust[field] for field in integer_fields):
        raise CalibrationError("Python/Rust extractor integer outputs differ")
    if len(python["modes"]) != len(rust["modes"]):
        raise CalibrationError("Python/Rust extractor mode counts differ")
    maximum_absolute = 0.0
    maximum_relative = 0.0
    pairs: list[tuple[Any, Any]] = []
    for field in [
        "persistent_mode_recall",
        "median_frequency_error_cents",
        "decaying_mode_fraction",
        "median_tail_prediction_rmse_db",
        "calibration_loss",
    ]:
        pairs.append((python[field], rust[field]))
    mode_fields = [
        "frequency_hz",
        "relative_level_db",
        "matched_tail_frequency_hz",
        "frequency_error_cents",
        "fit_decay_db_per_second",
        "tail_decay_db_per_second",
        "tail_prediction_rmse_db",
    ]
    for python_mode, rust_mode in zip(python["modes"], rust["modes"], strict=True):
        for field in mode_fields:
            pairs.append((python_mode[field], rust_mode[field]))
    for python_value, rust_value in pairs:
        if (python_value is None) != (rust_value is None):
            raise CalibrationError("Python/Rust extractor optional outputs differ")
        if python_value is None:
            continue
        absolute = abs(float(python_value) - float(rust_value))
        relative = absolute / max(abs(float(rust_value)), 1.0e-30)
        maximum_absolute = max(maximum_absolute, absolute)
        maximum_relative = max(maximum_relative, relative)
        if absolute > RUST_ABSOLUTE_TOLERANCE and relative > RUST_RELATIVE_TOLERANCE:
            raise CalibrationError(
                f"Python/Rust extractor parity failed: abs={absolute}, rel={relative}"
            )
    return {
        "maximum_absolute_error": maximum_absolute,
        "maximum_relative_error": maximum_relative,
        "absolute_tolerance": RUST_ABSOLUTE_TOLERANCE,
        "relative_tolerance": RUST_RELATIVE_TOLERANCE,
    }


def validate_fixture(path: Path) -> tuple[dict[str, Any], dict[str, float]]:
    report_bytes = read_exact(
        path / "report.json", FIXTURE_REPORT_SHA256, "Rust extractor fixture report"
    )
    report = json.loads(report_bytes)
    if (
        report.get("schema") != FIXTURE_REPORT_SCHEMA
        or report.get("decision") != "FrozenRustExtractorFixtureGenerated"
        or report.get("extractor_id") != EXTRACTOR_ID
        or report.get("network_requests") != 0
        or report.get("realimpact_payload_bytes_read") != 0
    ):
        raise CalibrationError("Rust extractor fixture report contract changed")
    sample_bytes = read_exact(
        path / report["sample_path"],
        FIXTURE_SAMPLE_SHA256,
        "Rust extractor fixture samples",
    )
    if len(sample_bytes) != report["sample_bytes"] or len(sample_bytes) % 8 != 0:
        raise CalibrationError("Rust extractor fixture sample dimensions changed")
    samples = np.frombuffer(sample_bytes, dtype="<f8").astype(np.float64, copy=True)
    python = analyze_v2(samples)
    return report, compare_analysis(python, report["analysis"])


def mapping_loss(
    measured: np.ndarray, eigenvalues: np.ndarray, alpha: float
) -> tuple[float, tuple[int, ...]]:
    errors = np.log2(measured[:, None] / (alpha * eigenvalues[None, :])) ** 2
    measured_count, proxy_count = errors.shape
    costs = np.full((measured_count, proxy_count), np.inf, dtype=np.float64)
    paths: list[list[tuple[int, ...] | None]] = [
        [None] * proxy_count for _ in range(measured_count)
    ]
    for proxy in range(proxy_count):
        costs[0, proxy] = errors[0, proxy]
        paths[0][proxy] = (proxy,)
    for mode in range(1, measured_count):
        for proxy in range(mode, proxy_count):
            choices = [
                (costs[mode - 1, previous], paths[mode - 1][previous])
                for previous in range(proxy)
                if paths[mode - 1][previous] is not None
            ]
            if not choices:
                continue
            prior_cost, prior_path = min(choices, key=lambda value: (value[0], value[1]))
            costs[mode, proxy] = prior_cost + errors[mode, proxy]
            paths[mode][proxy] = (*prior_path, proxy)  # type: ignore[arg-type]
    final = [
        (costs[-1, proxy], paths[-1][proxy])
        for proxy in range(proxy_count)
        if paths[-1][proxy] is not None
    ]
    if not final:
        raise CalibrationError("frequency mapping has no injective assignment")
    loss, path = min(final, key=lambda value: (value[0], value[1]))
    return float(loss), path  # type: ignore[return-value]


def calibrate_frequency_mapping(
    measured: np.ndarray, eigenvalues: np.ndarray
) -> dict[str, Any]:
    candidates = sorted(
        {
            float(frequency / eigenvalue)
            for frequency in measured
            for eigenvalue in eigenvalues
        }
    )
    scored = []
    for alpha in candidates:
        loss, path = mapping_loss(measured, eigenvalues, alpha)
        scored.append((loss, alpha, path))
    _, alpha, path = min(scored, key=lambda value: (value[0], value[1], value[2]))
    alpha = math.exp(
        math.fsum(
            math.log(float(measured[index] / eigenvalues[proxy]))
            for index, proxy in enumerate(path)
        )
        / len(path)
    )
    loss, path = mapping_loss(measured, eigenvalues, alpha)
    errors = np.abs(np.log2(measured / (alpha * eigenvalues[np.asarray(path)])))
    return {
        "alpha_metres_squared_per_second": alpha,
        "assignment": list(path),
        "squared_log2_loss": loss,
        "median_frequency_error_octaves": float(np.median(errors)),
        "p90_frequency_error_octaves": float(np.quantile(errors, 0.9, method="inverted_cdf")),
    }


def listener_coordinates() -> tuple[np.ndarray, list[tuple[int, int, int]]]:
    rows = []
    identities = []
    for angle_degrees in range(0, 181, 20):
        angle = math.radians(angle_degrees)
        for distance in [0, 333, 666, 1_000]:
            x = 0.23 + distance / 1_000.0
            y = -0.04345
            for microphone in range(15):
                z = -0.91 + microphone / 14.0 * 1.82
                rows.append(
                    [
                        math.cos(angle) * x - math.sin(angle) * y,
                        math.sin(angle) * x + math.cos(angle) * y,
                        z,
                    ]
                )
                identities.append((angle_degrees, distance, microphone))
    return np.asarray(rows, dtype=np.float64), identities


def validate_split() -> dict[str, Any]:
    coordinates, identities = listener_coordinates()
    anchors = [
        index
        for index, (angle, distance, microphone) in enumerate(identities)
        if angle in {0, 40, 80, 120, 160}
        and distance in {0, 666}
        and microphone in {0, 2, 4, 6, 7, 8, 10, 12, 14}
    ]
    held = [index for index in range(len(identities)) if index not in set(anchors)]
    if len(coordinates) != 600 or len(anchors) != 90 or len(held) != 510:
        raise CalibrationError("Pitcher listener split dimensions changed")
    if identities[7] != (0, 0, 7) or not np.array_equal(coordinates[7], [0.23, -0.04345, 0.0]):
        raise CalibrationError("Pitcher normalization row identity changed")
    return {
        "row_count": len(identities),
        "anchor_count": len(anchors),
        "held_count": len(held),
        "normalization_row_index": 7,
        "coordinate_sha256": sha256_bytes(np.ascontiguousarray(coordinates).tobytes()),
        "anchor_index_sha256": sha256_bytes(
            np.asarray(anchors, dtype="<i8").tobytes()
        ),
        "held_index_sha256": sha256_bytes(np.asarray(held, dtype="<i8").tobytes()),
    }


def publish(output: Path, manifest: bytes, report: bytes) -> None:
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise CalibrationError(f"output must be absent or empty: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "manifest.json").write_bytes(manifest)
        (staging / "report.json").write_bytes(report)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    args = parse_arguments()
    if not args.preflight_only:
        raise CalibrationError(
            "audio execution is not enabled in this runner revision; run --preflight-only"
        )
    manifest_bytes, manifest = load_manifest(args.manifest.resolve())
    geometry_path = args.manifest.resolve().parent / manifest["pitcher_geometry"]["path"]
    arrays = decode_geometry_block(geometry_path)
    fixture, parity = validate_fixture(args.rust_fixture.resolve())
    measured = np.asarray(
        [value["frequency_hz"] for value in fixture["analysis"]["modes"]],
        dtype=np.float64,
    )
    mapping = calibrate_frequency_mapping(measured, arrays["eigenvalues"])
    split = validate_split()
    script_hash = sha256_bytes(Path(__file__).read_bytes())
    report = {
        "schema": PREFLIGHT_REPORT_SCHEMA,
        "status": "Validated",
        "decision": "PitcherCalibrationRunnerPreflightSupported",
        "claim": "LOCAL_SYNTHETIC_PARITY_AND_FROZEN_INPUT_PREFLIGHT_ONLY / NO_NETWORK_OR_RESERVED_AUDIO_ACCESS / NO_REAL_SPATIAL_TRANSFER_CREDIT",
        "manifest_sha256": MANIFEST_SHA256,
        "script_sha256": script_hash,
        "geometry": {
            "sha256": GEOMETRY_SHA256,
            "bytes": GEOMETRY_BYTES,
            "mode_count": int(len(arrays["eigenvalues"])),
            "maximum_relative_eigen_residual": float(
                np.max(arrays["relative_eigen_residuals"])
            ),
        },
        "rust_fixture": {
            "report_sha256": FIXTURE_REPORT_SHA256,
            "sample_sha256": FIXTURE_SAMPLE_SHA256,
            "selected_mode_count": fixture["analysis"]["selected_mode_count"],
        },
        "python_rust_extractor_parity": parity,
        "synthetic_frequency_mapping_control": mapping,
        "listener_split": split,
        "network_requests": 0,
        "reserved_audio_payload_bytes_read": 0,
        "planter_audio_payload_bytes_read": 0,
        "audio_execution_enabled": False,
        "next_action": "freeze this runner revision and only then enable the exact one-request 512 MiB Pitcher prefix path; repeat analysis from the immutable cache and keep Planter sealed",
    }
    report_bytes = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode()
    publish(args.output.resolve(), manifest_bytes, report_bytes)
    print(f"Pitcher calibration runner preflight: {args.output.resolve()}")
    print(f"script sha256: {script_hash}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("reserved audio payload bytes read: 0")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except CalibrationError as error:
        print(f"pitcher calibration: {error}", file=sys.stderr)
        raise SystemExit(1) from error
