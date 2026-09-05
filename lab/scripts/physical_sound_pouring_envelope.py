"""Learn a low-dimensional amplitude envelope while preserving a frozen texture.

Local report-only experiment. No reference recording enters generation; source
audio supervises training and metrics only. Not calibrated pressure/flow rate.
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

FRAMES = 32
WINDOW = p.SAMPLES // FRAMES


def envelope(wave):
    if wave.shape != (p.SAMPLES,) or not np.isfinite(wave).all():
        raise ValueError("invalid envelope waveform")
    return np.sqrt(np.mean(wave.reshape(FRAMES, WINDOW) ** 2, axis=1) + 1e-12)


def encode(wave):
    return (np.log(np.maximum(envelope(wave), 1e-5)) / 3 + 2).astype(np.float32)


def shape_envelope(wave, encoded):
    if encoded.shape != (FRAMES,) or not np.isfinite(encoded).all():
        raise ValueError("invalid generated envelope")
    target = np.exp((np.clip(encoded, -2, 1.5) - 2) * 3)
    gain = target / np.maximum(envelope(wave), 1e-5)
    centers = (np.arange(FRAMES) + 0.5) * WINDOW - 0.5
    shaped = (wave * np.interp(np.arange(p.SAMPLES), centers, gain)).astype(np.float32)
    if not np.isfinite(shaped).all():
        raise ValueError("nonfinite shaped waveform")
    return shaped


def apply_envelope(wave, encoded):
    shaped = shape_envelope(wave, encoded)
    if abs(shaped).max() > 0.98:
        raise ValueError(
            "generated envelope exceeds headroom; do not silently normalize"
        )
    return shaped


class EnvelopeFlow(torch.nn.Module):
    def __init__(self):
        super().__init__()
        self.net = torch.nn.Sequential(
            torch.nn.Linear(FRAMES + 11 + 9, 128),
            torch.nn.SiLU(),
            torch.nn.Linear(128, 128),
            torch.nn.SiLU(),
            torch.nn.Linear(128, FRAMES),
        )

    def forward(self, x, t, c):
        angle = t[:, None] * torch.tensor([1, 2, 4, 8], device=x.device) * (2 * np.pi)
        return self.net(torch.cat([x, c, t[:, None], angle.sin(), angle.cos()], 1))


@torch.inference_mode()
def sample(model, controls, seed):
    device = next(model.parameters()).device
    x = torch.randn(
        1,
        FRAMES,
        generator=torch.Generator(device=device).manual_seed(seed),
        device=device,
    )
    c = torch.from_numpy(controls[None]).to(device)
    for step in range(64):
        x += model(x, torch.full((1,), step / 64, device=device), c) / 64
    result = x[0].cpu().numpy()
    if not np.isfinite(result).all():
        raise ValueError("nonfinite envelope")
    return result


def run(source, parent, output):
    source, parent, output = (x.resolve() for x in (source, parent, output))
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("use fresh external output")
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    training = [r for r in rows if r["role"] == "train"]
    texture, parent_meta = compare.load_model(parent, "cuda")
    if (
        parent_meta["train_ids"] != [r["item_id"] for r in training]
        or parent_meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("source/training exposure mismatch")
    patches = [
        compare.phase_crop(r, phase) for r in training for phase in ("first", "middle")
    ]
    targets = torch.from_numpy(np.stack([encode(x[2]) for x in patches])).cuda()
    controls = torch.from_numpy(np.stack([x[1] for x in patches])).cuda()
    output.mkdir(parents=True)
    models = {}
    fits = {}
    for kind in ("matched", "shuffled"):
        torch.manual_seed(53)
        rng = np.random.default_rng(53)
        model = EnvelopeFlow().cuda()
        optimizer = torch.optim.AdamW(model.parameters(), lr=3e-4, weight_decay=0.01)
        losses = []
        for step in range(1500):
            indices = rng.integers(len(patches), size=32)
            bits = rng.integers(2, size=32)
            y = targets[indices]
            c = controls[indices].clone()
            if kind == "shuffled":
                c[:, 4] = controls[(indices // 2) * 2 + bits, 4]
            noise = torch.randn_like(y)
            t = torch.rand(len(y), device="cuda")
            xt = (1 - t[:, None]) * noise + t[:, None] * y
            loss = torch.nn.functional.mse_loss(model(xt, t, c), y - noise)
            optimizer.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1)
            optimizer.step()
            losses.append(float(loss.detach()))
        model.eval()
        directory = output / kind
        directory.mkdir()
        checkpoint = directory / "model.safetensors"
        save_file(
            {k: v.detach().cpu().contiguous() for k, v in model.state_dict().items()},
            checkpoint,
        )
        meta = {
            "format": "pour-envelope-flow-v1",
            "parameters": sum(v.numel() for v in model.parameters()),
            "seed": 53,
            "steps": 1500,
            "phase_pairing": kind,
            "train_ids": parent_meta["train_ids"],
            "source_sha256": parent_meta["source_sha256"],
            "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
            "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
            "frames": FRAMES,
            "samples": p.SAMPLES,
            "scope": "coarse amplitude envelope, not calibrated sound pressure",
        }
        p.save_json(directory / "model.json", meta)
        models[kind] = model
        fits[kind] = {
            "model": meta,
            "loss_first100": float(np.mean(losses[:100])),
            "loss_last100": float(np.mean(losses[-100:])),
        }
        print(
            {"fit": kind, **{k: v for k, v in fits[kind].items() if k != "model"}},
            flush=True,
        )
    evaluate(rows, texture, models, fits, parent_meta, provenance, output)


def evaluate(rows, texture, models, fits, parent_meta, provenance, output):
    """Preserve raw level failures; common audition attenuation is not admission."""
    if (output / "result.json").exists() or any(output.glob("*.wav")):
        raise ValueError("refusing to overwrite previous evaluation artifacts")
    report = {
        "status": "complete",
        "scope": "same disclosed two-object development, three seeds, frozen texture plus neural envelope",
        "terms": provenance["terms"],
        "parent": parent_meta,
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
    all_pending = []
    development = [r for r in rows if r["role"] == "unseen_container"]
    for idx, row in enumerate(development):
        for phase in ("first", "middle"):
            _, c, real, offset = compare.phase_crop(row, phase)
            pending = [
                ("real", None, real),
                ("envelope-oracle", None, apply_envelope(real, encode(real))),
            ]
            for seed in (314, 2718, 1618):
                base = p.decode(p.sample(texture, c, seed), 314)
                pending.append(("base", seed, base))
                for kind, model in models.items():
                    pending.append(
                        (kind, seed, shape_envelope(base, sample(model, c, seed)))
                    )
            for kind, seed, wave in pending:
                key = f"{idx:02d}-{phase}-{kind}-{seed}"
                path = output / (key + ".wav")
                record = {
                    "item_id": row["item_id"],
                    "container_id": row["container_id"],
                    "phase": phase,
                    "kind": kind,
                    "seed": seed,
                    "texture_seed": seed,
                    "decoder_seed": 314,
                    "envelope_seed": seed if kind in models else None,
                    "controls": c.tolist(),
                    "start_sample": offset,
                    "wav": str(path),
                    "raw_peak": float(abs(wave).max()),
                    "raw_headroom_pass": bool(abs(wave).max() <= 0.98),
                    **p.metrics(wave, real),
                }
                all_pending.append((path, wave, record))
                if idx < 2 and (kind == "real" or seed == 314):
                    preview.extend([wave, np.zeros(p.RATE // 2)])
        print({"evaluated": idx + 1, "total": len(development)}, flush=True)
    peak = max(r["raw_peak"] for _, _, r in all_pending)
    gain = min(1, 0.98 / max(peak, 1e-12))
    report["raw_headroom_failures"] = sum(
        not r["raw_headroom_pass"] for _, _, r in all_pending
    )
    report["audition_gain"] = gain
    report["audition_scope"] = (
        "same gain for all real/base/candidate controls; raw failures remain failures"
    )
    for path, wave, record in all_pending:
        sf.write(path, wave * gain, p.RATE, subtype="PCM_16")
        record.update(sha256=hashlib.sha256(path.read_bytes()).hexdigest(), gain=gain)
        report["rows"].append(record)
        tags["cases"].append({"id": path.stem, "diagnostic_id": "water-pour"})
        tags["rows"].append({**record, "case": len(tags["cases"]) - 1})
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview) * gain, p.RATE, subtype="PCM_16")
    report["comparison"] = {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "gain": gain,
        "order": "first glass then PET; first/middle; real/base/matched/shuffled; generator/envelope314, decoder314",
    }
    p.save_json(output / "result.json", report)
    p.save_json(output / "tag-input.json", tags)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "parent", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.parent, args.output)
