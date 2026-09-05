"""Two-patch training-side conditioning discriminator; not generalization.

Fine-tune the same frozen parent with matched or shuffled phase conditions.
Default: first training recording. Optional paired-record extension uses only
the same disclosed training split and evaluates excluded objects. Research only.
"""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p
import soundfile as sf
import torch
from safetensors.torch import save_file


def batch(targets, conditions, shuffled):
    indices = torch.arange(6, device=targets.device) % 2
    if len(targets) > 2:
        indices = indices + 2 * torch.randint(
            len(targets) // 2, (3,), device=targets.device
        ).repeat_interleave(2)
    permutation = torch.randperm(6, device=targets.device)
    # Both branches consume identical RNG; only condition-to-target pairing differs.
    controls = conditions[indices].clone()
    if shuffled:
        other = 2 * (indices // 2) + (indices[permutation] % 2)
        controls[:, 4] = conditions[other, 4]
    y = targets[indices]
    noise = torch.randn_like(y)
    time = torch.rand(6, device=targets.device)
    t = time[:, None, None, None]
    return (1 - t) * noise + t * y, time, controls, y - noise


def run(source, parent, output, steps=600, device="cuda", all_training=False):
    source, parent, output = (x.resolve() for x in (source, parent, output))
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("use a fresh external output")
    if not 1 <= steps <= 600:
        raise ValueError("bounded probe, not an epoch sweep")
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    training = [r for r in rows if r["role"] == "train"]
    row = training[0]
    selected = training if all_training else training[:1]
    original, parent_meta = compare.load_model(parent, device)
    source_hash = hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    if (
        parent_meta["source_sha256"] != source_hash
        or [r["item_id"] for r in training] != parent_meta["train_ids"]
    ):
        raise ValueError("source/parent/training identity mismatch")
    patches = [
        compare.phase_crop(r, phase) for r in selected for phase in ("first", "middle")
    ]
    targets = torch.from_numpy(np.stack([x[0] for x in patches])[:, None]).to(device)
    conditions = torch.from_numpy(np.stack([x[1] for x in patches])).to(device)
    output.mkdir(parents=True)
    models = {"parent": original}
    fits = {}
    for kind in ("matched", "shuffled"):
        torch.manual_seed(53)
        model, _ = compare.load_model(parent, device)
        model.train()
        optimizer = torch.optim.AdamW(model.parameters(), lr=3e-4, weight_decay=0.01)
        losses = []
        for step in range(steps):
            xt, time, controls, velocity = batch(
                targets, conditions, kind == "shuffled"
            )
            loss = torch.nn.functional.mse_loss(model(xt, time, controls), velocity)
            optimizer.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1)
            optimizer.step()
            losses.append(float(loss.detach()))
            if (step + 1) % 100 == 0:
                print(
                    {
                        "kind": kind,
                        "step": step + 1,
                        "loss_mean100": float(np.mean(losses[-100:])),
                    },
                    flush=True,
                )
        model.eval()
        directory = output / kind
        directory.mkdir()
        weights = directory / "model.safetensors"
        save_file(
            {k: v.detach().cpu().contiguous() for k, v in model.state_dict().items()},
            weights,
        )
        meta = {
            **parent_meta,
            # Fine-tuning does not erase the parent's training exposure.
            "train_ids": parent_meta["train_ids"],
            "finetune_ids": [r["item_id"] for r in selected],
            "checkpoint_sha256": hashlib.sha256(weights.read_bytes()).hexdigest(),
            "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
            "steps": steps,
            "parent_training_steps": parent_meta["steps"],
            "objective": "velocity",
            "patch_sampling": "two fixed patches, balanced",
            "phase_pairing": kind,
            "scope": "paired-record development extension"
            if all_training
            else "training memorization probe; no generalization claim",
        }
        p.save_json(directory / "model.json", meta)
        models[kind] = model
        fits[kind] = {
            "model": meta,
            "loss_mean_first100": float(np.mean(losses[:100])),
            "loss_mean_last100": float(np.mean(losses[-100:])),
        }
    report = {
        "status": "complete",
        "scope": "disclosed two-object development, not independent test"
        if all_training
        else "same-recording two-phase memorization, not generalization",
        "all_training": all_training,
        "source_sha256": source_hash,
        "item_id": row["item_id"],
        "container_id": row["container_id"],
        "terms": provenance["terms"],
        "fits": fits,
        "reference_audio_input_to_generator": False,
        "rows": [],
    }
    tags = {
        "status": "complete",
        "seconds": p.SAMPLES / p.RATE,
        "cases": [],
        "rows": [],
        "controls": [],
    }
    preview = []
    evaluation = (
        [r for r in rows if r["role"] == "unseen_container"] if all_training else [row]
    )
    for record_index, evaluation_row in enumerate(evaluation):
        records, clips = evaluate_record(
            evaluation_row,
            models,
            output,
            f"{record_index:02d}-" if all_training else "",
        )
        for record in records:
            report["rows"].append(record)
            tags["cases"].append(
                {"id": Path(record["wav"]).stem, "diagnostic_id": "water-pour"}
            )
            tags["rows"].append({**record, "case": len(tags["cases"]) - 1})
        if record_index < 2:
            preview.extend(clips)
        print({"evaluated": record_index + 1, "total": len(evaluation)}, flush=True)
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview), p.RATE, subtype="PCM_16")
    report["comparison"] = {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "gain": 1,
        "order": "first two source-order objects (one in probe); first then middle; real/parent/matched/shuffled, seed314",
    }
    p.save_json(output / "result.json", report)
    p.save_json(output / "tag-input.json", tags)


def evaluate_record(row, models, output, prefix):
    patches = [compare.phase_crop(row, phase) for phase in ("first", "middle")]
    records, preview = [], []
    for index, (spec, controls, real, offset) in enumerate(patches):
        phase = ("first", "middle")[index]
        pending = [
            ("real", None, real, spec),
            ("oracle", 314, p.decode(spec, 314), spec),
        ]
        for seed in (314, 2718, 1618):
            for kind, model in models.items():
                pred = p.sample(model, controls, seed)
                pending.append((kind, seed, p.decode(pred, seed), pred))
        for kind, seed, wave, pred in pending:
            if not np.isfinite(wave).all() or abs(wave).max() > 0.98:
                raise ValueError("invalid waveform; no automatic level correction")
            key = f"{prefix}{phase}-{kind}-{seed}"
            path = output / (key + ".wav")
            sf.write(path, wave, p.RATE, subtype="PCM_16")
            own = float(np.sqrt(np.mean((pred - spec) ** 2)))
            other = float(np.sqrt(np.mean((pred - patches[1 - index][0]) ** 2)))
            record = {
                "item_id": row["item_id"],
                "container_id": row["container_id"],
                "phase": phase,
                "kind": kind,
                "seed": seed,
                "controls": controls.tolist(),
                "start_sample": offset,
                "wav": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "gain": 1,
                "template_rmse": own,
                "template_margin": other - own,
                **p.metrics(wave, real),
            }
            records.append(record)
            if kind == "real" or (seed == 314 and kind != "oracle"):
                preview.extend([wave, np.zeros(p.RATE // 2)])
    return records, preview


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "parent", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--all-training", action="store_true")
    args = parser.parse_args()
    run(args.source, args.parent, args.output, all_training=args.all_training)
