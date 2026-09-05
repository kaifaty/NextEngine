"""Factorized material-pair conditioning on frozen TangoFlux, EPIC research only.

Powered by Stability AI; Huh/Chalk et al. EPIC-SOUNDS CC-BY-NC4.
No target audio, object ID, participant or measured-physics claim at inference.
"""

import argparse
import hashlib
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import pandas as pd
import physical_sound_epic_slice as source
import physical_sound_pouring_bridge as b
import torch
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from scipy.signal import resample_poly, stft
from torch import nn

MATERIALS = ("metal", "glass", "wood", "plastic", "ceramic")
HELD_PAIR = "wood / glass collision"
PROMPT = "The sound of two objects colliding."
SECONDS = 3.0
FORMAT = "epic-factorized-material-bridge-v1"


def controls(label):
    pair = label.removesuffix(" collision").split(" / ")
    if label not in source.CLASSES or len(pair) != 2:
        raise ValueError("known unordered material pair required")
    return np.array([pair.count(m) / 2 for m in MATERIALS], np.float32)


class PairBridge(b.Bridge):
    input_width = 5

    def __init__(self):
        nn.Module.__init__(self)
        self.network = nn.Linear(5, 2048, bias=False)
        nn.init.zeros_(self.network.weight)
        self.register_buffer("offset", None, persistent=False)
        self.register_buffer("center_controls", None, persistent=False)


def select_training(frame):
    result = []
    for label in source.CLASSES:
        if label == HELD_PAIR:
            continue
        for participant in ("P01", "P02", "P03"):
            selected = []
            for row in frame[
                (frame["class"] == label) & (frame.participant_id == participant)
            ].to_dict("records"):
                if (
                    not 0.25
                    <= (row["stop_sample"] - row["start_sample"]) / source.RATE
                    <= 3
                ):
                    continue
                overlaps = frame[
                    (frame.video_id == row["video_id"])
                    & (frame.start_sample < row["stop_sample"])
                    & (frame.stop_sample > row["start_sample"])
                    & (frame.annotation_id != row["annotation_id"])
                ]
                if len(overlaps):
                    continue
                selected.append(row)
                if len(selected) == 3:
                    break
            if len(selected) != 3:
                raise ValueError("three TRAIN clips per participant/class required")
            result.extend(selected)
    if len(result) != 45 or len({r["annotation_id"] for r in result}) != 45:
        raise ValueError("unexpected training coverage")
    return result


def checked_wave(row):
    path = Path(row["wav"])
    if (
        path.stat().st_size > 200000
        or hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]
    ):
        raise ValueError("source clip identity mismatch")
    rate, pcm = wavfile.read(path)
    if (
        rate != 24000
        or pcm.dtype != np.int16
        or pcm.shape != (row["stop_sample"] - row["start_sample"],)
    ):
        raise ValueError("invalid source PCM")
    if not 6000 <= len(pcm) <= 72000 or abs(pcm.astype(np.int32)).max() > round(
        0.98 * 32767
    ):
        raise ValueError("source clip bounds")
    return pcm.astype(np.float32) / 32768


def data(root, output):
    manifest = json.loads((root / "result.json").read_text())
    if (
        manifest["status"] != "complete"
        or manifest["annotation_revision"] != source.ANNOTATIONS
    ):
        raise ValueError("source status/revision mismatch")
    for row in manifest["files"]:
        if (
            hashlib.sha256((root / row["path"]).read_bytes()).hexdigest()
            != row["sha256"]
        ):
            raise ValueError("metadata identity mismatch")
    frame = pd.read_csv(root / "train.csv")
    selected = select_training(frame)
    known = {row["annotation_id"]: row for row in manifest["rows"]}
    development = [
        row
        for row in manifest["rows"]
        if row["participant_id"] not in ("P01", "P02", "P03")
    ]
    if len(development) != 7 or {r["class"] for r in development} != set(
        source.CLASSES
    ):
        raise ValueError("development coverage changed")
    splits = pd.read_csv(root / "video-splits55.csv")
    hashes = pd.read_csv(root / "video-md5.csv")

    def acquire(row):
        if row["annotation_id"] in known:
            previous = known[row["annotation_id"]]
            if any(previous[k] != row[k] for k in row):
                raise ValueError("source annotation changed")
            checked_wave(previous)
            return previous
        return source.extract(row, source.video_source(row, splits, hashes), output)

    with ThreadPoolExecutor(max_workers=2) as pool:
        rows = list(pool.map(acquire, selected))
    for row in rows + development:
        checked_wave(row)
    return rows, development


