"""Numerical pouring controls over a frozen, pretrained TangoFlux generator.

Powered by Stability AI; TangoFlux / Hung et al. Local research only. Source
redistribution remains unspecified. No recording is an inference input.
"""

import argparse
import hashlib
import json
from contextlib import contextmanager
from pathlib import Path

import numpy as np
import physical_sound_pouring_latent_flow as flow
import physical_sound_tangoflux_train as train
import torch
from safetensors.torch import load_file, save_file
from torch import nn

PROMPT = "The sound of water being poured into a container."
SECONDS = 4.08
FORMAT = "pour-tango-numeric-bridge-v1"


class Bridge(nn.Module):
    def __init__(self):
        super().__init__()
        self.network = nn.Sequential(
            nn.Linear(11, 128), nn.SiLU(), nn.Linear(128, 2048)
        )
        nn.init.zeros_(self.network[-1].weight)
        nn.init.zeros_(self.network[-1].bias)
        self.register_buffer("offset", None, persistent=False)

    def condition(self, controls, hidden, pooled, cfg=False):
        if controls.shape != (1, 11) or not torch.isfinite(controls).all():
            raise ValueError("one finite eleven-control vector required")
        batch = 2 if cfg else 1
        if (
            hidden.ndim != 3
            or hidden.shape[0] != batch
            or hidden.shape[2] != 1024
            or pooled.shape != (batch, 1024)
        ):
            raise ValueError("unsupported pretrained conditioning layout")
        delta = self.network(controls)
        if self.offset is not None:
            delta = delta - self.offset
        positive = torch.cat(
            [hidden[-1:, :-1] + delta[:, None, :1024], hidden[-1:, -1:]], 1
        )
        positive_pool = pooled[-1:] + delta[:, 1024:]
        if cfg:
            return torch.cat([hidden[:1], positive]), torch.cat(
                [pooled[:1], positive_pool]
            )
        return positive, positive_pool

    @contextmanager
    def hook(self, transformer, controls):
        def inject(module, args, kwargs):
            kwargs = dict(kwargs)
            hidden, pooled = self.condition(
                controls,
                kwargs["encoder_hidden_states"],
                kwargs["pooled_projections"],
                cfg=True,
            )
            kwargs.update(encoder_hidden_states=hidden, pooled_projections=pooled)
            return args, kwargs

        handle = transformer.register_forward_pre_hook(inject, with_kwargs=True)
        try:
            yield
        finally:
            handle.remove()


def digest(module):
    result = hashlib.sha256()
    for name, value in module.state_dict().items():
        result.update(name.encode())
        result.update(value.detach().cpu().contiguous().numpy().tobytes())
    return result.hexdigest()


