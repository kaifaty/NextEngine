#!/usr/bin/env python3
"""Frozen corpus, truth, coverage and artifacts for Physical Sound V19 F0."""

from __future__ import annotations

import math
import os
import platform
import subprocess
from dataclasses import dataclass, replace
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
import physical_sound_v18_o0_common as geometry_common
import physical_sound_v19_c0_common as coverage_common
import physical_sound_v19_c0_oracle as coverage_oracle
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v19-f0-residual-harmonic-field"
REVISION = "residual-harmonic-operator-v0"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v19-f0.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v19-f0.corpus.v0"
DECISION_SCHEMA = "nextengine.experimental-physical-sound-v19-f0.decision.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v19-f0.report.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v19-f0.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/physical-sound-v19-p0b-field-integration-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "1c2681a3ac4d505118d08e485cff9e38b23e72a897e8d149b21c909cd11ac24d"
B0_RESULT_PATH = Path(
    "docs/development/"
    "physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md"
)
B0_RESULT_SHA256 = "c275dc49664e91064f01a9123727a8f1cc5c61b8a2b47a95f1b942f56680ce75"
C0_RESULT_PATH = Path(
    "docs/development/physical-sound-v19-c0-composite-coverage-result-2026-09-01.md"
)
C0_RESULT_SHA256 = "fc3141a354d900d5cdda57ece99ce4b45ed9c9334db4fdbadc5ffb9aefb30d42"
C0_COMMON_FILE = "physical_sound_v19_c0_common.py"
C0_COMMON_SHA256 = "1224038462ba54e94986419f5db0510125474fbb9487d964066a6b3200548024"
C0_ORACLE_FILE = "physical_sound_v19_c0_oracle.py"
C0_ORACLE_SHA256 = "be586995efd17efda30bf6631526a6315a2c04b65228d046d8218cb46b6c5e88"
GEOMETRY_FILE = "physical_sound_v18_o0_common.py"
GEOMETRY_SHA256 = "baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad"

IMPLEMENTATION_FILES = (
    "physical_sound_v19_f0_common.py",
    "physical_sound_v19_f0_model.py",
    "physical_sound_v19_f0_oracle.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v19_f0_oracle.py")
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
    "torch": "2.13.0+cu130",
}

MATERIAL_ORDER = ("Steel", "Wood", "Glass")
TOPOLOGY_ORDER = ("Plate", "Cylinder", "Bowl", "RolledSheet")
SUPPORT_ORDER = ("Free", "BaseClamped")
MATERIAL_DENSITY = {"Steel": 7_850.0, "Wood": 650.0, "Glass": 2_500.0}
MODE_COUNT = 8
K = np.asarray([1, 1, 2, 2, 3, 3, 4, 5], dtype=np.float64)
ELL = np.asarray([1, 2, 1, 3, 2, 4, 3, 2], dtype=np.float64)
RESIDUAL_K = np.asarray([1, 1, 2, 1, 2, 2, 2, 2], dtype=np.float64)
RESIDUAL_ELL = np.asarray([1, 2, 1, 2, 1, 2, 1, 2], dtype=np.float64)
IMPULSE_DIRECTION = np.asarray([0.27, -0.91, 0.31], dtype=np.float64)
IMPULSE_DIRECTION /= np.linalg.norm(IMPULSE_DIRECTION)

ROLE_BASE = {"train": 1301, "development": 1401, "test": 1501, "integration": 1601}
ROW_ROOTS = {
    "train": "273d1bd08b2a35d7a27116a01ce5f4deec34ff286b50966bbcfad6bd802293cf",
    "development": "825d0aa5de9bda2ffefc0737cfd8221f8e55585d3d6431b19bf5ef4db33806a2",
    "test": "94e66ca654a6bfa51525803ec578894348a358011ae209d5ac7d1b87bdfa6283",
    "integration": "2532b99e6332ab42c502720f4abf6fd6d46918a5636716ce5ffe23f7219b5a0b",
}
TRAIN_RECORD_ROOT = "39d517f2d467059e162f694a6b700fc3bc73c16cd8a73cc32329fefe72969718"
DEVELOPMENT_RECORD_ROOT = (
    "f21db879bdf497cb37d52a5dfd3fcc92d488c30d9ad9e013a93d348898f0785e"
)

