"""One shared variable-mode student over the prepared nine OF2 objects.

Six TRAIN, three open DEV; no target acoustic parameters/audio at generation.
Point-cloud/size/material encoder, predicted mode count, rank-conditioned poles
and signed excitation gains. This is synthetic vibration, not pressure/realism.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import torch
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from torch import nn

TRAIN = {7, 23, 29, 66, 75, 82}
DEV = {11, 54, 88}
MATERIALS = ("Wood", "Steel", "Ceramic", "Polycarbonate", "Plastic")
POINTS = 32


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def geometry(vertices, material):
    lower, upper = vertices.min(0), vertices.max(0)
    dimensions = upper - lower
    if not np.isfinite(vertices).all() or np.any(dimensions <= 0):
        raise ValueError("finite volumetric geometry required")
    center = (lower + upper) / 2
    length = dimensions.max()
    ids = np.linspace(0, len(vertices) - 1, 512).astype(int)
    cloud = (vertices[ids] - center) / length
    features = np.r_[np.log(dimensions), [material == m for m in MATERIALS]]
    if material not in MATERIALS:
        raise ValueError("untrained material vocabulary")
    return cloud.astype(np.float32), features.astype(np.float32), center, length


def prepare(args):
    cohort = json.loads(args.cohort.read_text())
    if (
        len(cohort["rows"]) != 9
        or {r["object_id"] for r in cohort["rows"]} != TRAIN | DEV
    ):
        raise ValueError("fixed nine-object cohort required")
    args.output.mkdir(parents=True)
    definitions = teacher.source_definitions(args.source / "AudioNet_model.py")
    records, clouds, features, contacts = [], [], [], []
    for row in cohort["rows"]:
        identity = row["object_id"]
        mesh = Path(row["mesh"])
        if teacher.sha(mesh) != row["mesh_sha256"]:
            raise ValueError("mesh changed")
        vertices = np.array(
            [
                [float(x) for x in line.split()[1:]]
                for line in mesh.read_text().splitlines()
                if line.startswith("v ")
            ]
        )
        audio = teacher.load_audio(
            Path(row["source_model"]), row["source_model_sha256"]
        )
        if np.any((audio["frequencies"] < 1) | (audio["frequencies"] > 22049)):
            raise ValueError("source outside common frequency head")
        norm = audio["normalizer"]
        # Restrict new queries to the published support, without clamping,
        # dropping an object or selecting by sound. No protected contacts here.
        support = np.flatnonzero(
            np.all(
                (vertices >= norm["xyz_min"]) & (vertices <= norm["xyz_max"]), axis=1
            )
        )
        if len(support) < POINTS:
            raise ValueError("insufficient in-support mesh vertices")
        ids = support[np.linspace(0, len(support) - 1, POINTS).astype(int)]
        points = vertices[ids]
        gains = teacher.point_gains(audio, definitions, points)
        cloud, feature, center, length = geometry(vertices, row["assigned_material"])
        normalized = ((points - center) / length).astype(np.float32)
        name = f"object-{identity}.npz"
        payload = {
            "cloud": cloud,
            "features": feature,
            "contacts": normalized,
            "frequency": audio["frequencies"],
            "damping": audio["dampings"],
            "gains": gains,
        }
        np.savez(args.output / name, **payload)
        with np.load(args.output / name, allow_pickle=False) as saved:
            if any(not np.array_equal(v, saved[k]) for k, v in payload.items()):
                raise ValueError("variable-mode packing lost data")
        reference = teacher.waveform(
            gains[0], audio["frequencies"], audio["dampings"], [1, 1, 1]
        )
        # Teacher audition uses its prior per-object common gain, never a target
        # supplied to the student or an output normalization learned by it.
        previous = json.loads((Path(row["teacher_render"]) / "result.json").read_text())
        gain = previous["common_pcm_gain"]
        wavfile.write(
            args.output / f"object-{identity}-full-teacher.wav",
            teacher.RATE,
            (reference * gain).astype(np.float32),
        )
        record = {
            "object_id": identity,
            "role": "train" if identity in TRAIN else "development",
            "file": name,
            "sha256": teacher.sha(args.output / name),
            "modes": len(audio["frequencies"]),
            "mesh_sha256": row["mesh_sha256"],
            "source_model_sha256": row["source_model_sha256"],
            "query_vertex_indices": ids.tolist(),
            "out_of_support_vertices": len(vertices) - len(support),
            "material": row["assigned_material"],
            "length": float(length),
            "audition_gain": gain,
        }
        records.append(record)
        clouds.append(cloud)
        features.append(feature)
        contacts.append(normalized)
        print(
            "packed",
            identity,
            record["modes"],
            "modes; excluded geometric vertices",
            record["out_of_support_vertices"],
            flush=True,
        )
    write_json(
        args.output / "data.json", {"rows": records, "all_modes_preserved": True}
    )
    write_json(
        args.output / "train.json",
        {"rows": [r for r in records if r["role"] == "train"]},
    )
    np.savez(
        args.output / "inputs.npz",
        cloud=np.array(clouds),
        features=np.array(features),
        contacts=np.array(contacts),
    )
    write_json(
        args.output / "inputs.json",
        {
            "rows": [{"object_id": r["object_id"], "role": r["role"]} for r in records],
            "sha256": teacher.sha(args.output / "inputs.npz"),
        },
    )


def fourier(x, bands):
    scales = 2.0 ** torch.arange(bands, device=x.device, dtype=x.dtype)
    phase = x[..., None] * scales * math.pi
    return torch.cat([x, phase.sin().flatten(-2), phase.cos().flatten(-2)], dim=-1)


class SharedStudent(nn.Module):
    def __init__(self):
        super().__init__()
        self.point = nn.Sequential(
            nn.Linear(3, 32), nn.SiLU(), nn.Linear(32, 64), nn.SiLU()
        )
        self.body = nn.Sequential(
            nn.Linear(136, 128), nn.SiLU(), nn.Linear(128, 64), nn.SiLU()
        )
        self.count = nn.Linear(64, 1)
        self.poles = nn.Sequential(
            nn.Linear(81, 64), nn.SiLU(), nn.Linear(64, 64), nn.SiLU(), nn.Linear(64, 2)
        )
        self.field = nn.Sequential(
            nn.Linear(108, 128),
            nn.SiLU(),
            nn.Linear(128, 128),
            nn.SiLU(),
            nn.Linear(128, 3),
        )
        self.register_buffer("size_mean", torch.zeros(3))
        self.register_buffer("size_std", torch.ones(3))
        self.register_buffer("pole_mean", torch.zeros(2))
        self.register_buffer("pole_std", torch.ones(2))
        self.register_buffer("count_mean", torch.zeros(()))
        self.register_buffer("count_std", torch.ones(()))
        self.register_buffer("gain_scale", torch.ones(()))

    def encode(self, cloud, features):
        h = self.point(cloud)
        size_material = torch.cat(
            [(features[:, :3] - self.size_mean) / self.size_std, features[:, 3:]],
            dim=-1,
        )
        return self.body(torch.cat([h.mean(1), h.amax(1), size_material], dim=-1))

    def log_count(self, encoded):
        return self.count(encoded).squeeze(-1) * self.count_std + self.count_mean

    def mode_parameters(self, encoded, rank):
        x = torch.cat([encoded, fourier(rank, 8)], dim=-1)
        out = self.poles(x)
        # All nine source bands lie strictly inside [1,22049] Hz. Bounded
        # log-frequency parameterization avoids aliasing; no mode is discarded.
        log_frequency = out[:, 0].sigmoid() * math.log(22049)
        log_damping = out[:, 1] * self.pole_std[1] + self.pole_mean[1]
        return torch.stack([log_frequency, log_damping], dim=-1)

    def signed_field(self, encoded, rank, contacts):
        return self.field(
            torch.cat([encoded, fourier(rank, 8), fourier(contacts, 4)], dim=-1)
        )


def fit(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 6
        or {r["object_id"] for r in rows} != TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact TRAIN-only projection required")
    data = []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("teacher changed")
        with np.load(path, allow_pickle=False) as saved:
            data.append(dict(saved))
    counts = torch.tensor([len(d["frequency"]) for d in data])
    capacity = int(counts.max())
    cloud = torch.tensor(np.array([d["cloud"] for d in data]))
    features = torch.tensor(np.array([d["features"] for d in data]))
    points = torch.tensor(np.array([d["contacts"] for d in data]))
    poles = torch.zeros(6, capacity, 2)
    gains = torch.zeros(6, POINTS, capacity, 3)
    for i, d in enumerate(data):
        poles[i, : counts[i]] = torch.tensor(
            np.log(np.stack([d["frequency"], d["damping"]], axis=-1)),
            dtype=torch.float32,
        )
        gains[i, :, : counts[i]] = torch.tensor(d["gains"].transpose(0, 2, 1))
    torch.manual_seed(42)
    model = SharedStudent()
    model.size_mean.copy_(features[:, :3].mean(0))
    model.size_std.copy_(features[:, :3].std(0).clamp_min(1e-6))
    means = torch.stack([poles[i, :n].mean(0) for i, n in enumerate(counts)])
    model.pole_mean.copy_(means.mean(0))
    model.pole_std.copy_(
        torch.cat([poles[i, :n] for i, n in enumerate(counts)]).std(0).clamp_min(1e-6)
    )
    model.count_mean.copy_(counts.float().log().mean())
    model.count_std.copy_(counts.float().log().std())
    model.gain_scale.copy_(
        torch.stack(
            [gains[i, :, :n].square().mean().sqrt() for i, n in enumerate(counts)]
        )
        .median()
        .clamp_min(1e-8)
    )
    targets = torch.asinh(gains / model.gain_scale)
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    seen = torch.zeros(6, capacity, dtype=torch.bool)
    losses = []
    for step in range(2000):
        optimizer.zero_grad(set_to_none=True)
        encoded = model.encode(cloud, features)
        body_id = torch.randint(6, (1024,))
        contact_id = torch.randint(POINTS, (1024,))
        mode_id = (torch.rand(1024) * counts[body_id]).long()
        seen[body_id, mode_id] = True
        rank = ((mode_id.float() + 0.5) / counts[body_id])[:, None]
        p = model.mode_parameters(encoded[body_id], rank)
        g = model.signed_field(encoded[body_id], rank, points[body_id, contact_id])
        count_loss = (
            ((model.log_count(encoded) - counts.float().log()) / model.count_std)
            .square()
            .mean()
        )
        pole_loss = ((p - poles[body_id, mode_id]) / model.pole_std).square().mean()
        gain_loss = (g - targets[body_id, contact_id, mode_id]).square().mean()
        loss = count_loss + pole_loss + gain_loss
        if not torch.isfinite(loss):
            raise ValueError("nonfinite shared fit")
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 5, error_if_nonfinite=True)
        optimizer.step()
        if step == 0 or (step + 1) % 250 == 0:
            entry = {
                "step": step + 1,
                "count": float(count_loss.detach()),
                "poles": float(pole_loss.detach()),
                "gain": float(gain_loss.detach()),
            }
            losses.append(entry)
            print("shared fit", entry, flush=True)
    args.output.mkdir(parents=True)
    save_file(model.state_dict(), args.output / "model.safetensors")
    write_json(
        args.output / "fit.json",
        {
            "steps": 2000,
            "seed": 42,
            "lr": 0.001,
            "parameters": sum(p.numel() for p in model.parameters()),
            "losses": losses,
            "train_ids": [r["object_id"] for r in rows],
            "all_train_modes_sampled": all(
                bool(seen[i, :n].all()) for i, n in enumerate(counts)
            ),
            "weights_sha256": teacher.sha(args.output / "model.safetensors"),
            "script_sha256": teacher.sha(Path(__file__)),
            "scope": "one fixed fit; no DEV reads/checkpoint selection, raw synthetic signed gains, not pressure",
        },
    )


def predict(model, cloud, features, contacts):
    with torch.no_grad():
        encoded = model.encode(torch.tensor(cloud[None]), torch.tensor(features[None]))
        estimate = float(model.log_count(encoded).exp()[0])
        if not np.isfinite(estimate) or not 1 <= estimate <= 4096:
            raise ValueError("predicted mode count outside bounded renderer")
        count = round(estimate)
        rank = ((torch.arange(count) + 0.5) / count)[:, None]
        poles = model.mode_parameters(encoded.expand(count, -1), rank).exp().numpy()
        gains = []
        for contact in contacts:
            value = model.signed_field(
                encoded.expand(count, -1),
                rank,
                torch.tensor(contact[None]).expand(count, -1),
            )
            gains.append((value.sinh() * model.gain_scale).numpy().T)
    result = {
        "frequency": poles[:, 0].astype(float),
        "damping": poles[:, 1].astype(float),
        "gains": np.array(gains).astype(float),
    }
    if not all(np.isfinite(v).all() for v in result.values()) or np.any(
        result["damping"] <= 0
    ):
        raise ValueError("invalid predicted coefficients")
    return result


def render(args, *, model_class=SharedStudent, prediction_fn=predict):
    info = json.loads((args.fit / "fit.json").read_text())
    if teacher.sha(args.fit / "model.safetensors") != info["weights_sha256"]:
        raise ValueError("weights changed")
    model = model_class()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    model.eval().requires_grad_(False)
    inputs = json.loads((args.data / "inputs.json").read_text())
    if teacher.sha(args.data / "inputs.npz") != inputs["sha256"]:
        raise ValueError("geometry inputs changed")
    with np.load(args.data / "inputs.npz", allow_pickle=False) as d:
        if set(d.files) != {"cloud", "features", "contacts"}:
            raise ValueError("acoustic target in generation inputs")
        clouds, features, contacts = [
            d[k].copy() for k in ("cloud", "features", "contacts")
        ]
    if (
        len(inputs["rows"]) != 9
        or {r["object_id"] for r in inputs["rows"]} != TRAIN | DEV
    ):
        raise ValueError("exact nine-object inputs required")
    if (
        clouds.shape != (9, 512, 3)
        or features.shape != (9, 8)
        or contacts.shape != (9, POINTS, 3)
    ):
        raise ValueError("invalid geometry input shapes")
    if not all(np.isfinite(x).all() for x in (clouds, features, contacts)):
        raise ValueError("nonfinite geometry inputs")
    args.output.mkdir(parents=True)
    waves, rows = {}, []
    for row, cloud, feature, points in zip(
        inputs["rows"], clouds, features, contacts, strict=True
    ):
        identity = row["object_id"]
        predicted = prediction_fn(model, cloud, feature, points)
        np.savez(args.output / f"object-{identity}.npz", **predicted)
        for c in range(4):
            name = f"object-{identity}-contact-{c}.wav"
            waves[name] = teacher.waveform(
                predicted["gains"][c],
                predicted["frequency"],
                predicted["damping"],
                [1, 1, 1],
            )
            rows.append(
                {
                    **row,
                    "contact": c,
                    "file": name,
                    "modes": len(predicted["frequency"]),
                }
            )
        print(
            "standalone generated",
            identity,
            len(predicted["frequency"]),
            "modes",
            flush=True,
        )
    if not all(np.isfinite(y).all() for y in waves.values()):
        raise ValueError("nonfinite generated waveform")
    peak = max(float(abs(y).max()) for y in waves.values())
    if peak <= 0:
        raise ValueError("all generated responses silent")
    gain = min(1.0, 0.5 / peak)
    for row in rows:
        path = args.output / row["file"]
        wavfile.write(
            path, teacher.RATE, (waves[row["file"]] * gain).astype(np.float32)
        )
        row["sha256"] = teacher.sha(path)
    write_json(
        args.output / "render.json",
        {
            "rows": rows,
            "common_gain": gain,
            "fit_sha256": info["weights_sha256"],
            "scope": "geometry/material/points and shared weights ONLY; no target poles, count, gains, WAV, teacher model or FEM",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("prepare", "fit", "render"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cohort", type=Path)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--data", type=Path)
    parser.add_argument("--fit", type=Path)
    args = parser.parse_args()
    torch.set_num_threads(4)
    {"prepare": prepare, "fit": fit, "render": render}[args.stage](args)
