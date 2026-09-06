#!/usr/bin/env python3
"""Freeze repetitions and evaluate R2C candidates once on grouped queries."""

from __future__ import annotations

import argparse
import math
import subprocess
import wave
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_listener_field_r2c as trainer
import physical_sound_listener_field_r2c_common as common

RUST_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-validator.manifest.v1"
RUST_REPORT_SCHEMA = "nextengine.experimental-physical-sound-validator.report.v1"
SELECTION_POLICY = (
    "candidate_must_strictly_beat_all_three_controls_on_all_five_endpoints;_"
    "if_both_pass_choose_helmholtz_only_if_it_strictly_beats_data_only_on_all_"
    "five_else_choose_data_only"
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
        "lab/scripts/physical_sound_listener_field_r2c.py",
        "lab/scripts/physical_sound_listener_field_r2c_common.py",
        "lab/scripts/physical_sound_listener_field_r2c_evaluate.py",
    ]


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise common.R2CError(f"{label} must be an external directory: {resolved}")
    return resolved


def resolve_output_file(
    root: Path, report_path: Path, reference: dict[str, Any], label: str
) -> Path:
    ref = common.require_ref(reference, label)
    unresolved = Path(ref["path"])
    path = unresolved if unresolved.is_absolute() else report_path.parent / unresolved
    path = common.external_file(root, path, label)
    if path.stat().st_size != ref["byte_count"] or common.sha256_file(path) != ref["sha256"]:
        raise common.R2CError(f"{label} changed")
    return path


def validate_training_report(
    root: Path, path: Path
) -> tuple[bytes, dict[str, Any], dict[str, Any]]:
    report_path = common.external_file(root, path, "R2C training report")
    data, report = common.read_json(report_path, "R2C training report")
    candidate = common.exact_candidate(report.get("candidate_id"))
    if (
        report.get("schema") != common.TRAINING_SCHEMA
        or report.get("status") != "Trained"
        or report.get("decision") != "FrozenCandidateReadyForQueryEvaluation"
        or report.get("revision") != common.REVISION
        or report.get("runner_sha256")
        != common.sha256_file(Path(trainer.__file__).resolve(strict=True))
        or report.get("environment") != common.environment_profile()
        or report.get("candidate_profile") != common.candidate_profile()
        or report.get("helmholtz_weight") != candidate["helmholtz_weight"]
        or report.get("optimizer_steps") != common.STEPS
        or report.get("query_audio_bytes_read") != 0
        or report.get("query_audio_rows_read") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
        or report.get("checkpoint_selection")
        != "final_step_only_no_query_or_early_stopping"
        or report.get("quality_or_admission_authorized") is not False
        or report.get("cook_failures") != []
    ):
        raise common.R2CError("R2C training report boundary changed")
    predictions = report.get("predictions")
    if not isinstance(predictions, list) or len(predictions) != common.r2b.QUERY_ROWS:
        raise common.R2CError("R2C candidate does not cover every grouped query")
    indices = [record.get("row_index") for record in predictions]
    if indices != sorted(indices) or len(set(indices)) != len(indices):
        raise common.R2CError("R2C prediction rows must be sorted and unique")
    for record in predictions:
        path = report_path.parent / record["prediction_file"]
        path = common.external_file(root, path, "R2C prediction WAV")
        if (
            path.stat().st_size != record["prediction_byte_count"]
            or common.sha256_file(path) != record["prediction_sha256"]
        ):
            raise common.R2CError(f"R2C prediction changed at row {record['row_index']}")
    checkpoint = report.get("checkpoint")
    if not isinstance(checkpoint, dict):
        raise common.R2CError("R2C checkpoint record changed")
    for file_key, hash_key in (
        ("descriptor_file", "descriptor_sha256"),
        ("weights_file", "weights_sha256"),
    ):
        file_path = common.external_file(
            root, report_path.parent / checkpoint[file_key], f"R2C {file_key}"
        )
        if common.sha256_file(file_path) != checkpoint[hash_key]:
            raise common.R2CError(f"R2C {file_key} changed")
    lineage_path = common.external_file(
        root, report_path.parent / "mlflow-lineage.json", "R2C MLflow lineage"
    )
    lineage_bytes, lineage = common.read_json(lineage_path, "R2C MLflow lineage")
    if (
        lineage.get("schema") != common.MLFLOW_SCHEMA
        or lineage.get("candidate_id") != report["candidate_id"]
        or lineage.get("deterministic_training_report_sha256")
        != common.sha256_bytes(data)
        or lineage.get("checkpoint_weights_sha256") != checkpoint["weights_sha256"]
    ):
        raise common.R2CError("R2C MLflow lineage changed")
    return data, report, {
        "report_path": report_path,
        "lineage_path": lineage_path,
        "lineage_bytes": lineage_bytes,
    }


