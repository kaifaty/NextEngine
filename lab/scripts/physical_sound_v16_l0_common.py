#!/usr/bin/env python3
"""Frozen synthetic corpus and metrics for Physical Sound V16 L0."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import shutil
import struct
import sys
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
from typing import Any, Iterable

import numpy as np
import scipy
from scipy.signal import hilbert
from scipy.sparse import csr_matrix
from scipy.sparse.csgraph import shortest_path
import torch


STUDY_ID = "physical-sound-v16-l0-known-truth-neural-oracle"
REVISION = "structured-modal-field-known-truth-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v16-l0.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v16-l0.report.v1"
PREDICTION_SCHEMA = "nextengine.experimental-physical-sound-v16-l0.prediction.v1"

PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v16-l0a-known-truth-neural-oracle-protocol-2026-09-01.md"
)
IMPLEMENTATION_FILES = (
    "physical_sound_v16_l0_common.py",
    "physical_sound_v16_l0_model.py",
    "physical_sound_v16_l0_oracle.py",
)
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
    "torch": "2.13.0+cu130",
}

MODE_COUNT = 8
SAMPLE_RATE_HZ = 16_000
SAMPLE_COUNT = 8_192
SEEDS = (160_001, 160_002, 160_003)
UPDATES = 2_500
LEARNING_RATE_START = 2.0e-3
LEARNING_RATE_END = 1.0e-5
WEIGHT_DECAY = 1.0e-6

Q = np.asarray([1.40, 2.30, 3.40, 4.80, 6.50, 8.50, 10.80, 13.40])
K = np.asarray([1, 1, 2, 2, 3, 3, 4, 5], dtype=np.int64)
ELL = np.asarray([1, 2, 1, 3, 2, 4, 3, 2], dtype=np.int64)
IMPULSE_DIRECTION = np.asarray([0.31, -0.89, 0.33], dtype=np.float64)
IMPULSE_DIRECTION /= np.linalg.norm(IMPULSE_DIRECTION)

MATERIALS = {
    "Steel": {"density": 7_850.0, "wave_speed": 5_000.0, "decay": 6.0},
    "Wood": {"density": 650.0, "wave_speed": 3_300.0, "decay": 16.0},
    "Glass": {"density": 2_500.0, "wave_speed": 5_200.0, "decay": 4.0},
}
MATERIAL_ORDER = ("Steel", "Wood", "Glass")
SURFACE_ORDER = ("Plate", "Cylinder", "Bowl")
SUPPORT_ORDER = ("Free", "BaseClamped")
TOPOLOGY_FACTORS = {"Plate": 1.0, "Cylinder": 1.18, "Bowl": 1.36}
SUPPORT_FREQUENCY_FACTORS = {"Free": 1.0, "BaseClamped": 1.22}
SUPPORT_DAMPING_FACTORS = {"Free": 1.0, "BaseClamped": 1.12}

ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "protected_signal_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
    "validator_calibration_values_decoded": 0,
}


class L0Error(RuntimeError):
    """The frozen V16 L0 boundary or numerical contract was violated."""


@dataclass(frozen=True)
class ObjectSpec:
    object_id: str
    role: str
    material: str
    surface: str
    support: str
    length_m: float
    aspect: float
    wall_m: float
    grid_u: int
    grid_v: int

    def record(self) -> dict[str, Any]:
        return {
            "aspect": self.aspect,
            "grid": [self.grid_u, self.grid_v],
            "length_m": self.length_m,
            "material": self.material,
            "object_id": self.object_id,
            "role": self.role,
            "support": self.support,
            "surface": self.surface,
            "wall_m": self.wall_m,
        }


OBJECT_SPECS = (
    ObjectSpec("tr-steel-plate", "train", "Steel", "Plate", "Free", 0.320, 1.30, 0.0040, 9, 10),
    ObjectSpec("tr-steel-cylinder", "train", "Steel", "Cylinder", "BaseClamped", 0.280, 1.10, 0.0030, 10, 9),
    ObjectSpec("tr-steel-bowl", "train", "Steel", "Bowl", "Free", 0.240, 0.85, 0.0035, 11, 9),
    ObjectSpec("tr-wood-plate", "train", "Wood", "Plate", "BaseClamped", 0.380, 1.50, 0.0120, 9, 11),
    ObjectSpec("tr-wood-cylinder", "train", "Wood", "Cylinder", "Free", 0.340, 1.00, 0.0090, 10, 10),
    ObjectSpec("tr-wood-bowl", "train", "Wood", "Bowl", "BaseClamped", 0.300, 0.90, 0.0100, 11, 10),
    ObjectSpec("tr-glass-plate", "train", "Glass", "Plate", "Free", 0.220, 1.20, 0.0040, 9, 12),
    ObjectSpec("tr-glass-cylinder", "train", "Glass", "Cylinder", "BaseClamped", 0.260, 0.85, 0.0030, 10, 11),
    ObjectSpec("tr-glass-bowl", "train", "Glass", "Bowl", "Free", 0.200, 1.05, 0.0025, 11, 11),
    ObjectSpec("dv-steel-cylinder", "development", "Steel", "Cylinder", "Free", 0.310, 0.92, 0.0032, 12, 10),
    ObjectSpec("dv-wood-bowl", "development", "Wood", "Bowl", "Free", 0.330, 1.10, 0.0110, 12, 11),
    ObjectSpec("dv-glass-plate", "development", "Glass", "Plate", "BaseClamped", 0.240, 1.40, 0.0032, 12, 12),
    ObjectSpec("te-steel-plate", "test", "Steel", "Plate", "BaseClamped", 0.290, 0.80, 0.0045, 13, 11),
    ObjectSpec("te-steel-bowl", "test", "Steel", "Bowl", "BaseClamped", 0.270, 1.20, 0.0038, 13, 12),
    ObjectSpec("te-wood-plate", "test", "Wood", "Plate", "Free", 0.350, 1.00, 0.0105, 13, 13),
    ObjectSpec("te-wood-cylinder", "test", "Wood", "Cylinder", "BaseClamped", 0.310, 1.25, 0.0085, 14, 11),
    ObjectSpec("te-glass-cylinder", "test", "Glass", "Cylinder", "Free", 0.230, 1.15, 0.0028, 14, 12),
    ObjectSpec("te-glass-bowl", "test", "Glass", "Bowl", "BaseClamped", 0.210, 0.95, 0.0030, 14, 13),
)


@dataclass
class ObjectData:
    spec: ObjectSpec
    uv: np.ndarray
    vertices: np.ndarray
    normals: np.ndarray
    curvatures: np.ndarray
    faces: np.ndarray
    edges: np.ndarray
    edge_lengths: np.ndarray
    all_geodesic: np.ndarray
    geodesic_diameter: float
    context_indices: np.ndarray
    query_indices: np.ndarray
    frequencies: np.ndarray
    damping: np.ndarray
    gains: np.ndarray
    mesh_sha256: str


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, dict):
        return {str(key): json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False)
            + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise L0Error(f"cannot encode canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def environment_identity(require_exact: bool = True) -> dict[str, Any]:
    observed = {
        "byteorder": sys.byteorder,
        "compute_device": "cpu",
        "machine": platform.machine(),
        "numpy": np.__version__,
        "platform": platform.platform(),
        "python": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
        "torch_deterministic_algorithms": True,
        "torch_interop_threads": 1,
        "torch_threads": 1,
    }
    if require_exact:
        for key, expected in REQUIRED_ENVIRONMENT.items():
            if observed[key] != expected:
                raise L0Error(
                    f"L0 environment changed for {key}: {observed[key]} != {expected}"
                )
    return observed


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    result: dict[str, str] = {}
    for filename in IMPLEMENTATION_FILES:
        path = directory / filename
        if not path.is_file():
            raise L0Error(f"missing L0 implementation file: {filename}")
        result[filename] = sha256_file(path)
    return result


def prepare_output(output: Path) -> tuple[Path, Path]:
    root = repository_root().resolve()
    resolved = output.resolve()
    allowed = Path(
        "/home/kaifaty/.codex/experiments/nextengine/physical-sound"
    ).resolve()
    if resolved.is_relative_to(root):
        raise L0Error("L0 output must stay outside the repository")
    if not resolved.is_relative_to(allowed):
        raise L0Error("L0 output must stay under the physical-sound experiment root")
    if resolved.exists():
        raise L0Error("L0 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def publish_output(output: Path, staging: Path) -> None:
    staging.rename(output)


def _faces(grid_u: int, grid_v: int, periodic_u: bool) -> np.ndarray:
    faces: list[tuple[int, int, int]] = []
    u_limit = grid_u if periodic_u else grid_u - 1
    for i in range(u_limit):
        ni = (i + 1) % grid_u
        for j in range(grid_v - 1):
            a = i * grid_v + j
            b = ni * grid_v + j
            c = ni * grid_v + j + 1
            d = i * grid_v + j + 1
            faces.append((a, b, c))
            faces.append((a, c, d))
    return np.asarray(faces, dtype=np.int64)


def _surface_arrays(spec: ObjectSpec) -> tuple[np.ndarray, ...]:
    if spec.surface == "Plate":
        u_values = np.linspace(-1.0, 1.0, spec.grid_u, dtype=np.float64)
        v_values = np.linspace(-1.0, 1.0, spec.grid_v, dtype=np.float64)
    else:
        u_values = -1.0 + 2.0 * np.arange(spec.grid_u) / spec.grid_u
        if spec.surface == "Cylinder":
            v_values = np.linspace(-1.0, 1.0, spec.grid_v, dtype=np.float64)
        else:
            v_values = -1.0 + 2.0 * (
                np.arange(spec.grid_v, dtype=np.float64) + 0.5
            ) / spec.grid_v
    u_grid, v_grid = np.meshgrid(u_values, v_values, indexing="ij")
    u = u_grid.reshape(-1)
    v = v_grid.reshape(-1)
    uv = np.column_stack((u, v))

    if spec.surface == "Plate":
        vertices = np.column_stack(
            (
                0.5 * spec.length_m * u,
                0.5 * spec.length_m / spec.aspect * v,
                np.zeros_like(u),
            )
        )
        normals = np.tile(np.asarray([0.0, 0.0, 1.0]), (u.size, 1))
        curvatures = np.zeros((u.size, 2), dtype=np.float64)
    elif spec.surface == "Cylinder":
        theta = np.pi * (u + 1.0)
        radius = spec.length_m / (2.0 * np.pi)
        vertices = np.column_stack(
            (
                radius * np.cos(theta),
                radius * np.sin(theta),
                0.5 * spec.length_m * spec.aspect * v,
            )
        )
        normals = np.column_stack(
            (np.cos(theta), np.sin(theta), np.zeros_like(theta))
        )
        curvatures = np.column_stack(
            (np.full_like(theta, 1.0 / radius), np.zeros_like(theta))
        )
    elif spec.surface == "Bowl":
        theta = np.pi * (u + 1.0)
        alpha = np.pi * (v + 1.0) / 4.0
        radial = 0.5 * spec.length_m
        axial = 0.5 * spec.length_m * spec.aspect
        vertices = np.column_stack(
            (
                radial * np.sin(alpha) * np.cos(theta),
                radial * np.sin(alpha) * np.sin(theta),
                -axial * np.cos(alpha),
            )
        )
        normals = np.column_stack(
            (
                vertices[:, 0] / (radial * radial),
                vertices[:, 1] / (radial * radial),
                vertices[:, 2] / (axial * axial),
            )
        )
        normals /= np.linalg.norm(normals, axis=1, keepdims=True)
        metric = (
            radial * radial * np.cos(alpha) ** 2
            + axial * axial * np.sin(alpha) ** 2
        )
        meridional = radial * axial / np.power(metric, 1.5)
        azimuthal = axial / (radial * np.sqrt(metric))
        curvatures = np.column_stack((meridional, azimuthal))
    else:
        raise L0Error(f"unknown surface: {spec.surface}")

    faces = _faces(spec.grid_u, spec.grid_v, spec.surface != "Plate")
    return uv, vertices, normals, curvatures, faces


def _edge_graph(
    vertices: np.ndarray, faces: np.ndarray
) -> tuple[np.ndarray, np.ndarray, np.ndarray, float]:
    edge_set: set[tuple[int, int]] = set()
    for face in faces:
        for left, right in ((face[0], face[1]), (face[1], face[2]), (face[2], face[0])):
            a, b = sorted((int(left), int(right)))
            edge_set.add((a, b))
    edges = np.asarray(sorted(edge_set), dtype=np.int64)
    lengths = np.linalg.norm(vertices[edges[:, 0]] - vertices[edges[:, 1]], axis=1)
    rows = np.concatenate((edges[:, 0], edges[:, 1]))
    cols = np.concatenate((edges[:, 1], edges[:, 0]))
    values = np.concatenate((lengths, lengths))
    graph = csr_matrix((values, (rows, cols)), shape=(vertices.shape[0],) * 2)
    distances = np.asarray(shortest_path(graph, directed=False), dtype=np.float64)
    if not np.isfinite(distances).all():
        raise L0Error("generated mesh graph is disconnected")
    diameter = float(np.max(distances))
    if diameter <= 0.0:
        raise L0Error("generated mesh has zero geodesic diameter")
    return edges, lengths, distances, diameter


def farthest_point_indices(vertices: np.ndarray, count: int) -> np.ndarray:
    if count <= 0 or count > vertices.shape[0]:
        raise L0Error("invalid farthest-point count")
    lexicographic = np.lexsort((vertices[:, 2], vertices[:, 1], vertices[:, 0]))
    selected = [int(lexicographic[0])]
    minimum = np.sum((vertices - vertices[selected[0]]) ** 2, axis=1)
    minimum[selected[0]] = -1.0
    while len(selected) < count:
        index = int(np.argmax(minimum))
        selected.append(index)
        distance = np.sum((vertices - vertices[index]) ** 2, axis=1)
        minimum = np.minimum(minimum, distance)
        minimum[np.asarray(selected, dtype=np.int64)] = -1.0
    return np.asarray(selected, dtype=np.int64)


def _mesh_identity(
    spec: ObjectSpec, vertices: np.ndarray, faces: np.ndarray
) -> str:
    payload = bytearray(canonical_json(spec.record()))
    payload.extend(np.asarray(vertices, dtype="<f8").tobytes(order="C"))
    payload.extend(np.asarray(faces, dtype="<i8").tobytes(order="C"))
    return sha256_bytes(bytes(payload))


def _modal_truth(
    spec: ObjectSpec,
    uv: np.ndarray,
    normals: np.ndarray,
    curvatures: np.ndarray,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    material = MATERIALS[spec.material]
    base = (
        0.5
        * material["wave_speed"]
        * spec.wall_m
        / (spec.length_m * spec.length_m)
    )
    frequencies = (
        base
        * Q
        * TOPOLOGY_FACTORS[spec.surface]
        * (1.0 + 0.12 * abs(math.log(spec.aspect)))
        * SUPPORT_FREQUENCY_FACTORS[spec.support]
    )
    mean_absolute_curvature = float(np.mean(np.abs(curvatures)))
    damping = (
        material["decay"]
        * (1.0 + 0.10 * np.arange(MODE_COUNT))
        * (1.0 + 0.25 * mean_absolute_curvature * spec.length_m)
        * SUPPORT_DAMPING_FACTORS[spec.support]
    )
    if (
        np.any(np.diff(frequencies) <= 0.0)
        or frequencies[0] < 40.0
        or frequencies[-1] > 12_000.0
    ):
        raise L0Error(f"{spec.object_id} truth frequencies violate bounds")
    if np.any(damping <= 0.0) or np.any(damping > 128.0):
        raise L0Error(f"{spec.object_id} truth damping violates bounds")

    u = uv[:, 0:1]
    v = uv[:, 1:2]
    mode = np.arange(MODE_COUNT, dtype=np.float64)[None, :]
    if spec.surface == "Plate":
        phi = np.sin(K[None, :] * np.pi * (u + 1.0) / 2.0) * np.sin(
            ELL[None, :] * np.pi * (v + 1.0) / 2.0
        )
    elif spec.surface == "Cylinder":
        phi = np.cos(K[None, :] * np.pi * u + mode * np.pi / 11.0) * np.sin(
            ELL[None, :] * np.pi * (v + 1.0) / 2.0
        )
    else:
        phi = (
            np.cos(K[None, :] * np.pi * u + mode * np.pi / 13.0)
            * np.cos(ELL[None, :] * np.pi * v / 2.0)
            * (1.0 - 0.15 * (u * u + v * v))
        )
    amplitude = (
        1.0 + 0.08 * math.log(material["density"] / 650.0)
    ) / np.sqrt(np.arange(1, MODE_COUNT + 1, dtype=np.float64))
    directional = 0.75 + 0.25 * np.abs(normals @ IMPULSE_DIRECTION)
    gains = amplitude[None, :] * phi * directional[:, None]
    if not all(np.isfinite(value).all() for value in (frequencies, damping, gains)):
        raise L0Error(f"{spec.object_id} truth is non-finite")
    return frequencies, damping, gains


def generate_object(spec: ObjectSpec) -> ObjectData:
    uv, vertices, normals, curvatures, faces = _surface_arrays(spec)
    edges, edge_lengths, geodesic, diameter = _edge_graph(vertices, faces)
    context_count = max(20, int(math.ceil(0.25 * vertices.shape[0])))
    context = farthest_point_indices(vertices, context_count)
    mask = np.ones(vertices.shape[0], dtype=bool)
    mask[context] = False
    query = np.flatnonzero(mask)
    if context.size < 20 or query.size < 40:
        raise L0Error(f"{spec.object_id} contact split is too small")
    frequencies, damping, gains = _modal_truth(spec, uv, normals, curvatures)
    return ObjectData(
        spec=spec,
        uv=uv,
        vertices=vertices,
        normals=normals,
        curvatures=curvatures,
        faces=faces,
        edges=edges,
        edge_lengths=edge_lengths,
        all_geodesic=geodesic,
        geodesic_diameter=diameter,
        context_indices=context,
        query_indices=query,
        frequencies=frequencies,
        damping=damping,
        gains=gains,
        mesh_sha256=_mesh_identity(spec, vertices, faces),
    )


def generate_corpus() -> tuple[ObjectData, ...]:
    corpus = tuple(generate_object(spec) for spec in OBJECT_SPECS)
    role_counts = {
        role: sum(item.spec.role == role for item in corpus)
        for role in ("train", "development", "test")
    }
    if role_counts != {"train": 9, "development": 3, "test": 6}:
        raise L0Error(f"corpus role counts changed: {role_counts}")
    return corpus


def static_features(spec: ObjectSpec, geometry_aware: bool = True) -> np.ndarray:
    material = MATERIALS[spec.material]
    material_one_hot = [float(spec.material == value) for value in MATERIAL_ORDER]
    support_one_hot = [float(spec.support == value) for value in SUPPORT_ORDER]
    if not geometry_aware:
        return np.asarray(material_one_hot + support_one_hot, dtype=np.float64)
    surface_one_hot = [float(spec.surface == value) for value in SURFACE_ORDER]
    return np.asarray(
        material_one_hot
        + support_one_hot
        + surface_one_hot
        + [
            math.log(spec.length_m),
            math.log(spec.aspect),
            math.log(spec.wall_m),
            math.log(material["density"]),
            math.log(material["wave_speed"]),
        ],
        dtype=np.float64,
    )


def local_features(data: ObjectData, geometry_aware: bool = True) -> np.ndarray:
    if not geometry_aware:
        return np.asarray(data.uv, dtype=np.float64)
    minimum = np.min(data.vertices, axis=0)
    maximum = np.max(data.vertices, axis=0)
    center = 0.5 * (minimum + maximum)
    half_extent = 0.5 * (maximum - minimum)
    safe_extent = np.where(half_extent > 1.0e-12, half_extent, 1.0)
    xyz = (data.vertices - center) / safe_extent
    curvature = data.curvatures * data.spec.length_m
    return np.column_stack((data.uv, xyz, data.normals, curvature))


@dataclass(frozen=True)
class Normalizer:
    mean: np.ndarray
    scale: np.ndarray

    def apply(self, value: np.ndarray) -> np.ndarray:
        return (np.asarray(value, dtype=np.float64) - self.mean) / self.scale

    def record(self) -> dict[str, Any]:
        return {"mean": self.mean, "scale": self.scale}


def fit_normalizer(values: Iterable[np.ndarray]) -> Normalizer:
    matrix = np.concatenate([np.atleast_2d(value) for value in values], axis=0)
    mean = np.mean(matrix, axis=0)
    scale = np.std(matrix, axis=0)
    scale = np.where(scale > 1.0e-12, scale, 1.0)
    return Normalizer(mean=mean, scale=scale)


def inverse_softplus(value: np.ndarray) -> np.ndarray:
    value = np.asarray(value, dtype=np.float64)
    return value + np.log(-np.expm1(-value))


def render_modal(
    frequencies: np.ndarray, damping: np.ndarray, gains: np.ndarray
) -> np.ndarray:
    time = np.arange(SAMPLE_COUNT, dtype=np.float64) / SAMPLE_RATE_HZ
    basis = np.exp(-damping[:, None] * time[None, :]) * np.sin(
        2.0 * np.pi * frequencies[:, None] * time[None, :]
    )
    value = np.asarray(gains, dtype=np.float64) @ basis
    if not np.isfinite(value).all():
        raise L0Error("modal renderer produced non-finite PCM")
    return value


def nrmse(prediction: np.ndarray, truth: np.ndarray, axis: Any = None) -> np.ndarray:
    numerator = np.sqrt(np.mean((prediction - truth) ** 2, axis=axis))
    denominator = np.sqrt(np.mean(truth**2, axis=axis))
    if np.any(denominator <= 1.0e-12):
        raise L0Error("NRMSE truth denominator is zero")
    return numerator / denominator


def multiresolution_spectrum_rmse(
    prediction: np.ndarray, truth: np.ndarray
) -> np.ndarray:
    prediction = np.atleast_2d(np.asarray(prediction, dtype=np.float64))
    truth = np.atleast_2d(np.asarray(truth, dtype=np.float64))
    prediction_peak = np.max(np.abs(prediction), axis=1, keepdims=True)
    truth_peak = np.max(np.abs(truth), axis=1, keepdims=True)
    if np.any(truth_peak <= 1.0e-12):
        raise L0Error("spectrum metric received a zero-peak truth waveform")
    active_prediction = prediction_peak[:, 0] > 1.0e-12
    result = np.full(prediction.shape[0], 80.0, dtype=np.float64)
    if not np.any(active_prediction):
        return result
    prediction = prediction[active_prediction] / prediction_peak[active_prediction]
    truth = truth[active_prediction]
    truth_peak = truth_peak[active_prediction]
    truth = truth / truth_peak
    results: list[np.ndarray] = []
    for length in (512, 1_024, 2_048, 4_096):
        index = np.arange(length, dtype=np.float64)
        window = 0.5 - 0.5 * np.cos(2.0 * np.pi * index / length)
        pred_spectrum = np.abs(np.fft.rfft(prediction[:, :length] * window, axis=1))
        truth_spectrum = np.abs(np.fft.rfft(truth[:, :length] * window, axis=1))
        pred_spectrum /= np.maximum(np.max(pred_spectrum, axis=1, keepdims=True), 1.0e-12)
        truth_spectrum /= np.maximum(np.max(truth_spectrum, axis=1, keepdims=True), 1.0e-12)
        pred_db = 20.0 * np.log10(np.maximum(pred_spectrum, 1.0e-4))
        truth_db = 20.0 * np.log10(np.maximum(truth_spectrum, 1.0e-4))
        results.append(np.sqrt(np.mean((pred_db - truth_db) ** 2, axis=1)))
    result[active_prediction] = np.mean(np.stack(results, axis=1), axis=1)
    return result


def object_metrics(
    data: ObjectData,
    frequencies: np.ndarray,
    damping: np.ndarray,
    query_gains: np.ndarray,
) -> dict[str, Any]:
    query = data.query_indices
    truth_gains = data.gains[query]
    truth_pcm = render_modal(data.frequencies, data.damping, truth_gains)
    prediction_pcm = render_modal(frequencies, damping, query_gains)
    frequency_cents = np.abs(1_200.0 * np.log2(frequencies / data.frequencies))
    damping_relative = np.abs(damping - data.damping) / data.damping
    truth_envelope = np.abs(hilbert(truth_pcm, axis=1))
    prediction_envelope = np.abs(hilbert(prediction_pcm, axis=1))
    envelope_denominator = float(np.sqrt(np.mean(truth_envelope**2)))
    if envelope_denominator <= 1.0e-12:
        raise L0Error("object envelope truth denominator is zero")
    envelope_per_query = np.sqrt(
        np.mean((prediction_envelope - truth_envelope) ** 2, axis=1)
    ) / envelope_denominator
    truth_peaks = np.max(np.abs(truth_pcm), axis=1)
    active = truth_peaks > 1.0e-12
    if not np.any(active):
        raise L0Error("object has no non-nodal query waveform")
    spectrum_per_query = multiresolution_spectrum_rmse(
        prediction_pcm[active], truth_pcm[active]
    )
    zero_prediction_count = int(
        np.sum(np.max(np.abs(prediction_pcm[active]), axis=1) <= 1.0e-12)
    )
    return {
        "damping_relative": damping_relative,
        "frequency_cents": frequency_cents,
        "gain_nrmse": float(nrmse(query_gains, truth_gains)),
        "envelope_nrmse_p95": float(np.quantile(envelope_per_query, 0.95)),
        "nodal_query_count": int(np.sum(~active)),
        "spectrum_rmse_db_mean": float(np.mean(spectrum_per_query)),
        "waveform_nrmse_mean": float(nrmse(prediction_pcm, truth_pcm)),
        "zero_prediction_for_active_truth_count": zero_prediction_count,
    }


def continuity_metric(data: ObjectData, all_predicted_gains: np.ndarray) -> float:
    truth_delta = data.gains[data.edges[:, 0]] - data.gains[data.edges[:, 1]]
    predicted_delta = (
        all_predicted_gains[data.edges[:, 0]] - all_predicted_gains[data.edges[:, 1]]
    )
    edge_error = np.sqrt(np.mean((predicted_delta - truth_delta) ** 2, axis=1))
    truth_rms = float(np.sqrt(np.mean(data.gains**2)))
    if truth_rms <= 1.0e-12:
        raise L0Error("continuity truth denominator is zero")
    return float(np.quantile(edge_error / truth_rms, 0.99))


def gain_controls(data: ObjectData, query_indices: np.ndarray | None = None) -> dict[str, np.ndarray]:
    context = data.context_indices
    query = data.query_indices if query_indices is None else query_indices
    context_xyz = data.vertices[context]
    query_xyz = data.vertices[query]
    context_gains = data.gains[context]
    euclidean = np.linalg.norm(
        query_xyz[:, None, :] - context_xyz[None, :, :], axis=2
    )
    nearest = np.argmin(euclidean, axis=1)
    bbox_diagonal = float(
        np.linalg.norm(np.max(data.vertices, axis=0) - np.min(data.vertices, axis=0))
    )
    bandwidth = max(0.25 * bbox_diagonal, 1.0e-12)
    rbf_weight = np.exp(-0.5 * (euclidean / bandwidth) ** 2)
    rbf_weight /= np.sum(rbf_weight, axis=1, keepdims=True)

    geodesic = data.all_geodesic[np.ix_(query, context)]
    geodesic_bandwidth = 0.25 * data.geodesic_diameter
    geodesic_weight = np.exp(-0.5 * (geodesic / geodesic_bandwidth) ** 2)
    geodesic_weight /= np.sum(geodesic_weight, axis=1, keepdims=True)

    local_linear = np.empty((query.size, MODE_COUNT), dtype=np.float64)
    for row in range(query.size):
        neighbour = np.argsort(euclidean[row], kind="stable")[:8]
        delta = context_xyz[neighbour] - query_xyz[row]
        design = np.column_stack((np.ones(8), delta))
        gram = design.T @ design + 1.0e-3 * np.eye(4)
        coefficient = np.linalg.solve(gram, design.T @ context_gains[neighbour])
        local_linear[row] = coefficient[0]

    return {
        "context_mean": np.tile(np.mean(context_gains, axis=0), (query.size, 1)),
        "nearest_context": context_gains[nearest],
        "euclidean_rbf": rbf_weight @ context_gains,
        "geodesic_rbf": geodesic_weight @ context_gains,
        "local_linear_ridge": local_linear,
    }


def global_controls(
    train: tuple[ObjectData, ...], target: ObjectData, static_normalizer: Normalizer
) -> dict[str, tuple[np.ndarray, np.ndarray]]:
    material_rows = [item for item in train if item.spec.material == target.spec.material]
    material_frequency = np.mean([item.frequencies for item in material_rows], axis=0)
    material_damping = np.mean([item.damping for item in material_rows], axis=0)
    target_static = static_normalizer.apply(static_features(target.spec))
    train_static = np.stack(
        [static_normalizer.apply(static_features(item.spec)) for item in train]
    )
    nearest_index = int(np.argmin(np.linalg.norm(train_static - target_static, axis=1)))
    nearest = train[nearest_index]
    return {
        "material_mean": (material_frequency, material_damping),
        "nearest_train_object": (nearest.frequencies, nearest.damping),
    }


def ood_raw_components(
    data: ObjectData,
    query_indices: np.ndarray,
    ensemble_gains: np.ndarray,
    train_gain_rms: np.ndarray,
    raw_static: np.ndarray,
    train_static_minimum: np.ndarray,
    train_static_maximum: np.ndarray,
    context_indices: np.ndarray | None = None,
) -> np.ndarray:
    if ensemble_gains.ndim != 3 or ensemble_gains.shape[1:] != (
        query_indices.size,
        MODE_COUNT,
    ):
        raise L0Error("OOD ensemble gain shape changed")
    disagreement = np.std(ensemble_gains, axis=0, ddof=0)
    disagreement_score = np.max(
        disagreement / np.maximum(train_gain_rms[None, :], 1.0e-12), axis=1
    )
    train_range = train_static_maximum - train_static_minimum
    outside = np.maximum(
        np.maximum(train_static_minimum - raw_static, raw_static - train_static_maximum),
        0.0,
    )
    static_terms = np.where(
        train_range > 1.0e-12,
        outside / np.maximum(train_range, 1.0e-12),
        np.where(outside > 0.0, np.inf, 0.0),
    )
    static_score = float(np.max(static_terms))
    context = data.context_indices if context_indices is None else context_indices
    distances = np.linalg.norm(
        data.vertices[query_indices, None, :] - data.vertices[context][None, :, :],
        axis=2,
    )
    coverage = np.min(distances, axis=1) / data.geodesic_diameter
    return np.column_stack(
        (
            disagreement_score,
            np.full(query_indices.size, static_score),
            coverage,
        )
    )


def calibrate_ood(
    development_raw: np.ndarray,
    raw: np.ndarray,
) -> tuple[np.ndarray, np.ndarray, float]:
    maxima = np.max(development_raw, axis=0)
    calibrated = np.empty_like(raw)
    development_calibrated = np.empty_like(development_raw)
    for column, maximum in enumerate(maxima):
        if column == 1:
            calibrated[:, column] = raw[:, column] / 0.25
            development_calibrated[:, column] = development_raw[:, column] / 0.25
        elif maximum > 0.0:
            calibrated[:, column] = raw[:, column] / max(maximum, 1.0e-12)
            development_calibrated[:, column] = development_raw[:, column] / max(
                maximum, 1.0e-12
            )
        else:
            calibrated[:, column] = np.where(raw[:, column] > 0.0, np.inf, 0.0)
            development_calibrated[:, column] = 0.0
    development_score = np.max(development_calibrated, axis=1)
    threshold = max(1.0, 1.25 * float(np.max(development_score)))
    return np.max(calibrated, axis=1), maxima, threshold


def hard_validate_modes(frequencies: np.ndarray, damping: np.ndarray) -> bool:
    return bool(
        np.isfinite(frequencies).all()
        and np.isfinite(damping).all()
        and np.all(np.diff(frequencies) > 0.0)
        and frequencies[0] >= 40.0
        and frequencies[-1] <= 12_000.0
        and np.all(damping > 0.0)
        and np.all(damping <= 128.0)
    )


def array_bytes(value: np.ndarray) -> bytes:
    buffer = BytesIO()
    np.save(buffer, np.asarray(value), allow_pickle=False)
    return buffer.getvalue()


def model_parameter_bytes(state: dict[str, torch.Tensor]) -> bytes:
    output = bytearray(b"NEXTENGINE-L0-MODEL\0")
    for name in sorted(state):
        value = state[name].detach().cpu().numpy().astype("<f8", copy=False)
        encoded = name.encode("utf-8")
        output.extend(struct.pack("<I", len(encoded)))
        output.extend(encoded)
        output.extend(struct.pack("<I", value.ndim))
        for dimension in value.shape:
            output.extend(struct.pack("<Q", dimension))
        output.extend(value.tobytes(order="C"))
    return bytes(output)
