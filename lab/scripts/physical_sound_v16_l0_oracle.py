#!/usr/bin/env python3
"""Run the frozen Physical Sound V16 L0 known-truth neural oracle."""

from __future__ import annotations

import argparse
from dataclasses import replace
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v16_l0_common as common
import physical_sound_v16_l0_model as neural


def _percentile_by_object(
    rows: list[dict[str, Any]], key: str, percentile: float
) -> float:
    per_object = [float(np.quantile(row[key], percentile)) for row in rows]
    return float(np.quantile(per_object, percentile))


def _median_by_object(rows: list[dict[str, Any]], key: str) -> float:
    return float(np.median([float(np.median(row[key])) for row in rows]))


def _mean_scalar(rows: list[dict[str, Any]], key: str) -> float:
    return float(np.mean([float(row[key]) for row in rows]))


def _spectrum_endpoint(
    data: common.ObjectData,
    frequencies: np.ndarray,
    damping: np.ndarray,
    gains: np.ndarray,
    truth_pcm: np.ndarray | None = None,
) -> float:
    truth = (
        common.render_modal(
            data.frequencies,
            data.damping,
            data.gains[data.query_indices],
        )
        if truth_pcm is None
        else truth_pcm
    )
    prediction = common.render_modal(frequencies, damping, gains)
    active = np.max(np.abs(truth), axis=1) > 1.0e-12
    if not np.any(active):
        raise common.L0Error("control spectrum has no non-nodal truth query")
    return float(
        np.mean(common.multiresolution_spectrum_rmse(prediction[active], truth[active]))
    )


def _quality_reject(metrics: dict[str, Any]) -> bool:
    return bool(
        np.median(metrics["frequency_cents"]) > 30.0
        or np.quantile(metrics["frequency_cents"], 0.95) > 80.0
        or np.median(metrics["damping_relative"]) > 0.12
        or np.quantile(metrics["damping_relative"], 0.95) > 0.30
        or metrics["gain_nrmse"] > 0.40
        or metrics["waveform_nrmse_mean"] > 0.20
        or metrics["spectrum_rmse_db_mean"] > 2.5
        or metrics["envelope_nrmse_p95"] > 0.15
    )


def _write_model(
    directory: Path,
    family: str,
    seed: int,
    model: neural.StructuredModalField,
) -> tuple[str, str]:
    relative = f"models/{family}-seed-{seed}.nmodel"
    path = directory / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = common.model_parameter_bytes(model.state_dict())
    path.write_bytes(payload)
    return relative, common.sha256_bytes(payload)


def _write_array(directory: Path, relative: str, value: np.ndarray) -> str:
    path = directory / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = common.array_bytes(np.asarray(value, dtype=np.float64))
    path.write_bytes(payload)
    return common.sha256_bytes(payload)


def _prediction_artifacts(
    directory: Path,
    data: common.ObjectData,
    candidate: dict[str, np.ndarray],
    ablation: dict[str, np.ndarray],
    all_candidate_gains: np.ndarray,
    ood_scores: np.ndarray,
) -> dict[str, str]:
    query = data.query_indices
    query_matrix = np.column_stack(
        (
            query,
            data.gains[query],
            candidate["gains"],
            candidate["gain_members"].transpose(1, 0, 2).reshape(query.size, -1),
            ablation["gains"],
            ood_scores,
        )
    )
    global_matrix = np.vstack(
        (
            np.concatenate((data.frequencies, data.damping)),
            np.concatenate((candidate["frequencies"], candidate["damping"])),
            *[
                np.concatenate(
                    (
                        candidate["frequency_members"][index],
                        candidate["damping_members"][index],
                    )
                )
                for index in range(len(common.SEEDS))
            ],
            np.concatenate((ablation["frequencies"], ablation["damping"])),
        )
    )
    all_gain_matrix = np.column_stack(
        (
            np.arange(data.vertices.shape[0]),
            data.gains,
            all_candidate_gains,
        )
    )
    result = {}
    for suffix, value in (
        ("query", query_matrix),
        ("global", global_matrix),
        ("all-gains", all_gain_matrix),
    ):
        relative = f"predictions/{data.spec.object_id}-{suffix}.npy"
        result[relative] = _write_array(directory, relative, value)
    return result


