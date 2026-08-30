#!/usr/bin/env python3
"""Freeze and train the single R2E low-rank listener-coordinate field."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import torch

import physical_sound_listener_field_r2d_v2_trainability as r2d_v2_runner
import physical_sound_listener_field_r2e_common as common

v1 = common.v1
r2c = common.r2c


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["freeze", "run"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--r2d-v2-manifest", type=Path)
    parser.add_argument("--r2d-v2-run-report", action="append", type=Path)
    parser.add_argument("--manifest", type=Path)
    return parser.parse_args()


def implementation_paths() -> list[str]:
    return [
        "lab/scripts/physical_sound_listener_field_r2b_dense_preflight.py",
        "lab/scripts/physical_sound_listener_field_r2c_common.py",
        "lab/scripts/physical_sound_listener_field_r2d_common.py",
        "lab/scripts/physical_sound_listener_field_r2d_trainability.py",
        "lab/scripts/physical_sound_listener_field_r2d_v2_common.py",
        "lab/scripts/physical_sound_listener_field_r2d_v2_trainability.py",
        "lab/scripts/physical_sound_listener_field_r2e_common.py",
        "lab/scripts/physical_sound_listener_field_r2e_train.py",
    ]


def validate_v2_run_report(
    root: Path, argument: Path
) -> tuple[Path, bytes, dict[str, Any]]:
    path = r2c.external_file(root, argument, "R2D V2 run report")
    data, report = r2c.read_json(path, "R2D V2 run report")
    if r2c.sha256_bytes(data) != common.R2D_V2_RUN_REPORT_SHA256:
        raise common.R2EError("R2D V2 run report identity changed")
    if (
        report.get("schema") != common.r2d_v2.RUN_REPORT_SCHEMA
        or report.get("revision") != common.r2d_v2.REVISION
        or report.get("decision") != "R2DTrainabilityGatePass"
        or report.get("profile") != common.r2d_v2.profile()
        or report.get("manifest_sha256") != common.R2D_V2_MANIFEST_SHA256
        or report.get("all_tasks_pass") is not True
        or report.get("n0_3e_authorized") is not True
        or report.get("query_audio_bytes_read") != 0
        or report.get("query_audio_rows_read") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
        or len(report.get("tasks", [])) != 3
        or not all(
            task.get("passes_trainability_gate") is True
            for task in report.get("tasks", [])
        )
    ):
        raise common.R2EError("R2D V2 did not authorize the R2E substrate")
    return path, data, report


def validate_coordinate_rows(
    context_rows: list[dict[str, Any]], query_rows: list[dict[str, Any]]
) -> None:
    cpu = torch.device("cpu")
    _, context_groups = common.encode_rows(
        context_rows, cpu, expected_angles=common.CONTEXT_ANGLES_DEGREES
    )
    _, query_groups = common.encode_rows(
        query_rows, cpu, expected_angles=common.QUERY_ANGLES_DEGREES
    )
    if (
        len(context_rows) != common.r2b.CONTEXT_ROWS
        or len(query_rows) != common.r2b.QUERY_ROWS
        or set(context_groups.tolist()) != set(range(common.GROUP_COUNT))
        or set(query_groups.tolist()) != set(range(common.GROUP_COUNT))
    ):
        raise common.R2EError("R2E published coordinate coverage changed")
    expected_context = {
        (angle, group)
        for angle in common.CONTEXT_ANGLES_DEGREES
        for group in range(common.GROUP_COUNT)
    }
    expected_query = {
        (angle, group)
        for angle in common.QUERY_ANGLES_DEGREES
        for group in range(common.GROUP_COUNT)
    }
    measured_context = {
        (
            row["azimuth_degrees"],
            common.distance_index(row["gantry_distance_offset_millimetres"])
            * len(common.MICROPHONE_IDS)
            + row["microphone_id"],
        )
        for row in context_rows
    }
    measured_query = {
        (
            row["azimuth_degrees"],
            common.distance_index(row["gantry_distance_offset_millimetres"])
            * len(common.MICROPHONE_IDS)
            + row["microphone_id"],
        )
        for row in query_rows
    }
    if measured_context != expected_context or measured_query != expected_query:
        raise common.R2EError("R2E coordinate grid is incomplete")


def validate_v2_manifest(
    root: Path, argument: Path
) -> tuple[Path, bytes, dict[str, Any], dict[str, Any], dict[str, Any]]:
    path = r2c.external_file(root, argument, "R2D V2 manifest")
    data, manifest = r2c.read_json(path, "R2D V2 manifest")
    if r2c.sha256_bytes(data) != common.R2D_V2_MANIFEST_SHA256:
        raise common.R2EError("R2D V2 manifest identity changed")
    v1_manifest, inputs = r2d_v2_runner.validate_manifest(root, path, manifest)
    context_rows = list(inputs["preflight"]["context_rows"])
    context_rows.sort(key=lambda row: row["cache_index"])
    if [row["cache_index"] for row in context_rows] != list(
        range(common.r2b.CONTEXT_ROWS)
    ):
        raise common.R2EError("R2E context coefficient row order changed")
    query_rows = list(inputs["preflight"]["query_rows_without_audio"])
    query_rows.sort(key=lambda row: row["row_index"])
    validate_coordinate_rows(context_rows, query_rows)
    inputs = {**inputs, "context_rows": context_rows, "query_rows": query_rows}
    return path, data, manifest, v1_manifest, inputs


def freeze(
    root: Path,
    v2_manifest_argument: Path | None,
    v2_run_arguments: list[Path] | None,
    output_argument: Path,
) -> None:
    if v2_manifest_argument is None or v2_run_arguments is None:
        raise common.R2EError(
            "freeze requires --r2d-v2-manifest and two --r2d-v2-run-report values"
        )
    if len(v2_run_arguments) != 2:
        raise common.R2EError("freeze requires exactly two R2D V2 run reports")
    v2_path, v2_bytes, _, v1_manifest, inputs = validate_v2_manifest(
        root, v2_manifest_argument
    )
    repetitions = []
    for argument in v2_run_arguments:
        path, data, report = validate_v2_run_report(root, argument)
        repetitions.append(
            {
                "run_report": r2c.file_ref(path),
                "run_report_sha256": r2c.sha256_bytes(data),
                "checkpoint_weights_sha256": {
                    task["task_id"]: task["checkpoint"]["weights_sha256"]
                    for task in report["tasks"]
                },
            }
        )
    repetitions.sort(key=lambda record: record["run_report"]["path"])
    if repetitions[0]["checkpoint_weights_sha256"] != repetitions[1][
        "checkpoint_weights_sha256"
    ]:
        raise common.R2EError("R2D V2 checkpoint repetition changed")
    output, staging = r2c.prepare_output(
        root, output_argument, "R2E training freeze output"
    )
    try:
        manifest = {
            "schema": common.TRAIN_MANIFEST_SCHEMA,
            "revision": common.REVISION,
            "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": r2c.source_records(
                root, implementation_paths()
            ),
            "environment": r2c.environment_profile(),
            "profile": common.profile(),
            "r2d_v2_manifest": r2c.file_ref(v2_path),
            "r2d_v2_repetitions": repetitions,
            "mlflow_experiment": "nextengine-physical-sound-r2e-harmonic-field",
        }
        manifest_bytes = r2c.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": common.TRAIN_FREEZE_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R2ESingleCandidateProtocolFrozen",
            "claim": (
                "ONE_CONTEXT_ONLY_COORDINATE_TO_COEFFICIENT_CANDIDATE / NO_"
                "QUERY_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "r2d_v2_manifest_sha256": r2c.sha256_bytes(v2_bytes),
            "r2d_v2_run_report_sha256": common.R2D_V2_RUN_REPORT_SHA256,
            "factorization_diagnostics": v1_manifest["factorization"][
                "diagnostics"
            ],
            "context_row_count": len(inputs["context_rows"]),
            "query_coordinate_row_count": len(inputs["query_rows"]),
            "optimizer_steps": 0,
            "query_audio_bytes_read": 0,
            "method_holdout_or_shadow_bytes_read": 0,
            "training_runs_authorized": 2,
            "query_evaluation_runs_authorized_after_repeat_freeze": 1,
            "quality_or_admission_authorized": False,
        }
        report_bytes = r2c.canonical_json(report)
        (staging / "freeze-report.json").write_bytes(report_bytes)
        r2c.publish_staging(staging, output)
    except BaseException:
        r2c.discard_staging(staging)
        raise
    r2c.emit_summary(
        {
            "output": str(output),
            "decision": report["decision"],
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "query_audio_bytes_read": 0,
        },
        ("output", "decision", "manifest_sha256", "query_audio_bytes_read"),
    )


def validate_manifest(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any]]:
    required = {
        "schema",
        "revision",
        "runner_sha256",
        "implementation_sources",
        "environment",
        "profile",
        "r2d_v2_manifest",
        "r2d_v2_repetitions",
        "mlflow_experiment",
    }
    if set(manifest) != required:
        raise common.R2EError("R2E training manifest fields changed")
    if (
        manifest.get("schema") != common.TRAIN_MANIFEST_SCHEMA
        or manifest.get("revision") != common.REVISION
        or manifest.get("runner_sha256")
        != r2c.sha256_file(Path(__file__).resolve(strict=True))
        or manifest.get("implementation_sources")
        != r2c.source_records(root, implementation_paths())
        or manifest.get("environment") != r2c.environment_profile()
        or manifest.get("profile") != common.profile()
        or manifest.get("mlflow_experiment")
        != "nextengine-physical-sound-r2e-harmonic-field"
        or len(manifest.get("r2d_v2_repetitions", [])) != 2
    ):
        raise common.R2EError("R2E training manifest identity changed")
    v2_path = v1.validate_file_ref(
        root,
        manifest_path.parent,
        manifest["r2d_v2_manifest"],
        "R2D V2 manifest",
    )
    _, _, _, v1_manifest, inputs = validate_v2_manifest(root, v2_path)
    for repetition in manifest["r2d_v2_repetitions"]:
        report_path = v1.validate_file_ref(
            root,
            manifest_path.parent,
            repetition["run_report"],
            "R2D V2 run report",
        )
        _, data, report = validate_v2_run_report(root, report_path)
        expected_checkpoints = {
            task["task_id"]: task["checkpoint"]["weights_sha256"]
            for task in report["tasks"]
        }
        if (
            repetition.get("run_report_sha256") != r2c.sha256_bytes(data)
            or repetition.get("checkpoint_weights_sha256")
            != expected_checkpoints
        ):
            raise common.R2EError("R2D V2 repetition lineage changed")
    return v1_manifest, inputs


def train_field(
    arrays: dict[str, np.ndarray],
    context_rows: list[dict[str, Any]],
    output: Path,
    device: torch.device,
) -> tuple[dict[str, Any], common.HarmonicCoefficientField]:
    component_rms = torch.from_numpy(
        np.array(arrays["coefficient_rms"], dtype=np.float32, copy=True)
    ).to(device)
    target_raw = np.array(arrays["coefficients"], copy=True)
    target_normalized = target_raw / np.array(
        arrays["coefficient_rms"], dtype=np.float32, copy=True
    )[None, :]
    target = v1.tensor_pairs(target_normalized, device)
    mean_projection = v1.tensor_pairs(
        np.array(arrays["mean_projection"], copy=True), device
    )
    mean_energy = torch.tensor(
        float(np.sum(np.abs(arrays["mean_field"]) ** 2, dtype=np.float64)),
        dtype=torch.float32,
        device=device,
    )
    angles, groups = common.encode_rows(
        context_rows,
        device,
        expected_angles=common.CONTEXT_ANGLES_DEGREES,
    )
    torch.manual_seed(common.SEED)
    torch.cuda.manual_seed_all(common.SEED)
    model = common.HarmonicCoefficientField().to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=common.r2d_v2.INITIAL_LEARNING_RATE,
        betas=common.r2d_v2.ADAM_BETAS,
        eps=common.r2d_v2.ADAM_EPSILON,
        weight_decay=common.r2d_v2.WEIGHT_DECAY,
    )
    zero_objective, zero_parts = v1.coefficient_objective(
        model(angles, groups),
        target,
        component_rms,
        mean_energy,
        mean_projection,
    )
    trace = []
    clip_count = 0
    for step in range(common.STEPS):
        learning_rate = common.learning_rate_for_step(step)
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        prediction = model(angles, groups)
        objective, parts = v1.coefficient_objective(
            prediction,
            target,
            component_rms,
            mean_energy,
            mean_projection,
        )
        if not torch.isfinite(objective):
            raise common.R2EError("R2E context objective became non-finite")
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
                    "learning_rate": learning_rate,
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
    prediction = model(angles, groups)
    final_objective, final_parts = v1.coefficient_objective(
        prediction,
        target,
        component_rms,
        mean_energy,
        mean_projection,
    )
    checkpoint_root = output / "checkpoint"
    checkpoint_root.mkdir()
    weights = model.weights.detach().cpu().numpy().astype("<f4")
    weights_path = checkpoint_root / "harmonic-field-weights.f32le"
    weights_path.write_bytes(weights.tobytes(order="C"))
    descriptor = {
        "schema": "nextengine.experimental-r2e-harmonic-field-checkpoint.v1",
        "revision": common.REVISION,
        "candidate_id": common.CANDIDATE_ID,
        "shape": list(weights.shape),
        "dtype": "float32le",
        "weights_sha256": r2c.sha256_file(weights_path),
        "weights_byte_count": weights_path.stat().st_size,
    }
    descriptor_bytes = r2c.canonical_json(descriptor)
    descriptor_path = checkpoint_root / "checkpoint.json"
    descriptor_path.write_bytes(descriptor_bytes)
    report = {
        "context_row_count": len(context_rows),
        "optimizer_steps": common.STEPS,
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


def cook_coefficients(
    rows: list[dict[str, Any]],
    normalized: torch.Tensor,
    arrays: dict[str, np.ndarray],
    output: Path,
    device: torch.device,
    *,
    identity_key: str,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if len(rows) != normalized.shape[0]:
        raise common.R2EError("R2E cook row/coefficient count changed")
    directory = output / "predictions" / identity_key
    directory.mkdir(parents=True)
    rms = torch.from_numpy(
        np.array(arrays["coefficient_rms"], dtype=np.float32, copy=True)
    ).to(device)
    raw = v1.as_complex(normalized.detach()) * rms[None, :]
    total_tf = arrays["mean_field"].shape[0]
    temporary = output / f".{identity_key}-spectra.c64le"
    spectra = np.memmap(
        temporary, dtype="<c8", mode="w+", shape=(len(rows), total_tf)
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
    transform = r2c.r2b.ComplexTransform(r2c.r2b.SAMPLE_COUNT)
    records = []
    failures = []
    for offset, row in enumerate(rows):
        identity = row[identity_key]
        spectrum = np.asarray(spectra[offset]).reshape(
            transform.frame_count, r2c.r2b.FFT_LENGTH // 2 + 1
        )
        signal = transform.synthesize(spectrum)
        peak = float(np.max(np.abs(signal)))
        if not np.isfinite(signal).all() or peak >= 1.0:
            failures.append(
                {
                    identity_key: identity,
                    "reason": "nonfinite_or_peak_not_below_one",
                    "pre_cook_peak_abs": peak,
                }
            )
            continue
        path = directory / f"{identity_key}-{identity:04}.wav"
        payload = r2c.encode_wav(signal)
        path.write_bytes(payload)
        records.append(
            {
                identity_key: identity,
                "azimuth_degrees": row["azimuth_degrees"],
                "gantry_distance_offset_millimetres": row[
                    "gantry_distance_offset_millimetres"
                ],
                "microphone_id": row["microphone_id"],
                "prediction_file": str(path.relative_to(output)),
                "prediction_sha256": r2c.sha256_bytes(payload),
                "prediction_byte_count": len(payload),
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
        raise common.R2EError("run requires --manifest")
    manifest_path = r2c.external_file(root, manifest_argument, "R2E manifest")
    manifest_bytes, manifest = r2c.read_json(manifest_path, "R2E manifest")
    v1_manifest, inputs = validate_manifest(root, manifest_path, manifest)
    output, staging = r2c.prepare_output(root, output_argument, "R2E run output")
    try:
        device = v1.configure_determinism()
        feature_shape = tuple(inputs["preflight"]["feature_shape"])
        total_tf = feature_shape[1] * feature_shape[2]
        arrays = v1.load_factorization_arrays(inputs["artifact_paths"], total_tf)
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
        with mlflow.start_run(
            experiment_id=experiment_id,
            run_name="r2e-harmonic-listener-field",
        ) as active_run:
            mlflow.log_params(
                {
                    "revision": common.REVISION,
                    "candidate_id": common.CANDIDATE_ID,
                    "seed": common.SEED,
                    "rank": common.RANK,
                    "steps": common.STEPS,
                    "model_parameter_count": common.profile()["model"][
                        "parameter_count"
                    ],
                    "query_audio_bytes_read": 0,
                }
            )
            training, model = train_field(
                arrays, inputs["context_rows"], staging, device
            )
            context_rows_by_index = {
                row["cache_index"]: row for row in inputs["context_rows"]
            }
            probe_indices = list(v1.FULL_PROBE_CACHE_INDICES)
            probe_rows = [context_rows_by_index[index] for index in probe_indices]
            with torch.no_grad():
                angles, groups = common.encode_rows(
                    probe_rows,
                    device,
                    expected_angles=common.CONTEXT_ANGLES_DEGREES,
                )
                probe_coefficients = model(angles, groups)
            context_predictions, context_cook_failures = cook_coefficients(
                probe_rows,
                probe_coefficients,
                arrays,
                staging,
                device,
                identity_key="cache_index",
            )
            context_gate_pass = False
            gate_evidence: dict[str, Any] = {"checks": {"cook": False}}
            trained_metrics = None
            if not context_cook_failures and len(context_predictions) == len(
                probe_rows
            ):
                prediction_by_index = {
                    record["cache_index"]: staging / record["prediction_file"]
                    for record in context_predictions
                }
                pairs = [
                    {
                        "cache_index": index,
                        "candidate_path": str(prediction_by_index[index]),
                        "reference_path": str(inputs["references"][index]),
                    }
                    for index in probe_indices
                ]
                metrics_root = staging / "context-metrics"
                metrics_root.mkdir()
                trained_metrics = v1.run_rust_metrics(
                    root, "r2e-context", pairs, metrics_root
                )
                zero = v1.subset_metrics(
                    inputs["controls"]["zero"]["metrics"], probe_indices
                )
                mean = v1.subset_metrics(
                    inputs["controls"]["global_mean"]["metrics"], probe_indices
                )
                oracle = v1.subset_metrics(
                    inputs["controls"]["rank96_oracle"]["metrics"],
                    probe_indices,
                )
                context_gate_pass, gate_evidence = v1.task_metrics_pass(
                    training, trained_metrics, oracle, zero, mean
                )
                frozen_controls = {
                    "zero": zero,
                    "global_mean": mean,
                    "rank96_oracle": oracle,
                }
            else:
                frozen_controls = None
            query_predictions = []
            query_cook_failures = []
            if context_gate_pass:
                with torch.no_grad():
                    angles, groups = common.encode_rows(
                        inputs["query_rows"],
                        device,
                        expected_angles=common.QUERY_ANGLES_DEGREES,
                    )
                    query_coefficients = model(angles, groups)
                query_predictions, query_cook_failures = cook_coefficients(
                    inputs["query_rows"],
                    query_coefficients,
                    arrays,
                    staging,
                    device,
                    identity_key="row_index",
                )
            ready = (
                context_gate_pass
                and not query_cook_failures
                and len(query_predictions) == common.r2b.QUERY_ROWS
            )
            decision = (
                "R2ECandidateFrozenForQueryEvaluation"
                if ready
                else "RejectLowRankCoefficientFieldBeforeQuery"
            )
            report = {
                "schema": common.TRAIN_REPORT_SCHEMA,
                "status": "Validated",
                "decision": decision,
                "claim": (
                    "CONTEXT_TRAINED_LOW_RANK_COORDINATE_FIELD_AND_QUERY_"
                    "PREDICTIONS_WITHOUT_QUERY_AUDIO / NO_QUERY_QUALITY_"
                    "ADMISSION_OR_RUNTIME_AUTHORITY"
                ),
                "revision": common.REVISION,
                "candidate_id": common.CANDIDATE_ID,
                "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
                "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
                "implementation_sources": manifest["implementation_sources"],
                "environment": r2c.environment_profile(),
                "profile": common.profile(),
                "training": training,
                "context_predictions": context_predictions,
                "context_cook_failures": context_cook_failures,
                "context_trained_metrics": trained_metrics,
                "context_frozen_controls": frozen_controls,
                "context_gate_evidence": gate_evidence,
                "context_gate_pass": context_gate_pass,
                "query_predictions": query_predictions,
                "query_cook_failures": query_cook_failures,
                "candidate_ready_for_query_evaluation": ready,
                "optimizer_steps": common.STEPS,
                "context_feature_cache_bytes_read": 0,
                "frozen_context_factorization_bytes_read": sum(
                    reference.get("byte_count", 0)
                    for reference in v1_manifest["factorization"][
                        "artifacts"
                    ].values()
                ),
                "query_coordinate_rows_read": len(inputs["query_rows"]),
                "query_audio_bytes_read": 0,
                "query_audio_rows_read": 0,
                "method_holdout_or_shadow_bytes_read": 0,
                "quality_or_admission_authorized": False,
                "next_action": (
                    "freeze_two_repetitions_then_run_one_grouped_query_evaluation"
                    if ready
                    else "reject_candidate_without_opening_query_audio"
                ),
            }
            report_bytes = r2c.canonical_json(report)
            report_path = staging / "training-report.json"
            report_path.write_bytes(report_bytes)
            for name, value in training["final_metrics"].items():
                mlflow.log_metric(name, value)
            mlflow.log_metric("context_gate_pass", int(context_gate_pass))
            mlflow.log_metric("candidate_ready", int(ready))
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
            "deterministic_training_report_sha256": r2c.sha256_bytes(
                report_bytes
            ),
            "checkpoint_weights_sha256": training["checkpoint"][
                "weights_sha256"
            ],
        }
        (staging / "mlflow-lineage.json").write_bytes(r2c.canonical_json(lineage))
        r2c.publish_staging(staging, output)
    except BaseException:
        r2c.discard_staging(staging)
        raise
    r2c.emit_summary(
        {
            "output": str(output),
            "decision": decision,
            "training_report_sha256": r2c.sha256_bytes(report_bytes),
            "checkpoint_weights_sha256": training["checkpoint"][
                "weights_sha256"
            ],
            "context_gate_pass": context_gate_pass,
            "query_prediction_count": len(query_predictions),
            "query_audio_bytes_read": 0,
            "final_metrics": training["final_metrics"],
        },
        (
            "output",
            "decision",
            "training_report_sha256",
            "checkpoint_weights_sha256",
            "context_gate_pass",
            "query_prediction_count",
            "query_audio_bytes_read",
            "final_metrics",
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    root = r2c.root_from_script(Path(__file__))
    if arguments.stage == "freeze":
        if arguments.manifest is not None:
            raise common.R2EError("freeze accepts no --manifest")
        freeze(
            root,
            arguments.r2d_v2_manifest,
            arguments.r2d_v2_run_report,
            arguments.output,
        )
    else:
        if (
            arguments.r2d_v2_manifest is not None
            or arguments.r2d_v2_run_report is not None
        ):
            raise common.R2EError("run accepts only the frozen --manifest")
        run(root, arguments.manifest, arguments.output)


if __name__ == "__main__":
    main()
