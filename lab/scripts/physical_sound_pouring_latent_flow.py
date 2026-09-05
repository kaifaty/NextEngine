"""Object/event-conditioned latent flow with a frozen Oobleck decoder.

Powered by Stability AI; TangoFlux / Hung et al. Report-only local research.
No source waveform is needed by render/sample. Dataset terms remain unspecified.
"""

import argparse
import hashlib
import json
import shutil
from pathlib import Path

import numpy as np
import physical_sound_pouring_wave_codec as c
import torch
from safetensors.torch import load_file, save_file
from torch import nn
from torch.nn import functional as F

STEPS = 2000


def gaussian_velocity(x, t):
    """Unit-Gaussian marginal field; a baseline, not the true data field."""
    coefficient = (2 * t - 1) / ((1 - t).square() + t.square())
    return x * coefficient[:, None, None]


class Block(nn.Module):
    def __init__(self, width):
        super().__init__()
        self.norm1, self.norm2 = nn.GroupNorm(8, width), nn.GroupNorm(8, width)
        self.conv1, self.conv2 = (
            nn.Conv1d(width, width, 3, padding=1),
            nn.Conv1d(width, width, 3, padding=1),
        )
        self.affine = nn.Linear(128, width * 2)

    def forward(self, x, context):
        scale, shift = self.affine(context)[:, :, None].chunk(2, 1)
        value = self.conv1(F.silu(self.norm1(x)))
        return x + self.conv2(F.silu(self.norm2(value) * (1 + scale) + shift))


class LatentFlow(nn.Module):
    def __init__(self, gaussian_skip=False):
        super().__init__()
        self.gaussian_skip = gaussian_skip
        self.context = nn.Sequential(nn.Linear(20, 128), nn.SiLU(), nn.Linear(128, 128))
        self.input = nn.Conv1d(65, 64, 3, padding=1)
        self.blocks = nn.ModuleList([Block(w) for w in (64, 128, 192, 128, 64)])
        self.down1, self.down2 = (
            nn.Conv1d(64, 128, 4, stride=2, padding=1),
            nn.Conv1d(128, 192, 4, stride=2, padding=1),
        )
        self.up1, self.up2 = (
            nn.Conv1d(192, 128, 3, padding=1),
            nn.Conv1d(128, 64, 3, padding=1),
        )
        self.output = nn.Conv1d(64, 64, 3, padding=1)
        self.register_buffer("center", torch.zeros(1, 64, 1))
        self.register_buffer("scale", torch.ones(1, 64, 1))

    def forward(self, x, t, controls):
        angle = t[:, None] * torch.tensor([1, 2, 4, 8], device=x.device) * (2 * np.pi)
        context = self.context(
            torch.cat([controls, t[:, None], angle.sin(), angle.cos()], 1)
        )
        position = torch.linspace(-1, 1, c.LATENT_FRAMES, device=x.device)[
            None, None
        ].expand(len(x), 1, -1)
        first = self.blocks[0](self.input(torch.cat([x, position], 1)), context)
        second = self.blocks[1](self.down1(first), context)
        third = self.blocks[2](self.down2(second), context)
        value = self.blocks[3](
            self.up1(
                F.interpolate(
                    third, size=second.shape[-1], mode="linear", align_corners=False
                )
            )
            + second,
            context,
        )
        value = self.blocks[4](
            self.up2(
                F.interpolate(
                    value, size=first.shape[-1], mode="linear", align_corners=False
                )
            )
            + first,
            context,
        )
        result = self.output(value)
        return result + gaussian_velocity(x, t) if self.gaussian_skip else result


def normalization(mean, std):
    if (
        mean.shape != std.shape
        or mean.ndim != 3
        or mean.shape[1:] != (64, c.LATENT_FRAMES)
        or not torch.isfinite(mean).all()
        or not torch.isfinite(std).all()
        or (std < 0).any()
    ):
        raise ValueError("invalid training posterior")
    center = mean.mean((0, 2), keepdim=True)
    scale = (
        ((mean - center).square() + std.square())
        .mean((0, 2), keepdim=True)
        .sqrt()
        .clamp_min(0.1)
    )
    return center, scale


