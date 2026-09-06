#!/usr/bin/env python3
"""Run the frozen R3A V9 time-varying residual synthetic preflight."""

from __future__ import annotations

import argparse
import json
import math
import shutil
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_contact_field_r3a_v4_common as endpoint
import physical_sound_contact_field_r3a_v9_common as common
import physical_sound_contact_field_r3a_v9_model as model


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def build_manifest(
    environment: dict[str, Any], implementation: dict[str, str]
) -> dict[str, Any]:
    return {
        "schema": common.MANIFEST_SCHEMA,
        "status": "FrozenSyntheticPreflight",
        "study_id": common.STUDY_ID,
        "revision": common.REVISION,
        "source": {
            "kind": "deterministic_known_truth",
            "real_audio": False,
            "sample_rate_hz": common.SAMPLE_RATE_HZ,
            "sample_count": common.SAMPLE_COUNT,
            "grid_size": common.GRID_SIZE,
            "point_count": common.GRID_SIZE * common.GRID_SIZE,
        },
        "representation": {
            "explicit_mode_count": len(common.MODAL_FREQUENCIES_HZ),
            "noise_band_count": common.NOISE_BAND_COUNT,
            "noise_loop_samples": common.NOISE_LOOP_SAMPLES,
            "noise_seed": common.NOISE_SEED,
            "residual_atom_count": common.RESIDUAL_ATOM_COUNT,
            "residual_frame_hop": common.RESIDUAL_FRAME_HOP,
            "query_modal_role": "known_truth_oracle_to_isolate_residual_field",
        },
        "field": common.FIELD_CONFIG,
        "gates": common.GATES,
        "maximum_shared_bytes": common.MAXIMUM_SHARED_BYTES,
        "maximum_contact_bytes": common.MAXIMUM_CONTACT_BYTES,
        "environment": environment,
        "implementation_sha256": implementation,
        "real_waveform_sample_values_decoded": 0,
        "prior_v8_development_waveform_sample_values_decoded": 0,
        "sealed_waveform_sample_values_decoded": 0,
        "method_holdout_accessed": False,
        "admission_shadow_accessed": False,
        "real_quality_credit_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def normalized_rmse(target: np.ndarray, candidate: np.ndarray) -> float:
    numerator = math.sqrt(float(np.mean(np.square(candidate - target))))
    denominator = max(math.sqrt(float(np.mean(np.square(target)))), 1.0e-30)
    return numerator / denominator


def renderer_and_record_control(
    coordinates: np.ndarray,
    gains: np.ndarray,
    latents: np.ndarray,
    noise_bands: np.ndarray,
    atoms: np.ndarray,
) -> tuple[dict[str, Any], list[dict[str, np.ndarray]], list[np.ndarray]]:
    decoded = []
    targets = []
    records = []
    errors = []
    renderer_repeat = True
    for coordinate, contact_gains, latent in zip(
        coordinates, gains, latents, strict=True
    ):
        target = model.render_contact(contact_gains, latent, noise_bands, atoms)
        record_a, decoded_a = model.encode_contact_record(
            coordinate, contact_gains, latent
        )
        record_b, decoded_b = model.encode_contact_record(
            coordinate, contact_gains, latent
        )
        if record_a != record_b:
            raise common.V9Error("contact record is not deterministic")
        candidate_a = model.render_contact(
            decoded_a["gains"], decoded_a["latent"], noise_bands, atoms
        )
        candidate_b = model.render_contact(
            decoded_b["gains"], decoded_b["latent"], noise_bands, atoms
        )
        renderer_repeat &= np.array_equal(candidate_a, candidate_b)
        errors.append(normalized_rmse(target, candidate_a))
        decoded.append(decoded_a)
        targets.append(target)
        records.append(record_a)
    maximum_bytes = max(len(record) for record in records)
    maximum_error = max(errors)
    report = {
        "basis_exact_repeat": True,
        "renderer_exact_repeat": renderer_repeat,
        "maximum_contact_record_bytes": maximum_bytes,
        "record_sha256": common.sha256_bytes(b"".join(records)),
        "maximum_record_waveform_nrmse": maximum_error,
        "record_error_gate_passed": (
            maximum_error <= common.GATES["maximum_record_waveform_nrmse"]
        ),
        "contact_budget_passed": maximum_bytes <= common.MAXIMUM_CONTACT_BYTES,
    }
    return report, decoded, targets


def query_waveform_control(
    decoded: list[dict[str, np.ndarray]],
    targets: list[np.ndarray],
    query_indices: np.ndarray,
    neural_latents: np.ndarray,
    nearest_latents: np.ndarray,
    noise_bands: np.ndarray,
    atoms: np.ndarray,
) -> dict[str, Any]:
    rows = []
    for query_row, contact_index in enumerate(query_indices):
        gains = decoded[int(contact_index)]["gains"]
        target = targets[int(contact_index)]
        neural = model.render_contact(
            gains, neural_latents[query_row], noise_bands, atoms
        )
        nearest = model.render_contact(
            gains, nearest_latents[query_row], noise_bands, atoms
        )
        modal = model.render_modal(gains)
        neural_metrics = endpoint.metrics(target, neural)
        nearest_metrics = endpoint.metrics(target, nearest)
        modal_metrics = endpoint.metrics(target, modal)
        rows.append(
            {
                "contact_index": int(contact_index),
                "neural_waveform_nrmse": normalized_rmse(target, neural),
                "nearest_waveform_nrmse": normalized_rmse(target, nearest),
                "modal_only_waveform_nrmse": normalized_rmse(target, modal),
                "neural_metrics": neural_metrics,
                "nearest_metrics": nearest_metrics,
                "modal_only_metrics": modal_metrics,
                "neural_prediction_sha256": model.array_hash(neural),
            }
        )
    neural_mean = float(np.mean([item["neural_waveform_nrmse"] for item in rows]))
    nearest_mean = float(np.mean([item["nearest_waveform_nrmse"] for item in rows]))
    modal_mean = float(np.mean([item["modal_only_waveform_nrmse"] for item in rows]))
    maximum_envelope = max(
        item["neural_metrics"]["normalized_envelope_rmse"] for item in rows
    )
    median_spectrum = float(
        np.median(
            [
                item["neural_metrics"][
                    "gain_matched_multiresolution_log_spectrum_rmse_db"
                ]
                for item in rows
            ]
        )
    )
    finite = all(
        math.isfinite(value)
        for item in rows
        for value in (
            item["neural_waveform_nrmse"],
            item["nearest_waveform_nrmse"],
            item["modal_only_waveform_nrmse"],
            *item["neural_metrics"].values(),
        )
    )
    return {
        "finite": finite,
        "neural_mean_waveform_nrmse": neural_mean,
        "nearest_mean_waveform_nrmse": nearest_mean,
        "modal_only_mean_waveform_nrmse": modal_mean,
        "neural_to_nearest_waveform_nrmse_ratio": neural_mean / nearest_mean,
        "neural_below_modal_only": neural_mean < modal_mean,
        "maximum_neural_query_envelope_rmse": maximum_envelope,
        "median_neural_query_log_spectrum_rmse_db": median_spectrum,
        "query_contacts": rows,
    }


def run(root: Path, output_path: Path) -> Path:
    directory = Path(__file__).resolve().parent
    environment = common.environment_identity()
    implementation = common.implementation_hashes(directory)
    manifest = build_manifest(environment, implementation)

    first_bands = model.generate_noise_bands()
    second_bands = model.generate_noise_bands()
    basis_repeat = np.array_equal(first_bands, second_bands)
    if not basis_repeat:
        raise common.V9Error("noise-band basis did not repeat")
    atoms = model.residual_atoms()
    coordinates = model.grid_coordinates()
    latents = model.truth_latents(coordinates)
    gains = model.truth_modal_gains(coordinates)
    record_report, decoded, targets = renderer_and_record_control(
        coordinates, gains, latents, first_bands, atoms
    )
    record_report["basis_exact_repeat"] = basis_repeat
    record_report["noise_basis_sha256"] = model.array_hash(first_bands)
    record_report["residual_atoms_sha256"] = model.array_hash(atoms)

    first_field, first_prediction, first_nearest, first_query = (
        model.train_latent_field(coordinates, latents)
    )
    second_field, second_prediction, second_nearest, second_query = (
        model.train_latent_field(coordinates, latents)
    )
    field_repeat = (
        first_field == second_field
        and np.array_equal(first_prediction, second_prediction)
        and np.array_equal(first_nearest, second_nearest)
        and np.array_equal(first_query, second_query)
    )
    query_report = query_waveform_control(
        decoded,
        targets,
        first_query,
        first_prediction,
        first_nearest,
        first_bands,
        atoms,
    )

    renderer_descriptor_bytes = (
        np.asarray(common.MODAL_FREQUENCIES_HZ, dtype="<f8").nbytes
        + np.asarray(common.MODAL_DAMPING_PER_SECOND, dtype="<f8").nbytes
        + model.band_centers_hz().astype("<f8").nbytes
        + 8 * common.RESIDUAL_ATOM_COUNT * 8
    )
    shared_bytes = first_field["parameter_bytes"] + renderer_descriptor_bytes
    budget_gate = {
        "shared_decoder": shared_bytes <= common.MAXIMUM_SHARED_BYTES,
        "contact_record": record_report["contact_budget_passed"],
    }
    gates = {
        "finite": first_field["finite"] and query_report["finite"],
        "basis_exact_repeat": basis_repeat,
        "renderer_exact_repeat": record_report["renderer_exact_repeat"],
        "field_exact_repeat": field_repeat,
        "record_waveform": record_report["record_error_gate_passed"],
        "latent_beats_constant": first_field["neural_below_constant"],
        "latent_beats_nearest": (
            first_field["neural_to_nearest_latent_rmse_ratio"]
            <= common.GATES["maximum_neural_to_nearest_latent_rmse_ratio"]
        ),
        "waveform_beats_nearest": (
            query_report["neural_to_nearest_waveform_nrmse_ratio"]
            <= common.GATES["maximum_neural_to_nearest_waveform_nrmse_ratio"]
        ),
        "waveform_beats_modal_only": query_report["neural_below_modal_only"],
        "envelope": (
            query_report["maximum_neural_query_envelope_rmse"]
            <= common.GATES["maximum_query_envelope_rmse"]
        ),
        "spectrum": (
            query_report["median_neural_query_log_spectrum_rmse_db"]
            <= common.GATES["maximum_median_query_log_spectrum_rmse_db"]
        ),
        "shared_budget": budget_gate["shared_decoder"],
        "contact_budget": budget_gate["contact_record"],
        "protected_reads_zero": True,
    }
    passed = all(gates.values())
    decision = (
        "READY_TO_FREEZE_SOURCE_DISJOINT_V9_REAL_PROTOCOL"
        if passed
        else "REJECT_V9_RESIDUAL_SUBSTRATE"
    )

    output, staging = common.prepare_output(root, output_path)
    try:
        manifest_bytes = common.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": common.REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "renderer_and_record": record_report,
            "latent_field": first_field,
            "field_exact_repeat": field_repeat,
            "query_waveforms": query_report,
            "budget": {
                "shared_decoder_bytes": shared_bytes,
                "renderer_descriptor_bytes": renderer_descriptor_bytes,
                "maximum_contact_record_bytes": record_report[
                    "maximum_contact_record_bytes"
                ],
                "gate": budget_gate,
            },
            "gates": gates,
            "real_waveform_sample_values_decoded": 0,
            "prior_v8_development_waveform_sample_values_decoded": 0,
            "sealed_waveform_sample_values_decoded": 0,
            "method_holdout_accessed": False,
            "admission_shadow_accessed": False,
            "real_quality_credit": False,
            "r3b_authorized": False,
            "runtime_or_public_contract_changed": False,
            "authored_clip_fallback_required": True,
        }
        (staging / "report.json").write_bytes(common.canonical_json(report))
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    output = run(common.repository_root(), arguments.output)
    value = json.loads((output / "report.json").read_bytes())
    print(f"R3A V9 synthetic preflight: {output}")
    print(f"decision: {value['decision']}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()

