#!/usr/bin/env python3
"""Run a frozen adaptive-decay counterfactual on existing Ceramic Cup rows."""

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

import physical_sound_adaptive_decay_control as adaptive
import physical_sound_multioutput_decay_control as multioutput
import physical_sound_realimpact_pitcher_calibration as single


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-adaptive-decay-counterfactual.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-adaptive-decay-counterfactual-preflight.report.v1",
    "analyze": "nextengine.experimental-realimpact-adaptive-decay-counterfactual.report.v1",
}
STUDY_ID = "physical-sound-realimpact-observation-first-discriminator"
REVISION = "ceramic-cup-fixed-input-spatial-adaptive-rms-envelope-v1"
OBJECT_ID = "78_CeramicCup"
ROW_COUNT = 600
SAMPLE_COUNT = 208_895
DECODED_BYTES = 501_348_000
ANALYSIS_ROWS = list(range(15))
REFERENCE_ROW = 7
DECODED_SHA256 = (
    "3405843a7a4bfe825dedf9e4406bce2898b2270de1bb273eb8e91a4a247be6ca"
)
DECODE_REPORT_SHA256 = (
    "9f1c23118ac2d451bd423d2f2a8deb604ad5ccd085900501682b6e31e71adaf0"
)
OBSERVATION_REPORT_SHA256 = (
    "56591bb82ab50470296013c432c57e2b31b5addbae002ccbc833d595f3823fd9"
)
FIXED_COUNTERFACTUAL_SHA256 = (
    "9947c427a3416b3a3635c43ea24e074df84ce4db97f0a22cca3be8c93ac96cbf"
)
ADAPTIVE_CONTROL_SHA256 = (
    "9fabc2bdcd7d174aafb28d672d4f8e55df96817f8bcf02131f9f1fafb6b0297f"
)
ADAPTIVE_SOURCE_SHA256 = (
    "fe59b3d6c8845ed9433bab6f225625d53485f4245e7dac5968c4952eb8459c20"
)
MULTIOUTPUT_SOURCE_SHA256 = (
    "f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e"
)
FIXED_DECAY_FRACTION = 0.1875

THRESHOLDS = {
    "minimum_selected_modes": 6,
    "minimum_persistent_mode_recall": 0.50,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_valid_adaptive_fit_fraction": 0.75,
    "minimum_decaying_mode_fraction": 0.50,
    "minimum_median_fit_r_squared": 0.95,
    "minimum_decaying_fraction_improvement": 0.25,
}


