"""One shared physical-port network: three ceramic TRAIN shapes, one open DEV.

Geometry -> dimensionless frequencies and vector modal fields. Train invariant
port matrices, not arbitrary eigenvector signs. No recording/operator at NN
inference. Restricted ceramic experiment, not pressure or arbitrary materials.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
import physical_sound_objectfolder2_elastic_solve as elastic
import physical_sound_objectfolder2_shared as legacy
import torch
from physical_sound_sonicgauss_waveform_fit import audio_metrics
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from scipy.spatial.distance import cdist
from torch import nn

TRAIN = {59, 66, 78}
DEV = {88}


def farthest(points, count):
    if count > len(points) or not np.isfinite(points).all():
        raise ValueError("bounded finite sampling required")
    selected = []
    distance = np.full(len(points), np.inf)
    index = int(np.lexsort(points.T[::-1])[0])
    for _ in range(count):
        selected.append(index)
        distance = np.minimum(distance, np.sum((points - points[index]) ** 2, axis=1))
        distance[selected] = -1
        index = int(distance.argmax())
    return np.array(selected)


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def prepare(args):
    args.output.mkdir(parents=True)
    rows = []
    for identity in sorted(TRAIN | DEV):
        meshroot = args.root / f"objectfolder2-elastic-{identity}-wild-2026-09-06"
        solve = args.root / f"objectfolder2-elastic-{identity}-p2-2026-09-06"
        meta = json.loads((meshroot / "mesh.json").read_text())
        result = json.loads((solve / "result.json").read_text())
        checked_geometry = json.loads(
            (
                args.root
                / f"objectfolder2-elastic-{identity}-geometry-2026-09-06/geometry.json"
            ).read_text()
        )
        if checked_geometry["mesh_sha256"] != meta["mesh_sha256"] or not all(
            r["within_requested_envelope"]
            for r in checked_geometry["distances"].values()
        ):
            raise ValueError("geometry approximation checks failed or mismatched")
        if (
            meta["object_id"] != identity
            or meta["mesh_sha256"] != result["mesh_sha256"]
            or result["order"] != 2
        ):
            raise ValueError("geometry/P2 identity mismatch")
        if source.sha(meshroot / "mesh.npz") != meta["mesh_sha256"]:
            raise ValueError("changed mesh")
        if (
            not meta["volume_one_percent_check"]
            or result["max_elastic_residual"] > 1e-7
        ):
            raise ValueError("rejected geometry/solver cannot become training data")
        with np.load(meshroot / "mesh.npz", allow_pickle=False) as d:
            nodes, elements = d["nodes"], d["elements"]
        with np.load(solve / "modes.npz", allow_pickle=False) as d:
            center, length = d["center"], float(d["length"])
            # Vector nodal DOFs precede P2 edge DOFs. Check against saved ports.
            field = d["eigenvectors"][: 3 * len(nodes), 6:].reshape(len(nodes), 3, 32)
            check = np.einsum(
                "pcm,pc->pm", field[d["contact_node_ids"]], d["contact_directions"]
            ) / np.sqrt(2700 * length**3)
            np.testing.assert_allclose(check, d["ports"], rtol=1e-12, atol=1e-15)
            omega = np.sqrt(d["eigenvalues"][6:])
        _, boundary, normals = elastic.boundary(nodes, elements)
        xyz = (nodes - center) / length
        sample = boundary[farthest(xyz[boundary], 512)]
        query = sample[:48]
        # Fixed observation belongs to TRAIN query points, not a target-audio choice.
        probe = query[0]
        features = np.r_[np.log(np.ptp(nodes, axis=0) / length), 0.19].astype(
            np.float32
        )
        geometry = {
            "cloud": xyz[sample].astype(np.float32),
            "features": features,
            "points": xyz[query].astype(np.float32),
            "normals": normals[query],
            "length": length,
            "probe": 0,
        }
        targets = {"omega": omega, "field": field[query]}
        name = f"object-{identity}"
        np.savez(args.output / f"{name}-geometry.npz", **geometry)
        np.savez(args.output / f"{name}-target.npz", **targets)
        role = "train" if identity in TRAIN else "development"
        if role == "train":
            # The fitting file contains no held-contact labels (last16 points).
            np.savez(
                args.output / f"{name}-train.npz",
                cloud=geometry["cloud"],
                features=features,
                points=geometry["points"][:32],
                omega=omega,
                field=field[query[:32]],
            )
        row = {
            "object_id": identity,
            "role": role,
            "mesh_sha256": meta["mesh_sha256"],
            "modes_sha256": source.sha(solve / "modes.npz"),
            "point_node_ids": query.tolist(),
            "probe_node_id": int(probe),
        }
        for kind in ("geometry", "target") + (("train",) if role == "train" else ()):
            row[kind + "_file"] = f"{name}-{kind}.npz"
            row[kind + "_sha256"] = source.sha(args.output / row[kind + "_file"])
        rows.append(row)
    write(
        args.output / "data.json",
        {
            "rows": rows,
            "scope": "three TRAIN shapes, last16 contacts withheld; DEV88 whole body, already open development not pristine holdout",
        },
    )
    write(
        args.output / "train.json", {"rows": [r for r in rows if r["role"] == "train"]}
    )


def gram(field):
    # B,P,3,M -> B,M,Q,Q, Q enumerates point/axis physical ports.
    f = field.flatten(1, 2).transpose(1, 2)
    return f[..., :, None] * f[..., None, :]


class PhysicalStudent(nn.Module):
    def __init__(self):
        super().__init__()
        self.point = nn.Sequential(
            nn.Linear(3, 32), nn.SiLU(), nn.Linear(32, 64), nn.SiLU()
        )
        self.body = nn.Sequential(
            nn.Linear(132, 128), nn.SiLU(), nn.Linear(128, 64), nn.SiLU()
        )
        self.frequency = nn.Sequential(nn.Linear(64, 64), nn.SiLU(), nn.Linear(64, 32))
        self.field = nn.Sequential(
            nn.Linear(91, 128),
            nn.SiLU(),
            nn.Linear(128, 128),
            nn.SiLU(),
            nn.Linear(128, 96),
        )
        self.register_buffer("log_mean", torch.zeros(32))
        self.register_buffer("log_std", torch.ones(32))
        self.register_buffer("field_scale", torch.ones(32))

    def forward(self, cloud, features, points):
        h = self.point(cloud)
        encoded = self.body(torch.cat([h.mean(1), h.amax(1), features], -1))
        omega = (self.frequency(encoded) * self.log_std + self.log_mean).exp()
        query = torch.cat(
            [
                encoded[:, None].expand(-1, points.shape[1], -1),
                legacy.fourier(points, 4),
            ],
            -1,
        )
        field = (
            self.field(query).reshape(points.shape[0], points.shape[1], 3, 32)
            * self.field_scale
        )
        return omega, field


def fit(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 3
        or {r["object_id"] for r in rows} != TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact three TRAIN bodies required")
    loaded = []
    for row in rows:
        path = args.data / row["train_file"]
        if source.sha(path) != row["train_sha256"]:
            raise ValueError("TRAIN data changed")
        with np.load(path, allow_pickle=False) as d:
            loaded.append(dict(d))
    cloud, features, points, omega, fields = [
        torch.tensor(np.array([d[k] for d in loaded]), dtype=torch.float32)
        for k in ("cloud", "features", "points", "omega", "field")
    ]
    torch.manual_seed(42)
    model = PhysicalStudent()
    model.log_mean.copy_(omega.log().mean(0))
    model.log_std.copy_(omega.log().std(0).clamp_min(1e-3))
    model.field_scale.copy_(fields.square().mean((0, 1, 2)).sqrt())
    if not (model.field_scale > 0).all():
        raise ValueError("zero mode normalization")
    target = gram(fields / model.field_scale)
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    losses = []
    for step in range(2000):
        optimizer.zero_grad(set_to_none=True)
        w, f = model(cloud, features, points)
        frequency_loss = ((w.log() - omega.log()) / model.log_std).square().mean()
        field_loss = (gram(f / model.field_scale) - target).square().mean()
        loss = frequency_loss + field_loss
        if not torch.isfinite(loss):
            raise ValueError("nonfinite loss")
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1.0, error_if_nonfinite=True)
        optimizer.step()
        if step == 0 or (step + 1) % 250 == 0:
            losses.append(
                {
                    "step": step + 1,
                    "frequency": float(frequency_loss.detach()),
                    "field": float(field_loss.detach()),
                }
            )
            print(losses[-1], flush=True)
    args.output.mkdir(parents=True)
    save_file(model.state_dict(), args.output / "model.safetensors")
    write(
        args.output / "fit.json",
        {
            "weights_sha256": source.sha(args.output / "model.safetensors"),
            "train_ids": sorted(TRAIN),
            "steps": 2000,
            "seed": 42,
            "lr": 0.001,
            "parameters": sum(p.numel() for p in model.parameters()),
            "losses": losses,
            "script_sha256": source.sha(Path(__file__)),
            "scope": "one shared ceramic physical-port model;32TRAIN contacts/body, no DEV body or held-contact labels; no pressure/general-material claim",
        },
    )


def physical_waves(omega, field, geometry, impulse=0.001):
    ports = np.einsum("pcm,pc->pm", field, geometry["normals"])
    # Inward force, outward displacement. Fixed outward observation is appended.
    ports = np.vstack([-ports, ports[int(geometry["probe"])]]) / np.sqrt(
        2700 * float(geometry["length"]) ** 3
    )
    physical = omega * np.sqrt(7.2e10 / 2700) / float(geometry["length"])
    return elastic.port_response(physical, ports, impulse)[0]


def render(args):
    receipt = json.loads((args.fit / "fit.json").read_text())
    if source.sha(args.fit / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("weights changed")
    model = PhysicalStudent().eval().requires_grad_(False)
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    rows = json.loads((args.data / "data.json").read_text())["rows"]
    args.output.mkdir(parents=True)
    outputs = []
    for row in rows:
        path = args.data / row["geometry_file"]
        if source.sha(path) != row["geometry_sha256"]:
            raise ValueError("geometry changed")
        with np.load(path, allow_pickle=False) as d:
            g = dict(d)
        if set(g) != {"cloud", "features", "points", "normals", "length", "probe"}:
            raise ValueError("non-geometric generation inputs")
        with torch.no_grad():
            omega, field = model(
                *[
                    torch.tensor(g[k][None], dtype=torch.float32)
                    for k in ("cloud", "features", "points")
                ]
            )
        omega, field = omega[0].numpy().astype(float), field[0].numpy().astype(float)
        if (
            not np.isfinite(omega).all()
            or not np.isfinite(field).all()
            or np.any(omega <= 0)
        ):
            raise ValueError("invalid neural parameters")
        file = f"object-{row['object_id']}.npz"
        np.savez(args.output / file, omega=omega, field=field)
        outputs.append(
            {
                "object_id": row["object_id"],
                "file": file,
                "sha256": source.sha(args.output / file),
            }
        )
    write(
        args.output / "render.json",
        {
            "rows": outputs,
            "weights_sha256": receipt["weights_sha256"],
            "scope": "geometry/own weights only; source-free parameters, audition handled by separate comparison",
        },
    )


def assess(args):
    rows = json.loads((args.data / "data.json").read_text())["rows"]
    generated = {
        r["object_id"]: r
        for r in json.loads((args.generated / "render.json").read_text())["rows"]
    }
    geometries, targets = {}, {}
    for row in rows:
        identity = row["object_id"]
        for kind, dest in (("geometry", geometries), ("target", targets)):
            path = args.data / row[kind + "_file"]
            if source.sha(path) != row[kind + "_sha256"]:
                raise ValueError("assessment input changed")
            with np.load(path, allow_pickle=False) as d:
                dest[identity] = dict(d)
    args.output.mkdir(parents=True)
    results = []
    for row in rows:
        identity = row["object_id"]
        g, t = geometries[identity], targets[identity]
        path = args.generated / generated[identity]["file"]
        if source.sha(path) != generated[identity]["sha256"]:
            raise ValueError("neural parameters changed")
        with np.load(path, allow_pickle=False) as d:
            prediction = dict(d)
        distances = {}
        for other in sorted(TRAIN):
            pair = cdist(g["cloud"], geometries[other]["cloud"], "sqeuclidean")
            distances[other] = float(pair.min(0).mean() + pair.min(1).mean())
        neighbor = min(distances, key=distances.get)
        nearest_points = cdist(g["points"], geometries[neighbor]["points"][:32]).argmin(
            1
        )
        # Baseline uses only TRAIN's first32 labels, never held-contact fields.
        reference = physical_waves(t["omega"], t["field"], g)
        neural = physical_waves(prediction["omega"], prediction["field"], g)
        baseline = physical_waves(
            targets[neighbor]["omega"], targets[neighbor]["field"][nearest_points], g
        )
        indices = range(32, 48) if identity in TRAIN else range(48)
        selected = (32, 40, 47) if identity in TRAIN else (0, 24, 47)
        metrics = []
        neural_gallery = []
        for i in indices:
            variants = [reference[i], neural[i], baseline[i]]
            peak = max(abs(w).max() for w in variants)
            if not np.isfinite(peak) or peak <= 0:
                raise ValueError("invalid audition signal")
            gain = 0.5 / peak
            metric = {
                name: audio_metrics(reference[i] * gain, w * gain)
                for name, w in (("neural", neural[i]), ("nearest", baseline[i]))
            }
            metrics.append(
                {"contact": i, "common_gain": float(gain), "metrics": metric}
            )
            if i in selected:
                stem = f"object-{identity}-contact-{i}"
                for name, w in zip(
                    ("reference", "neural", "nearest"), variants, strict=True
                ):
                    wavfile.write(
                        args.output / f"{stem}-{name}.wav",
                        source.RATE,
                        (w * gain).astype(np.float32),
                    )
                wavfile.write(
                    args.output / f"{stem}-comparison.wav",
                    source.RATE,
                    (np.concatenate(variants) * gain).astype(np.float32),
                )
                neural_gallery.append((neural[i] * gain).astype(np.float32))
        # Each snippet retains its documented common triple gain, not a
        # separately normalized neural level. See per-contact gain in report.
        wavfile.write(
            args.output / f"object-{identity}-three-neural.wav",
            source.RATE,
            np.concatenate(neural_gallery),
        )
        mean = {
            name: {
                key: float(np.mean([r["metrics"][name][key] for r in metrics]))
                for key in ("spectrum", "envelope", "level")
            }
            for name in ("neural", "nearest")
        }
        result = {
            "object_id": identity,
            "role": row["role"],
            "nearest_train": neighbor,
            "mean": mean,
            "contacts": metrics,
            "neural_frequency_mean_relative_error": float(
                np.mean(abs(prediction["omega"] / t["omega"] - 1))
            ),
            "selected_contacts": list(selected),
            "order": ["reference", "neural", "nearest"],
        }
        results.append(result)
        print({k: v for k, v in result.items() if k != "contacts"}, flush=True)
    write(
        args.output / "assessment.json",
        {
            "rows": results,
            "scope": "3x16 withheld TRAIN contacts and48contacts on ONE open DEV body88, all ceramic; target/NN/nearest same gain per triple; not48independentobjects, pristineholdout or realism; no checkpoint selection",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("prepare", "fit", "render", "assess"))
    for name in ("root", "data", "fit", "generated"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    torch.set_num_threads(4)
    {"prepare": prepare, "fit": fit, "render": render, "assess": assess}[args.stage](
        args
    )
