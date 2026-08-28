#!/usr/bin/env python3
"""Prove broad-band Gabor discovery, pole merging, amplitudes, and resynthesis."""

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

import physical_sound_subband_common_pole_control as core


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-broadband-pole.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-physical-sound-broadband-pole-preflight.report.v1"
    ),
    "run": "nextengine.experimental-physical-sound-broadband-pole.report.v1",
}
STUDY_ID = "physical-sound-broadband-common-pole-control"
REVISION = "broadband-gabor-common-pole-resynthesis-v1"

PARENT_REPORT_SHA256 = (
    "3fcb5fc864283576add71192afcf4b73b8ddd9ceccde5b3dead9c8acc442284f"
)
PARENT_RESULT_DOC_SHA256 = (
    "06c3f4a6d9f22972a21e3849f5334ddb94319222deddaa47e121231d832c78f7"
)
CORE_RUNNER_SHA256 = (
    "4acf8016791a0072ea68ce8902d3e4e9d0230ec4438f26a55f0254b6a02bf141"
)

SAMPLE_RATE_HZ = core.SAMPLE_RATE_HZ
SAMPLE_COUNT = core.SAMPLE_COUNT
CHANNEL_COUNT = core.CHANNEL_COUNT
ONSET_SAMPLE = core.ONSET_SAMPLE
WINDOW_SAMPLES = core.WINDOW_SAMPLES
HOP_SAMPLES = core.HOP_SAMPLES
GABOR_FRAME_COUNT = core.GABOR_FRAME_COUNT

TRUTH_FREQUENCIES_HZ = [
    1_373.0,
    1_401.0,
    3_479.0,
    3_512.0,
    6_237.0,
    9_488.0,
    11_231.0,
]
TRUTH_DECAYS_PER_SECOND = [1.8, 4.5, 2.7, 8.5, 5.5, 14.0, 9.0]
WEAK_FREQUENCY_HZ = 7_313.0
WEAK_DECAY_PER_SECOND = 3.0
WEAK_AMPLITUDE = 0.04
TRANSIENT_DECAY_PER_SECOND = 70.0
TRANSIENT_GAIN = 0.6
NOISE_STANDARD_DEVIATION = 1.0e-8
CALIBRATION_SEED = 20_260_830
HOLDOUT_SEED = 20_260_831

MINIMUM_FREQUENCY_HZ = 500.0
MAXIMUM_FREQUENCY_HZ = 12_000.0
REGION_ENERGY_FLOOR_DB = -30.0
NEIGHBORHOOD_RADIUS_BINS = 1
MINIMUM_ORDER_SCORE_MARGIN = 50.0
DUPLICATE_FREQUENCY_HZ = 1.0
MODE_ENERGY_FLOOR_DB = -25.0
SCALE_FACTORS = core.SCALE_FACTORS

GATES = {
    "selected_region_count": 6,
    "selected_analysis_bin_count": 18,
    "minimum_duplicate_estimates_removed": 10,
    "preprune_cluster_count": 8,
    "retained_mode_count": 7,
    "pruned_mode_count": 1,
    "truth_match_count": 7,
    "false_positive_count": 0,
    "weak_nuisance_pruned": True,
    "maximum_frequency_error_hz": 0.25,
    "maximum_decay_error_per_second": 0.75,
    "maximum_modal_truth_nrmse": 0.08,
    "maximum_observed_full_nrmse": 0.20,
    "maximum_observed_post_transient_nrmse": 0.10,
    "scale_invariant_region_bins": True,
}


