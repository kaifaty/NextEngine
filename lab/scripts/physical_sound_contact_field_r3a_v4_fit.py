#!/usr/bin/env python3
"""Fit only the frozen R3A V4 object modes and residual dictionaries."""

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


def load_fit_contacts(directory: Path) -> np.ndarray:
    value = np.load(directory / "contacts.npy", mmap_mode="r", allow_pickle=False)
    fit = np.asarray(value[: common.FIT_CONTACTS_PER_OBJECT], dtype=np.float64)
    if fit.shape[0] != common.FIT_CONTACTS_PER_OBJECT:
        raise common.V4Error("fit-only loader returned an invalid contact count")
    return fit


def training_residuals(
    fit_by_object: dict[str, np.ndarray],
    poles_by_object: dict[str, np.ndarray],
) -> dict[str, list[np.ndarray]]:
    residuals = {}
    for object_id, fit_values in fit_by_object.items():
        poles = poles_by_object[object_id]
        object_residuals = []
        for value in fit_values:
            gains = common.modal_gains(value, poles)
            object_residuals.append(value - common.synthesize_modal(poles, gains))
        residuals[object_id] = object_residuals
    return residuals


def fit_capacity(
    staging: Path,
    capacity: dict[str, Any],
    fit_by_object: dict[str, np.ndarray],
    maximum_poles: dict[str, np.ndarray],
) -> tuple[dict[str, Any], dict[str, Any]]:
    capacity_id = capacity["id"]
    poles = {
        object_id: common.selected_poles(value, capacity["mode_count"])
        for object_id, value in maximum_poles.items()
    }
    residuals_by_object = training_residuals(fit_by_object, poles)
    long_frames = common.normalized_training_frames(
        [value for rows in residuals_by_object.values() for value in rows],
        common.LONG_FRAME_SAMPLES,
        common.LONG_FRAME_HOP,
    )
    long_basis = common.randomized_basis(
        long_frames,
        capacity["long_residual_rank"],
        common.RANDOMIZED_SVD_SEED + capacity["long_residual_rank"],
    )
    spectral_bins = {}
    remaining = []
    for object_id, residuals in residuals_by_object.items():
        post_long = []
        for residual in residuals:
            long_value, _ = common.project_basis(
                residual,
                long_basis,
                common.LONG_FRAME_SAMPLES,
                common.LONG_FRAME_HOP,
            )
            post_long.append(residual - long_value)
        spectral_bins[object_id] = common.select_spectral_bins(
            post_long, capacity["spectral_residual_bins"]
        )
        for residual in post_long:
            spectral_value, _ = common.project_spectral_bins(
                residual, spectral_bins[object_id]
            )
            remaining.append((residual - spectral_value)[: common.TRANSIENT_SAMPLES])
    transient_frames = common.normalized_training_frames(
        remaining, common.TRANSIENT_FRAME_SAMPLES, common.TRANSIENT_FRAME_HOP
    )
    transient_basis = common.randomized_basis(
        transient_frames,
        capacity["transient_residual_rank"],
        common.RANDOMIZED_SVD_SEED + capacity["transient_residual_rank"],
    )

    artifacts = {
        "long_basis": common.save_npy(
            staging / f"{capacity_id}-long-basis.npy", long_basis
        ),
        "transient_basis": common.save_npy(
            staging / f"{capacity_id}-transient-basis.npy", transient_basis
        ),
        "object_poles": {
            object_id: common.save_npy(
                staging / f"{capacity_id}-{object_id}-poles.npy", value
            )
            for object_id, value in poles.items()
        },
        "object_spectral_bins": {
            object_id: common.save_npy(
                staging / f"{capacity_id}-{object_id}-spectral-bins.npy", value
            )
            for object_id, value in spectral_bins.items()
        },
    }
    shared_float_count = long_basis.size + transient_basis.size
    shared_float_count += sum(value.shape[0] * 2 for value in poles.values())
    shared_bytes = 4 * shared_float_count
    shared_bytes += sum(value.size * 2 for value in spectral_bins.values())
    fit_results = []
    maximum_record_bytes = 0
    all_fit_absolute = True
    for object_id, fit_values in fit_by_object.items():
        for contact_index, value in enumerate(fit_values):
            candidate, record = common.cook_contact(
                value,
                poles[object_id],
                long_basis,
                spectral_bins[object_id],
                transient_basis,
            )
            result_metrics = common.metrics(value, candidate)
            within = {
                endpoint: result_metrics[endpoint]
                <= common.ABSOLUTE_THRESHOLDS[endpoint]
                for endpoint in common.PRIMARY_ENDPOINTS
            }
            maximum_record_bytes = max(maximum_record_bytes, record["encoded_bytes"])
            all_fit_absolute &= all(within.values())
            fit_results.append(
                {
                    "object_id": object_id,
                    "contact_index": contact_index,
                    "metrics": result_metrics,
                    "within_absolute_threshold": within,
                    "record_bytes": record["encoded_bytes"],
                }
            )
    budget_gate = {
        "shared_decoder": shared_bytes <= common.MAXIMUM_SHARED_DECODER_BYTES,
        "contact_record": maximum_record_bytes <= common.MAXIMUM_CONTACT_RECORD_BYTES,
    }
    fit_gate = all_fit_absolute and all(budget_gate.values())
    descriptor = {
        "capacity": capacity,
        "artifacts": artifacts,
        "shared_decoder_float32_count": shared_float_count,
        "shared_decoder_bytes": shared_bytes,
        "maximum_contact_record_bytes": maximum_record_bytes,
        "fit_gate_passed": fit_gate,
    }
    report = {
        "capacity_id": capacity_id,
        "fit_gate_passed": fit_gate,
        "all_fit_contacts_within_absolute_thresholds": all_fit_absolute,
        "budget_gate": budget_gate,
        "shared_decoder_bytes": shared_bytes,
        "maximum_contact_record_bytes": maximum_record_bytes,
        "fit_contacts": fit_results,
    }
    return descriptor, report


