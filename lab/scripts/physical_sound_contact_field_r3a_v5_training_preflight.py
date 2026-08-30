#!/usr/bin/env python3
"""Freeze and validate the R3A V5 CUDA training environment and runner."""

from __future__ import annotations

import argparse
import importlib.metadata
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_training as training
import scipy
import torch

SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-training.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-training.report.v1"
REVISION = "cuda-runner-controls-v1"
V5_PREFLIGHT_MANIFEST_SHA256 = (
    "1cc234962963232a1c33c2b7c9973a13788743d07225c77073dbfbfc0b60fb31"
)
EXPECTED_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "torch": "2.12.1+cu130",
    "torch_cuda": "13.0",
    "gpu_name": "NVIDIA GeForce RTX 3080",
    "gpu_uuid": "GPU-8a3af62f-30db-ce95-caf9-201a67bf5810",
    "gpu_driver": "610.43.02",
    "gpu_memory_mib": 10_240,
    "gpu_compute_capability": "8.6",
    "torch_gpu_memory_mib": 9_871,
}
IMPLEMENTATION_FILES = {
    "v5_common": "physical_sound_contact_field_r3a_v5_common.py",
    "v5_model": "physical_sound_contact_field_r3a_v5_model.py",
    "training": "physical_sound_contact_field_r3a_v5_training.py",
    "training_preflight": "physical_sound_contact_field_r3a_v5_training_preflight.py",
}


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--v5-manifest", required=True, type=Path)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def load_v5_manifest(root: Path, path: Path) -> dict[str, Any]:
    resolved = common.require_external_file(root, path, "V5 preflight manifest")
    payload, value = common.load_json(resolved, "V5 preflight manifest", canonical=True)
    if (
        common.sha256_bytes(payload) != V5_PREFLIGHT_MANIFEST_SHA256
        or value.get("schema") != common.MANIFEST_SCHEMA
        or value.get("neural_training_authorized") is not False
        or value.get("development_waveform_samples_decoded") != 0
        or value.get("sealed_waveform_samples_decoded") != 0
    ):
        raise common.V5Error("V5 preflight commitment changed")
    return value


def _nvidia_smi_identity() -> dict[str, Any]:
    command = [
        "nvidia-smi",
        "--query-gpu=name,uuid,driver_version,memory.total,compute_cap",
        "--format=csv,noheader,nounits",
    ]
    completed = subprocess.run(
        command,
        check=True,
        capture_output=True,
        text=True,
        timeout=10,
    )
    rows = [line.strip() for line in completed.stdout.splitlines() if line.strip()]
    if len(rows) != 1:
        raise common.V5Error("V5 requires exactly one visible GPU")
    fields = [field.strip() for field in rows[0].split(",")]
    if len(fields) != 5:
        raise common.V5Error("cannot parse the V5 GPU identity")
    return {
        "gpu_name": fields[0],
        "gpu_uuid": fields[1],
        "gpu_driver": fields[2],
        "gpu_memory_mib": int(fields[3]),
        "gpu_compute_capability": fields[4],
    }


def _package_inventory() -> list[dict[str, str]]:
    packages = {
        distribution.metadata["Name"].lower(): distribution.version
        for distribution in importlib.metadata.distributions()
        if distribution.metadata["Name"]
    }
    return [
        {"name": name, "version": version} for name, version in sorted(packages.items())
    ]


