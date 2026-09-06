"""Shared modal port field with rank-one positive-semidefinite residues.

Existing frequencies stay frozen. One field evaluated at all points gives
R_i(p,q)=a_i(p)*a_i(q): reciprocity, collocation and passivity by construction.
This does not establish fidelity, pressure radiation, or arbitrary-shape quality.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_modal3d_impact as impact
import physical_sound_modal3d_pilot as pilot
import torch
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from torch import nn


def residues(amplitudes):
    return torch.einsum("bpm,bqm->bmpq", amplitudes, amplitudes)


class PassiveModes(nn.Module):
    def __init__(self):
        super().__init__()
        original = pilot.SharedModes()
        self.body = original.body
        self.field = original.contact
        self.body.requires_grad_(False)
        self.register_buffer("log_mean", torch.zeros(8))
        self.register_buffer("log_std", torch.ones(8))
        self.register_buffer("field_scale", torch.ones(8))

    def amplitudes(self, objects, points):
        batch, count, _ = points.shape
        inputs = torch.cat(
            [objects[:, None].expand(-1, count, -1), points * 2 - 1], dim=-1
        )
        return (
            self.field(inputs.reshape(batch * count, 5)).reshape(batch, count, 8)
            * self.field_scale
        )

    def forward(self, objects, points):
        omega = (self.body(objects) * self.log_std + self.log_mean).exp()
        return omega, self.amplitudes(objects, points)


def fit(args):
    data = json.loads((args.data / "train.json").read_text())
    rows = data["rows"]
    if (
        len(rows) != 36
        or any(r["role"] != "train" for r in rows)
        or {tuple(r["shape"]) for r in rows} != set(pilot.TRAIN_SHAPES)
    ):
        raise ValueError("exact TRAIN-only family required")
    objects, points, targets = [], [], []
    for row in rows:
        path = args.data / row["file"]
        if pilot.integrity.sha256(path) != row["sha256"]:
            raise ValueError("teacher changed")
        with np.load(path, allow_pickle=False) as d:
            objects.append(pilot.object_input(d["shape"]))
            points.append(d["contacts"])
            # All nine unique contact points; the fixed probe already occurs
            # in this grid, so do not weight it again via the duplicate last row.
            targets.append(d["port_modes"][:-1])
    objects, points, targets = [
        torch.tensor(np.array(x), dtype=torch.float32)
        for x in (objects, points, targets)
    ]
    source = json.loads((args.source / "fit.json").read_text())
    path = args.source / "model.safetensors"
    if pilot.integrity.sha256(path) != source["weights_sha256"]:
        raise ValueError("source weights changed")
    saved = load_file(path)
    torch.manual_seed(42)
    model = PassiveModes()
    model.body.load_state_dict(
        {k.removeprefix("body."): v for k, v in saved.items() if k.startswith("body.")},
        strict=True,
    )
    model.log_mean.copy_(saved["log_mean"])
    model.log_std.copy_(saved["log_std"])
    model.field_scale.copy_(targets.square().mean((0, 1)).sqrt().clamp_min(1e-12))
    target_matrices = residues(targets / model.field_scale)
    optimizer = torch.optim.Adam(model.field.parameters(), lr=0.001)
    losses = []
    for step in range(1500):
        optimizer.zero_grad(set_to_none=True)
        a = model.amplitudes(objects, points) / model.field_scale
        loss = (residues(a) - target_matrices).square().mean()
        if not torch.isfinite(loss):
            raise ValueError("nonfinite passive fit")
        loss.backward()
        nn.utils.clip_grad_norm_(model.field.parameters(), 1.0, error_if_nonfinite=True)
        optimizer.step()
        if step == 0 or (step + 1) % 250 == 0:
            losses.append({"step": step + 1, "matrix_loss": float(loss.detach())})
            print("passive fit", losses[-1], flush=True)
    for key, value in model.body.state_dict().items():
        if not torch.equal(value, saved["body." + key]):
            raise ValueError("frequency head changed")
    args.output.mkdir(parents=True)
    save_file(model.state_dict(), args.output / "model.safetensors")
    pilot.storage.save(
        args.output / "fit.json",
        {
            "steps": 1500,
            "seed": 42,
            "lr": 0.001,
            "parameters": sum(p.numel() for p in model.parameters()),
            "trainable_parameters": sum(
                p.numel() for p in model.parameters() if p.requires_grad
            ),
            "weights_sha256": pilot.integrity.sha256(args.output / "model.safetensors"),
            "source_weights_sha256": source["weights_sha256"],
            "train_sha256": pilot.integrity.sha256(args.data / "train.json"),
            "losses": losses,
            "script_sha256": pilot.integrity.sha256(Path(__file__)),
            "scope": "one final shared field fit to36x9x9 port matrices; frequency head frozen EXACT; no DEV reads/checkpoint selection; algebraic passivity is not accuracy",
        },
    )


def load_model(root):
    receipt = json.loads((root / "fit.json").read_text())
    if pilot.integrity.sha256(root / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("weights changed")
    model = PassiveModes().eval()
    model.load_state_dict(load_file(root / "model.safetensors"), strict=True)
    return model


def query(model, shape, contact):
    with torch.no_grad():
        omega, a = model(
            torch.tensor(pilot.object_input(shape)[None], dtype=torch.float32),
            torch.tensor([[list(contact), [1.0, 0.5]]], dtype=torch.float32),
        )
    w, a = omega[0].numpy().astype(float), a[0].numpy().astype(float)
    return w, a


def physical(omega, amplitudes, config):
    scale = 2230 * 0.18**3
    return (
        pilot.physical_modes(omega, np.ones(8), 0.18, config["target_young"], 2230)[0],
        amplitudes[0] * amplitudes[1] / scale,
        amplitudes[0] ** 2 / scale,
    )


def render(args):
    model = load_model(args.fit)
    args.output.mkdir(parents=True)
    scenarios = [
        {
            "name": f"case-{i:02d}-contact-{c}",
            "shape_index": i,
            "contact_index": c,
            "shape": shape,
            "contact": point.tolist(),
            "config": {**impact.BASE, "target_poisson": shape[2]},
        }
        for i, shape in enumerate(pilot.DEV_SHAPES)
        for c, point in enumerate(pilot.DEV_CONTACTS)
    ]
    scenarios += [
        {
            "name": "impact-" + name,
            "shape_index": 0,
            "contact_index": 0,
            "shape": impact.SHAPE,
            "contact": impact.CONTACT.tolist(),
            "config": {**impact.BASE, **changes, "target_poisson": impact.SHAPE[2]},
        }
        for name, changes in impact.CASES
    ]
    waves = {}
    for row in scenarios:
        omega, a = query(model, row["shape"], row["contact"])
        wp, g, sg = physical(omega, a, row["config"])
        contact = impact.collision(row["config"], wp, sg)
        wave = impact.render_force(wp, g, contact)
        np.savez(args.output / (row["name"] + ".npz"), omega=omega, amplitudes=a)
        waves[row["name"]] = wave
        row["collision"] = impact.summary(contact)
    gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in waves.values()))
    for row in scenarios:
        path = args.output / (row["name"] + ".wav")
        wavfile.write(path, pilot.RATE, waves[row["name"]] * gain)
        row["wav_sha256"] = pilot.integrity.sha256(path)
    for title, names in (
        ("striker-stiffness", ("striker-E2GPa", "base", "striker-E200GPa")),
        ("impact-speed", ("speed-0p1", "base", "speed-1p0")),
        ("striker-radius", ("radius-1mm", "base", "radius-5mm")),
        ("target-stiffness", ("target-E16GPa", "base", "target-E200GPa")),
    ):
        wavfile.write(
            args.output / (title + ".wav"),
            pilot.RATE,
            np.concatenate(
                [
                    v
                    for name in names
                    for v in (waves["impact-" + name] * gain, np.zeros(pilot.RATE // 2))
                ]
            ).astype(np.float32),
        )
    pilot.storage.save(
        args.output / "render.json",
        {
            "rows": scenarios,
            "gain": gain,
            "fit_sha256": pilot.integrity.sha256(args.fit / "fit.json"),
            "script_sha256": pilot.integrity.sha256(Path(__file__)),
            "scope": "59 coupled neural impacts,4galleries; all coefficients from shared weights, no target recording/FEM; first separation, fixed probe velocity, no real material/radiation claim",
        },
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("fit", "render"))
    for name in ("data", "source", "fit", "output"):
        parser.add_argument("--" + name, type=Path, required=name == "output")
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    if (
        args.stage == "fit"
        and (args.data is None or args.source is None)
        or args.stage == "render"
        and args.fit is None
    ):
        parser.error("missing stage inputs")
    torch.set_num_threads(4)
    {"fit": fit, "render": render}[args.stage](args)


if __name__ == "__main__":
    main()
