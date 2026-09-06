"""Conditional latent spectrogram model; report-only, internet data, no runtime.

Posterior reads a recording during training/reconstruction. Prior generation uses
only object/event controls and randomness. Keep their evaluation claims separate.
"""

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_phase_oracle as phase
import physical_sound_pouring_pilot as p
import physical_sound_pouring_resonance_head as h
import torch
from safetensors.torch import load_file, save_file
from torch import nn
from torch.nn import functional as F

LATENT = 8
STEPS = 2000


def condition_field(controls):
    axis_f = torch.linspace(-1, 1, 32, device=controls.device)
    axis_t = torch.linspace(-1, 1, 16, device=controls.device)
    return torch.cat(
        [
            controls[:, :, None, None].expand(-1, -1, 32, 16),
            axis_f[None, None, :, None].expand(len(controls), 1, 32, 16),
            axis_t[None, None, None, :].expand(len(controls), 1, 32, 16),
        ],
        1,
    )


def block(inputs, outputs, stride=1):
    return nn.Sequential(
        nn.Conv2d(inputs, outputs, 3, stride=stride, padding=1),
        nn.GroupNorm(4, outputs),
        nn.SiLU(),
        nn.Conv2d(outputs, outputs, 3, padding=1),
        nn.SiLU(),
    )


class PourCVAE(nn.Module):
    def __init__(self):
        super().__init__()
        self.encoder = nn.Sequential(
            block(1, 16, 2), block(16, 32, 2), block(32, 64, 2), block(64, 128, 2)
        )
        self.posterior_head = nn.Conv2d(128 + 13, LATENT * 2, 3, padding=1)
        self.prior_net = nn.Sequential(
            block(13, 64), nn.Conv2d(64, LATENT * 2, 3, padding=1)
        )
        self.decoder_input = block(LATENT + 13, 128)
        self.decoder_blocks = nn.ModuleList(
            [block(128, 64), block(64, 32), block(32, 16), block(16, 16)]
        )
        self.output = nn.Conv2d(16, 1, 3, padding=1)

    @staticmethod
    def distribution(value):
        mean, log_variance = value.chunk(2, dim=1)
        return mean, log_variance.clamp(-6, 2)

    def prior(self, controls):
        return self.distribution(self.prior_net(condition_field(controls)))

    def posterior(self, target, controls):
        return self.distribution(
            self.posterior_head(
                torch.cat([self.encoder(target), condition_field(controls)], 1)
            )
        )

    def decode(self, latent, controls):
        value = self.decoder_input(torch.cat([latent, condition_field(controls)], 1))
        for layer in self.decoder_blocks:
            value = layer(
                F.interpolate(
                    value, scale_factor=2, mode="bilinear", align_corners=False
                )
            )
        return 2 * torch.tanh(self.output(value) / 2)


def gaussian_kl(q, prior):
    qm, qv = q
    pm, pv = prior
    return 0.5 * (pv - qv + (qv.exp() + (qm - pm).square()) / pv.exp() - 1).mean()


def encode(wave):
    magnitude = phase.transform(wave).abs()[..., 1:, :] / (phase.d.FFT / 2)
    return ((20 * magnitude.clamp_min(1e-5).log10() + 50) / 25).clamp(-2, 2)


def magnitude(value):
    value = value[:, 0]
    linear = torch.pow(10, (value * 25 - 50) / 20) * (phase.d.FFT / 2)
    return torch.cat([torch.zeros_like(linear[:, :1]), linear], 1)


def reconstruct(value, excitation, iterations):
    target = magnitude(value)
    initial = phase.transform(excitation)
    angle = initial / initial.abs().clamp_min(1e-12)
    for _ in range(iterations):
        updated = phase.transform(phase.inverse(target * angle))
        angle = updated / updated.abs().clamp_min(1e-12)
    return phase.inverse(target * angle)


def reconstruction_loss(prediction, target, excitation):
    # Two differentiable consistency iterations; 32 at audition/inference.
    wave = reconstruct(prediction, excitation, 2)
    consistent = encode(wave)[:, None]
    return F.l1_loss(prediction, target) + 0.25 * F.l1_loss(consistent, target)


