"""Deterministic point-on-triangle and per-vertex field evaluation for V26 M0b."""

from __future__ import annotations

import math
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from typing import Any

import numpy as np
import physical_sound_v25_m0a_common as base
import physical_sound_v26_m0b_common as contract

SURFACE_TOLERANCE_FLOOR_METRES = 1.0e-12
RELATIVE_TOLERANCE = 2.0**-40
DEGENERATE_FACTOR = 2.0**-80
AFFINE_PAIR_LIMIT = 2.0**-48
COARSE_GRID = (17, 13)
REFINED_GRID = (33, 25)
PLATE_DIMENSIONS = (0.137, 0.083, 0.0027)
BEAM_DIMENSIONS = (0.311, 0.041, 0.0043)
CONTACTS = (
    (1.0 / 8.0, 1.0 / 8.0),
    (7.0 / 8.0, 1.0 / 8.0),
    (1.0 / 8.0, 7.0 / 8.0),
    (7.0 / 8.0, 7.0 / 8.0),
    (1.0 / 2.0, 1.0 / 4.0),
    (1.0 / 4.0, 1.0 / 2.0),
    (3.0 / 4.0, 1.0 / 2.0),
    (1.0 / 2.0, 3.0 / 4.0),
    (3.0 / 8.0, 3.0 / 8.0),
    (5.0 / 8.0, 3.0 / 8.0),
    (3.0 / 8.0, 5.0 / 8.0),
    (5.0 / 8.0, 5.0 / 8.0),
)


@dataclass(frozen=True)
class FaceTerms:
    canonical_key: tuple[int, int, int]
    indices: tuple[int, int, int]
    a: np.ndarray
    e0: np.ndarray
    e1: np.ndarray
    normal: np.ndarray
    squared_normal: float
    d00: float
    d01: float
    d11: float
    denominator: float


@dataclass(frozen=True)
class SurfaceSample:
    face_vertices: tuple[int, int, int]
    weights: np.ndarray
    projected_point_metres: np.ndarray
    plane_residual_metres: float
    reconstruction_residual_metres: float
    surface_tolerance_metres: float

    def canonical_record(self) -> dict[str, Any]:
        return {
            "face_vertices": list(self.face_vertices),
            "weights": self.weights.tolist(),
            "projected_point_metres": self.projected_point_metres.tolist(),
            "plane_residual_metres": self.plane_residual_metres,
            "reconstruction_residual_metres": self.reconstruction_residual_metres,
            "surface_tolerance_metres": self.surface_tolerance_metres,
        }