def reference_records(
    root: Path, reference_root: Path, r2b_report: dict[str, Any]
) -> list[dict[str, Any]]:
    records_by_control = r2b_report.get("control_prediction_records", {})
    first_id = common.r2b.CONTROL_IDS[0]
    source = records_by_control.get(first_id)
    if not isinstance(source, list) or len(source) != common.r2b.QUERY_ROWS:
        raise common.R2CError("R2B query reference records changed")
    records = []
    for record in source:
        unresolved = Path(record["reference_path"])
        path = (reference_root / unresolved).resolve(strict=True)
        if not path.is_relative_to(reference_root) or not path.is_file():
            raise common.R2CError("R2B reference path escaped its external root")
        if common.sha256_file(path) != record["reference_sha256"]:
            raise common.R2CError(f"R2B reference changed at row {record['row_index']}")
        records.append(
            {
                "row_index": record["row_index"],
                "reference_audio": common.file_ref(path),
            }
        )
    records.sort(key=lambda record: record["row_index"])
    if len({record["row_index"] for record in records}) != common.r2b.QUERY_ROWS:
        raise common.R2CError("R2B reference row identity changed")
    return records


def freeze_evaluation(
    root: Path,
    r2b_argument: Path | None,
    reference_argument: Path | None,
    training_arguments: list[Path] | None,
    output_argument: Path,
) -> None:
    if r2b_argument is None or reference_argument is None or training_arguments is None:
        raise common.R2CError(
            "evaluation freeze requires --r2b-report, --reference-root, and four --training-report values"
        )
    if len(training_arguments) != 4:
        raise common.R2CError("evaluation freeze requires exactly four training reports")
    r2b_path = common.external_file(root, r2b_argument, "R2B preflight report")
    r2b_bytes, r2b_report = common.read_json(r2b_path, "R2B preflight report")
    if common.sha256_bytes(r2b_bytes) != common.R2B_REPORT_SHA256:
        raise common.R2CError("R2B preflight report changed before evaluation freeze")
    reference_root = external_directory(root, reference_argument, "R2B reference root")
    repetitions: dict[str, list[dict[str, Any]]] = {
        candidate["candidate_id"]: [] for candidate in common.CANDIDATES
    }
    manifest_sha = None
    for argument in training_arguments:
        data, report, lineage = validate_training_report(root, argument)
        if manifest_sha is None:
            manifest_sha = report["manifest_sha256"]
        elif report["manifest_sha256"] != manifest_sha:
            raise common.R2CError("R2C repetitions do not share one training manifest")
        repetitions[report["candidate_id"]].append(
            {
                "training_report": common.file_ref(lineage["report_path"]),
                "mlflow_lineage": common.file_ref(lineage["lineage_path"]),
                "training_report_sha256": common.sha256_bytes(data),
                "checkpoint_weights_sha256": report["checkpoint"]["weights_sha256"],
            }
        )
    selected = {}
    for candidate_id, records in repetitions.items():
        if len(records) != 2:
            raise common.R2CError(f"R2C candidate {candidate_id} requires two repetitions")
        records.sort(key=lambda record: record["training_report"]["path"])
        if (
            records[0]["training_report_sha256"]
            != records[1]["training_report_sha256"]
            or records[0]["checkpoint_weights_sha256"]
            != records[1]["checkpoint_weights_sha256"]
        ):
            raise common.R2CError(f"R2C candidate {candidate_id} is not byte reproducible")
        selected[candidate_id] = records[0]["training_report"]
    references = reference_records(root, reference_root, r2b_report)
    output, staging = common.prepare_output(root, output_argument, "R2C evaluation freeze")
    try:
        manifest = {
            "schema": common.EVALUATION_MANIFEST_SCHEMA,
            "revision": common.REVISION,
            "runner_sha256": common.sha256_file(Path(__file__).resolve(strict=True)),
            "implementation_sources": common.source_records(root, implementation_paths()),
            "r2b_preflight_report": common.file_ref(r2b_path),
            "training_manifest_sha256": manifest_sha,
            "training_repetitions": repetitions,
            "selected_training_reports": selected,
            "query_references": references,
            "metric_sources": common.normalized_metric_sources(root),
            "primary_endpoints": list(common.PRIMARY_ENDPOINTS),
            "selection_policy": SELECTION_POLICY,
        }
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": "nextengine.experimental-physical-sound-listener-field-r2c-evaluation-freeze.report.v1",
            "status": "Validated",
            "decision": "R2CFrozenCandidatesReadyForOneQueryEvaluation",
            "claim": (
                "BYTE_IDENTICAL_TRAINING_REPETITIONS_AND_FROZEN_QUERY_EVALUATION_"
                "ONLY / NO_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "training_report_sha256_by_candidate": {
                candidate_id: records[0]["training_report_sha256"]
                for candidate_id, records in repetitions.items()
            },
            "checkpoint_weights_sha256_by_candidate": {
                candidate_id: records[0]["checkpoint_weights_sha256"]
                for candidate_id, records in repetitions.items()
            },
            "query_reference_rows_hashed_after_candidate_freeze": len(references),
            "query_reference_bytes_hashed_after_candidate_freeze": sum(
                record["reference_audio"]["byte_count"] for record in references
            ),
            "method_holdout_or_shadow_bytes_read": 0,
            "evaluation_runs_authorized": 1,
            "quality_or_admission_authorized": False,
        }
        report_bytes = common.canonical_json(report)
        (staging / "freeze-report.json").write_bytes(report_bytes)
        common.publish_staging(staging, output)
    except BaseException:
        common.discard_staging(staging)
        raise
    common.emit_summary(
        {
            "output": str(output),
            "decision": report["decision"],
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "training_report_sha256_by_candidate": report[
                "training_report_sha256_by_candidate"
            ],
        },
        (
            "output",
            "decision",
            "manifest_sha256",
            "training_report_sha256_by_candidate",
        ),
    )


