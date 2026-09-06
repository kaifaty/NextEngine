#!/usr/bin/env python3
"""Freeze, preflight, and train the R2C dense complex acoustic field."""

from __future__ import annotations

import argparse
import math
import os
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import torch

import physical_sound_listener_field_r2c_common as common


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["freeze", "preflight", "train"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--acquisition-manifest", type=Path)
    parser.add_argument("--r2b-report", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--preflight", type=Path)
    parser.add_argument(
        "--candidate",
        choices=[candidate["candidate_id"] for candidate in common.CANDIDATES],
    )
    return parser.parse_args()


def implementation_paths() -> list[str]:
    return [
        "lab/scripts/physical_sound_listener_field_r2b_dense_preflight.py",
        "lab/scripts/physical_sound_listener_field_r2c.py",
        "lab/scripts/physical_sound_listener_field_r2c_common.py",
    ]


def validate_implementation_sources(root: Path, sources: Any) -> list[dict[str, Any]]:
    expected = common.source_records(root, implementation_paths())
    if sources != expected:
        raise common.R2CError("R2C implementation source lineage changed")
    return expected


def freeze_manifest(
    root: Path,
    acquisition_argument: Path | None,
    r2b_argument: Path | None,
    output_argument: Path,
) -> None:
    if acquisition_argument is None or r2b_argument is None:
        raise common.R2CError("freeze requires --acquisition-manifest and --r2b-report")
    acquisition_path = common.external_file(
        root, acquisition_argument, "R2C acquisition manifest"
    )
    r2b_path = common.external_file(root, r2b_argument, "R2B preflight report")
    acquisition_bytes, acquisition = common.read_json(
        acquisition_path, "R2C acquisition manifest"
    )
    r2b_bytes, r2b_report = common.read_json(r2b_path, "R2B preflight report")
    common.validate_acquisition_manifest(
        root, acquisition_path, acquisition_bytes, acquisition
    )
    common.validate_r2b_report(acquisition_bytes, r2b_bytes, r2b_report)
    output, staging = common.prepare_output(root, output_argument, "R2C freeze output")
    try:
        runner = Path(__file__).resolve(strict=True)
        manifest = {
            "schema": common.MANIFEST_SCHEMA,
            "revision": common.REVISION,
            "runner_sha256": common.sha256_file(runner),
            "implementation_sources": common.source_records(root, implementation_paths()),
            "acquisition_manifest": common.file_ref(acquisition_path),
            "r2b_preflight_report": common.file_ref(r2b_path),
            "environment": common.environment_profile(),
            "candidate_profile": common.candidate_profile(),
            "mlflow_experiment": "nextengine-physical-sound-r2c-complex-field",
        }
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        freeze_report = {
            "schema": "nextengine.experimental-physical-sound-listener-field-r2c-freeze.report.v1",
            "status": "Validated",
            "decision": "R2CTrainingProtocolFrozen",
            "claim": (
                "DENSE_COMPLEX_FIELD_TRAINING_PROTOCOL_ONLY / NO_OPTIMIZER_"
                "QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "acquisition_manifest_sha256": common.sha256_bytes(acquisition_bytes),
            "r2b_preflight_report_sha256": common.sha256_bytes(r2b_bytes),
            "candidate_profile": common.candidate_profile(),
            "environment": common.environment_profile(),
            "optimizer_steps": 0,
            "query_audio_bytes_read": 0,
            "method_holdout_or_shadow_bytes_read": 0,
            "training_authorized": False,
        }
        freeze_bytes = common.canonical_json(freeze_report)
        (staging / "freeze-report.json").write_bytes(freeze_bytes)
        common.publish_staging(staging, output)
    except BaseException:
        common.discard_staging(staging)
        raise
    common.emit_summary(
        {
            "output": str(output),
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "decision": freeze_report["decision"],
        },
        ("output", "manifest_sha256", "decision"),
    )


def validate_manifest(
    root: Path, path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    expected = {
        "schema",
        "revision",
        "runner_sha256",
        "implementation_sources",
        "acquisition_manifest",
        "r2b_preflight_report",
        "environment",
        "candidate_profile",
        "mlflow_experiment",
    }
    if set(manifest) != expected:
        raise common.R2CError("R2C manifest fields changed")
    if (
        manifest.get("schema") != common.MANIFEST_SCHEMA
        or manifest.get("revision") != common.REVISION
        or manifest.get("runner_sha256")
        != common.sha256_file(Path(__file__).resolve(strict=True))
        or manifest.get("environment") != common.environment_profile()
        or manifest.get("candidate_profile") != common.candidate_profile()
        or manifest.get("mlflow_experiment")
        != "nextengine-physical-sound-r2c-complex-field"
    ):
        raise common.R2CError("R2C manifest identity or environment changed")
    sources = validate_implementation_sources(root, manifest["implementation_sources"])
    acquisition_path, acquisition_bytes = common.resolve_ref(
        root, path.parent, manifest["acquisition_manifest"], "R2C acquisition manifest"
    )
    r2b_path, r2b_bytes = common.resolve_ref(
        root, path.parent, manifest["r2b_preflight_report"], "R2B preflight report"
    )
    acquisition = common.read_json(acquisition_path, "R2C acquisition manifest")[1]
    r2b_report = common.read_json(r2b_path, "R2B preflight report")[1]
    block_path, rows = common.validate_acquisition_manifest(
        root, acquisition_path, acquisition_bytes, acquisition
    )
    common.validate_r2b_report(acquisition_bytes, r2b_bytes, r2b_report)
    return {
        "sources": sources,
        "acquisition_path": acquisition_path,
        "acquisition_bytes": acquisition_bytes,
        "acquisition": acquisition,
        "r2b_path": r2b_path,
        "r2b_bytes": r2b_bytes,
        "r2b_report": r2b_report,
        "block_path": block_path,
        "rows": rows,
    }


def feature_row_hashes(r2b_report: dict[str, Any]) -> dict[int, str]:
    rows = r2b_report.get("representation", {}).get("rows")
    if not isinstance(rows, list) or len(rows) != common.r2b.ROW_COUNT:
        raise common.R2CError("R2B feature row records changed")
    result = {}
    for row in rows:
        index = row.get("row_index")
        digest = row.get("complex_stft_sha256")
        if not isinstance(index, int) or not isinstance(digest, str):
            raise common.R2CError("R2B feature row identity changed")
        result[index] = digest
    return result


def run_preflight(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    inputs: dict[str, Any],
    output_argument: Path,
) -> None:
    output, staging = common.prepare_output(root, output_argument, "R2C preflight output")
    try:
        rows = inputs["rows"]
        context_rows = [row for row in rows if row["split_role"] == "context"]
        query_rows = [row for row in rows if row["split_role"] == "query"]
        if len(context_rows) != common.r2b.CONTEXT_ROWS or len(query_rows) != common.r2b.QUERY_ROWS:
            raise common.R2CError("R2C context/query count changed")
        report_scale = inputs["r2b_report"]["normalization"]["scale"]
        if not math.isfinite(report_scale) or report_scale <= 0.0:
            raise common.R2CError("R2C context normalization scale changed")
        transform = common.r2b.ComplexTransform(common.r2b.SAMPLE_COUNT)
        feature_shape = (
            len(context_rows),
            transform.frame_count,
            common.r2b.FFT_LENGTH // 2 + 1,
        )
        cache_path = staging / "context-features.c64le"
        cache = np.memmap(cache_path, dtype="<c8", mode="w+", shape=feature_shape)
        mapped = np.memmap(
            inputs["block_path"],
            dtype="<f4",
            mode="r",
            shape=(common.r2b.ROW_COUNT, common.r2b.SAMPLE_COUNT),
        )
        expected_feature_hash = feature_row_hashes(inputs["r2b_report"])
        total_tf = feature_shape[1] * feature_shape[2]
        energy = np.zeros(total_tf, dtype=np.float64)
        sum_abs = 0.0
        sum_squared = 0.0
        value_count = 0
        maximum_abs = 0.0
        context_records = []
        for cache_index, row in enumerate(context_rows):
            row_index = row["row_index"]
            source_view = np.asarray(mapped[row_index], dtype="<f4")
            source_bytes = source_view.tobytes()
            if common.sha256_bytes(source_bytes) != row["raw_f32le_sha256"]:
                raise common.R2CError(f"R2C context source hash changed at row {row_index}")
            source = source_view.astype(np.float64) * report_scale
            spectrum = transform.analyze(source)
            spectrum_bytes = spectrum.astype("<c8", copy=False).tobytes()
            if common.sha256_bytes(spectrum_bytes) != expected_feature_hash[row_index]:
                raise common.R2CError(f"R2C context STFT hash changed at row {row_index}")
            cache[cache_index] = spectrum
            magnitude = np.abs(spectrum).astype(np.float64).reshape(-1)
            energy += magnitude * magnitude
            sum_abs += float(np.sum(magnitude))
            sum_squared += float(np.sum(magnitude * magnitude))
            value_count += magnitude.size
            maximum_abs = max(maximum_abs, float(np.max(magnitude)))
            context_records.append(
                {
                    "cache_index": cache_index,
                    "row_index": row_index,
                    "azimuth_degrees": row["azimuth_degrees"],
                    "gantry_distance_offset_millimetres": row[
                        "gantry_distance_offset_millimetres"
                    ],
                    "microphone_id": row["microphone_id"],
                    "listener_position_metres": row["listener_position_metres"],
                    "source_sha256": row["raw_f32le_sha256"],
                    "complex_stft_sha256": expected_feature_hash[row_index],
                }
            )
        cache.flush()
        del cache
        del mapped
        mean_abs = sum_abs / value_count
        rms_abs = math.sqrt(sum_squared / value_count)
        if not all(math.isfinite(value) and value > 0.0 for value in (mean_abs, rms_abs)):
            raise common.R2CError("R2C context feature scale is invalid")
        indices = np.arange(total_tf, dtype=np.int64)
        salient = np.lexsort((indices, -energy))[: common.SALIENT_TF_COUNT].astype("<u4")
        salient_path = staging / "salient-time-frequency-indices.u32le"
        salient_path.write_bytes(salient.tobytes())
        impact = np.asarray(inputs["acquisition"]["impact_position_metres"], dtype=np.float64)
        spatial_center, spatial_scale = common.spatial_normalization(context_rows, impact)
        collocation = common.collocation_positions(rows)
        collocation_bytes = collocation.astype("<f8", copy=False).tobytes()
        query_records = [
            {
                "row_index": row["row_index"],
                "azimuth_degrees": row["azimuth_degrees"],
                "gantry_distance_offset_millimetres": row[
                    "gantry_distance_offset_millimetres"
                ],
                "microphone_id": row["microphone_id"],
                "listener_position_metres": row["listener_position_metres"],
            }
            for row in query_rows
        ]
        report = {
            "schema": common.PREFLIGHT_SCHEMA,
            "status": "Validated",
            "decision": "R2CContextFeaturesReadyForFrozenTraining",
            "claim": (
                "CONTEXT_ONLY_COMPLEX_FEATURES_AND_TRAINING_SCALES / NO_TRAINED_"
                "MODEL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "revision": common.REVISION,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "runner_sha256": common.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": inputs["sources"],
            "acquisition_manifest_sha256": common.sha256_bytes(inputs["acquisition_bytes"]),
            "r2b_preflight_report_sha256": common.sha256_bytes(inputs["r2b_bytes"]),
            "environment": common.environment_profile(),
            "candidate_profile": common.candidate_profile(),
            "representation_id": common.r2b.REPRESENTATION_ID,
            "feature_shape": list(feature_shape),
            "context_feature_cache": common.file_ref(cache_path, relative_to=staging),
            "salient_time_frequency_indices": common.file_ref(
                salient_path, relative_to=staging
            ),
            "context_feature_statistics": {
                "complex_value_count": value_count,
                "mean_magnitude": mean_abs,
                "rms_magnitude": rms_abs,
                "maximum_magnitude": maximum_abs,
                "pressure_scale": rms_abs,
                "mean_magnitude_after_pressure_scale": mean_abs / rms_abs,
            },
            "impact_position_metres": impact.tolist(),
            "spatial_normalization": {
                "center_relative_to_impact_metres": spatial_center.tolist(),
                "scale_metres": spatial_scale.tolist(),
            },
            "collocation": {
                "position_count": len(collocation),
                "positions_f64le_sha256": common.sha256_bytes(collocation_bytes),
                "construction": common.candidate_profile()["physics_ablation"][
                    "collocation"
                ],
            },
            "context_rows": context_records,
            "query_rows_without_audio": query_records,
            "context_raw_audio_bytes_read": len(context_rows)
            * common.r2b.SAMPLE_COUNT
            * 4,
            "query_audio_bytes_read": 0,
            "query_rows_used_for_normalization": 0,
            "query_rows_cached_for_fit": 0,
            "method_holdout_or_shadow_bytes_read": 0,
            "optimizer_steps": 0,
            "training_authorized": True,
            "quality_or_admission_authorized": False,
        }
        report_bytes = common.canonical_json(report)
        (staging / "preflight-report.json").write_bytes(report_bytes)
        common.publish_staging(staging, output)
    except BaseException:
        common.discard_staging(staging)
        raise
    common.emit_summary(
        {
            "output": str(output),
            "decision": report["decision"],
            "preflight_report_sha256": common.sha256_bytes(report_bytes),
            "context_feature_cache_sha256": report["context_feature_cache"]["sha256"],
            "query_audio_bytes_read": 0,
        },
        (
            "output",
            "decision",
            "preflight_report_sha256",
            "context_feature_cache_sha256",
            "query_audio_bytes_read",
        ),
    )


def validate_preflight(
    root: Path,
    manifest_bytes: bytes,
    path: Path,
) -> tuple[dict[str, Any], Path, np.ndarray]:
    preflight_path = common.external_file(root, path, "R2C preflight report")
    _, report = common.read_json(preflight_path, "R2C preflight report")
    if (
        report.get("schema") != common.PREFLIGHT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "R2CContextFeaturesReadyForFrozenTraining"
        or report.get("manifest_sha256") != common.sha256_bytes(manifest_bytes)
        or report.get("runner_sha256")
        != common.sha256_file(Path(__file__).resolve(strict=True))
        or report.get("environment") != common.environment_profile()
        or report.get("candidate_profile") != common.candidate_profile()
        or report.get("query_audio_bytes_read") != 0
        or report.get("query_rows_cached_for_fit") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
        or report.get("optimizer_steps") != 0
        or report.get("training_authorized") is not True
    ):
        raise common.R2CError("R2C preflight lineage changed")
    cache_ref = common.require_ref(report["context_feature_cache"], "context feature cache")
    cache_path = preflight_path.parent / cache_ref["path"]
    cache_path = common.external_file(root, cache_path, "R2C context feature cache")
    if cache_path.stat().st_size != cache_ref["byte_count"] or common.sha256_file(
        cache_path
    ) != cache_ref["sha256"]:
        raise common.R2CError("R2C context feature cache changed")
    salient_ref = common.require_ref(
        report["salient_time_frequency_indices"], "salient TF indices"
    )
    salient_path = common.external_file(
        root, preflight_path.parent / salient_ref["path"], "salient TF indices"
    )
    if salient_path.stat().st_size != salient_ref["byte_count"] or common.sha256_file(
        salient_path
    ) != salient_ref["sha256"]:
        raise common.R2CError("R2C salient TF indices changed")
    salient = np.fromfile(salient_path, dtype="<u4")
    if salient.shape != (common.SALIENT_TF_COUNT,):
        raise common.R2CError("R2C salient TF coverage changed")
    return report, cache_path, salient


def physics_loss(
    model: common.SeparableComplexField,
    collocation: torch.Tensor,
    impact: torch.Tensor,
    time_seconds: torch.Tensor,
    frequency_hz: torch.Tensor,
) -> torch.Tensor:
    step = common.PHYSICS_FINITE_DIFFERENCE_METRES
    offsets = torch.zeros((7, 3), dtype=torch.float32, device=collocation.device)
    offsets[1, 0] = step
    offsets[2, 0] = -step
    offsets[3, 1] = step
    offsets[4, 1] = -step
    offsets[5, 2] = step
    offsets[6, 2] = -step
    positions = (collocation[None, :, :] + offsets[:, None, :]).reshape(-1, 3)
    pressure = model(positions, impact, time_seconds, frequency_hz).reshape(
        7, collocation.shape[0], time_seconds.shape[0], 2
    )
    centre = pressure[0]
    laplacian = (torch.sum(pressure[1:], dim=0) - 6.0 * centre) / (step * step)
    wave_number_squared = (
        2.0 * math.pi * frequency_hz / common.SPEED_OF_SOUND_METRES_PER_SECOND
    ) ** 2
    residual = laplacian + wave_number_squared[None, :, None] * centre
    operator_scale = wave_number_squared + 1.0 / (
        common.PHYSICS_CHARACTERISTIC_LENGTH_METRES**2
    )
    normalized = residual / operator_scale[None, :, None]
    return torch.mean(torch.sum(normalized * normalized, dim=-1))


def exact_context_losses(
    model: common.SeparableComplexField,
    cache: np.memmap,
    listeners: torch.Tensor,
    impact: torch.Tensor,
    pressure_scale: float,
    mean_abs_scaled: float,
    device: torch.device,
) -> dict[str, float]:
    total_tf = cache.shape[1] * cache.shape[2]
    complex_sum = 0.0
    log_sum = 0.0
    count = 0
    model.eval()
    with torch.no_grad():
        for start in range(0, total_tf, common.PREDICTION_TF_CHUNK):
            stop = min(total_tf, start + common.PREDICTION_TF_CHUNK)
            indices = np.arange(start, stop, dtype=np.int64)
            times, frequencies = common.tf_coordinates(indices)
            prediction = model(
                listeners,
                impact,
                torch.from_numpy(times).to(device),
                torch.from_numpy(frequencies).to(device),
            )
            target_complex = np.asarray(cache[:, :, :]).reshape(cache.shape[0], -1)[
                :, start:stop
            ]
            target = np.stack(
                (target_complex.real, target_complex.imag), axis=-1
            ).astype(np.float32) / pressure_scale
            target_tensor = torch.from_numpy(target).to(device)
            difference = prediction - target_tensor
            complex_error = torch.sqrt(
                torch.sum(difference * difference, dim=-1) + common.LOSS_EPSILON**2
            )
            prediction_magnitude = torch.sqrt(
                torch.sum(prediction * prediction, dim=-1) + common.LOSS_EPSILON**2
            )
            target_magnitude = torch.sqrt(
                torch.sum(target_tensor * target_tensor, dim=-1)
                + common.LOSS_EPSILON**2
            )
            complex_sum += float(torch.sum(complex_error).cpu())
            log_sum += float(
                torch.sum(
                    torch.abs(
                        torch.log1p(prediction_magnitude / mean_abs_scaled)
                        - torch.log1p(target_magnitude / mean_abs_scaled)
                    )
                ).cpu()
            )
            count += prediction_magnitude.numel()
    complex_l1 = complex_sum / count / mean_abs_scaled
    log_l1 = log_sum / count
    return {
        "full_context_complex_l1": complex_l1,
        "full_context_log_magnitude_l1": log_l1,
        "full_context_objective": complex_l1 + log_l1,
        "full_context_complex_value_count": count,
    }


def cook_predictions(
    output: Path,
    model: common.SeparableComplexField,
    query_records: list[dict[str, Any]],
    impact: torch.Tensor,
    pressure_scale: float,
    device: torch.device,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    query = np.asarray(
        [row["listener_position_metres"] for row in query_records], dtype=np.float32
    )
    listeners = torch.from_numpy(query).to(device)
    total_tf = common.r2b.ComplexTransform(common.r2b.SAMPLE_COUNT).frame_count * (
        common.r2b.FFT_LENGTH // 2 + 1
    )
    temporary = output / ".query-spectra.c64le"
    spectra = np.memmap(
        temporary,
        dtype="<c8",
        mode="w+",
        shape=(len(query_records), total_tf),
    )
    model.eval()
    with torch.no_grad():
        for start in range(0, total_tf, common.PREDICTION_TF_CHUNK):
            stop = min(total_tf, start + common.PREDICTION_TF_CHUNK)
            indices = np.arange(start, stop, dtype=np.int64)
            times, frequencies = common.tf_coordinates(indices)
            prediction = model(
                listeners,
                impact,
                torch.from_numpy(times).to(device),
                torch.from_numpy(frequencies).to(device),
            ).cpu().numpy()
            complex_values = (
                prediction[..., 0] + 1.0j * prediction[..., 1]
            ).astype("<c8") * pressure_scale
            spectra[:, start:stop] = complex_values
    spectra.flush()
    transform = common.r2b.ComplexTransform(common.r2b.SAMPLE_COUNT)
    predictions = []
    failures = []
    directory = output / "predictions"
    directory.mkdir()
    for row_index, row in enumerate(query_records):
        spectrum = np.asarray(spectra[row_index]).reshape(
            transform.frame_count, common.r2b.FFT_LENGTH // 2 + 1
        )
        signal = transform.synthesize(spectrum)
        peak = float(np.max(np.abs(signal)))
        if not np.isfinite(signal).all() or peak >= 1.0:
            failures.append(
                {
                    "row_index": row["row_index"],
                    "reason": "nonfinite_or_peak_not_below_one",
                    "pre_cook_peak_abs": peak,
                }
            )
            continue
        wav = common.encode_wav(signal)
        relative = Path("predictions") / f"row-{row['row_index']:04}.wav"
        path = output / relative
        path.write_bytes(wav)
        predictions.append(
            {
                "row_index": row["row_index"],
                "listener_position_metres": row["listener_position_metres"],
                "prediction_file": str(relative),
                "prediction_sha256": common.sha256_bytes(wav),
                "prediction_byte_count": len(wav),
                "pre_cook_peak_abs": peak,
            }
        )
    del spectra
    temporary.unlink()
    return predictions, failures


def run_training(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    inputs: dict[str, Any],
    preflight_argument: Path | None,
    candidate_id: str | None,
    output_argument: Path,
) -> None:
    if preflight_argument is None or candidate_id is None:
        raise common.R2CError("train requires --preflight and --candidate")
    candidate = common.exact_candidate(candidate_id)
    preflight, cache_path, salient = validate_preflight(
        root, manifest_bytes, preflight_argument
    )
    unresolved = output_argument if output_argument.is_absolute() else root / output_argument
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise common.R2CError(f"training output must be a new external directory: {output}")
    output.mkdir()
    device = common.configure_determinism()
    feature_shape = tuple(preflight["feature_shape"])
    cache = np.memmap(cache_path, dtype="<c8", mode="r", shape=feature_shape)
    context_records = preflight["context_rows"]
    query_records = preflight["query_rows_without_audio"]
    context_positions = np.asarray(
        [row["listener_position_metres"] for row in context_records], dtype=np.float32
    )
    collocation = common.collocation_positions(inputs["rows"]).astype(np.float32)
    impact_array = np.asarray(preflight["impact_position_metres"], dtype=np.float32)
    stats = preflight["context_feature_statistics"]
    pressure_scale = float(stats["pressure_scale"])
    mean_abs_scaled = float(stats["mean_magnitude_after_pressure_scale"])
    spatial = preflight["spatial_normalization"]
    last_time = (feature_shape[1] - 1) * (
        common.r2b.HOP_LENGTH / common.r2b.SAMPLE_RATE_HZ
    )
    model = common.SeparableComplexField(
        np.asarray(spatial["center_relative_to_impact_metres"], dtype=np.float32),
        np.asarray(spatial["scale_metres"], dtype=np.float32),
        last_time,
    ).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=common.LEARNING_RATE,
        betas=common.ADAM_BETAS,
        eps=common.ADAM_EPSILON,
        weight_decay=common.WEIGHT_DECAY,
    )
    rng = np.random.default_rng(common.SEED)
    listeners_all = torch.from_numpy(context_positions).to(device)
    impact = torch.from_numpy(impact_array[None, :]).to(device)
    collocation_all = torch.from_numpy(collocation).to(device)
    total_tf = feature_shape[1] * feature_shape[2]
    loss_trace = []
    tracking = output / "mlflow"
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
    with mlflow.start_run(experiment_id=experiment_id, run_name=candidate_id) as run:
        mlflow.log_params(
            {
                "revision": common.REVISION,
                "candidate_id": candidate_id,
                "seed": common.SEED,
                "steps": common.STEPS,
                "learning_rate": common.LEARNING_RATE,
                "latent_width": common.LATENT_WIDTH,
                "helmholtz_weight": candidate["helmholtz_weight"],
                "device": "cuda:0",
                "dtype": "float32",
            }
        )
        model.train()
        for step in range(common.STEPS):
            row_indices = rng.choice(
                len(context_records), size=common.CONTEXT_BATCH_ROWS, replace=False
            )
            uniform = rng.integers(
                0, total_tf, size=common.UNIFORM_TF_SAMPLES, dtype=np.int64
            )
            salient_indices = salient[
                rng.integers(
                    0, len(salient), size=common.SALIENT_TF_SAMPLES, dtype=np.int64
                )
            ].astype(np.int64)
            tf_indices = np.concatenate((uniform, salient_indices))
            rng.shuffle(tf_indices)
            physics_positions = rng.choice(
                len(collocation), size=common.PHYSICS_BATCH_POSITIONS, replace=False
            )
            physics_frames = rng.integers(
                0, feature_shape[1], size=common.PHYSICS_BATCH_TF, dtype=np.int64
            )
            minimum_bin = math.ceil(
                common.PHYSICS_MIN_HZ
                * common.r2b.FFT_LENGTH
                / common.r2b.SAMPLE_RATE_HZ
            )
            maximum_bin = math.floor(
                common.PHYSICS_MAX_HZ
                * common.r2b.FFT_LENGTH
                / common.r2b.SAMPLE_RATE_HZ
            )
            physics_bins = rng.integers(
                minimum_bin,
                maximum_bin + 1,
                size=common.PHYSICS_BATCH_TF,
                dtype=np.int64,
            )
            times, frequencies = common.tf_coordinates(tf_indices)
            target_complex = np.asarray(cache).reshape(len(context_records), -1)[
                row_indices[:, None], tf_indices[None, :]
            ]
            target = np.stack(
                (target_complex.real, target_complex.imag), axis=-1
            ).astype(np.float32) / pressure_scale
            prediction = model(
                listeners_all[row_indices],
                impact,
                torch.from_numpy(times).to(device),
                torch.from_numpy(frequencies).to(device),
            )
            target_tensor = torch.from_numpy(target).to(device)
            data_total, complex_l1, log_l1 = common.data_losses(
                prediction, target_tensor, mean_abs_scaled
            )
            physics_time = torch.from_numpy(
                physics_frames.astype(np.float32)
                * (common.r2b.HOP_LENGTH / common.r2b.SAMPLE_RATE_HZ)
            ).to(device)
            physics_frequency = torch.from_numpy(
                physics_bins.astype(np.float32)
                * (common.r2b.SAMPLE_RATE_HZ / common.r2b.FFT_LENGTH)
            ).to(device)
            if candidate["helmholtz_weight"] > 0.0:
                helmholtz = physics_loss(
                    model,
                    collocation_all[physics_positions],
                    impact,
                    physics_time,
                    physics_frequency,
                )
            else:
                helmholtz = torch.zeros((), dtype=torch.float32, device=device)
            objective = data_total + candidate["helmholtz_weight"] * helmholtz
            if not torch.isfinite(objective):
                raise common.R2CError(f"R2C objective became non-finite at step {step + 1}")
            optimizer.zero_grad(set_to_none=True)
            objective.backward()
            gradient_norm = torch.nn.utils.clip_grad_norm_(
                model.parameters(), common.GRADIENT_CLIP_NORM
            )
            optimizer.step()
            if step == 0 or (step + 1) % common.LOG_INTERVAL == 0:
                record = {
                    "step": step + 1,
                    "objective": float(objective.detach().cpu()),
                    "complex_l1": float(complex_l1.detach().cpu()),
                    "log_magnitude_l1": float(log_l1.detach().cpu()),
                    "helmholtz_loss": float(helmholtz.detach().cpu()),
                    "gradient_norm_before_clip": float(gradient_norm.detach().cpu()),
                }
                loss_trace.append(record)
                for key in (
                    "objective",
                    "complex_l1",
                    "log_magnitude_l1",
                    "helmholtz_loss",
                    "gradient_norm_before_clip",
                ):
                    mlflow.log_metric(key, record[key], step=step + 1)
        exact_losses = exact_context_losses(
            model,
            cache,
            listeners_all,
            impact,
            pressure_scale,
            mean_abs_scaled,
            device,
        )
        checkpoint = common.write_checkpoint(output, candidate_id, model)
        predictions, cook_failures = cook_predictions(
            output,
            model,
            query_records,
            impact,
            pressure_scale,
            device,
        )
        report = {
            "schema": common.TRAINING_SCHEMA,
            "status": "Trained" if not cook_failures else "TrainedCookRejected",
            "decision": (
                "FrozenCandidateReadyForQueryEvaluation"
                if not cook_failures
                else "RejectCandidateCookFailure"
            ),
            "claim": (
                "CONTEXT_ONLY_DENSE_COMPLEX_FIELD_TRAINING_AND_QUERY_PCM_COOK / "
                "NO_QUERY_REFERENCE_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "revision": common.REVISION,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "preflight_report_sha256": common.sha256_file(
                common.external_file(root, preflight_argument, "R2C preflight report")
            ),
            "runner_sha256": common.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": inputs["sources"],
            "environment": common.environment_profile(),
            "candidate_profile": common.candidate_profile(),
            "candidate_id": candidate_id,
            "helmholtz_weight": candidate["helmholtz_weight"],
            "model_parameter_count": common.model_parameter_count(model),
            "optimizer_steps": common.STEPS,
            "loss_trace": loss_trace,
            "final_context_metrics": exact_losses,
            "checkpoint": checkpoint,
            "predictions": predictions,
            "cook_failures": cook_failures,
            "context_feature_cache_sha256": preflight["context_feature_cache"]["sha256"],
            "context_feature_bytes_read": preflight["context_feature_cache"]["byte_count"],
            "query_coordinate_rows_used_for_cook": len(query_records),
            "query_audio_bytes_read": 0,
            "query_audio_rows_read": 0,
            "method_holdout_or_shadow_bytes_read": 0,
            "checkpoint_selection": "final_step_only_no_query_or_early_stopping",
            "quality_or_admission_authorized": False,
        }
        report_bytes = common.canonical_json(report)
        report_path = output / "training-report.json"
        report_path.write_bytes(report_bytes)
        mlflow.log_artifact(str(report_path))
        mlflow.log_artifact(str(output / checkpoint["descriptor_file"]))
        mlflow.log_artifact(str(output / checkpoint["weights_file"]))
        lineage_run_id = run.info.run_id
        lineage_experiment_id = run.info.experiment_id
    lineage = {
        "schema": common.MLFLOW_SCHEMA,
        "mlflow_version": mlflow.__version__,
        "tracking_database": "mlflow/mlflow.db",
        "artifact_directory": "mlflow/artifacts",
        "experiment_name": manifest["mlflow_experiment"],
        "candidate_id": candidate_id,
        "run_id": lineage_run_id,
        "experiment_id": lineage_experiment_id,
        "deterministic_training_report_sha256": common.sha256_bytes(report_bytes),
        "checkpoint_weights_sha256": checkpoint["weights_sha256"],
    }
    (output / "mlflow-lineage.json").write_bytes(common.canonical_json(lineage))
    common.emit_summary(
        {
            "output": str(output),
            "candidate_id": candidate_id,
            "status": report["status"],
            "training_report_sha256": common.sha256_bytes(report_bytes),
            "checkpoint_weights_sha256": checkpoint["weights_sha256"],
            "final_context_objective": exact_losses["full_context_objective"],
            "query_audio_bytes_read": 0,
        },
        (
            "output",
            "candidate_id",
            "status",
            "training_report_sha256",
            "checkpoint_weights_sha256",
            "final_context_objective",
            "query_audio_bytes_read",
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    root = common.root_from_script(Path(__file__))
    if arguments.stage == "freeze":
        if arguments.manifest is not None or arguments.preflight is not None or arguments.candidate is not None:
            raise common.R2CError("freeze accepts no train/preflight arguments")
        freeze_manifest(
            root,
            arguments.acquisition_manifest,
            arguments.r2b_report,
            arguments.output,
        )
        return
    if arguments.acquisition_manifest is not None or arguments.r2b_report is not None:
        raise common.R2CError("preflight/train use only --manifest lineage")
    if arguments.manifest is None:
        raise common.R2CError("preflight/train require --manifest")
    manifest_path = common.external_file(root, arguments.manifest, "R2C manifest")
    manifest_bytes, manifest = common.read_json(manifest_path, "R2C manifest")
    inputs = validate_manifest(root, manifest_path, manifest)
    if arguments.stage == "preflight":
        if arguments.preflight is not None or arguments.candidate is not None:
            raise common.R2CError("preflight accepts no train-only arguments")
        run_preflight(
            root,
            manifest_path,
            manifest_bytes,
            manifest,
            inputs,
            arguments.output,
        )
    else:
        run_training(
            root,
            manifest_path,
            manifest_bytes,
            manifest,
            inputs,
            arguments.preflight,
            arguments.candidate,
            arguments.output,
        )


if __name__ == "__main__":
    main()
