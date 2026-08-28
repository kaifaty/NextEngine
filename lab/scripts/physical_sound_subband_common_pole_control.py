#!/usr/bin/env python3
"""Prove a bounded multichannel Gabor/subband common-pole estimator."""

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


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-subband-pole.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-physical-sound-subband-pole-preflight.report.v1"
    ),
    "run": "nextengine.experimental-physical-sound-subband-pole.report.v1",
}
STUDY_ID = "physical-sound-subband-common-pole-control"
REVISION = "multichannel-gabor-esprit-core-v1"

PARENT_REPORT_SHA256 = (
    "02551f2995764197575d5dd30bb36bd44da5452e2af79533b43922f44fec48d1"
)
PARENT_RESULT_DOC_SHA256 = (
    "a236f27be0dd5264abed82dd2ea52e71dc34b929ccf1f6efcd10486dc75cf5e9"
)

SAMPLE_RATE_HZ = 48_000
SAMPLE_COUNT = 60_000
CHANNEL_COUNT = 15
ONSET_SAMPLE = 240
WINDOW_SAMPLES = 1_024
HOP_SAMPLES = 32
GABOR_FRAME_COUNT = 256
PENCIL_LAGS = 24
MAXIMUM_ORDER = 6
BAND_BINS = [24, 64, 128, 192]
BAND_CENTERS_HZ = [value * SAMPLE_RATE_HZ / WINDOW_SAMPLES for value in BAND_BINS]
FREQUENCY_GROUPS_HZ = [
    [1_108.0, 1_142.0],
    [2_986.0, 3_017.0],
    [5_992.0],
    [8_990.0],
]
DECAY_GROUPS_PER_SECOND = [
    [2.0, 4.0],
    [3.0, 7.0],
    [5.0],
    [12.0],
]
EXPECTED_ORDERS = [2, 2, 1, 1]
NODE_CHANNEL = 7
NODE_MODE_FLAT_INDEX = 1
TRANSIENT_DECAY_PER_SECOND = 80.0
TRANSIENT_GAIN = 0.8
NOISE_STANDARD_DEVIATION = 1.0e-7
NOISE_SEED = 20_260_829
SCALE_FACTORS = [0.125, 1.0, 8.0]

GATES = {
    "expected_band_orders": EXPECTED_ORDERS,
    "expected_truth_mode_count": 6,
    "maximum_frequency_error_hz": 0.25,
    "maximum_decay_error_per_second": 0.50,
    "minimum_order_score_margin": 1_000.0,
    "expected_node_channel_order": 1,
    "expected_node_channel_missed_truth_count": 1,
    "maximum_scale_frequency_delta_hz": 0.0,
    "maximum_scale_decay_delta_per_second": 0.0,
    "scale_invariant_order": True,
    "close_pairs_below_one_fft_bin": 2,
}

RESEARCH_SOURCES = [
    {
        "id": "gabor-esprit-impact-analysis",
        "url": "https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf",
        "retrieved_sha256": (
            "d52c9a3fd1e9d14e23cdabca7c03ac39374a38c05f13ea7836236063d979bdd0"
        ),
        "bounded_claim": (
            "Gabor subbands preserve damped-exponential structure and allow "
            "longer-horizon ESPRIT analysis of impact sounds"
        ),
    },
    {
        "id": "ester-order-analysis",
        "url": (
            "https://perso.telecom-paristech.fr/grichard/Publications/"
            "SP06_Badeau1.pdf"
        ),
        "retrieved_sha256": (
            "bae56cfedd505b71bf01462a9a54f5639415178b50800099178708700d58dc07"
        ),
        "bounded_claim": (
            "ESPRIT requires model-order estimation and rotational-invariance "
            "error can discriminate candidate orders"
        ),
    },
    {
        "id": "svd-eds-order-selection",
        "url": "https://ftp.esat.kuleuven.be/stadius/ida/reports/05-107.pdf",
        "retrieved_sha256": (
            "d3e382423eabc54555d42bab102efb2d8eb0ae1ab6c3eeb61607c909954d25fd"
        ),
        "bounded_claim": (
            "an SVD signal subspace and shift-invariance residual support direct "
            "order selection for exponentially damped sinusoids"
        ),
    },
]