@torch.inference_mode()
def cache(source, output, device="cuda"):
    torch.set_num_threads(4)
    rows, provenance = c.v.p.load_source(source)
    codec, model_root = c.load_codec(device)
    output = c.v.phase.d.fresh_output(output)
    means, stds, controls, records = [], [], [], []
    training = [r for r in rows if r["role"] == "train"]
    for index, row in enumerate(training):
        last = max(0, row["spectrogram"].shape[1] - c.v.p.FRAMES)
        for part, start in [("first", 0), ("middle", last // 2), ("last", last)]:
            _, control = c.v.p.crop(row, start)
            wave = row["wave"][start * c.v.p.HOP : start * c.v.p.HOP + c.v.p.SAMPLES]
            wave = np.pad(wave, (0, c.v.p.SAMPLES - len(wave)))
            posterior = codec.encode(
                torch.tensor(c.input_wave(wave)[None], device=device)
            ).latent_dist
            means.append(posterior.mean[0].cpu())
            stds.append(posterior.std[0].cpu())
            controls.append(control)
            records.append(
                {
                    "item_id": row["item_id"],
                    "container_id": row["container_id"],
                    "phase": part,
                    "start_frame16k": start,
                }
            )
        if (index + 1) % 10 == 0:
            print({"encoded_training_records": index + 1}, flush=True)
    tensors = {
        "mean": torch.stack(means),
        "std": torch.stack(stds),
        "controls": torch.tensor(np.stack(controls)),
    }
    normalization(tensors["mean"], tensors["std"])
    path = output / "posterior.safetensors"
    save_file(tensors, path)
    for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
        shutil.copyfile(model_root / name, output / name)
    c.v.p.save_json(
        output / "cache.json",
        {
            "format": "pour-oobleck-cache-v1",
            "codec_sha256": c.CODEC_SHA,
            "source_sha256": hashlib.sha256(
                (source / "source.json").read_bytes()
            ).hexdigest(),
            "posterior_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "train_ids": [r["item_id"] for r in training],
            "terms": provenance["terms"],
            "rows": records,
        },
    )


def fit(cache_root, output, device="cuda", gaussian_skip=False, zero_output=False):
    torch.set_num_threads(4)
    meta = json.loads((cache_root / "cache.json").read_text())
    path = cache_root / "posterior.safetensors"
    if (
        meta["format"] != "pour-oobleck-cache-v1"
        or meta["codec_sha256"] != c.CODEC_SHA
        or path.stat().st_size > 30_000_000
        or hashlib.sha256(path.read_bytes()).hexdigest() != meta["posterior_sha256"]
    ):
        raise ValueError("cache integrity failure")
    data = load_file(path, device=device)
    center, scale = normalization(data["mean"], data["std"])
    if (
        len(meta["rows"]) != len(data["mean"])
        or {r["item_id"] for r in meta["rows"]} != set(meta["train_ids"])
        or any(r["container_id"] in c.v.p.HELDOUT for r in meta["rows"])
    ):
        raise ValueError("training cache roles mismatch")
    if (
        data["controls"].shape != (len(data["mean"]), 11)
        or not torch.isfinite(data["controls"]).all()
    ):
        raise ValueError("invalid training controls")
    output = c.v.phase.d.fresh_output(output)
    torch.manual_seed(53)
    model = LatentFlow(gaussian_skip=gaussian_skip).to(device)
    if zero_output:
        nn.init.zeros_(model.output.weight)
        nn.init.zeros_(model.output.bias)
    model.center.copy_(center)
    model.scale.copy_(scale)
    optimizer = torch.optim.AdamW(model.parameters(), lr=3e-4, weight_decay=1e-4)
    losses = []
    for step in range(STEPS):
        index = torch.randint(len(data["mean"]), (16,), device=device)
        mean, std = data["mean"][index], data["std"][index]
        target = (mean + std * torch.randn_like(mean) - center) / scale
        noise = torch.randn_like(target)
        t = torch.rand(len(target), device=device)
        x = noise * (1 - t[:, None, None]) + target * t[:, None, None]
        loss = F.mse_loss(model(x, t, data["controls"][index]), target - noise)
        if not torch.isfinite(loss):
            raise ValueError("nonfinite flow loss")
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
        losses.append(float(loss.detach()))
        if (step + 1) % 200 == 0:
            print(
                {"step": step + 1, "mean200_loss": float(np.mean(losses[-200:]))},
                flush=True,
            )
    checkpoint = output / "model.safetensors"
    save_file(
        {k: x.detach().cpu().contiguous() for k, x in model.state_dict().items()},
        checkpoint,
    )
    for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
        shutil.copyfile(cache_root / name, output / name)
    c.v.p.save_json(
        output / "model.json",
        {
            "format": "pour-latent-flow-gaussian-v1"
            if gaussian_skip
            else "pour-latent-flow-v1",
            "zero_initialized_output": zero_output,
            "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
            "codec_sha256": c.CODEC_SHA,
            "cache_sha256": hashlib.sha256(
                (cache_root / "cache.json").read_bytes()
            ).hexdigest(),
            "source_sha256": meta["source_sha256"],
            "train_ids": meta["train_ids"],
            "terms": meta["terms"],
            "steps": STEPS,
            "seed": 53,
            "parameters": sum(p.numel() for p in model.parameters()),
            "mean200_loss": float(np.mean(losses[-200:])),
            "scope": "frozen waveform codec, controls-to-latent flow; no physical calibration claim",
        },
    )
    render(output, output / "first-audition", c.v.phase.d.controls_for(), device=device)


def load(directory, device="cuda"):
    path = directory / "model.safetensors"
    mp = directory / "model.json"
    if path.stat().st_size > 32_000_000 or mp.stat().st_size > 100000:
        raise ValueError("oversized model")
    meta = json.loads(mp.read_text())
    if (
        meta["format"] not in ("pour-latent-flow-v1", "pour-latent-flow-gaussian-v1")
        or meta["codec_sha256"] != c.CODEC_SHA
        or hashlib.sha256(path.read_bytes()).hexdigest() != meta["checkpoint_sha256"]
    ):
        raise ValueError("model identity failure")
    model = LatentFlow(gaussian_skip=meta["format"] == "pour-latent-flow-gaussian-v1")
    state = load_file(path)
    if (
        not all(torch.isfinite(x).all() for x in state.values())
        or (state["scale"] <= 0).any()
    ):
        raise ValueError("invalid model tensors")
    model.load_state_dict(state, strict=True)
    return model.to(device).eval(), meta


@torch.inference_mode()
def integrate(model, x, controls, start=0):
    if not isinstance(start, int) or not 0 <= start <= 64:
        raise ValueError("invalid flow start")
    for step in range(start, 64):
        x = (
            x
            + model(x, torch.full((len(x),), step / 64, device=x.device), controls) / 64
        )
    if not torch.isfinite(x).all():
        raise ValueError("nonfinite normalized latent")
    return x


@torch.inference_mode()
def sample(model, controls, seed=2718):
    if not isinstance(seed, int) or not 0 <= seed < 2**32:
        raise ValueError("invalid seed")
    c.v.h.grid(controls, np.array([0]))
    device = next(model.parameters()).device
    controls = torch.tensor(np.asarray(controls, dtype=np.float32)[None], device=device)
    x = torch.randn(
        (1, 64, c.LATENT_FRAMES),
        generator=torch.Generator(device=device).manual_seed(seed),
        device=device,
    )
    x = integrate(model, x, controls)
    x = x * model.scale + model.center
    if not torch.isfinite(x).all():
        raise ValueError("nonfinite latent")
    return x


@torch.inference_mode()
def render(directory, output, controls, seed=2718, device="cuda"):
    torch.set_num_threads(4)
    model, meta = load(directory, device)
    codec, _ = c.load_codec(device)
    native = codec.decode(sample(model, controls, seed)).sample[0].cpu().numpy()
    wave = c.mono_wave(native)
    output = c.v.phase.d.fresh_output(output)
    row = c.v.h.save_wav(output / "generated.wav", wave)
    c.v.p.save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "model_sha256": meta["checkpoint_sha256"],
            "codec_sha256": c.CODEC_SHA,
            "controls": controls.tolist(),
            "seed": seed,
            "rows": [row],
        },
    )
    print({"primary_artifact": row["wav"]}, flush=True)


