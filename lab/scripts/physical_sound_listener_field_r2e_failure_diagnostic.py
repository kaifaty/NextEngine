#!/usr/bin/env python3
"""Post-reject query projection oracle for the frozen R2E listener field."""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Any

import numpy as np
import torch

import physical_sound_listener_field_r2c_evaluate as r2c_evaluate
import physical_sound_listener_field_r2e_evaluate as r2e_evaluate
import physical_sound_listener_field_r2e_train as r2e_train

common = r2e_train.common
r2c = common.r2c
v1 = common.v1

REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-failure-"
    "diagnostic.report.v1"
)
REVISION = "green-goblet-r2e-post-reject-query-projection-oracle-v1"
ORACLE_ID = "post_reject_rank96_query_projection_oracle"
R2E_EVALUATION_REPORT_SHA256 = (
    "efab8e2cf9d278412445edb2b57695cc7f53ce0ac883b015e21a2435abbb2327"
)


class DiagnosticError(RuntimeError):
    """The post-reject diagnostic escaped its frozen evidence boundary."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evaluation-manifest", required=True, type=Path)
    parser.add_argument("--evaluation-report", required=True, type=Path)
    parser.add_argument("--training-manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def validate_rejection(root: Path, argument: Path) -> tuple[bytes, dict[str, Any]]:
    path = r2c.external_file(root, argument, "R2E rejection report")
    data, report = r2c.read_json(path, "R2E rejection report")
    if (
        r2c.sha256_bytes(data) != R2E_EVALUATION_REPORT_SHA256
        or report.get("schema") != common.EVALUATION_REPORT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "RejectLowRankCoefficientField"
        or report.get("selected_candidate_id") is not None
        or report.get("quality_admission_or_runtime_authorized") is not False
    ):
        raise DiagnosticError("R2E rejection evidence changed")
    return data, report


def load_inputs(
    root: Path,
    evaluation_manifest_argument: Path,
    training_manifest_argument: Path,
) -> tuple[
    bytes,
    dict[str, Any],
    bytes,
    dict[str, Any],
    dict[str, Any],
]:
    evaluation_path = r2c.external_file(
        root, evaluation_manifest_argument, "R2E evaluation manifest"
    )
    evaluation_bytes, evaluation_manifest = r2c.read_json(
        evaluation_path, "R2E evaluation manifest"
    )
    evaluation_inputs = r2e_evaluate.validate_manifest(
        root, evaluation_path, evaluation_manifest
    )
    training_path = r2c.external_file(
        root, training_manifest_argument, "R2E training manifest"
    )
    training_bytes, training_manifest = r2c.read_json(
        training_path, "R2E training manifest"
    )
    _, training_inputs = r2e_train.validate_manifest(
        root, training_path, training_manifest
    )
    return (
        evaluation_bytes,
        evaluation_inputs,
        training_bytes,
        training_manifest,
        training_inputs,
    )


def reference_spectra(
    rows: list[dict[str, Any]],
    references: dict[int, Path],
    destination: Path,
) -> tuple[np.memmap, int, int]:
    transform = common.r2b.ComplexTransform(common.r2b.SAMPLE_COUNT)
    total_tf = transform.frame_count * (common.r2b.FFT_LENGTH // 2 + 1)
    spectra = np.memmap(
        destination, dtype="<c8", mode="w+", shape=(len(rows), total_tf)
    )
    byte_count = 0
    for offset, row in enumerate(rows):
        reference = references[row["row_index"]]
        signal = r2c_evaluate.read_pcm16(reference)
        spectrum = transform.analyze(signal)
        if not np.isfinite(spectrum).all():
            raise DiagnosticError("query reference produced a non-finite spectrum")
        spectra[offset] = spectrum.reshape(-1).astype("<c8", copy=False)
        byte_count += reference.stat().st_size
    spectra.flush()
    return spectra, total_tf, byte_count


def project_coefficients(
    spectra: np.memmap,
    arrays: dict[str, np.ndarray],
    device: torch.device,
) -> tuple[np.ndarray, dict[str, float]]:
    row_count, total_tf = spectra.shape
    coefficients = torch.zeros(
        (row_count, common.RANK), dtype=torch.complex64, device=device
    )
    centered_energy = 0.0
    total_energy = 0.0
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            query = torch.from_numpy(
                np.array(spectra[:, start:stop], dtype=np.complex64, copy=True)
            ).to(device)
            mean = torch.from_numpy(
                np.array(
                    arrays["mean_field"][start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            basis = torch.from_numpy(
                np.array(
                    arrays["basis"][:, start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            centered = query - mean[None, :]
            coefficients += centered @ torch.conj(basis.T)
            centered_energy += float(torch.sum(torch.abs(centered) ** 2).cpu())
            total_energy += float(torch.sum(torch.abs(query) ** 2).cpu())
    coefficient_energy = float(torch.sum(torch.abs(coefficients) ** 2).cpu())
    residual_energy = max(centered_energy - coefficient_energy, 0.0)
    if not math.isfinite(total_energy) or total_energy <= 0.0:
        raise DiagnosticError("query projection energy is invalid")
    diagnostics = {
        "query_total_spectral_energy": total_energy,
        "query_centered_spectral_energy": centered_energy,
        "query_projected_centered_energy": coefficient_energy,
        "query_residual_spectral_energy": residual_energy,
        "query_rank96_projection_frobenius_nrmse": math.sqrt(
            residual_energy / total_energy
        ),
        "query_rank96_projection_total_energy_fraction": max(
            0.0, min(1.0, 1.0 - residual_energy / total_energy)
        ),
    }
    return coefficients.cpu().numpy().astype("<c8"), diagnostics


def cook_oracle(
    root: Path,
    rows: list[dict[str, Any]],
    references: dict[int, Path],
    spectra: np.memmap,
    coefficients: np.ndarray,
    arrays: dict[str, np.ndarray],
    staging: Path,
    device: torch.device,
) -> tuple[dict[str, Any], dict[str, Any], str]:
    row_count, total_tf = spectra.shape
    frequency_bins = common.r2b.FFT_LENGTH // 2 + 1
    band_edges_hz = (
        0,
        3_000,
        8_000,
        16_000,
        common.r2b.SAMPLE_RATE_HZ // 2 + 1,
    )
    band_energy = {
        f"{lower}_{upper}_hz": {"reference": 0.0, "residual": 0.0}
        for lower, upper in zip(band_edges_hz[:-1], band_edges_hz[1:])
    }
    coefficient_tensor = torch.from_numpy(
        np.array(coefficients, dtype=np.complex64, copy=True)
    ).to(device)
    with torch.no_grad():
        for start in range(0, total_tf, common.TF_CHUNK):
            stop = min(total_tf, start + common.TF_CHUNK)
            mean = torch.from_numpy(
                np.array(
                    arrays["mean_field"][start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            basis = torch.from_numpy(
                np.array(
                    arrays["basis"][:, start:stop],
                    dtype=np.complex64,
                    copy=True,
                )
            ).to(device)
            reconstructed = mean[None, :] + coefficient_tensor @ basis
            reference = torch.from_numpy(
                np.array(spectra[:, start:stop], dtype=np.complex64, copy=True)
            ).to(device)
            bin_indices = torch.arange(start, stop, device=device) % frequency_bins
            bin_hz = (
                bin_indices.to(torch.float32)
                * common.r2b.SAMPLE_RATE_HZ
                / common.r2b.FFT_LENGTH
            )
            for lower, upper in zip(band_edges_hz[:-1], band_edges_hz[1:]):
                mask = (bin_hz >= lower) & (bin_hz < upper)
                label = f"{lower}_{upper}_hz"
                band_energy[label]["reference"] += float(
                    torch.sum(torch.abs(reference[:, mask]) ** 2).cpu()
                )
                band_energy[label]["residual"] += float(
                    torch.sum(
                        torch.abs(reference[:, mask] - reconstructed[:, mask]) ** 2
                    ).cpu()
                )
            spectra[:, start:stop] = reconstructed.cpu().numpy().astype("<c8")
    spectra.flush()

    transform = common.r2b.ComplexTransform(common.r2b.SAMPLE_COUNT)
    predictions_root = staging / "predictions"
    predictions_root.mkdir()
    predictions = []
    for offset, row in enumerate(rows):
        spectrum = np.asarray(spectra[offset]).reshape(
            transform.frame_count, common.r2b.FFT_LENGTH // 2 + 1
        )
        signal = transform.synthesize(spectrum)
        peak = float(np.max(np.abs(signal)))
        if not np.isfinite(signal).all() or peak >= 1.0:
            raise DiagnosticError(
                f"query projection oracle is not cookable at row {row['row_index']}"
            )
        path = predictions_root / f"row-{row['row_index']:04}.wav"
        payload = r2c.encode_wav(signal)
        path.write_bytes(payload)
        predictions.append(
            {
                "row_index": row["row_index"],
                "prediction_file": str(path.relative_to(staging)),
                "prediction_sha256": r2c.sha256_bytes(payload),
                "prediction_byte_count": len(payload),
                "pre_cook_peak_abs": peak,
            }
        )

    synthetic_report = staging / "oracle-predictions.json"
    synthetic_report.write_bytes(r2c.canonical_json({"predictions": predictions}))
    prediction_grid_sha256 = r2c.sha256_bytes(
        r2c.canonical_json(
            [
                {
                    "row_index": prediction["row_index"],
                    "prediction_sha256": prediction["prediction_sha256"],
                    "prediction_byte_count": prediction["prediction_byte_count"],
                }
                for prediction in predictions
            ]
        )
    )
    evaluated = r2c_evaluate.evaluate_candidate(
        root,
        ORACLE_ID,
        {
            "path": synthetic_report,
            "report": {"predictions": predictions},
        },
        references,
        staging,
    )
    frequency_diagnostics = {
        label: {
            **values,
            "projection_frobenius_nrmse": math.sqrt(
                values["residual"] / values["reference"]
            ),
        }
        for label, values in band_energy.items()
    }
    return evaluated, frequency_diagnostics, prediction_grid_sha256


def classify(
    oracle: dict[str, Any], controls: list[dict[str, Any]]
) -> tuple[str, dict[str, Any]]:
    compared = r2c_evaluate.compare_controls(oracle, controls)
    if compared["passes_frozen_r2c_rule"]:
        return "RepresentationSufficientInterpolationOrSamplingLimited", compared
    return "RepresentationAndInterpolationBothLimited", compared


def run(arguments: argparse.Namespace) -> None:
    root = r2c.root_from_script(Path(__file__))
    rejection_bytes, rejection = validate_rejection(
        root, arguments.evaluation_report
    )
    (
        evaluation_manifest_bytes,
        evaluation_inputs,
        training_manifest_bytes,
        _,
        training_inputs,
    ) = load_inputs(
        root, arguments.evaluation_manifest, arguments.training_manifest
    )
    output, staging = r2c.prepare_output(
        root, arguments.output, "R2E failure diagnostic output"
    )
    temporary = staging / ".query-spectra.c64le"
    try:
        device = v1.configure_determinism()
        feature_shape = tuple(training_inputs["preflight"]["feature_shape"])
        total_tf = feature_shape[1] * feature_shape[2]
        arrays = v1.load_factorization_arrays(
            training_inputs["artifact_paths"], total_tf
        )
        rows = list(training_inputs["query_rows"])
        rows.sort(key=lambda row: row["row_index"])
        spectra, measured_tf, query_bytes = reference_spectra(
            rows, evaluation_inputs["references"], temporary
        )
        if measured_tf != total_tf:
            raise DiagnosticError("query/reference time-frequency shape changed")
        coefficients, projection = project_coefficients(spectra, arrays, device)
        oracle, frequency_diagnostics, prediction_grid_sha256 = cook_oracle(
            root,
            rows,
            evaluation_inputs["references"],
            spectra,
            coefficients,
            arrays,
            staging,
            device,
        )
        decision, compared = classify(oracle, rejection["controls"])
        candidate = rejection["candidate"]
        normalized_oracle = {
            "candidate_id": compared["candidate_id"],
            "normalized_metric_rows_sha256": compared[
                "normalized_metric_rows_sha256"
            ],
            "query_count": compared["query_count"],
            "aggregate": compared["aggregate"],
            "rows": compared["rows"],
            "primary_endpoint_comparison": compared[
                "primary_endpoint_comparison"
            ],
            "passes_frozen_r2c_rule": compared["passes_frozen_r2c_rule"],
        }
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "claim": (
                "POST_REJECT_QUERY_SEEING_REPRESENTATION_DIAGNOSTIC_ONLY / "
                "NO_TRAINING_SELECTION_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
            ),
            "revision": REVISION,
            "runner_sha256": r2c.sha256_file(Path(__file__).resolve(strict=True)),
            "r2e_evaluation_manifest_sha256": r2c.sha256_bytes(
                evaluation_manifest_bytes
            ),
            "r2e_evaluation_report_sha256": r2c.sha256_bytes(rejection_bytes),
            "r2e_training_manifest_sha256": r2c.sha256_bytes(
                training_manifest_bytes
            ),
            "projection_diagnostics": projection,
            "projection_frequency_diagnostics": frequency_diagnostics,
            "oracle_prediction_grid_sha256": prediction_grid_sha256,
            "rank96_query_projection_oracle": normalized_oracle,
            "rejected_r2e_candidate_aggregate": candidate["aggregate"],
            "oracle_delta_from_rejected_candidate": {
                endpoint: compared["aggregate"][endpoint]
                - candidate["aggregate"][endpoint]
                for endpoint in common.PRIMARY_ENDPOINTS
            },
            "query_audio_rows_read_after_r2e_rejection": len(rows),
            "query_audio_file_bytes_read_after_r2e_rejection": query_bytes,
            "method_holdout_or_shadow_bytes_read": 0,
            "training_or_candidate_selection_authorized": False,
            "quality_admission_or_runtime_authorized": False,
            "next_action": (
                "stop_opened_realimpact_split_and_acquire_a_new_denser_or_"
                "cross_object_corpus_before_any_new_field_family"
            ),
        }
        report_bytes = r2c.canonical_json(report)
        (staging / "diagnostic-report.json").write_bytes(report_bytes)
        temporary.unlink()
        r2c.publish_staging(staging, output)
    except BaseException:
        if temporary.exists():
            temporary.unlink()
        r2c.discard_staging(staging)
        raise
    r2c.emit_summary(
        {
            "output": str(output),
            "decision": report["decision"],
            "diagnostic_report_sha256": r2c.sha256_bytes(report_bytes),
            "projection_frobenius_nrmse": projection[
                "query_rank96_projection_frobenius_nrmse"
            ],
            "oracle_aggregate": compared["aggregate"],
            "oracle_endpoint_passes": {
                row["endpoint"]: row["strictly_below_all_controls"]
                for row in compared["primary_endpoint_comparison"]
            },
        },
        (
            "output",
            "decision",
            "diagnostic_report_sha256",
            "projection_frobenius_nrmse",
            "oracle_aggregate",
            "oracle_endpoint_passes",
        ),
    )


def main() -> None:
    run(parse_arguments())


if __name__ == "__main__":
    main()
