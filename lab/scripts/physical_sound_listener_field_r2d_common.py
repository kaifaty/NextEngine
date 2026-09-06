#!/usr/bin/env python3
"""Shared frozen R2D context-trainability protocol and utilities."""

from __future__ import annotations

import math
import subprocess
import sys
from pathlib import Path
from typing import Any

import numpy as np
import torch
from torch import nn

import physical_sound_listener_field_r2c_common as r2c

MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d.manifest.v1"
)
FREEZE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-freeze.report.v1"
)
RUN_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-run.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-mlflow-lineage.v1"
)
REVISION = "green-goblet-context-trainability-r2d-v1"
R2C_PREFLIGHT_REPORT_SHA256 = (
    "8ed8d0764003ed21f2f2a75be3dc035f9d313d2e476c9adb6aaac707e1f1b233"
)
CONTEXT_CACHE_SHA256 = (
    "842a7c922e08abe20c8498eb8b44849781dbf11a94380737e7daa6ce6d5cc787"
)
RANK = 96
TF_CHUNK = 8_192
SEED = 2_608_300_204
LEARNING_RATE = 5.0e-2
ADAM_BETAS = (0.9, 0.99)
ADAM_EPSILON = 1.0e-8
WEIGHT_DECAY = 0.0
GRADIENT_CLIP_NORM = 10.0
LOG_INTERVAL = 100
LOSS_EPSILON = 1.0e-12
RAW_COEFFICIENT_WEIGHT = 0.5
WHITENED_COEFFICIENT_WEIGHT = 0.5
LOG_ENERGY_WEIGHT = 0.25
ONE_ROW_CACHE_INDICES = (0,)
SMALL_BLOCK_CACHE_INDICES = (0, 59, 60, 119, 300, 359, 360, 419)
FULL_PROBE_CACHE_INDICES = tuple(
    angle_group * 60 + distance_group * 15 + 7
    for angle_group in range(7)
    for distance_group in range(4)
)
ALL_PROBE_CACHE_INDICES = tuple(
    sorted(
        set(ONE_ROW_CACHE_INDICES)
        | set(SMALL_BLOCK_CACHE_INDICES)
        | set(FULL_PROBE_CACHE_INDICES)
    )
)
TASKS = (
    {
        "task_id": "one_row_micro_overfit",
        "cache_indices": ONE_ROW_CACHE_INDICES,
        "steps": 800,
        "probe_cache_indices": ONE_ROW_CACHE_INDICES,
    },
    {
        "task_id": "small_block_micro_overfit",
        "cache_indices": SMALL_BLOCK_CACHE_INDICES,
        "steps": 1_200,
        "probe_cache_indices": SMALL_BLOCK_CACHE_INDICES,
    },
    {
        "task_id": "full_context_coefficient_fit",
        "cache_indices": tuple(range(r2c.r2b.CONTEXT_ROWS)),
        "steps": 1_600,
        "probe_cache_indices": FULL_PROBE_CACHE_INDICES,
    },
)


class R2DError(RuntimeError):
    """The frozen R2D trainability boundary failed closed."""


