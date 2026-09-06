#!/usr/bin/env python3
"""Owning deterministic full-entry runner for the V26 M0b neural student."""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import physical_sound_v25_m0a_common as base
import physical_sound_v25_m0a_evaluate as evaluate
import physical_sound_v25_m0a_model as model_lib
import physical_sound_v25_m0a_train as inherited_train
import physical_sound_v26_m0b_common as contract
import physical_sound_v26_m0b_preprocess as preprocessing
import physical_sound_v26_m0b_surface as surface
import torch

REPORT_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.report.v1"
CONTROL_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.controls.v1"
FREEZE_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.freeze.v1"
HOLDOUT_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.method-holdout.v1"
REAL_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.disclosed-real.v1"
MAX_OUTPUT_BYTES = 256 * 1024 * 1024


def _mlflow_log(
    root: Path,
    manifest: dict[str, Any],
    report: dict[str, Any],
    step_metrics: tuple[dict[str, float], ...],
    artifact_hashes: dict[str, str],
) -> dict[str, str]:
    root.mkdir(parents=True, exist_ok=True)
    tracking_uri = root.resolve(strict=True).as_uri()
    if not tracking_uri.startswith("file://"):
        raise base.M0Error("M0b MLflow tracking URI must be local file://")
    os.environ["MLFLOW_ALLOW_FILE_STORE"] = "true"
    mlflow.autolog(disable=True)
    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment("nextengine-physical-sound-v26-m0b")
    with mlflow.start_run(run_name=contract.EXPERIMENT_ID) as active:
        mlflow.log_params(
            {
                "model_id": contract.MODEL_ID,
                "profile": manifest["profile"],
                "protocol_sha256": contract.PROTOCOL_SHA256,
                "implementation_root_sha256": manifest["implementation_root_sha256"],
                "seed": model_lib.SEED,
                "parameter_count": report["parameter_count"],
                "cpu_only": True,
            }
        )
        for item in step_metrics:
            mlflow.log_metric("loss", item["loss"], step=int(item["step"]))
        for name, value in sorted(artifact_hashes.items()):
            mlflow.log_param(f"sha256_{name.replace('.', '_')}", value)
        return {"tracking_uri": tracking_uri, "run_id": active.info.run_id}


def _validate_teacher_and_x0(
    manifest: dict[str, Any],
) -> tuple[dict[str, Any], bytes, bytes]:
    _, teacher_data = base.validate_ref(
        manifest["artifacts"]["teacher_evidence"],
        "M0b teacher evidence",
        base.MAX_JSON_BYTES,
    )
    teacher = base.parse_json_bytes(teacher_data, "M0b teacher evidence")
    if (
        teacher.get("schema") != base.TEACHER_SCHEMA
        or teacher.get("status") != "Validated"
        or teacher.get("implementation_sha256") != base.T0_IMPLEMENTATION_SHA256
        or teacher.get("model_training_authorized") is not False
        or teacher.get("real_material_authorized") is not False
        or teacher.get("runtime_authorized") is not False
    ):
        raise base.M0Error("M0b teacher evidence identity or authority changed")
    _, x0_data = base.validate_ref(
        manifest["artifacts"]["x0_lineage"],
        "M0b X0 lineage",
        base.MAX_JSON_BYTES,
    )
    x0 = base.parse_json_bytes(x0_data, "M0b X0 lineage")
    if (
        x0.get("schema") != base.X0_SCHEMA
        or x0.get("status") != "Validated"
        or x0.get("sealed_contact", {}).get("decoded_sample_count") != 0
        or x0.get("model_training_authorized") is not False
        or x0.get("real_material_admission_authorized") is not False
        or x0.get("runtime_authorized") is not False
    ):
        raise base.M0Error("M0b X0 lineage identity, seal or authority changed")
    return teacher, teacher_data, x0_data


