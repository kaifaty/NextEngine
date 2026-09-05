"""Paired TRAIN-only endpoint or full-sampler acoustic texture fine-tuning.

Report-only, fixed rubber probe. Source-free full-event inference is unchanged.
Eguchi et al./CC BY4; Powered by Stability AI/TangoFlux. Auxiliary endpoint-loss
motivation: WaveFM, NAACL2025; this latent/texture experiment is not a replication.
"""

from __future__ import annotations

import argparse
import copy
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_texture_surface as surface
import torch
from safetensors.torch import load_file, save_file
from scipy.signal import firwin
from torch.nn import functional as F

event, flow = surface.event, surface.flow
STEPS = 200
WEIGHT = 0.02
FORMAT = "texture-acoustic-endpoint-v1"
SAMPLED_FORMAT = "texture-acoustic-full-sampler-v1"
SEPARATED_FORMAT = "texture-acoustic-level-shape-v1"


def candidate_arm(model_format):
    if model_format == FORMAT:
        return "acoustic"
    if model_format == SAMPLED_FORMAT:
        return "sampled"
    if model_format == SEPARATED_FORMAT:
        return "level_shape"
    raise ValueError("unknown acoustic experiment format")


def sampled_latent(model, physical, seed):
    """Differentiable existing64-midpoint sampler: conditions/noise only."""
    surface.validate_features(physical)
    frames = physical.shape[-1]
    if type(seed) is not int or not 0 <= seed < 2**32 or not 32 <= frames <= 256:
        raise ValueError("bounded latent length and seed required")
    device = next(model.parameters()).device
    controls = torch.tensor(physical[None], device=device)
    x = torch.randn(
        (1, 64, frames),
        generator=torch.Generator(device=device).manual_seed(seed),
        device=device,
    ).repeat(1, 1, 1)
    for step in range(64):
        t = torch.full((1,), step / 64, device=device)
        first = model(x, t, controls)
        second = model(x + first / 128, t + 1 / 128, controls)
        x = x + second / 64
    latent = x * model.scale + model.center
    if not torch.isfinite(latent).all():
        raise ValueError("nonfinite differentiable sample")
    return latent


def shared_band(wave):
    if wave.ndim != 3 or wave.shape[1] != 2 or wave.shape[-1] < 2048:
        raise ValueError("batched stereo44100 wave required")
    # Same default zero-extended FIR as scipy resample_poly(x,1,2), differentiable.
    kernel = torch.tensor(
        firwin(41, 0.5, window=("kaiser", 5.0)), dtype=wave.dtype, device=wave.device
    )[None, None]
    return F.conv1d(wave.mean(1, keepdim=True), kernel, stride=2, padding=20)[:, 0]


