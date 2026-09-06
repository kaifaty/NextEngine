#!/usr/bin/env python3
"""Official frozen B0+C0+F0 integration runner for Physical Sound V19 I0."""

from __future__ import annotations

import argparse
import math
import resource
import sys
import time
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v19_f0_oracle as field_oracle
import physical_sound_v19_i0_common as common
from scipy.signal import hilbert

SAMPLE_RATE_HZ = 16_000
SAMPLE_COUNT = 8_192
SPECTRUM_WINDOWS = (512, 1_024, 2_048, 4_096)
OUTPUT_FILES = {
    "access-ledger.json",
    "corpus.json",
    "decisions.jsonl",
    "endpoints.npz",
    "fallback.json",
    "geometry.npz",
    "manifest.json",
    "report.json",
}


def _hard_modes(frequencies: np.ndarray, damping: np.ndarray) -> bool:
    return bool(
        frequencies.shape == (8,)
        and damping.shape == (8,)
        and np.isfinite(frequencies).all()
        and np.isfinite(damping).all()
        and frequencies[0] >= 40.0
        and frequencies[-1] <= 7_500.0
        and np.all(np.diff(frequencies) > 0.0)
        and np.all(damping > 0.0)
        and np.all(damping <= 128.0)
    )


def _render_modal(
    frequencies: np.ndarray, damping: np.ndarray, gains: np.ndarray
) -> np.ndarray:
    time_axis = np.arange(SAMPLE_COUNT, dtype=np.float64) / SAMPLE_RATE_HZ
    basis = np.exp(-damping[:, None] * time_axis[None, :]) * np.sin(
        2.0 * np.pi * frequencies[:, None] * time_axis[None, :]
    )
    result = np.asarray(gains, dtype=np.float64) @ basis
    if not np.isfinite(result).all():
        raise common.I0Error("I0 modal renderer produced a non-finite value")
    return result


def _spectrum_rmse(prediction: np.ndarray, truth: np.ndarray) -> np.ndarray:
    prediction_peak = np.max(np.abs(prediction), axis=1, keepdims=True)
    truth_peak = np.max(np.abs(truth), axis=1, keepdims=True)
    active = truth_peak[:, 0] > 1.0e-12
    if not np.any(active):
        raise common.I0Error("I0 spectrum metric has no active truth waveform")
    prediction = prediction[active]
    truth = truth[active]
    prediction_peak = prediction_peak[active]
    truth_peak = truth_peak[active]
    valid_prediction = prediction_peak[:, 0] > 1.0e-12
    result = np.full(prediction.shape[0], 80.0, dtype=np.float64)
    if np.any(valid_prediction):
        prediction = prediction[valid_prediction] / prediction_peak[valid_prediction]
        truth = truth[valid_prediction] / truth_peak[valid_prediction]
        rows = []
        for length in SPECTRUM_WINDOWS:
            index = np.arange(length, dtype=np.float64)
            window = 0.5 - 0.5 * np.cos(2.0 * np.pi * index / length)
            predicted_spectrum = np.abs(
                np.fft.rfft(prediction[:, :length] * window, axis=1)
            )
            truth_spectrum = np.abs(np.fft.rfft(truth[:, :length] * window, axis=1))
            predicted_spectrum /= np.maximum(
                np.max(predicted_spectrum, axis=1, keepdims=True), 1.0e-12
            )
            truth_spectrum /= np.maximum(
                np.max(truth_spectrum, axis=1, keepdims=True), 1.0e-12
            )
            predicted_db = 20.0 * np.log10(np.maximum(predicted_spectrum, 1.0e-4))
            truth_db = 20.0 * np.log10(np.maximum(truth_spectrum, 1.0e-4))
            rows.append(np.sqrt(np.mean((predicted_db - truth_db) ** 2, axis=1)))
        result[valid_prediction] = np.mean(np.stack(rows, axis=1), axis=1)
    return result


