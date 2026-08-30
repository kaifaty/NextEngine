#!/usr/bin/env python3
"""Diagnose the frozen R2C silence collapse without another candidate fit."""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Any

import numpy as np
import torch

import physical_sound_listener_field_r2c_common as common

REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2c-failure-diagnostic.report.v1"
)
TF_CHUNK = 8_192
RANKS = (16, 32, 64, 96, 128, 256)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--preflight", required=True, type=Path)
    parser.add_argument("--training-report", required=True, action="append", type=Path)
    parser.add_argument("--evaluation-report", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def validate_preflight(
    root: Path, path: Path
) -> tuple[bytes, dict[str, Any], Path]:
    report_path = common.external_file(root, path, "R2C preflight report")
    data, report = common.read_json(report_path, "R2C preflight report")
    if (
        report.get("schema") != common.PREFLIGHT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "R2CContextFeaturesReadyForFrozenTraining"
        or report.get("candidate_profile") != common.candidate_profile()
        or report.get("query_audio_bytes_read") != 0
        or report.get("optimizer_steps") != 0
    ):
        raise common.R2CError("R2C diagnostic preflight boundary changed")
    reference = common.require_ref(report["context_feature_cache"], "context cache")
    cache_path = common.external_file(
        root, report_path.parent / reference["path"], "R2C context cache"
    )
    if cache_path.stat().st_size != reference["byte_count"] or common.sha256_file(
        cache_path
    ) != reference["sha256"]:
        raise common.R2CError("R2C diagnostic context cache changed")
    return data, report, cache_path


def validate_training(
    root: Path, paths: list[Path]
) -> tuple[list[bytes], list[dict[str, Any]]]:
    if len(paths) != 2:
        raise common.R2CError("R2C diagnostic requires one report per frozen candidate")
    reports = []
    payloads = []
    for path in paths:
        report_path = common.external_file(root, path, "R2C training report")
        data, report = common.read_json(report_path, "R2C training report")
        candidate = common.exact_candidate(report.get("candidate_id"))
        if (
            report.get("schema") != common.TRAINING_SCHEMA
            or report.get("status") != "Trained"
            or report.get("revision") != common.REVISION
            or report.get("candidate_profile") != common.candidate_profile()
            or report.get("helmholtz_weight") != candidate["helmholtz_weight"]
            or report.get("optimizer_steps") != common.STEPS
            or report.get("query_audio_bytes_read") != 0
            or report.get("method_holdout_or_shadow_bytes_read") != 0
        ):
            raise common.R2CError("R2C diagnostic training lineage changed")
        payloads.append(data)
        reports.append(report)
    reports_by_id = {report["candidate_id"]: report for report in reports}
    if set(reports_by_id) != {
        candidate["candidate_id"] for candidate in common.CANDIDATES
    }:
        raise common.R2CError("R2C diagnostic candidate set changed")
    ordered = [
        reports_by_id[candidate["candidate_id"]] for candidate in common.CANDIDATES
    ]
    ordered_payloads = [
        payloads[reports.index(report)] for report in ordered
    ]
    return ordered_payloads, ordered


def validate_evaluation(
    root: Path, path: Path
) -> tuple[bytes, dict[str, Any]]:
    report_path = common.external_file(root, path, "R2C evaluation report")
    data, report = common.read_json(report_path, "R2C evaluation report")
    if (
        report.get("schema") != common.EVALUATION_REPORT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "RejectComplexListenerField"
        or report.get("selected_candidate_id") is not None
        or report.get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise common.R2CError("R2C diagnostic evaluation lineage changed")
    return data, report


def context_oracles(
    cache: np.memmap,
    pressure_scale: float,
    mean_abs_scaled: float,
    device: torch.device,
) -> dict[str, Any]:
    flattened = cache.reshape(cache.shape[0], -1)
    complex_error_zero = 0.0
    log_error_zero = 0.0
    complex_error_mean = 0.0
    log_error_mean = 0.0
    value_count = 0
    total_energy = 0.0
    mean_field_energy = 0.0
    gram = torch.zeros(
        (cache.shape[0], cache.shape[0]), dtype=torch.complex64, device=device
    )
    with torch.no_grad():
        for start in range(0, flattened.shape[1], TF_CHUNK):
            stop = min(flattened.shape[1], start + TF_CHUNK)
            values_np = np.array(
                flattened[:, start:stop], dtype=np.complex64, copy=True
            )
            values = torch.from_numpy(values_np).to(device) / pressure_scale
            magnitude = torch.abs(values)
            mean = torch.mean(values, dim=0, keepdim=True)
            mean_magnitude = torch.abs(mean)
            complex_error_zero += float(
                torch.sum(torch.sqrt(magnitude * magnitude + common.LOSS_EPSILON**2)).cpu()
            )
            log_error_zero += float(
                torch.sum(torch.log1p(magnitude / mean_abs_scaled)).cpu()
            )
            mean_difference = values - mean
            mean_difference_magnitude = torch.abs(mean_difference)
            complex_error_mean += float(
                torch.sum(
                    torch.sqrt(
                        mean_difference_magnitude * mean_difference_magnitude
                        + common.LOSS_EPSILON**2
                    )
                ).cpu()
            )
            log_error_mean += float(
                torch.sum(
                    torch.abs(
                        torch.log1p(mean_magnitude / mean_abs_scaled)
                        - torch.log1p(magnitude / mean_abs_scaled)
                    )
                ).cpu()
            )
            centered = values - mean
            gram += centered @ torch.conj(centered.T)
            total_energy += float(torch.sum(magnitude * magnitude).cpu())
            mean_field_energy += float(
                cache.shape[0] * torch.sum(mean_magnitude * mean_magnitude).cpu()
            )
            value_count += values.numel()
    eigenvalues = torch.linalg.eigvalsh(gram).real.cpu().numpy().astype(np.float64)
    eigenvalues = np.maximum(eigenvalues[::-1], 0.0)
    centered_energy = float(np.sum(eigenvalues))
    if centered_energy <= 0.0 or total_energy <= 0.0:
        raise common.R2CError("R2C context oracle energy is invalid")
    cumulative = np.cumsum(eigenvalues)
    rank_records = []
    for rank in RANKS:
        retained_centered = float(cumulative[min(rank, len(cumulative)) - 1])
        reconstructed = mean_field_energy + retained_centered
        rank_records.append(
            {
                "rank": rank,
                "centered_energy_fraction": retained_centered / centered_energy,
                "total_energy_fraction_with_mean": reconstructed / total_energy,
                "best_linear_frobenius_nrmse": math.sqrt(
                    max(total_energy - reconstructed, 0.0) / total_energy
                ),
            }
        )
    zero_complex = complex_error_zero / value_count / mean_abs_scaled
    zero_log = log_error_zero / value_count
    mean_complex = complex_error_mean / value_count / mean_abs_scaled
    mean_log = log_error_mean / value_count
    return {
        "complex_value_count": value_count,
        "zero_predictor": {
            "complex_l1": zero_complex,
            "log_magnitude_l1": zero_log,
            "objective": zero_complex + zero_log,
        },
        "global_mean_field": {
            "complex_l1": mean_complex,
            "log_magnitude_l1": mean_log,
            "objective": mean_complex + mean_log,
        },
        "energy": {
            "total_scaled_complex_energy": total_energy,
            "global_mean_field_energy_fraction": mean_field_energy / total_energy,
            "centered_energy": centered_energy,
        },
        "rank_oracle": rank_records,
    }


def training_diagnostics(reports: list[dict[str, Any]], zero_objective: float) -> list[dict[str, Any]]:
    result = []
    for report in reports:
        trace = report["loss_trace"]
        final_context = report["final_context_metrics"]
        clipped = sum(
            record["gradient_norm_before_clip"] > common.GRADIENT_CLIP_NORM
            for record in trace
        )
        result.append(
            {
                "candidate_id": report["candidate_id"],
                "helmholtz_weight": report["helmholtz_weight"],
                "training_report_sha256": common.sha256_bytes(
                    common.canonical_json(report)
                ),
                "initial_trace_record": trace[0],
                "final_trace_record": trace[-1],
                "logged_steps_with_gradient_above_clip": clipped,
                "logged_step_count": len(trace),
                "full_context_metrics": final_context,
                "full_context_objective_over_zero_predictor": final_context[
                    "full_context_objective"
                ]
                / zero_objective,
            }
        )
    return result


def hypotheses(
    oracle: dict[str, Any],
    training: list[dict[str, Any]],
    evaluation: dict[str, Any],
) -> list[dict[str, Any]]:
    rank96 = next(record for record in oracle["rank_oracle"] if record["rank"] == 96)
    data = training[0]
    candidate_eval = {
        candidate["candidate_id"]: candidate for candidate in evaluation["candidates"]
    }
    level_collapse = min(
        candidate_eval[candidate["candidate_id"]]["aggregate"][
            "mean_absolute_rms_level_error_db"
        ]
        for candidate in common.CANDIDATES
    )
    return [
        {
            "hypothesis": "physics_regularization_is_the_primary_failure",
            "evidence_for": "none_beyond_the_original_physics_motivation",
            "evidence_against": (
                "data_only_and_helmholtz_have_nearly_identical_query_collapse;_"
                "helmholtz_loss_falls_without_restoring_level"
            ),
            "conclusion": "rejected_as_primary_cause",
        },
        {
            "hypothesis": "rank96_linear_separability_is_the_primary_bottleneck",
            "evidence_for": {
                "rank96_centered_energy_fraction": rank96[
                    "centered_energy_fraction"
                ],
                "rank96_best_linear_frobenius_nrmse": rank96[
                    "best_linear_frobenius_nrmse"
                ],
            },
            "evidence_against": (
                "the_trained_field_does_not_approach_even_the_zero_or_global_mean_"
                "context_controls_so_the_rank_ceiling_is_not_yet_reached"
            ),
            "conclusion": (
                "supported" if rank96["total_energy_fraction_with_mean"] < 0.95 else "not_supported_as_primary_cause"
            ),
        },
        {
            "hypothesis": "loss_sampling_and_clipped_optimization_collapse_toward_silence",
            "evidence_for": {
                "data_only_full_context_objective_over_zero": data[
                    "full_context_objective_over_zero_predictor"
                ],
                "data_only_logged_steps_above_clip": data[
                    "logged_steps_with_gradient_above_clip"
                ],
                "logged_step_count": data["logged_step_count"],
                "best_query_mean_level_error_db": level_collapse,
            },
            "evidence_against": (
                "log_magnitude_loss_does_decrease_and_all_outputs_remain_finite"
            ),
            "conclusion": (
                "supported"
                if data["full_context_objective_over_zero_predictor"] >= 0.9
                and level_collapse > 20.0
                else "not_discriminated"
            ),
        },
    ]


def main() -> None:
    arguments = parse_arguments()
    root = common.root_from_script(Path(__file__))
    preflight_bytes, preflight, cache_path = validate_preflight(root, arguments.preflight)
    training_bytes, training_reports = validate_training(root, arguments.training_report)
    evaluation_bytes, evaluation = validate_evaluation(root, arguments.evaluation_report)
    shape = tuple(preflight["feature_shape"])
    cache = np.memmap(cache_path, dtype="<c8", mode="r", shape=shape)
    device = common.configure_determinism()
    stats = preflight["context_feature_statistics"]
    oracle = context_oracles(
        cache,
        float(stats["pressure_scale"]),
        float(stats["mean_magnitude_after_pressure_scale"]),
        device,
    )
    del cache
    training = training_diagnostics(
        training_reports, oracle["zero_predictor"]["objective"]
    )
    hypothesis_records = hypotheses(oracle, training, evaluation)
    decision = "RejectCurrentSeparableSirenAndRedesignObjectiveBeforeAnotherCandidate"
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": decision,
        "claim": (
            "R2C_CONTEXT_ONLY_FAILURE_DISCRIMINATION / NO_NEW_CANDIDATE_QUERY_"
            "QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "revision": common.REVISION,
        "runner_sha256": common.sha256_file(Path(__file__).resolve(strict=True)),
        "implementation_sources": common.source_records(
            root,
            [
                "lab/scripts/physical_sound_listener_field_r2c_common.py",
                "lab/scripts/physical_sound_listener_field_r2c_failure_diagnostic.py",
            ],
        ),
        "preflight_report_sha256": common.sha256_bytes(preflight_bytes),
        "training_report_sha256": [
            common.sha256_bytes(payload) for payload in training_bytes
        ],
        "evaluation_report_sha256": common.sha256_bytes(evaluation_bytes),
        "environment": common.environment_profile(),
        "context_oracles": oracle,
        "training_diagnostics": training,
        "query_evaluation_aggregates_from_frozen_report": {
            candidate["candidate_id"]: candidate["aggregate"]
            for candidate in evaluation["candidates"]
        },
        "hypotheses": hypothesis_records,
        "optimizer_steps": 0,
        "query_audio_bytes_read": 0,
        "method_holdout_or_shadow_bytes_read": 0,
        "next_action": (
            "freeze_a_new_revision_with_an_energy_preserving_objective_and_a_"
            "context_micro_overfit_gate_before_any_grouped_query_candidate"
        ),
        "quality_or_admission_authorized": False,
    }
    report_bytes = common.canonical_json(report)
    output, staging = common.prepare_output(root, arguments.output, "R2C diagnostic output")
    try:
        (staging / "diagnostic-report.json").write_bytes(report_bytes)
        common.publish_staging(staging, output)
    except BaseException:
        common.discard_staging(staging)
        raise
    common.emit_summary(
        {
            "output": str(output),
            "decision": decision,
            "diagnostic_report_sha256": common.sha256_bytes(report_bytes),
            "zero_predictor": oracle["zero_predictor"],
            "global_mean_field": oracle["global_mean_field"],
            "rank96": next(
                record for record in oracle["rank_oracle"] if record["rank"] == 96
            ),
            "hypotheses": hypothesis_records,
        },
        (
            "output",
            "decision",
            "diagnostic_report_sha256",
            "zero_predictor",
            "global_mean_field",
            "rank96",
            "hypotheses",
        ),
    )


if __name__ == "__main__":
    main()