def profile() -> dict[str, Any]:
    return {
        "profile_id": "context-low-rank-coefficient-trainability-rank96-v1",
        "revision": REVISION,
        "seed": SEED,
        "representation": {
            "source": "r2c_context_complex_stft_cache_only",
            "centering": "global_complex_mean_over_420_context_rows",
            "factorization": "exact_context_gram_eigh_then_streamed_basis",
            "rank": RANK,
            "basis_phase": "largest_magnitude_left_vector_element_real_positive",
            "query_audio_used": False,
        },
        "model": {
            "family": "direct_trainable_normalized_complex_coefficient_table",
            "initialization": "all_zero_coefficients_equal_global_mean_field",
            "purpose": "objective_optimizer_and_cooker_gate_not_spatial_model",
        },
        "optimizer": {
            "name": "torch_adamw_fixed_steps",
            "learning_rate": LEARNING_RATE,
            "betas": list(ADAM_BETAS),
            "epsilon": ADAM_EPSILON,
            "weight_decay": WEIGHT_DECAY,
            "gradient_clip_norm": GRADIENT_CLIP_NORM,
            "tasks": [
                {
                    "task_id": task["task_id"],
                    "row_count": len(task["cache_indices"]),
                    "steps": task["steps"],
                    "probe_count": len(task["probe_cache_indices"]),
                }
                for task in TASKS
            ],
        },
        "objective": {
            "raw_energy_weighted_complex_coefficient_nmse_weight": (
                RAW_COEFFICIENT_WEIGHT
            ),
            "component_whitened_complex_coefficient_mse_weight": (
                WHITENED_COEFFICIENT_WEIGHT
            ),
            "exact_reconstructed_row_log_energy_mse_weight": LOG_ENERGY_WEIGHT,
            "component_scale": "full_context_coefficient_complex_rms",
            "epsilon": LOSS_EPSILON,
        },
        "controls": {
            "identity": "context_complex_stft_inverse_to_pcm",
            "zero": "all_complex_stft_bins_zero",
            "global_mean": "context_global_complex_mean_field",
            "rank_oracle": "context_only_rank96_linear_reconstruction",
            "probe_cache_indices": list(ALL_PROBE_CACHE_INDICES),
        },
        "gates": {
            "rank96_total_energy_fraction_min": 0.9963,
            "rank96_total_energy_fraction_max": 0.9965,
            "rank96_frobenius_nrmse_min": 0.0599,
            "rank96_frobenius_nrmse_max": 0.0602,
            "normalized_coefficient_rmse_max": 5.0e-3,
            "raw_coefficient_nmse_max": 1.0e-4,
            "mean_absolute_log_energy_error_max": 5.0e-3,
            "objective_over_zero_max": 1.0e-2,
            "gradient_clip_fraction_max": 1.0e-2,
            "trained_to_oracle_aggregate_delta_db_max": 0.15,
            "trained_strictly_better_than_zero_and_mean_all_endpoints": True,
            "finite_and_peak_below_one": True,
            "two_runs": "byte_identical_excluding_mlflow_state_and_run_id",
        },
        "selection": (
            "all_three_context_tasks_and_cooker_comparisons_pass_else_"
            "reject_training_substrate_without_query_audio"
        ),
        "query_audio_available": False,
        "method_holdout_or_shadow_available": False,
    }


class CoefficientTable(nn.Module):
    """Direct coefficient parameters used only to discriminate trainability."""

    def __init__(self, row_count: int, rank: int = RANK) -> None:
        super().__init__()
        self.values = nn.Parameter(torch.zeros((row_count, rank, 2)))

    def forward(self) -> torch.Tensor:
        return self.values


def as_complex(values: torch.Tensor) -> torch.Tensor:
    if values.shape[-1] != 2:
        raise R2DError("complex tensor requires a final real/imaginary axis")
    return torch.complex(values[..., 0], values[..., 1])


def reconstructed_energy(
    normalized_coefficients: torch.Tensor,
    component_rms: torch.Tensor,
    mean_energy: torch.Tensor,
    basis_mean_projection: torch.Tensor,
) -> torch.Tensor:
    coefficients = as_complex(normalized_coefficients) * component_rms[None, :]
    projection = as_complex(basis_mean_projection)
    energy = (
        mean_energy
        + torch.sum(torch.abs(coefficients) ** 2, dim=1)
        + 2.0 * torch.real(torch.sum(coefficients * projection[None, :], dim=1))
    )
    return torch.clamp(energy, min=LOSS_EPSILON)


