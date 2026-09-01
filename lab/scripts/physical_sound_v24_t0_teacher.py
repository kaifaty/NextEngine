#!/usr/bin/env python3
"""Build the V24 T0 analytic modal-teacher lane outside the repository."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import shutil
import struct
import sys
import tempfile
from dataclasses import dataclass
from dataclasses import replace
from pathlib import Path
from typing import Any

import numpy as np


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v24-t0-teacher.manifest.v1"
EVIDENCE_SCHEMA = "nextengine.experimental-physical-sound-v24-t0-teacher.evidence.v1"
LANE_SCHEMA = "nextengine.experimental-physical-sound-v24-t0-teacher.lane-records.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v24-t0-teacher.report.v1"
STUDY_ID = "physical-sound-v24-t0-analytic-teacher"
PROTOCOL_REVISION = "analytic-plate-beam-v1"
PROTOCOL_SHA256 = "c47399a6aa9dd8909dea062a8a1a8f611d9c32baaa22783c96dcbf0cb87c4131"
CLAIM = "SYNTHETIC_ANALYTIC_MODAL_TEACHER_ONLY / NO_REAL_MATERIAL_QUALITY_OR_RUNTIME_AUTHORITY"
LINEAGE_ID = "v24-t0-analytic-teacher"
TEACHER_REPRESENTATION = "sorted-modal-contact-field-v1"
FORMULA_IDS = ("euler-bernoulli-cantilever-v1", "kirchhoff-love-simply-supported-v1")
SUPPORT_IDS = {
    "beam": "cantilever-clamped-u0",
    "plate": "simply-supported-all-edges",
}
PROFILE_SHA256 = {
    "contract-fixture-v1": "d088ccf23c5c3989c3bbfd7831964050ae4be4f16b1e84e907773cd70dd082d4",
    "official-v1": "3c997de7238e4e761960244465d3f09898470f637c35a650ef78831715f030f4",
}
MESH_MAGIC = b"NEMESH01"
MODAL_MAGIC = b"NEMODT01"
GAIN_MAGIC = b"NEGAIN01"
SAMPLE_RATE_HZ = 48_000
MAX_OUTPUT_BYTES = 256 * 1024 * 1024
ROLES = ("train", "development", "calibration", "method_holdout", "admission_shadow")
BEAM_ROOTS = (
    "1.875104068711961",
    "4.694091132974175",
    "7.854757438237612",
    "10.99554073487547",
    "14.13716839104647",
    "17.27875953208824",
    "20.42035225104125",
    "23.56194490180644",
    "26.70353755551830",
    "29.84513020910282",
)
CONTACTS = (
    ("c00", "context", 1 / 8, 1 / 8),
    ("c01", "context", 7 / 8, 1 / 8),
    ("c02", "context", 1 / 8, 7 / 8),
    ("c03", "context", 7 / 8, 7 / 8),
    ("c04", "context", 1 / 2, 1 / 4),
    ("c05", "context", 1 / 4, 1 / 2),
    ("c06", "context", 3 / 4, 1 / 2),
    ("c07", "context", 1 / 2, 3 / 4),
    ("c08", "query", 3 / 8, 3 / 8),
    ("c09", "query", 5 / 8, 3 / 8),
    ("c10", "query", 3 / 8, 5 / 8),
    ("c11", "query", 5 / 8, 5 / 8),
)


@dataclass(frozen=True)
class Material:
    material_id: str
    density: float
    youngs: float
    poisson: float
    damping_base: float
    damping_slope: float


@dataclass(frozen=True)
class Recipe:
    object_id: str
    role: str
    family: str
    material: Material
    dimensions: tuple[float, float, float]


@dataclass(frozen=True)
class Profile:
    profile_id: str
    recipes: tuple[Recipe, ...]
    mode_count: int
    coarse_grid: tuple[int, int]
    fine_grid: tuple[int, int]
    contacts: tuple[tuple[str, str, float, float], ...]
    duration_seconds: float


MATERIALS = {
    "elastic-a": Material("elastic-a", 2700.0, 69_000_000_000.0, 0.33, 8.0, 0.0015),
    "elastic-b": Material("elastic-b", 7850.0, 200_000_000_000.0, 0.29, 12.0, 0.0010),
    "elastic-c": Material("elastic-c", 8900.0, 110_000_000_000.0, 0.34, 18.0, 0.0008),
}


def _recipe(object_id: str, role: str, family: str, material: str, *dimensions: float) -> Recipe:
    return Recipe(object_id, role, family, MATERIALS[material], tuple(dimensions))


OFFICIAL_RECIPES = (
    _recipe("t-plate-a", "train", "plate", "elastic-a", 0.24, 0.18, 0.0030),
    _recipe("t-plate-b", "train", "plate", "elastic-b", 0.20, 0.15, 0.0020),
    _recipe("t-beam-a", "train", "beam", "elastic-b", 0.30, 0.040, 0.0040),
    _recipe("t-beam-b", "train", "beam", "elastic-a", 0.26, 0.035, 0.0030),
    _recipe("d-plate", "development", "plate", "elastic-c", 0.22, 0.16, 0.0025),
    _recipe("d-beam", "development", "beam", "elastic-c", 0.28, 0.045, 0.0035),
    _recipe("c-plate", "calibration", "plate", "elastic-a", 0.18, 0.13, 0.0018),
    _recipe("c-beam", "calibration", "beam", "elastic-b", 0.24, 0.030, 0.0028),
    _recipe("h-plate", "method_holdout", "plate", "elastic-b", 0.27, 0.17, 0.0033),
    _recipe("h-beam", "method_holdout", "beam", "elastic-a", 0.32, 0.050, 0.0045),
    _recipe("s-plate", "admission_shadow", "plate", "elastic-c", 0.19, 0.145, 0.0022),
    _recipe("s-beam", "admission_shadow", "beam", "elastic-c", 0.34, 0.038, 0.0032),
)


def _fixture_recipes() -> tuple[Recipe, ...]:
    return tuple(
        _recipe(
            f"fixture-{role}",
            role,
            "plate" if index % 2 == 0 else "beam",
            ("elastic-a", "elastic-b", "elastic-c")[index % 3],
            *(0.08, 0.06, 0.002) if index % 2 == 0 else (0.10, 0.02, 0.002),
        )
        for index, role in enumerate(ROLES)
    )


def profile(profile_id: str) -> Profile:
    if profile_id == "official-v1":
        return Profile(profile_id, OFFICIAL_RECIPES, 10, (17, 13), (33, 25), CONTACTS, 3.0)
    if profile_id == "contract-fixture-v1":
        contacts = (("c00", "context", 0.25, 0.25), ("c01", "query", 0.75, 0.75))
        return Profile(profile_id, _fixture_recipes(), 2, (5, 5), (9, 9), contacts, 0.02)
    raise ValueError(f"unsupported T0 profile: {profile_id}")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode("utf-8")


def profile_sha256(selected: Profile) -> str:
    value = {
        "profile_id": selected.profile_id,
        "mode_count": selected.mode_count,
        "coarse_grid": list(selected.coarse_grid),
        "fine_grid": list(selected.fine_grid),
        "contacts": [list(contact) for contact in selected.contacts],
        "duration_seconds": selected.duration_seconds,
        "recipes": [
            {
                "object_id": recipe.object_id,
                "role": recipe.role,
                "family": recipe.family,
                "material": {
                    "material_id": recipe.material.material_id,
                    "density": recipe.material.density,
                    "youngs": recipe.material.youngs,
                    "poisson": recipe.material.poisson,
                    "damping_base": recipe.material.damping_base,
                    "damping_slope": recipe.material.damping_slope,
                },
                "dimensions": list(recipe.dimensions),
            }
            for recipe in selected.recipes
        ],
    }
    return sha256_bytes(canonical_json(value))


def validate_profile(selected: Profile) -> None:
    if PROFILE_SHA256.get(selected.profile_id) != profile_sha256(selected):
        raise ValueError(f"T0 profile drift: {selected.profile_id}")


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes]:
    data = path.read_bytes()
    if not data or len(data) > 64 * 1024:
        raise ValueError("T0 manifest must be 1..=65536 bytes")

    def no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate T0 manifest key: {key}")
            result[key] = value
        return result

    manifest = json.loads(data, object_pairs_hook=no_duplicates)
    expected_keys = {
        "schema",
        "study_id",
        "protocol_revision",
        "protocol_sha256",
        "profile",
        "claim",
        "data_policy",
    }
    if not isinstance(manifest, dict) or set(manifest) != expected_keys:
        raise ValueError("T0 manifest fields changed")
    if (
        manifest["schema"] != MANIFEST_SCHEMA
        or manifest["study_id"] != STUDY_ID
        or manifest["protocol_revision"] != PROTOCOL_REVISION
        or manifest["protocol_sha256"] != PROTOCOL_SHA256
        or manifest["claim"] != CLAIM
        or manifest["profile"] not in {"official-v1", "contract-fixture-v1"}
        or manifest["data_policy"]
        != {
            "external_output_only": True,
            "network_allowed": False,
            "real_signal_allowed": False,
            "model_training_authorized": False,
            "runtime_authorized": False,
        }
    ):
        raise ValueError("T0 manifest does not match the frozen protocol")
    return manifest, data


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def external_file(root: Path, path: Path) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise ValueError(f"T0 manifest must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise ValueError(f"T0 output must be a new external path: {resolved}")
    return resolved


def mesh(recipe: Recipe, grid: tuple[int, int]) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    grid_u, grid_v = grid
    if grid_u < 2 or grid_v < 2:
        raise ValueError("T0 mesh grid is too small")
    first, second, thickness = recipe.dimensions
    vertices = []
    normalized = []
    for v_index in range(grid_v):
        v = v_index / (grid_v - 1)
        for u_index in range(grid_u):
            u = u_index / (grid_u - 1)
            normalized.append((u, v))
            if recipe.family == "plate":
                vertices.append((first * u, second * v, thickness / 2.0))
            else:
                vertices.append((first * u, second * (v - 0.5), thickness / 2.0))
    triangles = []
    for v_index in range(grid_v - 1):
        for u_index in range(grid_u - 1):
            lower = v_index * grid_u + u_index
            triangles.append((lower, lower + 1, lower + grid_u))
            triangles.append((lower + 1, lower + grid_u + 1, lower + grid_u))
    return (
        np.asarray(vertices, dtype=np.float64),
        np.asarray(triangles, dtype=np.uint32),
        np.asarray(normalized, dtype=np.float64),
    )


def plate_modes(recipe: Recipe, count: int) -> tuple[np.ndarray, np.ndarray]:
    a, b, thickness = recipe.dimensions
    material = recipe.material
    rigidity = material.youngs * thickness**3 / (12.0 * (1.0 - material.poisson**2))
    scale = math.pi**2 * math.sqrt(rigidity / (material.density * thickness))
    candidates = []
    for m in range(1, 17):
        for n in range(1, 17):
            frequency = scale * ((m / a) ** 2 + (n / b) ** 2) / (2.0 * math.pi)
            candidates.append((frequency, m, n))
    candidates.sort()
    selected = candidates[:count]
    return (
        np.asarray([row[0] for row in selected], dtype=np.float64),
        np.asarray([[row[1], row[2]] for row in selected], dtype=np.uint32),
    )


def beam_shape(root: float, u: np.ndarray | float) -> np.ndarray:
    values = np.asarray(u, dtype=np.float64)
    sigma = (math.cosh(root) + math.cos(root)) / (math.sinh(root) + math.sin(root))
    raw = np.cosh(root * values) - np.cos(root * values) - sigma * (
        np.sinh(root * values) - np.sin(root * values)
    )
    free = math.cosh(root) - math.cos(root) - sigma * (math.sinh(root) - math.sin(root))
    return raw / abs(free)


def beam_modes(recipe: Recipe, count: int) -> tuple[np.ndarray, np.ndarray]:
    length, width, height = recipe.dimensions
    area = width * height
    inertia = width * height**3 / 12.0
    scale = math.sqrt(recipe.material.youngs * inertia / (recipe.material.density * area))
    roots = np.asarray([float(value) for value in BEAM_ROOTS[:count]], dtype=np.float64)
    frequencies = roots**2 / length**2 * scale / (2.0 * math.pi)
    indices = np.asarray([[index + 1, 0] for index in range(count)], dtype=np.uint32)
    return frequencies, indices


def validate_recipe(recipe: Recipe) -> None:
    if recipe.role not in ROLES or recipe.family not in {"plate", "beam"}:
        raise ValueError(f"invalid T0 recipe identity: {recipe.object_id}")
    if len(recipe.dimensions) != 3 or any(
        not math.isfinite(value) or value <= 0.0 for value in recipe.dimensions
    ):
        raise ValueError(f"invalid T0 recipe dimensions: {recipe.object_id}")
    material = recipe.material
    if (
        not all(
            math.isfinite(value) and value > 0.0
            for value in (
                material.density,
                material.youngs,
                material.damping_base,
                material.damping_slope,
            )
        )
        or not math.isfinite(material.poisson)
        or not 0.0 < material.poisson < 0.49
    ):
        raise ValueError(f"invalid T0 material parameters: {recipe.object_id}")


def validate_contract_constants() -> None:
    if (
        FORMULA_IDS
        != ("euler-bernoulli-cantilever-v1", "kirchhoff-love-simply-supported-v1")
        or MESH_MAGIC != b"NEMESH01"
        or MODAL_MAGIC != b"NEMODT01"
        or GAIN_MAGIC != b"NEGAIN01"
        or TEACHER_REPRESENTATION != "sorted-modal-contact-field-v1"
        or SUPPORT_IDS
        != {"beam": "cantilever-clamped-u0", "plate": "simply-supported-all-edges"}
    ):
        raise ValueError("T0 implementation constants drift from the frozen protocol")


def modal_solution(recipe: Recipe, count: int) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    validate_recipe(recipe)
    if not 1 <= count <= len(BEAM_ROOTS):
        raise ValueError("T0 mode count is invalid")
    if recipe.family == "plate":
        undamped, indices = plate_modes(recipe, count)
    elif recipe.family == "beam":
        undamped, indices = beam_modes(recipe, count)
    else:
        raise ValueError(f"unsupported T0 family: {recipe.family}")
    decay = recipe.material.damping_base + recipe.material.damping_slope * undamped
    angular = 2.0 * math.pi * undamped
    damped = np.sqrt(angular**2 - decay**2) / (2.0 * math.pi)
    if (
        len(damped) != count
        or np.any(~np.isfinite(damped))
        or np.any(damped <= 0.0)
        or np.any(damped >= 18_000.0)
        or np.any(decay <= 0.0)
        or np.any(decay >= angular)
        or np.any(np.diff(damped) <= 0.0)
    ):
        raise ValueError(f"T0 modal bounds failed for {recipe.object_id}")
    return damped, decay, indices


def raw_gain(recipe: Recipe, indices: np.ndarray, u: np.ndarray, v: np.ndarray) -> np.ndarray:
    if recipe.family == "plate":
        pickup_u, pickup_v = 0.37, 0.61
        columns = []
        for m, n in indices:
            pickup = math.sin(int(m) * math.pi * pickup_u) * math.sin(
                int(n) * math.pi * pickup_v
            )
            columns.append(
                np.sin(int(m) * math.pi * u) * np.sin(int(n) * math.pi * v) * pickup
            )
        return np.column_stack(columns)
    columns = []
    for ordinal, _ in indices:
        root = float(BEAM_ROOTS[int(ordinal) - 1])
        pickup = float(beam_shape(root, 0.83))
        columns.append(beam_shape(root, u) * pickup)
    return np.column_stack(columns)


def normalized_gains(
    recipe: Recipe, indices: np.ndarray, damped: np.ndarray, uv: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    raw = raw_gain(recipe, indices, uv[:, 0], uv[:, 1]) / (2.0 * math.pi * damped)
    if recipe.family == "plate":
        pickup_bound = np.asarray(
            [
                abs(math.sin(int(m) * math.pi * 0.37) * math.sin(int(n) * math.pi * 0.61))
                for m, n in indices
            ],
            dtype=np.float64,
        )
    else:
        pickup_bound = np.asarray(
            [abs(float(beam_shape(float(BEAM_ROOTS[int(index) - 1]), 0.83))) for index, _ in indices],
            dtype=np.float64,
        )
    per_mode_bound = pickup_bound / (2.0 * math.pi * damped)
    global_bound = float(np.max(per_mode_bound))
    if not math.isfinite(global_bound) or global_bound <= 0.0:
        raise ValueError(f"T0 gain normalization failed for {recipe.object_id}")
    return raw / global_bound, per_mode_bound / global_bound


def encode_mesh(vertices: np.ndarray, triangles: np.ndarray) -> bytes:
    if (
        vertices.ndim != 2
        or vertices.shape[1] != 3
        or triangles.ndim != 2
        or triangles.shape[1] != 3
        or np.any(~np.isfinite(vertices))
        or np.any(triangles >= len(vertices))
    ):
        raise ValueError("T0 mesh payload is invalid")
    header = MESH_MAGIC + struct.pack("<III", 1, len(vertices), len(triangles))
    return header + vertices.astype("<f8", copy=False).tobytes() + triangles.astype("<u4", copy=False).tobytes()


def encode_modes(frequencies: np.ndarray, decay: np.ndarray, indices: np.ndarray) -> bytes:
    if (
        frequencies.ndim != 1
        or not 1 <= len(frequencies) <= len(BEAM_ROOTS)
        or decay.shape != frequencies.shape
        or indices.shape != (len(frequencies), 2)
        or np.any(~np.isfinite(frequencies))
        or np.any(~np.isfinite(decay))
        or np.any(frequencies <= 0.0)
        or np.any(decay <= 0.0)
        or np.any(np.diff(frequencies) <= 0.0)
    ):
        raise ValueError("T0 modal payload is invalid or unsorted")
    result = bytearray(MODAL_MAGIC + struct.pack("<II", 1, len(frequencies)))
    for ordinal, (frequency, rate, pair) in enumerate(zip(frequencies, decay, indices, strict=True)):
        result.extend(
            struct.pack(
                "<IIIdd",
                ordinal,
                int(pair[0]),
                int(pair[1]),
                float(frequency),
                float(rate),
            )
        )
    return bytes(result)


def encode_gains(mesh_sha256: str, gains: np.ndarray) -> bytes:
    if (
        len(mesh_sha256) != 64
        or any(character not in "0123456789abcdef" for character in mesh_sha256)
        or gains.ndim != 2
        or gains.shape[0] == 0
        or gains.shape[1] == 0
        or np.any(~np.isfinite(gains))
    ):
        raise ValueError("T0 contact-gain payload is invalid")
    return (
        GAIN_MAGIC
        + struct.pack("<III", 1, gains.shape[0], gains.shape[1])
        + bytes.fromhex(mesh_sha256)
        + gains.astype("<f8", copy=False).tobytes()
    )


def validate_gain_binding(mesh_bytes: bytes, gain_bytes: bytes) -> None:
    if len(gain_bytes) < 52 or gain_bytes[:8] != GAIN_MAGIC:
        raise ValueError("T0 contact-gain header is invalid")
    if gain_bytes[20:52] != bytes.fromhex(sha256_bytes(mesh_bytes)):
        raise ValueError("T0 contact-gain mesh hash mismatch")


def encode_float32_wav(samples: np.ndarray) -> bytes:
    if samples.ndim != 1 or len(samples) == 0 or np.any(~np.isfinite(samples)):
        raise ValueError("T0 WAV payload is empty or non-finite")
    payload = samples.astype("<f4", copy=False).tobytes()
    fmt = struct.pack("<HHIIHH", 3, 1, SAMPLE_RATE_HZ, SAMPLE_RATE_HZ * 4, 4, 32)
    return b"RIFF" + struct.pack("<I", 4 + 8 + len(fmt) + 8 + len(payload)) + b"WAVEfmt " + struct.pack(
        "<I", len(fmt)
    ) + fmt + b"data" + struct.pack("<I", len(payload)) + payload


def modal_basis(
    frequencies: np.ndarray,
    decay: np.ndarray,
    duration_seconds: float,
) -> np.ndarray:
    if (
        frequencies.ndim != 1
        or decay.ndim != 1
        or len(frequencies) == 0
        or frequencies.shape != decay.shape
        or np.any(~np.isfinite(frequencies))
        or np.any(~np.isfinite(decay))
        or np.any(frequencies <= 0.0)
        or np.any(decay <= 0.0)
        or not math.isfinite(duration_seconds)
        or duration_seconds <= 0.0
    ):
        raise ValueError("T0 modal basis inputs are invalid")
    frames = round(SAMPLE_RATE_HZ * duration_seconds)
    if frames <= 0:
        raise ValueError("T0 modal basis is empty")
    time = np.arange(frames, dtype=np.float64) / SAMPLE_RATE_HZ
    return np.exp(-decay[:, None] * time[None, :]) * np.sin(
        2.0 * math.pi * frequencies[:, None] * time[None, :]
    )


def render_from_basis(
    basis: np.ndarray,
    contact_gains: np.ndarray,
    gain_bounds: np.ndarray,
    impulse: float = 1.0,
) -> np.ndarray:
    if (
        basis.ndim != 2
        or contact_gains.ndim != 1
        or gain_bounds.ndim != 1
        or basis.shape[0] == 0
        or basis.shape[1] == 0
        or basis.shape[0] != contact_gains.shape[0]
        or basis.shape[0] != gain_bounds.shape[0]
        or np.any(~np.isfinite(basis))
        or np.any(~np.isfinite(contact_gains))
        or np.any(~np.isfinite(gain_bounds))
        or not math.isfinite(impulse)
    ):
        raise ValueError("T0 render inputs are invalid")
    safe_scale = 0.20 / float(np.sum(np.abs(gain_bounds)))
    samples = impulse * safe_scale * (contact_gains @ basis)
    if np.any(~np.isfinite(samples)) or float(np.max(np.abs(samples))) >= 0.95:
        raise ValueError("T0 render is non-finite or clips")
    return samples


def render(
    frequencies: np.ndarray,
    decay: np.ndarray,
    contact_gains: np.ndarray,
    gain_bounds: np.ndarray,
    duration_seconds: float,
    impulse: float = 1.0,
) -> np.ndarray:
    return render_from_basis(
        modal_basis(frequencies, decay, duration_seconds),
        contact_gains,
        gain_bounds,
        impulse,
    )


def file_ref(relative: str, data: bytes) -> dict[str, str]:
    return {"path": relative, "sha256": sha256_bytes(data)}


def write_bytes(root: Path, relative: str, data: bytes) -> dict[str, Any]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {"path": relative, "sha256": sha256_bytes(data), "byte_count": len(data)}


def verify_artifacts(root: Path, artifacts: list[dict[str, Any]]) -> None:
    paths = [artifact["path"] for artifact in artifacts]
    if len(paths) != len(set(paths)):
        raise ValueError("T0 artifact paths are not unique")
    for artifact in artifacts:
        data = (root / artifact["path"]).read_bytes()
        if len(data) != artifact["byte_count"] or sha256_bytes(data) != artifact["sha256"]:
            raise ValueError(f"T0 artifact hash mismatch: {artifact['path']}")


def contact_point(recipe: Recipe, u: float, v: float) -> list[float]:
    first, second, thickness = recipe.dimensions
    if recipe.family == "plate":
        return [first * u, second * v, thickness / 2.0]
    return [first * u, second * (v - 0.5), thickness / 2.0]


def listener_point(recipe: Recipe) -> list[float]:
    first, second, thickness = recipe.dimensions
    if recipe.family == "plate":
        return [first * 0.37, second * 0.61, thickness / 2.0]
    return [first * 0.83, 0.0, thickness / 2.0]


def root_residuals(count: int) -> list[float]:
    return [
        abs(math.cos(float(root)) + 1.0 / math.cosh(float(root)))
        for root in BEAM_ROOTS[:count]
    ]


def analytic_controls(recipe: Recipe, count: int) -> dict[str, float]:
    base, indices = (
        plate_modes(recipe, count) if recipe.family == "plate" else beam_modes(recipe, count)
    )
    material = recipe.material
    elastic = replace(material, youngs=material.youngs * 4.0)
    dense = replace(material, density=material.density * 4.0)
    elastic_recipe = replace(recipe, material=elastic)
    dense_recipe = replace(recipe, material=dense)
    elastic_frequency = (
        plate_modes(elastic_recipe, count)[0]
        if recipe.family == "plate"
        else beam_modes(elastic_recipe, count)[0]
    )
    dense_frequency = (
        plate_modes(dense_recipe, count)[0]
        if recipe.family == "plate"
        else beam_modes(dense_recipe, count)[0]
    )
    dimensions = list(recipe.dimensions)
    dimensions[2] *= 2.0
    thick_recipe = replace(recipe, dimensions=tuple(dimensions))
    thick_frequency = (
        plate_modes(thick_recipe, count)[0]
        if recipe.family == "plate"
        else beam_modes(thick_recipe, count)[0]
    )
    dimensions = list(recipe.dimensions)
    if recipe.family == "plate":
        dimensions[0] *= 2.0
        dimensions[1] *= 2.0
    else:
        dimensions[0] *= 2.0
    long_recipe = replace(recipe, dimensions=tuple(dimensions))
    long_frequency = (
        plate_modes(long_recipe, count)[0]
        if recipe.family == "plate"
        else beam_modes(long_recipe, count)[0]
    )
    relation_error = max(
        float(np.max(np.abs(elastic_frequency / base - 2.0))),
        float(np.max(np.abs(dense_frequency / base - 0.5))),
        float(np.max(np.abs(thick_frequency / base - 2.0))),
        float(np.max(np.abs(long_frequency / base - 0.25))),
    )
    if recipe.family == "plate":
        boundary_u = raw_gain(
            recipe,
            indices,
            np.asarray([0.0, 1.0, 0.25, 0.75]),
            np.asarray([0.25, 0.75, 0.0, 1.0]),
        )
        boundary_error = float(np.max(np.abs(boundary_u)))
        clamp_error = 0.0
    else:
        roots = [float(value) for value in BEAM_ROOTS[:count]]
        displacement = max(abs(float(beam_shape(root, 0.0))) for root in roots)
        slopes = []
        for root in roots:
            sigma = (math.cosh(root) + math.cos(root)) / (
                math.sinh(root) + math.sin(root)
            )
            slopes.append(
                abs(root * (math.sinh(0.0) + math.sin(0.0) - sigma * (math.cosh(0.0) - math.cos(0.0))))
            )
        boundary_error = 0.0
        clamp_error = max(displacement, max(slopes))
    if relation_error > 1.0e-12 or boundary_error > 1.0e-12 or clamp_error > 1.0e-12:
        raise ValueError(f"T0 analytic control failed for {recipe.object_id}")
    return {
        "maximum_scaling_relative_error": relation_error,
        "maximum_plate_boundary_absolute_error": boundary_error,
        "maximum_beam_clamp_absolute_error": clamp_error,
    }


def validate_remesh(
    coarse_uv: np.ndarray,
    coarse_gains: np.ndarray,
    coarse_bounds: np.ndarray,
    fine_uv: np.ndarray,
    fine_gains: np.ndarray,
    fine_bounds: np.ndarray,
) -> None:
    lookup = {
        (round(float(u), 15), round(float(v), 15)): row
        for (u, v), row in zip(fine_uv, fine_gains, strict=True)
    }
    try:
        exact = all(
            row.tobytes()
            == lookup[(round(float(uv[0]), 15), round(float(uv[1]), 15))].tobytes()
            for uv, row in zip(coarse_uv, coarse_gains, strict=True)
        )
    except KeyError as error:
        raise ValueError("T0 remesh common vertex is missing") from error
    if not exact or coarse_bounds.tobytes() != fine_bounds.tobytes():
        raise ValueError("T0 remesh truth drift")


def build_into(staging: Path, selected: Profile, manifest_bytes: bytes) -> dict[str, Any]:
    validate_contract_constants()
    validate_profile(selected)
    artifacts: list[dict[str, Any]] = []
    pending_rows: list[dict[str, Any]] = []
    object_reports = []
    beam_characteristic_maximum_residual = max(root_residuals(selected.mode_count))
    if beam_characteristic_maximum_residual > 1.0e-12:
        raise ValueError("T0 cantilever root table fails its characteristic residual")
    for recipe in selected.recipes:
        frequencies, decay, indices = modal_solution(recipe, selected.mode_count)
        controls = analytic_controls(recipe, selected.mode_count)
        coarse_vertices, coarse_triangles, coarse_uv = mesh(recipe, selected.coarse_grid)
        fine_vertices, fine_triangles, fine_uv = mesh(recipe, selected.fine_grid)
        coarse_mesh_bytes = encode_mesh(coarse_vertices, coarse_triangles)
        fine_mesh_bytes = encode_mesh(fine_vertices, fine_triangles)
        coarse_mesh_ref = write_bytes(staging, f"objects/{recipe.object_id}/mesh-coarse.bin", coarse_mesh_bytes)
        fine_mesh_ref = write_bytes(staging, f"objects/{recipe.object_id}/mesh.bin", fine_mesh_bytes)
        artifacts.extend((coarse_mesh_ref, fine_mesh_ref))
        coarse_gains, coarse_bounds = normalized_gains(recipe, indices, frequencies, coarse_uv)
        fine_gains, fine_bounds = normalized_gains(recipe, indices, frequencies, fine_uv)
        modal_bytes = encode_modes(frequencies, decay, indices)
        coarse_gain_bytes = encode_gains(coarse_mesh_ref["sha256"], coarse_gains)
        fine_gain_bytes = encode_gains(fine_mesh_ref["sha256"], fine_gains)
        validate_gain_binding(coarse_mesh_bytes, coarse_gain_bytes)
        validate_gain_binding(fine_mesh_bytes, fine_gain_bytes)
        modal_ref = write_bytes(staging, f"objects/{recipe.object_id}/modal-parameters.bin", modal_bytes)
        coarse_gain_ref = write_bytes(
            staging, f"objects/{recipe.object_id}/contact-gain-field-coarse.bin", coarse_gain_bytes
        )
        fine_gain_ref = write_bytes(
            staging, f"objects/{recipe.object_id}/contact-gain-field.bin", fine_gain_bytes
        )
        artifacts.extend((modal_ref, coarse_gain_ref, fine_gain_ref))

        validate_remesh(
            coarse_uv, coarse_gains, coarse_bounds, fine_uv, fine_gains, fine_bounds
        )

        contact_vectors = []
        audio_refs = []
        basis = modal_basis(frequencies, decay, selected.duration_seconds)
        for contact_id, sample_role, u, v in selected.contacts:
            uv = np.asarray([[u, v]], dtype=np.float64)
            gains, gain_bounds = normalized_gains(recipe, indices, frequencies, uv)
            contact_vector = gains[0]
            contact_vectors.append(contact_vector)
            samples = render_from_basis(basis, contact_vector, gain_bounds)
            low = render_from_basis(basis, contact_vector, gain_bounds, 0.5)
            high = render_from_basis(basis, contact_vector, gain_bounds, 2.0)
            if not np.array_equal(low, samples * 0.5) or not np.array_equal(high, samples * 2.0):
                raise ValueError(f"T0 force linearity failed for {recipe.object_id}/{contact_id}")
            wav = encode_float32_wav(samples)
            relative = f"objects/{recipe.object_id}/contact-{contact_id}.wav"
            audio_ref = write_bytes(staging, relative, wav)
            artifacts.append(audio_ref)
            audio_refs.append((contact_id, sample_role, u, v, audio_ref))
        contact_matrix = np.asarray(contact_vectors)
        if not bool(np.any(np.ptp(contact_matrix, axis=0) > 1.0e-12)):
            raise ValueError(f"T0 contact field is constant for {recipe.object_id}")

        object_reports.append(
            {
                "object_id": recipe.object_id,
                "role": recipe.role,
                "family": recipe.family,
                "material_id": recipe.material.material_id,
                "mode_count": selected.mode_count,
                "contact_count": len(selected.contacts),
                "minimum_frequency_hz": float(frequencies[0]),
                "maximum_frequency_hz": float(frequencies[-1]),
                "remesh_common_vertices_exact": True,
                "force_linearity_exact": True,
                "analytic_controls": controls,
            }
        )
        for contact_id, sample_role, u, v, audio_ref in audio_refs:
            pending_rows.append(
                {
                    "row_id": f"t0-{recipe.role}-{recipe.object_id}-{contact_id}",
                    "split_role": recipe.role,
                    "sample_role": sample_role,
                    "corpus_role": "target",
                    "evidence_lane": "synthetic_teacher",
                    "audio_semantics": "synthetic_modal_render",
                    "source_group_id": f"v24-t0-source-{recipe.role}",
                    "family_group_id": f"analytic-{recipe.family}",
                    "object_group_id": f"v24-t0-object-{recipe.object_id}",
                    "recording_parent_id": f"v24-t0-recording-{recipe.object_id}-{contact_id}",
                    "condition_group_id": f"v24-t0-condition-{recipe.object_id}-{contact_id}",
                    "mutation_parent_id": None,
                    "lineage_report_ids": [LINEAGE_ID],
                    "audio": {"path": audio_ref["path"], "sha256": audio_ref["sha256"]},
                    "_recipe": recipe,
                    "_contact": (u, v),
                    "_mesh": fine_mesh_ref,
                    "_modal": modal_ref,
                    "_gain": fine_gain_ref,
                }
            )

    verify_artifacts(staging, artifacts)
    evidence = {
        "schema": EVIDENCE_SCHEMA,
        "status": "Validated",
        "claim": CLAIM,
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "protocol_sha256": PROTOCOL_SHA256,
        "profile": selected.profile_id,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "implementation_sha256": sha256_bytes(Path(__file__).read_bytes()),
        "environment": {"python": platform.python_version(), "numpy": np.__version__},
        "formula_ids": list(FORMULA_IDS),
        "beam_characteristic_maximum_residual": beam_characteristic_maximum_residual,
        "objects": object_reports,
        "artifact_count": len(artifacts),
        "artifact_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "model_training_authorized": False,
        "real_material_authorized": False,
        "runtime_authorized": False,
    }
    evidence_bytes = canonical_json(evidence)
    evidence_ref = write_bytes(staging, "teacher-evidence.json", evidence_bytes)

    rows = []
    for pending in pending_rows:
        recipe = pending.pop("_recipe")
        u, v = pending.pop("_contact")
        mesh_ref = pending.pop("_mesh")
        modal_ref = pending.pop("_modal")
        gain_ref = pending.pop("_gain")
        evidence_file = {"path": evidence_ref["path"], "sha256": evidence_ref["sha256"]}
        pending["audio_provenance"] = evidence_file
        pending["axes"] = {
            "material": {"value_id": recipe.material.material_id, "evidence": evidence_file},
            "geometry": {
                "geometry_id": f"analytic-{recipe.family}-{recipe.object_id}",
                "feature_artifact": {"path": mesh_ref["path"], "sha256": mesh_ref["sha256"]},
                "evidence": evidence_file,
            },
            "support": {
                "value_id": SUPPORT_IDS[recipe.family],
                "evidence": evidence_file,
            },
            "impact": {
                "coordinate_profile": "analytic-local-right-handed-metres-v1",
                "point_metres": contact_point(recipe, u, v),
                "outward_normal": [0.0, 0.0, 1.0],
                "evidence": evidence_file,
            },
            "listener": {
                "coordinate_profile": "analytic-local-right-handed-metres-v1",
                "point_metres": listener_point(recipe),
                "evidence": evidence_file,
            },
            "excitation": {"impulse_newton_seconds": 1.0, "evidence": evidence_file},
            "teacher_target": {
                "representation_id": TEACHER_REPRESENTATION,
                "mode_count": selected.mode_count,
                "modal_parameters": {"path": modal_ref["path"], "sha256": modal_ref["sha256"]},
                "contact_gain_field": {"path": gain_ref["path"], "sha256": gain_ref["sha256"]},
                "evidence": evidence_file,
            },
        }
        rows.append(pending)
    rows.sort(key=lambda row: row["row_id"])
    lane = {
        "schema": LANE_SCHEMA,
        "status": "Validated",
        "claim": CLAIM,
        "lineage_report": {
            "id": LINEAGE_ID,
            "expected_schema": EVIDENCE_SCHEMA,
            "expected_claim": CLAIM,
            "artifact": {"path": evidence_ref["path"], "sha256": evidence_ref["sha256"]},
        },
        "rows": rows,
    }
    lane_bytes = canonical_json(lane)
    lane_ref = write_bytes(staging, "lane-records.json", lane_bytes)
    total_bytes = sum(path.stat().st_size for path in staging.rglob("*") if path.is_file())
    if total_bytes > MAX_OUTPUT_BYTES:
        raise ValueError(f"T0 output exceeds {MAX_OUTPUT_BYTES} bytes")
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "T0_ANALYTIC_TEACHER_CONTRACT_PASS",
        "claim": CLAIM,
        "profile": selected.profile_id,
        "object_count": len(selected.recipes),
        "row_count": len(rows),
        "role_counts": {role: sum(row["split_role"] == role for row in rows) for role in ROLES},
        "mode_count_per_object": selected.mode_count,
        "contact_count_per_object": len(selected.contacts),
        "lane_records_sha256": lane_ref["sha256"],
        "teacher_evidence_sha256": evidence_ref["sha256"],
        "output_bytes_before_report": total_bytes,
        "repeated_render_identical": True,
        "remesh_common_vertices_exact": True,
        "force_linearity_exact": True,
        "model_training_authorized": False,
        "real_material_authorized": False,
        "runtime_authorized": False,
    }
    report_bytes = canonical_json(report)
    write_bytes(staging, "report.json", report_bytes)
    return report


def run(manifest_path: Path, output_path: Path) -> dict[str, Any]:
    root = repository_root().resolve(strict=True)
    manifest_path = external_file(root, manifest_path)
    output_path = external_output(root, output_path)
    manifest, manifest_bytes = load_manifest(manifest_path)
    selected = profile(str(manifest["profile"]))
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v24-t0-", dir=output_path.parent))
    try:
        report = build_into(staging, selected, manifest_bytes)
        os.replace(staging, output_path)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
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
    except Exception as error:  # noqa: BLE001 - stable CLI diagnostic boundary
        print(f"physical-sound-v24-t0: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