@torch.no_grad()
def generate(
    model,
    vae,
    output,
    name,
    bridge=None,
    label=None,
    seed=2718,
    prompt=PROMPT,
    *,
    event_matched=False,
):
    model.requires_grad_(False).eval().to("cuda")
    vae.requires_grad_(False)
    torch.manual_seed(seed)

    def infer():
        return model.inference_flow(
            prompt,
            duration=SECONDS,
            num_inference_steps=50,
            guidance_scale=4.5,
            disable_progress=True,
        )

    if bridge is None:
        latent = infer()
    else:
        with bridge.hook(
            model.transformer, torch.tensor(controls(label)[None], device="cuda")
        ):
            latent = infer()
    model.to("cpu")
    torch.cuda.empty_cache()
    vae.to("cuda")
    wave = vae.decode(latent.transpose(1, 2)).sample[0].cpu().numpy()
    vae.to("cpu")
    torch.cuda.empty_cache()
    row = publish_generation(output, name, wave, event_matched=event_matched)
    print({"generated": name}, flush=True)
    return {**row, "seed": seed, "material_pair": label, "reference_audio_input": False}


def publish_generation(output, name, wave, *, event_matched):
    """Keep raw evidence; never amplify or score a silent prefix as an impact."""
    tango = b.train.tango
    prefix = wave[:, : round(SECONDS * tango.RATE)]
    if not event_matched:
        row, _ = tango.publish(output, name, prefix)
        return {**row, "postprocess": "prefix_diagnostic_only"}
    horizon = tango.horizon_metrics(wave, SECONDS)
    raw, _ = tango.publish(output, name + "-prefix", prefix)
    full, _ = tango.publish(output, name + "-full", wave)
    rate, pcm = wavfile.read(full["native_wav"])
    crop, event = tango.event_window(pcm.T.astype(np.float32) / 32767, SECONDS, rate)
    evidence = {
        "raw_prefix": raw,
        "full_horizon": full,
        "horizon_metrics": horizon,
        "event_window": event,
    }
    source.save(output / (name + "-extraction.json"), evidence)
    if crop is None:
        raise ValueError(
            "no detected event; retained horizon, no retry or amplification"
        )
    row, _ = tango.publish(output, name, crop)
    return {**row, **evidence, "postprocess": "matched_full_horizon_event"}


def profile(wave, rate):
    # Relative-power shape, independent of arbitrary recording/playback gain.
    _, _, z = stft(wave.astype(np.float64), fs=rate, nperseg=512, noverlap=256)
    power = np.mean(abs(z[1:]) ** 2, axis=1)
    if not np.isfinite(power).all() or power.max() <= 0:
        raise ValueError("invalid spectrum")
    db = 10 * np.log10(np.maximum(power / power.max(), 1e-10))
    return db - db.mean()


def load_bridge(directory):
    metadata, weights = directory / "result.json", directory / "bridge.safetensors"
    if metadata.stat().st_size > 2_000_000 or weights.stat().st_size > 100000:
        raise ValueError("oversized pair bridge")
    meta = json.loads(metadata.read_text())
    if (meta["format"], meta["status"], meta["revision"], meta["materials"]) != (
        FORMAT,
        "complete",
        b.train.tango.REVISION,
        list(MATERIALS),
    ):
        raise ValueError("pair bridge identity mismatch")
    if hashlib.sha256(weights.read_bytes()).hexdigest() != meta["bridge_sha256"]:
        raise ValueError("pair bridge hash mismatch")
    tensors = load_file(weights)
    if (
        set(tensors) != {"network.weight", "offset"}
        or tensors["offset"].shape != (1, 2048)
        or any(
            v.dtype != torch.float32 or not torch.isfinite(v).all()
            for v in tensors.values()
        )
    ):
        raise ValueError("invalid pair bridge tensors")
    bridge = PairBridge()
    bridge.offset = tensors.pop("offset")
    bridge.load_state_dict(tensors, strict=True)
    return bridge.eval().to("cuda"), meta


