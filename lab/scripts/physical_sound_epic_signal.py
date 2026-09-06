"""Paired TRAIN-only impact conditioning audit; not source-free synthesis.

Powered by Stability AI. EPIC-SOUNDS: Huh/Chalk et al., CC-BY-NC4 research.
Temporal masks describe source support, not measured acoustic modes/decay.
"""

import argparse
import hashlib
from contextlib import nullcontext
from pathlib import Path

import numpy as np
import physical_sound_epic_pair_bridge as pair
import torch
from safetensors.torch import load_file
from scipy.io import wavfile
from scipy.signal import resample_poly

SIGMAS = (0.2, 0.5, 0.8)
REGIONS = ("pre_onset", "attack", "body", "padding_transition", "padded_tail")


def regions(wave):
    if wave.ndim != 1 or not 6000 <= len(wave) <= 72000 or not np.isfinite(wave).all():
        raise ValueError("finite source waveform required")
    hop = 240
    rms = np.array(
        [
            np.sqrt(np.mean(wave[i : i + hop].astype(np.float64) ** 2))
            for i in range(0, len(wave), hop)
        ]
    )
    if rms.max() == 0:
        raise ValueError("source has no energy")
    # Relative onset only locates source activity; it is not an audibility gate.
    onset = np.flatnonzero(rms >= 0.1 * rms.max())[0] * hop / 24000
    end = len(wave) / 24000
    time = (np.arange(645) + 0.5) * 2048 / 44100
    attack_end = min(end, onset + 0.1)
    masks = {
        "pre_onset": time < onset,
        "attack": (time >= onset) & (time < attack_end),
        "body": (time >= attack_end) & (time < end),
        "padding_transition": (time >= end) & (time < end + 0.25),
        "padded_tail": time >= end + 0.25,
    }
    if not np.all(np.sum(np.stack(list(masks.values())), axis=0) == 1):
        raise ValueError("region partition failed")
    return masks, {
        "onset_seconds": float(onset),
        "source_seconds": end,
        "attack_end_seconds": float(attack_end),
        "padding_guard_seconds": 0.25,
        "counts": {k: int(v.sum()) for k, v in masks.items()},
    }


def measure(prediction, target, masks):
    if prediction.shape != (1, 645, 64) or target.shape != prediction.shape:
        raise ValueError("native prediction/target shapes required")
    # FP64 accumulation avoids subtracting rounded float32 region losses.
    error = (
        (prediction.detach().double() - target.detach().double())
        .square()
        .mean((0, 2))
        .cpu()
        .numpy()
    )
    if not np.isfinite(error).all():
        raise ValueError("nonfinite errors")
    return {
        "full": float(error.mean()),
        **{k: float(error[m].mean()) if m.any() else None for k, m in masks.items()},
    }


def aggregate(rows):
    result = {}
    for precision in ("fp32", "bf16"):
        result[precision] = {}
        for region in ("full", *REGIONS):
            subset = [
                r
                for r in rows
                if r["precision"] == precision
                and r["errors"]["base"][region] is not None
            ]
            if not subset:
                continue
            base = np.array([r["errors"]["base"][region] for r in subset])
            matched = np.array([r["errors"][r["label"]][region] for r in subset])
            wrong = np.array(
                [
                    [
                        r["errors"][label][region]
                        for label in pair.source.CLASSES
                        if label != r["label"]
                    ]
                    for r in subset
                ]
            )
            result[precision][region] = {
                "n": len(subset),
                "base": float(base.mean()),
                "matched": float(matched.mean()),
                "wrong_mean": float(wrong.mean()),
                "base_minus_matched": float((base - matched).mean()),
                "contribution_to_full_gain": float(
                    sum(
                        [
                            (
                                r["errors"]["base"][region]
                                - r["errors"][r["label"]][region]
                            )
                            * (
                                1
                                if region == "full"
                                else r["regions"]["counts"][region] / 645
                            )
                            for r in subset
                        ]
                    )
                    / sum(r["precision"] == precision for r in rows)
                ),
                "wrong_minus_matched": float((wrong.mean(1) - matched).mean()),
                "beats_base": int((matched < base).sum()),
                "beats_all_wrong": int((matched < wrong.min(1)).sum()),
                "mean_frame_count": float(
                    np.mean(
                        [
                            645 if region == "full" else r["regions"]["counts"][region]
                            for r in subset
                        ]
                    )
                ),
            }
    return result


