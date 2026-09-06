"""Shared contact residual trained through full 50-step generation and VAE.

No reference latent/recording enters generation. TRAIN waveforms supervise a
decoded-audio loss only. Existing source-free render/assessment CLIs load the
resulting adapter; no runtime promotion or per-object coefficients.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_codec_probe as codec
import physical_sound_sonicgauss_cohort as cohort
import physical_sound_sonicgauss_pilot as pilot
import physical_sound_sonicgauss_shared_fit as shared
import torch
from safetensors.torch import save_file
from torch.nn import functional as F
from torch.utils.checkpoint import checkpoint


def audible(wave):
    """Loss-only smooth 20 Hz spectral high-pass; raw output is unchanged.

    Odd reflection avoids joining endpoints or introducing slope cusps. This FFT operator is
    deliberately not claimed identical to the independent SciPy audit filter.
    """
    if wave.ndim != 2 or wave.shape[-1] < 8192:
        raise ValueError("expected long batch of mono signals")
    left = 2 * wave[:, :1] - wave[:, 1:4097].flip(-1)
    right = 2 * wave[:, -1:] - wave[:, -4097:-1].flip(-1)
    padded = torch.cat([left, wave, right], dim=-1)
    freq = torch.fft.rfftfreq(padded.shape[-1], 1 / 44100, device=wave.device)
    transfer = 1 / (1 + (20 / freq.clamp_min(1e-4)) ** 8)
    return torch.fft.irfft(torch.fft.rfft(padded) * transfer, n=padded.shape[-1])[
        :, 4096:-4096
    ]


def waveform_loss(reference, generated):
    if (
        reference.ndim != 3
        or generated.ndim != 3
        or reference.shape[:2] != generated.shape[:2]
    ):
        raise ValueError("expected compatible batch/stereo waveforms")
    length = max(reference.shape[-1], generated.shape[-1])
    signals = [
        F.pad(w.mean(1), (0, length - w.shape[-1])) for w in (reference, generated)
    ]
    target, prediction = [audible(w) for w in signals]
    distances = []
    for size in (512, 1024, 2048):
        window = torch.hann_window(size, device=reference.device)
        mags = [
            torch.stft(w, size, size // 4, window=window, return_complex=True).abs()
            for w in (target, prediction)
        ]
        distances.append(
            (mags[0] - mags[1]).abs().sum((1, 2)) / mags[0].sum((1, 2)).clamp_min(1e-8)
        )
    spectrum = torch.stack(distances).mean()
    # 2 ms amplitude envelopes expose misplaced/extra attacks, not just spectra.
    envelopes = [
        F.avg_pool1d(w[:, None].square(), 88, 88, ceil_mode=True).add(1e-16).sqrt()
        for w in (target, prediction)
    ]
    envelope = (envelopes[0] - envelopes[1]).abs().sum() / envelopes[0].sum().clamp_min(
        1e-8
    )
    rms = [w.square().mean(1).add(1e-16).sqrt() for w in (target, prediction)]
    level = (rms[1].clamp_min(1e-8).log() - rms[0].clamp_min(1e-8).log()).abs().mean()
    return spectrum + 0.25 * envelope + 0.1 * level, {
        "spectrum": spectrum,
        "envelope": envelope,
        "level": level,
    }


def generate(model, fused, initial_noise, use_checkpoint=False):
    """Same no-CFG 50-step Euler path, with gradients and immutable step inputs."""
    from diffusers import FlowMatchEulerDiscreteScheduler

    device = fused.device
    duration = model.duration_emebdder(torch.tensor([3.0], device=device))
    pooled = model.fc(torch.nanmean(fused, dim=1))
    states = torch.cat([fused, duration], dim=1)
    txt_ids = torch.zeros(1, states.shape[1], 3, device=device)
    audio_ids = torch.arange(model.audio_seq_len, device=device)[None, :, None].repeat(
        1, 1, 3
    )
    scheduler = FlowMatchEulerDiscreteScheduler.from_config(
        model.noise_scheduler.config
    )
    scheduler.set_timesteps(sigmas=np.linspace(1.0, 1 / 50, 50), device=device)

    def predict(latent, time, context, projection):
        return model.transformer(
            hidden_states=latent,
            timestep=time,
            guidance=None,
            pooled_projections=projection,
            encoder_hidden_states=context,
            txt_ids=txt_ids,
            img_ids=audio_ids,
            return_dict=False,
        )[0]

    latent = initial_noise.clone()
    for t in scheduler.timesteps:
        time = torch.tensor([t / 1000], device=device)
        if use_checkpoint:
            velocity = checkpoint(
                predict, latent, time, states, pooled, use_reentrant=False
            )
        else:
            velocity = predict(latent, time, states, pooled)
        # Scheduler mutation is outside checkpointed functions: backward never
        # replays a later step index or captures the final loop timestep.
        latent = scheduler.step(velocity, t, latent).prev_sample
    return latent


def audio_metrics(reference, generated):
    # Evaluation deliberately uses the prior independent SciPy IIR filter,
    # not the differentiable FFT operator optimized during training.
    target, predicted = [
        codec.audible_component(np.stack([w, w])).mean(0)
        for w in (reference, generated)
    ]
    length = max(len(target), len(predicted))
    signals = [np.pad(w, (0, length - len(w))) for w in (target, predicted)]
    padded = ((length + 87) // 88) * 88
    envelopes = [
        np.sqrt(
            np.mean(
                np.pad(w, (0, padded - length)).astype(float).reshape(-1, 88) ** 2,
                axis=1,
            )
            + 1e-16
        )
        for w in signals
    ]
    rms = [float(np.sqrt(np.mean(w.astype(float) ** 2) + 1e-16)) for w in signals]
    return {
        "spectrum": cohort.magnitude_distance(target, predicted),
        "envelope": float(
            np.abs(envelopes[0] - envelopes[1]).sum() / max(envelopes[0].sum(), 1e-8)
        ),
        "level": float(abs(np.log(max(rms[1], 1e-8) / max(rms[0], 1e-8)))),
    }


def evaluate(args):
    shared.assess(args)  # Keeps raw assessment, decision and all 60 audible triples.
    corpus = json.loads((args.data / "corpus.json").read_text())
    generated = json.loads((args.generated / "result.json").read_text())
    by_key = {
        (r["object_id"], r["contact_index"], r["variant"]): r for r in generated["rows"]
    }
    rows = []
    for obj in corpus["objects"]:
        for contact in obj["contacts"]:
            ref = contact["reference"]
            reference = cohort.audio(Path(ref["path"]), ref["sha256"])
            row = {
                "object_id": obj["object_id"],
                "contact_index": contact["index"],
                "local_fit_role": obj["local_fit_role"],
            }
            for kind in ("baseline", "candidate"):
                record = by_key[(obj["object_id"], contact["index"], kind)]
                wave = (
                    cohort.audio(Path(record["wav"]), record["wav_sha256"])
                    / generated["shared_gain"]
                )
                row[kind] = audio_metrics(reference, wave)
            rows.append(row)
    summary = {}
    for role in ("train", "development"):
        selected = [r for r in rows if r["local_fit_role"] == role]
        summary[role] = {
            metric: {
                kind: float(np.mean([r[kind][metric] for r in selected]))
                for kind in ("baseline", "candidate")
            }
            for metric in ("spectrum", "envelope", "level")
        }
        for metric in summary[role]:
            summary[role][metric]["wins"] = sum(
                r["candidate"][metric] < r["baseline"][metric] for r in selected
            )
    regressions = []
    for oid in sorted(set(cohort.OBJECTS) - cohort.FIT_TRAIN):
        selected = [r for r in rows if r["object_id"] == oid]
        for metric in ("spectrum", "envelope", "level"):
            if np.mean([r["candidate"][metric] for r in selected]) > np.mean(
                [r["baseline"][metric] for r in selected]
            ):
                regressions.append({"object_id": oid, "metric": metric})
    raw = json.loads((args.output / "assessment.json").read_text())["decision"][
        "decision"
    ]
    improved = (
        summary["development"]["spectrum"]["candidate"]
        < summary["development"]["spectrum"]["baseline"]
    )
    decision = (
        "REJECT"
        if regressions or not improved or raw == "REJECT"
        else "REPORT_ONLY_NONREGRESSION_PASS"
    )
    cohort.save(
        args.output / "audible-assessment.json",
        {
            "rows": rows,
            "summary": summary,
            "decision": decision,
            "regressions": regressions,
            "raw_decision": raw,
            "scope": "same fixed-noise local DEV, independent IIR audible spectrum/envelope/level checks; not physical-quality acceptance",
        },
    )
    print(
        json.dumps(
            {"summary": summary, "decision": decision, "regressions": regressions},
            indent=2,
        ),
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evaluate", action="store_true")
    parser.add_argument(
        "--energy-score",
        action="store_true",
        help="two independent samples and symmetric spectral energy objective",
    )
    for name in ("source", "assets", "data", "baseline", "generated", "output"):
        parser.add_argument("--" + name, type=Path, required=name in ("data", "output"))
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    if args.evaluate:
        if args.generated is None:
            parser.error("--evaluate needs --generated")
        evaluate(args)
        return
    if any(x is None for x in (args.source, args.assets, args.baseline)):
        parser.error("fit needs --source --assets --baseline")
    rows, geometry, hashes = shared.caches(args.data, args.baseline)
    corpus = json.loads((args.data / "corpus.json").read_text())
    shared.validate_inputs(corpus)
    torch.set_num_threads(4)
    torch.manual_seed(0)
    torch.backends.cudnn.deterministic = True
    ns = pilot.load_definitions(args.source)
    models = pilot.build_models(args.source, args.assets, ns)
    model, _, position, fusion, vae, _ = models
    args.output.mkdir(parents=True)
    features, positions, noises = {}, {}, {}
    for obj in rows:
        oid = obj["object_id"]
        cache = geometry[oid]
        features[oid] = cache["features"].cuda()
        with torch.no_grad():
            positions[oid] = [position(p.cuda())[None] for p in cache["contacts"]]
        torch.set_rng_state(cache["cpu_rng"])
        torch.cuda.set_rng_state(cache["cuda_rng"])
        noises[oid] = torch.randn(1, 64, 64, device="cuda")
    # Before training, verify the differentiable driver against the unchanged
    # published path. Both actual latent and full waveform must replay exactly.
    with (
        torch.no_grad(),
        np.load(args.baseline / "object-02-contact-0.npz", allow_pickle=False) as old,
    ):
        latent = generate(model, fusion(features[2], positions[2][0]), noises[2])
        wave = vae.decode(latent.transpose(1, 2)).sample[0]
        if not (
            np.array_equal(old["initial_noise"], noises[2].cpu().numpy())
            and np.array_equal(old["latent"], latent.cpu().numpy())
            and np.array_equal(old["wave"], wave.cpu().numpy())
        ):
            raise ValueError("differentiable driver differs from original; no fit")
    print("full 50-step latent/PCM equivalence PASS", flush=True)
    teachers = []
    for obj, thin in zip(corpus["objects"], rows, strict=True):
        oid = obj["object_id"]
        if [{k: c[k] for k in ("index", "contact")} for c in obj["contacts"]] != thin[
            "contacts"
        ]:
            raise ValueError("corpus/render contacts differ")
        if oid not in cohort.FIT_TRAIN:
            continue
        for contact in obj["contacts"]:
            ref = contact["reference"]
            audio = shared.teacher_audio(Path(ref["path"]), ref["sha256"])
            teachers.append(
                (
                    oid,
                    contact["index"],
                    torch.from_numpy(audio)[None].cuda(),
                    ref["sha256"],
                )
            )
    torch.manual_seed(42)
    adapter = shared.ContactResidual().cuda()
    adapted = shared.AdaptedFusion(fusion, adapter)
    versions = [(p, p._version) for module in models[:5] for p in module.parameters()]
    optimizer = torch.optim.Adam(adapter.parameters(), lr=1e-4)
    order = torch.randperm(
        len(teachers), generator=torch.Generator().manual_seed(42)
    ).tolist()
    losses = []
    noise_rng = torch.Generator(device="cuda").manual_seed(42)
    if args.energy_score:
        import physical_sound_sonicgauss_energy_probe as distribution

    for step in range(
        72
    ):  # two complete passes, one final checkpoint, no DEV selection
        oid, index, target, _ = teachers[order[step % len(order)]]
        optimizer.zero_grad(set_to_none=True)
        fused = adapted(features[oid], positions[oid][index])
        if args.energy_score:
            predictions = []
            for _ in range(2):
                noise = torch.randn(1, 64, 64, device="cuda", generator=noise_rng)
                latent = generate(model, fused, noise, use_checkpoint=True)
                wave = checkpoint(
                    lambda z: vae.decode(z.transpose(1, 2)).sample,
                    latent,
                    use_reentrant=False,
                )
                predictions.append(distribution.features(wave))
            loss, components = distribution.energy(
                distribution.features(target), *predictions
            )
        else:
            latent = generate(model, fused, noises[oid], use_checkpoint=True)
            wave = checkpoint(
                lambda z: vae.decode(z.transpose(1, 2)).sample,
                latent,
                use_reentrant=False,
            )
            loss, components = waveform_loss(target, wave)
        if not torch.isfinite(loss):
            raise ValueError("nonfinite decoded objective")
        loss.backward()
        norm = torch.nn.utils.clip_grad_norm_(
            adapter.parameters(), 1.0, error_if_nonfinite=True
        )
        optimizer.step()
        record = {
            "step": step + 1,
            "object_id": oid,
            "contact_index": index,
            "loss": loss.item(),
            "gradient_norm": norm.item(),
            **{k: v.item() for k, v in components.items()},
        }
        losses.append(record)
        if step == 0 or (step + 1) % 6 == 0:
            print(
                "decoded fit",
                record,
                "max_MB",
                torch.cuda.max_memory_allocated() / 2**20,
                flush=True,
            )
            cohort.save(args.output / "progress.json", {"rows": losses})
    if any(
        p._version != v or p.grad is not None or p.requires_grad for p, v in versions
    ):
        raise ValueError("frozen weights changed")
    save_file(
        {k: v.detach().cpu().contiguous() for k, v in adapter.state_dict().items()},
        args.output / "adapter.safetensors",
    )
    cohort.save(
        args.output / "fit.json",
        {
            "status": "REPORT_ONLY_SHARED_DECODED_AUDIO_FIT",
            "steps": 72,
            "batch_size": 1,
            "seed": 42,
            "learning_rate": 1e-4,
            "losses": losses,
            "objective": "two-sample symmetric FFT energy; full 50-step backpropagation"
            if args.energy_score
            else "FFT20Hz audible relative MRSTFT + .25*2ms envelope + .1*log RMS; full 50-step backpropagation",
            "energy_score_script_sha256": pilot.sha256(Path(distribution.__file__))
            if args.energy_score
            else None,
            "initialization": "zero residual on original published model, NOT rejected flow adapter",
            "noise": "two fresh independent draws per step; separate CUDA generator seed42"
            if args.energy_score
            else "fixed cached post-geometry RNG per object; no seed-generalization claim",
            "geometry_hashes": hashes,
            "input_sha256": pilot.sha256(args.data / "inputs.json"),
            "adapter_sha256": pilot.sha256(args.output / "adapter.safetensors"),
            "script_sha256": pilot.sha256(Path(__file__)),
            "weights": pilot.WEIGHTS,
            "train_rows": [
                {"object_id": o, "contact_index": c, "reference_sha256": h}
                for o, c, _, h in teachers
            ],
            "frozen_parameter_versions_unchanged": True,
            "original_latent_pcm_equivalence": True,
            "maximum_cuda_allocated_bytes": torch.cuda.max_memory_allocated(),
            "scope": "known pretraining TRAIN; local object-disjoint DEV; no runtime/physical acceptance",
        },
    )


if __name__ == "__main__":
    main()