def compare_development(output, report, dev):
    preview = []
    metrics = []
    rankings = []
    report["reference_previews"] = []
    generated_profiles = {}
    for row in report["rows"]:
        rate, pcm = wavfile.read(row["wav"])
        generated_profiles[(row["seed"], row["material_pair"])] = profile(
            pcm.astype(np.float32) / 32768, rate
        )
    previewed = set()
    for real in dev:
        label = real["class"]
        show = label not in previewed
        previewed.add(label)
        wave = resample_poly(checked_wave(real), 2, 3)
        target_profile = profile(wave, 16000)
        original = b.train.pilot.write_audio(
            output / (real["annotation_id"] + "-preview.wav"), wave
        )
        report["reference_previews"].append(
            {**original, "annotation_id": real["annotation_id"]}
        )
        if show:
            preview.append({**original, "seed": 2718})
        for seed in (314, 2718):
            scores = {
                pair: float(
                    np.sqrt(
                        np.mean(
                            (generated_profiles[(seed, pair)] - target_profile) ** 2
                        )
                    )
                )
                for pair in source.CLASSES
            }
            order = sorted(scores, key=scores.get)
            rankings.append(
                {
                    "target": real["annotation_id"],
                    "label": label,
                    "seed": seed,
                    "scores": scores,
                    "matched_rank": order.index(label) + 1,
                    "nearest": order[0],
                }
            )
            baseline = next(
                r for r in report["rows"] if r["kind"] == "base" and r["seed"] == seed
            )
            matched = next(
                r
                for r in report["rows"]
                if r["material_pair"] == label and r["seed"] == seed
            )
            swapped = next(
                r
                for r in report["rows"]
                if r["material_pair"]
                == source.CLASSES[(source.CLASSES.index(label) + 1) % 6]
                and r["seed"] == seed
            )
            for kind, row in [
                ("base", baseline),
                ("matched", matched),
                ("swapped", swapped),
            ]:
                # Entire fixed onset window, never a reference-dependent crop.
                metrics.append(
                    {
                        "label": label,
                        "seed": seed,
                        "kind": kind,
                        "target_annotation": real["annotation_id"],
                        "shape_rmse_db": float(
                            np.sqrt(
                                np.mean(
                                    (
                                        generated_profiles[(seed, row["material_pair"])]
                                        - target_profile
                                    )
                                    ** 2
                                )
                            )
                        ),
                    }
                )
            if seed == 2718 and show:
                preview.extend([baseline, matched, swapped])
    report["metrics"] = metrics
    report["all_material_ranking"] = {
        "scope": "gain-invariant spectrum diagnostic; not calibrated material truth",
        "rows": rankings,
        "top1_count": sum(r["matched_rank"] == 1 for r in rankings),
        "n": len(rankings),
    }
    report["comparison"] = {
        **b.write_comparison(output, preview),
        "order": "six classes in source order; each real/base/matched/next-pair swap,seed2718,published gains",
    }


def expanded_data(root, expanded):
    original = json.loads((root / "result.json").read_text())
    if (
        original["status"] != "complete"
        or original["annotation_revision"] != source.ANNOTATIONS
    ):
        raise ValueError("original source status/revision mismatch")
    path = expanded / "result.json"
    if not 0 < path.stat().st_size <= 2_000_000:
        raise ValueError("bounded expanded source required")
    manifest = json.loads(path.read_text())
    if (
        manifest["status"],
        manifest["annotation_revision"],
        manifest["source_result_sha256"],
    ) != (
        "complete",
        source.ANNOTATIONS,
        hashlib.sha256((root / "result.json").read_bytes()).hexdigest(),
    ):
        raise ValueError("expanded source identity mismatch")
    for row in original["files"]:
        if (
            hashlib.sha256((root / row["path"]).read_bytes()).hexdigest()
            != row["sha256"]
        ):
            raise ValueError("source metadata changed")
    frame = pd.read_csv(root / "train.csv")
    train = [r for r in manifest["rows"] if r["class"] != HELD_PAIR]
    if not 45 < len(train) <= 216 or len({r["annotation_id"] for r in train}) != len(
        train
    ):
        raise ValueError("bounded unique expanded training rows required")
    if {r["class"] for r in train} != set(source.CLASSES) - {HELD_PAIR} or any(
        r["participant_id"] in ("P04", "P07") for r in train
    ):
        raise ValueError("expanded training roles changed")
    for row in train:
        matches = frame[frame.annotation_id == row["annotation_id"]]
        if len(matches) != 1 or any(
            row[k] != matches.iloc[0][k] for k in frame.columns
        ):
            raise ValueError("expanded annotation mismatch")
        overlaps = frame[
            (frame.video_id == row["video_id"])
            & (frame.start_sample < row["stop_sample"])
            & (frame.stop_sample > row["start_sample"])
            & (frame.annotation_id != row["annotation_id"])
        ]
        if len(overlaps):
            raise ValueError("expanded annotation overlaps another event")
        checked_wave(row)
    dev = [
        r for r in original["rows"] if r["participant_id"] not in ("P01", "P02", "P03")
    ]
    if len(dev) != 7 or {r["class"] for r in dev} != set(source.CLASSES):
        raise ValueError("development coverage changed")
    for row in dev:
        checked_wave(row)
    return train, dev


