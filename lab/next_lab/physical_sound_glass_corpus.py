"""Build an external exact-geometry glass-impact corpus.

The experiment uses a deterministic structured tetrahedral mesh and a small
linear-elastic FEM generalized eigenproblem.  It emits modal evidence only to
an external directory; it does not create runtime content or identify a real
glass material from recordings.
"""

from __future__ import annotations

import hashlib
import json
import math
import platform
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import scipy
from scipy import sparse
from scipy.sparse.linalg import eigsh


RECIPE_SCHEMA = "nextengine.experimental-physical-sound-glass-corpus.recipe.v0"
PROFILE_SCHEMA = "nextengine.external-controlled-glass-modal-corpus.v0"
SOLVER_ID = "nextengine.clean-room-linear-tetrahedral-elasticity.v0"
MAX_TETRAHEDRA = 100_000
MAX_DEGREES_OF_FREEDOM = 150_000
POINT_TOLERANCE_M = 1.0e-10


@dataclass(frozen=True)
class Mesh:
    nodes: np.ndarray
    tetrahedra: np.ndarray
    fixed_nodes: np.ndarray


@dataclass(frozen=True)
class ModalSolution:
    undamped_frequency_hz: np.ndarray
    eigenvectors: np.ndarray
    relative_residuals: np.ndarray


