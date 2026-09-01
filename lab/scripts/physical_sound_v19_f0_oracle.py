#!/usr/bin/env python3
"""Official one-shot runner for Physical Sound V19 F0."""

from __future__ import annotations

import argparse
import math
import resource
import sys
import time
from dataclasses import replace
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v19_f0_common as common
import physical_sound_v19_f0_model as model
from scipy.interpolate import LinearNDInterpolator

METHOD_ORDER = (
    "candidate",
    "context-mean",
    "nearest-intrinsic",
    "prior-euclidean-rbf",
    "prior-geodesic-rbf",
    "prior-only",
    "raw-euclidean-rbf",
    "raw-geodesic-rbf",
    "raw-harmonic",
)
MUTATION_ORDER = (
    "query-shift",
    "context-shift",
    "alternating-context-sign",
)
STRUCTURAL_MUTATION_ORDER = (
    "duplicate-context",
    "out-of-range",
    "missing-context",
    "mesh-identity-mismatch",
    "topology-identity-mismatch",
)
OUTPUT_FILES = {
    "access-ledger.json",
    "corpus.json",
    "decisions.jsonl",
    "geometry.npz",
    "manifest.json",
    "model.npz",
    "predictions.npz",
    "report.json",
}


def _coverage_summary(item: common.FieldObject) -> dict[str, Any]:
    decisions = item.coverage_decisions
    if not decisions:
        return {
            "accepted_fraction": 0.0,
            "global_fill": None,
            "global_pass": False,
            "local_false_ood_fraction": 1.0,
        }
    global_values = {
        value["global_fill"] for value in decisions if value["global_fill"] is not None
    }
    global_fill = None if len(global_values) != 1 else float(next(iter(global_values)))
    global_pass = bool(
        global_fill is not None
        and global_fill <= common.coverage_common.GLOBAL_INTRINSIC_THRESHOLD
        and all(
            value["reason"]
            not in (
                "OOD_CONTEXT_BUDGET",
                "OOD_DISCONNECTED",
                "OOD_INTRINSIC_FILL",
            )
            for value in decisions
        )
    )
    false_ood = item.rejected_query.size / item.query.size
    return {
        "accepted_fraction": float(item.accepted_query.size / item.query.size),
        "global_fill": global_fill,
        "global_pass": global_pass,
        "local_false_ood_fraction": float(false_ood),
    }


def _prediction_evaluation(
    trained: model.PriorNetwork,
    gain_scale: np.ndarray,
    objects: tuple[common.FieldObject, ...],
) -> tuple[
    dict[str, tuple[np.ndarray, ...]],
    dict[str, list[dict[str, Any]]],
    list[dict[str, Any]],
]:
    predictions: dict[str, list[np.ndarray]] = {name: [] for name in METHOD_ORDER}
    metrics: dict[str, list[dict[str, Any]]] = {name: [] for name in METHOD_ORDER}
    decisions: list[dict[str, Any]] = []
    for item in objects:
        coverage = _coverage_summary(item)
        for decision in item.coverage_decisions:
            decisions.append({**decision, "source_schema": decision["schema"]})
        if not coverage["global_pass"] or item.accepted_query.size == 0:
            for name in METHOD_ORDER:
                predictions[name].append(
                    np.zeros(
                        (item.mesh.vertex_count, common.MODE_COUNT), dtype=np.float64
                    )
                )
                metrics[name].append(
                    {
                        "edge_gradient_p99": None,
                        "gain_nrmse": None,
                        "method": name,
                        "object_id": item.row.object_id,
                        "physical_group_id": item.row.physical_group_id,
                        "topology": item.row.topology,
                    }
                )
            continue
        values = model.predict_methods(trained, gain_scale, item)
        if tuple(values) != METHOD_ORDER:
            raise common.F0Error("F0 method order changed")
        for name, prediction in values.items():
            predictions[name].append(prediction)
            row = {
                **model.metrics(item, prediction),
                "method": name,
                "object_id": item.row.object_id,
                "physical_group_id": item.row.physical_group_id,
                "topology": item.row.topology,
            }
            metrics[name].append(row)
            decisions.append(
                {
                    **row,
                    "role": item.row.role,
                    "schema": common.DECISION_SCHEMA,
                    "type": "field-metric",
                }
            )
    return (
        {name: tuple(value) for name, value in predictions.items()},
        metrics,
        decisions,
    )


def _metric_summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
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


