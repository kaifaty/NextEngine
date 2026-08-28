#!/usr/bin/env python3
"""Evaluate a scale-invariant, source-derived modal salience selector."""

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
import scipy

import physical_sound_adaptive_decay_control as adaptive
import physical_sound_multioutput_decay_control as multioutput
import physical_sound_realimpact_pitcher_calibration as single


MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-salience-selector-control.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-physical-sound-salience-selector-control-preflight."
        "report.v1"
    ),
    "run": "nextengine.experimental-physical-sound-salience-selector-control.report.v1",
}
STUDY_ID = "physical-sound-source-derived-salience-selector-control"
REVISION = "relative-spatial-persistence-salience-v1"

SAMPLE_RATE_HZ = adaptive.SAMPLE_RATE_HZ
SAMPLE_COUNT = adaptive.SAMPLE_COUNT
CHANNEL_COUNT = adaptive.CHANNEL_COUNT
ONSET_SAMPLE = adaptive.ONSET_SAMPLE
TAIL_START_MS = 900
RELATIVE_PEAK_FLOOR_DB = -55.0
DOMINANCE_FRACTION = 0.10
MAXIMUM_SALIENT_CANDIDATES = 64
SCALE_FACTORS = [0.125, 1.0, 8.0]
NOISE_SEED = 20_260_828
NOISE_STANDARD_DEVIATION = 2.0e-6

FREQUENCIES_HZ = [
    830.0,
    990.0,
    1_180.0,
    1_400.0,
    1_670.0,
    1_990.0,
    2_370.0,
    2_820.0,
    3_360.0,
    4_000.0,
    4_760.0,
    5_670.0,
    6_750.0,
    8_030.0,
    9_550.0,
    11_350.0,
]
AMPLITUDE_DECAY_PER_SECOND = [
    1.15,
    1.23,
    1.31,
    1.39,
    1.47,
    1.55,
    1.63,
    1.71,
    1.79,
    1.87,
    1.95,
    2.03,
    2.11,
    2.19,
    2.27,
    2.35,
]
TRANSIENT_FREQUENCIES_HZ = [
    270.0,
    284.0,
    299.0,
    315.0,
    332.0,
    350.0,
    369.0,
    389.0,
    410.0,
    432.0,
    456.0,
    481.0,
    507.0,
    535.0,
    564.0,
    595.0,
    628.0,
    663.0,
    699.0,
]
TRANSIENT_AMPLITUDES = [
    440.0,
    484.0,
    528.0,
    572.0,
    616.0,
    660.0,
    704.0,
    748.0,
    792.0,
    836.0,
    796.0,
    756.0,
    716.0,
    676.0,
    636.0,
    596.0,
    556.0,
    516.0,
    476.0,
]
TRANSIENT_DECAY_PER_SECOND = 30.0

MULTIOUTPUT_PYTHON_SHA256 = (
    "f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e"
)
ADAPTIVE_PYTHON_SHA256 = (
    "fe59b3d6c8845ed9433bab6f225625d53485f4245e7dac5968c4952eb8459c20"
)
EXTRACTOR_PYTHON_SHA256 = (
    "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
)
ADAPTIVE_CONTROL_REPORT_SHA256 = (
    "9fabc2bdcd7d174aafb28d672d4f8e55df96817f8bcf02131f9f1fafb6b0297f"
)
CERAMIC_REJECTION_REPORT_SHA256 = (
    "2ec3b03e08792c34ca5113c63ea8d6174c845fddd6c1aa2d4bac54e9ee33d6b7"
)

GATES = {
    "selected_mode_count": 16,
    "truth_match_count": 16,
    "maximum_false_positive_count": 0,
    "minimum_truth_recall": 1.0,
    "minimum_truth_precision": 1.0,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_rejected_transient_count": 1,
    "maximum_comparator_truth_recall": 0.50,
    "minimum_selector_recall_advantage": 0.50,
    "minimum_valid_adaptive_fit_fraction": 0.75,
    "maximum_median_adaptive_decay_error_db_per_second": 3.0,
    "minimum_median_fit_r_squared": 0.95,
    "scale_invariant_bin_selection": True,
}