class SurfaceEvaluator:
    def __init__(self, mesh: base.Mesh) -> None:
        self.mesh = mesh
        lengths = np.ptp(mesh.vertices, axis=0)
        self.diagonal = float(np.linalg.norm(lengths))
        if not math.isfinite(self.diagonal) or self.diagonal <= 0.0:
            raise base.M0Error("M0b surface mesh diagonal is invalid")
        self.surface_tolerance = max(
            SURFACE_TOLERANCE_FLOOR_METRES, self.diagonal * RELATIVE_TOLERANCE
        )
        degeneracy_limit = self.diagonal**4 * DEGENERATE_FACTOR
        faces: list[FaceTerms] = []
        keys: set[tuple[int, int, int]] = set()
        for raw_indices in mesh.triangles:
            indices = tuple(int(value) for value in raw_indices)
            if len(set(indices)) != 3 or any(
                index < 0 or index >= len(mesh.vertices) for index in indices
            ):
                raise base.M0Error(
                    "M0b surface face indices are repeated or out of range"
                )
            key = tuple(sorted(indices))
            if key in keys:
                raise base.M0Error(
                    "M0b surface mesh contains a duplicate canonical face"
                )
            keys.add(key)
            indices = key
            a, b, c = mesh.vertices[np.asarray(indices, dtype=np.int64)]
            e0 = b - a
            e1 = c - a
            normal = np.cross(e0, e1)
            squared_normal = float(normal @ normal)
            d00 = float(e0 @ e0)
            d01 = float(e0 @ e1)
            d11 = float(e1 @ e1)
            denominator = d00 * d11 - d01 * d01
            values = np.asarray(
                [squared_normal, d00, d01, d11, denominator, *a, *e0, *e1, *normal],
                dtype=np.float64,
            )
            if (
                not np.all(np.isfinite(values))
                or squared_normal <= degeneracy_limit
                or denominator <= degeneracy_limit
            ):
                raise base.M0Error(
                    "M0b surface mesh contains a degenerate or non-finite face"
                )
            faces.append(
                FaceTerms(
                    key,
                    indices,
                    a,
                    e0,
                    e1,
                    normal,
                    squared_normal,
                    d00,
                    d01,
                    d11,
                    denominator,
                )
            )
        if not faces:
            raise base.M0Error("M0b surface mesh has no faces")
        self.faces = tuple(faces)

    def locate(self, point_metres: Iterable[float]) -> SurfaceSample:
        query = np.asarray(tuple(point_metres), dtype=np.float64)
        if query.shape != (3,) or not np.all(np.isfinite(query)):
            raise base.M0Error("M0b surface query is invalid")
        candidates: list[
            tuple[
                tuple[int, int, int],
                tuple[int, int, int],
                np.ndarray,
                np.ndarray,
                float,
            ]
        ] = []
        for face in self.faces:
            delta = query - face.a
            signed_offset = float(delta @ face.normal)
            plane_residual = abs(signed_offset) / math.sqrt(face.squared_normal)
            if (
                not math.isfinite(plane_residual)
                or plane_residual > self.surface_tolerance
            ):
                continue
            projected = query - face.normal * (signed_offset / face.squared_normal)
            projected_delta = projected - face.a
            d20 = float(projected_delta @ face.e0)
            d21 = float(projected_delta @ face.e1)
            second = (face.d11 * d20 - face.d01 * d21) / face.denominator
            third = (face.d00 * d21 - face.d01 * d20) / face.denominator
            first = 1.0 - second - third
            weights = np.asarray((first, second, third), dtype=np.float64)
            if (
                not np.all(np.isfinite(weights))
                or float(np.min(weights)) < -RELATIVE_TOLERANCE
                or float(np.max(weights)) > 1.0 + RELATIVE_TOLERANCE
                or abs(float(np.sum(weights)) - 1.0) > RELATIVE_TOLERANCE
            ):
                continue
            pairs = sorted(zip(face.indices, weights, strict=True))
            sorted_indices = tuple(index for index, _ in pairs)
            sorted_weights = np.asarray(
                [weight for _, weight in pairs], dtype=np.float64
            )
            reconstructed = (
                sorted_weights
                @ self.mesh.vertices[np.asarray(sorted_indices, dtype=np.int64)]
            )
            reconstruction_residual = float(np.linalg.norm(reconstructed - projected))
            if (
                not math.isfinite(reconstruction_residual)
                or reconstruction_residual > self.surface_tolerance
            ):
                continue
            candidates.append(
                (
                    face.canonical_key,
                    sorted_indices,
                    sorted_weights,
                    projected,
                    plane_residual,
                )
            )
        if not candidates:
            raise base.M0Error("M0b surface query is outside the declared mesh")
        _, indices, weights, projected, plane_residual = min(
            candidates, key=lambda item: item[0]
        )
        reconstructed = (
            weights @ self.mesh.vertices[np.asarray(indices, dtype=np.int64)]
        )
        reconstruction_residual = float(np.linalg.norm(reconstructed - projected))
        return SurfaceSample(
            indices,
            weights,
            projected,
            plane_residual,
            reconstruction_residual,
            self.surface_tolerance,
        )

    def interpolate(
        self,
        point_metres: Iterable[float],
        field: np.ndarray,
        *,
        mesh_sha256: str,
        channel_count: int,
    ) -> tuple[SurfaceSample, np.ndarray]:
        values = np.asarray(field)
        if (
            mesh_sha256 != self.mesh.sha256
            or values.dtype != np.dtype("float64")
            or values.shape != (len(self.mesh.vertices), channel_count)
            or not np.all(np.isfinite(values))
        ):
            raise base.M0Error(
                "M0b surface field shape, dtype, finiteness or mesh binding changed"
            )
        sample = self.locate(point_metres)
        output = (
            sample.weights @ values[np.asarray(sample.face_vertices, dtype=np.int64)]
        )
        if output.shape != (channel_count,) or not np.all(np.isfinite(output)):
            raise base.M0Error("M0b interpolated surface field is invalid")
        return sample, np.asarray(output, dtype=np.float64)


