#!/usr/bin/env python3
"""Run the frozen Bempp triaxial closed-mesh surface-mode control."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import sys
import tempfile
from typing import Any

import bempp_cl
import bempp_cl.api as bempp
import meshio
import numba
import numpy as np
import scipy


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-bempp-triaxial-mode.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-bempp-triaxial-mode.report.v1"
STUDY_ID = "physical-sound-independent-bempp-triaxial-xz-mode"
PROTOCOL_REVISION = "bempp-direct-neumann-to-dirichlet-triaxial-xz-v1"
MANIFEST_SHA256 = "74d8ebd1339faddf5727d0e02e1671a3a2a9295b83524f1f9336e1ec3cee267d"


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_external_file(root: Path, path: Path) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root):
        raise ValueError(f"manifest must stay outside the repository: {resolved}")
    if not resolved.is_file():
        raise ValueError(f"manifest is not a file: {resolved}")
    return resolved


def canonical_external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise ValueError(f"output must stay outside the repository: {resolved}")
    if resolved.exists():
        if not resolved.is_dir() or any(resolved.iterdir()):
            raise ValueError(f"output must be an empty directory: {resolved}")
    return resolved


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ValueError("manifest exceeds 128 KiB")
    actual = sha256(data)
    if actual != MANIFEST_SHA256:
        raise ValueError(f"manifest hash changed: expected {MANIFEST_SHA256}, got {actual}")
    manifest = json.loads(data)
    fixture = manifest["fixture"]
    solver = manifest["solver"]
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("study_id") != STUDY_ID
        or manifest.get("protocol_revision") != PROTOCOL_REVISION
        or fixture["semiaxes_metres"] != [0.08, 0.1, 0.13]
        or fixture["surface_mode"] != "two-times-parametric-unit-x-times-z"
        or fixture["surface_profile_projection"] != "ellipsoid-radial-pullback"
        or fixture["wave_number_reference_length_values"] != [0.75, 1.5]
        or fixture["listener_radius_reference_multipliers"] != [2.0, 4.0, 10.0]
        or fixture["listener_polar_cosines"] != [-0.9, -0.6, -0.3, 0.0, 0.3, 0.6, 0.9]
        or fixture["listener_azimuth_degrees"] != [0, 45, 90, 135, 180, 225, 270, 315]
        or len(fixture["listener_directions"]) != 56
        or solver["mesh_refinement_levels"] != [2, 3, 4]
        or solver["formulation"] != "W*p=(-0.5*I-K_prime)*q; field=-S*q+D*p"
        or manifest["data_policy"]
        != {
            "generated_synthetic_fixture_only": True,
            "fresh_realimpact_payload_access_allowed": False,
            "network_training_allowed": False,
            "runtime_or_quality_admission_credit_allowed": False,
        }
    ):
        raise ValueError("manifest does not match the frozen triaxial protocol")
    return manifest, data


def validate_environment(manifest: dict[str, Any]) -> dict[str, str]:
    actual = {
        "python": sys.version.split()[0],
        "bempp_cl": bempp_cl.__version__,
        "bempp_cl_revision": "a1eaaef9f96b9dd3d7c56b076740e06852a6e1c0",
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "numba": numba.__version__,
        "llvmlite": numba.llvmlite.__version__,
        "meshio": meshio.__version__,
        "device_interface": bempp.DEFAULT_DEVICE_INTERFACE,
    }
    if actual != manifest["environment"]:
        raise ValueError(f"Bempp environment changed: expected {manifest['environment']}, got {actual}")
    return actual


def normalize_direction(direction: list[float]) -> np.ndarray:
    value = np.asarray(direction, dtype=np.float64)
    norm = np.linalg.norm(value)
    if not np.isfinite(norm) or norm <= 0:
        raise ValueError("listener direction is invalid")
    return value / norm


def surface_profile(direction: np.ndarray) -> float:
    return 2.0 * float(direction[0]) * float(direction[2])


def neumann_grid_function(
    space: Any,
    semiaxes: np.ndarray,
    derivative_amplitude: complex,
) -> Any:
    axis_x, axis_y, axis_z = (float(value) for value in semiaxes)

    @bempp.complex_callable
    def prescribed_mode(x: np.ndarray, _normal: np.ndarray, _domain: int, result: np.ndarray) -> None:
        unit_x = x[0] / axis_x
        unit_y = x[1] / axis_y
        unit_z = x[2] / axis_z
        norm = math.sqrt(unit_x * unit_x + unit_y * unit_y + unit_z * unit_z)
        result[0] = derivative_amplitude * 2.0 * unit_x * unit_z / (norm * norm)

    return bempp.GridFunction(space, fun=prescribed_mode)


def solve_level(manifest: dict[str, Any], refinement_level: int) -> dict[str, Any]:
    fixture = manifest["fixture"]
    solver = manifest["solver"]
    semiaxes = np.asarray(fixture["semiaxes_metres"], dtype=np.float64)
    reference_length = float(fixture["reference_length_metres"])
    base_grid = bempp.shapes.regular_sphere(refinement_level)
    grid = bempp.Grid(base_grid.vertices * semiaxes[:, np.newaxis], base_grid.elements)
    neumann_space = bempp.function_space(grid, "DP", 0)
    dirichlet_space = bempp.function_space(grid, "P", 1)
    derivative_amplitude = complex(
        fixture["normal_pressure_derivative_amplitude_real"],
        fixture["normal_pressure_derivative_amplitude_imaginary"],
    )
    neumann = neumann_grid_function(neumann_space, semiaxes, derivative_amplitude)
    area_weighted_neumann_mean = complex(
        np.dot(neumann.coefficients, grid.volumes) / np.sum(grid.volumes)
    )
    directions = [normalize_direction(value) for value in fixture["listener_directions"]]
    profiles = [surface_profile(direction) for direction in directions]
    conditions: list[dict[str, Any]] = []
    solves: list[dict[str, Any]] = []

    for wave_number_reference_length in fixture["wave_number_reference_length_values"]:
        wave_number = float(wave_number_reference_length) / reference_length
        identity = bempp.operators.boundary.sparse.identity(
            neumann_space,
            dirichlet_space,
            dirichlet_space,
            precision=solver["precision"],
            device_interface=manifest["environment"]["device_interface"],
        )
        adjoint = bempp.operators.boundary.helmholtz.adjoint_double_layer(
            neumann_space,
            dirichlet_space,
            dirichlet_space,
            wave_number,
            assembler=solver["boundary_assembler"],
            precision=solver["precision"],
            device_interface=manifest["environment"]["device_interface"],
        )
        hypersingular = bempp.operators.boundary.helmholtz.hypersingular(
            dirichlet_space,
            dirichlet_space,
            dirichlet_space,
            wave_number,
            assembler=solver["boundary_assembler"],
            precision=solver["precision"],
            device_interface=manifest["environment"]["device_interface"],
        )
        rhs = (-0.5 * identity - adjoint) * neumann
        dirichlet, info, residuals, iteration_count = bempp.linalg.gmres(
            hypersingular,
            rhs,
            tol=float(solver["gmres_relative_tolerance"]),
            restart=int(solver["gmres_restart"]),
            maxiter=int(solver["gmres_max_iterations"]),
            return_residuals=True,
            return_iteration_count=True,
        )
        final_residual = float(residuals[-1]) if residuals else 0.0
        solves.append(
            {
                "wave_number_reference_length": float(wave_number_reference_length),
                "gmres_info": int(info),
                "gmres_iteration_count": int(iteration_count),
                "gmres_final_residual": final_residual,
            }
        )
        listeners: list[np.ndarray] = []
        listener_keys: list[tuple[float, int]] = []
        for radius_multiplier in fixture["listener_radius_reference_multipliers"]:
            for direction_index, direction in enumerate(directions):
                listeners.append(direction * reference_length * float(radius_multiplier))
                listener_keys.append((float(radius_multiplier), direction_index))
        points = np.stack(listeners, axis=1)
        single = bempp.operators.potential.helmholtz.single_layer(
            neumann_space,
            points,
            wave_number,
            assembler=solver["potential_assembler"],
            precision=solver["precision"],
            device_interface=manifest["environment"]["device_interface"],
        )
        double = bempp.operators.potential.helmholtz.double_layer(
            dirichlet_space,
            points,
            wave_number,
            assembler=solver["potential_assembler"],
            precision=solver["precision"],
            device_interface=manifest["environment"]["device_interface"],
        )
        computed_values = (-single.evaluate(neumann) + double.evaluate(dirichlet)).reshape(-1)
        for computed_raw, (radius_multiplier, direction_index) in zip(
            computed_values,
            listener_keys,
            strict=True,
        ):
            computed = complex(computed_raw)
            conditions.append(
                {
                    "wave_number_reference_length": float(wave_number_reference_length),
                    "frequency_hz": wave_number
                    * float(fixture["speed_of_sound_metres_per_second"])
                    / (2.0 * math.pi),
                    "listener_radius_reference_multiplier": radius_multiplier,
                    "direction_index": direction_index,
                    "surface_profile_direction_value": profiles[direction_index],
                    "computed": {"real": computed.real, "imaginary": computed.imag},
                }
            )
    return {
        "mesh_refinement_level": refinement_level,
        "panel_count": grid.number_of_elements,
        "condition_count": len(conditions),
        "area_weighted_neumann_mean": {
            "real": area_weighted_neumann_mean.real,
            "imaginary": area_weighted_neumann_mean.imag,
            "magnitude": abs(area_weighted_neumann_mean),
        },
        "solves": solves,
        "conditions": conditions,
    }


def complex_value(value: dict[str, float]) -> complex:
    return complex(value["real"], value["imaginary"])


def condition(
    level: dict[str, Any],
    wave_number_reference_length: float,
    radius_multiplier: float,
    direction_index: int,
) -> dict[str, Any]:
    matches = [
        row
        for row in level["conditions"]
        if row["wave_number_reference_length"] == wave_number_reference_length
        and row["listener_radius_reference_multiplier"] == radius_multiplier
        and row["direction_index"] == direction_index
    ]
    if len(matches) != 1:
        raise ValueError("triaxial condition grid is incomplete")
    return matches[0]


def directional_correlation(target: list[complex], candidate: list[complex]) -> float:
    target_values = np.asarray(target, dtype=np.complex128)
    candidate_values = np.asarray(candidate, dtype=np.complex128)
    denominator = float(np.linalg.norm(target_values) * np.linalg.norm(candidate_values))
    if denominator <= 1.0e-30:
        raise ValueError("triaxial directional correlation has a zero norm")
    return float(abs(np.vdot(target_values, candidate_values)) / denominator)


def convergence_report(
    manifest: dict[str, Any],
    coarse: dict[str, Any],
    medium: dict[str, Any],
    fine: dict[str, Any],
) -> dict[str, Any]:
    fixture = manifest["fixture"]
    tolerance = float(fixture["field_classification_absolute_tolerance"])
    active_threshold = float(fixture["active_field_minimum_peak_ratio"])
    coarse_medium_errors: list[float] = []
    medium_fine_errors: list[float] = []
    active_relative: list[float] = []
    active_magnitude_db: list[float] = []
    active_phase_degrees: list[float] = []
    correlations: list[dict[str, float]] = []
    active_count = 0

    for wave_number_reference_length in fixture["wave_number_reference_length_values"]:
        for radius_multiplier in fixture["listener_radius_reference_multipliers"]:
            coarse_values: list[complex] = []
            medium_values: list[complex] = []
            fine_values: list[complex] = []
            for direction_index in range(len(fixture["listener_directions"])):
                coarse_values.append(
                    complex_value(
                        condition(
                            coarse,
                            wave_number_reference_length,
                            radius_multiplier,
                            direction_index,
                        )["computed"]
                    )
                )
                medium_values.append(
                    complex_value(
                        condition(
                            medium,
                            wave_number_reference_length,
                            radius_multiplier,
                            direction_index,
                        )["computed"]
                    )
                )
                fine_values.append(
                    complex_value(
                        condition(
                            fine,
                            wave_number_reference_length,
                            radius_multiplier,
                            direction_index,
                        )["computed"]
                    )
                )
            peak = max(abs(value) for value in fine_values)
            if not math.isfinite(peak) or peak <= 1.0e-30:
                raise ValueError("triaxial fine field has no finite peak")
            for coarse_value, medium_value, fine_value in zip(
                coarse_values,
                medium_values,
                fine_values,
                strict=True,
            ):
                coarse_medium_errors.append(abs(medium_value - coarse_value) / peak)
                medium_fine_errors.append(abs(fine_value - medium_value) / peak)
                active = abs(fine_value) / peak + tolerance >= active_threshold
                if active:
                    active_count += 1
                    active_relative.append(abs(fine_value - medium_value) / abs(fine_value))
                    active_magnitude_db.append(
                        abs(20.0 * math.log10(abs(medium_value) / abs(fine_value)))
                    )
                    phase_delta = np.angle(medium_value) - np.angle(fine_value)
                    phase_error = math.atan2(math.sin(phase_delta), math.cos(phase_delta))
                    active_phase_degrees.append(abs(math.degrees(phase_error)))
            correlations.append(
                {
                    "wave_number_reference_length": float(wave_number_reference_length),
                    "listener_radius_reference_multiplier": float(radius_multiplier),
                    "medium_to_fine_complex_correlation": directional_correlation(
                        fine_values,
                        medium_values,
                    ),
                }
            )
    values = [
        coarse_medium_errors,
        medium_fine_errors,
        active_relative,
        active_magnitude_db,
        active_phase_degrees,
    ]
    if any(not group or any(not math.isfinite(value) for value in group) for group in values):
        raise ValueError("triaxial convergence metrics are invalid")
    coarse_medium_median = float(np.median(coarse_medium_errors))
    medium_fine_median = float(np.median(medium_fine_errors))
    ratio = medium_fine_median / coarse_medium_median
    return {
        "condition_count": len(medium_fine_errors),
        "active_condition_count": active_count,
        "medium_to_coarse_median_peak_normalized_complex_difference": coarse_medium_median,
        "fine_to_medium_median_peak_normalized_complex_difference": medium_fine_median,
        "fine_to_medium_vs_medium_to_coarse_median_difference_ratio": ratio,
        "fine_vs_medium_max_peak_normalized_complex_difference": max(medium_fine_errors),
        "fine_vs_medium_max_active_relative_complex_difference": max(active_relative),
        "fine_vs_medium_max_active_absolute_magnitude_difference_db": max(active_magnitude_db),
        "fine_vs_medium_max_active_absolute_phase_difference_degrees": max(
            active_phase_degrees
        ),
        "fine_vs_medium_min_directional_complex_correlation": min(
            row["medium_to_fine_complex_correlation"] for row in correlations
        ),
        "directional_correlations": correlations,
    }


def build_report(manifest: dict[str, Any], environment: dict[str, str]) -> dict[str, Any]:
    levels = [solve_level(manifest, level) for level in manifest["solver"]["mesh_refinement_levels"]]
    coarse, medium, fine = levels
    convergence = convergence_report(manifest, coarse, medium, fine)
    gates = manifest["admission_gates"]
    solve_rows = [row for level in levels for row in level["solves"]]
    gate = {
        "every_gmres_info_zero_passed": all(row["gmres_info"] == 0 for row in solve_rows),
        "maximum_final_gmres_residual_passed": max(
            row["gmres_final_residual"] for row in solve_rows
        )
        <= gates["maximum_final_gmres_residual"],
        "surface_mean_leakage_passed": max(
            level["area_weighted_neumann_mean"]["magnitude"] for level in levels
        )
        <= gates["maximum_abs_area_weighted_neumann_mean"],
        "fine_vs_medium_peak_difference_passed": convergence[
            "fine_vs_medium_max_peak_normalized_complex_difference"
        ]
        <= gates["fine_vs_medium_max_peak_normalized_complex_difference"],
        "fine_vs_medium_active_relative_difference_passed": convergence[
            "fine_vs_medium_max_active_relative_complex_difference"
        ]
        <= gates["fine_vs_medium_max_active_relative_complex_difference"],
        "fine_vs_medium_active_magnitude_difference_passed": convergence[
            "fine_vs_medium_max_active_absolute_magnitude_difference_db"
        ]
        <= gates["fine_vs_medium_max_active_absolute_magnitude_difference_db"],
        "fine_vs_medium_active_phase_difference_passed": convergence[
            "fine_vs_medium_max_active_absolute_phase_difference_degrees"
        ]
        <= gates["fine_vs_medium_max_active_absolute_phase_difference_degrees"],
        "fine_vs_medium_directional_correlation_passed": convergence[
            "fine_vs_medium_min_directional_complex_correlation"
        ]
        >= gates["fine_vs_medium_min_directional_complex_correlation"],
        "mesh_refinement_passed": convergence[
            "fine_to_medium_vs_medium_to_coarse_median_difference_ratio"
        ]
        <= gates["fine_to_medium_vs_medium_to_coarse_median_difference_ratio"],
    }
    passed = all(gate.values())
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": (
            "IndependentBemppTriaxialSurfaceModeSupported"
            if passed
            else "IndependentBemppTriaxialSurfaceModeRejected"
        ),
        "claim": (
            "SYNTHETIC_TRIAXIAL_CLOSED_MESH_XZ_SURFACE_MODE_CONVERGENCE_ONLY / "
            "NO_FEM_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT"
        ),
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "manifest_sha256": MANIFEST_SHA256,
        "environment": environment,
        "coarse": coarse,
        "medium": medium,
        "fine": fine,
        "convergence": convergence,
        "thresholds": gates,
        "gate": gate,
        "data_policy": manifest["data_policy"],
        "source_lineage": manifest["source_lineage"],
        "next_action": (
            "freeze this byte-exact fine field as the target for an axisymmetric control and "
            "a full-angular outgoing-multipole cooker before any fresh REALIMPACT payload"
            if passed
            else "keep REALIMPACT sealed and diagnose triaxial mesh convergence, prescribed-mode "
            "projection or function-space sufficiency on the unchanged fixture"
        ),
    }


def encode_report(report: dict[str, Any]) -> bytes:
    return (json.dumps(report, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def publish(output: Path, manifest: bytes, report: bytes) -> None:
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-bempp-triaxial-", dir=output.parent))
    try:
        (staging / "manifest.json").write_bytes(manifest)
        (staging / "report.json").write_bytes(report)
        if output.exists():
            output.rmdir()
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = canonical_external_file(root, arguments.manifest)
    output = canonical_external_output(root, arguments.output)
    manifest, manifest_bytes = load_manifest(manifest_path)
    environment = validate_environment(manifest)
    report = build_report(manifest, environment)
    report_bytes = encode_report(report)
    publish(output, manifest_bytes, report_bytes)
    print(f"physical sound Bempp triaxial mode: {output}")
    print(f"manifest sha256: {MANIFEST_SHA256}")
    print(f"coarse/medium/fine panels: {[report[key]['panel_count'] for key in ['coarse', 'medium', 'fine']]}")
    print(
        "fine-vs-medium max peak-normalized difference: "
        f"{report['convergence']['fine_vs_medium_max_peak_normalized_complex_difference']:.9f}"
    )
    print(
        "refinement ratio: "
        f"{report['convergence']['fine_to_medium_vs_medium_to_coarse_median_difference_ratio']:.9f}"
    )
    print(
        "minimum directional correlation: "
        f"{report['convergence']['fine_vs_medium_min_directional_complex_correlation']:.9f}"
    )
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256(report_bytes)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