@torch.inference_mode()
def sample(model, controls, seed=2718, reference=None):
    if not isinstance(seed, int) or not 0 <= seed < 2**32:
        raise ValueError("invalid seed")
    h.grid(controls, np.array([0]))
    device = next(model.parameters()).device
    c = torch.tensor(np.asarray(controls, dtype=np.float32)[None], device=device)
    if reference is None:
        distribution = model.prior(c)
    else:
        if reference.shape != (p.SAMPLES,) or not np.isfinite(reference).all():
            raise ValueError("invalid posterior reference")
        target = encode(torch.tensor(reference[None], device=device))[:, None]
        distribution = model.posterior(target, c)
    mu, lv = distribution
    rng = torch.Generator(device=device).manual_seed(seed)
    z = mu + (0.5 * lv).exp() * torch.randn(mu.shape, generator=rng, device=device)
    excitation = torch.randn(
        (1, p.SAMPLES),
        generator=torch.Generator(device=device).manual_seed(314),
        device=device,
    )
    value = model.decode(z, c)
    wave = reconstruct(value, excitation, 32)[0].cpu().numpy()
    if not np.isfinite(wave).all() or abs(wave).max() > 0.98:
        raise ValueError("unsafe output; no clipping/normalization")
    return wave


def load(directory, device="cuda"):
    meta_path, checkpoint = directory / "model.json", directory / "model.safetensors"
    if meta_path.stat().st_size > 100000 or checkpoint.stat().st_size > 10_000_000:
        raise ValueError("oversized model")
    meta = json.loads(meta_path.read_text())
    if (
        meta["format"] != "pour-cvae-v1"
        or meta["checkpoint_sha256"]
        != hashlib.sha256(checkpoint.read_bytes()).hexdigest()
    ):
        raise ValueError("model identity mismatch")
    state = load_file(checkpoint)
    if not all(torch.isfinite(v).all() for v in state.values()):
        raise ValueError("nonfinite model weights")
    model = PourCVAE()
    model.load_state_dict(state, strict=True)
    return model.to(device).eval(), meta


def render(directory, output, controls, seed=2718, device="cuda"):
    torch.set_num_threads(4)
    model, meta = load(directory, device)
    wave = sample(model, controls, seed)
    output = phase.d.fresh_output(output)
    row = h.save_wav(output / "prior.wav", wave)
    p.save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "checkpoint_sha256": meta["checkpoint_sha256"],
            "controls": controls.tolist(),
            "seed": seed,
            "phase_seed": 314,
            "rows": [row],
        },
    )
    print({"primary_artifact": row["wav"]}, flush=True)


def fit(source, output, device="cuda"):
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    training = [r for r in rows if r["role"] == "train"]
    output = phase.d.fresh_output(output)
    torch.manual_seed(53)
    model = PourCVAE().to(device)
    optimizer = torch.optim.AdamW(model.parameters(), lr=3e-4, weight_decay=1e-4)
    rng = np.random.default_rng(53)
    history = []
    for step in range(STEPS):
        waves, conditions = [], []
        for index in rng.integers(len(training), size=6):
            row = training[index]
            start = int(
                rng.integers(max(1, row["spectrogram"].shape[1] - p.FRAMES + 1))
            )
            _, c = p.crop(row, start)
            wave = row["wave"][start * p.HOP : start * p.HOP + p.SAMPLES]
            waves.append(np.pad(wave, (0, p.SAMPLES - len(wave))))
            conditions.append(c)
        wave = torch.tensor(np.stack(waves), device=device)
        target = encode(wave)[:, None]
        c = torch.tensor(np.stack(conditions), device=device)
        q, prior = model.posterior(target, c), model.prior(c)
        z = q[0] + (0.5 * q[1]).exp() * torch.randn_like(q[0])
        prediction = model.decode(z, c)
        rec = reconstruction_loss(prediction, target, torch.randn_like(wave))
        kl = gaussian_kl(q, prior)
        beta = 0.01 * min(1, (step + 1) / 500)
        loss = rec + beta * kl
        if not torch.isfinite(loss):
            raise ValueError("nonfinite CVAE loss")
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
        history.append((float(rec.detach()), float(kl.detach())))
        if (step + 1) % 100 == 0:
            avg = np.mean(history[-100:], axis=0)
            print(
                {
                    "step": step + 1,
                    "reconstruction": float(avg[0]),
                    "kl": float(avg[1]),
                    "beta": beta,
                },
                flush=True,
            )
    checkpoint = output / "model.safetensors"
    save_file(
        {k: v.detach().cpu().contiguous() for k, v in model.state_dict().items()},
        checkpoint,
    )
    p.save_json(
        output / "model.json",
        {
            "format": "pour-cvae-v1",
            "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
            "source_sha256": hashlib.sha256(
                (source / "source.json").read_bytes()
            ).hexdigest(),
            "train_ids": [r["item_id"] for r in training],
            "heldout_containers": list(p.HELDOUT),
            "terms": provenance["terms"],
            "steps": STEPS,
            "seed": 53,
            "parameters": sum(x.numel() for x in model.parameters()),
            "last100_reconstruction_kl": np.mean(history[-100:], axis=0).tolist(),
        },
    )
    render(output, output / "first-audition", phase.d.controls_for(), device=device)


