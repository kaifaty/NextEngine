#!/usr/bin/env python3
"""Freeze the R3A V4 development representation experiment before evaluation."""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path
from typing import Any

import physical_sound_contact_field_r3a_v4_common as common


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
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


def public_object_descriptor(validated: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in validated.items() if key not in {"directory"}}


def build_manifest(
    implementation: dict[str, str],
    environment: dict[str, Any],
    objects: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "schema": common.MANIFEST_SCHEMA,
        "status": "Frozen",
        "study_id": common.STUDY_ID,
        "revision": common.REVISION,
        "sample_rate_hz": common.SAMPLE_RATE_HZ,
        "analysis_samples": common.ANALYSIS_SAMPLES,
        "peak_alignment_sample": common.PEAK_ALIGNMENT_SAMPLE,
        "fit_contacts_per_object": common.FIT_CONTACTS_PER_OBJECT,
        "development_contact_index": common.DEVELOPMENT_CONTACT_INDEX,
        "development_waveforms_open_before_v4": True,
        "new_object_or_sealed_waveform_access_authorized": False,
        "objects": objects,
        "capacities": list(common.CAPACITIES),
        "representation": {
            "object_global_state": "stable_frequency_and_damping_poles",
            "contact_record": (
                "modal_cos_sin_gains_plus_long_and_transient_residual_coefficients"
            ),
            "long_residual": {
                "frame_samples": common.LONG_FRAME_SAMPLES,
                "hop_samples": common.LONG_FRAME_HOP,
                "window": "sqrt_hann_overlap_add",
            },
            "spectral_residual": {
                "source": "post_long_residual_fit_frames",
                "selection": "object_global_fit_normalized_energy_rank",
                "coefficients": "contact_specific_complex_stft",
                "frame_samples": common.LONG_FRAME_SAMPLES,
                "hop_samples": common.LONG_FRAME_HOP,
            },
            "transient_residual": {
                "analysis_samples": common.TRANSIENT_SAMPLES,
                "frame_samples": common.TRANSIENT_FRAME_SAMPLES,
                "hop_samples": common.TRANSIENT_FRAME_HOP,
                "window": "sqrt_hann_overlap_add",
            },
            "basis_training": "fit_role_residual_frames_only",
            "basis_solver": {
                "id": "fixed_seed_randomized_svd",
                "seed": common.RANDOMIZED_SVD_SEED,
                "oversample": common.RANDOMIZED_SVD_OVERSAMPLE,
                "power_iterations": common.RANDOMIZED_SVD_POWER_ITERATIONS,
            },
            "inverse": "deterministic_float64_reference_then_float32_budget",
        },
        "budgets": {
            "maximum_shared_decoder_bytes": common.MAXIMUM_SHARED_DECODER_BYTES,
            "maximum_contact_record_bytes": common.MAXIMUM_CONTACT_RECORD_BYTES,
            "cooked_scalar_format": "float32",
        },
        "evaluation_protocol": {
            "baseline": "nearest_fit_contact_by_euclidean_mesh_position",
            "selection": "smallest_capacity_passing_all_four_development_contacts",
            "failure": "REJECT_REPRESENTATION_without_new_object",
            "success": "READY_FOR_V4_HOLDOUT_on_one_new_unopened_object",
        },
        "primary_endpoints": list(common.PRIMARY_ENDPOINTS),
        "absolute_thresholds": common.ABSOLUTE_THRESHOLDS,
        "minimum_beaten_endpoints": common.MINIMUM_BEATEN_ENDPOINTS,
        "maximum_normalized_error_ratio": common.MAXIMUM_NORMALIZED_ERROR_RATIO,
        "environment": environment,
        "implementation_sha256": implementation,
        "neural_training_authorized": False,
        "runtime_or_public_contract_change_authorized": False,
        "authored_clip_fallback_required": True,
    }


def run(root: Path, arguments: argparse.Namespace) -> Path:
    implementation_directory = Path(__file__).resolve().parent
    implementation = common.implementation_hashes(implementation_directory)
    environment = common.environment_identity()
    provided = object_arguments(arguments)
    validated = [
        common.validate_parent_directory(root, provided[item["id"]], item)
        for item in common.PARENT_OBJECTS
    ]
    manifest = build_manifest(
        implementation,
        environment,
        [public_object_descriptor(item) for item in validated],
    )
    common.validate_manifest(manifest)
    output, staging = common.prepare_output(root, arguments.output)
    try:
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": common.PREFLIGHT_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "READY_FOR_FIT_ONLY_REPRESENTATION_LEARNING",
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "objects": [
                {
                    "id": item["id"],
                    "contacts_sha256": item["contacts_sha256"],
                    "roles": item["roles"],
                    "sealed_row": item["sealed_row"],
                }
                for item in validated
            ],
            "fit_waveform_samples_decoded": 0,
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
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
    print(f"R3A V4 preflight: {output}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