EXPERIMENT_ROOT = Path("/home/kaifaty/.codex/experiments/nextengine/physical-sound")
ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "b0_external_artifact_bytes_read": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
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

FieldRow = geometry_common.FieldRow
Mesh = geometry_common.Mesh


class F0Error(RuntimeError):
    """Stable failure for the frozen F0 experiment."""


@dataclass(frozen=True)
class FieldObject:
    row: FieldRow
    mesh: Mesh
    analysis: coverage_oracle.MeshAnalysis
    context: np.ndarray
    query: np.ndarray
    accepted_query: np.ndarray
    rejected_query: np.ndarray
    coverage_input: coverage_oracle.CoverageInput
    coverage_decisions: tuple[dict[str, Any], ...]
    normals: np.ndarray
    curvatures: np.ndarray
    gains: np.ndarray

    @property
    def is_twin(self) -> bool:
        return self.row.object_id.endswith("-twin")

    def identity_record(self) -> dict[str, Any]:
        return {
            "context_hash": identity_hash(self.context),
            "gain_hash": identity_hash(self.gains),
            "mesh_hash": self.mesh.mesh_hash,
            "query_hash": identity_hash(self.query),
            "row": self.row.record(),
            "vertex_count": self.mesh.vertex_count,
        }

    def record(self) -> dict[str, Any]:
        reasons: dict[str, int] = {}
        for item in self.coverage_decisions:
            reasons[item["reason"]] = reasons.get(item["reason"], 0) + 1
        return {
            **self.identity_record(),
            "accepted_query_count": int(self.accepted_query.size),
            "accepted_query_hash": identity_hash(self.accepted_query),
            "coverage_reason_counts": reasons,
            "graph_hash": self.analysis.graph_hash,
            "is_twin": self.is_twin,
            "rejected_query_count": int(self.rejected_query.size),
            "rejected_query_hash": identity_hash(self.rejected_query),
        }


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


sha256_bytes = coverage_common.sha256_bytes
sha256_file = coverage_common.sha256_file
canonical_json = coverage_common.canonical_json
canonical_json_lines = coverage_common.canonical_json_lines
array_bytes = coverage_common.array_bytes
deterministic_npz = coverage_common.deterministic_npz
identity_hash = coverage_common.identity_hash


def _grid(role: str, cell: int, replicate: int) -> tuple[int, int]:
    if role == "train":
        return 13 + 2 * replicate + cell % 2, 12 + replicate + cell % 3
    if role == "development":
        return 17 + cell % 2, 15 + cell % 3
    if role == "test":
        return 19 + cell % 2, 16 + cell % 3
    if role == "integration":
        return 21 + cell % 2, 17 + cell % 3
    raise F0Error(f"unknown F0 role: {role}")


def _support_index(role: str, cell: int, replicate: int) -> int:
    if role == "train":
        return (cell + replicate) % 2
    if role in ("development", "integration"):
        return (cell + 1) % 2
    if role == "test":
        return cell % 2
    raise F0Error(f"unknown F0 role: {role}")