@torch.inference_mode()
def codec_probe(source, directory, output, device="cuda"):
    """Source-dependent axis-coarsening controls; not a latent-capacity proof."""
    torch.set_num_threads(4)
    rows, _ = p.load_source(source)
    row = next(r for r in rows if r["role"] == "train")
    model, meta = load(directory, device)
    if (
        meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
        or row["item_id"] not in meta["train_ids"]
    ):
        raise ValueError("probe source mismatch")
    output = phase.d.fresh_output(output)
    records, preview = [], []
    for part in ("first", "middle"):
        _, controls, real, _ = compare.phase_crop(row, part)
        c = torch.tensor(controls[None], device=device)
        target = encode(torch.tensor(real[None], device=device))[:, None]
        q = model.posterior(target, c)
        rng = torch.Generator(device=device).manual_seed(53)
        z = q[0] + (q[1] * 0.5).exp() * torch.randn(
            q[0].shape, generator=rng, device=device
        )
        values = {"exact": target, "posterior": model.decode(z, c)}
        for kind, size in [
            ("coarse_time", (512, 16)),
            ("coarse_frequency", (32, 256)),
            ("coarse_both", (32, 16)),
        ]:
            values[kind] = F.interpolate(
                F.interpolate(target, size=size, mode="area"),
                size=(512, 256),
                mode="bilinear",
                align_corners=False,
            )
        pending = [("real", None, real)]
        for seed in (314, 2718, 1618):
            noise = torch.randn(
                (1, p.SAMPLES),
                generator=torch.Generator(device=device).manual_seed(seed),
                device=device,
            )
            pending.extend(
                (kind, seed, reconstruct(value, noise, 32)[0].cpu().numpy())
                for kind, value in values.items()
            )
        for kind, seed, wave in pending:
            records.append(
                {
                    "kind": kind,
                    "phase": part,
                    "seed": seed,
                    "latent_seed": 53 if kind == "posterior" else None,
                    **h.save_wav(output / f"{part}-{kind}-{seed}.wav", wave),
                    **p.metrics(wave, real),
                }
            )
            if seed in (None, 2718):
                preview.extend([wave, np.zeros(p.RATE // 2)])
    p.save_json(
        output / "result.json",
        {
            "status": "complete",
            "scope": "source-dependent TRAINING-record diagnostic, not new-condition synthesis or proof of latent capacity",
            "item_id": row["item_id"],
            "model_sha256": meta["checkpoint_sha256"],
            "rows": records,
            "comparison": {
                **h.save_wav(output / "comparison.wav", np.concatenate(preview)),
                "order": "first/middle; real/exact/posterior/coarse_time/coarse_frequency/coarse_both; phase2718; posterior latent53; gain1",
            },
        },
    )
    p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": 4.08,
            "cases": [
                {"id": f"{i}-{r['kind']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(records)
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(records)],
            "controls": [],
        },
    )


def evaluate(source, directory, base_directory, output, device="cuda"):
    torch.set_num_threads(4)
    rows, _ = p.load_source(source)
    model, meta = load(directory, device)
    base, bm = compare.load_model(base_directory, device)
    source_sha = hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    train_ids = [r["item_id"] for r in rows if r["role"] == "train"]
    if any(
        m["train_ids"] != train_ids or m["source_sha256"] != source_sha
        for m in (meta, bm)
    ):
        raise ValueError("evaluation model/source mismatch")
    output = phase.d.fresh_output(output)
    selected = {"train-first": next(r for r in rows if r["role"] == "train")}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    report = {
        "status": "running",
        "scope": "one training record and first record of two disclosed excluded objects; not independent test",
        "model_sha256": meta["checkpoint_sha256"],
        "rows": [],
        "novel": [],
    }
    for name, row in selected.items():
        for part in ("first", "middle"):
            _, c, real, _ = compare.phase_crop(row, part)
            pending = [("real", None, real)]
            for seed in (314, 2718, 1618):
                pending.extend(
                    [
                        ("base", seed, p.decode(p.sample(base, c, seed), 314)),
                        ("posterior", seed, sample(model, c, seed, real)),
                        ("prior", seed, sample(model, c, seed)),
                    ]
                )
            for kind, seed, wave in pending:
                report["rows"].append(
                    {
                        "object": name,
                        "item_id": row["item_id"],
                        "phase": part,
                        "kind": kind,
                        "seed": seed,
                        "reference_audio_input": kind == "posterior",
                        **h.save_wav(output / f"{name}-{part}-{kind}-{seed}.wav", wave),
                        **p.metrics(wave, real),
                    }
                )
        print({"evaluated": name}, flush=True)
    preview = []
    for name, height, material, duration in [
        ("glass10", 10, "glass", 15),
        ("glass16", 16, "glass", 15),
        ("pet10", 10, "plastic_pet", 15),
        ("glass10fast", 10, "glass", 8),
    ]:
        c = phase.d.controls_for(height, material, duration)
        for seed in (314, 2718, 1618):
            for kind, wave in [
                ("base", p.decode(p.sample(base, c, seed), 314)),
                ("prior", sample(model, c, seed)),
            ]:
                report["novel"].append(
                    {
                        "profile": name,
                        "controls": c.tolist(),
                        "kind": kind,
                        "seed": seed,
                        "reference_audio_input": False,
                        **h.save_wav(output / f"{name}-{kind}-{seed}.wav", wave),
                    }
                )
                if seed == 2718:
                    preview.extend([wave, np.zeros(p.RATE // 2)])
    report["comparison"] = {
        **h.save_wav(output / "comparison.wav", np.concatenate(preview)),
        "order": "glass10/glass16/PET10/glass10fast; base/prior; seed2718; gain1",
    }
    report["status"] = "complete"
    p.save_json(output / "result.json", report)
    inputs = report["rows"] + report["novel"]
    p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": 4.08,
            "cases": [
                {"id": f"{i}-{r['kind']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(inputs)
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(inputs)],
            "controls": [],
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    for name, paths in [
        ("train", ("source",)),
        ("evaluate", ("source", "model", "base")),
        ("probe", ("source", "model")),
        ("render", ("model",)),
    ]:
        command = sub.add_parser(name)
        for path in paths + ("output",):
            command.add_argument("--" + path, type=Path, required=True)
        command.add_argument("--device", choices=["cpu", "cuda"], default="cuda")
        if name == "render":
            command.add_argument("--controls", type=float, nargs=11, required=True)
            command.add_argument("--seed", type=int, default=2718)
    args = parser.parse_args()
    if args.mode == "train":
        fit(args.source, args.output, args.device)
    elif args.mode == "probe":
        codec_probe(args.source, args.model, args.output, args.device)
    elif args.mode == "evaluate":
        evaluate(args.source, args.model, args.base, args.output, args.device)
    else:
        render(
            args.model,
            args.output,
            np.asarray(args.controls, dtype=np.float32),
            args.seed,
            args.device,
        )
