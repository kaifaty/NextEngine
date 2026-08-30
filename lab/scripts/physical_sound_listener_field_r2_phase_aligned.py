#!/usr/bin/env python3
"""Train the preregistered propagation-delay-aligned R2 successor."""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Any

import mlflow
import numpy as np

import physical_sound_listener_field_r2 as base

MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned."
    "manifest.v1"
)
PREFLIGHT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "preflight.report.v1"
)
TRAINING_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "training.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "mlflow-lineage.v1"
)
PARENT_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-evaluation.report.v1"
)
REVISION = "green-goblet-fixed-impact-listener-field-r2-phase-aligned-v1"
PARENT_REVISION = "green-goblet-fixed-impact-listener-field-r2-evaluation-v1"
PARENT_DECISION = "RejectListenerField"
SPEED_OF_SOUND_METRES_PER_SECOND = 343.0
GUARD_SAMPLE_MARGIN = 2


class PhaseAlignedError(RuntimeError):
    """The frozen phase-aligned successor boundary failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["preflight", "train"])
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--preflight", type=Path)
    return parser.parse_args()


def phase_profile() -> dict[str, Any]:
    return {
        "profile_id": "listener-field-r2-relative-air-path-phase-v1",
        "coordinate_assumption": (
            "published_impact_and_listener_metric_coordinates_are_co_registered"
        ),
        "distance": "euclidean_listener_minus_impact_metres",
        "reference_distance": "minimum_distance_across_context_and_query_coordinates",
        "speed_of_sound_metres_per_second": SPEED_OF_SOUND_METRES_PER_SECOND,
        "relative_delay": "(distance-reference_distance)/speed_of_sound",
        "alignment": "rfft_times_exp_positive_i_2pi_f_delay_then_irfft",
        "restoration": "rfft_times_exp_negative_i_2pi_f_delay_then_irfft",
        "padding": (
            "input_at_guard_offset_in_smallest_power_of_two_covering_"
            "sample_count_plus_two_guards"
        ),
        "guard_samples": "ceil(max_relative_delay_seconds*sample_rate)+2",
        "output_crop": "original_sample_count_at_guard_offset_after_restoration",
        "query_audio_used_to_choose_profile": False,
    }


def candidate_profile() -> dict[str, Any]:
    profile = base.candidate_profile()
    profile["profile_id"] = "listener-field-r2-tanh-latent-phase-aligned-v1"
    profile["temporal_basis"] = (
        "canonical_sign_centered_context_gram_eigendecomposition_after_frozen_"
        "relative_air_path_phase_alignment"
    )
    profile["phase_alignment"] = phase_profile()
    return profile


def validate_local_sources(root: Path, sources: Any) -> list[dict[str, Any]]:
    if not isinstance(sources, list) or not sources:
        raise PhaseAlignedError("implementation_sources must be a non-empty array")
    records = []
    previous = None
    for source in sources:
        reference = base.require_file_ref(source, "implementation source")
        if previous is not None and previous >= reference["path"]:
            raise PhaseAlignedError("implementation sources must be strictly sorted")
        previous = reference["path"]
        path = (root / reference["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file():
            raise PhaseAlignedError("implementation source must remain repository-local")
        if base.sha256_file(path) != reference["sha256"]:
            raise PhaseAlignedError(f"implementation source changed: {reference['path']}")
        records.append({**reference, "byte_count": path.stat().st_size})
    return records


def validate_manifest(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    expected = {
        "schema",
        "revision",
        "runner_sha256",
        "implementation_sources",
        "parent_rejection",
        "projection",
        "baseline_report",
        "context_bindings",
        "query_row_ids",
        "environment",
        "candidate_profile",
        "mlflow_experiment",
    }
    if set(manifest) != expected:
        raise PhaseAlignedError("phase-aligned manifest fields changed")
    runner = Path(__file__).resolve(strict=True)
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("revision") != REVISION
        or manifest.get("runner_sha256") != base.sha256_file(runner)
    ):
        raise PhaseAlignedError("phase-aligned manifest identity changed")
    if manifest.get("environment") != base.environment_profile():
        raise PhaseAlignedError("phase-aligned environment differs from manifest")
    if manifest.get("candidate_profile") != candidate_profile():
        raise PhaseAlignedError("phase-aligned candidate profile changed")
    if manifest.get("mlflow_experiment") != "nextengine-physical-sound-r2-phase":
        raise PhaseAlignedError("phase-aligned MLflow experiment changed")
    bindings = manifest.get("context_bindings")
    query_ids = manifest.get("query_row_ids")
    if not isinstance(bindings, list) or not isinstance(query_ids, list):
        raise PhaseAlignedError("context bindings and query IDs must be arrays")
    binding_ids = [binding.get("row_id") for binding in bindings]
    if binding_ids != sorted(binding_ids) or len(set(binding_ids)) != len(binding_ids):
        raise PhaseAlignedError("context bindings must be sorted and unique")
    if query_ids != sorted(query_ids) or len(set(query_ids)) != len(query_ids):
        raise PhaseAlignedError("query IDs must be sorted and unique")
    for binding in bindings:
        if set(binding) != {"row_id", "audio"} or not isinstance(
            binding["row_id"], str
        ):
            raise PhaseAlignedError("invalid phase-aligned context binding")
        base.require_file_ref(binding["audio"], "context audio")
    return validate_local_sources(root, manifest["implementation_sources"])


def validate_parent_rejection(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> tuple[bytes, dict[str, Any]]:
    _, data = base.resolve_ref(
        root,
        manifest_path.parent,
        manifest["parent_rejection"],
        "parent R2 rejection",
    )
    report = base.json.loads(data)
    if (
        report.get("schema") != PARENT_REPORT_SCHEMA
        or report.get("revision") != PARENT_REVISION
        or report.get("decision") != PARENT_DECISION
        or report.get("selected_candidate_id") is not None
        or report.get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise PhaseAlignedError("parent rejection lineage changed")
    return data, report


def next_power_of_two(value: int) -> int:
    if value < 1:
        raise PhaseAlignedError("FFT length request must be positive")
    return 1 << (value - 1).bit_length()


def phase_geometry(
    manifest: dict[str, Any], projection: dict[str, Any], sample_rate: int, sample_count: int
) -> dict[str, Any]:
    rows = {row["row_id"]: row for row in projection["rows"]}
    ordered_ids = [binding["row_id"] for binding in manifest["context_bindings"]]
    ordered_ids.extend(manifest["query_row_ids"])
    records = []
    impact_point = None
    for row_id in ordered_ids:
        axes = rows[row_id]["axes"]
        point = np.asarray(axes["listener"]["point_metres"], dtype=np.float64)
        impact = np.asarray(axes["impact"]["point_metres"], dtype=np.float64)
        if point.shape != (3,) or impact.shape != (3,):
            raise PhaseAlignedError("phase alignment requires finite 3D coordinates")
        if not np.isfinite(point).all() or not np.isfinite(impact).all():
            raise PhaseAlignedError("phase alignment coordinates must be finite")
        if impact_point is None:
            impact_point = impact
        elif not np.array_equal(impact_point, impact):
            raise PhaseAlignedError("phase alignment requires one exact impact point")
        records.append(
            {
                "row_id": row_id,
                "distance_metres": float(np.linalg.norm(point - impact)),
            }
        )
    reference = min(record["distance_metres"] for record in records)
    maximum_delay = 0.0
    for record in records:
        delay = (
            record["distance_metres"] - reference
        ) / SPEED_OF_SOUND_METRES_PER_SECOND
        if delay < 0.0 or not math.isfinite(delay):
            raise PhaseAlignedError("relative propagation delay is invalid")
        record["relative_delay_seconds"] = delay
        record["relative_delay_samples"] = delay * sample_rate
        maximum_delay = max(maximum_delay, delay)
    guard = math.ceil(maximum_delay * sample_rate) + GUARD_SAMPLE_MARGIN
    fft_length = next_power_of_two(sample_count + 2 * guard)
    return {
        "profile": phase_profile(),
        "reference_distance_metres": reference,
        "maximum_relative_delay_seconds": maximum_delay,
        "guard_samples": guard,
        "fft_length": fft_length,
        "placement_offset_samples": guard,
        "rows": records,
    }


def delays_for(row_ids: list[str], geometry: dict[str, Any]) -> np.ndarray:
    by_id = {
        record["row_id"]: record["relative_delay_seconds"]
        for record in geometry["rows"]
    }
    return np.asarray([by_id[row_id] for row_id in row_ids], dtype=np.float64)


def align_context(
    signals: np.ndarray,
    delays: np.ndarray,
    sample_rate: int,
    geometry: dict[str, Any],
) -> np.ndarray:
    offset = geometry["placement_offset_samples"]
    length = geometry["fft_length"]
    padded = np.zeros((signals.shape[0], length), dtype=np.float64)
    padded[:, offset : offset + signals.shape[1]] = signals
    frequencies = np.fft.rfftfreq(length, d=1.0 / sample_rate)
    phase = np.exp(2.0j * np.pi * delays[:, None] * frequencies[None, :])
    return np.fft.irfft(np.fft.rfft(padded, axis=1) * phase, n=length, axis=1)


def restore_delay(
    aligned: np.ndarray,
    delays: np.ndarray,
    sample_rate: int,
    sample_count: int,
    geometry: dict[str, Any],
) -> np.ndarray:
    length = geometry["fft_length"]
    if aligned.shape[1] != length:
        raise PhaseAlignedError("aligned prediction FFT grid changed")
    frequencies = np.fft.rfftfreq(length, d=1.0 / sample_rate)
    phase = np.exp(-2.0j * np.pi * delays[:, None] * frequencies[None, :])
    restored = np.fft.irfft(
        np.fft.rfft(aligned, axis=1) * phase, n=length, axis=1
    )
    offset = geometry["placement_offset_samples"]
    return restored[:, offset : offset + sample_count]


def roundtrip_metrics(original: np.ndarray, restored: np.ndarray) -> dict[str, float]:
    error = restored - original
    denominator = float(np.sum(original**2))
    ratio = math.sqrt(float(np.sum(error**2)) / denominator)
    return {
        "maximum_absolute_error": float(np.max(np.abs(error))),
        "normalized_waveform_rmse_db": (
            base.DB_FLOOR if ratio <= 1.0e-12 else 20.0 * math.log10(ratio)
        ),
    }


def protocol_inputs(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    parent_bytes, _ = validate_parent_rejection(root, manifest_path, manifest)
    projection, baseline, projection_bytes, baseline_bytes = (
        base.validate_projection_and_baseline(root, manifest_path, manifest)
    )
    signals, coordinates, sample_rate, context_records = base.read_context(
        root, manifest_path, manifest, projection
    )
    query_coordinates, query_records = base.query_coordinates(manifest, projection)
    geometry = phase_geometry(
        manifest, projection, sample_rate, int(signals.shape[1])
    )
    context_ids = [record["row_id"] for record in context_records]
    context_delays = delays_for(context_ids, geometry)
    aligned = align_context(signals, context_delays, sample_rate, geometry)
    roundtrip = restore_delay(
        aligned, context_delays, sample_rate, int(signals.shape[1]), geometry
    )
    return {
        "parent_bytes": parent_bytes,
        "projection": projection,
        "projection_bytes": projection_bytes,
        "baseline": baseline,
        "baseline_bytes": baseline_bytes,
        "signals": signals,
        "aligned": aligned,
        "coordinates": coordinates,
        "query_coordinates": query_coordinates,
        "sample_rate": sample_rate,
        "context_records": context_records,
        "query_records": query_records,
        "geometry": geometry,
        "context_delays": context_delays,
        "roundtrip": roundtrip_metrics(signals, roundtrip),
    }


def preflight(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    sources: list[dict[str, Any]],
    output: Path,
) -> None:
    inputs = protocol_inputs(root, manifest_path, manifest)
    if inputs["signals"].shape[1] > inputs["sample_rate"] * 30:
        raise PhaseAlignedError("phase-aligned audio exceeds bounded duration")
    report = {
        "schema": PREFLIGHT_SCHEMA,
        "status": "Validated",
        "decision": "PhaseAlignedListenerFieldR2ProtocolFrozen",
        "claim": (
            "PROPAGATION_DELAY_REPRESENTATION_PROTOCOL_ONLY / NO_TRAINED_MODEL_"
            "QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "revision": REVISION,
        "manifest_sha256": base.sha256_bytes(manifest_bytes),
        "runner_sha256": base.sha256_file(Path(__file__).resolve(strict=True)),
        "implementation_sources": sources,
        "parent_rejection_sha256": base.sha256_bytes(inputs["parent_bytes"]),
        "projection_sha256": base.sha256_bytes(inputs["projection_bytes"]),
        "baseline_report_sha256": base.sha256_bytes(inputs["baseline_bytes"]),
        "baseline_metric_profile": inputs["baseline"]["metric_profile"]["id"],
        "candidate_profile": candidate_profile(),
        "environment": base.environment_profile(),
        "sample_rate_hz": inputs["sample_rate"],
        "sample_count": int(inputs["signals"].shape[1]),
        "phase_geometry": inputs["geometry"],
        "alignment_roundtrip": inputs["roundtrip"],
        "context_rows": inputs["context_records"],
        "query_rows_without_audio": inputs["query_records"],
        "optimizer_steps": 0,
        "query_audio_bytes_read": 0,
        "method_holdout_or_shadow_bytes_read": 0,
        "training_authorized_by_this_report": True,
        "quality_or_admission_authorized": False,
    }
    report_bytes = base.canonical_json(report)
    (output / "preflight-report.json").write_bytes(report_bytes)
    print(report_bytes.decode(), end="")


def validate_preflight(
    root: Path,
    manifest_bytes: bytes,
    preflight_path: Path | None,
) -> tuple[bytes, dict[str, Any]]:
    if preflight_path is None:
        raise PhaseAlignedError("train stage requires --preflight")
    path = base.external_file(root, preflight_path, "phase-aligned preflight")
    data, report = base.read_json(path, "phase-aligned preflight")
    if (
        report.get("schema") != PREFLIGHT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "PhaseAlignedListenerFieldR2ProtocolFrozen"
        or report.get("manifest_sha256") != base.sha256_bytes(manifest_bytes)
        or report.get("runner_sha256")
        != base.sha256_file(Path(__file__).resolve(strict=True))
        or report.get("optimizer_steps") != 0
        or report.get("query_audio_bytes_read") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise PhaseAlignedError("phase-aligned preflight lineage changed")
    return data, report


def log_mlflow_candidate(
    tracking_directory: Path,
    experiment_name: str,
    candidate_id: str,
    profile: dict[str, Any],
    training: dict[str, Any],
    output: Path,
    checkpoint: dict[str, Any],
    prediction_records: list[dict[str, Any]],
) -> dict[str, Any]:
    database = tracking_directory / "mlflow.db"
    artifacts = tracking_directory / "artifacts"
    artifacts.mkdir(exist_ok=True)
    tracking_uri = f"sqlite:///{database}"
    mlflow.set_tracking_uri(tracking_uri)
    client = mlflow.tracking.MlflowClient(tracking_uri=tracking_uri)
    experiment = client.get_experiment_by_name(experiment_name)
    if experiment is None:
        experiment_id = client.create_experiment(
            experiment_name, artifact_location=artifacts.as_uri()
        )
    else:
        experiment_id = experiment.experiment_id
    with mlflow.start_run(experiment_id=experiment_id, run_name=candidate_id) as run:
        mlflow.log_params(
            {
                "revision": REVISION,
                "candidate_id": candidate_id,
                "rank": profile["rank"],
                "seed": base.SEED,
                "epochs": base.EPOCHS,
                "learning_rate": base.LEARNING_RATE,
                "weight_decay": base.WEIGHT_DECAY,
                "smoothness_weight": base.SMOOTHNESS_WEIGHT,
                "phase_profile": phase_profile()["profile_id"],
                "speed_of_sound_mps": SPEED_OF_SOUND_METRES_PER_SECOND,
                "dtype": "float64",
                "device": "cpu",
            }
        )
        for key, value in training.items():
            if isinstance(value, (int, float)) and math.isfinite(float(value)):
                mlflow.log_metric(key, float(value))
        for relative in [
            checkpoint["descriptor_file"],
            checkpoint["weights_file"],
            *(record["prediction_file"] for record in prediction_records),
        ]:
            mlflow.log_artifact(str(output / relative))
        return {
            "candidate_id": candidate_id,
            "run_id": run.info.run_id,
            "experiment_id": run.info.experiment_id,
        }


def train(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    sources: list[dict[str, Any]],
    preflight_path: Path | None,
    output: Path,
) -> None:
    preflight_bytes, _ = validate_preflight(root, manifest_bytes, preflight_path)
    inputs = protocol_inputs(root, manifest_path, manifest)
    base.configure_determinism()
    (output / "predictions").mkdir()
    tracking_directory = output / "mlflow"
    tracking_directory.mkdir()
    query_ids = [record["row_id"] for record in inputs["query_records"]]
    query_delays = delays_for(query_ids, inputs["geometry"])
    candidate_reports = []
    mlflow_runs = []
    for profile in base.CANDIDATES:
        model, aligned_predictions, training = base.train_candidate(
            inputs["aligned"],
            inputs["coordinates"],
            inputs["query_coordinates"],
            profile["rank"],
        )
        predictions = restore_delay(
            aligned_predictions,
            query_delays,
            inputs["sample_rate"],
            int(inputs["signals"].shape[1]),
            inputs["geometry"],
        )
        checkpoint = base.write_checkpoint(output, profile["candidate_id"], model)
        prediction_records = []
        for row, samples in zip(inputs["query_records"], predictions, strict=True):
            relative = (
                Path("predictions")
                / profile["candidate_id"]
                / f"{row['row_id']}.wav"
            )
            path = output / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            wav = base.encode_wav(samples, inputs["sample_rate"])
            path.write_bytes(wav)
            prediction_records.append(
                {
                    "row_id": row["row_id"],
                    "listener_point_metres": row["listener_point_metres"],
                    "relative_delay_seconds": float(
                        query_delays[len(prediction_records)]
                    ),
                    "prediction_file": str(relative),
                    "prediction_sha256": base.sha256_bytes(wav),
                    "prediction_byte_count": len(wav),
                    "pre_cook_peak_abs": float(np.max(np.abs(samples))),
                }
            )
        report = {
            **profile,
            "training": training,
            "checkpoint": checkpoint,
            "predictions": prediction_records,
        }
        candidate_reports.append(report)
        mlflow_runs.append(
            log_mlflow_candidate(
                tracking_directory,
                manifest["mlflow_experiment"],
                profile["candidate_id"],
                profile,
                training,
                output,
                checkpoint,
                prediction_records,
            )
        )
    report = {
        "schema": TRAINING_SCHEMA,
        "status": "Trained",
        "decision": "PhaseAlignedCandidatesCookedForFrozenDevelopmentEvaluation",
        "claim": (
            "PHASE_ALIGNED_FIXED_IMPACT_TRAINING_AND_PCM_COOK_ONLY / NO_"
            "QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "revision": REVISION,
        "manifest_sha256": base.sha256_bytes(manifest_bytes),
        "preflight_sha256": base.sha256_bytes(preflight_bytes),
        "runner_sha256": base.sha256_file(Path(__file__).resolve(strict=True)),
        "implementation_sources": sources,
        "parent_rejection_sha256": base.sha256_bytes(inputs["parent_bytes"]),
        "projection_sha256": base.sha256_bytes(inputs["projection_bytes"]),
        "baseline_report_sha256": base.sha256_bytes(inputs["baseline_bytes"]),
        "candidate_profile": candidate_profile(),
        "environment": base.environment_profile(),
        "sample_rate_hz": inputs["sample_rate"],
        "sample_count": int(inputs["signals"].shape[1]),
        "phase_geometry": inputs["geometry"],
        "alignment_roundtrip": inputs["roundtrip"],
        "context_rows": inputs["context_records"],
        "candidates": candidate_reports,
        "query_audio_bytes_read": 0,
        "method_holdout_or_shadow_bytes_read": 0,
        "quality_or_admission_authorized": False,
    }
    report_bytes = base.canonical_json(report)
    (output / "training-report.json").write_bytes(report_bytes)
    lineage = {
        "schema": MLFLOW_SCHEMA,
        "mlflow_version": mlflow.__version__,
        "tracking_database": "mlflow/mlflow.db",
        "artifact_directory": "mlflow/artifacts",
        "experiment_name": manifest["mlflow_experiment"],
        "runs": mlflow_runs,
        "deterministic_training_report_sha256": base.sha256_bytes(report_bytes),
    }
    (output / "mlflow-lineage.json").write_bytes(base.canonical_json(lineage))
    print(report_bytes.decode(), end="")


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve(strict=True).parents[2]
    manifest_path = base.external_file(root, arguments.manifest, "phase manifest")
    manifest_bytes, manifest = base.read_json(manifest_path, "phase manifest")
    sources = validate_manifest(root, manifest)
    output = base.external_output(root, arguments.output)
    try:
        if arguments.stage == "preflight":
            if arguments.preflight is not None:
                raise PhaseAlignedError("preflight stage does not accept --preflight")
            preflight(
                root, manifest_path, manifest_bytes, manifest, sources, output
            )
        else:
            train(
                root,
                manifest_path,
                manifest_bytes,
                manifest,
                sources,
                arguments.preflight,
                output,
            )
    except Exception:
        if output.is_dir() and not any(output.iterdir()):
            output.rmdir()
        raise


if __name__ == "__main__":
    main()
