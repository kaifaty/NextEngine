#!/usr/bin/env python3
"""Official runner and hidden scorer for Physical Sound V18 B0."""

from __future__ import annotations

import argparse
import os
import sys
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v18_b0_common as common
import physical_sound_v18_b0_model as model


MUTATION_ORDER = ("wrong-material", "support-swap", "corrupt-scale")
FREQUENCY_MEDIAN_GATE = 20.0
FREQUENCY_P95_GATE = 60.0
DAMPING_MEDIAN_GATE = 0.08
DAMPING_P95_GATE = 0.20


@dataclass(frozen=True)
class Prediction:
    object_id: str
    frequency: np.ndarray
    damping: np.ndarray


def _training_examples(rows: tuple[common.BRow, ...]) -> tuple[model.TrainingExample, ...]:
    return tuple(model.training_example(row) for row in rows)


def _predict_rows(
    artifact: model.RidgeArtifact, rows: tuple[common.BRow, ...]
) -> tuple[Prediction, ...]:
    return tuple(
        Prediction(row.object_id, *model.predict(artifact, model.input_from_row(row)))
        for row in rows
    )


def _prediction_map(predictions: tuple[Prediction, ...]) -> dict[str, Prediction]:
    result = {prediction.object_id: prediction for prediction in predictions}
    if len(result) != len(predictions):
        raise common.B0Error("B0 prediction identity collision")
    return result


def _metric_summary(values: np.ndarray) -> dict[str, float]:
    values = np.asarray(values, dtype=np.float64)
    if values.ndim != 1 or values.size == 0 or not np.isfinite(values).all():
        raise common.B0Error("B0 metric vector is invalid")
    return {
        "maximum": float(np.max(values)),
        "mean": float(np.mean(values)),
        "median": float(np.median(values)),
        "p95": float(np.percentile(values, 95.0)),
    }


def evaluate_rows(
    rows: tuple[common.BRow, ...], predictions: tuple[Prediction, ...]
) -> dict[str, Any]:
    by_id = _prediction_map(predictions)
    frequency_errors: list[np.ndarray] = []
    damping_errors: list[np.ndarray] = []
    object_rows: list[dict[str, Any]] = []
    for row in rows:
        prediction = by_id.get(row.object_id)
        if prediction is None:
            raise common.B0Error(f"missing B0 prediction: {row.object_id}")
        frequency = common.frequency_cents(prediction.frequency, row.frequencies)
        damping = common.damping_relative(prediction.damping, row.damping)
        frequency_errors.append(frequency)
        damping_errors.append(damping)
        object_rows.append(
            {
                "damping_median": float(np.median(damping)),
                "frequency_median_cents": float(np.median(frequency)),
                "object_id": row.object_id,
            }
        )
    return {
        "damping": _metric_summary(np.concatenate(damping_errors)),
        "frequency_cents": _metric_summary(np.concatenate(frequency_errors)),
        "object_endpoints": object_rows,
    }


def _quality_pass(
    prediction_frequency: np.ndarray,
    prediction_damping: np.ndarray,
    truth: common.BRow,
) -> bool:
    if not common.hard_validate(prediction_frequency, prediction_damping):
        return False
    frequency = common.frequency_cents(prediction_frequency, truth.frequencies)
    damping = common.damping_relative(prediction_damping, truth.damping)
    return bool(
        np.median(frequency) <= FREQUENCY_MEDIAN_GATE
        and np.percentile(frequency, 95.0) <= FREQUENCY_P95_GATE
        and np.median(damping) <= DAMPING_MEDIAN_GATE
        and np.percentile(damping, 95.0) <= DAMPING_P95_GATE
    )