def fixture_mesh(family: str, grid: tuple[int, int]) -> base.Mesh:
    if family not in {"plate", "beam"} or grid not in {COARSE_GRID, REFINED_GRID}:
        raise base.M0Error("M0b surface fixture family or grid changed")
    first, second, thickness = (
        PLATE_DIMENSIONS if family == "plate" else BEAM_DIMENSIONS
    )
    grid_u, grid_v = grid
    vertices = []
    for v_index in range(grid_v):
        v = v_index / (grid_v - 1)
        for u_index in range(grid_u):
            u = u_index / (grid_u - 1)
            if family == "plate":
                vertices.append((first * u, second * v, thickness / 2.0))
            else:
                vertices.append((first * u, second * (v - 0.5), thickness / 2.0))
    triangles = []
    for v_index in range(grid_v - 1):
        for u_index in range(grid_u - 1):
            lower = v_index * grid_u + u_index
            triangles.append((lower, lower + 1, lower + grid_u))
            triangles.append((lower + 1, lower + grid_u + 1, lower + grid_u))
    vertex_array = np.asarray(vertices, dtype=np.float64)
    triangle_array = np.asarray(triangles, dtype=np.uint32)
    identity = base.sha256_bytes(
        vertex_array.astype("<f8", copy=False).tobytes()
        + triangle_array.astype("<u4", copy=False).tobytes()
    )
    return base.Mesh(vertex_array, triangle_array, identity)


def fixture_point(family: str, u: float, v: float) -> np.ndarray:
    first, second, thickness = (
        PLATE_DIMENSIONS if family == "plate" else BEAM_DIMENSIONS
    )
    if family == "plate":
        return np.asarray((first * u, second * v, thickness / 2.0), dtype=np.float64)
    return np.asarray(
        (first * u, second * (v - 0.5), thickness / 2.0), dtype=np.float64
    )


def affine_field(vertices: np.ndarray) -> np.ndarray:
    x = vertices[:, 0]
    y = vertices[:, 1]
    z = vertices[:, 2]
    return np.column_stack(
        (
            np.ones(len(vertices)),
            x,
            y,
            z,
            x + y,
            x - y,
            x + z,
            y + z,
            2.0 * x - 3.0 * y + z,
            -x + 4.0 * y - 2.0 * z,
        )
    ).astype(np.float64, copy=False)


def _expect_reject(operation: Callable[[], object], role: str) -> None:
    try:
        operation()
    except base.M0Error:
        return
    raise base.M0Error(f"M0b surface mutation did not reject: {role}")


