#!/usr/bin/env python3
"""Run the frozen independent Bempp quadrupole surface-mode control."""

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
from scipy.special import spherical_jn, spherical_yn


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-bempp-surface-mode.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-bempp-surface-mode.report.v1"
STUDY_ID = "physical-sound-independent-bempp-quadrupole-surface-mode"
PROTOCOL_REVISION = "bempp-direct-neumann-to-dirichlet-p2-v3"
MANIFEST_SHA256 = "3a67f4add95dfc08fe7f55a08d9f6372fa478c1712a5c375910e23c1e2650e39"


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
        or fixture["spherical_harmonic_degree"] != 2
        or fixture["spherical_harmonic_order"] != 0
        or fixture["angular_profile"] != "legendre_p2_z"
        or fixture["profile_classification_absolute_tolerance"] != 1.0e-12
        or fixture["wave_number_radius_values"] != [0.25, 0.75, 1.5]
        or fixture["listener_radius_multipliers"] != [1.5, 3.0, 10.0]
        or len(fixture["listener_directions"]) != 22
        or solver["mesh_refinement_levels"] != [2, 3]
        or solver["surface_profile_projection"] != "radial-pullback-x-over-norm-x"
        or solver["formulation"] != "W*p=(-0.5*I-K_prime)*q; field=-S*q+D*p"
        or manifest["data_policy"]
        != {
            "generated_analytical_fixture_only": True,
            "fresh_realimpact_payload_access_allowed": False,
            "network_training_allowed": False,
            "runtime_or_quality_admission_credit_allowed": False,
        }
    ):
        raise ValueError("manifest does not match the frozen quadrupole protocol")
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


def legendre_p2(direction: np.ndarray) -> float:
    cosine = float(direction[2])
    return 0.5 * (3.0 * cosine * cosine - 1.0)


def outgoing_spherical_hankel(degree: int, argument: float, derivative: bool = False) -> complex:
    return complex(
        spherical_jn(degree, argument, derivative=derivative)
        + 1j * spherical_yn(degree, argument, derivative=derivative)
    )


def analytical_radial_factor(
    radius: float,
    listener_radius: float,
    wave_number: float,
    derivative_amplitude: complex,
) -> complex:
    numerator = outgoing_spherical_hankel(2, wave_number * listener_radius)
    denominator = wave_number * outgoing_spherical_hankel(
        2,
        wave_number * radius,
        derivative=True,
    )
    if abs(denominator) <= 1.0e-30:
        raise ValueError("analytical quadrupole denominator is zero")
    return derivative_amplitude * numerator / denominator


def neumann_grid_function(
    space: Any,
    radius: float,
    derivative_amplitude: complex,
) -> Any:
    @bempp.complex_callable
    def prescribed_mode(x: np.ndarray, _normal: np.ndarray, _domain: int, result: np.ndarray) -> None:
        norm = math.sqrt(x[0] * x[0] + x[1] * x[1] + x[2] * x[2])
        cosine = x[2] / norm
        result[0] = derivative_amplitude * 0.5 * (3.0 * cosine * cosine - 1.0)

    return bempp.GridFunction(space, fun=prescribed_mode)


