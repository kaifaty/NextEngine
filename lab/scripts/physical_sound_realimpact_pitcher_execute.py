#!/usr/bin/env python3
"""Execute the hash-closed REALIMPACT Pitcher calibration stages."""

from __future__ import annotations

import argparse
import hashlib
import ipaddress
import json
import math
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
from typing import Any
import urllib.error
import urllib.parse
import urllib.request
import zlib

import bempp_cl
import bempp_cl.api as bempp
import numba
import numpy as np
import scipy
from scipy.special import lpmv

sys.path.insert(0, str(Path(__file__).resolve().parent))
import physical_sound_realimpact_pitcher_calibration as base  # noqa: E402


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-pitcher-calibration-execution.manifest.v1"
)
REPAIR_MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-pitcher-calibration-report-serializer-repair.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-pitcher-calibration-execution-preflight.report.v1",
    "acquire": "nextengine.experimental-realimpact-pitcher-calibration-prefix-acquisition.report.v1",
    "decode": "nextengine.experimental-realimpact-pitcher-calibration-prefix-decode.report.v1",
    "analyze": "nextengine.experimental-realimpact-pitcher-calibration.report.v1",
}
PREFIX_NAME = "pitcher-prefix-512m.deflate"
DECODED_NAME = "pitcher-impact000-rows000-599.f32le"
PROJECTION_INPUT_SCHEMA = "nextengine.experimental-realimpact-transfer-projection-input.v1"
ROW_COUNT = 600
SAMPLE_COUNT = 230_470
ROW_BYTES = SAMPLE_COUNT * 4
DECODED_BYTES = ROW_COUNT * ROW_BYTES
PREFIX_BYTES = 536_870_912
REFERENCE_ROW = 7
MODE_COUNT = 16
SOUND_SPEED = 343.0
REPAIR_REVISION = "v3-numpy-bool-and-parent-lineage-report-repair"
ORIGINAL_EXECUTION_MANIFEST_SHA256 = (
    "8e791327595f43c672361e486ed50aa8512f1c8068de5efdbe6212a81df8ba45"
)