def _static_ood_setup(
    train: tuple[common.BRow, ...], development: tuple[common.BRow, ...]
) -> tuple[np.ndarray, np.ndarray, float]:
    train_features = np.stack(
        [model.continuous_ood_features(model.input_from_row(row)) for row in train]
    )
    minimum = np.min(train_features, axis=0)
    maximum = np.max(train_features, axis=0)
    if np.any(maximum <= minimum):
        raise common.B0Error("B0 OOD train range is degenerate")
    scores = [
        _static_ood_score(model.input_from_row(row), minimum, maximum)
        for row in development
    ]
    threshold = max(0.25, 1.25 * max(scores))
    return minimum, maximum, float(threshold)


def _static_ood_score(
    value: model.BaselineInput, minimum: np.ndarray, maximum: np.ndarray
) -> float:
    features = model.continuous_ood_features(value)
    scale = maximum - minimum
    if scale.shape != (2,) or np.any(scale <= 0.0):
        raise common.B0Error("B0 OOD scale is invalid")
    below = np.maximum((minimum - features) / scale, 0.0)
    above = np.maximum((features - maximum) / scale, 0.0)
    return float(np.max(np.maximum(below, above)))


def _mutated_input(row: common.BRow, mutation: str) -> model.BaselineInput:
    original = model.input_from_row(row)
    if mutation == "wrong-material":
        material_index = common.MATERIAL_ORDER.index(row.material)
        material = common.MATERIAL_ORDER[(material_index + 1) % len(common.MATERIAL_ORDER)]
        properties = common.MATERIALS[material]
        return replace(
            original,
            object_id=f"{row.object_id}-wrong-material",
            material=material,
            frequency_scale=(
                properties["wave_speed"] * row.wall_m / (row.length_m * row.length_m)
            ),
            damping_scale=properties["decay"],
        )
    if mutation == "support-swap":
        support = common.SUPPORT_ORDER[1 - common.SUPPORT_ORDER.index(row.support)]
        return replace(
            original,
            object_id=f"{row.object_id}-support-swap",
            support=support,
        )
    if mutation == "corrupt-scale":
        length_m = row.length_m * 1.7
        material = common.MATERIALS[row.material]
        return replace(
            original,
            object_id=f"{row.object_id}-corrupt-scale",
            length_m=length_m,
            slenderness=row.wall_m / length_m,
            frequency_scale=(
                material["wave_speed"] * row.wall_m / (length_m * length_m)
            ),
        )
    raise common.B0Error(f"unknown B0 mutation: {mutation}")


def _valid_ood(
    rows: tuple[common.BRow, ...],
    minimum: np.ndarray,
    maximum: np.ndarray,
    threshold: float,
) -> dict[str, Any]:
    scores = [
        _static_ood_score(model.input_from_row(row), minimum, maximum) for row in rows
    ]
    rejected = sum(score > threshold for score in scores)
    return {
        "count": len(scores),
        "fraction_rejected": rejected / len(scores),
        "maximum_score": max(scores),
        "rejected": rejected,
    }


def _mutation_result(
    artifact: model.RidgeArtifact,
    rows: tuple[common.BRow, ...],
    minimum: np.ndarray,
    maximum: np.ndarray,
    threshold: float,
) -> dict[str, Any]:
    records: list[dict[str, Any]] = []
    for row in rows:
        for mutation in MUTATION_ORDER:
            changed = _mutated_input(row, mutation)
            frequency, damping = model.predict(artifact, changed)
            score = _static_ood_score(changed, minimum, maximum)
            quality_reject = not _quality_pass(frequency, damping, row)
            records.append(
                {
                    "mutation": mutation,
                    "object_id": row.object_id,
                    "ood_reject": score > threshold,
                    "ood_score": score,
                    "quality_reject": quality_reject,
                    "rejected": bool(score > threshold or quality_reject),
                }
            )
    return {
        "count": len(records),
        "records": records,
        "rejected": sum(record["rejected"] for record in records),
    }


def _absolute_gates(metrics: dict[str, Any]) -> dict[str, bool]:
    return {
        "damping_median": metrics["damping"]["median"] <= DAMPING_MEDIAN_GATE,
        "damping_p95": metrics["damping"]["p95"] <= DAMPING_P95_GATE,
        "frequency_median": (
            metrics["frequency_cents"]["median"] <= FREQUENCY_MEDIAN_GATE
        ),
        "frequency_p95": metrics["frequency_cents"]["p95"] <= FREQUENCY_P95_GATE,
    }