def _canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def _sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _require_mapping(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{name} must be an object")
    return value


def _positive_float(value: Any, name: str) -> float:
    result = float(value)
    if not math.isfinite(result) or result <= 0.0:
        raise ValueError(f"{name} must be positive and finite")
    return result


def _integer_ratio(numerator: float, denominator: float, name: str) -> int:
    ratio = numerator / denominator
    rounded = round(ratio)
    if rounded <= 0 or not math.isclose(ratio, rounded, rel_tol=0.0, abs_tol=1.0e-10):
        raise ValueError(f"{name} must be an exact positive cell multiple")
    return int(rounded)


def _vector3(value: Any, name: str, *, unit: bool = False) -> np.ndarray:
    vector = np.asarray(value, dtype=np.float64)
    if vector.shape != (3,) or not np.all(np.isfinite(vector)):
        raise ValueError(f"{name} must contain three finite values")
    if unit and not math.isclose(float(np.linalg.norm(vector)), 1.0, abs_tol=1.0e-12):
        raise ValueError(f"{name} must be a unit vector")
    return vector


def load_recipe(path: Path) -> tuple[dict[str, Any], bytes]:
    data = path.read_bytes()
    recipe = _require_mapping(json.loads(data), "recipe")
    if recipe.get("schema") != RECIPE_SCHEMA:
        raise ValueError(f"unsupported glass-corpus recipe schema: {recipe.get('schema')}")
    if recipe.get("claim") != (
        "EXTERNAL_CONTROLLED_P0_ONLY / NO_PHYSICAL_IDENTIFICATION_OR_P1_PROMOTION"
    ):
        raise ValueError("glass-corpus recipe must retain the external P0 claim")
    if not isinstance(recipe.get("object_id"), str) or not recipe["object_id"]:
        raise ValueError("glass-corpus object_id must be non-empty")
    return recipe, data


def build_structured_vessel_mesh(recipe: dict[str, Any]) -> Mesh:
    geometry = _require_mapping(recipe.get("geometry"), "geometry")
    if geometry.get("kind") != "open_rectangular_vessel":
        raise ValueError("only the exact open_rectangular_vessel geometry is supported")
    outer = _vector3(geometry.get("outer_size_m"), "geometry.outer_size_m")
    step = _positive_float(geometry.get("cell_size_m"), "geometry.cell_size_m")
    wall = _integer_ratio(
        _positive_float(geometry.get("wall_thickness_m"), "geometry.wall_thickness_m"),
        step,
        "wall thickness",
    )
    bottom = _integer_ratio(
        _positive_float(
            geometry.get("bottom_thickness_m"), "geometry.bottom_thickness_m"
        ),
        step,
        "bottom thickness",
    )
    nx, ny, nz = (
        _integer_ratio(float(outer[index]), step, f"outer_size_m[{index}]")
        for index in range(3)
    )
    if wall * 2 >= min(nx, nz) or bottom >= ny:
        raise ValueError("vessel wall or bottom consumes the complete geometry")
    if geometry.get("tetrahedra_per_cell") != 6:
        raise ValueError("the v0 deterministic cell split requires six tetrahedra")

    node_map: dict[tuple[int, int, int], int] = {}
    node_values: list[tuple[float, float, float]] = []
    tetrahedra: list[tuple[int, int, int, int]] = []

    def node(i: int, j: int, k: int) -> int:
        key = (i, j, k)
        existing = node_map.get(key)
        if existing is not None:
            return existing
        index = len(node_values)
        node_map[key] = index
        node_values.append(((i - nx / 2) * step, j * step, (k - nz / 2) * step))
        return index

    split = (
        (0, 1, 3, 7),
        (0, 3, 2, 7),
        (0, 2, 6, 7),
        (0, 6, 4, 7),
        (0, 4, 5, 7),
        (0, 5, 1, 7),
    )
    for i in range(nx):
        for j in range(ny):
            for k in range(nz):
                cavity = (
                    j >= bottom
                    and wall <= i < nx - wall
                    and wall <= k < nz - wall
                )
                if cavity:
                    continue
                vertices = (
                    node(i, j, k),
                    node(i + 1, j, k),
                    node(i, j + 1, k),
                    node(i + 1, j + 1, k),
                    node(i, j, k + 1),
                    node(i + 1, j, k + 1),
                    node(i, j + 1, k + 1),
                    node(i + 1, j + 1, k + 1),
                )
                tetrahedra.extend(tuple(vertices[index] for index in row) for row in split)

    nodes = np.asarray(node_values, dtype="<f8")
    tetrahedron_array = np.asarray(tetrahedra, dtype="<i4")
    if len(tetrahedron_array) == 0 or len(tetrahedron_array) > MAX_TETRAHEDRA:
        raise ValueError("generated tetrahedron count is empty or exceeds the P0 bound")
    if len(nodes) * 3 > MAX_DEGREES_OF_FREEDOM:
        raise ValueError("generated degrees of freedom exceed the P0 bound")

    boundary = _require_mapping(recipe.get("boundary"), "boundary")
    if boundary.get("kind") != "fixed_bottom_back_left_patch":
        raise ValueError("the v0 corpus requires the declared fixed support patch")
    patch = _positive_float(boundary.get("patch_size_m"), "boundary.patch_size_m")
    minimum = nodes.min(axis=0)
    fixed_nodes = np.flatnonzero(
        np.isclose(nodes[:, 1], 0.0, atol=POINT_TOLERANCE_M)
        & (nodes[:, 0] <= minimum[0] + patch + POINT_TOLERANCE_M)
        & (nodes[:, 2] <= minimum[2] + patch + POINT_TOLERANCE_M)
    ).astype("<i4")
    if len(fixed_nodes) < 4:
        raise ValueError("fixed support patch must contain at least four mesh nodes")
    return Mesh(nodes=nodes, tetrahedra=tetrahedron_array, fixed_nodes=fixed_nodes)


def _elasticity_matrix(youngs_modulus: float, poisson_ratio: float) -> np.ndarray:
    lame_lambda = (
        youngs_modulus
        * poisson_ratio
        / ((1.0 + poisson_ratio) * (1.0 - 2.0 * poisson_ratio))
    )
    shear_modulus = youngs_modulus / (2.0 * (1.0 + poisson_ratio))
    matrix = np.diag(
        [
            lame_lambda + 2.0 * shear_modulus,
            lame_lambda + 2.0 * shear_modulus,
            lame_lambda + 2.0 * shear_modulus,
            shear_modulus,
            shear_modulus,
            shear_modulus,
        ]
    )
    matrix[0, 1] = matrix[0, 2] = lame_lambda
    matrix[1, 0] = matrix[1, 2] = lame_lambda
    matrix[2, 0] = matrix[2, 1] = lame_lambda
    return matrix


def assemble_linear_elasticity(
    mesh: Mesh, material: dict[str, Any]
) -> tuple[sparse.csr_matrix, sparse.csr_matrix]:
    density = _positive_float(material.get("density_kg_m3"), "material.density_kg_m3")
    youngs = _positive_float(
        material.get("youngs_modulus_pa"), "material.youngs_modulus_pa"
    )
    poisson = float(material.get("poisson_ratio"))
    if not math.isfinite(poisson) or not 0.0 < poisson < 0.49:
        raise ValueError("material.poisson_ratio must be finite and in (0, 0.49)")
    elasticity = _elasticity_matrix(youngs, poisson)

    local_entry_count = 12 * 12
    entry_count = len(mesh.tetrahedra) * local_entry_count
    rows = np.empty(entry_count, dtype=np.int32)
    columns = np.empty(entry_count, dtype=np.int32)
    stiffness_values = np.empty(entry_count, dtype=np.float64)
    mass_values = np.empty(entry_count, dtype=np.float64)
    offset = 0
    for tetrahedron in mesh.tetrahedra:
        coordinates = mesh.nodes[tetrahedron]
        affine = np.ones((4, 4), dtype=np.float64)
        affine[:, 1:] = coordinates
        gradients = np.linalg.inv(affine)[1:, :].T
        volume = abs(float(np.linalg.det(coordinates[1:] - coordinates[0]))) / 6.0
        if not math.isfinite(volume) or volume <= 0.0:
            raise ValueError("mesh contains a degenerate tetrahedron")

        strain = np.zeros((6, 12), dtype=np.float64)
        for node_index, (gradient_x, gradient_y, gradient_z) in enumerate(gradients):
            dof = 3 * node_index
            strain[0, dof] = gradient_x
            strain[1, dof + 1] = gradient_y
            strain[2, dof + 2] = gradient_z
            strain[3, dof] = gradient_y
            strain[3, dof + 1] = gradient_x
            strain[4, dof + 1] = gradient_z
            strain[4, dof + 2] = gradient_y
            strain[5, dof] = gradient_z
            strain[5, dof + 2] = gradient_x
        local_stiffness = volume * (strain.T @ elasticity @ strain)

        local_mass = np.zeros((12, 12), dtype=np.float64)
        mass_scale = density * volume / 20.0
        for left in range(4):
            for right in range(4):
                coefficient = mass_scale * (2.0 if left == right else 1.0)
                for axis in range(3):
                    local_mass[3 * left + axis, 3 * right + axis] = coefficient

        degrees = np.asarray(
            [[3 * node, 3 * node + 1, 3 * node + 2] for node in tetrahedron],
            dtype=np.int32,
        ).reshape(-1)
        next_offset = offset + local_entry_count
        rows[offset:next_offset] = np.repeat(degrees, 12)
        columns[offset:next_offset] = np.tile(degrees, 12)
        stiffness_values[offset:next_offset] = local_stiffness.reshape(-1)
        mass_values[offset:next_offset] = local_mass.reshape(-1)
        offset = next_offset

    degree_count = len(mesh.nodes) * 3
    stiffness = sparse.coo_matrix(
        (stiffness_values, (rows, columns)), shape=(degree_count, degree_count)
    ).tocsr()
    mass = sparse.coo_matrix(
        (mass_values, (rows, columns)), shape=(degree_count, degree_count)
    ).tocsr()
    return stiffness, mass


def solve_modes(
    mesh: Mesh,
    stiffness: sparse.csr_matrix,
    mass: sparse.csr_matrix,
    solver: dict[str, Any],
) -> ModalSolution:
    solve_count = int(solver.get("eigensolve_mode_count"))
    retained_count = int(solver.get("retained_mode_count"))
    if solve_count < retained_count or retained_count <= 0 or solve_count > 128:
        raise ValueError("solver mode counts are invalid or unbounded")
    fixed_degrees = np.sort(
        np.concatenate([3 * mesh.fixed_nodes + axis for axis in range(3)])
    )
    free_degrees = np.setdiff1d(
        np.arange(stiffness.shape[0], dtype=np.int64), fixed_degrees
    )
    reduced_stiffness = stiffness[free_degrees][:, free_degrees]
    reduced_mass = mass[free_degrees][:, free_degrees]
    start_vector = np.linspace(1.0, 2.0, len(free_degrees), dtype=np.float64)
    start_vector /= np.linalg.norm(start_vector)
    eigenvalues, reduced_vectors = eigsh(
        reduced_stiffness,
        k=solve_count,
        M=reduced_mass,
        sigma=0.0,
        which="LM",
        v0=start_vector,
        tol=_positive_float(solver.get("relative_tolerance"), "solver.relative_tolerance"),
        maxiter=int(solver.get("maximum_iterations")),
    )
    order = np.argsort(eigenvalues)
    eigenvalues = eigenvalues[order]
    reduced_vectors = reduced_vectors[:, order]
    if np.any(~np.isfinite(eigenvalues)) or np.any(eigenvalues <= 0.0):
        raise ValueError("eigensolver returned non-positive or non-finite eigenvalues")
    frequencies = np.sqrt(eigenvalues) / (2.0 * math.pi)
    frequency_minimum = _positive_float(
        solver.get("minimum_frequency_hz"), "solver.minimum_frequency_hz"
    )
    frequency_maximum = _positive_float(
        solver.get("maximum_frequency_hz"), "solver.maximum_frequency_hz"
    )
    selected = np.flatnonzero(
        (frequencies >= frequency_minimum) & (frequencies < frequency_maximum)
    )[:retained_count]
    if len(selected) != retained_count:
        raise ValueError("eigensolver did not produce the requested in-band mode count")

    eigenvectors = np.zeros((stiffness.shape[0], retained_count), dtype=np.float64)
    eigenvectors[free_degrees] = reduced_vectors[:, selected]
    selected_values = eigenvalues[selected]
    residuals = np.empty(retained_count, dtype=np.float64)
    for mode_index in range(retained_count):
        vector = eigenvectors[:, mode_index]
        largest = int(np.argmax(np.abs(vector)))
        if vector[largest] < 0.0:
            vector *= -1.0
        left = (stiffness @ vector)[free_degrees]
        right = selected_values[mode_index] * (mass @ vector)[free_degrees]
        denominator = max(float(np.linalg.norm(left)), np.finfo(np.float64).tiny)
        residuals[mode_index] = float(np.linalg.norm(left - right)) / denominator
    maximum_residual = float(np.max(residuals))
    if maximum_residual > 1.0e-5:
        raise ValueError(
            "retained modal residual exceeds the P0 evidence bound: "
            f"{maximum_residual:.9g} > 1e-5"
        )
    return ModalSolution(
        undamped_frequency_hz=frequencies[selected],
        eigenvectors=eigenvectors,
        relative_residuals=residuals,
    )


def _exact_node(mesh: Mesh, point: np.ndarray, name: str) -> int:
    distances = np.linalg.norm(mesh.nodes - point, axis=1)
    index = int(np.argmin(distances))
    if float(distances[index]) > POINT_TOLERANCE_M:
        raise ValueError(f"{name} does not resolve to an exact mesh node")
    return index


def _modal_basis(
    damped_frequency_hz: np.ndarray,
    damping_per_second: np.ndarray,
    sample_rate_hz: int,
    frame_count: int,
) -> np.ndarray:
    time = np.arange(frame_count, dtype=np.float64) / sample_rate_hz
    return np.exp(-damping_per_second[:, None] * time[None, :]) * np.sin(
        2.0 * math.pi * damped_frequency_hz[:, None] * time[None, :]
    )


def build_solver_profile(
    recipe: dict[str, Any],
    recipe_bytes: bytes,
    mesh: Mesh,
    solution: ModalSolution,
    artifact_hashes: dict[str, str],
) -> dict[str, Any]:
    material = _require_mapping(recipe.get("material"), "material")
    damping = _require_mapping(material.get("damping"), "material.damping")
    if damping.get("kind") != "explicit_frequency_calibration":
        raise ValueError("the v0 corpus requires explicit frequency damping")
    damping_per_second = _positive_float(
        damping.get("base_per_second"), "damping.base_per_second"
    ) + _positive_float(damping.get("slope_per_hz"), "damping.slope_per_hz") * (
        solution.undamped_frequency_hz
    )
    angular = 2.0 * math.pi * solution.undamped_frequency_hz
    if np.any(damping_per_second >= angular):
        raise ValueError("damping calibration makes at least one retained mode overdamped")
    damped_frequency = np.sqrt(angular * angular - damping_per_second**2) / (
        2.0 * math.pi
    )

    pickup = _require_mapping(recipe.get("pickup"), "pickup")
    pickup_point = _vector3(pickup.get("point_m"), "pickup.point_m")
    pickup_direction = _vector3(
        pickup.get("direction"), "pickup.direction", unit=True
    )
    pickup_node = _exact_node(mesh, pickup_point, "pickup.point_m")
    pickup_displacement = np.asarray(
        [
            np.dot(
                solution.eigenvectors[3 * pickup_node : 3 * pickup_node + 3, mode],
                pickup_direction,
            )
            for mode in range(len(solution.undamped_frequency_hz))
        ]
    )

    strikes = recipe.get("strikes")
    impulses = recipe.get("impulses")
    if not isinstance(strikes, list) or len(strikes) < 3:
        raise ValueError("recipe must contain at least three strikes")
    if not isinstance(impulses, list) or len(impulses) != 3:
        raise ValueError("recipe must contain exactly three impulse levels")
    raw_participation: dict[str, np.ndarray] = {}
    strike_reports: list[dict[str, Any]] = []
    for strike in strikes:
        strike = _require_mapping(strike, "strike")
        strike_id = str(strike.get("id"))
        role = str(strike.get("role"))
        if role not in {"train", "heldout"} or strike_id in raw_participation:
            raise ValueError("strike role or identity is invalid")
        point = _vector3(strike.get("point_m"), f"strike {strike_id} point")
        direction = _vector3(
            strike.get("direction"), f"strike {strike_id} direction", unit=True
        )
        node_index = _exact_node(mesh, point, f"strike {strike_id}")
        strike_displacement = np.asarray(
            [
                np.dot(
                    solution.eigenvectors[3 * node_index : 3 * node_index + 3, mode],
                    direction,
                )
                for mode in range(len(solution.undamped_frequency_hz))
            ]
        )
        participation = (
            strike_displacement
            * pickup_displacement
            / (2.0 * math.pi * damped_frequency)
        )
        if not np.any(np.abs(participation) > np.finfo(np.float64).tiny):
            raise ValueError(f"strike {strike_id} has a silent participation vector")
        raw_participation[strike_id] = participation
        strike_reports.append(
            {
                "id": strike_id,
                "role": role,
                "point_m": point.tolist(),
                "direction": direction.tolist(),
                "mesh_node_index": node_index,
            }
        )

    render = _require_mapping(recipe.get("render"), "render")
    sample_rate_hz = int(render.get("sample_rate_hz"))
    duration_seconds = _positive_float(
        render.get("duration_seconds"), "render.duration_seconds"
    )
    frame_count_float = sample_rate_hz * duration_seconds
    frame_count = round(frame_count_float)
    if sample_rate_hz != 48_000 or not math.isclose(
        frame_count_float, frame_count, abs_tol=1.0e-9
    ):
        raise ValueError("v0 render must be an exact-duration 48 kHz window")
    basis = _modal_basis(
        damped_frequency, damping_per_second, sample_rate_hz, frame_count
    )
    impulse_rows: list[tuple[str, float]] = []
    for impulse in impulses:
        impulse = _require_mapping(impulse, "impulse")
        impulse_id = str(impulse.get("id"))
        value = _positive_float(impulse.get("newton_seconds"), f"impulse {impulse_id}")
        impulse_rows.append((impulse_id, value))
    if [row[0] for row in impulse_rows] != ["low", "medium", "high"]:
        raise ValueError("v0 impulse IDs must be sorted low, medium, high")
    if not impulse_rows[0][1] < impulse_rows[1][1] < impulse_rows[2][1]:
        raise ValueError("v0 impulse levels must be strictly increasing")

    maximum_peak = 0.0
    for participation in raw_participation.values():
        for _, impulse in impulse_rows:
            maximum_peak = max(
                maximum_peak, float(np.max(np.abs((participation * impulse) @ basis)))
            )
    target_peak = _positive_float(render.get("target_global_peak"), "render.target_global_peak")
    if target_peak >= 0.95 or maximum_peak <= np.finfo(np.float64).tiny:
        raise ValueError("render peak calibration is invalid")
    amplitude_gain = target_peak / maximum_peak

    heldout = _require_mapping(recipe.get("heldout"), "heldout")
    position_holdout_id = str(heldout.get("position_holdout_strike_id"))
    force_holdout_id = str(heldout.get("force_holdout_id"))
    train_force_ids = heldout.get("train_force_ids")
    if (
        position_holdout_id not in raw_participation
        or train_force_ids != ["low", "high"]
        or force_holdout_id != "medium"
        or heldout.get("position_interpolator") != "inverse_distance_squared"
    ):
        raise ValueError("heldout protocol does not match the closed v0 split")

    strike_role = {row["id"]: row["role"] for row in strike_reports}
    conditions: list[dict[str, Any]] = []
    for strike in strike_reports:
        for impulse_id, impulse in impulse_rows:
            role = strike_role[strike["id"]]
            if role == "train" and impulse_id in train_force_ids:
                split = "train"
            elif role == "train":
                split = "force-holdout"
            elif impulse_id in train_force_ids:
                split = "position-holdout"
            else:
                split = "joint-holdout"
            amplitudes = raw_participation[strike["id"]] * impulse * amplitude_gain
            conditions.append(
                {
                    "id": f"{strike['id']}--{impulse_id}",
                    "split": split,
                    "strike_id": strike["id"],
                    "force_id": impulse_id,
                    "impulse_newton_seconds": impulse,
                    "amplitudes": amplitudes.tolist(),
                }
            )
    conditions.sort(key=lambda row: row["id"])

    modes = []
    for mode_index in range(len(solution.undamped_frequency_hz)):
        decay = float(damping_per_second[mode_index])
        modes.append(
            {
                "index": mode_index,
                "undamped_frequency_hz": float(
                    solution.undamped_frequency_hz[mode_index]
                ),
                "damped_frequency_hz": float(damped_frequency[mode_index]),
                "damping_per_second": decay,
                "t60_seconds": math.log(1000.0) / decay,
                "relative_eigen_residual": float(
                    solution.relative_residuals[mode_index]
                ),
            }
        )

    geometry = _require_mapping(recipe.get("geometry"), "geometry")
    return {
        "schema": PROFILE_SCHEMA,
        "claim": recipe["claim"],
        "object_id": recipe["object_id"],
        "source_recipe": {
            "file": "source-recipe.json",
            "sha256": _sha256_bytes(recipe_bytes),
        },
        "solver": {
            "id": SOLVER_ID,
            "python": platform.python_version(),
            "numpy": np.__version__,
            "scipy": scipy.__version__,
            "method": "K phi = lambda M phi; first-order tetrahedral linear elasticity; consistent mass; deterministic ARPACK start vector",
        },
        "geometry": {
            **geometry,
            "node_count": len(mesh.nodes),
            "tetrahedron_count": len(mesh.tetrahedra),
            "fixed_node_count": len(mesh.fixed_nodes),
            "node_file": "nodes.npy",
            "node_sha256": artifact_hashes["nodes.npy"],
            "tetrahedron_file": "tetrahedra.npy",
            "tetrahedron_sha256": artifact_hashes["tetrahedra.npy"],
            "fixed_node_file": "fixed-nodes.npy",
            "fixed_node_sha256": artifact_hashes["fixed-nodes.npy"],
        },
        "material": material,
        "boundary": recipe["boundary"],
        "modal_basis": {
            "file": "eigenvectors.npy",
            "sha256": artifact_hashes["eigenvectors.npy"],
            "normalization": "scipy generalized eigensolver M-normalization",
            "sign_rule": "largest-absolute full-vector DOF is positive",
        },
        "pickup": {
            **pickup,
            "mesh_node_index": pickup_node,
        },
        "sample_rate_hz": sample_rate_hz,
        "frame_count": frame_count,
        "global_amplitude_gain": amplitude_gain,
        "target_global_peak": target_peak,
        "modes": modes,
        "strikes": strike_reports,
        "impulses": [
            {"id": impulse_id, "newton_seconds": value}
            for impulse_id, value in impulse_rows
        ],
        "heldout": heldout,
        "conditions": conditions,
    }


def solve_controlled_glass_corpus(profile_path: Path, output: Path) -> tuple[dict[str, Any], Path]:
    repository_root = Path(__file__).resolve().parents[2]
    output = output.resolve()
    if output == repository_root or repository_root in output.parents:
        raise ValueError("physical-sound corpus artifacts must stay outside the repository")
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError("physical-sound corpus output must be a missing or empty directory")
    output.mkdir(parents=True, exist_ok=True)

    recipe, recipe_bytes = load_recipe(profile_path)
    mesh = build_structured_vessel_mesh(recipe)
    material = _require_mapping(recipe.get("material"), "material")
    stiffness, mass = assemble_linear_elasticity(mesh, material)
    solution = solve_modes(
        mesh, stiffness, mass, _require_mapping(recipe.get("solver"), "solver")
    )

    (output / "source-recipe.json").write_bytes(recipe_bytes)
    np.save(output / "nodes.npy", mesh.nodes, allow_pickle=False)
    np.save(output / "tetrahedra.npy", mesh.tetrahedra, allow_pickle=False)
    np.save(output / "fixed-nodes.npy", mesh.fixed_nodes, allow_pickle=False)
    np.save(output / "eigenvectors.npy", solution.eigenvectors, allow_pickle=False)
    artifact_hashes = {
        name: _sha256_file(output / name)
        for name in [
            "nodes.npy",
            "tetrahedra.npy",
            "fixed-nodes.npy",
            "eigenvectors.npy",
        ]
    }
    profile = build_solver_profile(recipe, recipe_bytes, mesh, solution, artifact_hashes)
    profile_bytes = _canonical_json_bytes(profile)
    profile_path_out = output / "solver-profile.json"
    profile_path_out.write_bytes(profile_bytes)
    summary = {
        "check": "PHYSICAL-SOUND-CONTROLLED-GLASS-CORPUS-SOLVE-P0",
        "status": "PASS",
        "claim": recipe["claim"],
        "output": str(output),
        "solver_profile": str(profile_path_out),
        "solver_profile_sha256": _sha256_bytes(profile_bytes),
        "node_count": len(mesh.nodes),
        "tetrahedron_count": len(mesh.tetrahedra),
        "mode_count": len(solution.undamped_frequency_hz),
        "minimum_frequency_hz": float(solution.undamped_frequency_hz[0]),
        "maximum_frequency_hz": float(solution.undamped_frequency_hz[-1]),
        "maximum_relative_eigen_residual": float(
            np.max(solution.relative_residuals)
        ),
    }
    return summary, profile_path_out


def default_recipe_path() -> Path:
    return Path(__file__).resolve().parents[1] / "profiles" / (
        "physical-sound-glass-vessel-corpus.v0.json"
    )


if __name__ == "__main__":
    raise SystemExit("use python -m next_lab physical-sound-glass-corpus-solve")