def _train_families(
    train: tuple[common.ObjectData, ...],
) -> tuple[
    neural.TrainingStatistics,
    tuple[neural.StructuredModalField, ...],
    neural.TrainingStatistics,
    tuple[neural.StructuredModalField, ...],
    dict[str, Any],
]:
    candidate_statistics = neural.fit_training_statistics(train, True)
    ablation_statistics = neural.fit_training_statistics(train, False)
    candidate_models = []
    ablation_models = []
    summaries: dict[str, Any] = {"candidate": [], "geometry_agnostic": []}
    for seed in common.SEEDS:
        model, summary = neural.train_member(train, candidate_statistics, seed)
        candidate_models.append(model)
        summaries["candidate"].append(summary)
    for seed in common.SEEDS:
        model, summary = neural.train_member(train, ablation_statistics, seed)
        ablation_models.append(model)
        summaries["geometry_agnostic"].append(summary)
    return (
        candidate_statistics,
        tuple(candidate_models),
        ablation_statistics,
        tuple(ablation_models),
        summaries,
    )


def _development_ood_raw(
    development: tuple[common.ObjectData, ...],
    models: tuple[neural.StructuredModalField, ...],
    statistics: neural.TrainingStatistics,
    train_static_minimum: np.ndarray,
    train_static_maximum: np.ndarray,
) -> tuple[np.ndarray, dict[str, dict[str, np.ndarray]]]:
    rows = []
    predictions: dict[str, dict[str, np.ndarray]] = {}
    for data in development:
        prediction = neural.predict_ensemble(models, data, statistics)
        predictions[data.spec.object_id] = prediction
        rows.append(
            common.ood_raw_components(
                data,
                data.query_indices,
                prediction["gain_members"],
                statistics.gain_scale,
                common.static_features(data.spec),
                train_static_minimum,
                train_static_maximum,
            )
        )
    return np.concatenate(rows, axis=0), predictions


def _evaluate_controls(
    train: tuple[common.ObjectData, ...],
    data: common.ObjectData,
    static_normalizer: common.Normalizer,
) -> tuple[dict[str, float], dict[str, float]]:
    gains = common.gain_controls(data)
    globals_ = common.global_controls(train, data, static_normalizer)
    truth_gains = data.gains[data.query_indices]
    truth_pcm = common.render_modal(data.frequencies, data.damping, truth_gains)
    gain_metrics = {
        name: float(common.nrmse(value, truth_gains)) for name, value in gains.items()
    }
    spectrum_metrics = {}
    for global_name, (frequencies, damping) in globals_.items():
        for gain_name, value in gains.items():
            key = f"{global_name}+{gain_name}"
            spectrum_metrics[key] = _spectrum_endpoint(
                data, frequencies, damping, value, truth_pcm
            )
    return gain_metrics, spectrum_metrics


