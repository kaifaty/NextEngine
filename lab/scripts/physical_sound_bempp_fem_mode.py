#!/usr/bin/env python3
"""Run the frozen elastic FEM eigenmode to Bempp coupling control."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
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
from scipy.sparse.linalg import eigsh

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from next_lab.physical_sound_glass_corpus import (  # noqa: E402
    Mesh,
    assemble_linear_elasticity,
)


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-bempp-fem-mode.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-bempp-fem-mode.report.v1"
COARSE_MANIFEST_SHA256 = "eaff559de9840a852100b14717f652555d54d465974b5946b0813fd8a03a79d4"
REFINED_MANIFEST_SHA256 = "1adfe3d68ee5c9d806fe2161311a322cb6df2dd6161acbe340369fc9ee5aabf7"


@dataclass
class ElasticLevel:
    angular_level: int
    radial_divisions: int
    mesh: Mesh
    unit_vertices: np.ndarray
    surface_elements: np.ndarray
    outer_nodes: np.ndarray
    eigenvalues: np.ndarray
    eigenvectors: np.ndarray
    residuals: np.ndarray
    frequencies_hz: np.ndarray
    sample_profiles: np.ndarray
    surface_normal_energy_fractions: np.ndarray
    axisymmetric_residuals: np.ndarray


@dataclass
class SelectedLevel:
    level: ElasticLevel
    mode_index: int
    match_correlation: float
    signed_eigenvector: np.ndarray
    normalized_sample_profile: np.ndarray


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def array_sha256(value: np.ndarray) -> str:
    canonical = np.ascontiguousarray(value)
    return sha256(canonical.tobytes(order="C"))


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
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise ValueError(f"output must be an empty directory: {resolved}")
    return resolved


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes, str]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise ValueError("manifest exceeds 128 KiB")
    actual = sha256(data)
    if actual not in {COARSE_MANIFEST_SHA256, REFINED_MANIFEST_SHA256}:
        raise ValueError(f"FEM-to-Bempp manifest hash is not registered: {actual}")
    manifest = json.loads(data)
    fixture = manifest["fixture"]
    fem = manifest["fem"]
    bem = manifest["bem"]
    coarse = actual == COARSE_MANIFEST_SHA256
    expected_identity = (
        (
            "physical-sound-elastic-fem-mode-to-bempp-triaxial-control",
            "core-clamped-linear-tetrahedral-fem-to-bempp-v1",
            [1, 2, 3],
            [4, 8, 16],
        )
        if coarse
        else (
            "physical-sound-elastic-fem-mode-to-bempp-triaxial-refined-control",
            "core-clamped-linear-tetrahedral-fem-to-bempp-refined-v1",
            [2, 3, 4],
            [8, 12, 16],
        )
    )
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("study_id") != expected_identity[0]
        or manifest.get("protocol_revision") != expected_identity[1]
        or fixture["semiaxes_metres"] != [0.08, 0.1, 0.13]
        or fixture["material"]
        != {
            "density_kg_m3": 1000,
            "youngs_modulus_pa": 100000000,
            "poisson_ratio": 0.25,
        }
        or fixture["support"]
        != {"kind": "fixed_concentric_core", "maximum_normalized_radius": 0.25}
        or fixture["listener_polar_cosines"] != [-0.9, -0.6, -0.3, 0, 0.3, 0.6, 0.9]
        or fixture["listener_azimuth_degrees"] != [0, 45, 90, 135, 180, 225, 270, 315]
        or fixture["listener_radius_reference_multipliers"] != [2, 4, 10]
        or fem["angular_refinement_levels"] != expected_identity[2]
        or fem["radial_divisions"] != expected_identity[3]
        or fem["eigensolve_mode_count"] != 16
        or bem["surface_mesh"] != "outer-shell-of-matching-fem-mesh"
        or manifest["data_policy"]
        != {
            "generated_synthetic_fixture_only": True,
            "fresh_realimpact_payload_access_allowed": False,
            "network_training_allowed": False,
            "runtime_or_quality_admission_credit_allowed": False,
        }
    ):
        raise ValueError("manifest does not match the frozen FEM-to-Bempp protocol")
    return manifest, data, actual


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
        raise ValueError(
            "FEM/Bempp environment changed: "
            f"expected {manifest['environment']}, got {actual}"
        )
    return actual


def listener_directions(fixture: dict[str, Any]) -> list[np.ndarray]:
    directions = []
    for polar_cosine in fixture["listener_polar_cosines"]:
        radial = math.sqrt(max(0.0, 1.0 - float(polar_cosine) ** 2))
        for azimuth_degrees in fixture["listener_azimuth_degrees"]:
            azimuth = math.radians(float(azimuth_degrees))
            directions.append(
                np.asarray(
                    [radial * math.cos(azimuth), radial * math.sin(azimuth), polar_cosine],
                    dtype=np.float64,
                )
            )
    return directions


def build_elastic_mesh(
    angular_level: int,
    radial_divisions: int,
    semiaxes: np.ndarray,
    fixed_core_radius: float,
) -> tuple[Mesh, np.ndarray, np.ndarray, np.ndarray]:
    base = bempp.shapes.regular_sphere(angular_level)
    unit_vertices = np.asarray(base.vertices.T, dtype=np.float64)
    surface_elements = np.asarray(base.elements.T, dtype=np.int32)
    angular_count = len(unit_vertices)
    nodes = [np.zeros(3, dtype=np.float64)]
    shell_offsets = []
    for shell in range(1, radial_divisions + 1):
        shell_offsets.append(len(nodes))
        scale = shell / radial_divisions
        nodes.extend(unit_vertices * semiaxes * scale)
    node_array = np.asarray(nodes, dtype=np.float64)

    tetrahedra: list[tuple[int, int, int, int]] = []
    first_offset = shell_offsets[0]
    for triangle in surface_elements:
        a, b, c = (first_offset + int(index) for index in triangle)
        tetrahedra.append((0, a, b, c))
    for lower_shell in range(radial_divisions - 1):
        lower = shell_offsets[lower_shell]
        upper = shell_offsets[lower_shell + 1]
        for triangle in surface_elements:
            a, b, c = sorted(int(index) for index in triangle)
            a0, b0, c0 = lower + a, lower + b, lower + c
            a1, b1, c1 = upper + a, upper + b, upper + c
            tetrahedra.extend(
                [(a0, b0, c0, c1), (a0, b0, b1, c1), (a0, a1, b1, c1)]
            )
    tetrahedron_array = np.asarray(tetrahedra, dtype=np.int32)
    volumes = np.empty(len(tetrahedron_array), dtype=np.float64)
    for index, tetrahedron in enumerate(tetrahedron_array):
        coordinates = node_array[tetrahedron]
        volumes[index] = abs(float(np.linalg.det(coordinates[1:] - coordinates[0]))) / 6.0
    if np.any(~np.isfinite(volumes)) or np.min(volumes) <= 1.0e-18:
        raise ValueError("concentric FEM mesh contains a degenerate tetrahedron")
    normalized = node_array / semiaxes
    normalized_radius = np.linalg.norm(normalized, axis=1)
    fixed_nodes = np.flatnonzero(normalized_radius <= fixed_core_radius + 1.0e-12).astype(np.int32)
    if len(fixed_nodes) < 4:
        raise ValueError("fixed FEM core has too few nodes")
    outer_offset = shell_offsets[-1]
    outer_nodes = np.arange(outer_offset, outer_offset + angular_count, dtype=np.int32)
    return (
        Mesh(nodes=node_array, tetrahedra=tetrahedron_array, fixed_nodes=fixed_nodes),
        unit_vertices,
        surface_elements,
        outer_nodes,
    )


def outward_triangle_normal(points: np.ndarray) -> np.ndarray:
    normal = np.cross(points[1] - points[0], points[2] - points[0])
    norm = float(np.linalg.norm(normal))
    if not math.isfinite(norm) or norm <= 1.0e-18:
        raise ValueError("surface triangle normal is invalid")
    normal /= norm
    if float(np.dot(normal, points.mean(axis=0))) < 0.0:
        normal *= -1.0
    return normal


def sample_surface_profile(
    unit_vertices: np.ndarray,
    elements: np.ndarray,
    physical_vertices: np.ndarray,
    displacement: np.ndarray,
    directions: list[np.ndarray],
) -> np.ndarray:
    samples = []
    for direction in directions:
        match = None
        best_margin = -math.inf
        for triangle in elements:
            unit_triangle = unit_vertices[triangle]
            plane_normal = np.cross(
                unit_triangle[1] - unit_triangle[0],
                unit_triangle[2] - unit_triangle[0],
            )
            denominator = float(np.dot(plane_normal, direction))
            if abs(denominator) <= 1.0e-14:
                continue
            distance = float(np.dot(plane_normal, unit_triangle[0])) / denominator
            if distance <= 0.0:
                continue
            point = direction * distance
            edge0 = unit_triangle[1] - unit_triangle[0]
            edge1 = unit_triangle[2] - unit_triangle[0]
            delta = point - unit_triangle[0]
            gram = np.asarray(
                [[np.dot(edge0, edge0), np.dot(edge0, edge1)],
                 [np.dot(edge0, edge1), np.dot(edge1, edge1)]],
                dtype=np.float64,
            )
            weights12 = np.linalg.solve(
                gram,
                np.asarray([np.dot(delta, edge0), np.dot(delta, edge1)]),
            )
            weights = np.asarray([1.0 - weights12.sum(), weights12[0], weights12[1]])
            margin = float(np.min(weights))
            if margin >= -1.0e-10 and margin > best_margin:
                match = (triangle, weights)
                best_margin = margin
        if match is None:
            raise ValueError("listener direction did not intersect the FEM surface mesh")
        triangle, weights = match
        interpolated = weights @ displacement[triangle]
        normal = outward_triangle_normal(physical_vertices[triangle])
        samples.append(float(np.dot(interpolated, normal)))
    return np.asarray(samples, dtype=np.float64)


def axisymmetric_degree4_residual(samples: np.ndarray, directions: list[np.ndarray]) -> float:
    cosine = np.asarray([direction[2] for direction in directions], dtype=np.float64)
    columns = np.stack(
        [
            np.ones_like(cosine),
            cosine,
            0.5 * (3.0 * cosine**2 - 1.0),
            0.5 * (5.0 * cosine**3 - 3.0 * cosine),
            (35.0 * cosine**4 - 30.0 * cosine**2 + 3.0) / 8.0,
        ],
        axis=1,
    )
    fitted = columns @ np.linalg.lstsq(columns, samples, rcond=None)[0]
    denominator = float(np.linalg.norm(samples))
    if denominator <= 1.0e-30:
        return 0.0
    return float(np.linalg.norm(samples - fitted) / denominator)


def mode_correlation(left: np.ndarray, right: np.ndarray) -> float:
    denominator = float(np.linalg.norm(left) * np.linalg.norm(right))
    if denominator <= 1.0e-30:
        return 0.0
    return float(abs(np.vdot(left, right)) / denominator)


def solve_elastic_level(
    manifest: dict[str, Any],
    angular_level: int,
    radial_divisions: int,
    directions: list[np.ndarray],
) -> ElasticLevel:
    fixture = manifest["fixture"]
    fem = manifest["fem"]
    semiaxes = np.asarray(fixture["semiaxes_metres"], dtype=np.float64)
    mesh, unit_vertices, surface_elements, outer_nodes = build_elastic_mesh(
        angular_level,
        radial_divisions,
        semiaxes,
        float(fixture["support"]["maximum_normalized_radius"]),
    )
    stiffness, mass = assemble_linear_elasticity(mesh, fixture["material"])
    fixed_degrees = np.sort(
        np.concatenate([3 * mesh.fixed_nodes + axis for axis in range(3)])
    )
    free_degrees = np.setdiff1d(np.arange(stiffness.shape[0], dtype=np.int64), fixed_degrees)
    reduced_stiffness = stiffness[free_degrees][:, free_degrees]
    reduced_mass = mass[free_degrees][:, free_degrees]
    start = np.linspace(1.0, 2.0, len(free_degrees), dtype=np.float64)
    start /= np.linalg.norm(start)
    eigenvalues, reduced_vectors = eigsh(
        reduced_stiffness,
        k=int(fem["eigensolve_mode_count"]),
        M=reduced_mass,
        sigma=0.0,
        which="LM",
        v0=start,
        tol=float(fem["relative_tolerance"]),
        maxiter=int(fem["maximum_iterations"]),
    )
    order = np.argsort(eigenvalues)
    eigenvalues = np.asarray(eigenvalues[order], dtype=np.float64)
    reduced_vectors = np.asarray(reduced_vectors[:, order], dtype=np.float64)
    if np.any(~np.isfinite(eigenvalues)) or np.any(eigenvalues <= 0.0):
        raise ValueError("FEM eigensolver returned invalid eigenvalues")
    eigenvectors = np.zeros((stiffness.shape[0], len(eigenvalues)), dtype=np.float64)
    eigenvectors[free_degrees] = reduced_vectors
    residuals = np.empty(len(eigenvalues), dtype=np.float64)
    for mode_index, eigenvalue in enumerate(eigenvalues):
        vector = eigenvectors[:, mode_index]
        left = (stiffness @ vector)[free_degrees]
        right = eigenvalue * (mass @ vector)[free_degrees]
        residuals[mode_index] = float(np.linalg.norm(left - right)) / max(
            float(np.linalg.norm(left)), np.finfo(np.float64).tiny
        )
    displacement = eigenvectors.reshape(len(mesh.nodes), 3, len(eigenvalues))
    physical_vertices = mesh.nodes[outer_nodes]
    profiles = []
    energy_fractions = []
    axisymmetric_residuals = []
    vertex_normals = physical_vertices / semiaxes**2
    vertex_normals /= np.linalg.norm(vertex_normals, axis=1)[:, None]
    for mode_index in range(len(eigenvalues)):
        outer_displacement = displacement[outer_nodes, :, mode_index]
        profile = sample_surface_profile(
            unit_vertices,
            surface_elements,
            physical_vertices,
            outer_displacement,
            directions,
        )
        profiles.append(profile)
        normal_values = np.sum(outer_displacement * vertex_normals, axis=1)
        total_energy = float(np.sum(outer_displacement**2))
        energy_fractions.append(float(np.sum(normal_values**2)) / max(total_energy, 1.0e-30))
        axisymmetric_residuals.append(axisymmetric_degree4_residual(profile, directions))
    return ElasticLevel(
        angular_level=angular_level,
        radial_divisions=radial_divisions,
        mesh=mesh,
        unit_vertices=unit_vertices,
        surface_elements=surface_elements,
        outer_nodes=outer_nodes,
        eigenvalues=eigenvalues,
        eigenvectors=eigenvectors,
        residuals=residuals,
        frequencies_hz=np.sqrt(eigenvalues) / (2.0 * math.pi),
        sample_profiles=np.asarray(profiles),
        surface_normal_energy_fractions=np.asarray(energy_fractions),
        axisymmetric_residuals=np.asarray(axisymmetric_residuals),
    )


def select_modes(manifest: dict[str, Any], levels: list[ElasticLevel]) -> list[SelectedLevel]:
    fem = manifest["fem"]
    fine = levels[-1]
    eligible = np.flatnonzero(
        (fine.surface_normal_energy_fractions >= fem["minimum_surface_normal_energy_fraction"])
        & (
            fine.axisymmetric_residuals
            >= fem["minimum_axisymmetric_degree4_relative_residual"]
        )
    )
    if len(eligible) == 0:
        raise ValueError("fine FEM solve has no radiating non-axisymmetric mode")
    fine_index = int(eligible[0])
    fine_profile = fine.sample_profiles[fine_index].copy()
    sign_index = int(np.argmax(np.abs(fine_profile)))
    fine_sign = 1.0 if fine_profile[sign_index] >= 0.0 else -1.0
    fine_profile *= fine_sign
    selected = []
    for level in levels:
        correlations = np.asarray(
            [mode_correlation(profile, fine_profile) for profile in level.sample_profiles]
        )
        mode_index = int(np.argmax(correlations)) if level is not fine else fine_index
        profile = level.sample_profiles[mode_index].copy()
        sign = 1.0 if float(np.dot(profile, fine_profile)) >= 0.0 else -1.0
        profile *= sign
        peak = float(np.max(np.abs(profile)))
        if peak <= 1.0e-30:
            raise ValueError("selected FEM mode has a silent surface profile")
        selected.append(
            SelectedLevel(
                level=level,
                mode_index=mode_index,
                match_correlation=mode_correlation(profile, fine_profile),
                signed_eigenvector=level.eigenvectors[:, mode_index] * sign,
                normalized_sample_profile=profile / peak,
            )
        )
    return selected


def fem_convergence(selected: list[SelectedLevel]) -> dict[str, Any]:
    coarse, medium, fine = selected
    frequencies = [row.level.frequencies_hz[row.mode_index] for row in selected]
    coarse_medium_frequency = abs(frequencies[1] - frequencies[0]) / frequencies[1]
    medium_fine_frequency = abs(frequencies[2] - frequencies[1]) / frequencies[2]
    coarse_medium_surface = float(
        np.max(np.abs(medium.normalized_sample_profile - coarse.normalized_sample_profile))
    )
    medium_fine_surface = float(
        np.max(np.abs(fine.normalized_sample_profile - medium.normalized_sample_profile))
    )
    return {
        "coarse_to_medium_relative_eigenfrequency_difference": coarse_medium_frequency,
        "medium_to_fine_relative_eigenfrequency_difference": medium_fine_frequency,
        "fine_to_medium_vs_medium_to_coarse_eigenfrequency_difference_ratio": (
            medium_fine_frequency / coarse_medium_frequency
        ),
        "coarse_vs_medium_surface_peak_normalized_difference": coarse_medium_surface,
        "fine_vs_medium_surface_peak_normalized_difference": medium_fine_surface,
        "fine_to_medium_vs_medium_to_coarse_surface_difference_ratio": (
            medium_fine_surface / coarse_medium_surface
        ),
        "fine_vs_medium_surface_profile_correlation": mode_correlation(
            fine.normalized_sample_profile, medium.normalized_sample_profile
        ),
        "minimum_cross_level_mode_match_correlation": min(
            row.match_correlation for row in selected
        ),
    }


def neumann_coefficients(selected: SelectedLevel) -> np.ndarray:
    level = selected.level
    displacement = selected.signed_eigenvector.reshape(len(level.mesh.nodes), 3)
    outer_displacement = displacement[level.outer_nodes]
    physical_vertices = level.mesh.nodes[level.outer_nodes]
    coefficients = []
    for triangle in level.surface_elements:
        normal = outward_triangle_normal(physical_vertices[triangle])
        coefficients.append(float(np.dot(outer_displacement[triangle].mean(axis=0), normal)))
    peak = float(np.max(np.abs(selected.normalized_sample_profile)))
    raw_sample_peak = float(
        np.max(
            np.abs(
                level.sample_profiles[selected.mode_index]
            )
        )
    )
    if peak <= 0.0 or raw_sample_peak <= 1.0e-30:
        raise ValueError("selected FEM mode normalization is invalid")
    return np.asarray(coefficients, dtype=np.complex128) / raw_sample_peak


def solve_bem_level(
    manifest: dict[str, Any],
    selected: SelectedLevel,
    common_frequency_hz: float,
    directions: list[np.ndarray],
) -> dict[str, Any]:
    fixture = manifest["fixture"]
    solver = manifest["bem"]
    semiaxes = np.asarray(fixture["semiaxes_metres"], dtype=np.float64)
    physical_vertices = selected.level.unit_vertices.T * semiaxes[:, None]
    grid = bempp.Grid(physical_vertices, selected.level.surface_elements.T)
    neumann_space = bempp.function_space(grid, "DP", 0)
    dirichlet_space = bempp.function_space(grid, "P", 1)
    coefficients = neumann_coefficients(selected)
    neumann = bempp.GridFunction(neumann_space, coefficients=coefficients)
    area_weighted_mean = complex(
        np.dot(neumann.coefficients, grid.volumes) / np.sum(grid.volumes)
    )
    speed = float(fixture["speed_of_sound_metres_per_second"])
    wave_number = 2.0 * math.pi * common_frequency_hz / speed
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
    listeners = []
    listener_keys = []
    reference_length = float(fixture["reference_length_metres"])
    for radius in fixture["listener_radius_reference_multipliers"]:
        for direction_index, direction in enumerate(directions):
            listeners.append(direction * reference_length * float(radius))
            listener_keys.append((float(radius), direction_index))
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
    conditions = []
    for value, (radius, direction_index) in zip(computed_values, listener_keys, strict=True):
        computed = complex(value)
        conditions.append(
            {
                "wave_number_reference_length": wave_number * reference_length,
                "frequency_hz": common_frequency_hz,
                "listener_radius_reference_multiplier": radius,
                "direction_index": direction_index,
                "computed": {"real": computed.real, "imaginary": computed.imag},
            }
        )
    final_residual = float(residuals[-1]) if residuals else 0.0
    return {
        "angular_refinement_level": selected.level.angular_level,
        "panel_count": grid.number_of_elements,
        "condition_count": len(conditions),
        "area_weighted_neumann_mean": {
            "real": area_weighted_mean.real,
            "imaginary": area_weighted_mean.imag,
            "magnitude": abs(area_weighted_mean),
        },
        "gmres_info": int(info),
        "gmres_iteration_count": int(iteration_count),
        "gmres_final_residual": final_residual,
        "conditions": conditions,
    }


def complex_value(value: dict[str, float]) -> complex:
    return complex(value["real"], value["imaginary"])


def field_convergence(
    manifest: dict[str, Any],
    coarse: dict[str, Any],
    medium: dict[str, Any],
    fine: dict[str, Any],
) -> dict[str, Any]:
    fixture = manifest["fixture"]
    active_threshold = float(fixture["active_field_minimum_peak_ratio"])
    tolerance = float(fixture["field_classification_absolute_tolerance"])
    coarse_medium_errors = []
    medium_fine_errors = []
    active_relative = []
    active_magnitude_db = []
    active_phase_degrees = []
    correlations = []
    active_count = 0
    for radius in fixture["listener_radius_reference_multipliers"]:
        slices = []
        for level in [coarse, medium, fine]:
            slices.append(
                [
                    complex_value(row["computed"])
                    for row in level["conditions"]
                    if row["listener_radius_reference_multiplier"] == float(radius)
                ]
            )
        coarse_values, medium_values, fine_values = slices
        if any(len(values) != 56 for values in slices):
            raise ValueError("FEM/Bempp direction group is incomplete")
        peak = max(abs(value) for value in fine_values)
        if not math.isfinite(peak) or peak <= 1.0e-30:
            raise ValueError("FEM/Bempp fine field has no finite peak")
        for coarse_value, medium_value, fine_value in zip(
            coarse_values, medium_values, fine_values, strict=True
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
                active_phase_degrees.append(
                    abs(math.degrees(math.atan2(math.sin(phase_delta), math.cos(phase_delta))))
                )
        correlations.append(
            {
                "listener_radius_reference_multiplier": float(radius),
                "fine_vs_medium_complex_correlation": mode_correlation(
                    np.asarray(fine_values), np.asarray(medium_values)
                ),
            }
        )
    coarse_medium_median = float(np.median(coarse_medium_errors))
    medium_fine_median = float(np.median(medium_fine_errors))
    return {
        "condition_count": len(medium_fine_errors),
        "active_condition_count": active_count,
        "medium_to_coarse_median_peak_normalized_complex_difference": coarse_medium_median,
        "fine_to_medium_median_peak_normalized_complex_difference": medium_fine_median,
        "fine_to_medium_vs_medium_to_coarse_median_difference_ratio": (
            medium_fine_median / coarse_medium_median
        ),
        "fine_vs_medium_max_peak_normalized_complex_difference": max(medium_fine_errors),
        "fine_vs_medium_max_active_relative_complex_difference": max(active_relative),
        "fine_vs_medium_max_active_absolute_magnitude_difference_db": max(active_magnitude_db),
        "fine_vs_medium_max_active_absolute_phase_difference_degrees": max(
            active_phase_degrees
        ),
        "fine_vs_medium_min_directional_complex_correlation": min(
            row["fine_vs_medium_complex_correlation"] for row in correlations
        ),
        "directional_correlations": correlations,
    }


def level_report(selected: SelectedLevel) -> dict[str, Any]:
    level = selected.level
    return {
        "angular_refinement_level": level.angular_level,
        "radial_divisions": level.radial_divisions,
        "node_count": len(level.mesh.nodes),
        "tetrahedron_count": len(level.mesh.tetrahedra),
        "fixed_node_count": len(level.mesh.fixed_nodes),
        "surface_panel_count": len(level.surface_elements),
        "node_sha256": array_sha256(level.mesh.nodes),
        "tetrahedron_sha256": array_sha256(level.mesh.tetrahedra),
        "eigenvalue_sha256": array_sha256(level.eigenvalues),
        "eigenvector_sha256": array_sha256(level.eigenvectors),
        "selected_mode_index": selected.mode_index,
        "selected_frequency_hz": float(level.frequencies_hz[selected.mode_index]),
        "selected_relative_eigen_residual": float(level.residuals[selected.mode_index]),
        "maximum_solved_relative_eigen_residual": float(np.max(level.residuals)),
        "selected_surface_normal_energy_fraction": float(
            level.surface_normal_energy_fractions[selected.mode_index]
        ),
        "selected_axisymmetric_degree4_relative_residual": float(
            level.axisymmetric_residuals[selected.mode_index]
        ),
        "fine_mode_match_correlation": selected.match_correlation,
        "normalized_surface_normal_samples": selected.normalized_sample_profile.tolist(),
        "solved_frequencies_hz": level.frequencies_hz.tolist(),
        "solved_relative_eigen_residuals": level.residuals.tolist(),
    }


def build_report(
    manifest: dict[str, Any], environment: dict[str, str], manifest_sha256: str
) -> dict[str, Any]:
    fixture = manifest["fixture"]
    directions = listener_directions(fixture)
    levels = [
        solve_elastic_level(manifest, angular, radial, directions)
        for angular, radial in zip(
            manifest["fem"]["angular_refinement_levels"],
            manifest["fem"]["radial_divisions"],
            strict=True,
        )
    ]
    selected = select_modes(manifest, levels)
    fem_metrics = fem_convergence(selected)
    common_frequency_hz = float(
        selected[-1].level.frequencies_hz[selected[-1].mode_index]
    )
    bem_levels = [
        solve_bem_level(manifest, row, common_frequency_hz, directions) for row in selected
    ]
    bem_metrics = field_convergence(manifest, *bem_levels)
    thresholds = manifest["admission_gates"]
    maximum_eigen_residual = max(float(np.max(level.residuals)) for level in levels)
    gate = {
        "maximum_relative_eigen_residual_passed": maximum_eigen_residual
        <= thresholds["maximum_relative_eigen_residual"],
        "fine_to_medium_eigenfrequency_passed": fem_metrics[
            "medium_to_fine_relative_eigenfrequency_difference"
        ]
        <= thresholds["maximum_fine_to_medium_relative_eigenfrequency_difference"],
        "medium_to_coarse_eigenfrequency_passed": fem_metrics[
            "coarse_to_medium_relative_eigenfrequency_difference"
        ]
        <= thresholds["maximum_medium_to_coarse_relative_eigenfrequency_difference"],
        "eigenfrequency_refinement_passed": fem_metrics[
            "fine_to_medium_vs_medium_to_coarse_eigenfrequency_difference_ratio"
        ]
        <= thresholds[
            "maximum_fine_to_medium_vs_medium_to_coarse_eigenfrequency_difference_ratio"
        ],
        "surface_profile_difference_passed": fem_metrics[
            "fine_vs_medium_surface_peak_normalized_difference"
        ]
        <= thresholds["maximum_fine_vs_medium_surface_peak_normalized_difference"],
        "surface_profile_correlation_passed": fem_metrics[
            "fine_vs_medium_surface_profile_correlation"
        ]
        >= thresholds["minimum_fine_vs_medium_surface_profile_correlation"],
        "surface_profile_refinement_passed": fem_metrics[
            "fine_to_medium_vs_medium_to_coarse_surface_difference_ratio"
        ]
        <= thresholds[
            "maximum_fine_to_medium_vs_medium_to_coarse_surface_difference_ratio"
        ],
        "cross_level_mode_match_passed": fem_metrics[
            "minimum_cross_level_mode_match_correlation"
        ]
        >= thresholds["minimum_cross_level_mode_match_correlation"],
        "every_gmres_info_zero_passed": all(level["gmres_info"] == 0 for level in bem_levels),
        "maximum_final_gmres_residual_passed": max(
            level["gmres_final_residual"] for level in bem_levels
        )
        <= thresholds["maximum_final_gmres_residual"],
        "field_peak_difference_passed": bem_metrics[
            "fine_vs_medium_max_peak_normalized_complex_difference"
        ]
        <= thresholds["fine_vs_medium_max_peak_normalized_complex_difference"],
        "field_active_relative_difference_passed": bem_metrics[
            "fine_vs_medium_max_active_relative_complex_difference"
        ]
        <= thresholds["fine_vs_medium_max_active_relative_complex_difference"],
        "field_active_magnitude_difference_passed": bem_metrics[
            "fine_vs_medium_max_active_absolute_magnitude_difference_db"
        ]
        <= thresholds["fine_vs_medium_max_active_absolute_magnitude_difference_db"],
        "field_active_phase_difference_passed": bem_metrics[
            "fine_vs_medium_max_active_absolute_phase_difference_degrees"
        ]
        <= thresholds["fine_vs_medium_max_active_absolute_phase_difference_degrees"],
        "field_directional_correlation_passed": bem_metrics[
            "fine_vs_medium_min_directional_complex_correlation"
        ]
        >= thresholds["fine_vs_medium_min_directional_complex_correlation"],
        "field_refinement_passed": bem_metrics[
            "fine_to_medium_vs_medium_to_coarse_median_difference_ratio"
        ]
        <= thresholds["fine_to_medium_vs_medium_to_coarse_median_difference_ratio"],
    }
    gate = {name: bool(value) for name, value in gate.items()}
    passed = all(gate.values())
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": (
            "ElasticFemEigenmodeToBemppCouplingSupported"
            if passed
            else "ElasticFemEigenmodeToBemppCouplingRejected"
        ),
        "claim": (
            "SYNTHETIC_CORE_CLAMPED_LINEAR_ELASTIC_FEM_MODE_TO_BEMPP_COUPLING_ONLY / "
            "NO_REAL_MATERIAL_OBJECT_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT"
        ),
        "study_id": manifest["study_id"],
        "protocol_revision": manifest["protocol_revision"],
        "manifest_sha256": manifest_sha256,
        "environment": environment,
        "common_frequency_hz": common_frequency_hz,
        "common_wave_number_reference_length": (
            2.0
            * math.pi
            * common_frequency_hz
            * float(fixture["reference_length_metres"])
            / float(fixture["speed_of_sound_metres_per_second"])
        ),
        "listener_directions": [direction.tolist() for direction in directions],
        "fem_levels": [level_report(row) for row in selected],
        "fem_convergence": fem_metrics,
        "bem_coarse": bem_levels[0],
        "bem_medium": bem_levels[1],
        "bem_fine": bem_levels[2],
        "bem_convergence": bem_metrics,
        "thresholds": thresholds,
        "gate": gate,
        "allowed_claims": manifest["allowed_claims"],
        "prohibited_claims": manifest["prohibited_claims"],
        "data_policy": manifest["data_policy"],
        "source_lineage": manifest["source_lineage"],
        "next_action": (
            "freeze this byte-exact FEM-derived fine field as the source for the "
            "full-near-shell cooker before any fresh REALIMPACT payload"
            if passed
            else "keep REALIMPACT sealed and diagnose FEM mode tracking, mesh "
            "refinement or normal-profile projection on the unchanged fixture"
        ),
    }


def encode_report(report: dict[str, Any]) -> bytes:
    return (json.dumps(report, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def publish(output: Path, manifest: bytes, report: bytes) -> None:
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-bempp-fem-mode-", dir=output.parent))
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
    manifest, manifest_bytes, manifest_sha256 = load_manifest(manifest_path)
    environment = validate_environment(manifest)
    report = build_report(manifest, environment, manifest_sha256)
    report_bytes = encode_report(report)
    publish(output, manifest_bytes, report_bytes)
    print(f"physical sound FEM-to-Bempp mode: {output}")
    print(f"manifest sha256: {manifest_sha256}")
    print(f"selected modes: {[row['selected_mode_index'] for row in report['fem_levels']]}")
    print(
        "selected frequencies Hz: "
        f"{[row['selected_frequency_hz'] for row in report['fem_levels']]}"
    )
    print(f"common kL: {report['common_wave_number_reference_length']:.9f}")
    print(
        "fine-vs-medium field max peak difference: "
        f"{report['bem_convergence']['fine_vs_medium_max_peak_normalized_complex_difference']:.9f}"
    )
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256(report_bytes)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
