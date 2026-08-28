#!/usr/bin/env python3
"""Run a frozen offline listener-axis diagnostic on Ceramic Cup observations."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from typing import Any

import numpy as np

import physical_sound_realimpact_pitcher_calibration as extractor


MANIFEST_SCHEMA = "nextengine.experimental-realimpact-observation-diagnostic.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-observation-diagnostic-preflight.report.v1",
    "analyze": "nextengine.experimental-realimpact-observation-diagnostic.report.v1",
}
STUDY_ID = "physical-sound-realimpact-observation-first-discriminator"
REVISION = "ceramic-cup-fixed-listener-axes-v1"
OBJECT_ID = "78_CeramicCup"
ROW_COUNT = 600
SAMPLE_COUNT = 208_895
DECODED_BYTES = 501_348_000
REFERENCE_ROW = 7
PARENT_REPORT_SHA256 = "56591bb82ab50470296013c432c57e2b31b5addbae002ccbc833d595f3823fd9"
DECODE_REPORT_SHA256 = "9f1c23118ac2d451bd423d2f2a8deb604ad5ccd085900501682b6e31e71adaf0"
DECODED_SHA256 = "3405843a7a4bfe825dedf9e4406bce2898b2270de1bb273eb8e91a4a247be6ca"
EXTRACTOR_PYTHON_SHA256 = "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
TRANSFER_CALIBRATION_SHA256 = "2624656ebefbdf20d210ddef3138824219ab8044435dc5440365908a720cbf80"
TRANSFER_DSP_SHA256 = "131bbf42d01e633b6cac0c4e0340178f3f5a0a83324bb64630d8423da8179ca4"
FIXTURE_REPORT_SHA256 = "2e3db2d3f231d69b67b2a3a094f83fbc7ed10144a02f4c424d75bbbb2f85572f"
FIXTURE_SAMPLE_SHA256 = "c8316f81da79b9823cbeaf1c0c49ee1708d2112fca8b4905a792546fd3f40477"

HEIGHT_ROWS = list(range(15))
ANGLE_ROWS = [7 + 60 * index for index in range(10)]
DISTANCE_ROWS = [7 + 15 * index for index in range(4)]
ANALYSIS_ROWS = sorted(set(HEIGHT_ROWS + ANGLE_ROWS + DISTANCE_ROWS))
ANGLE_VALUES = [0, 20, 40, 60, 80, 100, 120, 140, 160, 180]
DISTANCE_VALUES = [0, 333, 666, 1_000]

THRESHOLDS = {
    "minimum_selected_modes": 6,
    "minimum_persistent_mode_recall": 0.50,
    "maximum_median_frequency_error_cents": 40.0,
    "minimum_decaying_mode_fraction": 0.50,
    "maximum_median_tail_prediction_rmse_db": 24.0,
}
CLASSIFICATION = {
    "shared_minimum_failed_fraction": 0.80,
    "shared_minimum_each_axis_failed_fraction": 0.50,
    "listener_local_minimum_other_pass_fraction": 0.80,
    "low_frequency_boundary_hz": 500.0,
    "low_frequency_maximum_decay_fraction_ratio": 0.50,
    "minimum_modes_per_frequency_band": 16,
}


class DiagnosticError(RuntimeError):
    """The frozen diagnostic contract or immutable parent lineage failed."""


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
        raise DiagnosticError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise DiagnosticError(f"output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise DiagnosticError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "object": {"dataset_object_id": OBJECT_ID, "role": "opened_development"},
        "parents": {
            "rejected_observation_report": {
                "path": "observation-analysis-a/report.json",
                "sha256": PARENT_REPORT_SHA256,
            },
            "decode_report": {
                "path": "observation-decode/report.json",
                "sha256": DECODE_REPORT_SHA256,
            },
            "decoded_block": {
                "path": "observation-decode/ceramic-cup-impact000-rows000-599.f32le",
                "sha256": DECODED_SHA256,
                "bytes": DECODED_BYTES,
                "shape": [ROW_COUNT, SAMPLE_COUNT],
                "dtype": "<f4",
            },
        },
        "axes": {
            "height_rows": HEIGHT_ROWS,
            "angle_rows": ANGLE_ROWS,
            "distance_rows": DISTANCE_ROWS,
            "analysis_rows": ANALYSIS_ROWS,
            "analysis_row_count": len(ANALYSIS_ROWS),
            "reference_row": REFERENCE_ROW,
            "selection_rule": "all microphones at angle/distance 0/0; microphone 7 at all angles and distance 0; microphone 7 at angle 0 and all distances",
        },
        "extractor": {
            "profile_id": "injective-modal-16-fft65536-v2",
            "sample_rate_hz": 48_000,
            "python_source": {
                "path": "lab/scripts/physical_sound_realimpact_pitcher_calibration.py",
                "sha256": EXTRACTOR_PYTHON_SHA256,
            },
            "rust_sources": [
                {
                    "path": "tools/xtask/src/physical_sound_registry_command/transfer_calibration.rs",
                    "sha256": TRANSFER_CALIBRATION_SHA256,
                },
                {
                    "path": "tools/xtask/src/physical_sound_registry_command/transfer_calibration/dsp.rs",
                    "sha256": TRANSFER_DSP_SHA256,
                },
            ],
            "rust_fixture": {
                "report_path": "../ps2-realimpact-geometry-spatial-transfer-v1/extractor-parity-fixture-a/report.json",
                "report_sha256": FIXTURE_REPORT_SHA256,
                "sample_path": "../ps2-realimpact-geometry-spatial-transfer-v1/extractor-parity-fixture-a/fixture.f64le",
                "sample_sha256": FIXTURE_SAMPLE_SHA256,
            },
            "thresholds": THRESHOLDS,
        },
        "classification": CLASSIFICATION,
        "data_policy": {
            "offline_existing_decoded_rows_only": True,
            "network_or_additional_payload_allowed": False,
            "listener_selection_or_threshold_tuning_allowed": False,
            "denoising_or_filtering_allowed": False,
            "physics_solver_allowed": False,
            "planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": "publish listener-local, shared, or mixed observation failure; do not select a passing row, tune, denoise, run physics, fetch another object or open Planter",
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise DiagnosticError("diagnostic manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiagnosticError(f"parse diagnostic manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise DiagnosticError("diagnostic manifest contract changed")
    return data, manifest


def resolve_reference(base: Path, ref: dict[str, Any], label: str) -> tuple[Path, bytes]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise DiagnosticError(f"{label} escapes external physical-sound store")
    if "bytes" in ref:
        if path.stat().st_size != ref["bytes"] or sha256_file(path) != ref["sha256"]:
            raise DiagnosticError(f"{label} identity changed")
        return path, b""
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise DiagnosticError(f"{label} identity changed")
    return path, data


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    refs = [manifest["extractor"]["python_source"], *manifest["extractor"]["rust_sources"]]
    reports = []
    for ref in refs:
        path = (root / ref["path"]).resolve(strict=True)
        actual = sha256_file(path)
        if not path.is_relative_to(root) or actual != ref["sha256"]:
            raise DiagnosticError(f"bound extractor source changed: {ref['path']}")
        reports.append({"path": ref["path"], "sha256": actual, "bytes": path.stat().st_size})
    return reports


def fixture_directory(base: Path, manifest: dict[str, Any]) -> Path:
    ref = manifest["extractor"]["rust_fixture"]
    report = (base / ref["report_path"]).resolve(strict=True)
    sample = (base / ref["sample_path"]).resolve(strict=True)
    if (
        not report.is_relative_to(base.parent)
        or not sample.is_relative_to(base.parent)
        or sha256_file(report) != ref["report_sha256"]
        or sha256_file(sample) != ref["sample_sha256"]
        or report.parent != sample.parent
    ):
        raise DiagnosticError("extractor fixture identity changed")
    return report.parent


def validate_parents(base: Path, manifest: dict[str, Any]) -> tuple[dict[str, Any], Path]:
    refs = manifest["parents"]
    _, parent_bytes = resolve_reference(base, refs["rejected_observation_report"], "parent report")
    _, decode_bytes = resolve_reference(base, refs["decode_report"], "decode report")
    block, _ = resolve_reference(base, refs["decoded_block"], "decoded block")
    parent = json.loads(parent_bytes)
    decode = json.loads(decode_bytes)
    if (
        parent.get("schema") != "nextengine.experimental-realimpact-observation-admission.report.v1"
        or parent.get("decision") != "CeramicCupObservationRejected"
        or parent.get("observation_gate", {}).get("passed") is not False
        or parent.get("decoded_sha256") != DECODED_SHA256
        or parent.get("network_requests") != 0
        or parent.get("planter_payload_bytes_read") != 0
        or decode.get("schema") != "nextengine.experimental-realimpact-observation-decode.report.v1"
        or decode.get("decision") != "CeramicCupImpactZeroObservationDecoded"
        or decode.get("decoded_sha256") != DECODED_SHA256
        or decode.get("decoded_bytes") != DECODED_BYTES
        or decode.get("row_count") != ROW_COUNT
        or decode.get("sample_count") != SAMPLE_COUNT
        or decode.get("network_requests") != 0
        or decode.get("planter_payload_bytes_read") != 0
    ):
        raise DiagnosticError("observation parent lineage changed")
    return {
        "rejected_observation_report_sha256": sha256_bytes(parent_bytes),
        "decode_report_sha256": sha256_bytes(decode_bytes),
        "decoded_sha256": sha256_file(block),
        "decoded_bytes": block.stat().st_size,
    }, block


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "analysis_rows": ANALYSIS_ROWS,
        "network_requests": 0,
        "additional_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def row_condition(row: int) -> dict[str, int]:
    condition = row // 15
    return {
        "row": row,
        "angle_degrees": ANGLE_VALUES[condition // 4],
        "distance_millimetres": DISTANCE_VALUES[condition % 4],
        "microphone_id": row % 15,
    }


def gate_check(metric: str, observed: float, relation: str, threshold: float) -> dict[str, Any]:
    passed = observed >= threshold if relation == ">=" else observed <= threshold
    return {"metric": metric, "observed": observed, "relation": relation, "threshold": threshold, "passed": passed}


def evaluate_gate(analysis: dict[str, Any]) -> dict[str, Any]:
    checks = [
        gate_check("selected_mode_count", analysis["selected_mode_count"], ">=", THRESHOLDS["minimum_selected_modes"]),
        gate_check("persistent_mode_recall", analysis["persistent_mode_recall"], ">=", THRESHOLDS["minimum_persistent_mode_recall"]),
        gate_check("median_frequency_error_cents", analysis["median_frequency_error_cents"], "<=", THRESHOLDS["maximum_median_frequency_error_cents"]),
        gate_check("decaying_mode_fraction", analysis["decaying_mode_fraction"], ">=", THRESHOLDS["minimum_decaying_mode_fraction"]),
        gate_check("median_tail_prediction_rmse_db", analysis["median_tail_prediction_rmse_db"], "<=", THRESHOLDS["maximum_median_tail_prediction_rmse_db"]),
    ]
    return {"passed": all(check["passed"] for check in checks), "checks": checks}


def summarize_rows(rows: list[dict[str, Any]]) -> dict[str, Any]:
    failed = sum(not row["gate"]["passed"] for row in rows)
    decays = [row["analysis"]["decaying_mode_fraction"] for row in rows]
    below = [row["diagnostics"]["selected_modes_below_500_hz"] for row in rows]
    return {
        "row_count": len(rows),
        "passed_rows": len(rows) - failed,
        "failed_rows": failed,
        "failed_fraction": failed / len(rows),
        "decaying_mode_fraction": {
            "minimum": min(decays),
            "median": float(np.median(decays)),
            "maximum": max(decays),
        },
        "selected_modes_below_500_hz": {
            "minimum": min(below),
            "median": float(np.median(below)),
            "maximum": max(below),
        },
    }


def frequency_band_summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
    bands = {"below_500_hz": [], "at_or_above_500_hz": []}
    for row in rows:
        for mode in row["analysis"]["modes"]:
            key = "below_500_hz" if mode["frequency_hz"] < 500.0 else "at_or_above_500_hz"
            bands[key].append(mode)
    report = {}
    for key, modes in bands.items():
        fit_decaying = sum(mode["fit_decay_db_per_second"] < -1.0 for mode in modes)
        tail_decaying = sum(mode["tail_decay_db_per_second"] < -1.0 for mode in modes)
        rising_then_tail_decaying = sum(
            mode["fit_decay_db_per_second"] > 0.0 and mode["tail_decay_db_per_second"] < -1.0
            for mode in modes
        )
        report[key] = {
            "mode_count": len(modes),
            "fit_decaying_fraction": fit_decaying / len(modes) if modes else None,
            "tail_decaying_fraction": tail_decaying / len(modes) if modes else None,
            "rising_fit_then_decaying_tail_fraction": rising_then_tail_decaying / len(modes) if modes else None,
        }
    low = report["below_500_hz"]
    high = report["at_or_above_500_hz"]
    eligible = (
        low["mode_count"] >= CLASSIFICATION["minimum_modes_per_frequency_band"]
        and high["mode_count"] >= CLASSIFICATION["minimum_modes_per_frequency_band"]
        and high["fit_decaying_fraction"] > 0.0
    )
    ratio = low["fit_decaying_fraction"] / high["fit_decaying_fraction"] if eligible else None
    report["low_frequency_decay_association"] = {
        "eligible": eligible,
        "decay_fraction_ratio_low_over_high": ratio,
        "maximum_ratio": CLASSIFICATION["low_frequency_maximum_decay_fraction_ratio"],
        "observed": eligible and ratio <= CLASSIFICATION["low_frequency_maximum_decay_fraction_ratio"],
    }
    return report


def analyze_rows(block: Path) -> list[dict[str, Any]]:
    values = np.memmap(block, mode="r", dtype="<f4", shape=(ROW_COUNT, SAMPLE_COUNT))
    reports = []
    for row in ANALYSIS_ROWS:
        analysis = extractor.analyze_v2(np.asarray(values[row], dtype=np.float64))
        reports.append(
            {
                "condition": row_condition(row),
                "axes": [
                    name
                    for name, members in [
                        ("height", HEIGHT_ROWS),
                        ("angle", ANGLE_ROWS),
                        ("distance", DISTANCE_ROWS),
                    ]
                    if row in members
                ],
                "analysis": analysis,
                "gate": evaluate_gate(analysis),
                "diagnostics": {
                    "selected_modes_below_500_hz": sum(mode["frequency_hz"] < 500.0 for mode in analysis["modes"]),
                    "selected_modes_at_or_above_500_hz": sum(mode["frequency_hz"] >= 500.0 for mode in analysis["modes"]),
                },
            }
        )
    return reports


def classify(rows: list[dict[str, Any]], axes: dict[str, Any], bands: dict[str, Any]) -> dict[str, Any]:
    overall = summarize_rows(rows)
    reference = next(row for row in rows if row["condition"]["row"] == REFERENCE_ROW)
    other = [row for row in rows if row["condition"]["row"] != REFERENCE_ROW]
    other_pass_fraction = sum(row["gate"]["passed"] for row in other) / len(other)
    shared = (
        overall["failed_fraction"] >= CLASSIFICATION["shared_minimum_failed_fraction"]
        and all(
            axes[name]["failed_fraction"] >= CLASSIFICATION["shared_minimum_each_axis_failed_fraction"]
            for name in ["height", "angle", "distance"]
        )
    )
    listener_local = (
        not reference["gate"]["passed"]
        and other_pass_fraction >= CLASSIFICATION["listener_local_minimum_other_pass_fraction"]
    )
    low_frequency = bands["low_frequency_decay_association"]["observed"]
    if listener_local:
        decision = "CeramicCupReferenceListenerFailureSupported"
    elif shared and low_frequency:
        decision = "CeramicCupSharedLowFrequencyDecayMismatchSupported"
    elif shared:
        decision = "CeramicCupSharedDecayMismatchSupported"
    else:
        decision = "CeramicCupMixedSpatialObservationFailure"
    return {
        "decision": decision,
        "shared_failure_supported": shared,
        "reference_listener_local_failure_supported": listener_local,
        "low_frequency_decay_association_observed": low_frequency,
        "other_row_pass_fraction": other_pass_fraction,
        "criteria": CLASSIFICATION,
    }


def preflight(
    root: Path, base: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    parents, _ = validate_parents(base, manifest)
    sources = validate_sources(root, manifest)
    fixture = fixture_directory(base, manifest)
    fixture_report, parity = extractor.validate_fixture(fixture)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "CeramicCupObservationDiagnosticFrozen",
        "claim": "OFFLINE_FIXED_LISTENER_AXES_PREFLIGHT_ONLY / NO_SELECTION_TUNING_DENOISING_PHYSICS_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "bound_sources": sources,
        "fixture_report_sha256": FIXTURE_REPORT_SHA256,
        "fixture_analysis": fixture_report["analysis"],
        "fixture_parity": parity,
        "axes": manifest["axes"],
        "classification": CLASSIFICATION,
        "next_action": "commit this preflight, then execute the fixed 27-row diagnostic twice from the immutable decoded block",
    }


def analyze(
    root: Path, base: Path, manifest_bytes: bytes, manifest: dict[str, Any], runner_sha256: str
) -> dict[str, Any]:
    parents, block = validate_parents(base, manifest)
    validate_sources(root, manifest)
    fixture = fixture_directory(base, manifest)
    _, parity = extractor.validate_fixture(fixture)
    rows = analyze_rows(block)
    axes = {
        name: summarize_rows([row for row in rows if name in row["axes"]])
        for name in ["height", "angle", "distance"]
    }
    bands = frequency_band_summary(rows)
    classification = classify(rows, axes, bands)
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": classification["decision"],
        "claim": "OFFLINE_FIXED_LISTENER_AXES_CAUSAL_DIAGNOSTIC_ONLY / NO_SELECTION_TUNING_DENOISING_PHYSICS_OR_RUNTIME_CREDIT",
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "extractor_fixture_parity": parity,
        "thresholds": THRESHOLDS,
        "overall": summarize_rows(rows),
        "axes": axes,
        "frequency_bands": bands,
        "classification": classification,
        "rows": rows,
        "next_action": "interpret only against the frozen classification; do not select a row, tune, denoise, fetch another object, run physics or open Planter",
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
    print(f"Ceramic Cup observation diagnostic {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("additional payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