class CeramicAdaptiveError(RuntimeError):
    """The frozen real adaptive counterfactual or lineage failed."""


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
        raise CeramicAdaptiveError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise CeramicAdaptiveError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise CeramicAdaptiveError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "opened_development",
            "impact_ordinal": 0,
        },
        "parents": {
            "observation_report": {
                "path": "observation-analysis-a/report.json",
                "sha256": OBSERVATION_REPORT_SHA256,
            },
            "decode_report": {
                "path": "observation-decode/report.json",
                "sha256": DECODE_REPORT_SHA256,
            },
            "fixed_counterfactual_rejection": {
                "path": "multioutput-analysis-a/report.json",
                "sha256": FIXED_COUNTERFACTUAL_SHA256,
            },
            "adaptive_synthetic_control": {
                "path": "../ps2-adaptive-decay-control-v1/run-a/report.json",
                "sha256": ADAPTIVE_CONTROL_SHA256,
            },
            "decoded_block": {
                "path": "observation-decode/ceramic-cup-impact000-rows000-599.f32le",
                "sha256": DECODED_SHA256,
                "bytes": DECODED_BYTES,
                "shape": [ROW_COUNT, SAMPLE_COUNT],
                "dtype": "<f4",
            },
        },
        "fixed_input": {
            "rows": ANALYSIS_ROWS,
            "reference_row": REFERENCE_ROW,
            "condition": {
                "angle_degrees": 0,
                "distance_millimetres": 0,
                "microphone_ids": ANALYSIS_ROWS,
            },
            "selection_rule": "same rows 0..14 from the rejected fixed-window counterfactual; no value-dependent selection",
        },
        "candidate": {
            "id": "spatial-adaptive-rms-envelope-v1",
            "source": {
                "path": "lab/scripts/physical_sound_adaptive_decay_control.py",
                "sha256": ADAPTIVE_SOURCE_SHA256,
            },
            "frequency_selection": "unchanged spatial-modal-power-15-v1",
            "filter_width_hz": adaptive.FILTER_WIDTH_HZ,
            "filter_order": adaptive.FILTER_ORDER,
            "envelope_eta_seconds": adaptive.ENVELOPE_ETA_SECONDS,
            "noise_floor_tail_fraction": adaptive.NOISE_FLOOR_TAIL_FRACTION,
            "start_dynamic_fraction": adaptive.START_DYNAMIC_FRACTION,
            "end_dynamic_fraction": adaptive.END_DYNAMIC_FRACTION,
            "minimum_dynamic_range_db": adaptive.MINIMUM_DYNAMIC_RANGE_DB,
            "minimum_fit_seconds": adaptive.MINIMUM_FIT_SECONDS,
        },
        "fixed_comparator": {
            "decaying_mode_fraction": FIXED_DECAY_FRACTION,
            "source": {
                "path": "lab/scripts/physical_sound_multioutput_decay_control.py",
                "sha256": MULTIOUTPUT_SOURCE_SHA256,
            },
        },
        "thresholds": THRESHOLDS,
        "data_policy": {
            "offline_existing_decoded_block_only": True,
            "network_or_additional_payload_allowed": False,
            "row_selection_threshold_tuning_or_denoising_allowed": False,
            "physics_solver_allowed": False,
            "planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": "publish support or rejection once; do not tune, select rows, fetch, run physics or open Planter",
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise CeramicAdaptiveError("Ceramic adaptive manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise CeramicAdaptiveError(f"parse Ceramic adaptive manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise CeramicAdaptiveError("Ceramic adaptive manifest changed")
    return data, manifest


def resolve_report(base: Path, ref: dict[str, Any], label: str) -> tuple[bytes, dict[str, Any]]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise CeramicAdaptiveError(f"{label} escapes external physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise CeramicAdaptiveError(f"{label} identity changed")
    return data, json.loads(data)


def decoded_block(base: Path, ref: dict[str, Any], hash_payload: bool) -> Path:
    path = (base / ref["path"]).resolve(strict=True)
    if (
        not path.is_file()
        or not path.is_relative_to(base.parent)
        or path.stat().st_size != ref["bytes"]
    ):
        raise CeramicAdaptiveError("decoded block identity or size changed")
    if hash_payload and sha256_file(path) != ref["sha256"]:
        raise CeramicAdaptiveError("decoded block hash changed")
    return path


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [manifest["candidate"]["source"], manifest["fixed_comparator"]["source"]]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or sha256_file(path) != ref["sha256"]:
            raise CeramicAdaptiveError(f"bound source changed: {ref['path']}")
        reports.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return reports


def validate_parents(
    base: Path, manifest: dict[str, Any], hash_payload: bool
) -> tuple[dict[str, Any], Path]:
    refs = manifest["parents"]
    observation_bytes, observation = resolve_report(
        base, refs["observation_report"], "observation report"
    )
    decode_bytes, decode = resolve_report(base, refs["decode_report"], "decode report")
    fixed_bytes, fixed = resolve_report(
        base, refs["fixed_counterfactual_rejection"], "fixed counterfactual"
    )
    adaptive_bytes, adaptive_report = resolve_report(
        base, refs["adaptive_synthetic_control"], "adaptive synthetic control"
    )
    block = decoded_block(base, refs["decoded_block"], hash_payload)
    if (
        observation.get("decision") != "CeramicCupObservationRejected"
        or observation.get("decoded_sha256") != DECODED_SHA256
        or decode.get("decision") != "CeramicCupImpactZeroObservationDecoded"
        or decode.get("decoded_sha256") != DECODED_SHA256
        or decode.get("decoded_bytes") != DECODED_BYTES
        or fixed.get("decision") != "CeramicCupMultiOutputCounterfactualRejected"
        or fixed.get("candidate_analysis", {}).get("decaying_mode_fraction")
        != FIXED_DECAY_FRACTION
        or adaptive_report.get("decision") != "AdaptiveDecaySyntheticControlSupported"
        or adaptive_report.get("gate", {}).get("passed") is not True
    ):
        raise CeramicAdaptiveError("Ceramic adaptive parent lineage changed")
    return (
        {
            "observation_report_sha256": sha256_bytes(observation_bytes),
            "decode_report_sha256": sha256_bytes(decode_bytes),
            "fixed_counterfactual_sha256": sha256_bytes(fixed_bytes),
            "adaptive_synthetic_control_sha256": sha256_bytes(adaptive_bytes),
            "decoded_sha256": DECODED_SHA256,
            "decoded_bytes": block.stat().st_size,
            "decoded_payload_rehashed": hash_payload,
        },
        block,
    )


def filtered_spatial_power(channels: np.ndarray, frequency_hz: float) -> np.ndarray:
    low = (frequency_hz - adaptive.FILTER_WIDTH_HZ / 2.0) / (
        adaptive.SAMPLE_RATE_HZ / 2.0
    )
    high = (frequency_hz + adaptive.FILTER_WIDTH_HZ / 2.0) / (
        adaptive.SAMPLE_RATE_HZ / 2.0
    )
    highpass_b, highpass_a = signal.butter(adaptive.FILTER_ORDER, low, btype="highpass")
    lowpass_b, lowpass_a = signal.butter(adaptive.FILTER_ORDER, high, btype="lowpass")
    power = np.zeros(channels.shape[1], dtype=np.float64)
    for channel in channels:
        filtered = signal.lfilter(lowpass_b, lowpass_a, channel)
        filtered = signal.lfilter(highpass_b, highpass_a, filtered)
        power += filtered * filtered
    return power


def rms_envelope(power: np.ndarray) -> np.ndarray:
    pole = math.exp(
        -1.0 / (adaptive.ENVELOPE_ETA_SECONDS * adaptive.SAMPLE_RATE_HZ)
    )
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
    tail_start = int(len(levels) * (1.0 - adaptive.NOISE_FLOOR_TAIL_FRACTION))
    noise_floor_db = float(np.max(levels[tail_start:]))
    peak_db = float(levels[peak_sample])
    dynamic_range_db = peak_db - noise_floor_db
    start_threshold = noise_floor_db + dynamic_range_db * adaptive.START_DYNAMIC_FRACTION
    end_threshold = noise_floor_db + dynamic_range_db * adaptive.END_DYNAMIC_FRACTION
    fit_start = first_below(levels, peak_sample, start_threshold)
    fit_end = first_below(levels, fit_start or peak_sample, end_threshold)
    minimum_samples = round(adaptive.MINIMUM_FIT_SECONDS * adaptive.SAMPLE_RATE_HZ)
    valid = (
        dynamic_range_db >= adaptive.MINIMUM_DYNAMIC_RANGE_DB
        and fit_start is not None
        and fit_end is not None
        and fit_end - fit_start >= minimum_samples
    )
    report = {
        "frequency_hz": peak.frequency_hz,
        "valid": valid,
        "peak_sample": peak_sample,
        "noise_floor_db": noise_floor_db,
        "dynamic_range_db": dynamic_range_db,
        "fit_start_sample": fit_start,
        "fit_end_sample": fit_end,
        "fit_decay_db_per_second": None,
        "fit_r_squared": None,
    }
    if not valid:
        return report
    assert fit_start is not None and fit_end is not None
    x = np.arange(fit_start, fit_end, dtype=np.float64) / adaptive.SAMPLE_RATE_HZ
    y = levels[fit_start:fit_end]
    slope, intercept = np.polyfit(x, y, 1)
    predicted = slope * x + intercept
    residual = float(np.sum((y - predicted) ** 2))
    total = float(np.sum((y - np.mean(y)) ** 2))
    report["fit_decay_db_per_second"] = float(slope)
    report["fit_r_squared"] = (
        1.0 - residual / total if total > single.SAMPLE_EPSILON else 0.0
    )
    return report


def analyze_adaptive(channels: np.ndarray) -> dict[str, Any]:
    norm = np.sqrt(np.sum(channels * channels, axis=0))
    maximum = float(np.max(norm))
    onset_indices = np.flatnonzero(norm >= maximum * single.ONSET_PEAK_FRACTION)
    if not len(onset_indices):
        raise CeramicAdaptiveError("Ceramic spatial onset was not found")
    onset = int(onset_indices[0])
    peaks = multioutput.spatial_modal_peaks(channels, onset)
    tail_peaks = multioutput.spatial_modal_peaks(
        channels, onset + single.TAIL_START_MS * adaptive.SAMPLE_RATE_HZ // 1_000
    )
    assignments = single.injective_matches(peaks, tail_peaks)
    modes = []
    for peak, tail_index in zip(peaks, assignments, strict=True):
        mode = adaptive_fit(channels, peak)
        tail_peak = tail_peaks[tail_index] if tail_index is not None else None
        mode["matched_tail_frequency_hz"] = tail_peak.frequency_hz if tail_peak else None
        mode["frequency_error_cents"] = (
            abs(single.cents_between(peak.frequency_hz, tail_peak.frequency_hz))
            if tail_peak
            else None
        )
        modes.append(mode)
    valid = [mode for mode in modes if mode["valid"]]
    persistent = [mode["frequency_error_cents"] for mode in modes if mode["frequency_error_cents"] is not None]
    decaying = sum(
        mode["valid"] and mode["fit_decay_db_per_second"] < -1.0 for mode in modes
    )
    return {
        "onset_sample": onset,
        "selected_mode_count": len(modes),
        "persistent_mode_count": len(persistent),
        "persistent_mode_recall": len(persistent) / len(modes),
        "median_frequency_error_cents": single.median(persistent) if persistent else 80.0,
        "valid_fit_count": len(valid),
        "valid_fit_fraction": len(valid) / len(modes),
        "decaying_mode_fraction": decaying / len(modes),
        "median_fit_r_squared": (
            single.median([mode["fit_r_squared"] for mode in valid]) if valid else 0.0
        ),
        "median_decay_db_per_second": (
            single.median([mode["fit_decay_db_per_second"] for mode in valid])
            if valid
            else None
        ),
        "modes": modes,
    }


def check(name: str, observed: float, relation: str, threshold: float) -> dict[str, Any]:
    passed = observed >= threshold if relation == ">=" else observed <= threshold
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
        "dataset_object_id": OBJECT_ID,
        "analysis_rows": ANALYSIS_ROWS,
        "reference_row": REFERENCE_ROW,
        "network_requests": 0,
        "additional_payload_bytes_read": 0,
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
    parents, _ = validate_parents(base, manifest, hash_payload=False)
    sources = validate_sources(root, manifest)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "CeramicCupAdaptiveDecayCounterfactualFrozen",
        "claim": "READ_ONLY_FIXED_INPUT_ADAPTIVE_PREFLIGHT_ONLY / NO_REAL_PAYLOAD_BYTES_READ_OR_ADMISSION_PHYSICS_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "existing_real_payload_bytes_read": 0,
        "parents": parents,
        "bound_sources": sources,
        "fixed_input": manifest["fixed_input"],
        "candidate": manifest["candidate"],
        "fixed_comparator_decaying_mode_fraction": FIXED_DECAY_FRACTION,
        "thresholds": THRESHOLDS,
        "next_action": "commit this preflight, then execute the immutable Ceramic adaptive counterfactual twice",
    }


def analyze(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents, block = validate_parents(base, manifest, hash_payload=True)
    validate_sources(root, manifest)
    values = np.memmap(block, mode="r", dtype="<f4", shape=(ROW_COUNT, SAMPLE_COUNT))
    channels = np.asarray(values[ANALYSIS_ROWS], dtype=np.float64)
    result = analyze_adaptive(channels)
    improvement = result["decaying_mode_fraction"] - FIXED_DECAY_FRACTION
    checks = [
        check("selected_mode_count", result["selected_mode_count"], ">=", THRESHOLDS["minimum_selected_modes"]),
        check("persistent_mode_recall", result["persistent_mode_recall"], ">=", THRESHOLDS["minimum_persistent_mode_recall"]),
        check("median_frequency_error_cents", result["median_frequency_error_cents"], "<=", THRESHOLDS["maximum_median_frequency_error_cents"]),
        check("valid_adaptive_fit_fraction", result["valid_fit_fraction"], ">=", THRESHOLDS["minimum_valid_adaptive_fit_fraction"]),
        check("decaying_mode_fraction", result["decaying_mode_fraction"], ">=", THRESHOLDS["minimum_decaying_mode_fraction"]),
        check("median_fit_r_squared", result["median_fit_r_squared"], ">=", THRESHOLDS["minimum_median_fit_r_squared"]),
        check("decaying_fraction_improvement", improvement, ">=", THRESHOLDS["minimum_decaying_fraction_improvement"]),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": (
            "CeramicCupAdaptiveDecayCounterfactualSupported"
            if passed
            else "CeramicCupAdaptiveDecayCounterfactualRejected"
        ),
        "claim": "READ_ONLY_FIXED_INPUT_ADAPTIVE_COUNTERFACTUAL_ONLY / NO_ADMISSION_PHYSICS_QUALITY_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "existing_payload_bytes_hashed": DECODED_BYTES,
        "existing_payload_bytes_analyzed": len(ANALYSIS_ROWS) * SAMPLE_COUNT * 4,
        "parents": parents,
        "adaptive_analysis": result,
        "fixed_comparator_decaying_mode_fraction": FIXED_DECAY_FRACTION,
        "decaying_fraction_improvement": improvement,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "publish method support only; require independent unopened-object validation before admission or mechanics"
            if passed
            else "reject this adaptive counterfactual; do not tune, select rows, fetch, run physics or open Planter"
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
        else analyze(root, base, manifest_bytes, manifest, runner_sha256)
    )
    report_bytes = publish(output, report)
    print(f"Ceramic Cup adaptive counterfactual {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("additional payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