def run(root: Path, arguments: argparse.Namespace) -> Path:
    manifest_path = common.require_external_file(
        root, arguments.manifest, "V4 manifest"
    )
    manifest_bytes, manifest = common.load_json(manifest_path, "V4 manifest")
    common.validate_manifest(manifest)
    common.validate_implementation(manifest, Path(__file__).resolve().parent)
    if common.environment_identity() != manifest["environment"]:
        raise common.V4Error("V4 environment changed after preflight")
    provided = object_arguments(arguments)
    validated = {
        item["id"]: common.validate_parent_directory(root, provided[item["id"]], item)
        for item in common.PARENT_OBJECTS
    }

    fit_by_object: dict[str, np.ndarray] = {}
    scale_by_object = {}
    for object_id, item in validated.items():
        normalized, scale = common.normalize_fit_group(
            load_fit_contacts(Path(item["directory"]))
        )
        fit_by_object[object_id] = normalized[: common.FIT_CONTACTS_PER_OBJECT]
        scale_by_object[object_id] = scale
    maximum_modes = max(item["mode_count"] for item in common.CAPACITIES)
    maximum_poles = {
        object_id: common.estimate_object_poles(values, maximum_modes)
        for object_id, values in fit_by_object.items()
    }

    output, staging = common.prepare_output(root, arguments.output)
    try:
        descriptors = []
        capacity_reports = []
        for capacity in common.CAPACITIES:
            descriptor, report = fit_capacity(
                staging, capacity, fit_by_object, maximum_poles
            )
            descriptor["fit_only_scale_by_object"] = scale_by_object
            descriptors.append(descriptor)
            capacity_reports.append(report)
        eligible = [
            item["capacity"]["id"] for item in descriptors if item["fit_gate_passed"]
        ]
        decision = (
            "READY_FOR_DEVELOPMENT_EVALUATION"
            if eligible
            else "REJECT_FIT_REPRESENTATION"
        )
        report = {
            "schema": common.FIT_REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "capacities_eligible_for_development": eligible,
            "capacity_results": capacity_reports,
            "fit_waveform_samples_decoded": (
                len(common.PARENT_OBJECTS)
                * common.FIT_CONTACTS_PER_OBJECT
                * common.ANALYSIS_SAMPLES
            ),
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
        }
        report_bytes = common.canonical_json(report)
        (staging / "fit-report.json").write_bytes(report_bytes)
        model = {
            "schema": common.MODEL_SCHEMA,
            "status": "Frozen",
            "decision": decision,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "fit_report_sha256": common.sha256_bytes(report_bytes),
            "capacities": descriptors,
            "fit_only_inputs": [
                {
                    "id": item["id"],
                    "contacts_sha256": item["contacts_sha256"],
                    "fit_contact_indices": [0, 1, 2],
                    "fit_only_scale": scale_by_object[item["id"]],
                }
                for item in validated.values()
            ],
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
        }
        (staging / "model.json").write_bytes(common.canonical_json(model))
        common.publish_output(output, staging)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    output = run(common.repository_root(), arguments)
    _, report = common.load_json(output / "fit-report.json", "V4 fit report")
    print(f"R3A V4 fit: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {common.sha256_file(output / 'fit-report.json')}")
    print(f"model sha256: {common.sha256_file(output / 'model.json')}")


if __name__ == "__main__":
    main()
