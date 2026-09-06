#!/usr/bin/env python3
"""Evaluate the three frozen V5-C codecs once on four development contacts."""

from __future__ import annotations

import argparse
import math
import shutil
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v4_common as endpoint
import physical_sound_contact_field_r3a_v5_capacity_frontier as frontier
import physical_sound_contact_field_r3a_v5_capacity_train as capacity_train
import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as codec_model
import physical_sound_contact_field_r3a_v5_training as training
import physical_sound_contact_field_r3a_v5_training_preflight as training_preflight
import torch

SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-development.v1"
REVISION = "three-capacity-single-development-evaluation-v1"
FRONTIER_REPORT_SHA256 = (
    "2db6e7c62f8d20bb3d0350b3f5ad677b3b1408e77877a683cc14677ea1caa911"
)
V4_MANIFEST_SHA256 = capacity_train.V4_MANIFEST_SHA256
OUTPUT_GAIN_DTYPE = "float32"


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frontier-report", required=True, type=Path)
    parser.add_argument("--v4-manifest", required=True, type=Path)
    parser.add_argument(
        "--run",
        action="append",
        default=[],
        metavar="CAPACITY_ID=EXTERNAL_DIRECTORY",
    )
    parser.add_argument(
        "--fit-extraction",
        action="append",
        default=[],
        metavar="OBJECT_ID=EXTERNAL_DIRECTORY",
    )
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def load_frontier(root: Path, path: Path) -> dict[str, Any]:
    resolved = common.require_external_file(root, path, "V5 frontier report")
    payload, value = common.load_json(resolved, "V5 frontier report", True)
    if (
        common.sha256_bytes(payload) != FRONTIER_REPORT_SHA256
        or value.get("schema") != frontier.SCHEMA
        or value.get("decision") != "BOUNDED_CAPACITY_FRONTIER_COMPLETE"
        or value.get("next_development_evaluation_authorized") is not True
        or value.get("development_access_authorized") is not False
        or value.get("holdout_access_authorized") is not False
        or value.get("long_capacity_training_authorized") is not False
    ):
        raise common.V5Error("V5 frontier does not authorize development")
    return value


def load_development_contacts(
    root: Path,
    v4_manifest: dict[str, Any],
    directories: dict[str, Path],
) -> tuple[list[dict[str, Any]], int]:
    by_id = {item["id"]: item for item in v4_manifest["objects"]}
    objects = []
    decoded_samples = 0
    for object_id in common.V4_OBJECT_IDS:
        descriptor = by_id[object_id]
        directory = capacity_train._require_external_directory(
            root, directories[object_id], f"{object_id} development extraction"
        )
        metadata_path = directory / "contacts-metadata.json"
        contacts_path = directory / "contacts.npy"
        report_path = directory / "report.json"
        metadata_payload, metadata = common.load_json(
            metadata_path, f"{object_id} development metadata", True
        )
        report_payload, report = common.load_json(
            report_path, f"{object_id} development report", True
        )
        contacts = metadata.get("contacts", [])
        if (
            common.sha256_bytes(metadata_payload) != descriptor["metadata_sha256"]
            or common.sha256_bytes(report_payload) != descriptor["report_sha256"]
            or common.sha256_file(contacts_path) != descriptor["contacts_sha256"]
            or [item.get("role") for item in contacts]
            != ["fit", "fit", "fit", "representation_development"]
            or metadata.get("sample_rate_hz") != endpoint.SAMPLE_RATE_HZ
            or metadata.get("sample_count") != descriptor["sample_count"]
            or report.get("sealed_waveform_samples_decoded") != 0
        ):
            raise common.V5Error(
                f"V5 development extraction lineage changed for {object_id}"
            )
        values = np.load(contacts_path, mmap_mode="r", allow_pickle=False)
        if values.shape != (4, descriptor["sample_count"]):
            raise common.V5Error(f"V5 development array changed for {object_id}")
        aligned = [
            endpoint.align_peak(np.asarray(values[index], dtype=np.float64))
            for index in range(4)
        ]
        positions = np.asarray(
            [item["position_metres"] for item in contacts], dtype=np.float64
        )
        if positions.shape != (4, 3) or not np.isfinite(positions).all():
            raise common.V5Error(f"V5 development positions changed for {object_id}")
        decoded_samples += int(values[3].size)
        objects.append(
            {
                "object_id": object_id,
                "fit": aligned[:3],
                "development": aligned[3],
                "positions": positions,
                "development_aligned_sha256": common.sha256_bytes(
                    np.asarray(aligned[3], dtype="<f8").tobytes()
                ),
                "development_source_samples_decoded": int(values[3].size),
                "sealed_waveform_samples_decoded": 0,
            }
        )
    return objects, decoded_samples


