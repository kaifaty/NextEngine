#!/usr/bin/env python3
"""Run the frozen all-region partial-SVD candidate on existing Iron rows."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_broadband_common_pole_control as broadband
import physical_sound_dense_broadband_scaling_control as dense
import physical_sound_realimpact_broadband_counterfactual as v1
import scipy

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-broadband-v2-counterfactual.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-broadband-v2-counterfactual-"
        "preflight.report.v1"
    ),
    "analyze": (
        "nextengine.experimental-realimpact-broadband-v2-counterfactual.report.v1"
    ),
}
STUDY_ID = "physical-sound-realimpact-broadband-v2-counterfactual"
REVISION = "iron-skillet-all-region-partial-svd-counterfactual-v2"

V1_RUNNER_SHA256 = "63d46bf476798e540cfc726a0e2bca58beaacc0f829ffff7ecc2b1083efb2e76"
V1_MANIFEST_SHA256 = "8a445a4f0474ec560e659f6bf8ada9a90f4fb04fd5c04ef3e308843fcb43ff18"
V1_REPORT_SHA256 = "7635f8aca44286e0a46709aa3268afd049d1038023768a8a265d6a0b24a5075d"
V1_RESULT_DOC_SHA256 = (
    "f9f6da5f3be8e566f0c27b5e2736200d2e79382bb90639b6a95e35ee32255a81"
)
DENSE_RUNNER_SHA256 = "8e4cc18ba25f90adb43483e5d4ff1ea7be46869e55c28ebd7c9bb5831b29fbbe"
DENSE_MANIFEST_SHA256 = (
    "5ca8b1dad8a1d68f2cea5042c0a6ff305ffa6edc8658ea5739e1a8d0909e1ab0"
)
DENSE_REPORT_SHA256 = "b9a5b226e2f96ecca7830d3adc6374f3c11baceed3b78e06eb71a2e58201978b"
DENSE_RESULT_DOC_SHA256 = (
    "e55615f3e7412bd8ca54bd075c9b235ed25d673071df577b2dd1be85622ced00"
)

GATES = {
    "required_region_count": 31,
    "required_analysis_bin_count": 91,
    "maximum_region_count": 32,
    "maximum_analysis_bin_count": 96,
    "minimum_duplicate_estimates_removed": 1,
    "minimum_preprune_cluster_count": 6,
    "maximum_preprune_cluster_count": 64,
    "minimum_retained_mode_count": 6,
    "maximum_retained_mode_count": 64,
    "scale_invariant_region_bins": True,
    "minimum_even_partition_match_fraction": 0.50,
    "minimum_odd_partition_match_fraction": 0.50,
    "maximum_full_observed_nrmse": 0.95,
    "maximum_full_damped_to_undamped_nrmse_ratio": 0.95,
    "maximum_prediction_error_ratio_window_a": 0.95,
    "maximum_prediction_error_ratio_window_b": 0.95,
}


class CounterfactualV2Error(RuntimeError):
    """The frozen V2 counterfactual or one of its parents changed."""


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
        "parents": {
            "v1_runner": {
                "path": "lab/scripts/physical_sound_realimpact_broadband_counterfactual.py",
                "sha256": V1_RUNNER_SHA256,
            },
            "v1_manifest": {
                "path": (
                    "../ps2-realimpact-iron-skillet-broadband-"
                    "counterfactual-v1/manifest.json"
                ),
                "sha256": V1_MANIFEST_SHA256,
            },
            "v1_report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-broadband-"
                    "counterfactual-v1/analysis-a/report.json"
                ),
                "sha256": V1_REPORT_SHA256,
            },
            "v1_result_document": {
                "path": (
                    "docs/development/physical-sound-realimpact-broadband-"
                    "counterfactual-result-ps2-2026-08-28.md"
                ),
                "sha256": V1_RESULT_DOC_SHA256,
            },
            "dense_runner": {
                "path": (
                    "lab/scripts/physical_sound_dense_broadband_scaling_control.py"
                ),
                "sha256": DENSE_RUNNER_SHA256,
            },
            "dense_manifest": {
                "path": ("../ps2-dense-broadband-scaling-control-v1/manifest.json"),
                "sha256": DENSE_MANIFEST_SHA256,
            },
            "dense_report": {
                "path": ("../ps2-dense-broadband-scaling-control-v1/run-a/report.json"),
                "sha256": DENSE_REPORT_SHA256,
            },
            "dense_result_document": {
                "path": (
                    "docs/development/physical-sound-dense-broadband-scaling-"
                    "control-result-ps2-2026-08-28.md"
                ),
                "sha256": DENSE_RESULT_DOC_SHA256,
            },
        },
        "observation": {
            "dataset_object_id": "17_IronSkillet",
            "impact_ordinal": 0,
            "role": "opened_development_counterfactual_only",
            "sample_rate_hz": v1.SAMPLE_RATE_HZ,
            "source_shape": [v1.SOURCE_ROW_COUNT, v1.SOURCE_SAMPLE_COUNT],
            "analysis_rows": v1.ANALYSIS_ROWS,
            "onset_sample": v1.ONSET_SAMPLE,
            "post_onset_observation_samples": v1.OBSERVATION_SAMPLES,
            "sample_format": "f32le-row-major",
            "normalization": "none",
        },
        "candidate": {
            "parent_id": broadband.REVISION,
            "only_algorithmic_change_from_v1": (
                "rank-seven scipy.sparse.linalg.svds PROPACK partial SVD "
                "instead of complete SVD"
            ),
            "partial_svd": {
                "rank": dense.PARTIAL_SVD_RANK,
                "solver": "propack",
                "which": "LM",
                "rng_seed_per_bin": 0,
            },
            "discovery": {
                "all_v1_regions_required": True,
                "opened_region_ranking": "forbidden",
                "minimum_frequency_hz": broadband.MINIMUM_FREQUENCY_HZ,
                "maximum_frequency_hz": broadband.MAXIMUM_FREQUENCY_HZ,
                "relative_local_peak_floor_db": broadband.REGION_ENERGY_FLOOR_DB,
                "neighborhood_radius_bins": broadband.NEIGHBORHOOD_RADIUS_BINS,
            },
            "common_poles": {
                "minimum_order_score_margin": broadband.MINIMUM_ORDER_SCORE_MARGIN,
                "duplicate_frequency_hz": broadband.DUPLICATE_FREQUENCY_HZ,
            },
            "amplitudes": {
                "fit": (
                    "joint all-output least-squares cosine/sine coefficients "
                    "over the frozen 60000-sample post-onset observation"
                ),
                "mode_energy_floor_db": broadband.MODE_ENERGY_FLOOR_DB,
                "pruning_order": "after pole estimation and amplitude fit",
                "maximum_design_columns": 128,
            },
            "scale_factors": broadband.SCALE_FACTORS,
        },
        "controls": {
            "spatial_partitions": v1.SPATIAL_PARTITIONS,
            "partition_matching_tolerance_cents": v1.MATCH_TOLERANCE_CENTS,
            "partition_matching": (
                "injective minimum-cents matching of retained full-output "
                "frequencies to unpruned partition estimates"
            ),
            "undamped_ablation": (
                "same discovered frequencies and joint fit, every decay set to zero"
            ),
            "prediction": {
                "amplitude_calibration_samples": [
                    0,
                    v1.PREDICTION_CALIBRATION_SAMPLES,
                ],
                "holdout_windows": v1.PREDICTION_WINDOWS,
                "error_ratio": (
                    "damped squared error divided by undamped squared error"
                ),
            },
        },
        "gates": GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "data_policy": {
            "existing_decoded_rows_only": True,
            "new_network_or_payload_access_allowed": False,
            "threshold_or_variant_tuning_after_preflight_allowed": False,
            "opened_region_ranking_allowed": False,
            "new_object_or_retry_allowed": False,
            "physics_solver_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish support or rejection twice on every hash-bound V1 Iron "
            "region; on failure do not tune or open payload; on support freeze "
            "an independent internet-sourced real holdout before any domain, "
            "material or quality claim"
        ),
    }


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise CounterfactualV2Error(f"{label} must be external: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise CounterfactualV2Error(
            f"output must remain outside repository: {resolved}"
        )
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise CounterfactualV2Error(f"output must be absent or empty: {resolved}")
    return resolved


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise CounterfactualV2Error("V2 manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise CounterfactualV2Error(f"parse V2 manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise CounterfactualV2Error("V2 manifest changed")
    return data, manifest


def checked_store_file(base: Path, reference: dict[str, Any], label: str) -> Path:
    path = (base / reference["path"]).resolve(strict=True)
    store = base.parent.resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(store):
        raise CounterfactualV2Error(f"{label} escapes physical-sound store")
    if sha256_file(path) != reference["sha256"]:
        raise CounterfactualV2Error(f"{label} hash changed")
    return path


def checked_repo_file(root: Path, reference: dict[str, Any], label: str) -> Path:
    path = (root / reference["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(root):
        raise CounterfactualV2Error(f"{label} escapes repository")
    if sha256_file(path) != reference["sha256"]:
        raise CounterfactualV2Error(f"{label} hash changed")
    return path


def validate_lineage(
    root: Path, base: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    parents = manifest["parents"]
    v1_runner = checked_repo_file(root, parents["v1_runner"], "V1 runner")
    v1_doc = checked_repo_file(
        root, parents["v1_result_document"], "V1 result document"
    )
    dense_runner = checked_repo_file(root, parents["dense_runner"], "dense runner")
    dense_doc = checked_repo_file(
        root, parents["dense_result_document"], "dense result document"
    )
    v1_manifest_path = checked_store_file(base, parents["v1_manifest"], "V1 manifest")
    v1_report_path = checked_store_file(base, parents["v1_report"], "V1 report")
    dense_manifest_path = checked_store_file(
        base, parents["dense_manifest"], "dense manifest"
    )
    dense_report_path = checked_store_file(
        base, parents["dense_report"], "dense report"
    )

    _, v1_manifest = v1.load_manifest(v1_manifest_path, V1_RUNNER_SHA256)
    v1_lineage = v1.validate_lineage(root, v1_manifest_path.parent, v1_manifest)
    v1_report = json.loads(v1_report_path.read_bytes())
    if (
        v1_report.get("decision") != "ExistingIronBroadbandCounterfactualRejected"
        or v1_report.get("analysis_skipped_due_to_capacity") is not True
        or len(v1_report.get("discovery", {}).get("regions", []))
        != GATES["required_region_count"]
        or len(v1_report.get("discovery", {}).get("analysis_bins", []))
        != GATES["required_analysis_bin_count"]
        or v1_report.get("additional_payload_bytes_read") != 0
        or v1_report.get("network_requests") != 0
        or v1_report.get("physics_solver_runs") != 0
        or v1_report.get("planter_payload_bytes_read") != 0
        or v1_report.get("quality_admission_or_runtime_credit") is not False
    ):
        raise CounterfactualV2Error("V1 Iron capacity parent changed")

    _, dense_manifest = dense.load_manifest(dense_manifest_path, DENSE_RUNNER_SHA256)
    dense.validate_parents(root, dense_manifest_path.parent, dense_manifest)
    dense_report = json.loads(dense_report_path.read_bytes())
    if (
        dense_report.get("decision") != "DenseBroadbandScalingSyntheticControlSupported"
        or dense_report.get("gate", {}).get("passed") is not True
        or len(dense_report.get("discovery", {}).get("regions", [])) != 32
        or len(dense_report.get("discovery", {}).get("analysis_bins", [])) != 96
        or dense_report.get("truth", {}).get("truth_match_count") != 64
        or dense_report.get("truth", {}).get("false_positive_count") != 0
        or dense_report.get("real_payload_bytes_read") != 0
        or dense_report.get("network_requests") != 0
        or dense_report.get("physics_solver_runs") != 0
        or dense_report.get("planter_payload_bytes_read") != 0
        or dense_report.get("quality_admission_or_runtime_credit") is not False
    ):
        raise CounterfactualV2Error("dense synthetic support parent changed")

    return {
        "v1_runner_sha256": sha256_file(v1_runner),
        "v1_manifest_sha256": sha256_file(v1_manifest_path),
        "v1_report_sha256": sha256_file(v1_report_path),
        "v1_result_document_sha256": sha256_file(v1_doc),
        "v1_decision": v1_report["decision"],
        "v1_discovery": v1_report["discovery"],
        "dense_runner_sha256": sha256_file(dense_runner),
        "dense_manifest_sha256": sha256_file(dense_manifest_path),
        "dense_report_sha256": sha256_file(dense_report_path),
        "dense_result_document_sha256": sha256_file(dense_doc),
        "dense_decision": dense_report["decision"],
        "decoded_block_path": v1_lineage["decoded_block_path"],
        "decoded_block_sha256": v1_lineage["iron_decoded_block_sha256"],
        "decoded_block_bytes": v1_lineage["iron_decoded_block_bytes"],
    }


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": "17_IronSkillet",
        "impact_ordinal": 0,
        "role": "opened_development_counterfactual_only",
        "analysis_rows": v1.ANALYSIS_ROWS,
        "onset_sample": v1.ONSET_SAMPLE,
        "additional_payload_bytes_read": 0,
        "network_requests": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
        "opened_region_ranking_used": False,
    }


def public_lineage(lineage: dict[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in lineage.items()
        if key not in {"decoded_block_path", "v1_discovery"}
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    lineage = validate_lineage(root, base, manifest)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "ExistingIronBroadbandV2CounterfactualFrozen",
        "claim": (
            "EXISTING_IRON_ALL_REGION_PARTIAL_SVD_PREFLIGHT_ONLY / NO_NEW_"
            "PAYLOAD_OBJECT_PHYSICS_PLANTER_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "lineage": public_lineage(lineage),
        "frozen_parent_discovery": lineage["v1_discovery"],
        "observation": manifest["observation"],
        "candidate": manifest["candidate"],
        "controls": manifest["controls"],
        "gates": manifest["gates"],
        "runtime": manifest["runtime"],
        "next_action": "commit this preflight, then analyze the Iron rows twice",
    }


def analyze_partition(
    coefficients: np.ndarray,
    discovery: dict[str, Any],
    output_indices: list[int],
    retained_modes: list[dict[str, Any]],
) -> dict[str, Any]:
    raw, analyses = dense.extract_partial_estimates(
        coefficients[:, :, output_indices], discovery
    )
    clusters = broadband.cluster_duplicates(raw)
    return {
        "output_indices": output_indices,
        "raw_estimate_count": len(raw),
        "cluster_count": len(clusters),
        "clusters": clusters,
        "bin_analyses": analyses,
        "retained_full_output_matching": v1.injective_match(
            retained_modes, v1.representatives(clusters)
        ),
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


def rejected_report(
    *,
    common: dict[str, Any],
    lineage: dict[str, Any],
    discovery: dict[str, Any],
    scale_invariance: dict[str, Any],
    checks: list[dict[str, Any]],
    stage: str,
    reason: str,
    details: dict[str, Any] | None = None,
) -> dict[str, Any]:
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": "ExistingIronBroadbandV2CounterfactualRejected",
        "claim": (
            "EXISTING_IRON_READ_ONLY_METHOD_OR_CAPACITY_REJECTION / NO_"
            "MATERIAL_IDENTITY_PERCEPTUAL_QUALITY_DOMAIN_ADMISSION_PHYSICS_"
            "PLANTER_OR_RUNTIME_CREDIT"
        ),
        **common,
        "lineage": public_lineage(lineage),
        "existing_decoded_payload_bytes_analyzed": (
            len(v1.ANALYSIS_ROWS)
            * (v1.ONSET_SAMPLE + v1.OBSERVATION_SAMPLES)
            * np.dtype("<f4").itemsize
        ),
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "all_parent_regions_reproduced": (
            discovery["regions"] == lineage["v1_discovery"]["regions"]
            and discovery["analysis_bins"] == lineage["v1_discovery"]["analysis_bins"]
        ),
        "scale_invariance": scale_invariance,
        "rejection_stage": stage,
        "rejection_reason": reason,
        "details": details or {},
        "gate": {"passed": False, "checks": checks},
        "next_action": (
            "preserve the rejection and research the failed gate without "
            "tuning Iron or opening new payload"
        ),
    }


def run_counterfactual(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    lineage = validate_lineage(root, base, manifest)
    channels = v1.load_observation(Path(lineage["decoded_block_path"]))
    post_onset = channels[:, v1.ONSET_SAMPLE : v1.ONSET_SAMPLE + v1.OBSERVATION_SAMPLES]
    gabor_stop = (
        broadband.WINDOW_SAMPLES
        + (broadband.GABOR_FRAME_COUNT - 1) * broadband.HOP_SAMPLES
    )
    coefficients = broadband.gabor_coefficients(channels[:, :gabor_stop])
    discovery = broadband.discover_regions(coefficients)
    scale_invariance = broadband.discovery_scale_invariance(coefficients)
    common = common_report(manifest_bytes, runner_sha256)
    parent_discovery_exact = (
        discovery["regions"] == lineage["v1_discovery"]["regions"]
        and discovery["analysis_bins"] == lineage["v1_discovery"]["analysis_bins"]
    )
    discovery_checks = [
        check(
            "parent_discovery_exact",
            parent_discovery_exact,
            "==",
            True,
        ),
        check(
            "required_region_count",
            len(discovery["regions"]),
            "==",
            GATES["required_region_count"],
        ),
        check(
            "required_analysis_bin_count",
            len(discovery["analysis_bins"]),
            "==",
            GATES["required_analysis_bin_count"],
        ),
        check(
            "maximum_region_count",
            len(discovery["regions"]),
            "<=",
            GATES["maximum_region_count"],
        ),
        check(
            "maximum_analysis_bin_count",
            len(discovery["analysis_bins"]),
            "<=",
            GATES["maximum_analysis_bin_count"],
        ),
        check(
            "scale_invariant_region_bins",
            scale_invariance["passed"],
            "==",
            GATES["scale_invariant_region_bins"],
        ),
    ]
    if not all(item["passed"] for item in discovery_checks):
        return rejected_report(
            common=common,
            lineage=lineage,
            discovery=discovery,
            scale_invariance=scale_invariance,
            checks=discovery_checks,
            stage="discovery",
            reason="frozen all-region discovery identity or envelope failed",
        )

    raw, bin_analyses = dense.extract_partial_estimates(coefficients, discovery)
    clusters = broadband.cluster_duplicates(raw)
    duplicate_estimates_removed = len(raw) - len(clusters)
    cluster_checks = [
        *discovery_checks,
        check(
            "duplicate_estimates_removed",
            duplicate_estimates_removed,
            ">=",
            GATES["minimum_duplicate_estimates_removed"],
        ),
        check(
            "minimum_preprune_cluster_count",
            len(clusters),
            ">=",
            GATES["minimum_preprune_cluster_count"],
        ),
        check(
            "maximum_preprune_cluster_count",
            len(clusters),
            "<=",
            GATES["maximum_preprune_cluster_count"],
        ),
    ]
    if not all(item["passed"] for item in cluster_checks):
        return rejected_report(
            common=common,
            lineage=lineage,
            discovery=discovery,
            scale_invariance=scale_invariance,
            checks=cluster_checks,
            stage="pole_estimation",
            reason="partial-SVD cluster count left the supported fit envelope",
            details={
                "bin_analyses": bin_analyses,
                "raw_estimate_count": len(raw),
                "duplicate_estimates_removed": duplicate_estimates_removed,
                "preprune_cluster_count": len(clusters),
            },
        )

    fitted, candidate_reconstruction, undamped_reconstruction = v1.fit_and_prune(
        post_onset, clusters
    )
    retained = [item for item in fitted if item["retained"]]
    retained_modes = [item["representative"] for item in retained]
    retained_checks = [
        *cluster_checks,
        check(
            "minimum_retained_mode_count",
            len(retained),
            ">=",
            GATES["minimum_retained_mode_count"],
        ),
        check(
            "maximum_retained_mode_count",
            len(retained),
            "<=",
            GATES["maximum_retained_mode_count"],
        ),
    ]
    if not all(item["passed"] for item in retained_checks):
        return rejected_report(
            common=common,
            lineage=lineage,
            discovery=discovery,
            scale_invariance=scale_invariance,
            checks=retained_checks,
            stage="amplitude_pruning",
            reason="retained mode count left the frozen method envelope",
            details={
                "bin_analyses": bin_analyses,
                "raw_estimate_count": len(raw),
                "duplicate_estimates_removed": duplicate_estimates_removed,
                "fitted_clusters": fitted,
                "retained_mode_count": len(retained),
            },
        )

    candidate_nrmse = v1.normalized_rms_error(post_onset, candidate_reconstruction)
    undamped_nrmse = v1.normalized_rms_error(post_onset, undamped_reconstruction)
    full_ratio = candidate_nrmse / max(undamped_nrmse, 1.0e-300)
    prediction = v1.prediction_control(post_onset, retained_modes)
    partitions = {
        name: analyze_partition(coefficients, discovery, indices, retained_modes)
        for name, indices in v1.SPATIAL_PARTITIONS.items()
    }
    even_fraction = partitions["even_microphones"]["retained_full_output_matching"][
        "match_fraction"
    ]
    odd_fraction = partitions["odd_microphones"]["retained_full_output_matching"][
        "match_fraction"
    ]
    checks = [
        *retained_checks,
        check(
            "even_partition_match_fraction",
            even_fraction,
            ">=",
            GATES["minimum_even_partition_match_fraction"],
        ),
        check(
            "odd_partition_match_fraction",
            odd_fraction,
            ">=",
            GATES["minimum_odd_partition_match_fraction"],
        ),
        check(
            "full_observed_nrmse",
            candidate_nrmse,
            "<=",
            GATES["maximum_full_observed_nrmse"],
        ),
        check(
            "full_damped_to_undamped_nrmse_ratio",
            full_ratio,
            "<=",
            GATES["maximum_full_damped_to_undamped_nrmse_ratio"],
        ),
        check(
            "prediction_error_ratio_window_a",
            prediction["windows"][0]["damped_to_undamped_error_ratio"],
            "<=",
            GATES["maximum_prediction_error_ratio_window_a"],
        ),
        check(
            "prediction_error_ratio_window_b",
            prediction["windows"][1]["damped_to_undamped_error_ratio"],
            "<=",
            GATES["maximum_prediction_error_ratio_window_b"],
        ),
    ]
    passed = all(item["passed"] for item in checks)
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": (
            "ExistingIronBroadbandV2MethodTransferSupported"
            if passed
            else "ExistingIronBroadbandV2CounterfactualRejected"
        ),
        "claim": (
            "EXISTING_IRON_READ_ONLY_ALL_REGION_PARTIAL_SVD_METHOD_TRANSFER_"
            "ONLY / NO_MATERIAL_IDENTITY_PERCEPTUAL_QUALITY_DOMAIN_ADMISSION_"
            "PHYSICS_PLANTER_OR_RUNTIME_CREDIT"
        ),
        **common,
        "lineage": public_lineage(lineage),
        "existing_decoded_payload_bytes_analyzed": (
            len(v1.ANALYSIS_ROWS)
            * (v1.ONSET_SAMPLE + v1.OBSERVATION_SAMPLES)
            * np.dtype("<f4").itemsize
        ),
        "input_slice_sha256": sha256_bytes(
            channels.astype("<f8", copy=False).tobytes()
        ),
        "gabor_coefficients_sha256": sha256_bytes(
            coefficients.astype("<c16", copy=False).tobytes()
        ),
        "discovery": {
            "regions": discovery["regions"],
            "analysis_bins": discovery["analysis_bins"],
        },
        "all_parent_regions_reproduced": parent_discovery_exact,
        "all_discovered_regions_analyzed": len(bin_analyses)
        == len(discovery["analysis_bins"]),
        "scale_invariance": scale_invariance,
        "bin_analyses": bin_analyses,
        "raw_estimate_count": len(raw),
        "duplicate_estimates_removed": duplicate_estimates_removed,
        "fitted_clusters": fitted,
        "retained_mode_count": len(retained),
        "pruned_mode_count": len(fitted) - len(retained),
        "spatial_partitions": partitions,
        "reconstruction": {
            "observation_samples": v1.OBSERVATION_SAMPLES,
            "candidate_nrmse": candidate_nrmse,
            "undamped_nrmse": undamped_nrmse,
            "damped_to_undamped_nrmse_ratio": full_ratio,
        },
        "prediction_control": prediction,
        "deterministic_work_envelope": {
            "region_count": len(discovery["regions"]),
            "analysis_bin_count": len(discovery["analysis_bins"]),
            "partial_svd_rank": dense.PARTIAL_SVD_RANK,
            "preprune_cluster_count": len(clusters),
            "amplitude_design_columns": 2 * len(clusters),
        },
        "gate": {"passed": passed, "checks": checks},
        "next_action": (
            "freeze an independent internet-sourced real holdout before any "
            "domain, material or quality claim"
            if passed
            else "preserve the rejection and research the failed gate without "
            "tuning Iron or opening new payload"
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
        else run_counterfactual(root, base, manifest_bytes, manifest, runner_sha256)
    )
    report_bytes = publish(output, report)
    print(f"Existing-Iron broad-band V2 {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("additional payload bytes read: 0")
    print("network requests: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
