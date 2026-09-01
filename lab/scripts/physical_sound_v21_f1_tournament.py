#!/usr/bin/env python3
"""Official train/development tournament for Physical Sound V21 F1a."""

from __future__ import annotations

import argparse
import math
import resource
import sys
import time
from dataclasses import replace
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import numpy as np
import physical_sound_v19_f0_model as f0_model
import physical_sound_v19_f0_oracle as f0_oracle
import physical_sound_v21_f1_common as common
import physical_sound_v21_f1_model as model

METHOD_ORDER = (
    "candidate",
    "context-mean",
    "nearest-intrinsic",
    "prior-euclidean-rbf",
    "prior-only",
    "raw-continuous",
    "raw-euclidean-rbf",
    "raw-geodesic-rbf",
    "raw-harmonic",
)
MUTATION_ORDER = ("query-shift", "context-shift", "alternating-context-sign")
STRUCTURAL_ORDER = (
    "duplicate-context",
    "out-of-range",
    "missing-context",
    "mesh-identity-mismatch",
    "topology-identity-mismatch",
    "role-identity-mismatch",
    "unknown-basis-id",
    "changed-lambda",
    "nonfinite-coefficient",
    "insufficient-budget",
)
OUTPUT_FILES = {
    "access-ledger.json",
    "corpus.json",
    "geometry.npz",
    "manifest.json",
    "metrics.jsonl",
    "models.npz",
    "predictions.npz",
    "report.json",
    "selection.json",
}


def _coverage_summary(item: common.FieldObject) -> dict[str, Any]:
    decisions = item.coverage_decisions
    global_values = {
        value["global_fill"] for value in decisions if value["global_fill"] is not None
    }
    global_fill = None if len(global_values) != 1 else float(next(iter(global_values)))
    global_pass = bool(
        global_fill is not None
        and global_fill <= common.f0_common.coverage_common.GLOBAL_INTRINSIC_THRESHOLD
        and all(
            value["reason"]
            not in ("OOD_CONTEXT_BUDGET", "OOD_DISCONNECTED", "OOD_INTRINSIC_FILL")
            for value in decisions
        )
    )
    return {
        "accepted_fraction": float(item.accepted_query.size / item.query.size),
        "global_fill": global_fill,
        "global_pass": global_pass,
        "local_false_ood_fraction": float(item.rejected_query.size / item.query.size),
        "object_id": item.row.object_id,
        "role": item.row.role,
    }


def _metric_row(
    item: common.FieldObject, prediction: np.ndarray, method: str, candidate_id: str
) -> dict[str, Any]:
    return {
        **f0_model.metrics(item, prediction),
        "candidate_id": candidate_id,
        "method": method,
        "object_id": item.row.object_id,
        "physical_group_id": item.row.physical_group_id,
        "role": item.row.role,
        "schema": common.METRIC_SCHEMA,
        "topology": item.row.topology,
        "type": "field-metric",
    }


def _summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
    if not rows or any(row["gain_nrmse"] is None for row in rows):
        return {
            "complete": False,
            "edge_gradient_p99_max": None,
            "edge_gradient_p99_mean": None,
            "gain_nrmse_max": None,
            "gain_nrmse_mean": None,
        }
    gain = np.asarray([row["gain_nrmse"] for row in rows], dtype=np.float64)
    gradient = np.asarray([row["edge_gradient_p99"] for row in rows], dtype=np.float64)
    return {
        "complete": True,
        "edge_gradient_p99_max": float(np.max(gradient)),
        "edge_gradient_p99_mean": float(np.mean(gradient)),
        "gain_nrmse_max": float(np.max(gain)),
        "gain_nrmse_mean": float(np.mean(gain)),
    }


def _topology_means(rows: list[dict[str, Any]], field: str) -> dict[str, float | None]:
    result = {}
    for topology in common.f0_common.TOPOLOGY_ORDER:
        values = [row[field] for row in rows if row["topology"] == topology]
        result[topology] = None if not values else float(np.mean(values))
    return result