def _mutation_suite(
    test: tuple[common.ObjectData, ...],
    models: tuple[neural.StructuredModalField, ...],
    statistics: neural.TrainingStatistics,
    clean_predictions: dict[str, dict[str, np.ndarray]],
    development_raw: np.ndarray,
    train_static_minimum: np.ndarray,
    train_static_maximum: np.ndarray,
) -> dict[str, Any]:
    hard_rejections = 0
    ordinary_cases: list[dict[str, Any]] = []
    collapsed_scores = []
    material_cycle = {"Steel": "Wood", "Wood": "Glass", "Glass": "Steel"}

    for data in test:
        corrupt_frequency = clean_predictions[data.spec.object_id]["frequencies"][::-1]
        corrupt_damping = clean_predictions[data.spec.object_id]["damping"].copy()
        corrupt_damping[0] = -abs(corrupt_damping[0])
        hard_rejections += int(
            not common.hard_validate_modes(corrupt_frequency, corrupt_damping)
        )

        clean = clean_predictions[data.spec.object_id]
        shuffled_gains = np.roll(clean["gains"], 17, axis=0)
        shuffled_metrics = common.object_metrics(
            data, clean["frequencies"], clean["damping"], shuffled_gains
        )
        ordinary_cases.append(
            {
                "mutation": "query_contact_cyclic_shift_17",
                "object_id": data.spec.object_id,
                "quality_rejected": _quality_reject(shuffled_metrics),
                "ood_rejected": False,
                "rejected": _quality_reject(shuffled_metrics),
            }
        )

        wrong_material_spec = replace(
            data.spec, material=material_cycle[data.spec.material]
        )
        wrong_material = neural.predict_ensemble(
            models, data, statistics, spec_override=wrong_material_spec
        )
        wrong_material_metrics = common.object_metrics(
            data,
            wrong_material["frequencies"],
            wrong_material["damping"],
            wrong_material["gains"],
        )
        wrong_material_raw = common.ood_raw_components(
            data,
            data.query_indices,
            wrong_material["gain_members"],
            statistics.gain_scale,
            common.static_features(wrong_material_spec),
            train_static_minimum,
            train_static_maximum,
        )
        wrong_material_ood, _, threshold = common.calibrate_ood(
            development_raw, wrong_material_raw
        )
        material_quality = _quality_reject(wrong_material_metrics)
        material_ood = bool(np.any(wrong_material_ood > threshold))
        ordinary_cases.append(
            {
                "mutation": "cyclic_wrong_material",
                "object_id": data.spec.object_id,
                "quality_rejected": material_quality,
                "ood_rejected": material_ood,
                "rejected": material_quality or material_ood,
            }
        )

        wrong_scale_spec = replace(data.spec, length_m=data.spec.length_m * 1.6)
        wrong_scale = neural.predict_ensemble(
            models, data, statistics, spec_override=wrong_scale_spec
        )
        wrong_scale_metrics = common.object_metrics(
            data,
            wrong_scale["frequencies"],
            wrong_scale["damping"],
            wrong_scale["gains"],
        )
        wrong_scale_raw = common.ood_raw_components(
            data,
            data.query_indices,
            wrong_scale["gain_members"],
            statistics.gain_scale,
            common.static_features(wrong_scale_spec),
            train_static_minimum,
            train_static_maximum,
        )
        wrong_scale_ood, _, threshold = common.calibrate_ood(
            development_raw, wrong_scale_raw
        )
        scale_quality = _quality_reject(wrong_scale_metrics)
        scale_ood = bool(np.any(wrong_scale_ood > threshold))
        ordinary_cases.append(
            {
                "mutation": "metric_scale_times_1_6",
                "object_id": data.spec.object_id,
                "quality_rejected": scale_quality,
                "ood_rejected": scale_ood,
                "rejected": scale_quality or scale_ood,
            }
        )

        collapsed_context = np.flatnonzero(data.uv[:, 0] <= 0.0)
        collapsed_query = np.flatnonzero(data.uv[:, 0] >= 0.5)
        if collapsed_context.size < 20 or collapsed_query.size == 0:
            raise common.L0Error(
                f"{data.spec.object_id} collapsed coverage fixture is too small"
            )
        collapsed = neural.predict_ensemble(
            models,
            data,
            statistics,
            output_indices=collapsed_query,
            context_indices=collapsed_context,
        )
        collapsed_raw = common.ood_raw_components(
            data,
            collapsed_query,
            collapsed["gain_members"],
            statistics.gain_scale,
            common.static_features(data.spec),
            train_static_minimum,
            train_static_maximum,
            context_indices=collapsed_context,
        )
        score, _, threshold = common.calibrate_ood(development_raw, collapsed_raw)
        collapsed_scores.append(score > threshold)

    ordinary_rejected = sum(bool(row["rejected"]) for row in ordinary_cases)
    collapsed = np.concatenate(collapsed_scores)
    return {
        "collapsed_coverage_query_count": int(collapsed.size),
        "collapsed_coverage_rejection_fraction": float(np.mean(collapsed)),
        "hard_case_count": len(test),
        "hard_rejection_fraction": hard_rejections / len(test),
        "ordinary_case_count": len(ordinary_cases),
        "ordinary_cases": ordinary_cases,
        "ordinary_rejection_fraction": ordinary_rejected / len(ordinary_cases),
    }


