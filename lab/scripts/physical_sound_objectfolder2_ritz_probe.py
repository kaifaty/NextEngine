"""Frozen NN versus P1 initialization, one identical P2 inverse/Ritz correction.

Open DEV88 only. Generation assembles operators from geometry; it never reads
reference eigenvectors or sound. This is an offline hybrid diagnostic, not a
newly trained network, NeuralSound reproduction, pressure or runtime synthesis.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
import physical_sound_objectfolder2_physical_shared as shared
import torch
from safetensors.torch import load_file
from scipy.io import wavfile
from scipy.linalg import eigh
from scipy.sparse.linalg import eigsh, splu


def mass_basis(v, m):
    """Scale before symmetric orthogonalization; reject rank loss explicitly."""
    norms = np.sqrt(np.einsum("ij,ij->j", v, m @ v))
    if not np.isfinite(norms).all() or np.any(norms <= 0):
        raise ValueError("invalid mass norms")
    v = v / norms
    gram = v.T @ (m @ v)
    values, axes = eigh((gram + gram.T) / 2)
    if values[0] <= values[-1] * 1e-12:
        raise ValueError("rank deficient subspace")
    return v @ (axes / np.sqrt(values))


def rigid_fields(points):
    v = np.zeros((len(points), 3, 6))
    v[:, :, :3] = np.eye(3)
    for axis in range(3):
        v[:, :, axis + 3] = np.cross(np.eye(3)[axis], points)
    return v.reshape(-1, 6)


def remove_rigid(v, rigid, m):
    return v - rigid @ (rigid.T @ (m @ v))


def ritz(v, k, m, rigid):
    q = mass_basis(remove_rigid(v, rigid, m), m)
    small = q.T @ (k @ q)
    values, axes = eigh((small + small.T) / 2)
    u = q @ axes
    ku, mu = k @ u, m @ u
    residual = np.linalg.norm(ku - mu * values, axis=0) / (
        np.linalg.norm(ku, axis=0) + np.linalg.norm(mu * values, axis=0)
    )
    if np.any(values <= 0) or not np.isfinite(residual).all():
        raise ValueError("invalid elastic Ritz result")
    return values, u, residual


def generate(args):
    from skfem import (
        Basis,
        BilinearForm,
        ElementTetP1,
        ElementTetP2,
        ElementVector,
        MeshTet,
        asm,
    )
    from skfem.helpers import dot
    from skfem.models.elasticity import lame_parameters, linear_elasticity

    started = time.monotonic()
    meshroot = args.root / "objectfolder2-elastic-88-wild-2026-09-06"
    data = args.root / "objectfolder2-physical-shared-data-2026-09-06"
    fit = args.fit or args.root / "objectfolder2-physical-shared-fit-2026-09-06"
    meta = json.loads((meshroot / "mesh.json").read_text())
    row = next(
        r
        for r in json.loads((data / "data.json").read_text())["rows"]
        if r["object_id"] == 88
    )
    if row["role"] != "development" or meta["object_id"] != 88:
        raise ValueError("only existing open DEV88 allowed")
    if source.sha(meshroot / "mesh.npz") != row["mesh_sha256"]:
        raise ValueError("changed mesh")
    with np.load(meshroot / "mesh.npz", allow_pickle=False) as d:
        nodes, elements = d["nodes"], d["elements"]
    if source.sha(data / row["geometry_file"]) != row["geometry_sha256"]:
        raise ValueError("changed geometry")
    with np.load(data / row["geometry_file"], allow_pickle=False) as d:
        g = dict(d)
    receipt = json.loads((fit / "fit.json").read_text())
    if source.sha(fit / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("changed frozen weights")
    center = (nodes.min(0) + nodes.max(0)) / 2
    xyz = (nodes - center) / float(g["length"])
    mesh = MeshTet(xyz.T, elements.T)
    b1, b2 = [Basis(mesh, ElementVector(e)) for e in (ElementTetP1(), ElementTetP2())]
    np.testing.assert_array_equal(b1.nodal_dofs, b2.nodal_dofs)
    points = b2.doflocs[:, ::3].T
    np.testing.assert_array_equal(b2.doflocs, np.repeat(points.T, 3, axis=1))
    if b2.N > 200000:
        raise ValueError("bounded DEV probe exceeded DOF limit")
    lam, mu = lame_parameters(1.0, 0.19)
    k = asm(linear_elasticity(lam, mu), b2).tocsr()
    m = asm(BilinearForm(lambda u, v, w: dot(u, v)), b2).tocsr()
    rigid = mass_basis(rigid_fields(points), m)
    geometry_seconds = time.monotonic() - started
    print("assembled", b2.N, geometry_seconds, flush=True)

    begin = time.monotonic()
    model = shared.PhysicalStudent().eval().requires_grad_(False)
    model.load_state_dict(load_file(fit / "model.safetensors"), strict=True)
    cloud = torch.tensor(g["cloud"][None])
    features = torch.tensor(g["features"][None])
    fields = []
    with torch.no_grad():
        for chunk in np.array_split(points, int(np.ceil(len(points) / 2048))):
            _, f = model(
                cloud, features, torch.tensor(chunk[None], dtype=torch.float32)
            )
            fields.append(f[0].numpy().astype(float))
    neural = np.concatenate(fields).reshape(b2.N, 32)
    neural_seconds = time.monotonic() - begin
    print("neural initialization", neural_seconds, flush=True)

    begin = time.monotonic()
    k1 = asm(linear_elasticity(lam, mu), b1).tocsr()
    m1 = asm(BilinearForm(lambda u, v, w: dot(u, v)), b1).tocsr()
    values1, u1 = eigsh(
        k1, k=38, M=m1, sigma=-1e-4, which="LM", tol=1e-9, v0=np.linspace(1, 2, b1.N)
    )
    order = np.argsort(values1)
    values1, u1 = values1[order], u1[:, order]
    if np.max(abs(values1[:6])) > values1[6] * 1e-6 or values1[6] <= 0:
        raise ValueError("P1 six rigid modes missing")
    p1 = np.empty_like(neural)
    p1[b2.nodal_dofs] = u1[b1.nodal_dofs, 6:]
    # Exact prolongation on the same tetrahedra: P1 at P2 edge midpoint.
    p1[b2.edge_dofs] = (
        u1[b1.nodal_dofs[:, mesh.edges[0]], 6:]
        + u1[b1.nodal_dofs[:, mesh.edges[1]], 6:]
    ) / 2
    p1_seconds = time.monotonic() - begin
    print("P1 initialization", p1_seconds, flush=True)
    args.output.mkdir(parents=True)
    results, bases = [], {}
    query_dofs = b2.nodal_dofs[:, row["point_node_ids"]].T

    def save(name, v, elapsed):
        values, u, residual = ritz(v, k, m, rigid)
        np.savez(
            args.output / f"{name}.npz", omega=np.sqrt(values), field=u[query_dofs]
        )
        result = {
            "name": name,
            "max_residual": float(residual.max()),
            "mean_residual": float(residual.mean()),
            "mass_error": float(abs(u.T @ (m @ u) - np.eye(32)).max()),
            "initialization_or_solve_seconds": elapsed,
            "sha256": source.sha(args.output / f"{name}.npz"),
        }
        results.append(result)
        print(result, flush=True)
        return u

    bases["neural"] = save("neural-ritz", neural, neural_seconds)
    bases["p1"] = save("p1-ritz", p1, p1_seconds)
    begin = time.monotonic()
    factor = splu((k + 1e-4 * m).tocsc())
    factor_seconds = time.monotonic() - begin
    print("shared factorization", factor_seconds, flush=True)
    for name, initial in bases.items():
        begin = time.monotonic()
        corrected = factor.solve(m @ initial)
        elapsed = time.monotonic() - begin
        save(name + "-inverse1", corrected, elapsed)
    shared.write(
        args.output / "generation.json",
        {
            "rows": results,
            "mesh_sha256": row["mesh_sha256"],
            "weights_sha256": receipt["weights_sha256"],
            "script_sha256": source.sha(Path(__file__)),
            "geometry_assembly_seconds": geometry_seconds,
            "shared_factor_seconds": factor_seconds,
            "total_seconds": time.monotonic() - started,
            "scope": "fresh geometry-only P2 operators; frozen NN vs freshly solved P1 initialization;32vectors,oneidenticalinversecorrectioneach; no targetacoustics; P1 has additional eigensolve cost, not equal total runtime; openDEV88 only",
        },
    )


def render_variants(variants, geometry):
    waves, rejected = {}, {}
    for name, v in variants.items():
        try:
            waves[name] = shared.physical_waves(v["omega"], v["field"], geometry)
        except ValueError as error:
            if str(error) != "no audible oscillatory modes" or name == "reference":
                raise
            # Preserve a failed candidate, without pitch shifting or fake silence.
            rejected[name] = str(error)
    return waves, rejected


def assess(args):
    data = args.root / "objectfolder2-physical-shared-data-2026-09-06"
    generated = json.loads((args.generated / "generation.json").read_text())
    with np.load(data / "object-88-geometry.npz", allow_pickle=False) as d:
        g = dict(d)
    with np.load(data / "object-88-target.npz", allow_pickle=False) as d:
        target = dict(d)
    variants = {"reference": target}
    if args.baseline_generated is not None:
        old = json.loads((args.baseline_generated / "generation.json").read_text())
        if old["mesh_sha256"] != generated["mesh_sha256"]:
            raise ValueError("baseline geometry mismatch")
        row = next(r for r in old["rows"] if r["name"] == "neural-inverse1")
        path = args.baseline_generated / "neural-inverse1.npz"
        if source.sha(path) != row["sha256"]:
            raise ValueError("changed previous hybrid")
        with np.load(path, allow_pickle=False) as d:
            variants["previous-neural-inverse1"] = dict(d)
    with np.load(
        args.root / "objectfolder2-physical-shared-render-2026-09-06/object-88.npz",
        allow_pickle=False,
    ) as d:
        variants["standalone"] = dict(d)
    for row in generated["rows"]:
        path = args.generated / (row["name"] + ".npz")
        if source.sha(path) != row["sha256"]:
            raise ValueError("changed candidate")
        with np.load(path, allow_pickle=False) as d:
            variants[row["name"]] = dict(d)
    waves, rejected = render_variants(variants, g)
    gain = 0.5 / max(abs(w).max() for w in waves.values())
    args.output.mkdir(parents=True)
    results = {}
    for name, wave in waves.items():
        if name == "reference":
            continue
        metrics = [
            shared.audio_metrics(a * gain, b * gain)
            for a, b in zip(waves["reference"], wave, strict=True)
        ]
        results[name] = {
            "mean": {k: float(np.mean([r[k] for r in metrics])) for k in metrics[0]},
            "frequency_mean_relative_error": float(
                np.mean(abs(variants[name]["omega"] / target["omega"] - 1))
            ),
            "contacts": metrics,
        }
        print(
            name,
            {k: v for k, v in results[name].items() if k != "contacts"},
            flush=True,
        )
    before = "previous-neural-inverse1" if args.baseline_generated else "standalone"
    order = ["reference", before, "neural-inverse1", "p1-inverse1"]
    for contact in (0, 24, 47):
        for name in waves:
            wavfile.write(
                args.output / f"contact-{contact}-{name}.wav",
                source.RATE,
                (waves[name][contact] * gain).astype(np.float32),
            )
        wavfile.write(
            args.output / f"contact-{contact}-comparison.wav",
            source.RATE,
            (np.concatenate([waves[n][contact] for n in order]) * gain).astype(
                np.float32
            ),
        )
    shared.write(
        args.output / "assessment.json",
        {
            "results": results,
            "not_renderable": rejected,
            "common_gain_all_variants_contacts": gain,
            "order": order,
            "scope": "48contacts ONE already-open DEV88; displacement sonification, not pressure/realism; no fit/checkpoint/iteration selection",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("generate", "assess"))
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--generated", type=Path)
    parser.add_argument("--fit", type=Path)
    parser.add_argument("--baseline-generated", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    torch.set_num_threads(4)
    {"generate": generate, "assess": assess}[args.stage](args)