def _waveform_metrics(
    objects: tuple[common.f0_common.FieldObject, ...],
    modes: tuple[common.GlobalModes, ...],
    candidate: tuple[np.ndarray, ...],
) -> tuple[dict[str, float], list[dict[str, float | str]]]:
    early_error = 0.0
    early_truth = 0.0
    full_error = 0.0
    full_truth = 0.0
    spectra = []
    envelopes = []
    rows = []
    for item, global_value, prediction in zip(objects, modes, candidate, strict=True):
        query = item.accepted_query
        truth_pcm = _render_modal(
            global_value.truth_frequencies,
            global_value.truth_damping,
            item.gains[query],
        )
        predicted_pcm = _render_modal(
            global_value.predicted_frequencies,
            global_value.predicted_damping,
            prediction[query],
        )
        early_delta = predicted_pcm[:, :1024] - truth_pcm[:, :1024]
        full_delta = predicted_pcm - truth_pcm
        early_error += float(np.sum(early_delta**2))
        early_truth += float(np.sum(truth_pcm[:, :1024] ** 2))
        full_error += float(np.sum(full_delta**2))
        full_truth += float(np.sum(truth_pcm**2))
        object_spectrum = _spectrum_rmse(predicted_pcm, truth_pcm)
        spectra.extend(object_spectrum.tolist())
        truth_envelope = np.abs(hilbert(truth_pcm, axis=1))
        predicted_envelope = np.abs(hilbert(predicted_pcm, axis=1))
        envelope_denominator = float(np.sqrt(np.mean(truth_envelope**2)))
        if envelope_denominator <= 1.0e-12:
            raise common.I0Error("I0 envelope denominator is degenerate")
        object_envelope = (
            np.sqrt(np.mean((predicted_envelope - truth_envelope) ** 2, axis=1))
            / envelope_denominator
        )
        envelopes.extend(object_envelope.tolist())
        rows.append(
            {
                "early_waveform_nrmse": float(
                    np.sqrt(np.mean(early_delta**2))
                    / np.sqrt(np.mean(truth_pcm[:, :1024] ** 2))
                ),
                "envelope_nrmse_p95": float(np.quantile(object_envelope, 0.95)),
                "object_id": item.row.object_id,
                "spectrum_rmse_db_median": float(np.median(object_spectrum)),
            }
        )
    aggregate = {
        "early_waveform_nrmse": math.sqrt(early_error / early_truth),
        "envelope_nrmse_p95": float(np.quantile(envelopes, 0.95)),
        "full_waveform_nrmse": math.sqrt(full_error / full_truth),
        "spectrum_rmse_db_median": float(np.median(spectra)),
    }
    if not all(math.isfinite(value) for value in aggregate.values()):
        raise common.I0Error("I0 waveform aggregate is non-finite")
    return aggregate, rows


def _global_metrics(modes: tuple[common.GlobalModes, ...]) -> dict[str, float]:
    frequency = np.concatenate(
        [
            np.abs(
                1_200.0 * np.log2(value.predicted_frequencies / value.truth_frequencies)
            )
            for value in modes
        ]
    )
    damping = np.concatenate(
        [
            np.abs(value.predicted_damping - value.truth_damping) / value.truth_damping
            for value in modes
        ]
    )
    return {
        "damping_relative_median": float(np.median(damping)),
        "damping_relative_p95": float(np.quantile(damping, 0.95)),
        "frequency_cents_max": float(np.max(frequency)),
        "frequency_cents_median": float(np.median(frequency)),
        "frequency_cents_p95": float(np.quantile(frequency, 0.95)),
    }


def _fallback_records(
    objects: tuple[common.f0_common.FieldObject, ...],
) -> list[dict[str, Any]]:
    rows = []
    for item in objects:
        by_vertex = {
            value["query_vertex"]: value
            for value in item.coverage_decisions
            if value["reason"] != "ACCEPT"
        }
        for vertex in item.rejected_query:
            decision = by_vertex[int(vertex)]
            rows.append(
                {
                    "fallback": "authored-clip",
                    "object_id": item.row.object_id,
                    "query_vertex": int(vertex),
                    "reason": decision["reason"],
                    "schema": common.FALLBACK_SCHEMA,
                }
            )
    return rows


def _fallback_complete(
    rejected: set[tuple[str, int]], fallback: list[dict[str, Any]]
) -> bool:
    observed = {(row["object_id"], row["query_vertex"]) for row in fallback}
    return observed == rejected and len(observed) == len(fallback)