def _active_vertices(item: common.FieldObject) -> np.ndarray:
    return np.unique(np.concatenate((item.context, item.accepted_query)))


def _probe_values(
    item: common.FieldObject, values: np.ndarray, active_only: bool
) -> np.ndarray:
    index = _active_vertices(item) if active_only else np.arange(item.mesh.vertex_count)
    uv = item.mesh.uv[index]
    source = np.asarray(values, dtype=np.float64)[index]
    if item.row.topology in ("Cylinder", "Bowl"):
        uv = np.concatenate((uv, uv + [2.0, 0.0], uv - [2.0, 0.0]))
        source = np.concatenate((source, source, source))
    grid = np.linspace(-0.75, 0.75, 7, dtype=np.float64)
    probes = np.asarray([(u, v) for v in grid for u in grid], dtype=np.float64)
    result = np.column_stack(
        [
            LinearNDInterpolator(uv, source[:, mode_index])(probes)
            for mode_index in range(common.MODE_COUNT)
        ]
    )
    if result.shape != (49, common.MODE_COUNT) or not np.isfinite(result).all():
        raise common.F0Error("F0 canonical remesh probe interpolation failed")
    return result


def _remesh_metrics(
    objects: tuple[common.FieldObject, ...],
    candidate: tuple[np.ndarray, ...],
    metric_rows: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    by_group: dict[str, list[int]] = {}
    for index, item in enumerate(objects):
        by_group.setdefault(item.row.physical_group_id, []).append(index)
    rows = []
    for group, indices in sorted(by_group.items()):
        if len(indices) != 2:
            raise common.F0Error(f"F0 remesh group changed: {group}")
        primary_index = next(index for index in indices if not objects[index].is_twin)
        twin_index = next(index for index in indices if objects[index].is_twin)
        primary = objects[primary_index]
        twin = objects[twin_index]
        primary_probe = _probe_values(primary, candidate[primary_index], True)
        twin_probe = _probe_values(twin, candidate[twin_index], True)
        truth_probe = _probe_values(primary, primary.gains, False)
        denominator = float(np.sqrt(np.mean(truth_probe**2)))
        disagreement = float(
            np.sqrt(np.mean((primary_probe - twin_probe) ** 2)) / denominator
        )
        primary_metric = metric_rows[primary_index]["gain_nrmse"]
        twin_metric = metric_rows[twin_index]["gain_nrmse"]
        if primary_metric is None or twin_metric is None:
            drift = None
        else:
            drift = float(
                abs(float(twin_metric) - float(primary_metric))
                / max(float(primary_metric), 1.0e-12)
            )
        rows.append(
            {
                "gain_metric_drift": drift,
                "physical_group_id": group,
                "probe_disagreement_nrmse": disagreement,
                "topology": primary.row.topology,
            }
        )
    return rows


def _mutated_prediction(
    trained: model.PriorNetwork,
    gain_scale: np.ndarray,
    item: common.FieldObject,
    clean: np.ndarray,
    mutation: str,
) -> np.ndarray:
    if mutation == "query-shift":
        result = clean.copy()
        result[item.accepted_query] = np.roll(result[item.accepted_query], 17, axis=0)
        return result
    observed = item.gains[item.context].copy()
    if mutation == "context-shift":
        observed = np.roll(observed, 17, axis=0)
    elif mutation == "alternating-context-sign":
        observed[:, 1::2] *= -1.0
    else:
        raise common.F0Error(f"unknown F0 numerical mutation: {mutation}")
    return model.predict_methods(trained, gain_scale, item, observed)["candidate"]


def _mutation_metrics(
    trained: model.PriorNetwork,
    gain_scale: np.ndarray,
    objects: tuple[common.FieldObject, ...],
    candidate: tuple[np.ndarray, ...],
) -> list[dict[str, Any]]:
    rows = []
    for index, item in enumerate(objects):
        if item.is_twin:
            continue
        for mutation in MUTATION_ORDER:
            prediction = _mutated_prediction(
                trained, gain_scale, item, candidate[index], mutation
            )
            value = model.metrics(item, prediction)
            rejected = bool(
                value["gain_nrmse"] > 0.40 or value["edge_gradient_p99"] > 0.80
            )
            rows.append(
                {
                    **value,
                    "mutation": mutation,
                    "object_id": item.row.object_id,
                    "quality_reject": rejected,
                    "topology": item.row.topology,
                }
            )
    return rows


def _structural_input(
    item: common.FieldObject, mutation: str
) -> common.coverage_oracle.CoverageInput:
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
        required = common.coverage_common.required_context_count(value.vertex_count)
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
        changed = common.TOPOLOGY_ORDER[
            (common.TOPOLOGY_ORDER.index(value.topology) + 1)
            % len(common.TOPOLOGY_ORDER)
        ]
        return replace(value, topology=changed)
    raise common.F0Error(f"unknown F0 structural mutation: {mutation}")


def _structural_metrics(
    objects: tuple[common.FieldObject, ...],
) -> list[dict[str, Any]]:
    rows = []
    for item in objects:
        if item.is_twin:
            continue
        for mutation in STRUCTURAL_MUTATION_ORDER:
            value = _structural_input(item, mutation)
            decisions = common.coverage_oracle.evaluate(
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
                }
            )
    return rows


