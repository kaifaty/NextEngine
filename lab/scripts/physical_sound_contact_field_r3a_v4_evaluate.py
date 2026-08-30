#!/usr/bin/env python3
"""Evaluate the frozen R3A V4 frontier on already-opened development contacts."""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v4_common as common


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--model", required=True, type=Path)
    parser.add_argument("--blue-bowl", required=True, type=Path)
    parser.add_argument("--large-swan", required=True, type=Path)
    parser.add_argument("--plastic-bin", required=True, type=Path)
    parser.add_argument("--purple-scoop", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def object_arguments(arguments: argparse.Namespace) -> dict[str, Path]:
    return {
        "blue_bowl": arguments.blue_bowl,
        "large_swan": arguments.large_swan,
        "plastic_bin": arguments.plastic_bin,
        "purple_scoop": arguments.purple_scoop,
    }


def load_positions(directory: Path) -> np.ndarray:
    _, metadata = common.load_json(directory / "contacts-metadata.json", "metadata")
    return np.asarray(
        [item["position_metres"] for item in metadata["contacts"]],
        dtype=np.float64,
    )


def load_analysis_contacts(directory: Path, scale: float) -> np.ndarray:
    mapped = np.load(directory / "contacts.npy", mmap_mode="r", allow_pickle=False)
    contacts = np.asarray(mapped[: common.DEVELOPMENT_CONTACT_INDEX + 1])
    aligned = np.stack([common.align_peak(row) for row in contacts])
    return np.ascontiguousarray(aligned * scale, dtype=np.float64)


def capacity_by_id(model: dict[str, Any], capacity_id: str) -> dict[str, Any]:
    matches = [
        item for item in model["capacities"] if item["capacity"]["id"] == capacity_id
    ]
    if len(matches) != 1:
        raise common.V4Error(f"model capacity is missing or duplicated: {capacity_id}")
    return matches[0]


def validate_model(
    model_path: Path,
    model: dict[str, Any],
    manifest_bytes: bytes,
) -> tuple[Path, dict[str, Any]]:
    directory = model_path.parent
    if (
        model.get("schema") != common.MODEL_SCHEMA
        or model.get("manifest_sha256") != common.sha256_bytes(manifest_bytes)
        or model.get("development_waveform_samples_decoded") != 0
        or model.get("sealed_waveform_samples_decoded") != 0
    ):
        raise common.V4Error("V4 fitted model boundary changed")
    fit_report_bytes, fit_report = common.load_json(
        directory / "fit-report.json", "V4 fit report"
    )
    if (
        common.sha256_bytes(fit_report_bytes) != model.get("fit_report_sha256")
        or fit_report.get("schema") != common.FIT_REPORT_SCHEMA
        or fit_report.get("manifest_sha256") != common.sha256_bytes(manifest_bytes)
        or fit_report.get("decision") != model.get("decision")
        or fit_report.get("development_waveform_samples_decoded") != 0
        or fit_report.get("sealed_waveform_samples_decoded") != 0
    ):
        raise common.V4Error("V4 fit report lineage changed")
    if model.get("decision") != "READY_FOR_DEVELOPMENT_EVALUATION":
        raise common.V4Error("V4 fit stage did not authorize development evaluation")
    return directory, fit_report


def run(root: Path, arguments: argparse.Namespace) -> Path:
    manifest_path = common.require_external_file(
        root, arguments.manifest, "V4 manifest"
    )
    manifest_bytes, manifest = common.load_json(manifest_path, "V4 manifest")
    common.validate_manifest(manifest)
    common.validate_implementation(manifest, Path(__file__).resolve().parent)
    if common.environment_identity() != manifest["environment"]:
        raise common.V4Error("V4 environment changed after preflight")
    model_path = common.require_external_file(root, arguments.model, "V4 model")
    model_bytes, model = common.load_json(model_path, "V4 model")
    model_directory, fit_report = validate_model(model_path, model, manifest_bytes)
    provided = object_arguments(arguments)
    validated = {
        item["id"]: common.validate_parent_directory(root, provided[item["id"]], item)
        for item in common.PARENT_OBJECTS
    }

    scales = {item["id"]: item["fit_only_scale"] for item in model["fit_only_inputs"]}
    contacts = {
        object_id: load_analysis_contacts(Path(item["directory"]), scales[object_id])
        for object_id, item in validated.items()
    }
    positions = {
        object_id: load_positions(Path(item["directory"]))
        for object_id, item in validated.items()
    }
    identity_results = {
        object_id: common.identity_gate(common.metrics(values[3], values[3].copy()))
        for object_id, values in contacts.items()
    }
    identity_passed = all(item["passed"] for item in identity_results.values())

    capacity_results = []
    audition_signals: dict[str, np.ndarray] = {}
    for capacity in common.CAPACITIES:
        descriptor = capacity_by_id(model, capacity["id"])
        long_basis = common.load_bound_npy(
            model_directory, descriptor["artifacts"]["long_basis"]
        )
        transient_basis = common.load_bound_npy(
            model_directory, descriptor["artifacts"]["transient_basis"]
        )
        object_results = []
        for object_id, values in contacts.items():
            poles = common.load_bound_npy(
                model_directory,
                descriptor["artifacts"]["object_poles"][object_id],
            )
            spectral_bins = common.load_bound_npy(
                model_directory,
                descriptor["artifacts"]["object_spectral_bins"][object_id],
            ).astype(np.int64)
            target = values[common.DEVELOPMENT_CONTACT_INDEX]
            distances = np.linalg.norm(
                positions[object_id][: common.FIT_CONTACTS_PER_OBJECT]
                - positions[object_id][common.DEVELOPMENT_CONTACT_INDEX],
                axis=1,
            )
            nearest = int(np.argmin(distances))
            baseline = values[nearest]
            candidate, record = common.cook_contact(
                target, poles, long_basis, spectral_bins, transient_basis
            )
            candidate_metrics = common.metrics(target, candidate)
            baseline_metrics = common.metrics(target, baseline)
            gate = common.candidate_gate(candidate_metrics, baseline_metrics)
            record_gate = record["encoded_bytes"] <= common.MAXIMUM_CONTACT_RECORD_BYTES
            object_results.append(
                {
                    "object_id": object_id,
                    "nearest_fit_contact_index": nearest,
                    "nearest_fit_distance_metres": float(distances[nearest]),
                    "baseline_metrics": baseline_metrics,
                    "candidate_metrics": candidate_metrics,
                    "candidate_gate": gate,
                    "contact_record_bytes": record["encoded_bytes"],
                    "contact_record_budget_passed": record_gate,
                    "passed": gate["passed"] and record_gate,
                }
            )
            audition_signals[f"{capacity['id']}-{object_id}-target"] = target
            audition_signals[f"{capacity['id']}-{object_id}-candidate"] = candidate
        shared_gate = (
            descriptor["shared_decoder_bytes"] <= common.MAXIMUM_SHARED_DECODER_BYTES
        )
        capacity_results.append(
            {
                "capacity_id": capacity["id"],
                "shared_decoder_bytes": descriptor["shared_decoder_bytes"],
                "shared_decoder_budget_passed": shared_gate,
                "fit_gate_passed": descriptor["fit_gate_passed"],
                "objects": object_results,
                "passed": (
                    descriptor["fit_gate_passed"]
                    and shared_gate
                    and all(item["passed"] for item in object_results)
                ),
            }
        )
    passing = [item["capacity_id"] for item in capacity_results if item["passed"]]
    selected = passing[0] if passing else None
    decision = (
        "READY_FOR_V4_HOLDOUT"
        if identity_passed and selected is not None
        else (
            "INVALID_IDENTITY_CONTROL"
            if not identity_passed
            else "REJECT_REPRESENTATION"
        )
    )

    output, staging = common.prepare_output(root, arguments.output)
    try:
        audition_hashes = {}
        for name, value in audition_signals.items():
            payload = common.pcm16_wav(value)
            filename = f"{name}.wav"
            (staging / filename).write_bytes(payload)
            audition_hashes[filename] = common.sha256_bytes(payload)
        report = {
            "schema": common.EVALUATION_REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "model_sha256": common.sha256_bytes(model_bytes),
            "fit_report_sha256": model["fit_report_sha256"],
            "identity_controls": identity_results,
            "selected_smallest_capacity": selected,
            "capacity_results": capacity_results,
            "development_waveform_samples_decoded": (
                len(common.PARENT_OBJECTS) * common.ANALYSIS_SAMPLES
            ),
            "new_object_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "audition_artifact_policy": (
                "independently_peak_normalized_pcm16_diagnostic_only"
            ),
            "audition_artifact_sha256": audition_hashes,
            "authored_clip_fallback_required": True,
            "neural_training_authorized": False,
            "runtime_or_public_contract_changed": False,
            "fit_capacity_eligibility": fit_report[
                "capacities_eligible_for_development"
            ],
        }
        (staging / "report.json").write_bytes(common.canonical_json(report))
        common.publish_output(output, staging)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    output = run(common.repository_root(), arguments)
    _, report = common.load_json(output / "report.json", "V4 evaluation report")
    print(f"R3A V4 evaluation: {output}")
    print(f"decision: {report['decision']}")
    print(f"selected capacity: {report['selected_smallest_capacity']}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
