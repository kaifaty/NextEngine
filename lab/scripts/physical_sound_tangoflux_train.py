"""Bounded text-only TangoFlux LoRA learnability experiment on disclosed glass.

Powered by Stability AI; TangoFlux / Hung et al., non-commercial research only.
Internet recordings are training targets, never inference inputs. Development
is recording-disjoint from this fit, not pristine or object-independent evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_audible_glass as glass
import physical_sound_audible_glass_cycle as cycle
import physical_sound_audible_glass_unseen as unseen
import physical_sound_tangoflux_pilot as tango
import physical_sound_text_pilot as pilot
import torch
from scipy.signal import resample_poly

PROMPT = "A knife hits a wine glass once, followed by a ringing glass decay."
SECONDS = 1.5
ACTIVE_FRAMES = math.ceil(SECONDS * tango.RATE / 2048)
CASES = (("glass-metal", PROMPT), *(pilot.CASES[i] for i in (1, 2, 6, 8)))


def loss_parts(prediction, target, active_frames=ACTIVE_FRAMES):
    if prediction.shape != target.shape or target.ndim != 3:
        raise ValueError("expected matching batch/time/channel tensors")
    if not 0 < active_frames < target.shape[1]:
        raise ValueError("active and padding regions must both be nonempty")
    error = (prediction.float() - target.float()).square()
    active = error[:, :active_frames].mean()
    tail = error[:, active_frames:].mean()
    return {
        "active": active,
        "tail": tail,
        "full": error.mean(),
        "balanced": (active + tail) / 2,
    }


def prepare_sources(source_root: Path):
    rows, waves = [], []
    for sound_id, _, expected in glass.SOURCES:
        path = source_root / f"{sound_id}_13431397-hq.mp3"
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"source hash mismatch: {sound_id}")
        decoded = glass.decode(path)
        for index, peak in enumerate(cycle.PEAKS[sound_id]):
            wave, _, metadata = unseen.crop(decoded, peak)
            # Retain existing disclosed crop identity; peak-normalize to 0.5.
            wave = resample_poly(wave.numpy() * (0.5 / 0.8), 441, 320).astype(
                np.float32
            )
            rows.append(
                {
                    "id": f"{sound_id}-{index}",
                    "sound_id": sound_id,
                    "source": str(path),
                    "sha256": expected,
                    **metadata,
                    "additional_gain": 0.5 / 0.8,
                }
            )
            waves.append(np.stack([wave, wave]))
    train, development = cycle.split_rows(rows)
    return rows, waves, train, development


def cache_latents(vae, waves, rows, output):
    means, stds, controls = [], [], []
    vae.to("cuda")
    with torch.no_grad():
        for row, wave in zip(rows, waves):
            padded = np.pad(wave, ((0, 0), (0, 30 * tango.RATE - wave.shape[1])))
            posterior = vae.encode(
                torch.from_numpy(padded)[None].to("cuda")
            ).latent_dist
            mean = posterior.mean.transpose(1, 2).cpu()
            std = posterior.std.transpose(1, 2).cpu()
            if (
                mean.shape != (1, 645, 64)
                or not torch.isfinite(mean).all()
                or not torch.isfinite(std).all()
            ):
                raise ValueError("invalid VAE posterior")
            means.append(mean)
            stds.append(std)
            if row["id"].endswith("-0"):
                reconstructed = (
                    vae.decode(posterior.mean)
                    .sample[0, :, : wave.shape[1]]
                    .cpu()
                    .numpy()
                )
                original, _ = tango.publish(output, row["id"] + "-original", wave)
                roundtrip, _ = tango.publish(output, row["id"] + "-vae", reconstructed)
                controls.append(
                    {"source_id": row["id"], "original": original, "vae": roundtrip}
                )
            print(json.dumps({"encoded": row["id"]}), flush=True)
    vae.to("cpu")
    torch.cuda.empty_cache()
    return torch.cat(means), torch.cat(stds), controls


def conditioning(model):
    with torch.no_grad():
        text, mask = model.encode_text([PROMPT])
        pooled = model.fc(torch.where(mask[..., None], text, float("nan")).nanmean(1))
        duration = model.encode_duration(torch.tensor([SECONDS], device="cuda"))
        hidden = torch.cat([text, duration], dim=1)
    return hidden, pooled


def velocity(model, noisy, sigma, condition):
    hidden, pooled = condition
    return model.transformer(
        hidden_states=noisy,
        encoder_hidden_states=hidden,
        pooled_projections=pooled,
        img_ids=torch.arange(noisy.shape[1], device=noisy.device)[None, :, None].repeat(
            1, 1, 3
        ),
        txt_ids=torch.zeros(1, hidden.shape[1], 3, device=noisy.device),
        guidance=None,
        timestep=sigma.reshape(1),
        return_dict=False,
    )[0]


def evaluate_flow(model, means, indices, condition):
    records = []
    model.transformer.eval()
    with torch.no_grad(), torch.autocast("cuda", dtype=torch.bfloat16):
        for index in indices:
            latent = means[index : index + 1].to("cuda")
            for value in (0.2, 0.5, 0.8):
                rng = torch.Generator(device="cuda").manual_seed(10000 + index)
                noise = torch.randn(latent.shape, generator=rng, device="cuda")
                sigma = torch.tensor(value, device="cuda")
                prediction = velocity(
                    model, (1 - sigma) * latent + sigma * noise, sigma, condition
                )
                parts = loss_parts(prediction, noise - latent)
                records.append(
                    {
                        "index": index,
                        "sigma": value,
                        **{k: float(v) for k, v in parts.items()},
                    }
                )
    return {
        "mean": {k: float(np.mean([r[k] for r in records])) for k in parts},
        "rows": records,
    }


def render(model, vae, output, step):
    output.mkdir()
    model.eval().to("cuda")
    report = {
        "status": "running",
        "model": tango.MODEL,
        "revision": tango.REVISION,
        "local_training_steps": step,
        "reference_audio_input": False,
        "seconds": 5.0,
        "steps": 50,
        "seeds": [42, 123],
        "precision": "float32",
        "cases": [{"id": k, "prompt": p} for k, p in CASES],
        "rows": [],
        "controls": [],
    }
    preview = []
    for seed in report["seeds"]:
        for index, (key, prompt) in enumerate((*CASES, ("empty-prompt", ""))):
            duration = SECONDS if index == 0 else 5.0
            torch.manual_seed(seed)
            with torch.no_grad():
                latent = model.inference_flow(
                    prompt,
                    duration=duration,
                    num_inference_steps=50,
                    guidance_scale=4.5,
                    disable_progress=True,
                )
                model.to("cpu")
                torch.cuda.empty_cache()
                vae.to("cuda")
                wave = (
                    vae.decode(latent.transpose(2, 1))
                    .sample[0, :, : int(duration * tango.RATE)]
                    .cpu()
                    .numpy()
                )
                vae.to("cpu")
                torch.cuda.empty_cache()
                model.to("cuda")
            record, mono = tango.publish(output, f"{key}-seed{seed}", wave)
            record.update({"seed": seed, "seconds": duration})
            if index < len(CASES):
                report["rows"].append({"case": index, **record})
                if seed == 42:
                    preview.extend((mono, np.zeros(pilot.RATE // 2, dtype=np.float32)))
                    pilot.write_audio(output / "preview.wav", np.concatenate(preview))
            else:
                report["controls"].append({"id": key, **record})
            pilot.save_report(output / "result.json", report)
            print(json.dumps({"step": step, "rendered": key, "seed": seed}), flush=True)
    report["status"] = "complete"
    pilot.save_report(output / "result.json", report)
    model.text_encoder.to("cpu")
    torch.cuda.empty_cache()
    return str(output / "result.json")


def run(source_root, output, steps=120, lr=1e-4, objective="balanced"):
    from diffusers.training_utils import compute_density_for_timestep_sampling
    from peft import LoraConfig
    from safetensors.torch import save_file

    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must be a new external directory")
    if not 1 <= steps <= 400 or not np.isfinite(lr) or not 0 < lr <= 1e-3:
        raise ValueError("bounded training steps and learning rate required")
    if objective not in ("balanced", "full"):
        raise ValueError("unknown training objective")
    output.mkdir(parents=True)
    shutil.copyfile(__file__, output / "executed-script.py")
    started = time.monotonic()
    torch.set_num_threads(4)
    torch.manual_seed(0)
    report = {
        "status": "running",
        "steps_requested": steps,
        "steps_completed": 0,
        "model": tango.MODEL,
        "revision": tango.REVISION,
        "upstream_code_sha256": tango.CODE_HASH,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "attribution": "Powered by Stability AI; TangoFlux / Hung et al.; CC0 wasserbjorn recordings",
        "license": "non-commercial research only; no engine admission or distribution",
        "prompt": PROMPT,
        "duration": SECONDS,
        "learning_rate": lr,
        "precision": "FP32 frozen weights and LoRA; BF16 training autocast; FP32 generation",
        "loss": (
            "half active 33-frame MSE plus half remaining 612-frame MSE; no CLAP reward"
            if objective == "balanced"
            else "upstream uniform full-horizon MSE; no CLAP reward"
        ),
        "objective": objective,
        "scope": "disclosed recording-disjoint development, not unseen objects or physical controls",
        "checkpoints": [],
        "training": [],
    }
    try:
        rows, waves, train, development = prepare_sources(source_root)
        report.update(
            {
                "sources": rows,
                "train_indices": train,
                "development_indices": development,
            }
        )
        pilot.save_report(output / "result.json", report)
        model, vae = tango.load_models(output)
        model.requires_grad_(False)
        vae.requires_grad_(False)
        means, stds, controls = cache_latents(vae, waves, rows, output)
        save_file({"mean": means, "std": stds}, output / "posterior.safetensors")
        report["vae_controls"] = controls
        model.to("cuda")
        condition = conditioning(model)
        # Compare cached-conditioning implementation against the exact upstream
        # unweighted SFT loss before adapting anything; same RNG/scheduler.
        latent = means[:1].to("cuda")
        with torch.no_grad():
            torch.manual_seed(765)
            upstream = model(
                latent, [PROMPT], duration=torch.tensor([SECONDS], device="cuda")
            )[0]
            torch.manual_seed(765)
            noise = torch.randn_like(latent)
            u = compute_density_for_timestep_sampling("logit_normal", 1, 0, 1, None)
            index = (u * 1000).long()
            sigma = model.noise_scheduler_copy.sigmas[index].to("cuda")
            prediction = velocity(
                model, (1 - sigma) * latent + sigma * noise, sigma, condition
            )
            cached = loss_parts(prediction, noise - latent)["full"]
            torch.testing.assert_close(upstream, cached, rtol=1e-5, atol=1e-6)
            report["upstream_loss_control"] = {
                "upstream": float(upstream),
                "cached": float(cached),
            }
        config = LoraConfig(
            r=8, lora_alpha=8, target_modules=["to_q", "to_v"], init_lora_weights=True
        )
        model.transformer.add_adapter(config)
        model.transformer.enable_gradient_checkpointing()
        parameters = [p for p in model.parameters() if p.requires_grad]
        names = [name for name, p in model.named_parameters() if p.requires_grad]
        if not names or any("lora_" not in name for name in names):
            raise ValueError("only LoRA parameters may train")
        report["adapter"] = {
            "rank": 8,
            "alpha": 8,
            "target_modules": ["to_q", "to_v"],
            "trainable_parameters": sum(p.numel() for p in parameters),
            "names": names,
        }
        model.text_encoder.to("cpu")
        torch.cuda.empty_cache()
        optimizer = torch.optim.AdamW(parameters, lr=lr, weight_decay=0.01)
        rng = torch.Generator().manual_seed(2026)
        checkpoints = {0, min(40, steps), steps}
        for step in range(steps + 1):
            if step in checkpoints:
                measured = {
                    "step": step,
                    "train": evaluate_flow(model, means, train, condition),
                    "development": evaluate_flow(model, means, development, condition),
                }
                if step:
                    weights = {
                        name: p.detach().cpu().contiguous()
                        for name, p in model.named_parameters()
                        if p.requires_grad
                    }
                    checkpoint = output / f"adapter-step{step}.safetensors"
                    save_file(weights, checkpoint)
                    measured["adapter_sha256"] = hashlib.sha256(
                        checkpoint.read_bytes()
                    ).hexdigest()
                measured["generation"] = render(
                    model, vae, output / f"step{step}", step
                )
                report["checkpoints"].append(measured)
                pilot.save_report(output / "result.json", report)
            if step == steps:
                break
            model.transformer.train()
            optimizer.zero_grad(set_to_none=True)
            index = train[int(torch.randint(len(train), (1,), generator=rng))]
            latent = (
                means[index : index + 1]
                + stds[index : index + 1] * torch.randn(means[:1].shape, generator=rng)
            ).to("cuda")
            noise = torch.randn(latent.shape, generator=rng).to("cuda")
            schedule_index = int(
                (torch.randn(1, generator=rng).sigmoid() * 1000).long()
            )
            sigma = model.noise_scheduler_copy.sigmas[schedule_index].to("cuda")
            with torch.autocast("cuda", dtype=torch.bfloat16):
                prediction = velocity(
                    model, (1 - sigma) * latent + sigma * noise, sigma, condition
                )
                loss = loss_parts(prediction, noise - latent)[objective]
            if not torch.isfinite(loss):
                raise ValueError("nonfinite training loss")
            loss.backward()
            norm = torch.nn.utils.clip_grad_norm_(
                parameters, 1.0, error_if_nonfinite=True
            )
            optimizer.step()
            report["training"].append(
                {
                    "step": step + 1,
                    "source_index": index,
                    "loss": float(loss.detach()),
                    "grad_norm": float(norm),
                }
            )
            report["steps_completed"] = step + 1
            if (step + 1) % 10 == 0:
                pilot.save_report(output / "result.json", report)
                print(json.dumps(report["training"][-1]), flush=True)
        report.update(
            {
                "status": "complete",
                "elapsed_seconds": time.monotonic() - started,
                "peak_cuda_allocated_bytes": torch.cuda.max_memory_allocated(),
            }
        )
    except BaseException as error:
        report.update({"status": "failed", "error": f"{type(error).__name__}: {error}"})
        pilot.save_report(output / "result.json", report)
        raise
    pilot.save_report(output / "result.json", report)
    print(
        json.dumps(
            {k: report[k] for k in ("status", "steps_completed", "elapsed_seconds")}
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sources", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--steps", type=int, default=120)
    parser.add_argument("--lr", type=float, default=1e-4)
    parser.add_argument("--objective", choices=("balanced", "full"), default="balanced")
    args = parser.parse_args()
    run(args.sources, args.output, args.steps, args.lr, args.objective)
