"""Physical conditions -> latent sequence -> frozen Oobleck friction sound.

Report-only Cluster Haptic Texture experiment. Fixed rubber probe/three known
surfaces; no arbitrary geometry, material-pair or perceptual-quality admission.
Powered by Stability AI; TangoFlux / Hung et al.; source Eguchi et al., CC BY4.
"""

from __future__ import annotations

import argparse
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_texture_codec as codec
import physical_sound_texture_fit as spectrum
import soundfile as sf
import torch
from safetensors.torch import load_file, save_file
from scipy.signal import resample_poly
from torch import nn
from torch.nn import functional as F

FRAMES, HOP, CONTEXT = 32, 2048, 8
CORE = (FRAMES - 2 * CONTEXT) * HOP
GAIN = 17.374337221633088
PLAYBACK = 100 / GAIN
CODEC_SHA = "d73619a1d1e1dc48e606632931ffce440b4959ce2a4ed5a3522c3bb573b103be"
STEPS = 2000
FORMAT = "texture-latent-flow-v1"


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def fresh(output):
    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("new external output directory required")
    output.mkdir(parents=True)
    return output


def load_codec(device):
    from diffusers import AutoencoderOobleck
    from huggingface_hub import snapshot_download

    root = Path(
        snapshot_download(
            codec.tango.MODEL, revision=codec.tango.REVISION, local_files_only=True
        )
    )
    if codec.sha(root / "vae.safetensors") != CODEC_SHA:
        raise ValueError("codec identity changed")
    vae = AutoencoderOobleck().eval().requires_grad_(False)
    vae.load_state_dict(load_file(root / "vae.safetensors"), strict=True)
    if vae.hop_length != HOP or vae.sampling_rate != codec.RATE:
        raise ValueError("codec rate/stride changed")
    return vae.to(device), root


class Block(nn.Module):
    def __init__(self):
        super().__init__()
        self.norm = nn.GroupNorm(8, 64)
        self.affine = nn.Linear(64, 128)
        self.first = nn.Conv1d(64, 64, 3, padding=1)
        self.second = nn.Conv1d(64, 64, 3, padding=1)

    def forward(self, x, context):
        scale, shift = self.affine(context)[:, :, None].chunk(2, 1)
        value = F.silu(self.norm(x) * (1 + scale) + shift)
        return x + 0.25 * self.second(F.silu(self.first(value)))


class TextureFlow(nn.Module):
    def __init__(self):
        super().__init__()
        self.context = nn.Sequential(nn.Linear(14, 64), nn.SiLU(), nn.Linear(64, 64))
        self.input = nn.Conv1d(64, 64, 3, padding=1)
        self.blocks = nn.ModuleList([Block() for _ in range(4)])
        self.output = nn.Conv1d(64, 64, 3, padding=1)
        nn.init.zeros_(self.output.weight)
        nn.init.zeros_(self.output.bias)
        self.register_buffer("center", torch.zeros(1, 64, 1))
        self.register_buffer("scale", torch.ones(1, 64, 1))

    def forward(self, x, t, physical):
        angle = t[:, None] * torch.tensor([1, 2, 4, 8], device=x.device) * (2 * np.pi)
        context = self.context(
            torch.cat((physical, t[:, None], angle.sin(), angle.cos()), dim=1)
        )
        value = self.input(x)
        for block in self.blocks:
            value = block(value, context)
        return self.output(value)


def normalization(mean, std):
    if mean.shape != std.shape or mean.ndim != 3 or mean.shape[1:] != (64, FRAMES):
        raise ValueError("unexpected TRAIN posterior shape")
    if (
        not torch.isfinite(mean).all()
        or not torch.isfinite(std).all()
        or (std < 0).any()
    ):
        raise ValueError("invalid posterior")
    center = mean.mean((0, 2), keepdim=True)
    scale = (
        ((mean - center).square() + std.square())
        .mean((0, 2), keepdim=True)
        .sqrt()
        .clamp_min(0.1)
    )
    return center, scale


def validate_controls(controls):
    if controls.ndim != 2 or controls.shape[1] != 5 or not np.isfinite(controls).all():
        raise ValueError("invalid controls")
    if (
        not 1 <= len(controls) <= 128
        or not np.isin(controls[:, :3], [0, 1]).all()
        or not np.all(controls[:, :3].sum(1) == 1)
        or (np.abs(controls[:, 3:]) > 1).any()
    ):
        raise ValueError("controls outside three-surface training domain")