def coefficient_objective(
    prediction: torch.Tensor,
    target: torch.Tensor,
    component_rms: torch.Tensor,
    mean_energy: torch.Tensor,
    basis_mean_projection: torch.Tensor,
) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    prediction_complex = as_complex(prediction)
    target_complex = as_complex(target)
    difference = prediction_complex - target_complex
    raw_difference = difference * component_rms[None, :]
    raw_target = target_complex * component_rms[None, :]
    raw_nmse = torch.sum(torch.abs(raw_difference) ** 2) / torch.clamp(
        torch.sum(torch.abs(raw_target) ** 2), min=LOSS_EPSILON
    )
    whitened_mse = torch.mean(torch.abs(difference) ** 2)
    prediction_energy = reconstructed_energy(
        prediction,
        component_rms,
        mean_energy,
        basis_mean_projection,
    )
    target_energy = reconstructed_energy(
        target,
        component_rms,
        mean_energy,
        basis_mean_projection,
    )
    log_energy_difference = torch.log(prediction_energy) - torch.log(target_energy)
    log_energy_mse = torch.mean(log_energy_difference * log_energy_difference)
    log_energy_l1 = torch.mean(torch.abs(log_energy_difference))
    objective = (
        RAW_COEFFICIENT_WEIGHT * raw_nmse
        + WHITENED_COEFFICIENT_WEIGHT * whitened_mse
        + LOG_ENERGY_WEIGHT * log_energy_mse
    )
    return objective, {
        "raw_coefficient_nmse": raw_nmse,
        "whitened_coefficient_mse": whitened_mse,
        "mean_squared_log_energy_error": log_energy_mse,
        "mean_absolute_log_energy_error": log_energy_l1,
    }


def canonicalize_eigenvectors(vectors: torch.Tensor) -> torch.Tensor:
    result = vectors.clone()
    for column in range(result.shape[1]):
        values = result[:, column]
        pivot = values[torch.argmax(torch.abs(values))]
        if float(torch.abs(pivot).cpu()) <= 0.0:
            raise R2DError("rank basis contains an empty eigenvector")
        phase = torch.conj(pivot) / torch.abs(pivot)
        result[:, column] = values * phase
    return result


def configure_determinism() -> torch.device:
    return r2c.configure_determinism()


def validate_r2c_preflight(
    root: Path, argument: Path
) -> tuple[Path, bytes, dict[str, Any], Path]:
    path = r2c.external_file(root, argument, "R2C preflight report")
    data, report = r2c.read_json(path, "R2C preflight report")
    if r2c.sha256_bytes(data) != R2C_PREFLIGHT_REPORT_SHA256:
        raise R2DError("R2C preflight report identity changed")
    if (
        report.get("schema") != r2c.PREFLIGHT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "R2CContextFeaturesReadyForFrozenTraining"
        or report.get("feature_shape")
        != [
            r2c.r2b.CONTEXT_ROWS,
            r2c.r2b.ComplexTransform(r2c.r2b.SAMPLE_COUNT).frame_count,
            r2c.r2b.FFT_LENGTH // 2 + 1,
        ]
        or report.get("query_audio_bytes_read") != 0
        or report.get("query_rows_cached_for_fit") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
        or report.get("optimizer_steps") != 0
    ):
        raise R2DError("R2C preflight semantics changed")
    reference = r2c.require_ref(report.get("context_feature_cache"), "context cache")
    if reference.get("sha256") != CONTEXT_CACHE_SHA256:
        raise R2DError("R2C context cache identity changed")
    cache_path = r2c.external_file(
        root, path.parent / reference["path"], "R2C context cache"
    )
    if (
        cache_path.stat().st_size != reference.get("byte_count")
        or r2c.sha256_file(cache_path) != CONTEXT_CACHE_SHA256
    ):
        raise R2DError("R2C context cache payload changed")
    return path, data, report, cache_path


def validate_file_ref(
    root: Path, directory: Path, value: Any, label: str
) -> Path:
    if isinstance(value, dict) and {"path", "sha256", "byte_count"}.issubset(value):
        value = {
            "path": value["path"],
            "sha256": value["sha256"],
            "byte_count": value["byte_count"],
        }
    path, _ = r2c.resolve_ref(root, directory, value, label)
    return path