class ExecutionError(RuntimeError):
    """A frozen execution contract failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
    parser.add_argument("--execution-manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--rust-fixture", type=Path)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--repository-root", type=Path)
    return parser.parse_args()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def read_hashed_json(
    path: Path, expected_hash: str, label: str
) -> tuple[bytes, dict[str, Any]]:
    try:
        data = path.read_bytes()
    except OSError as error:
        raise ExecutionError(f"read {label} {path}: {error}") from error
    actual = sha256_bytes(data)
    if actual != expected_hash:
        raise ExecutionError(f"{label} hash changed: expected {expected_hash}, got {actual}")
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise ExecutionError(f"parse {label}: {error}") from error
    return data, value


def load_execution_manifest(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    repair_manifest = json.loads(data)
    repair = repair_manifest.get("report_serializer_repair")
    parent_ref = repair.get("parent_execution_manifest") if isinstance(repair, dict) else None
    if (
        repair_manifest.get("schema") != REPAIR_MANIFEST_SCHEMA
        or repair_manifest.get("study_id")
        != "physical-sound-realimpact-geometry-spatial-transfer"
        or repair_manifest.get("phase") != "pitcher-calibration-report-serializer-repair"
        or repair_manifest.get("revision") != REPAIR_REVISION
        or parent_ref
        != {
            "path": "pitcher-execution-manifest.json",
            "sha256": ORIGINAL_EXECUTION_MANIFEST_SHA256,
        }
    ):
        raise ExecutionError("Pitcher serializer-repair manifest contract changed")
    expected_script = repair_manifest.get("implementation", {}).get(
        "execution_script_sha256"
    )
    actual_script = sha256_file(Path(__file__).resolve())
    if expected_script != actual_script:
        raise ExecutionError(
            f"execution script hash changed: expected {expected_script}, got {actual_script}"
        )
    parent_path = (path.parent / parent_ref["path"]).resolve()
    parent_bytes, parent = read_hashed_json(
        parent_path, parent_ref["sha256"], "original execution manifest"
    )
    if (
        sha256_bytes(parent_bytes) != ORIGINAL_EXECUTION_MANIFEST_SHA256
        or parent.get("schema") != MANIFEST_SCHEMA
        or parent.get("study_id")
        != "physical-sound-realimpact-geometry-spatial-transfer"
        or parent.get("phase") != "pitcher-calibration-execution"
        or parent.get("opening_protocol", {}).get("planter_audio_allowed") is not False
    ):
        raise ExecutionError("original Pitcher execution manifest changed")
    expected_repair_manifest = {
        "schema": REPAIR_MANIFEST_SCHEMA,
        "study_id": "physical-sound-realimpact-geometry-spatial-transfer",
        "phase": "pitcher-calibration-report-serializer-repair",
        "revision": REPAIR_REVISION,
        "implementation": {"execution_script_sha256": actual_script},
        "report_serializer_repair": repair,
    }
    if repair_manifest != expected_repair_manifest:
        raise ExecutionError("serializer-repair manifest has undeclared fields")
    effective = json.loads(json.dumps(parent))
    effective["revision"] = REPAIR_REVISION
    effective["implementation"]["execution_script_sha256"] = actual_script
    effective["report_serializer_repair"] = repair
    return data, effective


def validate_repository_sources(
    root: Path, manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    reports = []
    for item in manifest["implementation"]["repository_sources"]:
        path = root / item["path"]
        actual = sha256_file(path)
        if actual != item["sha256"]:
            raise ExecutionError(
                f"bound repository source changed for {item['path']}: {actual}"
            )
        reports.append(
            {"path": item["path"], "sha256": actual, "bytes": path.stat().st_size}
        )
    return reports


def validate_environment(manifest: dict[str, Any]) -> dict[str, str]:
    expected = manifest["environment"]
    actual = {
        "python": sys.version.split()[0],
        "bempp_cl": bempp_cl.__version__,
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "numba": numba.__version__,
        "device_interface": "numba",
    }
    if any(actual[key] != str(expected[key]) for key in actual):
        raise ExecutionError(f"Bempp execution environment changed: {actual}")
    return actual


def resolve_reference(
    base_dir: Path, reference: dict[str, Any], label: str
) -> tuple[Path, bytes]:
    path = (base_dir / reference["path"]).resolve()
    try:
        path.relative_to(base_dir.parent)
    except ValueError as error:
        raise ExecutionError(f"{label} escapes the physical-sound store") from error
    data = path.read_bytes()
    actual = sha256_bytes(data)
    if actual != reference["sha256"]:
        raise ExecutionError(f"{label} hash changed: expected {reference['sha256']}, got {actual}")
    return path, data


def validate_prerequisites(
    execution_path: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], np.ndarray, list[dict[str, Any]]]:
    base_dir = execution_path.parent
    reports = []
    validate_report_serializer_repair(base_dir, manifest)
    parent_ref = manifest["parent_calibration_manifest"]
    parent_path, parent_bytes = resolve_reference(
        base_dir, parent_ref, "parent calibration manifest"
    )
    if sha256_bytes(parent_bytes) != base.MANIFEST_SHA256:
        raise ExecutionError("execution parent is not the frozen calibration manifest")
    parent = json.loads(parent_bytes)
    geometry_path = parent_path.parent / parent["pitcher_geometry"]["path"]
    geometry = base.decode_geometry_block(geometry_path)
    for reference in manifest["prerequisites"]:
        path, data = resolve_reference(base_dir, reference, reference["id"])
        value = json.loads(data)
        if value.get("schema") != reference["expected_schema"]:
            raise ExecutionError(f"prerequisite schema changed for {reference['id']}")
        if (
            "expected_decision" in reference
            and value.get("decision") != reference["expected_decision"]
        ):
            raise ExecutionError(f"prerequisite decision changed for {reference['id']}")
        if value.get("status") != reference.get("expected_status", value.get("status")):
            raise ExecutionError(f"prerequisite status changed for {reference['id']}")
        reports.append(
            {
                "id": reference["id"],
                "path": str(path),
                "sha256": reference["sha256"],
                "bytes": len(data),
                "schema": value["schema"],
            }
        )
    direction_ref = manifest["direction_source"]
    _, direction_bytes = resolve_reference(base_dir, direction_ref, "direction source")
    direction_report = json.loads(direction_bytes)
    if (
        direction_report.get("schema") != direction_ref["expected_schema"]
        or direction_report.get("decision") != direction_ref["expected_decision"]
    ):
        raise ExecutionError("frozen cooker direction source contract changed")
    directions = np.asarray(direction_report["listener_directions"], dtype=np.float64)
    if directions.shape != (56, 3) or not np.allclose(
        np.linalg.norm(directions, axis=1), 1.0, rtol=0.0, atol=1.0e-12
    ):
        raise ExecutionError("frozen cooker directions changed")
    validate_manifest_derivation(manifest, parent, geometry)
    return parent, directions, reports


def validate_report_serializer_repair(
    base_dir: Path, manifest: dict[str, Any]
) -> None:
    repair = manifest.get("report_serializer_repair")
    expected_repair = {
        "parent_execution_manifest": {
            "path": "pitcher-execution-manifest.json",
            "sha256": ORIGINAL_EXECUTION_MANIFEST_SHA256,
        },
        "failed_stage": "analyze",
        "failures": [
            "numpy.bool_ comparison result was not JSON serializable after computation and before report publication",
            "first serializer repair incorrectly expected the existing decode report to name the repair manifest instead of its immutable original execution manifest",
        ],
        "code_changes": [
            "cast gate comparison results to built-in bool before report assembly",
            "accept only the exact existing decode report bound to the immutable original execution manifest while binding the new analysis report to this repair manifest",
        ],
        "superseded_repair_manifest": {
            "path": "pitcher-execution-report-serializer-repair-manifest.json",
            "sha256": "603c1185884e50de0fda0df08ff98214d9ba56b70c325bb424c869e6f98128e3",
        },
        "numeric_model_changed": False,
        "decoder_changed": False,
        "thresholds_changed": False,
        "additional_network_access_allowed": False,
        "acquire_or_decode_stage_allowed": False,
        "existing_acquisition_report": {
            "path": "pitcher-prefix-acquisition/report.json",
            "sha256": "899fbbe9b6f098f72438817d27d701d0eabc5b5c4a7a0cbbd45de1ad74a5c819",
        },
        "existing_decode_report": {
            "path": "pitcher-rows000-599/report.json",
            "sha256": "29496c6f8f8f5aebb0525d056b7bdf40575ea2b66fc04f1687e449e13e359eef",
        },
        "decoded_block_sha256": "182f2010410bf3061176830309b83c3dad482447c8dd2e5f40346f81e0591e0f",
    }
    if repair != expected_repair:
        raise ExecutionError("Pitcher report-serializer repair contract changed")
    parent_ref = repair["parent_execution_manifest"]
    _, parent_bytes = resolve_reference(
        base_dir, parent_ref, "original execution manifest"
    )
    parent = json.loads(parent_bytes)
    expected = json.loads(json.dumps(parent))
    expected["revision"] = REPAIR_REVISION
    expected["implementation"]["execution_script_sha256"] = manifest[
        "implementation"
    ]["execution_script_sha256"]
    expected["report_serializer_repair"] = repair
    if manifest != expected:
        raise ExecutionError("repair manifest changes more than serialization lineage")
    for label in ["existing_acquisition_report", "existing_decode_report"]:
        _, report_bytes = resolve_reference(
            base_dir, repair[label], label.replace("_", " ")
        )
        report = json.loads(report_bytes)
        if report.get("execution_manifest_sha256") != ORIGINAL_EXECUTION_MANIFEST_SHA256:
            raise ExecutionError(f"{label} is not bound to the original execution manifest")
    resolve_reference(
        base_dir,
        repair["superseded_repair_manifest"],
        "superseded serializer repair manifest",
    )


def validate_manifest_derivation(
    manifest: dict[str, Any],
    parent: dict[str, Any],
    geometry: dict[str, np.ndarray],
) -> None:
    audio = parent["pitcher_audio"]
    acquisition = manifest["acquisition"]
    acquisition_pairs = {
        "archive_url": "archive_url",
        "archive_bytes": "archive_bytes",
        "archive_etag": "archive_etag",
        "archive_last_modified_http": "archive_last_modified_http",
        "entry_data_offset": "entry_data_offset",
        "compressed_prefix_bytes": "compressed_prefix_bytes",
        "decoded_payload_bytes": "decoded_payload_bytes",
    }
    if any(
        acquisition[target] != audio[source]
        for target, source in acquisition_pairs.items()
    ):
        raise ExecutionError("execution acquisition contract drifted from preregistration")
    if (
        manifest["frequency_mapping_gate"]
        != {
            "maximum_median_frequency_error_octaves": parent["frequency_mapping"][
                "maximum_median_frequency_error_octaves"
            ],
            "maximum_p90_frequency_error_octaves": parent["frequency_mapping"][
                "maximum_p90_frequency_error_octaves"
            ],
        }
        or manifest["mode_admission"] != parent["mode_admission"]
        or manifest["condition_gate"] != parent["condition_gate"]
        or manifest["comparison_gate"] != parent["comparison_gate"]
    ):
        raise ExecutionError("execution calibration gates drifted from preregistration")
    if manifest["rbf_control"] != {
        "id": "coordinate-only-rbf-sigma052-ridge001-v1",
        "sigma_metres": 0.52,
        "ridge": 0.001,
    }:
        raise ExecutionError("execution RBF control drifted from preregistration")
    expansion = manifest["expansion"]
    welded = geometry["welded_points"]
    minimum = np.min(welded, axis=0)
    maximum = np.max(welded, axis=0)
    expected_origin = (minimum + maximum) * 0.5
    expected_length = float(np.linalg.norm(maximum - minimum))
    if (
        expansion.get("origin_rule")
        != "axis-aligned bounding-box center of the frozen exact-weld original mesh"
        or not np.array_equal(
            np.asarray(expansion["origin_metres"], dtype=np.float64), expected_origin
        )
        or float(expansion["reference_length_metres"]) != expected_length
        or expansion.get("fit_radius_reference_multipliers") != [2.0]
        or expansion.get("held_radius_reference_multipliers") != [4.0, 10.0]
    ):
        raise ExecutionError("execution expansion origin or radii drifted")
    if manifest["cooker"]["held_gates"] != {
        "held_max_peak_normalized_complex_error": 0.05,
        "held_max_active_relative_complex_error": 0.08,
        "held_max_active_absolute_magnitude_error_db": 0.75,
        "held_max_active_absolute_phase_error_degrees": 5.0,
        "held_min_directional_complex_correlation": 0.998,
    }:
        raise ExecutionError("execution cooker gates drifted from frozen control")
    if (
        manifest["cooker"].get("candidate_maximum_degrees") != [2, 4, 6]
        or manifest["cooker"].get("ridge") != 1.0e-12
        or manifest["cooker"].get("selection_rule")
        != "smallest full-real degree passing every frozen 4L/10L gate"
        or manifest["bempp"]
        != {
            "boundary_assembler": "default_nonlocal",
            "potential_assembler": "dense",
            "precision": "double",
            "neumann_space": "DP0",
            "dirichlet_space": "P1",
            "gmres_relative_tolerance": 1.0e-8,
            "gmres_restart": 200,
            "gmres_max_iterations": 1000,
        }
        or manifest["decode"]
        != {
            "container": "ZIP raw DEFLATE entry prefix",
            "npy_header_bytes": 128,
            "npy_dtype": "<f4",
            "npy_fortran_order": False,
            "npy_shape": [3000, 230470],
            "decoded_row_range": [0, 600],
            "decoded_bytes": DECODED_BYTES,
        }
        or manifest["opening_protocol"]
        != {
            "pitcher_http_range_requests_allowed": 1,
            "pitcher_retry_allowed": False,
            "pitcher_prefix_growth_allowed": False,
            "planter_audio_allowed": False,
        }
    ):
        raise ExecutionError("execution solver or opening protocol drifted")


def listener_coordinates() -> tuple[np.ndarray, list[tuple[int, int, int]]]:
    return base.listener_coordinates()


def basis_columns(maximum_degree: int) -> list[tuple[int, int, str]]:
    columns = []
    for degree in range(maximum_degree + 1):
        columns.append((degree, 0, "m_zero"))
        for order in range(1, degree + 1):
            columns.append((degree, order, "cosine"))
            columns.append((degree, order, "sine"))
    return columns


def spherical_hankel(argument: float, maximum_degree: int) -> np.ndarray:
    if not math.isfinite(argument) or argument <= 1.0e-6:
        raise ExecutionError("cooker wave argument is invalid")
    result = np.zeros(maximum_degree + 1, dtype=np.complex128)
    sine = math.sin(argument)
    cosine = math.cos(argument)
    result[0] = complex(sine / argument, -cosine / argument)
    if maximum_degree > 0:
        result[1] = complex(
            sine / argument**2 - cosine / argument,
            -cosine / argument**2 - sine / argument,
        )
        for degree in range(1, maximum_degree):
            result[degree + 1] = (
                (2 * degree + 1) / argument * result[degree] - result[degree - 1]
            )
    return result


def cooker_basis(
    wave_number: float,
    positions: np.ndarray,
    origin: np.ndarray,
    maximum_degree: int,
) -> tuple[np.ndarray, list[tuple[int, int, str]]]:
    columns = basis_columns(maximum_degree)
    output = np.empty((len(positions), len(columns)), dtype=np.complex128)
    for row, position in enumerate(positions):
        delta = position - origin
        radius = float(np.linalg.norm(delta))
        if not math.isfinite(radius) or radius <= 1.0e-30:
            raise ExecutionError("cooker listener radius is invalid")
        cosine = max(-1.0, min(1.0, float(delta[2] / radius)))
        azimuth = math.atan2(float(delta[1]), float(delta[0]))
        hankel = spherical_hankel(wave_number * radius, maximum_degree)
        for column, (degree, order, component) in enumerate(columns):
            legendre = float(lpmv(order, degree, cosine))
            angular = {
                "m_zero": legendre,
                "cosine": legendre * math.cos(order * azimuth),
                "sine": legendre * math.sin(order * azimuth),
            }[component]
            output[row, column] = hankel[degree] * angular
    return output, columns


def fit_cooker(
    wave_number: float,
    positions: np.ndarray,
    values: np.ndarray,
    origin: np.ndarray,
    maximum_degree: int,
    ridge: float,
) -> tuple[np.ndarray, list[tuple[int, int, str]], float]:
    if (
        values.shape != (len(positions),)
        or not np.all(np.isfinite(values))
        or not math.isfinite(ridge)
        or ridge <= 0.0
    ):
        raise ExecutionError("cooker fit inputs are invalid")
    basis, columns = cooker_basis(wave_number, positions, origin, maximum_degree)
    scales = np.sqrt(np.mean(np.abs(basis) ** 2, axis=0))
    if np.any(~np.isfinite(scales)) or np.any(scales <= 1.0e-18):
        raise ExecutionError("cooker basis column has zero scale")
    standardized = basis / scales[None, :]
    real_matrix = np.block(
        [[standardized.real, -standardized.imag], [standardized.imag, standardized.real]]
    )
    target = np.concatenate([values.real, values.imag])
    normal = real_matrix.T @ real_matrix
    normal.flat[:: len(normal) + 1] += ridge
    parameters = np.linalg.solve(normal, real_matrix.T @ target)
    count = len(columns)
    coefficients = (parameters[:count] + 1j * parameters[count:]) / scales
    predicted = basis @ coefficients
    peak = float(np.max(np.abs(values)))
    if (
        not math.isfinite(peak)
        or peak <= 1.0e-30
        or not np.all(np.isfinite(coefficients))
    ):
        raise ExecutionError("cooker fit result is invalid")
    fit_error = float(np.max(np.abs(predicted - values)) / peak)
    return coefficients, columns, fit_error


def evaluate_cooker(
    wave_number: float,
    positions: np.ndarray,
    origin: np.ndarray,
    maximum_degree: int,
    coefficients: np.ndarray,
) -> np.ndarray:
    basis, _ = cooker_basis(wave_number, positions, origin, maximum_degree)
    return basis @ coefficients


def held_cooker_metrics(target: np.ndarray, predicted: np.ndarray) -> dict[str, float | int]:
    if (
        target.shape != (2, 56)
        or predicted.shape != target.shape
        or not np.all(np.isfinite(target))
        or not np.all(np.isfinite(predicted))
    ):
        raise ExecutionError("cooker held dimensions changed")
    peak_errors = []
    active_relative = []
    active_magnitude_db = []
    active_phase_degrees = []
    correlations = []
    active_count = 0
    for target_group, predicted_group in zip(target, predicted, strict=True):
        peak = float(np.max(np.abs(target_group)))
        if not math.isfinite(peak) or peak <= 1.0e-30:
            raise ExecutionError("cooker held target has no finite peak")
        active = np.abs(target_group) / peak + 1.0e-12 >= 0.25
        difference = np.abs(predicted_group - target_group)
        peak_errors.extend((difference / peak).tolist())
        active_count += int(np.sum(active))
        active_relative.extend((difference[active] / np.abs(target_group[active])).tolist())
        active_magnitude_db.extend(
            np.abs(20.0 * np.log10(np.abs(predicted_group[active]) / np.abs(target_group[active]))).tolist()
        )
        phase = np.angle(predicted_group[active]) - np.angle(target_group[active])
        active_phase_degrees.extend(
            np.abs(np.degrees(np.arctan2(np.sin(phase), np.cos(phase)))).tolist()
        )
        numerator = abs(np.vdot(target_group, predicted_group))
        denominator = math.sqrt(
            float(np.vdot(target_group, target_group).real)
            * float(np.vdot(predicted_group, predicted_group).real)
        )
        if not math.isfinite(denominator) or denominator <= 1.0e-30:
            raise ExecutionError("cooker held correlation is undefined")
        correlations.append(numerator / denominator)
    return {
        "held_condition_count": 112,
        "held_active_condition_count": active_count,
        "held_median_peak_normalized_complex_error": float(np.median(peak_errors)),
        "held_max_peak_normalized_complex_error": max(peak_errors),
        "held_max_active_relative_complex_error": max(active_relative),
        "held_max_active_absolute_magnitude_error_db": max(active_magnitude_db),
        "held_max_active_absolute_phase_error_degrees": max(active_phase_degrees),
        "held_min_directional_complex_correlation": min(correlations),
    }


def cooker_pass(metrics: dict[str, Any], gates: dict[str, Any]) -> bool:
    return bool(
        metrics["held_max_peak_normalized_complex_error"]
        <= gates["held_max_peak_normalized_complex_error"]
        and metrics["held_max_active_relative_complex_error"]
        <= gates["held_max_active_relative_complex_error"]
        and metrics["held_max_active_absolute_magnitude_error_db"]
        <= gates["held_max_active_absolute_magnitude_error_db"]
        and metrics["held_max_active_absolute_phase_error_degrees"]
        <= gates["held_max_active_absolute_phase_error_degrees"]
        and metrics["held_min_directional_complex_correlation"]
        >= gates["held_min_directional_complex_correlation"]
    )


def synthetic_cooker_control(manifest: dict[str, Any], directions: np.ndarray) -> dict[str, Any]:
    expansion = manifest["expansion"]
    origin = np.asarray(expansion["origin_metres"], dtype=np.float64)
    length = float(expansion["reference_length_metres"])
    frequency = 1_337.0
    wave_number = 2.0 * math.pi * frequency / SOUND_SPEED
    degree = 4
    columns = basis_columns(degree)
    coefficients = np.asarray(
        [complex(math.cos(i * 0.37), math.sin(i * 0.23)) / (i + 1) for i in range(len(columns))],
        dtype=np.complex128,
    )
    fit_positions = origin[None, :] + directions * (2.0 * length)
    held_positions = np.stack(
        [origin[None, :] + directions * (radius * length) for radius in [4.0, 10.0]]
    )
    fit_values = evaluate_cooker(wave_number, fit_positions, origin, degree, coefficients)
    fitted, _, fit_error = fit_cooker(
        wave_number, fit_positions, fit_values, origin, degree, 1.0e-12
    )
    target = np.stack(
        [
            evaluate_cooker(wave_number, positions, origin, degree, coefficients)
            for positions in held_positions
        ]
    )
    predicted = np.stack(
        [evaluate_cooker(wave_number, positions, origin, degree, fitted) for positions in held_positions]
    )
    metrics = held_cooker_metrics(target, predicted)
    if fit_error > 1.0e-8 or metrics["held_max_peak_normalized_complex_error"] > 1.0e-8:
        raise ExecutionError("local full-angular cooker recovery control failed")
    return {"maximum_degree": degree, "fit_error": fit_error, "held": metrics}


def preflight_report(
    execution_path: Path,
    execution_bytes: bytes,
    manifest: dict[str, Any],
    root: Path,
    rust_fixture: Path,
) -> dict[str, Any]:
    sources = validate_repository_sources(root, manifest)
    environment = validate_environment(manifest)
    _, directions, prerequisites = validate_prerequisites(execution_path, manifest)
    fixture, parity = base.validate_fixture(rust_fixture)
    control = synthetic_cooker_control(manifest, directions)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "PitcherCalibrationExecutionPreflightSupported",
        "claim": "LOCAL_EXECUTION_ENVIRONMENT_AND_COOKER_CONTROL_ONLY / NO_NETWORK_OR_RESERVED_AUDIO_ACCESS",
        "execution_manifest_sha256": sha256_bytes(execution_bytes),
        "execution_script_sha256": sha256_file(Path(__file__).resolve()),
        "environment": environment,
        "repository_sources": sources,
        "prerequisites": prerequisites,
        "extractor_fixture_report_sha256": base.FIXTURE_REPORT_SHA256,
        "extractor_parity": parity,
        "direction_count": len(directions),
        "direction_sha256": sha256_bytes(np.ascontiguousarray(directions).tobytes()),
        "synthetic_full_angular_cooker_control": control,
        "network_requests": 0,
        "reserved_audio_payload_bytes_read": 0,
        "planter_audio_payload_bytes_read": 0,
        "audio_acquisition_authorized": False,
        "offline_analysis_authorized": True,
        "report_serializer_repair": manifest["report_serializer_repair"],
        "next_action": "analyze the already decoded immutable Pitcher block twice without network, acquire or decode access",
    }


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file_pointer, code, message, headers, new_url):  # type: ignore[no-untyped-def]
        return None


def validate_public_host(url: str) -> list[str]:
    parsed = urllib.parse.urlparse(url)
    hostname = parsed.hostname
    if parsed.scheme != "https" or hostname is None or parsed.port not in {None, 443}:
        raise ExecutionError("Pitcher archive URL has no hostname")
    addresses = sorted(
        {
            item[4][0]
            for item in socket.getaddrinfo(hostname, 443, type=socket.SOCK_STREAM)
        }
    )
    if not addresses or any(not ipaddress.ip_address(value).is_global for value in addresses):
        raise ExecutionError(f"Pitcher archive host did not resolve only to public addresses: {addresses}")
    return addresses


def acquire_prefix(
    execution_path: Path,
    execution_bytes: bytes,
    manifest: dict[str, Any],
    output: Path,
) -> dict[str, Any]:
    protocol = manifest["acquisition"]
    url = protocol["archive_url"]
    addresses = validate_public_host(url)
    start = int(protocol["entry_data_offset"])
    length = int(protocol["compressed_prefix_bytes"])
    end = start + length - 1
    request = urllib.request.Request(
        url,
        headers={"Range": f"bytes={start}-{end}", "Accept-Encoding": "identity"},
        method="GET",
    )
    opener = urllib.request.build_opener(NoRedirect)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    prefix_path = staging / PREFIX_NAME
    digest = hashlib.sha256()
    count = 0
    try:
        try:
            response = opener.open(request, timeout=120)
        except urllib.error.HTTPError as error:
            raise ExecutionError(f"Pitcher range request failed: HTTP {error.code}") from error
        with response, prefix_path.open("wb") as handle:
            status = getattr(response, "status", response.getcode())
            content_range = response.headers.get("Content-Range")
            content_length = response.headers.get("Content-Length")
            etag = response.headers.get("ETag", "").strip('"')
            last_modified = response.headers.get("Last-Modified")
            if (
                status != 206
                or content_range != f"bytes {start}-{end}/{protocol['archive_bytes']}"
                or content_length != str(length)
                or etag != protocol["archive_etag"]
                or last_modified != protocol["archive_last_modified_http"]
            ):
                raise ExecutionError(
                    f"Pitcher range response identity changed: {status}, {content_range}, {content_length}, {etag}, {last_modified}"
                )
            while block := response.read(1024 * 1024):
                count += len(block)
                if count > length:
                    raise ExecutionError("Pitcher range response exceeded frozen prefix")
                digest.update(block)
                handle.write(block)
        if count != length:
            raise ExecutionError(f"Pitcher range response truncated: expected {length}, got {count}")
        report = {
            "schema": REPORT_SCHEMAS["acquire"],
            "status": "Validated",
            "decision": "PitcherCompressedPrefixAcquired",
            "claim": "ONE_FROZEN_PITCHER_PREFIX_REQUEST / PLANTER_REMAINS_SEALED / NO_CALIBRATION_RESULT",
            "execution_manifest_sha256": sha256_bytes(execution_bytes),
            "archive_url": url,
            "resolved_public_addresses": addresses,
            "http_status": 206,
            "content_range": f"bytes {start}-{end}/{protocol['archive_bytes']}",
            "prefix_path": PREFIX_NAME,
            "prefix_bytes": count,
            "prefix_sha256": digest.hexdigest(),
            "network_requests": 1,
            "reserved_pitcher_audio_payload_bytes_read": count,
            "planter_audio_payload_bytes_read": 0,
            "retry_allowed": False,
            "prefix_growth_allowed": False,
            "next_action": "decode only the frozen NPY header and rows 0 through 599; reject without another request if incomplete",
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        if output.exists():
            if not output.is_dir() or any(output.iterdir()):
                raise ExecutionError(f"output is not absent or empty: {output}")
            output.rmdir()
        staging.rename(output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def validate_npy_header(header: bytes) -> None:
    if (
        len(header) != 128
        or header[:6] != b"\x93NUMPY"
        or header[6:8] != b"\x01\x00"
        or int.from_bytes(header[8:10], "little") != 118
    ):
        raise ExecutionError("Pitcher NPY header identity changed")
    text = header[10:128].decode("utf-8")
    if (
        "'descr': '<f4'" not in text
        or "'fortran_order': False" not in text
        or "'shape': (3000, 230470)" not in text
        or not text.endswith("\n")
    ):
        raise ExecutionError("Pitcher NPY descriptor changed")


def decode_prefix(
    execution_bytes: bytes,
    manifest: dict[str, Any],
    acquisition: Path,
    output: Path,
) -> dict[str, Any]:
    report_bytes = (acquisition / "report.json").read_bytes()
    acquisition_report = json.loads(report_bytes)
    prefix_path = acquisition / acquisition_report["prefix_path"]
    if (
        acquisition_report.get("schema") != REPORT_SCHEMAS["acquire"]
        or acquisition_report.get("decision") != "PitcherCompressedPrefixAcquired"
        or acquisition_report.get("execution_manifest_sha256") != sha256_bytes(execution_bytes)
        or acquisition_report.get("network_requests") != 1
        or acquisition_report.get("content_range")
        != (
            f"bytes {manifest['acquisition']['entry_data_offset']}-"
            f"{manifest['acquisition']['entry_data_offset'] + PREFIX_BYTES - 1}/"
            f"{manifest['acquisition']['archive_bytes']}"
        )
        or acquisition_report.get("prefix_bytes") != PREFIX_BYTES
        or acquisition_report.get("planter_audio_payload_bytes_read") != 0
        or prefix_path.stat().st_size != PREFIX_BYTES
        or sha256_file(prefix_path) != acquisition_report.get("prefix_sha256")
    ):
        raise ExecutionError("Pitcher acquisition lineage changed")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    decoded_path = staging / DECODED_NAME
    decoder = zlib.decompressobj(-15)
    header = bytearray()
    digest = hashlib.sha256()
    decoded_count = 0
    target_count = 128 + DECODED_BYTES
    try:
        with prefix_path.open("rb") as source, decoded_path.open("wb") as target:
            while decoded_count + len(header) < target_count:
                compressed = source.read(1024 * 1024)
                if not compressed:
                    break
                pending = compressed
                while pending and decoded_count + len(header) < target_count:
                    remaining = target_count - decoded_count - len(header)
                    block = decoder.decompress(pending, remaining)
                    pending = decoder.unconsumed_tail
                    if len(header) < 128:
                        take = min(128 - len(header), len(block))
                        header.extend(block[:take])
                        block = block[take:]
                    if block:
                        digest.update(block)
                        target.write(block)
                        decoded_count += len(block)
                    if not pending:
                        break
        validate_npy_header(bytes(header))
        if decoded_count != DECODED_BYTES:
            raise ExecutionError(
                f"fixed Pitcher prefix decoded {decoded_count} row bytes, expected {DECODED_BYTES}"
            )
        report = {
            "schema": REPORT_SCHEMAS["decode"],
            "status": "Validated",
            "decision": "PitcherImpactZeroRowsDecoded",
            "claim": "PITCHER_CALIBRATION_ROWS_ONLY / PLANTER_REMAINS_SEALED / NO_CALIBRATION_RESULT",
            "execution_manifest_sha256": sha256_bytes(execution_bytes),
            "acquisition_report_sha256": sha256_bytes(report_bytes),
            "prefix_sha256": acquisition_report["prefix_sha256"],
            "npy_header_sha256": sha256_bytes(bytes(header)),
            "decoded_path": DECODED_NAME,
            "decoded_sha256": digest.hexdigest(),
            "decoded_bytes": decoded_count,
            "row_count": ROW_COUNT,
            "sample_count": SAMPLE_COUNT,
            "network_requests": 0,
            "additional_reserved_audio_payload_bytes_read": 0,
            "planter_audio_payload_bytes_read": 0,
            "next_action": "run calibration twice from this exact immutable decoded block; no further network access",
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        if output.exists():
            if not output.is_dir() or any(output.iterdir()):
                raise ExecutionError(f"output is not absent or empty: {output}")
            output.rmdir()
        staging.rename(output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def rbf_predict(
    coordinates: np.ndarray,
    anchors: np.ndarray,
    target: np.ndarray,
    sigma: float,
    ridge: float,
) -> np.ndarray:
    anchor_coordinates = coordinates[anchors]
    distance = np.linalg.norm(
        anchor_coordinates[:, None, :] - anchor_coordinates[None, :, :], axis=2
    )
    kernel = np.exp(-0.5 * (distance / sigma) ** 2)
    kernel.flat[:: len(kernel) + 1] += ridge
    weights = np.linalg.solve(kernel, target[anchors])
    prediction_distance = np.linalg.norm(
        coordinates[:, None, :] - anchor_coordinates[None, :, :], axis=2
    )
    return np.exp(-0.5 * (prediction_distance / sigma) ** 2) @ weights


def triangle_areas(points: np.ndarray, faces: np.ndarray) -> np.ndarray:
    triangles = points[faces]
    return 0.5 * np.linalg.norm(
        np.cross(triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]),
        axis=1,
    )


def bem_fields_for_mode(
    manifest: dict[str, Any],
    points: np.ndarray,
    faces: np.ndarray,
    vertex_mode: np.ndarray,
    frequency_hz: float,
    positions: np.ndarray,
) -> tuple[np.ndarray, dict[str, Any]]:
    solver = manifest["bempp"]
    grid = bempp.Grid(points.T, faces.T)
    neumann_space = bempp.function_space(grid, "DP", 0)
    dirichlet_space = bempp.function_space(grid, "P", 1)
    triangle_mode = np.mean(vertex_mode[faces], axis=1)
    norm = math.sqrt(float(np.sum(triangle_areas(points, faces) * triangle_mode**2)))
    if not math.isfinite(norm) or norm <= 1.0e-30:
        raise ExecutionError("Pitcher triangle mode normalization is invalid")
    triangle_mode /= norm
    neumann = bempp.GridFunction(
        neumann_space, coefficients=np.asarray(triangle_mode, dtype=np.complex128)
    )
    wave_number = 2.0 * math.pi * frequency_hz / SOUND_SPEED
    identity = bempp.operators.boundary.sparse.identity(
        neumann_space,
        dirichlet_space,
        dirichlet_space,
        precision="double",
        device_interface="numba",
    )
    adjoint = bempp.operators.boundary.helmholtz.adjoint_double_layer(
        neumann_space,
        dirichlet_space,
        dirichlet_space,
        wave_number,
        assembler="default_nonlocal",
        precision="double",
        device_interface="numba",
    )
    hypersingular = bempp.operators.boundary.helmholtz.hypersingular(
        dirichlet_space,
        dirichlet_space,
        dirichlet_space,
        wave_number,
        assembler="default_nonlocal",
        precision="double",
        device_interface="numba",
    )
    rhs = (-0.5 * identity - adjoint) * neumann
    dirichlet, info, residuals, iterations = bempp.linalg.gmres(
        hypersingular,
        rhs,
        tol=float(solver["gmres_relative_tolerance"]),
        restart=int(solver["gmres_restart"]),
        maxiter=int(solver["gmres_max_iterations"]),
        return_residuals=True,
        return_iteration_count=True,
    )
    single = bempp.operators.potential.helmholtz.single_layer(
        neumann_space,
        positions.T,
        wave_number,
        assembler="dense",
        precision="double",
        device_interface="numba",
    )
    double = bempp.operators.potential.helmholtz.double_layer(
        dirichlet_space,
        positions.T,
        wave_number,
        assembler="dense",
        precision="double",
        device_interface="numba",
    )
    values = (-single.evaluate(neumann) + double.evaluate(dirichlet)).reshape(-1)
    final_residual = float(residuals[-1]) if residuals else 0.0
    if not np.all(np.isfinite(values)) or not math.isfinite(final_residual):
        raise ExecutionError("Pitcher Bempp result is non-finite")
    return values, {
        "gmres_info": int(info),
        "gmres_iteration_count": int(iterations),
        "gmres_final_residual": final_residual,
        "triangle_mode_area_norm_before_normalization": norm,
    }


def condition_indices() -> tuple[np.ndarray, dict[str, np.ndarray]]:
    _, identities = listener_coordinates()
    anchor = np.asarray(
        [
            index
            for index, (angle, distance, microphone) in enumerate(identities)
            if angle in {0, 40, 80, 120, 160}
            and distance in {0, 666}
            and microphone in {0, 2, 4, 6, 7, 8, 10, 12, 14}
        ],
        dtype=np.int64,
    )
    strata = {
        "held-height": np.asarray(
            [
                index
                for index, (angle, distance, microphone) in enumerate(identities)
                if angle in {0, 40, 80, 120, 160}
                and distance in {0, 666}
                and microphone not in {0, 2, 4, 6, 7, 8, 10, 12, 14}
            ],
            dtype=np.int64,
        ),
        "held-angle": np.asarray(
            [
                index
                for index, (angle, distance, _) in enumerate(identities)
                if angle not in {0, 40, 80, 120, 160} and distance in {0, 666}
            ],
            dtype=np.int64,
        ),
        "held-distance": np.asarray(
            [
                index
                for index, (_, distance, _) in enumerate(identities)
                if distance not in {0, 666}
            ],
            dtype=np.int64,
        ),
    }
    if len(anchor) != 90 or [len(strata[key]) for key in strata] != [60, 150, 300]:
        raise ExecutionError("Pitcher held strata changed")
    return anchor, strata


def metric_summary(
    target_db: np.ndarray,
    predicted_db: np.ndarray,
    indices: np.ndarray,
    persistent: np.ndarray,
) -> dict[str, Any]:
    errors = np.abs(predicted_db[:, indices] - target_db[:, indices])
    constant_errors = np.abs(target_db[:, indices])
    per_mode = np.median(errors, axis=1)
    constant_per_mode = np.median(constant_errors, axis=1)
    median_error = float(np.median(errors))
    constant_median = float(np.median(constant_errors))
    if not math.isfinite(constant_median) or constant_median <= 1.0e-12:
        raise ExecutionError("constant-listener control has no measurable error")
    return {
        "condition_count": len(indices),
        "median_abs_error_db": median_error,
        "p90_abs_error_db": float(np.quantile(errors, 0.9, method="inverted_cdf")),
        "persistent_median_abs_error_db": float(np.median(errors[persistent])),
        "constant_median_abs_error_db": constant_median,
        "median_error_ratio_to_constant": median_error / constant_median,
        "improved_component_fraction_vs_constant": float(np.mean(per_mode < constant_per_mode)),
        "per_mode_median_abs_error_db": per_mode.tolist(),
    }


def condition_gate(summary: dict[str, Any], gates: dict[str, Any]) -> bool:
    return bool(
        summary["median_abs_error_db"] <= gates["maximum_median_abs_error_db"]
        and summary["p90_abs_error_db"] <= gates["maximum_p90_abs_error_db"]
        and summary["persistent_median_abs_error_db"]
        <= gates["maximum_persistent_median_abs_error_db"]
        and summary["improved_component_fraction_vs_constant"]
        >= gates["minimum_improved_component_fraction_vs_constant"]
        and summary["median_error_ratio_to_constant"]
        <= gates["maximum_median_error_ratio_to_constant"]
    )


def run_projection(
    root: Path,
    decoded_path: Path,
    modes: list[dict[str, Any]],
    output: Path,
) -> tuple[np.ndarray, dict[str, Any]]:
    input_value = {
        "schema": PROJECTION_INPUT_SCHEMA,
        "object_id": "65_PitcherCeramic",
        "extractor_id": base.EXTRACTOR_ID,
        "sample_rate_hz": base.SAMPLE_RATE_HZ,
        "sample_count": SAMPLE_COUNT,
        "row_count": ROW_COUNT,
        "normalization_row_index": REFERENCE_ROW,
        "modes": [
            {
                "frequency_hz": value["frequency_hz"],
                "persistent": value["matched_tail_frequency_hz"] is not None,
            }
            for value in modes
        ],
    }
    with tempfile.TemporaryDirectory(prefix="nextengine-pitcher-projection-") as temporary:
        input_path = Path(temporary) / "input.json"
        input_path.write_bytes(canonical_json(input_value))
        binary = root / "target/debug/xtask"
        if not binary.is_file():
            raise ExecutionError("build target/debug/xtask before Pitcher analysis")
        completed = subprocess.run(
            [
                str(binary),
                "physical-sound-registry",
                "realimpact-transfer-project",
                "--block",
                str(decoded_path),
                "--frequencies",
                str(input_path),
                "--output",
                str(output),
            ],
            cwd=root,
            check=False,
            capture_output=True,
            text=True,
        )
        if completed.returncode != 0:
            raise ExecutionError(f"Rust spatial projection failed: {completed.stderr}")
    report = json.loads((output / "report.json").read_bytes())
    projection_path = output / report["projection_path"]
    values = np.fromfile(projection_path, dtype="<f8")
    if values.size != MODE_COUNT * ROW_COUNT * 2:
        raise ExecutionError("Rust projection output dimensions changed")
    complex_values = values.reshape(MODE_COUNT, ROW_COUNT, 2)
    return complex_values[..., 0] + 1j * complex_values[..., 1], report


def analyze(
    execution_path: Path,
    execution_bytes: bytes,
    manifest: dict[str, Any],
    root: Path,
    decode_dir: Path,
    output: Path,
) -> dict[str, Any]:
    decode_report_bytes = (decode_dir / "report.json").read_bytes()
    decode_report = json.loads(decode_report_bytes)
    decoded_path = decode_dir / decode_report["decoded_path"]
    if (
        decode_report.get("schema") != REPORT_SCHEMAS["decode"]
        or decode_report.get("decision") != "PitcherImpactZeroRowsDecoded"
        or decode_report.get("execution_manifest_sha256")
        != ORIGINAL_EXECUTION_MANIFEST_SHA256
        or decode_report.get("decoded_bytes") != DECODED_BYTES
        or decode_report.get("row_count") != ROW_COUNT
        or decode_report.get("sample_count") != SAMPLE_COUNT
        or decode_report.get("network_requests") != 0
        or decode_report.get("planter_audio_payload_bytes_read") != 0
        or decoded_path.stat().st_size != DECODED_BYTES
        or sha256_file(decoded_path) != decode_report.get("decoded_sha256")
    ):
        raise ExecutionError("Pitcher decoded block lineage changed")
    parent, directions, prerequisites = validate_prerequisites(execution_path, manifest)
    geometry_path = execution_path.parent / parent["pitcher_geometry"]["path"]
    geometry = base.decode_geometry_block(geometry_path)
    rows = np.memmap(decoded_path, mode="r", dtype="<f4", shape=(ROW_COUNT, SAMPLE_COUNT))
    normalization = np.asarray(rows[REFERENCE_ROW], dtype=np.float64)
    extraction = base.analyze_v2(normalization)
    if extraction["selected_mode_count"] != MODE_COUNT:
        raise ExecutionError("Pitcher extractor did not produce 16 modes")
    frequencies = np.asarray(
        [value["frequency_hz"] for value in extraction["modes"]], dtype=np.float64
    )
    mapping = base.calibrate_frequency_mapping(frequencies, geometry["eigenvalues"])
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    try:
        target_complex, projection_report = run_projection(
            root, decoded_path, extraction["modes"], staging / "projection"
        )
        target_db = 20.0 * np.log10(np.maximum(np.abs(target_complex), 1.0e-24))
        coordinates, _ = listener_coordinates()
        anchors, strata = condition_indices()
        rbf_complex = np.stack(
            [
                rbf_predict(coordinates, anchors, component, 0.52, 0.001)
                for component in target_complex
            ]
        )
        rbf_reference = rbf_complex[:, REFERENCE_ROW]
        if np.any(np.abs(rbf_reference) <= 1.0e-30):
            raise ExecutionError("RBF control has a zero normalization response")
        rbf_complex /= rbf_reference[:, None]
        rbf_db = 20.0 * np.log10(np.maximum(np.abs(rbf_complex), 1.0e-24))
        origin = np.asarray(manifest["expansion"]["origin_metres"], dtype=np.float64)
        length = float(manifest["expansion"]["reference_length_metres"])
        shell_positions = np.concatenate(
            [origin[None, :] + directions * (radius * length) for radius in [2.0, 4.0, 10.0]],
            axis=0,
        )
        candidate_complex = np.full((MODE_COUNT, ROW_COUNT), np.nan + 1j * np.nan)
        mode_reports = []
        admitted = []
        for mode_index, (frequency, proxy_index) in enumerate(
            zip(frequencies, mapping["assignment"], strict=True)
        ):
            values, solve = bem_fields_for_mode(
                manifest,
                np.asarray(geometry["bem_points"], dtype=np.float64),
                np.asarray(geometry["bem_faces"], dtype=np.int64),
                np.asarray(geometry["bem_modes"][:, proxy_index], dtype=np.float64),
                float(frequency),
                shell_positions,
            )
            groups = values.reshape(3, 56)
            wave_number = 2.0 * math.pi * float(frequency) / SOUND_SPEED
            candidate_rows = []
            selected = None
            for degree in [2, 4, 6]:
                coefficients, columns, fit_error = fit_cooker(
                    wave_number,
                    shell_positions[:56],
                    groups[0],
                    origin,
                    degree,
                    1.0e-12,
                )
                held_prediction = np.stack(
                    [
                        evaluate_cooker(
                            wave_number,
                            shell_positions[56 + group * 56 : 56 + (group + 1) * 56],
                            origin,
                            degree,
                            coefficients,
                        )
                        for group in range(2)
                    ]
                )
                held = held_cooker_metrics(groups[1:], held_prediction)
                passed = cooker_pass(held, manifest["cooker"]["held_gates"])
                candidate_rows.append(
                    {
                        "maximum_degree": degree,
                        "column_count": len(columns),
                        "fit_max_peak_normalized_complex_error": fit_error,
                        "held": held,
                        "passed": passed,
                    }
                )
                if selected is None and passed:
                    selected = (degree, coefficients)
            reference_ratio = 0.0
            mode_admitted = False
            if (
                selected is not None
                and solve["gmres_info"] == 0
                and solve["gmres_final_residual"]
                <= manifest["mode_admission"]["maximum_gmres_residual"]
                and geometry["relative_eigen_residuals"][proxy_index]
                <= manifest["mode_admission"]["maximum_relative_eigen_residual"]
            ):
                degree, coefficients = selected
                predicted = evaluate_cooker(
                    wave_number, coordinates, origin, degree, coefficients
                )
                reference_ratio = float(
                    abs(predicted[REFERENCE_ROW]) / max(float(np.max(np.abs(predicted))), 1.0e-30)
                )
                if (
                    reference_ratio
                    >= manifest["mode_admission"]["minimum_reference_magnitude_to_mode_peak"]
                ):
                    predicted /= predicted[REFERENCE_ROW]
                    candidate_complex[mode_index] = predicted
                    admitted.append(mode_index)
                    mode_admitted = True
            mode_reports.append(
                {
                    "measured_mode_index": mode_index,
                    "frequency_hz": float(frequency),
                    "proxy_mode_index": int(proxy_index),
                    "relative_eigen_residual": float(
                        geometry["relative_eigen_residuals"][proxy_index]
                    ),
                    "solve": solve,
                    "cooker_candidates": candidate_rows,
                    "reference_magnitude_to_mode_peak": reference_ratio,
                    "admitted": mode_admitted,
                }
            )
        minimum_modes = manifest["mode_admission"]["minimum_admitted_modes"]
        admitted_persistent = [
            index
            for index in admitted
            if extraction["modes"][index]["matched_tail_frequency_hz"] is not None
        ]
        enough_modes = len(admitted) >= minimum_modes and bool(admitted_persistent)
        admitted_indices = np.asarray(admitted, dtype=np.int64)
        if enough_modes:
            candidate_db = 20.0 * np.log10(
                np.maximum(np.abs(candidate_complex[admitted_indices]), 1.0e-24)
            )
            admitted_target = target_db[admitted_indices]
            admitted_rbf = rbf_db[admitted_indices]
            persistent = np.asarray(
                [
                    extraction["modes"][index]["matched_tail_frequency_hz"] is not None
                    for index in admitted
                ],
                dtype=bool,
            )
            stratum_reports = {}
            all_condition_pass = True
            for name, indices in strata.items():
                summary = metric_summary(admitted_target, candidate_db, indices, persistent)
                passed = condition_gate(summary, manifest["condition_gate"])
                summary["passed"] = passed
                stratum_reports[name] = summary
                all_condition_pass &= passed
            held = np.concatenate(list(strata.values()))
            candidate_overall = metric_summary(admitted_target, candidate_db, held, persistent)
            rbf_overall = metric_summary(admitted_target, admitted_rbf, held, persistent)
            candidate_per_mode = np.asarray(candidate_overall["per_mode_median_abs_error_db"])
            rbf_per_mode = np.asarray(rbf_overall["per_mode_median_abs_error_db"])
            if rbf_overall["median_abs_error_db"] <= 1.0e-12:
                raise ExecutionError("RBF control error is too small for the frozen ratio")
            comparison = {
                "candidate_to_rbf_median_error_ratio": candidate_overall["median_abs_error_db"]
                / rbf_overall["median_abs_error_db"],
                "p90_regression_db": candidate_overall["p90_abs_error_db"]
                - rbf_overall["p90_abs_error_db"],
                "improved_component_fraction_vs_rbf": float(
                    np.mean(candidate_per_mode < rbf_per_mode)
                ),
            }
            comparison["passed"] = bool(
                comparison["candidate_to_rbf_median_error_ratio"]
                <= manifest["comparison_gate"]["maximum_candidate_to_rbf_median_error_ratio"]
                and comparison["p90_regression_db"]
                <= manifest["comparison_gate"]["maximum_p90_regression_db"]
                and comparison["improved_component_fraction_vs_rbf"]
                >= manifest["comparison_gate"]["minimum_improved_component_fraction_vs_rbf"]
            )
        else:
            stratum_reports = {}
            candidate_overall = None
            rbf_overall = None
            comparison = {"passed": False}
            all_condition_pass = False
        frequency_gate = bool(
            mapping["median_frequency_error_octaves"]
            <= manifest["frequency_mapping_gate"]["maximum_median_frequency_error_octaves"]
            and mapping["p90_frequency_error_octaves"]
            <= manifest["frequency_mapping_gate"]["maximum_p90_frequency_error_octaves"]
        )
        passed = bool(
            enough_modes
            and frequency_gate
            and all_condition_pass
            and comparison["passed"]
        )
        decision = (
            "PitcherGeometrySpatialCalibrationSupported"
            if passed
            else "PitcherGeometrySpatialCalibrationRejected"
        )
        report = {
            "schema": REPORT_SCHEMAS["analyze"],
            "status": "Validated",
            "decision": decision,
            "claim": "PITCHER_CALIBRATION_ONLY / PLANTER_REMAINS_SEALED / NO_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
            "execution_manifest_sha256": sha256_bytes(execution_bytes),
            "execution_script_sha256": sha256_file(Path(__file__).resolve()),
            "decode_report_sha256": sha256_bytes(decode_report_bytes),
            "decoded_sha256": decode_report["decoded_sha256"],
            "prerequisites": prerequisites,
            "extractor": extraction,
            "frequency_mapping": mapping,
            "frequency_mapping_gate_passed": frequency_gate,
            "projection_report_sha256": sha256_file(staging / "projection/report.json"),
            "projection_report": projection_report,
            "mode_reports": mode_reports,
            "admitted_mode_count": len(admitted),
            "admitted_persistent_mode_count": len(admitted_persistent),
            "minimum_admitted_modes": minimum_modes,
            "candidate_strata": stratum_reports,
            "candidate_overall": candidate_overall,
            "rbf_overall": rbf_overall,
            "candidate_vs_rbf": comparison,
            "gate": {
                "frequency_mapping_passed": frequency_gate,
                "minimum_mode_coverage_passed": enough_modes,
                "every_held_stratum_passed": all_condition_pass,
                "candidate_comparison_passed": comparison["passed"],
                "passed": passed,
            },
            "network_requests": 0,
            "additional_reserved_audio_payload_bytes_read": 0,
            "planter_audio_payload_bytes_read": 0,
            "next_action": (
                "repeat byte-identically, then freeze a separate Planter one-shot holdout manifest before access"
                if passed
                else "publish immutable calibration rejection, retain the authored clip fallback and stop before Planter"
            ),
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        if output.exists():
            if not output.is_dir() or any(output.iterdir()):
                raise ExecutionError(f"output is not absent or empty: {output}")
            output.rmdir()
        staging.rename(output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def publish_small(output: Path, execution_bytes: bytes, report: dict[str, Any]) -> None:
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ExecutionError(f"output must be absent or empty: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))
    try:
        (staging / "execution-manifest.json").write_bytes(execution_bytes)
        (staging / "report.json").write_bytes(canonical_json(report))
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    args = parse_arguments()
    execution_path = args.execution_manifest.resolve()
    execution_bytes, manifest = load_execution_manifest(execution_path)
    root = (
        args.repository_root.resolve()
        if args.repository_root is not None
        else Path(__file__).resolve().parents[2]
    )
    output = args.output.resolve()
    if args.stage in {"acquire", "decode"}:
        raise ExecutionError(
            "serializer-repair revision prohibits acquire and decode; use the frozen existing cache"
        )
    if args.stage == "preflight":
        if args.rust_fixture is None:
            raise ExecutionError("preflight requires --rust-fixture")
        report = preflight_report(
            execution_path,
            execution_bytes,
            manifest,
            root,
            args.rust_fixture.resolve(),
        )
        publish_small(output, execution_bytes, report)
    else:
        if args.input is None:
            raise ExecutionError("analyze requires --input <decode-directory>")
        validate_repository_sources(root, manifest)
        validate_environment(manifest)
        report = analyze(
            execution_path,
            execution_bytes,
            manifest,
            root,
            args.input.resolve(),
            output,
        )
    report_bytes = canonical_json(report)
    print(f"Pitcher execution stage {args.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print(f"Planter audio payload bytes read: {report.get('planter_audio_payload_bytes_read', 0)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ExecutionError, OSError, ValueError, KeyError) as error:
        print(f"pitcher execution: {error}", file=sys.stderr)
        raise SystemExit(1) from error
