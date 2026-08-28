#!/usr/bin/env python3
"""Run the frozen independent Bempp pulsating-sphere control."""

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


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-bempp-control.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-bempp-control.report.v1"
STUDY_ID = "physical-sound-independent-bempp-pulsating-sphere"
PROTOCOL_REVISION = "bempp-direct-neumann-to-dirichlet-v2"
MANIFEST_SHA256 = "33d30a3579ee3abfb2e9b1081a08ff89271f621ca3199c3ac2dd8e4fe9e13873"


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
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("study_id") != STUDY_ID
        or manifest.get("protocol_revision") != PROTOCOL_REVISION
        or manifest["fixture"]["wave_number_radius_values"] != [0.25, 0.75, 1.5]
        or manifest["fixture"]["listener_radius_multipliers"] != [1.5, 3.0, 10.0]
        or len(manifest["fixture"]["listener_directions"]) != 10
        or manifest["solver"]["mesh_refinement_levels"] != [2, 3]
        or manifest["solver"]["formulation"]
        != "W*p=(-0.5*I-K_prime)*q; field=-S*q+D*p"
        or manifest["data_policy"]
        != {
            "generated_analytical_fixture_only": True,
            "fresh_realimpact_payload_access_allowed": False,
            "network_training_allowed": False,
            "runtime_or_quality_admission_credit_allowed": False,
        }
    ):
        raise ValueError("manifest does not match the frozen Bempp protocol")
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


def analytical_field(radius: float, listener_radius: float, wave_number: float, derivative: complex) -> complex:
    numerator = derivative * radius**2 * np.exp(1j * wave_number * (listener_radius - radius))
    denominator = listener_radius * (-1.0 + 1j * wave_number * radius)
    return complex(numerator / denominator)


def normalize_direction(direction: list[float]) -> np.ndarray:
    value = np.asarray(direction, dtype=np.float64)
    norm = np.linalg.norm(value)
    if not np.isfinite(norm) or norm <= 0:
        raise ValueError("listener direction is invalid")
    return value / norm


def solve_level(manifest: dict[str, Any], refinement_level: int) -> dict[str, Any]:
    fixture = manifest["fixture"]
    solver = manifest["solver"]
    radius = float(fixture["radius_metres"])
    base_grid = bempp.shapes.regular_sphere(refinement_level)
    grid = bempp.Grid(base_grid.vertices * radius, base_grid.elements)
    neumann_space = bempp.function_space(grid, "DP", 0)
    dirichlet_space = bempp.function_space(grid, "P", 1)
    derivative = complex(
        fixture["normal_pressure_derivative_real"],
        fixture["normal_pressure_derivative_imaginary"],
    )
    neumann = bempp.GridFunction(
        neumann_space,
        coefficients=np.full(neumann_space.global_dof_count, derivative, dtype=np.complex128),
    )
    directions = [normalize_direction(value) for value in fixture["listener_directions"]]
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
            computed = complex(computed_raw)
            analytical = analytical_field(
                radius,
                radius * radius_multiplier,
                wave_number,
                derivative,
            )
            relative_error = abs(computed - analytical) / abs(analytical)
            magnitude_error_db = abs(20.0 * math.log10(abs(computed) / abs(analytical)))
            phase_error = math.atan2(
                math.sin(np.angle(computed) - np.angle(analytical)),
                math.cos(np.angle(computed) - np.angle(analytical)),
            )
            conditions.append(
                {
                    "wave_number_radius": float(wave_number_radius),
                    "frequency_hz": wave_number
                    * float(fixture["speed_of_sound_metres_per_second"])
                    / (2.0 * math.pi),
                    "listener_radius_multiplier": radius_multiplier,
                    "direction_index": direction_index,
                    "computed": {"real": computed.real, "imaginary": computed.imag},
                    "analytical": {"real": analytical.real, "imaginary": analytical.imag},
                    "relative_complex_error": float(relative_error),
                    "absolute_magnitude_error_db": float(magnitude_error_db),
                    "absolute_phase_error_degrees": abs(math.degrees(phase_error)),
                }
            )
    return aggregate_level(refinement_level, grid.number_of_elements, conditions, solves, fixture)


