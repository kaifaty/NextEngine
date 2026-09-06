#!/usr/bin/env python3
"""Frozen corpus, serialization and output boundary for Physical Sound V19 C0."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import shutil
import zipfile
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
import physical_sound_v18_o0_common as legacy
import scipy

STUDY_ID = "physical-sound-v19-c0-composite-coverage"
REVISION = "integrity-fill-reachability-v0"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v19-c0.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v19-c0.corpus.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v19-c0.report.v0"
DECISION_SCHEMA = "nextengine.experimental-physical-sound-v19-c0.decision.v0"
CALIBRATION_SCHEMA = "nextengine.experimental-physical-sound-v19-c0.calibration.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v19-c0.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/physical-sound-v19-p0a-composite-coverage-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "ea28f844a63036fc6273f829baa6ff8f405050152367da7c8530a1683c315e5d"
PARENT_PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md"
)
PARENT_PROTOCOL_SHA256 = (
    "39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e"
)
LEGACY_COMMON_FILE = "physical_sound_v18_o0_common.py"
LEGACY_COMMON_SHA256 = (
    "baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad"
)
IMPLEMENTATION_FILES = (
    "physical_sound_v19_c0_common.py",
    "physical_sound_v19_c0_oracle.py",
)
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
}

MATERIAL_ORDER = ("Steel", "Wood", "Glass")
TOPOLOGY_ORDER = ("Plate", "Cylinder", "Bowl", "RolledSheet")
SUPPORT_ORDER = ("Free", "BaseClamped")
ROLE_ORDER = ("development", "test")
ROLE_SPECS = {
    "development": {"base": 1101, "support_offset": 0},
    "test": {"base": 1201, "support_offset": 1},
}
DEVELOPMENT_RECORD_ROOT = (
    "b3af64b6ed2b9c4f7bafbbcd16a614a15db9609763731898d05046c8db5d8b52"
)
DEVELOPMENT_MESH_HASHES = (
    "29e5c434f246682a5eaa3cf374a51208397a9d3928a00c9c8639d4e79a7ae8ef",
    "8ae090b0d076a74fe8bd963d7f601451e926375032ed106db981d6771f8762b5",
    "1638f9c5ea9259aff1e893e4f0ff9da03ef6c4f2e830089f0cfae913833e1f34",
    "1a071f694491d4e45a828bac6f868f1ac4fea97361f82d0994f17454476a1182",
    "5c73fb313e4d0657af64f9f4283a3b81bbfaf9c4ccbc4aa4a78a39a89b23893f",
    "44cf05e811445a5e57d777404ca97df236c68f758e026b62dbb69b5391904159",
    "9e2d8310540c23b27035a73e03e1743a28ded0b04873b26af1bba8f498b42aac",
    "494a31c9d83c7250ea07ca509c4a38cc2428769ca8a5559a02081c0a2a5bbfa1",
    "590dce3c53471c59e1216877f1fdb24e96ed62b62f124e91c455a2399c4122a8",
    "ee63418f0b06a1a59bb97189d56a3d37d693f01f9101225d6d6415f99399dfd7",
    "519e6afc6e02168733a7c281d9ec17ca9d1d9a1f0f807d5ec9238313837b2a49",
    "943af29fcc9629f3e6e4fa5d2a186d0e77f7a9392d1bdd081f947fc13e85b229",
)

LOCAL_INTRINSIC_THRESHOLD = 0.11869598258542362
GLOBAL_INTRINSIC_THRESHOLD = 0.1332105169640669
LOCAL_EUCLIDEAN_THRESHOLD = 0.16904819773036295
GLOBAL_EUCLIDEAN_THRESHOLD = 0.19098768258200008

EXPERIMENT_ROOT = Path("/home/kaifaty/.codex/experiments/nextengine/physical-sound")
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
    "opened_v17_artifact_bytes_read": 0,
    "opened_v18_artifact_bytes_read": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
    "test_rows_generated_before_implementation_commit": 0,
}

FieldRow = legacy.FieldRow
Mesh = legacy.Mesh


class C0Error(RuntimeError):
    """Stable failure for the frozen C0 experiment."""


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


def canonical_json_lines(records: list[dict[str, Any]]) -> bytes:
    return b"".join(canonical_json(record) for record in records)


def array_bytes(value: np.ndarray) -> bytes:
    array = np.asarray(value)
    if array.dtype.kind not in "fiu":
        raise C0Error(f"unsupported canonical C0 array dtype: {array.dtype}")
    if array.dtype.kind == "f" and not np.isfinite(array).all():
        raise C0Error("canonical C0 array is non-finite")
    little = np.ascontiguousarray(
        array.astype(array.dtype.newbyteorder("<"), copy=False)
    )
    buffer = BytesIO()
    np.save(buffer, little, allow_pickle=False)
    return buffer.getvalue()


def deterministic_npz(arrays: dict[str, np.ndarray]) -> bytes:
    buffer = BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in sorted(arrays):
            if not name or any(
                character not in "abcdefghijklmnopqrstuvwxyz0123456789_"
                for character in name
            ):
                raise C0Error(f"invalid C0 NPZ member name: {name}")
            info = zipfile.ZipInfo(f"{name}.npy", date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, array_bytes(np.asarray(arrays[name])))
    return buffer.getvalue()


def _grid(role: str, cell: int) -> tuple[int, int]:
    if role == "development":
        return 22 + cell % 2, 18 + cell % 3
    if role == "test":
        return 24 + cell % 2, 19 + cell % 3
    raise C0Error(f"unknown C0 role: {role}")


def _make_row(role: str, cell: int) -> FieldRow:
    if role not in ROLE_SPECS:
        raise C0Error(f"unknown C0 role: {role}")
    if not 0 <= cell < 12:
        raise C0Error(f"C0 cell is out of range: {cell}")
    spec = ROLE_SPECS[role]
    n = int(spec["base"]) + cell
    material = MATERIAL_ORDER[cell // 4]
    topology = TOPOLOGY_ORDER[cell % 4]
    support = SUPPORT_ORDER[(cell + int(spec["support_offset"])) % 2]
    length_m = 0.19 + 0.31 * legacy.radical_inverse(n, 2)
    aspect = 0.68 + 0.88 * legacy.radical_inverse(n, 3)
    slenderness = 0.0035 + 0.0050 * legacy.radical_inverse(n, 5)
    grid_u, grid_v = _grid(role, cell)
    physical_group_id = f"v19-c0-{role}-{material}-{topology}-{support}-{n}".lower()
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
        wall_m=length_m * slenderness,
        grid_u=grid_u,
        grid_v=grid_v,
    )


def generate_rows(role: str) -> tuple[FieldRow, ...]:
    rows = tuple(_make_row(role, cell) for cell in range(12))
    if len({row.object_id for row in rows}) != 12:
        raise C0Error("C0 object identity collision")
    return rows


def build_mesh(row: FieldRow) -> Mesh:
    try:
        return legacy.build_mesh(row)
    except legacy.O0Error as error:
        raise C0Error(f"C0 mesh construction failed: {error}") from error


def generate_meshes(role: str) -> tuple[Mesh, ...]:
    meshes = tuple(build_mesh(row) for row in generate_rows(role))
    if len({mesh.mesh_hash for mesh in meshes}) != len(meshes):
        raise C0Error("C0 mesh hash collision")
    if role == "development":
        observed = sha256_bytes(canonical_json([mesh.record() for mesh in meshes]))
        if observed != DEVELOPMENT_RECORD_ROOT:
            raise C0Error(f"C0 development record root changed: {observed}")
        hashes = tuple(mesh.mesh_hash for mesh in meshes)
        if hashes != DEVELOPMENT_MESH_HASHES:
            raise C0Error("C0 development mesh hashes changed")
    return meshes


def required_context_count(vertex_count: int) -> int:
    if vertex_count <= 0:
        raise C0Error("C0 vertex count must be positive")
    return max(16, math.ceil(vertex_count / 8))


def identity_hash(value: Any) -> str:
    if isinstance(value, np.ndarray):
        return sha256_bytes(array_bytes(value))
    return sha256_bytes(canonical_json(value))


def geometry_npz(meshes: tuple[Mesh, ...]) -> bytes:
    vertices: list[np.ndarray] = []
    uv: list[np.ndarray] = []
    faces: list[np.ndarray] = []
    edges: list[np.ndarray] = []
    edge_lengths: list[np.ndarray] = []
    vertex_offsets = [0]
    face_offsets = [0]
    edge_offsets = [0]
    for mesh in meshes:
        offset = vertex_offsets[-1]
        vertices.append(mesh.vertices)
        uv.append(mesh.uv)
        faces.append(mesh.faces + offset)
        edges.append(mesh.edges + offset)
        edge_lengths.append(mesh.edge_lengths)
        vertex_offsets.append(offset + mesh.vertex_count)
        face_offsets.append(face_offsets[-1] + mesh.faces.shape[0])
        edge_offsets.append(edge_offsets[-1] + mesh.edges.shape[0])
    return deterministic_npz(
        {
            "edge_lengths": np.concatenate(edge_lengths),
            "edge_offsets": np.asarray(edge_offsets, dtype=np.int64),
            "edges": np.concatenate(edges, axis=0),
            "face_offsets": np.asarray(face_offsets, dtype=np.int64),
            "faces": np.concatenate(faces, axis=0),
            "uv": np.concatenate(uv, axis=0),
            "vertex_offsets": np.asarray(vertex_offsets, dtype=np.int64),
            "vertices": np.concatenate(vertices, axis=0),
        }
    )


def verify_protocol_environment() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        PARENT_PROTOCOL_PATH: PARENT_PROTOCOL_SHA256,
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise C0Error(f"C0 dependency hash changed for {path}: {observed}")
    legacy_path = Path(legacy.__file__).resolve()
    observed_legacy = sha256_file(legacy_path)
    if observed_legacy != LEGACY_COMMON_SHA256:
        raise C0Error(f"C0 geometry dependency changed: {observed_legacy}")
    observed_environment = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
    }
    if observed_environment != REQUIRED_ENVIRONMENT:
        raise C0Error(f"C0 environment changed: {observed_environment}")
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
        raise C0Error(f"C0 thread environment changed: {threads}")
    return {
        "geometry_dependency_sha256": observed_legacy,
        "libraries": observed_environment,
        "threads": threads,
    }


def implementation_hashes() -> dict[str, str]:
    root = Path(__file__).resolve().parent
    return {name: sha256_file(root / name) for name in IMPLEMENTATION_FILES}


def _check_no_symlink(path: Path, root: Path) -> None:
    current = path
    while current != root:
        if current.is_symlink():
            raise C0Error(f"symlinked C0 output path is forbidden: {current}")
        if current.parent == current:
            raise C0Error("C0 output path escaped experiment root")
        current = current.parent


def prepare_output(output: Path) -> tuple[Path, Path]:
    if not output.is_absolute():
        raise C0Error("C0 output path must be absolute")
    root = EXPERIMENT_ROOT.resolve()
    resolved = output.resolve(strict=False)
    if resolved == root or not resolved.is_relative_to(root):
        raise C0Error("C0 output path must be below the external experiment root")
    _check_no_symlink(output, EXPERIMENT_ROOT)
    if output.exists():
        raise C0Error("C0 output path must not already exist")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.with_name(f".{output.name}.staging-{os.getpid()}")
    if staging.exists() or staging.is_symlink():
        raise C0Error("C0 staging path already exists")
    staging.mkdir()
    return staging, output


def publish_output(staging: Path, output: Path) -> None:
    if output.exists():
        raise C0Error("C0 output appeared before atomic publication")
    os.replace(staging, output)


def abandon_output(staging: Path) -> None:
    if staging.exists() and staging.is_dir() and not staging.is_symlink():
        shutil.rmtree(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    for name in sorted(files):
        if Path(name).name != name:
            raise C0Error(f"invalid C0 artifact name: {name}")
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
