#!/usr/bin/env python3
"""Compare three bounded R3A V5-C capacity runs without development reads."""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Any

import physical_sound_contact_field_r3a_v5_capacity_train as capacity_train
import physical_sound_contact_field_r3a_v5_common as common

SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity-frontier.v1"
REVISION = "bounded-three-capacity-frontier-v1"
FRONTIER_STEP = capacity_train.CURRICULUM["quantizer_ramp_end_step"]


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--run",
        action="append",
        default=[],
        metavar="CAPACITY_ID=EXTERNAL_DIRECTORY",
    )
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def parse_runs(values: list[str]) -> dict[str, Path]:
    result = {}
    expected = {item["id"] for item in common.CAPACITIES}
    for value in values:
        capacity_id, separator, path = value.partition("=")
        if not separator or capacity_id not in expected or capacity_id in result:
            raise common.V5Error("invalid or duplicate V5 frontier run")
        result[capacity_id] = Path(path)
    if set(result) != expected:
        raise common.V5Error("V5 frontier requires exactly three capacities")
    return result


def _load_metrics(path: Path) -> list[dict[str, Any]]:
    records = []
    for line in path.read_text(encoding="utf-8").splitlines():
        value = json.loads(line)
        if not isinstance(value, dict):
            raise common.V5Error("V5 frontier metric record changed")
        records.append(value)
    if not records:
        raise common.V5Error("V5 frontier metrics are empty")
    return records


def _shared_identity(manifest: dict[str, Any]) -> dict[str, Any]:
    return {
        key: manifest[key]
        for key in (
            "schema",
            "study_id",
            "revision",
            "v5_preflight_manifest_sha256",
            "training_preflight_manifest_sha256",
            "v4_manifest_sha256",
            "archive_sha256",
            "implementation_sha256",
            "environment",
            "model",
            "loss",
            "loss_implementation_revision",
            "training",
            "execution",
            "codebook_initialization_revision",
            "sampling",
            "train_items",
            "validation_items",
            "fit_lineage",
        )
    }


def load_run(root: Path, capacity_id: str, directory: Path) -> dict[str, Any]:
    resolved = directory.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise common.V5Error("V5 frontier run must be an external directory")
    manifest_path = resolved / "run-manifest.json"
    metrics_path = resolved / "metrics.jsonl"
    report_path = resolved / "report.json"
    if not all(path.is_file() for path in (manifest_path, metrics_path, report_path)):
        raise common.V5Error("V5 frontier run artifacts are incomplete")
    manifest_payload, manifest = common.load_json(
        manifest_path, "V5 frontier run manifest", canonical=True
    )
    report_payload, report = common.load_json(
        report_path, "V5 frontier run report", canonical=True
    )
    manifest_sha256 = common.sha256_bytes(manifest_payload)
    metrics_sha256 = common.sha256_file(metrics_path)
    report_sha256 = common.sha256_bytes(report_payload)
    expected_capacity = next(
        item for item in common.CAPACITIES if item["id"] == capacity_id
    )
    gate = report.get("anti_collapse_gate", {})
    if (
        manifest.get("revision") != capacity_train.REVISION
        or manifest.get("capacity") != expected_capacity
        or manifest.get("execution", {}).get("requested_stop_after_step")
        != FRONTIER_STEP
        or manifest.get("execution", {}).get("long_capacity_training_authorized")
        is not False
        or report.get("status") != "ResearchComplete"
        or report.get("decision") != "RESEARCH_DISCRIMINATOR_PASSED"
        or report.get("completed_steps") != FRONTIER_STEP
        or report.get("capacity_training_complete") is not False
        or report.get("run_manifest_sha256") != manifest_sha256
        or report.get("metrics_sha256") != metrics_sha256
        or gate.get("passed") is not True
        or gate.get("admissible_for_checkpoint_selection") is not True
        or report.get("development_waveform_samples_decoded") != 0
        or report.get("sealed_waveform_samples_decoded") != 0
        or report.get("development_access_authorized") is not False
        or report.get("holdout_access_authorized") is not False
    ):
        raise common.V5Error(f"V5 frontier run is not admissible: {capacity_id}")
    best = report.get("best_checkpoint", {})
    checkpoint_path = resolved / best.get("path", "")
    if (
        best.get("step") != FRONTIER_STEP
        or not checkpoint_path.is_file()
        or common.sha256_file(checkpoint_path) != best.get("sha256")
        or checkpoint_path.stat().st_size != best.get("bytes")
    ):
        raise common.V5Error(f"V5 frontier checkpoint changed: {capacity_id}")
    records = _load_metrics(metrics_path)
    matches = [record for record in records if record.get("step") == best["step"]]
    if len(matches) != 1 or matches[0].get("anti_collapse_gate", {}).get("passed") is not True:
        raise common.V5Error(f"V5 frontier best metric is invalid: {capacity_id}")
    validation = matches[0]["validation"]
    return {
        "capacity_id": capacity_id,
        "quantizers": expected_capacity["quantizers"],
        "nominal_bits_per_second": expected_capacity["nominal_bits_per_second"],
        "run_manifest_sha256": manifest_sha256,
        "metrics_sha256": metrics_sha256,
        "report_sha256": report_sha256,
        "checkpoint": best,
        "validation_total": validation["mean"]["total"],
        "validation_log_spectrum": validation["mean"]["log_spectrum"],
        "validation_waveform_l1": validation["mean"]["waveform_l1"],
        "mean_output_target_rms_ratio": validation["waveform_diagnostics"][
            "mean_output_target_rms_ratio"
        ],
        "mean_absolute_correlation": validation["waveform_diagnostics"][
            "mean_absolute_correlation"
        ],
        "output_diversity_ratio": validation["waveform_diagnostics"][
            "output_diversity_ratio"
        ],
        "unique_codes_per_quantizer": validation["unique_codes_per_quantizer"],
        "maximum_encoder_latent_rms": validation["latent_diagnostics"][
            "maximum_rms"
        ],
        "shared_identity": _shared_identity(manifest),
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
    }


