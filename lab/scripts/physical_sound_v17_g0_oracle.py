#!/usr/bin/env python3
"""Run the frozen V17 G0 scale-separated global-mode oracle."""

from __future__ import annotations

import argparse
import shutil
from dataclasses import replace
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v17_g0_common as common
import physical_sound_v17_g0_model as neural


CONTROL_ORDER = (
    "material_support_mean",
    "nearest_train_row",
    "dimensionless_degree2_ridge",
)
MUTATION_ORDER = ("wrong_material", "support_swap", "corrupt_scale")


def _row_metrics(
    row: common.GRow, frequencies: np.ndarray, damping: np.ndarray
) -> dict[str, Any]:
    frequency = common.frequency_cents(frequencies, row.frequencies)
    damping_error = common.damping_relative(damping, row.damping)
    return {
        "cell": row.cell,
        "damping_median": float(np.median(damping_error)),
        "damping_p95": float(np.quantile(damping_error, 0.95)),
        "frequency_median_cents": float(np.median(frequency)),
        "frequency_p95_cents": float(np.quantile(frequency, 0.95)),
        "object_id": row.object_id,
        "stable": common.hard_validate(frequencies, damping),
        "stratum": row.stratum,
    }


def _aggregate(rows: list[dict[str, Any]]) -> dict[str, float]:
    return {
        "damping_mean_object_median": float(
            np.mean([row["damping_median"] for row in rows])
        ),
        "damping_median": float(
            np.median([row["damping_median"] for row in rows])
        ),
        "damping_p95": float(np.quantile([row["damping_p95"] for row in rows], 0.95)),
        "frequency_mean_object_median_cents": float(
            np.mean([row["frequency_median_cents"] for row in rows])
        ),
        "frequency_median_cents": float(
            np.median([row["frequency_median_cents"] for row in rows])
        ),
        "frequency_p95_cents": float(
            np.quantile([row["frequency_p95_cents"] for row in rows], 0.95)
        ),
    }


def _polynomial_design(value: np.ndarray) -> np.ndarray:
    value = np.atleast_2d(np.asarray(value, dtype=np.float64))
    columns = [np.ones(value.shape[0], dtype=np.float64)]
    columns.extend(value[:, index] for index in range(value.shape[1]))
    for left in range(value.shape[1]):
        for right in range(left, value.shape[1]):
            columns.append(value[:, left] * value[:, right])
    return np.column_stack(columns)


class Controls:
    def __init__(self, train: tuple[common.GRow, ...]) -> None:
        self.train = train
        self.raw_normalizer = common.fit_normalizer(
            neural.raw_features(row) for row in train
        )
        candidate_normalizer = common.fit_normalizer(
            neural.candidate_features(row) for row in train
        )
        self.candidate_normalizer = candidate_normalizer
        x = np.stack(
            [candidate_normalizer.apply(neural.candidate_features(row)) for row in train]
        )
        design = _polynomial_design(x)
        target = np.stack(
            [
                np.concatenate(
                    (
                        np.log(row.frequencies / row.frequency_scale),
                        np.log(row.damping / row.damping_scale),
                    )
                )
                for row in train
            ]
        )
        gram = design.T @ design + 1.0e-6 * np.eye(design.shape[1])
        self.ridge = np.linalg.solve(gram, design.T @ target)

    def predict(self, row: common.GRow) -> dict[str, tuple[np.ndarray, np.ndarray]]:
        matching = tuple(
            item
            for item in self.train
            if item.material == row.material and item.support == row.support
        )
        mean_frequency_ratio = np.mean(
            [item.frequencies / item.frequency_scale for item in matching], axis=0
        )
        mean_damping_multiplier = np.mean(
            [item.damping / item.damping_scale for item in matching], axis=0
        )
        target_raw = self.raw_normalizer.apply(neural.raw_features(row))
        train_raw = np.stack(
            [self.raw_normalizer.apply(neural.raw_features(item)) for item in self.train]
        )
        nearest = self.train[int(np.argmin(np.linalg.norm(train_raw - target_raw, axis=1)))]
        candidate = self.candidate_normalizer.apply(neural.candidate_features(row))
        ridge_value = _polynomial_design(candidate[None, :])[0] @ self.ridge
        ridge_frequency = np.exp(ridge_value[: common.MODE_COUNT]) * row.frequency_scale
        ridge_damping = np.exp(ridge_value[common.MODE_COUNT :]) * row.damping_scale
        return {
            "material_support_mean": (
                mean_frequency_ratio * row.frequency_scale,
                mean_damping_multiplier * row.damping_scale,
            ),
            "nearest_train_row": (nearest.frequencies, nearest.damping),
            "dimensionless_degree2_ridge": (ridge_frequency, ridge_damping),
        }


