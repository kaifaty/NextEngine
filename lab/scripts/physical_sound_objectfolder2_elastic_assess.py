"""Geometry-only approximation checks and same-gain P1/P2 audible comparison."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
import physical_sound_objectfolder2_elastic as meshing
import physical_sound_objectfolder2_elastic_solve as elastic
from physical_sound_sonicgauss_waveform_fit import audio_metrics
from scipy.io import wavfile


def surface_distances(points, triangles):
    """Exact nearest-triangle distance for each sampled point; not Hausdorff."""
    a, b, c = np.moveaxis(triangles, 1, 0)
    e, f = b - a, c - a
    ee, ef, ff = [np.einsum("ij,ij->i", x, y) for x, y in ((e, e), (e, f), (f, f))]
    normal = np.cross(e, f)
    nn = np.einsum("ij,ij->i", normal, normal)
    if np.any(nn <= 0):
        raise ValueError("nondegenerate triangles required for distance check")
    result = []
    for start in range(0, len(points), 16):
        p = points[start : start + 16, None]
        w = p - a
        we, wf = np.einsum("qfi,fi->qf", w, e), np.einsum("qfi,fi->qf", w, f)
        u, v = (ff * we - ef * wf) / nn, (ee * wf - ef * we) / nn
        plane = np.einsum("qfi,fi->qf", w, normal) ** 2 / nn
        distance = np.where((u >= 0) & (v >= 0) & (u + v <= 1), plane, np.inf)
        for x, y in ((a, b), (b, c), (c, a)):
            edge = y - x
            delta = p - x
            t = np.clip(
                np.einsum("qfi,fi->qf", delta, edge)
                / np.einsum("fi,fi->f", edge, edge),
                0,
                1,
            )
            difference = delta - t[..., None] * edge
            distance = np.minimum(
                distance, np.einsum("qfi,qfi->qf", difference, difference)
            )
        result.extend(np.sqrt(distance.min(1)))
    return np.array(result)


def geometry(args):
    info = json.loads((args.mesh / "mesh.json").read_text())
    if source.sha(args.mesh / "mesh.npz") != info["mesh_sha256"]:
        raise ValueError("mesh changed")
    with np.load(args.mesh / "mesh.npz", allow_pickle=False) as data:
        nodes, cells, vertices, original = [
            data[k] for k in ("nodes", "elements", "source_vertices", "source_faces")
        ]
    faces, ids, _ = elastic.boundary(nodes, cells)
    remap = np.full(len(nodes), -1)
    remap[ids] = np.arange(len(ids))
    structure = meshing.validate_surface(nodes[ids], remap[faces])
    directions = {}
    for name, p, triangles in (
        (
            "output_to_source",
            np.vstack([nodes[ids], nodes[faces].mean(1)]),
            vertices[original],
        ),
        (
            "source_to_output",
            np.vstack([vertices, vertices[original].mean(1)]),
            nodes[faces],
        ),
    ):
        d = surface_distances(p, triangles)
        directions[name] = {
            "sample_count": len(p),
            "max_meters": float(d.max()),
            "rms_meters": float(np.sqrt(np.mean(d * d))),
            "within_requested_envelope": bool(d.max() <= info["envelope_meters"]),
        }
        print(name, directions[name], flush=True)
    result = {
        "mesh_sha256": info["mesh_sha256"],
        "output_surface": structure,
        "distances": directions,
        "scope": "all boundary vertices and face centroids in both directions; exact nearest triangle for these samples, NOT continuous Hausdorff or acoustic convergence",
    }
    (args.output / "geometry.json").write_text(json.dumps(result, indent=2) + "\n")


def compare(args):
    inputs = []
    for path in (args.coarse, args.fine):
        with np.load(path / "modes.npz", allow_pickle=False) as data:
            inputs.append(dict(data))
    a, b = inputs
    for key in ("contact_node_ids", "contact_positions", "contact_directions"):
        np.testing.assert_array_equal(a[key], b[key])
    waves = [elastic.port_response(d["omega"], d["ports"])[0] for d in inputs]
    # Metres -> audition PCM, one conversion across both solvers/all contacts.
    # Relative level is preserved; raw metre amplitudes are below PCM metric floors.
    gain = 0.5 / max(abs(w).max() for w in waves)
    metrics = [audio_metrics(waves[1][i] * gain, waves[0][i] * gain) for i in range(3)]
    galleries = []
    for i in range(3):
        gallery = (np.concatenate([w[i] for w in waves]) * gain).astype(np.float32)
        wavfile.write(args.output / f"contact-{i}-p1-p2.wav", source.RATE, gallery)
        wavfile.write(
            args.output / f"contact-{i}-p2-audition.wav",
            source.RATE,
            (waves[1][i] * gain).astype(np.float32),
        )
        galleries.append(gallery)
    wavfile.write(
        args.output / "three-contacts-p1-p2.wav", source.RATE, np.concatenate(galleries)
    )
    wavfile.write(
        args.output / "three-contacts-p2-audition.wav",
        source.RATE,
        (waves[1].ravel() * gain).astype(np.float32),
    )
    result = {
        "common_gain": float(gain),
        "audio_metrics": metrics,
        "raw_rms_meters": [np.sqrt(np.mean(w * w, axis=1)).tolist() for w in waves],
        "relative_frequency_difference": (abs(a["omega"] / b["omega"] - 1)).tolist(),
        "coarse_modes_sha256": source.sha(args.coarse / "modes.npz"),
        "fine_modes_sha256": source.sha(args.fine / "modes.npz"),
        "scope": "same approximate mesh and physical ports, P1 then P2 for each contact; no per-wave normalization; neither is a measured recording or neural generation",
    }
    (args.output / "comparison.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2), flush=True)


def reference(args):
    # Post-generation diagnostic only. This target is never an elastic input.
    rows = json.loads((args.reference / "data.json").read_text())["rows"]
    row = next(r for r in rows if r["object_id"] == 59)
    if row["role"] != "train":
        raise ValueError("TRAIN59 reference only")
    path = args.reference / row["file"]
    if source.sha(path) != row["sha256"]:
        raise ValueError("reference changed")
    with np.load(path, allow_pickle=False) as d:
        target = np.sqrt((2 * np.pi * d["frequency"]) ** 2 + d["damping"] ** 2) / (
            2 * np.pi
        )
    if np.any(np.diff(target) <= 0):
        raise ValueError("ordered distinct reference frequencies required")
    result = {
        "target_sha256": row["sha256"],
        "target_mesh_sha256": row["mesh_sha256"],
        "rows": [],
    }
    for label, path in (("P1", args.coarse), ("P2", args.fine)):
        with np.load(path / "modes.npz", allow_pickle=False) as d:
            hz = d["omega"] / (2 * np.pi)
        errors = abs(hz / target[: len(hz)] - 1)
        result["rows"].append(
            {
                "label": label,
                "modes_sha256": source.sha(path / "modes.npz"),
                "relative_errors": errors.tolist(),
                "mean": float(errors.mean()),
                "max": float(errors.max()),
            }
        )
    result["scope"] = (
        "post-generation TRAIN59 ordered lowest32 natural-frequency comparison; no coefficient/eigenvector matching, pressure/realism, predeclared acceptance or generalization claim"
    )
    (args.output / "reference.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("geometry", "compare", "reference"))
    for key in ("mesh", "coarse", "fine", "reference"):
        parser.add_argument("--" + key, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    args.output.mkdir(parents=True)
    {"geometry": geometry, "compare": compare, "reference": reference}[args.stage](args)