def acoustic_parts(candidate, reference, *, separate=False):
    if candidate.shape != reference.shape or candidate.shape[-1] != 32 * flow.HOP:
        raise ValueError("matching32-frame waveform windows required")
    if not torch.isfinite(candidate).all() or not torch.isfinite(reference).all():
        raise ValueError("nonfinite acoustic input")
    # Fixed8-frame decoder margins; auxiliary loss is NOT full-event timing loss.
    a, b = [shared_band(x)[:, 8 * 1024 : -8 * 1024] for x in (candidate, reference)]
    spectral = []
    for size in (512, 2048):
        window = torch.hann_window(size, device=a.device, dtype=a.dtype)
        spectra = []
        for x in (a, b):
            frames = x.unfold(-1, size, size // 2)
            frames = frames - frames.mean(-1, keepdim=True)
            power = torch.fft.rfft(frames * window).abs().square().mean(1)
            log_power = power[:, 1:].clamp_min(1e-12).log()
            if separate:
                # Training objective only: remove uniform log-spectral offset.
                # Gain invariance applies above the unchanged numerical floor.
                log_power = log_power - log_power.mean(-1, keepdim=True)
            spectra.append(log_power)
        spectral.append((spectra[0] - spectra[1]).abs().mean())
    energies = [
        x.unfold(-1, 441, 441).square().mean(-1).clamp_min(1e-12).log() for x in (a, b)
    ]
    return {
        "spectrum": torch.stack(spectral).mean(),
        "envelope": (energies[0] - energies[1]).abs().mean(),
    }


def endpoint(mixed, velocity, t):
    return mixed + (1 - t[:, None, None]) * velocity


def training_records(records):
    selected = [r for r in records if r["row"]["role"] == "train"]
    expected = {
        r["id"]
        for r in surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        if surface.role(r) == "train"
    }
    if len(selected) != 48 or {r["row"]["id"] for r in selected} != expected:
        raise ValueError("exact48 existing TRAIN records required")
    return selected


def fit(
    model, vae, records, acoustic, output, report, *, sampled=False, separate=False
):
    if separate and not sampled:
        raise ValueError("level/shape separation requires the full-sampler arm")
    training = training_records(records)
    device = next(model.parameters()).device
    torch.manual_seed(23)
    model.train()
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-4, weight_decay=1e-4)
    history = []
    started = time.monotonic()
    for step in range(STEPS):
        targets, physical, starts = [], [], []
        for record in training:
            last = record["valid_frames"] - flow.FRAMES
            first = (
                0
                if step % 3 == 0
                else last
                if step % 3 == 2
                else int(torch.randint(last + 1, (1,)))
            )
            mean = record["mean"][:, first : first + flow.FRAMES]
            std = record["std"][:, first : first + flow.FRAMES]
            targets.append(mean + std * torch.randn_like(std))
            physical.append(record["physical"][:, first : first + flow.FRAMES])
            starts.append(first)
        target = (torch.stack(targets).to(device) - model.center) / model.scale
        controls = torch.tensor(np.stack(physical), device=device)
        noise = torch.randn_like(target)
        t = torch.rand(len(target), device=device)
        mixed = noise * (1 - t[:, None, None]) + target * t[:, None, None]
        prediction = model(mixed, t, controls)
        fm_loss = (prediction - (target - noise)).square().mean()
        loss = fm_loss
        values = {"step": step + 1, "fm": float(fm_loss.detach())}
        if acoustic:
            index = step % len(training)
            start = starts[index] * flow.HOP
            if sampled:
                # Dedicated RNG does not change the paired FM minibatch sequence.
                latent = sampled_latent(model, training[index]["physical"], 607 + step)
                full = vae.decode(latent).sample
                decoded = full[:, :, start : start + 32 * flow.HOP]
            else:
                predicted = endpoint(
                    mixed[index : index + 1],
                    prediction[index : index + 1],
                    t[index : index + 1],
                )
                latent = predicted * model.scale + model.center
                decoded = vae.decode(latent).sample
            truth = torch.tensor(
                training[index]["wave"][start : start + 32 * flow.HOP].T[None],
                device=device,
            )
            parts = acoustic_parts(decoded, truth, separate=separate)
            auxiliary = parts["spectrum"] + parts["envelope"]
            if step == 0:
                destination = model.output.weight if sampled else prediction
                grad = torch.autograd.grad(auxiliary, destination, retain_graph=True)[0]
                norm = float(grad.norm())
                if not np.isfinite(norm) or norm <= 0:
                    raise ValueError("missing finite acoustic gradient to flow")
                name = "output_weight" if sampled else "velocity"
                report[f"acoustic_gradient_to_{name}_norm"] = norm
            loss = loss + WEIGHT * auxiliary
            values.update({k: float(v.detach()) for k, v in parts.items()})
            values.update(
                acoustic_training_id=training[index]["row"]["id"], t=float(t[index])
            )
            if sampled:
                values.update(sampler_seed=607 + step, sampled_frames=latent.shape[-1])
        if not torch.isfinite(loss):
            raise ValueError("nonfinite training loss")
        optimizer.zero_grad()
        loss.backward()
        norm = torch.nn.utils.clip_grad_norm_(
            model.parameters(), 1, error_if_nonfinite=True
        )
        if any(p.grad is not None or p.requires_grad for p in vae.parameters()):
            raise ValueError("frozen codec changed")
        optimizer.step()
        history.append({**values, "gradient_norm": float(norm)})
        if (step + 1) % 20 == 0:
            name = ("sampled" if sampled else "acoustic") if acoustic else "fm_only"
            if acoustic and separate:
                name = "level_shape"
            report["training_progress"] = {
                "arm": name,
                "step": step + 1,
                "elapsed_seconds": time.monotonic() - started,
            }
            flow.save(output / "result.json", report)
            print(json.dumps({**report["training_progress"], **values}), flush=True)
    model.eval()
    return history