def validate_evaluation_manifest(
    root: Path, path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    expected = {
        "schema",
        "revision",
        "runner_sha256",
        "implementation_sources",
        "r2b_preflight_report",
        "training_manifest_sha256",
        "training_repetitions",
        "selected_training_reports",
        "query_references",
        "metric_sources",
        "primary_endpoints",
        "selection_policy",
    }
    if set(manifest) != expected:
        raise common.R2CError("R2C evaluation manifest fields changed")
    if (
        manifest.get("schema") != common.EVALUATION_MANIFEST_SCHEMA
        or manifest.get("revision") != common.REVISION
        or manifest.get("runner_sha256")
        != common.sha256_file(Path(__file__).resolve(strict=True))
        or manifest.get("implementation_sources")
        != common.source_records(root, implementation_paths())
        or manifest.get("metric_sources") != common.normalized_metric_sources(root)
        or manifest.get("primary_endpoints") != list(common.PRIMARY_ENDPOINTS)
        or manifest.get("selection_policy") != SELECTION_POLICY
    ):
        raise common.R2CError("R2C evaluation protocol changed")
    r2b_path, r2b_bytes = common.resolve_ref(
        root, path.parent, manifest["r2b_preflight_report"], "R2B preflight report"
    )
    _, r2b_report = common.read_json(r2b_path, "R2B preflight report")
    if common.sha256_bytes(r2b_bytes) != common.R2B_REPORT_SHA256:
        raise common.R2CError("R2B control report changed")
    references = {}
    for record in manifest["query_references"]:
        reference_path, _ = common.resolve_ref(
            root, path.parent, record["reference_audio"], "R2C query reference"
        )
        references[record["row_index"]] = reference_path
    training = {}
    for candidate in common.CANDIDATES:
        candidate_id = candidate["candidate_id"]
        selected = manifest["selected_training_reports"].get(candidate_id)
        selected_path, _ = common.resolve_ref(
            root, path.parent, selected, f"selected {candidate_id} training report"
        )
        _, report, _ = validate_training_report(root, selected_path)
        if report["manifest_sha256"] != manifest["training_manifest_sha256"]:
            raise common.R2CError("selected R2C training manifest lineage changed")
        prediction_indices = {record["row_index"] for record in report["predictions"]}
        if prediction_indices != set(references):
            raise common.R2CError("R2C prediction/reference query identities differ")
        training[candidate_id] = {"path": selected_path, "report": report}
    return {"r2b_bytes": r2b_bytes, "r2b_report": r2b_report, "references": references, "training": training}


def read_pcm16(path: Path) -> np.ndarray:
    with wave.open(str(path), "rb") as source:
        if (
            source.getnchannels() != 1
            or source.getsampwidth() != 2
            or source.getframerate() != common.r2b.SAMPLE_RATE_HZ
            or source.getnframes() != common.r2b.SAMPLE_COUNT
            or source.getcomptype() != "NONE"
        ):
            raise common.R2CError(f"R2C evaluation WAV grid changed: {path}")
        payload = source.readframes(common.r2b.SAMPLE_COUNT)
    return np.frombuffer(payload, dtype="<i2").astype(np.float64) / 32768.0


def waveform_nrmse_db(candidate: Path, reference: Path) -> float:
    prediction = read_pcm16(candidate)
    target = read_pcm16(reference)
    target_energy = float(np.sum(target * target))
    error_energy = float(np.sum((prediction - target) ** 2))
    if target_energy <= 0.0:
        raise common.R2CError("R2C query reference is silent")
    ratio = math.sqrt(error_energy / target_energy)
    return -240.0 if ratio <= 1.0e-12 else max(-240.0, 20.0 * math.log10(ratio))


def evaluate_candidate(
    root: Path,
    candidate_id: str,
    training: dict[str, Any],
    references: dict[int, Path],
    staging: Path,
) -> dict[str, Any]:
    report_path = training["path"]
    report = training["report"]
    paths = {}
    entries = []
    for prediction in report["predictions"]:
        row_index = prediction["row_index"]
        candidate_path = common.external_file(
            root, report_path.parent / prediction["prediction_file"], "R2C prediction"
        )
        reference = references[row_index]
        entry_id = f"row-{row_index:04}"
        entries.append(
            {
                "id": entry_id,
                "object_id": "realimpact-green-goblet",
                "material": "glass-vessel-published-label",
                "impact_position": "fixed-mesh-vertex-31676",
                "force_band": "force-deconvolved-transfer",
                "expected_signal": "impact",
                "candidate": {
                    "path": str(candidate_path),
                    "sha256": prediction["prediction_sha256"],
                },
                "reference": {
                    "path": str(reference),
                    "sha256": common.sha256_file(reference),
                },
            }
        )
        paths[entry_id] = (candidate_path, reference)
    rust_manifest = {
        "schema": RUST_MANIFEST_SCHEMA,
        "split": f"r2c-{candidate_id}",
        "relations": [],
        "entries": entries,
    }
    rust_bytes = common.canonical_json(rust_manifest)
    rust_path = staging / f"{candidate_id}.rust-manifest.json"
    rust_path.write_bytes(rust_bytes)
    rust_output = staging / f"{candidate_id}.rust-eval"
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "physical-sound-eval",
            "--manifest",
            str(rust_path),
            "--output",
            str(rust_output),
        ],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise common.R2CError(
            f"Rust evaluation failed for {candidate_id}: {completed.stdout}\n{completed.stderr}"
        )
    rust_report_bytes, rust_report = common.read_json(
        rust_output / "report.json", "R2C Rust evaluation report"
    )
    if rust_report.get("schema") != RUST_REPORT_SCHEMA:
        raise common.R2CError("R2C Rust evaluator schema changed")
    rows = []
    for entry in rust_report.get("entries", []):
        matched = entry.get("matched")
        if matched is None or entry.get("id") not in paths:
            raise common.R2CError("R2C Rust evaluator omitted a query match")
        prediction, reference = paths[entry["id"]]
        rows.append(
            {
                "row_index": int(entry["id"].rsplit("-", 1)[1]),
                "absolute_rms_level_error_db": abs(matched["raw_rms_delta_db"]),
                "gain_matched_multiresolution_log_spectrum_rmse_db": matched[
                    "gain_matched_multiresolution_log_spectrum_rmse_db"
                ],
                "normalized_waveform_rmse_db": waveform_nrmse_db(
                    prediction, reference
                ),
            }
        )
    rows.sort(key=lambda row: row["row_index"])
    if len(rows) != common.r2b.QUERY_ROWS:
        raise common.R2CError("R2C Rust evaluator query count changed")
    normalized_rows = common.canonical_json(rows)
    return {
        "candidate_id": candidate_id,
        "training_report_sha256": common.sha256_file(report_path),
        "rust_manifest_sha256": common.sha256_bytes(rust_bytes),
        "rust_report_sha256": common.sha256_bytes(rust_report_bytes),
        "normalized_metric_rows_sha256": common.sha256_bytes(normalized_rows),
        "query_count": len(rows),
        "aggregate": common.aggregate_metric_rows(rows),
        "rows": rows,
    }