def _prediction_array(
    rows: tuple[common.BRow, ...], predictions: tuple[Prediction, ...]
) -> np.ndarray:
    by_id = _prediction_map(predictions)
    return np.stack(
        [
            np.stack((by_id[row.object_id].frequency, by_id[row.object_id].damping))
            for row in rows
        ]
    )


def _hard_prediction_gate(predictions: tuple[Prediction, ...]) -> bool:
    return all(
        common.hard_validate(prediction.frequency, prediction.damping)
        for prediction in predictions
    )


def build_run() -> tuple[dict[str, bytes], dict[str, Any]]:
    environment = common.verify_protocol_and_environment()
    implementation = common.implementation_hashes()
    train = common.generate_training_rows()
    fresh = common.generate_fresh_rows()
    if {common.row_signature(row) for row in fresh} & common.v17_opened_g_signatures():
        raise common.B0Error("fresh B row overlaps opened V17 G")

    artifact = model.fit_ridge(
        _training_examples(train), common.PROTOCOL_SHA256, implementation
    )
    model_files = model.encode_artifact(artifact)
    decoded = model.decode_artifact(model_files)
    predictions = _predict_rows(decoded, fresh)
    direct_predictions = _predict_rows(artifact, fresh)
    if not np.array_equal(
        _prediction_array(fresh, predictions),
        _prediction_array(fresh, direct_predictions),
    ):
        raise common.B0Error("B0 serialized artifact changed predictions")

    development = tuple(row for row in fresh if row.role == "development")
    interpolation = tuple(row for row in fresh if row.role == "test-interpolation")
    transfer = tuple(row for row in fresh if row.role == "test-scale-transfer")
    test = interpolation + transfer
    prediction_by_id = _prediction_map(predictions)
    development_predictions = tuple(prediction_by_id[row.object_id] for row in development)
    interpolation_predictions = tuple(prediction_by_id[row.object_id] for row in interpolation)
    transfer_predictions = tuple(prediction_by_id[row.object_id] for row in transfer)
    test_predictions = interpolation_predictions + transfer_predictions

    minimum, maximum, threshold = _static_ood_setup(train, development)
    overall_metrics = evaluate_rows(test, test_predictions)
    interpolation_metrics = evaluate_rows(interpolation, interpolation_predictions)
    transfer_metrics = evaluate_rows(transfer, transfer_predictions)
    valid_ood_overall = _valid_ood(test, minimum, maximum, threshold)
    valid_ood_interpolation = _valid_ood(interpolation, minimum, maximum, threshold)
    valid_ood_transfer = _valid_ood(transfer, minimum, maximum, threshold)
    mutations = _mutation_result(artifact, interpolation, minimum, maximum, threshold)

    gates: dict[str, bool] = {
        "hard_predictions": _hard_prediction_gate(test_predictions),
        "mutations_72_of_72": mutations["count"] == 72
        and mutations["rejected"] == 72,
        "serialized_roundtrip": True,
        "valid_ood_interpolation": valid_ood_interpolation["fraction_rejected"] <= 0.10,
        "valid_ood_overall": valid_ood_overall["fraction_rejected"] <= 0.10,
        "valid_ood_scale_transfer": valid_ood_transfer["fraction_rejected"] <= 0.10,
        "zero_access": all(value == 0 for value in common.ZERO_ACCESS.values()),
    }
    for prefix, metrics in (
        ("overall", overall_metrics),
        ("interpolation", interpolation_metrics),
        ("scale_transfer", transfer_metrics),
    ):
        for name, passed in _absolute_gates(metrics).items():
            gates[f"{prefix}_{name}"] = passed
    single_run_pass = all(gates.values())

    report = {
        "access": common.ZERO_ACCESS,
        "decision": (
            "B0_SINGLE_RUN_PASS_REPEAT_PENDING"
            if single_run_pass
            else "B0_CAPABILITY_REJECT"
        ),
        "environment": environment,
        "gates": gates,
        "metrics": {
            "development_is_calibration_only": True,
            "interpolation": interpolation_metrics,
            "overall_test": overall_metrics,
            "scale_transfer": transfer_metrics,
        },
        "mutations": mutations,
        "ood": {
            "continuous_feature_order": ["log_aspect", "log_slenderness"],
            "development_maximum_distance": float(
                max(
                    _static_ood_score(model.input_from_row(row), minimum, maximum)
                    for row in development
                )
            ),
            "threshold": threshold,
            "train_maximum": maximum,
            "train_minimum": minimum,
            "valid_interpolation": valid_ood_interpolation,
            "valid_overall": valid_ood_overall,
            "valid_scale_transfer": valid_ood_transfer,
        },
        "repeat_gate": "REQUIRES_SECOND_INDEPENDENT_FRESH_ROOT",
        "schema": common.REPORT_SCHEMA,
        "single_run_pass": single_run_pass,
        "study_id": common.STUDY_ID,
    }
    corpus_bytes = common.canonical_json(
        {
            "fresh": [row.record() for row in fresh],
            "fresh_identity_root": common.identity_root(fresh),
            "parent_protocol_sha256": common.PARENT_PROTOCOL_SHA256,
            "schema": "nextengine.experimental-physical-sound-v18-b0.corpus.v1",
            "training": [row.record() for row in train],
            "training_identity_root": common.identity_root(train),
        }
    )
    prediction_bytes = common.array_bytes(_prediction_array(fresh, predictions))
    report_bytes = common.canonical_json(report)
    payload_files = {
        **model_files,
        "corpus.json": corpus_bytes,
        "predictions.npy": prediction_bytes,
        "report.json": report_bytes,
    }
    manifest = {
        "access": common.ZERO_ACCESS,
        "artifact_files": {
            name: common.sha256_bytes(value)
            for name, value in sorted(payload_files.items())
        },
        "counts": {"fresh": len(fresh), "training": len(train)},
        "implementation_hashes": implementation,
        "parent_protocol_sha256": common.PARENT_PROTOCOL_SHA256,
        "protocol_sha256": common.PROTOCOL_SHA256,
        "revision": common.REVISION,
        "schema": common.MANIFEST_SCHEMA,
        "study_id": common.STUDY_ID,
    }
    files = {**payload_files, "manifest.json": common.canonical_json(manifest)}
    return files, report