@torch.inference_mode()
def sample(model, controls, seed):
    """No recording, source path, posterior or retrieval input is accepted."""
    controls = np.asarray(controls, np.float32)
    validate_controls(controls)
    if not isinstance(seed, int) or not 0 <= seed < 2**32:
        raise ValueError("invalid seed")
    device = next(model.parameters()).device
    physical = torch.tensor(controls, device=device)
    # Common random numbers isolate condition changes; batches are not diverse
    # seeds. Different seed314/2718 runs provide the disclosed stochastic control.
    noise = torch.randn(
        (1, 64, FRAMES),
        generator=torch.Generator(device=device).manual_seed(seed),
        device=device,
    )
    x = noise.repeat(len(controls), 1, 1)
    for step in range(64):
        t = torch.full((len(x),), step / 64, device=device)
        first = model(x, t, physical)
        second = model(x + first / 128, t + 1 / 128, physical)
        x = x + second / 64
    latent = x * model.scale + model.center
    if not torch.isfinite(latent).all():
        raise ValueError("nonfinite generated latent")
    return latent


def load_model(directory, device="cuda"):
    mp, wp = directory / "model.json", directory / "model.safetensors"
    if mp.stat().st_size > 100000 or wp.stat().st_size > 4_000_000:
        raise ValueError("oversized model")
    meta = json.loads(mp.read_text())
    if (
        meta["format"] != FORMAT
        or meta["codec_sha256"] != CODEC_SHA
        or meta["input_gain"] != GAIN
        or meta["checkpoint_sha256"] != codec.sha(wp)
    ):
        raise ValueError("model identity mismatch")
    state = load_file(wp)
    if (
        not all(torch.isfinite(v).all() for v in state.values())
        or (state["scale"] <= 0).any()
    ):
        raise ValueError("invalid weights")
    model = TextureFlow()
    model.load_state_dict(state, strict=True)
    return model.to(device).eval(), meta


def load_baseline(directory, source_sha):
    model, meta = spectrum.load_model(directory)
    path = directory / "result.json"
    if path.stat().st_size > 1_000_000:
        raise ValueError("oversized baseline result")
    report = json.loads(path.read_text())
    expected = {r["id"]: r for r in spectrum.source.conditions(True)}
    rows = report["rows"]
    if (
        meta.get("rank") != 4
        or meta.get("corpus_sha256") != source_sha
        or report["status"] != "complete"
        or report["model"] != meta
        or len(rows) != 60
        or {r["id"] for r in rows} != set(expected)
    ):
        raise ValueError("original40mm/s rank4 baseline required")
    for row in rows:
        if row["role"] != spectrum.role(expected[row["id"]]) or any(
            row[k] != v for k, v in expected[row["id"]].items()
        ):
            raise ValueError("baseline TRAIN/development roles changed")
    return model, {**meta, "verified_role_report_sha256": codec.sha(path)}


def window_start(row, frames):
    center = (row["crop_start_seconds"] + 0.375) * codec.RATE
    first = round(center / HOP) - FRAMES // 2
    if first < 0 or first + FRAMES > frames:
        raise ValueError("insufficient real context around sliding interval")
    return first