def matched_output_gain(target: np.ndarray, decoded: np.ndarray) -> np.float32:
    target_rms = endpoint.rms(target)
    decoded_rms = endpoint.rms(decoded)
    if not math.isfinite(decoded_rms) or decoded_rms <= 1.0e-12:
        raise common.V5Error("V5 decoded development waveform is silent")
    return np.float32(target_rms / decoded_rms)


def reconstruct(
    model: codec_model.NeuralImpactCodec,
    target: np.ndarray,
    quantizers: int,
    device: torch.device,
) -> tuple[np.ndarray, dict[str, Any]]:
    peak = float(np.max(np.abs(target)))
    if not math.isfinite(peak) or peak <= 0.0:
        raise common.V5Error("V5 development target is silent")
    normalization_gain = 0.92 / peak
    normalized = np.asarray(target * normalization_gain, dtype="<f4")
    value = torch.from_numpy(normalized.copy()).reshape(1, 1, -1).to(device)
    model.eval()
    with torch.inference_mode():
        output, codes, _, _ = model(value, quantizers)
    decoded = output[0, 0].cpu().numpy().astype(np.float64)
    if decoded.shape != target.shape or not np.isfinite(decoded).all():
        raise common.V5Error("V5 decoded development waveform is invalid")
    output_gain = matched_output_gain(target, decoded)
    candidate = np.asarray(decoded * float(output_gain), dtype=np.float64)
    bits_per_code = math.ceil(math.log2(model.config.codebook_size))
    code_bytes = (codes.numel() * bits_per_code + 7) // 8
    return candidate, {
        "normalization_gain": normalization_gain,
        "output_gain": float(output_gain),
        "output_gain_dtype": OUTPUT_GAIN_DTYPE,
        "code_shape": list(codes.shape),
        "encoded_code_bytes": code_bytes,
        "encoded_record_bytes": code_bytes + 4,
        "codes_sha256": codec_model.tensor_sha256(codes),
        "candidate_sha256": common.sha256_bytes(
            np.asarray(candidate, dtype="<f8").tobytes()
        ),
    }


def select_smallest_passing(results: list[dict[str, Any]]) -> str | None:
    passing = {item["capacity_id"] for item in results if item["passed"]}
    return next(
        (item["id"] for item in common.CAPACITIES if item["id"] in passing), None
    )