def run(output: Path) -> dict[str, Any]:
    staging, destination = common.prepare_output(output)
    try:
        files, report = build_run()
        common.write_files(staging, files)
        observed = common.directory_file_map(staging)
        if len(observed) != len(files):
            raise common.B0Error("B0 output file count changed")
        common.publish_output(staging, destination)
    except BaseException:
        common.abandon_output(staging)
        raise
    return {
        "decision": report["decision"],
        "file_count": len(files),
        "output": destination.as_posix(),
        "single_run_pass": report["single_run_pass"],
        "tree_digest": common.tree_digest(common.directory_file_map(destination)),
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    run_parser = commands.add_parser("run", help="execute one frozen B0 run")
    run_parser.add_argument("--output", required=True, type=Path)
    compare = commands.add_parser("compare", help="compare two completed B0 roots")
    compare.add_argument("--left", required=True, type=Path)
    compare.add_argument("--right", required=True, type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    if arguments.command == "run":
        result = run(arguments.output)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0 if result["single_run_pass"] else 2
    if arguments.command == "compare":
        result = common.compare_directories(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0 if result["byte_identical"] else 1
    raise common.B0Error(f"unknown B0 command: {arguments.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except common.B0Error as error:
        sys.stderr.write(f"B0_ERROR: {error}\n")
        raise SystemExit(3) from error