def rank_capacities(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(
        records,
        key=lambda item: (
            item["validation_total"],
            item["nominal_bits_per_second"],
            item["capacity_id"],
        ),
    )


def run(root: Path, arguments: argparse.Namespace) -> Path:
    output = arguments.output.resolve()
    if output.is_relative_to(root):
        raise common.V5Error("V5 frontier output must stay outside the repository")
    directories = parse_runs(arguments.run)
    records = [
        load_run(root, capacity["id"], directories[capacity["id"]])
        for capacity in common.CAPACITIES
    ]
    shared_hashes = {
        common.sha256_bytes(common.canonical_json(record.pop("shared_identity")))
        for record in records
    }
    if len(shared_hashes) != 1:
        raise common.V5Error("V5 frontier run identities are not comparable")
    ranking = rank_capacities(records)
    result = {
        "schema": SCHEMA,
        "status": "BoundedFrontierComplete",
        "decision": "BOUNDED_CAPACITY_FRONTIER_COMPLETE",
        "study_id": common.STUDY_ID,
        "revision": REVISION,
        "frontier_step": FRONTIER_STEP,
        "shared_identity_sha256": next(iter(shared_hashes)),
        "ranking": ranking,
        "selected_capacity_id": ranking[0]["capacity_id"],
        "selection_metric": "minimum_full_internal_validation_total_then_rate_then_id",
        "long_capacity_training_authorized": False,
        "development_access_authorized": False,
        "holdout_access_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
        "next_authorized_step": "REVIEW_FRONTIER_AND_FREEZE_ONE_BOUNDED_FOLLOWUP",
    }
    output, staging = common.prepare_output(root, output)
    try:
        (staging / "report.json").write_bytes(common.canonical_json(result))
        common.publish_output(output, staging)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    output = run(common.repository_root(), arguments)
    _, report = common.load_json(output / "report.json", "V5 frontier", True)
    print(f"R3A V5 capacity frontier: {output}")
    print(f"decision: {report['decision']}")
    print(f"selected capacity: {report['selected_capacity_id']}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