def load_factorization_arrays(
    paths: dict[str, Path], total_tf: int
) -> dict[str, np.ndarray]:
    return {
        "mean_field": np.memmap(
            paths["mean_field"], dtype="<c8", mode="r", shape=(total_tf,)
        ),
        "basis": np.memmap(
            paths["basis"], dtype="<c8", mode="r", shape=(RANK, total_tf)
        ),
        "coefficients": np.memmap(
            paths["target_coefficients"],
            dtype="<c8",
            mode="r",
            shape=(r2c.r2b.CONTEXT_ROWS, RANK),
        ),
        "coefficient_rms": np.fromfile(paths["coefficient_rms"], dtype="<f4"),
        "mean_projection": np.fromfile(
            paths["basis_mean_projection"], dtype="<c8"
        ),
    }


def task(task_id: str) -> dict[str, Any]:
    for value in TASKS:
        if value["task_id"] == task_id:
            return value
    raise R2DError(f"unknown R2D task: {task_id}")


def aggregate(rows: list[dict[str, Any]]) -> dict[str, float]:
    return r2c.aggregate_metric_rows(rows)


def write_wav(
    path: Path, signal: np.ndarray, label: str, relative_to: Path
) -> dict[str, Any]:
    peak = float(np.max(np.abs(signal)))
    if not np.isfinite(signal).all() or peak >= 1.0:
        raise R2DError(f"{label} is not a finite cookable signal: {peak}")
    payload = r2c.encode_wav(signal)
    path.write_bytes(payload)
    return {
        **r2c.file_ref(path, relative_to=relative_to),
        "pre_cook_peak_abs": peak,
    }


def tensor_pairs(values: np.ndarray, device: torch.device) -> torch.Tensor:
    array = np.stack((values.real, values.imag), axis=-1).astype(np.float32)
    return torch.from_numpy(array).to(device)


def subset_metrics(metrics: dict[str, Any], indices: list[int]) -> dict[str, Any]:
    selected_indices = set(indices)
    selected = [
        row for row in metrics["rows"] if row["cache_index"] in selected_indices
    ]
    selected.sort(key=lambda row: row["cache_index"])
    if [row["cache_index"] for row in selected] != sorted(indices):
        raise R2DError("R2D frozen control subset changed")
    return {
        "comparison_id": metrics["comparison_id"],
        "row_count": len(selected),
        "normalized_metric_rows_sha256": r2c.sha256_bytes(
            r2c.canonical_json(selected)
        ),
        "aggregate": aggregate(selected),
        "rows": selected,
    }


def run_rust_metrics(
    root: Path,
    comparison_id: str,
    pairs: list[dict[str, Any]],
    work: Path,
) -> dict[str, Any]:
    entries = []
    paths: dict[str, tuple[Path, Path]] = {}
    for pair in pairs:
        entry_id = f"cache-{pair['cache_index']:04}"
        candidate = Path(pair["candidate_path"])
        reference = Path(pair["reference_path"])
        entries.append(
            {
                "id": entry_id,
                "object_id": "realimpact-green-goblet",
                "material": "glass-vessel-published-label",
                "impact_position": "fixed-mesh-vertex-31676",
                "force_band": "force-deconvolved-transfer",
                "expected_signal": "impact",
                "candidate": {
                    "path": str(candidate),
                    "sha256": r2c.sha256_file(candidate),
                },
                "reference": {
                    "path": str(reference),
                    "sha256": r2c.sha256_file(reference),
                },
            }
        )
        paths[entry_id] = (candidate, reference)
    manifest = {
        "schema": r2c.r2b.RUST_MANIFEST_SCHEMA,
        "split": f"r2d-{comparison_id}",
        "relations": [],
        "entries": entries,
    }
    manifest_bytes = r2c.canonical_json(manifest)
    manifest_path = work / f"{comparison_id}.manifest.json"
    manifest_path.write_bytes(manifest_bytes)
    output = work / f"{comparison_id}.rust-eval"
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "physical-sound-eval",
            "--manifest",
            str(manifest_path),
            "--output",
            str(output),
        ],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise R2DError(
            f"Rust metrics failed for {comparison_id}: "
            f"{completed.stdout}\n{completed.stderr}"
        )
    _, report = r2c.read_json(output / "report.json", "R2D Rust metric report")
    if report.get("schema") != r2c.r2b.RUST_REPORT_SCHEMA:
        raise R2DError("R2D Rust metric schema changed")
    rows = []
    for entry in report.get("entries", []):
        entry_id = entry.get("id")
        matched = entry.get("matched")
        if entry_id not in paths or not isinstance(matched, dict):
            raise R2DError("R2D Rust metric report omitted a matched row")
        candidate, reference = paths[entry_id]
        rows.append(
            {
                "cache_index": int(entry_id.rsplit("-", 1)[1]),
                "absolute_rms_level_error_db": abs(matched["raw_rms_delta_db"]),
                "gain_matched_multiresolution_log_spectrum_rmse_db": matched[
                    "gain_matched_multiresolution_log_spectrum_rmse_db"
                ],
                "normalized_waveform_rmse_db": r2c.r2b.nrmse_db(
                    r2c.r2b.read_pcm16(candidate),
                    r2c.r2b.read_pcm16(reference),
                ),
            }
        )
    rows.sort(key=lambda row: row["cache_index"])
    if len(rows) != len(pairs):
        raise R2DError("R2D Rust metric row count changed")
    return {
        "comparison_id": comparison_id,
        "row_count": len(rows),
        "normalized_metric_rows_sha256": r2c.sha256_bytes(
            r2c.canonical_json(rows)
        ),
        "aggregate": aggregate(rows),
        "rows": rows,
    }