def _evaluate_candidate(
    trained: model.TrainingResult,
    objects: tuple[common.FieldObject, ...],
) -> tuple[dict[str, tuple[np.ndarray, ...]], dict[str, list[dict[str, Any]]]]:
    assert trained.spec is not None
    predictions: dict[str, list[np.ndarray]] = {name: [] for name in METHOD_ORDER}
    metrics: dict[str, list[dict[str, Any]]] = {name: [] for name in METHOD_ORDER}
    for item in objects:
        values = model.compatible_predictions(
            trained.model, trained.gain_scale, item, trained.spec
        )
        if tuple(values) != METHOD_ORDER:
            raise common.F1Error("F1 method order changed")
        for name, prediction in values.items():
            predictions[name].append(prediction)
            metrics[name].append(
                _metric_row(item, prediction, name, trained.spec.candidate_id)
            )
    return (
        {name: tuple(values) for name, values in predictions.items()},
        metrics,
    )


def _evaluate_single_method(
    candidate_id: str,
    method: str,
    objects: tuple[common.FieldObject, ...],
    predictions: tuple[np.ndarray, ...],
) -> list[dict[str, Any]]:
    return [
        _metric_row(item, prediction, method, candidate_id)
        for item, prediction in zip(objects, predictions, strict=True)
    ]


def _truth_at_probes(item: common.FieldObject) -> np.ndarray:
    grid = np.linspace(-0.75, 0.75, 7, dtype=np.float64)
    probes = np.asarray([(u, v) for v in grid for u in grid], dtype=np.float64)
    _xyz, normals, _curvature = model.analytic_surface(item.row, probes)
    proxy = SimpleNamespace(uv=probes, row=item.row, vertex_count=probes.shape[0])
    return common.f0_common.gain_truth(proxy, normals)