def aggregate_level(
    refinement_level: int,
    panel_count: int,
    conditions: list[dict[str, Any]],
    solves: list[dict[str, Any]],
    fixture: dict[str, Any],
) -> dict[str, Any]:
    errors = sorted(condition["relative_complex_error"] for condition in conditions)
    direction_spans: list[float] = []
    for wave_number_radius in fixture["wave_number_radius_values"]:
        for radius_multiplier in fixture["listener_radius_multipliers"]:
            magnitudes = [
                20.0
                * math.log10(
                    math.hypot(
                        condition["computed"]["real"],
                        condition["computed"]["imaginary"],
                    )
                )
                for condition in conditions
                if condition["wave_number_radius"] == wave_number_radius
                and condition["listener_radius_multiplier"] == radius_multiplier
            ]
            direction_spans.append(max(magnitudes) - min(magnitudes))
    return {
        "mesh_refinement_level": refinement_level,
        "panel_count": panel_count,
        "condition_count": len(conditions),
        "median_relative_complex_error": float(np.median(errors)),
        "max_relative_complex_error": max(errors),
        "max_absolute_magnitude_error_db": max(
            condition["absolute_magnitude_error_db"] for condition in conditions
        ),
        "max_absolute_phase_error_degrees": max(
            condition["absolute_phase_error_degrees"] for condition in conditions
        ),
        "max_direction_magnitude_span_db": max(direction_spans),
        "solves": solves,
        "conditions": conditions,
    }


def build_report(manifest: dict[str, Any], environment: dict[str, str]) -> dict[str, Any]:
    coarse = solve_level(manifest, manifest["solver"]["mesh_refinement_levels"][0])
    fine = solve_level(manifest, manifest["solver"]["mesh_refinement_levels"][1])
    refinement_ratio = (
        fine["median_relative_complex_error"] / coarse["median_relative_complex_error"]
    )
    gates = manifest["admission_gates"]
    solve_rows = coarse["solves"] + fine["solves"]
    gate = {
        "every_gmres_info_zero_passed": all(row["gmres_info"] == 0 for row in solve_rows),
        "maximum_final_gmres_residual_passed": max(
            row["gmres_final_residual"] for row in solve_rows
        )
        <= gates["maximum_final_gmres_residual"],
        "fine_relative_complex_error_passed": fine["max_relative_complex_error"]
        <= gates["fine_max_relative_complex_error"],
        "fine_magnitude_error_passed": fine["max_absolute_magnitude_error_db"]
        <= gates["fine_max_absolute_magnitude_error_db"],
        "fine_phase_error_passed": fine["max_absolute_phase_error_degrees"]
        <= gates["fine_max_absolute_phase_error_degrees"],
        "fine_direction_symmetry_passed": fine["max_direction_magnitude_span_db"]
        <= gates["fine_max_direction_magnitude_span_db"],
        "refinement_passed": refinement_ratio
        <= gates["fine_to_coarse_median_relative_complex_error_ratio"],
    }
    passed = all(gate.values())
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": (
            "IndependentBemppAnalyticalControlSupported"
            if passed
            else "IndependentBemppAnalyticalControlRejected"
        ),
        "claim": (
            "INDEPENDENT_ANALYTICAL_SPHERE_CONTROL_ONLY / "
            "NO_NONSPHERICAL_SURFACE_MODE_REAL_OBJECT_MATERIAL_QUALITY_"
            "ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT"
        ),
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "manifest_sha256": MANIFEST_SHA256,
        "environment": environment,
        "coarse": coarse,
        "fine": fine,
        "refinement": {"median_relative_complex_error_ratio": refinement_ratio},
        "thresholds": gates,
        "gate": gate,
        "data_policy": manifest["data_policy"],
        "source_lineage": manifest["source_lineage"],
        "next_action": (
            "freeze one synthetic non-spherical prescribed surface mode and "
            "cross-check its field against the repository boundary harness "
            "before any fresh REALIMPACT payload"
            if passed
            else "keep REALIMPACT sealed and diagnose formulation, sign, "
            "function-space or mesh convergence on the same analytical sphere"
        ),
    }


def encode_report(report: dict[str, Any]) -> bytes:
    return (json.dumps(report, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def publish(output: Path, manifest: bytes, report: bytes) -> None:
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-bempp-", dir=output.parent))
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
    print(f"physical sound Bempp control: {output}")
    print(f"manifest sha256: {MANIFEST_SHA256}")
    print(f"coarse panels: {report['coarse']['panel_count']}")
    print(f"fine panels: {report['fine']['panel_count']}")
    print(f"fine max relative complex error: {report['fine']['max_relative_complex_error']:.9f}")
    print(f"refinement ratio: {report['refinement']['median_relative_complex_error_ratio']:.9f}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256(report_bytes)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
