#!/usr/bin/env python3
"""Frozen mesh and artifact boundary for Physical Sound V18 O0."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import shutil
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
from typing import Any

for _thread_variable in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
):
    os.environ[_thread_variable] = "1"

import numpy as np
import scipy


STUDY_ID = "physical-sound-v18-o0-intrinsic-coverage"
REVISION = "hybrid-truth-intrinsic-coverage-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v18-o0.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v18-o0.report.v1"
PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v18-p0b-hybrid-truth-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "216acd012108725cbcbcf5454f1db707a6ac688e182bfd9fa92d45082280475a"
PARENT_PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md"
)
PARENT_PROTOCOL_SHA256 = (
    "39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e"
)
B0_RESULT_PATH = Path(
    "docs/development/"
    "physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md"
)
B0_RESULT_SHA256 = "c275dc49664e91064f01a9123727a8f1cc5c61b8a2b47a95f1b942f56680ce75"
IMPLEMENTATION_FILES = (
    "physical_sound_v18_o0_common.py",
    "physical_sound_v18_o0_oracle.py",
)
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
}

MATERIAL_ORDER = ("Steel", "Wood", "Glass")
TOPOLOGY_ORDER = ("Plate", "Cylinder", "Bowl", "RolledSheet")
SUPPORT_ORDER = ("Free", "BaseClamped")
MATERIAL_DENSITY = {"Steel": 7_850.0, "Wood": 650.0, "Glass": 2_500.0}
EXPERIMENT_ROOT = Path(
    "/home/kaifaty/.codex/experiments/nextengine/physical-sound"
)
ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "b0_external_artifact_bytes_read": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "f0_model_parameters_loaded": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "integration_rows_generated": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "opened_v16_artifact_bytes_read": 0,
    "opened_v17_g0_artifact_bytes_read": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
}


class O0Error(RuntimeError):
    """Stable failure for the frozen O0 experiment."""


@dataclass(frozen=True)
class FieldRow:
    physical_group_id: str
    object_id: str
    role: str
    cell: int
    halton_index: int
    material: str
    topology: str
    support: str
    length_m: float
    aspect: float
    slenderness: float
    wall_m: float
    grid_u: int
    grid_v: int

    def record(self) -> dict[str, Any]:
        return {
            "aspect": self.aspect,
            "cell": self.cell,
            "grid_u": self.grid_u,
            "grid_v": self.grid_v,
            "halton_index": self.halton_index,
            "length_m": self.length_m,
            "material": self.material,
            "object_id": self.object_id,
            "physical_group_id": self.physical_group_id,
            "role": self.role,
            "slenderness": self.slenderness,
            "support": self.support,
            "topology": self.topology,
            "wall_m": self.wall_m,
        }


@dataclass(frozen=True)
class Mesh:
    row: FieldRow
    uv: np.ndarray
    vertices: np.ndarray
    faces: np.ndarray
    edges: np.ndarray
    edge_lengths: np.ndarray
    mesh_hash: str

    @property
    def vertex_count(self) -> int:
        return int(self.vertices.shape[0])

    def record(self) -> dict[str, Any]:
        return {
            "edge_count": int(self.edges.shape[0]),
            "face_count": int(self.faces.shape[0]),
            "mesh_hash": self.mesh_hash,
            "row": self.row.record(),
            "vertex_count": self.vertex_count,
        }


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def _json_value(value: Any) -> Any:
    if isinstance(value, dict):
        return {str(key): _json_value(item) for key, item in value.items()}
    if isinstance(value, (tuple, list)):
        return [_json_value(item) for item in value]
    if isinstance(value, np.ndarray):
        return _json_value(value.tolist())
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, Path):
        return value.as_posix()
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(
            _json_value(value),
            allow_nan=False,
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )
        + "\n"
    ).encode("utf-8")


def array_bytes(value: np.ndarray) -> bytes:
    array = np.asarray(value)
    if array.dtype.kind not in "fiu":
        raise O0Error(f"unsupported canonical O0 array dtype: {array.dtype}")
    if array.dtype.kind == "f" and not np.isfinite(array).all():
        raise O0Error("canonical O0 geometry array is non-finite")
    little = np.ascontiguousarray(array.astype(array.dtype.newbyteorder("<"), copy=False))
    buffer = BytesIO()
    np.save(buffer, little, allow_pickle=False)
    return buffer.getvalue()


def radical_inverse(index: int, base: int) -> float:
    if index <= 0 or base <= 1:
        raise O0Error("radical inverse requires positive index and base > 1")
    result = 0.0
    factor = 1.0
    value = index
    while value:
        factor /= base
        result += factor * (value % base)
        value //= base
    return result


def _grid(role: str, cell: int) -> tuple[int, int]:
    if role == "development":
        return 17 + cell % 2, 15 + cell % 3
    if role == "test":
        return 19 + cell % 2, 16 + cell % 3
    raise O0Error(f"O0 cannot generate sealed role: {role}")


def _make_row(role: str, cell: int) -> FieldRow:
    if not 0 <= cell < 12:
        raise O0Error(f"O0 field cell is out of range: {cell}")
    base = {"development": 501, "test": 601}.get(role)
    if base is None:
        raise O0Error(f"O0 cannot generate sealed role: {role}")
    n = base + cell
    material = MATERIAL_ORDER[cell // 4]
    topology = TOPOLOGY_ORDER[cell % 4]
    support = SUPPORT_ORDER[(n + cell) % 2]
    length_m = 0.21 + 0.27 * radical_inverse(n, 2)
    aspect = 0.72 + 0.76 * radical_inverse(n, 3)
    slenderness = 0.004 + 0.004 * radical_inverse(n, 5)
    wall_m = length_m * slenderness
    grid_u, grid_v = _grid(role, cell)
    physical_group_id = f"f-{role}-{material}-{topology}-{support}-0".lower()
    return FieldRow(
        physical_group_id=physical_group_id,
        object_id=f"{physical_group_id}-primary",
        role=role,
        cell=cell,
        halton_index=n,
        material=material,
        topology=topology,
        support=support,
        length_m=length_m,
        aspect=aspect,
        slenderness=slenderness,
        wall_m=wall_m,
        grid_u=grid_u,
        grid_v=grid_v,
    )


def generate_rows(role: str) -> tuple[FieldRow, ...]:
    rows = tuple(_make_row(role, cell) for cell in range(12))
    if len({row.object_id for row in rows}) != 12:
        raise O0Error("O0 object identity collision")
    return rows


def _uv(row: FieldRow) -> np.ndarray:
    periodic_u = row.topology in ("Cylinder", "Bowl")
    if periodic_u:
        u = -1.0 + 2.0 * np.arange(row.grid_u, dtype=np.float64) / row.grid_u
    else:
        u = np.linspace(-1.0, 1.0, row.grid_u, dtype=np.float64)
    if row.topology == "Bowl":
        v = -1.0 + 2.0 * (
            np.arange(row.grid_v, dtype=np.float64) + 0.5
        ) / row.grid_v
    else:
        v = np.linspace(-1.0, 1.0, row.grid_v, dtype=np.float64)
    return np.asarray([(u_value, v_value) for v_value in v for u_value in u])


def _vertices(row: FieldRow, uv: np.ndarray) -> np.ndarray:
    u = uv[:, 0]
    v = uv[:, 1]
    if row.topology == "Plate":
        return np.column_stack(
            (0.5 * row.length_m * u, 0.5 * row.length_m / row.aspect * v, np.zeros_like(u))
        )
    theta = math.pi * (u + 1.0)
    if row.topology == "Bowl":
        alpha = math.pi * (v + 1.0) / 4.0
        radial = 0.5 * row.length_m
        axial = 0.5 * row.length_m * row.aspect
        return np.column_stack(
            (
                radial * np.sin(alpha) * np.cos(theta),
                radial * np.sin(alpha) * np.sin(theta),
                -axial * np.cos(alpha),
            )
        )
    if row.topology in ("Cylinder", "RolledSheet"):
        radius = row.length_m / (2.0 * math.pi)
        height = row.length_m * row.aspect
        return np.column_stack(
            (radius * np.cos(theta), radius * np.sin(theta), 0.5 * height * v)
        )
    raise O0Error(f"unknown O0 topology: {row.topology}")


def _faces(row: FieldRow) -> np.ndarray:
    wrap_u = row.topology in ("Cylinder", "Bowl")
    faces: list[tuple[int, int, int]] = []
    u_cells = row.grid_u if wrap_u else row.grid_u - 1
    for v_index in range(row.grid_v - 1):
        for u_index in range(u_cells):
            next_u = (u_index + 1) % row.grid_u
            a = v_index * row.grid_u + u_index
            b = v_index * row.grid_u + next_u
            c = (v_index + 1) * row.grid_u + next_u
            d = (v_index + 1) * row.grid_u + u_index
            faces.append((a, b, c))
            faces.append((a, c, d))
    return np.asarray(faces, dtype=np.int64)


def _edges(vertices: np.ndarray, faces: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    pairs: set[tuple[int, int]] = set()
    for face in faces:
        for left, right in ((face[0], face[1]), (face[1], face[2]), (face[2], face[0])):
            first, second = sorted((int(left), int(right)))
            if first == second:
                raise O0Error("O0 face contains a self edge")
            pairs.add((first, second))
    edges = np.asarray(sorted(pairs), dtype=np.int64)
    lengths = np.linalg.norm(vertices[edges[:, 0]] - vertices[edges[:, 1]], axis=1)
    if not np.isfinite(lengths).all() or np.any(lengths <= 0.0):
        raise O0Error("O0 edge length is non-finite or non-positive")
    return edges, lengths


def _mesh_hash(
    row: FieldRow, uv: np.ndarray, vertices: np.ndarray, faces: np.ndarray
) -> str:
    digest = hashlib.sha256()
    digest.update(canonical_json(row.record()))
    digest.update(array_bytes(uv))
    digest.update(array_bytes(vertices))
    digest.update(array_bytes(faces))
    return digest.hexdigest()


def build_mesh(row: FieldRow) -> Mesh:
    uv = _uv(row)
    vertices = _vertices(row, uv)
    faces = _faces(row)
    if uv.shape != (row.grid_u * row.grid_v, 2):
        raise O0Error("O0 UV shape changed")
    if vertices.shape != (uv.shape[0], 3) or not np.isfinite(vertices).all():
        raise O0Error("O0 vertex shape/value changed")
    if faces.ndim != 2 or faces.shape[1] != 3:
        raise O0Error("O0 face shape changed")
    if np.min(faces) < 0 or np.max(faces) >= vertices.shape[0]:
        raise O0Error("O0 face index is out of range")
    edges, edge_lengths = _edges(vertices, faces)
    return Mesh(
        row=row,
        uv=uv,
        vertices=vertices,
        faces=faces,
        edges=edges,
        edge_lengths=edge_lengths,
        mesh_hash=_mesh_hash(row, uv, vertices, faces),
    )


def generate_meshes(role: str) -> tuple[Mesh, ...]:
    meshes = tuple(build_mesh(row) for row in generate_rows(role))
    if len({mesh.mesh_hash for mesh in meshes}) != len(meshes):
        raise O0Error("O0 mesh hash collision")
    return meshes


def geometry_files(meshes: tuple[Mesh, ...]) -> dict[str, bytes]:
    vertex_parts: list[np.ndarray] = []
    uv_parts: list[np.ndarray] = []
    face_parts: list[np.ndarray] = []
    records: list[dict[str, Any]] = []
    vertex_offset = 0
    face_offset = 0
    for mesh in meshes:
        vertex_parts.append(mesh.vertices)
        uv_parts.append(mesh.uv)
        face_parts.append(mesh.faces + vertex_offset)
        records.append(
            {
                **mesh.record(),
                "face_offset": face_offset,
                "vertex_offset": vertex_offset,
            }
        )
        vertex_offset += mesh.vertex_count
        face_offset += mesh.faces.shape[0]
    return {
        "geometry-faces.npy": array_bytes(np.concatenate(face_parts, axis=0)),
        "geometry-records.json": canonical_json(records),
        "geometry-uv.npy": array_bytes(np.concatenate(uv_parts, axis=0)),
        "geometry-vertices.npy": array_bytes(np.concatenate(vertex_parts, axis=0)),
    }


def verify_protocol_environment_and_b0() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        PARENT_PROTOCOL_PATH: PARENT_PROTOCOL_SHA256,
        B0_RESULT_PATH: B0_RESULT_SHA256,
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise O0Error(f"O0 dependency hash changed for {path}: {observed}")
    observed_environment = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
    }
    if observed_environment != REQUIRED_ENVIRONMENT:
        raise O0Error(f"O0 environment changed: {observed_environment}")
    threads = {
        name: os.environ.get(name)
        for name in (
            "OMP_NUM_THREADS",
            "OPENBLAS_NUM_THREADS",
            "MKL_NUM_THREADS",
            "NUMEXPR_NUM_THREADS",
        )
    }
    if any(value != "1" for value in threads.values()):
        raise O0Error(f"O0 thread environment changed: {threads}")
    return {"libraries": observed_environment, "threads": threads}


def implementation_hashes() -> dict[str, str]:
    root = Path(__file__).resolve().parent
    return {name: sha256_file(root / name) for name in IMPLEMENTATION_FILES}


def _check_no_symlink(path: Path, root: Path) -> None:
    current = path
    while current != root:
        if current.is_symlink():
            raise O0Error(f"symlinked O0 output path is forbidden: {current}")
        if current.parent == current:
            raise O0Error("O0 output path escaped experiment root")
        current = current.parent


def prepare_output(output: Path) -> tuple[Path, Path]:
    if not output.is_absolute():
        raise O0Error("O0 output path must be absolute")
    root = EXPERIMENT_ROOT.resolve()
    resolved = output.resolve(strict=False)
    if resolved == root or not resolved.is_relative_to(root):
        raise O0Error("O0 output path must be below the external experiment root")
    _check_no_symlink(output, EXPERIMENT_ROOT)
    if output.exists():
        raise O0Error("O0 output path must not already exist")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.with_name(f".{output.name}.staging-{os.getpid()}")
    if staging.exists() or staging.is_symlink():
        raise O0Error("O0 staging path already exists")
    staging.mkdir()
    return staging, output


def publish_output(staging: Path, output: Path) -> None:
    if output.exists():
        raise O0Error("O0 output appeared before atomic publication")
    os.replace(staging, output)


def abandon_output(staging: Path) -> None:
    if staging.exists() and staging.is_dir() and not staging.is_symlink():
        shutil.rmtree(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    for name in sorted(files):
        if Path(name).name != name:
            raise O0Error(f"invalid O0 artifact name: {name}")
        (directory / name).write_bytes(files[name])


def directory_file_map(directory: Path) -> dict[str, str]:
    return {
        path.name: sha256_file(path)
        for path in sorted(directory.iterdir(), key=lambda value: value.name)
        if path.is_file()
    }


def tree_digest(file_map: dict[str, str]) -> str:
    return sha256_bytes(canonical_json(file_map))


def compare_directories(left: Path, right: Path) -> dict[str, Any]:
    left_map = directory_file_map(left)
    right_map = directory_file_map(right)
    return {
        "byte_identical": left_map == right_map,
        "file_count": len(left_map),
        "left_tree_digest": tree_digest(left_map),
        "right_tree_digest": tree_digest(right_map),
    }