def _continuous_remesh(
    trained: model.TrainingResult,
    objects: tuple[common.FieldObject, ...],
    metric_rows: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    assert trained.spec is not None
    by_group: dict[str, list[int]] = {}
    for index, item in enumerate(objects):
        by_group.setdefault(item.row.physical_group_id, []).append(index)
    rows = []
    for group, indices in sorted(by_group.items()):
        if len(indices) != 2:
            raise common.F1Error(f"F1 remesh group changed: {group}")
        primary_index = next(index for index in indices if not objects[index].is_twin)
        twin_index = next(index for index in indices if objects[index].is_twin)
        primary = objects[primary_index]
        twin = objects[twin_index]
        primary_probe = model.direct_probe_prediction(
            trained.model, trained.gain_scale, primary, trained.spec
        )
        twin_probe = model.direct_probe_prediction(
            trained.model, trained.gain_scale, twin, trained.spec
        )
        truth_probe = _truth_at_probes(primary)
        denominator = float(np.sqrt(np.mean(truth_probe**2)))
        disagreement = float(
            np.sqrt(np.mean((primary_probe - twin_probe) ** 2)) / denominator
        )
        primary_metric = float(metric_rows[primary_index]["gain_nrmse"])
        twin_metric = float(metric_rows[twin_index]["gain_nrmse"])
        drift = abs(twin_metric - primary_metric) / max(primary_metric, 1.0e-12)
        rows.append(
            {
                "candidate_id": trained.spec.candidate_id,
                "gain_metric_drift": float(drift),
                "physical_group_id": group,
                "probe_disagreement_nrmse": disagreement,
                "schema": common.METRIC_SCHEMA,
                "topology": primary.row.topology,
                "type": "remesh",
            }
        )
    return rows


def _f0_control(
    trained: f0_model.PriorNetwork,
    gain_scale: np.ndarray,
    objects: tuple[common.FieldObject, ...],
) -> tuple[tuple[np.ndarray, ...], list[dict[str, Any]], list[dict[str, Any]]]:
    predictions = tuple(
        f0_model.predict_methods(trained, gain_scale, item)["candidate"]
        for item in objects
    )
    metrics = _evaluate_single_method(
        "frozen-f0-harmonic", "candidate", objects, predictions
    )
    remesh = f0_oracle._remesh_metrics(objects, predictions, metrics)
    normalized = [
        {
            **row,
            "candidate_id": "frozen-f0-harmonic",
            "schema": common.METRIC_SCHEMA,
            "type": "remesh",
        }
        for row in remesh
    ]
    return predictions, metrics, normalized


def _paired_control(
    trained: model.TrainingResult,
    objects: tuple[common.FieldObject, ...],
) -> tuple[tuple[np.ndarray, ...], list[dict[str, Any]]]:
    predictions = tuple(
        model.predict_paired_harmonic(trained.model, trained.gain_scale, item)
        for item in objects
    )
    return predictions, _evaluate_single_method(
        "paired-harmonic-v1", "candidate", objects, predictions
    )


def _mutation_rows(
    trained: model.TrainingResult,
    objects: tuple[common.FieldObject, ...],
    clean: tuple[np.ndarray, ...],
) -> list[dict[str, Any]]:
    assert trained.spec is not None
    rows = []
    for index, item in enumerate(objects):
        if item.is_twin:
            continue
        for mutation in MUTATION_ORDER:
            if mutation == "query-shift":
                prediction = clean[index].copy()
                prediction[item.accepted_query] = np.roll(
                    prediction[item.accepted_query], 17, axis=0
                )
            else:
                observed = item.gains[item.context].copy()
                if mutation == "context-shift":
                    observed = np.roll(observed, 17, axis=0)
                else:
                    observed[:, 1::2] *= -1.0
                prediction = model.predict_continuous(
                    trained.model,
                    trained.gain_scale,
                    item,
                    trained.spec,
                    observed,
                )
            values = f0_model.metrics(item, prediction)
            rows.append(
                {
                    **values,
                    "candidate_id": trained.spec.candidate_id,
                    "mutation": mutation,
                    "object_id": item.row.object_id,
                    "quality_reject": bool(
                        values["gain_nrmse"] > 0.40
                        or values["edge_gradient_p99"] > 0.80
                    ),
                    "schema": common.METRIC_SCHEMA,
                    "topology": item.row.topology,
                    "type": "numerical-mutation",
                }
            )
    return rows


def _coverage_mutation(
    item: common.FieldObject, mutation: str
) -> common.f0_common.coverage_oracle.CoverageInput:
    value = item.coverage_input
    if mutation == "duplicate-context":
        context = value.context.copy()
        context[-1] = context[0]
        return replace(
            value, context=context, context_hash=common.identity_hash(context)
        )
    if mutation == "out-of-range":
        context = value.context.copy()
        context[-1] = value.vertex_count
        return replace(
            value, context=context, context_hash=common.identity_hash(context)
        )
    if mutation == "missing-context":
        required = common.f0_common.coverage_common.required_context_count(
            value.vertex_count
        )
        context = value.context[: required - 1]
        return replace(
            value,
            context=context,
            context_hash=common.identity_hash(context),
            declared_context_count=int(context.size),
        )
    if mutation == "mesh-identity-mismatch":
        return replace(value, mesh_hash="0" * 64)
    if mutation == "topology-identity-mismatch":
        changed = common.f0_common.TOPOLOGY_ORDER[
            (common.f0_common.TOPOLOGY_ORDER.index(value.topology) + 1)
            % len(common.f0_common.TOPOLOGY_ORDER)
        ]
        return replace(value, topology=changed)
    raise common.F1Error(f"unknown F1 coverage mutation: {mutation}")


def _structural_rows(
    objects: tuple[common.FieldObject, ...], spec: model.CandidateSpec
) -> list[dict[str, Any]]:
    rows = []
    coverage_mutations = STRUCTURAL_ORDER[:5]
    local_mutations = STRUCTURAL_ORDER[5:]
    for item in objects:
        if item.is_twin:
            continue
        for mutation in coverage_mutations:
            value = _coverage_mutation(item, mutation)
            decisions = common.f0_common.coverage_oracle.evaluate(
                item.analysis, value, "composite"
            )
            rejected = bool(
                decisions
                and all(
                    decision["reason"] == "OOD_CONTEXT_BUDGET"
                    and not decision["distance_evaluated"]
                    for decision in decisions
                )
            )
            rows.append(
                {
                    "mutation": mutation,
                    "object_id": item.row.object_id,
                    "pre_inference_reject": rejected,
                    "reason": None if not decisions else decisions[0]["reason"],
                    "schema": common.METRIC_SCHEMA,
                    "type": "structural-mutation",
                }
            )
        for mutation in local_mutations:
            basis_size = (1 + 2 * spec.order) ** 2
            arguments: dict[str, Any] = {}
            if mutation == "role-identity-mismatch":
                arguments["declared_role"] = "train"
            elif mutation == "unknown-basis-id":
                arguments["declared_basis_revision"] = "unknown-basis"
            elif mutation == "changed-lambda":
                arguments["declared_regularization"] = spec.regularization * 10.0
            elif mutation == "nonfinite-coefficient":
                coefficient = np.zeros(
                    (basis_size, common.MODE_COUNT), dtype=np.float64
                )
                coefficient[0, 0] = np.nan
                arguments["coefficient"] = coefficient
            elif mutation == "insufficient-budget":
                arguments["available_model_bytes"] = model.PARAMETER_BYTES - 1
            try:
                model.validate_request(
                    item, spec, expected_role="development", **arguments
                )
            except common.F1Error as error:
                rejected = True
                reason = str(error)
            else:
                rejected = False
                reason = None
            rows.append(
                {
                    "candidate_id": spec.candidate_id,
                    "mutation": mutation,
                    "object_id": item.row.object_id,
                    "pre_inference_reject": rejected,
                    "reason": reason,
                    "schema": common.METRIC_SCHEMA,
                    "type": "structural-mutation",
                }
            )
    return rows


def _paired_wins(metrics: dict[str, list[dict[str, Any]]]) -> int:
    wins = 0
    for candidate, prior, euclidean in zip(
        metrics["candidate"],
        metrics["prior-only"],
        metrics["prior-euclidean-rbf"],
        strict=True,
    ):
        if candidate["object_id"].endswith("-twin"):
            continue
        if all(
            candidate[field] < prior[field] and candidate[field] < euclidean[field]
            for field in ("gain_nrmse", "edge_gradient_p99")
        ):
            wins += 1
    return wins


def _candidate_gates(
    metrics: dict[str, list[dict[str, Any]]],
    remesh: list[dict[str, Any]],
    mutations: list[dict[str, Any]],
    structural: list[dict[str, Any]],
    f0_metrics: list[dict[str, Any]],
    f0_remesh: list[dict[str, Any]],
    coverage: list[dict[str, Any]],
    roundtrip_exact: bool,
) -> dict[str, bool]:
    summaries = {name: _summary(rows) for name, rows in metrics.items()}
    candidate = summaries["candidate"]
    f0 = _summary(f0_metrics)
    pool = [
        summaries[name]
        for name in (
            "context-mean",
            "nearest-intrinsic",
            "prior-only",
            "raw-continuous",
            "raw-euclidean-rbf",
            "raw-geodesic-rbf",
            "raw-harmonic",
        )
    ]
    complete = bool(
        candidate["complete"]
        and f0["complete"]
        and all(value["complete"] for value in pool)
    )
    best_gain = min(value["gain_nrmse_mean"] for value in pool) if complete else None
    best_gradient = (
        min(value["edge_gradient_p99_mean"] for value in pool) if complete else None
    )
    topology_gain = _topology_means(metrics["candidate"], "gain_nrmse")
    topology_gradient = _topology_means(metrics["candidate"], "edge_gradient_p99")
    mutation_counts = {
        name: sum(row["quality_reject"] for row in mutations if row["mutation"] == name)
        for name in MUTATION_ORDER
    }
    max_drift = max(row["gain_metric_drift"] for row in remesh)
    f0_max_drift = max(row["gain_metric_drift"] for row in f0_remesh)
    drift_limit = 1.0e-12 if f0_max_drift == 0.0 else 0.90 * f0_max_drift
    fallback_complete = all(
        item["fallback_complete"] for item in coverage if "fallback_complete" in item
    )
    return {
        "absolute_gain": bool(
            complete
            and candidate["gain_nrmse_mean"] <= 0.25
            and candidate["gain_nrmse_max"] <= 0.40
        ),
        "absolute_gradient": bool(
            complete
            and candidate["edge_gradient_p99_mean"] <= 0.45
            and candidate["edge_gradient_p99_max"] <= 0.80
        ),
        "best_compatible_ratio": bool(
            complete
            and candidate["gain_nrmse_mean"] <= 0.80 * best_gain
            and candidate["edge_gradient_p99_mean"] <= 0.80 * best_gradient
        ),
        "coverage_global": all(value["global_pass"] for value in coverage),
        "coverage_local": all(
            value["local_false_ood_fraction"] <= 0.05 for value in coverage
        ),
        "fallback_complete": fallback_complete,
        "finite_complete": bool(
            complete
            and all(
                math.isfinite(float(row[field]))
                for rows in metrics.values()
                for row in rows
                for field in ("gain_nrmse", "edge_gradient_p99")
            )
        ),
        "f0_quality_non_regression": bool(
            complete
            and candidate["gain_nrmse_mean"] <= 1.05 * f0["gain_nrmse_mean"]
            and candidate["edge_gradient_p99_mean"]
            <= 1.05 * f0["edge_gradient_p99_mean"]
        ),
        "mutation_rejection": all(value >= 11 for value in mutation_counts.values()),
        "paired_primary_wins": _paired_wins(metrics) >= 10,
        "prior_ratio": bool(
            complete
            and candidate["gain_nrmse_mean"]
            <= 0.75 * summaries["prior-only"]["gain_nrmse_mean"]
            and candidate["edge_gradient_p99_mean"]
            <= 0.75 * summaries["prior-only"]["edge_gradient_p99_mean"]
        ),
        "raw_harmonic_ratio": bool(
            complete
            and candidate["gain_nrmse_mean"]
            <= 0.60 * summaries["raw-harmonic"]["gain_nrmse_mean"]
            and candidate["edge_gradient_p99_mean"]
            <= 0.60 * summaries["raw-harmonic"]["edge_gradient_p99_mean"]
        ),
        "remesh_drift": bool(
            len(remesh) == 12
            and all(row["gain_metric_drift"] <= 0.10 for row in remesh)
        ),
        "remesh_f0_improvement": bool(max_drift <= drift_limit),
        "remesh_probe": bool(
            len(remesh) == 12
            and np.mean([row["probe_disagreement_nrmse"] for row in remesh]) <= 0.13
            and max(row["probe_disagreement_nrmse"] for row in remesh) <= 0.18
        ),
        "serialization_roundtrip": roundtrip_exact,
        "structural_pre_inference": bool(
            len(structural) == 120
            and all(row["pre_inference_reject"] for row in structural)
        ),
        "topology_gain": all(
            value is not None and value <= 0.30 for value in topology_gain.values()
        ),
        "topology_gradient_strict": all(
            value is not None and value <= 0.50 for value in topology_gradient.values()
        ),
        "zero_forbidden_access": all(
            value == 0 for value in common.ZERO_ACCESS.values()
        ),
    }


def _selection_key(
    result: model.TrainingResult,
    metrics: dict[str, list[dict[str, Any]]],
    remesh: list[dict[str, Any]],
) -> tuple[Any, ...]:
    assert result.spec is not None
    summary = _summary(metrics["candidate"])
    topology_gradient = _topology_means(metrics["candidate"], "edge_gradient_p99")
    return (
        max(row["gain_metric_drift"] for row in remesh),
        max(row["probe_disagreement_nrmse"] for row in remesh),
        max(float(value) for value in topology_gradient.values() if value is not None),
        summary["gain_nrmse_mean"],
        summary["edge_gradient_p99_mean"],
        result.spec.order,
        result.spec.regularization,
        result.spec.candidate_id,
    )


def _predictions_npz(
    objects: tuple[common.FieldObject, ...],
    candidates: dict[str, dict[str, tuple[np.ndarray, ...]]],
    f0_predictions: tuple[np.ndarray, ...],
    paired_predictions: tuple[np.ndarray, ...],
) -> bytes:
    offsets = [0]
    truth = []
    active = []
    for item in objects:
        offsets.append(offsets[-1] + item.mesh.vertex_count)
        truth.append(item.gains)
        mask = np.zeros(item.mesh.vertex_count, dtype=np.int64)
        mask[item.context] = 1
        mask[item.accepted_query] = 1
        active.append(mask)
    arrays: dict[str, np.ndarray] = {
        "active_mask": np.concatenate(active),
        "frozen_f0": np.concatenate(f0_predictions),
        "object_offsets": np.asarray(offsets, dtype=np.int64),
        "paired_harmonic": np.concatenate(paired_predictions),
        "truth": np.concatenate(truth),
    }
    for candidate_index, spec in enumerate(model.CANDIDATES):
        values = candidates[spec.candidate_id]
        for method_index, method in enumerate(METHOD_ORDER):
            arrays[f"c{candidate_index:02d}_m{method_index:02d}"] = np.concatenate(
                values[method]
            )
    return common.deterministic_npz(arrays)


def run(output: Path, f0_root: Path) -> dict[str, Any]:
    started = time.monotonic()
    common.verify_protocol_environment()
    implementation_commit = common.verify_committed_implementation()
    implementation = common.implementation_hashes()
    frozen_f0, frozen_f0_scale, f0_bytes = common.load_f0_control(f0_root)
    staging, target = common.prepare_output(output)
    try:
        train = common.generate_objects("train", implementation_commit)
        development = common.generate_objects("development", implementation_commit)
        trained_candidates = model.train_candidates(train)
        shared_scale = trained_candidates[0].gain_scale
        paired = model.train_paired_harmonic(train, shared_scale)
        models_payload = model.encode_models(trained_candidates, paired)
        decoded_candidates, decoded_paired, decoded_scale = model.decode_models(
            models_payload
        )

        candidate_predictions: dict[str, dict[str, tuple[np.ndarray, ...]]] = {}
        candidate_metrics: dict[str, dict[str, list[dict[str, Any]]]] = {}
        candidate_remesh: dict[str, list[dict[str, Any]]] = {}
        candidate_mutations: dict[str, list[dict[str, Any]]] = {}
        candidate_structural: dict[str, list[dict[str, Any]]] = {}
        candidate_gates: dict[str, dict[str, bool]] = {}

        coverage = []
        for item in development:
            summary = _coverage_summary(item)
            summary["fallback_complete"] = set(item.rejected_query.tolist()) == {
                int(value["query_vertex"])
                for value in item.coverage_decisions
                if value["reason"] != "ACCEPT"
            }
            coverage.append(summary)
        f0_predictions, f0_metrics, f0_remesh = _f0_control(
            frozen_f0, frozen_f0_scale, development
        )
        paired_predictions, paired_metrics = _paired_control(paired, development)

        all_metric_rows: list[dict[str, Any]] = []
        all_metric_rows.extend(f0_metrics)
        all_metric_rows.extend(paired_metrics)
        all_metric_rows.extend(f0_remesh)

        for candidate_index, trained in enumerate(trained_candidates):
            assert trained.spec is not None
            candidate_id = trained.spec.candidate_id
            predictions, metrics = _evaluate_candidate(trained, development)
            remesh = _continuous_remesh(trained, development, metrics["candidate"])
            structural = _structural_rows(development, trained.spec)
            mutations = _mutation_rows(trained, development, predictions["candidate"])
            roundtrip = bool(
                np.array_equal(decoded_scale, trained.gain_scale)
                and np.array_equal(
                    model.predict_continuous(
                        trained.model,
                        trained.gain_scale,
                        development[0],
                        trained.spec,
                    ),
                    model.predict_continuous(
                        decoded_candidates[candidate_index],
                        decoded_scale,
                        development[0],
                        trained.spec,
                    ),
                )
            )
            gates = _candidate_gates(
                metrics,
                remesh,
                mutations,
                structural,
                f0_metrics,
                f0_remesh,
                coverage,
                roundtrip,
            )
            candidate_predictions[candidate_id] = predictions
            candidate_metrics[candidate_id] = metrics
            candidate_remesh[candidate_id] = remesh
            candidate_mutations[candidate_id] = mutations
            candidate_structural[candidate_id] = structural
            candidate_gates[candidate_id] = gates
            for rows in metrics.values():
                all_metric_rows.extend(rows)
            all_metric_rows.extend(remesh)
            all_metric_rows.extend(mutations)
            all_metric_rows.extend(structural)

        paired_roundtrip = np.array_equal(
            model.predict_paired_harmonic(
                paired.model, paired.gain_scale, development[0]
            ),
            model.predict_paired_harmonic(
                decoded_paired, decoded_scale, development[0]
            ),
        )
        if not paired_roundtrip:
            raise common.F1Error("F1 paired-harmonic roundtrip changed")

        eligible = [
            trained
            for trained in trained_candidates
            if all(candidate_gates[trained.spec.candidate_id].values())  # type: ignore[union-attr]
        ]
        selected = (
            None
            if not eligible
            else min(
                eligible,
                key=lambda trained: _selection_key(
                    trained,
                    candidate_metrics[trained.spec.candidate_id],  # type: ignore[union-attr]
                    candidate_remesh[trained.spec.candidate_id],  # type: ignore[union-attr]
                ),
            )
        )
        selected_id = None if selected is None else selected.spec.candidate_id  # type: ignore[union-attr]
        selection = {
            "candidate_order": [spec.candidate_id for spec in model.CANDIDATES],
            "eligible_candidates": [
                trained.spec.candidate_id
                for trained in eligible
                if trained.spec is not None
            ],
            "passed": selected is not None,
            "schema": common.SELECTION_SCHEMA,
            "selected_candidate_id": selected_id,
            "selection_key": (
                None
                if selected is None
                else _selection_key(
                    selected,
                    candidate_metrics[selected_id],
                    candidate_remesh[selected_id],
                )
            ),
        }
        report = {
            "candidate_gates": candidate_gates,
            "candidate_metrics": {
                candidate_id: {
                    method: _summary(rows) for method, rows in methods.items()
                }
                for candidate_id, methods in candidate_metrics.items()
            },
            "candidate_mutations": candidate_mutations,
            "candidate_remesh": candidate_remesh,
            "coverage": coverage,
            "f0_control": {
                "metrics": _summary(f0_metrics),
                "remesh": f0_remesh,
                "status": "CONTROL_ONLY_PROTOCOL_CONFORMANCE_REJECT",
            },
            "paired_harmonic_control": _summary(paired_metrics),
            "schema": common.REPORT_SCHEMA,
            "selection": selection,
            "single_run_pass": selected is not None,
            "strict_topology_gradient_threshold": 0.50,
            "structural_mutations": candidate_structural,
            "study_id": common.STUDY_ID,
            "training": {
                "candidates": [result.summary() for result in trained_candidates],
                "paired_harmonic": paired.summary(),
            },
        }
        corpus = {
            "candidate_order": [spec.candidate_id for spec in model.CANDIDATES],
            "method_order": METHOD_ORDER,
            "objects": [item.record() for item in (*train, *development)],
            "role_counts": {
                "development_physical_groups": 12,
                "development_views": len(development),
                "train_physical_groups": 24,
                "train_views": len(train),
            },
            "row_ledger": common.row_ledger(),
            "schema": common.CORPUS_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        access = {
            **common.ZERO_ACCESS,
            "development_mesh_views_generated_after_commit": len(development),
            "development_physical_groups_generated_after_commit": 12,
            "f0_control_bytes_read": f0_bytes,
            "schema": common.ACCESS_SCHEMA,
            "train_mesh_views_generated_after_commit": len(train),
            "train_physical_groups_generated_after_commit": 24,
        }
        files = {
            "access-ledger.json": common.canonical_json(access),
            "corpus.json": common.canonical_json(corpus),
            "geometry.npz": common.geometry_npz((*train, *development)),
            "metrics.jsonl": common.canonical_json_lines(all_metric_rows),
            "models.npz": models_payload,
            "predictions.npz": _predictions_npz(
                development,
                candidate_predictions,
                f0_predictions,
                paired_predictions,
            ),
            "report.json": common.canonical_json(report),
            "selection.json": common.canonical_json(selection),
        }
        artifact_hashes = {
            name: common.sha256_bytes(payload)
            for name, payload in sorted(files.items())
        }
        manifest = {
            "artifact_hashes": artifact_hashes,
            "b0_tree_digest": common.B0_TREE_DIGEST,
            "c0_tree_digest": common.C0_TREE_DIGEST,
            "f0_control_tree_digest": common.F0_TREE_DIGEST,
            "implementation_commit": implementation_commit,
            "implementation_hashes": implementation,
            "m0b_tree_digest": common.M0B_TREE_DIGEST,
            "protocol_path": common.PROTOCOL_PATH,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "revision": common.REVISION,
            "row_ledger_root": common.ROW_LEDGER_ROOT,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        files["manifest.json"] = common.canonical_json(manifest)
        if set(files) != OUTPUT_FILES:
            raise common.F1Error(f"F1 output file set changed: {sorted(files)}")
        total_bytes = sum(len(payload) for payload in files.values())
        if total_bytes > 100 * 1024 * 1024:
            raise common.F1Error("F1 output byte ceiling exceeded")
        if time.monotonic() - started > 1_800.0:
            raise common.F1Error("F1 runtime ceiling exceeded")
        if resource.getrusage(resource.RUSAGE_SELF).ru_maxrss > 4 * 1024 * 1024:
            raise common.F1Error("F1 RSS ceiling exceeded")
        common.write_files(staging, files)
        common.publish_output(staging, target)
        return {
            "file_count": len(files),
            "output": str(target),
            "passed": selected is not None,
            "report_sha256": artifact_hashes["report.json"],
            "selected_candidate_id": selected_id,
            "tree_digest": common.tree_digest(common.directory_file_map(target)),
        }
    except Exception:
        common.abandon_output(staging)
        raise


def compare(left: Path, right: Path) -> dict[str, Any]:
    result = common.compare_directories(left, right)
    if result["file_count"] != len(OUTPUT_FILES):
        raise common.F1Error("F1 compared output file count changed")
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--output", type=Path, required=True)
    run_parser.add_argument("--f0-root", type=Path, required=True)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--left", type=Path, required=True)
    compare_parser.add_argument("--right", type=Path, required=True)
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    try:
        if arguments.command == "run":
            result = run(arguments.output, arguments.f0_root)
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (common.F1Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
