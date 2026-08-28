#!/usr/bin/env python3
"""Run a frozen read-only multi-output counterfactual on Ceramic Cup."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from typing import Any

import numpy as np

import physical_sound_multioutput_decay_control as multioutput


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-multioutput-counterfactual.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-multioutput-counterfactual-preflight.report.v1",
    "analyze": "nextengine.experimental-realimpact-multioutput-counterfactual.report.v1",
}
STUDY_ID = "physical-sound-realimpact-observation-first-discriminator"
REVISION = "ceramic-cup-fixed-impact-all-microphones-spatial-power-v1"
OBJECT_ID = "78_CeramicCup"
ROW_COUNT = 600
SAMPLE_COUNT = 208_895
DECODED_BYTES = 501_348_000
ANALYSIS_ROWS = list(range(15))
REFERENCE_ROW = 7
PARENT_REPORT_SHA256 = (
    "56591bb82ab50470296013c432c57e2b31b5addbae002ccbc833d595f3823fd9"
)
DECODE_REPORT_SHA256 = (
    "9f1c23118ac2d451bd423d2f2a8deb604ad5ccd085900501682b6e31e71adaf0"
)
DIAGNOSTIC_REPORT_SHA256 = (
    "47b578ac3706743778fba753f5260594c75a9fff46c4cba6761d7f8def8d2603"
)
DECODED_SHA256 = (
    "3405843a7a4bfe825dedf9e4406bce2898b2270de1bb273eb8e91a4a247be6ca"
)
SYNTHETIC_REPORT_SHA256 = (
    "a099f50d017e4f86ac7e2519663c457622e755ddd4a1969dcbd629fc0e0e8bea"
)
EXTRACTOR_PYTHON_SHA256 = (
    "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
)
MULTIOUTPUT_PYTHON_SHA256 = (
    "f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e"
)

THRESHOLDS = {
    "minimum_selected_modes": 6,
    "minimum_persistent_mode_recall": 0.50,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_decaying_mode_fraction": 0.50,
    "maximum_median_tail_prediction_rmse_db": 24.0,
    "minimum_decaying_fraction_improvement": 0.25,
}


class CounterfactualError(RuntimeError):
    """The frozen counterfactual contract or immutable lineage failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise CounterfactualError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise CounterfactualError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise CounterfactualError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "opened_development",
            "impact_ordinal": 0,
        },
        "parents": {
            "rejected_observation_report": {
                "path": "observation-analysis-a/report.json",
                "sha256": PARENT_REPORT_SHA256,
            },
            "decode_report": {
                "path": "observation-decode/report.json",
                "sha256": DECODE_REPORT_SHA256,
            },
            "causal_diagnostic_report": {
                "path": "observation-diagnostic-analysis-a/report.json",
                "sha256": DIAGNOSTIC_REPORT_SHA256,
            },
            "decoded_block": {
                "path": "observation-decode/ceramic-cup-impact000-rows000-599.f32le",
                "sha256": DECODED_SHA256,
                "bytes": DECODED_BYTES,
                "shape": [ROW_COUNT, SAMPLE_COUNT],
                "dtype": "<f4",
            },
            "synthetic_control_report": {
                "path": "../ps2-multioutput-decay-control-v1/run-a/report.json",
                "sha256": SYNTHETIC_REPORT_SHA256,
            },
        },
        "fixed_input": {
            "rows": ANALYSIS_ROWS,
            "reference_row": REFERENCE_ROW,
            "condition": {
                "angle_degrees": 0,
                "distance_millimetres": 0,
                "microphone_ids": ANALYSIS_ROWS,
            },
            "selection_rule": "all 15 microphones at the pre-existing angle/distance 0/0 and impact ordinal 0; no value-dependent row selection",
        },
        "candidate": {
            "id": "spatial-modal-power-15-v1",
            "source": {
                "path": "lab/scripts/physical_sound_multioutput_decay_control.py",
                "sha256": MULTIOUTPUT_PYTHON_SHA256,
            },
            "operation": "sum per-microphone FFT power before unchanged V2 peak selection and three-bin decay tracking",
            "no_cross_channel_phase_fit": True,
        },
        "reference_extractor": {
            "profile_id": "injective-modal-16-fft65536-v2",
            "source": {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": EXTRACTOR_PYTHON_SHA256,
            },
        },
        "thresholds": THRESHOLDS,
        "data_policy": {
            "offline_existing_decoded_block_only": True,
            "network_or_additional_payload_allowed": False,
            "row_selection_threshold_tuning_or_denoising_allowed": False,
            "physics_solver_allowed": False,
            "planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": "publish support or rejection once; do not select rows, tune, denoise, fetch, run physics or open Planter",
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise CounterfactualError("counterfactual manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise CounterfactualError(f"parse counterfactual manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise CounterfactualError("counterfactual manifest contract changed")
    return data, manifest


def resolve_report(base: Path, ref: dict[str, Any], label: str) -> tuple[bytes, dict[str, Any]]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise CounterfactualError(f"{label} escapes external physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise CounterfactualError(f"{label} identity changed")
    return data, json.loads(data)


def decoded_block(base: Path, ref: dict[str, Any], hash_payload: bool) -> Path:
    path = (base / ref["path"]).resolve(strict=True)
    if (
        not path.is_file()
        or not path.is_relative_to(base.parent)
        or path.stat().st_size != ref["bytes"]
    ):
        raise CounterfactualError("decoded block identity or size changed")
    if hash_payload and sha256_file(path) != ref["sha256"]:
        raise CounterfactualError("decoded block hash changed")
    return path


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [
        manifest["candidate"]["source"],
        manifest["reference_extractor"]["source"],
    ]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        if not path.is_relative_to(root) or sha256_file(path) != ref["sha256"]:
            raise CounterfactualError(f"bound source changed: {ref['path']}")
        reports.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return reports


def validate_parents(
    base: Path, manifest: dict[str, Any], hash_payload: bool
) -> tuple[dict[str, Any], Path, dict[str, Any]]:
    refs = manifest["parents"]
    parent_bytes, parent = resolve_report(
        base, refs["rejected_observation_report"], "rejected observation report"
    )
    decode_bytes, decode = resolve_report(base, refs["decode_report"], "decode report")
    diagnostic_bytes, diagnostic = resolve_report(
        base, refs["causal_diagnostic_report"], "causal diagnostic report"
    )
    synthetic_bytes, synthetic = resolve_report(
        base, refs["synthetic_control_report"], "synthetic control report"
    )
    block = decoded_block(base, refs["decoded_block"], hash_payload)
    if (
        parent.get("decision") != "CeramicCupObservationRejected"
        or parent.get("reference_row") != REFERENCE_ROW
        or parent.get("observation_gate", {}).get("passed") is not False
        or parent.get("observation_analysis", {}).get("decaying_mode_fraction") != 0.25
        or parent.get("decoded_sha256") != DECODED_SHA256
        or decode.get("decision") != "CeramicCupImpactZeroObservationDecoded"
        or decode.get("decoded_sha256") != DECODED_SHA256
        or decode.get("decoded_bytes") != DECODED_BYTES
        or decode.get("row_count") != ROW_COUNT
        or decode.get("sample_count") != SAMPLE_COUNT
        or diagnostic.get("decision") != "CeramicCupSharedDecayMismatchSupported"
        or diagnostic.get("parents", {}).get("decoded_sha256") != DECODED_SHA256
        or synthetic.get("decision") != "MultiOutputSpatialDecayControlSupported"
        or synthetic.get("gate", {}).get("passed") is not True
        or synthetic.get("network_requests") != 0
        or synthetic.get("real_payload_bytes_read") != 0
    ):
        raise CounterfactualError("counterfactual parent lineage changed")
    return (
        {
            "rejected_observation_report_sha256": sha256_bytes(parent_bytes),
            "decode_report_sha256": sha256_bytes(decode_bytes),
            "causal_diagnostic_report_sha256": sha256_bytes(diagnostic_bytes),
            "synthetic_control_report_sha256": sha256_bytes(synthetic_bytes),
            "decoded_sha256": DECODED_SHA256,
            "decoded_bytes": block.stat().st_size,
            "decoded_payload_rehashed": hash_payload,
        },
        block,
        parent["observation_analysis"],
    )


def gate_check(metric: str, observed: float, relation: str, threshold: float) -> dict[str, Any]:
    passed = observed >= threshold if relation == ">=" else observed <= threshold
    return {
        "metric": metric,
        "observed": observed,
        "relation": relation,
        "threshold": threshold,
        "passed": passed,
    }


def evaluate_candidate(analysis: dict[str, Any]) -> dict[str, Any]:
    checks = [
        gate_check(
            "selected_mode_count",
            analysis["selected_mode_count"],
            ">=",
            THRESHOLDS["minimum_selected_modes"],
        ),
        gate_check(
            "persistent_mode_recall",
            analysis["persistent_mode_recall"],
            ">=",
            THRESHOLDS["minimum_persistent_mode_recall"],
        ),
        gate_check(
            "median_frequency_error_cents",
            analysis["median_frequency_error_cents"],
            "<=",
            THRESHOLDS["maximum_median_frequency_error_cents"],
        ),
        gate_check(
            "decaying_mode_fraction",
            analysis["decaying_mode_fraction"],
            ">=",
            THRESHOLDS["minimum_decaying_mode_fraction"],
        ),
        gate_check(
            "median_tail_prediction_rmse_db",
            analysis["median_tail_prediction_rmse_db"],
            "<=",
            THRESHOLDS["maximum_median_tail_prediction_rmse_db"],
        ),
    ]
    return {"passed": all(check["passed"] for check in checks), "checks": checks}


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "analysis_rows": ANALYSIS_ROWS,
        "reference_row": REFERENCE_ROW,
        "network_requests": 0,
        "additional_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents, _, reference = validate_parents(base, manifest, hash_payload=False)
    sources = validate_sources(root, manifest)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "CeramicCupMultiOutputCounterfactualFrozen",
        "claim": "READ_ONLY_FIXED_INPUT_PREFLIGHT_ONLY / NO_REAL_PAYLOAD_BYTES_READ_OR_ADMISSION_PHYSICS_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "existing_real_payload_bytes_read": 0,
        "parents": parents,
        "bound_sources": sources,
        "fixed_input": manifest["fixed_input"],
        "candidate": manifest["candidate"],
        "reference_decaying_mode_fraction": reference["decaying_mode_fraction"],
        "thresholds": THRESHOLDS,
        "next_action": "commit this preflight, then execute the fixed read-only counterfactual twice",
    }


def analyze(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents, block, reference = validate_parents(base, manifest, hash_payload=True)
    validate_sources(root, manifest)
    values = np.memmap(block, mode="r", dtype="<f4", shape=(ROW_COUNT, SAMPLE_COUNT))
    channels = np.asarray(values[ANALYSIS_ROWS], dtype=np.float64)
    spatial = multioutput.analyze_spatial(channels)
    candidate_gate = evaluate_candidate(spatial)
    improvement = spatial["decaying_mode_fraction"] - reference["decaying_mode_fraction"]
    improvement_gate = gate_check(
        "decaying_fraction_improvement",
        improvement,
        ">=",
        THRESHOLDS["minimum_decaying_fraction_improvement"],
    )
    passed = candidate_gate["passed"] and improvement_gate["passed"]
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": (
            "CeramicCupMultiOutputCounterfactualSupported"
            if passed
            else "CeramicCupMultiOutputCounterfactualRejected"
        ),
        "claim": "READ_ONLY_FIXED_INPUT_MULTI_OUTPUT_COUNTERFACTUAL_ONLY / NO_ADMISSION_PHYSICS_QUALITY_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "existing_payload_bytes_hashed": DECODED_BYTES,
        "existing_payload_bytes_analyzed": len(ANALYSIS_ROWS) * SAMPLE_COUNT * 4,
        "parents": parents,
        "candidate_analysis": spatial,
        "candidate_gate": candidate_gate,
        "reference_analysis": reference,
        "comparison_gate": improvement_gate,
        "gate": {"passed": passed},
        "next_action": (
            "interpret the observation-statistic result without admission or mechanics credit"
            if passed
            else "reject this counterfactual; do not tune, select rows, fetch, run physics or open Planter"
        ),
    }


def publish(output: Path, report: dict[str, Any]) -> bytes:
    report_bytes = canonical_json(report)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report_bytes


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = external_file(root, arguments.manifest, "manifest")
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    report = (
        preflight(root, base, manifest_bytes, manifest, runner_sha256)
        if arguments.stage == "preflight"
        else analyze(root, base, manifest_bytes, manifest, runner_sha256)
    )
    report_bytes = publish(output, report)
    print(f"Ceramic Cup multi-output counterfactual {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("additional payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
