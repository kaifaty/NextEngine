#!/usr/bin/env python3
"""Freeze and execute the optimizer-only R2D V2 context trainability gate."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import torch

import physical_sound_listener_field_r2d_trainability as v1_runner
import physical_sound_listener_field_r2d_v2_common as common

v1 = common.v1
r2c = common.r2c


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["freeze", "run"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--r2d-v1-manifest", type=Path)
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
    ]


def validate_v1_manifest(
    root: Path, manifest_argument: Path
) -> tuple[Path, bytes, dict[str, Any], dict[str, Any]]:
    path = r2c.external_file(root, manifest_argument, "R2D V1 manifest")
    data, manifest = r2c.read_json(path, "R2D V1 manifest")
    if r2c.sha256_bytes(data) != common.V1_MANIFEST_SHA256:
        raise common.R2DError("R2D V1 manifest identity changed")
    inputs = v1_runner.validate_manifest(root, path, manifest)
    return path, data, manifest, inputs


def schedule_samples() -> dict[str, list[dict[str, Any]]]:
    result = {}
    for task in common.TASKS:
        steps = task["steps"]
        indices = sorted({0, (steps - 1) // 2, steps - 1})
        result[task["task_id"]] = [
            {
                "optimizer_step": index + 1,
                "learning_rate": common.learning_rate_for_step(index, steps),
            }
            for index in indices
        ]
    return result


def freeze(
    root: Path, v1_manifest_argument: Path | None, output_argument: Path
) -> None:
    if v1_manifest_argument is None:
        raise common.R2DError("freeze requires --r2d-v1-manifest")
    v1_path, v1_bytes, v1_manifest, _ = validate_v1_manifest(
        root, v1_manifest_argument
    )
    output, staging = r2c.prepare_output(
        root, output_argument, "R2D V2 freeze output"
    )
    try:
        manifest = {
            "schema": common.MANIFEST_SCHEMA,
            "revision": common.REVISION,
            "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": r2c.source_records(
                root, implementation_paths()
            ),
            "environment": r2c.environment_profile(),
            "profile": common.profile(),
            "frozen_v1_manifest": r2c.file_ref(v1_path),
            "authorized_change": {
                "only": "adamw_learning_rate_schedule",
                "from": {
                    "kind": "fixed",
                    "value": v1.LEARNING_RATE,
                },
                "to": common.profile()["optimizer"]["learning_rate_schedule"],
                "unchanged": [
                    "basis",
                    "objective",
                    "initialization",
                    "tasks",
                    "steps",
                    "adam_betas_epsilon_weight_decay",
                    "gradient_clip_norm",
                    "cooker",
                    "metrics",
                    "gates",
                    "context_rows",
                ],
            },
            "schedule_samples": schedule_samples(),
            "mlflow_experiment": (
                "nextengine-physical-sound-r2d-v2-cosine-trainability"
            ),
        }
        manifest_bytes = r2c.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        controls = v1_manifest["controls"]
        report = {
            "schema": common.FREEZE_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R2DV2ContextTrainabilityScheduleFrozen",
            "claim": (
                "CONTEXT_ONLY_OPTIMIZER_SCHEDULE_REVISION / NO_QUERY_QUALITY_"
                "ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "frozen_v1_manifest_sha256": r2c.sha256_bytes(v1_bytes),
            "authorized_change": manifest["authorized_change"],
            "schedule_samples": manifest["schedule_samples"],
            "factorization_diagnostics": v1_manifest["factorization"][
                "diagnostics"
            ],
            "control_aggregates": {
                "identity": controls["identity_metrics"]["aggregate"],
                "zero": controls["zero"]["metrics"]["aggregate"],
                "global_mean": controls["global_mean"]["metrics"]["aggregate"],
                "rank96_oracle": controls["rank96_oracle"]["metrics"][
                    "aggregate"
                ],
            },
            "optimizer_steps": 0,
            "query_audio_bytes_read": 0,
            "method_holdout_or_shadow_bytes_read": 0,
            "training_authorized": True,
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
            "frozen_v1_manifest_sha256": r2c.sha256_bytes(v1_bytes),
            "query_audio_bytes_read": 0,
        },
        (
            "output",
            "decision",
            "manifest_sha256",
            "frozen_v1_manifest_sha256",
            "query_audio_bytes_read",
        ),
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
        "frozen_v1_manifest",
        "authorized_change",
        "schedule_samples",
        "mlflow_experiment",
    }
    if set(manifest) != required:
        raise common.R2DError("R2D V2 manifest fields changed")
    if (
        manifest.get("schema") != common.MANIFEST_SCHEMA
        or manifest.get("revision") != common.REVISION
        or manifest.get("runner_sha256")
        != r2c.sha256_file(Path(__file__).resolve(strict=True))
        or manifest.get("implementation_sources")
        != r2c.source_records(root, implementation_paths())
        or manifest.get("environment") != r2c.environment_profile()
        or manifest.get("profile") != common.profile()
        or manifest.get("schedule_samples") != schedule_samples()
        or manifest.get("mlflow_experiment")
        != "nextengine-physical-sound-r2d-v2-cosine-trainability"
    ):
        raise common.R2DError("R2D V2 manifest identity or environment changed")
    expected_change = {
        "only": "adamw_learning_rate_schedule",
        "from": {"kind": "fixed", "value": v1.LEARNING_RATE},
        "to": common.profile()["optimizer"]["learning_rate_schedule"],
        "unchanged": [
            "basis",
            "objective",
            "initialization",
            "tasks",
            "steps",
            "adam_betas_epsilon_weight_decay",
            "gradient_clip_norm",
            "cooker",
            "metrics",
            "gates",
            "context_rows",
        ],
    }
    if manifest.get("authorized_change") != expected_change:
        raise common.R2DError("R2D V2 authorized change widened")
    v1_path = v1.validate_file_ref(
        root,
        manifest_path.parent,
        manifest["frozen_v1_manifest"],
        "R2D V1 manifest",
    )
    _, _, v1_manifest, inputs = validate_v1_manifest(root, v1_path)
    return v1_manifest, inputs


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
    target = v1.tensor_pairs(target_normalized, device)
    mean_projection = v1.tensor_pairs(
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
        lr=common.INITIAL_LEARNING_RATE,
        betas=common.ADAM_BETAS,
        eps=common.ADAM_EPSILON,
        weight_decay=common.WEIGHT_DECAY,
    )
    zero_objective, zero_parts = v1.coefficient_objective(
        model(), target, component_rms, mean_energy, mean_projection
    )
    trace = []
    clip_count = 0
    total_steps = task_profile["steps"]
    for step in range(total_steps):
        learning_rate = common.learning_rate_for_step(step, total_steps)
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        objective, parts = v1.coefficient_objective(
            model(), target, component_rms, mean_energy, mean_projection
        )
        if not torch.isfinite(objective):
            raise common.R2DError(
                f"R2D V2 objective became non-finite for {task_profile['task_id']}"
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
    final_objective, final_parts = v1.coefficient_objective(
        model(), target, component_rms, mean_energy, mean_projection
    )
    task_directory = output / "checkpoints" / task_profile["task_id"]
    task_directory.mkdir(parents=True)
    weights = model.values.detach().cpu().numpy().astype("<f4")
    weights_path = task_directory / "normalized-coefficients.f32le"
    weights_path.write_bytes(weights.tobytes(order="C"))
    descriptor = {
        "schema": "nextengine.experimental-r2d-v2-coefficient-table-checkpoint.v1",
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
        "optimizer_steps": total_steps,
        "learning_rate_schedule": common.profile()["optimizer"][
            "learning_rate_schedule"
        ],
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


def run(
    root: Path, manifest_argument: Path | None, output_argument: Path
) -> None:
    if manifest_argument is None:
        raise common.R2DError("run requires --manifest")
    manifest_path = r2c.external_file(root, manifest_argument, "R2D V2 manifest")
    manifest_bytes, manifest = r2c.read_json(manifest_path, "R2D V2 manifest")
    v1_manifest, inputs = validate_manifest(root, manifest_path, manifest)
    output, staging = r2c.prepare_output(root, output_argument, "R2D V2 run output")
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
        task_reports = []
        with mlflow.start_run(
            experiment_id=experiment_id,
            run_name="r2d-v2-cosine-context-trainability",
        ) as active_run:
            mlflow.log_params(
                {
                    "revision": common.REVISION,
                    "seed": common.SEED,
                    "rank": common.RANK,
                    "learning_rate_schedule": common.SCHEDULE,
                    "initial_learning_rate": common.INITIAL_LEARNING_RATE,
                    "final_learning_rate": common.FINAL_LEARNING_RATE,
                    "gradient_clip_norm": common.GRADIENT_CLIP_NORM,
                    "frozen_v1_manifest_sha256": common.V1_MANIFEST_SHA256,
                    "query_audio_bytes_read": 0,
                }
            )
            for task_profile in common.TASKS:
                training, model = train_task(
                    task_profile, arrays, staging, device
                )
                predictions, failures = v1_runner.cook_trained_probes(
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
                trained_metrics = v1.run_rust_metrics(
                    root, task_profile["task_id"], pairs, metrics_root
                )
                zero = v1.subset_metrics(
                    inputs["controls"]["zero"]["metrics"], probes
                )
                mean = v1.subset_metrics(
                    inputs["controls"]["global_mean"]["metrics"], probes
                )
                oracle = v1.subset_metrics(
                    inputs["controls"]["rank96_oracle"]["metrics"], probes
                )
                passed, gate_evidence = v1.task_metrics_pass(
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
                for name, value in training["final_metrics"].items():
                    mlflow.log_metric(f"{task_profile['task_id']}.{name}", value)
                mlflow.log_metric(f"{task_profile['task_id']}.passes", int(passed))
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
                "frozen_v1_manifest_sha256": common.V1_MANIFEST_SHA256,
                "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
                "implementation_sources": manifest["implementation_sources"],
                "environment": r2c.environment_profile(),
                "profile": common.profile(),
                "authorized_change": manifest["authorized_change"],
                "factorization_diagnostics": v1_manifest["factorization"][
                    "diagnostics"
                ],
                "tasks": task_reports,
                "all_tasks_pass": all_pass,
                "n0_3e_authorized": all_pass,
                "optimizer_steps": sum(task["steps"] for task in common.TASKS),
                "context_feature_cache_bytes_read": 0,
                "frozen_context_factorization_bytes_read": sum(
                    reference.get("byte_count", 0)
                    for reference in v1_manifest["factorization"][
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
                    else "research_first_failing_context_training_boundary_"
                    "without_query_audio"
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
                task_report["task_id"]: task_report["checkpoint"]["weights_sha256"]
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
        freeze(root, arguments.r2d_v1_manifest, arguments.output)
    else:
        if arguments.r2d_v1_manifest is not None:
            raise common.R2DError("run accepts no --r2d-v1-manifest")
        run(root, arguments.manifest, arguments.output)


if __name__ == "__main__":
    main()
