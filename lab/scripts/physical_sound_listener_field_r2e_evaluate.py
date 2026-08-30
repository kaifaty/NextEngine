#!/usr/bin/env python3
"""Freeze R2E repetitions and perform the one authorized grouped query evaluation."""

from __future__ import annotations

import argparse
from collections import defaultdict
from pathlib import Path
from typing import Any

import physical_sound_listener_field_r2c_evaluate as r2c_evaluate
import physical_sound_listener_field_r2e_common as common
import physical_sound_listener_field_r2e_train as trainer

r2c = common.r2c

SELECTION_POLICY = (
    "candidate_must_strictly_beat_all_three_frozen_controls_on_all_five_"
    "primary_endpoints_else_reject_low_rank_coefficient_field"
)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["freeze", "evaluate"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--r2b-report", type=Path)
    parser.add_argument("--reference-root", type=Path)
    parser.add_argument("--training-report", action="append", type=Path)
    parser.add_argument("--manifest", type=Path)
    return parser.parse_args()


def implementation_paths() -> list[str]:
    return [
        "lab/scripts/physical_sound_listener_field_r2b_dense_preflight.py",
        "lab/scripts/physical_sound_listener_field_r2c_common.py",
        "lab/scripts/physical_sound_listener_field_r2c_evaluate.py",
        "lab/scripts/physical_sound_listener_field_r2e_common.py",
        "lab/scripts/physical_sound_listener_field_r2e_train.py",
        "lab/scripts/physical_sound_listener_field_r2e_evaluate.py",
    ]


def validate_r2b_report(
    root: Path, argument: Path
) -> tuple[Path, bytes, dict[str, Any]]:
    path = r2c.external_file(root, argument, "R2B preflight report")
    data, report = r2c.read_json(path, "R2B preflight report")
    if (
        r2c.sha256_bytes(data) != common.R2B_REPORT_SHA256
        or report.get("schema") != common.r2b.REPORT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "ReadyForComplexFieldTraining"
        or report.get("profile") != common.r2b.PROFILE
        or report.get("controls") is None
        or [control.get("control_id") for control in report.get("controls", [])]
        != list(common.r2b.CONTROL_IDS)
        or report.get("split", {}).get("query_rows") != common.r2b.QUERY_ROWS
        or report.get("split", {}).get("method_holdout_or_shadow_bytes_read")
        != 0
    ):
        raise common.R2EError("R2B frozen query controls changed")
    return path, data, report


def validate_training_report(
    root: Path, argument: Path
) -> tuple[Path, bytes, dict[str, Any], Path]:
    path = r2c.external_file(root, argument, "R2E training report")
    data, report = r2c.read_json(path, "R2E training report")
    if (
        report.get("schema") != common.TRAIN_REPORT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "R2ECandidateFrozenForQueryEvaluation"
        or report.get("revision") != common.REVISION
        or report.get("candidate_id") != common.CANDIDATE_ID
        or report.get("profile") != common.profile()
        or report.get("context_gate_pass") is not True
        or report.get("candidate_ready_for_query_evaluation") is not True
        or report.get("context_cook_failures") != []
        or report.get("query_cook_failures") != []
        or report.get("query_audio_bytes_read") != 0
        or report.get("query_audio_rows_read") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
        or report.get("quality_or_admission_authorized") is not False
    ):
        raise common.R2EError("R2E training report is not query-evaluation ready")
    predictions = report.get("query_predictions")
    if not isinstance(predictions, list) or len(predictions) != common.r2b.QUERY_ROWS:
        raise common.R2EError("R2E training report lacks the full query grid")
    indices = [record.get("row_index") for record in predictions]
    if indices != sorted(indices) or len(set(indices)) != len(indices):
        raise common.R2EError("R2E query predictions are not sorted and unique")
    for record in predictions:
        prediction = r2c.external_file(
            root,
            path.parent / record["prediction_file"],
            "R2E query prediction",
        )
        if (
            prediction.stat().st_size != record["prediction_byte_count"]
            or r2c.sha256_file(prediction) != record["prediction_sha256"]
        ):
            raise common.R2EError(
                f"R2E query prediction changed at row {record['row_index']}"
            )
    checkpoint = report.get("training", {}).get("checkpoint")
    if not isinstance(checkpoint, dict):
        raise common.R2EError("R2E checkpoint record changed")
    for file_key, hash_key in (
        ("descriptor_file", "descriptor_sha256"),
        ("weights_file", "weights_sha256"),
    ):
        checkpoint_path = r2c.external_file(
            root, path.parent / checkpoint[file_key], f"R2E {file_key}"
        )
        if r2c.sha256_file(checkpoint_path) != checkpoint[hash_key]:
            raise common.R2EError(f"R2E {file_key} changed")
    lineage_path = r2c.external_file(
        root, path.parent / "mlflow-lineage.json", "R2E MLflow lineage"
    )
    _, lineage = r2c.read_json(lineage_path, "R2E MLflow lineage")
    if (
        lineage.get("schema") != common.MLFLOW_SCHEMA
        or lineage.get("deterministic_training_report_sha256")
        != r2c.sha256_bytes(data)
        or lineage.get("checkpoint_weights_sha256")
        != checkpoint["weights_sha256"]
    ):
        raise common.R2EError("R2E MLflow lineage changed")
    return path, data, report, lineage_path


def freeze(
    root: Path,
    r2b_argument: Path | None,
    reference_argument: Path | None,
    training_arguments: list[Path] | None,
    output_argument: Path,
) -> None:
    if (
        r2b_argument is None
        or reference_argument is None
        or training_arguments is None
    ):
        raise common.R2EError(
            "freeze requires --r2b-report, --reference-root, and two "
            "--training-report values"
        )
    if len(training_arguments) != 2:
        raise common.R2EError("freeze requires exactly two R2E training reports")
    r2b_path, r2b_bytes, r2b_report = validate_r2b_report(root, r2b_argument)
    reference_root = r2c_evaluate.external_directory(
        root, reference_argument, "R2B reference root"
    )
    repetitions = []
    reports = []
    for argument in training_arguments:
        path, data, report, lineage_path = validate_training_report(root, argument)
        reports.append(report)
        repetitions.append(
            {
                "training_report": r2c.file_ref(path),
                "mlflow_lineage": r2c.file_ref(lineage_path),
                "training_report_sha256": r2c.sha256_bytes(data),
                "checkpoint_weights_sha256": report["training"]["checkpoint"][
                    "weights_sha256"
                ],
                "query_prediction_hashes": {
                    str(record["row_index"]): record["prediction_sha256"]
                    for record in report["query_predictions"]
                },
            }
        )
    repetitions.sort(key=lambda record: record["training_report"]["path"])
    first, second = repetitions
    if (
        first["training_report_sha256"] != second["training_report_sha256"]
        or first["checkpoint_weights_sha256"]
        != second["checkpoint_weights_sha256"]
        or first["query_prediction_hashes"] != second["query_prediction_hashes"]
        or reports[0]["manifest_sha256"] != reports[1]["manifest_sha256"]
    ):
        raise common.R2EError("R2E training repetitions are not byte reproducible")
    references = r2c_evaluate.reference_records(
        root, reference_root, r2b_report
    )
    output, staging = r2c.prepare_output(
        root, output_argument, "R2E evaluation freeze output"
    )
    try:
        manifest = {
            "schema": common.EVALUATION_MANIFEST_SCHEMA,
            "revision": common.REVISION,
            "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": r2c.source_records(
                root, implementation_paths()
            ),
            "r2b_preflight_report": r2c.file_ref(r2b_path),
            "training_manifest_sha256": reports[0]["manifest_sha256"],
            "training_repetitions": repetitions,
            "selected_training_report": first["training_report"],
            "query_references": references,
            "metric_sources": r2c.normalized_metric_sources(root),
            "primary_endpoints": list(common.PRIMARY_ENDPOINTS),
            "selection_policy": SELECTION_POLICY,
        }
        manifest_bytes = r2c.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": common.EVALUATION_FREEZE_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R2ECandidateReadyForOneGroupedQueryEvaluation",
            "claim": (
                "BYTE_IDENTICAL_CONTEXT_ONLY_CANDIDATE_REPETITIONS_AND_FROZEN_"
                "QUERY_EVALUATION / NO_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "training_report_sha256": first["training_report_sha256"],
            "checkpoint_weights_sha256": first["checkpoint_weights_sha256"],
            "query_prediction_count": len(first["query_prediction_hashes"]),
            "query_reference_rows_hashed_after_candidate_freeze": len(references),
            "query_reference_bytes_hashed_after_candidate_freeze": sum(
                record["reference_audio"]["byte_count"] for record in references
            ),
            "method_holdout_or_shadow_bytes_read": 0,
            "evaluation_runs_authorized": 1,
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
            "training_report_sha256": first["training_report_sha256"],
            "query_reference_rows_hashed": len(references),
        },
        (
            "output",
            "decision",
            "manifest_sha256",
            "training_report_sha256",
            "query_reference_rows_hashed",
        ),
    )