def solve_level(manifest: dict[str, Any], refinement_level: int) -> dict[str, Any]:
    fixture = manifest["fixture"]
    solver = manifest["solver"]
    radius = float(fixture["radius_metres"])
    base_grid = bempp.shapes.regular_sphere(refinement_level)
    grid = bempp.Grid(base_grid.vertices * radius, base_grid.elements)
    neumann_space = bempp.function_space(grid, "DP", 0)
    dirichlet_space = bempp.function_space(grid, "P", 1)
    derivative_amplitude = complex(
        fixture["normal_pressure_derivative_amplitude_real"],
        fixture["normal_pressure_derivative_amplitude_imaginary"],
    )
    neumann = neumann_grid_function(neumann_space, radius, derivative_amplitude)
    area_weighted_neumann_mean = complex(
        np.dot(neumann.coefficients, grid.volumes) / np.sum(grid.volumes)
    )
    directions = [normalize_direction(value) for value in fixture["listener_directions"]]
    profiles = [legendre_p2(direction) for direction in directions]
    conditions: list[dict[str, Any]] = []
    solves: list[dict[str, Any]] = []

    for wave_number_radius in fixture["wave_number_radius_values"]:
        wave_number = float(wave_number_radius) / radius
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
                "wave_number_radius": float(wave_number_radius),
                "gmres_info": int(info),
                "gmres_iteration_count": int(iteration_count),
                "gmres_final_residual": final_residual,
            }
        )
        listeners: list[np.ndarray] = []
        listener_keys: list[tuple[float, int]] = []
        for radius_multiplier in fixture["listener_radius_multipliers"]:
            for direction_index, direction in enumerate(directions):
                listeners.append(direction * radius * float(radius_multiplier))
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
            profile = profiles[direction_index]
            radial = analytical_radial_factor(
                radius,
                radius * radius_multiplier,
                wave_number,
                derivative_amplitude,
            )
            analytical = radial * profile
            computed = complex(computed_raw)
            peak_magnitude = abs(radial)
            peak_normalized_error = abs(computed - analytical) / peak_magnitude
            classification_tolerance = float(
                fixture["profile_classification_absolute_tolerance"]
            )
            active = (
                abs(profile) + classification_tolerance
                >= float(fixture["active_profile_minimum_absolute_value"])
            )
            nodal = abs(profile) <= float(fixture["nodal_profile_maximum_absolute_value"])
            active_relative_error = abs(computed - analytical) / abs(analytical) if active else None
            magnitude_error_db = (
                abs(20.0 * math.log10(abs(computed) / abs(analytical))) if active else None
            )
            phase_error_degrees = None
            if active:
                phase_error = math.atan2(
                    math.sin(np.angle(computed) - np.angle(analytical)),
                    math.cos(np.angle(computed) - np.angle(analytical)),
                )
                phase_error_degrees = abs(math.degrees(phase_error))
            nodal_leakage_ratio = abs(computed) / peak_magnitude if nodal else None
            conditions.append(
                {
                    "wave_number_radius": float(wave_number_radius),
                    "frequency_hz": wave_number
                    * float(fixture["speed_of_sound_metres_per_second"])
                    / (2.0 * math.pi),
                    "listener_radius_multiplier": radius_multiplier,
                    "direction_index": direction_index,
                    "angular_profile_value": profile,
                    "active_lobe": active,
                    "nodal_direction": nodal,
                    "computed": {"real": computed.real, "imaginary": computed.imag},
                    "analytical": {"real": analytical.real, "imaginary": analytical.imag},
                    "analytical_peak_magnitude": peak_magnitude,
                    "peak_normalized_complex_error": float(peak_normalized_error),
                    "active_relative_complex_error": active_relative_error,
                    "active_absolute_magnitude_error_db": magnitude_error_db,
                    "active_absolute_phase_error_degrees": phase_error_degrees,
                    "nodal_leakage_ratio": nodal_leakage_ratio,
                }
            )
    return aggregate_level(
        refinement_level,
        grid.number_of_elements,
        area_weighted_neumann_mean,
        conditions,
        solves,
        fixture,
    )


def complex_value(value: dict[str, float]) -> complex:
    return complex(value["real"], value["imaginary"])


def directional_correlation(group: list[dict[str, Any]]) -> float:
    analytical = np.asarray([complex_value(row["analytical"]) for row in group])
    computed = np.asarray([complex_value(row["computed"]) for row in group])
    denominator = float(np.linalg.norm(analytical) * np.linalg.norm(computed))
    if denominator <= 1.0e-30:
        raise ValueError("directional correlation has a zero norm")
    return float(abs(np.vdot(analytical, computed)) / denominator)


def required_values(conditions: list[dict[str, Any]], key: str) -> list[float]:
    values = [row[key] for row in conditions if row[key] is not None]
    if not values or any(not math.isfinite(value) for value in values):
        raise ValueError(f"surface-mode conditions have invalid {key}")
    return values


def aggregate_level(
    refinement_level: int,
    panel_count: int,
    area_weighted_neumann_mean: complex,
    conditions: list[dict[str, Any]],
    solves: list[dict[str, Any]],
    fixture: dict[str, Any],
) -> dict[str, Any]:
    peak_errors = required_values(conditions, "peak_normalized_complex_error")
    active_relative = required_values(conditions, "active_relative_complex_error")
    active_magnitude = required_values(conditions, "active_absolute_magnitude_error_db")
    active_phase = required_values(conditions, "active_absolute_phase_error_degrees")
    nodal_leakage = required_values(conditions, "nodal_leakage_ratio")
    correlations: list[dict[str, float]] = []
    for wave_number_radius in fixture["wave_number_radius_values"]:
        for radius_multiplier in fixture["listener_radius_multipliers"]:
            group = [
                row
                for row in conditions
                if row["wave_number_radius"] == wave_number_radius
                and row["listener_radius_multiplier"] == radius_multiplier
            ]
            if len(group) != len(fixture["listener_directions"]):
                raise ValueError("directional correlation group is incomplete")
            correlations.append(
                {
                    "wave_number_radius": float(wave_number_radius),
                    "listener_radius_multiplier": float(radius_multiplier),
                    "complex_correlation": directional_correlation(group),
                }
            )
    return {
        "mesh_refinement_level": refinement_level,
        "panel_count": panel_count,
        "condition_count": len(conditions),
        "active_condition_count": len(active_relative),
        "nodal_condition_count": len(nodal_leakage),
        "area_weighted_neumann_mean": {
            "real": area_weighted_neumann_mean.real,
            "imaginary": area_weighted_neumann_mean.imag,
            "magnitude": abs(area_weighted_neumann_mean),
        },
        "median_peak_normalized_complex_error": float(np.median(peak_errors)),
        "max_peak_normalized_complex_error": max(peak_errors),
        "max_active_relative_complex_error": max(active_relative),
        "max_active_absolute_magnitude_error_db": max(active_magnitude),
        "max_active_absolute_phase_error_degrees": max(active_phase),
        "max_nodal_leakage_ratio": max(nodal_leakage),
        "min_directional_complex_correlation": min(
            row["complex_correlation"] for row in correlations
        ),
        "directional_correlations": correlations,
        "solves": solves,
        "conditions": conditions,
    }