class ControlError(RuntimeError):
    """The frozen common-pole control or its parent lineage failed."""


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


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "parent": {
            "diagnostic_report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-selector-tail-diagnostic-v1/"
                    "analysis-a/report.json"
                ),
                "sha256": PARENT_REPORT_SHA256,
            },
            "result_document": {
                "path": (
                    "docs/development/physical-sound-realimpact-selector-tail-"
                    "diagnostic-result-ps2-2026-08-28.md"
                ),
                "sha256": PARENT_RESULT_DOC_SHA256,
            },
        },
        "fixture": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "sample_count": SAMPLE_COUNT,
            "channel_count": CHANNEL_COUNT,
            "onset_sample": ONSET_SAMPLE,
            "frequency_groups_hz": FREQUENCY_GROUPS_HZ,
            "decay_groups_per_second": DECAY_GROUPS_PER_SECOND,
            "participation_rule": (
                "0.45 + 0.035*((7*channel + 3*mode) mod 11); deterministic phase"
            ),
            "node": {
                "channel": NODE_CHANNEL,
                "mode_flat_index": NODE_MODE_FLAT_INDEX,
                "participation": 0.0,
            },
            "transient": {
                "generator": "shared numpy.PCG64 broadband burst",
                "decay_per_second": TRANSIENT_DECAY_PER_SECOND,
                "gain": TRANSIENT_GAIN,
            },
            "noise": {
                "generator": "numpy.PCG64 independent outputs",
                "seed": NOISE_SEED,
                "standard_deviation": NOISE_STANDARD_DEVIATION,
            },
            "sample_format": "f64le-channel-major",
        },
        "candidate": {
            "id": REVISION,
            "scope": "preselected-high-energy-subband common-pole core only",
            "gabor": {
                "window": "scipy blackmanharris",
                "window_samples": WINDOW_SAMPLES,
                "hop_samples": HOP_SAMPLES,
                "fft_samples": WINDOW_SAMPLES,
                "band_bins": BAND_BINS,
                "band_centers_hz": BAND_CENTERS_HZ,
                "frame_count": GABOR_FRAME_COUNT,
                "boundary": None,
                "padded": False,
            },
            "common_pole": {
                "pencil_lags": PENCIL_LAGS,
                "maximum_order": MAXIMUM_ORDER,
                "row_layout": "lag-major then output-major block Hankel",
                "order_score": (
                    "inverse normalized least-squares rotational-invariance residual"
                ),
                "pole_estimator": "least-squares ESPRIT eigenvalues",
                "frequency_unwrap": "nearest declared Gabor band center",
                "post_filter": "finite stable pole inside plus-or-minus one hop alias",
            },
            "scale_factors": SCALE_FACTORS,
        },
        "gates": GATES,
        "research_sources": RESEARCH_SOURCES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "synthetic_only": True,
            "real_payload_allowed": False,
            "network_allowed": False,
            "iron_parameter_or_early_tail_reuse_allowed": False,
            "threshold_or_fixture_tuning_after_preflight_allowed": False,
            "physics_solver_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish support or rejection twice; on failure do not weaken gates; on "
            "support freeze a separate broad-band discovery control before real reuse"
        ),
    }


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
        raise ControlError(f"output must remain outside repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise ControlError(f"output must be absent or empty: {resolved}")
    return resolved


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ControlError("subband common-pole manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ControlError(f"parse subband common-pole manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ControlError("subband common-pole manifest changed")
    return data, manifest


def validate_parent(root: Path, base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    report_ref = manifest["parent"]["diagnostic_report"]
    report_path = (base / report_ref["path"]).resolve(strict=True)
    if not report_path.is_file() or not report_path.is_relative_to(base.parent):
        raise ControlError("parent diagnostic report escapes physical-sound store")
    report_bytes = report_path.read_bytes()
    report = json.loads(report_bytes)
    doc_ref = manifest["parent"]["result_document"]
    doc_path = (root / doc_ref["path"]).resolve(strict=True)
    if (
        sha256_bytes(report_bytes) != report_ref["sha256"]
        or report.get("decision") != "FixedTailTimingMismatchSupported"
        or report.get("classification", {}).get("fixed_tail_timing_mismatch_supported")
        is not True
        or report.get("classification", {}).get(
            "source_selector_composition_mismatch_supported"
        )
        is not False
        or report.get("network_requests") != 0
        or report.get("physics_solver_runs") != 0
        or report.get("planter_payload_bytes_read") != 0
        or not doc_path.is_file()
        or not doc_path.is_relative_to(root)
        or sha256_file(doc_path) != doc_ref["sha256"]
    ):
        raise ControlError("fixed-tail diagnostic parent lineage changed")
    return {
        "diagnostic_report_sha256": sha256_bytes(report_bytes),
        "result_document_sha256": sha256_file(doc_path),
        "decision": report["decision"],
    }


def generate_fixture() -> np.ndarray:
    channels = np.zeros((CHANNEL_COUNT, SAMPLE_COUNT), dtype=np.float64)
    time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE, dtype=np.float64) / SAMPLE_RATE_HZ
    truth = [
        (frequency, decay)
        for frequencies, decays in zip(
            FREQUENCY_GROUPS_HZ, DECAY_GROUPS_PER_SECOND, strict=True
        )
        for frequency, decay in zip(frequencies, decays, strict=True)
    ]
    for channel in range(CHANNEL_COUNT):
        for mode, (frequency, decay) in enumerate(truth):
            gain = 0.45 + 0.035 * ((7 * channel + 3 * mode) % 11)
            if channel == NODE_CHANNEL and mode == NODE_MODE_FLAT_INDEX:
                gain = 0.0
            phase = 0.13 * channel + 0.19 * mode + 0.021 * channel * mode
            channels[channel, ONSET_SAMPLE:] += (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
    generator = np.random.Generator(np.random.PCG64(NOISE_SEED))
    burst = generator.standard_normal(len(time)) * np.exp(
        -TRANSIENT_DECAY_PER_SECOND * time
    )
    for channel in range(CHANNEL_COUNT):
        channels[channel, ONSET_SAMPLE:] += (
            TRANSIENT_GAIN * (0.7 + 0.02 * channel) * burst
        )
    channels += NOISE_STANDARD_DEVIATION * generator.standard_normal(channels.shape)
    return channels


def gabor_coefficients(channels: np.ndarray) -> np.ndarray:
    outputs = []
    for channel in channels:
        _, _, coefficients = signal.stft(
            channel,
            fs=SAMPLE_RATE_HZ,
            window="blackmanharris",
            nperseg=WINDOW_SAMPLES,
            noverlap=WINDOW_SAMPLES - HOP_SAMPLES,
            nfft=WINDOW_SAMPLES,
            boundary=None,
            padded=False,
            return_onesided=True,
        )
        outputs.append(coefficients[BAND_BINS, :GABOR_FRAME_COUNT])
    return np.stack(outputs, axis=2)


def block_hankel(values: np.ndarray) -> np.ndarray:
    snapshots = len(values) - PENCIL_LAGS + 1
    if snapshots <= MAXIMUM_ORDER:
        raise ControlError("common-pole pencil has too few snapshots")
    return np.vstack(
        [values[lag : lag + snapshots].T for lag in range(PENCIL_LAGS)]
    )


def estimate_common_poles(values: np.ndarray, center_hz: float) -> dict[str, Any]:
    output_count = values.shape[1]
    hankel = block_hankel(values)
    left, singular_values, _ = np.linalg.svd(hankel, full_matrices=False)
    order_scores = []
    for order in range(1, MAXIMUM_ORDER + 1):
        subspace = left[:, :order]
        upper = subspace[:-output_count]
        lower = subspace[output_count:]
        rotation = np.linalg.lstsq(upper, lower, rcond=None)[0]
        residual = lower - upper @ rotation
        denominator = max(float(np.linalg.norm(lower, "fro") ** 2), 1.0e-30)
        relative_error = float(np.linalg.norm(residual, "fro") ** 2) / denominator
        order_scores.append(
            {
                "order": order,
                "relative_rotational_error": relative_error,
                "inverse_error_score": 1.0 / max(relative_error, 1.0e-30),
                "singular_value": float(singular_values[order - 1]),
                "next_singular_value": (
                    float(singular_values[order])
                    if order < len(singular_values)
                    else 0.0
                ),
            }
        )
    selected = max(order_scores, key=lambda item: item["inverse_error_score"])
    order = selected["order"]
    runner_up = max(
        item["inverse_error_score"]
        for item in order_scores
        if item["order"] != order
    )
    subspace = left[:, :order]
    rotation = np.linalg.lstsq(
        subspace[:-output_count], subspace[output_count:], rcond=None
    )[0]
    estimates = []
    alias_rate_hz = SAMPLE_RATE_HZ / HOP_SAMPLES
    for pole in np.linalg.eigvals(rotation):
        alias_frequency = float(np.angle(pole)) * alias_rate_hz / (2.0 * math.pi)
        frequency = alias_frequency + round(
            (center_hz - alias_frequency) / alias_rate_hz
        ) * alias_rate_hz
        decay = -math.log(abs(pole)) * alias_rate_hz
        if (
            math.isfinite(frequency)
            and math.isfinite(decay)
            and 0.0 < abs(pole) < 1.0
            and abs(frequency - center_hz) <= alias_rate_hz / 2.0
        ):
            estimates.append(
                {
                    "frequency_hz": frequency,
                    "decay_per_second": decay,
                    "pole_magnitude": float(abs(pole)),
                    "pole_phase_radians": float(np.angle(pole)),
                }
            )
    estimates.sort(key=lambda item: item["frequency_hz"])
    return {
        "output_count": output_count,
        "selected_order": order,
        "order_score_margin": selected["inverse_error_score"] / runner_up,
        "order_scores": order_scores,
        "accepted_pole_count": len(estimates),
        "rejected_pole_count": order - len(estimates),
        "estimates": estimates,
    }


def compare_truth(
    analysis: dict[str, Any], frequencies: list[float], decays: list[float]
) -> dict[str, Any]:
    estimates = analysis["estimates"]
    comparisons = []
    for estimate, expected_frequency, expected_decay in zip(
        estimates, frequencies, decays, strict=False
    ):
        comparisons.append(
            {
                "expected_frequency_hz": expected_frequency,
                "observed_frequency_hz": estimate["frequency_hz"],
                "frequency_error_hz": abs(
                    estimate["frequency_hz"] - expected_frequency
                ),
                "expected_decay_per_second": expected_decay,
                "observed_decay_per_second": estimate["decay_per_second"],
                "decay_error_per_second": abs(
                    estimate["decay_per_second"] - expected_decay
                ),
            }
        )
    return {
        "truth_mode_count": len(frequencies),
        "observed_mode_count": len(estimates),
        "missed_truth_count": max(0, len(frequencies) - len(estimates)),
        "extra_estimate_count": max(0, len(estimates) - len(frequencies)),
        "comparisons": comparisons,
    }


def analyze_scale(coefficients: np.ndarray, scale: float) -> dict[str, Any]:
    bands = []
    for index, center in enumerate(BAND_CENTERS_HZ):
        estimate = estimate_common_poles(coefficients[index] * scale, center)
        bands.append(
            {
                "band_index": index,
                "gabor_bin": BAND_BINS[index],
                "center_hz": center,
                "analysis": estimate,
                "truth": compare_truth(
                    estimate,
                    FREQUENCY_GROUPS_HZ[index],
                    DECAY_GROUPS_PER_SECOND[index],
                ),
            }
        )
    return {"scale": scale, "bands": bands}


def scale_invariance(variants: list[dict[str, Any]]) -> dict[str, Any]:
    baseline = next(item for item in variants if item["scale"] == 1.0)
    maximum_frequency = 0.0
    maximum_decay = 0.0
    order_invariant = True
    for variant in variants:
        for reference_band, band in zip(
            baseline["bands"], variant["bands"], strict=True
        ):
            reference = reference_band["analysis"]
            observed = band["analysis"]
            order_invariant &= (
                reference["selected_order"] == observed["selected_order"]
            )
            for expected, actual in zip(
                reference["estimates"], observed["estimates"], strict=True
            ):
                maximum_frequency = max(
                    maximum_frequency,
                    abs(expected["frequency_hz"] - actual["frequency_hz"]),
                )
                maximum_decay = max(
                    maximum_decay,
                    abs(expected["decay_per_second"] - actual["decay_per_second"]),
                )
    return {
        "order_invariant": order_invariant,
        "maximum_frequency_delta_hz": maximum_frequency,
        "maximum_decay_delta_per_second": maximum_decay,
    }


def check(name: str, observed: Any, relation: str, threshold: Any) -> dict[str, Any]:
    if relation == "<=":
        passed = observed <= threshold
    elif relation == ">=":
        passed = observed >= threshold
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
        "synthetic_only": True,
        "real_payload_bytes_read": 0,
        "network_requests": 0,
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
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "SubbandCommonPoleControlFrozen",
        "claim": (
            "SYNTHETIC_PRESELECTED_SUBBAND_CORE_PREFLIGHT_ONLY / "
            "NO_REAL_PAYLOAD_TAIL_REUSE_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": validate_parent(root, base, manifest),
        "fixture": manifest["fixture"],
        "candidate": manifest["candidate"],
        "gates": manifest["gates"],
        "research_sources": manifest["research_sources"],
        "runtime": manifest["runtime"],
        "next_action": "commit this preflight, then execute the synthetic control twice",
    }


def run_control(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parent = validate_parent(root, base, manifest)
    channels = generate_fixture()
    coefficients = gabor_coefficients(channels)
    variants = [analyze_scale(coefficients, scale) for scale in SCALE_FACTORS]
    baseline = next(item for item in variants if item["scale"] == 1.0)
    node_analysis = estimate_common_poles(
        coefficients[0, :, NODE_CHANNEL : NODE_CHANNEL + 1],
        BAND_CENTERS_HZ[0],
    )
    node_truth = compare_truth(
        node_analysis, FREQUENCY_GROUPS_HZ[0], DECAY_GROUPS_PER_SECOND[0]
    )
    invariance = scale_invariance(variants)
    frequency_errors = [
        item["frequency_error_hz"]
        for band in baseline["bands"]
        for item in band["truth"]["comparisons"]
    ]
    decay_errors = [
        item["decay_error_per_second"]
        for band in baseline["bands"]
        for item in band["truth"]["comparisons"]
    ]
    order_margins = [
        band["analysis"]["order_score_margin"] for band in baseline["bands"]
    ]
    observed_orders = [
        band["analysis"]["selected_order"] for band in baseline["bands"]
    ]
    observed_modes = sum(
        band["truth"]["observed_mode_count"] for band in baseline["bands"]
    )
    fft_bin_hz = SAMPLE_RATE_HZ / WINDOW_SAMPLES
    close_pairs = sum(
        len(frequencies) == 2
        and abs(frequencies[1] - frequencies[0]) < fft_bin_hz
        for frequencies in FREQUENCY_GROUPS_HZ
    )
    checks = [
        check(
            "band_orders",
            observed_orders,
            "==",
            GATES["expected_band_orders"],
        ),
        check(
            "truth_mode_count",
            observed_modes,
            "==",
            GATES["expected_truth_mode_count"],
        ),
        check(
            "maximum_frequency_error_hz",
            max(frequency_errors),
            "<=",
            GATES["maximum_frequency_error_hz"],
        ),
        check(
            "maximum_decay_error_per_second",
            max(decay_errors),
            "<=",
            GATES["maximum_decay_error_per_second"],
        ),
        check(
            "minimum_order_score_margin",
            min(order_margins),
            ">=",
            GATES["minimum_order_score_margin"],
        ),
        check(
            "node_channel_order",
            node_analysis["selected_order"],
            "==",
            GATES["expected_node_channel_order"],
        ),
        check(
            "node_channel_missed_truth_count",
            node_truth["missed_truth_count"],
            "==",
            GATES["expected_node_channel_missed_truth_count"],
        ),
        check(
            "scale_frequency_delta_hz",
            invariance["maximum_frequency_delta_hz"],
            "<=",
            GATES["maximum_scale_frequency_delta_hz"],
        ),
        check(
            "scale_decay_delta_per_second",
            invariance["maximum_decay_delta_per_second"],
            "<=",
            GATES["maximum_scale_decay_delta_per_second"],
        ),
        check(
            "scale_invariant_order",
            invariance["order_invariant"],
            "==",
            GATES["scale_invariant_order"],
        ),
        check(
            "close_pairs_below_one_fft_bin",
            close_pairs,
            "==",
            GATES["close_pairs_below_one_fft_bin"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["run"],
        "status": "Validated",
        "decision": (
            "SubbandCommonPoleSyntheticControlSupported"
            if passed
            else "SubbandCommonPoleSyntheticControlRejected"
        ),
        "claim": (
            "SYNTHETIC_PRESELECTED_SUBBAND_COMMON_POLE_CORE_ONLY / "
            "NO_BROADBAND_DISCOVERY_REAL_TRANSFER_PHYSICS_QUALITY_ADMISSION_OR_"
            "RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": parent,
        "fixture_sha256": sha256_bytes(channels.astype("<f8", copy=False).tobytes()),
        "gabor_coefficients_sha256": sha256_bytes(
            coefficients.astype("<c16", copy=False).tobytes()
        ),
        "fft_bin_hz": fft_bin_hz,
        "scale_variants": variants,
        "scale_invariance": invariance,
        "node_channel_control": {
            "channel": NODE_CHANNEL,
            "analysis": node_analysis,
            "truth": node_truth,
        },
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze a separate synthetic broad-band subband-discovery and duplicate-"
            "pruning control before any real reuse"
            if passed
            else "reject this estimator revision; do not weaken gates or reuse real data"
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
    print(f"Subband common-pole {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("real payload bytes read: 0")
    print("network requests: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