@torch.no_grad()
def cache_targets(source, cache_root, vae, output):
    rows, provenance = flow.c.v.p.load_source(source)
    lookup = {r["item_id"]: r for r in rows if r["role"] == "train"}
    meta = json.loads((cache_root / "cache.json").read_text())
    expected_rows = []
    for row in lookup.values():
        last = max(0, row["spectrogram"].shape[1] - flow.c.v.p.FRAMES)
        for phase, start in (("first", 0), ("middle", last // 2), ("last", last)):
            expected_rows.append(
                {
                    "item_id": row["item_id"],
                    "container_id": row["container_id"],
                    "phase": phase,
                    "start_frame16k": start,
                }
            )
    if (
        meta["format"] != "pour-oobleck-cache-v1"
        or meta["rows"] != expected_rows
        or meta["source_sha256"]
        != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
        or meta["codec_sha256"] != flow.c.CODEC_SHA
        or set(meta["train_ids"]) != set(lookup)
        or len(meta["rows"]) != 279
        or any(r["item_id"] not in lookup for r in meta["rows"])
    ):
        raise ValueError("training metadata/source mismatch")
    means, stds, controls = [], [], []
    vae.to("cuda")
    for i, record in enumerate(meta["rows"]):
        row = lookup[record["item_id"]]
        start = record["start_frame16k"]
        _, control = flow.c.v.p.crop(row, start)
        wave = row["wave"][start * 256 : start * 256 + flow.c.v.p.SAMPLES]
        wave = np.pad(wave, (0, flow.c.v.p.SAMPLES - len(wave)))
        native = flow.c.input_wave(wave)[:, : round(SECONDS * train.tango.RATE)]
        native = np.pad(native, ((0, 0), (0, 30 * train.tango.RATE - native.shape[1])))
        posterior = vae.encode(torch.tensor(native[None], device="cuda")).latent_dist
        mean, std = (
            posterior.mean.transpose(1, 2).cpu(),
            posterior.std.transpose(1, 2).cpu(),
        )
        if (
            mean.shape != (1, 645, 64)
            or not torch.isfinite(mean).all()
            or not torch.isfinite(std).all()
        ):
            raise ValueError("invalid native pretrained posterior")
        means.append(mean)
        stds.append(std)
        controls.append(control)
        if (i + 1) % 30 == 0:
            print({"encoded": i + 1}, flush=True)
    data = {
        "mean": torch.cat(means),
        "std": torch.cat(stds),
        "controls": torch.tensor(np.stack(controls)),
    }
    save_file(data, output / "posterior.safetensors")
    vae.to("cpu")
    torch.cuda.empty_cache()
    return data, {
        "source_sha256": meta["source_sha256"],
        "train_ids": meta["train_ids"],
        "crop_metadata_sha256": hashlib.sha256(
            (cache_root / "cache.json").read_bytes()
        ).hexdigest(),
        "posterior_sha256": hashlib.sha256(
            (output / "posterior.safetensors").read_bytes()
        ).hexdigest(),
        "terms": provenance["terms"],
        "rows": meta["rows"],
    }


@torch.no_grad()
def generate(
    model, vae, output, name, prompt=PROMPT, controls=None, bridge=None, seed=2718
):
    if not isinstance(seed, int) or not 0 <= seed < 2**32:
        raise ValueError("invalid seed")
    # Match training-time baseline execution, including parameter flags.
    # no_grad alone did not give byte-exact reload in the paired flag control.
    model.requires_grad_(False)
    vae.requires_grad_(False)
    model.eval().to("cuda")
    torch.manual_seed(seed)
    if bridge is None:
        latent = model.inference_flow(
            prompt,
            duration=SECONDS,
            num_inference_steps=50,
            guidance_scale=4.5,
            disable_progress=True,
        )
    else:
        flow.c.v.h.grid(controls, np.array([0]))
        vector = torch.tensor(
            np.asarray(controls, dtype=np.float32)[None], device="cuda"
        )
        with bridge.hook(model.transformer, vector):
            latent = model.inference_flow(
                prompt,
                duration=SECONDS,
                num_inference_steps=50,
                guidance_scale=4.5,
                disable_progress=True,
            )
    model.to("cpu")
    torch.cuda.empty_cache()
    vae.to("cuda")
    native = (
        vae.decode(latent.transpose(1, 2))
        .sample[0, :, : round(SECONDS * train.tango.RATE)]
        .cpu()
        .numpy()
    )
    vae.to("cpu")
    torch.cuda.empty_cache()
    row, mono = train.tango.publish(output, name, native)
    print({"generated": name}, flush=True)
    return {**row, "seed": seed, "reference_audio_input": False}, mono


def load_bridge(directory):
    path = directory / "bridge.safetensors"
    metadata = directory / "result.json"
    if path.stat().st_size > 4_000_000 or metadata.stat().st_size > 2_000_000:
        raise ValueError("oversized bridge")
    meta = json.loads(metadata.read_text())
    if (meta["format"], meta["status"], meta["revision"]) != (
        FORMAT,
        "complete",
        train.tango.REVISION,
    ) or hashlib.sha256(path.read_bytes()).hexdigest() != meta["bridge_sha256"]:
        raise ValueError("bridge identity mismatch")
    state = load_file(path)
    if not all(
        x.dtype == torch.float32 and torch.isfinite(x).all() for x in state.values()
    ):
        raise ValueError("invalid bridge weights")
    bridge = Bridge()
    bridge.load_state_dict(state, strict=True)
    return bridge.eval().to("cuda"), meta


def write_comparison(output, rows):
    from scipy.io import wavfile

    selected = [r for r in rows if r["seed"] == 2718]
    if not selected:
        raise ValueError("comparison requires the disclosed preview seed")
    # Concatenate the actual audition PCM, not float intermediates rounded just
    # above 0.98. Preserve the publisher's recorded gains; do not relax a guard.
    preview = []
    for row in selected:
        path = Path(row["wav"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]:
            raise ValueError("comparison input hash mismatch")
        rate, pcm = wavfile.read(path)
        if (
            rate != 16000
            or pcm.dtype != np.int16
            or pcm.ndim != 1
            or not 0 < len(pcm) <= 160000
            or np.abs(pcm.astype(np.int32)).max() > round(0.98 * 32767)
        ):
            raise ValueError("invalid comparison PCM")
        preview.extend([pcm, np.zeros(8000, dtype=np.int16)])
    pcm = np.concatenate(preview)
    path = output / "comparison.wav"
    wavfile.write(path, 16000, pcm)
    return {
        **train.pilot.signal_stats(pcm.astype(np.float32) / 32768),
        "pcm_gain": 1,
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def load_offset(directory, bridge_meta):
    """Frozen TRAIN mean, not fitted from an inference reference or dev score."""
    metadata, weights = directory / "result.json", directory / "offset.safetensors"
    if metadata.stat().st_size > 2_000_000 or weights.stat().st_size > 100_000:
        raise ValueError("oversized centering artifact")
    meta = json.loads(metadata.read_text())
    if (
        meta["status"] != "complete"
        or meta["bridge_sha256"] != bridge_meta["bridge_sha256"]
        or meta["frozen_model_sha256"] != bridge_meta["frozen_model_sha256_after"]
        or meta["training_posterior_sha256"]
        != bridge_meta["source"]["posterior_sha256"]
        or hashlib.sha256(weights.read_bytes()).hexdigest() != meta["offset_sha256"]
    ):
        raise ValueError("centering identity mismatch")
    state = load_file(weights)
    if set(state) != {"offset"}:
        raise ValueError("invalid centering keys")
    offset = state["offset"]
    if (
        offset.shape != (1, 2048)
        or offset.dtype != torch.float32
        or not torch.isfinite(offset).all()
    ):
        raise ValueError("invalid centering tensor")
    return offset, meta["offset_sha256"]


def finish_export(report, model, vae, output):
    report["comparison"] = write_comparison(output, report["rows"])
    report["regression_after"] = {}
    regressions = [
        ("glass", train.PROMPT),
        ("wood", train.pilot.CASES[2][1]),
        ("rain", "The sound of rain falling."),
    ]
    for key, prompt in regressions:
        row, _ = generate(model, vae, output, f"after-{key}", prompt=prompt)
        original = report["regression_before"][key]
        if (
            row["sha256"] != original["sha256"]
            or row["native_sha256"] != original["native_sha256"]
        ):
            raise ValueError("adapter-off regression changed")
        report["regression_after"][key] = row
    report["checks"]["regression_exact"] = True
    report["status"] = "complete"
    flow.c.v.p.save_json(output / "result.json", report)
    flow.c.v.p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": SECONDS,
            "cases": report["cases"],
            "rows": report["rows"],
            "controls": [],
        },
    )


def run(source, cache_root, output, steps=200):
    if not isinstance(steps, int) or not 1 <= steps <= 400:
        raise ValueError("one to 400 training updates required")
    output = flow.c.v.phase.d.fresh_output(output)
    torch.set_num_threads(4)
    torch.manual_seed(53)
    model, vae = train.tango.load_models(output)
    model.requires_grad_(False)
    vae.requires_grad_(False)
    before = digest(model)
    data, provenance = cache_targets(source, cache_root, vae, output)
    bridge = Bridge().to("cuda")
    report = {
        "format": FORMAT,
        "status": "running",
        "revision": train.tango.REVISION,
        "source": provenance,
        "training": [],
        "checks": {},
        "rows": [],
        "scope": "frozen pretrained generator, numerical controls; local research, no physical calibration claim",
        "parameters": sum(p.numel() for p in bridge.parameters()),
        "seed": 53,
        "steps_requested": steps,
        "frozen_model_sha256_before": before,
    }
    save = lambda: flow.c.v.p.save_json(output / "result.json", report)
    save()
    try:
        c0 = flow.c.v.phase.d.controls_for()
        baseline, _ = generate(model, vae, output, "initial-base")
        zero, _ = generate(
            model, vae, output, "initial-zero", controls=c0, bridge=bridge
        )
        if (
            baseline["sha256"] != zero["sha256"]
            or baseline["native_sha256"] != zero["native_sha256"]
        ):
            raise ValueError("zero adapter changed the pretrained generator")
        report["checks"]["zero_exact"] = True
        report["initial"] = [baseline, zero]
        save()
        regressions = [
            ("glass", train.PROMPT),
            ("wood", train.pilot.CASES[2][1]),
            ("rain", "The sound of rain falling."),
        ]
        originals = {}
        for key, prompt in regressions:
            originals[key], _ = generate(
                model, vae, output, f"before-{key}", prompt=prompt
            )
        report["regression_before"] = originals
        save()
        model.to("cuda")
        condition = train.conditioning(model, PROMPT, SECONDS)
        from diffusers.training_utils import compute_density_for_timestep_sampling

        with torch.no_grad(), torch.random.fork_rng(devices=[0]):
            target = data["mean"][:1].to("cuda")
            torch.manual_seed(765)
            upstream = model(
                target, [PROMPT], duration=torch.tensor([SECONDS], device="cuda")
            )[0]
            torch.manual_seed(765)
            noise = torch.randn_like(target)
            u = compute_density_for_timestep_sampling("logit_normal", 1, 0, 1, None)
            sigma = model.noise_scheduler_copy.sigmas[(u * 1000).long()].to("cuda")
            prediction = train.velocity(
                model, (1 - sigma) * target + sigma * noise, sigma, condition
            )
            cached = train.loss_parts(prediction, noise - target, active_frames=88)[
                "full"
            ]
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
        for step in range(steps):
            i = int(torch.randint(len(data["mean"]), (1,), generator=rng))
            mean, std = data["mean"][i : i + 1], data["std"][i : i + 1]
            target = (mean + std * torch.randn(mean.shape, generator=rng)).to("cuda")
            noise = torch.randn(target.shape, generator=rng).to("cuda")
            index = int((torch.randn(1, generator=rng).sigmoid() * 1000).long())
            sigma = model.noise_scheduler_copy.sigmas[index].to("cuda")
            optimizer.zero_grad(set_to_none=True)
            with torch.autocast("cuda", dtype=torch.bfloat16):
                adapted = bridge.condition(
                    data["controls"][i : i + 1].to("cuda"), *condition
                )
                prediction = train.velocity(
                    model, (1 - sigma) * target + sigma * noise, sigma, adapted
                )
                parts = train.loss_parts(prediction, noise - target, active_frames=88)
                loss = parts["full"]
            if not torch.isfinite(loss):
                raise ValueError("nonfinite bridge loss")
            loss.backward()
            norm = torch.nn.utils.clip_grad_norm_(
                bridge.parameters(), 1.0, error_if_nonfinite=True
            )
            if any(p.grad is not None for p in model.parameters()):
                raise ValueError("frozen generator received gradients")
            optimizer.step()
            report["training"].append(
                {
                    "step": step + 1,
                    "cache_row": i,
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
        after = digest(model)
        if after != before:
            raise ValueError("pretrained weights changed")
        report["checks"]["frozen_model_exact"] = True
        report["frozen_model_sha256_after"] = after
        weights = output / "bridge.safetensors"
        save_file(
            {k: v.detach().cpu().contiguous() for k, v in bridge.state_dict().items()},
            weights,
        )
        report["bridge_sha256"] = hashlib.sha256(weights.read_bytes()).hexdigest()
        save()
        bridge.eval()
        cases = []
        for seed in (314, 2718):
            base, _ = generate(model, vae, output, f"base-{seed}", seed=seed)
            for profile, controls in profiles():
                row, _ = generate(
                    model,
                    vae,
                    output,
                    f"{profile}-{seed}",
                    controls=controls,
                    bridge=bridge,
                    seed=seed,
                )
                case = len(cases)
                cases.append({"id": f"{profile}-{seed}", "diagnostic_id": "water-pour"})
                report["rows"].extend(
                    [
                        {**base, "case": case, "kind": "base", "profile": profile},
                        {
                            **row,
                            "case": case,
                            "kind": "bridge",
                            "profile": profile,
                            "controls": controls.tolist(),
                        },
                    ]
                )
                save()
        report["cases"] = cases
        finish_export(report, model, vae, output)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


def profiles():
    base = flow.c.v.phase.d.controls_for()
    tall = base.copy()
    tall[0] = 0.8
    pet = base.copy()
    pet[5:9] = [0, 0, 1, 0]
    fast = base.copy()
    fast[3] = 0.3
    return [("glass10", base), ("glass16", tall), ("pet10", pet), ("glass10fast", fast)]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    run_parser = sub.add_parser("run")
    for name in ("source", "cache", "output"):
        run_parser.add_argument("--" + name, type=Path, required=True)
    run_parser.add_argument("--steps", type=int, default=200)
    render_parser = sub.add_parser("render")
    render_parser.add_argument("--model", type=Path, required=True)
    render_parser.add_argument("--output", type=Path, required=True)
    render_parser.add_argument("--controls", type=float, nargs=11, required=True)
    render_parser.add_argument("--seed", type=int, default=2718)
    render_parser.add_argument("--offset", type=Path)
    args = parser.parse_args()
    if args.mode == "run":
        run(args.source, args.cache, args.output, args.steps)
    else:
        torch.set_num_threads(4)
        bridge, meta = load_bridge(args.model)
        offset_sha = None
        if args.offset:
            offset, offset_sha = load_offset(args.offset, meta)
            bridge.offset = offset.to("cuda")
        out = flow.c.v.phase.d.fresh_output(args.output)
        model, vae = train.tango.load_models(out)
        if digest(model) != meta["frozen_model_sha256_after"]:
            raise ValueError("pretrained generator differs from the fitted bridge")
        row, _ = generate(
            model,
            vae,
            out,
            "generated",
            controls=np.array(args.controls, dtype=np.float32),
            bridge=bridge,
            seed=args.seed,
        )
        flow.c.v.p.save_json(
            out / "result.json",
            {
                "reference_audio_input": False,
                "bridge_sha256": meta["bridge_sha256"],
                "offset_sha256": offset_sha,
                "controls": args.controls,
                "rows": [row],
            },
        )