class SalienceControlError(RuntimeError):
    """The frozen salience-control contract or numeric invariant failed."""


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
        raise SalienceControlError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise SalienceControlError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise SalienceControlError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "parents": {
            "adaptive_synthetic_control": {
                "path": "../ps2-adaptive-decay-control-v1/run-a/report.json",
                "sha256": ADAPTIVE_CONTROL_REPORT_SHA256,
            },
            "ceramic_adaptive_rejection": {
                "path": (
                    "../ps2-realimpact-ceramic-cup-observation-v1/"
                    "adaptive-analysis-a/report.json"
                ),
                "sha256": CERAMIC_REJECTION_REPORT_SHA256,
            },
        },
        "synthetic_fixture": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "sample_count": SAMPLE_COUNT,
            "channel_count": CHANNEL_COUNT,
            "onset_sample": ONSET_SAMPLE,
            "frequencies_hz": FREQUENCIES_HZ,
            "amplitude_decay_per_second": AMPLITUDE_DECAY_PER_SECOND,
            "transient_frequencies_hz": TRANSIENT_FREQUENCIES_HZ,
            "transient_amplitudes": TRANSIENT_AMPLITUDES,
            "transient_decay_per_second": TRANSIENT_DECAY_PER_SECOND,
            "modal_participation_rule": (
                "(0.65 + 0.035*((7*channel + 3*mode) mod 11))/(1 + 0.03*mode); "
                "deterministic phase"
            ),
            "transient_participation_rule": (
                "amplitude*(0.75 + 0.025*((5*channel + 2*artifact) mod 9)); "
                "deterministic phase"
            ),
            "noise": {
                "generator": "numpy.PCG64",
                "seed": NOISE_SEED,
                "standard_deviation": NOISE_STANDARD_DEVIATION,
            },
            "sample_format": "f64le-channel-major",
        },
        "candidate": {
            "id": REVISION,
            "input": "sum of Hann-windowed per-output FFT power",
            "frequency_range_hz": [
                single.MINIMUM_FREQUENCY_HZ,
                single.MAXIMUM_FREQUENCY_HZ,
            ],
            "fft_size": single.FFT_SIZE,
            "relative_peak_floor_db": RELATIVE_PEAK_FLOOR_DB,
            "dominance": {
                "fraction": DOMINANCE_FRACTION,
                "rule": (
                    "reject a local peak when any strictly stronger FFT bin lies within "
                    "+/-10 percent of its bin index"
                ),
                "maximum_candidates": MAXIMUM_SALIENT_CANDIDATES,
            },
            "persistence": {
                "tail_start_ms": TAIL_START_MS,
                "injective_match_tolerance_cents": single.MATCH_TOLERANCE_CENTS,
            },
            "scale_factors": SCALE_FACTORS,
            "normalization_boundary": {
                "decision": "relative-per-analysis-window",
                "reason": (
                    "the public per-object archive exposes deconvolved_0db.npy, while "
                    "the pinned notebook applies an absolute 5 dB threshold to "
                    "deconvolved.npy after an unavailable full-object global maximum"
                ),
                "claim_limit": (
                    "source-derived fractional dominance, not exact reproduction of the "
                    "notebook absolute threshold"
                ),
            },
            "source_provenance": {
                "realimpact_commit": "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987",
                "realimpact_notebook": {
                    "url": (
                        "https://raw.githubusercontent.com/samuel-clarke/RealImpact/"
                        "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/"
                        "modes_dsp_sweep.ipynb"
                    ),
                    "sha256": (
                        "2a8ce3c65e6fb3ac304602d3e2841779d917a3cbfdaedf72c9ee5146561fc7b4"
                    ),
                },
                "realimpact_preprocessing": {
                    "url": (
                        "https://raw.githubusercontent.com/samuel-clarke/RealImpact/"
                        "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/"
                        "preprocess_measurements.py"
                    ),
                    "sha256": (
                        "db55f2017a037fb50a7b8b59542e85e35a7c7d5581b15fb75b25dbdf67737bbc"
                    ),
                },
                "audio_dspy_commit": (
                    "2ad0b05f81b014c27612f6f91087265ee52e9238"
                ),
                "audio_dspy_modal_tools": {
                    "url": (
                        "https://raw.githubusercontent.com/jatinchowdhury18/audio_dspy/"
                        "2ad0b05f81b014c27612f6f91087265ee52e9238/"
                        "audio_dspy/modal_tools.py"
                    ),
                    "sha256": (
                        "a1e5b99112a06a7172c8dbaf270954f33d6e69f3041b15bf976e0eb585ccc494"
                    ),
                },
            },
        },
        "comparator": {
            "id": "spatial-modal-power-15-v1-top16-fixed-separation",
            "source": {
                "path": "lab/scripts/physical_sound_multioutput_decay_control.py",
                "sha256": MULTIOUTPUT_PYTHON_SHA256,
            },
        },
        "adaptive_estimator": {
            "id": adaptive.REVISION,
            "source": {
                "path": "lab/scripts/physical_sound_adaptive_decay_control.py",
                "sha256": ADAPTIVE_PYTHON_SHA256,
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
        "stop_rule": (
            "publish support or rejection once; on rejection do not weaken gates; on "
            "support preregister one previously unopened non-Planter object before any "
            "payload access"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise SalienceControlError("salience control manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise SalienceControlError(f"parse salience control manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise SalienceControlError("salience control manifest changed")
    return data, manifest


def resolve_report(
    base: Path, ref: dict[str, Any], label: str
) -> tuple[bytes, dict[str, Any]]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise SalienceControlError(f"{label} escapes external physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise SalienceControlError(f"{label} identity changed")
    return data, json.loads(data)


def validate_parents(base: Path, manifest: dict[str, Any]) -> dict[str, str]:
    adaptive_bytes, adaptive_report = resolve_report(
        base,
        manifest["parents"]["adaptive_synthetic_control"],
        "adaptive synthetic control",
    )
    ceramic_bytes, ceramic_report = resolve_report(
        base,
        manifest["parents"]["ceramic_adaptive_rejection"],
        "Ceramic adaptive rejection",
    )
    if (
        adaptive_report.get("decision") != "AdaptiveDecaySyntheticControlSupported"
        or adaptive_report.get("gate", {}).get("passed") is not True
        or adaptive_report.get("real_payload_bytes_read") != 0
        or ceramic_report.get("decision")
        != "CeramicCupAdaptiveDecayCounterfactualRejected"
        or ceramic_report.get("gate", {}).get("passed") is not False
        or ceramic_report.get("adaptive_analysis", {}).get("valid_fit_fraction")
        != 0.375
        or ceramic_report.get("network_requests") != 0
        or ceramic_report.get("physics_solver_runs") != 0
        or ceramic_report.get("planter_payload_bytes_read") != 0
    ):
        raise SalienceControlError("salience control parent lineage changed")
    return {
        "adaptive_synthetic_control_sha256": sha256_bytes(adaptive_bytes),
        "ceramic_adaptive_rejection_sha256": sha256_bytes(ceramic_bytes),
    }


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [
        manifest["comparator"]["source"],
        manifest["adaptive_estimator"]["source"],
        manifest["reference_extractor"]["source"],
    ]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or sha256_file(path) != ref["sha256"]:
            raise SalienceControlError(f"bound source changed: {ref['path']}")
        reports.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return reports


def generate_fixture() -> np.ndarray:
    channels = np.zeros((CHANNEL_COUNT, SAMPLE_COUNT), dtype=np.float64)
    time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE, dtype=np.float64) / SAMPLE_RATE_HZ
    for channel in range(CHANNEL_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, strict=True)
        ):
            phase = 0.113 * channel + 0.173 * mode + 0.019 * channel * mode
            scale = (0.65 + 0.035 * ((7 * channel + 3 * mode) % 11)) / (
                1.0 + 0.03 * mode
            )
            channels[channel, ONSET_SAMPLE:] += (
                scale
                * np.exp(-decay * time)
                * np.sin(2.0 * math.pi * frequency * time + phase)
            )
        for artifact, (frequency, amplitude) in enumerate(
            zip(TRANSIENT_FREQUENCIES_HZ, TRANSIENT_AMPLITUDES, strict=True)
        ):
            phase = 0.07 * channel + 0.11 * artifact
            scale = amplitude * (0.75 + 0.025 * ((5 * channel + 2 * artifact) % 9))
            channels[channel, ONSET_SAMPLE:] += (
                scale
                * np.exp(-TRANSIENT_DECAY_PER_SECOND * time)
                * np.sin(2.0 * math.pi * frequency * time + phase)
            )
    noise = np.random.Generator(np.random.PCG64(NOISE_SEED)).standard_normal(
        channels.shape
    )
    channels += NOISE_STANDARD_DEVIATION * noise
    return channels


def source_derived_candidates(
    channels: np.ndarray, start: int
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    power = multioutput.spatial_power_spectrum(channels, start, single.FFT_SIZE)
    bin_hz = SAMPLE_RATE_HZ / single.FFT_SIZE
    first = math.ceil(single.MINIMUM_FREQUENCY_HZ / bin_hz)
    last = min(
        math.floor(single.MAXIMUM_FREQUENCY_HZ / bin_hz), len(power) - 2
    )
    maximum = max(single.SAMPLE_EPSILON, float(np.max(power[first : last + 1])))
    local_peaks = []
    for index in range(first, last + 1):
        if power[index] <= power[index - 1] or power[index] < power[index + 1]:
            continue
        relative = 10.0 * math.log10(
            max(float(power[index]), single.SAMPLE_EPSILON) / maximum
        )
        if relative >= RELATIVE_PEAK_FLOOR_DB:
            local_peaks.append(
                {
                    "bin_index": index,
                    "frequency_hz": single.interpolated_frequency(power, index, bin_hz),
                    "relative_level_db": relative,
                }
            )
    salient = []
    for candidate in local_peaks:
        index = candidate["bin_index"]
        lower = max(first, math.floor((1.0 - DOMINANCE_FRACTION) * index))
        upper = min(last, math.ceil((1.0 + DOMINANCE_FRACTION) * index))
        if np.any(power[lower : upper + 1] > power[index]):
            continue
        salient.append(candidate)
    salient.sort(key=lambda value: (-value["relative_level_db"], value["frequency_hz"]))
    salient = salient[:MAXIMUM_SALIENT_CANDIDATES]
    salient.sort(key=lambda value: value["frequency_hz"])
    return local_peaks, salient


def persistent_candidates(
    onset: list[dict[str, Any]], tail: list[dict[str, Any]]
) -> tuple[list[dict[str, Any]], list[int | None]]:
    onset_peaks = [
        single.SpectralPeak(item["frequency_hz"], item["relative_level_db"])
        for item in onset
    ]
    tail_peaks = [
        single.SpectralPeak(item["frequency_hz"], item["relative_level_db"])
        for item in tail
    ]
    assignments = single.injective_matches(onset_peaks, tail_peaks)
    persistent = [
        candidate
        for candidate, assignment in zip(onset, assignments, strict=True)
        if assignment is not None
    ]
    return persistent, assignments


def classification(
    peaks: list[dict[str, Any]], truth_frequencies: list[float]
) -> dict[str, Any]:
    observed = [
        single.SpectralPeak(item["frequency_hz"], item.get("relative_level_db", 0.0))
        for item in peaks
    ]
    truth = [single.SpectralPeak(frequency, 0.0) for frequency in truth_frequencies]
    assignments = single.injective_matches(observed, truth)
    errors = [
        abs(single.cents_between(observed[index].frequency_hz, truth[assignment].frequency_hz))
        for index, assignment in enumerate(assignments)
        if assignment is not None
    ]
    matches = len(errors)
    selected = len(observed)
    return {
        "selected_count": selected,
        "truth_count": len(truth),
        "truth_match_count": matches,
        "false_positive_count": selected - matches,
        "truth_recall": matches / len(truth),
        "truth_precision": matches / selected if selected else 0.0,
        "median_frequency_error_cents": single.median(errors) if errors else None,
        "assignments": assignments,
    }


def analyze_candidate(channels: np.ndarray) -> dict[str, Any]:
    local_onset, salient_onset = source_derived_candidates(channels, ONSET_SAMPLE)
    local_tail, salient_tail = source_derived_candidates(
        channels, ONSET_SAMPLE + TAIL_START_MS * SAMPLE_RATE_HZ // 1_000
    )
    persistent, assignments = persistent_candidates(salient_onset, salient_tail)
    metrics = classification(persistent, FREQUENCIES_HZ)
    transient = classification(persistent, TRANSIENT_FREQUENCIES_HZ)
    return {
        "local_onset_candidate_count": len(local_onset),
        "salient_onset_candidate_count": len(salient_onset),
        "local_tail_candidate_count": len(local_tail),
        "salient_tail_candidate_count": len(salient_tail),
        "persistent_candidate_count": len(persistent),
        "rejected_transient_count": len(salient_onset) - len(persistent),
        "persistent_bin_indices": [item["bin_index"] for item in persistent],
        "persistent_frequencies_hz": [item["frequency_hz"] for item in persistent],
        "onset_to_tail_assignments": assignments,
        "truth": metrics,
        "persistent_transient_matches": transient["truth_match_count"],
        "salient_onset": salient_onset,
        "salient_tail": salient_tail,
        "persistent": persistent,
    }


def analyze_comparator(channels: np.ndarray) -> dict[str, Any]:
    peaks = multioutput.spatial_modal_peaks(channels, ONSET_SAMPLE)
    serialized = [
        {
            "frequency_hz": peak.frequency_hz,
            "relative_level_db": peak.relative_level_db,
        }
        for peak in peaks
    ]
    return {"peaks": serialized, "truth": classification(serialized, FREQUENCIES_HZ)}


def analyze_scale_invariance(channels: np.ndarray) -> dict[str, Any]:
    variants = []
    signatures = []
    for scale in SCALE_FACTORS:
        analysis = analyze_candidate(channels * scale)
        signature = analysis["persistent_bin_indices"]
        signatures.append(signature)
        variants.append(
            {
                "scale": scale,
                "persistent_bin_indices": signature,
                "persistent_frequency_sha256": sha256_bytes(canonical_json(signature)),
                "truth_match_count": analysis["truth"]["truth_match_count"],
                "false_positive_count": analysis["truth"]["false_positive_count"],
            }
        )
    return {
        "passed": all(signature == signatures[0] for signature in signatures[1:]),
        "variants": variants,
    }


def analyze_adaptive(
    channels: np.ndarray, persistent: list[dict[str, Any]]
) -> dict[str, Any]:
    modes = [
        adaptive.adaptive_fit(
            channels,
            single.SpectralPeak(item["frequency_hz"], item["relative_level_db"]),
        )
        for item in persistent
    ]
    comparisons = []
    for mode, expected_frequency, decay in zip(
        modes, FREQUENCIES_HZ, AMPLITUDE_DECAY_PER_SECOND, strict=True
    ):
        expected_decay = -20.0 * decay / math.log(10.0)
        comparisons.append(
            {
                "expected_frequency_hz": expected_frequency,
                "observed_frequency_hz": mode["frequency_hz"],
                "expected_decay_db_per_second": expected_decay,
                "observed_decay_db_per_second": mode["fit_decay_db_per_second"],
                "decay_error_db_per_second": (
                    abs(mode["fit_decay_db_per_second"] - expected_decay)
                    if mode["valid"]
                    else None
                ),
            }
        )
    valid_modes = [mode for mode in modes if mode["valid"]]
    valid_comparisons = [
        item for item in comparisons if item["decay_error_db_per_second"] is not None
    ]
    return {
        "selected_mode_count": len(modes),
        "valid_fit_count": len(valid_modes),
        "valid_fit_fraction": len(valid_modes) / len(modes) if modes else 0.0,
        "median_decay_error_db_per_second": (
            single.median(
                [item["decay_error_db_per_second"] for item in valid_comparisons]
            )
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


def check(name: str, observed: Any, relation: str, threshold: Any) -> dict[str, Any]:
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
        "decision": "SalienceSelectorSyntheticControlFrozen",
        "claim": (
            "SYNTHETIC_SOURCE_DERIVED_SALIENCE_PREFLIGHT_ONLY / "
            "NO_REAL_DATA_PHYSICS_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "bound_sources": sources,
        "synthetic_fixture": manifest["synthetic_fixture"],
        "candidate": manifest["candidate"],
        "comparator": manifest["comparator"],
        "adaptive_estimator": manifest["adaptive_estimator"],
        "runtime": manifest["runtime"],
        "gates": GATES,
        "next_action": (
            "commit this preflight, then execute the deterministic salience-selector "
            "control twice"
        ),
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
    fixture_sha256 = sha256_bytes(
        channels.astype("<f8", copy=False).tobytes(order="C")
    )
    candidate = analyze_candidate(channels)
    comparator = analyze_comparator(channels)
    scale_invariance = analyze_scale_invariance(channels)
    adaptive_analysis = analyze_adaptive(channels, candidate["persistent"])
    candidate_truth = candidate["truth"]
    comparator_truth = comparator["truth"]
    recall_advantage = candidate_truth["truth_recall"] - comparator_truth["truth_recall"]
    adaptive_error = (
        adaptive_analysis["median_decay_error_db_per_second"]
        if adaptive_analysis["median_decay_error_db_per_second"] is not None
        else 1.0e9
    )
    adaptive_r_squared = (
        adaptive_analysis["median_fit_r_squared"]
        if adaptive_analysis["median_fit_r_squared"] is not None
        else 0.0
    )
    checks = [
        check(
            "selected_mode_count",
            candidate_truth["selected_count"],
            "==",
            GATES["selected_mode_count"],
        ),
        check(
            "truth_match_count",
            candidate_truth["truth_match_count"],
            "==",
            GATES["truth_match_count"],
        ),
        check(
            "false_positive_count",
            candidate_truth["false_positive_count"],
            "<=",
            GATES["maximum_false_positive_count"],
        ),
        check(
            "truth_recall",
            candidate_truth["truth_recall"],
            ">=",
            GATES["minimum_truth_recall"],
        ),
        check(
            "truth_precision",
            candidate_truth["truth_precision"],
            ">=",
            GATES["minimum_truth_precision"],
        ),
        check(
            "median_frequency_error_cents",
            (
                candidate_truth["median_frequency_error_cents"]
                if candidate_truth["median_frequency_error_cents"] is not None
                else 1.0e9
            ),
            "<=",
            GATES["maximum_median_frequency_error_cents"],
        ),
        check(
            "rejected_transient_count",
            candidate["rejected_transient_count"],
            ">=",
            GATES["minimum_rejected_transient_count"],
        ),
        check(
            "persistent_transient_matches",
            candidate["persistent_transient_matches"],
            "==",
            0,
        ),
        check(
            "comparator_truth_recall",
            comparator_truth["truth_recall"],
            "<=",
            GATES["maximum_comparator_truth_recall"],
        ),
        check(
            "selector_recall_advantage",
            recall_advantage,
            ">=",
            GATES["minimum_selector_recall_advantage"],
        ),
        check(
            "scale_invariant_bin_selection",
            scale_invariance["passed"],
            "==",
            GATES["scale_invariant_bin_selection"],
        ),
        check(
            "valid_adaptive_fit_fraction",
            adaptive_analysis["valid_fit_fraction"],
            ">=",
            GATES["minimum_valid_adaptive_fit_fraction"],
        ),
        check(
            "median_adaptive_decay_error_db_per_second",
            adaptive_error,
            "<=",
            GATES["maximum_median_adaptive_decay_error_db_per_second"],
        ),
        check(
            "median_fit_r_squared",
            adaptive_r_squared,
            ">=",
            GATES["minimum_median_fit_r_squared"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["run"],
        "status": "Validated",
        "decision": (
            "SalienceSelectorSyntheticControlSupported"
            if passed
            else "SalienceSelectorSyntheticControlRejected"
        ),
        "claim": (
            "SYNTHETIC_SOURCE_DERIVED_SCALE_INVARIANT_SALIENCE_CONTROL_ONLY / "
            "NO_REAL_DATA_PHYSICS_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "synthetic_fixture_sha256": fixture_sha256,
        "synthetic_fixture_bytes": channels.nbytes,
        "candidate_analysis": candidate,
        "comparator_analysis": comparator,
        "scale_invariance": scale_invariance,
        "adaptive_analysis": adaptive_analysis,
        "selector_recall_advantage": recall_advantage,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "preregister one previously unopened non-Planter object without payload access"
            if passed
            else "reject this selector revision; do not weaken gates or access real data"
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
    print(f"Salience selector control {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("real payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
