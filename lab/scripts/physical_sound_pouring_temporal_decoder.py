"""Report-only condition-to-waveform temporal noise/resonance decoder.

Training uses published pouring audio; generation uses conditions, frozen learned
trajectory weights and random excitation only. No runtime or physical admission.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p
import physical_sound_pouring_resonance_head as h
import torch
from safetensors.torch import load_file, save_file
from torch import nn
from torch.nn import functional as F

FFT = 1024
BANDS = 65
TIMES = np.arange(256) * p.HOP / p.RATE


def filtered_noise(noise, gain):
    window = torch.hann_window(FFT, device=noise.device)
    spectrum = torch.stft(noise, FFT, p.HOP, window=window, return_complex=True)
    return torch.istft(spectrum * gain, FFT, p.HOP, window=window, length=p.SAMPLES)


class TemporalDecoder(nn.Module):
    def __init__(self, head=None):
        super().__init__()
        self.head = h.ResonanceHead() if head is None else head
        self.head.requires_grad_(False)
        self.input = nn.Sequential(nn.Linear(12, 96), nn.SiLU())
        self.temporal = nn.GRU(96, 96, batch_first=True)
        self.output = nn.Linear(96, BANDS + 2)
        nn.init.zeros_(self.output.weight)
        with torch.no_grad():
            self.output.bias[:BANDS].fill_(-4.6)
            self.output.bias[BANDS] = -2
            self.output.bias[BANDS + 1] = -1

    def frequency(self, controls):
        return 34000 / torch.exp2(self.head(controls) * 2 + 5)

    def response(self, controls, hz, mode="temporal"):
        if mode not in ("temporal", "static", "no_resonance"):
            raise ValueError("unknown synthesis mode")
        features = torch.cat([controls, torch.log2(hz[:, :, None] / 1000)], -1)
        sequence, _ = self.temporal(self.input(features))
        coefficients = self.output(sequence)
        log_gain = (
            F.interpolate(
                coefficients[:, :, :BANDS].reshape(-1, 1, BANDS),
                size=FFT // 2 + 1,
                mode="linear",
                align_corners=True,
            )
            .reshape(len(controls), 256, FFT // 2 + 1)
            .transpose(1, 2)
        )
        gain = log_gain.clamp(-10, 0).exp()
        if mode != "no_resonance":
            width = 50 + 850 * coefficients[:, :, BANDS + 1].sigmoid()
            strength = F.softplus(coefficients[:, :, BANDS])
            frequency = torch.arange(FFT // 2 + 1, device=hz.device) * p.RATE / FFT
            cents = 1200 * torch.log2(
                frequency.clamp_min(1)[None, :, None] / hz[:, None]
            )
            gain = gain * (
                1
                + strength[:, None]
                * torch.exp(-0.5 * (cents / width[:, None]).square())
            )
        if mode == "static":
            # Same mean-square filter energy, but no changing spectral envelope.
            gain = gain.square().mean(-1, keepdim=True).sqrt().expand_as(gain)
        return gain

    def forward(self, controls, noise, hz=None, mode="temporal"):
        hz = self.frequency(controls) if hz is None else hz
        return filtered_noise(noise, self.response(controls, hz, mode))


def objective(wave, target):
    losses = []
    for size in (256, 1024, 2048):
        window = torch.hann_window(size, device=wave.device)

        def magnitude(x, size=size, window=window):
            return torch.stft(
                x, size, size // 4, window=window, return_complex=True
            ).abs() / (size / 2)

        a, b = magnitude(wave), magnitude(target)
        log_error = ((a + 1e-5).log() - (b + 1e-5).log()).abs().mean()
        convergence = (
            torch.linalg.vector_norm(a - b, dim=(1, 2))
            / torch.linalg.vector_norm(b, dim=(1, 2)).clamp_min(1e-6)
        ).mean()
        losses.append(log_error + convergence)

    def envelope(x):
        return F.avg_pool1d(x[:, None].square(), 320, 320).clamp_min(1e-12).sqrt()

    temporal = (
        ((envelope(wave) + 1e-5).log() - (envelope(target) + 1e-5).log()).abs().mean()
    )
    return torch.stack(losses).mean() + 0.5 * temporal


def controls_for(height=10, material="glass", duration=15):
    return p.condition(
        {"net_height": height, "diameter_top": 7, "diameter_bottom": 7},
        material,
        "cylindrical",
        duration,
        0.1,
    )


@torch.inference_mode()
def sample(model, controls, seed=2718, mode="temporal"):
    if not isinstance(seed, int) or not 0 <= seed < 2**32:
        raise ValueError("invalid seed")
    device = next(model.parameters()).device
    c = torch.tensor(h.grid(controls, TIMES)[None], device=device)
    hz = model.frequency(c)
    if not torch.isfinite(hz).all() or (hz < 50).any() or (hz > 7800).any():
        raise ValueError("frequency outside numerical range")
    noise = torch.randn(
        (1, p.SAMPLES),
        generator=torch.Generator(device=device).manual_seed(seed),
        device=device,
    )
    wave = model(c, noise, hz, mode)[0].cpu().numpy()
    if not np.isfinite(wave).all() or abs(wave).max() > 0.98:
        raise ValueError("unsafe raw generation; no attenuation or clipping")
    return wave


def load(directory, device="cuda"):
    meta_path, weights = directory / "model.json", directory / "model.safetensors"
    if meta_path.stat().st_size > 100000 or weights.stat().st_size > 2_000_000:
        raise ValueError("oversized decoder")
    meta = json.loads(meta_path.read_text())
    if (
        meta["format"] != "pour-temporal-decoder-v1"
        or hashlib.sha256(weights.read_bytes()).hexdigest() != meta["checkpoint_sha256"]
    ):
        raise ValueError("decoder identity mismatch")
    state = load_file(weights)
    if not all(torch.isfinite(v).all() for v in state.values()):
        raise ValueError("nonfinite decoder weights")
    model = TemporalDecoder()
    model.load_state_dict(state, strict=True)
    return model.to(device).eval(), meta


def fresh_output(output):
    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    output.mkdir(parents=True)
    return output


def render(directory, output, controls, seed=2718, device="cuda"):
    model, meta = load(directory, device)
    wave = sample(model, controls, seed)
    output = fresh_output(output)
    row = h.save_wav(output / "generated.wav", wave)
    p.save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "teacher_required_at_inference": False,
            "model": meta,
            "controls": controls.tolist(),
            "seed": seed,
            "rows": [row],
        },
    )
    print({"primary_artifact": row["wav"]}, flush=True)


def learnability(output, device):
    """In-family synthetic positive control, NOT real-data quality evidence."""
    torch.manual_seed(53)
    model = TemporalDecoder().to(device)
    controls = torch.tensor(
        np.stack([h.grid(controls_for(z), TIMES) for z in (10, 16)]), device=device
    )
    hz = torch.stack(
        [
            torch.linspace(500, 1800, 256, device=device),
            torch.linspace(1800, 500, 256, device=device),
        ]
    )
    f = torch.arange(FFT // 2 + 1, device=device) * p.RATE / FFT
    ridge = torch.exp(
        -0.5
        * (
            1200 * torch.log2(f.clamp_min(1)[None, :, None] / hz[:, None]) / 180
        ).square()
    )
    amplitude = torch.stack(
        [
            torch.linspace(0.5, 1.5, 256, device=device),
            torch.linspace(1.5, 0.5, 256, device=device),
        ]
    )
    target_gain = (0.003 + 0.03 * ridge) * amplitude[:, None]
    rng = torch.Generator(device=device).manual_seed(314)
    target = filtered_noise(
        torch.randn((2, p.SAMPLES), generator=rng, device=device), target_gain
    )
    evaluation_noise = torch.randn(
        (2, p.SAMPLES),
        generator=torch.Generator(device=device).manual_seed(2718),
        device=device,
    )
    with torch.no_grad():
        before = model(controls, evaluation_noise, hz)
        initial = float(objective(before, target))
    optimizer = torch.optim.AdamW(
        [v for v in model.parameters() if v.requires_grad], lr=1e-3, weight_decay=1e-4
    )
    for step in range(300):
        generated = model(controls, torch.randn_like(target), hz)
        loss = objective(generated, target)
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
    with torch.no_grad():
        after = model(controls, evaluation_noise, hz)
        static = model(controls, evaluation_noise, hz, "static")
        final, stationary = (
            float(objective(after, target)),
            float(objective(static, target)),
        )
    output.mkdir()
    rows = []
    for i in range(2):
        for kind, waves in [
            ("target", target),
            ("before", before),
            ("after", after),
            ("static", static),
        ]:
            rows.append(
                {
                    "case": i,
                    "kind": kind,
                    **h.save_wav(
                        output / f"{i}-{kind}.wav", waves[i].detach().cpu().numpy()
                    ),
                }
            )
    result = {
        "scope": "synthetic known-response family; supplied true trajectory, not condition-to-pitch validation",
        "initial_loss": initial,
        "final_loss": final,
        "static_loss": stationary,
        "passed": final < 0.7 * initial and final < stationary,
        "rows": rows,
    }
    p.save_json(output / "result.json", result)
    print({k: v for k, v in result.items() if k != "rows"}, flush=True)
    return result["passed"]


def run(source, parent, head, output, device="cuda"):
    torch.set_num_threads(4)
    output = fresh_output(output)
    if not learnability(output / "synthetic", device):
        raise ValueError("synthetic positive control failed; do not launch real fit")
    rows, provenance = p.load_source(source)
    training = [r for r in rows if r["role"] == "train"]
    base, pm = compare.load_model(parent, device)
    if (
        pm["train_ids"] != [r["item_id"] for r in training]
        or pm["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("source/parent training identity mismatch")
    trajectory, hm = h.load_head(head, pm, device)
    torch.manual_seed(53)
    model = TemporalDecoder(trajectory).to(device)
    optimizer = torch.optim.AdamW(
        [v for v in model.parameters() if v.requires_grad], lr=1e-3, weight_decay=1e-4
    )
    rng = np.random.default_rng(53)
    history = []
    for step in range(1200):
        conditions, targets = [], []
        for index in rng.integers(len(training), size=6):
            row = training[index]
            start = int(
                rng.integers(max(1, row["spectrogram"].shape[1] - p.FRAMES + 1))
            )
            _, c = p.crop(row, start)
            conditions.append(h.grid(c, TIMES))
            wave = row["wave"][start * p.HOP : start * p.HOP + p.SAMPLES]
            targets.append(np.pad(wave, (0, p.SAMPLES - len(wave))))
        c = torch.tensor(np.stack(conditions), device=device)
        target = torch.tensor(np.stack(targets), device=device)
        loss = objective(model(c, torch.randn_like(target)), target)
        if not torch.isfinite(loss):
            raise ValueError("nonfinite training loss")
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
        history.append(float(loss.detach()))
        if (step + 1) % 100 == 0:
            print(
                {"step": step + 1, "loss_mean100": float(np.mean(history[-100:]))},
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
            "format": "pour-temporal-decoder-v1",
            "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
            "source_sha256": pm["source_sha256"],
            "train_ids": pm["train_ids"],
            "heldout_containers": list(p.HELDOUT),
            "head_checkpoint_sha256": hm["checkpoint_sha256"],
            "parent_control_checkpoint_sha256": pm["checkpoint_sha256"],
            "steps": 1200,
            "seed": 53,
            "trainable_parameters": sum(
                v.numel() for v in model.parameters() if v.requires_grad
            ),
            "loss_mean_last100": float(np.mean(history[-100:])),
            "terms": provenance["terms"],
        },
    )
    render(output, output / "first-audition", controls_for(), device=device)
    evaluate(model.eval(), base, rows, output)


def single_record_probe(source, directory, output, device="cuda"):
    """Fit the first training recording only to distinguish fit from transfer."""
    torch.set_num_threads(4)
    model, meta = load(directory, device)
    rows, _ = p.load_source(source)
    if (
        meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("probe source mismatch")
    row = next(r for r in rows if r["role"] == "train")
    if row["item_id"] not in meta["train_ids"]:
        raise ValueError("probe must use a training recording")
    output = fresh_output(output)
    phases = [compare.phase_crop(row, phase) for phase in ("first", "middle")]
    c = torch.tensor(np.stack([h.grid(x[1], TIMES) for x in phases]), device=device)
    target = torch.tensor(np.stack([x[2] for x in phases]), device=device)
    records = []

    def capture(kind, mode="temporal"):
        for i, (_, control, real, _) in enumerate(phases):
            for seed in (314, 2718, 1618):
                wave = sample(model, control, seed, mode)
                records.append(
                    {
                        "kind": kind,
                        "phase": ("first", "middle")[i],
                        "seed": seed,
                        **h.save_wav(output / f"{i}-{kind}-{seed}.wav", wave),
                        **p.metrics(wave, real),
                    }
                )

    capture("before")
    for i, (_, _, real, _) in enumerate(phases):
        records.append(
            {
                "kind": "real",
                "phase": ("first", "middle")[i],
                "seed": None,
                **h.save_wav(output / f"{i}-real.wav", real),
            }
        )
    torch.manual_seed(53)
    model.train()  # Loaded checkpoints are eval-mode; cuDNN GRU backward needs train.
    optimizer = torch.optim.AdamW(
        [v for v in model.parameters() if v.requires_grad], lr=1e-3, weight_decay=1e-4
    )
    history = []
    for step in range(300):
        loss = objective(model(c, torch.randn_like(target)), target)
        if not torch.isfinite(loss):
            raise ValueError("nonfinite probe loss")
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
        history.append(float(loss.detach()))
        if (step + 1) % 100 == 0:
            print(
                {
                    "single_record_step": step + 1,
                    "loss_mean100": float(np.mean(history[-100:])),
                },
                flush=True,
            )
    capture("after")
    capture("static", "static")
    checkpoint = output / "model.safetensors"
    save_file(
        {k: v.detach().cpu().contiguous() for k, v in model.state_dict().items()},
        checkpoint,
    )
    p.save_json(
        output / "model.json",
        {
            **meta,
            "scope": "single-training-record memorization diagnostic, NOT new-condition validation",
            "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
            "probe_parent_sha256": meta["checkpoint_sha256"],
            "inherited_train_ids": meta["train_ids"],
            "train_ids": [row["item_id"]],
            "steps": 300,
            "loss_mean_last100": float(np.mean(history[-100:])),
        },
    )
    p.save_json(
        output / "result.json",
        {
            "status": "complete",
            "scope": "first source-order TRAINING recording/two phases; no transfer claim",
            "item_id": row["item_id"],
            "container_id": row["container_id"],
            "rows": records,
        },
    )


def evaluate(model, base, source_rows, output):
    report = {
        "status": "running",
        "scope": "two disclosed development objects, first source-order recording each; not independent test",
        "reference_audio_input_to_generator": False,
        "rows": [],
        "novel": [],
    }
    selected = {}
    for row in source_rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    for i, row in enumerate(selected.values()):
        for phase in ("first", "middle"):
            _, c, real, _ = compare.phase_crop(row, phase)
            pending = [("real", None, real)]
            for seed in (314, 2718, 1618):
                pending.append(("base", seed, p.decode(p.sample(base, c, seed), 314)))
                pending.extend(
                    (mode, seed, sample(model, c, seed, mode))
                    for mode in ("temporal", "static", "no_resonance")
                )
            for kind, seed, wave in pending:
                report["rows"].append(
                    {
                        "container_id": row["container_id"],
                        "item_id": row["item_id"],
                        "phase": phase,
                        "kind": kind,
                        "seed": seed,
                        **h.save_wav(
                            output / f"dev-{i}-{phase}-{kind}-{seed}.wav", wave
                        ),
                        **p.metrics(wave, real),
                    }
                )
    preview = []
    for name, height, material, duration in [
        ("glass10", 10, "glass", 15),
        ("glass16", 16, "glass", 15),
        ("pet10", 10, "plastic_pet", 15),
        ("glass10fast", 10, "glass", 8),
    ]:
        c = controls_for(height, material, duration)
        for seed in (314, 2718, 1618):
            sounds = {
                "base": p.decode(p.sample(base, c, seed), 314),
                **{
                    mode: sample(model, c, seed, mode)
                    for mode in ("temporal", "static")
                },
            }
            for kind, wave in sounds.items():
                report["novel"].append(
                    {
                        "profile": name,
                        "controls": c.tolist(),
                        "kind": kind,
                        "seed": seed,
                        **h.save_wav(output / f"{name}-{kind}-{seed}.wav", wave),
                    }
                )
                if seed == 2718:
                    preview.extend([wave, np.zeros(p.RATE // 2)])
    report["comparison"] = {
        **h.save_wav(output / "comparison.wav", np.concatenate(preview)),
        "order": "glass10/glass16/PET10/glass10fast; base/temporal/static; seed2718; gain1",
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
    parser.add_argument("--output", type=Path, required=True)
    for name in ("source", "parent", "head", "model", "probe-model"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--controls", type=float, nargs=11)
    parser.add_argument("--seed", type=int, default=2718)
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cuda")
    args = parser.parse_args()
    if args.probe_model:
        if (
            not args.source
            or any((args.model, args.parent, args.head, args.controls))
            or args.seed != 2718
        ):
            parser.error("probe requires source/probe-model and no other mode inputs")
        single_record_probe(args.source, args.probe_model, args.output, args.device)
    elif args.model:
        if args.controls is None or args.source or args.parent or args.head:
            parser.error("render requires controls and excludes training inputs")
        render(
            args.model,
            args.output,
            np.asarray(args.controls, dtype=np.float32),
            args.seed,
            args.device,
        )
    else:
        if (
            not all((args.source, args.parent, args.head))
            or args.controls is not None
            or args.seed != 2718
        ):
            parser.error(
                "training requires source/parent/head; controls/seed overrides are render-only"
            )
        run(args.source, args.parent, args.head, args.output, args.device)
