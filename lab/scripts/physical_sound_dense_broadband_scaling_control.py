#!/usr/bin/env python3
"""Prove a partial-SVD dense broad-band common-pole analysis envelope."""

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
from scipy.sparse.linalg import svds
import scipy

import physical_sound_broadband_common_pole_control as broadband
import physical_sound_subband_common_pole_control as core


MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-dense-broadband.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-physical-sound-dense-broadband-"
        "preflight.report.v1"
    ),
    "run": "nextengine.experimental-physical-sound-dense-broadband.report.v1",
}
STUDY_ID = "physical-sound-dense-broadband-scaling-control"
REVISION = "dense-broadband-partial-svd-common-pole-v1"

IRON_REJECTION_REPORT_SHA256 = (
    "7635f8aca44286e0a46709aa3268afd049d1038023768a8a265d6a0b24a5075d"
)
IRON_REJECTION_DOC_SHA256 = (
    "f9f6da5f3be8e566f0c27b5e2736200d2e79382bb90639b6a95e35ee32255a81"
)
BROADBAND_REPORT_SHA256 = (
    "dcd832525fbd1955c9122fc52579e4a8ddd10ab76bc5d79393f8f9df9d270094"
)
BROADBAND_RESULT_DOC_SHA256 = (
    "08a83bed456ebe2f963924303f226170a5bb145d964920d34463259f021b736d"
)
BROADBAND_RUNNER_SHA256 = (
    "317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d"
)

CALIBRATION_SEED = 20_260_901
HOLDOUT_SEED = 20_260_902
REGION_BINS = [12 + 7 * index for index in range(32)]
STRONG_OFFSETS_HZ = [-17.0, 17.0]
WEAK_REGION_BINS = [value + 3 for value in REGION_BINS[::2]]
WEAK_AMPLITUDE = 0.01
TRANSIENT_DECAY_PER_SECOND = 70.0
TRANSIENT_GAIN = 0.6
NOISE_STANDARD_DEVIATION = 1.0e-8
PARTIAL_SVD_RANK = core.MAXIMUM_ORDER + 1

GATES = {
    "parent_order_and_count_equivalent": True,
    "maximum_parent_frequency_delta_hz": 1.0e-8,
    "maximum_parent_decay_delta_per_second": 1.0e-8,
    "maximum_parent_order_margin_delta": 1.0e-4,
    "selected_region_count": 32,
    "selected_analysis_bin_count": 96,
    "selected_weak_region_count": 0,
    "minimum_duplicate_estimates_removed": 100,
    "preprune_cluster_count": 64,
    "retained_mode_count": 64,
    "truth_match_count": 64,
    "false_positive_count": 0,
    "maximum_frequency_error_hz": 0.25,
    "maximum_decay_error_per_second": 0.75,
    "maximum_modal_truth_nrmse": 0.10,
    "maximum_observed_full_nrmse": 0.20,
    "maximum_observed_post_transient_nrmse": 0.10,
    "scale_invariant_region_bins": True,
}


class DenseControlError(RuntimeError):
    """The dense synthetic control or exact parent lineage changed."""


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


def truth_parameters() -> tuple[list[float], list[float]]:
    frequencies = []
    decays = []
    bin_hz = core.SAMPLE_RATE_HZ / core.WINDOW_SAMPLES
    for region, bin_index in enumerate(REGION_BINS):
        center = bin_index * bin_hz
        frequencies.extend(center + offset for offset in STRONG_OFFSETS_HZ)
        decays.extend(
            [
                1.5 + 0.35 * (region % 9),
                4.0 + 0.45 * (region % 11),
            ]
        )
    return frequencies, decays