def conformance_fixture_counts() -> tuple[int, int]:
    mesh = fixture_mesh("plate", COARSE_GRID)
    point = fixture_point("plate", 3.0 / 8.0, 3.0 / 8.0)
    field = affine_field(mesh.vertices)
    reference_sample, reference_value = SurfaceEvaluator(mesh).interpolate(
        point,
        field,
        mesh_sha256=mesh.sha256,
        channel_count=10,
    )
    reference_record = reference_sample.canonical_record()
    variants = (
        mesh.triangles[::-1].copy(),
        np.roll(mesh.triangles, 17, axis=0),
        mesh.triangles[:, [1, 2, 0]].copy(),
        mesh.triangles[:, [0, 2, 1]].copy(),
    )
    invariance_count = 0
    for triangles in variants:
        identity = base.sha256_bytes(
            mesh.vertices.astype("<f8", copy=False).tobytes()
            + triangles.astype("<u4", copy=False).tobytes()
        )
        variant = base.Mesh(mesh.vertices.copy(), triangles, identity)
        sample, value = SurfaceEvaluator(variant).interpolate(
            point,
            field,
            mesh_sha256=identity,
            channel_count=10,
        )
        if sample.canonical_record() != reference_record or not np.array_equal(
            value, reference_value
        ):
            raise base.M0Error(
                "M0b surface canonical result changed under face representation"
            )
        invariance_count += 1

    mutations: list[tuple[str, Callable[[], object]]] = []
    duplicate = np.concatenate((mesh.triangles, mesh.triangles[:1, ::-1]), axis=0)
    mutations.append(
        (
            "duplicate-face",
            lambda: SurfaceEvaluator(base.Mesh(mesh.vertices, duplicate, "0" * 64)),
        )
    )
    repeated = mesh.triangles.copy()
    repeated[0] = (0, 0, 1)
    mutations.append(
        (
            "repeated-index",
            lambda: SurfaceEvaluator(base.Mesh(mesh.vertices, repeated, "0" * 64)),
        )
    )
    out_of_range = mesh.triangles.copy()
    out_of_range[0, 2] = len(mesh.vertices)
    mutations.append(
        (
            "out-of-range-index",
            lambda: SurfaceEvaluator(base.Mesh(mesh.vertices, out_of_range, "0" * 64)),
        )
    )
    degenerate_vertices = mesh.vertices.copy()
    first = mesh.triangles[0]
    degenerate_vertices[first[2]] = (
        degenerate_vertices[first[0]] + degenerate_vertices[first[1]]
    ) * 0.5
    mutations.append(
        (
            "degenerate-face",
            lambda: SurfaceEvaluator(
                base.Mesh(degenerate_vertices, mesh.triangles, "0" * 64)
            ),
        )
    )
    evaluator = SurfaceEvaluator(mesh)
    outside_point = fixture_point("plate", 0.0, 0.5) + np.asarray(
        (-evaluator.diagonal * 2.0**-12, 0.0, 0.0)
    )
    mutations.extend(
        (
            ("query-nan", lambda: evaluator.locate((math.nan, 0.0, 0.0))),
            ("query-inf", lambda: evaluator.locate((math.inf, 0.0, 0.0))),
            ("query-outside", lambda: evaluator.locate(outside_point)),
            (
                "query-off-plane",
                lambda: evaluator.locate(
                    point + np.asarray((0.0, 0.0, evaluator.surface_tolerance * 2.0))
                ),
            ),
        )
    )
    nan_field = field.copy()
    nan_field[0, 0] = math.nan
    inf_field = field.copy()
    inf_field[0, 0] = math.inf
    mutations.extend(
        (
            (
                "field-nan",
                lambda: evaluator.interpolate(
                    point, nan_field, mesh_sha256=mesh.sha256, channel_count=10
                ),
            ),
            (
                "field-inf",
                lambda: evaluator.interpolate(
                    point, inf_field, mesh_sha256=mesh.sha256, channel_count=10
                ),
            ),
            (
                "field-shape",
                lambda: evaluator.interpolate(
                    point, field[:-1], mesh_sha256=mesh.sha256, channel_count=10
                ),
            ),
            (
                "field-channels",
                lambda: evaluator.interpolate(
                    point, field, mesh_sha256=mesh.sha256, channel_count=9
                ),
            ),
            (
                "field-dtype",
                lambda: evaluator.interpolate(
                    point,
                    field.astype(np.float32),
                    mesh_sha256=mesh.sha256,
                    channel_count=10,
                ),
            ),
            (
                "field-binding",
                lambda: evaluator.interpolate(
                    point, field, mesh_sha256="0" * 64, channel_count=10
                ),
            ),
        )
    )
    for role, operation in mutations:
        _expect_reject(operation, role)

    boundary_vertices = np.asarray(
        ((0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)),
        dtype=np.float64,
    )
    boundary_mesh = base.Mesh(
        boundary_vertices, np.asarray(((0, 1, 2),), dtype=np.uint32), "1" * 64
    )
    boundary = SurfaceEvaluator(boundary_mesh)
    inside_offset = np.nextafter(boundary.surface_tolerance, 0.0)
    outside_offset = np.nextafter(boundary.surface_tolerance, math.inf)
    boundary.locate((0.25, 0.25, inside_offset))
    _expect_reject(
        lambda: boundary.locate((0.25, 0.25, outside_offset)),
        "surface-tolerance-nextafter",
    )
    return invariance_count, len(mutations) + 1


