"""Shared fixed-frequency-channel student, not a per-object network.

Same compact targets and 2000-step recipe as the preceding rank-field student.
128 physical frequency bands x three slots, explicit occupancy, poles shared
across contacts. Full teacher audio remains an assessment-only target.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_shared as shared
import torch
from safetensors.torch import save_file
from torch import nn
from torch.nn import functional as F

BANDS = 128
SLOTS = 3 * BANDS


def band_edges():
    result = (
        np.expm1(np.linspace(np.log1p(1 / 700), np.log1p(22049 / 700), BANDS + 1)) * 700
    )
    result[0], result[-1] = 1, 22049
    return result


def channel_indices(row, data):
    indices, offset, previous = [], 0, -1
    for band in row["bands"]:
        b, count = band["band"], band["packed_modes"]
        if not previous < b < BANDS or not 1 <= count <= 3:
            raise ValueError("invalid compact band slots")
        previous = b
        indices.extend(3 * b + np.arange(count))
        frequencies = data["frequency"][offset : offset + count]
        if len(frequencies) != count or np.any(
            (frequencies < band_edges()[b]) | (frequencies > band_edges()[b + 1])
        ):
            raise ValueError("pole outside assigned frequency band")
        offset += count
    if offset != len(data["frequency"]) or len(set(indices)) != offset:
        raise ValueError("compact modes lost or duplicated")
    return np.array(indices, dtype=np.int64)


class ChannelStudent(nn.Module):
    def __init__(self):
        super().__init__()
        base = shared.SharedStudent()
        self.point, self.body = base.point, base.body
        self.poles = nn.Sequential(
            nn.Linear(64, 16), nn.SiLU(), nn.Linear(16, SLOTS * 3)
        )
        self.field = nn.Sequential(
            nn.Linear(91, 16), nn.SiLU(), nn.Linear(16, SLOTS * 3)
        )
        for name, value in (
            ("size_mean", torch.zeros(3)),
            ("size_std", torch.ones(3)),
            ("pole_mean", torch.zeros(2)),
            ("pole_std", torch.ones(2)),
            ("gain_scale", torch.ones(())),
        ):
            self.register_buffer(name, value)
        edges = torch.tensor(band_edges(), dtype=torch.float32)
        self.register_buffer("lower", edges[:-1].repeat_interleave(3))
        self.register_buffer("upper", edges[1:].repeat_interleave(3))

    encode = shared.SharedStudent.encode

    def mode_parameters(self, encoded):
        raw = self.poles(encoded).reshape(-1, SLOTS, 3)
        frequency = self.lower + raw[:, :, 0].sigmoid() * (self.upper - self.lower)
        log_damping = raw[:, :, 1] * self.pole_std[1] + self.pole_mean[1]
        return torch.stack([frequency.log(), log_damping], -1), raw[:, :, 2]

    def signed_field(self, encoded, contacts):
        x = torch.cat([encoded, shared.fourier(contacts, 4)], -1)
        return self.field(x).reshape(-1, SLOTS, 3)


def fit(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 6
        or {r["object_id"] for r in rows} != shared.TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact TRAIN-only projection required")
    data, slots = [], []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("compact teacher changed")
        with np.load(path, allow_pickle=False) as saved:
            item = dict(saved)
        if (
            not all(np.isfinite(v).all() for v in item.values())
            or item["gains"].shape != (32, 3, len(item["frequency"]))
            or np.any(item["damping"] <= 0)
        ):
            raise ValueError("invalid compact teacher")
        data.append(item)
        slots.append(channel_indices(row, item))
    cloud = torch.tensor(np.array([d["cloud"] for d in data]))
    features = torch.tensor(np.array([d["features"] for d in data]))
    points = torch.tensor(np.array([d["contacts"] for d in data]))
    counts = torch.tensor([len(s) for s in slots])
    indices = torch.zeros(6, int(counts.max()), dtype=torch.int64)
    target_poles = torch.zeros(6, SLOTS, 2)
    target_gains = torch.zeros(6, 32, SLOTS, 3)
    target_mask = torch.zeros(6, SLOTS)
    for i, (d, slot) in enumerate(zip(data, slots, strict=True)):
        indices[i, : len(slot)] = torch.tensor(slot)
        target_mask[i, slot] = 1
        target_poles[i, slot] = torch.tensor(
            np.log(np.stack([d["frequency"], d["damping"]], -1)), dtype=torch.float32
        )
        target_gains[i, :, slot] = torch.tensor(d["gains"].transpose(0, 2, 1))
    torch.manual_seed(42)
    model = ChannelStudent()
    model.size_mean.copy_(features[:, :3].mean(0))
    model.size_std.copy_(features[:, :3].std(0).clamp_min(1e-6))
    valid_poles = [target_poles[i, slot] for i, slot in enumerate(slots)]
    model.pole_mean.copy_(torch.stack([v.mean(0) for v in valid_poles]).mean(0))
    model.pole_std.copy_(torch.cat(valid_poles).std(0).clamp_min(1e-6))
    model.gain_scale.copy_(
        torch.stack([torch.tensor(d["gains"]).square().mean().sqrt() for d in data])
        .median()
        .clamp_min(1e-8)
    )
    targets = torch.asinh(target_gains / model.gain_scale)
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    seen = torch.zeros(6, SLOTS, dtype=torch.bool)
    losses = []
    for step in range(2000):
        optimizer.zero_grad(set_to_none=True)
        encoded = model.encode(cloud, features)
        body_id = torch.randint(6, (1024,))
        contact_id = torch.randint(32, (1024,))
        mode_id = (torch.rand(1024) * counts[body_id]).long()
        slot = indices[body_id, mode_id]
        seen[body_id, slot] = True
        poles, logits = model.mode_parameters(encoded)
        fields = model.signed_field(
            encoded[:, None].expand(-1, 32, -1).reshape(192, 64), points.reshape(192, 3)
        )
        occupancy_loss = F.binary_cross_entropy_with_logits(logits, target_mask)
        pole_loss = (
            ((poles[body_id, slot] - target_poles[body_id, slot]) / model.pole_std)
            .square()
            .mean()
        )
        gain_loss = (
            (
                fields[body_id * 32 + contact_id, slot]
                - targets[body_id, contact_id, slot]
            )
            .square()
            .mean()
        )
        loss = occupancy_loss + pole_loss + gain_loss
        if not torch.isfinite(loss):
            raise ValueError("nonfinite channel fit")
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 5, error_if_nonfinite=True)
        optimizer.step()
        if step == 0 or (step + 1) % 250 == 0:
            entry = {
                "step": step + 1,
                "occupancy": float(occupancy_loss.detach()),
                "poles": float(pole_loss.detach()),
                "gain": float(gain_loss.detach()),
            }
            losses.append(entry)
            print("channel fit", entry, flush=True)
    args.output.mkdir(parents=True)
    save_file(model.state_dict(), args.output / "model.safetensors")
    shared.write_json(
        args.output / "fit.json",
        {
            "steps": 2000,
            "seed": 42,
            "lr": 0.001,
            "batch_sampled_modes": 1024,
            "parameters": sum(p.numel() for p in model.parameters()),
            "rank_baseline_parameters": 68486,
            "losses": losses,
            "train_ids": [r["object_id"] for r in rows],
            "all_train_modes_sampled": all(
                bool(seen[i, slot].all()) for i, slot in enumerate(slots)
            ),
            "weights_sha256": teacher.sha(args.output / "model.safetensors"),
            "script_sha256": teacher.sha(Path(__file__)),
            "shared_script_sha256": teacher.sha(Path(shared.__file__)),
            "scope": "fixed physical-frequency channels replace normalized-rank queries; occupancy replaces scalar count; no DEV reads or sweeps",
        },
    )


def predict(model, cloud, features, contacts):
    with torch.no_grad():
        encoded = model.encode(torch.tensor(cloud[None]), torch.tensor(features[None]))
        log_poles, logits = model.mode_parameters(encoded)
        keep = logits[0] > 0  # Fixed 0.5 occupancy threshold, never tuned on DEV.
        poles = log_poles[0, keep].exp().numpy().astype(float)
        values = model.signed_field(
            encoded.expand(len(contacts), -1), torch.tensor(contacts)
        )
        gains = (
            (values[:, keep].sinh() * model.gain_scale)
            .numpy()
            .transpose(0, 2, 1)
            .astype(float)
        )
    result = {
        "frequency": poles[:, 0],
        "damping": poles[:, 1],
        "gains": gains,
        "occupancy_probability": logits[0].sigmoid().numpy().astype(float),
    }
    if (
        not all(np.isfinite(v).all() for v in result.values())
        or np.any(result["damping"] <= 0)
        or np.any(
            (result["frequency"] <= 0) | (result["frequency"] >= teacher.RATE / 2)
        )
    ):
        raise ValueError("invalid predicted channel response")
    # An empty predicted mask remains an explicit silent failure, not a target-
    # dependent repair or a reason to select another checkpoint/threshold.
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("fit", "render"))
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--fit", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    torch.set_num_threads(4)
    if args.stage == "fit":
        fit(args)
    else:
        shared.render(args, model_class=ChannelStudent, prediction_fn=predict)