def run(root: Path, arguments: argparse.Namespace) -> Path:
    output = arguments.output.resolve()
    if output.is_relative_to(root):
        raise common.V5Error("V5 development output must stay outside repository")
    frontier_report = load_frontier(root, arguments.frontier_report)
    run_directories = frontier.parse_runs(arguments.run)
    run_records = [
        frontier.load_run(root, item["id"], run_directories[item["id"]])
        for item in common.CAPACITIES
    ]
    expected_by_id = {
        item["capacity_id"]: item for item in frontier_report["ranking"]
    }
    for record in run_records:
        expected = expected_by_id.get(record["capacity_id"], {})
        if any(
            record[key] != expected.get(key)
            for key in ("run_manifest_sha256", "metrics_sha256", "report_sha256")
        ):
            raise common.V5Error("V5 development run differs from frontier")
    v4_manifest = capacity_train.load_parent_manifest(
        root,
        arguments.v4_manifest,
        "V4 development manifest",
        V4_MANIFEST_SHA256,
    )
    fit_directories = capacity_train.parse_fit_extractions(
        arguments.fit_extraction
    )
    objects, development_samples = load_development_contacts(
        root, v4_manifest, fit_directories
    )
    environment = training_preflight.training_environment()
    training.configure_training_determinism(common.TRAINING_CONFIG["random_seed"])
    device = torch.device("cuda:0")
    capacity_results = []
    for capacity in common.CAPACITIES:
        run_record = next(
            item for item in run_records if item["capacity_id"] == capacity["id"]
        )
        checkpoint_path = (
            run_directories[capacity["id"]].resolve()
            / run_record["checkpoint"]["path"]
        )
        checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=True)
        model = codec_model.NeuralImpactCodec(codec_model.CodecConfig.frozen()).to(
            device
        )
        if (
            checkpoint.get("capacity_id") != capacity["id"]
            or checkpoint.get("quantizers") != capacity["quantizers"]
            or checkpoint.get("step") != frontier.FRONTIER_STEP
            or checkpoint.get("run_manifest_sha256")
            != run_record["run_manifest_sha256"]
        ):
            raise common.V5Error("V5 development checkpoint identity changed")
        model.load_state_dict(checkpoint["model"], strict=True)
        object_results = []
        for object_record in objects:
            target = object_record["development"]
            positions = object_record["positions"]
            distances = np.linalg.norm(positions[:3] - positions[3], axis=1)
            nearest = int(np.argmin(distances))
            baseline = object_record["fit"][nearest]
            candidate, encoded = reconstruct(
                model, target, capacity["quantizers"], device
            )
            candidate_metrics = endpoint.metrics(target, candidate)
            baseline_metrics = endpoint.metrics(target, baseline)
            gate = endpoint.candidate_gate(candidate_metrics, baseline_metrics)
            record_budget_passed = encoded["encoded_record_bytes"] <= 64 * 1024
            object_results.append(
                {
                    "object_id": object_record["object_id"],
                    "development_aligned_sha256": object_record[
                        "development_aligned_sha256"
                    ],
                    "nearest_fit_contact_index": nearest,
                    "nearest_fit_distance_metres": float(distances[nearest]),
                    "baseline_metrics": baseline_metrics,
                    "candidate_metrics": candidate_metrics,
                    "candidate_gate": gate,
                    "encoded": encoded,
                    "record_budget_passed": record_budget_passed,
                    "passed": gate["passed"] and record_budget_passed,
                }
            )
        capacity_results.append(
            {
                "capacity_id": capacity["id"],
                "quantizers": capacity["quantizers"],
                "nominal_bits_per_second": capacity["nominal_bits_per_second"],
                "checkpoint": run_record["checkpoint"],
                "objects": object_results,
                "passed": all(item["passed"] for item in object_results),
            }
        )
        del model
        torch.cuda.empty_cache()
    selected = select_smallest_passing(capacity_results)
    decision = (
        "READY_FOR_V5_HOLDOUT"
        if selected is not None
        else "REJECT_NEURAL_REPRESENTATION"
    )
    report = {
        "schema": SCHEMA,
        "status": "Validated",
        "decision": decision,
        "study_id": common.STUDY_ID,
        "revision": REVISION,
        "implementation_sha256": common.sha256_file(Path(__file__).resolve()),
        "frontier_report_sha256": FRONTIER_REPORT_SHA256,
        "environment": environment,
        "evaluation_policy": {
            "primary_endpoints": list(endpoint.PRIMARY_ENDPOINTS),
            "absolute_thresholds": endpoint.ABSOLUTE_THRESHOLDS,
            "baseline": "nearest_fit_contact_by_euclidean_mesh_position",
            "minimum_beaten_endpoints": endpoint.MINIMUM_BEATEN_ENDPOINTS,
            "maximum_normalized_error_ratio": endpoint.MAXIMUM_NORMALIZED_ERROR_RATIO,
            "selection": "smallest_capacity_passing_every_object",
            "per_record_budget_bytes": 64 * 1024,
            "output_gain_sidecar": OUTPUT_GAIN_DTYPE,
        },
        "capacities": capacity_results,
        "selected_capacity_id": selected,
        "development_object_count": len(objects),
        "development_accessed": True,
        "development_waveform_samples_decoded": development_samples,
        "sealed_waveform_samples_decoded": 0,
        "method_holdout_waveform_samples_decoded": 0,
        "admission_shadow_waveform_samples_decoded": 0,
        "holdout_access_authorized": selected is not None,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
        "next_authorized_step": (
            "FREEZE_ONE_SOURCE_DISJOINT_REPRESENTATION_HOLDOUT"
            if selected is not None
            else "STOP_V5_KEEP_AUTHORED_CLIPS"
        ),
    }
    output, staging = common.prepare_output(root, output)
    try:
        (staging / "report.json").write_bytes(common.canonical_json(report))
        common.publish_output(output, staging)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    output = run(common.repository_root(), arguments)
    _, report = common.load_json(output / "report.json", "V5 development", True)
    print(f"R3A V5 development evaluation: {output}")
    print(f"decision: {report['decision']}")
    print(f"selected capacity: {report['selected_capacity_id']}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
