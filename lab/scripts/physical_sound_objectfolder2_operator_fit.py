"""Shared geometry field training with a physical Rayleigh subspace objective.

Coarse P1 operators on three existing TRAIN ceramics; no eigenvector/audio labels
in this fine-tune. Previous surface-supervised weights initialize it. P2 DEV
generation remains a separate frozen-model experiment. Not NeuralSound/U-Net.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
import physical_sound_objectfolder2_physical_shared as shared
import physical_sound_objectfolder2_ritz_probe as probe
import torch
from safetensors.torch import load_file, save_file
from scipy.sparse import load_npz, save_npz


def prepare(args):
    from skfem import Basis, BilinearForm, ElementTetP1, ElementVector, MeshTet, asm
    from skfem.helpers import dot
    from skfem.models.elasticity import lame_parameters, linear_elasticity

    args.output.mkdir(parents=True)
    data = args.root / "objectfolder2-physical-shared-data-2026-09-06"
    rows = json.loads((data / "train.json").read_text())["rows"]
    if len(rows) != 3 or {r["object_id"] for r in rows} != shared.TRAIN:
        raise ValueError("exact existing TRAIN cohort required")
    output = []
    for row in rows:
        identity = row["object_id"]
        if row["role"] != "train":
            raise ValueError("TRAIN role required")
        meshroot = args.root / f"objectfolder2-elastic-{identity}-wild-2026-09-06"
        if source.sha(meshroot / "mesh.npz") != row["mesh_sha256"]:
            raise ValueError("changed mesh")
        with np.load(meshroot / "mesh.npz", allow_pickle=False) as d:
            nodes, elements = d["nodes"], d["elements"]
        if source.sha(data / row["geometry_file"]) != row["geometry_sha256"]:
            raise ValueError("changed geometry")
        with np.load(data / row["geometry_file"], allow_pickle=False) as d:
            cloud, features, length = d["cloud"], d["features"], float(d["length"])
        points = (nodes - (nodes.min(0) + nodes.max(0)) / 2) / length
        mesh = MeshTet(points.T, elements.T)
        basis = Basis(mesh, ElementVector(ElementTetP1()))
        # Verify integer DOF identity and native coordinates, not floating-point
        # equality to coordinates re-evaluated through an affine element map.
        np.testing.assert_array_equal(mesh.p.T, points)
        np.testing.assert_array_equal(
            basis.nodal_dofs, np.arange(basis.N).reshape(-1, 3).T
        )
        lam, mu = lame_parameters(1.0, 0.19)
        k = asm(linear_elasticity(lam, mu), basis).tocsr()
        m = asm(BilinearForm(lambda u, v, w: dot(u, v)), basis).tocsr()
        rigid = probe.mass_basis(probe.rigid_fields(points), m)
        prefix = args.output / f"object-{identity}"
        save_npz(str(prefix) + "-k.npz", k)
        save_npz(str(prefix) + "-m.npz", m)
        np.savez(
            str(prefix) + "-geometry.npz",
            points=points.astype(np.float32),
            cloud=cloud,
            features=features,
            rigid=rigid,
        )
        files = {
            name: {
                "file": prefix.name + "-" + name + ".npz",
                "sha256": source.sha(Path(str(prefix) + "-" + name + ".npz")),
            }
            for name in ("k", "m", "geometry")
        }
        output.append({"object_id": identity, "role": "train", "files": files})
        print("prepared", identity, basis.N, k.nnz, flush=True)
    shared.write(
        args.output / "operators.json",
        {
            "rows": output,
            "order": 1,
            "scope": "P1geometry-only TRAIN59/66/78 operators, no source acoustic/modal labels",
        },
    )


def sparse(matrix, device):
    matrix = matrix.tocsr()
    matrix.sort_indices()
    return torch.sparse_csr_tensor(
        torch.tensor(matrix.indptr, dtype=torch.int64, device=device),
        torch.tensor(matrix.indices, dtype=torch.int64, device=device),
        torch.tensor(matrix.data, dtype=torch.float64, device=device),
        size=matrix.shape,
        check_invariants=True,
    )


def rayleigh_trace(field, k, m, rigid):
    """Sum of physical Ritz eigenvalues, invariant to invertible basis changes.

    Cholesky solves avoid eigenvector derivatives near repeated eigenvalues. No
    diagonal loading can fabricate rank. This minimizes low elastic subspace energy;
    it is not the paper's separately weighted operator-residual loss.
    """
    v = field.reshape(-1, field.shape[-1]).double()
    v = v - rigid @ (rigid.T @ (m @ v))
    v = v / (v * (m @ v)).sum(0).sqrt()
    mass = v.T @ (m @ v)
    stiffness = v.T @ (k @ v)
    mass, stiffness = (mass + mass.T) / 2, (stiffness + stiffness.T) / 2
    factor = torch.linalg.cholesky(mass)
    value = torch.cholesky_solve(stiffness, factor).trace()
    if not torch.isfinite(value) or value <= 0:
        raise ValueError("invalid Rayleigh energy")
    return value


def fit(args):
    started = time.monotonic()
    rows = json.loads((args.data / "operators.json").read_text())["rows"]
    if (
        len(rows) != 3
        or {r["object_id"] for r in rows} != shared.TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact TRAIN cohort required")
    device = "cuda" if torch.cuda.is_available() else "cpu"
    receipt = json.loads((args.initial / "fit.json").read_text())
    if source.sha(args.initial / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("changed initial weights")
    model = shared.PhysicalStudent().to(device)
    model.load_state_dict(load_file(args.initial / "model.safetensors", device=device))
    # Frequencies come from operators, never this now-unused independent head.
    model.frequency.requires_grad_(False)
    items = []
    for row in rows:
        paths = {}
        for kind, record in row["files"].items():
            path = args.data / record["file"]
            if source.sha(path) != record["sha256"]:
                raise ValueError("changed TRAIN operator input")
            paths[kind] = path
        with np.load(paths["geometry"], allow_pickle=False) as d:
            g = {key: torch.tensor(d[key], device=device) for key in d.files}
        item = {
            "geometry": g,
            "k": sparse(load_npz(paths["k"]), device),
            "m": sparse(load_npz(paths["m"]), device),
            "id": row["object_id"],
        }
        with torch.no_grad():
            _, field = model(g["cloud"][None], g["features"][None], g["points"][None])
            item["normalizer"] = float(
                rayleigh_trace(field[0], item["k"], item["m"], g["rigid"])
            )
        items.append(item)
    optimizer = torch.optim.Adam(
        [p for p in model.parameters() if p.requires_grad], lr=0.001
    )
    history = []
    for step in range(300):
        item = items[step % 3]
        g = item["geometry"]
        optimizer.zero_grad(set_to_none=True)
        _, field = model(g["cloud"][None], g["features"][None], g["points"][None])
        loss = (
            rayleigh_trace(field[0], item["k"], item["m"], g["rigid"])
            / item["normalizer"]
        )
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0, error_if_nonfinite=True)
        optimizer.step()
        if (step + 1) % 30 == 0:
            record = {
                "step": step + 1,
                "object_id": item["id"],
                "loss": float(loss.detach()),
                "seconds": time.monotonic() - started,
            }
            history.append(record)
            print(record, flush=True)
    final = []
    model.eval().requires_grad_(False)
    for item in items:
        g = item["geometry"]
        _, field = model(g["cloud"][None], g["features"][None], g["points"][None])
        value = float(rayleigh_trace(field[0], item["k"], item["m"], g["rigid"]))
        final.append(
            {
                "object_id": item["id"],
                "initial_trace": item["normalizer"],
                "final_trace": value,
                "ratio": value / item["normalizer"],
            }
        )
    args.output.mkdir(parents=True)
    save_file(
        {k: v.cpu().contiguous() for k, v in model.state_dict().items()},
        args.output / "model.safetensors",
    )
    shared.write(
        args.output / "fit.json",
        {
            "weights_sha256": source.sha(args.output / "model.safetensors"),
            "requires_operator_ritz": True,
            "initial_weights_sha256": receipt["weights_sha256"],
            "script_sha256": source.sha(Path(__file__)),
            "train_ids": sorted(shared.TRAIN),
            "steps": 300,
            "lr": 0.001,
            "trainable_parameters": sum(
                p.numel()
                for n, p in model.named_parameters()
                if not n.startswith("frequency.")
            ),
            "device": device,
            "history": history,
            "final": final,
            "seconds": time.monotonic() - started,
            "scope": "one shared P1operator Rayleigh-trace fine-tune, initial surface-supervised weights; no new acoustic labels/DEV at fit; unused frequency head retained ONLY for state compatibility, must use Ritz generation",
        },
    )
    print(final, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("prepare", "fit"))
    for key in ("root", "data", "initial"):
        parser.add_argument("--" + key, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    torch.set_num_threads(4)
    {"prepare": prepare, "fit": fit}[args.stage](args)
