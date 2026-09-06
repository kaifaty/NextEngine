#!/usr/bin/env python3
"""Freeze and execute the query-free R2D context trainability gate."""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import torch

import physical_sound_listener_field_r2d_common as common

r2c = common.r2c


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["freeze", "run"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--r2c-preflight", type=Path)
    parser.add_argument("--manifest", type=Path)
    return parser.parse_args()


def implementation_paths() -> list[str]:
    return [
        "lab/scripts/physical_sound_listener_field_r2b_dense_preflight.py",
        "lab/scripts/physical_sound_listener_field_r2c_common.py",
        "lab/scripts/physical_sound_listener_field_r2d_common.py",
        "lab/scripts/physical_sound_listener_field_r2d_trainability.py",
    ]


def build_low_rank_artifacts(
    cache: np.memmap,
    staging: Path,
    device: torch.device,
) -> tuple[dict[str, Any], dict[str, np.ndarray]]:
    row_count = cache.shape[0]
    flattened = cache.reshape(row_count, -1)
    total_tf = flattened.shape[1]
    mean_path = staging / "context-mean-field.c64le"
    mean_field = np.memmap(mean_path, dtype="<c8", mode="w+", shape=(total_tf,))
    gram = torch.zeros(
        (row_count, row_count), dtype=torch.complex64, device=device
    )
    total_energy = 0.0
    mean_energy_all_rows = 0.0
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            values_np = np.array(
                flattened[:, start:stop], dtype=np.complex64, copy=True
            )
            values = torch.from_numpy(values_np).to(device)
            mean = torch.mean(values, dim=0)
            centered = values - mean[None, :]
            gram += centered @ torch.conj(centered.T)
            mean_field[start:stop] = mean.cpu().numpy().astype("<c8")
            total_energy += float(torch.sum(torch.abs(values) ** 2).cpu())
            mean_energy_all_rows += float(
                row_count * torch.sum(torch.abs(mean) ** 2).cpu()
            )
    mean_field.flush()
    eigenvalues, eigenvectors = torch.linalg.eigh(gram)
    order = torch.argsort(eigenvalues.real, descending=True)
    eigenvalues = torch.clamp(eigenvalues.real[order], min=0.0)
    eigenvectors = common.canonicalize_eigenvectors(eigenvectors[:, order])
    selected_values = eigenvalues[: common.RANK]
    selected_vectors = eigenvectors[:, : common.RANK]
    if float(selected_values[-1].cpu()) <= 0.0:
        raise common.R2DError("R2D rank oracle contains a non-positive mode")
    singular = torch.sqrt(selected_values)
    coefficients = selected_vectors * singular[None, :]

    basis_path = staging / "context-basis-rank96.c64le"
    basis = np.memmap(
        basis_path,
        dtype="<c8",
        mode="w+",
        shape=(common.RANK, total_tf),
    )
    basis_gram = torch.zeros(
        (common.RANK, common.RANK), dtype=torch.complex64, device=device
    )
    mean_projection = torch.zeros(
        common.RANK, dtype=torch.complex64, device=device
    )
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            values_np = np.array(
                flattened[:, start:stop], dtype=np.complex64, copy=True
            )
            values = torch.from_numpy(values_np).to(device)
            mean = torch.from_numpy(
                np.array(mean_field[start:stop], dtype=np.complex64, copy=True)
            ).to(device)
            centered = values - mean[None, :]
            chunk = (
                torch.conj(selected_vectors.T) @ centered
            ) / singular[:, None]
            basis[:, start:stop] = chunk.cpu().numpy().astype("<c8")
            basis_gram += chunk @ torch.conj(chunk.T)
            mean_projection += chunk @ torch.conj(mean)
    basis.flush()
    correction_values, correction_vectors = torch.linalg.eigh(basis_gram)
    if torch.any(correction_values.real <= 0.0):
        raise common.R2DError("R2D streamed basis Gram is not positive definite")
    correction = (
        correction_vectors
        @ torch.diag(torch.rsqrt(correction_values.real).to(torch.complex64))
        @ torch.conj(correction_vectors.T)
    )
    correction_inverse = (
        correction_vectors
        @ torch.diag(torch.sqrt(correction_values.real).to(torch.complex64))
        @ torch.conj(correction_vectors.T)
    )
    coefficients = coefficients @ correction_inverse
    mean_projection = correction @ mean_projection
    basis_gram.zero_()
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            chunk = torch.from_numpy(
                np.array(
                    basis[:, start:stop], dtype=np.complex64, copy=True
                )
            ).to(device)
            chunk = correction @ chunk
            basis[:, start:stop] = chunk.cpu().numpy().astype("<c8")
            basis_gram += chunk @ torch.conj(chunk.T)
    basis.flush()
    identity = torch.eye(common.RANK, dtype=torch.complex64, device=device)
    orthonormal_max_error = float(torch.max(torch.abs(basis_gram - identity)).cpu())
    if orthonormal_max_error > 2.0e-4:
        raise common.R2DError(
            f"R2D streamed basis lost orthonormality: {orthonormal_max_error}"
        )

    coefficient_rms = torch.sqrt(torch.mean(torch.abs(coefficients) ** 2, dim=0))
    if not torch.isfinite(coefficient_rms).all() or torch.any(coefficient_rms <= 0):
        raise common.R2DError("R2D coefficient scale is invalid")
    coefficients_path = staging / "context-target-coefficients.c64le"
    coefficient_array = coefficients.cpu().numpy().astype("<c8")
    coefficients_path.write_bytes(coefficient_array.tobytes(order="C"))
    rms_path = staging / "context-coefficient-rms.f32le"
    rms_array = coefficient_rms.cpu().numpy().astype("<f4")
    rms_path.write_bytes(rms_array.tobytes(order="C"))
    projection_path = staging / "basis-mean-projection.c64le"
    projection_array = mean_projection.cpu().numpy().astype("<c8")
    projection_path.write_bytes(projection_array.tobytes(order="C"))

    retained_centered = float(torch.sum(selected_values).cpu())
    retained_total = mean_energy_all_rows + retained_centered
    total_fraction = retained_total / total_energy
    frobenius_nrmse = math.sqrt(max(total_energy - retained_total, 0.0) / total_energy)
    gates = common.profile()["gates"]
    if not (
        gates["rank96_total_energy_fraction_min"]
        <= total_fraction
        <= gates["rank96_total_energy_fraction_max"]
        and gates["rank96_frobenius_nrmse_min"]
        <= frobenius_nrmse
        <= gates["rank96_frobenius_nrmse_max"]
    ):
        raise common.R2DError("R2D rank-96 oracle drifted from frozen evidence")
    artifacts = {
        "mean_field": r2c.file_ref(mean_path, relative_to=staging),
        "basis": r2c.file_ref(basis_path, relative_to=staging),
        "target_coefficients": r2c.file_ref(
            coefficients_path, relative_to=staging
        ),
        "coefficient_rms": r2c.file_ref(rms_path, relative_to=staging),
        "basis_mean_projection": r2c.file_ref(
            projection_path, relative_to=staging
        ),
    }
    diagnostics = {
        "row_count": row_count,
        "time_frequency_value_count_per_row": total_tf,
        "total_complex_value_count": row_count * total_tf,
        "rank": common.RANK,
        "total_energy": total_energy,
        "global_mean_energy_fraction": mean_energy_all_rows / total_energy,
        "retained_total_energy_fraction": total_fraction,
        "best_linear_frobenius_nrmse": frobenius_nrmse,
        "basis_orthonormal_max_error": orthonormal_max_error,
        "smallest_retained_eigenvalue": float(selected_values[-1].cpu()),
        "largest_eigenvalue": float(selected_values[0].cpu()),
        "mean_field_energy": mean_energy_all_rows / row_count,
    }
    arrays = {
        "mean_field": mean_field,
        "basis": basis,
        "coefficients": coefficient_array,
        "coefficient_rms": rms_array,
        "mean_projection": projection_array,
    }
    return {"artifacts": artifacts, "diagnostics": diagnostics}, arrays


def build_frozen_controls(
    root: Path,
    staging: Path,
    cache: np.memmap,
    preflight: dict[str, Any],
    arrays: dict[str, np.ndarray],
    device: torch.device,
) -> dict[str, Any]:
    controls_root = staging / "controls"
    reference_root = controls_root / "references"
    oracle_root = controls_root / "oracle"
    metric_root = controls_root / "metrics"
    reference_root.mkdir(parents=True)
    oracle_root.mkdir()
    metric_root.mkdir()
    transform = r2c.r2b.ComplexTransform(r2c.r2b.SAMPLE_COUNT)
    total_tf = arrays["mean_field"].shape[0]
    probe_indices = np.asarray(common.ALL_PROBE_CACHE_INDICES, dtype=np.int64)
    oracle_spectra_path = controls_root / ".oracle-probes.c64le"
    oracle_spectra = np.memmap(
        oracle_spectra_path,
        dtype="<c8",
        mode="w+",
        shape=(len(probe_indices), total_tf),
    )
    probe_coefficients = torch.from_numpy(
        np.array(arrays["coefficients"][probe_indices], copy=True)
    ).to(device)
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            basis = torch.from_numpy(
                np.array(
                    arrays["basis"][:, start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            mean = torch.from_numpy(
                np.array(
                    arrays["mean_field"][start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            values = mean[None, :] + probe_coefficients @ basis
            oracle_spectra[:, start:stop] = values.cpu().numpy().astype("<c8")
    oracle_spectra.flush()

    zero_signal = np.zeros(r2c.r2b.SAMPLE_COUNT, dtype=np.float64)
    mean_signal = transform.synthesize(
        np.asarray(arrays["mean_field"]).reshape(
            transform.frame_count, r2c.r2b.FFT_LENGTH // 2 + 1
        )
    )
    zero_path = controls_root / "zero.wav"
    mean_path = controls_root / "global-mean.wav"
    zero_ref = common.write_wav(zero_path, zero_signal, "zero control", staging)
    mean_ref = common.write_wav(
        mean_path, mean_signal, "global mean control", staging
    )
    references = []
    oracle_records = []
    row_records = preflight["context_rows"]
    for probe_offset, cache_index in enumerate(common.ALL_PROBE_CACHE_INDICES):
        target_spectrum = np.asarray(cache[cache_index])
        target_signal = transform.synthesize(target_spectrum)
        oracle_spectrum = np.asarray(oracle_spectra[probe_offset]).reshape(
            transform.frame_count, r2c.r2b.FFT_LENGTH // 2 + 1
        )
        oracle_signal = transform.synthesize(oracle_spectrum)
        reference_path = reference_root / f"cache-{cache_index:04}.wav"
        oracle_path = oracle_root / f"cache-{cache_index:04}.wav"
        reference = common.write_wav(
            reference_path,
            target_signal,
            f"reference cache {cache_index}",
            staging,
        )
        oracle = common.write_wav(
            oracle_path,
            oracle_signal,
            f"oracle cache {cache_index}",
            staging,
        )
        row = row_records[cache_index]
        references.append(
            {
                "cache_index": cache_index,
                "row_index": row["row_index"],
                "listener_position_metres": row["listener_position_metres"],
                **reference,
            }
        )
        oracle_records.append({"cache_index": cache_index, **oracle})
    del oracle_spectra
    oracle_spectra_path.unlink()
    reference_by_index = {
        record["cache_index"]: staging / record["path"] for record in references
    }
    oracle_by_index = {
        record["cache_index"]: staging / record["path"] for record in oracle_records
    }

    def pairs(candidate_by_index: dict[int, Path]) -> list[dict[str, Any]]:
        return [
            {
                "cache_index": index,
                "candidate_path": str(candidate_by_index[index]),
                "reference_path": str(reference_by_index[index]),
            }
            for index in common.ALL_PROBE_CACHE_INDICES
        ]

    repeated_zero = {index: zero_path for index in common.ALL_PROBE_CACHE_INDICES}
    repeated_mean = {index: mean_path for index in common.ALL_PROBE_CACHE_INDICES}
    identity = common.run_rust_metrics(
        root, "identity", pairs(reference_by_index), metric_root
    )
    zero = common.run_rust_metrics(root, "zero", pairs(repeated_zero), metric_root)
    mean = common.run_rust_metrics(
        root, "global-mean", pairs(repeated_mean), metric_root
    )
    oracle = common.run_rust_metrics(
        root, "rank96-oracle", pairs(oracle_by_index), metric_root
    )
    return {
        "probe_cache_indices": list(common.ALL_PROBE_CACHE_INDICES),
        "references": references,
        "zero": {"wav": zero_ref, "metrics": zero},
        "global_mean": {"wav": mean_ref, "metrics": mean},
        "rank96_oracle": {"wavs": oracle_records, "metrics": oracle},
        "identity_metrics": identity,
        "query_audio_bytes_read": 0,
        "optimizer_steps": 0,
    }


def freeze(
    root: Path, preflight_argument: Path | None, output_argument: Path
) -> None:
    if preflight_argument is None:
        raise common.R2DError("freeze requires --r2c-preflight")
    preflight_path, preflight_bytes, preflight, cache_path = (
        common.validate_r2c_preflight(root, preflight_argument)
    )
    output, staging = r2c.prepare_output(root, output_argument, "R2D freeze output")
    try:
        device = common.configure_determinism()
        feature_shape = tuple(preflight["feature_shape"])
        cache = np.memmap(cache_path, dtype="<c8", mode="r", shape=feature_shape)
        factorization, arrays = build_low_rank_artifacts(cache, staging, device)
        controls = build_frozen_controls(
            root, staging, cache, preflight, arrays, device
        )
        manifest = {
            "schema": common.MANIFEST_SCHEMA,
            "revision": common.REVISION,
            "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": r2c.source_records(
                root, implementation_paths()
            ),
            "r2c_preflight_report": r2c.file_ref(preflight_path),
            "environment": r2c.environment_profile(),
            "profile": common.profile(),
            "factorization": factorization,
            "controls": controls,
            "mlflow_experiment": "nextengine-physical-sound-r2d-trainability",
        }
        manifest_bytes = r2c.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": common.FREEZE_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R2DContextTrainabilityProtocolFrozen",
            "claim": (
                "CONTEXT_ONLY_BASIS_CONTROLS_AND_TRAINABILITY_PROTOCOL / NO_"
                "OPTIMIZER_QUERY_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "r2c_preflight_report_sha256": r2c.sha256_bytes(preflight_bytes),
            "factorization_diagnostics": factorization["diagnostics"],
            "control_aggregates": {
                "identity": controls["identity_metrics"]["aggregate"],
                "zero": controls["zero"]["metrics"]["aggregate"],
                "global_mean": controls["global_mean"]["metrics"]["aggregate"],
                "rank96_oracle": controls["rank96_oracle"]["metrics"][
                    "aggregate"
                ],
            },
            "context_feature_cache_bytes_read": preflight[
                "context_feature_cache"
            ]["byte_count"],
            "optimizer_steps": 0,
            "query_audio_bytes_read": 0,
            "method_holdout_or_shadow_bytes_read": 0,
            "training_authorized": True,
            "quality_or_admission_authorized": False,
        }
        report_bytes = r2c.canonical_json(report)
        (staging / "freeze-report.json").write_bytes(report_bytes)
        del cache
        r2c.publish_staging(staging, output)
    except BaseException:
        r2c.discard_staging(staging)
        raise
    r2c.emit_summary(
        {
            "output": str(output),
            "decision": report["decision"],
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "retained_total_energy_fraction": factorization["diagnostics"][
                "retained_total_energy_fraction"
            ],
            "rank96_frobenius_nrmse": factorization["diagnostics"][
                "best_linear_frobenius_nrmse"
            ],
            "query_audio_bytes_read": 0,
        },
        (
            "output",
            "decision",
            "manifest_sha256",
            "retained_total_energy_fraction",
            "rank96_frobenius_nrmse",
            "query_audio_bytes_read",
        ),
    )


def validate_manifest(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    required = {
        "schema",
        "revision",
        "runner_sha256",
        "implementation_sources",
        "r2c_preflight_report",
        "environment",
        "profile",
        "factorization",
        "controls",
        "mlflow_experiment",
    }
    if set(manifest) != required:
        raise common.R2DError("R2D manifest fields changed")
    if (
        manifest.get("schema") != common.MANIFEST_SCHEMA
        or manifest.get("revision") != common.REVISION
        or manifest.get("runner_sha256")
        != r2c.sha256_file(Path(__file__).resolve(strict=True))
        or manifest.get("implementation_sources")
        != r2c.source_records(root, implementation_paths())
        or manifest.get("environment") != r2c.environment_profile()
        or manifest.get("profile") != common.profile()
        or manifest.get("mlflow_experiment")
        != "nextengine-physical-sound-r2d-trainability"
    ):
        raise common.R2DError("R2D manifest identity or environment changed")
    preflight_path = common.validate_file_ref(
        root,
        manifest_path.parent,
        manifest["r2c_preflight_report"],
        "R2C preflight report",
    )
    _, _, preflight, cache_path = common.validate_r2c_preflight(
        root, preflight_path
    )
    factorization = manifest.get("factorization")
    if (
        not isinstance(factorization, dict)
        or set(factorization) != {"artifacts", "diagnostics"}
        or factorization["diagnostics"].get("rank") != common.RANK
    ):
        raise common.R2DError("R2D factorization manifest changed")
    artifact_paths = {
        name: common.validate_file_ref(
            root,
            manifest_path.parent,
            reference,
            f"R2D {name}",
        )
        for name, reference in factorization["artifacts"].items()
    }
    controls = manifest.get("controls")
    if (
        not isinstance(controls, dict)
        or controls.get("probe_cache_indices")
        != list(common.ALL_PROBE_CACHE_INDICES)
        or controls.get("query_audio_bytes_read") != 0
        or controls.get("optimizer_steps") != 0
    ):
        raise common.R2DError("R2D frozen controls changed")
    references = {}
    for record in controls.get("references", []):
        index = record.get("cache_index")
        references[index] = common.validate_file_ref(
            root, manifest_path.parent, record, f"R2D reference {index}"
        )
    oracle_paths = {}
    for record in controls.get("rank96_oracle", {}).get("wavs", []):
        index = record.get("cache_index")
        oracle_paths[index] = common.validate_file_ref(
            root, manifest_path.parent, record, f"R2D oracle {index}"
        )
    zero_path = common.validate_file_ref(
        root,
        manifest_path.parent,
        controls["zero"]["wav"],
        "R2D zero WAV",
    )
    mean_path = common.validate_file_ref(
        root,
        manifest_path.parent,
        controls["global_mean"]["wav"],
        "R2D global mean WAV",
    )
    if set(references) != set(common.ALL_PROBE_CACHE_INDICES) or set(
        oracle_paths
    ) != set(common.ALL_PROBE_CACHE_INDICES):
        raise common.R2DError("R2D frozen probe WAV set changed")
    return {
        "preflight": preflight,
        "cache_path": cache_path,
        "artifact_paths": artifact_paths,
        "references": references,
        "oracle_paths": oracle_paths,
        "zero_path": zero_path,
        "mean_path": mean_path,
        "controls": controls,
    }


def train_task(
    task_profile: dict[str, Any],
    arrays: dict[str, np.ndarray],
    output: Path,
    device: torch.device,
) -> tuple[dict[str, Any], common.CoefficientTable]:
    indices = np.asarray(task_profile["cache_indices"], dtype=np.int64)
    component_rms = torch.from_numpy(
        np.array(arrays["coefficient_rms"], dtype=np.float32, copy=True)
    ).to(device)
    target_raw = np.array(arrays["coefficients"][indices], copy=True)
    target_normalized = target_raw / np.array(
        arrays["coefficient_rms"], dtype=np.float32, copy=True
    )[None, :]
    target = common.tensor_pairs(target_normalized, device)
    mean_projection = common.tensor_pairs(
        np.array(arrays["mean_projection"], copy=True), device
    )
    mean_energy = torch.tensor(
        float(np.sum(np.abs(arrays["mean_field"]) ** 2, dtype=np.float64)),
        dtype=torch.float32,
        device=device,
    )
    torch.manual_seed(common.SEED)
    torch.cuda.manual_seed_all(common.SEED)
    model = common.CoefficientTable(len(indices)).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=common.LEARNING_RATE,
        betas=common.ADAM_BETAS,
        eps=common.ADAM_EPSILON,
        weight_decay=common.WEIGHT_DECAY,
    )
    zero_objective, zero_parts = common.coefficient_objective(
        model(), target, component_rms, mean_energy, mean_projection
    )
    trace = []
    clip_count = 0
    for step in range(task_profile["steps"]):
        objective, parts = common.coefficient_objective(
            model(), target, component_rms, mean_energy, mean_projection
        )
        if not torch.isfinite(objective):
            raise common.R2DError(
                f"R2D objective became non-finite for {task_profile['task_id']}"
            )
        optimizer.zero_grad(set_to_none=True)
        objective.backward()
        gradient_norm = torch.nn.utils.clip_grad_norm_(
            model.parameters(), common.GRADIENT_CLIP_NORM
        )
        if float(gradient_norm.detach().cpu()) > common.GRADIENT_CLIP_NORM:
            clip_count += 1
        optimizer.step()
        if step == 0 or (step + 1) % common.LOG_INTERVAL == 0:
            trace.append(
                {
                    "step": step + 1,
                    "objective": float(objective.detach().cpu()),
                    "raw_coefficient_nmse": float(
                        parts["raw_coefficient_nmse"].detach().cpu()
                    ),
                    "whitened_coefficient_mse": float(
                        parts["whitened_coefficient_mse"].detach().cpu()
                    ),
                    "mean_absolute_log_energy_error": float(
                        parts["mean_absolute_log_energy_error"].detach().cpu()
                    ),
                    "gradient_norm_before_clip": float(
                        gradient_norm.detach().cpu()
                    ),
                }
            )
    final_objective, final_parts = common.coefficient_objective(
        model(), target, component_rms, mean_energy, mean_projection
    )
    task_directory = output / "checkpoints" / task_profile["task_id"]
    task_directory.mkdir(parents=True)
    weights = model.values.detach().cpu().numpy().astype("<f4")
    weights_path = task_directory / "normalized-coefficients.f32le"
    weights_path.write_bytes(weights.tobytes(order="C"))
    descriptor = {
        "schema": "nextengine.experimental-r2d-coefficient-table-checkpoint.v1",
        "revision": common.REVISION,
        "task_id": task_profile["task_id"],
        "shape": list(weights.shape),
        "dtype": "float32le",
        "weights_sha256": r2c.sha256_file(weights_path),
        "weights_byte_count": weights_path.stat().st_size,
    }
    descriptor_bytes = r2c.canonical_json(descriptor)
    descriptor_path = task_directory / "checkpoint.json"
    descriptor_path.write_bytes(descriptor_bytes)
    report = {
        "task_id": task_profile["task_id"],
        "context_row_count": len(indices),
        "cache_indices": indices.tolist(),
        "optimizer_steps": task_profile["steps"],
        "model_parameter_count": int(weights.size),
        "zero_predictor_metrics": {
            "objective": float(zero_objective.detach().cpu()),
            **{
                name: float(value.detach().cpu())
                for name, value in zero_parts.items()
            },
        },
        "loss_trace": trace,
        "gradient_clip_count": clip_count,
        "final_metrics": {
            "objective": float(final_objective.detach().cpu()),
            **{
                name: float(value.detach().cpu())
                for name, value in final_parts.items()
            },
        },
        "checkpoint": {
            "descriptor_file": str(descriptor_path.relative_to(output)),
            "descriptor_sha256": r2c.sha256_bytes(descriptor_bytes),
            "weights_file": str(weights_path.relative_to(output)),
            "weights_sha256": descriptor["weights_sha256"],
            "weights_byte_count": descriptor["weights_byte_count"],
        },
    }
    return report, model


def cook_trained_probes(
    task_profile: dict[str, Any],
    model: common.CoefficientTable,
    arrays: dict[str, np.ndarray],
    output: Path,
    device: torch.device,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    task_indices = list(task_profile["cache_indices"])
    local_by_global = {index: offset for offset, index in enumerate(task_indices)}
    probes = list(task_profile["probe_cache_indices"])
    normalized = model.values.detach()
    rms = torch.from_numpy(
        np.array(arrays["coefficient_rms"], dtype=np.float32, copy=True)
    ).to(device)
    raw_all = common.as_complex(normalized) * rms[None, :]
    raw = raw_all[[local_by_global[index] for index in probes]]
    total_tf = arrays["mean_field"].shape[0]
    temporary = output / f".{task_profile['task_id']}-spectra.c64le"
    spectra = np.memmap(
        temporary, dtype="<c8", mode="w+", shape=(len(probes), total_tf)
    )
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            basis = torch.from_numpy(
                np.array(
                    arrays["basis"][:, start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            mean = torch.from_numpy(
                np.array(
                    arrays["mean_field"][start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            spectra[:, start:stop] = (mean[None, :] + raw @ basis).cpu().numpy()
    spectra.flush()
    directory = output / "predictions" / task_profile["task_id"]
    directory.mkdir(parents=True)
    transform = r2c.r2b.ComplexTransform(r2c.r2b.SAMPLE_COUNT)
    records = []
    failures = []
    for offset, cache_index in enumerate(probes):
        spectrum = np.asarray(spectra[offset]).reshape(
            transform.frame_count, r2c.r2b.FFT_LENGTH // 2 + 1
        )
        signal = transform.synthesize(spectrum)
        peak = float(np.max(np.abs(signal)))
        if not np.isfinite(signal).all() or peak >= 1.0:
            failures.append(
                {
                    "cache_index": cache_index,
                    "reason": "nonfinite_or_peak_not_below_one",
                    "pre_cook_peak_abs": peak,
                }
            )
            continue
        path = directory / f"cache-{cache_index:04}.wav"
        payload = r2c.encode_wav(signal)
        path.write_bytes(payload)
        records.append(
            {
                "cache_index": cache_index,
                "path": str(path.relative_to(output)),
                "sha256": r2c.sha256_bytes(payload),
                "byte_count": len(payload),
                "pre_cook_peak_abs": peak,
            }
        )
    del spectra
    temporary.unlink()
    return records, failures


def run(
    root: Path, manifest_argument: Path | None, output_argument: Path
) -> None:
    if manifest_argument is None:
        raise common.R2DError("run requires --manifest")
    manifest_path = r2c.external_file(root, manifest_argument, "R2D manifest")
    manifest_bytes, manifest = r2c.read_json(manifest_path, "R2D manifest")
    inputs = validate_manifest(root, manifest_path, manifest)
    output, staging = r2c.prepare_output(root, output_argument, "R2D run output")
    try:
        device = common.configure_determinism()
        feature_shape = tuple(inputs["preflight"]["feature_shape"])
        total_tf = feature_shape[1] * feature_shape[2]
        arrays = common.load_factorization_arrays(
            inputs["artifact_paths"], total_tf
        )
        tracking = staging / "mlflow"
        artifacts = tracking / "artifacts"
        artifacts.mkdir(parents=True)
        tracking_uri = f"sqlite:///{tracking / 'mlflow.db'}"
        mlflow.set_tracking_uri(tracking_uri)
        client = mlflow.tracking.MlflowClient(tracking_uri=tracking_uri)
        experiment = client.get_experiment_by_name(manifest["mlflow_experiment"])
        experiment_id = (
            client.create_experiment(
                manifest["mlflow_experiment"], artifact_location=artifacts.as_uri()
            )
            if experiment is None
            else experiment.experiment_id
        )
        task_reports = []
        with mlflow.start_run(
            experiment_id=experiment_id, run_name="r2d-context-trainability"
        ) as active_run:
            mlflow.log_params(
                {
                    "revision": common.REVISION,
                    "seed": common.SEED,
                    "rank": common.RANK,
                    "learning_rate": common.LEARNING_RATE,
                    "gradient_clip_norm": common.GRADIENT_CLIP_NORM,
                    "query_audio_bytes_read": 0,
                }
            )
            for task_profile in common.TASKS:
                training, model = train_task(
                    task_profile, arrays, staging, device
                )
                predictions, failures = cook_trained_probes(
                    task_profile, model, arrays, staging, device
                )
                probes = list(task_profile["probe_cache_indices"])
                if failures or len(predictions) != len(probes):
                    task_reports.append(
                        {
                            **training,
                            "predictions": predictions,
                            "cook_failures": failures,
                            "passes_trainability_gate": False,
                            "gate_evidence": {"checks": {"cook": False}},
                        }
                    )
                    continue
                prediction_by_index = {
                    record["cache_index"]: staging / record["path"]
                    for record in predictions
                }
                pairs = [
                    {
                        "cache_index": index,
                        "candidate_path": str(prediction_by_index[index]),
                        "reference_path": str(inputs["references"][index]),
                    }
                    for index in probes
                ]
                metrics_root = staging / "metrics"
                metrics_root.mkdir(exist_ok=True)
                trained_metrics = common.run_rust_metrics(
                    root, task_profile["task_id"], pairs, metrics_root
                )
                zero = common.subset_metrics(
                    inputs["controls"]["zero"]["metrics"], probes
                )
                mean = common.subset_metrics(
                    inputs["controls"]["global_mean"]["metrics"], probes
                )
                oracle = common.subset_metrics(
                    inputs["controls"]["rank96_oracle"]["metrics"], probes
                )
                passed, gate_evidence = common.task_metrics_pass(
                    training, trained_metrics, oracle, zero, mean
                )
                task_report = {
                    **training,
                    "predictions": predictions,
                    "cook_failures": [],
                    "trained_metrics": trained_metrics,
                    "frozen_control_metrics": {
                        "zero": zero,
                        "global_mean": mean,
                        "rank96_oracle": oracle,
                    },
                    "passes_trainability_gate": passed,
                    "gate_evidence": gate_evidence,
                }
                task_reports.append(task_report)
                final = training["final_metrics"]
                for name, value in final.items():
                    mlflow.log_metric(f"{task_profile['task_id']}.{name}", value)
                mlflow.log_metric(
                    f"{task_profile['task_id']}.passes", int(passed)
                )
            all_pass = len(task_reports) == len(common.TASKS) and all(
                report["passes_trainability_gate"] for report in task_reports
            )
            decision = (
                "R2DTrainabilityGatePass"
                if all_pass
                else "RejectTrainingSubstrate"
            )
            report = {
                "schema": common.RUN_REPORT_SCHEMA,
                "status": "Validated",
                "decision": decision,
                "claim": (
                    "CONTEXT_ONLY_OBJECTIVE_OPTIMIZER_AND_COOKER_TRAINABILITY / "
                    "NO_SPATIAL_QUERY_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
                ),
                "revision": common.REVISION,
                "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
                "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
                "implementation_sources": manifest["implementation_sources"],
                "environment": r2c.environment_profile(),
                "profile": common.profile(),
                "factorization_diagnostics": manifest["factorization"][
                    "diagnostics"
                ],
                "tasks": task_reports,
                "all_tasks_pass": all_pass,
                "n0_3e_authorized": all_pass,
                "optimizer_steps": sum(
                    task_profile["steps"] for task_profile in common.TASKS
                ),
                "context_feature_cache_bytes_read": 0,
                "frozen_context_factorization_bytes_read": sum(
                    reference.get("byte_count", 0)
                    for reference in manifest["factorization"][
                        "artifacts"
                    ].values()
                ),
                "query_audio_bytes_read": 0,
                "query_audio_rows_read": 0,
                "method_holdout_or_shadow_bytes_read": 0,
                "quality_or_admission_authorized": False,
                "next_action": (
                    "freeze_one_context_basis_coordinate_to_coefficient_r2e_"
                    "candidate_without_query_feedback"
                    if all_pass
                    else "redesign_one_context_training_substrate_hypothesis",
                ),
            }
            report_bytes = r2c.canonical_json(report)
            report_path = staging / "run-report.json"
            report_path.write_bytes(report_bytes)
            mlflow.log_artifact(str(report_path))
            run_id = active_run.info.run_id
            experiment_id_value = active_run.info.experiment_id
        lineage = {
            "schema": common.MLFLOW_SCHEMA,
            "mlflow_version": mlflow.__version__,
            "tracking_database": "mlflow/mlflow.db",
            "artifact_directory": "mlflow/artifacts",
            "experiment_name": manifest["mlflow_experiment"],
            "run_id": run_id,
            "experiment_id": experiment_id_value,
            "deterministic_run_report_sha256": r2c.sha256_bytes(report_bytes),
            "checkpoint_weights_sha256": {
                task_report["task_id"]: task_report["checkpoint"][
                    "weights_sha256"
                ]
                for task_report in task_reports
            },
        }
        (staging / "mlflow-lineage.json").write_bytes(r2c.canonical_json(lineage))
        r2c.publish_staging(staging, output)
    except BaseException:
        r2c.discard_staging(staging)
        raise
    r2c.emit_summary(
        {
            "output": str(output),
            "decision": report["decision"],
            "run_report_sha256": r2c.sha256_bytes(report_bytes),
            "task_results": [
                {
                    "task_id": value["task_id"],
                    "passes": value["passes_trainability_gate"],
                    "final_metrics": value["final_metrics"],
                }
                for value in task_reports
            ],
            "query_audio_bytes_read": 0,
        },
        (
            "output",
            "decision",
            "run_report_sha256",
            "task_results",
            "query_audio_bytes_read",
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    root = r2c.root_from_script(Path(__file__))
    if arguments.stage == "freeze":
        if arguments.manifest is not None:
            raise common.R2DError("freeze accepts no --manifest")
        freeze(root, arguments.r2c_preflight, arguments.output)
    else:
        if arguments.r2c_preflight is not None:
            raise common.R2DError("run accepts no --r2c-preflight")
        run(root, arguments.manifest, arguments.output)


if __name__ == "__main__":
    main()