def compare_controls(candidate: dict[str, Any], controls: list[dict[str, Any]]) -> dict[str, Any]:
    endpoints = []
    passes_all = True
    for endpoint in common.PRIMARY_ENDPOINTS:
        control_values = {
            control["control_id"]: control["aggregate"][endpoint]
            for control in controls
        }
        value = candidate["aggregate"][endpoint]
        passed = all(value < control for control in control_values.values())
        passes_all &= passed
        endpoints.append(
            {
                "endpoint": endpoint,
                "direction": "lower_is_better",
                "candidate": value,
                "controls": control_values,
                "strictly_below_all_controls": passed,
            }
        )
    return {
        **candidate,
        "primary_endpoint_comparison": endpoints,
        "passes_frozen_r2c_rule": passes_all,
    }


def select_candidate(candidates: list[dict[str, Any]]) -> tuple[str | None, str]:
    passing = [candidate for candidate in candidates if candidate["passes_frozen_r2c_rule"]]
    if not passing:
        return None, "RejectComplexListenerField"
    if len(passing) == 1:
        return passing[0]["candidate_id"], "GoComplexListenerField"
    by_id = {candidate["candidate_id"]: candidate for candidate in passing}
    data = by_id["dense_complex_field_data_only_v1"]
    physics = by_id["dense_complex_field_helmholtz_v1"]
    physics_dominates = all(
        physics["aggregate"][endpoint] < data["aggregate"][endpoint]
        for endpoint in common.PRIMARY_ENDPOINTS
    )
    selected = physics["candidate_id"] if physics_dominates else data["candidate_id"]
    return selected, "GoComplexListenerField"


