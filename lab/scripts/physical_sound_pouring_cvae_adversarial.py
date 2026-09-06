"""Decoder-only CVAE fine-tuning with a training-only spectral critic.

Matched reconstruction-only continuation distinguishes critic effects from extra
updates. AST/CLAP never enter training. Report-only; no runtime promotion.
"""

import argparse
import hashlib
from pathlib import Path

import numpy as np
import physical_sound_pouring_cvae as v
import torch
from torch import nn
from torch.nn import functional as F

STEPS = 600


class PatchCritic(nn.Module):
    def __init__(self):
        super().__init__()
        self.layers = nn.ModuleList(
            [
                nn.Conv2d(a, b, 4, stride=2, padding=1)
                for a, b in ((1, 16), (16, 32), (32, 64), (64, 128))
            ]
        )
        self.output = nn.Conv2d(128, 1, 3, padding=1)

    def forward(self, value):
        features = []
        for layer in self.layers:
            value = F.leaky_relu(layer(value), 0.2)
            features.append(value)
        return self.output(value), features


class Critic(nn.Module):
    def __init__(self):
        super().__init__()
        self.scales = nn.ModuleList([PatchCritic(), PatchCritic()])

    def forward(self, value):
        return [self.scales[0](value), self.scales[1](F.avg_pool2d(value, 2))]


def decoder_parameter(name):
    return name.startswith(("decoder_input.", "decoder_blocks.", "output."))


def freeze_representation(model):
    for name, parameter in model.named_parameters():
        parameter.requires_grad_(decoder_parameter(name))
    return [p for p in model.parameters() if p.requires_grad]


def discriminator_loss(real, fake):
    return torch.stack(
        [
            F.relu(1 - r[0]).mean() + F.relu(1 + f[0]).mean()
            for r, f in zip(real, fake, strict=True)
        ]
    ).mean()


def generator_losses(fake, real, posterior_count):
    adversarial = torch.stack([-x[0].mean() for x in fake]).mean()
    matching = torch.stack(
        [
            (f[:posterior_count] - r.detach()).abs().mean()
            for (_, ff), (_, rf) in zip(fake, real, strict=True)
            for f, r in zip(ff, rf, strict=True)
        ]
    ).mean()
    return adversarial, matching


def train_step(model, critic, g_optimizer, d_optimizer, wave, controls, adversarial):
    target = v.encode(wave)[:, None]
    with torch.no_grad():
        q, prior = model.posterior(target, controls), model.prior(controls)
        zq = q[0] + (0.5 * q[1]).exp() * torch.randn_like(q[0])
        zp = prior[0] + (0.5 * prior[1]).exp() * torch.randn_like(prior[0])
    c = torch.cat([controls, controls])
    predicted = model.decode(torch.cat([zq, zp]), c)
    noise = torch.randn((len(predicted), v.p.SAMPLES), device=wave.device)
    synthesized = v.reconstruct(predicted, noise, 2)
    fake_image = v.encode(synthesized)[:, None]
    count = len(wave)
    reconstruction = F.l1_loss(predicted[:count], target) + 0.25 * F.l1_loss(
        fake_image[:count], target
    )
    disc = torch.zeros((), device=wave.device)
    adv = torch.zeros_like(disc)
    fm = torch.zeros_like(disc)
    if adversarial:
        critic.requires_grad_(True)
        disc = discriminator_loss(critic(target), critic(fake_image.detach()))
        if not torch.isfinite(disc):
            raise ValueError("nonfinite critic loss")
        d_optimizer.zero_grad(set_to_none=True)
        disc.backward()
        nn.utils.clip_grad_norm_(critic.parameters(), 1)
        d_optimizer.step()
        d_optimizer.zero_grad(set_to_none=True)
        critic.requires_grad_(False)
        adv, fm = generator_losses(critic(fake_image), critic(target), count)
    loss = reconstruction + 0.1 * adv + 0.5 * fm
    if not torch.isfinite(loss) or not torch.isfinite(disc):
        raise ValueError("nonfinite adversarial training")
    g_optimizer.zero_grad(set_to_none=True)
    loss.backward()
    nn.utils.clip_grad_norm_(model.parameters(), 1)
    g_optimizer.step()
    critic.requires_grad_(True)
    return [float(x.detach()) for x in (reconstruction, disc, adv, fm)]