def _hard_mutations(
    objects: tuple[common.f0_common.FieldObject, ...],
    modes: tuple[common.GlobalModes, ...],
    fallback: list[dict[str, Any]],
) -> dict[str, int]:
    counts = {
        "dependency-hash": 0,
        "missing-fallback": 0,
        "negative-damping": 0,
        "unordered-frequency": 0,
    }
    for item, value in zip(objects, modes, strict=True):
        if item.is_twin:
            continue
        negative = value.predicted_damping.copy()
        negative[0] = -abs(negative[0])
        counts["negative-damping"] += int(
            not _hard_modes(value.predicted_frequencies, negative)
        )
        unordered = value.predicted_frequencies.copy()
        unordered[1], unordered[2] = unordered[2], unordered[1]
        counts["unordered-frequency"] += int(
            not _hard_modes(unordered, value.predicted_damping)
        )
        counts["dependency-hash"] += int(
            not _dependency_claim_valid("0" * 64, common.F0_TREE_DIGEST)
        )
        rejected = {
            (row["object_id"], row["query_vertex"])
            for row in fallback
            if row["object_id"] == item.row.object_id
        }
        injected = set(rejected)
        injected.add((item.row.object_id, -1))
        counts["missing-fallback"] += int(
            not _fallback_complete(
                injected,
                [row for row in fallback if row["object_id"] == item.row.object_id],
            )
        )
    return counts


def _dependency_claim_valid(b0_tree: str, f0_tree: str) -> bool:
    return b0_tree == common.B0_TREE_DIGEST and f0_tree == common.F0_TREE_DIGEST


def _endpoint_npz(
    objects: tuple[common.f0_common.FieldObject, ...],
    modes: tuple[common.GlobalModes, ...],
    candidate: tuple[np.ndarray, ...],
) -> bytes:
    offsets = [0]
    accepted = []
    truth_gain = []
    predicted_gain = []
    for item, prediction in zip(objects, candidate, strict=True):
        offsets.append(offsets[-1] + item.mesh.vertex_count)
        mask = np.zeros(item.mesh.vertex_count, dtype=np.int64)
        mask[item.accepted_query] = 1
        accepted.append(mask)
        truth_gain.append(item.gains)
        predicted_gain.append(prediction)
    return common.deterministic_npz(
        {
            "accepted_query_mask": np.concatenate(accepted),
            "object_offsets": np.asarray(offsets, dtype=np.int64),
            "predicted_damping": np.stack([value.predicted_damping for value in modes]),
            "predicted_frequencies": np.stack(
                [value.predicted_frequencies for value in modes]
            ),
            "predicted_gains": np.concatenate(predicted_gain),
            "truth_damping": np.stack([value.truth_damping for value in modes]),
            "truth_frequencies": np.stack([value.truth_frequencies for value in modes]),
            "truth_gains": np.concatenate(truth_gain),
        }
    )


def _integration_gates(
    field_gates: dict[str, bool],
    global_metrics: dict[str, float],
    waveform: dict[str, float],
    fallback_complete: bool,
    hard_mutations: dict[str, int],
) -> dict[str, bool]:
    return {
        "b0_damping": global_metrics["damping_relative_median"] <= 0.08
        and global_metrics["damping_relative_p95"] <= 0.20,
        "b0_frequency": global_metrics["frequency_cents_median"] <= 20.0
        and global_metrics["frequency_cents_p95"] <= 60.0,
        "dependency_identity": _dependency_claim_valid(
            common.B0_TREE_DIGEST, common.F0_TREE_DIGEST
        ),
        "early_waveform": waveform["early_waveform_nrmse"] <= 0.35,
        "envelope": waveform["envelope_nrmse_p95"] <= 0.15,
        "f0_all_gates": all(field_gates.values()),
        "fallback_complete": fallback_complete,
        "hard_mutations": all(value == 12 for value in hard_mutations.values()),
        "modal_peak": global_metrics["frequency_cents_max"] <= 60.0,
        "spectrum": waveform["spectrum_rmse_db_median"] <= 2.5,
        "zero_forbidden_access": all(
            value == 0 for value in common.ZERO_ACCESS.values()
        ),
    }