def _primary_row(role: str, cell: int, replicate: int = 0) -> FieldRow:
    if role not in ROLE_BASE or not 0 <= cell < 12:
        raise F0Error(f"invalid F0 row request: {role}/{cell}")
    if role == "train":
        if replicate not in (0, 1):
            raise F0Error("F0 train replicate is out of range")
        n = ROLE_BASE[role] + 2 * cell + replicate
    else:
        if replicate != 0:
            raise F0Error("non-train F0 replicate must be zero")
        n = ROLE_BASE[role] + cell
    material = MATERIAL_ORDER[cell // 4]
    topology = TOPOLOGY_ORDER[cell % 4]
    support = SUPPORT_ORDER[_support_index(role, cell, replicate)]
    length_m = 0.21 + 0.27 * geometry_common.radical_inverse(n, 2)
    aspect = 0.72 + 0.76 * geometry_common.radical_inverse(n, 3)
    slenderness = 0.004 + 0.004 * geometry_common.radical_inverse(n, 5)
    grid_u, grid_v = _grid(role, cell, replicate)
    group = f"v19-f0-{role}-{material}-{topology}-{support}-{n}".lower()
    return FieldRow(
        physical_group_id=group,
        object_id=f"{group}-primary",
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
    if role not in ROLE_BASE:
        raise F0Error(f"unknown F0 role: {role}")
    rows: list[FieldRow] = []
    for cell in range(12):
        replicates = range(2) if role == "train" else range(1)
        for replicate in replicates:
            primary = _primary_row(role, cell, replicate)
            rows.append(primary)
            if role != "train":
                rows.append(
                    replace(
                        primary,
                        object_id=f"{primary.physical_group_id}-twin",
                        grid_u=primary.grid_u + 1,
                        grid_v=primary.grid_v - 1,
                    )
                )
    observed = sha256_bytes(canonical_json([row.record() for row in rows]))
    if observed != ROW_ROOTS[role]:
        raise F0Error(f"F0 {role} row root changed: {observed}")
    return tuple(rows)


def _normal_and_curvature(mesh: Mesh) -> tuple[np.ndarray, np.ndarray]:
    u = mesh.uv[:, 0]
    if mesh.row.topology == "Plate":
        normal = np.tile(np.asarray([0.0, 0.0, 1.0]), (u.size, 1))
        curvature = np.zeros((u.size, 2), dtype=np.float64)
    elif mesh.row.topology in ("Cylinder", "RolledSheet"):
        theta = np.pi * (u + 1.0)
        normal = np.column_stack((np.cos(theta), np.sin(theta), np.zeros_like(theta)))
        curvature = np.column_stack(
            (np.full(u.size, 2.0 * np.pi / mesh.row.length_m), np.zeros(u.size))
        )
    elif mesh.row.topology == "Bowl":
        theta = np.pi * (u + 1.0)
        alpha = np.pi * (mesh.uv[:, 1] + 1.0) / 4.0
        radial = 0.5 * mesh.row.length_m
        axial = 0.5 * mesh.row.length_m * mesh.row.aspect
        raw = np.column_stack(
            (
                np.sin(alpha) * np.cos(theta) / radial,
                np.sin(alpha) * np.sin(theta) / radial,
                -np.cos(alpha) / axial,
            )
        )
        normal = raw / np.linalg.norm(raw, axis=1, keepdims=True)
        denominator = np.sqrt(
            radial * radial * np.cos(alpha) ** 2 + axial * axial * np.sin(alpha) ** 2
        )
        curvature = np.column_stack(
            (
                radial * axial / denominator**3,
                axial / (radial * denominator),
            )
        )
    else:
        raise F0Error(f"unknown F0 topology: {mesh.row.topology}")
    if (
        normal.shape != (mesh.vertex_count, 3)
        or curvature.shape != (mesh.vertex_count, 2)
        or not np.isfinite(normal).all()
        or not np.isfinite(curvature).all()
    ):
        raise F0Error("F0 analytic surface feature changed")
    return normal, curvature


def gain_truth(mesh: Mesh, normals: np.ndarray) -> np.ndarray:
    u = mesh.uv[:, 0:1]
    v = mesh.uv[:, 1:2]
    mode = np.arange(MODE_COUNT, dtype=np.float64)[None, :]
    phase = 0.0 if mesh.row.support == "Free" else np.pi / 17.0
    if mesh.row.topology == "Plate":
        phi = np.cos(K * np.pi * u / 2.0 + phase) * np.cos(
            ELL * np.pi * v / 2.0
        ) + 0.12 * np.sin((K + ELL) * np.pi * u * v / 2.0)
    elif mesh.row.topology == "Cylinder":
        phi = np.cos(K * np.pi * u + mode * np.pi / 9.0 + phase) * np.cos(
            ELL * np.pi * v / 2.0
        )
    elif mesh.row.topology == "Bowl":
        phi = (
            np.cos(K * np.pi * u + mode * np.pi / 12.0 + phase)
            * np.cos(ELL * np.pi * v / 2.0)
            * (1.0 - 0.10 * (u * u + v * v))
        )
    elif mesh.row.topology == "RolledSheet":
        phi = np.sin(K * np.pi * (u + 1.0) / 2.0 + phase) * np.cos(
            ELL * np.pi * v / 2.0
        ) + 0.10 * np.cos((K + 1.0) * np.pi * u * v)
    else:
        raise F0Error(f"unknown F0 topology: {mesh.row.topology}")
    amplitude = (
        1.0 + 0.06 * np.log(MATERIAL_DENSITY[mesh.row.material] / 650.0)
    ) / np.sqrt(mode + 1.0)
    hidden = np.asarray(
        [
            2.0
            * np.pi
            * geometry_common.radical_inverse(
                mesh.row.halton_index + 13 * (index + 1), 7
            )
            for index in range(MODE_COUNT)
        ]
    )[None, :]
    mix = np.asarray(
        [
            0.25
            + 0.20
            * geometry_common.radical_inverse(
                mesh.row.halton_index + 17 * (index + 1), 11
            )
            for index in range(MODE_COUNT)
        ]
    )[None, :]
    residual = (
        mix
        * np.sin(RESIDUAL_K * np.pi * (u + 1.0) / 2.0 + hidden + phase)
        * np.cos(RESIDUAL_ELL * np.pi * v / 2.0 - 0.5 * hidden)
    )
    directional = 0.72 + 0.28 * np.abs(normals @ IMPULSE_DIRECTION)
    gains = amplitude * (phi + residual) * directional[:, None]
    if gains.shape != (mesh.vertex_count, MODE_COUNT) or not np.isfinite(gains).all():
        raise F0Error("F0 gain truth changed")
    return gains


def _context_count(row: FieldRow, vertex_count: int) -> int:
    if row.role == "train":
        return max(16, math.ceil(vertex_count / 3))
    result = coverage_common.required_context_count(vertex_count)
    if row.object_id.endswith("-twin"):
        result += 2
    return result


def generate_object(row: FieldRow) -> FieldObject:
    if row.role == "integration":
        raise F0Error("F0 cannot generate sealed integration values")
    mesh = geometry_common.build_mesh(row)
    analysis = coverage_oracle.analyze_mesh(mesh)
    allowed = np.arange(mesh.vertex_count, dtype=np.int64)
    context = coverage_oracle._restricted_fps(
        analysis.all_pairs,
        analysis.lexicographic_rank,
        allowed,
        _context_count(row, mesh.vertex_count),
    )
    selected = np.zeros(mesh.vertex_count, dtype=bool)
    selected[context] = True
    query = np.flatnonzero(~selected).astype(np.int64)
    coverage_input = coverage_oracle._coverage_input(
        analysis,
        "valid",
        "valid",
        context,
        query,
        analysis.graph,
    )
    decisions = tuple(coverage_oracle.evaluate(analysis, coverage_input, "composite"))
    accepted_query = np.asarray(
        [item["query_vertex"] for item in decisions if item["reason"] == "ACCEPT"],
        dtype=np.int64,
    )
    rejected_query = np.asarray(
        [item["query_vertex"] for item in decisions if item["reason"] != "ACCEPT"],
        dtype=np.int64,
    )
    if decisions and accepted_query.size + rejected_query.size != query.size:
        raise F0Error("F0 C0 decision coverage changed")
    normals, curvatures = _normal_and_curvature(mesh)
    gains = gain_truth(mesh, normals)
    result = FieldObject(
        row=row,
        mesh=mesh,
        analysis=analysis,
        context=context,
        query=query,
        accepted_query=accepted_query,
        rejected_query=rejected_query,
        coverage_input=coverage_input,
        coverage_decisions=decisions,
        normals=normals,
        curvatures=curvatures,
        gains=gains,
    )
    if row.role == "train" and rejected_query.size:
        raise F0Error(f"unsupported F0 train supervision: {row.object_id}")
    return result


def generate_objects(role: str) -> tuple[FieldObject, ...]:
    if role not in ("train", "development", "test"):
        raise F0Error(f"F0 cannot generate role: {role}")
    objects = tuple(generate_object(row) for row in generate_rows(role))
    if len({item.mesh.mesh_hash for item in objects}) != len(objects):
        raise F0Error("F0 mesh hash collision")
    if len({item.row.object_id for item in objects}) != len(objects):
        raise F0Error("F0 object identity collision")
    if role in ("train", "development"):
        observed = sha256_bytes(
            canonical_json([item.identity_record() for item in objects])
        )
        expected = TRAIN_RECORD_ROOT if role == "train" else DEVELOPMENT_RECORD_ROOT
        if observed != expected:
            raise F0Error(f"F0 {role} record root changed: {observed}")
    return objects


def geometry_npz(objects: tuple[FieldObject, ...]) -> bytes:
    vertices: list[np.ndarray] = []
    uv: list[np.ndarray] = []
    normals: list[np.ndarray] = []
    curvatures: list[np.ndarray] = []
    faces: list[np.ndarray] = []
    edges: list[np.ndarray] = []
    context: list[np.ndarray] = []
    query: list[np.ndarray] = []
    accepted: list[np.ndarray] = []
    vertex_offsets = [0]
    face_offsets = [0]
    edge_offsets = [0]
    context_offsets = [0]
    query_offsets = [0]
    accepted_offsets = [0]
    for item in objects:
        offset = vertex_offsets[-1]
        vertices.append(item.mesh.vertices)
        uv.append(item.mesh.uv)
        normals.append(item.normals)
        curvatures.append(item.curvatures)
        faces.append(item.mesh.faces + offset)
        edges.append(item.mesh.edges + offset)
        context.append(item.context + offset)
        query.append(item.query + offset)
        accepted.append(item.accepted_query + offset)
        vertex_offsets.append(offset + item.mesh.vertex_count)
        face_offsets.append(face_offsets[-1] + item.mesh.faces.shape[0])
        edge_offsets.append(edge_offsets[-1] + item.mesh.edges.shape[0])
        context_offsets.append(context_offsets[-1] + item.context.size)
        query_offsets.append(query_offsets[-1] + item.query.size)
        accepted_offsets.append(accepted_offsets[-1] + item.accepted_query.size)
    return deterministic_npz(
        {
            "accepted": np.concatenate(accepted),
            "accepted_offsets": np.asarray(accepted_offsets, dtype=np.int64),
            "context": np.concatenate(context),
            "context_offsets": np.asarray(context_offsets, dtype=np.int64),
            "curvatures": np.concatenate(curvatures),
            "edge_offsets": np.asarray(edge_offsets, dtype=np.int64),
            "edges": np.concatenate(edges),
            "face_offsets": np.asarray(face_offsets, dtype=np.int64),
            "faces": np.concatenate(faces),
            "normals": np.concatenate(normals),
            "query": np.concatenate(query),
            "query_offsets": np.asarray(query_offsets, dtype=np.int64),
            "uv": np.concatenate(uv),
            "vertex_offsets": np.asarray(vertex_offsets, dtype=np.int64),
            "vertices": np.concatenate(vertices),
        }
    )


def verify_protocol_environment() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        B0_RESULT_PATH: B0_RESULT_SHA256,
        C0_RESULT_PATH: C0_RESULT_SHA256,
        Path("lab/scripts") / C0_COMMON_FILE: C0_COMMON_SHA256,
        Path("lab/scripts") / C0_ORACLE_FILE: C0_ORACLE_SHA256,
        Path("lab/scripts") / GEOMETRY_FILE: GEOMETRY_SHA256,
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise F0Error(f"F0 dependency hash changed for {path}: {observed}")
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT:
        raise F0Error(f"F0 environment changed: {libraries}")
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
        raise F0Error(f"F0 thread environment changed: {threads}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise F0Error("F0 torch determinism changed")
    return {"libraries": libraries, "threads": threads}


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    result = {}
    for name in IMPLEMENTATION_FILES:
        path = directory / name
        if not path.is_file():
            raise F0Error(f"missing F0 implementation: {name}")
        result[name] = sha256_file(path)
    return result


def verify_committed_implementation() -> str:
    root = repository_root()
    paths = [str(Path("lab/scripts") / name) for name in IMPLEMENTATION_FILES]
    paths.append(str(TEST_FILE))
    status = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=all", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    if status.strip():
        raise F0Error("F0 implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise F0Error("F0 implementation commit is unavailable")
    return commit


def prepare_output(output: Path) -> tuple[Path, Path]:
    try:
        return coverage_common.prepare_output(output)
    except coverage_common.C0Error as error:
        raise F0Error(str(error).replace("C0", "F0")) from error


def publish_output(staging: Path, output: Path) -> None:
    try:
        coverage_common.publish_output(staging, output)
    except coverage_common.C0Error as error:
        raise F0Error(str(error).replace("C0", "F0")) from error


def abandon_output(staging: Path) -> None:
    coverage_common.abandon_output(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    try:
        coverage_common.write_files(directory, files)
    except coverage_common.C0Error as error:
        raise F0Error(str(error).replace("C0", "F0")) from error


directory_file_map = coverage_common.directory_file_map
tree_digest = coverage_common.tree_digest
compare_directories = coverage_common.compare_directories
