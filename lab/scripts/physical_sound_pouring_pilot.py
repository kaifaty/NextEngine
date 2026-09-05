"""Report-only geometry/material/duration conditioned pouring audio.

Bagad et al., Sound of Water. Dataset redistribution terms are unspecified;
local research only. Do not inherit the separate code/model MIT license.
Only publisher training recordings are acquired. No runtime integration.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import pandas as pd
import requests
import soundfile as sf
import torch
from safetensors.torch import load_file, save_file
from scipy.signal import istft, resample_poly, stft
from torch import nn
from torch.nn import functional as F

REVISION = "12575460ee39d6adaebbe5aff531a5f4a24a627b"
REPOSITORY = "bpiyush/sound-of-water"
HELDOUT = ("container_18", "container_30")
MATERIALS = ("glass", "plastic", "plastic_pet", "plastic_pp")
SHAPES = ("cylindrical", "semiconical")
RATE = 16000
FFT = 512
HOP = 256
FRAMES = 256
SAMPLES = (FRAMES - 1) * HOP


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def parse_measurements(value: str) -> dict:
    if not isinstance(value, str) or len(value) > 2048:
        raise ValueError("invalid measurement annotation")
    result = ast.literal_eval(value)
    if not isinstance(result, dict):
        raise TypeError("expected measurement mapping")
    for key in ("net_height", "diameter_top", "diameter_bottom"):
        x = result.get(key)
        if not isinstance(x, (int, float)) or not np.isfinite(x) or not 0 < x < 100:
            raise ValueError("missing or invalid measured centimetres")
    return result


def eligible(frame):
    # Annotation-only selection; do not choose by listening or model performance.
    mask = (
        (frame.flow_rate_appx == "constant")
        & (frame.liquid == "water_normal")
        & (frame.clean == "yes")
        & frame.material.isin(MATERIALS)
        & frame["shape"].isin(SHAPES)
    )
    result = frame[mask].copy()
    if result.item_id.duplicated().any() or result.video_id.duplicated().any():
        raise ValueError("duplicate source recording")
    if not result.item_id.str.fullmatch(
        r"VID_[0-9]{8}_[0-9]{6}_[0-9]+\.[0-9]_[0-9]+\.[0-9]"
    ).all():
        raise ValueError("unexpected source identity")
    for value in result.measurements:
        parse_measurements(value)
    result["role"] = np.where(
        result.container_id.isin(HELDOUT), "unseen_container", "train"
    )
    return result


def acquire(output: Path):
    output.mkdir(parents=True, exist_ok=False)
    api = f"https://huggingface.co/api/datasets/{REPOSITORY}/tree/{REVISION}"
    inventory = []
    for suffix in ("", "/splits", "/audios"):
        response = requests.get(api + suffix, params={"limit": 1000}, timeout=30)
        response.raise_for_status()
        if response.links.get("next"):
            raise ValueError("unexpected paginated inventory")
        inventory.extend(response.json())
    save_json(output / "inventory.json", inventory)
    files = {row["path"]: row for row in inventory if row["type"] == "file"}

    def fetch(name):
        record = files[name]
        if not 0 < record["size"] < 20_000_000:
            raise ValueError("source file outside byte bound")
        url = f"https://huggingface.co/datasets/{REPOSITORY}/resolve/{REVISION}/{name}"
        response = requests.get(url, timeout=(15, 45))
        response.raise_for_status()
        data = response.content
        if len(data) != record["size"]:
            raise ValueError("source byte count mismatch")
        if "lfs" in record:
            digest = hashlib.sha256(data).hexdigest()
            expected = record["lfs"]["oid"]
        else:
            digest = hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()
            expected = record["oid"]
        if digest != expected:
            raise ValueError("publisher source hash mismatch")
        path = output / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return {
            "path": name,
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
            "url": url,
        }

    records = [fetch("README.md"), fetch("splits/train.csv")]
    selected = eligible(pd.read_csv(output / "splits/train.csv"))
    names = [f"audios/{item}.wav" for item in selected.item_id]
    print(
        json.dumps(
            {
                "selected": len(names),
                "bytes": sum(files[name]["size"] for name in names),
                "roles": selected.role.value_counts().to_dict(),
            }
        ),
        flush=True,
    )
    with ThreadPoolExecutor(max_workers=4) as pool:
        for index, result in enumerate(pool.map(fetch, names)):
            records.append(result)
            if (index + 1) % 20 == 0:
                print(
                    json.dumps({"downloaded": index + 1, "total": len(names)}),
                    flush=True,
                )
    save_json(
        output / "source.json",
        {
            "status": "complete",
            "repository": REPOSITORY,
            "revision": REVISION,
            "terms": "dataset redistribution unspecified; local research only",
            "selection": "publisher train; clean=yes, constant, water_normal, supported material/shape",
            "heldout_containers": HELDOUT,
            "files": records,
        },
    )


def condition(measurements, material, shape, duration, progress):
    if material not in MATERIALS or shape not in SHAPES:
        raise ValueError("unsupported material or shape")
    numeric = [
        measurements[k] for k in ("net_height", "diameter_top", "diameter_bottom")
    ]
    if not all(np.isfinite(x) and 0 < x < 100 for x in numeric):
        raise ValueError("invalid geometry")
    if (
        not np.isfinite(duration)
        or not 1 <= duration <= 60
        or not np.isfinite(progress)
        or not 0 <= progress <= 1
    ):
        raise ValueError("invalid event timing")
    return np.array(
        [x / 20 for x in numeric]
        + [duration / 30, progress]
        + [float(material == x) for x in MATERIALS]
        + [float(shape == x) for x in SHAPES],
        dtype=np.float32,
    )


def transform(wave):
    return stft(wave, fs=RATE, nperseg=FFT, noverlap=FFT - HOP)[2]


def encode(wave):
    # Fixed digital scale, not a statistic fitted to evaluation recordings.
    return np.clip(
        (20 * np.log10(np.maximum(abs(transform(wave)[1:]), 1e-5)) + 50) / 25, -2, 2
    ).astype(np.float32)


def decode(value, seed=314, iterations=32):
    if (
        value.shape != (256, FRAMES)
        or not np.isfinite(value).all()
        or not 0 <= seed < 2**32
    ):
        raise ValueError("invalid spectrogram request")
    magnitude = np.vstack(
        [np.zeros((1, FRAMES)), 10 ** ((np.clip(value, -2, 2) * 25 - 50) / 20)]
    )
    phase = np.exp(2j * np.pi * np.random.default_rng(seed).random(magnitude.shape))
    for _ in range(iterations):
        wave = istft(magnitude * phase, fs=RATE, nperseg=FFT, noverlap=FFT - HOP)[1]
        updated = transform(wave)
        phase = updated / np.maximum(abs(updated), 1e-12)
    return istft(magnitude * phase, fs=RATE, nperseg=FFT, noverlap=FFT - HOP)[1].astype(
        np.float32
    )


class Block(nn.Module):
    def __init__(self, width):
        super().__init__()
        self.norm1 = nn.GroupNorm(4, width)
        self.norm2 = nn.GroupNorm(4, width)
        self.conv1 = nn.Conv2d(width, width, 3, padding=1)
        self.conv2 = nn.Conv2d(width, width, 3, padding=1)
        self.affine = nn.Linear(128, width * 2)

    def forward(self, x, context):
        scale, shift = self.affine(context)[:, :, None, None].chunk(2, 1)
        h = self.conv1(F.silu(self.norm1(x)))
        h = self.norm2(h) * (1 + scale) + shift
        return x + self.conv2(F.silu(h))


class PourFlow(nn.Module):
    def __init__(self):
        super().__init__()
        self.context = nn.Sequential(nn.Linear(20, 128), nn.SiLU(), nn.Linear(128, 128))
        self.input = nn.Conv2d(3, 16, 3, padding=1)
        self.blocks = nn.ModuleList([Block(x) for x in (16, 32, 64, 32, 16)])
        self.down1 = nn.Conv2d(16, 32, 4, stride=2, padding=1)
        self.down2 = nn.Conv2d(32, 64, 4, stride=2, padding=1)
        self.up1 = nn.Conv2d(64, 32, 3, padding=1)
        self.up2 = nn.Conv2d(32, 16, 3, padding=1)
        self.output = nn.Conv2d(16, 1, 3, padding=1)

    def forward(self, x, t, controls):
        angle = t[:, None] * torch.tensor([1, 2, 4, 8], device=x.device) * (2 * np.pi)
        context = self.context(
            torch.cat([controls, t[:, None], angle.sin(), angle.cos()], 1)
        )
        axis = torch.linspace(-1, 1, 256, device=x.device)
        frequency = axis[None, None, :, None].expand(len(x), 1, 256, 256)
        position = axis[None, None, None, :].expand(len(x), 1, 256, 256)
        a = self.blocks[0](self.input(torch.cat([x, frequency, position], 1)), context)
        b = self.blocks[1](self.down1(a), context)
        c = self.blocks[2](self.down2(b), context)
        d = self.blocks[3](self.up1(F.interpolate(c, scale_factor=2)) + b, context)
        e = self.blocks[4](self.up2(F.interpolate(d, scale_factor=2)) + a, context)
        return self.output(F.silu(e))


def load_source(source):
    manifest = json.loads((source / "source.json").read_text())
    if manifest["status"] != "complete" or manifest["revision"] != REVISION:
        raise ValueError("wrong or incomplete source")
    records = {row["path"]: row for row in manifest["files"]}
    for name, row in records.items():
        path = source / name
        if (
            path.stat().st_size != row["bytes"]
            or hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]
        ):
            raise ValueError("source integrity mismatch")
    frame = eligible(pd.read_csv(source / "splits/train.csv"))
    rows = []
    for row in frame.to_dict("records"):
        wave, rate = sf.read(source / f"audios/{row['item_id']}.wav", dtype="float32")
        duration = row["end_time"] - row["start_time"]
        if (
            wave.ndim != 1
            or rate != 48000
            or not np.isfinite(wave).all()
            or abs(len(wave) / rate - duration) > 0.05
        ):
            raise ValueError("unexpected source waveform/timestamp alignment")
        wave = resample_poly(wave, 1, 3)
        row["duration"] = duration
        row["dimensions"] = parse_measurements(row["measurements"])
        row["wave"] = wave
        row["spectrogram"] = encode(wave)
        rows.append(row)
    return rows, manifest


def crop(row, start):
    spec = row["spectrogram"][:, start : start + FRAMES]
    if spec.shape[1] < FRAMES:
        spec = np.pad(spec, ((0, 0), (0, FRAMES - spec.shape[1])), constant_values=-2)
    controls = condition(
        row["dimensions"],
        row["material"],
        row["shape"],
        row["duration"],
        start * HOP / RATE / row["duration"],
    )
    return spec, controls


@torch.inference_mode()
def sample(model, controls, seed=314, steps=64):
    device = next(model.parameters()).device
    rng = torch.Generator(device=device).manual_seed(seed)
    x = torch.randn((1, 1, 256, 256), generator=rng, device=device)
    c = torch.from_numpy(controls[None]).to(device)
    for step in range(steps):
        t = torch.full((1,), step / steps, device=device)
        x += model(x, t, c) / steps
    result = x[0, 0].cpu().numpy()
    if not np.isfinite(result).all():
        raise ValueError("nonfinite neural sample")
    return np.clip(result, -2, 2)


def metrics(wave, reference):
    def envelopes(w):
        n = len(w) // 160 * 160
        v = np.sqrt(np.mean(w[:n].reshape(-1, 160) ** 2, axis=1))
        return float(v.std() / max(v.mean(), 1e-12))

    a, b = encode(wave), encode(reference)

    # Time-averaged power and coarse temporal bands; no phase-aligned waveform MSE.
    def profile(v):
        power = 10 ** ((v * 25 - 50) / 10)
        return 10 * np.log10(power.mean(1) + 1e-12)

    pa, pb = profile(a), profile(b)
    return {
        "spectrum_rmse_db": float(np.sqrt(np.mean((pa - pb) ** 2))),
        "spectrum_shape_rmse_db": float(
            np.sqrt(np.mean(((pa - pa.mean()) - (pb - pb.mean())) ** 2))
        ),
        "level_error_db": float(
            20
            * np.log10(
                max(float(np.sqrt(np.mean(wave**2))), 1e-12)
                / max(float(np.sqrt(np.mean(reference**2))), 1e-12)
            )
        ),
        "rms_cv": envelopes(wave),
        "reference_rms_cv": envelopes(reference),
        "rms": float(np.sqrt(np.mean(wave**2))),
    }


def power_envelope_loss(estimate, target):
    """Source-trained power/variation constraint, not a perceptual validator.

    Offsets cancel in spectrum differences and envelope CV. Relative log power
    stays bounded; all losses are per example for diffusion-time weighting.
    """

    def summary(value):
        log_power = value.clamp(-2, 2) * (2.5 * np.log(10))
        spectrum = (torch.logsumexp(log_power, dim=-1) - np.log(value.shape[-1])) / (
            2.5 * np.log(10)
        )
        envelope = torch.exp(log_power).mean(dim=-2).sqrt()
        cv = envelope.std(dim=-1, unbiased=False) / envelope.mean(dim=-1).clamp_min(
            1e-6
        )
        return spectrum, cv

    a, acv = summary(estimate)
    b, bcv = summary(target)
    return (a - b).square().mean(dim=(1, 2)) + (acv - bcv).square().mean(dim=1)


def patch_start(rng, frames, sampling, ticket):
    if sampling not in ("uniform", "onset-balanced") or frames < 1:
        raise ValueError("invalid patch sampling")
    # Consume the same random draw in both modes so recording selections stay
    # matched. The condition retains the actual start; no time warp or relabel.
    start = int(rng.integers(max(1, frames - FRAMES + 1)))
    return 0 if sampling == "onset-balanced" and ticket % 2 == 0 else start


def fit(source, output, objective="velocity", sampling="uniform"):
    if objective not in ("velocity", "power-envelope"):
        raise ValueError("unknown training objective")
    torch.set_num_threads(4)
    torch.manual_seed(53)
    rows, provenance = load_source(source)
    training = [row for row in rows if row["role"] == "train"]
    development = [row for row in rows if row["role"] == "unseen_container"]
    if (
        not training
        or not development
        or {r["container_id"] for r in training} & set(HELDOUT)
    ):
        raise ValueError("invalid object separation")
    output.mkdir(parents=True, exist_ok=False)
    model = PourFlow().to("cuda")
    optimizer = torch.optim.AdamW(model.parameters(), lr=3e-4, weight_decay=0.01)
    rng = np.random.default_rng(53)
    losses = []
    for step in range(1500):
        batch = []
        for slot in range(6):
            row = training[rng.integers(len(training))]
            start = patch_start(
                rng, row["spectrogram"].shape[1], sampling, step * 6 + slot
            )
            batch.append(crop(row, start))
        y = torch.from_numpy(np.stack([v[0] for v in batch])[:, None]).to("cuda")
        controls = torch.from_numpy(np.stack([v[1] for v in batch])).to("cuda")
        noise = torch.randn_like(y)
        time = torch.rand(len(y), device="cuda")
        xt = (1 - time[:, None, None, None]) * noise + time[:, None, None, None] * y
        prediction = model(xt, time, controls)
        loss = F.mse_loss(prediction, y - noise)
        if objective == "power-envelope":
            endpoint = xt + (1 - time[:, None, None, None]) * prediction
            # Low-noise examples carry more weight: the target at pure noise
            # is not identifiable, so do not force a phase-aligned endpoint.
            loss = (
                loss + 0.25 * (time.square() * power_envelope_loss(endpoint, y)).mean()
            )
        optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
        losses.append(float(loss.detach()))
        if (step + 1) % 100 == 0:
            print(
                json.dumps(
                    {"step": step + 1, "loss_mean100": float(np.mean(losses[-100:]))}
                ),
                flush=True,
            )
    model.eval()
    save_file(
        {k: v.detach().cpu().contiguous() for k, v in model.state_dict().items()},
        output / "model.safetensors",
    )
    meta = {
        "format": "pour-flow-v1",
        "source_revision": REVISION,
        "source_sha256": hashlib.sha256(
            (source / "source.json").read_bytes()
        ).hexdigest(),
        "checkpoint_sha256": hashlib.sha256(
            (output / "model.safetensors").read_bytes()
        ).hexdigest(),
        "seed": 53,
        "steps": 1500,
        "objective": objective,
        "patch_sampling": sampling,
        "power_envelope_weight": 0.25 if objective == "power-envelope" else 0,
        "parameters": sum(p.numel() for p in model.parameters()),
        "train_ids": [r["item_id"] for r in training],
        "heldout_containers": HELDOUT,
        "terms": provenance["terms"],
        "input_scope": "dimensions in cm, material, shape, pouring duration and elapsed fraction; NOT measured flow rate or exact liquid level",
    }
    save_json(output / "model.json", meta)
    report = {
        "status": "complete",
        "model": meta,
        "loss_mean_first100": float(np.mean(losses[:100])),
        "loss_mean_last100": float(np.mean(losses[-100:])),
        "rows": [],
    }
    pending = []
    # All withheld recordings, a single fixed crop and seed; no favourable selection.
    for index, row in enumerate(development):
        start = 0
        oracle, controls = crop(row, start)
        pred = sample(model, controls)
        # Nearest training object/timing, not a neural gain by construction.
        baseline_row = min(
            training, key=lambda r: float(np.sum((crop(r, 0)[1] - controls) ** 2))
        )
        baseline = crop(baseline_row, 0)[0]
        real = np.pad(row["wave"][:SAMPLES], (0, max(0, SAMPLES - len(row["wave"]))))
        for kind, wave in [
            ("real", real),
            ("neural", decode(pred)),
            ("nearest", decode(baseline)),
            ("oracle", decode(oracle)),
        ]:
            record = {
                "item_id": row["item_id"],
                "container_id": row["container_id"],
                "kind": kind,
                "controls": controls.tolist(),
                "nearest_train_id": baseline_row["item_id"],
                **metrics(wave, real),
            }
            pending.append((f"{index:02d}-{kind}", wave, record))
        print(
            json.dumps({"evaluated": index + 1, "total": len(development)}), flush=True
        )
    gain = min(1, 0.98 / max(float(abs(w).max()) for _, w, _ in pending))
    preview = []
    shown = set()
    for name, wave, record in pending:
        path = output / (name + ".wav")
        sf.write(path, wave * gain, RATE, subtype="PCM_16")
        record.update(
            wav=str(path),
            sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
            gain=gain,
        )
        report["rows"].append(record)
        container = record["container_id"]
        if container not in shown and record["kind"] in ("real", "neural", "oracle"):
            preview.extend([wave, np.zeros(RATE // 2)])
            if record["kind"] == "oracle":
                shown.add(container)
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview) * gain, RATE, subtype="PCM_16")
    report["comparison"] = {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "gain": gain,
        "order": "first source-order heldout glass and plastic recordings; real/neural/oracle, same gain",
    }
    save_json(output / "result.json", report)


def render(
    directory,
    output,
    *,
    height=10,
    diameter_top=7,
    diameter_bottom=7,
    material="glass",
    shape="cylindrical",
    duration=15,
    progress=0.2,
    seed=2718,
    playback_gain=1,
    device="cuda",
):
    if not np.isfinite(playback_gain) or not 0 < playback_gain <= 100:
        raise ValueError("invalid audition gain")
    dimensions = {
        "net_height": height,
        "diameter_top": diameter_top,
        "diameter_bottom": diameter_bottom,
    }
    c = condition(dimensions, material, shape, duration, progress)
    if not 0 <= seed < 2**32:
        raise ValueError("invalid noise seed")
    metadata = directory / "model.json"
    checkpoint = directory / "model.safetensors"
    if metadata.stat().st_size > 100000 or checkpoint.stat().st_size > 10_000_000:
        raise ValueError("oversized model")
    meta = json.loads(metadata.read_text())
    if (
        meta["format"] != "pour-flow-v1"
        or hashlib.sha256(checkpoint.read_bytes()).hexdigest()
        != meta["checkpoint_sha256"]
    ):
        raise ValueError("wrong model identity")
    model = PourFlow()
    state = load_file(checkpoint)
    if not all(torch.isfinite(v).all() for v in state.values()):
        raise ValueError("invalid weights")
    model.load_state_dict(state, strict=True)
    model = model.to(device).eval()
    wave = decode(sample(model, c, seed), seed)
    output.mkdir(parents=True, exist_ok=False)
    gain = min(playback_gain, 0.98 / max(float(abs(wave).max()), 1e-12))
    path = output / "generated.wav"
    sf.write(path, wave * gain, RATE, subtype="PCM_16")
    save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "model": meta,
            "controls": c.tolist(),
            "request": {
                "dimensions_cm": dimensions,
                "material": material,
                "shape": shape,
                "pour_duration_seconds": duration,
                "elapsed_fraction": progress,
            },
            "seed": seed,
            "requested_audition_gain": playback_gain,
            "raw_rms": float(np.sqrt(np.mean(wave**2))),
            "wav": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "gain": gain,
        },
    )
    print(json.dumps({"wav": str(path)}), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--acquire", action="store_true")
    mode.add_argument("--source", type=Path)
    mode.add_argument("--render-model", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--objective", choices=("velocity", "power-envelope"), default="velocity"
    )
    parser.add_argument(
        "--patch-sampling", choices=("uniform", "onset-balanced"), default="uniform"
    )
    parser.add_argument("--height", type=float, default=10)
    parser.add_argument("--diameter-top", type=float, default=7)
    parser.add_argument("--diameter-bottom", type=float, default=7)
    parser.add_argument("--material", choices=MATERIALS, default="glass")
    parser.add_argument("--shape", choices=SHAPES, default="cylindrical")
    parser.add_argument("--duration", type=float, default=15)
    parser.add_argument("--progress", type=float, default=0.2)
    parser.add_argument("--seed", type=int, default=2718)
    parser.add_argument("--playback-gain", type=float, default=1)
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cuda")
    args = parser.parse_args()
    output = args.output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("keep source/model/media outside Git")
    if args.acquire:
        acquire(output)
    elif args.source:
        fit(args.source.resolve(), output, args.objective, args.patch_sampling)
    else:
        render(
            args.render_model.resolve(),
            output,
            height=args.height,
            diameter_top=args.diameter_top,
            diameter_bottom=args.diameter_bottom,
            material=args.material,
            shape=args.shape,
            duration=args.duration,
            progress=args.progress,
            seed=args.seed,
            playback_gain=args.playback_gain,
            device=args.device,
        )