def run(root, output, expanded=None):
    output = b.flow.c.v.phase.d.fresh_output(output)
    report = {
        "format": FORMAT,
        "status": "running",
        "revision": b.train.tango.REVISION,
        "source_result_sha256": hashlib.sha256(
            (root / "result.json").read_bytes()
        ).hexdigest(),
        "scope": "noncommercial research; categorical unordered materials only; no reference input, geometry/force calibration, object-independent or foundation-independent claim",
        "held_pair": HELD_PAIR,
        "materials": list(MATERIALS),
        "training": [],
        "rows": [],
        "checks": {},
    }
    save = lambda: source.save(output / "result.json", report)
    save()
    try:
        train, dev = (
            data(root, output) if expanded is None else expanded_data(root, expanded)
        )
        if expanded is not None:
            report["expanded_source_result"] = str(expanded / "result.json")
            report["expanded_source_sha256"] = hashlib.sha256(
                (expanded / "result.json").read_bytes()
            ).hexdigest()
        report.update(train_rows=train, development_rows=dev)
        save()
        print({"train": len(train), "development": len(dev)}, flush=True)
        torch.set_num_threads(4)
        torch.manual_seed(53)
        model, vae = b.train.tango.load_models(output)
        model.requires_grad_(False)
        vae.requires_grad_(False)
        report["frozen_model_before"] = b.digest(model)
        means, stds = [], []
        vae.to("cuda")
        with torch.no_grad():
            for row in train:
                mono = resample_poly(checked_wave(row), 147, 80)
                wave = np.pad(
                    np.stack([mono, mono]), ((0, 0), (0, 30 * 44100 - len(mono)))
                )
                posterior = vae.encode(
                    torch.tensor(wave[None], device="cuda")
                ).latent_dist
                means.append(posterior.mean.transpose(1, 2).cpu())
                stds.append(posterior.std.transpose(1, 2).cpu())
        vae.to("cpu")
        torch.cuda.empty_cache()
        means, stds = torch.cat(means), torch.cat(stds)
        if (
            means.shape != (len(train), 645, 64)
            or stds.shape != means.shape
            or not torch.isfinite(means).all()
            or not torch.isfinite(stds).all()
        ):
            raise ValueError("invalid native posteriors")
        save_file({"mean": means, "std": stds}, output / "posterior.safetensors")
        report["posterior_sha256"] = hashlib.sha256(
            (output / "posterior.safetensors").read_bytes()
        ).hexdigest()
        vectors = torch.tensor(
            np.stack([controls(r["class"]) for r in train]), device="cuda"
        )
        bridge = PairBridge().to("cuda")
        bridge.center_controls = vectors
        report["parameters"] = sum(p.numel() for p in bridge.parameters())
        initial = generate(model, vae, output, "initial-base")
        zero = generate(model, vae, output, "initial-zero", bridge, source.CLASSES[0])
        if any(initial[k] != zero[k] for k in ("sha256", "native_sha256")):
            raise ValueError("zero bridge changed generator")
        report["initial"] = [initial, zero]
        report["checks"]["zero_exact"] = True
        regressions = {
            k: generate(model, vae, output, "before-" + k, prompt=p)
            for k, p in [("water", b.PROMPT), ("rain", "The sound of rain falling.")]
        }
        report["regression_before"] = regressions
        save()
        model.to("cuda")
        condition = b.train.conditioning(model, PROMPT, SECONDS)
        from diffusers.training_utils import compute_density_for_timestep_sampling

        with torch.no_grad(), torch.random.fork_rng(devices=[0]):
            target = means[:1].to("cuda")
            torch.manual_seed(765)
            upstream = model(
                target, [PROMPT], duration=torch.tensor([SECONDS], device="cuda")
            )[0]
            torch.manual_seed(765)
            noise = torch.randn_like(target)
            u = compute_density_for_timestep_sampling("logit_normal", 1, 0, 1, None)
            sigma = model.noise_scheduler_copy.sigmas[(u * 1000).long()].to("cuda")
            prediction = b.train.velocity(
                model, (1 - sigma) * target + sigma * noise, sigma, condition
            )
            cached = b.train.loss_parts(prediction, noise - target, 65)["full"]
            torch.testing.assert_close(upstream, cached, rtol=1e-5, atol=1e-6)
            report["checks"]["upstream_loss"] = {
                "upstream": float(upstream),
                "cached": float(cached),
            }
        model.text_encoder.to("cpu")
        torch.cuda.empty_cache()
        model.transformer.enable_gradient_checkpointing()
        model.transformer.train()
        optimizer = torch.optim.AdamW(bridge.parameters(), lr=1e-4, weight_decay=0.01)
        rng = torch.Generator().manual_seed(2026)
        for step in range(200):
            i = int(torch.randint(len(train), (1,), generator=rng))
            target = (
                means[i : i + 1]
                + stds[i : i + 1] * torch.randn(means[:1].shape, generator=rng)
            ).to("cuda")
            noise = torch.randn(target.shape, generator=rng).to("cuda")
            index = int((torch.randn(1, generator=rng).sigmoid() * 1000).long())
            sigma = model.noise_scheduler_copy.sigmas[index].to("cuda")
            optimizer.zero_grad(set_to_none=True)
            with torch.autocast("cuda", dtype=torch.bfloat16):
                adapted = bridge.condition(vectors[i : i + 1], *condition)
                prediction = b.train.velocity(
                    model, (1 - sigma) * target + sigma * noise, sigma, adapted
                )
                parts = b.train.loss_parts(prediction, noise - target, 65)
                loss = parts["full"]
            if not torch.isfinite(loss):
                raise ValueError("nonfinite loss")
            loss.backward()
            norm = torch.nn.utils.clip_grad_norm_(
                bridge.parameters(), 1, error_if_nonfinite=True
            )
            if bridge.network.weight.grad is None or any(
                p.grad is not None for p in model.parameters()
            ):
                raise ValueError("gradient ownership failure")
            optimizer.step()
            report["training"].append(
                {
                    "step": step + 1,
                    "row": i,
                    "loss": float(loss.detach()),
                    "active_loss": float(parts["active"].detach()),
                    "grad_norm": float(norm),
                }
            )
            if (step + 1) % 20 == 0:
                save()
                print(report["training"][-1], flush=True)
        del optimizer
        model.transformer.eval()
        bridge.eval()
        bridge.freeze_centering()
        report["frozen_model_after"] = b.digest(model)
        if report["frozen_model_after"] != report["frozen_model_before"]:
            raise ValueError("frozen weights changed")
        report["checks"]["frozen_model_exact"] = True
        save_file(
            {
                **{k: v.cpu().contiguous() for k, v in bridge.state_dict().items()},
                "offset": bridge.offset.cpu().contiguous(),
            },
            output / "bridge.safetensors",
        )
        report["bridge_sha256"] = hashlib.sha256(
            (output / "bridge.safetensors").read_bytes()
        ).hexdigest()
        save()
        for seed in (314, 2718):
            base = generate(
                model, vae, output, f"base-{seed}", seed=seed, event_matched=True
            )
            report["rows"].append({**base, "kind": "base"})
            for i, label in enumerate(source.CLASSES):
                row = generate(
                    model,
                    vae,
                    output,
                    f"pair{i}-{seed}",
                    bridge,
                    label,
                    seed,
                    event_matched=True,
                )
                report["rows"].append(
                    {**row, "kind": "matched", "training_pair": label != HELD_PAIR}
                )
                save()
        report["regression_after"] = {
            k: generate(model, vae, output, "after-" + k, prompt=p)
            for k, p in [("water", b.PROMPT), ("rain", "The sound of rain falling.")]
        }
        for key, old in regressions.items():
            if any(
                old[k] != report["regression_after"][key][k]
                for k in ("sha256", "native_sha256")
            ):
                raise ValueError("adapter-off regression changed")
        report["checks"]["regression_exact"] = True
        compare_development(output, report, dev)
        report["status"] = "complete"
        save()
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