def task_metrics_pass(
    training: dict[str, Any],
    trained: dict[str, Any],
    oracle: dict[str, Any],
    zero: dict[str, Any],
    mean: dict[str, Any],
) -> tuple[bool, dict[str, Any]]:
    gates = profile()["gates"]
    final = training["final_metrics"]
    numeric = {
        "normalized_coefficient_rmse": math.sqrt(
            final["whitened_coefficient_mse"]
        ),
        "raw_coefficient_nmse": final["raw_coefficient_nmse"],
        "mean_absolute_log_energy_error": final[
            "mean_absolute_log_energy_error"
        ],
        "objective_over_zero": final["objective"]
        / training["zero_predictor_metrics"]["objective"],
        "gradient_clip_fraction": training["gradient_clip_count"]
        / training["optimizer_steps"],
    }
    aggregate_delta = {
        endpoint: abs(trained["aggregate"][endpoint] - oracle["aggregate"][endpoint])
        for endpoint in r2c.PRIMARY_ENDPOINTS
    }
    strictly_better = {
        endpoint: (
            trained["aggregate"][endpoint] < zero["aggregate"][endpoint]
            and trained["aggregate"][endpoint] < mean["aggregate"][endpoint]
        )
        for endpoint in r2c.PRIMARY_ENDPOINTS
    }
    checks = {
        "normalized_coefficient_rmse": numeric["normalized_coefficient_rmse"]
        <= gates["normalized_coefficient_rmse_max"],
        "raw_coefficient_nmse": numeric["raw_coefficient_nmse"]
        <= gates["raw_coefficient_nmse_max"],
        "mean_absolute_log_energy_error": numeric[
            "mean_absolute_log_energy_error"
        ]
        <= gates["mean_absolute_log_energy_error_max"],
        "objective_over_zero": numeric["objective_over_zero"]
        <= gates["objective_over_zero_max"],
        "gradient_clip_fraction": numeric["gradient_clip_fraction"]
        <= gates["gradient_clip_fraction_max"],
        "trained_to_oracle_aggregate_delta": all(
            value <= gates["trained_to_oracle_aggregate_delta_db_max"]
            for value in aggregate_delta.values()
        ),
        "trained_strictly_better_than_zero_and_mean": all(
            strictly_better.values()
        ),
    }
    return all(checks.values()), {
        "numeric": numeric,
        "trained_to_oracle_aggregate_delta_db": aggregate_delta,
        "strictly_better_than_zero_and_mean": strictly_better,
        "checks": checks,
    }


if __name__ == "__main__":
    sys.exit("physical_sound_listener_field_r2d_common.py is a library module")