def official_shape_report(implementation_root_sha256: str) -> dict[str, Any]:
    records = []
    maximum_plane = 0.0
    maximum_reconstruction = 0.0
    maximum_pair = 0.0
    pair_count = 0
    for family in ("beam", "plate"):
        evaluators = {}
        fields = {}
        for label, grid in (("coarse", COARSE_GRID), ("refined", REFINED_GRID)):
            mesh = fixture_mesh(family, grid)
            evaluators[label] = SurfaceEvaluator(mesh)
            fields[label] = affine_field(mesh.vertices)
        for contact_index, (u, v) in enumerate(CONTACTS):
            point = fixture_point(family, u, v)
            pair = {}
            for label in ("coarse", "refined"):
                evaluator = evaluators[label]
                sample, value = evaluator.interpolate(
                    point,
                    fields[label],
                    mesh_sha256=evaluator.mesh.sha256,
                    channel_count=10,
                )
                record = {
                    "family": family,
                    "grid": label,
                    "contact_index": contact_index,
                    "u": u,
                    "v": v,
                    "sample": sample.canonical_record(),
                    "field": value.tolist(),
                }
                records.append(record)
                pair[label] = value
                maximum_plane = max(maximum_plane, sample.plane_residual_metres)
                maximum_reconstruction = max(
                    maximum_reconstruction, sample.reconstruction_residual_metres
                )
            maximum_pair = max(
                maximum_pair, float(np.max(np.abs(pair["coarse"] - pair["refined"])))
            )
            pair_count += 1
    if len(records) != 48 or pair_count != 24 or maximum_pair > AFFINE_PAIR_LIMIT:
        raise base.M0Error(
            "M0b official-shape fixture failed its frozen counts or affine gate"
        )
    invariance_count, mutation_count = conformance_fixture_counts()
    query_root = base.sha256_bytes(base.canonical_json(records))
    return {
        "schema": contract.SURFACE_REPORT_SCHEMA,
        "status": "Validated",
        "fixture_id": "surface-query-official-shape-v1",
        "protocol_sha256": contract.PROTOCOL_SHA256,
        "implementation_root_sha256": implementation_root_sha256,
        "coarse_grid": list(COARSE_GRID),
        "refined_grid": list(REFINED_GRID),
        "plate_dimensions_metres": list(PLATE_DIMENSIONS),
        "beam_dimensions_metres": list(BEAM_DIMENSIONS),
        "contact_count": len(CONTACTS),
        "query_evaluation_count": len(records),
        "coarse_refined_pair_count": pair_count,
        "maximum_plane_residual_metres": maximum_plane,
        "maximum_reconstruction_residual_metres": maximum_reconstruction,
        "maximum_affine_pair_difference": maximum_pair,
        "affine_pair_limit": AFFINE_PAIR_LIMIT,
        "canonical_query_root_sha256": query_root,
        "invariance_fixture_count": invariance_count,
        "mutation_fixture_count": mutation_count,
        "teacher_values_opened": False,
        "model_values_opened": False,
        "protected_roles_opened": False,
        "runtime_authorized": False,
    }


def validate_report(report: dict[str, Any], implementation_root_sha256: str) -> None:
    expected_fields = {
        "schema",
        "status",
        "fixture_id",
        "protocol_sha256",
        "implementation_root_sha256",
        "coarse_grid",
        "refined_grid",
        "plate_dimensions_metres",
        "beam_dimensions_metres",
        "contact_count",
        "query_evaluation_count",
        "coarse_refined_pair_count",
        "maximum_plane_residual_metres",
        "maximum_reconstruction_residual_metres",
        "maximum_affine_pair_difference",
        "affine_pair_limit",
        "canonical_query_root_sha256",
        "invariance_fixture_count",
        "mutation_fixture_count",
        "teacher_values_opened",
        "model_values_opened",
        "protected_roles_opened",
        "runtime_authorized",
    }
    if set(report) != expected_fields:
        raise base.M0Error("M0b surface report fields changed")
    if (
        report["schema"] != contract.SURFACE_REPORT_SCHEMA
        or report["status"] != "Validated"
        or report["fixture_id"] != "surface-query-official-shape-v1"
        or report["protocol_sha256"] != contract.PROTOCOL_SHA256
        or report["implementation_root_sha256"] != implementation_root_sha256
        or report["coarse_grid"] != list(COARSE_GRID)
        or report["refined_grid"] != list(REFINED_GRID)
        or report["plate_dimensions_metres"] != list(PLATE_DIMENSIONS)
        or report["beam_dimensions_metres"] != list(BEAM_DIMENSIONS)
        or report["contact_count"] != len(CONTACTS)
        or report["query_evaluation_count"] != 48
        or report["coarse_refined_pair_count"] != 24
        or report["invariance_fixture_count"] != 4
        or report["mutation_fixture_count"] < 15
        or report["affine_pair_limit"] != AFFINE_PAIR_LIMIT
        or report["maximum_affine_pair_difference"] > AFFINE_PAIR_LIMIT
        or report["maximum_plane_residual_metres"] < 0.0
        or report["maximum_reconstruction_residual_metres"] < 0.0
        or not isinstance(report["canonical_query_root_sha256"], str)
        or len(report["canonical_query_root_sha256"]) != 64
        or report["teacher_values_opened"] is not False
        or report["model_values_opened"] is not False
        or report["protected_roles_opened"] is not False
        or report["runtime_authorized"] is not False
    ):
        raise base.M0Error("M0b surface report identity, values or authority changed")