def training_environment() -> dict[str, Any]:
    if not torch.cuda.is_available() or torch.cuda.device_count() != 1:
        raise common.V5Error("V5 CUDA training device is unavailable or ambiguous")
    gpu = _nvidia_smi_identity()
    properties = torch.cuda.get_device_properties(0)
    observed = {
        "python": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "torch": torch.__version__,
        "torch_cuda": torch.version.cuda,
        "cudnn": torch.backends.cudnn.version(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "byteorder": sys.byteorder,
        "cublas_workspace_config": os.environ.get("CUBLAS_WORKSPACE_CONFIG", ""),
        "torch_gpu_memory_mib": properties.total_memory // (1024 * 1024),
        **gpu,
    }
    for key, expected in EXPECTED_ENVIRONMENT.items():
        if observed[key] != expected:
            raise common.V5Error(
                f"V5 training environment changed for {key}: {observed[key]}"
            )
    if observed["byteorder"] != "little":
        raise common.V5Error("V5 requires a little-endian training host")
    if observed["cublas_workspace_config"] != ":4096:8":
        raise common.V5Error("V5 CUBLAS workspace configuration changed")
    if (
        properties.name != EXPECTED_ENVIRONMENT["gpu_name"]
        or f"{properties.major}.{properties.minor}"
        != EXPECTED_ENVIRONMENT["gpu_compute_capability"]
        or observed["torch_gpu_memory_mib"]
        != EXPECTED_ENVIRONMENT["torch_gpu_memory_mib"]
    ):
        raise common.V5Error("PyTorch and nvidia-smi GPU identities disagree")
    packages = _package_inventory()
    observed["packages"] = packages
    observed["packages_sha256"] = common.sha256_bytes(common.canonical_json(packages))
    return observed


def implementation_hashes(directory: Path) -> dict[str, str]:
    return {
        key: common.sha256_file(directory / filename)
        for key, filename in IMPLEMENTATION_FILES.items()
    }


def decoded_descriptors(decoded: list[training.DecodedClip]) -> list[dict[str, Any]]:
    return [
        {
            "member": clip.member,
            "role": clip.role,
            "event_group": clip.event_group,
            "source_scalar_samples_decoded": clip.source_scalar_samples_decoded,
            "segment_samples": int(clip.samples.size),
            "segment_sha256": common.sha256_bytes(clip.samples.tobytes()),
            "output_gain": clip.output_gain,
        }
        for clip in decoded
    ]


def run(root: Path, arguments: argparse.Namespace) -> Path:
    directory = Path(__file__).resolve().parent
    v5_manifest = load_v5_manifest(root, arguments.v5_manifest)
    archive = common.require_external_file(root, arguments.archive, "Heller archive")
    if common.sha256_file(archive) != common.ARCHIVE_SHA256:
        raise common.V5Error("V5 training archive changed")
    environment = training_environment()
    output, staging = common.prepare_output(root, arguments.output)
    try:
        decoded = training.decode_control_pair(archive, v5_manifest["corpus"])
        controls = training.run_training_controls(
            decoded, staging / "control-checkpoint.pt"
        )
        decoded_records = decoded_descriptors(decoded)
        manifest = {
            "schema": SCHEMA,
            "status": "FrozenTrainingControls",
            "study_id": common.STUDY_ID,
            "revision": REVISION,
            "v5_preflight_manifest_sha256": V5_PREFLIGHT_MANIFEST_SHA256,
            "source_archive_sha256": common.ARCHIVE_SHA256,
            "environment": environment,
            "loss_implementation_revision": training.LOSS_IMPLEMENTATION_REVISION,
            "control_policy": {
                "capacity_id": training.CONTROL_CAPACITY_ID,
                "quantizers": training.CONTROL_QUANTIZERS,
                "codebook_initialization_revision": training.CODEBOOK_INITIALIZATION_REVISION,
                "optimization_steps_before_checkpoint": training.CONTROL_OPTIMIZATION_STEPS,
                "minimum_relative_l1_improvement": training.CONTROL_MINIMUM_L1_IMPROVEMENT,
                "minimum_train_unique_codes_per_quantizer": training.CONTROL_MINIMUM_TRAIN_UNIQUE_CODES_PER_QUANTIZER,
                "minimum_validation_active_quantizers": training.CONTROL_MINIMUM_VALIDATION_ACTIVE_QUANTIZERS,
                "exact_checkpoint_resume_required": True,
            },
            "decoded_control_clips": decoded_records,
            "controls": controls,
            "implementation_sha256": implementation_hashes(directory),
            "canonical_tracking": common.TRACKING_CONTRACT,
            "source_waveform_scalar_samples_decoded": sum(
                clip.source_scalar_samples_decoded for clip in decoded
            ),
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "real_capacity_training_steps": 0,
            "frozen_capacity_training_authorized": True,
            "development_access_authorized": False,
            "holdout_access_authorized": False,
            "runtime_neural_inference_authorized": False,
            "authored_clip_fallback_required": True,
            "next_authorized_step": "TRAIN_THREE_FROZEN_CAPACITIES_WITHOUT_DEVELOPMENT_READS",
        }
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": "READY_FOR_FROZEN_CAPACITY_TRAINING",
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "control_checkpoint_sha256": controls["checkpoint_sha256"],
            "control_checkpoint_bytes": controls["checkpoint_bytes"],
            "decoded_control_clip_count": len(decoded),
            "source_waveform_scalar_samples_decoded": sum(
                clip.source_scalar_samples_decoded for clip in decoded
            ),
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
            "control_optimizer_steps": (training.CONTROL_OPTIMIZATION_STEPS + 2),
            "real_capacity_training_steps": 0,
            "exact_checkpoint_resume": controls["exact_checkpoint_resume"],
            "minimum_train_unique_codes": min(
                controls["train_unique_codes_per_quantizer"]
            ),
            "minimum_validation_unique_codes": min(
                controls["validation_unique_codes_per_quantizer"]
            ),
            "validation_active_quantizers": controls["validation_active_quantizers"],
            "relative_l1_improvement": controls["relative_l1_improvement"],
            "mlflow_dependency_installed": any(
                package["name"] == "mlflow" for package in environment["packages"]
            ),
            "mlflow_policy": "optional_external_local_file_store_mirror",
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
    _, report = common.load_json(output / "report.json", "V5 training report", True)
    print(f"R3A V5 CUDA runner preflight: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
