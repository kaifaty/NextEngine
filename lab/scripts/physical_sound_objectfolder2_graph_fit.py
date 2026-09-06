"""Shared linear multilevel graph correction, physically scored on TRAIN P1.

Frozen coordinate-field initialization; graph propagation uses actual stiffness
connectivity. No acoustic/eigenvector targets. Not NeuralSound reproduction.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
import physical_sound_objectfolder2_feature_probe as data_source
import physical_sound_objectfolder2_operator_fit as physical
import physical_sound_objectfolder2_physical_shared as shared
import physical_sound_objectfolder2_ritz_probe as probe
import torch
from safetensors.torch import load_file, save_file
from scipy.sparse import coo_matrix, diags, eye
from torch import nn

CHANNELS = 4


def hierarchy(k, points, device="cpu"):
    """Fixed geometric aggregation; Galerkin K supplies each graph's edges."""
    levels = []
    points = np.asarray(points)
    for bins in (16, 8, 4, None):
        rowsum = np.asarray(abs(k).sum(1)).ravel()
        if np.any(rowsum <= 0) or not np.isfinite(rowsum).all():
            raise ValueError("invalid stiffness graph")
        transition = (eye(k.shape[0], format="csr") - diags(1 / rowsum) @ k).tocsr()
        level = {
            "transition": physical.sparse(transition, device),
            "nnz": transition.nnz,
        }
        if bins is not None:
            cells = np.floor((points + 0.5) * bins).astype(np.int64)
            _, assignment = np.unique(cells, axis=0, return_inverse=True)
            count = int(assignment.max()) + 1
            parent = (assignment[:, None] * 3 + np.arange(3)).ravel()
            p = coo_matrix(
                (np.ones(len(parent)), (np.arange(len(parent)), parent)),
                shape=(len(parent), count * 3),
            ).tocsr()
            counts = np.bincount(parent, minlength=count * 3)
            restriction = (diags(1 / counts) @ p.T).tocsr()
            level.update(
                parent=torch.tensor(parent, dtype=torch.int64, device=device),
                restriction=physical.sparse(restriction, device),
                restriction_nnz=restriction.nnz,
            )
            k = (p.T @ k @ p).tocsr()
            sums = np.zeros((count, 3))
            np.add.at(sums, assignment, points)
            points = sums / np.bincount(assignment)[:, None]
        levels.append(level)
    return levels


class GraphLayer(nn.Module):
    def __init__(self):
        super().__init__()
        self.local = nn.Linear(CHANNELS, CHANNELS, bias=False, dtype=torch.float64)
        self.neighbor = nn.Linear(CHANNELS, CHANNELS, bias=False, dtype=torch.float64)
        with torch.no_grad():
            self.local.weight.copy_(torch.eye(CHANNELS, dtype=torch.float64))
            self.neighbor.weight.mul_(0.1)

    def forward(self, x, level):
        propagated = torch.sparse.mm(level["transition"], x.flatten(1)).reshape_as(x)
        return self.local(x) + self.neighbor(propagated)


class GraphCorrection(nn.Module):
    def __init__(self):
        super().__init__()
        self.lift = nn.Linear(1, CHANNELS, bias=False, dtype=torch.float64)
        self.down = nn.ModuleList([GraphLayer() for _ in range(4)])
        self.up = nn.ModuleList([GraphLayer() for _ in range(3)])
        self.output = nn.Linear(CHANNELS, 1, bias=False, dtype=torch.float64)
        nn.init.zeros_(self.output.weight)

    def forward(self, initial, levels):
        def visit(x, depth):
            x = self.down[depth](x, levels[depth])
            if depth < 3:
                coarse = torch.sparse.mm(levels[depth]["restriction"], x.flatten(1))
                coarse = visit(coarse.reshape(-1, x.shape[1], CHANNELS), depth + 1)
                x = self.up[depth](x + coarse[levels[depth]["parent"]], levels[depth])
            return x

        update = self.output(visit(self.lift(initial[..., None]), 0))[..., 0]
        return initial + update