class ControlError(RuntimeError):
    """The frozen broad-band control or parent lineage failed."""


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
            "control_report": {
                "path": "../ps2-subband-common-pole-control-v1/run-a/report.json",
                "sha256": PARENT_REPORT_SHA256,
            },
            "result_document": {
                "path": (
                    "docs/development/physical-sound-subband-common-pole-control-"
                    "result-ps2-2026-08-28.md"
                ),
                "sha256": PARENT_RESULT_DOC_SHA256,
            },
            "core_runner": {
                "path": "lab/scripts/physical_sound_subband_common_pole_control.py",
                "sha256": CORE_RUNNER_SHA256,
            },
        },
        "fixture": {
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "sample_count": SAMPLE_COUNT,
            "channel_count": CHANNEL_COUNT,
            "onset_sample": ONSET_SAMPLE,
            "truth_frequencies_hz": TRUTH_FREQUENCIES_HZ,
            "truth_decays_per_second": TRUTH_DECAYS_PER_SECOND,
            "participation_rule": (
                "(0.40 + 0.03*((5*channel + 7*mode) mod 13))/(1 + 0.05*mode); "
                "deterministic phase"
            ),
            "weak_nuisance": {
                "frequency_hz": WEAK_FREQUENCY_HZ,
                "decay_per_second": WEAK_DECAY_PER_SECOND,
                "amplitude": WEAK_AMPLITUDE,
                "participation_rule": "amplitude*(0.8 + 0.01*channel)",
            },
            "transient": {
                "generator": "shared numpy.PCG64 broadband burst",
                "decay_per_second": TRANSIENT_DECAY_PER_SECOND,
                "gain": TRANSIENT_GAIN,
            },
            "noise": {
                "generator": "numpy.PCG64 independent outputs",
                "standard_deviation": NOISE_STANDARD_DEVIATION,
            },
            "calibration_seed_not_evidence": CALIBRATION_SEED,
            "holdout_seed": HOLDOUT_SEED,
            "sample_format": "f64le-channel-major",
        },
        "candidate": {
            "id": REVISION,
            "gabor": {
                "window": "scipy blackmanharris",
                "window_samples": WINDOW_SAMPLES,
                "hop_samples": HOP_SAMPLES,
                "fft_samples": WINDOW_SAMPLES,
                "frame_count": GABOR_FRAME_COUNT,
                "boundary": None,
                "padded": False,
            },
            "discovery": {
                "minimum_frequency_hz": MINIMUM_FREQUENCY_HZ,
                "maximum_frequency_hz": MAXIMUM_FREQUENCY_HZ,
                "spatial_energy": "sum squared complex magnitude over frames and outputs",
                "relative_local_peak_floor_db": REGION_ENERGY_FLOOR_DB,
                "neighborhood_radius_bins": NEIGHBORHOOD_RADIUS_BINS,
            },
            "common_pole_core": {
                "source_sha256": CORE_RUNNER_SHA256,
                "minimum_order_score_margin": MINIMUM_ORDER_SCORE_MARGIN,
            },
            "duplicates": {
                "single_link_frequency_hz": DUPLICATE_FREQUENCY_HZ,
                "representative": (
                    "maximum order-score margin, then band energy, then lower bin"
                ),
            },
            "amplitudes": {
                "fit": (
                    "joint all-output least-squares real cosine/sine coefficients "
                    "over all post-onset samples"
                ),
                "equivalent_complex_amplitude": "cosine coefficient minus i*sine coefficient",
                "mode_energy_floor_db": MODE_ENERGY_FLOOR_DB,
                "pruning_order": "after common-pole estimation and amplitude fit",
            },
            "reconstruction": {
                "modal_truth": "strong seven-mode fixture only",
                "observed_full": "strong modes plus weak nuisance, transient and noise",
                "post_transient_start": (
                    "ceil(log(1000)/declared transient decay * sample rate)"
                ),
            },
            "scale_factors": SCALE_FACTORS,
        },
        "gates": GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "synthetic_holdout_only": True,
            "calibration_seed_execution_allowed": False,
            "real_payload_allowed": False,
            "network_allowed": False,
            "iron_parameter_or_early_tail_reuse_allowed": False,
            "threshold_or_fixture_tuning_after_preflight_allowed": False,
            "physics_solver_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish support or rejection twice on the frozen holdout seed; on failure "
            "do not tune; on support only preregister a read-only existing-Iron "
            "counterfactual before real reuse"
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
        raise ControlError("broad-band manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ControlError(f"parse broad-band manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise ControlError("broad-band manifest changed")
    return data, manifest


def validate_parent(root: Path, base: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    parent = manifest["parent"]
    report_path = (base / parent["control_report"]["path"]).resolve(strict=True)
    if not report_path.is_file() or not report_path.is_relative_to(base.parent):
        raise ControlError("parent control report escapes physical-sound store")
    report_bytes = report_path.read_bytes()
    report = json.loads(report_bytes)
    document_path = (root / parent["result_document"]["path"]).resolve(strict=True)
    core_path = (root / parent["core_runner"]["path"]).resolve(strict=True)
    if (
        sha256_bytes(report_bytes) != parent["control_report"]["sha256"]
        or report.get("decision") != "SubbandCommonPoleSyntheticControlSupported"
        or report.get("gate", {}).get("passed") is not True
        or report.get("real_payload_bytes_read") != 0
        or report.get("network_requests") != 0
        or report.get("physics_solver_runs") != 0
        or report.get("planter_payload_bytes_read") != 0
        or not document_path.is_file()
        or not document_path.is_relative_to(root)
        or sha256_file(document_path) != parent["result_document"]["sha256"]
        or not core_path.is_file()
        or not core_path.is_relative_to(root)
        or sha256_file(core_path) != parent["core_runner"]["sha256"]
    ):
        raise ControlError("subband common-pole parent lineage changed")
    return {
        "control_report_sha256": sha256_bytes(report_bytes),
        "result_document_sha256": sha256_file(document_path),
        "core_runner_sha256": sha256_file(core_path),
        "decision": report["decision"],
    }


def generate_fixture() -> tuple[np.ndarray, np.ndarray]:
    observed = np.zeros((CHANNEL_COUNT, SAMPLE_COUNT), dtype=np.float64)
    modal_truth = np.zeros_like(observed)
    time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE, dtype=np.float64) / SAMPLE_RATE_HZ
    for channel in range(CHANNEL_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(TRUTH_FREQUENCIES_HZ, TRUTH_DECAYS_PER_SECOND, strict=True)
        ):
            gain = (0.40 + 0.03 * ((5 * channel + 7 * mode) % 13)) / (
                1.0 + 0.05 * mode
            )
            phase = 0.17 * channel + 0.23 * mode + 0.017 * channel * mode
            values = (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
            observed[channel, ONSET_SAMPLE:] += values
            modal_truth[channel, ONSET_SAMPLE:] += values
        observed[channel, ONSET_SAMPLE:] += (
            WEAK_AMPLITUDE
            * (0.8 + 0.01 * channel)
            * np.exp(-WEAK_DECAY_PER_SECOND * time)
            * np.cos(2.0 * math.pi * WEAK_FREQUENCY_HZ * time + 0.11 * channel)
        )
    generator = np.random.Generator(np.random.PCG64(HOLDOUT_SEED))
    burst = generator.standard_normal(len(time)) * np.exp(
        -TRANSIENT_DECAY_PER_SECOND * time
    )
    for channel in range(CHANNEL_COUNT):
        observed[channel, ONSET_SAMPLE:] += (
            TRANSIENT_GAIN * (0.75 + 0.01 * channel) * burst
        )
    observed += NOISE_STANDARD_DEVIATION * generator.standard_normal(observed.shape)
    return observed, modal_truth


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
        outputs.append(coefficients[:, :GABOR_FRAME_COUNT])
    return np.stack(outputs, axis=2)


def discover_regions(coefficients: np.ndarray) -> dict[str, Any]:
    spatial_energy = np.sum(np.abs(coefficients) ** 2, axis=(1, 2))
    bin_hz = SAMPLE_RATE_HZ / WINDOW_SAMPLES
    first = math.ceil(MINIMUM_FREQUENCY_HZ / bin_hz)
    last = min(
        math.floor(MAXIMUM_FREQUENCY_HZ / bin_hz), len(spatial_energy) - 2
    )
    maximum = max(float(np.max(spatial_energy[first : last + 1])), 1.0e-300)
    regions = []
    for index in range(first, last + 1):
        relative_db = 10.0 * math.log10(
            max(float(spatial_energy[index]), 1.0e-300) / maximum
        )
        if (
            spatial_energy[index] > spatial_energy[index - 1]
            and spatial_energy[index] >= spatial_energy[index + 1]
            and relative_db >= REGION_ENERGY_FLOOR_DB
        ):
            regions.append(
                {
                    "bin": index,
                    "frequency_hz": index * bin_hz,
                    "relative_energy_db": relative_db,
                }
            )
    analysis_bins = sorted(
        {
            index
            for region in regions
            for index in range(
                region["bin"] - NEIGHBORHOOD_RADIUS_BINS,
                region["bin"] + NEIGHBORHOOD_RADIUS_BINS + 1,
            )
            if first <= index <= last
        }
    )
    return {
        "first_bin": first,
        "last_bin": last,
        "bin_hz": bin_hz,
        "regions": regions,
        "analysis_bins": analysis_bins,
        "spatial_energy": spatial_energy,
        "maximum_energy": maximum,
    }


def discovery_scale_invariance(coefficients: np.ndarray) -> dict[str, Any]:
    variants = []
    signatures = []
    for scale in SCALE_FACTORS:
        discovery = discover_regions(coefficients * scale)
        signature = [item["bin"] for item in discovery["regions"]]
        signatures.append(signature)
        variants.append(
            {
                "scale": scale,
                "region_bins": signature,
                "analysis_bins": discovery["analysis_bins"],
            }
        )
    return {
        "passed": all(signature == signatures[0] for signature in signatures[1:]),
        "variants": variants,
    }


def extract_raw_estimates(
    coefficients: np.ndarray, discovery: dict[str, Any]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    raw = []
    analyses = []
    bin_hz = discovery["bin_hz"]
    maximum = discovery["maximum_energy"]
    energies = discovery["spatial_energy"]
    for index in discovery["analysis_bins"]:
        analysis = core.estimate_common_poles(coefficients[index], index * bin_hz)
        relative_energy_db = 10.0 * math.log10(
            max(float(energies[index]), 1.0e-300) / maximum
        )
        accepted = analysis["order_score_margin"] >= MINIMUM_ORDER_SCORE_MARGIN
        analyses.append(
            {
                "bin": index,
                "center_hz": index * bin_hz,
                "relative_energy_db": relative_energy_db,
                "accepted_by_order_score": accepted,
                "analysis": analysis,
            }
        )
        if accepted:
            for estimate in analysis["estimates"]:
                raw.append(
                    {
                        **estimate,
                        "source_bin": index,
                        "source_relative_energy_db": relative_energy_db,
                        "order_score_margin": analysis["order_score_margin"],
                    }
                )
    raw.sort(key=lambda item: item["frequency_hz"])
    return raw, analyses


def cluster_duplicates(raw: list[dict[str, Any]]) -> list[dict[str, Any]]:
    groups: list[list[dict[str, Any]]] = []
    for estimate in raw:
        if (
            not groups
            or estimate["frequency_hz"] - groups[-1][-1]["frequency_hz"]
            > DUPLICATE_FREQUENCY_HZ
        ):
            groups.append([estimate])
        else:
            groups[-1].append(estimate)
    clusters = []
    for index, members in enumerate(groups):
        representative = max(
            members,
            key=lambda item: (
                item["order_score_margin"],
                item["source_relative_energy_db"],
                -item["source_bin"],
            ),
        )
        clusters.append(
            {
                "cluster_index": index,
                "member_count": len(members),
                "frequency_min_hz": members[0]["frequency_hz"],
                "frequency_max_hz": members[-1]["frequency_hz"],
                "representative": representative,
                "members": members,
            }
        )
    return clusters


def fit_amplitudes(
    observed: np.ndarray, clusters: list[dict[str, Any]]
) -> tuple[list[dict[str, Any]], np.ndarray]:
    time = np.arange(SAMPLE_COUNT - ONSET_SAMPLE, dtype=np.float64) / SAMPLE_RATE_HZ
    columns = []
    for cluster in clusters:
        estimate = cluster["representative"]
        envelope = np.exp(-estimate["decay_per_second"] * time)
        phase = 2.0 * math.pi * estimate["frequency_hz"] * time
        columns.extend([envelope * np.cos(phase), envelope * np.sin(phase)])
    design = np.stack(columns, axis=1)
    coefficients = np.linalg.lstsq(
        design, observed[:, ONSET_SAMPLE:].T, rcond=None
    )[0]
    components = []
    energies = []
    for index in range(len(clusters)):
        component = (
            design[:, 2 * index, None] * coefficients[2 * index][None, :]
            + design[:, 2 * index + 1, None]
            * coefficients[2 * index + 1][None, :]
        )
        components.append(component)
        energies.append(float(np.sum(component * component)))
    maximum_energy = max(energies)
    reports = []
    retained_components = []
    for cluster, component, energy, index in zip(
        clusters, components, energies, range(len(clusters)), strict=True
    ):
        relative_db = 10.0 * math.log10(max(energy, 1.0e-300) / maximum_energy)
        retained = relative_db >= MODE_ENERGY_FLOOR_DB
        complex_amplitudes = [
            complex(
                float(coefficients[2 * index, channel]),
                -float(coefficients[2 * index + 1, channel]),
            )
            for channel in range(CHANNEL_COUNT)
        ]
        reports.append(
            {
                **cluster,
                "mode_energy": energy,
                "relative_mode_energy_db": relative_db,
                "retained": retained,
                "complex_amplitudes": [
                    {
                        "real": value.real,
                        "imaginary": value.imag,
                        "magnitude": abs(value),
                        "phase_radians": math.atan2(value.imag, value.real),
                    }
                    for value in complex_amplitudes
                ],
            }
        )
        if retained:
            retained_components.append(component)
    reconstruction = np.sum(retained_components, axis=0).T
    return reports, reconstruction


def normalized_rms_error(expected: np.ndarray, observed: np.ndarray) -> float:
    denominator = math.sqrt(float(np.mean(expected * expected)))
    if denominator <= 1.0e-300:
        raise ControlError("reconstruction reference has zero RMS")
    return math.sqrt(float(np.mean((expected - observed) ** 2))) / denominator


def match_truth(retained: list[dict[str, Any]]) -> dict[str, Any]:
    candidates = [item["representative"] for item in retained]
    pairs = []
    for candidate_index, candidate in enumerate(candidates):
        for truth_index, frequency in enumerate(TRUTH_FREQUENCIES_HZ):
            error = abs(candidate["frequency_hz"] - frequency)
            if error <= 1.0:
                pairs.append((error, candidate_index, truth_index))
    pairs.sort()
    assignments: list[int | None] = [None] * len(candidates)
    used_truth = [False] * len(TRUTH_FREQUENCIES_HZ)
    for _, candidate_index, truth_index in pairs:
        if assignments[candidate_index] is None and not used_truth[truth_index]:
            assignments[candidate_index] = truth_index
            used_truth[truth_index] = True
    comparisons = []
    for candidate, assignment in zip(candidates, assignments, strict=True):
        if assignment is None:
            continue
        comparisons.append(
            {
                "expected_frequency_hz": TRUTH_FREQUENCIES_HZ[assignment],
                "observed_frequency_hz": candidate["frequency_hz"],
                "frequency_error_hz": abs(
                    candidate["frequency_hz"] - TRUTH_FREQUENCIES_HZ[assignment]
                ),
                "expected_decay_per_second": TRUTH_DECAYS_PER_SECOND[assignment],
                "observed_decay_per_second": candidate["decay_per_second"],
                "decay_error_per_second": abs(
                    candidate["decay_per_second"]
                    - TRUTH_DECAYS_PER_SECOND[assignment]
                ),
            }
        )
    return {
        "truth_count": len(TRUTH_FREQUENCIES_HZ),
        "candidate_count": len(candidates),
        "truth_match_count": len(comparisons),
        "false_positive_count": len(candidates) - len(comparisons),
        "assignments": assignments,
        "comparisons": comparisons,
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
        "synthetic_holdout_seed": HOLDOUT_SEED,
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
        "decision": "BroadbandCommonPoleControlFrozen",
        "claim": (
            "SYNTHETIC_HOLDOUT_BROADBAND_PREFLIGHT_ONLY / "
            "NO_CALIBRATION_OR_REAL_PAYLOAD_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": validate_parent(root, base, manifest),
        "fixture": manifest["fixture"],
        "candidate": manifest["candidate"],
        "gates": manifest["gates"],
        "runtime": manifest["runtime"],
        "next_action": "commit this preflight, then execute the holdout control twice",
    }


def run_control(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parent = validate_parent(root, base, manifest)
    observed, modal_truth = generate_fixture()
    coefficients = gabor_coefficients(observed)
    discovery = discover_regions(coefficients)
    scale_invariance = discovery_scale_invariance(coefficients)
    raw, bin_analyses = extract_raw_estimates(coefficients, discovery)
    clusters = cluster_duplicates(raw)
    fitted, reconstruction = fit_amplitudes(observed, clusters)
    retained = [item for item in fitted if item["retained"]]
    pruned = [item for item in fitted if not item["retained"]]
    truth = match_truth(retained)
    weak_pruned = any(
        abs(item["representative"]["frequency_hz"] - WEAK_FREQUENCY_HZ) <= 1.0
        for item in pruned
    )
    frequency_errors = [
        item["frequency_error_hz"] for item in truth["comparisons"]
    ]
    decay_errors = [
        item["decay_error_per_second"] for item in truth["comparisons"]
    ]
    modal_reference = modal_truth[:, ONSET_SAMPLE:]
    observed_reference = observed[:, ONSET_SAMPLE:]
    post_transient_sample = math.ceil(
        math.log(1_000.0) / TRANSIENT_DECAY_PER_SECOND * SAMPLE_RATE_HZ
    )
    reconstruction_metrics = {
        "modal_truth_nrmse": normalized_rms_error(modal_reference, reconstruction),
        "observed_full_nrmse": normalized_rms_error(
            observed_reference, reconstruction
        ),
        "post_transient_start_sample": post_transient_sample,
        "post_transient_start_ms": post_transient_sample * 1_000.0 / SAMPLE_RATE_HZ,
        "observed_post_transient_nrmse": normalized_rms_error(
            observed_reference[:, post_transient_sample:],
            reconstruction[:, post_transient_sample:],
        ),
    }
    duplicate_estimates_removed = len(raw) - len(clusters)
    checks = [
        check(
            "selected_region_count",
            len(discovery["regions"]),
            "==",
            GATES["selected_region_count"],
        ),
        check(
            "selected_analysis_bin_count",
            len(discovery["analysis_bins"]),
            "==",
            GATES["selected_analysis_bin_count"],
        ),
        check(
            "duplicate_estimates_removed",
            duplicate_estimates_removed,
            ">=",
            GATES["minimum_duplicate_estimates_removed"],
        ),
        check(
            "preprune_cluster_count",
            len(clusters),
            "==",
            GATES["preprune_cluster_count"],
        ),
        check(
            "retained_mode_count",
            len(retained),
            "==",
            GATES["retained_mode_count"],
        ),
        check(
            "pruned_mode_count",
            len(pruned),
            "==",
            GATES["pruned_mode_count"],
        ),
        check(
            "truth_match_count",
            truth["truth_match_count"],
            "==",
            GATES["truth_match_count"],
        ),
        check(
            "false_positive_count",
            truth["false_positive_count"],
            "==",
            GATES["false_positive_count"],
        ),
        check(
            "weak_nuisance_pruned",
            weak_pruned,
            "==",
            GATES["weak_nuisance_pruned"],
        ),
        check(
            "maximum_frequency_error_hz",
            max(frequency_errors) if frequency_errors else 1.0e9,
            "<=",
            GATES["maximum_frequency_error_hz"],
        ),
        check(
            "maximum_decay_error_per_second",
            max(decay_errors) if decay_errors else 1.0e9,
            "<=",
            GATES["maximum_decay_error_per_second"],
        ),
        check(
            "modal_truth_nrmse",
            reconstruction_metrics["modal_truth_nrmse"],
            "<=",
            GATES["maximum_modal_truth_nrmse"],
        ),
        check(
            "observed_full_nrmse",
            reconstruction_metrics["observed_full_nrmse"],
            "<=",
            GATES["maximum_observed_full_nrmse"],
        ),
        check(
            "observed_post_transient_nrmse",
            reconstruction_metrics["observed_post_transient_nrmse"],
            "<=",
            GATES["maximum_observed_post_transient_nrmse"],
        ),
        check(
            "scale_invariant_region_bins",
            scale_invariance["passed"],
            "==",
            GATES["scale_invariant_region_bins"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["run"],
        "status": "Validated",
        "decision": (
            "BroadbandCommonPoleSyntheticControlSupported"
            if passed
            else "BroadbandCommonPoleSyntheticControlRejected"
        ),
        "claim": (
            "SYNTHETIC_HOLDOUT_BROADBAND_DISCOVERY_CLUSTER_AMPLITUDE_"
            "RECONSTRUCTION_ONLY / NO_REAL_TRANSFER_PHYSICS_QUALITY_ADMISSION_OR_"
            "RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parent": parent,
        "observed_fixture_sha256": sha256_bytes(
            observed.astype("<f8", copy=False).tobytes()
        ),
        "modal_truth_sha256": sha256_bytes(
            modal_truth.astype("<f8", copy=False).tobytes()
        ),
        "gabor_coefficients_sha256": sha256_bytes(
            coefficients.astype("<c16", copy=False).tobytes()
        ),
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "scale_invariance": scale_invariance,
        "bin_analyses": bin_analyses,
        "raw_estimate_count": len(raw),
        "duplicate_estimates_removed": duplicate_estimates_removed,
        "fitted_clusters": fitted,
        "truth": truth,
        "weak_nuisance_pruned": weak_pruned,
        "reconstruction": reconstruction_metrics,
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze a read-only existing-Iron broad-band counterfactual before real reuse"
            if passed
            else "reject this revision; do not tune the frozen holdout or reuse real data"
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
    print(f"Broad-band common-pole {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("real payload bytes read: 0")
    print("network requests: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