@torch.inference_mode()
def evaluate(source, directory, base_root, output, device="cuda"):
    torch.set_num_threads(4)
    rows, _ = c.v.p.load_source(source)
    model, meta = load(directory, device)
    base, bm = c.compare.load_model(base_root, device)
    source_sha = hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    train_ids = [r["item_id"] for r in rows if r["role"] == "train"]
    if any(
        m["source_sha256"] != source_sha or m["train_ids"] != train_ids
        for m in (meta, bm)
    ):
        raise ValueError("evaluation roles/source mismatch")
    codec, _ = c.load_codec(device)
    output = c.v.phase.d.fresh_output(output)
    selected = {"train-first": next(r for r in rows if r["role"] == "train")}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    records, preview = [], []

    def generate(kind, controls, seed):
        if kind == "base":
            return c.v.p.decode(c.v.p.sample(base, controls, seed), 314)
        native = codec.decode(sample(model, controls, seed)).sample[0].cpu().numpy()
        return c.mono_wave(native)

    def record(info, wave, reference=None):
        row = {
            **info,
            **c.v.h.save_wav(output / f"{len(records):03d}-{info['kind']}.wav", wave),
        }
        if reference is not None:
            row.update(c.v.p.metrics(wave, reference))
        records.append(row)

    for name, source_row in selected.items():
        for part in ("first", "middle"):
            _, controls, real, _ = c.compare.phase_crop(source_row, part)
            info = {
                "object": name,
                "item_id": source_row["item_id"],
                "phase": part,
                "controls": controls.tolist(),
                "reference_audio_input": False,
            }
            record({**info, "kind": "real", "seed": None}, real, real)
            for seed in (314, 2718, 1618):
                for kind in ("base", "latent"):
                    info_row = {**info, "kind": kind, "seed": seed}
                    try:
                        wave = generate(kind, controls, seed)
                        record(info_row, wave, real)
                    except ValueError as error:
                        records.append({**info_row, "failure": str(error)})
        print({"evaluated": name}, flush=True)
    for name, height, material, duration in [
        ("glass10", 10, "glass", 15),
        ("glass16", 16, "glass", 15),
        ("pet10", 10, "plastic_pet", 15),
        ("glass10fast", 10, "glass", 8),
    ]:
        controls = c.v.phase.d.controls_for(height, material, duration)
        for seed in (314, 2718, 1618):
            for kind in ("base", "latent"):
                info = {
                    "profile": name,
                    "kind": kind,
                    "seed": seed,
                    "controls": controls.tolist(),
                    "reference_audio_input": False,
                }
                try:
                    wave = generate(kind, controls, seed)
                    record(info, wave)
                    if seed == 2718:
                        preview.extend([wave, np.zeros(c.v.p.RATE // 2)])
                except ValueError as error:
                    records.append({**info, "failure": str(error)})
    failures = sum("failure" in row for row in records)
    report = {
        "status": "complete",
        "scope": "one training record, two disclosed development objects, four hypothetical profiles; not independent test",
        "model_sha256": meta["checkpoint_sha256"],
        "codec_sha256": c.CODEC_SHA,
        "rows": records,
        "failed_outputs": failures,
    }
    if not failures:
        report["comparison"] = {
            **c.v.h.save_wav(output / "comparison.wav", np.concatenate(preview)),
            "order": "glass10/glass16/PET10/glass10fast; base/latent; seed2718/gain1",
        }
    c.v.p.save_json(output / "result.json", report)
    valid = [r for r in records if "wav" in r]
    c.v.p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": 4.08,
            "cases": [
                {"id": f"{i}-{r['kind']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(valid)
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(valid)],
            "controls": [],
        },
    )


@torch.inference_mode()
def trajectory_probe(source, directory, output, device="cuda"):
    """Privileged partial-path diagnostic; only start0 is reference-free."""
    torch.set_num_threads(4)
    rows, _ = c.v.p.load_source(source)
    model, meta = load(directory, device)
    if (
        meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("source mismatch")
    codec, _ = c.load_codec(device)
    output = c.v.phase.d.fresh_output(output)
    selected = {"train-first": next(r for r in rows if r["role"] == "train")}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    records, preview = [], []
    for name, row in selected.items():
        for part in ("first", "middle"):
            _, controls, real, _ = c.compare.phase_crop(row, part)
            control = torch.tensor(controls[None], device=device)
            posterior = codec.encode(
                torch.tensor(c.input_wave(real)[None], device=device)
            ).latent_dist
            for seed in (314, 2718, 1618):
                target = posterior.sample(
                    generator=torch.Generator(device=device).manual_seed(seed + 1000)
                )
                target = (target - model.center) / model.scale
                noise = torch.randn(
                    target.shape,
                    generator=torch.Generator(device=device).manual_seed(seed),
                    device=device,
                )
                for start in (0, 32, 56, 64):
                    initial = (1 - start / 64) * noise + start / 64 * target
                    latent = (
                        integrate(model, initial, control, start) * model.scale
                        + model.center
                    )
                    wave = c.mono_wave(codec.decode(latent).sample[0].cpu().numpy())
                    records.append(
                        {
                            "object": name,
                            "item_id": row["item_id"],
                            "phase": part,
                            "seed": seed,
                            "start_step": start,
                            "reference_audio_input": start != 0,
                            **c.v.h.save_wav(
                                output / f"{name}-{part}-{seed}-{start}.wav", wave
                            ),
                            **c.v.p.metrics(wave, real),
                        }
                    )
                    if seed == 2718 and part == "middle":
                        preview.extend([wave, np.zeros(c.v.p.RATE // 2)])
        print({"trajectory_probed": name}, flush=True)
    c.v.p.save_json(
        output / "result.json",
        {
            "status": "complete",
            "scope": "partial flow paths start with privileged target audio except start0; no generalization claim",
            "model_sha256": meta["checkpoint_sha256"],
            "rows": records,
            "comparison": {
                **c.v.h.save_wav(output / "comparison.wav", np.concatenate(preview)),
                "order": "train-first/glass18/PET30 middle; start0/.5/.875/1; seed2718; gain1",
            },
        },
    )
    c.v.p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": 4.08,
            "cases": [
                {"id": f"{i}-start{r['start_step']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(records)
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(records)],
            "controls": [],
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    for mode, paths in [
        ("cache", ("source",)),
        ("train", ("cache",)),
        ("render", ("model",)),
        ("evaluate", ("source", "model", "base")),
        ("probe", ("source", "model")),
    ]:
        command = sub.add_parser(mode)
        for name in paths + ("output",):
            command.add_argument("--" + name, type=Path, required=True)
        command.add_argument("--device", choices=("cuda", "cpu"), default="cuda")
        if mode == "train":
            command.add_argument("--gaussian-skip", action="store_true")
            command.add_argument("--zero-output", action="store_true")
        if mode == "render":
            command.add_argument("--controls", type=float, nargs=11, required=True)
            command.add_argument("--seed", type=int, default=2718)
    args = parser.parse_args()
    if args.mode == "cache":
        cache(args.source, args.output, args.device)
    elif args.mode == "train":
        fit(args.cache, args.output, args.device, args.gaussian_skip, args.zero_output)
    elif args.mode == "evaluate":
        evaluate(args.source, args.model, args.base, args.output, args.device)
    elif args.mode == "probe":
        trajectory_probe(args.source, args.model, args.output, args.device)
    else:
        render(
            args.model,
            args.output,
            np.array(args.controls, dtype=np.float32),
            args.seed,
            args.device,
        )