def classical_steps(levels, k, m):
    # Match an explicit sparse scalar multiply count, NOT total wall-clock cost.
    work = CHANNELS * sum(
        (2 if i < 3 else 1) * level["nnz"] + level.get("restriction_nnz", 0)
        for i, level in enumerate(levels)
    )
    return max(1, int(work // (k.nnz + m.nnz))), int(work)


def relax(initial, k, m, rigid, steps):
    u = probe.remove_rigid(initial.copy(), rigid, m)
    rowsum = np.asarray(abs(k).sum(1)).ravel()
    for _ in range(steps):
        ku, mu = k @ u, m @ u
        frequency_squared = np.einsum("ij,ij->j", u, ku) / np.einsum("ij,ij->j", u, mu)
        u -= (ku - mu * frequency_squared) / rowsum[:, None]
        u /= np.linalg.norm(u, axis=0)
    if not np.isfinite(u).all():
        raise ValueError("nonfinite classical correction")
    return u


def fit(args):
    started = time.monotonic()
    device = "cuda" if torch.cuda.is_available() else "cpu"
    front = args.root / "objectfolder2-operator-fit-2026-09-06"
    receipt = json.loads((front / "fit.json").read_text())
    if source.sha(front / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("changed frozen front end")
    coordinate = shared.PhysicalStudent().eval().requires_grad_(False)
    coordinate.load_state_dict(load_file(front / "model.safetensors"))
    torch.manual_seed(42)
    model = GraphCorrection().to(device)
    items = []
    for identity in sorted(shared.TRAIN):
        k, m, g, _, _ = data_source.load_inputs(args.root, identity)
        with torch.no_grad():
            _, v = coordinate(
                *[torch.tensor(g[n][None]) for n in ("cloud", "features", "points")]
            )
        initial = torch.tensor(
            v[0].numpy().reshape(-1, 32), dtype=torch.float64, device=device
        )
        kt, mt = physical.sparse(k, device), physical.sparse(m, device)
        rigid = torch.tensor(g["rigid"], device=device)
        levels = hierarchy(k, g["points"], device)
        normalizer = float(physical.rayleigh_trace(initial, kt, mt, rigid))
        steps, work = classical_steps(levels, k, m)
        items.append(
            {
                "id": identity,
                "initial": initial,
                "k": kt,
                "m": mt,
                "rigid": rigid,
                "levels": levels,
                "normalizer": normalizer,
                "classical_steps": steps,
                "graph_sparse_work_per_column": work,
            }
        )
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    history = []
    for step in range(300):
        item = items[step % 3]
        optimizer.zero_grad(set_to_none=True)
        predicted = model(item["initial"], item["levels"])
        loss = (
            physical.rayleigh_trace(predicted, item["k"], item["m"], item["rigid"])
            / item["normalizer"]
        )
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1.0, error_if_nonfinite=True)
        optimizer.step()
        if (step + 1) % 30 == 0:
            row = {
                "step": step + 1,
                "object_id": item["id"],
                "loss": float(loss.detach()),
                "seconds": time.monotonic() - started,
            }
            history.append(row)
            print(row, flush=True)
    model.eval().requires_grad_(False)
    final = []
    for item in items:
        value = float(
            physical.rayleigh_trace(
                model(item["initial"], item["levels"]),
                item["k"],
                item["m"],
                item["rigid"],
            )
        )
        final.append(
            {
                "object_id": item["id"],
                "initial_trace": item["normalizer"],
                "final_trace": value,
                "ratio": value / item["normalizer"],
                "classical_steps": item["classical_steps"],
            }
        )
    args.output.mkdir(parents=True)
    save_file(
        {n: t.cpu().contiguous() for n, t in model.state_dict().items()},
        args.output / "model.safetensors",
    )
    shared.write(
        args.output / "fit.json",
        {
            "weights_sha256": source.sha(args.output / "model.safetensors"),
            "front_weights_sha256": receipt["weights_sha256"],
            "script_sha256": source.sha(Path(__file__)),
            "train_ids": sorted(shared.TRAIN),
            "parameters": sum(p.numel() for p in model.parameters()),
            "steps": 300,
            "seed": 42,
            "lr": 0.001,
            "device": device,
            "history": history,
            "final": final,
            "seconds": time.monotonic() - started,
            "scope": "one shared linear four-level stiffness-graph correction, TRAIN P1 Rayleigh objective; frozen ce01 coordinate front end; no mode/audio targets, noDEV; P2transfer unproven; not exact inverse or pressure model",
        },
    )
    print(final, flush=True)


def generate_correction(path, front_hash, initial, k, m, points, rigid):
    """CPU generation hook: geometric operators and own model weights only."""
    receipt = json.loads((path / "fit.json").read_text())
    if (
        receipt["front_weights_sha256"] != front_hash
        or source.sha(path / "model.safetensors") != receipt["weights_sha256"]
    ):
        raise ValueError("changed graph model or incompatible front end")
    started = time.monotonic()
    levels = hierarchy(k, points)
    construction = time.monotonic() - started
    model = GraphCorrection().eval().requires_grad_(False)
    model.load_state_dict(load_file(path / "model.safetensors"))
    started = time.monotonic()
    corrected = model(torch.tensor(initial), levels).numpy()
    neural_seconds = time.monotonic() - started
    steps, work = classical_steps(levels, k, m)
    started = time.monotonic()
    baseline = relax(initial, k, m, rigid, steps)
    seconds = time.monotonic() - started
    info = {
        "weights_sha256": receipt["weights_sha256"],
        "construction_seconds": construction,
        "graph_seconds": neural_seconds,
        "classical_seconds": seconds,
        "classical_steps": steps,
        "graph_sparse_work_per_column": work,
        "classical_sparse_work_per_column": steps * (k.nnz + m.nnz),
        "scope": "matched sparse multiply-count budget only; dense channel mixing, hierarchy construction and gathers not matched; no equal-runtime claim",
    }
    print(info, flush=True)
    return corrected, baseline, info


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    torch.set_num_threads(4)
    fit(args)
