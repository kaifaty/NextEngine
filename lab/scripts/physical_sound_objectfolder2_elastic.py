"""OF2 ceramic elastic-operator control, not neural or acoustic pressure.

No source sound/model enters meshing or elastic solving. Preserve the original
PLC boundary; fail instead of repairing, smoothing or rescaling from CSV.
An explicit wild backend separately labels approximate geometry after strict
meshing rejection; it never claims native boundary identity.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components

CERAMIC_ROLES = {59: "train", 66: "train", 78: "train", 88: "development"}


def source_mesh(manifest, identity):
    """Existing fixed ceramic TRAIN/DEV identities; no role inference from sound."""
    if identity not in CERAMIC_ROLES:
        raise ValueError("fixed ceramic cohort required")
    if "rows" in manifest:
        row = next(r for r in manifest["rows"] if r["object_id"] == identity)
        if row["role"] != CERAMIC_ROLES[identity] or row["material"] != "Ceramic":
            raise ValueError("ceramic role/material mismatch")
        return row["files"]["model.obj"]
    metadata = manifest["object_metadata"][str(identity)]
    if metadata[3] != "Ceramic":
        raise ValueError("ceramic material required")
    row = next(
        r
        for r in manifest["files"]
        if r["member"].split("/")[-2:] == [str(identity), "model.obj"]
    )
    return {"path": row["file"], "sha256": row["sha256"]}


def surface(path):
    lines = path.read_text().splitlines()
    vertices = np.array(
        [
            [float(x) for x in line.split()[1:4]]
            for line in lines
            if line.startswith("v ")
        ]
    )
    faces = [
        [int(x.split("/")[0]) - 1 for x in line.split()[1:]]
        for line in lines
        if line.startswith("f ")
    ]
    if not faces or any(len(f) != 3 for f in faces):
        raise ValueError("triangular surface required; no automatic triangulation")
    faces = np.asarray(faces, dtype=np.int32)
    return vertices, faces, validate_surface(vertices, faces)


def validate_surface(vertices, faces):
    if (
        vertices.ndim != 2
        or vertices.shape[1] != 3
        or not np.isfinite(vertices).all()
        or faces.ndim != 2
        or faces.shape[1] != 3
        or not np.issubdtype(faces.dtype, np.integer)
        or len(faces) < 4
        or faces.min() < 0
        or faces.max() >= len(vertices)
        or len(np.unique(vertices, axis=0)) != len(vertices)
    ):
        raise ValueError("finite unique vertices and valid triangle indices required")
    edges = np.concatenate([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]])
    ordered = np.sort(edges, axis=1)
    unique, inverse, counts = np.unique(
        ordered, axis=0, return_inverse=True, return_counts=True
    )
    balance = np.bincount(inverse, weights=np.where(edges[:, 0] < edges[:, 1], 1, -1))
    if np.any(counts != 2) or np.any(balance != 0):
        raise ValueError("closed consistently oriented indexed surface required")
    graph = coo_matrix(
        (np.ones(len(unique)), (unique[:, 0], unique[:, 1])),
        shape=(len(vertices), len(vertices)),
    ).tocsr()
    components, labels = connected_components(graph, directed=False)
    # Translation improves volume conditioning without changing native geometry.
    triangles = (vertices - vertices.mean(0))[faces]
    areas = np.cross(
        triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
    )
    if np.any(np.linalg.norm(areas, axis=1) == 0):
        raise ValueError("degenerate surface triangle")
    volume = float(np.einsum("ij,ij->i", triangles[:, 0], areas).sum() / 6)
    if not np.isfinite(volume) or volume == 0:
        raise ValueError("nonzero oriented surface volume required")
    volumes, parts = [], []
    for i in range(components):
        part = faces[labels[faces[:, 0]] == i]
        x = (vertices - vertices.mean(0))[part]
        signed = np.einsum("ij,ij->i", x[:, 0], np.cross(x[:, 1], x[:, 2])).sum() / 6
        volumes.append(float(signed))
        parts.append(part)
    outer = int(np.argmax(np.abs(volumes)))
    holes = []
    for i, part in enumerate(parts):
        if i == outer:
            continue
        if volumes[i] * volumes[outer] >= 0:
            raise ValueError("disconnected solids are not one elastic body")
        point = vertices[np.unique(part)].mean(0)
        if not (
            abs(abs(winding(vertices[part], point)) - 1) < 1e-8
            and abs(abs(winding(vertices[parts[outer]], point)) - 1) < 1e-8
        ):
            raise ValueError("cavity seed not proven inside inner and outer shells")
        holes.append(point.tolist())
    return {
        "signed_volume": volume,
        "indexed_edges": len(unique),
        "component_signed_volumes": volumes,
        "hole_seeds": holes,
    }


def winding(triangles, point):
    """Oriented solid angle: an interior point of a closed shell has |w|=1."""
    a, b, c = np.moveaxis(triangles - point, 1, 0)
    la, lb, lc = [np.linalg.norm(v, axis=1) for v in (a, b, c)]
    numerator = np.einsum("ij,ij->i", a, np.cross(b, c))
    denominator = (
        la * lb * lc
        + np.einsum("ij,ij->i", a, b) * lc
        + np.einsum("ij,ij->i", b, c) * la
        + np.einsum("ij,ij->i", c, a) * lb
    )
    return float(np.arctan2(numerator, denominator).sum() / (2 * np.pi))


def mesh(args):
    import tetgen

    manifest = json.loads(args.manifest.read_text())
    item = source_mesh(manifest, args.object_id)
    path = Path(item["path"])
    if source.sha(path) != item["sha256"]:
        raise ValueError("source geometry changed")
    vertices, faces, stats = surface(path)
    options = {
        "order": 1,
        "quality": True,
        "minratio": 2.0,
        "nobisect": True,
        "nomergefacet": True,
        "nomergevertex": True,
        "steinerleft": 20000,
        "docheck": True,
        "quiet": False,
    }
    tgen = tetgen.TetGen(vertices, faces)
    for point in stats["hole_seeds"]:
        tgen.add_hole(point)
    args.output.mkdir(parents=True)
    # TetGen writes failure diagnostics into cwd even without an output request.
    previous_cwd = Path.cwd()
    try:
        os.chdir(args.output)
        nodes, elements, _, _ = tgen.tetrahedralize(**options)
    except RuntimeError as exc:
        (args.output / "failure.json").write_text(
            json.dumps(
                {
                    "object_id": args.object_id,
                    "source_mesh_sha256": item["sha256"],
                    "options": options,
                    "surface": stats,
                    "error": str(exc),
                    "scope": "meshing rejected; original mesh unchanged, no elastic/audio result",
                },
                indent=2,
            )
            + "\n"
        )
        raise
    finally:
        os.chdir(previous_cwd)
    if len(nodes) > 30000 or len(elements) > 150000 or elements.shape[1] != 4:
        raise ValueError("bounded linear volume mesh required")
    if not np.array_equal(nodes[: len(vertices)], vertices):
        raise ValueError("input vertices changed or reordered")
    facets = np.sort(
        np.concatenate(
            [elements[:, f] for f in ([0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3])]
        ),
        axis=1,
    )
    unique, counts = np.unique(facets, axis=0, return_counts=True)
    boundary = unique[counts == 1]
    original = np.sort(faces, axis=1)
    original = original[np.lexsort(original.T[::-1])]
    if np.any(counts > 2) or not np.array_equal(boundary, original):
        raise ValueError("native boundary was not preserved exactly")
    t = nodes[elements]
    volumes = np.abs(np.linalg.det(t[:, 1:] - t[:, :1])) / 6
    error = abs(volumes.sum() / abs(stats["signed_volume"]) - 1)
    if not np.isfinite(volumes).all() or volumes.min() <= 0 or error > 1e-8:
        raise ValueError("invalid cells or filled volume differs from input shell")
    np.savez(args.output / "mesh.npz", nodes=nodes, elements=elements, faces=faces)
    report = {
        "object_id": args.object_id,
        "source_mesh_sha256": item["sha256"],
        "manifest_sha256": source.sha(args.manifest),
        "options": options,
        "tetgen_version": tetgen.__version__,
        "vertices": len(vertices),
        "nodes": len(nodes),
        "cells": len(elements),
        "surface": stats,
        "volume_relative_error": float(error),
        "minimum_cell_volume": float(volumes.min()),
        "boundary_exact": True,
        "mesh_sha256": source.sha(args.output / "mesh.npz"),
        "script_sha256": source.sha(Path(__file__)),
        "scope": "native fixed-cohort ceramic geometry only; exact indexed surface, no source acoustics, no repair; not yet elastic solve",
    }
    (args.output / "mesh.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2), flush=True)


def wild_mesh(args):
    # Direct installed native extension: 0.4.2 package __init__ unconditionally
    # imports optional PyVista. Array API mirrors the inspected shipped wrapper.
    import PyfTetWildWrapper as wild

    data = json.loads(args.manifest.read_text())
    item = source_mesh(data, args.object_id)
    path = Path(item["path"])
    if source.sha(path) != item["sha256"]:
        raise ValueError("source geometry changed")
    vertices, faces, stats = surface(path)
    args.output.mkdir(parents=True)
    old_cwd = Path.cwd()
    try:
        os.chdir(args.output)
        nodes, elements = wild.tetrahedralize_mesh(
            vertices,
            faces.astype(np.uint32),
            True,
            False,
            0.05,
            0.0,
            0.001,
            10.0,
            False,
            4,
            80,
            2,
            False,
            True,
            False,
            np.empty((0, 3)),
            np.empty((0, 4), np.int32),
            np.empty(0),
        )
    finally:
        os.chdir(old_cwd)
    if (
        nodes.ndim != 2
        or nodes.shape[1] != 3
        or elements.ndim != 2
        or elements.shape[1] != 4
        or len(nodes) > 30000
        or len(elements) > 150000
    ):
        raise ValueError("bounded volume mesh required")
    t = nodes[elements]
    volumes = np.abs(np.linalg.det(t[:, 1:] - t[:, :1])) / 6
    error = abs(volumes.sum() / abs(stats["signed_volume"]) - 1)
    if not np.isfinite(volumes).all() or volumes.min() <= 0:
        raise ValueError("invalid volume elements")
    # Keep even a geometrically rejected report-only result; never train from it.
    np.savez(
        args.output / "mesh.npz",
        nodes=nodes,
        elements=elements,
        source_vertices=vertices,
        source_faces=faces,
    )
    report = {
        "object_id": args.object_id,
        "source_mesh_sha256": item["sha256"],
        "mesh_sha256": source.sha(args.output / "mesh.npz"),
        "role": CERAMIC_ROLES[args.object_id],
        "script_sha256": source.sha(Path(__file__)),
        "backend": "pytetwild0.4.2 native array API",
        "native_boundary_exact": False,
        "relative_envelope": 0.001,
        "envelope_meters": float(np.linalg.norm(np.ptp(vertices, axis=0)) * 0.001),
        "edge_length_fac": 0.05,
        "threads": 4,
        "max_opt_iterations": 80,
        "nodes": len(nodes),
        "cells": len(elements),
        "source_surface": stats,
        "volume": float(volumes.sum()),
        "volume_relative_error": float(error),
        "volume_one_percent_check": bool(error < 0.01),
        "scope": "explicit approximate geometry; envelope is requested, independent distance/elastic checks pending; no source acoustics read",
    }
    (args.output / "mesh.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--object-id", type=int, choices=tuple(CERAMIC_ROLES), default=59
    )
    parser.add_argument("--backend", choices=("strict", "wild"), default="strict")
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    (mesh if args.backend == "strict" else wild_mesh)(args)