def _topology_means(rows: list[dict[str, Any]], field: str) -> dict[str, float | None]:
    result = {}
    for topology in common.TOPOLOGY_ORDER:
        values = [row[field] for row in rows if row["topology"] == topology]
        result[topology] = (
            None
            if not values or any(value is None for value in values)
            else float(np.mean(values))
        )
    return result


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
        fields = ("gain_nrmse", "edge_gradient_p99")
        if all(
            candidate[field] is not None
            and candidate[field] < prior[field]
            and candidate[field] < euclidean[field]
            for field in fields
        ):
            wins += 1
    return wins


def _gates(
    objects: tuple[common.FieldObject, ...],
    metrics: dict[str, list[dict[str, Any]]],
    remesh: list[dict[str, Any]],
    mutations: list[dict[str, Any]],
    structural: list[dict[str, Any]],
    roundtrip_exact: bool,
) -> dict[str, bool]:
    summaries = {name: _metric_summary(rows) for name, rows in metrics.items()}
    candidate = summaries["candidate"]
    compatible = [
        summaries[name]
        for name in (
            "context-mean",
            "nearest-intrinsic",
            "prior-euclidean-rbf",
            "prior-geodesic-rbf",
            "prior-only",
            "raw-euclidean-rbf",
            "raw-geodesic-rbf",
            "raw-harmonic",
        )
    ]
    complete = bool(
        candidate["complete"] and all(value["complete"] for value in compatible)
    )
    best_gain = (
        min(value["gain_nrmse_mean"] for value in compatible) if complete else None
    )
    best_gradient = (
        min(value["edge_gradient_p99_mean"] for value in compatible)
        if complete
        else None
    )
    coverage = [_coverage_summary(item) for item in objects]
    topology_gain = _topology_means(metrics["candidate"], "gain_nrmse")
    topology_gradient = _topology_means(metrics["candidate"], "edge_gradient_p99")
    mutation_counts = {
        name: sum(row["quality_reject"] for row in mutations if row["mutation"] == name)
        for name in MUTATION_ORDER
    }
    return {
        "candidate_absolute_gain": bool(
            complete
            and candidate["gain_nrmse_mean"] <= 0.25
            and candidate["gain_nrmse_max"] <= 0.40
        ),
        "candidate_absolute_gradient": bool(
            complete
            and candidate["edge_gradient_p99_mean"] <= 0.45
            and candidate["edge_gradient_p99_max"] <= 0.80
        ),
        "candidate_best_control_ratio": bool(
            complete
            and candidate["gain_nrmse_mean"] <= 0.80 * best_gain
            and candidate["edge_gradient_p99_mean"] <= 0.80 * best_gradient
        ),
        "candidate_prior_ratio": bool(
            complete
            and candidate["gain_nrmse_mean"]
            <= 0.75 * summaries["prior-only"]["gain_nrmse_mean"]
            and candidate["edge_gradient_p99_mean"]
            <= 0.75 * summaries["prior-only"]["edge_gradient_p99_mean"]
        ),
        "candidate_raw_harmonic_ratio": bool(
            complete
            and candidate["gain_nrmse_mean"]
            <= 0.60 * summaries["raw-harmonic"]["gain_nrmse_mean"]
            and candidate["edge_gradient_p99_mean"]
            <= 0.60 * summaries["raw-harmonic"]["edge_gradient_p99_mean"]
        ),
        "coverage_global": all(value["global_pass"] for value in coverage),
        "coverage_local": all(
            value["local_false_ood_fraction"] <= 0.05 for value in coverage
        ),
        "finite_complete": bool(
            complete
            and all(
                math.isfinite(float(row[field]))
                for rows in metrics.values()
                for row in rows
                for field in ("gain_nrmse", "edge_gradient_p99")
            )
        ),
        "mutation_rejection": all(value >= 11 for value in mutation_counts.values()),
        "paired_primary_wins": _paired_wins(metrics) >= 10,
        "remesh_metric_drift": bool(
            len(remesh) == 12
            and all(
                row["gain_metric_drift"] is not None
                and row["gain_metric_drift"] <= 0.10
                for row in remesh
            )
        ),
        "remesh_probe": bool(
            len(remesh) == 12
            and np.mean([row["probe_disagreement_nrmse"] for row in remesh]) <= 0.13
            and max(row["probe_disagreement_nrmse"] for row in remesh) <= 0.18
        ),
        "serialization_roundtrip": roundtrip_exact,
        "structural_pre_inference": bool(
            len(structural) == 60
            and all(row["pre_inference_reject"] for row in structural)
        ),
        "topology_gain": all(
            value is not None and value <= 0.30 for value in topology_gain.values()
        ),
        "topology_gradient": all(
            value is not None and value <= 0.55 for value in topology_gradient.values()
        ),
        "zero_forbidden_access": all(
            value == 0 for value in common.ZERO_ACCESS.values()
        ),
    }