def validate_manifest(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    required = {
        "schema",
        "revision",
        "runner_sha256",
        "implementation_sources",
        "r2b_preflight_report",
        "training_manifest_sha256",
        "training_repetitions",
        "selected_training_report",
        "query_references",
        "metric_sources",
        "primary_endpoints",
        "selection_policy",
    }
    if set(manifest) != required:
        raise common.R2EError("R2E evaluation manifest fields changed")
    if (
        manifest.get("schema") != common.EVALUATION_MANIFEST_SCHEMA
        or manifest.get("revision") != common.REVISION
        or manifest.get("runner_sha256")
        != r2c.sha256_file(Path(__file__).resolve(strict=True))
        or manifest.get("implementation_sources")
        != r2c.source_records(root, implementation_paths())
        or manifest.get("metric_sources") != r2c.normalized_metric_sources(root)
        or manifest.get("primary_endpoints") != list(common.PRIMARY_ENDPOINTS)
        or manifest.get("selection_policy") != SELECTION_POLICY
    ):
        raise common.R2EError("R2E evaluation protocol changed")
    r2b_path = common.v1.validate_file_ref(
        root,
        manifest_path.parent,
        manifest["r2b_preflight_report"],
        "R2B preflight report",
    )
    _, r2b_bytes, r2b_report = validate_r2b_report(root, r2b_path)
    references = {}
    for record in manifest["query_references"]:
        path = common.v1.validate_file_ref(
            root,
            manifest_path.parent,
            record["reference_audio"],
            "R2E query reference",
        )
        references[record["row_index"]] = path
    selected_path = common.v1.validate_file_ref(
        root,
        manifest_path.parent,
        manifest["selected_training_report"],
        "selected R2E training report",
    )
    _, _, training_report, _ = validate_training_report(root, selected_path)
    if training_report["manifest_sha256"] != manifest["training_manifest_sha256"]:
        raise common.R2EError("selected R2E training manifest lineage changed")
    prediction_indices = {
        record["row_index"] for record in training_report["query_predictions"]
    }
    if prediction_indices != set(references):
        raise common.R2EError("R2E query prediction/reference identities differ")
    adapted_report = {
        **training_report,
        "predictions": training_report["query_predictions"],
    }
    return {
        "r2b_bytes": r2b_bytes,
        "r2b_report": r2b_report,
        "references": references,
        "training": {"path": selected_path, "report": adapted_report},
    }


def grouped_metrics(
    rows: list[dict[str, Any]],
    predictions: list[dict[str, Any]],
) -> dict[str, list[dict[str, Any]]]:
    metadata = {record["row_index"]: record for record in predictions}
    dimensions = {
        "azimuth_degrees": defaultdict(list),
        "gantry_distance_offset_millimetres": defaultdict(list),
        "microphone_id": defaultdict(list),
    }
    for row in rows:
        record = metadata[row["row_index"]]
        for dimension, groups in dimensions.items():
            groups[record[dimension]].append(row)
    result = {}
    for dimension, groups in dimensions.items():
        result[dimension] = [
            {
                "value": value,
                "row_count": len(values),
                "aggregate": r2c.aggregate_metric_rows(values),
            }
            for value, values in sorted(groups.items())
        ]
    return result


def evaluate(
    root: Path, manifest_argument: Path | None, output_argument: Path
) -> None:
    if manifest_argument is None:
        raise common.R2EError("evaluate requires --manifest")
    manifest_path = r2c.external_file(
        root, manifest_argument, "R2E evaluation manifest"
    )
    manifest_bytes, manifest = r2c.read_json(
        manifest_path, "R2E evaluation manifest"
    )
    inputs = validate_manifest(root, manifest_path, manifest)
    output, staging = r2c.prepare_output(
        root, output_argument, "R2E evaluation output"
    )
    try:
        evaluated = r2c_evaluate.evaluate_candidate(
            root,
            common.CANDIDATE_ID,
            inputs["training"],
            inputs["references"],
            staging,
        )
        compared = r2c_evaluate.compare_controls(
            evaluated, inputs["r2b_report"]["controls"]
        )
        passed = compared["passes_frozen_r2c_rule"]
        decision = (
            "GoLowRankCoefficientField"
            if passed
            else "RejectLowRankCoefficientField"
        )
        selected = common.CANDIDATE_ID if passed else None
        predictions = inputs["training"]["report"]["query_predictions"]
        report = {
            "schema": common.EVALUATION_REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "claim": (
                "GROUPED_FIXED_IMPACT_LOW_RANK_COEFFICIENT_FIELD_DEVELOPMENT_"
                "DECISION / NO_METHOD_HOLDOUT_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "revision": common.REVISION,
            "manifest_sha256": r2c.sha256_bytes(manifest_bytes),
            "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
            "r2b_preflight_report_sha256": r2c.sha256_bytes(inputs["r2b_bytes"]),
            "primary_endpoints": list(common.PRIMARY_ENDPOINTS),
            "selection_policy": SELECTION_POLICY,
            "selected_candidate_id": selected,
            "controls": inputs["r2b_report"]["controls"],
            "candidate": compared,
            "grouped_diagnostics": grouped_metrics(
                evaluated["rows"], predictions
            ),
            "query_audio_rows_read_after_candidate_freeze": common.r2b.QUERY_ROWS,
            "query_audio_bytes_read_after_candidate_freeze": (
                common.r2b.QUERY_ROWS * common.r2b.SAMPLE_COUNT * 2
            ),
            "method_holdout_or_shadow_bytes_read": 0,
            "quality_admission_or_runtime_authorized": False,
            "next_action": (
                "freeze_exact_object_multi_impact_data_boundary"
                if passed
                else "stop_fixed_impact_low_rank_listener_field_family"
            ),
        }
        report_bytes = r2c.canonical_json(report)
        (staging / "evaluation-report.json").write_bytes(report_bytes)
        r2c.publish_staging(staging, output)
    except BaseException:
        r2c.discard_staging(staging)
        raise
    r2c.emit_summary(
        {
            "output": str(output),
            "decision": decision,
            "selected_candidate_id": selected,
            "evaluation_report_sha256": r2c.sha256_bytes(report_bytes),
            "candidate_aggregate": compared["aggregate"],
            "endpoint_passes": {
                row["endpoint"]: row["strictly_below_all_controls"]
                for row in compared["primary_endpoint_comparison"]
            },
        },
        (
            "output",
            "decision",
            "selected_candidate_id",
            "evaluation_report_sha256",
            "candidate_aggregate",
            "endpoint_passes",
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
            arguments.r2b_report,
            arguments.reference_root,
            arguments.training_report,
            arguments.output,
        )
    else:
        if (
            arguments.r2b_report is not None
            or arguments.reference_root is not None
            or arguments.training_report is not None
        ):
            raise common.R2EError("evaluate accepts only the frozen --manifest")
        evaluate(root, arguments.manifest, arguments.output)


if __name__ == "__main__":
    main()