def weak_frequencies() -> list[float]:
    bin_hz = core.SAMPLE_RATE_HZ / core.WINDOW_SAMPLES
    return [value * bin_hz for value in WEAK_REGION_BINS]


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    frequencies, decays = truth_parameters()
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "parents": {
            "iron_rejection_report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-broadband-counterfactual-v1/"
                    "analysis-a/report.json"
                ),
                "sha256": IRON_REJECTION_REPORT_SHA256,
            },
            "iron_rejection_document": {
                "path": (
                    "docs/development/physical-sound-realimpact-broadband-"
                    "counterfactual-result-ps2-2026-08-28.md"
                ),
                "sha256": IRON_REJECTION_DOC_SHA256,
            },
            "broadband_report": {
                "path": "../ps2-broadband-common-pole-control-v1/run-a/report.json",
                "sha256": BROADBAND_REPORT_SHA256,
            },
            "broadband_result_document": {
                "path": (
                    "docs/development/physical-sound-broadband-common-pole-"
                    "control-result-ps2-2026-08-28.md"
                ),
                "sha256": BROADBAND_RESULT_DOC_SHA256,
            },
            "broadband_runner": {
                "path": "lab/scripts/physical_sound_broadband_common_pole_control.py",
                "sha256": BROADBAND_RUNNER_SHA256,
            },
        },
        "fixture": {
            "sample_rate_hz": core.SAMPLE_RATE_HZ,
            "sample_count": core.SAMPLE_COUNT,
            "channel_count": core.CHANNEL_COUNT,
            "onset_sample": core.ONSET_SAMPLE,
            "region_bins": REGION_BINS,
            "strong_offsets_hz": STRONG_OFFSETS_HZ,
            "truth_frequencies_hz": frequencies,
            "truth_decays_per_second": decays,
            "participation_rule": (
                "(0.40 + 0.025*((5*channel + 7*mode) mod 13))/"
                "(1 + 0.015*(mode mod 4)); deterministic phase"
            ),
            "weak_nuisance": {
                "region_bins": WEAK_REGION_BINS,
                "frequencies_hz": weak_frequencies(),
                "amplitude": WEAK_AMPLITUDE,
                "decay_rule": "3 + 0.1*weak-index per second",
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
            "parent_id": broadband.REVISION,
            "only_algorithmic_change": (
                "top-seven partial SVD via scipy.sparse.linalg.svds PROPACK "
                "instead of complete SVD"
            ),
            "partial_svd": {
                "rank": PARTIAL_SVD_RANK,
                "solver": "propack",
                "which": "LM",
                "rng_seed_per_bin": 0,
            },
            "parent_equivalence": {
                "fixture": "frozen broad-band seven-mode parent holdout",
                "all_parent_analysis_bins": True,
                "compare": [
                    "selected order",
                    "accepted pole count",
                    "frequency",
                    "decay",
                    "order-score margin",
                ],
            },
            "gabor_and_discovery": {
                "source_sha256": BROADBAND_RUNNER_SHA256,
                "window_samples": broadband.WINDOW_SAMPLES,
                "hop_samples": broadband.HOP_SAMPLES,
                "frame_count": broadband.GABOR_FRAME_COUNT,
                "minimum_frequency_hz": broadband.MINIMUM_FREQUENCY_HZ,
                "maximum_frequency_hz": broadband.MAXIMUM_FREQUENCY_HZ,
                "relative_local_peak_floor_db": broadband.REGION_ENERGY_FLOOR_DB,
                "neighborhood_radius_bins": broadband.NEIGHBORHOOD_RADIUS_BINS,
            },
            "common_poles_and_resynthesis": {
                "minimum_order_score_margin": (
                    broadband.MINIMUM_ORDER_SCORE_MARGIN
                ),
                "duplicate_frequency_hz": broadband.DUPLICATE_FREQUENCY_HZ,
                "mode_energy_floor_db": broadband.MODE_ENERGY_FLOOR_DB,
                "amplitudes": (
                    "joint all-output least-squares over all post-onset samples"
                ),
            },
            "scale_factors": broadband.SCALE_FACTORS,
        },
        "gates": GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "research_sources": [
            {
                "url": "https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf",
                "bounded_claim": (
                    "real metal Gabor/ESPRIT analysis extracts dense modes and "
                    "removes duplicate or insignificant components after estimation"
                ),
            },
            {
                "url": "https://doi.org/10.1016/j.apnum.2014.10.003",
                "bounded_claim": (
                    "partial SVD and fast Hankel products reduce ESPRIT cost while "
                    "retaining noisy exponential-sum parameter identification"
                ),
            },
        ],
        "data_policy": {
            "synthetic_holdout_only": True,
            "calibration_seed_execution_allowed": False,
            "iron_payload_or_frequency_reuse_allowed": False,
            "network_allowed": False,
            "threshold_or_fixture_tuning_after_preflight_allowed": False,
            "physics_solver_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish support or rejection twice on the frozen dense holdout; "
            "on failure do not tune; on support only preregister an Iron V2 with "
            "the complete discovered region set and no opened-region ranking"
        ),
    }


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise DenseControlError(f"{label} must be external: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise DenseControlError(f"output must remain outside repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise DenseControlError(f"output must be absent or empty: {resolved}")
    return resolved


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 256 * 1024:
        raise DenseControlError("dense manifest exceeds 256 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise DenseControlError(f"parse dense manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise DenseControlError("dense manifest changed")
    return data, manifest


def validate_parents(
    root: Path, base: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    parents = manifest["parents"]
    store = base.parent.resolve(strict=True)

    def store_report(reference: dict[str, Any], label: str) -> dict[str, Any]:
        path = (base / reference["path"]).resolve(strict=True)
        if not path.is_file() or not path.is_relative_to(store):
            raise DenseControlError(f"{label} escapes physical-sound store")
        data = path.read_bytes()
        if sha256_bytes(data) != reference["sha256"]:
            raise DenseControlError(f"{label} hash changed")
        return json.loads(data)

    def repo_file(reference: dict[str, Any], label: str) -> Path:
        path = (root / reference["path"]).resolve(strict=True)
        if not path.is_file() or not path.is_relative_to(root):
            raise DenseControlError(f"{label} escapes repository")
        if sha256_file(path) != reference["sha256"]:
            raise DenseControlError(f"{label} hash changed")
        return path

    iron = store_report(parents["iron_rejection_report"], "Iron rejection")
    broad = store_report(parents["broadband_report"], "broad-band parent")
    iron_doc = repo_file(
        parents["iron_rejection_document"], "Iron result document"
    )
    broad_doc = repo_file(
        parents["broadband_result_document"], "broad-band result document"
    )
    broad_runner = repo_file(parents["broadband_runner"], "broad-band runner")
    if (
        iron.get("decision") != "ExistingIronBroadbandCounterfactualRejected"
        or iron.get("analysis_skipped_due_to_capacity") is not True
        or iron.get("additional_payload_bytes_read") != 0
        or iron.get("network_requests") != 0
        or iron.get("physics_solver_runs") != 0
        or iron.get("planter_payload_bytes_read") != 0
        or broad.get("decision") != "BroadbandCommonPoleSyntheticControlSupported"
        or broad.get("gate", {}).get("passed") is not True
        or broad.get("real_payload_bytes_read") != 0
        or broad.get("network_requests") != 0
    ):
        raise DenseControlError("dense-control parent decision changed")
    return {
        "iron_rejection_report_sha256": parents["iron_rejection_report"][
            "sha256"
        ],
        "iron_rejection_document_sha256": sha256_file(iron_doc),
        "iron_decision": iron["decision"],
        "iron_region_count": len(iron["discovery"]["regions"]),
        "iron_analysis_bin_count": len(iron["discovery"]["analysis_bins"]),
        "broadband_report_sha256": parents["broadband_report"]["sha256"],
        "broadband_result_document_sha256": sha256_file(broad_doc),
        "broadband_runner_sha256": sha256_file(broad_runner),
        "broadband_decision": broad["decision"],
    }


def generate_fixture(seed: int) -> tuple[np.ndarray, np.ndarray]:
    if seed == CALIBRATION_SEED:
        raise DenseControlError("calibration seed execution is forbidden")
    if seed != HOLDOUT_SEED:
        raise DenseControlError("unknown dense fixture seed")
    frequencies, decays = truth_parameters()
    observed = np.zeros(
        (core.CHANNEL_COUNT, core.SAMPLE_COUNT), dtype=np.float64
    )
    modal_truth = np.zeros_like(observed)
    time = np.arange(
        core.SAMPLE_COUNT - core.ONSET_SAMPLE, dtype=np.float64
    ) / core.SAMPLE_RATE_HZ
    for channel in range(core.CHANNEL_COUNT):
        for mode, (frequency, decay) in enumerate(
            zip(frequencies, decays, strict=True)
        ):
            gain = (
                0.40 + 0.025 * ((5 * channel + 7 * mode) % 13)
            ) / (1.0 + 0.015 * (mode % 4))
            phase = 0.13 * channel + 0.19 * mode + 0.009 * channel * mode
            values = (
                gain
                * np.exp(-decay * time)
                * np.cos(2.0 * math.pi * frequency * time + phase)
            )
            observed[channel, core.ONSET_SAMPLE :] += values
            modal_truth[channel, core.ONSET_SAMPLE :] += values
        for weak_index, frequency in enumerate(weak_frequencies()):
            observed[channel, core.ONSET_SAMPLE :] += (
                WEAK_AMPLITUDE
                * (0.8 + 0.01 * channel)
                * np.exp(-(3.0 + 0.1 * weak_index) * time)
                * np.cos(
                    2.0 * math.pi * frequency * time + 0.11 * channel
                )
            )
    generator = np.random.Generator(np.random.PCG64(seed))
    burst = generator.standard_normal(len(time)) * np.exp(
        -TRANSIENT_DECAY_PER_SECOND * time
    )
    for channel in range(core.CHANNEL_COUNT):
        observed[channel, core.ONSET_SAMPLE :] += (
            TRANSIENT_GAIN * (0.75 + 0.01 * channel) * burst
        )
    observed += NOISE_STANDARD_DEVIATION * generator.standard_normal(
        observed.shape
    )
    return observed, modal_truth


def estimate_partial_common_poles(
    values: np.ndarray, center_hz: float
) -> dict[str, Any]:
    output_count = values.shape[1]
    hankel = core.block_hankel(values)
    left, singular_values, _ = svds(
        hankel,
        k=PARTIAL_SVD_RANK,
        which="LM",
        solver="propack",
        rng=np.random.default_rng(0),
        return_singular_vectors=True,
    )
    order = np.argsort(singular_values)[::-1]
    singular_values = singular_values[order]
    left = left[:, order]
    order_scores = []
    for candidate_order in range(1, core.MAXIMUM_ORDER + 1):
        subspace = left[:, :candidate_order]
        upper = subspace[:-output_count]
        lower = subspace[output_count:]
        rotation = np.linalg.lstsq(upper, lower, rcond=None)[0]
        residual = lower - upper @ rotation
        denominator = max(
            float(np.linalg.norm(lower, "fro") ** 2), 1.0e-30
        )
        relative_error = (
            float(np.linalg.norm(residual, "fro") ** 2) / denominator
        )
        order_scores.append(
            {
                "order": candidate_order,
                "relative_rotational_error": relative_error,
                "inverse_error_score": 1.0 / max(relative_error, 1.0e-30),
                "singular_value": float(singular_values[candidate_order - 1]),
                "next_singular_value": float(
                    singular_values[candidate_order]
                ),
            }
        )
    selected = max(
        order_scores, key=lambda item: item["inverse_error_score"]
    )
    runner_up = max(
        item["inverse_error_score"]
        for item in order_scores
        if item["order"] != selected["order"]
    )
    subspace = left[:, : selected["order"]]
    rotation = np.linalg.lstsq(
        subspace[:-output_count], subspace[output_count:], rcond=None
    )[0]
    estimates = []
    alias_rate_hz = core.SAMPLE_RATE_HZ / core.HOP_SAMPLES
    for pole in np.linalg.eigvals(rotation):
        alias_frequency = (
            float(np.angle(pole)) * alias_rate_hz / (2.0 * math.pi)
        )
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
        "selected_order": selected["order"],
        "order_score_margin": (
            selected["inverse_error_score"] / runner_up
        ),
        "order_scores": order_scores,
        "accepted_pole_count": len(estimates),
        "rejected_pole_count": selected["order"] - len(estimates),
        "estimates": estimates,
    }


def extract_partial_estimates(
    coefficients: np.ndarray, discovery: dict[str, Any]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    raw = []
    analyses = []
    maximum = discovery["maximum_energy"]
    energies = discovery["spatial_energy"]
    bin_hz = discovery["bin_hz"]
    for index in discovery["analysis_bins"]:
        analysis = estimate_partial_common_poles(
            coefficients[index], index * bin_hz
        )
        relative_db = 10.0 * math.log10(
            max(float(energies[index]), 1.0e-300) / maximum
        )
        accepted = (
            analysis["order_score_margin"]
            >= broadband.MINIMUM_ORDER_SCORE_MARGIN
        )
        analyses.append(
            {
                "bin": index,
                "center_hz": index * bin_hz,
                "relative_energy_db": relative_db,
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
                        "source_relative_energy_db": relative_db,
                        "order_score_margin": analysis[
                            "order_score_margin"
                        ],
                    }
                )
    raw.sort(key=lambda item: item["frequency_hz"])
    return raw, analyses


def parent_equivalence() -> dict[str, Any]:
    observed, _ = broadband.generate_fixture()
    coefficients = broadband.gabor_coefficients(observed)
    discovery = broadband.discover_regions(coefficients)
    comparisons = []
    maximum_frequency = 0.0
    maximum_decay = 0.0
    maximum_margin = 0.0
    equivalent = True
    for bin_index in discovery["analysis_bins"]:
        center = bin_index * discovery["bin_hz"]
        full = core.estimate_common_poles(coefficients[bin_index], center)
        partial = estimate_partial_common_poles(
            coefficients[bin_index], center
        )
        equivalent &= (
            full["selected_order"] == partial["selected_order"]
            and len(full["estimates"]) == len(partial["estimates"])
        )
        frequency_delta = 0.0
        decay_delta = 0.0
        for expected, actual in zip(
            full["estimates"], partial["estimates"], strict=True
        ):
            frequency_delta = max(
                frequency_delta,
                abs(expected["frequency_hz"] - actual["frequency_hz"]),
            )
            decay_delta = max(
                decay_delta,
                abs(
                    expected["decay_per_second"]
                    - actual["decay_per_second"]
                ),
            )
        margin_delta = abs(
            full["order_score_margin"] - partial["order_score_margin"]
        )
        maximum_frequency = max(maximum_frequency, frequency_delta)
        maximum_decay = max(maximum_decay, decay_delta)
        maximum_margin = max(maximum_margin, margin_delta)
        comparisons.append(
            {
                "bin": bin_index,
                "center_hz": center,
                "full_order": full["selected_order"],
                "partial_order": partial["selected_order"],
                "full_pole_count": len(full["estimates"]),
                "partial_pole_count": len(partial["estimates"]),
                "maximum_frequency_delta_hz": frequency_delta,
                "maximum_decay_delta_per_second": decay_delta,
                "order_score_margin_delta": margin_delta,
            }
        )
    return {
        "order_and_count_equivalent": equivalent,
        "maximum_frequency_delta_hz": maximum_frequency,
        "maximum_decay_delta_per_second": maximum_decay,
        "maximum_order_score_margin_delta": maximum_margin,
        "comparisons": comparisons,
    }


def match_truth(retained: list[dict[str, Any]]) -> dict[str, Any]:
    frequencies, decays = truth_parameters()
    modes = [item["representative"] for item in retained]
    pairs = []
    for mode_index, mode in enumerate(modes):
        for truth_index, frequency in enumerate(frequencies):
            error = abs(mode["frequency_hz"] - frequency)
            if error <= 1.0:
                pairs.append((error, mode_index, truth_index))
    pairs.sort()
    assignments: list[int | None] = [None] * len(modes)
    used_truth: set[int] = set()
    for _, mode_index, truth_index in pairs:
        if assignments[mode_index] is None and truth_index not in used_truth:
            assignments[mode_index] = truth_index
            used_truth.add(truth_index)
    comparisons = []
    for mode, assignment in zip(modes, assignments, strict=True):
        if assignment is None:
            continue
        comparisons.append(
            {
                "expected_frequency_hz": frequencies[assignment],
                "observed_frequency_hz": mode["frequency_hz"],
                "frequency_error_hz": abs(
                    mode["frequency_hz"] - frequencies[assignment]
                ),
                "expected_decay_per_second": decays[assignment],
                "observed_decay_per_second": mode["decay_per_second"],
                "decay_error_per_second": abs(
                    mode["decay_per_second"] - decays[assignment]
                ),
            }
        )
    return {
        "truth_count": len(frequencies),
        "candidate_count": len(modes),
        "truth_match_count": len(comparisons),
        "false_positive_count": len(modes) - len(comparisons),
        "assignments": assignments,
        "comparisons": comparisons,
    }


def normalized_rms_error(expected: np.ndarray, observed: np.ndarray) -> float:
    denominator = math.sqrt(float(np.mean(expected * expected)))
    if denominator <= 1.0e-300:
        raise DenseControlError("reconstruction reference has zero RMS")
    return math.sqrt(float(np.mean((expected - observed) ** 2))) / denominator


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
        "decision": "DenseBroadbandScalingControlFrozen",
        "claim": (
            "SYNTHETIC_DENSE_PARTIAL_SVD_PREFLIGHT_ONLY / NO_CALIBRATION_REAL_"
            "PAYLOAD_PHYSICS_PLANTER_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parents": validate_parents(root, base, manifest),
        "fixture": manifest["fixture"],
        "candidate": manifest["candidate"],
        "gates": manifest["gates"],
        "runtime": manifest["runtime"],
        "next_action": "commit this preflight, then run the dense holdout twice",
    }


def run_control(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents = validate_parents(root, base, manifest)
    equivalence = parent_equivalence()
    observed, modal_truth = generate_fixture(HOLDOUT_SEED)
    coefficients = broadband.gabor_coefficients(observed)
    discovery = broadband.discover_regions(coefficients)
    scale_invariance = broadband.discovery_scale_invariance(coefficients)
    raw, bin_analyses = extract_partial_estimates(coefficients, discovery)
    clusters = broadband.cluster_duplicates(raw)
    fitted, reconstruction = broadband.fit_amplitudes(observed, clusters)
    retained = [item for item in fitted if item["retained"]]
    truth = match_truth(retained)
    region_bins = {item["bin"] for item in discovery["regions"]}
    selected_weak = sorted(region_bins.intersection(WEAK_REGION_BINS))
    frequency_errors = [
        item["frequency_error_hz"] for item in truth["comparisons"]
    ]
    decay_errors = [
        item["decay_error_per_second"] for item in truth["comparisons"]
    ]
    modal_reference = modal_truth[:, core.ONSET_SAMPLE :]
    observed_reference = observed[:, core.ONSET_SAMPLE :]
    post_transient_sample = math.ceil(
        math.log(1_000.0)
        / TRANSIENT_DECAY_PER_SECOND
        * core.SAMPLE_RATE_HZ
    )
    reconstruction_metrics = {
        "modal_truth_nrmse": normalized_rms_error(
            modal_reference, reconstruction
        ),
        "observed_full_nrmse": normalized_rms_error(
            observed_reference, reconstruction
        ),
        "post_transient_start_sample": post_transient_sample,
        "post_transient_start_ms": (
            post_transient_sample * 1_000.0 / core.SAMPLE_RATE_HZ
        ),
        "observed_post_transient_nrmse": normalized_rms_error(
            observed_reference[:, post_transient_sample:],
            reconstruction[:, post_transient_sample:],
        ),
    }
    duplicate_estimates_removed = len(raw) - len(clusters)
    checks = [
        check(
            "parent_order_and_count_equivalent",
            equivalence["order_and_count_equivalent"],
            "==",
            GATES["parent_order_and_count_equivalent"],
        ),
        check(
            "parent_frequency_delta_hz",
            equivalence["maximum_frequency_delta_hz"],
            "<=",
            GATES["maximum_parent_frequency_delta_hz"],
        ),
        check(
            "parent_decay_delta_per_second",
            equivalence["maximum_decay_delta_per_second"],
            "<=",
            GATES["maximum_parent_decay_delta_per_second"],
        ),
        check(
            "parent_order_margin_delta",
            equivalence["maximum_order_score_margin_delta"],
            "<=",
            GATES["maximum_parent_order_margin_delta"],
        ),
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
            "selected_weak_region_count",
            len(selected_weak),
            "==",
            GATES["selected_weak_region_count"],
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
            "DenseBroadbandScalingSyntheticControlSupported"
            if passed
            else "DenseBroadbandScalingSyntheticControlRejected"
        ),
        "claim": (
            "SYNTHETIC_DENSE_PARTIAL_SVD_EQUIVALENCE_DISCOVERY_TO_"
            "RESYNTHESIS_ONLY / NO_REAL_TRANSFER_PHYSICS_PLANTER_QUALITY_"
            "ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "parent_equivalence": equivalence,
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
            "selected_weak_region_bins": selected_weak,
        },
        "scale_invariance": scale_invariance,
        "bin_analyses": bin_analyses,
        "raw_estimate_count": len(raw),
        "duplicate_estimates_removed": duplicate_estimates_removed,
        "fitted_clusters": fitted,
        "truth": truth,
        "reconstruction": reconstruction_metrics,
        "deterministic_work_envelope": {
            "region_count": len(discovery["regions"]),
            "analysis_bin_count": len(discovery["analysis_bins"]),
            "partial_svd_rank": PARTIAL_SVD_RANK,
            "maximum_order": core.MAXIMUM_ORDER,
            "preprune_cluster_count": len(clusters),
            "amplitude_design_column_count": 2 * len(clusters),
        },
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze an Iron V2 over all discovered regions without ranking"
            if passed
            else "reject the revision; do not tune the holdout or reopen Iron"
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
    print(f"Dense broad-band {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("real payload bytes read: 0")
    print("network requests: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