@torch.inference_mode()
def prepare(manifest_path, vae, device):
    rows, _, _, controls, manifest = spectrum.load_data(manifest_path)
    originals = {r["id"]: r for r in manifest["rows"]}
    means, stds, references = [], [], []
    for row in rows:
        original = originals[row["id"]]
        path = spectrum.checked(original["audio"], manifest, manifest_path.parent)
        audio, _ = sf.read(path, always_2d=True, dtype="float32")
        native = np.repeat(audio * GAIN, 2, axis=1)
        padded = np.pad(native, ((0, (-len(native)) % HOP), (0, 0)))
        posterior = vae.encode(torch.tensor(padded.T[None], device=device)).latent_dist
        # Use only real context, not the right-padding region.
        first = window_start(row, len(native) // HOP)
        means.append(posterior.mean[0, :, first : first + FRAMES].cpu())
        stds.append(posterior.std[0, :, first : first + FRAMES].cpu())
        start = (first + CONTEXT) * HOP
        reference = native[start : start + CORE]
        if len(reference) != CORE:
            raise ValueError("incomplete real core")
        references.append(reference)
        row.update(latent_start=first, core_start_sample=start, core_frames=CORE)
    return rows, torch.stack(means), torch.stack(stds), controls, references, manifest


def publish_decode(output, name, native):
    if native.shape != (FRAMES * HOP, 2):
        raise ValueError("decoded sequence shape changed")
    full, _ = codec.publish(output / f"{name}-full.wav", native, "FLOAT")
    core = native[CONTEXT * HOP : CONTEXT * HOP + CORE]
    record, pcm = codec.publish(output / f"{name}.wav", core * PLAYBACK)
    return {**record, "full": full, "playback_gain": PLAYBACK}, pcm


def waveform_metrics(candidate, reference):
    metrics = codec.metrics(candidate, reference)
    metrics["envelope_cv_absolute_error"] = abs(
        metrics["envelope_cv"] - metrics["real_envelope_cv"]
    )
    metrics["absolute_level_error_db"] = abs(metrics["stereo_level_error_db"])

    def acf(wave):
        mono = wave.mean(1)
        hop = round(0.025 * codec.RATE)
        blocks = mono[: len(mono) // hop * hop].reshape(-1, hop)
        env = np.sqrt(np.mean(blocks**2, axis=1))
        env = env - env.mean()
        denominator = max(float(np.mean(env**2)), 1e-18)
        return np.array(
            [np.mean(env[:-lag] * env[lag:]) / denominator for lag in (1, 2, 3, 4)]
        )

    actual, expected = acf(candidate), acf(reference)
    metrics.update(
        envelope_acf=actual.tolist(),
        real_envelope_acf=expected.tolist(),
        envelope_acf_mae=float(np.mean(np.abs(actual - expected))),
    )
    return metrics


def envelope_acf_mono(wave, rate=22050):
    hop = round(0.025 * rate)
    if wave.ndim != 1 or len(wave) < 8 * hop or not np.isfinite(wave).all():
        raise ValueError("finite mono core required")
    blocks = wave[: len(wave) // hop * hop].reshape(-1, hop)
    env = np.sqrt(np.mean(blocks**2, axis=1))
    env -= env.mean()
    denominator = max(float(np.mean(env**2)), 1e-18)
    return np.array(
        [np.mean(env[:-lag] * env[lag:]) / denominator for lag in (1, 2, 3, 4)]
    )


def bandmatched_audit(result_path):
    """Posthoc fairness check on fixed PCM; never reruns or selects a model."""
    report = json.loads(result_path.read_text())
    if report["status"] != "complete" or report["model"]["format"] != FORMAT:
        raise ValueError("completed friction flow result required")
    cache = {}

    def read(entry):
        key = entry["wav"], entry["sha256"]
        if key not in cache:
            path = Path(entry["wav"]).resolve()
            if (
                not path.is_relative_to(result_path.parent.resolve())
                or path.stat().st_size > 1_000_000
                or codec.sha(path) != entry["sha256"]
            ):
                raise ValueError("invalid core PCM identity")
            wave, rate = sf.read(path, always_2d=True)
            if (
                rate != codec.RATE
                or wave.shape != (CORE, 2)
                or not np.isfinite(wave).all()
            ):
                raise ValueError("unexpected published core")
            cache[key] = resample_poly(wave.mean(1), 1, 2)
        return cache[key]

    references = {r["id"]: read(r["real"]) for r in report["references"]}
    rows = []
    for row in report["rows"]:
        if row["variant"] not in ("flow", "previous_spectrum", "interpolation"):
            continue
        real, wave = references[row["id"]], read(row)
        rows.append(
            {
                "id": row["id"],
                "role": row["role"],
                "variant": row["variant"],
                "seed": row["seed"],
                "level_dbfs": codec.level(wave),
                "real_level_dbfs": codec.level(real),
                "absolute_level_error_db": abs(codec.level(wave) - codec.level(real)),
                "envelope_acf_mae": float(
                    np.mean(np.abs(envelope_acf_mono(wave) - envelope_acf_mono(real)))
                ),
            }
        )
    summary = []
    for role in ("train", "unseen_speed", "repeat_development"):
        for variant in ("flow", "previous_spectrum", "interpolation"):
            selected = [
                r for r in rows if r["role"] == role and r["variant"] == variant
            ]
            summary.append(
                {
                    "role": role,
                    "variant": variant,
                    "count": len(selected),
                    **{
                        key: float(np.mean([r[key] for r in selected]))
                        for key in ("absolute_level_error_db", "envelope_acf_mae")
                    },
                }
            )
    levels = {r["id"]: r["real_level_dbfs"] for r in rows}
    pairs = paired_responses(
        {
            "conditions": report["conditions"],
            "references": [
                {"id": ident, "real": {"level_dbfs": value}}
                for ident, value in levels.items()
            ],
            "rows": rows,
        }
    )
    return {
        "scope": "posthoc22.05kHz-mono bandwidth-matched audit of unchanged published PCM, not new independent evidence",
        "source_report_sha256": codec.sha(result_path),
        "resampler": "scipy.signal.resample_poly mono,1/2,default filter",
        "rows": rows,
        "summary": summary,
        "paired_responses": pairs,
    }


@torch.inference_mode()
def evaluate(
    model,
    vae,
    rows,
    means,
    stds,
    controls,
    references,
    baseline,
    output,
    report,
    device,
):
    previous, previous_meta = load_baseline(baseline, report["source_sha256"])
    report["baseline"] = previous_meta
    train = np.array([r["role"] == "train" for r in rows])
    train_rows = [r for r in rows if r["role"] == "train"]
    psds = np.stack(
        [spectrum.spectrum(resample_poly(w.mean(1) / GAIN, 1, 2)) for w in references]
    )
    generated, comparisons = {}, []
    # Generate all distinct conditions before scoring any target. No waveform
    # or posterior is passed to sample; only the five physical features.
    unique = np.unique(controls, axis=0)
    for seed in (314, 2718):
        latent = sample(model, unique, seed)
        for idx, physical in enumerate(unique):
            texture = spectrum.TEXTURES[int(np.argmax(physical[:3]))]
            speed, force = (
                float(physical[3] * 20 + 40),
                float(physical[4] * 0.25 + 0.75),
            )
            name = f"generated-{texture}-{speed}-{force}-seed{seed}"
            native = vae.decode(latent[idx : idx + 1]).sample[0].T.cpu().numpy()
            generated[tuple(physical), seed] = publish_decode(output, name, native)
    report["generated_conditions"] = [
        {"features": list(map(float, key[0])), "seed": key[1], **value[0]}
        for key, value in generated.items()
    ]
    save(output / "result.json", report)
    for i, row in enumerate(rows):
        real_entry, real = codec.publish(
            output / f"{row['id']}-real.wav", references[i] * PLAYBACK
        )
        posterior = means[i : i + 1] + stds[i : i + 1] * torch.randn(
            (1, 64, FRAMES), generator=torch.Generator().manual_seed(314)
        )
        native = vae.decode(posterior.to(device)).sample[0].T.cpu().numpy()
        oracle_entry, oracle = publish_decode(output, f"{row['id']}-codec314", native)
        report["references"].append(
            {
                "id": row["id"],
                "real": real_entry,
                "codec": oracle_entry,
                "codec_metrics": waveform_metrics(oracle, real),
            }
        )
        interpolation = spectrum.interpolate(row, train_rows, psds[train])
        old_psd = previous(torch.from_numpy(controls[i])).numpy()
        for seed in (314, 2718):
            physical = controls[i]
            wrong_material = physical.copy()
            wrong_material[:3] = np.roll(wrong_material[:3], 1)
            wrong_speed = physical.copy()
            wrong_speed[3] = 1 if physical[3] <= 0 else -1
            wrong_load = physical.copy()
            wrong_load[4] *= -1
            variants = {
                name: generated[tuple(feature), seed]
                for name, feature in (
                    ("flow", physical),
                    ("wrong_material", wrong_material),
                    ("wrong_speed", wrong_speed),
                    ("wrong_load", wrong_load),
                )
            }
            for name, db in (
                ("previous_spectrum", old_psd),
                ("interpolation", interpolation),
            ):
                # Same new core interval for all comparisons. The existing rank4
                # model was fitted on.75s; its weights are not changed here.
                wave = spectrum.synthesize(db, CORE / codec.RATE, seed)
                native = resample_poly(wave, 2, 1)
                entry, pcm = codec.publish(
                    output / f"{row['id']}-{name}-{seed}.wav",
                    np.repeat((native * 100)[:, None], 2, axis=1),
                )
                variants[name] = (entry, pcm)
            for name, (entry, pcm) in variants.items():
                report["rows"].append(
                    {
                        "id": row["id"],
                        "role": row["role"],
                        "seed": seed,
                        "variant": name,
                        **entry,
                        **waveform_metrics(pcm, real),
                    }
                )
            if row["commanded_speed_mm_s"] == 40 and row["repeat"] == 0 and seed == 314:
                for pcm in (
                    real,
                    variants["flow"][1],
                    variants["previous_spectrum"][1],
                    variants["interpolation"][1],
                    oracle,
                ):
                    comparisons.extend((pcm, np.zeros((round(0.25 * codec.RATE), 2))))
        print(
            json.dumps({"evaluated": row["id"], "records": len(report["rows"])}),
            flush=True,
        )
        save(output / "result.json", report)
    report["comparison"], _ = codec.publish(
        output / "comparison.wav", np.concatenate(comparisons)
    )
    report["comparison_order"] = (
        "wood/steel/glass,.5/1N,40mm/s,repeat0,seed314: real/flow/old-rank4/interpolation/reference-aided-codec"
    )
    report["metrics_summary"] = summarize(report["rows"])
    report["paired_responses"] = paired_responses(report)


def summarize(rows):
    result = []
    for role in ("train", "unseen_speed", "repeat_development"):
        for variant in (
            "flow",
            "wrong_material",
            "wrong_speed",
            "wrong_load",
            "previous_spectrum",
            "interpolation",
        ):
            selected = [
                r for r in rows if r["role"] == role and r["variant"] == variant
            ]
            result.append(
                {
                    "role": role,
                    "variant": variant,
                    "count": len(selected),
                    **{
                        key: float(np.mean([r[key] for r in selected]))
                        for key in (
                            "waveform_spectrum_rmse_db",
                            "shape_rmse_db",
                            "absolute_level_error_db",
                            "envelope_cv_absolute_error",
                            "envelope_acf_mae",
                        )
                    },
                }
            )
    return result


def paired_responses(report):
    conditions = report["conditions"]
    by_key = {
        (
            r["texture_id"],
            r["commanded_speed_mm_s"],
            r["commanded_normal_force_N"],
            r["repeat"],
        ): r["id"]
        for r in conditions
    }
    expected = spectrum.source.conditions(True)
    if len(conditions) != 60 or {r["id"] for r in conditions} != {
        r["id"] for r in expected
    }:
        raise ValueError("complete disclosed60-condition response grid required")
    refs = {r["id"]: r["real"]["level_dbfs"] for r in report["references"]}
    index = {
        (r["id"], r["variant"], r["seed"]): r["level_dbfs"] for r in report["rows"]
    }
    output = []
    for repeat in (0, 1):
        for axis in ("speed", "load"):
            pairs = []
            for texture in spectrum.TEXTURES:
                if axis == "speed":
                    for force in (0.5, 1):
                        for low, high in ((20, 30), (30, 40), (40, 50), (50, 60)):
                            pairs.append(
                                (
                                    by_key[texture, low, force, repeat],
                                    by_key[texture, high, force, repeat],
                                )
                            )
                else:
                    for speed in (20, 30, 40, 50, 60):
                        pairs.append(
                            (
                                by_key[texture, speed, 0.5, repeat],
                                by_key[texture, speed, 1.0, repeat],
                            )
                        )
            for variant in ("flow", "previous_spectrum", "interpolation"):
                for seed in (314, 2718):
                    values = []
                    for low, high in pairs:
                        real = refs[high] - refs[low]
                        generated = (
                            index[high, variant, seed] - index[low, variant, seed]
                        )
                        values.append(
                            {
                                "low": low,
                                "high": high,
                                "real_delta_db": real,
                                "generated_delta_db": generated,
                            }
                        )
                    output.append(
                        {
                            "repeat": repeat,
                            "axis": axis,
                            "variant": variant,
                            "seed": seed,
                            "count": len(values),
                            "same_direction": sum(
                                int(
                                    np.sign(v["real_delta_db"])
                                    == np.sign(v["generated_delta_db"])
                                )
                                for v in values
                            ),
                            "delta_mae_db": float(
                                np.mean(
                                    [
                                        abs(
                                            v["real_delta_db"] - v["generated_delta_db"]
                                        )
                                        for v in values
                                    ]
                                )
                            ),
                            "pairs": values,
                        }
                    )
    return output


def run(manifest_path, baseline, output, device="cuda"):
    torch.set_num_threads(4)
    torch.manual_seed(23)
    output = fresh(output)
    shutil.copyfile(__file__, output / "executed-script.py")
    started = time.monotonic()
    report = {
        "status": "running",
        "scope": "disclosed physical-friction conditional generation; no independent test or quality admission",
        "source": str(manifest_path),
        "source_sha256": codec.sha(manifest_path),
        "training_steps": STEPS,
        "seed": 23,
        "input_gain": GAIN,
        "playback_gain": PLAYBACK,
        "references": [],
        "rows": [],
    }
    save(output / "result.json", report)
    try:
        load_baseline(baseline, report["source_sha256"])
        vae, codec_root = load_codec(device)
        for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
            shutil.copyfile(codec_root / name, output / name)
        rows, means, stds, controls, references, manifest = prepare(
            manifest_path, vae, device
        )
        train = np.array([row["role"] == "train" for row in rows])
        if train.sum() != 24:
            raise ValueError("original24 TRAIN required")
        mean, std = means[train].to(device), stds[train].to(device)
        physical = torch.tensor(controls[train], device=device)
        torch.manual_seed(23)
        model = TextureFlow().to(device)
        center, scale = normalization(mean, std)
        model.center.copy_(center)
        model.scale.copy_(scale)
        optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
        losses = []
        for step in range(STEPS):
            target = (mean + std * torch.randn_like(std) - center) / scale
            noise, t = torch.randn_like(target), torch.rand(len(target), device=device)
            x = (1 - t[:, None, None]) * noise + t[:, None, None] * target
            loss = (model(x, t, physical) - (target - noise)).square().mean()
            if not torch.isfinite(loss):
                raise ValueError("nonfinite training loss")
            optimizer.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(
                model.parameters(), 1, error_if_nonfinite=True
            )
            optimizer.step()
            losses.append(float(loss.detach()))
            if (step + 1) % 200 == 0:
                print(
                    json.dumps(
                        {
                            "step": step + 1,
                            "mean_loss200": float(np.mean(losses[-200:])),
                        }
                    ),
                    flush=True,
                )
        model.eval()
        save_file(model.state_dict(), output / "model.safetensors")
        meta = {
            "format": FORMAT,
            "codec_sha256": CODEC_SHA,
            "checkpoint_sha256": codec.sha(output / "model.safetensors"),
            "input_gain": GAIN,
            "playback_gain": PLAYBACK,
            "source_sha256": report["source_sha256"],
            "source_license": manifest["license"],
            "source_authors": manifest["authors"],
            "source_article": manifest["article"],
            "frames": FRAMES,
            "hop": HOP,
            "context_frames": CONTEXT,
            "training_ids": [r["id"] for r in rows if r["role"] == "train"],
            "training_steps": STEPS,
            "seed": 23,
            "solver": "64 explicit-midpoint steps",
            "parameters": sum(p.numel() for p in model.parameters()),
            "loss_windows200": [
                float(np.mean(losses[i : i + 200])) for i in range(0, STEPS, 200)
            ],
            "scope": report["scope"],
        }
        save(output / "model.json", meta)
        report.update(
            model=meta, conditions=rows, training_seconds=time.monotonic() - started
        )
        save(output / "result.json", report)
        evaluate(
            model,
            vae,
            rows,
            means,
            stds,
            controls,
            references,
            baseline,
            output,
            report,
            device,
        )
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
        save(output / "result.json", report)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save(output / "result.json", report)
        raise
    print(
        json.dumps({"status": "complete", "comparison": report["comparison"]}),
        flush=True,
    )


@torch.inference_mode()
def render(directory, output, texture, speed, force, seed, device="cuda"):
    torch.set_num_threads(4)
    physical = spectrum.features(texture, speed, force)[None]
    model, meta = load_model(directory, device)
    vae, root = load_codec(device)
    native = vae.decode(sample(model, physical, seed)).sample[0].T.cpu().numpy()
    output = fresh(output)
    entry, _ = publish_decode(output, "generated", native)
    for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
        shutil.copyfile(root / name, output / name)
    report = {
        "reference_audio_input": False,
        "model_sha256": meta["checkpoint_sha256"],
        "codec_sha256": CODEC_SHA,
        "texture": texture,
        "speed_mm_s": speed,
        "normal_force_N": force,
        "seed": seed,
        **entry,
    }
    save(output / "result.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--corpus", type=Path)
    group.add_argument("--model", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--texture", type=int, default=74)
    parser.add_argument("--speed", type=float, default=40)
    parser.add_argument("--force", type=float, default=0.5)
    parser.add_argument("--seed", type=int, default=314)
    args = parser.parse_args()
    if args.corpus:
        if args.baseline is None:
            parser.error("--corpus requires --baseline")
        run(args.corpus.resolve(), args.baseline.resolve(), args.output)
    else:
        print(
            json.dumps(
                render(
                    args.model.resolve(),
                    args.output,
                    args.texture,
                    args.speed,
                    args.force,
                    args.seed,
                )
            )
        )