def _predictions_npz(
    objects: tuple[common.FieldObject, ...],
    predictions: dict[str, tuple[np.ndarray, ...]],
) -> bytes:
    offsets = [0]
    truth = []
    active = []
    arrays: dict[str, np.ndarray] = {}
    for item in objects:
        offsets.append(offsets[-1] + item.mesh.vertex_count)
        truth.append(item.gains)
        mask = np.zeros(item.mesh.vertex_count, dtype=np.int64)
        mask[item.context] = 1
        mask[item.accepted_query] = 1
        active.append(mask)
    arrays["active_mask"] = np.concatenate(active)
    arrays["object_offsets"] = np.asarray(offsets, dtype=np.int64)
    arrays["truth"] = np.concatenate(truth)
    for index, name in enumerate(METHOD_ORDER):
        arrays[f"method_{index:02d}"] = np.concatenate(predictions[name])
    return common.deterministic_npz(arrays)


def _roundtrip_exact(
    trained: model.PriorNetwork,
    gain_scale: np.ndarray,
    payload: bytes,
    item: common.FieldObject,
) -> bool:
    decoded, decoded_scale = model.decode_model(payload)
    if not np.array_equal(decoded_scale, gain_scale):
        return False
    left = model.predict_methods(trained, gain_scale, item)["candidate"]
    right = model.predict_methods(decoded, decoded_scale, item)["candidate"]
    return bool(np.array_equal(left, right))


def development_preview() -> dict[str, Any]:
    """Run the frozen recipe on development only; never generates F0 test."""
    common.verify_protocol_environment()
    train = common.generate_objects("train")
    development = common.generate_objects("development")
    trained = model.train_prior(train)
    payload = model.encode_model(trained.model, trained.gain_scale)
    predictions, metrics, _decisions = _prediction_evaluation(
        trained.model, trained.gain_scale, development
    )
    remesh = _remesh_metrics(
        development, predictions["candidate"], metrics["candidate"]
    )
    mutations = _mutation_metrics(
        trained.model,
        trained.gain_scale,
        development,
        predictions["candidate"],
    )
    structural = _structural_metrics(development)
    roundtrip = _roundtrip_exact(
        trained.model, trained.gain_scale, payload, development[0]
    )
    gates = _gates(development, metrics, remesh, mutations, structural, roundtrip)
    return {
        "gates": gates,
        "metrics": {name: _metric_summary(rows) for name, rows in metrics.items()},
        "mutations": mutations,
        "remesh": remesh,
        "structural_mutations": structural,
        "test_rows_generated": 0,
        "training": trained.summary(),
    }


