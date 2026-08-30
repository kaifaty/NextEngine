#!/usr/bin/env python3
"""Freeze corpus, model and loss identity for R3A V5 without development reads."""

from __future__ import annotations

import argparse
import math
import shutil
from pathlib import Path
from typing import Any

import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as model


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-manifest", required=True, type=Path)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--v4-manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def capacity_descriptors() -> list[dict[str, Any]]:
    frame_count = common.ANALYSIS_SAMPLES // common.MODEL_CONFIG["hop_samples"]
    bits_per_code = math.ceil(math.log2(common.MODEL_CONFIG["codebook_size"]))
    return [
        {
            **capacity,
            "analysis_code_shape": [
                1,
                capacity["quantizers"],
                frame_count,
            ],
            "analysis_encoded_bytes": (
                capacity["quantizers"] * frame_count * bits_per_code + 7
            )
            // 8
            + 4,
            "sidecar": "one_float32_output_gain",
        }
        for capacity in common.CAPACITIES
    ]


def build_manifest(
    implementation: dict[str, str],
    environment: dict[str, Any],
    corpus: dict[str, Any],
    development_commitment: dict[str, Any],
    controls: dict[str, Any],
) -> dict[str, Any]:
    return {
        "schema": common.MANIFEST_SCHEMA,
        "status": "FrozenPreflightA",
        "study_id": common.STUDY_ID,
        "revision": common.REVISION,
        "source_manifest_sha256": common.SOURCE_MANIFEST_SHA256,
        "corpus": corpus,
        "development_commitment": development_commitment,
        "preprocessing": {
            "source": "pcm16_stereo_44100",
            "downmix": "float64_arithmetic_mean",
            "resample": "scipy_resample_poly_up160_down147_default_fir",
            "target_sample_rate_hz": common.TARGET_SAMPLE_RATE_HZ,
            "alignment": "maximum_absolute_sample_to_index_512",
            "training_segment_samples": common.TRAINING_SEGMENT_SAMPLES,
            "evaluation_samples": common.ANALYSIS_SAMPLES,
        },
        "model": common.MODEL_CONFIG,
        "capacities": capacity_descriptors(),
        "loss": common.LOSS_CONFIG,
        "training": common.TRAINING_CONFIG,
        "tracking": common.TRACKING_CONTRACT,
        "control_environment": environment,
        "training_environment": {
            "status": "MUST_FREEZE_BEFORE_REAL_TRAINING",
            "gpu_allowed": True,
            "remote_tracking_server_authorized": False,
        },
        "controls": controls,
        "implementation_sha256": implementation,
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
        "neural_training_authorized": False,
        "next_authorized_step": "FREEZE_GPU_TRAINING_ENVIRONMENT_AND_RUNNER",
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def run(root: Path, arguments: argparse.Namespace) -> Path:
    directory = Path(__file__).resolve().parent
    source_manifest = common.require_external_file(
        root, arguments.source_manifest, "Heller source manifest"
    )
    archive = common.require_external_file(root, arguments.archive, "Heller archive")
    v4_manifest = common.require_external_file(
        root, arguments.v4_manifest, "V4 manifest"
    )
    common.validate_source_manifest(source_manifest)
    corpus = common.inspect_archive(archive)
    development_commitment = common.validate_v4_manifest(v4_manifest)
    environment = common.control_environment()
    controls = {
        "full_model": model.full_model_control(),
        "micro_overfit": model.micro_overfit_control(),
    }
    implementation = common.implementation_hashes(directory)
    manifest = build_manifest(
        implementation, environment, corpus, development_commitment, controls
    )
    output, staging = common.prepare_output(root, arguments.output)
    try:
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        role_counts = {
            role: sum(clip["role"] == role for clip in corpus["clips"])
            for role in ("train", "internal_validation")
        }
        report = {
            "schema": common.REPORT_SCHEMA,
            "status": "Validated",
            "decision": "READY_FOR_TRAINING_ENVIRONMENT_FREEZE",
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "source_archive_bytes_hashed": common.ARCHIVE_BYTES,
            "source_waveform_samples_decoded": 0,
            "corpus_clip_count": len(corpus["clips"]),
            "event_group_count": len(corpus["groups"]),
            "role_clip_counts": role_counts,
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "neural_training_steps": 0,
            "control_training_steps": controls["micro_overfit"]["result"]["steps"] * 2,
            "mlflow_dependency_installed": False,
            "mlflow_policy": "optional_external_file_store_mirror_after_environment_freeze",
            "runtime_or_public_contract_changed": False,
            "authored_clip_fallback_required": True,
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
    _, report = common.load_json(output / "report.json", "V5 report", canonical=True)
    print(f"R3A V5 neural preflight A: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
