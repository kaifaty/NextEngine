"""One oracle-motivated magnitude factorization, frozen core/poles/sign field.

Not sign canonicalization, physical port reconstruction or radiation. Positive
gain magnitudes are learned on the same11TRAIN; old neural signs stay unchanged.
No target acoustics at inference. Added capacity is reported, not hidden as a
pure loss comparison. Old and new DEV must both be checked before any claim.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_expanded_eval as expanded
import physical_sound_objectfolder2_shared as shared
import torch
from physical_sound_objectfolder2_expanded_train import ALL_TRAIN
from safetensors.torch import load_file, save_file
from torch import nn
from torch.nn import functional as F

SOURCE = "81cc9a90a9da5c1a4cf5b2f82823fa267aaba20a7c75896fa9feb56fecad1fdb"


class MagnitudeStudent(nn.Module):
    def __init__(self):
        super().__init__()
        self.core = shared.SharedStudent()
        self.core.requires_grad_(False)
        self.magnitude = nn.Sequential(
            nn.Linear(108, 128),
            nn.SiLU(),
            nn.Linear(128, 128),
            nn.SiLU(),
            nn.Linear(128, 3),
        )

    def positive_field(self, encoded, rank, contacts):
        return F.softplus(
            self.magnitude(
                torch.cat(
                    [encoded, shared.fourier(rank, 8), shared.fourier(contacts, 4)],
                    dim=-1,
                )
            )
        )


def predict(model, cloud, features, contacts):
    original = shared.predict(model.core, cloud, features, contacts)
    count = len(original["frequency"])
    rank = ((torch.arange(count) + 0.5) / count)[:, None]
    with torch.no_grad():
        encoded = model.core.encode(
            torch.tensor(cloud[None]), torch.tensor(features[None])
        ).expand(count, -1)
        magnitude = np.array(
            [
                (
                    model.positive_field(
                        encoded, rank, torch.tensor(p[None]).expand(count, -1)
                    ).sinh()
                    * model.core.gain_scale
                )
                .numpy()
                .T
                for p in contacts
            ]
        ).astype(float)
    if not np.isfinite(magnitude).all():
        raise ValueError("nonfinite predicted magnitude")
    return {
        **original,
        "original_gains": original["gains"],
        "gains": magnitude * np.sign(original["gains"]),
    }


def fit(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 11
        or {r["object_id"] for r in rows} != ALL_TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact eleven TRAIN objects required")
    if teacher.sha(args.source / "model.safetensors") != SOURCE:
        raise ValueError("fixed eleven-body core required")
    torch.manual_seed(42)
    model = MagnitudeStudent()
    source = load_file(args.source / "model.safetensors")
    model.core.load_state_dict(source, strict=True)
    model.core.eval()
    model.magnitude.load_state_dict(model.core.field.state_dict(), strict=True)
    data = [expanded.read_npz(args.data / r["file"], r["sha256"]) for r in rows]
    counts = torch.tensor([len(d["frequency"]) for d in data])
    points = torch.tensor(np.array([d["contacts"] for d in data]))
    with torch.no_grad():
        encoded = model.core.encode(
            torch.tensor(np.array([d["cloud"] for d in data])),
            torch.tensor(np.array([d["features"] for d in data])),
        )
    targets = torch.zeros(11, 32, int(counts.max()), 3)
    for i, d in enumerate(data):
        targets[i, :, : counts[i]] = torch.asinh(
            torch.tensor(abs(d["gains"].transpose(0, 2, 1))) / model.core.gain_scale
        )
    optimizer = torch.optim.Adam(model.magnitude.parameters(), lr=0.001)
    losses = []
    seen = torch.zeros(11, int(counts.max()), dtype=torch.bool)
    for step in range(2000):
        optimizer.zero_grad(set_to_none=True)
        body = torch.randint(11, (1024,))
        contact = torch.randint(32, (1024,))
        mode = (torch.rand(1024) * counts[body]).long()
        seen[body, mode] = True
        rank = ((mode.float() + 0.5) / counts[body])[:, None]
        value = model.positive_field(encoded[body], rank, points[body, contact])
        loss = (value - targets[body, contact, mode]).square().mean()
        if not torch.isfinite(loss):
            raise ValueError("nonfinite magnitude loss")
        loss.backward()
        nn.utils.clip_grad_norm_(
            model.magnitude.parameters(), 5, error_if_nonfinite=True
        )
        optimizer.step()
        if step == 0 or (step + 1) % 250 == 0:
            row = {"step": step + 1, "magnitude_loss": float(loss.detach())}
            losses.append(row)
            print(row, flush=True)
    for key, value in model.core.state_dict().items():
        if not torch.equal(value, source[key]):
            raise ValueError("frozen core changed")
    args.output.mkdir(parents=True)
    save_file(model.state_dict(), args.output / "model.safetensors")
    shared.write_json(
        args.output / "fit.json",
        {
            "steps": 2000,
            "seed": 42,
            "lr": 0.001,
            "train_ids": [r["object_id"] for r in rows],
            "parameters": sum(p.numel() for p in model.parameters()),
            "trainable_parameters": sum(
                p.numel() for p in model.parameters() if p.requires_grad
            ),
            "core_unchanged_exact": True,
            "all_train_modes_sampled": all(
                bool(seen[i, :n].all()) for i, n in enumerate(counts)
            ),
            "source_weights_sha256": SOURCE,
            "weights_sha256": teacher.sha(args.output / "model.safetensors"),
            "script_sha256": teacher.sha(Path(__file__)),
            "losses": losses,
            "scope": "one positive magnitude head, core encoder/poles/count/signs frozen; additional capacity AND supervision change, not isolated loss proof; no physical gauge/radiation claim",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("fit", "render-old", "render-new"))
    for name in ("data", "source", "fit", "output"):
        parser.add_argument("--" + name, type=Path, required=name in ("data", "output"))
    parser.add_argument("--weights-sha256")
    args = parser.parse_args()
    torch.set_num_threads(4)
    if args.stage == "fit":
        fit(args)
    elif args.stage == "render-old":
        shared.render(args, model_class=MagnitudeStudent, prediction_fn=predict)
    else:
        expanded.render(args, model_class=MagnitudeStudent, prediction_fn=predict)
