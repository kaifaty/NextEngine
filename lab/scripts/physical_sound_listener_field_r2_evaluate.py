#!/usr/bin/env python3
"""Evaluate frozen R2 listener fields with the repository Rust audio metrics."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
import wave
from pathlib import Path
from typing import Any

import numpy as np

MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-evaluation.manifest.v1"
)
TRAINING_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-training.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-mlflow-lineage.v1"
)
BASELINE_SCHEMA = (
    "nextengine.experimental-physical-sound-transfer-field-baseline.report.v1"
)
PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v2"
)
RUST_MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-validator.manifest.v1"
RUST_REPORT_SCHEMA = "nextengine.experimental-physical-sound-validator.report.v1"
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-evaluation.report.v1"
)
REVISION = "green-goblet-fixed-impact-listener-field-r2-evaluation-v1"
TRAINING_REVISION = "green-goblet-fixed-impact-listener-field-r2-v2"
METRIC_PROFILE = "transfer-listener-field-r1-v1"
PRIMARY_ENDPOINTS = (
    "mean_absolute_rms_level_error_db",
    "p95_absolute_rms_level_error_db",
    "mean_gain_matched_multiresolution_log_spectrum_rmse_db",
    "p95_gain_matched_multiresolution_log_spectrum_rmse_db",
    "mean_normalized_waveform_rmse_db",
)
DB_FLOOR = -240.0


class EvaluationError(RuntimeError):
    """The frozen R2 evaluation boundary failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise EvaluationError(f"{label} must be an external file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise EvaluationError(f"output must be a new external directory: {resolved}")
    resolved.mkdir()
    return resolved


def read_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise EvaluationError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise EvaluationError(f"{label} must contain one JSON object")
    return data, value


def require_ref(value: Any, label: str) -> dict[str, str]:
    if (
        not isinstance(value, dict)
        or set(value) != {"path", "sha256"}
        or not isinstance(value["path"], str)
        or not isinstance(value["sha256"], str)
        or len(value["sha256"]) != 64
        or any(character not in "0123456789abcdef" for character in value["sha256"])
    ):
        raise EvaluationError(f"invalid {label} file reference")
    return value


def resolve_external_ref(
    root: Path, directory: Path, value: Any, label: str
) -> tuple[Path, bytes]:
    reference = require_ref(value, label)
    unresolved = Path(reference["path"])
    path = unresolved if unresolved.is_absolute() else directory / unresolved
    path = external_file(root, path, label)
    data = path.read_bytes()
    actual = sha256_bytes(data)
    if actual != reference["sha256"]:
        raise EvaluationError(
            f"{label} hash mismatch: expected {reference['sha256']}, got {actual}"
        )
    return path, data


def validate_local_sources(root: Path, sources: Any) -> list[dict[str, Any]]:
    if not isinstance(sources, list) or not sources:
        raise EvaluationError("metric_sources must be a non-empty array")
    records = []
    previous = None
    for source in sources:
        reference = require_ref(source, "metric source")
        if previous is not None and previous >= reference["path"]:
            raise EvaluationError("metric sources must be strictly sorted")
        previous = reference["path"]
        path = (root / reference["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file():
            raise EvaluationError("metric source must remain repository-local")
        actual = sha256_file(path)
        if actual != reference["sha256"]:
            raise EvaluationError(f"metric source changed: {reference['path']}")
        records.append({**reference, "byte_count": path.stat().st_size})
    return records


def validate_manifest(
    root: Path, path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    expected = {
        "schema",
        "revision",
        "runner_sha256",
        "training_report",
        "mlflow_lineage",
        "baseline_report",
        "projection",
        "query_bindings",
        "metric_sources",
    }
    if set(manifest) != expected:
        raise EvaluationError("listener-field evaluation manifest fields changed")
    if manifest.get("schema") != MANIFEST_SCHEMA or manifest.get("revision") != REVISION:
        raise EvaluationError("listener-field evaluation schema or revision changed")
    if manifest.get("runner_sha256") != sha256_file(Path(__file__).resolve(strict=True)):
        raise EvaluationError("listener-field evaluation runner changed")
    bindings = manifest.get("query_bindings")
    if not isinstance(bindings, list) or not bindings:
        raise EvaluationError("query bindings must be non-empty")
    ids = [binding.get("row_id") for binding in bindings]
    if ids != sorted(ids) or len(set(ids)) != len(ids):
        raise EvaluationError("query bindings must be strictly sorted and unique")
    for binding in bindings:
        if set(binding) != {"row_id", "reference_audio"}:
            raise EvaluationError("query binding fields changed")
        require_ref(binding["reference_audio"], "query reference audio")
    return {"metric_sources": validate_local_sources(root, manifest["metric_sources"])}


def validate_lineage(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    directory = manifest_path.parent
    training_path, training_bytes = resolve_external_ref(
        root, directory, manifest["training_report"], "training report"
    )
    mlflow_path, mlflow_bytes = resolve_external_ref(
        root, directory, manifest["mlflow_lineage"], "MLflow lineage"
    )
    _, baseline_bytes = resolve_external_ref(
        root, directory, manifest["baseline_report"], "baseline report"
    )
    _, projection_bytes = resolve_external_ref(
        root, directory, manifest["projection"], "fit projection"
    )
    training = json.loads(training_bytes)
    mlflow_lineage = json.loads(mlflow_bytes)
    baseline = json.loads(baseline_bytes)
    projection = json.loads(projection_bytes)
    if (
        training.get("schema") != TRAINING_SCHEMA
        or training.get("status") != "Trained"
        or training.get("revision") != TRAINING_REVISION
        or training.get("query_audio_bytes_read") != 0
        or training.get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise EvaluationError("training report does not preserve R2 boundary")
    if (
        mlflow_lineage.get("schema") != MLFLOW_SCHEMA
        or mlflow_lineage.get("deterministic_training_report_sha256")
        != sha256_bytes(training_bytes)
    ):
        raise EvaluationError("MLflow lineage does not bind training report")
    if (
        baseline.get("schema") != BASELINE_SCHEMA
        or baseline.get("status") != "Validated"
        or baseline.get("metric_profile", {}).get("id") != METRIC_PROFILE
        or baseline.get("admission_shadow_opened") is not False
    ):
        raise EvaluationError("baseline metric boundary changed")
    if projection.get("schema") != PROJECTION_SCHEMA:
        raise EvaluationError("projection schema changed")
    projected = {row["row_id"]: row for row in projection.get("rows", [])}
    expected_query = sorted(
        row_id
        for row_id, row in projected.items()
        if row.get("split_role") == "development"
        and row.get("sample_role") == "query"
        and row.get("corpus_role") == "target"
        and row.get("audio_semantics")
        == "force_deconvolved_transfer_response"
    )
    binding_ids = [binding["row_id"] for binding in manifest["query_bindings"]]
    if binding_ids != expected_query:
        raise EvaluationError("query bindings differ from projected development queries")
    for binding in manifest["query_bindings"]:
        _, data = resolve_external_ref(
            root,
            directory,
            binding["reference_audio"],
            "development query reference",
        )
        row = projected[binding["row_id"]]
        if sha256_bytes(data) != row["audio"]["sha256"] or len(data) != row["audio"][
            "byte_count"
        ]:
            raise EvaluationError(f"query reference changed: {binding['row_id']}")
    candidate_runs = {run["candidate_id"] for run in mlflow_lineage.get("runs", [])}
    candidate_reports = training.get("candidates", [])
    candidate_ids = [candidate["candidate_id"] for candidate in candidate_reports]
    if candidate_ids != [
        "listener_tanh_rank4_modal_latent_v1",
        "listener_tanh_rank7_modal_plus_residual_v1",
    ] or candidate_runs != set(candidate_ids):
        raise EvaluationError("candidate ladder differs from frozen training/MLflow lineage")
    return {
        "training_path": training_path,
        "training_bytes": training_bytes,
        "training": training,
        "mlflow_path": mlflow_path,
        "mlflow_bytes": mlflow_bytes,
        "baseline_bytes": baseline_bytes,
        "baseline": baseline,
        "projection_bytes": projection_bytes,
        "projection": projection,
    }


def read_pcm16(path: Path) -> tuple[int, np.ndarray]:
    with wave.open(str(path), "rb") as source:
        if (
            source.getnchannels() != 1
            or source.getsampwidth() != 2
            or source.getcomptype() != "NONE"
        ):
            raise EvaluationError(f"WAV must be uncompressed mono PCM16: {path}")
        rate = source.getframerate()
        frames = source.getnframes()
        payload = source.readframes(frames)
    if len(payload) != frames * 2:
        raise EvaluationError(f"truncated PCM16 payload: {path}")
    return rate, np.frombuffer(payload, dtype="<i2").astype(np.float64) / 32768.0


def normalized_waveform_rmse_db(candidate: Path, reference: Path) -> float:
    candidate_rate, candidate_samples = read_pcm16(candidate)
    reference_rate, reference_samples = read_pcm16(reference)
    if candidate_rate != reference_rate or candidate_samples.shape != reference_samples.shape:
        raise EvaluationError("candidate/reference PCM grid differs")
    reference_energy = float(np.sum(reference_samples**2))
    if reference_energy <= 0.0:
        raise EvaluationError("query reference is silent")
    error_energy = float(np.sum((candidate_samples - reference_samples) ** 2))
    ratio = math.sqrt(error_energy / reference_energy)
    return DB_FLOOR if ratio <= 1.0e-12 else max(DB_FLOOR, 20.0 * math.log10(ratio))


def prediction_path(training_path: Path, record: dict[str, Any]) -> Path:
    path = (training_path.parent / record["prediction_file"]).resolve(strict=True)
    if not path.is_file() or sha256_file(path) != record["prediction_sha256"]:
        raise EvaluationError(f"candidate prediction changed: {record['row_id']}")
    if path.stat().st_size != record["prediction_byte_count"]:
        raise EvaluationError(f"candidate prediction size changed: {record['row_id']}")
    return path


def build_rust_manifest(
    manifest: dict[str, Any],
    lineage: dict[str, Any],
    candidate: dict[str, Any],
    output: Path,
) -> tuple[Path, bytes, dict[str, tuple[Path, Path]]]:
    references = {
        binding["row_id"]: Path(binding["reference_audio"]["path"])
        for binding in manifest["query_bindings"]
    }
    predictions = {record["row_id"]: record for record in candidate["predictions"]}
    if set(references) != set(predictions):
        raise EvaluationError("candidate predictions do not cover every query")
    entries = []
    paths = {}
    for row_id in sorted(references):
        prediction = prediction_path(lineage["training_path"], predictions[row_id])
        reference = references[row_id].resolve(strict=True)
        entry_id = row_id.rsplit("-", 1)[-1]
        entries.append(
            {
                "id": entry_id,
                "object_id": "realimpact-green-goblet",
                "material": "glass-vessel-published-label",
                "impact_position": "fixed-mesh-vertex-31676",
                "force_band": "force-deconvolved-transfer",
                "expected_signal": "impact",
                "candidate": {
                    "path": str(prediction),
                    "sha256": sha256_file(prediction),
                },
                "reference": {
                    "path": str(reference),
                    "sha256": sha256_file(reference),
                },
            }
        )
        paths[entry_id] = (prediction, reference)
    value = {
        "schema": RUST_MANIFEST_SCHEMA,
        "split": f"r2-{candidate['candidate_id']}",
        "relations": [],
        "entries": entries,
    }
    data = canonical_json(value)
    path = output / f"{candidate['candidate_id']}.rust-eval-manifest.json"
    path.write_bytes(data)
    return path, data, paths


def run_rust_evaluator(root: Path, manifest: Path, output: Path) -> tuple[bytes, dict[str, Any]]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "xtask",
        "--",
        "physical-sound-eval",
        "--manifest",
        str(manifest),
        "--output",
        str(output),
    ]
    completed = subprocess.run(
        command, cwd=root, text=True, capture_output=True, check=False
    )
    if completed.returncode != 0:
        raise EvaluationError(
            f"Rust physical-sound-eval failed: {completed.stdout}\n{completed.stderr}"
        )
    report_path = output / "report.json"
    data, report = read_json(report_path, "Rust physical-sound report")
    if report.get("schema") != RUST_REPORT_SCHEMA:
        raise EvaluationError("Rust physical-sound report schema changed")
    return data, report


def nearest_rank(values: list[float], percentile: int) -> float:
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered) / 100.0))
    return ordered[rank - 1]


def aggregate(rows: list[dict[str, Any]]) -> dict[str, float]:
    level = [row["absolute_rms_level_error_db"] for row in rows]
    spectrum = [
        row["gain_matched_multiresolution_log_spectrum_rmse_db"] for row in rows
    ]
    waveform = [row["normalized_waveform_rmse_db"] for row in rows]
    return {
        "mean_absolute_rms_level_error_db": sum(level) / len(level),
        "p95_absolute_rms_level_error_db": nearest_rank(level, 95),
        "mean_gain_matched_multiresolution_log_spectrum_rmse_db": sum(spectrum)
        / len(spectrum),
        "p95_gain_matched_multiresolution_log_spectrum_rmse_db": nearest_rank(
            spectrum, 95
        ),
        "mean_normalized_waveform_rmse_db": sum(waveform) / len(waveform),
    }


def evaluate_candidate(
    root: Path,
    manifest: dict[str, Any],
    lineage: dict[str, Any],
    candidate: dict[str, Any],
    output: Path,
) -> dict[str, Any]:
    rust_manifest, rust_manifest_bytes, paths = build_rust_manifest(
        manifest, lineage, candidate, output
    )
    rust_output = output / f"{candidate['candidate_id']}.rust-eval"
    rust_bytes, rust_report = run_rust_evaluator(root, rust_manifest, rust_output)
    rows = []
    for entry in rust_report["entries"]:
        matched = entry.get("matched")
        if matched is None:
            raise EvaluationError(f"Rust evaluator omitted matched metrics: {entry['id']}")
        prediction, reference = paths[entry["id"]]
        rows.append(
            {
                "row_id": next(
                    row_id
                    for row_id in (record["row_id"] for record in candidate["predictions"])
                    if row_id.endswith(entry["id"])
                ),
                "absolute_rms_level_error_db": abs(matched["raw_rms_delta_db"]),
                "gain_matched_multiresolution_log_spectrum_rmse_db": matched[
                    "gain_matched_multiresolution_log_spectrum_rmse_db"
                ],
                "normalized_waveform_rmse_db": normalized_waveform_rmse_db(
                    prediction, reference
                ),
            }
        )
    rows.sort(key=lambda row: row["row_id"])
    return {
        "candidate_id": candidate["candidate_id"],
        "rank": candidate["rank"],
        "rust_manifest_sha256": sha256_bytes(rust_manifest_bytes),
        "rust_report_sha256": sha256_bytes(rust_bytes),
        "query_count": len(rows),
        "aggregate": aggregate(rows),
        "rows": rows,
    }


def comparison(candidate: dict[str, Any], controls: list[dict[str, Any]]) -> dict[str, Any]:
    endpoints = []
    all_pass = True
    for endpoint in PRIMARY_ENDPOINTS:
        candidate_value = candidate["aggregate"][endpoint]
        values = {control["control"]: control[endpoint] for control in controls}
        passed = all(candidate_value < value for value in values.values())
        all_pass &= passed
        endpoints.append(
            {
                "endpoint": endpoint,
                "direction": "lower_is_better",
                "candidate": candidate_value,
                "controls": values,
                "strictly_below_both": passed,
            }
        )
    return {
        **candidate,
        "primary_endpoint_comparison": endpoints,
        "passes_frozen_r2_rule": all_pass,
    }


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve(strict=True).parents[2]
    runner = Path(__file__).resolve(strict=True)
    if not runner.is_relative_to(root):
        raise EvaluationError("evaluation runner must remain repository-local")
    manifest_path = external_file(root, arguments.manifest, "evaluation manifest")
    manifest_bytes, manifest = read_json(manifest_path, "evaluation manifest")
    validation = validate_manifest(root, manifest_path, manifest)
    lineage = validate_lineage(root, manifest_path, manifest)
    output = external_output(root, arguments.output)
    evaluated = [
        evaluate_candidate(root, manifest, lineage, candidate, output)
        for candidate in lineage["training"]["candidates"]
    ]
    controls = lineage["baseline"]["controls"]
    compared = [comparison(candidate, controls) for candidate in evaluated]
    passing = [candidate for candidate in compared if candidate["passes_frozen_r2_rule"]]
    if passing:
        selected = min(passing, key=lambda candidate: candidate["rank"])["candidate_id"]
        decision = "GoListenerField"
    else:
        selected = None
        decision = "RejectListenerField"
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": decision,
        "claim": (
            "FIXED_IMPACT_DEVELOPMENT_LISTENER_FIELD_DECISION_ONLY / NO_METHOD_"
            "HOLDOUT_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": sha256_file(runner),
        "training_report_sha256": sha256_bytes(lineage["training_bytes"]),
        "mlflow_lineage_sha256": sha256_bytes(lineage["mlflow_bytes"]),
        "baseline_report_sha256": sha256_bytes(lineage["baseline_bytes"]),
        "projection_sha256": sha256_bytes(lineage["projection_bytes"]),
        "metric_profile": METRIC_PROFILE,
        "primary_endpoints": list(PRIMARY_ENDPOINTS),
        "selection_rule": (
            "smallest_rank_candidate_strictly_below_both_controls_on_every_"
            "primary_endpoint_else_reject"
        ),
        "metric_sources": validation["metric_sources"],
        "selected_candidate_id": selected,
        "candidates": compared,
        "query_audio_rows_read": len(manifest["query_bindings"]),
        "method_holdout_or_shadow_bytes_read": 0,
        "quality_admission_or_runtime_authorized": False,
    }
    report_bytes = canonical_json(report)
    (output / "evaluation-report.json").write_bytes(report_bytes)
    print(report_bytes.decode(), end="")


if __name__ == "__main__":
    main()
