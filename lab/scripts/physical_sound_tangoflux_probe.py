"""Same-seed duration/CFG counterfactual after a rejected glass LoRA fit.

Powered by Stability AI; TangoFlux / Hung et al., non-commercial research only.
No training, reference-audio input, metric optimization or engine promotion.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_tangoflux_pilot as tango
import physical_sound_tangoflux_train as train
import physical_sound_text_pilot as pilot
import physical_sound_text_tags as tags
import torch
from scipy.io import wavfile


def guided_prediction(conditional, unconditional, scale):
    if not np.isfinite(scale) or not 1 <= scale <= 5:
        raise ValueError("guidance must be bounded between 1 and 5")
    # Avoid cancellation at scale one: this is conditional generation, not
    # unconditional generation. The HF revision's scale<=1 branch has a bad
    # encode_text keyword; do not modify or execute that branch.
    return (
        conditional
        if scale == 1
        else unconditional + scale * (conditional - unconditional)
    )


@torch.no_grad()
def generate(model, duration, guidance, mode, seed, steps=50):
    if mode not in ("base", "adapted", "base-unconditional"):
        raise ValueError("unknown adapter counterfactual")
    if not 1 <= duration <= 10 or not 1 <= steps <= 100:
        raise ValueError("bounded duration/steps required")
    guided_prediction(torch.zeros(1), torch.zeros(1), guidance)
    transformer = model.transformer
    device = transformer.device
    torch.manual_seed(seed)
    transformer.disable_adapters() if mode == "base" else transformer.enable_adapters()
    duration_hidden = model.encode_duration(
        torch.tensor([duration], device=device)
    ).repeat(2, 1, 1)
    hidden, mask = model.encode_text_classifier_free([train.PROMPT])
    pooled = model.fc(torch.where(mask[..., None], hidden, float("nan")).nanmean(1))
    hidden = torch.cat([hidden, duration_hidden], dim=1)
    scheduler = model.noise_scheduler
    scheduler.set_timesteps(sigmas=np.linspace(1.0, 1 / steps, steps), device=device)
    latent = torch.randn(1, model.audio_seq_len, 64).to(device)
    inputs = {
        "encoder_hidden_states": hidden,
        "pooled_projections": pooled,
        "txt_ids": torch.zeros(2, hidden.shape[1], 3, device=device),
        "img_ids": torch.arange(model.audio_seq_len, device=device)[
            None, :, None
        ].repeat(2, 1, 3),
        "guidance": None,
        "return_dict": False,
    }
    for timestep in scheduler.timesteps:
        inputs.update(
            hidden_states=torch.cat([latent, latent]),
            timestep=(timestep / 1000).reshape(1),
        )
        unconditional, conditional = transformer(**inputs)[0].chunk(2)
        if mode == "base-unconditional" and guidance != 1:
            # Same evolving latent and batch shape; only unconditional weights
            # differ. Restore the adapter even if the diagnostic forward fails.
            transformer.disable_adapters()
            try:
                unconditional = transformer(**inputs)[0][:1]
            finally:
                transformer.enable_adapters()
        prediction = guided_prediction(conditional, unconditional, guidance)
        latent = scheduler.step(prediction, timestep, latent).prev_sample
    return latent


def paired_summary(rows):
    indexed = {(r["mode"], r["duration"], r["guidance"], r["seed"]): r for r in rows}
    if len(indexed) != len(rows):
        raise ValueError("duplicate counterfactual row")
    groups = []
    for duration in (1.5, 5.0):
        for guidance in (1.0, 2.0, 4.5):
            for mode in ("adapted", "base-unconditional"):
                if mode == "base-unconditional" and guidance != 4.5:
                    continue
                deltas = []
                for seed in (42, 123):
                    base = indexed[("base", duration, guidance, seed)]
                    candidate = indexed[(mode, duration, guidance, seed)]
                    deltas.append(candidate["clap"][0] - base["clap"][0])
                groups.append(
                    {
                        "duration": duration,
                        "guidance": guidance,
                        "mode": mode,
                        "target_cosine_deltas": deltas,
                        "mean_delta": float(np.mean(deltas)),
                        "both_seeds_improve": all(d > 0 for d in deltas),
                    }
                )
    return groups


def score_positive_controls(checkpoint, output):
    training = json.loads((checkpoint.parent / "result.json").read_text())
    rows = [
        {"source_id": control["source_id"], "kind": kind, **control[kind]}
        for control in training["vae_controls"]
        for kind in ("original", "vae")
    ]
    prompts = [prompt for _, prompt in train.CASES]
    scores = tags.clap_similarities(prompts, rows)
    for row, score in zip(rows, scores, strict=True):
        row["similarities"] = score
    pilot.save_report(
        output / "real-positive-controls.json",
        {
            "scope": "disclosed original/VAE positive controls; not clean held-out data",
            "prompts": prompts,
            "rows": rows,
        },
    )


def run(checkpoint, output):
    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must be a new external directory")
    output.mkdir(parents=True)
    shutil.copyfile(__file__, output / "executed-script.py")
    torch.set_num_threads(4)
    torch.manual_seed(0)
    started = time.monotonic()
    report = {
        "status": "running",
        "model": tango.MODEL,
        "revision": tango.REVISION,
        "prompt": train.PROMPT,
        "reference_audio_input": False,
        "new_training_steps": 0,
        "steps": 50,
        "precision": "float32",
        "seconds": 5.0,
        "cases": [{"id": "glass-metal", "prompt": train.PROMPT}],
        "controls": [],
        "rows": [],
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "attribution": "Powered by Stability AI; TangoFlux / Hung et al., non-commercial research only",
        "hypotheses": [
            "short requested duration explains degradation",
            "guidance amplifies adaptation error",
            "adapted unconditional branch causes degradation",
            "fit changes are not a free-generation gain",
        ],
        "scope": "disclosed same-prompt counterfactual, not physical generalization or naturalness admission",
    }
    try:
        model, vae = tango.load_models(output)
        report["adapter"] = tango.load_adapter(model, checkpoint)
        model.eval().to("cuda")
        vae.requires_grad_(False)
        # Exact upstream control at its supported CFG path before this custom
        # path can produce evidence. No reference recording is involved.
        model.transformer.disable_adapters()
        torch.manual_seed(42)
        expected = model.inference_flow(
            train.PROMPT,
            duration=1.5,
            num_inference_steps=50,
            guidance_scale=4.5,
            disable_progress=True,
        )
        actual = generate(model, 1.5, 4.5, "base", 42)
        torch.testing.assert_close(expected, actual, rtol=0, atol=0)
        report["upstream_latent_exact"] = True
        for duration in (1.5, 5.0):
            for guidance in (1.0, 2.0, 4.5):
                for seed in (42, 123):
                    modes = (
                        ("base", "adapted", "base-unconditional")
                        if guidance == 4.5
                        else ("base", "adapted")
                    )
                    for mode in modes:
                        latent = generate(model, duration, guidance, mode, seed)
                        model.to("cpu")
                        torch.cuda.empty_cache()
                        vae.to("cuda")
                        with torch.no_grad():
                            wave = (
                                vae.decode(latent.transpose(2, 1))
                                .sample[0, :, : int(duration * tango.RATE)]
                                .cpu()
                                .numpy()
                            )
                        vae.to("cpu")
                        torch.cuda.empty_cache()
                        model.to("cuda")
                        name = f"{mode}-duration{duration}-cfg{guidance}-seed{seed}"
                        record, _ = tango.publish(output, name, wave)
                        record.update(
                            case=0,
                            mode=mode,
                            duration=duration,
                            guidance=guidance,
                            seed=seed,
                        )
                        # The original short/default-CFG cases must reproduce
                        # published WAVs for both the base and trained adapter.
                        if (
                            duration == 1.5
                            and guidance == 4.5
                            and mode != "base-unconditional"
                        ):
                            stage = "step0" if mode == "base" else "step120"
                            old = json.loads(
                                (checkpoint.parent / stage / "result.json").read_text()
                            )
                            previous = next(
                                r
                                for r in old["rows"]
                                if r["case"] == 0 and r["seed"] == seed
                            )
                            if any(
                                record[k] != previous[k]
                                for k in ("sha256", "native_sha256")
                            ):
                                raise ValueError("published generation replay differs")
                            record["published_replay_exact"] = True
                        report["rows"].append(record)
                        pilot.save_report(output / "result.json", report)
                        print(json.dumps({"generated": name}), flush=True)
        del model, vae
        torch.cuda.empty_cache()
        # Same frozen scorers; no scores feed back into generation or fitting.
        similarities = tags.clap_similarities(
            [p for _, p in train.CASES], report["rows"]
        )
        for row, scores in zip(report["rows"], similarities, strict=True):
            row["clap"] = scores
        report["clap_prompts"] = [{"id": k, "prompt": p} for k, p in train.CASES]
        report["paired_summary"] = paired_summary(report["rows"])
        # Re-score the historical default controls with the factored scorer:
        # refactoring must not manufacture the measured differences.
        for row in report["rows"]:
            if row.get("published_replay_exact"):
                stage = "step0" if row["mode"] == "base" else "step120"
                old = json.loads(
                    (checkpoint.parent / stage / "ast-clap.json").read_text()
                )["clap"]
                previous = next(
                    r
                    for r in old["rows"]
                    if r["case"] == 0 and r["seed"] == row["seed"]
                )
                np.testing.assert_allclose(
                    row["clap"], previous["similarities"], atol=1e-6, rtol=0
                )
        report["scorer_replay_within_1e_6"] = True
        score_positive_controls(checkpoint, output)
        preview = []
        for row in report["rows"]:
            if row["seed"] == 123 and row["guidance"] in (1.0, 4.5):
                rate, pcm = wavfile.read(row["wav"])
                preview.extend(
                    (pcm.astype(np.float32) / 32768, np.zeros(rate // 2, np.float32))
                )
        pilot.write_audio(output / "comparison.wav", np.concatenate(preview))
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
        pilot.save_report(output / "result.json", report)
        tags.run(output / "result.json", output / "ast-tags.json", [], with_clap=False)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        pilot.save_report(output / "result.json", report)
        raise
    print(
        json.dumps(
            {
                "status": report["status"],
                "elapsed_seconds": report["elapsed_seconds"],
                "paired_summary": report["paired_summary"],
            }
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.checkpoint.resolve(), args.output)
