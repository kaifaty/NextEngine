"""Audible VAE mean/posterior-sample controls for the disclosed glass targets.

Powered by Stability AI; TangoFlux / Hung et al., non-commercial research only.
This reconstructs reference audio. It is not text-only generation or admission.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_audible_glass as glass
import physical_sound_tangoflux_pilot as tango
import physical_sound_tangoflux_train as train
import physical_sound_text_pilot as pilot
import physical_sound_text_tags as tags
import torch
from scipy.signal import resample_poly

POSTERIOR_HASH = "03a024064bf211f9f7d2773ac4df62fcf49f1334fd2059571353152810754631"


def posterior_sample(mean, std, seed):
    if (
        mean.shape != std.shape
        or not torch.isfinite(mean).all()
        or not torch.isfinite(std).all()
        or (std < 0).any()
    ):
        raise ValueError("invalid posterior tensors")
    rng = torch.Generator(device=mean.device).manual_seed(seed)
    return mean + std * torch.randn(mean.shape, generator=rng, device=mean.device)


def energy_regions(wave, rate=44100):
    if wave.ndim != 2 or wave.shape != (2, 5 * rate) or not np.isfinite(wave).all():
        raise ValueError("expected finite five-second stereo audio")
    split = round(1.5 * rate)
    active = float(np.sqrt(np.mean(wave[:, :split].astype(np.float64) ** 2)))
    tail = float(np.sqrt(np.mean(wave[:, split:].astype(np.float64) ** 2)))
    return {
        "active_rms": active,
        "padding_rms": tail,
        "padding_dbfs": 20 * math_log10(tail),
        "padding_to_active_db": 20 * (math_log10(tail) - math_log10(active)),
    }


def math_log10(value):
    return float(np.log10(max(value, 1e-12)))


def run(fit, output):
    from diffusers import AutoencoderOobleck
    from huggingface_hub import snapshot_download
    from safetensors.torch import load_file

    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must be a new external directory")
    posterior_path = fit / "posterior.safetensors"
    if hashlib.sha256(posterior_path.read_bytes()).hexdigest() != POSTERIOR_HASH:
        raise ValueError("disclosed posterior identity changed")
    training = json.loads((fit / "result.json").read_text())
    if training["status"] != "complete" or training["revision"] != tango.REVISION:
        raise ValueError("incomplete or incompatible training evidence")
    output.mkdir(parents=True)
    shutil.copyfile(__file__, output / "executed-script.py")
    torch.set_num_threads(4)
    started = time.monotonic()
    report = {
        "status": "running",
        "model": tango.MODEL,
        "revision": tango.REVISION,
        "posterior_sha256": POSTERIOR_HASH,
        "reference_audio_input": True,
        "scope": "codec-target diagnostic only; no new text generator or independent holdout",
        "new_training_steps": 0,
        "seconds": 1.5,
        "attribution": "Powered by Stability AI; TangoFlux / Hung et al.; CC0 wasserbjorn source recordings",
        "cases": [{"id": "real-glass", "prompt": train.PROMPT}],
        "controls": [],
        "rows": [],
    }
    try:
        source = Path(
            snapshot_download(
                tango.MODEL, revision=tango.REVISION, local_files_only=True
            )
        )
        vae = AutoencoderOobleck().eval().requires_grad_(False)
        vae.load_state_dict(load_file(source / "vae.safetensors"), strict=True)
        report["vae_sha256"] = hashlib.sha256(
            (source / "vae.safetensors").read_bytes()
        ).hexdigest()
        for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
            shutil.copyfile(source / name, output / name)
        vae.to("cuda")
        posterior = load_file(posterior_path)
        if set(posterior) != {"mean", "std"} or posterior["mean"].shape != (
            27,
            645,
            64,
        ):
            raise ValueError("unexpected posterior cache shape")
        preview = []
        for control in training["vae_controls"]:
            index = next(
                i
                for i, row in enumerate(training["sources"])
                if row["id"] == control["source_id"]
            )
            reference = tags.load_audio(
                Path(control["original"]["wav"]), control["original"]["sha256"]
            )
            target = torch.from_numpy(resample_poly(reference, 2, 1))[None]
            loss = glass.AudioLoss(target)
            report["rows"].append(
                {
                    "case": 0,
                    "source_id": control["source_id"],
                    "variant": "original",
                    "seed": 0,
                    **control["original"],
                }
            )
            selected = [reference]
            for variant, seed in (
                ("mean", 0),
                ("sample", 0),
                ("sample", 42),
                ("sample", 123),
            ):
                mean = posterior["mean"][index : index + 1]
                std = posterior["std"][index : index + 1]
                latent = (
                    mean if variant == "mean" else posterior_sample(mean, std, seed)
                )
                with torch.no_grad():
                    wave = (
                        vae.decode(latent.transpose(1, 2).to("cuda"))
                        .sample[0, :, : 5 * tango.RATE]
                        .cpu()
                        .numpy()
                    )
                name = f"{control['source_id']}-{variant}-seed{seed}"
                record, _ = tango.publish(
                    output, name, wave[:, : round(1.5 * tango.RATE)]
                )
                context, _ = tango.publish(output, name + "-with-padding", wave)
                if variant == "mean" and any(
                    record[k] != control["vae"][k] for k in ("sha256", "native_sha256")
                ):
                    raise ValueError("published mean reconstruction no longer replays")
                pcm = tags.load_audio(Path(record["wav"]), record["sha256"])
                spectral, envelope = loss.parts(
                    torch.from_numpy(resample_poly(pcm, 2, 1))[None]
                )
                report["rows"].append(
                    {
                        "case": 0,
                        "source_id": control["source_id"],
                        "variant": variant,
                        "seed": seed,
                        **record,
                        "context": context,
                        **energy_regions(wave),
                        "relative_spectral": float(spectral[0]),
                        "relative_envelope": float(envelope[0]),
                    }
                )
                if variant == "mean" or seed == 42:
                    selected.append(pcm)
                print(
                    json.dumps(
                        {
                            "decoded": name,
                            "spectral": float(spectral[0]),
                            **energy_regions(wave),
                        }
                    ),
                    flush=True,
                )
                pilot.save_report(output / "result.json", report)
            for audio in selected:
                preview.extend((audio, np.zeros(8000, np.float32)))
            pilot.write_audio(output / "comparison.wav", np.concatenate(preview))
        del vae
        torch.cuda.empty_cache()
        scores = tags.clap_similarities([p for _, p in train.CASES], report["rows"])
        for row, similarities in zip(report["rows"], scores, strict=True):
            row["clap"] = similarities
        report["clap_prompts"] = [p for _, p in train.CASES]
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
        pilot.save_report(output / "result.json", report)
        tags.run(output / "result.json", output / "ast-tags.json", [], with_clap=False)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        pilot.save_report(output / "result.json", report)
        raise
    print(
        json.dumps(
            {"status": report["status"], "elapsed_seconds": report["elapsed_seconds"]}
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fit", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.fit.resolve(), args.output)