def event_matrix(directory, output):
    """Re-evaluate an existing checkpoint with matched onset extraction; no fitting."""
    torch.set_num_threads(4)
    bridge, meta = load_bridge(directory)
    output = b.flow.c.v.phase.d.fresh_output(output)
    report = {
        "status": "running",
        "format": FORMAT + "-event-evaluation",
        "checkpoint_result": str(directory / "result.json"),
        "checkpoint_result_sha256": hashlib.sha256(
            (directory / "result.json").read_bytes()
        ).hexdigest(),
        "bridge_sha256": meta["bridge_sha256"],
        "rows": [],
        "reference_audio_input": False,
        "training_performed": False,
        "seconds": SECONDS,
        "steps": 50,
        "guidance_scale": 4.5,
        "prompt": PROMPT,
        "policy": "same full-horizon 10ms RMS onset extraction for base and all pairs; no amplification or seed selection",
        "development_rows": meta["development_rows"],
    }
    save = lambda: source.save(output / "result.json", report)
    save()
    try:
        for row in meta["development_rows"]:
            checked_wave(row)
        model, vae = b.train.tango.load_models(output)
        if b.digest(model) != meta["frozen_model_after"]:
            raise ValueError("generator differs")
        for seed in (314, 2718):
            base = generate(
                model, vae, output, f"base-{seed}", seed=seed, event_matched=True
            )
            report["rows"].append({**base, "kind": "base"})
            save()
            for i, label in enumerate(source.CLASSES):
                row = generate(
                    model,
                    vae,
                    output,
                    f"pair{i}-{seed}",
                    bridge,
                    label,
                    seed,
                    event_matched=True,
                )
                report["rows"].append(
                    {**row, "kind": "matched", "training_pair": label != HELD_PAIR}
                )
                save()
        compare_development(output, report, meta["development_rows"])
        report["status"] = "complete"
        save()
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    inputs = p.add_mutually_exclusive_group(required=True)
    inputs.add_argument("--source", type=Path)
    inputs.add_argument("--model", type=Path)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--pair", choices=source.CLASSES)
    p.add_argument("--seed", type=int, default=2718)
    p.add_argument("--event-matrix", action="store_true")
    p.add_argument(
        "--expanded",
        type=Path,
        help="completed broader TRAIN source; excludes held pair and P04/P07",
    )
    modes = p.add_mutually_exclusive_group()
    modes.add_argument(
        "--event-window",
        action="store_true",
        help="default: retained full horizon and onset crop",
    )
    modes.add_argument(
        "--prefix-diagnostic",
        action="store_true",
        help="save first seconds only; not impact-quality evidence",
    )
    a = p.parse_args()
    if a.expanded is not None and a.source is None:
        raise ValueError("expanded data requires --source training")
    if a.source and (a.event_matrix or a.event_window or a.prefix_diagnostic):
        raise ValueError("event options require saved --model")
    if a.source:
        run(a.source, a.output, a.expanded)
    elif a.event_matrix:
        if a.pair is not None or a.event_window or a.prefix_diagnostic:
            raise ValueError("matrix fixes all pairs and applies event extraction")
        event_matrix(a.model, a.output)
    else:
        if a.pair is None or not 0 <= a.seed < 2**32:
            raise ValueError("pair and uint32 seed required")
        torch.set_num_threads(4)
        bridge, meta = load_bridge(a.model)
        output = b.flow.c.v.phase.d.fresh_output(a.output)
        model, vae = b.train.tango.load_models(output)
        if b.digest(model) != meta["frozen_model_after"]:
            raise ValueError("generator differs")
        row = generate(
            model,
            vae,
            output,
            "generated",
            bridge,
            a.pair,
            a.seed,
            event_matched=not a.prefix_diagnostic,
        )
        source.save(
            output / "result.json",
            {
                "reference_audio_input": False,
                "bridge_sha256": meta["bridge_sha256"],
                "rows": [row],
            },
        )