def development_preview(b0_root: Path, f0_root: Path) -> dict[str, Any]:
    """Compose frozen dependencies on development only; never generates I0."""
    common.verify_protocol_environment()
    dependencies = common.load_dependencies(b0_root, f0_root)
    objects = common.f0_common.generate_objects("development")
    modes = tuple(common.global_modes(dependencies.b0, item.row) for item in objects)
    predictions, field_metrics, _field_decisions = field_oracle._prediction_evaluation(
        dependencies.f0_model, dependencies.gain_scale, objects
    )
    remesh = field_oracle._remesh_metrics(
        objects, predictions["candidate"], field_metrics["candidate"]
    )
    field_mutations = field_oracle._mutation_metrics(
        dependencies.f0_model,
        dependencies.gain_scale,
        objects,
        predictions["candidate"],
    )
    structural = field_oracle._structural_metrics(objects)
    field_gates = field_oracle._gates(
        objects, field_metrics, remesh, field_mutations, structural, True
    )
    global_summary = _global_metrics(modes)
    waveform, _rows = _waveform_metrics(objects, modes, predictions["candidate"])
    truth_modes = tuple(
        common.GlobalModes(
            value.truth_frequencies,
            value.truth_damping,
            value.truth_frequencies,
            value.truth_damping,
        )
        for value in modes
    )
    field_only, _ = _waveform_metrics(objects, truth_modes, predictions["candidate"])
    global_only, _ = _waveform_metrics(
        objects, modes, tuple(item.gains for item in objects)
    )
    fallback = _fallback_records(objects)
    rejected = {
        (item.row.object_id, int(vertex))
        for item in objects
        for vertex in item.rejected_query
    }
    hard_mutations = _hard_mutations(objects, modes, fallback)
    gates = _integration_gates(
        field_gates,
        global_summary,
        waveform,
        _fallback_complete(rejected, fallback),
        hard_mutations,
    )
    return {
        "field_gates": field_gates,
        "gates": gates,
        "global_metrics": global_summary,
        "integration_rows_generated": 0,
        "waveform_attribution": {
            "combined": waveform,
            "field_only": field_only,
            "global_only": global_only,
        },
        "waveform": waveform,
    }