def build_report(manifest: dict[str, Any], environment: dict[str, str]) -> dict[str, Any]:
    coarse = solve_level(manifest, manifest["solver"]["mesh_refinement_levels"][0])
    fine = solve_level(manifest, manifest["solver"]["mesh_refinement_levels"][1])
    refinement_ratio = (
        fine["median_peak_normalized_complex_error"]
        / coarse["median_peak_normalized_complex_error"]
    )
    gates = manifest["admission_gates"]
    solve_rows = coarse["solves"] + fine["solves"]
    gate = {
        "every_gmres_info_zero_passed": all(row["gmres_info"] == 0 for row in solve_rows),
        "maximum_final_gmres_residual_passed": max(
            row["gmres_final_residual"] for row in solve_rows
        )
        <= gates["maximum_final_gmres_residual"],
        "surface_mean_leakage_passed": max(
            coarse["area_weighted_neumann_mean"]["magnitude"],
            fine["area_weighted_neumann_mean"]["magnitude"],
        )
        <= gates["maximum_abs_area_weighted_neumann_mean"],
        "fine_peak_normalized_complex_error_passed": fine[
            "max_peak_normalized_complex_error"
        ]
        <= gates["fine_max_peak_normalized_complex_error"],
        "fine_active_relative_complex_error_passed": fine[
            "max_active_relative_complex_error"
        ]
        <= gates["fine_max_active_relative_complex_error"],
        "fine_active_magnitude_error_passed": fine[
            "max_active_absolute_magnitude_error_db"
        ]
        <= gates["fine_max_active_absolute_magnitude_error_db"],
        "fine_active_phase_error_passed": fine[
            "max_active_absolute_phase_error_degrees"
        ]
        <= gates["fine_max_active_absolute_phase_error_degrees"],
        "fine_nodal_leakage_passed": fine["max_nodal_leakage_ratio"]
        <= gates["fine_max_nodal_leakage_ratio"],
        "fine_directional_correlation_passed": fine[
            "min_directional_complex_correlation"
        ]
        >= gates["fine_min_directional_complex_correlation"],
        "refinement_passed": refinement_ratio
        <= gates["fine_to_coarse_median_peak_normalized_complex_error_ratio"],
    }
    passed = all(gate.values())
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": (
            "IndependentBemppQuadrupoleSurfaceModeSupported"
            if passed
            else "IndependentBemppQuadrupoleSurfaceModeRejected"
        ),
        "claim": (
            "ANALYTICAL_AXISYMMETRIC_QUADRUPOLE_SURFACE_MODE_ONLY / "
            "NO_NONSPHERICAL_GEOMETRY_FEM_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_"
            "RUNTIME_OR_REALIMPACT_CREDIT"
        ),
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "manifest_sha256": MANIFEST_SHA256,
        "environment": environment,
        "coarse": coarse,
        "fine": fine,
        "refinement": {"median_peak_normalized_complex_error_ratio": refinement_ratio},
        "thresholds": gates,
        "gate": gate,
        "data_policy": manifest["data_policy"],
        "source_lineage": manifest["source_lineage"],
        "next_action": (
            "freeze the byte-exact Bempp report as a target and cross-check the repository "
            "outgoing-multipole cooker before any fresh REALIMPACT payload"
            if passed
            else "keep REALIMPACT sealed and diagnose prescribed-mode projection, analytical "
            "Hankel convention, function spaces or mesh convergence on this fixture"
        ),
    }


def encode_report(report: dict[str, Any]) -> bytes:
    return (json.dumps(report, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def publish(output: Path, manifest: bytes, report: bytes) -> None:
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-bempp-mode-", dir=output.parent))
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
    print(f"physical sound Bempp surface mode: {output}")
    print(f"manifest sha256: {MANIFEST_SHA256}")
    print(f"coarse panels: {report['coarse']['panel_count']}")
    print(f"fine panels: {report['fine']['panel_count']}")
    print(
        "fine max peak-normalized complex error: "
        f"{report['fine']['max_peak_normalized_complex_error']:.9f}"
    )
    print(f"fine max nodal leakage: {report['fine']['max_nodal_leakage_ratio']:.9f}")
    print(
        "fine minimum directional correlation: "
        f"{report['fine']['min_directional_complex_correlation']:.9f}"
    )
    print(
        "refinement ratio: "
        f"{report['refinement']['median_peak_normalized_complex_error_ratio']:.9f}"
    )
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256(report_bytes)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
