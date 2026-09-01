#!/usr/bin/env python3
"""Official frozen B0+C0+F0 integration runner for Physical Sound V20 I1."""

from __future__ import annotations

import argparse
import json
import resource
import sys
import time
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v19_f0_oracle as field_oracle
import physical_sound_v19_i0_oracle as i0_oracle
import physical_sound_v20_i1_common as common
import physical_sound_v20_m0b_metric_lab as metric_lab

MRSC_THRESHOLD = 0.6860209112961239
MCLM_THRESHOLD = 0.46162132517706367
DSR_THRESHOLD = 0.14714357326302258
TE_THRESHOLD = 0.09215386292722896
IDENTITY_TOLERANCE = metric_lab.IDENTITY_TOLERANCE
CASE_COUNT = metric_lab.CASE_COUNT
CONTROL_NAMES = (
    "identity",
    "combined",
    "global-polarity",
    "alternating-gain-sign",
    "frequency-uniform-090-cents",
    "frequency-alternating-090-cents",
    "damping-positive-0p35",
    "damping-negative-0p35",
    "mode-removal-2",
    "onset-delay-64-samples",
    "sample-zero-impulse-008",
)
OUTPUT_FILES = {
    "access-ledger.json",
    "corpus.json",
    "decisions.jsonl",
    "endpoints.npz",
    "fallback.json",
    "geometry.npz",
    "manifest.json",
    "metrics.jsonl",
    "report.json",
}


def control_specs() -> tuple[metric_lab.m0_lab.ControlSpec, ...]:
    by_name = {item.name: item for item in metric_lab.control_specs()}
    controls = tuple(by_name[name] for name in CONTROL_NAMES)
    if tuple(item.name for item in controls) != CONTROL_NAMES:
        raise common.I1Error("I1 counterfactual control order changed")
    return controls


def build_corpus(
    dependencies: common.m0b_common.m0_common.i0_common.LoadedDependencies,
    objects: tuple[common.m0b_common.m0_common.i0_common.f0_common.FieldObject, ...],
) -> tuple[metric_lab.m0_lab.EvaluationCorpus, list[dict[str, Any]]]:
    i0 = common.m0b_common.m0_common.i0_common
    modes = tuple(i0.global_modes(dependencies.b0, item.row) for item in objects)
    predictions, field_metrics, field_decisions = field_oracle._prediction_evaluation(
        dependencies.f0_model, dependencies.gain_scale, objects
    )
    candidate = predictions["candidate"]
    case_records: list[dict[str, Any]] = []
    object_indices: list[int] = []
    vertices: list[int] = []
    truth_frequencies: list[np.ndarray] = []
    truth_damping: list[np.ndarray] = []
    truth_gains: list[np.ndarray] = []
    predicted_frequencies: list[np.ndarray] = []
    predicted_damping: list[np.ndarray] = []
    predicted_gains: list[np.ndarray] = []
    for object_index, (item, mode, field) in enumerate(
        zip(objects, modes, candidate, strict=True)
    ):
        selected = metric_lab.m0_lab.selected_queries(item.accepted_query)
        for query_vertex in selected:
            query = int(query_vertex)
            case_index = len(case_records)
            case_records.append(
                {
                    "case_index": case_index,
                    "is_twin": item.is_twin,
                    "object_id": item.row.object_id,
                    "physical_group_id": item.row.physical_group_id,
                    "query_vertex": query,
                }
            )
            object_indices.append(object_index)
            vertices.append(query)
            truth_frequencies.append(mode.truth_frequencies)
            truth_damping.append(mode.truth_damping)
            truth_gains.append(item.gains[query])
            predicted_frequencies.append(mode.predicted_frequencies)
            predicted_damping.append(mode.predicted_damping)
            predicted_gains.append(field[query])
    if len(case_records) != CASE_COUNT:
        raise common.I1Error(f"I1 waveform case count changed: {len(case_records)}")
    corpus = metric_lab.m0_lab.EvaluationCorpus(
        objects=objects,
        modes=modes,
        candidate=candidate,
        case_records=tuple(case_records),
        case_object_index=np.asarray(object_indices, dtype=np.int64),
        query_vertices=np.asarray(vertices, dtype=np.int64),
        truth_frequencies=np.stack(truth_frequencies),
        truth_damping=np.stack(truth_damping),
        truth_gains=np.stack(truth_gains),
        predicted_frequencies=np.stack(predicted_frequencies),
        predicted_damping=np.stack(predicted_damping),
        predicted_gains=np.stack(predicted_gains),
        field_metrics=field_metrics,
    )
    return corpus, field_decisions