def run(b0_root: Path, f0_root: Path, output: Path) -> dict[str, Any]:
    if not common.I0_EXECUTION_AUTHORIZED:
        raise common.I0Error(common.I0_CLOSED_REASON)
    started = time.monotonic()
    common.verify_protocol_environment()
    implementation_commit = common.verify_committed_implementation()
    implementation = common.implementation_hashes()
    dependencies = common.load_dependencies(b0_root, f0_root)
    staging, target = common.prepare_output(output)
    try:
        # This is the first operation that generates any sealed I0 value.
        objects = common.generate_integration_objects()
        modes = tuple(
            common.global_modes(dependencies.b0, item.row) for item in objects
        )
        if not all(
            _hard_modes(value.predicted_frequencies, value.predicted_damping)
            for value in modes
        ):
            raise common.I0Error("I0 loaded B0 prediction failed hard validation")

        predictions, field_metrics, field_decisions = (
            field_oracle._prediction_evaluation(
                dependencies.f0_model, dependencies.gain_scale, objects
            )
        )
        remesh = field_oracle._remesh_metrics(
            objects, predictions["candidate"], field_metrics["candidate"]
        )
        field_mutations = field_oracle._mutation_metrics(
            dependencies.f0_model,
            dependencies.gain_scale,
            objects,
            predictions["candidate"],
        )
        structural = field_oracle._structural_metrics(objects)
        field_gates = field_oracle._gates(
            objects,
            field_metrics,
            remesh,
            field_mutations,
            structural,
            True,
        )
        global_summary = _global_metrics(modes)
        waveform, waveform_rows = _waveform_metrics(
            objects, modes, predictions["candidate"]
        )
        fallback = _fallback_records(objects)
        rejected = {
            (item.row.object_id, int(vertex))
            for item in objects
            for vertex in item.rejected_query
        }
        fallback_complete = _fallback_complete(rejected, fallback)
        hard_mutations = _hard_mutations(objects, modes, fallback)
        gates = _integration_gates(
            field_gates,
            global_summary,
            waveform,
            fallback_complete,
            hard_mutations,
        )
        field_summaries = {
            name: field_oracle._metric_summary(rows)
            for name, rows in field_metrics.items()
        }
        report = {
            "field_gates": field_gates,
            "field_metrics": field_summaries,
            "gates": gates,
            "global_metrics": global_summary,
            "hard_mutations": hard_mutations,
            "remesh": remesh,
            "schema": common.REPORT_SCHEMA,
            "single_run_pass": all(gates.values()),
            "study_id": common.STUDY_ID,
            "waveform": waveform,
            "waveform_by_object": waveform_rows,
        }
        corpus = {
            "b0_tree_digest": common.B0_TREE_DIGEST,
            "f0_tree_digest": common.F0_TREE_DIGEST,
            "objects": [
                {
                    **item.record(),
                    "predicted_damping_hash": common.identity_hash(
                        value.predicted_damping
                    ),
                    "predicted_frequency_hash": common.identity_hash(
                        value.predicted_frequencies
                    ),
                    "truth_damping_hash": common.identity_hash(value.truth_damping),
                    "truth_frequency_hash": common.identity_hash(
                        value.truth_frequencies
                    ),
                }
                for item, value in zip(objects, modes, strict=True)
            ],
            "row_root": common.f0_common.ROW_ROOTS["integration"],
            "schema": common.CORPUS_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        decisions = (
            field_decisions
            + [
                {
                    **row,
                    "schema": common.DECISION_SCHEMA,
                    "type": "field-mutation",
                }
                for row in field_mutations
            ]
            + [
                {
                    **row,
                    "schema": common.DECISION_SCHEMA,
                    "type": "structural-mutation",
                }
                for row in structural
            ]
        )
        access = {
            **common.ZERO_ACCESS,
            "b0_artifact_bytes_read": dependencies.b0_bytes_read,
            "f0_artifact_bytes_read": dependencies.f0_bytes_read,
            "integration_mesh_views_generated_after_commit": len(objects),
            "integration_physical_groups_generated_after_commit": 12,
            "schema": common.ACCESS_SCHEMA,
        }
        files = {
            "access-ledger.json": common.canonical_json(access),
            "corpus.json": common.canonical_json(corpus),
            "decisions.jsonl": common.canonical_json_lines(decisions),
            "endpoints.npz": _endpoint_npz(objects, modes, predictions["candidate"]),
            "fallback.json": common.canonical_json(
                {"records": fallback, "schema": common.FALLBACK_SCHEMA}
            ),
            "geometry.npz": common.geometry_npz(objects),
            "report.json": common.canonical_json(report),
        }
        artifact_hashes = {
            name: common.sha256_bytes(payload)
            for name, payload in sorted(files.items())
        }
        manifest = {
            "artifact_hashes": artifact_hashes,
            "b0_tree_digest": common.B0_TREE_DIGEST,
            "f0_tree_digest": common.F0_TREE_DIGEST,
            "implementation_commit": implementation_commit,
            "implementation_hashes": implementation,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        files["manifest.json"] = common.canonical_json(manifest)
        if set(files) != OUTPUT_FILES:
            raise common.I0Error(f"I0 output file set changed: {sorted(files)}")
        common.write_files(staging, files)
        file_bytes = sum(
            path.stat().st_size for path in staging.iterdir() if path.is_file()
        )
        if file_bytes > 100 * 1024 * 1024:
            raise common.I0Error("I0 output byte ceiling exceeded")
        if time.monotonic() - started > 600.0:
            raise common.I0Error("I0 runtime ceiling exceeded")
        if resource.getrusage(resource.RUSAGE_SELF).ru_maxrss > 4 * 1024 * 1024:
            raise common.I0Error("I0 RSS ceiling exceeded")
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
        raise common.I0Error("I0 compared output file count changed")
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--b0-root", type=Path, required=True)
    run_parser.add_argument("--f0-root", type=Path, required=True)
    run_parser.add_argument("--output", type=Path, required=True)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--left", type=Path, required=True)
    compare_parser.add_argument("--right", type=Path, required=True)
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    try:
        if arguments.command == "run":
            result = run(arguments.b0_root, arguments.f0_root, arguments.output)
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (common.I0Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