def load_candidates(directory, parent):
    path = directory / "model.json"
    if path.stat().st_size > 200000:
        raise ValueError("oversized model metadata")
    meta = json.loads(path.read_text())
    candidate = candidate_arm(meta["format"])
    expected = {
        r["id"]
        for r in surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        if surface.role(r) == "train"
    }
    if (
        meta["parent"] != parent
        or meta["steps_per_arm"] != STEPS
        or meta["acoustic_weight"] != WEIGHT
        or len(meta["training_ids"]) != 48
        or set(meta["training_ids"]) != expected
        or parent["codec_sha256"] != flow.CODEC_SHA
        or parent["input_gain"] != flow.GAIN
        or parent["table_sha256"] != surface.TABLE_SHA
        or parent["descriptor"]["sha256"]
        != "6d36e47c055512eef37a9b182c19b31f2db246c0c03d33a00c21822a4672a8e0"
    ):
        raise ValueError("incompatible acoustic experiment")
    models = {}
    for arm in ("fm_only", candidate):
        path = directory / f"{arm}.safetensors"
        if (
            path.stat().st_size > 4_000_000
            or flow.codec.sha(path) != meta[arm]["sha256"]
        ):
            raise ValueError("weights size/hash mismatch")
        state = load_file(path)
        if (
            not all(torch.isfinite(v).all() for v in state.values())
            or (state["scale"] <= 0).any()
        ):
            raise ValueError("invalid weights")
        model = flow.TextureFlow(7).to("cuda").eval()
        model.load_state_dict(state, strict=True)
        models[arm] = model
    return models, meta


def render(directory, output):
    """Only saved weights/metadata and cached decoder; no dataset or source WAV."""
    torch.set_num_threads(4)
    path = directory / "model.json"
    if path.stat().st_size > 200000:
        raise ValueError("oversized model metadata")
    parent = json.loads(path.read_text())["parent"]
    models, meta = load_candidates(directory, parent)
    vae, _ = flow.load_codec("cuda")
    output = flow.fresh(output)
    report = {
        "status": "running",
        "model": meta,
        "new_training_steps": 0,
        "requested": {},
    }
    flow.save(output / "result.json", report)
    try:
        requested(models, vae, output, report)
        report["status"] = "complete"
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


def requested(models, vae, output, report):
    physical, request = event.profile()
    physical = surface.features(
        "Glass",
        0.3970170073501798,
        0.3827100881663895,
        physical[3] * 20 + 40,
        physical[4] * 0.25 + 0.75,
    )
    previews = []
    for name, model in models.items():
        entry, pcm = event.publish(
            output,
            f"requested-{name}",
            surface.generate(model, vae, physical, 314),
            round(request["duration_seconds"] * 44100),
        )
        report["requested"][name] = {
            **entry,
            "seed": 314,
            "reference_audio_input": False,
            "sensor_input": False,
            "physical_features": physical[:, 0].tolist(),
            "request": {k: v for k, v in request.items() if k != "texture"},
        }
        previews.extend((pcm, np.zeros((11025, 2))))
    report["requested_comparison"], _ = flow.codec.publish(
        output / "requested-comparison.wav", np.concatenate(previews)
    )
    report["requested_order"] = list(models)
    flow.save(output / "result.json", report)


@torch.inference_mode()
def evaluate(models, vae, records, output, report):
    preview = []
    for record in records:
        row = record["row"]
        if row["commanded_speed_mm_s"] not in (20, 40, 60):
            continue
        if not (
            (row["texture_id"] in surface.HELD and row["repeat"] == 0)
            or (row["texture_id"] in surface.source.TEXTURES and row["repeat"] == 1)
        ):
            continue
        entry, real = flow.codec.publish(
            output / f"{row['id']}-real.wav", record["wave"] * event.PLAYBACK
        )
        report["references"].append({"id": row["id"], **entry})
        for seed in (314, 2718):
            pcms = []
            for name, model in models.items():
                entry, pcm = event.publish(
                    output,
                    f"{row['id']}-{name}-{seed}",
                    surface.generate(model, vae, record["physical"], seed),
                    len(real),
                )
                report["rows"].append(
                    {
                        "id": row["id"],
                        "texture_id": row["texture_id"],
                        "speed_mm_s": row["commanded_speed_mm_s"],
                        "role": row["role"],
                        "seed": seed,
                        "variant": name,
                        **entry,
                        **surface.metrics(pcm, real, record),
                    }
                )
                pcms.append(pcm)
            if (
                row["texture_id"] == 76
                and row["commanded_speed_mm_s"] == 40
                and row["commanded_normal_force_N"] == 0.5
                and seed == 314
            ):
                for pcm in (real, *pcms):
                    preview.extend((pcm, np.zeros((11025, 2))))
        print(json.dumps({"evaluated": row["id"]}), flush=True)
        flow.save(output / "result.json", report)
    if len(report["rows"]) != 216:
        raise ValueError("exact36-record/two-seed/three-arm evaluation required")
    report["comparison"], _ = flow.codec.publish(
        output / "comparison.wav", np.concatenate(preview)
    )
    report["comparison_order"] = "frosted glass40mm/s,.5N,seed314: real/" + "/".join(
        models
    )