def run(manifest_path: Path, output_path: Path) -> dict[str, Any]:
    manifest, manifest_bytes = contract.load_manifest(manifest_path)
    base.external_directory(output_path, "M0b output", must_exist=False)
    profile = base.execution_profile(manifest["profile"])
    surface_report = surface.official_shape_report(
        manifest["implementation_root_sha256"]
    )
    surface.validate_report(surface_report, manifest["implementation_root_sha256"])
    staging, output = base.prepare_output(output_path)
    try:
        artifacts = {
            "surface-query-report.json": base.write_artifact(
                staging,
                "surface-query-report.json",
                base.canonical_json(surface_report),
            )
        }
        combined, combined_bytes = contract.load_combined(manifest)
        teacher, teacher_data, x0_data = _validate_teacher_and_x0(manifest)
        contract.validate_combined_lineage_bindings(combined, manifest)
        teacher_root = base.reconstruct_teacher_artifact_root(
            Path(manifest["t0_root"]).resolve(strict=True),
            teacher,
            combined,
        )
        vault = base.EvidenceVault(combined)
        preprocessor = preprocessing.Preprocessor(manifest, combined, teacher, profile)
        vault.begin_training()
        synthetic_train = [
            preprocessor.synthetic(row) for row in vault.synthetic_train_context()
        ]
        synthetic_train_query = [
            preprocessor.synthetic(row) for row in vault.synthetic_train_query()
        ]
        transfer_context = [
            preprocessor.transfer(row)
            for row in vault.real_context("exact_real_transfer")
        ]
        recording_context = [
            preprocessor.recording(row)
            for row in vault.real_context("identified_real_recording")
        ]
        if len(transfer_context) != 3 or len(recording_context) != 2:
            raise base.M0Error("M0b disclosed real context shape changed")
        ridge = model_lib.fit_ridge([item.example for item in synthetic_train])
        candidate_a = inherited_train.train_variant(
            synthetic_train,
            transfer_context,
            recording_context,
            profile,
            "neural-full-v1",
        )
        candidate_b = inherited_train.train_variant(
            synthetic_train,
            transfer_context,
            recording_context,
            profile,
            "neural-full-v1",
        )
        weights_a = base.canonical_tensor_bytes(
            model_lib.model_tensors(candidate_a.model)
        )
        weights_b = base.canonical_tensor_bytes(
            model_lib.model_tensors(candidate_b.model)
        )
        train_predictions_a = inherited_train.predictions_bytes(
            candidate_a.model,
            [*synthetic_train, *synthetic_train_query],
            transfer_context,
        )
        train_predictions_b = inherited_train.predictions_bytes(
            candidate_b.model,
            [*synthetic_train, *synthetic_train_query],
            transfer_context,
        )
        if weights_a != weights_b or train_predictions_a != train_predictions_b:
            raise base.M0Error("M0b internal neural A/B canonical bytes differ")
        no_geometry = inherited_train.train_variant(
            synthetic_train,
            transfer_context,
            recording_context,
            profile,
            "neural-no-geometry-v1",
        )
        no_contact = inherited_train.train_variant(
            synthetic_train,
            transfer_context,
            recording_context,
            profile,
            "neural-no-contact-v1",
        )
        no_residual = inherited_train.train_variant(
            synthetic_train,
            transfer_context,
            recording_context,
            profile,
            "neural-no-residual-v1",
        )
        vault.finish_training()
        development = [
            preprocessor.synthetic(row) for row in vault.synthetic_development()
        ]
        calibration = [
            preprocessor.synthetic(row) for row in vault.synthetic_calibration()
        ]
        if not development or not calibration:
            raise base.M0Error("M0b synthetic development/calibration is empty")
        full_metrics = inherited_train.synthetic_metrics(
            candidate_a.model, ridge, development
        )
        geometry_metrics = inherited_train.synthetic_metrics(
            no_geometry.model,
            ridge,
            development,
            "neural-no-geometry-v1",
        )
        contact_metrics = inherited_train.synthetic_metrics(
            no_contact.model,
            ridge,
            development,
            "neural-no-contact-v1",
        )
        residual_metrics = inherited_train.synthetic_metrics(
            no_residual.model,
            ridge,
            development,
            "neural-no-residual-v1",
        )
        hard = inherited_train.hard_causal_report(
            candidate_a.model,
            [*synthetic_train, *development, *calibration],
        )
        parameter_count = model_lib.parameter_count(candidate_a.model)
        official_synthetic_pass = (
            all(
                full_metrics[f"ratio_{name}"] <= 0.90
                for name in ("frequency", "decay", "gain", "spectrum")
            )
            and geometry_metrics["neural_frequency"]
            >= full_metrics["neural_frequency"] * 1.05
            and contact_metrics["neural_gain"] >= full_metrics["neural_gain"] * 1.05
            and hard["maximum_render_peak"] < 0.95
            and hard["maximum_remesh_gain_absolute_difference"] <= 1.0e-5
            and hard["contact_gradients_finite_nonconstant"]
            and all(
                item["relative_error"] <= 0.10
                for item in hard["counterfactuals"].values()
            )
        )
        synthetic_pass = (
            official_synthetic_pass or profile.profile_id == "contract-fixture-v1"
        )
        weights_sha256 = base.sha256_bytes(weights_a)
        predictions = inherited_train.predictions_bytes(
            candidate_a.model,
            [*synthetic_train, *synthetic_train_query, *development, *calibration],
            transfer_context,
        )
        predictions_sha256 = base.sha256_bytes(predictions)
        source_hashes = {
            "manifest": base.sha256_bytes(manifest_bytes),
            "combined_manifest": base.sha256_bytes(combined_bytes),
            "teacher_evidence": base.sha256_bytes(teacher_data),
            "x0_lineage": base.sha256_bytes(x0_data),
            "protocol": contract.PROTOCOL_SHA256,
            "implementation": manifest["implementation_root_sha256"],
        }
        preprocess_record = preprocessor.manifest_record(source_hashes)
        control_report = {
            "schema": CONTROL_SCHEMA,
            "status": "Validated" if synthetic_pass else "Rejected",
            "profile": profile.profile_id,
            "ridge_lambda": model_lib.RIDGE_LAMBDA,
            "full": full_metrics,
            "no_geometry": geometry_metrics,
            "no_contact": contact_metrics,
            "no_residual": residual_metrics,
            "hard_causal": hard,
            "official_synthetic_gates_pass": official_synthetic_pass,
            "contract_fixture_has_no_quality_claim": profile.profile_id
            == "contract-fixture-v1",
        }
        artifacts["preprocess-manifest.json"] = base.write_artifact(
            staging,
            "preprocess-manifest.json",
            base.canonical_json(preprocess_record),
        )
        artifacts["control-report.json"] = base.write_artifact(
            staging,
            "control-report.json",
            base.canonical_json(control_report),
        )
        artifacts["candidate-weights.bin"] = base.write_artifact(
            staging, "candidate-weights.bin", weights_a
        )
        artifacts["candidate-predictions.bin"] = base.write_artifact(
            staging,
            "candidate-predictions.bin",
            predictions,
        )
        decision = "REPRESENTATION_REJECT"
        if synthetic_pass:
            freeze = {
                "schema": FREEZE_SCHEMA,
                "status": "Frozen",
                "model_id": contract.MODEL_ID,
                "profile": profile.profile_id,
                "seed": model_lib.SEED,
                "final_step": profile.synthetic_steps + profile.real_steps,
                "parameter_count": parameter_count,
                "weights_sha256": weights_sha256,
                "predictions_sha256": predictions_sha256,
                "protocol_sha256": contract.PROTOCOL_SHA256,
                "implementation_root_sha256": manifest["implementation_root_sha256"],
                "method_holdout_opened": False,
                "admission_shadow_opened": False,
            }
            artifacts["candidate-freeze.json"] = base.write_artifact(
                staging,
                "candidate-freeze.json",
                base.canonical_json(freeze),
            )
            vault.freeze_candidate(weights_sha256)
            holdout = [
                preprocessor.synthetic(row) for row in vault.synthetic_method_holdout()
            ]
            holdout_metrics = inherited_train.synthetic_metrics(
                candidate_a.model, ridge, holdout
            )
            official_holdout_pass = all(
                holdout_metrics[f"ratio_{name}"] <= 0.95
                for name in ("frequency", "decay", "gain", "spectrum")
            )
            holdout_pass = (
                official_holdout_pass or profile.profile_id == "contract-fixture-v1"
            )
            holdout_report = {
                "schema": HOLDOUT_SCHEMA,
                "status": "Pass" if holdout_pass else "Reject",
                "candidate_weights_sha256": weights_sha256,
                "metrics": holdout_metrics,
                "official_gates_pass": official_holdout_pass,
                "contract_fixture_has_no_quality_claim": profile.profile_id
                == "contract-fixture-v1",
                "admission_shadow_opened": False,
            }
            artifacts["method-holdout-report.json"] = base.write_artifact(
                staging,
                "method-holdout-report.json",
                base.canonical_json(holdout_report),
            )
            if holdout_pass:
                transfer_queries = [
                    preprocessor.transfer(row)
                    for row in vault.real_queries("exact_real_transfer")
                ]
                recording_queries = [
                    preprocessor.recording(row)
                    for row in vault.real_queries("identified_real_recording")
                ]
                if len(transfer_queries) != 1 or len(recording_queries) != 1:
                    raise base.M0Error("M0b disclosed real query shape changed")
                real_metrics = evaluate.disclosed_real_metrics(
                    candidate_a.model,
                    ridge,
                    transfer_context,
                    transfer_queries[0],
                    recording_context,
                    recording_queries[0],
                )
                official_real_pass = (
                    real_metrics["full_to_ridge_ratio"] <= 0.95
                    and real_metrics["full_to_nearest_ratio"] <= 0.95
                    and real_metrics["improved_component_count"] >= 2
                    and real_metrics["objectfolder_candidate_distance"]
                    <= real_metrics["objectfolder_nearest_distance"]
                )
                real_pass = (
                    official_real_pass or profile.profile_id == "contract-fixture-v1"
                )
                vault.finish_real_queries()
                real_report = {
                    "schema": REAL_SCHEMA,
                    "status": "Pass" if real_pass else "DomainGapReject",
                    "candidate_weights_sha256": weights_sha256,
                    "metrics": real_metrics,
                    "official_gates_pass": official_real_pass,
                    "contract_fixture_has_no_quality_claim": profile.profile_id
                    == "contract-fixture-v1",
                    "realimpact_2407_opened": False,
                    "admission_shadow_opened": False,
                }
                artifacts["disclosed-real-report.json"] = base.write_artifact(
                    staging,
                    "disclosed-real-report.json",
                    base.canonical_json(real_report),
                )
                decision = "PASS" if real_pass else "DOMAIN_GAP_REJECT"
            else:
                decision = "METHOD_HOLDOUT_REJECT"
        output_bytes = sum(item["byte_count"] for item in artifacts.values())
        if output_bytes > MAX_OUTPUT_BYTES:
            raise base.M0Error("M0b canonical output exceeds 256 MiB")
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "profile": profile.profile_id,
            "experiment_id": contract.EXPERIMENT_ID,
            "model_id": contract.MODEL_ID,
            "protocol_sha256": contract.PROTOCOL_SHA256,
            "implementation_root_sha256": manifest["implementation_root_sha256"],
            "parameter_count": parameter_count,
            "seed": model_lib.SEED,
            "final_step": profile.synthetic_steps + profile.real_steps,
            "environment": {
                "python": platform.python_version(),
                "numpy": np.__version__,
                "torch": torch.__version__,
                "mlflow": mlflow.__version__,
                "cpu_only": True,
                "threads": 1,
            },
            "teacher_aggregate_verification": teacher_root,
            "surface_query_report_sha256": artifacts["surface-query-report.json"][
                "sha256"
            ],
            "internal_candidate_ab_byte_exact": True,
            "canonical_payload_bytes_before_report": output_bytes,
            "access_log": vault.access_log,
            "admission_shadow_opened": False,
            "realimpact_2407_opened": False,
            "model_training_executed": True,
            "real_material_admission_authorized": False,
            "runtime_authorized": False,
            "mlflow": {
                "tracking_uri_scheme": "file",
                "diagnostic_only": True,
                "autolog_disabled": True,
                "registry_disabled": True,
                "serving_disabled": True,
                "volatile_run_id_excluded": True,
            },
        }
        artifacts["report.json"] = base.write_artifact(
            staging,
            "report.json",
            base.canonical_json(report),
        )
        artifact_hashes = {name: item["sha256"] for name, item in artifacts.items()}
        _mlflow_log(
            Path(manifest["mlflow_root"]),
            manifest,
            report,
            candidate_a.step_metrics,
            artifact_hashes,
        )
        base.publish_output(staging, output)
        return report
    except BaseException:
        base.abandon_output(staging)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.manifest, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable owning CLI boundary
        print(f"physical-sound-v26-m0b: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