def write_diagnostic_comparison(output, rows):
    """Keep all disclosed diagnostic seeds; no source-free seed2718 selection."""
    if len(rows) != 20:
        raise ValueError("five complete source/posterior/base/matched groups required")
    pieces = []
    for row in rows:
        path = Path(row["wav"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]:
            raise ValueError("diagnostic PCM identity mismatch")
        rate, pcm = wavfile.read(path)
        if (
            rate != 16000
            or pcm.dtype != np.int16
            or pcm.ndim != 1
            or not 0 < len(pcm) <= 48000
            or abs(pcm.astype(np.int32)).max() > round(0.98 * 32767)
        ):
            raise ValueError("invalid diagnostic PCM")
        pieces.extend([pcm, np.zeros(8000, np.int16)])
    pcm = np.concatenate(pieces)
    path = output / "comparison.wav"
    wavfile.write(path, 16000, pcm)
    return {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "seconds": len(pcm) / 16000,
        "pcm_gain": 1,
        "reference_audio_input": True,
    }


def run(directory, output):
    output = pair.b.flow.c.v.phase.d.fresh_output(output)
    torch.set_num_threads(4)
    bridge, metadata = pair.load_bridge(directory)
    path = directory / "posterior.safetensors"
    if (
        not 0 < path.stat().st_size <= 80_000_000
        or hashlib.sha256(path.read_bytes()).hexdigest() != metadata["posterior_sha256"]
    ):
        raise ValueError("native posterior identity mismatch")
    tensors = load_file(path)
    sources = metadata["train_rows"]
    if (
        set(tensors) != {"mean", "std"}
        or any(
            t.shape != (len(sources), 645, 64) or not torch.isfinite(t).all()
            for t in tensors.values()
        )
        or (tensors["std"] < 0).any()
    ):
        raise ValueError("invalid native posterior")
    if any(
        r["class"] == pair.HELD_PAIR or r["participant_id"] in ("P04", "P07")
        for r in sources
    ):
        raise ValueError("TRAIN roles changed")
    waves = [pair.checked_wave(r) for r in sources]
    precision_controls = {
        next(i for i, r in enumerate(sources) if r["class"] == label)
        for label in pair.source.CLASSES
        if label != pair.HELD_PAIR
    }
    seen = {r["row"] for r in metadata["training"]}
    report = {
        "status": "running",
        "checkpoint_sha256": metadata["bridge_sha256"],
        "posterior_sha256": metadata["posterior_sha256"],
        "source_result": str(directory / "result.json"),
        "source_result_sha256": hashlib.sha256(
            (directory / "result.json").read_bytes()
        ).hexdigest(),
        "scope": "paired TRAIN diagnosis, reference-aided one-step media; NOT independent evaluation or source-free generation",
        "policy": "all TRAIN rows FP32, same posterior sample/noise per row, sigmas .2/.5/.8; base and all six conditions; BF16 saved-offset controls on first row per fitted pair only",
        "bf16_control_indices": sorted(precision_controls),
        "rows": [],
        "preview_rows": [],
        "training_performed": False,
    }
    save = lambda: pair.source.save(output / "result.json", report)
    save()
    try:
        model, vae = pair.b.train.tango.load_models(output)
        model.requires_grad_(False).eval().to("cuda")
        vae.requires_grad_(False)
        bridge.requires_grad_(False)
        if pair.b.digest(model) != metadata["frozen_model_after"]:
            raise ValueError("frozen generator differs")
        condition = pair.b.train.conditioning(model, pair.PROMPT, pair.SECONDS)
        model.text_encoder.to("cpu")
        torch.cuda.empty_cache()
        preview_latents = []
        previewed = set()
        for i, (source, wave) in enumerate(zip(sources, waves, strict=True)):
            masks, window = regions(wave)
            rng = torch.Generator().manual_seed(10000 + i)
            target = (
                tensors["mean"][i : i + 1]
                + tensors["std"][i : i + 1] * torch.randn((1, 645, 64), generator=rng)
            ).to("cuda")
            noise = torch.randn(target.shape, generator=rng).to("cuda")
            for sigma_value in SIGMAS:
                sigma = torch.tensor(sigma_value, device="cuda")
                noisy = (1 - sigma) * target + sigma * noise
                for precision in (
                    ("fp32", "bf16") if i in precision_controls else ("fp32",)
                ):
                    record = {
                        "row": i,
                        "annotation_id": source["annotation_id"],
                        "label": source["class"],
                        "exposed_during_fit": i in seen,
                        "sigma": sigma_value,
                        "precision": precision,
                        "regions": window,
                        "errors": {},
                    }
                    predictions = {}
                    for label in ("base", *pair.source.CLASSES):
                        with (
                            torch.no_grad(),
                            (
                                torch.autocast("cuda", dtype=torch.bfloat16)
                                if precision == "bf16"
                                else nullcontext()
                            ),
                        ):
                            adapted = (
                                condition
                                if label == "base"
                                else bridge.condition(
                                    torch.tensor(
                                        pair.controls(label)[None], device="cuda"
                                    ),
                                    *condition,
                                )
                            )
                            prediction = pair.b.train.velocity(
                                model, noisy, sigma, adapted
                            )
                        record["errors"][label] = measure(
                            prediction, noise - target, masks
                        )
                        if (
                            precision == "fp32"
                            and sigma_value == 0.5
                            and source["class"] not in previewed
                            and label in ("base", source["class"])
                        ):
                            predictions[label] = (
                                noisy - sigma * prediction.float()
                            ).cpu()
                    if predictions:
                        preview_latents.append(
                            {"row": i, "target": target.cpu(), **predictions}
                        )
                        previewed.add(source["class"])
                    report["rows"].append(record)
            if (i + 1) % 10 == 0:
                save()
                print({"audited": i + 1, "total": len(sources)}, flush=True)
        report["summary"] = aggregate(report["rows"])
        save()
        model.to("cpu")
        torch.cuda.empty_cache()
        vae.to("cuda")
        for item in preview_latents:
            i = item["row"]
            source = sources[i]
            native = resample_poly(waves[i], 147, 80)
            original, _ = pair.b.train.tango.publish(
                output, f"source-{i}", np.stack([native, native])
            )
            report["preview_rows"].append(
                {
                    **original,
                    "kind": "source",
                    "seed": 10000 + i,
                    "label": source["class"],
                }
            )
            for name, latent in item.items():
                if name == "row":
                    continue
                with torch.no_grad():
                    decoded = (
                        vae.decode(latent.to("cuda").transpose(1, 2))
                        .sample[0]
                        .cpu()
                        .numpy()
                    )
                artifact = pair.publish_generation(
                    output,
                    f"{i}-" + name.replace(" / ", "-").replace(" ", "-"),
                    decoded,
                    event_matched=True,
                )
                report["preview_rows"].append(
                    {
                        **artifact,
                        "kind": name,
                        "seed": 10000 + i,
                        "label": source["class"],
                        "reference_audio_input": True,
                    }
                )
        report["comparison"] = {
            **write_diagnostic_comparison(output, report["preview_rows"]),
            "order": "first TRAIN clip per fitted pair; source/posterior/base-one-step/matched-one-step; sigma .5, reference-aided, published gains",
        }
        report["status"] = "complete"
        save()
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--model", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    run(a.model, a.output)
