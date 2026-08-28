#!/usr/bin/env python3
"""Run the frozen geometry-only REALIMPACT spatial-transfer preflight."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import struct
import sys
import urllib.request
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import fast_simplification
import numpy as np
import scipy
import scipy.sparse
import scipy.sparse.linalg


MANIFEST_SHA256 = "5be5f195ddc5b124e7220959efc9bb69c0bf469f495b2906e7da9f7515aae576"
PREREGISTRATION_REPORT_SHA256 = (
    "c2f51cffef3cac1e7b62de5b187fbd8b16e8fa89f0e8c9d755c4752405dd01ef"
)
SOURCE_MANIFEST_SHA256 = (
    "92ebe6a7fbb3207dd7d2082076af3209304a0972430d3e59fd24ed8fa99642f8"
)
REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-geometry-spatial-transfer-preflight.report.v1"
)
STUDY_ID = "physical-sound-realimpact-geometry-spatial-transfer"
PROTOCOL_REVISION = "geometry-only-cotangent-biharmonic-preflight-v1"
EXPECTED_OBJECTS = ("65_PitcherCeramic", "63_SmallPlanterCeramic")
SPECTRAL_FACE_TARGET = 8192
BEM_FACE_TARGET = 2048
EIGENPAIR_COUNT = 65
NONCONSTANT_MODE_COUNT = 64
EIGEN_TOLERANCE = 1.0e-10
EIGEN_MAXITER = 20_000
MAX_RELATIVE_EIGEN_RESIDUAL = 1.0e-8
SIMPLIFIER_VERSION = "0.1.13"
SIMPLIFIER_REVISION = "4a193ef"
USER_AGENT = "NextEngine-physical-sound-geometry-preflight/1"


class PreflightError(RuntimeError):
    """A stable, reportable setup failure."""


@dataclass(frozen=True)
class MeshTopology:
    vertex_count: int
    face_count: int
    used_vertex_count: int
    connected_component_count: int
    unique_undirected_edge_count: int
    boundary_edge_count: int
    nonmanifold_edge_count: int
    inconsistent_oriented_edge_count: int
    duplicate_face_count: int
    minimum_triangle_area: float
    signed_volume_m3: float

    @property
    def passed(self) -> bool:
        return (
            self.vertex_count == self.used_vertex_count
            and self.connected_component_count == 1
            and self.boundary_edge_count == 0
            and self.nonmanifold_edge_count == 0
            and self.inconsistent_oriented_edge_count == 0
            and self.duplicate_face_count == 0
            and self.minimum_triangle_area > 0.0
            and self.signed_volume_m3 > 0.0
        )

    def as_json(self) -> dict[str, Any]:
        return {
            "vertex_count": self.vertex_count,
            "face_count": self.face_count,
            "used_vertex_count": self.used_vertex_count,
            "connected_component_count": self.connected_component_count,
            "unique_undirected_edge_count": self.unique_undirected_edge_count,
            "boundary_edge_count": self.boundary_edge_count,
            "nonmanifold_edge_count": self.nonmanifold_edge_count,
            "inconsistent_oriented_edge_count": self.inconsistent_oriented_edge_count,
            "duplicate_face_count": self.duplicate_face_count,
            "minimum_triangle_area": self.minimum_triangle_area,
            "signed_volume_m3": self.signed_volume_m3,
            "passed": self.passed,
        }


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def read_json(path: Path, expected_sha256: str, role: str) -> tuple[bytes, dict[str, Any]]:
    payload = path.read_bytes()
    actual = sha256_bytes(payload)
    if actual != expected_sha256:
        raise PreflightError(
            f"{role} hash changed: expected {expected_sha256}, got {actual}"
        )
    try:
        value = json.loads(payload)
    except json.JSONDecodeError as error:
        raise PreflightError(f"parse {role}: {error.msg}") from error
    if not isinstance(value, dict):
        raise PreflightError(f"{role} must be a JSON object")
    return payload, value


def resolve_source_manifest(
    manifest_path: Path, manifest: dict[str, Any]
) -> tuple[Path, dict[str, Any]]:
    references = manifest.get("prerequisites")
    if not isinstance(references, list):
        raise PreflightError("preregistration prerequisites are absent")
    reference = next(
        (
            item
            for item in references
            if isinstance(item, dict)
            and item.get("id") == "realimpact-frequency-calibration-manifest"
        ),
        None,
    )
    if reference is None or reference.get("sha256") != SOURCE_MANIFEST_SHA256:
        raise PreflightError("frozen REALIMPACT source manifest reference changed")
    source_path = (manifest_path.parent / str(reference.get("path"))).resolve()
    _, source = read_json(source_path, SOURCE_MANIFEST_SHA256, "source manifest")
    return source_path, source


def validate_inputs(
    manifest: dict[str, Any], preregistration_report: dict[str, Any]
) -> None:
    if (
        manifest.get("schema")
        != "nextengine.experimental-realimpact-geometry-spatial-transfer-preregistration.manifest.v1"
        or manifest.get("study_id") != STUDY_ID
        or manifest.get("revision") != "v1-preregistration"
        or manifest.get("phase") != "preregistration"
    ):
        raise PreflightError("preregistration contract changed")
    if (
        preregistration_report.get("schema")
        != "nextengine.experimental-realimpact-geometry-spatial-transfer-preregistration.report.v1"
        or preregistration_report.get("decision")
        != "RealSpatialTransferProtocolFrozen"
        or preregistration_report.get("reserved_audio_payload_bytes_read") != 0
        or preregistration_report.get("preregistration_network_requests") != 0
    ):
        raise PreflightError("preregistration report does not authorize geometry preflight")
    geometry = manifest.get("geometry_preflight", {})
    if (
        geometry.get("spectral_face_target") != SPECTRAL_FACE_TARGET
        or geometry.get("bem_face_target") != BEM_FACE_TARGET
        or f"fast-simplification {SIMPLIFIER_VERSION}" not in geometry.get("simplifier", "")
        or SIMPLIFIER_REVISION not in geometry.get("simplifier", "")
    ):
        raise PreflightError("geometry preflight parameters changed")
    if fast_simplification.__version__ != SIMPLIFIER_VERSION:
        raise PreflightError(
            "fast-simplification version changed: "
            f"expected {SIMPLIFIER_VERSION}, got {fast_simplification.__version__}"
        )


def fetch_deflated_entry(
    archive_url: str, entry: dict[str, Any], expected_raw_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    name = str(entry.get("name", ""))
    if not name.endswith("/preprocessed/transformed.obj"):
        raise PreflightError(f"prohibited geometry entry: {name}")
    if "deconvolved_0db" in name:
        raise PreflightError("audio payload entry reached geometry fetch boundary")
    start = int(entry.get("data_offset", -1))
    compressed_bytes = int(entry.get("compressed_bytes", -1))
    uncompressed_bytes = int(entry.get("uncompressed_bytes", -1))
    if start < 0 or compressed_bytes <= 0 or compressed_bytes > 2_000_000:
        raise PreflightError(f"invalid bounded mesh entry range: {name}")
    end = start + compressed_bytes - 1
    request = urllib.request.Request(
        archive_url,
        headers={
            "Range": f"bytes={start}-{end}",
            "Accept-Encoding": "identity",
            "User-Agent": USER_AGENT,
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            status = response.status
            content_range = response.headers.get("Content-Range", "")
            compressed = response.read(compressed_bytes + 1)
    except OSError as error:
        raise PreflightError(f"bounded mesh fetch failed for {name}: {error}") from error
    expected_range = f"bytes {start}-{end}/"
    if status != 206 or not content_range.startswith(expected_range):
        raise PreflightError(f"mesh endpoint ignored exact range for {name}")
    if len(compressed) != compressed_bytes:
        raise PreflightError(
            f"compressed mesh entry size changed for {name}: "
            f"expected {compressed_bytes}, got {len(compressed)}"
        )
    try:
        raw = zlib.decompress(compressed, -15)
    except zlib.error as error:
        raise PreflightError(f"raw-deflate mesh decode failed for {name}") from error
    if len(raw) != uncompressed_bytes:
        raise PreflightError(
            f"raw mesh entry size changed for {name}: "
            f"expected {uncompressed_bytes}, got {len(raw)}"
        )
    raw_sha256 = sha256_bytes(raw)
    if raw_sha256 != expected_raw_sha256 or entry.get("raw_sha256") != expected_raw_sha256:
        raise PreflightError(f"raw mesh entry hash changed for {name}")
    return raw, {
        "entry_name": name,
        "compressed_bytes_read": len(compressed),
        "uncompressed_bytes": len(raw),
        "raw_sha256": raw_sha256,
        "http_status": status,
        "content_range": content_range,
    }


def parse_obj(payload: bytes) -> tuple[np.ndarray, np.ndarray]:
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PreflightError("transformed OBJ is not UTF-8") from error
    vertices: list[tuple[float, float, float]] = []
    faces: list[tuple[int, int, int]] = []
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if parts[0] == "v":
            if len(parts) != 4:
                raise PreflightError(f"OBJ vertex arity changed at line {line_number}")
            try:
                vertex = tuple(float(value) for value in parts[1:])
            except ValueError as error:
                raise PreflightError(
                    f"OBJ vertex is not numeric at line {line_number}"
                ) from error
            if len(vertex) != 3 or not all(math.isfinite(value) for value in vertex):
                raise PreflightError(f"OBJ vertex is non-finite at line {line_number}")
            vertices.append(vertex)
        elif parts[0] == "f":
            if len(parts) != 4:
                raise PreflightError(f"OBJ face is not triangular at line {line_number}")
            face: list[int] = []
            for token in parts[1:]:
                raw_index = token.split("/", 1)[0]
                try:
                    index = int(raw_index)
                except ValueError as error:
                    raise PreflightError(
                        f"OBJ face index is invalid at line {line_number}"
                    ) from error
                resolved = index - 1 if index > 0 else len(vertices) + index
                if index == 0 or resolved < 0 or resolved >= len(vertices):
                    raise PreflightError(
                        f"OBJ face index is out of range at line {line_number}"
                    )
                face.append(resolved)
            if len(set(face)) != 3:
                raise PreflightError(f"OBJ face repeats a vertex at line {line_number}")
            faces.append(tuple(face))
    if len(vertices) < 4 or len(faces) < SPECTRAL_FACE_TARGET:
        raise PreflightError("OBJ mesh is too small for frozen face targets")
    points = np.ascontiguousarray(vertices, dtype=np.float64)
    triangles = np.ascontiguousarray(faces, dtype=np.int32)
    return points, triangles


def topology(points: np.ndarray, triangles: np.ndarray) -> MeshTopology:
    if points.ndim != 2 or points.shape[1] != 3:
        raise PreflightError("mesh points do not have shape Nx3")
    if triangles.ndim != 2 or triangles.shape[1] != 3:
        raise PreflightError("mesh faces do not have shape Mx3")
    if triangles.size == 0 or np.min(triangles) < 0 or np.max(triangles) >= len(points):
        raise PreflightError("mesh face indices are invalid")
    used = np.unique(triangles)
    parent = np.arange(len(points), dtype=np.int64)

    def find(index: int) -> int:
        root = index
        while parent[root] != root:
            root = int(parent[root])
        while parent[index] != index:
            next_index = int(parent[index])
            parent[index] = root
            index = next_index
        return root

    def union(left: int, right: int) -> None:
        left_root = find(left)
        right_root = find(right)
        if left_root != right_root:
            if left_root < right_root:
                parent[right_root] = left_root
            else:
                parent[left_root] = right_root

    edge_counts: dict[tuple[int, int], int] = {}
    edge_orientation: dict[tuple[int, int], int] = {}
    canonical_faces: set[tuple[int, int, int]] = set()
    duplicate_faces = 0
    for face in triangles:
        a, b, c = (int(face[0]), int(face[1]), int(face[2]))
        canonical = tuple(sorted((a, b, c)))
        if canonical in canonical_faces:
            duplicate_faces += 1
        canonical_faces.add(canonical)
        for start, finish in ((a, b), (b, c), (c, a)):
            union(start, finish)
            key = (min(start, finish), max(start, finish))
            edge_counts[key] = edge_counts.get(key, 0) + 1
            orientation = 1 if (start, finish) == key else -1
            edge_orientation[key] = edge_orientation.get(key, 0) + orientation
    components = len({find(int(index)) for index in used})
    boundary_edges = sum(count == 1 for count in edge_counts.values())
    nonmanifold_edges = sum(count != 2 for count in edge_counts.values())
    inconsistent_edges = sum(
        edge_counts[key] == 2 and edge_orientation[key] != 0 for key in edge_counts
    )
    v0 = points[triangles[:, 0]]
    v1 = points[triangles[:, 1]]
    v2 = points[triangles[:, 2]]
    cross = np.cross(v1 - v0, v2 - v0)
    areas = 0.5 * np.linalg.norm(cross, axis=1)
    signed_volume = float(np.sum(np.einsum("ij,ij->i", v0, np.cross(v1, v2))) / 6.0)
    return MeshTopology(
        vertex_count=len(points),
        face_count=len(triangles),
        used_vertex_count=len(used),
        connected_component_count=components,
        unique_undirected_edge_count=len(edge_counts),
        boundary_edge_count=boundary_edges,
        nonmanifold_edge_count=nonmanifold_edges,
        inconsistent_oriented_edge_count=inconsistent_edges,
        duplicate_face_count=duplicate_faces,
        minimum_triangle_area=float(np.min(areas)),
        signed_volume_m3=signed_volume,
    )


def simplify_mesh(
    points: np.ndarray, triangles: np.ndarray, target: int
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    reduced_points, reduced_faces, collapses = fast_simplification.simplify(
        points,
        triangles,
        target_count=target,
        agg=7.0,
        verbose=False,
        return_collapses=True,
        lossless=False,
    )
    reduced_points = np.ascontiguousarray(reduced_points, dtype=np.float64)
    reduced_faces = np.ascontiguousarray(reduced_faces, dtype=np.int32)
    collapses = np.ascontiguousarray(collapses, dtype=np.int64)
    if len(reduced_faces) != target:
        raise PreflightError(
            f"simplifier did not reach face target {target}: got {len(reduced_faces)}"
        )
    return reduced_points, reduced_faces, collapses


def cotangent_system(
    points: np.ndarray, triangles: np.ndarray
) -> tuple[scipy.sparse.csr_matrix, scipy.sparse.csr_matrix, np.ndarray]:
    vertex_count = len(points)
    rows: list[int] = []
    columns: list[int] = []
    data: list[float] = []
    mass = np.zeros(vertex_count, dtype=np.float64)
    for face in triangles:
        indices = [int(face[0]), int(face[1]), int(face[2])]
        vertices = points[indices]
        cross_norm = float(np.linalg.norm(np.cross(vertices[1] - vertices[0], vertices[2] - vertices[0])))
        if not math.isfinite(cross_norm) or cross_norm <= 0.0:
            raise PreflightError("spectral mesh contains a degenerate triangle")
        area = 0.5 * cross_norm
        for index in indices:
            mass[index] += area / 3.0
        cotangents = [
            float(
                np.dot(vertices[(corner + 1) % 3] - vertices[corner], vertices[(corner + 2) % 3] - vertices[corner])
                / cross_norm
            )
            for corner in range(3)
        ]
        for corner, cotangent in enumerate(cotangents):
            left = indices[(corner + 1) % 3]
            right = indices[(corner + 2) % 3]
            weight = 0.5 * cotangent
            rows.extend((left, right, left, right))
            columns.extend((left, right, right, left))
            data.extend((weight, weight, -weight, -weight))
    if np.any(~np.isfinite(mass)) or np.any(mass <= 0.0):
        raise PreflightError("spectral mass matrix is not positive")
    stiffness = scipy.sparse.coo_matrix(
        (data, (rows, columns)), shape=(vertex_count, vertex_count), dtype=np.float64
    ).tocsr()
    mass_matrix = scipy.sparse.diags(mass, format="csr")
    return stiffness, mass_matrix, mass


def canonical_eigenmodes(
    stiffness: scipy.sparse.csr_matrix,
    mass_matrix: scipy.sparse.csr_matrix,
    mass: np.ndarray,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    ramp = np.arange(stiffness.shape[0], dtype=np.float64)
    v0 = 1.0 + ramp / max(1.0, float(stiffness.shape[0] - 1))
    v0 /= np.linalg.norm(v0)
    try:
        eigenvalues, eigenvectors = scipy.sparse.linalg.eigsh(
            stiffness,
            k=EIGENPAIR_COUNT,
            M=mass_matrix,
            sigma=0.0,
            which="LM",
            v0=v0,
            tol=EIGEN_TOLERANCE,
            maxiter=EIGEN_MAXITER,
        )
    except (scipy.sparse.linalg.ArpackError, RuntimeError, ValueError) as error:
        raise PreflightError(f"frozen eigsh solve failed: {type(error).__name__}") from error
    order = np.argsort(eigenvalues, kind="stable")
    eigenvalues = np.asarray(eigenvalues[order], dtype=np.float64)
    eigenvectors = np.asarray(eigenvectors[:, order], dtype=np.float64)
    if len(eigenvalues) != EIGENPAIR_COUNT:
        raise PreflightError("eigensolver returned the wrong eigenpair count")
    modes = eigenvectors[:, 1 : NONCONSTANT_MODE_COUNT + 1].copy()
    values = eigenvalues[1 : NONCONSTANT_MODE_COUNT + 1].copy()
    if np.any(~np.isfinite(values)) or np.any(values <= 0.0):
        raise PreflightError("nonconstant surface eigenvalue is not positive")
    residuals = np.empty(NONCONSTANT_MODE_COUNT, dtype=np.float64)
    for mode_index in range(NONCONSTANT_MODE_COUNT):
        mode = modes[:, mode_index]
        norm = math.sqrt(float(np.dot(mass * mode, mode)))
        if not math.isfinite(norm) or norm <= 0.0:
            raise PreflightError("surface eigenmode mass norm is invalid")
        mode /= norm
        pivot = int(np.argmax(np.abs(mode)))
        if mode[pivot] < 0.0:
            mode *= -1.0
        left = stiffness @ mode
        right = values[mode_index] * (mass * mode)
        denominator = max(float(np.linalg.norm(left)), float(np.linalg.norm(right)), 1.0e-30)
        residuals[mode_index] = float(np.linalg.norm(left - right) / denominator)
    if np.max(residuals) > MAX_RELATIVE_EIGEN_RESIDUAL:
        raise PreflightError("surface eigenmode relative residual gate failed")
    hashes = [sha256_bytes(canonical_array_bytes(modes[:, index])) for index in range(modes.shape[1])]
    stable_order = sorted(range(len(values)), key=lambda index: (values[index], hashes[index]))
    return values[stable_order], modes[:, stable_order], residuals[stable_order]


def transfer_modes(
    spectral_points: np.ndarray,
    spectral_faces: np.ndarray,
    spectral_mass: np.ndarray,
    spectral_modes: np.ndarray,
    bem_points: np.ndarray,
    bem_faces: np.ndarray,
    collapses: np.ndarray,
) -> tuple[np.ndarray, np.ndarray]:
    replay_points, replay_faces, mapping = fast_simplification.replay_simplification(
        spectral_points, spectral_faces, collapses
    )
    replay_points = np.ascontiguousarray(replay_points, dtype=np.float64)
    replay_faces = np.ascontiguousarray(replay_faces, dtype=np.int32)
    mapping = np.ascontiguousarray(mapping, dtype=np.int64)
    if not np.array_equal(replay_points, bem_points) or not np.array_equal(replay_faces, bem_faces):
        raise PreflightError("simplification replay does not reproduce the BEM mesh")
    if len(mapping) != len(spectral_points) or np.min(mapping) < 0 or np.max(mapping) >= len(bem_points):
        raise PreflightError("simplification replay mapping is invalid")
    _, _, bem_mass = cotangent_system(bem_points, bem_faces)
    denominator = np.bincount(mapping, weights=spectral_mass, minlength=len(bem_points))
    if np.any(denominator <= 0.0):
        raise PreflightError("field replay leaves an unmapped BEM vertex")
    transferred = np.empty((len(bem_points), spectral_modes.shape[1]), dtype=np.float64)
    for index in range(spectral_modes.shape[1]):
        numerator = np.bincount(
            mapping,
            weights=spectral_mass * spectral_modes[:, index],
            minlength=len(bem_points),
        )
        mode = numerator / denominator
        norm = math.sqrt(float(np.dot(bem_mass * mode, mode)))
        if not math.isfinite(norm) or norm <= 0.0:
            raise PreflightError("transferred BEM mode mass norm is invalid")
        mode /= norm
        pivot = int(np.argmax(np.abs(mode)))
        if mode[pivot] < 0.0:
            mode *= -1.0
        transferred[:, index] = mode
    return transferred, mapping


def canonical_array_bytes(array: np.ndarray) -> bytes:
    value = np.ascontiguousarray(array)
    if value.dtype.kind == "f":
        value = value.astype("<f8", copy=False)
    elif value.dtype.kind in ("i", "u"):
        value = value.astype("<i8", copy=False)
    else:
        raise PreflightError(f"unsupported canonical array dtype: {value.dtype}")
    return value.tobytes(order="C")


def encode_geometry_block(arrays: list[tuple[str, np.ndarray]]) -> bytes:
    output = bytearray(b"NEPSGEO1")
    output.extend(struct.pack("<I", len(arrays)))
    for name, array in arrays:
        encoded_name = name.encode("ascii")
        canonical = np.ascontiguousarray(array)
        if canonical.dtype.kind == "f":
            canonical = canonical.astype("<f8", copy=False)
            dtype = b"f64le"
        elif canonical.dtype.kind in ("i", "u"):
            canonical = canonical.astype("<i8", copy=False)
            dtype = b"i64le"
        else:
            raise PreflightError(f"unsupported block dtype for {name}")
        payload = canonical.tobytes(order="C")
        output.extend(struct.pack("<H", len(encoded_name)))
        output.extend(encoded_name)
        output.extend(struct.pack("<B", len(dtype)))
        output.extend(dtype)
        output.extend(struct.pack("<B", canonical.ndim))
        for dimension in canonical.shape:
            output.extend(struct.pack("<Q", dimension))
        output.extend(struct.pack("<Q", len(payload)))
        output.extend(payload)
    return bytes(output)


def array_summary(array: np.ndarray) -> dict[str, Any]:
    return {
        "shape": list(array.shape),
        "dtype": str(array.dtype),
        "sha256": sha256_bytes(canonical_array_bytes(array)),
    }


def source_profile(source: dict[str, Any], object_id: str) -> dict[str, Any]:
    profiles = source.get("archive_profiles")
    if not isinstance(profiles, list):
        raise PreflightError("source archive profiles are absent")
    profile = next(
        (
            item
            for item in profiles
            if isinstance(item, dict) and item.get("dataset_object_id") == object_id
        ),
        None,
    )
    if profile is None or profile.get("role") != "holdout":
        raise PreflightError(f"reserved source profile changed for {object_id}")
    return profile


def manifest_object(manifest: dict[str, Any], object_id: str) -> dict[str, Any]:
    objects = manifest.get("objects")
    if not isinstance(objects, list):
        raise PreflightError("preregistration objects are absent")
    value = next(
        (
            item
            for item in objects
            if isinstance(item, dict) and item.get("object_id") == object_id
        ),
        None,
    )
    if value is None:
        raise PreflightError(f"preregistered object missing: {object_id}")
    return value


def process_object(
    manifest: dict[str, Any], source: dict[str, Any], object_id: str
) -> tuple[dict[str, Any], bytes | None]:
    frozen = manifest_object(manifest, object_id)
    profile = source_profile(source, object_id)
    entries = profile.get("metadata_entries")
    if not isinstance(entries, list):
        raise PreflightError(f"source metadata entries missing for {object_id}")
    entry = next(
        (
            item
            for item in entries
            if isinstance(item, dict) and item.get("name") == frozen.get("mesh_entry_name")
        ),
        None,
    )
    if entry is None or entry.get("raw_sha256") != frozen.get("mesh_raw_sha256"):
        raise PreflightError(f"frozen mesh entry identity changed for {object_id}")
    raw, fetch = fetch_deflated_entry(
        str(frozen.get("archive_url")), entry, str(frozen.get("mesh_raw_sha256"))
    )
    points, faces = parse_obj(raw)
    original = topology(points, faces)
    spectral_points, spectral_faces, spectral_collapses = simplify_mesh(
        points, faces, SPECTRAL_FACE_TARGET
    )
    spectral = topology(spectral_points, spectral_faces)
    bem_points, bem_faces, bem_collapses = simplify_mesh(
        spectral_points, spectral_faces, BEM_FACE_TARGET
    )
    bem = topology(bem_points, bem_faces)
    topology_passed = spectral.passed and bem.passed
    result: dict[str, Any] = {
        "object_id": object_id,
        "role": frozen.get("role"),
        "selection_sha256": frozen.get("selection_sha256"),
        "audio_entry_name_bound_but_not_read": frozen.get("audio_entry_name"),
        "fetch": fetch,
        "original": original.as_json(),
        "spectral": spectral.as_json(),
        "bem": bem.as_json(),
        "topology_gate_passed": topology_passed,
        "block": None,
    }
    if not topology_passed:
        result["decision"] = "GeometryUnsupportedFallback"
        result["reason"] = "derived mesh topology gate failed"
        return result, None
    stiffness, mass_matrix, spectral_mass = cotangent_system(
        spectral_points, spectral_faces
    )
    eigenvalues, modes, residuals = canonical_eigenmodes(
        stiffness, mass_matrix, spectral_mass
    )
    transferred, mapping = transfer_modes(
        spectral_points,
        spectral_faces,
        spectral_mass,
        modes,
        bem_points,
        bem_faces,
        bem_collapses,
    )
    block = encode_geometry_block(
        [
            ("spectral_points", spectral_points),
            ("spectral_faces", spectral_faces),
            ("spectral_collapses", spectral_collapses),
            ("bem_points", bem_points),
            ("bem_faces", bem_faces),
            ("bem_collapses", bem_collapses),
            ("spectral_to_bem_mapping", mapping),
            ("eigenvalues", eigenvalues),
            ("eigenmodes", modes),
            ("relative_eigen_residuals", residuals),
            ("bem_modes", transferred),
        ]
    )
    result.update(
        {
            "decision": "GeometryPreflightSupported",
            "maximum_relative_eigen_residual": float(np.max(residuals)),
            "eigenvalue_minimum_per_square_metre": float(np.min(eigenvalues)),
            "eigenvalue_maximum_per_square_metre": float(np.max(eigenvalues)),
            "arrays": {
                "spectral_points": array_summary(spectral_points),
                "spectral_faces": array_summary(spectral_faces),
                "spectral_collapses": array_summary(spectral_collapses),
                "bem_points": array_summary(bem_points),
                "bem_faces": array_summary(bem_faces),
                "bem_collapses": array_summary(bem_collapses),
                "spectral_to_bem_mapping": array_summary(mapping),
                "eigenvalues": array_summary(eigenvalues),
                "eigenmodes": array_summary(modes),
                "relative_eigen_residuals": array_summary(residuals),
                "bem_modes": array_summary(transferred),
            },
            "block": {
                "path": f"{object_id}-geometry-block.bin",
                "bytes": len(block),
                "sha256": sha256_bytes(block),
            },
        }
    )
    return result, block


def pretty_json(value: dict[str, Any]) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode(
        "utf-8"
    )


def publish(
    output: Path,
    manifest_bytes: bytes,
    preregistration_report_bytes: bytes,
    report_bytes: bytes,
    blocks: dict[str, bytes],
) -> None:
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise PreflightError(f"output must be absent or empty: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "preregistration-manifest.json").write_bytes(manifest_bytes)
        (staging / "preregistration-report.json").write_bytes(
            preregistration_report_bytes
        )
        for name, payload in blocks.items():
            (staging / name).write_bytes(payload)
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--preregistration-report", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()
    manifest_path = arguments.manifest.resolve()
    report_path = arguments.preregistration_report.resolve()
    manifest_bytes, manifest = read_json(
        manifest_path, MANIFEST_SHA256, "preregistration manifest"
    )
    preregistration_report_bytes, preregistration_report = read_json(
        report_path, PREREGISTRATION_REPORT_SHA256, "preregistration report"
    )
    validate_inputs(manifest, preregistration_report)
    source_path, source = resolve_source_manifest(manifest_path, manifest)
    results: list[dict[str, Any]] = []
    blocks: dict[str, bytes] = {}
    for object_id in EXPECTED_OBJECTS:
        try:
            result, block = process_object(manifest, source, object_id)
        except PreflightError as error:
            result = {
                "object_id": object_id,
                "role": manifest_object(manifest, object_id).get("role"),
                "decision": "GeometryUnsupportedFallback",
                "reason": str(error),
                "block": None,
            }
            block = None
        results.append(result)
        if block is not None:
            blocks[f"{object_id}-geometry-block.bin"] = block
    supported = all(
        result.get("decision") == "GeometryPreflightSupported" for result in results
    )
    script_sha256 = sha256_bytes(Path(__file__).read_bytes())
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": (
            "GeometryOnlyPreflightSupported"
            if supported
            else "GeometryOnlyPreflightRejected"
        ),
        "claim": "GEOMETRY_ONLY_PREFLIGHT / RESERVED_AUDIO_PAYLOAD_BYTES_READ_ZERO / NO_REAL_SPATIAL_TRANSFER_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        "study_id": STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "manifest_sha256": MANIFEST_SHA256,
        "preregistration_report_sha256": PREREGISTRATION_REPORT_SHA256,
        "source_manifest": {
            "path": str(source_path),
            "sha256": SOURCE_MANIFEST_SHA256,
        },
        "implementation": {
            "script_sha256": script_sha256,
            "python": sys.version.split()[0],
            "numpy": np.__version__,
            "scipy": scipy.__version__,
            "fast_simplification": fast_simplification.__version__,
            "fast_simplification_revision": SIMPLIFIER_REVISION,
        },
        "network_requests": len(EXPECTED_OBJECTS),
        "network_request_scope": "two transformed.obj raw-deflate entry ranges only",
        "mesh_compressed_bytes_read": sum(
            int(result.get("fetch", {}).get("compressed_bytes_read", 0))
            for result in results
        ),
        "reserved_audio_payload_bytes_read": 0,
        "objects": results,
        "gate": {
            "every_object_geometry_preflight_supported": supported,
            "every_derived_mesh_topology_gate_passed": all(
                bool(result.get("topology_gate_passed")) for result in results
            ),
            "every_supported_object_has_64_modes": all(
                result.get("decision") != "GeometryPreflightSupported"
                or result.get("arrays", {}).get("eigenvalues", {}).get("shape")
                == [NONCONSTANT_MODE_COUNT]
                for result in results
            ),
            "reserved_audio_payload_bytes_zero": True,
        },
        "allowed_claims": [
            "deterministic_geometry_setup_for_the_frozen_real_transfer_protocol_if_supported"
        ],
        "prohibited_claims": manifest.get("prohibited_claims"),
        "next_action": (
            "freeze the byte-identical geometry blocks into a calibration manifest before opening the Pitcher audio prefix"
            if supported
            else "retain clip fallback and research a new geometry setup without opening Pitcher or Planter audio"
        ),
    }
    report_bytes = pretty_json(report)
    publish(
        arguments.output.resolve(),
        manifest_bytes,
        preregistration_report_bytes,
        report_bytes,
        blocks,
    )
    print(f"REALIMPACT geometry preflight: {arguments.output.resolve()}")
    print(f"decision: {report['decision']}")
    print("reserved audio payload bytes read: 0")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    for result in results:
        print(f"{result['object_id']}: {result['decision']} ({result.get('reason', 'ok')})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