def build(output: Path) -> Path:
    resolved, staging = common.prepare_output(output)
    try:
        environment = common.environment_identity(require_exact=True)
        implementation_hashes = common.implementation_hashes()
        protocol = common.repository_root() / common.PROTOCOL_PATH
        if not protocol.is_file():
            raise common.L0Error("frozen L0a protocol is missing")
        protocol_sha256 = common.sha256_file(protocol)
        corpus = common.generate_corpus()
        train = tuple(item for item in corpus if item.spec.role == "train")
        development = tuple(
            item for item in corpus if item.spec.role == "development"
        )
        test = tuple(item for item in corpus if item.spec.role == "test")

        (
            candidate_statistics,
            candidate_models,
            ablation_statistics,
            ablation_models,
            training_summaries,
        ) = _train_families(train)

        model_hashes: dict[str, str] = {}
        for family, models in (
            ("candidate", candidate_models),
            ("geometry-agnostic", ablation_models),
        ):
            for seed, model in zip(common.SEEDS, models, strict=True):
                relative, digest = _write_model(staging, family, seed, model)
                model_hashes[relative] = digest

        train_static = np.stack(
            [common.static_features(item.spec) for item in train]
        )
        train_static_minimum = np.min(train_static, axis=0)
        train_static_maximum = np.max(train_static, axis=0)
        development_raw, _ = _development_ood_raw(
            development,
            candidate_models,
            candidate_statistics,
            train_static_minimum,
            train_static_maximum,
        )
        _, ood_component_maxima, ood_threshold = common.calibrate_ood(
            development_raw, development_raw
        )

        candidate_rows: list[dict[str, Any]] = []
        ablation_rows: list[dict[str, Any]] = []
        continuity_rows = []
        valid_ood_rows = []
        control_gain_rows: dict[str, list[float]] = {}
        control_spectrum_rows: dict[str, list[float]] = {}
        prediction_hashes: dict[str, str] = {}
        clean_predictions: dict[str, dict[str, np.ndarray]] = {}
        finite_stable = True

        for data in test:
            candidate = neural.predict_ensemble(
                candidate_models, data, candidate_statistics
            )
            ablation = neural.predict_ensemble(
                ablation_models, data, ablation_statistics
            )
            all_candidate = neural.predict_ensemble(
                candidate_models,
                data,
                candidate_statistics,
                output_indices=np.arange(data.vertices.shape[0]),
            )
            clean_predictions[data.spec.object_id] = candidate
            candidate_metrics = common.object_metrics(
                data,
                candidate["frequencies"],
                candidate["damping"],
                candidate["gains"],
            )
            ablation_metrics = common.object_metrics(
                data,
                ablation["frequencies"],
                ablation["damping"],
                ablation["gains"],
            )
            candidate_metrics["object_id"] = data.spec.object_id
            ablation_metrics["object_id"] = data.spec.object_id
            candidate_rows.append(candidate_metrics)
            ablation_rows.append(ablation_metrics)
            continuity_rows.append(
                common.continuity_metric(data, all_candidate["gains"])
            )
            finite_stable = finite_stable and common.hard_validate_modes(
                candidate["frequencies"], candidate["damping"]
            )
            finite_stable = finite_stable and (
                candidate_metrics["zero_prediction_for_active_truth_count"] == 0
            )
            finite_stable = finite_stable and common.hard_validate_modes(
                ablation["frequencies"], ablation["damping"]
            )

            raw_ood = common.ood_raw_components(
                data,
                data.query_indices,
                candidate["gain_members"],
                candidate_statistics.gain_scale,
                common.static_features(data.spec),
                train_static_minimum,
                train_static_maximum,
            )
            ood_score, _, observed_threshold = common.calibrate_ood(
                development_raw, raw_ood
            )
            if observed_threshold != ood_threshold:
                raise common.L0Error("OOD threshold changed during test evaluation")
            valid_ood_rows.append(ood_score > ood_threshold)
            prediction_hashes.update(
                _prediction_artifacts(
                    staging,
                    data,
                    candidate,
                    ablation,
                    all_candidate["gains"],
                    ood_score,
                )
            )

            gain_controls, spectrum_controls = _evaluate_controls(
                train, data, candidate_statistics.static_normalizer
            )
            for name, value in gain_controls.items():
                control_gain_rows.setdefault(name, []).append(value)
            for name, value in spectrum_controls.items():
                control_spectrum_rows.setdefault(name, []).append(value)

        mutation = _mutation_suite(
            test,
            candidate_models,
            candidate_statistics,
            clean_predictions,
            development_raw,
            train_static_minimum,
            train_static_maximum,
        )

        frequency_median = _median_by_object(candidate_rows, "frequency_cents")
        frequency_p95 = _percentile_by_object(
            candidate_rows, "frequency_cents", 0.95
        )
        damping_median = _median_by_object(candidate_rows, "damping_relative")
        damping_p95 = _percentile_by_object(
            candidate_rows, "damping_relative", 0.95
        )
        candidate_gain = _mean_scalar(candidate_rows, "gain_nrmse")
        candidate_gain_maximum = max(row["gain_nrmse"] for row in candidate_rows)
        ablation_gain = _mean_scalar(ablation_rows, "gain_nrmse")
        candidate_spectrum = float(
            np.median([row["spectrum_rmse_db_mean"] for row in candidate_rows])
        )
        ablation_spectrum = float(
            np.median([row["spectrum_rmse_db_mean"] for row in ablation_rows])
        )
        best_control_gain_name, best_control_gain = min(
            (
                (name, float(np.mean(values)))
                for name, values in control_gain_rows.items()
            ),
            key=lambda item: (item[1], item[0]),
        )
        best_control_spectrum_name, best_control_spectrum = min(
            (
                (name, float(np.median(values)))
                for name, values in control_spectrum_rows.items()
            ),
            key=lambda item: (item[1], item[0]),
        )
        waveform_mean = _mean_scalar(candidate_rows, "waveform_nrmse_mean")
        envelope_p95 = float(
            np.quantile(
                [row["envelope_nrmse_p95"] for row in candidate_rows], 0.95
            )
        )
        continuity_p99 = float(np.quantile(continuity_rows, 0.99))
        valid_ood = np.concatenate(valid_ood_rows)
        valid_ood_fraction = float(np.mean(valid_ood))
        maximum_peak_error = max(
            float(np.max(row["frequency_cents"])) for row in candidate_rows
        )

        gates = {
            "classical_gain_ratio": candidate_gain <= 0.98 * best_control_gain,
            "classical_spectrum_ratio": candidate_spectrum
            <= 0.98 * best_control_spectrum,
            "damping_median": damping_median <= 0.12,
            "damping_p95": damping_p95 <= 0.30,
            "finite_stable": finite_stable,
            "frequency_median": frequency_median <= 30.0,
            "frequency_p95": frequency_p95 <= 80.0,
            "gain_every_object": candidate_gain_maximum <= 0.40,
            "gain_mean": candidate_gain <= 0.25,
            "geometry_ablation_gain_ratio": candidate_gain <= 0.95 * ablation_gain,
            "geometry_ablation_spectrum_ratio": candidate_spectrum
            <= 0.95 * ablation_spectrum,
            "isolation": all(value == 0 for value in common.ZERO_ACCESS.values()),
            "modal_peaks": maximum_peak_error <= 80.0,
            "mutation_collapsed_coverage": mutation[
                "collapsed_coverage_rejection_fraction"
            ]
            >= 0.95,
            "mutation_hard": mutation["hard_rejection_fraction"] == 1.0,
            "mutation_ordinary": mutation["ordinary_rejection_fraction"] >= 0.95,
            "surface_continuity": continuity_p99 <= 0.20,
            "valid_test_ood": valid_ood_fraction <= 0.10,
            "waveform_envelope": envelope_p95 <= 0.15,
            "waveform_mean": waveform_mean <= 0.20,
            "waveform_spectrum": candidate_spectrum <= 2.5,
        }
        single_execution_pass = all(gates.values())
        decision = (
            "REPEAT_COMPARISON_REQUIRED"
            if single_execution_pass
            else "L0_CAPABILITY_REJECT"
        )

        report = {
            "access": common.ZERO_ACCESS,
            "authority": {
                "authored_clip_fallback_required": True,
                "candidate_quality_credit_authorized": False,
                "product_schema_authorized": False,
                "real_sound_quality_credit_authorized": False,
                "runtime_neural_inference_authorized": False,
            },
            "candidate_objects": candidate_rows,
            "controls": {
                "best_gain": {
                    "name": best_control_gain_name,
                    "value": best_control_gain,
                },
                "best_spectrum": {
                    "name": best_control_spectrum_name,
                    "value_db": best_control_spectrum,
                },
                "gain_by_control": {
                    name: float(np.mean(values))
                    for name, values in sorted(control_gain_rows.items())
                },
                "spectrum_by_control": {
                    name: float(np.median(values))
                    for name, values in sorted(control_spectrum_rows.items())
                },
            },
            "decision": decision,
            "gates": gates,
            "geometry_agnostic_objects": ablation_rows,
            "metrics": {
                "candidate_gain_nrmse_mean": candidate_gain,
                "candidate_gain_nrmse_maximum": candidate_gain_maximum,
                "candidate_spectrum_rmse_db_median": candidate_spectrum,
                "continuity_p99": continuity_p99,
                "damping_relative_median": damping_median,
                "damping_relative_p95": damping_p95,
                "envelope_nrmse_p95": envelope_p95,
                "frequency_cents_maximum": maximum_peak_error,
                "frequency_cents_median": frequency_median,
                "frequency_cents_p95": frequency_p95,
                "geometry_agnostic_gain_nrmse_mean": ablation_gain,
                "geometry_agnostic_spectrum_rmse_db_median": ablation_spectrum,
                "valid_test_ood_fraction": valid_ood_fraction,
                "waveform_nrmse_mean": waveform_mean,
            },
            "mutation": mutation,
            "ood": {
                "development_component_maxima": ood_component_maxima,
                "threshold": ood_threshold,
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
        report_sha256 = common.sha256_bytes(report_payload)

        artifact_hashes = {**model_hashes, **prediction_hashes}
        manifest = {
            "access": common.ZERO_ACCESS,
            "artifacts": artifact_hashes,
            "authority": {
                "authored_clip_fallback_required": True,
                "candidate_quality_credit_authorized": False,
                "cooker_promotion_authorized": False,
                "demo_integration_authorized": False,
                "runtime_neural_inference_authorized": False,
            },
            "corpus": [
                {
                    **item.spec.record(),
                    "context_count": int(item.context_indices.size),
                    "mesh_sha256": item.mesh_sha256,
                    "query_count": int(item.query_indices.size),
                    "vertex_count": int(item.vertices.shape[0]),
                }
                for item in corpus
            ],
            "environment": environment,
            "implementation_sha256s": implementation_hashes,
            "model": {
                "candidate_statistics": candidate_statistics.record(),
                "geometry_agnostic_statistics": ablation_statistics.record(),
                "learning_rate_end": common.LEARNING_RATE_END,
                "learning_rate_start": common.LEARNING_RATE_START,
                "seeds": common.SEEDS,
                "training_summaries": training_summaries,
                "updates_per_member": common.UPDATES,
                "weight_decay": common.WEIGHT_DECAY,
            },
            "protocol": {
                "path": common.PROTOCOL_PATH.as_posix(),
                "sha256": protocol_sha256,
            },
            "report_sha256": report_sha256,
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        (staging / "manifest.json").write_bytes(common.canonical_json(manifest))
        common.publish_output(resolved, staging)
        return resolved
    except BaseException:
        if staging.exists():
            import shutil

            shutil.rmtree(staging)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    result = build(arguments.output)
    report = (result / "report.json").read_text(encoding="utf-8")
    print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