def run(root, output, existing=None, *, sampled=False, separate=False):
    sampled = sampled or separate
    torch.set_num_threads(4)
    root = root.resolve()
    source = root / "cluster-surface-transfer-grid-2026-09-05" / "result.json"
    parents, parent_meta = surface.load_models(
        root / "texture-surface-transfer-2026-09-05", "cuda"
    )
    if parent_meta["source_sha256"] != flow.codec.sha(source):
        raise ValueError("source/model lineage mismatch")
    vae, codec_root = flow.load_codec("cuda")
    output = flow.fresh(output)
    report = {
        "status": "running",
        "scope": "paired TRAIN-only acoustic correction; not quality admission",
        "source_sha256": flow.codec.sha(source),
        "requested": {},
        "rows": [],
        "references": [],
    }
    flow.save(output / "result.json", report)
    try:
        if sampled:
            profile, _ = event.profile()
            physical = surface.features(
                "Glass", 0.4, 0.38, profile[3] * 20 + 40, profile[4] * 0.25 + 0.75
            )
            actual = sampled_latent(parents["descriptor"], physical, 607)
            expected = flow.sample_features(
                parents["descriptor"], physical[None], 607, physical.shape[-1]
            )
            report["sampler_forward_max_absolute_delta"] = float(
                (actual.detach() - expected).abs().max()
            )
            torch.testing.assert_close(actual.detach(), expected, rtol=1e-5, atol=1e-6)
            del actual, expected
            flow.save(output / "result.json", report)
        records, _, _ = event.prepare(
            source,
            vae,
            "cuda",
            loader=surface.load_data,
            feature_builder=surface.builder(surface.metadata(source)),
        )
        models = {"base": parents["descriptor"]}
        if existing:
            candidates, meta = load_candidates(existing, parent_meta)
            models.update(candidates)
        else:
            meta = {
                "format": SEPARATED_FORMAT
                if separate
                else SAMPLED_FORMAT
                if sampled
                else FORMAT,
                "parent": parent_meta,
                "steps_per_arm": STEPS,
                "acoustic_weight": WEIGHT,
                "training_seed": 23,
                "learning_rate": 1e-4,
                "training_ids": [r["row"]["id"] for r in training_records(records)],
            }
            for arm in ("fm_only", candidate_arm(meta["format"])):
                model = copy.deepcopy(models["base"])
                history = fit(
                    model,
                    vae,
                    records,
                    arm != "fm_only",
                    output,
                    report,
                    sampled=sampled,
                    separate=separate,
                )
                path = output / f"{arm}.safetensors"
                save_file(model.state_dict(), path)
                meta[arm] = {"sha256": flow.codec.sha(path), "history": history}
                models[arm] = model
                flow.save(output / "model.json", meta)
            for module in (__file__, surface.__file__, event.__file__, flow.__file__):
                shutil.copyfile(module, output / Path(module).name)
            for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
                shutil.copyfile(codec_root / name, output / name)
        report.update(model=meta, new_training_steps=0 if existing else 2 * STEPS)
        requested(models, vae, output, report)
        evaluate(models, vae, records, output, report)
        report["status"] = "complete"
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lab-root", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--evaluate-model", type=Path)
    parser.add_argument("--render-model", type=Path)
    parser.add_argument("--sampled-loss", action="store_true")
    parser.add_argument("--separate-level-shape", action="store_true")
    args = parser.parse_args()
    if args.render_model:
        if (
            args.lab_root
            or args.evaluate_model
            or args.sampled_loss
            or args.separate_level_shape
        ):
            parser.error("standalone rendering takes no dataset/evaluation inputs")
        render(args.render_model, args.output)
    elif args.lab_root:
        if (args.sampled_loss or args.separate_level_shape) and args.evaluate_model:
            parser.error(
                "evaluation uses the saved objective, not an objective override"
            )
        run(
            args.lab_root,
            args.output,
            args.evaluate_model,
            sampled=args.sampled_loss,
            separate=args.separate_level_shape,
        )
    else:
        parser.error("--lab-root or --render-model required")