def _static_ood_setup(
    train: tuple[common.GRow, ...], development: tuple[common.GRow, ...]
) -> tuple[np.ndarray, np.ndarray, float]:
    train_feature = np.stack([neural.continuous_ood_features(row) for row in train])
    minimum = np.min(train_feature, axis=0)
    maximum = np.max(train_feature, axis=0)
    development_score = np.asarray(
        [_static_ood_score(row, minimum, maximum) for row in development]
    )
    threshold = max(0.25, 1.25 * float(np.max(development_score)))
    return minimum, maximum, threshold


def _static_ood_score(
    row: common.GRow, minimum: np.ndarray, maximum: np.ndarray
) -> float:
    feature = neural.continuous_ood_features(row)
    outside = np.maximum(np.maximum(minimum - feature, feature - maximum), 0.0)
    scale = maximum - minimum
    terms = np.where(
        scale > 1.0e-12,
        outside / np.maximum(scale, 1.0e-12),
        np.where(outside > 0.0, np.inf, 0.0),
    )
    return float(np.max(terms))


def _quality_reject(metrics: dict[str, Any]) -> bool:
    return bool(
        not metrics["stable"]
        or metrics["frequency_median_cents"] > 20.0
        or metrics["frequency_p95_cents"] > 60.0
        or metrics["damping_median"] > 0.08
        or metrics["damping_p95"] > 0.20
    )


def _mutated_row(row: common.GRow, mutation: str) -> common.GRow:
    if mutation == "wrong_material":
        material = common.MATERIAL_ORDER[(row.material_index + 1) % 3]
        return replace(row, material=material)
    if mutation == "support_swap":
        support = common.SUPPORT_ORDER[1 - row.support_index]
        return replace(row, support=support)
    if mutation == "corrupt_scale":
        length = 1.7 * row.length_m
        return replace(
            row,
            length_m=length,
            slenderness=row.wall_m / length,
        )
    raise common.G0Error(f"unknown G0 mutation: {mutation}")


def _write_model(
    staging: Path,
    family: str,
    seed: int,
    model: neural.GlobalHead,
) -> tuple[str, str]:
    relative = f"models/{family}-seed-{seed}.nmodel"
    path = staging / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = common.model_parameter_bytes(model.state_dict())
    path.write_bytes(payload)
    return relative, common.sha256_bytes(payload)


def _write_array(staging: Path, relative: str, value: np.ndarray) -> str:
    path = staging / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = common.array_bytes(np.asarray(value, dtype=np.float64))
    path.write_bytes(payload)
    return common.sha256_bytes(payload)