def run(output: Path) -> dict[str, Any]:
    started = time.monotonic()
    common.verify_protocol_environment()
    implementation_commit = common.verify_committed_implementation()
    implementation = common.implementation_hashes()
    staging, target = common.prepare_output(output)
    try:
        train = common.generate_objects("train")
        trained = model.train_prior(train)
        model_payload = model.encode_model(trained.model, trained.gain_scale)

        # This is the first operation that generates any sealed F0 test value.
        test = common.generate_objects("test")
        predictions, metrics, decisions = _prediction_evaluation(
            trained.model, trained.gain_scale, test
        )
        roundtrip_exact = _roundtrip_exact(
            trained.model, trained.gain_scale, model_payload, test[0]
        )
        remesh = _remesh_metrics(test, predictions["candidate"], metrics["candidate"])
        mutations = _mutation_metrics(
            trained.model,
            trained.gain_scale,
            test,
            predictions["candidate"],
        )
        structural = _structural_metrics(test)
        gates = _gates(test, metrics, remesh, mutations, structural, roundtrip_exact)
        summaries = {name: _metric_summary(rows) for name, rows in metrics.items()}
        coverage = [
            {"object_id": item.row.object_id, **_coverage_summary(item)}
            for item in test
        ]
        all_decisions = (
            decisions
            + [
                {**row, "schema": common.DECISION_SCHEMA, "type": "numerical-mutation"}
                for row in mutations
            ]
            + [
                {**row, "schema": common.DECISION_SCHEMA, "type": "structural-mutation"}
                for row in structural
            ]
        )
        report = {
            "coverage": coverage,
            "gates": gates,
            "metrics": summaries,
            "mutations": mutations,
            "remesh": remesh,
            "schema": common.REPORT_SCHEMA,
            "single_run_pass": all(gates.values()),
            "structural_mutations": structural,
            "study_id": common.STUDY_ID,
            "topology": {
                "edge_gradient_p99_mean": _topology_means(
                    metrics["candidate"], "edge_gradient_p99"
                ),
                "gain_nrmse_mean": _topology_means(metrics["candidate"], "gain_nrmse"),
            },
            "training": trained.summary(),
        }
        corpus = {
            "method_order": METHOD_ORDER,
            "objects": [item.record() for item in (*train, *test)],
            "role_counts": {"test_mesh_views": len(test), "train": len(train)},
            "row_roots": common.ROW_ROOTS,
            "schema": common.CORPUS_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        access = {
            **common.ZERO_ACCESS,
            "schema": common.ACCESS_SCHEMA,
            "test_mesh_views_generated_after_commit": len(test),
            "test_physical_groups_generated_after_commit": 12,
        }
        files = {
            "access-ledger.json": common.canonical_json(access),
            "corpus.json": common.canonical_json(corpus),
            "decisions.jsonl": common.canonical_json_lines(all_decisions),
            "geometry.npz": common.geometry_npz((*train, *test)),
            "model.npz": model_payload,
            "predictions.npz": _predictions_npz(test, predictions),
            "report.json": common.canonical_json(report),
        }
        artifact_hashes = {
            name: common.sha256_bytes(payload)
            for name, payload in sorted(files.items())
        }
        manifest = {
            "artifact_hashes": artifact_hashes,
            "b0_complete_tree_digest": "bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111",
            "c0_complete_tree_digest": "ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace",
            "implementation_commit": implementation_commit,
            "implementation_hashes": implementation,
            "protocol_path": common.PROTOCOL_PATH,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        files["manifest.json"] = common.canonical_json(manifest)
        if set(files) != OUTPUT_FILES:
            raise common.F0Error(f"F0 output file set changed: {sorted(files)}")
        common.write_files(staging, files)
        file_bytes = sum(
            path.stat().st_size for path in staging.iterdir() if path.is_file()
        )
        if file_bytes > 100 * 1024 * 1024:
            raise common.F0Error("F0 output byte ceiling exceeded")
        if time.monotonic() - started > 600.0:
            raise common.F0Error("F0 runtime ceiling exceeded")
        peak_kib = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        if peak_kib > 4 * 1024 * 1024:
            raise common.F0Error("F0 RSS ceiling exceeded")
        common.publish_output(staging, target)
        return {
            "file_count": len(files),
            "output": str(target),
            "passed": all(gates.values()),
            "report_sha256": artifact_hashes["report.json"],
            "tree_digest": common.tree_digest(common.directory_file_map(target)),
        }
    except Exception:
        common.abandon_output(staging)
        raise


def compare(left: Path, right: Path) -> dict[str, Any]:
    result = common.compare_directories(left, right)
    if result["file_count"] != len(OUTPUT_FILES):
        raise common.F0Error("F0 compared output file count changed")
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--output", type=Path, required=True)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--left", type=Path, required=True)
    compare_parser.add_argument("--right", type=Path, required=True)
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    try:
        if arguments.command == "run":
            result = run(arguments.output)
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (common.F0Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