def fit(source, parent, output, adversarial, device="cuda"):
    torch.set_num_threads(4)
    rows, _ = v.p.load_source(source)
    training = [r for r in rows if r["role"] == "train"]
    model, meta = v.load(parent, device)
    if (
        meta["train_ids"] != [r["item_id"] for r in training]
        or meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("parent/source mismatch")
    output = v.phase.d.fresh_output(output)
    trainable = freeze_representation(model)
    frozen = {
        k: x.detach().clone()
        for k, x in model.state_dict().items()
        if not decoder_parameter(k)
    }
    model.train()
    torch.manual_seed(53)
    # Same initialization and RNG consumption in both matched runs.
    critic = Critic().to(device)
    go = torch.optim.Adam(trainable, lr=1e-4, betas=(0.5, 0.9))
    do = torch.optim.Adam(critic.parameters(), lr=1e-4, betas=(0.5, 0.9))
    rng = np.random.default_rng(53)
    history = []
    for step in range(STEPS):
        waves, controls = [], []
        for index in rng.integers(len(training), size=3):
            row = training[index]
            start = int(
                rng.integers(max(1, row["spectrogram"].shape[1] - v.p.FRAMES + 1))
            )
            _, c = v.p.crop(row, start)
            wave = row["wave"][start * v.p.HOP : start * v.p.HOP + v.p.SAMPLES]
            waves.append(np.pad(wave, (0, v.p.SAMPLES - len(wave))))
            controls.append(c)
        history.append(
            train_step(
                model,
                critic,
                go,
                do,
                torch.tensor(np.stack(waves), device=device),
                torch.tensor(np.stack(controls), device=device),
                adversarial,
            )
        )
        if (step + 1) % 100 == 0:
            print(
                {
                    "adversarial": adversarial,
                    "step": step + 1,
                    "mean100_rec_d_adv_fm": np.mean(history[-100:], axis=0).tolist(),
                },
                flush=True,
            )
    if not all(
        torch.equal(model.state_dict()[k], value) for k, value in frozen.items()
    ):
        raise ValueError("frozen representation changed")
    checkpoint = output / "model.safetensors"
    v.save_file(
        {k: x.detach().cpu().contiguous() for k, x in model.state_dict().items()},
        checkpoint,
    )
    critic_meta = {}
    if adversarial:
        path = output / "critic.safetensors"
        v.save_file(
            {k: x.detach().cpu().contiguous() for k, x in critic.state_dict().items()},
            path,
        )
        critic_meta = {"critic_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    v.p.save_json(
        output / "model.json",
        {
            **meta,
            **critic_meta,
            "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
            "finetune_parent_sha256": meta["checkpoint_sha256"],
            "finetune_steps": STEPS,
            "finetune_seed": 53,
            "finetune_batch_records": 3,
            "finetune_learning_rate": 1e-4,
            "finetune_adam_betas": [0.5, 0.9],
            "finetune_loss_weights": {
                "reconstruction": 1,
                "adversarial": 0.1,
                "feature_matching": 0.5,
            },
            "inherited_metrics_scope": "steps and last100_reconstruction_kl describe the parent fit only",
            "adversarial": adversarial,
            "trainable_parameters": sum(x.numel() for x in trainable),
            "critic_parameters": sum(x.numel() for x in critic.parameters()),
            "frozen_representation_verified": True,
            "last100_rec_d_adv_fm": np.mean(history[-100:], axis=0).tolist(),
            "scope": "matched decoder-only continuation; critic training-only, not an independent validator",
        },
    )
    v.render(output, output / "first-audition", v.phase.d.controls_for(), device=device)


@torch.inference_mode()
def phase_budget_probe(source, parent, output, device="cuda"):
    """Test the two-iteration training / 32-iteration inference mismatch."""
    torch.set_num_threads(4)
    rows, _ = v.p.load_source(source)
    model, meta = v.load(parent, device)
    row = next(r for r in rows if r["role"] == "train")
    if (
        meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
        or row["item_id"] not in meta["train_ids"]
    ):
        raise ValueError("probe source mismatch")
    output = v.phase.d.fresh_output(output)
    records, preview = [], []
    for part in ("first", "middle"):
        _, controls, real, _ = v.compare.phase_crop(row, part)
        c = torch.tensor(controls[None], device=device)
        target = v.encode(torch.tensor(real[None], device=device))[:, None]
        values = {"exact": target}
        for kind, distribution in [
            ("posterior", model.posterior(target, c)),
            ("prior", model.prior(c)),
        ]:
            mu, lv = distribution
            latent = torch.randn(
                mu.shape,
                generator=torch.Generator(device=device).manual_seed(53),
                device=device,
            )
            values[kind] = model.decode(mu + (lv * 0.5).exp() * latent, c)
        pending = [("real", None, None, real)]
        for seed in (314, 2718, 1618):
            noise = torch.randn(
                (1, v.p.SAMPLES),
                generator=torch.Generator(device=device).manual_seed(seed),
                device=device,
            )
            for kind, value in values.items():
                for iterations in (2, 32):
                    wave = v.reconstruct(value, noise, iterations)[0].cpu().numpy()
                    pending.append((kind, seed, iterations, wave))
        for kind, seed, iterations, wave in pending:
            records.append(
                {
                    "kind": kind,
                    "phase": part,
                    "seed": seed,
                    "iterations": iterations,
                    "latent_seed": 53 if kind in ("posterior", "prior") else None,
                    "reference_audio_input": kind in ("exact", "posterior"),
                    **v.h.save_wav(
                        output / f"{part}-{kind}-{seed}-{iterations}.wav", wave
                    ),
                    **v.p.metrics(wave, real),
                }
            )
            if seed in (None, 2718):
                preview.extend([wave, np.zeros(v.p.RATE // 2)])
    v.p.save_json(
        output / "result.json",
        {
            "status": "complete",
            "scope": "training-record codec/mode discriminator; not generalization",
            "item_id": row["item_id"],
            "model_sha256": meta["checkpoint_sha256"],
            "rows": records,
            "comparison": {
                **v.h.save_wav(output / "comparison.wav", np.concatenate(preview)),
                "order": "first/middle; real/exact2/exact32/posterior2/posterior32/prior2/prior32; latent53/phase2718/gain1",
            },
        },
    )
    v.p.save_json(
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


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "parent", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--adversarial", action="store_true")
    parser.add_argument("--phase-budget-probe", action="store_true")
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cuda")
    args = parser.parse_args()
    if args.phase_budget_probe:
        if args.adversarial:
            parser.error("probe does not train")
        phase_budget_probe(args.source, args.parent, args.output, args.device)
    else:
        fit(args.source, args.parent, args.output, args.adversarial, args.device)