def _train(
    train: tuple[common.GRow, ...],
) -> tuple[
    neural.FamilyStatistics,
    tuple[neural.GlobalHead, ...],
    neural.FamilyStatistics,
    tuple[neural.GlobalHead, ...],
    dict[str, Any],
]:
    candidate_statistics = neural.fit_statistics(train, "scale_separated")
    raw_statistics = neural.fit_statistics(train, "raw_dimensional")
    candidate_models = []
    raw_models = []
    summaries: dict[str, Any] = {"raw_dimensional": [], "scale_separated": []}
    for seed in common.SEEDS:
        model, summary = neural.train_member(train, candidate_statistics, seed)
        candidate_models.append(model)
        summaries["scale_separated"].append(summary)
    for seed in common.SEEDS:
        model, summary = neural.train_member(train, raw_statistics, seed)
        raw_models.append(model)
        summaries["raw_dimensional"].append(summary)
    return (
        candidate_statistics,
        tuple(candidate_models),
        raw_statistics,
        tuple(raw_models),
        summaries,
    )


def build(output: Path) -> Path:
    resolved, staging = common.prepare_output(output)
    try:
        environment = common.environment_identity(require_exact=True)
        implementation_hashes = common.implementation_hashes()
        protocol = common.repository_root() / common.PROTOCOL_PATH
        if not protocol.is_file():
            raise common.G0Error("frozen P0a protocol is missing")
        corpus = common.generate_corpus()
        train = tuple(row for row in corpus if row.role == "train")
        development = tuple(row for row in corpus if row.role == "development")
        interpolation = tuple(
            row for row in corpus if row.role == "test-interpolation"
        )
        transfer = tuple(row for row in corpus if row.role == "test-scale-transfer")
        test = interpolation + transfer
        (
            candidate_statistics,
            candidate_models,
            raw_statistics,
            raw_models,
            training_summaries,
        ) = _train(train)
        controls = Controls(train)
        ood_minimum, ood_maximum, ood_threshold = _static_ood_setup(
            train, development
        )

        model_hashes: dict[str, str] = {}
        for family, models in (
            ("scale-separated", candidate_models),
            ("raw-dimensional", raw_models),
        ):
            for seed, model in zip(common.SEEDS, models, strict=True):
                relative, digest = _write_model(staging, family, seed, model)
                model_hashes[relative] = digest

        candidate_rows: list[dict[str, Any]] = []
        raw_rows: list[dict[str, Any]] = []
        control_rows: dict[str, list[dict[str, Any]]] = {
            name: [] for name in CONTROL_ORDER
        }
        prediction_matrix = []
        finite_stable = True
        valid_ood_count = 0
        for row in test:
            candidate = neural.predict_ensemble(
                candidate_models, row, candidate_statistics
            )
            raw = neural.predict_ensemble(raw_models, row, raw_statistics)
            candidate_metric = _row_metrics(
                row, candidate["frequencies"], candidate["damping"]
            )
            raw_metric = _row_metrics(row, raw["frequencies"], raw["damping"])
            candidate_rows.append(candidate_metric)
            raw_rows.append(raw_metric)
            finite_stable = finite_stable and candidate_metric["stable"]
            finite_stable = finite_stable and all(
                common.hard_validate(
                    candidate["frequency_members"][index],
                    candidate["damping_members"][index],
                )
                for index in range(len(common.SEEDS))
            )
            valid_ood_count += int(
                _static_ood_score(row, ood_minimum, ood_maximum) > ood_threshold
            )
            control_predictions = controls.predict(row)
            for name, (frequencies, damping) in control_predictions.items():
                control_rows[name].append(_row_metrics(row, frequencies, damping))
            prediction_matrix.append(
                np.concatenate(
                    (
                        np.asarray(
                            [row.cell, float(row.stratum == "scale_transfer")]
                        ),
                        row.frequencies,
                        row.damping,
                        candidate["frequencies"],
                        candidate["damping"],
                        candidate["frequency_members"].reshape(-1),
                        candidate["damping_members"].reshape(-1),
                        raw["frequencies"],
                        raw["damping"],
                        raw["frequency_members"].reshape(-1),
                        raw["damping_members"].reshape(-1),
                        *[
                            np.concatenate(control_predictions[name])
                            for name in CONTROL_ORDER
                        ],
                    )
                )
            )

        mutation_rows = []
        mutation_records = []
        for row in interpolation:
            for mutation_index, mutation in enumerate(MUTATION_ORDER):
                mutated = _mutated_row(row, mutation)
                prediction = neural.predict_ensemble(
                    candidate_models, mutated, candidate_statistics
                )
                metrics = _row_metrics(
                    row, prediction["frequencies"], prediction["damping"]
                )
                ood_score = _static_ood_score(
                    mutated, ood_minimum, ood_maximum
                )
                quality_rejected = _quality_reject(metrics)
                ood_rejected = ood_score > ood_threshold
                rejected = quality_rejected or ood_rejected
                mutation_records.append(
                    {
                        "cell": row.cell,
                        "mutation": mutation,
                        "object_id": row.object_id,
                        "ood_rejected": ood_rejected,
                        "ood_score": ood_score,
                        "quality_rejected": quality_rejected,
                        "rejected": rejected,
                    }
                )
                mutation_rows.append(
                    np.asarray(
                        [
                            row.cell,
                            mutation_index,
                            metrics["frequency_median_cents"],
                            metrics["frequency_p95_cents"],
                            metrics["damping_median"],
                            metrics["damping_p95"],
                            ood_score,
                            float(quality_rejected),
                            float(ood_rejected),
                            float(rejected),
                        ]
                    )
                )

        candidate_aggregate = _aggregate(candidate_rows)
        raw_aggregate = _aggregate(raw_rows)
        candidate_transfer = _aggregate(candidate_rows[24:])
        raw_transfer = _aggregate(raw_rows[24:])
        control_aggregates = {
            name: _aggregate(rows) for name, rows in control_rows.items()
        }
        best_frequency_control_name, best_frequency_control = min(
            (
                (name, values["frequency_mean_object_median_cents"])
                for name, values in control_aggregates.items()
            ),
            key=lambda item: (item[1], item[0]),
        )
        best_damping_control_name, best_damping_control = min(
            (
                (name, values["damping_mean_object_median"])
                for name, values in control_aggregates.items()
            ),
            key=lambda item: (item[1], item[0]),
        )
        paired_wins = sum(
            candidate_rows[24 + index]["frequency_median_cents"]
            <= raw_rows[24 + index]["frequency_median_cents"]
            and candidate_rows[24 + index]["damping_median"]
            <= raw_rows[24 + index]["damping_median"]
            for index in range(24)
        )
        mutation_rejection_fraction = float(
            np.mean([record["rejected"] for record in mutation_records])
        )
        valid_ood_fraction = valid_ood_count / len(test)

        gates = {
            "damping_median": candidate_aggregate["damping_median"] <= 0.08,
            "damping_p95": candidate_aggregate["damping_p95"] <= 0.20,
            "finite_stable": finite_stable,
            "frequency_median": candidate_aggregate["frequency_median_cents"]
            <= 20.0,
            "frequency_p95": candidate_aggregate["frequency_p95_cents"] <= 60.0,
            "isolation": all(value == 0 for value in common.ZERO_ACCESS.values()),
            "mutation_rejection": mutation_rejection_fraction == 1.0,
            "nonneural_damping_ratio": candidate_aggregate[
                "damping_mean_object_median"
            ]
            <= 0.90 * best_damping_control,
            "nonneural_frequency_ratio": candidate_aggregate[
                "frequency_mean_object_median_cents"
            ]
            <= 0.90 * best_frequency_control,
            "scale_transfer_damping_ratio": candidate_transfer[
                "damping_mean_object_median"
            ]
            <= 0.50 * raw_transfer["damping_mean_object_median"],
            "scale_transfer_frequency_ratio": candidate_transfer[
                "frequency_mean_object_median_cents"
            ]
            <= 0.50 * raw_transfer["frequency_mean_object_median_cents"],
            "scale_transfer_paired_wins": paired_wins >= 20,
        }
        single_execution_pass = all(gates.values())
        decision = (
            "REPEAT_COMPARISON_REQUIRED"
            if single_execution_pass
            else "G0_CAPABILITY_REJECT"
        )

        artifact_hashes = dict(model_hashes)
        artifact_hashes["predictions/test.npy"] = _write_array(
            staging, "predictions/test.npy", np.stack(prediction_matrix)
        )
        artifact_hashes["predictions/mutations.npy"] = _write_array(
            staging, "predictions/mutations.npy", np.stack(mutation_rows)
        )
        corpus_payload = common.canonical_json(
            {
                "rows": [row.record(include_truth=True) for row in corpus],
                "schema": "nextengine.experimental-physical-sound-v17-g0.corpus.v1",
            }
        )
        (staging / "corpus.json").write_bytes(corpus_payload)
        artifact_hashes["corpus.json"] = common.sha256_bytes(corpus_payload)

        report = {
            "access": common.ZERO_ACCESS,
            "authority": {
                "authored_clip_fallback_required": True,
                "candidate_quality_credit_authorized": False,
                "field_or_integration_authorized": False,
                "real_quality_credit_authorized": False,
                "runtime_neural_inference_authorized": False,
            },
            "candidate": {
                "aggregate": candidate_aggregate,
                "objects": candidate_rows,
                "scale_transfer": candidate_transfer,
            },
            "controls": {
                "aggregates": control_aggregates,
                "best_damping": {
                    "name": best_damping_control_name,
                    "value": best_damping_control,
                },
                "best_frequency": {
                    "name": best_frequency_control_name,
                    "value_cents": best_frequency_control,
                },
            },
            "decision": decision,
            "gates": gates,
            "mutation": {
                "case_count": len(mutation_records),
                "cases": mutation_records,
                "rejection_fraction": mutation_rejection_fraction,
            },
            "ood": {
                "continuous_feature_maximum": ood_maximum,
                "continuous_feature_minimum": ood_minimum,
                "threshold": ood_threshold,
                "valid_test_rejection_fraction_diagnostic": valid_ood_fraction,
            },
            "paired_scale_transfer_wins": paired_wins,
            "raw_dimensional": {
                "aggregate": raw_aggregate,
                "objects": raw_rows,
                "scale_transfer": raw_transfer,
            },
            "repeat_exact": {
                "required_complete_executions": 2,
                "status": "EXTERNAL_COMPARISON_REQUIRED",
            },
            "schema": common.REPORT_SCHEMA,
            "single_execution_pass": single_execution_pass,
            "study_id": common.STUDY_ID,
        }
        report_payload = common.canonical_json(report)
        (staging / "report.json").write_bytes(report_payload)
        manifest = {
            "access": common.ZERO_ACCESS,
            "artifacts": artifact_hashes,
            "authority": {
                "authored_clip_fallback_required": True,
                "field_or_integration_authorized": False,
                "real_quality_credit_authorized": False,
                "runtime_neural_inference_authorized": False,
            },
            "corpus_counts": {
                "development": len(development),
                "test_interpolation": len(interpolation),
                "test_scale_transfer": len(transfer),
                "train": len(train),
            },
            "environment": environment,
            "implementation_sha256s": implementation_hashes,
            "model": {
                "raw_statistics": raw_statistics.record(),
                "scale_separated_statistics": candidate_statistics.record(),
                "seeds": common.SEEDS,
                "training_summaries": training_summaries,
                "updates_per_member": common.UPDATES,
            },
            "protocol": {
                "path": common.PROTOCOL_PATH.as_posix(),
                "sha256": common.sha256_file(protocol),
            },
            "report_sha256": common.sha256_bytes(report_payload),
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        (staging / "manifest.json").write_bytes(common.canonical_json(manifest))
        common.publish_output(resolved, staging)
        return resolved
    except BaseException:
        if staging.exists():
            shutil.rmtree(staging)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    result = build(arguments.output)
    print((result / "report.json").read_text(encoding="utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
