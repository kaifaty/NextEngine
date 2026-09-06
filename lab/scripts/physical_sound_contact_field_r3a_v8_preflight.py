#!/usr/bin/env python3
"""Run the frozen R3A V8 explicit-modal synthetic preflight."""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Any

import physical_sound_contact_field_r3a_v8_common as common
import physical_sound_contact_field_r3a_v8_model as model


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--object-1-feature", required=True, type=Path)
    parser.add_argument("--object-2-feature", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def build_manifest(
    environment: dict[str, Any],
    implementation: dict[str, str],
    sources: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "schema": common.MANIFEST_SCHEMA,
        "status": "FrozenSyntheticPreflight",
        "study_id": common.STUDY_ID,
        "revision": common.REVISION,
        "research_decision": (
            "explicit_object_global_frequency_damping_plus_neural_mode_shape_field"
        ),
        "source": {
            "repository": common.NISR_REPOSITORY,
            "revision": common.NISR_REVISION,
            "url": common.NISR_SOURCE_URL,
            "license": common.NISR_LICENSE,
            "signal_role": "synthetic_fem_labels_only",
            "files": [common.public_source_descriptor(item) for item in sources],
        },
        "modal_control": common.MODAL_CONTROL_CONFIG,
        "field": common.FIELD_CONFIG,
        "environment": environment,
        "implementation_sha256": implementation,
        "opened_synthetic_development_files": len(sources),
        "real_v8_waveform_samples_decoded": 0,
        "prior_v5_development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
        "method_holdout_accessed": False,
        "admission_shadow_accessed": False,
        "real_quality_credit_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def run(root: Path, arguments: argparse.Namespace) -> Path:
    directory = Path(__file__).resolve().parent
    paths = (arguments.object_1_feature, arguments.object_2_feature)
    sources = [
        common.validate_nisr_feature(root, path, descriptor)
        for path, descriptor in zip(paths, common.NISR_FILES, strict=True)
    ]
    environment = common.environment_identity()
    implementation = common.implementation_hashes(directory)
    manifest = build_manifest(environment, implementation, sources)

    first_modal = model.modal_recovery_control()
    second_modal = model.modal_recovery_control()
    modal_exact_repeat = first_modal == second_modal
    field_results = []
    field_exact_repeat = True
    for source in sources:
        first = model.train_mode_shape_field(source)
        second = model.train_mode_shape_field(source)
        field_exact_repeat &= first == second
        field_results.append(first)
    passed = (
        first_modal["passed"]
        and modal_exact_repeat
        and field_exact_repeat
        and all(item["passed"] for item in field_results)
    )
    decision = (
        "READY_FOR_FRESH_REAL_MODAL_PROTOCOL"
        if passed
        else "STOP_V8_SYNTH_KEEP_CLIPS"
    )

    output, staging = common.prepare_output(root, arguments.output)
    try:
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": common.REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "modal_recovery": first_modal,
            "modal_exact_repeat": modal_exact_repeat,
            "mode_shape_fields": field_results,
            "field_exact_repeat": field_exact_repeat,
            "opened_synthetic_development_files": len(sources),
            "real_v8_waveform_samples_decoded": 0,
            "prior_v5_development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "real_quality_credit": False,
            "next_authorized_step": (
                "FREEZE_FRESH_OBJECTFOLDER_REAL_FIT_DEVELOPMENT_PROTOCOL"
                if passed
                else "DIAGNOSE_V8_SYNTH_WITHOUT_REAL_DATA"
            ),
            "r3b_authorized": False,
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
    value = json.loads((output / "report.json").read_bytes())
    print(f"R3A V8 synthetic preflight: {output}")
    print(f"decision: {value['decision']}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