def _fallback_records(
    objects: tuple[common.m0b_common.m0_common.i0_common.f0_common.FieldObject, ...],
) -> list[dict[str, Any]]:
    return [
        {**row, "schema": common.FALLBACK_SCHEMA}
        for row in i0_oracle._fallback_records(objects)
    ]


def _component_evidence(
    dependencies: common.m0b_common.m0_common.i0_common.LoadedDependencies,
    corpus: metric_lab.m0_lab.EvaluationCorpus,
    field_decisions: list[dict[str, Any]],
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    remesh = field_oracle._remesh_metrics(
        corpus.objects, corpus.candidate, corpus.field_metrics["candidate"]
    )
    mutations = field_oracle._mutation_metrics(
        dependencies.f0_model,
        dependencies.gain_scale,
        corpus.objects,
        corpus.candidate,
    )
    structural = field_oracle._structural_metrics(corpus.objects)
    field_gates = field_oracle._gates(
        corpus.objects,
        corpus.field_metrics,
        remesh,
        mutations,
        structural,
        True,
    )
    global_metrics = i0_oracle._global_metrics(corpus.modes)
    fallback = _fallback_records(corpus.objects)
    rejected = {
        (item.row.object_id, int(vertex))
        for item in corpus.objects
        for vertex in item.rejected_query
    }
    fallback_complete = i0_oracle._fallback_complete(rejected, fallback)
    hard_mutations = i0_oracle._hard_mutations(corpus.objects, corpus.modes, fallback)
    global_gates = {
        "damping": global_metrics["damping_relative_median"] <= 0.08
        and global_metrics["damping_relative_p95"] <= 0.20,
        "frequency": global_metrics["frequency_cents_median"] <= 20.0
        and global_metrics["frequency_cents_p95"] <= 60.0,
        "modal_peak": global_metrics["frequency_cents_max"] <= 60.0,
    }
    hard_modes = all(
        i0_oracle._hard_modes(value.predicted_frequencies, value.predicted_damping)
        for value in corpus.modes
    )
    decisions = [
        {
            **row,
            "schema": common.DECISION_SCHEMA,
            "source_schema": row.get("schema"),
            "type": row.get("type", "field-decision"),
        }
        for row in field_decisions
    ]
    decisions.extend(
        {
            **row,
            "schema": common.DECISION_SCHEMA,
            "type": "field-mutation",
        }
        for row in mutations
    )
    decisions.extend(
        {
            **row,
            "schema": common.DECISION_SCHEMA,
            "type": "structural-mutation",
        }
        for row in structural
    )
    evidence = {
        "dependency_identity": i0_oracle._dependency_claim_valid(
            common.B0_TREE_DIGEST, common.F0_TREE_DIGEST
        ),
        "fallback_complete": fallback_complete,
        "field_gates": field_gates,
        "field_metrics": {
            name: field_oracle._metric_summary(rows)
            for name, rows in corpus.field_metrics.items()
        },
        "global_gates": global_gates,
        "global_metrics": global_metrics,
        "hard_modes": hard_modes,
        "hard_mutations": hard_mutations,
        "pass": bool(
            all(field_gates.values())
            and all(global_gates.values())
            and hard_modes
            and fallback_complete
            and all(value == 12 for value in hard_mutations.values())
        ),
        "remesh": remesh,
    }
    return evidence, decisions, fallback


def _actual_gates(summaries: dict[str, dict[str, Any]]) -> dict[str, bool]:
    metrics = summaries["combined"]["metrics"]
    return {
        "decay_slope_residual": metrics["decay_slope_residual"]["p95"] <= DSR_THRESHOLD,
        "mean_centered_log_magnitude": metrics["mean_centered_log_magnitude"]["p95"]
        <= MCLM_THRESHOLD,
        "mrsc": metrics["mrsc"]["p95"] <= MRSC_THRESHOLD,
        "transient_energy": metrics["transient_energy"]["p95"] <= TE_THRESHOLD,
    }


def _counterfactual_gates(
    summaries: dict[str, dict[str, Any]],
) -> tuple[dict[str, bool], dict[str, bool]]:
    physical = metric_lab.m0_lab._physical_control_gates(summaries)
    polarity_gain = summaries["global-polarity"]["metrics"]["gain_nrmse"]
    physical["global_polarity_signed_gain_rejects"] = not (
        polarity_gain["mean"] <= 0.25 and polarity_gain["max"] <= 0.40
    )

    def spectral_reject(name: str) -> bool:
        metrics = summaries[name]["metrics"]
        return bool(
            metrics["mrsc"]["p95"] > MRSC_THRESHOLD
            or metrics["mean_centered_log_magnitude"]["p95"] > MCLM_THRESHOLD
        )

    identity = summaries["identity"]["metrics"]
    polarity = summaries["global-polarity"]["metrics"]
    gates = {
        "alternating_gain_sign": bool(
            physical["alternating_gain_sign_rejects"]
            and spectral_reject("alternating-gain-sign")
        ),
        "damping_negative": bool(
            physical["damping_negative_035_rejects"]
            and summaries["damping-negative-0p35"]["metrics"]["decay_slope_residual"][
                "p95"
            ]
            > DSR_THRESHOLD
        ),
        "damping_positive": bool(
            physical["damping_positive_035_rejects"]
            and summaries["damping-positive-0p35"]["metrics"]["decay_slope_residual"][
                "p95"
            ]
            > DSR_THRESHOLD
        ),
        "frequency_alternating": bool(
            physical["frequency_alternating_090_rejects"]
            and spectral_reject("frequency-alternating-090-cents")
        ),
        "frequency_uniform": bool(
            physical["frequency_uniform_090_rejects"]
            and spectral_reject("frequency-uniform-090-cents")
        ),
        "identity": all(
            identity[name]["max"] <= IDENTITY_TOLERANCE
            for name in metric_lab.ACOUSTIC_METRICS
        ),
        "mode_removal": bool(
            physical["mode_removal_two_rejects"] and spectral_reject("mode-removal-2")
        ),
        "onset_delay": summaries["onset-delay-64-samples"]["metrics"][
            "transient_energy"
        ]["p95"]
        > TE_THRESHOLD,
        "polarity": bool(
            all(
                polarity[name]["max"] <= IDENTITY_TOLERANCE
                for name in metric_lab.ACOUSTIC_METRICS
            )
            and polarity["full_waveform_nrmse"]["min"] >= 1.9
            and physical["global_polarity_signed_gain_rejects"]
        ),
        "sample_zero_impulse": summaries["sample-zero-impulse-008"]["metrics"][
            "transient_energy"
        ]["p95"]
        > TE_THRESHOLD,
    }
    return gates, physical


def _metric_roundtrip_exact(rows: list[dict[str, Any]]) -> bool:
    payload = common.canonical_json_lines(rows)
    decoded = [json.loads(line) for line in payload.decode("utf-8").splitlines()]
    return decoded == rows


def _corpus_record(corpus: metric_lab.m0_lab.EvaluationCorpus) -> dict[str, Any]:
    cases_by_object: dict[str, list[int]] = {}
    for row in corpus.case_records:
        cases_by_object.setdefault(row["object_id"], []).append(row["query_vertex"])
    objects = []
    for item, mode, prediction in zip(
        corpus.objects, corpus.modes, corpus.candidate, strict=True
    ):
        selected = cases_by_object[item.row.object_id]
        objects.append(
            {
                **item.record(),
                "predicted_damping_hash": common.identity_hash(mode.predicted_damping),
                "predicted_frequency_hash": common.identity_hash(
                    mode.predicted_frequencies
                ),
                "predicted_gain_hash": common.identity_hash(prediction),
                "selected_query_count": len(selected),
                "selected_query_hash": common.identity_hash(
                    np.asarray(selected, dtype=np.int64)
                ),
                "selected_query_vertices": selected,
                "truth_damping_hash": common.identity_hash(mode.truth_damping),
                "truth_frequency_hash": common.identity_hash(mode.truth_frequencies),
            }
        )
    return {
        "b0_tree_digest": common.B0_TREE_DIGEST,
        "case_count": len(corpus.case_records),
        "case_root": common.sha256_bytes(common.canonical_json(corpus.case_records)),
        "f0_tree_digest": common.F0_TREE_DIGEST,
        "i1_row_root": common.I1_ROW_ROOT,
        "objects": objects,
        "schema": common.CORPUS_SCHEMA,
        "study_id": common.STUDY_ID,
        "view_count": len(corpus.objects),
    }


def run(m0b_root: Path, b0_root: Path, f0_root: Path, output: Path) -> dict[str, Any]:
    started = time.monotonic()
    environment = common.verify_protocol_environment()
    implementation_commit = common.verify_committed_implementation()
    implementation = common.implementation_hashes()
    certificate = common.load_m0b_certificate(m0b_root)
    dependencies = common.load_dependencies(b0_root, f0_root)
    staging, target = common.prepare_output(output)
    try:
        # This is the first operation allowed to generate any fresh I1 value.
        objects = common.generate_i1_objects(implementation_commit)
        corpus, field_decisions = build_corpus(dependencies, objects)
        component, decisions, fallback = _component_evidence(
            dependencies, corpus, field_decisions
        )
        controls = control_specs()
        truth = metric_lab.build_truth_cache(corpus)
        rows = metric_lab.evaluate_controls(corpus, truth, controls)
        rows = [{**row, "schema": common.METRIC_SCHEMA} for row in rows]
        summaries = metric_lab._summaries(rows, controls)
        actual = _actual_gates(summaries)
        counterfactual, physical_controls = _counterfactual_gates(summaries)
        metric_roundtrip = _metric_roundtrip_exact(rows)
        endpoint = i0_oracle._endpoint_npz(
            corpus.objects, corpus.modes, corpus.candidate
        )
        geometry = common.geometry_npz(corpus.objects)
        binary_roundtrip = bool(
            endpoint
            == i0_oracle._endpoint_npz(corpus.objects, corpus.modes, corpus.candidate)
            and geometry == common.geometry_npz(corpus.objects)
        )
        complete_finite = bool(
            len(corpus.objects) == 24
            and len(corpus.case_records) == CASE_COUNT
            and len(rows) == CASE_COUNT * len(controls)
            and metric_lab._finite(rows)
            and metric_lab._finite(summaries)
        )
        gates = {
            "actual_acoustic": all(actual.values()),
            "component_hard_physical": bool(component["pass"]),
            "complete_finite": complete_finite,
            "counterfactual_non_regression": all(counterfactual.values()),
            "lineage": bool(
                common.tree_digest(certificate.file_map) == common.M0B_TREE_DIGEST
            ),
            "resource_ceiling_enforced": True,
            "serialization": metric_roundtrip and binary_roundtrip,
            "zero_forbidden_access": all(
                value == 0 for value in common.ZERO_ACCESS.values()
            ),
        }
        control_decisions = [
            {
                "control": name,
                "metrics": summaries[name]["metrics"],
                "schema": common.DECISION_SCHEMA,
                "type": "actual-candidate" if name == "combined" else "counterfactual",
            }
            for name in CONTROL_NAMES
        ]
        decisions.extend(control_decisions)
        report = {
            "actual_gates": actual,
            "component_evidence": component,
            "control_summaries": summaries,
            "counterfactual_gates": counterfactual,
            "gates": gates,
            "physical_control_gates": physical_controls,
            "schema": common.REPORT_SCHEMA,
            "single_run_pass": all(gates.values()),
            "study_id": common.STUDY_ID,
            "thresholds": {
                "decay_slope_residual_p95": DSR_THRESHOLD,
                "mean_centered_log_magnitude_p95": MCLM_THRESHOLD,
                "mrsc_p95": MRSC_THRESHOLD,
                "transient_energy_p95": TE_THRESHOLD,
            },
        }
        access = {
            **common.ZERO_ACCESS,
            "b0_artifact_bytes_read": dependencies.b0_bytes_read,
            "f0_artifact_bytes_read": dependencies.f0_bytes_read,
            "i1_control_count": len(controls),
            "i1_physical_groups_generated_after_commit": 12,
            "i1_views_generated_after_commit": len(corpus.objects),
            "i1_waveform_cases_evaluated": len(corpus.case_records),
            "m0b_certificate_bytes_read": certificate.bytes_read,
            "schema": common.ACCESS_SCHEMA,
        }
        files = {
            "access-ledger.json": common.canonical_json(access),
            "corpus.json": common.canonical_json(_corpus_record(corpus)),
            "decisions.jsonl": common.canonical_json_lines(decisions),
            "endpoints.npz": endpoint,
            "fallback.json": common.canonical_json(
                {"records": fallback, "schema": common.FALLBACK_SCHEMA}
            ),
            "geometry.npz": geometry,
            "metrics.jsonl": common.canonical_json_lines(rows),
            "report.json": common.canonical_json(report),
        }
        artifact_hashes = {
            name: common.sha256_bytes(payload)
            for name, payload in sorted(files.items())
        }
        manifest = {
            "artifact_hashes": artifact_hashes,
            "b0_tree_digest": common.B0_TREE_DIGEST,
            "environment": environment,
            "f0_tree_digest": common.F0_TREE_DIGEST,
            "i1_row_root": common.I1_ROW_ROOT,
            "implementation_commit": implementation_commit,
            "implementation_hashes": implementation,
            "m0b_implementation_commit": common.M0B_IMPLEMENTATION_COMMIT,
            "m0b_implementation_hashes": common.M0B_IMPLEMENTATION_HASHES,
            "m0b_tree_digest": common.M0B_TREE_DIGEST,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        files["manifest.json"] = common.canonical_json(manifest)
        if set(files) != OUTPUT_FILES:
            raise common.I1Error(f"I1 output file set changed: {sorted(files)}")
        total_bytes = sum(len(payload) for payload in files.values())
        if total_bytes > 100 * 1024 * 1024:
            raise common.I1Error("I1 output byte ceiling exceeded")
        if time.monotonic() - started > 600.0:
            raise common.I1Error("I1 runtime ceiling exceeded")
        if resource.getrusage(resource.RUSAGE_SELF).ru_maxrss > 4 * 1024 * 1024:
            raise common.I1Error("I1 RSS ceiling exceeded")
        common.write_files(staging, files)
        if time.monotonic() - started > 600.0:
            raise common.I1Error("I1 runtime ceiling exceeded after serialization")
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
        raise common.I1Error("I1 compared output file count changed")
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--m0b-root", type=Path, required=True)
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
            result = run(
                arguments.m0b_root,
                arguments.b0_root,
                arguments.f0_root,
                arguments.output,
            )
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (common.I1Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
