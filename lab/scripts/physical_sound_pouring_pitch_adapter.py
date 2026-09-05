"""Learn a small trajectory-conditioned decoder adapter; keep base weights frozen.

Full-record teacher curves enter training only. Inference uses the stored
condition-to-resonance head, without any reference recording or teacher model.
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
import soundfile as sf
import torch
from safetensors.torch import load_file, save_file
from torch import nn
from torch.nn import functional as F


def plane(hz):
    if (
        hz.ndim != 2
        or hz.shape[1] != 256
        or not torch.isfinite(hz).all()
        or (hz <= 0).any()
    ):
        raise ValueError("invalid resonance trajectory")
    frequencies = torch.arange(1, 257, device=hz.device) * (p.RATE / p.FFT)
    cents = 1200 * torch.log2(frequencies[None, :, None] / hz[:, None, :])
    return torch.exp(-0.5 * (cents / 150).square())[:, None]


class PitchAdapter(p.PourFlow):
    def __init__(self):
        super().__init__()
        self.requires_grad_(False)
        self.pitch_adapter = nn.Conv2d(1, 16, 3, padding=1, bias=False)
        nn.init.zeros_(self.pitch_adapter.weight)

    def forward(self, x, t, controls, guide):
        angle = t[:, None] * torch.tensor([1, 2, 4, 8], device=x.device) * (2 * np.pi)
        context = self.context(
            torch.cat([controls, t[:, None], angle.sin(), angle.cos()], 1)
        )
        axis = torch.linspace(-1, 1, 256, device=x.device)
        frequency = axis[None, None, :, None].expand(len(x), 1, 256, 256)
        position = axis[None, None, None, :].expand(len(x), 1, 256, 256)
        a = self.blocks[0](
            self.input(torch.cat([x, frequency, position], 1))
            + self.pitch_adapter(guide),
            context,
        )
        b = self.blocks[1](self.down1(a), context)
        c = self.blocks[2](self.down2(b), context)
        d = self.blocks[3](self.up1(F.interpolate(c, scale_factor=2)) + b, context)
        e = self.blocks[4](self.up2(F.interpolate(d, scale_factor=2)) + a, context)
        return self.output(F.silu(e))


def from_base(base):
    model = PitchAdapter().to(next(base.parameters()).device)
    result = model.load_state_dict(base.state_dict(), strict=False)
    if result.missing_keys != ["pitch_adapter.weight"] or result.unexpected_keys:
        raise ValueError("unexpected base layout")
    return model


def objective(prediction, target, noise, time):
    weight = time[:, None, None, None]
    mixed = (1 - weight) * noise + weight * target
    endpoint = mixed + (1 - weight) * prediction
    reconstruction = (
        sum(
            (F.avg_pool2d(endpoint, k) - F.avg_pool2d(target, k)).abs().mean((1, 2, 3))
            for k in (1, 4, 16)
        )
        / 3
    )
    return (
        F.mse_loss(prediction, target - noise)
        + 0.25 * (time.square() * reconstruction).mean()
    )


@torch.inference_mode()
def sample(model, controls, hz, seed=2718, steps=64):
    if (
        not isinstance(seed, int)
        or not 0 <= seed < 2**32
        or not isinstance(steps, int)
        or not 1 <= steps <= 1024
    ):
        raise ValueError("invalid sampling seed or step count")
    device = next(model.parameters()).device
    rng = torch.Generator(device=device).manual_seed(seed)
    x = torch.randn((1, 1, 256, 256), generator=rng, device=device)
    c = torch.from_numpy(controls[None]).to(device)
    guide = plane(torch.tensor(hz[None], dtype=torch.float32, device=device))
    for step in range(steps):
        t = torch.full((1,), step / steps, device=device)
        x += model(x, t, c, guide) / steps
    result = x[0, 0].cpu().numpy()
    if not np.isfinite(result).all():
        raise ValueError("invalid decoder sample")
    return result.clip(-2, 2)


def load(directory, parent_meta, base):
    metadata = directory / "model.json"
    if metadata.stat().st_size > 100000:
        raise ValueError("oversized adapter metadata")
    meta = json.loads(metadata.read_text())
    path = directory / "adapter.safetensors"
    if (
        path.stat().st_size > 10000
        or meta["format"] != "pour-pitch-adapter-v1"
        or meta["parent_checkpoint_sha256"] != parent_meta["checkpoint_sha256"]
        or hashlib.sha256(path.read_bytes()).hexdigest() != meta["checkpoint_sha256"]
    ):
        raise ValueError("adapter identity mismatch")
    state = load_file(path)
    if set(state) != {"weight"} or not torch.isfinite(state["weight"]).all():
        raise ValueError("invalid adapter tensors")
    model = from_base(base)
    model.pitch_adapter.load_state_dict(state, strict=True)
    return model.eval(), meta


def render(parent, head, adapter, output, controls, seed=2718, device="cuda"):
    output = output.resolve()
    if (
        output.exists()
        or output.is_relative_to(Path(__file__).resolve().parents[2])
        or not 0 <= seed < 2**32
    ):
        raise ValueError("fresh external output and valid seed required")
    base, pm = compare.load_model(parent, device)
    trajectory, hm = h.load_head(head, pm, device)
    model, meta = load(adapter, pm, base)
    if meta["head_checkpoint_sha256"] != hm["checkpoint_sha256"]:
        raise ValueError("wrong trajectory head")
    c = h.grid(controls, np.arange(256) * p.HOP / p.RATE)
    hz = h.predict(trajectory, c)
    sounds = {
        "base": p.decode(p.sample(base, controls, seed), 314),
        "adapter": p.decode(sample(model, controls, hz, seed), 314),
    }
    if max(abs(w).max() for w in sounds.values()) > 0.98:
        raise ValueError("unsafe raw output")
    output.mkdir(parents=True)
    rows = [
        {"kind": k, "seed": seed, **h.save_wav(output / (k + ".wav"), wave)}
        for k, wave in sounds.items()
    ]
    p.save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "teacher_required_at_inference": False,
            "controls": controls.tolist(),
            "trajectory_hz": hz.tolist(),
            "decoder_seed": 314,
            "adapter": meta,
            "rows": rows,
        },
    )
    return rows


def fit(base, spectra, controls, curves, shuffled, output, common):
    if output.exists():
        raise ValueError("fresh fit directory required")
    torch.manual_seed(53)
    model = from_base(base)
    optimizer = torch.optim.AdamW(
        model.pitch_adapter.parameters(), lr=1e-3, weight_decay=0.01
    )
    rng = np.random.default_rng(53)
    y = torch.tensor(spectra[:, None], device="cuda")
    c = torch.tensor(controls, device="cuda")
    guides = plane(torch.tensor(curves, dtype=torch.float32, device="cuda"))
    history = []
    for step in range(600):
        indices = (
            rng.integers(len(spectra) // 2, size=(3, 1)) * 2 + np.arange(2)
        ).reshape(-1)
        permutation = rng.permutation(6)
        target = y[indices]
        noise = torch.randn_like(target)
        time = torch.rand(6, device="cuda")
        mixed = (1 - time[:, None, None, None]) * noise + time[
            :, None, None, None
        ] * target
        guide = guides[indices]
        if shuffled:
            guide = guide[permutation]
        prediction = model(mixed, time, c[indices], guide)
        loss = objective(prediction, target, noise, time)
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.pitch_adapter.parameters(), 1)
        optimizer.step()
        history.append(float(loss.detach()))
        if (step + 1) % 100 == 0:
            print(
                {
                    "shuffled": shuffled,
                    "step": step + 1,
                    "loss_mean100": float(np.mean(history[-100:])),
                },
                flush=True,
            )
    # GroupNorm has no mutable running statistics; every parent tensor is unchanged.
    if not all(
        torch.equal(model.state_dict()[k], v) for k, v in base.state_dict().items()
    ):
        raise ValueError("frozen base changed")
    output.mkdir(parents=True)
    path = output / "adapter.safetensors"
    save_file(
        {
            k: v.detach().cpu().contiguous()
            for k, v in model.pitch_adapter.state_dict().items()
        },
        path,
    )
    p.save_json(
        output / "model.json",
        {
            **common,
            "format": "pour-pitch-adapter-v1",
            "checkpoint_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "shuffled_curves": shuffled,
            "steps": 600,
            "seed": 53,
            "parameters": 144,
            "loss_mean_last100": float(np.mean(history[-100:])),
        },
    )
    return model.eval()


def run(source, teacher_probe, parent, head, output):
    source, teacher_probe, parent, head, output = (
        x.resolve() for x in (source, teacher_probe, parent, head, output)
    )
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    training = compare.select_rows(rows, True)
    teacher = json.loads((teacher_probe / "result.json").read_text())
    references = {
        r["id"]: r
        for r in teacher["rows"]
        if r["variant"] == "original" and r["id"].startswith("container_")
    }
    if set(references) != {r["container_id"] for r in training}:
        raise ValueError("teacher roster mismatch")
    base, pm = compare.load_model(parent, "cuda")
    trajectory, hm = h.load_head(head, pm)
    if (
        pm["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    ):
        raise ValueError("wrong source")
    spectra = []
    controls = []
    curves = []
    for row in training:
        ref = references[row["container_id"]]
        path = Path(ref["input_wav"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != ref["input_sha256"]:
            raise ValueError("teacher input mismatch")
        wave, sr = sf.read(path, dtype="float32")
        if (
            sr != p.RATE
            or wave.shape != row["wave"].shape
            or not np.allclose(wave, row["wave"], atol=3.1e-5, rtol=0)
        ):
            raise ValueError("teacher/source alignment mismatch")
        for phase in ["first", "middle"]:
            spec, c, _, offset = compare.phase_crop(row, phase)
            spectra.append(spec)
            controls.append(c)
            curves.append(
                np.interp(
                    offset / p.RATE + np.arange(256) * p.HOP / p.RATE,
                    ref["time_seconds"],
                    ref["axial_hz"],
                )
            )
    common = {
        "parent_checkpoint_sha256": pm["checkpoint_sha256"],
        "head_checkpoint_sha256": hm["checkpoint_sha256"],
        "source_sha256": pm["source_sha256"],
        "teacher_result_sha256": hashlib.sha256(
            (teacher_probe / "result.json").read_bytes()
        ).hexdigest(),
        "adapter_train_ids": [r["item_id"] for r in training],
        "inherited_train_ids": pm["train_ids"],
        "scope": "13 training recordings/two phases; uncertain teacher labels, inherited93-record base",
    }
    output.mkdir(parents=True)
    models = {}
    previews = []
    for kind in ["matched", "shuffled"]:
        models[kind] = fit(
            base,
            np.stack(spectra),
            np.stack(controls),
            np.stack(curves),
            kind == "shuffled",
            output / kind,
            common,
        )
        c = p.condition(
            {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7},
            "glass",
            "cylindrical",
            15,
            0.1,
        )
        previews.append(
            {
                "kind": kind,
                "rows": render(
                    parent, head, output / kind, output / (kind + "-first-audition"), c
                ),
            }
        )
        print({"primary_artifact": previews[-1]["rows"][1]["wav"]}, flush=True)
    report = {
        "status": "running",
        "scope": "two disclosed objects, first source-order recording each, two phases, three seeds; not independent test",
        "terms": provenance["terms"],
        "reference_audio_input_to_generator": False,
        "previews": previews,
        "rows": [],
        "novel": [],
    }
    # Four fixed source-free profiles; same seeds and conditions as the head experiment.
    for name, height, material, duration in [
        ("glass10", 10, "glass", 15),
        ("glass16", 16, "glass", 15),
        ("pet10", 10, "plastic_pet", 15),
        ("glass10fast", 10, "glass", 8),
    ]:
        c = p.condition(
            {"net_height": height, "diameter_top": 7, "diameter_bottom": 7},
            material,
            "cylindrical",
            duration,
            0.1,
        )
        hz = h.predict(trajectory, h.grid(c, np.arange(256) * p.HOP / p.RATE))
        for seed in [314, 2718, 1618]:
            sounds = {
                "base": p.decode(p.sample(base, c, seed), 314),
                **{k: p.decode(sample(m, c, hz, seed), 314) for k, m in models.items()},
            }
            for kind, w in sounds.items():
                report["novel"].append(
                    {
                        "profile": name,
                        "controls": c.tolist(),
                        "kind": kind,
                        "seed": seed,
                        **h.save_wav(output / f"{name}-{kind}-{seed}.wav", w),
                    }
                )
    selected = {}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    for index, row in enumerate(selected.values()):
        for phase in ["first", "middle"]:
            _, c, real, _ = compare.phase_crop(row, phase)
            hz = h.predict(trajectory, h.grid(c, np.arange(256) * p.HOP / p.RATE))
            pending = [("real", None, real)]
            for seed in [314, 2718, 1618]:
                pending.append(("base", seed, p.decode(p.sample(base, c, seed), 314)))
                pending.extend(
                    (k, seed, p.decode(sample(m, c, hz, seed), 314))
                    for k, m in models.items()
                )
            for kind, seed, w in pending:
                report["rows"].append(
                    {
                        "item_id": row["item_id"],
                        "container_id": row["container_id"],
                        "phase": phase,
                        "kind": kind,
                        "seed": seed,
                        **h.save_wav(
                            output / f"dev-{index}-{phase}-{kind}-{seed}.wav", w
                        ),
                        **p.metrics(w, real),
                    }
                )
    preview = []
    for row in report["novel"]:
        if row["seed"] == 2718:
            wave, _ = sf.read(row["wav"], dtype="float32")
            preview.extend([wave, np.zeros(p.RATE // 2)])
    report["comparison"] = {
        **h.save_wav(output / "comparison.wav", np.concatenate(preview)),
        "order": "glass10/glass16/PET10/glass10fast; base/matched/shuffled; seed2718; gain1",
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
    for name in ["parent", "head", "output"]:
        parser.add_argument("--" + name, type=Path, required=True)
    for name in ["source", "teacher-probe", "render-adapter"]:
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--controls", type=float, nargs=11)
    parser.add_argument("--seed", type=int, default=2718)
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cuda")
    args = parser.parse_args()
    if args.render_adapter:
        if args.controls is None or args.source or args.teacher_probe:
            parser.error("render requires controls and excludes source/teacher inputs")
        render(
            args.parent,
            args.head,
            args.render_adapter,
            args.output,
            np.asarray(args.controls, dtype=np.float32),
            args.seed,
            args.device,
        )
    else:
        if not args.source or not args.teacher_probe or args.controls is not None:
            parser.error(
                "training requires source/teacher-probe and no render controls"
            )
        if args.device != "cuda" or args.seed != 2718:
            parser.error("device/seed overrides apply only to render mode")
        run(args.source, args.teacher_probe, args.parent, args.head, args.output)