def run_evaluation(
    root: Path, manifest_argument: Path | None, output_argument: Path
) -> None:
    if manifest_argument is None:
        raise common.R2CError("evaluate requires --manifest")
    manifest_path = common.external_file(root, manifest_argument, "R2C evaluation manifest")
    manifest_bytes, manifest = common.read_json(manifest_path, "R2C evaluation manifest")
    inputs = validate_evaluation_manifest(root, manifest_path, manifest)
    output, staging = common.prepare_output(root, output_argument, "R2C evaluation output")
    try:
        evaluated = [
            evaluate_candidate(
                root,
                candidate["candidate_id"],
                inputs["training"][candidate["candidate_id"]],
                inputs["references"],
                staging,
            )
            for candidate in common.CANDIDATES
        ]
        controls = inputs["r2b_report"]["controls"]
        compared = [compare_controls(candidate, controls) for candidate in evaluated]
        selected, decision = select_candidate(compared)
        report = {
            "schema": common.EVALUATION_REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "claim": (
                "GROUPED_FIXED_IMPACT_DENSE_COMPLEX_FIELD_DEVELOPMENT_DECISION_ONLY / "
                "NO_METHOD_HOLDOUT_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "revision": common.REVISION,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "runner_sha256": common.sha256_file(Path(__file__).resolve(strict=True)),
            "r2b_preflight_report_sha256": common.sha256_bytes(inputs["r2b_bytes"]),
            "primary_endpoints": list(common.PRIMARY_ENDPOINTS),
            "selection_policy": SELECTION_POLICY,
            "selected_candidate_id": selected,
            "controls": controls,
            "candidates": compared,
            "query_audio_rows_read_after_candidate_freeze": common.r2b.QUERY_ROWS,
            "query_audio_bytes_read_after_candidate_freeze": common.r2b.QUERY_ROWS
            * common.r2b.SAMPLE_COUNT
            * 2,
            "method_holdout_or_shadow_bytes_read": 0,
            "quality_admission_or_runtime_authorized": False,
        }
        report_bytes = common.canonical_json(report)
        (staging / "evaluation-report.json").write_bytes(report_bytes)
        common.publish_staging(staging, output)
    except BaseException:
        common.discard_staging(staging)
        raise
    common.emit_summary(
        {
            "output": str(output),
            "decision": decision,
            "selected_candidate_id": selected,
            "evaluation_report_sha256": common.sha256_bytes(report_bytes),
            "candidate_aggregates": {
                candidate["candidate_id"]: candidate["aggregate"]
                for candidate in compared
            },
        },
        (
            "output",
            "decision",
            "selected_candidate_id",
            "evaluation_report_sha256",
            "candidate_aggregates",
        ),
    )


def main() -> None:
    arguments = parse_arguments()
    root = common.root_from_script(Path(__file__))
    if arguments.stage == "freeze":
        if arguments.manifest is not None:
            raise common.R2CError("evaluation freeze accepts no --manifest")
        freeze_evaluation(
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
            raise common.R2CError("evaluate uses only the frozen --manifest")
        run_evaluation(root, arguments.manifest, arguments.output)


if __name__ == "__main__":
    main()
